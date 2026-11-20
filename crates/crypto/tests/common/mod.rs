//! OpenBao mode dev efemeral untuk test integrasi (dijalankan dari test lewat Docker CLI).
//!
//! Image diambil dari `infra/local/compose.yaml` (satu sumber versi + digest). Kontainer memakai port
//! host acak di 127.0.0.1, token root sintetis acak per kontainer (dikirim lewat variabel lingkungan
//! proses `docker`, bukan argumen), dan dihapus saat `OpenBao` dibuang. Docker wajib ada: bila tidak,
//! test GAGAL (tidak dilewati), sama seperti langkah basis data di skrip verifikasi.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, dead_code)]

use std::path::Path;
use std::process::{Command, Output};
use std::thread::sleep;
use std::time::Duration;

pub struct OpenBao {
    id: String,
    pub address: String,
    pub token: String,
}

fn image_from_compose() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../infra/local/compose.yaml");
    let text = std::fs::read_to_string(path).expect("infra/local/compose.yaml terbaca");
    text.lines()
        .map(str::trim)
        .find_map(|l| l.strip_prefix("image: openbao/openbao:"))
        .map(|rest| format!("openbao/openbao:{rest}"))
        .expect("image openbao di compose lokal")
}

fn random_token() -> String {
    let mut b = [0u8; 24];
    crypto::fill_random(&mut b).unwrap();
    let hex: String = b.iter().map(|x| format!("{x:02x}")).collect();
    format!("uji-{hex}")
}

fn run(cmd: &mut Command) -> Output {
    cmd.output()
        .unwrap_or_else(|e| panic!("docker tidak dapat dijalankan: {e}"))
}

impl OpenBao {
    pub fn start() -> Self {
        let image = image_from_compose();
        assert!(
            image.contains("@sha256:"),
            "image OpenBao wajib dipin digest: {image}"
        );
        let token = random_token();
        let out = run(Command::new("docker")
            .args(["run", "-d", "--rm", "--cap-add", "IPC_LOCK"])
            .args([
                "-e",
                "BAO_DEV_ROOT_TOKEN_ID",
                "-e",
                "BAO_DEV_LISTEN_ADDRESS=0.0.0.0:8200",
            ])
            .args(["-p", "127.0.0.1::8200", &image, "server", "-dev"])
            .env("BAO_DEV_ROOT_TOKEN_ID", &token));
        assert!(
            out.status.success(),
            "docker run OpenBao gagal: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        let id = String::from_utf8(out.stdout).unwrap().trim().to_string();
        let bao = OpenBao {
            id,
            address: String::new(),
            token,
        };
        let out = run(Command::new("docker").args(["port", &bao.id, "8200/tcp"]));
        let mapped = String::from_utf8(out.stdout).unwrap();
        let hostport = mapped
            .lines()
            .find(|l| l.starts_with("127.0.0.1:"))
            .unwrap_or_else(|| panic!("port OpenBao tidak dipetakan: {mapped}"))
            .trim()
            .to_string();
        let mut bao = bao;
        bao.address = format!("http://{hostport}");
        for _ in 0..120 {
            if bao.bao(&["status"]).status.success() {
                return bao;
            }
            sleep(Duration::from_millis(250));
        }
        panic!("OpenBao dev tidak siap dalam 30 detik");
    }

    /// Menjalankan CLI `bao` di dalam kontainer dengan token root (lewat variabel lingkungan).
    pub fn bao(&self, args: &[&str]) -> Output {
        run(Command::new("docker")
            .args([
                "exec",
                "-e",
                "BAO_ADDR=http://127.0.0.1:8200",
                "-e",
                "BAO_TOKEN",
                &self.id,
                "bao",
            ])
            .args(args)
            .env("BAO_TOKEN", &self.token))
    }

    /// Menjalankan CLI `bao` dengan token lain (misalnya token login operator, F1-T65).
    pub fn bao_as(&self, token: &str, args: &[&str]) -> Output {
        run(Command::new("docker")
            .args([
                "exec",
                "-e",
                "BAO_ADDR=http://127.0.0.1:8200",
                "-e",
                "BAO_TOKEN",
                &self.id,
                "bao",
            ])
            .args(args)
            .env("BAO_TOKEN", token))
    }

    /// Menjalankan CLI `bao` dengan token root dan masukan stdin (misalnya `policy write <nama> -`).
    pub fn bao_stdin(&self, args: &[&str], input: &str) {
        use std::io::Write;
        use std::process::Stdio;
        let mut child = Command::new("docker")
            .args([
                "exec",
                "-i",
                "-e",
                "BAO_ADDR=http://127.0.0.1:8200",
                "-e",
                "BAO_TOKEN",
                &self.id,
                "bao",
            ])
            .args(args)
            .env("BAO_TOKEN", &self.token)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("docker exec");
        child
            .stdin
            .take()
            .unwrap()
            .write_all(input.as_bytes())
            .unwrap();
        let out = child.wait_with_output().unwrap();
        assert!(
            out.status.success(),
            "bao {args:?} gagal: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }

    /// Seperti [`Self::bao`] tetapi wajib berhasil.
    pub fn bao_ok(&self, args: &[&str]) {
        let out = self.bao(args);
        assert!(
            out.status.success(),
            "bao {args:?} gagal: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }

    pub fn config(&self) -> crypto::OpenBaoConfig {
        self.config_with_token(&self.token)
    }

    pub fn config_with_token(&self, token: &str) -> crypto::OpenBaoConfig {
        crypto::OpenBaoConfig {
            address: self.address.clone(),
            token: foundation::Sensitive::new(token.to_string()),
            timeout: Duration::from_secs(10),
        }
    }
}

impl Drop for OpenBao {
    fn drop(&mut self) {
        let _ = Command::new("docker").args(["rm", "-f", &self.id]).output();
    }
}
