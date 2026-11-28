//! DTO endpoint verifikasi perusahaan (`/backoffice/v1`, F1-T104, US-1902). Field JSON snake_case bahasa
//! Inggris (coding-standards §2.5); waktu ISO 8601 dengan offset (ADR-017 butir 1); permintaan menolak field
//! yang tidak dikenal. NPWP (Terbatas, DD A20) hanya di detail, dengan `Debug` manual tanpa NPWP.

use super::api::{
    CorporateDecisionView, CorporateDetailView, CorporateQueueItemView, CorrectedFieldView,
    QueueReadiness, RejectionReasonView,
};
use crate::modules::platform::api::DownloadTicket;
use crate::openapi::MetaSchema;
use serde::{Deserialize, Serialize};
use std::fmt;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use utoipa::{IntoParams, ToSchema};

/// Nilai tersimpan tidak dapat dibentuk menjadi DTO (data rusak); dipetakan ke 500.
#[derive(Debug)]
pub(super) struct DtoError;

fn ts(t: OffsetDateTime) -> Result<String, DtoError> {
    t.format(&Rfc3339).map_err(|_| DtoError)
}

/// Halaman antrean.
#[derive(Debug, Deserialize, IntoParams)]
#[serde(deny_unknown_fields)]
#[into_params(parameter_in = Query)]
pub(super) struct QueueParams {
    /// Cursor dari `meta.next_cursor` halaman sebelumnya, dikirim apa adanya (opaque; memuat posisi
    /// `submitted_at` dan ID item terakhir, review T1 CR-035).
    #[param(
        pattern = "^[0-9]{1,19}\\.[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$"
    )]
    pub(super) cursor: Option<String>,
    /// Jumlah item (1-100, bawaan 20).
    #[param(minimum = 1, maximum = 100)]
    pub(super) limit: Option<i64>,
    /// Isi antrean menurut kelengkapan dokumen (bawaan `ready`, F1-T102).
    #[param(inline)]
    pub(super) readiness: Option<QueueReadinessParam>,
}

/// Pilihan isi antrean menurut kelengkapan dokumen NIB, NPWP, dan bukti kewenangan (F1-T102, keputusan user Q8).
#[derive(Debug, Clone, Copy, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub(super) enum QueueReadinessParam {
    /// Hanya pengajuan yang ketiga dokumennya lengkap ("siap diverifikasi"); bawaan.
    Ready,
    /// Hanya pengajuan yang dokumennya belum lengkap.
    Incomplete,
    /// Semua pengajuan menunggu verifikasi.
    All,
}

impl From<QueueReadinessParam> for QueueReadiness {
    fn from(p: QueueReadinessParam) -> Self {
        match p {
            QueueReadinessParam::Ready => Self::Ready,
            QueueReadinessParam::Incomplete => Self::Incomplete,
            QueueReadinessParam::All => Self::All,
        }
    }
}

/// Asal perusahaan.
#[derive(Debug, Clone, Copy, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub(super) enum OriginDto {
    /// Didaftarkan sendiri oleh HR (US-1901).
    SelfRegistered,
    /// Dibuatkan Verifikator (US-1903).
    CreatedByAdmin,
}

impl OriginDto {
    fn parse(code: &str) -> Result<Self, DtoError> {
        match code {
            "self_registered" => Ok(Self::SelfRegistered),
            "created_by_admin" => Ok(Self::CreatedByAdmin),
            _ => Err(DtoError),
        }
    }
}

/// Status perusahaan.
#[derive(Debug, Clone, Copy, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub(super) enum CorporateStatusDto {
    /// Menunggu verifikasi.
    PendingVerification,
    /// Aktif.
    Active,
    /// Ditolak (HR dapat memperbaiki dan mengajukan ulang).
    Rejected,
    /// Dinonaktifkan.
    Inactive,
}

impl CorporateStatusDto {
    fn parse(code: &str) -> Result<Self, DtoError> {
        match code {
            "pending_verification" => Ok(Self::PendingVerification),
            "active" => Ok(Self::Active),
            "rejected" => Ok(Self::Rejected),
            "inactive" => Ok(Self::Inactive),
            _ => Err(DtoError),
        }
    }
}

