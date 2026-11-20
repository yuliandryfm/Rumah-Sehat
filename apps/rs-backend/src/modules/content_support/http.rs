//! Handler CMS backoffice (`/backoffice/v1`, KEPUTUSAN-120). Setiap rute berkebijakan sub-peran Konten
//! (`policy.rs`); penegakan token, status akun, dan peran oleh middleware crate `authz` sebelum handler.
//! Handler hanya mengurai DTO, memanggil satu fungsi service, dan memetakan hasil ke envelope ADR-023.

use super::domain::{DisplayStatus, SitePage};
use super::dto::{
    AttachImageRequest, ContentDto, ContentEnvelope, ContentListEnvelope, ContentRequest,
    DeleteRequest, DeletedDto, DeletedEnvelope, ImageSlotRequest, ListParams, PublicContentDto,
    PublicContentEnvelope, PublicContentListEnvelope, PublicListParams, ScheduleRequest,
    UploadSlotDto, UploadSlotEnvelope,
};
use super::error::ContentError;
use super::policy;
use super::service::{self, ContentView, ListQuery, PublicContent, PublicQuery};
use crate::app::AppState;
use crate::http_support::{ApiFailure, ApiSuccess, ClientAddr};
use crate::openapi::ErrorEnvelope;
use crate::rate_limit::Verdict;
use audit::{AuditContext, Source, SourceComponent};
use authz::{PolicyRouter, Principal};
use axum::Extension;
use axum::extract::rejection::{JsonRejection, QueryRejection};
use axum::extract::{Json, Path, Query, State};
use axum::http::header::{
    CACHE_CONTROL, CONTENT_SECURITY_POLICY, CONTENT_TYPE, USER_AGENT, X_CONTENT_TYPE_OPTIONS,
};
use axum::http::{HeaderMap, HeaderValue};
use axum::response::{IntoResponse, Response};
use envelope::{FieldError, Success};
use foundation::{CorrelationId, Uuid};
use observability::security;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use utoipa_axum::routes;

pub(super) fn router() -> PolicyRouter<AppState> {
    PolicyRouter::new()
        .routes(policy::manage_content(), routes!(list, create))
        .routes(policy::manage_content(), routes!(get_one, update))
        .routes(policy::manage_content(), routes!(schedule))
        .routes(policy::manage_content(), routes!(archive))
        .routes(policy::manage_content(), routes!(delete))
        .routes(policy::manage_content(), routes!(restore))
        .routes(policy::manage_content(), routes!(image_upload_slot))
        .routes(policy::manage_content(), routes!(attach_image))
}

fn fail(state: &AppState, cid: CorrelationId, e: ContentError) -> Response {
    if let ContentError::Internal(what) = &e {
        tracing::error!(error = *what, "cms_request_failed");
    }
    ApiFailure::new(e, cid, state.problem_type_base()).into_response()
}

fn respond(
    state: &AppState,
    cid: CorrelationId,
    result: Result<ContentView, ContentError>,
    created: bool,
) -> Response {
    let dto = result.and_then(|v| {
        ContentDto::from_view(v).map_err(|_| ContentError::Internal("timestamp_format"))
    });
    match dto {
        Ok(d) => {
            let body = if created {
                Success::created(d)
            } else {
                Success::ok(d)
            };
            ApiSuccess(body.with_correlation_id(cid)).into_response()
        }
        Err(e) => fail(state, cid, e),
    }
}

/// ID di path yang bukan UUID = konten tidak ada (404, sama dengan konten yang tidak ada).
fn content_id(raw: &str) -> Result<Uuid, ContentError> {
    Uuid::parse_str(raw).map_err(|_| ContentError::NotFound)
}

/// Galat pembacaan badan JSON. Teks galat serde tidak diteruskan (dapat memuat nilai masukan).
fn body_error(e: &JsonRejection) -> ContentError {
    match e {
        JsonRejection::MissingJsonContentType(_) => ContentError::UnsupportedMediaType,
        JsonRejection::JsonDataError(_) => ContentError::Validation(vec![FieldError::new(
            "$",
            "invalid_format",
            "Isian tidak lengkap, bertipe salah, atau tidak dikenal",
        )]),
        _ => ContentError::BadRequest,
    }
}

