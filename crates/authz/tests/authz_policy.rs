//! Test otorisasi endpoint lewat HTTP sungguhan di localhost (F1-T37, F1-T38, T1): default deny dan
//! pemeriksaan saat start (KEPUTUSAN-127), realm per endpoint, status akun, sub-peran admin (UP-18),
//! 403 vs 404 (ADR-012 butir 4, KU-03), relasi kepemilikan pasien atas penerima layanan, dan pencatatan
//! penolakan di latar belakang (security log `access_denied`, audit E-05 hasil gagal lewat
//! `DenialAuditor`). Token dari kunci uji (tanpa Keycloak).

#![allow(clippy::unwrap_used, clippy::expect_used)] // berkas test

use authn::testing::{StaticJwks, TestRealmKey, jwks_document};
use authn::{
    Algorithm, AuthContext, AuthState, Authenticator, AuthnConfig, Backend, ErrorRenderer, Realm,
    RealmConfig,
};
use authz::{
    AccountRecord, AccountStatus, AccountType, AdminSubRole, CareRecipientOwner, ClientIp,
    DenialAuditError, DenialAuditFuture, DenialAuditor, DenialSink, DeniedAccess, Enforcement,
    Policy, PolicyRegistry, PolicyRouter, PolicySetupError, PreparedDenialAudit, Principal,
    PrincipalResolver, PublicReason, ResolveError, ResolveFuture, RevocationFuture, Sensitivity,
    SessionGuard, care_recipient_access, check_coverage, visible_object,
};
use axum::extract::Path;
use axum::http::header::CONTENT_TYPE;
use axum::response::{IntoResponse, Response};
use envelope::{ApiError, CommonError};
use foundation::{CorrelationId, FixedClock, Uuid};
use observability::LogLevel;
use observability::log::{Component, JsonLogLayer, LogSink, MemorySink, subscriber};
use serde_json::{Value, json};
use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use time::OffsetDateTime;
use time::macros::datetime;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Semaphore;
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

/// Simpanan uji penerima layanan: (pemilik, pemilik akun sendiri?).
fn care_recipients() -> HashMap<Uuid, CareRecipientOwner> {
    HashMap::from([
        (
            uuid(0x21),
            CareRecipientOwner::new(uuid(0x21), patient_a(), true),
        ),
        (
            uuid(0x22),
            CareRecipientOwner::new(uuid(0x22), patient_a(), false),
        ),
        (
            uuid(0x23),
            CareRecipientOwner::new(uuid(0x23), patient_b(), true),
        ),
    ])
}

fn render_care_recipient(p: &Principal, route: &str, raw_id: &str) -> Response {
    // ID tidak valid diperlakukan sama dengan tidak ada.
    let Ok(id) = Uuid::parse_str(raw_id) else {
        return render_common(CommonError::NotFound);
    };
    let owner = care_recipients().get(&id).copied();
    match care_recipient_access(p, route, id, owner, Sensitivity::Sensitive) {
        Ok(owned) => format!(
            "cr:{}:{}",
            owned.care_recipient_id(),
            owned.is_account_owner()
        )
        .into_response(),
        Err(e) => render_common(e.to_common_error()),
    }
}

#[utoipa::path(get, path = "/patient/v1/care-recipients/{care_recipient_id}", params(("care_recipient_id" = String, Path)), responses((status = 200)))]
async fn care_recipient(p: Principal, Path(id): Path<String>) -> Response {
    render_care_recipient(&p, "/patient/v1/care-recipients/{care_recipient_id}", &id)
}

#[utoipa::path(get, path = "/practitioner/v1/care-recipients/{care_recipient_id}", params(("care_recipient_id" = String, Path)), responses((status = 200)))]
async fn care_recipient_by_health_worker(p: Principal, Path(id): Path<String>) -> Response {
    render_care_recipient(
        &p,
        "/practitioner/v1/care-recipients/{care_recipient_id}",
        &id,
    )
}

/// Pemanggil keliru: pemilik dibaca untuk penerima layanan milik sendiri (0x21), lalu dipakai untuk ID
/// yang diminta. `care_recipient_access` wajib menolak karena ID-nya berbeda (review F1-T38 butir 3).
#[utoipa::path(get, path = "/patient/v1/swapped-owner/{care_recipient_id}", params(("care_recipient_id" = String, Path)), responses((status = 200)))]
async fn swapped_owner(p: Principal, Path(raw): Path<String>) -> Response {
    let Ok(id) = Uuid::parse_str(&raw) else {
        return render_common(CommonError::NotFound);
    };
    let owner = care_recipients().get(&uuid(0x21)).copied();
    match care_recipient_access(
        &p,
        "/patient/v1/swapped-owner/{care_recipient_id}",
        id,
        owner,
        Sensitivity::Sensitive,
    ) {
        Ok(_) => "lolos".into_response(),
        Err(e) => render_common(e.to_common_error()),
    }
}

