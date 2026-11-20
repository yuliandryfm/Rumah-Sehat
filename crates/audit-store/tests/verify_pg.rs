//! F1-T25 (ADR-009 butir 5, DD B4–B5, KEPUTUSAN-141, klarifikasi T-22, TD-12): jangkar kepala rantai dan
//! verifikasi rantai di PostgreSQL 18 sekali pakai (`scripts/db-ephemeral.sh`), dengan role sungguhan:
//! `rs_audit_writer_login` (relay: event + jangkar) dan `rs_audit_verifier_login` (verifikasi + hasil).
//! Manipulasi dilakukan superuser kontainer dengan trigger penolak dilewati (`session_replication_role =
//! replica`), hanya di database uji, lalu dipulihkan supaya rantai bersama tetap utuh untuk test lain.
//!
//! Semua test berbagi satu rantai, sehingga dijalankan berurutan (`SERIAL`).
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use audit_store::anchor::{AnchorStore, FileAnchorStore, anchor_head};
use audit_store::partition::{MIN_FUTURE_PARTITIONS, check_partition_runway};
use audit_store::verify::{VerifyOutcome, run_verification};
use audit_store::{
    AppendReport, ChainHead, EventBody, EventSubject, append_batch, chain_head, truncate_to_micros,
};
use foundation::{FixedClock, Uuid, new_id};
use observability::EnvSource;
use observability::LogLevel;
use observability::log::{Component, JsonLogLayer, LogSink, MemorySink, subscriber};
use observability::security::AuditChainMismatch;
use serde_json::json;
use sqlx::{AssertSqlSafe, Connection, PgConnection};
use std::net::{IpAddr, Ipv4Addr};
use std::path::PathBuf;
use std::sync::Arc;
use time::macros::datetime;
use tokio::sync::Mutex;
use tracing::instrument::WithSubscriber;

static SERIAL: Mutex<()> = Mutex::const_new(());

async fn connect(var: &str) -> PgConnection {
    let url = EnvSource::from_process()
        .required_secret(var)
        .unwrap_or_else(|_| {
            panic!(
                "{var} tidak diset: jalankan lewat `bash scripts/db-ephemeral.sh cargo test ...`"
            )
        });
    PgConnection::connect(url.expose())
        .await
        .unwrap_or_else(|_| panic!("koneksi {var} gagal"))
}

async fn writer() -> PgConnection {
    connect("RS_TEST_DB_AUDIT_WRITER_URL").await
}

async fn verifier() -> PgConnection {
    connect("RS_TEST_DB_AUDIT_VERIFIER_URL").await
}

/// Superuser kontainer uji dengan trigger penolak dilewati (klarifikasi T-22: "trigger penolak dilewati
/// superuser", hanya lingkungan uji).
async fn tamper() -> PgConnection {
    let mut c = connect("RS_TEST_DB_AUDIT_TAMPER_URL").await;
    sqlx::raw_sql("SET session_replication_role = replica")
        .execute(&mut c)
        .await
        .unwrap();
    c
}

async fn exec(c: &mut PgConnection, sql: String) {
    sqlx::raw_sql(AssertSqlSafe(sql)).execute(c).await.unwrap();
}

fn clock() -> FixedClock {
    FixedClock::new(datetime!(2027-03-15 08:00:00 UTC))
}

fn anchor_dir() -> PathBuf {
    std::env::temp_dir().join(format!("rs-audit-anchor-{}", new_id()))
}

