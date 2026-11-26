//! Endpoint identitas pasien `/patient/v1` (F1-T61, reference implementation US-106) lewat HTTP sungguhan:
//! router lengkap proses `api` (`build_router`: penegak kebijakan, validator token realm `admin` dan
//! `pasien` dengan kunci realm uji, resolver akun dan penjaga sesi modul identity di atas PostgreSQL,
//! `ClientIp` dari peer TCP, E-05 hasil gagal lewat `AuditDenialRecorder`), PostgreSQL efemeral
//! (`scripts/db-ephemeral.sh`, role runtime `rs_api`), SeaweedFS dan Valkey efemeral (Docker).
//!
//! Skenario: alur pengajuan, kode modul `02` (`4090201`, `4090202`), validasi NIK (`4220001`), slot unggah
//! (`4130001`, `4150001`), dan lima skenario penolakan per endpoint (coding-standards §7.3): tanpa token;
//! token realm lain, audience salah, issuer salah, algoritma di luar daftar izin, kedaluwarsa; akun
//! nonaktif dan sesi dicabut; peran lain (token admin) ditolak; penerima layanan milik akun lain → 404
//! dengan E-05 hasil gagal. Tanpa variabel database atau Docker, test GAGAL (tidak dilewati).
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
use identity_support::{Env, Patient, new_family_member, new_patient, synthetic_nik};
use observability::EnvSource;
use rs_backend::access_audit::AuditDenialRecorder;
use rs_backend::app::{AppServices, AppState, build_router};
use rs_backend::modules::identity::api::{AccountDirectory, RevocationReason};
use serde_json::{Value, json};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;

/// Header JWT `{"alg":"none","kid":"kid-pasien","typ":"JWT"}` (base64url): algoritma di luar daftar izin.
const ALG_NONE_HEADER: &str = "eyJhbGciOiJub25lIiwia2lkIjoia2lkLXBhc2llbiIsInR5cCI6IkpXVCJ9";

static SERIAL: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

struct Fixture {
    env: Env,
    addr: SocketAddr,
    keys: TestKeys,
    directory: Arc<AccountDirectory>,
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
            relay: relay_pool().await,
        }
    }

    fn token_with(&self, claims: Value) -> String {
        self.keys.pasien.sign(&claims).unwrap()
    }

    fn patient_token(&self, p: &Patient) -> String {
        common::token(&self.keys.pasien, PASIEN_ISS, PASIEN_AUD, &p.subject)
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

    async fn post_no_auth(&self, path: &str, body: &Value) -> Resp {
        request_json(self.addr, "POST", path, &[], body).await
    }

    /// Mengirim permintaan endpoint `e` dengan token `token` (`None` = tanpa header Authorization).
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
}

fn status_path(cr: Uuid) -> String {
    format!("/patient/v1/care-recipients/{cr}/identity")
}

fn submit_path(cr: Uuid) -> String {
    format!("/patient/v1/care-recipients/{cr}/identity-submissions")
}

fn cancel_path(cr: Uuid) -> String {
    format!("/patient/v1/care-recipients/{cr}/identity-submissions/cancel")
}

const SLOT_PATH: &str = "/patient/v1/ktp-photos/upload-slots";

fn submission(nik: &str, slot: Uuid) -> Value {
    json!({ "nik": nik, "upload_slot_id": slot.hyphenated().to_string() })
}

fn code(r: &Resp) -> String {
    r.json()["response_code"]
        .as_str()
        .unwrap_or_default()
        .to_string()
}

struct Endpoint {
    method: &'static str,
    path: String,
    body: Option<Value>,
}

/// Keempat endpoint untuk penerima layanan `cr` (slot unggah tidak terikat penerima layanan).
fn endpoints(cr: Uuid, slot: Uuid) -> Vec<Endpoint> {
    vec![
        Endpoint {
            method: "POST",
            path: SLOT_PATH.to_string(),
            body: Some(json!({ "content_type": "image/jpeg", "size_bytes": 1024 })),
        },
        Endpoint {
            method: "GET",
            path: status_path(cr),
            body: None,
        },
        Endpoint {
            method: "POST",
            path: submit_path(cr),
            body: Some(submission(&synthetic_nik(), slot)),
        },
        Endpoint {
            method: "POST",
            path: cancel_path(cr),
            body: Some(json!({})),
        },
    ]
}