fn query_error(_e: &QueryRejection) -> ContentError {
    ContentError::Validation(vec![FieldError::new(
        "$query",
        "invalid_format",
        "Parameter saringan tidak valid",
    )])
}

fn instant(field: &str, raw: Option<&str>) -> Result<Option<OffsetDateTime>, FieldError> {
    raw.map(|s| {
        OffsetDateTime::parse(s, &Rfc3339).map_err(|_| {
            FieldError::new(
                field,
                "invalid_format",
                "Waktu harus ISO 8601 dengan offset zona, misalnya 2027-03-01T08:00:00+07:00",
            )
        })
    })
    .transpose()
}

fn audit_context(cid: CorrelationId, addr: ClientAddr, headers: &HeaderMap) -> AuditContext {
    let mut source = Source::component(SourceComponent::Backoffice);
    if let Some(ip) = addr.0 {
        source = source.ip(ip);
    }
    if let Some(ua) = headers.get(USER_AGENT).and_then(|v| v.to_str().ok()) {
        source = source.user_agent(ua);
    }
    AuditContext::new(cid, source)
}

/// Daftar konten dengan saringan status, kanal, dan halaman situs (wireframe C14).
#[utoipa::path(
    get,
    path = "/backoffice/v1/contents",
    tag = "cms",
    operation_id = "cms_content_list",
    security(("bearer" = [])),
    params(ListParams),
    responses(
        (status = 200, description = "Daftar konten.", body = ContentListEnvelope, content_type = "application/json"),
        (status = 401, description = "Belum login, token tidak sah, atau akun nonaktif.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 403, description = "Akun tanpa sub-peran Konten.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 422, description = "Saringan tidak valid.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 503, description = "Layanan tidak tersedia.", body = ErrorEnvelope, content_type = "application/problem+json")
    )
)]
async fn list(
    State(state): State<AppState>,
    Extension(cid): Extension<CorrelationId>,
    _principal: Principal,
    params: Result<Query<ListParams>, QueryRejection>,
) -> Response {
    let Query(p) = match params {
        Ok(p) => p,
        Err(e) => return fail(&state, cid, query_error(&e)),
    };
    let cursor = match p.cursor.as_deref().map(Uuid::parse_str).transpose() {
        Ok(c) => c,
        Err(_) => {
            return fail(
                &state,
                cid,
                ContentError::Validation(vec![FieldError::new(
                    "cursor",
                    "invalid_format",
                    "Cursor tidak valid",
                )]),
            );
        }
    };
    let q = ListQuery {
        status: p.status,
        channel: p.channel,
        site_page: p.site_page,
        cursor,
        limit: p.limit,
    };
    match service::list(state.pool(), state.clock(), q).await {
        Ok((items, meta)) => {
            let dtos = items
                .into_iter()
                .map(ContentDto::from_view)
                .collect::<Result<Vec<_>, _>>();
            match dtos {
                Ok(d) => ApiSuccess(Success::ok(d).with_meta(meta).with_correlation_id(cid))
                    .into_response(),
                Err(_) => fail(&state, cid, ContentError::Internal("timestamp_format")),
            }
        }
        Err(e) => fail(&state, cid, e),
    }
}

