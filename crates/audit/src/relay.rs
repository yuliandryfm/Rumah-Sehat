//! Relay audit (F1-T24; ADR-009 butir 3, skema fisik §6.4, KEPUTUSAN-102, KEPUTUSAN-103, KEPUTUSAN-284).
//!
//! Satu relay aktif: kunci advisory tingkat sesi dari registri `platform_jobs::lock` (7001001 untuk
//! `rs-backend audit-relay`) dipegang di koneksi khusus ke database aplikasi dengan login `*_audit_relay`,
//! dan batch outbox dibaca serta dihapus **lewat koneksi yang sama** ([`relay_batch`] menerima
//! [`SingletonLock`]). Bila koneksi putus, kunci lepas dan batch ikut gagal (§6.4 butir 1).
//!
//! Per batch: baca `audit_outbox` urut `created_at, id` → petakan payload (pustaka audit, versi
//! [`PAYLOAD_VERSION`](crate::PAYLOAD_VERSION)) ke kolom DD B1 → tulis ke database audit dalam satu
//! transaksi dengan `seq`/`prev_hash`/`event_hash` (`audit_store::append_batch`, kiriman ulang dilewati per
//! constraint, KEPUTUSAN-284) → setelah commit, hapus baris outbox yang terkirim (§6.4 butir 4). Bila
//! penghapusan gagal, baris yang sama dikirim ulang di batch berikutnya dan dilewati sebagai kiriman ulang.
//!
//! Galat apa pun membatalkan batch: event tetap di outbox, entri security log ditulis, dan putaran
//! [`run_relay`] menyambung ulang setelah jeda. Umur baris outbox tertua dipantau; di atas 5 menit
//! security log `audit_relay_delayed` (ADR-009 butir 3).

use audit_store::{AppendReport, EventBody, EventSubject, StoreErrorKind, append_batch};
use foundation::{Clock, Sensitive, Uuid};
use observability::security::{self, AuditRelayFailure};
use platform_jobs::{LockAttempt, LockKey, SingletonLock};
use serde::Deserialize;
use serde_json::Value as Json;
use sqlx::{Connection, PgConnection};
use std::fmt;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;
use time::OffsetDateTime;

/// Ukuran batch bawaan.
pub const DEFAULT_BATCH_SIZE: usize = 50;
/// Ambang umur baris outbox tertua untuk peringatan keamanan (ADR-009 butir 3).
pub const LAG_ALERT: time::Duration = time::Duration::minutes(5);

/// Galat relay. Tidak membawa isi payload maupun pesan database.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelayError {
    /// Membaca atau menghapus `audit_outbox` gagal (SQLSTATE bila ada).
    Outbox {
        /// SQLSTATE.
        sqlstate: Option<String>,
    },
    /// Payload outbox tidak dapat dipetakan; `field` = nama field (statis).
    InvalidPayload {
        /// ID event (= `audit_outbox.id`).
        event_id: Uuid,
        /// Field yang bermasalah.
        field: &'static str,
    },
    /// Penulisan ke database audit gagal; batch dibatalkan.
    Store(audit_store::StoreError),
    /// Koneksi ke salah satu database gagal.
    Connect {
        /// `app` atau `audit`.
        database: &'static str,
    },
}

impl fmt::Display for RelayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Outbox { sqlstate } => write!(
                f,
                "audit_outbox gagal (SQLSTATE {})",
                sqlstate.as_deref().unwrap_or("-")
            ),
            Self::InvalidPayload { event_id, field } => {
                write!(f, "payload event {event_id} tidak valid: {field}")
            }
            Self::Store(e) => write!(f, "{e}"),
            Self::Connect { database } => write!(f, "koneksi database {database} gagal"),
        }
    }
}

impl std::error::Error for RelayError {}

impl RelayError {
    fn security_stage(&self) -> AuditRelayFailure {
        match self {
            Self::Outbox { .. } => AuditRelayFailure::Outbox,
            Self::InvalidPayload { .. } => AuditRelayFailure::InvalidPayload,
            Self::Store(e) if e.kind() == StoreErrorKind::SeqConflict => {
                AuditRelayFailure::SeqConflict
            }
            Self::Store(_) => AuditRelayFailure::AuditStore,
            Self::Connect { .. } => AuditRelayFailure::Connection,
        }
    }

    fn event_id(&self) -> Option<Uuid> {
        match self {
            Self::InvalidPayload { event_id, .. } => Some(*event_id),
            _ => None,
        }
    }
}

fn outbox_err(e: &sqlx::Error) -> RelayError {
    RelayError::Outbox {
        sqlstate: e
            .as_database_error()
            .and_then(|d| d.code().map(|c| c.into_owned())),
    }
}

