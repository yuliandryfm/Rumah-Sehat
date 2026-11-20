//! Test integrasi relay (F1-T36, ADR-026; modul kritis T1): klien SMTP sungguhan lewat TCP + STARTTLS (CA uji
//! dibangkitkan saat test) → relay → adaptor SES → tiruan SES HTTPS yang memvalidasi SigV4. Tanpa AWS.
//!
//! Skenario negatif mengikuti ADR-026 (Konsekuensi) dan threat model M-34/M-35: tanpa STARTTLS, `From` di
//! luar daftar izin, lebih dari satu penerima (amplop dan header), header `X-SES-*`, pesan melebihi batas,
//! baris tanpa CRLF yang benar, SES 429/5xx/400/batas waktu/tanda tangan ditolak, batas laju, dan teks
//! balasan maupun log tidak memuat alamat atau subjek.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

mod common;

use common::*;
use std::time::{Duration, Instant};

const SUBJECT: &str = "Verifikasi email Ruang Sehat subjek-rahasia-42";

fn no_leak(reply: &str) {
    for s in [
        RCPT,
        FROM,
        "penyerang",
        "lain@",
        "subjek-rahasia",
        "action-token",
    ] {
        assert!(!reply.contains(s), "balasan menggemakan {s:?}: {reply}");
    }
}

fn assert_code(reply: &str, code: &str) {
    assert!(
        reply.starts_with(code),
        "diharapkan {code}, didapat {reply:?}"
    );
    if !reply.starts_with("2") && !reply.starts_with("3") {
        assert!(
            reply.contains(" ref="),
            "balasan galat tanpa ID korelasi: {reply}"
        );
    }
    no_leak(reply);
}

#[tokio::test]
async fn message_is_forwarded_to_ses_only_after_ses_accepts() {
    let h = start().await;
    h.ses.set(200, Duration::from_millis(400));
    let (mut c, greeting) = Client::connect(h.port).await;
    assert!(greeting.starts_with("220 email-relay "), "{greeting}");
    let ehlo = c.starttls(&h.pki).await;
    assert!(
        ehlo.starts_with("250") && !ehlo.contains("STARTTLS") && ehlo.contains("SIZE 8192"),
        "{ehlo}"
    );

    let started = Instant::now();
    let r = c
        .send_message("NoReply@UJI.invalid", RCPT, &keycloak_like_message(SUBJECT))
        .await;
    assert_code(&r, "250 2.0.0");
    assert!(
        started.elapsed() >= Duration::from_millis(400),
        "250 dikirim sebelum SES menjawab"
    );
    assert!(r.contains(" ref="));

    let reqs = h.ses.requests();
    assert_eq!(reqs.len(), 1);
    let j = &reqs[0].json;
    // Permintaan disusun relay: From kanonik dari daftar izin, Destination dari RCPT, configuration set eksplisit.
    assert_eq!(j["FromEmailAddress"], FROM);
    assert_eq!(j["Destination"], serde_json::json!({"ToAddresses": [RCPT]}));
    assert_eq!(j["ConfigurationSetName"], CONFIG_SET);
    assert_eq!(
        j["ConfigurationOverrides"]["Tracking"],
        serde_json::json!({"ClickTrackingEnabled": "DISABLED", "OpenTrackingEnabled": "DISABLED"})
    );
    assert!(j.get("EndpointId").is_none());
    // MIME diteruskan apa adanya (CRLF), dengan dot-unstuffing.
    let expected = keycloak_like_message(SUBJECT).replace("\r\n..baris", "\r\n.baris");
    assert_eq!(String::from_utf8(reqs[0].raw.clone()).unwrap(), expected);
    assert_eq!(*h.ses.bad_signatures.lock().unwrap(), 0);
    assert_code(&c.cmd("QUIT").await, "221");
}

#[tokio::test]
async fn mail_without_starttls_is_rejected() {
    let h = start().await;
    let (mut c, _) = Client::connect(h.port).await;
    assert!(c.cmd("EHLO uji.invalid").await.contains("250 STARTTLS"));
    assert_code(&c.cmd(&format!("MAIL FROM:<{FROM}>")).await, "530 5.7.0");
    assert_code(&c.cmd(&format!("RCPT TO:<{RCPT}>")).await, "530 5.7.0");
    assert_code(&c.cmd("DATA").await, "530 5.7.0");
    assert_code(&c.cmd("AUTH PLAIN AGFiYwBkZWY=").await, "502");
    // Galat kelima menutup sesi.
    assert_code(&c.cmd("MAIL FROM:<x@y.z>").await, "530");
    assert!(c.read_reply().await.starts_with("421 4.7.0"));
    assert_eq!(c.read_reply().await, "");
    assert!(h.ses.requests().is_empty());
}

