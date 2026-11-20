//! JWKS realm: sumber, penguraian, dan cache (ADR-011 butir 8, KEPUTUSAN-74 "JWKS realm di-cache").
//!
//! Aturan kunci (RFC 8725 §3.1 dan §3.8, [SRC-79]):
//! - Hanya kunci dengan `use` kosong atau `sig`, `kid` ada, dan `alg` ada serta termasuk daftar izin
//!   realm. Keycloak 26.8.2 menerbitkan juga kunci enkripsi (`use: enc`, `alg: RSA-OAEP`) di JWKS yang
//!   sama; kunci itu dilewati (SRC-531).
//! - Jenis kunci harus sesuai keluarga algoritma (`RSA` untuk RS*/PS*, `EC` dengan kurva yang cocok
//!   untuk ES256/ES384).
//! - `kid` ganda dalam satu JWKS dianggap ambigu: semua kunci dengan `kid` itu dibuang.
//! - JWKS tanpa kunci yang dapat dipakai diperlakukan sebagai gagal ambil.
//!
//! Cache gagal-tertutup: setelah TTL lewat, kunci lama tidak dipakai lagi; bila pengambilan ulang gagal,
//! token ditolak ([`crate::AuthError::KeysUnavailable`]). `kid` tidak dikenal memicu pengambilan ulang
//! (rotasi kunci), dibatasi satu kali per interval minimum supaya `kid` acak tidak membanjiri Keycloak.

use crate::error::AuthnConfigError;
use bytes::Bytes;
use foundation::Clock;
use http::{HeaderValue, Method, Request, header};
use http_body_util::{BodyExt, Empty, Limited};
use hyper_util::rt::TokioIo;
use jsonwebtoken::{Algorithm, AlgorithmFamily, DecodingKey};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::net::{IpAddr, SocketAddr};
use std::pin::Pin;
use std::str::FromStr;
use std::sync::Arc;
use time::{Duration, OffsetDateTime};
use tokio::net::TcpStream;
use tokio::sync::{Mutex, RwLock};

/// Batas ukuran dokumen JWKS yang dibaca (byte).
pub const MAX_JWKS_BYTES: usize = 64 * 1024;

/// Batas panjang `kid` yang diterima.
const MAX_KID_LEN: usize = 256;

/// Kegagalan mengambil JWKS. Tidak memuat isi respons.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JwksFetchError {
    /// Koneksi gagal.
    Connect,
    /// Batas waktu terlewati.
    Timeout,
    /// Status HTTP bukan 200.
    Status(u16),
    /// Respons melebihi [`MAX_JWKS_BYTES`] atau terputus.
    Body,
}

/// Future hasil [`JwksSource::fetch`].
pub type JwksFuture<'a> =
    Pin<Box<dyn Future<Output = Result<Vec<u8>, JwksFetchError>> + Send + 'a>>;

/// Sumber dokumen JWKS satu realm. Implementasi produksi: [`HttpJwksSource`]; test memakai sumber
/// dalam memori atau server HTTP lokal.
pub trait JwksSource: Send + Sync {
    /// Mengambil dokumen JWKS mentah.
    fn fetch(&self) -> JwksFuture<'_>;
}

/// Klien HTTP JWKS minimal di atas hyper (pola klien OpenBao di `crates/crypto`).
///
/// Batasan saat ini (gagal-tertutup): hanya `http://<ip-loopback>:<port>/<path>`. TLS ke Keycloak lintas
/// host (docs/05-stack.md §2.1) belum diterapkan; alamat `https://`, nama host, atau IP non-loopback
/// ditolak saat konfigurasi. Alamat JWKS dikonfigurasi terpisah dari issuer dan **tidak pernah**
/// diturunkan dari isi token (`iss`, `jku`, `x5u` diabaikan).
#[derive(Debug, Clone)]
pub struct HttpJwksSource {
    addr: SocketAddr,
    host_header: HeaderValue,
    path: String,
    timeout: std::time::Duration,
}

