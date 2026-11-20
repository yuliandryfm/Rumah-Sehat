//! Galat pustaka audit. Pesan hanya memuat nama field/aturan dari kode, tidak pernah nilai masukan,
//! sehingga galat aman ditulis ke log (ADR-005, ADR-010 butir 1).

use std::fmt;

/// Alasan event ditolak pustaka sebelum ditulis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditErrorKind {
    /// Kode (aksi, jenis objek, peran, alasan, dan sejenisnya) tidak berbentuk snake_case yang sah.
    InvalidCode,
    /// Field wajib PRD §7.2 tidak diisi.
    MissingField,
    /// Nama field termasuk data yang tidak pernah boleh dicatat (PRD §7.3).
    ForbiddenField,
    /// Field tidak ada di daftar izin jenis objeknya (ADR-009 butir 7).
    FieldNotAllowed,
    /// Jenis nilai tidak cocok dengan aturan field (mis. nomor lengkap di field nilai biasa).
    ValueNotAllowed,
    /// Kebijakan objek tidak sah (mis. field klinis dengan nilai, nama terlarang).
    InvalidPolicy,
    /// Kombinasi field tidak sah (mis. tujuan pengiriman di luar E-17, field ganda).
    InvalidCombination,
    /// Nilai teks melebihi batas atau berisi karakter tidak sah.
    InvalidValue,
    /// Serialisasi payload gagal.
    Serialization,
    /// Penulisan ke `audit_outbox` gagal; transaksi dibatalkan (ADR-009 butir 2).
    Write,
    /// Aksi bisnis mengakhiri transaksi pustaka (SQL kendali transaksi mentah); event tidak ditulis.
    TransactionEscaped,
}

/// Galat pustaka audit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditError {
    kind: AuditErrorKind,
    what: &'static str,
}

impl AuditError {
    pub(crate) fn new(kind: AuditErrorKind, what: &'static str) -> Self {
        Self { kind, what }
    }

    /// Jenis galat.
    pub fn kind(&self) -> AuditErrorKind {
        self.kind
    }

    /// Nama field atau aturan yang dilanggar (teks statis dari kode, bukan nilai masukan).
    pub fn what(&self) -> &'static str {
        self.what
    }
}

impl fmt::Display for AuditError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "event audit ditolak ({:?}): {}", self.kind, self.what)
    }
}

impl std::error::Error for AuditError {}
