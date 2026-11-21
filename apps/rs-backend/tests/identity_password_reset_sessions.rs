//! TD-27 (T1; US-103 AC4d, KEPUTUSAN-198, ADR-011 butir 8 dan 10): setelah password diatur ulang, sesi lama
//! pengguna ditolak backend.
//!
//! Rantai yang diuji ujung ke ujung (selain Keycloak sungguhan, yang diuji `scripts/keycloak-check.sh`):
//! event `LOGOUT` per sesi yang dipancarkan Keycloak saat atur ulang (formulir tema mengirim
//! `logout-sessions=on`, SRC-770) → pengambil event `rs-backend worker` (F1-T34) → `session_revocation` +
//! salinan Valkey (F1-T33) → permintaan API berikutnya dengan token akses sesi lama yang belum kedaluwarsa
//! dijawab `4010001`. Pengambil berjalan tiap 60 detik (bawaan), sehingga penolakan terjadi paling lambat satu
//! putaran setelah atur ulang, di dalam batas AC4d 2 menit.
//!
//! Skenario negatif: sesi akun lain tetap jalan; event dibaca ulang (tumpang-tindih) dan event duplikat tidak
//! menambah baris maupun event audit; Valkey mati → pencabutan tetap dari PostgreSQL; pencabutan yang sudah
//! commit tetapi belum tersalin sebelum worker berhenti tersalin saat worker mulai lagi (TD-24 jalur ini);
//! event LOGOUT gagal, tanpa sesi, akun tidak dikenal, atau lebih tua dari umur token tidak mencabut apa pun;
//! payload audit tanpa ID sesi.
//!
//! Keycloak di sini adalah tiruan Admin REST kecil di 127.0.0.1 dengan semantik parameter yang dipakai pengambil
//! (`type`, `dateFrom`, `dateTo` inklusif dalam milidetik, `direction=asc`, `max`; SRC-620) dan bentuk event
//! LOGOUT dari `AuthenticatorUtil.logoutOtherSessions` 26.8.2 (`sessionId`, `userId`, detail
//! `logout_triggered_by_required_action`, SRC-770). PostgreSQL dari `scripts/db-ephemeral.sh`, Valkey 9.1.3
//! efemeral dari image compose lokal; tanpa keduanya test GAGAL.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

mod valkey_support;

