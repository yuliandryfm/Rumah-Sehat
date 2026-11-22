//! Middleware penegak kebijakan dan ekstraktor [`Principal`].
//!
//! Urutan per permintaan (ADR-012 butir 1 dan 4, coding-standards §5.4 dan §7.3):
//! 1. Tanpa `MatchedPath` (rute tidak dikenal) atau tanpa kebijakan terdaftar → `404` (default deny;
//!    sama dengan rute yang tidak ada). Bila permintaan itu `POST`/`PUT`/`PATCH`/`DELETE` ke jalur audit
//!    (US-1504 AC2, F1-T68): token diperiksa (gagal → `401`), lalu `404`, dan keduanya mencatat security
//!    log `audit_modification_attempt`.
//! 2. Kebijakan publik → diteruskan.
//! 3. Token divalidasi `authn` (gagal → `401`, JWKS tidak tersedia → `503`).
//! 4. Realm token harus realm salah satu jenis akun kebijakan (realm lain → `401`).
//! 5. Akun dicari resolver: tidak ada, belum aktif, atau realm akun berbeda → `401` `4010001`; akun
//!    nonaktif → `401` `4010002` (KU-10 bagian sesi, layar "Akun nonaktif"); resolver gagal → `503`.
//! 6. Sesi diperiksa penjaga sesi: dicabut (keluar, dicabut admin, penonaktifan, perubahan peran) →
//!    `401` `4010001` (US-103 AC5, ADR-011 butir 8); penjaga gagal → `503`. Status akun diperiksa lebih
//!    dulu supaya sesi akun yang dinonaktifkan (sesinya juga dicabut) tetap menampilkan layar "Akun
//!    nonaktif".
//! 7. Jenis akun dan sub-peran admin dievaluasi: tidak diizinkan → `403` + security log
//!    `access_denied` (`role_not_allowed`).
//! 8. [`AuthContext`] dan [`Principal`] dipasang di extensions, handler dijalankan.

use crate::audit_guard;
use crate::denial::ClientIp;
use crate::policy::Policy;
use crate::principal::Principal;
use crate::registry::PolicyRegistry;
use crate::role::AccountStatus;
use crate::router::Authentication;
use authn::{AuthError, ErrorRenderer, RequestRenderer, missing_context_response};
use axum::extract::{FromRequestParts, MatchedPath, Request, State};
use axum::http::StatusCode;
use axum::http::request::Parts;
use axum::middleware::Next;
use axum::response::Response;
use envelope::CommonError;
use foundation::CorrelationId;
use observability::security::{self, AccessDenial};
use std::sync::Arc;

/// Keadaan middleware penegak.
pub(crate) struct EnforcerState {
    registry: PolicyRegistry,
    authentication: Option<Authentication>,
    renderer: Arc<dyn ErrorRenderer>,
}

impl EnforcerState {
    pub(crate) fn new(
        registry: PolicyRegistry,
        authentication: Option<Authentication>,
        renderer: Arc<dyn ErrorRenderer>,
    ) -> Self {
        Self {
            registry,
            authentication,
            renderer,
        }
    }
}

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

/// Percobaan ubah/hapus audit log ke jalur tanpa kebijakan (US-1504 AC2, [`audit_guard`]): selalu ditolak
/// dan dicatat security log `audit_modification_attempt`. Tanpa token atau token tidak sah → `401` (sama
/// dengan endpoint terautentikasi lain); token sah, peran apa pun → `404` seragam dengan rute tidak ada.
/// Handler tidak pernah dijalankan. Resolver akun hanya dipakai untuk mengisi `account_id` log; galatnya
/// tidak mengubah jawaban.
async fn reject_audit_modification(
    state: &EnforcerState,
    req: Request,
    method: &'static str,
    route: &str,
) -> Response {
    let cid = correlation(req.extensions());
    let Some((auth, resolver, _, _)) = &state.authentication else {
        security::audit_modification_attempt(None, method, route, false);
        return state.renderer.render(CommonError::NotFound, cid);
    };
    let (parts, _body) = req.into_parts();
    match auth.authenticate_parts(&parts).await {
        Err(e) => {
            security::audit_modification_attempt(None, method, route, false);
            auth.reject(&parts, e)
        }
        Ok(ctx) => {
            let account_id = match resolver.resolve(&ctx).await {
                Ok(Some(record)) if record.account_type.realm() == ctx.realm() => {
                    Some(record.account_id)
                }
                _ => None,
            };
            security::audit_modification_attempt(account_id, method, route, true);
            state.renderer.render(CommonError::NotFound, cid)
        }
    }
}

/// Middleware penegak; dipasang [`crate::PolicyRouter::finish`].
pub(crate) async fn enforce(
    State(state): State<Arc<EnforcerState>>,
    mut req: Request,
    next: Next,
) -> Response {
    let cid = correlation(req.extensions());
    let audit_attempt = audit_guard::modification_attempt(req.method(), req.uri().path());
    let Some(route) = req
        .extensions()
        .get::<MatchedPath>()
        .map(|m| m.as_str().to_string())
    else {
        if let Some(method) = audit_attempt {
            return reject_audit_modification(&state, req, method, "unmatched").await;
        }
        return state.renderer.render(CommonError::NotFound, cid);
    };
    let Some(policy) = state.registry.get(req.method(), &route).cloned() else {
        if let Some(method) = audit_attempt {
            return reject_audit_modification(&state, req, method, &route).await;
        }
        tracing::warn!(route = route.as_str(), "route_without_policy_denied");
        return state.renderer.render(CommonError::NotFound, cid);
    };
    req.extensions_mut()
        .insert(RequestRenderer::new(Arc::clone(&state.renderer)));

    let requirement = match policy {
        Policy::Public(_) => return next.run(req).await,
        Policy::Require(r) => r,
    };
    let Some((auth, resolver, sessions, denials)) = &state.authentication else {
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
    let rejection = if record.account_type.realm() != ctx.realm() {
        Some(("account_realm_mismatch", CommonError::Unauthenticated))
    } else {
        match record.status {
            AccountStatus::Active => None,
            AccountStatus::Inactive => Some(("account_inactive", CommonError::AccountInactive)),
            AccountStatus::PendingActivation => {
                Some(("account_pending_activation", CommonError::Unauthenticated))
            }
        }
    };
    if let Some((reason, error)) = rejection {
        tracing::info!(route = route.as_str(), reason = reason, "account_rejected");
        return state.renderer.render(error, cid);
    }
    match sessions.is_revoked(&ctx).await {
        Ok(false) => {}
        Ok(true) => {
            tracing::info!(
                route = route.as_str(),
                reason = "session_revoked",
                "account_rejected"
            );
            return state.renderer.render(CommonError::Unauthenticated, cid);
        }
        Err(_) => {
            tracing::warn!(route = route.as_str(), "session_guard_unavailable");
            return state.renderer.render(CommonError::Unavailable, cid);
        }
    }
    let client_ip = parts.extensions.get::<ClientIp>().map(|c| c.0);
    let principal = Principal::new(record, ctx.clone(), cid, client_ip, denials.clone());
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
        Err(missing_context_response(parts))
    }
}
