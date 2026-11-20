//! Galat crate `platform-jobs`. `Display` hanya menyebut jenis galat, tidak pernah teks galat
//! database (bisa memuat nilai baris) atau isi job (coding-standards §3.4, §4.1).

use std::fmt;

/// Jenis galat.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobErrorKind {
    /// Kode (`job_type`, kode galat, kunci payload) tidak memenuhi pola `^[a-z][a-z0-9_]{0,63}$`.
    InvalidCode,
    /// Kunci deduplikasi tidak memenuhi pola yang diizinkan.
    InvalidDedupeKey,
    /// Payload bukan objek berisi ID/kode saja, atau melebihi batas jumlah entri.
    InvalidPayload,
    /// `max_attempts` di luar rentang yang diizinkan (minimal 1, CHECK `ck_job_max_attempts`).
    InvalidMaxAttempts,
    /// Konfigurasi pekerja tidak valid (masa sewa, ukuran batch, backoff, daftar jenis job).
    InvalidConfig,
    /// Perhitungan waktu di luar rentang yang didukung.
    TimeOutOfRange,
    /// Kunci advisory ternyata tidak dipegang sesi ini saat dilepas.
    LockNotHeld,
    /// Galat database atau koneksi.
    Database,
}

/// Galat crate `platform-jobs`.
#[derive(Debug)]
pub struct JobError {
    kind: JobErrorKind,
    source: Option<sqlx::Error>,
}

impl JobError {
    pub(crate) fn new(kind: JobErrorKind) -> Self {
        Self { kind, source: None }
    }

    /// Jenis galat.
    pub fn kind(&self) -> JobErrorKind {
        self.kind
    }
}

impl From<sqlx::Error> for JobError {
    fn from(e: sqlx::Error) -> Self {
        Self {
            kind: JobErrorKind::Database,
            source: Some(e),
        }
    }
}

impl fmt::Display for JobError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let what = match self.kind {
            JobErrorKind::InvalidCode => "kode tidak valid",
            JobErrorKind::InvalidDedupeKey => "kunci deduplikasi tidak valid",
            JobErrorKind::InvalidPayload => "payload job tidak valid",
            JobErrorKind::InvalidMaxAttempts => "max_attempts tidak valid",
            JobErrorKind::InvalidConfig => "konfigurasi pekerja job tidak valid",
            JobErrorKind::TimeOutOfRange => "waktu di luar rentang",
            JobErrorKind::LockNotHeld => "kunci advisory tidak dipegang sesi ini",
            JobErrorKind::Database => "galat database",
        };
        write!(f, "platform-jobs: {what}")
    }
}

impl std::error::Error for JobError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_ref()
            .map(|e| e as &(dyn std::error::Error + 'static))
    }
}
