//! Endpoint Verifikator `/backoffice/v1/identity-verifications` (F1-T62, reference implementation US-201)
//! lewat HTTP sungguhan: router lengkap proses `api` (`build_router`: penegak kebijakan, validator token
//! realm `admin` dan `pasien` dengan kunci realm uji, resolver akun dan penjaga sesi modul identity di atas
//! PostgreSQL, `ClientIp` dari peer TCP), PostgreSQL efemeral (`scripts/db-ephemeral.sh`, role runtime
//! `rs_api`), SeaweedFS dan Valkey efemeral (Docker). Pengajuan dibuat lewat endpoint pasien (F1-T61).
//!
//! Skenario: antrean terlama lebih dulu, detail NIK lengkap dengan E-05, URL foto dengan E-05 lebih dulu,
//! setujui/tolak (E-07, tujuh kode alasan, keterangan terenkripsi tampil ke pasien), keputusan pertama
//! menang (bersamaan dan berurutan; pihak kalah E-07 hasil gagal), NIK pemilik akun bentrok (AC7, termasuk
//! dua persetujuan bersamaan), registri NIK lama di-soft-delete saat NIK baru disetujui, dan penolakan akses:
//! tanpa token / realm lain → 401, sub-peran lain (termasuk Super Admin) → 403, akun nonaktif dan sesi
//! dicabut → 401, ID tidak ada atau bukan UUID → 404. Tanpa variabel database atau Docker, test GAGAL.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)] // berkas test

mod common;
mod files;
mod identity_support;
mod valkey_support;

use authz::{AdminSubRole, DenialSink};
use common::{ADMIN_AUD, ADMIN_ISS, PASIEN_AUD, PASIEN_ISS, Resp, TestKeys, request, request_json};
use foundation::{SystemClock, Uuid};
use identity_support::{
    Env, Patient, audit_events, new_admin, new_family_member, new_patient, set_staff_nik,
    synthetic_nik,
};
use observability::EnvSource;
use rs_backend::access_audit::AuditDenialRecorder;
use rs_backend::app::{AppServices, AppState, build_router};
use rs_backend::modules::identity::api::{AccountDirectory, RevocationReason};
use serde_json::{Value, json};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

static SERIAL: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

const BASE: &str = "/backoffice/v1/identity-verifications";
const REASONS: &str = "/backoffice/v1/identity-verification-reasons";
const SEVEN: [&str; 7] = [
    "photo_unreadable",
    "document_invalid",
    "nik_mismatch",
    "profile_mismatch",
    "guardian_not_listed",
    "nik_unavailable",
    "other",
];
const GENERAL_TEXT: &str =
    "NIK ini tidak dapat dipakai untuk akun Anda. Hubungi Layanan Pelanggan bila ini NIK Anda.";

struct Fixture {
    env: Env,
    addr: SocketAddr,
    keys: TestKeys,
    directory: Arc<AccountDirectory>,
    /// Pembaca `audit_outbox` (role relay; role aplikasi hanya INSERT).
    relay: PgPool,
}

async fn relay_pool() -> PgPool {
    let url = EnvSource::from_process()
        .required_secret("RS_TEST_DB_RELAY_URL")
        .unwrap_or_else(|_| {
            panic!("RS_TEST_DB_RELAY_URL tidak diset: jalankan lewat scripts/db-ephemeral.sh")
        });
    PgPoolOptions::new()
        .max_connections(2)
        .connect(url.expose())
        .await
        .unwrap()
}

