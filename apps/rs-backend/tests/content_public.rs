//! Endpoint konten publik tanpa login (F1-T94; US-1801 AC1, AC3, AC5, US-1802 AC1-AC2, ADR-012 butir 3,
//! ADR-027 butir 5) terhadap PostgreSQL 18 (role `rs_app`): hanya konten Tayang kanal situs; draf,
//! terjadwal, berakhir, diarsipkan, terhapus, dan konten kanal aplikasi tidak bocor (404 identik dengan
//! ID acak); tanpa cookie; batas laju per sumber dan global. Konten disiapkan lewat API backoffice
//! sungguhan (admin Konten dengan token realm uji).
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)] // berkas test

mod common;

use common::{ADMIN_AUD, ADMIN_ISS, Resp, TestKeys, request, request_json, token};
use foundation::{FixedClock, SystemClock, Uuid, new_id};
use observability::EnvSource;
use rs_backend::app::{AppServices, AppState};
use rs_backend::rate_limit::{RateBudget, RateLimiter};
use serde_json::{Value, json};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use std::sync::Arc;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use time::macros::datetime;

// Epoch berbeda dari tests/content_cms.rs (2030) supaya data uji berkas lain tidak sedang tayang di sini.
const T0: OffsetDateTime = datetime!(2031-05-01 01:00:00 UTC);
const ROOMY: RateBudget = RateBudget {
    per_source: 10_000,
    global: 10_000,
};

async fn app_pool() -> PgPool {
    let url = EnvSource::from_process()
        .required_secret("RS_TEST_DB_APP_URL")
        .unwrap_or_else(|_| {
            panic!("RS_TEST_DB_APP_URL tidak diset: jalankan lewat `bash scripts/db-ephemeral.sh cargo test ...`")
        });
    PgPoolOptions::new()
        .max_connections(4)
        .connect(url.expose())
        .await
        .unwrap()
}

async fn content_admin(pool: &PgPool) -> String {
    let id = new_id();
    let sub = format!("uji-konten-{}", id.simple());
    sqlx::query!(
        "INSERT INTO app.account (id, account_type, idp_realm, idp_subject, email, status, status_changed_at, full_name) \
         VALUES ($1, 'rs_admin', 'admin', $2, $3, 'active', now(), 'Admin Konten Sintetis')",
        id,
        sub,
        format!("{sub}@example.com"),
    )
    .execute(pool)
    .await
    .unwrap();
    sqlx::query!(
        "INSERT INTO app.rs_admin_role (id, account_id, sub_role) VALUES ($1, $2, 'content')",
        new_id(),
        id,
    )
    .execute(pool)
    .await
    .unwrap();
    sub
}

struct Fixture {
    addr: SocketAddr,
    clock: Arc<FixedClock>,
    bearer: String,
}

impl Fixture {
    async fn new(budget: RateBudget) -> Self {
        let pool = app_pool().await;
        let keys = TestKeys::generate();
        let clock = Arc::new(FixedClock::new(T0));
        let state = AppState::new(
            common::TYPE_BASE,
            Vec::new(),
            AppServices {
                pool: pool.clone(),
                clock: clock.clone(),
                identity: common::unused_identity_services(
                    pool.clone(),
                    Arc::new(common::unused_file_storage()),
                ),
                files: Arc::new(common::unused_file_storage()),
                public_content_limit: Arc::new(RateLimiter::new(budget)),
                public_image_limit: Arc::new(RateLimiter::new(ROOMY)),
                corporate_hr: common::corporate_hr(common::unreachable_pool()),
                consent_text_limit: common::roomy_limiter(),
            },
        );
        let auth = common::auth_state(&keys, Arc::new(SystemClock));
        let addr = common::spawn_app(
            state,
            auth,
            common::account_directory(pool.clone()),
            common::account_directory(pool.clone()),
        )
        .await;
        let sub = content_admin(&pool).await;
        let bearer = format!(
            "Authorization: Bearer {}",
            token(&keys.admin, ADMIN_ISS, ADMIN_AUD, &sub)
        );
        Self {
            addr,
            clock,
            bearer,
        }
    }

    async fn public(&self, path: &str) -> Resp {
        request(self.addr, "GET", path, &[]).await
    }

    async fn admin(&self, method: &str, path: &str, body: &Value) -> Resp {
        request_json(self.addr, method, path, &[&self.bearer], body).await
    }

    async fn draft(&self, body: Value) -> String {
        let r = self.admin("POST", "/backoffice/v1/contents", &body).await;
        assert_eq!(r.status, 201, "{}", r.body);
        r.json()["data"]["content_id"].as_str().unwrap().to_string()
    }

    async fn schedule(&self, id: &str, body: Value) {
        let r = self
            .admin(
                "POST",
                &format!("/backoffice/v1/contents/{id}/schedule"),
                &body,
            )
            .await;
        assert_eq!(r.status, 200, "{}", r.body);
    }
}

fn page(p: &str, title: &str, channel: &str) -> Value {
    json!({"content_type": "info_page", "title": title, "body": "## Isi\n\nTeks sintetis.", "channel": channel, "site_page": p})
}

