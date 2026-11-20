//! F1-T23 (ADR-009 butir 2, KEPUTUSAN-15): `in_audited_tx` memegang transaksi tingkat atas; event audit
//! ditulis ke `audit_outbox` di transaksi yang sama dengan aksi bisnis, dengan hak runtime sungguhan
//! (`rs_api`); event yang ditolak, aksi yang gagal, dan percobaan lolos lewat savepoint atau SQL kendali
//! transaksi mentah tidak menghasilkan aksi tanpa audit (review keamanan BLOKIR-2).
//! PostgreSQL 18 sekali pakai dari `scripts/db-ephemeral.sh`; tanpa itu test gagal (bukan dilewati).
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use audit::{
    Actor, AuditErrorKind, AuditEvent, AuditedTxError, EventCode, FieldChange, ObjectRef, Outcome,
    Subject, Value, in_audited_tx,
};
use common::*;
use foundation::{FixedClock, new_id};
use serde_json::json;
use sqlx::Connection;
use std::convert::Infallible;
use time::macros::datetime;

fn clock() -> FixedClock {
    FixedClock::new(datetime!(2027-03-01 10:00:00 UTC))
}

fn audit_kind<E>(e: AuditedTxError<E>) -> AuditErrorKind {
    match e {
        AuditedTxError::Audit(a) => a.kind(),
        AuditedTxError::Work(_) => panic!("galat aksi, bukan galat audit"),
        AuditedTxError::Database { sqlstate } => panic!("galat database {sqlstate:?}"),
    }
}

/// Event E-09 tanpa hasil: ditolak pustaka (field wajib PRD §7.2).
fn invalid_event(subject: foundation::Uuid) -> AuditEvent {
    AuditEvent::new(
        EventCode::E09,
        code("status_changed"),
        Actor::system(),
        ObjectRef::with_uuid(&CARE_RECIPIENT, subject),
    )
    .subject(Subject::care_recipient(subject))
    .context(&ctx())
}

/// Pemeriksaan saat compile: future `in_audited_tx` adalah `Send` (dapat dipakai di handler axum).
#[allow(dead_code)]
fn audited_tx_future_is_send(pool: &sqlx::PgPool, c: &FixedClock) {
    fn assert_send<T: Send>(_: T) {}
    assert_send(in_audited_tx(pool, c, |_conn| {
        Box::pin(async move { Ok::<_, Infallible>(((), Vec::new())) })
    }));
}

