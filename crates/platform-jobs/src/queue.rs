//! Antrean job di tabel `app.job` (ADR-007 butir 3, DD A19.4) dengan pola sewa KEPUTUSAN-108
//! (docs/05-physical-schema.md §9.1):
//!
//! - [`enqueue`] menulis job di transaksi pemanggil, sehingga job dan perubahan bisnisnya sama-sama
//!   tersimpan atau sama-sama batal. Kunci deduplikasi yang sama selama job masih `queued`/`running`
//!   menghasilkan [`EnqueueOutcome::Duplicate`] (`uqp_job_dedupe_key`).
//! - [`claim`] mengambil job jatuh tempo dengan `FOR UPDATE SKIP LOCKED` [SRC-36], menandainya `running`
//!   dengan `locked_by`/`locked_until`, dan menaikkan `attempt_count`; satu pernyataan, di-commit segera.
//! - [`complete`], [`retry_or_fail`], dan [`fail`] dipagari sewa: `status = 'running' AND locked_by AND
//!   attempt_count` sama dengan saat diambil. Nol baris = sewa sudah diambil alih ([`LeaseOutcome::LeaseLost`]);
//!   hasil pekerja lama dibuang. [`complete`] menerima transaksi pemanggil, sehingga efek job di database dan
//!   penandaan selesai dapat di-commit bersama (efek tidak terjadi dua kali bila sewa hilang).
//! - [`sweep_expired`] mengembalikan job `running` yang sewanya habis ke `queued`, atau `failed` bila
//!   `attempt_count >= max_attempts`.
//!
//! Waktu "sekarang" selalu dari [`Clock`] yang diinjeksi (coding-standards §5.7, KEPUTUSAN-131), bukan
//! `now()` server, supaya jadwal, sewa, dan backoff dapat diuji dengan jam tetap.

use crate::backoff::Backoff;
use crate::error::{JobError, JobErrorKind};
use crate::types::{DedupeKey, ErrorCode, JobId, JobPayload, JobType, MaxAttempts, WorkerId};
use foundation::Clock;
use serde_json::Value;
use sqlx::postgres::PgExecutor;
use time::{Duration, OffsetDateTime};

/// Job baru.
#[derive(Debug, Clone)]
pub struct NewJob {
    job_type: JobType,
    payload: JobPayload,
    run_at: OffsetDateTime,
    max_attempts: MaxAttempts,
    dedupe_key: Option<DedupeKey>,
}

impl NewJob {
    /// Job yang jatuh tempo pada `run_at` (instan absolut, ADR-017 butir 4).
    pub fn new(
        job_type: JobType,
        payload: JobPayload,
        run_at: OffsetDateTime,
        max_attempts: MaxAttempts,
    ) -> Self {
        Self {
            job_type,
            payload,
            run_at,
            max_attempts,
            dedupe_key: None,
        }
    }

    /// Menetapkan kunci deduplikasi.
    pub fn dedupe_key(mut self, key: DedupeKey) -> Self {
        self.dedupe_key = Some(key);
        self
    }
}

/// Hasil [`enqueue`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnqueueOutcome {
    /// Job baru tersimpan.
    Enqueued(JobId),
    /// Sudah ada job `queued`/`running` dengan kunci deduplikasi yang sama; tidak ada job baru.
    Duplicate,
}

/// Menulis job di transaksi pemanggil (`&mut *tx`) atau langsung di pool.
pub async fn enqueue<'e, E: PgExecutor<'e>>(
    executor: E,
    job: &NewJob,
) -> Result<EnqueueOutcome, JobError> {
    let id = JobId::new();
    // ON CONFLICT DO NOTHING dengan predikat indeks parsial: butuh INSERT + SELECT kolom dedupe_key/status
    // (SRC-512); rs_app punya keduanya (DD §0.6 A, kelas O).
    let done = sqlx::query!(
        "INSERT INTO app.job (id, job_type, payload, run_at, status, attempt_count, max_attempts, dedupe_key) \
         VALUES ($1, $2, $3, $4, 'queued', 0, $5, $6) \
         ON CONFLICT (dedupe_key) WHERE status IN ('queued', 'running') DO NOTHING",
        id.as_uuid(),
        job.job_type.as_str(),
        job.payload.to_json(),
        job.run_at,
        job.max_attempts.get(),
        job.dedupe_key.as_ref().map(DedupeKey::as_str),
    )
    .execute(executor)
    .await?;
    Ok(if done.rows_affected() == 1 {
        EnqueueOutcome::Enqueued(id)
    } else {
        EnqueueOutcome::Duplicate
    })
}

