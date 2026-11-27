//! F1-T107 sisa (T1; US-1904 AC2 bagian undangan; ADR-030 butir 2, 5, 6, 7; KEPUTUSAN-303 P30-2, KEPUTUSAN-306,
//! KEPUTUSAN-329): HR aktif mengundang HR lain lewat `POST /v1/corporate/member-invitations` memakai proses
//! `api` lengkap (router produksi, resolver akun dan pembaca keanggotaan sungguhan) terhadap PostgreSQL 18
//! sekali pakai (`scripts/db-ephemeral.sh`, role runtime `rs_api`). Tanpa variabel `RS_TEST_DB_*` test GAGAL.
//!
//! Yang diuji: akun baru `pending_provisioning` terikat ke perusahaan PENGUNDANG (bukan dari isi permintaan),
//! satu niat `create_user` + satu job berisi ID niat saja, E-10/E-11 dengan tenant; jawaban `202` seragam
//! (isi sama) untuk email baru dan email yang sudah dipakai (nol efek, E-10 gagal `email_in_use`), termasuk email
//! HR perusahaan lain, undangan yang masih diproses, dan huruf besar/kecil; email di realm lain tidak bentrok;
//! balapan satu email = satu akun; lima skenario penolakan (token, realm lain, akun nonaktif, perusahaan belum
//! Aktif), HR yang baru dinonaktifkan tidak dapat mengundang, isian ID perusahaan ditolak, validasi email;
//! undangan tampil "diproses" di daftar dan tidak dapat dinonaktifkan sebelum pengguna dibuat.
//!
//! Kedaluwarsa dan sekali pakai tautan aktivasi: perilaku token aksi Keycloak, diuji di
//! `infra/keycloak/test/keycloak_check.py` (`cek_undangan_hr`). Pengiriman email aktivasi: worker IdP
//! (`identity_idp_worker.rs`).
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

const LIST: &str = "/v1/corporate/members";
const INVITE: &str = "/v1/corporate/member-invitations";

fn deactivate_path(member: Uuid) -> String {
    format!("/v1/corporate/members/{member}/deactivate")
}

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

async fn insert_client(pool: &PgPool, status: &str) -> Uuid {
    let id = new_id();
    sqlx::query!(
        "INSERT INTO app.corporate_client (id, legal_name, nib, npwp, office_address, office_phone, status, origin) \
         VALUES ($1, $2, $3, $4, 'Jl. Uji No. 1', '0210000000', $5, 'self_registered')",
        id,
        format!("PT Uji Undang {}", random_hex(4)),
        format!("NIB{}", random_hex(8)),
        format!("NPWP{}", random_hex(8)),
        status,
    )
    .execute(pool)
    .await
    .unwrap();
    id
}

#[derive(Clone)]
struct Hr {
    account_id: Uuid,
    member_id: Uuid,
    subject: String,
    email: String,
}

