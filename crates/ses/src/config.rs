//! Konfigurasi adaptor SES (ADR-026 butir 3 dan 5).
//!
//! - Region selalu `ap-southeast-3` (Jakarta) dan endpoint bawaan `email.ap-southeast-3.amazonaws.com`
//!   [SRC-294]; tidak ada `EndpointId` multi-region (ADR-026 butir 4).
//! - Endpoint lain hanya untuk tiruan SES di test lokal ([`SesConfig::local_endpoint`]) dan selalu HTTPS.
//! - Batas waktu satu panggilan wajib lebih pendek dari batas tulis Keycloak 10000 ms [SRC-301]:
//!   maksimum [`MAX_TIMEOUT`].

use crate::error::SesError;
use foundation::Sensitive;
use std::fmt;
use std::path::PathBuf;
use std::time::Duration;

/// Region SES yang diizinkan (data di Indonesia, `[USER] T4-Q1`, ADR-026).
pub const REGION: &str = "ap-southeast-3";
/// Host endpoint API SES v2 Jakarta [SRC-294].
pub const DEFAULT_HOST: &str = "email.ap-southeast-3.amazonaws.com";
/// Batas waktu maksimum satu panggilan (koneksi + TLS + respons); < 10 detik batas tulis Keycloak.
pub const MAX_TIMEOUT: Duration = Duration::from_secs(9);
/// Panjang minimum kunci rahasia AWS: `"AWS4" + rahasia` menjadi kunci HMAC pertama dan crate `crypto`
/// hanya menerima kunci HMAC ≥ 32 byte.
pub const SECRET_MIN_LEN: usize = 28;

/// Kunci akses AWS akun email (ADR-026 butir 5). Rahasia tidak pernah muncul di `Debug`.
#[derive(Clone)]
pub struct AwsCredentials {
    /// ID kunci akses (bukan rahasia, tetapi tidak dicatat di log).
    pub access_key_id: String,
    /// Kunci rahasia.
    pub secret_access_key: Sensitive<String>,
}

impl fmt::Debug for AwsCredentials {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("AwsCredentials([REDACTED])")
    }
}

impl AwsCredentials {
    /// Memvalidasi bentuk: ID `[A-Z0-9]{16,128}`, rahasia ASCII tercetak ≥ [`SECRET_MIN_LEN`].
    pub fn validate(&self) -> Result<(), SesError> {
        let id = &self.access_key_id;
        if !(16..=128).contains(&id.len())
            || !id
                .bytes()
                .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit())
        {
            return Err(SesError::InvalidConfig("access_key_id"));
        }
        let s = self.secret_access_key.expose();
        if s.len() < SECRET_MIN_LEN || s.len() > 256 || !s.bytes().all(|b| b.is_ascii_graphic()) {
            return Err(SesError::InvalidConfig("secret_access_key"));
        }
        Ok(())
    }
}

/// Konfigurasi klien SES.
#[derive(Debug, Clone)]
pub struct SesConfig {
    /// `None` = endpoint SES Jakarta. `Some("https://host:port")` hanya untuk tiruan SES lokal; pemanggil
    /// (konfigurasi proses) wajib menolak nilai ini di luar lingkungan lokal.
    pub local_endpoint: Option<String>,
    /// Berkas PEM berisi CA tepercaya untuk TLS ke endpoint (production: bundel CA sistem kontainer).
    pub ca_file: PathBuf,
    /// Batas waktu satu panggilan, `1 ms ..= MAX_TIMEOUT`.
    pub timeout: Duration,
    /// Configuration set SES yang selalu diisi eksplisit (ADR-026 butir 4 dan 5).
    pub configuration_set: String,
    /// Kunci akses AWS.
    pub credentials: AwsCredentials,
}

/// Endpoint yang sudah divalidasi.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Endpoint {
    /// Host untuk koneksi dan SNI/verifikasi sertifikat.
    pub(crate) host: String,
    pub(crate) port: u16,
}

impl Endpoint {
    /// Nilai header `Host`.
    pub(crate) fn host_header(&self) -> String {
        if self.port == 443 {
            self.host.clone()
        } else {
            format!("{}:{}", self.host, self.port)
        }
    }
}

fn valid_host(host: &str) -> bool {
    !host.is_empty()
        && host.len() <= 253
        && host.split('.').all(|l| {
            !l.is_empty()
                && l.len() <= 63
                && !l.starts_with('-')
                && !l.ends_with('-')
                && l.bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        })
}

/// Nama configuration set SES: `[A-Za-z0-9_-]{1,64}`.
pub(crate) fn valid_configuration_set(name: &str) -> bool {
    (1..=64).contains(&name.len())
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}

