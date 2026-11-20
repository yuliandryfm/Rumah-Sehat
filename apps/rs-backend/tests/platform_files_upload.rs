//! Modul `platform` berkas (F1-T29, ADR-018, NFR-SEC-08, US-106 AC3) terhadap PostgreSQL 18 (role runtime
//! `rs_app`) dan SeaweedFS (KEPUTUSAN-233): slot unggah, URL bertanda tangan, validasi isi, kode ulang,
//! `stored_file`, dan skenario negatif modul T1 "Berkas dokumen identitas dan klinis"
//! (coding-standards §7.3): jenis palsu, ukuran, slot kedaluwarsa, slot milik akun lain, metadata terbuang.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)] // berkas test

mod files;

use files::{
    METADATA_MARKER, SeaweedFs, app_pool, contains, expire_slot_past_grace, jpeg, jpeg_with_exif,
    new_account, now, png, png_chunk_types, png_with_metadata, slot_consumed, slot_exists,
    slot_object_key, stored_file, upload,
};
use foundation::{Clock, FixedClock, SystemClock, Uuid, new_id};
use image::{GenericImageView, ImageFormat};
use object_storage::GetOutcome;
use rs_backend::modules::platform::api::{
    ContainerType, FileContainer, FileError, FileStorage, LogicalBucket, MAX_PHOTO_BYTES,
    UPLOAD_SLOT_TOMBSTONE_GRACE, UploadMethod, UploadPurpose, UploadSlotRequest,
    create_upload_slot, discard_validated, purge_expired_upload_slots, record_stored_file,
    validate_upload,
};
use sqlx::PgPool;
use std::time::Duration;

const TTL: Duration = Duration::from_secs(600);

fn request(owner: Uuid, content_type: &str, size: usize) -> UploadSlotRequest {
    UploadSlotRequest {
        owner_account_id: owner,
        purpose: UploadPurpose::KtpPhoto,
        content_type: content_type.to_string(),
        size_bytes: size as u64,
    }
}

async fn get(storage: &FileStorage, key: &str) -> GetOutcome {
    storage
        .object_store()
        .get_object(
            &storage.bucket_name(LogicalBucket::IdentityDocument),
            key,
            16 * 1024 * 1024,
            now(),
        )
        .await
        .unwrap()
}

async fn versions(storage: &FileStorage, key: &str) -> usize {
    storage
        .object_store()
        .list_versions(
            &storage.bucket_name(LogicalBucket::IdentityDocument),
            key,
            now(),
        )
        .await
        .unwrap()
        .len()
}

/// Slot + unggah lewat instruksi; mengembalikan (ID slot, kunci objek asli).
async fn slot_with_upload(
    pool: &PgPool,
    storage: &FileStorage,
    owner: Uuid,
    content_type: &str,
    bytes: &[u8],
) -> (Uuid, String) {
    let ticket = create_upload_slot(
        pool,
        &SystemClock,
        storage,
        request(owner, content_type, bytes.len()),
    )
    .await
    .unwrap();
    let code = upload(&ticket.instruction, bytes);
    assert!(code == 204 || code == 200, "unggah gagal: HTTP {code}");
    let key = slot_object_key(pool, ticket.upload_slot_id).await;
    (ticket.upload_slot_id, key)
}