#[tokio::test]
async fn starttls_before_ehlo_and_pipelined_data_are_rejected() {
    let h = start().await;
    let (mut c, _) = Client::connect(h.port).await;
    assert_code(&c.cmd("STARTTLS").await, "503");
    // Perintah disisipkan berbarengan dengan STARTTLS (sebelum TLS): relay menutup tanpa TLS.
    assert!(c.cmd("EHLO uji.invalid").await.starts_with("250"));
    c.send_raw(format!("STARTTLS\r\nMAIL FROM:<{FROM}>\r\n").as_bytes())
        .await;
    let r = c.read_reply().await;
    assert!(r.starts_with("220"), "{r}");
    assert_eq!(
        c.read_reply().await,
        "",
        "koneksi harus ditutup tanpa jabat tangan TLS"
    );
}

#[tokio::test]
async fn sender_outside_allowlist_is_rejected() {
    let h = start().await;
    let (mut c, _) = Client::connect(h.port).await;
    c.starttls(&h.pki).await;
    assert_code(
        &c.cmd("MAIL FROM:<penyerang@uji.invalid>").await,
        "550 5.7.1",
    );
    assert_code(&c.cmd("MAIL FROM:<>").await, "501");
    assert_code(&c.cmd(&format!("MAIL FROM:<{FROM}> SMTPUTF8")).await, "555");
    assert_code(
        &c.cmd(&format!("MAIL FROM:<{FROM}> SIZE=999999")).await,
        "552",
    );
    assert!(h.ses.requests().is_empty());
}

#[tokio::test]
async fn more_than_one_recipient_is_rejected() {
    let h = start().await;
    let (mut c, _) = Client::connect(h.port).await;
    c.starttls(&h.pki).await;
    assert_code(&c.cmd(&format!("MAIL FROM:<{FROM}>")).await, "250");
    assert_code(&c.cmd(&format!("RCPT TO:<{RCPT}>")).await, "250");
    assert_code(&c.cmd("RCPT TO:<lain@uji.invalid>").await, "452 4.5.3");
    assert_code(&c.cmd("DATA").await, "354");
    c.send_raw(keycloak_like_message(SUBJECT).as_bytes()).await;
    c.send_raw(b".\r\n").await;
    assert_code(&c.read_reply().await, "250");
    let reqs = h.ses.requests();
    assert_eq!(reqs.len(), 1);
    assert_eq!(
        reqs[0].json["Destination"]["ToAddresses"],
        serde_json::json!([RCPT])
    );
}

#[tokio::test]
async fn header_violations_are_rejected_before_ses() {
    let h = start().await;
    let base = keycloak_like_message(SUBJECT);
    let cases = [
        (
            "X-SES-CONFIGURATION-SET",
            base.replacen("Subject:", "X-SES-CONFIGURATION-SET: lain\r\nSubject:", 1),
        ),
        (
            "x-ses-source-arn",
            base.replacen("Subject:", "x-ses-source-arn: arn:aws:ses:x\r\nSubject:", 1),
        ),
        (
            "To tambahan",
            base.replacen(
                &format!("To: {RCPT}"),
                &format!("To: {RCPT}, lain@uji.invalid"),
                1,
            ),
        ),
        (
            "Cc",
            base.replacen("Subject:", "Cc: lain@uji.invalid\r\nSubject:", 1),
        ),
        (
            "Bcc",
            base.replacen("Subject:", "Bcc: lain@uji.invalid\r\nSubject:", 1),
        ),
        (
            "From lain",
            base.replacen(&format!("<{FROM}>"), "<penyerang@uji.invalid>", 1),
        ),
        (
            "From ganda",
            base.replacen("Subject:", &format!("From: {FROM}\r\nSubject:"), 1),
        ),
        (
            "header non-ASCII",
            base.replacen(SUBJECT, "Verifikasi \u{e9}", 1),
        ),
    ];
    for (name, msg) in cases {
        let (mut c, _) = Client::connect(h.port).await;
        c.starttls(&h.pki).await;
        let r = c.send_message(FROM, RCPT, &msg).await;
        assert!(r.starts_with("554 5.6.0"), "{name}: {r}");
        assert_code(&r, "554");
    }
    assert!(h.ses.requests().is_empty());
}

