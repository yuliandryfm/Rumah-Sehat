//! Pembungkus router yang mewajibkan kebijakan (KEPUTUSAN-127).
//!
//! Rute hanya didaftarkan lewat [`PolicyRouter::routes`], yang meminta [`Policy`] untuk setiap kelompok
//! rute `utoipa_axum::routes!`. [`PolicyRouter::finish`] menjalankan pemeriksaan saat start (setiap
//! operasi punya kebijakan, tidak ada kebijakan basi atau ganda, kebijakan dapat dipenuhi, realm yang
//! dibutuhkan diterima validator token), lalu memasang middleware penegak sebagai lapisan terakhir
//! router, sehingga rute tanpa kebijakan juga ditolak saat runtime (default deny, ADR-012 butir 1).
//!
//! Router hasil `finish` adalah [`EnforcedRouter`], newtype tertutup yang hanya menyediakan `layer`,
//! `with_state`, dan `serve` (TD-15, review F1-T32/T37). Rute yang ditambahkan ke `axum::Router` setelah
//! lapisan dipasang tidak dibungkus middleware (perilaku `Router::layer` axum); karena `axum::Router` di
//! dalamnya tidak pernah keluar, penambahan itu menjadi galat compile. Test arsitektur (aturan
//! `router-bypass`, crate arch-check) menolak pembuatan router axum sendiri di `apps/*/src`.

use crate::denial::DenialSink;
use crate::enforce::{EnforcerState, deny_fallback, enforce};
use crate::error::PolicySetupError;
use crate::policy::Policy;
use crate::principal::{PrincipalResolver, SessionGuard};
use crate::registry::{PolicyRegistry, check_coverage, operations};
use authn::{AuthState, ErrorRenderer};
use axum::Router;
use axum::extract::Request;
use axum::response::IntoResponse;
use axum::routing::Route;
use std::convert::Infallible;
use std::future::Future;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_layer::Layer;
use tower_service::Service;
use utoipa::openapi::OpenApi;
use utoipa_axum::router::{OpenApiRouter, UtoipaMethodRouter};

/// Validator token, resolver akun, penjaga sesi, dan antrean penolakan objek.
pub(crate) type Authentication = (
    AuthState,
    Arc<dyn PrincipalResolver>,
    Arc<dyn SessionGuard>,
    DenialSink,
);

/// Perangkat penegakan yang dipasang [`PolicyRouter::finish`].
#[derive(Clone)]
pub struct Enforcement {
    authentication: Option<Authentication>,
    renderer: Arc<dyn ErrorRenderer>,
}

impl Enforcement {
    /// Hanya endpoint publik. Bila ada endpoint terautentikasi, start gagal
    /// ([`PolicySetupError::MissingAuthentication`]).
    pub fn public_only(renderer: Arc<dyn ErrorRenderer>) -> Self {
        Self {
            authentication: None,
            renderer,
        }
    }

    /// Validator token (`authn`), resolver akun, penjaga sesi yang dicabut (ADR-011 butir 8), dan
    /// antrean pencatatan penolakan objek ([`DenialSink`], F1-T38). Pembentuk respons galat diambil dari
    /// `auth`.
    pub fn new(
        auth: AuthState,
        resolver: Arc<dyn PrincipalResolver>,
        sessions: Arc<dyn SessionGuard>,
        denials: DenialSink,
    ) -> Self {
        let renderer = Arc::clone(auth.renderer());
        Self {
            authentication: Some((auth, resolver, sessions, denials)),
            renderer,
        }
    }
}

/// Router OpenAPI yang setiap rutenya wajib berkebijakan.
pub struct PolicyRouter<S = ()> {
    inner: OpenApiRouter<S>,
    registry: PolicyRegistry,
    errors: Vec<PolicySetupError>,
}

impl<S> Default for PolicyRouter<S>
where
    S: Send + Sync + Clone + 'static,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<S> PolicyRouter<S>
