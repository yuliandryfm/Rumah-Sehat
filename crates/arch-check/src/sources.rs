//! Aturan berbasis teks atas berkas Rust (apps/ dan crates/).
//!
//! Aturan (kolom `rule`):
//! - `attr-allow`: atribut `allow`/`expect` yang membungkam lint larangan (coding-standards §3.4).
//! - `clock`: jam sistem langsung (KEPUTUSAN-131, §5.7). Belt-and-braces di samping clippy.
//! - `env`: pembacaan variabel lingkungan langsung (KEPUTUSAN-132, §6).
//! - `aws-lc`: pemakaian aws-lc-rs di luar crate `crypto` (KEPUTUSAN-74).
//! - `response-code`: pembuatan `ResponseCode` di luar `envelope` dan `error.rs` modul.
//! - `print`: `println!`/`eprintln!`/`dbg!` di kode non-test (log lewat tracing, §4.1).
//! - `log-debug`, `log-field`, `instrument`: pola log terlarang (§4.1, ADR-010 butir 1).
//! - `audit-outbox`: penyebutan `audit_outbox`/`rme_audit_outbox` di luar crate `audit` (ADR-009 butir 2).
//! - `tx-control`: literal SQL kendali transaksi (`COMMIT`, `ROLLBACK`, `BEGIN`, `SAVEPOINT`, ...);
//!   transaksi lewat API sqlx dan transaksi teraudit lewat `audit::in_audited_tx` (F1-T23).
//! - `audit-failed-access`: `audit::insert_failed_access`/`finish_failed_access`/`FailedAccessRecord`
//!   hanya di crate `audit` dan `apps/rs-backend/src/access_audit.rs` (KEPUTUSAN-283).
//! - `router-bypass` (TD-15): di `apps/*/src`, rute hanya lewat `authz::PolicyRouter` dan penyajian hanya
//!   lewat `authz::EnforcedRouter::serve`. Ditolak: router axum/utoipa sendiri (`Router`, `OpenApiRouter`,
//!   `MethodRouter`, `axum::routing`), pemanggilan `.route/.route_service/.nest/.nest_service/.merge`
//!   selain atas `PolicyRouter`, `.fallback/.fallback_service/.method_not_allowed_fallback/.route_layer`,
//!   `axum::serve`/`into_make_service`, serta validasi token di luar middleware penegak
//!   (`authenticate_parts`, `authenticate_headers`, `Authenticator::verify`/`.verify(`).

use crate::lexer::{Scanned, enclosing_fn, find_tokens, is_ident, line_of, matching_close, scan};
use crate::{Finding, fs};
use std::path::Path;

/// Lint yang tidak boleh dibungkam di kode produksi tanpa pengecualian tercatat.
const GUARDED_LINTS: &[&str] = &[
    "clippy::unwrap_used",
    "clippy::expect_used",
    "clippy::panic",
    "clippy::todo",
    "clippy::unimplemented",
    "clippy::indexing_slicing",
    "clippy::dbg_macro",
    "clippy::disallowed_methods",
    "clippy::all",
    "clippy::restriction",
    "warnings",
];

const CLOCK_TOKENS: &[&str] = &[
    "now_utc",
    "now_local",
    "SystemTime::now",
    "UtcDateTime::now",
];
const ENV_TOKENS: &[&str] = &["env::var", "env::var_os", "env::vars", "env::vars_os"];
const PRINT_MACROS: &[&str] = &["println", "print", "eprintln", "eprint", "dbg"];
const LOG_MACROS: &[&str] = &["trace", "debug", "info", "warn", "error", "event"];

/// Segmen nama (dipisah `_`) yang tidak boleh muncul sebagai field atau nilai log (ADR-005, ADR-010).
const LOG_DENY: &[&str] = &[
    "password",
    "passwd",
    "pin",
    "otp",
    "token",
    "secret",
    "authorization",
    "cookie",
    "jwt",
    "nik",
    "email",
    "phone",
    "msisdn",
    "address",
    "alamat",
    "nama",
    "card",
    "kartu",
    "rekening",
    "body",
    "payload",
    "headers",
    "session",
];

fn is_test_path(file: &str) -> bool {
    file.contains("/tests/") || file.contains("/benches/") || file.contains("/examples/")
}