/// Jenis dokumen perusahaan.
#[derive(Debug, Clone, Copy, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub(super) enum DocumentKindDto {
    /// Bukti kewenangan pendaftar.
    AuthorityProof,
    /// Dokumen NIB.
    Nib,
    /// Dokumen NPWP.
    Npwp,
}

impl DocumentKindDto {
    fn parse(code: &str) -> Result<Self, DtoError> {
        match code {
            "authority_proof" => Ok(Self::AuthorityProof),
            "nib" => Ok(Self::Nib),
            "npwp" => Ok(Self::Npwp),
            _ => Err(DtoError),
        }
    }
}

/// Satu pengajuan di antrean (US-1902 AC1). Tanpa NPWP.
#[derive(Debug, Serialize, ToSchema)]
pub(super) struct QueueItemDto {
    #[schema(example = "0193a1b2-c3d4-7e5f-8a6b-7c8d9e0f1a2b")]
    pub(super) corporate_client_id: String,
    #[schema(example = "PT Sintetis Sejahtera")]
    pub(super) legal_name: String,
    #[schema(example = "9120000000001")]
    pub(super) nib: String,
    pub(super) origin: OriginDto,
    /// Putaran pengajuan (naik saat diajukan ulang setelah ditolak).
    #[schema(example = 1, minimum = 1)]
    pub(super) submission_round: i32,
    #[schema(format = DateTime)]
    pub(super) submitted_at: String,
    /// NIB, NPWP, dan bukti kewenangan sudah diunggah semua ("siap diverifikasi", F1-T102).
    pub(super) documents_complete: bool,
    /// Penanda "NIB sudah dipakai perusahaan lain": NIB yang sama dipakai perusahaan lain yang belum terhapus,
    /// status apa pun (CR-035). Pemeriksaan kepemilikan NIB menjadi tugas Verifikator.
    pub(super) nib_used_by_other_company: bool,
}

impl QueueItemDto {
    pub(super) fn from_view(v: CorporateQueueItemView) -> Result<Self, DtoError> {
        Ok(Self {
            corporate_client_id: v.corporate_client_id.hyphenated().to_string(),
            legal_name: v.legal_name,
            nib: v.nib,
            origin: OriginDto::parse(v.origin)?,
            submission_round: v.submission_round,
            submitted_at: ts(v.submitted_at)?,
            documents_complete: v.documents_complete,
            nib_used_by_other_company: v.nib_used_by_other_company,
        })
    }
}

/// Bentuk dokumentasi respons antrean.
#[derive(ToSchema)]
#[allow(dead_code)] // hanya untuk pembangkitan skema
pub(super) struct QueueEnvelope {
    #[schema(example = "2000000")]
    response_code: String,
    #[schema(example = "Berhasil")]
    response_description: String,
    data: Vec<QueueItemDto>,
    meta: MetaSchema,
    #[schema(nullable = false)]
    correlation_id: Option<String>,
}

