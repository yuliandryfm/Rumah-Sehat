//! SeaweedFS efemeral untuk test kontrak object storage (KEPUTUSAN-233, KEPUTUSAN-235), dijalankan dari
//! test lewat Docker CLI.
//!
//! Image diambil dari `infra/local/compose.yaml` (satu sumber versi + digest). Kontainer memakai port host
//! acak di 127.0.0.1, kredensial S3 sintetis acak per kontainer (dikirim lewat variabel lingkungan proses
//! `docker`, bukan argumen), data di sistem berkas kontainer (bukan `~/ruang-sehat-data`), dan dihapus saat
//! `SeaweedFs` dibuang. Docker wajib ada: bila tidak, test GAGAL (tidak dilewati). Unggahan sisi klien
//! (presigned POST/PUT) memakai `curl`, seperti klien nyata yang tidak memegang kredensial.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, dead_code)]

use foundation::{Clock, Sensitive, SystemClock};
use object_storage::{
    AddressingStyle, Deployment, ObjectStorageConfig, ObjectStore, PresignedPost, PresignedPut,
};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::Duration;
use time::OffsetDateTime;

pub struct SeaweedFs {
    id: String,
    pub endpoint: String,
    pub access_key: String,
    pub secret_key: String,
}

fn image_from_compose() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../infra/local/compose.yaml");
    let text = std::fs::read_to_string(path).expect("infra/local/compose.yaml terbaca");
    text.lines()
        .map(str::trim)
        .find_map(|l| l.strip_prefix("image: chrislusf/seaweedfs:"))
        .map(|rest| format!("chrislusf/seaweedfs:{rest}"))
        .expect("image seaweedfs di compose lokal")
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

/// Konfigurasi identitas S3 dibuat saat start dari env (sama dengan infra/local/compose.yaml).
const ENTRYPOINT: &str = r#"printf '{"identities":[{"name":"dev","credentials":[{"accessKey":"%s","secretKey":"%s"}],"actions":["Admin","Read","Write","List","Tagging"]}]}' "$S3_ACCESS_KEY" "$S3_SECRET_KEY" > /tmp/s3.json && exec weed server -dir=/data -s3 -s3.port=8333 -s3.config=/tmp/s3.json -master.volumeSizeLimitMB=64"#;

pub fn now() -> OffsetDateTime {
    SystemClock.now()
}

impl SeaweedFs {
    pub async fn start() -> Self {
        let image = image_from_compose();
        assert!(
            image.contains("@sha256:"),
            "image SeaweedFS wajib dipin digest: {image}"
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
        // Siap bila bucket dapat dibuat dan objek dapat ditulis (gateway S3 menjawab lebih dulu dari filer).
        let store = sw.store();
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

    pub fn config(&self) -> ObjectStorageConfig {
        self.config_with_secret(&self.secret_key)
    }

    pub fn config_with_secret(&self, secret: &str) -> ObjectStorageConfig {
        ObjectStorageConfig {
            endpoint: self.endpoint.clone(),
            region: "us-east-1".to_string(),
            addressing: AddressingStyle::Path,
            access_key_id: self.access_key.clone(),
            secret_access_key: Sensitive::new(secret.to_string()),
            deployment: Deployment::Local,
            timeout: Duration::from_secs(20),
        }
    }

    pub fn store(&self) -> ObjectStore {
        ObjectStore::new(&self.config()).unwrap()
    }

    /// Bucket baru dengan versi objek aktif (ADR-018 butir 5).
    pub async fn versioned_bucket(&self, prefix: &str) -> String {
        let bucket = format!("{prefix}-{}", random_hex(4));
        let store = self.store();
        store.create_bucket(&bucket, now()).await.unwrap();
        store.enable_versioning(&bucket, now()).await.unwrap();
        bucket
    }
}

impl Drop for SeaweedFs {
    fn drop(&mut self) {
        let _ = Command::new("docker").args(["rm", "-f", &self.id]).output();
    }
}

/// Berkas sementara di direktori sementara target Cargo (bukan direktori data lokal).
pub fn temp_file(bytes: &[u8]) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("object-storage-uji");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(random_hex(8));
    std::fs::write(&path, bytes).unwrap();
    path
}

fn http_code(out: Output) -> u16 {
    assert!(
        out.status.success() || !out.stdout.is_empty(),
        "curl gagal: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout)
        .unwrap()
        .trim()
        .parse()
        .unwrap()
}

/// Unggah lewat presigned POST seperti klien (curl, `multipart/form-data`, berkas field terakhir).
pub fn upload_post(post: &PresignedPost, bytes: &[u8]) -> u16 {
    upload_post_with(post, bytes, &[])
}

/// Seperti [`upload_post`], dengan field yang diganti nilainya (untuk test pelanggaran policy).
pub fn upload_post_with(post: &PresignedPost, bytes: &[u8], overrides: &[(&str, &str)]) -> u16 {
    let path = temp_file(bytes);
    let mut cmd = Command::new("curl");
    cmd.args(["-s", "-o", "/dev/null", "-w", "%{http_code}"]);
    for (k, v) in &post.fields {
        let v = overrides
            .iter()
            .find(|(ok, _)| ok == k)
            .map(|(_, ov)| (*ov).to_string())
            .unwrap_or_else(|| v.clone());
        cmd.arg("--form-string").arg(format!("{k}={v}"));
    }
    cmd.arg("-F").arg(format!("file=@{}", path.display()));
    cmd.arg(&post.url);
    let code = http_code(run(&mut cmd));
    let _ = std::fs::remove_file(path);
    code
}

/// Unggah lewat presigned PUT dengan header yang diminta (dapat diganti untuk test pelanggaran).
pub fn upload_put(put: &PresignedPut, bytes: &[u8], content_type: Option<&str>) -> u16 {
    let path = temp_file(bytes);
    let ct = content_type.map(str::to_string).unwrap_or_else(|| {
        put.headers
            .iter()
            .find(|(k, _)| k == "Content-Type")
            .map(|(_, v)| v.clone())
            .unwrap()
    });
    let out = run(Command::new("curl")
        .args(["-s", "-o", "/dev/null", "-w", "%{http_code}", "-X", "PUT"])
        .arg("-H")
        .arg(format!("Content-Type: {ct}"))
        .arg("--data-binary")
        .arg(format!("@{}", path.display()))
        .arg(&put.url));
    let _ = std::fs::remove_file(path);
    http_code(out)
}
