//! Cakupan tenant perusahaan lewat HTTP sungguhan di localhost (F1-T100, T1; ADR-028 butir 1, ADR-027
//! butir 2, US-1904 AC3–AC5, US-1901 AC6, US-1916 AC2, KU-03, KU-10): namespace dan pemeriksaan saat start,
//! token dua arah, keanggotaan aktif dari resolver di setiap permintaan, status perusahaan, dan `404` untuk
//! objek perusahaan lain. Token dari kunci uji (tanpa Keycloak); keanggotaan dari resolver di memori
//! (resolver database diuji di `apps/rs-backend/tests/corporate_isolation.rs`).

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
    AccountRecord, AccountStatus, AccountType, AdminSubRole, CorporateClientStatus,
    CorporateMembership, CorporateScope, CorporateScopeResolver, DenialSink, Enforcement,
    MembershipFuture, Policy, PolicyRouter, PolicySetupError, Principal, PrincipalResolver,
    PublicReason, ResolveError, ResolveFuture, RevocationFuture, SessionGuard, corporate_object,
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
use std::time::Duration;
use time::OffsetDateTime;
use time::macros::datetime;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use utoipa_axum::routes;

const NOW: OffsetDateTime = datetime!(2027-03-01 08:00:00 UTC);
const CID: &str = "01936f8e-9c01-7e02-8f03-1a2b3c4d5e6f";
const MEMBER_ROUTE: &str = "/v1/corporate/members/{corporate_member_id}";

fn iss(realm: &str) -> String {
    format!("https://auth.ruang-sehat.invalid/realms/{realm}")
}

fn aud(realm: &str) -> String {
    format!("rs-api-{realm}")
}

fn uuid(n: u128) -> Uuid {
    Uuid::from_u128(0x0193_0000_0000_7000_8000_0000_0000_0000 | n)
}

fn company_a() -> Uuid {
    uuid(0xA0)
}

fn company_b() -> Uuid {
    uuid(0xB0)
}

/// Simpanan uji anggota HR: (id anggota → perusahaan pemilik).
fn members() -> HashMap<Uuid, Uuid> {
    HashMap::from([(uuid(0xA1), company_a()), (uuid(0xB1), company_b())])
}

// ----- handler uji -----

#[utoipa::path(get, path = "/v1/corporate/company", responses((status = 200)))]
async fn company(scope: CorporateScope) -> String {
    format!(
        "company:{}:{}",
        scope.corporate_client_id(),
        scope.client_status()
    )
}

#[utoipa::path(get, path = "/v1/corporate/members/{corporate_member_id}", params(("corporate_member_id" = String, Path)), responses((status = 200)))]
async fn member(scope: CorporateScope, Path(raw): Path<String>) -> Response {
    // ID tidak valid diperlakukan sama dengan tidak ada (KU-03).
    let Ok(id) = Uuid::parse_str(&raw) else {
        return render_common(CommonError::NotFound);
    };
    let found = members().get(&id).map(|owner| (id, *owner));
    match corporate_object(&scope, MEMBER_ROUTE, found, |(_, owner)| *owner) {
        Ok((id, _)) => format!("member:{id}").into_response(),
        Err(e) => render_common(e.to_common_error()),
    }
}

/// Endpoint HR yang (keliru) memakai ekstraktor `Principal` saja tetap mendapat cakupan dari middleware.
#[utoipa::path(get, path = "/v1/corporate/whoami", responses((status = 200)))]
async fn whoami(p: Principal) -> String {
    format!("{}:{}", p.account_type(), p.account_id())
}

#[utoipa::path(get, path = "/patient/v1/me", responses((status = 200)))]
async fn patient_me(p: Principal) -> String {
    p.account_type().to_string()
}

#[utoipa::path(get, path = "/backoffice/v1/verifications", responses((status = 200)))]
async fn verifications(_p: Principal) -> &'static str {
    "verifier-ok"
}

#[utoipa::path(get, path = "/v1/corporate/status-page", responses((status = 200)))]
async fn corporate_public() -> &'static str {
    "public-ok"
}

#[utoipa::path(get, path = "/patient/v1/corporate-scope", responses((status = 200)))]
async fn scope_outside_corporate(scope: CorporateScope) -> String {
    scope.corporate_client_id().to_string()
}