/// Detail perusahaan untuk Verifikator (US-1902 AC1).
// `Debug` manual tanpa NPWP (Terbatas, DD A20; KEPUTUSAN-122).
#[derive(Serialize, ToSchema)]
pub(super) struct DetailDto {
    #[schema(example = "0193a1b2-c3d4-7e5f-8a6b-7c8d9e0f1a2b")]
    pub(super) corporate_client_id: String,
    #[schema(example = "PT Sintetis Sejahtera")]
    pub(super) legal_name: String,
    #[schema(example = "9120000000001")]
    pub(super) nib: String,
    /// NPWP (klasifikasi Terbatas): jangan disimpan atau dicatat di log klien.
    #[schema(example = "990000000000001")]
    pub(super) npwp: String,
    #[schema(example = "Jl. Sintetis No. 1, Jakarta")]
    pub(super) office_address: String,
    #[schema(example = "0210000001")]
    pub(super) office_phone: String,
    pub(super) status: CorporateStatusDto,
    pub(super) origin: OriginDto,
    #[schema(example = 1, minimum = 1)]
    pub(super) submission_round: i32,
    #[schema(format = DateTime)]
    pub(super) submitted_at: String,
    /// Jenis dokumen aktif; dibuka lewat endpoint URL dokumen. Lengkap bila ketiganya ada (`nib`, `npwp`,
    /// `authority_proof`); persetujuan ditolak `4090113` bila belum (F1-T102).
    pub(super) documents: Vec<DocumentKindDto>,
    /// Token versi pengajuan (putaran + dokumen aktif, KEPUTUSAN-415); kirim kembali apa adanya saat menyetujui atau
    /// menolak (`4090114` bila dokumen diganti atau data dikoreksi dan diajukan ulang sejak detail dilihat).
    /// Nilai apa pun yang tidak sama dengan token saat ini (termasuk kosong atau terlalu panjang) dijawab `409`.
    #[schema(example = "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08")]
    pub(super) documents_version: String,
    /// Kode alasan penolakan putaran berjalan (hanya bila `rejected`).
    #[schema(example = "authority_proof_insufficient")]
    pub(super) rejection_reason: Option<String>,
    /// Waktu keputusan putaran berjalan, bila sudah diputuskan.
    #[schema(format = DateTime)]
    pub(super) decided_at: Option<String>,
    /// Setuju/tolak tersedia (masih menunggu verifikasi).
    pub(super) can_decide: bool,
    /// Penanda "NIB sudah dipakai perusahaan lain" (status apa pun, CR-035). Persetujuan dijawab `4090116` bila
    /// NIB sudah dipakai perusahaan Aktif lain.
    pub(super) nib_used_by_other_company: bool,
    /// Penanda "Data dikoreksi" (CR-040): field yang dikoreksi HR pada putaran berjalan dibandingkan putaran
    /// sebelumnya, beserta nilai sebelumnya. Kosong bila putaran ini pengajuan pertama, pengajuan ulang tanpa
    /// koreksi (dokumen saja), atau koreksi tanpa nilai yang benar-benar berubah.
    pub(super) corrected_fields: Vec<CorrectedFieldDto>,
}

/// Field data perusahaan yang dapat dikoreksi HR (CR-036).
#[derive(Debug, Clone, Copy, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub(super) enum CorrectableFieldDto {
    /// Nama badan usaha.
    LegalName,
    /// NIB.
    Nib,
    /// NPWP.
    Npwp,
    /// Alamat kantor.
    OfficeAddress,
    /// Telepon kantor.
    OfficePhone,
}

impl CorrectableFieldDto {
    fn parse(code: &str) -> Result<Self, DtoError> {
        match code {
            "legal_name" => Ok(Self::LegalName),
            "nib" => Ok(Self::Nib),
            "npwp" => Ok(Self::Npwp),
            "office_address" => Ok(Self::OfficeAddress),
            "office_phone" => Ok(Self::OfficePhone),
            _ => Err(DtoError),
        }
    }
}

/// Satu field yang dikoreksi HR beserta nilai putaran sebelumnya (CR-040).
// `Debug` manual tanpa nilai (KEPUTUSAN-122).
#[derive(Serialize, ToSchema)]
pub(super) struct CorrectedFieldDto {
    pub(super) field: CorrectableFieldDto,
    /// Nilai putaran sebelumnya. NIB dan NPWP selalu termasking (`************` + 4 karakter terakhir); nilai
    /// lengkap sebelumnya tidak disimpan.
    #[schema(example = "PT Sintetis Lama")]
    pub(super) previous_value: String,
}

impl fmt::Debug for CorrectedFieldDto {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CorrectedFieldDto")
            .field("field", &self.field)
            .finish_non_exhaustive()
    }
}

impl CorrectedFieldDto {
    fn from_view(v: CorrectedFieldView) -> Result<Self, DtoError> {
        Ok(Self {
            field: CorrectableFieldDto::parse(v.field)?,
            previous_value: v.previous_value,
        })
    }
}