impl HttpJwksSource {
    /// `url` misalnya `http://127.0.0.1:18080/realms/pasien/protocol/openid-connect/certs`.
    pub fn new(url: &str, timeout: std::time::Duration) -> Result<Self, AuthnConfigError> {
        let rest = url
            .strip_prefix("http://")
            .ok_or(AuthnConfigError::InvalidJwksAddress)?;
        let (authority, path) = rest
            .split_once('/')
            .ok_or(AuthnConfigError::InvalidJwksAddress)?;
        let addr: SocketAddr = authority
            .parse()
            .map_err(|_| AuthnConfigError::InvalidJwksAddress)?;
        let loopback = match addr.ip() {
            IpAddr::V4(ip) => ip.is_loopback(),
            IpAddr::V6(ip) => ip.is_loopback(),
        };
        if !loopback || addr.port() == 0 || !valid_path(path) {
            return Err(AuthnConfigError::InvalidJwksAddress);
        }
        if timeout.is_zero() || timeout > std::time::Duration::from_secs(30) {
            return Err(AuthnConfigError::InvalidDuration("jwks_fetch_timeout"));
        }
        let host_header = HeaderValue::from_str(&addr.to_string())
            .map_err(|_| AuthnConfigError::InvalidJwksAddress)?;
        Ok(Self {
            addr,
            host_header,
            path: format!("/{path}"),
            timeout,
        })
    }

    async fn fetch_inner(&self) -> Result<Vec<u8>, JwksFetchError> {
        let stream = TcpStream::connect(self.addr)
            .await
            .map_err(|_| JwksFetchError::Connect)?;
        let (mut sender, conn) = hyper::client::conn::http1::handshake(TokioIo::new(stream))
            .await
            .map_err(|_| JwksFetchError::Connect)?;
        let driver = tokio::spawn(conn);
        let request = Request::builder()
            .method(Method::GET)
            .uri(self.path.as_str())
            .header(header::HOST, self.host_header.clone())
            .header(header::ACCEPT, "application/json")
            .header(header::CONNECTION, "close")
            .body(Empty::<Bytes>::new())
            .map_err(|_| JwksFetchError::Connect)?;
        let result = async {
            let response = sender
                .send_request(request)
                .await
                .map_err(|_| JwksFetchError::Connect)?;
            let status = response.status();
            if status != http::StatusCode::OK {
                return Err(JwksFetchError::Status(status.as_u16()));
            }
            let collected = Limited::new(response.into_body(), MAX_JWKS_BYTES)
                .collect()
                .await
                .map_err(|_| JwksFetchError::Body)?;
            Ok(collected.to_bytes().to_vec())
        }
        .await;
        driver.abort();
        result
    }
}

impl JwksSource for HttpJwksSource {
    fn fetch(&self) -> JwksFuture<'_> {
        Box::pin(async move {
            match tokio::time::timeout(self.timeout, self.fetch_inner()).await {
                Ok(r) => r,
                Err(_) => Err(JwksFetchError::Timeout),
            }
        })
    }
}

/// Path JWKS: segmen `[A-Za-z0-9_.-]` dipisah `/`, tanpa segmen kosong, `.`, atau `..`, tanpa query.
fn valid_path(path: &str) -> bool {
    !path.is_empty()
        && path.split('/').all(|seg| {
            !seg.is_empty()
                && seg != "."
                && seg != ".."
                && seg
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-' || b == b'.')
        })
}

/// Satu kunci verifikasi yang terikat ke satu algoritma.
#[derive(Clone)]
pub(crate) struct VerifyKey {
    pub(crate) alg: Algorithm,
    pub(crate) key: DecodingKey,
}

/// Kunci JWKS yang dapat dipakai, per `kid`.
pub(crate) struct KeySet {
    keys: HashMap<String, VerifyKey>,
}

impl KeySet {
    pub(crate) fn get(&self, kid: &str) -> Option<&VerifyKey> {
        self.keys.get(kid)
    }

    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.keys.len()
    }
}

