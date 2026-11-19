//! Endpoint kesehatan lewat HTTP sungguhan: envelope ADR-023, korelasi, galat tanpa kebocoran.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)] // berkas test

mod common;

use common::{TYPE_BASE, request, spawn};
use rs_backend::modules::platform::api::{ReadinessCheck, ReadinessFuture};
use serde_json::json;
use std::sync::Arc;

struct Check {
    name: &'static str,
    ok: bool,
}

impl ReadinessCheck for Check {
    fn name(&self) -> &'static str {
        self.name
    }
    fn check(&self) -> ReadinessFuture<'_> {
        Box::pin(async move { self.ok })
    }
}

#[tokio::test(flavor = "current_thread")]
async fn live_returns_success_envelope_with_correlation() {
    let addr = spawn(vec![]).await;
    let id = "01936f8e-9c01-7e02-8f03-1a2b3c4d5e6f";
    let r = request(
        addr,
        "GET",
        "/health/live",
        &[&format!("X-Correlation-Id: {id}")],
    )
    .await;
    assert_eq!(r.status, 200);
    assert_eq!(r.header("content-type"), Some("application/json"));
    assert_eq!(r.header("x-correlation-id"), Some(id));
    assert_eq!(
        r.json(),
        json!({
            "response_code": "2000000",
            "response_description": "Berhasil",
            "data": {"status": "ok"},
            "correlation_id": id
        })
    );
}

#[tokio::test(flavor = "current_thread")]
async fn ready_without_checks_and_with_passing_checks_is_ok() {
    let addr = spawn(vec![]).await;
    assert_eq!(request(addr, "GET", "/health/ready", &[]).await.status, 200);
    let addr = spawn(vec![Arc::new(Check {
        name: "db",
        ok: true,
    })])
    .await;
    assert_eq!(request(addr, "GET", "/health/ready", &[]).await.status, 200);
}

#[tokio::test(flavor = "current_thread")]
async fn ready_fails_with_503_problem_and_does_not_leak_check_name() {
    let addr = spawn(vec![
        Arc::new(Check {
            name: "db-rahasia-internal",
            ok: true,
        }),
        Arc::new(Check {
            name: "valkey-rahasia-internal",
            ok: false,
        }),
    ])
    .await;
    let r = request(addr, "GET", "/health/ready", &[]).await;
    assert_eq!(r.status, 503);
    assert_eq!(r.header("content-type"), Some("application/problem+json"));
    let v = r.json();
    assert_eq!(v["response_code"], "5030001");
    assert_eq!(v["type"], format!("{TYPE_BASE}/5030001"));
    assert_eq!(v["correlation_id"], r.header("x-correlation-id").unwrap());
    assert!(!r.body.contains("rahasia-internal"), "{}", r.body);
    for k in ["status", "title", "detail", "data"] {
        assert!(v.get(k).is_none(), "{k}");
    }
}

#[tokio::test(flavor = "current_thread")]
async fn unknown_route_and_wrong_method_return_404_envelope() {
    let addr = spawn(vec![]).await;
    for (method, path) in [
        ("GET", "/tidak-ada"),
        ("POST", "/health/live"),
        ("DELETE", "/health/ready"),
    ] {
        let r = request(addr, method, path, &[]).await;
        assert_eq!(r.status, 404, "{method} {path}");
        assert_eq!(r.header("content-type"), Some("application/problem+json"));
        let v = r.json();
        assert_eq!(v["response_code"], "4040001");
        assert!(v["correlation_id"].is_string());
    }
}

#[tokio::test(flavor = "current_thread")]
async fn invalid_correlation_header_is_replaced_not_echoed() {
    let addr = spawn(vec![]).await;
    let r = request(
        addr,
        "GET",
        "/health/live",
        &["X-Correlation-Id: bukan-uuid\"; injeksi"],
    )
    .await;
    assert_eq!(r.status, 200);
    assert!(!r.body.contains("injeksi"));
    assert_ne!(r.header("x-correlation-id"), Some("bukan-uuid\"; injeksi"));
}
