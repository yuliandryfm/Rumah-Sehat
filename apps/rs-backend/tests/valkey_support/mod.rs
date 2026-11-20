//! Salinan `crates/valkey/tests/common/mod.rs` (pembantu test tidak dibagi lewat fitur crate, TD-15).
//! Valkey efemeral untuk test integrasi: image dipin dari `infra/local/compose.yaml` (Valkey 9.1.3, sama
//! dengan lingkungan lokal dan production), port acak hanya di 127.0.0.1, password acak per kontainer
//! lewat variabel lingkungan proses `docker` (tidak muncul di argumen), tanpa persistensi seperti compose.
//! Tanpa Docker, test GAGAL (tidak dilewati). Kontainer dihapus saat dibuang.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, dead_code)]

use foundation::Sensitive;
use std::path::Path;
use std::process::{Command, Output};
use std::time::Duration;
use valkey::{Valkey, ValkeyConfig};

fn run(cmd: &mut Command) -> Output {
    cmd.output()
        .unwrap_or_else(|e| panic!("docker tidak dapat dijalankan (wajib untuk test Valkey): {e}"))
}

fn random_hex() -> String {
    let mut b = [0u8; 16];
    crypto::fill_random(&mut b).unwrap();
    b.iter().map(|x| format!("{x:02x}")).collect()
}

/// Image Valkey (tag@digest) dari compose lokal.
pub fn image_from_compose() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../infra/local/compose.yaml");
    let text = std::fs::read_to_string(path).expect("infra/local/compose.yaml terbaca");
    text.lines()
        .map(str::trim)
        .find_map(|l| l.strip_prefix("image: valkey/valkey:"))
        .map(|rest| format!("valkey/valkey:{rest}"))
        .expect("image valkey di compose lokal")
}

pub struct ValkeyContainer {
    id: String,
    pub port: u16,
    password: String,
}

impl ValkeyContainer {
    pub fn start() -> Self {
        let password = random_hex();
        let out = run(Command::new("docker")
            .env("VALKEY_PASSWORD", &password)
            .args([
                "run",
                "-d",
                "--rm",
                "-p",
                "127.0.0.1::6379",
                "-e",
                "VALKEY_PASSWORD",
                &image_from_compose(),
                "sh",
                "-c",
                "exec valkey-server --requirepass \"$VALKEY_PASSWORD\" --save '' --appendonly no",
            ]));
        assert!(
            out.status.success(),
            "docker run Valkey gagal: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        let id = String::from_utf8_lossy(&out.stdout).trim().to_string();
        let out = run(Command::new("docker").args(["port", &id, "6379/tcp"]));
        let port = String::from_utf8_lossy(&out.stdout)
            .lines()
            .find_map(|l| l.strip_prefix("127.0.0.1:"))
            .and_then(|p| p.trim().parse().ok())
            .expect("port Valkey di 127.0.0.1");
        let c = Self { id, port, password };
        c.wait_ready();
        c
    }

    fn wait_ready(&self) {
        for _ in 0..100 {
            if self.cli(&["PING"]).trim() == "PONG" {
                return;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        panic!("Valkey tidak siap dalam 10 detik");
    }

    /// `valkey-cli` di dalam kontainer; password dibaca dari lingkungan kontainer (pola healthcheck
    /// compose), tidak lewat argumen proses di host.
    pub fn cli(&self, args: &[&str]) -> String {
        let out = run(Command::new("docker")
            .args([
                "exec",
                &self.id,
                "sh",
                "-c",
                "exec valkey-cli -a \"$VALKEY_PASSWORD\" --no-auth-warning \"$@\"",
                "sh",
            ])
            .args(args));
        String::from_utf8_lossy(&out.stdout).into_owned()
    }

    pub fn url(&self) -> Sensitive<String> {
        Sensitive::new(format!(
            "redis://:{}@127.0.0.1:{}/",
            self.password, self.port
        ))
    }

    pub fn url_with_password(&self, password: &str) -> Sensitive<String> {
        Sensitive::new(format!("redis://:{password}@127.0.0.1:{}/", self.port))
    }

    pub fn client(&self, op_timeout: Duration) -> Valkey {
        Valkey::connect_lazy(&ValkeyConfig {
            url: self.url(),
            op_timeout,
        })
        .unwrap()
    }

    /// Membekukan proses Valkey (koneksi tetap terbuka, tidak ada balasan).
    pub fn pause(&self) {
        assert!(
            run(Command::new("docker").args(["pause", &self.id]))
                .status
                .success()
        );
    }

    pub fn unpause(&self) {
        assert!(
            run(Command::new("docker").args(["unpause", &self.id]))
                .status
                .success()
        );
    }

    /// Menghapus seluruh isi (setara restart tanpa persistensi).
    pub fn flush_all(&self) {
        assert_eq!(self.cli(&["FLUSHALL"]).trim(), "OK");
    }

    /// Memutus semua koneksi klien.
    pub fn kill_clients(&self) {
        let _ = self.cli(&["CLIENT", "KILL", "TYPE", "normal"]);
    }
}

impl Drop for ValkeyContainer {
    fn drop(&mut self) {
        let _ = Command::new("docker").args(["rm", "-f", &self.id]).output();
    }
}

/// URL ke port yang tidak mendengarkan (Valkey mati).
pub fn dead_url() -> Sensitive<String> {
    let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = l.local_addr().unwrap().port();
    drop(l);
    Sensitive::new(format!("redis://:x@127.0.0.1:{port}/"))
}
