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
    if !test {
        out.extend(attribute_findings(file, &sc));
        out.extend(macro_findings(file, &sc));
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
