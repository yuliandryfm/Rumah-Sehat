//! Penolakan sesi seketika lewat HTTP sungguhan (F1-T33, T1): resolver akun dan penjaga sesi modul
//! `identity` dipasang di middleware penegak `authz`, dengan PostgreSQL dari `scripts/db-ephemeral.sh`
//! (role runtime `rs_api`) dan Valkey 9.1.3 efemeral dari image `infra/local/compose.yaml`. Tanpa database
//! atau Docker, test GAGAL.
//!
//! Skenario: KU-10 (akun nonaktif → `4010002` pada permintaan pertama setelah aksi), US-103 AC5 (sesi
//! dicabut → `4010001`), perubahan peran, RT-18 (Valkey mati, beku, atau kehilangan isi → PostgreSQL),
//! PostgreSQL dan Valkey sama-sama mati → `503` (gagal-tertutup).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)] // berkas test

mod valkey_support;

use authn::testing::{StaticJwks, TestRealmKey, jwks_document};
use authn::{
    Algorithm, AuthContext, AuthState, Authenticator, AuthnConfig, Backend, ErrorRenderer, Realm,
    RealmConfig,
};
use authz::{AdminSubRole, DenialSink, Enforcement, Policy, PolicyRouter, Principal, SessionGuard};
use axum::http::header::CONTENT_TYPE;
use axum::response::{IntoResponse, Response};
use envelope::{ApiError, CommonError};
use foundation::{Clock, CorrelationId, FixedClock, SystemClock, Uuid, new_id};
use observability::EnvSource;
use rs_backend::modules::identity::api::{AccountDirectory, IdentityError, RevocationReason};
use serde_json::{Value, json};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use time::OffsetDateTime;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use utoipa_axum::routes;
use valkey::{Valkey, ValkeyConfig};
use valkey_support::{ValkeyContainer, dead_url};

const OP: Duration = Duration::from_millis(250);

fn iss(realm: &str) -> String {
    format!("https://auth.ruang-sehat.invalid/realms/{realm}")
}

async fn app_pool() -> PgPool {
    let url = EnvSource::from_process()
        .required_secret("RS_TEST_DB_APP_URL")
        .unwrap_or_else(|_| {
            panic!(
                "RS_TEST_DB_APP_URL tidak diset: test identitas wajib dijalankan lewat \
                 `bash scripts/db-ephemeral.sh cargo test ...` (dipakai scripts/verify.sh)"
            )
        });
    PgPoolOptions::new()
        .max_connections(4)
        .connect(url.expose())
        .await
        .unwrap_or_else(|_| panic!("koneksi RS_TEST_DB_APP_URL gagal"))
}

fn random_hex(n: usize) -> String {
    let mut b = vec![0u8; n];
    crypto::fill_random(&mut b).unwrap();
    b.iter().map(|x| format!("{x:02x}")).collect()
}

/// Akun sintetis; email domain contoh RFC 2606.
async fn new_account(
    pool: &PgPool,
    account_type: &str,
    realm: &str,
    status: &str,
) -> (Uuid, String) {
    let id = new_id();
    let subject = format!("uji-{}", random_hex(8));
    let email = format!("{subject}@example.com");
    let full_name = (realm == "admin").then(|| "Admin Uji".to_string());
    sqlx::query!(
        "INSERT INTO app.account (id, account_type, idp_realm, idp_subject, email, status, status_changed_at, full_name) \
         VALUES ($1, $2, $3, $4, $5, $6, now(), $7)",
        id,
        account_type,
        realm,
        subject,
        email,
        status,
        full_name,
    )
    .execute(pool)
    .await
    .unwrap();
    (id, subject)
}

async fn set_status(pool: &PgPool, id: Uuid, status: &str) {
    sqlx::query!(
        "UPDATE app.account SET status = $2, status_changed_at = now(), updated_at = now() WHERE id = $1",
        id,
        status
    )
    .execute(pool)
    .await
    .unwrap();
}

async fn add_role(pool: &PgPool, account_id: Uuid, role: &str) -> Uuid {
    let id = new_id();
    sqlx::query!(
        "INSERT INTO app.rs_admin_role (id, account_id, sub_role) VALUES ($1, $2, $3)",
        id,
        account_id,
        role
    )
    .execute(pool)
    .await
    .unwrap();
    id
}

