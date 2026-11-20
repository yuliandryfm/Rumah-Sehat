//! Galat domain modul `content_support` dan pemetaannya ke kode terdaftar (coding-standards §3.2,
//! docs/05-response-codes.md). Tidak ada kode baru: semua kasus memakai kode umum modul `00`.

use envelope::{ApiError, FieldError, ResponseCode, codes};
use std::fmt;

/// Galat use case konten.
#[derive(Debug)]
pub enum ContentError {
    /// Badan permintaan tidak dapat dibaca (400).
    BadRequest,
    /// Konten tidak ada, terhapus, atau (jalur publik) tidak sedang tayang (404, KU-03, US-1801 AC3).
    NotFound,
    /// Transisi status tidak sah atau kalah balapan, atau versi halaman tetap lain bentrok (409).
    StatusConflict,
    /// Jenis media tidak didukung (415): badan bukan JSON.
    UnsupportedMediaType,
    /// Validasi gagal dengan galat per isian (422, KU-01).
    Validation(Vec<FieldError>),
    /// Batas laju jalur publik terlampaui (429).
    RateLimited,
    /// Galat tak terduga (500); sumbernya hanya untuk log, tidak pernah ke respons.
    Internal(&'static str),
}

impl fmt::Display for ContentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadRequest => f.write_str("permintaan tidak dapat dibaca"),
            Self::NotFound => f.write_str("konten tidak ditemukan"),
            Self::StatusConflict => f.write_str("konflik status konten"),
            Self::UnsupportedMediaType => f.write_str("jenis media tidak didukung"),
            Self::Validation(_) => f.write_str("data tidak valid"),
            Self::RateLimited => f.write_str("batas laju"),
            Self::Internal(what) => write!(f, "galat internal: {what}"),
        }
    }
}

impl std::error::Error for ContentError {}

impl ApiError for ContentError {
    fn response_code(&self) -> ResponseCode {
        match self {
            Self::BadRequest => codes::BAD_REQUEST,
            Self::NotFound => codes::NOT_FOUND,
            Self::StatusConflict => codes::STATUS_CONFLICT,
            Self::UnsupportedMediaType => codes::UNSUPPORTED_MEDIA_TYPE,
            Self::Validation(_) => codes::INVALID,
            Self::RateLimited => codes::RATE_LIMITED,
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

impl From<sqlx::Error> for ContentError {
    fn from(e: sqlx::Error) -> Self {
        // Pelanggaran unik halaman tetap (uqp_cms_content_site_page_open) = balapan penayangan (409).
        // Pesan galat database tidak pernah diteruskan (dapat memuat isi baris, TD-03).
        let unique_page = e
            .as_database_error()
            .and_then(|d| d.constraint())
            .is_some_and(|c| c == "uqp_cms_content_site_page_open");
        if unique_page {
            Self::StatusConflict
        } else {
            Self::Internal("database")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn errors_map_to_registered_codes() {
        let cases = [
            (ContentError::BadRequest, "4000001"),
            (ContentError::NotFound, "4040001"),
            (ContentError::StatusConflict, "4090001"),
            (ContentError::UnsupportedMediaType, "4150001"),
            (ContentError::Validation(vec![]), "4220001"),
            (ContentError::RateLimited, "4290001"),
            (ContentError::Internal("x"), "5000001"),
        ];
        for (e, code) in cases {
            assert_eq!(e.response_code().as_str(), code);
        }
    }
}
