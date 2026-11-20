//! Pencatatan penolakan akses atas objek di luar jalur permintaan (F1-T38, KU-03, ADR-012 butir 2 dan 4,
//! coding-standards §4.4 butir 3; TD-15 Low).
//!
//! Objek milik orang lain dijawab `404` yang identik dengan objek tidak ada. Pencatatannya (security log
//! `access_denied` `foreign_resource_id`, dan untuk data sensitif event audit E-05 hasil gagal) **tidak**
//! dikerjakan di jalur permintaan: [`DenialSink`] hanya memasukkan catatan ke antrean terbatas
//! (`try_send`, tanpa menunggu), lalu utas pekerja khusus menulisnya. Dengan begitu waktu tanggap "milik
//! orang lain" tidak lebih lambat dari "tidak ada" karena penulisan log atau audit (KU-03: tanpa selisih
//! waktu tanggap yang mencolok; review F1-T32/T37 TD-15). Test timing di build release:
//! `tests/denial_timing_release.rs` dengan probe `examples/denial_timing.rs`.
//!
//! - Security log dipancarkan utas pekerja dengan subscriber `tracing` milik permintaan (ditangkap saat
//!   antre), sehingga tujuan lognya sama dengan log permintaan itu. Sink log yang lambat hanya menahan
//!   utas pekerja, bukan runtime yang melayani permintaan.
//! - Event audit ditulis [`DenialAuditor`] milik aplikasi (crate ini tidak bergantung pada database) di
//!   transaksi tersendiri, dicoba ulang dengan jeda membesar; percobaan ulang tidak mengubah respons yang
//!   sudah terkirim (coding-standards §4.4 butir 3). Gagal permanen dicatat di application log.
//! - Antrean penuh (misalnya banjir percobaan ganti ID): catatan dibuang dan dihitung, respons tetap
//!   `404` tanpa menunggu; jumlah yang dibuang dilaporkan di application log oleh pekerja.

use crate::role::{AccountType, AdminSubRole};
use foundation::{CorrelationId, Uuid};
use observability::security::{self, AccessDenial};
use std::fmt;
use std::future::Future;
use std::net::IpAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio::runtime::Handle;
use tokio::sync::{Semaphore, mpsc};

/// Bawaan kapasitas antrean penolakan. Usulan teknis F1-T38.
pub const DEFAULT_DENIAL_QUEUE_CAPACITY: usize = 1024;
/// Bawaan jumlah penulisan audit penolakan yang berjalan bersamaan. Usulan teknis F1-T38.
pub const DEFAULT_DENIAL_AUDIT_CONCURRENCY: usize = 4;

/// Jeda sebelum percobaan ulang penulisan audit (jumlah percobaan = panjang + 1).
const AUDIT_RETRY_DELAYS: [Duration; 3] = [
    Duration::from_millis(100),
    Duration::from_millis(1_000),
    Duration::from_millis(5_000),
];

/// Alamat IP klien permintaan, dipasang di extensions oleh middleware IP klien aplikasi (sumber
/// alamatnya bergantung pada penerusan IP asli lewat proxy, docs/05-stack.md §2.6, PT-39). Dibaca
/// middleware penegak ke [`crate::Principal`] untuk field "Sumber" audit (PRD §7.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClientIp(pub IpAddr);

/// Objek data sensitif yang aksesnya ditolak (PRD §3.1 "Data sensitif", E-05): jenis objek, ID objek,
/// dan penerima layanan yang menjadi subjek data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SensitiveTarget {
    /// Jenis objek (kode DD, misalnya `care_recipient`).
    pub object_type: &'static str,
    /// ID objek.
    pub object_id: Uuid,
    /// `care_recipient.id` subjek data.
    pub subject_care_recipient_id: Uuid,
}

/// Satu penolakan akses atas objek yang ada tetapi tidak boleh diakses pemanggil.
#[derive(Clone, PartialEq, Eq)]
pub struct DeniedAccess {
    /// Akun pemanggil.
    pub account_id: Uuid,
    /// Jenis akun pemanggil.
    pub account_type: AccountType,
    /// Sub-peran admin pemanggil (kosong untuk jenis lain).
    pub sub_roles: Vec<AdminSubRole>,
    /// Pola rute (bukan URL berisi ID).
    pub route: String,
    /// ID korelasi permintaan.
    pub correlation_id: CorrelationId,
    /// Alamat IP klien bila middleware IP klien terpasang.
    pub client_ip: Option<IpAddr>,
    /// Diisi bila objeknya data sensitif: wajib event audit E-05 hasil gagal (KU-03).
    pub sensitive: Option<SensitiveTarget>,
}

// Debug tanpa alamat IP (data pribadi, ADR-010 butir 1).
impl fmt::Debug for DeniedAccess {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DeniedAccess")
            .field("account_id", &self.account_id)
            .field("account_type", &self.account_type)
            .field("route", &self.route)
            .field("sensitive", &self.sensitive)
            .finish_non_exhaustive()
    }
}

/// Galat penulisan audit penolakan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DenialAuditError {
    /// Gagal sementara (database): dicoba ulang.
    Retryable,
    /// Tidak dapat ditulis (data tidak lengkap atau ditolak pustaka audit): tidak dicoba ulang.
    Permanent,
}

