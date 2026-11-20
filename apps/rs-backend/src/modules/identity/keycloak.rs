//! Klien Admin REST Keycloak minimal untuk pengambil event (F1-T34; ADR-011 butir 10).
//!
//! - Token akses: grant `client_credentials` klien layanan `rs-backend-event-reader` per realm, yang hanya
//!   memegang peran `realm-management/view-events` dan cakupan `kc-admin-api` (audience Admin REST Keycloak
//!   26.8.2; tanpa cakupan itu Admin REST menjawab 401, diamati SRC-621). Konfigurasi klien ada di
//!   `infra/keycloak/realms/*.json` dan diperiksa `scripts/keycloak-check.sh`.
//! - Event: `GET /admin/realms/{realm}/events?type=..&dateFrom=<ms>&direction=asc&first=0&max=<n>`
//!   (parameter dan semantiknya dari kode sumber `RealmAdminResource`/`JpaEventQuery` 26.8.2, SRC-620:
//!   `dateFrom`/`dateTo` inklusif dalam milidetik epoch, urutan hanya menurut `time`).
//!
//! Batasan (gagal-tertutup, sama dengan klien JWKS `crates/authn`): hanya `http://<ip-loopback>:<port>`.
//! TLS ke Keycloak lintas host (docs/05-stack.md §2.1) belum diterapkan; alamat lain ditolak saat
//! konfigurasi. Rahasia klien dan token tidak pernah masuk log, galat, atau `Debug`.

use super::domain::{IdpRealm, PULLED_TYPES, RawEvent};
use bytes::Bytes;
use foundation::{Clock, Sensitive};
use http::{HeaderValue, Method, Request, StatusCode, header};
use http_body_util::{BodyExt, Full, Limited};
use hyper_util::rt::TokioIo;
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt;
use std::net::{IpAddr, SocketAddr};
use std::time::Duration;
use time::OffsetDateTime;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

/// Batas ukuran respons token (byte).
const MAX_TOKEN_BYTES: usize = 64 * 1024;
/// Batas ukuran satu halaman event (byte): 500 event x ~4 KiB.
const MAX_EVENTS_BYTES: usize = 2 * 1024 * 1024;
/// Token dianggap kedaluwarsa sekian detik sebelum `expires_in`.
const TOKEN_SKEW_SECONDS: i64 = 30;

/// Kegagalan Admin REST. Tidak memuat isi respons, URL, maupun rahasia.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeycloakError {
    /// Koneksi gagal.
    Connect,
    /// Batas waktu terlewati.
    Timeout,
    /// Endpoint token menolak kredensial atau gagal (status HTTP).
    Token(u16),
    /// Endpoint event menjawab status selain 200.
    Status(u16),
    /// Respons melebihi batas atau terputus.
    Body,
    /// Respons bukan JSON yang diharapkan.
    Json,
}

impl fmt::Display for KeycloakError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Connect => f.write_str("koneksi ke Keycloak gagal"),
            Self::Timeout => f.write_str("Keycloak tidak menjawab dalam batas waktu"),
            Self::Token(s) => write!(f, "token klien layanan ditolak (HTTP {s})"),
            Self::Status(s) => write!(f, "Admin REST event menjawab HTTP {s}"),
            Self::Body => f.write_str("respons Keycloak terlalu besar atau terputus"),
            Self::Json => f.write_str("respons Keycloak tidak dapat diuraikan"),
        }
    }
}

impl std::error::Error for KeycloakError {}

/// Konfigurasi galat alamat Keycloak.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidKeycloakAddress;

/// Alamat dasar Keycloak `http://<ip-loopback>:<port>` (tanpa path).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeycloakAddress(SocketAddr);

impl std::str::FromStr for KeycloakAddress {
    type Err = InvalidKeycloakAddress;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let rest = s.strip_prefix("http://").ok_or(InvalidKeycloakAddress)?;
        let authority = rest.strip_suffix('/').unwrap_or(rest);
        let addr: SocketAddr = authority.parse().map_err(|_| InvalidKeycloakAddress)?;
        let loopback = match addr.ip() {
            IpAddr::V4(ip) => ip.is_loopback(),
            IpAddr::V6(ip) => ip.is_loopback(),
        };
        if !loopback || addr.port() == 0 {
            return Err(InvalidKeycloakAddress);
        }
        Ok(Self(addr))
    }
}

