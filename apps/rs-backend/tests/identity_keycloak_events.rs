//! F1-T34 (T1; ADR-009 butir 2, ADR-011 butir 10, PRD §7.1-§7.3): pengambil event Keycloak E-01-E-04 ke
//! `audit_outbox` lewat `audit::in_audited_tx`, satu pengambil aktif (kunci 7001002), posisi baca tahan
//! restart, tanpa duplikasi, tanpa email/token/ID sesi di payload, dan peringatan keterlambatan.
//!
//! Keycloak di test ini adalah tiruan Admin REST yang setia di 127.0.0.1 (proses test): bentuk JSON event
//! disalin dari respons Keycloak 26.8.2 sungguhan (SRC-621, termasuk `details.username` = email,
//! `token_id`, `refresh_token_id`, `reason`), semantik parameter `type`/`dateFrom`/`dateTo`/`direction`/
//! `first`/`max` mengikuti kode sumber `RealmAdminResource` dan `JpaEventQuery` 26.8.2 (SRC-620): urutan
//! hanya menurut `time` (seri disusun berbeda per permintaan supaya pemecah seri tidak diandalkan),
//! `type` tak dikenal -> 500, token tanpa izin -> 401. Perilaku klien layanan terhadap Keycloak
//! sungguhan (hak minimum, cakupan `kc-admin-api`) diuji `scripts/keycloak-check.sh`.
//!
//! PostgreSQL 18 sekali pakai dari `scripts/db-ephemeral.sh`; tanpa variabel `RS_TEST_DB_*` test GAGAL.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use axum::Router;
use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use foundation::{Clock, FixedClock, Sensitive, Uuid, new_id};
use observability::log::{Component, JsonLogLayer, LogSink, MemorySink, subscriber};
use observability::{EnvSource, LogLevel};
use platform_jobs::lock::{RS_KEYCLOAK_EVENT_PULLER, RS_PRIVACY_WORKER};
use platform_jobs::{LockAttempt, SingletonLock};
use rs_backend::modules::identity::api::{
    IdpRealm, KeycloakAddress, KeycloakClient, KeycloakError, PullError, PullOptions, Puller,
    RealmCredential,
};
use serde_json::{Value, json};
use sqlx::postgres::PgPoolOptions;
use sqlx::{Connection, PgConnection, PgPool};
use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use time::OffsetDateTime;
use time::macros::datetime;

/// Test berbagi tabel posisi baca per realm dan `audit_outbox`: dijalankan satu per satu.
static SERIAL: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

const NOW: OffsetDateTime = datetime!(2027-03-01 03:00:00 UTC);
const CLIENT_ID: &str = "rs-backend-event-reader";
const SECRET_PASIEN: &str = "rahasia-uji-pasien-0123456789abcdef";
const SECRET_ADMIN: &str = "rahasia-uji-admin-0123456789abcdef";
const PATIENT_EMAIL: &str = "budi.uji@example.invalid";
const UNKNOWN_EMAIL: &str = "tidak.terdaftar@example.invalid";
const ADMIN_EMAIL: &str = "admin.uji@example.invalid";
const TOKEN_ID: &str = "onrtro:94a37201-2b5f-7e8e-aff1-64fda8203d40";
const REFRESH_TOKEN_ID: &str = "ca30a31c-2f6d-9af7-3c23-6766b3f8931a";
const SESSION_ID: &str = "YK9dKN5WZg0tGfmg2RvbYtzf";
const REASON_TEXT: &str = "Error when attempting to send the email to the server. More information is available in the server log.";

// ------------------------------------------------------------------------------------ tiruan Keycloak

/// Nama `EventType` Keycloak 26.8.2 yang dipakai fixture (SRC-622); nama lain -> HTTP 500 seperti
/// `EventType.valueOf`.
const KNOWN_TYPES: &[&str] = &[
    "LOGIN",
    "LOGIN_ERROR",
    "LOGOUT",
    "LOGOUT_ERROR",
    "REGISTER",
    "REGISTER_ERROR",
    "CODE_TO_TOKEN",
    "CLIENT_LOGIN",
    "CLIENT_LOGIN_ERROR",
    "UPDATE_PASSWORD",
    "UPDATE_PASSWORD_ERROR",
    "UPDATE_TOTP",
    "UPDATE_TOTP_ERROR",
    "REMOVE_TOTP",
    "REMOVE_TOTP_ERROR",
    "UPDATE_CREDENTIAL",
    "UPDATE_CREDENTIAL_ERROR",
    "REMOVE_CREDENTIAL",
    "REMOVE_CREDENTIAL_ERROR",
    "RESET_PASSWORD",
    "RESET_PASSWORD_ERROR",
    "SEND_RESET_PASSWORD",
    "SEND_RESET_PASSWORD_ERROR",
    "USER_DISABLED_BY_TEMPORARY_LOCKOUT",
    "USER_DISABLED_BY_PERMANENT_LOCKOUT",
    "VERIFY_EMAIL",
];

