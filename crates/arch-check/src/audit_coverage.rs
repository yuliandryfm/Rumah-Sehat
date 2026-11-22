//! Inventaris pemancar event audit (F1-T69, US-1501 AC1–AC2; aturan `audit-coverage`).
//!
//! Setiap berkas kode non-test di `apps/*/src` dan `crates/*/src` (di luar crate `audit` dan `arch-check`)
//! yang menyebut kode event (`EventCode::E01` … `EventCode::E18`) atau menyusun event (`AuditEvent::new`)
//! adalah pemancar. `audit-event-coverage.toml` di root mencatat setiap pemancar beserta kode event yang
//! disebutnya dan test integrasi yang memeriksa payload tersimpan dengan pemeriksa kelengkapan
//! (`audit::check_payload`, PRD §7.2–§7.3). Temuan:
//! - pemancar yang tidak terdaftar, atau kode event yang disebut tetapi tidak terdaftar (event baru tanpa
//!   cakupan test);
//! - entri usang (berkas tidak lagi memancar, atau kode terdaftar yang tidak lagi disebut);
//! - test yang dirujuk tidak ada, atau badannya tidak memanggil salah satu pemeriksa `checkers`;
//! - pemeriksa di `checkers` yang tidak didefinisikan di `apps/*/tests` dengan badan yang memanggil
//!   `check_payload` (atau pemeriksa baku `assert_event_complete`).
//!
//! Blok `#[cfg(test)]` di berkas kode dikeluarkan sebelum pencarian. Batas yang diketahui: pemeriksa ini
//! membuktikan bahwa setiap pemancar punya test yang memanggil pemeriksa kelengkapan, bukan bahwa test itu
//! menjalankan setiap cabang pemancar; kelengkapan per cabang ditegakkan saat runtime oleh
//! `AuditEvent::finish` (event kurang field ditolak sebelum ditulis).

use crate::lexer::{find_tokens, matching_close, scan};
use crate::{Finding, fs};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// Nama berkas registri di root repositori.
pub const REGISTRY: &str = "audit-event-coverage.toml";
const RULE: &str = "audit-coverage";
/// Pemeriksa baku (`apps/*/tests/audit_support`).
const BASE_CHECKER: &str = "assert_event_complete";

/// Isi `audit-event-coverage.toml`.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Registry {
    /// Nama fungsi test yang memeriksa payload dengan `audit::check_payload`.
    pub checkers: Vec<String>,
    /// Pemancar event.
    #[serde(default, rename = "emitter")]
    pub emitters: Vec<Emitter>,
}

/// Satu pemancar.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Emitter {
    /// Path relatif root.
    pub file: String,
    /// Kode event yang disebut berkas (`E01` … `E18`).
    pub codes: Vec<String>,
    /// Test yang mencakup: `apps/<app>/tests/<berkas>.rs::<nama_fn>`.
    pub tests: Vec<String>,
}

/// Mengurai isi registri.
pub fn parse(text: &str) -> Result<Registry, String> {
    toml::from_str(text).map_err(|e| format!("{REGISTRY} tidak valid: {e}"))
}

fn finding(file: &str, detail: &str, msg: impl Into<String>) -> Finding {
    Finding::new(RULE, file, 0, detail, msg)
}

fn is_inventoried_source(file: &str) -> bool {
    let parts: Vec<&str> = file.split('/').collect();
    let src = matches!(
        parts.as_slice(),
        ["apps", _, "src", ..] | ["crates", _, "src", ..]
    );
    src && !file.starts_with("crates/audit/") && !file.starts_with("crates/arch-check/")
}

