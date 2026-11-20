//! Perlengkapan test relay: CA uji yang dibangkitkan saat test (openssl CLI), tiruan SES HTTPS yang
//! memvalidasi SigV4 secara independen, dan klien SMTP+STARTTLS minimal.
//!
//! Tidak ada panggilan ke AWS; semua alamat berdomain `.invalid`, kredensial adalah contoh fiktif AWS.
//! openssl wajib ada di PATH: bila tidak, test GAGAL (tidak dilewati).
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    dead_code
)]

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use crypto::{HmacSha256Key, sha256};
use email_relay::address::Mailbox;
use email_relay::{Relay, RelaySettings, SesSender};
use foundation::{FixedClock, Sensitive};
use rustls_pki_types::pem::PemObject;
use rustls_pki_types::{CertificateDer, ServerName};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

pub const ACCESS_KEY_ID: &str = "AKIAIOSFODNN7EXAMPLE";
pub const SECRET: &str = "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY";
pub const FROM: &str = "noreply@uji.invalid";
pub const RCPT: &str = "pasien.uji@uji.invalid";
pub const CONFIG_SET: &str = "rs-keycloak-uji";

// ------------------------------------------------------------------------------------------- PKI uji

pub struct TestPki {
    pub dir: PathBuf,
    pub ca: PathBuf,
    pub cert: PathBuf,
    pub key: PathBuf,
}

