//! Teks persetujuan pemrosesan data (F1-T49; US-104 AC1, AC3; T1 persetujuan) terhadap PostgreSQL 18 dengan
//! skema migrasi terbaru dan role `rs_app`:
//! - versi draf hasil seed migrasi (KEPUTUSAN-289) tersaji lengkap: tujuh bagian Pasal 21 ayat (1), kontak,
//!   penanda "menunggu L-08/PT-24", dan butir wajib;
//! - versi yang sudah terbit tidak dapat diubah (trigger) dan butirnya tidak dapat ditambah atau diubah;
//! - versi baru tampil tepat mulai `effective_from` menurut jam server aplikasi;
//! - negatif: isi versi rusak tidak disajikan (500 tanpa isi galat), versi tanpa butir wajib tidak disajikan,
//!   belum ada versi berlaku = 503, batas laju = 429;
//! - gate production (KEPUTUSAN-289): pemeriksaan kesiapan `consent_text_final` gagal selama versi berlaku
//!   draf atau tidak ada, dan lulus untuk versi final.
//!
//! Versi uji memakai `effective_from` jauh di masa depan (2090-an) supaya tidak berlaku bagi test berkas lain
//! yang memakai jam sistem. PostgreSQL sekali pakai dari `scripts/db-ephemeral.sh`; tanpa `RS_TEST_DB_*`
//! test GAGAL.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)] // berkas test

mod audit_support;
mod common;

use audit::{AuditContext, EventCode, Source, SourceComponent};
use common::{Resp, TestKeys, request};
use foundation::{Clock, CorrelationId, FixedClock, SystemClock, Uuid, new_id};
use observability::EnvSource;
use rs_backend::app::{AppServices, AppState};
use rs_backend::modules::platform::api::ReadinessCheck;
use rs_backend::modules::privacy::api::{
    ConsentSubject, ConsentTextReadiness, ConsentType, DraftPolicy, PrivacyError,
    REQUIRED_SECTIONS, SelectionCheck, active_consent_text, check_selection,
    record_granted_consent,
};
use rs_backend::rate_limit::{RateBudget, RateLimiter};
use serde_json::json;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use std::sync::Arc;
use time::OffsetDateTime;
use time::macros::datetime;

const PATH: &str = "/patient/v1/consent-texts/current";
const SEED_VERSION: &str = "01a2008d-7400-7000-8000-000000000a49";
const SEED_ITEM: &str = "01a2008d-7400-7000-8000-000000000a4a";

/// Test berbagi versi teks per jenis: dijalankan satu per satu supaya versi uji satu test tidak berlaku di
/// test lain yang memakai jam sesudahnya.
static SERIAL: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

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

async fn spawn(pool: &PgPool, clock: Arc<FixedClock>, limit: Arc<RateLimiter>) -> SocketAddr {
    spawn_with(pool, clock, limit, false).await
}

/// `production` = proses `api` dengan `DEPLOY_ENV=production`: teks draf ditolak di jalur layanan.
async fn spawn_with(
    pool: &PgPool,
    clock: Arc<FixedClock>,
    limit: Arc<RateLimiter>,
    production: bool,
) -> SocketAddr {
    let keys = TestKeys::generate();
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
            files: Arc::new(common::unused_file_storage()),
            public_content_limit: common::roomy_limiter(),
            public_image_limit: common::roomy_limiter(),
            consent_text_limit: limit,
            corporate_hr: common::corporate_hr(common::unreachable_pool()),
        },
    );
    let state = if production {
        state.reject_consent_drafts()
    } else {
        state
    };
    common::spawn_app(
        state,
        common::auth_state(&keys, Arc::new(SystemClock)),
        Arc::new(common::NoAccounts),
        Arc::new(common::NoAccounts),
    )
    .await
}

fn valid_content(label: &str) -> String {
    let sections: Vec<_> = REQUIRED_SECTIONS
        .iter()
        .map(|c| json!({"code": c, "title": format!("Judul {c} {label}"), "body": format!("Isi {c} sintetis")}))
        .collect();
    json!({
        "format": "rs-consent-text/1",
        "sections": sections,
        "contact": {"title": "Kontak", "body": "Kontak sintetis"},
    })
    .to_string()
}

