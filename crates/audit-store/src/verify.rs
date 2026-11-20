//! Verifikasi rantai audit (F1-T25; ADR-009 butir 5, DD B5, KEPUTUSAN-141, klarifikasi T-22).
//!
//! Dijalankan deployment `audit-verify` dengan role `*_audit_verifier` (baca seluruh rantai, tulis
//! `audit_chain_verification`), terpisah dari relay yang menulis rantai (DBA5-B01). Langkah:
//! 1. Baca semua jangkar dari penyimpanan WORM ([`AnchorStore`]) **lebih dahulu**, lalu jangkar di tabel
//!    `audit_chain_anchor` dan ringkasan `audit_event_key`. Jangkar selalu ditulis setelah event-nya
//!    ter-commit, sehingga rantai yang dibaca sesudahnya selalu memuat semua `seq` yang dijangkarkan.
//! 2. Hitung ulang seluruh rantai daring urut (`seq`, `event_id`): `seq` harus 1, 2, 3, … tanpa celah dan
//!    tanpa ganda, `prev_hash` = hash event sebelumnya (genesis 32 byte nol), `event_hash` = hasil hitung
//!    ulang, dan hasil hitung ulang di setiap `seq` yang dijangkarkan = hash jangkar.
//! 3. Jangkar atau `audit_event_key` dengan `seq` di atas ujung rantai = ujung rantai dipotong.
//! 4. Hasil (`match`/`mismatch`/`error`, `seq` pertama yang tidak cocok) ditulis ke
//!    `audit_chain_verification`; ketidakcocokan dan galat dikirim ke security log secara mandiri.

use crate::anchor::{AnchorStore, ChainAnchor};
use crate::chain::{CANONICAL_VERSION, ChainHash, GENESIS_PREV_HASH, event_hash, to_hash};
use crate::error::{StoreError, StoreErrorKind};
use crate::partition::check_partition_runway;
use crate::reader::{PageCursor, StoredEvent, key_summary, read_page};
use foundation::{Clock, Sensitive, Uuid, new_id};
use observability::security::{self, AuditChainMismatch, AuditVerifyFailure};
use platform_jobs::{LockAttempt, LockKey, SingletonLock};
use sqlx::{Connection, PgConnection};
use std::sync::Arc;
use std::time::Duration;
use time::OffsetDateTime;

/// Jumlah event per halaman baca.
pub const PAGE_SIZE: i64 = 1_000;

/// Ketidakcocokan pertama.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mismatch {
    /// `seq` pertama yang tidak cocok.
    pub seq: i64,
    /// Alasan.
    pub reason: AuditChainMismatch,
}

/// Pemeriksa rantai murni (tanpa I/O): diberi event urut lalu jangkar.
#[derive(Debug, Clone)]
pub struct ChainChecker {
    expected_seq: i64,
    prev: ChainHash,
    anchors: Vec<(i64, ChainHash)>,
}

impl ChainChecker {
    /// Pemeriksa dengan titik jangkar (`seq`, `event_hash`) dari WORM dan tabel.
    pub fn new(anchors: impl IntoIterator<Item = (i64, ChainHash)>) -> Self {
        let mut anchors: Vec<(i64, ChainHash)> = anchors.into_iter().collect();
        anchors.sort();
        anchors.dedup();
        Self {
            expected_seq: 1,
            prev: GENESIS_PREV_HASH,
            anchors,
        }
    }

    /// `seq` terakhir yang lulus (0 bila belum ada).
    pub fn last_seq(&self) -> i64 {
        self.expected_seq - 1
    }