fn is_response_code_site(file: &str) -> bool {
    if file.starts_with("crates/envelope/src/") || is_test_path(file) {
        return true;
    }
    // apps/<app>/src/modules/<modul>/error.rs
    let parts: Vec<&str> = file.split('/').collect();
    matches!(
        parts.as_slice(),
        ["apps", _, "src", "modules", _, "error.rs"]
    )
}

/// Memeriksa semua berkas `.rs` di `apps/` dan `crates/` (temuan sebelum pengecualian).
pub fn check_sources(root: &Path) -> Vec<Finding> {
    let mut out = Vec::new();
    for top in ["apps", "crates"] {
        for p in fs::files_with_ext(&root.join(top), "rs") {
            let file = fs::rel(root, &p);
            if let Ok(src) = std::fs::read_to_string(&p) {
                out.extend(check_source_text(&file, &src));
            }
        }
    }
    out
}

/// Memeriksa satu berkas; `file` adalah path relatif terhadap root (menentukan klasifikasi test dan pemilik).
pub fn check_source_text(file: &str, src: &str) -> Vec<Finding> {
    let sc = scan(src);
    let mut out = Vec::new();
    let test = is_test_path(file);
    let code = sc.code.as_str();

    for tok in CLOCK_TOKENS {
        for off in find_tokens(code, tok) {
            let mut f = Finding::new(
                "clock",
                file,
                line_of(code, off),
                tok,
                format!("jam sistem langsung `{tok}`: pakai trait Clock (KEPUTUSAN-131)"),
            );
            f.func = enclosing_fn(code, off, false);
            out.push(f);
        }
    }
    for tok in ENV_TOKENS {
        for off in find_tokens(code, tok) {
            let mut f = Finding::new(
                "env",
                file,
                line_of(code, off),
                tok,
                format!("`{tok}`: konfigurasi lewat EnvSource/struct bertipe (KEPUTUSAN-132)"),
            );
            f.func = enclosing_fn(code, off, false);
            out.push(f);
        }
    }
    if !file.starts_with("crates/crypto/") {
        for tok in ["aws_lc_rs", "aws_lc_sys"] {
            for off in find_tokens(code, tok) {
                out.push(Finding::new(
                    "aws-lc",
                    file,
                    line_of(code, off),
                    tok,
                    "aws-lc-rs hanya boleh dipakai langsung di crate `crypto` (KEPUTUSAN-74)",
                ));
            }
        }
    }
    if !is_response_code_site(file) {
        for tok in ["ResponseCode::new", "ResponseCode as"] {
            for off in find_tokens(code, tok) {
                out.push(Finding::new(
                    "response-code",
                    file,
                    line_of(code, off),
                    tok,
                    "ResponseCode hanya dibuat di crate envelope dan error.rs modul (coding-standards §3.2)",
                ));
            }
        }
    }
    if !test && is_app_source(file) {
        out.extend(router_bypass_findings(file, &sc));
    }
    if !test && !file.starts_with("crates/audit/") && file != "apps/rs-backend/src/access_audit.rs"
    {
        for tok in [
            "insert_failed_access",
            "finish_failed_access",
            "FailedAccessRecord",
        ] {
            for off in find_tokens(code, tok) {
                out.push(Finding::new(
                    "audit-failed-access",
                    file,
                    line_of(code, off),
                    tok,
                    "penulisan audit di luar in_audited_tx hanya untuk penolakan akses, dari \
                     apps/rs-backend/src/access_audit.rs (KEPUTUSAN-283, review keamanan F1-T33)",
                ));
            }
        }
    }
    if !test {
        out.extend(idp_admin_client_findings(file, code));
    }
    if !test {
        out.extend(attribute_findings(file, &sc));
        out.extend(macro_findings(file, &sc));
        if !file.starts_with("crates/arch-check/") {
            out.extend(audit_outbox_findings(file, &sc));
            out.extend(tx_control_findings(file, &sc));
        }
    }
    out
}

/// Token klien pengguna Admin REST Keycloak ber-`manage-users` milik worker IdP (ADR-030 butir 1, langkah 3).
const IDP_ADMIN_CLIENT_TOKENS: &[&str] = &["IdpUserAdmin", "IdpWorker", "load_idp_credentials"];

