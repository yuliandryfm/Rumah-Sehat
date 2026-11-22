//! Modul `corporate` (modul Korporat, ADR-028; tabel A20 di `module-tables.toml`). Hanya `api` yang terlihat
//! dari luar modul (coding-standards §1.3).
//!
//! F1-T100: pembaca keanggotaan aktif HR untuk cakupan tenant perusahaan (`authz::CorporateScope`,
//! ADR-028 butir 1). Endpoint `/v1/corporate/*` dibuat task masing-masing (F1-T101 dst.); setiap endpoint
//! baru wajib terdaftar di harness isolasi `apps/rs-backend/tests/corporate_isolation.rs` (H-18).
//! F1-T104: verifikasi perusahaan oleh Verifikator (US-1902) di `/backoffice/v1`
//! (`openapi/backoffice.json`): antrean, detail, URL dokumen, setujui/tolak, daftar alasan penolakan.

pub mod api;
mod domain;
mod dto;
mod error;
mod http;
mod policy;
mod repo;
mod verification;

use crate::app::AppState;
use authz::PolicyRouter;

/// Rute backoffice verifikasi perusahaan (`/backoffice/v1`, realm `admin`, sub-peran Verifikator).
pub(crate) fn backoffice_router() -> PolicyRouter<AppState> {
    http::router()
}
