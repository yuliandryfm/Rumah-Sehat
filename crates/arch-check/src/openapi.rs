//! Kontrak API: tidak ada operasi ubah/hapus audit log di spesifikasi OpenAPI mana pun (US-1504 AC1,
//! ADR-009, PRD §7.5 "Semua peran: tidak ada yang dapat mengubah atau menghapus"; F1-T68, TC-US-1504-1).
//!
//! Semua berkas `*.json` di bawah `openapi/` (rekursif) yang punya objek `paths` diperiksa: path yang
//! merupakan jalur audit tidak boleh punya operasi `put`, `patch`, atau `delete`. Spesifikasi `openapi/`
//! sendiri dijaga identik dengan hasil generate kode (langkah sinkron OpenAPI di skrip verifikasi), sehingga
//! rute di kode ikut tertangkap. Berkas JSON yang tidak dapat diurai, dan spesifikasi YAML di tingkat atas
//! `openapi/` (dianggap OpenAPI oleh skrip verifikasi tetapi tidak diurai di sini), dilaporkan supaya
//! pemeriksaan tidak terlewat diam-diam.
//!
//! Definisi jalur audit sama dengan `authz::is_audit_path` (crate ini tidak bergantung pada `authz`):
//! salah satu segmen path, tanpa membedakan huruf besar-kecil, sama dengan `audit` atau berawalan
//! `audit-`/`audit_`.

use crate::{Finding, fs};
use serde_json::Value;
use std::path::Path;

/// Operasi OpenAPI yang dilarang di jalur audit.
pub const FORBIDDEN_AUDIT_OPERATIONS: &[&str] = &["put", "patch", "delete"];

/// Benar bila salah satu segmen `path` menandai jalur audit.
pub fn is_audit_path(path: &str) -> bool {
    path.split('/').any(|segment| {
        let s = segment.to_ascii_lowercase();
        s == "audit" || s.starts_with("audit-") || s.starts_with("audit_")
    })
}

fn finding(file: &str, detail: &str, msg: impl Into<String>) -> Finding {
    Finding::new("openapi-audit", file, 0, detail, msg)
}

/// Memeriksa semua spesifikasi di `root/openapi`.
pub fn check_openapi(root: &Path) -> Vec<Finding> {
    let dir = root.join("openapi");
    let mut out = Vec::new();
    if let Ok(rd) = std::fs::read_dir(&dir) {
        let mut yaml: Vec<String> = rd
            .flatten()
            .map(|e| e.path())
            .filter(|p| {
                p.is_file()
                    && matches!(p.extension().and_then(|e| e.to_str()), Some("yaml" | "yml"))
            })
            .map(|p| fs::rel(root, &p))
            .collect();
        yaml.sort();
        for f in yaml {
            out.push(finding(
                &f,
                "yaml",
                "spesifikasi OpenAPI YAML tidak diperiksa larangan operasi ubah/hapus audit; gunakan JSON",
            ));
        }
    }
    for p in fs::files_with_ext(&dir, "json") {
        let file = fs::rel(root, &p);
        let parsed = std::fs::read_to_string(&p)
            .map_err(|e| format!("tidak terbaca: {e}"))
            .and_then(|t| {
                serde_json::from_str::<Value>(&t).map_err(|e| format!("JSON tidak valid: {e}"))
            });
        match parsed {
            Ok(spec) => out.extend(check_spec(&file, &spec)),
            Err(e) => out.push(finding(&file, "parse", e)),
        }
    }
    out
}

/// Memeriksa satu spesifikasi yang sudah diurai.
pub fn check_spec(file: &str, spec: &Value) -> Vec<Finding> {
    let mut out = Vec::new();
    let Some(paths) = spec.get("paths").and_then(Value::as_object) else {
        return out;
    };
    for (path, item) in paths {
        if !is_audit_path(path) {
            continue;
        }
        for op in FORBIDDEN_AUDIT_OPERATIONS {
            if item.get(*op).is_some() {
                out.push(finding(
                    file,
                    &format!("{op} {path}"),
                    "operasi ubah/hapus audit log dilarang (US-1504 AC1, ADR-009)",
                ));
            }
        }
    }
    out
}
