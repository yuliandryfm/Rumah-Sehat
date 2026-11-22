//! Uji negatif aturan `audit-coverage` (F1-T69, US-1501 AC1): pemancar event audit baru tanpa entri
//! registri, kode event baru, entri usang, dan test yang tidak memanggil pemeriksa kelengkapan ditolak.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::indexing_slicing)] // berkas test

use arch_check::audit_coverage::{check, check_inventory, parse};
use std::collections::BTreeMap;
use std::path::Path;

const EMITTER: &str = "apps/x/src/modules/m/service.rs";
const TEST_FILE: &str = "apps/x/tests/m.rs";

const SRC: &str = "fn act() { let e = AuditEvent::new(EventCode::E10, a, b, c); }\n";
const TESTS: &str = "mod audit_support;\n\
    #[tokio::test]\nasync fn covered() { let p = read().await; audit_support::assert_event_complete(&p, \"e_10\"); }\n\
    #[tokio::test]\nasync fn not_checking() { let p = read().await; assert_eq!(p[\"event_code\"], \"e_10\"); }\n";
const SUPPORT: &str = "pub fn assert_event_complete(p: &Value, c: &str) { assert!(audit::check_payload(p).is_empty()); }\n";

fn registry(extra: &str) -> String {
    format!(
        "checkers = [\"assert_event_complete\"]\n\
         [[emitter]]\nfile = \"{EMITTER}\"\ncodes = [\"E10\"]\ntests = [\"{TEST_FILE}::covered\"]\n{extra}"
    )
}

fn maps(src: &str) -> (BTreeMap<String, String>, BTreeMap<String, String>) {
    let mut sources = BTreeMap::new();
    sources.insert(EMITTER.to_string(), src.to_string());
    sources.insert(
        "apps/x/src/modules/m/repo.rs".to_string(),
        "fn q() {}".to_string(),
    );
    let mut tests = BTreeMap::new();
    tests.insert(TEST_FILE.to_string(), TESTS.to_string());
    tests.insert(
        "apps/x/tests/audit_support/mod.rs".to_string(),
        SUPPORT.to_string(),
    );
    (sources, tests)
}

fn details(reg: &str, src: &str) -> Vec<String> {
    let (s, t) = maps(src);
    check_inventory(&parse(reg).unwrap(), &s, &t)
        .into_iter()
        .map(|f| {
            assert_eq!(f.rule, "audit-coverage");
            f.detail
        })
        .collect()
}

#[test]
fn registered_and_covered_emitter_passes() {
    assert_eq!(details(&registry(""), SRC), Vec::<String>::new());
}

#[test]
fn new_emitter_without_entry_is_rejected() {
    let (mut s, t) = maps(SRC);
    s.insert(
        "apps/x/src/modules/n/service.rs".to_string(),
        "fn b() { audit::AuditEvent::new(EventCode::E15, a, b, c); }".to_string(),
    );
    let f = check_inventory(&parse(&registry("")).unwrap(), &s, &t);
    assert_eq!(f.len(), 1, "{f:?}");
    assert_eq!(f[0].file, "apps/x/src/modules/n/service.rs");
    assert_eq!(f[0].detail, "unregistered");
}

#[test]
fn new_event_code_in_registered_emitter_is_rejected() {
    let src = format!("{SRC}fn other() {{ let c = EventCode::E16; }}\n");
    assert_eq!(details(&registry(""), &src), ["E16"]);
}

#[test]
fn stale_entries_and_codes_are_rejected() {
    let reg = registry(&format!(
        "[[emitter]]\nfile = \"apps/x/src/gone.rs\"\ncodes = [\"E05\"]\ntests = [\"{TEST_FILE}::covered\"]\n"
    ));
    assert_eq!(details(&reg, SRC), ["stale"]);
    let reg = registry("").replace("[\"E10\"]", "[\"E10\", \"E11\"]");
    assert_eq!(details(&reg, SRC), ["E11"]);
}

#[test]
fn tests_must_exist_and_call_a_checker() {
    let reg = registry("").replace("::covered", "::not_checking");
    assert_eq!(details(&reg, SRC), [format!("{TEST_FILE}::not_checking")]);
    let reg = registry("").replace("::covered", "::missing");
    assert_eq!(details(&reg, SRC), [format!("{TEST_FILE}::missing")]);
    let reg = registry("").replace(TEST_FILE, "apps/x/tests/nope.rs");
    assert_eq!(details(&reg, SRC), ["apps/x/tests/nope.rs::covered"]);
    let reg = registry("").replace("tests = [\"apps/x/tests/m.rs::covered\"]", "tests = []");
    assert_eq!(details(&reg, SRC), ["tests"]);
}

#[test]
fn checkers_must_reach_check_payload() {
    let reg = registry("").replace(
        "checkers = [\"assert_event_complete\"]",
        "checkers = [\"assert_event_complete\", \"not_checking\"]",
    );
    // `not_checking` didefinisikan tetapi tidak memanggil check_payload/assert_event_complete.
    assert_eq!(details(&reg, SRC), ["not_checking"]);
}

#[test]
fn emitter_only_in_cfg_test_block_is_not_inventoried() {
    let src = "fn a() {}\n#[cfg(test)]\nmod tests { fn t() { AuditEvent::new(EventCode::E07, a, b, c); } }\n";
    assert_eq!(details(&registry(""), src), ["stale"]);
}

#[test]
fn missing_or_invalid_registry_is_reported() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let f = check(&dir);
    assert_eq!(f.len(), 1);
    assert_eq!(f[0].detail, "missing");
    assert!(parse("checkers = []\nunknown = 1").is_err());
}