where
    S: Send + Sync + Clone + 'static,
{
    /// Router kosong.
    pub fn new() -> Self {
        Self::with_openapi(OpenApi::default())
    }

    /// Router dengan dokumen OpenAPI dasar (info, tag). Path yang sudah ada di dokumen dasar tanpa
    /// kebijakan membuat start gagal.
    pub fn with_openapi(openapi: OpenApi) -> Self {
        Self {
            inner: OpenApiRouter::with_openapi(openapi),
            registry: PolicyRegistry::default(),
            errors: Vec::new(),
        }
    }

    /// Mendaftarkan satu kelompok rute (`utoipa_axum::routes!(...)`) dengan kebijakannya. Kebijakan
    /// berlaku untuk semua operasi dalam kelompok itu; bedakan kebijakan per metode dengan kelompok
    /// terpisah.
    ///
    /// Endpoint ganda dicatat sebagai galat yang dikembalikan [`PolicyRouter::finish`]; rutenya tidak
    /// dipasang (axum akan panik pada metode yang tumpang tindih).
    #[must_use]
    pub fn routes(mut self, policy: Policy, routes: UtoipaMethodRouter<S>) -> Self {
        let mut ok = true;
        for (path, item) in &routes.1.paths {
            for method in operations(item) {
                if let Err(e) = self.registry.insert(&method, path, policy.clone()) {
                    self.errors.push(e);
                    ok = false;
                }
            }
        }
        if ok {
            self.inner = self.inner.routes(routes);
        }
        self
    }

    /// Menggabungkan router berkebijakan lain (misalnya router modul).
    /// Endpoint yang sudah ada di router ini dicatat sebagai galat untuk [`PolicyRouter::finish`]; router
    /// lain tidak dipasang.
    #[must_use]
    pub fn merge(mut self, other: PolicyRouter<S>) -> Self {
        self.errors.extend(other.errors);
        let mut ok = true;
        for ((method, path), policy) in other.registry.iter() {
            let parsed = axum::http::Method::from_bytes(method.as_bytes());
            let result = match parsed {
                Ok(m) => self.registry.insert(&m, path, policy.clone()),
                Err(_) => Err(PolicySetupError::MissingPolicy {
                    method: method.clone(),
                    path: path.clone(),
                }),
            };
            if let Err(e) = result {
                self.errors.push(e);
                ok = false;
            }
        }
        if ok {
            self.inner = self.inner.merge(other.inner);
        }
        self
    }

    /// Spesifikasi OpenAPI (untuk pembangkitan berkas `openapi/`), tanpa memasang router.
    pub fn into_openapi(self) -> OpenApi {
        self.inner.into_openapi()
    }

    /// Registri kebijakan (untuk test dan peninjauan).
    pub fn registry(&self) -> &PolicyRegistry {
        &self.registry
    }

    /// Pemeriksaan saat start dan pemasangan middleware penegak. Galat = proses gagal start.
    pub fn finish(
        self,
        enforcement: Enforcement,
    ) -> Result<(EnforcedRouter<S>, OpenApi), PolicySetupError> {
        if let Some(e) = self.errors.into_iter().next() {
            return Err(e);
        }
        let (router, spec) = self.inner.split_for_parts();
        check_coverage(&spec, &self.registry)?;

        for ((method, path), policy) in self.registry.iter() {
            let Policy::Require(req) = policy else {
                continue;
            };
            if !req.is_satisfiable() {
                return Err(PolicySetupError::UnsatisfiablePolicy {
                    method: method.clone(),
                    path: path.clone(),
                });
            }
            let Some((auth, _, _, _)) = &enforcement.authentication else {
                return Err(PolicySetupError::MissingAuthentication);
            };
            let accepted = auth.authenticator().realms();
            if let Some(missing) = req.realms().into_iter().find(|r| !accepted.contains(r)) {
                return Err(PolicySetupError::RealmNotConfigured {
                    realm: missing.as_str(),
                });
            }
        }

        let state = Arc::new(EnforcerState::new(
            self.registry,
            enforcement.authentication,
            enforcement.renderer,
        ));
        let router = router
            .fallback(deny_fallback)
            .method_not_allowed_fallback(deny_fallback)
            .layer(axum::middleware::from_fn_with_state(state, enforce));
        Ok((EnforcedRouter { inner: router }, spec))
    }
}

/// Router yang semua rutenya sudah dibungkus middleware penegak ([`PolicyRouter::finish`]).
///
/// Newtype tertutup (TD-15): tidak ada cara menambah rute, fallback, atau router lain, dan
/// `axum::Router` di dalamnya tidak dapat diambil. Yang tersedia hanya lapisan luar
/// ([`EnforcedRouter::layer`], misalnya middleware korelasi), pemasangan state, dan
/// [`EnforcedRouter::serve`].
///
/// ```compile_fail
/// // Penambahan rute setelah `finish` tidak dapat dikompilasi.
/// fn tambah(r: authz::EnforcedRouter) {
///     let _ = r.route("/x", axum::routing::get(|| async { "x" }));
/// }
/// ```
///
/// ```compile_fail
/// // `axum::Router` di dalamnya tidak dapat diambil.
/// fn ambil(r: authz::EnforcedRouter) -> axum::Router {
///     r.inner
/// }
/// ```
///
/// ```
/// // Pembanding positif untuk dua contoh di atas: lapisan luar dan state tetap dapat dipasang.
/// async fn lewat(req: axum::extract::Request, next: axum::middleware::Next) -> axum::response::Response {
///     next.run(req).await
/// }
/// fn rakit(r: authz::EnforcedRouter<u8>) -> authz::EnforcedRouter {
///     r.with_state(1).layer(axum::middleware::from_fn(lewat))
/// }
/// ```
pub struct EnforcedRouter<S = ()> {
    inner: Router<S>,
}

impl<S> EnforcedRouter<S>
where
    S: Clone + Send + Sync + 'static,
{
    /// Lapisan luar yang membungkus semua rute dan middleware penegak (batas trait sama dengan
    /// `axum::Router::layer` axum 0.8.9).
    #[must_use]
    pub fn layer<L>(self, layer: L) -> Self
    where
        L: Layer<Route> + Clone + Send + Sync + 'static,
        L::Service: Service<Request> + Clone + Send + Sync + 'static,
        <L::Service as Service<Request>>::Response: IntoResponse + 'static,
        <L::Service as Service<Request>>::Error: Into<Infallible> + 'static,
        <L::Service as Service<Request>>::Future: Send + 'static,
    {
        Self {
            inner: self.inner.layer(layer),
        }
    }

    /// Memasang state aplikasi.
    #[must_use]
    pub fn with_state<S2>(self, state: S) -> EnforcedRouter<S2> {
        EnforcedRouter {
            inner: self.inner.with_state(state),
        }
    }
}

impl EnforcedRouter<()> {
    /// Melayani HTTP di `listener` sampai `shutdown` selesai (`axum::serve` dengan graceful shutdown).
    pub async fn serve<F>(self, listener: TcpListener, shutdown: F) -> std::io::Result<()>
    where
        F: Future<Output = ()> + Send + 'static,
    {
        axum::serve(listener, self.inner)
            .with_graceful_shutdown(shutdown)
            .await
    }
}