async fn remove_role(pool: &PgPool, role_id: Uuid) {
    sqlx::query!(
        "UPDATE app.rs_admin_role SET deleted_at = now(), delete_reason = 'uji perubahan peran' WHERE id = $1",
        role_id
    )
    .execute(pool)
    .await
    .unwrap();
}

async fn revocation_rows(pool: &PgPool, sid: &str) -> i64 {
    sqlx::query_scalar!(
        r#"SELECT count(*) AS "n!" FROM app.session_revocation WHERE idp_session_id = $1"#,
        sid
    )
    .fetch_one(pool)
    .await
    .unwrap()
}

// ----- HTTP -----

struct Renderer;

impl ErrorRenderer for Renderer {
    fn render(&self, error: CommonError, correlation_id: CorrelationId) -> Response {
        let body = error.to_error_body(correlation_id, "https://d.example/problems");
        (
            error.http_status(),
            [(CONTENT_TYPE, "application/problem+json")],
            serde_json::to_vec(&body).unwrap(),
        )
            .into_response()
    }
}

#[utoipa::path(get, path = "/patient/v1/me", responses((status = 200)))]
async fn me(p: Principal) -> String {
    p.account_id().to_string()
}

#[utoipa::path(get, path = "/backoffice/v1/verifications", responses((status = 200)))]
async fn verifications(p: Principal) -> String {
    p.account_id().to_string()
}

struct Keys {
    pasien: TestRealmKey,
    admin: TestRealmKey,
}

fn keys() -> Keys {
    Keys {
        pasien: TestRealmKey::generate("pasien-k").unwrap(),
        admin: TestRealmKey::generate("admin-k").unwrap(),
    }
}

fn now() -> OffsetDateTime {
    SystemClock.now()
}

fn authenticator(keys: &Keys) -> Authenticator {
    let cfg = |realm: Realm, key: &TestRealmKey| RealmConfig {
        realm,
        issuer: iss(realm.as_str()),
        audiences: vec![format!("rs-api-{}", realm.as_str())],
        algorithms: vec![Algorithm::RS256],
        authorized_parties: vec![],
        jwks: Arc::new(StaticJwks::new(jwks_document(&[key.jwk()]))),
    };
    Authenticator::new(
        AuthnConfig::with_defaults(
            Backend::RuangSehat,
            vec![
                cfg(Realm::Pasien, &keys.pasien),
                cfg(Realm::Admin, &keys.admin),
            ],
        ),
        Arc::new(FixedClock::new(now())),
    )
    .unwrap()
}

fn token(key: &TestRealmKey, realm: &str, sub: &str, sid: &str) -> String {
    let t = now().unix_timestamp();
    key.sign(&json!({
        "iss": iss(realm), "aud": format!("rs-api-{realm}"), "sub": sub, "typ": "Bearer",
        "sid": sid, "azp": "app", "iat": t - 5, "exp": t + 300,
    }))
    .unwrap()
}

async fn context(keys: &Keys, realm: &str, sub: &str, sid: &str) -> AuthContext {
    let key = if realm == "admin" {
        &keys.admin
    } else {
        &keys.pasien
    };
    authenticator(keys)
        .verify(&token(key, realm, sub, sid))
        .await
        .unwrap()
}

fn directory(pool: PgPool, valkey: Valkey) -> Arc<AccountDirectory> {
    Arc::new(AccountDirectory::new(
        pool,
        valkey,
        Arc::new(SystemClock),
        Duration::from_secs(30),
    ))
}

async fn spawn(keys: &Keys, dir: Arc<AccountDirectory>) -> SocketAddr {
    let auth = AuthState::new(Arc::new(authenticator(keys)), Arc::new(Renderer));
    let (router, _) = PolicyRouter::<()>::new()
        .routes(Policy::patient(), routes!(me))
        .routes(
            Policy::rs_admin(&[AdminSubRole::Verifier]),
            routes!(verifications),
        )
        .finish(Enforcement::new(
            auth,
            dir.clone(),
            dir,
            DenialSink::start(64, 1, None).unwrap(),
        ))
        .unwrap();
    let app = router.layer(axum::middleware::from_fn(
        observability::correlation_middleware,
    ));
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { app.serve(listener, std::future::pending()).await.unwrap() });
    addr
}