use authn::testing::{StaticJwks, TestRealmKey, jwks_document};
use authn::{
    Algorithm, AuthState, Authenticator, AuthnConfig, Backend, ErrorRenderer, Realm, RealmConfig,
};
use authz::{DenialSink, Enforcement, Policy, PolicyRouter, Principal};
use axum::Router;
use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::header::CONTENT_TYPE;
use axum::http::{HeaderMap, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use axum::routing::{get as get_route, post};
use envelope::{ApiError, CommonError};
use foundation::{Clock, CorrelationId, Sensitive, SystemClock, Uuid, new_id};
use observability::EnvSource;
use platform_jobs::lock::RS_KEYCLOAK_EVENT_PULLER;
use platform_jobs::{LockAttempt, SingletonLock};
use rs_backend::modules::identity::api::{
    AccountDirectory, IdpRealm, KeycloakAddress, KeycloakClient, LOGOUT_REVOCATION_TTL,
    PullOptions, Puller, PullerSettings, RealmCredential, RevocationReason, run_puller,
};
use serde_json::{Value, json};
use sqlx::postgres::PgPoolOptions;
use sqlx::{Connection, PgConnection, PgPool};
use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use time::OffsetDateTime;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use utoipa_axum::routes;
use valkey::{Valkey, ValkeyConfig};
use valkey_support::{ValkeyContainer, dead_url};

/// Test berbagi kunci pengambil (7001002), posisi baca, dan tanda terima: dijalankan satu per satu.
static SERIAL: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

const OP: Duration = Duration::from_millis(250);
const CLIENT_ID: &str = "rs-backend-event-reader";
const SECRET: &str = "rahasia-uji-pasien-td27-0123456789abcdef";

// ------------------------------------------------------------------------------------ tiruan Keycloak

#[derive(Default)]
struct FakeKc {
    events: Mutex<Vec<Value>>,
    tokens: Mutex<HashSet<String>>,
}

type Fake = Arc<FakeKc>;

fn kc_json(status: StatusCode, v: Value) -> Response {
    (
        status,
        [("content-type", "application/json")],
        v.to_string(),
    )
        .into_response()
}

fn pairs(s: &str) -> Vec<(String, String)> {
    s.split('&')
        .filter(|p| !p.is_empty())
        .map(|p| {
            let (k, v) = p.split_once('=').unwrap_or((p, ""));
            (k.to_string(), v.to_string())
        })
        .collect()
}

async fn token(State(kc): State<Fake>, Path(realm): Path<String>, body: Bytes) -> Response {
    let form: HashMap<String, String> = pairs(std::str::from_utf8(&body).unwrap())
        .into_iter()
        .collect();
    let ok = realm == "pasien"
        && form.get("grant_type").map(String::as_str) == Some("client_credentials")
        && form.get("client_id").map(String::as_str) == Some(CLIENT_ID)
        && form.get("client_secret").map(String::as_str) == Some(SECRET);
    if !ok {
        return kc_json(
            StatusCode::UNAUTHORIZED,
            json!({"error": "unauthorized_client"}),
        );
    }
    let t = format!("pasien.{}", new_id());
    kc.tokens.lock().unwrap().insert(t.clone());
    kc_json(
        StatusCode::OK,
        json!({"access_token": t, "expires_in": 300, "token_type": "Bearer"}),
    )
}

async fn events(
    State(kc): State<Fake>,
    Path(realm): Path<String>,
    headers: HeaderMap,
    uri: Uri,
) -> Response {
    let bearer = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .unwrap_or("");
    if realm != "pasien" || !kc.tokens.lock().unwrap().contains(bearer) {
        return kc_json(StatusCode::UNAUTHORIZED, json!({"error": "HTTP 401"}));
    }
    let (mut types, mut from, mut to, mut max, mut asc) = (Vec::new(), 0i64, i64::MAX, 100, false);
    for (k, v) in pairs(uri.query().unwrap_or("")) {
        match k.as_str() {
            "type" => types.push(v),
            "dateFrom" => from = v.parse().unwrap(),
            "dateTo" => to = v.parse().unwrap(),
            "max" => max = v.parse().unwrap(),
            "direction" => asc = v == "asc",
            _ => {}
        }
    }
    let mut sel: Vec<Value> = kc
        .events
        .lock()
        .unwrap()
        .iter()
        .filter(|e| {
            let t = e["time"].as_i64().unwrap();
            types.iter().any(|x| e["type"] == *x) && t >= from && t <= to
        })
        .cloned()
        .collect();
    sel.sort_by_key(|e| e["time"].as_i64().unwrap());
    if !asc {
        sel.reverse();
    }
    sel.truncate(max);
    kc_json(StatusCode::OK, Value::Array(sel))
}

async fn start_fake() -> (Fake, SocketAddr) {
    let kc: Fake = Arc::new(FakeKc::default());
    let app = Router::new()
        .route("/realms/{realm}/protocol/openid-connect/token", post(token))
        .route("/admin/realms/{realm}/events", get_route(events))
        .with_state(kc.clone());
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    (kc, addr)
}

fn push(kc: &Fake, ev: Value) {
    kc.events.lock().unwrap().push(ev);
}

fn ms(t: OffsetDateTime) -> i64 {
    (t.unix_timestamp_nanos() / 1_000_000) as i64
}

/// Event berbentuk EventRepresentation Keycloak 26.8.2 (SRC-621).
fn kc_event(time: i64, kind: &str, user: &str, session: Option<&str>, details: Value) -> Value {
    let mut e = json!({
        "id": new_id().hyphenated().to_string(),
        "time": time,
        "type": kind,
        "realmId": "5cf927d1-f2b5-4d7b-aa5e-48394f57de41",
        "clientId": "rs-pasien-mobile",
        "userId": user,
        "ipAddress": "203.0.113.9",
        "details": details,
    });
    if let Some(s) = session {
        e["sessionId"] = json!(s);
    }
    e
}

/// Event yang dipancarkan Keycloak 26.8.2 saat password baru tersimpan lewat tautan atur ulang dengan
/// `logout-sessions=on`: satu LOGOUT per sesi lain (`AuthenticatorUtil.logoutOtherSessions`), lalu
/// UPDATE_CREDENTIAL password dan salinan usang UPDATE_PASSWORD (`UpdatePassword.processAction`, SRC-770).
fn reset_events(kc: &Fake, time: i64, user: &str, sessions: &[&str]) {
    for s in sessions {
        push(
            kc,
            kc_event(
                time,
                "LOGOUT",
                user,
                Some(s),
                json!({"logout_triggered_by_required_action": "UPDATE_PASSWORD", "username": "x@example.com"}),
            ),
        );
    }
    push(
        kc,
        kc_event(
            time + 1,
            "UPDATE_CREDENTIAL",
            user,
            None,
            json!({"credential_type": "password", "username": "x@example.com"}),
        ),
    );
    push(
        kc,
        kc_event(
            time + 1,
            "UPDATE_PASSWORD",
            user,
            None,
            json!({"username": "x@example.com"}),
        ),
    );
}

fn client(addr: SocketAddr) -> Arc<KeycloakClient> {
    let mut creds = HashMap::new();
    creds.insert(
        IdpRealm::Pasien,
        RealmCredential {
            client_id: CLIENT_ID.into(),
            secret: Sensitive::new(SECRET.into()),
        },
    );
    let a: KeycloakAddress = format!("http://{addr}").parse().unwrap();
    Arc::new(KeycloakClient::new(a, Duration::from_secs(5), creds).unwrap())
}

fn options() -> PullOptions {
    PullOptions {
        realms: vec![IdpRealm::Pasien],
        page_size: 100,
        overlap: time::Duration::seconds(120),
        lag_alert: time::Duration::minutes(5),
    }
}

// ------------------------------------------------------------------------------------ database

fn url(var: &str) -> Sensitive<String> {
    EnvSource::from_process().required_secret(var).unwrap_or_else(|_| {
        panic!("{var} tidak diset: test integrasi wajib lewat `bash scripts/db-ephemeral.sh cargo test ...` (scripts/verify.sh)")
    })
}

async fn pool(var: &str) -> PgPool {
    PgPoolOptions::new()
        .max_connections(4)
        .connect(url(var).expose())
        .await
        .unwrap_or_else(|_| panic!("koneksi {var} gagal"))
}

/// Kunci pengambil; dicoba ulang sebentar karena sesi worker yang baru dihentikan test lain (abort) dilepas
/// PostgreSQL secara asinkron.
async fn lock() -> SingletonLock {
    let mut conn = PgConnection::connect(url("RS_TEST_DB_APP_URL").expose())
        .await
        .unwrap();
    for _ in 0..50 {
        match SingletonLock::try_acquire(conn, RS_KEYCLOAK_EVENT_PULLER)
            .await
            .unwrap()
        {
            LockAttempt::Acquired(l) => return l,
            LockAttempt::Busy(c) => conn = c,
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    panic!("kunci 7001002 dipegang sesi lain");
}

/// Posisi baca dan tanda terima dikosongkan supaya event berwaktu sekarang terbaca (test lain di berkas
/// berbeda memakai waktu tetap 2027).
async fn reset_cursor(app: &PgPool) {
    sqlx::query!("DELETE FROM app.idp_event_cursor")
        .execute(app)
        .await
        .unwrap();
    sqlx::query!("DELETE FROM app.idp_event_receipt")
        .execute(app)
        .await
        .unwrap();
}

fn random_hex(n: usize) -> String {
    let mut b = vec![0u8; n];
    crypto::fill_random(&mut b).unwrap();
    b.iter().map(|x| format!("{x:02x}")).collect()
}

/// Akun pasien sintetis; `idp_subject` = ID pengguna Keycloak (UUID).
async fn new_patient(app: &PgPool) -> (Uuid, String) {
    let id = new_id();
    let subject = new_id().hyphenated().to_string();
    let email = format!("td27-{}@example.com", random_hex(4));
    sqlx::query!(
        "INSERT INTO app.account (id, account_type, idp_realm, idp_subject, email, status, status_changed_at) \
         VALUES ($1, 'patient', 'pasien', $2, $3, 'active', now())",
        id,
        subject,
        email,
    )
    .execute(app)
    .await
    .unwrap();
    (id, subject)
}

struct Revocation {
    reason: String,
    expires_at: OffsetDateTime,
    account_id: Uuid,
}

async fn revocations(app: &PgPool, sid: &str) -> Vec<Revocation> {
    sqlx::query_as!(
        Revocation,
        "SELECT reason_code AS reason, expires_at, account_id FROM app.session_revocation WHERE idp_session_id = $1",
        sid
    )
    .fetch_all(app)
    .await
    .unwrap()
}

/// Payload audit milik akun ini (outbox; relay belum berjalan di test).
async fn audit_rows(relay: &PgPool, account: Uuid) -> Vec<Value> {
    sqlx::query_scalar!(
        "SELECT payload FROM app.audit_outbox WHERE payload->>'actor_account_id' = $1 ORDER BY occurred_at, id",
        account.hyphenated().to_string()
    )
    .fetch_all(relay)
    .await
    .unwrap()
}

// ------------------------------------------------------------------------------------ API

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

fn iss() -> String {
    "https://auth.ruang-sehat.invalid/realms/pasien".to_string()
}

fn authenticator(key: &TestRealmKey) -> Authenticator {
    Authenticator::new(
        AuthnConfig::with_defaults(
            Backend::RuangSehat,
            vec![RealmConfig {
                realm: Realm::Pasien,
                issuer: iss(),
                audiences: vec!["rs-api-pasien".into()],
                algorithms: vec![Algorithm::RS256],
                authorized_parties: vec![],
                jwks: Arc::new(StaticJwks::new(jwks_document(&[key.jwk()]))),
            }],
        ),
        Arc::new(SystemClock),
    )
    .unwrap()
}

/// Token akses sesi `sid` yang diterbitkan sebelum atur ulang dan masih berlaku 5 menit lagi.
fn access_token(key: &TestRealmKey, sub: &str, sid: &str) -> String {
    let t = SystemClock.now().unix_timestamp();
    key.sign(&json!({
        "iss": iss(), "aud": "rs-api-pasien", "sub": sub, "typ": "Bearer",
        "sid": sid, "azp": "rs-pasien-mobile", "iat": t - 5, "exp": t + 295,
    }))
    .unwrap()
}

fn directory(app: PgPool, valkey: Valkey) -> Arc<AccountDirectory> {
    Arc::new(AccountDirectory::new(
        app,
        valkey,
        Arc::new(SystemClock),
        Duration::from_secs(30),
    ))
}

fn dead_valkey() -> Valkey {
    Valkey::connect_lazy(&ValkeyConfig {
        url: dead_url(),
        op_timeout: OP,
    })
    .unwrap()
}

/// Proses `api`: penegak `authz` dengan resolver akun dan penjaga sesi modul `identity`.
async fn spawn_api(key: &TestRealmKey, dir: Arc<AccountDirectory>) -> SocketAddr {
    let auth = AuthState::new(Arc::new(authenticator(key)), Arc::new(Renderer));
    let (router, _) = PolicyRouter::<()>::new()
        .routes(Policy::patient(), routes!(me))
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

/// (status HTTP, `response_code`).
async fn call(addr: SocketAddr, bearer: &str) -> (u16, String) {
    let mut s = TcpStream::connect(addr).await.unwrap();
    let req = format!(
        "GET /patient/v1/me HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\nAuthorization: Bearer {bearer}\r\n\r\n"
    );
    s.write_all(req.as_bytes()).await.unwrap();
    let mut buf = Vec::new();
    s.read_to_end(&mut buf).await.unwrap();
    let text = String::from_utf8_lossy(&buf).into_owned();
    let status = text.get(9..12).unwrap().parse().unwrap();
    let body = text.split("\r\n\r\n").nth(1).unwrap_or_default();
    let code = serde_json::from_str::<Value>(body).unwrap_or(Value::Null)["response_code"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    (status, code)
}

const REJECTED: (u16, &str) = (401, "4010001");

fn rejected(r: &(u16, String)) -> bool {
    (r.0, r.1.as_str()) == REJECTED
}

// ------------------------------------------------------------------------------------ test

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn password_reset_ends_all_old_sessions_within_one_puller_round() {
    // trace: US-103 AC4d (sesi diakhiri <= 2 menit, E-04), KEPUTUSAN-198 butir 1, ADR-011 butir 8 dan 10
    let _serial = SERIAL.lock().await;
    let app = pool("RS_TEST_DB_APP_URL").await;
    let relay = pool("RS_TEST_DB_RELAY_URL").await;
    reset_cursor(&app).await;
    let vk = ValkeyContainer::start();
    let key = TestRealmKey::generate("pasien-td27").unwrap();
    // Proses api: salinan Valkey lengkap (penanda terpasang), sehingga penolakan harus datang dari salinan
    // yang ditulis worker, bukan dari jalur cadangan PostgreSQL.
    let api = directory(app.clone(), vk.client(OP));
    api.rebuild_revocation_cache().await.unwrap();
    let addr = spawn_api(&key, api).await;

    let (patient, sub) = new_patient(&app).await;
    let (other, other_sub) = new_patient(&app).await;
    let (sid_phone, sid_tablet, sid_other) = (
        format!("td27-a-{}", random_hex(6)),
        format!("td27-b-{}", random_hex(6)),
        format!("td27-c-{}", random_hex(6)),
    );
    let t_phone = access_token(&key, &sub, &sid_phone);
    let t_tablet = access_token(&key, &sub, &sid_tablet);
    let t_other = access_token(&key, &other_sub, &sid_other);
    for t in [&t_phone, &t_tablet, &t_other] {
        assert_eq!(call(addr, t).await.0, 200, "sesi aktif sebelum atur ulang");
    }

    // Atur ulang password 20 detik lalu (Keycloak sudah mengakhiri kedua sesi dan memancarkan LOGOUT).
    let (kc, kc_addr) = start_fake().await;
    let reset_at = ms(SystemClock.now()) - 20_000;
    reset_events(&kc, reset_at, &sub, &[&sid_phone, &sid_tablet]);

    // Token akses lama masih sah secara kriptografis; sebelum putaran pengambil, API belum tahu.
    assert_eq!(call(addr, &t_phone).await.0, 200);

    let worker = directory(app.clone(), vk.client(OP));
    let mut puller =
        Puller::new(options(), client(kc_addr), SystemClock.now()).with_revocations(worker);
    let mut l = lock().await;
    let res = puller.round(&mut l, &app, &SystemClock).await.unwrap();
    let report = res[0].1.clone().unwrap();
    assert_eq!(
        (report.fetched, report.written),
        (3, 3),
        "2 LOGOUT + UPDATE_CREDENTIAL; UPDATE_PASSWORD (salinan usang) tidak diminta: {report:?}"
    );

    // Satu putaran (bawaan 60 detik) setelah atur ulang: semua sesi lama ditolak, sesi akun lain tetap jalan.
    assert!(rejected(&call(addr, &t_phone).await));
    assert!(rejected(&call(addr, &t_tablet).await));
    assert_eq!(call(addr, &t_other).await.0, 200);
    assert!(revocations(&app, &sid_other).await.is_empty());
    let other_rows: i64 = sqlx::query_scalar!(
        r#"SELECT count(*) AS "n!" FROM app.session_revocation WHERE account_id = $1"#,
        other
    )
    .fetch_one(&app)
    .await
    .unwrap();
    assert_eq!(other_rows, 0, "akun lain tidak punya baris pencabutan");

    // Baris pencabutan: alasan logout, berlaku sampai waktu event + umur token akses + kelonggaran.
    for sid in [&sid_phone, &sid_tablet] {
        let rows = revocations(&app, sid).await;
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].reason, RevocationReason::Logout.as_str());
        assert_eq!(rows[0].account_id, patient);
        let expected = OffsetDateTime::from_unix_timestamp_nanos(i128::from(reset_at) * 1_000_000)
            .unwrap()
            + LOGOUT_REVOCATION_TTL;
        assert_eq!(rows[0].expires_at, expected);
        let pttl: i64 = vk
            .cli(&["PTTL", &format!("rs:session-revoked:sid:{sid}")])
            .trim()
            .parse()
            .unwrap();
        assert!(
            pttl > 0 && pttl <= 340_000,
            "salinan Valkey ber-TTL: {pttl}"
        );
    }
    assert_eq!(LOGOUT_REVOCATION_TTL, time::Duration::seconds(360));

    // E-04 (perubahan password) dan E-03 per sesi, tanpa ID sesi, email, atau nilai kredensial.
    let rows = audit_rows(&relay, patient).await;
    let actions: Vec<(&str, &str)> = rows
        .iter()
        .map(|p| {
            (
                p["event_code"].as_str().unwrap(),
                p["action_name"].as_str().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        actions.iter().filter(|a| **a == ("e_03", "logout")).count(),
        2
    );
    assert_eq!(
        actions
            .iter()
            .filter(|a| **a == ("e_04", "password_changed"))
            .count(),
        1,
        "{actions:?}"
    );
    let all = serde_json::to_string(&rows).unwrap();
    for leak in [
        sid_phone.as_str(),
        sid_tablet.as_str(),
        "example.com",
        "password\":",
    ] {
        assert!(!all.contains(leak), "payload audit memuat {leak}");
    }

    // Event dibaca ulang (tumpang-tindih) dan kiriman ganda event yang sama: idempoten.
    let dup = kc.events.lock().unwrap()[0].clone();
    push(&kc, dup);
    let res = puller.round(&mut l, &app, &SystemClock).await.unwrap();
    let again = res[0].1.clone().unwrap();
    assert_eq!((again.written, again.duplicates), (0, 4), "{again:?}");
    assert_eq!(audit_rows(&relay, patient).await.len(), rows.len());
    assert_eq!(revocations(&app, &sid_phone).await.len(), 1);
    assert!(rejected(&call(addr, &t_phone).await));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn valkey_down_still_rejects_old_sessions_from_postgres() {
    // trace: US-103 AC4d, RT-18 (Valkey mati → PostgreSQL), ADR-011 butir 8
    let _serial = SERIAL.lock().await;
    let app = pool("RS_TEST_DB_APP_URL").await;
    reset_cursor(&app).await;
    let key = TestRealmKey::generate("pasien-td27-vk").unwrap();
    let addr = spawn_api(&key, directory(app.clone(), dead_valkey())).await;
    let (_, sub) = new_patient(&app).await;
    let sid = format!("td27-d-{}", random_hex(6));
    let t = access_token(&key, &sub, &sid);
    assert_eq!(call(addr, &t).await.0, 200);

    let (kc, kc_addr) = start_fake().await;
    reset_events(&kc, ms(SystemClock.now()) - 5_000, &sub, &[&sid]);
    let mut puller = Puller::new(options(), client(kc_addr), SystemClock.now())
        .with_revocations(directory(app.clone(), dead_valkey()));
    let mut l = lock().await;
    let res = puller.round(&mut l, &app, &SystemClock).await.unwrap();
    // Penyalinan ke Valkey gagal tidak menggagalkan putaran: event dan pencabutan sudah commit.
    assert_eq!(res[0].1.clone().unwrap().written, 2);
    assert_eq!(revocations(&app, &sid).await.len(), 1);
    assert!(rejected(&call(addr, &t).await));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ignored_logout_events_do_not_revoke() {
    // trace: US-103 AC4d (negatif): hanya LOGOUT sukses bersesi milik akun dikenal yang token sesinya masih
    // mungkin berlaku
    let _serial = SERIAL.lock().await;
    let app = pool("RS_TEST_DB_APP_URL").await;
    reset_cursor(&app).await;
    let (patient, sub) = new_patient(&app).await;
    let (kc, kc_addr) = start_fake().await;
    let now = ms(SystemClock.now());
    let (s_err, s_old, s_unknown) = (
        format!("td27-e-{}", random_hex(6)),
        format!("td27-f-{}", random_hex(6)),
        format!("td27-g-{}", random_hex(6)),
    );
    let mut failed = kc_event(now - 3_000, "LOGOUT_ERROR", &sub, Some(&s_err), json!({}));
    failed["error"] = json!("user_session_not_found");
    push(&kc, failed);
    // Tanpa sessionId.
    push(&kc, kc_event(now - 2_000, "LOGOUT", &sub, None, json!({})));
    // Lebih tua dari umur token (6 menit lalu, masih di jendela baca karena posisi baca kosong).
    push(
        &kc,
        kc_event(now - 400_000, "LOGOUT", &sub, Some(&s_old), json!({})),
    );
    // Subjek Keycloak tanpa akun aplikasi.
    let stranger = new_id().hyphenated().to_string();
    push(
        &kc,
        kc_event(
            now - 1_000,
            "LOGOUT",
            &stranger,
            Some(&s_unknown),
            json!({}),
        ),
    );
    let mut puller = Puller::new(options(), client(kc_addr), SystemClock.now())
        .with_revocations(directory(app.clone(), dead_valkey()));
    let mut l = lock().await;
    let res = puller.round(&mut l, &app, &SystemClock).await.unwrap();
    assert_eq!(res[0].1.clone().unwrap().written, 4);
    for sid in [&s_err, &s_old, &s_unknown] {
        assert!(revocations(&app, sid).await.is_empty(), "{sid}");
    }
    let n: i64 = sqlx::query_scalar!(
        r#"SELECT count(*) AS "n!" FROM app.session_revocation WHERE account_id = $1"#,
        patient
    )
    .fetch_one(&app)
    .await
    .unwrap();
    assert_eq!(n, 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn worker_start_copies_committed_but_unpublished_revocations() {
    // trace: TD-24 (jalur pengambil event), US-103 AC4d, ADR-011 butir 8
    let _serial = SERIAL.lock().await;
    let app = pool("RS_TEST_DB_APP_URL").await;
    reset_cursor(&app).await;
    let vk = ValkeyContainer::start();
    let key = TestRealmKey::generate("pasien-td27-rs").unwrap();
    let api = directory(app.clone(), vk.client(OP));
    api.rebuild_revocation_cache().await.unwrap();
    let addr = spawn_api(&key, api).await;
    let (patient, sub) = new_patient(&app).await;
    let sid = format!("td27-h-{}", random_hex(6));
    let t = access_token(&key, &sub, &sid);

    // Worker sebelumnya mati di antara commit dan penyalinan: baris ada di PostgreSQL, Valkey belum tahu.
    let crashed = directory(app.clone(), vk.client(OP));
    let mut tx = app.begin().await.unwrap();
    crashed
        .record_revocation(
            &mut tx,
            patient,
            &sid,
            RevocationReason::Logout,
            SystemClock.now() + time::Duration::minutes(5),
        )
        .await
        .unwrap();
    tx.commit().await.unwrap();
    assert_eq!(
        call(addr, &t).await.0,
        200,
        "celah yang ditutup: salinan Valkey lengkap tanpa entri sesi ini"
    );

    // Worker mulai lagi: membangun ulang salinan dari PostgreSQL saat kunci diperoleh.
    let (_kc, kc_addr) = start_fake().await;
    let settings = PullerSettings {
        options: options(),
        database_url: url("RS_TEST_DB_APP_URL"),
        poll_interval: Duration::from_millis(200),
        retry_interval: Duration::from_millis(200),
    };
    let worker = tokio::spawn(run_puller(
        settings,
        client(kc_addr),
        Arc::new(SystemClock),
        directory(app.clone(), vk.client(OP)),
    ));
    let key_name = format!("rs:session-revoked:sid:{sid}");
    let mut copied = false;
    // Batas longgar untuk mesin CI yang sibuk; biasanya < 1 detik.
    for _ in 0..600 {
        if vk.cli(&["EXISTS", &key_name]).trim() == "1" {
            copied = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    worker.abort();
    let _ = worker.await;
    assert!(copied, "salinan dibangun ulang saat worker mulai");
    assert!(rejected(&call(addr, &t).await));
}
