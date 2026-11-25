//! Galat modul `corporate` dan pemetaannya ke kode terdaftar (coding-standards §3.2,
//! docs/05-response-codes.md §3). Modul Korporat belum punya kode modul di registry §2.1; F1-T104 hanya
//! memakai kode umum modul `00`: keputusan atas pengajuan yang sudah diputuskan = `4090001` "Konflik status"
//! (transisi kalah dari transisi yang tersimpan lebih dulu, AB-36).

use crate::modules::platform::api::FileError;
use envelope::{ApiError, FieldError, ResponseCode, codes};
use std::fmt;

/// `4090110`: NIB sudah terdaftar pada perusahaan lain (US-1903 AC4, seperti US-1901 AC7). Pesan umum: tidak
/// memuat nama atau data perusahaan lain.
pub const NIB_ALREADY_REGISTERED: ResponseCode = ResponseCode::new(
    "4090110",
    "NIB sudah terdaftar. Hubungi Layanan Pelanggan bila perusahaan ini belum pernah didaftarkan",
);
/// `4090111`: email HR pertama sudah dipakai akun lain di realm `korporat` (US-1903 AC1). Hanya untuk
/// Verifikator (staf berwenang); undangan HR oleh HR memakai respons seragam (ADR-030 butir 7, P30-2).
pub const HR_EMAIL_ALREADY_REGISTERED: ResponseCode = ResponseCode::new(
    "4090111",
    "Email HR sudah dipakai akun lain di realm korporat",
);

/// Galat use case modul Korporat. Pesan dan isi tidak pernah memuat NPWP (klasifikasi Terbatas, DD A20).
#[derive(Debug)]
pub enum CorporateError {
    /// Badan permintaan bukan JSON yang dapat dibaca (400).
    BadRequest,
    /// Badan permintaan tanpa `Content-Type: application/json` (415).
    UnsupportedMediaType,
    /// Perusahaan atau dokumen tidak ada (404, KU-03).
    NotFound,
    /// Masukan tidak valid (422).
    Validation(Vec<FieldError>),
    /// Keputusan atas pengajuan yang sudah diputuskan atau tidak lagi menunggu verifikasi (409,
    /// US-1902 AC4).
    AlreadyDecided,
    /// NIB sudah dipakai perusahaan lain yang belum terhapus (409, US-1903 AC4).
    NibAlreadyRegistered,
    /// Email HR pertama sudah dipakai akun di realm `korporat` (409, US-1903 AC1).
    HrEmailAlreadyRegistered,
    /// Galat berkas dari modul `platform`.
    File(FileError),
    /// PostgreSQL tidak tersedia (503; detail hanya di log).
    Database,
    /// Galat tak terduga (500); sumbernya hanya untuk log, tidak pernah ke respons.
    Internal(&'static str),
}

impl fmt::Display for CorporateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadRequest => f.write_str("permintaan tidak dapat dibaca"),
            Self::UnsupportedMediaType => f.write_str("jenis media tidak didukung"),
            Self::NotFound => f.write_str("data tidak ditemukan"),
            Self::Validation(_) => f.write_str("data tidak valid"),
            Self::AlreadyDecided => f.write_str("pengajuan sudah diputuskan"),
            Self::NibAlreadyRegistered => f.write_str("NIB sudah terdaftar"),
            Self::HrEmailAlreadyRegistered => f.write_str("email HR sudah dipakai"),
            Self::File(e) => write!(f, "berkas: {e}"),
            Self::Database => f.write_str("database tidak tersedia"),
            Self::Internal(what) => write!(f, "galat internal: {what}"),
        }
    }
}

impl std::error::Error for CorporateError {}

impl ApiError for CorporateError {
    fn response_code(&self) -> ResponseCode {
        match self {
            Self::BadRequest => codes::BAD_REQUEST,
            Self::UnsupportedMediaType => codes::UNSUPPORTED_MEDIA_TYPE,
            Self::NotFound => codes::NOT_FOUND,
            Self::Validation(_) => codes::INVALID,
            Self::AlreadyDecided => codes::STATUS_CONFLICT,
            Self::NibAlreadyRegistered => NIB_ALREADY_REGISTERED,
            Self::HrEmailAlreadyRegistered => HR_EMAIL_ALREADY_REGISTERED,
            Self::File(e) => e.response_code(),
            Self::Database => codes::UNAVAILABLE,
            Self::Internal(_) => codes::INTERNAL,
        }
    }

    fn field_errors(&self) -> &[FieldError] {
        match self {
            Self::Validation(e) => e,
            Self::File(e) => e.field_errors(),
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
            (CorporateError::BadRequest, "4000001"),
            (CorporateError::UnsupportedMediaType, "4150001"),
            (CorporateError::NotFound, "4040001"),
            (CorporateError::Validation(vec![]), "4220001"),
            (CorporateError::AlreadyDecided, "4090001"),
            (CorporateError::NibAlreadyRegistered, "4090110"),
            (CorporateError::HrEmailAlreadyRegistered, "4090111"),
            (CorporateError::File(FileError::NotFound), "4040001"),
            (CorporateError::Database, "5030001"),
            (CorporateError::Internal("x"), "5000001"),
        ];
        for (e, code) in cases {
            assert_eq!(e.response_code().as_str(), code);
        }
    }
}
