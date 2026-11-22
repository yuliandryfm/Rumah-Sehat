//! Antarmuka publik modul `platform` untuk modul lain:
//! - pemeriksaan kesiapan yang dapat didaftarkan (F1-T11);
//! - berkas unggahan (F1-T29, ADR-018): slot unggah dengan URL bertanda tangan, validasi isi pasca-unggah,
//!   dan pencatatan `stored_file` di transaksi milik pemanggil;
//! - URL unduh berumur pendek setelah otorisasi dan E-05 (F1-T30, ADR-018 butir 3): [`issue_download_url`].
//!
//! Alur berkas (tiga langkah ADR-018 konsekuensi): (1) [`create_upload_slot`] → klien mengunggah langsung ke
//! object storage dengan [`UploadInstruction`]; (2) [`validate_upload`] di luar transaksi database
//! (memanggil object storage, KEPUTUSAN-125): ukuran, jenis dari isi, kode ulang foto, objek asli beserta
//! semua versinya dihapus; (3) [`record_stored_file`] di transaksi use case pemanggil (misalnya
//! `audit::in_audited_tx` bersama event E-10 pengajuan identitas, US-106), sekaligus memakai slot sekali.
//! Bila transaksi pemanggil gagal, pemanggil memanggil [`discard_validated`]. Berkas baru dapat dirujuk
//! setelah langkah 3 (ADR-018 butir 2).
//!
//! Pembersihan (TD-17): slot yang dipakai atau ditolak tetap ada sebagai tombstone (`consumed_at`) sampai
//! `expires_at` + [`UPLOAD_SLOT_TOMBSTONE_GRACE`], karena URL unggah masih dapat dipakai ulang sampai
//! kedaluwarsa. Job pembersih ([`purge_expired_upload_slots`], jenis job [`UPLOAD_SLOT_PURGE_JOB`]) lalu
//! menghapus semua versi objek mentah (prefiks `incoming/`) dan hasil kode ulang yang tidak pernah dicatat,
//! baru barisnya. Rekonsiliasi berkala ([`reconcile_bucket`], [`FILE_RECONCILE_JOB`]) menghapus objek yatim
//! di `incoming/` dan `files/` dan melaporkan baris `stored_file` yang objeknya hilang. Keduanya pengaman
//! untuk proses yang mati di tengah alur dan untuk [`ValidatedUpload`] yang dibuang tanpa
//! [`discard_validated`].

use super::service;
use audit::{AuditContext, Subject};
use authz::{AdminSubRole, Principal};
use foundation::{Clock, Uuid};
use object_storage::{
    AddressingStyle, Deployment, ObjectStorageConfig, ObjectStore, valid_bucket_name,
};
use observability::{ConfigError, EnvSource};
use platform_jobs::{ClaimedJob, JobOutcome};
use sqlx::{PgConnection, PgPool};
use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use time::OffsetDateTime;
use tokio::sync::Semaphore;

pub use super::domain::{
    LogicalBucket, MAX_PHOTO_BYTES, PEAK_VALIDATION_MEMORY_BYTES, PhotoFormat, UploadPurpose,
};
pub use super::error::FileError;
pub use crypto::ContainerType;

/// Hasil pemeriksaan: `true` bila dependensi siap.
pub type ReadinessFuture<'a> = Pin<Box<dyn Future<Output = bool> + Send + 'a>>;

/// Satu pemeriksaan kesiapan. Tidak boleh memuat data pribadi di nama maupun log (ADR-010).
pub trait ReadinessCheck: Send + Sync {
    /// Nama tetap untuk log kegagalan (bukan untuk respons).
    fn name(&self) -> &'static str;
    /// Menjalankan pemeriksaan.
    fn check(&self) -> ReadinessFuture<'_>;
}

/// Kesiapan database aplikasi: satu koneksi dari pool dapat diambil (sqlx memeriksa koneksi sebelum
/// diserahkan). Tanpa query ke tabel mana pun.
#[derive(Debug, Clone)]
pub struct DatabaseReadiness {
    pool: PgPool,
}

impl DatabaseReadiness {
    /// Pemeriksaan atas pool proses `api`.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl ReadinessCheck for DatabaseReadiness {
    fn name(&self) -> &'static str {
        "database_app"
    }

    fn check(&self) -> ReadinessFuture<'_> {
        Box::pin(async move { self.pool.acquire().await.is_ok() })
    }
}

