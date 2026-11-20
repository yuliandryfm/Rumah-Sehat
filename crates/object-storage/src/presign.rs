//! URL unggah bertanda tangan (ADR-018 butir 2, KEPUTUSAN-234) dan tanda tangan header untuk operasi server.
//!
//! - Presigned POST dengan policy: `content-length-range` 1..=batas, `Content-Type` dan `key` persis, umur
//!   URL di `expiration` [SRC-551]; ditegakkan emulator [SRC-552]. Jalur utama KEPUTUSAN-234.
//! - Presigned PUT: `content-length`, `content-type`, dan `host` ikut ditandatangani, sehingga klien wajib
//!   mengirim ukuran dan tipe persis seperti yang diminta (ditegakkan penyedia; dibuktikan pada emulator di
//!   spike F1-T03 §3). Jalur alternatif KEPUTUSAN-234.
//!
//! - Presigned GET (unduh, F1-T30, ADR-018 butir 3): hanya `host` yang ditandatangani; header respons
//!   (`Content-Type` tervalidasi, `Content-Disposition: attachment`, `Cache-Control`) ditimpa lewat parameter
//!   `response-*` yang ikut ditandatangani [SRC-563]; vektor uji dari botocore [SRC-564].
//!
//! Pembatasan URL hanya lapis pertama; validasi pasca-unggah tetap wajib (KEPUTUSAN-234).

use crate::config::{
    AddressingStyle, Endpoint, ObjectStorageConfig, parse_endpoint, valid_access_key_id,
    valid_bucket_name, valid_object_key, valid_region, valid_secret,
};
use crate::error::ObjectStorageError;
use crate::sigv4::{
    ALGORITHM, AmzTime, Credentials, SigningKey, UNSIGNED_PAYLOAD, canonical_query,
    canonical_request, credential_scope, hex, string_to_sign, uri_encode,
};
use base64::Engine as _;
use foundation::Sensitive;
use std::fmt;
use std::time::Duration;
use time::OffsetDateTime;

/// Umur maksimum URL presigned SigV4 (7 hari) [SRC-550].
pub const MAX_PRESIGN_SECONDS: u64 = 604_800;

/// URL presigned PUT beserta header yang wajib dikirim klien persis.
#[derive(Clone, PartialEq, Eq)]
pub struct PresignedPut {
    /// URL bertanda tangan (rahasia berumur pendek; jangan dicatat di log).
    pub url: String,
    /// Header wajib: `Content-Type` dan `Content-Length`.
    pub headers: Vec<(String, String)>,
    /// Instan kedaluwarsa URL.
    pub expires_at: OffsetDateTime,
}

impl fmt::Debug for PresignedPut {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PresignedPut")
            .field("url", &"[REDACTED]")
            .field("expires_at", &self.expires_at)
            .finish()
    }
}

/// Formulir presigned POST: kirim `multipart/form-data` ke `url` dengan semua `fields`, lalu berkas
/// sebagai field `file` terakhir [SRC-551].
#[derive(Clone, PartialEq, Eq)]
pub struct PresignedPost {
    /// URL tujuan formulir.
    pub url: String,
    /// Field formulir (urutan bebas, sebelum `file`).
    pub fields: Vec<(String, String)>,
    /// Instan kedaluwarsa policy.
    pub expires_at: OffsetDateTime,
}

impl fmt::Debug for PresignedPost {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PresignedPost")
            .field("url", &self.url)
            .field("fields", &"[REDACTED]")
            .field("expires_at", &self.expires_at)
            .finish()
    }
}

/// Header respons yang ditimpa lewat URL unduh bertanda tangan (`response-*` GetObject) [SRC-563].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResponseHeaders<'a> {
    /// `response-content-type`: tipe isi tervalidasi (bukan tipe yang tersimpan di penyedia).
    pub content_type: &'a str,
    /// `response-content-disposition`, misalnya `attachment; filename="<id>.jpg"` (ADR-018 butir 3).
    pub content_disposition: &'a str,
    /// `response-cache-control`, misalnya `private, no-store`.
    pub cache_control: &'a str,
}

