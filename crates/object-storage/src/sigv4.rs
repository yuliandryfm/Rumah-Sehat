//! AWS Signature Version 4 untuk S3 [SRC-550]: permintaan kanonik, string yang ditandatangani, dan kunci
//! penandatangan turunan `HMAC("AWS4" + rahasia, tanggal) → region → "s3" → "aws4_request"`.
//!
//! Primitif HMAC-SHA256 dan SHA-256 berasal dari crate `crypto` (aws-lc-rs); modul ini hanya menyusun
//! string sesuai spesifikasi, tidak mengimplementasikan primitif kriptografi (coding-standards §5.5).
//! Vektor uji dihasilkan botocore 1.40.72 [SRC-551] untuk masukan yang sama.

use crate::error::ObjectStorageError;
use crypto::{HmacSha256Key, sha256};
use foundation::Sensitive;
use time::OffsetDateTime;

/// Algoritma SigV4.
pub(crate) const ALGORITHM: &str = "AWS4-HMAC-SHA256";
/// Nama layanan SigV4 untuk S3.
const SERVICE: &str = "s3";
/// Hash payload untuk URL presigned S3 [SRC-550].
pub(crate) const UNSIGNED_PAYLOAD: &str = "UNSIGNED-PAYLOAD";

/// Kredensial penanda tangan. Rahasia hanya keluar sebagai bahan kunci HMAC.
pub(crate) struct Credentials {
    pub(crate) access_key_id: String,
    pub(crate) secret: Sensitive<String>,
}

/// Waktu permintaan dalam dua bentuk SigV4.
pub(crate) struct AmzTime {
    /// `YYYYMMDD`.
    pub(crate) date: String,
    /// `YYYYMMDD'T'HHMMSS'Z'`.
    pub(crate) datetime: String,
}

impl AmzTime {
    pub(crate) fn new(at: OffsetDateTime) -> Self {
        let utc = at.to_offset(time::UtcOffset::UTC);
        let date = format!(
            "{:04}{:02}{:02}",
            utc.year(),
            u8::from(utc.month()),
            utc.day()
        );
        let datetime = format!(
            "{date}T{:02}{:02}{:02}Z",
            utc.hour(),
            utc.minute(),
            utc.second()
        );
        Self { date, datetime }
    }
}

/// Heksadesimal huruf kecil.
pub(crate) fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        // Menulis ke String tidak pernah gagal.
        let _ = write!(out, "{b:02x}");
    }
    out
}

/// `UriEncode` SigV4: karakter tak-dicadangkan (`A-Z a-z 0-9 - _ . ~`) apa adanya, sisanya `%XX` huruf
/// besar; `/` dibiarkan hanya bila `keep_slash` (path objek) [SRC-550].
pub(crate) fn uri_encode(value: &str, keep_slash: bool) -> String {
    use std::fmt::Write as _;
    let mut out = String::with_capacity(value.len());
    for b in value.bytes() {
        let unreserved = b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~');
        if unreserved || (keep_slash && b == b'/') {
            out.push(char::from(b));
        } else {
            // Menulis ke String tidak pernah gagal.
            let _ = write!(out, "%{b:02X}");
        }
    }
    out
}

/// String query kanonik: setiap nama dan nilai di-encode, lalu diurutkan menurut nama (kemudian nilai).
pub(crate) fn canonical_query(params: &[(&str, &str)]) -> String {
    let mut encoded: Vec<(String, String)> = params
        .iter()
        .map(|(k, v)| (uri_encode(k, false), uri_encode(v, false)))
        .collect();
    encoded.sort();
    encoded
        .iter()
        .map(|(k, v)| format!("{k}={v}"))
        .collect::<Vec<_>>()
        .join("&")
}

