//! Pemeriksaan manifes dan konfigurasi clippy: larangan tidak boleh dilemahkan atau ditimpa per crate.
//!
//! Aturan `test-feature` (TD-15): fitur pembantu test (`authn/testing`, `crypto/test-keys`) tidak boleh
//! ikut build produksi. Fitur itu hanya boleh dinyalakan lewat `[dev-dependencies]`; dependensi produksi
//! (`[dependencies]`, `[build-dependencies]`, `[target.*.…]`, `[workspace.dependencies]`) dan tabel
//! `[features]` crate mana pun tidak boleh menyalakannya, dan fitur `default` crate pemiliknya tidak boleh
//! memuatnya. Satu-satunya pengecualian: dependensi opsional `crypto` (fitur `test-keys`) milik crate
//! `authn`, yang hanya dinyalakan fitur `testing` authn sendiri.

use crate::{Finding, fs};
use std::path::Path;
use toml::{Table, Value};

/// Lint larangan yang wajib `deny`/`forbid` di `[workspace.lints.clippy]` (coding-standards §3.4).
pub const REQUIRED_LINTS: &[&str] = &[
    "unwrap_used",
    "expect_used",
    "panic",
    "todo",
    "unimplemented",
    "indexing_slicing",
    "dbg_macro",
    "disallowed_methods",
];

/// Metode yang wajib ada di `disallowed-methods` pada `clippy.toml` root.
pub const REQUIRED_METHODS: &[&str] = &[
    "time::OffsetDateTime::now_utc",
    "time::OffsetDateTime::now_local",
    "time::UtcDateTime::now",
    "std::time::SystemTime::now",
    "std::env::var",
    "std::env::var_os",
    "std::env::vars",
    "std::env::vars_os",
];

/// Metode yang boleh memakai `allow-invalid` (berlaku hanya bila fitur crate-nya dinyalakan).
const ALLOW_INVALID_OK: &[&str] = &["time::OffsetDateTime::now_local"];

/// Crate yang dilarang di semua manifes (stack §1.1 dan coding-standards §5.7).
const FORBIDDEN_DEPS: &[&str] = &["ring", "aes-gcm", "chrono"];
/// Crate yang hanya boleh dideklarasikan di `crates/crypto`.
const CRYPTO_ONLY_DEPS: &[&str] = &["aws-lc-rs", "aws-lc-sys", "aws-lc-fips-sys"];
/// Crate yang hanya boleh dideklarasikan di `crates/valkey` (F1-T33: batas waktu dan galat tanpa isi
/// seragam untuk semua akses Valkey).
const VALKEY_ONLY_DEPS: &[&str] = &["redis"];

/// Fitur pembantu test per paket: (paket, fitur). TD-15.
pub const TEST_FEATURES: &[(&str, &str)] = &[("authn", "testing"), ("crypto", "test-keys")];

fn finding(file: &str, detail: &str, msg: impl Into<String>) -> Finding {
    Finding::new("manifest", file, 0, detail, msg)
}

fn read_table(path: &Path) -> Result<Table, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("tidak terbaca: {e}"))?;
    toml::from_str::<Table>(&text).map_err(|e| format!("TOML tidak valid: {e}"))
}

/// Memeriksa root dan semua anggota workspace.
pub fn check_manifests(root: &Path) -> Vec<Finding> {
    let mut out = Vec::new();
    out.extend(check_root_lints(root));
    out.extend(check_clippy_toml(root));
    for top in ["apps", "crates"] {
        for name in fs::subdirs(&root.join(top)) {
            let rel = format!("{top}/{name}");
            let manifest = root.join(&rel).join("Cargo.toml");
            if manifest.exists() {
                out.extend(check_member(root, &rel));
            }
        }
        for name in [".clippy.toml", "clippy.toml"] {
            for p in fs::files_named(&root.join(top), name) {
                out.push(finding(
                    &fs::rel(root, &p),
                    "clippy.toml",
                    "clippy.toml per crate dilarang: konfigurasi hanya di clippy.toml root (F1-T12)",
                ));
            }
        }
    }
    out
}

fn check_root_lints(root: &Path) -> Vec<Finding> {
    let file = "Cargo.toml";
    let table = match read_table(&root.join(file)) {
        Ok(t) => t,
        Err(e) => return vec![finding(file, "root", e)],
    };
    let mut out = Vec::new();
    let clippy = table
        .get("workspace")
        .and_then(|w| w.get("lints"))
        .and_then(|l| l.get("clippy"))
        .and_then(Value::as_table);
    for lint in REQUIRED_LINTS {
        let level = clippy.and_then(|c| c.get(*lint)).and_then(|v| {
            v.as_str()
                .or_else(|| v.get("level").and_then(Value::as_str))
        });
        if !matches!(level, Some("deny" | "forbid")) {
            out.push(finding(
                file,
                lint,
                format!("[workspace.lints.clippy] {lint} wajib \"deny\" atau \"forbid\" (ditemukan: {level:?})"),
            ));
        }
    }
    out.extend(check_deps(file, "", &table));
    out.extend(check_test_features(file, "", &table));
    out
}

