//! Adaptor Amazon SES API v2 (F1-T36; ADR-026, ADR-016 butir 5).
//!
//! - [`SesClient::send_raw`]: `SendEmail` dengan isi Raw, `Destination.ToAddresses` satu penerima,
//!   `FromEmailAddress` dan `ConfigurationSetName` dari pemanggil, pelacakan klik/buka dimatikan lewat
//!   `ConfigurationOverrides.Tracking` [SRC-580]; tanpa `EndpointId` (ADR-026 butir 4).
//! - Region selalu `ap-southeast-3`; endpoint lain hanya untuk tiruan SES lokal, selalu HTTPS ([`SesConfig`]).
//! - Tanda tangan SigV4 dari primitif crate `crypto` (vektor botocore [SRC-581]); TLS rustls dengan penyedia
//!   aws-lc-rs dari `crypto::tls_crypto_provider` [SRC-582].
//! - Kredensial dari OpenBao KV v2 ([`load_from_openbao`]).
//!
//! Kredensial, alamat, dan isi pesan tidak pernah ditulis ke `Debug`, galat, atau log; crate ini tidak menulis log.

mod client;
mod config;
mod credentials;
mod error;
mod sigv4;

pub use client::{MAX_RAW_BYTES, MessageId, RawEmail, SesClient};
pub use config::{AwsCredentials, DEFAULT_HOST, MAX_TIMEOUT, REGION, SECRET_MIN_LEN, SesConfig};
pub use credentials::{
    CredentialsError, FIELD_ACCESS_KEY_ID, FIELD_SECRET_ACCESS_KEY, load_from_openbao,
};
pub use error::SesError;
