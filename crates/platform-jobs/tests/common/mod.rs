//! Pembantu test integrasi PostgreSQL `platform-jobs`. Database disediakan `scripts/db-ephemeral.sh`
//! (PostgreSQL 18 sekali pakai, bootstrap db/roles/app.sql, migrasi db/migrations/app), pola yang sama
//! dengan crate audit (F1-T23). Koneksi memakai login `rs_api` (anggota `rs_app`): hak tabel `job`
//! sungguhan, sama dengan `rs_worker`. Tanpa variabelnya test GAGAL, tidak dilewati diam-diam.
//!
//! Test berjalan paralel di database yang sama; setiap test memakai `job_type` unik sehingga pengambilan
//! dan penyapuan (keduanya difilter `job_type`) tidak saling mengganggu.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, dead_code)]

use foundation::{FixedClock, Uuid, new_id};
use observability::EnvSource;
use platform_jobs::{
    Backoff, EnqueueOutcome, JobId, JobPayload, JobType, MaxAttempts, NewJob, WorkerConfig,
    WorkerId, enqueue,
};
use sqlx::postgres::PgPoolOptions;
use sqlx::{Connection, PgConnection, PgPool};
use time::macros::datetime;
use time::{Duration, OffsetDateTime};

const APP_URL: &str = "RS_TEST_DB_APP_URL";

fn app_url() -> String {
    let env = EnvSource::from_process();
    let url = env.required_secret(APP_URL).unwrap_or_else(|_| {
        panic!(
            "{APP_URL} tidak diset: test integrasi platform-jobs wajib dijalankan lewat \
             `bash scripts/db-ephemeral.sh cargo test ...` (PostgreSQL 18 sekali pakai)"
        )
    });
    url.expose().to_string()
}

/// Pool login `rs_api` (anggota `rs_app`).
pub async fn app_pool() -> PgPool {
    PgPoolOptions::new()
        .max_connections(6)
        .connect(&app_url())
        .await
        .unwrap_or_else(|_| panic!("koneksi {APP_URL} gagal"))
}

/// Koneksi khusus di luar pool (untuk kunci advisory).
pub async fn dedicated_connection() -> PgConnection {
    PgConnection::connect(&app_url())
        .await
        .unwrap_or_else(|_| panic!("koneksi khusus {APP_URL} gagal"))
}

/// Jam awal test: 1 Maret 2027 10.00 WIB.
pub fn start() -> OffsetDateTime {
    datetime!(2027-03-01 10:00:00 +07:00)
}

pub fn clock() -> FixedClock {
    FixedClock::new(start())
}

/// Jenis job unik per test.
pub fn unique_type(prefix: &str) -> JobType {
    let suffix: String = new_id().simple().to_string();
    JobType::new(&format!("uji_{prefix}_{suffix}")).unwrap()
}

pub fn payload(id: Uuid) -> JobPayload {
    JobPayload::new().with_id("entity_id", id).unwrap()
}

pub fn attempts(n: i16) -> MaxAttempts {
    MaxAttempts::new(n).unwrap()
}

pub fn backoff() -> Backoff {
    Backoff::new(Duration::seconds(30), Duration::minutes(10)).unwrap()
}

pub fn config(job_type: &JobType, batch: u16) -> WorkerConfig {
    WorkerConfig::new(
        WorkerId::new("worker").unwrap(),
        vec![job_type.clone()],
        batch,
        Duration::minutes(5),
        backoff(),
    )
    .unwrap()
}

/// Menulis job jatuh tempo `run_at` dan mengembalikan ID-nya.
pub async fn enqueue_at(
    pool: &PgPool,
    job_type: &JobType,
    run_at: OffsetDateTime,
    max: i16,
) -> JobId {
    let job = NewJob::new(job_type.clone(), payload(new_id()), run_at, attempts(max));
    match enqueue(pool, &job).await.unwrap() {
        EnqueueOutcome::Enqueued(id) => id,
        EnqueueOutcome::Duplicate => panic!("job tanpa kunci deduplikasi tidak mungkin duplikat"),
    }
}

/// Keadaan satu baris job.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobRow {
    pub status: String,
    pub attempt_count: i16,
    pub run_at: OffsetDateTime,
    pub locked_by: Option<String>,
    pub locked_until: Option<OffsetDateTime>,
    pub last_error: Option<String>,
    pub finished_at: Option<OffsetDateTime>,
}

pub async fn job_row(pool: &PgPool, id: JobId) -> JobRow {
    let r = sqlx::query!(
        "SELECT status, attempt_count, run_at, locked_by, locked_until, last_error, finished_at \
           FROM app.job WHERE id = $1",
        id.as_uuid()
    )
    .fetch_one(pool)
    .await
    .unwrap();
    JobRow {
        status: r.status,
        attempt_count: r.attempt_count,
        run_at: r.run_at,
        locked_by: r.locked_by,
        locked_until: r.locked_until,
        last_error: r.last_error,
        finished_at: r.finished_at,
    }
}

/// Jumlah job per status untuk satu jenis job.
pub async fn count_by_status(pool: &PgPool, job_type: &JobType, status: &str) -> i64 {
    sqlx::query_scalar!(
        r#"SELECT count(*) AS "n!" FROM app.job WHERE job_type = $1 AND status = $2"#,
        job_type.as_str(),
        status
    )
    .fetch_one(pool)
    .await
    .unwrap()
}
