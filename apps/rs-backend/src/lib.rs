//! Backend Ruang Sehat (F1-T11): satu biner dengan subcommand `api`, `worker`, `audit-relay`,
//! `audit-verify`, dan `email-relay` (docs/coding-standards.md §1.1, KEPUTUSAN-89): `api`, `worker` (F1-T34,
//! pengambil event Keycloak di modul `identity`), `audit-relay` (F1-T24, relay audit di crate `audit`),
//! `audit-verify` (F1-T25, crate `audit-store`), dan `email-relay` (F1-T36, ADR-026: crate `email-relay` +
//! adaptor `ses`).
//!
//! Lapisan: [`cli`] (pemilihan subcommand dan siklus hidup proses), [`config`] (konfigurasi bertipe),
//! [`access_audit`] (event E-05 hasil gagal untuk penolakan akses ke data sensitif, F1-T38),
//! [`app`] (perakitan router, middleware, fallback), [`http_support`] (adaptor envelope ADR-023 ke
//! axum), [`modules`] (modul bisnis dan platform), [`openapi`] (pembangkitan spesifikasi, ADR-022).

pub mod access_audit;
pub mod app;
pub mod cli;
pub mod config;
pub mod http_support;
pub mod modules;
pub mod openapi;
