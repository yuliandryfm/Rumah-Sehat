//! Galat modul `privacy` dan pemetaannya ke kode terdaftar (coding-standards §3.2, docs/05-response-codes.md).
//! F1-T49 memakai kode umum modul `00` saja.

use envelope::{ApiError, FieldError, ResponseCode, codes};
use std::fmt;

/// Galat use case persetujuan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrivacyError {
    /// Batas laju terlampaui (429).
    RateLimited,
    /// Belum ada versi teks yang berlaku, atau database tidak tersedia (503). Tanpa versi berlaku,
    /// pendaftaran tidak dapat dilakukan (US-101 AC4); ini galat penyebaran, bukan galat pengguna.
    Unavailable,
    /// Galat tak terduga, misalnya isi versi tidak berbentuk `rs-consent-text/1` (500). Sumbernya hanya di log.
    Internal(&'static str),
}

impl fmt::Display for PrivacyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RateLimited => f.write_str("batas laju"),
            Self::Unavailable => f.write_str("teks persetujuan tidak tersedia"),
            Self::Internal(what) => write!(f, "galat internal: {what}"),
        }
    }
}

impl std::error::Error for PrivacyError {}

impl ApiError for PrivacyError {
    fn response_code(&self) -> ResponseCode {
        match self {
            Self::RateLimited => codes::RATE_LIMITED,
            Self::Unavailable => codes::UNAVAILABLE,
            Self::Internal(_) => codes::INTERNAL,
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
        assert_eq!(
            PrivacyError::RateLimited.response_code().as_str(),
            "4290001"
        );
        assert_eq!(
            PrivacyError::Unavailable.response_code().as_str(),
            "5030001"
        );
        assert_eq!(
            PrivacyError::Internal("x").response_code().as_str(),
            "5000001"
        );
    }
}
