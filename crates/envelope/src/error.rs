//! Pemetaan galat domain ke kode HTTP dan `response_code` di satu tempat (coding-standards §3.2).

use crate::body::{ErrorBody, FieldError};
use crate::code::{ResponseCode, codes};
use foundation::CorrelationId;
use http::StatusCode;

/// Dipasang pada enum galat setiap modul. Varian menyatakan arti bisnis; trait ini
/// memetakannya ke kode terdaftar. Status HTTP diturunkan dari `response_code`, sehingga
/// 3 digit pertama kode dan header HTTP tidak mungkin berbeda (ADR-023 konsekuensi).
///
/// Berbeda dari ilustrasi di coding-standards §3.2 (`http_and_code`): tidak ada metode yang
/// mengembalikan pasangan, karena pasangan dapat tidak konsisten. Ilustrasi itu bertanda
/// "API final di reference implementation".
pub trait ApiError {
    /// Kode terdaftar untuk galat ini.
    fn response_code(&self) -> ResponseCode;

    /// Galat per isian; hanya untuk galat validasi.
    fn field_errors(&self) -> &[FieldError] {
        &[]
    }

    /// Kode HTTP untuk header, diturunkan dari [`ApiError::response_code`].
    fn http_status(&self) -> StatusCode {
        self.response_code().http_status()
    }

    /// Membangun isi galat. Isi tidak pernah memuat teks galat sumber (ADR-023 butir 5).
    fn to_error_body(&self, correlation_id: CorrelationId, type_base: &str) -> ErrorBody {
        ErrorBody::new(self.response_code(), correlation_id, type_base)
            .with_errors(self.field_errors().to_vec())
    }
}

/// Galat lintas modul untuk kode umum modul `00` (registry §3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommonError {
    /// 400: badan atau parameter tidak dapat dibaca.
    BadRequest,
    /// 401: belum login, token tidak sah, sesi berakhir atau dicabut.
    Unauthenticated,
    /// 401: akun pemegang sesi yang sah berstatus nonaktif (KU-10 bagian sesi, layar "Akun nonaktif").
    AccountInactive,
    /// 403: peran tidak boleh menjalankan fungsi (keberadaan fungsi tidak rahasia).
    Forbidden,
    /// 404: data tidak ada **atau** tidak boleh diakses (KU-03).
    NotFound,
    /// 409: transisi status kalah atau tidak sah.
    StatusConflict,
    /// 409: kunci idempotensi dipakai dengan isi berbeda.
    IdempotencyKeyReused,
    /// 413: berkas terlalu besar.
    PayloadTooLarge,
    /// 415: jenis berkas tidak didukung.
    UnsupportedMediaType,
    /// 422: validasi gagal, dengan galat per isian.
    Validation(Vec<FieldError>),
    /// 429: batas laju terlampaui.
    RateLimited,
    /// 500: galat tak terduga. Tidak membawa detail; detail hanya di log (ADR-010).
    Internal,
    /// 502: pihak ketiga membalas tidak benar.
    UpstreamBadGateway,
    /// 503: layanan tidak tersedia.
    Unavailable,
    /// 504: pihak ketiga melewati batas waktu.
    UpstreamTimeout,
}

impl ApiError for CommonError {
    fn response_code(&self) -> ResponseCode {
        match self {
            Self::BadRequest => codes::BAD_REQUEST,
            Self::Unauthenticated => codes::UNAUTHENTICATED,
            Self::AccountInactive => codes::ACCOUNT_INACTIVE,
            Self::Forbidden => codes::FORBIDDEN,
            Self::NotFound => codes::NOT_FOUND,
            Self::StatusConflict => codes::STATUS_CONFLICT,
            Self::IdempotencyKeyReused => codes::IDEMPOTENCY_KEY_REUSED,
            Self::PayloadTooLarge => codes::PAYLOAD_TOO_LARGE,
            Self::UnsupportedMediaType => codes::UNSUPPORTED_MEDIA_TYPE,
            Self::Validation(_) => codes::INVALID,
            Self::RateLimited => codes::RATE_LIMITED,
            Self::Internal => codes::INTERNAL,
            Self::UpstreamBadGateway => codes::UPSTREAM_BAD_GATEWAY,
            Self::Unavailable => codes::UNAVAILABLE,
            Self::UpstreamTimeout => codes::UPSTREAM_TIMEOUT,
        }
    }

    fn field_errors(&self) -> &[FieldError] {
        match self {
            Self::Validation(e) => e,
            _ => &[],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all() -> Vec<(CommonError, u16, &'static str)> {
        vec![
            (CommonError::BadRequest, 400, "4000001"),
            (CommonError::Unauthenticated, 401, "4010001"),
            (CommonError::AccountInactive, 401, "4010002"),
            (CommonError::Forbidden, 403, "4030001"),
            (CommonError::NotFound, 404, "4040001"),
            (CommonError::StatusConflict, 409, "4090001"),
            (CommonError::IdempotencyKeyReused, 409, "4090002"),
            (CommonError::PayloadTooLarge, 413, "4130001"),
            (CommonError::UnsupportedMediaType, 415, "4150001"),
            (CommonError::Validation(vec![]), 422, "4220001"),
            (CommonError::RateLimited, 429, "4290001"),
            (CommonError::Internal, 500, "5000001"),
            (CommonError::UpstreamBadGateway, 502, "5020001"),
            (CommonError::Unavailable, 503, "5030001"),
            (CommonError::UpstreamTimeout, 504, "5040001"),
        ]
    }

    #[test]
    fn mapping_is_exact_and_consistent() {
        for (e, http, code) in all() {
            assert_eq!(e.http_status().as_u16(), http, "{e:?}");
            assert_eq!(e.response_code().as_str(), code, "{e:?}");
            assert!(e.response_code().as_str().starts_with(&http.to_string()));
        }
    }

    #[test]
    fn validation_carries_field_errors_others_do_not() {
        let fe = FieldError::new("email", "invalid_format", "Format email tidak valid");
        let v = CommonError::Validation(vec![fe.clone()]);
        assert_eq!(v.field_errors(), &[fe]);
        assert!(CommonError::NotFound.field_errors().is_empty());
    }

    #[test]
    fn not_found_body_is_same_for_missing_and_forbidden_object() {
        // KU-03: "tidak ada" dan "tidak boleh" memakai varian yang sama, isi identik.
        let id = CorrelationId::new();
        let a = serde_json::to_string(&CommonError::NotFound.to_error_body(id, "https://x/p"));
        let b = serde_json::to_string(&CommonError::NotFound.to_error_body(id, "https://x/p"));
        assert_eq!(a.unwrap(), b.unwrap());
    }

    #[test]
    fn internal_body_carries_no_detail() {
        let s = serde_json::to_string(
            &CommonError::Internal.to_error_body(CorrelationId::new(), "https://x/p"),
        )
        .unwrap();
        assert!(s.contains("Terjadi kesalahan pada sistem"));
        assert!(!s.contains("errors"));
    }
}