#[derive(Deserialize)]
struct RawJwks {
    keys: Vec<serde_json::Value>,
}

#[derive(Deserialize)]
struct RawJwk {
    kty: String,
    kid: Option<String>,
    alg: Option<String>,
    #[serde(rename = "use")]
    key_use: Option<String>,
    key_ops: Option<Vec<String>>,
    n: Option<String>,
    e: Option<String>,
    crv: Option<String>,
    x: Option<String>,
    y: Option<String>,
}

fn usable_key(value: serde_json::Value, allowed: &[Algorithm]) -> Option<(String, VerifyKey)> {
    let raw: RawJwk = serde_json::from_value(value).ok()?;
    let kid = raw.kid?;
    if kid.is_empty() || kid.len() > MAX_KID_LEN {
        return None;
    }
    if raw.key_use.as_deref().is_some_and(|u| u != "sig") {
        return None;
    }
    if raw
        .key_ops
        .as_ref()
        .is_some_and(|ops| !ops.iter().any(|o| o == "verify"))
    {
        return None;
    }
    let alg = Algorithm::from_str(raw.alg.as_deref()?).ok()?;
    if !allowed.contains(&alg) {
        return None;
    }
    let key = match (alg.family(), raw.kty.as_str()) {
        (AlgorithmFamily::Rsa, "RSA") => {
            DecodingKey::from_rsa_components(raw.n.as_deref()?, raw.e.as_deref()?).ok()?
        }
        (AlgorithmFamily::Ec, "EC") => {
            let expected_curve = match alg {
                Algorithm::ES256 => "P-256",
                Algorithm::ES384 => "P-384",
                _ => return None,
            };
            if raw.crv.as_deref() != Some(expected_curve) {
                return None;
            }
            DecodingKey::from_ec_components(raw.x.as_deref()?, raw.y.as_deref()?).ok()?
        }
        _ => return None,
    };
    Some((kid, VerifyKey { alg, key }))
}

/// Mengurai dokumen JWKS menjadi kunci yang dapat dipakai. `None` bila dokumen tidak dapat diurai
/// atau tidak memuat satu pun kunci yang dapat dipakai.
pub(crate) fn parse_key_set(bytes: &[u8], allowed: &[Algorithm]) -> Option<KeySet> {
    let raw: RawJwks = serde_json::from_slice(bytes).ok()?;
    let mut keys: HashMap<String, VerifyKey> = HashMap::new();
    let mut duplicates: HashSet<String> = HashSet::new();
    for value in raw.keys {
        if let Some((kid, key)) = usable_key(value, allowed) {
            match keys.entry(kid) {
                std::collections::hash_map::Entry::Occupied(e) => {
                    duplicates.insert(e.key().clone());
                }
                std::collections::hash_map::Entry::Vacant(e) => {
                    e.insert(key);
                }
            }
        }
    }
    for kid in duplicates {
        keys.remove(&kid);
    }
    if keys.is_empty() {
        None
    } else {
        Some(KeySet { keys })
    }
}

/// Hasil pencarian kunci di cache.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LookupError {
    /// `kid` tidak ada di JWKS yang segar.
    UnknownKey,
    /// JWKS tidak tersedia (tidak ada cache segar dan pengambilan gagal).
    Unavailable,
}

/// Parameter waktu cache.
#[derive(Debug, Clone, Copy)]
pub(crate) struct CachePolicy {
    pub(crate) ttl: Duration,
    pub(crate) min_refresh_interval: Duration,
}

#[derive(Default)]
struct CacheState {
    keys: Option<Arc<KeySet>>,
    fetched_at: Option<OffsetDateTime>,
    last_attempt: Option<OffsetDateTime>,
    last_failed: bool,
}

impl CacheState {
    fn fresh_keys(&self, now: OffsetDateTime, ttl: Duration) -> Option<Arc<KeySet>> {
        let fetched = self.fetched_at?;
        let until = fetched.checked_add(ttl)?;
        if now >= fetched && now < until {
            self.keys.clone()
        } else {
            None
        }
    }

