//! Handler dokumen perusahaan oleh HR (`/v1/corporate/documents`, F1-T102; US-1901 AC2, AC6; ADR-018,
//! keputusan user Q8 dan Q9). Setiap rute berkebijakan `policy::hr_documents` (HR perusahaan, status
//! perusahaan dibatasi lagi oleh use case); token realm `korporat`, status akun, sesi, dan keanggotaan aktif
//! ditegakkan middleware `authz` sebelum handler. Cakupan perusahaan (`authz::CorporateScope`) selalu dari
//! keanggotaan: tidak ada ID perusahaan atau dokumen di permintaan, dan isian ID yang tidak dikenal ditolak
//! `deny_unknown_fields` (422). Setiap rute terdaftar di harness isolasi `tests/corporate_isolation.rs` (H-18).
//!
//! Alur unggah tiga langkah (ADR-018): `POST /v1/corporate/document-upload-slots` → unggah langsung ke object
//! storage → `POST /v1/corporate/documents`. URL dan field unggah/unduh adalah rahasia berumur pendek: tidak
//! pernah dicatat (`Debug` manual menyamarkannya, KEPUTUSAN-122).

use super::api::{
    self as corporate, CompanyProfileView, CorporateDocuments, CorporateError, DocumentStatusView,
    DocumentsOverviewView,
};
use super::policy;
use crate::app::AppState;
use crate::http_support::{ApiFailure, ApiSuccess};
use crate::modules::platform::api::{DownloadTicket, UploadInstruction, UploadTicket};
use crate::openapi::ErrorEnvelope;
use authz::{CorporateScope, PolicyRouter};
use axum::Extension;
use axum::extract::rejection::JsonRejection;
use axum::extract::{Json, State};
use axum::http::HeaderValue;
use axum::http::header::CACHE_CONTROL;
use axum::response::{IntoResponse, Response};
use envelope::{FieldError, Success};
use foundation::{CorrelationId, Uuid};
use serde::{Deserialize, Serialize};
use std::fmt;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use utoipa::ToSchema;
use utoipa_axum::routes;

/// Pola rute untuk security log dan audit (bukan URL berisi ID).
const SLOT_ROUTE: &str = "/v1/corporate/document-upload-slots";
const CONFIRM_ROUTE: &str = "/v1/corporate/documents";
const DOWNLOAD_ROUTE: &str = "/v1/corporate/document-download-urls";
const RESUBMIT_ROUTE: &str = "/v1/corporate/verification-resubmissions";

pub(super) fn router() -> PolicyRouter<AppState> {
    PolicyRouter::new()
        .routes(policy::hr_documents(), routes!(list_documents))
        .routes(policy::hr_documents(), routes!(create_slot))
        .routes(policy::hr_documents(), routes!(confirm_document))
        .routes(policy::hr_documents(), routes!(download_url))
        .routes(policy::hr_documents(), routes!(resubmit))
}

fn deps(state: &AppState) -> CorporateDocuments<'_> {
    CorporateDocuments {
        pool: state.pool(),
        clock: state.clock(),
        files: state.files(),
    }
}

/// Nilai tersimpan tidak dapat dibentuk menjadi DTO (data rusak); dipetakan ke 500.
#[derive(Debug)]
struct DtoError;

fn ts(t: OffsetDateTime) -> Result<String, DtoError> {
    t.format(&Rfc3339).map_err(|_| DtoError)
}

/// Jenis dokumen wajib perusahaan.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub(super) enum DocumentKindDto {
    /// Dokumen NIB.
    Nib,
    /// Dokumen NPWP.
    Npwp,
    /// Bukti kewenangan pendaftar: surat kuasa atau surat penugasan dari direksi, PDF atau foto, bertanda
    /// tangan (tanda tangan diperiksa Verifikator, bukan sistem).
    AuthorityProof,
}

impl DocumentKindDto {
    fn as_str(self) -> &'static str {
        match self {
            Self::Nib => "nib",
            Self::Npwp => "npwp",
            Self::AuthorityProof => "authority_proof",
        }
    }

    fn parse(code: &str) -> Result<Self, DtoError> {
        match code {
            "nib" => Ok(Self::Nib),
            "npwp" => Ok(Self::Npwp),
            "authority_proof" => Ok(Self::AuthorityProof),
            _ => Err(DtoError),
        }
    }
}