impl fmt::Debug for DetailDto {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DetailDto")
            .field("corporate_client_id", &self.corporate_client_id)
            .field("npwp", &"[REDACTED]")
            .field("status", &self.status)
            .field("submission_round", &self.submission_round)
            .field("can_decide", &self.can_decide)
            .finish_non_exhaustive()
    }
}

impl DetailDto {
    pub(super) fn from_view(v: CorporateDetailView) -> Result<Self, DtoError> {
        Ok(Self {
            corporate_client_id: v.corporate_client_id.hyphenated().to_string(),
            legal_name: v.legal_name,
            nib: v.nib,
            npwp: v.npwp.into_inner(),
            office_address: v.office_address,
            office_phone: v.office_phone,
            status: CorporateStatusDto::parse(v.status)?,
            origin: OriginDto::parse(v.origin)?,
            submission_round: v.submission_round,
            submitted_at: ts(v.submitted_at)?,
            documents: v
                .documents
                .into_iter()
                .map(DocumentKindDto::parse)
                .collect::<Result<_, _>>()?,
            documents_version: v.documents_version,
            rejection_reason: v.rejection_reason,
            decided_at: v.decided_at.map(ts).transpose()?,
            can_decide: v.can_decide,
            nib_used_by_other_company: v.nib_used_by_other_company,
            corrected_fields: v
                .corrected_fields
                .into_iter()
                .map(CorrectedFieldDto::from_view)
                .collect::<Result<_, _>>()?,
        })
    }
}

/// Bentuk dokumentasi respons detail.
#[derive(ToSchema)]
#[allow(dead_code)] // hanya untuk pembangkitan skema
pub(super) struct DetailEnvelope {
    #[schema(example = "2000000")]
    response_code: String,
    #[schema(example = "Berhasil")]
    response_description: String,
    data: DetailDto,
    #[schema(nullable = false)]
    correlation_id: Option<String>,
}

/// URL unduh dokumen berumur pendek (ADR-018 butir 3). URL adalah rahasia: jangan dicatat.
// `Debug` manual menyamarkan URL (KEPUTUSAN-122).
#[derive(Serialize, ToSchema)]
pub(super) struct DocumentUrlDto {
    pub(super) url: String,
    #[schema(format = DateTime)]
    pub(super) expires_at: String,
    #[schema(example = "image/jpeg")]
    pub(super) mime_type: String,
    #[schema(example = 204_800)]
    pub(super) size_bytes: i32,
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
    pub(super) fn from_ticket(t: DownloadTicket) -> Result<Self, DtoError> {
        Ok(Self {
            url: t.url,
            expires_at: ts(t.expires_at)?,
            mime_type: t.mime_type,
            size_bytes: t.size_bytes,
        })
    }
}

/// Bentuk dokumentasi respons URL dokumen.
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

/// Penolakan (US-1902 AC3).
#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct RejectRequest {
    /// Kode alasan dari `GET /backoffice/v1/corporate-verification-reasons`.
    #[schema(example = "authority_proof_insufficient", max_length = 64)]
    pub(super) reason_code: String,
    /// `documents_version` dari detail yang dilihat Verifikator; nilai yang berbeda dari token saat ini dijawab `409`.
    #[schema(example = "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08")]
    pub(super) documents_version: String,
}

/// Persetujuan (US-1902 AC2), terikat versi dokumen yang dilihat.
#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct ApproveRequest {
    /// `documents_version` dari detail yang dilihat Verifikator; nilai yang berbeda dari token saat ini dijawab `409`.
    #[schema(example = "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08")]
    pub(super) documents_version: String,
}

/// Hasil keputusan.
#[derive(Debug, Serialize, ToSchema)]
pub(super) struct DecisionDto {
    #[schema(example = "0193a1b2-c3d4-7e5f-8a6b-7c8d9e0f1a2b")]
    pub(super) corporate_client_id: String,
    pub(super) status: CorporateStatusDto,
    #[schema(example = 1, minimum = 1)]
    pub(super) submission_round: i32,
    /// Kode alasan (hanya penolakan).
    #[schema(example = "authority_proof_insufficient")]
    pub(super) rejection_reason: Option<String>,
}

