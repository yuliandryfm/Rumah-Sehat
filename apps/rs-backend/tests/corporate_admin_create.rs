//! F1-T105 (T1; US-1903 AC1-AC4; BR-136; ADR-030; KEPUTUSAN-303/306/328/329): perusahaan dibuatkan Verifikator
//! lewat `POST /backoffice/v1/corporate-clients` memakai proses `api` lengkap (router produksi, validator token
//! tiga realm, resolver akun sungguhan) terhadap PostgreSQL 18 sekali pakai (`scripts/db-ephemeral.sh`, role
//! runtime `rs_api`). Tanpa variabel `RS_TEST_DB_*` test GAGAL.
//!
//! Yang diuji: perusahaan langsung Aktif tanpa keputusan verifikasi; HR pertama = akun `pending_provisioning`
//! terikat perusahaan + SATU niat `create_user` (tujuan `hr_invitation`) + SATU job `idp_create_user` berisi ID
//! niat saja (tanpa email, NPWP, atau data lain) dan tanpa `idp_subject` (efek Keycloak hanya milik worker);
//! E-10 memuat pelaku dan sumber `created_by_admin` (AC3); NIB ganda (juga dengan spasi tepi), email HR sudah
//! dipakai (huruf besar/kecil), dan penolakan tanpa efek apa pun beserta E-10 hasil gagal; email di realm lain
//! tidak bentrok; balapan NIB dan email; lima skenario penolakan akses (tanpa token, token rusak/kedaluwarsa,
//! realm lain termasuk HR, akun nonaktif, sub-peran selain Verifikator termasuk Super Admin); validasi isian;
//! tidak ada pembuatan ulang akun atau niat; dan modul tidak memuat klien Keycloak Admin REST.
//!
//! Kedaluwarsa 24 jam dan sekali pakai tautan aktivasi (AC2) adalah perilaku token aksi Keycloak: diuji di
//! `infra/keycloak/test/keycloak_check.py` (`cek_undangan_hr`), bukan di sini.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)] // berkas test

mod audit_support;
mod common;

use common::{
    ADMIN_AUD, ADMIN_ISS, KORPORAT_AUD, KORPORAT_ISS, PASIEN_AUD, PASIEN_ISS, Resp, TestKeys, token,
};
use foundation::{Clock, SystemClock, Uuid, new_id};
use observability::EnvSource;
use rs_backend::app::{AppServices, AppState};
use serde_json::{Value, json};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use std::sync::Arc;

const CREATE: &str = "/backoffice/v1/corporate-clients";

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

struct World {
    addr: SocketAddr,
    keys: TestKeys,
    app: PgPool,
    relay: PgPool,
}

async fn world() -> World {
    let app = pool("RS_TEST_DB_APP_URL").await;
    let relay = pool("RS_TEST_DB_RELAY_URL").await;
    let keys = TestKeys::generate();
    let files = Arc::new(common::unused_file_storage());
    let state = AppState::new(
        common::TYPE_BASE,
        Vec::new(),
        AppServices {
            pool: app.clone(),
            clock: Arc::new(SystemClock),
            identity: common::unused_identity_services(app.clone(), Arc::clone(&files)),
            files,
            public_content_limit: common::roomy_limiter(),
            public_image_limit: common::roomy_limiter(),
            corporate_hr: common::corporate_hr(app.clone()),
        },
    );
    let auth = common::auth_state(&keys, Arc::new(SystemClock));
    let directory = common::account_directory(app.clone());
    let addr = common::spawn_app(state, auth, directory.clone(), directory).await;
    World {
        addr,
        keys,
        app,
        relay,
    }
}

/// Akun admin Ruang Sehat dengan sub-peran `roles` dan status `status`; mengembalikan (ID, subjek).
async fn admin(pool: &PgPool, roles: &[&str], status: &str) -> (Uuid, String) {
    let id = new_id();
    let subject = format!("uji-adm-{}", random_hex(8));
    sqlx::query!(
        "INSERT INTO app.account (id, account_type, idp_realm, idp_subject, email, status, status_changed_at, full_name) \
         VALUES ($1, 'rs_admin', 'admin', $2, $3, $4, now(), 'Admin Sintetis')",
        id,
        subject,
        format!("{subject}@example.com"),
        status,
    )
    .execute(pool)
    .await
    .unwrap();
    for r in roles {
        sqlx::query!(
            "INSERT INTO app.rs_admin_role (id, account_id, sub_role) VALUES ($1, $2, $3)",
            new_id(),
            id,
            r,
        )
        .execute(pool)
        .await
        .unwrap();
    }
    (id, subject)
}

