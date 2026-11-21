//! US-1504 AC1 (F1-T68, TC-US-1504-1): tidak ada operasi ubah/hapus audit log di OpenAPI mana pun.
//! Uji atas repositori sungguhan, uji negatif atas spesifikasi tiruan di memori, dan pemindaian berkas di
//! direktori sementara (JSON rusak, YAML tingkat atas).
#![allow(clippy::unwrap_used, clippy::expect_used)] // berkas test

use arch_check::openapi::{check_openapi, check_spec, is_audit_path};
use serde_json::json;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

static N: AtomicUsize = AtomicUsize::new(0);

struct Repo(PathBuf);

impl Repo {
    fn new() -> Self {
        let dir = std::env::temp_dir().join(format!(
            "arch-check-openapi-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::SeqCst)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        Repo(dir)
    }
    fn put(&self, rel: &str, content: &str) {
        let p = self.0.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, content).unwrap();
    }
}

impl Drop for Repo {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn details(spec: &serde_json::Value) -> Vec<String> {
    check_spec("openapi/x.json", spec)
        .into_iter()
        .map(|f| f.detail)
        .collect()
}

#[test]
fn repository_specs_have_no_audit_modification_operations() {
    let specs = std::fs::read_dir(root().join("openapi"))
        .unwrap()
        .flatten()
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("json"))
        .count();
    // Penjaga anti-vakum: spesifikasi backend Ruang Sehat ada dan ikut terperiksa.
    assert!(specs >= 4, "hanya {specs} spesifikasi JSON di openapi/");
    let v = check_openapi(&root());
    assert!(
        v.is_empty(),
        "{:?}",
        v.iter().map(ToString::to_string).collect::<Vec<_>>()
    );
}

#[test]
fn put_patch_delete_on_audit_paths_are_rejected() {
    let spec = json!({"openapi": "3.1.0", "paths": {
        "/backoffice/v1/audit-logs": {"get": {}, "delete": {}},
        "/backoffice/v1/audit-logs/{event_id}": {"get": {}, "put": {}, "patch": {}},
        "/rme/v1/Audit_Events/{id}": {"delete": {}},
        "/v1/audit": {"patch": {}}
    }});
    let mut d = details(&spec);
    d.sort();
    assert_eq!(
        d,
        [
            "delete /backoffice/v1/audit-logs",
            "delete /rme/v1/Audit_Events/{id}",
            "patch /backoffice/v1/audit-logs/{event_id}",
            "patch /v1/audit",
            "put /backoffice/v1/audit-logs/{event_id}",
        ]
    );
}

#[test]
fn read_operations_and_non_audit_paths_are_allowed() {
    let spec = json!({"openapi": "3.1.0", "paths": {
        "/backoffice/v1/audit-logs": {"get": {}},
        "/backoffice/v1/audit-logs/{event_id}": {"get": {}, "head": {}},
        "/backoffice/v1/contents/{id}": {"put": {}, "delete": {}},
        "/backoffice/v1/auditors/{id}": {"delete": {}}
    }});
    assert!(details(&spec).is_empty(), "{:?}", details(&spec));
    // Dokumen tanpa `paths` (misalnya AsyncAPI) tidak diperiksa.
    assert!(details(&json!({"asyncapi": "3.0.0", "channels": {}})).is_empty());
}

#[test]
fn audit_path_rule_matches_segments_only() {
    assert!(is_audit_path("/backoffice/v1/audit-logs/{id}"));
    assert!(is_audit_path("/x/AUDIT"));
    assert!(is_audit_path("/x/audit_events"));
    assert!(!is_audit_path("/backoffice/v1/auditors"));
    assert!(!is_audit_path("/backoffice/v1/xaudit-logs"));
    assert!(!is_audit_path("/"));
}

#[test]
fn files_are_scanned_recursively_and_unreadable_specs_are_reported() {
    let r = Repo::new();
    r.put(
        "openapi/ok.json",
        r#"{"paths": {"/backoffice/v1/audit-logs": {"get": {}}}}"#,
    );
    r.put(
        "openapi/rme/rme.json",
        r#"{"paths": {"/rme/v1/audit-events/{id}": {"delete": {}}}}"#,
    );
    r.put("openapi/rusak.json", "{\"paths\": ");
    r.put("openapi/lain.yaml", "openapi: 3.1.0\npaths: {}\n");
    // AsyncAPI YAML di subdirektori bukan spesifikasi OpenAPI: tidak dilaporkan.
    r.put("openapi/asyncapi/events.yaml", "asyncapi: 3.0.0\n");
    let mut got: Vec<String> = check_openapi(&r.0)
        .into_iter()
        .map(|f| format!("{}|{}", f.file, f.detail))
        .collect();
    got.sort();
    assert_eq!(
        got,
        [
            "openapi/lain.yaml|yaml",
            "openapi/rme/rme.json|delete /rme/v1/audit-events/{id}",
            "openapi/rusak.json|parse",
        ]
    );
}
