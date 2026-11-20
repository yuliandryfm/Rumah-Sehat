//! Galat akses database audit. Hanya jenis, nama statis dari kode, SQLSTATE, dan nama constraint yang
//! dibawa; pesan/DETAIL database dibuang karena dapat memuat isi baris (TD-03, ADR-010 butir 1).

use std::fmt;

/// Jenis galat.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreErrorKind {
    /// Galat database (koneksi, hak, constraint selain kiriman ulang, partisi tidak ada, ...).
    Database,
    /// `23505` pada `uq_audit_event_key_seq`: `seq` sudah dipakai, yaitu relay lain menulis rantai
    /// bersamaan (skema fisik §6.4 butir 5). Batch dibatalkan seluruhnya (KEPUTUSAN-284).
    SeqConflict,
    /// Bentuk kanonis tidak dapat dibentuk (versi tidak dikenal, angka pecahan, presisi jam).
    Canonical,
    /// Nilai di database tidak berbentuk yang diharapkan (mis. hash bukan 32 byte, IP tidak terbaca).
    CorruptRow,
    /// Kepala rantai tidak konsisten (mis. `seq` kepala tidak positif).
    ChainHead,
    /// Jangkar kepala rantai tidak dapat ditulis atau dibaca (F1-T25).
    Anchor,
}

/// Galat akses database audit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreError {
    kind: StoreErrorKind,
    what: &'static str,
    sqlstate: Option<String>,
    constraint: Option<String>,
}

impl StoreError {
    pub(crate) fn new(kind: StoreErrorKind, what: &'static str) -> Self {
        Self {
            kind,
            what,
            sqlstate: None,
            constraint: None,
        }
    }

    pub(crate) fn db(what: &'static str, e: &sqlx::Error) -> Self {
        let (sqlstate, constraint) = match e.as_database_error() {
            Some(d) => (
                d.code().map(|c| c.into_owned()),
                d.constraint().map(str::to_string),
            ),
            None => (None, None),
        };
        Self {
            kind: StoreErrorKind::Database,
            what,
            sqlstate,
            constraint,
        }
    }

    pub(crate) fn with_kind(mut self, kind: StoreErrorKind) -> Self {
        self.kind = kind;
        self
    }

    /// Jenis galat.
    pub fn kind(&self) -> StoreErrorKind {
        self.kind
    }

    /// Tahap atau field (teks statis dari kode).
    pub fn what(&self) -> &'static str {
        self.what
    }

    /// SQLSTATE bila galat database.
    pub fn sqlstate(&self) -> Option<&str> {
        self.sqlstate.as_deref()
    }

    /// Nama constraint bila galat database berasal dari constraint (nama objek skema, bukan data).
    pub fn constraint(&self) -> Option<&str> {
        self.constraint.as_deref()
    }
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "database audit ({:?}) pada {}: SQLSTATE {}, constraint {}",
            self.kind,
            self.what,
            self.sqlstate.as_deref().unwrap_or("-"),
            self.constraint.as_deref().unwrap_or("-")
        )
    }
}

impl std::error::Error for StoreError {}
