//! TD-17 (review F1-T29, data KTP; ADR-018 butir 2) terhadap PostgreSQL 18 (role runtime `rs_app`) dan
//! SeaweedFS: slot sebagai tombstone sampai `expires_at` + jeda, job pembersih yang menghapus semua versi
//! objek mentah setelah URL kedaluwarsa, prefiks `incoming/` vs `files/`, rekonsiliasi bucket terhadap
//! `stored_file`, penjaga `ValidatedUpload`, dan validasi bersamaan atas satu slot.
//!
//! Pembersih bekerja atas seluruh tabel `upload_slot`, sehingga test di berkas ini yang memundurkan
//! `expires_at` slot dijalankan berurutan lewat [`SERIAL`] (test di biner lain berjalan di proses
//! terpisah, berurutan). Jam aplikasi tidak dimajukan karena penyedia menolak tanda tangan SigV4 yang
//! waktunya terpaut jauh dari jam penyedia (diamati 403 pada ±25 menit, SRC-562).
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)] // berkas test

mod files;

use files::{
    SeaweedFs, app_pool, expire_slot_past_grace, jpeg, keys_under, new_account, now, png,
    slot_consumed, slot_exists, slot_object_key, upload,
};
use foundation::{FixedClock, SystemClock, Uuid, new_id};
use object_storage::GetOutcome;
use observability::LogLevel;
use observability::log::{Component, JsonLogLayer, LogSink, MemorySink, subscriber};
use platform_jobs::{Backoff, WorkerConfig, WorkerId, run_batch};
use rs_backend::modules::platform::api::{
    FILE_RECONCILE_JOB, FileError, FileStorage, LogicalBucket, UPLOAD_SLOT_PURGE_JOB,
    UPLOAD_SLOT_TOMBSTONE_GRACE, UploadMethod, UploadPurpose, UploadSlotRequest,
    create_upload_slot, enqueue_file_maintenance, purge_expired_upload_slots, reconcile_bucket,
    record_stored_file, run_file_maintenance_job, validate_upload,
};
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;

const TTL: Duration = Duration::from_secs(600);
static SERIAL: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

fn request(owner: Uuid, content_type: &str, size: usize) -> UploadSlotRequest {
    UploadSlotRequest {
        owner_account_id: owner,
        purpose: UploadPurpose::KtpPhoto,
        content_type: content_type.to_string(),
        size_bytes: size as u64,
    }
}

fn bucket(storage: &FileStorage) -> String {
    storage.bucket_name(LogicalBucket::IdentityDocument)
}

async fn versions(storage: &FileStorage, key: &str) -> usize {
    storage
        .object_store()
        .list_versions(&bucket(storage), key, now())
        .await
        .unwrap()
        .len()
}

async fn put(storage: &FileStorage, key: &str, bytes: &[u8]) {
    storage
        .object_store()
        .put_object(&bucket(storage), key, "image/jpeg", bytes.to_vec(), now())
        .await
        .unwrap();
}

/// Slot dibuat lewat batas pembersihan, lalu pembersih dijalankan dengan jam sungguhan.
async fn purge_after_grace(
    pool: &PgPool,
    storage: &FileStorage,
    slot_id: Uuid,
) -> rs_backend::modules::platform::api::SlotPurgeReport {
    assert!(UPLOAD_SLOT_TOMBSTONE_GRACE < Duration::from_secs(16 * 60));
    expire_slot_past_grace(pool, slot_id).await;
    purge_expired_upload_slots(pool, &SystemClock, storage, 1000)
        .await
        .unwrap()
}

async fn validate(
    pool: &PgPool,
    storage: &FileStorage,
    owner: Uuid,
    slot_id: Uuid,
) -> Result<rs_backend::modules::platform::api::ValidatedUpload, FileError> {
    validate_upload(
        pool,
        &SystemClock,
        storage,
        owner,
        slot_id,
        UploadPurpose::KtpPhoto,
    )
    .await
}

