//! Pengirim pesan yang sudah lolos pemeriksaan relay. Production memakai [`SesSender`] (crate `ses`).

use foundation::Clock;
use std::fmt;
use std::future::Future;
use std::sync::Arc;

/// Pesan keluar: alamat dari amplop/daftar izin dan MIME mentah ber-CRLF. Isi tidak muncul di `Debug`.
pub struct OutboundMessage {
    /// Pengirim (nilai kanonik dari daftar izin).
    pub from: String,
    /// Satu-satunya penerima (dari RCPT).
    pub to: String,
    /// Pesan MIME mentah.
    pub raw: Vec<u8>,
}

impl fmt::Debug for OutboundMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "OutboundMessage({} byte)", self.raw.len())
    }
}

/// Hasil pengiriman. Label galat tanpa nilai; aman dicatat.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SendOutcome {
    /// Diterima penyedia; isinya `MessageId` penyedia.
    Accepted(String),
    /// Galat sementara (balasan SMTP 4xx).
    Temporary {
        /// Jenis galat.
        kind: &'static str,
        /// Kode status HTTP penyedia bila ada.
        status: Option<u16>,
    },
    /// Galat permanen (balasan SMTP 5xx).
    Permanent {
        /// Jenis galat.
        kind: &'static str,
        /// Kode status HTTP penyedia bila ada.
        status: Option<u16>,
    },
}

/// Pengirim pesan.
pub trait MailSender: Send + Sync + 'static {
    /// Mengirim satu pesan secara sinkron (selesai setelah penyedia menjawab atau batas waktu habis).
    fn send(&self, msg: &OutboundMessage) -> impl Future<Output = SendOutcome> + Send;
}

/// Pengirim lewat SES API v2 (ADR-026 butir 3).
pub struct SesSender {
    client: ses::SesClient,
    clock: Arc<dyn Clock>,
}

impl fmt::Debug for SesSender {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SesSender({:?})", self.client)
    }
}

impl SesSender {
    /// Membungkus klien SES.
    pub fn new(client: ses::SesClient, clock: Arc<dyn Clock>) -> Self {
        Self { client, clock }
    }
}

impl MailSender for SesSender {
    async fn send(&self, msg: &OutboundMessage) -> SendOutcome {
        let email = ses::RawEmail {
            from: &msg.from,
            to: &msg.to,
            raw: &msg.raw,
        };
        match self.client.send_raw(&email, self.clock.now()).await {
            Ok(id) => SendOutcome::Accepted(id.as_str().to_string()),
            Err(e) if e.is_temporary() => SendOutcome::Temporary {
                kind: e.kind(),
                status: e.status(),
            },
            Err(e) => SendOutcome::Permanent {
                kind: e.kind(),
                status: e.status(),
            },
        }
    }
}