/// Membuat konten baru berstatus Draf (US-1101 AC1).
#[utoipa::path(
    post,
    path = "/backoffice/v1/contents",
    tag = "cms",
    operation_id = "cms_content_create",
    security(("bearer" = [])),
    request_body(content = ContentRequest, content_type = "application/json"),
    responses(
        (status = 201, description = "Draf dibuat.", body = ContentEnvelope, content_type = "application/json"),
        (status = 400, description = "Badan permintaan tidak dapat dibaca.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 401, description = "Belum login, token tidak sah, atau akun nonaktif.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 403, description = "Akun tanpa sub-peran Konten.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 415, description = "Badan bukan application/json.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 422, description = "Isian tidak valid (galat per isian).", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 503, description = "Layanan tidak tersedia.", body = ErrorEnvelope, content_type = "application/problem+json")
    )
)]
async fn create(
    State(state): State<AppState>,
    Extension(cid): Extension<CorrelationId>,
    principal: Principal,
    body: Result<Json<ContentRequest>, JsonRejection>,
) -> Response {
    let Json(req) = match body {
        Ok(b) => b,
        Err(e) => return fail(&state, cid, body_error(&e)),
    };
    let r = service::create(state.pool(), state.clock(), &principal, req.into()).await;
    respond(&state, cid, r, true)
}

/// Detail dan pratinjau konten yang tidak terhapus, apa pun statusnya (US-1101 AC1).
#[utoipa::path(
    get,
    path = "/backoffice/v1/contents/{content_id}",
    tag = "cms",
    operation_id = "cms_content_get",
    security(("bearer" = [])),
    params(("content_id" = String, Path, description = "ID konten (UUID)")),
    responses(
        (status = 200, description = "Konten.", body = ContentEnvelope, content_type = "application/json"),
        (status = 401, description = "Belum login, token tidak sah, atau akun nonaktif.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 403, description = "Akun tanpa sub-peran Konten.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 404, description = "Konten tidak ada atau terhapus.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 503, description = "Layanan tidak tersedia.", body = ErrorEnvelope, content_type = "application/problem+json")
    )
)]
async fn get_one(
    State(state): State<AppState>,
    Extension(cid): Extension<CorrelationId>,
    _principal: Principal,
    Path(raw): Path<String>,
) -> Response {
    let r = match content_id(&raw) {
        Ok(id) => service::get(state.pool(), state.clock(), id).await,
        Err(e) => Err(e),
    };
    respond(&state, cid, r, false)
}

/// Mengubah draf. Konten terjadwal atau diarsipkan tidak dapat diubah (409); buat versi baru.
#[utoipa::path(
    put,
    path = "/backoffice/v1/contents/{content_id}",
    tag = "cms",
    operation_id = "cms_content_update",
    security(("bearer" = [])),
    params(("content_id" = String, Path, description = "ID konten (UUID)")),
    request_body(content = ContentRequest, content_type = "application/json"),
    responses(
        (status = 200, description = "Draf diperbarui.", body = ContentEnvelope, content_type = "application/json"),
        (status = 400, description = "Badan permintaan tidak dapat dibaca.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 401, description = "Belum login, token tidak sah, atau akun nonaktif.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 403, description = "Akun tanpa sub-peran Konten.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 404, description = "Konten tidak ada atau terhapus.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 409, description = "Konten bukan draf.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 415, description = "Badan bukan application/json.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 422, description = "Isian tidak valid (galat per isian).", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 503, description = "Layanan tidak tersedia.", body = ErrorEnvelope, content_type = "application/problem+json")
    )
)]
async fn update(
    State(state): State<AppState>,
    Extension(cid): Extension<CorrelationId>,
    principal: Principal,
    Path(raw): Path<String>,
    body: Result<Json<ContentRequest>, JsonRejection>,
) -> Response {
    let id = match content_id(&raw) {
        Ok(id) => id,
        Err(e) => return fail(&state, cid, e),
    };
    let Json(req) = match body {
        Ok(b) => b,
        Err(e) => return fail(&state, cid, body_error(&e)),
    };
    let r = service::update(state.pool(), state.clock(), &principal, id, req.into()).await;
    respond(&state, cid, r, false)
}

