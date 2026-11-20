//! Test timing penolakan objek di **build release** (F1-T38, TD-15 Low, KU-03: tanpa selisih waktu
//! tanggap yang mencolok antara "milik orang lain" dan "tidak ada").
//!
//! Membangun probe `examples/denial_timing.rs` dengan `cargo build --release` (direktori target
//! terpisah supaya tidak berebut kunci dengan `cargo test` yang sedang berjalan), menjalankannya, lalu
//! membandingkan median waktu tanggap. Security log probe sengaja ditulis ke sink lambat (3 ms per
//! baris): jalur kontrol yang menulis log secara sinkron harus terlihat lebih lambat (bukti probe peka),
//! sedangkan jalur `authz::visible_object` (pencatatan di latar belakang) tidak boleh berbeda berarti
//! dari jalur "tidak ada".

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)] // berkas test

use std::path::Path;
use std::process::Command;

/// Selisih median maksimum yang diterima antara "milik orang lain" dan "tidak ada" (mikrodetik).
/// Usulan teknis F1-T38: jauh di bawah selisih yang ditimbulkan satu penulisan log lambat (3 ms).
const MAX_GAP_US: i128 = 1_000;
/// Selisih minimum jalur kontrol (log sinkron) agar probe dianggap peka (mikrodetik).
const MIN_CONTROL_GAP_US: i128 = 2_000;

fn value(line: &str, key: &str) -> i128 {
    line.split_whitespace()
        .find_map(|kv| kv.strip_prefix(&format!("{key}=")))
        .and_then(|v| v.parse().ok())
        .unwrap_or_else(|| panic!("{key} tidak ada di keluaran probe: {line}"))
}

#[test]
fn foreign_object_denial_is_not_slower_than_missing_object_in_release_build() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace = manifest_dir.join("../..");
    let target_dir = workspace.join("target/timing-release");
    let build = Command::new(env!("CARGO"))
        .current_dir(&workspace)
        .args([
            "build",
            "--release",
            "--locked",
            "-p",
            "authz",
            "--example",
            "denial_timing",
            "--target-dir",
        ])
        .arg(&target_dir)
        .status()
        .expect("cargo dapat dijalankan");
    assert!(build.success(), "build release probe gagal");

    let exe = target_dir.join("release/examples/denial_timing");
    let out = Command::new(&exe).output().expect("probe dapat dijalankan");
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success(),
        "probe gagal: {}{}",
        stdout,
        String::from_utf8_lossy(&out.stderr)
    );
    let line = stdout
        .lines()
        .find(|l| l.starts_with("missing_us="))
        .unwrap_or_else(|| panic!("keluaran probe tidak dikenal: {stdout}"));
    let missing = value(line, "missing_us");
    let foreign = value(line, "foreign_us");
    let control = value(line, "sync_control_us");
    assert!(
        control - missing >= MIN_CONTROL_GAP_US,
        "probe tidak peka (kontrol log sinkron tidak lebih lambat): {line}"
    );
    assert!(
        (foreign - missing).abs() <= MAX_GAP_US,
        "selisih waktu tanggap milik-orang-lain vs tidak-ada terlalu besar: {line}"
    );
}
