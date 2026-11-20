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
//! - Event audit dibentuk **sekali** per penolakan ([`DenialAuditor::prepare`]: ID event dan waktunya
//!   tetap), lalu ditulis [`DenialAuditor::write`] milik aplikasi (crate ini tidak bergantung pada
//!   database) di transaksi tersendiri, dicoba ulang dengan jeda membesar memakai event yang sama, sehingga
//!   percobaan ulang tidak menggandakan event. Percobaan ulang tidak mengubah respons yang sudah terkirim
//!   (coding-standards §4.4 butir 3). Gagal permanen dicatat di application log.
//! - Security log dipancarkan pekerja tanpa menunggu kuota penulisan audit; penulisan audit yang
//!   menunggu kuota dibatasi jumlahnya (kapasitas antrean), kelebihannya dibuang dan dihitung.
//! - Antrean penuh (misalnya banjir percobaan ganti ID): catatan dibuang dan dihitung, respons tetap
//!   `404` tanpa menunggu. Pembuangan pertama dalam satu jendela [`DROP_REPORT_WINDOW`] langsung
//!   dilaporkan di application log (`access_denial_queue_overflow`, jumlah sejak laporan sebelumnya);
//!   sisanya dilaporkan pada pembuangan pertama jendela berikutnya atau oleh pekerja.

use crate::role::{AccountType, AdminSubRole};
use foundation::{CorrelationId, Uuid};
use observability::security::{self, AccessDenial};
use std::any::Any;
use std::fmt;
use std::future::Future;
use std::net::IpAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use tokio::runtime::Handle;
use tokio::sync::{Semaphore, mpsc};

/// Bawaan kapasitas antrean penolakan. Usulan teknis F1-T38.
pub const DEFAULT_DENIAL_QUEUE_CAPACITY: usize = 1024;
/// Bawaan jumlah penulisan audit penolakan yang berjalan bersamaan. Usulan teknis F1-T38.
pub const DEFAULT_DENIAL_AUDIT_CONCURRENCY: usize = 4;

/// Jendela laporan pembuangan antrean: pembuangan pertama per jendela langsung dilaporkan.
pub const DROP_REPORT_WINDOW: Duration = Duration::from_secs(10);

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

/// Future hasil [`DenialAuditor::write`].
pub type DenialAuditFuture<'a> =
    Pin<Box<dyn Future<Output = Result<(), DenialAuditError>> + Send + 'a>>;

/// Event audit yang sudah dibentuk untuk satu penolakan (isi milik implementasi [`DenialAuditor`]).
pub struct PreparedDenialAudit(Box<dyn Any + Send + Sync>);

impl PreparedDenialAudit {
    /// Membungkus event yang sudah dibentuk.
    pub fn new<T: Any + Send + Sync>(event: T) -> Self {
        Self(Box::new(event))
    }

    /// Isi event bila jenisnya `T`.
    pub fn get<T: Any>(&self) -> Option<&T> {
        self.0.downcast_ref::<T>()
    }
}

impl fmt::Debug for PreparedDenialAudit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PreparedDenialAudit")
            .finish_non_exhaustive()
    }
}

/// Penulis event audit E-05 hasil gagal untuk penolakan atas data sensitif (implementasi aplikasi).
/// Dipanggil hanya untuk penolakan dengan [`DeniedAccess::sensitive`].
pub trait DenialAuditor: Send + Sync {
    /// Membentuk event satu kali per penolakan (ID event dan waktu tetap untuk semua percobaan).
    fn prepare(&self, denial: &DeniedAccess) -> Result<PreparedDenialAudit, DenialAuditError>;
    /// Menulis event yang sudah dibentuk; wajib idempoten (percobaan ulang dengan event yang sama tidak
    /// menggandakannya).
    fn write<'a>(&'a self, prepared: &'a PreparedDenialAudit) -> DenialAuditFuture<'a>;
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
    drops: Arc<DropCounter>,
}

/// Penghitung pembuangan dengan laporan pertama per jendela waktu.
struct DropCounter {
    count: AtomicU64,
    started: Instant,
    /// Milidetik sejak `started` saat laporan terakhir; `u64::MAX` = belum pernah.
    last_report_ms: AtomicU64,
}

impl DropCounter {
    fn new() -> Self {
        Self {
            count: AtomicU64::new(0),
            started: Instant::now(),
            last_report_ms: AtomicU64::new(u64::MAX),
        }
    }

