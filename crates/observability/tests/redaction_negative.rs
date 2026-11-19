//! Test negatif redaksi tambahan (review keamanan F1-T10): variasi nama field, nilai Debug bersarang,
//! JSON ber-escape, dan kolom sensitif yang dibaca dari kamus data.

#![allow(clippy::unwrap_used, clippy::expect_used)] // berkas test

use foundation::{CorrelationId, FixedClock};
use observability::LogLevel;
use observability::log::{Component, JsonLogLayer, LogSink, MemorySink, subscriber};
use observability::redact::{REDACTED, is_sensitive_key, scrub_text};
use std::sync::Arc;
use time::OffsetDateTime;

fn capture(level: LogLevel, f: impl FnOnce()) -> (String, String) {
    let app = MemorySink::default();
    let sec = MemorySink::default();
    let layer = JsonLogLayer::new(
        Component::Api,
        level,
        Arc::new(FixedClock::new(OffsetDateTime::UNIX_EPOCH)),
    )
    .with_sinks(
        Arc::new(app.clone()) as Arc<dyn LogSink>,
        Arc::new(sec.clone()),
    );
    tracing::subscriber::with_default(subscriber(layer), f);
    (app.text(), sec.text())
}

#[test]
fn key_name_variants_are_sensitive() {
    for k in [
        "otp_code",
        "pin_code",
        "user_pin",
        "new_pin",
        "mfa_code",
        "totp",
        "kata_sandi",
        "sandi",
        "pwd",
        "passphrase",
        "no_hp",
        "nohp",
        "telepon",
        "msisdn",
        "mobile_number",
        "nik_pasien",
        "no_kartu",
        "nomor_kartu",
        "card_no",
        "account_no",
        "no_rekening_bank",
        "nama_lengkap",
        "nama_pasien",
        "tgl_lahir",
        "address",
        "alamat_rumah",
        "recipient_phone",
        "npwp",
        "bpjs_number",
        "auth_code",
        "code_verifier",
        "x-api-key",
        "verification_code",
        "ibu_kandung",
        "otpCode",
        "userPin",
        "recipientPhone",
        "accessToken",
        "fullName",
        "refreshToken",
        "name",
        "nama",
        "patient_name",
        "account_holder_name",
        "before_values",
        "after_values",
    ] {
        assert!(is_sensitive_key(k), "{k} harus sensitif");
    }
}

#[test]
fn technical_key_names_are_not_over_matched() {
    for k in [
        "correlation_id",
        "account_id",
        "actor_account_id",
        "order_id",
        "route",
        "status",
        "duration_ms",
        "component_name",
        "event_name",
        "ip_address",
        "source_ip",
        "user_agent",
        "event_type",
        "reason",
        "lag_seconds",
        "entity",
        "entity_id",
        "shipping",
        "mapping",
        "footprint",
        "region_code",
        "message",
    ] {
        assert!(!is_sensitive_key(k), "{k} tidak boleh sensitif");
    }
}

#[test]
fn free_text_pairs_with_variant_keys_are_scrubbed() {
    for (input, secret) in [
        ("otp_code=123456 sent", "123456"),
        ("user_pin: 4321", "4321"),
        ("kata_sandi=hunter2 x", "hunter2"),
        ("nik=3174012345678901", "3174012345678901"),
        ("Authorization: Token abcSECRET", "abcSECRET"),
        ("authorization: Negotiate YIIabcSECRET", "YIIabcSECRET"),
        ("[(\"x-api-key\", \"k-SECRET\")]", "k-SECRET"),
        ("no_hp=081234567890", "081234567890"),
    ] {
        let out = scrub_text(input);
        assert!(!out.contains(secret), "{input} -> {out}");
        assert!(out.contains(REDACTED), "{out}");
    }
}

#[derive(Debug)]
#[allow(dead_code)]
struct Inner {
    value: String,
}

#[derive(Debug)]
#[allow(dead_code)]
struct Req {
    user_id: u32,
    token: Option<String>,
    pin: Option<u32>,
    refresh_token: Vec<String>,
    password: Inner,
    note: String,
}

