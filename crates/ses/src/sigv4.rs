//! AWS Signature Version 4 untuk permintaan `POST` berisi JSON ke SES API v2 [SRC-550] [SRC-580].
//!
//! Header yang ditandatangani tetap: `content-type`, `host`, `x-amz-date` (sama dengan botocore `SigV4Auth`
//! untuk layanan `ses`, vektor uji [SRC-581]). Primitif HMAC-SHA256/SHA-256 berasal dari crate `crypto`;
//! modul ini hanya menyusun string sesuai spesifikasi (coding-standards §5.5).
//!
//! Catatan: crate `object-storage` punya penyusun SigV4 sendiri untuk layanan `s3` (presigned POST/PUT).
//! Penyatuan keduanya ke satu modul bersama adalah kandidat consistency review, bukan bagian F1-T36.

use crate::error::SesError;
use crypto::{HmacSha256Key, sha256};
use foundation::Sensitive;
use time::OffsetDateTime;

/// Algoritma SigV4.
const ALGORITHM: &str = "AWS4-HMAC-SHA256";
/// Nama penandatanganan SES API v2 (`signingName` model layanan sesv2 [SRC-580]).
pub(crate) const SERVICE: &str = "ses";
/// Header yang ditandatangani, urut abjad.
const SIGNED_HEADERS: &str = "content-type;host;x-amz-date";

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

/// Bahan satu permintaan yang ditandatangani.
pub(crate) struct SignInput<'a> {
    pub(crate) access_key_id: &'a str,
    pub(crate) secret: &'a Sensitive<String>,
    pub(crate) region: &'a str,
    /// Nilai header `Host` persis seperti yang dikirim (dengan `:port` bila bukan 443).
    pub(crate) host: &'a str,
    /// Path absolut tanpa query, hanya karakter tak-dicadangkan dan `/`.
    pub(crate) path: &'a str,
    pub(crate) content_type: &'a str,
    pub(crate) body: &'a [u8],
}

/// Permintaan kanonik `POST` tanpa query.
fn canonical_request(input: &SignInput<'_>, datetime: &str) -> String {
    format!(
        "POST\n{path}\n\ncontent-type:{ct}\nhost:{host}\nx-amz-date:{datetime}\n\n{SIGNED_HEADERS}\n{hash}",
        path = input.path,
        ct = input.content_type.trim(),
        host = input.host.trim(),
        hash = hex(&sha256(input.body)),
    )
}

/// Kunci penandatangan turunan `HMAC("AWS4" + rahasia, tanggal) → region → "ses" → "aws4_request"`.
fn signing_key(
    secret: &Sensitive<String>,
    date: &str,
    region: &str,
) -> Result<HmacSha256Key, SesError> {
    let mut first = Vec::with_capacity(4 + secret.expose().len());
    first.extend_from_slice(b"AWS4");
    first.extend_from_slice(secret.expose().as_bytes());
    let k_date = HmacSha256Key::new(&first).map(|k| k.sign(date.as_bytes()));
    first.fill(0);
    let k_date = k_date.map_err(|_| SesError::InvalidConfig("secret_access_key"))?;
    let k_region = HmacSha256Key::new(&k_date)
        .map_err(|_| SesError::Signing)?
        .sign(region.as_bytes());
    let k_service = HmacSha256Key::new(&k_region)
        .map_err(|_| SesError::Signing)?
        .sign(SERVICE.as_bytes());
    let k_signing = HmacSha256Key::new(&k_service)
        .map_err(|_| SesError::Signing)?
        .sign(b"aws4_request");
    HmacSha256Key::new(&k_signing).map_err(|_| SesError::Signing)
}

/// Header `Authorization` untuk permintaan pada waktu `at`; mengembalikan juga nilai `X-Amz-Date`.
pub(crate) fn authorization(
    input: &SignInput<'_>,
    at: OffsetDateTime,
) -> Result<(String, String), SesError> {
    let t = AmzTime::new(at);
    let scope = format!("{}/{}/{SERVICE}/aws4_request", t.date, input.region);
    let creq = canonical_request(input, &t.datetime);
    let sts = format!(
        "{ALGORITHM}\n{}\n{scope}\n{}",
        t.datetime,
        hex(&sha256(creq.as_bytes()))
    );
    let key = signing_key(input.secret, &t.date, input.region)?;
    let signature = hex(&key.sign(sts.as_bytes()));
    let header = format!(
        "{ALGORITHM} Credential={}/{scope}, SignedHeaders={SIGNED_HEADERS}, Signature={signature}",
        input.access_key_id
    );
    Ok((header, t.datetime))
}

#[cfg(test)]
mod tests {
    use super::*;

    // Kredensial contoh fiktif dari dokumentasi AWS; vektor dihasilkan botocore 1.40.72 `SigV4Auth`
    // (layanan "ses", region ap-southeast-3, 2026-11-20T03:04:05Z) [SRC-581].
    fn input<'a>(secret: &'a Sensitive<String>, host: &'a str, body: &'a [u8]) -> SignInput<'a> {
        SignInput {
            access_key_id: "AKIAIOSFODNN7EXAMPLE",
            secret,
            region: "ap-southeast-3",
            host,
            path: "/v2/email/outbound-emails",
            content_type: "application/json",
            body,
        }
    }

    #[test]
    fn matches_botocore_vectors() {
        let secret = Sensitive::new("wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY".to_string());
        let at = time::macros::datetime!(2026-11-20 10:04:05 +07:00);
        let (auth, date) = authorization(
            &input(
                &secret,
                "email.ap-southeast-3.amazonaws.com",
                br#"{"Content":{"Raw":{"Data":"dWppCg=="}}}"#,
            ),
            at,
        )
        .unwrap();
        assert_eq!(date, "20261120T030405Z");
        assert_eq!(
            auth,
            "AWS4-HMAC-SHA256 Credential=AKIAIOSFODNN7EXAMPLE/20261120/ap-southeast-3/ses/aws4_request, \
             SignedHeaders=content-type;host;x-amz-date, \
             Signature=57b41f32ff82d414d4ea58b6f4fb2c13fda7114a9e49d4d9f923cd3ca3bd9fa9"
        );
        let (auth, _) = authorization(&input(&secret, "127.0.0.1:18443", b"{}"), at).unwrap();
        assert!(auth.ends_with(
            "Signature=88bf2c8c36f9234af45675ab3d7d808ea2260c0eb0162171280b364fe7454b46"
        ));
    }

    #[test]
    fn short_secret_is_rejected_without_leaking_it() {
        let secret = Sensitive::new("pendek".to_string());
        let at = time::macros::datetime!(2026-11-20 03:04:05 UTC);
        let e = authorization(&input(&secret, "h", b""), at).unwrap_err();
        assert_eq!(e, SesError::InvalidConfig("secret_access_key"));
        assert!(!format!("{e:?} {e}").contains("pendek"));
    }
}
