//! Relasi kepemilikan pasien atas penerima layanan lewat HTTP sungguhan (F1-T38, T1): pemilik dibaca
//! modul `profile` dari PostgreSQL (`scripts/db-ephemeral.sh`, role runtime `rs_api`), keputusan
//! `authz::care_recipient_access`, dan penolakan atas data sensitif dicatat sebagai event audit E-05 hasil
//! gagal di `audit_outbox` (dibaca lewat role relay) oleh `AuditDenialRecorder`. Tanpa database, test
//! GAGAL.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)] // berkas test

use authn::testing::{StaticJwks, TestRealmKey, jwks_document};
use authn::{
    Algorithm, AuthContext, AuthState, Authenticator, AuthnConfig, Backend, ErrorRenderer, Realm,
    RealmConfig,
};
use authz::{
    AccountRecord, AccountStatus, AccountType, ClientIp, DenialAuditor, DenialSink, Enforcement,
    Policy, PolicyRouter, Principal, PrincipalResolver, ResolveFuture, Sensitivity,
    care_recipient_access,
};
use axum::extract::{Path, State};
use axum::http::header::CONTENT_TYPE;
use axum::response::{IntoResponse, Response};
use envelope::{ApiError, CommonError};
use foundation::{Clock, CorrelationId, FixedClock, SystemClock, Uuid, new_id};
use observability::EnvSource;
use rs_backend::access_audit::AuditDenialRecorder;
use rs_backend::modules::profile::api::care_recipient_owner;
use serde_json::{Value, json};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use utoipa_axum::routes;

const ISS: &str = "https://auth.ruang-sehat.invalid/realms/pasien";
const AUD: &str = "rs-api-pasien";
const CID: &str = "01936f8e-9c01-7e02-8f03-1a2b3c4d5e6f";
const ROUTE: &str = "/patient/v1/care-recipients/{care_recipient_id}";

async fn pool(var: &str) -> PgPool {
    let url = EnvSource::from_process()
        .required_secret(var)
        .unwrap_or_else(|_| {
            panic!(
                "{var} tidak diset: test relasi wajib dijalankan lewat \
                 `bash scripts/db-ephemeral.sh cargo test ...` (dipakai scripts/verify.sh)"
            )
        });
    PgPoolOptions::new()
        .max_connections(4)
        .connect(url.expose())
        .await
        .unwrap_or_else(|_| panic!("koneksi {var} gagal"))
}

fn random_hex(n: usize) -> String {
    let mut b = vec![0u8; n];
    crypto::fill_random(&mut b).unwrap();
    b.iter().map(|x| format!("{x:02x}")).collect()
}

/// Pasien sintetis: akun, baris `patient`, penerima layanan pemilik akun dan satu anggota keluarga.
struct Family {
    account_id: Uuid,
    subject: String,
    owner_recipient: Uuid,
    child_recipient: Uuid,
}

async fn new_family(pool: &PgPool) -> Family {
    let account_id = new_id();
    let subject = format!("uji-{}", random_hex(8));
    let email = format!("{subject}@example.com");
    sqlx::query!(
        "INSERT INTO app.account (id, account_type, idp_realm, idp_subject, email, status, status_changed_at) \
         VALUES ($1, 'patient', 'pasien', $2, $3, 'active', now())",
        account_id,
        subject,
        email,
    )
    .execute(pool)
    .await
    .unwrap();
    let patient_id = new_id();
    sqlx::query!(
        "INSERT INTO app.patient (id, account_id, processing_status) VALUES ($1, $2, 'normal')",
        patient_id,
        account_id,
    )
    .execute(pool)
    .await
    .unwrap();
    let owner_recipient = new_id();
    let child_recipient = new_id();
    sqlx::query!(
        "INSERT INTO app.care_recipient (id, patient_id, is_account_owner, relationship, full_name, birth_date, sex) \
         VALUES ($1, $3, true, NULL, 'Pasien Uji', DATE '1990-01-01', 'female'), \
                ($2, $3, false, 'child', 'Anak Uji', DATE '2020-01-01', 'male')",
        owner_recipient,
        child_recipient,
        patient_id,
    )
    .execute(pool)
    .await
    .unwrap();
    Family {
        account_id,
        subject,
        owner_recipient,
        child_recipient,
    }
}

