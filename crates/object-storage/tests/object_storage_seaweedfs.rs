//! Test kontrak object storage terhadap emulator SeaweedFS (KEPUTUSAN-233, KEPUTUSAN-235). Rangkaian yang
//! sama kelak dijalankan terhadap NEO nyata (spike F1-T03 §5); hasil di sini membuktikan kode kita, bukan
//! perilaku NEO.
#![allow(clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::{SeaweedFs, now, random_hex, upload_post, upload_post_with, upload_put};
use object_storage::{GetOutcome, ObjectStorageError, ObjectStore};
use std::time::Duration;

const TTL: Duration = Duration::from_secs(600);

#[tokio::test(flavor = "multi_thread")]
async fn presigned_post_enforces_key_type_size_and_expiry() {
    let sw = SeaweedFs::start().await;
    let store = sw.store();
    let bucket = sw.versioned_bucket("rs-uji-post").await;
    let key = foundation::new_id().to_string();
    let p = store.presigner();

    // Sesuai policy: diterima (204), ukuran tepat batas.
    let post = p
        .presign_post(&bucket, &key, "image/jpeg", 1000, now(), TTL)
        .unwrap();
    assert_eq!(upload_post(&post, &[7u8; 1000]), 204);
    assert_eq!(
        store.head_object(&bucket, &key, now()).await.unwrap(),
        Some(1000)
    );

    // Melebihi content-length-range: ditolak penyedia.
    let key2 = foundation::new_id().to_string();
    let post = p
        .presign_post(&bucket, &key2, "image/jpeg", 1000, now(), TTL)
        .unwrap();
    assert_eq!(upload_post(&post, &[7u8; 1001]), 400);
    assert_eq!(
        store.head_object(&bucket, &key2, now()).await.unwrap(),
        None
    );

    // Content-Type di luar policy: ditolak.
    let code = upload_post_with(&post, &[7u8; 10], &[("Content-Type", "text/html")]);
    assert!(code == 403 || code == 400, "kode {code}");
    // Kunci lain dari policy: ditolak.
    let code = upload_post_with(&post, &[7u8; 10], &[("key", "kunci-lain")]);
    assert!(code == 403 || code == 400, "kode {code}");
    assert_eq!(
        store
            .head_object(&bucket, "kunci-lain", now())
            .await
            .unwrap(),
        None
    );

    // Policy kedaluwarsa (dibuat satu jam lalu, umur 10 menit): ditolak.
    let past = now() - time::Duration::hours(1);
    let post = p
        .presign_post(&bucket, &key2, "image/jpeg", 1000, past, TTL)
        .unwrap();
    let code = upload_post(&post, &[7u8; 10]);
    assert!(code == 403 || code == 400, "kode {code}");
    assert_eq!(
        store.head_object(&bucket, &key2, now()).await.unwrap(),
        None
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn presigned_put_binds_content_type_length_and_expiry() {
    let sw = SeaweedFs::start().await;
    let store = sw.store();
    let bucket = sw.versioned_bucket("rs-uji-put").await;
    let key = foundation::new_id().to_string();
    let p = store.presigner();

    let put = p
        .presign_put(&bucket, &key, "image/png", 100, now(), TTL)
        .unwrap();
    // Tipe berbeda dari yang ditandatangani.
    assert_eq!(upload_put(&put, &[1u8; 100], Some("text/html")), 403);
    // Ukuran berbeda dari yang ditandatangani.
    assert_eq!(upload_put(&put, &[1u8; 101], None), 403);
    assert_eq!(store.head_object(&bucket, &key, now()).await.unwrap(), None);
    // Sesuai.
    assert_eq!(upload_put(&put, &[1u8; 100], None), 200);
    assert_eq!(
        store.head_object(&bucket, &key, now()).await.unwrap(),
        Some(100)
    );

    // Kedaluwarsa.
    let past = now() - time::Duration::hours(1);
    let key2 = foundation::new_id().to_string();
    let put = p
        .presign_put(&bucket, &key2, "image/png", 100, past, TTL)
        .unwrap();
    assert_eq!(upload_put(&put, &[1u8; 100], None), 403);
    assert_eq!(
        store.head_object(&bucket, &key2, now()).await.unwrap(),
        None
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn server_operations_limit_reads_and_purge_every_version() {
    let sw = SeaweedFs::start().await;
    let store = sw.store();
    let bucket = sw.versioned_bucket("rs-uji-ops").await;
    let key = foundation::new_id().to_string();

    // Dua versi (versi objek aktif).
    store
        .put_object(&bucket, &key, "image/png", vec![1u8; 10], now())
        .await
        .unwrap();
    store
        .put_object(&bucket, &key, "image/png", vec![2u8; 20], now())
        .await
        .unwrap();
    assert_eq!(
        store
            .list_versions(&bucket, &key, now())
            .await
            .unwrap()
            .len(),
        2
    );

    // GET terbatas: tepat batas diterima, lebih besar ditolak.
    assert_eq!(
        store.get_object(&bucket, &key, 20, now()).await.unwrap(),
        GetOutcome::Body(vec![2u8; 20])
    );
    assert_eq!(
        store.get_object(&bucket, &key, 19, now()).await.unwrap(),
        GetOutcome::TooLarge
    );
    assert_eq!(
        store
            .get_object(&bucket, "tidak-ada", 19, now())
            .await
            .unwrap(),
        GetOutcome::NotFound
    );

    // Kunci lain dengan prefiks sama tidak ikut terhapus.
    let sibling = format!("{key}-lain");
    store
        .put_object(&bucket, &sibling, "image/png", vec![3u8; 5], now())
        .await
        .unwrap();

    store
        .delete_all_versions(&bucket, &key, now())
        .await
        .unwrap();
    assert!(
        store
            .list_versions(&bucket, &key, now())
            .await
            .unwrap()
            .is_empty()
    );
    assert_eq!(store.head_object(&bucket, &key, now()).await.unwrap(), None);
    assert_eq!(
        store.head_object(&bucket, &sibling, now()).await.unwrap(),
        Some(5)
    );
    // Idempoten.
    store
        .delete_all_versions(&bucket, &key, now())
        .await
        .unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn wrong_secret_is_rejected_by_provider() {
    let sw = SeaweedFs::start().await;
    let bucket = sw.versioned_bucket("rs-uji-auth").await;
    let bad = ObjectStore::new(&sw.config_with_secret(&random_hex(20))).unwrap();
    match bad.head_object(&bucket, "k", now()).await {
        Err(ObjectStorageError::Rejected { status }) => assert_eq!(status, 403),
        other => panic!("harus ditolak 403: {other:?}"),
    }
}
