//! Membaca rantai dari database audit (role `*_audit_verifier` atau `*_audit_reader`, hak `SELECT`;
//! DD §0.6 B). Dipakai verifikasi rantai (F1-T25) dan test relay. Writer tidak dapat memakai modul ini
//! (tanpa hak `SELECT`).

use crate::chain::{ChainEvent, ChainHash, EventBody, EventSubject, to_hash};
use crate::error::{StoreError, StoreErrorKind};
use foundation::Uuid;
use sqlx::PgConnection;
use std::fmt;
use std::net::IpAddr;

/// Event tersimpan beserta hash yang tercatat.
#[derive(Clone, PartialEq)]
pub struct StoredEvent {
    /// Event (kolom DD B1 + subjek DD B3).
    pub event: ChainEvent,
    /// `prev_hash` tersimpan.
    pub prev_hash: ChainHash,
    /// `event_hash` tersimpan.
    pub event_hash: ChainHash,
    /// Baris `audit_event_subject` ber-`event_id` sama tetapi `recorded_at` berbeda dari event-nya.
    /// Relay selalu menulis subjek dengan `recorded_at` event, sehingga nilai > 0 = manipulasi (review B1).
    pub misplaced_subjects: usize,
}

impl fmt::Debug for StoredEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StoredEvent")
            .field("seq", &self.event.seq)
            .field("event_id", &self.event.body.event_id)
            .finish_non_exhaustive()
    }
}

/// Posisi halaman: event sesudah (`seq`, `event_id`) ini. Awal rantai: [`PageCursor::START`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageCursor {
    /// `seq` terakhir yang sudah dibaca.
    pub seq: i64,
    /// `event_id` terakhir yang sudah dibaca (pemecah seri bila ada `seq` ganda hasil manipulasi).
    pub event_id: Uuid,
}

impl PageCursor {
    /// Sebelum event pertama.
    pub const START: Self = Self {
        seq: i64::MIN,
        event_id: Uuid::nil(),
    };
}

