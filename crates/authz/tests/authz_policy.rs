//! Test otorisasi endpoint lewat HTTP sungguhan di localhost (F1-T37, T1): default deny dan pemeriksaan
//! saat start (KEPUTUSAN-127), realm per endpoint, status akun, sub-peran admin (UP-18), 403 vs 404
//! (ADR-012 butir 4, KU-03), dan security log `access_denied`. Token dari kunci uji (tanpa Keycloak).

#![allow(clippy::unwrap_used, clippy::expect_used)] // berkas test

use authn::testing::{StaticJwks, TestRealmKey, jwks_document};
use authn::{
    Algorithm, AuthContext, AuthState, Authenticator, AuthnConfig, Backend, ErrorRenderer, Realm,
    RealmConfig,
};
use authz::{
    AccountRecord, AccountStatus, AccountType, AdminSubRole, Enforcement, Policy, PolicyRegistry,
    PolicyRouter, PolicySetupError, Principal, PrincipalResolver, PublicReason, ResolveError,
    ResolveFuture, check_coverage, visible_object,
};
use axum::extract::Path;
use axum::http::header::CONTENT_TYPE;
use axum::response::{IntoResponse, Response};
use envelope::{ApiError, CommonError};
use foundation::{CorrelationId, FixedClock, Uuid};
use observability::LogLevel;
use observability::log::{Component, JsonLogLayer, LogSink, MemorySink, subscriber};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use time::OffsetDateTime;
use time::macros::datetime;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

const NOW: OffsetDateTime = datetime!(2027-03-01 08:00:00 UTC);
const CID: &str = "01936f8e-9c01-7e02-8f03-1a2b3c4d5e6f";

fn iss(realm: &str) -> String {
    format!("https://auth.ruang-sehat.invalid/realms/{realm}")
}

fn aud(realm: &str) -> String {
    format!("rs-api-{realm}")
}

// ----- handler uji -----

#[utoipa::path(get, path = "/health", responses((status = 200)))]
async fn health() -> &'static str {
    "ok"
}

#[utoipa::path(get, path = "/patient/v1/me", responses((status = 200)))]
async fn me(p: Principal) -> String {
    format!("{}:{}", p.account_type(), p.account_id())
}

#[utoipa::path(get, path = "/practitioner/v1/sessions", responses((status = 200)))]
async fn sessions(p: Principal) -> String {
    p.account_type().to_string()
}

#[utoipa::path(get, path = "/backoffice/v1/verifications", responses((status = 200)))]
async fn verifications(p: Principal) -> String {
    format!("verifier-ok:{}", p.account_id())
}

#[utoipa::path(get, path = "/backoffice/v1/audit-logs", responses((status = 200)))]
async fn audit_logs(_p: Principal) -> &'static str {
    "audit-ok"
}

#[utoipa::path(get, path = "/portal/v1/reports", responses((status = 200)))]
async fn reports(_p: Principal) -> &'static str {
    "portal-ok"
}

#[utoipa::path(get, path = "/patient/v1/orders/{order_id}", params(("order_id" = String, Path)), responses((status = 200)))]
async fn order(p: Principal, Path(order_id): Path<String>) -> Response {
    // Simpanan uji: pesanan "o-1" milik akun PATIENT_A; "o-2" milik PATIENT_B.
    let owners: HashMap<&str, Uuid> = HashMap::from([("o-1", patient_a()), ("o-2", patient_b())]);
    let found = owners.get(order_id.as_str()).copied();
    match visible_object(&p, "/patient/v1/orders/{order_id}", found, |owner| {
        *owner == p.account_id()
    }) {
        Ok(_) => format!("order:{order_id}").into_response(),
        Err(e) => render_common(e.to_common_error()),
    }
}

#[utoipa::path(get, path = "/public/v1/public-only", responses((status = 200)))]
async fn principal_on_public(_p: Principal) -> &'static str {
    "tidak boleh sampai sini"
}

// ----- perangkat -----

