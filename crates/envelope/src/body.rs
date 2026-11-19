//! Bentuk isi respons ADR-023 butir 1 dan butir 3. Nama field `snake_case` bahasa Inggris.
//! `status` HTTP sengaja tidak dimuat di isi (ADR-023 butir 2).

use crate::code::{ResponseCode, codes};
use foundation::CorrelationId;
use serde::{Serialize, Serializer};

/// Media type respons sukses (ADR-023 butir 3).
pub const CONTENT_TYPE_JSON: &str = "application/json";
/// Media type respons galat, RFC 9457 `[SRC-47]` (ADR-023 butir 3).
pub const CONTENT_TYPE_PROBLEM: &str = "application/problem+json";

fn ser_correlation<S: Serializer>(v: &Option<CorrelationId>, s: S) -> Result<S::Ok, S::Error> {
    match v {
        Some(id) => s.collect_str(id),
        None => s.serialize_none(),
    }
}

fn ser_correlation_required<S: Serializer>(v: &CorrelationId, s: S) -> Result<S::Ok, S::Error> {
    s.collect_str(v)
}

/// Pagination berbasis cursor (ADR-023 butir 1). `next_cursor` bernilai `null` bila tidak ada.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Meta {
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

/// Respons sukses: `data` objek atau daftar, `meta` hanya untuk daftar.
#[derive(Debug, Clone, Serialize)]
pub struct Success<T> {
    response_code: ResponseCode,
    response_description: &'static str,
    data: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    meta: Option<Meta>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "ser_correlation"
    )]
    correlation_id: Option<CorrelationId>,
}

impl<T> Success<T> {
    /// Sukses dengan kode `code` (harus kode HTTP 2xx). Kode non-2xx adalah kesalahan pemrograman:
    /// build debug dan test gagal lewat `debug_assert!` supaya cepat ketahuan; build release jatuh ke
    /// [`codes::OK`] supaya envelope sukses tidak pernah membawa kode galat di production.
    pub fn with_code(code: ResponseCode, data: T) -> Self {
        debug_assert!(
            code.is_success(),
            "Success::with_code menerima kode non-2xx (kesalahan pemrograman)"
        );
        let code = if code.is_success() { code } else { codes::OK };
        Self {
            response_code: code,
            response_description: code.description(),
            data,
            meta: None,
            correlation_id: None,
        }
    }

    /// HTTP 200, `2000000`.
    pub fn ok(data: T) -> Self {
        Self::with_code(codes::OK, data)
    }

    /// HTTP 201, `2010000`.
    pub fn created(data: T) -> Self {
        Self::with_code(codes::CREATED, data)
    }

    /// HTTP 202, `2020000`.
    pub fn accepted(data: T) -> Self {
        Self::with_code(codes::ACCEPTED, data)
    }

    /// Menambahkan `meta` pagination (respons daftar).
    #[must_use]
    pub fn with_meta(mut self, meta: Meta) -> Self {
        self.meta = Some(meta);
        self
    }

    /// Menambahkan `correlation_id` (opsional di sukses).
    #[must_use]
    pub fn with_correlation_id(mut self, id: CorrelationId) -> Self {
        self.correlation_id = Some(id);
        self
    }

    /// Kode respons.
    pub fn response_code(&self) -> ResponseCode {
        self.response_code
    }

    /// Kode HTTP untuk header.
    pub fn http_status(&self) -> http::StatusCode {
        self.response_code.http_status()
    }
}

/// Galat per isian (KU-01): `field` path isian, `code` snake_case Inggris, `message`
/// Bahasa Indonesia untuk pengguna.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FieldError {
    pub field: String,
    pub code: String,
    pub message: String,
}

impl FieldError {
    pub fn new(
        field: impl Into<String>,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            field: field.into(),
            code: code.into(),
            message: message.into(),
        }
    }
}

/// URI `type` RFC 9457: `<base>/<response_code>` (KEPUTUSAN-84). Domain dokumentasi
/// `[TIDAK DIKETAHUI]` (registry §4.7), sehingga `base` diberikan konfigurasi aplikasi.
pub fn problem_type_uri(base: &str, code: ResponseCode) -> String {
    format!("{}/{}", base.trim_end_matches('/'), code.as_str())
}

/// Respons galat, kompatibel RFC 9457 (ADR-023 butir 3). `correlation_id` wajib. Tanpa
/// `title`, `status`, `detail`. Deskripsi diambil dari kode, bukan dari galat sumber, sehingga
/// pesan teknis tidak dapat bocor (ADR-023 butir 5).
#[derive(Debug, Clone, Serialize)]
pub struct ErrorBody {
    #[serde(rename = "type")]
    type_uri: String,
    response_code: ResponseCode,
    response_description: &'static str,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    errors: Vec<FieldError>,
    #[serde(skip_serializing_if = "Option::is_none")]
    instance: Option<String>,
    #[serde(serialize_with = "ser_correlation_required")]
    correlation_id: CorrelationId,
}

