//! Handler Verifikator (`/backoffice/v1`, KEPUTUSAN-120; reference implementation US-201, F1-T62). Setiap
//! rute berkebijakan sub-peran Verifikator (`policy.rs`); token, status akun, sesi, dan sub-peran
//! ditegakkan middleware crate `authz` sebelum handler (sub-peran lain → `403`, realm lain → `401`).
//! Handler hanya mengurai DTO, memanggil satu fungsi `api`, dan memetakan hasil ke envelope ADR-023.

use super::api::{self as identity, ProfileError};
use super::policy;
use super::verifier_dto::{
    DecisionDto, DecisionEnvelope, DocumentPhotoDto, DocumentPhotoEnvelope, QueueEnvelope,
    QueueItemDto, QueueParams, RejectRequest, RejectionReasonDto, RejectionReasonsEnvelope,
    VerificationDetailDto, VerificationDetailEnvelope,
};
use crate::app::AppState;
use crate::http_support::{ApiFailure, ApiSuccess};
use crate::openapi::ErrorEnvelope;
use authz::{PolicyRouter, Principal};
use axum::Extension;
use axum::extract::rejection::{JsonRejection, QueryRejection};
use axum::extract::{Json, Path, Query, State};
use axum::http::HeaderValue;
use axum::http::header::CACHE_CONTROL;
use axum::response::{IntoResponse, Response};
use envelope::{FieldError, Success};
use foundation::{CorrelationId, Uuid};
use utoipa_axum::routes;

/// Pola rute untuk security log dan audit (bukan URL berisi ID).
const DETAIL_ROUTE: &str =
    "/backoffice/v1/identity-verifications/{identity_verification_request_id}";
const PHOTO_ROUTE: &str =
    "/backoffice/v1/identity-verifications/{identity_verification_request_id}/document-photo-url";
const APPROVE_ROUTE: &str =
    "/backoffice/v1/identity-verifications/{identity_verification_request_id}/approve";
const REJECT_ROUTE: &str =
    "/backoffice/v1/identity-verifications/{identity_verification_request_id}/reject";

pub(super) fn router() -> PolicyRouter<AppState> {
    PolicyRouter::new()
        .routes(policy::verifier(), routes!(queue))
        .routes(policy::verifier(), routes!(detail))
        .routes(policy::verifier(), routes!(document_photo_url))
        .routes(policy::verifier(), routes!(approve))
        .routes(policy::verifier(), routes!(reject))
        .routes(policy::verifier(), routes!(rejection_reasons))
}

fn fail(state: &AppState, cid: CorrelationId, e: ProfileError) -> Response {
    match &e {
        ProfileError::Internal(what) => {
            tracing::error!(error = *what, "verification_request_failed")
        }
        ProfileError::ProjectionMismatch => tracing::error!("verification_projection_mismatch"),
        _ => {}
    }
    ApiFailure::new(e, cid, state.problem_type_base()).into_response()
}

/// Respons berisi data identitas (NIK lengkap, URL foto) tidak boleh disimpan cache mana pun.
fn no_store(mut r: Response) -> Response {
    r.headers_mut()
        .insert(CACHE_CONTROL, HeaderValue::from_static("no-store"));
    r
}

fn ok<T: serde::Serialize>(cid: CorrelationId, d: T) -> Response {
    no_store(ApiSuccess(Success::ok(d).with_correlation_id(cid)).into_response())
}

/// ID di path yang bukan UUID = tidak ada (404).
fn request_id(raw: &str) -> Result<Uuid, ProfileError> {
    Uuid::parse_str(raw).map_err(|_| ProfileError::NotFound)
}

fn body_error(e: &JsonRejection) -> ProfileError {
    match e {
        JsonRejection::MissingJsonContentType(_) => ProfileError::UnsupportedMediaType,
        JsonRejection::JsonDataError(_) => ProfileError::Validation(vec![FieldError::new(
            "$",
            "invalid_format",
            "Isian tidak lengkap, bertipe salah, atau tidak dikenal",
        )]),
        _ => ProfileError::BadRequest,
    }
}

