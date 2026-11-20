//! Kunci akses AWS akun email dari OpenBao KV v2 (ADR-026 butir 5, KEPUTUSAN-79): field `access_key_id`
//! dan `secret_access_key`. Policy OpenBao per proses/host hanya mengizinkan path ini (deploy, bukan repo).

use crate::config::AwsCredentials;
use crate::error::SesError;
use crypto::{CryptoError, OpenBaoConfig};

/// Nama field kunci akses di rahasia KV.
pub const FIELD_ACCESS_KEY_ID: &str = "access_key_id";
/// Nama field kunci rahasia di rahasia KV.
pub const FIELD_SECRET_ACCESS_KEY: &str = "secret_access_key";

/// Galat pemuatan kredensial (tanpa nilai).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CredentialsError {
    /// OpenBao gagal atau menolak (jenis galat crypto).
    OpenBao(CryptoError),
    /// Field wajib tidak ada atau kosong.
    MissingField(&'static str),
    /// Bentuk kredensial tidak valid.
    Invalid(SesError),
}

impl std::fmt::Display for CredentialsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OpenBao(e) => write!(f, "kredensial SES dari OpenBao gagal: {e}"),
            Self::MissingField(n) => write!(f, "kredensial SES: field {n} tidak ada"),
            Self::Invalid(e) => write!(f, "kredensial SES tidak valid: {e}"),
        }
    }
}

impl std::error::Error for CredentialsError {}

/// Memuat dan memvalidasi kunci akses AWS dari `<mount>/data/<path>`.
pub async fn load_from_openbao(
    openbao: &OpenBaoConfig,
    mount: &str,
    path: &str,
) -> Result<AwsCredentials, CredentialsError> {
    let mut secret = crypto::read_kv_secret(openbao, mount, path)
        .await
        .map_err(CredentialsError::OpenBao)?;
    let id = secret
        .take(FIELD_ACCESS_KEY_ID)
        .ok_or(CredentialsError::MissingField(FIELD_ACCESS_KEY_ID))?;
    let key = secret
        .take(FIELD_SECRET_ACCESS_KEY)
        .ok_or(CredentialsError::MissingField(FIELD_SECRET_ACCESS_KEY))?;
    let creds = AwsCredentials {
        access_key_id: id.into_inner(),
        secret_access_key: key,
    };
    creds.validate().map_err(CredentialsError::Invalid)?;
    Ok(creds)
}
