//! Handler perusahaan dibuatkan Verifikator (`POST /backoffice/v1/corporate-clients`, F1-T105; US-1903).
//! Kebijakan: sub-peran Verifikator (`policy.rs`); token, status akun, sesi, dan sub-peran ditegakkan
//! middleware crate `authz` sebelum handler (sub-peran lain `403`, realm lain termasuk HR realm `korporat`
//! `401`). Handler hanya mengurai DTO, memanggil satu fungsi `api`, dan memetakan hasil ke envelope ADR-023;
//! teks galat pustaka (serde, sqlx) tidak pernah diteruskan karena isian memuat NPWP (Terbatas) dan email.

use super::api::{
    self as corporate, CorporateError, CorporateVerification, CreatedCorporateClientView,
    NewCorporateClientInput,
};
use super::policy;
use crate::app::AppState;
use crate::http_support::{ApiFailure, ApiSuccess};
use crate::openapi::ErrorEnvelope;
use authz::{PolicyRouter, Principal};
use axum::Extension;
use axum::extract::Json;
use axum::extract::rejection::JsonRejection;
use axum::http::HeaderValue;
use axum::http::header::CACHE_CONTROL;
use axum::response::{IntoResponse, Response};
use envelope::{FieldError, Success};
use foundation::{CorrelationId, Sensitive};
use serde::{Deserialize, Serialize};
use std::fmt;
use utoipa::ToSchema;
use utoipa_axum::routes;

/// Pola rute untuk security log dan audit.
const CREATE_ROUTE: &str = "/backoffice/v1/corporate-clients";

pub(super) fn router() -> PolicyRouter<AppState> {
    PolicyRouter::new().routes(policy::verifier(), routes!(create_corporate_client))
}

/// Perusahaan baru dari Verifikator (US-1903 AC1).
#[derive(Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct CorporateClientCreateRequest {
    /// Nama badan usaha (maks 200 karakter).
    #[schema(example = "PT Contoh Sintetis", max_length = 200)]
    legal_name: String,
    /// NIB (maks 64 karakter, tanpa spasi; format resmi belum ditetapkan).
    #[schema(example = "1234567890123", max_length = 64)]
    nib: String,
    /// NPWP (Terbatas; maks 64 karakter, tanpa spasi).
    #[schema(example = "012345678901234", max_length = 64)]
    npwp: String,
    /// Alamat kantor (maks 500 karakter).
    #[schema(example = "Jl. Contoh No. 1, Jakarta", max_length = 500)]
    office_address: String,
    /// Telepon kantor (maks 32 karakter).
    #[schema(example = "021-0000000", max_length = 32)]
    office_phone: String,
    /// Email HR pertama: penerima undangan aktivasi (password dan TOTP) di realm `korporat`.
    #[schema(example = "hr@perusahaan.example", max_length = 254)]
    hr_email: String,
}

impl fmt::Debug for CorporateClientCreateRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CorporateClientCreateRequest")
            .finish_non_exhaustive()
    }
}

/// Hasil pembuatan perusahaan.
#[derive(Debug, Serialize, ToSchema)]
pub(super) struct CorporateClientCreatedDto {
    #[schema(example = "0193a1b2-c3d4-7e5f-8a6b-7c8d9e0f1a2b")]
    corporate_client_id: String,
    /// Selalu `active`: perusahaan dibuat Verifikator langsung Aktif (US-1903 AC1).
    #[schema(example = "active")]
    status: &'static str,
    #[schema(example = "0193a1b2-c3d4-7e5f-8a6b-7c8d9e0f1a2c")]
    corporate_member_id: String,
    #[schema(example = "0193a1b2-c3d4-7e5f-8a6b-7c8d9e0f1a2d")]
    hr_account_id: String,
    /// Selalu `processing`: pengguna HR dibuat dan email aktivasi dikirim worker beberapa saat setelah respons
    /// (ADR-030 butir 7-8); tautan berlaku 24 jam dan sekali pakai.
    #[schema(example = "processing")]
    hr_invitation: &'static str,
}