/// Bawaan umur slot dan URL unggah (detik). Usulan teknis F1-T29: ADR-018 hanya menyebut "berumur pendek";
/// 10 menit memberi ruang unggah 5 MiB lewat jaringan seluler lambat.
pub const DEFAULT_UPLOAD_SLOT_TTL_SECONDS: u64 = 600;
/// Rentang umur slot yang diterima konfigurasi (detik).
pub const UPLOAD_SLOT_TTL_RANGE: std::ops::RangeInclusive<u64> = 60..=3600;
/// Bawaan batas waktu satu operasi object storage (detik).
pub const DEFAULT_OBJECT_STORAGE_TIMEOUT_SECONDS: u64 = 10;
/// Bawaan jumlah validasi foto yang berjalan bersamaan (dekode memakan memori, lihat `domain`).
pub const DEFAULT_FILE_VALIDATION_CONCURRENCY: usize = 2;
/// Batas atas konkurensi validasi (TD-17, usulan teknis): 4 × [`PEAK_VALIDATION_MEMORY_BYTES`] = 2 GiB per
/// proses `api`. Diturunkan dari 16 (review F1-T29: ±0,9 GB per validasi).
pub const MAX_FILE_VALIDATION_CONCURRENCY: usize = 4;
/// Bawaan batas tunggu antrean validasi (detik); lewat batas → 503 (TD-17, usulan teknis).
pub const DEFAULT_FILE_VALIDATION_QUEUE_TIMEOUT_SECONDS: u64 = 10;
/// Rentang batas tunggu antrean validasi yang diterima konfigurasi (detik).
pub const FILE_VALIDATION_QUEUE_TIMEOUT_RANGE: std::ops::RangeInclusive<u64> = 1..=60;
/// Jeda setelah `expires_at` sebelum slot dan objek mentahnya dibersihkan (TD-17, usulan teknis): menutup
/// selisih jam aplikasi dan penyedia serta validasi yang masih berjalan saat slot kedaluwarsa.
pub const UPLOAD_SLOT_TOMBSTONE_GRACE: Duration = Duration::from_secs(15 * 60);
/// Jenis job pembersih slot kedaluwarsa (platform-jobs).
pub const UPLOAD_SLOT_PURGE_JOB: &str = "platform_upload_slot_purge";
/// Jenis job rekonsiliasi bucket terhadap `stored_file` (platform-jobs); payload `bucket` = nilai kolom
/// `bucket`.
pub const FILE_RECONCILE_JOB: &str = "platform_file_reconcile";
/// Jumlah slot per putaran pembersih.
pub const UPLOAD_SLOT_PURGE_BATCH: u16 = 100;

/// Jalur unggah (KEPUTUSAN-234): POST dengan policy diutamakan; PUT bertanda tangan sebagai alternatif bila
/// penyedia tidak mendukung POST (dukungan NEO `[TIDAK DIKETAHUI]`, spike F1-T03 §5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UploadMethod {
    /// Presigned POST dengan policy.
    Post,
    /// Presigned PUT dengan `Content-Type`/`Content-Length` bertanda tangan.
    Put,
}

impl std::str::FromStr for UploadMethod {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "post" => Ok(Self::Post),
            "put" => Ok(Self::Put),
            _ => Err(()),
        }
    }
}

/// Konfigurasi berkas (KEPUTUSAN-132, KEPUTUSAN-235). Kunci rahasia bertipe `Sensitive`.
#[derive(Debug)]
pub struct FileStorageConfig {
    /// Koneksi object storage.
    pub object_storage: ObjectStorageConfig,
    /// Awalan nama bucket fisik; nama fisik = awalan + nama bucket logis dengan `_` → `-`.
    pub bucket_prefix: String,
    /// Umur slot dan URL unggah.
    pub slot_ttl: Duration,
    /// Jalur unggah.
    pub upload_method: UploadMethod,
    /// Jumlah validasi foto bersamaan (1..=[`MAX_FILE_VALIDATION_CONCURRENCY`]).
    pub validation_concurrency: usize,
    /// Batas tunggu antrean validasi sebelum menjawab 503.
    pub validation_queue_timeout: Duration,
}