fn body(n: usize) -> EventBody {
    let subjects: Vec<EventSubject> = if n.is_multiple_of(2) {
        vec![
            EventSubject {
                subject_id: new_id(),
                subject_kind: "care_recipient".into(),
            },
            EventSubject {
                subject_id: new_id(),
                subject_kind: "care_recipient".into(),
            },
        ]
    } else {
        Vec::new()
    };
    EventBody {
        event_id: new_id(),
        occurred_at: truncate_to_micros(datetime!(2027-03-15 07:59:59.123456 UTC)),
        event_code: "e_12".into(),
        action_name: "report_exported".into(),
        actor_type: "account".into(),
        actor_account_id: Some(new_id()),
        actor_account_type: Some("rs_admin".into()),
        actor_roles: Some(vec!["auditor".into()]),
        actor_partner_id: None,
        object_type: "audit_log_export".into(),
        object_id: Some("csv".into()),
        object_version: None,
        subject_id: subjects.first().map(|s| s.subject_id),
        subject_count: i32::try_from(subjects.len()).unwrap(),
        tenant_id: None,
        source_component: "backoffice".into(),
        source_ip: Some(IpAddr::V4(Ipv4Addr::new(198, 51, 100, 7))),
        user_agent: Some("Mozilla/5.0 uji".into()),
        app_version: None,
        changed_fields: Some(vec!["status".into()]),
        before_values: None,
        after_values: Some(json!({"status": "done", "rows": 12})),
        disclosure_recipient: None,
        disclosure_data_categories: None,
        outcome: "success".into(),
        outcome_reason_code: None,
        correlation_id: new_id(),
        subjects,
    }
}

async fn append(w: &mut PgConnection, n: usize) -> AppendReport {
    let bodies: Vec<EventBody> = (0..n).map(body).collect();
    let r = append_batch(w, &bodies).await.unwrap();
    assert_eq!(r.appended.len(), n);
    r
}

async fn verify(
    v: &mut PgConnection,
    store: &FileAnchorStore,
) -> (VerifyOutcome, Option<Uuid>, i64, i64) {
    let (report, id) = run_verification(v, store, &clock()).await;
    (report.outcome, id, report.from_seq, report.to_seq)
}

async fn outcome_row(v: &mut PgConnection, id: Uuid) -> (String, Option<i64>, Option<Uuid>) {
    let r = sqlx::query!(
        "SELECT outcome, first_mismatch_seq, audit_chain_anchor_id FROM audit.audit_chain_verification WHERE id = $1",
        id
    )
    .fetch_one(v)
    .await
    .unwrap();
    (r.outcome, r.first_mismatch_seq, r.audit_chain_anchor_id)
}

fn mismatch(o: VerifyOutcome) -> (i64, AuditChainMismatch) {
    match o {
        VerifyOutcome::Mismatch(m) => (m.seq, m.reason),
        other => panic!("seharusnya mismatch, didapat {other:?}"),
    }
}

/// Rantai berisi minimal tiga event terbaru dengan kepala terjangkar; mengembalikan kepala.
async fn chain_with_anchor(w: &mut PgConnection, store: &FileAnchorStore) -> ChainHead {
    let head = append(w, 3).await.head.unwrap();
    let a = anchor_head(w, store, head, &clock()).await.unwrap();
    assert_eq!(a.anchor.seq, head.seq);
    head
}

#[tokio::test]
async fn intact_chain_verifies_and_result_is_recorded() {
    // trace: US-1504 AC3 (TC-US-1504-9): rantai utuh -> lulus dan hasil tercatat.
    let _s = SERIAL.lock().await;
    let dir = anchor_dir();
    let store = FileAnchorStore::new(&dir).unwrap();
    let mut w = writer().await;
    let mut v = verifier().await;
    let head = chain_with_anchor(&mut w, &store).await;
    // Jangkar ditulis ke WORM (berkas) dan tabel; menjangkarkan ulang kepala yang sama idempoten.
    let again = anchor_head(&mut w, &store, head, &clock()).await.unwrap();
    assert_eq!(again.row_id, None);
    assert_eq!(store.list().await.unwrap().len(), 1);

    let (outcome, id, from, to) = verify(&mut v, &store).await;
    assert_eq!(outcome, VerifyOutcome::Match);
    assert_eq!((from, to), (1, head.seq));
    let (o, first, anchor_id) = outcome_row(&mut v, id.unwrap()).await;
    assert_eq!((o.as_str(), first), ("match", None));
    assert!(anchor_id.is_some(), "jangkar WORM yang tercakup dirujuk");
    std::fs::remove_dir_all(dir).unwrap();
}

