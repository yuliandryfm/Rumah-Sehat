//! CMS backoffice (F1-T92; US-1101 AC1-AC4, US-1802 AC1-AC5; render aman F1-T111 US-1802 AC6)
//! lewat HTTP sungguhan terhadap PostgreSQL 18
//! (role runtime `rs_app`, `scripts/db-ephemeral.sh`): draf, pratinjau, jadwal, arsip, halaman tetap,
//! soft delete dan pulihkan dengan E-16, serta test penolakan per endpoint (coding-standards §7.3):
//! tanpa token, realm lain, akun nonaktif, sub-peran lain (403), objek tidak ada (404).
//! Token dari kunci realm uji (tanpa Keycloak); akun dan sub-peran dari database (resolver modul identity).
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)] // berkas test

mod audit_support;
mod common;

use common::{ADMIN_AUD, ADMIN_ISS, PASIEN_ISS, Resp, TestKeys, request, request_json, token};
use foundation::{FixedClock, SystemClock, Uuid, new_id};
use observability::EnvSource;
use rs_backend::app::{AppServices, AppState};
use serde_json::{Value, json};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use std::sync::Arc;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use time::macros::datetime;

const T0: OffsetDateTime = datetime!(2030-03-01 01:00:00 UTC);

async fn pool(var: &str) -> PgPool {
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

/// Akun admin sintetis dengan sub-peran; mengembalikan subjek Keycloak.
async fn admin(pool: &PgPool, status: &str, roles: &[&str]) -> String {
    let id = new_id();
    let sub = format!("uji-admin-{}", id.simple());
    sqlx::query!(
        "INSERT INTO app.account (id, account_type, idp_realm, idp_subject, email, status, status_changed_at, full_name) \
         VALUES ($1, 'rs_admin', 'admin', $2, $3, $4, now(), 'Admin Sintetis')",
        id,
        sub,
        format!("{sub}@example.com"),
        status,
    )
    .execute(pool)
    .await
    .unwrap();
    for r in roles {
        sqlx::query!(
            "INSERT INTO app.rs_admin_role (id, account_id, sub_role) VALUES ($1, $2, $3)",
            new_id(),
            id,
            *r,
        )
        .execute(pool)
        .await
        .unwrap();
    }
    sub
}

struct Fixture {
    addr: SocketAddr,
    keys: TestKeys,
    clock: Arc<FixedClock>,
    pool: PgPool,
    content: String,
}

impl Fixture {
    async fn new() -> Self {
        let pool = pool("RS_TEST_DB_APP_URL").await;
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
                public_content_limit: common::roomy_limiter(),
                public_image_limit: common::roomy_limiter(),
                corporate_hr: common::corporate_hr(common::unreachable_pool()),
                consent_text_limit: common::roomy_limiter(),
            },
        );
        // Masa berlaku token dinilai dengan jam sistem; aturan tayang dengan jam aplikasi yang diatur test.
        let auth = common::auth_state(&keys, Arc::new(SystemClock));
        let directory = common::account_directory(pool.clone());
        let addr = common::spawn_app(state, auth, directory.clone(), directory).await;
        let content = admin(&pool, "active", &["content"]).await;
        Self {
            addr,
            keys,
            clock,
            pool,
            content,
        }
    }

    fn bearer(&self, sub: &str) -> String {
        format!(
            "Authorization: Bearer {}",
            token(&self.keys.admin, ADMIN_ISS, ADMIN_AUD, sub)
        )
    }

    async fn get(&self, path: &str) -> Resp {
        request(self.addr, "GET", path, &[&self.bearer(&self.content)]).await
    }

    async fn send(&self, method: &str, path: &str, body: &Value) -> Resp {
        request_json(
            self.addr,
            method,
            path,
            &[&self.bearer(&self.content)],
            body,
        )
        .await
    }

    async fn draft(&self, body: Value) -> String {
        let r = self.send("POST", "/backoffice/v1/contents", &body).await;
        assert_eq!(r.status, 201, "{}", r.body);
        r.json()["data"]["content_id"].as_str().unwrap().to_string()
    }
}