/// Berkas modul worker IdP yang boleh memakai token di atas.
const IDP_ADMIN_CLIENT_FILES: &[&str] = &[
    "apps/rs-backend/src/modules/identity/idp_worker.rs",
    "apps/rs-backend/src/modules/identity/idp_worker_client.rs",
    "apps/rs-backend/src/modules/identity/idp_config.rs",
    "apps/rs-backend/src/modules/identity/api.rs",
];

/// Fungsi di `cli.rs` yang merakit worker IdP (subcommand `worker`).
const IDP_ADMIN_CLIENT_CLI_FNS: &[&str] = &["idp_worker", "run_worker"];

/// Pemanggil `admin_call` (permintaan Admin REST mentah) yang sah.
const ADMIN_CALL_FILES: &[&str] = &[
    "apps/rs-backend/src/modules/identity/keycloak.rs",
    "apps/rs-backend/src/modules/identity/idp_worker_client.rs",
];

/// ADR-030 langkah 3: klien pengguna Admin REST worker IdP (dan pemuat rahasianya dari OpenBao) hanya dirakit di
/// modul worker IdP dan fungsi subcommand `worker` di `cli.rs`; proses `api` tidak pernah memegangnya.
/// Permintaan Admin REST mentah (`admin_call`) hanya dari klien Keycloak dan klien worker IdP.
fn idp_admin_client_findings(file: &str, code: &str) -> Vec<Finding> {
    let mut out = Vec::new();
    if file.starts_with("crates/arch-check/") {
        return out;
    }
    if !IDP_ADMIN_CLIENT_FILES.contains(&file) {
        for tok in IDP_ADMIN_CLIENT_TOKENS {
            for off in find_tokens(code, tok) {
                let func = enclosing_fn(code, off, false);
                if file == "apps/rs-backend/src/cli.rs"
                    && func
                        .as_deref()
                        .is_some_and(|f| IDP_ADMIN_CLIENT_CLI_FNS.contains(&f))
                {
                    continue;
                }
                let mut f = Finding::new(
                    "idp-admin-client",
                    file,
                    line_of(code, off),
                    tok,
                    format!(
                        "`{tok}` (klien manage-users Keycloak) hanya di modul worker IdP dan subcommand \
                         worker (ADR-030)"
                    ),
                );
                f.func = func;
                out.push(f);
            }
        }
    }
    if !ADMIN_CALL_FILES.contains(&file) {
        for off in find_tokens(code, "admin_call") {
            out.push(Finding::new(
                "idp-admin-client",
                file,
                line_of(code, off),
                "admin_call",
                "permintaan Admin REST mentah hanya dari klien Keycloak dan klien worker IdP (ADR-030)",
            ));
        }
    }
    out
}

/// `apps/<app>/src/...`: kode aplikasi (bukan test, bukan crate pustaka).
fn is_app_source(file: &str) -> bool {
    let parts: Vec<&str> = file.split('/').collect();
    matches!(parts.as_slice(), ["apps", _, "src", ..])
}

/// Token yang tidak boleh muncul di `apps/*/src` (TD-15).
const ROUTER_BYPASS_TOKENS: &[&str] = &[
    "Router",
    "OpenApiRouter",
    "MethodRouter",
    "axum::Router",
    "axum::serve",
    "axum::routing",
    "into_make_service",
    "into_make_service_with_connect_info",
    "authenticate_parts",
    "authenticate_headers",
    "Authenticator::verify",
];

/// Metode yang tidak boleh dipanggil (`.nama(` atau `.nama::<`) di `apps/*/src` (TD-15). `merge`
/// diperiksa terpisah karena `PolicyRouter::merge` sah.
const ROUTER_BYPASS_METHODS: &[&str] = &[
    "route",
    "route_service",
    "nest",
    "nest_service",
    "fallback",
    "fallback_service",
    "method_not_allowed_fallback",
    "route_layer",
    "verify",
];

fn prev_non_ws(b: &[u8], off: usize) -> Option<u8> {
    b.get(..off)?
        .iter()
        .rev()
        .find(|c| !c.is_ascii_whitespace())
        .copied()
}

fn next_non_ws(b: &[u8], off: usize) -> Option<u8> {
    b.get(off..)?
        .iter()
        .find(|c| !c.is_ascii_whitespace())
        .copied()
}

