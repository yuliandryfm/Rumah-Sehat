//! Probe timing penolakan objek (F1-T38, TD-15 Low, KU-03), dijalankan dalam build release oleh
//! `tests/denial_timing_release.rs`. Bukan alat produksi.
//!
//! Server uji di 127.0.0.1 dengan middleware penegak sungguhan dan security log yang sink-nya sengaja
//! lambat (`SLOW_SINK`). Tiga jalur diukur bergantian, masing-masing `ROUNDS` permintaan:
//! - `missing`: objek tidak ada (`404`, tanpa pencatatan);
//! - `foreign`: objek milik orang lain lewat `authz::visible_object` (`404`, dicatat di latar belakang);
//! - `sync_control`: objek milik orang lain yang security log-nya ditulis sinkron di handler (cara lama),
//!   sebagai kontrol bahwa probe memang dapat melihat selisih yang disebabkan log sinkron.
//!
//! Keluaran satu baris: `missing_us=<median> foreign_us=<median> sync_control_us=<median>`.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)] // probe test

use authn::testing::{StaticJwks, TestRealmKey, jwks_document};
use authn::{
    Algorithm, AuthContext, AuthState, Authenticator, AuthnConfig, Backend, ErrorRenderer, Realm,
    RealmConfig,
};
use authz::{
    AccountRecord, AccountStatus, AccountType, DenialSink, Enforcement, Policy, PolicyRouter,
    Principal, PrincipalResolver, ResolveFuture, visible_object,
};
use axum::extract::Path;
use axum::http::header::CONTENT_TYPE;
use axum::response::{IntoResponse, Response};
use envelope::{ApiError, CommonError};
use foundation::{CorrelationId, FixedClock, SystemClock, Uuid};
use observability::LogLevel;
use observability::log::{Component, JsonLogLayer, LogSink, subscriber};
use observability::security::{self, AccessDenial};
use serde_json::json;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use utoipa_axum::routes;

const ROUNDS: usize = 300;
const SLOW_SINK: Duration = Duration::from_millis(3);
const ISS: &str = "https://auth.ruang-sehat.invalid/realms/pasien";
const AUD: &str = "rs-api-pasien";

fn owner() -> Uuid {
    Uuid::from_u128(0x0193_6f8e_9c01_7e02_8f03_1a2b_3c4d_5e0a)
}
fn other() -> Uuid {
    Uuid::from_u128(0x0193_6f8e_9c01_7e02_8f03_1a2b_3c4d_5e0b)
}

/// Sink log lambat (meniru penulisan log yang tertahan I/O).
struct SlowSink;