    /// Mencatat satu pembuangan; mengembalikan jumlah yang harus dilaporkan sekarang (pembuangan pertama
    /// dalam jendela baru), atau `None`.
    fn record_drop(&self) -> Option<u64> {
        self.count.fetch_add(1, Ordering::Relaxed);
        let now = u64::try_from(self.started.elapsed().as_millis()).unwrap_or(u64::MAX - 1);
        let window = u64::try_from(DROP_REPORT_WINDOW.as_millis()).unwrap_or(u64::MAX);
        let last = self.last_report_ms.load(Ordering::Relaxed);
        let due = last == u64::MAX || now.saturating_sub(last) >= window;
        if due
            && self
                .last_report_ms
                .compare_exchange(last, now, Ordering::AcqRel, Ordering::Relaxed)
                .is_ok()
        {
            return Some(self.count.swap(0, Ordering::Relaxed));
        }
        None
    }

    fn take(&self) -> u64 {
        self.count.swap(0, Ordering::Relaxed)
    }
}

fn report_overflow(lost: u64) {
    if lost > 0 {
        tracing::error!(count = lost, "access_denial_queue_overflow");
    }
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
        let drops = Arc::new(DropCounter::new());
        let limits = AuditLimits {
            permits: Arc::new(Semaphore::new(audit_concurrency)),
            pending: Arc::new(AtomicUsize::new(0)),
            max_pending: capacity,
        };
        let worker_drops = Arc::clone(&drops);
        std::thread::Builder::new()
            .name("authz-denials".to_string())
            .spawn(move || worker(rx, auditor, limits, worker_drops))
            .map_err(|_| DenialSinkError::Thread)?;
        Ok(Self { tx, drops })
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
        if self.tx.try_send(queued).is_err()
            && let Some(lost) = self.drops.record_drop()
        {
            report_overflow(lost);
        }
    }
}

/// Batas penulisan audit: konkurensi dan jumlah penulisan yang menunggu kuota.
struct AuditLimits {
    permits: Arc<Semaphore>,
    pending: Arc<AtomicUsize>,
    max_pending: usize,
}

fn worker(
    mut rx: mpsc::Receiver<Queued>,
    auditor: Option<Arc<dyn DenialAuditor>>,
    limits: AuditLimits,
    drops: Arc<DropCounter>,
) {
    while let Some(Queued {
        denial,
        dispatch,
        runtime,
    }) = rx.blocking_recv()
    {
        let lost = drops.take();
        tracing::dispatcher::with_default(&dispatch, || {
            report_overflow(lost);
            security::access_denied(
                Some(denial.account_id),
                &denial.route,
                AccessDenial::ForeignResourceId,
            );
        });
        let (Some(auditor), Some(_), Some(handle)) = (&auditor, denial.sensitive, runtime) else {
            continue;
        };
        // Event dibentuk sekali; percobaan ulang memakai event yang sama.
        let prepared = match auditor.prepare(&denial) {
            Ok(p) => p,
            Err(_) => {
                audit_failed(&dispatch, &denial.route, "permanent");
                continue;
            }
        };
        // Pekerja tidak menunggu kuota audit (security log berikutnya tidak tertahan); penulisan yang
        // menunggu kuota dibatasi jumlahnya.
        if limits.pending.fetch_add(1, Ordering::AcqRel) >= limits.max_pending {
            limits.pending.fetch_sub(1, Ordering::AcqRel);
            if let Some(lost) = drops.record_drop() {
                tracing::dispatcher::with_default(&dispatch, || report_overflow(lost));
            }
            continue;
        }
        let auditor = Arc::clone(auditor);
        let permits = Arc::clone(&limits.permits);
        let pending = Arc::clone(&limits.pending);
        let route = denial.route;
        handle.spawn(async move {
            if let Ok(_permit) = permits.acquire_owned().await {
                write_with_retry(auditor.as_ref(), &prepared, &route, &dispatch).await;
            }
            pending.fetch_sub(1, Ordering::AcqRel);
        });
    }
}

fn audit_failed(dispatch: &tracing::Dispatch, route: &str, reason: &str) {
    tracing::dispatcher::with_default(dispatch, || {
        tracing::error!(route = route, reason = reason, "access_denial_audit_failed");
    });
}

async fn write_with_retry(
    auditor: &dyn DenialAuditor,
    prepared: &PreparedDenialAudit,
    route: &str,
    dispatch: &tracing::Dispatch,
) {
    let mut attempt = 0usize;
    loop {
        let reason = match auditor.write(prepared).await {
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
        audit_failed(dispatch, route, reason);
        return;
    }
}
