//! Operasi server ke object storage (HEAD, GET terbatas, PUT, hapus semua versi) lewat hyper HTTP/1
//! [SRC-484] dengan tanda tangan header SigV4.
//!
//! Batasan saat ini (gagal-tertutup): hanya endpoint `http://`, yang hanya diterima di lingkungan lokal
//! (KEPUTUSAN-235). Endpoint `https://` (NEO di staging/production) ditolak saat konstruksi dengan
//! [`ObjectStorageError::TlsUnsupported`] sampai klien TLS diterapkan (TLS antar-host: docs/05-stack.md §2.1).
//! Isi respons penyedia tidak pernah dibawa ke galat atau log.

use crate::config::{ObjectStorageConfig, Scheme};
use crate::error::ObjectStorageError;
use crate::presign::{Presigner, Target};
use crate::xml::{VersionEntry, parse_version_page};
use bytes::Bytes;
use http::{Method, Request, StatusCode, header};
use http_body_util::{BodyExt, Full, LengthLimitError, Limited};
use hyper_util::rt::TokioIo;
use std::fmt;
use std::time::Duration;
use time::OffsetDateTime;
use tokio::net::TcpStream;

/// Batas ukuran respons daftar versi yang dibaca (byte).
const MAX_LIST_RESPONSE_BYTES: usize = 1024 * 1024;
/// Batas jumlah halaman daftar versi per penghapusan (pelindung putaran tanpa akhir).
const MAX_LIST_PAGES: usize = 100;

/// Hasil `GET` terbatas.
#[derive(Clone, PartialEq, Eq)]
pub enum GetOutcome {
    /// Objek tidak ada.
    NotFound,
    /// Isi melebihi batas baca.
    TooLarge,
    /// Isi objek.
    Body(Vec<u8>),
}

impl fmt::Debug for GetOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound => f.write_str("NotFound"),
            Self::TooLarge => f.write_str("TooLarge"),
            Self::Body(b) => write!(f, "Body({} byte)", b.len()),
        }
    }
}

/// Klien object storage: penanda tangan + operasi server.
#[derive(Debug)]
pub struct ObjectStore {
    presigner: Presigner,
    timeout: Duration,
}

/// Satu permintaan server.
struct Op<'a> {
    method: Method,
    target: &'a Target,
    query: &'a [(&'a str, &'a str)],
    content_type: Option<&'a str>,
    body: Vec<u8>,
    max_response: usize,
}

impl<'a> Op<'a> {
    fn new(method: Method, target: &'a Target) -> Self {
        Self {
            method,
            target,
            query: &[],
            content_type: None,
            body: Vec::new(),
            max_response: 64 * 1024,
        }
    }
}

struct RawResponse {
    status: StatusCode,
    content_length: Option<u64>,
    body: Vec<u8>,
    /// Isi sukses melebihi `max_response` (sisanya tidak dibaca).
    over_limit: bool,
}

impl ObjectStore {
    /// Memvalidasi konfigurasi. Endpoint `https://` ditolak (lihat dokumentasi modul).
    pub fn new(config: &ObjectStorageConfig) -> Result<Self, ObjectStorageError> {
        let presigner = Presigner::new(config)?;
        if presigner.endpoint.scheme != Scheme::Http {
            return Err(ObjectStorageError::TlsUnsupported);
        }
        if config.timeout.is_zero() {
            return Err(ObjectStorageError::InvalidConfig("timeout"));
        }
        Ok(Self {
            presigner,
            timeout: config.timeout,
        })
    }

    /// Penanda tangan URL unggah.
    pub fn presigner(&self) -> &Presigner {
        &self.presigner
    }

    async fn send(
        &self,
        op: Op<'_>,
        now: OffsetDateTime,
    ) -> Result<RawResponse, ObjectStorageError> {
        match tokio::time::timeout(self.timeout, self.send_inner(op, now)).await {
            Ok(r) => r,
            Err(_) => Err(ObjectStorageError::Timeout),
        }
    }