/// Menjadwalkan draf untuk tayang (US-1101 AC2). Halaman tetap situs: versi lama berakhir saat versi baru
/// mulai (US-1802 AC2); gambar tanpa teks alternatif ditolak (US-1802 AC5).
#[utoipa::path(
    post,
    path = "/backoffice/v1/contents/{content_id}/schedule",
    tag = "cms",
    operation_id = "cms_content_schedule",
    security(("bearer" = [])),
    params(("content_id" = String, Path, description = "ID konten (UUID)")),
    request_body(content = ScheduleRequest, content_type = "application/json"),
    responses(
        (status = 200, description = "Konten dijadwalkan.", body = ContentEnvelope, content_type = "application/json"),
        (status = 400, description = "Badan permintaan tidak dapat dibaca.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 401, description = "Belum login, token tidak sah, atau akun nonaktif.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 403, description = "Akun tanpa sub-peran Konten.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 404, description = "Konten tidak ada atau terhapus.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 409, description = "Konten bukan draf, atau versi halaman tetap lain bentrok.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 415, description = "Badan bukan application/json.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 422, description = "Jadwal atau teks alternatif tidak valid.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 503, description = "Layanan tidak tersedia.", body = ErrorEnvelope, content_type = "application/problem+json")
    )
)]
async fn schedule(
    State(state): State<AppState>,
    Extension(cid): Extension<CorrelationId>,
    principal: Principal,
    Path(raw): Path<String>,
    body: Result<Json<ScheduleRequest>, JsonRejection>,
) -> Response {
    let id = match content_id(&raw) {
        Ok(id) => id,
        Err(e) => return fail(&state, cid, e),
    };
    let Json(req) = match body {
        Ok(b) => b,
        Err(e) => return fail(&state, cid, body_error(&e)),
    };
    let start = instant("publish_start_at", req.publish_start_at.as_deref());
    let end = instant("publish_end_at", req.publish_end_at.as_deref());
    let (start, end) = match (start, end) {
        (Ok(s), Ok(e)) => (s, e),
        (s, e) => {
            let errors = [s.err(), e.err()].into_iter().flatten().collect();
            return fail(&state, cid, ContentError::Validation(errors));
        }
    };
    let r = service::schedule(state.pool(), state.clock(), &principal, id, start, end).await;
    respond(&state, cid, r, false)
}

/// Mengarsipkan draf atau konten terjadwal; konten tidak tampil lagi (US-1101 AC2).
#[utoipa::path(
    post,
    path = "/backoffice/v1/contents/{content_id}/archive",
    tag = "cms",
    operation_id = "cms_content_archive",
    security(("bearer" = [])),
    params(("content_id" = String, Path, description = "ID konten (UUID)")),
    responses(
        (status = 200, description = "Konten diarsipkan.", body = ContentEnvelope, content_type = "application/json"),
        (status = 401, description = "Belum login, token tidak sah, atau akun nonaktif.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 403, description = "Akun tanpa sub-peran Konten.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 404, description = "Konten tidak ada atau terhapus.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 409, description = "Konten sudah diarsipkan.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 503, description = "Layanan tidak tersedia.", body = ErrorEnvelope, content_type = "application/problem+json")
    )
)]
async fn archive(
    State(state): State<AppState>,
    Extension(cid): Extension<CorrelationId>,
    principal: Principal,
    Path(raw): Path<String>,
) -> Response {
    let r = match content_id(&raw) {
        Ok(id) => service::archive(state.pool(), state.clock(), &principal, id).await,
        Err(e) => Err(e),
    };
    respond(&state, cid, r, false)
}

