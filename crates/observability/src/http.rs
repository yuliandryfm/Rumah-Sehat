//! Middleware korelasi dan log permintaan (KEPUTUSAN-123, coding-standards §4.2).
//!
//! Header `X-Correlation-Id` diterima hanya bila tepat satu nilai berbentuk UUID valid; selain itu
//! dibuat UUIDv7 baru, sehingga nilai dari luar tidak dapat menyisipkan teks ke log. ID disimpan di
//! extensions permintaan ([`CorrelationId`]), dipasang sebagai field span `correlation_id` untuk
//! semua log di dalam permintaan, dan dikembalikan di header respons.
//!
//! Pemakaian: `router.layer(axum::middleware::from_fn(observability::correlation_middleware))`
//! dengan `Router::layer` (bukan di luar router) agar pola rute `MatchedPath` tersedia.

use axum::extract::{MatchedPath, Request};
use axum::http::{HeaderName, HeaderValue};
use axum::middleware::Next;
use axum::response::Response;
use foundation::CorrelationId;
use std::time::Instant;
use tracing::{Instrument, info, info_span};

/// Nama header korelasi.
pub const CORRELATION_HEADER: &str = "x-correlation-id";

/// Menentukan ID korelasi dari header masuk: tepat satu nilai UUID valid, atau ID baru.
pub fn resolve_correlation_id(headers: &axum::http::HeaderMap) -> CorrelationId {
    let mut values = headers.get_all(CORRELATION_HEADER).iter();
    match (values.next(), values.next()) {
        (Some(v), None) => v
            .to_str()
            .ok()
            .and_then(|s| CorrelationId::parse(s).ok())
            .unwrap_or_default(),
        _ => CorrelationId::new(),
    }
}

/// Middleware axum: menetapkan ID korelasi, membuka span, mencatat `request_completed`
/// (status, durasi, pola rute; tanpa URL mentah, query, header, atau body).
pub async fn correlation_middleware(mut req: Request, next: Next) -> Response {
    let id = resolve_correlation_id(req.headers());
    req.extensions_mut().insert(id);
    let route = req
        .extensions()
        .get::<MatchedPath>()
        .map(|m| m.as_str().to_string())
        .unwrap_or_else(|| "unmatched".to_string());

    let span = info_span!("request", correlation_id = %id, route = %route);
    let started = Instant::now();
    let mut response = next.run(req).instrument(span.clone()).await;

    let status = response.status().as_u16();
    let duration_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
    span.in_scope(|| {
        info!(
            status = status,
            duration_ms = duration_ms,
            "request_completed"
        )
    });

    if let Ok(value) = HeaderValue::from_str(&id.to_string()) {
        response
            .headers_mut()
            .insert(HeaderName::from_static(CORRELATION_HEADER), value);
    }
    response
}
