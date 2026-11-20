//! Modul `identity` (kode modul 01). Pada F1-T92 baru berisi resolver akun untuk middleware kebijakan
//! (`authz::PrincipalResolver`, ADR-011 butir 6: peran disimpan di aplikasi, bukan di token). Hanya `api`
//! yang terlihat dari luar modul (coding-standards §1.3).
//!
//! Belum di sini: daftar sesi yang dicabut dan cadangan Valkey (F1-T33, KU-10), pengelolaan akun admin
//! (F1-T65).

pub mod api;
mod repo;
