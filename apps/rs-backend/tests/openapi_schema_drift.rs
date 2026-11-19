//! Skema dokumentasi envelope (`HealthEnvelope`, `ErrorEnvelope`) harus sama dengan isi respons
//! sebenarnya yang dibangun `envelope::Success` dan `envelope::ErrorBody`.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)] // berkas test

mod common;

use serde_json::Value;
use std::collections::BTreeSet;

fn schema(name: &str) -> Value {
    let (_, json) = rs_backend::openapi::generate().unwrap().remove(0);
    let spec: Value = serde_json::from_str(&json).unwrap();
    spec["components"]["schemas"][name].clone()
}

fn keys(v: &Value) -> BTreeSet<String> {
    v.as_object().unwrap().keys().cloned().collect()
}

fn required(s: &Value) -> BTreeSet<String> {
    s["required"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_str().unwrap().to_string())
        .collect()
}

#[tokio::test(flavor = "current_thread")]
async fn success_body_matches_health_envelope_schema() {
    let s = schema("HealthEnvelope");
    let addr = common::spawn(vec![]).await;
    let body = common::request(addr, "GET", "/health/live", &[])
        .await
        .json();
    assert_eq!(keys(&body), keys(&s["properties"]));
    assert!(required(&s).is_subset(&keys(&body)));
}

#[tokio::test(flavor = "current_thread")]
async fn error_body_matches_error_envelope_schema() {
    let s = schema("ErrorEnvelope");
    let addr = common::spawn(vec![]).await;
    let body = common::request(addr, "GET", "/tidak-ada", &[]).await.json();
    // Isi nyata tanpa `errors` dan `instance` (opsional); semua kunci nyata ada di skema,
    // dan semua kunci wajib di skema ada di isi nyata.
    assert!(keys(&body).is_subset(&keys(&s["properties"])));
    assert_eq!(required(&s), keys(&body));
    let optional: BTreeSet<_> = keys(&s["properties"])
        .difference(&required(&s))
        .cloned()
        .collect();
    assert_eq!(
        optional,
        ["errors", "instance"]
            .iter()
            .map(|x| x.to_string())
            .collect()
    );
}
