//! IP klien asli dari proxy tepercaya (TD-22, KEPUTUSAN-400, prasyarat ADR-031): server sungguhan di localhost
//! (`build_router`, `ConnectInfo`). Peer test selalu `127.0.0.1`; ia berperan sebagai Caddy bila
//! `TRUSTED_PROXY_CIDRS` memuatnya, dan sebagai klien langsung bila tidak.
//!
//! Batas laju per sumber diperiksa sebelum database disentuh, sehingga test batas laju memakai pool yang tidak
//! pernah tersambung: jawaban selain 429 berarti lolos batas laju. Test sumber audit memakai PostgreSQL efemeral
//! (`scripts/db-ephemeral.sh`).
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)] // berkas test

mod common;

use foundation::SystemClock;
use observability::EnvSource;
use rs_backend::app::{AppServices, AppState, RegistrationGuard};
use rs_backend::http_support::TrustedProxies;
use rs_backend::rate_limit::{RateBudget, RateLimiter};
use serde_json::{Value, json};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

const REGISTER: &str = "/v1/public/corporate-registrations";
const CORPORATE_CONSENT: &str = "/v1/public/corporate-registrations/consent-text";
const PATIENT_CONSENT: &str = "/patient/v1/consent-texts/current";
/// Seed draf teks persetujuan HR dan butir wajibnya (migrasi 20261126310000).
const TEXT_VERSION: &str = "01a2008d-7400-7000-8000-000000000b01";
const TEXT_ITEM: &str = "01a2008d-7400-7000-8000-000000000b02";

fn budget(per_source: u32) -> RateBudget {
    RateBudget {
        per_source,
        global: 10_000,
    }
}

fn proxies(s: &str) -> TrustedProxies {
    s.parse().unwrap()
}

/// Server dengan anggaran per sumber 2 untuk POST pendaftaran, GET teks persetujuan HR, dan GET teks
/// persetujuan pasien.
async fn spawn(pool: PgPool, trusted: TrustedProxies, per_source: u32) -> SocketAddr {
    let keys = common::TestKeys::generate();
    let files = Arc::new(common::unused_file_storage());
    let state = AppState::new(
        common::TYPE_BASE,
        Vec::new(),
        AppServices {
            pool: pool.clone(),
            clock: Arc::new(SystemClock),
            identity: common::unused_identity_services(pool.clone(), Arc::clone(&files)),
            files,
            public_content_limit: common::roomy_limiter(),
            public_image_limit: common::roomy_limiter(),
            corporate_hr: common::corporate_hr(pool.clone()),
            consent_text_limit: Arc::new(RateLimiter::new(budget(per_source))),
        },
    )
    .with_registration_guard(RegistrationGuard {
        limiter: RateLimiter::new(budget(per_source)),
        consent_text_limiter: RateLimiter::new(budget(per_source)),
        min_response: Duration::ZERO,
    })
    .with_trusted_proxies(trusted);
    let auth = common::auth_state(&keys, Arc::new(SystemClock));
    let directory = common::account_directory(pool);
    common::spawn_app(state, auth, directory.clone(), directory).await
}

async fn status(addr: SocketAddr, method: &str, path: &str, forwarded_for: Option<&str>) -> u16 {
    let header = forwarded_for.map(|v| format!("X-Forwarded-For: {v}"));
    let headers: Vec<&str> = header.iter().map(String::as_str).collect();
    if method == "POST" {
        // Badan JSON kosong: lolos batas laju lalu ditolak validasi, tanpa database.
        common::request_json(addr, method, path, &headers, &json!({}))
            .await
            .status
    } else {
        common::request(addr, method, path, &headers).await.status
    }
}

// trace: TD-22, ADR-031 Konsekuensi (prasyarat keras `/daftar`), US-1901 AC4, US-104 AC1: dua klien berbeda di
// balik proxy tepercaya tidak saling memblokir; pemalsuan di kiri tidak menghindari batas
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn per_source_limits_apply_to_real_client_behind_trusted_proxy() {
    let addr = spawn(common::unreachable_pool(), proxies("127.0.0.1/32"), 2).await;
    for (method, path) in [
        ("POST", REGISTER),
        ("GET", CORPORATE_CONSENT),
        ("GET", PATIENT_CONSENT),
    ] {
        let a = Some("198.51.100.1");
        let first = status(addr, method, path, a).await;
        assert_ne!(first, 429, "{path}");
        assert_ne!(status(addr, method, path, a).await, 429, "{path}");
        assert_eq!(
            status(addr, method, path, a).await,
            429,
            "{path}: klien A habis"
        );
        // Klien A memalsukan alamat lain di kiri: tetap dihitung sebagai A (alamat paling kanan).
        assert_eq!(
            status(addr, method, path, Some("198.51.100.2, 198.51.100.1")).await,
            429,
            "{path}: pemalsuan di kiri"
        );
        // Klien B dan klien IPv6 di balik proxy yang sama tidak terblokir.
        assert_eq!(
            status(addr, method, path, Some("198.51.100.2")).await,
            first,
            "{path}: klien B"
        );
        assert_eq!(
            status(addr, method, path, Some("2001:db8::7")).await,
            first,
            "{path}: klien IPv6"
        );
    }
}

