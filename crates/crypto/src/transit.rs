//! KEK di OpenBao Transit: bungkus dan buka DEK (skema fisik §7.3, KEPUTUSAN-106, [SRC-222], [SRC-485]).
//!
//! - DEK baru: `POST /v1/<mount>/datakey/plaintext/<kunci>` dengan `bits` 256; respons memuat
//!   `plaintext` (base64) dan `ciphertext` (`vault:v<N>:...`). Plaintext hanya di memori proses.
//! - Buka DEK: `POST /v1/<mount>/decrypt/<kunci>` dengan `ciphertext`; respons `plaintext` (base64).
//! - `data_key.wrapped_dek` = string ciphertext Transit apa adanya sebagai byte UTF-8 (opaque).
//! - `data_key.kek_ref` = `openbao-transit:<mount>:<kunci>:v<N>`; versi dibaca dari awalan
//!   ciphertext `vault:v<N>:` dan, bila ada, dicocokkan dengan `key_version` di respons (keduanya
//!   teramati di OpenBao 2.7.2, [SRC-487]).

use crate::envelope::{DEK_LEN, SecretBytes, secret_bytes};
use crate::error::CryptoError;
use crate::openbao::{OpenBaoClient, OpenBaoConfig, valid_path};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use foundation::Sensitive;
use http::Method;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::future::Future;
use zeroize::{Zeroize, Zeroizing};

/// Awalan `kek_ref` untuk KEK OpenBao Transit.
pub const KEK_REF_SCHEME: &str = "openbao-transit";

/// DEK baru dari KEK: plaintext (untuk dipakai sekarang) dan bentuk terbungkus (untuk disimpan).
pub struct GeneratedDek {
    /// 32 byte DEK plaintext.
    pub plaintext: SecretBytes,
    /// Ciphertext KEK (opaque) untuk kolom `wrapped_dek`.
    pub wrapped: Sensitive<String>,
    /// Rujukan KEK untuk kolom `kek_ref`.
    pub kek_ref: String,
}

impl fmt::Debug for GeneratedDek {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GeneratedDek")
            .field("plaintext", &self.plaintext)
            .field("wrapped", &self.wrapped)
            .field("kek_ref", &self.kek_ref)
            .finish()
    }
}

/// Layanan KEK (envelope encryption). Implementasi produksi: [`TransitKek`].
pub trait KeyEncryptionKey: Send + Sync {
    /// Membangkitkan DEK 256 bit baru yang terbungkus KEK.
    fn generate_dek(&self) -> impl Future<Output = Result<GeneratedDek, CryptoError>> + Send;

    /// Membuka DEK terbungkus. `kek_ref` diperiksa cocok dengan KEK ini dan dengan versi di `wrapped`.
    fn unwrap_dek(
        &self,
        wrapped: &str,
        kek_ref: &str,
    ) -> impl Future<Output = Result<SecretBytes, CryptoError>> + Send;
}

/// Nama mount dan kunci Transit (misalnya `transit` dan `rs-dek-kek`, skema fisik §7.3).
#[derive(Debug, Clone)]
pub struct TransitConfig {
    /// Nama mount Transit.
    pub mount: String,
    /// Nama kunci Transit (KEK).
    pub key_name: String,
}

/// KEK di OpenBao Transit.
#[derive(Debug)]
pub struct TransitKek {
    client: OpenBaoClient,
    mount: String,
    key_name: String,
}

#[derive(Serialize)]
struct DatakeyRequest {
    bits: u32,
}

#[derive(Serialize)]
struct DecryptRequest<'a> {
    ciphertext: &'a str,
}

#[derive(Deserialize)]
struct Envelope<T> {
    data: T,
}

#[derive(Deserialize)]
struct DatakeyData {
    plaintext: String,
    ciphertext: String,
    /// Ada di respons OpenBao 2.7.2 (teramati, [SRC-487]); bila ada wajib sama dengan versi di awalan.
    key_version: Option<u32>,
}

#[derive(Deserialize)]
struct DecryptData {
    plaintext: String,
}

/// Mengurai versi KEK dari awalan ciphertext Transit `vault:v<N>:`.
fn wrapped_version(wrapped: &str) -> Option<u32> {
    let rest = wrapped.strip_prefix("vault:v")?;
    let (digits, payload) = rest.split_once(':')?;
    if digits.is_empty() || payload.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    digits.parse::<u32>().ok().filter(|v| *v >= 1)
}

/// Mendekode plaintext base64 dari respons menjadi DEK 32 byte; string sumbernya dinolkan.
fn decode_dek(mut b64: String) -> Result<SecretBytes, CryptoError> {
    let decoded = STANDARD.decode(b64.as_bytes()).map(Zeroizing::new);
    b64.zeroize();
    let bytes = decoded.map_err(|_| CryptoError::KmsInvalidResponse)?;
    if bytes.len() != DEK_LEN {
        return Err(CryptoError::KmsInvalidResponse);
    }
    Ok(secret_bytes(bytes.to_vec()))
}

impl TransitKek {
    /// Membuat klien Transit. Nama mount dan kunci hanya `[a-z0-9_-]`.
    pub fn new(openbao: &OpenBaoConfig, transit: TransitConfig) -> Result<Self, CryptoError> {
        if !valid_path(&transit.mount, false) || !valid_path(&transit.key_name, false) {
            return Err(CryptoError::InvalidConfig);
        }
        Ok(Self {
            client: OpenBaoClient::new(openbao)?,
            mount: transit.mount,
            key_name: transit.key_name,
        })
    }

    fn kek_ref_prefix(&self) -> String {
        format!("{KEK_REF_SCHEME}:{}:{}:v", self.mount, self.key_name)
    }

