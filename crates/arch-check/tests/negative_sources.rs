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