/// Offset pemanggilan metode `.name(` / `.name::<` di `code`.
fn method_calls(code: &str, name: &str) -> Vec<usize> {
    let b = code.as_bytes();
    find_tokens(code, name)
        .into_iter()
        .filter(|&off| {
            prev_non_ws(b, off) == Some(b'.')
                && matches!(next_non_ws(b, off + name.len()), Some(b'(' | b':'))
        })
        .collect()
}

/// Benar bila rantai pemanggilan yang berakhir di offset `dot_end` (posisi metode) berawal dari
/// `PolicyRouter` atau `router()` modul (fungsi yang mengembalikan `PolicyRouter`). Pemeriksaan teks:
/// mundur melewati rantai `.x(...)` sampai awal ekspresi.
fn chain_starts_with_policy_router(code: &str, method_off: usize) -> bool {
    let start = code
        .get(..method_off)
        .and_then(|h| h.rfind([';', '{', '}', '=']))
        .map_or(0, |p| p + 1);
    let head = code.get(start..method_off).unwrap_or("");
    !find_tokens(head, "PolicyRouter").is_empty() || !find_tokens(head, "router").is_empty()
}

/// `router-bypass` (TD-15): rute yang ditambahkan di luar `PolicyRouter`, atau router yang dilayani tanpa
/// `EnforcedRouter`, tidak dibungkus middleware penegak (default deny ADR-012 butir 1, KEPUTUSAN-127).
fn router_bypass_findings(file: &str, sc: &Scanned) -> Vec<Finding> {
    let code = sc.code.as_str();
    let mut out = Vec::new();
    let msg = "rute hanya lewat authz::PolicyRouter dan penyajian lewat EnforcedRouter::serve; \
               validasi token hanya di middleware penegak (TD-15, KEPUTUSAN-127)";
    for tok in ROUTER_BYPASS_TOKENS {
        for off in find_tokens(code, tok) {
            out.push(Finding::new(
                "router-bypass",
                file,
                line_of(code, off),
                tok,
                msg,
            ));
        }
    }
    for name in ROUTER_BYPASS_METHODS {
        for off in method_calls(code, name) {
            out.push(Finding::new(
                "router-bypass",
                file,
                line_of(code, off),
                name,
                msg,
            ));
        }
    }
    for off in method_calls(code, "merge") {
        if !chain_starts_with_policy_router(code, off) {
            out.push(Finding::new(
                "router-bypass",
                file,
                line_of(code, off),
                "merge",
                msg,
            ));
        }
    }
    // Impor dari crate axum dalam bentuk apa pun (review keamanan F1-T33 blokir 2): pohon `use`
    // dinormalisasi (spasi dan baris baru, `::` di depan, grup `{}` bertingkat, `pub use`,
    // `extern crate`). Impor `serve`, `routing`, `Router`, glob `*`, dan alias crate ditolak, sehingga
    // pemanggilan tanpa prefiks (`serve(..)`, `get(..)`) tidak mungkin.
    for (off, stmt) in import_statements(code) {
        if find_tokens(&stmt, "axum").is_empty() {
            continue;
        }
        let line = line_of(code, off);
        let mut push = |detail: &str| {
            out.push(Finding::new("router-bypass", file, line, detail, msg));
        };
        for tok in [
            "serve",
            "routing",
            "Router",
            "MethodRouter",
            "into_make_service",
        ] {
            if !find_tokens(&stmt, tok).is_empty() {
                push(tok);
            }
        }
        if stmt.contains('*') {
            push("axum-glob");
        }
        for alias in aliases(&stmt) {
            push("axum-alias");
            for item in ["serve", "routing", "Router"] {
                let tok = format!("{alias}::{item}");
                if !find_tokens(&compact(code).0, &tok).is_empty() {
                    push(item);
                }
            }
        }
    }
    // Path lengkap yang dipisah spasi atau baris baru (`axum :: serve`, `::axum::routing`).
    let (packed, map) = compact(code);
    for pat in ["axum::serve", "axum::routing", "axum::Router"] {
        for at in find_tokens(&packed, pat) {
            let orig = map.get(at).copied().unwrap_or(0);
            out.push(Finding::new(
                "router-bypass",
                file,
                line_of(code, orig),
                pat,
                msg,
            ));
        }
    }
    out
}

