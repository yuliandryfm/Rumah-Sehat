//! F1-T20: percobaan ulang dengan `attempt_count`/`max_attempts` dan backoff (docs/05-physical-schema.md
//! §9.1, KEPUTUSAN-108). PostgreSQL 18 sekali pakai dari `scripts/db-ephemeral.sh`.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod common;

use common::*;
use foundation::{Clock, new_id};
use platform_jobs::{
    BatchReport, ClaimRequest, ErrorCode, JobOutcome, LeaseOutcome, RetryOutcome, WorkerId, claim,
    complete, enqueue, fail, retry_or_fail, run_batch,
};
use std::cell::Cell;
use time::Duration;

fn code(s: &str) -> ErrorCode {
    ErrorCode::new(s).unwrap()
}

#[tokio::test]
async fn failing_job_is_retried_with_backoff_until_max_attempts_then_failed() {
    let pool = app_pool().await;
    let jt = unique_type("retry_limit");
    let clock = clock();
    let id = enqueue_at(&pool, &jt, clock.now(), 3).await;
    let cfg = config(&jt, 10);
    let calls = Cell::new(0);
    let handler = |job: platform_jobs::ClaimedJob| {
        calls.set(calls.get() + 1);
        let attempt = job.attempt();
        async move {
            assert!(attempt >= 1);
            JobOutcome::Retry(ErrorCode::new("partner_timeout").unwrap())
        }
    };

    // Percobaan 1 gagal: kembali queued, run_at = sekarang + 30 detik (backoff dasar).
    let r = run_batch(&pool, &cfg, &clock, handler).await.unwrap();
    assert_eq!(
        r,
        BatchReport {
            claimed: 1,
            requeued: 1,
            ..BatchReport::default()
        }
    );
    let row = job_row(&pool, id).await;
    assert_eq!(row.status, "queued");
    assert_eq!(row.attempt_count, 1);
    assert_eq!(row.run_at, clock.now() + Duration::seconds(30));
    assert_eq!(row.locked_by, None);
    assert_eq!(row.locked_until, None);
    assert_eq!(row.last_error.as_deref(), Some("partner_timeout"));
    assert_eq!(row.finished_at, None);

    // Sebelum backoff lewat, job tidak diambil.
    clock.advance(Duration::seconds(29));
    let r = run_batch(&pool, &cfg, &clock, handler).await.unwrap();
    assert_eq!(r.claimed, 0);

    // Percobaan 2 gagal: backoff berlipat menjadi 60 detik.
    clock.advance(Duration::seconds(1));
    let r = run_batch(&pool, &cfg, &clock, handler).await.unwrap();
    assert_eq!(r.requeued, 1);
    let row = job_row(&pool, id).await;
    assert_eq!(row.attempt_count, 2);
    assert_eq!(row.run_at, clock.now() + Duration::seconds(60));

    // Percobaan 3 = batas: failed, finished_at terisi, tidak diambil lagi.
    clock.advance(Duration::seconds(60));
    let r = run_batch(&pool, &cfg, &clock, handler).await.unwrap();
    assert_eq!(
        r,
        BatchReport {
            claimed: 1,
            failed: 1,
            ..BatchReport::default()
        }
    );
    let row = job_row(&pool, id).await;
    assert_eq!(row.status, "failed");
    assert_eq!(row.attempt_count, 3);
    assert_eq!(row.finished_at, Some(clock.now()));
    assert_eq!(row.last_error.as_deref(), Some("partner_timeout"));

    clock.advance(Duration::hours(1));
    let r = run_batch(&pool, &cfg, &clock, handler).await.unwrap();
    assert_eq!(r.claimed, 0);
    assert_eq!(calls.get(), 3, "handler dipanggil tepat max_attempts kali");
}

#[tokio::test]
async fn job_that_succeeds_on_retry_is_completed_once() {
    let pool = app_pool().await;
    let jt = unique_type("retry_success");
    let clock = clock();
    let id = enqueue_at(&pool, &jt, clock.now(), 5).await;
    let cfg = config(&jt, 10);
    let handler = |job: platform_jobs::ClaimedJob| async move {
        if job.attempt() < 2 {
            JobOutcome::Retry(ErrorCode::new("temporarily_unavailable").unwrap())
        } else {
            JobOutcome::Succeeded
        }
    };
    assert_eq!(
        run_batch(&pool, &cfg, &clock, handler)
            .await
            .unwrap()
            .requeued,
        1
    );
    clock.advance(Duration::seconds(30));
    assert_eq!(
        run_batch(&pool, &cfg, &clock, handler)
            .await
            .unwrap()
            .succeeded,
        1
    );
    let row = job_row(&pool, id).await;
    assert_eq!(row.status, "succeeded");
    assert_eq!(row.attempt_count, 2);
    assert_eq!(row.last_error, None);
    clock.advance(Duration::hours(1));
    assert_eq!(
        run_batch(&pool, &cfg, &clock, handler)
            .await
            .unwrap()
            .claimed,
        0
    );
}