fn check_clippy_toml(root: &Path) -> Vec<Finding> {
    let file = "clippy.toml";
    let table = match read_table(&root.join(file)) {
        Ok(t) => t,
        Err(e) => return vec![finding(file, "root", e)],
    };
    let mut out = Vec::new();
    let entries: Vec<&Value> = table
        .get("disallowed-methods")
        .and_then(Value::as_array)
        .map(|a| a.iter().collect())
        .unwrap_or_default();
    for m in REQUIRED_METHODS {
        let entry = entries
            .iter()
            .find(|e| e.get("path").and_then(Value::as_str) == Some(*m));
        match entry {
            None => out.push(finding(
                file,
                m,
                format!("disallowed-methods wajib memuat {m}"),
            )),
            Some(e) => {
                let invalid_ok = e
                    .get("allow-invalid")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                if invalid_ok && !ALLOW_INVALID_OK.contains(m) {
                    out.push(finding(
                        file,
                        m,
                        format!("allow-invalid tidak diizinkan untuk {m}"),
                    ));
                }
            }
        }
    }
    out
}

fn check_member(root: &Path, rel: &str) -> Vec<Finding> {
    let file = format!("{rel}/Cargo.toml");
    let table = match read_table(&root.join(&file)) {
        Ok(t) => t,
        Err(e) => return vec![finding(&file, "member", e)],
    };
    let mut out = Vec::new();
    let lints_ok = table
        .get("lints")
        .and_then(Value::as_table)
        .is_some_and(|l| l.len() == 1 && l.get("workspace").and_then(Value::as_bool) == Some(true));
    if !lints_ok {
        out.push(finding(
            &file,
            "lints",
            "crate wajib `[lints] workspace = true` tanpa lint tambahan (F1-T12)",
        ));
    }
    let crate_dir = rel.rsplit('/').next().unwrap_or("");
    out.extend(check_deps(&file, crate_dir, &table));
    if rel.starts_with("apps/") {
        out.extend(check_axum_rename(&file, &table));
    }
    out.extend(check_test_features(&file, crate_dir, &table));
    out
}

/// `router-bypass` (TD-15, review F1-T38): aplikasi tidak boleh mengganti nama dependensi axum
/// (`x = { package = "axum" }`), karena nama baru menyembunyikan token yang diperiksa aturan sumber.
fn check_axum_rename(file: &str, table: &Table) -> Vec<Finding> {
    let mut out = Vec::new();
    let mut tables: Vec<&Table> = Vec::new();
    for k in ["dependencies", "dev-dependencies", "build-dependencies"] {
        if let Some(t) = table.get(k).and_then(Value::as_table) {
            tables.push(t);
        }
    }
    if let Some(targets) = table.get("target").and_then(Value::as_table) {
        for cfg in targets.values() {
            for k in ["dependencies", "dev-dependencies", "build-dependencies"] {
                if let Some(t) = cfg.get(k).and_then(Value::as_table) {
                    tables.push(t);
                }
            }
        }
    }
    for deps in tables {
        for (key, spec) in deps {
            if key != "axum" && spec.get("package").and_then(Value::as_str) == Some("axum") {
                out.push(Finding::new(
                    "router-bypass",
                    file,
                    0,
                    "axum-rename",
                    format!("dependensi axum diganti nama menjadi `{key}` di aplikasi (TD-15)"),
                ));
            }
        }
    }
    out
}

fn test_finding(file: &str, detail: &str, msg: impl Into<String>) -> Finding {
    Finding::new("test-feature", file, 0, detail, msg)
}

/// Tabel dependensi yang ikut build produksi (tanpa `[dev-dependencies]`).
fn production_dep_tables(table: &Table) -> Vec<&Table> {
    let mut v = Vec::new();
    for k in ["dependencies", "build-dependencies"] {
        if let Some(t) = table.get(k).and_then(Value::as_table) {
            v.push(t);
        }
    }
    if let Some(t) = table
        .get("workspace")
        .and_then(|w| w.get("dependencies"))
        .and_then(Value::as_table)
    {
        v.push(t);
    }
    if let Some(targets) = table.get("target").and_then(Value::as_table) {
        for cfg in targets.values() {
            for k in ["dependencies", "build-dependencies"] {
                if let Some(t) = cfg.get(k).and_then(Value::as_table) {
                    v.push(t);
                }
            }
        }
    }
    v
}