    async fn send_inner(
        &self,
        op: Op<'_>,
        now: OffsetDateTime,
    ) -> Result<RawResponse, ObjectStorageError> {
        let Op {
            method,
            target,
            query,
            content_type,
            body,
            max_response,
        } = op;
        let signed = self.presigner.sign_request(
            method.as_str(),
            target,
            query,
            content_type,
            &body,
            now,
        )?;
        let cq = crate::sigv4::canonical_query(query);
        let path = if cq.is_empty() {
            target.canonical_uri.clone()
        } else {
            format!("{}?{cq}", target.canonical_uri)
        };
        let port = self.presigner.endpoint.connect_port();
        let stream = TcpStream::connect((target.connect_host.as_str(), port))
            .await
            .map_err(|_| ObjectStorageError::Unavailable)?;
        let (mut sender, conn) = hyper::client::conn::http1::handshake(TokioIo::new(stream))
            .await
            .map_err(|_| ObjectStorageError::Unavailable)?;
        let driver = tokio::spawn(conn);

        let is_head = method == Method::HEAD;
        let mut builder = Request::builder()
            .method(method)
            .uri(path)
            .header(header::CONNECTION, "close")
            .header(header::CONTENT_LENGTH, body.len());
        for (name, value) in signed {
            builder = builder.header(name, value);
        }
        let request = builder
            .body(Full::new(Bytes::from(body)))
            .map_err(|_| ObjectStorageError::InvalidInput("request"))?;

        let result = async {
            let response = sender
                .send_request(request)
                .await
                .map_err(|_| ObjectStorageError::Unavailable)?;
            let status = response.status();
            let content_length = response
                .headers()
                .get(header::CONTENT_LENGTH)
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok());
            // Isi galat penyedia tidak dibaca (tidak pernah dibawa ke galat atau log).
            let (body, over_limit) = if is_head || !status.is_success() {
                (Vec::new(), false)
            } else {
                match Limited::new(response.into_body(), max_response)
                    .collect()
                    .await
                {
                    Ok(c) => (c.to_bytes().to_vec(), false),
                    Err(e) if e.downcast_ref::<LengthLimitError>().is_some() => (Vec::new(), true),
                    Err(_) => return Err(ObjectStorageError::Unavailable),
                }
            };
            Ok(RawResponse {
                status,
                content_length,
                body,
                over_limit,
            })
        }
        .await;
        driver.abort();
        result
    }

    /// Ukuran objek dari `HEAD` (`None` bila objek tidak ada).
    pub async fn head_object(
        &self,
        bucket: &str,
        key: &str,
        now: OffsetDateTime,
    ) -> Result<Option<u64>, ObjectStorageError> {
        let target = self.presigner.target(bucket, Some(key))?;
        let r = self
            .send(
                Op {
                    max_response: 0,
                    ..Op::new(Method::HEAD, &target)
                },
                now,
            )
            .await?;
        match r.status {
            StatusCode::NOT_FOUND => Ok(None),
            s if s.is_success() => r
                .content_length
                .map(Some)
                .ok_or(ObjectStorageError::InvalidResponse),
            s => Err(ObjectStorageError::Rejected { status: s.as_u16() }),
        }
    }

    /// `GET` dengan batas baca `max_bytes`; isi lebih besar menghasilkan [`GetOutcome::TooLarge`] tanpa
    /// menyimpan sisanya.
    pub async fn get_object(
        &self,
        bucket: &str,
        key: &str,
        max_bytes: usize,
        now: OffsetDateTime,
    ) -> Result<GetOutcome, ObjectStorageError> {
        let target = self.presigner.target(bucket, Some(key))?;
        // Baca satu byte lebih dari batas supaya "tepat batas" dan "lebih dari batas" dapat dibedakan.
        let limit = max_bytes.saturating_add(1);
        let r = self
            .send(
                Op {
                    max_response: limit,
                    ..Op::new(Method::GET, &target)
                },
                now,
            )
            .await?;
        match r.status {
            StatusCode::NOT_FOUND => Ok(GetOutcome::NotFound),
            s if s.is_success() => {
                if r.over_limit || r.body.len() > max_bytes {
                    Ok(GetOutcome::TooLarge)
                } else {
                    Ok(GetOutcome::Body(r.body))
                }
            }
            s => Err(ObjectStorageError::Rejected { status: s.as_u16() }),
        }
    }

    /// `PUT` objek dengan isi dan tipe isi.
    pub async fn put_object(
        &self,
        bucket: &str,
        key: &str,
        content_type: &str,
        body: Vec<u8>,
        now: OffsetDateTime,
    ) -> Result<(), ObjectStorageError> {
        let target = self.presigner.target(bucket, Some(key))?;
        let r = self
            .send(
                Op {
                    content_type: Some(content_type),
                    body,
                    ..Op::new(Method::PUT, &target)
                },
                now,
            )
            .await?;
        if r.status.is_success() {
            Ok(())
        } else {
            Err(ObjectStorageError::Rejected {
                status: r.status.as_u16(),
            })
        }
    }

    /// Semua versi dan penanda hapus milik kunci persis `key` (`ListObjectVersions` dengan prefiks,
    /// berhalaman) [SRC-553].
    pub async fn list_versions(
        &self,
        bucket: &str,
        key: &str,
        now: OffsetDateTime,
    ) -> Result<Vec<String>, ObjectStorageError> {
        let target = self.presigner.target(bucket, None)?;
        let mut out = Vec::new();
        let mut key_marker: Option<String> = None;
        let mut version_marker: Option<String> = None;
        for _ in 0..MAX_LIST_PAGES {
            let mut query: Vec<(&str, &str)> = vec![("versions", ""), ("prefix", key)];
            if let Some(m) = key_marker.as_deref() {
                query.push(("key-marker", m));
            }
            if let Some(m) = version_marker.as_deref() {
                query.push(("version-id-marker", m));
            }
            let r = self
                .send(
                    Op {
                        query: &query,
                        max_response: MAX_LIST_RESPONSE_BYTES,
                        ..Op::new(Method::GET, &target)
                    },
                    now,
                )
                .await?;
            if !r.status.is_success() {
                return Err(ObjectStorageError::Rejected {
                    status: r.status.as_u16(),
                });
            }
            if r.over_limit {
                return Err(ObjectStorageError::InvalidResponse);
            }
            let text =
                std::str::from_utf8(&r.body).map_err(|_| ObjectStorageError::InvalidResponse)?;
            let page = parse_version_page(text)?;
            out.extend(
                page.entries
                    .into_iter()
                    .filter(|VersionEntry { key: k, .. }| k == key)
                    .map(|e| e.version_id),
            );
            if !page.is_truncated {
                return Ok(out);
            }
            if page.next_key_marker.is_none() && page.next_version_id_marker.is_none() {
                return Err(ObjectStorageError::InvalidResponse);
            }
            key_marker = page.next_key_marker;
            version_marker = page.next_version_id_marker;
        }
        Err(ObjectStorageError::InvalidResponse)
    }

    /// Menghapus objek beserta seluruh versi dan penanda hapusnya (ADR-018 butir 2 dan 5), lalu memastikan
    /// tidak ada versi tersisa. Idempoten: kunci tanpa versi tidak dianggap galat.
    pub async fn delete_all_versions(
        &self,
        bucket: &str,
        key: &str,
        now: OffsetDateTime,
    ) -> Result<(), ObjectStorageError> {
        let target = self.presigner.target(bucket, Some(key))?;
        for version_id in self.list_versions(bucket, key, now).await? {
            let r = self
                .send(
                    Op {
                        query: &[("versionId", version_id.as_str())],
                        ..Op::new(Method::DELETE, &target)
                    },
                    now,
                )
                .await?;
            if !(r.status.is_success() || r.status == StatusCode::NOT_FOUND) {
                return Err(ObjectStorageError::Rejected {
                    status: r.status.as_u16(),
                });
            }
        }
        if self.list_versions(bucket, key, now).await?.is_empty() {
            Ok(())
        } else {
            Err(ObjectStorageError::VersionsRemain)
        }
    }

    /// Membuat bucket. Hanya untuk penyiapan lingkungan lokal dan test; bucket staging/production dibuat
    /// lewat infrastruktur.
    pub async fn create_bucket(
        &self,
        bucket: &str,
        now: OffsetDateTime,
    ) -> Result<(), ObjectStorageError> {
        let target = self.presigner.target(bucket, None)?;
        let r = self.send(Op::new(Method::PUT, &target), now).await?;
        if r.status.is_success() {
            Ok(())
        } else {
            Err(ObjectStorageError::Rejected {
                status: r.status.as_u16(),
            })
        }
    }

    /// Mengaktifkan versi objek pada bucket (ADR-018 butir 5) [SRC-553]. Hanya untuk penyiapan lingkungan
    /// lokal dan test.
    pub async fn enable_versioning(
        &self,
        bucket: &str,
        now: OffsetDateTime,
    ) -> Result<(), ObjectStorageError> {
        let target = self.presigner.target(bucket, None)?;
        let body = br#"<VersioningConfiguration xmlns="http://s3.amazonaws.com/doc/2006-03-01/"><Status>Enabled</Status></VersioningConfiguration>"#.to_vec();
        let r = self
            .send(
                Op {
                    query: &[("versioning", "")],
                    content_type: Some("application/xml"),
                    body,
                    ..Op::new(Method::PUT, &target)
                },
                now,
            )
            .await?;
        if r.status.is_success() {
            Ok(())
        } else {
            Err(ObjectStorageError::Rejected {
                status: r.status.as_u16(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AddressingStyle, Deployment};
    use foundation::Sensitive;

    fn cfg(endpoint: &str, deployment: Deployment) -> ObjectStorageConfig {
        ObjectStorageConfig {
            endpoint: endpoint.to_string(),
            region: "us-east-1".to_string(),
            addressing: AddressingStyle::Path,
            access_key_id: "uji-akses".to_string(),
            secret_access_key: Sensitive::new("r".repeat(40)),
            deployment,
            timeout: Duration::from_secs(1),
        }
    }

    #[test]
    fn https_endpoint_is_rejected_until_tls_is_implemented() {
        assert_eq!(
            ObjectStore::new(&cfg("https://nos.jkt-1.neo.id", Deployment::Production)).err(),
            Some(ObjectStorageError::TlsUnsupported)
        );
        assert_eq!(
            ObjectStore::new(&cfg("http://nos.jkt-1.neo.id", Deployment::Production)).err(),
            Some(ObjectStorageError::InsecureEndpoint)
        );
        assert!(ObjectStore::new(&cfg("http://127.0.0.1:18333", Deployment::Local)).is_ok());
        let mut zero = cfg("http://127.0.0.1:18333", Deployment::Local);
        zero.timeout = Duration::ZERO;
        assert_eq!(
            ObjectStore::new(&zero).err(),
            Some(ObjectStorageError::InvalidConfig("timeout"))
        );
    }

    #[tokio::test]
    async fn unreachable_endpoint_is_unavailable() {
        // Port 1 di loopback tidak mendengarkan.
        let store = ObjectStore::new(&cfg("http://127.0.0.1:1", Deployment::Local)).unwrap();
        let now = time::macros::datetime!(2026-11-20 03:04:05 UTC);
        assert_eq!(
            store.head_object("rs-uji", "k", now).await,
            Err(ObjectStorageError::Unavailable)
        );
    }
}