#[derive(Default)]
struct FakeKc {
    events: Mutex<HashMap<String, Vec<Value>>>,
    secrets: Mutex<HashMap<String, String>>,
    tokens: Mutex<HashSet<String>>,
    down: AtomicBool,
    /// Bila > 0: permintaan event ke-N dan seterusnya dijawab 503.
    fail_from_request: AtomicUsize,
    event_requests: AtomicUsize,
    token_requests: AtomicUsize,
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

fn pct_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'%' if i + 2 < b.len() => {
                let h = std::str::from_utf8(&b[i + 1..i + 3]).unwrap();
                out.push(u8::from_str_radix(h, 16).unwrap());
                i += 3;
            }
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            c => {
                out.push(c);
                i += 1;
            }
        }
    }
    String::from_utf8(out).unwrap()
}

fn pairs(s: &str) -> Vec<(String, String)> {
    s.split('&')
        .filter(|p| !p.is_empty())
        .map(|p| {
            let (k, v) = p.split_once('=').unwrap_or((p, ""));
            (pct_decode(k), pct_decode(v))
        })
        .collect()
}

async fn token(State(kc): State<Fake>, Path(realm): Path<String>, body: Bytes) -> Response {
    if kc.down.load(Ordering::SeqCst) {
        return kc_json(StatusCode::SERVICE_UNAVAILABLE, json!({}));
    }
    kc.token_requests.fetch_add(1, Ordering::SeqCst);
    let form: HashMap<String, String> = pairs(std::str::from_utf8(&body).unwrap())
        .into_iter()
        .collect();
    let ok = form.get("grant_type").map(String::as_str) == Some("client_credentials")
        && form.get("client_id").map(String::as_str) == Some(CLIENT_ID)
        && kc.secrets.lock().unwrap().get(&realm) == form.get("client_secret");
    if !ok {
        return kc_json(
            StatusCode::UNAUTHORIZED,
            json!({"error": "unauthorized_client", "error_description": "Invalid client or Invalid client credentials"}),
        );
    }
    let t = format!("{realm}.{}", new_id());
    kc.tokens.lock().unwrap().insert(t.clone());
    kc_json(
        StatusCode::OK,
        json!({"access_token": t, "expires_in": 300, "refresh_expires_in": 0, "token_type": "Bearer",
               "not-before-policy": 0, "scope": "email profile kc-admin-api"}),
    )
}

/// `DateUtil.toStartOfDay`/`toEndOfDay` Keycloak 26.8.2: `yyyy-MM-dd` (UTC) atau milidetik epoch.
fn date_param(v: &str, end: bool) -> Option<i64> {
    if v.contains('-') {
        let mut it = v.split('-');
        let (y, m, d) = (
            it.next()?.parse().ok()?,
            it.next()?.parse::<u8>().ok()?,
            it.next()?.parse().ok()?,
        );
        let date = time::Date::from_calendar_date(y, time::Month::try_from(m).ok()?, d).ok()?;
        let t = if end {
            date.with_hms_milli(23, 59, 59, 999).ok()?
        } else {
            date.midnight()
        };
        Some((t.assume_utc().unix_timestamp_nanos() / 1_000_000) as i64)
    } else {
        v.parse().ok()
    }
}

async fn events(
    State(kc): State<Fake>,
    Path(realm): Path<String>,
    headers: HeaderMap,
    uri: Uri,
) -> Response {
    if kc.down.load(Ordering::SeqCst) {
        return kc_json(StatusCode::SERVICE_UNAVAILABLE, json!({}));
    }
    let n = kc.event_requests.fetch_add(1, Ordering::SeqCst) + 1;
    let fail_from = kc.fail_from_request.load(Ordering::SeqCst);
    if fail_from > 0 && n >= fail_from {
        return kc_json(StatusCode::SERVICE_UNAVAILABLE, json!({}));
    }
    let bearer = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .unwrap_or("");
    if !bearer.starts_with(&format!("{realm}.")) || !kc.tokens.lock().unwrap().contains(bearer) {
        return kc_json(
            StatusCode::UNAUTHORIZED,
            json!({"error": "HTTP 401 Unauthorized"}),
        );
    }
    let q = pairs(uri.query().unwrap_or(""));
    let mut types = Vec::new();
    let (mut from, mut to, mut first, mut max, mut asc) = (None, None, 0usize, 100usize, false);
    for (k, v) in &q {
        match k.as_str() {
            "type" => {
                if !KNOWN_TYPES.contains(&v.as_str()) {
                    return kc_json(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        json!({"error": "unknown_error"}),
                    );
                }
                types.push(v.clone());
            }
            "dateFrom" => match date_param(v, false) {
                Some(t) => from = Some(t),
                None => {
                    return kc_json(
                        StatusCode::BAD_REQUEST,
                        json!({"error": "Invalid value for 'dateFrom'"}),
                    );
                }
            },
            "dateTo" => match date_param(v, true) {
                Some(t) => to = Some(t),
                None => {
                    return kc_json(
                        StatusCode::BAD_REQUEST,
                        json!({"error": "Invalid value for 'dateTo'"}),
                    );
                }
            },
            "direction" => match v.as_str() {
                "asc" => asc = true,
                "desc" => asc = false,
                _ => {
                    return kc_json(
                        StatusCode::BAD_REQUEST,
                        json!({"error": "Invalid value for sortDirection"}),
                    );
                }
            },
            "first" => first = v.parse().unwrap(),
            "max" => max = v.parse().unwrap(),
            _ => {}
        }
    }
    let all = kc
        .events
        .lock()
        .unwrap()
        .get(&realm)
        .cloned()
        .unwrap_or_default();
    let mut sel: Vec<Value> = all
        .into_iter()
        .filter(|e| {
            let t = e["time"].as_i64().unwrap();
            (types.is_empty() || types.iter().any(|x| e["type"] == *x))
                && from.is_none_or(|f| t >= f)
                && to.is_none_or(|x| t <= x)
        })
        .collect();
    // Urutan hanya menurut time (JpaEventQuery); seri sengaja disusun berbeda tiap permintaan.
    if n % 2 == 0 {
        sel.reverse();
    }
    sel.sort_by_key(|e| e["time"].as_i64().unwrap());
    if !asc {
        sel.reverse();
    }
    let page: Vec<Value> = sel.into_iter().skip(first).take(max).collect();
    kc_json(StatusCode::OK, Value::Array(page))
}