    /// Memeriksa event berikutnya.
    pub fn feed(&mut self, e: &StoredEvent) -> Result<(), Mismatch> {
        let seq = e.event.seq;
        let fail = |seq, reason| Err(Mismatch { seq, reason });
        if seq < self.expected_seq {
            return fail(seq, AuditChainMismatch::SeqDuplicate);
        }
        if seq > self.expected_seq {
            return fail(self.expected_seq, AuditChainMismatch::SeqGap);
        }
        if e.event.canonical_version != CANONICAL_VERSION {
            return fail(seq, AuditChainMismatch::CanonicalVersion);
        }
        if e.prev_hash != self.prev {
            return fail(seq, AuditChainMismatch::PrevHash);
        }
        let Ok(h) = event_hash(&self.prev, &e.event) else {
            return fail(seq, AuditChainMismatch::CanonicalVersion);
        };
        if h != e.event_hash {
            return fail(seq, AuditChainMismatch::EventHash);
        }
        if self.anchors.iter().any(|(s, a)| *s == seq && *a != h) {
            return fail(seq, AuditChainMismatch::AnchorHash);
        }
        self.prev = h;
        self.expected_seq += 1;
        Ok(())
    }

    /// Akhir rantai: jangkar atau kunci event di atas ujung rantai berarti ujung dipotong.
    pub fn finish(&self, key_max_seq: Option<i64>) -> Result<i64, Mismatch> {
        let last = self.last_seq();
        if self.anchors.iter().any(|(s, _)| *s > last) {
            return Err(Mismatch {
                seq: last + 1,
                reason: AuditChainMismatch::AnchorBeyondChain,
            });
        }
        if key_max_seq.is_some_and(|k| k > last) {
            return Err(Mismatch {
                seq: last + 1,
                reason: AuditChainMismatch::KeyBeyondChain,
            });
        }
        Ok(last)
    }
}

/// Hasil verifikasi (`outcome` DD B5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerifyOutcome {
    /// Rantai utuh dan cocok dengan semua jangkar.
    Match,
    /// Tidak cocok di `seq` tertentu.
    Mismatch(Mismatch),
    /// Verifikasi tidak dapat diselesaikan (jangkar atau database tidak terbaca).
    Error(AuditVerifyFailure),
}

impl VerifyOutcome {
    fn db_value(&self) -> &'static str {
        match self {
            Self::Match => "match",
            Self::Mismatch(_) => "mismatch",
            Self::Error(_) => "error",
        }
    }
}

/// Laporan satu verifikasi.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerificationReport {
    /// Mulai (jam server verifikator).
    pub started_at: OffsetDateTime,
    /// Selesai.
    pub finished_at: OffsetDateTime,
    /// `seq` awal yang diperiksa (1, atau 0 bila rantai kosong).
    pub from_seq: i64,
    /// `seq` terakhir yang diperiksa (pada `mismatch` = `seq` pertama yang tidak cocok).
    pub to_seq: i64,
    /// Jangkar WORM terakhir yang tercakup, bila tercatat juga di `audit_chain_anchor`.
    pub audit_chain_anchor_id: Option<Uuid>,
    /// Jumlah jangkar WORM yang dibaca.
    pub worm_anchors: usize,
    /// Hasil.
    pub outcome: VerifyOutcome,
}

/// Memverifikasi rantai (lihat dokumentasi modul) tanpa menulis hasil.
pub async fn verify_chain(
    conn: &mut PgConnection,
    store: &dyn AnchorStore,
    clock: &dyn Clock,
) -> VerificationReport {
    let started_at = clock.now();
    let mut report = VerificationReport {
        started_at,
        finished_at: started_at,
        from_seq: 0,
        to_seq: 0,
        audit_chain_anchor_id: None,
        worm_anchors: 0,
        outcome: VerifyOutcome::Match,
    };
    let worm: Vec<ChainAnchor> = match store.list().await {
        Ok(v) => v,
        Err(_) => {
            report.outcome = VerifyOutcome::Error(AuditVerifyFailure::AnchorStore);
            report.finished_at = clock.now();
            return report;
        }
    };
    report.worm_anchors = worm.len();
    match check(conn, &worm).await {
        Ok((last, outcome, anchor_id)) => {
            let empty = last == 0 && matches!(outcome, VerifyOutcome::Match);
            report.from_seq = if empty { 0 } else { 1 };
            report.to_seq = match outcome {
                VerifyOutcome::Mismatch(m) => m.seq,
                _ => last,
            };
            report.audit_chain_anchor_id = anchor_id;
            report.outcome = outcome;
        }
        Err(_) => report.outcome = VerifyOutcome::Error(AuditVerifyFailure::Database),
    }
    report.finished_at = clock.now();
    report
}

