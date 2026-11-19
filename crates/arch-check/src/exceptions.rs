//! Pengecualian yang sah, dicatat di `architecture-exceptions.toml` (root). Tiap pengecualian
//! menyebut berkas, detail, (opsional) fungsi, jumlah kemunculan persis, dan alasan. Kemunculan
//! melebihi `count`, pengecualian tanpa kemunculan (usang), atau alasan kosong menjadi pelanggaran.

use crate::Finding;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Exception {
    pub rule: String,
    pub file: String,
    pub detail: String,
    #[serde(default)]
    pub in_fn: Option<String>,
    #[serde(default = "one")]
    pub count: usize,
    pub reason: String,
}

fn one() -> usize {
    1
}

#[derive(Debug, Deserialize, Default)]
struct Doc {
    #[serde(default)]
    exception: Vec<Exception>,
}

/// Membaca `architecture-exceptions.toml` di `root`.
pub fn load(root: &Path) -> Result<Vec<Exception>, String> {
    let text = std::fs::read_to_string(root.join("architecture-exceptions.toml"))
        .map_err(|e| format!("architecture-exceptions.toml tidak terbaca: {e}"))?;
    parse(&text)
}

/// Mengurai teks pengecualian.
pub fn parse(text: &str) -> Result<Vec<Exception>, String> {
    toml::from_str::<Doc>(text)
        .map(|d| d.exception)
        .map_err(|e| format!("architecture-exceptions.toml tidak valid: {e}"))
}

/// Menerapkan pengecualian: temuan yang tertutup dibuang, sisanya (dan masalah pengecualian) dikembalikan.
pub fn apply(findings: Vec<Finding>, exceptions: &[Exception]) -> Vec<Finding> {
    let mut used = vec![0usize; exceptions.len()];
    let mut out = Vec::new();
    for f in findings {
        let hit = exceptions.iter().enumerate().position(|(i, e)| {
            e.rule == f.rule
                && e.file == f.file
                && e.detail == f.detail
                && e.in_fn.as_ref().is_none_or(|n| f.func.as_ref() == Some(n))
                && used.get(i).copied().unwrap_or(usize::MAX) < e.count
        });
        match hit {
            Some(i) => {
                if let Some(u) = used.get_mut(i) {
                    *u += 1;
                }
            }
            None => out.push(f),
        }
    }
    for (e, n) in exceptions.iter().zip(used) {
        let mk = |msg: String| Finding::new("exceptions", &e.file, 0, &e.detail, msg);
        if e.reason.trim().is_empty() {
            out.push(mk(format!("pengecualian {} tanpa alasan", e.rule)));
        }
        if n == 0 {
            out.push(mk(format!(
                "pengecualian {} usang (tidak ada kemunculan); hapus dari architecture-exceptions.toml",
                e.rule
            )));
        } else if n < e.count {
            out.push(mk(format!(
                "pengecualian {} mengizinkan {} kemunculan tetapi hanya {n} ditemukan; sempitkan count",
                e.rule, e.count
            )));
        }
    }
    out
}
