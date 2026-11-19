//! Modul `platform` (kode modul 17). Pada F1-T11 hanya endpoint kesehatan. Hanya `api` yang terlihat
//! dari luar modul (coding-standards §1.3); submodul lain privat.

pub mod api;
mod dto;
mod http;
mod service;

use crate::app::AppState;
use utoipa_axum::router::OpenApiRouter;

/// Rute modul ini beserta metadata OpenAPI-nya (rute dan spesifikasi dibangun dari anotasi yang
/// sama, sehingga tidak dapat berbeda: ADR-022 butir 3).
pub(crate) fn router() -> OpenApiRouter<AppState> {
    http::router()
}