fn strings(v: Option<&Value>) -> Vec<&str> {
    v.and_then(Value::as_array)
        .map(|a| a.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default()
}

/// `test-feature` (TD-15): lihat dokumentasi modul.
fn check_test_features(file: &str, crate_dir: &str, table: &Table) -> Vec<Finding> {
    let mut out = Vec::new();
    for deps in production_dep_tables(table) {
        for (key, spec) in deps {
            let pkg = spec
                .get("package")
                .and_then(Value::as_str)
                .unwrap_or(key.as_str());
            let optional = spec.get("optional").and_then(Value::as_bool) == Some(true);
            for (owner, feature) in TEST_FEATURES {
                if pkg != *owner || !strings(spec.get("features")).contains(feature) {
                    continue;
                }
                if crate_dir == "authn" && pkg == "crypto" && optional {
                    continue;
                }
                out.push(test_finding(
                    file,
                    &format!("{owner}/{feature}"),
                    format!(
                        "fitur test `{owner}/{feature}` di dependensi produksi; hanya lewat [dev-dependencies] (TD-15)"
                    ),
                ));
            }
        }
    }
    let Some(features) = table.get("features").and_then(Value::as_table) else {
        return out;
    };
    for (name, enabled) in features {
        for item in strings(Some(enabled)) {
            for (owner, feature) in TEST_FEATURES {
                let direct = item == format!("{owner}/{feature}");
                let weak = item == format!("{owner}?/{feature}");
                let own_default = crate_dir == *owner && name == "default" && item == *feature;
                if direct || weak || own_default {
                    out.push(test_finding(
                        file,
                        &format!("{owner}/{feature}"),
                        format!(
                            "fitur `{name}` menyalakan fitur test `{owner}/{feature}`; hanya lewat [dev-dependencies] (TD-15)"
                        ),
                    ));
                }
            }
            // Dependensi opsional crypto milik authn hanya boleh dinyalakan fitur `testing`.
            if crate_dir == "authn"
                && name != "testing"
                && (item == "dep:crypto" || item == "crypto" || item.starts_with("crypto/"))
            {
                out.push(test_finding(
                    file,
                    "authn/testing",
                    format!(
                        "fitur `{name}` crate authn menyalakan dependensi crypto berfitur test-keys; hanya fitur `testing` (TD-15)"
                    ),
                ));
            }
        }
    }
    out
}

fn check_deps(file: &str, crate_dir: &str, table: &Table) -> Vec<Finding> {
    let mut out = Vec::new();
    let mut dep_tables: Vec<&Table> = Vec::new();
    for k in ["dependencies", "dev-dependencies", "build-dependencies"] {
        if let Some(t) = table.get(k).and_then(Value::as_table) {
            dep_tables.push(t);
        }
    }
    if let Some(t) = table
        .get("workspace")
        .and_then(|w| w.get("dependencies"))
        .and_then(Value::as_table)
    {
        dep_tables.push(t);
    }
    if let Some(targets) = table.get("target").and_then(Value::as_table) {
        for cfg in targets.values() {
            for k in ["dependencies", "dev-dependencies", "build-dependencies"] {
                if let Some(t) = cfg.get(k).and_then(Value::as_table) {
                    dep_tables.push(t);
                }
            }
        }
    }
    for deps in dep_tables {
        for (key, spec) in deps {
            let pkg = spec
                .get("package")
                .and_then(Value::as_str)
                .unwrap_or(key.as_str());
            if FORBIDDEN_DEPS.contains(&pkg) {
                out.push(finding(
                    file,
                    pkg,
                    format!(
                        "dependensi `{pkg}` dilarang (docs/05-stack.md, coding-standards §5.7)"
                    ),
                ));
            }
            if VALKEY_ONLY_DEPS.contains(&pkg) && crate_dir != "valkey" {
                out.push(finding(
                    file,
                    pkg,
                    format!("`{pkg}` hanya boleh dideklarasikan di crates/valkey (F1-T33)"),
                ));
            }
            if CRYPTO_ONLY_DEPS.contains(&pkg) && crate_dir != "crypto" {
                out.push(finding(
                    file,
                    pkg,
                    format!("`{pkg}` hanya boleh dideklarasikan di crates/crypto (KEPUTUSAN-74)"),
                ));
            }
            let features = spec.get("features").and_then(Value::as_array);
            if pkg == "time"
                && features.is_some_and(|f| f.iter().any(|x| x.as_str() == Some("local-offset")))
            {
                out.push(finding(
                    file,
                    "local-offset",
                    "fitur `local-offset` crate time dilarang: waktu hanya UTC (ADR-017)",
                ));
            }
        }
    }
    out
}
