//! F1-T30 (ADR-018 butir 3, ADR-012 butir 4, ADR-005; US-106, US-201 AC6) terhadap PostgreSQL 18 (role
//! runtime `rs_app`), database audit (relay + pembaca verifier), dan SeaweedFS: URL unduh berumur pendek
//! hanya setelah otorisasi (`authz`: kebijakan endpoint + `visible_object`) dan E-05 tercatat (`audit`).
//!
//! Endpoint unduh produksi dibuat modul pemakai (F1-T62 Verifikator); test ini memasang dua rute uji lewat
//! `PolicyRouter` + validator token sungguhan (kunci realm uji, tanpa Keycloak) supaya `Principal` berasal
//! dari jalur autentikasi/otorisasi yang sama dengan produksi.
//!
//! Skenario negatif modul T1 (coding-standards §7.3): tanpa token (401), peran lain (403), objek milik orang
//! lain (404 + security log + E-05 gagal), ID acak (404 tanpa E-05), kategori lain (404), URL tanpa tanda
//! tangan atau dirusak (403 penyedia), URL kedaluwarsa (403 penyedia), pencatatan audit gagal (tanpa URL),
//! jenis palsu dan ukuran berlebih tidak pernah menghasilkan berkas yang dapat diunduh.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)] // berkas test

mod files;

use audit::{AuditContext, Source, SourceComponent, Subject};
use audit_store::{PageCursor, StoredEvent, read_page};
use authn::testing::{StaticJwks, TestRealmKey, jwks_document};
use authn::{
    Algorithm, AuthContext, AuthState, Authenticator, AuthnConfig, Backend, ErrorRenderer, Realm,
    RealmConfig,
};
use authz::{
    AccountRecord, AccountStatus, AccountType, AdminSubRole, Enforcement, Policy, PolicyRouter,
    Principal, PrincipalResolver, ResolveError, ResolveFuture,
};
use axum::extract::{Path, State};
use axum::http::header::CONTENT_TYPE;
use axum::response::{IntoResponse, Response};
use envelope::{ApiError, CommonError};
use files::{SeaweedFs, app_pool, contains, jpeg, jpeg_with_exif, new_account, now, png, upload};
use foundation::{Clock, CorrelationId, FixedClock, SystemClock, Uuid, new_id};
use observability::LogLevel;
use observability::log::{Component, JsonLogLayer, LogSink, MemorySink, subscriber};
use platform_jobs::lock::RS_AUDIT_RELAY;
use platform_jobs::{LockAttempt, SingletonLock};
use rs_backend::modules::platform::api::{
    DOWNLOAD_URL_TTL, DownloadRequest, FileAccess, FileError, FileStorage, LogicalBucket,
    MAX_PHOTO_BYTES, UploadMethod, UploadPurpose, UploadSlotRequest, create_upload_slot,
    issue_download_url, record_stored_file, validate_upload,
};
use serde_json::{Value, json};
use sqlx::{Connection, PgConnection, PgPool};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use utoipa_axum::routes;

const TTL: Duration = Duration::from_secs(600);
const CID: &str = "01936f8e-9c01-7e02-8f03-1a2b3c4d5e6f";
static SERIAL: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

const OWNER_ROUTE: &str = "/patient/v1/uji-files/{file_id}/download-url";
const VERIFIER_ROUTE: &str = "/backoffice/v1/uji-identity-files/{file_id}/download-url";
/// Subjek data uji untuk akses Verifikator (penerima layanan sintetis; outbox tidak memeriksa FK).
const SUBJECT: Uuid = Uuid::from_u128(0x0193_6f8e_9c01_7e02_8f03_aaaa_bbbb_cc01);

fn iss(realm: &str) -> String {
    format!("https://auth.ruang-sehat.invalid/realms/{realm}")
}

fn aud(realm: &str) -> String {
    format!("rs-api-{realm}")
}

// ----- keadaan dan handler uji -----

struct Harness {
    pool: PgPool,
    storage: FileStorage,
    clock: Arc<dyn Clock>,
    /// Konteks audit tanpa alamat IP (event akun ditolak pustaka) untuk test kegagalan pencatatan.
    broken_audit: std::sync::atomic::AtomicBool,
}

