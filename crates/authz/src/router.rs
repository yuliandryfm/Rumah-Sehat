//! Pembungkus router yang mewajibkan kebijakan (KEPUTUSAN-127).
//!
//! Rute hanya didaftarkan lewat [`PolicyRouter::routes`], yang meminta [`Policy`] untuk setiap kelompok
//! rute `utoipa_axum::routes!`. [`PolicyRouter::finish`] menjalankan pemeriksaan saat start (setiap
//! operasi punya kebijakan, tidak ada kebijakan basi atau ganda, kebijakan dapat dipenuhi, realm yang
//! dibutuhkan diterima validator token), lalu memasang middleware penegak sebagai lapisan terakhir
//! router, sehingga rute tanpa kebijakan juga ditolak saat runtime (default deny, ADR-012 butir 1).
//!
//! Router hasil `finish` tidak boleh ditambah rute lagi: rute yang ditambahkan setelah lapisan dipasang
//! tidak dibungkus middleware (perilaku `Router::layer` axum). Aturan ini ditegakkan review
//! (coding-standards §10).

use crate::enforce::{EnforcerState, deny_fallback, enforce};
use crate::error::PolicySetupError;
use crate::policy::Policy;
use crate::principal::PrincipalResolver;
use crate::registry::{PolicyRegistry, check_coverage, operations};
use authn::{AuthState, ErrorRenderer};
use axum::Router;
use std::sync::Arc;
use utoipa::openapi::OpenApi;
use utoipa_axum::router::{OpenApiRouter, UtoipaMethodRouter};

/// Perangkat penegakan yang dipasang [`PolicyRouter::finish`].
#[derive(Clone)]
pub struct Enforcement {
    authentication: Option<(AuthState, Arc<dyn PrincipalResolver>)>,
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

    /// Validator token (`authn`) dan resolver akun. Pembentuk respons galat diambil dari `auth`.
    pub fn new(auth: AuthState, resolver: Arc<dyn PrincipalResolver>) -> Self {
        let renderer = Arc::clone(auth.renderer());
        Self {
            authentication: Some((auth, resolver)),
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
    ) -> Result<(Router<S>, OpenApi), PolicySetupError> {
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
            let Some((auth, _)) = &enforcement.authentication else {
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
        Ok((router, spec))
    }
}