fn dto_error(_: super::dto::DtoError) -> ProfileError {
    ProfileError::Internal("verification_dto")
}

/// Antrean pengajuan "Menunggu verifikasi", terlama lebih dulu (US-201 AC1).
#[utoipa::path(
    get,
    path = "/backoffice/v1/identity-verifications",
    tag = "identity-verification",
    operation_id = "identity_verification_queue",
    security(("bearer" = [])),
    params(QueueParams),
    responses(
        (status = 200, description = "Antrean.", body = QueueEnvelope, content_type = "application/json"),
        (status = 401, description = "Belum login, token tidak sah, sesi dicabut, atau akun nonaktif.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 403, description = "Akun tanpa sub-peran Verifikator.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 422, description = "Parameter tidak valid.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 503, description = "Layanan tidak tersedia.", body = ErrorEnvelope, content_type = "application/problem+json")
    )
)]
async fn queue(
    State(state): State<AppState>,
    Extension(cid): Extension<CorrelationId>,
    _principal: Principal,
    params: Result<Query<QueueParams>, QueryRejection>,
) -> Response {
    let Query(p) = match params {
        Ok(p) => p,
        Err(_) => {
            return fail(
                &state,
                cid,
                ProfileError::Validation(vec![FieldError::new(
                    "$query",
                    "invalid_format",
                    "Parameter tidak valid",
                )]),
            );
        }
    };
    let cursor = match p.cursor.as_deref().map(Uuid::parse_str).transpose() {
        Ok(c) => c,
        Err(_) => {
            return fail(
                &state,
                cid,
                ProfileError::Validation(vec![FieldError::new(
                    "cursor",
                    "invalid_format",
                    "Cursor tidak valid",
                )]),
            );
        }
    };
    match identity::verification_queue(state.identity(), cursor, p.limit).await {
        Ok((items, meta)) => {
            match items
                .into_iter()
                .map(QueueItemDto::from_view)
                .collect::<Result<Vec<_>, _>>()
            {
                Ok(d) => no_store(
                    ApiSuccess(Success::ok(d).with_meta(meta).with_correlation_id(cid))
                        .into_response(),
                ),
                Err(e) => fail(&state, cid, dto_error(e)),
            }
        }
        Err(e) => fail(&state, cid, e),
    }
}

/// Detail pengajuan dengan NIK lengkap (US-201 AC1). Membuka detail tercatat E-05 lebih dulu.
#[utoipa::path(
    get,
    path = "/backoffice/v1/identity-verifications/{identity_verification_request_id}",
    tag = "identity-verification",
    operation_id = "identity_verification_get",
    security(("bearer" = [])),
    params(("identity_verification_request_id" = String, Path, description = "ID permintaan verifikasi (UUID).")),
    responses(
        (status = 200, description = "Detail pengajuan.", body = VerificationDetailEnvelope, content_type = "application/json"),
        (status = 401, description = "Belum login, token tidak sah, sesi dicabut, atau akun nonaktif.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 403, description = "Akun tanpa sub-peran Verifikator.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 404, description = "Pengajuan tidak ada.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 503, description = "Layanan tidak tersedia.", body = ErrorEnvelope, content_type = "application/problem+json")
    )
)]
async fn detail(
    State(state): State<AppState>,
    Extension(cid): Extension<CorrelationId>,
    principal: Principal,
    Path(raw): Path<String>,
) -> Response {
    let id = match request_id(&raw) {
        Ok(id) => id,
        Err(e) => return fail(&state, cid, e),
    };
    let r = identity::verification_detail(state.identity(), &principal, DETAIL_ROUTE, id)
        .await
        .and_then(|v| VerificationDetailDto::from_view(v).map_err(dto_error));
    match r {
        Ok(d) => ok(cid, d),
        Err(e) => fail(&state, cid, e),
    }
}