// trace: TD-17 (URL POST/PUT dipakai ulang setelah slot dikonsumsi → objek mentah dibersihkan setelah
// URL kedaluwarsa + jeda; berkas tercatat tidak tersentuh)
#[tokio::test(flavor = "multi_thread")]
async fn reupload_after_consumption_is_purged_after_expiry_and_grace() {
    let _serial = SERIAL.lock().await;
    let pool = app_pool().await;
    for method in [UploadMethod::Post, UploadMethod::Put] {
        // Satu SeaweedFS per jalur: unggahan PUT pertama ke bucket kedua di server yang sama diamati
        // ditolak HTTP 500 oleh emulator (penyebab tidak diselidiki; bukan perilaku yang diuji di sini).
        let sw = SeaweedFs::start().await;
        let storage = sw.file_storage(method, TTL).await;
        let owner = new_account(&pool).await;
        let bytes = jpeg(8, 8);
        let ticket = create_upload_slot(
            &pool,
            &SystemClock,
            &storage,
            request(owner, "image/jpeg", bytes.len()),
        )
        .await
        .unwrap();
        let code = upload(&ticket.instruction, &bytes);
        assert!(code == 200 || code == 204, "{method:?}: HTTP {code}");
        let raw_key = slot_object_key(&pool, ticket.upload_slot_id).await;
        let validated = validate(&pool, &storage, owner, ticket.upload_slot_id)
            .await
            .unwrap();
        let file_key = validated.object_key().to_string();
        let mut tx = pool.begin().await.unwrap();
        record_stored_file(&mut tx, &SystemClock, &validated, None)
            .await
            .unwrap();
        tx.commit().await.unwrap();
        assert!(slot_consumed(&pool, ticket.upload_slot_id).await);
        assert_eq!(versions(&storage, &raw_key).await, 0);

        // URL masih berlaku: unggah ulang diterima penyedia, tetapi slot tombstone tidak dapat dipakai.
        let code = upload(&ticket.instruction, &bytes);
        assert!(
            code == 200 || code == 204,
            "{method:?}: unggah ulang HTTP {code}"
        );
        assert_eq!(versions(&storage, &raw_key).await, 1);
        assert!(matches!(
            validate(&pool, &storage, owner, ticket.upload_slot_id).await,
            Err(FileError::NotFound)
        ));

        // Sebelum batas pembersihan: tidak ada yang dihapus.
        purge_expired_upload_slots(&pool, &SystemClock, &storage, 1000)
            .await
            .unwrap();
        assert!(slot_exists(&pool, ticket.upload_slot_id).await);
        assert_eq!(versions(&storage, &raw_key).await, 1);

        // Setelah expires_at + jeda: semua versi objek mentah dan baris slot hilang; berkas tercatat tetap.
        let report = purge_after_grace(&pool, &storage, ticket.upload_slot_id).await;
        assert!(report.purged >= 1, "{report:?}");
        assert!(!slot_exists(&pool, ticket.upload_slot_id).await);
        assert_eq!(versions(&storage, &raw_key).await, 0);
        assert_eq!(versions(&storage, &file_key).await, 1);
    }
}

// trace: TD-17 (slot berkas ditolak menjadi tombstone; unggahan ulang lewat URL lama tidak divalidasi dan
// dibersihkan pembersih)
#[tokio::test(flavor = "multi_thread")]
async fn rejected_slot_is_a_tombstone_until_purged() {
    let _serial = SERIAL.lock().await;
    let pool = app_pool().await;
    let sw = SeaweedFs::start().await;
    let storage = sw.file_storage(UploadMethod::Post, TTL).await;
    let owner = new_account(&pool).await;
    let fake = b"<!DOCTYPE html><html><script>alert(1)</script></html>".to_vec();
    let ticket = create_upload_slot(
        &pool,
        &SystemClock,
        &storage,
        request(owner, "image/jpeg", fake.len()),
    )
    .await
    .unwrap();
    assert_eq!(upload(&ticket.instruction, &fake), 204);
    let raw_key = slot_object_key(&pool, ticket.upload_slot_id).await;
    assert!(matches!(
        validate(&pool, &storage, owner, ticket.upload_slot_id).await,
        Err(FileError::UnsupportedType)
    ));
    assert!(slot_consumed(&pool, ticket.upload_slot_id).await);
    assert_eq!(versions(&storage, &raw_key).await, 0);

    // Unggah ulang (kali ini JPEG sah) lewat URL yang masih berlaku: tetap tidak dapat divalidasi.
    let mut real = jpeg(8, 8);
    real.resize(fake.len().max(real.len()), 0);
    assert_eq!(upload(&ticket.instruction, &real), 204);
    assert!(matches!(
        validate(&pool, &storage, owner, ticket.upload_slot_id).await,
        Err(FileError::NotFound)
    ));
    let report = purge_after_grace(&pool, &storage, ticket.upload_slot_id).await;
    assert_eq!(report.failed, 0, "{report:?}");
    assert!(!slot_exists(&pool, ticket.upload_slot_id).await);
    assert_eq!(versions(&storage, &raw_key).await, 0);
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
        Arc::new(app.clone()) as Arc<dyn LogSink>,
        Arc::new(sec) as Arc<dyn LogSink>,
    );
    let guard = tracing::subscriber::set_default(subscriber(layer));
    (app, guard)
}

