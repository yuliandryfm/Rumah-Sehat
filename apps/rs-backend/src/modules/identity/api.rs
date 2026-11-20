//! Antarmuka publik modul `identity` untuk perakitan aplikasi dan modul lain (F1-T33):
//!
//! - [`AccountDirectory`]: dipasang ke `authz::Enforcement::new(auth, directory.clone(), directory)`
//!   sebagai resolver akun dan penjaga sesi yang dicabut.
//! - Pencabutan sesi: [`AccountDirectory::record_revocation`] di transaksi use case pemanggil (misalnya
//!   keluar F1-T54, penonaktifan US-207), lalu [`AccountDirectory::publish_revocation`] setelah commit.
//! - Penulis status akun memanggil [`AccountDirectory::refresh_account`] setelah commit.
//! - Aksi berisiko memakai [`AccountDirectory::resolve_fresh`] (tanpa cache, M-13).
//! - Job berkala: [`AccountDirectory::purge_expired_revocations`]; saat start atau setelah isi Valkey
//!   hilang: [`AccountDirectory::rebuild_revocation_cache`] (juga dipicu otomatis di latar belakang).

pub use super::error::IdentityError;
pub use super::service::{
    AccountDirectory, DEFAULT_ACCOUNT_CACHE_TTL_SECONDS, RecordedRevocation, RevocationReason,
};
