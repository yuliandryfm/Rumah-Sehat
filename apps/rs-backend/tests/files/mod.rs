//! Pembantu test integrasi berkas (F1-T29): PostgreSQL dari `scripts/db-ephemeral.sh` (role runtime
//! `rs_api`/`rs_app`) dan SeaweedFS efemeral dari image `infra/local/compose.yaml` (KEPUTUSAN-233).
//!
//! Tanpa variabel database atau Docker, test GAGAL (tidak dilewati). Kredensial S3 acak per kontainer lewat
//! variabel lingkungan proses `docker`; data di sistem berkas kontainer (bukan `~/ruang-sehat-data`), dan
//! kontainer dihapus saat dibuang. Unggahan sisi klien memakai `curl` (klien tanpa kredensial).
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, dead_code)]

use foundation::{Clock, Sensitive, SystemClock, Uuid, new_id};
use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::PngEncoder;
use image::{DynamicImage, Rgb, RgbImage, Rgba, RgbaImage};
use object_storage::{AddressingStyle, Deployment, ObjectStorageConfig, ObjectStore};
use observability::EnvSource;
use rs_backend::modules::platform::api::{
    FileStorage, FileStorageConfig, LogicalBucket, UploadInstruction, UploadMethod,
};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::Duration;
use time::OffsetDateTime;

pub fn now() -> OffsetDateTime {
    SystemClock.now()
}

pub fn random_hex(bytes: usize) -> String {
    let mut b = vec![0u8; bytes];
    crypto::fill_random(&mut b).unwrap();
    b.iter().map(|x| format!("{x:02x}")).collect()
}

fn run(cmd: &mut Command) -> Output {
    cmd.output()
        .unwrap_or_else(|e| panic!("perintah tidak dapat dijalankan: {e}"))
}

// ---------------------------------------------------------------- PostgreSQL

/// Login `rs_api` (anggota `rs_app`): hak runtime sungguhan.
pub async fn app_pool() -> PgPool {
    let env = EnvSource::from_process();
    let url = env
        .required_secret("RS_TEST_DB_APP_URL")
        .unwrap_or_else(|_| {
            panic!(
                "RS_TEST_DB_APP_URL tidak diset: test integrasi berkas wajib dijalankan lewat \
             `bash scripts/db-ephemeral.sh cargo test ...` (dipakai scripts/verify.sh)"
            )
        });
    PgPoolOptions::new()
        .max_connections(4)
        .connect(url.expose())
        .await
        .unwrap_or_else(|_| panic!("koneksi RS_TEST_DB_APP_URL gagal"))
}

/// Akun pasien sintetis (email domain contoh RFC 2606).
pub async fn new_account(pool: &PgPool) -> Uuid {
    let id = new_id();
    let subject = format!("uji-{}", random_hex(8));
    let email = format!("{subject}@example.com");
    sqlx::query!(
        "INSERT INTO app.account (id, account_type, idp_realm, idp_subject, email, status, status_changed_at) \
         VALUES ($1, 'patient', 'pasien', $2, $3, 'active', now())",
        id,
        subject,
        email,
    )
    .execute(pool)
    .await
    .unwrap();
    id
}

pub async fn slot_exists(pool: &PgPool, id: Uuid) -> bool {
    sqlx::query_scalar!(
        r#"SELECT count(*) AS "n!" FROM app.upload_slot WHERE id = $1"#,
        id
    )
    .fetch_one(pool)
    .await
    .unwrap()
        == 1
}

pub async fn slot_object_key(pool: &PgPool, id: Uuid) -> String {
    sqlx::query_scalar!("SELECT object_key FROM app.upload_slot WHERE id = $1", id)
        .fetch_one(pool)
        .await
        .unwrap()
}

/// Slot ada dan sudah menjadi tombstone (`consumed_at` terisi, TD-17).
pub async fn slot_consumed(pool: &PgPool, id: Uuid) -> bool {
    sqlx::query_scalar!(
        r#"SELECT consumed_at IS NOT NULL AS "consumed!" FROM app.upload_slot WHERE id = $1"#,
        id
    )
    .fetch_optional(pool)
    .await
    .unwrap()
    .unwrap_or(false)
}