impl FileStorageConfig {
    /// Memuat dari variabel lingkungan; gagal cepat tanpa mencetak nilai.
    ///
    /// `DEPLOY_ENV` (`local`|`staging`|`production`), `OBJECT_STORAGE_ENDPOINT`, `OBJECT_STORAGE_REGION`,
    /// `OBJECT_STORAGE_ADDRESSING` (`path`|`virtual_host`), `OBJECT_STORAGE_ACCESS_KEY_ID`,
    /// `OBJECT_STORAGE_SECRET_ACCESS_KEY` (rahasia), `OBJECT_STORAGE_BUCKET_PREFIX`; opsional
    /// `OBJECT_STORAGE_TIMEOUT_SECONDS`, `UPLOAD_SLOT_TTL_SECONDS`, `UPLOAD_METHOD` (`post`|`put`),
    /// `FILE_VALIDATION_CONCURRENCY`, `FILE_VALIDATION_QUEUE_TIMEOUT_SECONDS`.
    pub fn load(env: &EnvSource) -> Result<Self, ConfigError> {
        let deployment: Deployment = env.required("DEPLOY_ENV")?;
        let timeout: u64 = env.optional_or(
            "OBJECT_STORAGE_TIMEOUT_SECONDS",
            DEFAULT_OBJECT_STORAGE_TIMEOUT_SECONDS,
        )?;
        let ttl: u64 =
            env.optional_or("UPLOAD_SLOT_TTL_SECONDS", DEFAULT_UPLOAD_SLOT_TTL_SECONDS)?;
        let concurrency: usize = env.optional_or(
            "FILE_VALIDATION_CONCURRENCY",
            DEFAULT_FILE_VALIDATION_CONCURRENCY,
        )?;
        let queue_timeout: u64 = env.optional_or(
            "FILE_VALIDATION_QUEUE_TIMEOUT_SECONDS",
            DEFAULT_FILE_VALIDATION_QUEUE_TIMEOUT_SECONDS,
        )?;
        let config = Self {
            object_storage: ObjectStorageConfig {
                endpoint: env.required("OBJECT_STORAGE_ENDPOINT")?,
                region: env.required("OBJECT_STORAGE_REGION")?,
                addressing: env.required::<AddressingStyle>("OBJECT_STORAGE_ADDRESSING")?,
                access_key_id: env.required("OBJECT_STORAGE_ACCESS_KEY_ID")?,
                secret_access_key: env.required_secret("OBJECT_STORAGE_SECRET_ACCESS_KEY")?,
                deployment,
                timeout: Duration::from_secs(timeout),
            },
            bucket_prefix: env.required("OBJECT_STORAGE_BUCKET_PREFIX")?,
            slot_ttl: Duration::from_secs(ttl),
            upload_method: env.optional_or("UPLOAD_METHOD", UploadMethod::Post)?,
            validation_concurrency: concurrency,
            validation_queue_timeout: Duration::from_secs(queue_timeout),
        };
        Ok(config)
    }
}

/// Galat penyiapan [`FileStorage`] (nama field, bukan nilai).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileStorageSetupError(pub &'static str);

impl fmt::Display for FileStorageSetupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "konfigurasi berkas tidak valid: {}", self.0)
    }
}

impl std::error::Error for FileStorageSetupError {}

/// Layanan berkas yang sudah tervalidasi konfigurasinya. Murah dibagi lewat `Arc`.
pub struct FileStorage {
    pub(super) store: ObjectStore,
    bucket_prefix: String,
    pub(super) slot_ttl: Duration,
    pub(super) upload_method: UploadMethod,
    pub(super) permits: Arc<Semaphore>,
    pub(super) queue_timeout: Duration,
}

impl fmt::Debug for FileStorage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FileStorage")
            .field("store", &self.store)
            .field("slot_ttl", &self.slot_ttl)
            .field("upload_method", &self.upload_method)
            .finish()
    }
}

impl FileStorage {
    /// Memvalidasi konfigurasi: endpoint (`http://` hanya lokal; `https://` belum didukung klien server),
    /// nama bucket fisik ketujuh bucket logis, umur slot, konkurensi, dan batas tunggu antrean.
    pub fn new(config: &FileStorageConfig) -> Result<Self, FileStorageSetupError> {
        let store = ObjectStore::new(&config.object_storage).map_err(|e| {
            FileStorageSetupError(match e {
                object_storage::ObjectStorageError::InsecureEndpoint => {
                    "OBJECT_STORAGE_ENDPOINT (http:// hanya untuk DEPLOY_ENV=local)"
                }
                object_storage::ObjectStorageError::TlsUnsupported => {
                    "OBJECT_STORAGE_ENDPOINT (https:// belum didukung)"
                }
                _ => "OBJECT_STORAGE_*",
            })
        })?;
        for b in LogicalBucket::ALL {
            if !valid_bucket_name(&physical_bucket(&config.bucket_prefix, b)) {
                return Err(FileStorageSetupError("OBJECT_STORAGE_BUCKET_PREFIX"));
            }
        }
        if !UPLOAD_SLOT_TTL_RANGE.contains(&config.slot_ttl.as_secs())
            || config.slot_ttl.subsec_nanos() != 0
        {
            return Err(FileStorageSetupError("UPLOAD_SLOT_TTL_SECONDS"));
        }
        if !(1..=MAX_FILE_VALIDATION_CONCURRENCY).contains(&config.validation_concurrency) {
            return Err(FileStorageSetupError("FILE_VALIDATION_CONCURRENCY"));
        }
        if !FILE_VALIDATION_QUEUE_TIMEOUT_RANGE.contains(&config.validation_queue_timeout.as_secs())
            || config.validation_queue_timeout.subsec_nanos() != 0
        {
            return Err(FileStorageSetupError(
                "FILE_VALIDATION_QUEUE_TIMEOUT_SECONDS",
            ));
        }
        Ok(Self {
            store,
            bucket_prefix: config.bucket_prefix.clone(),
            slot_ttl: config.slot_ttl,
            upload_method: config.upload_method,
            permits: Arc::new(Semaphore::new(config.validation_concurrency)),
            queue_timeout: config.validation_queue_timeout,
        })
    }

