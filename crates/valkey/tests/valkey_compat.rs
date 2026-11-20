//! Kompatibilitas crate `redis` 1.8.2 dengan Valkey 9.1.3 (F1-T33, docs/05-stack.md §1.1 "dibuktikan test
//! integrasi Tahap 8", §13 butir 21). Valkey sungguhan dari image `infra/local/compose.yaml`; tanpa Docker
//! test GAGAL. Mencakup setiap perintah yang dipakai modul identitas (GET, MGET, SET PX/NX, MULTI/EXEC,
//! INCR, DEL, PTTL, EVAL) dan perilaku gagal: password salah, server beku (habis waktu), server mati,
//! koneksi diputus.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)] // berkas test

mod common;

use common::{ValkeyContainer, dead_url};
use std::time::{Duration, Instant};
use valkey::{Valkey, ValkeyConfig, ValkeyErrorKind};

const OP: Duration = Duration::from_millis(250);

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn server_is_valkey_9_1_3() {
    let c = ValkeyContainer::start();
    let v = c.client(OP);
    v.ping().await.unwrap();
    assert_eq!(v.server_version().await.unwrap(), "9.1.3");
    // Entri pencabutan tidak boleh dibuang saat memori penuh (infra/local/compose.yaml).
    assert!(
        c.cli(&["CONFIG", "GET", "maxmemory-policy"])
            .contains("noeviction")
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn commands_used_by_identity_behave_as_expected() {
    let c = ValkeyContainer::start();
    let v = c.client(OP);

    // GET/SET PX dan TTL dibulatkan ke atas.
    assert_eq!(v.get("k:a").await.unwrap(), None);
    v.set("k:a", b"satu", Duration::from_secs(30))
        .await
        .unwrap();
    assert_eq!(v.get("k:a").await.unwrap(), Some(b"satu".to_vec()));
    let ttl = v.ttl("k:a").await.unwrap().unwrap();
    assert!(
        ttl > Duration::from_secs(29) && ttl <= Duration::from_secs(30),
        "{ttl:?}"
    );

    // SET NX: tidak menimpa.
    assert!(
        !v.set_if_absent("k:a", b"dua", Duration::from_secs(30))
            .await
            .unwrap()
    );
    assert_eq!(v.get("k:a").await.unwrap(), Some(b"satu".to_vec()));
    assert!(
        v.set_if_absent("k:b", b"dua", Duration::from_secs(30))
            .await
            .unwrap()
    );

    // MGET sesuai urutan, termasuk kunci yang tidak ada.
    assert_eq!(
        v.get_many(&["k:b", "k:tidak-ada", "k:a"]).await.unwrap(),
        vec![Some(b"dua".to_vec()), None, Some(b"satu".to_vec())]
    );

    // MULTI/EXEC banyak SET PX.
    let items: Vec<(String, Vec<u8>, Duration)> = (0..50)
        .map(|i| (format!("k:m:{i}"), b"1".to_vec(), Duration::from_secs(60)))
        .collect();
    v.set_many(&items).await.unwrap();
    let keys: Vec<String> = (0..50).map(|i| format!("k:m:{i}")).collect();
    let refs: Vec<&str> = keys.iter().map(String::as_str).collect();
    assert!(v.get_many(&refs).await.unwrap().iter().all(Option::is_some));

    // Kedaluwarsa PX.
    v.set("k:pendek", b"x", Duration::from_millis(50))
        .await
        .unwrap();
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert_eq!(v.get("k:pendek").await.unwrap(), None);

    // DEL dan penanda tanpa TTL.
    v.delete("k:a").await.unwrap();
    assert_eq!(v.get("k:a").await.unwrap(), None);
    v.set_persistent("k:p", b"1").await.unwrap();
    assert_eq!(v.ttl("k:p").await.unwrap(), None);

    // INCR dan EVAL bersyarat (penanda hanya dipasang bila penghitung tidak berubah).
    assert!(
        v.set_persistent_if_unchanged("k:loaded", b"1", "k:epoch", None)
            .await
            .unwrap()
    );
    assert_eq!(v.incr("k:epoch").await.unwrap(), 1);
    v.delete("k:loaded").await.unwrap();
    assert!(
        !v.set_persistent_if_unchanged("k:loaded", b"1", "k:epoch", None)
            .await
            .unwrap()
    );
    assert_eq!(v.get("k:loaded").await.unwrap(), None);
    assert!(
        !v.set_persistent_if_unchanged("k:loaded", b"1", "k:epoch", Some(b"0"))
            .await
            .unwrap()
    );
    assert!(
        v.set_persistent_if_unchanged("k:loaded", b"1", "k:epoch", Some(b"1"))
            .await
            .unwrap()
    );
    assert_eq!(v.get("k:loaded").await.unwrap(), Some(b"1".to_vec()));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn wrong_password_is_rejected() {
    let c = ValkeyContainer::start();
    let v = Valkey::connect_lazy(&ValkeyConfig {
        url: c.url_with_password("salah"),
        op_timeout: OP,
    })
    .unwrap();
    let e = v.get("k").await.unwrap_err();
    assert_ne!(e.kind(), ValkeyErrorKind::Config, "{e}");
    // Galat tidak membawa password atau pesan server.
    assert!(!format!("{e} {e:?}").contains("salah"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn frozen_server_times_out_within_bound_then_recovers() {
    let c = ValkeyContainer::start();
    let v = c.client(OP);
    v.ping().await.unwrap();
    c.pause();
    let started = Instant::now();
    let e = v.get("k").await.unwrap_err();
    let elapsed = started.elapsed();
    c.unpause();
    assert_eq!(e.kind(), ValkeyErrorKind::Timeout);
    assert!(elapsed < OP + Duration::from_millis(500), "{elapsed:?}");
    let mut ok = false;
    for _ in 0..20 {
        if v.ping().await.is_ok() {
            ok = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    assert!(ok, "klien tidak pulih setelah server aktif kembali");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn dead_server_fails_fast_and_lazy_client_creation_does_not_fail() {
    let v = Valkey::connect_lazy(&ValkeyConfig {
        url: dead_url(),
        op_timeout: OP,
    })
    .unwrap();
    let started = Instant::now();
    let e = v.get("k").await.unwrap_err();
    assert!(
        matches!(
            e.kind(),
            ValkeyErrorKind::Unavailable | ValkeyErrorKind::Timeout
        ),
        "{e}"
    );
    assert!(started.elapsed() < OP + Duration::from_millis(500));
    assert!(
        Valkey::connect_lazy(&ValkeyConfig {
            url: foundation::Sensitive::new("bukan-url".to_string()),
            op_timeout: OP,
        })
        .is_err()
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn dropped_connections_are_reestablished() {
    let c = ValkeyContainer::start();
    let v = c.client(OP);
    v.set("k", b"1", Duration::from_secs(30)).await.unwrap();
    c.kill_clients();
    let mut value = None;
    for _ in 0..20 {
        if let Ok(Some(x)) = v.get("k").await {
            value = Some(x);
            break;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    assert_eq!(value, Some(b"1".to_vec()));
}