/// Memundurkan `expires_at` slot sehingga sudah lewat batas pembersihan (`expires_at` + jeda 15 menit)
/// menurut jam sungguhan. Jam aplikasi tidak dimajukan: permintaan bertanda tangan SigV4 dengan waktu
/// ±25 menit di depan jam penyedia diamati ditolak 403 oleh SeaweedFS [SRC-562], jadi pembersih di test
/// memakai `SystemClock`.
pub async fn expire_slot_past_grace(pool: &PgPool, id: Uuid) {
    let n = sqlx::query!(
        "UPDATE app.upload_slot SET expires_at = $2 WHERE id = $1",
        id,
        now() - time::Duration::minutes(16),
    )
    .execute(pool)
    .await
    .unwrap()
    .rows_affected();
    assert_eq!(n, 1, "slot tidak ada");
}

pub struct StoredFileRow {
    pub bucket: String,
    pub object_key: String,
    pub category: String,
    pub owner_account_id: Option<Uuid>,
    pub created_by: Option<Uuid>,
    pub container_type: Option<String>,
    pub container_id: Option<Uuid>,
    pub mime_type: String,
    pub size_bytes: i32,
    pub sha256: Vec<u8>,
    pub deleted_at: Option<OffsetDateTime>,
}

pub async fn stored_file(pool: &PgPool, id: Uuid) -> StoredFileRow {
    sqlx::query_as!(
        StoredFileRow,
        "SELECT bucket, object_key, category, owner_account_id, created_by, container_type, container_id, \
                mime_type, size_bytes, sha256, deleted_at \
         FROM app.stored_file WHERE id = $1",
        id
    )
    .fetch_one(pool)
    .await
    .unwrap()
}

// ---------------------------------------------------------------- SeaweedFS

pub struct SeaweedFs {
    id: String,
    pub endpoint: String,
    access_key: String,
    secret_key: String,
}

const ENTRYPOINT: &str = r#"printf '{"identities":[{"name":"dev","credentials":[{"accessKey":"%s","secretKey":"%s"}],"actions":["Admin","Read","Write","List","Tagging"]}]}' "$S3_ACCESS_KEY" "$S3_SECRET_KEY" > /tmp/s3.json && exec weed server -dir=/data -s3 -s3.port=8333 -s3.config=/tmp/s3.json -master.volumeSizeLimitMB=64"#;

fn image_from_compose() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../infra/local/compose.yaml");
    let text = std::fs::read_to_string(path).expect("infra/local/compose.yaml terbaca");
    text.lines()
        .map(str::trim)
        .find_map(|l| l.strip_prefix("image: chrislusf/seaweedfs:"))
        .map(|rest| format!("chrislusf/seaweedfs:{rest}"))
        .expect("image seaweedfs di compose lokal")
}