/// Future hasil [`DenialAuditor::record`].
pub type DenialAuditFuture<'a> =
    Pin<Box<dyn Future<Output = Result<(), DenialAuditError>> + Send + 'a>>;

/// Penulis event audit E-05 hasil gagal untuk penolakan atas data sensitif (implementasi aplikasi,
/// lewat `audit::in_audited_tx` di transaksi tersendiri).
pub trait DenialAuditor: Send + Sync {
    /// Menulis satu event. Dipanggil hanya untuk penolakan dengan [`DeniedAccess::sensitive`].
    fn record<'a>(&'a self, denial: &'a DeniedAccess) -> DenialAuditFuture<'a>;
}

/// Galat pembuatan [`DenialSink`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DenialSinkError {
    /// Kapasitas atau konkurensi nol.
    InvalidCapacity,
    /// Utas pekerja tidak dapat dibuat.
    Thread,
}

impl fmt::Display for DenialSinkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidCapacity => "kapasitas antrean penolakan tidak valid",
            Self::Thread => "utas pekerja antrean penolakan tidak dapat dibuat",
        })
    }
}

impl std::error::Error for DenialSinkError {}

struct Queued {
    denial: DeniedAccess,
    dispatch: tracing::Dispatch,
    runtime: Option<Handle>,
}

/// Antrean penolakan akses dengan utas pekerja. Murah di-clone; pekerja berhenti saat semua salinan
/// dibuang.
#[derive(Clone)]
pub struct DenialSink {
    tx: mpsc::Sender<Queued>,
    dropped: Arc<AtomicU64>,
}

impl fmt::Debug for DenialSink {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DenialSink").finish_non_exhaustive()
    }
}

impl DenialSink {
    /// Memulai utas pekerja. Penulisan audit berjalan di runtime tokio permintaan yang menolak.
    /// `auditor` = penulis E-05 (wajib untuk backend yang melayani data sensitif; `None` hanya untuk
    /// backend tanpa data sensitif dan test).
    pub fn start(
        capacity: usize,
        audit_concurrency: usize,
        auditor: Option<Arc<dyn DenialAuditor>>,
    ) -> Result<Self, DenialSinkError> {
        if capacity == 0 || audit_concurrency == 0 {
            return Err(DenialSinkError::InvalidCapacity);
        }
        let (tx, rx) = mpsc::channel(capacity);
        let dropped = Arc::new(AtomicU64::new(0));
        let worker_dropped = Arc::clone(&dropped);
        let permits = Arc::new(Semaphore::new(audit_concurrency));
        std::thread::Builder::new()
            .name("authz-denials".to_string())
            .spawn(move || worker(rx, auditor, permits, worker_dropped))
            .map_err(|_| DenialSinkError::Thread)?;
        Ok(Self { tx, dropped })
    }

    /// Memasukkan penolakan ke antrean tanpa menunggu. Antrean penuh = dibuang dan dihitung.
    pub(crate) fn submit(&self, denial: DeniedAccess) {
        let dispatch = tracing::dispatcher::get_default(Clone::clone);
        let runtime = Handle::try_current().ok();
        let queued = Queued {
            denial,
            dispatch,
            runtime,
        };
        if self.tx.try_send(queued).is_err() {
            self.dropped.fetch_add(1, Ordering::Relaxed);
        }
    }
}

fn worker(
    mut rx: mpsc::Receiver<Queued>,
    auditor: Option<Arc<dyn DenialAuditor>>,
    permits: Arc<Semaphore>,
    dropped: Arc<AtomicU64>,
) {
    while let Some(Queued {
        denial,
        dispatch,
        runtime,
    }) = rx.blocking_recv()
    {
        let lost = dropped.swap(0, Ordering::Relaxed);
        tracing::dispatcher::with_default(&dispatch, || {
            if lost > 0 {
                tracing::error!(count = lost, "access_denial_queue_overflow");
            }
            security::access_denied(
                Some(denial.account_id),
                &denial.route,
                AccessDenial::ForeignResourceId,
            );
        });
        let (Some(auditor), Some(_), Some(handle)) = (&auditor, denial.sensitive, runtime) else {
            continue;
        };
        // Batas konkurensi audit ditunggu di utas pekerja, bukan di runtime.
        let Ok(permit) = handle.block_on(Arc::clone(&permits).acquire_owned()) else {
            continue;
        };
        let auditor = Arc::clone(auditor);
        handle.spawn(async move {
            record_with_retry(auditor.as_ref(), &denial, &dispatch).await;
            drop(permit);
        });
    }
}

async fn record_with_retry(
    auditor: &dyn DenialAuditor,
    denial: &DeniedAccess,
    dispatch: &tracing::Dispatch,
) {
    let mut attempt = 0usize;
    loop {
        let reason = match auditor.record(denial).await {
            Ok(()) => return,
            Err(DenialAuditError::Permanent) => "permanent",
            Err(DenialAuditError::Retryable) => match AUDIT_RETRY_DELAYS.get(attempt) {
                Some(delay) => {
                    attempt += 1;
                    tokio::time::sleep(*delay).await;
                    continue;
                }
                None => "retries_exhausted",
            },
        };
        tracing::dispatcher::with_default(dispatch, || {
            tracing::error!(
                route = denial.route.as_str(),
                reason = reason,
                "access_denial_audit_failed"
            );
        });
        return;
    }
}
