//! Setiap `response_code` di spesifikasi `openapi/` terdaftar di `docs/05-response-codes.md`, deskripsinya
//! sama, dan tiga digit pertamanya sama dengan kode HTTP respons tempatnya berada
//! (docs/05-response-codes.md §5 butir 6, KEPUTUSAN-82). Diurai dengan `envelope::registry::parse`.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)] // berkas test

mod common;

use envelope::registry::{RegistryEntry, parse};
use serde_json::{Value, json};
use std::fs;

fn registry() -> Vec<RegistryEntry> {
    let path = common::workspace_root().join("docs/05-response-codes.md");
    parse(&fs::read_to_string(path).unwrap()).unwrap()
}

/// Mengumpulkan semua kode di `v`: nilai string pada kunci `response_code`, serta `example`/`enum`
/// pada definisi skema properti `response_code`. Sekaligus mengumpulkan (kode, deskripsi) dari
/// objek contoh yang memuat keduanya.
fn collect(v: &Value, codes: &mut Vec<String>, pairs: &mut Vec<(String, String)>) {
    match v {
        Value::Object(m) => {
            if let (Some(Value::String(c)), Some(Value::String(d))) =
                (m.get("response_code"), m.get("response_description"))
            {
                pairs.push((c.clone(), d.clone()));
            }
            for (k, child) in m {
                if k == "response_code" {
                    match child {
                        Value::String(s) => codes.push(s.clone()),
                        Value::Object(def) => {
                            if let Some(Value::String(s)) = def.get("example") {
                                codes.push(s.clone());
                            }
                            if let Some(Value::Array(a)) = def.get("enum") {
                                codes.extend(a.iter().filter_map(|x| x.as_str()).map(String::from));
                            }
                        }
                        _ => {}
                    }
                }
                collect(child, codes, pairs);
            }
        }
        Value::Array(a) => a.iter().for_each(|c| collect(c, codes, pairs)),
        _ => {}
    }
}

/// Memeriksa satu dokumen spesifikasi terhadap registry; mengembalikan daftar pelanggaran.
fn violations(spec: &Value, reg: &[RegistryEntry]) -> Vec<String> {
    let mut out = Vec::new();
    let (mut codes, mut pairs) = (Vec::new(), Vec::new());
    collect(spec, &mut codes, &mut pairs);
    for c in &codes {
        if !reg.iter().any(|e| &e.code == c) {
            out.push(format!("kode {c} tidak terdaftar di registry"));
        }
    }
    for (c, d) in &pairs {
        if let Some(e) = reg.iter().find(|e| &e.code == c)
            && &e.description != d
        {
            out.push(format!("deskripsi kode {c} berbeda dari registry: {d:?}"));
        }
    }
    // Tiga digit pertama kode = kunci status HTTP respons tempatnya berada.
    if let Some(paths) = spec.get("paths").and_then(Value::as_object) {
        for (path, item) in paths {
            for (method, op) in item.as_object().into_iter().flatten() {
                for (status, resp) in op["responses"].as_object().into_iter().flatten() {
                    let (mut cs, mut ps) = (Vec::new(), Vec::new());
                    collect(resp, &mut cs, &mut ps);
                    for c in cs {
                        if !c.starts_with(status.as_str()) {
                            out.push(format!(
                                "{method} {path}: kode {c} di respons HTTP {status}"
                            ));
                        }
                    }
                }
            }
        }
    }
    out
}

#[test]
fn all_generated_and_committed_specs_use_registered_codes() {
    let reg = registry();
    assert!(!reg.is_empty());
    let mut total = 0;
    for (name, json) in rs_backend::openapi::generate().unwrap() {
        let spec: Value = serde_json::from_str(&json).unwrap();
        let (mut codes, mut pairs) = (Vec::new(), Vec::new());
        collect(&spec, &mut codes, &mut pairs);
        total += codes.len();
        let v = violations(&spec, &reg);
        assert!(v.is_empty(), "{name}: {v:?}");
        // Berkas yang di-commit juga diperiksa langsung (bukan hanya hasil generate).
        let committed =
            fs::read_to_string(common::workspace_root().join("openapi").join(name)).unwrap();
        let spec: Value = serde_json::from_str(&committed).unwrap();
        let v = violations(&spec, &reg);
        assert!(v.is_empty(), "openapi/{name}: {v:?}");
    }
    assert!(
        total > 0,
        "tidak ada response_code yang diperiksa (test kosong)"
    );
}

#[test]
fn checker_detects_unregistered_code() {
    let spec = json!({"paths": {"/x": {"get": {"responses": {"200": {"content": {"application/json": {
        "example": {"response_code": "2009999", "response_description": "Berhasil"}}}}}}}}});
    let v = violations(&spec, &registry());
    assert!(v.iter().any(|m| m.contains("2009999")), "{v:?}");
}

#[test]
fn checker_detects_http_mismatch() {
    let spec = json!({"paths": {"/x": {"get": {"responses": {"200": {"content": {"application/json": {
        "example": {"response_code": "4040001", "response_description": "Data tidak ditemukan"}}}}}}}}});
    let v = violations(&spec, &registry());
    assert!(v.iter().any(|m| m.contains("HTTP 200")), "{v:?}");
}

#[test]
fn checker_detects_description_drift_and_schema_example() {
    let spec = json!({"components": {"schemas": {"E": {"properties": {
        "response_code": {"type": "string", "example": "1230000", "enum": ["2000000", "7770000"]}}}}},
        "x": {"response_code": "2000000", "response_description": "Salah"}});
    let v = violations(&spec, &registry());
    assert!(v.iter().any(|m| m.contains("1230000")), "{v:?}");
    assert!(v.iter().any(|m| m.contains("7770000")), "{v:?}");
    assert!(v.iter().any(|m| m.contains("deskripsi")), "{v:?}");
}
