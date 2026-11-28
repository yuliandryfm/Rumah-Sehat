//! Handler pendaftaran perusahaan mandiri tanpa login (F1-T101; US-1901): `POST
//! /v1/public/corporate-registrations` dan `GET /v1/public/corporate-registrations/consent-text`
//! (`PublicReason::CorporateRegistration`, KEPUTUSAN-323). Handler menegakkan batas laju (sebelum membaca
//! isian) dan menyamakan waktu respons jawaban seragam (ADR-030 butir 7), lalu memanggil satu fungsi `api`.
//! Teks galat pustaka (serde, sqlx) tidak pernah diteruskan karena isian memuat NPWP (Terbatas), nama, dan
//! email.

use super::api::{self as corporate, CorporateError, NewCorporateRegistration, RegistrationDeps};
use super::policy;
use crate::app::AppState;
use crate::http_support::{ApiFailure, ApiSuccess, ClientAddr};
use crate::openapi::ErrorEnvelope;
use crate::rate_limit::Verdict;
use authz::PolicyRouter;
use axum::Extension;
use axum::extract::rejection::JsonRejection;
use axum::extract::{Json, State};
use axum::http::HeaderValue;
use axum::http::header::CACHE_CONTROL;
use axum::response::{IntoResponse, Response};
use envelope::{FieldError, Success};
use foundation::{CorrelationId, Sensitive, Uuid};
use observability::security;
use serde::{Deserialize, Serialize};
use std::fmt;
use time::format_description::well_known::Rfc3339;
use tokio::time::Instant;
use utoipa::ToSchema;
use utoipa_axum::routes;

/// Pola rute untuk security log.
const REGISTER_ROUTE: &str = "/v1/public/corporate-registrations";
const CONSENT_ROUTE: &str = "/v1/public/corporate-registrations/consent-text";

/// Pesan jawaban seragam untuk pendaftaran baru, NIB terdaftar, dan email terdaftar (KEPUTUSAN-324). Tidak
/// membedakan ketiganya; memuat arahan Layanan Pelanggan (US-1901 AC7) tanpa menyebut sebab.
pub(super) const UNIFORM_MESSAGE: &str = "Permintaan pendaftaran diterima. Periksa email Anda untuk melanjutkan. \
Jika Anda membutuhkan bantuan, hubungi Layanan Pelanggan.";

pub(super) fn router() -> PolicyRouter<AppState> {
    PolicyRouter::new()
        .routes(policy::registration(), routes!(register_corporate_client))
        .routes(policy::registration(), routes!(registration_consent_text))
}

