//! Primitif dasar lewat aws-lc-rs 1.19.0 [SRC-173] [SRC-480]: RNG, SHA-256, HMAC-SHA256, dan
//! perbandingan waktu-konstan. Tidak ada primitif buatan sendiri (coding-standards §5.5).

use crate::error::CryptoError;
use aws_lc_rs::{constant_time, digest, hmac, rand};
use std::fmt;

/// Panjang keluaran SHA-256 dan HMAC-SHA256 (byte).
pub const SHA256_LEN: usize = 32;

/// Panjang minimum kunci HMAC yang diterima (256 bit).
pub const HMAC_MIN_KEY_LEN: usize = 32;

/// Mengisi `dest` dengan byte acak dari RNG aws-lc-rs (`aws_lc_rs::rand::fill`).
pub fn fill_random(dest: &mut [u8]) -> Result<(), CryptoError> {
    rand::fill(dest).map_err(|_| CryptoError::Rng)
}

/// SHA-256 atas `data` (misalnya hash berantai audit, hash isi permintaan idempotensi).
pub fn sha256(data: &[u8]) -> [u8; SHA256_LEN] {
    let d = digest::digest(&digest::SHA256, data);
    let mut out = [0u8; SHA256_LEN];
    // Digest SHA-256 selalu 32 byte; bila tidak (mustahil), keluaran tetap nol dan test menangkapnya.
    if let Some(src) = d.as_ref().get(..SHA256_LEN) {
        out.copy_from_slice(src);
    }
    out
}

/// Membandingkan dua slice dalam waktu konstan terhadap isinya (tidak konstan terhadap panjang).
pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    constant_time::verify_slices_are_equal(a, b).is_ok()
}

/// Kunci HMAC-SHA256. Bahan kunci hanya ada di konteks aws-lc-rs; `Debug` tersamar.
pub struct HmacSha256Key {
    key: hmac::Key,
}

impl HmacSha256Key {
    /// Membuat kunci dari bahan kunci minimal 32 byte (kunci lebih pendek ditolak).
    pub fn new(key_bytes: &[u8]) -> Result<Self, CryptoError> {
        if key_bytes.len() < HMAC_MIN_KEY_LEN {
            return Err(CryptoError::InvalidKey);
        }
        Ok(Self {
            key: hmac::Key::new(hmac::HMAC_SHA256, key_bytes),
        })
    }

    /// Tag HMAC-SHA256 atas `data`.
    pub fn sign(&self, data: &[u8]) -> [u8; SHA256_LEN] {
        let tag = hmac::sign(&self.key, data);
        let mut out = [0u8; SHA256_LEN];
        if let Some(src) = tag.as_ref().get(..SHA256_LEN) {
            out.copy_from_slice(src);
        }
        out
    }

    /// Memverifikasi tag dalam waktu konstan (`aws_lc_rs::hmac::verify`).
    pub fn verify(&self, data: &[u8], tag: &[u8]) -> Result<(), CryptoError> {
        hmac::verify(&self.key, data, tag).map_err(|_| CryptoError::DecryptionFailed)
    }
}

impl fmt::Debug for HmacSha256Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("HmacSha256Key([REDACTED])")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{b:02x}")).collect()
    }

    fn unhex(s: &str) -> Vec<u8> {
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
            .collect()
    }

    #[test]
    fn sha256_matches_fips_180_2_vectors() {
        // Vektor uji SHA-256 "abc" dan string kosong (FIPS 180-2, contoh B.1).
        assert_eq!(
            hex(&sha256(b"abc")),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            hex(&sha256(b"")),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn hmac_sha256_matches_rfc4231_case_6() {
        // RFC 4231 test case 2 memakai kunci 4 byte ("Jefe") yang di bawah batas minimum kita;
        // karena itu dipakai test case 6 (kunci 131 byte 0xaa).
        let key = vec![0xaa_u8; 131];
        let data = b"Test Using Larger Than Block-Size Key - Hash Key First";
        let k = HmacSha256Key::new(&key).unwrap();
        assert_eq!(
            hex(&k.sign(data)),
            "60e431591ee0b67f0d8a26aacbf5b77f8e0bc6213728c5140546040f0ee37f54"
        );
        assert!(
            k.verify(
                data,
                &unhex("60e431591ee0b67f0d8a26aacbf5b77f8e0bc6213728c5140546040f0ee37f54")
            )
            .is_ok()
        );
    }

    #[test]
    fn hmac_verify_with_wrong_tag_or_data_is_rejected() {
        let k = HmacSha256Key::new(&[7u8; 32]).unwrap();
        let tag = k.sign(b"isi");
        assert!(k.verify(b"isi", &tag).is_ok());
        assert_eq!(
            k.verify(b"isi lain", &tag),
            Err(CryptoError::DecryptionFailed)
        );
        let mut bad = tag;
        bad[0] ^= 1;
        assert_eq!(k.verify(b"isi", &bad), Err(CryptoError::DecryptionFailed));
        assert_eq!(
            k.verify(b"isi", &tag[..31]),
            Err(CryptoError::DecryptionFailed)
        );
        let other = HmacSha256Key::new(&[8u8; 32]).unwrap();
        assert_eq!(
            other.verify(b"isi", &tag),
            Err(CryptoError::DecryptionFailed)
        );
    }

    #[test]
    fn hmac_key_shorter_than_32_bytes_is_rejected() {
        assert_eq!(
            HmacSha256Key::new(&[1u8; 31]).err(),
            Some(CryptoError::InvalidKey)
        );
        assert_eq!(HmacSha256Key::new(&[]).err(), Some(CryptoError::InvalidKey));
    }

    #[test]
    fn hmac_key_debug_is_redacted() {
        let k = HmacSha256Key::new(&[0x41u8; 32]).unwrap();
        let out = format!("{k:?} {k:#?}");
        assert!(out.contains("REDACTED"));
        assert!(!out.contains("41") && !out.contains("AAAA"));
    }

    #[test]
    fn constant_time_eq_compares_content_and_length() {
        assert!(constant_time_eq(b"abc", b"abc"));
        assert!(!constant_time_eq(b"abc", b"abd"));
        assert!(!constant_time_eq(b"abc", b"ab"));
        assert!(constant_time_eq(b"", b""));
    }

    #[test]
    fn fill_random_produces_distinct_values() {
        let mut a = [0u8; 32];
        let mut b = [0u8; 32];
        fill_random(&mut a).unwrap();
        fill_random(&mut b).unwrap();
        assert_ne!(a, b);
        assert_ne!(a, [0u8; 32]);
    }
}