/// Hasil satu batch.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RelayBatch {
    /// Baris outbox yang dibaca.
    pub fetched: usize,
    /// Hasil penulisan rantai.
    pub append: AppendReport,
    /// Baris outbox yang terhapus.
    pub deleted: u64,
}

/// Satu batch relay (lihat dokumentasi modul). `lock` = kunci relay beserta koneksinya ke database
/// aplikasi; `audit` = koneksi `*_audit_writer` ke database audit. `batch_size` dibatasi
/// `1..=audit_store::MAX_BATCH_EVENTS`.
pub async fn relay_batch(
    lock: &mut SingletonLock,
    audit: &mut PgConnection,
    batch_size: usize,
) -> Result<RelayBatch, RelayError> {
    let limit = i64::try_from(batch_size.clamp(1, audit_store::MAX_BATCH_EVENTS)).unwrap_or(1);
    let rows = sqlx::query!(
        "SELECT id, occurred_at, payload FROM app.audit_outbox ORDER BY created_at, id LIMIT $1",
        limit,
    )
    .fetch_all(lock.connection())
    .await
    .map_err(|e| outbox_err(&e))?;
    if rows.is_empty() {
        return Ok(RelayBatch::default());
    }
    let mut bodies = Vec::with_capacity(rows.len());
    for r in &rows {
        bodies.push(map_payload(r.id, r.occurred_at, &r.payload)?);
    }
    let append = append_batch(audit, &bodies)
        .await
        .map_err(RelayError::Store)?;
    let delivered = append.delivered();
    let deleted = sqlx::query!(
        "DELETE FROM app.audit_outbox WHERE id = ANY($1)",
        &delivered
    )
    .execute(lock.connection())
    .await
    .map_err(|e| outbox_err(&e))?
    .rows_affected();
    Ok(RelayBatch {
        fetched: rows.len(),
        append,
        deleted,
    })
}

/// Umur baris outbox tertua (None bila outbox kosong).
pub async fn oldest_outbox_age(
    lock: &mut SingletonLock,
    clock: &dyn Clock,
) -> Result<Option<time::Duration>, RelayError> {
    let oldest = sqlx::query_scalar!("SELECT min(created_at) AS oldest FROM app.audit_outbox")
        .fetch_one(lock.connection())
        .await
        .map_err(|e| outbox_err(&e))?;
    Ok(oldest.map(|t| clock.now() - t))
}

/// Payload `audit_outbox` versi 1 (bentuk `crate::event::Payload`). Field tak dikenal ditolak.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PayloadV1 {
    payload_version: u16,
    event_id: String,
    // occurred_at diambil dari kolom outbox (nilai timestamptz yang tersimpan), bukan teks payload.
    #[serde(rename = "occurred_at")]
    _occurred_at: String,
    event_code: String,
    action_name: String,
    actor_type: String,
    actor_account_id: Option<String>,
    actor_account_type: Option<String>,
    actor_roles: Vec<String>,
    actor_partner_id: Option<String>,
    object_type: String,
    object_id: Option<String>,
    object_version: Option<i32>,
    subjects: Vec<PayloadSubjectV1>,
    tenant_id: Option<String>,
    source_component: String,
    source_ip: Option<String>,
    user_agent: Option<String>,
    app_version: Option<String>,
    changed_fields: Option<Vec<String>>,
    before_values: Option<Json>,
    after_values: Option<Json>,
    disclosure_recipient: Option<String>,
    disclosure_data_categories: Option<Vec<String>>,
    outcome: String,
    outcome_reason_code: Option<String>,
    correlation_id: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PayloadSubjectV1 {
    subject_id: String,
    subject_kind: String,
}