#[utoipa::path(get, path = "/public/v1/public-only", responses((status = 200)))]
async fn principal_on_public(_p: Principal) -> &'static str {
    "tidak boleh sampai sini"
}

#[utoipa::path(get, path = "/public/v1/context-only", responses((status = 200)))]
async fn context_on_public(ctx: AuthContext) -> String {
    format!("tidak boleh sampai sini:{}", ctx.subject())
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
    revoked: Mutex<HashSet<String>>,
    guard_failing: Mutex<bool>,
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

impl SessionGuard for MemResolver {
    fn is_revoked<'a>(&'a self, ctx: &'a AuthContext) -> RevocationFuture<'a> {
        let failing = *self.guard_failing.lock().unwrap();
        let revoked = ctx
            .session_id()
            .is_none_or(|sid| self.revoked.lock().unwrap().contains(sid));
        Box::pin(async move {
            if failing {
                Err(ResolveError)
            } else {
                Ok(revoked)
            }
        })
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
        .routes(
            Policy::Public(PublicReason::PublicContent),
            routes!(context_on_public),
        )
        .routes(Policy::patient(), routes!(care_recipient))
        .routes(Policy::patient(), routes!(swapped_owner))
        .routes(
            Policy::health_worker(),
            routes!(care_recipient_by_health_worker),
        )
}

/// Penulis audit penolakan dalam memori: mencatat panggilan, dapat dibuat gagal atau ditahan.
#[derive(Default)]
struct MemAuditor {
    seen: Mutex<Vec<DeniedAccess>>,
    prepares: AtomicUsize,
    calls: AtomicUsize,
    written_ids: Mutex<Vec<usize>>,
    fail_retryable: Mutex<usize>,
    permanent: Mutex<bool>,
    gate: Option<Arc<Semaphore>>,
}

/// Event uji yang dibentuk sekali: nomor urut pembentukan + penolakannya.
struct MemEvent(usize, DeniedAccess);

impl DenialAuditor for MemAuditor {
    fn prepare(&self, denial: &DeniedAccess) -> Result<PreparedDenialAudit, DenialAuditError> {
        let n = self.prepares.fetch_add(1, Ordering::SeqCst);
        Ok(PreparedDenialAudit::new(MemEvent(n, denial.clone())))
    }

    fn write<'a>(&'a self, prepared: &'a PreparedDenialAudit) -> DenialAuditFuture<'a> {
        Box::pin(async move {
            self.calls.fetch_add(1, Ordering::SeqCst);
            let MemEvent(n, denial) = prepared.get::<MemEvent>().unwrap();
            self.written_ids.lock().unwrap().push(*n);
            if let Some(g) = &self.gate {
                let _p = g.acquire().await.unwrap();
            }
            if *self.permanent.lock().unwrap() {
                return Err(DenialAuditError::Permanent);
            }
            {
                let mut left = self.fail_retryable.lock().unwrap();
                if *left > 0 {
                    *left -= 1;
                    return Err(DenialAuditError::Retryable);
                }
            }
            self.seen.lock().unwrap().push(denial.clone());
            Ok(())
        })
    }
}

fn sink(auditor: Option<Arc<MemAuditor>>) -> DenialSink {
    DenialSink::start(64, 2, auditor.map(|a| a as Arc<dyn DenialAuditor>)).unwrap()
}

async fn with_client_ip(mut req: axum::extract::Request, next: axum::middleware::Next) -> Response {
    req.extensions_mut()
        .insert(ClientIp(IpAddr::V4(Ipv4Addr::new(198, 51, 100, 7))));
    next.run(req).await
}

/// Menunggu sampai `cond` benar (pencatatan di latar belakang), paling lama 3 detik.
async fn eventually(mut cond: impl FnMut() -> bool) -> bool {
    for _ in 0..300 {
        if cond() {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    cond()
}

fn token(key: &TestRealmKey, realm: &str, sub: &str) -> String {
    token_sid(key, realm, sub, "sesi-uji")
}

fn token_sid(key: &TestRealmKey, realm: &str, sub: &str, sid: &str) -> String {
    let now = NOW.unix_timestamp();
    key.sign(&json!({
        "iss": iss(realm), "aud": aud(realm), "sub": sub, "typ": "Bearer",
        "sid": sid, "azp": "app", "iat": now - 5, "exp": now + 300,
    }))
    .unwrap()
}

async fn spawn(keys: &Keys, resolver: Arc<MemResolver>) -> SocketAddr {
    spawn_with(keys, resolver, sink(None)).await
}

async fn spawn_with(keys: &Keys, resolver: Arc<MemResolver>, denials: DenialSink) -> SocketAddr {
    let auth = AuthState::new(
        Arc::new(authenticator(
            keys,
            &[Realm::Pasien, Realm::Nakes, Realm::Admin],
        )),
        Arc::new(Renderer),
    );
    let (router, _spec) = app_router()
        .finish(Enforcement::new(auth, resolver.clone(), resolver, denials))
        .unwrap();
    let app = router
        .layer(axum::middleware::from_fn(with_client_ip))
        .layer(axum::middleware::from_fn(
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
            Arc::new(MemResolver::default()),
            sink(None),
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
        .finish(Enforcement::new(
            auth,
            Arc::new(MemResolver::default()),
            Arc::new(MemResolver::default()),
            sink(None),
        ))
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
        .finish(Enforcement::new(
            auth,
            Arc::new(MemResolver::default()),
            Arc::new(MemResolver::default()),
            sink(None),
        ))
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
async fn auth_context_extractor_on_public_route_is_500_even_with_valid_token() {
    // TD-15 (Medium): sebelumnya ekstraktor AuthContext memvalidasi token sendiri di endpoint publik,
    // melewati resolver akun (akun nonaktif atau tidak dikenal tetap lolos). Kini hanya konteks dari
    // middleware yang diterima; endpoint publik tidak punya konteks → 500 envelope.
    let keys = keys();
    let resolver = Arc::new(MemResolver::default());
    resolver.add(
        Realm::Pasien,
        "inactive",
        record(uuid(1), AccountType::Patient, AccountStatus::Inactive, &[]),
    );
    let addr = spawn(&keys, resolver).await;
    for bearer in [
        Some(token(&keys.pasien, "pasien", "inactive")),
        Some(token(&keys.pasien, "pasien", "tidak-dikenal")),
        None,
    ] {
        let r = call(addr, "GET", "/public/v1/context-only", bearer.as_deref()).await;
        assert_eq!(r.status, 500, "{bearer:?}");
        assert_eq!(r.json()["response_code"], "5000001");
        assert_eq!(r.json()["correlation_id"], CID);
        assert!(!r.body.contains("tidak boleh sampai sini"));
    }
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
    // trace: KU-10 (b): akun nonaktif → 4010002 (layar "Akun nonaktif"); lainnya 4010001.
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
        let code = if sub == "inactive" {
            "4010002"
        } else {
            "4010001"
        };
        assert_eq!(r.json()["response_code"], code, "{sub}");
        assert_eq!(r.json()["correlation_id"], CID);
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
async fn revoked_session_is_401_inactive_status_wins_and_guard_failure_is_503() {
    // trace: US-103 AC5, ADR-011 butir 8: sesi dicabut ditolak walaupun token belum kedaluwarsa.
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
        Realm::Pasien,
        "inactive",
        record(uuid(1), AccountType::Patient, AccountStatus::Inactive, &[]),
    );
    let addr = spawn(&keys, resolver.clone()).await;
    let live = token_sid(&keys.pasien, "pasien", "p-a", "sesi-hidup");
    let gone = token_sid(&keys.pasien, "pasien", "p-a", "sesi-dicabut");
    assert_eq!(
        call(addr, "GET", "/patient/v1/me", Some(&gone))
            .await
            .status,
        200
    );
    resolver
        .revoked
        .lock()
        .unwrap()
        .insert("sesi-dicabut".to_string());
    let r = call(addr, "GET", "/patient/v1/me", Some(&gone)).await;
    assert_eq!(r.status, 401);
    assert_eq!(r.json()["response_code"], "4010001");
    // Sesi lain akun yang sama tetap berjalan.
    assert_eq!(
        call(addr, "GET", "/patient/v1/me", Some(&live))
            .await
            .status,
        200
    );
    // Akun nonaktif yang sesinya juga dicabut: layar "Akun nonaktif" didahulukan (KU-10 b).
    resolver
        .revoked
        .lock()
        .unwrap()
        .insert("sesi-nonaktif".to_string());
    let r = call(
        addr,
        "GET",
        "/patient/v1/me",
        Some(&token_sid(
            &keys.pasien,
            "pasien",
            "inactive",
            "sesi-nonaktif",
        )),
    )
    .await;
    assert_eq!(r.json()["response_code"], "4010002");
    // Penjaga sesi gagal → 503, tidak pernah dilewatkan.
    *resolver.guard_failing.lock().unwrap() = true;
    let r = call(addr, "GET", "/patient/v1/me", Some(&live)).await;
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

    assert!(
        eventually(|| sec_log.text().contains("foreign_resource_id")).await,
        "security log penolakan tidak tercatat"
    );
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

#[tokio::test(flavor = "current_thread")]
async fn care_recipient_ownership_is_enforced_and_sensitive_denials_are_audited() {
    // trace: KU-03 (akses ditolak ke data sensitif tercatat hasil gagal), ADR-012 butir 1 dan 4, E-05
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
    resolver.add(
        Realm::Nakes,
        "n-1",
        record(
            uuid(0x31),
            AccountType::HealthWorker,
            AccountStatus::Active,
            &[],
        ),
    );
    let auditor = Arc::new(MemAuditor::default());
    let addr = spawn_with(&keys, resolver, sink(Some(auditor.clone()))).await;
    let t = token(&keys.pasien, "pasien", "p-a");
    let path = |id: Uuid| format!("/patient/v1/care-recipients/{id}");

    // Pemilik akun sendiri dan anggota keluarganya: lolos, dengan penanda E-05 untuk anggota keluarga.
    let own = call(addr, "GET", &path(uuid(0x21)), Some(&t)).await;
    assert_eq!(
        (own.status, own.body.clone()),
        (200, format!("cr:{}:true", uuid(0x21)))
    );
    let family = call(addr, "GET", &path(uuid(0x22)), Some(&t)).await;
    assert_eq!(family.body, format!("cr:{}:false", uuid(0x22)));
    // Ganti ID ke penerima layanan pasien lain: 404 identik dengan yang tidak ada (KU-03).
    let foreign = call(addr, "GET", &path(uuid(0x23)), Some(&t)).await;
    let missing = call(addr, "GET", &path(uuid(0x99)), Some(&t)).await;
    assert_eq!(foreign.status, 404);
    assert_eq!(foreign.json()["response_code"], "4040001");
    assert_eq!(foreign.body, missing.body);
    // Tenaga kesehatan tanpa relasi: 404 dan tercatat.
    let nakes = call(
        addr,
        "GET",
        &format!("/practitioner/v1/care-recipients/{}", uuid(0x21)),
        Some(&token(&keys.nakes, "nakes", "n-1")),
    )
    .await;
    assert_eq!(nakes.body, missing.body);

    assert!(eventually(|| auditor.seen.lock().unwrap().len() == 2).await);
    let seen = auditor.seen.lock().unwrap().clone();
    let by_patient = seen
        .iter()
        .find(|d| d.account_id == patient_a())
        .expect("penolakan pasien");
    let target = by_patient.sensitive.unwrap();
    assert_eq!(target.object_type, "care_recipient");
    assert_eq!(target.subject_care_recipient_id, uuid(0x23));
    assert_eq!(
        by_patient.route,
        "/patient/v1/care-recipients/{care_recipient_id}"
    );
    assert_eq!(by_patient.correlation_id.to_string(), CID);
    assert_eq!(
        by_patient.client_ip,
        Some(IpAddr::V4(Ipv4Addr::new(198, 51, 100, 7)))
    );
    let by_nakes = seen
        .iter()
        .find(|d| d.account_type == AccountType::HealthWorker)
        .expect("penolakan tenaga kesehatan");
    assert_eq!(
        by_nakes.sensitive.unwrap().subject_care_recipient_id,
        uuid(0x21)
    );
    // Objek yang tidak ada dan akses sah tidak dicatat sebagai penolakan.
    assert!(!seen.iter().any(|d| {
        d.sensitive
            .is_some_and(|s| s.object_id == uuid(0x99) || s.object_id == uuid(0x22))
    }));
    assert!(eventually(|| sec_log.text().matches("foreign_resource_id").count() == 2).await);
    assert!(
        !sec_log
            .text()
            .contains(&uuid(0x23).hyphenated().to_string())
    );
}

#[tokio::test(flavor = "current_thread")]
async fn denial_audit_is_retried_and_permanent_failure_is_logged_without_changing_response() {
    let (app_log, _sec_log, _guard) = capture();
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
    let auditor = Arc::new(MemAuditor::default());
    *auditor.fail_retryable.lock().unwrap() = 2;
    let addr = spawn_with(&keys, resolver, sink(Some(auditor.clone()))).await;
    let t = token(&keys.pasien, "pasien", "p-a");
    let foreign = format!("/patient/v1/care-recipients/{}", uuid(0x23));
    assert_eq!(call(addr, "GET", &foreign, Some(&t)).await.status, 404);
    // Dua kegagalan sementara lalu berhasil (jeda 100 ms + 1 s).
    for _ in 0..40 {
        if auditor.seen.lock().unwrap().len() == 1 {
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    assert_eq!(auditor.seen.lock().unwrap().len(), 1);
    assert_eq!(auditor.calls.load(Ordering::SeqCst), 3);
    // Event dibentuk sekali; ketiga percobaan menulis event yang sama (tidak menggandakan).
    assert_eq!(auditor.prepares.load(Ordering::SeqCst), 1);
    assert_eq!(*auditor.written_ids.lock().unwrap(), vec![0, 0, 0]);

    *auditor.permanent.lock().unwrap() = true;
    assert_eq!(call(addr, "GET", &foreign, Some(&t)).await.status, 404);
    assert!(eventually(|| app_log.text().contains("access_denial_audit_failed")).await);
    assert!(app_log.text().contains("\"reason\":\"permanent\""));
    assert_eq!(
        auditor.calls.load(Ordering::SeqCst),
        4,
        "galat permanen tidak dicoba ulang"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn full_denial_queue_never_delays_responses_and_overflow_is_reported() {
    let (app_log, _sec_log, _guard) = capture();
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
    let gate = Arc::new(Semaphore::new(0));
    let auditor = Arc::new(MemAuditor {
        gate: Some(gate.clone()),
        ..MemAuditor::default()
    });
    let denials = DenialSink::start(1, 1, Some(auditor.clone() as Arc<dyn DenialAuditor>)).unwrap();
    let addr = spawn_with(&keys, resolver, denials).await;
    let t = token(&keys.pasien, "pasien", "p-a");
    let foreign = format!("/patient/v1/care-recipients/{}", uuid(0x23));
    for _ in 0..10 {
        let started = Instant::now();
        let r = call(addr, "GET", &foreign, Some(&t)).await;
        assert_eq!(r.status, 404);
        assert!(
            started.elapsed() < Duration::from_millis(500),
            "respons tertahan antrean"
        );
    }
    // Pembuangan pertama dalam jendela langsung dilaporkan, tanpa menunggu antrean lega.
    assert!(eventually(|| app_log.text().contains("access_denial_queue_overflow")).await);
    gate.add_permits(100);
}

#[tokio::test(flavor = "current_thread")]
async fn security_log_is_not_held_back_by_pending_audit_writes() {
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
    let gate = Arc::new(Semaphore::new(0));
    let auditor = Arc::new(MemAuditor {
        gate: Some(gate.clone()),
        ..MemAuditor::default()
    });
    let denials =
        DenialSink::start(64, 1, Some(auditor.clone() as Arc<dyn DenialAuditor>)).unwrap();
    let addr = spawn_with(&keys, resolver, denials).await;
    let t = token(&keys.pasien, "pasien", "p-a");
    let foreign = format!("/patient/v1/care-recipients/{}", uuid(0x23));
    for _ in 0..5 {
        assert_eq!(call(addr, "GET", &foreign, Some(&t)).await.status, 404);
    }
    // Penulisan audit tertahan (kuota 1, gerbang tertutup), security log kelima penolakan tetap keluar.
    assert!(
        eventually(|| sec_log.text().matches("foreign_resource_id").count() == 5).await,
        "security log tertahan kuota audit"
    );
    assert!(auditor.seen.lock().unwrap().is_empty());
    gate.add_permits(100);
    assert!(eventually(|| auditor.seen.lock().unwrap().len() == 5).await);
}

#[tokio::test(flavor = "current_thread")]
async fn owner_read_for_another_care_recipient_is_rejected() {
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
    let auditor = Arc::new(MemAuditor::default());
    let addr = spawn_with(&keys, resolver, sink(Some(auditor.clone()))).await;
    let t = token(&keys.pasien, "pasien", "p-a");
    let own = call(
        addr,
        "GET",
        &format!("/patient/v1/swapped-owner/{}", uuid(0x21)),
        Some(&t),
    )
    .await;
    assert_eq!(own.body, "lolos");
    let r = call(
        addr,
        "GET",
        &format!("/patient/v1/swapped-owner/{}", uuid(0x23)),
        Some(&t),
    )
    .await;
    assert_eq!(r.status, 404);
    assert_eq!(r.json()["response_code"], "4040001");
}
