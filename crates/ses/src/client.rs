//! Klien SES API v2 `SendEmail` (isi Raw) lewat HTTPS: TCP → rustls (penyedia aws-lc-rs dari crate
//! `crypto`) → hyper HTTP/1, satu koneksi per pesan, tanpa coba ulang (ADR-026 butir 3: sinkron, tanpa
//! penyimpanan). Isi respons galat SES tidak dibaca (dapat menggemakan alamat).

use crate::config::{Endpoint, REGION, SesConfig};
use crate::error::SesError;
use crate::sigv4::{SignInput, authorization};
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use bytes::Bytes;
use foundation::Sensitive;
use http::{Request, StatusCode, header};
use http_body_util::{BodyExt, Full, Limited};
use hyper_util::rt::TokioIo;
use rustls::RootCertStore;
use rustls_pki_types::pem::PemObject;
use rustls_pki_types::{CertificateDer, ServerName};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::Arc;
use std::time::Duration;
use time::OffsetDateTime;
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;

/// Path operasi `SendEmail` [SRC-580].
const SEND_EMAIL_PATH: &str = "/v2/email/outbound-emails";
/// Batas ukuran respons yang dibaca (byte).
const MAX_RESPONSE_BYTES: usize = 16 * 1024;
/// Batas ukuran pesan Raw yang diteruskan (byte, sebelum base64). Batas relay jauh lebih kecil.
pub const MAX_RAW_BYTES: usize = 10 * 1024 * 1024;

/// Pesan yang dikirim. Alamat dan isi tidak pernah muncul di `Debug`.
pub struct RawEmail<'a> {
    /// `FromEmailAddress` (dari daftar izin, ADR-026 butir 4).
    pub from: &'a str,
    /// Satu-satunya penerima (`Destination.ToAddresses`).
    pub to: &'a str,
    /// Pesan MIME mentah dengan pemisah CRLF.
    pub raw: &'a [u8],
}

impl fmt::Debug for RawEmail<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RawEmail({} byte)", self.raw.len())
    }
}

/// `MessageId` dari SES (boleh dicatat, ADR-026 butir 4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageId(String);

impl MessageId {
    /// Nilai.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Hanya ASCII tercetak tanpa spasi, 1–256 karakter (aman ditulis ke log).
    fn parse(raw: &str) -> Option<Self> {
        ((1..=256).contains(&raw.len()) && raw.bytes().all(|b| b.is_ascii_graphic()))
            .then(|| Self(raw.to_string()))
    }
}