/// Kredensial klien layanan satu realm.
#[derive(Clone)]
pub struct RealmCredential {
    /// `client_id` klien layanan.
    pub client_id: String,
    /// Rahasia klien (dari OpenBao lewat lingkungan proses).
    pub secret: Sensitive<String>,
}

impl fmt::Debug for RealmCredential {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RealmCredential")
            .field("client_id", &self.client_id)
            .finish_non_exhaustive()
    }
}

struct CachedToken {
    value: Sensitive<String>,
    valid_until: OffsetDateTime,
}

/// Klien Admin REST event Keycloak.
pub struct KeycloakClient {
    addr: SocketAddr,
    host_header: HeaderValue,
    timeout: Duration,
    credentials: HashMap<IdpRealm, RealmCredential>,
    tokens: Mutex<HashMap<IdpRealm, CachedToken>>,
}

impl fmt::Debug for KeycloakClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("KeycloakClient")
            .field("addr", &self.addr)
            .finish_non_exhaustive()
    }
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: Option<i64>,
}

/// Satu halaman event mentah beserta jumlah elemen yang diterima.
pub struct EventPage {
    /// Event urut menurut `time` naik (urutan Keycloak `direction=asc`).
    pub events: Vec<RawEvent>,
}

impl KeycloakClient {
    /// Klien untuk `addr` dengan kredensial per realm.
    pub fn new(
        addr: KeycloakAddress,
        timeout: Duration,
        credentials: HashMap<IdpRealm, RealmCredential>,
    ) -> Result<Self, InvalidKeycloakAddress> {
        let host_header =
            HeaderValue::from_str(&addr.0.to_string()).map_err(|_| InvalidKeycloakAddress)?;
        Ok(Self {
            addr: addr.0,
            host_header,
            timeout,
            credentials,
            tokens: Mutex::new(HashMap::new()),
        })
    }

    /// Mengambil satu halaman event realm `realm` dengan `time >= from_ms` (dan `<= to_ms` bila ada),
    /// urut `time` naik, paling banyak `max`. Token dicoba ulang sekali bila ditolak (401).
    pub async fn fetch_events(
        &self,
        realm: IdpRealm,
        from_ms: i64,
        to_ms: Option<i64>,
        max: usize,
        clock: &dyn Clock,
    ) -> Result<EventPage, KeycloakError> {
        let mut path = format!(
            "/admin/realms/{}/events?direction=asc&first=0&max={max}&dateFrom={from_ms}",
            realm.as_str()
        );
        if let Some(to) = to_ms {
            path.push_str(&format!("&dateTo={to}"));
        }
        for t in PULLED_TYPES {
            path.push_str("&type=");
            path.push_str(t);
        }
        for attempt in 0..2 {
            let token = self.token(realm, clock).await?;
            let (status, body) = self
                .send(Method::GET, &path, Some(&token), None, MAX_EVENTS_BYTES)
                .await?;
            if status == StatusCode::UNAUTHORIZED && attempt == 0 {
                self.tokens.lock().await.remove(&realm);
                continue;
            }
            if status != StatusCode::OK {
                return Err(KeycloakError::Status(status.as_u16()));
            }
            let events: Vec<RawEvent> =
                serde_json::from_slice(&body).map_err(|_| KeycloakError::Json)?;
            return Ok(EventPage { events });
        }
        Err(KeycloakError::Status(StatusCode::UNAUTHORIZED.as_u16()))
    }

    async fn token(
        &self,
        realm: IdpRealm,
        clock: &dyn Clock,
    ) -> Result<Sensitive<String>, KeycloakError> {
        let now = clock.now();
        let mut tokens = self.tokens.lock().await;
        if let Some(t) = tokens.get(&realm)
            && t.valid_until > now
        {
            return Ok(t.value.clone());
        }
        let cred = self
            .credentials
            .get(&realm)
            .ok_or(KeycloakError::Token(0))?;
        let form = format!(
            "grant_type=client_credentials&client_id={}&client_secret={}",
            form_encode(&cred.client_id),
            form_encode(cred.secret.expose())
        );
        let path = format!("/realms/{}/protocol/openid-connect/token", realm.as_str());
        let (status, body) = self
            .send(
                Method::POST,
                &path,
                None,
                Some(Sensitive::new(form)),
                MAX_TOKEN_BYTES,
            )
            .await?;
        if status != StatusCode::OK {
            return Err(KeycloakError::Token(status.as_u16()));
        }
        let parsed: TokenResponse =
            serde_json::from_slice(&body).map_err(|_| KeycloakError::Json)?;
        let lifetime = parsed.expires_in.unwrap_or(60).clamp(0, 3600) - TOKEN_SKEW_SECONDS;
        let value = Sensitive::new(parsed.access_token);
        tokens.insert(
            realm,
            CachedToken {
                value: value.clone(),
                valid_until: now + time::Duration::seconds(lifetime.max(0)),
            },
        );
        Ok(value)
    }