/// Memeriksa registri terhadap repositori di `root`.
pub fn check(root: &Path) -> Vec<Finding> {
    let text = match std::fs::read_to_string(root.join(REGISTRY)) {
        Ok(t) => t,
        Err(_) => {
            return vec![finding(
                REGISTRY,
                "missing",
                "registri pemancar audit tidak ada",
            )];
        }
    };
    let registry = match parse(&text) {
        Ok(r) => r,
        Err(e) => return vec![finding(REGISTRY, "parse", e)],
    };
    let mut sources = BTreeMap::new();
    let mut tests = BTreeMap::new();
    for top in ["apps", "crates"] {
        for p in fs::files_with_ext(&root.join(top), "rs") {
            let file = fs::rel(root, &p);
            let Ok(src) = std::fs::read_to_string(&p) else {
                continue;
            };
            if is_inventoried_source(&file) {
                sources.insert(file, src);
            } else if file.starts_with("apps/") && file.contains("/tests/") {
                tests.insert(file, src);
            }
        }
    }
    check_inventory(&registry, &sources, &tests)
}

/// Inti pemeriksaan (murni): `sources` = berkas kode non-test, `tests` = berkas test `apps/*/tests`.
pub fn check_inventory(
    registry: &Registry,
    sources: &BTreeMap<String, String>,
    tests: &BTreeMap<String, String>,
) -> Vec<Finding> {
    let mut out = Vec::new();
    let found: BTreeMap<&str, BTreeSet<String>> = sources
        .iter()
        .filter_map(|(file, src)| emitted_codes(src).map(|codes| (file.as_str(), codes)))
        .collect();
    let mut registered: BTreeMap<&str, &Emitter> = BTreeMap::new();
    for e in &registry.emitters {
        if registered.insert(e.file.as_str(), e).is_some() {
            out.push(finding(&e.file, "duplicate", "pemancar terdaftar dua kali"));
        }
    }

    for (file, codes) in &found {
        let Some(entry) = registered.get(file) else {
            out.push(finding(
                file,
                "unregistered",
                format!(
                    "pemancar event audit {codes:?} tanpa entri di {REGISTRY} (test kelengkapan field \
                     PRD §7.2 wajib, US-1501 AC1)"
                ),
            ));
            continue;
        };
        let listed: BTreeSet<String> = entry.codes.iter().cloned().collect();
        for c in codes.difference(&listed) {
            out.push(finding(
                file,
                c,
                format!("kode event {c} dipancarkan tetapi tidak terdaftar di {REGISTRY}"),
            ));
        }
        for c in listed.difference(codes) {
            out.push(finding(
                file,
                c,
                format!("kode event {c} terdaftar tetapi tidak lagi disebut berkas (entri usang)"),
            ));
        }
    }
    for e in &registry.emitters {
        if !found.contains_key(e.file.as_str()) {
            out.push(finding(
                &e.file,
                "stale",
                "entri usang: berkas tidak ada atau tidak lagi memancarkan event audit",
            ));
        }
        if e.tests.is_empty() {
            out.push(finding(&e.file, "tests", "pemancar tanpa test cakupan"));
        }
        for t in &e.tests {
            if let Err(msg) = test_calls_checker(t, &registry.checkers, tests) {
                out.push(finding(&e.file, t, msg));
            }
        }
    }
    if registry.checkers.is_empty() {
        out.push(finding(REGISTRY, "checkers", "daftar pemeriksa kosong"));
    }
    for c in &registry.checkers {
        if !checker_is_defined(c, tests) {
            out.push(finding(
                REGISTRY,
                c,
                format!(
                    "pemeriksa `{c}` tidak didefinisikan di apps/*/tests dengan badan yang memanggil \
                     check_payload atau {BASE_CHECKER}"
                ),
            ));
        }
    }
    out
}

/// Kode event yang disebut berkas (di luar blok `#[cfg(test)]`), atau `None` bila bukan pemancar.
pub fn emitted_codes(src: &str) -> Option<BTreeSet<String>> {
    let code = strip_cfg_test(&scan(src).code);
    let mut codes = BTreeSet::new();
    for off in find_tokens(&code, "EventCode") {
        let rest = code.get(off + "EventCode".len()..).unwrap_or("");
        let rest = rest.trim_start();
        let Some(after) = rest.strip_prefix("::") else {
            continue;
        };
        let after = after.trim_start();
        let b = after.as_bytes();
        let is_code = b.first() == Some(&b'E')
            && b.get(1).is_some_and(u8::is_ascii_digit)
            && b.get(2).is_some_and(u8::is_ascii_digit)
            && !b
                .get(3)
                .is_some_and(|c| c.is_ascii_alphanumeric() || *c == b'_');
        if is_code && let Some(c) = after.get(..3) {
            codes.insert(c.to_string());
        }
    }
    let builds = !find_tokens(&code, "AuditEvent::new").is_empty();
    (builds || !codes.is_empty()).then_some(codes)
}

