//! Test lewat HTTP sungguhan di localhost (F1-T32, T1): klien JWKS [`HttpJwksSource`] terhadap server
//! JWKS uji, dan ekstraktor axum [`AuthContext`] dengan respons envelope.

#![allow(clippy::unwrap_used, clippy::expect_used)] // berkas test

use authn::testing::{TestRealmKey, jwks_document};
use authn::{
    Algorithm, AuthContext, AuthError, AuthState, Authenticator, AuthnConfig, Backend,
    ErrorRenderer, HttpJwksSource, MAX_JWKS_BYTES, Realm, RealmConfig,
};
use axum::Router;
use axum::extract::FromRef;
use axum::http::header::CONTENT_TYPE;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use envelope::{ApiError, CommonError};
use foundation::{CorrelationId, FixedClock};
use serde_json::{Value, json};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration as StdDuration;
use time::OffsetDateTime;
use time::macros::datetime;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

const ISS: &str = "https://auth.ruang-sehat.invalid/realms/pasien";
const AUD: &str = "rs-api-pasien";
const NOW: OffsetDateTime = datetime!(2027-03-01 08:00:00 UTC);

/// Server JWKS uji: status, isi, dan jeda dapat diganti.
#[derive(Clone)]
struct JwksServer {
    reply: Arc<Mutex<(u16, Vec<u8>, u64)>>,
}

impl JwksServer {
    async fn spawn(status: u16, body: Vec<u8>) -> (Self, SocketAddr) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let server = Self {
            reply: Arc::new(Mutex::new((status, body, 0))),
        };
        let s = server.clone();
        tokio::spawn(async move {
            loop {
                let Ok((mut sock, _)) = listener.accept().await else {
                    return;
                };
                let (status, body, delay_ms) = s.reply.lock().unwrap().clone();
                tokio::spawn(async move {
                    let mut buf = vec![0u8; 4096];
                    let mut read = Vec::new();
                    while !read.windows(4).any(|w| w == b"\r\n\r\n") {
                        match sock.read(&mut buf).await {
                            Ok(0) | Err(_) => return,
                            Ok(n) => read.extend_from_slice(buf.get(..n).unwrap_or_default()),
                        }
                    }
                    tokio::time::sleep(StdDuration::from_millis(delay_ms)).await;
                    let head = format!(
                        "HTTP/1.1 {status} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                        body.len()
                    );
                    let _ = sock.write_all(head.as_bytes()).await;
                    let _ = sock.write_all(&body).await;
                });
            }
        });
        (server, addr)
    }

    fn set(&self, status: u16, body: Vec<u8>, delay_ms: u64) {
        *self.reply.lock().unwrap() = (status, body, delay_ms);
    }
}

fn claims() -> Value {
    let now = NOW.unix_timestamp();
    json!({
        "iss": ISS, "aud": AUD, "sub": "6f1c1d1e-0000-4000-8000-000000000001",
        "typ": "Bearer", "azp": "app-pasien", "sid": "sesi-uji-1",
        "iat": now - 10, "exp": now + 300,
    })
}

fn authenticator(addr: SocketAddr, clock: Arc<FixedClock>) -> Authenticator {
    let jwks = HttpJwksSource::new(
        &format!("http://{addr}/realms/pasien/protocol/openid-connect/certs"),
        StdDuration::from_millis(500),
    )
    .unwrap();
    Authenticator::new(
        AuthnConfig::with_defaults(
            Backend::RuangSehat,
            vec![RealmConfig {
                realm: Realm::Pasien,
                issuer: ISS.into(),
                audiences: vec![AUD.into()],
                algorithms: vec![Algorithm::RS256],
                authorized_parties: vec![],
                jwks: Arc::new(jwks),
            }],
        ),
        clock,
    )
    .unwrap()
}

#[tokio::test]
async fn http_jwks_source_serves_keys_and_fails_closed() {
    let key = TestRealmKey::generate("k1").unwrap();
    let (server, addr) = JwksServer::spawn(200, jwks_document(&[key.jwk()])).await;
    let clock = Arc::new(FixedClock::new(NOW));
    let auth = authenticator(addr, clock.clone());
    let token = key.sign(&claims()).unwrap();
    assert_eq!(auth.verify(&token).await.unwrap().realm(), Realm::Pasien);

    let big = vec![b' '; MAX_JWKS_BYTES + 1];
    for (status, body, delay) in [
        (500, jwks_document(&[key.jwk()]), 0),
        (302, Vec::new(), 0),
        (200, b"bukan json".to_vec(), 0),
        (200, big, 0),
        (200, jwks_document(&[key.jwk()]), 2_000),
    ] {
        server.set(status, body, delay);
        // TTL 5 menit lewat: wajib ambil ulang; kegagalan = tolak (gagal-tertutup).
        clock.advance(time::Duration::minutes(6));
        let now = clock_now(&clock).unix_timestamp();
        let mut c = claims();
        c["iat"] = json!(now);
        c["exp"] = json!(now + 300);
        assert_eq!(
            auth.verify(&key.sign(&c).unwrap()).await.unwrap_err(),
            AuthError::KeysUnavailable,
            "status {status} delay {delay}"
        );
    }
}