/// Versi uji terbit dengan butir: disisipkan role `rs_app` (INSERT versi belum terbit, butir, lalu UPDATE
/// `published_at`), sama dengan jalur yang diizinkan hak role dan trigger.
async fn publish_version(
    pool: &PgPool,
    effective_from: OffsetDateTime,
    content: &str,
    required_item: bool,
) -> Uuid {
    let id = new_id();
    let label = format!("uji-{}", id.simple());
    sqlx::query!(
        "INSERT INTO app.consent_text_version (id, consent_type, version_label, content, effective_from) \
         VALUES ($1, 'patient_data_processing', $2, $3, $4)",
        id,
        label,
        content,
        effective_from,
    )
    .execute(pool)
    .await
    .unwrap();
    sqlx::query!(
        "INSERT INTO app.consent_item (id, consent_text_version_id, item_code, label, is_primary, is_required, sort_order) \
         VALUES ($1, $2, 'data_processing', 'Butir sintetis', $3, $3, 1)",
        new_id(),
        id,
        required_item,
    )
    .execute(pool)
    .await
    .unwrap();
    sqlx::query!(
        "UPDATE app.consent_text_version SET published_at = $2, updated_at = $2 WHERE id = $1",
        id,
        effective_from - time::Duration::days(1),
    )
    .execute(pool)
    .await
    .unwrap();
    id
}

async fn get(addr: SocketAddr) -> Resp {
    request(addr, "GET", PATH, &[]).await
}

// trace: US-104 AC1
#[tokio::test]
async fn seeded_draft_text_has_seven_sections_contact_and_required_item() {
    let _g = SERIAL.lock().await;
    let pool = app_pool().await;
    let clock = Arc::new(FixedClock::new(SystemClock.now()));
    let addr = spawn(&pool, clock, common::roomy_limiter()).await;
    let r = get(addr).await;
    assert_eq!(r.status, 200, "{}", r.body);
    assert_eq!(r.header("content-type"), Some("application/json"));
    let v = r.json();
    assert_eq!(v["response_code"], "2000000");
    let d = &v["data"];
    assert_eq!(d["consent_text_version_id"], SEED_VERSION);
    assert_eq!(d["consent_type"], "patient_data_processing");
    assert_eq!(d["version_label"], "0.1-draf");
    let codes: Vec<&str> = d["sections"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["code"].as_str().unwrap())
        .collect();
    assert_eq!(codes, REQUIRED_SECTIONS);
    for s in d["sections"].as_array().unwrap() {
        assert!(!s["title"].as_str().unwrap().is_empty());
        assert!(!s["body"].as_str().unwrap().is_empty());
    }
    assert!(!d["contact"]["body"].as_str().unwrap().is_empty());
    // Penanda draf (KEPUTUSAN-289, klarifikasi T-10): menyebut L-08 dan PT-24.
    let notice = d["draft_notice"].as_str().unwrap();
    assert!(
        notice.contains("L-08") && notice.contains("PT-24"),
        "{notice}"
    );
    assert_eq!(
        d["sections"][0]["body"],
        "[DRAF] Teks bagian ini menunggu L-08 (tim legal)."
    );
    let items = d["items"].as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["consent_item_id"], SEED_ITEM);
    assert_eq!(items[0]["item_code"], "data_processing");
    assert_eq!(items[0]["is_required"], true);
    assert_eq!(items[0]["is_primary"], true);
    // Waktu ISO 8601 dengan offset (ADR-017 butir 1).
    assert!(
        OffsetDateTime::parse(
            d["effective_from"].as_str().unwrap(),
            &time::format_description::well_known::Rfc3339
        )
        .is_ok()
    );
    // Tanpa cookie (endpoint publik).
    assert!(r.header("set-cookie").is_none());
}

// trace: US-104 AC3
#[tokio::test]
async fn published_version_and_its_items_cannot_be_changed_by_app_role() {
    let _g = SERIAL.lock().await;
    let pool = app_pool().await;
    let seed = Uuid::parse_str(SEED_VERSION).unwrap();
    let e = sqlx::query!(
        "UPDATE app.consent_text_version SET content = 'diubah', updated_at = now() WHERE id = $1",
        seed,
    )
    .execute(&pool)
    .await
    .unwrap_err();
    assert_eq!(
        e.as_database_error().and_then(|d| d.code()).as_deref(),
        Some("42501"),
        "versi terbit ditolak trigger"
    );
    let e = sqlx::query!(
        "UPDATE app.consent_item SET label = 'diubah', updated_at = now() WHERE consent_text_version_id = $1",
        seed,
    )
    .execute(&pool)
    .await
    .unwrap_err();
    assert_eq!(
        e.as_database_error().and_then(|d| d.code()).as_deref(),
        Some("42501")
    );
    let e = sqlx::query!(
        "INSERT INTO app.consent_item (id, consent_text_version_id, item_code, label, is_primary, is_required, sort_order) \
         VALUES ($1, $2, 'tambahan', 'x', false, false, 2)",
        new_id(),
        seed,
    )
    .execute(&pool)
    .await
    .unwrap_err();
    assert_eq!(
        e.as_database_error().and_then(|d| d.code()).as_deref(),
        Some("42501"),
        "butir baru untuk versi terbit ditolak"
    );
    // Role aplikasi tidak punya DELETE atas data master (ADR-003).
    let e = sqlx::query!("DELETE FROM app.consent_text_version WHERE id = $1", seed)
        .execute(&pool)
        .await
        .unwrap_err();
    assert_eq!(
        e.as_database_error().and_then(|d| d.code()).as_deref(),
        Some("42501")
    );
}

