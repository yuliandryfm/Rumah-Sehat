//! Relay SMTP internal untuk email Keycloak (dan Alertmanager) ke Amazon SES API v2 (F1-T36, ADR-026).
//!
//! Satu instans per host, hanya terjangkau dari jaringan Docker khusus (ADR-026 butir 1). Relay:
//! - mewajibkan STARTTLS sebelum MAIL/RCPT/DATA dan menolak data yang dikirim berbarengan dengan STARTTLS;
//! - menerima satu penerima per pesan dan pengirim dari daftar izin saja;
//! - membatasi panjang baris, ukuran pesan, waktu menganggur, jumlah koneksi, dan laju (global serta per
//!   penerima; saat batas per penerima tercapai relay membalas 250 dan membuang pesan);
//! - menolak header `X-SES-*`, `From` yang bukan pengirim amplop, dan `To`/`Cc`/`Bcc` selain penerima amplop;
//! - meneruskan MIME mentah ke SES secara sinkron dan membalas 250 hanya setelah SES menerima; 429/5xx → 4xx,
//!   400 → 5xx; teks balasan tetap ditambah ID korelasi;
//! - tidak mencatat badan, subjek, atau alamat; tidak menyimpan pesan.

pub mod address;
mod io;
pub mod limits;
pub mod message;
mod sender;
mod session;
mod settings;
mod tls;

pub use sender::{MailSender, OutboundMessage, SendOutcome, SesSender};
pub use settings::{
    InvalidSetting, MAX_COMMAND_LINE, MAX_MESSAGE_BYTES_LIMIT, MAX_TEXT_LINE, RelaySettings,
};
pub use tls::{TlsSetupError, server_tls};

use crate::limits::RateLimiter;
use crate::session::Shared;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::sync::Semaphore;
use tokio_rustls::TlsAcceptor;

/// Galat pembuatan relay.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelayError {
    /// Pengaturan tidak valid.
    Settings(InvalidSetting),
    /// RNG untuk kunci batas laju gagal.
    Rng,
}

impl std::fmt::Display for RelayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Settings(InvalidSetting(name)) => {
                write!(f, "pengaturan relay tidak valid: {name}")
            }
            Self::Rng => f.write_str("RNG gagal"),
        }
    }
}

impl std::error::Error for RelayError {}

/// Relay siap melayani.
pub struct Relay<S: MailSender> {
    shared: Arc<Shared<S>>,
    connections: Arc<Semaphore>,
}

impl<S: MailSender> Relay<S> {
    /// Memvalidasi pengaturan dan merakit relay.
    pub fn new(settings: RelaySettings, tls: TlsAcceptor, sender: S) -> Result<Self, RelayError> {
        settings.validate().map_err(RelayError::Settings)?;
        let limiter = RateLimiter::new(settings.global_per_minute, settings.per_recipient_per_hour)
            .map_err(|_| RelayError::Rng)?;
        let connections = Arc::new(Semaphore::new(settings.max_connections));
        Ok(Self {
            shared: Arc::new(Shared {
                settings,
                limiter,
                sender,
                tls,
            }),
            connections,
        })
    }

    /// Menerima koneksi sampai future ini dibatalkan (misalnya oleh sinyal berhenti).
    pub async fn serve(self, listener: TcpListener) {
        let session_timeout = self.shared.settings.session_timeout;
        loop {
            let Ok((mut tcp, _peer)) = listener.accept().await else {
                // Galat accept sementara (misalnya batas berkas terbuka): jeda singkat lalu lanjut.
                tracing::warn!("relay_accept_failed");
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                continue;
            };
            let Ok(permit) = Arc::clone(&self.connections).try_acquire_owned() else {
                tracing::warn!(class = "connection_limit", "relay_session_closed");
                tokio::spawn(async move {
                    let _ = tokio::time::timeout(
                        std::time::Duration::from_secs(2),
                        tcp.write_all(b"421 4.3.2 Service busy, try again later\r\n"),
                    )
                    .await;
                });
                continue;
            };
            let shared = Arc::clone(&self.shared);
            tokio::spawn(async move {
                if tokio::time::timeout(session_timeout, session::handle(tcp, shared))
                    .await
                    .is_err()
                {
                    tracing::info!(class = "session_timeout", "relay_session_closed");
                }
                drop(permit);
            });
        }
    }
}
