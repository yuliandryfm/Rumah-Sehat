//! Gambar konten CMS (F1-T93; US-1802 AC5, ADR-018, ADR-027 butir 5) terhadap PostgreSQL 18 (role `rs_app`)
//! dan SeaweedFS efemeral (KEPUTUSAN-233): slot unggah lewat API backoffice, validasi isi dan buang metadata
//! (modul platform F1-T29), teks alternatif wajib, gambar publik hanya untuk konten yang sedang tayang di
//! situs dan disajikan backend (bukan bucket publik), batas laju, serta skenario negatif berkas.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)] // berkas test

mod common;
mod files;

use common::{ADMIN_AUD, ADMIN_ISS, TestKeys, request, request_json, request_raw, token};
use files::{METADATA_MARKER, SeaweedFs, app_pool, contains, png, png_with_metadata, upload};
use foundation::{Clock, SystemClock, Uuid, new_id};
use rs_backend::app::{AppServices, AppState};
use rs_backend::modules::platform::api::{
    FileStorage, LogicalBucket, UploadInstruction, UploadMethod,
};
use rs_backend::rate_limit::{RateBudget, RateLimiter};
use serde_json::{Value, json};
use sqlx::PgPool;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

const ROOMY: RateBudget = RateBudget {
    per_source: 10_000,
    global: 10_000,
};

async fn content_admin(pool: &PgPool) -> String {
    let id = new_id();
    let sub = format!("uji-gambar-{}", id.simple());
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
    _sw: SeaweedFs,
    addr: SocketAddr,
    keys: TestKeys,
    pool: PgPool,
    admin: String,
}

async fn storage_with_public_bucket(sw: &SeaweedFs) -> FileStorage {
    let storage = sw
        .file_storage(UploadMethod::Post, Duration::from_secs(600))
        .await;
    let bucket = storage.bucket_name(LogicalBucket::PublicContent);
    let now = SystemClock.now();
    storage
        .object_store()
        .create_bucket(&bucket, now)
        .await
        .unwrap();
    storage
        .object_store()
        .enable_versioning(&bucket, now)
        .await
        .unwrap();
    storage
}