// trace: US-104 AC3
#[tokio::test]
async fn new_version_is_served_from_effective_from_onwards() {
    let _g = SERIAL.lock().await;
    let pool = app_pool().await;
    let effective = datetime!(2091-01-01 00:00:00 UTC);
    let id = publish_version(&pool, effective, &valid_content("baru"), true).await;
    let clock = Arc::new(FixedClock::new(effective - time::Duration::seconds(1)));
    let addr = spawn(&pool, clock.clone(), common::roomy_limiter()).await;
    let before = get(addr).await.json();
    assert_ne!(before["data"]["consent_text_version_id"], id.to_string());
    clock.set(effective);
    let at = get(addr).await.json();
    assert_eq!(at["data"]["consent_text_version_id"], id.to_string());
    // Teks final tanpa penanda draf.
    assert!(at["data"]["draft_notice"].is_null());
}

// Negatif T1: isi rusak dan versi tanpa butir wajib tidak pernah disajikan.
#[tokio::test]
async fn malformed_content_or_version_without_required_item_is_not_served() {
    let _g = SERIAL.lock().await;
    let pool = app_pool().await;
    let broken_at = datetime!(2092-01-01 00:00:00 UTC);
    let mut six: serde_json::Value = serde_json::from_str(&valid_content("rusak")).unwrap();
    six["sections"].as_array_mut().unwrap().pop();
    publish_version(&pool, broken_at, &six.to_string(), true).await;
    let clock = Arc::new(FixedClock::new(broken_at));
    let addr = spawn(&pool, clock.clone(), common::roomy_limiter()).await;
    let r = get(addr).await;
    assert_eq!(r.status, 500, "{}", r.body);
    assert_eq!(r.json()["response_code"], "5000001");
    assert!(!r.body.contains("legal_basis") && !r.body.contains("sections"));

    let optional_only_at = datetime!(2093-01-01 00:00:00 UTC);
    publish_version(&pool, optional_only_at, &valid_content("opsional"), false).await;
    clock.set(optional_only_at);
    let r = get(addr).await;
    assert_eq!(r.status, 500, "{}", r.body);
}

#[tokio::test]
async fn no_effective_version_is_unavailable() {
    let _g = SERIAL.lock().await;
    let pool = app_pool().await;
    let clock = Arc::new(FixedClock::new(datetime!(2020-01-01 00:00:00 UTC)));
    let addr = spawn(&pool, clock, common::roomy_limiter()).await;
    let r = get(addr).await;
    assert_eq!(r.status, 503, "{}", r.body);
    assert_eq!(r.json()["response_code"], "5030001");
    assert!(r.json()["correlation_id"].is_string());
}

#[tokio::test]
async fn consent_text_is_rate_limited_per_source() {
    let _g = SERIAL.lock().await;
    let pool = app_pool().await;
    let clock = Arc::new(FixedClock::new(SystemClock.now()));
    let limit = Arc::new(RateLimiter::new(RateBudget {
        per_source: 2,
        global: 100,
    }));
    let addr = spawn(&pool, clock, limit).await;
    assert_eq!(get(addr).await.status, 200);
    assert_eq!(get(addr).await.status, 200);
    let r = get(addr).await;
    assert_eq!(r.status, 429, "{}", r.body);
    assert_eq!(r.json()["response_code"], "4290001");
}

