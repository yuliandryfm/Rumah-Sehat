//! Klien HTTP minimal ke OpenBao (API `/v1/...`, header `X-Vault-Token`) [SRC-485].
//!
//! Batasan saat ini (gagal-tertutup): hanya `http://` ke alamat IP loopback (OpenBao mode dev di
//! Compose lokal, `infra/local/compose.yaml`; dev server tanpa TLS [SRC-486]). TLS ke OpenBao staging
//! dan production belum diterapkan (TLS antar-host: docs/05-stack.md §2.1, phase plan EN-15/EN-16);
//! alamat `https://` atau non-loopback ditolak dengan [`CryptoError::InvalidConfig`].
//!
//! Token dan isi respons (yang dapat memuat DEK plaintext base64) tidak pernah ditulis ke galat atau
//! log; header token ditandai sensitif.

use crate::error::CryptoError;
use bytes::Bytes;
use foundation::Sensitive;
use http::{HeaderValue, Method, Request, header};
use http_body_util::{BodyExt, Full, Limited};
use hyper_util::rt::TokioIo;
use std::fmt;
use std::net::{IpAddr, SocketAddr};
use std::time::Duration;
use tokio::net::TcpStream;
use zeroize::Zeroizing;

/// Batas ukuran respons OpenBao yang dibaca (byte).
const MAX_RESPONSE_BYTES: usize = 64 * 1024;

/// Konfigurasi koneksi OpenBao. Token bertipe [`Sensitive`].
pub struct OpenBaoConfig {
    /// Alamat dasar, misalnya `http://127.0.0.1:18200` (lihat batasan di dokumentasi modul).
    pub address: String,
    /// Token OpenBao proses ini (identitas proses, KEPUTUSAN-79).
    pub token: Sensitive<String>,
    /// Batas waktu satu permintaan (koneksi + respons).
    pub timeout: Duration,
}

impl fmt::Debug for OpenBaoConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OpenBaoConfig")
            .field("address", &self.address)
            .field("token", &self.token)
            .field("timeout", &self.timeout)
            .finish()
    }
}

/// Klien OpenBao yang sudah divalidasi.
pub(crate) struct OpenBaoClient {
    addr: SocketAddr,
    host_header: HeaderValue,
    token: HeaderValue,
    timeout: Duration,
}

impl fmt::Debug for OpenBaoClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OpenBaoClient")
            .field("addr", &self.addr)
            .field("token", &"[REDACTED]")
            .field("timeout", &self.timeout)
            .finish()
    }
}

/// Mengurai `http://<ip-loopback>:<port>` (tanpa path, tanpa userinfo).
fn parse_address(address: &str) -> Result<SocketAddr, CryptoError> {
    let rest = address
        .strip_prefix("http://")
        .ok_or(CryptoError::InvalidConfig)?;
    let rest = rest.strip_suffix('/').unwrap_or(rest);
    let addr: SocketAddr = rest.parse().map_err(|_| CryptoError::InvalidConfig)?;
    let loopback = match addr.ip() {
        IpAddr::V4(ip) => ip.is_loopback(),
        IpAddr::V6(ip) => ip.is_loopback(),
    };
    if !loopback || addr.port() == 0 {
        return Err(CryptoError::InvalidConfig);
    }
    Ok(addr)
}

/// Segmen path OpenBao yang diizinkan (nama mount, nama kunci, path rahasia): `[a-z0-9_-]`,
/// dipisah `/` bila `allow_slash`. Mencegah penyisipan `..`, query, atau karakter kontrol.
pub(crate) fn valid_path(value: &str, allow_slash: bool) -> bool {
    !value.is_empty()
        && value.split('/').all(|seg| {
            !seg.is_empty()
                && seg
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_' || b == b'-')
        })
        && (allow_slash || !value.contains('/'))
}

impl OpenBaoClient {
    pub(crate) fn new(config: &OpenBaoConfig) -> Result<Self, CryptoError> {
        let addr = parse_address(&config.address)?;
        if config.timeout.is_zero() {
            return Err(CryptoError::InvalidConfig);
        }
        let host_header =
            HeaderValue::from_str(&addr.to_string()).map_err(|_| CryptoError::InvalidConfig)?;
        let raw = config.token.expose();
        if raw.is_empty() {
            return Err(CryptoError::InvalidConfig);
        }
        let mut token = HeaderValue::from_str(raw).map_err(|_| CryptoError::InvalidConfig)?;
        token.set_sensitive(true);
        Ok(Self {
            addr,
            host_header,
            token,
            timeout: config.timeout,
        })
    }

    /// Mengirim satu permintaan dan mengembalikan isi respons 2xx. Path harus sudah divalidasi pemanggil.
    pub(crate) async fn call(
        &self,
        method: Method,
        path: &str,
        body: Option<Vec<u8>>,
    ) -> Result<Zeroizing<Vec<u8>>, CryptoError> {
        match tokio::time::timeout(self.timeout, self.call_inner(method, path, body)).await {
            Ok(result) => result,
            Err(_) => Err(CryptoError::KmsTimeout),
        }
    }