/// UUID uji berpola heksadesimal campuran (deret angka panjang akan diredaksi logger sebagai
/// kemungkinan NIK/nomor telepon).
fn uuid(n: u128) -> Uuid {
    Uuid::from_u128(0x0193_6f8e_9c01_7e02_8f03_1a2b_3c4d_5e00 | (n & 0xff))
}
fn patient_a() -> Uuid {
    uuid(0xA)
}
fn patient_b() -> Uuid {
    uuid(0xB)
}

fn render_common(error: CommonError) -> Response {
    let cid = CorrelationId::parse(CID).unwrap();
    let body = error.to_error_body(cid, "https://d.example/problems");
    (
        error.http_status(),
        [(CONTENT_TYPE, "application/problem+json")],
        serde_json::to_vec(&body).unwrap(),
    )
        .into_response()
}

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
struct MemResolver {
    accounts: Mutex<HashMap<(Realm, String), AccountRecord>>,
    failing: Mutex<bool>,
}

impl MemResolver {
    fn add(&self, realm: Realm, sub: &str, record: AccountRecord) {
        self.accounts
            .lock()
            .unwrap()
            .insert((realm, sub.to_string()), record);
    }
}

impl PrincipalResolver for MemResolver {
    fn resolve<'a>(&'a self, ctx: &'a AuthContext) -> ResolveFuture<'a> {
        let failing = *self.failing.lock().unwrap();
        let rec = self
            .accounts
            .lock()
            .unwrap()
            .get(&(ctx.realm(), ctx.subject().to_string()))
            .cloned();
        Box::pin(async move { if failing { Err(ResolveError) } else { Ok(rec) } })
    }
}

fn record(
    id: Uuid,
    t: AccountType,
    status: AccountStatus,
    roles: &[AdminSubRole],
) -> AccountRecord {
    AccountRecord {
        account_id: id,
        account_type: t,
        status,
        sub_roles: roles.to_vec(),
    }
}

struct Keys {
    pasien: TestRealmKey,
    nakes: TestRealmKey,
    admin: TestRealmKey,
    rme: TestRealmKey,
}

fn authenticator(keys: &Keys, realms: &[Realm]) -> Authenticator {
    let cfgs = realms
        .iter()
        .map(|r| {
            let key = match r {
                Realm::Pasien => &keys.pasien,
                Realm::Nakes => &keys.nakes,
                _ => &keys.admin,
            };
            RealmConfig {
                realm: *r,
                issuer: iss(r.as_str()),
                audiences: vec![aud(r.as_str())],
                algorithms: vec![Algorithm::RS256],
                authorized_parties: vec![],
                jwks: Arc::new(StaticJwks::new(jwks_document(&[key.jwk()]))),
            }
        })
        .collect();
    Authenticator::new(
        AuthnConfig::with_defaults(Backend::RuangSehat, cfgs),
        Arc::new(FixedClock::new(NOW)),
    )
    .unwrap()
}

fn app_router() -> PolicyRouter {
    PolicyRouter::new()
        .routes(Policy::Public(PublicReason::Operational), routes!(health))
        .routes(Policy::patient(), routes!(me))
        .routes(Policy::patient(), routes!(order))
        .routes(Policy::health_worker(), routes!(sessions))
        .routes(
            Policy::rs_admin(&[AdminSubRole::Verifier]),
            routes!(verifications),
        )
        .routes(
            Policy::rs_admin(&[AdminSubRole::Auditor, AdminSubRole::SuperAdmin]),
            routes!(audit_logs),
        )
        .routes(Policy::partner_admin(), routes!(reports))
        .routes(
            Policy::Public(PublicReason::PublicContent),
            routes!(principal_on_public),
        )
}

fn token(key: &TestRealmKey, realm: &str, sub: &str) -> String {
    let now = NOW.unix_timestamp();
    key.sign(&json!({
        "iss": iss(realm), "aud": aud(realm), "sub": sub, "typ": "Bearer",
        "sid": "sesi-uji", "azp": "app", "iat": now - 5, "exp": now + 300,
    }))
    .unwrap()
}