    /// Nama bucket fisik untuk bucket logis.
    pub fn bucket_name(&self, bucket: LogicalBucket) -> String {
        physical_bucket(&self.bucket_prefix, bucket)
    }

    /// Klien object storage (untuk penyiapan bucket lokal/test dan modul unduh F1-T30).
    pub fn object_store(&self) -> &ObjectStore {
        &self.store
    }
}

fn physical_bucket(prefix: &str, bucket: LogicalBucket) -> String {
    format!("{prefix}{}", bucket.as_str().replace('_', "-"))
}

/// Permintaan slot unggah. `owner_account_id` berasal dari konteks autentikasi (bukan dari isi permintaan).
#[derive(Debug, Clone)]
pub struct UploadSlotRequest {
    /// Akun pemilik slot.
    pub owner_account_id: Uuid,
    /// Tujuan unggahan.
    pub purpose: UploadPurpose,
    /// Tipe MIME yang dinyatakan klien (`image/jpeg` atau `image/png`); isi tetap diperiksa setelah unggah.
    pub content_type: String,
    /// Ukuran berkas yang dinyatakan klien (byte); wajib persis untuk jalur PUT.
    pub size_bytes: u64,
}

/// Cara klien mengunggah. URL dan field adalah rahasia berumur pendek: tidak boleh dicatat di log.
#[derive(Clone, PartialEq, Eq)]
pub enum UploadInstruction {
    /// `multipart/form-data` ke `url` dengan semua `fields`, berkas sebagai field `file` terakhir.
    Post {
        /// URL formulir.
        url: String,
        /// Field formulir.
        fields: Vec<(String, String)>,
    },
    /// `PUT` ke `url` dengan `headers` persis.
    Put {
        /// URL bertanda tangan.
        url: String,
        /// Header wajib (`Content-Type`, `Content-Length`).
        headers: Vec<(String, String)>,
    },
}

impl fmt::Debug for UploadInstruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Post { .. } => f.write_str("UploadInstruction::Post([REDACTED])"),
            Self::Put { .. } => f.write_str("UploadInstruction::Put([REDACTED])"),
        }
    }
}

/// Slot unggah yang diterbitkan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UploadTicket {
    /// ID slot, dikirim klien kembali saat konfirmasi.
    pub upload_slot_id: Uuid,
    /// Instan kedaluwarsa slot dan URL.
    pub expires_at: OffsetDateTime,
    /// Cara mengunggah.
    pub instruction: UploadInstruction,
}

/// Wadah retensi berkas (`stored_file.container_type`/`container_id`, ADR-018 butir 4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileContainer {
    /// Jenis wadah (nilai DD §0.7).
    pub container_type: ContainerType,
    /// ID wadah.
    pub container_id: Uuid,
}

/// Berkas yang lolos validasi dan sudah dikode ulang di object storage, tetapi belum dicatat. Hanya dapat
/// dibuat oleh [`validate_upload`]; dipakai sekali oleh [`record_stored_file`] atau [`discard_validated`].
///
/// Penjaga (TD-17): nilai yang dibuang sebelum dicatat atau dibuang lewat [`discard_validated`] menulis
/// peringatan `validated_upload_dropped` di log aplikasi. Objeknya tidak bocor permanen: slot masih
/// mengklaimnya, sehingga job pembersih menghapusnya setelah slot lewat `expires_at` + jeda.
#[must_use = "catat dengan record_stored_file atau buang dengan discard_validated"]
pub struct ValidatedUpload {
    pub(super) upload_slot_id: Uuid,
    pub(super) owner_account_id: Uuid,
    pub(super) purpose: UploadPurpose,
    pub(super) object_key: String,
    pub(super) format: PhotoFormat,
    pub(super) size_bytes: i32,
    pub(super) sha256: [u8; crypto::SHA256_LEN],
    pub(super) validated_at: OffsetDateTime,
    /// Sudah diserahkan ke [`record_stored_file`] atau [`discard_validated`].
    pub(super) settled: AtomicBool,
}

impl ValidatedUpload {
    pub(super) fn mark_settled(&self) {
        self.settled.store(true, Ordering::Release);
    }
}

impl Drop for ValidatedUpload {
    fn drop(&mut self) {
        if !self.settled.load(Ordering::Acquire) {
            tracing::warn!(upload_slot_id = %self.upload_slot_id, "validated_upload_dropped");
        }
    }
}

impl fmt::Debug for ValidatedUpload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ValidatedUpload")
            .field("upload_slot_id", &self.upload_slot_id)
            .field("purpose", &self.purpose)
            .field("format", &self.format)
            .field("size_bytes", &self.size_bytes)
            .finish()
    }
}