/// Isian pendaftaran perusahaan. Tanpa password: password diatur lewat tautan aktivasi email (KEPUTUSAN-303).
#[derive(Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct CorporateRegistrationRequest {
    /// Nama badan usaha (maks 200 karakter).
    #[schema(
        example = "PT Contoh Sintetis",
        min_length = 1,
        max_length = 200,
        pattern = "^[^\\s\\x00-\\x1f\\x7f-\\x9f\\u200b-\\u200f\\u202a-\\u202e\\u2066-\\u2069\\ufeff][^\\x00-\\x1f\\x7f-\\x9f\\u200b-\\u200f\\u202a-\\u202e\\u2066-\\u2069\\ufeff]*$"
    )]
    legal_name: String,
    /// NIB: tepat 13 angka (Peraturan BKPM 3/2021). Keaslian diperiksa Verifikator.
    #[schema(
        example = "1234567890123",
        min_length = 13,
        max_length = 13,
        pattern = "^[0-9]{13}$"
    )]
    nib: String,
    /// NPWP badan: tepat 16 angka (PMK 112/PMK.03/2022). NPWP 15 angka lama: tambahkan angka 0 di depan
    /// sebelum dikirim.
    #[schema(
        example = "0123456789012345",
        min_length = 16,
        max_length = 16,
        pattern = "^[0-9]{16}$"
    )]
    npwp: String,
    /// Alamat kantor (maks 500 karakter).
    #[schema(
        example = "Jl. Contoh No. 1, Jakarta",
        min_length = 1,
        max_length = 500,
        pattern = "^[^\\s\\x00-\\x1f\\x7f-\\x9f\\u200b-\\u200f\\u202a-\\u202e\\u2066-\\u2069\\ufeff][^\\x00-\\x1f\\x7f-\\x9f\\u200b-\\u200f\\u202a-\\u202e\\u2066-\\u2069\\ufeff]*$"
    )]
    office_address: String,
    /// Telepon kantor (maks 32 karakter).
    #[schema(
        example = "021-0000000",
        min_length = 1,
        max_length = 32,
        pattern = "^[^\\s\\x00-\\x1f\\x7f-\\x9f\\u200b-\\u200f\\u202a-\\u202e\\u2066-\\u2069\\ufeff][^\\x00-\\x1f\\x7f-\\x9f\\u200b-\\u200f\\u202a-\\u202e\\u2066-\\u2069\\ufeff]*$"
    )]
    office_phone: String,
    /// Nama HR pendaftar (maks 120 karakter).
    #[schema(
        example = "Nama HR Sintetis",
        min_length = 1,
        max_length = 120,
        pattern = "^[^\\s\\x00-\\x1f\\x7f-\\x9f\\u200b-\\u200f\\u202a-\\u202e\\u2066-\\u2069\\ufeff][^\\x00-\\x1f\\x7f-\\x9f\\u200b-\\u200f\\u202a-\\u202e\\u2066-\\u2069\\ufeff]*$"
    )]
    hr_full_name: String,
    /// Jabatan HR pendaftar (maks 100 karakter).
    #[schema(
        example = "Manajer SDM",
        min_length = 1,
        max_length = 100,
        pattern = "^[^\\s\\x00-\\x1f\\x7f-\\x9f\\u200b-\\u200f\\u202a-\\u202e\\u2066-\\u2069\\ufeff][^\\x00-\\x1f\\x7f-\\x9f\\u200b-\\u200f\\u202a-\\u202e\\u2066-\\u2069\\ufeff]*$"
    )]
    hr_job_title: String,
    /// Email HR pendaftar: penerima tautan aktivasi (verifikasi email, password, TOTP).
    #[schema(
        example = "hr@perusahaan.example",
        min_length = 5,
        max_length = 254,
        pattern = "^[A-Za-z0-9._%+-]{1,64}@[A-Za-z0-9-]+(\\.[A-Za-z0-9-]+)+$"
    )]
    hr_email: String,
    /// Versi teks persetujuan yang ditampilkan (dari `GET .../consent-text`).
    #[schema(
        example = "01a2008d-7400-7000-8000-000000000b01",
        pattern = "^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$"
    )]
    consent_text_version_id: String,
    /// Butir persetujuan yang dicentang; butir wajib harus ada.
    #[schema(example = json!(["01a2008d-7400-7000-8000-000000000b02"]), max_items = 20)]
    consent_item_ids: Vec<String>,
}

impl fmt::Debug for CorporateRegistrationRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CorporateRegistrationRequest")
            .finish_non_exhaustive()
    }
}

/// Jawaban seragam pendaftaran.
#[derive(Debug, Serialize, ToSchema)]
pub(super) struct CorporateRegistrationAcceptedDto {
    /// Selalu `processing`, juga untuk NIB atau email yang sudah terdaftar (anti-enumerasi).
    #[schema(example = "processing")]
    status: &'static str,
    /// Pesan seragam untuk layar "Periksa email Anda".
    message: &'static str,
}

/// Bentuk dokumentasi respons pendaftaran.
#[derive(ToSchema)]
#[allow(dead_code)] // hanya untuk pembangkitan skema
pub(super) struct CorporateRegistrationAcceptedEnvelope {
    #[schema(example = "2020000")]
    response_code: String,
    #[schema(example = "Diterima")]
    response_description: String,
    data: CorporateRegistrationAcceptedDto,
    #[schema(nullable = false)]
    correlation_id: Option<String>,
}