/// Kode tanpa whitespace beserta peta offset ke kode asli.
fn compact(code: &str) -> (String, Vec<usize>) {
    let mut packed = String::with_capacity(code.len());
    let mut map = Vec::with_capacity(code.len());
    for (i, c) in code.char_indices() {
        if !c.is_whitespace() {
            packed.push(c);
            for _ in 0..c.len_utf8() {
                map.push(i);
            }
        }
    }
    (packed, map)
}

/// Pernyataan `use ...;` dan `extern crate ...;`: (offset, teks dinormalisasi). Whitespace dirapatkan
/// menjadi satu spasi lalu dihapus di sekitar `::`, `{`, `}`, dan `,`; `::` di awal path dibuang.
fn import_statements(code: &str) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    let mut starts: Vec<usize> = find_tokens(code, "use");
    for off in find_tokens(code, "extern") {
        let rest = code.get(off + "extern".len()..).unwrap_or("");
        if rest.trim_start().starts_with("crate") {
            starts.push(off);
        }
    }
    for off in starts {
        let end = code
            .get(off..)
            .and_then(|s| s.find(';'))
            .map_or(code.len(), |p| off + p);
        let raw = code.get(off..end).unwrap_or("");
        let mut norm = raw.split_whitespace().collect::<Vec<_>>().join(" ");
        for sep in ["::", "{", "}", ","] {
            norm = norm
                .replace(&format!(" {sep}"), sep)
                .replace(&format!("{sep} "), sep);
        }
        // `use ::axum` / `{::axum` → path tanpa `::` depan.
        norm = norm
            .replace("use::", "use ")
            .replace("{::", "{")
            .replace(",::", ",");
        out.push((off, norm));
    }
    out
}

/// Nama alias yang mengganti nama crate axum di satu pernyataan impor (`axum as x`, `self as x`).
fn aliases(stmt: &str) -> Vec<String> {
    let mut out = Vec::new();
    for marker in ["axum as ", "self as "] {
        for (pos, _) in stmt.match_indices(marker) {
            let before_ok = stmt
                .get(..pos)
                .and_then(|h| h.chars().last())
                .is_none_or(|c| !(c.is_ascii_alphanumeric() || c == '_'));
            if !before_ok {
                continue;
            }
            let name: String = stmt
                .get(pos + marker.len()..)
                .unwrap_or("")
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect();
            if !name.is_empty() {
                out.push(name);
            }
        }
    }
    out
}

/// Nama tabel outbox audit, dirakit saat runtime supaya berkas pemeriksa ini tidak memuat literalnya.
fn outbox_table_names() -> [String; 2] {
    let base = ["audit", "outbox"].join("_");
    [base.clone(), format!("rme_{base}")]
}

/// `audit-outbox` (ADR-009 butir 2, review keamanan F1-T23 BLOKIR-2): tabel `audit_outbox` dan
/// `rme_audit_outbox` hanya boleh disebut di crate `audit`. Melengkapi aturan kepemilikan tabel
/// (`table-owner`, hanya literal berbentuk SQL) dengan menolak penyebutan apa pun: literal yang dirakit
/// menjadi SQL dinamis, konstanta nama tabel, atau identifier.
fn audit_outbox_findings(file: &str, sc: &Scanned) -> Vec<Finding> {
    if file.starts_with("crates/audit/") {
        return Vec::new();
    }
    let mut out = Vec::new();
    let names = outbox_table_names();
    for (off, lit) in &sc.literals {
        let lower = lit.to_ascii_lowercase();
        if let Some(name) = names.iter().find(|n| lower.contains(n.as_str())) {
            out.push(Finding::new(
                "audit-outbox",
                file,
                line_of(&sc.code, *off),
                name,
                "tabel outbox audit hanya ditulis lewat crate audit (audit::in_audited_tx, ADR-009 butir 2)",
            ));
        }
    }
    for name in &names {
        for off in find_tokens(&sc.code, name) {
            out.push(Finding::new(
                "audit-outbox",
                file,
                line_of(&sc.code, off),
                name,
                "tabel outbox audit hanya ditulis lewat crate audit (audit::in_audited_tx, ADR-009 butir 2)",
            ));
        }
    }
    out
}

