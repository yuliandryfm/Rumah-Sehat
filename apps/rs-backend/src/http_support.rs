//! Adaptor envelope ADR-023 ke axum. Crate `envelope` tidak bergantung pada framework HTTP
//! (adaptor ada di lapisan aplikasi), sehingga tipe pembungkus ada di sini.
//!
//! Status HTTP selalu diturunkan dari `response_code` (envelope), jadi header dan tiga digit pertama
//! kode tidak mungkin berbeda. Respons galat memakai `application/problem+json`, sukses
//! `application/json` (ADR-023 butir 3).

use axum::http::{HeaderValue, StatusCode, header::CONTENT_TYPE};
use axum::response::{IntoResponse, Response};
use envelope::CommonError;
use envelope::{ApiError, CONTENT_TYPE_JSON, CONTENT_TYPE_PROBLEM, Success};
use foundation::CorrelationId;
use serde::Serialize;
use std::sync::Arc;

fn json_response<B: Serialize>(
    status: StatusCode,
    content_type: &'static str,
    body: &B,
) -> Response {
    match serde_json::to_vec(body) {
        Ok(bytes) => (
            status,
            [(CONTENT_TYPE, HeaderValue::from_static(content_type))],
            bytes,
        )
            .into_response(),
        // Isi envelope hanya berisi tipe yang selalu dapat diserialisasi; cabang ini pertahanan
        // terakhir dan sengaja tanpa isi supaya tidak membocorkan teks galat (ADR-023 butir 5).
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

/// Respons sukses envelope.
#[derive(Debug)]
pub struct ApiSuccess<T>(pub Success<T>);

impl<T: Serialize> IntoResponse for ApiSuccess<T> {
    fn into_response(self) -> Response {
        json_response(self.0.http_status(), CONTENT_TYPE_JSON, &self.0)
    }
}

/// Respons galat envelope dari galat domain yang mengimplementasikan [`ApiError`].
#[derive(Debug)]
pub struct ApiFailure<E: ApiError> {
    error: E,
    correlation_id: CorrelationId,
    type_base: String,
}

impl<E: ApiError> ApiFailure<E> {
    /// `type_base` adalah awalan URI jenis masalah dari konfigurasi.
    pub fn new(error: E, correlation_id: CorrelationId, type_base: &str) -> Self {
        Self {
            error,
            correlation_id,
            type_base: type_base.to_string(),
        }
    }
}

impl<E: ApiError> IntoResponse for ApiFailure<E> {
    fn into_response(self) -> Response {
        let body = self
            .error
            .to_error_body(self.correlation_id, &self.type_base);
        json_response(body.http_status(), CONTENT_TYPE_PROBLEM, &body)
    }
}

/// Pembentuk respons galat envelope untuk middleware autentikasi dan otorisasi (`authn`, `authz`).
#[derive(Debug, Clone)]
pub struct EnvelopeRenderer {
    type_base: Arc<str>,
}

impl EnvelopeRenderer {
    /// `type_base` adalah awalan URI jenis masalah dari konfigurasi.
    pub fn new(type_base: &str) -> Self {
        Self {
            type_base: Arc::from(type_base),
        }
    }
}

impl authn::ErrorRenderer for EnvelopeRenderer {
    fn render(&self, error: CommonError, correlation_id: CorrelationId) -> Response {
        ApiFailure::new(error, correlation_id, &self.type_base).into_response()
    }
}