/// Perusahaan sintetis langsung di database (status `status`, NIB `nib`).
async fn insert_client(pool: &PgPool, nib: &str, status: &str) -> Uuid {
    let id = new_id();
    sqlx::query!(
        "INSERT INTO app.corporate_client (id, legal_name, nib, npwp, office_address, office_phone, status, origin) \
         VALUES ($1, 'PT Pemilik NIB Lama', $2, $3, 'Jl. Uji No. 1', '0210000000', $4, 'self_registered')",
        id,
        nib,
        format!("NPWP{}", random_hex(8)),
        status,
    )
    .execute(pool)
    .await
    .unwrap();
    id
}

/// Akun HR di realm `korporat` dengan email `email` (belum terhapus).
async fn insert_hr_account(pool: &PgPool, email: &str) -> Uuid {
    let id = new_id();
    sqlx::query!(
        "INSERT INTO app.account (id, account_type, idp_realm, idp_subject, email, status, status_changed_at) \
         VALUES ($1, 'corporate_admin', 'korporat', $2, $3, 'active', now())",
        id,
        format!("hr-{}", random_hex(8)),
        email,
    )
    .execute(pool)
    .await
    .unwrap();
    id
}

impl World {
    fn bearer(&self, subject: &str) -> String {
        format!(
            "Authorization: Bearer {}",
            token(&self.keys.admin, ADMIN_ISS, ADMIN_AUD, subject)
        )
    }

    async fn create(&self, subject: &str, body: &Value) -> Resp {
        common::request_json(self.addr, "POST", CREATE, &[&self.bearer(subject)], body).await
    }

    /// Event audit dengan `tenant_id` = perusahaan, urut waktu.
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

    /// Event penolakan pembuatan perusahaan oleh Verifikator `actor`.
    async fn rejected_events(&self, actor: Uuid) -> Vec<Value> {
        sqlx::query_scalar!(
            r#"SELECT payload AS "payload!" FROM app.audit_outbox
               WHERE payload ->> 'actor_account_id' = $1
                 AND payload ->> 'action_name' = 'corporate_client_create_rejected'
               ORDER BY occurred_at, id"#,
            actor.hyphenated().to_string(),
        )
        .fetch_all(&self.relay)
        .await
        .unwrap()
    }

    async fn clients_with_nib(&self, nib: &str) -> i64 {
        sqlx::query_scalar!(
            r#"SELECT count(*) AS "n!" FROM app.corporate_client WHERE nib = $1"#,
            nib
        )
        .fetch_one(&self.app)
        .await
        .unwrap()
    }

    async fn accounts_with_email(&self, email: &str) -> i64 {
        sqlx::query_scalar!(
            r#"SELECT count(*) AS "n!" FROM app.account
               WHERE idp_realm = 'korporat' AND lower(email) = lower($1)"#,
            email
        )
        .fetch_one(&self.app)
        .await
        .unwrap()
    }

    async fn intents_for(&self, account: Uuid) -> Vec<(String, String, String)> {
        sqlx::query!(
            r#"SELECT i.operation AS "operation!", i.purpose AS "purpose!", s.status AS "status!"
               FROM app.idp_provisioning_intent i
               JOIN app.idp_provisioning_intent_state s ON s.idp_provisioning_intent_id = i.id
               WHERE i.account_id = $1 ORDER BY i.created_at, i.id"#,
            account
        )
        .fetch_all(&self.app)
        .await
        .unwrap()
        .into_iter()
        .map(|r| (r.operation, r.purpose, r.status))
        .collect()
    }
}

fn code(r: &Resp) -> String {
    r.json()["response_code"].as_str().unwrap_or("").to_string()
}

fn nib() -> String {
    format!("99{}", random_hex(6))
}

fn email() -> String {
    format!("hr-{}@example.com", random_hex(6))
}