impl Fixture {
    async fn new() -> Self {
        let env = Env::start().await;
        let keys = TestKeys::generate();
        let state = AppState::new(
            common::TYPE_BASE,
            Vec::new(),
            AppServices {
                pool: env.pool.clone(),
                clock: Arc::clone(&env.clock),
                files: Arc::clone(&env.storage),
                identity: Arc::new(env.services()),
                public_content_limit: common::roomy_limiter(),
                public_image_limit: common::roomy_limiter(),
            },
        );
        let auth = common::auth_state(&keys, Arc::new(SystemClock));
        let directory = common::account_directory(env.pool.clone());
        let denials = DenialSink::start(
            64,
            2,
            Some(Arc::new(AuditDenialRecorder::new(
                env.pool.clone(),
                Arc::new(SystemClock),
            ))),
        )
        .unwrap();
        let app = build_router(state, auth, directory.clone(), directory.clone(), denials).unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            app.serve_with_peer_addr(listener, std::future::pending())
                .await
                .unwrap();
        });
        Self {
            env,
            addr,
            keys,
            directory,
            relay: relay_pool().await,
        }
    }

    fn patient_token(&self, p: &Patient) -> String {
        common::token(&self.keys.pasien, PASIEN_ISS, PASIEN_AUD, &p.subject)
    }

    fn admin_token(&self, subject: &str) -> String {
        common::token(&self.keys.admin, ADMIN_ISS, ADMIN_AUD, subject)
    }

    /// Akun Verifikator baru dengan NIK staf sintetis acak (US-201 AC8) dan tokennya.
    async fn verifier(&self) -> (Uuid, String) {
        self.verifier_with_nik(&synthetic_nik(), 2).await
    }

    /// Akun Verifikator dengan NIK staf `nik` (blind index versi kunci `key_version`).
    async fn verifier_with_nik(&self, nik: &str, key_version: i16) -> (Uuid, String) {
        let (id, sub) = new_admin(&self.env.pool, &[AdminSubRole::Verifier]).await;
        set_staff_nik(&self.env, id, nik, key_version).await;
        (id, self.admin_token(&sub))
    }

    async fn get(&self, path: &str, token: &str) -> Resp {
        request(
            self.addr,
            "GET",
            path,
            &[&format!("Authorization: Bearer {token}")],
        )
        .await
    }

    async fn post(&self, path: &str, token: &str, body: &Value) -> Resp {
        request_json(
            self.addr,
            "POST",
            path,
            &[&format!("Authorization: Bearer {token}")],
            body,
        )
        .await
    }

    /// Pengajuan NIK + foto KTP lewat endpoint pasien; mengembalikan ID permintaan.
    async fn submit(&self, p: &Patient, care_recipient_id: Uuid, nik: &str) -> Uuid {
        let slot = self.env.uploaded_slot(p.account_id).await;
        let r = self
            .post(
                &format!("/patient/v1/care-recipients/{care_recipient_id}/identity-submissions"),
                &self.patient_token(p),
                &json!({ "nik": nik, "upload_slot_id": slot.hyphenated().to_string() }),
            )
            .await;
        assert_eq!(r.status, 201, "{}", r.body);
        r.json()["data"]["identity_verification_request_id"]
            .as_str()
            .unwrap()
            .parse()
            .unwrap()
    }

    async fn patient_status(&self, p: &Patient, care_recipient_id: Uuid) -> Value {
        let r = self
            .get(
                &format!("/patient/v1/care-recipients/{care_recipient_id}/identity"),
                &self.patient_token(p),
            )
            .await;
        assert_eq!(r.status, 200, "{}", r.body);
        r.json()["data"].clone()
    }

    async fn approve(&self, token: &str, id: Uuid) -> Resp {
        self.post(&format!("{BASE}/{id}/approve"), token, &json!({}))
            .await
    }

    async fn reject(&self, token: &str, id: Uuid, body: &Value) -> Resp {
        self.post(&format!("{BASE}/{id}/reject"), token, body).await
    }
}

fn code(r: &Resp) -> String {
    r.json()["response_code"]
        .as_str()
        .unwrap_or_default()
        .to_string()
}

/// Jumlah event outbox berkode `event_code` dan aksi `action` untuk subjek `subject` dengan hasil `outcome`.
async fn outbox_count(f: &Fixture, event_code: &str, action: &str, subject: Uuid) -> i64 {
    sqlx::query_scalar!(
        r#"SELECT count(*) AS "n!" FROM app.audit_outbox
           WHERE payload ->> 'event_code' = $1 AND payload ->> 'action_name' = $2
             AND payload -> 'subjects' -> 0 ->> 'subject_id' = $3"#,
        event_code,
        action,
        subject.hyphenated().to_string(),
    )
    .fetch_one(&f.relay)
    .await
    .unwrap()
}

async fn request_status(f: &Fixture, id: Uuid) -> String {
    sqlx::query_scalar!(
        "SELECT status FROM app.identity_verification_request_state \
         WHERE identity_verification_request_id = $1",
        id
    )
    .fetch_one(&f.env.pool)
    .await
    .unwrap()
}

/// Antrean lengkap (semua halaman) sebagai daftar ID berurutan.
async fn whole_queue(f: &Fixture, token: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cursor: Option<String> = None;
    loop {
        let path = match &cursor {
            Some(c) => format!("{BASE}?limit=2&cursor={c}"),
            None => format!("{BASE}?limit=2"),
        };
        let r = f.get(&path, token).await;
        assert_eq!(r.status, 200, "{}", r.body);
        let j = r.json();
        for item in j["data"].as_array().unwrap() {
            assert!(item.get("nik").is_none(), "antrean hanya 4 digit terakhir");
            out.push(
                item["identity_verification_request_id"]
                    .as_str()
                    .unwrap()
                    .to_string(),
            );
        }
        if j["meta"]["has_more"] == true {
            cursor = Some(j["meta"]["next_cursor"].as_str().unwrap().to_string());
        } else {
            break;
        }
    }
    out
}