async fn start_fake() -> (Fake, SocketAddr) {
    let kc: Fake = Arc::new(FakeKc::default());
    kc.secrets
        .lock()
        .unwrap()
        .insert("pasien".into(), SECRET_PASIEN.into());
    kc.secrets
        .lock()
        .unwrap()
        .insert("admin".into(), SECRET_ADMIN.into());
    let app = Router::new()
        .route("/realms/{realm}/protocol/openid-connect/token", post(token))
        .route("/admin/realms/{realm}/events", get(events))
        .with_state(kc.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    (kc, addr)
}

fn push(kc: &Fake, realm: &str, ev: Value) {
    kc.events
        .lock()
        .unwrap()
        .entry(realm.into())
        .or_default()
        .push(ev);
}

fn ms(t: OffsetDateTime) -> i64 {
    (t.unix_timestamp_nanos() / 1_000_000) as i64
}

/// Event berbentuk EventRepresentation Keycloak 26.8.2 (SRC-621).
fn kc_event(
    time: i64,
    kind: &str,
    user: Option<&str>,
    error: Option<&str>,
    details: Value,
) -> Value {
    let mut e = json!({
        "id": new_id().hyphenated().to_string(),
        "time": time,
        "type": kind,
        "realmId": "5cf927d1-f2b5-4d7b-aa5e-48394f57de41",
        "clientId": "rs-pasien-mobile",
        "ipAddress": "203.0.113.7",
        "details": details,
    });
    if let Some(u) = user {
        e["userId"] = json!(u);
    }
    if let Some(x) = error {
        e["error"] = json!(x);
    }
    e
}

fn login_details(email: &str) -> Value {
    json!({"auth_method": "openid-connect", "token_id": TOKEN_ID, "grant_type": "authorization_code",
           "refresh_token_type": "Refresh", "scope": "openid email profile", "refresh_token_id": REFRESH_TOKEN_ID,
           "code_id": SESSION_ID, "client_auth_method": "client-secret", "username": email,
           "redirect_uri": "com.example.ruangsehat.pasien:/oauth2redirect"})
}

// ------------------------------------------------------------------------------------ pembantu

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

async fn lock() -> SingletonLock {
    let conn = PgConnection::connect(url("RS_TEST_DB_APP_URL").expose())
        .await
        .unwrap();
    match SingletonLock::try_acquire(conn, RS_KEYCLOAK_EVENT_PULLER)
        .await
        .unwrap()
    {
        LockAttempt::Acquired(l) => l,
        LockAttempt::Busy(_) => panic!("kunci 7001002 dipegang sesi lain"),
    }
}

struct Env {
    app: PgPool,
    relay: PgPool,
    kc: Fake,
    addr: SocketAddr,
}

/// Mengosongkan posisi baca, tanda terima, dan outbox (hanya data test ini; dijalankan serial).
async fn env() -> Env {
    let app = pool("RS_TEST_DB_APP_URL").await;
    let relay = pool("RS_TEST_DB_RELAY_URL").await;
    sqlx::query!("DELETE FROM app.idp_event_cursor")
        .execute(&app)
        .await
        .unwrap();
    sqlx::query!("DELETE FROM app.idp_event_receipt")
        .execute(&app)
        .await
        .unwrap();
    sqlx::query!("DELETE FROM app.audit_outbox")
        .execute(&relay)
        .await
        .unwrap();
    let (kc, addr) = start_fake().await;
    Env {
        app,
        relay,
        kc,
        addr,
    }
}

fn client(addr: SocketAddr, secret_pasien: &str) -> Arc<KeycloakClient> {
    let mut creds = HashMap::new();
    creds.insert(
        IdpRealm::Pasien,
        RealmCredential {
            client_id: CLIENT_ID.into(),
            secret: Sensitive::new(secret_pasien.into()),
        },
    );
    creds.insert(
        IdpRealm::Admin,
        RealmCredential {
            client_id: CLIENT_ID.into(),
            secret: Sensitive::new(SECRET_ADMIN.into()),
        },
    );
    let a: KeycloakAddress = format!("http://{addr}").parse().unwrap();
    Arc::new(KeycloakClient::new(a, Duration::from_secs(5), creds).unwrap())
}

fn options(realms: &[IdpRealm], page_size: usize) -> PullOptions {
    PullOptions {
        realms: realms.to_vec(),
        page_size,
        overlap: time::Duration::seconds(120),
        lag_alert: time::Duration::minutes(5),
    }
}

async fn outbox(relay: &PgPool) -> Vec<Value> {
    sqlx::query_scalar!("SELECT payload FROM app.audit_outbox ORDER BY occurred_at, id")
        .fetch_all(relay)
        .await
        .unwrap()
}

async fn receipts(app: &PgPool) -> i64 {
    sqlx::query_scalar!(r#"SELECT count(*) AS "n!" FROM app.idp_event_receipt"#)
        .fetch_one(app)
        .await
        .unwrap()
}

async fn position(app: &PgPool, realm: &str) -> Option<i64> {
    sqlx::query_scalar!(
        "SELECT position_ms FROM app.idp_event_cursor WHERE idp_realm = $1",
        realm
    )
    .fetch_optional(app)
    .await
    .unwrap()
}

async fn insert_account(
    app: &PgPool,
    account_type: &str,
    realm: &str,
    subject: &str,
    email: &str,
) -> Uuid {
    let id = new_id();
    let full_name = (account_type == "rs_admin").then_some("Admin Uji");
    sqlx::query!(
        "INSERT INTO app.account (id, account_type, idp_realm, idp_subject, email, status, status_changed_at, full_name) \
         VALUES ($1, $2, $3, $4, $5, 'active', now(), $6)",
        id, account_type, realm, subject, email, full_name,
    )
    .execute(app)
    .await
    .unwrap();
    id
}

fn capture() -> (MemorySink, tracing::subscriber::DefaultGuard) {
    let app = MemorySink::default();
    let sec = MemorySink::default();
    let layer = JsonLogLayer::new(
        Component::Worker,
        LogLevel::Info,
        Arc::new(FixedClock::new(NOW)),
    )
    .with_sinks(
        Arc::new(app) as Arc<dyn LogSink>,
        Arc::new(sec.clone()) as Arc<dyn LogSink>,
    );
    (sec, tracing::subscriber::set_default(subscriber(layer)))
}

fn by_action<'a>(rows: &'a [Value], action: &str) -> Vec<&'a Value> {
    rows.iter().filter(|p| p["action_name"] == action).collect()
}