/// Status verifikasi perusahaan yang dapat mengelola dokumen.
#[derive(Debug, Clone, Copy, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub(super) enum VerificationStatusDto {
    /// Menunggu verifikasi.
    PendingVerification,
    /// Ditolak Verifikator (HR dapat memperbaiki dokumen).
    Rejected,
}

impl VerificationStatusDto {
    fn parse(code: &str) -> Result<Self, DtoError> {
        match code {
            "pending_verification" => Ok(Self::PendingVerification),
            "rejected" => Ok(Self::Rejected),
            _ => Err(DtoError),
        }
    }
}

/// Status satu jenis dokumen wajib.
#[derive(Debug, Serialize, ToSchema)]
pub(super) struct DocumentStatusDto {
    kind: DocumentKindDto,
    /// Dokumen jenis ini sudah diunggah dan lolos validasi isi.
    uploaded: bool,
    /// Waktu dokumen aktif dicatat.
    #[schema(format = DateTime)]
    uploaded_at: Option<String>,
}

impl DocumentStatusDto {
    fn from_view(v: DocumentStatusView) -> Result<Self, DtoError> {
        Ok(Self {
            kind: DocumentKindDto::parse(v.kind)?,
            uploaded: v.uploaded,
            uploaded_at: v.uploaded_at.map(ts).transpose()?,
        })
    }
}

/// Profil perusahaan pemanggil (US-1901 AC6, F1-T102b): hanya perusahaan dari keanggotaan sesi. NPWP Terbatas
/// (DD A20): hanya untuk HR perusahaan itu sendiri; `Debug` manual tanpa isian.
#[derive(Serialize, ToSchema)]
pub(super) struct CompanyProfileDto {
    /// Nama badan usaha.
    #[schema(example = "PT Contoh Sehat Sejahtera")]
    legal_name: String,
    /// NIB (13 angka).
    #[schema(example = "1234567890123")]
    nib: String,
    /// NPWP badan (16 angka).
    #[schema(example = "0123456789012345")]
    npwp: String,
    /// Alamat kantor.
    office_address: String,
    /// Telepon kantor.
    office_phone: String,
    /// Waktu pengajuan terakhir (berubah saat diajukan ulang).
    #[schema(format = DateTime)]
    submitted_at: String,
}

impl fmt::Debug for CompanyProfileDto {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CompanyProfileDto").finish_non_exhaustive()
    }
}

impl CompanyProfileDto {
    fn from_view(v: CompanyProfileView) -> Result<Self, DtoError> {
        Ok(Self {
            legal_name: v.legal_name,
            nib: v.nib,
            npwp: v.npwp.into_inner(),
            office_address: v.office_address,
            office_phone: v.office_phone,
            submitted_at: ts(v.submitted_at)?,
        })
    }
}

/// Status verifikasi dan kelengkapan dokumen perusahaan pemanggil (US-1901 AC6).
#[derive(Debug, Serialize, ToSchema)]
pub(super) struct DocumentsOverviewDto {
    company: CompanyProfileDto,
    status: VerificationStatusDto,
    /// Putaran pengajuan berjalan.
    #[schema(example = 1, minimum = 1)]
    submission_round: i32,
    /// Kode alasan penolakan putaran berjalan (hanya bila `rejected`).
    #[schema(example = "authority_proof_insufficient")]
    rejection_reason: Option<String>,
    /// Ketiga jenis dokumen wajib, urut `nib`, `npwp`, `authority_proof`.
    documents: Vec<DocumentStatusDto>,
    /// Ketiganya sudah diunggah: pengajuan "siap diverifikasi" dan tampil di antrean Verifikator.
    complete: bool,
}

impl DocumentsOverviewDto {
    fn from_view(v: DocumentsOverviewView) -> Result<Self, DtoError> {
        Ok(Self {
            company: CompanyProfileDto::from_view(v.company)?,
            status: VerificationStatusDto::parse(v.status)?,
            submission_round: v.submission_round,
            rejection_reason: v.rejection_reason,
            documents: v
                .documents
                .into_iter()
                .map(DocumentStatusDto::from_view)
                .collect::<Result<_, _>>()?,
            complete: v.complete,
        })
    }
}