/// Benar bila `stmt` berbentuk pernyataan SQL kendali transaksi PostgreSQL. Bentuk prosa
/// (`commit message`, `release notes`) tidak dianggap pernyataan.
fn is_tx_control_statement(stmt: &str) -> bool {
    let words: Vec<String> = stmt
        .split_whitespace()
        .map(|w| w.to_ascii_lowercase())
        .collect();
    let first = words.first().map(String::as_str);
    let second = words.get(1).map(String::as_str);
    let alone = words.len() == 1;
    match first {
        Some("commit") | Some("rollback") | Some("abort") | Some("end") => {
            alone
                || matches!(
                    second,
                    Some("transaction" | "work" | "and" | "to" | "prepared")
                )
        }
        Some("begin") => {
            alone
                || matches!(
                    second,
                    Some("transaction" | "work" | "isolation" | "read" | "deferrable" | "not")
                )
        }
        Some("savepoint") => words.len() == 2,
        Some("release") => second == Some("savepoint"),
        Some("start" | "set" | "prepare") => second == Some("transaction"),
        _ => false,
    }
}

/// `tx-control` (review keamanan F1-T23 BLOKIR-2): literal SQL kendali transaksi (`COMMIT`, `ROLLBACK`,
/// `BEGIN`, `SAVEPOINT`, `RELEASE`, `START TRANSACTION`, ...) ditolak di kode non-test. Transaksi hanya
/// lewat API sqlx (`begin`/`commit`), dan transaksi yang mencatat audit hanya lewat
/// `audit::in_audited_tx`; SQL mentah dapat mengakhiri transaksi teraudit dari dalam aksi bisnis.
fn tx_control_findings(file: &str, sc: &Scanned) -> Vec<Finding> {
    let mut out = Vec::new();
    for (off, lit) in &sc.literals {
        for stmt in lit.split(";") {
            if is_tx_control_statement(stmt) {
                let word = stmt
                    .split_whitespace()
                    .next()
                    .unwrap_or_default()
                    .to_ascii_lowercase();
                out.push(Finding::new(
                    "tx-control",
                    file,
                    line_of(&sc.code, *off),
                    &word,
                    "SQL kendali transaksi mentah dilarang; pakai API transaksi sqlx atau audit::in_audited_tx",
                ));
                break;
            }
        }
    }
    out
}

/// `#[...]` / `#![...]`: teks atribut beserta offset awal dan akhir.
fn attributes(code: &str) -> Vec<(usize, usize)> {
    let b = code.as_bytes();
    let mut v = Vec::new();
    for (i, &c) in b.iter().enumerate() {
        if c != b'#' {
            continue;
        }
        let mut k = i + 1;
        if b.get(k) == Some(&b'!') {
            k += 1;
        }
        while b.get(k).is_some_and(u8::is_ascii_whitespace) {
            k += 1;
        }
        if b.get(k) == Some(&b'[')
            && let Some(end) = matching_close(code, k)
        {
            v.push((k, end));
        }
    }
    v
}

fn attribute_findings(file: &str, sc: &Scanned) -> Vec<Finding> {
    let code = sc.code.as_str();
    let mut out = Vec::new();
    for (s, e) in attributes(code) {
        let Some(text) = code.get(s..=e) else {
            continue;
        };
        let mutes =
            !find_tokens(text, "allow").is_empty() || !find_tokens(text, "expect").is_empty();
        if mutes {
            for lint in GUARDED_LINTS {
                if !find_tokens(text, lint).is_empty() {
                    let mut f = Finding::new(
                        "attr-allow",
                        file,
                        line_of(code, s),
                        lint,
                        format!(
                            "atribut membungkam `{lint}` di kode non-test; catat di architecture-exceptions.toml"
                        ),
                    );
                    f.func = enclosing_fn(code, e, true);
                    out.push(f);
                }
            }
        }
        if !find_tokens(text, "instrument").is_empty() && find_tokens(text, "skip_all").is_empty() {
            out.push(Finding::new(
                "instrument",
                file,
                line_of(code, s),
                "instrument",
                "#[instrument] tanpa skip_all mencatat semua argumen lewat Debug (ADR-010 butir 1)",
            ));
        }
    }
    out
}