/// Satu bagian teks persetujuan.
#[derive(Debug, Serialize, ToSchema)]
pub(super) struct RegistrationConsentSectionDto {
    #[schema(example = "purpose")]
    code: String,
    #[schema(example = "(b) Tujuan pemrosesan")]
    title: String,
    /// Isi bagian, teks biasa (bukan HTML).
    body: String,
}

/// Kontak Ruang Sehat.
#[derive(Debug, Serialize, ToSchema)]
pub(super) struct RegistrationConsentContactDto {
    title: String,
    body: String,
}

/// Butir persetujuan: satu kotak centang yang tidak tercentang otomatis.
#[derive(Debug, Serialize, ToSchema)]
pub(super) struct RegistrationConsentItemDto {
    #[schema(example = "01a2008d-7400-7000-8000-000000000b02")]
    consent_item_id: String,
    #[schema(example = "corporate_hr_data_processing")]
    item_code: String,
    label: String,
    is_primary: bool,
    /// Wajib dicentang untuk mendaftar.
    is_required: bool,
    sort_order: i16,
}

/// Versi teks persetujuan HR yang berlaku.
#[derive(Debug, Serialize, ToSchema)]
pub(super) struct RegistrationConsentTextDto {
    #[schema(
        example = "01a2008d-7400-7000-8000-000000000b01",
        pattern = "^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$"
    )]
    consent_text_version_id: String,
    #[schema(example = "corporate_hr_registration")]
    consent_type: String,
    #[schema(example = "0.1-draf")]
    version_label: String,
    #[schema(format = DateTime, example = "2026-11-26T00:00:01Z")]
    effective_from: String,
    /// Penanda teks draf (`null` untuk teks final yang disetujui legal dan Petugas PDP).
    draft_notice: Option<String>,
    sections: Vec<RegistrationConsentSectionDto>,
    contact: RegistrationConsentContactDto,
    items: Vec<RegistrationConsentItemDto>,
}

/// Bentuk dokumentasi respons teks persetujuan.
#[derive(ToSchema)]
#[allow(dead_code)] // hanya untuk pembangkitan skema
pub(super) struct RegistrationConsentTextEnvelope {
    #[schema(example = "2000000")]
    response_code: String,
    #[schema(example = "Berhasil")]
    response_description: String,
    data: RegistrationConsentTextDto,
    #[schema(nullable = false)]
    correlation_id: Option<String>,
}