    fn attempted_recently(&self, now: OffsetDateTime, interval: Duration) -> bool {
        match self.last_attempt.and_then(|t| t.checked_add(interval)) {
            Some(until) => self.last_attempt.is_some_and(|t| now >= t) && now < until,
            None => false,
        }
    }
}

/// Cache JWKS satu realm.
pub(crate) struct JwksCache {
    source: Arc<dyn JwksSource>,
    allowed: Vec<Algorithm>,
    policy: CachePolicy,
    state: RwLock<CacheState>,
    refresh: Mutex<()>,
}

impl JwksCache {
    pub(crate) fn new(
        source: Arc<dyn JwksSource>,
        allowed: Vec<Algorithm>,
        policy: CachePolicy,
    ) -> Self {
        Self {
            source,
            allowed,
            policy,
            state: RwLock::new(CacheState::default()),
            refresh: Mutex::new(()),
        }
    }

    /// Mencari kunci `kid`, mengambil ulang JWKS bila cache kedaluwarsa atau `kid` belum dikenal.
    pub(crate) async fn key(&self, kid: &str, clock: &dyn Clock) -> Result<VerifyKey, LookupError> {
        let now = clock.now();
        {
            let state = self.state.read().await;
            if let Some(keys) = state.fresh_keys(now, self.policy.ttl) {
                if let Some(k) = keys.get(kid) {
                    return Ok(k.clone());
                }
                if state.attempted_recently(now, self.policy.min_refresh_interval) {
                    return Err(LookupError::UnknownKey);
                }
            } else if state.last_failed
                && state.attempted_recently(now, self.policy.min_refresh_interval)
            {
                return Err(LookupError::Unavailable);
            }
        }

        // Satu pengambilan pada satu waktu; permintaan lain menunggu lalu memeriksa ulang.
        let _guard = self.refresh.lock().await;
        let now = clock.now();
        {
            let state = self.state.read().await;
            let fresh = state.fresh_keys(now, self.policy.ttl);
            if let Some(keys) = &fresh
                && let Some(k) = keys.get(kid)
            {
                return Ok(k.clone());
            }
            // Permintaan lain baru saja mengambil: jangan ambil lagi dalam interval minimum.
            if state.attempted_recently(now, self.policy.min_refresh_interval) {
                return Err(if fresh.is_some() {
                    LookupError::UnknownKey
                } else {
                    LookupError::Unavailable
                });
            }
        }

        let fetched = match self.source.fetch().await {
            Ok(bytes) => parse_key_set(&bytes, &self.allowed),
            Err(_) => None,
        };
        let mut state = self.state.write().await;
        state.last_attempt = Some(now);
        match fetched {
            Some(set) => {
                let set = Arc::new(set);
                state.keys = Some(Arc::clone(&set));
                state.fetched_at = Some(now);
                state.last_failed = false;
                set.get(kid).cloned().ok_or(LookupError::UnknownKey)
            }
            None => {
                state.last_failed = true;
                // Kunci lama yang masih segar tetap berlaku (pengambilan untuk `kid` baru gagal);
                // kunci yang kedaluwarsa tidak pernah dipakai lagi.
                if state.fresh_keys(now, self.policy.ttl).is_some() {
                    Err(LookupError::UnknownKey)
                } else {
                    state.keys = None;
                    state.fetched_at = None;
                    Err(LookupError::Unavailable)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use base64::Engine;

    fn n() -> String {
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode([0xABu8; 256])
    }

    fn rsa_jwk(kid: &str, alg: &str, key_use: Option<&str>) -> serde_json::Value {
        let mut v =
            serde_json::json!({"kty": "RSA", "kid": kid, "alg": alg, "n": n(), "e": "AQAB"});
        if let Some(u) = key_use {
            v["use"] = serde_json::Value::String(u.to_string());
        }
        v
    }

    fn doc(keys: Vec<serde_json::Value>) -> Vec<u8> {
        serde_json::to_vec(&serde_json::json!({ "keys": keys })).unwrap()
    }

    #[test]
    fn keycloak_style_jwks_keeps_only_signature_key() {
        // Bentuk JWKS Keycloak 26.8.2 (SRC-531): kunci enc RSA-OAEP + kunci sig RS256.
        let set = parse_key_set(
            &doc(vec![
                rsa_jwk("enc-1", "RSA-OAEP", Some("enc")),
                rsa_jwk("sig-1", "RS256", Some("sig")),
            ]),
            &[Algorithm::RS256],
        )
        .unwrap();
        assert_eq!(set.len(), 1);
        assert!(set.get("sig-1").is_some());
        assert!(set.get("enc-1").is_none());
    }

    #[test]
    fn keys_outside_allowlist_or_mismatched_family_are_skipped() {
        let hmac = serde_json::json!({"kty": "oct", "kid": "h", "alg": "HS256", "k": "c2VjcmV0"});
        let rsa_as_ec =
            serde_json::json!({"kty": "RSA", "kid": "x", "alg": "ES256", "n": n(), "e": "AQAB"});
        let no_alg = serde_json::json!({"kty": "RSA", "kid": "na", "n": n(), "e": "AQAB"});
        let no_kid = serde_json::json!({"kty": "RSA", "alg": "RS256", "n": n(), "e": "AQAB"});
        let wrong_ops = serde_json::json!({"kty": "RSA", "kid": "o", "alg": "RS256", "key_ops": ["encrypt"], "n": n(), "e": "AQAB"});
        let rs384 = rsa_jwk("r384", "RS384", None);
        assert!(
            parse_key_set(
                &doc(vec![hmac, rsa_as_ec, no_alg, no_kid, wrong_ops, rs384]),
                &[Algorithm::RS256, Algorithm::ES256],
            )
            .is_none()
        );
    }

    #[test]
    fn duplicate_kid_is_dropped_entirely() {
        let set = parse_key_set(
            &doc(vec![
                rsa_jwk("dup", "RS256", None),
                rsa_jwk("dup", "RS256", None),
                rsa_jwk("ok", "RS256", None),
            ]),
            &[Algorithm::RS256],
        )
        .unwrap();
        assert!(set.get("dup").is_none());
        assert!(set.get("ok").is_some());
    }

    #[test]
    fn unparseable_document_is_rejected() {
        for bad in [&b"not json"[..], b"{}", b"{\"keys\": {}}", b"[]"] {
            assert!(parse_key_set(bad, &[Algorithm::RS256]).is_none());
        }
    }

    #[test]
    fn http_source_accepts_only_loopback_http_with_clean_path() {
        let t = std::time::Duration::from_secs(2);
        assert!(
            HttpJwksSource::new(
                "http://127.0.0.1:18080/realms/pasien/protocol/openid-connect/certs",
                t
            )
            .is_ok()
        );
        assert!(HttpJwksSource::new("http://[::1]:8080/certs", t).is_ok());
        for bad in [
            "https://127.0.0.1:8443/realms/pasien/protocol/openid-connect/certs",
            "http://10.0.0.5:8080/certs",
            "http://keycloak:8080/certs",
            "http://localhost:8080/certs",
            "http://127.0.0.1:0/certs",
            "http://127.0.0.1:8080",
            "http://127.0.0.1:8080/",
            "http://127.0.0.1:8080/a/../b",
            "http://127.0.0.1:8080/certs?x=1",
            "http://user@127.0.0.1:8080/certs",
            "",
        ] {
            assert_eq!(
                HttpJwksSource::new(bad, t).err(),
                Some(AuthnConfigError::InvalidJwksAddress),
                "{bad}"
            );
        }
        assert!(matches!(
            HttpJwksSource::new("http://127.0.0.1:8080/certs", std::time::Duration::ZERO),
            Err(AuthnConfigError::InvalidDuration(_))
        ));
    }
}
