//! Middleware penegak kebijakan dan ekstraktor [`Principal`].
//!
//! Urutan per permintaan (ADR-012 butir 1 dan 4, coding-standards §5.4 dan §7.3):
//! 1. Tanpa `MatchedPath` (rute tidak dikenal) atau tanpa kebijakan terdaftar → `404` (default deny;
//!    sama dengan rute yang tidak ada).
//! 2. Kebijakan publik → diteruskan.
//! 3. Token divalidasi `authn` (gagal → `401`, JWKS tidak tersedia → `503`).
//! 4. Realm token harus realm salah satu jenis akun kebijakan (realm lain → `401`).
//! 5. Akun dicari resolver: tidak ada, tidak aktif, atau realm akun berbeda → `401` (KU-10); resolver
//!    gagal → `503`.
//! 6. Jenis akun dan sub-peran admin dievaluasi: tidak diizinkan → `403` + security log
//!    `access_denied` (`role_not_allowed`).
//! 7. [`AuthContext`] dan [`Principal`] dipasang di extensions, handler dijalankan.

use crate::policy::Policy;
use crate::principal::{Principal, PrincipalResolver};
use crate::registry::PolicyRegistry;
use crate::role::AccountStatus;
use authn::{AuthError, AuthState, ErrorRenderer};
use axum::extract::{FromRequestParts, MatchedPath, Request, State};
use axum::http::StatusCode;
use axum::http::request::Parts;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use envelope::CommonError;
use foundation::CorrelationId;
use observability::security::{self, AccessDenial};
use std::sync::Arc;

/// Keadaan middleware penegak.
pub(crate) struct EnforcerState {
    registry: PolicyRegistry,
    authentication: Option<(AuthState, Arc<dyn PrincipalResolver>)>,
    renderer: Arc<dyn ErrorRenderer>,
}

impl EnforcerState {
    pub(crate) fn new(
        registry: PolicyRegistry,
        authentication: Option<(AuthState, Arc<dyn PrincipalResolver>)>,
        renderer: Arc<dyn ErrorRenderer>,
    ) -> Self {
        Self {
            registry,
            authentication,
            renderer,
        }
    }
}

/// Pembentuk respons galat yang dipasang middleware di extensions untuk ekstraktor.
#[derive(Clone)]
struct RendererHandle(Arc<dyn ErrorRenderer>);

fn correlation(parts_ext: &axum::http::Extensions) -> CorrelationId {
    parts_ext
        .get::<CorrelationId>()
        .copied()
        .unwrap_or_default()
}

/// Fallback router (jaring pengaman): permintaan ke rute tidak dikenal sudah dijawab `404` oleh
/// middleware sebelum mencapai fallback.
pub(crate) async fn deny_fallback() -> StatusCode {
    StatusCode::NOT_FOUND
}

/// Middleware penegak; dipasang [`crate::PolicyRouter::finish`].
pub(crate) async fn enforce(
    State(state): State<Arc<EnforcerState>>,
    mut req: Request,
    next: Next,
) -> Response {
    let cid = correlation(req.extensions());
    let Some(route) = req
        .extensions()
        .get::<MatchedPath>()
        .map(|m| m.as_str().to_string())
    else {
        return state.renderer.render(CommonError::NotFound, cid);
    };
    let Some(policy) = state.registry.get(req.method(), &route).cloned() else {
        tracing::warn!(route = route.as_str(), "route_without_policy_denied");
        return state.renderer.render(CommonError::NotFound, cid);
    };
    req.extensions_mut()
        .insert(RendererHandle(Arc::clone(&state.renderer)));

    let requirement = match policy {
        Policy::Public(_) => return next.run(req).await,
        Policy::Require(r) => r,
    };
    let Some((auth, resolver)) = &state.authentication else {
        // Dicegah pemeriksaan saat start; bila tetap terjadi, tolak.
        tracing::error!(route = route.as_str(), "authentication_not_configured");
        return state.renderer.render(CommonError::Internal, cid);
    };

    let (mut parts, body) = req.into_parts();
    let ctx = match auth.authenticate_parts(&parts).await {
        Ok(ctx) => ctx,
        Err(e) => return auth.reject(&parts, e),
    };
    if !requirement.accepts_realm(ctx.realm()) {
        return auth.reject(&parts, AuthError::RealmNotAccepted);
    }
    let record = match resolver.resolve(&ctx).await {
        Ok(Some(record)) => record,
        Ok(None) => {
            tracing::info!(
                route = route.as_str(),
                reason = "account_not_found",
                "account_rejected"
            );
            return state.renderer.render(CommonError::Unauthenticated, cid);
        }
        Err(_) => {
            tracing::warn!(route = route.as_str(), "account_resolver_unavailable");
            return state.renderer.render(CommonError::Unavailable, cid);
        }
    };
    if record.status != AccountStatus::Active || record.account_type.realm() != ctx.realm() {
        let reason = if record.status != AccountStatus::Active {
            "account_not_active"
        } else {
            "account_realm_mismatch"
        };
        tracing::info!(route = route.as_str(), reason = reason, "account_rejected");
        return state.renderer.render(CommonError::Unauthenticated, cid);
    }
    let principal = Principal::new(record, ctx.clone());
    if !requirement.allows(&principal) {
        security::access_denied(
            Some(principal.account_id()),
            &route,
            AccessDenial::RoleNotAllowed,
        );
        return state.renderer.render(CommonError::Forbidden, cid);
    }
    parts.extensions.insert(ctx);
    parts.extensions.insert(principal);
    next.run(Request::from_parts(parts, body)).await
}

/// Mengambil [`Principal`] yang dipasang middleware penegak. Tidak ada (endpoint publik atau middleware
/// tidak terpasang) = kesalahan pemrograman → `500`, tidak pernah dilewatkan.
impl<S> FromRequestParts<S> for Principal
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        if let Some(p) = parts.extensions.get::<Principal>() {
            return Ok(p.clone());
        }
        tracing::error!("principal_missing");
        match parts.extensions.get::<RendererHandle>() {
            Some(h) => Err(h
                .0
                .render(CommonError::Internal, correlation(&parts.extensions))),
            None => Err(StatusCode::INTERNAL_SERVER_ERROR.into_response()),
        }
    }
}
