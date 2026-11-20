//! Handler endpoint kesehatan. Tanpa autentikasi (dibaca pemeriksa luar dan load balancer);
//! respons tidak memuat data pribadi, versi, atau status dependensi per nama.

use super::dto::{HealthData, HealthEnvelope, HealthStatus};
use super::policy;
use super::service;
use crate::app::AppState;
use crate::http_support::{ApiFailure, ApiSuccess};
use crate::openapi::ErrorEnvelope;
use authz::PolicyRouter;
use axum::Extension;
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use envelope::{CommonError, Success};
use foundation::CorrelationId;
use utoipa_axum::routes;

pub(super) fn router() -> PolicyRouter<AppState> {
    PolicyRouter::new()
        .routes(policy::health(), routes!(live))
        .routes(policy::health(), routes!(ready))
}

/// Liveness: proses hidup dan dapat melayani HTTP. Tidak memeriksa dependensi.
#[utoipa::path(
    get,
    path = "/health/live",
    tag = "ops",
    operation_id = "health_live",
    responses(
        (
            status = 200,
            description = "Proses hidup.",
            body = HealthEnvelope,
            content_type = "application/json",
            example = json!({
                "response_code": "2000000",
                "response_description": "Berhasil",
                "data": {"status": "ok"},
                "correlation_id": "01936f8e-9c01-7e02-8f03-1a2b3c4d5e6f"
            })
        )
    )
)]
async fn live(Extension(correlation_id): Extension<CorrelationId>) -> ApiSuccess<HealthData> {
    ApiSuccess(
        Success::ok(HealthData {
            status: HealthStatus::Ok,
        })
        .with_correlation_id(correlation_id),
    )
}

/// Readiness: semua dependensi terdaftar siap. Gagal dengan 503 (`5030001`) bila ada yang tidak siap.
#[utoipa::path(
    get,
    path = "/health/ready",
    tag = "ops",
    operation_id = "health_ready",
    responses(
        (
            status = 200,
            description = "Siap menerima lalu lintas.",
            body = HealthEnvelope,
            content_type = "application/json",
            example = json!({
                "response_code": "2000000",
                "response_description": "Berhasil",
                "data": {"status": "ok"},
                "correlation_id": "01936f8e-9c01-7e02-8f03-1a2b3c4d5e6f"
            })
        ),
        (
            status = 503,
            description = "Ada dependensi yang belum siap.",
            body = ErrorEnvelope,
            content_type = "application/problem+json",
            example = json!({
                "type": "https://problems.ruang-sehat.invalid/5030001",
                "response_code": "5030001",
                "response_description": "Layanan sedang tidak tersedia",
                "correlation_id": "01936f8e-9c01-7e02-8f03-1a2b3c4d5e6f"
            })
        )
    )
)]
async fn ready(
    State(state): State<AppState>,
    Extension(correlation_id): Extension<CorrelationId>,
) -> Response {
    if service::is_ready(state.readiness()).await {
        ApiSuccess(
            Success::ok(HealthData {
                status: HealthStatus::Ok,
            })
            .with_correlation_id(correlation_id),
        )
        .into_response()
    } else {
        ApiFailure::new(
            CommonError::Unavailable,
            correlation_id,
            state.problem_type_base(),
        )
        .into_response()
    }
}