// trace: US-106 AC3 (JPG menurut isi diterima), ADR-018 butir 2 (metadata terbuang, berkas asli dihapus)
#[tokio::test(flavor = "multi_thread")]
async fn jpeg_via_presigned_post_is_reencoded_without_exif_and_recorded_once() {
    let pool = app_pool().await;
    let sw = SeaweedFs::start().await;
    let storage = sw.file_storage(UploadMethod::Post, TTL).await;
    let owner = new_account(&pool).await;

    let original = jpeg_with_exif(4, 2);
    assert!(contains(&original, b"Exif\0\0") && contains(&original, METADATA_MARKER));
    let (slot_id, original_key) =
        slot_with_upload(&pool, &storage, owner, "image/jpeg", &original).await;

    let validated = validate_upload(
        &pool,
        &SystemClock,
        &storage,
        owner,
        slot_id,
        UploadPurpose::KtpPhoto,
    )
    .await
    .unwrap();
    assert_eq!(validated.mime_type(), "image/jpeg");
    assert_ne!(validated.object_key(), original_key);
    // Prefiks TD-17: unggahan mentah di incoming/, hasil kode ulang di files/.
    assert!(original_key.starts_with("incoming/"));
    assert!(validated.object_key().starts_with("files/"));

    // Berkas asli beserta semua versinya hilang; hasil kode ulang tanpa EXIF dan penanda, orientasi diterapkan.
    assert_eq!(versions(&storage, &original_key).await, 0);
    let GetOutcome::Body(stored) = get(&storage, validated.object_key()).await else {
        panic!("berkas hasil kode ulang tidak ada");
    };
    assert!(!contains(&stored, b"Exif"));
    assert!(!contains(&stored, METADATA_MARKER));
    let decoded = image::load_from_memory_with_format(&stored, ImageFormat::Jpeg).unwrap();
    assert_eq!(
        decoded.dimensions(),
        (2, 4),
        "orientasi EXIF 6 diterapkan ke piksel"
    );
    assert_eq!(&crypto::sha256(&stored), validated.sha256());
    assert_eq!(stored.len(), validated.size_bytes() as usize);

    // Dicatat di transaksi pemanggil; slot terpakai.
    let container = FileContainer {
        container_type: ContainerType::IdentityDocument,
        container_id: new_id(),
    };
    let mut tx = pool.begin().await.unwrap();
    let file_id = record_stored_file(&mut tx, &SystemClock, &validated, Some(container))
        .await
        .unwrap();
    tx.commit().await.unwrap();
    let row = stored_file(&pool, file_id).await;
    assert_eq!(row.bucket, "identity_document");
    assert_eq!(row.category, "ktp_photo");
    assert_eq!(row.object_key, validated.object_key());
    assert_eq!(row.owner_account_id, Some(owner));
    assert_eq!(row.created_by, Some(owner));
    assert_eq!(row.container_type.as_deref(), Some("identity_document"));
    assert_eq!(row.container_id, Some(container.container_id));
    assert_eq!(row.mime_type, "image/jpeg");
    assert_eq!(row.size_bytes, validated.size_bytes());
    assert_eq!(row.sha256, validated.sha256().to_vec());
    assert!(row.deleted_at.is_none());
    // TD-17: slot tetap ada sebagai tombstone sampai expires_at + jeda.
    assert!(slot_consumed(&pool, slot_id).await);

    // Slot sekali pakai: pencatatan dan validasi ulang ditolak.
    let mut tx = pool.begin().await.unwrap();
    assert!(matches!(
        record_stored_file(&mut tx, &SystemClock, &validated, None).await,
        Err(FileError::NotFound)
    ));
    drop(tx);
    assert!(matches!(
        validate_upload(
            &pool,
            &SystemClock,
            &storage,
            owner,
            slot_id,
            UploadPurpose::KtpPhoto
        )
        .await,
        Err(FileError::NotFound)
    ));
}

// trace: US-106 AC3 (PNG menurut isi diterima), KEPUTUSAN-234 jalur PUT bertanda tangan
#[tokio::test(flavor = "multi_thread")]
async fn png_via_presigned_put_loses_text_and_exif_chunks() {
    let pool = app_pool().await;
    let sw = SeaweedFs::start().await;
    let storage = sw.file_storage(UploadMethod::Put, TTL).await;
    let owner = new_account(&pool).await;

    let original = png_with_metadata(3, 5);
    let chunks = png_chunk_types(&original);
    assert!(chunks.contains(&"tEXt".to_string()) && chunks.contains(&"eXIf".to_string()));
    let (slot_id, original_key) =
        slot_with_upload(&pool, &storage, owner, "image/png", &original).await;
    let validated = validate_upload(
        &pool,
        &SystemClock,
        &storage,
        owner,
        slot_id,
        UploadPurpose::KtpPhoto,
    )
    .await
    .unwrap();
    assert_eq!(validated.mime_type(), "image/png");
    let GetOutcome::Body(stored) = get(&storage, validated.object_key()).await else {
        panic!("berkas hasil kode ulang tidak ada");
    };
    let chunks = png_chunk_types(&stored);
    for meta in ["tEXt", "zTXt", "iTXt", "eXIf", "iCCP", "tIME"] {
        assert!(
            !chunks.contains(&meta.to_string()),
            "chunk {meta} tersisa: {chunks:?}"
        );
    }
    assert!(!contains(&stored, METADATA_MARKER));
    assert_eq!(
        image::load_from_memory_with_format(&stored, ImageFormat::Png)
            .unwrap()
            .dimensions(),
        (5, 3),
        "orientasi eXIf 6 diterapkan"
    );
    assert_eq!(versions(&storage, &original_key).await, 0);

    // Transaksi pemanggil batal: berkas tervalidasi dibuang, slot tetap ada (rollback).
    let mut tx = pool.begin().await.unwrap();
    record_stored_file(&mut tx, &SystemClock, &validated, None)
        .await
        .unwrap();
    tx.rollback().await.unwrap();
    assert!(slot_exists(&pool, slot_id).await);
    assert!(!slot_consumed(&pool, slot_id).await);
    let key = validated.object_key().to_string();
    discard_validated(&SystemClock, &storage, validated)
        .await
        .unwrap();
    assert_eq!(versions(&storage, &key).await, 0);
}