fn body(nib: &str, email: &str) -> Value {
    json!({
        "legal_name": "PT Dibuatkan Verifikator",
        "nib": nib,
        "npwp": format!("98{}", random_hex(7)),
        "office_address": "Jl. Sintetis No. 7, Jakarta",
        "office_phone": "021-0000007",
        "hr_email": email,
    })
}

// trace: US-1903 AC1, AC3, ADR-030 butir 2 dan 5 (perusahaan Aktif, HR pending_provisioning, satu niat, satu job,
// E-10 dengan pelaku dan sumber "dibuat admin")
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn verifier_creates_an_active_company_with_one_provisioning_intent_and_audit() {
    let w = world().await;
    let (verifier, sub) = admin(&w.app, &["verifier"], "active").await;
    let (nib, mixed_email) = (nib(), format!("HR.Baru-{}@Example.COM", random_hex(4)));
    let r = w.create(&sub, &body(&nib, &mixed_email)).await;
    assert_eq!(r.status, 201, "{}", r.body);
    assert_eq!(r.header("cache-control"), Some("no-store"));
    let d = r.json()["data"].clone();
    assert_eq!(d["status"], "active");
    assert_eq!(d["hr_invitation"], "processing");
    let client: Uuid = d["corporate_client_id"].as_str().unwrap().parse().unwrap();
    let member: Uuid = d["corporate_member_id"].as_str().unwrap().parse().unwrap();
    let hr: Uuid = d["hr_account_id"].as_str().unwrap().parse().unwrap();

    // Perusahaan langsung Aktif, dibuat admin, tanpa keputusan verifikasi.
    let c = sqlx::query!(
        r#"SELECT status AS "status!", origin AS "origin!", submission_round AS "round!", created_by
           FROM app.corporate_client WHERE id = $1"#,
        client
    )
    .fetch_one(&w.app)
    .await
    .unwrap();
    assert_eq!(
        (c.status.as_str(), c.origin.as_str(), c.round),
        ("active", "created_by_admin", 1)
    );
    assert_eq!(c.created_by, Some(verifier));
    let decisions = sqlx::query_scalar!(
        r#"SELECT count(*) AS "n!" FROM app.corporate_verification_decision WHERE corporate_client_id = $1"#,
        client
    )
    .fetch_one(&w.app)
    .await
    .unwrap();
    assert_eq!(decisions, 0);

    // HR pertama: akun menunggu penyediaan, tanpa idp_subject, email ternormalisasi, terikat perusahaan.
    let a = sqlx::query!(
        r#"SELECT account_type AS "account_type!", idp_realm AS "realm!", idp_subject, status AS "status!",
                  email AS "email!", created_by
           FROM app.account WHERE id = $1"#,
        hr
    )
    .fetch_one(&w.app)
    .await
    .unwrap();
    assert_eq!(
        (a.account_type.as_str(), a.realm.as_str(), a.status.as_str()),
        ("corporate_admin", "korporat", "pending_provisioning")
    );
    assert_eq!(a.idp_subject, None, "efek Keycloak hanya milik worker");
    assert_eq!(a.email, mixed_email.to_lowercase());
    assert_eq!(a.created_by, Some(verifier));
    let m = sqlx::query!(
        r#"SELECT corporate_client_id, account_id, deleted_at IS NULL AS "active!"
           FROM app.corporate_member WHERE id = $1"#,
        member
    )
    .fetch_one(&w.app)
    .await
    .unwrap();
    assert_eq!(
        (m.corporate_client_id, m.account_id, m.active),
        (client, hr, true)
    );

    // Tepat satu niat create_user (menunggu) dan satu job berisi ID niat saja; tidak ada email aktivasi dari api.
    assert_eq!(
        w.intents_for(hr).await,
        vec![(
            "create_user".to_string(),
            "hr_invitation".to_string(),
            "pending".to_string()
        )]
    );
    let intent = sqlx::query_scalar!(
        r#"SELECT id AS "id!" FROM app.idp_provisioning_intent WHERE account_id = $1"#,
        hr
    )
    .fetch_one(&w.app)
    .await
    .unwrap();
    let jobs = sqlx::query!(
        r#"SELECT job_type AS "job_type!", payload AS "payload!", status AS "status!"
           FROM app.job WHERE payload ->> 'intent_id' = $1"#,
        intent.hyphenated().to_string()
    )
    .fetch_all(&w.app)
    .await
    .unwrap();
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].job_type, "idp_create_user");
    assert_eq!(jobs[0].status, "queued");
    assert_eq!(
        jobs[0].payload,
        json!({ "intent_id": intent.hyphenated().to_string() })
    );

    // Audit: E-10 perusahaan (pelaku + sumber), E-10 akun HR, E-11 keanggotaan; tanpa data terlarang.
    let evs = w.tenant_events(client).await;
    let kinds: Vec<(String, String)> = evs
        .iter()
        .map(|e| {
            (
                e["event_code"].as_str().unwrap().to_string(),
                e["action_name"].as_str().unwrap().to_string(),
            )
        })
        .collect();
    assert_eq!(
        kinds,
        vec![
            (
                "e_10".to_string(),
                "corporate_client_created_by_admin".to_string()
            ),
            (
                "e_10".to_string(),
                "corporate_admin_account_invited".to_string()
            ),
            (
                "e_11".to_string(),
                "corporate_membership_granted".to_string()
            ),
        ]
    );
    audit_support::assert_events_complete(&evs, &["e_10", "e_11"]);
    for e in &evs {
        assert_eq!(e["outcome"], "success");
        assert_eq!(e["actor_account_id"], verifier.to_string());
        assert_eq!(e["actor_account_type"], "rs_admin");
        assert_eq!(e["actor_roles"], json!(["verifier"]));
        assert_eq!(e["source_component"], "backoffice");
        assert!(e["source_ip"].as_str().is_some());
    }
    assert_eq!(evs[0]["object_id"], client.to_string());
    assert_eq!(evs[0]["after_values"]["origin"], "created_by_admin");
    assert_eq!(evs[0]["after_values"]["status"], "active");
    assert_eq!(evs[1]["object_id"], hr.to_string());
    assert_eq!(evs[1]["after_values"]["status"], "pending_provisioning");
    assert_eq!(
        evs[2]["after_values"]["corporate_member_id"],
        member.to_string()
    );
    let all = Value::Array(evs).to_string() + &jobs[0].payload.to_string();
    for secret in [
        mixed_email.to_lowercase().as_str(),
        nib.as_str(),
        "PT Dibuatkan Verifikator",
    ] {
        assert!(
            !all.contains(secret),
            "data terlarang di audit/job: {secret}"
        );
    }
}

