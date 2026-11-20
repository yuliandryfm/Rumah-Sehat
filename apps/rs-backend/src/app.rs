//! Perakitan aplikasi HTTP: router berkebijakan (KEPUTUSAN-127, crate `authz`), middleware korelasi
//! (KEPUTUSAN-123), dan penolakan berformat envelope.

use crate::http_support::EnvelopeRenderer;
use crate::modules::platform;
use crate::modules::platform::api::ReadinessCheck;
use crate::openapi::OpsApiDoc;
use authz::{Enforcement, PolicyRouter, PolicySetupError};
use axum::Router;
use observability::correlation_middleware;
use std::sync::Arc;
use utoipa::OpenApi;

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

/// Router endpoint operasional beserta spesifikasinya (`openapi/ops.json`); setiap rute berkebijakan.
pub(crate) fn ops_router() -> PolicyRouter<AppState> {
    PolicyRouter::with_openapi(OpsApiDoc::openapi()).merge(platform::router())
}

/// Router lengkap siap `axum::serve`.
///
/// - `PolicyRouter::finish` memeriksa saat start bahwa setiap rute punya kebijakan (galat = start
///   gagal) dan memasang middleware penegak: rute tidak dikenal, metode tidak diizinkan, atau rute tanpa
///   kebijakan dijawab `404` envelope (`4040001`). Registry `docs/05-response-codes.md` tidak memuat
///   kode 405, dan 404 tidak membedakan "tidak ada" dari "tidak boleh" (KU-03).
/// - Pada tahap ini semua endpoint publik (kesehatan), sehingga validator token belum dipasang;
///   endpoint terautentikasi pertama wajib memasang `Enforcement::new` (tanpa itu start gagal).
/// - Middleware korelasi dipasang terakhir dengan `Router::layer`, sehingga membungkus penegak dan
///   `MatchedPath` tersedia.
pub fn build_router(state: AppState) -> Result<Router, PolicySetupError> {
    let renderer = Arc::new(EnvelopeRenderer::new(state.problem_type_base()));
    let (router, _spec) = ops_router().finish(Enforcement::public_only(renderer))?;
    Ok(router
        .with_state(state)
        .layer(axum::middleware::from_fn(correlation_middleware)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use authz::{Policy, PublicReason};

    #[test]
    fn every_ops_route_has_a_policy_and_router_starts() {
        // KEPUTUSAN-127: pemeriksaan cakupan dijalankan finish(); galat = start gagal.
        let router = ops_router();
        let entries: Vec<_> = router.registry().iter().collect();
        assert_eq!(entries.len(), 2);
        for ((method, _path), policy) in entries {
            assert_eq!(method, "GET");
            assert_eq!(policy, &Policy::Public(PublicReason::Operational));
        }
        assert!(build_router(AppState::new("https://d.example/problems", Vec::new())).is_ok());
    }
}
