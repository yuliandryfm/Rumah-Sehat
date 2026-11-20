//! Modul `identity` (kode modul 01): akun aplikasi, sub-peran admin, dan pencabutan sesi (DD A1). Hanya
//! `api` yang terlihat dari luar modul (coding-standards §1.3).
//!
//! F1-T33: resolver akun untuk `authz::PrincipalResolver` dan penolakan sesi seketika untuk
//! `authz::SessionGuard`, dengan cache Valkey dan cadangan PostgreSQL (ADR-011 butir 8, KU-10,
//! US-103 AC5, RT-18). Endpoint modul ini (keluar, kelola akun) dibuat task masing-masing.

pub mod api;
mod cache;
mod error;
mod repo;
mod service;