/// URL unduh foto dokumen berumur pendek (US-201 AC1, AC6; ADR-018 butir 3). E-05 tercatat sebelum URL
/// terbit; gagal mencatat = tidak ada URL.
#[utoipa::path(
    post,
    path = "/backoffice/v1/identity-verifications/{identity_verification_request_id}/document-photo-url",
    tag = "identity-verification",
    operation_id = "identity_verification_document_photo_url",
    security(("bearer" = [])),
    params(("identity_verification_request_id" = String, Path, description = "ID permintaan verifikasi (UUID).")),
    responses(
        (status = 200, description = "URL unduh berumur pendek.", body = DocumentPhotoEnvelope, content_type = "application/json"),
        (status = 401, description = "Belum login, token tidak sah, sesi dicabut, atau akun nonaktif.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 403, description = "Akun tanpa sub-peran Verifikator.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 404, description = "Pengajuan atau berkas tidak ada.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 503, description = "Layanan tidak tersedia.", body = ErrorEnvelope, content_type = "application/problem+json")
    )
)]
async fn document_photo_url(
    State(state): State<AppState>,
    Extension(cid): Extension<CorrelationId>,
    principal: Principal,
    Path(raw): Path<String>,
) -> Response {
    let id = match request_id(&raw) {
        Ok(id) => id,
        Err(e) => return fail(&state, cid, e),
    };
    let r =
        identity::verification_document_photo_url(state.identity(), &principal, PHOTO_ROUTE, id)
            .await
            .and_then(|t| DocumentPhotoDto::from_ticket(t).map_err(dto_error));
    match r {
        Ok(d) => ok(cid, d),
        Err(e) => fail(&state, cid, e),
    }
}

/// Menyetujui pengajuan (US-201 AC2, AC5, AC7).
#[utoipa::path(
    post,
    path = "/backoffice/v1/identity-verifications/{identity_verification_request_id}/approve",
    tag = "identity-verification",
    operation_id = "identity_verification_approve",
    security(("bearer" = [])),
    params(("identity_verification_request_id" = String, Path, description = "ID permintaan verifikasi (UUID).")),
    responses(
        (status = 200, description = "Disetujui.", body = DecisionEnvelope, content_type = "application/json"),
        (status = 401, description = "Belum login, token tidak sah, sesi dicabut, atau akun nonaktif.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 403, description = "Akun tanpa sub-peran Verifikator.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 404, description = "Pengajuan tidak ada.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 409, description = "4090203: NIK sudah disetujui sebagai pemilik akun lain; tolak dengan alasan nik_unavailable (US-201 AC7). 4090204: pengajuan sudah diputuskan (US-201 AC5). 4090205: pemisahan tugas, pengajuan akun pasien milik Verifikator sendiri atau Verifikator tanpa NIK staf (US-201 AC8).", body = ErrorEnvelope, content_type = "application/problem+json",
            examples(
                ("owner_nik_taken" = (summary = "NIK dipegang pemilik akun lain", value = json!({
                    "type": "https://problems.ruang-sehat.invalid/4090203",
                    "response_code": "4090203",
                    "response_description": "NIK sudah disetujui sebagai pemilik akun lain. Tolak pengajuan dengan alasan nik_unavailable.",
                    "instance": "/backoffice/v1/identity-verifications/{identity_verification_request_id}/approve",
                    "correlation_id": "01936f8e-9c01-7e02-8f03-1a2b3c4d5e6f"
                }))),
                ("already_decided" = (summary = "Sudah diputuskan", value = json!({
                    "type": "https://problems.ruang-sehat.invalid/4090204",
                    "response_code": "4090204",
                    "response_description": "Pengajuan sudah diputuskan",
                    "instance": "/backoffice/v1/identity-verifications/{identity_verification_request_id}/approve",
                    "correlation_id": "01936f8e-9c01-7e02-8f03-1a2b3c4d5e6f"
                })))
            )
        ),
        (status = 503, description = "Layanan tidak tersedia.", body = ErrorEnvelope, content_type = "application/problem+json")
    )
)]
async fn approve(
    State(state): State<AppState>,
    Extension(cid): Extension<CorrelationId>,
    principal: Principal,
    Path(raw): Path<String>,
) -> Response {
    let id = match request_id(&raw) {
        Ok(id) => id,
        Err(e) => return fail(&state, cid, e),
    };
    let r = identity::approve_identity(state.identity(), &principal, APPROVE_ROUTE, id)
        .await
        .and_then(|v| DecisionDto::from_view(v).map_err(dto_error));
    match r {
        Ok(d) => ok(cid, d),
        Err(e) => fail(&state, cid, e),
    }
}