async fn check(
    conn: &mut PgConnection,
    worm: &[ChainAnchor],
) -> Result<(i64, VerifyOutcome, Option<Uuid>), StoreError> {
    let db_anchors =
        sqlx::query!("SELECT id, seq, event_hash FROM audit.audit_chain_anchor ORDER BY seq")
            .fetch_all(&mut *conn)
            .await
            .map_err(|e| StoreError::db("read_chain_anchors", &e))?;
    let mut points: Vec<(i64, ChainHash)> = worm.iter().map(|a| (a.seq, a.event_hash)).collect();
    for a in &db_anchors {
        let h = to_hash(&a.event_hash)
            .ok_or(StoreError::new(StoreErrorKind::CorruptRow, "anchor_hash"))?;
        points.push((a.seq, h));
    }
    let (_, key_max) = key_summary(conn).await?;
    let mut checker = ChainChecker::new(points);
    let mut cursor = PageCursor::START;
    loop {
        let page = read_page(conn, cursor, PAGE_SIZE).await?;
        if page.is_empty() {
            break;
        }
        for e in &page {
            if let Err(m) = checker.feed(e) {
                return Ok((checker.last_seq(), VerifyOutcome::Mismatch(m), None));
            }
            cursor = PageCursor {
                seq: e.event.seq,
                event_id: e.event.body.event_id,
            };
        }
    }
    match checker.finish(key_max) {
        Err(m) => Ok((checker.last_seq(), VerifyOutcome::Mismatch(m), None)),
        Ok(last) => {
            let latest = worm.iter().filter(|a| a.seq <= last).max_by_key(|a| a.seq);
            let anchor_id = latest.and_then(|w| {
                db_anchors
                    .iter()
                    .find(|d| d.seq == w.seq && to_hash(&d.event_hash) == Some(w.event_hash))
                    .map(|d| d.id)
            });
            Ok((last, VerifyOutcome::Match, anchor_id))
        }
    }
}

/// Menulis hasil ke `audit_chain_verification` (hak `INSERT` `*_audit_verifier`); mengembalikan ID baris.
pub async fn record_verification(
    conn: &mut PgConnection,
    report: &VerificationReport,
) -> Result<Uuid, StoreError> {
    let id = new_id();
    let first_mismatch = match report.outcome {
        VerifyOutcome::Mismatch(m) => Some(m.seq),
        _ => None,
    };
    sqlx::query!(
        r#"INSERT INTO audit.audit_chain_verification
             (id, started_at, finished_at, from_seq, to_seq, audit_chain_anchor_id, outcome, first_mismatch_seq)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"#,
        id,
        report.started_at,
        report.finished_at,
        report.from_seq,
        report.to_seq,
        report.audit_chain_anchor_id,
        report.outcome.db_value(),
        first_mismatch,
    )
    .execute(&mut *conn)
    .await
    .map_err(|e| StoreError::db("insert_chain_verification", &e))?;
    Ok(id)
}

/// Satu putaran lengkap: pantau partisi masa depan (TD-12), verifikasi, catat hasil, kirim security log.
/// Mengembalikan laporan dan ID baris hasil (None bila hasil tidak dapat dicatat).
pub async fn run_verification(
    conn: &mut PgConnection,
    store: &dyn AnchorStore,
    clock: &dyn Clock,
) -> (VerificationReport, Option<Uuid>) {
    if check_partition_runway(conn, clock).await.is_err() {
        security::audit_chain_verification_failed(AuditVerifyFailure::PartitionCheck);
    }
    let report = verify_chain(conn, store, clock).await;
    match report.outcome {
        VerifyOutcome::Match => tracing::info!(
            from_seq = report.from_seq,
            to_seq = report.to_seq,
            worm_anchors = report.worm_anchors,
            "audit_chain_verified"
        ),
        VerifyOutcome::Mismatch(m) => security::audit_chain_mismatch(m.seq, m.reason),
        VerifyOutcome::Error(stage) => security::audit_chain_verification_failed(stage),
    }
    let id = match record_verification(conn, &report).await {
        Ok(id) => Some(id),
        Err(_) => {
            security::audit_chain_verification_failed(AuditVerifyFailure::RecordResult);
            None
        }
    };
    (report, id)
}