/// Bentuk dokumentasi respons status dokumen.
#[derive(ToSchema)]
#[allow(dead_code)] // hanya untuk pembangkitan skema
pub(super) struct DocumentsOverviewEnvelope {
    #[schema(example = "2000000")]
    response_code: String,
    #[schema(example = "Berhasil")]
    response_description: String,
    data: DocumentsOverviewDto,
    #[schema(nullable = false)]
    correlation_id: Option<String>,
}

/// Bentuk dokumentasi respons dokumen tercatat.
#[derive(ToSchema)]
#[allow(dead_code)] // hanya untuk pembangkitan skema
pub(super) struct DocumentsCreatedEnvelope {
    #[schema(example = "2010000")]
    response_code: String,
    #[schema(example = "Berhasil dibuat")]
    response_description: String,
    data: DocumentsOverviewDto,
    #[schema(nullable = false)]
    correlation_id: Option<String>,
}

/// Permintaan slot unggah dokumen.
#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct DocumentSlotRequest {
    /// Jenis yang dinyatakan: `image/jpeg`, `image/png`, atau `application/pdf` (isi tetap diperiksa saat
    /// konfirmasi; ekstensi dan header tidak dipercaya).
    #[schema(example = "application/pdf")]
    content_type: String,
    /// Ukuran berkas (byte): maksimal 5.242.880 untuk foto dan 10.485.760 untuk PDF (NFR-SEC-08).
    #[schema(example = 204800, minimum = 1, maximum = 10485760)]
    size_bytes: u64,
}

/// Pasangan nama-nilai (field formulir POST atau header PUT). Nilainya bagian dari tanda tangan URL unggah
/// (rahasia berumur pendek): tanpa `Debug` (KEPUTUSAN-122).
#[derive(Serialize, ToSchema)]
pub(super) struct CorporateNameValueDto {
    name: String,
    value: String,
}

/// Cara unggah langsung ke object storage.
#[derive(Debug, Clone, Copy, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub(super) enum CorporateUploadMethodDto {
    /// `multipart/form-data` POST ke `url` dengan semua `fields`, berkas sebagai field `file` terakhir.
    Post,
    /// `PUT` ke `url` dengan `headers` persis.
    Put,
}

/// Slot unggah yang diterbitkan. `url`, `fields`, dan `headers` rahasia berumur pendek: `Debug` manual
/// menyamarkannya (KEPUTUSAN-122).
#[derive(Serialize, ToSchema)]
pub(super) struct CorporateUploadSlotDto {
    #[schema(example = "0193a1b2-c3d4-7e5f-8a6b-7c8d9e0f1a2b")]
    upload_slot_id: String,
    #[schema(format = DateTime)]
    expires_at: String,
    method: CorporateUploadMethodDto,
    url: String,
    /// Field formulir (POST); kosong untuk PUT.
    fields: Vec<CorporateNameValueDto>,
    /// Header wajib (PUT); kosong untuk POST.
    headers: Vec<CorporateNameValueDto>,
}

impl fmt::Debug for CorporateUploadSlotDto {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CorporateUploadSlotDto")
            .field("upload_slot_id", &self.upload_slot_id)
            .field("expires_at", &self.expires_at)
            .field("method", &self.method)
            .field("url", &"[REDACTED]")
            .field("fields", &"[REDACTED]")
            .field("headers", &"[REDACTED]")
            .finish()
    }
}

fn pairs(v: Vec<(String, String)>) -> Vec<CorporateNameValueDto> {
    v.into_iter()
        .map(|(name, value)| CorporateNameValueDto { name, value })
        .collect()
}

impl CorporateUploadSlotDto {
    fn from_ticket(t: UploadTicket) -> Result<Self, DtoError> {
        let (method, url, fields, headers) = match t.instruction {
            UploadInstruction::Post { url, fields } => (
                CorporateUploadMethodDto::Post,
                url,
                pairs(fields),
                Vec::new(),
            ),
            UploadInstruction::Put { url, headers } => (
                CorporateUploadMethodDto::Put,
                url,
                Vec::new(),
                pairs(headers),
            ),
        };
        Ok(Self {
            upload_slot_id: t.upload_slot_id.hyphenated().to_string(),
            expires_at: ts(t.expires_at)?,
            method,
            url,
            fields,
            headers,
        })
    }
}