    async fn send(
        &self,
        method: Method,
        path: &str,
        bearer: Option<&Sensitive<String>>,
        form: Option<Sensitive<String>>,
        limit: usize,
    ) -> Result<(StatusCode, Bytes), KeycloakError> {
        match tokio::time::timeout(
            self.timeout,
            self.send_inner(method, path, bearer, form, limit),
        )
        .await
        {
            Ok(r) => r,
            Err(_) => Err(KeycloakError::Timeout),
        }
    }

    async fn send_inner(
        &self,
        method: Method,
        path: &str,
        bearer: Option<&Sensitive<String>>,
        form: Option<Sensitive<String>>,
        limit: usize,
    ) -> Result<(StatusCode, Bytes), KeycloakError> {
        let stream = TcpStream::connect(self.addr)
            .await
            .map_err(|_| KeycloakError::Connect)?;
        let (mut sender, conn) = hyper::client::conn::http1::handshake(TokioIo::new(stream))
            .await
            .map_err(|_| KeycloakError::Connect)?;
        let driver = tokio::spawn(conn);
        let mut builder = Request::builder()
            .method(method)
            .uri(path)
            .header(header::HOST, self.host_header.clone())
            .header(header::ACCEPT, "application/json")
            .header(header::CONNECTION, "close");
        if let Some(t) = bearer {
            let mut v = HeaderValue::from_str(&format!("Bearer {}", t.expose()))
                .map_err(|_| KeycloakError::Connect)?;
            v.set_sensitive(true);
            builder = builder.header(header::AUTHORIZATION, v);
        }
        let body = match form {
            Some(f) => {
                builder = builder.header(header::CONTENT_TYPE, "application/x-www-form-urlencoded");
                Full::new(Bytes::from(f.into_inner()))
            }
            None => Full::new(Bytes::new()),
        };
        let request = builder.body(body).map_err(|_| KeycloakError::Connect)?;
        let result = async {
            let response = sender
                .send_request(request)
                .await
                .map_err(|_| KeycloakError::Connect)?;
            let status = response.status();
            let collected = Limited::new(response.into_body(), limit)
                .collect()
                .await
                .map_err(|_| KeycloakError::Body)?;
            Ok((status, collected.to_bytes()))
        }
        .await;
        driver.abort();
        result
    }
}

/// `application/x-www-form-urlencoded`: karakter tak-cadangan RFC 3986 apa adanya, sisanya `%XX`.
fn form_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_' | b'~') {
            out.push(char::from(b));
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn address_accepts_only_loopback_http() {
        assert!("http://127.0.0.1:18081".parse::<KeycloakAddress>().is_ok());
        assert!("http://127.0.0.1:18081/".parse::<KeycloakAddress>().is_ok());
        assert!("http://[::1]:8080".parse::<KeycloakAddress>().is_ok());
        for bad in [
            "https://127.0.0.1:18081",
            "http://10.0.0.5:8080",
            "http://keycloak:8080",
            "http://127.0.0.1:0",
            "http://127.0.0.1:8080/realms",
            "127.0.0.1:8080",
        ] {
            assert!(bad.parse::<KeycloakAddress>().is_err(), "{bad}");
        }
    }

    #[test]
    fn form_encoding_escapes_reserved_characters() {
        assert_eq!(form_encode("a-b_c.d~e"), "a-b_c.d~e");
        assert_eq!(form_encode("a b&c=d+e/f%"), "a%20b%26c%3Dd%2Be%2Ff%25");
    }

    #[test]
    fn debug_hides_secret() {
        let c = RealmCredential {
            client_id: "rs-backend-event-reader".into(),
            secret: Sensitive::new("rahasia-sangat-panjang".into()),
        };
        assert!(!format!("{c:?}").contains("rahasia"));
    }
}