fn article(channel: &str, page: Option<&str>) -> Value {
    json!({"content_type": "article", "title": "Tips sehat sintetis", "body": "## Judul\n\nIsi **tebal**.",
           "category": "edukasi", "channel": channel, "site_page": page})
}

fn fixed_page(page: &str, title: &str) -> Value {
    json!({"content_type": "info_page", "title": title, "body": "Isi halaman", "channel": "site", "site_page": page})
}

fn rfc(t: OffsetDateTime) -> String {
    t.format(&Rfc3339).unwrap()
}

// trace: US-1101 AC1
#[tokio::test(flavor = "multi_thread")]
async fn new_content_is_draft_and_can_be_previewed() {
    let f = Fixture::new().await;
    let r = f
        .send(
            "POST",
            "/backoffice/v1/contents",
            &article("app_and_site", Some("articles")),
        )
        .await;
    assert_eq!(r.status, 201, "{}", r.body);
    let j = r.json();
    assert_eq!(j["response_code"], "2010000");
    assert_eq!(j["data"]["editorial_status"], "draft");
    assert_eq!(j["data"]["display_status"], "draft");
    let id = j["data"]["content_id"].as_str().unwrap();
    let p = f.get(&format!("/backoffice/v1/contents/{id}")).await;
    assert_eq!(p.status, 200);
    assert_eq!(p.json()["data"]["body"], "## Judul\n\nIsi **tebal**.");
    assert_eq!(p.json()["data"]["site_page"], "articles");
    // Ubah draf.
    let mut changed = article("site", Some("articles"));
    changed["title"] = json!("Judul baru");
    let u = f
        .send("PUT", &format!("/backoffice/v1/contents/{id}"), &changed)
        .await;
    assert_eq!(u.status, 200, "{}", u.body);
    assert_eq!(u.json()["data"]["title"], "Judul baru");
    assert_eq!(u.json()["data"]["channel"], "site");
}

// trace: US-1802 AC1
#[tokio::test(flavor = "multi_thread")]
async fn channel_and_site_page_are_validated_per_field() {
    let f = Fixture::new().await;
    let r = f
        .send("POST", "/backoffice/v1/contents", &article("site", None))
        .await;
    assert_eq!(r.status, 422);
    assert_eq!(r.json()["errors"][0]["field"], "site_page");
    let r = f
        .send(
            "POST",
            "/backoffice/v1/contents",
            &article("app", Some("home")),
        )
        .await;
    assert_eq!(r.status, 422);
    let mut bad = article("app", None);
    bad["title"] = json!("   ");
    let r = f.send("POST", "/backoffice/v1/contents", &bad).await;
    assert_eq!(r.json()["errors"][0]["field"], "title");
    // Field tidak dikenal, enum salah, JSON rusak, dan bukan JSON.
    let mut extra = article("app", None);
    extra["harga_idr"] = json!(1000);
    assert_eq!(
        f.send("POST", "/backoffice/v1/contents", &extra)
            .await
            .status,
        422
    );
    let mut wrong = article("app", None);
    wrong["channel"] = json!("web");
    assert_eq!(
        f.send("POST", "/backoffice/v1/contents", &wrong)
            .await
            .status,
        422
    );
    let broken = common::request_with_body(
        f.addr,
        "POST",
        "/backoffice/v1/contents",
        &[&f.bearer(&f.content), "Content-Type: application/json"],
        Some(b"{\"title\":"),
    )
    .await;
    assert_eq!(broken.status, 400);
    assert_eq!(broken.json()["response_code"], "4000001");
    let plain = common::request_with_body(
        f.addr,
        "POST",
        "/backoffice/v1/contents",
        &[&f.bearer(&f.content), "Content-Type: text/plain"],
        Some(b"x"),
    )
    .await;
    assert_eq!(plain.status, 415);
}