/// URL unduh bertanda tangan (GET presigned).
#[derive(Clone, PartialEq, Eq)]
pub struct PresignedGet {
    /// URL bertanda tangan (rahasia berumur pendek; jangan dicatat di log).
    pub url: String,
    /// Instan kedaluwarsa URL.
    pub expires_at: OffsetDateTime,
}

impl fmt::Debug for PresignedGet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PresignedGet")
            .field("url", &"[REDACTED]")
            .field("expires_at", &self.expires_at)
            .finish()
    }
}

/// Nilai header bebas yang aman ditandatangani: ASCII tampak + spasi, 1–200 karakter.
fn valid_header_value(v: &str) -> bool {
    (1..=200).contains(&v.len()) && v.bytes().all(|b| (0x20..=0x7E).contains(&b))
}

/// Penanda tangan SigV4 untuk satu endpoint dan kredensial. Murni (tanpa I/O), sehingga juga dapat dipakai
/// untuk endpoint `https://`.
pub struct Presigner {
    pub(crate) endpoint: Endpoint,
    addressing: AddressingStyle,
    region: String,
    creds: Credentials,
}

impl fmt::Debug for Presigner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Presigner")
            .field("endpoint", &self.endpoint)
            .field("addressing", &self.addressing)
            .field("region", &self.region)
            .field("credentials", &"[REDACTED]")
            .finish()
    }
}

/// Sasaran permintaan: nilai header `Host`, URI kanonik, dan URL lengkap tanpa query.
pub(crate) struct Target {
    /// Nama host untuk koneksi TCP (tanpa port).
    pub(crate) connect_host: String,
    pub(crate) host: String,
    pub(crate) canonical_uri: String,
    pub(crate) url: String,
}

/// Tipe isi `type/subtype` sederhana (`[a-z0-9.+-]`), maksimal 100 karakter.
fn valid_content_type(ct: &str) -> bool {
    let Some((t, s)) = ct.split_once('/') else {
        return false;
    };
    let part = |p: &str| {
        !p.is_empty()
            && p.bytes().all(|b| {
                b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'.' | b'+' | b'-')
            })
    };
    ct.len() <= 100 && part(t) && part(s)
}

fn expiry(now: OffsetDateTime, expires_in: Duration) -> Result<OffsetDateTime, ObjectStorageError> {
    let secs = expires_in.as_secs();
    if secs == 0 || secs > MAX_PRESIGN_SECONDS || expires_in.subsec_nanos() != 0 {
        return Err(ObjectStorageError::InvalidInput("expires_in"));
    }
    let secs = i64::try_from(secs).map_err(|_| ObjectStorageError::InvalidInput("expires_in"))?;
    now.checked_add(time::Duration::seconds(secs))
        .ok_or(ObjectStorageError::InvalidInput("expires_in"))
}

fn iso8601_millis_utc(t: OffsetDateTime) -> String {
    let u = t.to_offset(time::UtcOffset::UTC);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.000Z",
        u.year(),
        u8::from(u.month()),
        u.day(),
        u.hour(),
        u.minute(),
        u.second()
    )
}

impl Presigner {
    /// Memvalidasi konfigurasi (endpoint, region, kredensial, gaya alamat).
    pub fn new(config: &ObjectStorageConfig) -> Result<Self, ObjectStorageError> {
        let endpoint = parse_endpoint(&config.endpoint, config.deployment)?;
        if config.addressing == AddressingStyle::VirtualHost && endpoint.is_ip {
            return Err(ObjectStorageError::InvalidConfig("addressing"));
        }
        if !valid_region(&config.region) {
            return Err(ObjectStorageError::InvalidConfig("region"));
        }
        if !valid_access_key_id(&config.access_key_id) {
            return Err(ObjectStorageError::InvalidConfig("access_key_id"));
        }
        if !valid_secret(config.secret_access_key.expose()) {
            return Err(ObjectStorageError::InvalidConfig("secret_access_key"));
        }
        Ok(Self {
            endpoint,
            addressing: config.addressing,
            region: config.region.clone(),
            creds: Credentials {
                access_key_id: config.access_key_id.clone(),
                secret: Sensitive::new(config.secret_access_key.expose().clone()),
            },
        })
    }

