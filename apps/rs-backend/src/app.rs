//! Perakitan aplikasi HTTP: router berkebijakan (KEPUTUSAN-127, crate `authz`), validasi token (`authn`)
//! dan resolver akun, middleware korelasi (KEPUTUSAN-123), dan penolakan berformat envelope.

use crate::modules::content_support;
use crate::modules::corporate::api::CorporateDirectory;
use crate::modules::platform;
use crate::modules::platform::api::{FileStorage, ReadinessCheck};
use crate::modules::profile;
use crate::modules::profile::api::IdentityServices;
use crate::openapi::{BackofficeApiDoc, OpsApiDoc, PatientApiDoc, PublicApiDoc};
use crate::rate_limit::RateLimiter;
use authn::AuthState;
use authz::{
    ClientIp, DenialSink, EndpointKey, EnforcedRouter, Enforcement, Policy, PolicyRouter,
    PolicySetupError, PrincipalResolver, SessionGuard, is_corporate_path,
};
use axum::extract::{ConnectInfo, Request};
use axum::middleware::Next;
use axum::response::Response;
use foundation::Clock;
use observability::correlation_middleware;
use sqlx::PgPool;
use std::net::SocketAddr;
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
    /// Use case identitas pasien (modul profile, F1-T60/F1-T61): DEK lewat Transit, blind index NIK,
    /// batas NIK berlapis di Valkey.
    pub identity: Arc<IdentityServices>,
    /// Batas laju endpoint konten publik (F1-T94), terpisah dari API lain (M-47).
    pub public_content_limit: Arc<RateLimiter>,
    /// Batas laju endpoint gambar konten publik (F1-T93).
    pub public_image_limit: Arc<RateLimiter>,
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

    pub(crate) fn files(&self) -> &FileStorage {
        &self.services.files
    }

    pub(crate) fn identity(&self) -> &IdentityServices {
        &self.services.identity
    }

    pub(crate) fn public_content_limit(&self) -> &RateLimiter {
        &self.services.public_content_limit
    }

    pub(crate) fn public_image_limit(&self) -> &RateLimiter {
        &self.services.public_image_limit
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

/// Router API aplikasi pasien (`openapi/pasien.json`, awalan `/patient/v1`, KEPUTUSAN-120, realm `pasien`).
pub(crate) fn patient_router() -> PolicyRouter<AppState> {
    PolicyRouter::with_openapi(PatientApiDoc::openapi()).merge(profile::patient_router())
}

/// Router API publik tanpa login (`openapi/public.json`, awalan `/v1/public/content`, ADR-027 butir 2 dan 5,
/// ADR-012 butir 3). Jaringan `web/situs` hanya menjangkau `/v1/public/*` (ADR-027 butir 6).
pub(crate) fn public_router() -> PolicyRouter<AppState> {
    PolicyRouter::with_openapi(PublicApiDoc::openapi()).merge(content_support::public_router())
}

/// Endpoint terdaftar di namespace API korporat `/v1/corporate/` beserta kebijakannya (F1-T100, ADR-028
/// butir 1). Dipakai harness isolasi `tests/corporate_isolation.rs`, yang gagal bila ada endpoint di sini
/// tanpa kasus harness (H-18).
pub fn corporate_endpoints() -> Vec<(EndpointKey, Policy)> {
    all_routes()
        .registry()
        .iter()
        .filter(|((_, path), _)| is_corporate_path(path))
        .map(|(k, p)| (k.clone(), p.clone()))
        .collect()
}

/// Semua endpoint terautentikasi proses `api` di luar namespace korporat (F1-T100): harness memastikan
/// token realm `korporat` ditolak di semuanya (US-1904 AC4).
pub fn non_corporate_authenticated_endpoints() -> Vec<EndpointKey> {
    all_routes()
        .registry()
        .iter()
        .filter(|((_, path), p)| !is_corporate_path(path) && matches!(p, Policy::Require(_)))
        .map(|(k, _)| k.clone())
        .collect()
}

/// Semua rute proses `api` (ops, backoffice, pasien, publik), masing-masing dengan kebijakannya.
pub(crate) fn all_routes() -> PolicyRouter<AppState> {
    PolicyRouter::new()
        .merge(ops_router())
        .merge(backoffice_router())
        .merge(patient_router())
        .merge(public_router())
}

/// Memasang [`ClientIp`] dari alamat peer TCP (`ConnectInfo`, tersedia lewat
/// `EnforcedRouter::serve_with_peer_addr`) untuk field "Sumber" audit (PRD §7.2) dan batas laju per sumber.
/// Tidak ada header klien yang dipercaya (IP asli di balik proxy `[TIDAK DIKETAHUI]`, PT-39).
async fn client_ip_middleware(mut req: Request, next: Next) -> Response {
    if let Some(ip) = req
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|c| c.0.ip())
    {
        req.extensions_mut().insert(ClientIp(ip));
    }
    next.run(req).await
}

