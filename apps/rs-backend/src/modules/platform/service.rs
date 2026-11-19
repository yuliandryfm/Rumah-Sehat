//! Logika kesiapan: siap bila semua pemeriksaan terdaftar lulus.

use super::api::ReadinessCheck;
use std::sync::Arc;

/// Menjalankan semua pemeriksaan berurutan; mencatat nama yang gagal (tanpa detail) dan
/// mengembalikan `true` bila semuanya lulus (daftar kosong berarti siap).
pub(super) async fn is_ready(checks: &[Arc<dyn ReadinessCheck>]) -> bool {
    let mut ready = true;
    for c in checks {
        if !c.check().await {
            tracing::warn!(check = c.name(), "readiness_check_failed");
            ready = false;
        }
    }
    ready
}
