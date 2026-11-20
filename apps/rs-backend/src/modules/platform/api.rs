//! Antarmuka publik modul `platform` untuk modul lain:
//! - pemeriksaan kesiapan yang dapat didaftarkan (F1-T11);
//! - berkas unggahan (F1-T29, ADR-018): slot unggah dengan URL bertanda tangan, validasi isi pasca-unggah,
//!   dan pencatatan `stored_file` di transaksi milik pemanggil.
//!
//! Alur berkas (tiga langkah ADR-018 konsekuensi): (1) [`create_upload_slot`] → klien mengunggah langsung ke
//! object storage dengan [`UploadInstruction`]; (2) [`validate_upload`] di luar transaksi database
//! (memanggil object storage, KEPUTUSAN-125): ukuran, jenis dari isi, kode ulang foto, objek asli beserta
//! semua versinya dihapus; (3) [`record_stored_file`] di transaksi use case pemanggil (misalnya
//! `audit::in_audited_tx` bersama event E-10 pengajuan identitas, US-106), sekaligus memakai slot sekali.
//! Bila transaksi pemanggil gagal, pemanggil memanggil [`discard_validated`]. Berkas baru dapat dirujuk
//! setelah langkah 3 (ADR-018 butir 2).

use super::service;
use foundation::{Clock, Uuid};
use object_storage::{
    AddressingStyle, Deployment, ObjectStorageConfig, ObjectStore, valid_bucket_name,
};
use observability::{ConfigError, EnvSource};
use sqlx::{PgConnection, PgPool};
use std::fmt;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use time::OffsetDateTime;
use tokio::sync::Semaphore;

pub use super::domain::{LogicalBucket, MAX_PHOTO_BYTES, PhotoFormat, UploadPurpose};
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

/// Bawaan umur slot dan URL unggah (detik). Usulan teknis F1-T29: ADR-018 hanya menyebut "berumur pendek";
/// 10 menit memberi ruang unggah 5 MiB lewat jaringan seluler lambat.
pub const DEFAULT_UPLOAD_SLOT_TTL_SECONDS: u64 = 600;
/// Rentang umur slot yang diterima konfigurasi (detik).
pub const UPLOAD_SLOT_TTL_RANGE: std::ops::RangeInclusive<u64> = 60..=3600;
/// Bawaan batas waktu satu operasi object storage (detik).
pub const DEFAULT_OBJECT_STORAGE_TIMEOUT_SECONDS: u64 = 10;
/// Bawaan jumlah validasi foto yang berjalan bersamaan (dekode memakan memori, lihat `domain`).
pub const DEFAULT_FILE_VALIDATION_CONCURRENCY: usize = 2;

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
    /// Jumlah validasi foto bersamaan.
    pub validation_concurrency: usize,
}

impl FileStorageConfig {
    /// Memuat dari variabel lingkungan; gagal cepat tanpa mencetak nilai.
    ///
    /// `DEPLOY_ENV` (`local`|`staging`|`production`), `OBJECT_STORAGE_ENDPOINT`, `OBJECT_STORAGE_REGION`,
    /// `OBJECT_STORAGE_ADDRESSING` (`path`|`virtual_host`), `OBJECT_STORAGE_ACCESS_KEY_ID`,
    /// `OBJECT_STORAGE_SECRET_ACCESS_KEY` (rahasia), `OBJECT_STORAGE_BUCKET_PREFIX`; opsional
    /// `OBJECT_STORAGE_TIMEOUT_SECONDS`, `UPLOAD_SLOT_TTL_SECONDS`, `UPLOAD_METHOD` (`post`|`put`),
    /// `FILE_VALIDATION_CONCURRENCY`.
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
    /// nama bucket fisik ketujuh bucket logis, umur slot, dan konkurensi.
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
        if !(1..=16).contains(&config.validation_concurrency) {
            return Err(FileStorageSetupError("FILE_VALIDATION_CONCURRENCY"));
        }
        Ok(Self {
            store,
            bucket_prefix: config.bucket_prefix.clone(),
            slot_ttl: config.slot_ttl,
            upload_method: config.upload_method,
            permits: Arc::new(Semaphore::new(config.validation_concurrency)),
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
pub struct ValidatedUpload {
    pub(super) upload_slot_id: Uuid,
    pub(super) owner_account_id: Uuid,
    pub(super) purpose: UploadPurpose,
    pub(super) object_key: String,
    pub(super) format: PhotoFormat,
    pub(super) size_bytes: i32,
    pub(super) sha256: [u8; crypto::SHA256_LEN],
    pub(super) validated_at: OffsetDateTime,
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

/// Langkah 3: di transaksi milik pemanggil, memakai slot sekali dan menulis `stored_file`. Mengembalikan ID
/// `stored_file`. Slot yang sudah dipakai menghasilkan [`FileError::NotFound`].
pub async fn record_stored_file(
    conn: &mut PgConnection,
    upload: &ValidatedUpload,
    container: Option<FileContainer>,
) -> Result<Uuid, FileError> {
    service::record_stored_file(conn, upload, container).await
}

/// Membuang berkas tervalidasi yang tidak jadi dicatat (transaksi pemanggil gagal).
pub async fn discard_validated(
    clock: &dyn Clock,
    storage: &FileStorage,
    upload: ValidatedUpload,
) -> Result<(), FileError> {
    service::discard_validated(clock, storage, upload).await
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
        assert_eq!(
            bad(&[("OBJECT_STORAGE_SECRET_ACCESS_KEY", "pendek")]),
            Some(FileStorageSetupError("OBJECT_STORAGE_*"))
        );
    }
}
