//! F1-T101 (T1; US-1901 AC1, AC3-AC7; ADR-030 butir 2, 5, 7; KEPUTUSAN-321 s.d. KEPUTUSAN-327): pendaftaran
//! perusahaan mandiri `POST /v1/public/corporate-registrations` dan `GET .../consent-text` memakai proses `api`
//! lengkap (router produksi) terhadap PostgreSQL 18 sekali pakai (`scripts/db-ephemeral.sh`, role runtime
//! `rs_api`). Tanpa variabel `RS_TEST_DB_*` test GAGAL.
//!
//! Yang diuji: pendaftaran baru membuat perusahaan Menunggu verifikasi, akun HR `pending_provisioning` tanpa
//! password, keanggotaan, satu niat `create_user` + satu job berisi ID niat saja, bukti persetujuan per butir, dan
//! event E-10/E-11/E-13 lengkap tanpa data pribadi; jawaban seragam untuk NIB dan email terdaftar (nol efek, E-10
//! gagal) dengan waktu respons yang disamakan; isolasi perusahaan lain; persetujuan wajib dan versi basi; teks draf
//! ditolak di production; validasi per field (NIB 13 angka, NPWP 16 angka, password tidak diterima); batas laju;
//! balapan NIB.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::cloned_ref_to_slice_refs
)] // berkas test

mod audit_support;
mod common;

use common::Resp;
use foundation::{SystemClock, Uuid, new_id};
use observability::EnvSource;
use rs_backend::app::{AppServices, AppState, RegistrationGuard};
use rs_backend::rate_limit::{RateBudget, RateLimiter};
use serde_json::{Value, json};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};

const REGISTER: &str = "/v1/public/corporate-registrations";
const CONSENT_TEXT: &str = "/v1/public/corporate-registrations/consent-text";
/// Seed draf teks persetujuan HR dan butir wajibnya (migrasi 20261126310000).
const TEXT_VERSION: &str = "01a2008d-7400-7000-8000-000000000b01";
const TEXT_ITEM: &str = "01a2008d-7400-7000-8000-000000000b02";

async fn pool(var: &str) -> PgPool {
    let url = EnvSource::from_process()
        .required_secret(var)
        .unwrap_or_else(|_| {
            panic!(
                "{var} tidak diset: jalankan lewat `bash scripts/db-ephemeral.sh cargo test ...`"
            )
        });
    PgPoolOptions::new()
        .max_connections(8)
        .connect(url.expose())
        .await
        .unwrap()
}

fn random_hex(n: usize) -> String {
    let mut b = vec![0u8; n];
    crypto::fill_random(&mut b).unwrap();
    b.iter().map(|x| format!("{x:02x}")).collect()
}

fn random_digits(n: usize) -> String {
    let mut b = vec![0u8; n];
    crypto::fill_random(&mut b).unwrap();
    b.iter().map(|x| char::from(b'0' + x % 10)).collect()
}

fn roomy() -> RateLimiter {
    RateLimiter::new(RateBudget {
        per_source: 10_000,
        global: 10_000,
    })
}

struct Options {
    guard: RegistrationGuard,
    reject_drafts: bool,
}

impl Options {
    fn quick() -> Self {
        Self {
            guard: RegistrationGuard {
                limiter: roomy(),
                consent_text_limiter: roomy(),
                min_response: Duration::ZERO,
            },
            reject_drafts: false,
        }
    }
}

struct World {
    addr: SocketAddr,
    app: PgPool,
    relay: PgPool,
}

async fn world(o: Options) -> World {
    let app = pool("RS_TEST_DB_APP_URL").await;
    let relay = pool("RS_TEST_DB_RELAY_URL").await;
    let keys = common::TestKeys::generate();
    let files = Arc::new(common::unused_file_storage());
    let mut state = AppState::new(
        common::TYPE_BASE,
        Vec::new(),
        AppServices {
            pool: app.clone(),
            clock: Arc::new(SystemClock),
            identity: common::unused_identity_services(app.clone(), Arc::clone(&files)),
            files,
            public_content_limit: common::roomy_limiter(),
            public_image_limit: common::roomy_limiter(),
            consent_text_limit: common::roomy_limiter(),
            corporate_hr: common::corporate_hr(app.clone()),
        },
    )
    .with_registration_guard(o.guard);
    if o.reject_drafts {
        state = state.reject_consent_drafts();
    }
    let auth = common::auth_state(&keys, Arc::new(SystemClock));
    let directory = common::account_directory(app.clone());
    let addr = common::spawn_app(state, auth, directory.clone(), directory).await;
    World { addr, app, relay }
}

#[derive(Clone)]
struct Form {
    legal_name: String,
    nib: String,
    npwp: String,
    email: String,
}

impl Form {
    fn new() -> Self {
        Self {
            legal_name: format!("PT Uji Daftar {}", random_hex(4)),
            nib: random_digits(13),
            npwp: random_digits(16),
            email: format!("daftar-{}@example.com", random_hex(6)),
        }
    }