// trace: US-1903 AC4, US-1901 AC7 (NIB ganda ditolak dengan pesan umum; tanpa efek; tercatat gagal)
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn duplicate_nib_is_rejected_generically_without_any_side_effect() {
    let w = world().await;
    let (verifier, sub) = admin(&w.app, &["verifier"], "active").await;
    let nib = nib();
    // Terdaftar sebagai Ditolak pun tetap memblokir NIB (uqp_corporate_client_nib).
    insert_client(&w.app, &nib, "rejected").await;
    let hr_email = email();
    for tried in [nib.clone(), format!("  {nib}  ")] {
        let r = w.create(&sub, &body(&tried, &hr_email)).await;
        assert_eq!(
            (r.status, code(&r)),
            (409, "4090110".to_string()),
            "{}",
            r.body
        );
        assert!(
            !r.body.contains("PT Pemilik NIB Lama"),
            "nama perusahaan lain bocor"
        );
        assert!(!r.body.contains(&nib), "NIB tidak digemakan");
    }
    assert_eq!(w.clients_with_nib(&nib).await, 1);
    assert_eq!(
        w.accounts_with_email(&hr_email).await,
        0,
        "akun HR tidak dibuat"
    );
    let evs = w.rejected_events(verifier).await;
    assert_eq!(evs.len(), 2, "{evs:?}");
    for e in &evs {
        audit_support::assert_event_complete(e, "e_10");
        assert_eq!(e["outcome"], "failure");
        assert_eq!(e["outcome_reason_code"], "nib_already_registered");
        assert!(e["changed_fields"].is_null());
        assert!(!e.to_string().contains(&nib));
    }
}

