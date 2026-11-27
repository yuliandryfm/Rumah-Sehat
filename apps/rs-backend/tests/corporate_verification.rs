//! Endpoint verifikasi perusahaan `/backoffice/v1/corporate-verifications` (F1-T104, US-1902) lewat HTTP
//! sungguhan: router lengkap proses `api` (`build_router`: penegak kebijakan, validator token realm `admin`,
//! `pasien`, dan `korporat` dengan kunci realm uji, resolver akun dan penjaga sesi modul identity di atas
//! PostgreSQL, `ClientIp` dari peer TCP) dan PostgreSQL efemeral (`scripts/db-ephemeral.sh`, role runtime
//! `rs_api`). URL unduh ditandatangani lokal (tanpa object storage berjalan).
//!
//! Endpoint pendaftaran perusahaan (F1-T101) dan unggah dokumen (F1-T102) belum ada, sehingga perusahaan,
//! dokumen, dan pengajuan ulang dibuat helper test langsung di database (data sintetis, coding-standards
//! §7.2).
//!
//! Skenario: antrean terlama lebih dulu tanpa NPWP, detail, URL dokumen dengan E-05 lebih dulu (tenant =
//! perusahaan), setujui/tolak dengan E-10, alasan di luar daftar atau tidak aktif di `reason_code`, keputusan
//! pertama menang (bersamaan dan berurutan; pihak kalah E-10 hasil gagal), pengajuan ulang setelah ditolak
//! (putaran baru), NPWP tidak bocor ke payload audit maupun galat, dan penolakan akses: tanpa token / realm
//! lain (pasien, korporat/HR) → 401, sub-peran lain termasuk Super Admin → 403, akun nonaktif → 401, ID tidak
//! ada atau bukan UUID → 404. Tanpa variabel database, test GAGAL.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)] // berkas test

mod audit_support;
mod common;
mod files;
mod identity_support;
mod valkey_support;

use audit_support::assert_event_complete;
use authz::{AdminSubRole, DenialSink};
use common::{
    ADMIN_AUD, ADMIN_ISS, KORPORAT_AUD, KORPORAT_ISS, PASIEN_AUD, PASIEN_ISS, Resp, TestKeys,
    request, request_json,
};
use foundation::{SystemClock, Uuid, new_id};
use identity_support::new_admin;
use observability::EnvSource;
use rs_backend::access_audit::AuditDenialRecorder;
use rs_backend::app::{AppServices, AppState, build_router};
use serde_json::{Value, json};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

static SERIAL: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

const BASE: &str = "/backoffice/v1/corporate-verifications";
const REASONS: &str = "/backoffice/v1/corporate-verification-reasons";
const FOUR: [&str; 4] = [
    "nib_document_invalid",
    "npwp_document_invalid",
    "company_data_mismatch",
    "authority_proof_insufficient",
];

struct Fixture {
    addr: SocketAddr,
    keys: TestKeys,
    pool: PgPool,
    /// Pembaca `audit_outbox` (role relay; role aplikasi hanya INSERT).
    relay: PgPool,
}

async fn pool_from(var: &str, max: u32) -> PgPool {
    let url = EnvSource::from_process()
        .required_secret(var)
        .unwrap_or_else(|_| panic!("{var} tidak diset: jalankan lewat scripts/db-ephemeral.sh"));
    PgPoolOptions::new()
        .max_connections(max)
        .connect(url.expose())
        .await
        .unwrap()
}

impl Fixture {
    async fn new() -> Self {
        let pool = pool_from("RS_TEST_DB_APP_URL", 10).await;
        let keys = TestKeys::generate();
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
                consent_text_limit: common::roomy_limiter(),
                corporate_hr: common::corporate_hr(common::unreachable_pool()),
            },
        );
        let auth = common::auth_state(&keys, Arc::new(SystemClock));
        let directory = common::account_directory(pool.clone());
        let denials = DenialSink::start(
            64,
            2,
            Some(Arc::new(AuditDenialRecorder::new(
                pool.clone(),
                Arc::new(SystemClock),
            ))),
        )
        .unwrap();
        let app = build_router(state, auth, directory.clone(), directory, denials).unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            app.serve_with_peer_addr(listener, std::future::pending())
                .await
                .unwrap();
        });
        Self {
            addr,
            keys,
            pool,
            relay: pool_from("RS_TEST_DB_RELAY_URL", 2).await,
        }
    }

    fn admin_token(&self, subject: &str) -> String {
        common::token(&self.keys.admin, ADMIN_ISS, ADMIN_AUD, subject)
    }

    /// Akun Verifikator baru dan tokennya.
    async fn verifier(&self) -> (Uuid, String) {
        let (id, sub) = new_admin(&self.pool, &[AdminSubRole::Verifier]).await;
        (id, self.admin_token(&sub))
    }

    async fn get(&self, path: &str, token: &str) -> Resp {
        request(
            self.addr,
            "GET",
            path,
            &[&format!("Authorization: Bearer {token}")],
        )
        .await
    }

    async fn post(&self, path: &str, token: &str, body: &Value) -> Resp {
        request_json(
            self.addr,
            "POST",
            path,
            &[&format!("Authorization: Bearer {token}")],
            body,
        )
        .await
    }

    /// Token versi dokumen dari detail (keputusan terikat versi dokumen, F1-T102); perusahaan yang tidak ada
    /// mendapat token nol sehingga jawaban 404 tetap yang diuji.
    async fn version(&self, token: &str, id: Uuid) -> String {
        let r = self.get(&format!("{BASE}/{id}"), token).await;
        if r.status == 200 {
            r.json()["data"]["documents_version"]
                .as_str()
                .unwrap()
                .to_string()
        } else {
            "0".repeat(64)
        }
    }

    async fn approve(&self, token: &str, id: Uuid) -> Resp {
        let v = self.version(token, id).await;
        self.post(
            &format!("{BASE}/{id}/approve"),
            token,
            &json!({ "documents_version": v }),
        )
        .await
    }

    async fn reject(&self, token: &str, id: Uuid, reason: &str) -> Resp {
        let v = self.version(token, id).await;
        self.post(
            &format!("{BASE}/{id}/reject"),
            token,
            &json!({ "reason_code": reason, "documents_version": v }),
        )
        .await
    }
}