#[tokio::test]
async fn modified_event_is_detected_at_its_seq_and_alerted() {
    // trace: US-1504 AC3 (TC-US-1504-6): satu event diubah langsung di database -> seq k disebut,
    // entri security log tercatat.
    let _s = SERIAL.lock().await;
    let dir = anchor_dir();
    let store = FileAnchorStore::new(&dir).unwrap();
    let mut w = writer().await;
    let mut v = verifier().await;
    let head = chain_with_anchor(&mut w, &store).await;
    let k = head.seq - 1;
    let mut t = tamper().await;
    exec(
        &mut t,
        format!("UPDATE audit.audit_event SET action_name = 'dipalsukan' WHERE seq = {k}"),
    )
    .await;

    let app_sink = MemorySink::default();
    let sec_sink = MemorySink::default();
    let layer = JsonLogLayer::new(Component::AuditVerify, LogLevel::Info, Arc::new(clock()))
        .with_sinks(
            Arc::new(app_sink.clone()) as Arc<dyn LogSink>,
            Arc::new(sec_sink.clone()),
        );
    let (outcome, id, _, to) = verify(&mut v, &store)
        .with_subscriber(subscriber(layer))
        .await;
    assert_eq!(mismatch(outcome), (k, AuditChainMismatch::EventHash));
    assert_eq!(to, k);
    let (o, first, _) = outcome_row(&mut v, id.unwrap()).await;
    assert_eq!((o.as_str(), first), ("mismatch", Some(k)));
    let sec = sec_sink.lines();
    let alert = sec
        .iter()
        .find(|l| l.contains("\"event_type\":\"audit_chain_mismatch\""))
        .unwrap_or_else(|| panic!("security log ketidakcocokan tidak ada: {sec:?}"));
    assert!(
        alert.contains(&format!("\"first_mismatch_seq\":{k}")),
        "{alert}"
    );
    assert!(alert.contains("\"reason\":\"event_hash\""), "{alert}");
    assert!(!alert.contains("198.51.100.7"), "{alert}");

    exec(
        &mut t,
        format!("UPDATE audit.audit_event SET action_name = 'report_exported' WHERE seq = {k}"),
    )
    .await;
    assert_eq!(verify(&mut v, &store).await.0, VerifyOutcome::Match);
    std::fs::remove_dir_all(dir).unwrap();
}

#[tokio::test]
async fn deleted_middle_event_or_replaced_prev_hash_is_detected() {
    // trace: US-1504 AC3 (TC-US-1504-7): event dihapus dari tengah rantai, atau prev_hash diganti.
    let _s = SERIAL.lock().await;
    let dir = anchor_dir();
    let store = FileAnchorStore::new(&dir).unwrap();
    let mut w = writer().await;
    let mut v = verifier().await;
    let head = chain_with_anchor(&mut w, &store).await;
    let k = head.seq - 1;
    let mut t = tamper().await;
    exec(
        &mut t,
        format!(
            "CREATE TEMP TABLE saved_event AS SELECT * FROM audit.audit_event WHERE seq = {k}; \
         DELETE FROM audit.audit_event WHERE seq = {k}"
        ),
    )
    .await;
    assert_eq!(
        mismatch(verify(&mut v, &store).await.0),
        (k, AuditChainMismatch::SeqGap)
    );
    exec(
        &mut t,
        "INSERT INTO audit.audit_event SELECT * FROM saved_event".into(),
    )
    .await;
    assert_eq!(verify(&mut v, &store).await.0, VerifyOutcome::Match);

    exec(&mut t, format!(
        "UPDATE audit.audit_event SET prev_hash = decode(repeat('ab', 32), 'hex') WHERE seq = {k}"
    ))
    .await;
    assert_eq!(
        mismatch(verify(&mut v, &store).await.0),
        (k, AuditChainMismatch::PrevHash)
    );
    exec(&mut t, format!(
        "UPDATE audit.audit_event e SET prev_hash = s.prev_hash FROM saved_event s WHERE e.seq = {k}"
    ))
    .await;
    assert_eq!(verify(&mut v, &store).await.0, VerifyOutcome::Match);
    std::fs::remove_dir_all(dir).unwrap();
}

