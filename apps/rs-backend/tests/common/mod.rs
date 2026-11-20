//! Pembantu test: server sungguhan di localhost, klien HTTP/1.1 mentah, dan perangkat autentikasi uji
//! (kunci realm uji + JWKS statis dari `authn::testing`, tanpa Keycloak; coding-standards §7.2).
#![allow(dead_code)] // tiap berkas test memakai sebagian
#![allow(clippy::unwrap_used, clippy::expect_used)] // berkas pembantu test

use authn::testing::{StaticJwks, TestRealmKey, jwks_document};
use authn::{
    Algorithm, AuthContext, AuthState, Authenticator, AuthnConfig, Backend, Realm, RealmConfig,
};
use authz::{PrincipalResolver, ResolveFuture};
use foundation::{Clock, SystemClock};
use observability::EnvSource;
use rs_backend::app::{AppServices, AppState, build_router};
use rs_backend::http_support::EnvelopeRenderer;
use rs_backend::modules::platform::api::{FileStorage, FileStorageConfig, ReadinessCheck};
use rs_backend::rate_limit::{RateBudget, RateLimiter};
use serde_json::{Value, json};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

pub const TYPE_BASE: &str = "https://d.example/problems";
pub const ADMIN_ISS: &str = "https://auth.ruang-sehat.invalid/realms/admin";
pub const ADMIN_AUD: &str = "rs-backoffice-api";
pub const PASIEN_ISS: &str = "https://auth.ruang-sehat.invalid/realms/pasien";

pub fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap()
}

/// Layanan berkas dengan endpoint lokal yang tidak dipakai (cukup untuk endpoint tanpa object storage).
pub fn unused_file_storage() -> FileStorage {
    let env = EnvSource::from_pairs([
        ("DEPLOY_ENV", "local"),
        ("OBJECT_STORAGE_ENDPOINT", "http://127.0.0.1:9"),
        ("OBJECT_STORAGE_REGION", "us-east-1"),
        ("OBJECT_STORAGE_ADDRESSING", "path"),
        ("OBJECT_STORAGE_ACCESS_KEY_ID", "sintetis-uji-akses"),
        (
            "OBJECT_STORAGE_SECRET_ACCESS_KEY",
            "sintetis-uji-rahasia-0123456789abcdef",
        ),
        ("OBJECT_STORAGE_BUCKET_PREFIX", "rs-uji-"),
    ]);
    FileStorage::new(&FileStorageConfig::load(&env).unwrap()).unwrap()
}

/// Pembatas laju longgar untuk test yang tidak menguji batas laju.
pub fn roomy_limiter() -> Arc<RateLimiter> {
    Arc::new(RateLimiter::new(RateBudget {
        per_source: 10_000,
        global: 10_000,
    }))
}

/// Pool yang tidak pernah tersambung (untuk test tanpa database).
pub fn unreachable_pool() -> PgPool {
    PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_millis(200))
        .connect_lazy("postgres://uji:uji@127.0.0.1:9/uji")
        .unwrap()
}

/// Kunci realm uji `admin` dan `pasien`.
pub struct TestKeys {
    pub admin: TestRealmKey,
    pub pasien: TestRealmKey,
}

impl TestKeys {
    pub fn generate() -> Self {
        Self {
            admin: TestRealmKey::generate("kid-admin").unwrap(),
            pasien: TestRealmKey::generate("kid-pasien").unwrap(),
        }
    }
}

/// Validator token realm `admin` (kunci uji) seperti konfigurasi proses `api`.
pub fn auth_state(keys: &TestKeys, clock: Arc<dyn Clock>) -> AuthState {
    let realm = RealmConfig {
        realm: Realm::Admin,
        issuer: ADMIN_ISS.to_string(),
        audiences: vec![ADMIN_AUD.to_string()],
        algorithms: vec![Algorithm::RS256],
        authorized_parties: vec![],
        jwks: Arc::new(StaticJwks::new(jwks_document(&[keys.admin.jwk()]))),
    };
    let authenticator = Authenticator::new(
        AuthnConfig::with_defaults(Backend::RuangSehat, vec![realm]),
        clock,
    )
    .unwrap();
    AuthState::new(
        Arc::new(authenticator),
        Arc::new(EnvelopeRenderer::new(TYPE_BASE)),
    )
}

