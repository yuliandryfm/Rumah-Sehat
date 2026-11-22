//! F1-T107 (T1; US-1904 AC2, AC5; ADR-027 butir 3, ADR-028 butir 1, KEPUTUSAN-290): daftar dan penonaktifan
//! anggota HR perusahaan lewat proses `api` lengkap (router produksi, resolver akun dan pembaca keanggotaan
//! sungguhan) terhadap PostgreSQL 18 sekali pakai (`scripts/db-ephemeral.sh`, role runtime `rs_api`). Tanpa
//! variabel `RS_TEST_DB_*` test GAGAL.
//!
//! Isolasi lintas perusahaan untuk semua endpoint juga dijalankan harness `corporate_isolation.rs`; di sini
//! skenario bisnis dan negatif: HR lain dinonaktifkan dan sesi lamanya langsung ditolak, diri sendiri
//! ditolak `[ASUMSI]`, HR aktif terakhir tahan balapan (dua HR saling menonaktifkan bersamaan), keanggotaan
//! perusahaan lain/tidak ada `404`, lima skenario penolakan endpoint, dan isi event E-10/E-11.
//!
//! Undangan HR baru (kedaluwarsa, dipakai ulang) belum diuji: endpoint undangan belum dibuat karena penyedia
//! realm `korporat` (pekerja ADR-030) belum dipasang (lihat `corporate_hr_service.rs`).
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
use serde_json::Value;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use std::sync::Arc;

const LIST: &str = "/v1/corporate/members";

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
        format!("PT Uji HR {}", random_hex(4)),
        format!("NIB{}", random_hex(8)),
        format!("NPWP{}", random_hex(8)),
        status,
    )
    .execute(pool)
    .await
    .unwrap();
    id
}

/// Satu HR: akun realm `korporat` dan keanggotaannya.
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

impl World {
    fn bearer(&self, hr: &Hr) -> String {
        format!(
            "Authorization: Bearer {}",
            token(&self.keys.korporat, KORPORAT_ISS, KORPORAT_AUD, &hr.subject)
        )
    }

    async fn get(&self, hr: &Hr, path: &str) -> Resp {
        common::request(self.addr, "GET", path, &[&self.bearer(hr)]).await
    }

    async fn deactivate(&self, hr: &Hr, member: Uuid) -> Resp {
        common::request(
            self.addr,
            "POST",
            &deactivate_path(member),
            &[&self.bearer(hr)],
        )
        .await
    }

    async fn account_status(&self, hr: &Hr) -> String {
        sqlx::query_scalar!(
            "SELECT status FROM app.account WHERE id = $1",
            hr.account_id
        )
        .fetch_one(&self.app)
        .await
        .unwrap()
    }

    async fn member_active(&self, hr: &Hr) -> bool {
        sqlx::query_scalar!(
            r#"SELECT deleted_at IS NULL AS "active!" FROM app.corporate_member WHERE id = $1"#,
            hr.member_id
        )
        .fetch_one(&self.app)
        .await
        .unwrap()
    }