impl RegistrationConsentTextDto {
    fn from_text(
        t: crate::modules::privacy::api::ActiveConsentText,
    ) -> Result<Self, time::error::Format> {
        Ok(Self {
            consent_text_version_id: t.version_id.to_string(),
            consent_type: t.consent_type,
            version_label: t.version_label,
            effective_from: t.effective_from.format(&Rfc3339)?,
            draft_notice: t.content.draft_notice,
            sections: t
                .content
                .sections
                .into_iter()
                .map(|s| RegistrationConsentSectionDto {
                    code: s.code,
                    title: s.title,
                    body: s.body,
                })
                .collect(),
            contact: RegistrationConsentContactDto {
                title: t.content.contact.title,
                body: t.content.contact.body,
            },
            items: t
                .items
                .into_iter()
                .map(|i| RegistrationConsentItemDto {
                    consent_item_id: i.id.to_string(),
                    item_code: i.item_code,
                    label: i.label,
                    is_primary: i.is_primary,
                    is_required: i.is_required,
                    sort_order: i.sort_order,
                })
                .collect(),
        })
    }
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

fn fail(state: &AppState, cid: CorrelationId, e: CorporateError) -> Response {
    if let CorporateError::Internal(what) = &e {
        tracing::error!(error = *what, "corporate_registration_failed");
    }
    let mut r = ApiFailure::new(e, cid, state.problem_type_base()).into_response();
    r.headers_mut()
        .insert(CACHE_CONTROL, HeaderValue::from_static("no-store"));
    r
}

/// Batas laju per sumber dan global; penolakan pertama per jendela dicatat di security log.
fn limited(
    limiter: &crate::rate_limit::RateLimiter,
    addr: &ClientAddr,
    route: &'static str,
) -> bool {
    if let Verdict::Limited { first } = limiter.check(addr.0) {
        if first {
            security::rate_limited(None, route);
        }
        return true;
    }
    false
}

fn parse_uuid(name: &str, raw: &str) -> Result<Uuid, CorporateError> {
    Uuid::parse_str(raw.trim()).map_err(|_| {
        CorporateError::Validation(vec![FieldError::new(
            name,
            "invalid",
            "Harus berupa ID yang sah",
        )])
    })
}

fn to_input(req: CorporateRegistrationRequest) -> Result<NewCorporateRegistration, CorporateError> {
    let consent_text_version_id =
        parse_uuid("consent_text_version_id", &req.consent_text_version_id)?;
    let consent_item_ids = req
        .consent_item_ids
        .iter()
        .map(|s| parse_uuid("consent_item_ids", s))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(NewCorporateRegistration {
        legal_name: req.legal_name,
        nib: req.nib,
        npwp: Sensitive::new(req.npwp),
        office_address: req.office_address,
        office_phone: req.office_phone,
        hr_full_name: Sensitive::new(req.hr_full_name),
        hr_job_title: req.hr_job_title,
        hr_email: Sensitive::new(req.hr_email),
        consent_text_version_id,
        consent_item_ids,
    })
}

/// Mendaftarkan perusahaan mandiri tanpa login (US-1901). Jawaban `202` seragam untuk pendaftaran baru, NIB
/// terdaftar, dan email terdaftar, dengan waktu respons yang disamakan; password tidak diminta (diatur lewat
/// tautan aktivasi email).
#[utoipa::path(
    post,
    path = "/v1/public/corporate-registrations",
    tag = "corporate-registration",
    operation_id = "corporate_registration_submit",
    request_body(content = CorporateRegistrationRequest, content_type = "application/json"),
    responses(
        (status = 202, description = "Permintaan diterima; jawaban sama untuk pendaftaran baru dan NIB atau email yang sudah terdaftar.", body = CorporateRegistrationAcceptedEnvelope, content_type = "application/json"),
        (status = 400, description = "Badan permintaan tidak dapat dibaca.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 409, description = "4090106: teks persetujuan telah berubah; muat ulang teks lalu setujui kembali.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 415, description = "Badan bukan application/json.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 422, description = "Isian tidak valid per field, atau persetujuan wajib tidak dicentang.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 429, description = "Batas laju terlampaui.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 500, description = "Galat internal, termasuk pencatatan audit gagal (aksi dibatalkan).", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 503, description = "Teks persetujuan belum tersedia atau layanan tidak tersedia.", body = ErrorEnvelope, content_type = "application/problem+json")
    )
)]
async fn register_corporate_client(
    State(state): State<AppState>,
    Extension(cid): Extension<CorrelationId>,
    addr: ClientAddr,
    body: Result<Json<CorporateRegistrationRequest>, JsonRejection>,
) -> Response {
    let started = Instant::now();
    if limited(&state.registration().limiter, &addr, REGISTER_ROUTE) {
        return fail(&state, cid, CorporateError::RateLimited);
    }
    let Json(req) = match body {
        Ok(b) => b,
        Err(e) => return fail(&state, cid, body_error(&e)),
    };
    let input = match to_input(req) {
        Ok(i) => i,
        Err(e) => return fail(&state, cid, e),
    };
    let deps = RegistrationDeps {
        pool: state.pool(),
        clock: state.clock(),
        drafts: state.consent_draft_policy(),
        correlation_id: cid,
        client_ip: addr.0,
    };
    match corporate::register_corporate_client(&deps, &input).await {
        Ok(()) => {
            // Semua jalur sukses (baru, NIB atau email terdaftar) ditahan sampai waktu minimum yang sama.
            tokio::time::sleep_until(started + state.registration().min_response).await;
            let mut r = ApiSuccess(
                Success::accepted(CorporateRegistrationAcceptedDto {
                    status: "processing",
                    message: UNIFORM_MESSAGE,
                })
                .with_correlation_id(cid),
            )
            .into_response();
            r.headers_mut()
                .insert(CACHE_CONTROL, HeaderValue::from_static("no-store"));
            r
        }
        Err(e) => fail(&state, cid, e),
    }
}