/// Soft delete dengan alasan wajib; dapat dipulihkan (US-1101 AC3, ADR-003). Dicatat E-16.
#[utoipa::path(
    post,
    path = "/backoffice/v1/contents/{content_id}/delete",
    tag = "cms",
    operation_id = "cms_content_delete",
    security(("bearer" = [])),
    params(("content_id" = String, Path, description = "ID konten (UUID)")),
    request_body(content = DeleteRequest, content_type = "application/json"),
    responses(
        (status = 200, description = "Konten dihapus (soft delete).", body = DeletedEnvelope, content_type = "application/json"),
        (status = 400, description = "Badan permintaan tidak dapat dibaca.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 401, description = "Belum login, token tidak sah, atau akun nonaktif.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 403, description = "Akun tanpa sub-peran Konten.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 404, description = "Konten tidak ada atau sudah terhapus.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 415, description = "Badan bukan application/json.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 422, description = "Alasan wajib diisi.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 503, description = "Layanan tidak tersedia.", body = ErrorEnvelope, content_type = "application/problem+json")
    )
)]
async fn delete(
    State(state): State<AppState>,
    Extension(cid): Extension<CorrelationId>,
    addr: ClientAddr,
    headers: HeaderMap,
    principal: Principal,
    Path(raw): Path<String>,
    body: Result<Json<DeleteRequest>, JsonRejection>,
) -> Response {
    let id = match content_id(&raw) {
        Ok(id) => id,
        Err(e) => return fail(&state, cid, e),
    };
    let Json(req) = match body {
        Ok(b) => b,
        Err(e) => return fail(&state, cid, body_error(&e)),
    };
    let ctx = audit_context(cid, addr, &headers);
    match service::delete(
        state.pool(),
        state.clock(),
        &principal,
        ctx,
        id,
        &req.delete_reason,
    )
    .await
    {
        Ok(()) => ApiSuccess(
            Success::ok(DeletedDto {
                content_id: id.hyphenated().to_string(),
                display_status: DisplayStatus::Deleted,
            })
            .with_correlation_id(cid),
        )
        .into_response(),
        Err(e) => fail(&state, cid, e),
    }
}

/// Memulihkan konten terhapus (US-1101 AC3, ADR-003). Dicatat E-16.
#[utoipa::path(
    post,
    path = "/backoffice/v1/contents/{content_id}/restore",
    tag = "cms",
    operation_id = "cms_content_restore",
    security(("bearer" = [])),
    params(("content_id" = String, Path, description = "ID konten (UUID)")),
    responses(
        (status = 200, description = "Konten dipulihkan.", body = ContentEnvelope, content_type = "application/json"),
        (status = 401, description = "Belum login, token tidak sah, atau akun nonaktif.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 403, description = "Akun tanpa sub-peran Konten.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 404, description = "Konten tidak ada atau tidak terhapus.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 409, description = "Halaman tetap lain sedang terjadwal tanpa batas akhir.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 503, description = "Layanan tidak tersedia.", body = ErrorEnvelope, content_type = "application/problem+json")
    )
)]
async fn restore(
    State(state): State<AppState>,
    Extension(cid): Extension<CorrelationId>,
    addr: ClientAddr,
    headers: HeaderMap,
    principal: Principal,
    Path(raw): Path<String>,
) -> Response {
    let r = match content_id(&raw) {
        Ok(id) => {
            let ctx = audit_context(cid, addr, &headers);
            service::restore(state.pool(), state.clock(), &principal, ctx, id).await
        }
        Err(e) => Err(e),
    };
    respond(&state, cid, r, false)
}

// ---------------------------------------------------------------- jalur publik (F1-T94)
//
// Tanpa login (ADR-012 butir 3, `PublicReason::PublicContent`), hanya konten Tayang kanal situs, batas laju
// per sumber dan global sebelum database disentuh (ADR-027 butir 5, M-47), `Cache-Control: no-store`
// (cache Caddy belum diputuskan, ADR-027 butir 5; tanpa cache basi setelah konten diturunkan, SIT-02),
// tanpa cookie (US-1801 AC5).

pub(super) fn public_router() -> PolicyRouter<AppState> {
    PolicyRouter::new()
        .routes(policy::public_content(), routes!(public_page))
        .routes(policy::public_content(), routes!(public_items))
        .routes(policy::public_content(), routes!(public_item))
        .routes(policy::public_content(), routes!(public_image))
}

fn no_store(r: Response) -> Response {
    let mut r = r;
    r.headers_mut()
        .insert(CACHE_CONTROL, HeaderValue::from_static("no-store"));
    r
}

/// Batas laju jalur publik: penolakan pertama per jendela dicatat di security log (tanpa data pribadi).
fn public_limit(
    state: &AppState,
    addr: ClientAddr,
    route: &'static str,
) -> Result<(), ContentError> {
    match state.public_content_limit().check(addr.0) {
        Verdict::Allowed => Ok(()),
        Verdict::Limited { first } => {
            if first {
                security::rate_limited(None, route);
            }
            Err(ContentError::RateLimited)
        }
    }
}