// trace: KEPUTUSAN-289 (production tidak siap selama teks persetujuan berlaku masih draf)
#[tokio::test]
async fn production_readiness_requires_final_effective_text() {
    let _g = SERIAL.lock().await;
    let pool = app_pool().await;
    let clock = Arc::new(FixedClock::new(SystemClock.now()));
    let check = ConsentTextReadiness::new(pool.clone(), clock.clone());
    assert_eq!(check.name(), "consent_text_final");
    assert!(!check.check().await, "versi berlaku = draf hasil seed");
    clock.set(datetime!(2020-01-01 00:00:00 UTC));
    assert!(!check.check().await, "belum ada versi berlaku");
    let effective = datetime!(2094-01-01 00:00:00 UTC);
    publish_version(&pool, effective, &valid_content("final"), true).await;
    clock.set(effective);
    assert!(check.check().await, "versi final berlaku");
    // Database tidak dapat dijangkau = tidak siap.
    let down = ConsentTextReadiness::new(common::unreachable_pool(), clock.clone());
    assert!(!down.check().await);
}

async fn make_patient(pool: &PgPool) -> (Uuid, Uuid, Uuid) {
    let account_id = new_id();
    let patient_id = new_id();
    sqlx::query!(
        "INSERT INTO app.account (id, account_type, idp_realm, idp_subject, email, status, status_changed_at) \
         VALUES ($1, 'patient', 'pasien', $2, $3, 'active', now())",
        account_id,
        format!("uji-{}", account_id.simple()),
        format!("uji-{}@example.com", account_id.simple()),
    )
    .execute(pool)
    .await
    .unwrap();
    sqlx::query!(
        "INSERT INTO app.patient (id, account_id, processing_status) VALUES ($1, $2, 'normal')",
        patient_id,
        account_id,
    )
    .execute(pool)
    .await
    .unwrap();
    let owner_recipient = new_id();
    sqlx::query!(
        "INSERT INTO app.care_recipient (id, patient_id, is_account_owner, relationship, full_name, birth_date, sex) \
         VALUES ($1, $2, true, NULL, 'Pasien Uji', DATE '1990-01-01', 'female')",
        owner_recipient,
        patient_id,
    )
    .execute(pool)
    .await
    .unwrap();
    (patient_id, account_id, owner_recipient)
}