#[test]
fn nested_debug_values_are_fully_redacted() {
    let r = Req {
        user_id: 7,
        token: Some("tokSECRET1".into()),
        pin: Some(4321),
        refresh_token: vec!["rtSECRET2".into(), "rtSECRET2b".into()],
        password: Inner {
            value: "pwSECRET4".into(),
        },
        note: "aman".into(),
    };
    let text = scrub_text(&format!("{r:?} | {r:#?}"));
    for secret in ["tokSECRET1", "4321", "rtSECRET2", "rtSECRET2b", "pwSECRET4"] {
        assert!(!text.contains(secret), "bocor {secret}: {text}");
    }
    // Sisa bidang tidak sensitif tetap terbaca dan sisa penutup tidak tertinggal di dalam nilai.
    assert!(
        text.contains("user_id: 7") && text.contains("aman"),
        "{text}"
    );
    assert!(!text.contains("[REDACTED]\""), "sisa kutip: {text}");
    assert!(!text.contains("[REDACTED])"), "sisa kurung: {text}");

    let (app, _) = capture(LogLevel::Info, || {
        tracing::info!(request = ?r, "dbg");
    });
    for secret in ["tokSECRET1", "4321", "rtSECRET2", "pwSECRET4"] {
        assert!(!app.contains(secret), "bocor {secret}: {app}");
    }
}

#[test]
fn json_and_escaped_json_values_are_redacted() {
    let body = String::from("{\"password\":\"pwSECRET3\",\"user\":\"budi\"}");
    // Debug dari String meng-escape kutip: \"password\":\"pwSECRET3\".
    let debug = format!("{body:?}");
    assert!(debug.contains("\\\"password\\\""));
    let out = scrub_text(&debug);
    assert!(!out.contains("pwSECRET3"), "{out}");
    assert!(out.contains("budi"), "{out}");
    assert!(!scrub_text(&body).contains("pwSECRET3"));
    let double = format!("{debug:?}");
    assert!(!scrub_text(&double).contains("pwSECRET3"));

    let (app, _) = capture(LogLevel::Info, || {
        tracing::info!(detail = ?body, "dbg");
        tracing::info!(detail = %body, "disp");
        tracing::info!("body {body}");
    });
    assert!(!app.contains("pwSECRET3"), "{app}");
}

#[test]
fn invalid_correlation_id_on_event_does_not_override_valid_span_id() {
    let id = CorrelationId::new().to_string();
    let (app, _) = capture(LogLevel::Error, || {
        let span = tracing::info_span!("request", correlation_id = %id);
        let _g = span.enter();
        tracing::error!(correlation_id = "not-a-uuid\n{}", "override");
    });
    assert!(
        app.contains(&format!("\"correlation_id\":\"{id}\"")),
        "{app}"
    );
    assert!(!app.contains("not-a-uuid"), "{app}");
}

#[test]
fn multiline_message_cannot_forge_a_line() {
    let (app, _) = capture(LogLevel::Info, || {
        tracing::info!("multi\nline {}", "x\r\n{\"level\":\"error\"}");
    });
    assert_eq!(app.lines().count(), 1, "{app}");
}

#[test]
fn security_stream_cannot_be_forged_from_app_events() {
    let (app, sec) = capture(LogLevel::Info, || {
        tracing::info!(log_stream = "security", component = "evil", "forged");
    });
    assert!(sec.is_empty(), "{sec}");
    assert!(
        !app.contains("evil") && !app.contains("log_stream"),
        "{app}"
    );
}

/// Kriteria yang dapat dibaca dari dokumen: kamus data (docs/04-data-dictionary.md §0.5) menandai kolom
/// dengan sensitivitas `Enc`, `Rahasia`, `PII`, atau `Spesifik`. Semua kolom bertanda itu harus tersamar,
/// kecuali yang dikecualikan dengan alasan tertulis di bawah.
#[test]
fn sensitive_columns_from_data_dictionary_are_redacted() {
    let doc = include_str!("../../../docs/04-data-dictionary.md");
    // Pengecualian: pengenal teknis (ID akun/berkas/IdP) dan alamat IP/user agent boleh di log
    // (ADR-010 butir 6: ID akun dan IP dicatat); kode wilayah, hubungan keluarga, dan params generik
    // bukan data pribadi yang mengidentifikasi orang.
    let excluded = |name: &str| {
        name.ends_with("_id")
            || matches!(
                name,
                "idp_subject"
                    | "rs_recipient_alias"
                    | "source_ip"
                    | "user_agent"
                    | "region_code"
                    | "relationship"
                    | "params"
            )
    };
    let mut checked = 0;
    for line in doc.lines().filter(|l| l.starts_with('|')) {
        let cells: Vec<&str> = line
            .trim()
            .trim_matches('|')
            .split('|')
            .map(str::trim)
            .collect();
        if cells.len() < 5 {
            continue;
        }
        let name = cells[0].trim_matches('`');
        let is_ident = !name.is_empty()
            && name.chars().next().is_some_and(|c| c.is_ascii_lowercase())
            && name
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
        let sens = cells[4];
        let flagged = ["Enc", "Rahasia", "PII", "Spesifik"]
            .iter()
            .any(|t| sens.contains(t));
        if !is_ident || !flagged || excluded(name) {
            continue;
        }
        checked += 1;
        assert!(
            is_sensitive_key(name),
            "kolom {name} ({sens}) lolos redaksi"
        );
    }
    assert!(
        checked >= 40,
        "parser kamus data hanya menemukan {checked} kolom"
    );
}