// trace: US-106 AC3 (jenis menurut isi bukan JPG/PNG, termasuk ekstensi palsu → 4150001)
#[tokio::test(flavor = "multi_thread")]
async fn fake_types_are_rejected_and_purged() {
    let pool = app_pool().await;
    let sw = SeaweedFs::start().await;
    let storage = sw.file_storage(UploadMethod::Post, TTL).await;
    let owner = new_account(&pool).await;

    // Jenis yang dinyatakan di luar daftar izin ditolak sebelum slot dibuat.
    for ct in [
        "image/svg+xml",
        "text/html",
        "application/pdf",
        "image/gif",
        "image/JPEG",
    ] {
        assert!(
            matches!(
                create_upload_slot(&pool, &SystemClock, &storage, request(owner, ct, 100)).await,
                Err(FileError::UnsupportedType)
            ),
            "{ct}"
        );
    }

    let mut html_with_jpeg_magic = vec![0xFF, 0xD8, 0xFF, 0xE0];
    html_with_jpeg_magic.extend_from_slice(b"<html><body><script>alert(1)</script></body></html>");
    let mut svg_with_png_magic = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    svg_with_png_magic
        .extend_from_slice(b"<svg xmlns=\"http://www.w3.org/2000/svg\" onload=\"alert(1)\"/>");
    let cases: Vec<(&str, &str, Vec<u8>)> = vec![
        // PNG yang dinyatakan JPG (ekstensi/MIME klien palsu).
        ("png sebagai jpg", "image/jpeg", png(8, 8)),
        // JPG yang dinyatakan PNG.
        ("jpg sebagai png", "image/png", jpeg(8, 8)),
        // SVG dan HTML berganti nama.
        (
            "svg sebagai png",
            "image/png",
            b"<?xml version=\"1.0\"?><svg xmlns=\"http://www.w3.org/2000/svg\" onload=\"alert(1)\"/>".to_vec(),
        ),
        (
            "html sebagai jpg",
            "image/jpeg",
            b"<!DOCTYPE html><html><script>alert(1)</script></html>".to_vec(),
        ),
        // Polyglot: magic bytes benar tetapi isinya bukan gambar.
        ("polyglot html+jpeg magic", "image/jpeg", html_with_jpeg_magic),
        ("polyglot svg+png magic", "image/png", svg_with_png_magic),
    ];
    for (name, ct, bytes) in cases {
        let (slot_id, key) = slot_with_upload(&pool, &storage, owner, ct, &bytes).await;
        let r = validate_upload(
            &pool,
            &SystemClock,
            &storage,
            owner,
            slot_id,
            UploadPurpose::KtpPhoto,
        )
        .await;
        assert!(
            matches!(r, Err(FileError::UnsupportedType)),
            "{name}: {r:?}"
        );
        assert!(
            slot_consumed(&pool, slot_id).await,
            "{name}: slot menjadi tombstone"
        );
        assert_eq!(versions(&storage, &key).await, 0, "{name}: berkas dihapus");
    }
}

// trace: ADR-018 butir 2 (kode ulang membuang muatan di luar gambar)
#[tokio::test(flavor = "multi_thread")]
async fn polyglot_payload_appended_to_valid_image_is_neutralised() {
    let pool = app_pool().await;
    let sw = SeaweedFs::start().await;
    let storage = sw.file_storage(UploadMethod::Post, TTL).await;
    let owner = new_account(&pool).await;

    let html = b"<html><script>fetch('//x.invalid')</script></html>";
    let zip = b"PK\x03\x04\x14\x00\x00\x00\x08\x00";
    for (ct, mut bytes) in [("image/jpeg", jpeg(16, 16)), ("image/png", png(16, 16))] {
        bytes.extend_from_slice(html);
        bytes.extend_from_slice(zip);
        let (slot_id, _) = slot_with_upload(&pool, &storage, owner, ct, &bytes).await;
        let validated = validate_upload(
            &pool,
            &SystemClock,
            &storage,
            owner,
            slot_id,
            UploadPurpose::KtpPhoto,
        )
        .await
        .unwrap();
        let GetOutcome::Body(stored) = get(&storage, validated.object_key()).await else {
            panic!("berkas hasil kode ulang tidak ada");
        };
        assert!(!contains(&stored, b"<script"), "{ct}");
        assert!(!contains(&stored, b"PK\x03\x04"), "{ct}");
    }
}