/// Pemanggilan makro `name!(...)`: (nama, offset awal, rentang isi argumen).
fn macro_calls(code: &str, names: &[&str]) -> Vec<(String, usize, usize, usize)> {
    let b = code.as_bytes();
    let mut v = Vec::new();
    for name in names {
        for off in find_tokens(code, name) {
            let mut k = off + name.len();
            while b.get(k).is_some_and(u8::is_ascii_whitespace) {
                k += 1;
            }
            if b.get(k) != Some(&b'!') {
                continue;
            }
            k += 1;
            while b.get(k).is_some_and(u8::is_ascii_whitespace) {
                k += 1;
            }
            if matches!(b.get(k), Some(b'(' | b'[' | b'{'))
                && let Some(end) = matching_close(code, k)
            {
                v.push(((*name).to_string(), off, k + 1, end));
            }
        }
    }
    v
}

fn macro_findings(file: &str, sc: &Scanned) -> Vec<Finding> {
    let code = sc.code.as_str();
    let mut out = Vec::new();
    for (name, off, _, _) in macro_calls(code, PRINT_MACROS) {
        out.push(Finding::new(
            "print",
            file,
            line_of(code, off),
            &name,
            format!("`{name}!` di kode non-test: log lewat tracing, bukan stdout/stderr (§4.1)"),
        ));
    }
    if !code.contains("tracing") {
        return out;
    }
    for (name, off, s, e) in macro_calls(code, LOG_MACROS) {
        let args = code.get(s..e).unwrap_or("");
        let line = line_of(code, off);
        if has_debug_sigil(args) {
            out.push(Finding::new(
                "log-debug",
                file,
                line,
                &name,
                "field log berformat Debug (`?x`): objek utuh dilarang, catat ID satu per satu (ADR-010 butir 1)",
            ));
        }
        for ident in denied_idents(args) {
            out.push(Finding::new(
                "log-field",
                file,
                line,
                &ident,
                format!("`{ident}` memuat nama data sensitif/terlarang di log (ADR-005, ADR-010)"),
            ));
        }
        for (lo, text) in &sc.literals {
            if *lo < s || *lo >= e {
                continue;
            }
            for (ident, dbg) in inline_args(text) {
                if dbg {
                    out.push(Finding::new(
                        "log-debug",
                        file,
                        line,
                        &name,
                        format!("format Debug `{{{ident}:?}}` di pesan log (ADR-010 butir 1)"),
                    ));
                }
                if is_denied(&ident) {
                    out.push(Finding::new(
                        "log-field",
                        file,
                        line,
                        &ident,
                        format!("`{ident}` di pesan log memuat nama data sensitif/terlarang"),
                    ));
                }
            }
        }
    }
    out
}

fn has_debug_sigil(args: &str) -> bool {
    let b = args.as_bytes();
    b.iter().enumerate().any(|(i, &c)| {
        c == b'?'
            && b.get(..i)
                .and_then(|h| h.iter().rev().find(|x| !x.is_ascii_whitespace()))
                .is_none_or(|p| matches!(p, b'=' | b',' | b'('))
    })
}

fn is_denied(ident: &str) -> bool {
    ident
        .to_ascii_lowercase()
        .split('_')
        .any(|seg| LOG_DENY.contains(&seg))
}

fn denied_idents(args: &str) -> Vec<String> {
    let b = args.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < b.len() {
        if is_ident(b.get(i).copied().unwrap_or(0)) {
            let s = i;
            while i < b.len() && is_ident(b.get(i).copied().unwrap_or(0)) {
                i += 1;
            }
            let id = args.get(s..i).unwrap_or("");
            if !id.starts_with(|c: char| c.is_ascii_digit()) && is_denied(id) {
                out.push(id.to_string());
            }
        } else {
            i += 1;
        }
    }
    out
}

/// Argumen inline `{ident}` / `{ident:?}` dalam literal format; `{{` diabaikan.
fn inline_args(lit: &str) -> Vec<(String, bool)> {
    let mut out = Vec::new();
    let mut chars = lit.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '{' {
            continue;
        }
        if chars.peek() == Some(&'{') {
            chars.next();
            continue;
        }
        let mut inner = String::new();
        for d in chars.by_ref() {
            if d == '}' {
                break;
            }
            inner.push(d);
        }
        let (name, spec) = inner.split_once(':').unwrap_or((inner.as_str(), ""));
        out.push((name.trim().to_string(), spec.contains('?')));
    }
    out
}