/// Bentuk dokumentasi respons slot unggah.
#[derive(ToSchema)]
#[allow(dead_code)] // hanya untuk pembangkitan skema
pub(super) struct CorporateUploadSlotEnvelope {
    #[schema(example = "2010000")]
    response_code: String,
    #[schema(example = "Berhasil dibuat")]
    response_description: String,
    data: CorporateUploadSlotDto,
    #[schema(nullable = false)]
    correlation_id: Option<String>,
}

/// Konfirmasi unggahan dokumen.
#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct DocumentConfirmRequest {
    document_kind: DocumentKindDto,
    /// ID slot dari `POST /v1/corporate/document-upload-slots` yang berkasnya sudah diunggah.
    #[schema(example = "0193a1b2-c3d4-7e5f-8a6b-7c8d9e0f1a2b")]
    upload_slot_id: String,
}

/// Permintaan URL unduh dokumen perusahaan sendiri.
#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct DocumentDownloadRequest {
    document_kind: DocumentKindDto,
}

/// URL unduh dokumen berumur pendek (ADR-018 butir 3). URL adalah rahasia: jangan dicatat.
// `Debug` manual menyamarkan URL (KEPUTUSAN-122).
#[derive(Serialize, ToSchema)]
pub(super) struct DocumentUrlDto {
    url: String,
    #[schema(format = DateTime)]
    expires_at: String,
    #[schema(example = "application/pdf")]
    mime_type: String,
    #[schema(example = 204_800)]
    size_bytes: i32,
}

impl fmt::Debug for DocumentUrlDto {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DocumentUrlDto")
            .field("url", &"[REDACTED]")
            .field("expires_at", &self.expires_at)
            .field("mime_type", &self.mime_type)
            .field("size_bytes", &self.size_bytes)
            .finish()
    }
}

impl DocumentUrlDto {
    fn from_ticket(t: DownloadTicket) -> Result<Self, DtoError> {
        Ok(Self {
            url: t.url,
            expires_at: ts(t.expires_at)?,
            mime_type: t.mime_type,
            size_bytes: t.size_bytes,
        })
    }
}

/// Bentuk dokumentasi respons URL unduh.
#[derive(ToSchema)]
#[allow(dead_code)] // hanya untuk pembangkitan skema
pub(super) struct DocumentUrlEnvelope {
    #[schema(example = "2000000")]
    response_code: String,
    #[schema(example = "Berhasil")]
    response_description: String,
    data: DocumentUrlDto,
    #[schema(nullable = false)]
    correlation_id: Option<String>,
}

fn fail(state: &AppState, cid: CorrelationId, e: CorporateError) -> Response {
    if let CorporateError::Internal(what) = &e {
        tracing::error!(error = *what, "corporate_documents_failed");
    }
    ApiFailure::new(e, cid, state.problem_type_base()).into_response()
}

/// Respons berisi status perusahaan atau URL dokumen tidak disimpan cache mana pun.
fn no_store(mut r: Response) -> Response {
    r.headers_mut()
        .insert(CACHE_CONTROL, HeaderValue::from_static("no-store"));
    r
}

fn dto_error(_: DtoError) -> CorporateError {
    CorporateError::Internal("corporate_documents_dto")
}

fn body_error(e: &JsonRejection) -> CorporateError {
    match e {
        JsonRejection::MissingJsonContentType(_) => CorporateError::UnsupportedMediaType,
        JsonRejection::JsonDataError(_) => CorporateError::Validation(vec![FieldError::new(
            "$",
            "invalid_format",
            "Isian tidak lengkap, bertipe salah, atau tidak dikenal",
        )]),
        _ => CorporateError::BadRequest,
    }
}

/// Profil perusahaan, status verifikasi, alasan penolakan, dan kelengkapan dokumen perusahaan pemanggil
/// (US-1901 AC6).
#[utoipa::path(
    get,
    path = "/v1/corporate/documents",
    tag = "corporate-documents",
    operation_id = "corporate_document_list",
    security(("bearer" = [])),
    responses(
        (status = 200, description = "Profil perusahaan pemanggil, status verifikasi, alasan penolakan, dan status dokumen.", body = DocumentsOverviewEnvelope, content_type = "application/json"),
        (status = 401, description = "Belum login, token tidak sah, sesi dicabut, akun atau perusahaan nonaktif, atau keanggotaan dicabut.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 409, description = "4090112: perusahaan sudah Aktif; dokumen hanya dikelola saat menunggu verifikasi atau ditolak.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 503, description = "Layanan tidak tersedia.", body = ErrorEnvelope, content_type = "application/problem+json")
    )
)]
async fn list_documents(
    State(state): State<AppState>,
    Extension(cid): Extension<CorrelationId>,
    scope: CorporateScope,
) -> Response {
    let r = corporate::corporate_documents_overview(&deps(&state), &scope)
        .await
        .and_then(|v| DocumentsOverviewDto::from_view(v).map_err(dto_error));
    match r {
        Ok(d) => no_store(ApiSuccess(Success::ok(d).with_correlation_id(cid)).into_response()),
        Err(e) => fail(&state, cid, e),
    }
}