fn rfc(t: OffsetDateTime) -> String {
    t.format(&Rfc3339).unwrap()
}

/// Isi galat tanpa ID korelasi (yang selalu berbeda per permintaan).
fn without_cid(r: &Resp) -> Value {
    let mut v = r.json();
    v.as_object_mut().unwrap().remove("correlation_id");
    v
}

// trace: US-1801 AC1, US-1801 AC5
#[tokio::test(flavor = "multi_thread")]
async fn live_fixed_page_is_public_without_cookie_or_editorial_fields() {
    let f = Fixture::new(ROOMY).await;
    let id = f.draft(page("about", "Tentang Ruang Sehat", "site")).await;
    f.schedule(&id, json!({})).await;
    let r = f.public("/v1/public/content/pages/about").await;
    assert_eq!(r.status, 200, "{}", r.body);
    assert_eq!(r.header("set-cookie"), None);
    assert_eq!(r.header("cache-control"), Some("no-store"));
    let d = &r.json()["data"];
    assert_eq!(d["content_id"], id.as_str());
    assert_eq!(d["title"], "Tentang Ruang Sehat");
    assert_eq!(d["published_at"], rfc(T0));
    let keys: Vec<&str> = d.as_object().unwrap().keys().map(String::as_str).collect();
    for hidden in [
        "editorial_status",
        "display_status",
        "deleted_at",
        "delete_reason",
        "created_at",
        "publish_end_at",
    ] {
        assert!(!keys.contains(&hidden), "{hidden} bocor: {keys:?}");
    }
    let item = f.public(&format!("/v1/public/content/items/{id}")).await;
    assert_eq!(item.status, 200);
}

// trace: US-1801 AC3
#[tokio::test(flavor = "multi_thread")]
async fn draft_scheduled_ended_archived_deleted_and_app_only_content_do_not_leak() {
    let f = Fixture::new(ROOMY).await;
    let h = time::Duration::hours(1);
    let random = Uuid::from_u128(0x0193_6f8e_9c01_7e02_8f03_1a2b_3c4d_5e77);
    let baseline = f
        .public(&format!("/v1/public/content/items/{random}"))
        .await;
    assert_eq!(baseline.status, 404);
    let baseline = without_cid(&baseline);

    let draft = f.draft(page("home", "Beranda draf", "site")).await;
    let future = f.draft(page("services", "Layanan terjadwal", "site")).await;
    f.schedule(&future, json!({"publish_start_at": rfc(T0 + h)}))
        .await;
    let ended = f
        .draft(json!({"content_type": "article", "title": "Berakhir", "channel": "site", "site_page": "articles"}))
        .await;
    f.schedule(
        &ended,
        json!({"publish_end_at": rfc(T0 + time::Duration::minutes(1))}),
    )
    .await;
    let archived = f
        .draft(json!({"content_type": "promo", "title": "Promo", "channel": "site", "site_page": "home"}))
        .await;
    f.schedule(&archived, json!({})).await;
    let r = f
        .admin(
            "POST",
            &format!("/backoffice/v1/contents/{archived}/archive"),
            &json!({}),
        )
        .await;
    assert_eq!(r.status, 200);
    let deleted = f
        .draft(json!({"content_type": "article", "title": "Terhapus", "channel": "app_and_site", "site_page": "articles"}))
        .await;
    f.schedule(&deleted, json!({})).await;
    let r = f
        .admin(
            "POST",
            &format!("/backoffice/v1/contents/{deleted}/delete"),
            &json!({"delete_reason": "uji"}),
        )
        .await;
    assert_eq!(r.status, 200);
    let app_only = f
        .draft(json!({"content_type": "banner", "title": "Banner aplikasi", "channel": "app"}))
        .await;
    f.schedule(&app_only, json!({})).await;

    f.clock.set(T0 + time::Duration::minutes(5));
    for (label, id) in [
        ("draf", &draft),
        ("terjadwal", &future),
        ("berakhir", &ended),
        ("diarsipkan", &archived),
        ("terhapus", &deleted),
        ("kanal aplikasi", &app_only),
    ] {
        let r = f.public(&format!("/v1/public/content/items/{id}")).await;
        assert_eq!(r.status, 404, "{label}");
        assert_eq!(
            without_cid(&r),
            baseline,
            "{label}: isi 404 berbeda dari ID acak"
        );
        assert!(!r.body.contains(id.as_str()), "{label}");
    }
    // Halaman tetap draf/terjadwal: halaman tidak ditemukan, sama dengan halaman tanpa konten.
    for p in ["home", "services"] {
        let r = f.public(&format!("/v1/public/content/pages/{p}")).await;
        assert_eq!(r.status, 404, "{p}");
        assert_eq!(without_cid(&r), baseline, "{p}");
    }
    // Daftar publik tidak memuat satu pun.
    let list = f.public("/v1/public/content/items?limit=100").await;
    assert_eq!(list.status, 200);
    for id in [&draft, &future, &ended, &archived, &deleted, &app_only] {
        assert!(!list.body.contains(id.as_str()), "{id}");
    }
    // ID draf sebagai cursor = daftar kosong yang sama dengan cursor basi.
    let by_draft = f
        .public(&format!("/v1/public/content/items?cursor={draft}"))
        .await;
    assert_eq!(by_draft.json()["data"], json!([]));
    let by_random = f
        .public(&format!("/v1/public/content/items?cursor={random}"))
        .await;
    assert_eq!(by_random.json()["data"], json!([]));
    // Setelah waktu mulai tiba, konten terjadwal tampil (US-1101 AC2).
    f.clock.set(T0 + h);
    assert_eq!(
        f.public("/v1/public/content/pages/services").await.status,
        200
    );
}