impl ValidatedUpload {
    /// ID slot asal.
    pub fn upload_slot_id(&self) -> Uuid {
        self.upload_slot_id
    }
    /// Tipe MIME menurut isi (setelah kode ulang).
    pub fn mime_type(&self) -> &'static str {
        self.format.mime()
    }
    /// Ukuran berkas tersimpan (setelah kode ulang).
    pub fn size_bytes(&self) -> i32 {
        self.size_bytes
    }
    /// SHA-256 berkas tersimpan.
    pub fn sha256(&self) -> &[u8; crypto::SHA256_LEN] {
        &self.sha256
    }
    /// Kunci objek berkas tersimpan (nama UUID buatan aplikasi).
    pub fn object_key(&self) -> &str {
        &self.object_key
    }
}

/// Langkah 1: memeriksa jenis/ukuran yang dinyatakan, menulis `upload_slot`, dan menerbitkan URL unggah
/// bertanda tangan berumur pendek (ADR-018 butir 2, KEPUTUSAN-234).
pub async fn create_upload_slot(
    pool: &PgPool,
    clock: &dyn Clock,
    storage: &FileStorage,
    request: UploadSlotRequest,
) -> Result<UploadTicket, FileError> {
    service::create_upload_slot(pool, clock, storage, request).await
}

/// Langkah 2: validasi pasca-unggah di luar transaksi database. Slot milik akun lain, kedaluwarsa, sudah
/// dipakai, atau untuk tujuan lain menghasilkan [`FileError::NotFound`]. Berkas yang ditolak (413/415)
/// dihapus beserta semua versinya, lalu slotnya dihapus.
pub async fn validate_upload(
    pool: &PgPool,
    clock: &dyn Clock,
    storage: &FileStorage,
    owner_account_id: Uuid,
    upload_slot_id: Uuid,
    purpose: UploadPurpose,
) -> Result<ValidatedUpload, FileError> {
    service::validate_upload(
        pool,
        clock,
        storage,
        owner_account_id,
        upload_slot_id,
        purpose,
    )
    .await
}

/// Langkah 3: di transaksi milik pemanggil, memakai slot sekali (tombstone `consumed_at`) dan menulis
/// `stored_file`. Mengembalikan ID `stored_file`. Slot yang sudah dipakai atau sudah kedaluwarsa pada
/// `clock.now()` menghasilkan [`FileError::NotFound`]; pemanggil lalu memanggil [`discard_validated`].
pub async fn record_stored_file(
    conn: &mut PgConnection,
    clock: &dyn Clock,
    upload: &ValidatedUpload,
    container: Option<FileContainer>,
) -> Result<Uuid, FileError> {
    service::record_stored_file(conn, clock, upload, container).await
}

/// Isi berkas konten publik (gambar konten CMS) untuk disajikan backend (ADR-027 butir 5).
pub struct PublicFile {
    /// Tipe MIME menurut isi tervalidasi (`image/jpeg` atau `image/png`).
    pub mime_type: &'static str,
    /// Isi berkas.
    pub bytes: Vec<u8>,
}

impl fmt::Debug for PublicFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PublicFile")
            .field("mime_type", &self.mime_type)
            .field("len", &self.bytes.len())
            .finish()
    }
}

/// Membaca berkas tervalidasi di bucket `public_content` (F1-T93). Pemanggil wajib lebih dulu memastikan
/// berkas dipakai konten yang sedang tayang; fungsi ini hanya menjamin bucket, status hapus, dan jenis.
/// `None` bila tidak ada.
pub async fn read_public_content_file(
    pool: &PgPool,
    clock: &dyn Clock,
    storage: &FileStorage,
    stored_file_id: Uuid,
) -> Result<Option<PublicFile>, FileError> {
    service::read_public_content_file(pool, clock, storage, stored_file_id).await
}

/// Satu kode alasan aktif (A6.7 `reason_code`): data rujukan lintas modul milik `platform`, diisi lewat
/// migrasi data (misalnya daftar alasan penolakan identitas US-201 AC3, F1-T62).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReasonCode {
    /// Kode (snake_case).
    pub code: String,
    /// Label tampilan.
    pub label: String,
}

/// Database tidak tersedia saat membaca `reason_code` (detail hanya di log pemanggil).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReasonCodeUnavailable;

/// Kode alasan aktif (tanpa yang terhapus, ADR-003) satu `context` (nilai CHECK `ck_reason_code_context`,
/// misalnya `identity_rejection`), urut ID (urutan migrasi data). `conn` boleh koneksi transaksi pemanggil.
pub async fn active_reason_codes(
    conn: &mut PgConnection,
    context: &str,
) -> Result<Vec<ReasonCode>, ReasonCodeUnavailable> {
    service::active_reason_codes(conn, context).await
}

/// Membuang berkas tervalidasi yang tidak jadi dicatat (transaksi pemanggil gagal).
pub async fn discard_validated(
    clock: &dyn Clock,
    storage: &FileStorage,
    upload: ValidatedUpload,
) -> Result<(), FileError> {
    service::discard_validated(clock, storage, upload).await
}

/// Umur URL unduh (ADR-018 butir 3: "nilai awal 5 menit").
pub const DOWNLOAD_URL_TTL: Duration = Duration::from_secs(300);