/// Slot unggah dokumen perusahaan (langkah 1 dari 3): berkas diunggah langsung ke object storage, lalu
/// dikonfirmasi lewat `POST /v1/corporate/documents`. JPG/PNG paling besar 5.242.880 byte atau PDF paling besar
/// 10.485.760 byte.
#[utoipa::path(
    post,
    path = "/v1/corporate/document-upload-slots",
    tag = "corporate-documents",
    operation_id = "corporate_document_upload_slot_create",
    security(("bearer" = [])),
    request_body(content = DocumentSlotRequest, content_type = "application/json"),
    responses(
        (status = 201, description = "Slot unggah diterbitkan.", body = CorporateUploadSlotEnvelope, content_type = "application/json"),
        (status = 400, description = "Badan permintaan tidak dapat dibaca.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 401, description = "Belum login, token tidak sah, sesi dicabut, akun atau perusahaan nonaktif, atau keanggotaan dicabut.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 409, description = "4090112: perusahaan sudah Aktif; dokumen hanya dikelola saat menunggu verifikasi atau ditolak.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 413, description = "Ukuran melebihi 5.242.880 byte (foto) atau 10.485.760 byte (PDF).", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 415, description = "Jenis bukan image/jpeg, image/png, atau application/pdf, PDF belum didukung server, atau badan bukan JSON.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 422, description = "Isian tidak valid.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 429, description = "Kuota slot unggah per akun terlampaui (4290001).", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 503, description = "Layanan tidak tersedia.", body = ErrorEnvelope, content_type = "application/problem+json")
    )
)]
async fn create_slot(
    State(state): State<AppState>,
    Extension(cid): Extension<CorrelationId>,
    scope: CorporateScope,
    body: Result<Json<DocumentSlotRequest>, JsonRejection>,
) -> Response {
    let Json(req) = match body {
        Ok(b) => b,
        Err(e) => return fail(&state, cid, body_error(&e)),
    };
    let r = corporate::create_corporate_document_slot(
        &deps(&state),
        &scope,
        SLOT_ROUTE,
        req.content_type,
        req.size_bytes,
    )
    .await
    .and_then(|t| CorporateUploadSlotDto::from_ticket(t).map_err(dto_error));
    match r {
        Ok(d) => no_store(ApiSuccess(Success::created(d).with_correlation_id(cid)).into_response()),
        Err(e) => fail(&state, cid, e),
    }
}

