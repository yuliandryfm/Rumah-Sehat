//! Galat crate `object-storage`. Varian tidak pernah membawa kredensial, URL bertanda tangan, isi objek,
//! atau isi respons penyedia (ADR-005, ADR-010), sehingga aman dicatat sebagai jenis galat.

use std::fmt;

/// Galat object storage.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ObjectStorageError {
    /// Konfigurasi tidak valid; membawa nama field (bukan nilainya).
    InvalidConfig(&'static str),
    /// Endpoint `http://` di luar lingkungan lokal (KEPUTUSAN-235).
    InsecureEndpoint,
    /// Operasi server ke endpoint `https://` belum didukung (TLS belum diterapkan, gagal-tertutup).
    TlsUnsupported,
    /// Masukan operasi tidak valid (nama bucket, kunci objek, tipe isi, ukuran, umur URL).
    InvalidInput(&'static str),
    /// Penyedia tidak dapat dihubungi.
    Unavailable,
    /// Penyedia tidak menjawab dalam batas waktu.
    Timeout,
    /// Penyedia menolak permintaan (kode status HTTP saja; isi respons tidak dibawa).
    Rejected {
        /// Kode status HTTP.
        status: u16,
    },
    /// Respons penyedia tidak sesuai bentuk yang diharapkan atau melebihi batas ukuran baca.
    InvalidResponse,
    /// Objek masih memiliki versi setelah penghapusan semua versi.
    VersionsRemain,
    /// Penyusunan tanda tangan gagal (kunci HMAC ditolak crate `crypto`).
    Signing,
}

impl fmt::Display for ObjectStorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidConfig(field) => {
                write!(f, "konfigurasi object storage tidak valid: {field}")
            }
            Self::InsecureEndpoint => {
                f.write_str("endpoint http:// hanya diizinkan di lingkungan lokal")
            }
            Self::TlsUnsupported => {
                f.write_str("operasi object storage lewat https belum didukung")
            }
            Self::InvalidInput(field) => write!(f, "masukan object storage tidak valid: {field}"),
            Self::Unavailable => f.write_str("object storage tidak dapat dihubungi"),
            Self::Timeout => f.write_str("object storage melewati batas waktu"),
            Self::Rejected { status } => {
                write!(f, "object storage menolak permintaan (HTTP {status})")
            }
            Self::InvalidResponse => f.write_str("respons object storage tidak valid"),
            Self::VersionsRemain => f.write_str("versi objek masih tersisa setelah penghapusan"),
            Self::Signing => f.write_str("penyusunan tanda tangan gagal"),
        }
    }
}

impl std::error::Error for ObjectStorageError {}
