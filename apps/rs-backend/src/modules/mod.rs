//! Modul Ruang Sehat (coding-standards §1.3): `platform` (kode modul 17; endpoint kesehatan F1-T11,
//! berkas F1-T29), `identity` (kode modul 01; pengambil event Keycloak F1-T34, resolver akun dan
//! pencabutan sesi F1-T33), dan `profile` (kode modul 02; pemilik penerima layanan untuk relasi
//! kepemilikan F1-T38). Modul bisnis lain menyusul di task masing-masing.

pub mod identity;
pub mod platform;
pub mod profile;