/// JPEG sah berukuran persis `size` byte (sisa diisi nol setelah penanda akhir).
fn jpeg_of_size(size: usize) -> Vec<u8> {
    let mut b = jpeg(32, 32);
    assert!(b.len() < size);
    b.resize(size, 0);
    b
}

// trace: US-106 AC3 (5.242.880 byte diterima; 5.242.881 byte ditolak 4130001), NFR-SEC-08
#[tokio::test(flavor = "multi_thread")]
async fn size_limit_is_inclusive_and_enforced_at_slot_storage_and_validation() {
    let pool = app_pool().await;
    let sw = SeaweedFs::start().await;
    let storage = sw.file_storage(UploadMethod::Post, TTL).await;
    let owner = new_account(&pool).await;
    let max = MAX_PHOTO_BYTES as usize;
    assert_eq!(max, 5_242_880);

    // Tepat batas: diterima.
    let (slot_id, _) =
        slot_with_upload(&pool, &storage, owner, "image/jpeg", &jpeg_of_size(max)).await;
    let accepted = validate_upload(
        &pool,
        &SystemClock,
        &storage,
        owner,
        slot_id,
        UploadPurpose::KtpPhoto,
    )
    .await
    .unwrap();
    discard_validated(&SystemClock, &storage, accepted)
        .await
        .unwrap();

    // Satu byte lebih: ditolak saat meminta slot (ukuran yang dinyatakan).
    assert!(matches!(
        create_upload_slot(
            &pool,
            &SystemClock,
            &storage,
            request(owner, "image/jpeg", max + 1)
        )
        .await,
        Err(FileError::TooLarge)
    ));

    // Klien berbohong soal ukuran: policy POST menolak di penyedia (content-length-range).
    let over = jpeg_of_size(max + 1);
    let ticket = create_upload_slot(
        &pool,
        &SystemClock,
        &storage,
        request(owner, "image/jpeg", 1000),
    )
    .await
    .unwrap();
    assert_eq!(upload(&ticket.instruction, &over), 400);

    // Penyedia tidak menegakkan batas (misalnya objek ditulis di luar URL): validasi tetap menolak 413,
    // lalu objek dan slot dihapus.
    let key = slot_object_key(&pool, ticket.upload_slot_id).await;
    let bucket = storage.bucket_name(LogicalBucket::IdentityDocument);
    storage
        .object_store()
        .put_object(&bucket, &key, "image/jpeg", over, now())
        .await
        .unwrap();
    assert!(matches!(
        validate_upload(
            &pool,
            &SystemClock,
            &storage,
            owner,
            ticket.upload_slot_id,
            UploadPurpose::KtpPhoto
        )
        .await,
        Err(FileError::TooLarge)
    ));
    assert!(slot_consumed(&pool, ticket.upload_slot_id).await);
    assert_eq!(versions(&storage, &key).await, 0);
}