/// Pengaturan proses `audit-verify`.
#[derive(Clone)]
pub struct VerifierSettings {
    /// Kunci registri (`platform_jobs::lock::RS_AUDIT_VERIFY` / `RME_AUDIT_VERIFY`).
    pub lock_key: LockKey,
    /// URL login `*_audit_verifier` ke database audit.
    pub database_url: Sensitive<String>,
    /// Penyimpanan jangkar WORM.
    pub anchors: Arc<dyn AnchorStore>,
    /// Jeda antar-verifikasi (harian).
    pub interval: Duration,
    /// Jeda coba ulang setelah galat / saat instans lain memegang kunci.
    pub retry_interval: Duration,
}

impl std::fmt::Debug for VerifierSettings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VerifierSettings")
            .field("lock_key", &self.lock_key)
            .field("interval", &self.interval)
            .finish_non_exhaustive()
    }
}

async fn acquire(settings: &VerifierSettings) -> Result<SingletonLock, StoreError> {
    let conn_err = |_| StoreError::new(StoreErrorKind::Database, "connect");
    let mut conn = PgConnection::connect(settings.database_url.expose())
        .await
        .map_err(conn_err)?;
    loop {
        match SingletonLock::try_acquire(conn, settings.lock_key).await {
            Ok(LockAttempt::Acquired(l)) => return Ok(l),
            Ok(LockAttempt::Busy(c)) => {
                conn = c;
                tokio::time::sleep(settings.retry_interval).await;
            }
            Err(_) => return Err(StoreError::new(StoreErrorKind::Database, "lock")),
        }
    }
}

/// Satu verifikasi dengan kunci tunggal (pemicu manual / `AUDIT_VERIFY_ONCE`).
pub async fn verify_once(
    settings: &VerifierSettings,
    clock: &dyn Clock,
) -> Result<(VerificationReport, Option<Uuid>), StoreError> {
    let mut lock = acquire(settings).await?;
    let out = run_verification(lock.connection(), settings.anchors.as_ref(), clock).await;
    let _released = lock.release().await;
    Ok(out)
}