type Shared = Arc<Harness>;

fn audit_context(h: &Harness) -> AuditContext {
    let cid = CorrelationId::parse(CID).unwrap();
    let source = Source::component(SourceComponent::Backoffice);
    let source = if h.broken_audit.load(std::sync::atomic::Ordering::SeqCst) {
        source
    } else {
        source.ip(IpAddr::V4(Ipv4Addr::new(198, 51, 100, 7)))
    };
    AuditContext::new(cid, source)
}

fn render(e: &FileError) -> Response {
    let cid = CorrelationId::parse(CID).unwrap();
    let common = match e.response_code().as_str() {
        "4040001" => CommonError::NotFound,
        "5030001" => CommonError::Unavailable,
        _ => CommonError::Internal,
    };
    let body = common.to_error_body(cid, "https://d.example/problems");
    (
        common.http_status(),
        [(CONTENT_TYPE, "application/problem+json")],
        serde_json::to_vec(&body).unwrap(),
    )
        .into_response()
}

async fn issue(h: &Harness, p: &Principal, id: &str, route: &str, access: FileAccess) -> Response {
    let Ok(stored_file_id) = Uuid::parse_str(id) else {
        return render(&FileError::NotFound);
    };
    let audit = audit_context(h);
    match issue_download_url(
        &h.pool,
        h.clock.as_ref(),
        &h.storage,
        p,
        &audit,
        DownloadRequest {
            stored_file_id,
            purpose: UploadPurpose::KtpPhoto,
            route,
            access,
        },
    )
    .await
    {
        Ok(t) => axum::Json(json!({
            "url": t.url,
            "expires_at": t.expires_at.unix_timestamp(),
            "mime_type": t.mime_type,
            "size_bytes": t.size_bytes,
        }))
        .into_response(),
        Err(e) => render(&e),
    }
}

#[utoipa::path(get, path = "/patient/v1/uji-files/{file_id}/download-url", params(("file_id" = String, Path)), responses((status = 200)))]
async fn owner_download(State(h): State<Shared>, p: Principal, Path(id): Path<String>) -> Response {
    issue(&h, &p, &id, OWNER_ROUTE, FileAccess::Owner).await
}

#[utoipa::path(get, path = "/backoffice/v1/uji-identity-files/{file_id}/download-url", params(("file_id" = String, Path)), responses((status = 200)))]
async fn verifier_download(
    State(h): State<Shared>,
    p: Principal,
    Path(id): Path<String>,
) -> Response {
    let access = FileAccess::Staff {
        roles: &[AdminSubRole::Verifier],
        subject: Subject::care_recipient(SUBJECT),
    };
    issue(&h, &p, &id, VERIFIER_ROUTE, access).await
}

/// Rute dengan kebijakan endpoint yang lebih longgar (admin sub-peran apa pun) supaya penolakan
/// sub-peran di tingkat objek (`FileAccess::Staff`) ikut teruji.
#[utoipa::path(get, path = "/backoffice/v1/uji-any-admin-files/{file_id}/download-url", params(("file_id" = String, Path)), responses((status = 200)))]
async fn any_admin_download(
    State(h): State<Shared>,
    p: Principal,
    Path(id): Path<String>,
) -> Response {
    let access = FileAccess::Staff {
        roles: &[AdminSubRole::Verifier],
        subject: Subject::care_recipient(SUBJECT),
    };
    issue(
        &h,
        &p,
        &id,
        "/backoffice/v1/uji-any-admin-files/{file_id}/download-url",
        access,
    )
    .await
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
        let rec = self
            .accounts
            .lock()
            .unwrap()
            .get(&(ctx.realm(), ctx.subject().to_string()))
            .cloned();
        Box::pin(async move { Ok::<_, ResolveError>(rec) })
    }
}

struct Keys {
    pasien: TestRealmKey,
    admin: TestRealmKey,
}

