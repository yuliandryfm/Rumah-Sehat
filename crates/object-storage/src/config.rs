//! Konfigurasi dan validasi endpoint (KEPUTUSAN-235): endpoint, region SigV4, dan gaya pengalamatan dari
//! konfigurasi, bukan nilai tertanam; `http://` ditolak di luar lingkungan lokal (port 80 NEO menjawab,
//! spike F1-T03 §1).

use crate::error::ObjectStorageError;
use foundation::Sensitive;
use std::fmt;
use std::time::Duration;

/// Panjang minimum kunci rahasia: `"AWS4" + rahasia` dipakai sebagai kunci HMAC-SHA256 pertama SigV4
/// [SRC-550], dan crate `crypto` hanya menerima kunci HMAC ≥ 32 byte (`crypto::HMAC_MIN_KEY_LEN`).
pub const SECRET_MIN_LEN: usize = 28;

/// Lingkungan penerapan. Hanya `Local` yang boleh memakai endpoint `http://` (KEPUTUSAN-235).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Deployment {
    /// Laptop developer dan test (emulator SeaweedFS, KEPUTUSAN-233).
    Local,
    /// Staging.
    Staging,
    /// Production.
    Production,
}

impl std::str::FromStr for Deployment {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "local" => Ok(Self::Local),
            "staging" => Ok(Self::Staging),
            "production" => Ok(Self::Production),
            _ => Err(()),
        }
    }
}

/// Gaya pengalamatan bucket.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressingStyle {
    /// `<endpoint>/<bucket>/<kunci>` (emulator lokal; dukungan NEO `[TIDAK DIKETAHUI]`, spike F1-T03 §1).
    Path,
    /// `<bucket>.<host endpoint>/<kunci>` (contoh dokumentasi NEO, spike F1-T03 §1).
    VirtualHost,
}

impl std::str::FromStr for AddressingStyle {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "path" => Ok(Self::Path),
            "virtual_host" => Ok(Self::VirtualHost),
            _ => Err(()),
        }
    }
}

/// Konfigurasi koneksi object storage. Kunci rahasia bertipe [`Sensitive`].
pub struct ObjectStorageConfig {
    /// Endpoint dasar, misalnya `http://127.0.0.1:18333` (lokal) atau `https://<host NEO>`; tanpa path.
    pub endpoint: String,
    /// Region SigV4 (nilai NEO `[TIDAK DIKETAHUI]`, spike F1-T03 §1).
    pub region: String,
    /// Gaya pengalamatan.
    pub addressing: AddressingStyle,
    /// ID kunci akses.
    pub access_key_id: String,
    /// Kunci rahasia akses.
    pub secret_access_key: Sensitive<String>,
    /// Lingkungan penerapan.
    pub deployment: Deployment,
    /// Batas waktu satu operasi server (koneksi + respons).
    pub timeout: Duration,
}

impl fmt::Debug for ObjectStorageConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ObjectStorageConfig")
            .field("endpoint", &self.endpoint)
            .field("region", &self.region)
            .field("addressing", &self.addressing)
            .field("access_key_id", &"[REDACTED]")
            .field("secret_access_key", &self.secret_access_key)
            .field("deployment", &self.deployment)
            .field("timeout", &self.timeout)
            .finish()
    }
}

/// Skema endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scheme {
    /// HTTP polos (hanya lokal).
    Http,
    /// HTTPS.
    Https,
}

impl Scheme {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Http => "http",
            Self::Https => "https",
        }
    }

    fn default_port(self) -> u16 {
        match self {
            Self::Http => 80,
            Self::Https => 443,
        }
    }
}

/// Endpoint tervalidasi.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Endpoint {
    pub(crate) scheme: Scheme,
    pub(crate) host: String,
    /// Port eksplisit non-bawaan (port bawaan skema dinormalkan menjadi `None`, seperti header `Host` browser).
    pub(crate) port: Option<u16>,
    pub(crate) is_ip: bool,
}

impl Endpoint {
    /// Nilai header `Host` untuk host tertentu (`host` atau `host:port`).
    pub(crate) fn authority(&self, host: &str) -> String {
        match self.port {
            Some(p) => format!("{host}:{p}"),
            None => host.to_string(),
        }
    }

    pub(crate) fn connect_port(&self) -> u16 {
        self.port.unwrap_or(self.scheme.default_port())
    }
}

fn valid_hostname(host: &str) -> bool {
    !host.is_empty()
        && host.len() <= 253
        && host.split('.').all(|label| {
            !label.is_empty()
                && label.len() <= 63
                && !label.starts_with('-')
                && !label.ends_with('-')
                && label
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        })
}