/// Mengosongkan item yang diberi `#[cfg(test)]` (modul/fungsi berbadan `{}` atau item diakhiri `;`).
fn strip_cfg_test(code: &str) -> String {
    let mut out = code.as_bytes().to_vec();
    for off in find_tokens(code, "#[cfg(test)]") {
        let start = off;
        let after = off + "#[cfg(test)]".len();
        let rest = code.get(after..).unwrap_or("");
        let brace = rest.find('{').map(|p| after + p);
        let semi = rest.find(';').map(|p| after + p);
        let end = match (brace, semi) {
            (Some(b), Some(s)) if s < b => Some(s),
            (Some(b), _) => matching_close(code, b),
            (None, Some(s)) => Some(s),
            (None, None) => None,
        };
        let end = end.unwrap_or(code.len().saturating_sub(1));
        for c in out.iter_mut().take(end + 1).skip(start) {
            if *c != b'\n' {
                *c = b' ';
            }
        }
    }
    String::from_utf8(out).unwrap_or_default()
}

/// Badan fungsi `name` di `code` (sudah dipindai), bila ada.
fn fn_body<'a>(code: &'a str, name: &str) -> Option<&'a str> {
    let needle = format!("fn {name}");
    let off = *find_tokens(code, &needle).first()?;
    let open = off + code.get(off..)?.find('{')?;
    let close = matching_close(code, open)?;
    code.get(open..=close)
}

fn test_calls_checker(
    reference: &str,
    checkers: &[String],
    tests: &BTreeMap<String, String>,
) -> Result<(), String> {
    let (file, name) = reference
        .split_once("::")
        .ok_or_else(|| format!("rujukan test `{reference}` bukan `<berkas>::<fn>`"))?;
    let src = tests
        .get(file)
        .ok_or_else(|| format!("berkas test `{file}` tidak ada"))?;
    let code = scan(src).code;
    let body = fn_body(&code, name).ok_or_else(|| format!("test `{name}` tidak ada di {file}"))?;
    if checkers
        .iter()
        .any(|c| !find_tokens(body, c.as_str()).is_empty())
    {
        Ok(())
    } else {
        Err(format!(
            "test `{name}` tidak memanggil pemeriksa kelengkapan event ({})",
            checkers.join(", ")
        ))
    }
}

fn checker_is_defined(name: &str, tests: &BTreeMap<String, String>) -> bool {
    tests.values().any(|src| {
        let code = scan(src).code;
        fn_body(&code, name).is_some_and(|body| {
            !find_tokens(body, "check_payload").is_empty()
                || (name != BASE_CHECKER && !find_tokens(body, BASE_CHECKER).is_empty())
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_and_builders_are_found_outside_test_blocks() {
        let src = "use audit::EventCode;\nfn a() { let _ = EventCode::E07; let _ = EventCode :: E10; }\n\
                   // EventCode::E99 di komentar\nfn s() -> &'static str { \"EventCode::E12\" }\n\
                   #[cfg(test)]\nmod tests { fn t() { let _ = EventCode::E16; } }\n";
        let codes = emitted_codes(src).unwrap();
        assert_eq!(codes.into_iter().collect::<Vec<_>>(), ["E07", "E10"]);
        assert!(emitted_codes("fn b() { AuditEvent::new(code, a, b, c); }").is_some());
        assert!(emitted_codes("fn c() { let _ = EventCode::ALL; }").is_none());
        assert!(emitted_codes("#[cfg(test)]\nuse audit::EventCode;\nfn d() {}").is_none());
    }
}