// ------------------------------------------------------------------------------------ test

#[tokio::test]
async fn events_are_mapped_masked_and_written_once() {
    let _serial = SERIAL.lock().await;
    let e = env().await;
    let (sec, _guard) = capture();
    let kc_patient = new_id().hyphenated().to_string();
    let kc_admin = new_id().hyphenated().to_string();
    let patient = insert_account(&e.app, "patient", "pasien", &kc_patient, PATIENT_EMAIL).await;
    let admin = insert_account(&e.app, "rs_admin", "admin", &kc_admin, ADMIN_EMAIL).await;
    sqlx::query!(
        "INSERT INTO app.rs_admin_role (id, account_id, sub_role) VALUES ($1, $2, 'verifier')",
        new_id(),
        admin
    )
    .execute(&e.app)
    .await
    .unwrap();

    let t0 = ms(NOW) - 60_000;
    let p = Some(kc_patient.as_str());
    push(
        &e.kc,
        "pasien",
        kc_event(t0, "LOGIN", p, None, login_details(PATIENT_EMAIL)),
    );
    push(
        &e.kc,
        "pasien",
        kc_event(
            t0 + 1,
            "LOGIN_ERROR",
            p,
            Some("invalid_user_credentials"),
            json!({"auth_method": "openid-connect", "username": PATIENT_EMAIL, "redirect_uri": "x"}),
        ),
    );
    push(
        &e.kc,
        "pasien",
        kc_event(
            t0 + 2,
            "LOGIN_ERROR",
            None,
            Some("user_not_found"),
            json!({"auth_method": "openid-connect", "username": UNKNOWN_EMAIL}),
        ),
    );
    push(
        &e.kc,
        "pasien",
        kc_event(
            t0 + 3,
            "USER_DISABLED_BY_TEMPORARY_LOCKOUT",
            p,
            None,
            json!({}),
        ),
    );
    push(
        &e.kc,
        "pasien",
        kc_event(t0 + 4, "LOGOUT", p, None, json!({"redirect_uri": "x"})),
    );
    // Satu perubahan OTP: Keycloak 26.8.2 memancarkan UPDATE_CREDENTIAL dan salinan usang UPDATE_TOTP.
    push(
        &e.kc,
        "pasien",
        kc_event(
            t0 + 5,
            "UPDATE_CREDENTIAL",
            p,
            None,
            json!({"credential_type": "otp", "username": PATIENT_EMAIL, "custom_required_action": "CONFIGURE_TOTP"}),
        ),
    );
    push(
        &e.kc,
        "pasien",
        kc_event(
            t0 + 5,
            "UPDATE_TOTP",
            p,
            None,
            json!({"username": PATIENT_EMAIL}),
        ),
    );
    // Lupa password dengan email gagal terkirim (F1-T36): details berisi email dan teks galat.
    push(
        &e.kc,
        "pasien",
        kc_event(
            t0 + 6,
            "SEND_RESET_PASSWORD_ERROR",
            p,
            Some("email_send_failed"),
            json!({"reason": REASON_TEXT, "email": PATIENT_EMAIL, "username": PATIENT_EMAIL, "auth_method": "openid-connect"}),
        ),
    );
    // Jenis di luar pemetaan: token klien layanan dan pertukaran kode.
    push(
        &e.kc,
        "pasien",
        kc_event(
            t0 + 7,
            "CLIENT_LOGIN",
            None,
            None,
            json!({"token_id": TOKEN_ID, "username": "service-account-rs-backend-event-reader"}),
        ),
    );
    push(
        &e.kc,
        "pasien",
        kc_event(
            t0 + 8,
            "CODE_TO_TOKEN",
            p,
            None,
            json!({"token_id": TOKEN_ID}),
        ),
    );
    let mut adm = kc_event(
        t0 + 9,
        "LOGIN",
        Some(kc_admin.as_str()),
        None,
        login_details(ADMIN_EMAIL),
    );
    adm["clientId"] = json!("rs-backoffice-bff");
    adm["ipAddress"] = json!("2001:db8::17");
    push(&e.kc, "admin", adm);

    let client = client(e.addr, SECRET_PASIEN);
    let clock = FixedClock::new(NOW);
    let mut puller = Puller::new(
        options(&[IdpRealm::Pasien, IdpRealm::Admin], 100),
        client,
        clock.now(),
    );
    let mut l = lock().await;
    let res = puller.round(&mut l, &e.app, &clock).await.unwrap();
    let pasien = res
        .iter()
        .find(|(r, _)| *r == IdpRealm::Pasien)
        .unwrap()
        .1
        .clone()
        .unwrap();
    assert_eq!(
        (pasien.fetched, pasien.written, pasien.duplicates),
        (7, 7, 0),
        "{pasien:?}"
    );
    assert_eq!(pasien.position_ms, t0 + 6);

    let rows = outbox(&e.relay).await;
    assert_eq!(
        rows.len(),
        8,
        "7 event pasien + 1 admin; UPDATE_TOTP, CLIENT_LOGIN, CODE_TO_TOKEN tidak dicatat"
    );

    let login = by_action(&rows, "login");
    let pl = login
        .iter()
        .find(|p| p["source_component"] == "patient_app")
        .unwrap();
    assert_eq!(pl["event_code"], "e_01");
    assert_eq!(pl["actor_type"], "account");
    assert_eq!(
        pl["actor_account_id"],
        json!(patient.hyphenated().to_string())
    );
    assert_eq!(pl["actor_account_type"], "patient");
    assert_eq!(pl["object_type"], "account");
    assert_eq!(pl["object_id"], json!(patient.hyphenated().to_string()));
    assert_eq!(pl["source_ip"], "203.0.113.7");
    assert_eq!(pl["outcome"], "success");
    assert_eq!(
        pl["occurred_at"], "2027-03-01T02:59:00Z",
        "occurred_at = waktu event Keycloak"
    );
    let al = login
        .iter()
        .find(|p| p["source_component"] == "backoffice")
        .unwrap();
    assert_eq!(
        al["actor_account_id"],
        json!(admin.hyphenated().to_string())
    );
    assert_eq!(al["actor_roles"], json!(["verifier"]));
    assert_eq!(al["source_ip"], "2001:db8::17");

    let failed = by_action(&rows, "login_failed");
    assert_eq!(failed.len(), 2);
    let known = failed
        .iter()
        .find(|p| p["actor_type"] == "account")
        .unwrap();
    assert_eq!(
        (
            known["event_code"].as_str(),
            known["outcome_reason_code"].as_str()
        ),
        (Some("e_02"), Some("invalid_user_credentials"))
    );
    let unknown = failed.iter().find(|p| p["actor_type"] == "system").unwrap();
    assert_eq!(unknown["event_code"], "e_02");
    assert_eq!(unknown["outcome"], "failure");
    assert_eq!(unknown["outcome_reason_code"], "user_not_found");
    assert_eq!(unknown["actor_account_id"], Value::Null);
    assert_eq!(unknown["object_id"], Value::Null);
    assert_eq!(unknown["source_ip"], "203.0.113.7");

    assert_eq!(
        by_action(&rows, "account_locked_temporarily")[0]["event_code"],
        "e_02"
    );
    assert_eq!(by_action(&rows, "logout")[0]["event_code"], "e_03");
    let otp = by_action(&rows, "otp_configured");
    assert_eq!(otp.len(), 1, "UPDATE_TOTP usang tidak menggandakan E-04");
    assert_eq!(otp[0]["event_code"], "e_04");
    let reset = by_action(&rows, "password_reset_requested");
    assert_eq!(
        (
            reset[0]["event_code"].as_str(),
            reset[0]["outcome_reason_code"].as_str()
        ),
        (Some("e_04"), Some("email_send_failed"))
    );

    // Field terlarang tidak masuk outbox (PRD §7.3, ADR-005): email, token, ID sesi, teks alasan Keycloak.
    let all = serde_json::to_string(&rows).unwrap();
    for forbidden in [
        PATIENT_EMAIL,
        UNKNOWN_EMAIL,
        ADMIN_EMAIL,
        "@",
        TOKEN_ID,
        REFRESH_TOKEN_ID,
        SESSION_ID,
        "Error when attempting",
        "username",
        "redirect_uri",
        "token_id",
        "service-account",
        kc_patient.as_str(),
        kc_admin.as_str(),
    ] {
        assert!(!all.contains(forbidden), "outbox memuat {forbidden}");
    }
    // Diteruskan juga ke security log (ADR-011 butir 10), tanpa email.
    let log = sec.text();
    assert_eq!(
        log.matches("\"event_type\":\"idp_event\"").count(),
        8,
        "{log}"
    );
    assert!(!log.contains('@') && !log.contains(TOKEN_ID), "{log}");
    assert_eq!(receipts(&e.app).await, 8);

    // Putaran kedua membaca ulang jendela tumpang-tindih: tidak ada yang tercatat dua kali.
    let again = puller.round(&mut l, &e.app, &clock).await.unwrap();
    let r2 = again[0].1.clone().unwrap();
    assert_eq!((r2.written, r2.duplicates), (0, 7), "{r2:?}");
    assert_eq!(outbox(&e.relay).await.len(), 8);
    assert_eq!(
        e.kc.token_requests.load(Ordering::SeqCst),
        2,
        "token disimpan per realm sampai kedaluwarsa"
    );
    l.release().await.unwrap();
}