/// Permintaan kanonik. `headers` berisi pasangan (nama huruf kecil, nilai) dan diurutkan di sini.
pub(crate) fn canonical_request(
    method: &str,
    canonical_uri: &str,
    canonical_query: &str,
    headers: &[(&str, &str)],
    payload_hash: &str,
) -> (String, String) {
    let mut sorted: Vec<(&str, String)> = headers
        .iter()
        .map(|(k, v)| (*k, v.split_whitespace().collect::<Vec<_>>().join(" ")))
        .collect();
    sorted.sort_by(|a, b| a.0.cmp(b.0));
    let canonical_headers: String = sorted.iter().map(|(k, v)| format!("{k}:{v}\n")).collect();
    let signed_headers = sorted.iter().map(|(k, _)| *k).collect::<Vec<_>>().join(";");
    let request = format!(
        "{method}\n{canonical_uri}\n{canonical_query}\n{canonical_headers}\n{signed_headers}\n{payload_hash}"
    );
    (request, signed_headers)
}

/// Cakupan kredensial `YYYYMMDD/<region>/s3/aws4_request`.
pub(crate) fn credential_scope(date: &str, region: &str) -> String {
    format!("{date}/{region}/{SERVICE}/aws4_request")
}

/// String yang ditandatangani.
pub(crate) fn string_to_sign(datetime: &str, scope: &str, canonical_request: &str) -> String {
    format!(
        "{ALGORITHM}\n{datetime}\n{scope}\n{}",
        hex(&sha256(canonical_request.as_bytes()))
    )
}

/// Kunci penandatangan turunan untuk tanggal dan region tertentu.
pub(crate) struct SigningKey {
    key: HmacSha256Key,
}

impl SigningKey {
    pub(crate) fn derive(
        creds: &Credentials,
        date: &str,
        region: &str,
    ) -> Result<Self, ObjectStorageError> {
        let mut first = Vec::with_capacity(4 + creds.secret.expose().len());
        first.extend_from_slice(b"AWS4");
        first.extend_from_slice(creds.secret.expose().as_bytes());
        let k_date = HmacSha256Key::new(&first)
            .map_err(|_| ObjectStorageError::Signing)?
            .sign(date.as_bytes());
        first.fill(0);
        let k_region = HmacSha256Key::new(&k_date)
            .map_err(|_| ObjectStorageError::Signing)?
            .sign(region.as_bytes());
        let k_service = HmacSha256Key::new(&k_region)
            .map_err(|_| ObjectStorageError::Signing)?
            .sign(SERVICE.as_bytes());
        let k_signing = HmacSha256Key::new(&k_service)
            .map_err(|_| ObjectStorageError::Signing)?
            .sign(b"aws4_request");
        let key = HmacSha256Key::new(&k_signing).map_err(|_| ObjectStorageError::Signing)?;
        Ok(Self { key })
    }

    /// Tanda tangan heksadesimal atas `data` (string yang ditandatangani atau policy POST base64).
    pub(crate) fn sign_hex(&self, data: &str) -> String {
        hex(&self.key.sign(data.as_bytes()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uri_encode_follows_sigv4_rules() {
        assert_eq!(uri_encode("aZ09-_.~", false), "aZ09-_.~");
        assert_eq!(uri_encode("a/b c+", false), "a%2Fb%20c%2B");
        assert_eq!(uri_encode("a/b", true), "a/b");
        assert_eq!(
            uri_encode("content-length;host", false),
            "content-length%3Bhost"
        );
    }

    #[test]
    fn canonical_query_sorts_after_encoding_and_keeps_empty_values() {
        assert_eq!(
            canonical_query(&[("versions", ""), ("prefix", "k")]),
            "prefix=k&versions="
        );
    }

    #[test]
    fn amz_time_formats_utc() {
        let t = time::macros::datetime!(2026-11-20 10:04:05 +07:00);
        let a = AmzTime::new(t);
        assert_eq!(a.date, "20261120");
        assert_eq!(a.datetime, "20261120T030405Z");
    }
}