/// Pagar sewa satu job yang diambil: dipakai untuk menyelesaikan atau menjadwal ulang job itu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lease {
    job_id: JobId,
    worker: WorkerId,
    attempt: i16,
    locked_until: OffsetDateTime,
}

impl Lease {
    /// ID job.
    pub fn job_id(&self) -> JobId {
        self.job_id
    }

    /// Nomor percobaan ini (`attempt_count` setelah diambil, dimulai dari 1).
    pub fn attempt(&self) -> i16 {
        self.attempt
    }

    /// Akhir masa sewa.
    pub fn locked_until(&self) -> OffsetDateTime {
        self.locked_until
    }
}

/// Job yang sedang disewa pekerja.
#[derive(Debug, Clone)]
pub struct ClaimedJob {
    lease: Lease,
    job_type: JobType,
    payload: Value,
    max_attempts: i16,
    dedupe_key: Option<String>,
}

impl ClaimedJob {
    /// Pagar sewa.
    pub fn lease(&self) -> &Lease {
        &self.lease
    }

    /// ID job.
    pub fn id(&self) -> JobId {
        self.lease.job_id
    }

    /// Jenis job.
    pub fn job_type(&self) -> &JobType {
        &self.job_type
    }

    /// Nomor percobaan ini (dimulai dari 1).
    pub fn attempt(&self) -> i16 {
        self.lease.attempt
    }

    /// Batas percobaan.
    pub fn max_attempts(&self) -> i16 {
        self.max_attempts
    }

    /// Kunci deduplikasi job, bila ada.
    pub fn dedupe_key(&self) -> Option<&str> {
        self.dedupe_key.as_deref()
    }

    /// Payload, diurai dengan aturan yang sama dengan pembuatannya.
    pub fn payload(&self) -> Result<JobPayload, JobError> {
        JobPayload::from_json(&self.payload)
    }
}

fn add(at: OffsetDateTime, d: Duration) -> Result<OffsetDateTime, JobError> {
    at.checked_add(d)
        .ok_or_else(|| JobError::new(JobErrorKind::TimeOutOfRange))
}

/// Parameter pengambilan job.
#[derive(Debug, Clone)]
pub struct ClaimRequest<'a> {
    /// Identitas pekerja (`locked_by`).
    pub worker: &'a WorkerId,
    /// Jenis job yang ditangani pekerja; job jenis lain tidak diambil.
    pub job_types: &'a [JobType],
    /// Jumlah maksimum job per pengambilan (minimal 1).
    pub limit: u16,
    /// Masa sewa (positif).
    pub lease: Duration,
}

