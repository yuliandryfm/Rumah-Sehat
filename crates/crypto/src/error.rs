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
        };
        f.write_str(msg)
    }
}

impl std::error::Error for CryptoError {}
