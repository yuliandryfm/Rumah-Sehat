//! TLS server untuk STARTTLS (ADR-026 butir 6): sertifikat relay dari CA internal, TLS 1.2/1.3 rustls dengan
//! penyedia aws-lc-rs dari crate `crypto` [SRC-582], tanpa autentikasi klien.

use rustls_pki_types::pem::PemObject;
use rustls_pki_types::{CertificateDer, PrivateKeyDer};
use std::path::Path;
use std::sync::Arc;
use tokio_rustls::TlsAcceptor;

/// Galat penyiapan TLS (tanpa isi berkas).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TlsSetupError {
    /// Rantai sertifikat tidak terbaca atau kosong.
    Certificate,
    /// Kunci privat tidak terbaca.
    PrivateKey,
    /// Sertifikat dan kunci tidak cocok, atau konfigurasi ditolak rustls.
    Config,
}

impl std::fmt::Display for TlsSetupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Certificate => "sertifikat TLS relay tidak terbaca",
            Self::PrivateKey => "kunci privat TLS relay tidak terbaca",
            Self::Config => "sertifikat dan kunci TLS relay tidak cocok atau ditolak",
        })
    }
}

impl std::error::Error for TlsSetupError {}

/// Membuat penerima TLS dari berkas PEM rantai sertifikat (sertifikat relay lebih dulu) dan kunci privat.
pub fn server_tls(cert_file: &Path, key_file: &Path) -> Result<TlsAcceptor, TlsSetupError> {
    let chain: Vec<CertificateDer<'static>> = CertificateDer::pem_file_iter(cert_file)
        .map_err(|_| TlsSetupError::Certificate)?
        .collect::<Result<_, _>>()
        .map_err(|_| TlsSetupError::Certificate)?;
    if chain.is_empty() {
        return Err(TlsSetupError::Certificate);
    }
    let key = PrivateKeyDer::from_pem_file(key_file).map_err(|_| TlsSetupError::PrivateKey)?;
    let config = rustls::ServerConfig::builder_with_provider(crypto::tls_crypto_provider())
        .with_safe_default_protocol_versions()
        .map_err(|_| TlsSetupError::Config)?
        .with_no_client_auth()
        .with_single_cert(chain, key)
        .map_err(|_| TlsSetupError::Config)?;
    Ok(TlsAcceptor::from(Arc::new(config)))
}