impl ErrorBody {
    /// Membuat galat dengan `code`. `type_base` adalah awalan URI dokumentasi jenis masalah.
    /// Kode sukses (2xx) adalah kesalahan pemrograman: build debug dan test gagal lewat
    /// `debug_assert!`; build release menggantinya dengan [`codes::INTERNAL`] agar galat tidak
    /// tampak sukses.
    pub fn new(code: ResponseCode, correlation_id: CorrelationId, type_base: &str) -> Self {
        debug_assert!(
            !code.is_success(),
            "ErrorBody::new menerima kode 2xx (kesalahan pemrograman)"
        );
        let code = if code.is_success() {
            codes::INTERNAL
        } else {
            code
        };
        Self {
            type_uri: problem_type_uri(type_base, code),
            response_code: code,
            response_description: code.description(),
            errors: Vec::new(),
            instance: None,
            correlation_id,
        }
    }

    /// Menambahkan galat per isian.
    #[must_use]
    pub fn with_errors(mut self, errors: Vec<FieldError>) -> Self {
        self.errors = errors;
        self
    }

    /// Menambahkan `instance`: wajib berupa pola rute, tanpa ID data, query string, atau data
    /// pribadi (registry §4.7, ADR-010). Pemanggil bertanggung jawab atas isinya.
    #[must_use]
    pub fn with_instance(mut self, route_pattern: impl Into<String>) -> Self {
        self.instance = Some(route_pattern.into());
        self
    }

    /// Kode respons.
    pub fn response_code(&self) -> ResponseCode {
        self.response_code
    }

    /// Kode HTTP untuk header.
    pub fn http_status(&self) -> http::StatusCode {
        self.response_code.http_status()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    fn cid() -> CorrelationId {
        CorrelationId::parse("01936f8e-9c01-7e02-8f03-1a2b3c4d5e6f").unwrap()
    }

    fn to_value<T: Serialize>(v: &T) -> Value {
        serde_json::to_value(v).unwrap()
    }

    #[test]
    fn success_single_object_matches_registry_example() {
        let v = to_value(&Success::ok(json!({"order_id": "x", "total_idr": 150000})));
        assert_eq!(
            v,
            json!({
                "response_code": "2000000",
                "response_description": "Berhasil",
                "data": {"order_id": "x", "total_idr": 150000}
            })
        );
    }

    #[test]
    fn success_list_has_meta_and_optional_correlation() {
        let s = Success::ok(json!([1, 2]))
            .with_meta(Meta {
                next_cursor: Some("abc".into()),
                has_more: true,
            })
            .with_correlation_id(cid());
        let v = to_value(&s);
        assert_eq!(v["meta"], json!({"next_cursor": "abc", "has_more": true}));
        assert_eq!(v["correlation_id"], "01936f8e-9c01-7e02-8f03-1a2b3c4d5e6f");
        assert_eq!(s.http_status(), http::StatusCode::OK);
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "non-2xx")]
    fn success_with_error_code_panics_in_debug() {
        let _ = Success::with_code(codes::NOT_FOUND, json!({}));
    }

    #[test]
    #[cfg(not(debug_assertions))]
    fn success_never_carries_error_code_in_release() {
        let s = Success::with_code(codes::NOT_FOUND, json!({}));
        assert_eq!(s.response_code(), codes::OK);
        assert_eq!(s.http_status(), http::StatusCode::OK);
    }

    #[test]
    fn success_helpers_use_2xx_statuses() {
        assert_eq!(Success::ok(1).http_status(), http::StatusCode::OK);
        assert_eq!(Success::created(1).http_status(), http::StatusCode::CREATED);
        assert_eq!(
            Success::accepted(1).http_status(),
            http::StatusCode::ACCEPTED
        );
    }

    #[test]
    fn error_body_has_no_status_title_detail_and_requires_correlation() {
        let v = to_value(&ErrorBody::new(
            codes::NOT_FOUND,
            cid(),
            "https://d.example/problems/",
        ));
        assert_eq!(
            v,
            json!({
                "type": "https://d.example/problems/4040001",
                "response_code": "4040001",
                "response_description": "Data tidak ditemukan",
                "correlation_id": "01936f8e-9c01-7e02-8f03-1a2b3c4d5e6f"
            })
        );
        for k in ["status", "title", "detail", "data"] {
            assert!(v.get(k).is_none(), "{k}");
        }
    }

    #[test]
    fn validation_error_lists_fields_and_instance() {
        let b = ErrorBody::new(codes::INVALID, cid(), "https://d.example/problems")
            .with_errors(vec![FieldError::new(
                "email",
                "invalid_format",
                "Format email tidak valid",
            )])
            .with_instance("/v1/accounts");
        let v = to_value(&b);
        assert_eq!(
            v["errors"],
            json!([{"field": "email", "code": "invalid_format", "message": "Format email tidak valid"}])
        );
        assert_eq!(v["instance"], "/v1/accounts");
        assert_eq!(b.http_status(), http::StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "kode 2xx")]
    fn error_body_with_success_code_panics_in_debug() {
        let _ = ErrorBody::new(codes::OK, cid(), "https://d.example/problems");
    }

    #[test]
    #[cfg(not(debug_assertions))]
    fn error_body_never_reports_success_code_in_release() {
        let b = ErrorBody::new(codes::OK, cid(), "https://d.example/problems");
        assert_eq!(b.response_code(), codes::INTERNAL);
        assert_eq!(b.http_status(), http::StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn media_types() {
        assert_eq!(CONTENT_TYPE_JSON, "application/json");
        assert_eq!(CONTENT_TYPE_PROBLEM, "application/problem+json");
    }
}
