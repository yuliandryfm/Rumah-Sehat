//! Pemeriksaan manifes dan konfigurasi clippy: larangan tidak boleh dilemahkan atau ditimpa per crate.

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