// trace: US-201 AC1, AC2, KU-06 (antrean terlama lebih dulu dengan cursor; detail NIK lengkap dengan E-05
// lebih dulu; URL foto dengan E-05 lebih dulu; setuju → Disetujui, E-07 tanpa NIK lengkap), TC-US-201-1,
// -3, -5, -19
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn queue_detail_photo_and_approve() {
    let _g = SERIAL.lock().await;
    let f = Fixture::new().await;
    let (verifier_id, t) = f.verifier().await;
    let mut ids = Vec::new();
    let mut people = Vec::new();
    for _ in 0..3 {
        let p = new_patient(&f.env.pool).await;
        let nik = synthetic_nik();
        ids.push(f.submit(&p, p.owner_care_recipient_id, &nik).await);
        people.push((p, nik));
    }
    let queue = whole_queue(&f, &t).await;
    let positions: Vec<usize> = ids
        .iter()
        .map(|id| {
            queue
                .iter()
                .position(|q| *q == id.hyphenated().to_string())
                .unwrap()
        })
        .collect();
    assert!(
        positions.windows(2).all(|w| w[0] < w[1]),
        "terlama lebih dulu: {positions:?}"
    );

    let (p, nik) = &people[0];
    let cr = p.owner_care_recipient_id;
    let r = f.get(&format!("{BASE}/{}", ids[0]), &t).await;
    assert_eq!(r.status, 200, "{}", r.body);
    assert_eq!(r.header("cache-control"), Some("no-store"));
    let d = r.json()["data"].clone();
    assert_eq!(
        d["nik"],
        nik.as_str(),
        "NIK lengkap hanya untuk Verifikator"
    );
    assert_eq!(d["status"], "pending");
    assert_eq!(d["can_decide"], true);
    assert_eq!(d["document_type"], "ktp");
    assert_eq!(d["birth_date"], "1990-01-01");
    assert_eq!(
        outbox_count(&f, "e_05", "identity_document_viewed", cr).await,
        1
    );

    let before = outbox_count(&f, "e_05", "stored_file_opened", cr).await;
    let r = f
        .post(
            &format!("{BASE}/{}/document-photo-url", ids[0]),
            &t,
            &json!({}),
        )
        .await;
    assert_eq!(r.status, 200, "{}", r.body);
    assert!(
        r.json()["data"]["url"]
            .as_str()
            .unwrap()
            .starts_with("http")
    );
    assert_eq!(r.json()["data"]["mime_type"], "image/jpeg");
    assert_eq!(
        outbox_count(&f, "e_05", "stored_file_opened", cr).await,
        before + 1,
        "E-05 tercatat sebelum URL terbit"
    );

    let r = f.approve(&t, ids[0]).await;
    assert_eq!(r.status, 200, "{}", r.body);
    assert_eq!(r.json()["data"]["status"], "approved");
    assert_eq!(f.patient_status(p, cr).await["status"], "approved");
    let registry: i64 = sqlx::query_scalar!(
        r#"SELECT count(*) AS "n!" FROM app.account_owner_nik_registry
           WHERE care_recipient_id = $1 AND deleted_at IS NULL"#,
        cr
    )
    .fetch_one(&f.env.pool)
    .await
    .unwrap();
    assert_eq!(registry, 1);

    // Tidak lagi di antrean; detail tidak dapat diputuskan lagi.
    assert!(
        !whole_queue(&f, &t)
            .await
            .contains(&ids[0].hyphenated().to_string())
    );
    let r = f.get(&format!("{BASE}/{}", ids[0]), &t).await;
    assert_eq!(r.json()["data"]["can_decide"], false);
    assert!(r.json()["data"]["decided_at"].is_string());

    let events = audit_events("e_07", cr).await;
    let approved: Vec<_> = events
        .iter()
        .filter(|e| e.event.body.action_name == "identity_verification_approved")
        .collect();
    assert_eq!(approved.len(), 1);
    let body = &approved[0].event.body;
    assert_eq!(body.outcome, "success");
    assert_eq!(body.actor_account_id, Some(verifier_id));
    assert_eq!(body.object_type, "identity_verification_request");
    assert_eq!(body.source_ip.map(|ip| ip.is_loopback()), Some(true));
    let raw = serde_json::to_string(&body.after_values).unwrap()
        + &serde_json::to_string(&body.before_values).unwrap();
    assert!(!raw.contains(nik.as_str()), "NIK lengkap tidak masuk audit");
}

