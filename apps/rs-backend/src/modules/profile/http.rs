//! Handler identitas pasien (`/patient/v1`, KEPUTUSAN-120; reference implementation US-106, F1-T61).
//! Setiap rute berkebijakan akun pasien (`policy.rs`); token, status akun, dan sesi ditegakkan middleware
//! crate `authz` sebelum handler. Handler hanya mengurai DTO, memanggil satu fungsi `api`, dan memetakan
//! hasil ke envelope ADR-023. Relasi ke penerima layanan, batas NIK, dan audit ada di service.

use super::api::{
    self as identity, ProfileError, cancel_identity_submission, create_ktp_upload_slot,
    submit_identity_document,
};
use super::dto::{
    IdentityDto, IdentityEnvelope, IdentitySubmissionRequest, KtpUploadSlotRequest, SubmissionDto,
    SubmissionEnvelope, UploadSlotDto, UploadSlotEnvelope,
};
use super::policy;
use crate::app::AppState;
use crate::http_support::{ApiFailure, ApiSuccess};
use crate::openapi::ErrorEnvelope;
use authz::{PolicyRouter, Principal};
use axum::Extension;
use axum::extract::rejection::JsonRejection;
use axum::extract::{Json, Path, State};
use axum::response::{IntoResponse, Response};
use envelope::{FieldError, Success};
use foundation::{CorrelationId, Uuid};
use utoipa_axum::routes;

/// Pola rute untuk security log dan audit (bukan URL berisi ID).
const STATUS_ROUTE: &str = "/patient/v1/care-recipients/{care_recipient_id}/identity";
const SUBMIT_ROUTE: &str = "/patient/v1/care-recipients/{care_recipient_id}/identity-submissions";
const CANCEL_ROUTE: &str =
    "/patient/v1/care-recipients/{care_recipient_id}/identity-submissions/cancel";

pub(super) fn router() -> PolicyRouter<AppState> {
    PolicyRouter::new()
        .routes(policy::patient(), routes!(ktp_upload_slot))
        .routes(policy::patient(), routes!(get_identity))
        .routes(policy::patient(), routes!(submit))
        .routes(policy::patient(), routes!(cancel))
}

fn fail(state: &AppState, cid: CorrelationId, e: ProfileError) -> Response {
    match &e {
        ProfileError::Internal(what) => tracing::error!(error = *what, "identity_request_failed"),
        ProfileError::ProjectionMismatch => tracing::error!("identity_projection_mismatch"),
        _ => {}
    }
    ApiFailure::new(e, cid, state.problem_type_base()).into_response()
}

/// ID penerima layanan di path yang bukan UUID = tidak ada (404, sama dengan milik orang lain, KU-03).
fn care_recipient_id(raw: &str) -> Result<Uuid, ProfileError> {
    Uuid::parse_str(raw).map_err(|_| ProfileError::NotFound)
}

/// Galat pembacaan badan JSON. Teks galat serde tidak diteruskan (dapat memuat NIK).
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

fn identity_response(
    state: &AppState,
    cid: CorrelationId,
    r: Result<identity::IdentityStatusView, ProfileError>,
) -> Response {
    let dto = r.and_then(|v| {
        IdentityDto::from_view(v).map_err(|_| ProfileError::Internal("identity_dto"))
    });
    match dto {
        Ok(d) => ApiSuccess(Success::ok(d).with_correlation_id(cid)).into_response(),
        Err(e) => fail(state, cid, e),
    }
}

