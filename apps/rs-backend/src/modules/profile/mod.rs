//! Modul `profile` (kode modul 02): pasien, penerima layanan, alamat, dan identitas (DD A2). Hanya `api`
//! yang terlihat dari luar modul (coding-standards §1.3).
//!
//! F1-T38: pemilik penerima layanan untuk relasi kepemilikan `authz::care_recipient_access`. Endpoint
//! profil dibuat task masing-masing (F1-T56 dan seterusnya).

pub mod api;
// Aturan verifikasi identitas F1-T59; dipakai service pengajuan identitas F1-T60.
#[allow(dead_code)]
mod domain;
mod error;
mod repo;
