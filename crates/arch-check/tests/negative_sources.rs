//! Uji negatif aturan berbasis teks: tiap larangan terbukti menolak pelanggaran dan tidak
//! menolak yang sah. Fixture berupa teks Rust di memori.
#![allow(clippy::unwrap_used, clippy::expect_used)] // berkas test

use arch_check::exceptions::{self, apply};
use arch_check::sources::check_source_text;

const SRC: &str = "apps/rs-backend/src/modules/orders/service.rs";

fn rules(file: &str, src: &str) -> Vec<String> {
    check_source_text(file, src)
        .into_iter()
        .map(|f| format!("{}:{}", f.rule, f.detail))
        .collect()
}

fn has(file: &str, src: &str, want: &str) -> bool {
    rules(file, src).iter().any(|r| r == want)
}

#[test]
fn clock_calls_are_rejected() {
    for call in [
        "time::OffsetDateTime::now_utc()",
        "OffsetDateTime::now_local()",
        "std::time::SystemTime::now()",
        "time::UtcDateTime::now()",
    ] {
        let src = format!("fn f() {{ let _ = {call}; }}");
        assert!(!rules(SRC, &src).is_empty(), "lolos: {call}");
    }
}

#[test]
fn clock_mentions_in_comments_and_strings_are_ignored() {
    let src = "// OffsetDateTime::now_utc()\nfn f() { let _ = \"SystemTime::now\"; /* now_utc */ }";
    assert!(rules(SRC, src).is_empty());
}

#[test]
fn env_reads_are_rejected_but_neighbors_are_not_confused() {
    assert!(has(
        SRC,
        "fn f() { let _ = std::env::var(\"X\"); }",
        "env:env::var"
    ));
    assert!(has(
        SRC,
        "fn f() { let _ = std::env::var_os(\"X\"); }",
        "env:env::var_os"
    ));
    assert!(has(
        SRC,
        "fn f() { let _ = std::env::vars(); }",
        "env:env::vars"
    ));
    assert!(has(
        SRC,
        "fn f() { let _ = std::env::vars_os(); }",
        "env:env::vars_os"
    ));
    // `env::var_os` tidak boleh juga dilaporkan sebagai `env::var`.
    assert!(!has(
        SRC,
        "fn f() { let _ = std::env::var_os(\"X\"); }",
        "env:env::var"
    ));
    // args() dan makro compile-time bukan pembacaan lingkungan runtime.
    assert!(
        rules(
            SRC,
            "fn f() { let _ = std::env::args(); let _ = env!(\"CARGO\"); }"
        )
        .is_empty()
    );
}

#[test]
fn env_and_clock_report_enclosing_fn_for_narrow_exceptions() {
    let src = "impl A {\n fn from_process() { std::env::vars_os(); }\n fn other() { std::env::vars_os(); }\n}";
    let f = check_source_text("crates/observability/src/config.rs", src);
    let funcs: Vec<_> = f
        .iter()
        .map(|x| x.func.clone().unwrap_or_default())
        .collect();
    assert_eq!(funcs, ["from_process", "other"]);
}

#[test]
fn aws_lc_rs_only_in_crypto_crate() {
    let src = "use aws_lc_rs::aead::AES_256_GCM;";
    assert!(has("crates/foundation/src/x.rs", src, "aws-lc:aws_lc_rs"));
    assert!(has(SRC, src, "aws-lc:aws_lc_rs"));
    assert!(rules("crates/crypto/src/lib.rs", src).is_empty());
}

#[test]
fn response_code_creation_is_restricted() {
    let src = "const X: ResponseCode = ResponseCode::new(\"2000000\", \"ok\");";
    assert!(has(
        "apps/rs-backend/src/modules/orders/http.rs",
        src,
        "response-code:ResponseCode::new"
    ));
    assert!(has(
        "apps/rs-backend/src/app.rs",
        src,
        "response-code:ResponseCode::new"
    ));
    assert!(has(
        "crates/observability/src/log.rs",
        src,
        "response-code:ResponseCode::new"
    ));
    assert!(has(
        SRC,
        "use envelope::ResponseCode as RC;",
        "response-code:ResponseCode as"
    ));
    assert!(rules("crates/envelope/src/code.rs", src).is_empty());
    assert!(rules("apps/rs-backend/src/modules/orders/error.rs", src).is_empty());
    assert!(rules("apps/rs-backend/tests/x.rs", src).is_empty());
}

