//! Galat modul `identity` dan pemetaannya ke kode terdaftar (coding-standards §3.2,
//! docs/05-response-codes.md §3). Tidak ada kode baru di modul ini: semua kasus memakai kode umum.

use envelope::{ApiError, FieldError, ResponseCode, codes};
use std::fmt;

/// Galat use case identitas dan sesi.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentityError {
    /// PostgreSQL tidak tersedia atau query gagal (503; detail hanya di log).
    Database,
    /// Valkey tidak tersedia saat menulis cache (503). Pemanggil mencoba ulang lewat job.
    CacheUnavailable,
    /// Pencabutan tanpa ID sesi atau dengan `expires_at` yang sudah lewat (500: kesalahan pemanggil).
    InvalidRevocation,
}

impl fmt::Display for IdentityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Database => "database tidak tersedia",
            Self::CacheUnavailable => "cache tidak tersedia",
            Self::InvalidRevocation => "pencabutan sesi tidak valid",
        })
    }
}

impl std::error::Error for IdentityError {}

impl IdentityError {
    /// Kode tetap untuk log.
    pub fn code(self) -> &'static str {
        match self {
            Self::Database => "database",
            Self::CacheUnavailable => "cache_unavailable",
            Self::InvalidRevocation => "invalid_revocation",
        }
    }
}

impl ApiError for IdentityError {
    fn response_code(&self) -> ResponseCode {
        match self {
            Self::Database | Self::CacheUnavailable => codes::UNAVAILABLE,
            Self::InvalidRevocation => codes::INTERNAL,
        }
    }

    fn field_errors(&self) -> &[FieldError] {
        &[]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn errors_map_to_registered_codes() {
        assert_eq!(IdentityError::Database.response_code().as_str(), "5030001");
        assert_eq!(
            IdentityError::CacheUnavailable.response_code().as_str(),
            "5030001"
        );
        assert_eq!(
            IdentityError::InvalidRevocation.response_code().as_str(),
            "5000001"
        );
    }
}
