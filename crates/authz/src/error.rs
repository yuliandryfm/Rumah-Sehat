//! Galat otorisasi dan galat penyiapan kebijakan.

use envelope::CommonError;
use std::fmt;

/// Penolakan otorisasi di tingkat service (ADR-012 butir 4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthzError {
    /// Aksi dikenal tetapi peran tidak diizinkan → `403` `4030001`.
    Forbidden,
    /// Objek tidak ada **atau** tidak boleh diakses → `404` `4040001` dengan isi identik (KU-03).
    NotFound,
}

impl AuthzError {
    /// Kode respons umum.
    pub fn to_common_error(self) -> CommonError {
        match self {
            Self::Forbidden => CommonError::Forbidden,
            Self::NotFound => CommonError::NotFound,
        }
    }
}

impl fmt::Display for AuthzError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Forbidden => "forbidden",
            Self::NotFound => "not_found",
        })
    }
}

impl std::error::Error for AuthzError {}

/// Galat penyiapan router dan kebijakan. Proses `api` gagal start (KEPUTUSAN-127).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicySetupError {
    /// Operasi terpasang tanpa kebijakan.
    MissingPolicy {
        /// Metode HTTP.
        method: String,
        /// Pola path.
        path: String,
    },
    /// Kebijakan untuk operasi yang tidak terpasang.
    StalePolicy {
        /// Metode HTTP.
        method: String,
        /// Pola path.
        path: String,
    },
    /// Endpoint didaftarkan dua kali.
    DuplicatePolicy {
        /// Metode HTTP.
        method: String,
        /// Pola path.
        path: String,
    },
    /// Kebijakan tidak dapat dipenuhi siapa pun (tanpa jenis akun, atau admin Ruang Sehat tanpa sub-peran).
    UnsatisfiablePolicy {
        /// Metode HTTP.
        method: String,
        /// Pola path.
        path: String,
    },
    /// Ada endpoint terautentikasi tetapi validator token atau resolver akun tidak dipasang.
    MissingAuthentication,
    /// Kebijakan membutuhkan realm yang tidak diterima validator token.
    RealmNotConfigured {
        /// Realm.
        realm: &'static str,
    },
    /// Endpoint HR perusahaan di luar namespace `/v1/corporate/`, atau endpoint terautentikasi di namespace
    /// itu untuk jenis akun lain (ADR-027 butir 2).
    CorporateNamespace {
        /// Metode HTTP.
        method: String,
        /// Pola path.
        path: String,
    },
    /// Ada endpoint HR perusahaan tetapi pembaca keanggotaan perusahaan tidak dipasang (ADR-028 butir 1).
    MissingCorporateScope,
}

impl fmt::Display for PolicySetupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingPolicy { method, path } => {
                write!(f, "endpoint {method} {path} tidak punya kebijakan")
            }
            Self::StalePolicy { method, path } => {
                write!(f, "kebijakan {method} {path} tidak menunjuk endpoint yang terpasang")
            }
            Self::DuplicatePolicy { method, path } => {
                write!(f, "kebijakan {method} {path} didaftarkan dua kali")
            }
            Self::UnsatisfiablePolicy { method, path } => {
                write!(f, "kebijakan {method} {path} tidak dapat dipenuhi")
            }
            Self::MissingAuthentication => f.write_str(
                "ada endpoint terautentikasi tetapi validator token atau resolver akun tidak dipasang",
            ),
            Self::RealmNotConfigured { realm } => {
                write!(f, "kebijakan membutuhkan realm {realm} yang tidak dikonfigurasi")
            }
            Self::CorporateNamespace { method, path } => write!(
                f,
                "endpoint {method} {path} melanggar namespace korporat (HR hanya di /v1/corporate/, \
                 dan namespace itu hanya untuk HR atau publik)"
            ),
            Self::MissingCorporateScope => f.write_str(
                "ada endpoint HR perusahaan tetapi pembaca keanggotaan perusahaan tidak dipasang",
            ),
        }
    }
}

impl std::error::Error for PolicySetupError {}