#[test]
fn muting_lint_attributes_are_rejected_in_production_code_only() {
    for attr in [
        "#[allow(clippy::unwrap_used)]",
        "#![allow(clippy::expect_used, dead_code)]",
        "#[expect(clippy::panic)]",
        "#[allow(clippy::indexing_slicing)]",
        "#[allow(clippy::disallowed_methods)]",
        "#[cfg_attr(not(test), allow(clippy::todo))]",
        "#![allow(warnings)]",
        "#[allow(clippy::all)]",
    ] {
        let src = format!("{attr}\nfn f() {{}}");
        assert!(
            rules(SRC, &src)
                .iter()
                .any(|r| r.starts_with("attr-allow:")),
            "lolos: {attr}"
        );
        assert!(
            rules("apps/rs-backend/tests/t.rs", &src).is_empty(),
            "test seharusnya boleh: {attr}"
        );
    }
    assert!(rules(SRC, "#[allow(dead_code)]\nfn f() {}").is_empty());
    // `.expect(` sebagai pemanggilan metode bukan atribut.
    assert!(rules(SRC, "fn f() { x.expect(\"clippy::panic\"); }").is_empty());
}

#[test]
fn print_macros_are_rejected_outside_tests() {
    for m in [
        "println!(\"x\")",
        "eprintln!(\"x\")",
        "print!(\"x\")",
        "eprint!(\"x\")",
        "dbg!(1)",
    ] {
        let src = format!("fn f() {{ {m}; }}");
        assert!(
            rules(SRC, &src).iter().any(|r| r.starts_with("print:")),
            "lolos: {m}"
        );
        assert!(rules("crates/foundation/tests/t.rs", &src).is_empty());
    }
}

#[test]
fn debug_formatted_log_fields_are_rejected() {
    for call in [
        "tracing::info!(?order, \"x\")",
        "tracing::info!(order = ?o, \"x\")",
        "tracing::warn!(id = %id, detail = ?d, \"x\")",
        "tracing::error!(\"gagal {e:?}\")",
        "tracing::debug!(\"gagal {:?}\", e)",
    ] {
        let src = format!("fn f() {{ {call}; }}");
        assert!(
            rules(SRC, &src).iter().any(|r| r.starts_with("log-debug:")),
            "lolos: {call}"
        );
    }
}

#[test]
fn sensitive_names_in_log_are_rejected() {
    for call in [
        "tracing::info!(email = %acc.email, \"x\")",
        "tracing::info!(token = %t, \"x\")",
        "tracing::info!(password, \"x\")",
        "tracing::info!(nik = %p.nik, \"x\")",
        "tracing::info!(\"login {token}\")",
        "tracing::warn!(request_body = %b, \"x\")",
        "tracing::warn!(Authorization = %h, \"x\")",
    ] {
        let src = format!("fn f() {{ {call}; }}");
        assert!(
            rules(SRC, &src).iter().any(|r| r.starts_with("log-field:")),
            "lolos: {call}"
        );
    }
}

#[test]
fn clean_log_calls_pass() {
    let src = r#"use tracing as _;
fn f() {
    tracing::info!(component = "api", order_id = %id, status = 200, duration_ms = 3u64, "request_completed");
    tracing::warn!(check = c.name(), "readiness_check_failed");
    tracing::info!(count = f()?, "ok {{literal}} {order_id}");
}"#;
    assert!(rules(SRC, src).is_empty(), "{:?}", rules(SRC, src));
}