    /// Event audit untuk akun `id` (objek), urut waktu.
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

// trace: US-1904 AC2 (daftar HR perusahaan sendiri, termasuk undangan; tanpa data perusahaan lain)
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn list_shows_only_own_company_members_including_invites() {
    let w = world().await;
    let client = insert_client(&w.app, "active").await;
    let a = insert_hr(&w.app, client, "active").await;
    let invited = insert_hr(&w.app, client, "pending_activation").await;
    let other_client = insert_client(&w.app, "active").await;
    let foreign = insert_hr(&w.app, other_client, "active").await;

    let r = w.get(&a, LIST).await;
    assert_eq!(r.status, 200, "{}", r.body);
    assert_eq!(r.header("cache-control"), Some("no-store"));
    let data = r.json()["data"].as_array().unwrap().clone();
    assert_eq!(data.len(), 2, "{data:?}");
    let me = data
        .iter()
        .find(|m| m["corporate_member_id"] == a.member_id.to_string())
        .unwrap();
    assert_eq!(me["is_self"], true);
    assert_eq!(me["status"], "active");
    assert_eq!(me["email"], a.email);
    let inv = data
        .iter()
        .find(|m| m["corporate_member_id"] == invited.member_id.to_string())
        .unwrap();
    assert_eq!(inv["status"], "pending_activation");
    assert_eq!(inv["is_self"], false);
    assert!(!r.body.contains(&foreign.email));
    assert!(!r.body.contains(&foreign.member_id.to_string()));
}

// trace: US-1904 AC2, AC5, KU-10 (HR lain dinonaktifkan; sesi lamanya langsung ditolak; E-10/E-11)
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn deactivating_another_hr_revokes_access_immediately_and_is_audited() {
    let w = world().await;
    let client = insert_client(&w.app, "active").await;
    let a = insert_hr(&w.app, client, "active").await;
    let b = insert_hr(&w.app, client, "active").await;

    // Sesi B terbuka dan berfungsi sebelum dinonaktifkan.
    assert_eq!(w.get(&b, LIST).await.status, 200);

    let r = w.deactivate(&a, b.member_id).await;
    assert_eq!(r.status, 200, "{}", r.body);
    assert_eq!(r.header("cache-control"), Some("no-store"));
    let d = &r.json()["data"];
    assert_eq!(d["corporate_member_id"], b.member_id.to_string());
    assert_eq!(d["status"], "inactive");
    // Penyedia Keycloak belum dipasang (ADR-030): akses backend tetap tertutup.
    assert_eq!(d["identity_provider_synced"], false);

    assert_eq!(w.account_status(&b).await, "inactive");
    assert!(!w.member_active(&b).await);
    assert_eq!(w.account_status(&a).await, "active");

    // Token B yang masih berlaku ditolak pada permintaan berikutnya (layar "Akun nonaktif").
    for path in [LIST.to_string(), deactivate_path(a.member_id)] {
        let method = if path == LIST { "GET" } else { "POST" };
        let again = common::request(w.addr, method, &path, &[&w.bearer(&b)]).await;
        assert_eq!(again.status, 401, "{path}: {}", again.body);
        assert_eq!(code(&again), "4010002", "{path}");
    }
    // Pemanggil tetap dapat bekerja dan melihat B nonaktif hilang dari daftar keanggotaan aktif.
    let list = w.get(&a, LIST).await;
    assert_eq!(list.status, 200);
    assert!(!list.body.contains(&b.member_id.to_string()));

    let evs = w.events(b.account_id).await;
    let kinds: Vec<(&str, &str, &str)> = evs
        .iter()
        .map(|e| {
            (
                e["event_code"].as_str().unwrap(),
                e["action_name"].as_str().unwrap(),
                e["outcome"].as_str().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        kinds,
        vec![
            ("e_10", "corporate_admin_account_deactivated", "success"),
            ("e_11", "corporate_membership_revoked", "success"),
        ]
    );
    for e in &evs {
        assert_eq!(e["actor_account_id"], a.account_id.to_string());
        assert_eq!(e["actor_account_type"], "corporate_admin");
        assert_eq!(e["tenant_id"], client.to_string());
        assert_eq!(e["source_component"], "corporate_portal");
        assert!(e["source_ip"].as_str().is_some());
        let text = e.to_string();
        assert!(
            !text.contains(&b.email) && !text.contains(&a.email),
            "{text}"
        );
        assert!(!text.contains(&b.subject), "subjek Keycloak tidak dicatat");
    }
    audit_support::assert_events_complete(&evs, &["e_10", "e_11"]);
    assert_eq!(evs[0]["before_values"]["status"], "active");
    assert_eq!(evs[0]["after_values"]["status"], "inactive");
    assert_eq!(
        evs[1]["before_values"]["corporate_member_id"],
        b.member_id.to_string()
    );

    // Dinonaktifkan ulang: keanggotaan sudah dicabut = tidak ada.
    let r = w.deactivate(&a, b.member_id).await;
    assert_eq!((r.status, code(&r)), (404, "4040001".to_string()));
}

// trace: US-1904 AC2 (diri sendiri ditolak [ASUMSI]; tercatat E-10 hasil gagal; tanpa efek)
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn self_deactivation_is_rejected_without_side_effects() {
    let w = world().await;
    let client = insert_client(&w.app, "active").await;
    let a = insert_hr(&w.app, client, "active").await;
    let _b = insert_hr(&w.app, client, "active").await;

    let r = w.deactivate(&a, a.member_id).await;
    assert_eq!(
        (r.status, code(&r)),
        (409, "4090101".to_string()),
        "{}",
        r.body
    );
    assert_eq!(w.account_status(&a).await, "active");
    assert!(w.member_active(&a).await);
    let evs = w.events(a.account_id).await;
    assert_eq!(evs.len(), 1, "{evs:?}");
    audit_support::assert_event_complete(&evs[0], "e_10");
    assert_eq!(evs[0]["outcome"], "failure");
    assert_eq!(evs[0]["outcome_reason_code"], "self_deactivation");
    assert!(evs[0]["changed_fields"].is_null());
}

// trace: US-1904 AC2 (HR aktif satu-satunya: tidak dapat dinonaktifkan; undangan dapat dinonaktifkan)
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn only_active_hr_stays_and_invited_hr_can_be_deactivated() {
    let w = world().await;
    let client = insert_client(&w.app, "active").await;
    let a = insert_hr(&w.app, client, "active").await;
    let invited = insert_hr(&w.app, client, "pending_activation").await;

    let r = w.deactivate(&a, invited.member_id).await;
    assert_eq!(r.status, 200, "{}", r.body);
    assert_eq!(w.account_status(&invited).await, "inactive");
    // Satu-satunya HR aktif hanya dapat mencoba menonaktifkan dirinya sendiri: ditolak, tetap aktif.
    let r = w.deactivate(&a, a.member_id).await;
    assert_eq!(r.status, 409);
    assert_eq!(w.account_status(&a).await, "active");
}

// trace: US-1904 AC2, KEPUTUSAN-290 (dua HR saling menonaktifkan bersamaan: tersisa tepat satu HR aktif)
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_mutual_deactivation_leaves_exactly_one_active_hr() {
    let w = Arc::new(world().await);
    for _ in 0..5 {
        let client = insert_client(&w.app, "active").await;
        let a = insert_hr(&w.app, client, "active").await;
        let b = insert_hr(&w.app, client, "active").await;
        let (w1, w2) = (Arc::clone(&w), Arc::clone(&w));
        let (a1, b1, a2, b2) = (a.clone(), b.clone(), a.clone(), b.clone());
        let t1 = tokio::spawn(async move { w1.deactivate(&a1, b1.member_id).await });
        let t2 = tokio::spawn(async move { w2.deactivate(&b2, a2.member_id).await });
        let (r1, r2) = (t1.await.unwrap(), t2.await.unwrap());
        let ok = [&r1, &r2].iter().filter(|r| r.status == 200).count();
        assert_eq!(
            ok, 1,
            "{} {} / {} {}",
            r1.status, r1.body, r2.status, r2.body
        );
        let loser = if r1.status == 200 { &r2 } else { &r1 };
        assert_eq!(loser.status, 401, "{}", loser.body);
        let active: i64 = sqlx::query_scalar!(
            r#"SELECT count(*) AS "n!" FROM app.corporate_member m
               JOIN app.account a ON a.id = m.account_id
               WHERE m.corporate_client_id = $1 AND m.deleted_at IS NULL AND a.status = 'active'"#,
            client
        )
        .fetch_one(&w.app)
        .await
        .unwrap();
        assert_eq!(active, 1);
    }
}

// trace: US-1904 AC3 (keanggotaan perusahaan lain, ID acak, dan bukan UUID: 404 sama, tanpa efek)
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn foreign_unknown_or_malformed_member_is_not_found() {
    let w = world().await;
    let client = insert_client(&w.app, "active").await;
    let a = insert_hr(&w.app, client, "active").await;
    let other = insert_client(&w.app, "active").await;
    let victim = insert_hr(&w.app, other, "active").await;
    let _victim_peer = insert_hr(&w.app, other, "active").await;

    for path in [
        deactivate_path(victim.member_id),
        deactivate_path(new_id()),
        "/v1/corporate/members/bukan-uuid/deactivate".to_string(),
    ] {
        let r = common::request(w.addr, "POST", &path, &[&w.bearer(&a)]).await;
        assert_eq!((r.status, code(&r)), (404, "4040001".to_string()), "{path}");
        assert!(!r.body.contains(&victim.email));
    }
    assert_eq!(w.account_status(&victim).await, "active");
    assert!(w.member_active(&victim).await);
    assert!(w.events(victim.account_id).await.is_empty());
}

// trace: coding-standards §7.3 (lima skenario penolakan) untuk kedua endpoint
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn every_endpoint_rejects_missing_tokens_other_realms_inactive_and_unverified_companies() {
    let w = world().await;
    let client = insert_client(&w.app, "active").await;
    let a = insert_hr(&w.app, client, "active").await;
    let b = insert_hr(&w.app, client, "active").await;
    let endpoints = [
        ("GET", LIST.to_string()),
        ("POST", deactivate_path(b.member_id)),
    ];
    let pending_client = insert_client(&w.app, "pending_verification").await;
    let pending_hr = insert_hr(&w.app, pending_client, "active").await;
    let pending_peer = insert_hr(&w.app, pending_client, "active").await;
    let inactive_hr = insert_hr(&w.app, client, "inactive").await;

    let expired = {
        let now = SystemClock.now().unix_timestamp();
        w.keys
            .korporat
            .sign(&serde_json::json!({
                "iss": KORPORAT_ISS, "aud": KORPORAT_AUD, "sub": a.subject, "typ": "Bearer",
                "sid": "sesi-uji", "azp": "rs-korporat-bff", "iat": now - 900, "exp": now - 600,
            }))
            .unwrap()
    };
    for (method, path) in &endpoints {
        // 1-2. Tanpa token, token rusak/kedaluwarsa, realm lain.
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
            let r = common::request(w.addr, method, path, &extra).await;
            assert_eq!(r.status, 401, "{method} {path}: {}", r.body);
        }
        // 3. Akun nonaktif dengan token masih berlaku.
        let r = common::request(w.addr, method, path, &[&w.bearer(&inactive_hr)]).await;
        assert_eq!((r.status, code(&r)), (401, "4010002".to_string()));
        // 4. Perusahaan belum Aktif: hanya status verifikasi (US-1901 AC6).
        let p = if *method == "GET" {
            LIST.to_string()
        } else {
            deactivate_path(pending_peer.member_id)
        };
        let r = common::request(w.addr, method, &p, &[&w.bearer(&pending_hr)]).await;
        assert_eq!((r.status, code(&r)), (403, "4030001".to_string()), "{p}");
    }
    assert_eq!(w.account_status(&b).await, "active");
    assert_eq!(w.account_status(&pending_peer).await, "active");
}