fn render_common(error: CommonError) -> Response {
    Renderer.render(error, CorrelationId::parse(CID).unwrap())
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

// ----- resolver uji -----

#[derive(Default)]
struct Directory {
    accounts: Mutex<HashMap<(Realm, String), AccountRecord>>,
    memberships: Mutex<HashMap<Uuid, CorporateMembership>>,
    membership_failing: Mutex<bool>,
    membership_calls: Mutex<usize>,
}

impl Directory {
    fn add_account(&self, realm: Realm, sub: &str, id: Uuid, t: AccountType, s: AccountStatus) {
        self.accounts.lock().unwrap().insert(
            (realm, sub.to_string()),
            AccountRecord {
                account_id: id,
                account_type: t,
                status: s,
                sub_roles: if t == AccountType::RsAdmin {
                    vec![AdminSubRole::Verifier]
                } else {
                    vec![]
                },
            },
        );
    }

    fn set_membership(&self, account_id: Uuid, client: Uuid, status: CorporateClientStatus) {
        self.memberships.lock().unwrap().insert(
            account_id,
            CorporateMembership {
                corporate_member_id: uuid(account_id.as_u128() & 0xFFFF | 0x1_0000),
                corporate_client_id: client,
                client_status: status,
            },
        );
    }
}

impl PrincipalResolver for Directory {
    fn resolve<'a>(&'a self, ctx: &'a AuthContext) -> ResolveFuture<'a> {
        let rec = self
            .accounts
            .lock()
            .unwrap()
            .get(&(ctx.realm(), ctx.subject().to_string()))
            .cloned();
        Box::pin(async move { Ok(rec) })
    }
}

impl SessionGuard for Directory {
    fn is_revoked<'a>(&'a self, _ctx: &'a AuthContext) -> RevocationFuture<'a> {
        Box::pin(async { Ok(false) })
    }
}

impl CorporateScopeResolver for Directory {
    fn active_membership<'a>(&'a self, account_id: Uuid) -> MembershipFuture<'a> {
        *self.membership_calls.lock().unwrap() += 1;
        let failing = *self.membership_failing.lock().unwrap();
        let m = self.memberships.lock().unwrap().get(&account_id).copied();
        Box::pin(async move { if failing { Err(ResolveError) } else { Ok(m) } })
    }
}

// ----- perakitan -----

struct Keys {
    pasien: TestRealmKey,
    admin: TestRealmKey,
    korporat: TestRealmKey,
}

fn keys() -> Keys {
    Keys {
        pasien: TestRealmKey::generate("pasien-k").unwrap(),
        admin: TestRealmKey::generate("admin-k").unwrap(),
        korporat: TestRealmKey::generate("korporat-k").unwrap(),
    }
}

