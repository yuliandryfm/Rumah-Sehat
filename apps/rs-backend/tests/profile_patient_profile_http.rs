//! Endpoint profil dan alamat pasien `/patient/v1` (F1-T56, US-105 AC1-AC3) lewat HTTP sungguhan: router
//! lengkap proses `api` (penegak kebijakan, validator token realm `admin` dan `pasien` dengan kunci realm
//! uji, resolver akun dan penjaga sesi di atas PostgreSQL), PostgreSQL efemeral (role runtime `rs_api`),
//! SeaweedFS dan Valkey efemeral (untuk pengajuan identitas yang menjadi prasyarat AC3).
//!
//! Skenario: alur baca/simpan profil (AC1), validasi per field (AC1a, AC1b, `4220001`), simpan tanpa
//! perubahan tanpa event, simpan paralel, alamat jamak dengan tepat satu alamat utama (AC2) termasuk dua
//! permintaan bersamaan, soft delete (KU-04), perubahan nama/tanggal lahir mengembalikan identitas
//! "Disetujui" ke "Menunggu verifikasi" (AC3), audit E-10/E-16 tanpa data pribadi, dan lima skenario
//! penolakan per endpoint (coding-standards §7.3). Tanpa variabel database atau Docker, test GAGAL.
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

use authz::DenialSink;
use common::{ADMIN_AUD, ADMIN_ISS, PASIEN_AUD, PASIEN_ISS, Resp, TestKeys, request, request_json};
use foundation::{SystemClock, Uuid};
use identity_support::{
    Env, Patient, approve_in_db, audit_events, audit_events_for_object, new_patient, synthetic_nik,
};
use rs_backend::access_audit::AuditDenialRecorder;
use rs_backend::app::{AppServices, AppState, build_router};
use rs_backend::modules::identity::api::{AccountDirectory, RevocationReason};
use serde_json::{Value, json};
use sqlx::PgPool;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

const PROFILE: &str = "/patient/v1/profile";
const ADDRESSES: &str = "/patient/v1/addresses";

static SERIAL: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

struct Fixture {
    env: Env,
    addr: SocketAddr,
    keys: TestKeys,
    directory: Arc<AccountDirectory>,
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
                consent_text_limit: common::roomy_limiter(),
                corporate_hr: common::corporate_hr(common::unreachable_pool()),
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
        }
    }

    fn token(&self, p: &Patient) -> String {
        common::token(&self.keys.pasien, PASIEN_ISS, PASIEN_AUD, &p.subject)
    }

    fn pool(&self) -> &PgPool {
        &self.env.pool
    }

    async fn send(&self, method: &str, path: &str, token: &str, body: Option<&Value>) -> Resp {
        let auth = format!("Authorization: Bearer {token}");
        match body {
            None => request(self.addr, method, path, &[&auth]).await,
            Some(b) => request_json(self.addr, method, path, &[&auth], b).await,
        }
    }

    async fn call(&self, e: &Endpoint, token: Option<&str>) -> Resp {
        let headers: Vec<String> = token
            .map(|t| vec![format!("Authorization: Bearer {t}")])
            .unwrap_or_default();
        let refs: Vec<&str> = headers.iter().map(String::as_str).collect();
        match &e.body {
            None => request(self.addr, e.method, &e.path, &refs).await,
            Some(b) => request_json(self.addr, e.method, &e.path, &refs, b).await,
        }
    }

    /// Akun pasien dengan alamat pertama sudah ada; mengembalikan ID alamat pertama.
    async fn add_address(&self, t: &str, body: &Value) -> Resp {
        self.send("POST", ADDRESSES, t, Some(body)).await
    }
}

struct Endpoint {
    method: &'static str,
    path: String,
    body: Option<Value>,
}

fn code(r: &Resp) -> String {
    r.json()["response_code"]
        .as_str()
        .unwrap_or_default()
        .to_string()
}

fn profile_body(name: &str, birth: &str) -> Value {
    json!({ "full_name": name, "birth_date": birth, "sex": "female", "phone": "0812-3456-7890" })
}

fn address_body(name: &str) -> Value {
    json!({
        "label": "Rumah", "recipient_name": name, "recipient_phone": "081234567890",
        "address_line": "Jl. Contoh No. 1", "province_name": "DKI Jakarta",
        "city_name": "Jakarta Selatan", "postal_code": "12345", "notes": "Dekat warung"
    })
}