// trace: US-1802 AC1, US-1802 AC2
#[tokio::test(flavor = "multi_thread")]
async fn site_list_follows_channel_page_and_new_fixed_page_version_replaces_old() {
    let f = Fixture::new(ROOMY).await;
    let h = time::Duration::hours(1);
    let a1 = f
        .draft(json!({"content_type": "article", "title": "Artikel 1", "channel": "app_and_site", "site_page": "download_app"}))
        .await;
    f.schedule(&a1, json!({})).await;
    f.clock.set(T0 + time::Duration::minutes(1));
    let a2 = f
        .draft(json!({"content_type": "article", "title": "Artikel 2", "channel": "site", "site_page": "download_app"}))
        .await;
    f.schedule(&a2, json!({})).await;
    let first = f
        .public("/v1/public/content/items?site_page=download_app&content_type=article&limit=1")
        .await
        .json();
    assert_eq!(first["data"][0]["content_id"], a2.as_str());
    assert_eq!(first["meta"]["has_more"], true);
    let cursor = first["meta"]["next_cursor"].as_str().unwrap();
    let second = f
        .public(&format!(
            "/v1/public/content/items?site_page=download_app&content_type=article&limit=1&cursor={cursor}"
        ))
        .await
        .json();
    assert_eq!(second["data"][0]["content_id"], a1.as_str());
    assert_eq!(second["meta"]["has_more"], false);

    let v1 = f.draft(page("privacy_terms", "Kebijakan v1", "site")).await;
    f.schedule(&v1, json!({})).await;
    let v2 = f
        .draft(page("privacy_terms", "Kebijakan v2", "app_and_site"))
        .await;
    f.schedule(&v2, json!({"publish_start_at": rfc(T0 + h)}))
        .await;
    let now = f
        .public("/v1/public/content/pages/privacy_terms")
        .await
        .json();
    assert_eq!(now["data"]["content_id"], v1.as_str());
    f.clock.set(T0 + h);
    let later = f
        .public("/v1/public/content/pages/privacy_terms")
        .await
        .json();
    assert_eq!(later["data"]["content_id"], v2.as_str());
    assert_eq!(
        f.public(&format!("/v1/public/content/items/{v1}"))
            .await
            .status,
        404
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn invalid_page_and_filters_are_rejected_without_details() {
    let f = Fixture::new(ROOMY).await;
    for p in [
        "/v1/public/content/pages/admin",
        "/v1/public/content/pages/HOME",
        "/v1/public/content/items/bukan-uuid",
    ] {
        let r = f.public(p).await;
        assert_eq!(r.status, 404, "{p}");
        assert_eq!(r.json()["response_code"], "4040001");
    }
    for q in [
        "site_page=lain",
        "content_type=html",
        "limit=x",
        "limit=0",
        "limit=101",
        "cursor=bukan-uuid",
        "tidak_dikenal=1",
    ] {
        let r = f.public(&format!("/v1/public/content/items?{q}")).await;
        assert_eq!(r.status, 422, "{q}: {}", r.body);
        assert_eq!(r.header("content-type"), Some("application/problem+json"));
    }
    // Endpoint backoffice tetap wajib login walau dipanggil dari jalur yang sama (default deny).
    assert_eq!(f.public("/backoffice/v1/contents").await.status, 401);
    assert_eq!(f.public("/v1/public/content/lain").await.status, 404);
}

// trace: ADR-012 butir 3 (batas laju per sumber dan global)
#[tokio::test(flavor = "multi_thread")]
async fn public_endpoints_are_rate_limited_per_source() {
    let f = Fixture::new(RateBudget {
        per_source: 3,
        global: 1_000,
    })
    .await;
    for _ in 0..3 {
        assert_eq!(f.public("/v1/public/content/pages/admin").await.status, 404);
    }
    let r = f.public("/v1/public/content/items").await;
    assert_eq!(r.status, 429);
    assert_eq!(r.json()["response_code"], "4290001");
    assert_eq!(r.header("content-type"), Some("application/problem+json"));
    // Backoffice memakai anggaran terpisah (M-47).
    let b = f
        .admin("GET", "/backoffice/v1/contents?limit=1", &json!(null))
        .await;
    assert_ne!(b.status, 429);
}
