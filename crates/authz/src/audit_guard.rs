//! Penolakan dan pencatatan permintaan ubah/hapus audit log lewat API (US-1504 AC1–AC2, ADR-009,
//! PRD §7.5 "Semua peran: tidak ada yang dapat mengubah atau menghapus"; F1-T68, modul kritis T1).
//!
//! Tidak ada endpoint ubah/hapus audit di OpenAPI mana pun (dijaga test arsitektur `arch-check`,
//! `openapi_audit`). Permintaan `POST`/`PUT`/`PATCH`/`DELETE` ke jalur audit yang tidak punya kebijakan
//! terdaftar (rute tidak ada, atau rute baca audit dengan metode lain) dijawab:
//! - tanpa token atau token tidak sah → `401` (penolakan autentikasi biasa), dan
//! - token sah (peran apa pun, termasuk Super Admin dan Auditor) → `404` yang identik dengan rute tidak ada,
//!
//! dan keduanya mencatat security log `audit_modification_attempt` (TC-US-1504-2–TC-US-1504-4). Permintaan
//! ke rute yang punya kebijakan (misalnya ekspor audit kelak, US-1503) tidak disentuh modul ini.
//!
//! Jalur audit = salah satu segmen path, tanpa membedakan huruf besar-kecil, sama dengan `audit` atau
//! berawalan `audit-`/`audit_` (contoh: `/backoffice/v1/audit-logs/{id}`, `/v1/audit_events`). Segmen
//! seperti `auditors` tidak termasuk. Path ber-percent-encoding tidak didekode: permintaan seperti itu
//! tetap `404` (tidak ada rute), hanya tidak tercatat sebagai kejadian ini.

use axum::http::Method;

/// Metode ubah/hapus yang dianggap percobaan mengubah audit (daftar tertutup).
pub(crate) fn modification_method(method: &Method) -> Option<&'static str> {
    if method == Method::POST {
        Some("POST")
    } else if method == Method::PUT {
        Some("PUT")
    } else if method == Method::PATCH {
        Some("PATCH")
    } else if method == Method::DELETE {
        Some("DELETE")
    } else {
        None
    }
}

/// Benar bila salah satu segmen `path` menandai jalur audit (lihat dokumentasi modul).
pub fn is_audit_path(path: &str) -> bool {
    path.split('/').any(|segment| {
        let s = segment.to_ascii_lowercase();
        s == "audit" || s.starts_with("audit-") || s.starts_with("audit_")
    })
}

/// Metode bila permintaan adalah percobaan ubah/hapus audit (metode ubah/hapus ke jalur audit).
pub(crate) fn modification_attempt(method: &Method, path: &str) -> Option<&'static str> {
    modification_method(method).filter(|_| is_audit_path(path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audit_paths_are_recognised_by_segment() {
        for p in [
            "/backoffice/v1/audit-logs",
            "/backoffice/v1/audit-logs/01936f8e-9c01-7e02-8f03-1a2b3c4d5e6f",
            "/backoffice/v1/AUDIT-EVENTS/x",
            "/v1/audit_events",
            "/audit",
            "/rme/v1/audit/events/",
        ] {
            assert!(is_audit_path(p), "{p}");
        }
        for p in [
            "/backoffice/v1/auditors",
            "/backoffice/v1/contents",
            "/health/live",
            "/backoffice/v1/xaudit-logs",
            "",
            "/",
        ] {
            assert!(!is_audit_path(p), "{p}");
        }
    }

    #[test]
    fn only_modifying_methods_count_as_attempts() {
        let path = "/backoffice/v1/audit-logs/1";
        for (m, label) in [
            (Method::POST, "POST"),
            (Method::PUT, "PUT"),
            (Method::PATCH, "PATCH"),
            (Method::DELETE, "DELETE"),
        ] {
            assert_eq!(modification_attempt(&m, path), Some(label));
        }
        for m in [Method::GET, Method::HEAD, Method::OPTIONS, Method::TRACE] {
            assert_eq!(modification_attempt(&m, path), None, "{m}");
        }
        assert_eq!(
            modification_attempt(&Method::DELETE, "/backoffice/v1/contents/1"),
            None
        );
    }
}