async fn soft_delete_recipient(pool: &PgPool, id: Uuid) {
    sqlx::query!(
        "UPDATE app.care_recipient SET deleted_at = now(), delete_reason = 'uji' WHERE id = $1",
        id
    )
    .execute(pool)
    .await
    .unwrap();
}

/// Event E-05 di `audit_outbox` untuk subjek `subject` (role relay punya SELECT).
async fn e05_events(relay: &PgPool, subject: Uuid) -> Vec<Value> {
    sqlx::query_scalar!(
        r#"SELECT payload AS "payload!" FROM app.audit_outbox
           WHERE payload ->> 'event_code' = 'e_05'
             AND payload -> 'subjects' -> 0 ->> 'subject_id' = $1"#,
        subject.hyphenated().to_string(),
    )
    .fetch_all(relay)
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

#[derive(Default)]
struct MemResolver(Mutex<HashMap<String, Uuid>>);

impl PrincipalResolver for MemResolver {
    fn resolve<'a>(&'a self, ctx: &'a AuthContext) -> ResolveFuture<'a> {
        let id = self.0.lock().unwrap().get(ctx.subject()).copied();
        Box::pin(async move {
            Ok(id.map(|account_id| AccountRecord {
                account_id,
                account_type: AccountType::Patient,
                status: AccountStatus::Active,
                sub_roles: vec![],
            }))
        })
    }
}

#[derive(Clone)]
struct TestState {
    pool: PgPool,
}

fn not_found(cid: CorrelationId) -> Response {
    Renderer.render(CommonError::NotFound, cid)
}

#[utoipa::path(get, path = "/patient/v1/care-recipients/{care_recipient_id}", params(("care_recipient_id" = String, Path)), responses((status = 200)))]
async fn care_recipient(
    State(s): State<TestState>,
    p: Principal,
    Path(raw): Path<String>,
) -> Response {
    let Ok(id) = Uuid::parse_str(&raw) else {
        return not_found(p.correlation_id());
    };
    let owner = match care_recipient_owner(&s.pool, id).await {
        Ok(o) => o,
        Err(_) => return Renderer.render(CommonError::Unavailable, p.correlation_id()),
    };
    match care_recipient_access(&p, ROUTE, id, owner, Sensitivity::Sensitive) {
        Ok(owned) => format!("cr:{}", owned.care_recipient_id()).into_response(),
        Err(e) => Renderer.render(e.to_common_error(), p.correlation_id()),
    }
}

async fn with_client_ip(mut req: axum::extract::Request, next: axum::middleware::Next) -> Response {
    req.extensions_mut()
        .insert(ClientIp(IpAddr::V4(Ipv4Addr::new(198, 51, 100, 9))));
    next.run(req).await
}

async fn spawn(pool: PgPool, key: &TestRealmKey, resolver: Arc<MemResolver>) -> SocketAddr {
    let now = SystemClock.now();
    let auth = Authenticator::new(
        AuthnConfig::with_defaults(
            Backend::RuangSehat,
            vec![RealmConfig {
                realm: Realm::Pasien,
                issuer: ISS.into(),
                audiences: vec![AUD.into()],
                algorithms: vec![Algorithm::RS256],
                authorized_parties: vec![],
                jwks: Arc::new(StaticJwks::new(jwks_document(&[key.jwk()]))),
            }],
        ),
        Arc::new(FixedClock::new(now)),
    )
    .unwrap();
    let recorder: Arc<dyn DenialAuditor> = Arc::new(AuditDenialRecorder::new(
        pool.clone(),
        Arc::new(SystemClock),
    ));
    let denials = DenialSink::start(64, 2, Some(recorder)).unwrap();
    let (router, _) = PolicyRouter::<TestState>::new()
        .routes(Policy::patient(), routes!(care_recipient))
        .finish(Enforcement::new(
            AuthState::new(Arc::new(auth), Arc::new(Renderer)),
            resolver,
            denials,
        ))
        .unwrap();
    let app = router
        .with_state(TestState { pool })
        .layer(axum::middleware::from_fn(with_client_ip))
        .layer(axum::middleware::from_fn(
            observability::correlation_middleware,
        ));
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { app.serve(listener, std::future::pending()).await.unwrap() });
    addr
}

