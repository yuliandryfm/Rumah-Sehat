//! Amplop ciphertext versi 1 dan AAD v1 (skema fisik §7.1, KEPUTUSAN-104, KEPUTUSAN-130).
//!
//! Amplop: `0x01` ‖ nonce 12 byte ‖ ciphertext (*n* byte) ‖ tag GCM 16 byte = *n* + 29 byte, sehingga
//! CHECK amplop (§4.1 butir 5: minimal 29 byte, byte pertama 1) selalu terpenuhi oleh ciphertext sah.
//!
//! AAD v1: lima bidang berurutan, masing-masing diawali panjang 2 byte big-endian: label `rs.aad.v1`,
//! konteks enkripsi (registri [`EncryptionContext`]), sistem (`rs`/`rme`), ID baris (16 byte UUID),
//! dan `data_key_id` (16 byte UUID). Ciphertext yang dipindah ke baris, kolom, sistem, atau DEK lain
//! gagal dibuka.
//!
//! AES-256-GCM lewat `aws_lc_rs::aead::RandomizedNonceKey`: nonce 96 bit dibangkitkan pustaka untuk
//! setiap enkripsi dan dikembalikan ke pemanggil, sehingga kode ini tidak pernah memilih nonce sendiri
//! [SRC-480].

use crate::context::{EncryptionContext, System};
use crate::error::CryptoError;
use aws_lc_rs::aead::{AES_256_GCM, Aad, NONCE_LEN, Nonce, RandomizedNonceKey};
use foundation::{Sensitive, Uuid};
use std::fmt;
use zeroize::Zeroizing;

/// Byte versi amplop v1 (AES-256-GCM, nonce 12 byte, tag 16 byte, AAD v1).
pub const ENVELOPE_V1: u8 = 0x01;
/// Panjang tag GCM (byte).
pub const TAG_LEN: usize = 16;
/// Panjang tetap amplop di luar ciphertext: versi + nonce + tag = 29 byte.
pub const ENVELOPE_OVERHEAD: usize = 1 + NONCE_LEN + TAG_LEN;
/// Panjang DEK AES-256 (byte).
pub const DEK_LEN: usize = 32;
/// Label AAD v1.
pub const AAD_LABEL_V1: &str = "rs.aad.v1";

/// Plaintext hasil dekripsi atau bahan kunci: tersamar di `Debug`/`Display` dan dinolkan saat dibuang.
pub type SecretBytes = Sensitive<Zeroizing<Vec<u8>>>;

/// Membungkus byte rahasia.
pub fn secret_bytes(bytes: Vec<u8>) -> SecretBytes {
    Sensitive::new(Zeroizing::new(bytes))
}

/// Pengikat ciphertext ke tempatnya: konteks kolom, sistem, dan ID baris pemilik kolom.
/// `data_key_id` (bidang 5) diambil dari DEK yang dipakai, sehingga tidak dapat tertukar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AadV1 {
    /// Konteks kolom dari registri.
    pub context: EncryptionContext,
    /// Sistem pemilik data.
    pub system: System,
    /// UUID baris pemilik kolom (dibuat aplikasi sebelum `INSERT`, ADR-002).
    pub row_id: Uuid,
}

impl AadV1 {
    /// Menyusun byte AAD v1 untuk DEK `data_key_id`.
    pub fn encode(&self, data_key_id: Uuid) -> Result<Vec<u8>, CryptoError> {
        let fields: [&[u8]; 5] = [
            AAD_LABEL_V1.as_bytes(),
            self.context.id().as_bytes(),
            self.system.as_str().as_bytes(),
            self.row_id.as_bytes(),
            data_key_id.as_bytes(),
        ];
        let mut out = Vec::with_capacity(fields.iter().map(|f| f.len() + 2).sum());
        for field in fields {
            let len = u16::try_from(field.len()).map_err(|_| CryptoError::InvalidAad)?;
            out.extend_from_slice(&len.to_be_bytes());
            out.extend_from_slice(field);
        }
        Ok(out)
    }
}

/// DEK AES-256-GCM yang terikat ke satu baris `data_key` (ID-nya masuk AAD bidang 5).
/// Bahan kunci hanya ada di konteks aws-lc-rs; `Debug` hanya menampilkan ID.
pub struct DataEncryptionKey {
    data_key_id: Uuid,
    key: RandomizedNonceKey,
}

impl DataEncryptionKey {
    /// Membuat DEK dari 32 byte bahan kunci (DEK dari OpenBao Transit). Pemanggil wajib menolkan
    /// salinan bahan kuncinya sendiri.
    pub fn from_bytes(data_key_id: Uuid, key_bytes: &[u8]) -> Result<Self, CryptoError> {
        if key_bytes.len() != DEK_LEN {
            return Err(CryptoError::InvalidKey);
        }
        let key = RandomizedNonceKey::new(&AES_256_GCM, key_bytes)
            .map_err(|_| CryptoError::InvalidKey)?;
        Ok(Self { data_key_id, key })
    }