/// Dasar akses berkas yang sudah diputuskan pemanggil (ADR-012 butir 1, 4; KU-03).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileAccess {
    /// Pemilik berkas (subjek data) membuka berkasnya sendiri: hanya `stored_file.owner_account_id` yang
    /// sama dengan pemanggil yang lolos. Tanpa E-05 (E-05 hanya untuk pihak selain subjek data, PRD §7.1).
    Owner,
    /// Admin Ruang Sehat dengan salah satu sub-peran `roles` (misalnya Verifikator membuka foto KTP,
    /// US-201) yang relasinya ke berkas sudah diperiksa modul pemanggil. E-05 dicatat dan di-commit
    /// sebelum URL diterbitkan; `subject` = subjek data berkas (PRD §7.2).
    Staff {
        /// Sub-peran yang diizinkan (minimal satu harus dipegang).
        roles: &'static [AdminSubRole],
        /// Subjek data berkas.
        subject: Subject,
    },
    /// Admin Ruang Sehat dengan salah satu sub-peran `roles` membuka dokumen perusahaan (Verifikator membuka
    /// NIB, NPWP, atau bukti kewenangan pendaftar, US-1902 AC1, F1-T104) yang relasinya ke berkas sudah
    /// diperiksa modul pemanggil. Seperti [`FileAccess::Staff`], E-05 dicatat dan di-commit sebelum URL
    /// diterbitkan; perusahaan bukan subjek data pribadi, sehingga event tanpa subjek dan `tenant_id` =
    /// `corporate_client_id` (ADR-028 butir 1).
    StaffCorporate {
        /// Sub-peran yang diizinkan (minimal satu harus dipegang).
        roles: &'static [AdminSubRole],
        /// Perusahaan pemilik dokumen.
        corporate_client_id: Uuid,
    },
}

/// Permintaan URL unduh.
#[derive(Debug, Clone, Copy)]
pub struct DownloadRequest<'a> {
    /// ID `stored_file`.
    pub stored_file_id: Uuid,
    /// Tujuan berkas yang diharapkan endpoint (berkas kategori/bucket lain → 404).
    pub purpose: UploadPurpose,
    /// Pola rute untuk security log (misalnya `/backoffice/v1/identity-verifications/{id}/ktp-photo`).
    pub route: &'a str,
    /// Dasar akses.
    pub access: FileAccess,
}

/// URL unduh berumur pendek. URL adalah rahasia: tidak pernah dicatat atau disimpan (ADR-018 butir 3).
#[derive(Clone, PartialEq, Eq)]
pub struct DownloadTicket {
    /// URL GET bertanda tangan dengan `Content-Disposition: attachment` dan `Cache-Control: private,
    /// no-store`.
    pub url: String,
    /// Instan kedaluwarsa URL.
    pub expires_at: OffsetDateTime,
    /// Tipe isi tervalidasi.
    pub mime_type: String,
    /// Ukuran berkas (byte).
    pub size_bytes: i32,
}

impl fmt::Debug for DownloadTicket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DownloadTicket")
            .field("url", &"[REDACTED]")
            .field("expires_at", &self.expires_at)
            .finish_non_exhaustive()
    }
}

/// Menerbitkan URL unduh berumur pendek untuk satu `stored_file` (F1-T30, ADR-018 butir 3):
/// 1. berkas dicari (tanpa yang terhapus/dimusnahkan, ADR-003); tidak ada, kategori lain, atau tidak boleh
///    diakses menurut [`FileAccess`] → [`FileError::NotFound`] yang sama (KU-03, ADR-012 butir 4) lewat
///    `authz::visible_object` (security log `access_denied`); percobaan atas berkas yang ada juga dicatat
///    E-05 hasil gagal di transaksi tersendiri (KU-03, coding-standards §4.4 butir 3);
/// 2. akses [`FileAccess::Staff`] dan [`FileAccess::StaffCorporate`]: E-05 ditulis dan di-commit lewat
///    `audit::in_audited_tx` sebelum URL dibuat; gagal mencatat → akses ditolak (500) tanpa URL
///    (coding-standards §4.4 butir 2);
/// 3. URL GET bertanda tangan [`DOWNLOAD_URL_TTL`] dengan header respons lampiran.
pub async fn issue_download_url(
    pool: &PgPool,
    clock: &dyn Clock,
    storage: &FileStorage,
    principal: &Principal,
    audit: &AuditContext,
    request: DownloadRequest<'_>,
) -> Result<DownloadTicket, FileError> {
    service::issue_download_url(pool, clock, storage, principal, audit, request).await
}

/// Hasil satu putaran [`purge_expired_upload_slots`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SlotPurgeReport {
    /// Slot yang objeknya dan barisnya terhapus.
    pub purged: usize,
    /// Slot yang gagal dibersihkan (dicoba lagi putaran berikutnya).
    pub failed: usize,
}

