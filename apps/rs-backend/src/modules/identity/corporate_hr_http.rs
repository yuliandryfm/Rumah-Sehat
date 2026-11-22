//! Handler kelola anggota HR perusahaan (`/v1/corporate/members`, F1-T107; US-1904 AC2, AC5). Setiap rute
//! berkebijakan `Policy::corporate_hr` (HR perusahaan Aktif): token realm `korporat`, status akun, sesi, dan
//! keanggotaan aktif ditegakkan middleware `authz` sebelum handler; cakupan perusahaan
//! (`authz::CorporateScope`) selalu dari keanggotaan, tidak pernah dari permintaan. Setiap rute terdaftar di
//! harness isolasi `tests/corporate_isolation.rs` (H-18).

use super::corporate_hr_service::{HrDeactivated, HrMemberView};
use super::error::IdentityError;
use crate::app::AppState;
use crate::http_support::{ApiFailure, ApiSuccess};
use crate::openapi::ErrorEnvelope;
use authz::{AccountStatus, CorporateScope, Policy, PolicyRouter};
use axum::Extension;
use axum::extract::{Path, State};
use axum::http::HeaderValue;
use axum::http::header::CACHE_CONTROL;
use axum::response::{IntoResponse, Response};
use envelope::Success;
use foundation::{CorrelationId, Uuid};
use serde::Serialize;
use utoipa::ToSchema;
use utoipa_axum::routes;

/// Pola rute untuk security log (bukan URL berisi ID).
const DEACTIVATE_ROUTE: &str = "/v1/corporate/members/{corporate_member_id}/deactivate";

/// HR perusahaan Aktif (US-1904 AC2; perusahaan Menunggu verifikasi/Ditolak hanya melihat status, US-1901
/// AC6).
fn manage_hr() -> Policy {
    Policy::corporate_hr()
}

pub(super) fn router() -> PolicyRouter<AppState> {
    PolicyRouter::new()
        .routes(manage_hr(), routes!(list_members))
        .routes(manage_hr(), routes!(deactivate_member))
}

/// Status akun HR.
#[derive(Debug, Clone, Copy, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub(super) enum HrStatusDto {
    /// Undangan belum diaktivasi.
    PendingActivation,
    /// Aktif.
    Active,
    /// Nonaktif.
    Inactive,
}

impl From<AccountStatus> for HrStatusDto {
    fn from(s: AccountStatus) -> Self {
        match s {
            AccountStatus::PendingActivation => Self::PendingActivation,
            AccountStatus::Active => Self::Active,
            AccountStatus::Inactive => Self::Inactive,
        }
    }
}

/// Satu anggota HR perusahaan pemanggil.
#[derive(Debug, Serialize, ToSchema)]
pub(super) struct HrMemberDto {
    #[schema(example = "0193a1b2-c3d4-7e5f-8a6b-7c8d9e0f1a2b")]
    corporate_member_id: String,
    #[schema(example = "hr@perusahaan.example")]
    email: String,
    #[schema(example = "HR Sintetis")]
    full_name: Option<String>,
    #[schema(example = "HR Manager")]
    job_title: Option<String>,
    status: HrStatusDto,
    /// Anggota ini adalah pemanggil (tidak dapat dinonaktifkan sendiri).
    is_self: bool,
}

impl From<HrMemberView> for HrMemberDto {
    fn from(v: HrMemberView) -> Self {
        Self {
            corporate_member_id: v.corporate_member_id.hyphenated().to_string(),
            email: v.email,
            full_name: v.full_name,
            job_title: v.job_title,
            status: v.status.into(),
            is_self: v.is_self,
        }
    }
}

/// Hasil penonaktifan.
#[derive(Debug, Serialize, ToSchema)]
pub(super) struct HrDeactivatedDto {
    #[schema(example = "0193a1b2-c3d4-7e5f-8a6b-7c8d9e0f1a2b")]
    corporate_member_id: String,
    status: HrStatusDto,
    /// Pengguna Keycloak sudah dinonaktifkan. `false` = akses backend sudah tertutup, sesi login Keycloak
    /// diakhiri kemudian.
    identity_provider_synced: bool,
}

impl From<HrDeactivated> for HrDeactivatedDto {
    fn from(v: HrDeactivated) -> Self {
        Self {
            corporate_member_id: v.corporate_member_id.hyphenated().to_string(),
            status: HrStatusDto::Inactive,
            identity_provider_synced: v.identity_provider_synced,
        }
    }
}

/// Bentuk dokumentasi respons daftar anggota HR.
#[derive(ToSchema)]
#[allow(dead_code)] // hanya untuk pembangkitan skema
pub(super) struct HrMembersEnvelope {
    #[schema(example = "2000000")]
    response_code: String,
    #[schema(example = "Berhasil")]
    response_description: String,
    data: Vec<HrMemberDto>,
    #[schema(nullable = false)]
    correlation_id: Option<String>,
}