impl Fixture {
    async fn new(image_budget: RateBudget) -> Self {
        let sw = SeaweedFs::start().await;
        let storage = storage_with_public_bucket(&sw).await;
        let pool = app_pool().await;
        let keys = TestKeys::generate();
        // Jam sistem: object storage memeriksa waktu tanda tangan SigV4 (jam tiruan jauh di depan ditolak
        // penyedia). Isolasi dari berkas test lain lewat ID konten/gambar, bukan lewat jam.
        let clock: Arc<dyn Clock> = Arc::new(SystemClock);
        let state = AppState::new(
            common::TYPE_BASE,
            Vec::new(),
            AppServices {
                pool: pool.clone(),
                clock,
                identity: common::unused_identity_services(
                    pool.clone(),
                    Arc::new(common::unused_file_storage()),
                ),
                files: Arc::new(storage),
                public_content_limit: Arc::new(RateLimiter::new(ROOMY)),
                public_image_limit: Arc::new(RateLimiter::new(image_budget)),
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
        let admin = content_admin(&pool).await;
        Self {
            _sw: sw,
            addr,
            keys,
            pool,
            admin,
        }
    }

    fn bearer(&self, sub: &str) -> String {
        format!(
            "Authorization: Bearer {}",
            token(&self.keys.admin, ADMIN_ISS, ADMIN_AUD, sub)
        )
    }

    async fn send_as(&self, sub: &str, method: &str, path: &str, body: &Value) -> common::Resp {
        request_json(self.addr, method, path, &[&self.bearer(sub)], body).await
    }

    async fn send(&self, method: &str, path: &str, body: &Value) -> common::Resp {
        self.send_as(&self.admin, method, path, body).await
    }

    async fn draft(&self) -> String {
        let r = self
            .send(
                "POST",
                "/backoffice/v1/contents",
                &json!({"content_type": "banner", "title": "Banner bergambar", "channel": "site", "site_page": "home"}),
            )
            .await;
        assert_eq!(r.status, 201, "{}", r.body);
        r.json()["data"]["content_id"].as_str().unwrap().to_string()
    }

    /// Slot + unggah lewat instruksi API; mengembalikan ID slot.
    async fn slot_with_upload(&self, sub: &str, declared: &str, bytes: &[u8]) -> String {
        let r = self
            .send_as(
                sub,
                "POST",
                "/backoffice/v1/content-images/upload-slots",
                &json!({"content_type": declared, "size_bytes": bytes.len()}),
            )
            .await;
        assert_eq!(r.status, 201, "{}", r.body);
        let d = &r.json()["data"];
        assert_eq!(d["method"], "post");
        let instruction = UploadInstruction::Post {
            url: d["url"].as_str().unwrap().to_string(),
            fields: d["fields"]
                .as_array()
                .unwrap()
                .iter()
                .map(|f| {
                    (
                        f["name"].as_str().unwrap().to_string(),
                        f["value"].as_str().unwrap().to_string(),
                    )
                })
                .collect(),
        };
        let code = upload(&instruction, bytes);
        assert!((200..300).contains(&code), "unggah {code}");
        d["upload_slot_id"].as_str().unwrap().to_string()
    }
}

// trace: US-1802 AC5, ADR-027 butir 5
#[tokio::test(flavor = "multi_thread")]
async fn image_is_public_only_while_content_is_live_and_served_without_metadata() {
    let f = Fixture::new(ROOMY).await;
    let id = f.draft().await;
    let original = png_with_metadata(64, 48);
    let slot = f
        .slot_with_upload(&f.admin.clone(), "image/png", &original)
        .await;
    let r = f
        .send(
            "PUT",
            &format!("/backoffice/v1/contents/{id}/image"),
            &json!({"upload_slot_id": slot, "image_alt_text": "  Dokter memeriksa pasien (ilustrasi)  "}),
        )
        .await;
    assert_eq!(r.status, 200, "{}", r.body);
    let d = r.json()["data"].clone();
    assert_eq!(d["image_alt_text"], "Dokter memeriksa pasien (ilustrasi)");
    let file_id = d["image_file_id"].as_str().unwrap().to_string();
    let image_path = format!("/v1/public/content/images/{file_id}");

    // Draf: gambar tidak publik (sama dengan ID acak).
    let random = Uuid::from_u128(0x0193_6f8e_9c01_7e02_8f03_1a2b_3c4d_5e99);
    let baseline = request(
        f.addr,
        "GET",
        &format!("/v1/public/content/images/{random}"),
        &[],
    )
    .await;
    assert_eq!(baseline.status, 404);
    let draft_view = request(f.addr, "GET", &image_path, &[]).await;
    assert_eq!(draft_view.status, 404);
    assert_eq!(draft_view.json()["response_code"], "4040001");

    let s = f
        .send(
            "POST",
            &format!("/backoffice/v1/contents/{id}/schedule"),
            &json!({}),
        )
        .await;
    assert_eq!(s.status, 200, "{}", s.body);
    let (status, headers, bytes) = request_raw(f.addr, "GET", &image_path, &[]).await;
    assert_eq!(status, 200);
    let header = |n: &str| {
        headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(n))
            .map(|(_, v)| v.as_str())
    };
    assert_eq!(header("content-type"), Some("image/png"));
    assert_eq!(header("x-content-type-options"), Some("nosniff"));
    assert_eq!(header("cache-control"), Some("no-store"));
    assert_eq!(header("set-cookie"), None);
    assert!(bytes.starts_with(&[0x89, b'P', b'N', b'G']));
    assert!(
        !contains(&bytes, METADATA_MARKER),
        "metadata tidak terbuang"
    );

    // Konten publik menunjuk endpoint gambar backend dengan teks alternatif.
    let item = request(
        f.addr,
        "GET",
        &format!("/v1/public/content/items/{id}"),
        &[],
    )
    .await;
    assert_eq!(item.json()["data"]["image"]["url"], image_path.as_str());
    assert_eq!(
        item.json()["data"]["image"]["alt_text"],
        "Dokter memeriksa pasien (ilustrasi)"
    );

    // Konten diturunkan: gambar tidak lagi publik.
    let a = f
        .send(
            "POST",
            &format!("/backoffice/v1/contents/{id}/archive"),
            &json!({}),
        )
        .await;
    assert_eq!(a.status, 200);
    assert_eq!(request(f.addr, "GET", &image_path, &[]).await.status, 404);
}

#[tokio::test(flavor = "multi_thread")]
async fn image_upload_rejects_fake_type_size_wrong_owner_missing_alt_and_non_draft() {
    let f = Fixture::new(ROOMY).await;
    let id = f.draft().await;
    let path = format!("/backoffice/v1/contents/{id}/image");
    // Jenis yang dinyatakan bukan JPG/PNG, dan ukuran > 5 MiB: ditolak sebelum slot terbit.
    let r = f
        .send(
            "POST",
            "/backoffice/v1/content-images/upload-slots",
            &json!({"content_type": "image/gif", "size_bytes": 10}),
        )
        .await;
    assert_eq!(r.status, 415);
    let r = f
        .send(
            "POST",
            "/backoffice/v1/content-images/upload-slots",
            &json!({"content_type": "image/png", "size_bytes": 5_242_881}),
        )
        .await;
    assert_eq!(r.status, 413);
    // Teks alternatif kosong: ditolak tanpa memakai slot.
    let good = png(32, 32);
    let slot = f
        .slot_with_upload(&f.admin.clone(), "image/png", &good)
        .await;
    let r = f
        .send(
            "PUT",
            &path,
            &json!({"upload_slot_id": slot, "image_alt_text": "   "}),
        )
        .await;
    assert_eq!(r.status, 422);
    assert_eq!(r.json()["errors"][0]["field"], "image_alt_text");
    // Slot milik admin lain = 404 (KU-03).
    let other = content_admin(&f.pool).await;
    let r = f
        .send_as(
            &other,
            "PUT",
            &path,
            &json!({"upload_slot_id": slot, "image_alt_text": "Foto"}),
        )
        .await;
    assert_eq!(r.status, 404);
    // Pemilik slot berhasil.
    let r = f
        .send(
            "PUT",
            &path,
            &json!({"upload_slot_id": slot, "image_alt_text": "Foto"}),
        )
        .await;
    assert_eq!(r.status, 200, "{}", r.body);
    // Slot yang sudah dipakai = 404.
    let r = f
        .send(
            "PUT",
            &path,
            &json!({"upload_slot_id": slot, "image_alt_text": "Foto"}),
        )
        .await;
    assert_eq!(r.status, 404);
    // Isi palsu (teks dinyatakan PNG) = 415.
    let fake = f
        .slot_with_upload(
            &f.admin.clone(),
            "image/png",
            b"bukan gambar, hanya teks sintetis",
        )
        .await;
    let r = f
        .send(
            "PUT",
            &path,
            &json!({"upload_slot_id": fake, "image_alt_text": "Foto"}),
        )
        .await;
    assert_eq!(r.status, 415);
    // Konten bukan draf = 409 tanpa memakai slot.
    let s = f
        .send(
            "POST",
            &format!("/backoffice/v1/contents/{id}/schedule"),
            &json!({}),
        )
        .await;
    assert_eq!(s.status, 200, "{}", s.body);
    let slot2 = f
        .slot_with_upload(&f.admin.clone(), "image/png", &good)
        .await;
    let r = f
        .send(
            "PUT",
            &path,
            &json!({"upload_slot_id": slot2, "image_alt_text": "Foto"}),
        )
        .await;
    assert_eq!(r.status, 409);
    // Slot bukan UUID = 404.
    let r = f
        .send(
            "PUT",
            &path,
            &json!({"upload_slot_id": "x", "image_alt_text": "Foto"}),
        )
        .await;
    assert_eq!(r.status, 404);
}

// trace: US-1802 AC4
#[tokio::test(flavor = "multi_thread")]
async fn image_endpoints_reject_missing_token_and_other_sub_roles() {
    let f = Fixture::new(ROOMY).await;
    let id = f.draft().await;
    let verifier = {
        let sub = content_admin(&f.pool).await;
        sqlx::query!(
            "UPDATE app.rs_admin_role SET sub_role = 'verifier' \
             WHERE account_id = (SELECT id FROM app.account WHERE idp_subject = $1)",
            sub
        )
        .execute(&f.pool)
        .await
        .unwrap();
        sub
    };
    for (m, p, b) in [
        (
            "POST",
            "/backoffice/v1/content-images/upload-slots".to_string(),
            json!({"content_type": "image/png", "size_bytes": 10}),
        ),
        (
            "PUT",
            format!("/backoffice/v1/contents/{id}/image"),
            json!({"upload_slot_id": Uuid::from_u128(1).to_string(), "image_alt_text": "x"}),
        ),
    ] {
        let r = request_json(f.addr, m, &p, &[], &b).await;
        assert_eq!(r.status, 401, "{m} {p}");
        let r = f.send_as(&verifier, m, &p, &b).await;
        assert_eq!(r.status, 403, "{m} {p}");
    }
}

// trace: ADR-027 butir 5 (batas laju gambar publik)
#[tokio::test(flavor = "multi_thread")]
async fn public_image_endpoint_is_rate_limited() {
    let f = Fixture::new(RateBudget {
        per_source: 2,
        global: 1_000,
    })
    .await;
    let random = Uuid::from_u128(0x0193_6f8e_9c01_7e02_8f03_1a2b_3c4d_5e98);
    let p = format!("/v1/public/content/images/{random}");
    assert_eq!(request(f.addr, "GET", &p, &[]).await.status, 404);
    assert_eq!(
        request(f.addr, "GET", "/v1/public/content/images/bukan-uuid", &[])
            .await
            .status,
        404
    );
    let r = request(f.addr, "GET", &p, &[]).await;
    assert_eq!(r.status, 429);
    assert_eq!(r.json()["response_code"], "4290001");
    // Anggaran konten publik terpisah dari anggaran gambar.
    let content = request(f.addr, "GET", "/v1/public/content/pages/contact", &[]).await;
    assert_ne!(content.status, 429);
}