fn public_respond(
    state: &AppState,
    cid: CorrelationId,
    result: Result<PublicContent, ContentError>,
) -> Response {
    let dto = result.and_then(|p| {
        PublicContentDto::from_public(p).map_err(|_| ContentError::Internal("timestamp_format"))
    });
    no_store(match dto {
        Ok(d) => ApiSuccess(Success::ok(d).with_correlation_id(cid)).into_response(),
        Err(e) => fail(state, cid, e),
    })
}

/// Halaman tetap situs yang sedang tayang (US-1801 AC1, AC2, AC4). Tidak ada versi tayang = 404.
#[utoipa::path(
    get,
    path = "/v1/public/content/pages/{site_page}",
    tag = "public-content",
    operation_id = "public_content_page",
    params(("site_page" = SitePage, Path, description = "Halaman situs (BR-130)")),
    responses(
        (status = 200, description = "Halaman yang sedang tayang.", body = PublicContentEnvelope, content_type = "application/json"),
        (status = 404, description = "Tidak ada versi tayang (draf, terjadwal, berakhir, diarsipkan, atau terhapus tidak dibedakan).", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 429, description = "Batas laju terlampaui.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 500, description = "Galat tak terduga.", body = ErrorEnvelope, content_type = "application/problem+json")
    )
)]
async fn public_page(
    State(state): State<AppState>,
    Extension(cid): Extension<CorrelationId>,
    addr: ClientAddr,
    Path(raw): Path<String>,
) -> Response {
    if let Err(e) = public_limit(&state, addr, "/v1/public/content/pages/{site_page}") {
        return no_store(fail(&state, cid, e));
    }
    let r = match SitePage::parse(&raw) {
        Some(page) => service::public_page(state.pool(), state.clock(), page).await,
        None => Err(ContentError::NotFound),
    };
    public_respond(&state, cid, r)
}

/// Daftar konten situs yang sedang tayang (misalnya artikel atau banner per halaman), urut terbaru.
#[utoipa::path(
    get,
    path = "/v1/public/content/items",
    tag = "public-content",
    operation_id = "public_content_list",
    params(PublicListParams),
    responses(
        (status = 200, description = "Daftar konten tayang.", body = PublicContentListEnvelope, content_type = "application/json"),
        (status = 422, description = "Saringan tidak valid.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 429, description = "Batas laju terlampaui.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 500, description = "Galat tak terduga.", body = ErrorEnvelope, content_type = "application/problem+json")
    )
)]
async fn public_items(
    State(state): State<AppState>,
    Extension(cid): Extension<CorrelationId>,
    addr: ClientAddr,
    params: Result<Query<PublicListParams>, QueryRejection>,
) -> Response {
    if let Err(e) = public_limit(&state, addr, "/v1/public/content/items") {
        return no_store(fail(&state, cid, e));
    }
    let Query(p) = match params {
        Ok(p) => p,
        Err(e) => return no_store(fail(&state, cid, query_error(&e))),
    };
    let cursor = match p.cursor.as_deref().map(Uuid::parse_str).transpose() {
        Ok(c) => c,
        Err(_) => {
            return no_store(fail(
                &state,
                cid,
                ContentError::Validation(vec![FieldError::new(
                    "cursor",
                    "invalid_format",
                    "Cursor tidak valid",
                )]),
            ));
        }
    };
    let q = PublicQuery {
        site_page: p.site_page,
        content_type: p.content_type,
        cursor,
        limit: p.limit,
    };
    let r = service::public_items(state.pool(), state.clock(), q).await;
    no_store(match r {
        Ok((items, meta)) => {
            match items
                .into_iter()
                .map(PublicContentDto::from_public)
                .collect::<Result<Vec<_>, _>>()
            {
                Ok(d) => ApiSuccess(Success::ok(d).with_meta(meta).with_correlation_id(cid))
                    .into_response(),
                Err(_) => fail(&state, cid, ContentError::Internal("timestamp_format")),
            }
        }
        Err(e) => fail(&state, cid, e),
    })
}

