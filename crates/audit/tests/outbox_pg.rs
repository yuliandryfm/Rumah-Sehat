//! F1-T23 (ADR-009 butir 2, KEPUTUSAN-15): event audit ditulis ke `audit_outbox` di transaksi
//! pemanggil dengan hak runtime sungguhan (`rs_api`); event yang ditolak membatalkan aksi bisnis.
//! PostgreSQL 18 sekali pakai dari `scripts/db-ephemeral.sh`; tanpa itu test gagal (bukan dilewati).
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use audit::{AuditErrorKind, AuditEvent, EventCode, ObjectRef, Outcome, record};
use common::*;
use foundation::{FixedClock, new_id};
use serde_json::json;
use time::macros::datetime;

fn clock() -> FixedClock {
    FixedClock::new(datetime!(2027-03-01 10:00:00 UTC))
}

#[tokio::test]
async fn event_and_business_change_commit_together() {
    let app = app_pool().await;
    let relay = relay_pool().await;
    let subject = new_id();

    let mut tx = app.begin().await.unwrap();
    let job = insert_business_row(&mut tx, "uji_audit_commit").await;
    let tx = record(tx, valid_event(subject), &clock()).await.unwrap();
    tx.commit().await.unwrap();

    assert!(business_row_exists(&app, job).await);
    let row = sqlx::query!(
        r#"SELECT id, occurred_at, payload FROM app.audit_outbox
           WHERE payload -> 'subjects' -> 0 ->> 'subject_id' = $1"#,
        subject.hyphenated().to_string(),
    )
    .fetch_one(&relay)
    .await
    .unwrap();
    assert_eq!(row.id.get_version_num(), 7);
    assert_eq!(row.occurred_at, datetime!(2027-03-01 10:00:00 UTC));
    let p = row.payload;
    assert_eq!(p["event_id"], json!(row.id.hyphenated().to_string()));
    assert_eq!(p["event_code"], json!("e_07"));
    assert_eq!(p["after_values"]["nik"], json!({"last4": "8901"}));
    assert_eq!(p["after_values"]["status"], json!("verified"));
    assert_eq!(p["source_ip"], json!("198.51.100.23"));
    let text = p.to_string();
    assert!(
        !text.contains(NIK),
        "NIK lengkap bocor ke audit_outbox: {text}"
    );
}

#[tokio::test]
async fn rollback_by_caller_discards_event_too() {
    let app = app_pool().await;
    let relay = relay_pool().await;
    let subject = new_id();

    let mut tx = app.begin().await.unwrap();
    let job = insert_business_row(&mut tx, "uji_audit_rollback").await;
    let tx = record(tx, valid_event(subject), &clock()).await.unwrap();
    tx.rollback().await.unwrap();

    assert!(!business_row_exists(&app, job).await);
    assert_eq!(outbox_count_for_subject(&relay, subject).await, 0);
}

#[tokio::test]
async fn invalid_event_aborts_business_change() {
    let app = app_pool().await;
    let relay = relay_pool().await;
    let subject = new_id();

    // NEG: event tanpa hasil (field wajib PRD §7.2) ditolak; transaksi tidak dikembalikan dan
    // baris bisnis yang sudah ditulis di transaksi itu batal.
    let invalid = AuditEvent::new(
        EventCode::E09,
        code("status_changed"),
        audit::Actor::system(),
        ObjectRef::with_uuid(&CARE_RECIPIENT, subject),
    )
    .subject(audit::Subject::care_recipient(subject))
    .context(&ctx());
    let mut tx = app.begin().await.unwrap();
    let job = insert_business_row(&mut tx, "uji_audit_invalid").await;
    let err = record(tx, invalid, &clock()).await.unwrap_err();
    assert_eq!(err.kind(), AuditErrorKind::MissingField);

    assert!(!business_row_exists(&app, job).await);
    assert_eq!(outbox_count_for_subject(&relay, subject).await, 0);

    // NEG: NIK lengkap sebagai teks di field nilai biasa ditolak dengan cara yang sama.
    let leaky = AuditEvent::new(
        EventCode::E09,
        code("status_changed"),
        audit::Actor::system(),
        ObjectRef::with_uuid(&CARE_RECIPIENT, subject),
    )
    .subject(audit::Subject::care_recipient(subject))
    .change(audit::FieldChange::new(
        "status",
        None,
        Some(audit::Value::text(NIK)),
    ))
    .outcome(Outcome::success())
    .context(&ctx());
    let mut tx = app.begin().await.unwrap();
    let job = insert_business_row(&mut tx, "uji_audit_invalid").await;
    let err = record(tx, leaky, &clock()).await.unwrap_err();
    assert_eq!(err.kind(), AuditErrorKind::ValueNotAllowed);
    assert!(!business_row_exists(&app, job).await);
    assert_eq!(outbox_count_for_subject(&relay, subject).await, 0);
}

#[tokio::test]
async fn app_role_has_insert_only_on_outbox() {
    let app = app_pool().await;
    // NEG: rs_app tanpa SELECT/UPDATE/DELETE (ADR-009 butir 2); karena itu pustaka tidak memakai RETURNING.
    for sql in [
        "SELECT count(*) FROM app.audit_outbox",
        "UPDATE app.audit_outbox SET payload = '{}'::jsonb",
        "DELETE FROM app.audit_outbox",
        "INSERT INTO app.audit_outbox (id, occurred_at, payload) VALUES (gen_random_uuid(), now(), '{}'::jsonb) RETURNING id",
    ] {
        let err = sqlx::raw_sql(sql).execute(&app).await.expect_err(sql);
        let code = err
            .as_database_error()
            .and_then(|e| e.code().map(|c| c.into_owned()));
        assert_eq!(code.as_deref(), Some("42501"), "{sql}: {err}");
    }
}