/// Memetakan satu baris outbox ke kolom DD B1/B3. `occurred_at` dari kolom outbox (dibulatkan ke
/// mikrodetik); subjek pertama menjadi `subject_id`, dan bila lebih dari satu subjek semuanya juga
/// ditulis ke `audit_event_subject` (DD B1 `subject_count`, B3).
pub fn map_payload(
    id: Uuid,
    occurred_at: OffsetDateTime,
    payload: &Json,
) -> Result<EventBody, RelayError> {
    let bad = |field: &'static str| RelayError::InvalidPayload {
        event_id: id,
        field,
    };
    let p: PayloadV1 = serde_json::from_value(payload.clone()).map_err(|_| bad("payload"))?;
    if p.payload_version != crate::PAYLOAD_VERSION {
        return Err(bad("payload_version"));
    }
    if parse_uuid(&p.event_id).ok_or(bad("event_id"))? != id {
        return Err(bad("event_id"));
    }
    let opt_uuid = |v: &Option<String>, field: &'static str| -> Result<Option<Uuid>, RelayError> {
        v.as_deref()
            .map(|s| parse_uuid(s).ok_or(bad(field)))
            .transpose()
    };
    let mut subjects = Vec::with_capacity(p.subjects.len());
    for s in &p.subjects {
        subjects.push(EventSubject {
            subject_id: parse_uuid(&s.subject_id).ok_or(bad("subject_id"))?,
            subject_kind: s.subject_kind.clone(),
        });
    }
    let subject_count = i32::try_from(subjects.len()).map_err(|_| bad("subjects"))?;
    let subject_id = subjects.first().map(|s| s.subject_id);
    let source_ip = p
        .source_ip
        .as_deref()
        .map(|s| s.parse::<IpAddr>().map_err(|_| bad("source_ip")))
        .transpose()?;
    Ok(EventBody {
        event_id: id,
        occurred_at: audit_store::truncate_to_micros(occurred_at),
        event_code: p.event_code,
        action_name: p.action_name,
        actor_type: p.actor_type,
        actor_account_id: opt_uuid(&p.actor_account_id, "actor_account_id")?,
        actor_account_type: p.actor_account_type,
        actor_roles: (!p.actor_roles.is_empty()).then_some(p.actor_roles),
        actor_partner_id: opt_uuid(&p.actor_partner_id, "actor_partner_id")?,
        object_type: p.object_type,
        object_id: p.object_id,
        object_version: p.object_version,
        subject_id,
        subject_count,
        tenant_id: opt_uuid(&p.tenant_id, "tenant_id")?,
        source_component: p.source_component,
        source_ip,
        user_agent: p.user_agent,
        app_version: p.app_version,
        changed_fields: p.changed_fields,
        before_values: p.before_values,
        after_values: p.after_values,
        disclosure_recipient: p.disclosure_recipient,
        disclosure_data_categories: p.disclosure_data_categories,
        outcome: p.outcome,
        outcome_reason_code: p.outcome_reason_code,
        correlation_id: parse_uuid(&p.correlation_id).ok_or(bad("correlation_id"))?,
        subjects: if subjects.len() > 1 {
            subjects
        } else {
            Vec::new()
        },
    })
}

fn parse_uuid(s: &str) -> Option<Uuid> {
    Uuid::parse_str(s).ok()
}

/// Pengaturan [`run_relay`].
#[derive(Debug, Clone)]
pub struct RelaySettings {
    /// Kunci relay dari registri (`platform_jobs::lock::RS_AUDIT_RELAY` / `RME_AUDIT_RELAY`).
    pub lock_key: LockKey,
    /// URL login `*_audit_relay` ke database aplikasi.
    pub app_database_url: Sensitive<String>,
    /// URL login `*_audit_writer` ke database audit.
    pub audit_database_url: Sensitive<String>,
    /// Ukuran batch (1..=64).
    pub batch_size: usize,
    /// Jeda saat outbox kosong.
    pub poll_interval: Duration,
    /// Jeda sebelum mencoba ulang setelah galat atau saat kunci dipegang instans lain.
    pub retry_interval: Duration,
}

/// Putaran relay: ambil kunci (instans siaga mencoba ulang berkala), lalu proses batch terus-menerus.
/// Tidak pernah selesai; dihentikan pemanggil (sinyal proses). Galat dicatat ke security log dan
/// application log, koneksi ditutup, lalu dicoba ulang setelah `retry_interval`.
pub async fn run_relay(settings: RelaySettings, clock: Arc<dyn Clock>) {
    loop {
        if let Err(e) = relay_session(&settings, clock.as_ref()).await {
            security::audit_relay_failed(e.security_stage(), e.event_id());
            tracing::error!(error = %e, "audit_relay_session_failed");
        }
        tokio::time::sleep(settings.retry_interval).await;
    }
}