#[tokio::test]
async fn forged_event_inserted_between_two_events_is_detected() {
    // trace: US-1504 AC3 (TC-US-1504-10): event palsu disisipkan di antara dua event.
    let _s = SERIAL.lock().await;
    let dir = anchor_dir();
    let store = FileAnchorStore::new(&dir).unwrap();
    let mut w = writer().await;
    let mut v = verifier().await;
    let head = chain_with_anchor(&mut w, &store).await;
    let k = head.seq - 1;
    let forged = new_id();
    let mut t = tamper().await;
    exec(&mut t, format!(
        "INSERT INTO audit.audit_event SELECT '{forged}'::uuid, recorded_at, seq, occurred_at, event_code, \
         'disisipkan', actor_type, actor_account_id, actor_account_type, actor_roles, actor_partner_id, \
         object_type, object_id, object_version, subject_id, subject_count, tenant_id, source_component, \
         source_ip, user_agent, app_version, changed_fields, before_values, after_values, \
         disclosure_recipient, disclosure_data_categories, outcome, outcome_reason_code, correlation_id, \
         canonical_version, prev_hash, event_hash FROM audit.audit_event WHERE seq = {k}"
    ))
    .await;
    let (seq, _) = mismatch(verify(&mut v, &store).await.0);
    assert_eq!(seq, k, "seq pertama yang tidak cocok = posisi sisipan");
    exec(
        &mut t,
        format!("DELETE FROM audit.audit_event WHERE event_id = '{forged}'"),
    )
    .await;
    assert_eq!(verify(&mut v, &store).await.0, VerifyOutcome::Match);
    std::fs::remove_dir_all(dir).unwrap();
}

#[tokio::test]
async fn truncated_tail_is_detected_through_worm_anchor() {
    // trace: US-1504 AC3 (TC-US-1504-8): event terakhir dihapus beserta kuncinya; rantai yang tersisa
    // konsisten, hanya jangkar WORM yang menangkapnya.
    let _s = SERIAL.lock().await;
    let dir = anchor_dir();
    let store = FileAnchorStore::new(&dir).unwrap();
    let mut w = writer().await;
    let mut v = verifier().await;
    let head = chain_with_anchor(&mut w, &store).await;
    let n = head.seq;
    let mut t = tamper().await;
    // Jejak di tabel jangkar ikut dihapus penyerang; jangkar di WORM tidak dapat dihapus.
    exec(&mut t, format!(
        "CREATE TEMP TABLE saved_tail AS SELECT * FROM audit.audit_event WHERE seq = {n}; \
         CREATE TEMP TABLE saved_tail_subject AS SELECT s.* FROM audit.audit_event_subject s \
           JOIN saved_tail e ON e.event_id = s.event_id AND e.recorded_at = s.recorded_at; \
         CREATE TEMP TABLE saved_tail_key AS SELECT * FROM audit.audit_event_key WHERE seq = {n}; \
         CREATE TEMP TABLE saved_tail_anchor AS SELECT * FROM audit.audit_chain_anchor WHERE seq = {n}; \
         DELETE FROM audit.audit_event_subject s USING saved_tail e \
           WHERE e.event_id = s.event_id AND e.recorded_at = s.recorded_at; \
         DELETE FROM audit.audit_event WHERE seq = {n}; \
         DELETE FROM audit.audit_event_key WHERE seq = {n}; \
         DELETE FROM audit.audit_chain_verification WHERE audit_chain_anchor_id IN (SELECT id FROM saved_tail_anchor); \
         DELETE FROM audit.audit_chain_anchor WHERE seq = {n}"
    ))
    .await;
    let (outcome, _, _, _) = verify(&mut v, &store).await;
    assert_eq!(
        mismatch(outcome),
        (n, AuditChainMismatch::AnchorBeyondChain)
    );
    // Tanpa jangkar WORM, pemotongan ujung tidak terlihat (bukti peran jangkar).
    let empty_dir = anchor_dir();
    let empty = FileAnchorStore::new(&empty_dir).unwrap();
    assert_eq!(verify(&mut v, &empty).await.0, VerifyOutcome::Match);

    exec(
        &mut t,
        "INSERT INTO audit.audit_event_key SELECT * FROM saved_tail_key; \
                  INSERT INTO audit.audit_event SELECT * FROM saved_tail; \
                  INSERT INTO audit.audit_event_subject SELECT * FROM saved_tail_subject; \
                  INSERT INTO audit.audit_chain_anchor SELECT * FROM saved_tail_anchor"
            .into(),
    )
    .await;
    assert_eq!(chain_head(&mut w).await.unwrap().unwrap().seq, n);
    assert_eq!(verify(&mut v, &store).await.0, VerifyOutcome::Match);
    std::fs::remove_dir_all(dir).unwrap();
    std::fs::remove_dir_all(empty_dir).unwrap();
}

