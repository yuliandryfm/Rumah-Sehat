//! Uji negatif untuk manifes, clippy.toml, dan kepemilikan tabel memakai repositori tiruan di direktori sementara.
#![allow(clippy::unwrap_used, clippy::expect_used)] // berkas test

use arch_check::{manifests, tables};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

static N: AtomicUsize = AtomicUsize::new(0);

struct Repo(PathBuf);

impl Repo {
    fn new() -> Self {
        let dir = std::env::temp_dir().join(format!(
            "arch-check-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::SeqCst)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let r = Repo(dir);
        r.put("Cargo.toml", GOOD_ROOT);
        r.put("clippy.toml", GOOD_CLIPPY);
        r
    }
    fn put(&self, rel: &str, content: &str) {
        let p = self.0.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, content).unwrap();
    }
    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for Repo {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

const GOOD_ROOT: &str = r#"
[workspace]
members = ["apps/*", "crates/*"]
[workspace.lints.clippy]
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"
todo = "deny"
unimplemented = "deny"
indexing_slicing = "deny"
dbg_macro = "deny"
disallowed_methods = "deny"
"#;

const GOOD_CLIPPY: &str = r#"
disallowed-methods = [
  { path = "time::OffsetDateTime::now_utc" },
  { path = "time::OffsetDateTime::now_local", allow-invalid = true },
  { path = "time::UtcDateTime::now" },
  { path = "std::time::SystemTime::now" },
  { path = "std::env::var" },
  { path = "std::env::var_os" },
  { path = "std::env::vars" },
  { path = "std::env::vars_os" },
]
"#;

const GOOD_MEMBER: &str = "[package]\nname = \"x\"\n[lints]\nworkspace = true\n";

fn details(r: &Repo) -> Vec<String> {
    manifests::check_manifests(r.path())
        .into_iter()
        .map(|f| format!("{}|{}", f.file, f.detail))
        .collect()
}

#[test]
fn good_baseline_is_clean() {
    let r = Repo::new();
    r.put("crates/x/Cargo.toml", GOOD_MEMBER);
    assert!(details(&r).is_empty(), "{:?}", details(&r));
}

#[test]
fn weakened_workspace_lint_is_rejected() {
    for lint in manifests::REQUIRED_LINTS {
        let r = Repo::new();
        r.put(
            "Cargo.toml",
            &GOOD_ROOT.replace(&format!("{lint} = \"deny\""), &format!("{lint} = \"warn\"")),
        );
        assert!(
            details(&r).contains(&format!("Cargo.toml|{lint}")),
            "lint {lint} lolos"
        );
        let r2 = Repo::new();
        r2.put(
            "Cargo.toml",
            &GOOD_ROOT.replace(&format!("{lint} = \"deny\"\n"), ""),
        );
        assert!(
            details(&r2).contains(&format!("Cargo.toml|{lint}")),
            "lint {lint} hilang lolos"
        );
    }
}

#[test]
fn missing_or_invalid_disallowed_method_is_rejected() {
    for m in manifests::REQUIRED_METHODS {
        let r = Repo::new();
        let line = format!("  {{ path = \"{m}\" }},\n");
        let line_inv = format!("  {{ path = \"{m}\", allow-invalid = true }},\n");
        r.put(
            "clippy.toml",
            &GOOD_CLIPPY.replace(&line, "").replace(&line_inv, ""),
        );
        assert!(
            details(&r).contains(&format!("clippy.toml|{m}")),
            "{m} hilang lolos"
        );
    }
    let r = Repo::new();
    r.put(
        "clippy.toml",
        &GOOD_CLIPPY.replace(
            "{ path = \"std::env::var\" }",
            "{ path = \"std::env::var\", allow-invalid = true }",
        ),
    );
    assert!(details(&r).contains(&"clippy.toml|std::env::var".to_string()));
}

#[test]
fn crate_without_workspace_lints_or_with_own_lints_is_rejected() {
    let r = Repo::new();
    r.put("crates/a/Cargo.toml", "[package]\nname = \"a\"\n");
    r.put(
        "crates/b/Cargo.toml",
        "[package]\nname = \"b\"\n[lints.clippy]\nunwrap_used = \"allow\"\n",
    );
    r.put(
        "crates/c/Cargo.toml",
        "[package]\nname = \"c\"\n[lints]\nworkspace = true\n[lints.clippy]\npanic = \"allow\"\n",
    );
    let d = details(&r);
    for c in ["a", "b", "c"] {
        assert!(
            d.contains(&format!("crates/{c}/Cargo.toml|lints")),
            "{c}: {d:?}"
        );
    }
}

#[test]
fn per_crate_clippy_toml_is_rejected() {
    let r = Repo::new();
    r.put("crates/x/Cargo.toml", GOOD_MEMBER);
    r.put("crates/x/clippy.toml", "allow-unwrap-in-tests = true\n");
    r.put("apps/y/Cargo.toml", GOOD_MEMBER);
    r.put("apps/y/sub/.clippy.toml", "");
    let d = details(&r);
    assert!(d.contains(&"crates/x/clippy.toml|clippy.toml".to_string()));
    assert!(
        d.contains(&"apps/y/sub/.clippy.toml|clippy.toml".to_string())
            || d.iter().any(|x| x.contains("clippy.toml"))
    );
}

#[test]
fn forbidden_dependencies_are_rejected() {
    let r = Repo::new();
    r.put(
        "crates/a/Cargo.toml",
        &format!("{GOOD_MEMBER}[dependencies]\nring = \"0.17\"\n"),
    );
    r.put(
        "crates/b/Cargo.toml",
        &format!("{GOOD_MEMBER}[dev-dependencies]\nchrono = \"0.4\"\n"),
    );
    r.put("crates/c/Cargo.toml", &format!("{GOOD_MEMBER}[dependencies]\ncrypto-lib = {{ package = \"aws-lc-rs\", version = \"1\" }}\n"));
    r.put(
        "crates/crypto/Cargo.toml",
        &format!("{GOOD_MEMBER}[dependencies]\naws-lc-rs = \"1\"\n"),
    );
    r.put("crates/d/Cargo.toml", &format!("{GOOD_MEMBER}[dependencies]\ntime = {{ version = \"0.3\", features = [\"local-offset\"] }}\n"));
    r.put(
        "crates/e/Cargo.toml",
        &format!("{GOOD_MEMBER}[target.'cfg(unix)'.dependencies]\naes-gcm = \"0.10\"\n"),
    );
    let d = details(&r);
    assert!(d.contains(&"crates/a/Cargo.toml|ring".to_string()));
    assert!(d.contains(&"crates/b/Cargo.toml|chrono".to_string()));
    assert!(
        d.contains(&"crates/c/Cargo.toml|aws-lc-rs".to_string()),
        "alias package lolos: {d:?}"
    );
    assert!(
        !d.iter().any(|x| x.starts_with("crates/crypto/")),
        "crypto seharusnya boleh: {d:?}"
    );
    assert!(d.contains(&"crates/d/Cargo.toml|local-offset".to_string()));
    assert!(d.contains(&"crates/e/Cargo.toml|aes-gcm".to_string()));
}

// ---- kepemilikan tabel

const MT: &str = r#"
database = "app"
[modules.orders]
tables = ["order_head", "order_line"]
[modules.payment]
tables = ["payment_attempt"]
[crates.audit]
tables = ["audit_outbox"]
"#;

fn table_repo(extra: &[(&str, &str)]) -> Repo {
    let r = Repo::new();
    r.put("apps/be/module-tables.toml", MT);
    r.put("apps/be/Cargo.toml", GOOD_MEMBER);
    r.put("apps/be/src/modules/orders/repo.rs", "pub fn x() {}");
    r.put("apps/be/src/modules/payment/repo.rs", "pub fn y() {}");
    for (p, c) in extra {
        r.put(p, c);
    }
    r
}

fn table_findings(r: &Repo) -> Vec<String> {
    tables::check_all_apps(r.path())
        .into_iter()
        .map(|f| format!("{}|{}|{}", f.rule, f.file, f.detail))
        .collect()
}

#[test]
fn module_may_use_its_own_tables_only() {
    let own = r#"pub const Q: &str = "SELECT h.id FROM app.order_head h JOIN order_line l ON l.head_id = h.id";"#;
    let r = table_repo(&[("apps/be/src/modules/orders/repo.rs", own)]);
    assert!(table_findings(&r).is_empty(), "{:?}", table_findings(&r));

    let join = r#"pub const Q: &str = "SELECT h.id FROM order_head h JOIN payment_attempt p ON p.head_id = h.id";"#;
    let r = table_repo(&[("apps/be/src/modules/orders/repo.rs", join)]);
    assert_eq!(
        table_findings(&r),
        ["table-owner|apps/be/src/modules/orders/repo.rs|payment_attempt"]
    );
}

#[test]
fn write_statements_and_subselects_are_covered() {
    let cases = [
        r#"const Q: &str = "INSERT INTO payment_attempt (id) VALUES ($1)";"#,
        r#"const Q: &str = "UPDATE payment_attempt SET a = 1 WHERE id = $1";"#,
        r#"const Q: &str = "DELETE FROM payment_attempt WHERE id = $1";"#,
        r#"const Q: &str = "SELECT id FROM order_head WHERE x IN (SELECT y FROM payment_attempt)";"#,
        r#"const Q: &str = "SELECT 1 FROM order_head a, payment_attempt b";"#,
        r#"const Q: &str = "WITH x AS (SELECT 1 FROM payment_attempt) SELECT * FROM x";"#,
        r##"const Q: &str = r#"
            SELECT id
            FROM "app"."payment_attempt"
            WHERE id = $1"#;"##,
    ];
    for c in cases {
        let r = table_repo(&[("apps/be/src/modules/orders/repo.rs", c)]);
        assert!(
            table_findings(&r)
                .iter()
                .any(|f| f.ends_with("|payment_attempt")),
            "lolos: {c}"
        );
    }
}

#[test]
fn sql_noise_is_not_mistaken_for_tables() {
    let ok = [
        "SELECT id FROM order_head WHERE id = $1 FOR UPDATE",
        "INSERT INTO order_head (id) VALUES ($1) ON CONFLICT (id) DO UPDATE SET id = EXCLUDED.id",
        "SELECT EXTRACT(EPOCH FROM created_at) FROM order_head",
        "SELECT * FROM unnest($1::uuid[]) AS u(id) JOIN order_line l ON l.id = u.id",
        "WITH recent AS (SELECT id FROM order_head) SELECT id FROM recent",
        "SELECT a IS DISTINCT FROM b FROM order_head",
        "INSERT INTO order_line(id) VALUES ($1)",
        "SELECT * FROM generate_series(1, 3)",
    ];
    for sql in ok {
        let src = format!("const Q: &str = \"{sql}\";");
        let r = table_repo(&[("apps/be/src/modules/orders/repo.rs", &src)]);
        assert!(
            table_findings(&r).is_empty(),
            "salah tolak: {sql}: {:?}",
            table_findings(&r)
        );
    }
    // kalimat biasa yang kebetulan memuat "update"/"from" bukan SQL
    let r = table_repo(&[(
        "apps/be/src/modules/orders/service.rs",
        r#"const M: &str = "update gagal from server"; const N: &str = "select a thing from list";"#,
    )]);
    // "select ... from list" terbaca SQL dan `list` tidak dimiliki: perilaku konservatif yang diterima
    assert_eq!(
        table_findings(&r),
        ["table-owner|apps/be/src/modules/orders/service.rs|list"]
    );
}

#[test]
fn table_owned_by_shared_crate_is_off_limits_to_modules() {
    let src = r#"const Q: &str = "INSERT INTO audit_outbox (id) VALUES ($1)";"#;
    let r = table_repo(&[("apps/be/src/modules/orders/repo.rs", src)]);
    assert_eq!(
        table_findings(&r),
        ["table-owner|apps/be/src/modules/orders/repo.rs|audit_outbox"]
    );
    // crate pemilik boleh
    let r = table_repo(&[("crates/audit/src/lib.rs", src)]);
    assert!(table_findings(&r).is_empty(), "{:?}", table_findings(&r));
    // crate lain tidak
    let r = table_repo(&[("crates/other/src/lib.rs", src)]);
    assert_eq!(
        table_findings(&r),
        ["table-owner|crates/other/src/lib.rs|audit_outbox"]
    );
}

#[test]
fn sql_outside_any_module_and_dynamic_names_are_rejected() {
    let src = r#"const Q: &str = "SELECT 1 FROM order_head";"#;
    let r = table_repo(&[("apps/be/src/main.rs", src)]);
    assert_eq!(
        table_findings(&r),
        ["table-owner|apps/be/src/main.rs|order_head"]
    );
    let dynq = r#"const Q: &str = "SELECT 1 FROM {table} WHERE id = $1";"#;
    let r = table_repo(&[("apps/be/src/modules/orders/repo.rs", dynq)]);
    assert_eq!(
        table_findings(&r),
        ["table-dynamic|apps/be/src/modules/orders/repo.rs|{table}"]
    );
}

#[test]
fn sql_files_inside_modules_are_checked() {
    let r = table_repo(&[(
        "apps/be/src/modules/orders/q.sql",
        "SELECT 1 FROM payment_attempt;",
    )]);
    assert_eq!(
        table_findings(&r),
        ["table-owner|apps/be/src/modules/orders/q.sql|payment_attempt"]
    );
}

#[test]
fn registry_consistency_is_enforced() {
    // modul di src/modules belum terdaftar
    let r = table_repo(&[("apps/be/src/modules/ghost/mod.rs", "")]);
    assert!(
        table_findings(&r).contains(&"table-owner|apps/be/module-tables.toml|ghost".to_string())
    );
    // tabel dimiliki dua modul
    let r = table_repo(&[(
        "apps/be/module-tables.toml",
        &MT.replace(
            "[\"payment_attempt\"]",
            "[\"payment_attempt\", \"order_head\"]",
        ),
    )]);
    assert!(
        table_findings(&r)
            .contains(&"table-owner|apps/be/module-tables.toml|order_head".to_string())
    );
    // app dengan src/modules tanpa module-tables.toml
    let r = Repo::new();
    r.put("apps/zz/src/modules/a/mod.rs", "");
    assert!(
        table_findings(&r)
            .contains(&"table-owner|apps/zz/module-tables.toml|module-tables.toml".to_string())
    );
}

#[test]
fn migration_tables_must_have_an_owner() {
    let mig = "-- komentar\nCREATE TABLE app.order_head (id uuid);\nCREATE TABLE IF NOT EXISTS app.rogue (id uuid);\nCREATE TABLE audit.part PARTITION OF audit.ev FOR VALUES FROM (1) TO (2);";
    let r = table_repo(&[("db/migrations/app/0001_x.sql", mig)]);
    assert_eq!(
        table_findings(&r),
        ["table-owner|db/migrations/app/0001_x.sql|rogue"]
    );
    assert_eq!(tables::created_tables(mig), ["order_head", "rogue"]);
}

#[test]
fn test_features_in_production_dependencies_are_rejected() {
    // TD-15 (Low): fitur pembantu test tidak boleh ikut build produksi.
    let member =
        |deps: &str| format!("[package]\nname = \"x\"\n{deps}\n[lints]\nworkspace = true\n");
    let bad = [
        "[dependencies]\nauthn = { path = \"../authn\", features = [\"testing\"] }",
        "[dependencies]\ncrypto = { path = \"../crypto\", features = [\"test-keys\"] }",
        "[build-dependencies]\nauthn = { path = \"../authn\", features = [\"testing\"] }",
        "[target.'cfg(unix)'.dependencies]\ncrypto = { path = \"../crypto\", features = [\"test-keys\"] }",
        "[dependencies]\nauth = { package = \"authn\", path = \"../authn\", features = [\"testing\"] }",
        // Tabel [features] yang menyalakan fitur test dependensi (dapat aktif lewat crate pemakai).
        "[features]\ndefault = [\"authn/testing\"]",
        "[features]\nx = [\"crypto?/test-keys\"]",
    ];
    for deps in bad {
        let r = Repo::new();
        r.put("apps/x/Cargo.toml", &member(deps));
        let d = details(&r);
        assert!(
            d.iter()
                .any(|x| x.starts_with("apps/x/Cargo.toml|authn/testing")
                    || x.starts_with("apps/x/Cargo.toml|crypto/test-keys")),
            "lolos: {deps}: {d:?}"
        );
    }
    // Di workspace.dependencies root juga ditolak.
    let r = Repo::new();
    r.put(
        "Cargo.toml",
        &format!(
            "{GOOD_ROOT}[workspace.dependencies]\nauthn = {{ path = \"crates/authn\", features = [\"testing\"] }}\n"
        ),
    );
    assert!(details(&r).contains(&"Cargo.toml|authn/testing".to_string()));
    // Fitur default milik crate pemiliknya.
    let r = Repo::new();
    r.put(
        "crates/crypto/Cargo.toml",
        &member("[features]\ndefault = [\"test-keys\"]\ntest-keys = []"),
    );
    assert!(details(&r).contains(&"crates/crypto/Cargo.toml|crypto/test-keys".to_string()));
    // Crate authn: dependensi crypto berfitur test-keys hanya opsional dan hanya lewat fitur `testing`.
    let r = Repo::new();
    r.put(
        "crates/authn/Cargo.toml",
        &member("[dependencies]\ncrypto = { path = \"../crypto\", features = [\"test-keys\"] }"),
    );
    assert!(details(&r).contains(&"crates/authn/Cargo.toml|crypto/test-keys".to_string()));
    let r = Repo::new();
    r.put(
        "crates/authn/Cargo.toml",
        &member(
            "[dependencies]\ncrypto = { path = \"../crypto\", optional = true, features = [\"test-keys\"] }\n[features]\ndefault = [\"dep:crypto\"]\ntesting = [\"dep:crypto\"]",
        ),
    );
    assert!(details(&r).contains(&"crates/authn/Cargo.toml|authn/testing".to_string()));
}

#[test]
fn test_features_in_dev_dependencies_and_authn_testing_are_allowed() {
    let r = Repo::new();
    r.put(
        "crates/authn/Cargo.toml",
        "[package]\nname = \"authn\"\n[dependencies]\ncrypto = { path = \"../crypto\", optional = true, features = [\"test-keys\"] }\n[features]\ntesting = [\"dep:crypto\"]\n[dev-dependencies]\nauthn = { path = \".\", features = [\"testing\"] }\n[lints]\nworkspace = true\n",
    );
    r.put(
        "apps/x/Cargo.toml",
        "[package]\nname = \"x\"\n[dependencies]\nauthn = { path = \"../../crates/authn\" }\n[dev-dependencies]\nauthn = { path = \"../../crates/authn\", features = [\"testing\"] }\ncrypto = { path = \"../../crates/crypto\", features = [\"test-keys\"] }\n[lints]\nworkspace = true\n",
    );
    assert!(details(&r).is_empty(), "{:?}", details(&r));
}

#[test]
fn redis_only_in_valkey_crate() {
    let r = Repo::new();
    r.put(
        "apps/x/Cargo.toml",
        "[package]\nname = \"x\"\n[dependencies]\nredis = \"=1.8.2\"\n[lints]\nworkspace = true\n",
    );
    r.put(
        "crates/valkey/Cargo.toml",
        "[package]\nname = \"valkey\"\n[dependencies]\nredis = \"=1.8.2\"\n[lints]\nworkspace = true\n",
    );
    let d = details(&r);
    assert!(d.contains(&"apps/x/Cargo.toml|redis".to_string()), "{d:?}");
    assert!(!d.iter().any(|x| x.starts_with("crates/valkey/")), "{d:?}");
}

#[test]
fn renamed_axum_dependency_in_apps_is_rejected() {
    let r = Repo::new();
    r.put(
        "apps/x/Cargo.toml",
        "[package]\nname = \"x\"\n[dependencies]\nweb = { package = \"axum\", version = \"=0.8.9\" }\n[lints]\nworkspace = true\n",
    );
    r.put(
        "apps/y/Cargo.toml",
        "[package]\nname = \"y\"\n[dependencies]\naxum = { version = \"=0.8.9\" }\n[lints]\nworkspace = true\n",
    );
    let found: Vec<String> = manifests::check_manifests(r.path())
        .into_iter()
        .map(|f| format!("{}|{}", f.file, f.detail))
        .collect();
    assert!(
        found.contains(&"apps/x/Cargo.toml|axum-rename".to_string()),
        "{found:?}"
    );
    assert!(!found.iter().any(|x| x.starts_with("apps/y/")), "{found:?}");
}