impl LogSink for SlowSink {
    fn write_line(&self, _line: &str) {
        std::thread::sleep(SLOW_SINK);
    }
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

struct OneResolver;

impl PrincipalResolver for OneResolver {
    fn resolve<'a>(&'a self, _ctx: &'a AuthContext) -> ResolveFuture<'a> {
        Box::pin(async move {
            Ok(Some(AccountRecord {
                account_id: owner(),
                account_type: AccountType::Patient,
                status: AccountStatus::Active,
                sub_roles: vec![],
            }))
        })
    }
}

fn lookup(id: &str) -> Option<Uuid> {
    match id {
        "milik-sendiri" => Some(owner()),
        "milik-orang-lain" => Some(other()),
        _ => None,
    }
}

fn not_found() -> Response {
    let body =
        CommonError::NotFound.to_error_body(CorrelationId::default(), "https://d.example/problems");
    (
        CommonError::NotFound.http_status(),
        [(CONTENT_TYPE, "application/problem+json")],
        serde_json::to_vec(&body).unwrap(),
    )
        .into_response()
}

#[utoipa::path(get, path = "/patient/v1/orders/{order_id}", params(("order_id" = String, Path)), responses((status = 200)))]
async fn order(p: Principal, Path(id): Path<String>) -> Response {
    match visible_object(&p, "/patient/v1/orders/{order_id}", lookup(&id), |o| {
        *o == p.account_id()
    }) {
        Ok(_) => "ok".into_response(),
        Err(_) => not_found(),
    }
}

#[utoipa::path(get, path = "/patient/v1/sync-orders/{order_id}", params(("order_id" = String, Path)), responses((status = 200)))]
async fn sync_order(p: Principal, Path(id): Path<String>) -> Response {
    match lookup(&id) {
        Some(o) if o == p.account_id() => "ok".into_response(),
        Some(_) => {
            // Cara lama (kontrol): security log ditulis sinkron di jalur permintaan.
            security::access_denied(
                Some(p.account_id()),
                "/patient/v1/sync-orders/{order_id}",
                AccessDenial::ForeignResourceId,
            );
            not_found()
        }
        None => not_found(),
    }
}

async fn get(addr: SocketAddr, path: &str, bearer: &str) -> (u16, Duration) {
    let started = Instant::now();
    let mut s = TcpStream::connect(addr).await.unwrap();
    let req = format!(
        "GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\nAuthorization: Bearer {bearer}\r\n\r\n"
    );
    s.write_all(req.as_bytes()).await.unwrap();
    let mut buf = Vec::new();
    s.read_to_end(&mut buf).await.unwrap();
    let elapsed = started.elapsed();
    let status = String::from_utf8_lossy(buf.get(9..12).unwrap_or_default())
        .parse()
        .unwrap_or(0);
    (status, elapsed)
}

fn median(mut v: Vec<Duration>) -> u128 {
    v.sort();
    v.get(v.len() / 2).copied().unwrap_or_default().as_micros()
}

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() {
    let layer = JsonLogLayer::new(Component::Api, LogLevel::Info, Arc::new(SystemClock))
        .with_sinks(Arc::new(SlowSink), Arc::new(SlowSink));
    tracing::subscriber::set_global_default(subscriber(layer)).unwrap();

    let now = foundation::Clock::now(&SystemClock);
    let key = TestRealmKey::generate("k").unwrap();
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
    let t = now.unix_timestamp();
    let token = key
        .sign(&json!({
            "iss": ISS, "aud": AUD, "sub": "s", "typ": "Bearer", "sid": "sid",
            "azp": "app", "iat": t - 5, "exp": t + 3600,
        }))
        .unwrap();
    let denials = DenialSink::start(8192, 1, None).unwrap();
    let (router, _) = PolicyRouter::<()>::new()
        .routes(Policy::patient(), routes!(order))
        .routes(Policy::patient(), routes!(sync_order))
        .finish(Enforcement::new(
            AuthState::new(Arc::new(auth), Arc::new(Renderer)),
            Arc::new(OneResolver),
            denials,
        ))
        .unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        router
            .serve(listener, std::future::pending())
            .await
            .unwrap()
    });

    // Pemanasan (koneksi, JWKS, alokasi).
    for _ in 0..30 {
        get(addr, "/patient/v1/orders/milik-sendiri", &token).await;
    }
    let mut missing = Vec::with_capacity(ROUNDS);
    let mut foreign = Vec::with_capacity(ROUNDS);
    let mut control = Vec::with_capacity(ROUNDS);
    for _ in 0..ROUNDS {
        let (s1, d1) = get(addr, "/patient/v1/orders/tidak-ada", &token).await;
        let (s2, d2) = get(addr, "/patient/v1/orders/milik-orang-lain", &token).await;
        let (s3, d3) = get(addr, "/patient/v1/sync-orders/milik-orang-lain", &token).await;
        assert_eq!((s1, s2, s3), (404, 404, 404));
        missing.push(d1);
        foreign.push(d2);
        control.push(d3);
    }
    println!(
        "missing_us={} foreign_us={} sync_control_us={}",
        median(missing),
        median(foreign),
        median(control)
    );
}