/// Slot unggah foto KTP untuk akun pemanggil (US-106 AC3). Berkas diunggah langsung ke object storage,
/// lalu dikirim bersama NIK lewat `identity-submissions`.
#[utoipa::path(
    post,
    path = "/patient/v1/ktp-photos/upload-slots",
    tag = "identity",
    operation_id = "identity_ktp_upload_slot_create",
    security(("bearer" = [])),
    request_body(content = KtpUploadSlotRequest, content_type = "application/json"),
    responses(
        (status = 201, description = "Slot unggah diterbitkan.", body = UploadSlotEnvelope, content_type = "application/json"),
        (status = 400, description = "Badan permintaan tidak dapat dibaca.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 401, description = "Belum login, token tidak sah, sesi dicabut, atau akun nonaktif.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 413, description = "Ukuran melebihi 5.242.880 byte.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 415, description = "Jenis bukan image/jpeg atau image/png, atau badan bukan JSON.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 422, description = "Isian tidak valid.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 503, description = "Layanan tidak tersedia.", body = ErrorEnvelope, content_type = "application/problem+json")
    )
)]
async fn ktp_upload_slot(
    State(state): State<AppState>,
    Extension(cid): Extension<CorrelationId>,
    principal: Principal,
    body: Result<Json<KtpUploadSlotRequest>, JsonRejection>,
) -> Response {
    let Json(req) = match body {
        Ok(b) => b,
        Err(e) => return fail(&state, cid, body_error(&e)),
    };
    let r = create_ktp_upload_slot(
        state.identity(),
        &principal,
        req.content_type,
        req.size_bytes,
    )
    .await
    .and_then(|t| {
        UploadSlotDto::from_ticket(t).map_err(|_| ProfileError::Internal("timestamp_format"))
    });
    match r {
        Ok(d) => ApiSuccess(Success::created(d).with_correlation_id(cid)).into_response(),
        Err(e) => fail(&state, cid, e),
    }
}

/// Status identitas penerima layanan milik pemanggil (AB-05, US-106 AC1, AC5).
#[utoipa::path(
    get,
    path = "/patient/v1/care-recipients/{care_recipient_id}/identity",
    tag = "identity",
    operation_id = "identity_status_get",
    security(("bearer" = [])),
    params(("care_recipient_id" = String, Path, description = "ID penerima layanan (UUID).")),
    responses(
        (status = 200, description = "Status identitas.", body = IdentityEnvelope, content_type = "application/json"),
        (status = 401, description = "Belum login, token tidak sah, sesi dicabut, atau akun nonaktif.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 404, description = "Penerima layanan tidak ada atau bukan milik akun ini (KU-03).", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 503, description = "Layanan tidak tersedia.", body = ErrorEnvelope, content_type = "application/problem+json")
    )
)]
async fn get_identity(
    State(state): State<AppState>,
    Extension(cid): Extension<CorrelationId>,
    principal: Principal,
    Path(raw): Path<String>,
) -> Response {
    let id = match care_recipient_id(&raw) {
        Ok(id) => id,
        Err(e) => return fail(&state, cid, e),
    };
    let r = identity::identity_status(state.identity(), &principal, STATUS_ROUTE, id).await;
    identity_response(&state, cid, r)
}