    fn body(&self) -> Value {
        json!({
            "legal_name": self.legal_name,
            "nib": self.nib,
            "npwp": self.npwp,
            "office_address": "Jl. Uji Sintetis No. 7, Jakarta",
            "office_phone": "021-5550101",
            "hr_full_name": "Sari Pendaftar Sintetis",
            "hr_job_title": "Manajer SDM Sintetis",
            "hr_email": self.email,
            "consent_text_version_id": TEXT_VERSION,
            "consent_item_ids": [TEXT_ITEM],
        })
    }
}

/// Isi respons tanpa `correlation_id` (berbeda per permintaan).
fn answer(r: &Resp) -> (u16, Value) {
    let mut v = r.json();
    if let Some(o) = v.as_object_mut() {
        o.remove("correlation_id");
    }
    (r.status, v)
}

fn code(r: &Resp) -> String {
    r.json()["response_code"].as_str().unwrap_or("").to_string()
}

fn field_names(r: &Resp) -> Vec<String> {
    let mut v: Vec<String> = r.json()["errors"]
        .as_array()
        .map(|a| {
            a.iter()
                .filter_map(|e| e["field"].as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    v.sort();
    v
}

/// Penghitung baris yang tidak boleh berubah oleh pendaftaran yang ditolak atau diseragamkan, dibatasi pada NIB
/// dan email uji (test lain berjalan paralel di database yang sama).
#[derive(Debug, PartialEq, Eq)]
struct Counts {
    clients: i64,
    accounts: i64,
    members: i64,
    intents: i64,
    jobs: i64,
    consents: i64,
}

impl World {
    async fn post(&self, body: &Value) -> Resp {
        common::request_json(self.addr, "POST", REGISTER, &[], body).await
    }

    async fn counts(&self, nibs: &[String], emails: &[String]) -> Counts {
        let r = sqlx::query!(
            r#"SELECT
                 (SELECT count(*) FROM app.corporate_client c WHERE c.nib = ANY($1)) AS "clients!",
                 (SELECT count(*) FROM app.account a
                   WHERE a.idp_realm = 'korporat' AND lower(a.email) = ANY($2)) AS "accounts!",
                 (SELECT count(*) FROM app.corporate_member m
                   WHERE m.account_id IN (SELECT a.id FROM app.account a
                                           WHERE a.idp_realm = 'korporat' AND lower(a.email) = ANY($2))) AS "members!",
                 (SELECT count(*) FROM app.idp_provisioning_intent i
                   WHERE i.account_id IN (SELECT a.id FROM app.account a
                                           WHERE a.idp_realm = 'korporat' AND lower(a.email) = ANY($2))) AS "intents!",
                 (SELECT count(*) FROM app.job j
                   WHERE j.job_type = 'idp_create_user'
                     AND (j.payload ->> 'intent_id')::uuid IN (
                           SELECT i.id FROM app.idp_provisioning_intent i
                            WHERE i.account_id IN (SELECT a.id FROM app.account a
                                                    WHERE a.idp_realm = 'korporat'
                                                      AND lower(a.email) = ANY($2)))) AS "jobs!",
                 (SELECT count(*) FROM app.consent_record r
                   WHERE r.account_id IN (SELECT a.id FROM app.account a
                                           WHERE a.idp_realm = 'korporat'
                                             AND lower(a.email) = ANY($2))) AS "consents!""#,
            nibs,
            emails,
        )
        .fetch_one(&self.app)
        .await
        .unwrap();
        Counts {
            clients: r.clients,
            accounts: r.accounts,
            members: r.members,
            intents: r.intents,
            jobs: r.jobs,
            consents: r.consents,
        }
    }

    async fn client_of(&self, nib: &str) -> Option<Uuid> {
        sqlx::query_scalar!(
            r#"SELECT id AS "id!" FROM app.corporate_client WHERE nib = $1 AND deleted_at IS NULL"#,
            nib
        )
        .fetch_optional(&self.app)
        .await
        .unwrap()
    }

    /// Event audit milik perusahaan (tenant), urut waktu.
    async fn tenant_events(&self, client: Uuid) -> Vec<Value> {
        sqlx::query_scalar!(
            r#"SELECT payload AS "payload!" FROM app.audit_outbox
               WHERE payload ->> 'tenant_id' = $1 ORDER BY occurred_at, id"#,
            client.hyphenated().to_string(),
        )
        .fetch_all(&self.relay)
        .await
        .unwrap()
    }

    /// Event audit untuk satu permintaan (ID korelasi).
    async fn events_by_correlation(&self, correlation: &str) -> Vec<Value> {
        sqlx::query_scalar!(
            r#"SELECT payload AS "payload!" FROM app.audit_outbox
               WHERE payload ->> 'correlation_id' = $1 ORDER BY occurred_at, id"#,
            correlation,
        )
        .fetch_all(&self.relay)
        .await
        .unwrap()
    }

    async fn insert_client(&self, status: &str, nib: &str) -> Uuid {
        let id = new_id();
        sqlx::query!(
            "INSERT INTO app.corporate_client (id, legal_name, nib, npwp, office_address, office_phone, status, origin) \
             VALUES ($1, 'PT Uji Lain', $2, $3, 'Jl. Uji', '021', $4, 'self_registered')",
            id,
            nib,
            random_digits(16),
            status,
        )
        .execute(&self.app)
        .await
        .unwrap();
        id
    }

    async fn insert_hr(&self, client: Uuid, email: &str) -> Uuid {
        let account = new_id();
        sqlx::query!(
            "INSERT INTO app.account (id, account_type, idp_realm, idp_subject, email, status, status_changed_at) \
             VALUES ($1, 'corporate_admin', 'korporat', $2, $3, 'active', now())",
            account,
            format!("hr-{}", random_hex(6)),
            email,
        )
        .execute(&self.app)
        .await
        .unwrap();
        sqlx::query!(
            "INSERT INTO app.corporate_member (id, account_id, corporate_client_id) VALUES ($1, $2, $3)",
            new_id(),
            account,
            client,
        )
        .execute(&self.app)
        .await
        .unwrap();
        account
    }
}

// trace: US-1901 AC1, AC5; ADR-030 butir 2, 5; KEPUTUSAN-303 (tanpa password), KEPUTUSAN-322 (E-13 HR)
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn corporate_registration_creates_pending_company_hr_account_consent_and_complete_audit() {
    let w = world(Options::quick()).await;
    let f = Form::new();
    let r = w.post(&f.body()).await;
    assert_eq!(r.status, 202, "{}", r.body);
    assert_eq!(r.header("cache-control"), Some("no-store"));
    assert_eq!(code(&r), "2020000");
    assert_eq!(r.json()["data"]["status"], "processing");
    let cid = r.json()["correlation_id"].as_str().unwrap().to_string();

    let client = w.client_of(&f.nib).await.expect("perusahaan tersimpan");
    let c = sqlx::query!(
        r#"SELECT status AS "status!", origin AS "origin!", submission_round AS "round!", created_by,
                  legal_name AS "legal_name!", npwp AS "npwp!"
           FROM app.corporate_client WHERE id = $1"#,
        client
    )
    .fetch_one(&w.app)
    .await
    .unwrap();
    assert_eq!(
        (c.status.as_str(), c.origin.as_str(), c.round),
        ("pending_verification", "self_registered", 1)
    );
    assert_eq!(c.legal_name, f.legal_name);
    assert_eq!(c.npwp, f.npwp);

    // Akun HR menunggu penyediaan, tanpa idp_subject dan tanpa password di mana pun.
    let acc = sqlx::query!(
        r#"SELECT id AS "id!", account_type AS "t!", idp_realm AS "realm!", idp_subject, status AS "status!",
                  full_name
           FROM app.account WHERE idp_realm = 'korporat' AND lower(email) = lower($1) AND deleted_at IS NULL"#,
        f.email
    )
    .fetch_one(&w.app)
    .await
    .unwrap();
    assert_eq!(
        (acc.t.as_str(), acc.realm.as_str(), acc.status.as_str()),
        ("corporate_admin", "korporat", "pending_provisioning")
    );
    assert_eq!(acc.idp_subject, None, "Keycloak hanya disentuh worker");
    assert_eq!(acc.full_name.as_deref(), Some("Sari Pendaftar Sintetis"));
    assert_eq!(c.created_by, Some(acc.id));
    let m = sqlx::query!(
        r#"SELECT corporate_client_id, job_title FROM app.corporate_member
           WHERE account_id = $1 AND deleted_at IS NULL"#,
        acc.id
    )
    .fetch_one(&w.app)
    .await
    .unwrap();
    assert_eq!(m.corporate_client_id, client);
    assert_eq!(m.job_title.as_deref(), Some("Manajer SDM Sintetis"));

    // Satu niat create_user (hr_self_registration, menunggu) dan satu job berisi ID niat saja.
    let intents = sqlx::query!(
        r#"SELECT i.id AS "id!", i.operation AS "operation!", i.purpose AS "purpose!", s.status AS "status!"
           FROM app.idp_provisioning_intent i
           JOIN app.idp_provisioning_intent_state s ON s.idp_provisioning_intent_id = i.id
           WHERE i.account_id = $1"#,
        acc.id
    )
    .fetch_all(&w.app)
    .await
    .unwrap();
    assert_eq!(intents.len(), 1);
    assert_eq!(
        (
            intents[0].operation.as_str(),
            intents[0].purpose.as_str(),
            intents[0].status.as_str()
        ),
        ("create_user", "hr_self_registration", "pending")
    );
    let jobs = sqlx::query_scalar!(
        r#"SELECT payload AS "payload!" FROM app.job
           WHERE job_type = 'idp_create_user' AND (payload ->> 'intent_id')::uuid = $1"#,
        intents[0].id
    )
    .fetch_all(&w.app)
    .await
    .unwrap();
    assert_eq!(jobs.len(), 1);
    assert_eq!(
        jobs[0],
        json!({ "intent_id": intents[0].id.hyphenated().to_string() })
    );

    // Bukti persetujuan: subjek akun HR, kanal corporate_portal, versi dan butir seed.
    let cr = sqlx::query!(
        r#"SELECT patient_id, care_recipient_id, channel AS "channel!", action AS "action!",
                  consent_text_version_id AS "v!", consent_item_id AS "i!"
           FROM app.consent_record WHERE account_id = $1"#,
        acc.id
    )
    .fetch_all(&w.app)
    .await
    .unwrap();
    assert_eq!(cr.len(), 1);
    assert_eq!(cr[0].patient_id, None);
    assert_eq!(cr[0].care_recipient_id, None);
    assert_eq!(cr[0].channel, "corporate_portal");
    assert_eq!(cr[0].action, "granted");
    assert_eq!(cr[0].v.to_string(), TEXT_VERSION);
    assert_eq!(cr[0].i.to_string(), TEXT_ITEM);

    // Audit: E-10 x2, E-11, E-13, lengkap menurut PRD §7.2 dan tanpa data pribadi/rahasia (ADR-005).
    let events = w.tenant_events(client).await;
    let mut codes: Vec<&str> = events
        .iter()
        .map(|e| e["event_code"].as_str().unwrap())
        .collect();
    codes.sort_unstable();
    assert_eq!(codes, ["e_10", "e_10", "e_11", "e_13"], "{events:?}");
    audit_support::assert_events_complete(&events, &["e_10", "e_11", "e_13"]);
    let e13 = events.iter().find(|e| e["event_code"] == "e_13").unwrap();
    assert_eq!(e13["object_type"], "corporate_hr_consent_record");
    assert_eq!(e13["actor_account_type"], "corporate_admin");
    assert_eq!(e13["source_component"], "corporate_portal");
    assert_eq!(e13["subjects"], json!([]), "HR bukan penerima layanan");
    let created = events
        .iter()
        .find(|e| e["action_name"] == "corporate_client_self_registered")
        .unwrap();
    assert_eq!(created["after_values"]["origin"], "self_registered");
    assert_eq!(created["after_values"]["status"], "pending_verification");
    assert_eq!(created["correlation_id"].as_str(), Some(cid.as_str()));
    let dump = serde_json::to_string(&events).unwrap().to_lowercase();
    for secret in [
        f.nib.as_str(),
        f.npwp.as_str(),
        f.email.as_str(),
        f.legal_name.as_str(),
        "sari pendaftar",
        "manajer sdm",
        "jl. uji sintetis",
        "021-5550101",
        "password",
    ] {
        assert!(
            !dump.contains(&secret.to_lowercase()),
            "audit memuat data rahasia/pribadi: {secret}"
        );
    }
    // Niat dan job juga tanpa data pribadi.
    let intent_dump = sqlx::query_scalar!(
        r#"SELECT (SELECT coalesce(string_agg(i::text, ' '), '') FROM app.idp_provisioning_intent i WHERE i.account_id = $1)
                  || (SELECT coalesce(string_agg(j.payload::text, ' '), '') FROM app.job j
                       WHERE j.job_type = 'idp_create_user' AND (j.payload ->> 'intent_id')::uuid = $2) AS "t!""#,
        acc.id,
        intents[0].id
    )
    .fetch_one(&w.app)
    .await
    .unwrap()
    .to_lowercase();
    assert!(!intent_dump.contains(&f.email.to_lowercase()));
    assert!(!intent_dump.contains("password"));
}

// trace: US-1901 AC4, AC7; KEPUTUSAN-324 (anti-enumerasi); isolasi perusahaan lain; ADR-005 (audit tanpa data)
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn corporate_registration_existing_nib_or_email_gets_identical_answer_and_changes_nothing() {
    let w = world(Options::quick()).await;
    let first = Form::new();
    let baseline = w.post(&first.body()).await;
    assert_eq!(baseline.status, 202, "{}", baseline.body);
    let expected = answer(&baseline);

    // Perusahaan lain (status apa pun) beserta HR aktifnya: NIB dan email-nya tidak boleh bocor atau berubah.
    let nib_rejected = random_digits(13);
    let other = w.insert_client("rejected", &nib_rejected).await;
    let hr_email = format!("hr-lain-{}@example.com", random_hex(5));
    let other_hr = w.insert_hr(other, &hr_email).await;

    let nibs = vec![first.nib.clone(), nib_rejected.clone()];
    let emails = vec![first.email.to_lowercase(), hr_email.to_lowercase()];
    let before = w.counts(&nibs, &emails).await;
    let members_before = sqlx::query_scalar!(
        r#"SELECT count(*) AS "n!" FROM app.corporate_member WHERE corporate_client_id = $1"#,
        other
    )
    .fetch_one(&w.app)
    .await
    .unwrap();

    // 1. NIB sudah dipakai perusahaan Menunggu verifikasi (email baru).
    let mut dup_nib = Form::new();
    dup_nib.nib = first.nib.clone();
    // 2. NIB perusahaan Ditolak (status apa pun tetap memblokir), email baru.
    let mut dup_rejected_nib = Form::new();
    dup_rejected_nib.nib = nib_rejected.clone();
    // 3. Email pendaftar pertama dipakai ulang (NIB baru), huruf besar/kecil berbeda.
    let mut dup_email = Form::new();
    dup_email.email = first.email.to_uppercase();
    // 4. Email HR aktif perusahaan lain (NIB baru): tidak boleh mengikat HR itu ke perusahaan baru.
    let mut foreign_email = Form::new();
    foreign_email.email = hr_email.clone();
    let mut correlations = Vec::new();
    for (label, f) in [
        ("nib menunggu", &dup_nib),
        ("nib ditolak", &dup_rejected_nib),
        ("email pendaftar", &dup_email),
        ("email hr perusahaan lain", &foreign_email),
    ] {
        let r = w.post(&f.body()).await;
        assert_eq!(answer(&r), expected, "{label}: jawaban harus identik");
        assert_eq!(r.header("cache-control"), Some("no-store"), "{label}");
        correlations.push((
            label,
            r.json()["correlation_id"].as_str().unwrap().to_string(),
        ));
    }

    // Nol efek: tidak ada perusahaan, akun, keanggotaan, niat, job, atau bukti persetujuan baru.
    let after = w.counts(&nibs, &emails).await;
    assert_eq!(after, before);
    for f in [&dup_nib, &dup_rejected_nib, &dup_email, &foreign_email] {
        let c = w.counts(&[f.nib.clone()], &[f.email.to_lowercase()]).await;
        // Hanya NIB/email yang memang sudah terdaftar yang terhitung; milik pendaftar yang ditolak tidak ada.
        assert!(c.clients <= 1 && c.accounts <= 1, "{c:?}");
    }
    assert_eq!(
        w.client_of(&dup_email.nib).await,
        None,
        "NIB milik pendaftaran yang diseragamkan tidak tersimpan"
    );
    assert_eq!(w.client_of(&foreign_email.nib).await, None);
    let members_after = sqlx::query_scalar!(
        r#"SELECT count(*) AS "n!" FROM app.corporate_member WHERE corporate_client_id = $1"#,
        other
    )
    .fetch_one(&w.app)
    .await
    .unwrap();
    assert_eq!(
        members_before, members_after,
        "perusahaan lain tidak berubah"
    );
    let other_state = sqlx::query!(
        r#"SELECT status AS "status!" FROM app.corporate_client WHERE id = $1"#,
        other
    )
    .fetch_one(&w.app)
    .await
    .unwrap();
    assert_eq!(other_state.status, "rejected");
    let hr_state = sqlx::query!(
        r#"SELECT status AS "status!" FROM app.account WHERE id = $1"#,
        other_hr
    )
    .fetch_one(&w.app)
    .await
    .unwrap();
    assert_eq!(
        hr_state.status, "active",
        "HR perusahaan lain tidak berubah"
    );

    // Audit: E-10 hasil gagal dengan alasan, aktor sistem, tanpa tenant dan tanpa NIB/email di payload.
    for (label, cid) in &correlations {
        let events = w.events_by_correlation(cid).await;
        assert_eq!(events.len(), 1, "{label}: {events:?}");
        let e = &events[0];
        assert_eq!(e["event_code"], "e_10", "{label}");
        assert_eq!(e["action_name"], "corporate_registration_rejected");
        assert_eq!(e["outcome"], "failure");
        let reason = e["outcome_reason_code"].as_str().unwrap();
        assert!(
            ["nib_already_registered", "hr_email_in_use"].contains(&reason),
            "{label}: {reason}"
        );
        assert_eq!(e["actor_type"], "system");
        assert!(
            e["tenant_id"].is_null(),
            "{label}: tidak membuka perusahaan lain"
        );
        audit_support::assert_event_complete(e, "e_10");
        let dump = serde_json::to_string(e).unwrap().to_lowercase();
        for secret in [
            first.nib.as_str(),
            nib_rejected.as_str(),
            first.email.as_str(),
            hr_email.as_str(),
        ] {
            assert!(!dump.contains(&secret.to_lowercase()), "{label}: {secret}");
        }
        assert!(
            !dump.contains(&other.hyphenated().to_string()),
            "{label}: ID perusahaan lain tidak boleh muncul"
        );
    }
}

// trace: US-1901 AC4, KEPUTUSAN-326 (waktu respons disamakan, ADR-030 butir 7)
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn corporate_registration_new_and_duplicate_take_at_least_the_minimum_time() {
    let hold = Duration::from_millis(500);
    let w = world(Options {
        guard: RegistrationGuard {
            limiter: roomy(),
            consent_text_limiter: roomy(),
            min_response: hold,
        },
        reject_drafts: false,
    })
    .await;
    let f = Form::new();
    let t = Instant::now();
    let first = w.post(&f.body()).await;
    let new_took = t.elapsed();
    assert_eq!(first.status, 202);
    let mut dup = Form::new();
    dup.nib = f.nib.clone();
    let t = Instant::now();
    let second = w.post(&dup.body()).await;
    let dup_took = t.elapsed();
    assert_eq!(answer(&first), answer(&second));
    let slack = Duration::from_millis(20);
    assert!(new_took + slack >= hold, "baru: {new_took:?}");
    assert!(dup_took + slack >= hold, "terdaftar: {dup_took:?}");
    // Selisih kedua jalur kecil (kedua ditahan pada batas bawah yang sama).
    let diff = new_took.abs_diff(dup_took);
    assert!(diff < Duration::from_millis(250), "selisih {diff:?}");
}

// trace: US-1901 AC1, US-104 AC5 (persetujuan wajib), US-104 AC3 (versi basi)
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn corporate_registration_requires_consent_and_current_text_version() {
    let w = world(Options::quick()).await;
    let f = Form::new();
    let nibs = vec![f.nib.clone()];
    let emails = vec![f.email.to_lowercase()];
    let before = w.counts(&nibs, &emails).await;
    let events_before = sqlx::query_scalar!(r#"SELECT count(*) AS "n!" FROM app.audit_outbox"#)
        .fetch_one(&w.relay)
        .await
        .unwrap();

    // Tanpa butir, atau butir kosong: 422 per field.
    for items in [json!([]), json!([])] {
        let mut b = f.body();
        b["consent_item_ids"] = items;
        let r = w.post(&b).await;
        assert_eq!(r.status, 422, "{}", r.body);
        assert_eq!(code(&r), "4220001");
        assert_eq!(field_names(&r), ["consent_item_ids"]);
    }
    // Versi lain, butir asing, ID bukan UUID.
    let mut stale = f.body();
    stale["consent_text_version_id"] = json!(new_id().hyphenated().to_string());
    let r = w.post(&stale).await;
    assert_eq!(
        (r.status, code(&r).as_str()),
        (409, "4090106"),
        "{}",
        r.body
    );
    let mut foreign = f.body();
    foreign["consent_item_ids"] = json!([TEXT_ITEM, new_id().hyphenated().to_string()]);
    let r = w.post(&foreign).await;
    assert_eq!(
        (r.status, code(&r).as_str()),
        (409, "4090106"),
        "{}",
        r.body
    );
    let mut bad_id = f.body();
    bad_id["consent_text_version_id"] = json!("bukan-uuid");
    let r = w.post(&bad_id).await;
    assert_eq!(r.status, 422);
    assert_eq!(field_names(&r), ["consent_text_version_id"]);
    // Butir persetujuan HR dicoba dengan teks pasien (versi seed pasien): bukan versi berlaku untuk HR.
    let mut patient_text = f.body();
    patient_text["consent_text_version_id"] = json!("01a2008d-7400-7000-8000-000000000a49");
    patient_text["consent_item_ids"] = json!(["01a2008d-7400-7000-8000-000000000a4a"]);
    let r = w.post(&patient_text).await;
    assert_eq!(
        (r.status, code(&r).as_str()),
        (409, "4090106"),
        "{}",
        r.body
    );

    assert_eq!(w.counts(&nibs, &emails).await, before, "nol efek");
    let events_after = sqlx::query_scalar!(r#"SELECT count(*) AS "n!" FROM app.audit_outbox"#)
        .fetch_one(&w.relay)
        .await
        .unwrap();
    assert!(
        events_after >= events_before,
        "outbox hanya bertambah oleh test lain"
    );
    // Dengan persetujuan lengkap pendaftaran berhasil (kontrol).
    assert_eq!(w.post(&f.body()).await.status, 202);
    assert_eq!(w.counts(&nibs, &emails).await.consents, 1);
}

// trace: KEPUTUSAN-289, US-1901 AC1: teks draf ditolak di production (nol efek)
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn corporate_registration_draft_text_is_refused_in_production() {
    let w = world(Options {
        reject_drafts: true,
        ..Options::quick()
    })
    .await;
    let f = Form::new();
    let before = w.counts(&[f.nib.clone()], &[f.email.to_lowercase()]).await;
    let r = w.post(&f.body()).await;
    assert_eq!(
        (r.status, code(&r).as_str()),
        (503, "5030001"),
        "{}",
        r.body
    );
    assert!(
        !r.body.contains("DRAF") && !r.body.contains("menunggu L-20"),
        "isi draf tidak dibocorkan"
    );
    let t = common::request(w.addr, "GET", CONSENT_TEXT, &[]).await;
    assert_eq!(
        (t.status, code(&t).as_str()),
        (503, "5030001"),
        "{}",
        t.body
    );
    assert!(!t.body.contains("DRAF"));
    assert_eq!(
        w.counts(&[f.nib.clone()], &[f.email.to_lowercase()]).await,
        before
    );
    assert_eq!(w.client_of(&f.nib).await, None);
}

// trace: US-1901 AC1, US-104 AC1: teks persetujuan HR tersedia tanpa login di lingkungan non-production
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn corporate_registration_consent_text_is_public_outside_production() {
    let w = world(Options::quick()).await;
    let r = common::request(w.addr, "GET", CONSENT_TEXT, &[]).await;
    assert_eq!(r.status, 200, "{}", r.body);
    assert_eq!(r.header("cache-control"), Some("no-store"));
    let d = &r.json()["data"];
    assert_eq!(d["consent_text_version_id"], TEXT_VERSION);
    assert_eq!(d["consent_type"], "corporate_hr_registration");
    assert!(d["draft_notice"].is_string());
    assert_eq!(d["sections"].as_array().unwrap().len(), 7);
    assert_eq!(d["items"][0]["consent_item_id"], TEXT_ITEM);
    assert_eq!(d["items"][0]["is_required"], true);
}

// trace: US-1901 AC1, AC3; KEPUTUSAN-325 (NIB 13 angka, NPWP 16 angka), KEPUTUSAN-303 (tanpa password)
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn corporate_registration_validates_each_field_and_never_accepts_a_password() {
    let w = world(Options::quick()).await;
    let f = Form::new();
    let nibs = vec![f.nib.clone()];
    let emails = vec![f.email.to_lowercase()];
    let before = w.counts(&nibs, &emails).await;
    let bad_nibs = [
        random_digits(12),
        random_digits(14),
        "12345678901a3".to_string(),
        "1234 5678 9012".to_string(),
        String::new(),
    ];
    for nib in bad_nibs {
        let mut b = f.body();
        b["nib"] = json!(nib);
        let r = w.post(&b).await;
        assert_eq!(r.status, 422, "{nib}: {}", r.body);
        assert_eq!(field_names(&r), ["nib"], "{nib}");
    }
    // NPWP 15 angka (format lama) ditolak tanpa ditambah angka 0 secara diam-diam; 17 angka dan non-angka juga.
    for npwp in [
        random_digits(15),
        random_digits(17),
        "01.234.567.8-901.234".to_string(),
        "01234567890123ab".to_string(),
        String::new(),
    ] {
        let mut b = f.body();
        b["npwp"] = json!(npwp);
        let r = w.post(&b).await;
        assert_eq!(r.status, 422, "{npwp}: {}", r.body);
        assert_eq!(field_names(&r), ["npwp"], "{npwp}");
        assert!(
            !r.body.contains(&npwp) || npwp.is_empty(),
            "NPWP tidak dipantulkan"
        );
    }
    let mut many = f.body();
    many["legal_name"] = json!("x".repeat(201));
    many["office_address"] = json!("");
    many["office_phone"] = json!("0".repeat(33));
    many["hr_full_name"] = json!("   ");
    many["hr_job_title"] = json!("j".repeat(101));
    many["hr_email"] = json!("bukan-email");
    let r = w.post(&many).await;
    assert_eq!(r.status, 422);
    assert_eq!(
        field_names(&r),
        [
            "hr_email",
            "hr_full_name",
            "hr_job_title",
            "legal_name",
            "office_address",
            "office_phone"
        ]
    );
    // Password (atau field lain di luar skema) tidak diterima: password dibuat lewat tautan aktivasi.
    let mut with_password = f.body();
    with_password["password"] = json!("Rahasia-Sintetis-123!");
    let r = w.post(&with_password).await;
    assert_eq!(r.status, 422, "{}", r.body);
    assert!(
        !r.body.contains("Rahasia-Sintetis"),
        "password tidak dipantulkan"
    );
    // Badan rusak dan jenis media.
    let r = common::request_with_body(
        w.addr,
        "POST",
        REGISTER,
        &["Content-Type: application/json"],
        Some(b"{bukan json"),
    )
    .await;
    assert_eq!(r.status, 400, "{}", r.body);
    let r = common::request_with_body(
        w.addr,
        "POST",
        REGISTER,
        &["Content-Type: text/plain"],
        Some(f.body().to_string().as_bytes()),
    )
    .await;
    assert_eq!(r.status, 415, "{}", r.body);
    assert_eq!(w.counts(&nibs, &emails).await, before, "nol efek");
    // Kontrol: NIB 13 angka dan NPWP 16 angka diterima.
    assert_eq!(w.post(&f.body()).await.status, 202);
}

// trace: US-1901 AC4, RT-45 (batas laju pendaftaran), KEPUTUSAN-326; gagal tanpa efek
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn corporate_registration_is_rate_limited_per_source_without_side_effects() {
    let w = world(Options {
        guard: RegistrationGuard {
            limiter: RateLimiter::new(RateBudget {
                per_source: 3,
                global: 10_000,
            }),
            consent_text_limiter: roomy(),
            min_response: Duration::ZERO,
        },
        reject_drafts: false,
    })
    .await;
    let forms: Vec<Form> = (0..5).map(|_| Form::new()).collect();
    let mut statuses = Vec::new();
    for f in &forms {
        statuses.push(w.post(&f.body()).await);
    }
    let codes: Vec<u16> = statuses.iter().map(|r| r.status).collect();
    assert_eq!(codes, [202, 202, 202, 429, 429]);
    assert_eq!(code(&statuses[3]), "4290001");
    // Yang ditolak batas laju tidak meninggalkan efek apa pun.
    for f in &forms[3..] {
        assert_eq!(w.client_of(&f.nib).await, None);
        let c = w.counts(&[f.nib.clone()], &[f.email.to_lowercase()]).await;
        assert_eq!(
            c,
            Counts {
                clients: 0,
                accounts: 0,
                members: 0,
                intents: 0,
                jobs: 0,
                consents: 0
            }
        );
    }
    // Anggaran GET teks persetujuan terpisah: banjir POST tidak memblokirnya.
    let t = common::request(w.addr, "GET", CONSENT_TEXT, &[]).await;
    assert_eq!(t.status, 200, "{}", t.body);
}

// trace: M-47, RT-45 (anggaran terpisah: banjir GET teks persetujuan tidak memblokir POST pendaftaran)
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn corporate_registration_consent_text_flood_does_not_block_registration() {
    let w = world(Options {
        guard: RegistrationGuard {
            limiter: roomy(),
            consent_text_limiter: RateLimiter::new(RateBudget {
                per_source: 2,
                global: 10_000,
            }),
            min_response: Duration::ZERO,
        },
        reject_drafts: false,
    })
    .await;
    let mut got = Vec::new();
    for _ in 0..4 {
        got.push(
            common::request(w.addr, "GET", CONSENT_TEXT, &[])
                .await
                .status,
        );
    }
    assert_eq!(got, [200, 200, 429, 429]);
    let f = Form::new();
    assert_eq!(
        w.post(&f.body()).await.status,
        202,
        "POST tidak ikut terblokir"
    );
}

// trace: ADR-030 butir 5 (kunci advisory email), US-1901 AC7 (balapan NIB = satu perusahaan)
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn corporate_registration_concurrent_same_nib_creates_one_company() {
    let w = world(Options::quick()).await;
    let nib = random_digits(13);
    let forms: Vec<Form> = (0..6)
        .map(|_| {
            let mut f = Form::new();
            f.nib = nib.clone();
            f
        })
        .collect();
    let addr = w.addr;
    let mut tasks = Vec::new();
    for f in &forms {
        let body = f.body();
        tasks.push(tokio::spawn(async move {
            common::request_json(addr, "POST", REGISTER, &[], &body).await
        }));
    }
    let mut answers = Vec::new();
    for t in tasks {
        answers.push(answer(&t.await.unwrap()));
    }
    assert!(
        answers.iter().all(|a| a == &answers[0]),
        "semua jawaban sama"
    );
    assert_eq!(answers[0].0, 202);
    let emails: Vec<String> = forms.iter().map(|f| f.email.to_lowercase()).collect();
    let c = w.counts(&[nib.clone()], &emails).await;
    assert_eq!(c.clients, 1);
    assert_eq!(c.accounts, 1, "hanya pemenang yang punya akun");
    assert_eq!((c.members, c.intents, c.jobs, c.consents), (1, 1, 1, 1));

    // Dua pendaftaran bersamaan dengan email sama (NIB berbeda): satu akun.
    let email = format!("balap-{}@example.com", random_hex(5));
    let a = Form {
        email: email.clone(),
        ..Form::new()
    };
    let b = Form {
        email: email.to_uppercase(),
        ..Form::new()
    };
    let (body_a, body_b) = (a.body(), b.body());
    let (ra, rb) = tokio::join!(w.post(&body_a), w.post(&body_b));
    assert_eq!(answer(&ra), answer(&rb));
    let c = w
        .counts(&[a.nib.clone(), b.nib.clone()], &[email.to_lowercase()])
        .await;
    assert_eq!((c.clients, c.accounts, c.intents, c.jobs), (1, 1, 1, 1));
}

// trace: ADR-012 butir 3 (amandemen KEPUTUSAN-323), US-1904 AC4: token tidak mengubah jalur publik
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn corporate_registration_endpoint_ignores_unrelated_credentials_and_unknown_methods() {
    let w = world(Options::quick()).await;
    // Metode lain pada rute yang sama dan rute di bawahnya yang tidak terdaftar dijawab 404 seragam.
    for (m, p) in [
        ("GET", REGISTER),
        ("DELETE", REGISTER),
        ("POST", CONSENT_TEXT),
        ("GET", "/v1/public/corporate-registrations/lain"),
    ] {
        let r = common::request(w.addr, m, p, &[]).await;
        assert_eq!(r.status, 404, "{m} {p}");
    }
    // Token sembarang tidak diperlukan dan tidak mengubah jawaban (jalur publik).
    let f = Form::new();
    let r = common::request_json(
        w.addr,
        "POST",
        REGISTER,
        &["Authorization: Bearer token-sembarang"],
        &f.body(),
    )
    .await;
    assert_eq!(r.status, 202, "{}", r.body);
}