// trace: US-1903 AC1, ADR-030 butir 5 (email HR sudah dipakai di realm korporat: 409 untuk Verifikator, tanpa
// efek; email di realm lain tidak bentrok)
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn hr_email_already_registered_in_the_realm_is_rejected_but_other_realms_do_not_clash() {
    let w = world().await;
    let (verifier, sub) = admin(&w.app, &["verifier"], "active").await;
    let used = email();
    insert_hr_account(&w.app, &used).await;
    for tried in [used.clone(), used.to_uppercase(), format!(" {used} ")] {
        let n = nib();
        let r = w.create(&sub, &body(&n, &tried)).await;
        assert_eq!(
            (r.status, code(&r)),
            (409, "4090111".to_string()),
            "{tried}: {}",
            r.body
        );
        assert_eq!(w.clients_with_nib(&n).await, 0, "perusahaan tidak dibuat");
    }
    assert_eq!(w.accounts_with_email(&used).await, 1);
    let evs = w.rejected_events(verifier).await;
    assert_eq!(evs.len(), 3);
    assert!(
        evs.iter()
            .all(|e| e["outcome_reason_code"] == "hr_email_in_use")
    );
    assert!(evs.iter().all(|e| !e.to_string().contains(&used)));

    // Email yang sama sebagai akun admin (realm lain) tidak menghalangi (keunikan per realm).
    let admin_email: String = sqlx::query_scalar!(
        r#"SELECT email AS "email!" FROM app.account WHERE id = $1"#,
        verifier
    )
    .fetch_one(&w.app)
    .await
    .unwrap();
    let r = w.create(&sub, &body(&nib(), &admin_email)).await;
    assert_eq!(r.status, 201, "{}", r.body);
}

