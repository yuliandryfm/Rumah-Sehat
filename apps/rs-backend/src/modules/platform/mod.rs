//! Modul `platform` (kode modul 17): endpoint kesehatan (F1-T11) dan berkas unggahan (F1-T29: slot
//! unggah, URL bertanda tangan, validasi isi, `stored_file`). Hanya `api` yang terlihat dari luar modul
//! (coding-standards §1.3); submodul lain privat. Endpoint HTTP berkas dibuat modul pemakai (misalnya
//! pengajuan identitas F1-T60) setelah validasi token tersedia.

pub mod api;
mod domain;
mod dto;
mod error;
mod http;
mod repo;
mod service;

use crate::app::AppState;
use utoipa_axum::router::OpenApiRouter;

/// Rute modul ini beserta metadata OpenAPI-nya (rute dan spesifikasi dibangun dari anotasi yang
/// sama, sehingga tidak dapat berbeda: ADR-022 butir 3).
pub(crate) fn router() -> OpenApiRouter<AppState> {
    http::router()
}
