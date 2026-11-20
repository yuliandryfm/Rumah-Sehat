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
//! Endpoint HTTP modul ini (registrasi, keluar, kelola akun) menyusul di task masing-masing.

mod admin_domain;
mod admin_repo;
mod admin_service;
pub mod api;
mod cache;
mod domain;
mod error;
mod jobs;
mod keycloak;
mod repo;
mod service;
