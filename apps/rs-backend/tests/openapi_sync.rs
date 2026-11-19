//! Spesifikasi di `openapi/` harus identik dengan hasil generate (ADR-022 butir 3, KEPUTUSAN-82).
//! Beda berarti test gagal. Perbarui dengan: `cargo run -p rs-backend --bin gen-openapi -- openapi`.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)] // berkas test

mod common;

use std::collections::BTreeSet;
use std::fs;

const FIX: &str = "perbarui dengan: cargo run -p rs-backend --bin gen-openapi -- openapi";

/// Nama berkas spesifikasi HTTP (json/yaml/yml) langsung di `openapi/`, tidak termasuk `asyncapi/`.
fn committed_names() -> BTreeSet<String> {
    let dir = common::workspace_root().join("openapi");
    fs::read_dir(dir)
        .unwrap()
        .flatten()
        .filter(|e| e.path().is_file())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| [".json", ".yaml", ".yml"].iter().any(|x| n.ends_with(x)))
        .collect()
}

#[test]
fn committed_specs_equal_generated() {
    let generated = rs_backend::openapi::generate().unwrap();
    assert!(!generated.is_empty());
    let dir = common::workspace_root().join("openapi");
    for (name, json) in &generated {
        let committed = fs::read_to_string(dir.join(name))
            .unwrap_or_else(|_| panic!("openapi/{name} belum ada; {FIX}"));
        assert!(
            &committed == json,
            "openapi/{name} berbeda dari hasil generate; {FIX}"
        );
    }
}

#[test]
fn no_committed_spec_without_generator() {
    let generated: BTreeSet<String> = rs_backend::openapi::generate()
        .unwrap()
        .into_iter()
        .map(|(n, _)| n.to_string())
        .collect();
    let extra: Vec<_> = committed_names().difference(&generated).cloned().collect();
    assert!(
        extra.is_empty(),
        "berkas di openapi/ tanpa pembangkit (hapus atau tambahkan pembangkitnya): {extra:?}"
    );
}

#[test]
fn generation_is_deterministic() {
    assert_eq!(
        rs_backend::openapi::generate().unwrap(),
        rs_backend::openapi::generate().unwrap()
    );
}

#[test]
fn every_route_in_router_is_in_spec_and_vice_versa() {
    // Spesifikasi dan rute berasal dari satu anotasi (utoipa-axum). Test ini membuktikannya lewat
    // HTTP: setiap path di spesifikasi menjawab (bukan 404 fallback) dan path lain menjawab 404.
    let (_, json) = rs_backend::openapi::generate().unwrap().remove(0);
    let spec: serde_json::Value = serde_json::from_str(&json).unwrap();
    let paths: Vec<String> = spec["paths"].as_object().unwrap().keys().cloned().collect();
    assert!(paths.contains(&"/health/live".to_string()));
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async {
        let addr = common::spawn(vec![]).await;
        for p in &paths {
            let r = common::request(addr, "GET", p, &[]).await;
            assert_eq!(
                r.status, 200,
                "{p} ada di spesifikasi tetapi tidak dilayani"
            );
        }
        let r = common::request(addr, "GET", "/health/tidak-ada", &[]).await;
        assert_eq!(r.status, 404);
    });
}