/// Router lengkap siap dilayani (`EnforcedRouter::serve_with_peer_addr`).
///
/// - `PolicyRouter::finish` memeriksa saat start bahwa setiap rute punya kebijakan, kebijakan dapat
///   dipenuhi, dan realm yang dibutuhkan diterima validator token (galat = start gagal); lalu memasang
///   middleware penegak: rute tidak dikenal, metode tidak diizinkan, atau rute tanpa kebijakan dijawab
///   `404` envelope (`4040001`). Registry `docs/05-response-codes.md` tidak memuat kode 405, dan 404 tidak
///   membedakan "tidak ada" dari "tidak boleh" (KU-03).
/// - Endpoint terautentikasi memakai `auth` (validasi token realm `admin` untuk backoffice dan realm `pasien`
///   untuk `/patient/v1`), `resolver` dan
///   `sessions` (modul identity `AccountDirectory`, F1-T33), dan `denials` (E-05 hasil gagal, F1-T38).
/// - Endpoint HR perusahaan memakai pembaca keanggotaan aktif modul `corporate` di atas pool aplikasi
///   ([`CorporateDirectory`], F1-T100, ADR-028 butir 1).
/// - Middleware korelasi dan IP klien dipasang terakhir dengan `EnforcedRouter::layer`, sehingga membungkus
///   penegak dan `MatchedPath` tersedia. Router hasil `finish` tertutup untuk penambahan rute (TD-15).
pub fn build_router(
    state: AppState,
    auth: AuthState,
    resolver: Arc<dyn PrincipalResolver>,
    sessions: Arc<dyn SessionGuard>,
    denials: DenialSink,
) -> Result<EnforcedRouter, PolicySetupError> {
    let corporate = Arc::new(CorporateDirectory::new(state.pool().clone()));
    let (router, _spec) = all_routes().finish(
        Enforcement::new(auth, resolver, sessions, denials).with_corporate_scopes(corporate),
    )?;
    Ok(router
        .with_state(state)
        .layer(axum::middleware::from_fn(client_ip_middleware))
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
        assert_eq!(entries.len(), 10);
        for ((_method, path), policy) in entries {
            assert!(path.starts_with("/backoffice/v1/"), "{path}");
            assert_eq!(
                policy,
                &Policy::rs_admin(&[authz::AdminSubRole::Content]),
                "{path}"
            );
        }
    }

    // trace: ADR-012 butir 1, KEPUTUSAN-120 (endpoint pasien hanya untuk akun pasien, awalan /patient/v1)
    #[test]
    fn every_patient_route_requires_patient_account_only() {
        let router = patient_router();
        let entries: Vec<_> = router.registry().iter().collect();
        assert_eq!(entries.len(), 4);
        for ((_method, path), policy) in entries {
            assert!(path.starts_with("/patient/v1/"), "{path}");
            assert_eq!(policy, &Policy::patient(), "{path}");
        }
    }

    // trace: ADR-012 butir 3
    #[test]
    fn every_public_route_is_public_content_under_v1_public() {
        let router = public_router();
        let entries: Vec<_> = router.registry().iter().collect();
        assert_eq!(entries.len(), 4);
        for ((method, path), policy) in entries {
            assert_eq!(method, "GET", "{path}");
            assert!(path.starts_with("/v1/public/content/"), "{path}");
            assert_eq!(
                policy,
                &Policy::Public(PublicReason::PublicContent),
                "{path}"
            );
        }
    }

    #[test]
    fn authenticated_routes_cannot_start_without_token_validation() {
        let renderer = Arc::new(crate::http_support::EnvelopeRenderer::new(
            "https://d.example/problems",
        ));
        let e = all_routes()
            .finish(Enforcement::public_only(renderer))
            .err();
        assert_eq!(e, Some(PolicySetupError::MissingAuthentication));
    }
}