// trace: US-1101 AC2
#[tokio::test(flavor = "multi_thread")]
async fn scheduled_content_goes_live_ends_and_archives() {
    let f = Fixture::new().await;
    let id = f.draft(article("app", None)).await;
    let h = time::Duration::hours(1);
    let r = f
        .send(
            "POST",
            &format!("/backoffice/v1/contents/{id}/schedule"),
            &json!({"publish_start_at": rfc(T0 + h), "publish_end_at": "2030-03-01T12:00:00+09:00"}),
        )
        .await;
    assert_eq!(r.status, 200, "{}", r.body);
    assert_eq!(r.json()["data"]["display_status"], "scheduled");
    let path = format!("/backoffice/v1/contents/{id}");
    f.clock.set(T0 + h);
    assert_eq!(f.get(&path).await.json()["data"]["display_status"], "live");
    // 12.00 WIT = 03.00 UTC (ADR-017): selesai tepat pada batas akhir.
    f.clock.set(datetime!(2030-03-01 03:00:00 UTC));
    assert_eq!(f.get(&path).await.json()["data"]["display_status"], "ended");
    // Konten terjadwal tidak dapat diubah atau dijadwalkan ulang (versi baru).
    assert_eq!(
        f.send("PUT", &path, &article("app", None)).await.status,
        409
    );
    let again = f
        .send("POST", &format!("{path}/schedule"), &json!({}))
        .await;
    assert_eq!(again.status, 409);
    assert_eq!(again.json()["response_code"], "4090001");
    let a = f.send("POST", &format!("{path}/archive"), &json!({})).await;
    assert_eq!(a.status, 200, "{}", a.body);
    assert_eq!(a.json()["data"]["display_status"], "archived");
    assert_eq!(
        f.send("POST", &format!("{path}/archive"), &json!({}))
            .await
            .status,
        409
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn schedule_rejects_past_start_offsetless_time_and_inverted_window() {
    let f = Fixture::new().await;
    let id = f.draft(article("app", None)).await;
    let path = format!("/backoffice/v1/contents/{id}/schedule");
    let past = f
        .send(
            "POST",
            &path,
            &json!({"publish_start_at": rfc(T0 - time::Duration::minutes(1))}),
        )
        .await;
    assert_eq!(past.status, 422);
    assert_eq!(past.json()["errors"][0]["code"], "in_past");
    let no_offset = f
        .send(
            "POST",
            &path,
            &json!({"publish_start_at": "2030-03-02T08:00:00"}),
        )
        .await;
    assert_eq!(no_offset.status, 422);
    assert_eq!(no_offset.json()["errors"][0]["field"], "publish_start_at");
    let inverted = f
        .send(
            "POST",
            &path,
            &json!({"publish_start_at": rfc(T0 + time::Duration::hours(2)), "publish_end_at": rfc(T0 + time::Duration::hours(1))}),
        )
        .await;
    assert_eq!(inverted.status, 422);
    assert_eq!(inverted.json()["errors"][0]["field"], "publish_end_at");
    // Tanpa waktu mulai = tayang sekarang (jam server).
    let now = f.send("POST", &path, &json!({})).await;
    assert_eq!(now.status, 200, "{}", now.body);
    assert_eq!(now.json()["data"]["display_status"], "live");
    assert_eq!(now.json()["data"]["publish_start_at"], rfc(T0));
}

// trace: US-1802 AC2
#[tokio::test(flavor = "multi_thread")]
async fn fixed_site_page_has_one_live_version() {
    let f = Fixture::new().await;
    let page = "for_business";
    let v1 = f.draft(fixed_page(page, "Untuk Perusahaan v1")).await;
    let r = f
        .send(
            "POST",
            &format!("/backoffice/v1/contents/{v1}/schedule"),
            &json!({}),
        )
        .await;
    assert_eq!(r.status, 200, "{}", r.body);
    let h = time::Duration::hours(1);
    let v2 = f.draft(fixed_page(page, "Untuk Perusahaan v2")).await;
    let r = f
        .send(
            "POST",
            &format!("/backoffice/v1/contents/{v2}/schedule"),
            &json!({"publish_start_at": rfc(T0 + h)}),
        )
        .await;
    assert_eq!(r.status, 200, "{}", r.body);
    // Versi lama berakhir saat versi baru mulai.
    let old = f.get(&format!("/backoffice/v1/contents/{v1}")).await.json();
    assert_eq!(old["data"]["publish_end_at"], rfc(T0 + h));
    assert_eq!(old["data"]["display_status"], "live");
    f.clock.set(T0 + h);
    let live = f
        .get(&format!(
            "/backoffice/v1/contents?status=live&site_page={page}"
        ))
        .await
        .json();
    let ids: Vec<&str> = live["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["content_id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, [v2.as_str()]);
    // Versi lain yang mulai sebelum versi terjadwal berikutnya tetapi tumpang tindih dengannya: konflik.
    let v3 = f.draft(fixed_page(page, "Untuk Perusahaan v3")).await;
    f.clock.set(T0);
    let v4 = f.draft(fixed_page("contact", "Kontak")).await;
    let r = f
        .send(
            "POST",
            &format!("/backoffice/v1/contents/{v3}/schedule"),
            &json!({"publish_start_at": rfc(T0 + time::Duration::minutes(30))}),
        )
        .await;
    assert_eq!(r.status, 409, "{}", r.body);
    // Halaman lain tidak terpengaruh.
    let r = f
        .send(
            "POST",
            &format!("/backoffice/v1/contents/{v4}/schedule"),
            &json!({}),
        )
        .await;
    assert_eq!(r.status, 200);
}

// trace: US-1802 AC5
#[tokio::test(flavor = "multi_thread")]
async fn image_without_alt_text_cannot_be_scheduled() {
    let f = Fixture::new().await;
    let id = f
        .draft(json!({"content_type": "banner", "title": "Banner", "channel": "app"}))
        .await;
    let file = new_id();
    sqlx::query!(
        "INSERT INTO app.stored_file (id, bucket, object_key, category, mime_type, size_bytes, sha256, validated_at) \
         VALUES ($1, 'public_content', $2, 'cms_image', 'image/png', 10, $3, now())",
        file,
        format!("uji-{}", file.simple()),
        &[7u8; 32][..],
    )
    .execute(&f.pool)
    .await
    .unwrap();
    let content_id: Uuid = id.parse().unwrap();
    sqlx::query!(
        "UPDATE app.cms_content SET image_file_id = $2 WHERE id = $1",
        content_id,
        file
    )
    .execute(&f.pool)
    .await
    .unwrap();
    let path = format!("/backoffice/v1/contents/{id}");
    let r = f
        .send("POST", &format!("{path}/schedule"), &json!({}))
        .await;
    assert_eq!(r.status, 422);
    assert_eq!(r.json()["errors"][0]["field"], "image_alt_text");
    let mut with_alt = json!({"content_type": "banner", "title": "Banner", "channel": "app"});
    with_alt["image_alt_text"] = json!("Dokter memeriksa pasien (ilustrasi)");
    assert_eq!(f.send("PUT", &path, &with_alt).await.status, 200);
    let r = f
        .send("POST", &format!("{path}/schedule"), &json!({}))
        .await;
    assert_eq!(r.status, 200, "{}", r.body);
}

// trace: US-1101 AC3
#[tokio::test(flavor = "multi_thread")]
async fn soft_delete_requires_reason_hides_content_and_restores() {
    let f = Fixture::new().await;
    let id = f.draft(article("site", Some("articles"))).await;
    let path = format!("/backoffice/v1/contents/{id}");
    let r = f
        .send(
            "POST",
            &format!("{path}/delete"),
            &json!({"delete_reason": "  "}),
        )
        .await;
    assert_eq!(r.status, 422);
    assert_eq!(r.json()["errors"][0]["field"], "delete_reason");
    let r = f
        .send(
            "POST",
            &format!("{path}/delete"),
            &json!({"delete_reason": "Konten usang"}),
        )
        .await;
    assert_eq!(r.status, 200, "{}", r.body);
    assert_eq!(r.json()["data"]["display_status"], "deleted");
    assert_eq!(f.get(&path).await.status, 404);
    let again = f
        .send(
            "POST",
            &format!("{path}/delete"),
            &json!({"delete_reason": "x"}),
        )
        .await;
    assert_eq!(again.status, 404);
    let deleted = f
        .get("/backoffice/v1/contents?status=deleted&limit=100")
        .await
        .json();
    let row = deleted["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["content_id"] == id.as_str())
        .cloned()
        .unwrap();
    assert_eq!(row["delete_reason"], "Konten usang");
    assert_eq!(row["display_status"], "deleted");
    // Daftar bawaan tidak memuat konten terhapus (ADR-003).
    let default = f.get("/backoffice/v1/contents?limit=100").await.json();
    assert!(
        !default["data"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c["content_id"] == id.as_str())
    );
    let r = f.send("POST", &format!("{path}/restore"), &json!({})).await;
    assert_eq!(r.status, 200, "{}", r.body);
    assert_eq!(r.json()["data"]["display_status"], "draft");
    assert_eq!(f.get(&path).await.status, 200);
    assert_eq!(
        f.send("POST", &format!("{path}/restore"), &json!({}))
            .await
            .status,
        404
    );

    // Event E-16 ditulis di transaksi yang sama oleh audit::in_audited_tx (gagal tulis = 500 dan rollback,
    // diuji crate audit); isi event diuji unit test service (`deletion_and_restore_events_*`).
    // F1-T69 (US-1501 AC1–AC2): event yang tersimpan di outbox lengkap per PRD §7.2 dan tanpa alasan
    // teks bebas (hanya nama field `delete_reason`).
    let relay = pool("RS_TEST_DB_RELAY_URL").await;
    let all: Vec<Value> =
        sqlx::query_scalar!("SELECT payload FROM app.audit_outbox ORDER BY occurred_at, id")
            .fetch_all(&relay)
            .await
            .unwrap();
    let mine: Vec<Value> = all
        .into_iter()
        .filter(|p| p["object_id"].as_str() == Some(id.as_str()))
        .collect();
    let actions: Vec<_> = mine.iter().map(|p| p["action_name"].clone()).collect();
    assert_eq!(
        actions,
        [json!("cms_content_deleted"), json!("cms_content_restored")]
    );
    audit_support::assert_events_complete(&mine, &["e_16"]);
    assert!(
        !serde_json::to_string(&mine)
            .unwrap()
            .contains("Konten usang")
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn list_paginates_with_cursor_and_filters() {
    let f = Fixture::new().await;
    let page = "partners";
    let mut ids = Vec::new();
    for _ in 0..3 {
        ids.push(f.draft(article("site", Some(page))).await);
    }
    let first = f
        .get(&format!(
            "/backoffice/v1/contents?site_page={page}&status=draft&limit=2"
        ))
        .await
        .json();
    assert_eq!(first["meta"]["has_more"], true);
    let cursor = first["meta"]["next_cursor"].as_str().unwrap();
    let second = f
        .get(&format!(
            "/backoffice/v1/contents?site_page={page}&status=draft&limit=2&cursor={cursor}"
        ))
        .await
        .json();
    let mut seen: Vec<String> = first["data"]
        .as_array()
        .unwrap()
        .iter()
        .chain(second["data"].as_array().unwrap())
        .map(|c| c["content_id"].as_str().unwrap().to_string())
        .collect();
    seen.sort();
    ids.sort();
    // Halaman lain dari test paralel tidak ikut karena saringan halaman; urutan terbaru dulu.
    assert!(ids.iter().all(|i| seen.contains(i)), "{seen:?}");
    let bad = f.get("/backoffice/v1/contents?status=published").await;
    assert_eq!(bad.status, 422);
    let bad = f.get("/backoffice/v1/contents?cursor=bukan-uuid").await;
    assert_eq!(bad.status, 422);
}

// trace: US-1802 AC4
#[tokio::test(flavor = "multi_thread")]
async fn every_endpoint_rejects_missing_token_other_realm_inactive_and_other_sub_roles() {
    let f = Fixture::new().await;
    let id = f.draft(article("app", None)).await;
    let verifier = admin(&f.pool, "active", &["verifier"]).await;
    let super_admin = admin(&f.pool, "active", &["super_admin", "auditor"]).await;
    let inactive = admin(&f.pool, "inactive", &["content"]).await;
    let pasien = format!(
        "Authorization: Bearer {}",
        token(&f.keys.pasien, PASIEN_ISS, "rs-api-pasien", "uji-pasien")
    );
    let wrong_aud = format!(
        "Authorization: Bearer {}",
        token(&f.keys.admin, ADMIN_ISS, "rs-portal-api", &f.content)
    );
    let body = article("app", None);
    let endpoints: Vec<(&str, String, Value)> = vec![
        ("GET", "/backoffice/v1/contents".into(), Value::Null),
        ("POST", "/backoffice/v1/contents".into(), body.clone()),
        ("GET", format!("/backoffice/v1/contents/{id}"), Value::Null),
        ("PUT", format!("/backoffice/v1/contents/{id}"), body.clone()),
        (
            "POST",
            format!("/backoffice/v1/contents/{id}/schedule"),
            json!({}),
        ),
        (
            "POST",
            format!("/backoffice/v1/contents/{id}/archive"),
            json!({}),
        ),
        (
            "POST",
            format!("/backoffice/v1/contents/{id}/delete"),
            json!({"delete_reason": "x"}),
        ),
        (
            "POST",
            format!("/backoffice/v1/contents/{id}/restore"),
            json!({}),
        ),
    ];
    let call = |m: &'static str, p: String, b: Value, auth: Option<String>| {
        let addr = f.addr;
        async move {
            let headers: Vec<String> = auth.into_iter().collect();
            let refs: Vec<&str> = headers.iter().map(String::as_str).collect();
            if b.is_null() {
                request(addr, m, &p, &refs).await
            } else {
                request_json(addr, m, &p, &refs, &b).await
            }
        }
    };
    for (m, p, b) in &endpoints {
        let cases: [(Option<String>, u16, &str); 6] = [
            (None, 401, "tanpa token"),
            (Some(pasien.clone()), 401, "realm pasien"),
            (Some(wrong_aud.clone()), 401, "audience lain"),
            (Some(f.bearer(&inactive)), 401, "akun nonaktif"),
            (Some(f.bearer(&verifier)), 403, "Verifikator"),
            (
                Some(f.bearer(&super_admin)),
                403,
                "Super Admin tanpa Konten",
            ),
        ];
        for (auth, status, label) in cases {
            let r = call(m, p.clone(), b.clone(), auth).await;
            assert_eq!(r.status, status, "{m} {p} {label}: {}", r.body);
            assert_eq!(
                r.header("content-type"),
                Some("application/problem+json"),
                "{m} {p} {label}"
            );
            assert!(r.json()["correlation_id"].is_string());
        }
    }
    // Ditolak = tidak ada perubahan: konten masih draf, tidak terhapus.
    let still = f.get(&format!("/backoffice/v1/contents/{id}")).await.json();
    assert_eq!(still["data"]["display_status"], "draft");
    // Objek tidak ada atau ID bukan UUID = 404 yang sama.
    let missing = Uuid::from_u128(0x0193_6f8e_9c01_7e02_8f03_1a2b_3c4d_5eff);
    for p in [
        format!("/backoffice/v1/contents/{missing}"),
        "/backoffice/v1/contents/bukan-uuid".to_string(),
    ] {
        let r = f.get(&p).await;
        assert_eq!(r.status, 404, "{p}");
        assert_eq!(r.json()["response_code"], "4040001");
        let r = f.send("POST", &format!("{p}/archive"), &json!({})).await;
        assert_eq!(r.status, 404, "{p}");
    }
}

// trace: US-1802 AC6, US-1802 AC7
#[tokio::test(flavor = "multi_thread")]
async fn xss_payloads_are_rejected_on_save_and_never_stored() {
    let f = Fixture::new().await;
    let id = f.draft(article("site", Some("articles"))).await;
    let path = format!("/backoffice/v1/contents/{id}");
    let image = "/v1/public/content/images/0193a1b2-c3d4-7e5f-8a6b-7c8d9e0f1a2b";
    let payloads: [(&str, &str); 12] = [
        ("<script>alert(1)</script>", "raw_html_not_allowed"),
        (
            "Teks <img src=x onerror=alert(1)> teks",
            "raw_html_not_allowed",
        ),
        (
            "<iframe src=\"https://example.com\"></iframe>",
            "raw_html_not_allowed",
        ),
        (
            "<a href=\"javascript:alert(1)\">x</a>",
            "raw_html_not_allowed",
        ),
        ("<svg onload=alert(1)>", "raw_html_not_allowed"),
        ("[klik](javascript:alert(1))", "link_scheme_not_allowed"),
        (
            "[klik](jav&#x61;script:alert(1))",
            "link_scheme_not_allowed",
        ),
        (
            "[klik](data:text/html;base64,PHNjcmlwdD4=)",
            "link_scheme_not_allowed",
        ),
        ("<javascript:alert(1)>", "link_scheme_not_allowed"),
        ("[x](//example.com/jalur)", "link_scheme_not_allowed"),
        (
            "![foto](https://img.example.test/x.png)",
            "image_source_not_allowed",
        ),
        (
            "![](/v1/public/content/images/0193a1b2-c3d4-7e5f-8a6b-7c8d9e0f1a2b)",
            "image_alt_missing",
        ),
    ];
    for (payload, code) in payloads {
        let mut body = article("site", Some("articles"));
        body["body"] = json!(format!("## Judul\n\n{payload}\n"));
        for (method, target) in [("POST", "/backoffice/v1/contents"), ("PUT", path.as_str())] {
            let r = f.send(method, target, &body).await;
            assert_eq!(r.status, 422, "{method} {payload}: {}", r.body);
            let e = &r.json()["errors"][0];
            assert_eq!(e["field"], "body", "{payload}");
            assert_eq!(e["code"], code, "{payload}");
            assert!(
                e["message"].as_str().unwrap().ends_with("(baris 3)"),
                "{payload}"
            );
            // Galat tidak menggemakan isi berbahaya.
            assert!(!r.body.contains("alert(1)"), "{payload}");
        }
    }
    // Isi draf tetap yang lama (tidak ada yang tersimpan).
    let stored = f.get(&path).await.json();
    assert_eq!(stored["data"]["body"], "## Judul\n\nIsi **tebal**.");
    // Markdown aman (tautan https/mailto/relatif, gambar dari endpoint publik, HTML di blok kode) diterima.
    let mut safe = article("site", Some("articles"));
    safe["body"] = json!(format!(
        "## Aman\n\n[Layanan](/layanan) [Surel](mailto:halo@example.com) [Mitra](https://example.com)\n\n![Foto klinik]({image})\n\n```\n<script>sebagai teks kode</script>\n```\n"
    ));
    let r = f.send("PUT", &path, &safe).await;
    assert_eq!(r.status, 200, "{}", r.body);
}