/// Satu konten situs yang sedang tayang (misalnya artikel). Selain tayang = 404 (US-1801 AC3).
#[utoipa::path(
    get,
    path = "/v1/public/content/items/{content_id}",
    tag = "public-content",
    operation_id = "public_content_get",
    params(("content_id" = String, Path, description = "ID konten (UUID)")),
    responses(
        (status = 200, description = "Konten yang sedang tayang.", body = PublicContentEnvelope, content_type = "application/json"),
        (status = 404, description = "Tidak ada atau tidak sedang tayang (tidak dibedakan).", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 429, description = "Batas laju terlampaui.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 500, description = "Galat tak terduga.", body = ErrorEnvelope, content_type = "application/problem+json")
    )
)]
async fn public_item(
    State(state): State<AppState>,
    Extension(cid): Extension<CorrelationId>,
    addr: ClientAddr,
    Path(raw): Path<String>,
) -> Response {
    if let Err(e) = public_limit(&state, addr, "/v1/public/content/items/{content_id}") {
        return no_store(fail(&state, cid, e));
    }
    let r = match content_id(&raw) {
        Ok(id) => service::public_item(state.pool(), state.clock(), id).await,
        Err(e) => Err(e),
    };
    public_respond(&state, cid, r)
}

// ---------------------------------------------------------------- gambar konten (F1-T93)

/// Slot unggah gambar konten: URL bertanda tangan berumur pendek ke object storage (ADR-018). Berkas baru
/// dapat dipakai setelah dipasang ke draf (validasi isi JPG/PNG, ukuran, metadata dibuang).
#[utoipa::path(
    post,
    path = "/backoffice/v1/content-images/upload-slots",
    tag = "cms",
    operation_id = "cms_content_image_upload_slot",
    security(("bearer" = [])),
    request_body(content = ImageSlotRequest, content_type = "application/json"),
    responses(
        (status = 201, description = "Slot unggah diterbitkan.", body = UploadSlotEnvelope, content_type = "application/json"),
        (status = 400, description = "Badan permintaan tidak dapat dibaca.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 401, description = "Belum login, token tidak sah, atau akun nonaktif.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 403, description = "Akun tanpa sub-peran Konten.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 413, description = "Ukuran melebihi 5 MiB.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 415, description = "Jenis bukan image/jpeg atau image/png, atau badan bukan JSON.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 422, description = "Isian tidak valid.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 503, description = "Layanan tidak tersedia.", body = ErrorEnvelope, content_type = "application/problem+json")
    )
)]
async fn image_upload_slot(
    State(state): State<AppState>,
    Extension(cid): Extension<CorrelationId>,
    principal: Principal,
    body: Result<Json<ImageSlotRequest>, JsonRejection>,
) -> Response {
    let Json(req) = match body {
        Ok(b) => b,
        Err(e) => return fail(&state, cid, body_error(&e)),
    };
    let r = service::create_image_slot(
        state.pool(),
        state.clock(),
        state.files(),
        &principal,
        req.content_type,
        req.size_bytes,
    )
    .await
    .and_then(|t| {
        UploadSlotDto::from_ticket(t).map_err(|_| ContentError::Internal("timestamp_format"))
    });
    match r {
        Ok(d) => ApiSuccess(Success::created(d).with_correlation_id(cid)).into_response(),
        Err(e) => fail(&state, cid, e),
    }
}

