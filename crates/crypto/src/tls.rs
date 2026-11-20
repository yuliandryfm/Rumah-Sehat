//! Penyedia kripto TLS untuk rustls (F1-T36): aws-lc-rs, pustaka kripto tunggal proyek (KEPUTUSAN-74).
//!
//! Crate lain membangun `rustls::ServerConfig`/`ClientConfig` dengan `builder_with_provider(...)` memakai
//! penyedia dari fungsi ini, sehingga pemilihan penyedia tidak bergantung pada fitur crate yang aktif di graf
//! (`ServerConfig::builder()` panik bila penyedia tidak dapat ditentukan, [SRC-582]) dan nama aws-lc-rs tetap
//! hanya muncul di crate ini (test arsitektur).

use std::sync::Arc;

/// Penyedia kripto rustls berbasis aws-lc-rs (suite sandi bawaan rustls 0.23.46: TLS 1.3 dan TLS 1.2 AEAD).
pub fn tls_crypto_provider() -> Arc<rustls::crypto::CryptoProvider> {
    Arc::new(rustls::crypto::aws_lc_rs::default_provider())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_has_cipher_suites_and_safe_versions() {
        let p = tls_crypto_provider();
        assert!(!p.cipher_suites.is_empty());
        assert!(
            rustls::ServerConfig::builder_with_provider(p)
                .with_safe_default_protocol_versions()
                .is_ok()
        );
    }
}