/// Mengambil sampai `limit` job jatuh tempo (`run_at <= sekarang`) dengan `FOR UPDATE SKIP LOCKED`:
/// baris yang sedang dikunci transaksi lain dilewati, tidak ditunggu, sehingga dua pekerja tidak pernah
/// mengambil job yang sama. Urutan `run_at, id`.
pub async fn claim<'e, E: PgExecutor<'e>>(
    executor: E,
    req: &ClaimRequest<'_>,
    clock: &dyn Clock,
) -> Result<Vec<ClaimedJob>, JobError> {
    if req.limit == 0 || req.job_types.is_empty() || !req.lease.is_positive() {
        return Err(JobError::new(JobErrorKind::InvalidConfig));
    }
    let now = clock.now();
    let locked_until = add(now, req.lease)?;
    let types: Vec<String> = req
        .job_types
        .iter()
        .map(|t| t.as_str().to_string())
        .collect();
    let rows = sqlx::query!(
        "WITH next AS ( \
           SELECT id FROM app.job \
            WHERE status = 'queued' AND run_at <= $1 AND job_type = ANY($2::text[]) \
            ORDER BY run_at, id \
            LIMIT $3 \
            FOR UPDATE SKIP LOCKED \
         ) \
         UPDATE app.job j \
            SET status = 'running', locked_by = $4, locked_until = $5, attempt_count = j.attempt_count + 1 \
           FROM next \
          WHERE j.id = next.id \
         RETURNING j.id, j.job_type, j.payload, j.run_at, j.attempt_count, j.max_attempts, j.dedupe_key",
        now,
        &types,
        i64::from(req.limit),
        req.worker.as_str(),
        locked_until,
    )
    .fetch_all(executor)
    .await?;
    let mut out = Vec::with_capacity(rows.len());
    for r in rows {
        out.push((
            r.run_at,
            ClaimedJob {
                lease: Lease {
                    job_id: JobId::from_uuid(r.id),
                    worker: req.worker.clone(),
                    attempt: r.attempt_count,
                    locked_until,
                },
                job_type: JobType::new(&r.job_type)?,
                payload: r.payload,
                max_attempts: r.max_attempts,
                dedupe_key: r.dedupe_key,
            },
        ));
    }
    // Urutan RETURNING tidak dijamin; kembalikan urut jatuh tempo (run_at, id) seperti urutan pengambilan.
    out.sort_by_key(|(run_at, j)| (*run_at, j.lease.job_id));
    Ok(out.into_iter().map(|(_, j)| j).collect())
}

/// Hasil operasi yang dipagari sewa.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeaseOutcome {
    /// Perubahan diterapkan.
    Applied,
    /// Sewa sudah tidak dipegang pekerja ini (disapu atau diambil alih); tidak ada perubahan.
    LeaseLost,
}

fn outcome(rows: u64) -> LeaseOutcome {
    if rows == 1 {
        LeaseOutcome::Applied
    } else {
        LeaseOutcome::LeaseLost
    }
}

/// Menandai job `succeeded`. Dapat dipanggil di transaksi yang sama dengan efek job; bila hasilnya
/// [`LeaseOutcome::LeaseLost`], pemanggil wajib membatalkan transaksinya.
pub async fn complete<'e, E: PgExecutor<'e>>(
    executor: E,
    lease: &Lease,
    clock: &dyn Clock,
) -> Result<LeaseOutcome, JobError> {
    let done = sqlx::query!(
        "UPDATE app.job \
            SET status = 'succeeded', finished_at = $4, locked_until = NULL, last_error = NULL \
          WHERE id = $1 AND status = 'running' AND locked_by = $2 AND attempt_count = $3",
        lease.job_id.as_uuid(),
        lease.worker.as_str(),
        lease.attempt,
        clock.now(),
    )
    .execute(executor)
    .await?;
    Ok(outcome(done.rows_affected()))
}

/// Hasil [`retry_or_fail`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetryOutcome {
    /// Kembali `queued`, jatuh tempo lagi pada `run_at`.
    Requeued {
        /// Jadwal percobaan berikutnya.
        run_at: OffsetDateTime,
    },
    /// Batas `max_attempts` tercapai; job `failed`.
    Failed,
    /// Sewa sudah tidak dipegang pekerja ini; tidak ada perubahan.
    LeaseLost,
}

