//! Kunci tanda tangan RSA untuk **test** validasi token (F1-T32, coding-standards §7.2: token uji dari
//! kunci uji, tidak pernah dari lingkungan nyata).
//!
//! Modul ini hanya ada bila fitur `test-keys` aktif. Fitur itu hanya dipasang lewat
//! `[dev-dependencies]` crate lain, sehingga tidak ikut biner produksi. Ditaruh di crate `crypto`
//! karena hanya crate ini yang boleh bergantung langsung pada aws-lc-rs (KEPUTUSAN-129); crate lain
//! membangkitkan kunci uji lewat API ini.
//!
//! API aws-lc-rs 1.19.0 yang dipakai (source crate, `src/rsa/key.rs`, `src/signature.rs`):
//! `rsa::KeyPair::generate(KeySize::Rsa2048)`, `KeyPair::sign(padding, rng, msg, sig)`,
//! `KeyPair::public_modulus_len()`, `signature::KeyPair::public_key()`, dan
//! `From<&PublicKey> for PublicKeyComponents<Vec<u8>>` (fitur `ring-io`, dinyalakan fitur `test-keys`).

use crate::error::CryptoError;
use aws_lc_rs::rand::SystemRandom;
use aws_lc_rs::rsa::{KeyPair, KeySize, PublicKeyComponents};
use aws_lc_rs::signature::{
    KeyPair as _, RSA_PKCS1_SHA256, RSA_PKCS1_SHA384, RSA_PSS_SHA256, RsaEncoding,
};
use std::fmt;

/// Skema tanda tangan RSA yang dapat dibuat kunci uji.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestRsaScheme {
    /// RSASSA-PKCS1-v1_5 SHA-256 (JWS `RS256`).
    Rs256,
    /// RSASSA-PKCS1-v1_5 SHA-384 (JWS `RS384`).
    Rs384,
    /// RSASSA-PSS SHA-256 (JWS `PS256`).
    Ps256,
}

impl TestRsaScheme {
    fn encoding(self) -> &'static dyn RsaEncoding {
        match self {
            Self::Rs256 => &RSA_PKCS1_SHA256,
            Self::Rs384 => &RSA_PKCS1_SHA384,
            Self::Ps256 => &RSA_PSS_SHA256,
        }
    }
}

/// Pasangan kunci RSA 2048 bit yang dibangkitkan saat test berjalan.
pub struct RsaTestKey {
    pair: KeyPair,
}

impl fmt::Debug for RsaTestKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("RsaTestKey([REDACTED])")
    }
}

impl RsaTestKey {
    /// Membangkitkan pasangan kunci baru.
    pub fn generate() -> Result<Self, CryptoError> {
        let pair = KeyPair::generate(KeySize::Rsa2048).map_err(|_| CryptoError::Rng)?;
        Ok(Self { pair })
    }

    /// Menandatangani `msg` dengan skema `scheme`.
    pub fn sign(&self, scheme: TestRsaScheme, msg: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let mut signature = vec![0u8; self.pair.public_modulus_len()];
        self.pair
            .sign(scheme.encoding(), &SystemRandom::new(), msg, &mut signature)
            .map_err(|_| CryptoError::InvalidKey)?;
        Ok(signature)
    }

    /// Modulus publik `n` (big-endian, tanpa nol di depan).
    pub fn modulus(&self) -> Vec<u8> {
        PublicKeyComponents::<Vec<u8>>::from(self.pair.public_key()).n
    }

    /// Eksponen publik `e` (big-endian, tanpa nol di depan).
    pub fn exponent(&self) -> Vec<u8> {
        PublicKeyComponents::<Vec<u8>>::from(self.pair.public_key()).e
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_key_signs_and_exposes_public_components() {
        let k = RsaTestKey::generate().unwrap();
        let sig = k.sign(TestRsaScheme::Rs256, b"pesan").unwrap();
        assert_eq!(sig.len(), 256);
        assert_eq!(k.modulus().len(), 256);
        assert_eq!(k.exponent(), vec![1, 0, 1]);
        assert_eq!(format!("{k:?}"), "RsaTestKey([REDACTED])");
    }
}