#[test]
fn instrument_without_skip_all_is_rejected() {
    assert!(has(
        SRC,
        "#[tracing::instrument]\nfn f() {}",
        "instrument:instrument"
    ));
    assert!(has(
        SRC,
        "#[instrument(skip(a))]\nfn f() {}",
        "instrument:instrument"
    ));
    assert!(rules(SRC, "#[tracing::instrument(skip_all)]\nfn f() {}").is_empty());
}

#[test]
fn lexer_handles_raw_strings_chars_and_lifetimes() {
    let src = "fn f<'a>(x: &'a str) { let _c = '\"'; let _s = r#\"now_utc\"#; let _t = \"a\\\"now_utc\"; }";
    assert!(rules(SRC, src).is_empty(), "{:?}", rules(SRC, src));
    assert!(!rules(SRC, "fn f<'a>(x: &'a str) { now_utc(); }").is_empty());
    // Identifier berawalan `br` (mis. `break`) bukan pembuka byte raw string; sebelum F1-T23 lexer
    // menganggapnya literal sehingga kode sesudahnya tersembunyi dari pemeriksaan.
    let src = "fn f() { loop { break; } now_utc(); let _b = br\"x\"; let _c = b\"y\"; }";
    assert!(has(SRC, src, "clock:now_utc"), "{:?}", rules(SRC, src));
    let src = "fn f() { loop { break; } let _ = br#\"now_utc\"#; }";
    assert!(rules(SRC, src).is_empty(), "{:?}", rules(SRC, src));
}

// ---- pengecualian

const EX: &str = r#"
[[exception]]
rule = "clock"
file = "crates/foundation/src/clock.rs"
detail = "now_utc"
in_fn = "now"
reason = "SystemClock"
"#;

fn apply_to(file: &str, src: &str, ex: &str) -> Vec<String> {
    let ex = exceptions::parse(ex).unwrap();
    apply(check_source_text(file, src), &ex)
        .into_iter()
        .map(|f| format!("{}:{}", f.rule, f.detail))
        .collect()
}

#[test]
fn exception_covers_exactly_the_declared_site() {
    let file = "crates/foundation/src/clock.rs";
    let ok = "impl C { fn now(&self) { OffsetDateTime::now_utc() } }";
    assert!(apply_to(file, ok, EX).is_empty());
    // fungsi lain di berkas yang sama tidak tertutup
    let other = "impl C { fn now(&self) { OffsetDateTime::now_utc() } fn later() { OffsetDateTime::now_utc() } }";
    assert_eq!(apply_to(file, other, EX), ["clock:now_utc"]);
    // berkas lain tidak tertutup
    assert_eq!(
        apply_to("crates/foundation/src/other.rs", ok, EX)
            .first()
            .map(String::as_str),
        Some("clock:now_utc")
    );
}

#[test]
fn exception_beyond_count_or_stale_or_without_reason_fails() {
    let file = "crates/foundation/src/clock.rs";
    let twice = "fn now() { OffsetDateTime::now_utc(); OffsetDateTime::now_utc(); }";
    assert_eq!(apply_to(file, twice, EX), ["clock:now_utc"]);
    let stale = apply_to(file, "fn now() {}", EX);
    assert_eq!(stale, ["exceptions:now_utc"]);
    let no_reason = EX.replace("reason = \"SystemClock\"", "reason = \" \"");
    let r = apply_to(file, "fn now() { OffsetDateTime::now_utc() }", &no_reason);
    assert_eq!(r, ["exceptions:now_utc"]);
    // count terlalu longgar
    let loose = format!("{EX}count = 3\n");
    let l = apply_to(file, "fn now() { OffsetDateTime::now_utc() }", &loose);
    assert_eq!(l, ["exceptions:now_utc"]);
}

