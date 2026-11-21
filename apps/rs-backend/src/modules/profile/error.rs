//! Galat modul `profile` dan pemetaannya ke kode terdaftar (coding-standards §3.2,
//! docs/05-response-codes.md §3 dan §6 modul `02`). Kode khusus modul `02` didaftarkan bersama endpoint
//! pengajuan identitas (F1-T61, US-106); arti lain memakai kode umum modul `00`.

use crate::modules::platform::api::FileError;
use envelope::{ApiError, FieldError, ResponseCode, codes};
use std::fmt;

/// US-106 AC5: pengajuan baru saat pengajuan sebelumnya masih menunggu verifikasi.
const SUBMISSION_LOCKED: ResponseCode = ResponseCode::new(
    "4090201",
    "Foto KTP tidak dapat diganti sampai pengajuan diputuskan atau dibatalkan",
);

/// US-106 AC4: pesan umum yang sama untuk registri NIK pemilik akun bentrok dan batas berlapis.
const NIK_UNAVAILABLE: ResponseCode = ResponseCode::new(
    "4090202",
    "NIK ini tidak dapat dipakai untuk akun Anda. Hubungi Layanan Pelanggan bila ini NIK Anda.",
);

/// Galat use case profil dan verifikasi identitas.
#[derive(Debug)]
pub enum ProfileError {
    /// Badan permintaan bukan JSON yang dapat dibaca (400, F1-T61).
    BadRequest,
    /// Badan permintaan tanpa `Content-Type: application/json` (415, F1-T61).
    UnsupportedMediaType,
    /// Penerima layanan atau pengajuan tidak ada **atau** tidak boleh diakses (404, KU-03).
    NotFound,
    /// Masukan tidak valid (422), misalnya NIK bukan 16 digit (US-106 AC2).
    Validation(Vec<FieldError>),
    /// Pengajuan masih menunggu verifikasi: foto tidak dapat diganti sampai diputuskan atau dibatalkan
    /// (US-106 AC5).
    SubmissionLocked,
    /// NIK tidak dapat dipakai (pesan umum US-106 AC4: registri pemilik akun bentrok atau batas berlapis
    /// tercapai). Isi respons sama untuk semua sebab.
    NikUnavailable,
    /// NIK sama dengan NIK penerima layanan lain di akun yang sama (US-108 AC4-AC5).
    DuplicateNikInAccount,
    /// Transisi status tidak sah atau kalah oleh transisi yang tersimpan lebih dulu (AB-36, US-106 AC5,
    /// US-201 AC5).
    TransitionConflict,
    /// Proyeksi status menyimpang dari riwayat (ADR-008 butir 4); transaksi dibatalkan (500).
    ProjectionMismatch,
    /// Galat berkas dari modul `platform` (404/413/415/422/503).
    File(FileError),
    /// PostgreSQL, Valkey, atau OpenBao tidak tersedia (503; detail hanya di log).
    Database,
    /// Galat tak terduga (500); sumbernya hanya untuk log, tidak pernah ke respons.
    Internal(&'static str),
}

impl fmt::Display for ProfileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadRequest => f.write_str("permintaan tidak dapat dibaca"),
            Self::UnsupportedMediaType => f.write_str("jenis media tidak didukung"),
            Self::NotFound => f.write_str("data tidak ditemukan"),
            Self::Validation(_) => f.write_str("data tidak valid"),
            Self::SubmissionLocked => f.write_str("pengajuan masih menunggu verifikasi"),
            Self::NikUnavailable => f.write_str("NIK tidak dapat dipakai"),
            Self::DuplicateNikInAccount => f.write_str("NIK sudah dipakai di akun yang sama"),
            Self::TransitionConflict => f.write_str("konflik status"),
            Self::ProjectionMismatch => f.write_str("proyeksi status menyimpang"),
            Self::File(e) => write!(f, "berkas: {e}"),
            Self::Database => f.write_str("database tidak tersedia"),
            Self::Internal(what) => write!(f, "galat internal: {what}"),
        }
    }
}

impl std::error::Error for ProfileError {}

impl ApiError for ProfileError {
    fn response_code(&self) -> ResponseCode {
        match self {
            Self::BadRequest => codes::BAD_REQUEST,
            Self::UnsupportedMediaType => codes::UNSUPPORTED_MEDIA_TYPE,
            Self::NotFound => codes::NOT_FOUND,
            Self::Validation(_) => codes::INVALID,
            Self::SubmissionLocked => SUBMISSION_LOCKED,
            Self::NikUnavailable => NIK_UNAVAILABLE,
            // US-108 (anggota keluarga) belum punya endpoint; kodenya didaftarkan task itu.
            Self::DuplicateNikInAccount | Self::TransitionConflict => codes::STATUS_CONFLICT,
            Self::ProjectionMismatch | Self::Internal(_) => codes::INTERNAL,
            Self::File(e) => e.response_code(),
            Self::Database => codes::UNAVAILABLE,
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
            (ProfileError::BadRequest, "4000001"),
            (ProfileError::UnsupportedMediaType, "4150001"),
            (ProfileError::NotFound, "4040001"),
            (ProfileError::Validation(vec![]), "4220001"),
            (ProfileError::SubmissionLocked, "4090201"),
            (ProfileError::NikUnavailable, "4090202"),
            (ProfileError::DuplicateNikInAccount, "4090001"),
            (ProfileError::TransitionConflict, "4090001"),
            (ProfileError::ProjectionMismatch, "5000001"),
            (ProfileError::File(FileError::TooLarge), "4130001"),
            (ProfileError::File(FileError::UnsupportedType), "4150001"),
            (ProfileError::Database, "5030001"),
            (ProfileError::Internal("x"), "5000001"),
        ];
        for (e, code) in cases {
            assert_eq!(e.response_code().as_str(), code);
        }
    }
}
