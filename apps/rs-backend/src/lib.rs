//! Backend Ruang Sehat (F1-T11): satu biner dengan subcommand `api`, `worker`, `audit-relay`,
//! `audit-verify`, dan `email-relay` (docs/coding-standards.md §1.1, KEPUTUSAN-89). Yang berfungsi: `api`,
//! `audit-relay` (F1-T24, relay audit di crate `audit`), `audit-verify` (F1-T25, crate `audit-store`), dan
//! `email-relay` (F1-T36, ADR-026: crate `email-relay` + adaptor `ses`);
//! `worker` adalah kerangka yang gagal dengan pesan jelas.
//!
//! Lapisan: [`cli`] (pemilihan subcommand dan siklus hidup proses), [`config`] (konfigurasi bertipe),
//! [`app`] (perakitan router, middleware, fallback), [`http_support`] (adaptor envelope ADR-023 ke
//! axum), [`modules`] (modul bisnis dan platform), [`openapi`] (pembangkitan spesifikasi, ADR-022),
//! [`rate_limit`] (batas laju endpoint publik, F1-T94).

pub mod app;
pub mod cli;
pub mod config;
pub mod http_support;
pub mod modules;
pub mod openapi;
pub mod rate_limit;