// trace: US-104 AC2, AC3, AC5, E-13 (API untuk modul lain: baca versi, periksa pilihan, tulis bukti + event)
#[tokio::test]
async fn other_modules_read_active_text_check_selection_and_record_consent_with_e13() {
    let _g = SERIAL.lock().await;
    let pool = app_pool().await;
    let clock = FixedClock::new(SystemClock.now());
    let text = active_consent_text(
        &pool,
        &clock,
        ConsentType::PatientDataProcessing,
        DraftPolicy::Allow,
    )
    .await
    .unwrap()
    .expect("seed berlaku");
    assert_eq!(text.consent_type, "patient_data_processing");
    // Jenis lain tanpa versi berlaku: None, bukan galat.
    assert!(
        active_consent_text(
            &pool,
            &clock,
            ConsentType::GuardianConsent,
            DraftPolicy::Allow,
        )
        .await
        .unwrap()
        .is_none()
    );
    let required: Vec<Uuid> = text
        .items
        .iter()
        .filter(|i| i.is_required)
        .map(|i| i.id)
        .collect();
    assert_eq!(
        check_selection(text.version_id, &text.items, text.version_id, &[]),
        SelectionCheck::MissingRequired
    );
    assert_eq!(
        check_selection(new_id(), &text.items, text.version_id, &required),
        SelectionCheck::Outdated
    );
    let SelectionCheck::Accepted(chosen) =
        check_selection(text.version_id, &text.items, text.version_id, &required)
    else {
        panic!("pilihan butir wajib harus diterima");
    };
    let items: Vec<_> = text
        .items
        .iter()
        .filter(|i| chosen.contains(&i.id))
        .cloned()
        .collect();

    let (patient_id, account_id, owner_recipient) = make_patient(&pool).await;

    let ctx = AuditContext::new(
        CorrelationId::new(),
        Source::component(SourceComponent::PatientApp).ip("198.51.100.7".parse().unwrap()),
    );
    let mut tx = pool.begin().await.unwrap();
    let events = record_granted_consent(
        &mut tx,
        ConsentSubject::Patient {
            patient_id,
            account_id,
            owner_care_recipient_id: owner_recipient,
        },
        text.version_id,
        &items,
        &ctx,
        new_id(),
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();
    assert_eq!(events.len(), items.len());
    for e in events {
        assert_eq!(e.code(), EventCode::E13);
        let rec = e.finish(&clock).unwrap();
        // Payload tersimpan lengkap menurut PRD §7.2 dan tanpa data terlarang §7.3 (US-1501 AC1).
        audit_support::assert_event_complete(rec.payload(), "e_13");
        assert_eq!(rec.payload()["action_name"], "consent_granted");
    }
    let n = sqlx::query_scalar!(
        "SELECT count(*) FROM app.consent_record WHERE patient_id = $1 AND action = 'granted' \
           AND channel = 'patient_app' AND consent_text_version_id = $2",
        patient_id,
        text.version_id,
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(n, Some(i64::try_from(items.len()).unwrap()));
}

// trace: KEPUTUSAN-289 (production tidak pernah menyajikan teks draf di jalur layanan, bukan hanya di probe)
#[tokio::test]
async fn production_rejects_draft_text_on_endpoint_and_api_but_serves_final() {
    let _g = SERIAL.lock().await;
    let pool = app_pool().await;
    let clock = Arc::new(FixedClock::new(SystemClock.now()));
    // Seed draf berlaku: endpoint production 503 tanpa membocorkan isi draf; non-production tetap 200.
    let addr = spawn_with(&pool, clock.clone(), common::roomy_limiter(), true).await;
    let r = get(addr).await;
    assert_eq!(r.status, 503, "{}", r.body);
    assert_eq!(r.json()["response_code"], "5030001");
    assert!(!r.body.contains("draf"), "{}", r.body);
    assert_eq!(
        get(spawn(&pool, clock.clone(), common::roomy_limiter()).await)
            .await
            .status,
        200
    );
    // privacy::api menolak draf dengan Reject, menyajikannya dengan Allow.
    let t = ConsentType::PatientDataProcessing;
    assert_eq!(
        active_consent_text(&pool, clock.as_ref(), t, DraftPolicy::Reject)
            .await
            .unwrap_err(),
        PrivacyError::Unavailable
    );
    assert!(
        active_consent_text(&pool, clock.as_ref(), t, DraftPolicy::Allow)
            .await
            .unwrap()
            .is_some()
    );
    // Versi final berlaku: production melayani.
    let effective = datetime!(2095-01-01 00:00:00 UTC);
    publish_version(&pool, effective, &valid_content("final-prod"), true).await;
    clock.set(effective);
    let r = get(addr).await;
    assert_eq!(r.status, 200, "{}", r.body);
    assert!(
        active_consent_text(&pool, clock.as_ref(), t, DraftPolicy::Reject)
            .await
            .unwrap()
            .is_some()
    );
}

// trace: US-104 AC2 (butir asing ditolak: tidak ada consent_record maupun event E-13)
#[tokio::test]
async fn record_rejects_item_not_belonging_to_version_and_writes_nothing() {
    let _g = SERIAL.lock().await;
    let pool = app_pool().await;
    let clock = FixedClock::new(SystemClock.now());
    let text = active_consent_text(
        &pool,
        &clock,
        ConsentType::PatientDataProcessing,
        DraftPolicy::Allow,
    )
    .await
    .unwrap()
    .unwrap();
    // Butir milik versi lain.
    let other_version = publish_version(
        &pool,
        datetime!(2096-01-01 00:00:00 UTC),
        &valid_content("lain"),
        true,
    )
    .await;
    let foreign_id = sqlx::query_scalar!(
        "SELECT id FROM app.consent_item WHERE consent_text_version_id = $1",
        other_version
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    let mut items = text.items.clone();
    let mut foreign = items[0].clone();
    foreign.id = foreign_id;
    items.push(foreign);
    let (patient_id, account_id, owner_recipient) = make_patient(&pool).await;
    let ctx = AuditContext::new(
        CorrelationId::new(),
        Source::component(SourceComponent::PatientApp).ip("198.51.100.7".parse().unwrap()),
    );
    let mut tx = pool.begin().await.unwrap();
    let r = record_granted_consent(
        &mut tx,
        ConsentSubject::Patient {
            patient_id,
            account_id,
            owner_care_recipient_id: owner_recipient,
        },
        text.version_id,
        &items,
        &ctx,
        new_id(),
    )
    .await;
    assert!(matches!(
        r,
        Err(PrivacyError::Internal("consent_item_foreign"))
    ));
    tx.commit().await.unwrap();
    let n = sqlx::query_scalar!(
        "SELECT count(*) FROM app.consent_record WHERE patient_id = $1",
        patient_id
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(n, Some(0));
}