    /// Memeriksa `kek_ref` milik KEK ini dan versinya sama dengan awalan `wrapped`.
    fn check_kek_ref(&self, wrapped: &str, kek_ref: &str) -> Result<(), CryptoError> {
        let version = kek_ref
            .strip_prefix(&self.kek_ref_prefix())
            .ok_or(CryptoError::KekMismatch)?;
        let wrapped_v = wrapped_version(wrapped).ok_or(CryptoError::MalformedWrappedKey)?;
        if version != wrapped_v.to_string() {
            return Err(CryptoError::KekMismatch);
        }
        Ok(())
    }
}

impl KeyEncryptionKey for TransitKek {
    async fn generate_dek(&self) -> Result<GeneratedDek, CryptoError> {
        let path = format!("/v1/{}/datakey/plaintext/{}", self.mount, self.key_name);
        let body = serde_json::to_vec(&DatakeyRequest { bits: 256 })
            .map_err(|_| CryptoError::InvalidConfig)?;
        let raw = self.client.call(Method::POST, &path, Some(body)).await?;
        let parsed: Envelope<DatakeyData> =
            serde_json::from_slice(&raw).map_err(|_| CryptoError::KmsInvalidResponse)?;
        let DatakeyData {
            plaintext,
            ciphertext,
            key_version,
        } = parsed.data;
        let plaintext = decode_dek(plaintext)?;
        let version = wrapped_version(&ciphertext).ok_or(CryptoError::KmsInvalidResponse)?;
        if key_version.is_some_and(|v| v != version) {
            return Err(CryptoError::KmsInvalidResponse);
        }
        Ok(GeneratedDek {
            plaintext,
            wrapped: Sensitive::new(ciphertext),
            kek_ref: format!("{}{version}", self.kek_ref_prefix()),
        })
    }

    async fn unwrap_dek(&self, wrapped: &str, kek_ref: &str) -> Result<SecretBytes, CryptoError> {
        self.check_kek_ref(wrapped, kek_ref)?;
        let path = format!("/v1/{}/decrypt/{}", self.mount, self.key_name);
        let body = serde_json::to_vec(&DecryptRequest {
            ciphertext: wrapped,
        })
        .map_err(|_| CryptoError::InvalidConfig)?;
        let raw = self.client.call(Method::POST, &path, Some(body)).await?;
        let parsed: Envelope<DecryptData> =
            serde_json::from_slice(&raw).map_err(|_| CryptoError::KmsInvalidResponse)?;
        decode_dek(parsed.data.plaintext)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn kek() -> TransitKek {
        TransitKek::new(
            &OpenBaoConfig {
                address: "http://127.0.0.1:18200".into(),
                token: Sensitive::new("uji".into()),
                timeout: Duration::from_secs(1),
            },
            TransitConfig {
                mount: "transit".into(),
                key_name: "rs-dek-kek".into(),
            },
        )
        .unwrap()
    }

    #[test]
    fn wrapped_version_parses_transit_prefix() {
        assert_eq!(wrapped_version("vault:v1:abc"), Some(1));
        assert_eq!(wrapped_version("vault:v12:abc"), Some(12));
        for bad in [
            "vault:v0:abc",
            "vault:v:abc",
            "vault:v1:",
            "vault:vx:abc",
            "bao:v1:abc",
            "abc",
            "",
        ] {
            assert_eq!(wrapped_version(bad), None, "{bad}");
        }
    }

    #[test]
    fn kek_ref_must_match_mount_key_and_wrapped_version() {
        let k = kek();
        assert!(
            k.check_kek_ref("vault:v2:x", "openbao-transit:transit:rs-dek-kek:v2")
                .is_ok()
        );
        assert_eq!(
            k.check_kek_ref("vault:v2:x", "openbao-transit:transit:rs-dek-kek:v1"),
            Err(CryptoError::KekMismatch)
        );
        assert_eq!(
            k.check_kek_ref("vault:v1:x", "openbao-transit:transit:rme-dek-kek:v1"),
            Err(CryptoError::KekMismatch)
        );
        assert_eq!(
            k.check_kek_ref("vault:v1:x", "openbao-transit:other:rs-dek-kek:v1"),
            Err(CryptoError::KekMismatch)
        );
        assert_eq!(
            k.check_kek_ref("tidak-berformat", "openbao-transit:transit:rs-dek-kek:v1"),
            Err(CryptoError::MalformedWrappedKey)
        );
    }

    #[test]
    fn invalid_mount_or_key_name_is_rejected() {
        let o = OpenBaoConfig {
            address: "http://127.0.0.1:18200".into(),
            token: Sensitive::new("uji".into()),
            timeout: Duration::from_secs(1),
        };
        for (m, k) in [
            ("transit/x", "k"),
            ("transit", "../k"),
            ("", "k"),
            ("transit", "K"),
        ] {
            let r = TransitKek::new(
                &o,
                TransitConfig {
                    mount: m.into(),
                    key_name: k.into(),
                },
            );
            assert_eq!(r.err(), Some(CryptoError::InvalidConfig), "{m} {k}");
        }
    }

    #[test]
    fn decode_dek_requires_32_bytes_of_base64() {
        assert_eq!(
            decode_dek(STANDARD.encode([7u8; 32]))
                .unwrap()
                .expose()
                .len(),
            32
        );
        assert_eq!(
            decode_dek(STANDARD.encode([7u8; 16])).err(),
            Some(CryptoError::KmsInvalidResponse)
        );
        assert_eq!(
            decode_dek("bukan base64!".into()).err(),
            Some(CryptoError::KmsInvalidResponse)
        );
    }
}