    /// ID baris `data_key` pemilik DEK ini.
    pub fn data_key_id(&self) -> Uuid {
        self.data_key_id
    }

    /// Mengenkripsi `plaintext` menjadi amplop v1 yang terikat ke `aad`.
    pub fn seal(&self, aad: &AadV1, plaintext: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let aad_bytes = aad.encode(self.data_key_id)?;
        let mut buf = Zeroizing::new(Vec::with_capacity(plaintext.len() + TAG_LEN));
        buf.extend_from_slice(plaintext);
        let nonce = self
            .key
            .seal_in_place_append_tag(Aad::from(aad_bytes.as_slice()), &mut *buf)
            .map_err(|_| CryptoError::EncryptionFailed)?;
        let mut out = Vec::with_capacity(ENVELOPE_OVERHEAD + plaintext.len());
        out.push(ENVELOPE_V1);
        out.extend_from_slice(nonce.as_ref());
        out.extend_from_slice(&buf);
        Ok(out)
    }

    /// Membuka amplop v1. Gagal bila amplop rusak, versinya lain, atau tag tidak cocok dengan `aad`.
    pub fn open(&self, aad: &AadV1, envelope: &[u8]) -> Result<SecretBytes, CryptoError> {
        let (version, rest) = envelope
            .split_first()
            .ok_or(CryptoError::MalformedCiphertext)?;
        if envelope.len() < ENVELOPE_OVERHEAD {
            return Err(CryptoError::MalformedCiphertext);
        }
        if *version != ENVELOPE_V1 {
            return Err(CryptoError::UnsupportedVersion);
        }
        let (nonce_bytes, sealed) = rest
            .split_at_checked(NONCE_LEN)
            .ok_or(CryptoError::MalformedCiphertext)?;
        let nonce = Nonce::try_assume_unique_for_key(nonce_bytes)
            .map_err(|_| CryptoError::MalformedCiphertext)?;
        let aad_bytes = aad.encode(self.data_key_id)?;
        let mut buf = Zeroizing::new(sealed.to_vec());
        let plain_len = self
            .key
            .open_in_place(nonce, Aad::from(aad_bytes.as_slice()), &mut buf)
            .map_err(|_| CryptoError::DecryptionFailed)?
            .len();
        buf.truncate(plain_len);
        Ok(Sensitive::new(buf))
    }
}