/// Review keamanan F1-T23 BLOKIR-2: tabel outbox audit hanya disebut di crate audit, termasuk lewat
/// SQL dinamis atau konstanta nama tabel yang tidak terlihat oleh aturan `table-owner`.
#[test]
fn audit_outbox_only_in_audit_crate() {
    for src in [
        r#"const T: &str = "app.audit_outbox";"#,
        r#"fn q(t: &str) -> String { format!("INSERT INTO {} (id) VALUES ($1)", "AUDIT_OUTBOX") }"#,
        r#"const T: &str = "rme_audit_outbox";"#,
        "struct audit_outbox;",
    ] {
        let found = rules(SRC, src);
        assert!(
            found.iter().any(|r| r.starts_with("audit-outbox:")),
            "lolos: {src} -> {found:?}"
        );
        assert!(
            rules("crates/platform-jobs/src/lib.rs", src)
                .iter()
                .any(|r| r.starts_with("audit-outbox:")),
            "lolos di crate lain: {src}"
        );
        assert!(
            !rules("crates/audit/src/outbox.rs", src)
                .iter()
                .any(|r| r.starts_with("audit-outbox:")),
            "crate audit ditolak: {src}"
        );
    }
    // Test boleh menyebut tabel (mis. memeriksa hak role); komentar tidak dihitung.
    let src = r#"// audit_outbox
fn f() {}"#;
    assert!(rules(SRC, src).is_empty());
    assert!(rules("crates/x/tests/t.rs", r#"const T: &str = "audit_outbox";"#).is_empty());
}

/// Review keamanan F1-T23 BLOKIR-2: SQL kendali transaksi mentah dapat mengakhiri transaksi teraudit
/// dari dalam aksi bisnis; literal semacam itu ditolak di semua kode non-test, termasuk crate audit.
#[test]
fn raw_transaction_control_sql_is_rejected() {
    for stmt in [
        "COMMIT",
        "commit;",
        "ROLLBACK",
        "rollback to savepoint sp1",
        "END",
        "ABORT",
        "BEGIN",
        "begin isolation level serializable",
        "START TRANSACTION",
        "SAVEPOINT sp1",
        "RELEASE SAVEPOINT sp1",
        "SELECT 1; COMMIT",
        "COMMIT PREPARED 'x'",
        "SET TRANSACTION READ ONLY",
    ] {
        let src = format!("fn f() {{ let _ = sqlx::raw_sql({stmt:?}); }}");
        for file in [SRC, "crates/audit/src/outbox.rs"] {
            assert!(
                rules(file, &src)
                    .iter()
                    .any(|r| r.starts_with("tx-control:")),
                "lolos di {file}: {stmt}"
            );
        }
    }
    for ok in [
        "commit message",
        "release notes",
        "end of list",
        "begin_at",
        "SELECT committed FROM t",
        "rollback_reason",
        "set timezone",
    ] {
        let src = format!("fn f() {{ let _ = {ok:?}; }}");
        assert!(
            !rules(SRC, &src)
                .iter()
                .any(|r| r.starts_with("tx-control:")),
            "salah tolak: {ok}"
        );
    }
    assert!(rules("crates/audit/tests/x.rs", r#"const S: &str = "COMMIT";"#).is_empty());
}

#[test]
fn router_bypass_in_app_sources_is_rejected() {
    // TD-15 (Medium): rute di luar PolicyRouter atau router yang dilayani tanpa EnforcedRouter tidak
    // dibungkus middleware penegak.
    let cases = [
        ("fn f(r: axum::Router) {}", "router-bypass:axum::Router"),
        ("use axum::Router;", "router-bypass:Router"),
        ("use axum::{Json, Router};", "router-bypass:Router"),
        (
            "fn f() { let _ = utoipa_axum::router::OpenApiRouter::<()>::new(); }",
            "router-bypass:OpenApiRouter",
        ),
        (
            "fn f(r: R) { r.route(\"/x\", get(h)); }",
            "router-bypass:route",
        ),
        (
            "fn f(r: R) { r\n  .nest(\"/x\", o); }",
            "router-bypass:nest",
        ),
        ("fn f(r: R) { r.fallback(h); }", "router-bypass:fallback"),
        (
            "fn f(r: R) { r.route_service(\"/x\", s); }",
            "router-bypass:route_service",
        ),
        (
            "fn f(r: R) { r.nest_service(\"/x\", s); }",
            "router-bypass:nest_service",
        ),
        (
            "fn f(r: R) { r.fallback_service(s); }",
            "router-bypass:fallback_service",
        ),
        (
            "fn f(r: R) { r.method_not_allowed_fallback(h); }",
            "router-bypass:method_not_allowed_fallback",
        ),
        (
            "fn f(r: R) { r.route_layer(l); }",
            "router-bypass:route_layer",
        ),
        (
            "fn f(a: R, b: R) { let x = a.merge(b); }",
            "router-bypass:merge",
        ),
        (
            "async fn f(l: L, r: R) { axum::serve(l, r).await; }",
            "router-bypass:axum::serve",
        ),
        ("use axum::{serve, Json};", "router-bypass:serve"),
        ("use axum::{routing::get};", "router-bypass:routing"),
        (
            "fn f() { let _ = axum::routing::get(h); }",
            "router-bypass:axum::routing",
        ),
        (
            "fn f(m: M) { let _ = m.into_make_service(); }",
            "router-bypass:into_make_service",
        ),
        (
            "async fn f(a: A, p: P) { a.authenticate_parts(&p).await; }",
            "router-bypass:authenticate_parts",
        ),
        (
            "async fn f(a: A, h: H) { a.authenticate_headers(h).await; }",
            "router-bypass:authenticate_headers",
        ),
        (
            "async fn f(a: A, t: &str) { a.verify(t).await; }",
            "router-bypass:verify",
        ),
        (
            "async fn f(a: A, t: &str) { Authenticator::verify(&a, t).await; }",
            "router-bypass:Authenticator::verify",
        ),
    ];
    for (src, want) in cases {
        assert!(has(SRC, src, want), "lolos: {src}: {:?}", rules(SRC, src));
    }
}

#[test]
fn policy_router_usage_and_non_app_files_are_allowed() {
    let ok = r#"
use authz::{EnforcedRouter, PolicyRouter};
pub(crate) fn ops_router() -> PolicyRouter<AppState> {
    PolicyRouter::with_openapi(OpsApiDoc::openapi()).merge(platform::router())
}
fn modules() -> PolicyRouter<AppState> {
    platform::router().merge(identity::router())
}
pub(super) fn router() -> PolicyRouter<AppState> {
    PolicyRouter::new().routes(policy::health(), routes!(live))
}
async fn run(r: EnforcedRouter, l: TcpListener) { r.serve(l, shutdown()).await; }
// axum::Router dan .route( di komentar diabaikan
fn s() -> &'static str { "axum::serve .route(" }
"#;
    let found: Vec<String> = rules(SRC, ok)
        .into_iter()
        .filter(|r| r.starts_with("router-bypass"))
        .collect();
    assert!(found.is_empty(), "{found:?}");
    // Aturan hanya untuk apps/*/src: test aplikasi dan crate pustaka (authz, authn) tidak diperiksa.
    let raw = "fn f(r: axum::Router) { r.route(\"/x\", get(h)); }";
    for file in [
        "apps/rs-backend/tests/common/mod.rs",
        "crates/authz/src/router.rs",
        "crates/authn/tests/authn_http.rs",
    ] {
        assert!(
            !rules(file, raw)
                .iter()
                .any(|r| r.starts_with("router-bypass")),
            "{file}"
        );
    }
}

#[test]
fn axum_aliases_in_app_sources_are_rejected() {
    // Review F1-T38 butir 2: alias crate menyembunyikan token axum::serve/axum::routing/Router.
    let cases = [
        ("use axum as ax;\nfn f() {}", "router-bypass:axum-alias"),
        ("use axum::{self as ax, Json};", "router-bypass:axum-alias"),
        ("extern crate axum as ax;", "router-bypass:axum-alias"),
        (
            "use axum as ax;\nasync fn f(l: L, r: R) { ax::serve(l, r).await; }",
            "router-bypass:serve",
        ),
        (
            "use axum as ax;\nfn f() { let _ = ax::routing::get(h); }",
            "router-bypass:routing",
        ),
        (
            "use axum::{self as ax};\nfn f(r: ax::Router) {}",
            "router-bypass:Router",
        ),
    ];
    for (src, want) in cases {
        assert!(has(SRC, src, want), "lolos: {src}: {:?}", rules(SRC, src));
    }
    // Impor biasa tanpa alias tidak ditolak.
    assert!(
        !rules(
            SRC,
            "use axum::extract::Path;\nuse axum::http::StatusCode as Status;"
        )
        .iter()
        .any(|r| r == "router-bypass:axum-alias")
    );
}

#[test]
fn axum_imports_in_any_shape_are_rejected() {
    // Review keamanan F1-T33 blokir 2: bentuk impor yang lolos pemeriksa sebelumnya.
    let cases = [
        ("use ::axum as x;", "router-bypass:axum-alias"),
        ("use  axum  as  x;", "router-bypass:axum-alias"),
        ("use axum\n    as x;", "router-bypass:axum-alias"),
        ("pub use axum as x;", "router-bypass:axum-alias"),
        ("extern  crate\n axum\n as x;", "router-bypass:axum-alias"),
        (
            "use ::axum::{serve, routing::get};\nasync fn f(l: L, r: R) { serve(l, r).await; }",
            "router-bypass:serve",
        ),
        (
            "use ::axum::{serve, routing::get};",
            "router-bypass:routing",
        ),
        ("use {axum::{serve, routing::get}};", "router-bypass:serve"),
        (
            "use {axum::{serve, routing::get}};",
            "router-bypass:routing",
        ),
        ("use axum::{\n    serve,\n};", "router-bypass:serve"),
        ("use axum :: serve;", "router-bypass:serve"),
        ("use axum::*;", "router-bypass:axum-glob"),
        ("use {std::io, axum::*};", "router-bypass:axum-glob"),
        ("use axum::routing::get as g;", "router-bypass:routing"),
        (
            "async fn f(l: L, r: R) { axum :: serve(l, r).await; }",
            "router-bypass:axum::serve",
        ),
        (
            "async fn f(l: L, r: R) { ::axum::\n serve(l, r).await; }",
            "router-bypass:axum::serve",
        ),
        (
            "fn f() { let _ = axum\n::routing::get(h); }",
            "router-bypass:axum::routing",
        ),
        (
            "use ::axum as x;\nfn f() { let _ = x :: routing :: get(h); }",
            "router-bypass:routing",
        ),
    ];
    for (src, want) in cases {
        assert!(has(SRC, src, want), "lolos: {src:?}: {:?}", rules(SRC, src));
    }
    // Impor axum lain dan alias tipe tetap boleh.
    let ok = "use axum::extract::{Path, State};\nuse axum::http::StatusCode as Status;\nuse ::axum::response::IntoResponse;\nuse std::io::{self as sio};";
    let found: Vec<String> = rules(SRC, ok)
        .into_iter()
        .filter(|r| r.starts_with("router-bypass"))
        .collect();
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn failed_access_audit_path_is_limited_to_access_audit_module() {
    // Review keamanan F1-T33 blokir 1: penulisan audit di luar in_audited_tx hanya untuk penolakan akses,
    // dan hanya dari apps/rs-backend/src/access_audit.rs.
    let src = "async fn f(p: P, r: R) { audit::insert_failed_access(&p, &r).await; }";
    for file in [
        "apps/rs-backend/src/modules/orders/service.rs",
        "apps/rs-backend/src/app.rs",
        "crates/authz/src/denial.rs",
    ] {
        assert!(
            has(file, src, "audit-failed-access:insert_failed_access"),
            "{file}: {:?}",
            rules(file, src)
        );
    }
    let ev = "fn f(e: E, c: C) { let _ = e.finish_failed_access(c); }";
    assert!(has(SRC, ev, "audit-failed-access:finish_failed_access"));
    for file in [
        "apps/rs-backend/src/access_audit.rs",
        "crates/audit/src/outbox.rs",
        "crates/audit/tests/outbox_pg.rs",
    ] {
        assert!(
            !rules(file, src)
                .iter()
                .any(|r| r.starts_with("audit-failed-access")),
            "{file}"
        );
    }
}