/// Putaran harian: satu verifikasi per `interval` selama proses hidup; dihentikan pemanggil.
pub async fn run_verifier(settings: VerifierSettings, clock: Arc<dyn Clock>) {
    loop {
        match acquire(&settings).await {
            Ok(mut lock) => loop {
                let (_, recorded) =
                    run_verification(lock.connection(), settings.anchors.as_ref(), clock.as_ref())
                        .await;
                if recorded.is_none() {
                    // Koneksi kemungkinan putus: sambung ulang (kunci ikut lepas).
                    break;
                }
                tokio::time::sleep(settings.interval).await;
            },
            Err(_) => security::audit_chain_verification_failed(AuditVerifyFailure::Database),
        }
        tokio::time::sleep(settings.retry_interval).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chain::{ChainEvent, EventBody};
    use time::macros::datetime;

    fn body(n: u128) -> EventBody {
        EventBody {
            event_id: Uuid::from_u128(n),
            occurred_at: datetime!(2027-03-01 10:00:00 UTC),
            event_code: "e_09".into(),
            action_name: "status_changed".into(),
            actor_type: "system".into(),
            actor_account_id: None,
            actor_account_type: None,
            actor_roles: None,
            actor_partner_id: None,
            object_type: "job".into(),
            object_id: None,
            object_version: None,
            subject_id: None,
            subject_count: 0,
            tenant_id: None,
            source_component: "worker".into(),
            source_ip: None,
            user_agent: None,
            app_version: None,
            changed_fields: None,
            before_values: None,
            after_values: None,
            disclosure_recipient: None,
            disclosure_data_categories: None,
            outcome: "success".into(),
            outcome_reason_code: None,
            correlation_id: Uuid::from_u128(1000 + n),
            subjects: Vec::new(),
        }
    }

    /// Rantai sah n event (seperti ditulis relay).
    fn chain(n: i64) -> Vec<StoredEvent> {
        let mut prev = GENESIS_PREV_HASH;
        (1..=n)
            .map(|seq| {
                let event = ChainEvent {
                    seq,
                    recorded_at: datetime!(2027-03-01 10:00:01 UTC),
                    canonical_version: CANONICAL_VERSION,
                    body: body(seq as u128),
                };
                let h = event_hash(&prev, &event).unwrap();
                let e = StoredEvent {
                    event,
                    prev_hash: prev,
                    event_hash: h,
                };
                prev = h;
                e
            })
            .collect()
    }

    fn run(
        events: &[StoredEvent],
        anchors: Vec<(i64, ChainHash)>,
        key_max: Option<i64>,
    ) -> Result<i64, Mismatch> {
        let mut c = ChainChecker::new(anchors);
        for e in events {
            c.feed(e)?;
        }
        c.finish(key_max)
    }

    fn mismatch(seq: i64, reason: AuditChainMismatch) -> Result<i64, Mismatch> {
        Err(Mismatch { seq, reason })
    }

    #[test]
    fn intact_chain_matches_with_anchors() {
        let c = chain(5);
        let anchors = vec![(3, c[2].event_hash), (5, c[4].event_hash)];
        assert_eq!(run(&c, anchors, Some(5)), Ok(5));
        assert_eq!(run(&[], vec![], None), Ok(0));
    }

    #[test]
    fn modified_field_is_detected_at_its_seq() {
        let mut c = chain(5);
        c[2].event.body.action_name = "diubah".into();
        assert_eq!(
            run(&c, vec![], Some(5)),
            mismatch(3, AuditChainMismatch::EventHash)
        );
    }

    #[test]
    fn removed_middle_event_is_a_gap_at_its_seq() {
        let mut c = chain(5);
        c.remove(2);
        assert_eq!(
            run(&c, vec![], Some(5)),
            mismatch(3, AuditChainMismatch::SeqGap)
        );
    }

    #[test]
    fn replaced_prev_hash_and_duplicate_seq_are_detected() {
        let mut c = chain(4);
        c[1].prev_hash = [7u8; 32];
        assert_eq!(
            run(&c, vec![], None),
            mismatch(2, AuditChainMismatch::PrevHash)
        );
        let mut d = chain(4);
        let dup = d[1].clone();
        d.insert(2, dup);
        assert_eq!(
            run(&d, vec![], None),
            mismatch(2, AuditChainMismatch::SeqDuplicate)
        );
    }

    #[test]
    fn consistently_rewritten_chain_is_caught_by_anchor() {
        // Penyerang mengubah seq 2 lalu menghitung ulang seluruh rantai sesudahnya; hanya jangkar WORM
        // yang menangkapnya (ADR-009 butir 5).
        let original = chain(4);
        let mut forged = chain(4);
        forged[1].event.body.action_name = "dipalsukan".into();
        let mut prev = GENESIS_PREV_HASH;
        for e in forged.iter_mut() {
            e.prev_hash = prev;
            e.event_hash = event_hash(&prev, &e.event).unwrap();
            prev = e.event_hash;
        }
        assert_eq!(
            run(&forged, vec![], Some(4)),
            Ok(4),
            "tanpa jangkar tidak terdeteksi"
        );
        let anchors = vec![(4, original[3].event_hash)];
        assert_eq!(
            run(&forged, anchors, Some(4)),
            mismatch(4, AuditChainMismatch::AnchorHash)
        );
    }

    #[test]
    fn truncated_tail_is_caught_by_anchor_or_event_key() {
        let c = chain(5);
        let anchors = vec![(5, c[4].event_hash)];
        assert_eq!(
            run(&c[..4], anchors, Some(4)),
            mismatch(5, AuditChainMismatch::AnchorBeyondChain)
        );
        assert_eq!(
            run(&c[..4], vec![], Some(5)),
            mismatch(5, AuditChainMismatch::KeyBeyondChain)
        );
    }

    #[test]
    fn unknown_canonical_version_is_a_mismatch() {
        let mut c = chain(2);
        c[1].event.canonical_version = 9;
        assert_eq!(
            run(&c, vec![], None),
            mismatch(2, AuditChainMismatch::CanonicalVersion)
        );
    }
}