impl DecisionDto {
    pub(super) fn from_view(v: CorporateDecisionView) -> Result<Self, DtoError> {
        Ok(Self {
            corporate_client_id: v.corporate_client_id.hyphenated().to_string(),
            status: CorporateStatusDto::parse(v.status)?,
            submission_round: v.submission_round,
            rejection_reason: v.rejection_reason.map(str::to_string),
        })
    }
}

/// Bentuk dokumentasi respons keputusan.
#[derive(ToSchema)]
#[allow(dead_code)] // hanya untuk pembangkitan skema
pub(super) struct DecisionEnvelope {
    #[schema(example = "2000000")]
    response_code: String,
    #[schema(example = "Berhasil")]
    response_description: String,
    data: DecisionDto,
    #[schema(nullable = false)]
    correlation_id: Option<String>,
}

/// Satu alasan penolakan (data migrasi, US-1902 AC3).
#[derive(Debug, Serialize, ToSchema)]
pub(super) struct RejectionReasonDto {
    #[schema(example = "authority_proof_insufficient")]
    pub(super) code: String,
    #[schema(example = "Bukti kewenangan pendaftar tidak memadai")]
    pub(super) label: String,
}

impl RejectionReasonDto {
    pub(super) fn from_view(v: RejectionReasonView) -> Self {
        Self {
            code: v.code.to_string(),
            label: v.label,
        }
    }
}

/// Bentuk dokumentasi respons daftar alasan.
#[derive(ToSchema)]
#[allow(dead_code)] // hanya untuk pembangkitan skema
pub(super) struct RejectionReasonsEnvelope {
    #[schema(example = "2000000")]
    response_code: String,
    #[schema(example = "Berhasil")]
    response_description: String,
    data: Vec<RejectionReasonDto>,
    #[schema(nullable = false)]
    correlation_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::corporate::api::CorporateDetailView;
    use foundation::{Sensitive, Uuid};

    // trace: DD A20 (NPWP Terbatas), KEPUTUSAN-122, ADR-005: NPWP dan URL dokumen tidak pernah ada di Debug
    #[test]
    fn debug_redacts_npwp_and_document_url() {
        let npwp = "990000000000777";
        let view = CorporateDetailView {
            corporate_client_id: Uuid::nil(),
            legal_name: "PT Sintetis".to_string(),
            nib: "9120000000001".to_string(),
            npwp: Sensitive::new(npwp.to_string()),
            office_address: "Jl. Sintetis".to_string(),
            office_phone: "0210000001".to_string(),
            status: "pending_verification",
            origin: "self_registered",
            submission_round: 1,
            submitted_at: OffsetDateTime::UNIX_EPOCH,
            documents: vec!["nib"],
            documents_version: "0".repeat(64),
            rejection_reason: None,
            decided_at: None,
            can_decide: true,
            nib_used_by_other_company: false,
            corrected_fields: vec![crate::modules::corporate::api::CorrectedFieldView {
                field: "npwp",
                previous_value: "************0777".to_string(),
            }],
        };
        let d = format!("{view:?}");
        assert!(!d.contains(npwp), "NPWP bocor di Debug view: {d}");
        let dto = DetailDto::from_view(view).unwrap();
        assert_eq!(dto.npwp, npwp, "NPWP tetap terkirim ke Verifikator");
        let d = format!("{dto:?}");
        assert!(!d.contains(npwp), "NPWP bocor di Debug DTO: {d}");
        assert!(d.contains("[REDACTED]"));
        assert!(
            !d.contains("0777"),
            "nilai sebelumnya bocor di Debug DTO: {d}"
        );

        let url = DocumentUrlDto {
            url: "https://s3.example/obj?X-Amz-Signature=tandatangan".to_string(),
            expires_at: "2026-11-24T00:05:00Z".to_string(),
            mime_type: "image/jpeg".to_string(),
            size_bytes: 1,
        };
        let d = format!("{url:?}");
        assert!(!d.contains("tandatangan") && !d.contains("https://"), "{d}");
    }
}
