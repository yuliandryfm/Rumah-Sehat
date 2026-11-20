//! F1-T20: pengambilan job dengan `FOR UPDATE SKIP LOCKED` dan sewa (ADR-007 butir 3, KEPUTUSAN-108).
//! PostgreSQL 18 sekali pakai dari `scripts/db-ephemeral.sh`; tanpa itu test gagal (bukan dilewati).
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use common::*;
use foundation::{Clock, FixedClock};
use platform_jobs::{
    ClaimRequest, JobErrorKind, JobId, JobType, LeaseOutcome, SweepReport, WorkerId, claim,
    complete, sweep_expired,
};
use std::collections::BTreeSet;
use std::sync::Arc;
use time::Duration;
use time::macros::datetime;

fn request<'a>(worker: &'a WorkerId, types: &'a [JobType], limit: u16) -> ClaimRequest<'a> {
    ClaimRequest {
        worker,
        job_types: types,
        limit,
        lease: Duration::minutes(5),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_workers_never_claim_the_same_job() {
    let pool = app_pool().await;
    let jt = unique_type("concurrent");
    let clock = Arc::new(clock());
    let mut all = BTreeSet::new();
    for _ in 0..60 {
        all.insert(enqueue_at(&pool, &jt, clock.now(), 1).await);
    }

    let mut handles = Vec::new();
    for _ in 0..4 {
        let pool = pool.clone();
        let jt = jt.clone();
        let clock = Arc::clone(&clock);
        handles.push(tokio::spawn(async move {
            let worker = WorkerId::new("worker").unwrap();
            let types = [jt];
            let mut mine = Vec::new();
            loop {
                let got = claim(&pool, &request(&worker, &types, 3), clock.as_ref())
                    .await
                    .unwrap();
                if got.is_empty() {
                    break;
                }
                mine.extend(got.into_iter().map(|j| j.id()));
            }
            mine
        }));
    }
    let mut claimed: Vec<JobId> = Vec::new();
    for h in handles {
        claimed.extend(h.await.unwrap());
    }
    let unique: BTreeSet<JobId> = claimed.iter().copied().collect();
    assert_eq!(unique.len(), claimed.len(), "satu job diambil dua pekerja");
    assert_eq!(unique, all, "semua job terambil tepat sekali");
    assert_eq!(count_by_status(&pool, &jt, "running").await, 60);
}

#[tokio::test]
async fn row_locked_by_another_transaction_is_skipped_not_awaited() {
    let pool = app_pool().await;
    let jt = unique_type("skip_locked");
    let clock = clock();
    let first = enqueue_at(&pool, &jt, clock.now() - Duration::minutes(2), 1).await;
    let second = enqueue_at(&pool, &jt, clock.now() - Duration::minutes(1), 1).await;
    let types = [jt.clone()];

    // Pekerja A mengambil job pertama di transaksi yang belum di-commit (kunci baris masih dipegang).
    let worker_a = WorkerId::new("worker").unwrap();
    let mut tx = pool.begin().await.unwrap();
    let a = claim(&mut *tx, &request(&worker_a, &types, 1), &clock)
        .await
        .unwrap();
    assert_eq!(a.iter().map(|j| j.id()).collect::<Vec<_>>(), vec![first]);

    // Pekerja B tidak menunggu kunci baris A (lock_timeout rs_api 2 detik akan menggagalkannya), tetapi
    // melewatinya dan mendapat job kedua.
    let worker_b = WorkerId::new("worker").unwrap();
    let b = claim(&pool, &request(&worker_b, &types, 5), &clock)
        .await
        .unwrap();
    assert_eq!(b.iter().map(|j| j.id()).collect::<Vec<_>>(), vec![second]);

    // A batal: job pertama kembali queued tanpa percobaan terpakai, lalu dapat diambil lagi.
    tx.rollback().await.unwrap();
    let row = job_row(&pool, first).await;
    assert_eq!((row.status.as_str(), row.attempt_count), ("queued", 0));
    let again = claim(&pool, &request(&worker_b, &types, 5), &clock)
        .await
        .unwrap();
    assert_eq!(
        again.iter().map(|j| j.id()).collect::<Vec<_>>(),
        vec![first]
    );
}

#[tokio::test]
async fn claim_takes_only_due_jobs_of_handled_types_in_run_at_order() {
    let pool = app_pool().await;
    let jt = unique_type("due");
    let other = unique_type("other");
    let clock = clock();
    let late = enqueue_at(&pool, &jt, clock.now() - Duration::minutes(1), 1).await;
    let early = enqueue_at(&pool, &jt, clock.now() - Duration::hours(1), 1).await;
    let future = enqueue_at(&pool, &jt, clock.now() + Duration::seconds(1), 1).await;
    let foreign = enqueue_at(&pool, &other, clock.now() - Duration::hours(2), 1).await;
    let worker = WorkerId::new("worker").unwrap();
    let types = [jt.clone()];

    let got = claim(&pool, &request(&worker, &types, 10), &clock)
        .await
        .unwrap();
    assert_eq!(
        got.iter().map(|j| j.id()).collect::<Vec<_>>(),
        vec![early, late]
    );
    let job = got.first().unwrap();
    assert_eq!(job.attempt(), 1);
    assert_eq!(job.max_attempts(), 1);
    assert_eq!(job.job_type(), &jt);
    assert!(job.payload().unwrap().id("entity_id").is_some());
    assert_eq!(
        job.lease().locked_until(),
        clock.now() + Duration::minutes(5)
    );
    let row = job_row(&pool, early).await;
    assert_eq!(row.status, "running");
    assert_eq!(row.locked_by.as_deref(), Some(worker.as_str()));
    assert_eq!(row.locked_until, Some(clock.now() + Duration::minutes(5)));

    // Job masa depan dan job jenis lain tidak tersentuh.
    assert_eq!(job_row(&pool, future).await.status, "queued");
    assert_eq!(job_row(&pool, foreign).await.status, "queued");
    // Setelah jam maju, job masa depan jatuh tempo.
    clock.advance(Duration::seconds(1));
    let got = claim(&pool, &request(&worker, &types, 10), &clock)
        .await
        .unwrap();
    assert_eq!(got.iter().map(|j| j.id()).collect::<Vec<_>>(), vec![future]);
}

#[tokio::test]
async fn run_at_in_two_indonesian_zones_is_compared_as_absolute_instant() {
    // ADR-017: instan absolut; 10.00 WIT (+09) = 08.00 WIB (+07) = 01.00 UTC.
    let pool = app_pool().await;
    let jt = unique_type("zones");
    let wit = enqueue_at(&pool, &jt, datetime!(2027-03-01 10:00:00 +09:00), 1).await;
    let wib = enqueue_at(&pool, &jt, datetime!(2027-03-01 08:30:00 +07:00), 1).await;
    let worker = WorkerId::new("worker").unwrap();
    let types = [jt.clone()];

    // 08.15 WIB: job WIT (08.00 WIB) jatuh tempo, job WIB 08.30 belum.
    let clock = FixedClock::new(datetime!(2027-03-01 08:15:00 +07:00));
    let got = claim(&pool, &request(&worker, &types, 10), &clock)
        .await
        .unwrap();
    assert_eq!(got.iter().map(|j| j.id()).collect::<Vec<_>>(), vec![wit]);
    assert_eq!(
        job_row(&pool, wit).await.run_at,
        datetime!(2027-03-01 01:00:00 UTC)
    );
    // 10.30 WIT = 08.30 WIB: job WIB jatuh tempo.
    clock.set(datetime!(2027-03-01 10:30:00 +09:00));
    let got = claim(&pool, &request(&worker, &types, 10), &clock)
        .await
        .unwrap();
    assert_eq!(got.iter().map(|j| j.id()).collect::<Vec<_>>(), vec![wib]);
}

#[tokio::test]
async fn completion_by_worker_whose_lease_was_taken_over_is_discarded() {
    let pool = app_pool().await;
    let jt = unique_type("stale_lease");
    let clock = clock();
    let id = enqueue_at(&pool, &jt, clock.now(), 3).await;
    let types = [jt.clone()];
    let worker_a = WorkerId::new("worker").unwrap();
    let worker_b = WorkerId::new("worker").unwrap();

    let a = claim(&pool, &request(&worker_a, &types, 1), &clock)
        .await
        .unwrap()
        .pop()
        .unwrap();
    // A dianggap mati: sewa habis, penyapu mengembalikan job, B mengambilnya.
    clock.advance(Duration::minutes(5) + Duration::seconds(1));
    let swept = sweep_expired(&pool, &types, &clock).await.unwrap();
    assert_eq!(
        swept,
        SweepReport {
            requeued: 1,
            failed: 0
        }
    );
    let swept_row = job_row(&pool, id).await;
    assert_eq!(swept_row.status, "queued");
    assert_eq!(swept_row.locked_by, None);
    assert_eq!(swept_row.last_error.as_deref(), Some("lease_expired"));
    let b = claim(&pool, &request(&worker_b, &types, 1), &clock)
        .await
        .unwrap()
        .pop()
        .unwrap();
    assert_eq!(b.attempt(), 2);

    // NEG: A yang "hidup kembali" tidak dapat menyelesaikan job; efeknya di transaksi yang sama dibatalkan.
    let mut tx = pool.begin().await.unwrap();
    assert_eq!(
        complete(&mut *tx, a.lease(), &clock).await.unwrap(),
        LeaseOutcome::LeaseLost
    );
    tx.rollback().await.unwrap();
    // NEG: di luar transaksi pun A tetap ditolak (pagar locked_by + attempt_count); hanya B yang berhasil.
    assert_eq!(
        complete(&pool, a.lease(), &clock).await.unwrap(),
        LeaseOutcome::LeaseLost
    );
    assert_eq!(
        complete(&pool, b.lease(), &clock).await.unwrap(),
        LeaseOutcome::Applied
    );
    let row = job_row(&pool, id).await;
    assert_eq!(row.status, "succeeded");
    assert_eq!(row.finished_at, Some(clock.now()));
    assert_eq!(row.locked_until, None);
    assert_eq!(row.last_error, None);
    // Selesai dua kali tidak mungkin.
    assert_eq!(
        complete(&pool, b.lease(), &clock).await.unwrap(),
        LeaseOutcome::LeaseLost
    );
}

#[tokio::test]
async fn sweep_ignores_live_leases_and_fails_jobs_at_attempt_limit() {
    let pool = app_pool().await;
    let jt = unique_type("sweep");
    let clock = clock();
    let id = enqueue_at(&pool, &jt, clock.now(), 1).await;
    let types = [jt.clone()];
    let worker = WorkerId::new("worker").unwrap();
    claim(&pool, &request(&worker, &types, 1), &clock)
        .await
        .unwrap();

    // Sewa masih berlaku (locked_until = sekarang, bukan lewat): tidak disapu.
    clock.advance(Duration::minutes(5));
    assert_eq!(
        sweep_expired(&pool, &types, &clock).await.unwrap(),
        SweepReport::default()
    );
    assert_eq!(job_row(&pool, id).await.status, "running");

    // Lewat sewa dan percobaan habis (1/1): failed, bukan diulang tanpa akhir.
    clock.advance(Duration::seconds(1));
    assert_eq!(
        sweep_expired(&pool, &types, &clock).await.unwrap(),
        SweepReport {
            requeued: 0,
            failed: 1
        }
    );
    let row = job_row(&pool, id).await;
    assert_eq!(row.status, "failed");
    assert_eq!(row.finished_at, Some(clock.now()));
    assert_eq!(row.locked_by.as_deref(), Some(worker.as_str()));
    assert_eq!(row.last_error.as_deref(), Some("lease_expired"));
}

#[tokio::test]
async fn invalid_claim_requests_are_rejected_before_touching_the_database() {
    let pool = app_pool().await;
    let clock = clock();
    let worker = WorkerId::new("worker").unwrap();
    let jt = [unique_type("invalid")];
    for req in [
        request(&worker, &[], 1),
        request(&worker, &jt, 0),
        ClaimRequest {
            lease: Duration::ZERO,
            ..request(&worker, &jt, 1)
        },
    ] {
        assert_eq!(
            claim(&pool, &req, &clock).await.err().map(|e| e.kind()),
            Some(JobErrorKind::InvalidConfig)
        );
    }
    assert_eq!(
        sweep_expired(&pool, &[], &clock)
            .await
            .err()
            .map(|e| e.kind()),
        Some(JobErrorKind::InvalidConfig)
    );
}