#[derive(Debug)]
#[allow(dead_code)]
struct W {
    password: String,
    next: String,
}

#[test]
fn escaped_quotes_inside_values_do_not_end_redaction_early() {
    // Debug meng-escape kutip di dalam nilai: "ab\"cdSECRET1".
    let w = W {
        password: "ab\"cdSECRET1".into(),
        next: "terlihat".into(),
    };
    let out = scrub_text(&format!("{w:?}"));
    assert!(!out.contains("SECRET1"), "{out}");
    assert!(out.contains("terlihat"), "{out}");
    // Garis miring berpasangan lalu kutip menutup nilai; field sesudahnya tetap terlihat.
    let out = scrub_text(r#"password="a\\" next=ok"#);
    assert!(out.contains("next=ok"), "{out}");

    // JSON ber-escape dengan kutip di dalam nilai: {\"token\":\"SEC\\\"RET11\"}
    let inner = "{\"token\":\"SEC\\\"RET11\",\"user\":\"budi\"}";
    let out = scrub_text(&format!("{inner:?}"));
    assert!(!out.contains("RET11"), "{out}");
    assert!(out.contains("budi"), "{out}");
    // Nilai rancangan mencoba menutup lebih awal dan menyelipkan field sesudahnya.
    let attack = "{\"token\":\"x\\\",\\\"a\\\":\\\"INNERSECRET\",\"password\":\"TAILSECRET\"}";
    let out = scrub_text(&format!("{attack:?}"));
    assert!(
        !out.contains("INNERSECRET") && !out.contains("TAILSECRET"),
        "{out}"
    );
}

#[test]
fn ihs_and_other_extra_key_variants_are_sensitive() {
    for k in [
        "satusehat_patient_ihs",
        "satusehat_practitioner_ihs",
        "ihs_number",
        "encryption_key",
        "hmac_key",
        "master_key",
        "webhook_key",
        "signing_key",
        "OTPCode",
        "NIKPasien",
        "user_pass",
        "va_number",
        "nomor_va",
        "virtual_account",
    ] {
        assert!(is_sensitive_key(k), "{k}");
    }
    for k in ["key_version", "status_code", "bypass_count", "region_code"] {
        assert!(!is_sensitive_key(k), "{k}");
    }
}

#[test]
fn whitespace_variants_and_url_credentials_are_scrubbed() {
    let url = "GET /cb?access_token=ATSECRET&id_token=IDSECRET&refresh_token=RTSECRET";
    for (input, secret) in [
        ("password\t=\tTABSECRET", "TABSECRET"),
        ("password:\u{a0}NBSPSECRET", "NBSPSECRET"),
        ("pin :   SPACESECRET", "SPACESECRET"),
        ("GET /cb?code=OAUTHCODE1&state=abc", "OAUTHCODE1"),
        ("GET /cb?x=1&code=OAUTHCODE2", "OAUTHCODE2"),
        (url, "ATSECRET"),
        (url, "IDSECRET"),
        (url, "RTSECRET"),
    ] {
        let out = scrub_text(input);
        assert!(!out.contains(secret), "{input} -> {out}");
    }
    // `code` di luar konteks URL tidak disamarkan.
    assert_eq!(
        scrub_text("status_code=200 code 5"),
        "status_code=200 code 5"
    );
    assert!(scrub_text("?state=abc").contains("state=abc"));
}