async fn insert_hr(pool: &PgPool, client_id: Uuid, status: &str) -> Hr {
    let account_id = new_id();
    let subject = format!("hr-{}", random_hex(8));
    let email = format!("{subject}@example.com");
    sqlx::query!(
        "INSERT INTO app.account (id, account_type, idp_realm, idp_subject, email, status, status_changed_at) \
         VALUES ($1, 'corporate_admin', 'korporat', $2, $3, $4, now())",
        account_id,
        subject,
        email,
        status,
    )
    .execute(pool)
    .await
    .unwrap();
    let member_id = new_id();
    sqlx::query!(
        "INSERT INTO app.corporate_member (id, account_id, corporate_client_id) VALUES ($1, $2, $3)",
        member_id,
        account_id,
        client_id,
    )
    .execute(pool)
    .await
    .unwrap();
    Hr {
        account_id,
        member_id,
        subject,
        email,
    }
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
            consent_text_limit: common::roomy_limiter(),
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

/// Jumlah baris (dibatasi pada perusahaan uji, karena test lain berjalan paralel di database yang sama) yang
/// tidak boleh berubah oleh undangan yang ditolak atau diseragamkan.
#[derive(Debug, PartialEq, Eq)]
struct Counts {
    members: i64,
    intents: i64,
    jobs: i64,
}

impl World {
    fn bearer(&self, hr: &Hr) -> String {
        format!(
            "Authorization: Bearer {}",
            token(&self.keys.korporat, KORPORAT_ISS, KORPORAT_AUD, &hr.subject)
        )
    }

    async fn invite(&self, hr: &Hr, email: &str) -> Resp {
        common::request_json(
            self.addr,
            "POST",
            INVITE,
            &[&self.bearer(hr)],
            &json!({ "email": email }),
        )
        .await
    }

    async fn get(&self, hr: &Hr, path: &str) -> Resp {
        common::request(self.addr, "GET", path, &[&self.bearer(hr)]).await
    }

    async fn counts(&self, clients: &[Uuid]) -> Counts {
        let r = sqlx::query!(
            r#"SELECT
                 (SELECT count(*) FROM app.corporate_member m
                   WHERE m.corporate_client_id = ANY($1)) AS "members!",
                 (SELECT count(*) FROM app.idp_provisioning_intent i
                   WHERE i.account_id IN (SELECT m.account_id FROM app.corporate_member m
                                           WHERE m.corporate_client_id = ANY($1))) AS "intents!",
                 (SELECT count(*) FROM app.job j
                   WHERE j.job_type = 'idp_create_user'
                     AND (j.payload ->> 'intent_id')::uuid IN (
                           SELECT i.id FROM app.idp_provisioning_intent i
                            WHERE i.account_id IN (SELECT m.account_id FROM app.corporate_member m
                                                    WHERE m.corporate_client_id = ANY($1)))) AS "jobs!""#,
            clients
        )
        .fetch_one(&self.app)
        .await
        .unwrap();
        Counts {
            members: r.members,
            intents: r.intents,
            jobs: r.jobs,
        }
    }

    async fn account_by_email(&self, email: &str) -> Vec<Uuid> {
        sqlx::query_scalar!(
            r#"SELECT id AS "id!" FROM app.account
               WHERE idp_realm = 'korporat' AND lower(email) = lower($1) AND deleted_at IS NULL"#,
            email
        )
        .fetch_all(&self.app)
        .await
        .unwrap()
    }

    /// Event audit untuk objek `id` (akun), urut waktu.
    async fn events(&self, id: Uuid) -> Vec<Value> {
        sqlx::query_scalar!(
            r#"SELECT payload AS "payload!" FROM app.audit_outbox
               WHERE payload ->> 'object_id' = $1 ORDER BY occurred_at, id"#,
            id.hyphenated().to_string(),
        )
        .fetch_all(&self.relay)
        .await
        .unwrap()
    }
}

fn code(r: &Resp) -> String {
    r.json()["response_code"].as_str().unwrap_or("").to_string()
}

fn new_email() -> String {
    format!("undang-{}@example.com", random_hex(6))
}

/// Isi respons tanpa `correlation_id` (berbeda per permintaan).
fn answer(r: &Resp) -> (u16, Value) {
    let mut v = r.json();
    if let Some(o) = v.as_object_mut() {
        o.remove("correlation_id");
    }
    (r.status, v)
}

// trace: US-1904 AC2, ADR-030 butir 2 dan 5 (akun terikat perusahaan pengundang; satu niat; satu job; E-10/E-11)
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn active_hr_invites_a_colleague_bound_to_its_own_company() {
    let w = world().await;
    let client = insert_client(&w.app, "active").await;
    let a = insert_hr(&w.app, client, "active").await;
    let other = insert_client(&w.app, "active").await;
    let foreign = insert_hr(&w.app, other, "active").await;
    let mixed = format!("Rekan.Baru-{}@Example.COM", random_hex(4));

    let r = w.invite(&a, &mixed).await;
    assert_eq!(r.status, 202, "{}", r.body);
    assert_eq!(r.header("cache-control"), Some("no-store"));
    assert_eq!(code(&r), "2020000");
    assert_eq!(r.json()["data"], json!({ "status": "processing" }));

    let ids = w.account_by_email(&mixed).await;
    assert_eq!(ids.len(), 1, "tepat satu akun baru");
    let invitee = ids[0];
    let acc = sqlx::query!(
        r#"SELECT account_type AS "t!", idp_realm AS "realm!", idp_subject, status AS "status!",
                  email AS "email!", created_by
           FROM app.account WHERE id = $1"#,
        invitee
    )
    .fetch_one(&w.app)
    .await
    .unwrap();
    assert_eq!(
        (acc.t.as_str(), acc.realm.as_str(), acc.status.as_str()),
        ("corporate_admin", "korporat", "pending_provisioning")
    );
    assert_eq!(acc.idp_subject, None, "Keycloak hanya disentuh worker");
    assert_eq!(acc.email, mixed.to_lowercase());
    assert_eq!(acc.created_by, Some(a.account_id));
    let member = sqlx::query!(
        r#"SELECT id AS "id!", corporate_client_id FROM app.corporate_member
           WHERE account_id = $1 AND deleted_at IS NULL"#,
        invitee
    )
    .fetch_one(&w.app)
    .await
    .unwrap();
    assert_eq!(
        member.corporate_client_id, client,
        "terikat perusahaan pengundang"
    );

    // Satu niat create_user (menunggu, pelaku HR) dan satu job berisi ID niat saja.
    let intents = sqlx::query!(
        r#"SELECT i.id AS "id!", i.operation AS "operation!", i.purpose AS "purpose!", s.status AS "status!"
           FROM app.idp_provisioning_intent i
           JOIN app.idp_provisioning_intent_state s ON s.idp_provisioning_intent_id = i.id
           WHERE i.account_id = $1"#,
        invitee
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
        ("create_user", "hr_invitation", "pending")
    );
    let actor_type = sqlx::query_scalar!(
        r#"SELECT actor_type AS "t!" FROM app.idp_provisioning_intent_status_event
           WHERE idp_provisioning_intent_id = $1"#,
        intents[0].id
    )
    .fetch_one(&w.app)
    .await
    .unwrap();
    assert_eq!(actor_type, "corporate_admin");
    let job = sqlx::query!(
        r#"SELECT job_type AS "job_type!", payload AS "payload!", status AS "status!"
           FROM app.job WHERE payload ->> 'intent_id' = $1"#,
        intents[0].id.hyphenated().to_string()
    )
    .fetch_all(&w.app)
    .await
    .unwrap();
    assert_eq!(job.len(), 1);
    assert_eq!(
        (job[0].job_type.as_str(), job[0].status.as_str()),
        ("idp_create_user", "queued")
    );
    assert_eq!(
        job[0].payload,
        json!({ "intent_id": intents[0].id.hyphenated().to_string() })
    );

    // Audit.
    let evs = w.events(invitee).await;
    let kinds: Vec<(&str, &str)> = evs
        .iter()
        .map(|e| {
            (
                e["event_code"].as_str().unwrap(),
                e["action_name"].as_str().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        kinds,
        vec![
            ("e_10", "corporate_admin_account_invited"),
            ("e_11", "corporate_membership_granted"),
        ]
    );
    audit_support::assert_events_complete(&evs, &["e_10", "e_11"]);
    // F1-T109: pemeriksa korporat bersama (tenant = perusahaan pengundang, tanpa email undangan).
    let tenant = client.to_string();
    audit_support::assert_corporate_events_clean(
        &evs,
        &["e_10", "e_11"],
        Some(&tenant),
        &[mixed.as_str(), a.email.as_str()],
    );
    for e in &evs {
        assert_eq!(e["actor_account_id"], a.account_id.to_string());
        assert_eq!(e["actor_account_type"], "corporate_admin");
        assert_eq!(e["tenant_id"], client.to_string());
        assert_eq!(e["source_component"], "corporate_portal");
        assert!(e["source_ip"].as_str().is_some());
        let text = e.to_string();
        assert!(
            !text.contains(&mixed.to_lowercase()) && !text.contains(&a.email),
            "{text}"
        );
    }
    assert_eq!(evs[0]["after_values"]["status"], "pending_provisioning");
    assert_eq!(
        evs[1]["after_values"]["corporate_member_id"],
        member.id.to_string()
    );

    // Daftar: undangan tampil "diproses" untuk pemanggil; perusahaan lain tidak melihatnya.
    let list = w.get(&a, LIST).await;
    assert_eq!(list.status, 200, "{}", list.body);
    let row = list.json()["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["corporate_member_id"] == member.id.to_string())
        .cloned()
        .expect("undangan tampil di daftar");
    assert_eq!(row["status"], "invitation_processing");
    assert_eq!(row["email"], mixed.to_lowercase());
    assert_eq!(row["is_self"], false);
    let foreign_list = w.get(&foreign, LIST).await;
    assert!(!foreign_list.body.contains(&mixed.to_lowercase()));
}

// trace: ADR-030 butir 7, P30-2 (KEPUTUSAN-303): email yang sudah dipakai dijawab SAMA dengan undangan baru,
// tanpa efek; tidak ada undang ulang yang menduplikasi akun atau niat
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn already_registered_email_gets_an_identical_answer_and_changes_nothing() {
    let w = world().await;
    let client = insert_client(&w.app, "active").await;
    let a = insert_hr(&w.app, client, "active").await;
    let other = insert_client(&w.app, "active").await;
    let foreign = insert_hr(&w.app, other, "active").await;
    let fresh = w.invite(&a, &new_email()).await;
    assert_eq!(fresh.status, 202);
    let reference = answer(&fresh);

    // Undangan yang masih diproses, HR rekan sendiri, HR perusahaan lain, dan varian huruf besar/spasi.
    let pending_email = new_email();
    assert_eq!(w.invite(&a, &pending_email).await.status, 202);
    let before = w.counts(&[client, other]).await;
    for tried in [
        pending_email.clone(),
        pending_email.to_uppercase(),
        format!(" {pending_email} "),
        a.email.clone(),
        foreign.email.clone(),
        foreign.email.to_uppercase(),
    ] {
        let r = w.invite(&a, &tried).await;
        assert_eq!(answer(&r), reference, "jawaban berbeda untuk {tried}");
    }
    assert_eq!(w.counts(&[client, other]).await, before, "tidak ada efek");
    assert_eq!(w.account_by_email(&pending_email).await.len(), 1);
    assert_eq!(
        w.account_by_email(&foreign.email).await,
        vec![foreign.account_id]
    );
    // Tercatat internal sebagai E-10 hasil gagal `email_in_use`, objek = akun pelaku; tanpa email.
    let failed: Vec<Value> = w
        .events(a.account_id)
        .await
        .into_iter()
        .filter(|e| e["outcome"] == "failure")
        .collect();
    assert_eq!(failed.len(), 6, "{failed:?}");
    let tenant = client.to_string();
    audit_support::assert_corporate_events_clean(
        &failed,
        &["e_10"],
        Some(&tenant),
        &[
            pending_email.as_str(),
            foreign.email.as_str(),
            a.email.as_str(),
        ],
    );
    for e in &failed {
        audit_support::assert_event_complete(e, "e_10");
        assert_eq!(e["action_name"], "corporate_admin_account_invited");
        assert_eq!(e["outcome_reason_code"], "email_in_use");
        let text = e.to_string();
        assert!(
            !text.contains(&foreign.email) && !text.contains(&pending_email),
            "{text}"
        );
    }
    // Perusahaan lain tidak mendapat anggota baru dari percobaan ini.
    let foreign_members = sqlx::query_scalar!(
        r#"SELECT count(*) AS "n!" FROM app.corporate_member WHERE corporate_client_id = $1"#,
        other
    )
    .fetch_one(&w.app)
    .await
    .unwrap();
    assert_eq!(foreign_members, 1);
}

// trace: ADR-027 butir 3 (keunikan email per realm): email akun di realm admin tidak menghalangi undangan HR
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn an_email_used_in_another_realm_does_not_clash() {
    let w = world().await;
    let client = insert_client(&w.app, "active").await;
    let a = insert_hr(&w.app, client, "active").await;
    let admin_email = format!("admin-{}@example.com", random_hex(6));
    sqlx::query!(
        "INSERT INTO app.account (id, account_type, idp_realm, idp_subject, email, status, status_changed_at, full_name) \
         VALUES ($1, 'rs_admin', 'admin', $2, $3, 'active', now(), 'Admin Sintetis')",
        new_id(),
        format!("adm-{}", random_hex(6)),
        admin_email,
    )
    .execute(&w.app)
    .await
    .unwrap();
    let r = w.invite(&a, &admin_email).await;
    assert_eq!(r.status, 202, "{}", r.body);
    assert_eq!(w.account_by_email(&admin_email).await.len(), 1);
}

// trace: ADR-030 butir 5 (kunci advisory per realm+email): dua undangan bersamaan ke satu email = satu akun
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_invitations_of_one_email_create_a_single_account() {
    let w = Arc::new(world().await);
    for _ in 0..5 {
        let c1 = insert_client(&w.app, "active").await;
        let c2 = insert_client(&w.app, "active").await;
        let a = insert_hr(&w.app, c1, "active").await;
        let b = insert_hr(&w.app, c2, "active").await;
        let email = new_email();
        let (w1, w2) = (Arc::clone(&w), Arc::clone(&w));
        let (e1, e2) = (email.clone(), email.clone());
        let t1 = tokio::spawn(async move { w1.invite(&a, &e1).await });
        let t2 = tokio::spawn(async move { w2.invite(&b, &e2).await });
        let (r1, r2) = (t1.await.unwrap(), t2.await.unwrap());
        assert_eq!(answer(&r1), answer(&r2), "{} / {}", r1.body, r2.body);
        assert_eq!(r1.status, 202);
        let ids = w.account_by_email(&email).await;
        assert_eq!(ids.len(), 1, "tepat satu akun");
        let members = sqlx::query_scalar!(
            r#"SELECT count(*) AS "n!" FROM app.corporate_member WHERE account_id = $1"#,
            ids[0]
        )
        .fetch_one(&w.app)
        .await
        .unwrap();
        assert_eq!(members, 1, "tepat satu keanggotaan");
        let intents = sqlx::query_scalar!(
            r#"SELECT count(*) AS "n!" FROM app.idp_provisioning_intent WHERE account_id = $1"#,
            ids[0]
        )
        .fetch_one(&w.app)
        .await
        .unwrap();
        assert_eq!(intents, 1, "tepat satu niat");
    }
}

// trace: coding-standards §7.3 (lima skenario penolakan), US-1904 AC3-AC5, US-1901 AC6
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn invitations_are_rejected_for_missing_tokens_other_realms_inactive_hr_and_unverified_companies()
 {
    let w = world().await;
    let client = insert_client(&w.app, "active").await;
    let a = insert_hr(&w.app, client, "active").await;
    let inactive = insert_hr(&w.app, client, "inactive").await;
    let pending_client = insert_client(&w.app, "pending_verification").await;
    let pending_hr = insert_hr(&w.app, pending_client, "active").await;
    let email = new_email();
    let expired = {
        let now = SystemClock.now().unix_timestamp();
        w.keys
            .korporat
            .sign(&json!({
                "iss": KORPORAT_ISS, "aud": KORPORAT_AUD, "sub": a.subject, "typ": "Bearer",
                "sid": "sesi-uji", "azp": "rs-korporat-bff", "iat": now - 900, "exp": now - 600,
            }))
            .unwrap()
    };
    let before = w.counts(&[client, pending_client]).await;
    for auth in [
        String::new(),
        "Authorization: Bearer bukan.token.sah".to_string(),
        format!("Authorization: Bearer {expired}"),
        format!(
            "Authorization: Bearer {}",
            token(&w.keys.pasien, PASIEN_ISS, PASIEN_AUD, &a.subject)
        ),
        format!(
            "Authorization: Bearer {}",
            token(&w.keys.admin, ADMIN_ISS, ADMIN_AUD, &a.subject)
        ),
    ] {
        let extra: Vec<&str> = if auth.is_empty() { vec![] } else { vec![&auth] };
        let r =
            common::request_json(w.addr, "POST", INVITE, &extra, &json!({ "email": email })).await;
        assert_eq!(r.status, 401, "{auth}: {}", r.body);
    }
    let r = w.invite(&inactive, &email).await;
    assert_eq!((r.status, code(&r)), (401, "4010002".to_string()));
    let r = w.invite(&pending_hr, &email).await;
    assert_eq!((r.status, code(&r)), (403, "4030001".to_string()));
    assert_eq!(
        w.counts(&[client, pending_client]).await,
        before,
        "tidak ada efek"
    );
    assert!(w.account_by_email(&email).await.is_empty());
}

// trace: US-1904 AC2, AC5, KU-10, KEPUTUSAN-290 (HR yang baru dinonaktifkan tidak dapat mengundang)
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_hr_deactivated_by_a_colleague_can_no_longer_invite() {
    let w = world().await;
    let client = insert_client(&w.app, "active").await;
    let a = insert_hr(&w.app, client, "active").await;
    let b = insert_hr(&w.app, client, "active").await;
    // B masih dapat mengundang sebelum dinonaktifkan.
    assert_eq!(w.invite(&b, &new_email()).await.status, 202);
    let d = common::request(
        w.addr,
        "POST",
        &deactivate_path(b.member_id),
        &[&w.bearer(&a)],
    )
    .await;
    assert_eq!(d.status, 200, "{}", d.body);
    let before = w.counts(&[client]).await;
    let email = new_email();
    let r = w.invite(&b, &email).await;
    assert_eq!(
        (r.status, code(&r)),
        (401, "4010002".to_string()),
        "{}",
        r.body
    );
    assert_eq!(w.counts(&[client]).await, before);
    assert!(w.account_by_email(&email).await.is_empty());
}

// trace: F1-T109 KEPUTUSAN-387 (prasyarat KEPUTUSAN-381): satu akun HR hanya pernah punya satu keanggotaan. HR yang
// sudah dinonaktifkan (keanggotaan dihapus lunak, akun tetap ada dengan email yang sama) tidak dapat diundang
// kembali, baik oleh rekan di perusahaan yang sama maupun oleh HR perusahaan lain: tidak ada keanggotaan kedua,
// sehingga tenant audit akun itu tidak bergantung pada waktu event. Test ini gagal bila perpindahan atau
// pengaktifan ulang keanggotaan dibuka tanpa menyelesaikan TD-52.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_deactivated_hr_cannot_gain_a_second_membership_through_an_invitation() {
    let w = world().await;
    let client = insert_client(&w.app, "active").await;
    let a = insert_hr(&w.app, client, "active").await;
    let b = insert_hr(&w.app, client, "active").await;
    let other = insert_client(&w.app, "active").await;
    let foreign = insert_hr(&w.app, other, "active").await;
    let d = common::request(
        w.addr,
        "POST",
        &deactivate_path(b.member_id),
        &[&w.bearer(&a)],
    )
    .await;
    assert_eq!(d.status, 200, "{}", d.body);
    let before = w.counts(&[client, other]).await;
    for (inviter, tried) in [
        (&a, b.email.clone()),
        (&a, b.email.to_uppercase()),
        (&foreign, b.email.clone()),
    ] {
        let r = w.invite(inviter, &tried).await;
        assert_eq!(r.status, 202, "jawaban seragam: {}", r.body);
    }
    assert_eq!(w.counts(&[client, other]).await, before, "tidak ada efek");
    let memberships = sqlx::query!(
        r#"SELECT corporate_client_id, deleted_at IS NOT NULL AS "revoked!"
           FROM app.corporate_member WHERE account_id = $1"#,
        b.account_id
    )
    .fetch_all(&w.app)
    .await
    .unwrap();
    assert_eq!(memberships.len(), 1, "satu akun HR, satu keanggotaan");
    assert_eq!(memberships[0].corporate_client_id, client);
    assert!(memberships[0].revoked);
    assert_eq!(w.account_by_email(&b.email).await, vec![b.account_id]);
}

// trace: US-1904 AC3 (cakupan perusahaan selalu dari keanggotaan; isian ID perusahaan tidak diterima), AC2
// (validasi email), coding-standards §5.1
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn invalid_bodies_and_injected_company_ids_are_rejected_without_effect() {
    let w = world().await;
    let client = insert_client(&w.app, "active").await;
    let a = insert_hr(&w.app, client, "active").await;
    let victim_client = insert_client(&w.app, "active").await;
    let before = w.counts(&[client, victim_client]).await;
    let auth = w.bearer(&a);
    for body in [
        json!({ "email": "bukan-email" }),
        json!({ "email": "" }),
        json!({ "email": "h\u{fc}r@example.com" }),
        json!({ "email": "a b@example.com" }),
        json!({ "email": format!("{}@example.com", "a".repeat(250)) }),
        json!({ "email": 42 }),
        json!({}),
        // Perusahaan sasaran tidak pernah diterima dari isi permintaan.
        json!({ "email": new_email(), "corporate_client_id": victim_client.to_string() }),
        json!({ "email": new_email(), "corporate_member_id": new_id().to_string() }),
    ] {
        let r = common::request_json(w.addr, "POST", INVITE, &[&auth], &body).await;
        assert_eq!(
            (r.status, code(&r)),
            (422, "4220001".to_string()),
            "{body}: {}",
            r.body
        );
    }
    let raw = common::request_with_body(
        w.addr,
        "POST",
        INVITE,
        &[&auth, "Content-Type: application/json"],
        Some(b"{bukan json"),
    )
    .await;
    assert_eq!(raw.status, 400, "{}", raw.body);
    let media = common::request_with_body(
        w.addr,
        "POST",
        INVITE,
        &[&auth, "Content-Type: text/plain"],
        Some(b"{}"),
    )
    .await;
    assert_eq!(media.status, 415, "{}", media.body);
    assert_eq!(
        w.counts(&[client, victim_client]).await,
        before,
        "tidak ada efek"
    );
    let members_of_victim = sqlx::query_scalar!(
        r#"SELECT count(*) AS "n!" FROM app.corporate_member WHERE corporate_client_id = $1"#,
        victim_client
    )
    .fetch_one(&w.app)
    .await
    .unwrap();
    assert_eq!(members_of_victim, 0);
}

// trace: ADR-030 butir 5, US-1904 AC2 (undangan yang penggunanya belum dibuat tidak dapat dinonaktifkan dan tidak
// dihitung HR aktif; setelah worker mengadopsi pengguna, dapat dinonaktifkan seperti undangan biasa)
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn provisioning_invitee_cannot_be_deactivated_until_the_worker_adopted_the_user() {
    let w = world().await;
    let client = insert_client(&w.app, "active").await;
    let a = insert_hr(&w.app, client, "active").await;
    let email = new_email();
    assert_eq!(w.invite(&a, &email).await.status, 202);
    let invitee = w.account_by_email(&email).await[0];
    let member = sqlx::query_scalar!(
        r#"SELECT id AS "id!" FROM app.corporate_member WHERE account_id = $1"#,
        invitee
    )
    .fetch_one(&w.app)
    .await
    .unwrap();

    let r = common::request(w.addr, "POST", &deactivate_path(member), &[&w.bearer(&a)]).await;
    assert_eq!(
        (r.status, code(&r)),
        (409, "4090001".to_string()),
        "{}",
        r.body
    );
    let status = sqlx::query_scalar!(
        r#"SELECT status AS "s!" FROM app.account WHERE id = $1"#,
        invitee
    )
    .fetch_one(&w.app)
    .await
    .unwrap();
    assert_eq!(status, "pending_provisioning");
    let failed: Vec<Value> = w
        .events(invitee)
        .await
        .into_iter()
        .filter(|e| e["outcome"] == "failure")
        .collect();
    assert_eq!(failed.len(), 1, "{failed:?}");
    assert_eq!(failed[0]["outcome_reason_code"], "target_provisioning");

    // Worker mengadopsi pengguna Keycloak (disimulasikan): menunggu aktivasi, lalu dapat dinonaktifkan.
    sqlx::query!(
        "UPDATE app.account SET idp_subject = $2, status = 'pending_activation', status_changed_at = now() WHERE id = $1",
        invitee,
        format!("kc-{}", random_hex(6)),
    )
    .execute(&w.app)
    .await
    .unwrap();
    let list = w.get(&a, LIST).await;
    let row = list.json()["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|m| m["corporate_member_id"] == member.to_string())
        .cloned()
        .unwrap();
    assert_eq!(row["status"], "pending_activation");
    let r = common::request(w.addr, "POST", &deactivate_path(member), &[&w.bearer(&a)]).await;
    assert_eq!(r.status, 200, "{}", r.body);
}
