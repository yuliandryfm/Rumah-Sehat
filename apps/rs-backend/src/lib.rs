//! Backend Ruang Sehat (F1-T11): satu biner dengan subcommand `api`, `worker`, `audit-relay`, dan
//! `audit-verify` (docs/coding-standards.md §1.1, KEPUTUSAN-89). Hanya `api` yang berfungsi pada
//! task ini; subcommand lain adalah kerangka yang gagal dengan pesan jelas.
//!
//! Lapisan: [`cli`] (pemilihan subcommand dan siklus hidup proses), [`config`] (konfigurasi bertipe),
//! [`app`] (perakitan router, middleware, fallback), [`http_support`] (adaptor envelope ADR-023 ke
//! axum), [`modules`] (modul bisnis dan platform), [`openapi`] (pembangkitan spesifikasi, ADR-022).

pub mod app;
pub mod cli;
pub mod config;
pub mod http_support;
pub mod modules;
pub mod openapi;