#[tokio::test]
async fn oversized_message_is_rejected() {
    let h = start().await;
    let (mut c, _) = Client::connect(h.port).await;
    c.starttls(&h.pki).await;
    let mut msg = keycloak_like_message(SUBJECT);
    for _ in 0..200 {
        msg.push_str(&format!("{}\r\n", "x".repeat(76)));
    }
    let r = c.send_message(FROM, RCPT, &msg).await;
    assert_code(&r, "552 5.3.4");
    // Sesi tetap dapat dipakai setelah pesan terlalu besar.
    let r = c
        .send_message(FROM, RCPT, &keycloak_like_message(SUBJECT))
        .await;
    assert_code(&r, "250");
    assert_eq!(h.ses.requests().len(), 1);
}

#[tokio::test]
async fn bad_line_endings_and_long_lines_close_the_session() {
    let h = start().await;
    for bad in [
        "Subject: x\nTo: y\r\n".to_string(),
        "Subject: x\r\rz\r\n".to_string(),
        format!("{}\r\n", "a".repeat(1001)),
    ] {
        let (mut c, _) = Client::connect(h.port).await;
        c.starttls(&h.pki).await;
        assert_code(&c.cmd(&format!("MAIL FROM:<{FROM}>")).await, "250");
        assert_code(&c.cmd(&format!("RCPT TO:<{RCPT}>")).await, "250");
        assert_code(&c.cmd("DATA").await, "354");
        c.send_raw(bad.as_bytes()).await;
        c.send_raw(b".\r\n").await;
        let r = c.read_reply().await;
        assert_code(&r, "500 5.5.2");
        assert_eq!(c.read_reply().await, "", "sesi harus ditutup");
    }
    // Perintah dengan LF tunggal juga ditolak.
    let (mut c, _) = Client::connect(h.port).await;
    c.send_raw(b"EHLO uji.invalid\n").await;
    assert_code(&c.read_reply().await, "500 5.5.2");
    assert!(h.ses.requests().is_empty());
}

#[tokio::test]
async fn ses_failures_map_to_temporary_or_permanent_replies() {
    let h = start_with(settings(), SECRET, Duration::from_millis(800)).await;
    for (status, delay, expected) in [
        (429, Duration::ZERO, "451 4.4.1"),
        (500, Duration::ZERO, "451 4.4.1"),
        (503, Duration::ZERO, "451 4.4.1"),
        (400, Duration::ZERO, "554 5.0.0"),
        (200, Duration::from_secs(3), "451 4.4.1"), // batas waktu SES
    ] {
        h.ses.set(status, delay);
        let (mut c, _) = Client::connect(h.port).await;
        c.starttls(&h.pki).await;
        let started = Instant::now();
        let r = c
            .send_message(FROM, RCPT, &keycloak_like_message(SUBJECT))
            .await;
        assert!(r.starts_with(expected), "SES {status}: {r}");
        assert_code(&r, &expected[..3]);
        assert!(
            started.elapsed() < Duration::from_secs(2),
            "relay menunggu melewati batas waktu SES"
        );
    }
}

#[tokio::test]
async fn ses_rejects_wrong_signature_and_relay_reports_temporary_failure() {
    let h = start_with(
        settings(),
        "kunci-rahasia-salah-tetapi-cukup-panjang-000",
        Duration::from_secs(5),
    )
    .await;
    let (mut c, _) = Client::connect(h.port).await;
    c.starttls(&h.pki).await;
    let r = c
        .send_message(FROM, RCPT, &keycloak_like_message(SUBJECT))
        .await;
    assert_code(&r, "451 4.4.1");
    assert_eq!(
        *h.ses.bad_signatures.lock().unwrap(),
        1,
        "tiruan SES harus menolak tanda tangan salah"
    );
    assert!(h.ses.requests().is_empty());
}

