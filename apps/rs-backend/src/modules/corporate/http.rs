//! Handler verifikasi perusahaan (`/backoffice/v1`, KEPUTUSAN-120; F1-T104, US-1902), pola
//! `profile::verifier_http` (reference implementation US-201). Setiap rute berkebijakan sub-peran Verifikator
//! (`policy.rs`); token, status akun, sesi, dan sub-peran ditegakkan middleware crate `authz` sebelum handler
//! (sub-peran lain → `403`, realm lain termasuk HR realm `korporat` → `401`). Handler hanya mengurai DTO,
//! memanggil satu fungsi `api`, dan memetakan hasil ke envelope ADR-023. Teks galat pustaka (serde, sqlx)
//! tidak pernah diteruskan.

use super::api::{self as corporate, CorporateError, CorporateVerification};
use super::dto::{
    ApproveRequest, DecisionDto, DecisionEnvelope, DetailDto, DetailEnvelope, DocumentKindDto,
    DocumentUrlDto, DocumentUrlEnvelope, DtoError, QueueEnvelope, QueueItemDto, QueueParams,
    RejectRequest, RejectionReasonDto, RejectionReasonsEnvelope,
};
use super::policy;
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
const DOCUMENT_ROUTE: &str = "/backoffice/v1/corporate-verifications/{corporate_client_id}/documents/{document_kind}/download-url";
const APPROVE_ROUTE: &str = "/backoffice/v1/corporate-verifications/{corporate_client_id}/approve";
const REJECT_ROUTE: &str = "/backoffice/v1/corporate-verifications/{corporate_client_id}/reject";

pub(super) fn router() -> PolicyRouter<AppState> {
    PolicyRouter::new()
        .routes(policy::verifier(), routes!(queue))
        .routes(policy::verifier(), routes!(detail))
        .routes(policy::verifier(), routes!(document_url))
        .routes(policy::verifier(), routes!(approve))
        .routes(policy::verifier(), routes!(reject))
        .routes(policy::verifier(), routes!(rejection_reasons))
}

fn deps(state: &AppState) -> CorporateVerification<'_> {
    CorporateVerification {
        pool: state.pool(),
        clock: state.clock(),
        files: state.files(),
    }
}

fn fail(state: &AppState, cid: CorrelationId, e: CorporateError) -> Response {
    if let CorporateError::Internal(what) = &e {
        tracing::error!(error = *what, "corporate_verification_failed");
    }
    ApiFailure::new(e, cid, state.problem_type_base()).into_response()
}

/// Respons berisi data perusahaan (NPWP) atau URL dokumen tidak boleh disimpan cache mana pun.
fn no_store(mut r: Response) -> Response {
    r.headers_mut()
        .insert(CACHE_CONTROL, HeaderValue::from_static("no-store"));
    r
}

fn ok<T: serde::Serialize>(cid: CorrelationId, d: T) -> Response {
    no_store(ApiSuccess(Success::ok(d).with_correlation_id(cid)).into_response())
}