    /// Sasaran untuk bucket dan (opsional) kunci objek.
    pub(crate) fn target(
        &self,
        bucket: &str,
        key: Option<&str>,
    ) -> Result<Target, ObjectStorageError> {
        if !valid_bucket_name(bucket) {
            return Err(ObjectStorageError::InvalidInput("bucket"));
        }
        if let Some(k) = key
            && !valid_object_key(k)
        {
            return Err(ObjectStorageError::InvalidInput("key"));
        }
        let scheme = self.endpoint.scheme.as_str();
        let encoded_key = key.map(|k| uri_encode(k, true));
        Ok(match self.addressing {
            AddressingStyle::Path => {
                let host = self.endpoint.authority(&self.endpoint.host);
                let canonical_uri = match &encoded_key {
                    Some(k) => format!("/{bucket}/{k}"),
                    None => format!("/{bucket}"),
                };
                Target {
                    connect_host: self.endpoint.host.clone(),
                    url: format!("{scheme}://{host}{canonical_uri}"),
                    host,
                    canonical_uri,
                }
            }
            AddressingStyle::VirtualHost => {
                let connect_host = format!("{bucket}.{}", self.endpoint.host);
                let host = self.endpoint.authority(&connect_host);
                let canonical_uri = match &encoded_key {
                    Some(k) => format!("/{k}"),
                    None => "/".to_string(),
                };
                Target {
                    connect_host,
                    url: format!("{scheme}://{host}{canonical_uri}"),
                    host,
                    canonical_uri,
                }
            }
        })
    }

    fn signing_key(&self, time: &AmzTime) -> Result<SigningKey, ObjectStorageError> {
        SigningKey::derive(&self.creds, &time.date, &self.region)
    }

    fn credential(&self, time: &AmzTime) -> String {
        format!(
            "{}/{}",
            self.creds.access_key_id,
            credential_scope(&time.date, &self.region)
        )
    }

    /// Presigned PUT yang mengikat `Content-Type` dan `Content-Length` (KEPUTUSAN-234 jalur 2).
    pub fn presign_put(
        &self,
        bucket: &str,
        key: &str,
        content_type: &str,
        content_length: u64,
        now: OffsetDateTime,
        expires_in: Duration,
    ) -> Result<PresignedPut, ObjectStorageError> {
        if !valid_content_type(content_type) {
            return Err(ObjectStorageError::InvalidInput("content_type"));
        }
        if content_length == 0 {
            return Err(ObjectStorageError::InvalidInput("content_length"));
        }
        let expires_at = expiry(now, expires_in)?;
        let target = self.target(bucket, Some(key))?;
        let t = AmzTime::new(now);
        let credential = self.credential(&t);
        let expires = expires_in.as_secs().to_string();
        let length = content_length.to_string();
        let headers = [
            ("content-length", length.as_str()),
            ("content-type", content_type),
            ("host", target.host.as_str()),
        ];
        let signed_headers = "content-length;content-type;host";
        let params = [
            ("X-Amz-Algorithm", ALGORITHM),
            ("X-Amz-Credential", credential.as_str()),
            ("X-Amz-Date", t.datetime.as_str()),
            ("X-Amz-Expires", expires.as_str()),
            ("X-Amz-SignedHeaders", signed_headers),
        ];
        let query = canonical_query(&params);
        let (request, _) = canonical_request(
            "PUT",
            &target.canonical_uri,
            &query,
            &headers,
            UNSIGNED_PAYLOAD,
        );
        let sts = string_to_sign(
            &t.datetime,
            &credential_scope(&t.date, &self.region),
            &request,
        );
        let signature = self.signing_key(&t)?.sign_hex(&sts);
        Ok(PresignedPut {
            // Tanda tangan di akhir, seperti keluaran botocore [SRC-551]; urutan query tidak memengaruhi
            // verifikasi karena penyedia menyusun ulang query kanonik.
            url: format!("{}?{query}&X-Amz-Signature={signature}", target.url),
            headers: vec![
                ("Content-Type".to_string(), content_type.to_string()),
                ("Content-Length".to_string(), length),
            ],
            expires_at,
        })
    }