struct Resp {
    status: u16,
    body: String,
}

impl Resp {
    fn code(&self) -> String {
        serde_json::from_str::<Value>(&self.body).unwrap_or(Value::Null)["response_code"]
            .as_str()
            .unwrap_or_default()
            .to_string()
    }
}

async fn get(addr: SocketAddr, path: &str, bearer: &str) -> Resp {
    let mut s = TcpStream::connect(addr).await.unwrap();
    let req = format!(
        "GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\nAuthorization: Bearer {bearer}\r\n\r\n"
    );
    s.write_all(req.as_bytes()).await.unwrap();
    let mut buf = Vec::new();
    s.read_to_end(&mut buf).await.unwrap();
    let text = String::from_utf8_lossy(&buf).into_owned();
    Resp {
        status: text.get(9..12).unwrap().parse().unwrap(),
        body: text
            .split("\r\n\r\n")
            .nth(1)
            .unwrap_or_default()
            .to_string(),
    }
}

fn dead_valkey() -> Valkey {
    Valkey::connect_lazy(&ValkeyConfig {
        url: dead_url(),
        op_timeout: OP,
    })
    .unwrap()
}

/// Pencabutan sesi seperti use case keluar: catat di transaksi sendiri, commit, lalu salin ke Valkey
/// (tanpa event audit; use case sungguhan menjalankannya di `audit::in_audited_tx`).
async fn revoke(
    pool: &PgPool,
    dir: &AccountDirectory,
    account_id: Uuid,
    sid: &str,
    reason: RevocationReason,
    expires_at: OffsetDateTime,
) -> Result<(), IdentityError> {
    let mut tx = pool.begin().await.unwrap();
    let rec = dir
        .record_revocation(&mut tx, account_id, sid, reason, expires_at)
        .await?;
    tx.commit().await.unwrap();
    dir.publish_revocation(&rec).await
}