/// Percobaan gagal: kembali `queued` dengan `run_at = sekarang + backoff`, atau `failed` bila
/// `attempt_count >= max_attempts`. `last_error` diisi kode galat.
pub async fn retry_or_fail<'e, E: PgExecutor<'e>>(
    executor: E,
    lease: &Lease,
    error: &ErrorCode,
    backoff: &Backoff,
    clock: &dyn Clock,
) -> Result<RetryOutcome, JobError> {
    let now = clock.now();
    let next_run_at = add(now, backoff.delay_after(lease.attempt))?;
    let row = sqlx::query!(
        "UPDATE app.job \
            SET status = CASE WHEN attempt_count >= max_attempts THEN 'failed' ELSE 'queued' END, \
                run_at = CASE WHEN attempt_count >= max_attempts THEN run_at ELSE $4::timestamptz END, \
                finished_at = CASE WHEN attempt_count >= max_attempts THEN $5::timestamptz ELSE NULL END, \
                locked_by = CASE WHEN attempt_count >= max_attempts THEN locked_by ELSE NULL END, \
                locked_until = NULL, \
                last_error = $6 \
          WHERE id = $1 AND status = 'running' AND locked_by = $2 AND attempt_count = $3 \
         RETURNING status, run_at",
        lease.job_id.as_uuid(),
        lease.worker.as_str(),
        lease.attempt,
        next_run_at,
        now,
        error.as_str(),
    )
    .fetch_optional(executor)
    .await?;
    Ok(match row {
        None => RetryOutcome::LeaseLost,
        Some(r) if r.status == "queued" => RetryOutcome::Requeued { run_at: r.run_at },
        Some(_) => RetryOutcome::Failed,
    })
}

/// Kegagalan permanen (tidak dicoba ulang walaupun percobaan masih tersisa).
pub async fn fail<'e, E: PgExecutor<'e>>(
    executor: E,
    lease: &Lease,
    error: &ErrorCode,
    clock: &dyn Clock,
) -> Result<LeaseOutcome, JobError> {
    let done = sqlx::query!(
        "UPDATE app.job \
            SET status = 'failed', finished_at = $4, locked_until = NULL, last_error = $5 \
          WHERE id = $1 AND status = 'running' AND locked_by = $2 AND attempt_count = $3",
        lease.job_id.as_uuid(),
        lease.worker.as_str(),
        lease.attempt,
        clock.now(),
        error.as_str(),
    )
    .execute(executor)
    .await?;
    Ok(outcome(done.rows_affected()))
}

/// Hasil [`sweep_expired`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SweepReport {
    /// Job yang dikembalikan ke `queued`.
    pub requeued: u64,
    /// Job yang menjadi `failed` karena batas percobaan tercapai.
    pub failed: u64,
}

/// Penyapu: job `running` dengan `locked_until < sekarang` (pekerja mati) dari jenis `job_types`
/// dikembalikan ke `queued`, atau `failed` bila `attempt_count >= max_attempts`, dengan
/// `last_error = 'lease_expired'` (docs/05-physical-schema.md §9.1, review N06 b).
pub async fn sweep_expired<'e, E: PgExecutor<'e>>(
    executor: E,
    job_types: &[JobType],
    clock: &dyn Clock,
) -> Result<SweepReport, JobError> {
    if job_types.is_empty() {
        return Err(JobError::new(JobErrorKind::InvalidConfig));
    }
    let types: Vec<String> = job_types.iter().map(|t| t.as_str().to_string()).collect();
    let rows = sqlx::query!(
        "UPDATE app.job \
            SET status = CASE WHEN attempt_count >= max_attempts THEN 'failed' ELSE 'queued' END, \
                finished_at = CASE WHEN attempt_count >= max_attempts THEN $1::timestamptz ELSE NULL END, \
                locked_by = CASE WHEN attempt_count >= max_attempts THEN locked_by ELSE NULL END, \
                locked_until = NULL, \
                last_error = $3 \
          WHERE status = 'running' AND locked_until < $1::timestamptz AND job_type = ANY($2::text[]) \
         RETURNING status",
        clock.now(),
        &types,
        ErrorCode::LEASE_EXPIRED,
    )
    .fetch_all(executor)
    .await?;
    let mut report = SweepReport::default();
    for r in rows {
        if r.status == "queued" {
            report.requeued += 1;
        } else {
            report.failed += 1;
        }
    }
    Ok(report)
}
