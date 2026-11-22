//! Pemeriksa kelengkapan event audit untuk test integrasi (F1-T69, US-1501 AC1–AC2, PRD §7.2–§7.3).
//!
//! Setiap test yang dirujuk `audit-event-coverage.toml` membaca payload dari `audit_outbox` lalu memanggil
//! [`assert_event_complete`] (aturan arch-check `audit-coverage` memeriksa panggilan itu ada di badan test).
//! Pemeriksaan memakai `audit::check_payload` yang sama dengan yang dijalankan pustaka sebelum menulis,
//! sehingga yang dibuktikan di sini adalah payload yang benar-benar tersimpan, setelah melewati database.
#![allow(dead_code)] // tiap berkas test memakai sebagian
#![allow(clippy::panic)] // berkas pembantu test

use serde_json::Value;

/// Gagal bila payload kurang field wajib PRD §7.2 untuk kodenya atau memuat data terlarang §7.3. Juga
/// memastikan event membawa `event_code` yang diharapkan pemanggil (pemancar yang diuji).
pub fn assert_event_complete(payload: &Value, expected_code: &str) {
    assert_eq!(
        payload["event_code"].as_str(),
        Some(expected_code),
        "kode event: {payload}"
    );
    let violations = audit::check_payload(payload);
    assert!(
        violations.is_empty(),
        "event {} `{}` tidak lengkap/terlarang: {:?}",
        expected_code,
        payload["action_name"],
        violations
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
    );
}

/// [`assert_event_complete`] untuk setiap event; `expected_codes` = himpunan kode yang boleh muncul.
pub fn assert_events_complete(payloads: &[Value], expected_codes: &[&str]) {
    assert!(!payloads.is_empty(), "tidak ada event audit");
    for p in payloads {
        let code = p["event_code"].as_str().unwrap_or("");
        assert!(
            expected_codes.contains(&code),
            "kode event tak terduga {code}: {p}"
        );
        assert_event_complete(p, code);
    }
}