#[tokio::test]
async fn per_recipient_limit_answers_250_but_drops() {
    let mut s = settings();
    s.per_recipient_per_hour = 1;
    let h = start_with(s, SECRET, Duration::from_secs(5)).await;
    for _ in 0..3 {
        let (mut c, _) = Client::connect(h.port).await;
        c.starttls(&h.pki).await;
        let r = c
            .send_message(FROM, RCPT, &keycloak_like_message(SUBJECT))
            .await;
        assert_code(&r, "250 2.0.0");
    }
    assert_eq!(
        h.ses.requests().len(),
        1,
        "hanya pesan pertama yang sampai ke SES"
    );
}

#[tokio::test]
async fn ses_failures_do_not_consume_recipient_quota() {
    // Review keamanan F1-T36 (Medium): kegagalan berulang (misalnya "lupa password" saat SES menolak) tidak
    // boleh menghabiskan kuota per penerima sehingga email sah korban tertahan.
    let mut s = settings();
    s.per_recipient_per_hour = 5;
    let h = start_with(s, SECRET, Duration::from_millis(500)).await;
    for status in [503, 429, 400, 500, 503] {
        h.ses.set(status, Duration::ZERO);
        let (mut c, _) = Client::connect(h.port).await;
        c.starttls(&h.pki).await;
        let r = c
            .send_message(FROM, RCPT, &keycloak_like_message(SUBJECT))
            .await;
        assert!(
            r.starts_with("451") || r.starts_with("554"),
            "SES {status}: {r}"
        );
    }
    // Batas waktu SES juga mengembalikan kuota.
    h.ses.set(200, Duration::from_secs(2));
    let (mut c, _) = Client::connect(h.port).await;
    c.starttls(&h.pki).await;
    assert_code(
        &c.send_message(FROM, RCPT, &keycloak_like_message(SUBJECT))
            .await,
        "451",
    );
    let before = h.ses.requests().len();
    h.ses.set(200, Duration::ZERO);
    for _ in 0..5 {
        let (mut c, _) = Client::connect(h.port).await;
        c.starttls(&h.pki).await;
        assert_code(
            &c.send_message(FROM, RCPT, &keycloak_like_message(SUBJECT))
                .await,
            "250",
        );
    }
    assert_eq!(
        h.ses.requests().len(),
        before + 5,
        "lima kiriman sah sesudah kegagalan tetap sampai ke SES"
    );
    let (mut c, _) = Client::connect(h.port).await;
    c.starttls(&h.pki).await;
    assert_code(
        &c.send_message(FROM, RCPT, &keycloak_like_message(SUBJECT))
            .await,
        "250",
    );
    assert_eq!(
        h.ses.requests().len(),
        before + 5,
        "kiriman ke-6 sesudah 5 diterima dibuang"
    );
}

#[tokio::test]
async fn global_limit_defers_with_4xx() {
    let mut s = settings();
    s.global_per_minute = 1;
    let h = start_with(s, SECRET, Duration::from_secs(5)).await;
    let (mut c, _) = Client::connect(h.port).await;
    c.starttls(&h.pki).await;
    assert_code(
        &c.send_message(FROM, RCPT, &keycloak_like_message(SUBJECT))
            .await,
        "250",
    );
    let other = keycloak_like_message(SUBJECT).replace(RCPT, "pasien.dua@uji.invalid");
    let r = c.send_message(FROM, "pasien.dua@uji.invalid", &other).await;
    assert_code(&r, "451 4.7.1");
    assert!(!r.contains("pasien.dua"));
    assert_eq!(h.ses.requests().len(), 1);
}

#[tokio::test]
async fn connection_limit_answers_421() {
    let mut s = settings();
    s.max_connections = 1;
    let h = start_with(s, SECRET, Duration::from_secs(5)).await;
    let (_first, g) = Client::connect(h.port).await;
    assert!(g.starts_with("220"));
    let (_second, g2) = Client::connect(h.port).await;
    assert!(g2.starts_with("421 4.3.2"), "{g2}");
}

#[tokio::test]
async fn client_that_does_not_trust_the_relay_certificate_cannot_complete_tls() {
    let h = start().await;
    let other = TestPki::generate();
    let (mut c, _) = Client::connect(h.port).await;
    assert!(c.cmd("EHLO uji.invalid").await.contains("STARTTLS"));
    assert!(c.cmd("STARTTLS").await.starts_with("220"));
    // Jabat tangan dengan CA lain gagal di sisi klien; relay tidak menerima perintah apa pun.
    assert!(c.upgrade(&other).await.is_err());
    assert!(h.ses.requests().is_empty());
}