impl SeaweedFs {
    pub async fn start() -> Self {
        let image = image_from_compose();
        assert!(
            image.contains("@sha256:"),
            "image SeaweedFS wajib dipin digest"
        );
        let access_key = format!("uji-{}", random_hex(8));
        let secret_key = random_hex(20);
        let out = run(Command::new("docker")
            .args([
                "run",
                "-d",
                "--rm",
                "-e",
                "S3_ACCESS_KEY",
                "-e",
                "S3_SECRET_KEY",
            ])
            .args([
                "-p",
                "127.0.0.1::8333",
                "--entrypoint",
                "sh",
                &image,
                "-c",
                ENTRYPOINT,
            ])
            .env("S3_ACCESS_KEY", &access_key)
            .env("S3_SECRET_KEY", &secret_key));
        assert!(
            out.status.success(),
            "docker run SeaweedFS gagal: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        let id = String::from_utf8(out.stdout).unwrap().trim().to_string();
        let mut sw = SeaweedFs {
            id,
            endpoint: String::new(),
            access_key,
            secret_key,
        };
        let out = run(Command::new("docker").args(["port", &sw.id, "8333/tcp"]));
        let mapped = String::from_utf8(out.stdout).unwrap();
        let hostport = mapped
            .lines()
            .find(|l| l.starts_with("127.0.0.1:"))
            .unwrap_or_else(|| panic!("port SeaweedFS tidak dipetakan: {mapped}"))
            .trim()
            .to_string();
        sw.endpoint = format!("http://{hostport}");
        let store = ObjectStore::new(&sw.object_storage_config()).unwrap();
        let probe = format!("rs-probe-{}", random_hex(4));
        for _ in 0..240 {
            if store.create_bucket(&probe, now()).await.is_ok()
                && store
                    .put_object(&probe, "probe", "text/plain", b"ok".to_vec(), now())
                    .await
                    .is_ok()
            {
                return sw;
            }
            tokio::time::sleep(Duration::from_millis(250)).await;
        }
        panic!("SeaweedFS tidak siap dalam 60 detik");
    }

    pub fn object_storage_config(&self) -> ObjectStorageConfig {
        ObjectStorageConfig {
            endpoint: self.endpoint.clone(),
            region: "us-east-1".to_string(),
            addressing: AddressingStyle::Path,
            access_key_id: self.access_key.clone(),
            secret_access_key: Sensitive::new(self.secret_key.clone()),
            deployment: Deployment::Local,
            timeout: Duration::from_secs(20),
        }
    }

    /// Layanan berkas dengan awalan bucket unik; bucket dokumen identitas dibuat dengan versi objek aktif.
    pub async fn file_storage(&self, method: UploadMethod, ttl: Duration) -> FileStorage {
        let config = FileStorageConfig {
            object_storage: self.object_storage_config(),
            bucket_prefix: format!("rs-uji-{}-", random_hex(3)),
            slot_ttl: ttl,
            upload_method: method,
            validation_concurrency: 2,
            validation_queue_timeout: Duration::from_secs(10),
        };
        let storage = FileStorage::new(&config).unwrap();
        let bucket = storage.bucket_name(LogicalBucket::IdentityDocument);
        let store = storage.object_store();
        store.create_bucket(&bucket, now()).await.unwrap();
        store.enable_versioning(&bucket, now()).await.unwrap();
        storage
    }

    /// Membuat (dengan versi objek) bucket logis lain selain dokumen identitas.
    pub async fn create_other_buckets(&self, storage: &FileStorage) {
        let store = storage.object_store();
        for b in LogicalBucket::ALL {
            if b == LogicalBucket::IdentityDocument {
                continue;
            }
            let name = storage.bucket_name(b);
            store.create_bucket(&name, now()).await.unwrap();
            store.enable_versioning(&name, now()).await.unwrap();
        }
    }
}

impl Drop for SeaweedFs {
    fn drop(&mut self) {
        let _ = Command::new("docker").args(["rm", "-f", &self.id]).output();
    }
}

// ---------------------------------------------------------------- klien unggah (curl)

fn temp_file(bytes: &[u8]) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("platform-files-uji");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(random_hex(8));
    std::fs::write(&path, bytes).unwrap();
    path
}

fn http_code(out: Output) -> u16 {
    String::from_utf8(out.stdout)
        .unwrap()
        .trim()
        .parse()
        .unwrap_or_else(|_| panic!("curl gagal: {}", String::from_utf8_lossy(&out.stderr)))
}

/// Mengunggah seperti klien sesuai instruksi slot; mengembalikan kode HTTP penyedia.
pub fn upload(instruction: &UploadInstruction, bytes: &[u8]) -> u16 {
    let path = temp_file(bytes);
    let mut cmd = Command::new("curl");
    cmd.args(["-s", "-o", "/dev/null", "-w", "%{http_code}"]);
    match instruction {
        UploadInstruction::Post { url, fields } => {
            for (k, v) in fields {
                cmd.arg("--form-string").arg(format!("{k}={v}"));
            }
            cmd.arg("-F").arg(format!("file=@{}", path.display()));
            cmd.arg(url);
        }
        UploadInstruction::Put { url, headers } => {
            cmd.args(["-X", "PUT"]);
            for (k, v) in headers {
                if k != "Content-Length" {
                    cmd.arg("-H").arg(format!("{k}: {v}"));
                }
            }
            cmd.arg("--data-binary").arg(format!("@{}", path.display()));
            cmd.arg(url);
        }
    }
    let code = http_code(run(&mut cmd));
    let _ = std::fs::remove_file(path);
    code
}

// ---------------------------------------------------------------- berkas uji sintetis

/// Penanda teks di metadata (deskripsi EXIF / chunk teks PNG) yang wajib hilang setelah proses.
pub const METADATA_MARKER: &[u8] = b"LOKASI-SINTETIS-GPS-6.2-106.8";

pub fn jpeg(w: u32, h: u32) -> Vec<u8> {
    let img = RgbImage::from_fn(w, h, |x, y| {
        Rgb([(x * 37 % 256) as u8, (y * 53 % 256) as u8, 128])
    });
    let mut out = Vec::new();
    DynamicImage::ImageRgb8(img)
        .write_with_encoder(JpegEncoder::new_with_quality(&mut out, 90))
        .unwrap();
    out
}