/// Menolak pengajuan dengan alasan dari daftar (US-201 AC3, AC3a, AC5; AC7 dengan `nik_unavailable`).
#[utoipa::path(
    post,
    path = "/backoffice/v1/identity-verifications/{identity_verification_request_id}/reject",
    tag = "identity-verification",
    operation_id = "identity_verification_reject",
    security(("bearer" = [])),
    params(("identity_verification_request_id" = String, Path, description = "ID permintaan verifikasi (UUID).")),
    request_body(content = RejectRequest, content_type = "application/json"),
    responses(
        (status = 200, description = "Ditolak.", body = DecisionEnvelope, content_type = "application/json"),
        (status = 400, description = "Badan permintaan tidak dapat dibaca.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 401, description = "Belum login, token tidak sah, sesi dicabut, atau akun nonaktif.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 403, description = "Akun tanpa sub-peran Verifikator.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 404, description = "Pengajuan tidak ada.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 409, description = "4090204: pengajuan sudah diputuskan (US-201 AC5). 4090205: pemisahan tugas (US-201 AC8).", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 415, description = "Badan bukan application/json.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 422, description = "Alasan di luar daftar, keterangan wajib untuk other, atau lebih dari 500 karakter.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 503, description = "Layanan tidak tersedia.", body = ErrorEnvelope, content_type = "application/problem+json")
    )
)]
async fn reject(
    State(state): State<AppState>,
    Extension(cid): Extension<CorrelationId>,
    principal: Principal,
    Path(raw): Path<String>,
    body: Result<Json<RejectRequest>, JsonRejection>,
) -> Response {
    let id = match request_id(&raw) {
        Ok(id) => id,
        Err(e) => return fail(&state, cid, e),
    };
    let Json(req) = match body {
        Ok(b) => b,
        Err(e) => return fail(&state, cid, body_error(&e)),
    };
    let r = identity::reject_identity(
        state.identity(),
        &principal,
        REJECT_ROUTE,
        id,
        &req.reason_code,
        req.note.as_deref(),
    )
    .await
    .and_then(|v| DecisionDto::from_view(v).map_err(dto_error));
    match r {
        Ok(d) => ok(cid, d),
        Err(e) => fail(&state, cid, e),
    }
}

/// Daftar alasan penolakan (hanya baca; dikelola lewat migrasi data, US-201 AC3).
#[utoipa::path(
    get,
    path = "/backoffice/v1/identity-verification-reasons",
    tag = "identity-verification",
    operation_id = "identity_verification_reason_list",
    security(("bearer" = [])),
    responses(
        (status = 200, description = "Daftar alasan penolakan.", body = RejectionReasonsEnvelope, content_type = "application/json"),
        (status = 401, description = "Belum login, token tidak sah, sesi dicabut, atau akun nonaktif.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 403, description = "Akun tanpa sub-peran Verifikator.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 503, description = "Layanan tidak tersedia.", body = ErrorEnvelope, content_type = "application/problem+json")
    )
)]
async fn rejection_reasons(
    State(state): State<AppState>,
    Extension(cid): Extension<CorrelationId>,
    _principal: Principal,
) -> Response {
    match identity::identity_rejection_reasons(state.identity()).await {
        Ok(v) => ok(
            cid,
            v.into_iter()
                .map(RejectionReasonDto::from_view)
                .collect::<Vec<_>>(),
        ),
        Err(e) => fail(&state, cid, e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn route_constants_match_registered_paths() {
        let router = router();
        let paths: Vec<String> = router
            .registry()
            .iter()
            .map(|((_m, p), _)| p.to_string())
            .collect();
        for r in [DETAIL_ROUTE, PHOTO_ROUTE, APPROVE_ROUTE, REJECT_ROUTE] {
            assert!(paths.iter().any(|p| p == r), "{r}");
        }
    }
}
