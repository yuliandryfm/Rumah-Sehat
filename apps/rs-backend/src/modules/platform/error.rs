//! Galat domain berkas modul `platform` dan pemetaannya ke kode terdaftar (coding-standards §3.2,
//! docs/05-response-codes.md §3). Tidak ada kode baru: semua kasus memakai kode umum modul `00`.

use envelope::{ApiError, FieldError, ResponseCode, codes};
use std::fmt;

/// Galat use case berkas.
#[derive(Debug)]
pub enum FileError {
    /// Slot atau berkas tidak ada, milik akun lain, sudah dipakai, kedaluwarsa, atau tidak boleh diakses
    /// (404, KU-03 dan ADR-012 butir 4: tidak membedakan).
    NotFound,
    /// Berkas melebihi batas ukuran (413, NFR-SEC-08, US-106 AC3).
    TooLarge,
    /// Jenis berkas menurut isi tidak didukung atau tidak cocok dengan yang dinyatakan (415, US-106 AC3).
    UnsupportedType,
    /// Masukan tidak valid (422), misalnya jenis yang dinyatakan bukan JPG/PNG atau berkas belum diunggah.
    Validation(Vec<FieldError>),
    /// Object storage tidak tersedia atau habis waktu (503).
    StorageUnavailable,
    /// Antrean validasi penuh melewati batas tunggu (503, TD-17).
    Busy,
    /// Galat tak terduga (500); sumbernya hanya untuk log, tidak pernah ke respons.
    Internal(&'static str),
}

impl fmt::Display for FileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound => f.write_str("slot unggah atau berkas tidak ditemukan"),
            Self::TooLarge => f.write_str("berkas terlalu besar"),
            Self::UnsupportedType => f.write_str("jenis berkas tidak didukung"),
            Self::Validation(_) => f.write_str("data tidak valid"),
            Self::StorageUnavailable => f.write_str("object storage tidak tersedia"),
            Self::Busy => f.write_str("antrean validasi berkas penuh"),
            Self::Internal(what) => write!(f, "galat internal: {what}"),
        }
    }
}

impl std::error::Error for FileError {}

impl ApiError for FileError {
    fn response_code(&self) -> ResponseCode {
        match self {
            Self::NotFound => codes::NOT_FOUND,
            Self::TooLarge => codes::PAYLOAD_TOO_LARGE,
            Self::UnsupportedType => codes::UNSUPPORTED_MEDIA_TYPE,
            Self::Validation(_) => codes::INVALID,
            Self::StorageUnavailable | Self::Busy => codes::UNAVAILABLE,
            Self::Internal(_) => codes::INTERNAL,
        }
    }

    fn field_errors(&self) -> &[FieldError] {
        match self {
            Self::Validation(e) => e,
            _ => &[],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn errors_map_to_registered_codes() {
        let cases = [
            (FileError::NotFound, "4040001"),
            (FileError::TooLarge, "4130001"),
            (FileError::UnsupportedType, "4150001"),
            (FileError::Validation(vec![]), "4220001"),
            (FileError::StorageUnavailable, "5030001"),
            (FileError::Busy, "5030001"),
            (FileError::Internal("x"), "5000001"),
        ];
        for (e, code) in cases {
            assert_eq!(e.response_code().as_str(), code);
        }
    }
}