impl fmt::Debug for DataEncryptionKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DataEncryptionKey")
            .field("data_key_id", &self.data_key_id)
            .field("key", &"[REDACTED]")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::System;
    use crate::primitives::fill_random;
    use foundation::new_id;

    fn dek(id: Uuid) -> DataEncryptionKey {
        let mut k = [0u8; DEK_LEN];
        fill_random(&mut k).unwrap();
        DataEncryptionKey::from_bytes(id, &k).unwrap()
    }

    fn aad(row_id: Uuid) -> AadV1 {
        AadV1 {
            context: EncryptionContext::IDENTITY_DOCUMENT_NIK,
            system: System::RuangSehat,
            row_id,
        }
    }

    // NIK sintetis (bukan milik orang nyata): pola 16 digit berulang.
    const NIK: &[u8] = b"3200000000000001";

    #[test]
    fn seal_then_open_returns_plaintext() {
        let k = dek(new_id());
        let a = aad(new_id());
        let ct = k.seal(&a, NIK).unwrap();
        assert_eq!(k.open(&a, &ct).unwrap().expose().as_slice(), NIK);
    }

    #[test]
    fn envelope_layout_is_v1_with_29_byte_overhead() {
        let k = dek(new_id());
        let a = aad(new_id());
        let ct = k.seal(&a, NIK).unwrap();
        assert_eq!(ct.len(), NIK.len() + 29); // NIK 16 digit -> 45 byte (skema fisik §7.1)
        assert_eq!(ct[0], 0x01);
        assert_ne!(&ct[13..29], NIK); // isi tidak dalam teks terbuka
        let empty = k.seal(&a, b"").unwrap();
        assert_eq!(empty.len(), 29); // memenuhi CHECK minimal 29 byte
        assert_eq!(k.open(&a, &empty).unwrap().expose().len(), 0);
    }

    #[test]
    fn nonce_is_fresh_for_each_encryption() {
        let k = dek(new_id());
        let a = aad(new_id());
        let c1 = k.seal(&a, NIK).unwrap();
        let c2 = k.seal(&a, NIK).unwrap();
        assert_ne!(&c1[1..13], &c2[1..13]);
        assert_ne!(c1, c2);
    }

    #[test]
    fn tampered_ciphertext_fails_to_open() {
        let k = dek(new_id());
        let a = aad(new_id());
        let ct = k.seal(&a, NIK).unwrap();
        for i in 1..ct.len() {
            let mut bad = ct.clone();
            bad[i] ^= 0x01;
            assert_eq!(
                k.open(&a, &bad).err(),
                Some(CryptoError::DecryptionFailed),
                "byte {i}"
            );
        }
        let mut truncated = ct.clone();
        truncated.pop();
        assert_eq!(
            k.open(&a, &truncated).err(),
            Some(CryptoError::DecryptionFailed)
        );
    }

    #[test]
    fn ciphertext_moved_to_other_row_fails_to_open() {
        let k = dek(new_id());
        let ct = k.seal(&aad(new_id()), NIK).unwrap();
        assert_eq!(
            k.open(&aad(new_id()), &ct).err(),
            Some(CryptoError::DecryptionFailed)
        );
    }

    #[test]
    fn ciphertext_moved_to_other_column_or_system_fails_to_open() {
        let k = dek(new_id());
        let row = new_id();
        let ct = k.seal(&aad(row), NIK).unwrap();
        let other_column = AadV1 {
            context: EncryptionContext::IDENTITY_VERIFICATION_REQUEST_STATUS_EVENT_NOTE,
            ..aad(row)
        };
        assert_eq!(
            k.open(&other_column, &ct).err(),
            Some(CryptoError::DecryptionFailed)
        );
        let other_system = AadV1 {
            system: System::Rme,
            ..aad(row)
        };
        assert_eq!(
            k.open(&other_system, &ct).err(),
            Some(CryptoError::DecryptionFailed)
        );
    }

    #[test]
    fn ciphertext_under_other_data_key_fails_to_open() {
        // Bahan kunci sama, data_key_id beda: AAD bidang 5 berbeda sehingga tetap gagal.
        let mut raw = [0u8; DEK_LEN];
        fill_random(&mut raw).unwrap();
        let k1 = DataEncryptionKey::from_bytes(new_id(), &raw).unwrap();
        let k2 = DataEncryptionKey::from_bytes(new_id(), &raw).unwrap();
        let a = aad(new_id());
        let ct = k1.seal(&a, NIK).unwrap();
        assert_eq!(k2.open(&a, &ct).err(), Some(CryptoError::DecryptionFailed));
        // DEK lain sama sekali.
        assert_eq!(
            dek(k1.data_key_id()).open(&a, &ct).err(),
            Some(CryptoError::DecryptionFailed)
        );
    }

    #[test]
    fn short_or_unknown_version_envelope_is_rejected() {
        let k = dek(new_id());
        let a = aad(new_id());
        assert_eq!(
            k.open(&a, b"").err(),
            Some(CryptoError::MalformedCiphertext)
        );
        assert_eq!(
            k.open(&a, &[0x01; 28]).err(),
            Some(CryptoError::MalformedCiphertext)
        );
        let mut ct = k.seal(&a, NIK).unwrap();
        ct[0] = 0x02;
        assert_eq!(k.open(&a, &ct).err(), Some(CryptoError::UnsupportedVersion));
        // Teks terbuka 29+ byte yang tidak diawali 0x01 (bug penulisan teks terbuka) ditolak.
        assert_eq!(
            k.open(&a, b"32000000000000013200000000000001").err(),
            Some(CryptoError::UnsupportedVersion)
        );
    }

    #[test]
    fn dek_with_wrong_length_is_rejected() {
        assert_eq!(
            DataEncryptionKey::from_bytes(new_id(), &[0u8; 16]).err(),
            Some(CryptoError::InvalidKey)
        );
        assert_eq!(
            DataEncryptionKey::from_bytes(new_id(), &[0u8; 33]).err(),
            Some(CryptoError::InvalidKey)
        );
    }

    #[test]
    fn aad_v1_encoding_is_length_prefixed_fields() {
        let row = Uuid::from_bytes([0x11; 16]);
        let dk = Uuid::from_bytes([0x22; 16]);
        let enc = aad(row).encode(dk).unwrap();
        let mut expected = Vec::new();
        for f in [
            b"rs.aad.v1".as_slice(),
            b"identity_document.nik_enc".as_slice(),
            b"rs".as_slice(),
            &[0x11; 16],
            &[0x22; 16],
        ] {
            expected.extend_from_slice(&(f.len() as u16).to_be_bytes());
            expected.extend_from_slice(f);
        }
        assert_eq!(enc, expected);
    }

    #[test]
    fn debug_output_does_not_leak_key_or_plaintext() {
        let id = new_id();
        let k = DataEncryptionKey::from_bytes(id, &[0x5a; DEK_LEN]).unwrap();
        assert_eq!(
            format!("{k:?}"),
            format!("DataEncryptionKey {{ data_key_id: {id:?}, key: \"[REDACTED]\" }}")
        );
        let a = aad(new_id());
        let pt = k.open(&a, &k.seal(&a, NIK).unwrap()).unwrap();
        assert_eq!(
            format!("{pt:?} {pt} {pt:#?}"),
            "[REDACTED] [REDACTED] [REDACTED]"
        );
    }
}
