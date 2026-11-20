//! Jalur tulis relay ke database audit dengan role `*_audit_writer` (INSERT saja + `audit.fn_chain_head`;
//! DD §0.6 B, skema fisik §6.3–§6.4, KEPUTUSAN-102, KEPUTUSAN-284).
//!
//! Satu batch = satu transaksi di database audit:
//! 1. Kepala rantai dibaca lewat `audit.fn_chain_head()`; rantai kosong = `seq` 0 dan `prev_hash` 32 byte nol.
//! 2. `recorded_at = greatest(now(), head.recorded_at)` sekali per batch (§6.4 butir 3).
//! 3. Per event, di dalam SAVEPOINT (transaksi bersarang sqlx, bukan SQL mentah): `INSERT` ke
//!    `audit_event_key` dengan `seq = head + 1`. Galat `23505` dibedakan lewat **nama constraint**, bukan
//!    teks pesan (KEPUTUSAN-284):
//!    - `pk_audit_event_key`: event sudah tersimpan (kiriman ulang setelah relay gagal di tengah, sebelum
//!      baris outbox terhapus) → ROLLBACK TO SAVEPOINT, event dilewati tanpa menaikkan `seq`;
//!    - constraint lain (mis. `uq_audit_event_key_seq` = relay lain menulis bersamaan) atau galat apa pun →
//!      seluruh batch dibatalkan.
//!
//!    Bila berhasil: `INSERT` `audit_event` (dengan `prev_hash`/`event_hash`) dan `audit_event_subject`,
//!    lalu RELEASE SAVEPOINT.
//! 4. Commit. Penghapusan baris outbox dilakukan pemanggil setelah commit (§6.4 butir 4).
//!
//! Tidak ada `ON CONFLICT` (butuh hak `SELECT`, SRC-470) dan tidak ada `RETURNING`: writer hanya `INSERT`.

use crate::chain::{
    CANONICAL_VERSION, ChainEvent, ChainHash, EventBody, GENESIS_PREV_HASH, event_hash, to_hash,
};
use crate::error::{StoreError, StoreErrorKind};
use foundation::{Uuid, new_id};
use sqlx::{Connection, PgConnection};
use time::OffsetDateTime;

/// Nama constraint PK `audit_event_key`: pelanggarannya = kiriman ulang event yang sama (KEPUTUSAN-284).
pub const RESEND_CONSTRAINT: &str = "pk_audit_event_key";
/// Nama constraint unik `seq`: pelanggarannya = cabang rantai dicegah (skema fisik §6.4 butir 5).
pub const SEQ_CONSTRAINT: &str = "uq_audit_event_key_seq";
/// Batas jumlah event per batch. Setiap event memakai satu SAVEPOINT (subtransaksi); batas ini menjaga
/// jumlah subtransaksi per transaksi tetap kecil.
pub const MAX_BATCH_EVENTS: usize = 64;

/// Kepala rantai: event terakhir.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChainHead {
    /// `seq` terakhir.
    pub seq: i64,
    /// `event_hash` terakhir.
    pub event_hash: ChainHash,
    /// `recorded_at` event terakhir.
    pub recorded_at: OffsetDateTime,
}

/// Hasil satu batch.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AppendReport {
    /// Event yang ditulis di batch ini (urut `seq`).
    pub appended: Vec<Uuid>,
    /// Event yang ternyata sudah tersimpan (kiriman ulang), dilewati.
    pub resent: Vec<Uuid>,
    /// Kepala rantai setelah commit (None bila rantai masih kosong).
    pub head: Option<ChainHead>,
}

impl AppendReport {
    /// Semua event yang boleh dihapus dari outbox: tertulis atau sudah tersimpan sebelumnya.
    pub fn delivered(&self) -> Vec<Uuid> {
        self.appended
            .iter()
            .chain(self.resent.iter())
            .copied()
            .collect()
    }
}

/// Membaca kepala rantai lewat `audit.fn_chain_head()` (akses sempit, §6.3).
pub async fn chain_head(conn: &mut PgConnection) -> Result<Option<ChainHead>, StoreError> {
    let row = sqlx::query!(
        r#"SELECT seq AS "seq!", event_hash AS "event_hash!", recorded_at AS "recorded_at!"
             FROM audit.fn_chain_head()"#
    )
    .fetch_optional(&mut *conn)
    .await
    .map_err(|e| StoreError::db("chain_head", &e))?;
    match row {
        None => Ok(None),
        Some(r) => {
            let event_hash = to_hash(&r.event_hash).ok_or(StoreError::new(
                StoreErrorKind::CorruptRow,
                "chain_head_hash",
            ))?;
            if r.seq < 1 {
                return Err(StoreError::new(StoreErrorKind::ChainHead, "chain_head_seq"));
            }
            Ok(Some(ChainHead {
                seq: r.seq,
                event_hash,
                recorded_at: r.recorded_at,
            }))
        }
    }
}

