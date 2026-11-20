//! F1-T20: idempotensi job lewat kunci deduplikasi (`uqp_job_dedupe_key`, DD A19.4) dan penulisan job
//! di transaksi pemanggil (ADR-007 butir 3). PostgreSQL 18 sekali pakai dari `scripts/db-ephemeral.sh`.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use common::*;
use foundation::{Clock, new_id};
use platform_jobs::{DedupeKey, EnqueueOutcome, JobOutcome, NewJob, enqueue, run_batch};
use std::cell::Cell;
use time::Duration;

#[tokio::test]
async fn same_dedupe_key_while_active_yields_one_job_processed_once() {
    let pool = app_pool().await;
    let jt = unique_type("dedupe");
    let clock = clock();
    let entity = new_id();
    let key = DedupeKey::for_entity(jt.as_str(), entity).unwrap();
    let job = NewJob::new(jt.clone(), payload(entity), clock.now(), attempts(3)).dedupe_key(key);

    let first = enqueue(&pool, &job).await.unwrap();
    assert!(matches!(first, EnqueueOutcome::Enqueued(_)));
    assert_eq!(
        enqueue(&pool, &job).await.unwrap(),
        EnqueueOutcome::Duplicate
    );
    assert_eq!(count_by_status(&pool, &jt, "queued").await, 1);

    // Saat job sedang running, kiriman ulang tetap ditolak sebagai duplikat.
    let cfg = config(&jt, 10);
    let calls = Cell::new(0);
    let r = run_batch(&pool, &cfg, &clock, |_job| {
        calls.set(calls.get() + 1);
        let pool = pool.clone();
        let job = job.clone();
        async move {
            assert_eq!(
                enqueue(&pool, &job).await.unwrap(),
                EnqueueOutcome::Duplicate
            );
            JobOutcome::Succeeded
        }
    })
    .await
    .unwrap();
    assert_eq!(r.succeeded, 1);
    assert_eq!(calls.get(), 1);
    let r = run_batch(&pool, &cfg, &clock, |_job| async {
        panic!("job yang sama tidak boleh diproses dua kali")
    })
    .await
    .unwrap();
    assert_eq!(r.claimed, 0);
    assert_eq!(count_by_status(&pool, &jt, "succeeded").await, 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_enqueue_with_same_dedupe_key_creates_exactly_one_job() {
    let pool = app_pool().await;
    let jt = unique_type("dedupe_race");
    let clock = clock();
    let entity = new_id();
    let job = NewJob::new(jt.clone(), payload(entity), clock.now(), attempts(1))
        .dedupe_key(DedupeKey::for_entity("uji_race", entity).unwrap());
    let mut handles = Vec::new();
    for _ in 0..8 {
        let pool = pool.clone();
        let job = job.clone();
        handles.push(tokio::spawn(
            async move { enqueue(&pool, &job).await.unwrap() },
        ));
    }
    let mut enqueued = 0;
    for h in handles {
        if matches!(h.await.unwrap(), EnqueueOutcome::Enqueued(_)) {
            enqueued += 1;
        }
    }
    assert_eq!(enqueued, 1);
    assert_eq!(count_by_status(&pool, &jt, "queued").await, 1);
}

#[tokio::test]
async fn dedupe_key_applies_only_to_active_jobs_and_null_keys_never_conflict() {
    // Batas UQp: status queued/running saja (DD A19.4). Setelah job selesai, kunci yang sama boleh
    // dijadwalkan lagi (misalnya pemeriksaan berkala); karena itu handler tetap wajib idempoten (ADR-007).
    let pool = app_pool().await;
    let jt = unique_type("dedupe_done");
    let clock = clock();
    let entity = new_id();
    // Kunci memuat entitas unik per test (kunci berbentuk tanggal seperti `sip_daily_check:2027-03-01`
    // diuji di unit test DedupeKey) supaya test paralel tidak berbagi kunci.
    let job = NewJob::new(jt.clone(), payload(entity), clock.now(), attempts(1))
        .dedupe_key(DedupeKey::for_entity("uji_berkala", entity).unwrap());
    assert!(matches!(
        enqueue(&pool, &job).await.unwrap(),
        EnqueueOutcome::Enqueued(_)
    ));
    let cfg = config(&jt, 10);
    run_batch(&pool, &cfg, &clock, |_job| async { JobOutcome::Succeeded })
        .await
        .unwrap();
    assert!(matches!(
        enqueue(&pool, &job).await.unwrap(),
        EnqueueOutcome::Enqueued(_)
    ));

    // Tanpa kunci deduplikasi: job identik tidak dianggap duplikat.
    let plain = NewJob::new(jt.clone(), payload(entity), clock.now(), attempts(1));
    for _ in 0..2 {
        assert!(matches!(
            enqueue(&pool, &plain).await.unwrap(),
            EnqueueOutcome::Enqueued(_)
        ));
    }
    assert_eq!(count_by_status(&pool, &jt, "queued").await, 3);
}

#[tokio::test]
async fn job_enqueued_in_rolled_back_transaction_does_not_exist() {
    let pool = app_pool().await;
    let jt = unique_type("atomic");
    let clock = clock();
    let job = NewJob::new(
        jt.clone(),
        payload(new_id()),
        clock.now() + Duration::minutes(1),
        attempts(1),
    );
    let mut tx = pool.begin().await.unwrap();
    assert!(matches!(
        enqueue(&mut *tx, &job).await.unwrap(),
        EnqueueOutcome::Enqueued(_)
    ));
    tx.rollback().await.unwrap();
    assert_eq!(count_by_status(&pool, &jt, "queued").await, 0);

    let mut tx = pool.begin().await.unwrap();
    enqueue(&mut *tx, &job).await.unwrap();
    tx.commit().await.unwrap();
    assert_eq!(count_by_status(&pool, &jt, "queued").await, 1);
}
