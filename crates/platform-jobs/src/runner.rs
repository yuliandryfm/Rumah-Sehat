//! Satu putaran pekerja: ambil batch, jalankan handler per job, lalu selesaikan atau jadwalkan ulang
//! dengan pagar sewa (docs/05-physical-schema.md §9.1).
//!
//! Handler menerima [`ClaimedJob`] tanpa transaksi terbuka, supaya boleh memanggil pihak ketiga
//! (dilarang selama transaksi database terbuka, KEPUTUSAN-125). Handler yang efeknya di database
//! membuka transaksinya sendiri dan memanggil [`complete`](crate::complete) di transaksi itu
//! (hasilnya dikembalikan sebagai [`JobOutcome::Completed`]), sehingga efek dan penandaan selesai
//! di-commit bersama. Job wajib idempoten (ADR-007 konsekuensi): job dapat dijalankan ulang setelah
//! pekerja mati di tengah jalan.

use crate::backoff::Backoff;
use crate::error::{JobError, JobErrorKind};
use crate::queue::{
    ClaimRequest, ClaimedJob, LeaseOutcome, RetryOutcome, claim, complete, fail, retry_or_fail,
};
use crate::types::{ErrorCode, JobType, WorkerId};
use foundation::Clock;
use sqlx::PgPool;
use std::future::Future;
use time::Duration;

/// Batas ukuran batch per putaran.
pub const MAX_BATCH_SIZE: u16 = 100;

/// Konfigurasi pekerja (nilai per deployment; dokumen belum menetapkan angka `[TIDAK DIKETAHUI]`).
#[derive(Debug, Clone)]
pub struct WorkerConfig {
    worker: WorkerId,
    job_types: Vec<JobType>,
    batch_size: u16,
    lease: Duration,
    backoff: Backoff,
}

impl WorkerConfig {
    /// Memvalidasi konfigurasi: minimal satu jenis job, batch `1..=MAX_BATCH_SIZE`, sewa positif.
    pub fn new(
        worker: WorkerId,
        job_types: Vec<JobType>,
        batch_size: u16,
        lease: Duration,
        backoff: Backoff,
    ) -> Result<Self, JobError> {
        if job_types.is_empty()
            || batch_size == 0
            || batch_size > MAX_BATCH_SIZE
            || !lease.is_positive()
        {
            return Err(JobError::new(JobErrorKind::InvalidConfig));
        }
        Ok(Self {
            worker,
            job_types,
            batch_size,
            lease,
            backoff,
        })
    }

    /// Identitas pekerja.
    pub fn worker(&self) -> &WorkerId {
        &self.worker
    }

    /// Jenis job yang ditangani.
    pub fn job_types(&self) -> &[JobType] {
        &self.job_types
    }

    /// Backoff percobaan ulang.
    pub fn backoff(&self) -> &Backoff {
        &self.backoff
    }

    fn claim_request(&self) -> ClaimRequest<'_> {
        ClaimRequest {
            worker: &self.worker,
            job_types: &self.job_types,
            limit: self.batch_size,
            lease: self.lease,
        }
    }
}

/// Hasil handler untuk satu job.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobOutcome {
    /// Berhasil; runner menandai `succeeded` di transaksi tersendiri.
    Succeeded,
    /// Handler sudah memanggil [`complete`](crate::complete) di transaksinya sendiri; ini hasilnya.
    Completed(LeaseOutcome),
    /// Gagal sementara: dicoba ulang dengan backoff sampai `max_attempts`.
    Retry(ErrorCode),
    /// Gagal permanen: langsung `failed`.
    Fail(ErrorCode),
}

/// Ringkasan satu putaran.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BatchReport {
    /// Job yang diambil.
    pub claimed: usize,
    /// Job yang selesai `succeeded`.
    pub succeeded: usize,
    /// Job yang dijadwalkan ulang.
    pub requeued: usize,
    /// Job yang menjadi `failed`.
    pub failed: usize,
    /// Job yang sewanya hilang sebelum hasilnya dicatat (hasil dibuang).
    pub lease_lost: usize,
}