    /// Presigned POST dengan policy: kunci dan `Content-Type` persis, ukuran 1..=`max_length` byte
    /// (KEPUTUSAN-234 jalur 1).
    pub fn presign_post(
        &self,
        bucket: &str,
        key: &str,
        content_type: &str,
        max_length: u64,
        now: OffsetDateTime,
        expires_in: Duration,
    ) -> Result<PresignedPost, ObjectStorageError> {
        if !valid_content_type(content_type) {
            return Err(ObjectStorageError::InvalidInput("content_type"));
        }
        if max_length == 0 {
            return Err(ObjectStorageError::InvalidInput("max_length"));
        }
        let expires_at = expiry(now, expires_in)?;
        // Validasi bucket dan kunci; URL formulir menunjuk bucket (tanpa kunci).
        self.target(bucket, Some(key))?;
        let target = self.target(bucket, None)?;
        let t = AmzTime::new(now);
        let credential = self.credential(&t);
        let policy = serde_json::json!({
            "expiration": iso8601_millis_utc(expires_at),
            "conditions": [
                { "bucket": bucket },
                ["eq", "$key", key],
                ["eq", "$Content-Type", content_type],
                ["content-length-range", 1, max_length],
                { "x-amz-algorithm": ALGORITHM },
                { "x-amz-credential": credential },
                { "x-amz-date": t.datetime },
            ]
        });
        let policy_b64 = base64::engine::general_purpose::STANDARD.encode(policy.to_string());
        let signature = self.signing_key(&t)?.sign_hex(&policy_b64);
        Ok(PresignedPost {
            url: target.url,
            fields: vec![
                ("key".to_string(), key.to_string()),
                ("Content-Type".to_string(), content_type.to_string()),
                ("x-amz-algorithm".to_string(), ALGORITHM.to_string()),
                ("x-amz-credential".to_string(), credential),
                ("x-amz-date".to_string(), t.datetime),
                ("policy".to_string(), policy_b64),
                ("x-amz-signature".to_string(), signature),
            ],
            expires_at,
        })
    }

    /// Presigned GET untuk unduhan (ADR-018 butir 3): hanya header `host` yang ditandatangani; header
    /// respons ditimpa lewat parameter `response-*` yang ikut ditandatangani, sehingga tidak dapat
    /// diubah pemegang URL [SRC-563].
    pub fn presign_get(
        &self,
        bucket: &str,
        key: &str,
        response: &ResponseHeaders<'_>,
        now: OffsetDateTime,
        expires_in: Duration,
    ) -> Result<PresignedGet, ObjectStorageError> {
        if !valid_content_type(response.content_type) {
            return Err(ObjectStorageError::InvalidInput("content_type"));
        }
        if !valid_header_value(response.content_disposition) {
            return Err(ObjectStorageError::InvalidInput("content_disposition"));
        }
        if !valid_header_value(response.cache_control) {
            return Err(ObjectStorageError::InvalidInput("cache_control"));
        }
        let expires_at = expiry(now, expires_in)?;
        let target = self.target(bucket, Some(key))?;
        let t = AmzTime::new(now);
        let credential = self.credential(&t);
        let expires = expires_in.as_secs().to_string();
        let params = [
            ("X-Amz-Algorithm", ALGORITHM),
            ("X-Amz-Credential", credential.as_str()),
            ("X-Amz-Date", t.datetime.as_str()),
            ("X-Amz-Expires", expires.as_str()),
            ("X-Amz-SignedHeaders", "host"),
            ("response-cache-control", response.cache_control),
            ("response-content-disposition", response.content_disposition),
            ("response-content-type", response.content_type),
        ];
        let query = canonical_query(&params);
        let (request, _) = canonical_request(
            "GET",
            &target.canonical_uri,
            &query,
            &[("host", target.host.as_str())],
            UNSIGNED_PAYLOAD,
        );
        let sts = string_to_sign(
            &t.datetime,
            &credential_scope(&t.date, &self.region),
            &request,
        );
        let signature = self.signing_key(&t)?.sign_hex(&sts);
        Ok(PresignedGet {
            url: format!("{}?{query}&X-Amz-Signature={signature}", target.url),
            expires_at,
        })
    }