/// Pembersih (TD-17): sampai `limit` slot yang `expires_at`-nya lewat lebih dari
/// [`UPLOAD_SLOT_TOMBSTONE_GRACE`] dihapus beserta semua versi objek mentahnya dan hasil kode ulang yang
/// tidak pernah dicatat di `stored_file`. Idempoten; slot yang gagal dibiarkan untuk putaran berikutnya.
pub async fn purge_expired_upload_slots(
    pool: &PgPool,
    clock: &dyn Clock,
    storage: &FileStorage,
    limit: u16,
) -> Result<SlotPurgeReport, FileError> {
    service::purge_expired_upload_slots(pool, clock, storage, limit).await
}

/// Hasil [`reconcile_bucket`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ReconcileReport {
    /// Objek yatim (tanpa slot atau `stored_file`) yang dihapus beserta semua versinya.
    pub orphans_deleted: usize,
    /// Objek yatim yang gagal dihapus (dicoba lagi pada rekonsiliasi berikutnya).
    pub orphans_failed: usize,
    /// Baris `stored_file` belum dimusnahkan yang objeknya tidak ada (dicatat `stored_file_object_missing`).
    pub missing_objects: usize,
}

/// Rekonsiliasi berkala satu bucket (TD-17): objek di `incoming/` tanpa slot dan objek di `files/` tanpa
/// `stored_file` maupun klaim slot dihapus semua versinya; baris `stored_file` yang objeknya hilang
/// dilaporkan di log aplikasi (tanpa mengubah data). Objek di luar kedua prefiks tidak disentuh.
pub async fn reconcile_bucket(
    pool: &PgPool,
    clock: &dyn Clock,
    storage: &FileStorage,
    bucket: LogicalBucket,
) -> Result<ReconcileReport, FileError> {
    service::reconcile_bucket(pool, clock, storage, bucket).await
}

/// Mengantrekan job pemeliharaan berkas (pembersih slot dan rekonsiliasi setiap bucket) yang jatuh tempo
/// sekarang, dengan kunci deduplikasi sehingga pemanggilan berulang tidak menggandakan job yang masih
/// antre. Dipanggil penjadwal pekerja secara berkala.
pub async fn enqueue_file_maintenance(pool: &PgPool, clock: &dyn Clock) -> Result<(), FileError> {
    service::enqueue_file_maintenance(pool, clock).await
}