/// Bentuk dokumentasi respons penonaktifan.
#[derive(ToSchema)]
#[allow(dead_code)] // hanya untuk pembangkitan skema
pub(super) struct HrDeactivatedEnvelope {
    #[schema(example = "2000000")]
    response_code: String,
    #[schema(example = "Berhasil")]
    response_description: String,
    data: HrDeactivatedDto,
    #[schema(nullable = false)]
    correlation_id: Option<String>,
}

fn fail(state: &AppState, cid: CorrelationId, e: IdentityError) -> Response {
    if matches!(e, IdentityError::Database | IdentityError::Audit) {
        tracing::error!(error = e.code(), "corporate_hr_request_failed");
    }
    ApiFailure::new(e, cid, state.problem_type_base()).into_response()
}

/// Respons berisi email rekan kerja tidak disimpan cache mana pun.
fn ok<T: Serialize>(cid: CorrelationId, d: T) -> Response {
    let mut r = ApiSuccess(Success::ok(d).with_correlation_id(cid)).into_response();
    r.headers_mut()
        .insert(CACHE_CONTROL, HeaderValue::from_static("no-store"));
    r
}

/// Anggota HR perusahaan pemanggil, termasuk undangan yang belum diaktivasi (US-1904 AC2).
#[utoipa::path(
    get,
    path = "/v1/corporate/members",
    tag = "corporate-hr",
    operation_id = "corporate_member_list",
    security(("bearer" = [])),
    responses(
        (status = 200, description = "Anggota HR perusahaan pemanggil.", body = HrMembersEnvelope, content_type = "application/json"),
        (status = 401, description = "Belum login, token tidak sah, sesi dicabut, akun atau perusahaan nonaktif, atau keanggotaan dicabut.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 403, description = "Perusahaan belum Aktif (menunggu verifikasi atau ditolak).", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 503, description = "Layanan tidak tersedia.", body = ErrorEnvelope, content_type = "application/problem+json")
    )
)]
async fn list_members(
    State(state): State<AppState>,
    Extension(cid): Extension<CorrelationId>,
    scope: CorporateScope,
) -> Response {
    match state.corporate_hr().list_members(&scope).await {
        Ok(v) => ok(
            cid,
            v.into_iter().map(HrMemberDto::from).collect::<Vec<_>>(),
        ),
        Err(e) => fail(&state, cid, e),
    }
}

/// Menonaktifkan HR lain di perusahaan pemanggil (US-1904 AC2, AC5): keanggotaan dicabut dan akun
/// nonaktif; permintaan berikutnya HR itu langsung ditolak.
#[utoipa::path(
    post,
    path = "/v1/corporate/members/{corporate_member_id}/deactivate",
    tag = "corporate-hr",
    operation_id = "corporate_member_deactivate",
    security(("bearer" = [])),
    params(("corporate_member_id" = String, Path, description = "ID keanggotaan HR (UUID).")),
    responses(
        (status = 200, description = "Dinonaktifkan.", body = HrDeactivatedEnvelope, content_type = "application/json"),
        (status = 401, description = "Belum login, token tidak sah, sesi dicabut, akun atau perusahaan nonaktif, atau keanggotaan dicabut (termasuk pemanggil yang baru saja dinonaktifkan HR lain).", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 403, description = "Perusahaan belum Aktif (menunggu verifikasi atau ditolak).", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 404, description = "Keanggotaan tidak ada, sudah dicabut, atau milik perusahaan lain.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 409, description = "4090101: HR menonaktifkan dirinya sendiri. 4090104: HR aktif terakhir perusahaan. 4090001: akun sudah nonaktif.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 500, description = "Galat internal, termasuk pencatatan audit gagal (aksi dibatalkan).", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 503, description = "Layanan tidak tersedia.", body = ErrorEnvelope, content_type = "application/problem+json")
    )
)]
async fn deactivate_member(
    State(state): State<AppState>,
    Extension(cid): Extension<CorrelationId>,
    scope: CorporateScope,
    Path(raw): Path<String>,
) -> Response {
    // ID bukan UUID = tidak ada (404).
    let Ok(id) = Uuid::parse_str(&raw) else {
        return fail(&state, cid, IdentityError::NotFound);
    };
    match state
        .corporate_hr()
        .deactivate_member(&scope, DEACTIVATE_ROUTE, id)
        .await
    {
        Ok(v) => ok(cid, HrDeactivatedDto::from(v)),
        Err(e) => fail(&state, cid, e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn route_constant_matches_registered_path_and_every_route_is_corporate_hr() {
        let router = router();
        let entries: Vec<_> = router.registry().iter().collect();
        assert_eq!(entries.len(), 2);
        assert!(entries.iter().any(|((_, p), _)| p == DEACTIVATE_ROUTE));
        for ((_, path), policy) in entries {
            assert!(path.starts_with("/v1/corporate/"), "{path}");
            assert_eq!(policy, &Policy::corporate_hr(), "{path}");
        }
    }
}
