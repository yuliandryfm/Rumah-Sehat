//! Ekstraktor axum untuk [`AuthContext`] (coding-standards §1.2: handler mengambil konteks autentikasi
//! dari ekstraktor).
//!
//! Respons galat dibentuk aplikasi lewat [`ErrorRenderer`] (envelope ADR-023 dan `problem_type_base`
//! milik aplikasi), karena crate `envelope` sengaja tidak bergantung pada framework HTTP.

use crate::context::AuthContext;
use crate::error::AuthError;
use crate::verify::Authenticator;
use axum::extract::{FromRef, FromRequestParts, MatchedPath};
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;
use axum::response::Response;
use envelope::CommonError;
use foundation::CorrelationId;
use std::sync::Arc;

/// Pembentuk respons galat envelope milik aplikasi.
pub trait ErrorRenderer: Send + Sync + 'static {
    /// Membentuk respons galat untuk `error` dengan ID korelasi permintaan.
    fn render(&self, error: CommonError, correlation_id: CorrelationId) -> Response;
}

/// Keadaan autentikasi yang diambil ekstraktor dari state aplikasi lewat [`FromRef`].
#[derive(Clone)]
pub struct AuthState {
    authenticator: Arc<Authenticator>,
    renderer: Arc<dyn ErrorRenderer>,
}

impl AuthState {
    /// Menggabungkan validator dan pembentuk respons galat.
    pub fn new(authenticator: Arc<Authenticator>, renderer: Arc<dyn ErrorRenderer>) -> Self {
        Self {
            authenticator,
            renderer,
        }
    }

    /// Validator token.
    pub fn authenticator(&self) -> &Arc<Authenticator> {
        &self.authenticator
    }

    /// Pembentuk respons galat.
    pub fn renderer(&self) -> &Arc<dyn ErrorRenderer> {
        &self.renderer
    }

    /// Memvalidasi header `Authorization` dari `parts`.
    pub async fn authenticate_parts(&self, parts: &Parts) -> Result<AuthContext, AuthError> {
        self.authenticator
            .authenticate_headers(parts.headers.get_all(AUTHORIZATION).iter())
            .await
    }

    /// Membentuk respons penolakan untuk `error` dan mencatatnya ke application log (alasan dari daftar
    /// tertutup, tanpa isi token).
    pub fn reject(&self, parts: &Parts, error: AuthError) -> Response {
        let correlation_id = parts
            .extensions
            .get::<CorrelationId>()
            .copied()
            .unwrap_or_default();
        let route = parts
            .extensions
            .get::<MatchedPath>()
            .map_or("unmatched", MatchedPath::as_str);
        tracing::info!(
            route = route,
            reason = error.reason(),
            "authentication_rejected"
        );
        self.renderer
            .render(error.to_common_error(), correlation_id)
    }
}

/// Mengambil [`AuthContext`]: memakai konteks yang sudah dipasang middleware kebijakan di extensions
/// permintaan bila ada, atau memvalidasi header `Authorization` sendiri. Gagal = `401` (atau `503` bila
/// JWKS tidak tersedia) dengan envelope dari [`ErrorRenderer`].
impl<S> FromRequestParts<S> for AuthContext
where
    S: Send + Sync,
    AuthState: FromRef<S>,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        if let Some(ctx) = parts.extensions.get::<AuthContext>() {
            return Ok(ctx.clone());
        }
        let auth = AuthState::from_ref(state);
        match auth.authenticate_parts(parts).await {
            Ok(ctx) => {
                parts.extensions.insert(ctx.clone());
                Ok(ctx)
            }
            Err(e) => Err(auth.reject(parts, e)),
        }
    }
}
