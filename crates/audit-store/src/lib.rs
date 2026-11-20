//! Database audit Ruang Sehat dan RME minimalis (DD B1–B6, ADR-009 butir 3–5), modul kritis T1.
//!
//! - [`chain`]: bentuk kanonis versi 1 dan `event_hash = SHA-256(prev_hash ‖ kanonis)` (ADR-009 butir 5).
//! - [`writer`]: jalur tulis relay dengan role `*_audit_writer` (INSERT saja + `audit.fn_chain_head`),
//!   penanganan kiriman ulang per constraint (KEPUTUSAN-284), `seq` tanpa celah (KEPUTUSAN-102).
//! - [`reader`]: membaca rantai dengan role ber-`SELECT` (`*_audit_verifier`, `*_audit_reader`).
//!
//! Makro query crate ini dicek terhadap database audit (`sqlx.toml`: `AUDIT_DATABASE_URL`), terpisah dari
//! crate `audit` yang query-nya menyasar database aplikasi (`audit_outbox`). Tabel audit dimiliki crate ini
//! (`[crates.audit-store]` di `apps/rs-backend/module-tables.toml`).

pub mod chain;
mod error;
pub mod reader;
pub mod writer;

pub use chain::{
    CANONICAL_VERSION, ChainEvent, ChainHash, EventBody, EventSubject, GENESIS_PREV_HASH, HASH_LEN,
    canonical_bytes, event_hash, truncate_to_micros,
};
pub use error::{StoreError, StoreErrorKind};
pub use reader::{PageCursor, StoredEvent, find_event, key_summary, read_page};
pub use writer::{AppendReport, ChainHead, MAX_BATCH_EVENTS, append_batch, chain_head};