/// Menulis satu batch event berurutan ke rantai (lihat dokumentasi modul). `events` paling banyak
/// [`MAX_BATCH_EVENTS`]; batch kosong tidak membuka transaksi.
pub async fn append_batch(
    conn: &mut PgConnection,
    events: &[EventBody],
) -> Result<AppendReport, StoreError> {
    if events.len() > MAX_BATCH_EVENTS {
        return Err(StoreError::new(StoreErrorKind::Database, "batch_too_large"));
    }
    if events.is_empty() {
        return Ok(AppendReport {
            head: chain_head(conn).await?,
            ..AppendReport::default()
        });
    }
    let mut tx = Connection::begin(&mut *conn)
        .await
        .map_err(|e| StoreError::db("begin_batch", &e))?;
    let head = chain_head(&mut tx).await?;
    let recorded_at = sqlx::query_scalar!(
        r#"SELECT greatest(now(), $1::timestamptz) AS "recorded_at!""#,
        head.map(|h| h.recorded_at),
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| StoreError::db("recorded_at", &e))?;

    let (mut seq, mut prev) = match head {
        Some(h) => (h.seq, h.event_hash),
        None => (0, GENESIS_PREV_HASH),
    };
    let mut report = AppendReport::default();
    for body in events {
        let next_seq = seq
            .checked_add(1)
            .ok_or(StoreError::new(StoreErrorKind::ChainHead, "seq_overflow"))?;
        let event = ChainEvent {
            seq: next_seq,
            recorded_at,
            canonical_version: CANONICAL_VERSION,
            body: body.clone(),
        };
        let hash = event_hash(&prev, &event)?;

        let mut sp = Connection::begin(&mut *tx)
            .await
            .map_err(|e| StoreError::db("open_savepoint", &e))?;
        if let Err(e) = insert_key(&mut sp, &event).await {
            if is_unique_violation_on(&e, RESEND_CONSTRAINT) {
                sp.rollback()
                    .await
                    .map_err(|e| StoreError::db("rollback_to_savepoint", &e))?;
                report.resent.push(body.event_id);
                continue;
            }
            // Batch batal: savepoint dan transaksi di-drop = rollback.
            let err = StoreError::db("insert_event_key", &e);
            return Err(if is_unique_violation_on(&e, SEQ_CONSTRAINT) {
                err.with_kind(StoreErrorKind::SeqConflict)
            } else {
                err
            });
        }
        insert_event(&mut sp, &event, &prev, &hash).await?;
        insert_subjects(&mut sp, &event).await?;
        sp.commit()
            .await
            .map_err(|e| StoreError::db("release_savepoint", &e))?;
        report.appended.push(body.event_id);
        seq = next_seq;
        prev = hash;
    }
    tx.commit()
        .await
        .map_err(|e| StoreError::db("commit_batch", &e))?;
    report.head = if seq == 0 {
        None
    } else if report.appended.is_empty() {
        head
    } else {
        Some(ChainHead {
            seq,
            event_hash: prev,
            recorded_at,
        })
    };
    Ok(report)
}

fn is_unique_violation_on(e: &sqlx::Error, constraint: &str) -> bool {
    e.as_database_error()
        .is_some_and(|d| d.code().as_deref() == Some("23505") && d.constraint() == Some(constraint))
}

async fn insert_key(conn: &mut PgConnection, e: &ChainEvent) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO audit.audit_event_key (event_id, seq, recorded_at) VALUES ($1, $2, $3)",
        e.body.event_id,
        e.seq,
        e.recorded_at,
    )
    .execute(conn)
    .await?;
    Ok(())
}

async fn insert_event(
    conn: &mut PgConnection,
    e: &ChainEvent,
    prev_hash: &ChainHash,
    event_hash: &ChainHash,
) -> Result<(), StoreError> {
    let b = &e.body;
    sqlx::query!(
        r#"INSERT INTO audit.audit_event (
             event_id, recorded_at, seq, occurred_at, event_code, action_name, actor_type,
             actor_account_id, actor_account_type, actor_roles, actor_partner_id, object_type, object_id,
             object_version, subject_id, subject_count, tenant_id, source_component, source_ip, user_agent,
             app_version, changed_fields, before_values, after_values, disclosure_recipient,
             disclosure_data_categories, outcome, outcome_reason_code, correlation_id, canonical_version,
             prev_hash, event_hash)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18,
                   ($19::text)::inet, $20, $21, $22, $23, $24, $25, $26, $27, $28, $29, $30, $31, $32)"#,
        b.event_id,
        e.recorded_at,
        e.seq,
        b.occurred_at,
        b.event_code,
        b.action_name,
        b.actor_type,
        b.actor_account_id,
        b.actor_account_type,
        b.actor_roles.as_deref(),
        b.actor_partner_id,
        b.object_type,
        b.object_id,
        b.object_version,
        b.subject_id,
        b.subject_count,
        b.tenant_id,
        b.source_component,
        b.source_ip.map(|ip| ip.to_string()),
        b.user_agent,
        b.app_version,
        b.changed_fields.as_deref(),
        b.before_values.as_ref(),
        b.after_values.as_ref(),
        b.disclosure_recipient,
        b.disclosure_data_categories.as_deref(),
        b.outcome,
        b.outcome_reason_code,
        b.correlation_id,
        e.canonical_version,
        prev_hash.as_slice(),
        event_hash.as_slice(),
    )
    .execute(conn)
    .await
    .map_err(|err| StoreError::db("insert_event", &err))?;
    Ok(())
}

async fn insert_subjects(conn: &mut PgConnection, e: &ChainEvent) -> Result<(), StoreError> {
    if e.body.subjects.is_empty() {
        return Ok(());
    }
    let ids: Vec<Uuid> = e.body.subjects.iter().map(|_| new_id()).collect();
    let subject_ids: Vec<Uuid> = e.body.subjects.iter().map(|s| s.subject_id).collect();
    let kinds: Vec<String> = e
        .body
        .subjects
        .iter()
        .map(|s| s.subject_kind.clone())
        .collect();
    sqlx::query!(
        r#"INSERT INTO audit.audit_event_subject (id, recorded_at, event_id, subject_id, subject_kind)
           SELECT u.id, $2, $3, u.subject_id, u.subject_kind
             FROM unnest($1::uuid[], $4::uuid[], $5::text[]) AS u(id, subject_id, subject_kind)"#,
        &ids,
        e.recorded_at,
        e.body.event_id,
        &subject_ids,
        &kinds,
    )
    .execute(conn)
    .await
    .map_err(|err| StoreError::db("insert_event_subject", &err))?;
    Ok(())
}
