//! Pustaka kripto Ruang Sehat (modul kritis T1, ADR-013, coding-standards §5.5).
//!
//! - Satu pustaka kripto: aws-lc-rs 1.19.0 untuk AES-256-GCM, HMAC-SHA256, SHA-256, dan RNG
//!   (KEPUTUSAN-74). Hanya crate ini yang bergantung langsung pada aws-lc-rs (KEPUTUSAN-129).
//! - Amplop ciphertext v1 + AAD v1 yang mengikat ciphertext ke konteks kolom, sistem, ID baris, dan
//!   DEK (KEPUTUSAN-104, KEPUTUSAN-130, skema fisik §7.1).
//! - Kunci, DEK, dan plaintext tidak pernah muncul di `Debug`/`Display` maupun galat; crate ini tidak
//!   menulis log.

mod context;
mod envelope;
mod error;
mod primitives;

pub use context::{ContainerType, EncryptionContext, System};
pub use envelope::{
    AAD_LABEL_V1, AadV1, DEK_LEN, DataEncryptionKey, ENVELOPE_OVERHEAD, ENVELOPE_V1, SecretBytes,
    TAG_LEN, secret_bytes,
};
pub use error::CryptoError;
pub use primitives::{
    HMAC_MIN_KEY_LEN, HmacSha256Key, SHA256_LEN, constant_time_eq, fill_random, sha256,
};