/// Mengurai dan memvalidasi endpoint. IPv6, userinfo, path, query, dan fragmen ditolak.
pub(crate) fn parse_endpoint(
    endpoint: &str,
    deployment: Deployment,
) -> Result<Endpoint, ObjectStorageError> {
    let (scheme, rest) = if let Some(r) = endpoint.strip_prefix("https://") {
        (Scheme::Https, r)
    } else if let Some(r) = endpoint.strip_prefix("http://") {
        (Scheme::Http, r)
    } else {
        return Err(ObjectStorageError::InvalidConfig("endpoint"));
    };
    if scheme == Scheme::Http && deployment != Deployment::Local {
        return Err(ObjectStorageError::InsecureEndpoint);
    }
    let rest = rest.strip_suffix('/').unwrap_or(rest);
    if rest.is_empty() || rest.contains(['/', '?', '#', '@', '[', ']', ' ']) {
        return Err(ObjectStorageError::InvalidConfig("endpoint"));
    }
    let (host, port) = match rest.rsplit_once(':') {
        Some((h, p)) => {
            let port: u16 = p
                .parse()
                .map_err(|_| ObjectStorageError::InvalidConfig("endpoint"))?;
            if port == 0 {
                return Err(ObjectStorageError::InvalidConfig("endpoint"));
            }
            (h, Some(port))
        }
        None => (rest, None),
    };
    let is_ip = host.parse::<std::net::Ipv4Addr>().is_ok();
    if !is_ip && !valid_hostname(host) {
        return Err(ObjectStorageError::InvalidConfig("endpoint"));
    }
    let port = port.filter(|p| *p != scheme.default_port());
    Ok(Endpoint {
        scheme,
        host: host.to_string(),
        port,
        is_ip,
    })
}

/// Region SigV4: `[a-z0-9-]`, 1–32 karakter.
pub(crate) fn valid_region(region: &str) -> bool {
    (1..=32).contains(&region.len())
        && region
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

/// ID kunci akses: `[A-Za-z0-9._-]`, 3–128 karakter (tanpa `/`, yang memisah cakupan kredensial).
pub(crate) fn valid_access_key_id(id: &str) -> bool {
    (3..=128).contains(&id.len())
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
}

/// Kunci rahasia: ASCII tampak, `SECRET_MIN_LEN`–256 karakter.
pub(crate) fn valid_secret(secret: &str) -> bool {
    (SECRET_MIN_LEN..=256).contains(&secret.len()) && secret.bytes().all(|b| b.is_ascii_graphic())
}

/// Nama bucket: 3–63 karakter `[a-z0-9-]`, diawali dan diakhiri huruf/angka. Titik ditolak supaya nama
/// selalu sah sebagai satu label host (virtual-host + sertifikat wildcard, spike F1-T03 §1).
pub fn valid_bucket_name(bucket: &str) -> bool {
    let b = bucket.as_bytes();
    (3..=63).contains(&b.len())
        && b.iter()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == b'-')
        && b.first().is_some_and(u8::is_ascii_alphanumeric)
        && b.last().is_some_and(u8::is_ascii_alphanumeric)
}

/// Kunci objek: 1–256 karakter `[A-Za-z0-9._-]` (kunci dibuat aplikasi dari UUID, ADR-018 butir 2).
pub fn valid_object_key(key: &str) -> bool {
    (1..=256).contains(&key.len())
        && key
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
        && key != "."
        && key != ".."
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn http_endpoint_is_accepted_only_for_local() {
        assert!(parse_endpoint("http://127.0.0.1:18333", Deployment::Local).is_ok());
        assert_eq!(
            parse_endpoint("http://127.0.0.1:18333", Deployment::Staging),
            Err(ObjectStorageError::InsecureEndpoint)
        );
        assert_eq!(
            parse_endpoint("http://nos.jkt-1.neo.id", Deployment::Production),
            Err(ObjectStorageError::InsecureEndpoint)
        );
        assert!(parse_endpoint("https://nos.jkt-1.neo.id", Deployment::Production).is_ok());
    }

    #[test]
    fn malformed_endpoints_are_rejected() {
        for bad in [
            "",
            "nos.jkt-1.neo.id",
            "ftp://host",
            "https://",
            "https://host/path",
            "https://user@host",
            "https://host?x=1",
            "https://[::1]:9000",
            "https://host:0",
            "https://host:99999",
            "https://Host.Example",
            "https://-bad.example",
        ] {
            assert!(
                parse_endpoint(bad, Deployment::Local).is_err(),
                "harus ditolak: {bad}"
            );
        }
    }

    #[test]
    fn default_port_is_normalised_away() {
        let e = parse_endpoint("https://nos.jkt-1.neo.id:443/", Deployment::Production).unwrap();
        assert_eq!(e.port, None);
        assert_eq!(e.authority("x.nos.jkt-1.neo.id"), "x.nos.jkt-1.neo.id");
        let e = parse_endpoint("http://127.0.0.1:18333", Deployment::Local).unwrap();
        assert_eq!(e.authority("127.0.0.1"), "127.0.0.1:18333");
        assert!(e.is_ip);
    }

    #[test]
    fn names_are_validated() {
        assert!(valid_bucket_name("rs-identity-document"));
        assert!(!valid_bucket_name("rs.identity"));
        assert!(!valid_bucket_name("Rs-identity"));
        assert!(!valid_bucket_name("-rs"));
        assert!(!valid_bucket_name("ab"));
        assert!(valid_object_key("0193a1b2-c3d4-7e5f-8a9b-0c1d2e3f4a5b"));
        assert!(!valid_object_key("../etc"));
        assert!(!valid_object_key("a b"));
        assert!(!valid_object_key(".."));
        assert!(valid_region("us-east-1"));
        assert!(!valid_region("US"));
        assert!(!valid_region(""));
        assert!(valid_access_key_id("sintetis-lokal-s3-akses"));
        assert!(!valid_access_key_id("a/b"));
        assert!(valid_secret(&"x".repeat(SECRET_MIN_LEN)));
        assert!(!valid_secret(&"x".repeat(SECRET_MIN_LEN - 1)));
        assert!(!valid_secret(&format!("{} ", "x".repeat(40))));
    }
}
