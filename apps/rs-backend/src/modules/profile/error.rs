//! Galat modul `profile` dan pemetaannya ke kode terdaftar (coding-standards §3.2,
//! docs/05-response-codes.md §3). Tidak ada kode baru: kasus yang ada memakai kode umum modul `00`.

use envelope::{ApiError, FieldError, ResponseCode, codes};
use std::fmt;

/// Galat use case profil.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileError {
    /// PostgreSQL tidak tersedia atau query gagal (503; detail hanya di log).
    Database,
}

impl fmt::Display for ProfileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Database => f.write_str("database tidak tersedia"),
        }
    }
}

impl std::error::Error for ProfileError {}

impl ApiError for ProfileError {
    fn response_code(&self) -> ResponseCode {
        match self {
            Self::Database => codes::UNAVAILABLE,
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
        assert_eq!(ProfileError::Database.response_code().as_str(), "5030001");
    }
}