fn code(r: &Resp) -> String {
    r.json()["response_code"]
        .as_str()
        .unwrap_or_default()
        .to_string()
}

/// Angka acak `n` digit (NIB/NPWP sintetis berawalan `99`, bukan nomor nyata).
fn digits(n: usize) -> String {
    let mut b = vec![0u8; n.saturating_sub(2)];
    crypto::fill_random(&mut b).unwrap();
    let tail: String = b.iter().map(|x| char::from(b'0' + x % 10)).collect();
    format!("99{tail}")
}

/// Perusahaan sintetis.
struct Company {
    id: Uuid,
    npwp: String,
}

/// Perusahaan berstatus `status` dengan `submitted_at` = sekarang dikurangi `age_minutes` menit, beserta
/// dokumen aktif NIB, NPWP, dan bukti kewenangan (berkas kategori `corporate_document`, bucket
/// `license_document`).
async fn company(pool: &PgPool, status: &str, age_minutes: i32) -> Company {
    let id = new_id();
    let npwp = digits(16);
    sqlx::query!(
        "INSERT INTO app.corporate_client \
           (id, legal_name, nib, npwp, office_address, office_phone, status, origin, submitted_at) \
         VALUES ($1, 'PT Sintetis Uji', $2, $3, 'Jl. Sintetis No. 1', '0210000001', $4, 'self_registered', \
                 now() - make_interval(mins => $5))",
        id,
        digits(13),
        npwp,
        status,
        age_minutes,
    )
    .execute(pool)
    .await
    .unwrap();
    for kind in ["nib", "npwp", "authority_proof"] {
        let file = stored_file(pool, "corporate_document", "license_document").await;
        document(pool, id, kind, file).await;
    }
    Company { id, npwp }
}

async fn stored_file(pool: &PgPool, category: &str, bucket: &str) -> Uuid {
    let id = new_id();
    sqlx::query!(
        "INSERT INTO app.stored_file \
           (id, bucket, object_key, category, mime_type, size_bytes, sha256, validated_at) \
         VALUES ($1, $2, $3, $4, 'image/jpeg', 1024, '\\x00', now())",
        id,
        bucket,
        format!("uji/{}", files::random_hex(12)),
        category,
    )
    .execute(pool)
    .await
    .unwrap();
    id
}

async fn document(pool: &PgPool, client: Uuid, kind: &str, file: Uuid) {
    sqlx::query!(
        "INSERT INTO app.corporate_client_document (id, corporate_client_id, document_kind, stored_file_id) \
         VALUES ($1, $2, $3, $4)",
        new_id(),
        client,
        kind,
        file,
    )
    .execute(pool)
    .await
    .unwrap();
}

async fn file_of(pool: &PgPool, client: Uuid, kind: &str) -> Uuid {
    sqlx::query_scalar!(
        "SELECT stored_file_id FROM app.corporate_client_document \
         WHERE corporate_client_id = $1 AND document_kind = $2 AND deleted_at IS NULL",
        client,
        kind,
    )
    .fetch_one(pool)
    .await
    .unwrap()
}

/// Pengajuan ulang tiruan setelah ditolak (jalur HR F1-T101/F1-T103 belum ada): putaran naik, status kembali
/// "Menunggu verifikasi", waktu pengajuan diperbarui (DD A20 Rincian K1).
async fn resubmit(pool: &PgPool, client: Uuid) {
    let n = sqlx::query!(
        "UPDATE app.corporate_client \
         SET status = 'pending_verification', submission_round = submission_round + 1, \
             submitted_at = now(), updated_at = now() \
         WHERE id = $1 AND status = 'rejected'",
        client,
    )
    .execute(pool)
    .await
    .unwrap()
    .rows_affected();
    assert_eq!(n, 1);
}

async fn client_status(f: &Fixture, id: Uuid) -> (String, i32) {
    let r = sqlx::query!(
        "SELECT status, submission_round FROM app.corporate_client WHERE id = $1",
        id
    )
    .fetch_one(&f.pool)
    .await
    .unwrap();
    (r.status, r.submission_round)
}

/// Keputusan tersimpan `(putaran, keputusan, kode alasan, pemutus)` urut putaran.
async fn decisions(f: &Fixture, id: Uuid) -> Vec<(i32, String, Option<String>, Uuid)> {
    sqlx::query!(
        "SELECT submission_round, decision, reason_code, decided_by \
         FROM app.corporate_verification_decision WHERE corporate_client_id = $1 \
         ORDER BY submission_round",
        id
    )
    .fetch_all(&f.pool)
    .await
    .unwrap()
    .into_iter()
    .map(|r| (r.submission_round, r.decision, r.reason_code, r.decided_by))
    .collect()
}