async fn spawn(keys: &Keys, resolver: Arc<MemResolver>) -> SocketAddr {
    let auth = AuthState::new(
        Arc::new(authenticator(
            keys,
            &[Realm::Pasien, Realm::Nakes, Realm::Admin],
        )),
        Arc::new(Renderer),
    );
    let (router, _spec) = app_router()
        .finish(Enforcement::new(auth, resolver))
        .unwrap();
    let app = router.layer(axum::middleware::from_fn(
        observability::correlation_middleware,
    ));
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    addr
}

struct Resp {
    status: u16,
    body: String,
}

impl Resp {
    fn json(&self) -> Value {
        serde_json::from_str(&self.body).unwrap_or(Value::Null)
    }
}

async fn call(addr: SocketAddr, method: &str, path: &str, bearer: Option<&str>) -> Resp {
    let mut s = TcpStream::connect(addr).await.unwrap();
    let mut req = format!(
        "{method} {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\nContent-Length: 0\r\nX-Correlation-Id: {CID}\r\n"
    );
    if let Some(t) = bearer {
        req.push_str(&format!("Authorization: Bearer {t}\r\n"));
    }
    req.push_str("\r\n");
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

fn keys() -> Keys {
    Keys {
        pasien: TestRealmKey::generate("pasien-k").unwrap(),
        nakes: TestRealmKey::generate("nakes-k").unwrap(),
        admin: TestRealmKey::generate("admin-k").unwrap(),
        rme: TestRealmKey::generate("rme-k").unwrap(),
    }
}

fn capture() -> (MemorySink, MemorySink, tracing::subscriber::DefaultGuard) {
    let app = MemorySink::default();
    let sec = MemorySink::default();
    let layer = JsonLogLayer::new(
        Component::Api,
        LogLevel::Info,
        Arc::new(FixedClock::new(NOW)),
    )
    .with_sinks(
        Arc::new(app.clone()) as Arc<dyn LogSink>,
        Arc::new(sec.clone()) as Arc<dyn LogSink>,
    );
    let guard = tracing::subscriber::set_default(subscriber(layer));
    (app, sec, guard)
}

// ----- pemeriksaan saat start (KEPUTUSAN-127) -----

#[test]
fn route_without_policy_fails_startup_check() {
    // Rute dipasang langsung di OpenApiRouter (melewati PolicyRouter): cakupan gagal.
    let raw: OpenApiRouter = OpenApiRouter::new().routes(routes!(health));
    let (_router, spec) = raw.split_for_parts();
    assert_eq!(
        check_coverage(&spec, &PolicyRegistry::default())
            .err()
            .unwrap(),
        PolicySetupError::MissingPolicy {
            method: "GET".into(),
            path: "/health".into()
        }
    );
    // Dokumen dasar yang sudah memuat path tanpa kebijakan juga gagal di finish.
    let base = OpenApiRouter::<()>::new()
        .routes(routes!(me))
        .into_openapi();
    let err = PolicyRouter::<()>::with_openapi(base)
        .routes(Policy::Public(PublicReason::Operational), routes!(health))
        .finish(Enforcement::public_only(Arc::new(Renderer)))
        .err()
        .unwrap();
    assert_eq!(
        err,
        PolicySetupError::MissingPolicy {
            method: "GET".into(),
            path: "/patient/v1/me".into()
        }
    );
}

#[test]
fn stale_duplicate_and_unsatisfiable_policies_fail_startup() {
    let mut reg = PolicyRegistry::default();
    reg.insert(
        &axum::http::Method::GET,
        "/hilang",
        Policy::Public(PublicReason::Operational),
    )
    .unwrap();
    let spec = OpenApiRouter::<()>::new().into_openapi();
    assert!(matches!(
        check_coverage(&spec, &reg).err().unwrap(),
        PolicySetupError::StalePolicy { .. }
    ));

    let err = PolicyRouter::<()>::new()
        .routes(Policy::Public(PublicReason::Operational), routes!(health))
        .merge(PolicyRouter::new().routes(Policy::patient(), routes!(health)))
        .finish(Enforcement::public_only(Arc::new(Renderer)))
        .err()
        .unwrap();
    assert!(
        matches!(err, PolicySetupError::DuplicatePolicy { .. }),
        "{err}"
    );

    let keys = keys();
    let auth = AuthState::new(
        Arc::new(authenticator(&keys, &[Realm::Admin])),
        Arc::new(Renderer),
    );
    let err = PolicyRouter::<()>::new()
        .routes(Policy::rs_admin(&[]), routes!(verifications))
        .finish(Enforcement::new(
            auth.clone(),
            Arc::new(MemResolver::default()),
        ))
        .err()
        .unwrap();
    assert!(
        matches!(err, PolicySetupError::UnsatisfiablePolicy { .. }),
        "{err}"
    );
    let err = PolicyRouter::<()>::new()
        .routes(
            Policy::accounts(&[AccountType::RsAdmin]),
            routes!(verifications),
        )
        .finish(Enforcement::new(auth, Arc::new(MemResolver::default())))
        .err()
        .unwrap();
    assert!(
        matches!(err, PolicySetupError::UnsatisfiablePolicy { .. }),
        "{err}"
    );
}

#[test]
fn authenticated_route_requires_configured_authenticator_and_realm() {
    let err = PolicyRouter::<()>::new()
        .routes(Policy::patient(), routes!(me))
        .finish(Enforcement::public_only(Arc::new(Renderer)))
        .err()
        .unwrap();
    assert_eq!(err, PolicySetupError::MissingAuthentication);

    let keys = keys();
    let auth = AuthState::new(
        Arc::new(authenticator(&keys, &[Realm::Pasien])),
        Arc::new(Renderer),
    );
    let err = PolicyRouter::<()>::new()
        .routes(Policy::health_worker(), routes!(sessions))
        .finish(Enforcement::new(auth, Arc::new(MemResolver::default())))
        .err()
        .unwrap();
    assert_eq!(err, PolicySetupError::RealmNotConfigured { realm: "nakes" });
}

// ----- penegakan saat runtime -----

#[tokio::test(flavor = "current_thread")]
async fn default_deny_unknown_routes_and_methods_return_uniform_404() {
    let keys = keys();
    let addr = spawn(&keys, Arc::new(MemResolver::default())).await;
    let unknown = call(addr, "GET", "/patient/v1/tidak-ada", None).await;
    assert_eq!(unknown.status, 404);
    assert_eq!(unknown.json()["response_code"], "4040001");
    assert_eq!(unknown.json()["correlation_id"], CID);
    // Rute ada tetapi metode tidak terdaftar: tidak punya kebijakan → 404, handler tidak berjalan.
    let post = call(addr, "POST", "/patient/v1/me", None).await;
    assert_eq!(post.status, 404);
    assert_eq!(post.body, unknown.body);
    let delete = call(addr, "DELETE", "/health", None).await;
    assert_eq!(delete.body, unknown.body);
    // Endpoint publik terbuka tanpa token; HEAD memakai kebijakan GET.
    assert_eq!(call(addr, "GET", "/health", None).await.status, 200);
    assert_eq!(call(addr, "HEAD", "/health", None).await.status, 200);
}

#[tokio::test(flavor = "current_thread")]
async fn handler_on_public_route_cannot_obtain_principal() {
    let keys = keys();
    let addr = spawn(&keys, Arc::new(MemResolver::default())).await;
    let r = call(addr, "GET", "/public/v1/public-only", None).await;
    assert_eq!(r.status, 500);
    assert_eq!(r.json()["response_code"], "5000001");
    assert!(!r.body.contains("tidak boleh sampai sini"));
}

#[tokio::test(flavor = "current_thread")]
async fn missing_invalid_or_other_realm_token_is_401() {
    let keys = keys();
    let resolver = Arc::new(MemResolver::default());
    resolver.add(
        Realm::Pasien,
        "p-a",
        record(
            patient_a(),
            AccountType::Patient,
            AccountStatus::Active,
            &[],
        ),
    );
    resolver.add(
        Realm::Nakes,
        "n-1",
        record(
            uuid(0x1),
            AccountType::HealthWorker,
            AccountStatus::Active,
            &[],
        ),
    );
    let addr = spawn(&keys, resolver).await;

    let ok = call(
        addr,
        "GET",
        "/patient/v1/me",
        Some(&token(&keys.pasien, "pasien", "p-a")),
    )
    .await;
    assert_eq!(ok.status, 200);
    assert_eq!(ok.body, format!("patient:{}", patient_a()));

    let cases = [
        None,
        Some("bukan.token.sah".to_string()),
        // Token realm nakes yang sah ke endpoint pasien.
        Some(token(&keys.nakes, "nakes", "n-1")),
        // Token realm rme (issuer tidak dikonfigurasi backend Ruang Sehat).
        Some(token(&keys.rme, "rme", "r-1")),
        // Token realm pasien yang ditandatangani kunci lain.
        Some(token(&keys.admin, "pasien", "p-a")),
    ];
    let mut bodies = Vec::new();
    for t in &cases {
        let r = call(addr, "GET", "/patient/v1/me", t.as_deref()).await;
        assert_eq!(r.status, 401, "{t:?}");
        assert_eq!(r.json()["response_code"], "4010001");
        bodies.push(r.body);
    }
    assert!(bodies.windows(2).all(|w| w[0] == w[1]));
    // Token pasien ke endpoint tenaga kesehatan.
    let r = call(
        addr,
        "GET",
        "/practitioner/v1/sessions",
        Some(&token(&keys.pasien, "pasien", "p-a")),
    )
    .await;
    assert_eq!(r.status, 401);
}

#[tokio::test(flavor = "current_thread")]
async fn unknown_inactive_or_mismatched_account_is_401_and_resolver_failure_is_503() {
    let keys = keys();
    let resolver = Arc::new(MemResolver::default());
    resolver.add(
        Realm::Pasien,
        "inactive",
        record(uuid(1), AccountType::Patient, AccountStatus::Inactive, &[]),
    );
    resolver.add(
        Realm::Pasien,
        "pending",
        record(
            uuid(2),
            AccountType::Patient,
            AccountStatus::PendingActivation,
            &[],
        ),
    );
    // Data akun tidak konsisten: subjek realm pasien terpetakan ke admin.
    resolver.add(
        Realm::Pasien,
        "mismatch",
        record(
            uuid(3),
            AccountType::RsAdmin,
            AccountStatus::Active,
            &[AdminSubRole::SuperAdmin],
        ),
    );
    let addr = spawn(&keys, resolver.clone()).await;
    for sub in ["tidak-ada", "inactive", "pending", "mismatch"] {
        let r = call(
            addr,
            "GET",
            "/patient/v1/me",
            Some(&token(&keys.pasien, "pasien", sub)),
        )
        .await;
        assert_eq!(r.status, 401, "{sub}");
        assert_eq!(r.json()["response_code"], "4010001");
    }
    *resolver.failing.lock().unwrap() = true;
    let r = call(
        addr,
        "GET",
        "/patient/v1/me",
        Some(&token(&keys.pasien, "pasien", "inactive")),
    )
    .await;
    assert_eq!(r.status, 503);
    assert_eq!(r.json()["response_code"], "5030001");
}

#[tokio::test(flavor = "current_thread")]
async fn admin_sub_roles_are_enforced_without_implicit_superuser() {
    let (_app_log, sec_log, _guard) = capture();
    let keys = keys();
    let resolver = Arc::new(MemResolver::default());
    let admins: [(&str, AccountType, &[AdminSubRole]); 6] = [
        ("verifier", AccountType::RsAdmin, &[AdminSubRole::Verifier]),
        ("content", AccountType::RsAdmin, &[AdminSubRole::Content]),
        ("super", AccountType::RsAdmin, &[AdminSubRole::SuperAdmin]),
        ("auditor", AccountType::RsAdmin, &[AdminSubRole::Auditor]),
        ("none", AccountType::RsAdmin, &[]),
        // Admin rekanan yang (keliru) membawa sub-peran: sub-peran dibuang, jenis akun tetap ditolak.
        (
            "partner",
            AccountType::PartnerAdmin,
            &[AdminSubRole::Verifier],
        ),
    ];
    for (i, (sub, t, roles)) in admins.iter().enumerate() {
        resolver.add(
            Realm::Admin,
            sub,
            record(uuid(100 + i as u128), *t, AccountStatus::Active, roles),
        );
    }
    let addr = spawn(&keys, resolver).await;
    let t = |sub: &str| token(&keys.admin, "admin", sub);

    let expect = [
        ("/backoffice/v1/verifications", "verifier", 200),
        ("/backoffice/v1/verifications", "content", 403),
        ("/backoffice/v1/verifications", "super", 403),
        ("/backoffice/v1/verifications", "none", 403),
        ("/backoffice/v1/verifications", "partner", 403),
        ("/backoffice/v1/audit-logs", "auditor", 200),
        ("/backoffice/v1/audit-logs", "super", 200),
        ("/backoffice/v1/audit-logs", "verifier", 403),
        ("/portal/v1/reports", "partner", 200),
        ("/portal/v1/reports", "super", 403),
    ];
    for (path, sub, status) in expect {
        let r = call(addr, "GET", path, Some(&t(sub))).await;
        assert_eq!(r.status, status, "{path} {sub}");
        if status == 403 {
            assert_eq!(r.json()["response_code"], "4030001");
            assert_eq!(r.json()["correlation_id"], CID);
        }
    }
    // Token admin ke endpoint pasien: realm lain → 401.
    let r = call(addr, "GET", "/patient/v1/me", Some(&t("super"))).await;
    assert_eq!(r.status, 401);

    let sec = sec_log.text();
    let denied = sec
        .lines()
        .filter(|l| {
            l.contains("\"event_type\":\"access_denied\"") && l.contains("role_not_allowed")
        })
        .count();
    assert_eq!(denied, 6, "{sec}");
    assert!(sec.contains("/backoffice/v1/verifications"));
    assert!(sec.contains(&uuid(101).hyphenated().to_string()));
    assert!(!sec.contains("sesi-uji"));
}

#[tokio::test(flavor = "current_thread")]
async fn foreign_object_is_indistinguishable_from_missing_and_logged() {
    let (_app_log, sec_log, _guard) = capture();
    let keys = keys();
    let resolver = Arc::new(MemResolver::default());
    resolver.add(
        Realm::Pasien,
        "p-a",
        record(
            patient_a(),
            AccountType::Patient,
            AccountStatus::Active,
            &[],
        ),
    );
    let addr = spawn(&keys, resolver).await;
    let t = token(&keys.pasien, "pasien", "p-a");

    let own = call(addr, "GET", "/patient/v1/orders/o-1", Some(&t)).await;
    assert_eq!(own.status, 200);
    assert_eq!(own.body, "order:o-1");
    let foreign = call(addr, "GET", "/patient/v1/orders/o-2", Some(&t)).await;
    let missing = call(addr, "GET", "/patient/v1/orders/o-404", Some(&t)).await;
    assert_eq!(foreign.status, 404);
    assert_eq!(missing.status, 404);
    assert_eq!(foreign.json()["response_code"], "4040001");
    // KU-03: isi respons identik untuk "milik orang lain" dan "tidak ada".
    assert_eq!(foreign.body, missing.body);

    let sec = sec_log.text();
    let lines: Vec<&str> = sec
        .lines()
        .filter(|l| l.contains("foreign_resource_id"))
        .collect();
    assert_eq!(lines.len(), 1, "{sec}");
    let line = lines.first().unwrap();
    assert!(line.contains("/patient/v1/orders/{order_id}"));
    assert!(
        line.contains(&patient_a().hyphenated().to_string()),
        "{line}"
    );
    assert!(!sec.contains("o-2"), "ID objek tidak masuk log: {sec}");
}
