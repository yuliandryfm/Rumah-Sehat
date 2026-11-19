//! Envelope respons API ADR-023 dan registry `response_code` (docs/05-response-codes.md).
//!
//! Isi: [`ResponseCode`] dan [`codes`] (kode umum modul `00`), [`Success`], [`Meta`],
//! [`ErrorBody`], [`FieldError`], trait [`ApiError`] (pemetaan galat ke HTTP dan kode), enum
//! [`CommonError`], dan [`registry`] (pembaca tabel registry untuk pemeriksaan otomatis).
//!
//! Crate ini tidak bergantung pada framework HTTP; adaptor axum ada di lapisan aplikasi.

mod body;
mod code;
mod error;
pub mod registry;

pub use body::{
    CONTENT_TYPE_JSON, CONTENT_TYPE_PROBLEM, ErrorBody, FieldError, Meta, Success, problem_type_uri,
};
pub use code::{ResponseCode, codes};
pub use error::{ApiError, CommonError};
pub use http::StatusCode;
