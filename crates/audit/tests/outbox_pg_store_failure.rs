//! F1-T23 NEG (ADR-009 butir 2, KEPUTUSAN-15): `INSERT` ke `audit_outbox` gagal di database
//! (lock_timeout karena tabel dikunci sesi lain) -> transaksi pemanggil batal, perubahan bisnis hilang,
//! dan satu entri security log `audit_write_failed` (klarifikasi T-21). Berkas terpisah supaya kunci
//! tabel tidak mengganggu test lain (cargo menjalankan biner test satu per satu).
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use audit::{AuditErrorKind, record};
use common::*;
use foundation::{FixedClock, new_id};
use observability::LogLevel;
use observability::log::{Component, JsonLogLayer, LogSink, MemorySink, subscriber};
use std::sync::Arc;
use time::macros::datetime;
use tracing::instrument::WithSubscriber;

#[tokio::test]
async fn store_failure_aborts_business_change_and_emits_security_log() {
    let app = app_pool().await;
    let relay = relay_pool().await;
    let subject = new_id();
    let clock = FixedClock::new(datetime!(2027-03-01 10:00:00 UTC));

    // Sesi relay menahan kunci SHARE (bentrok dengan ROW EXCLUSIVE milik INSERT).
    let mut holder = relay.begin().await.unwrap();
    sqlx::raw_sql("LOCK TABLE app.audit_outbox IN SHARE MODE")
        .execute(&mut *holder)
        .await
        .unwrap();

    let app_sink = MemorySink::default();
    let sec_sink = MemorySink::default();
    let layer = JsonLogLayer::new(
        Component::Api,
        LogLevel::Info,
        Arc::new(FixedClock::new(datetime!(2027-03-01 10:00:00 UTC))),
    )
    .with_sinks(
        Arc::new(app_sink.clone()) as Arc<dyn LogSink>,
        Arc::new(sec_sink.clone()),
    );

    let (job, err) = async {
        let mut tx = app.begin().await.unwrap();
        sqlx::raw_sql("SET LOCAL lock_timeout = '300ms'")
            .execute(&mut *tx)
            .await
            .unwrap();
        let job = insert_business_row(&mut tx, "uji_audit_store_failure").await;
        let err = record(tx, valid_event(subject), &clock).await.unwrap_err();
        (job, err)
    }
    .with_subscriber(subscriber(layer))
    .await;
    holder.rollback().await.unwrap();

    assert_eq!(err.kind(), AuditErrorKind::Write);
    assert!(
        !business_row_exists(&app, job).await,
        "perubahan bisnis ikut ter-commit padahal event audit gagal ditulis"
    );
    assert_eq!(outbox_count_for_subject(&relay, subject).await, 0);

    let sec = sec_sink.lines();
    assert_eq!(sec.len(), 1, "{sec:?}");
    let line = sec.first().unwrap();
    assert!(
        line.contains("\"event_type\":\"audit_write_failed\""),
        "{line}"
    );
    assert!(line.contains("\"stage\":\"store_error\""), "{line}");
    assert!(line.contains("\"event_code\":\"e_07\""), "{line}");
    for forbidden in [NIK, "198.51.100.23", &subject.hyphenated().to_string()] {
        assert!(!line.contains(forbidden), "bocor {forbidden}: {line}");
    }
    assert!(!app_sink.text().contains(NIK));

    // Setelah kunci dilepas, penulisan berikutnya berhasil (kegagalan bukan permanen).
    let mut tx = app.begin().await.unwrap();
    let job = insert_business_row(&mut tx, "uji_audit_store_failure").await;
    let tx = record(tx, valid_event(subject), &clock).await.unwrap();
    tx.commit().await.unwrap();
    assert!(business_row_exists(&app, job).await);
    assert_eq!(outbox_count_for_subject(&relay, subject).await, 1);
}