// trace: US-201 AC3, AC3a (tujuh kode alasan hasil migrasi data, keterangan terenkripsi tampil utuh ke
// pasien, keterangan wajib untuk other, 500/501 karakter, kode di luar daftar ditolak tanpa mengubah
// status), TC-US-201-7, -8, -9, -20, -21, -22
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn reject_reasons_and_note_rules() {
    let _g = SERIAL.lock().await;
    let f = Fixture::new().await;
    let (_vid, t) = f.verifier().await;

    let r = f.get(REASONS, &t).await;
    assert_eq!(r.status, 200, "{}", r.body);
    let codes: Vec<String> = r.json()["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x["code"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(codes, SEVEN, "tepat tujuh kode, urutan PRD");
    assert_eq!(r.json()["data"][5]["label"], GENERAL_TEXT);
    // TC-US-201-21: tidak ada endpoint ubah daftar alasan.
    for m in ["POST", "PUT", "PATCH", "DELETE"] {
        let r = request(f.addr, m, REASONS, &[&format!("Authorization: Bearer {t}")]).await;
        assert_eq!(r.status, 404, "{m}");
    }

    let p = new_patient(&f.env.pool).await;
    let cr = p.owner_care_recipient_id;
    let id = f.submit(&p, cr, &synthetic_nik()).await;
    let max = "é".repeat(500);
    let bad = [
        json!({ "reason_code": "fraud" }),
        json!({ "reason_code": "" }),
        json!({ "reason_code": "PHOTO_UNREADABLE" }),
        json!({ "note": "tanpa alasan" }),
        json!({ "reason_code": "other" }),
        json!({ "reason_code": "other", "note": "   " }),
        json!({ "reason_code": "other", "note": format!("{max}a") }),
        json!({ "reason_code": "photo_unreadable", "extra": true }),
    ];
    for b in &bad {
        let r = f.reject(&t, id, b).await;
        assert_eq!(
            (r.status, code(&r).as_str()),
            (422, "4220001"),
            "{b}: {}",
            r.body
        );
        assert_eq!(request_status(&f, id).await, "pending", "{b}");
    }
    let r = f
        .reject(&t, id, &json!({ "reason_code": "other", "note": max }))
        .await;
    assert_eq!(r.status, 200, "{}", r.body);
    assert_eq!(r.json()["data"]["rejection_reason"], "other");
    let s = f.patient_status(&p, cr).await;
    assert_eq!(s["status"], "rejected");
    assert_eq!(s["rejection_reason"], "other");
    assert_eq!(
        s["rejection_note"],
        max.as_str(),
        "500 karakter tampil utuh"
    );
    assert_eq!(s["can_submit"], true, "pasien dapat unggah ulang");
    // Keterangan terenkripsi (amplop v1) dengan DEK wadah status_note.
    let row = sqlx::query!(
        r#"SELECT e.note_enc AS "note_enc!", k.container_type
           FROM app.identity_verification_request_status_event e
           JOIN app.data_key k ON k.id = e.note_data_key_id
           WHERE e.identity_verification_request_id = $1 AND e.to_status = 'rejected'"#,
        id
    )
    .fetch_one(&f.env.pool)
    .await
    .unwrap();
    assert_eq!(row.container_type, "status_note");
    assert_eq!(row.note_enc[0], 1);
    assert!(!files::contains(&row.note_enc, "éé".as_bytes()));
    let events = audit_events("e_07", cr).await;
    let raw = serde_json::to_string(&events[0].event.body.after_values).unwrap();
    assert!(
        raw.contains("other") && !raw.contains("éé"),
        "keterangan tidak masuk audit"
    );

    // Setiap kode alasan dapat disimpan (satu pengajuan per kode).
    for reason in SEVEN {
        let id = f.submit(&p, cr, &synthetic_nik()).await;
        let body = if reason == "other" {
            json!({ "reason_code": reason, "note": "Keterangan sintetis" })
        } else {
            json!({ "reason_code": reason })
        };
        let r = f.reject(&t, id, &body).await;
        assert_eq!(r.status, 200, "{reason}: {}", r.body);
        let s = f.patient_status(&p, cr).await;
        assert_eq!(s["rejection_reason"], reason);
        if reason != "other" {
            assert!(s["rejection_note"].is_null());
        }
    }
}

// trace: US-201 AC5, AB-36 (dua Verifikator memutuskan bersamaan: hanya keputusan pertama tersimpan, yang
// kedua 4090204 dan E-07 hasil gagal; keputusan setelahnya juga ditolak), TC-US-201-12, -13
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn first_decision_wins() {
    let _g = SERIAL.lock().await;
    let f = Fixture::new().await;
    let (_a, ta) = f.verifier().await;
    let (_b, tb) = f.verifier().await;
    let p = new_patient(&f.env.pool).await;
    let cr = p.owner_care_recipient_id;
    let id = f.submit(&p, cr, &synthetic_nik()).await;
    let reject_body = json!({ "reason_code": "photo_unreadable" });
    let (r1, r2) = tokio::join!(f.approve(&ta, id), f.reject(&tb, id, &reject_body));
    let mut statuses = [r1.status, r2.status];
    statuses.sort_unstable();
    assert_eq!(statuses, [200, 409], "{} / {}", r1.body, r2.body);
    let loser = if r1.status == 409 { &r1 } else { &r2 };
    assert_eq!(code(loser), "4090204");
    assert_eq!(
        loser.json()["response_description"],
        "Pengajuan sudah diputuskan"
    );
    let winner_status = if r1.status == 200 {
        "approved"
    } else {
        "rejected"
    };
    assert_eq!(request_status(&f, id).await, winner_status);

    let r = f.approve(&tb, id).await;
    assert_eq!((r.status, code(&r).as_str()), (409, "4090204"));
    assert_eq!(request_status(&f, id).await, winner_status);

    let events = audit_events("e_07", cr).await;
    let ok = events
        .iter()
        .filter(|e| e.event.body.outcome == "success")
        .count();
    let failed: Vec<_> = events
        .iter()
        .filter(|e| e.event.body.outcome == "failure")
        .collect();
    assert_eq!(ok, 1);
    assert_eq!(failed.len(), 2);
    assert!(
        failed
            .iter()
            .all(|e| e.event.body.outcome_reason_code.as_deref() == Some("already_decided"))
    );
}

// trace: US-201 AC7, UP-40 (NIK sudah disetujui sebagai pemilik akun lain: persetujuan ditolak 4090203,
// status tetap menunggu, E-07 gagal; lalu tolak nik_unavailable → teks umum ke pasien; dua persetujuan
// NIK sama bersamaan → tepat satu disetujui; anggota keluarga boleh), TC-US-201-14, -17
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn owner_nik_held_by_other_account() {
    let _g = SERIAL.lock().await;
    let f = Fixture::new().await;
    let (_v, t) = f.verifier().await;
    let a = new_patient(&f.env.pool).await;
    let b = new_patient(&f.env.pool).await;
    let nik = synthetic_nik();
    let ida = f.submit(&a, a.owner_care_recipient_id, &nik).await;
    let idb = f.submit(&b, b.owner_care_recipient_id, &nik).await;
    assert_eq!(f.approve(&t, ida).await.status, 200);

    let r = f.approve(&t, idb).await;
    assert_eq!(
        (r.status, code(&r).as_str()),
        (409, "4090203"),
        "{}",
        r.body
    );
    assert_eq!(
        r.json()["response_description"],
        "NIK sudah disetujui sebagai pemilik akun lain. Tolak pengajuan dengan alasan nik_unavailable."
    );
    assert_eq!(request_status(&f, idb).await, "pending");
    let r = f
        .reject(&t, idb, &json!({ "reason_code": "nik_unavailable" }))
        .await;
    assert_eq!(r.status, 200, "{}", r.body);
    let s = f.patient_status(&b, b.owner_care_recipient_id).await;
    assert_eq!(s["status"], "rejected");
    assert_eq!(s["rejection_reason"], "nik_unavailable");
    let events = audit_events("e_07", b.owner_care_recipient_id).await;
    assert!(events.iter().any(|e| e.event.body.outcome == "failure"
        && e.event.body.outcome_reason_code.as_deref() == Some("nik_unavailable")));
    assert!(events.iter().any(|e| e.event.body.outcome == "success"
        && e.event.body.action_name == "identity_verification_rejected"));

    // Anggota keluarga dengan NIK yang sama di akun lain tetap dapat disetujui (UP-40).
    let e = new_patient(&f.env.pool).await;
    let family = new_family_member(&f.env.pool, e.account_id).await;
    let idf = f.submit(&e, family, &nik).await;
    assert_eq!(f.approve(&t, idf).await.status, 200);

    // Dua akun, NIK sama, keduanya menunggu, disetujui bersamaan: tepat satu berhasil.
    let c = new_patient(&f.env.pool).await;
    let d = new_patient(&f.env.pool).await;
    let shared = synthetic_nik();
    let idc = f.submit(&c, c.owner_care_recipient_id, &shared).await;
    let idd = f.submit(&d, d.owner_care_recipient_id, &shared).await;
    let (_w, t2) = f.verifier().await;
    let (r1, r2) = tokio::join!(f.approve(&t, idc), f.approve(&t2, idd));
    let mut got = [(r1.status, code(&r1)), (r2.status, code(&r2))];
    got.sort();
    assert_eq!(
        got[0],
        (200, "2000000".to_string()),
        "{} / {}",
        r1.body,
        r2.body
    );
    assert_eq!(got[1], (409, "4090203".to_string()));
    let approved = [request_status(&f, idc).await, request_status(&f, idd).await];
    assert_eq!(approved.iter().filter(|s| *s == "approved").count(), 1);
}

// trace: US-106 AC1 (NIK lama tetap tercatat sampai NIK baru disetujui), ADR-003 (registri NIK lama
// di-soft-delete saat NIK baru penerima layanan yang sama disetujui; NIK lama bebas dipakai akun lain)
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn new_approval_retires_previous_registry_row() {
    let _g = SERIAL.lock().await;
    let f = Fixture::new().await;
    let (v, t) = f.verifier().await;
    let a = new_patient(&f.env.pool).await;
    let cr = a.owner_care_recipient_id;
    let old = synthetic_nik();
    let first = f.submit(&a, cr, &old).await;
    assert_eq!(f.approve(&t, first).await.status, 200);
    let second = f.submit(&a, cr, &synthetic_nik()).await;
    assert_eq!(f.approve(&t, second).await.status, 200);
    let rows = sqlx::query!(
        "SELECT identity_verification_request_id, deleted_at, deleted_by, delete_reason \
         FROM app.account_owner_nik_registry WHERE care_recipient_id = $1 ORDER BY created_at",
        cr
    )
    .fetch_all(&f.env.pool)
    .await
    .unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].identity_verification_request_id, first);
    assert!(rows[0].deleted_at.is_some());
    assert_eq!(rows[0].deleted_by, Some(v));
    assert_eq!(
        rows[0].delete_reason.as_deref(),
        Some("superseded_by_new_approval")
    );
    assert_eq!(rows[1].identity_verification_request_id, second);
    assert!(rows[1].deleted_at.is_none());

    // NIK lama kini tidak dipegang siapa pun: akun lain dapat disetujui dengannya.
    let b = new_patient(&f.env.pool).await;
    let idb = f.submit(&b, b.owner_care_recipient_id, &old).await;
    assert_eq!(f.approve(&t, idb).await.status, 200);
}

