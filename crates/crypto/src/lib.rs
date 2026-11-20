//! Pustaka kripto Ruang Sehat (modul kritis T1, ADR-013, coding-standards §5.5).
//!
//! - Satu pustaka kripto: aws-lc-rs 1.19.0 untuk AES-256-GCM, HMAC-SHA256, SHA-256, dan RNG
//!   (KEPUTUSAN-74). Hanya crate ini yang bergantung langsung pada aws-lc-rs (KEPUTUSAN-129).
//! - Amplop ciphertext v1 + AAD v1 yang mengikat ciphertext ke konteks kolom, sistem, ID baris, dan
//!   DEK (KEPUTUSAN-104, KEPUTUSAN-130, skema fisik §7.1).
//! - Wadah DEK (`data_key`) dengan KEK di OpenBao Transit dan cache DEK berbatas waktu
//!   (KEPUTUSAN-79, KEPUTUSAN-106, skema fisik §7.3). Bahan kunci DEK tidak keluar dari crate ini.
//! - Baris `data_key` di database aplikasi ([`insert_data_key`], [`load_data_key`]; F1-T60) dan keyring
//!   yang dapat dipakai sebagai objek trait ([`DataKeyring`]) oleh service modul bisnis.
//! - Blind index NIK HMAC-SHA256 dengan kunci berversi dari OpenBao KV (skema fisik §7.2).
//! - Pembacaan rahasia OpenBao KV v2 untuk kredensial pihak ketiga ([`read_kv_secret`], F1-T36) dan
//!   penyedia kripto TLS rustls berbasis aws-lc-rs ([`tls_crypto_provider`], F1-T36).
//! - Kunci, DEK, dan plaintext tidak pernah muncul di `Debug`/`Display` maupun galat; crate ini tidak
//!   menulis log.

mod blind_index;
mod context;
mod envelope;
mod error;
mod keyring;
mod kv;
mod openbao;
mod primitives;
mod store;
#[cfg(feature = "test-keys")]
pub mod test_keys;
mod tls;
mod transit;

pub use blind_index::{
    BLIND_INDEX_KEY_LEN, BLIND_INDEX_LEN, BlindIndex, InvalidNik, NIK_BIDX_LABEL_V1, NIK_DIGITS,
    Nik, NikBlindIndexKeyring, load_nik_blind_index_keyring,
};
pub use context::{ContainerType, EncryptionContext, System};
pub use envelope::{
    AAD_LABEL_V1, AadV1, DEK_LEN, ENVELOPE_OVERHEAD, ENVELOPE_V1, SecretBytes, TAG_LEN,
    secret_bytes,
};
pub use error::CryptoError;
pub use keyring::{
    Container, DataKey, DataKeyFuture, DataKeyRecord, DataKeyring, Keyring, KeyringConfig,
    NewDataKey,
};
pub use kv::{KvSecret, read_kv_secret};
pub use openbao::OpenBaoConfig;
pub use primitives::{
    HMAC_MIN_KEY_LEN, HmacSha256Key, SHA256_LEN, constant_time_eq, fill_random, sha256,
};
pub use store::{insert_data_key, load_data_key};
pub use tls::tls_crypto_provider;
pub use transit::{GeneratedDek, KEK_REF_SCHEME, KeyEncryptionKey, TransitConfig, TransitKek};
