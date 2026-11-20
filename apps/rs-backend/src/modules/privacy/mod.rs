//! Modul `privacy` (kode modul 15, "Privasi" docs/04-arsitektur.md §3; tabel DD A3). Hanya `api` yang
//! terlihat dari luar modul (coding-standards §1.3).
//!
//! F1-T49: versi teks dan butir persetujuan (US-104 AC1, AC3): seed versi draf lewat migrasi data
//! (UP-75 a, KEPUTUSAN-289) dan endpoint teks yang berlaku `GET /patient/v1/consent-texts/current`.

pub mod api;
mod domain;
mod error;
mod http;
mod policy;
mod repo;
mod service;

use crate::app::AppState;
use authz::PolicyRouter;

/// Rute API pasien modul ini (`openapi/pasien.json`), masing-masing dengan kebijakannya (KEPUTUSAN-127).
pub(crate) fn patient_router() -> PolicyRouter<AppState> {
    http::router()
}