async fn relay_session(settings: &RelaySettings, clock: &dyn Clock) -> Result<(), RelayError> {
    let mut conn = PgConnection::connect(settings.app_database_url.expose())
        .await
        .map_err(|_| RelayError::Connect { database: "app" })?;
    let mut lock = loop {
        match SingletonLock::try_acquire(conn, settings.lock_key).await {
            Ok(LockAttempt::Acquired(l)) => break l,
            Ok(LockAttempt::Busy(c)) => {
                conn = c;
                tokio::time::sleep(settings.retry_interval).await;
            }
            Err(_) => return Err(RelayError::Connect { database: "app" }),
        }
    };
    let mut audit = PgConnection::connect(settings.audit_database_url.expose())
        .await
        .map_err(|_| RelayError::Connect { database: "audit" })?;
    let mut last_lag_alert: Option<OffsetDateTime> = None;
    loop {
        let batch = relay_batch(&mut lock, &mut audit, settings.batch_size).await?;
        if batch.fetched > 0 {
            tracing::info!(
                fetched = batch.fetched,
                appended = batch.append.appended.len(),
                resent = batch.append.resent.len(),
                deleted = batch.deleted,
                head_seq = batch.append.head.map_or(0, |h| h.seq),
                "audit_relay_batch"
            );
        }
        if let Some(age) = oldest_outbox_age(&mut lock, clock).await?
            && age > LAG_ALERT
        {
            let now = clock.now();
            if last_lag_alert.is_none_or(|t| now - t >= time::Duration::minutes(1)) {
                security::audit_relay_delayed(u64::try_from(age.whole_seconds()).unwrap_or(0));
                last_lag_alert = Some(now);
            }
        }
        if batch.fetched < settings.batch_size.clamp(1, audit_store::MAX_BATCH_EVENTS) {
            tokio::time::sleep(settings.poll_interval).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Actor, AuditContext, AuditEvent, AuditRecord, Code, EventCode, FieldChange, FieldRule,
        ObjectPolicy, ObjectRef, Outcome, Source, SourceComponent, Subject, Value,
    };
    use foundation::{CorrelationId, FixedClock, new_id};
    use serde_json::json;
    use std::net::Ipv6Addr;
    use time::macros::datetime;

    static POLICY: ObjectPolicy =
        ObjectPolicy::non_clinical("care_recipient", &[FieldRule::value("status")]);

    fn code(s: &str) -> Code {
        Code::new(s).unwrap()
    }

    fn record(subjects: usize) -> AuditRecord {
        let ctx = AuditContext::new(
            CorrelationId::new(),
            Source::component(SourceComponent::Backoffice)
                .ip(IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1)))
                .user_agent("uji/1.0"),
        );
        let mut e = AuditEvent::new(
            EventCode::E07,
            code("status_changed"),
            Actor::account(new_id(), code("rs_admin"), Vec::new()),
            ObjectRef::with_uuid(&POLICY, new_id()),
        )
        .change(FieldChange::new(
            "status",
            None,
            Some(Value::code("verified").unwrap()),
        ))
        .outcome(Outcome::success())
        .context(&ctx);
        for _ in 0..subjects {
            e = e.subject(Subject::care_recipient(new_id()));
        }
        e.finish(&FixedClock::new(
            datetime!(2027-03-01 10:00:00.123456789 UTC),
        ))
        .unwrap()
    }

    fn map(rec: &AuditRecord, payload: &Json) -> Result<EventBody, RelayError> {
        map_payload(rec.event_id(), rec.occurred_at(), payload)
    }

    #[test]
    fn payload_maps_to_audit_event_columns() {
        let rec = record(0);
        let b = map(&rec, rec.payload()).unwrap();
        assert_eq!(b.event_id, rec.event_id());
        assert_eq!(b.event_code, "e_07");
        assert_eq!(b.actor_type, "account");
        assert_eq!(b.actor_roles, None, "tanpa peran = NULL");
        assert_eq!(b.subject_count, 0);
        assert_eq!(b.subject_id, None);
        assert_eq!(
            b.source_ip.map(|i| i.to_string()).as_deref(),
            Some("2001:db8::1")
        );
        assert_eq!(b.after_values, Some(json!({"status": "verified"})));
        assert_eq!(b.occurred_at, datetime!(2027-03-01 10:00:00.123456 UTC));
        let multi = record(3);
        let m = map(&multi, multi.payload()).unwrap();
        assert_eq!(m.subject_count, 3);
        assert_eq!(m.subjects.len(), 3);
        assert_eq!(m.subject_id, m.subjects.first().map(|s| s.subject_id));
        let single = record(1);
        let s = map(&single, single.payload()).unwrap();
        assert_eq!((s.subject_count, s.subjects.len()), (1, 0));
        assert!(s.subject_id.is_some());
    }

    #[test]
    fn malformed_payloads_are_rejected_with_field_name() {
        let rec = record(1);
        let with = |k: &str, v: Json| {
            let mut p = rec.payload().clone();
            if let Some(o) = p.as_object_mut() {
                o.insert(k.to_string(), v);
            }
            p
        };
        let field = |p: &Json| match map(&rec, p) {
            Err(RelayError::InvalidPayload { field, .. }) => field,
            other => panic!("seharusnya ditolak, didapat {other:?}"),
        };
        assert_eq!(field(&with("unknown", json!(1))), "payload");
        assert_eq!(field(&with("payload_version", json!(2))), "payload_version");
        assert_eq!(
            field(&with("event_id", json!(new_id().hyphenated().to_string()))),
            "event_id"
        );
        assert_eq!(field(&with("source_ip", json!("bukan-ip"))), "source_ip");
        assert_eq!(field(&with("correlation_id", json!("x"))), "correlation_id");
        assert_eq!(field(&with("tenant_id", json!("x"))), "tenant_id");
        assert_eq!(field(&json!("bukan objek")), "payload");
    }
}