// trace: TD-17 (ValidatedUpload dibuang tanpa discard_validated: peringatan + hasil kode ulang dihapus
// pembersih; juga mewakili proses mati setelah put hasil kode ulang)
#[tokio::test(flavor = "current_thread")]
async fn dropped_validated_upload_is_logged_and_purged() {
    let _serial = SERIAL.lock().await;
    let pool = app_pool().await;
    let sw = SeaweedFs::start().await;
    let storage = sw.file_storage(UploadMethod::Post, TTL).await;
    let owner = new_account(&pool).await;
    let bytes = png(6, 6);
    let ticket = create_upload_slot(
        &pool,
        &SystemClock,
        &storage,
        request(owner, "image/png", bytes.len()),
    )
    .await
    .unwrap();
    assert_eq!(upload(&ticket.instruction, &bytes), 204);
    let validated = validate(&pool, &storage, owner, ticket.upload_slot_id)
        .await
        .unwrap();
    let file_key = validated.object_key().to_string();
    let (logs, guard) = capture();
    drop(validated);
    drop(guard);
    let text = logs.text();
    assert!(text.contains("validated_upload_dropped"), "{text}");
    assert!(!text.contains(&file_key), "kunci objek tidak boleh dicatat");
    assert_eq!(versions(&storage, &file_key).await, 1);
    // Slot masih mengklaim hasil kode ulang: validasi ulang ditolak, pembersih menghapusnya.
    assert!(matches!(
        validate(&pool, &storage, owner, ticket.upload_slot_id).await,
        Err(FileError::NotFound)
    ));
    let report = purge_after_grace(&pool, &storage, ticket.upload_slot_id).await;
    assert_eq!(report.failed, 0, "{report:?}");
    assert_eq!(versions(&storage, &file_key).await, 0);
    assert!(!slot_exists(&pool, ticket.upload_slot_id).await);
}

// trace: TD-17 (validasi bersamaan atas satu slot: hanya satu hasil kode ulang yang tercatat dan tersimpan)
#[tokio::test(flavor = "multi_thread")]
async fn concurrent_validations_produce_a_single_file() {
    let pool = app_pool().await;
    let sw = SeaweedFs::start().await;
    let storage = sw.file_storage(UploadMethod::Post, TTL).await;
    let owner = new_account(&pool).await;
    let bytes = jpeg(64, 64);
    let ticket = create_upload_slot(
        &pool,
        &SystemClock,
        &storage,
        request(owner, "image/jpeg", bytes.len()),
    )
    .await
    .unwrap();
    assert_eq!(upload(&ticket.instruction, &bytes), 204);
    let (a, b) = tokio::join!(
        validate(&pool, &storage, owner, ticket.upload_slot_id),
        validate(&pool, &storage, owner, ticket.upload_slot_id)
    );
    let mut ok = Vec::new();
    for r in [a, b] {
        match r {
            Ok(v) => ok.push(v),
            // Pemenang sudah menghapus berkas mentah (422) atau sudah mengklaim slot (404).
            Err(FileError::NotFound) | Err(FileError::Validation(_)) => {}
            Err(e) => panic!("galat tak terduga: {e:?}"),
        }
    }
    assert_eq!(ok.len(), 1, "tepat satu validasi menang");
    let files = keys_under(&storage, "files/").await;
    assert_eq!(files, vec![ok[0].object_key().to_string()]);
    let winner = ok.pop().unwrap();
    let mut tx = pool.begin().await.unwrap();
    record_stored_file(&mut tx, &SystemClock, &winner, None)
        .await
        .unwrap();
    tx.commit().await.unwrap();
}