/// Semua endpoint Verifikator untuk permintaan `id`.
fn endpoints(id: Uuid) -> Vec<(&'static str, String, Option<Value>)> {
    vec![
        ("GET", BASE.to_string(), None),
        ("GET", format!("{BASE}/{id}"), None),
        (
            "POST",
            format!("{BASE}/{id}/document-photo-url"),
            Some(json!({})),
        ),
        ("POST", format!("{BASE}/{id}/approve"), Some(json!({}))),
        (
            "POST",
            format!("{BASE}/{id}/reject"),
            Some(json!({ "reason_code": "photo_unreadable" })),
        ),
        ("GET", REASONS.to_string(), None),
    ]
}

async fn call(f: &Fixture, e: &(&str, String, Option<Value>), token: Option<&str>) -> Resp {
    let headers: Vec<String> = token
        .map(|t| vec![format!("Authorization: Bearer {t}")])
        .unwrap_or_default();
    let refs: Vec<&str> = headers.iter().map(String::as_str).collect();
    match &e.2 {
        None => request(f.addr, e.0, &e.1, &refs).await,
        Some(b) => request_json(f.addr, e.0, &e.1, &refs, b).await,
    }
}

// trace: US-201 AC6, KU-03, KU-10, coding-standards §7.3 (tanpa token, token realm pasien → 401; setiap
// sub-peran selain Verifikator termasuk Super Admin → 403; akun nonaktif → 4010002; sesi dicabut → 401;
// ID tidak ada atau bukan UUID → 404; tidak ada efek), TC-US-201-15, -18
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn access_is_denied_to_everyone_but_active_verifiers() {
    let _g = SERIAL.lock().await;
    let f = Fixture::new().await;
    let p = new_patient(&f.env.pool).await;
    let id = f
        .submit(&p, p.owner_care_recipient_id, &synthetic_nik())
        .await;

    for e in endpoints(id) {
        let r = call(&f, &e, None).await;
        assert_eq!(
            (r.status, code(&r).as_str()),
            (401, "4010001"),
            "{} {}",
            e.0,
            e.1
        );
        let r = call(&f, &e, Some(&f.patient_token(&p))).await;
        assert_eq!(
            (r.status, code(&r).as_str()),
            (401, "4010001"),
            "pasien {} {}",
            e.0,
            e.1
        );
    }
    for role in AdminSubRole::ALL
        .into_iter()
        .filter(|r| *r != AdminSubRole::Verifier)
    {
        let (_id, sub) = new_admin(&f.env.pool, &[role]).await;
        let tok = f.admin_token(&sub);
        for e in endpoints(id) {
            let r = call(&f, &e, Some(&tok)).await;
            assert_eq!(
                (r.status, code(&r).as_str()),
                (403, "4030001"),
                "{role:?} {} {}",
                e.0,
                e.1
            );
        }
    }

    let (inactive, sub) = new_admin(&f.env.pool, &[AdminSubRole::Verifier]).await;
    sqlx::query!(
        "UPDATE app.account SET status = 'inactive', status_changed_at = now() WHERE id = $1",
        inactive
    )
    .execute(&f.env.pool)
    .await
    .unwrap();
    for e in endpoints(id) {
        let r = call(&f, &e, Some(&f.admin_token(&sub))).await;
        assert_eq!(
            (r.status, code(&r).as_str()),
            (401, "4010002"),
            "{} {}",
            e.0,
            e.1
        );
    }

    let (revoked, rsub) = new_admin(&f.env.pool, &[AdminSubRole::Verifier]).await;
    let sid = format!("sesi-dicabut-{}", files::random_hex(6));
    let now = foundation::Clock::now(&SystemClock).unix_timestamp();
    let revoked_token = f
        .keys
        .admin
        .sign(&json!({
            "iss": ADMIN_ISS, "aud": ADMIN_AUD, "sub": rsub, "typ": "Bearer",
            "sid": sid, "azp": "rs-backoffice-bff", "iat": now - 5, "exp": now + 300,
        }))
        .unwrap();
    let mut tx = f.env.pool.begin().await.unwrap();
    f.directory
        .record_revocation(
            &mut tx,
            revoked,
            &sid,
            RevocationReason::Logout,
            foundation::Clock::now(&SystemClock) + time::Duration::hours(1),
        )
        .await
        .unwrap();
    tx.commit().await.unwrap();
    for e in endpoints(id) {
        let r = call(&f, &e, Some(&revoked_token)).await;
        assert_eq!(
            (r.status, code(&r).as_str()),
            (401, "4010001"),
            "{} {}",
            e.0,
            e.1
        );
    }
    assert_eq!(request_status(&f, id).await, "pending", "tidak ada efek");

    // ID tidak ada atau bukan UUID → 404 untuk Verifikator sah.
    let (_v, t) = f.verifier().await;
    for target in [
        foundation::new_id().hyphenated().to_string(),
        "bukan-uuid".to_string(),
    ] {
        for (m, path, body) in [
            ("GET", format!("{BASE}/{target}"), None),
            (
                "POST",
                format!("{BASE}/{target}/document-photo-url"),
                Some(json!({})),
            ),
            ("POST", format!("{BASE}/{target}/approve"), Some(json!({}))),
            (
                "POST",
                format!("{BASE}/{target}/reject"),
                Some(json!({ "reason_code": "photo_unreadable" })),
            ),
        ] {
            let r = call(&f, &(m, path.clone(), body), Some(&t)).await;
            assert_eq!(
                (r.status, code(&r).as_str()),
                (404, "4040001"),
                "{m} {path}"
            );
        }
    }
    let r = f.get(&format!("{BASE}?cursor=bukan-uuid"), &t).await;
    assert_eq!((r.status, code(&r).as_str()), (422, "4220001"));
    let r = f.get(&format!("{BASE}?limit=101"), &t).await;
    assert_eq!((r.status, code(&r).as_str()), (422, "4220001"));
}

