//! Pembantu test integrasi PostgreSQL. Database disediakan `scripts/db-ephemeral.sh` (PostgreSQL 18
//! sekali pakai, image dipin sama dengan scripts/db-migrations-check.sh, bootstrap db/roles/app.sql,
//! migrasi db/migrations/app). Tanpa variabelnya test GAGAL, tidak dilewati diam-diam.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, dead_code)]

use audit::{
    Actor, AuditContext, AuditEvent, Code, EventCode, FieldChange, FieldRule, ObjectPolicy,
    ObjectRef, Outcome, Source, SourceComponent, Subject, Value,
};
use foundation::{CorrelationId, Sensitive, Uuid, new_id};
use observability::EnvSource;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::net::{IpAddr, Ipv4Addr};

pub const NIK: &str = "3174012345678901";

pub static CARE_RECIPIENT: ObjectPolicy = ObjectPolicy::non_clinical(
    "care_recipient",
    &[FieldRule::value("status"), FieldRule::last4("nik")],
);

async fn pool(var: &str) -> PgPool {
    let env = EnvSource::from_process();
    let url = env.required_secret(var).unwrap_or_else(|_| {
        panic!(
            "{var} tidak diset: test integrasi audit wajib dijalankan lewat \
             `bash scripts/db-ephemeral.sh cargo test ...` (dipakai scripts/verify.sh)"
        )
    });
    PgPoolOptions::new()
        .max_connections(4)
        .connect(url.expose())
        .await
        .unwrap_or_else(|_| panic!("koneksi {var} gagal"))
}

/// Login `rs_api` (anggota `rs_app`): hak runtime sungguhan, `INSERT` saja di `audit_outbox`.
pub async fn app_pool() -> PgPool {
    pool("RS_TEST_DB_APP_URL").await
}

/// Login `rs_audit_relay_login` (anggota `rs_audit_relay`): `SELECT`, `DELETE` di `audit_outbox`.
pub async fn relay_pool() -> PgPool {
    pool("RS_TEST_DB_RELAY_URL").await
}

/// Login `rs_audit_writer_login` di `ruang_sehat_audit` (relay: INSERT + `audit.fn_chain_head`).
pub async fn audit_writer_pool() -> PgPool {
    pool("RS_TEST_DB_AUDIT_WRITER_URL").await
}

/// Login `rs_audit_verifier_login` di `ruang_sehat_audit` (SELECT; untuk memeriksa hasil relay).
pub async fn audit_verifier_pool() -> PgPool {
    pool("RS_TEST_DB_AUDIT_VERIFIER_URL").await
}

/// URL rahasia dari lingkungan test (db-ephemeral).
pub fn secret_url(var: &str) -> Sensitive<String> {
    EnvSource::from_process()
        .required_secret(var)
        .unwrap_or_else(|_| panic!("{var} tidak diset: jalankan lewat scripts/db-ephemeral.sh"))
}

/// Koneksi khusus di luar pool (kunci advisory, koneksi writer relay).
pub async fn connection(var: &str) -> sqlx::PgConnection {
    use sqlx::Connection;
    sqlx::PgConnection::connect(secret_url(var).expose())
        .await
        .unwrap_or_else(|_| panic!("koneksi {var} gagal"))
}

pub fn code(s: &str) -> Code {
    Code::new(s).unwrap()
}

pub fn ctx() -> AuditContext {
    AuditContext::new(
        CorrelationId::new(),
        Source::component(SourceComponent::Backoffice)
            .ip(IpAddr::V4(Ipv4Addr::new(198, 51, 100, 23)))
            .user_agent("Mozilla/5.0 uji"),
    )
}

/// Event E-07 sah dengan NIK lengkap sebagai masukan (wajib keluar 4 digit terakhir).
pub fn valid_event(subject: Uuid) -> AuditEvent {
    AuditEvent::new(
        EventCode::E07,
        code("identity_verification_approved"),
        Actor::account(new_id(), code("rs_admin"), vec![code("verifier")]),
        ObjectRef::with_uuid(&CARE_RECIPIENT, subject),
    )
    .subject(Subject::care_recipient(subject))
    .change(FieldChange::new(
        "status",
        Some(Value::code("pending").unwrap()),
        Some(Value::code("verified").unwrap()),
    ))
    .change(FieldChange::new(
        "nik",
        None,
        Some(Value::digits(Sensitive::new(NIK.to_string()))),
    ))
    .outcome(Outcome::success())
    .context(&ctx())
}

/// Aksi bisnis tiruan di transaksi yang sama: baris `job` (rs_app punya S/I/U/D, DD A19.4).
pub async fn insert_business_row(conn: &mut sqlx::PgConnection, job_type: &str) -> Uuid {
    let id = new_id();
    sqlx::query!(
        "INSERT INTO app.job (id, job_type, payload, run_at, status, attempt_count, max_attempts) \
         VALUES ($1, $2, '{}'::jsonb, now(), 'queued', 0, 1)",
        id,
        job_type,
    )
    .execute(conn)
    .await
    .unwrap();
    id
}

pub async fn business_row_exists(pool: &PgPool, id: Uuid) -> bool {
    sqlx::query_scalar!(r#"SELECT count(*) AS "n!" FROM app.job WHERE id = $1"#, id)
        .fetch_one(pool)
        .await
        .unwrap()
        == 1
}

pub async fn outbox_count_for_subject(relay: &PgPool, subject: Uuid) -> i64 {
    sqlx::query_scalar!(
        r#"SELECT count(*) AS "n!" FROM app.audit_outbox
           WHERE payload -> 'subjects' -> 0 ->> 'subject_id' = $1"#,
        subject.hyphenated().to_string(),
    )
    .fetch_one(relay)
    .await
    .unwrap()
}
