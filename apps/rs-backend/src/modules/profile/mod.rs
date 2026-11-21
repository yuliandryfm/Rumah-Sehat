//! Modul `profile` (kode modul 02): pasien, penerima layanan, alamat, dan identitas (DD A2). Hanya `api`
//! yang terlihat dari luar modul (coding-standards §1.3).
//!
//! F1-T38: pemilik penerima layanan untuk relasi kepemilikan `authz::care_recipient_access`.
//! F1-T60/F1-T61 (reference implementation US-106): use case dan endpoint identitas pasien
//! (`/patient/v1`, `openapi/pasien.json`). Endpoint profil lain dibuat task masing-masing.

pub mod api;
// Sebagian aturan (keputusan Verifikator US-201) baru dipakai service F1-T62.
#[allow(dead_code)]
mod domain;
mod dto;
mod error;
mod http;
mod policy;
mod repo;
mod service;

use crate::app::AppState;
use authz::PolicyRouter;

/// Rute aplikasi pasien modul ini (`/patient/v1`, realm `pasien`).
pub(crate) fn patient_router() -> PolicyRouter<AppState> {
    http::router()
}