// trace: TD-22 (peer tak tepercaya: X-Forwarded-For tidak pernah dipercaya; bawaan = perilaku lama)
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn forwarded_for_from_untrusted_peer_is_ignored() {
    for trusted in [TrustedProxies::none(), proxies("10.0.0.0/8")] {
        let addr = spawn(common::unreachable_pool(), trusted, 2).await;
        assert_ne!(
            status(addr, "POST", REGISTER, Some("198.51.100.1")).await,
            429
        );
        assert_ne!(
            status(addr, "POST", REGISTER, Some("198.51.100.2")).await,
            429
        );
        // Alamat palsu yang berganti-ganti tidak memberi anggaran baru: semuanya peer 127.0.0.1.
        assert_eq!(
            status(addr, "POST", REGISTER, Some("198.51.100.3")).await,
            429
        );
        assert_eq!(status(addr, "POST", REGISTER, None).await, 429);
    }
}

// trace: TD-22 (header tidak valid dari proxy tepercaya = peer TCP)
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn invalid_forwarded_for_from_trusted_peer_counts_as_proxy() {
    let addr = spawn(common::unreachable_pool(), proxies("127.0.0.1"), 2).await;
    // Rantai sah tetapi lebih dari 1024 byte (spasi di awal nilai dipangkas parser HTTP, jadi pakai entri).
    let long = format!("{}198.51.100.9", "203.0.113.1, ".repeat(100));
    assert_ne!(status(addr, "POST", REGISTER, Some("bukan-ip")).await, 429);
    assert_ne!(status(addr, "POST", REGISTER, Some(&long)).await, 429);
    // Peer (127.0.0.1) sudah memakai anggarannya: tanpa header pun ditolak.
    assert_eq!(status(addr, "POST", REGISTER, None).await, 429);
    assert_eq!(
        status(addr, "POST", REGISTER, Some("198.51.100.9:443")).await,
        429
    );
    // Klien dengan header sah tetap punya anggaran sendiri.
    assert_ne!(
        status(addr, "POST", REGISTER, Some("198.51.100.9")).await,
        429
    );
}

async fn db_pool(var: &str) -> PgPool {
    let url = EnvSource::from_process()
        .required_secret(var)
        .unwrap_or_else(|_| {
            panic!(
                "{var} tidak diset: jalankan lewat `bash scripts/db-ephemeral.sh cargo test ...`"
            )
        });
    PgPoolOptions::new()
        .max_connections(4)
        .connect(url.expose())
        .await
        .unwrap()
}

fn random_digits(n: usize) -> String {
    let mut b = vec![0u8; n];
    crypto::fill_random(&mut b).unwrap();
    b.iter().map(|x| char::from(b'0' + x % 10)).collect()
}

// trace: PRD §7.2 "Sumber", TD-22: field sumber audit (E-10/E-11/E-13 pendaftaran) memuat IP klien asli dari
// proxy tepercaya, bukan alamat proxy; tidak ada field baru
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn audit_source_ip_is_real_client_behind_trusted_proxy() {
    let app = db_pool("RS_TEST_DB_APP_URL").await;
    let relay = db_pool("RS_TEST_DB_RELAY_URL").await;
    let addr = spawn(app, proxies("127.0.0.1/32"), 10).await;
    let nib = random_digits(13);
    let body = json!({
        "legal_name": format!("PT Uji Proxy {nib}"),
        "nib": nib,
        "npwp": random_digits(16),
        "office_address": "Jl. Uji Sintetis No. 7, Jakarta",
        "office_phone": "021-5550101",
        "hr_full_name": "Sari Pendaftar Sintetis",
        "hr_job_title": "Manajer SDM Sintetis",
        "hr_email": format!("proxy-{nib}@example.com"),
        "consent_text_version_id": TEXT_VERSION,
        "consent_item_ids": [TEXT_ITEM],
    });
    let r = common::request_json(
        addr,
        "POST",
        REGISTER,
        &["X-Forwarded-For: 203.0.113.250, 198.51.100.44"],
        &body,
    )
    .await;
    assert_eq!(r.status, 202, "{}", r.body);
    let cid = r.json()["correlation_id"].as_str().unwrap().to_string();
    let events: Vec<Value> = sqlx::query_scalar!(
        r#"SELECT payload AS "payload!" FROM app.audit_outbox
               WHERE payload ->> 'correlation_id' = $1 ORDER BY occurred_at, id"#,
        cid,
    )
    .fetch_all(&relay)
    .await
    .unwrap();
    assert!(!events.is_empty());
    for e in &events {
        assert_eq!(e["source_ip"], "198.51.100.44", "{e}");
    }
}