/// Payload outbox berkode `event_code` dan aksi `action` dengan `tenant_id` = perusahaan.
async fn tenant_events(f: &Fixture, event_code: &str, action: &str, tenant: Uuid) -> Vec<Value> {
    sqlx::query_scalar!(
        r#"SELECT payload AS "payload!" FROM app.audit_outbox
           WHERE payload ->> 'event_code' = $1 AND payload ->> 'action_name' = $2
             AND payload ->> 'tenant_id' = $3
           ORDER BY payload ->> 'occurred_at'"#,
        event_code,
        action,
        tenant.hyphenated().to_string(),
    )
    .fetch_all(&f.relay)
    .await
    .unwrap()
}

/// Jumlah baris outbox yang teks payload-nya memuat `needle`.
async fn outbox_containing(f: &Fixture, needle: &str) -> i64 {
    sqlx::query_scalar!(
        r#"SELECT count(*) AS "n!" FROM app.audit_outbox WHERE payload::text LIKE '%' || $1 || '%'"#,
        needle,
    )
    .fetch_one(&f.relay)
    .await
    .unwrap()
}

async fn outbox_total(f: &Fixture) -> i64 {
    sqlx::query_scalar!(r#"SELECT count(*) AS "n!" FROM app.audit_outbox"#)
        .fetch_one(&f.relay)
        .await
        .unwrap()
}

/// Antrean lengkap (semua halaman, `limit=2`) sebagai daftar ID berurutan; NPWP tidak pernah ada.
async fn whole_queue(f: &Fixture, token: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cursor: Option<String> = None;
    loop {
        let path = match &cursor {
            Some(c) => format!("{BASE}?limit=2&cursor={c}"),
            None => format!("{BASE}?limit=2"),
        };
        let r = f.get(&path, token).await;
        assert_eq!(r.status, 200, "{}", r.body);
        assert_eq!(r.header("cache-control"), Some("no-store"));
        let j = r.json();
        for item in j["data"].as_array().unwrap() {
            assert!(item.get("npwp").is_none(), "antrean tanpa NPWP: {item}");
            out.push(item["corporate_client_id"].as_str().unwrap().to_string());
        }
        if j["meta"]["has_more"] == true {
            cursor = Some(j["meta"]["next_cursor"].as_str().unwrap().to_string());
        } else {
            break;
        }
    }
    out
}

// trace: US-1902 AC1 (antrean terlama lebih dulu; hanya "Menunggu verifikasi"), DD A20 (antrean tanpa NPWP)
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn queue_lists_pending_submissions_oldest_first_without_npwp() {
    let _g = SERIAL.lock().await;
    let f = Fixture::new().await;
    let (_v, tok) = f.verifier().await;
    let newest = company(&f.pool, "pending_verification", 1).await;
    let oldest = company(&f.pool, "pending_verification", 300).await;
    let middle = company(&f.pool, "pending_verification", 120).await;
    let active = company(&f.pool, "active", 500).await;
    let rejected = company(&f.pool, "rejected", 600).await;

    let ids = whole_queue(&f, &tok).await;
    let pos = |c: &Company| ids.iter().position(|x| *x == c.id.hyphenated().to_string());
    let (o, m, n) = (
        pos(&oldest).unwrap(),
        pos(&middle).unwrap(),
        pos(&newest).unwrap(),
    );
    assert!(o < m && m < n, "urut terlama: {o} {m} {n}");
    assert_eq!(pos(&active), None);
    assert_eq!(pos(&rejected), None);

    let r = f.get(&format!("{BASE}?limit=0"), &tok).await;
    assert_eq!((r.status, code(&r).as_str()), (422, "4220001"));
    let r = f.get(&format!("{BASE}?cursor=bukan-uuid"), &tok).await;
    assert_eq!((r.status, code(&r).as_str()), (422, "4220001"));
    // Kursor lama yang hanya berisi ID perusahaan tidak lagi diterima (review T1 CR-035, DBA 3).
    let r = f
        .get(&format!("{BASE}?cursor={}", oldest.id.hyphenated()), &tok)
        .await;
    assert_eq!((r.status, code(&r).as_str()), (422, "4220001"));
}

// trace: US-1902 AC1 (antrean berhalaman), US-1902 AC3 (pengajuan ulang = `submitted_at` baru), review T1 CR-035
// DBA 3 (negatif): perusahaan yang menjadi kursor diajukan ulang di antara dua halaman; pengajuan lain yang
// lebih baru dari posisi kursor tetap muncul di halaman berikutnya, tidak terlewati
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn queue_cursor_keeps_its_position_when_the_cursor_company_is_resubmitted_between_pages() {
    let _g = SERIAL.lock().await;
    let f = Fixture::new().await;
    let (_v, tok) = f.verifier().await;
    let x = company(&f.pool, "pending_verification", 300).await;
    let y = company(&f.pool, "pending_verification", 200).await;
    let z = company(&f.pool, "pending_verification", 100).await;
    let id_of = |c: &Company| c.id.hyphenated().to_string();

    // Halaman satu item sampai X menjadi item terakhir halaman.
    let mut cursor: Option<String> = None;
    loop {
        let path = match &cursor {
            Some(c) => format!("{BASE}?limit=1&cursor={c}"),
            None => format!("{BASE}?limit=1"),
        };
        let r = f.get(&path, &tok).await;
        assert_eq!(r.status, 200, "{}", r.body);
        let j = r.json();
        let item = j["data"][0]["corporate_client_id"]
            .as_str()
            .unwrap()
            .to_string();
        assert!(item != id_of(&y) && item != id_of(&z), "X harus lebih dulu");
        assert_eq!(j["meta"]["has_more"], true);
        cursor = Some(j["meta"]["next_cursor"].as_str().unwrap().to_string());
        if item == id_of(&x) {
            break;
        }
    }

    // X ditolak lalu diajukan ulang di antara dua halaman: putaran baru dengan `submitted_at` sekarang.
    sqlx::query!(
        "UPDATE app.corporate_client \
            SET submitted_at = now(), submission_round = submission_round + 1 WHERE id = $1",
        x.id
    )
    .execute(&f.pool)
    .await
    .unwrap();

    let mut rest = Vec::new();
    loop {
        let r = f
            .get(
                &format!("{BASE}?limit=1&cursor={}", cursor.as_deref().unwrap()),
                &tok,
            )
            .await;
        assert_eq!(r.status, 200, "{}", r.body);
        let j = r.json();
        for item in j["data"].as_array().unwrap() {
            rest.push(item["corporate_client_id"].as_str().unwrap().to_string());
        }
        if j["meta"]["has_more"] == true {
            cursor = Some(j["meta"]["next_cursor"].as_str().unwrap().to_string());
        } else {
            break;
        }
    }
    let pos = |c: &Company| rest.iter().position(|i| *i == id_of(c));
    let (py, pz, px) = (pos(&y), pos(&z), pos(&x));
    assert!(
        matches!((py, pz, px), (Some(a), Some(b), Some(c)) if a < b && b < c),
        "Y dan Z tidak terlewati; X muncul lagi di akhir sebagai pengajuan terbaru: {py:?} {pz:?} {px:?}"
    );
}

// trace: US-1902 AC1 (data perusahaan, dokumen NIB/NPWP/bukti kewenangan), US-1902 AC3 (alasan dari daftar)
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn detail_and_reason_list_show_company_data_documents_and_draft_reasons() {
    let _g = SERIAL.lock().await;
    let f = Fixture::new().await;
    let (_v, tok) = f.verifier().await;
    let c = company(&f.pool, "pending_verification", 5).await;

    let r = f.get(&format!("{BASE}/{}", c.id), &tok).await;
    assert_eq!(r.status, 200, "{}", r.body);
    assert_eq!(r.header("cache-control"), Some("no-store"));
    let d = &r.json()["data"];
    assert_eq!(d["corporate_client_id"], c.id.hyphenated().to_string());
    assert_eq!(d["npwp"], c.npwp.as_str(), "NPWP tampil ke Verifikator");
    assert_eq!(d["status"], "pending_verification");
    assert_eq!(d["origin"], "self_registered");
    assert_eq!(d["submission_round"], 1);
    assert_eq!(d["documents"], json!(["authority_proof", "nib", "npwp"]));
    assert_eq!(d["can_decide"], true);
    assert!(d["rejection_reason"].is_null() && d["decided_at"].is_null());

    let r = f.get(REASONS, &tok).await;
    assert_eq!(r.status, 200, "{}", r.body);
    let codes: Vec<String> = r.json()["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x["code"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(codes, FOUR);
}

// trace: US-1902 AC1 (dokumen hanya lewat URL berumur pendek setelah otorisasi, ADR-018), PRD §7.2 E-05
// (dokumen dibuka; dicatat sebelum URL terbit), KU-03 (jenis/dokumen tidak ada → 404 sama)
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn document_url_is_issued_only_after_e05_is_recorded() {
    let _g = SERIAL.lock().await;
    let f = Fixture::new().await;
    let (verifier, tok) = f.verifier().await;
    let c = company(&f.pool, "pending_verification", 5).await;

    let mut urls = Vec::new();
    for kind in ["nib", "npwp", "authority_proof"] {
        let file = file_of(&f.pool, c.id, kind).await;
        let r = f
            .post(
                &format!("{BASE}/{}/documents/{kind}/download-url", c.id),
                &tok,
                &json!({}),
            )
            .await;
        assert_eq!(r.status, 200, "{kind}: {}", r.body);
        assert_eq!(r.header("cache-control"), Some("no-store"));
        let d = &r.json()["data"];
        assert!(d["url"].as_str().unwrap().contains("X-Amz-Signature"));
        urls.push(d["url"].as_str().unwrap().to_string());
        assert_eq!(d["mime_type"], "image/jpeg");
        let events = tenant_events(&f, "e_05", "stored_file_opened", c.id).await;
        let e = events
            .iter()
            .find(|e| e["object_id"] == file.hyphenated().to_string())
            .unwrap_or_else(|| panic!("E-05 {kind} tidak tercatat"));
        assert_event_complete(e, "e_05");
        assert_eq!(e["outcome"], "success");
        assert_eq!(e["actor_account_id"], verifier.hyphenated().to_string());
        assert_eq!(e["object_type"], "corporate_document_file");
        assert_eq!(e["subjects"], json!([]), "perusahaan bukan subjek data");
    }
    // F1-T109: E-05 Verifikator atas dokumen perusahaan bertenant perusahaan, tanpa URL presign, NPWP, atau nama.
    let opened = tenant_events(&f, "e_05", "stored_file_opened", c.id).await;
    assert_eq!(opened.len(), 3);
    let tenant = c.id.to_string();
    let mut raw: Vec<&str> = urls.iter().map(String::as_str).collect();
    raw.extend([c.npwp.as_str(), "PT Sintetis Uji", "Jl. Sintetis"]);
    audit_support::assert_corporate_events_clean(&opened, &["e_05"], Some(&tenant), &raw);

    // Jenis di luar daftar, perusahaan tanpa dokumen jenis itu, dan berkas kategori lain → 404.
    let before = outbox_total(&f).await;
    let r = f
        .post(
            &format!("{BASE}/{}/documents/ktp/download-url", c.id),
            &tok,
            &json!({}),
        )
        .await;
    assert_eq!((r.status, code(&r).as_str()), (404, "4040001"));
    let bare = new_id();
    sqlx::query!(
        "INSERT INTO app.corporate_client \
           (id, legal_name, nib, npwp, office_address, office_phone, status, origin) \
         VALUES ($1, 'PT Tanpa Dokumen', $2, $3, 'Jl. Sintetis', '0210000002', 'pending_verification', \
                 'self_registered')",
        bare,
        digits(13),
        digits(16),
    )
    .execute(&f.pool)
    .await
    .unwrap();
    let r = f
        .post(
            &format!("{BASE}/{bare}/documents/nib/download-url"),
            &tok,
            &json!({}),
        )
        .await;
    assert_eq!((r.status, code(&r).as_str()), (404, "4040001"));
    assert_eq!(
        outbox_total(&f).await,
        before,
        "tanpa E-05 bila tidak dibuka"
    );

    let ktp = stored_file(&f.pool, "ktp_photo", "identity_document").await;
    document(&f.pool, bare, "npwp", ktp).await;
    let r = f
        .post(
            &format!("{BASE}/{bare}/documents/npwp/download-url"),
            &tok,
            &json!({}),
        )
        .await;
    assert_eq!(
        (r.status, code(&r).as_str()),
        (404, "4040001"),
        "berkas kategori lain tidak dapat dibuka lewat endpoint dokumen perusahaan"
    );
}

// trace: US-1902 AC2 (setuju → Aktif), PRD US-1902 audit E-10 (keputusan)
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn approval_activates_the_company_and_records_e10() {
    let _g = SERIAL.lock().await;
    let f = Fixture::new().await;
    let (verifier, tok) = f.verifier().await;
    let c = company(&f.pool, "pending_verification", 5).await;

    let r = f.approve(&tok, c.id).await;
    assert_eq!(r.status, 200, "{}", r.body);
    let d = &r.json()["data"];
    assert_eq!(d["status"], "active");
    assert_eq!(d["submission_round"], 1);
    assert!(d["rejection_reason"].is_null());
    assert_eq!(client_status(&f, c.id).await, ("active".to_string(), 1));
    assert_eq!(
        decisions(&f, c.id).await,
        vec![(1, "approved".to_string(), None, verifier)]
    );
    let events = tenant_events(&f, "e_10", "corporate_verification_approved", c.id).await;
    assert_eq!(events.len(), 1);
    let e = &events[0];
    assert_event_complete(e, "e_10");
    assert_eq!(e["outcome"], "success");
    assert_eq!(e["object_type"], "corporate_client");
    assert_eq!(e["object_id"], c.id.hyphenated().to_string());
    assert_eq!(e["before_values"]["status"], "pending_verification");
    assert_eq!(e["after_values"]["status"], "active");
    assert_eq!(e["after_values"]["submission_round"], 1);
    assert_eq!(e["source_component"], "backoffice");
    assert!(e["source_ip"].is_string());
    let tenant = c.id.to_string();
    audit_support::assert_corporate_events_clean(
        &events,
        &["e_10"],
        Some(&tenant),
        &[
            c.npwp.as_str(),
            "PT Sintetis Uji",
            "Jl. Sintetis",
            "0210000001",
        ],
    );

    let r = f.get(&format!("{BASE}/{}", c.id), &tok).await;
    let d = &r.json()["data"];
    assert_eq!(d["status"], "active");
    assert_eq!(d["can_decide"], false);
    assert!(d["decided_at"].is_string());
    assert!(d["rejection_reason"].is_null());
}

// trace: US-1902 AC3 (tolak dengan kode alasan termasuk bukti kewenangan; daftar dari migrasi data), E-10
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn rejection_requires_an_active_listed_reason() {
    let _g = SERIAL.lock().await;
    let f = Fixture::new().await;
    let (verifier, tok) = f.verifier().await;
    let c = company(&f.pool, "pending_verification", 5).await;

    // Di luar daftar, badan rusak, dan field tak dikenal: ditolak tanpa efek.
    let before = outbox_total(&f).await;
    let ver = f.version(&tok, c.id).await;
    for body in [
        json!({ "reason_code": "other", "documents_version": ver }),
        json!({ "reason_code": "photo_unreadable", "documents_version": ver }),
        json!({ "reason_code": "", "documents_version": ver }),
        json!({ "reason_code": "authority_proof_insufficient", "documents_version": ver, "note": "x" }),
        json!({ "reason_code": "authority_proof_insufficient" }),
        json!({}),
    ] {
        let r = f
            .post(&format!("{BASE}/{}/reject", c.id), &tok, &body)
            .await;
        assert_eq!((r.status, code(&r).as_str()), (422, "4220001"), "{body}");
    }
    let r = request(
        f.addr,
        "POST",
        &format!("{BASE}/{}/reject", c.id),
        &[&format!("Authorization: Bearer {tok}")],
    )
    .await;
    assert_eq!((r.status, code(&r).as_str()), (415, "4150001"));
    assert_eq!(
        client_status(&f, c.id).await,
        ("pending_verification".to_string(), 1)
    );
    assert!(decisions(&f, c.id).await.is_empty());
    assert_eq!(outbox_total(&f).await, before);

    // Kode dikenal kode tetapi tidak aktif di reason_code (dihapus lunak lewat migrasi data) → 422.
    sqlx::query!(
        "UPDATE app.reason_code SET deleted_at = now(), delete_reason = 'uji_f1_t104' \
         WHERE context = 'corporate_rejection' AND code = 'company_data_mismatch' AND deleted_at IS NULL"
    )
    .execute(&f.pool)
    .await
    .unwrap();
    let r = f.reject(&tok, c.id, "company_data_mismatch").await;
    let listed = f.get(REASONS, &tok).await.json()["data"]
        .as_array()
        .unwrap()
        .len();
    sqlx::query!(
        "UPDATE app.reason_code SET deleted_at = NULL, deleted_by = NULL, delete_reason = NULL \
         WHERE context = 'corporate_rejection' AND code = 'company_data_mismatch' \
           AND delete_reason = 'uji_f1_t104'"
    )
    .execute(&f.pool)
    .await
    .unwrap();
    assert_eq!((r.status, code(&r).as_str()), (422, "4220001"));
    assert_eq!(listed, 3, "kode nonaktif tidak ditawarkan");
    assert!(decisions(&f, c.id).await.is_empty());

    let r = f.reject(&tok, c.id, "authority_proof_insufficient").await;
    assert_eq!(r.status, 200, "{}", r.body);
    let d = &r.json()["data"];
    assert_eq!(d["status"], "rejected");
    assert_eq!(d["rejection_reason"], "authority_proof_insufficient");
    assert_eq!(client_status(&f, c.id).await, ("rejected".to_string(), 1));
    assert_eq!(
        decisions(&f, c.id).await,
        vec![(
            1,
            "rejected".to_string(),
            Some("authority_proof_insufficient".to_string()),
            verifier
        )]
    );
    let events = tenant_events(&f, "e_10", "corporate_verification_rejected", c.id).await;
    assert_eq!(events.len(), 1);
    assert_event_complete(&events[0], "e_10");
    assert_eq!(
        events[0]["after_values"]["reason_code"],
        "authority_proof_insufficient"
    );
    assert_eq!(events[0]["after_values"]["status"], "rejected");

    let d = f.get(&format!("{BASE}/{}", c.id), &tok).await.json()["data"].clone();
    assert_eq!(d["rejection_reason"], "authority_proof_insufficient");
    assert_eq!(d["can_decide"], false);
}

// trace: US-1902 AC4 (dua Verifikator hampir bersamaan: yang kedua ditolak dan tercatat), AB-36
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn first_decision_wins_when_two_verifiers_decide_concurrently() {
    let _g = SERIAL.lock().await;
    let f = Fixture::new().await;
    for round in 0..3 {
        let (_a, ta) = f.verifier().await;
        let (_b, tb) = f.verifier().await;
        let c = company(&f.pool, "pending_verification", 5).await;
        let (ra, rb) = tokio::join!(
            f.approve(&ta, c.id),
            f.reject(&tb, c.id, "nib_document_invalid")
        );
        let mut statuses = [ra.status, rb.status];
        statuses.sort_unstable();
        assert_eq!(
            statuses,
            [200, 409],
            "putaran {round}: {} | {}",
            ra.body,
            rb.body
        );
        let loser = if ra.status == 409 { &ra } else { &rb };
        assert_eq!(code(loser), "4090001");
        let stored = decisions(&f, c.id).await;
        assert_eq!(stored.len(), 1, "satu keputusan per putaran");
        let expected = if ra.status == 200 {
            "active"
        } else {
            "rejected"
        };
        assert_eq!(client_status(&f, c.id).await.0, expected);
        let lost_action = if ra.status == 409 {
            "corporate_verification_approved"
        } else {
            "corporate_verification_rejected"
        };
        let failed: Vec<Value> = tenant_events(&f, "e_10", lost_action, c.id)
            .await
            .into_iter()
            .filter(|e| e["outcome"] == "failure")
            .collect();
        assert_eq!(failed.len(), 1, "pihak kalah tercatat E-10 hasil gagal");
        assert_eq!(failed[0]["outcome_reason_code"], "already_decided");
        assert_event_complete(&failed[0], "e_10");
    }

    // Berurutan: keputusan kedua atas perusahaan yang sudah diputuskan → 409, tanpa perubahan.
    let (_v, tok) = f.verifier().await;
    let c = company(&f.pool, "pending_verification", 5).await;
    assert_eq!(f.approve(&tok, c.id).await.status, 200);
    for r in [
        f.approve(&tok, c.id).await,
        f.reject(&tok, c.id, "npwp_document_invalid").await,
    ] {
        assert_eq!(
            (r.status, code(&r).as_str()),
            (409, "4090001"),
            "{}",
            r.body
        );
    }
    assert_eq!(client_status(&f, c.id).await.0, "active");
    assert_eq!(decisions(&f, c.id).await.len(), 1);
    // Perusahaan nonaktif juga tidak dapat diputuskan.
    let inactive = company(&f.pool, "inactive", 5).await;
    let r = f.approve(&tok, inactive.id).await;
    assert_eq!((r.status, code(&r).as_str()), (409, "4090001"));
    assert!(decisions(&f, inactive.id).await.is_empty());
}

// trace: US-1902 AC3 (HR memperbaiki dan mengajukan ulang setelah ditolak), US-1902 AC4 (UQ per putaran)
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn resubmission_after_rejection_opens_a_new_round() {
    let _g = SERIAL.lock().await;
    let f = Fixture::new().await;
    let (v1, t1) = f.verifier().await;
    let (v2, t2) = f.verifier().await;
    let c = company(&f.pool, "pending_verification", 30).await;

    assert_eq!(
        f.reject(&t1, c.id, "authority_proof_insufficient")
            .await
            .status,
        200
    );
    assert!(
        !whole_queue(&f, &t1)
            .await
            .contains(&c.id.hyphenated().to_string())
    );
    resubmit(&f.pool, c.id).await;
    assert!(
        whole_queue(&f, &t1)
            .await
            .contains(&c.id.hyphenated().to_string()),
        "pengajuan ulang kembali ke antrean"
    );
    let d = f.get(&format!("{BASE}/{}", c.id), &t2).await.json()["data"].clone();
    assert_eq!(d["status"], "pending_verification");
    assert_eq!(d["submission_round"], 2);
    assert_eq!(d["can_decide"], true);
    assert!(
        d["rejection_reason"].is_null() && d["decided_at"].is_null(),
        "alasan putaran lama tidak dibawa ke putaran baru"
    );

    let r = f.approve(&t2, c.id).await;
    assert_eq!(r.status, 200, "{}", r.body);
    assert_eq!(r.json()["data"]["submission_round"], 2);
    assert_eq!(client_status(&f, c.id).await, ("active".to_string(), 2));
    assert_eq!(
        decisions(&f, c.id).await,
        vec![
            (
                1,
                "rejected".to_string(),
                Some("authority_proof_insufficient".to_string()),
                v1
            ),
            (2, "approved".to_string(), None, v2),
        ]
    );
    let r = f.reject(&t1, c.id, "nib_document_invalid").await;
    assert_eq!((r.status, code(&r).as_str()), (409, "4090001"));
}

// trace: DD A20 (NPWP Terbatas: tidak di log, payload audit, atau pesan galat), ADR-005, KEPUTUSAN-122
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn npwp_never_reaches_audit_payloads_or_error_bodies() {
    let _g = SERIAL.lock().await;
    let f = Fixture::new().await;
    let (_v, tok) = f.verifier().await;
    let c = company(&f.pool, "pending_verification", 5).await;

    let mut bodies = Vec::new();
    bodies.push(f.get(&format!("{BASE}/{}", c.id), &tok).await);
    for kind in ["nib", "npwp", "authority_proof"] {
        bodies.push(
            f.post(
                &format!("{BASE}/{}/documents/{kind}/download-url", c.id),
                &tok,
                &json!({}),
            )
            .await,
        );
    }
    // Galat yang menggemakan masukan pun tidak boleh memuat NPWP.
    let failures = [
        f.post(
            &format!("{BASE}/{}/reject", c.id),
            &tok,
            &json!({ "reason_code": c.npwp }),
        )
        .await,
        f.post(
            &format!("{BASE}/{}/reject", c.id),
            &tok,
            &json!({ "reason_code": "authority_proof_insufficient", "documents_version": "0", "npwp": c.npwp }),
        )
        .await,
        f.get(&format!("{BASE}/{}", c.npwp), &tok).await,
        f.get(&format!("{BASE}?cursor={}", c.npwp), &tok).await,
    ];
    for r in &failures {
        assert!(r.status >= 400, "{}", r.body);
        assert!(!r.body.contains(&c.npwp), "NPWP bocor di galat: {}", r.body);
    }
    bodies.push(f.reject(&tok, c.id, "npwp_document_invalid").await);
    bodies.push(f.approve(&tok, c.id).await);
    for r in &bodies[4..] {
        assert!(
            !r.body.contains(&c.npwp),
            "NPWP di respons keputusan: {}",
            r.body
        );
    }
    assert!(bodies[0].body.contains(&c.npwp), "detail memuat NPWP");
    assert_eq!(
        outbox_containing(&f, &c.npwp).await,
        0,
        "NPWP tidak boleh masuk payload audit"
    );
    assert!(
        !tenant_events(&f, "e_10", "corporate_verification_approved", c.id)
            .await
            .is_empty(),
        "kegagalan keputusan tercatat"
    );
}

fn endpoints(id: Uuid) -> Vec<(&'static str, String, Option<Value>)> {
    vec![
        ("GET", BASE.to_string(), None),
        ("GET", format!("{BASE}/{id}"), None),
        (
            "POST",
            format!("{BASE}/{id}/documents/nib/download-url"),
            Some(json!({})),
        ),
        (
            "POST",
            format!("{BASE}/{id}/approve"),
            Some(json!({ "documents_version": "0".repeat(64) })),
        ),
        (
            "POST",
            format!("{BASE}/{id}/reject"),
            Some(
                json!({ "reason_code": "nib_document_invalid", "documents_version": "0".repeat(64) }),
            ),
        ),
        ("GET", REASONS.to_string(), None),
    ]
}

async fn call(f: &Fixture, e: &(&str, String, Option<Value>), token: Option<&str>) -> Resp {
    let headers: Vec<String> = token
        .map(|t| vec![format!("Authorization: Bearer {t}")])
        .unwrap_or_default();
    let refs: Vec<&str> = headers.iter().map(String::as_str).collect();
    match &e.2 {
        None => request(f.addr, e.0, &e.1, &refs).await,
        Some(b) => request_json(f.addr, e.0, &e.1, &refs, b).await,
    }
}

// trace: US-1902 AC5 (admin tanpa sub-peran Verifikator → 403; akun HR → ditolak), KU-03, KU-10,
// coding-standards §7.3 (tanpa token / realm lain → 401; akun nonaktif → 4010002; ID asing → 404; tanpa efek)
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn access_is_denied_to_everyone_but_active_verifiers() {
    let _g = SERIAL.lock().await;
    let f = Fixture::new().await;
    let c = company(&f.pool, "pending_verification", 5).await;
    let before = outbox_containing(&f, &c.id.hyphenated().to_string()).await;

    let hr = common::token(
        &f.keys.korporat,
        KORPORAT_ISS,
        KORPORAT_AUD,
        &format!("uji-hr-{}", files::random_hex(6)),
    );
    let patient = common::token(
        &f.keys.pasien,
        PASIEN_ISS,
        PASIEN_AUD,
        &format!("uji-pasien-{}", files::random_hex(6)),
    );
    for e in endpoints(c.id) {
        for (who, tok) in [
            ("tanpa token", None),
            ("HR", Some(hr.as_str())),
            ("pasien", Some(patient.as_str())),
        ] {
            let r = call(&f, &e, tok).await;
            assert_eq!(
                (r.status, code(&r).as_str()),
                (401, "4010001"),
                "{who} {} {}",
                e.0,
                e.1
            );
        }
    }
    for role in AdminSubRole::ALL
        .into_iter()
        .filter(|r| *r != AdminSubRole::Verifier)
    {
        let (_id, sub) = new_admin(&f.pool, &[role]).await;
        let tok = f.admin_token(&sub);
        for e in endpoints(c.id) {
            let r = call(&f, &e, Some(&tok)).await;
            assert_eq!(
                (r.status, code(&r).as_str()),
                (403, "4030001"),
                "{role:?} {} {}",
                e.0,
                e.1
            );
        }
    }
    let (inactive, sub) = new_admin(&f.pool, &[AdminSubRole::Verifier]).await;
    sqlx::query!(
        "UPDATE app.account SET status = 'inactive', status_changed_at = now() WHERE id = $1",
        inactive
    )
    .execute(&f.pool)
    .await
    .unwrap();
    for e in endpoints(c.id) {
        let r = call(&f, &e, Some(&f.admin_token(&sub))).await;
        assert_eq!(
            (r.status, code(&r).as_str()),
            (401, "4010002"),
            "{} {}",
            e.0,
            e.1
        );
    }
    assert_eq!(
        client_status(&f, c.id).await,
        ("pending_verification".to_string(), 1),
        "tidak ada efek"
    );
    assert!(decisions(&f, c.id).await.is_empty());
    assert_eq!(
        outbox_containing(&f, &c.id.hyphenated().to_string()).await,
        before
    );

    // ID tidak ada, terhapus lunak, atau bukan UUID → 404 untuk Verifikator sah.
    let (_v, tok) = f.verifier().await;
    let deleted = company(&f.pool, "pending_verification", 5).await;
    sqlx::query!(
        "UPDATE app.corporate_client SET deleted_at = now(), delete_reason = 'uji' WHERE id = $1",
        deleted.id
    )
    .execute(&f.pool)
    .await
    .unwrap();
    for target in [
        new_id().hyphenated().to_string(),
        deleted.id.hyphenated().to_string(),
        "bukan-uuid".to_string(),
    ] {
        for (m, path, body) in [
            ("GET", format!("{BASE}/{target}"), None),
            (
                "POST",
                format!("{BASE}/{target}/documents/nib/download-url"),
                Some(json!({})),
            ),
            (
                "POST",
                format!("{BASE}/{target}/approve"),
                Some(json!({ "documents_version": "0".repeat(64) })),
            ),
            (
                "POST",
                format!("{BASE}/{target}/reject"),
                Some(
                    json!({ "reason_code": "nib_document_invalid", "documents_version": "0".repeat(64) }),
                ),
            ),
        ] {
            let r = call(&f, &(m, path.clone(), body), Some(&tok)).await;
            assert_eq!(
                (r.status, code(&r).as_str()),
                (404, "4040001"),
                "{m} {path}"
            );
        }
    }
    assert!(decisions(&f, deleted.id).await.is_empty());
}