/// Mengonfirmasi unggahan (langkah 3 dari 3): isi berkas divalidasi (jenis dari isi, ukuran, PDF diperiksa
/// konten aktif) lalu dicatat sebagai dokumen aktif jenis itu, menggantikan yang lama. Setelah NIB, NPWP, dan
/// bukti kewenangan lengkap, pengajuan tampil di antrean Verifikator.
#[utoipa::path(
    post,
    path = "/v1/corporate/documents",
    tag = "corporate-documents",
    operation_id = "corporate_document_confirm",
    security(("bearer" = [])),
    request_body(content = DocumentConfirmRequest, content_type = "application/json"),
    responses(
        (status = 201, description = "Dokumen tercatat; status terbaru semua dokumen.", body = DocumentsCreatedEnvelope, content_type = "application/json"),
        (status = 400, description = "Badan permintaan tidak dapat dibaca.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 401, description = "Belum login, token tidak sah, sesi dicabut, akun atau perusahaan nonaktif, atau keanggotaan dicabut.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 404, description = "Slot unggah tidak ada, kedaluwarsa, sudah dipakai, atau milik akun lain (KU-03).", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 409, description = "4090112: perusahaan sudah Aktif; dokumen hanya dikelola saat menunggu verifikasi atau ditolak.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 413, description = "Berkas melebihi 5.242.880 byte (foto) atau 10.485.760 byte (PDF).", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 415, description = "Jenis berkas menurut isinya bukan JPG, PNG, atau PDF aman (PDF berkonten aktif, terenkripsi, atau rusak ditolak), atau badan bukan JSON.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 422, description = "Jenis dokumen atau ID slot tidak valid, atau berkas belum diunggah.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 500, description = "Galat internal, termasuk pencatatan audit gagal (aksi dibatalkan).", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 503, description = "Layanan tidak tersedia.", body = ErrorEnvelope, content_type = "application/problem+json")
    )
)]
async fn confirm_document(
    State(state): State<AppState>,
    Extension(cid): Extension<CorrelationId>,
    scope: CorporateScope,
    body: Result<Json<DocumentConfirmRequest>, JsonRejection>,
) -> Response {
    let Json(req) = match body {
        Ok(b) => b,
        Err(e) => return fail(&state, cid, body_error(&e)),
    };
    let slot = match Uuid::parse_str(&req.upload_slot_id) {
        Ok(s) => s,
        Err(_) => {
            return fail(
                &state,
                cid,
                CorporateError::Validation(vec![FieldError::new(
                    "upload_slot_id",
                    "invalid_format",
                    "ID slot unggah tidak valid",
                )]),
            );
        }
    };
    let r = corporate::confirm_corporate_document(
        &deps(&state),
        &scope,
        CONFIRM_ROUTE,
        req.document_kind.as_str(),
        slot,
    )
    .await
    .and_then(|v| DocumentsOverviewDto::from_view(v).map_err(dto_error));
    match r {
        Ok(d) => no_store(ApiSuccess(Success::created(d).with_correlation_id(cid)).into_response()),
        Err(e) => fail(&state, cid, e),
    }
}

/// URL unduh berumur pendek (5 menit) untuk dokumen perusahaan pemanggil sendiri. Dicatat E-12 sebelum URL
/// terbit.
#[utoipa::path(
    post,
    path = "/v1/corporate/document-download-urls",
    tag = "corporate-documents",
    operation_id = "corporate_document_download_url_create",
    security(("bearer" = [])),
    request_body(content = DocumentDownloadRequest, content_type = "application/json"),
    responses(
        (status = 200, description = "URL unduh berumur pendek.", body = DocumentUrlEnvelope, content_type = "application/json"),
        (status = 400, description = "Badan permintaan tidak dapat dibaca.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 401, description = "Belum login, token tidak sah, sesi dicabut, akun atau perusahaan nonaktif, atau keanggotaan dicabut.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 404, description = "Dokumen jenis itu belum diunggah.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 409, description = "4090112: perusahaan sudah Aktif; dokumen hanya dikelola saat menunggu verifikasi atau ditolak.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 415, description = "Badan bukan application/json.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 422, description = "Jenis dokumen tidak valid.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 500, description = "Galat internal, termasuk pencatatan audit gagal (tanpa URL).", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 503, description = "Layanan tidak tersedia.", body = ErrorEnvelope, content_type = "application/problem+json")
    )
)]
async fn download_url(
    State(state): State<AppState>,
    Extension(cid): Extension<CorrelationId>,
    scope: CorporateScope,
    body: Result<Json<DocumentDownloadRequest>, JsonRejection>,
) -> Response {
    let Json(req) = match body {
        Ok(b) => b,
        Err(e) => return fail(&state, cid, body_error(&e)),
    };
    let r = corporate::corporate_document_download_url(
        &deps(&state),
        &scope,
        DOWNLOAD_ROUTE,
        req.document_kind.as_str(),
    )
    .await
    .and_then(|t| DocumentUrlDto::from_ticket(t).map_err(dto_error));
    match r {
        Ok(d) => no_store(ApiSuccess(Success::ok(d).with_correlation_id(cid)).into_response()),
        Err(e) => fail(&state, cid, e),
    }
}