fn endpoints(address: Uuid) -> Vec<Endpoint> {
    vec![
        Endpoint {
            method: "GET",
            path: PROFILE.to_string(),
            body: None,
        },
        Endpoint {
            method: "PUT",
            path: PROFILE.to_string(),
            body: Some(profile_body("Budi Santoso", "1990-01-31")),
        },
        Endpoint {
            method: "GET",
            path: ADDRESSES.to_string(),
            body: None,
        },
        Endpoint {
            method: "POST",
            path: ADDRESSES.to_string(),
            body: Some(address_body("Siti Aminah")),
        },
        Endpoint {
            method: "PUT",
            path: format!("{ADDRESSES}/{address}"),
            body: Some(address_body("Siti Aminah")),
        },
        Endpoint {
            method: "DELETE",
            path: format!("{ADDRESSES}/{address}"),
            body: None,
        },
    ]
}

fn items(r: &Resp) -> Vec<Value> {
    r.json()["data"]["items"].as_array().unwrap().clone()
}

fn default_count(list: &[Value]) -> usize {
    list.iter().filter(|a| a["is_default"] == true).count()
}

async fn db_default_count(pool: &PgPool, account_id: Uuid) -> i64 {
    sqlx::query_scalar!(
        r#"SELECT count(*) AS "n!" FROM app.address a JOIN app.patient p ON p.id = a.patient_id
           WHERE p.account_id = $1 AND a.is_default AND a.deleted_at IS NULL"#,
        account_id
    )
    .fetch_one(pool)
    .await
    .unwrap()
}