fn authenticator(keys: &Keys, realms: &[Realm]) -> Authenticator {
    let cfgs = realms
        .iter()
        .map(|r| {
            let key = match r {
                Realm::Pasien => &keys.pasien,
                Realm::Korporat => &keys.korporat,
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

fn auth(keys: &Keys, realms: &[Realm]) -> AuthState {
    AuthState::new(Arc::new(authenticator(keys, realms)), Arc::new(Renderer))
}

fn app_router() -> PolicyRouter {
    PolicyRouter::new()
        .routes(
            Policy::corporate_hr_any_verification_state(),
            routes!(company),
        )
        .routes(Policy::corporate_hr(), routes!(member))
        .routes(Policy::corporate_hr(), routes!(whoami))
        .routes(
            Policy::Public(PublicReason::PublicContent),
            routes!(corporate_public),
        )
        .routes(Policy::patient(), routes!(patient_me))
        .routes(
            Policy::rs_admin(&[AdminSubRole::Verifier]),
            routes!(verifications),
        )
        .routes(Policy::patient(), routes!(scope_outside_corporate))
}

fn enforcement(keys: &Keys, dir: &Arc<Directory>) -> Enforcement {
    Enforcement::new(
        auth(keys, &[Realm::Pasien, Realm::Admin, Realm::Korporat]),
        dir.clone(),
        dir.clone(),
        DenialSink::start(64, 1, None).unwrap(),
    )
    .with_corporate_scopes(dir.clone())
}

async fn spawn(keys: &Keys, dir: &Arc<Directory>) -> SocketAddr {
    let (router, _spec) = app_router().finish(enforcement(keys, dir)).unwrap();
    let app = router.layer(axum::middleware::from_fn(
        observability::correlation_middleware,
    ));
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { app.serve(listener, std::future::pending()).await.unwrap() });
    addr
}

fn token(key: &TestRealmKey, realm: &str, sub: &str) -> String {
    token_with(key, &iss(realm), &aud(realm), sub, 300)
}

fn token_with(key: &TestRealmKey, iss: &str, aud: &str, sub: &str, ttl: i64) -> String {
    let now = NOW.unix_timestamp();
    key.sign(&json!({
        "iss": iss, "aud": aud, "sub": sub, "typ": "Bearer",
        "sid": "sesi-uji", "azp": "rs-korporat-bff", "iat": now - 5, "exp": now + ttl,
    }))
    .unwrap()
}

struct Resp {
    status: u16,
    body: String,
}

impl Resp {
    fn code(&self) -> Value {
        serde_json::from_str::<Value>(&self.body).unwrap_or(Value::Null)["response_code"].clone()
    }
}

async fn call(addr: SocketAddr, path: &str, bearer: Option<&str>) -> Resp {
    let mut s = TcpStream::connect(addr).await.unwrap();
    let mut req = format!(
        "GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\nContent-Length: 0\r\nX-Correlation-Id: {CID}\r\n"
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

fn capture() -> (MemorySink, tracing::subscriber::DefaultGuard) {
    let app = MemorySink::default();
    let sec = MemorySink::default();
    let layer = JsonLogLayer::new(
        Component::Api,
        LogLevel::Info,
        Arc::new(FixedClock::new(NOW)),
    )
    .with_sinks(
        Arc::new(app) as Arc<dyn LogSink>,
        Arc::new(sec.clone()) as Arc<dyn LogSink>,
    );
    let guard = tracing::subscriber::set_default(subscriber(layer));
    (sec, guard)
}

async fn eventually(mut cond: impl FnMut() -> bool) -> bool {
    for _ in 0..300 {
        if cond() {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    cond()
}

/// Akun HR `sub` dengan ID `id`, aktif, beranggotakan perusahaan `client` berstatus `status`.
fn hr(dir: &Directory, sub: &str, id: Uuid, client: Uuid, status: CorporateClientStatus) {
    dir.add_account(
        Realm::Korporat,
        sub,
        id,
        AccountType::CorporateAdmin,
        AccountStatus::Active,
    );
    dir.set_membership(id, client, status);
}

// ----- pemeriksaan saat start -----

// trace: ADR-027 butir 2 (namespace /v1/corporate/* khusus HR)
#[test]
fn corporate_namespace_and_scope_resolver_are_checked_at_startup() {
    let keys = keys();
    let dir = Arc::new(Directory::default());

    // Endpoint HR di luar namespace korporat.
    let e = PolicyRouter::<()>::new()
        .routes(Policy::corporate_hr(), routes!(patient_me))
        .finish(enforcement(&keys, &dir))
        .err();
    assert_eq!(
        e,
        Some(PolicySetupError::CorporateNamespace {
            method: "GET".into(),
            path: "/patient/v1/me".into()
        })
    );
    // Endpoint terautentikasi non-HR di namespace korporat (pasien, admin).
    for policy in [
        Policy::patient(),
        Policy::rs_admin(&[AdminSubRole::Verifier]),
    ] {
        let e = PolicyRouter::<()>::new()
            .routes(policy, routes!(company))
            .finish(enforcement(&keys, &dir))
            .err();
        assert_eq!(
            e,
            Some(PolicySetupError::CorporateNamespace {
                method: "GET".into(),
                path: "/v1/corporate/company".into()
            })
        );
    }
    // HR digabung jenis akun lain di namespace korporat: tidak dapat dipenuhi.
    let e = PolicyRouter::<()>::new()
        .routes(
            Policy::accounts(&[AccountType::CorporateAdmin, AccountType::Patient]),
            routes!(company),
        )
        .finish(enforcement(&keys, &dir))
        .err();
    assert_eq!(
        e,
        Some(PolicySetupError::UnsatisfiablePolicy {
            method: "GET".into(),
            path: "/v1/corporate/company".into()
        })
    );
    // Tanpa pembaca keanggotaan perusahaan.
    let e = PolicyRouter::<()>::new()
        .routes(Policy::corporate_hr(), routes!(company))
        .finish(Enforcement::new(
            auth(&keys, &[Realm::Korporat]),
            dir.clone(),
            dir.clone(),
            DenialSink::start(4, 1, None).unwrap(),
        ))
        .err();
    assert_eq!(e, Some(PolicySetupError::MissingCorporateScope));
    // Realm korporat tidak diterima validator token.
    let e = PolicyRouter::<()>::new()
        .routes(Policy::corporate_hr(), routes!(company))
        .finish(
            Enforcement::new(
                auth(&keys, &[Realm::Pasien, Realm::Admin]),
                dir.clone(),
                dir.clone(),
                DenialSink::start(4, 1, None).unwrap(),
            )
            .with_corporate_scopes(dir.clone()),
        )
        .err();
    assert_eq!(
        e,
        Some(PolicySetupError::RealmNotConfigured { realm: "korporat" })
    );
    // Endpoint publik di namespace korporat sah (misalnya pendaftaran perusahaan, US-1901).
    assert!(
        PolicyRouter::<()>::new()
            .routes(
                Policy::Public(PublicReason::PublicContent),
                routes!(corporate_public)
            )
            .finish(Enforcement::public_only(Arc::new(Renderer)))
            .is_ok()
    );
}

// ----- token dua arah -----

// trace: US-1904 AC4
#[tokio::test(flavor = "current_thread")]
async fn korporat_tokens_and_other_realm_tokens_are_rejected_both_ways() {
    let keys = keys();
    let dir = Arc::new(Directory::default());
    hr(
        &dir,
        "hr-a",
        uuid(0x11),
        company_a(),
        CorporateClientStatus::Active,
    );
    dir.add_account(
        Realm::Pasien,
        "pasien",
        uuid(0x21),
        AccountType::Patient,
        AccountStatus::Active,
    );
    dir.add_account(
        Realm::Admin,
        "verifier",
        uuid(0x31),
        AccountType::RsAdmin,
        AccountStatus::Active,
    );
    let addr = spawn(&keys, &dir).await;
    let hr_token = token(&keys.korporat, "korporat", "hr-a");
    let patient = token(&keys.pasien, "pasien", "pasien");
    let admin = token(&keys.admin, "admin", "verifier");

    // Pembanding positif.
    assert_eq!(
        call(addr, "/v1/corporate/company", Some(&hr_token))
            .await
            .status,
        200
    );
    assert_eq!(
        call(addr, "/patient/v1/me", Some(&patient)).await.status,
        200
    );
    assert_eq!(
        call(addr, "/backoffice/v1/verifications", Some(&admin))
            .await
            .status,
        200
    );

    // Token korporat ke API pasien dan backoffice; token pasien/admin ke API korporat.
    for (path, t) in [
        ("/patient/v1/me", &hr_token),
        ("/backoffice/v1/verifications", &hr_token),
        ("/v1/corporate/company", &patient),
        ("/v1/corporate/company", &admin),
        ("/v1/corporate/members/{id}", &patient),
    ] {
        let path = path.replace("{id}", &uuid(0xA1).to_string());
        let r = call(addr, &path, Some(t)).await;
        assert_eq!((r.status, r.code()), (401, json!("4010001")), "{path}");
    }
    // Token korporat cacat: tanpa token, audience lain, issuer lain, kedaluwarsa, ditandatangani kunci
    // realm lain.
    let bad = [
        None,
        Some(token_with(
            &keys.korporat,
            &iss("korporat"),
            &aud("admin"),
            "hr-a",
            300,
        )),
        Some(token_with(
            &keys.korporat,
            &iss("admin"),
            &aud("korporat"),
            "hr-a",
            300,
        )),
        Some(token_with(
            &keys.korporat,
            &iss("korporat"),
            &aud("korporat"),
            "hr-a",
            -60,
        )),
        Some(token_with(
            &keys.admin,
            &iss("korporat"),
            &aud("korporat"),
            "hr-a",
            300,
        )),
    ];
    for t in bad {
        let r = call(addr, "/v1/corporate/company", t.as_deref()).await;
        assert_eq!((r.status, r.code()), (401, json!("4010001")));
    }
    // Endpoint publik di namespace korporat tetap terbuka.
    assert_eq!(
        call(addr, "/v1/corporate/status-page", None).await.status,
        200
    );
}

// ----- cakupan dan isolasi -----

// trace: US-1904 AC3
#[tokio::test(flavor = "current_thread")]
async fn hr_sees_only_own_company_and_foreign_objects_are_indistinguishable_404() {
    let (sec_log, _guard) = capture();
    let keys = keys();
    let dir = Arc::new(Directory::default());
    hr(
        &dir,
        "hr-a",
        uuid(0x11),
        company_a(),
        CorporateClientStatus::Active,
    );
    hr(
        &dir,
        "hr-b",
        uuid(0x12),
        company_b(),
        CorporateClientStatus::Active,
    );
    let addr = spawn(&keys, &dir).await;
    let a = token(&keys.korporat, "korporat", "hr-a");
    let b = token(&keys.korporat, "korporat", "hr-b");

    let r = call(addr, "/v1/corporate/company", Some(&a)).await;
    assert_eq!(r.body, format!("company:{}:active", company_a()));
    // corporate_client_id di query diabaikan: cakupan hanya dari keanggotaan di sesi.
    let r = call(
        addr,
        &format!("/v1/corporate/company?corporate_client_id={}", company_b()),
        Some(&a),
    )
    .await;
    assert_eq!(r.body, format!("company:{}:active", company_a()));
    assert!(!r.body.contains(&company_b().to_string()));

    let own = call(
        addr,
        &format!("/v1/corporate/members/{}", uuid(0xA1)),
        Some(&a),
    )
    .await;
    assert_eq!(
        (own.status, own.body.as_str()),
        (200, format!("member:{}", uuid(0xA1)).as_str())
    );
    let foreign = call(
        addr,
        &format!("/v1/corporate/members/{}", uuid(0xB1)),
        Some(&a),
    )
    .await;
    let missing = call(
        addr,
        &format!("/v1/corporate/members/{}", uuid(0xEE)),
        Some(&a),
    )
    .await;
    let garbage = call(addr, "/v1/corporate/members/bukan-uuid", Some(&a)).await;
    assert_eq!((foreign.status, foreign.code()), (404, json!("4040001")));
    // Isi identik: milik perusahaan lain tidak dapat dibedakan dari tidak ada (KU-03).
    assert_eq!(foreign.body, missing.body);
    assert_eq!(foreign.body, garbage.body);
    // Perusahaan B melihat objeknya sendiri.
    let r = call(
        addr,
        &format!("/v1/corporate/members/{}", uuid(0xB1)),
        Some(&b),
    )
    .await;
    assert_eq!(r.status, 200);

    // Hanya percobaan atas objek perusahaan lain yang tercatat (bukan objek yang tidak ada), tanpa ID objek.
    assert!(eventually(|| sec_log.text().contains("foreign_resource_id")).await);
    tokio::time::sleep(Duration::from_millis(50)).await;
    let denied: Vec<String> = sec_log
        .text()
        .lines()
        .filter(|l| l.contains("foreign_resource_id"))
        .map(str::to_string)
        .collect();
    assert_eq!(denied.len(), 1, "{denied:?}");
    assert!(denied[0].contains(MEMBER_ROUTE));
    assert!(
        denied[0].contains("\"event_type\":\"access_denied\""),
        "{}",
        denied[0]
    );
    assert!(!denied[0].contains(&uuid(0xB1).to_string()));
}

// trace: US-1904 AC5, US-1916 AC2, US-1901 AC6, KU-10, M-51
#[tokio::test(flavor = "current_thread")]
async fn membership_and_company_status_are_enforced_on_every_request() {
    let (sec_log, _guard) = capture();
    let keys = keys();
    let dir = Arc::new(Directory::default());
    hr(
        &dir,
        "hr-a",
        uuid(0x11),
        company_a(),
        CorporateClientStatus::Active,
    );
    let addr = spawn(&keys, &dir).await;
    let a = token(&keys.korporat, "korporat", "hr-a");
    let member_a = format!("/v1/corporate/members/{}", uuid(0xA1));
    let member_b = format!("/v1/corporate/members/{}", uuid(0xB1));
    assert_eq!(call(addr, &member_a, Some(&a)).await.status, 200);

    // Perusahaan menunggu verifikasi atau ditolak: hanya profil/status (US-1901 AC6); selain itu 403.
    for status in [
        CorporateClientStatus::PendingVerification,
        CorporateClientStatus::Rejected,
    ] {
        dir.set_membership(uuid(0x11), company_a(), status);
        let r = call(addr, "/v1/corporate/company", Some(&a)).await;
        assert_eq!(r.body, format!("company:{}:{status}", company_a()));
        for path in [member_a.as_str(), "/v1/corporate/whoami"] {
            let r = call(addr, path, Some(&a)).await;
            assert_eq!(
                (r.status, r.code()),
                (403, json!("4030001")),
                "{path} {status}"
            );
        }
    }
    assert!(
        sec_log
            .text()
            .lines()
            .any(|l| l.contains("\"event_type\":\"access_denied\"")
                && l.contains("role_not_allowed"))
    );

    // Perusahaan dinonaktifkan saat sesi terbuka: permintaan berikutnya 401 "Akun nonaktif".
    dir.set_membership(uuid(0x11), company_a(), CorporateClientStatus::Inactive);
    for path in ["/v1/corporate/company", member_a.as_str()] {
        let r = call(addr, path, Some(&a)).await;
        assert_eq!((r.status, r.code()), (401, json!("4010002")), "{path}");
    }

    // Keanggotaan dipindah ke perusahaan B: cakupan berikutnya B, objek A menjadi 404 (tanpa cache).
    dir.set_membership(uuid(0x11), company_b(), CorporateClientStatus::Active);
    assert_eq!(call(addr, &member_b, Some(&a)).await.status, 200);
    let r = call(addr, &member_a, Some(&a)).await;
    assert_eq!((r.status, r.code()), (404, json!("4040001")));

    // Keanggotaan dicabut: 401 seketika.
    dir.memberships.lock().unwrap().clear();
    let r = call(addr, "/v1/corporate/company", Some(&a)).await;
    assert_eq!((r.status, r.code()), (401, json!("4010001")));

    // Akun HR nonaktif: 401 4010002 sebelum keanggotaan dibaca.
    hr(
        &dir,
        "hr-a",
        uuid(0x11),
        company_a(),
        CorporateClientStatus::Active,
    );
    dir.add_account(
        Realm::Korporat,
        "hr-a",
        uuid(0x11),
        AccountType::CorporateAdmin,
        AccountStatus::Inactive,
    );
    let before = *dir.membership_calls.lock().unwrap();
    let r = call(addr, "/v1/corporate/company", Some(&a)).await;
    assert_eq!((r.status, r.code()), (401, json!("4010002")));
    assert_eq!(*dir.membership_calls.lock().unwrap(), before);

    // Pembaca keanggotaan gagal: 503, tidak pernah dilewatkan.
    dir.add_account(
        Realm::Korporat,
        "hr-a",
        uuid(0x11),
        AccountType::CorporateAdmin,
        AccountStatus::Active,
    );
    *dir.membership_failing.lock().unwrap() = true;
    for path in ["/v1/corporate/company", member_a.as_str()] {
        let r = call(addr, path, Some(&a)).await;
        assert_eq!((r.status, r.code()), (503, json!("5030001")), "{path}");
    }
}

// trace: ADR-028 butir 1 (cakupan hanya untuk endpoint HR)
#[tokio::test(flavor = "current_thread")]
async fn scope_extractor_outside_corporate_endpoints_fails_closed() {
    let keys = keys();
    let dir = Arc::new(Directory::default());
    dir.add_account(
        Realm::Pasien,
        "pasien",
        uuid(0x21),
        AccountType::Patient,
        AccountStatus::Active,
    );
    let addr = spawn(&keys, &dir).await;
    let patient = token(&keys.pasien, "pasien", "pasien");
    // Handler non-HR yang meminta CorporateScope: 500 tanpa data, bukan cakupan kosong.
    let r = call(addr, "/patient/v1/corporate-scope", Some(&patient)).await;
    assert_eq!((r.status, r.code()), (500, json!("5000001")));
    assert_eq!(*dir.membership_calls.lock().unwrap(), 0);
}