impl fmt::Display for MessageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct SendEmailRequest<'a> {
    from_email_address: &'a str,
    destination: Destination<'a>,
    content: Content,
    configuration_set_name: &'a str,
    configuration_overrides: ConfigurationOverrides,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct Destination<'a> {
    to_addresses: [&'a str; 1],
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct Content {
    raw: Raw,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct Raw {
    data: String,
}

/// Pelacakan klik/buka dimatikan per pesan (ADR-026 butir 5) [SRC-580].
#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct ConfigurationOverrides {
    tracking: Tracking,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct Tracking {
    click_tracking_enabled: &'static str,
    open_tracking_enabled: &'static str,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct SendEmailResponse {
    message_id: String,
}

/// Klien SES yang sudah divalidasi.
pub struct SesClient {
    endpoint: Endpoint,
    server_name: ServerName<'static>,
    connector: TlsConnector,
    timeout: Duration,
    configuration_set: String,
    access_key_id: String,
    secret: Sensitive<String>,
}

impl fmt::Debug for SesClient {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SesClient")
            .field("endpoint", &self.endpoint.host_header())
            .field("timeout", &self.timeout)
            .finish_non_exhaustive()
    }
}

/// Memuat CA tepercaya dari berkas PEM; minimal satu sertifikat harus terbaca.
fn load_roots(path: &std::path::Path) -> Result<RootCertStore, SesError> {
    let certs: Vec<CertificateDer<'static>> = CertificateDer::pem_file_iter(path)
        .map_err(|_| SesError::CaBundle)?
        .collect::<Result<_, _>>()
        .map_err(|_| SesError::CaBundle)?;
    let mut roots = RootCertStore::empty();
    let (added, _ignored) = roots.add_parsable_certificates(certs);
    if added == 0 {
        return Err(SesError::CaBundle);
    }
    Ok(roots)
}

impl SesClient {
    /// Memvalidasi konfigurasi dan memuat CA. Tidak membuka koneksi.
    pub fn new(config: &SesConfig) -> Result<Self, SesError> {
        let endpoint = config.validate()?;
        let roots = load_roots(&config.ca_file)?;
        let tls = rustls::ClientConfig::builder_with_provider(crypto::tls_crypto_provider())
            .with_safe_default_protocol_versions()
            .map_err(|_| SesError::InvalidConfig("tls"))?
            .with_root_certificates(roots)
            .with_no_client_auth();
        let server_name = ServerName::try_from(endpoint.host.clone())
            .map_err(|_| SesError::InvalidConfig("endpoint"))?;
        Ok(Self {
            endpoint,
            server_name,
            connector: TlsConnector::from(Arc::new(tls)),
            timeout: config.timeout,
            configuration_set: config.configuration_set.clone(),
            access_key_id: config.credentials.access_key_id.clone(),
            secret: config.credentials.secret_access_key.clone(),
        })
    }

    /// Mengirim satu pesan Raw ke satu penerima. Sukses hanya bila SES menjawab 200 dengan `MessageId`.
    pub async fn send_raw(
        &self,
        email: &RawEmail<'_>,
        now: OffsetDateTime,
    ) -> Result<MessageId, SesError> {
        if email.raw.is_empty() || email.raw.len() > MAX_RAW_BYTES {
            return Err(SesError::InvalidInput("raw"));
        }
        if email.from.is_empty() || email.to.is_empty() {
            return Err(SesError::InvalidInput("address"));
        }
        let body = serde_json::to_vec(&SendEmailRequest {
            from_email_address: email.from,
            destination: Destination {
                to_addresses: [email.to],
            },
            content: Content {
                raw: Raw {
                    data: STANDARD.encode(email.raw),
                },
            },
            configuration_set_name: &self.configuration_set,
            configuration_overrides: ConfigurationOverrides {
                tracking: Tracking {
                    click_tracking_enabled: "DISABLED",
                    open_tracking_enabled: "DISABLED",
                },
            },
        })
        .map_err(|_| SesError::InvalidInput("request"))?;
        match tokio::time::timeout(self.timeout, self.post(body, now)).await {
            Ok(r) => r,
            Err(_) => Err(SesError::Timeout),
        }
    }

    async fn post(&self, body: Vec<u8>, now: OffsetDateTime) -> Result<MessageId, SesError> {
        let host = self.endpoint.host_header();
        let (auth, amz_date) = authorization(
            &SignInput {
                access_key_id: &self.access_key_id,
                secret: &self.secret,
                region: REGION,
                host: &host,
                path: SEND_EMAIL_PATH,
                content_type: "application/json",
                body: &body,
            },
            now,
        )?;
        let tcp = TcpStream::connect((self.endpoint.host.as_str(), self.endpoint.port))
            .await
            .map_err(|_| SesError::Unavailable)?;
        let tls = self
            .connector
            .connect(self.server_name.clone(), tcp)
            .await
            .map_err(|_| SesError::Tls)?;
        let (mut sender, conn) = hyper::client::conn::http1::handshake(TokioIo::new(tls))
            .await
            .map_err(|_| SesError::Unavailable)?;
        // Task koneksi dihentikan saat future ini dibuang, termasuk saat batas waktu `send_raw` habis atau
        // galat `?` di bawah (review keamanan F1-T36): koneksi ke SES tidak tertinggal di latar belakang.
        let _driver = AbortOnDrop(tokio::spawn(conn));
        let mut authorization =
            http::HeaderValue::from_str(&auth).map_err(|_| SesError::Signing)?;
        authorization.set_sensitive(true);
        let request = Request::builder()
            .method(http::Method::POST)
            .uri(SEND_EMAIL_PATH)
            .header(header::HOST, host)
            .header(header::CONTENT_TYPE, "application/json")
            .header(header::CONTENT_LENGTH, body.len())
            .header("x-amz-date", amz_date)
            .header(header::AUTHORIZATION, authorization)
            .header(header::CONNECTION, "close")
            .body(Full::new(Bytes::from(body)))
            .map_err(|_| SesError::InvalidInput("request"))?;
        let response = sender
            .send_request(request)
            .await
            .map_err(|_| SesError::Unavailable)?;
        let status = response.status();
        if status != StatusCode::OK {
            return Err(map_status(status));
        }
        let collected = Limited::new(response.into_body(), MAX_RESPONSE_BYTES)
            .collect()
            .await
            .map_err(|_| SesError::InvalidResponse)?;
        let parsed: SendEmailResponse =
            serde_json::from_slice(&collected.to_bytes()).map_err(|_| SesError::InvalidResponse)?;
        MessageId::parse(&parsed.message_id).ok_or(SesError::InvalidResponse)
    }
}

/// Menghentikan task saat dibuang (`JoinHandle` yang dibuang hanya melepas task, tidak menghentikannya).
struct AbortOnDrop<T>(tokio::task::JoinHandle<T>);

impl<T> Drop for AbortOnDrop<T> {
    fn drop(&mut self) {
        self.0.abort();
    }
}

/// Pemetaan status non-200 (ADR-026 butir 3): 429 dan 5xx sementara, 401/403 sementara (konfigurasi),
/// 4xx lain permanen.
fn map_status(status: StatusCode) -> SesError {
    let code = status.as_u16();
    match code {
        429 => SesError::Throttled,
        401 | 403 => SesError::AuthRejected { status: code },
        500..=599 => SesError::ServerError { status: code },
        400..=499 => SesError::Rejected { status: code },
        // 1xx/2xx selain 200/3xx: SES tidak mendokumentasikannya untuk SendEmail; tidak dianggap sukses.
        _ => SesError::InvalidResponse,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_mapping_follows_adr_026() {
        assert_eq!(
            map_status(StatusCode::TOO_MANY_REQUESTS),
            SesError::Throttled
        );
        assert_eq!(
            map_status(StatusCode::BAD_REQUEST),
            SesError::Rejected { status: 400 }
        );
        assert_eq!(
            map_status(StatusCode::NOT_FOUND),
            SesError::Rejected { status: 404 }
        );
        assert_eq!(
            map_status(StatusCode::FORBIDDEN),
            SesError::AuthRejected { status: 403 }
        );
        assert_eq!(
            map_status(StatusCode::SERVICE_UNAVAILABLE),
            SesError::ServerError { status: 503 }
        );
        assert_eq!(map_status(StatusCode::ACCEPTED), SesError::InvalidResponse);
        assert!(map_status(StatusCode::TOO_MANY_REQUESTS).is_temporary());
        assert!(map_status(StatusCode::BAD_GATEWAY).is_temporary());
        assert!(!map_status(StatusCode::BAD_REQUEST).is_temporary());
    }

    #[test]
    fn message_id_is_restricted_to_printable_ascii() {
        assert!(MessageId::parse("0100018f-abc").is_some());
        assert!(MessageId::parse("").is_none());
        assert!(MessageId::parse("a b").is_none());
        assert!(MessageId::parse("a\nb").is_none());
        assert!(MessageId::parse(&"x".repeat(257)).is_none());
    }

    #[test]
    fn request_body_has_explicit_destination_from_configuration_set_and_no_tracking() {
        let body = serde_json::to_value(SendEmailRequest {
            from_email_address: "noreply@uji.invalid",
            destination: Destination {
                to_addresses: ["pasien@uji.invalid"],
            },
            content: Content {
                raw: Raw {
                    data: STANDARD.encode(b"x"),
                },
            },
            configuration_set_name: "rs-keycloak",
            configuration_overrides: ConfigurationOverrides {
                tracking: Tracking {
                    click_tracking_enabled: "DISABLED",
                    open_tracking_enabled: "DISABLED",
                },
            },
        })
        .unwrap();
        assert_eq!(
            body,
            serde_json::json!({
                "FromEmailAddress": "noreply@uji.invalid",
                "Destination": {"ToAddresses": ["pasien@uji.invalid"]},
                "Content": {"Raw": {"Data": "eA=="}},
                "ConfigurationSetName": "rs-keycloak",
                "ConfigurationOverrides": {"Tracking": {
                    "ClickTrackingEnabled": "DISABLED", "OpenTrackingEnabled": "DISABLED"}}
            })
        );
        assert!(body.get("EndpointId").is_none());
    }
}