/// ID di path yang bukan UUID = tidak ada (404).
fn client_id(raw: &str) -> Result<Uuid, CorporateError> {
    Uuid::parse_str(raw).map_err(|_| CorporateError::NotFound)
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

fn dto_error(_: DtoError) -> CorporateError {
    CorporateError::Internal("corporate_verification_dto")
}

/// Antrean pengajuan perusahaan "Menunggu verifikasi", terlama lebih dulu (US-1902 AC1).
#[utoipa::path(
    get,
    path = "/backoffice/v1/corporate-verifications",
    tag = "corporate-verification",
    operation_id = "corporate_verification_queue",
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
                CorporateError::Validation(vec![FieldError::new(
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
                CorporateError::Validation(vec![FieldError::new(
                    "cursor",
                    "invalid_format",
                    "Cursor tidak valid",
                )]),
            );
        }
    };
    let readiness = p.readiness.map(Into::into).unwrap_or_default();
    match corporate::corporate_verification_queue(&deps(&state), cursor, p.limit, readiness).await {
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

/// Detail perusahaan termasuk NPWP dan jenis dokumen aktif (US-1902 AC1).
#[utoipa::path(
    get,
    path = "/backoffice/v1/corporate-verifications/{corporate_client_id}",
    tag = "corporate-verification",
    operation_id = "corporate_verification_get",
    security(("bearer" = [])),
    params(("corporate_client_id" = String, Path, description = "ID perusahaan (UUID).")),
    responses(
        (status = 200, description = "Detail perusahaan.", body = DetailEnvelope, content_type = "application/json"),
        (status = 401, description = "Belum login, token tidak sah, sesi dicabut, atau akun nonaktif.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 403, description = "Akun tanpa sub-peran Verifikator.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 404, description = "Perusahaan tidak ada.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 503, description = "Layanan tidak tersedia.", body = ErrorEnvelope, content_type = "application/problem+json")
    )
)]
async fn detail(
    State(state): State<AppState>,
    Extension(cid): Extension<CorrelationId>,
    _principal: Principal,
    Path(raw): Path<String>,
) -> Response {
    let id = match client_id(&raw) {
        Ok(id) => id,
        Err(e) => return fail(&state, cid, e),
    };
    let r = corporate::corporate_verification_detail(&deps(&state), id)
        .await
        .and_then(|v| DetailDto::from_view(v).map_err(dto_error));
    match r {
        Ok(d) => ok(cid, d),
        Err(e) => fail(&state, cid, e),
    }
}

/// URL unduh dokumen NIB, NPWP, atau bukti kewenangan pendaftar berumur pendek (US-1902 AC1; ADR-018
/// butir 3). E-05 tercatat sebelum URL terbit; gagal mencatat = tidak ada URL.
#[utoipa::path(
    post,
    path = "/backoffice/v1/corporate-verifications/{corporate_client_id}/documents/{document_kind}/download-url",
    tag = "corporate-verification",
    operation_id = "corporate_verification_document_url",
    security(("bearer" = [])),
    params(
        ("corporate_client_id" = String, Path, description = "ID perusahaan (UUID)."),
        ("document_kind" = DocumentKindDto, Path, description = "Jenis dokumen.")
    ),
    responses(
        (status = 200, description = "URL unduh berumur pendek.", body = DocumentUrlEnvelope, content_type = "application/json"),
        (status = 401, description = "Belum login, token tidak sah, sesi dicabut, atau akun nonaktif.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 403, description = "Akun tanpa sub-peran Verifikator.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 404, description = "Perusahaan, jenis dokumen, atau berkas tidak ada.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 503, description = "Layanan tidak tersedia.", body = ErrorEnvelope, content_type = "application/problem+json")
    )
)]
async fn document_url(
    State(state): State<AppState>,
    Extension(cid): Extension<CorrelationId>,
    principal: Principal,
    Path((raw, kind)): Path<(String, String)>,
) -> Response {
    let id = match client_id(&raw) {
        Ok(id) => id,
        Err(e) => return fail(&state, cid, e),
    };
    let r = corporate::corporate_document_url(&deps(&state), &principal, DOCUMENT_ROUTE, id, &kind)
        .await
        .and_then(|t| DocumentUrlDto::from_ticket(t).map_err(dto_error));
    match r {
        Ok(d) => ok(cid, d),
        Err(e) => fail(&state, cid, e),
    }
}

