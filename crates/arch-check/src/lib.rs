//! Test arsitektur Ruang Sehat (F1-T12, EN-08; coding-standards §1.3, §3.4, §4.1, §5.7, §6).
//!
//! Crate ini hanya dipakai test. Pemeriksaan berbasis teks atas berkas Rust dan manifes; larangan
//! yang bisa ditegakkan compiler memakai clippy (`[workspace.lints]` dan `clippy.toml` di root),
//! dan crate ini menjaga agar konfigurasi itu tidak dilemahkan serta menutup celah yang tidak
//! dijangkau clippy (kepemilikan tabel, pola log, pengecualian yang melebar).

pub mod audit_coverage;
pub mod exceptions;
pub mod fs;
pub mod lexer;
pub mod manifests;
pub mod openapi;
pub mod sources;
pub mod tables;

/// Satu pelanggaran (atau temuan sebelum pengecualian diterapkan).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    /// Identitas aturan, misalnya `clock`, `env`, `log-debug`, `table-owner`.
    pub rule: &'static str,
    /// Path relatif terhadap root repositori, pemisah `/`.
    pub file: String,
    /// Nomor baris (1-based), 0 bila tidak berlaku.
    pub line: usize,
    /// Detail yang dicocokkan pengecualian: nama lint, token, atau nama makro.
    pub detail: String,
    /// Nama fungsi pembungkus terdekat, bila relevan.
    pub func: Option<String>,
    /// Penjelasan untuk manusia.
    pub msg: String,
}

impl Finding {
    pub(crate) fn new(
        rule: &'static str,
        file: &str,
        line: usize,
        detail: &str,
        msg: impl Into<String>,
    ) -> Self {
        Self {
            rule,
            file: file.to_string(),
            line,
            detail: detail.to_string(),
            func: None,
            msg: msg.into(),
        }
    }
}

impl std::fmt::Display for Finding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] {}:{} ({}): {}",
            self.rule, self.file, self.line, self.detail, self.msg
        )
    }
}

/// Menjalankan semua pemeriksaan sumber, manifes, dan pengecualian di `root`; mengembalikan pelanggaran.
pub fn check_all(root: &std::path::Path) -> Vec<Finding> {
    let mut findings = sources::check_sources(root);
    let mut out = Vec::new();
    match exceptions::load(root) {
        Ok(ex) => out.extend(exceptions::apply(std::mem::take(&mut findings), &ex)),
        Err(e) => {
            out.push(Finding::new(
                "exceptions",
                "architecture-exceptions.toml",
                0,
                "load",
                e,
            ));
            out.extend(findings);
        }
    }
    out.extend(manifests::check_manifests(root));
    out.extend(tables::check_all_apps(root));
    out.extend(openapi::check_openapi(root));
    out.extend(audit_coverage::check(root));
    out
}