fn clock_now(clock: &FixedClock) -> OffsetDateTime {
    foundation::Clock::now(clock)
}

#[tokio::test]
async fn unreachable_jwks_endpoint_fails_closed() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    drop(listener);
    let key = TestRealmKey::generate("k1").unwrap();
    let auth = authenticator(addr, Arc::new(FixedClock::new(NOW)));
    assert_eq!(
        auth.verify(&key.sign(&claims()).unwrap())
            .await
            .unwrap_err(),
        AuthError::KeysUnavailable
    );
}

// ----- ekstraktor axum -----

struct EnvelopeRenderer;

impl ErrorRenderer for EnvelopeRenderer {
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

#[derive(Clone)]
struct AppState {
    auth: AuthState,
}

impl FromRef<AppState> for AuthState {
    fn from_ref(s: &AppState) -> AuthState {
        s.auth.clone()
    }
}

async fn whoami(ctx: AuthContext) -> String {
    format!("{}:{}", ctx.realm(), ctx.subject())
}

async fn spawn_app(auth: Authenticator) -> SocketAddr {
    let state = AppState {
        auth: AuthState::new(Arc::new(auth), Arc::new(EnvelopeRenderer)),
    };
    let app = Router::new()
        .route("/whoami", get(whoami))
        .with_state(state)
        .layer(axum::middleware::from_fn(
            observability::correlation_middleware,
        ));
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    addr
}

async fn get_raw(addr: SocketAddr, headers: &[String]) -> (u16, Value, String) {
    let mut s = TcpStream::connect(addr).await.unwrap();
    let mut req = String::from("GET /whoami HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n");
    for h in headers {
        req.push_str(h);
        req.push_str("\r\n");
    }
    req.push_str("\r\n");
    s.write_all(req.as_bytes()).await.unwrap();
    let mut buf = Vec::new();
    s.read_to_end(&mut buf).await.unwrap();
    let text = String::from_utf8_lossy(&buf).into_owned();
    let status: u16 = text.get(9..12).unwrap().parse().unwrap();
    let body = text
        .split("\r\n\r\n")
        .nth(1)
        .unwrap_or_default()
        .to_string();
    let json = serde_json::from_str(&body).unwrap_or(Value::Null);
    (status, json, body)
}

#[tokio::test]
async fn extractor_accepts_valid_token_and_rejects_others_uniformly() {
    let key = TestRealmKey::generate("k1").unwrap();
    let (_server, addr) = JwksServer::spawn(200, jwks_document(&[key.jwk()])).await;
    let app = spawn_app(authenticator(addr, Arc::new(FixedClock::new(NOW)))).await;

    let token = key.sign(&claims()).unwrap();
    let (status, _, body) = get_raw(app, &[format!("Authorization: Bearer {token}")]).await;
    assert_eq!(status, 200);
    assert_eq!(body, "pasien:6f1c1d1e-0000-4000-8000-000000000001");

    let cid = "01936f8e-9c01-7e02-8f03-1a2b3c4d5e6f";
    let other = TestRealmKey::generate("k1")
        .unwrap()
        .sign(&claims())
        .unwrap();
    let mut expired = claims();
    expired["exp"] = json!(NOW.unix_timestamp() - 3600);
    let expired = key.sign(&expired).unwrap();
    let cases: Vec<Vec<String>> = vec![
        vec![],
        vec!["Authorization: Basic dXNlcjpwYXNz".into()],
        vec![format!("Authorization: Bearer {other}")],
        vec![format!("Authorization: Bearer {expired}")],
        vec![
            format!("Authorization: Bearer {token}"),
            format!("Authorization: Bearer {token}"),
        ],
    ];
    let mut bodies = Vec::new();
    for mut h in cases {
        h.push(format!("X-Correlation-Id: {cid}"));
        let (status, json, body) = get_raw(app, &h).await;
        assert_eq!(status, 401, "{h:?}");
        assert_eq!(json["response_code"], "4010001");
        assert_eq!(json["correlation_id"], cid);
        assert!(!body.contains(&token));
        bodies.push(body);
    }
    // Isi respons identik untuk semua alasan (tanpa alasan rinci, coding-standards §3.3).
    assert!(bodies.windows(2).all(|w| w[0] == w[1]));
}

#[tokio::test]
async fn extractor_returns_503_when_jwks_unavailable() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let dead = listener.local_addr().unwrap();
    drop(listener);
    let key = TestRealmKey::generate("k1").unwrap();
    let app = spawn_app(authenticator(dead, Arc::new(FixedClock::new(NOW)))).await;
    let token = key.sign(&claims()).unwrap();
    let (status, json, _) = get_raw(app, &[format!("Authorization: Bearer {token}")]).await;
    assert_eq!(status, 503);
    assert_eq!(json["response_code"], "5030001");
}