#[tokio::test]
async fn permanent_failure_and_invalid_payload_stop_without_retry() {
    let pool = app_pool().await;
    let jt = unique_type("permanent");
    let clock = clock();
    let id = enqueue_at(&pool, &jt, clock.now(), 5).await;
    let cfg = config(&jt, 10);
    let r = run_batch(&pool, &cfg, &clock, |_job| async {
        JobOutcome::Fail(ErrorCode::new("entity_not_found").unwrap())
    })
    .await
    .unwrap();
    assert_eq!(r.failed, 1);
    let row = job_row(&pool, id).await;
    assert_eq!(
        (row.status.as_str(), row.attempt_count),
        ("failed", 1),
        "gagal permanen tidak memakai sisa percobaan"
    );
    assert_eq!(row.last_error.as_deref(), Some("entity_not_found"));

    // NEG: payload di tabel yang melanggar aturan "hanya ID" (ditulis di luar crate) tidak diserahkan ke
    // handler; job langsung failed dengan kode invalid_payload.
    let bad = new_id();
    sqlx::query!(
        "INSERT INTO app.job (id, job_type, payload, run_at, status, attempt_count, max_attempts) \
         VALUES ($1, $2, '{\"email\": \"budi@example.com\"}'::jsonb, $3, 'queued', 0, 5)",
        bad,
        jt.as_str(),
        clock.now(),
    )
    .execute(&pool)
    .await
    .unwrap();
    let r = run_batch(&pool, &cfg, &clock, |_job| async {
        panic!("handler tidak boleh menerima payload tidak valid")
    })
    .await
    .unwrap();
    assert_eq!(r.failed, 1);
    let row = sqlx::query!("SELECT status, last_error FROM app.job WHERE id = $1", bad)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(row.status, "failed");
    assert_eq!(row.last_error.as_deref(), Some("invalid_payload"));
}

#[tokio::test]
async fn retry_and_fail_are_fenced_by_the_lease() {
    let pool = app_pool().await;
    let jt = unique_type("fenced_retry");
    let clock = clock();
    let id = enqueue_at(&pool, &jt, clock.now(), 3).await;
    let types = [jt.clone()];
    let worker = WorkerId::new("worker").unwrap();
    let req = ClaimRequest {
        worker: &worker,
        job_types: &types,
        limit: 1,
        lease: Duration::minutes(1),
    };
    let job = claim(&pool, &req, &clock).await.unwrap().pop().unwrap();
    let r = retry_or_fail(
        &pool,
        job.lease(),
        &code("partner_timeout"),
        &backoff(),
        &clock,
    )
    .await
    .unwrap();
    assert_eq!(
        r,
        RetryOutcome::Requeued {
            run_at: clock.now() + Duration::seconds(30)
        }
    );
    // NEG: lease yang sama tidak dapat dipakai lagi setelah job kembali queued.
    assert_eq!(
        retry_or_fail(&pool, job.lease(), &code("again"), &backoff(), &clock)
            .await
            .unwrap(),
        RetryOutcome::LeaseLost
    );
    assert_eq!(
        fail(&pool, job.lease(), &code("again"), &clock)
            .await
            .unwrap(),
        LeaseOutcome::LeaseLost
    );
    assert_eq!(
        complete(&pool, job.lease(), &clock).await.unwrap(),
        LeaseOutcome::LeaseLost
    );
    let row = job_row(&pool, id).await;
    assert_eq!((row.status.as_str(), row.attempt_count), ("queued", 1));
    assert_eq!(row.last_error.as_deref(), Some("partner_timeout"));
}

#[tokio::test]
async fn handler_completing_in_its_own_transaction_commits_effect_and_completion_together() {
    let pool = app_pool().await;
    let jt = unique_type("tx_complete");
    let clock = clock();
    let id = enqueue_at(&pool, &jt, clock.now(), 3).await;
    let cfg = config(&jt, 10);
    let marker = unique_type("effect");
    let pool2 = pool.clone();
    let clock_ref = &clock;
    let r = run_batch(&pool, &cfg, &clock, |job| {
        let pool = pool2.clone();
        let marker = marker.clone();
        async move {
            // Efek bisnis tiruan (baris job lain sebagai penanda) + penandaan selesai, satu transaksi.
            let mut tx = pool.begin().await.unwrap();
            let effect = platform_jobs::NewJob::new(
                marker,
                payload(new_id()),
                clock_ref.now() + Duration::days(365),
                attempts(1),
            );
            enqueue(&mut *tx, &effect).await.unwrap();
            let done = complete(&mut *tx, job.lease(), clock_ref).await.unwrap();
            if done == LeaseOutcome::Applied {
                tx.commit().await.unwrap();
            } else {
                tx.rollback().await.unwrap();
            }
            JobOutcome::Completed(done)
        }
    })
    .await
    .unwrap();
    assert_eq!(r.succeeded, 1);
    assert_eq!(job_row(&pool, id).await.status, "succeeded");
    assert_eq!(count_by_status(&pool, &marker, "queued").await, 1);
}
