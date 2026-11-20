//! Batas waktu adaptor SES tidak meninggalkan koneksi di latar belakang (review keamanan F1-T36, Low): server
//! TLS yang tidak pernah menjawab harus melihat koneksinya ditutup segera setelah `send_raw` mengembalikan
//! `Timeout`. Berada di crate email-relay karena perlengkapan CA uji dan TLS server ada di sini.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

mod common;

use common::*;
use foundation::Sensitive;
use std::time::{Duration, Instant};
use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;
use tokio::sync::oneshot;

#[tokio::test]
async fn timed_out_ses_call_closes_its_connection() {
    let pki = TestPki::generate();
    let acceptor = email_relay::server_tls(&pki.cert, &pki.key).unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let (tx, rx) = oneshot::channel::<Instant>();
    tokio::spawn(async move {
        let (tcp, _) = listener.accept().await.unwrap();
        let mut tls = acceptor.accept(tcp).await.unwrap();
        // Baca permintaan lalu diam; catat kapan klien menutup koneksi (EOF atau galat baca).
        let mut buf = vec![0u8; 64 * 1024];
        loop {
            match tls.read(&mut buf).await {
                Ok(0) | Err(_) => break,
                Ok(_) => {}
            }
        }
        let _ = tx.send(Instant::now());
    });

    let client = ses::SesClient::new(&ses::SesConfig {
        local_endpoint: Some(format!("https://127.0.0.1:{port}")),
        ca_file: pki.ca.clone(),
        timeout: Duration::from_millis(300),
        configuration_set: CONFIG_SET.to_string(),
        credentials: ses::AwsCredentials {
            access_key_id: ACCESS_KEY_ID.to_string(),
            secret_access_key: Sensitive::new(SECRET.to_string()),
        },
    })
    .unwrap();
    let raw = keycloak_like_message("uji batas waktu");
    let email = ses::RawEmail {
        from: FROM,
        to: RCPT,
        raw: raw.as_bytes(),
    };
    let now = time::macros::datetime!(2026-11-20 03:04:05 UTC);
    let r = client.send_raw(&email, now).await;
    let returned = Instant::now();
    assert_eq!(r, Err(ses::SesError::Timeout));
    let closed = tokio::time::timeout(Duration::from_secs(3), rx)
        .await
        .expect("koneksi ke SES harus ditutup setelah batas waktu (task koneksi tidak tertinggal)")
        .unwrap();
    assert!(closed.saturating_duration_since(returned) < Duration::from_secs(1));
}