/// E-05 hasil gagal di `audit_outbox` untuk subjek `subject` (dicatat asinkron oleh `DenialSink`).
async fn e05_failures(relay: &PgPool, subject: Uuid) -> i64 {
    for _ in 0..50 {
        let n = sqlx::query_scalar!(
            r#"SELECT count(*) AS "n!" FROM app.audit_outbox
               WHERE payload ->> 'event_code' = 'e_05'
                 AND payload -> 'subjects' -> 0 ->> 'subject_id' = $1"#,
            subject.hyphenated().to_string(),
        )
        .fetch_one(relay)
        .await
        .unwrap();
        if n > 0 {
            return n;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    0
}

// trace: US-106 AC1, AC5 (alur HTTP: ajukan → Menunggu verifikasi → pengajuan kedua ditolak 4090201 →
// batal → status kembali), AB-05
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn submit_status_lock_and_cancel_over_http() {
    let _g = SERIAL.lock().await;
    let f = Fixture::new().await;
    let p = new_patient(&f.env.pool).await;
    let t = f.patient_token(&p);
    let cr = p.owner_care_recipient_id;

    let r = f.get(&status_path(cr), &t).await;
    assert_eq!(r.status, 200, "{}", r.body);
    assert_eq!(r.json()["data"]["status"], "not_uploaded");
    assert_eq!(r.json()["data"]["can_submit"], true);

    let nik = synthetic_nik();
    let slot = f.env.uploaded_slot(p.account_id).await;
    let r = f.post(&submit_path(cr), &t, &submission(&nik, slot)).await;
    assert_eq!(r.status, 201, "{}", r.body);
    assert_eq!(code(&r), "2010000");
    assert_eq!(r.json()["data"]["status"], "pending");
    assert!(!r.body.contains(&nik), "NIK tidak pernah ada di respons");

    let r = f.get(&status_path(cr), &t).await;
    let d = &r.json()["data"];
    assert_eq!(d["status"], "pending");
    assert_eq!(d["can_cancel"], true);
    assert_eq!(d["can_submit"], false);
    assert_eq!(d["nik_last4"], nik[12..]);
    assert!(!r.body.contains(&nik));

    // US-106 AC5: foto terkunci selama menunggu.
    let slot2 = f.env.uploaded_slot(p.account_id).await;
    let r = f
        .post(&submit_path(cr), &t, &submission(&synthetic_nik(), slot2))
        .await;
    assert_eq!(r.status, 409, "{}", r.body);
    assert_eq!(code(&r), "4090201");
    assert_eq!(
        r.json()["response_description"],
        "Foto KTP tidak dapat diganti sampai pengajuan diputuskan atau dibatalkan"
    );
    assert_eq!(r.header("content-type"), Some("application/problem+json"));

    let r = f.post(&cancel_path(cr), &t, &json!({})).await;
    assert_eq!(r.status, 200, "{}", r.body);
    assert_eq!(r.json()["data"]["status"], "not_uploaded");
    assert_eq!(r.json()["data"]["can_submit"], true);

    // Tidak ada lagi yang dapat dibatalkan: konflik status umum.
    let r = f.post(&cancel_path(cr), &t, &json!({})).await;
    assert_eq!(r.status, 409, "{}", r.body);
    assert_eq!(code(&r), "4090001");
}

// trace: US-106 AC2 (NIK bukan 16 digit → 4220001 dengan pesan "NIK harus 16 digit angka"), KU-01
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn invalid_nik_and_body_are_rejected() {
    let _g = SERIAL.lock().await;
    let f = Fixture::new().await;
    let p = new_patient(&f.env.pool).await;
    let t = f.patient_token(&p);
    let cr = p.owner_care_recipient_id;
    for bad in [
        "123456789012345",
        "12345678901234567",
        "12345678901234ab",
        "",
    ] {
        let slot = f.env.uploaded_slot(p.account_id).await;
        let r = f.post(&submit_path(cr), &t, &submission(bad, slot)).await;
        assert_eq!(r.status, 422, "{bad}: {}", r.body);
        assert_eq!(code(&r), "4220001");
        assert_eq!(r.json()["errors"][0]["field"], "nik");
        assert_eq!(r.json()["errors"][0]["message"], "NIK harus 16 digit angka");
        assert!(
            bad.is_empty() || !r.body.contains(bad),
            "nilai NIK tidak dipantulkan"
        );
    }
    // Field tidak dikenal, slot bukan UUID, badan bukan JSON.
    let r = f
        .post(
            &submit_path(cr),
            &t,
            &json!({ "nik": synthetic_nik(), "upload_slot_id": Uuid::nil().hyphenated().to_string(), "extra": 1 }),
        )
        .await;
    assert_eq!(
        (r.status, code(&r).as_str()),
        (422, "4220001"),
        "{}",
        r.body
    );
    let r = f
        .post(
            &submit_path(cr),
            &t,
            &json!({ "nik": synthetic_nik(), "upload_slot_id": "bukan-uuid" }),
        )
        .await;
    assert_eq!(r.status, 422, "{}", r.body);
    assert_eq!(r.json()["errors"][0]["field"], "upload_slot_id");
}

// trace: US-106 AC4 (NIK pemilik akun lain → 4090202 dengan pesan umum; pengajuan tidak disimpan)
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn owner_nik_in_use_returns_general_message() {
    let _g = SERIAL.lock().await;
    let f = Fixture::new().await;
    let a = new_patient(&f.env.pool).await;
    let b = new_patient(&f.env.pool).await;
    let nik = synthetic_nik();
    let slot = f.env.uploaded_slot(a.account_id).await;
    let r = f
        .post(
            &submit_path(a.owner_care_recipient_id),
            &f.patient_token(&a),
            &submission(&nik, slot),
        )
        .await;
    assert_eq!(r.status, 201, "{}", r.body);
    let request_id: Uuid = r.json()["data"]["identity_verification_request_id"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();
    identity_support::approve_in_db(&f.env, a.owner_care_recipient_id, request_id, &nik, false)
        .await;

    let slot_b = f.env.uploaded_slot(b.account_id).await;
    let r = f
        .post(
            &submit_path(b.owner_care_recipient_id),
            &f.patient_token(&b),
            &submission(&nik, slot_b),
        )
        .await;
    assert_eq!(r.status, 409, "{}", r.body);
    assert_eq!(code(&r), "4090202");
    assert_eq!(
        r.json()["response_description"],
        "NIK ini tidak dapat dipakai untuk akun Anda. Hubungi Layanan Pelanggan bila ini NIK Anda."
    );
    let r = f
        .get(
            &status_path(b.owner_care_recipient_id),
            &f.patient_token(&b),
        )
        .await;
    assert_eq!(r.json()["data"]["status"], "not_uploaded");

    // NIK yang sama untuk anggota keluarga di akun lain tetap diterima (UP-40).
    let family = new_family_member(&f.env.pool, b.account_id).await;
    let slot_f = f.env.uploaded_slot(b.account_id).await;
    let r = f
        .post(
            &submit_path(family),
            &f.patient_token(&b),
            &submission(&nik, slot_f),
        )
        .await;
    assert_eq!(r.status, 201, "{}", r.body);
}

// trace: US-106 AC3, review F1-T61 N3 (slot unggah: jenis bukan JPG/PNG → 4150001, > 5.242.880 byte → 4130001,
// tepat 5.242.880 byte diterima)
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ktp_upload_slot_checks_declared_type_and_size() {
    let _g = SERIAL.lock().await;
    let f = Fixture::new().await;
    let p = new_patient(&f.env.pool).await;
    let t = f.patient_token(&p);
    let r = f
        .post(
            SLOT_PATH,
            &t,
            &json!({ "content_type": "image/png", "size_bytes": 5_242_880 }),
        )
        .await;
    assert_eq!(r.status, 201, "{}", r.body);
    let d = &r.json()["data"];
    assert!(
        d["upload_slot_id"]
            .as_str()
            .unwrap()
            .parse::<Uuid>()
            .is_ok()
    );
    assert!(d["url"].as_str().unwrap().starts_with("http"));
    let r = f
        .post(
            SLOT_PATH,
            &t,
            &json!({ "content_type": "image/png", "size_bytes": 5_242_881 }),
        )
        .await;
    assert_eq!(
        (r.status, code(&r).as_str()),
        (413, "4130001"),
        "{}",
        r.body
    );
    let r = f
        .post(
            SLOT_PATH,
            &t,
            &json!({ "content_type": "application/pdf", "size_bytes": 1024 }),
        )
        .await;
    assert_eq!(
        (r.status, code(&r).as_str()),
        (415, "4150001"),
        "{}",
        r.body
    );
    let r = request_json(
        f.addr,
        "POST",
        SLOT_PATH,
        &[&format!("Authorization: Bearer {t}")],
        &json!("bukan objek"),
    )
    .await;
    assert_eq!(r.status, 422, "{}", r.body);

    // Review F1-T61 N3: kuota 10 slot per akun per jam [ASUMSI]. Tiga permintaan berisi di atas sudah
    // terhitung (termasuk yang ditolak 413/415); tujuh berikutnya diterima, yang kesebelas 429.
    let ok = json!({ "content_type": "image/jpeg", "size_bytes": 1024 });
    for i in 4..=10 {
        let r = f.post(SLOT_PATH, &t, &ok).await;
        assert_eq!(r.status, 201, "permintaan ke-{i}: {}", r.body);
    }
    let r = f.post(SLOT_PATH, &t, &ok).await;
    assert_eq!(
        (r.status, code(&r).as_str()),
        (429, "4290001"),
        "{}",
        r.body
    );
    // Akun lain tidak terpengaruh.
    let other = new_patient(&f.env.pool).await;
    let r = f.post(SLOT_PATH, &f.patient_token(&other), &ok).await;
    assert_eq!(r.status, 201, "{}", r.body);
}

// trace: coding-standards §7.3 butir 1, 2 (tanpa token; realm lain, audience salah, issuer salah,
// algoritma di luar daftar izin, kedaluwarsa, kunci tak dikenal → 401), ADR-011 butir 8
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn every_endpoint_rejects_missing_and_invalid_tokens() {
    let _g = SERIAL.lock().await;
    let f = Fixture::new().await;
    let p = new_patient(&f.env.pool).await;
    let slot = f.env.uploaded_slot(p.account_id).await;
    let now = foundation::Clock::now(&SystemClock).unix_timestamp();
    let claims = |iss: &str, aud: &str, iat: i64, exp: i64| {
        json!({
            "iss": iss, "aud": aud, "sub": p.subject, "typ": "Bearer",
            "sid": "sesi-uji", "azp": "rs-pasien-mobile", "iat": iat, "exp": exp,
        })
    };
    let valid = f.patient_token(&p);
    let payload = valid.split('.').nth(1).unwrap();
    let invalid = [
        (
            "admin_realm",
            common::token(&f.keys.admin, ADMIN_ISS, ADMIN_AUD, &p.subject),
        ),
        (
            "wrong_audience",
            f.token_with(claims(PASIEN_ISS, ADMIN_AUD, now - 5, now + 300)),
        ),
        (
            "wrong_issuer",
            f.token_with(claims(
                "https://auth.ruang-sehat.invalid/realms/lain",
                PASIEN_AUD,
                now - 5,
                now + 300,
            )),
        ),
        (
            "expired",
            f.token_with(claims(PASIEN_ISS, PASIEN_AUD, now - 900, now - 600)),
        ),
        ("alg_none", format!("{ALG_NONE_HEADER}.{payload}.")),
        (
            "unknown_key",
            common::token(&f.keys.admin, PASIEN_ISS, PASIEN_AUD, &p.subject),
        ),
    ];
    for e in endpoints(p.owner_care_recipient_id, slot) {
        let r = f.call(&e, None).await;
        assert_eq!(r.status, 401, "tanpa token {} {}", e.method, e.path);
        assert_eq!(code(&r), "4010001");
        for (why, tok) in &invalid {
            let r = f.call(&e, Some(tok)).await;
            assert_eq!(r.status, 401, "{why} {} {}: {}", e.method, e.path, r.body);
            assert_eq!(code(&r), "4010001", "{why}");
        }
    }
    // Tidak ada efek: pengajuan tidak tersimpan.
    assert_eq!(
        identity_support::request_count(&f.env.pool, p.owner_care_recipient_id).await,
        0
    );
}

// trace: coding-standards §7.3 butir 3, 4 (akun nonaktif → 4010002, sesi dicabut → 4010001, akun admin
// pada endpoint pasien ditolak; realm pasien hanya memuat akun pasien sehingga "peran lain" = token admin
// → 401, bukan 403), KU-10, US-103 AC5
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn inactive_account_revoked_session_and_admin_are_rejected() {
    let _g = SERIAL.lock().await;
    let f = Fixture::new().await;
    let inactive = new_patient(&f.env.pool).await;
    sqlx::query!(
        "UPDATE app.account SET status = 'inactive', status_changed_at = now() WHERE id = $1",
        inactive.account_id
    )
    .execute(&f.env.pool)
    .await
    .unwrap();
    let revoked = new_patient(&f.env.pool).await;
    // `sid` unik: pencabutan berlaku per sesi IdP untuk semua token yang membawanya, dan database efemeral
    // dipakai bersama berkas test lain (token `common::token` membawa `sesi-uji`).
    let sid = format!("sesi-dicabut-{}", files::random_hex(6));
    let now = foundation::Clock::now(&SystemClock).unix_timestamp();
    let revoked_token = f.token_with(json!({
        "iss": PASIEN_ISS, "aud": PASIEN_AUD, "sub": revoked.subject, "typ": "Bearer",
        "sid": sid, "azp": "rs-pasien-mobile", "iat": now - 5, "exp": now + 300,
    }));
    let mut tx = f.env.pool.begin().await.unwrap();
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
        identity_support::new_admin(&f.env.pool, &authz::AdminSubRole::ALL).await;
    let admin_token = common::token(&f.keys.admin, ADMIN_ISS, ADMIN_AUD, &admin_sub);

    let slot_i = f.env.uploaded_slot(inactive.account_id).await;
    for e in endpoints(inactive.owner_care_recipient_id, slot_i) {
        let r = f.call(&e, Some(&f.patient_token(&inactive))).await;
        assert_eq!(
            (r.status, code(&r).as_str()),
            (401, "4010002"),
            "{} {}",
            e.method,
            e.path
        );
    }
    let slot_r = f.env.uploaded_slot(revoked.account_id).await;
    for e in endpoints(revoked.owner_care_recipient_id, slot_r) {
        let r = f.call(&e, Some(&revoked_token)).await;
        assert_eq!(
            (r.status, code(&r).as_str()),
            (401, "4010001"),
            "{} {}",
            e.method,
            e.path
        );
        // Sesi lain akun yang sama tetap berlaku (bukti penolakan karena sesi, bukan akun).
        let r = f.call(&e, Some(&f.patient_token(&revoked))).await;
        assert_ne!(r.status, 401, "{} {}: {}", e.method, e.path, r.body);
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
}

// trace: coding-standards §7.3 butir 5, KU-03 (penerima layanan milik akun lain dan yang tidak ada dijawab
// 404 sama; percobaan atas dokumen identitas tercatat E-05 hasil gagal), ADR-012 butir 4
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn foreign_care_recipient_is_not_found_and_audited() {
    let _g = SERIAL.lock().await;
    let f = Fixture::new().await;
    let a = new_patient(&f.env.pool).await;
    let b = new_patient(&f.env.pool).await;
    let t = f.patient_token(&a);
    for target in [b.owner_care_recipient_id, foundation::new_id()] {
        let slot = f.env.uploaded_slot(a.account_id).await;
        for e in endpoints(target, slot).into_iter().skip(1) {
            let r = f.call(&e, Some(&t)).await;
            assert_eq!(
                (r.status, code(&r).as_str()),
                (404, "4040001"),
                "{} {}",
                e.method,
                e.path
            );
        }
    }
    // Path bukan UUID: 404 yang sama.
    let r = f
        .get("/patient/v1/care-recipients/bukan-uuid/identity", &t)
        .await;
    assert_eq!((r.status, code(&r).as_str()), (404, "4040001"));
    assert!(
        e05_failures(&f.relay, b.owner_care_recipient_id).await >= 1,
        "percobaan atas data sensitif milik orang lain tercatat E-05 hasil gagal"
    );
    assert_eq!(
        identity_support::request_count(&f.env.pool, b.owner_care_recipient_id).await,
        0
    );
    let r = f
        .post_no_auth(&cancel_path(b.owner_care_recipient_id), &json!({}))
        .await;
    assert_eq!(r.status, 401);
}