/// Pengajuan NIK dan foto KTP (US-106 AC1, AC2, AC4, AC5). Status menjadi "Menunggu verifikasi";
/// verifikasi dilakukan manual oleh admin.
#[utoipa::path(
    post,
    path = "/patient/v1/care-recipients/{care_recipient_id}/identity-submissions",
    tag = "identity",
    operation_id = "identity_submission_create",
    security(("bearer" = [])),
    params(("care_recipient_id" = String, Path, description = "ID penerima layanan (UUID).")),
    request_body(content = IdentitySubmissionRequest, content_type = "application/json"),
    responses(
        (status = 201, description = "Pengajuan diterima; status Menunggu verifikasi.", body = SubmissionEnvelope, content_type = "application/json"),
        (status = 400, description = "Badan permintaan tidak dapat dibaca.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 401, description = "Belum login, token tidak sah, sesi dicabut, atau akun nonaktif.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 404, description = "Penerima layanan atau slot unggah tidak ada atau bukan milik akun ini (KU-03).", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 409, description = "4090201: pengajuan sebelumnya masih menunggu verifikasi (US-106 AC5). 4090202: NIK tidak dapat dipakai untuk akun ini, pesan umum (US-106 AC4).", body = ErrorEnvelope, content_type = "application/problem+json",
            examples(
                ("submission_locked" = (summary = "Masih menunggu verifikasi", value = json!({
                    "type": "https://problems.ruang-sehat.invalid/4090201",
                    "response_code": "4090201",
                    "response_description": "Foto KTP tidak dapat diganti sampai pengajuan diputuskan atau dibatalkan",
                    "instance": "/patient/v1/care-recipients/{care_recipient_id}/identity-submissions",
                    "correlation_id": "01936f8e-9c01-7e02-8f03-1a2b3c4d5e6f"
                }))),
                ("nik_unavailable" = (summary = "NIK tidak dapat dipakai", value = json!({
                    "type": "https://problems.ruang-sehat.invalid/4090202",
                    "response_code": "4090202",
                    "response_description": "NIK ini tidak dapat dipakai untuk akun Anda. Hubungi Layanan Pelanggan bila ini NIK Anda.",
                    "instance": "/patient/v1/care-recipients/{care_recipient_id}/identity-submissions",
                    "correlation_id": "01936f8e-9c01-7e02-8f03-1a2b3c4d5e6f"
                })))
            )
        ),
        (status = 413, description = "Berkas melebihi 5.242.880 byte.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 415, description = "Jenis berkas menurut isinya bukan JPG/PNG, atau badan bukan JSON.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 422, description = "NIK bukan 16 digit angka (4220001), slot tidak valid, atau berkas belum diunggah.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 503, description = "Layanan tidak tersedia.", body = ErrorEnvelope, content_type = "application/problem+json")
    )
)]
async fn submit(
    State(state): State<AppState>,
    Extension(cid): Extension<CorrelationId>,
    principal: Principal,
    Path(raw): Path<String>,
    body: Result<Json<IdentitySubmissionRequest>, JsonRejection>,
) -> Response {
    let id = match care_recipient_id(&raw) {
        Ok(id) => id,
        Err(e) => return fail(&state, cid, e),
    };
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
                ProfileError::Validation(vec![FieldError::new(
                    "upload_slot_id",
                    "invalid_format",
                    "ID slot unggah tidak valid",
                )]),
            );
        }
    };
    let r = submit_identity_document(
        state.identity(),
        &principal,
        SUBMIT_ROUTE,
        id,
        &req.nik,
        slot,
    )
    .await
    .and_then(|a| {
        SubmissionDto::from_accepted(a).map_err(|_| ProfileError::Internal("submission_dto"))
    });
    match r {
        Ok(d) => ApiSuccess(Success::created(d).with_correlation_id(cid)).into_response(),
        Err(e) => fail(&state, cid, e),
    }
}

/// Membatalkan pengiriman yang masih menunggu verifikasi (US-106 AC5): status kembali seperti sebelum
/// pengajuan itu. Bila Verifikator sudah memutuskan lebih dulu, dijawab `409` (US-201 AC5).
#[utoipa::path(
    post,
    path = "/patient/v1/care-recipients/{care_recipient_id}/identity-submissions/cancel",
    tag = "identity",
    operation_id = "identity_submission_cancel",
    security(("bearer" = [])),
    params(("care_recipient_id" = String, Path, description = "ID penerima layanan (UUID).")),
    responses(
        (status = 200, description = "Dibatalkan; status identitas sesudahnya.", body = IdentityEnvelope, content_type = "application/json"),
        (status = 401, description = "Belum login, token tidak sah, sesi dicabut, atau akun nonaktif.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 404, description = "Penerima layanan tidak ada atau bukan milik akun ini (KU-03).", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 409, description = "Tidak ada pengajuan dokumen yang menunggu, atau keputusan Verifikator tersimpan lebih dulu (4090001).", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 503, description = "Layanan tidak tersedia.", body = ErrorEnvelope, content_type = "application/problem+json")
    )
)]
async fn cancel(
    State(state): State<AppState>,
    Extension(cid): Extension<CorrelationId>,
    principal: Principal,
    Path(raw): Path<String>,
) -> Response {
    let id = match care_recipient_id(&raw) {
        Ok(id) => id,
        Err(e) => return fail(&state, cid, e),
    };
    let r = cancel_identity_submission(state.identity(), &principal, CANCEL_ROUTE, id).await;
    identity_response(&state, cid, r)
}