#[tokio::test]
async fn event_and_business_change_commit_together() {
    let app = app_pool().await;
    let relay = relay_pool().await;
    let subject = new_id();

    let job = in_audited_tx(&app, &clock(), move |conn| {
        Box::pin(async move {
            let job = insert_business_row(conn, "uji_audit_commit").await;
            Ok::<_, Infallible>((job, vec![valid_event(subject)]))
        })
    })
    .await
    .unwrap();

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
async fn work_error_rolls_back_business_change() {
    let app = app_pool().await;
    let relay = relay_pool().await;
    let subject = new_id();
    let job = std::sync::Arc::new(std::sync::Mutex::new(None));
    let seen = job.clone();

    let err = in_audited_tx(&app, &clock(), move |conn| {
        Box::pin(async move {
            let id = insert_business_row(conn, "uji_audit_work_err").await;
            *seen.lock().unwrap() = Some(id);
            Err::<((), Vec<AuditEvent>), _>("aksi_gagal")
        })
    })
    .await
    .unwrap_err();
    assert!(matches!(err, AuditedTxError::Work("aksi_gagal")));
    let id = job.lock().unwrap().unwrap();
    assert!(!business_row_exists(&app, id).await);
    assert_eq!(outbox_count_for_subject(&relay, subject).await, 0);
}

#[tokio::test]
async fn invalid_or_missing_event_aborts_business_change() {
    let app = app_pool().await;
    let relay = relay_pool().await;
    let subject = new_id();
    let leaky = AuditEvent::new(
        EventCode::E09,
        code("status_changed"),
        Actor::system(),
        ObjectRef::with_uuid(&CARE_RECIPIENT, subject),
    )
    .subject(Subject::care_recipient(subject))
    .change(FieldChange::new("status", None, Some(Value::text(NIK))))
    .outcome(Outcome::success())
    .context(&ctx());

    // NEG: event tanpa hasil; NIK teks di field nilai; daftar event kosong; satu event sah + satu tidak.
    let cases: Vec<(Vec<AuditEvent>, AuditErrorKind)> = vec![
        (vec![invalid_event(subject)], AuditErrorKind::MissingField),
        (vec![leaky], AuditErrorKind::ValueNotAllowed),
        (vec![], AuditErrorKind::MissingField),
        (
            vec![valid_event(subject), invalid_event(subject)],
            AuditErrorKind::MissingField,
        ),
    ];
    for (events, kind) in cases {
        let ids = std::sync::Arc::new(std::sync::Mutex::new(None));
        let seen = ids.clone();
        let err = in_audited_tx(&app, &clock(), move |conn| {
            Box::pin(async move {
                let id = insert_business_row(conn, "uji_audit_invalid").await;
                *seen.lock().unwrap() = Some(id);
                Ok::<_, Infallible>(((), events))
            })
        })
        .await
        .unwrap_err();
        assert_eq!(audit_kind(err), kind);
        let id = ids.lock().unwrap().unwrap();
        assert!(!business_row_exists(&app, id).await);
        assert_eq!(outbox_count_for_subject(&relay, subject).await, 0);
    }
}

/// Probe review keamanan BLOKIR-2: aksi membuka savepoint, menulis data bisnis, me-release savepoint,
/// lalu event-nya ditolak. Savepoint berada di dalam transaksi pustaka, jadi ikut batal.
#[tokio::test]
async fn savepoint_inside_work_cannot_commit_without_audit() {
    let app = app_pool().await;
    let relay = relay_pool().await;
    let subject = new_id();
    let ids = std::sync::Arc::new(std::sync::Mutex::new(None));
    let seen = ids.clone();

    let err = in_audited_tx(&app, &clock(), move |conn| {
        Box::pin(async move {
            let mut sp = conn.begin().await.unwrap();
            let id = insert_business_row(&mut sp, "uji_audit_savepoint").await;
            sp.commit().await.unwrap();
            *seen.lock().unwrap() = Some(id);
            Ok::<_, Infallible>(((), vec![invalid_event(subject)]))
        })
    })
    .await
    .unwrap_err();
    assert_eq!(audit_kind(err), AuditErrorKind::MissingField);
    let id = ids.lock().unwrap().unwrap();
    assert!(
        !business_row_exists(&app, id).await,
        "data bisnis dari savepoint ter-commit tanpa event audit"
    );
    assert_eq!(outbox_count_for_subject(&relay, subject).await, 0);
}

/// Probe: SQL kendali transaksi mentah di dalam aksi terdeteksi (ID transaksi berubah); event tidak
/// ditulis dan galat dikembalikan. Literal seperti ini di kode non-test ditolak test arsitektur
/// (`tx-control`), karena perubahan yang sudah di-commit tidak dapat dibatalkan pustaka.
#[tokio::test]
async fn raw_commit_inside_work_is_detected() {
    let app = app_pool().await;
    let relay = relay_pool().await;
    let subject = new_id();
    for stmt in ["COMMIT", "ROLLBACK", "COMMIT; BEGIN"] {
        let err = in_audited_tx(&app, &clock(), move |conn| {
            Box::pin(async move {
                sqlx::raw_sql(stmt).execute(&mut *conn).await.unwrap();
                Ok::<_, Infallible>(((), vec![valid_event(subject)]))
            })
        })
        .await
        .unwrap_err();
        assert_eq!(
            audit_kind(err),
            AuditErrorKind::TransactionEscaped,
            "{stmt}"
        );
        assert_eq!(outbox_count_for_subject(&relay, subject).await, 0, "{stmt}");
    }
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
