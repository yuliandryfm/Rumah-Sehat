//! F1-T24 (+ jangkar F1-T25) (ADR-009 butir 3 dan 5, skema fisik §6.3–§6.4, KEPUTUSAN-102, KEPUTUSAN-103, KEPUTUSAN-284):
//! relay memindahkan `audit_outbox` ke database audit dengan `seq` tanpa celah dan rantai hash, satu relay
//! aktif, kiriman ulang dilewati per constraint, konflik `seq` membatalkan batch. Hak runtime sungguhan:
//! `rs_audit_relay_login` (database aplikasi) dan `rs_audit_writer_login` (database audit, INSERT saja).
//! PostgreSQL 18 sekali pakai dari `scripts/db-ephemeral.sh`; tanpa itu test gagal (bukan dilewati).
//!
//! Semua test berbagi satu rantai dan satu kunci relay, sehingga dijalankan berurutan (`SERIAL`).
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

mod common;

use audit::relay::{RelayError, RelaySettings, map_payload, relay_batch, run_relay};
use audit::{AuditEvent, Subject, in_audited_tx};
use audit_store::anchor::{AnchorStore, FileAnchorStore};
use audit_store::verify::{VerifyOutcome, verify_chain};
use audit_store::{
    GENESIS_PREV_HASH, PageCursor, StoreErrorKind, append_batch, chain_head, event_hash,
    find_event, read_page,
};
use common::*;
use foundation::{Clock, FixedClock, SystemClock, Uuid, new_id};
use platform_jobs::lock::RS_AUDIT_RELAY;
use platform_jobs::{LockAttempt, SingletonLock};
use serde_json::json;
use sqlx::{Connection, PgConnection, PgPool};
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;
use time::macros::datetime;
use tokio::sync::Mutex;

static SERIAL: Mutex<()> = Mutex::const_new(());

/// Jaring pengaman penantian kondisi relay (bukan sinkronisasinya; lihat test dua relay).
const RELAY_WAIT: Duration = Duration::from_secs(60);

/// Jumlah sesi yang memegang kunci advisory relay 7001001.
async fn relay_lock_holders(relay: &PgPool) -> i64 {
    sqlx::query_scalar!(
        r#"SELECT count(*) AS "n!" FROM pg_locks
               WHERE locktype = 'advisory' AND classid = 0 AND objid = 7001001 AND objsubid = 1 AND granted"#
    )
    .fetch_one(relay)
    .await
    .unwrap()
}

fn clock() -> FixedClock {
    FixedClock::new(datetime!(2027-03-01 10:00:00 UTC))
}

