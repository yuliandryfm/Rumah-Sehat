//! Log relay (F1-T36, ADR-026 butir 4, ADR-005, ADR-010): hasil, ukuran, ID korelasi, dan `MessageId` SES
//! dicatat; alamat, subjek, dan badan tidak. Berkas test terpisah supaya subscriber log per-thread tidak
//! bersaing dengan test lain dalam biner yang sama (cache minat callsite tracing).
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

mod common;

use common::*;
use std::time::Duration;

const SUBJECT: &str = "Verifikasi email Ruang Sehat subjek-rahasia-42";

#[tokio::test(flavor = "current_thread")]
async fn logs_contain_outcome_and_ids_but_no_addresses_or_subject() {
    use observability::LogLevel;
    use observability::log::{Component, JsonLogLayer, LogSink, MemorySink, subscriber};
    let app = MemorySink::default();
    let sec = MemorySink::default();
    let layer = JsonLogLayer::new(
        Component::EmailRelay,
        LogLevel::Trace,
        std::sync::Arc::new(foundation::FixedClock::new(
            time::OffsetDateTime::UNIX_EPOCH,
        )),
    )
    .with_sinks(
        std::sync::Arc::new(app.clone()) as std::sync::Arc<dyn LogSink>,
        std::sync::Arc::new(sec.clone()),
    );
    let _guard = tracing::subscriber::set_default(subscriber(layer));

    let h = start().await;
    let (mut c, _) = Client::connect(h.port).await;
    c.starttls(&h.pki).await;
    assert!(
        c.send_message(FROM, RCPT, &keycloak_like_message(SUBJECT))
            .await
            .starts_with("250")
    );
    assert!(
        c.cmd("MAIL FROM:<penyerang@uji.invalid>")
            .await
            .starts_with("550")
    );
    let x_ses = keycloak_like_message(SUBJECT).replacen("Subject:", "X-SES-X: y\r\nSubject:", 1);
    assert!(c.send_message(FROM, RCPT, &x_ses).await.starts_with("554"));
    c.cmd("QUIT").await;
    tokio::time::sleep(Duration::from_millis(100)).await;

    let text = app.text();
    assert!(text.contains("\"outcome\":\"sent\""), "{text}");
    assert!(
        text.contains("0100018f-uji-0001"),
        "MessageId SES dicatat: {text}"
    );
    assert!(
        text.contains("from_not_allowed") && text.contains("x_ses_header"),
        "{text}"
    );
    assert!(text.contains("\"component\":\"email-relay\""), "{text}");
    for s in [
        RCPT,
        FROM,
        "penyerang",
        "subjek-rahasia",
        "action-token",
        "token-aksi-sintetis",
    ] {
        assert!(!text.contains(s), "log memuat {s:?}");
    }
    assert!(
        !sec.text().contains("email_recipient_rate_dropped"),
        "belum ada pembuangan"
    );

    // Batas per penerima 5/jam: 5 pesan diterima SES, pesan ke-6 dibalas 250 tetapi dibuang dan dicatat di
    // security log (review keamanan F1-T36).
    let mut s = settings();
    s.per_recipient_per_hour = 5;
    let h2 = start_with(s, SECRET, Duration::from_secs(5)).await;
    for i in 0..6 {
        let (mut c, _) = Client::connect(h2.port).await;
        c.starttls(&h2.pki).await;
        let r = c
            .send_message(FROM, RCPT, &keycloak_like_message(SUBJECT))
            .await;
        assert!(r.starts_with("250 2.0.0"), "pesan {i}: {r}");
    }
    tokio::time::sleep(Duration::from_millis(100)).await;
    assert_eq!(h2.ses.requests().len(), 5, "pesan ke-6 tidak sampai ke SES");
    let sec_text = sec.text();
    assert_eq!(
        sec_text
            .matches("\"event_type\":\"email_recipient_rate_dropped\"")
            .count(),
        1,
        "{sec_text}"
    );
    assert!(
        sec_text.contains("\"log_stream\":\"security\""),
        "{sec_text}"
    );
    assert!(app.text().contains("\"outcome\":\"dropped\""));
    for s in [RCPT, FROM, "subjek-rahasia"] {
        assert!(!sec_text.contains(s), "security log memuat {s:?}");
    }
}