fn openssl(dir: &Path, args: &[&str]) {
    let out = Command::new("openssl")
        .current_dir(dir)
        .args(args)
        .output()
        .unwrap_or_else(|e| panic!("openssl tidak dapat dijalankan (wajib untuk test TLS): {e}"));
    assert!(
        out.status.success(),
        "openssl {args:?} gagal: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

impl TestPki {
    /// CA uji + sertifikat server (SAN `email-relay`, `localhost`, `127.0.0.1`), kunci EC P-256, berlaku 1 hari.
    pub fn generate() -> Self {
        let mut rnd = [0u8; 8];
        crypto::fill_random(&mut rnd).unwrap();
        let tag: String = rnd.iter().map(|b| format!("{b:02x}")).collect();
        let dir = std::env::temp_dir().join(format!("rs-email-relay-uji-{tag}"));
        std::fs::create_dir_all(&dir).unwrap();
        openssl(
            &dir,
            &[
                "req",
                "-x509",
                "-newkey",
                "ec",
                "-pkeyopt",
                "ec_paramgen_curve:P-256",
                "-nodes",
                "-keyout",
                "ca.key",
                "-out",
                "ca.pem",
                "-days",
                "1",
                "-subj",
                "/CN=rs-uji-ca",
                "-addext",
                "basicConstraints=critical,CA:TRUE",
                "-addext",
                "keyUsage=critical,keyCertSign,cRLSign",
            ],
        );
        openssl(
            &dir,
            &[
                "req",
                "-newkey",
                "ec",
                "-pkeyopt",
                "ec_paramgen_curve:P-256",
                "-nodes",
                "-keyout",
                "server.key",
                "-out",
                "server.csr",
                "-subj",
                "/CN=email-relay",
            ],
        );
        std::fs::write(
            dir.join("ext.cnf"),
            "subjectAltName=DNS:email-relay,DNS:localhost,IP:127.0.0.1\n\
             basicConstraints=critical,CA:FALSE\n\
             keyUsage=critical,digitalSignature\n\
             extendedKeyUsage=serverAuth\n",
        )
        .unwrap();
        openssl(
            &dir,
            &[
                "x509",
                "-req",
                "-in",
                "server.csr",
                "-CA",
                "ca.pem",
                "-CAkey",
                "ca.key",
                "-CAcreateserial",
                "-out",
                "server.pem",
                "-days",
                "1",
                "-extfile",
                "ext.cnf",
            ],
        );
        Self {
            ca: dir.join("ca.pem"),
            cert: dir.join("server.pem"),
            key: dir.join("server.key"),
            dir,
        }
    }

    pub fn client_config(&self) -> Arc<rustls::ClientConfig> {
        let mut roots = rustls::RootCertStore::empty();
        for c in CertificateDer::pem_file_iter(&self.ca).unwrap() {
            roots.add(c.unwrap()).unwrap();
        }
        Arc::new(
            rustls::ClientConfig::builder_with_provider(crypto::tls_crypto_provider())
                .with_safe_default_protocol_versions()
                .unwrap()
                .with_root_certificates(roots)
                .with_no_client_auth(),
        )
    }
}

impl Drop for TestPki {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

// ------------------------------------------------------------------------------------------- tiruan SES

/// Satu permintaan yang diterima tiruan SES (sudah lolos validasi SigV4).
#[derive(Debug, Clone)]
pub struct SesRequest {
    pub json: serde_json::Value,
    pub raw: Vec<u8>,
}

#[derive(Clone)]
pub struct MockSes {
    pub port: u16,
    pub received: Arc<Mutex<Vec<SesRequest>>>,
    /// Permintaan dengan tanda tangan tidak valid (dijawab 403).
    pub bad_signatures: Arc<Mutex<u32>>,
    /// (status, jeda) jawaban berikutnya; bawaan 200 tanpa jeda.
    pub behavior: Arc<Mutex<(u16, Duration)>>,
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

/// Verifikasi SigV4 independen dari implementasi crate `ses` (string kanonik disusun ulang dari header yang
/// benar-benar diterima, untuk header yang disebut di `SignedHeaders`).
fn verify_sigv4(method: &str, path: &str, headers: &[(String, String)], body: &[u8]) -> bool {
    let get = |n: &str| headers.iter().find(|(k, _)| k == n).map(|(_, v)| v.clone());
    let Some(auth) = get("authorization") else {
        return false;
    };
    let Some(date) = get("x-amz-date") else {
        return false;
    };
    let Some(rest) = auth.strip_prefix("AWS4-HMAC-SHA256 ") else {
        return false;
    };
    let mut cred = None;
    let mut signed = None;
    let mut sig = None;
    for part in rest.split(", ") {
        if let Some(v) = part.strip_prefix("Credential=") {
            cred = Some(v.to_string());
        } else if let Some(v) = part.strip_prefix("SignedHeaders=") {
            signed = Some(v.to_string());
        } else if let Some(v) = part.strip_prefix("Signature=") {
            sig = Some(v.to_string());
        }
    }
    let (Some(cred), Some(signed), Some(sig)) = (cred, signed, sig) else {
        return false;
    };
    let scope_parts: Vec<&str> = cred.split('/').collect();
    if scope_parts.len() != 5
        || scope_parts[0] != ACCESS_KEY_ID
        || scope_parts[2] != "ap-southeast-3"
        || scope_parts[3] != "ses"
        || scope_parts[4] != "aws4_request"
        || !date.starts_with(scope_parts[1])
    {
        return false;
    }
    for required in ["host", "x-amz-date", "content-type"] {
        if !signed.split(';').any(|h| h == required) {
            return false;
        }
    }
    let mut canon_headers = String::new();
    for h in signed.split(';') {
        let Some(v) = get(h) else { return false };
        canon_headers.push_str(&format!("{h}:{}\n", v.trim()));
    }
    let creq = format!(
        "{method}\n{path}\n\n{canon_headers}\n{signed}\n{}",
        hex(&sha256(body))
    );
    let scope = format!("{}/ap-southeast-3/ses/aws4_request", scope_parts[1]);
    let sts = format!(
        "AWS4-HMAC-SHA256\n{date}\n{scope}\n{}",
        hex(&sha256(creq.as_bytes()))
    );
    let k = |key: &[u8], data: &str| HmacSha256Key::new(key).unwrap().sign(data.as_bytes());
    let k_date = k(format!("AWS4{SECRET}").as_bytes(), scope_parts[1]);
    let k_region = k(&k_date, "ap-southeast-3");
    let k_service = k(&k_region, "ses");
    let k_signing = k(&k_service, "aws4_request");
    hex(&k(&k_signing, &sts)) == sig
}

impl MockSes {
    pub async fn start(pki: &TestPki) -> Self {
        let acceptor = email_relay::server_tls(&pki.cert, &pki.key).unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let mock = Self {
            port,
            received: Arc::default(),
            bad_signatures: Arc::default(),
            behavior: Arc::new(Mutex::new((200, Duration::ZERO))),
        };
        let m = mock.clone();
        tokio::spawn(async move {
            loop {
                let (tcp, _) = listener.accept().await.unwrap();
                let acceptor = acceptor.clone();
                let m = m.clone();
                tokio::spawn(async move {
                    let Ok(tls) = acceptor.accept(tcp).await else {
                        return;
                    };
                    m.serve_one(tls).await;
                });
            }
        });
        mock
    }

    pub fn set(&self, status: u16, delay: Duration) {
        *self.behavior.lock().unwrap() = (status, delay);
    }

    pub fn requests(&self) -> Vec<SesRequest> {
        self.received.lock().unwrap().clone()
    }

    async fn serve_one<S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin>(&self, s: S) {
        let mut r = BufReader::new(s);
        let mut start = String::new();
        if r.read_line(&mut start).await.unwrap_or(0) == 0 {
            return;
        }
        let mut headers = Vec::new();
        loop {
            let mut l = String::new();
            r.read_line(&mut l).await.unwrap();
            let l = l.trim_end_matches("\r\n");
            if l.is_empty() {
                break;
            }
            let (k, v) = l.split_once(':').unwrap();
            headers.push((k.to_ascii_lowercase(), v.trim().to_string()));
        }
        let len: usize = headers
            .iter()
            .find(|(k, _)| k == "content-length")
            .map(|(_, v)| v.parse().unwrap())
            .unwrap_or(0);
        let mut body = vec![0u8; len];
        r.read_exact(&mut body).await.unwrap();
        let mut parts = start.split_whitespace();
        let method = parts.next().unwrap_or("").to_string();
        let path = parts.next().unwrap_or("").to_string();
        let (status, delay) = *self.behavior.lock().unwrap();
        let (code, resp) = if method != "POST" || path != "/v2/email/outbound-emails" {
            (404, "{}".to_string())
        } else if !verify_sigv4(&method, &path, &headers, &body) {
            *self.bad_signatures.lock().unwrap() += 1;
            (403, r#"{"message":"signature"}"#.to_string())
        } else {
            let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
            let raw = STANDARD
                .decode(json["Content"]["Raw"]["Data"].as_str().unwrap())
                .unwrap();
            self.received.lock().unwrap().push(SesRequest { json, raw });
            tokio::time::sleep(delay).await;
            if status == 200 {
                (200, r#"{"MessageId":"0100018f-uji-0001"}"#.to_string())
            } else {
                // Isi galat tiruan sengaja menggemakan alamat: relay tidak boleh meneruskannya.
                (status, format!(r#"{{"message":"ditolak untuk {RCPT}"}}"#))
            }
        };
        let s = r.get_mut();
        let out = format!(
            "HTTP/1.1 {code} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{resp}",
            resp.len()
        );
        let _ = s.write_all(out.as_bytes()).await;
        let _ = s.shutdown().await;
    }
}

// ------------------------------------------------------------------------------------------- relay uji

pub fn settings() -> RelaySettings {
    RelaySettings {
        hostname: "email-relay".to_string(),
        allowed_from: vec![Mailbox::parse(FROM).unwrap()],
        max_message_bytes: 8 * 1024,
        global_per_minute: 100,
        per_recipient_per_hour: 100,
        max_connections: 8,
        idle_timeout: Duration::from_secs(5),
        session_timeout: Duration::from_secs(30),
    }
}

pub fn ses_client(pki: &TestPki, ses: &MockSes, secret: &str, timeout: Duration) -> ses::SesClient {
    ses::SesClient::new(&ses::SesConfig {
        local_endpoint: Some(format!("https://127.0.0.1:{}", ses.port)),
        ca_file: pki.ca.clone(),
        timeout,
        configuration_set: CONFIG_SET.to_string(),
        credentials: ses::AwsCredentials {
            access_key_id: ACCESS_KEY_ID.to_string(),
            secret_access_key: Sensitive::new(secret.to_string()),
        },
    })
    .unwrap()
}

pub struct Harness {
    pub pki: TestPki,
    pub ses: MockSes,
    pub port: u16,
}

pub async fn start_with(settings: RelaySettings, secret: &str, ses_timeout: Duration) -> Harness {
    let pki = TestPki::generate();
    let ses = MockSes::start(&pki).await;
    let clock = Arc::new(FixedClock::new(
        time::macros::datetime!(2026-11-20 03:04:05 UTC),
    ));
    let sender = SesSender::new(ses_client(&pki, &ses, secret, ses_timeout), clock);
    let tls = email_relay::server_tls(&pki.cert, &pki.key).unwrap();
    let relay = Relay::new(settings, tls, sender).unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(relay.serve(listener));
    Harness { pki, ses, port }
}

pub async fn start() -> Harness {
    start_with(settings(), SECRET, Duration::from_secs(5)).await
}

// ------------------------------------------------------------------------------------------- klien SMTP

/// Klien SMTP minimal: baris dikirim apa adanya dengan CRLF; balasan multi-baris digabung.
pub struct Client {
    plain: Option<BufReader<TcpStream>>,
    tls: Option<BufReader<tokio_rustls::client::TlsStream<TcpStream>>>,
}

impl Client {
    pub async fn connect(port: u16) -> (Self, String) {
        let tcp = TcpStream::connect(("127.0.0.1", port)).await.unwrap();
        let mut c = Self {
            plain: Some(BufReader::new(tcp)),
            tls: None,
        };
        let greeting = c.read_reply().await;
        (c, greeting)
    }

    pub async fn send_raw(&mut self, data: &[u8]) {
        if let Some(t) = self.tls.as_mut() {
            t.get_mut().write_all(data).await.unwrap();
        } else {
            self.plain
                .as_mut()
                .unwrap()
                .get_mut()
                .write_all(data)
                .await
                .unwrap();
        }
    }

    /// Satu balasan lengkap (baris `NNN-` dilanjutkan sampai `NNN `); string kosong bila koneksi ditutup.
    pub async fn read_reply(&mut self) -> String {
        let mut all = String::new();
        loop {
            let mut l = String::new();
            let n = match (self.tls.as_mut(), self.plain.as_mut()) {
                (Some(t), _) => {
                    tokio::time::timeout(Duration::from_secs(15), t.read_line(&mut l)).await
                }
                (None, Some(p)) => {
                    tokio::time::timeout(Duration::from_secs(15), p.read_line(&mut l)).await
                }
                _ => panic!("klien tanpa aliran"),
            }
            .expect("balasan relay dalam 15 detik")
            .unwrap_or(0);
            if n == 0 {
                return all;
            }
            all.push_str(&l);
            if l.len() >= 4 && l.as_bytes()[3] == b' ' {
                return all;
            }
        }
    }

    pub async fn cmd(&mut self, line: &str) -> String {
        self.send_raw(format!("{line}\r\n").as_bytes()).await;
        self.read_reply().await
    }

    /// Jabat tangan TLS (nama `email-relay`) setelah balasan 220 STARTTLS.
    pub async fn upgrade(&mut self, pki: &TestPki) -> std::io::Result<()> {
        let tcp = self.plain.take().unwrap().into_inner();
        let connector = tokio_rustls::TlsConnector::from(pki.client_config());
        let tls = connector
            .connect(ServerName::try_from("email-relay").unwrap(), tcp)
            .await?;
        self.tls = Some(BufReader::new(tls));
        Ok(())
    }

    /// EHLO + STARTTLS + jabat tangan TLS + EHLO lagi.
    pub async fn starttls(&mut self, pki: &TestPki) -> String {
        assert!(self.cmd("EHLO uji.invalid").await.contains("STARTTLS"));
        let r = self.cmd("STARTTLS").await;
        assert!(r.starts_with("220 "), "{r}");
        self.upgrade(pki).await.unwrap();
        self.cmd("EHLO uji.invalid").await
    }

    /// MAIL/RCPT/DATA lengkap; mengembalikan balasan akhir DATA.
    pub async fn send_message(&mut self, from: &str, rcpt: &str, message: &str) -> String {
        let r = self.cmd(&format!("MAIL FROM:<{from}>")).await;
        if !r.starts_with("250") {
            return r;
        }
        let r = self.cmd(&format!("RCPT TO:<{rcpt}>")).await;
        if !r.starts_with("250") {
            return r;
        }
        let r = self.cmd("DATA").await;
        assert!(r.starts_with("354"), "{r}");
        self.send_raw(message.as_bytes()).await;
        self.send_raw(b".\r\n").await;
        self.read_reply().await
    }
}

/// Pesan serupa Keycloak (multipart, tautan token aksi di badan).
pub fn keycloak_like_message(subject: &str) -> String {
    format!(
        "Date: Fri, 20 Nov 2026 10:04:05 +0700\r\n\
         From: Ruang Sehat <{FROM}>\r\n\
         To: {RCPT}\r\n\
         Message-ID: <uji-1@uji.invalid>\r\n\
         Subject: {subject}\r\n\
         MIME-Version: 1.0\r\n\
         Content-Type: text/plain; charset=UTF-8\r\n\
         Content-Transfer-Encoding: 7bit\r\n\
         \r\n\
         Buka http://127.0.0.1/realms/pasien/login-actions/action-token?key=token-aksi-sintetis-uji\r\n\
         ..baris diawali titik\r\n"
    )
}