// trace: US-201 AC8, KEPUTUSAN-294 (Verifikator yang NIK stafnya = NIK pemilik akun pasien tidak dapat
// memutuskan pengajuan akun itu: tombol tidak tersedia, API 4090205, status tetap, E-07 gagal beraktor
// Verifikator; Verifikator lain dapat; berlaku juga untuk anggota keluarga dan lintas versi kunci blind index;
// Verifikator tanpa NIK staf tidak dapat memutuskan apa pun), TC-US-201-24, -25, -27
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn verifier_cannot_decide_own_account() {
    let _g = SERIAL.lock().await;
    let f = Fixture::new().await;
    let own = synthetic_nik();
    // NIK staf V tersimpan di versi kunci 1; registri NIK pasien ditulis versi 2 (rotasi, TD-10).
    let (v, tv) = f.verifier_with_nik(&own, 1).await;
    let (w, tw) = f.verifier().await;
    let p = new_patient(&f.env.pool).await;
    let cr = p.owner_care_recipient_id;

    // Pengajuan pertama P (belum ada NIK disetujui) memakai NIK staf V.
    let first = f.submit(&p, cr, &own).await;
    let r = f.get(&format!("{BASE}/{first}"), &tv).await;
    assert_eq!(r.status, 200, "{}", r.body);
    assert_eq!(
        r.json()["data"]["can_decide"],
        false,
        "tombol tidak tersedia"
    );
    let r = f.get(&format!("{BASE}/{first}"), &tw).await;
    assert_eq!(r.json()["data"]["can_decide"], true);
    let denied = |r: &Resp| (r.status, code(r));
    let r = f.approve(&tv, first).await;
    assert_eq!(denied(&r), (409, "4090205".to_string()), "{}", r.body);
    assert_eq!(
        r.json()["response_description"],
        "Pengajuan ini tidak dapat Anda putuskan (pemisahan tugas). Pengajuan diputuskan Verifikator lain."
    );
    let r = f
        .reject(&tv, first, &json!({ "reason_code": "photo_unreadable" }))
        .await;
    assert_eq!(denied(&r), (409, "4090205".to_string()), "{}", r.body);
    assert_eq!(
        request_status(&f, first).await,
        "pending",
        "status tidak berubah"
    );

    // TC-US-201-27: Verifikator lain memutuskan.
    assert_eq!(f.approve(&tw, first).await.status, 200);

    // Pengajuan baru P dengan NIK lain: NIK pemilik akun yang disetujui tetap NIK staf V.
    let second = f.submit(&p, cr, &synthetic_nik()).await;
    let r = f.approve(&tv, second).await;
    assert_eq!(denied(&r), (409, "4090205".to_string()), "{}", r.body);
    // Anggota keluarga di akun P juga tidak dapat diputuskan V.
    let family = new_family_member(&f.env.pool, p.account_id).await;
    let fam = f.submit(&p, family, &synthetic_nik()).await;
    let r = f
        .reject(&tv, fam, &json!({ "reason_code": "photo_unreadable" }))
        .await;
    assert_eq!(denied(&r), (409, "4090205".to_string()), "{}", r.body);
    assert_eq!(f.approve(&tw, fam).await.status, 200);

    let events = audit_events("e_07", cr).await;
    let failed: Vec<_> = events
        .iter()
        .filter(|e| e.event.body.outcome == "failure")
        .collect();
    assert_eq!(failed.len(), 3);
    for e in &failed {
        assert_eq!(e.event.body.actor_account_id, Some(v));
        assert_eq!(
            e.event.body.outcome_reason_code.as_deref(),
            Some("separation_of_duties")
        );
    }
    assert!(
        events
            .iter()
            .any(|e| e.event.body.outcome == "success" && e.event.body.actor_account_id == Some(w))
    );

    // V tetap dapat memutuskan pengajuan akun lain.
    let q = new_patient(&f.env.pool).await;
    let other = f
        .submit(&q, q.owner_care_recipient_id, &synthetic_nik())
        .await;
    assert_eq!(f.approve(&tv, other).await.status, 200);

    // Verifikator tanpa NIK staf: gagal-tertutup untuk semua pengajuan.
    let (_n, sub) = new_admin(&f.env.pool, &[AdminSubRole::Verifier]).await;
    let tn = f.admin_token(&sub);
    let pending = f
        .submit(&q, q.owner_care_recipient_id, &synthetic_nik())
        .await;
    let r = f.get(&format!("{BASE}/{pending}"), &tn).await;
    assert_eq!(r.json()["data"]["can_decide"], false);
    let r = f.approve(&tn, pending).await;
    assert_eq!(denied(&r), (409, "4090205".to_string()), "{}", r.body);
    assert_eq!(request_status(&f, pending).await, "pending");
}