// trace: TD-17 (rekonsiliasi berkala bucket vs stored_file: objek yatim dihapus semua versinya, objek sah dan
// di luar prefiks tidak tersentuh, objek hilang dilaporkan)
#[tokio::test(flavor = "multi_thread")]
async fn reconciliation_deletes_orphans_and_reports_missing_objects() {
    let pool = app_pool().await;
    let sw = SeaweedFs::start().await;
    let storage = sw.file_storage(UploadMethod::Post, TTL).await;
    let owner = new_account(&pool).await;

    // Sah: slot yang menunggu unggahan (incoming/), berkas tercatat (files/).
    let bytes = jpeg(8, 8);
    let pending = create_upload_slot(
        &pool,
        &SystemClock,
        &storage,
        request(owner, "image/jpeg", bytes.len()),
    )
    .await
    .unwrap();
    assert_eq!(upload(&pending.instruction, &bytes), 204);
    let pending_key = slot_object_key(&pool, pending.upload_slot_id).await;
    let recorded = create_upload_slot(
        &pool,
        &SystemClock,
        &storage,
        request(owner, "image/jpeg", bytes.len()),
    )
    .await
    .unwrap();
    assert_eq!(upload(&recorded.instruction, &bytes), 204);
    let validated = validate(&pool, &storage, owner, recorded.upload_slot_id)
        .await
        .unwrap();
    let recorded_key = validated.object_key().to_string();
    let mut tx = pool.begin().await.unwrap();
    let recorded_id = record_stored_file(&mut tx, &SystemClock, &validated, None)
        .await
        .unwrap();
    tx.commit().await.unwrap();

    // Yatim: tanpa slot / tanpa stored_file, masing-masing dua versi; di luar prefiks tidak disentuh.
    let orphan_in = format!("incoming/{}", new_id());
    let orphan_file = format!("files/{}", new_id());
    let outside = format!("lain/{}", new_id());
    for k in [&orphan_in, &orphan_file] {
        put(&storage, k, b"versi-1").await;
        put(&storage, k, b"versi-2").await;
        assert_eq!(versions(&storage, k).await, 2);
    }
    put(&storage, &outside, b"x").await;

    let report = reconcile_bucket(
        &pool,
        &SystemClock,
        &storage,
        LogicalBucket::IdentityDocument,
    )
    .await
    .unwrap();
    assert_eq!(report.orphans_deleted, 2, "{report:?}");
    assert_eq!(report.orphans_failed, 0);
    assert_eq!(versions(&storage, &orphan_in).await, 0);
    assert_eq!(versions(&storage, &orphan_file).await, 0);
    assert_eq!(versions(&storage, &pending_key).await, 1);
    assert_eq!(versions(&storage, &recorded_key).await, 1);
    assert_eq!(versions(&storage, &outside).await, 1);

    // Objek berkas tercatat hilang: dilaporkan (tanpa mengubah data). Baris stored_file lain di database
    // (dari test lain) yang bucket fisiknya tidak ada di penyimpanan ini juga terhitung, jadi minimal satu.
    storage
        .object_store()
        .delete_all_versions(&bucket(&storage), &recorded_key, now())
        .await
        .unwrap();
    let report = reconcile_bucket(
        &pool,
        &SystemClock,
        &storage,
        LogicalBucket::IdentityDocument,
    )
    .await
    .unwrap();
    assert!(report.missing_objects >= 1, "{report:?}");
    assert_eq!(report.orphans_deleted, 0);
    let row_count: i64 = sqlx::query_scalar!(
        r#"SELECT count(*) AS "n!" FROM app.stored_file WHERE id = $1 AND purged_at IS NULL"#,
        recorded_id
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(row_count, 1, "rekonsiliasi tidak mengubah stored_file");
    assert!(matches!(
        storage
            .object_store()
            .get_object(&bucket(&storage), &pending_key, 1024, now())
            .await
            .unwrap(),
        GetOutcome::Body(_)
    ));
}

async fn job_count(pool: &PgPool, types: &[&str]) -> i64 {
    let types: Vec<String> = types.iter().map(|t| t.to_string()).collect();
    sqlx::query_scalar!(
        r#"SELECT count(*) AS "n!" FROM app.job WHERE job_type = ANY($1) AND status = 'queued'"#,
        &types
    )
    .fetch_one(pool)
    .await
    .unwrap()
}

// trace: TD-17 (job pembersih dan rekonsiliasi lewat platform-jobs: deduplikasi antrean, dijalankan run_batch)
#[tokio::test(flavor = "multi_thread")]
async fn maintenance_jobs_are_deduplicated_and_run_by_the_job_runner() {
    let _serial = SERIAL.lock().await;
    let pool = app_pool().await;
    let sw = SeaweedFs::start().await;
    let storage = sw.file_storage(UploadMethod::Post, TTL).await;
    sw.create_other_buckets(&storage).await;
    let types = [UPLOAD_SLOT_PURGE_JOB, FILE_RECONCILE_JOB];
    let clock = FixedClock::new(now());
    enqueue_file_maintenance(&pool, &clock).await.unwrap();
    enqueue_file_maintenance(&pool, &clock).await.unwrap();
    assert_eq!(
        job_count(&pool, &types).await,
        1 + i64::try_from(LogicalBucket::ALL.len()).unwrap()
    );

    let orphan = format!("incoming/{}", new_id());
    put(&storage, &orphan, b"x").await;
    let cfg = WorkerConfig::new(
        WorkerId::new("worker").unwrap(),
        types
            .iter()
            .map(|t| platform_jobs::JobType::new(t).unwrap())
            .collect(),
        50,
        time::Duration::seconds(120),
        Backoff::new(time::Duration::seconds(1), time::Duration::seconds(10)).unwrap(),
    )
    .unwrap();
    let (p, c, s) = (&pool, &clock, &storage);
    let report = run_batch(&pool, &cfg, &clock, move |job| async move {
        run_file_maintenance_job(p, c, s, &job).await
    })
    .await
    .unwrap();
    assert_eq!(report.claimed, 8, "{report:?}");
    assert_eq!(report.succeeded, 8, "{report:?}");
    assert_eq!(job_count(&pool, &types).await, 0);
    assert_eq!(versions(&storage, &orphan).await, 0);
}
