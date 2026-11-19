//! Test arsitektur atas repositori sungguhan (langkah "test-arsitektur" di skrip verifikasi).
//! Gagal bila ada pelanggaran larangan, kepemilikan tabel, atau pengecualian yang melebar/usang.

use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn repository_has_no_architecture_violations() {
    let v = arch_check::check_all(&root());
    let report: Vec<String> = v.iter().map(ToString::to_string).collect();
    assert!(
        v.is_empty(),
        "pelanggaran arsitektur:\n{}",
        report.join("\n")
    );
}

#[test]
fn module_tables_files_exist_for_every_backend_app() {
    // Skrip verifikasi menyalakan langkah ini hanya bila berkasnya ada; test ini menjaga agar
    // berkas itu tidak hilang diam-diam.
    assert!(root().join("apps/rs-backend/module-tables.toml").is_file());
    assert!(root().join("architecture-exceptions.toml").is_file());
}

#[test]
fn exceptions_stay_narrow() {
    // Batas keras jumlah pengecualian: menambah entri harus disengaja (ubah angka ini di review).
    let ex = arch_check::exceptions::load(&root()).unwrap();
    assert!(
        ex.len() <= 8,
        "jumlah pengecualian {} melebihi batas review",
        ex.len()
    );
    assert!(
        ex.iter()
            .all(|e| e.count <= 4 && !e.reason.trim().is_empty())
    );
}
