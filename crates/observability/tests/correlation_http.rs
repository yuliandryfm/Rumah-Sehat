//! Test middleware korelasi lewat HTTP sungguhan di localhost (KEPUTUSAN-123).

#![allow(clippy::unwrap_used, clippy::expect_used)] // berkas test

use axum::Router;
use axum::routing::get;
use foundation::{FixedClock, new_id};
use observability::LogLevel;
use observability::correlation_middleware;
use observability::log::{Component, JsonLogLayer, LogSink, MemorySink, subscriber};
use std::sync::Arc;
use time::OffsetDateTime;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

async fn handler() -> &'static str {
    tracing::info!("di handler");
    "ok"
}

/// Mengirim permintaan mentah dan mengembalikan seluruh respons.
async fn request(addr: std::net::SocketAddr, path: &str, headers: &[String]) -> String {
    let mut s = TcpStream::connect(addr).await.unwrap();
    let mut req = format!("GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n");
    for h in headers {
        req.push_str(h);
        req.push_str("\r\n");
    }
    req.push_str("\r\n");
    s.write_all(req.as_bytes()).await.unwrap();
    let mut buf = Vec::new();
    s.read_to_end(&mut buf).await.unwrap();
    String::from_utf8_lossy(&buf).into_owned()
}

fn response_correlation(resp: &str) -> String {
    resp.lines()
        .find_map(|l| {
            let (k, v) = l.split_once(':')?;
            k.eq_ignore_ascii_case("x-correlation-id")
                .then(|| v.trim().to_string())
        })
        .unwrap_or_default()
}

#[tokio::test(flavor = "current_thread")]
async fn correlation_header_rules_and_request_log() {
    let sink = MemorySink::default();
    let layer = JsonLogLayer::new(
        Component::Api,
        LogLevel::Info,
        Arc::new(FixedClock::new(OffsetDateTime::UNIX_EPOCH)),
    )
    .with_sinks(
        Arc::new(sink.clone()) as Arc<dyn LogSink>,
        Arc::new(MemorySink::default()),
    );
    let _guard = tracing::subscriber::set_default(subscriber(layer));

    let app = Router::new()
        .route("/orders/{order_id}", get(handler))
        .layer(axum::middleware::from_fn(correlation_middleware));
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // 1. UUID valid dipertahankan dan muncul di log handler dan log selesai.
    let id = new_id().hyphenated().to_string();
    let resp = request(
        addr,
        "/orders/9876543210987654",
        &[format!("X-Correlation-Id: {id}")],
    )
    .await;
    assert!(resp.starts_with("HTTP/1.1 200"), "{resp}");
    assert_eq!(response_correlation(&resp), id);

    // 2. Tanpa header: dibuat baru, UUIDv7 valid.
    let resp = request(addr, "/orders/1", &[]).await;
    let fresh = response_correlation(&resp);
    assert_eq!(fresh.len(), 36, "{resp}");
    assert_eq!(
        foundation::CorrelationId::parse(&fresh)
            .unwrap()
            .as_uuid()
            .get_version_num(),
        7
    );

    // 3. Nilai tidak valid / berisi teks penyisip: diganti, tidak pernah masuk log.
    let resp = request(
        addr,
        "/orders/2",
        &["X-Correlation-Id: forged-value level=ERROR".to_string()],
    )
    .await;
    let replaced = response_correlation(&resp);
    assert_eq!(replaced.len(), 36);
    assert!(!replaced.contains("forged"));

    // 4. Dua header berbeda: ditolak, ID baru.
    let other = new_id().hyphenated().to_string();
    let resp = request(
        addr,
        "/orders/3",
        &[
            format!("X-Correlation-Id: {id}"),
            format!("X-Correlation-Id: {other}"),
        ],
    )
    .await;
    let dup = response_correlation(&resp);
    assert!(dup != id && dup != other, "{resp}");

    let log = sink.text();
    assert!(!log.contains("forged"), "{log}");
    assert!(
        !log.contains("9876543210987654"),
        "ID di URL tidak boleh masuk log: {log}"
    );
    assert!(log.contains("\"route\":\"/orders/{order_id}\""), "{log}");
    assert!(log.contains("\"message\":\"request_completed\""), "{log}");
    assert!(log.contains("\"status\":200"), "{log}");
    assert!(log.contains("duration_ms"), "{log}");
    // Log handler membawa ID korelasi yang sama dengan permintaan 1.
    let handler_line = sink
        .lines()
        .into_iter()
        .find(|l| l.contains("di handler") && l.contains(&id))
        .unwrap_or_default();
    assert!(
        handler_line.contains(&format!("\"correlation_id\":\"{id}\"")),
        "{log}"
    );
}
