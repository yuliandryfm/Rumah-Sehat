//! Antrean job di PostgreSQL dan kunci job tunggal (F1-T20).
//!
//! - Antrean `app.job` (ADR-007 butir 3, DD A19.4): [`enqueue`] di transaksi pemanggil dengan
//!   deduplikasi, [`claim`] dengan `FOR UPDATE SKIP LOCKED` [SRC-36] dan sewa, penyelesaian dipagari sewa
//!   ([`complete`], [`retry_or_fail`] dengan [`Backoff`] dan `attempt_count`/`max_attempts`, [`fail`]),
//!   serta penyapu sewa kedaluwarsa [`sweep_expired`] (KEPUTUSAN-108, docs/05-physical-schema.md §9.1).
//! - Putaran pekerja [`run_batch`] di atas fungsi-fungsi itu.
//! - Kunci job tunggal [`SingletonLock`] dengan `pg_try_advisory_lock` tingkat sesi di koneksi khusus,
//!   beserta registri kuncinya ([`lock::REGISTRY`], KEPUTUSAN-103, §6.4) [SRC-234].
//!
//! Tabel `job` milik crate ini (`[crates.platform-jobs]` di `apps/rs-backend/module-tables.toml`,
//! coding-standards §1.3). Job wajib idempoten (ADR-007 konsekuensi).

mod backoff;
mod error;
pub mod lock;
mod queue;
mod runner;
mod types;

pub use backoff::Backoff;
pub use error::{JobError, JobErrorKind};
pub use lock::{LockAttempt, LockKey, SingletonLock};
pub use queue::{
    ClaimRequest, ClaimedJob, EnqueueOutcome, Lease, LeaseOutcome, NewJob, RetryOutcome,
    SweepReport, claim, complete, enqueue, fail, retry_or_fail, sweep_expired,
};
pub use runner::{BatchReport, JobOutcome, MAX_BATCH_SIZE, WorkerConfig, run_batch};
pub use types::{
    DedupeKey, ErrorCode, JobId, JobPayload, JobType, MAX_CODE_LEN, MAX_DEDUPE_KEY_LEN,
    MAX_PAYLOAD_ENTRIES, MaxAttempts, PayloadValue, WorkerId,
};