/// Mengurai `https://<host>[:port]` tanpa path, query, atau userinfo.
pub(crate) fn parse_endpoint(raw: &str) -> Result<Endpoint, SesError> {
    let rest = raw
        .strip_prefix("https://")
        .ok_or(SesError::InvalidConfig("endpoint"))?;
    let rest = rest.strip_suffix('/').unwrap_or(rest);
    let (host, port) = match rest.rsplit_once(':') {
        Some((h, p)) => (
            h,
            p.parse::<u16>()
                .ok()
                .filter(|p| *p != 0)
                .ok_or(SesError::InvalidConfig("endpoint"))?,
        ),
        None => (rest, 443),
    };
    if !valid_host(host) {
        return Err(SesError::InvalidConfig("endpoint"));
    }
    Ok(Endpoint {
        host: host.to_string(),
        port,
    })
}

impl SesConfig {
    pub(crate) fn endpoint(&self) -> Result<Endpoint, SesError> {
        match &self.local_endpoint {
            None => Ok(Endpoint {
                host: DEFAULT_HOST.to_string(),
                port: 443,
            }),
            Some(raw) => parse_endpoint(raw),
        }
    }

    pub(crate) fn validate(&self) -> Result<Endpoint, SesError> {
        if self.timeout.is_zero() || self.timeout > MAX_TIMEOUT {
            return Err(SesError::InvalidConfig("timeout"));
        }
        if !valid_configuration_set(&self.configuration_set) {
            return Err(SesError::InvalidConfig("configuration_set"));
        }
        self.credentials.validate()?;
        self.endpoint()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn creds() -> AwsCredentials {
        AwsCredentials {
            access_key_id: "AKIAIOSFODNN7EXAMPLE".to_string(),
            secret_access_key: Sensitive::new(
                "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY".to_string(),
            ),
        }
    }

    fn cfg() -> SesConfig {
        SesConfig {
            local_endpoint: None,
            ca_file: PathBuf::from("/etc/ssl/certs/ca-certificates.crt"),
            timeout: Duration::from_secs(8),
            configuration_set: "rs-keycloak".to_string(),
            credentials: creds(),
        }
    }

    #[test]
    fn default_endpoint_is_jakarta() {
        let e = cfg().validate().unwrap();
        assert_eq!(e.host, "email.ap-southeast-3.amazonaws.com");
        assert_eq!(e.host_header(), "email.ap-southeast-3.amazonaws.com");
    }

    #[test]
    fn timeout_must_be_below_keycloak_write_timeout() {
        let mut c = cfg();
        c.timeout = Duration::from_millis(9_001);
        assert_eq!(c.validate().err(), Some(SesError::InvalidConfig("timeout")));
        c.timeout = Duration::ZERO;
        assert_eq!(c.validate().err(), Some(SesError::InvalidConfig("timeout")));
        c.timeout = MAX_TIMEOUT;
        assert!(c.validate().is_ok());
    }

    #[test]
    fn endpoint_must_be_https_without_path_or_userinfo() {
        assert_eq!(
            parse_endpoint("https://ses-tiruan.uji.invalid:18443")
                .unwrap()
                .host_header(),
            "ses-tiruan.uji.invalid:18443"
        );
        assert_eq!(
            parse_endpoint("https://127.0.0.1:443")
                .unwrap()
                .host_header(),
            "127.0.0.1"
        );
        for bad in [
            "http://127.0.0.1:18443",
            "https://u@h:1",
            "https://h/v2",
            "https://h:0",
            "https://h:x",
            "https://H",
            "https://",
            "https://[::1]:1",
            "h",
        ] {
            assert_eq!(
                parse_endpoint(bad).err(),
                Some(SesError::InvalidConfig("endpoint")),
                "{bad}"
            );
        }
    }

    #[test]
    fn credentials_and_configuration_set_are_validated_without_leaking() {
        let mut c = cfg();
        c.configuration_set = "a b".to_string();
        assert_eq!(
            c.validate().err(),
            Some(SesError::InvalidConfig("configuration_set"))
        );
        let mut c = cfg();
        c.credentials.access_key_id = "akia".to_string();
        assert_eq!(
            c.validate().err(),
            Some(SesError::InvalidConfig("access_key_id"))
        );
        let mut c = cfg();
        c.credentials.secret_access_key = Sensitive::new("pendek".to_string());
        assert_eq!(
            c.validate().err(),
            Some(SesError::InvalidConfig("secret_access_key"))
        );
        let out = format!("{:?}", cfg());
        assert!(!out.contains("EXAMPLEKEY") && !out.contains("AKIAIOSFODNN7EXAMPLE"));
    }
}