impl From<CreatedCorporateClientView> for CorporateClientCreatedDto {
    fn from(v: CreatedCorporateClientView) -> Self {
        Self {
            corporate_client_id: v.corporate_client_id.hyphenated().to_string(),
            status: "active",
            corporate_member_id: v.corporate_member_id.hyphenated().to_string(),
            hr_account_id: v.hr_account_id.hyphenated().to_string(),
            hr_invitation: "processing",
        }
    }
}

/// Bentuk dokumentasi respons pembuatan perusahaan.
#[derive(ToSchema)]
#[allow(dead_code)] // hanya untuk pembangkitan skema
pub(super) struct CorporateClientCreatedEnvelope {
    #[schema(example = "2010000")]
    response_code: String,
    #[schema(example = "Berhasil dibuat")]
    response_description: String,
    data: CorporateClientCreatedDto,
    #[schema(nullable = false)]
    correlation_id: Option<String>,
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
        tracing::error!(error = *what, "corporate_admin_create_failed");
    }
    ApiFailure::new(e, cid, state.problem_type_base()).into_response()
}

/// Membuat perusahaan Aktif dan mengundang HR pertama (US-1903 AC1-AC4). Perusahaan langsung Aktif tanpa
/// antrean verifikasi; HR menerima tautan aktivasi (password dan TOTP) berlaku 24 jam dan sekali pakai.
#[utoipa::path(
    post,
    path = "/backoffice/v1/corporate-clients",
    tag = "corporate-verification",
    operation_id = "corporate_client_create_by_admin",
    security(("bearer" = [])),
    request_body(content = CorporateClientCreateRequest, content_type = "application/json"),
    responses(
        (status = 201, description = "Perusahaan Aktif dibuat; undangan HR diproses.", body = CorporateClientCreatedEnvelope, content_type = "application/json"),
        (status = 400, description = "Badan permintaan tidak dapat dibaca.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 401, description = "Belum login, token tidak sah, sesi dicabut, atau akun nonaktif.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 403, description = "Akun tanpa sub-peran Verifikator.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 409, description = "4090110: NIB sudah terdaftar (US-1903 AC4). 4090111: email HR sudah dipakai akun lain di realm korporat.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 415, description = "Badan bukan application/json.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 422, description = "Isian tidak valid.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 500, description = "Galat internal, termasuk pencatatan audit gagal (aksi dibatalkan).", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 503, description = "Layanan tidak tersedia.", body = ErrorEnvelope, content_type = "application/problem+json")
    )
)]
async fn create_corporate_client(
    axum::extract::State(state): axum::extract::State<AppState>,
    Extension(cid): Extension<CorrelationId>,
    principal: Principal,
    body: Result<Json<CorporateClientCreateRequest>, JsonRejection>,
) -> Response {
    let Json(req) = match body {
        Ok(b) => b,
        Err(e) => return fail(&state, cid, body_error(&e)),
    };
    let input = NewCorporateClientInput {
        legal_name: req.legal_name,
        nib: req.nib,
        npwp: Sensitive::new(req.npwp),
        office_address: req.office_address,
        office_phone: req.office_phone,
        hr_email: Sensitive::new(req.hr_email),
    };
    let deps = CorporateVerification {
        pool: state.pool(),
        clock: state.clock(),
        files: state.files(),
    };
    match corporate::create_corporate_client_by_admin(&deps, &principal, CREATE_ROUTE, &input).await
    {
        Ok(v) => {
            let mut r = ApiSuccess(
                Success::created(CorporateClientCreatedDto::from(v)).with_correlation_id(cid),
            )
            .into_response();
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

    #[test]
    fn route_constant_matches_registered_path_and_requires_verifier() {
        let router = router();
        let entries: Vec<_> = router.registry().iter().collect();
        assert_eq!(entries.len(), 1);
        assert!(entries.iter().any(|((_, p), _)| p == CREATE_ROUTE));
        for ((method, _), p) in entries {
            assert_eq!(method, "POST");
            assert_eq!(p, &policy::verifier());
        }
    }
}