/// Handler job pemeliharaan berkas untuk `platform_jobs::run_batch` (jenis [`UPLOAD_SLOT_PURGE_JOB`] dan
/// [`FILE_RECONCILE_JOB`]). Galat object storage atau database → dicoba ulang dengan backoff.
pub async fn run_file_maintenance_job(
    pool: &PgPool,
    clock: &dyn Clock,
    storage: &FileStorage,
    job: &ClaimedJob,
) -> JobOutcome {
    service::run_file_maintenance_job(pool, clock, storage, job).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use observability::ConfigErrorKind;

    const SECRET: &str = "sintetis-uji-rahasia-0123456789abcdef";

    fn env(pairs: &[(&'static str, &'static str)]) -> EnvSource {
        let mut all = vec![
            ("DEPLOY_ENV", "local"),
            ("OBJECT_STORAGE_ENDPOINT", "http://127.0.0.1:18333"),
            ("OBJECT_STORAGE_REGION", "us-east-1"),
            ("OBJECT_STORAGE_ADDRESSING", "path"),
            ("OBJECT_STORAGE_ACCESS_KEY_ID", "sintetis-lokal-s3-akses"),
            ("OBJECT_STORAGE_SECRET_ACCESS_KEY", SECRET),
            ("OBJECT_STORAGE_BUCKET_PREFIX", "rs-local-"),
        ];
        for (k, v) in pairs {
            all.retain(|(ek, _)| ek != k);
            all.push((k, v));
        }
        EnvSource::from_pairs(all)
    }

    #[test]
    fn local_config_loads_with_defaults_and_physical_bucket_names() {
        let c = FileStorageConfig::load(&env(&[])).unwrap();
        assert_eq!(c.slot_ttl, Duration::from_secs(600));
        assert_eq!(c.upload_method, UploadMethod::Post);
        assert_eq!(c.validation_concurrency, 2);
        assert_eq!(c.validation_queue_timeout, Duration::from_secs(10));
        assert!(!format!("{c:?}").contains(SECRET));
        let s = FileStorage::new(&c).unwrap();
        assert_eq!(
            s.bucket_name(LogicalBucket::IdentityDocument),
            "rs-local-identity-document"
        );
        assert!(!format!("{s:?}").contains(SECRET));
    }

    // trace: KEPUTUSAN-235 (http:// ditolak di luar lokal)
    #[test]
    fn http_endpoint_outside_local_is_rejected() {
        for deploy in ["staging", "production"] {
            let c = FileStorageConfig::load(&env(&[("DEPLOY_ENV", deploy)])).unwrap();
            assert_eq!(
                FileStorage::new(&c).err(),
                Some(FileStorageSetupError(
                    "OBJECT_STORAGE_ENDPOINT (http:// hanya untuk DEPLOY_ENV=local)"
                ))
            );
        }
        // https belum didukung klien server (gagal-tertutup saat start).
        let c = FileStorageConfig::load(&env(&[
            ("DEPLOY_ENV", "production"),
            ("OBJECT_STORAGE_ENDPOINT", "https://nos.jkt-1.neo.id"),
            ("OBJECT_STORAGE_ADDRESSING", "virtual_host"),
        ]))
        .unwrap();
        assert_eq!(
            FileStorage::new(&c).err(),
            Some(FileStorageSetupError(
                "OBJECT_STORAGE_ENDPOINT (https:// belum didukung)"
            ))
        );
    }

    #[test]
    fn invalid_values_fail_fast_by_name() {
        let e = FileStorageConfig::load(&env(&[("DEPLOY_ENV", "dev")])).unwrap_err();
        assert_eq!(
            (e.kind(), e.name()),
            (ConfigErrorKind::Invalid, "DEPLOY_ENV")
        );
        let e =
            FileStorageConfig::load(&env(&[("OBJECT_STORAGE_ADDRESSING", "auto")])).unwrap_err();
        assert_eq!(e.name(), "OBJECT_STORAGE_ADDRESSING");
        let e = FileStorageConfig::load(&env(&[("UPLOAD_METHOD", "get")])).unwrap_err();
        assert_eq!(e.name(), "UPLOAD_METHOD");
        let bad = |pairs: &[(&'static str, &'static str)]| {
            FileStorage::new(&FileStorageConfig::load(&env(pairs)).unwrap()).err()
        };
        assert_eq!(
            bad(&[("OBJECT_STORAGE_BUCKET_PREFIX", "RS_")]),
            Some(FileStorageSetupError("OBJECT_STORAGE_BUCKET_PREFIX"))
        );
        assert_eq!(
            bad(&[("UPLOAD_SLOT_TTL_SECONDS", "30")]),
            Some(FileStorageSetupError("UPLOAD_SLOT_TTL_SECONDS"))
        );
        assert_eq!(
            bad(&[("UPLOAD_SLOT_TTL_SECONDS", "3601")]),
            Some(FileStorageSetupError("UPLOAD_SLOT_TTL_SECONDS"))
        );
        assert_eq!(
            bad(&[("FILE_VALIDATION_CONCURRENCY", "0")]),
            Some(FileStorageSetupError("FILE_VALIDATION_CONCURRENCY"))
        );
        // TD-17: batas atas konkurensi diturunkan (16 tidak lagi diterima).
        assert_eq!(
            bad(&[("FILE_VALIDATION_CONCURRENCY", "5")]),
            Some(FileStorageSetupError("FILE_VALIDATION_CONCURRENCY"))
        );
        assert_eq!(
            bad(&[("FILE_VALIDATION_CONCURRENCY", "16")]),
            Some(FileStorageSetupError("FILE_VALIDATION_CONCURRENCY"))
        );
        assert!(bad(&[("FILE_VALIDATION_CONCURRENCY", "4")]).is_none());
        for t in ["0", "61"] {
            assert_eq!(
                bad(&[("FILE_VALIDATION_QUEUE_TIMEOUT_SECONDS", t)]),
                Some(FileStorageSetupError(
                    "FILE_VALIDATION_QUEUE_TIMEOUT_SECONDS"
                ))
            );
        }
        assert_eq!(
            bad(&[("OBJECT_STORAGE_SECRET_ACCESS_KEY", "pendek")]),
            Some(FileStorageSetupError("OBJECT_STORAGE_*"))
        );
    }

    // trace: TD-17 (anggaran memori: konkurensi x puncak per validasi)
    #[test]
    fn validation_memory_budget_is_bounded() {
        let max = u64::try_from(MAX_FILE_VALIDATION_CONCURRENCY).unwrap();
        assert!(max * PEAK_VALIDATION_MEMORY_BYTES <= 2 * 1024 * 1024 * 1024);
    }

    // trace: TD-17 (permit diambil dengan batas tunggu; antrean penuh -> 503 Busy)
    #[tokio::test]
    async fn validation_queue_times_out_when_permits_are_exhausted() {
        let c = FileStorageConfig::load(&env(&[
            ("FILE_VALIDATION_CONCURRENCY", "1"),
            ("FILE_VALIDATION_QUEUE_TIMEOUT_SECONDS", "1"),
        ]))
        .unwrap();
        let s = FileStorage::new(&c).unwrap();
        let held = service::acquire_validation_permit(&s).await.unwrap();
        let started = std::time::Instant::now();
        assert!(matches!(
            service::acquire_validation_permit(&s).await,
            Err(FileError::Busy)
        ));
        assert!(started.elapsed() >= Duration::from_secs(1));
        drop(held);
        assert!(service::acquire_validation_permit(&s).await.is_ok());
    }
}