    async fn call_inner(
        &self,
        method: Method,
        path: &str,
        body: Option<Vec<u8>>,
    ) -> Result<Zeroizing<Vec<u8>>, CryptoError> {
        let stream = TcpStream::connect(self.addr)
            .await
            .map_err(|_| CryptoError::KmsUnavailable)?;
        let (mut sender, conn) = hyper::client::conn::http1::handshake(TokioIo::new(stream))
            .await
            .map_err(|_| CryptoError::KmsUnavailable)?;
        let driver = tokio::spawn(conn);

        let mut builder = Request::builder()
            .method(method)
            .uri(path)
            .header(header::HOST, self.host_header.clone())
            .header("X-Vault-Token", self.token.clone())
            .header(header::CONNECTION, "close");
        if body.is_some() {
            builder = builder.header(header::CONTENT_TYPE, "application/json");
        }
        let request = builder
            .body(Full::new(Bytes::from(body.unwrap_or_default())))
            .map_err(|_| CryptoError::InvalidConfig)?;

        let result = async {
            let response = sender
                .send_request(request)
                .await
                .map_err(|_| CryptoError::KmsUnavailable)?;
            let status = response.status();
            let collected = Limited::new(response.into_body(), MAX_RESPONSE_BYTES)
                .collect()
                .await
                .map_err(|_| CryptoError::KmsInvalidResponse)?;
            // Isi disalin ke buffer yang dinolkan saat dibuang; buffer `Bytes` milik hyper tidak dapat dinolkan.
            let bytes = Zeroizing::new(collected.to_bytes().to_vec());
            if status.is_success() {
                Ok(bytes)
            } else {
                Err(CryptoError::KmsRejected {
                    status: status.as_u16(),
                })
            }
        }
        .await;
        driver.abort();
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg(address: &str) -> OpenBaoConfig {
        OpenBaoConfig {
            address: address.to_string(),
            token: Sensitive::new("uji-token-sintetis".to_string()),
            timeout: Duration::from_secs(2),
        }
    }

    #[test]
    fn loopback_http_address_is_accepted() {
        assert!(OpenBaoClient::new(&cfg("http://127.0.0.1:18200")).is_ok());
        assert!(OpenBaoClient::new(&cfg("http://127.0.0.1:18200/")).is_ok());
        assert!(OpenBaoClient::new(&cfg("http://[::1]:8200")).is_ok());
    }

    #[test]
    fn non_loopback_tls_or_malformed_address_is_rejected() {
        for a in [
            "https://127.0.0.1:8200",
            "http://10.0.0.5:8200",
            "http://0.0.0.0:8200",
            "http://openbao:8200",
            "http://localhost:8200",
            "http://127.0.0.1",
            "http://127.0.0.1:0",
            "http://127.0.0.1:8200/v1",
            "http://user@127.0.0.1:8200",
            "127.0.0.1:8200",
            "",
        ] {
            assert_eq!(
                OpenBaoClient::new(&cfg(a)).err(),
                Some(CryptoError::InvalidConfig),
                "{a}"
            );
        }
    }

    #[test]
    fn empty_or_invalid_token_and_zero_timeout_are_rejected() {
        let mut c = cfg("http://127.0.0.1:8200");
        c.token = Sensitive::new(String::new());
        assert_eq!(
            OpenBaoClient::new(&c).err(),
            Some(CryptoError::InvalidConfig)
        );
        c.token = Sensitive::new("baris\nbaru".to_string());
        assert_eq!(
            OpenBaoClient::new(&c).err(),
            Some(CryptoError::InvalidConfig)
        );
        let mut c = cfg("http://127.0.0.1:8200");
        c.timeout = Duration::ZERO;
        assert_eq!(
            OpenBaoClient::new(&c).err(),
            Some(CryptoError::InvalidConfig)
        );
    }

    #[test]
    fn debug_does_not_leak_token() {
        let c = cfg("http://127.0.0.1:8200");
        let client = OpenBaoClient::new(&c).unwrap();
        let out = format!("{c:?} {client:?}");
        assert!(!out.contains("uji-token-sintetis"));
        assert!(out.contains("REDACTED"));
    }

    #[test]
    fn path_validation_rejects_traversal_and_query() {
        assert!(valid_path("transit", false));
        assert!(valid_path("rs-dek-kek", false));
        assert!(valid_path("ruang-sehat/blind-index/nik", true));
        for bad in ["", "a/b", "..", "a?b", "A", "a b", "a%2f", "a\n"] {
            assert!(!valid_path(bad, false), "{bad:?}");
        }
        for bad in ["/a", "a/", "a//b", "a/../b", "a/.b"] {
            assert!(!valid_path(bad, true), "{bad:?}");
        }
    }

    #[tokio::test]
    async fn unreachable_server_is_reported_as_unavailable() {
        // Port loopback yang (hampir pasti) tidak didengarkan: bind lalu lepas.
        let port = std::net::TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap()
            .port();
        let client = OpenBaoClient::new(&cfg(&format!("http://127.0.0.1:{port}"))).unwrap();
        let err = client.call(Method::GET, "/v1/sys/health", None).await.err();
        assert_eq!(err, Some(CryptoError::KmsUnavailable));
    }
}
