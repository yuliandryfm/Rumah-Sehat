//! Perakitan aplikasi HTTP: router, middleware korelasi (KEPUTUSAN-123), dan fallback berformat
//! envelope.

use crate::modules::platform;
use crate::modules::platform::api::ReadinessCheck;
use crate::openapi::OpsApiDoc;
use axum::Extension;
use axum::Router;
use axum::extract::State;
use axum::response::IntoResponse;
use envelope::CommonError;
use foundation::CorrelationId;
use observability::correlation_middleware;
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;

use crate::http_support::ApiFailure;

/// Keadaan bersama handler. Murah di-clone.
#[derive(Clone)]
pub struct AppState {
    problem_type_base: Arc<str>,
    readiness: Arc<[Arc<dyn ReadinessCheck>]>,
}

impl AppState {
    /// `problem_type_base` adalah awalan URI `type` respons galat; `readiness` pemeriksaan kesiapan.
    pub fn new(problem_type_base: &str, readiness: Vec<Arc<dyn ReadinessCheck>>) -> Self {
        Self {
            problem_type_base: Arc::from(problem_type_base),
            readiness: Arc::from(readiness),
        }
    }

    pub(crate) fn problem_type_base(&self) -> &str {
        &self.problem_type_base
    }

    pub(crate) fn readiness(&self) -> &[Arc<dyn ReadinessCheck>] {
        &self.readiness
    }
}

/// Router endpoint operasional beserta spesifikasinya (`openapi/ops.json`).
pub(crate) fn ops_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::with_openapi(OpsApiDoc::openapi()).merge(platform::router())
}

/// Rute tidak dikenal atau metode tidak diizinkan: `404` envelope (`4040001`). Registry
/// `docs/05-response-codes.md` tidak memuat kode 405, dan 404 tidak membedakan "tidak ada" dari
/// "tidak boleh" (KU-03).
async fn not_found(
    State(state): State<AppState>,
    Extension(correlation_id): Extension<CorrelationId>,
) -> impl IntoResponse {
    ApiFailure::new(
        CommonError::NotFound,
        correlation_id,
        state.problem_type_base(),
    )
}

/// Router lengkap siap `axum::serve`. Middleware korelasi dipasang dengan `Router::layer` setelah
/// fallback, sehingga juga membungkus respons fallback dan `MatchedPath` tersedia.
pub fn build_router(state: AppState) -> Router {
    let (router, _spec) = ops_router().split_for_parts();
    router
        .fallback(not_found)
        .method_not_allowed_fallback(not_found)
        .with_state(state)
        .layer(axum::middleware::from_fn(correlation_middleware))
}
