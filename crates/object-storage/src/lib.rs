//! Klien object storage S3-kompatibel minimal untuk berkas unggahan (F1-T29, ADR-018).
//!
//! - [`Presigner`]: URL unggah bertanda tangan SigV4, urutan preferensi KEPUTUSAN-234: presigned POST
//!   dengan policy (`content-length-range`, `Content-Type`, `key`), lalu presigned PUT dengan
//!   `Content-Type`/`Content-Length` bertanda tangan. Murni, tanpa I/O.
//! - [`ObjectStore`]: operasi server (HEAD, GET terbatas, PUT, daftar dan hapus semua versi) dengan tanda
//!   tangan header SigV4 lewat hyper HTTP/1.
//! - Endpoint, region, dan gaya pengalamatan dari [`ObjectStorageConfig`]; `http://` ditolak di luar
//!   [`Deployment::Local`] (KEPUTUSAN-235).
//!
//! Batasan yang diketahui (gagal-tertutup): [`ObjectStore`] belum mendukung `https://` (TLS belum
//! diterapkan), sehingga staging/production dengan NEO tidak dapat start sampai klien TLS ditambahkan.
//! Kemampuan NEO atas presigned POST/PUT, region, dan gaya alamat masih `[TIDAK DIKETAHUI]` (spike F1-T03
//! §5); test integrasi membuktikan kode terhadap emulator SeaweedFS, bukan perilaku NEO.
//!
//! Kredensial, URL bertanda tangan, dan isi objek tidak pernah ditulis ke `Debug`, galat, atau log; crate
//! ini tidak menulis log.

mod client;
mod config;
mod error;
mod presign;
mod sigv4;
mod xml;

pub use client::{GetOutcome, ObjectStore, VersionKeysPage, VersionListCursor};
pub use config::{
    AddressingStyle, Deployment, ObjectStorageConfig, SECRET_MIN_LEN, valid_bucket_name,
    valid_object_key,
};
pub use error::ObjectStorageError;
pub use presign::{MAX_PRESIGN_SECONDS, PresignedPost, PresignedPut, Presigner};