/// Mengajukan ulang pengajuan yang ditolak (US-1902 AC3): setelah HR memperbaiki dokumen, status kembali
/// "Menunggu verifikasi" putaran berikutnya dan pengajuan masuk antrean Verifikator lagi. Tanpa isi permintaan:
/// perusahaan selalu dari keanggotaan sesi.
#[utoipa::path(
    post,
    path = "/v1/corporate/verification-resubmissions",
    tag = "corporate-documents",
    operation_id = "corporate_verification_resubmit",
    security(("bearer" = [])),
    responses(
        (status = 200, description = "Pengajuan diajukan ulang; status terbaru (Menunggu verifikasi, putaran naik satu).", body = DocumentsOverviewEnvelope, content_type = "application/json"),
        (status = 401, description = "Belum login, token tidak sah, sesi dicabut, akun atau perusahaan nonaktif, atau keanggotaan dicabut.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 409, description = "4090001: pengajuan tidak berstatus Ditolak (masih menunggu verifikasi, termasuk pengajuan ulang ganda); 4090112: perusahaan sudah Aktif; 4090115: dokumen NIB, NPWP, dan bukti kewenangan belum lengkap.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 500, description = "Galat internal, termasuk pencatatan audit gagal (aksi dibatalkan).", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 503, description = "Layanan tidak tersedia.", body = ErrorEnvelope, content_type = "application/problem+json")
    )
)]
async fn resubmit(
    State(state): State<AppState>,
    Extension(cid): Extension<CorrelationId>,
    scope: CorporateScope,
) -> Response {
    let r = corporate::resubmit_corporate_verification(&deps(&state), &scope, RESUBMIT_ROUTE)
        .await
        .and_then(|v| DocumentsOverviewDto::from_view(v).map_err(dto_error));
    match r {
        Ok(d) => no_store(ApiSuccess(Success::ok(d).with_correlation_id(cid)).into_response()),
        Err(e) => fail(&state, cid, e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use authz::Policy;

    #[test]
    fn route_constants_match_registered_paths_and_every_route_is_corporate_hr() {
        let router = router();
        let entries: Vec<_> = router.registry().iter().collect();
        assert_eq!(entries.len(), 5);
        let paths: Vec<String> = entries.iter().map(|((_, p), _)| p.to_string()).collect();
        for r in [SLOT_ROUTE, CONFIRM_ROUTE, DOWNLOAD_ROUTE, RESUBMIT_ROUTE] {
            assert!(paths.iter().any(|p| p == r), "{r}");
        }
        for ((_, path), policy) in entries {
            assert!(path.starts_with("/v1/corporate/"), "{path}");
            assert_eq!(
                policy,
                &Policy::corporate_hr_any_verification_state(),
                "{path}"
            );
        }
    }

    #[test]
    fn dto_debug_hides_signed_urls() {
        let slot = CorporateUploadSlotDto {
            upload_slot_id: "slot".to_string(),
            expires_at: "2030-01-01T00:00:00Z".to_string(),
            method: CorporateUploadMethodDto::Post,
            url: "https://rahasia.example/bucket".to_string(),
            fields: vec![CorporateNameValueDto {
                name: "x-amz-signature".to_string(),
                value: "tanda-tangan-rahasia".to_string(),
            }],
            headers: Vec::new(),
        };
        let d = format!("{slot:?}");
        assert!(!d.contains("rahasia"), "{d}");
        let url = DocumentUrlDto {
            url: "https://rahasia.example/obj?X-Amz-Signature=abc".to_string(),
            expires_at: "2030-01-01T00:00:00Z".to_string(),
            mime_type: "application/pdf".to_string(),
            size_bytes: 1,
        };
        assert!(!format!("{url:?}").contains("rahasia"));
        let profile = CompanyProfileDto {
            legal_name: "PT Rahasia".to_string(),
            nib: "1234567890123".to_string(),
            npwp: "0123456789012345".to_string(),
            office_address: "Jl. Rahasia".to_string(),
            office_phone: "021".to_string(),
            submitted_at: "2030-01-01T00:00:00Z".to_string(),
        };
        let d = format!("{profile:?}");
        assert!(
            !d.contains("0123456789012345") && !d.contains("Rahasia"),
            "{d}"
        );
    }

    #[test]
    fn document_kind_dto_round_trips_with_domain_codes() {
        for (dto, code) in [
            (DocumentKindDto::Nib, "nib"),
            (DocumentKindDto::Npwp, "npwp"),
            (DocumentKindDto::AuthorityProof, "authority_proof"),
        ] {
            assert_eq!(dto.as_str(), code);
            assert!(matches!(DocumentKindDto::parse(code), Ok(k) if k.as_str() == code));
        }
        assert!(DocumentKindDto::parse("ktp").is_err());
    }
}