// trace: ADR-030 butir 6 (satu niat satu efek; tidak ada jalur undang ulang yang menduplikasi akun atau niat)
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn repeating_the_same_creation_never_duplicates_the_account_or_the_intent() {
    let w = world().await;
    let (_, sub) = admin(&w.app, &["verifier"], "active").await;
    let hr_email = email();
    let first = w.create(&sub, &body(&nib(), &hr_email)).await;
    assert_eq!(first.status, 201, "{}", first.body);
    let hr: Uuid = first.json()["data"]["hr_account_id"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();
    // Perusahaan lain (NIB berbeda) dengan email yang sama, juga setelah akun menunggu aktivasi.
    for _ in 0..2 {
        let again = w.create(&sub, &body(&nib(), &hr_email)).await;
        assert_eq!((again.status, code(&again)), (409, "4090111".to_string()));
    }
    assert_eq!(w.accounts_with_email(&hr_email).await, 1);
    assert_eq!(w.intents_for(hr).await.len(), 1);
    // Akun yang sudah aktif (undangan dipakai) pun tidak dapat dibuatkan ulang.
    sqlx::query!(
        "UPDATE app.account SET idp_subject = $2, status = 'active' WHERE id = $1",
        hr,
        format!("hr-{}", random_hex(6)),
    )
    .execute(&w.app)
    .await
    .unwrap();
    let r = w.create(&sub, &body(&nib(), &hr_email)).await;
    assert_eq!((r.status, code(&r)), (409, "4090111".to_string()));
    assert_eq!(w.intents_for(hr).await.len(), 1);
}

// trace: US-1903 AC4, ADR-030 butir 5 (balapan NIB dan balapan email: tepat satu pemenang, tanpa sisa)
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_creations_with_the_same_nib_or_the_same_email_create_exactly_one() {
    let w = Arc::new(world().await);
    for round in 0..4 {
        let (_, s1) = admin(&w.app, &["verifier"], "active").await;
        let (_, s2) = admin(&w.app, &["verifier"], "active").await;
        let (n1, n2, e1, e2) = (nib(), nib(), email(), email());
        // Satu NIB, dua email berbeda.
        let (b1, b2) = (body(&n1, &e1), body(&n1, &e2));
        let (w1, w2) = (Arc::clone(&w), Arc::clone(&w));
        let t1 = tokio::spawn(async move { w1.create(&s1, &b1).await });
        let t2 = tokio::spawn(async move { w2.create(&s2, &b2).await });
        let (r1, r2) = (t1.await.unwrap(), t2.await.unwrap());
        let mut got = [(r1.status, code(&r1)), (r2.status, code(&r2))];
        got.sort();
        assert_eq!(
            got,
            [(201, "2010000".to_string()), (409, "4090110".to_string())],
            "ronde {round} (NIB sama): {} / {}",
            r1.body,
            r2.body
        );
        assert_eq!(w.clients_with_nib(&n1).await, 1);
        assert_eq!(
            w.accounts_with_email(&e1).await + w.accounts_with_email(&e2).await,
            1
        );

        // Satu email, dua NIB berbeda.
        let (_, s3) = admin(&w.app, &["verifier"], "active").await;
        let (_, s4) = admin(&w.app, &["verifier"], "active").await;
        let e3 = email();
        let (b3, b4) = (body(&n2, &e3), body(&nib(), &e3));
        let other_nib = b4["nib"].as_str().unwrap().to_string();
        let (w3, w4) = (Arc::clone(&w), Arc::clone(&w));
        let t3 = tokio::spawn(async move { w3.create(&s3, &b3).await });
        let t4 = tokio::spawn(async move { w4.create(&s4, &b4).await });
        let (r3, r4) = (t3.await.unwrap(), t4.await.unwrap());
        let mut got = [(r3.status, code(&r3)), (r4.status, code(&r4))];
        got.sort();
        assert_eq!(
            got,
            [(201, "2010000".to_string()), (409, "4090111".to_string())],
            "ronde {round} (email sama): {} / {}",
            r3.body,
            r4.body
        );
        assert_eq!(w.accounts_with_email(&e3).await, 1);
        assert_eq!(
            w.clients_with_nib(&n2).await + w.clients_with_nib(&other_nib).await,
            1,
            "perusahaan pesaing harus dibatalkan seluruhnya"
        );
    }
}

// trace: US-1902 AC5 pola, coding-standards §7.3 (lima skenario penolakan), US-1903 AC1 (hanya Verifikator)
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn only_active_verifiers_may_create_companies() {
    let w = world().await;
    let (_, verifier_sub) = admin(&w.app, &["verifier"], "active").await;
    let n = nib();
    let e = email();
    let payload = body(&n, &e);
    let expired = {
        let now = SystemClock.now().unix_timestamp();
        w.keys
            .admin
            .sign(&json!({
                "iss": ADMIN_ISS, "aud": ADMIN_AUD, "sub": verifier_sub, "typ": "Bearer",
                "sid": "sesi-uji", "azp": "rs-backoffice-bff", "iat": now - 900, "exp": now - 600,
            }))
            .unwrap()
    };
    // 1-2. Tanpa token, token rusak/kedaluwarsa, realm lain (pasien dan HR korporat).
    for auth in [
        String::new(),
        "Authorization: Bearer bukan.token.sah".to_string(),
        format!("Authorization: Bearer {expired}"),
        format!(
            "Authorization: Bearer {}",
            token(&w.keys.pasien, PASIEN_ISS, PASIEN_AUD, &verifier_sub)
        ),
        format!(
            "Authorization: Bearer {}",
            token(&w.keys.korporat, KORPORAT_ISS, KORPORAT_AUD, &verifier_sub)
        ),
    ] {
        let extra: Vec<&str> = if auth.is_empty() { vec![] } else { vec![&auth] };
        let r = common::request_json(w.addr, "POST", CREATE, &extra, &payload).await;
        assert_eq!(r.status, 401, "{auth}: {}", r.body);
    }
    // 3. Akun nonaktif dengan token masih berlaku.
    let (_, inactive_sub) = admin(&w.app, &["verifier"], "inactive").await;
    let r = w.create(&inactive_sub, &payload).await;
    assert_eq!((r.status, code(&r)), (401, "4010002".to_string()));
    // 4. Sub-peran lain termasuk Super Admin, dan admin tanpa sub-peran: 403.
    for roles in [
        vec!["super_admin"],
        vec!["customer_service"],
        vec!["finance"],
        vec!["content"],
        vec!["auditor"],
        vec!["privacy_officer"],
        vec![],
    ] {
        let (_, sub) = admin(&w.app, &roles, "active").await;
        let r = w.create(&sub, &payload).await;
        assert_eq!(
            (r.status, code(&r)),
            (403, "4030001".to_string()),
            "{roles:?}"
        );
    }
    assert_eq!(
        w.clients_with_nib(&n).await,
        0,
        "tidak ada perusahaan dibuat"
    );
    assert_eq!(
        w.accounts_with_email(&e).await,
        0,
        "tidak ada akun HR dibuat"
    );
    // Pembanding positif: Verifikator yang sama berhasil.
    let r = w.create(&verifier_sub, &payload).await;
    assert_eq!(r.status, 201, "{}", r.body);
}

// trace: US-1903 AC1, coding-standards §5.1 (validasi isian; badan rusak; tanpa efek)
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn invalid_input_is_rejected_before_any_effect() {
    let w = world().await;
    let (_, sub) = admin(&w.app, &["verifier"], "active").await;
    let n = nib();
    let e = email();
    let mut bad = body(&n, "bukan-email");
    bad["legal_name"] = json!("");
    bad["npwp"] = json!("x".repeat(65));
    bad["office_phone"] = json!("0".repeat(33));
    let r = w.create(&sub, &bad).await;
    assert_eq!(
        (r.status, code(&r)),
        (422, "4220001".to_string()),
        "{}",
        r.body
    );
    let fields: Vec<String> = r.json()["errors"]
        .as_array()
        .map(|a| {
            a.iter()
                .map(|f| f["field"].as_str().unwrap_or("").to_string())
                .collect()
        })
        .unwrap_or_default();
    for f in ["legal_name", "npwp", "office_phone", "hr_email"] {
        assert!(
            fields.iter().any(|x| x == f),
            "{f} tidak dilaporkan: {}",
            r.body
        );
    }
    assert!(!r.body.contains(&"x".repeat(65)), "NPWP tidak digemakan");
    // Field tak dikenal, HR berformat Unicode, badan rusak, dan jenis media salah.
    let mut extra = body(&n, &e);
    extra["status"] = json!("active");
    assert_eq!(w.create(&sub, &extra).await.status, 422);
    let mut unicode = body(&n, &e);
    unicode["hr_email"] = json!("h\u{fc}r@example.com");
    assert_eq!(w.create(&sub, &unicode).await.status, 422);
    let raw = common::request_with_body(
        w.addr,
        "POST",
        CREATE,
        &[&w.bearer(&sub), "Content-Type: application/json"],
        Some(b"{bukan json"),
    )
    .await;
    assert_eq!(raw.status, 400, "{}", raw.body);
    let media = common::request_with_body(
        w.addr,
        "POST",
        CREATE,
        &[&w.bearer(&sub), "Content-Type: text/plain"],
        Some(b"{}"),
    )
    .await;
    assert_eq!(media.status, 415, "{}", media.body);
    assert_eq!(w.clients_with_nib(&n).await, 0);
    assert_eq!(w.accounts_with_email(&e).await, 0);
}

// trace: ADR-030 butir 1 (KEPUTUSAN-302), TD-26/TD-30: proses api tidak memegang atau memakai klien Admin REST
// Keycloak. Pemeriksaan arsitektur menyeluruh ada di arch-check (`idp-admin-client`); di sini berkas use case
// F1-T105/F1-T107 dipastikan bebas dari klien itu.
#[test]
fn use_case_files_contain_no_keycloak_admin_client() {
    let root = common::workspace_root().join("apps/rs-backend/src/modules");
    let files = [
        "corporate/admin_create.rs",
        "corporate/admin_create_http.rs",
        "identity/corporate_hr_invite.rs",
        "identity/corporate_hr_service.rs",
        "identity/corporate_hr_http.rs",
    ];
    for f in files {
        let text = std::fs::read_to_string(root.join(f)).unwrap();
        // Hanya kode non-test; komentar dokumentasi boleh menyebut nama klien.
        let code: String = text
            .lines()
            .filter(|l| !l.trim_start().starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n");
        for forbidden in [
            "KeycloakClient",
            "IdpUserAdmin",
            "KeycloakAdminUsers",
            "execute-actions-email",
            "/admin/realms",
            "reqwest",
        ] {
            assert!(!code.contains(forbidden), "{f} memuat {forbidden}");
        }
    }
}