// trace: ADR-018 butir 2 (berkas tidak pernah dikonfirmasi dihapus), coding-standards §7.3 (slot
// kedaluwarsa), TD-17 (dibersihkan job pembersih setelah expires_at + jeda, bukan di jalur permintaan)
#[tokio::test(flavor = "multi_thread")]
async fn expired_slot_is_not_found_and_cleaned_up() {
    let pool = app_pool().await;
    let sw = SeaweedFs::start().await;
    let ttl = Duration::from_secs(60);
    let storage = sw.file_storage(UploadMethod::Post, ttl).await;
    let owner = new_account(&pool).await;

    let clock = FixedClock::new(now());
    let bytes = jpeg(8, 8);
    let ticket = create_upload_slot(
        &pool,
        &clock,
        &storage,
        request(owner, "image/jpeg", bytes.len()),
    )
    .await
    .unwrap();
    assert_eq!(ticket.expires_at, clock.now() + time::Duration::seconds(60));
    assert_eq!(upload(&ticket.instruction, &bytes), 204);
    let key = slot_object_key(&pool, ticket.upload_slot_id).await;

    assert!(clock.advance(time::Duration::seconds(61)));
    assert!(matches!(
        validate_upload(
            &pool,
            &clock,
            &storage,
            owner,
            ticket.upload_slot_id,
            UploadPurpose::KtpPhoto
        )
        .await,
        Err(FileError::NotFound)
    ));
    // Belum lewat jeda: pembersih tidak menyentuh slot ini.
    purge_expired_upload_slots(&pool, &clock, &storage, 1000)
        .await
        .unwrap();
    assert!(slot_exists(&pool, ticket.upload_slot_id).await);
    assert_eq!(versions(&storage, &key).await, 1);
    // Lewat expires_at + jeda: objek mentah dan slot dihapus. `expires_at` dimundurkan (bukan jam yang
    // dimajukan): penyedia menolak tanda tangan yang waktunya terpaut jauh (diamati 403, SRC-562).
    assert!(UPLOAD_SLOT_TOMBSTONE_GRACE < Duration::from_secs(16 * 60));
    expire_slot_past_grace(&pool, ticket.upload_slot_id).await;
    let report = purge_expired_upload_slots(&pool, &SystemClock, &storage, 1000)
        .await
        .unwrap();
    assert_eq!(report.failed, 0, "{report:?}");
    assert!(!slot_exists(&pool, ticket.upload_slot_id).await);
    assert_eq!(versions(&storage, &key).await, 0);

    // URL yang sudah kedaluwarsa ditolak penyedia (slot dibuat satu jam lalu).
    let past = FixedClock::new(now() - time::Duration::hours(1));
    let ticket = create_upload_slot(
        &pool,
        &past,
        &storage,
        request(owner, "image/jpeg", bytes.len()),
    )
    .await
    .unwrap();
    let code = upload(&ticket.instruction, &bytes);
    assert!(code == 403 || code == 400, "HTTP {code}");
}

// trace: KU-03, ADR-012 butir 4 (objek milik pengguna lain → 404), coding-standards §7.3
#[tokio::test(flavor = "multi_thread")]
async fn slot_of_another_account_is_not_found_and_left_intact() {
    let pool = app_pool().await;
    let sw = SeaweedFs::start().await;
    let storage = sw.file_storage(UploadMethod::Post, TTL).await;
    let owner = new_account(&pool).await;
    let intruder = new_account(&pool).await;

    let (slot_id, key) = slot_with_upload(&pool, &storage, owner, "image/jpeg", &jpeg(8, 8)).await;
    assert!(matches!(
        validate_upload(
            &pool,
            &SystemClock,
            &storage,
            intruder,
            slot_id,
            UploadPurpose::KtpPhoto
        )
        .await,
        Err(FileError::NotFound)
    ));
    // Slot ID acak yang tidak ada: jawaban sama.
    assert!(matches!(
        validate_upload(
            &pool,
            &SystemClock,
            &storage,
            intruder,
            new_id(),
            UploadPurpose::KtpPhoto
        )
        .await,
        Err(FileError::NotFound)
    ));
    // Slot dan berkas pemilik tidak tersentuh; pemilik tetap dapat memakainya.
    assert!(slot_exists(&pool, slot_id).await);
    assert_eq!(versions(&storage, &key).await, 1);
    let validated = validate_upload(
        &pool,
        &SystemClock,
        &storage,
        owner,
        slot_id,
        UploadPurpose::KtpPhoto,
    )
    .await
    .unwrap();
    let mut tx = pool.begin().await.unwrap();
    record_stored_file(&mut tx, &SystemClock, &validated, None)
        .await
        .unwrap();
    tx.commit().await.unwrap();
}

// trace: ADR-018 butir 2 (konfirmasi sebelum unggah tidak menghapus slot)
#[tokio::test(flavor = "multi_thread")]
async fn validating_before_upload_is_rejected_without_consuming_slot() {
    let pool = app_pool().await;
    let sw = SeaweedFs::start().await;
    let storage = sw.file_storage(UploadMethod::Post, TTL).await;
    let owner = new_account(&pool).await;
    let ticket = create_upload_slot(
        &pool,
        &SystemClock,
        &storage,
        request(owner, "image/png", 100),
    )
    .await
    .unwrap();
    match validate_upload(
        &pool,
        &SystemClock,
        &storage,
        owner,
        ticket.upload_slot_id,
        UploadPurpose::KtpPhoto,
    )
    .await
    {
        Err(FileError::Validation(errors)) => {
            assert_eq!(errors[0].field, "upload_slot_id");
            assert_eq!(errors[0].code, "not_uploaded");
        }
        other => panic!("harus 422 not_uploaded: {other:?}"),
    }
    assert!(slot_exists(&pool, ticket.upload_slot_id).await);
}