// ----- test -----

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn deactivated_account_is_rejected_on_first_request_after_refresh() {
    // trace: KU-10 (b), klarifikasi T-09 (definisi "berikutnya")
    let pool = app_pool().await;
    let vk = ValkeyContainer::start();
    let keys = keys();
    let dir = directory(pool.clone(), vk.client(OP));
    let addr = spawn(&keys, dir.clone()).await;
    let (id, sub) = new_account(&pool, "patient", "pasien", "active").await;
    let t = token(
        &keys.pasien,
        "pasien",
        &sub,
        &format!("sid-{}", random_hex(6)),
    );

    let r = get(addr, "/patient/v1/me", &t).await;
    assert_eq!((r.status, r.body.as_str()), (200, id.to_string().as_str()));
    // Entri akun tersimpan di Valkey (hanya ID dan kode).
    let cached = vk.cli(&["GET", &format!("rs:account:pasien:{sub}")]);
    assert!(
        cached.contains(&id.to_string()) && cached.contains("\"active\""),
        "{cached}"
    );
    assert!(!cached.contains("example.com"));

    // Aksi penonaktifan: status di database lalu penyegaran cache (langkah 1 dan 2 ADR-011 butir 8).
    set_status(&pool, id, "inactive").await;
    dir.refresh_account(Realm::Pasien, &sub).await.unwrap();
    let r = get(addr, "/patient/v1/me", &t).await;
    assert_eq!(r.status, 401);
    assert_eq!(r.code(), "4010002");

    // Akun belum aktif → 4010001 (bukan layar "Akun nonaktif").
    set_status(&pool, id, "pending_activation").await;
    dir.refresh_account(Realm::Pasien, &sub).await.unwrap();
    let r = get(addr, "/patient/v1/me", &t).await;
    assert_eq!((r.status, r.code().as_str()), (401, "4010001"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn revoked_session_is_rejected_on_next_request_and_other_sessions_continue() {
    // trace: US-103 AC5, ADR-011 butir 8
    let pool = app_pool().await;
    let vk = ValkeyContainer::start();
    let keys = keys();
    let dir = directory(pool.clone(), vk.client(OP));
    dir.rebuild_revocation_cache().await.unwrap();
    let addr = spawn(&keys, dir.clone()).await;
    let (id, sub) = new_account(&pool, "patient", "pasien", "active").await;
    let sid_a = format!("sid-{}", random_hex(6));
    let sid_b = format!("sid-{}", random_hex(6));
    let ta = token(&keys.pasien, "pasien", &sub, &sid_a);
    let tb = token(&keys.pasien, "pasien", &sub, &sid_b);
    assert_eq!(get(addr, "/patient/v1/me", &ta).await.status, 200);

    revoke(
        &pool,
        &dir,
        id,
        &sid_a,
        RevocationReason::Logout,
        now() + time::Duration::minutes(10),
    )
    .await
    .unwrap();
    let r = get(addr, "/patient/v1/me", &ta).await;
    assert_eq!((r.status, r.code().as_str()), (401, "4010001"));
    assert_eq!(get(addr, "/patient/v1/me", &tb).await.status, 200);
    assert_eq!(revocation_rows(&pool, &sid_a).await, 1);
    // Salinan Valkey ber-TTL sampai pencabutan berakhir.
    let pttl: i64 = vk
        .cli(&["PTTL", &format!("rs:session-revoked:sid:{sid_a}")])
        .trim()
        .parse()
        .unwrap();
    assert!(pttl > 9 * 60 * 1000 && pttl <= 10 * 60 * 1000, "{pttl}");

    // Akun nonaktif yang sesinya juga dicabut tetap melihat layar "Akun nonaktif".
    set_status(&pool, id, "inactive").await;
    dir.refresh_account(Realm::Pasien, &sub).await.unwrap();
    assert_eq!(get(addr, "/patient/v1/me", &ta).await.code(), "4010002");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn role_change_takes_effect_after_refresh() {
    // trace: KU-10 (perubahan peran, US-1302 AC4 definisi KEPUTUSAN-193)
    let pool = app_pool().await;
    let vk = ValkeyContainer::start();
    let keys = keys();
    let dir = directory(pool.clone(), vk.client(OP));
    let addr = spawn(&keys, dir.clone()).await;
    let (id, sub) = new_account(&pool, "rs_admin", "admin", "active").await;
    let role = add_role(&pool, id, "verifier").await;
    let t = token(
        &keys.admin,
        "admin",
        &sub,
        &format!("sid-{}", random_hex(6)),
    );
    assert_eq!(
        get(addr, "/backoffice/v1/verifications", &t).await.status,
        200
    );
    remove_role(&pool, role).await;
    add_role(&pool, id, "content").await;
    dir.refresh_account(Realm::Admin, &sub).await.unwrap();
    let r = get(addr, "/backoffice/v1/verifications", &t).await;
    assert_eq!((r.status, r.code().as_str()), (403, "4030001"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn revocation_survives_valkey_content_loss_and_cache_is_rebuilt() {
    // trace: RT-18, ADR-011 butir 8 ("pencabutan tetap berlaku walaupun isi Redis hilang")
    let pool = app_pool().await;
    let vk = ValkeyContainer::start();
    let keys = keys();
    let dir = directory(pool.clone(), vk.client(OP));
    dir.rebuild_revocation_cache().await.unwrap();
    let addr = spawn(&keys, dir.clone()).await;
    let (id, sub) = new_account(&pool, "patient", "pasien", "active").await;
    let sid = format!("sid-{}", random_hex(6));
    let t = token(&keys.pasien, "pasien", &sub, &sid);
    revoke(
        &pool,
        &dir,
        id,
        &sid,
        RevocationReason::AdminRevoke,
        now() + time::Duration::minutes(10),
    )
    .await
    .unwrap();

    vk.flush_all(); // setara restart Valkey tanpa persistensi
    let r = get(addr, "/patient/v1/me", &t).await;
    assert_eq!((r.status, r.code().as_str()), (401, "4010001"));
    // Penanda dan salinan dibangun ulang di latar belakang.
    let mut rebuilt = false;
    for _ in 0..50 {
        if vk.cli(&["GET", "rs:session-revoked:loaded"]).trim() == "1"
            && vk
                .cli(&["EXISTS", &format!("rs:session-revoked:sid:{sid}")])
                .trim()
                == "1"
        {
            rebuilt = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    assert!(rebuilt, "salinan pencabutan tidak dibangun ulang");
    assert_eq!(get(addr, "/patient/v1/me", &t).await.status, 401);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn failed_cache_publish_forces_database_path() {
    // trace: RT-18; penyalinan gagal → penanda dihapus → PostgreSQL
    let pool = app_pool().await;
    let vk = ValkeyContainer::start();
    let keys = keys();
    let dir = directory(pool.clone(), vk.client(OP));
    dir.rebuild_revocation_cache().await.unwrap();
    let addr = spawn(&keys, dir.clone()).await;
    let (id, sub) = new_account(&pool, "patient", "pasien", "active").await;
    let sid = format!("sid-{}", random_hex(6));
    let t = token(&keys.pasien, "pasien", &sub, &sid);
    assert_eq!(get(addr, "/patient/v1/me", &t).await.status, 200);

    // Valkey beku saat penyalinan: pencatatan database tetap, penyalinan gagal dan dilaporkan.
    let mut tx = pool.begin().await.unwrap();
    let rec = dir
        .record_revocation(
            &mut tx,
            id,
            &sid,
            RevocationReason::Logout,
            now() + time::Duration::minutes(10),
        )
        .await
        .unwrap();
    tx.commit().await.unwrap();
    vk.pause();
    let published = dir.publish_revocation(&rec).await;
    vk.unpause();
    assert_eq!(published, Err(IdentityError::CacheUnavailable));
    // Proses ini langsung menolak lewat daftar tunggu, walaupun Valkey belum memuat sid itu.
    let r = get(addr, "/patient/v1/me", &t).await;
    assert_eq!((r.status, r.code().as_str()), (401, "4010001"));
    // Percobaan ulang di latar belakang menyalin pencabutan, sehingga proses API lain yang memakai Valkey
    // yang sama juga menolak.
    let mut copied = false;
    for _ in 0..50 {
        if vk
            .cli(&["EXISTS", &format!("rs:session-revoked:sid:{sid}")])
            .trim()
            == "1"
        {
            copied = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    assert!(copied, "penyalinan tidak dicoba ulang");
    let other = directory(pool.clone(), vk.client(OP));
    let addr_other = spawn(&keys, other).await;
    let r = get(addr_other, "/patient/v1/me", &t).await;
    assert_eq!((r.status, r.code().as_str()), (401, "4010001"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn valkey_down_or_frozen_falls_back_to_database() {
    // trace: RT-18 mitigasi 2 ("backend membaca status akun langsung dari PostgreSQL bila Redis tidak
    // tersedia")
    let pool = app_pool().await;
    let keys = keys();
    let dir = directory(pool.clone(), dead_valkey());
    let addr = spawn(&keys, dir.clone()).await;
    let (active, sub_active) = new_account(&pool, "patient", "pasien", "active").await;
    let (_, sub_inactive) = new_account(&pool, "patient", "pasien", "inactive").await;
    let sid = format!("sid-{}", random_hex(6));
    let t = token(&keys.pasien, "pasien", &sub_active, &sid);
    assert_eq!(get(addr, "/patient/v1/me", &t).await.status, 200);
    let ti = token(
        &keys.pasien,
        "pasien",
        &sub_inactive,
        &format!("sid-{}", random_hex(6)),
    );
    assert_eq!(get(addr, "/patient/v1/me", &ti).await.code(), "4010002");
    // Pencabutan tercatat walaupun penyalinan ke Valkey gagal; pemeriksaan lewat PostgreSQL.
    assert_eq!(
        revoke(
            &pool,
            &dir,
            active,
            &sid,
            RevocationReason::Logout,
            now() + time::Duration::minutes(10)
        )
        .await,
        Err(IdentityError::CacheUnavailable)
    );
    assert_eq!(revocation_rows(&pool, &sid).await, 1);
    assert_eq!(get(addr, "/patient/v1/me", &t).await.code(), "4010001");

    // Valkey beku: tetap dijawab dalam batas waktu operasi, dengan hasil yang benar.
    let vk = ValkeyContainer::start();
    let dir2 = directory(pool.clone(), vk.client(OP));
    dir2.rebuild_revocation_cache().await.unwrap();
    let addr2 = spawn(&keys, dir2).await;
    vk.pause();
    let started = Instant::now();
    let r = get(addr2, "/patient/v1/me", &t).await;
    let elapsed = started.elapsed();
    vk.unpause();
    assert_eq!(r.code(), "4010001");
    assert!(elapsed < Duration::from_millis(1500), "{elapsed:?}");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn database_and_valkey_both_down_fail_closed_with_503() {
    let keys = keys();
    let dead_pool = PgPoolOptions::new()
        .acquire_timeout(Duration::from_millis(500))
        .connect_lazy("postgres://rs_api:x@127.0.0.1:1/ruang_sehat?sslmode=disable")
        .unwrap();
    let dir = directory(dead_pool, dead_valkey());
    let addr = spawn(&keys, dir).await;
    let r = get(
        addr,
        "/patient/v1/me",
        &token(&keys.pasien, "pasien", "s", "sid"),
    )
    .await;
    assert_eq!((r.status, r.code().as_str()), (503, "5030001"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn revocation_records_validate_keep_longest_expiry_and_are_purged() {
    let pool = app_pool().await;
    let keys = keys();
    let (id, sub) = new_account(&pool, "patient", "pasien", "active").await;
    let base = now();
    let clock = Arc::new(FixedClock::new(base));
    let dir = AccountDirectory::new(
        pool.clone(),
        dead_valkey(),
        clock.clone(),
        Duration::from_secs(30),
    );
    let sid = format!("sid-{}", random_hex(6));
    let mut conn = pool.acquire().await.unwrap();
    // Tanpa ID sesi atau kedaluwarsa yang sudah lewat → ditolak.
    for (s, exp) in [
        ("", base + time::Duration::minutes(1)),
        (sid.as_str(), base),
    ] {
        assert_eq!(
            dir.record_revocation(&mut conn, id, s, RevocationReason::Logout, exp)
                .await
                .err(),
            Some(IdentityError::InvalidRevocation)
        );
    }
    // Pencatatan ulang tidak mempersingkat pencabutan.
    dir.record_revocation(
        &mut conn,
        id,
        &sid,
        RevocationReason::Logout,
        base + time::Duration::seconds(40),
    )
    .await
    .unwrap();
    dir.record_revocation(
        &mut conn,
        id,
        &sid,
        RevocationReason::AdminRevoke,
        base + time::Duration::seconds(20),
    )
    .await
    .unwrap();
    drop(conn);
    let ctx = context(&keys, "pasien", &sub, &sid).await;
    clock.advance(time::Duration::seconds(30));
    assert!(
        dir.is_revoked(&ctx).await.unwrap(),
        "pencabutan dipersingkat"
    );
    clock.advance(time::Duration::seconds(15));
    assert!(
        !dir.is_revoked(&ctx).await.unwrap(),
        "pencabutan tidak berakhir"
    );
    assert!(dir.purge_expired_revocations().await.unwrap() >= 1);
    assert_eq!(revocation_rows(&pool, &sid).await, 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn resolver_ignores_soft_deleted_accounts_and_corrupt_cache_values() {
    let pool = app_pool().await;
    let vk = ValkeyContainer::start();
    let keys = keys();
    let dir = directory(pool.clone(), vk.client(OP));
    let addr = spawn(&keys, dir.clone()).await;
    let (id, sub) = new_account(&pool, "patient", "pasien", "active").await;
    let t = token(
        &keys.pasien,
        "pasien",
        &sub,
        &format!("sid-{}", random_hex(6)),
    );
    // Nilai cache rusak tidak memberi hak dan dibaca ulang dari database.
    vk.cli(&[
        "SET",
        &format!("rs:account:pasien:{sub}"),
        "bukan-json",
        "PX",
        "30000",
    ]);
    assert_eq!(get(addr, "/patient/v1/me", &t).await.status, 200);
    // Akun terhapus (soft delete, ADR-003) → tidak dikenal.
    sqlx::query!(
        "UPDATE app.account SET deleted_at = now(), delete_reason = 'uji' WHERE id = $1",
        id
    )
    .execute(&pool)
    .await
    .unwrap();
    dir.refresh_account(Realm::Pasien, &sub).await.unwrap();
    assert_eq!(
        vk.cli(&["EXISTS", &format!("rs:account:pasien:{sub}")])
            .trim(),
        "0"
    );
    assert_eq!(get(addr, "/patient/v1/me", &t).await.code(), "4010001");
}
