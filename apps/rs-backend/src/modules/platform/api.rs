//! Antarmuka publik modul `platform`: pemeriksaan kesiapan yang dapat didaftarkan modul lain
//! (database, Valkey, dan seterusnya, ditambah task yang memperkenalkannya).

use std::future::Future;
use std::pin::Pin;

/// Hasil pemeriksaan: `true` bila dependensi siap.
pub type ReadinessFuture<'a> = Pin<Box<dyn Future<Output = bool> + Send + 'a>>;

/// Satu pemeriksaan kesiapan. Tidak boleh memuat data pribadi di nama maupun log (ADR-010).
pub trait ReadinessCheck: Send + Sync {
    /// Nama tetap untuk log kegagalan (bukan untuk respons).
    fn name(&self) -> &'static str;
    /// Menjalankan pemeriksaan.
    fn check(&self) -> ReadinessFuture<'_>;
}