#[tokio::test]
async fn restart_and_rereads_neither_lose_nor_duplicate_events() {
    let _serial = SERIAL.lock().await;
    let e = env().await;
    let t0 = ms(NOW) - 30_000;
    // Tujuh event, dua pasang di milidetik yang sama (batas halaman ukuran 2 jatuh di tengah seri).
    let times = [t0, t0 + 1, t0 + 1, t0 + 2, t0 + 3, t0 + 3, t0 + 4];
    let mut ids = Vec::new();
    for t in times {
        let ev = kc_event(
            t,
            "LOGIN_ERROR",
            None,
            Some("user_not_found"),
            json!({"username": UNKNOWN_EMAIL}),
        );
        ids.push(ev["id"].as_str().unwrap().to_string());
        push(&e.kc, "pasien", ev);
    }
    // Keycloak mengembalikan event yang sama dua kali (salinan identik).
    let dup = e.kc.events.lock().unwrap()["pasien"][4].clone();
    push(&e.kc, "pasien", dup);
    let clock = FixedClock::new(NOW);
    let opts = options(&[IdpRealm::Pasien], 2);

    // Proses pertama "mati" di tengah: permintaan event ke-3 dan seterusnya gagal.
    e.kc.fail_from_request.store(3, Ordering::SeqCst);
    let mut first = Puller::new(opts.clone(), client(e.addr, SECRET_PASIEN), clock.now());
    let mut l = lock().await;
    let res = first.round(&mut l, &e.app, &clock).await.unwrap();
    assert_eq!(
        res[0].1,
        Err(PullError::Keycloak(KeycloakError::Status(503)))
    );
    let partial = outbox(&e.relay).await.len();
    assert!(
        partial > 0 && partial < 7,
        "sebagian tertulis sebelum gagal: {partial}"
    );
    let saved = position(&e.app, "pasien").await.unwrap();
    assert!(
        saved >= t0 && saved < t0 + 4,
        "posisi tersimpan per halaman: {saved}"
    );
    l.release().await.unwrap();
    drop(first);

    // Proses baru (restart): melanjutkan dari posisi tersimpan.
    e.kc.fail_from_request.store(0, Ordering::SeqCst);
    let mut second = Puller::new(opts.clone(), client(e.addr, SECRET_PASIEN), clock.now());
    let mut l = lock().await;
    let r = second.round(&mut l, &e.app, &clock).await.unwrap()[0]
        .1
        .clone()
        .unwrap();
    assert_eq!(r.position_ms, t0 + 4);
    assert_eq!(
        outbox(&e.relay).await.len(),
        7,
        "tepat tujuh: tanpa kehilangan, tanpa duplikat"
    );
    assert_eq!(receipts(&e.app).await, 7);

    // Event yang commit terlambat dengan waktu di dalam jendela tumpang-tindih tetap terambil sekali.
    push(
        &e.kc,
        "pasien",
        kc_event(
            t0 + 2,
            "LOGIN_ERROR",
            None,
            Some("user_not_found"),
            json!({}),
        ),
    );
    let r = second.round(&mut l, &e.app, &clock).await.unwrap()[0]
        .1
        .clone()
        .unwrap();
    assert_eq!(r.written, 1, "{r:?}");
    let r = second.round(&mut l, &e.app, &clock).await.unwrap()[0]
        .1
        .clone()
        .unwrap();
    assert_eq!(r.written, 0, "{r:?}");
    assert_eq!(outbox(&e.relay).await.len(), 8);

    // Halaman penuh yang seluruhnya di satu milidetik tetap diselesaikan.
    for _ in 0..5 {
        push(
            &e.kc,
            "pasien",
            kc_event(t0 + 10, "LOGOUT", None, None, json!({})),
        );
    }
    let r = second.round(&mut l, &e.app, &clock).await.unwrap()[0]
        .1
        .clone()
        .unwrap();
    assert_eq!((r.written, r.position_ms), (5, t0 + 10), "{r:?}");
    assert_eq!(outbox(&e.relay).await.len(), 13);
    l.release().await.unwrap();
}