/// Token akses bergaya Keycloak untuk `sub` (berlaku 5 menit dari jam sistem).
pub fn token(key: &TestRealmKey, iss: &str, aud: &str, sub: &str) -> String {
    let now = SystemClock.now().unix_timestamp();
    key.sign(&json!({
        "iss": iss, "aud": aud, "sub": sub, "typ": "Bearer",
        "sid": "sesi-uji", "azp": "rs-backoffice-bff", "iat": now - 5, "exp": now + 300,
    }))
    .unwrap()
}

/// Resolver yang tidak mengenal akun mana pun (untuk test endpoint publik).
pub struct NoAccounts;

impl PrincipalResolver for NoAccounts {
    fn resolve<'a>(&'a self, _ctx: &'a AuthContext) -> ResolveFuture<'a> {
        Box::pin(async { Ok(None) })
    }
}

/// Menjalankan router lengkap di localhost dengan `ConnectInfo` (alamat klien untuk audit dan batas laju).
pub async fn spawn_app(
    state: AppState,
    auth: AuthState,
    resolver: Arc<dyn PrincipalResolver>,
) -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let app = build_router(state, auth, resolver).unwrap();
    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });
    addr
}

/// Server tanpa database dan tanpa akun (endpoint kesehatan dan penolakan).
pub async fn spawn(readiness: Vec<Arc<dyn ReadinessCheck>>) -> SocketAddr {
    let clock: Arc<dyn Clock> = Arc::new(SystemClock);
    let keys = TestKeys::generate();
    let state = AppState::new(
        TYPE_BASE,
        readiness,
        AppServices {
            pool: unreachable_pool(),
            clock: Arc::clone(&clock),
            files: Arc::new(unused_file_storage()),
            public_content_limit: roomy_limiter(),
            public_image_limit: roomy_limiter(),
        },
    );
    spawn_app(state, auth_state(&keys, clock), Arc::new(NoAccounts)).await
}

pub struct Resp {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

impl Resp {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }

    pub fn json(&self) -> Value {
        serde_json::from_str(&self.body).unwrap()
    }
}

pub async fn request(addr: SocketAddr, method: &str, path: &str, extra: &[&str]) -> Resp {
    request_with_body(addr, method, path, extra, None).await
}

/// Permintaan dengan badan JSON opsional (`Content-Type: application/json` ditambahkan bila ada badan).
pub async fn request_json(
    addr: SocketAddr,
    method: &str,
    path: &str,
    extra: &[&str],
    body: &Value,
) -> Resp {
    let text = body.to_string();
    let mut headers: Vec<&str> = extra.to_vec();
    headers.push("Content-Type: application/json");
    request_with_body(addr, method, path, &headers, Some(text.as_bytes())).await
}

pub async fn request_with_body(
    addr: SocketAddr,
    method: &str,
    path: &str,
    extra: &[&str],
    body: Option<&[u8]>,
) -> Resp {
    let mut s = TcpStream::connect(addr).await.unwrap();
    let body = body.unwrap_or_default();
    let mut req = format!(
        "{method} {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\nContent-Length: {}\r\n",
        body.len()
    );
    for h in extra {
        req.push_str(h);
        req.push_str("\r\n");
    }
    req.push_str("\r\n");
    let mut bytes = req.into_bytes();
    bytes.extend_from_slice(body);
    s.write_all(&bytes).await.unwrap();
    let mut buf = Vec::new();
    s.read_to_end(&mut buf).await.unwrap();
    let text = String::from_utf8_lossy(&buf).into_owned();
    let (head, body) = text.split_once("\r\n\r\n").unwrap();
    let mut lines = head.lines();
    let status = lines
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|c| c.parse().ok())
        .unwrap();
    let headers = lines
        .filter_map(|l| l.split_once(':'))
        .map(|(k, v)| (k.trim().to_string(), v.trim().to_string()))
        .collect();
    Resp {
        status,
        headers,
        body: body.to_string(),
    }
}