/// Menjalankan satu putaran: ambil sampai `batch_size` job lalu proses berurutan.
pub async fn run_batch<F, Fut>(
    pool: &PgPool,
    cfg: &WorkerConfig,
    clock: &dyn Clock,
    mut handler: F,
) -> Result<BatchReport, JobError>
where
    F: FnMut(ClaimedJob) -> Fut,
    Fut: Future<Output = JobOutcome>,
{
    let jobs = claim(pool, &cfg.claim_request(), clock).await?;
    let mut report = BatchReport {
        claimed: jobs.len(),
        ..BatchReport::default()
    };
    for job in jobs {
        let lease = job.lease().clone();
        let job_type = job.job_type().clone();
        let outcome = if job.payload().is_err() {
            JobOutcome::Fail(ErrorCode::new(ErrorCode::INVALID_PAYLOAD)?)
        } else {
            handler(job).await
        };
        let (label, error) = match &outcome {
            JobOutcome::Succeeded => match complete(pool, &lease, clock).await? {
                LeaseOutcome::Applied => ("succeeded", None),
                LeaseOutcome::LeaseLost => ("lease_lost", None),
            },
            JobOutcome::Completed(LeaseOutcome::Applied) => ("succeeded", None),
            JobOutcome::Completed(LeaseOutcome::LeaseLost) => ("lease_lost", None),
            JobOutcome::Retry(code) => {
                match retry_or_fail(pool, &lease, code, &cfg.backoff, clock).await? {
                    RetryOutcome::Requeued { .. } => ("requeued", Some(code)),
                    RetryOutcome::Failed => ("failed", Some(code)),
                    RetryOutcome::LeaseLost => ("lease_lost", Some(code)),
                }
            }
            JobOutcome::Fail(code) => match fail(pool, &lease, code, clock).await? {
                LeaseOutcome::Applied => ("failed", Some(code)),
                LeaseOutcome::LeaseLost => ("lease_lost", Some(code)),
            },
        };
        let error_code = error.map_or("", ErrorCode::as_str);
        match label {
            "succeeded" => {
                report.succeeded += 1;
                tracing::debug!(job_id = %lease.job_id(), job_type = %job_type, attempt = lease.attempt(), "job_succeeded");
            }
            "requeued" => {
                report.requeued += 1;
                tracing::warn!(job_id = %lease.job_id(), job_type = %job_type, attempt = lease.attempt(), error_code, "job_requeued");
            }
            "failed" => {
                report.failed += 1;
                tracing::warn!(job_id = %lease.job_id(), job_type = %job_type, attempt = lease.attempt(), error_code, "job_failed");
            }
            _ => {
                report.lease_lost += 1;
                tracing::warn!(job_id = %lease.job_id(), job_type = %job_type, attempt = lease.attempt(), "job_lease_lost");
            }
        }
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn backoff() -> Backoff {
        Backoff::new(Duration::seconds(1), Duration::seconds(10)).unwrap()
    }

    #[test]
    fn worker_config_validates_limits() {
        let w = WorkerId::new("worker").unwrap();
        let t = vec![JobType::new("uji").unwrap()];
        assert!(
            WorkerConfig::new(w.clone(), t.clone(), 1, Duration::seconds(30), backoff()).is_ok()
        );
        assert!(
            WorkerConfig::new(
                w.clone(),
                t.clone(),
                MAX_BATCH_SIZE,
                Duration::seconds(30),
                backoff()
            )
            .is_ok()
        );
        for (types, batch, lease) in [
            (vec![], 1, Duration::seconds(30)),
            (t.clone(), 0, Duration::seconds(30)),
            (t.clone(), MAX_BATCH_SIZE + 1, Duration::seconds(30)),
            (t.clone(), 1, Duration::ZERO),
            (t.clone(), 1, Duration::seconds(-1)),
        ] {
            assert_eq!(
                WorkerConfig::new(w.clone(), types, batch, lease, backoff())
                    .err()
                    .map(|e| e.kind()),
                Some(JobErrorKind::InvalidConfig)
            );
        }
    }
}