/// Memasang gambar yang sudah diunggah ke draf, dengan teks alternatif wajib (US-1802 AC5).
#[utoipa::path(
    put,
    path = "/backoffice/v1/contents/{content_id}/image",
    tag = "cms",
    operation_id = "cms_content_image_attach",
    security(("bearer" = [])),
    params(("content_id" = String, Path, description = "ID konten (UUID)")),
    request_body(content = AttachImageRequest, content_type = "application/json"),
    responses(
        (status = 200, description = "Gambar dipasang.", body = ContentEnvelope, content_type = "application/json"),
        (status = 400, description = "Badan permintaan tidak dapat dibaca.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 401, description = "Belum login, token tidak sah, atau akun nonaktif.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 403, description = "Akun tanpa sub-peran Konten.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 404, description = "Konten atau slot tidak ada, kedaluwarsa, atau milik akun lain.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 409, description = "Konten bukan draf.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 413, description = "Berkas melebihi 5 MiB.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 415, description = "Isi berkas bukan JPG/PNG yang sah, atau badan bukan JSON.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 422, description = "Teks alternatif kosong atau berkas belum diunggah.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 503, description = "Layanan tidak tersedia.", body = ErrorEnvelope, content_type = "application/problem+json")
    )
)]
async fn attach_image(
    State(state): State<AppState>,
    Extension(cid): Extension<CorrelationId>,
    principal: Principal,
    Path(raw): Path<String>,
    body: Result<Json<AttachImageRequest>, JsonRejection>,
) -> Response {
    let id = match content_id(&raw) {
        Ok(id) => id,
        Err(e) => return fail(&state, cid, e),
    };
    let Json(req) = match body {
        Ok(b) => b,
        Err(e) => return fail(&state, cid, body_error(&e)),
    };
    let slot = match Uuid::parse_str(&req.upload_slot_id) {
        Ok(s) => s,
        Err(_) => return fail(&state, cid, ContentError::NotFound),
    };
    let r = service::attach_image(
        state.pool(),
        state.clock(),
        state.files(),
        &principal,
        id,
        slot,
        &req.image_alt_text,
    )
    .await;
    respond(&state, cid, r, false)
}

/// Gambar konten yang sedang tayang di situs, disajikan backend dari object storage privat (bukan bucket
/// publik, ADR-027 butir 5), dengan batas laju. Selain itu 404 (tidak membedakan draf dan ID tebakan).
#[utoipa::path(
    get,
    path = "/v1/public/content/images/{image_id}",
    tag = "public-content",
    operation_id = "public_content_image",
    params(("image_id" = String, Path, description = "ID gambar (UUID) dari `image.url` konten tayang")),
    responses(
        (status = 200, description = "Isi gambar (JPG/PNG hasil kode ulang tanpa metadata).", content(("image/png"), ("image/jpeg"))),
        (status = 404, description = "Tidak ada atau tidak dipakai konten yang sedang tayang.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 429, description = "Batas laju terlampaui.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 500, description = "Galat tak terduga.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 503, description = "Object storage tidak tersedia.", body = ErrorEnvelope, content_type = "application/problem+json")
    )
)]
async fn public_image(
    State(state): State<AppState>,
    Extension(cid): Extension<CorrelationId>,
    addr: ClientAddr,
    Path(raw): Path<String>,
) -> Response {
    let limited = match state.public_image_limit().check(addr.0) {
        Verdict::Allowed => None,
        Verdict::Limited { first } => {
            if first {
                security::rate_limited(None, "/v1/public/content/images/{image_id}");
            }
            Some(ContentError::RateLimited)
        }
    };
    if let Some(e) = limited {
        return no_store(fail(&state, cid, e));
    }
    let r = match content_id(&raw) {
        Ok(id) => service::public_image(state.pool(), state.clock(), state.files(), id).await,
        Err(e) => Err(e),
    };
    match r {
        Ok(file) => {
            let mut resp = (
                [
                    (CONTENT_TYPE, HeaderValue::from_static(file.mime_type)),
                    (CACHE_CONTROL, HeaderValue::from_static("no-store")),
                    (X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff")),
                    (
                        CONTENT_SECURITY_POLICY,
                        HeaderValue::from_static("default-src 'none'; sandbox"),
                    ),
                ],
                file.bytes,
            )
                .into_response();
            *resp.status_mut() = axum::http::StatusCode::OK;
            resp
        }
        Err(e) => no_store(fail(&state, cid, e)),
    }
}