    /// Header untuk permintaan server bertanda tangan (header `Authorization`, payload ikut di-hash).
    /// Mengembalikan pasangan header yang wajib dikirim bersama URL hasil [`Target`].
    pub(crate) fn sign_request(
        &self,
        method: &str,
        target: &Target,
        query: &[(&str, &str)],
        content_type: Option<&str>,
        payload: &[u8],
        now: OffsetDateTime,
    ) -> Result<Vec<(&'static str, String)>, ObjectStorageError> {
        let t = AmzTime::new(now);
        let payload_hash = hex(&crypto::sha256(payload));
        let mut headers: Vec<(&str, &str)> = vec![
            ("host", target.host.as_str()),
            ("x-amz-content-sha256", payload_hash.as_str()),
            ("x-amz-date", t.datetime.as_str()),
        ];
        if let Some(ct) = content_type {
            headers.push(("content-type", ct));
        }
        let cq = canonical_query(query);
        let (request, signed_headers) =
            canonical_request(method, &target.canonical_uri, &cq, &headers, &payload_hash);
        let scope = credential_scope(&t.date, &self.region);
        let sts = string_to_sign(&t.datetime, &scope, &request);
        let signature = self.signing_key(&t)?.sign_hex(&sts);
        let authorization = format!(
            "{ALGORITHM} Credential={}/{scope}, SignedHeaders={signed_headers}, Signature={signature}",
            self.creds.access_key_id
        );
        let mut out = vec![
            ("host", target.host.clone()),
            ("x-amz-content-sha256", payload_hash.clone()),
            ("x-amz-date", t.datetime.clone()),
            ("authorization", authorization),
        ];
        if let Some(ct) = content_type {
            out.push(("content-type", ct.to_string()));
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Deployment;

    // Kredensial contoh fiktif dokumentasi AWS; vektor dihasilkan botocore 1.40.72 [SRC-551]
    // (skrip: `botocore.auth.S3SigV4QueryAuth` / `S3SigV4Auth` / `S3SigV4PostAuth.signature`, waktu
    // 2026-11-20T03:04:05Z).
    const AK: &str = "AKIAIOSFODNN7EXAMPLE";
    const SK: &str = "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY";
    const KEY: &str = "0193a1b2-c3d4-7e5f-8a9b-0c1d2e3f4a5b";

    fn presigner(endpoint: &str, region: &str, addressing: AddressingStyle) -> Presigner {
        let deployment = if endpoint.starts_with("http://") {
            Deployment::Local
        } else {
            Deployment::Production
        };
        Presigner::new(&ObjectStorageConfig {
            endpoint: endpoint.to_string(),
            region: region.to_string(),
            addressing,
            access_key_id: AK.to_string(),
            secret_access_key: Sensitive::new(SK.to_string()),
            deployment,
            timeout: Duration::from_secs(5),
        })
        .unwrap()
    }

    fn at() -> OffsetDateTime {
        time::macros::datetime!(2026-11-20 03:04:05 UTC)
    }

    #[test]
    fn presigned_put_path_style_matches_botocore() {
        let p = presigner("http://127.0.0.1:18333", "us-east-1", AddressingStyle::Path);
        let put = p
            .presign_put(
                "rs-identity-document",
                KEY,
                "image/jpeg",
                1234,
                at(),
                Duration::from_secs(600),
            )
            .unwrap();
        assert_eq!(
            put.url,
            "http://127.0.0.1:18333/rs-identity-document/0193a1b2-c3d4-7e5f-8a9b-0c1d2e3f4a5b?X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential=AKIAIOSFODNN7EXAMPLE%2F20261120%2Fus-east-1%2Fs3%2Faws4_request&X-Amz-Date=20261120T030405Z&X-Amz-Expires=600&X-Amz-SignedHeaders=content-length%3Bcontent-type%3Bhost&X-Amz-Signature=fdf63953131e45bd4b17db3c4f4fe5cb691dab563cbc6d1e4978edbb143619a7"
        );
        assert_eq!(
            put.expires_at,
            time::macros::datetime!(2026-11-20 03:14:05 UTC)
        );
        assert!(
            put.headers
                .contains(&("Content-Length".to_string(), "1234".to_string()))
        );
        assert!(!format!("{put:?}").contains("Signature"));
    }

    #[test]
    fn presigned_put_virtual_host_https_matches_botocore() {
        let p = presigner(
            "https://nos.jkt-1.neo.id",
            "id-jkt-1",
            AddressingStyle::VirtualHost,
        );
        let put = p
            .presign_put(
                "rs-identity-document",
                KEY,
                "image/png",
                5_242_880,
                at(),
                Duration::from_secs(600),
            )
            .unwrap();
        assert_eq!(
            put.url,
            "https://rs-identity-document.nos.jkt-1.neo.id/0193a1b2-c3d4-7e5f-8a9b-0c1d2e3f4a5b?X-Amz-Algorithm=AWS4-HMAC-SHA256&X-Amz-Credential=AKIAIOSFODNN7EXAMPLE%2F20261120%2Fid-jkt-1%2Fs3%2Faws4_request&X-Amz-Date=20261120T030405Z&X-Amz-Expires=600&X-Amz-SignedHeaders=content-length%3Bcontent-type%3Bhost&X-Amz-Signature=32a3edb6e8fc6eb728b38dc6c73f0f95fb8c749ed11d04f6f9d97099da676a86"
        );
    }

    #[test]
    fn post_policy_signature_matches_botocore() {
        let p = presigner("http://127.0.0.1:18333", "us-east-1", AddressingStyle::Path);
        let key = p.signing_key(&AmzTime::new(at())).unwrap();
        assert_eq!(
            key.sign_hex("eyJleGFtcGxlIjoicG9saWN5In0="),
            "adbafbb9ed9f2e2e8f7a4fc9e60b4a523f5cdce1f7287f95980c1cc9c8e0e33c"
        );
    }

    #[test]
    fn presigned_post_policy_binds_key_type_and_size() {
        let p = presigner("http://127.0.0.1:18333", "us-east-1", AddressingStyle::Path);
        let post = p
            .presign_post(
                "rs-identity-document",
                KEY,
                "image/jpeg",
                5_242_880,
                at(),
                Duration::from_secs(600),
            )
            .unwrap();
        assert_eq!(post.url, "http://127.0.0.1:18333/rs-identity-document");
        let field = |n: &str| {
            post.fields
                .iter()
                .find(|(k, _)| k == n)
                .map(|(_, v)| v.clone())
                .unwrap()
        };
        let policy = base64::engine::general_purpose::STANDARD
            .decode(field("policy"))
            .unwrap();
        let policy: serde_json::Value = serde_json::from_slice(&policy).unwrap();
        assert_eq!(policy["expiration"], "2026-11-20T03:14:05.000Z");
        let conditions = policy["conditions"].as_array().unwrap();
        assert!(conditions.contains(&serde_json::json!(["eq", "$key", KEY])));
        assert!(conditions.contains(&serde_json::json!(["eq", "$Content-Type", "image/jpeg"])));
        assert!(conditions.contains(&serde_json::json!(["content-length-range", 1, 5_242_880])));
        assert!(conditions.contains(&serde_json::json!({"bucket": "rs-identity-document"})));
        let key = p.signing_key(&AmzTime::new(at())).unwrap();
        assert_eq!(field("x-amz-signature"), key.sign_hex(&field("policy")));
        assert_eq!(
            field("x-amz-credential"),
            "AKIAIOSFODNN7EXAMPLE/20261120/us-east-1/s3/aws4_request"
        );
    }

    #[test]
    fn header_signed_requests_match_botocore() {
        let p = presigner("http://127.0.0.1:18333", "us-east-1", AddressingStyle::Path);
        let auth = |method: &str, key: Option<&str>, q: &[(&str, &str)], ct, body: &[u8]| {
            let t = p.target("rs-identity-document", key).unwrap();
            p.sign_request(method, &t, q, ct, body, at())
                .unwrap()
                .into_iter()
                .find(|(n, _)| *n == "authorization")
                .unwrap()
                .1
        };
        let prefix =
            "AWS4-HMAC-SHA256 Credential=AKIAIOSFODNN7EXAMPLE/20261120/us-east-1/s3/aws4_request, ";
        assert_eq!(
            auth("GET", Some(KEY), &[], None, b""),
            format!(
                "{prefix}SignedHeaders=host;x-amz-content-sha256;x-amz-date, Signature=064e8d929752d26df4649a58b0a76fbeca8d6a7028ae3d6eac3dd4ba8d863a1b"
            )
        );
        assert_eq!(
            auth("GET", None, &[("versions", ""), ("prefix", KEY)], None, b""),
            format!(
                "{prefix}SignedHeaders=host;x-amz-content-sha256;x-amz-date, Signature=59c7e002069dfb0479dd889397dd79ef0eb4de402d37dd77c8681a33a925c1c2"
            )
        );
        assert_eq!(
            auth("PUT", Some(KEY), &[], Some("image/png"), b"isi-uji"),
            format!(
                "{prefix}SignedHeaders=content-type;host;x-amz-content-sha256;x-amz-date, Signature=d7501357aeea7498af40c11f38f42e619421fe1f7a02253ace5d52de3bf21bbc"
            )
        );
        assert_eq!(
            auth(
                "DELETE",
                Some(KEY),
                &[("versionId", "3/L4kqtJlcpXroDTDmJ+rmSpXd3dIbrHY")],
                None,
                b""
            ),
            format!(
                "{prefix}SignedHeaders=host;x-amz-content-sha256;x-amz-date, Signature=34eb8a9d29599fc57c673c52e370199cbfca6ae0a7ef35d715f5a6899d79af5e"
            )
        );
    }

    #[test]
    fn invalid_presign_inputs_are_rejected() {
        let p = presigner("http://127.0.0.1:18333", "us-east-1", AddressingStyle::Path);
        let ok = Duration::from_secs(600);
        assert!(
            p.presign_put("rs-x", KEY, "image/jpeg", 0, at(), ok)
                .is_err()
        );
        assert!(
            p.presign_put("rs-x", KEY, "image/jpeg\r\nx: y", 1, at(), ok)
                .is_err()
        );
        assert!(
            p.presign_put("rs-x", "../k", "image/jpeg", 1, at(), ok)
                .is_err()
        );
        assert!(p.presign_put("RS", KEY, "image/jpeg", 1, at(), ok).is_err());
        assert!(
            p.presign_put("rs-x", KEY, "image/jpeg", 1, at(), Duration::ZERO)
                .is_err()
        );
        assert!(
            p.presign_put(
                "rs-x",
                KEY,
                "image/jpeg",
                1,
                at(),
                Duration::from_secs(MAX_PRESIGN_SECONDS + 1)
            )
            .is_err()
        );
        assert!(
            p.presign_post("rs-x", KEY, "image/jpeg", 0, at(), ok)
                .is_err()
        );
    }

    #[test]
    fn virtual_host_with_ip_endpoint_or_short_secret_is_rejected() {
        let cfg = |endpoint: &str, addressing, secret: &str| ObjectStorageConfig {
            endpoint: endpoint.to_string(),
            region: "us-east-1".to_string(),
            addressing,
            access_key_id: AK.to_string(),
            secret_access_key: Sensitive::new(secret.to_string()),
            deployment: Deployment::Local,
            timeout: Duration::from_secs(5),
        };
        assert_eq!(
            Presigner::new(&cfg("http://127.0.0.1:1", AddressingStyle::VirtualHost, SK)).err(),
            Some(ObjectStorageError::InvalidConfig("addressing"))
        );
        assert_eq!(
            Presigner::new(&cfg("http://127.0.0.1:1", AddressingStyle::Path, "pendek")).err(),
            Some(ObjectStorageError::InvalidConfig("secret_access_key"))
        );
        let dbg = format!(
            "{:?}",
            Presigner::new(&cfg("http://127.0.0.1:1", AddressingStyle::Path, SK)).unwrap()
        );
        assert!(!dbg.contains(SK) && !dbg.contains(AK));
    }

    // trace: F1-T30 (URL unduh: header respons lampiran ikut ditandatangani), vektor botocore [SRC-564]
    #[test]
    fn presigned_get_with_response_overrides_matches_botocore() {
        let headers = ResponseHeaders {
            content_type: "image/jpeg",
            content_disposition: "attachment; filename=\"0193a1b2-c3d4-7e5f-8a9b-0c1d2e3f4a5b.jpg\"",
            cache_control: "private, no-store",
        };
        let key = format!("files/{KEY}");
        for (endpoint, region, addressing, sig, base) in [
            (
                "http://127.0.0.1:18333",
                "us-east-1",
                AddressingStyle::Path,
                "87c36989d8aa3120c15e26f7f6566ae0cd2df776e32b56b29d5ef17dabe2b23b",
                "http://127.0.0.1:18333/rs-identity-document/files/0193a1b2-c3d4-7e5f-8a9b-0c1d2e3f4a5b?",
            ),
            (
                "https://nos.jkt-1.neo.id",
                "id-jkt-1",
                AddressingStyle::VirtualHost,
                "3c88e92f5925f71a7a3378d6718bee3af7e9d25d11c9f2c83cc62a286faca716",
                "https://rs-identity-document.nos.jkt-1.neo.id/files/0193a1b2-c3d4-7e5f-8a9b-0c1d2e3f4a5b?",
            ),
        ] {
            let p = presigner(endpoint, region, addressing);
            let get = p
                .presign_get(
                    "rs-identity-document",
                    &key,
                    &headers,
                    at(),
                    Duration::from_secs(300),
                )
                .unwrap();
            assert!(get.url.starts_with(base), "{}", get.url);
            assert!(
                get.url.ends_with(&format!("&X-Amz-Signature={sig}")),
                "{}",
                get.url
            );
            for part in [
                "response-cache-control=private%2C%20no-store",
                "response-content-disposition=attachment%3B%20filename%3D%220193a1b2-c3d4-7e5f-8a9b-0c1d2e3f4a5b.jpg%22",
                "response-content-type=image%2Fjpeg",
                "X-Amz-Expires=300",
                "X-Amz-SignedHeaders=host",
            ] {
                assert!(get.url.contains(part), "{part} tidak ada di {}", get.url);
            }
            assert_eq!(
                get.expires_at,
                time::macros::datetime!(2026-11-20 03:09:05 UTC)
            );
            assert!(!format!("{get:?}").contains(sig));
        }
    }

    #[test]
    fn presigned_get_rejects_unsafe_response_headers() {
        let p = presigner("http://127.0.0.1:18333", "us-east-1", AddressingStyle::Path);
        let ok = ResponseHeaders {
            content_type: "image/png",
            content_disposition: "attachment",
            cache_control: "no-store",
        };
        let ttl = Duration::from_secs(300);
        assert!(p.presign_get("rs-x", KEY, &ok, at(), ttl).is_ok());
        let long = "a".repeat(201);
        for (bad, field) in [
            (
                ResponseHeaders {
                    content_type: "text/html\r\nx: y",
                    ..ok
                },
                "content_type",
            ),
            (
                ResponseHeaders {
                    content_disposition: "attachment\r\nSet-Cookie: x=y",
                    ..ok
                },
                "content_disposition",
            ),
            (
                ResponseHeaders {
                    content_disposition: "",
                    ..ok
                },
                "content_disposition",
            ),
            (
                ResponseHeaders {
                    cache_control: long.as_str(),
                    ..ok
                },
                "cache_control",
            ),
        ] {
            assert_eq!(
                p.presign_get("rs-x", KEY, &bad, at(), ttl).err(),
                Some(ObjectStorageError::InvalidInput(field))
            );
        }
        assert!(
            p.presign_get("rs-x", KEY, &ok, at(), Duration::ZERO)
                .is_err()
        );
        assert!(p.presign_get("rs-x", "../k", &ok, at(), ttl).is_err());
    }
}