/// Teks persetujuan HR yang berlaku untuk layar pendaftaran (US-1901 AC1): tujuh bagian, kontak, dan butir.
/// Tanpa login, dengan batas laju sendiri (terpisah dari POST). Teks draf tidak disajikan di production.
#[utoipa::path(
    get,
    path = "/v1/public/corporate-registrations/consent-text",
    tag = "corporate-registration",
    operation_id = "corporate_registration_consent_text",
    responses(
        (status = 200, description = "Versi teks yang berlaku.", body = RegistrationConsentTextEnvelope, content_type = "application/json"),
        (status = 429, description = "Batas laju terlampaui.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 500, description = "Galat tak terduga.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 503, description = "Belum ada versi teks final yang berlaku atau layanan tidak tersedia.", body = ErrorEnvelope, content_type = "application/problem+json")
    )
)]
async fn registration_consent_text(
    State(state): State<AppState>,
    Extension(cid): Extension<CorrelationId>,
    addr: ClientAddr,
) -> Response {
    if limited(
        &state.registration().consent_text_limiter,
        &addr,
        CONSENT_ROUTE,
    ) {
        return fail(&state, cid, CorporateError::RateLimited);
    }
    let r = corporate::corporate_registration_consent_text(
        state.pool(),
        state.clock(),
        state.consent_draft_policy(),
    )
    .await
    .and_then(|t| {
        RegistrationConsentTextDto::from_text(t)
            .map_err(|_| CorporateError::Internal("timestamp_format"))
    });
    match r {
        Ok(d) => {
            let mut r = ApiSuccess(Success::ok(d).with_correlation_id(cid)).into_response();
            r.headers_mut()
                .insert(CACHE_CONTROL, HeaderValue::from_static("no-store"));
            r
        }
        Err(e) => fail(&state, cid, e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use authz::{Policy, PublicReason};

    // trace: ADR-012 butir 3 (amandemen KEPUTUSAN-323), US-1901
    #[test]
    fn routes_are_public_with_the_corporate_registration_reason() {
        let router = router();
        let entries: Vec<_> = router.registry().iter().collect();
        assert_eq!(entries.len(), 2);
        for ((method, path), p) in entries {
            assert!(path.starts_with(REGISTER_ROUTE), "{path}");
            assert!(method == "POST" || method == "GET");
            assert_eq!(
                p,
                &Policy::Public(PublicReason::CorporateRegistration),
                "{path}"
            );
        }
    }

    // trace: US-1901 AC4, AC7, KEPUTUSAN-324: pesan seragam tidak menyebut NIB, email, atau akun
    #[test]
    fn uniform_message_does_not_reveal_existence() {
        let m = UNIFORM_MESSAGE.to_lowercase();
        for w in [
            "sudah terdaftar",
            "sudah ada",
            "nib ",
            "ditolak",
            "akun anda sudah",
            "tidak tiba",
            "diduga",
            "didaftarkan",
            "beberapa menit",
        ] {
            assert!(!m.contains(w), "{w}");
        }
        assert!(m.contains("layanan pelanggan"));
    }
}