#[tokio::test]
async fn keycloak_unavailable_warns_without_losing_position() {
    let _serial = SERIAL.lock().await;
    let e = env().await;
    let (sec, _guard) = capture();
    let t0 = ms(NOW) - 10_000;
    push(
        &e.kc,
        "pasien",
        kc_event(t0, "LOGOUT", None, None, json!({})),
    );
    let clock = FixedClock::new(NOW);
    let mut puller = Puller::new(
        options(&[IdpRealm::Pasien], 100),
        client(e.addr, SECRET_PASIEN),
        clock.now(),
    );
    let mut l = lock().await;
    puller.round(&mut l, &e.app, &clock).await.unwrap()[0]
        .1
        .clone()
        .unwrap();
    assert_eq!(position(&e.app, "pasien").await, Some(t0));

    // Keycloak tidak tersedia: galat per realm, peringatan, posisi tetap.
    e.kc.down.store(true, Ordering::SeqCst);
    push(
        &e.kc,
        "pasien",
        kc_event(t0 + 5, "LOGOUT", None, None, json!({})),
    );
    clock.advance(time::Duration::minutes(1));
    let res = puller.round(&mut l, &e.app, &clock).await.unwrap();
    assert!(
        matches!(res[0].1, Err(PullError::Keycloak(_))),
        "{:?}",
        res[0].1
    );
    assert_eq!(position(&e.app, "pasien").await, Some(t0));
    let log = sec.text();
    assert!(
        log.contains("\"event_type\":\"idp_event_pull_failed\"")
            && log.contains("\"stage\":\"keycloak_unavailable\""),
        "{log}"
    );
    assert!(
        !log.contains("idp_event_pull_delayed"),
        "belum lewat ambang 5 menit"
    );

    // Lewat ambang keterlambatan 5 menit sejak putaran terakhir yang berhasil.
    clock.advance(time::Duration::minutes(5));
    puller.round(&mut l, &e.app, &clock).await.unwrap();
    let log = sec.text();
    assert!(
        log.contains("\"event_type\":\"idp_event_pull_delayed\"")
            && log.contains("\"realm\":\"pasien\""),
        "{log}"
    );
    assert_eq!(position(&e.app, "pasien").await, Some(t0));
    assert_eq!(outbox(&e.relay).await.len(), 1);

    // Pulih: event yang tertahan terambil, tanpa kehilangan.
    e.kc.down.store(false, Ordering::SeqCst);
    let r = puller.round(&mut l, &e.app, &clock).await.unwrap()[0]
        .1
        .clone()
        .unwrap();
    assert_eq!((r.written, r.position_ms), (1, t0 + 5));
    assert_eq!(outbox(&e.relay).await.len(), 2);

    // Kredensial klien salah: ditolak endpoint token, tidak ada event yang ditulis, posisi tetap.
    push(
        &e.kc,
        "pasien",
        kc_event(t0 + 9, "LOGOUT", None, None, json!({})),
    );
    let mut wrong = Puller::new(
        options(&[IdpRealm::Pasien], 100),
        client(e.addr, "rahasia-salah"),
        clock.now(),
    );
    let res = wrong.round(&mut l, &e.app, &clock).await.unwrap();
    assert_eq!(
        res[0].1,
        Err(PullError::Keycloak(KeycloakError::Token(401)))
    );
    assert!(sec.text().contains("\"stage\":\"keycloak_auth\""));
    assert!(!sec.text().contains("rahasia-salah") && !sec.text().contains(SECRET_PASIEN));
    assert_eq!(position(&e.app, "pasien").await, Some(t0 + 5));
    assert_eq!(outbox(&e.relay).await.len(), 2);
    l.release().await.unwrap();
}