pub fn png(w: u32, h: u32) -> Vec<u8> {
    let img = RgbaImage::from_fn(w, h, |x, y| {
        Rgba([(x * 37 % 256) as u8, (y * 53 % 256) as u8, 7, 255])
    });
    let mut out = Vec::new();
    DynamicImage::ImageRgba8(img)
        .write_with_encoder(PngEncoder::new(&mut out))
        .unwrap();
    out
}

/// TIFF little-endian berisi IFD0: ImageDescription (penanda) dan Orientation = 6 (putar 90°).
fn exif_tiff() -> Vec<u8> {
    let mut desc = METADATA_MARKER.to_vec();
    desc.push(0);
    let mut t = Vec::new();
    t.extend_from_slice(b"II\x2a\x00\x08\x00\x00\x00");
    t.extend_from_slice(&2u16.to_le_bytes());
    // 0x010E ImageDescription, ASCII, count, offset data = 8 + 2 + 2*12 + 4 = 38
    t.extend_from_slice(&0x010Eu16.to_le_bytes());
    t.extend_from_slice(&2u16.to_le_bytes());
    t.extend_from_slice(&(desc.len() as u32).to_le_bytes());
    t.extend_from_slice(&38u32.to_le_bytes());
    // 0x0112 Orientation, SHORT, 1, nilai 6
    t.extend_from_slice(&0x0112u16.to_le_bytes());
    t.extend_from_slice(&3u16.to_le_bytes());
    t.extend_from_slice(&1u32.to_le_bytes());
    t.extend_from_slice(&[6, 0, 0, 0]);
    t.extend_from_slice(&0u32.to_le_bytes());
    t.extend_from_slice(&desc);
    t
}

/// JPEG dengan segmen APP1 EXIF (orientasi 6 + penanda) tepat setelah SOI.
pub fn jpeg_with_exif(w: u32, h: u32) -> Vec<u8> {
    let base = jpeg(w, h);
    let mut payload = b"Exif\x00\x00".to_vec();
    payload.extend_from_slice(&exif_tiff());
    let len = (payload.len() + 2) as u16;
    let mut out = base[..2].to_vec();
    out.extend_from_slice(&[0xFF, 0xE1]);
    out.extend_from_slice(&len.to_be_bytes());
    out.extend_from_slice(&payload);
    out.extend_from_slice(&base[2..]);
    out
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &b in data {
        crc ^= u32::from(b);
        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                (crc >> 1) ^ 0xEDB8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

fn png_chunk(kind: &[u8; 4], data: &[u8]) -> Vec<u8> {
    let mut c = (data.len() as u32).to_be_bytes().to_vec();
    let mut body = kind.to_vec();
    body.extend_from_slice(data);
    c.extend_from_slice(&body);
    c.extend_from_slice(&crc32(&body).to_be_bytes());
    c
}

/// PNG dengan chunk `tEXt` dan `eXIf` (penanda) setelah IHDR.
pub fn png_with_metadata(w: u32, h: u32) -> Vec<u8> {
    let base = png(w, h);
    // signature 8 + IHDR (4 len + 4 type + 13 data + 4 crc) = 33
    let mut out = base[..33].to_vec();
    let mut text = b"Comment\x00".to_vec();
    text.extend_from_slice(METADATA_MARKER);
    out.extend_from_slice(&png_chunk(b"tEXt", &text));
    out.extend_from_slice(&png_chunk(b"eXIf", &exif_tiff()));
    out.extend_from_slice(&base[33..]);
    out
}

/// Nama chunk PNG berurutan.
pub fn png_chunk_types(bytes: &[u8]) -> Vec<String> {
    let mut out = Vec::new();
    let mut i = 8;
    while i + 8 <= bytes.len() {
        let len = u32::from_be_bytes(bytes[i..i + 4].try_into().unwrap()) as usize;
        out.push(String::from_utf8_lossy(&bytes[i + 4..i + 8]).to_string());
        i += 12 + len;
    }
    out
}

pub fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len()).any(|w| w == needle)
}

/// Semua kunci berversi di bawah prefiks bucket dokumen identitas (TD-17).
pub async fn keys_under(storage: &FileStorage, prefix: &str) -> Vec<String> {
    let bucket = storage.bucket_name(LogicalBucket::IdentityDocument);
    let mut out = Vec::new();
    let mut cursor = None;
    loop {
        let page = storage
            .object_store()
            .list_version_keys(&bucket, prefix, cursor.as_ref(), now())
            .await
            .unwrap();
        out.extend(page.keys);
        match page.next {
            Some(n) => cursor = Some(n),
            None => break,
        }
    }
    out.sort();
    out.dedup();
    out
}
