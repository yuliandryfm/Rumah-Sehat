//! Perakitan aplikasi HTTP: router berkebijakan (KEPUTUSAN-127, crate `authz`), validasi token (`authn`)
//! dan resolver akun, middleware korelasi (KEPUTUSAN-123), dan penolakan berformat envelope.

use crate::modules::content_support;
use crate::modules::platform;
use crate::modules::platform::api::{FileStorage, ReadinessCheck};
use crate::openapi::{BackofficeApiDoc, OpsApiDoc};
use authn::AuthState;
use authz::{Enforcement, PolicyRouter, PolicySetupError, PrincipalResolver};
use axum::Router;
use foundation::Clock;
use observability::correlation_middleware;
use sqlx::PgPool;
use std::sync::Arc;
use utoipa::OpenApi;

/// Dependensi bersama use case (pool database role `rs_app`, jam server, layanan berkas).
pub struct AppServices {
    /// Pool login `rs_api` (anggota `rs_app`).
    pub pool: PgPool,
    /// Jam server (KEPUTUSAN-131).
    pub clock: Arc<dyn Clock>,
    /// Layanan berkas (modul platform, F1-T29).
    pub files: Arc<FileStorage>,
}

/// Keadaan bersama handler. Murah di-clone.
#[derive(Clone)]
pub struct AppState {
    problem_type_base: Arc<str>,
    readiness: Arc<[Arc<dyn ReadinessCheck>]>,
    services: Arc<AppServices>,
}

impl AppState {
    /// `problem_type_base` adalah awalan URI `type` respons galat; `readiness` pemeriksaan kesiapan.
    pub fn new(
        problem_type_base: &str,
        readiness: Vec<Arc<dyn ReadinessCheck>>,
        services: AppServices,
    ) -> Self {
        Self {
            problem_type_base: Arc::from(problem_type_base),
            readiness: Arc::from(readiness),
            services: Arc::new(services),
        }
    }

    pub(crate) fn problem_type_base(&self) -> &str {
        &self.problem_type_base
    }

    pub(crate) fn readiness(&self) -> &[Arc<dyn ReadinessCheck>] {
        &self.readiness
    }

    pub(crate) fn pool(&self) -> &PgPool {
        &self.services.pool
    }

    pub(crate) fn clock(&self) -> &dyn Clock {
        self.services.clock.as_ref()
    }

    #[allow(dead_code)] // dipakai endpoint gambar konten (F1-T93)
    pub(crate) fn files(&self) -> &FileStorage {
        &self.services.files
    }
}

/// Router endpoint operasional beserta spesifikasinya (`openapi/ops.json`); setiap rute berkebijakan.
pub(crate) fn ops_router() -> PolicyRouter<AppState> {
    PolicyRouter::with_openapi(OpsApiDoc::openapi()).merge(platform::router())
}

/// Router API backoffice (`openapi/backoffice.json`, awalan `/backoffice/v1`, KEPUTUSAN-120).
pub(crate) fn backoffice_router() -> PolicyRouter<AppState> {
    PolicyRouter::with_openapi(BackofficeApiDoc::openapi())
        .merge(content_support::backoffice_router())
}

/// Router lengkap siap `axum::serve`.
///
/// - `PolicyRouter::finish` memeriksa saat start bahwa setiap rute punya kebijakan, kebijakan dapat
///   dipenuhi, dan realm yang dibutuhkan diterima validator token (galat = start gagal); lalu memasang
///   middleware penegak: rute tidak dikenal, metode tidak diizinkan, atau rute tanpa kebijakan dijawab
///   `404` envelope (`4040001`). Registry `docs/05-response-codes.md` tidak memuat kode 405, dan 404 tidak
///   membedakan "tidak ada" dari "tidak boleh" (KU-03).
/// - Endpoint terautentikasi (backoffice) memakai `auth` (validasi token realm `admin`) dan `resolver`
///   (akun dan sub-peran dari database aplikasi).
/// - Middleware korelasi dipasang terakhir dengan `Router::layer`, sehingga membungkus penegak dan
///   `MatchedPath` tersedia.
pub fn build_router(
    state: AppState,
    auth: AuthState,
    resolver: Arc<dyn PrincipalResolver>,
) -> Result<Router, PolicySetupError> {
    let (router, _spec) = ops_router()
        .merge(backoffice_router())
        .finish(Enforcement::new(auth, resolver))?;
    Ok(router
        .with_state(state)
        .layer(axum::middleware::from_fn(correlation_middleware)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use authz::{Policy, PublicReason};

    #[test]
    fn every_ops_route_has_a_policy() {
        // KEPUTUSAN-127: pemeriksaan cakupan dijalankan finish(); galat = start gagal.
        let router = ops_router();
        let entries: Vec<_> = router.registry().iter().collect();
        assert_eq!(entries.len(), 2);
        for ((method, _path), policy) in entries {
            assert_eq!(method, "GET");
            assert_eq!(policy, &Policy::Public(PublicReason::Operational));
        }
    }

    // trace: US-1802 AC4
    #[test]
    fn every_backoffice_route_requires_content_sub_role_only() {
        let router = backoffice_router();
        let entries: Vec<_> = router.registry().iter().collect();
        assert_eq!(entries.len(), 8);
        for ((_method, path), policy) in entries {
            assert!(path.starts_with("/backoffice/v1/"), "{path}");
            assert_eq!(
                policy,
                &Policy::rs_admin(&[authz::AdminSubRole::Content]),
                "{path}"
            );
        }
    }

    #[test]
    fn authenticated_routes_cannot_start_without_token_validation() {
        let renderer = Arc::new(crate::http_support::EnvelopeRenderer::new(
            "https://d.example/problems",
        ));
        let e = ops_router()
            .merge(backoffice_router())
            .finish(Enforcement::public_only(renderer))
            .err();
        assert_eq!(e, Some(PolicySetupError::MissingAuthentication));
    }
}