/// Menyetujui pengajuan perusahaan: status Aktif (US-1902 AC2, AC4).
#[utoipa::path(
    post,
    path = "/backoffice/v1/corporate-verifications/{corporate_client_id}/approve",
    tag = "corporate-verification",
    operation_id = "corporate_verification_approve",
    security(("bearer" = [])),
    params(("corporate_client_id" = String, Path, description = "ID perusahaan (UUID).")),
    request_body(content = ApproveRequest, content_type = "application/json"),
    responses(
        (status = 200, description = "Disetujui.", body = DecisionEnvelope, content_type = "application/json"),
        (status = 400, description = "Badan permintaan tidak dapat dibaca.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 401, description = "Belum login, token tidak sah, sesi dicabut, atau akun nonaktif.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 403, description = "Akun tanpa sub-peran Verifikator.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 404, description = "Perusahaan tidak ada.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 409, description = "4090001: pengajuan sudah diputuskan atau tidak lagi menunggu verifikasi; keputusan pertama yang berlaku (US-1902 AC4). 4090113: NIB, NPWP, atau bukti kewenangan belum diunggah (F1-T102). 4090114: dokumen berubah sejak detail dilihat (F1-T102).", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 415, description = "Badan bukan application/json.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 422, description = "Field documents_version tidak ada atau ada field yang tidak dikenal (token yang salah nilainya dijawab 409 4090114).", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 503, description = "Layanan tidak tersedia.", body = ErrorEnvelope, content_type = "application/problem+json")
    )
)]
async fn approve(
    State(state): State<AppState>,
    Extension(cid): Extension<CorrelationId>,
    principal: Principal,
    Path(raw): Path<String>,
    body: Result<Json<ApproveRequest>, JsonRejection>,
) -> Response {
    let id = match client_id(&raw) {
        Ok(id) => id,
        Err(e) => return fail(&state, cid, e),
    };
    let Json(req) = match body {
        Ok(b) => b,
        Err(e) => return fail(&state, cid, body_error(&e)),
    };
    let r = corporate::approve_corporate_client(
        &deps(&state),
        &principal,
        APPROVE_ROUTE,
        id,
        &req.documents_version,
    )
    .await
    .and_then(|v| DecisionDto::from_view(v).map_err(dto_error));
    match r {
        Ok(d) => ok(cid, d),
        Err(e) => fail(&state, cid, e),
    }
}

/// Menolak pengajuan perusahaan dengan kode alasan dari daftar: status Ditolak (US-1902 AC3, AC4).
#[utoipa::path(
    post,
    path = "/backoffice/v1/corporate-verifications/{corporate_client_id}/reject",
    tag = "corporate-verification",
    operation_id = "corporate_verification_reject",
    security(("bearer" = [])),
    params(("corporate_client_id" = String, Path, description = "ID perusahaan (UUID).")),
    request_body(content = RejectRequest, content_type = "application/json"),
    responses(
        (status = 200, description = "Ditolak.", body = DecisionEnvelope, content_type = "application/json"),
        (status = 400, description = "Badan permintaan tidak dapat dibaca.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 401, description = "Belum login, token tidak sah, sesi dicabut, atau akun nonaktif.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 403, description = "Akun tanpa sub-peran Verifikator.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 404, description = "Perusahaan tidak ada.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 409, description = "4090001: pengajuan sudah diputuskan atau tidak lagi menunggu verifikasi; keputusan pertama yang berlaku (US-1902 AC4). 4090114: dokumen berubah sejak detail dilihat (F1-T102).", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 415, description = "Badan bukan application/json.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 422, description = "Alasan di luar daftar.", body = ErrorEnvelope, content_type = "application/problem+json"),
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
    let id = match client_id(&raw) {
        Ok(id) => id,
        Err(e) => return fail(&state, cid, e),
    };
    let Json(req) = match body {
        Ok(b) => b,
        Err(e) => return fail(&state, cid, body_error(&e)),
    };
    let r = corporate::reject_corporate_client(
        &deps(&state),
        &principal,
        REJECT_ROUTE,
        id,
        &req.reason_code,
        &req.documents_version,
    )
    .await
    .and_then(|v| DecisionDto::from_view(v).map_err(dto_error));
    match r {
        Ok(d) => ok(cid, d),
        Err(e) => fail(&state, cid, e),
    }
}

/// Daftar alasan penolakan perusahaan (hanya baca; dikelola lewat migrasi data, US-1902 AC3).
#[utoipa::path(
    get,
    path = "/backoffice/v1/corporate-verification-reasons",
    tag = "corporate-verification",
    operation_id = "corporate_verification_reason_list",
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
    match corporate::corporate_rejection_reasons(&deps(&state)).await {
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
        for r in [DOCUMENT_ROUTE, APPROVE_ROUTE, REJECT_ROUTE] {
            assert!(paths.iter().any(|p| p == r), "{r}");
        }
    }
}