fn authenticator(keys: &Keys) -> Authenticator {
    let realm = |r: Realm, key: &TestRealmKey| RealmConfig {
        realm: r,
        issuer: iss(r.as_str()),
        audiences: vec![aud(r.as_str())],
        algorithms: vec![Algorithm::RS256],
        authorized_parties: vec![],
        jwks: Arc::new(StaticJwks::new(jwks_document(&[key.jwk()]))),
    };
    Authenticator::new(
        AuthnConfig::with_defaults(
            Backend::RuangSehat,
            vec![
                realm(Realm::Pasien, &keys.pasien),
                realm(Realm::Admin, &keys.admin),
            ],
        ),
        Arc::new(SystemClock),
    )
    .unwrap()
}

fn token(key: &TestRealmKey, realm: &str, sub: &str) -> String {
    let now = now().unix_timestamp();
    key.sign(&json!({
        "iss": iss(realm), "aud": aud(realm), "sub": sub, "typ": "Bearer",
        "sid": "sesi-uji", "azp": "app", "iat": now - 5, "exp": now + 300,
    }))
    .unwrap()
}

async fn spawn(keys: &Keys, resolver: Arc<MemResolver>, harness: Shared) -> SocketAddr {
    let auth = AuthState::new(Arc::new(authenticator(keys)), Arc::new(Renderer));
    let (router, _spec) = PolicyRouter::<Shared>::new()
        .routes(Policy::patient(), routes!(owner_download))
        .routes(
            Policy::rs_admin(&[AdminSubRole::Verifier]),
            routes!(verifier_download),
        )
        .routes(
            Policy::rs_admin(&AdminSubRole::ALL),
            routes!(any_admin_download),
        )
        .finish(Enforcement::new(auth, resolver))
        .unwrap();
    let app = router.with_state(harness).layer(axum::middleware::from_fn(
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
    fn url(&self) -> String {
        self.json()["url"].as_str().unwrap().to_string()
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

/// Unduhan lewat URL bertanda tangan (klien tanpa kredensial): (kode HTTP, header, isi).
fn download(url: &str) -> (u16, String, Vec<u8>) {
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join("platform-files-unduh");
    std::fs::create_dir_all(&dir).unwrap();
    let body = dir.join(files::random_hex(8));
    let headers = dir.join(files::random_hex(8));
    let out = Command::new("curl")
        .args(["-s", "-w", "%{http_code}", "-o"])
        .arg(&body)
        .arg("-D")
        .arg(&headers)
        .arg(url)
        .output()
        .unwrap();
    let code: u16 = String::from_utf8(out.stdout)
        .unwrap()
        .trim()
        .parse()
        .unwrap();
    let h = std::fs::read_to_string(&headers).unwrap_or_default();
    let b = std::fs::read(&body).unwrap_or_default();
    let _ = std::fs::remove_file(body);
    let _ = std::fs::remove_file(headers);
    (code, h.to_ascii_lowercase(), b)
}

fn capture() -> (MemorySink, tracing::subscriber::DefaultGuard) {
    let app = MemorySink::default();
    let sec = MemorySink::default();
    let layer = JsonLogLayer::new(
        Component::Api,
        LogLevel::Info,
        Arc::new(FixedClock::new(now())),
    )
    .with_sinks(
        Arc::new(app) as Arc<dyn LogSink>,
        Arc::new(sec.clone()) as Arc<dyn LogSink>,
    );
    let guard = tracing::subscriber::set_default(subscriber(layer));
    (sec, guard)
}

// ----- audit: relay ke database audit lalu baca lewat verifier -----

async fn connection(var: &str) -> PgConnection {
    let env = observability::EnvSource::from_process();
    let url = env
        .required_secret(var)
        .unwrap_or_else(|_| panic!("{var} tidak diset (jalankan lewat scripts/db-ephemeral.sh)"));
    PgConnection::connect(url.expose())
        .await
        .unwrap_or_else(|_| panic!("koneksi {var} gagal"))
}

/// Memindahkan semua event outbox ke database audit lewat relay, lalu mengembalikan event E-05 berobjek
/// `file_id`.
async fn e05_events(file_id: Uuid) -> Vec<StoredEvent> {
    let mut conn = connection("RS_TEST_DB_RELAY_URL").await;
    let mut lock = None;
    for _ in 0..100 {
        match SingletonLock::try_acquire(conn, RS_AUDIT_RELAY)
            .await
            .unwrap()
        {
            LockAttempt::Acquired(l) => {
                lock = Some(l);
                break;
            }
            LockAttempt::Busy(c) => {
                conn = c;
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }
    }
    let mut lock = lock.expect("kunci relay");
    let mut writer = connection("RS_TEST_DB_AUDIT_WRITER_URL").await;
    loop {
        let b = audit::relay::relay_batch(&mut lock, &mut writer, 50)
            .await
            .unwrap();
        if b.fetched == 0 {
            break;
        }
    }
    lock.release().await.unwrap();
    let mut verifier = connection("RS_TEST_DB_AUDIT_VERIFIER_URL").await;
    let fid = file_id.hyphenated().to_string();
    let mut out = Vec::new();
    let mut cursor = PageCursor::START;
    loop {
        let page = read_page(&mut verifier, cursor, 500).await.unwrap();
        let Some(last) = page.last() else {
            break;
        };
        cursor = PageCursor {
            seq: last.event.seq,
            event_id: last.event.body.event_id,
        };
        out.extend(page.into_iter().filter(|e| {
            e.event.body.event_code == "e_05"
                && e.event.body.object_id.as_deref() == Some(fid.as_str())
        }));
    }
    out
}

// ----- perangkat berkas -----

/// Berkas KTP tervalidasi dan tercatat milik `owner`; mengembalikan (ID stored_file, isi tersimpan sha256).
async fn stored_ktp(
    pool: &PgPool,
    storage: &FileStorage,
    owner: Uuid,
    bytes: &[u8],
) -> (Uuid, Vec<u8>) {
    let ticket = create_upload_slot(
        pool,
        &SystemClock,
        storage,
        UploadSlotRequest {
            owner_account_id: owner,
            purpose: UploadPurpose::KtpPhoto,
            content_type: "image/jpeg".to_string(),
            size_bytes: bytes.len() as u64,
        },
    )
    .await
    .unwrap();
    assert_eq!(upload(&ticket.instruction, bytes), 204);
    let v = validate_upload(
        pool,
        &SystemClock,
        storage,
        owner,
        ticket.upload_slot_id,
        UploadPurpose::KtpPhoto,
    )
    .await
    .unwrap();
    let sha = v.sha256().to_vec();
    let mut tx = pool.begin().await.unwrap();
    let id = record_stored_file(&mut tx, &SystemClock, &v, None)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    (id, sha)
}

struct World {
    addr: SocketAddr,
    keys: Keys,
    harness: Shared,
    owner: Uuid,
    intruder: Uuid,
    verifier: Uuid,
    customer_service: Uuid,
    _sw: SeaweedFs,
}

async fn world(clock: Arc<dyn Clock>) -> World {
    let pool = app_pool().await;
    let sw = SeaweedFs::start().await;
    let storage = sw.file_storage(UploadMethod::Post, TTL).await;
    let owner = new_account(&pool).await;
    let intruder = new_account(&pool).await;
    let verifier = new_id();
    let customer_service = new_id();
    let resolver = Arc::new(MemResolver::default());
    let rec = |id: Uuid, t: AccountType, roles: &[AdminSubRole]| AccountRecord {
        account_id: id,
        account_type: t,
        status: AccountStatus::Active,
        sub_roles: roles.to_vec(),
    };
    resolver.add(
        Realm::Pasien,
        "pemilik",
        rec(owner, AccountType::Patient, &[]),
    );
    resolver.add(
        Realm::Pasien,
        "penyusup",
        rec(intruder, AccountType::Patient, &[]),
    );
    resolver.add(
        Realm::Admin,
        "verifikator",
        rec(verifier, AccountType::RsAdmin, &[AdminSubRole::Verifier]),
    );
    resolver.add(
        Realm::Admin,
        "layanan-pelanggan",
        rec(
            customer_service,
            AccountType::RsAdmin,
            &[AdminSubRole::CustomerService, AdminSubRole::SuperAdmin],
        ),
    );
    let keys = Keys {
        pasien: TestRealmKey::generate("pasien-k").unwrap(),
        admin: TestRealmKey::generate("admin-k").unwrap(),
    };
    let harness = Arc::new(Harness {
        pool,
        storage,
        clock,
        broken_audit: std::sync::atomic::AtomicBool::new(false),
    });
    let addr = spawn(&keys, resolver, Arc::clone(&harness)).await;
    World {
        addr,
        keys,
        harness,
        owner,
        intruder,
        verifier,
        customer_service,
        _sw: sw,
    }
}

impl World {
    fn owner_path(&self, id: Uuid) -> String {
        OWNER_ROUTE.replace("{file_id}", &id.to_string())
    }
    fn verifier_path(&self, id: Uuid) -> String {
        VERIFIER_ROUTE.replace("{file_id}", &id.to_string())
    }
    fn tok(&self, who: &str) -> String {
        match who {
            "pemilik" | "penyusup" => token(&self.keys.pasien, "pasien", who),
            _ => token(&self.keys.admin, "admin", who),
        }
    }
}

// ----- test -----

// trace: US-201 (Verifikator membuka foto KTP: E-05 dulu), ADR-018 butir 3 (URL berumur pendek, lampiran),
// ADR-009 butir 2 (E-05 di-commit sebelum data dikirim)
#[tokio::test(flavor = "current_thread")]
async fn verifier_gets_short_lived_attachment_url_after_e05_is_recorded() {
    let _serial = SERIAL.lock().await;
    let w = world(Arc::new(SystemClock)).await;
    let (file_id, sha) = stored_ktp(
        &w.harness.pool,
        &w.harness.storage,
        w.owner,
        &jpeg_with_exif(6, 4),
    )
    .await;

    let started = now();
    let r = call(
        w.addr,
        &w.verifier_path(file_id),
        Some(&w.tok("verifikator")),
    )
    .await;
    assert_eq!(r.status, 200, "{}", r.body);
    let body = r.json();
    assert_eq!(body["mime_type"], "image/jpeg");
    let expires = body["expires_at"].as_i64().unwrap();
    let ttl = i64::try_from(DOWNLOAD_URL_TTL.as_secs()).unwrap();
    assert!(
        (expires - started.unix_timestamp() - ttl).abs() <= 5,
        "umur URL 5 menit"
    );
    let url = r.url();
    assert!(url.contains("X-Amz-Expires=300"));
    assert!(url.contains("files%2F") || url.contains("/files/"));

    // E-05 sukses tercatat dengan aktor Verifikator dan subjek data.
    let events = e05_events(file_id).await;
    assert_eq!(events.len(), 1, "{events:?}");
    let e = &events[0].event.body;
    assert_eq!(e.outcome, "success");
    assert_eq!(e.action_name, "stored_file_opened");
    assert_eq!(e.actor_account_id, Some(w.verifier));
    assert_eq!(e.object_type, "stored_file");
    assert_eq!(e.subject_id, Some(SUBJECT));

    // URL bekerja tanpa kredensial: isi = berkas tervalidasi, sebagai lampiran, tanpa cache.
    let (code, headers, bytes) = download(&url);
    assert_eq!(code, 200);
    assert_eq!(crypto::sha256(&bytes).to_vec(), sha);
    assert!(!contains(&bytes, b"Exif"));
    assert!(headers.contains("content-type: image/jpeg"), "{headers}");
    assert!(
        headers.contains(&format!(
            "content-disposition: attachment; filename=\"{file_id}.jpg\""
        )),
        "{headers}"
    );
    assert!(
        headers.contains("cache-control: private, no-store"),
        "{headers}"
    );

    // URL yang dirusak atau tanpa tanda tangan ditolak penyedia.
    let tampered = url.replace(
        "response-content-disposition=attachment",
        "response-content-disposition=inline",
    );
    assert_ne!(tampered, url);
    assert_eq!(download(&tampered).0, 403);
    let unsigned = url.split('?').next().unwrap().to_string();
    assert_eq!(download(&unsigned).0, 403);
}

// trace: ADR-018 butir 3 (URL kedaluwarsa ditolak)
#[tokio::test(flavor = "current_thread")]
async fn expired_download_url_is_rejected_by_provider() {
    let _serial = SERIAL.lock().await;
    // Jam aplikasi 6 menit di belakang: URL 5 menit sudah kedaluwarsa saat diterbitkan.
    let w = world(Arc::new(FixedClock::new(
        now() - time::Duration::minutes(6),
    )))
    .await;
    let (file_id, _) = stored_ktp(&w.harness.pool, &w.harness.storage, w.owner, &jpeg(4, 4)).await;
    let r = call(w.addr, &w.owner_path(file_id), Some(&w.tok("pemilik"))).await;
    assert_eq!(r.status, 200, "{}", r.body);
    assert_eq!(download(&r.url()).0, 403);
}

// trace: KU-03, ADR-012 butir 4, US-201 AC6 (bukan Verifikator / objek milik orang lain → ditolak dan tercatat)
#[tokio::test(flavor = "current_thread")]
async fn unauthorised_requests_get_no_url_and_are_logged() {
    let _serial = SERIAL.lock().await;
    let w = world(Arc::new(SystemClock)).await;
    let (file_id, _) = stored_ktp(&w.harness.pool, &w.harness.storage, w.owner, &jpeg(4, 4)).await;

    // Tanpa token: 401. Token realm pasien di rute backoffice: 401. Admin tanpa sub-peran Verifikator: 403.
    assert_eq!(
        call(w.addr, &w.verifier_path(file_id), None).await.status,
        401
    );
    assert_eq!(call(w.addr, &w.owner_path(file_id), None).await.status, 401);
    assert_eq!(
        call(w.addr, &w.verifier_path(file_id), Some(&w.tok("pemilik")))
            .await
            .status,
        401
    );
    let (sec, guard) = capture();
    let r = call(
        w.addr,
        &w.verifier_path(file_id),
        Some(&w.tok("layanan-pelanggan")),
    )
    .await;
    assert_eq!(r.status, 403, "{}", r.body);
    assert!(r.json().get("url").is_none());

    // Admin lolos kebijakan endpoint tetapi tanpa sub-peran Verifikator: objek → 404 + security log.
    let any = format!("/backoffice/v1/uji-any-admin-files/{file_id}/download-url");
    let r = call(w.addr, &any, Some(&w.tok("layanan-pelanggan"))).await;
    assert_eq!(r.status, 404, "{}", r.body);
    assert_eq!(r.json()["response_code"], "4040001");

    // Pasien lain membuka berkas milik pemilik: 404 sama dengan ID acak, + security log.
    let foreign = call(w.addr, &w.owner_path(file_id), Some(&w.tok("penyusup"))).await;
    let random = call(w.addr, &w.owner_path(new_id()), Some(&w.tok("penyusup"))).await;
    drop(guard);
    assert_eq!(foreign.status, 404);
    assert_eq!(random.status, 404);
    assert_eq!(
        foreign.json()["response_code"],
        random.json()["response_code"]
    );
    assert!(foreign.json().get("url").is_none());
    let log = sec.text();
    assert!(log.contains("access_denied"), "{log}");
    assert!(log.contains("foreign_resource_id"), "{log}");
    assert!(log.contains("role_not_allowed"), "{log}");
    assert!(log.contains(OWNER_ROUTE), "{log}");
    assert!(
        !log.contains(&file_id.to_string()),
        "ID objek tidak dicatat di route"
    );

    // Audit trail: dua percobaan atas berkas yang ada tercatat E-05 gagal; pemilik sendiri tidak memicu E-05.
    let ok = call(w.addr, &w.owner_path(file_id), Some(&w.tok("pemilik"))).await;
    assert_eq!(ok.status, 200);
    let events = e05_events(file_id).await;
    let failures: Vec<_> = events
        .iter()
        .map(|e| &e.event.body)
        .filter(|b| b.outcome == "failure")
        .collect();
    assert_eq!(failures.len(), 2, "{events:?}");
    for b in &failures {
        assert_eq!(b.outcome_reason_code.as_deref(), Some("access_denied"));
    }
    let actors: Vec<_> = failures.iter().map(|b| b.actor_account_id).collect();
    assert!(actors.contains(&Some(w.intruder)));
    assert!(actors.contains(&Some(w.customer_service)));
    assert_eq!(events.len(), 2, "akses pemilik sendiri bukan E-05");
}

// trace: coding-standards §4.4 butir 2 (E-05 gagal dicatat → akses ditolak, tanpa URL)
#[tokio::test(flavor = "current_thread")]
async fn audit_failure_denies_access_without_url() {
    let _serial = SERIAL.lock().await;
    let w = world(Arc::new(SystemClock)).await;
    let (file_id, _) = stored_ktp(&w.harness.pool, &w.harness.storage, w.owner, &jpeg(4, 4)).await;
    w.harness
        .broken_audit
        .store(true, std::sync::atomic::Ordering::SeqCst);
    let r = call(
        w.addr,
        &w.verifier_path(file_id),
        Some(&w.tok("verifikator")),
    )
    .await;
    assert_eq!(r.status, 500, "{}", r.body);
    assert!(r.json().get("url").is_none());
    assert!(e05_events(file_id).await.is_empty());
}

// trace: US-106 AC3 (jenis palsu 4150001, ukuran 4130001): berkas yang ditolak tidak pernah dapat diunduh;
// berkas kategori lain tidak dapat dibuka lewat endpoint KTP
#[tokio::test(flavor = "current_thread")]
async fn rejected_uploads_and_other_categories_are_never_downloadable() {
    let _serial = SERIAL.lock().await;
    let w = world(Arc::new(SystemClock)).await;
    let pool = &w.harness.pool;
    let storage = &w.harness.storage;
    let before: i64 = sqlx::query_scalar!(
        r#"SELECT count(*) AS "n!" FROM app.stored_file WHERE owner_account_id = $1"#,
        w.owner
    )
    .fetch_one(pool)
    .await
    .unwrap();
    // Jenis palsu (PNG dinyatakan JPEG; HTML) dan ukuran berlebih ditolak tanpa stored_file.
    let mut oversized = jpeg(16, 16);
    oversized.resize(MAX_PHOTO_BYTES as usize + 1, 0);
    for (ct, bytes, expected) in [
        ("image/jpeg", png(4, 4), "unsupported"),
        (
            "image/jpeg",
            b"<html><script>alert(1)</script></html>".to_vec(),
            "unsupported",
        ),
        ("image/jpeg", oversized, "too_large"),
    ] {
        let ticket = create_upload_slot(
            pool,
            &SystemClock,
            storage,
            UploadSlotRequest {
                owner_account_id: w.owner,
                purpose: UploadPurpose::KtpPhoto,
                content_type: ct.to_string(),
                size_bytes: 1000,
            },
        )
        .await
        .unwrap();
        let slot_key = files::slot_object_key(pool, ticket.upload_slot_id).await;
        // Ditulis langsung (lewat URL, penyedia menolak ukuran berlebih lebih dulu).
        storage
            .object_store()
            .put_object(
                &storage.bucket_name(LogicalBucket::IdentityDocument),
                &slot_key,
                ct,
                bytes,
                now(),
            )
            .await
            .unwrap();
        let r = validate_upload(
            pool,
            &SystemClock,
            storage,
            w.owner,
            ticket.upload_slot_id,
            UploadPurpose::KtpPhoto,
        )
        .await;
        match (expected, r) {
            ("unsupported", Err(FileError::UnsupportedType))
            | ("too_large", Err(FileError::TooLarge)) => {}
            (_, other) => panic!("{expected}: {other:?}"),
        }
    }
    let after: i64 = sqlx::query_scalar!(
        r#"SELECT count(*) AS "n!" FROM app.stored_file WHERE owner_account_id = $1"#,
        w.owner
    )
    .fetch_one(pool)
    .await
    .unwrap();
    assert_eq!(before, after, "berkas ditolak tidak pernah tercatat");

    // Berkas kategori lain (tercatat langsung, sintetis) tidak dapat dibuka lewat endpoint KTP: 404.
    let other = new_id();
    sqlx::query!(
        "INSERT INTO app.stored_file (id, created_by, updated_by, bucket, object_key, category, owner_account_id, \
         mime_type, size_bytes, sha256, validated_at) \
         VALUES ($1, $2, $2, 'claim_evidence', $3, 'claim_photo', $2, 'image/jpeg', 10, '\\x00', now())",
        other,
        w.owner,
        format!("files/{}", new_id()),
    )
    .execute(pool)
    .await
    .unwrap();
    let r = call(w.addr, &w.owner_path(other), Some(&w.tok("pemilik"))).await;
    assert_eq!(r.status, 404, "{}", r.body);
}
