//! Galat adaptor SES. Varian tidak pernah membawa kredensial, alamat, isi pesan, atau isi respons SES
//! (isi galat SES dapat menggemakan alamat), sehingga aman dicatat sebagai jenis galat (ADR-005, ADR-010).

use std::fmt;

/// Galat pengiriman atau konfigurasi SES.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SesError {
    /// Konfigurasi tidak valid; isinya nama field, bukan nilai.
    InvalidConfig(&'static str),
    /// Berkas CA tidak dapat dibaca atau tidak memuat sertifikat.
    CaBundle,
    /// Masukan permintaan tidak valid (alamat, ukuran); isinya nama field.
    InvalidInput(&'static str),
    /// Tanda tangan SigV4 tidak dapat disusun.
    Signing,
    /// Koneksi TCP gagal.
    Unavailable,
    /// Jabat tangan TLS gagal (sertifikat tidak dipercaya, nama tidak cocok, protokol).
    Tls,
    /// Tidak ada jawaban dalam batas waktu.
    Timeout,
    /// HTTP 429 (TooManyRequestsException) [SRC-580].
    Throttled,
    /// HTTP 5xx.
    ServerError {
        /// Kode status HTTP.
        status: u16,
    },
    /// HTTP 401/403: kredensial atau tanda tangan ditolak (masalah konfigurasi, bukan isi pesan).
    AuthRejected {
        /// Kode status HTTP.
        status: u16,
    },
    /// HTTP 4xx lain (400 MessageRejected/BadRequest dan sejenisnya) [SRC-580]: pesan ditolak permanen.
    Rejected {
        /// Kode status HTTP.
        status: u16,
    },
    /// Respons 200 tanpa `MessageId` yang valid, atau respons rusak.
    InvalidResponse,
}

impl SesError {
    /// Galat sementara (pantas dicoba ulang oleh pengirim): 429, 5xx, 401/403, jaringan, TLS, batas waktu.
    pub fn is_temporary(&self) -> bool {
        matches!(
            self,
            Self::Unavailable
                | Self::Tls
                | Self::Timeout
                | Self::Throttled
                | Self::ServerError { .. }
                | Self::AuthRejected { .. }
                | Self::InvalidResponse
        )
    }

    /// Label pendek untuk log (tanpa nilai).
    pub fn kind(&self) -> &'static str {
        match self {
            Self::InvalidConfig(_) => "invalid_config",
            Self::CaBundle => "ca_bundle",
            Self::InvalidInput(_) => "invalid_input",
            Self::Signing => "signing",
            Self::Unavailable => "unavailable",
            Self::Tls => "tls",
            Self::Timeout => "timeout",
            Self::Throttled => "throttled",
            Self::ServerError { .. } => "server_error",
            Self::AuthRejected { .. } => "auth_rejected",
            Self::Rejected { .. } => "rejected",
            Self::InvalidResponse => "invalid_response",
        }
    }

    /// Kode status HTTP SES bila ada.
    pub fn status(&self) -> Option<u16> {
        match self {
            Self::ServerError { status }
            | Self::AuthRejected { status }
            | Self::Rejected { status } => Some(*status),
            Self::Throttled => Some(429),
            _ => None,
        }
    }
}

impl fmt::Display for SesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidConfig(name) => write!(f, "konfigurasi SES tidak valid: {name}"),
            Self::InvalidInput(name) => write!(f, "masukan SES tidak valid: {name}"),
            Self::ServerError { status }
            | Self::AuthRejected { status }
            | Self::Rejected { status } => {
                write!(f, "SES menjawab HTTP {status} ({})", self.kind())
            }
            other => write!(f, "galat SES: {}", other.kind()),
        }
    }
}

impl std::error::Error for SesError {}