#[tokio::test]
async fn single_active_puller_and_wrong_lock_is_rejected() {
    let _serial = SERIAL.lock().await;
    let e = env().await;
    let l = lock().await;
    let other = PgConnection::connect(url("RS_TEST_DB_APP_URL").expose())
        .await
        .unwrap();
    assert!(matches!(
        SingletonLock::try_acquire(other, RS_KEYCLOAK_EVENT_PULLER)
            .await
            .unwrap(),
        LockAttempt::Busy(_)
    ));
    let conn = PgConnection::connect(url("RS_TEST_DB_APP_URL").expose())
        .await
        .unwrap();
    let LockAttempt::Acquired(mut foreign) = SingletonLock::try_acquire(conn, RS_PRIVACY_WORKER)
        .await
        .unwrap()
    else {
        panic!("kunci uji lain dipegang")
    };
    let clock = FixedClock::new(NOW);
    let mut puller = Puller::new(
        options(&[IdpRealm::Pasien], 100),
        client(e.addr, SECRET_PASIEN),
        clock.now(),
    );
    assert_eq!(
        puller
            .round(&mut foreign, &e.app, &clock)
            .await
            .unwrap_err(),
        PullError::WrongLock
    );
    assert_eq!(e.kc.event_requests.load(Ordering::SeqCst), 0);
    foreign.release().await.unwrap();
    l.release().await.unwrap();
}