/// Pasien dengan identitas "Disetujui" (pengajuan lewat HTTP, persetujuan lewat jalur tulis di database).
async fn approved_patient(f: &Fixture) -> (Patient, String) {
    let p = new_patient(f.pool()).await;
    let t = f.token(&p);
    let nik = synthetic_nik();
    let slot = f.env.uploaded_slot(p.account_id).await;
    let r = f
        .send(
            "POST",
            &format!(
                "/patient/v1/care-recipients/{}/identity-submissions",
                p.owner_care_recipient_id
            ),
            &t,
            Some(&json!({ "nik": nik, "upload_slot_id": slot.hyphenated().to_string() })),
        )
        .await;
    assert_eq!(r.status, 201, "{}", r.body);
    let request_id: Uuid = r.json()["data"]["identity_verification_request_id"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();
    approve_in_db(&f.env, p.owner_care_recipient_id, request_id, &nik, false).await;
    let r = f.send("GET", PROFILE, &t, None).await;
    assert_eq!(
        r.json()["data"]["identity_status"],
        "approved",
        "{}",
        r.body
    );
    (p, t)
}

// trace: US-105 AC1, AC1a (TC-US-105-1: profil tampil kembali; nomor HP dinormalkan ke +62; simpan tanpa
// perubahan tidak mencatat event, TC-US-105-4)
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn profile_is_read_saved_and_unchanged_save_writes_no_event() {
    let _g = SERIAL.lock().await;
    let f = Fixture::new().await;
    let p = new_patient(f.pool()).await;
    let t = f.token(&p);
    let cr = p.owner_care_recipient_id;

    let r = f.send("GET", PROFILE, &t, None).await;
    assert_eq!(r.status, 200, "{}", r.body);
    let d = &r.json()["data"];
    assert_eq!(d["care_recipient_id"], cr.hyphenated().to_string());
    assert_eq!(d["identity_status"], "not_uploaded");
    assert_eq!(d["phone"], Value::Null);

    let r = f
        .send(
            "PUT",
            PROFILE,
            &t,
            Some(&profile_body("  Budi Santoso  ", "1990-01-31")),
        )
        .await;
    assert_eq!(r.status, 200, "{}", r.body);
    let d = &r.json()["data"];
    assert_eq!(d["full_name"], "Budi Santoso");
    assert_eq!(d["birth_date"], "1990-01-31");
    assert_eq!(d["sex"], "female");
    assert_eq!(d["phone"], "+6281234567890");
    assert_eq!(d["identity_status"], "not_uploaded");

    // E-10 hanya memuat nama field yang berubah, tanpa nilai data pribadi.
    let events = audit_events("e_10", cr).await;
    let saved: Vec<_> = events
        .iter()
        .filter(|e| e.event.body.action_name == "profile_updated")
        .collect();
    assert_eq!(saved.len(), 1, "{events:?}");
    let body = &saved[0].event.body;
    let mut changed = body.changed_fields.clone().unwrap();
    changed.sort();
    assert_eq!(changed, ["birth_date", "full_name", "phone"]); // jenis kelamin sama dengan awal
    assert_eq!(body.outcome, "success");
    assert_eq!(body.actor_account_id, Some(p.account_id));
    assert!(body.source_ip.is_some());
    assert!(body.before_values.is_none() && body.after_values.is_none());

    // Simpan identik: respons sama, tanpa event baru (TC-US-105-4).
    let again = f
        .send(
            "PUT",
            PROFILE,
            &t,
            Some(&profile_body("Budi Santoso", "1990-01-31")),
        )
        .await;
    assert_eq!(again.status, 200, "{}", again.body);
    assert_eq!(again.json()["data"], r.json()["data"]);
    let n = audit_events("e_10", cr)
        .await
        .iter()
        .filter(|e| e.event.body.action_name == "profile_updated")
        .count();
    assert_eq!(n, 1);

    // Nomor HP kosong mengosongkan (US-105 AC1a: opsional).
    let r = f
        .send(
            "PUT",
            PROFILE,
            &t,
            Some(&json!({ "full_name": "Budi Santoso", "birth_date": "1990-01-31", "sex": "female" })),
        )
        .await;
    assert_eq!(r.json()["data"]["phone"], Value::Null, "{}", r.body);
}

// trace: US-105 AC1a, AC1b, KU-01 (TC-US-105-2, -15, -16, -17, -20: pelanggaran per field dengan 4220001,
// semua sekaligus; tidak ada efek samping; nilai tidak dipantulkan)
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn profile_validation_rejects_each_field_without_side_effects() {
    let _g = SERIAL.lock().await;
    let f = Fixture::new().await;
    let p = new_patient(f.pool()).await;
    let t = f.token(&p);
    let r = f
        .send(
            "PUT",
            PROFILE,
            &t,
            Some(&json!({ "full_name": "Budi1@", "birth_date": "2099-01-01", "sex": "male", "phone": "12345" })),
        )
        .await;
    assert_eq!(r.status, 422, "{}", r.body);
    assert_eq!(code(&r), "4220001");
    let fields: Vec<String> = r.json()["errors"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["field"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(fields, ["full_name", "birth_date", "phone"]);
    assert!(!r.body.contains("Budi1@"), "nilai tidak dipantulkan");

    // Batas usia minimum (nilai uji 17 tahun) dan batas 120 tahun, relatif terhadap hari ini WIB.
    let wib = time::UtcOffset::from_hms(7, 0, 0).unwrap();
    let today = foundation::Clock::now(&SystemClock).to_offset(wib).date();
    let fmt = |d: time::Date| d.to_string();
    // Tepat 17 tahun hari ini diterima; sehari kurang ditolak (lewati bila hari ini 29 Februari).
    if let Ok(exactly_17) =
        time::Date::from_calendar_date(today.year() - 17, today.month(), today.day())
    {
        let r = f
            .send(
                "PUT",
                PROFILE,
                &t,
                Some(&profile_body("Anak Muda", &fmt(exactly_17))),
            )
            .await;
        assert_eq!(r.status, 200, "tepat 17 tahun diterima: {}", r.body);
        let too_young = exactly_17 + time::Duration::days(1);
        let r = f
            .send(
                "PUT",
                PROFILE,
                &t,
                Some(&profile_body("Anak Muda", &fmt(too_young))),
            )
            .await;
        assert_eq!(r.status, 422, "{}", r.body);
        assert_eq!(r.json()["errors"][0]["code"], "below_minimum_age");
    }
    let tomorrow = today + time::Duration::days(1);
    let r = f
        .send(
            "PUT",
            PROFILE,
            &t,
            Some(&profile_body("Budi", &fmt(tomorrow))),
        )
        .await;
    assert_eq!(r.status, 422, "{}", r.body);
    // Jenis kelamin di luar nilai, field tidak dikenal, dan badan bukan objek.
    for body in [
        json!({ "full_name": "Budi", "birth_date": "1990-01-01", "sex": "other" }),
        json!({ "full_name": "Budi", "birth_date": "1990-01-01", "sex": "male", "email": "x@y.z" }),
        json!({ "full_name": "Budi", "birth_date": "1990-01-01" }),
        json!({ "full_name": 5, "birth_date": "1990-01-01", "sex": "male" }),
    ] {
        let r = f.send("PUT", PROFILE, &t, Some(&body)).await;
        assert_eq!(
            (r.status, code(&r).as_str()),
            (422, "4220001"),
            "{body}: {}",
            r.body
        );
    }
    let r = f
        .send("PUT", PROFILE, &t, Some(&json!("bukan objek")))
        .await;
    assert_eq!(r.status, 422, "{}", r.body);
    // Penolakan tidak mengubah profil: nama tetap hasil simpan sah terakhir.
    let r = f.send("GET", PROFILE, &t, None).await;
    assert_ne!(r.json()["data"]["full_name"], "Budi1@");
}

// trace: US-105 AC1 (TC-US-105-4: dua simpan identik paralel → satu pembaruan, satu event)
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn parallel_identical_saves_write_one_update_and_one_event() {
    let _g = SERIAL.lock().await;
    let f = Arc::new(Fixture::new().await);
    let p = new_patient(f.pool()).await;
    let t = f.token(&p);
    let mut tasks = Vec::new();
    for _ in 0..4 {
        let f = Arc::clone(&f);
        let t = t.clone();
        tasks.push(tokio::spawn(async move {
            f.send(
                "PUT",
                PROFILE,
                &t,
                Some(&profile_body("Nama Paralel", "1985-05-05")),
            )
            .await
        }));
    }
    for task in tasks {
        let r = task.await.unwrap();
        assert_eq!(r.status, 200, "{}", r.body);
        assert_eq!(r.json()["data"]["full_name"], "Nama Paralel");
    }
    let n = audit_events("e_10", p.owner_care_recipient_id)
        .await
        .iter()
        .filter(|e| e.event.body.action_name == "profile_updated")
        .count();
    assert_eq!(n, 1, "satu event untuk satu perubahan");
}

// trace: US-105 AC3 (TC-US-105-7, -8, -9, -10: nama atau tanggal lahir pada identitas Disetujui →
// Menunggu verifikasi dengan permintaan profile_changed; nomor HP atau jenis kelamin saja tidak; tidak
// dapat dibatalkan; E-10 tercatat)
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn name_or_birth_date_change_returns_approved_identity_to_pending() {
    let _g = SERIAL.lock().await;
    let f = Fixture::new().await;
    let (p, t) = approved_patient(&f).await;
    let cr = p.owner_care_recipient_id;

    // Nomor HP dan jenis kelamin saja: status tetap Disetujui, tidak ada permintaan baru.
    let r = f
        .send(
            "PUT",
            PROFILE,
            &t,
            Some(&json!({ "full_name": "Pasien Sintetis", "birth_date": "1990-01-01", "sex": "male", "phone": "081200000001" })),
        )
        .await;
    assert_eq!(r.status, 200, "{}", r.body);
    assert_eq!(r.json()["data"]["identity_status"], "approved");
    assert_eq!(identity_support::request_count(f.pool(), cr).await, 1);

    // Ubah nama: kembali ke Menunggu verifikasi.
    let r = f
        .send(
            "PUT",
            PROFILE,
            &t,
            Some(&json!({ "full_name": "Pasien Berganti Nama", "birth_date": "1990-01-01", "sex": "male", "phone": "081200000001" })),
        )
        .await;
    assert_eq!(r.status, 200, "{}", r.body);
    assert_eq!(r.json()["data"]["identity_status"], "pending");
    assert_eq!(identity_support::request_count(f.pool(), cr).await, 2);
    let trigger: String = sqlx::query_scalar!(
        "SELECT trigger_type FROM app.identity_verification_request WHERE care_recipient_id = $1 \
         ORDER BY created_at DESC, id DESC LIMIT 1",
        cr
    )
    .fetch_one(f.pool())
    .await
    .unwrap();
    assert_eq!(trigger, "profile_changed");

    // Status identitas: Menunggu verifikasi, tidak dapat dibatalkan, NIK lama (4 digit) tetap.
    let r = f
        .send(
            "GET",
            &format!("/patient/v1/care-recipients/{cr}/identity"),
            &t,
            None,
        )
        .await;
    let d = &r.json()["data"];
    assert_eq!(
        (d["status"].as_str(), d["can_cancel"].as_bool()),
        (Some("pending"), Some(false))
    );
    assert!(d["nik_last4"].is_string());
    let r = f
        .send(
            "POST",
            &format!("/patient/v1/care-recipients/{cr}/identity-submissions/cancel"),
            &t,
            Some(&json!({})),
        )
        .await;
    assert_eq!(
        (r.status, code(&r).as_str()),
        (409, "4090001"),
        "{}",
        r.body
    );

    // Selama menunggu, perubahan lain tidak membuat permintaan baru (Verifikator melihat data terbaru).
    let r = f
        .send(
            "PUT",
            PROFILE,
            &t,
            Some(&json!({ "full_name": "Pasien Berganti Nama Lagi", "birth_date": "1990-01-01", "sex": "male", "phone": "081200000001" })),
        )
        .await;
    assert_eq!(r.json()["data"]["identity_status"], "pending");
    assert_eq!(identity_support::request_count(f.pool(), cr).await, 2);

    // E-10: pembaruan profil dan permintaan verifikasi ulang, tanpa nilai nama.
    let events = audit_events("e_10", cr).await;
    let actions: Vec<&str> = events
        .iter()
        .map(|e| e.event.body.action_name.as_str())
        .collect();
    assert!(
        actions.contains(&"identity_reverification_requested"),
        "{actions:?}"
    );
    assert!(
        actions.iter().filter(|a| **a == "profile_updated").count() >= 3,
        "{actions:?}"
    );
    let rr = events
        .iter()
        .find(|e| e.event.body.action_name == "identity_reverification_requested")
        .unwrap();
    assert_eq!(rr.event.body.object_type, "identity_verification_request");
    assert_eq!(rr.event.body.outcome, "success");
    assert!(
        !format!("{:?}", rr.event.body.after_values).contains("Berganti"),
        "nama tidak ada di audit"
    );
}

// trace: US-105 AC3 (TC-US-105-8: tanggal lahir saja juga menurunkan status)
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn birth_date_change_alone_returns_approved_identity_to_pending() {
    let _g = SERIAL.lock().await;
    let f = Fixture::new().await;
    let (_p, t) = approved_patient(&f).await;
    let r = f
        .send(
            "PUT",
            PROFILE,
            &t,
            Some(&json!({ "full_name": "Pasien Sintetis", "birth_date": "1991-02-03", "sex": "female" })),
        )
        .await;
    assert_eq!(r.status, 200, "{}", r.body);
    assert_eq!(r.json()["data"]["identity_status"], "pending");
}

// trace: US-105 AC2 (TC-US-105-5, -19, -6; ADR-003; KU-04: tiga alamat, tepat satu utama setelah tiap
// operasi, alamat terhapus tidak tampil dan baris tetap ada dengan alasan hapus)
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn addresses_keep_exactly_one_default_and_soft_delete() {
    let _g = SERIAL.lock().await;
    let f = Fixture::new().await;
    let p = new_patient(f.pool()).await;
    let t = f.token(&p);

    let r = f.send("GET", ADDRESSES, &t, None).await;
    assert_eq!(items(&r).len(), 0, "{}", r.body);

    // Alamat pertama otomatis utama (walau is_default=false).
    let mut body = address_body("Alamat Satu");
    body["is_default"] = json!(false);
    let r = f.add_address(&t, &body).await;
    assert_eq!(r.status, 201, "{}", r.body);
    assert_eq!(code(&r), "2010000");
    let list = items(&r);
    assert_eq!((list.len(), default_count(&list)), (1, 1));
    let a1: Uuid = list[0]["id"].as_str().unwrap().parse().unwrap();
    assert_eq!(list[0]["time_zone"], "Asia/Jakarta");
    assert_eq!(list[0]["recipient_phone"], "+6281234567890");

    // Alamat kedua dan ketiga tanpa is_default: tetap satu utama (alamat satu).
    let r = f.add_address(&t, &address_body("Alamat Dua")).await;
    let a2: Uuid = items(&r)
        .iter()
        .find(|a| a["recipient_name"] == "Alamat Dua")
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();
    let mut third = address_body("Alamat Tiga");
    third["time_zone"] = json!("Asia/Makassar");
    let r = f.add_address(&t, &third).await;
    let list = items(&r);
    assert_eq!((list.len(), default_count(&list)), (3, 1));
    assert_eq!(list[0]["id"], a1.hyphenated().to_string(), "utama pertama");

    // Alamat ketiga ditandai utama saat dibuat: yang lama dilepas.
    let mut fourth = address_body("Alamat Empat");
    fourth["is_default"] = json!(true);
    let r = f.add_address(&t, &fourth).await;
    let list = items(&r);
    assert_eq!((list.len(), default_count(&list)), (4, 1));
    assert_eq!(list[0]["recipient_name"], "Alamat Empat");

    // Menandai alamat dua utama lewat PUT; menonaktifkan utama ditolak.
    let mut put = address_body("Alamat Dua");
    put["is_default"] = json!(true);
    let r = f
        .send("PUT", &format!("{ADDRESSES}/{a2}"), &t, Some(&put))
        .await;
    assert_eq!(r.status, 200, "{}", r.body);
    let list = items(&r);
    assert_eq!(default_count(&list), 1);
    assert_eq!(list[0]["id"], a2.hyphenated().to_string());
    put["is_default"] = json!(false);
    let r = f
        .send("PUT", &format!("{ADDRESSES}/{a2}"), &t, Some(&put))
        .await;
    assert_eq!(
        (r.status, code(&r).as_str()),
        (422, "4220001"),
        "{}",
        r.body
    );
    assert_eq!(r.json()["errors"][0]["field"], "is_default");
    assert_eq!(db_default_count(f.pool(), p.account_id).await, 1);

    // Hapus alamat utama: alamat aktif tertua (alamat satu) menjadi utama; baris hanya soft delete.
    let r = f
        .send("DELETE", &format!("{ADDRESSES}/{a2}"), &t, None)
        .await;
    assert_eq!(r.status, 200, "{}", r.body);
    let list = items(&r);
    assert_eq!((list.len(), default_count(&list)), (3, 1));
    assert_eq!(list[0]["id"], a1.hyphenated().to_string());
    assert!(list.iter().all(|a| a["id"] != a2.hyphenated().to_string()));
    let row = sqlx::query!(
        "SELECT deleted_at IS NOT NULL AS \"deleted!\", delete_reason, deleted_by, is_default \
         FROM app.address WHERE id = $1",
        a2
    )
    .fetch_one(f.pool())
    .await
    .unwrap();
    assert!(row.deleted && !row.is_default);
    assert_eq!(row.delete_reason.as_deref(), Some("patient_request"));
    assert_eq!(row.deleted_by, Some(p.account_id));
    // Alamat terhapus tidak dapat diubah atau dihapus lagi.
    for method in ["PUT", "DELETE"] {
        let body = (method == "PUT").then(|| address_body("Alamat Dua"));
        let r = f
            .send(method, &format!("{ADDRESSES}/{a2}"), &t, body.as_ref())
            .await;
        assert_eq!((r.status, code(&r).as_str()), (404, "4040001"), "{method}");
    }
    // Hapus semua: daftar kosong, tidak ada alamat utama.
    for a in list {
        let r = f
            .send(
                "DELETE",
                &format!("{ADDRESSES}/{}", a["id"].as_str().unwrap()),
                &t,
                None,
            )
            .await;
        assert_eq!(r.status, 200, "{}", r.body);
    }
    assert_eq!(db_default_count(f.pool(), p.account_id).await, 0);
    let r = f.send("GET", ADDRESSES, &t, None).await;
    assert!(items(&r).is_empty());

    // Audit: E-10 tambah/ubah dan E-16 hapus, tanpa nilai alamat.
    let created = audit_events("e_10", p.owner_care_recipient_id).await;
    assert!(
        created
            .iter()
            .any(|e| e.event.body.action_name == "address_created")
    );
    assert!(
        created
            .iter()
            .any(|e| e.event.body.action_name == "address_default_changed")
    );
    let deleted = audit_events("e_16", p.owner_care_recipient_id).await;
    assert!(
        deleted
            .iter()
            .any(|e| e.event.body.action_name == "address_deleted")
    );
    for e in created.iter().chain(deleted.iter()) {
        let b = &e.event.body;
        let all = format!("{:?}{:?}", b.before_values, b.after_values);
        assert!(
            !all.contains("Jl. Contoh") && !all.contains("0812"),
            "{all}"
        );
    }
}

// trace: US-105 AC1a (TC-US-105-18: batas alamat; semua galat per field sekaligus), AC2 (batas teknis)
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn address_validation_and_limit() {
    let _g = SERIAL.lock().await;
    let f = Fixture::new().await;
    let p = new_patient(f.pool()).await;
    let t = f.token(&p);
    let r = f
        .add_address(
            &t,
            &json!({
                "recipient_name": "", "recipient_phone": "12", "address_line": "abcd",
                "province_name": "J", "city_name": "", "postal_code": "12a", "time_zone": "Asia/Tokyo"
            }),
        )
        .await;
    assert_eq!(
        (r.status, code(&r).as_str()),
        (422, "4220001"),
        "{}",
        r.body
    );
    let fields: Vec<String> = r.json()["errors"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["field"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(
        fields,
        [
            "recipient_name",
            "recipient_phone",
            "address_line",
            "province_name",
            "city_name",
            "postal_code",
            "time_zone"
        ]
    );
    assert_eq!(
        items(&f.send("GET", ADDRESSES, &t, None).await).len(),
        0,
        "penolakan tidak menyimpan apa pun"
    );
    // Batas jumlah alamat aktif (20, batas teknis awal).
    for i in 0..20 {
        let r = f
            .add_address(
                &t,
                &address_body(&format!("Alamat Ke Satu{}", "a".repeat(i % 3))),
            )
            .await;
        assert_eq!(r.status, 201, "alamat ke-{i}: {}", r.body);
    }
    let r = f.add_address(&t, &address_body("Alamat Berlebih")).await;
    assert_eq!(
        (r.status, code(&r).as_str()),
        (422, "4220001"),
        "{}",
        r.body
    );
    assert_eq!(r.json()["errors"][0]["code"], "too_many_addresses");
    assert_eq!(db_default_count(f.pool(), p.account_id).await, 1);
}

// trace: US-105 AC2, TC-US-105-19 (dua permintaan bersamaan menandai alamat berbeda sebagai utama →
// tepat satu alamat utama)
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn parallel_default_changes_keep_exactly_one_default() {
    let _g = SERIAL.lock().await;
    let f = Arc::new(Fixture::new().await);
    let p = new_patient(f.pool()).await;
    let t = f.token(&p);
    let mut ids = Vec::new();
    for n in ["Satu", "Dua", "Tiga", "Empat"] {
        let r = f
            .add_address(&t, &address_body(&format!("Alamat {n}")))
            .await;
        let id = items(&r)
            .iter()
            .find(|a| a["recipient_name"] == format!("Alamat {n}"))
            .unwrap()["id"]
            .as_str()
            .unwrap()
            .to_string();
        ids.push((n, id));
    }
    let mut tasks = Vec::new();
    for (n, id) in ids.clone() {
        let f = Arc::clone(&f);
        let t = t.clone();
        tasks.push(tokio::spawn(async move {
            let mut body = address_body(&format!("Alamat {n}"));
            body["is_default"] = json!(true);
            f.send("PUT", &format!("{ADDRESSES}/{id}"), &t, Some(&body))
                .await
        }));
    }
    for task in tasks {
        let r = task.await.unwrap();
        assert_eq!(r.status, 200, "{}", r.body);
        assert_eq!(default_count(&items(&r)), 1);
    }
    assert_eq!(db_default_count(f.pool(), p.account_id).await, 1);
}

// trace: coding-standards §7.3 butir 5, KU-03 (TC-US-105-3: alamat milik pasien lain dan ID yang tidak ada
// dijawab 404 identik; tidak ada perubahan; percobaan tercatat E-10 hasil gagal), ADR-012 butir 4
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn foreign_address_is_not_found_identically_and_audited() {
    let _g = SERIAL.lock().await;
    let f = Fixture::new().await;
    let a = new_patient(f.pool()).await;
    let b = new_patient(f.pool()).await;
    let ta = f.token(&a);
    let tb = f.token(&b);
    let r = f.add_address(&ta, &address_body("Milik A")).await;
    let addr_a: Uuid = items(&r)[0]["id"].as_str().unwrap().parse().unwrap();
    let unknown = foundation::new_id();

    let mut seen = Vec::new();
    for target in [addr_a, unknown] {
        for method in ["PUT", "DELETE"] {
            let body = (method == "PUT").then(|| address_body("Disusupi"));
            let r = f
                .send(method, &format!("{ADDRESSES}/{target}"), &tb, body.as_ref())
                .await;
            assert_eq!((r.status, code(&r).as_str()), (404, "4040001"), "{method}");
            seen.push((method, r.status, r.json()["response_description"].clone()));
        }
    }
    assert_eq!(seen[0], seen[2], "PUT: ID asing = ID tidak ada");
    assert_eq!(seen[1], seen[3], "DELETE: ID asing = ID tidak ada");
    let r = f
        .send("DELETE", &format!("{ADDRESSES}/bukan-uuid"), &tb, None)
        .await;
    assert_eq!((r.status, code(&r).as_str()), (404, "4040001"));

    // Alamat A utuh; daftar B kosong.
    let r = f.send("GET", ADDRESSES, &ta, None).await;
    assert_eq!(items(&r)[0]["recipient_name"], "Milik A");
    assert_eq!(items(&f.send("GET", ADDRESSES, &tb, None).await).len(), 0);
    // Percobaan tercatat sebagai E-10 hasil gagal (ubah) dan E-16 hasil gagal (hapus).
    let failed_update = audit_events_for_object("e_10", addr_a).await;
    assert!(
        failed_update.iter().any(|e| {
            e.event.body.action_name == "address_updated"
                && e.event.body.outcome != "success"
                && e.event.body.actor_account_id == Some(b.account_id)
        }),
        "{failed_update:?}"
    );
    let failed_delete = audit_events_for_object("e_16", addr_a).await;
    assert!(
        failed_delete
            .iter()
            .any(|e| e.event.body.action_name == "address_deleted"
                && e.event.body.outcome != "success"),
        "{failed_delete:?}"
    );
}

// trace: coding-standards §7.3 butir 1, 2 (tanpa token; realm lain, audience salah, issuer salah,
// algoritma di luar daftar izin, kedaluwarsa → 401), ADR-011 butir 8
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn every_endpoint_rejects_missing_and_invalid_tokens() {
    let _g = SERIAL.lock().await;
    let f = Fixture::new().await;
    let p = new_patient(f.pool()).await;
    let t = f.token(&p);
    let r = f.add_address(&t, &address_body("Milik P")).await;
    let address: Uuid = items(&r)[0]["id"].as_str().unwrap().parse().unwrap();
    let now = foundation::Clock::now(&SystemClock).unix_timestamp();
    let sign = |iss: &str, aud: &str, iat: i64, exp: i64| {
        f.keys
            .pasien
            .sign(&json!({
                "iss": iss, "aud": aud, "sub": p.subject, "typ": "Bearer",
                "sid": "sesi-uji", "azp": "rs-pasien-mobile", "iat": iat, "exp": exp,
            }))
            .unwrap()
    };
    let payload = t.split('.').nth(1).unwrap().to_string();
    let invalid = [
        (
            "admin_realm",
            common::token(&f.keys.admin, ADMIN_ISS, ADMIN_AUD, &p.subject),
        ),
        (
            "wrong_audience",
            sign(PASIEN_ISS, ADMIN_AUD, now - 5, now + 300),
        ),
        (
            "wrong_issuer",
            sign(
                "https://auth.ruang-sehat.invalid/realms/lain",
                PASIEN_AUD,
                now - 5,
                now + 300,
            ),
        ),
        (
            "expired",
            sign(PASIEN_ISS, PASIEN_AUD, now - 900, now - 600),
        ),
        (
            "alg_none",
            format!("eyJhbGciOiJub25lIiwia2lkIjoia2lkLXBhc2llbiIsInR5cCI6IkpXVCJ9.{payload}."),
        ),
    ];
    for e in endpoints(address) {
        let r = f.call(&e, None).await;
        assert_eq!(r.status, 401, "tanpa token {} {}", e.method, e.path);
        assert_eq!(code(&r), "4010001");
        for (why, tok) in &invalid {
            let r = f.call(&e, Some(tok)).await;
            assert_eq!(r.status, 401, "{why} {} {}: {}", e.method, e.path, r.body);
            assert_eq!(code(&r), "4010001", "{why}");
        }
    }
    // Tidak ada efek: alamat dan profil utuh.
    assert_eq!(items(&f.send("GET", ADDRESSES, &t, None).await).len(), 1);
    assert_eq!(
        f.send("GET", PROFILE, &t, None).await.json()["data"]["full_name"],
        "Pasien Sintetis"
    );
}

// trace: coding-standards §7.3 butir 3, 4 (akun nonaktif → 4010002, sesi dicabut → 4010001, token admin
// pada endpoint pasien → 401), KU-10, US-103 AC5
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn inactive_account_revoked_session_and_admin_are_rejected() {
    let _g = SERIAL.lock().await;
    let f = Fixture::new().await;
    let inactive = new_patient(f.pool()).await;
    sqlx::query!(
        "UPDATE app.account SET status = 'inactive', status_changed_at = now() WHERE id = $1",
        inactive.account_id
    )
    .execute(f.pool())
    .await
    .unwrap();
    let revoked = new_patient(f.pool()).await;
    let sid = format!("sesi-dicabut-{}", files::random_hex(6));
    let now = foundation::Clock::now(&SystemClock).unix_timestamp();
    let revoked_token = f
        .keys
        .pasien
        .sign(&json!({
            "iss": PASIEN_ISS, "aud": PASIEN_AUD, "sub": revoked.subject, "typ": "Bearer",
            "sid": sid, "azp": "rs-pasien-mobile", "iat": now - 5, "exp": now + 300,
        }))
        .unwrap();
    let mut tx = f.pool().begin().await.unwrap();
    f.directory
        .record_revocation(
            &mut tx,
            revoked.account_id,
            &sid,
            RevocationReason::Logout,
            foundation::Clock::now(&SystemClock) + time::Duration::hours(1),
        )
        .await
        .unwrap();
    tx.commit().await.unwrap();
    let (_admin_id, admin_sub) =
        identity_support::new_admin(f.pool(), &authz::AdminSubRole::ALL).await;
    let admin_token = common::token(&f.keys.admin, ADMIN_ISS, ADMIN_AUD, &admin_sub);

    let probe = foundation::new_id();
    for e in endpoints(probe) {
        let r = f.call(&e, Some(&f.token(&inactive))).await;
        assert_eq!(
            (r.status, code(&r).as_str()),
            (401, "4010002"),
            "{} {}",
            e.method,
            e.path
        );
        let r = f.call(&e, Some(&revoked_token)).await;
        assert_eq!(
            (r.status, code(&r).as_str()),
            (401, "4010001"),
            "{} {}",
            e.method,
            e.path
        );
        let r = f.call(&e, Some(&admin_token)).await;
        assert_eq!(
            (r.status, code(&r).as_str()),
            (401, "4010001"),
            "admin {} {}",
            e.method,
            e.path
        );
    }
    // Sesi lain akun yang sama tetap berlaku (bukti penolakan karena sesi, bukan akun).
    let r = f.send("GET", PROFILE, &f.token(&revoked), None).await;
    assert_eq!(r.status, 200, "{}", r.body);
}