/// Satu halaman event urut (`seq`, `event_id`) sesudah `after`, paling banyak `limit` baris.
pub async fn read_page(
    conn: &mut PgConnection,
    after: PageCursor,
    limit: i64,
) -> Result<Vec<StoredEvent>, StoreError> {
    let rows = sqlx::query!(
        r#"SELECT event_id, recorded_at, seq, occurred_at, event_code, action_name, actor_type,
                  actor_account_id, actor_account_type, actor_roles, actor_partner_id, object_type,
                  object_id, object_version, subject_id, subject_count, tenant_id, source_component,
                  host(source_ip) AS source_ip, user_agent, app_version, changed_fields, before_values,
                  after_values, disclosure_recipient, disclosure_data_categories, outcome,
                  outcome_reason_code, correlation_id, canonical_version, prev_hash, event_hash
             FROM audit.audit_event
            WHERE (seq, event_id) > ($1, $2)
            ORDER BY seq, event_id
            LIMIT $3"#,
        after.seq,
        after.event_id,
        limit,
    )
    .fetch_all(&mut *conn)
    .await
    .map_err(|e| StoreError::db("read_events", &e))?;
    if rows.is_empty() {
        return Ok(Vec::new());
    }
    // Subjek dibaca untuk SEMUA event halaman menurut event_id (bukan hanya subject_count > 1), supaya baris
    // subjek sisipan pada event bersubjek ≤ 1 ikut masuk bentuk kanonis dan menggagalkan hash (review B1).
    let ids: Vec<Uuid> = rows.iter().map(|r| r.event_id).collect();
    let subject_rows = sqlx::query!(
        r#"SELECT event_id, recorded_at, subject_id, subject_kind
             FROM audit.audit_event_subject
            WHERE event_id = ANY($1)"#,
        &ids,
    )
    .fetch_all(&mut *conn)
    .await
    .map_err(|e| StoreError::db("read_event_subjects", &e))?;
    let corrupt = |what| StoreError::new(StoreErrorKind::CorruptRow, what);
    let mut out = Vec::with_capacity(rows.len());
    for r in rows {
        let subjects: Vec<EventSubject> = subject_rows
            .iter()
            .filter(|s| s.event_id == r.event_id && s.recorded_at == r.recorded_at)
            .map(|s| EventSubject {
                subject_id: s.subject_id,
                subject_kind: s.subject_kind.clone(),
            })
            .collect();
        let misplaced_subjects = subject_rows
            .iter()
            .filter(|s| s.event_id == r.event_id && s.recorded_at != r.recorded_at)
            .count();
        let source_ip = r
            .source_ip
            .as_deref()
            .map(|s| s.parse::<IpAddr>().map_err(|_| corrupt("source_ip")))
            .transpose()?;
        out.push(StoredEvent {
            misplaced_subjects,
            prev_hash: to_hash(&r.prev_hash).ok_or(corrupt("prev_hash"))?,
            event_hash: to_hash(&r.event_hash).ok_or(corrupt("event_hash"))?,
            event: ChainEvent {
                seq: r.seq,
                recorded_at: r.recorded_at,
                canonical_version: r.canonical_version,
                body: EventBody {
                    event_id: r.event_id,
                    occurred_at: r.occurred_at,
                    event_code: r.event_code,
                    action_name: r.action_name,
                    actor_type: r.actor_type,
                    actor_account_id: r.actor_account_id,
                    actor_account_type: r.actor_account_type,
                    actor_roles: r.actor_roles,
                    actor_partner_id: r.actor_partner_id,
                    object_type: r.object_type,
                    object_id: r.object_id,
                    object_version: r.object_version,
                    subject_id: r.subject_id,
                    subject_count: r.subject_count,
                    tenant_id: r.tenant_id,
                    source_component: r.source_component,
                    source_ip,
                    user_agent: r.user_agent,
                    app_version: r.app_version,
                    changed_fields: r.changed_fields,
                    before_values: r.before_values,
                    after_values: r.after_values,
                    disclosure_recipient: r.disclosure_recipient,
                    disclosure_data_categories: r.disclosure_data_categories,
                    outcome: r.outcome,
                    outcome_reason_code: r.outcome_reason_code,
                    correlation_id: r.correlation_id,
                    subjects,
                },
            },
        });
    }
    Ok(out)
}

/// Satu event menurut `event_id` (None bila tidak ada).
pub async fn find_event(
    conn: &mut PgConnection,
    event_id: Uuid,
) -> Result<Option<StoredEvent>, StoreError> {
    let seq = sqlx::query_scalar!(
        "SELECT seq FROM audit.audit_event_key WHERE event_id = $1",
        event_id
    )
    .fetch_optional(&mut *conn)
    .await
    .map_err(|e| StoreError::db("find_event_key", &e))?;
    let Some(seq) = seq else {
        return Ok(None);
    };
    let after = PageCursor {
        seq: seq.saturating_sub(1),
        event_id: Uuid::max(),
    };
    Ok(read_page(conn, after, 16)
        .await?
        .into_iter()
        .find(|e| e.event.body.event_id == event_id))
}

/// Jumlah baris `audit_event_subject` yatim (tanpa `audit_event` ber-`event_id` sama). Relay selalu menulis subjek bersama event-nya, sehingga baris yatim = manipulasi
/// (review B1).
pub async fn orphan_subjects(conn: &mut PgConnection) -> Result<i64, StoreError> {
    sqlx::query_scalar!(
        r#"SELECT count(*) AS "count!"
             FROM audit.audit_event_subject s
            WHERE NOT EXISTS (SELECT 1 FROM audit.audit_event e WHERE e.event_id = s.event_id)"#
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(|e| StoreError::db("orphan_subjects", &e))
}

/// Ringkasan `audit_event_key`: jumlah baris dan `seq` terbesar.
pub async fn key_summary(conn: &mut PgConnection) -> Result<(i64, Option<i64>), StoreError> {
    let r = sqlx::query!(
        r#"SELECT count(*) AS "count!", max(seq) AS max_seq FROM audit.audit_event_key"#
    )
    .fetch_one(&mut *conn)
    .await
    .map_err(|e| StoreError::db("key_summary", &e))?;
    Ok((r.count, r.max_seq))
}
