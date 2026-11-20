//! Galat autentikasi dan pemetaannya ke kode respons umum (coding-standards §3.3).

use envelope::CommonError;
use std::fmt;

/// Alasan token ditolak. Hanya untuk keputusan dan log internal: respons ke klien selalu umum
/// (`4010001` tanpa alasan rinci, coding-standards §3.3), kecuali JWKS tidak tersedia (`5030001`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthError {
    /// Tidak ada header `Authorization`.
    MissingCredentials,
    /// Header `Authorization` ganda, bukan skema `Bearer`, atau berisi karakter tidak sah.
    MalformedHeader,
    /// Token bukan JWS compact yang dapat diurai, terlalu panjang, atau klaim wajib hilang/salah bentuk.
    MalformedToken,
    /// `iss` bukan issuer realm yang dikonfigurasi backend ini (termasuk realm milik backend lain).
    UnknownIssuer,
    /// `alg` di luar daftar izin realm, termasuk `none` dan HMAC (RFC 8725 §3.1).
    AlgorithmNotAllowed,
    /// `kid` tidak ada atau tidak dikenal di JWKS realm.
    UnknownKey,
    /// Tanda tangan tidak cocok dengan kunci realm.
    InvalidSignature,
    /// `iss` tidak sama dengan issuer realm (RFC 8725 §3.8).
    WrongIssuer,
    /// `aud` tidak ada atau bukan untuk API ini (RFC 8725 §3.9).
    WrongAudience,
    /// Klaim `typ` bukan `Bearer` (misalnya ID token `ID` atau refresh token `Refresh`).
    WrongTokenType,
    /// `exp` sudah lewat (di luar toleransi jam).
    Expired,
    /// `nbf` atau `iat` di masa depan (di luar toleransi jam).
    NotYetValid,
    /// Realm akun manusia tetapi token tanpa klaim sesi `sid`.
    MissingSession,
    /// `azp` tidak ada di daftar klien yang diizinkan realm.
    UnauthorizedParty,
    /// Realm token sah, tetapi tidak diizinkan untuk endpoint ini.
    RealmNotAccepted,
    /// JWKS realm tidak dapat diambil atau tidak memuat kunci yang dapat dipakai: gagal-tertutup.
    KeysUnavailable,
}

impl AuthError {
    /// Kode alasan snake_case untuk log internal (daftar tertutup, tanpa isi token).
    pub const fn reason(self) -> &'static str {
        match self {
            Self::MissingCredentials => "missing_credentials",
            Self::MalformedHeader => "malformed_header",
            Self::MalformedToken => "malformed",
            Self::UnknownIssuer => "unknown_issuer",
            Self::AlgorithmNotAllowed => "algorithm_not_allowed",
            Self::UnknownKey => "unknown_key",
            Self::InvalidSignature => "invalid_signature",
            Self::WrongIssuer => "wrong_issuer",
            Self::WrongAudience => "wrong_audience",
            Self::WrongTokenType => "wrong_type",
            Self::Expired => "expired",
            Self::NotYetValid => "not_yet_valid",
            Self::MissingSession => "missing_session",
            Self::UnauthorizedParty => "unauthorized_party",
            Self::RealmNotAccepted => "realm_not_accepted",
            Self::KeysUnavailable => "keys_unavailable",
        }
    }

    /// Kode respons umum: `503` bila JWKS tidak tersedia (masalah server, bukan token; permintaan
    /// tetap ditolak), `401` untuk semua alasan lain dengan isi yang sama.
    pub fn to_common_error(self) -> CommonError {
        match self {
            Self::KeysUnavailable => CommonError::Unavailable,
            _ => CommonError::Unauthenticated,
        }
    }
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.reason())
    }
}

impl std::error::Error for AuthError {}

/// Konfigurasi autentikasi tidak valid. Start proses gagal (KEPUTUSAN-132); pesan tanpa nilai rahasia.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthnConfigError {
    /// Tidak ada realm.
    NoRealms,
    /// Realm yang sama dikonfigurasi dua kali.
    DuplicateRealm(&'static str),
    /// Issuer yang sama dipakai dua realm.
    DuplicateIssuer(&'static str),
    /// Realm tidak boleh diterima backend ini (ADR-011 butir 11).
    RealmNotAccepted(&'static str),
    /// Issuer kosong atau bukan URL `https://`/`http://`.
    InvalidIssuer(&'static str),
    /// Daftar audience kosong atau berisi string kosong.
    NoAudience(&'static str),
    /// Daftar algoritma kosong.
    NoAlgorithm(&'static str),
    /// Algoritma di luar keluarga RSA/EC (HMAC dan EdDSA tidak diizinkan).
    AlgorithmNotAllowed(&'static str),
    /// Alamat JWKS tidak valid (lihat [`crate::HttpJwksSource`]).
    InvalidJwksAddress,
    /// Durasi (TTL, interval, toleransi, batas waktu) di luar rentang.
    InvalidDuration(&'static str),
}

impl fmt::Display for AuthnConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoRealms => f.write_str("tidak ada realm yang dikonfigurasi"),
            Self::DuplicateRealm(r) => write!(f, "realm {r} dikonfigurasi dua kali"),
            Self::DuplicateIssuer(r) => write!(f, "issuer realm {r} dipakai realm lain"),
            Self::RealmNotAccepted(r) => write!(f, "realm {r} tidak boleh diterima backend ini"),
            Self::InvalidIssuer(r) => write!(f, "issuer realm {r} tidak valid"),
            Self::NoAudience(r) => write!(f, "audience realm {r} kosong"),
            Self::NoAlgorithm(r) => write!(f, "daftar algoritma realm {r} kosong"),
            Self::AlgorithmNotAllowed(r) => {
                write!(f, "realm {r} memuat algoritma di luar keluarga RSA/EC")
            }
            Self::InvalidJwksAddress => f.write_str("alamat JWKS tidak valid"),
            Self::InvalidDuration(name) => write!(f, "durasi {name} di luar rentang"),
        }
    }
}

impl std::error::Error for AuthnConfigError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_keys_unavailable_maps_to_503_everything_else_is_uniform_401() {
        let all = [
            AuthError::MissingCredentials,
            AuthError::MalformedHeader,
            AuthError::MalformedToken,
            AuthError::UnknownIssuer,
            AuthError::AlgorithmNotAllowed,
            AuthError::UnknownKey,
            AuthError::InvalidSignature,
            AuthError::WrongIssuer,
            AuthError::WrongAudience,
            AuthError::WrongTokenType,
            AuthError::Expired,
            AuthError::NotYetValid,
            AuthError::MissingSession,
            AuthError::UnauthorizedParty,
            AuthError::RealmNotAccepted,
        ];
        for e in all {
            assert_eq!(e.to_common_error(), CommonError::Unauthenticated, "{e}");
        }
        assert_eq!(
            AuthError::KeysUnavailable.to_common_error(),
            CommonError::Unavailable
        );
    }
}
