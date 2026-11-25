//! Modul `identity` (kode modul 01, "Identitas dan akses" docs/04-arsitektur.md §3; tabel DD A1). Hanya
//! `api` yang terlihat dari luar modul (coding-standards §1.3).
//!
//! - F1-T34: pengambil event Keycloak E-01–E-04 ke `audit_outbox` (ADR-009 butir 2, ADR-011 butir 10)
//!   untuk `rs-backend worker`, dengan posisi baca `idp_event_cursor` dan tanda terima
//!   `idp_event_receipt`.
//! - F1-T33: resolver akun untuk `authz::PrincipalResolver` dan penolakan sesi seketika untuk
//!   `authz::SessionGuard`, dengan cache Valkey dan cadangan PostgreSQL (ADR-011 butir 8, KU-10,
//!   US-103 AC5, RT-18).
//!
//! - F1-T65: akun dan sub-peran admin Ruang Sehat (US-208): aturan murni `admin_domain`, query
//!   `admin_repo`, use case `admin_service` (bootstrap Super Admin pertama dengan persetujuan orang kedua,
//!   buat akun + undangan, ubah sub-peran, nonaktifkan), dan aktivasi pertama di pengambil event.
//!
//! - F1-T107: kelola anggota HR perusahaan (US-1904 AC2, AC5): aturan murni `corporate_hr_domain`, query
//!   akun `corporate_hr_repo` (keanggotaan lewat `corporate::api`), use case `corporate_hr_service`, dan
//!   endpoint `/v1/corporate/members` (`corporate_hr_http`).
//!
//! - ADR-030 langkah 1-4 (CR-026, KEPUTUSAN-302/305): penyediaan identitas lewat worker. Aturan murni
//!   `idp_domain`, query niat `idp_repo`, API internal pencatatan niat `idp_service` (dipanggil use case di
//!   transaksinya), klien pengguna Keycloak `idp_worker_client` (hanya worker), dan worker `idp_worker`.
//!
//! Endpoint HTTP lain modul ini (registrasi, keluar, kelola akun admin) menyusul di task masing-masing.

mod admin_domain;
mod admin_repo;
mod admin_service;
pub mod api;
mod cache;
mod corporate_hr_domain;
mod corporate_hr_http;
mod corporate_hr_repo;
mod corporate_hr_service;
mod domain;
mod error;
mod idp_config;
mod idp_domain;
mod idp_repo;
mod idp_service;
mod idp_worker;
mod idp_worker_client;
mod jobs;
mod keycloak;
mod repo;
mod service;

use crate::app::AppState;
use authz::PolicyRouter;

/// Rute API korporat modul ini (`openapi/korporat.json`, `/v1/corporate/members`, F1-T107), masing-masing
/// dengan kebijakannya (KEPUTUSAN-127).
pub(crate) fn corporate_router() -> PolicyRouter<AppState> {
    corporate_hr_http::router()
}