/// Mengambil kunci relay (menunggu bila sesi test sebelumnya belum selesai melepasnya).
async fn relay_lock() -> SingletonLock {
    let mut conn = connection("RS_TEST_DB_RELAY_URL").await;
    for _ in 0..100 {
        match SingletonLock::try_acquire(conn, RS_AUDIT_RELAY)
            .await
            .unwrap()
        {
            LockAttempt::Acquired(l) => return l,
            LockAttempt::Busy(c) => {
                conn = c;
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }
    }
    panic!("kunci relay tidak terlepas");
}

async fn writer() -> PgConnection {
    connection("RS_TEST_DB_AUDIT_WRITER_URL").await
}

/// Menghabiskan outbox (termasuk sisa test lain) lewat relay.
async fn drain(lock: &mut SingletonLock, w: &mut PgConnection) {
    loop {
        let b = relay_batch(lock, w, 50).await.unwrap();
        if b.fetched == 0 {
            return;
        }
    }
}

async fn record(app: &PgPool, events: Vec<AuditEvent>) {
    in_audited_tx(app, &clock(), move |_conn| {
        Box::pin(async move { Ok::<_, Infallible>(((), events)) })
    })
    .await
    .unwrap();
}

/// Baris outbox milik subjek (dibaca relay sebelum dikirim).
async fn outbox_row(
    relay: &PgPool,
    subject: Uuid,
) -> (Uuid, time::OffsetDateTime, serde_json::Value) {
    let r = sqlx::query!(
        r#"SELECT id, occurred_at, payload FROM app.audit_outbox
           WHERE payload -> 'subjects' -> 0 ->> 'subject_id' = $1"#,
        subject.hyphenated().to_string(),
    )
    .fetch_one(relay)
    .await
    .unwrap();
    (r.id, r.occurred_at, r.payload)
}

/// Seluruh rantai: `seq` 1..n tanpa celah, `prev_hash` menyambung, `event_hash` dihitung ulang cocok.
async fn assert_chain_intact(verifier: &PgPool) -> i64 {
    let mut conn = verifier.acquire().await.unwrap();
    let mut cursor = PageCursor::START;
    let mut expected_seq = 1;
    let mut prev = GENESIS_PREV_HASH;
    loop {
        let page = read_page(&mut conn, cursor, 500).await.unwrap();
        if page.is_empty() {
            return expected_seq - 1;
        }
        for e in page {
            assert_eq!(e.event.seq, expected_seq, "seq bercelah atau ganda");
            assert_eq!(
                e.prev_hash, prev,
                "prev_hash tidak menyambung di seq {}",
                e.event.seq
            );
            let h = event_hash(&prev, &e.event).unwrap();
            assert_eq!(
                h, e.event_hash,
                "event_hash tidak cocok di seq {}",
                e.event.seq
            );
            prev = h;
            expected_seq += 1;
            cursor = PageCursor {
                seq: e.event.seq,
                event_id: e.event.body.event_id,
            };
        }
    }
}

async fn outbox_contains(relay: &PgPool, id: Uuid) -> bool {
    sqlx::query_scalar!(
        r#"SELECT count(*) AS "n!" FROM app.audit_outbox WHERE id = $1"#,
        id
    )
    .fetch_one(relay)
    .await
    .unwrap()
        == 1
}

#[tokio::test]
async fn relay_moves_outbox_into_gapless_hash_chain() {
    // trace: US-1501 AC1 (TC-US-1501-12): seq berurutan tanpa celah, prev_hash/event_hash sambung.
    let _serial = SERIAL.lock().await;
    let app = app_pool().await;
    let relay = relay_pool().await;
    let verifier = audit_verifier_pool().await;
    let mut lock = relay_lock().await;
    let mut w = writer().await;
    drain(&mut lock, &mut w).await;

    let (s1, s2, s3) = (new_id(), new_id(), new_id());
    let multi = valid_event(s2).subject(Subject::care_recipient(s3));
    record(&app, vec![valid_event(s1), multi]).await;
    let before1 = outbox_row(&relay, s1).await;
    let before2 = outbox_row(&relay, s2).await;
    let head_before = chain_head(&mut w).await.unwrap();

    let batch = relay_batch(&mut lock, &mut w, 50).await.unwrap();
    assert_eq!(batch.fetched, 2);
    assert_eq!(batch.append.appended, vec![before1.0, before2.0]);
    assert!(batch.append.resent.is_empty());
    assert_eq!(batch.deleted, 2);
    assert!(!outbox_contains(&relay, before1.0).await);
    assert!(!outbox_contains(&relay, before2.0).await);
    let head = batch.append.head.unwrap();
    let base = head_before.map_or(0, |h| h.seq);
    assert_eq!(head.seq, base + 2);

    let mut v = verifier.acquire().await.unwrap();
    let e1 = find_event(&mut v, before1.0).await.unwrap().unwrap();
    let e2 = find_event(&mut v, before2.0).await.unwrap().unwrap();
    assert_eq!((e1.event.seq, e2.event.seq), (base + 1, base + 2));
    assert_eq!(
        e1.prev_hash,
        head_before.map_or(GENESIS_PREV_HASH, |h| h.event_hash)
    );
    assert_eq!(e2.prev_hash, e1.event_hash);
    assert_eq!(head.event_hash, e2.event_hash);
    // Nilai yang dibaca kembali dari database = hasil pemetaan payload (jsonb, inet, text[], jam).
    assert_eq!(
        e1.event.body,
        map_payload(before1.0, before1.1, &before1.2).unwrap()
    );
    assert_eq!(
        e2.event.body,
        map_payload(before2.0, before2.1, &before2.2).unwrap()
    );
    assert_eq!(e1.event.body.subject_count, 1);
    assert_eq!(e1.event.body.subject_id, Some(s1));
    assert!(e1.event.body.subjects.is_empty());
    // Lebih dari satu subjek: subjek utama + semua subjek di audit_event_subject (DD B1, B3).
    assert_eq!(e2.event.body.subject_count, 2);
    assert_eq!(e2.event.body.subject_id, Some(s2));
    let mut kinds: Vec<Uuid> = e2
        .event
        .body
        .subjects
        .iter()
        .map(|s| s.subject_id)
        .collect();
    kinds.sort();
    let mut expected = vec![s2, s3];
    expected.sort();
    assert_eq!(kinds, expected);
    // Masking pustaka tetap utuh di audit store: NIK hanya 4 digit terakhir.
    assert_eq!(
        e1.event.body.after_values,
        Some(json!({"status": "verified", "nik": {"last4": "8901"}}))
    );
    // recorded_at = jam server database audit, sama untuk satu batch dan tidak mundur (§6.4 butir 3).
    assert_eq!(e1.event.recorded_at, e2.event.recorded_at);
    assert_eq!(head.recorded_at, e2.event.recorded_at);
    assert_chain_intact(&verifier).await;
    lock.release().await.unwrap();
}

#[tokio::test]
async fn resent_event_is_skipped_without_consuming_seq() {
    // KEPUTUSAN-284: 23505 pk_audit_event_key = kiriman ulang -> ROLLBACK TO SAVEPOINT, seq tidak naik.
    let _serial = SERIAL.lock().await;
    let app = app_pool().await;
    let relay = relay_pool().await;
    let verifier = audit_verifier_pool().await;
    let mut lock = relay_lock().await;
    let mut w = writer().await;
    drain(&mut lock, &mut w).await;

    // Relay sebelumnya sudah commit event A di database audit lalu mati sebelum menghapus outbox.
    let (sa, sb) = (new_id(), new_id());
    record(&app, vec![valid_event(sa)]).await;
    let a = outbox_row(&relay, sa).await;
    let first = append_batch(&mut w, &[map_payload(a.0, a.1, &a.2).unwrap()])
        .await
        .unwrap();
    assert_eq!(first.appended, vec![a.0]);
    let head_a = first.head.unwrap();
    record(&app, vec![valid_event(sb)]).await;
    let b = outbox_row(&relay, sb).await;

    let batch = relay_batch(&mut lock, &mut w, 50).await.unwrap();
    assert_eq!(batch.append.resent, vec![a.0]);
    assert_eq!(batch.append.appended, vec![b.0]);
    assert_eq!(batch.deleted, 2, "kiriman ulang ikut dihapus dari outbox");
    assert!(!outbox_contains(&relay, a.0).await);
    let mut v = verifier.acquire().await.unwrap();
    let ea = find_event(&mut v, a.0).await.unwrap().unwrap();
    let eb = find_event(&mut v, b.0).await.unwrap().unwrap();
    assert_eq!(
        ea.event.seq, head_a.seq,
        "event A tetap satu kali dengan seq semula"
    );
    assert_eq!(
        eb.event.seq,
        head_a.seq + 1,
        "kiriman ulang tidak memakan seq"
    );
    assert_eq!(eb.prev_hash, ea.event_hash);

    // Batch berisi kiriman ulang saja: tidak ada yang ditulis, kepala rantai tetap.
    let c = map_payload(b.0, b.1, &b.2).unwrap();
    let again = append_batch(&mut w, &[c]).await.unwrap();
    assert!(again.appended.is_empty());
    assert_eq!(again.resent, vec![b.0]);
    assert_eq!(again.head.unwrap().seq, head_a.seq + 1);
    assert_chain_intact(&verifier).await;
    lock.release().await.unwrap();
}

#[tokio::test]
async fn seq_conflict_from_concurrent_writer_aborts_whole_batch() {
    // NEG (KEPUTUSAN-284, §6.4 butir 5): penulis lain mengambil seq yang sama -> 23505
    // uq_audit_event_key_seq -> seluruh batch batal, event tetap di outbox, tidak ada cabang rantai.
    let _serial = SERIAL.lock().await;
    let app = app_pool().await;
    let relay = relay_pool().await;
    let verifier = audit_verifier_pool().await;
    let mut lock = relay_lock().await;
    let mut w = writer().await;
    drain(&mut lock, &mut w).await;
    let head_before = chain_head(&mut w).await.unwrap().map_or(0, |h| h.seq);

    // Penulis "liar" (relay kedua yang mengabaikan kunci) menulis seq berikutnya, belum commit.
    let mut rogue = writer().await;
    let mut rogue_tx = rogue.begin().await.unwrap();
    let rx = valid_event(new_id()).finish(&clock()).unwrap();
    let rogue_body = map_payload(rx.event_id(), rx.occurred_at(), rx.payload()).unwrap();
    let rogue_report = append_batch(&mut rogue_tx, &[rogue_body]).await.unwrap();
    assert_eq!(rogue_report.head.unwrap().seq, head_before + 1);

    let (s1, s2) = (new_id(), new_id());
    record(&app, vec![valid_event(s1), valid_event(s2)]).await;
    let y1 = outbox_row(&relay, s1).await.0;
    let y2 = outbox_row(&relay, s2).await.0;

    let task = tokio::spawn(async move {
        let r = relay_batch(&mut lock, &mut w, 50).await;
        (r, lock, w)
    });
    // Tunggu sampai INSERT relay menunggu baris seq milik penulis liar.
    let mut probe = writer().await;
    let mut waiting = false;
    for _ in 0..200 {
        let n = sqlx::query_scalar!(
            r#"SELECT count(*) AS "n!" FROM pg_stat_activity
               WHERE datname = current_database() AND wait_event_type = 'Lock'"#
        )
        .fetch_one(&mut probe)
        .await
        .unwrap();
        if n >= 1 {
            waiting = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert!(waiting, "relay tidak menunggu baris seq penulis lain");
    rogue_tx.commit().await.unwrap();

    let (result, mut lock, mut w) = task.await.unwrap();
    match result {
        Err(RelayError::Store(e)) => {
            assert_eq!(e.kind(), StoreErrorKind::SeqConflict);
            assert_eq!(e.sqlstate(), Some("23505"));
            assert_eq!(e.constraint(), Some("uq_audit_event_key_seq"));
        }
        other => panic!("seharusnya konflik seq, didapat {other:?}"),
    }
    // Batch batal seluruhnya: kedua event tetap di outbox dan belum ada di database audit.
    assert!(outbox_contains(&relay, y1).await && outbox_contains(&relay, y2).await);
    let mut v = verifier.acquire().await.unwrap();
    assert!(find_event(&mut v, y1).await.unwrap().is_none());
    assert!(find_event(&mut v, y2).await.unwrap().is_none());

    // Putaran berikutnya melanjutkan dari kepala baru tanpa celah.
    let batch = relay_batch(&mut lock, &mut w, 50).await.unwrap();
    assert_eq!(batch.append.appended, vec![y1, y2]);
    assert_eq!(batch.append.head.unwrap().seq, head_before + 3);
    assert_chain_intact(&verifier).await;
    lock.release().await.unwrap();
}

#[tokio::test]
async fn invalid_payload_aborts_batch_and_keeps_outbox() {
    // NEG: payload yang tidak dapat dipetakan menghentikan batch; tidak ada event yang hilang.
    let _serial = SERIAL.lock().await;
    let app = app_pool().await;
    let relay = relay_pool().await;
    let mut lock = relay_lock().await;
    let mut w = writer().await;
    drain(&mut lock, &mut w).await;
    let head_before = chain_head(&mut w).await.unwrap();

    let good = new_id();
    record(&app, vec![valid_event(good)]).await;
    let good_id = outbox_row(&relay, good).await.0;
    let bad_id = new_id();
    sqlx::query!(
        "INSERT INTO app.audit_outbox (id, occurred_at, payload) VALUES ($1, now(), $2)",
        bad_id,
        json!({"payload_version": 1, "event_id": bad_id.hyphenated().to_string(), "unknown": true}),
    )
    .execute(&app)
    .await
    .unwrap();

    match relay_batch(&mut lock, &mut w, 50).await {
        Err(RelayError::InvalidPayload { event_id, field }) => {
            assert_eq!(event_id, bad_id);
            assert_eq!(field, "payload");
        }
        other => panic!("seharusnya payload tidak valid, didapat {other:?}"),
    }
    assert!(outbox_contains(&relay, good_id).await && outbox_contains(&relay, bad_id).await);
    assert_eq!(chain_head(&mut w).await.unwrap(), head_before);

    // Pembersihan test: baris rusak dihapus (hak DELETE relay), relay lanjut.
    sqlx::query!("DELETE FROM app.audit_outbox WHERE id = $1", bad_id)
        .execute(&relay)
        .await
        .unwrap();
    drain(&mut lock, &mut w).await;
    assert!(!outbox_contains(&relay, good_id).await);
    lock.release().await.unwrap();
}

#[tokio::test]
async fn two_relays_running_together_keep_one_active_and_chain_intact() {
    // §6.4 butir 1, KEPUTUSAN-103: dua instans relay bersamaan; satu memegang kunci 7001001, yang lain
    // siaga. Semua event terkirim tepat satu kali, rantai tanpa celah.
    let _serial = SERIAL.lock().await;
    let app = app_pool().await;
    let relay = relay_pool().await;
    let verifier = audit_verifier_pool().await;
    {
        let mut lock = relay_lock().await;
        let mut w = writer().await;
        drain(&mut lock, &mut w).await;
        lock.release().await.unwrap();
    }
    let anchor_dir = std::env::temp_dir().join(format!("rs-relay-anchor-{}", new_id()));
    let settings = RelaySettings {
        lock_key: RS_AUDIT_RELAY,
        app_database_url: secret_url("RS_TEST_DB_RELAY_URL"),
        audit_database_url: secret_url("RS_TEST_DB_AUDIT_WRITER_URL"),
        batch_size: 7,
        poll_interval: Duration::from_millis(20),
        retry_interval: Duration::from_millis(50),
        anchors: Arc::new(FileAnchorStore::new(&anchor_dir).unwrap()),
        anchor_interval: Duration::ZERO,
    };
    // Event menunggu di outbox sebelum kedua relay mulai berebut kunci.
    let mut ids = Vec::new();
    for _ in 0..30 {
        let s = new_id();
        record(&app, vec![valid_event(s)]).await;
        ids.push(outbox_row(&relay, s).await.0);
    }
    let clock: Arc<dyn Clock> = Arc::new(SystemClock);
    let r1 = tokio::spawn(run_relay(settings.clone(), clock.clone()));
    let r2 = tokio::spawn(run_relay(settings, clock));

    // Selama keduanya hidup, kunci advisory 7001001 dipegang tepat satu sesi.
    let deadline = tokio::time::Instant::now() + RELAY_WAIT;
    let mut holders = relay_lock_holders(&relay).await;
    while holders == 0 && tokio::time::Instant::now() < deadline {
        tokio::time::sleep(Duration::from_millis(20)).await;
        holders = relay_lock_holders(&relay).await;
    }
    assert_eq!(holders, 1, "kunci relay harus dipegang tepat satu sesi");

    // TD-21: relay baru selesai satu putaran setelah DELETE outbox (terlihat sebagai outbox kosong) DAN
    // jangkar kepala rantai tertulis (berkas WORM lalu baris audit_chain_anchor). Menghentikan relay begitu
    // outbox kosong memotong putaran terakhir sebelum jangkarnya tertulis (test dulu gagal sesekali saat mesin
    // sibuk: jangkar tertinggal satu batch dari kepala). Maka relay dihentikan hanya setelah kondisi selesai itu
    // teramati; batas waktu hanya jaring pengaman agar test tidak menggantung bila relay rusak.
    let store = FileAnchorStore::new(&anchor_dir).unwrap();
    let mut w = writer().await;
    let mut v = verifier.acquire().await.unwrap();
    loop {
        let holders = relay_lock_holders(&relay).await;
        assert!(holders <= 1, "kunci relay dipegang {holders} sesi");
        let pending = sqlx::query_scalar!(r#"SELECT count(*) AS "n!" FROM app.audit_outbox"#)
            .fetch_one(&relay)
            .await
            .unwrap();
        let mut state = format!("outbox {pending}");
        if pending == 0 {
            let head = chain_head(&mut w).await.unwrap().unwrap();
            let last_worm = store.list().await.unwrap().last().map(|a| a.seq);
            let report = verify_chain(&mut v, &store, &SystemClock).await;
            if last_worm == Some(head.seq)
                && report.outcome == VerifyOutcome::Match
                && report.to_seq == head.seq
                && report.audit_chain_anchor_id.is_some()
            {
                break;
            }
            state = format!(
                "kepala {} jangkar WORM {last_worm:?} verifikasi {:?}/{} baris jangkar {:?}",
                head.seq, report.outcome, report.to_seq, report.audit_chain_anchor_id
            );
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "relay tidak menyelesaikan putaran dalam {RELAY_WAIT:?}: {state}"
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    r1.abort();
    r2.abort();
    let _ = r1.await;
    let _ = r2.await;

    // Setiap event tepat satu kali (PK audit_event_key) dan tidak tersisa di outbox; rantai utuh.
    let mut seqs = Vec::new();
    for id in &ids {
        assert!(!outbox_contains(&relay, *id).await, "event masih di outbox");
        seqs.push(
            find_event(&mut v, *id)
                .await
                .unwrap()
                .expect("event hilang")
                .event
                .seq,
        );
    }
    // Urutan relatif event milik test ini mengikuti urutan outbox (occurred_at sama → urutan id), tanpa
    // ganda. Kerapatan seq tidak diperiksa di sini (event lain dapat terselip di antaranya); rantai tanpa
    // celah diperiksa assert_chain_intact di bawah.
    let mut by_id: Vec<(Uuid, i64)> = ids.iter().copied().zip(seqs.iter().copied()).collect();
    by_id.sort_by_key(|(id, _)| *id);
    for pair in by_id.windows(2) {
        assert!(
            pair[1].1 > pair[0].1,
            "seq tidak mengikuti urutan outbox: {pair:?}"
        );
    }
    assert_chain_intact(&verifier).await;

    // F1-T25: relay menjangkarkan kepala rantai ke WORM (berkas) dan tabel audit_chain_anchor; verifikasi
    // terhadap jangkar itu lulus dan merujuk baris jangkarnya.
    let anchors = store.list().await.unwrap();
    let head = chain_head(&mut w).await.unwrap().unwrap();
    assert_eq!(
        anchors.last().map(|a| (a.seq, a.event_hash)),
        Some((head.seq, head.event_hash))
    );
    let report = verify_chain(&mut v, &store, &SystemClock).await;
    assert_eq!(report.outcome, VerifyOutcome::Match);
    assert_eq!(report.to_seq, head.seq);
    assert!(report.audit_chain_anchor_id.is_some());
    std::fs::remove_dir_all(anchor_dir).unwrap();
}
