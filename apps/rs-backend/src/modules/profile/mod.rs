//! Modul `profile` (kode modul 02): pasien, penerima layanan, alamat, dan identitas (DD A2). Hanya `api`
//! yang terlihat dari luar modul (coding-standards §1.3).
//!
//! F1-T38: pemilik penerima layanan untuk relasi kepemilikan `authz::care_recipient_access`.
//! F1-T60/F1-T61 (reference implementation US-106): use case dan endpoint identitas pasien
//! (`/patient/v1`, `openapi/pasien.json`). Endpoint profil lain dibuat task masing-masing.
//! F1-T62 (reference implementation US-201): use case dan endpoint Verifikator (`/backoffice/v1`,
//! `openapi/backoffice.json`): antrean, detail, URL foto dokumen, setujui/tolak.

pub mod api;
mod domain;
mod dto;
mod error;
mod http;
mod policy;
mod repo;
mod self_decision;
mod service;
mod verification;
mod verifier_dto;
mod verifier_http;

use crate::app::AppState;
use authz::PolicyRouter;

/// Rute aplikasi pasien modul ini (`/patient/v1`, realm `pasien`).
pub(crate) fn patient_router() -> PolicyRouter<AppState> {
    http::router()
}

/// Rute backoffice Verifikator modul ini (`/backoffice/v1`, realm `admin`, sub-peran Verifikator).
pub(crate) fn backoffice_router() -> PolicyRouter<AppState> {
    verifier_http::router()
}
