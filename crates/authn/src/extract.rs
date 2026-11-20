//! Ekstraktor axum untuk [`AuthContext`] (coding-standards §1.2: handler mengambil konteks autentikasi
//! dari ekstraktor).
//!
//! Ekstraktor **tidak** memvalidasi token sendiri (TD-15, review F1-T32/T37): ia hanya mengambil konteks
//! yang dipasang middleware penegak crate `authz` setelah token, status akun, sesi yang dicabut, dan
//! kebijakan endpoint lolos. Konteks tidak ada (endpoint publik, atau middleware tidak terpasang) =
//! kesalahan pemrograman → `500`, tidak pernah dilewatkan. Dengan begitu tidak ada jalur yang memperoleh
//! [`AuthContext`] tanpa resolver akun (KU-10).
//!
//! Respons galat dibentuk aplikasi lewat [`ErrorRenderer`] (envelope ADR-023 dan `problem_type_base`
//! milik aplikasi), karena crate `envelope` sengaja tidak bergantung pada framework HTTP.

use crate::context::AuthContext;
use crate::error::AuthError;
use crate::verify::Authenticator;
use axum::extract::{FromRequestParts, MatchedPath};
use axum::http::StatusCode;
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;
use axum::response::{IntoResponse, Response};
use envelope::CommonError;
use foundation::CorrelationId;
use std::sync::Arc;

/// Pembentuk respons galat envelope milik aplikasi.
pub trait ErrorRenderer: Send + Sync + 'static {
    /// Membentuk respons galat untuk `error` dengan ID korelasi permintaan.
    fn render(&self, error: CommonError, correlation_id: CorrelationId) -> Response;
}

/// Pembentuk respons galat yang dipasang middleware penegak (crate `authz`) di extensions setiap
/// permintaan berkebijakan, supaya ekstraktor dapat menjawab `500` dengan envelope.
#[derive(Clone)]
pub struct RequestRenderer(Arc<dyn ErrorRenderer>);

impl RequestRenderer {
    /// Membungkus pembentuk respons galat aplikasi.
    pub fn new(renderer: Arc<dyn ErrorRenderer>) -> Self {
        Self(renderer)
    }
}

/// Respons `500` untuk ekstraktor yang tidak menemukan konteks dari middleware penegak: envelope bila
/// [`RequestRenderer`] terpasang, status polos bila tidak (middleware sama sekali tidak terpasang).
pub fn missing_context_response(parts: &Parts) -> Response {
    let correlation_id = parts
        .extensions
        .get::<CorrelationId>()
        .copied()
        .unwrap_or_default();
    match parts.extensions.get::<RequestRenderer>() {
        Some(r) => r.0.render(CommonError::Internal, correlation_id),
        None => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

/// Validator token dan pembentuk respons galat, dipakai middleware penegak crate `authz`.
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

/// Mengambil [`AuthContext`] yang dipasang middleware penegak crate `authz` (TD-15). Tidak memvalidasi
/// header `Authorization` sendiri. Konteks tidak ada → `500` (envelope bila [`RequestRenderer`]
/// terpasang) dan application log `auth_context_missing`.
impl<S> FromRequestParts<S> for AuthContext
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        if let Some(ctx) = parts.extensions.get::<AuthContext>() {
            return Ok(ctx.clone());
        }
        let route = parts
            .extensions
            .get::<MatchedPath>()
            .map_or("unmatched", MatchedPath::as_str);
        tracing::error!(route = route, "auth_context_missing");
        Err(missing_context_response(parts))
    }
}