#[tokio::test]
async fn unreadable_anchor_store_records_error_outcome() {
    // NEG: jangkar WORM tidak terbaca -> hasil `error` tercatat, bukan `match`.
    let _s = SERIAL.lock().await;
    let dir = anchor_dir();
    let store = FileAnchorStore::new(&dir).unwrap();
    std::fs::write(dir.join("chain-head-00000000000000000001.json"), b"rusak").unwrap();
    let mut v = verifier().await;
    let (outcome, id, _, _) = verify(&mut v, &store).await;
    assert!(matches!(outcome, VerifyOutcome::Error(_)), "{outcome:?}");
    assert_eq!(outcome_row(&mut v, id.unwrap()).await.0, "error");
    std::fs::remove_dir_all(dir).unwrap();
}

#[tokio::test]
async fn partition_runway_metric_reads_catalog_and_alerts_below_six_months() {
    // TD-12, skema fisik §6.2 butir 2: partisi migrasi 2026-11 s.d. 2029-12.
    let _s = SERIAL.lock().await;
    let mut v = verifier().await;
    let ok = check_partition_runway(&mut v, &FixedClock::new(datetime!(2027-03-15 08:00 UTC)))
        .await
        .unwrap();
    assert!(ok.current_covered);
    assert_eq!(ok.future_partitions, 33);
    assert!(!ok.is_low());

    let app_sink = MemorySink::default();
    let sec_sink = MemorySink::default();
    let layer = JsonLogLayer::new(Component::AuditVerify, LogLevel::Info, Arc::new(clock()))
        .with_sinks(
            Arc::new(app_sink.clone()) as Arc<dyn LogSink>,
            Arc::new(sec_sink.clone()),
        );
    let low = check_partition_runway(&mut v, &FixedClock::new(datetime!(2029-08-10 00:00 UTC)))
        .with_subscriber(subscriber(layer))
        .await
        .unwrap();
    assert_eq!(low.future_partitions, 4);
    assert!(low.future_partitions < MIN_FUTURE_PARTITIONS && low.is_low());
    assert!(
        app_sink.text().contains("\"future_partitions\":4"),
        "{}",
        app_sink.text()
    );
    let sec = sec_sink.lines();
    assert!(
        sec.iter().any(
            |l| l.contains("\"event_type\":\"audit_partition_runway_low\"")
                && l.contains("\"future_partitions\":4")
        ),
        "{sec:?}"
    );
}