#[tokio::test]
async fn invalid_event_stops_the_realm_without_skipping_it() {
    let _serial = SERIAL.lock().await;
    let e = env().await;
    let (sec, _guard) = capture();
    let t0 = ms(NOW) - 20_000;
    push(
        &e.kc,
        "pasien",
        kc_event(t0, "LOGOUT", None, None, json!({})),
    );
    let mut bad = kc_event(
        t0 + 1,
        "LOGIN_ERROR",
        None,
        Some("user_not_found"),
        json!({}),
    );
    bad["ipAddress"] = json!("bukan-alamat-ip");
    push(&e.kc, "pasien", bad);
    push(
        &e.kc,
        "pasien",
        kc_event(t0 + 2, "LOGOUT", None, None, json!({})),
    );
    let clock = FixedClock::new(NOW);
    let mut puller = Puller::new(
        options(&[IdpRealm::Pasien], 100),
        client(e.addr, SECRET_PASIEN),
        clock.now(),
    );
    let mut l = lock().await;
    let res = puller.round(&mut l, &e.app, &clock).await.unwrap();
    assert_eq!(res[0].1, Err(PullError::InvalidEvent("ipAddress")));
    assert_eq!(
        position(&e.app, "pasien").await,
        None,
        "posisi tidak maju melewati event yang gagal"
    );
    assert_eq!(
        outbox(&e.relay).await.len(),
        1,
        "event sebelum event rusak tercatat, sesudahnya tidak"
    );
    assert!(
        sec.text().contains("\"stage\":\"invalid_event\""),
        "{}",
        sec.text()
    );
    assert!(!sec.text().contains("bukan-alamat-ip"));
    l.release().await.unwrap();
}

/// F1-T65 (US-208 AC1/AC4): login pertama yang berhasil di realm `admin` mengaktifkan akun admin yang menunggu
/// aktivasi, dengan E-10 `admin_account_activated` di transaksi yang sama dengan E-01. Login gagal, login
/// kedua, dan akun admin nonaktif tidak berubah.
#[tokio::test]
async fn first_admin_login_activates_pending_admin_account() {
    let _serial = SERIAL.lock().await;
    let e = env().await;
    let pending_kc = new_id().hyphenated().to_string();
    let inactive_kc = new_id().hyphenated().to_string();
    let pending = insert_account(&e.app, "rs_admin", "admin", &pending_kc, ADMIN_EMAIL).await;
    let inactive = insert_account(&e.app, "rs_admin", "admin", &inactive_kc, ADMIN_EMAIL).await;
    for (id, st) in [(pending, "pending_activation"), (inactive, "inactive")] {
        sqlx::query!("UPDATE app.account SET status = $2 WHERE id = $1", id, st)
            .execute(&e.app)
            .await
            .unwrap();
    }
    sqlx::query!(
        "INSERT INTO app.rs_admin_role (id, account_id, sub_role) VALUES ($1, $2, 'super_admin')",
        new_id(),
        pending
    )
    .execute(&e.app)
    .await
    .unwrap();
    let t0 = ms(NOW) - 60_000;
    let pk = Some(pending_kc.as_str());
    let admin_login = |t: i64, user: Option<&str>, err: Option<&str>| {
        let kind = if err.is_some() {
            "LOGIN_ERROR"
        } else {
            "LOGIN"
        };
        let mut ev = kc_event(t, kind, user, err, login_details(ADMIN_EMAIL));
        ev["clientId"] = json!("rs-backoffice-bff");
        ev
    };
    push(
        &e.kc,
        "admin",
        admin_login(t0, pk, Some("invalid_user_credentials")),
    );
    push(
        &e.kc,
        "admin",
        admin_login(t0 + 1, Some(inactive_kc.as_str()), None),
    );
    push(&e.kc, "admin", admin_login(t0 + 2, pk, None));
    push(&e.kc, "admin", admin_login(t0 + 3, pk, None));

    let mut l = lock().await;
    let clock = FixedClock::new(NOW);
    let mut puller = Puller::new(
        options(&[IdpRealm::Admin], 100),
        client(e.addr, SECRET_PASIEN),
        clock.now(),
    );
    puller.round(&mut l, &e.app, &clock).await.unwrap();
    l.release().await.unwrap();

    for (id, want) in [(pending, "active"), (inactive, "inactive")] {
        let st = sqlx::query_scalar!("SELECT status FROM app.account WHERE id = $1", id)
            .fetch_one(&e.app)
            .await
            .unwrap();
        assert_eq!(st, want);
    }
    let rows = outbox(&e.relay).await;
    let activated = by_action(&rows, "admin_account_activated");
    assert_eq!(activated.len(), 1, "hanya login pertama yang berhasil");
    let a = activated[0];
    assert_eq!(a["event_code"], "e_10");
    assert_eq!(a["object_id"], pending.to_string());
    assert_eq!(a["actor_account_id"], pending.to_string());
    assert_eq!(a["before_values"]["status"], "pending_activation");
    assert_eq!(a["after_values"]["status"], "active");
    assert_eq!(a["source_ip"], "203.0.113.7");
    assert_eq!(by_action(&rows, "login").len(), 3);
    assert!(!serde_json::to_string(&rows).unwrap().contains(ADMIN_EMAIL));
}
