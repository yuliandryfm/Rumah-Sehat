//! Galat crate `crypto`. Varian tidak pernah membawa kunci, DEK, plaintext, atau ciphertext
//! (ADR-005, coding-standards §5.5), sehingga aman ditulis ke log sebagai jenis galat.

use std::fmt;

/// Galat operasi kriptografi.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CryptoError {
    /// Amplop ciphertext lebih pendek dari 29 byte atau bentuknya rusak.
    MalformedCiphertext,
    /// Byte versi amplop tidak dikenal (bukan `0x01`).
    UnsupportedVersion,
    /// Tag autentikasi tidak cocok: ciphertext diubah, dipindah ke baris/kolom/wadah/DEK lain, atau kunci salah.
    DecryptionFailed,
    /// Bahan kunci tidak valid (panjang salah atau ditolak pustaka).
    InvalidKey,
    /// Pembangkit bilangan acak gagal.
    Rng,
    /// Enkripsi gagal di pustaka.
    EncryptionFailed,
    /// Data tambahan terautentikasi (AAD) tidak dapat disusun (bidang terlalu panjang).
    InvalidAad,
    /// Konfigurasi tidak valid (alamat, token, nama mount/kunci/path, TTL, kapasitas).
    InvalidConfig,
    /// OpenBao tidak dapat dihubungi.
    KmsUnavailable,
    /// OpenBao tidak menjawab dalam batas waktu.
    KmsTimeout,
    /// OpenBao menolak permintaan (kode status HTTP saja; isi respons tidak dibawa).
    KmsRejected {
        /// Kode status HTTP.
        status: u16,
    },
    /// Respons OpenBao tidak sesuai bentuk yang diharapkan.
    KmsInvalidResponse,
    /// `kek_ref` bukan milik KEK yang dikonfigurasi atau versinya tidak cocok dengan DEK terbungkus.
    KekMismatch,
    /// DEK terbungkus tidak berbentuk ciphertext Transit.
    MalformedWrappedKey,
    /// Wadah DEK (`container_type`/`container_id`) atau konteks kolom tidak cocok dengan yang diharapkan.
    ContainerMismatch,
    /// DEK sudah dimusnahkan (crypto-shredding); data wadahnya tidak dapat dibuka lagi.
    KeyShredded,
    /// Versi kunci (misalnya `nik_bidx_key_version`) tidak dikenal di keyring.
    UnknownKeyVersion,
}

impl fmt::Display for CryptoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = match self {
            Self::MalformedCiphertext => "amplop ciphertext rusak",
            Self::UnsupportedVersion => "versi amplop ciphertext tidak dikenal",
            Self::DecryptionFailed => "dekripsi gagal",
            Self::InvalidKey => "bahan kunci tidak valid",
            Self::Rng => "pembangkit bilangan acak gagal",
            Self::EncryptionFailed => "enkripsi gagal",
            Self::InvalidAad => "AAD tidak valid",
            Self::InvalidConfig => "konfigurasi kripto tidak valid",
            Self::KmsUnavailable => "OpenBao tidak dapat dihubungi",
            Self::KmsTimeout => "OpenBao melewati batas waktu",
            Self::KmsRejected { .. } => "OpenBao menolak permintaan",
            Self::KmsInvalidResponse => "respons OpenBao tidak valid",
            Self::KekMismatch => "rujukan KEK tidak cocok",
            Self::MalformedWrappedKey => "DEK terbungkus rusak",
            Self::ContainerMismatch => "wadah DEK tidak cocok",
            Self::KeyShredded => "DEK sudah dimusnahkan",
            Self::UnknownKeyVersion => "versi kunci tidak dikenal",
        };
        f.write_str(msg)
    }
}

impl std::error::Error for CryptoError {}
