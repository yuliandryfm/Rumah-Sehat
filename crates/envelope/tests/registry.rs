//! Pemeriksaan registry `response_code` (F1-T09; registry §5 butir 6, KEPUTUSAN-82).
//!
//! Berjalan lewat `cargo test --workspace` di `scripts/verify.sh`. Memeriksa:
//! 1. dokumen `docs/05-response-codes.md` dapat diurai, tanpa kode ganda, HHH sama dengan HTTP;
//! 2. `codes::ALL` sama persis (dua arah, termasuk deskripsi) dengan kode modul `00` di dokumen;
//! 3. setiap literal pemanggilan konstruktor `ResponseCode` berliteral di `crates/` dan `apps/` terdaftar di dokumen
//!    dengan deskripsi yang sama, dan kode modul spesifik tidak dipakai dengan deskripsi lain.

// Berkas test: unwrap/panic di helper diperbolehkan (coding-standards §3.4); clippy.toml hanya
// membebaskan fungsi #[test], bukan helper.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use envelope::codes;
use envelope::registry::{RegistryEntry, parse};
use std::fs;
use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap()
}

fn load_registry() -> Vec<RegistryEntry> {
    let path = workspace_root().join("docs/05-response-codes.md");
    let text = fs::read_to_string(&path).unwrap();
    parse(&text).unwrap()
}

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = fs::read_dir(dir) else { return };
    for entry in rd.flatten() {
        let p = entry.path();
        if p.is_dir() {
            if p.file_name().is_some_and(|n| n == "target") {
                continue;
            }
            rust_files(&p, out);
        } else if p.extension().is_some_and(|e| e == "rs") {
            out.push(p);
        }
    }
}

/// Mengambil pasangan (kode, deskripsi) dari setiap pemanggilan konstruktor `ResponseCode` bergaya literal.
/// Literal diasumsikan tanpa escape (kode dan deskripsi registry tidak memuat tanda kutip).
fn used_codes(source: &str) -> Vec<(String, String)> {
    // concat! agar pola tidak muncul utuh di berkas ini dan tidak dipindai sebagai pemakaian.
    const PAT: &str = concat!("ResponseCode", "::new(");
    let mut found = Vec::new();
    let mut rest = source;
    while let Some(i) = rest.find(PAT) {
        rest = &rest[i + PAT.len()..];
        let args = rest.trim_start();
        let Some(args) = args.strip_prefix('"') else {
            continue;
        };
        let Some((code, tail)) = args.split_once('"') else {
            continue;
        };
        let tail = tail.trim_start().trim_start_matches(',').trim_start();
        let Some(tail) = tail.strip_prefix('"') else {
            continue;
        };
        let Some((desc, _)) = tail.split_once('"') else {
            continue;
        };
        found.push((code.to_string(), desc.to_string()));
    }
    found
}

#[test]
fn registry_document_is_parseable_and_non_empty() {
    let reg = load_registry();
    assert!(reg.len() >= 17, "registry harus memuat minimal kode umum");
}

#[test]
fn general_codes_match_registry_both_ways() {
    let reg = load_registry();
    for c in codes::ALL {
        let e = reg
            .iter()
            .find(|e| e.code == c.as_str())
            .unwrap_or_else(|| panic!("{} tidak ada di registry", c.as_str()));
        assert_eq!(e.description, c.description(), "{}", c.as_str());
        assert_eq!(e.http, c.http_status().as_u16(), "{}", c.as_str());
    }
    let general: Vec<&RegistryEntry> = reg
        .iter()
        .filter(|e| e.code.get(3..5) == Some("00"))
        .collect();
    assert_eq!(general.len(), codes::ALL.len());
    for e in general {
        assert!(
            codes::ALL.iter().any(|c| c.as_str() == e.code),
            "{} ada di registry tetapi tidak di codes::ALL",
            e.code
        );
    }
}

#[test]
fn every_response_code_literal_in_workspace_is_registered() {
    let reg = load_registry();
    let root = workspace_root();
    let mut files = Vec::new();
    rust_files(&root.join("crates"), &mut files);
    rust_files(&root.join("apps"), &mut files);
    assert!(!files.is_empty());
    let mut checked = 0;
    for f in files {
        let text = fs::read_to_string(&f).unwrap();
        for (code, desc) in used_codes(&text) {
            let e = reg.iter().find(|e| e.code == code).unwrap_or_else(|| {
                panic!("{code} dipakai di {} tetapi tidak terdaftar", f.display())
            });
            assert_eq!(e.description, desc, "deskripsi {code} di {}", f.display());
            checked += 1;
        }
    }
    assert!(
        checked >= codes::ALL.len(),
        "pemindai harus menemukan kode umum"
    );
}

#[test]
fn scanner_extracts_literals_across_lines() {
    let src = "const A: X = ResponseCode::new(\n    \"4090002\",\n    \"Kunci\",\n);\nResponseCode::new(var, \"x\")";
    assert_eq!(
        used_codes(src),
        vec![("4090002".to_string(), "Kunci".to_string())]
    );
}

#[test]
fn example_format_code_is_not_registered() {
    let reg = load_registry();
    // 4220101 hanya contoh format di ADR-023, belum terdaftar (registry §1): pemeriksa harus
    // menganggapnya tidak terdaftar. Literal sengaja tidak ditulis sebagai ResponseCode::new.
    assert!(reg.iter().all(|e| e.code != "4220101"));
    let used = used_codes("ResponseCode::new(\"4220101\", \"Contoh\")");
    assert!(used.iter().any(|(c, _)| reg.iter().all(|e| &e.code != c)));
}
