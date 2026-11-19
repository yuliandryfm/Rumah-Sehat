//! Pembaca registry `response_code` dari `docs/05-response-codes.md` (§3 dan §6).
//!
//! Dokumen itu sumber kebenaran daftar kode (ADR-023 butir 6). Pembaca ini mengurai baris tabel
//! yang kolom pertamanya kode 7 digit dalam backtick, kolom kedua kode HTTP, kolom ketiga
//! deskripsi. Baris lain (tabel modul 2 digit, placeholder `—`) dilewati. Dipakai test
//! `tests/registry.rs` dan boleh dipakai test modul lain.

use crate::code::is_valid_shape;
use std::fmt;

/// Satu baris registry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistryEntry {
    pub code: String,
    pub http: u16,
    pub description: String,
}

/// Galat isi registry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryError {
    /// Baris kode dengan kolom HTTP atau deskripsi tidak sah.
    MalformedRow(String),
    /// Tiga digit pertama kode tidak sama dengan kolom HTTP.
    HttpMismatch(String),
    /// Kode muncul lebih dari sekali (ADR-023 butir 6: satu kode satu arti).
    Duplicate(String),
}

impl fmt::Display for RegistryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MalformedRow(c) => write!(f, "baris registry tidak sah untuk kode {c}"),
            Self::HttpMismatch(c) => {
                write!(f, "tiga digit pertama {c} tidak sama dengan kolom HTTP")
            }
            Self::Duplicate(c) => write!(f, "kode {c} terdaftar lebih dari sekali"),
        }
    }
}

impl std::error::Error for RegistryError {}

/// Mengurai teks Markdown registry. Gagal bila ada baris kode yang rusak, HTTP tidak
/// konsisten dengan kode, atau kode ganda.
pub fn parse(markdown: &str) -> Result<Vec<RegistryEntry>, RegistryError> {
    let mut entries: Vec<RegistryEntry> = Vec::new();
    for line in markdown.lines() {
        let line = line.trim();
        if !line.starts_with('|') {
            continue;
        }
        let mut cells = line.trim_matches('|').split('|').map(str::trim);
        let first = cells.next().unwrap_or_default();
        let Some(code) = first
            .strip_prefix('`')
            .and_then(|s| s.strip_suffix('`'))
            .filter(|s| s.len() == 7 && s.bytes().all(|b| b.is_ascii_digit()))
        else {
            continue;
        };
        let http = cells.next().and_then(|c| c.parse::<u16>().ok());
        let description = cells.next().filter(|d| !d.is_empty());
        let (Some(http), Some(description)) = (http, description) else {
            return Err(RegistryError::MalformedRow(code.to_string()));
        };
        if !is_valid_shape(code) {
            return Err(RegistryError::MalformedRow(code.to_string()));
        }
        if code.get(0..3).and_then(|h| h.parse::<u16>().ok()) != Some(http) {
            return Err(RegistryError::HttpMismatch(code.to_string()));
        }
        if entries.iter().any(|e| e.code == code) {
            return Err(RegistryError::Duplicate(code.to_string()));
        }
        entries.push(RegistryEntry {
            code: code.to_string(),
            http,
            description: description.to_string(),
        });
    }
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\
| Kode | Modul |
| --- | --- |
| `00` | Umum |
| `response_code` | HTTP | `response_description` |
| `2000000` | 200 | Berhasil | Sukses | `data` |
| `4040001` | 404 | Data tidak ditemukan | x | — |
| — | — | — | — | — | — |
";

    #[test]
    fn parses_code_rows_only() {
        let e = parse(SAMPLE).unwrap();
        assert_eq!(e.len(), 2);
        assert_eq!(e[0].code, "2000000");
        assert_eq!(e[1].http, 404);
        assert_eq!(e[1].description, "Data tidak ditemukan");
    }

    #[test]
    fn rejects_http_mismatch_duplicate_and_malformed() {
        assert_eq!(
            parse("| `4040001` | 400 | X |"),
            Err(RegistryError::HttpMismatch("4040001".into()))
        );
        assert_eq!(
            parse("| `2000000` | 200 | A |\n| `2000000` | 200 | B |"),
            Err(RegistryError::Duplicate("2000000".into()))
        );
        assert_eq!(
            parse("| `2000000` | dua ratus | A |"),
            Err(RegistryError::MalformedRow("2000000".into()))
        );
        assert_eq!(
            parse("| `2000000` | 200 |  |"),
            Err(RegistryError::MalformedRow("2000000".into()))
        );
        // Digit pertama 1 atau 6 bukan kategori yang dipakai.
        assert_eq!(
            parse("| `1000000` | 100 | A |"),
            Err(RegistryError::MalformedRow("1000000".into()))
        );
    }
}
