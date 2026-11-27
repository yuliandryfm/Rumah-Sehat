//! Pemeriksa kelengkapan event audit untuk test integrasi (F1-T69, US-1501 AC1–AC2, PRD §7.2–§7.3).
//!
//! Setiap test yang dirujuk `audit-event-coverage.toml` membaca payload dari `audit_outbox` lalu memanggil
//! [`assert_event_complete`] (aturan arch-check `audit-coverage` memeriksa panggilan itu ada di badan test).
//! Pemeriksaan memakai `audit::check_payload` yang sama dengan yang dijalankan pustaka sebelum menulis,
//! sehingga yang dibuktikan di sini adalah payload yang benar-benar tersimpan, setelah melewati database.
#![allow(dead_code)] // tiap berkas test memakai sebagian
#![allow(clippy::panic, clippy::unwrap_used, clippy::expect_used)] // berkas pembantu test

use serde_json::Value;

/// Rantai audit untuk event perusahaan `tenant` (F1-T109, US-1501, US-1504 AC3, ADR-009 butir 5): memindahkan
/// SEMUA event outbox ke database audit lewat relay sungguhan (`audit::relay::relay_batch`, kunci tunggal relay),
/// lalu membaca rantai lewat role verifier dan memastikan setiap event bertenant `tenant` ada di rantai dengan
/// `event_hash` = hash ulang (`audit_store::event_hash`) atas `prev_hash` + isi, dan `prev_hash` = `event_hash`
/// event `seq - 1`. Hanya event perusahaan ini yang dinilai (test lain di database yang sama dapat sengaja
/// memanipulasi event miliknya). Mengembalikan jumlah event perusahaan di rantai.
pub async fn relay_and_verify_tenant_chain(tenant: foundation::Uuid) -> usize {
    use audit_store::{PageCursor, event_hash, read_page};
    use platform_jobs::lock::RS_AUDIT_RELAY;
    use platform_jobs::{LockAttempt, SingletonLock};
    use sqlx::{Connection, PgConnection};
    use std::collections::HashMap;

    async fn connect(var: &str) -> PgConnection {
        let url = observability::EnvSource::from_process()
            .required_secret(var)
            .unwrap_or_else(|_| {
                panic!("{var} tidak diset (jalankan lewat scripts/db-ephemeral.sh)")
            });
        PgConnection::connect(url.expose())
            .await
            .unwrap_or_else(|_| panic!("koneksi {var} gagal"))
    }

    let mut conn = connect("RS_TEST_DB_RELAY_URL").await;
    let mut lock = None;
    for _ in 0..300 {
        match SingletonLock::try_acquire(conn, RS_AUDIT_RELAY)
            .await
            .unwrap()
        {
            LockAttempt::Acquired(l) => {
                lock = Some(l);
                break;
            }
            LockAttempt::Busy(c) => {
                conn = c;
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
        }
    }
    let mut lock = lock.expect("kunci relay");
    let mut writer = connect("RS_TEST_DB_AUDIT_WRITER_URL").await;
    loop {
        let b = audit::relay::relay_batch(&mut lock, &mut writer, 50)
            .await
            .unwrap();
        if b.fetched == 0 {
            break;
        }
    }
    lock.release().await.unwrap();

    let mut verifier = connect("RS_TEST_DB_AUDIT_VERIFIER_URL").await;
    let mut hashes = HashMap::new();
    let mut mine = Vec::new();
    let mut cursor = PageCursor::START;
    loop {
        let page = read_page(&mut verifier, cursor, 500).await.unwrap();
        let Some(last) = page.last() else { break };
        cursor = PageCursor {
            seq: last.event.seq,
            event_id: last.event.body.event_id,
        };
        for e in page {
            hashes.insert(e.event.seq, e.event_hash);
            if e.event.body.tenant_id == Some(tenant) {
                mine.push(e);
            }
        }
    }
    for e in &mine {
        let seq = e.event.seq;
        assert_eq!(
            event_hash(&e.prev_hash, &e.event).unwrap(),
            e.event_hash,
            "event_hash event perusahaan di seq {seq} tidak cocok"
        );
        if seq > 1 {
            assert_eq!(
                hashes.get(&(seq - 1)),
                Some(&e.prev_hash),
                "prev_hash seq {seq} tidak menyambung"
            );
        }
    }
    mine.len()
}

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

/// Penanda generik yang tidak pernah boleh ada di payload event korporat mana pun (F1-T109, US-1501 AC2,
/// PRD §7.3, ADR-018, DD A20): URL (termasuk URL unduh bertanda tangan), parameter tanda tangan URL, kunci objek
/// storage (awalan `incoming/` dan `files/`), alamat email, dan kata kredensial. Dicocokkan huruf kecil.
pub const CORPORATE_FORBIDDEN_MARKERS: &[&str] = &[
    "://",
    "x-amz-",
    "signature=",
    "incoming/",
    "files/",
    "@",
    "password",
    "bearer",
];

/// Deret digit terpanjang yang boleh ada di payload event korporat. NIB (13 angka), NPWP (16 angka), NIK (16
/// angka), dan nomor kartu/rekening lengkap lebih panjang; kelompok UUID paling panjang 12 karakter, waktu dan
/// alamat IP terpecah pemisah.
pub const CORPORATE_MAX_DIGIT_RUN: usize = 12;

fn longest_digit_run(text: &str) -> usize {
    let mut best = 0;
    let mut run = 0;
    for c in text.chars() {
        if c.is_ascii_digit() {
            run += 1;
            best = best.max(run);
        } else {
            run = 0;
        }
    }
    best
}

/// Pemeriksa event korporat (F1-T109): setiap payload lengkap menurut [`assert_event_complete`] (kode termasuk
/// `expected_codes`), membawa `tenant_id` = `tenant` (atau kosong bila `None`, mis. penolakan yang tidak boleh
/// membuka perusahaan), dan tidak memuat nilai mentah `raw_values` (NIB, NPWP, email, nama, alamat, telepon,
/// jabatan, nama berkas, isi berkas, URL unduh, kunci objek yang dipakai test) maupun
/// [`CORPORATE_FORBIDDEN_MARKERS`] dan deret digit lebih dari [`CORPORATE_MAX_DIGIT_RUN`].
pub fn assert_corporate_events_clean(
    payloads: &[Value],
    expected_codes: &[&str],
    tenant: Option<&str>,
    raw_values: &[&str],
) {
    assert!(!payloads.is_empty(), "tidak ada event audit korporat");
    for p in payloads {
        let action = &p["action_name"];
        let code = p["event_code"].as_str().unwrap_or("");
        assert!(
            expected_codes.contains(&code),
            "kode event tak terduga {code}: {p}"
        );
        assert_event_complete(p, code);
        match tenant {
            Some(t) => assert_eq!(p["tenant_id"].as_str(), Some(t), "tenant {action}: {p}"),
            None => assert!(
                p["tenant_id"].is_null(),
                "tenant harus kosong {action}: {p}"
            ),
        }
        let dump = p.to_string().to_lowercase();
        // Nama aksi adalah kode tertutup dari kode sumber (mis. E-04 `password_changed`); penanda dicari di luar itu.
        let mut without_action = p.clone();
        without_action["action_name"] = Value::Null;
        let marker_dump = without_action.to_string().to_lowercase();
        for marker in CORPORATE_FORBIDDEN_MARKERS {
            assert!(
                !marker_dump.contains(marker),
                "{action} memuat penanda terlarang {marker:?}: {p}"
            );
        }
        for raw in raw_values.iter().filter(|r| !r.trim().is_empty()) {
            assert!(
                !dump.contains(&raw.to_lowercase()),
                "{action} memuat nilai mentah {raw:?}: {p}"
            );
        }
        assert!(
            longest_digit_run(&dump) <= CORPORATE_MAX_DIGIT_RUN,
            "{action} memuat deret digit panjang (NIB/NPWP/NIK/nomor): {p}"
        );
    }
}

/// Pemeriksa negatif untuk [`assert_corporate_events_clean`] sendiri: payload sah lolos, dan setiap jenis
/// kebocoran (penanda, nilai mentah, deret digit, tenant salah) ditolak. Dipanggil dari test korporat.
pub fn corporate_checker_rejects_leaks(valid: &Value, tenant: &str) {
    assert_corporate_events_clean(
        std::slice::from_ref(valid),
        &["e_10", "e_11", "e_13", "e_16"],
        Some(tenant),
        &[],
    );
    let leaks = [
        (
            "user_agent",
            Value::from("Mozilla https://s3.example/x?X-Amz-Signature=ab"),
        ),
        ("user_agent", Value::from("hr@example.com")),
        ("user_agent", Value::from("incoming/0190f5c2")),
        ("user_agent", Value::from("npwp 0123456789012345")),
        ("tenant_id", Value::Null),
    ];
    for (field, bad) in leaks {
        let mut p = valid.clone();
        p[field] = bad;
        let caught = std::panic::catch_unwind(|| {
            assert_corporate_events_clean(
                std::slice::from_ref(&p),
                &["e_10", "e_11", "e_13", "e_16"],
                Some(tenant),
                &[],
            )
        });
        assert!(caught.is_err(), "kebocoran {field} tidak tertangkap");
    }
    let caught = std::panic::catch_unwind(|| {
        assert_corporate_events_clean(
            std::slice::from_ref(valid),
            &["e_10", "e_11", "e_13", "e_16"],
            Some(tenant),
            &["corporate"],
        )
    });
    assert!(caught.is_err(), "nilai mentah tidak tertangkap");
}