fn token(key: &TestRealmKey, sub: &str) -> String {
    let t = SystemClock.now().unix_timestamp();
    key.sign(&json!({
        "iss": ISS, "aud": AUD, "sub": sub, "typ": "Bearer", "sid": "sid-uji",
        "azp": "app", "iat": t - 5, "exp": t + 300,
    }))
    .unwrap()
}

async fn get(addr: SocketAddr, id: Uuid, bearer: &str) -> (u16, String) {
    let mut s = TcpStream::connect(addr).await.unwrap();
    let req = format!(
        "GET /patient/v1/care-recipients/{id} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\
         Authorization: Bearer {bearer}\r\nX-Correlation-Id: {CID}\r\n\r\n"
    );
    s.write_all(req.as_bytes()).await.unwrap();
    let mut buf = Vec::new();
    s.read_to_end(&mut buf).await.unwrap();
    let text = String::from_utf8_lossy(&buf).into_owned();
    (
        text.get(9..12).unwrap().parse().unwrap(),
        text.split("\r\n\r\n")
            .nth(1)
            .unwrap_or_default()
            .to_string(),
    )
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn patient_reaches_own_recipients_only_and_foreign_attempts_are_audited_as_failed_e05() {
    // trace: KU-03, ADR-012 butir 1 dan 4, PRD §3.1 (E-05 hasil gagal), coding-standards §7.3 butir 5
    let app = pool("RS_TEST_DB_APP_URL").await;
    let relay = pool("RS_TEST_DB_RELAY_URL").await;
    let a = new_family(&app).await;
    let b = new_family(&app).await;
    let key = TestRealmKey::generate("k").unwrap();
    let resolver = Arc::new(MemResolver::default());
    resolver
        .0
        .lock()
        .unwrap()
        .insert(a.subject.clone(), a.account_id);
    let addr = spawn(app.clone(), &key, resolver).await;
    let t = token(&key, &a.subject);

    // Pemilik akun dan anggota keluarganya.
    assert_eq!(
        get(addr, a.owner_recipient, &t).await,
        (200, format!("cr:{}", a.owner_recipient))
    );
    assert_eq!(
        get(addr, a.child_recipient, &t).await,
        (200, format!("cr:{}", a.child_recipient))
    );
    // Ganti ID ke penerima layanan pasien B, ke ID acak, dan ke penerima layanan terhapus: 404 identik.
    let foreign = get(addr, b.child_recipient, &t).await;
    let missing = get(addr, new_id(), &t).await;
    soft_delete_recipient(&app, a.child_recipient).await;
    let deleted = get(addr, a.child_recipient, &t).await;
    assert_eq!(foreign.0, 404);
    assert_eq!(foreign, missing);
    assert_eq!(deleted, missing);
    assert!(!foreign.1.contains(&b.child_recipient.to_string()));

    // E-05 hasil gagal untuk percobaan atas data B, ditulis di latar belakang.
    let mut events = Vec::new();
    for _ in 0..100 {
        events = e05_events(&relay, b.child_recipient).await;
        if !events.is_empty() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert_eq!(events.len(), 1, "{events:?}");
    let e = &events[0];
    assert_eq!(e["outcome"], "failure");
    assert_eq!(e["outcome_reason_code"], "access_denied");
    assert_eq!(e["action_name"], "access");
    assert_eq!(
        e["actor_account_id"],
        json!(a.account_id.hyphenated().to_string())
    );
    assert_eq!(e["actor_account_type"], "patient");
    assert_eq!(e["object_type"], "care_recipient");
    assert_eq!(
        e["object_id"],
        json!(b.child_recipient.hyphenated().to_string())
    );
    assert_eq!(e["subjects"][0]["subject_kind"], "care_recipient");
    assert_eq!(e["source_component"], "patient_app");
    assert_eq!(e["source_ip"], "198.51.100.9");
    assert_eq!(e["correlation_id"], CID);
    // Akses sah dan objek yang tidak ada (atau sudah terhapus) tidak menghasilkan event penolakan.
    for subject in [a.owner_recipient, a.child_recipient, b.owner_recipient] {
        assert!(e05_events(&relay, subject).await.is_empty(), "{subject}");
    }
}
