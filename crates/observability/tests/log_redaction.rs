//! Test keluaran log: nilai rahasia dan pola terlarang tidak boleh muncul (ADR-005, ADR-010 butir 1,
//! KEPUTUSAN-124). Keluaran ditangkap lewat `MemorySink`.

use foundation::{FixedClock, Sensitive, Uuid, new_id};
use observability::LogLevel;
use observability::log::{Component, JsonLogLayer, LogSink, MemorySink, subscriber};
use observability::security::{self, AccessDenial, LoginFailure};
use std::sync::Arc;
use time::OffsetDateTime;

struct Capture {
    app: MemorySink,
    security: MemorySink,
}

fn capture(level: LogLevel, f: impl FnOnce()) -> Capture {
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
    Capture { app, security: sec }
}

#[test]
fn sensitive_field_names_never_leak_values() {
    let c = capture(LogLevel::Info, || {
        tracing::info!(
            password = "P@ssw0rd-Unik-1",
            pin = 123456u64,
            otp = "654321",
            access_token = "tok-abc-XYZ-999",
            authorization = "Bearer tok-bearer-777",
            cookie = "sid=cookie-secret-1",
            card_number = "4111111111111111",
            account_number = "1234567890",
            nik = "3174012345678901",
            email = "pasien@example.com",
            name_enc = "ciphertext-0001",
            order_id = "ord-1",
            "login_attempt"
        );
    });
    let text = c.app.text();
    for secret in [
        "P@ssw0rd-Unik-1",
        "123456",
        "654321",
        "tok-abc-XYZ-999",
        "tok-bearer-777",
        "cookie-secret-1",
        "4111111111111111",
        "1234567890",
        "3174012345678901",
        "pasien@example.com",
        "ciphertext-0001",
    ] {
        assert!(!text.contains(secret), "bocor {secret}: {text}");
    }
    assert!(text.contains("\"password\":\"[REDACTED]\""), "{text}");
    assert!(text.contains("\"pin\":\"[REDACTED]\""), "{text}");
    assert!(text.contains("\"order_id\":\"ord-1\""), "{text}");
    assert!(text.contains("\"message\":\"login_attempt\""), "{text}");
}

#[test]
fn sensitive_wrapper_in_debug_field_is_redacted() {
    #[derive(Debug)]
    #[allow(dead_code)]
    struct Req {
        user: String,
        pin: Sensitive<String>,
    }
    let req = Req {
        user: "budi".into(),
        pin: Sensitive::new("pin-rahasia-8812".into()),
    };
    let c = capture(LogLevel::Info, || {
        tracing::info!(request = ?req, secret_value = ?Sensitive::new("sv-1234"), "got");
    });
    let text = c.app.text();
    assert!(!text.contains("pin-rahasia-8812"), "{text}");
    assert!(!text.contains("sv-1234"), "{text}");
}

#[test]
fn forbidden_patterns_in_message_and_values_are_scrubbed() {
    let jwt = "eyJhbGciOiJSUzI1NiJ9.eyJzdWIiOiIxMjMifQ.c2lnbmF0dXJl";
    let c = capture(LogLevel::Info, || {
        tracing::info!("header Authorization: Bearer abc.def-123 dikirim");
        tracing::info!("jwt {jwt} dipakai");
        tracing::warn!(detail = "kartu 4111 1111 1111 1111 ditolak", "gagal bayar");
        tracing::warn!(detail = "user budi@example.co.id tidak ada", "gagal login");
        tracing::info!(
            url = "/f?X-Amz-Signature=deadbeefcafe&X-Amz-Expires=60",
            "unduh"
        );
        tracing::info!(err = "password=hunter2 salah", "galat");
    });
    let text = c.app.text();
    for secret in [
        "abc.def-123",
        jwt,
        "4111 1111",
        "budi@example",
        "deadbeefcafe",
        "hunter2",
    ] {
        assert!(!text.contains(secret), "bocor {secret}: {text}");
    }
    assert!(text.contains("X-Amz-Expires=60"), "{text}");
}

#[test]
fn newline_injection_cannot_forge_a_log_line() {
    let c = capture(LogLevel::Info, || {
        tracing::info!(
            note = "ok\n{\"level\":\"error\",\"component\":\"forged\"}",
            "pesan\r\nbaris-palsu"
        );
    });
    let lines = c.app.lines();
    assert_eq!(lines.len(), 1);
    let line = lines.first().unwrap();
    assert!(!line.contains('\n') && !line.contains('\r'), "{line}");
    assert!(line.starts_with('{') && line.ends_with('}'));
    assert!(line.contains("\\n"), "{line}");
}

#[test]
fn reserved_fields_cannot_be_forged_by_application_code() {
    let c = capture(LogLevel::Info, || {
        tracing::info!(
            component = "forged",
            level = "fatal",
            timestamp = "1999-01-01T00:00:00Z",
            log_stream = "security",
            correlation_id = "bukan-uuid\nlevel=ERROR",
            "x"
        );
    });
    let text = c.app.text();
    assert!(text.contains("\"component\":\"api\""), "{text}");
    assert!(text.contains("\"level\":\"info\""), "{text}");
    assert!(
        text.contains("\"timestamp\":\"1970-01-01T00:00:00Z\""),
        "{text}"
    );
    for forged in [
        "forged",
        "fatal",
        "1999",
        "log_stream",
        "bukan-uuid",
        "correlation_id",
    ] {
        assert!(!text.contains(forged), "{forged}: {text}");
    }
    assert!(
        c.security.lines().is_empty(),
        "app log tidak boleh masuk aliran security"
    );
}

#[test]
fn valid_correlation_id_is_emitted_and_inherited_from_span() {
    let id = new_id().hyphenated().to_string();
    let c = capture(LogLevel::Info, || {
        let span = tracing::info_span!("request", correlation_id = %id, route = "/a/{id}");
        let _g = span.enter();
        tracing::info!("di dalam span");
    });
    let text = c.app.text();
    assert!(
        text.contains(&format!("\"correlation_id\":\"{id}\"")),
        "{text}"
    );
    assert!(text.contains("\"route\":\"/a/{id}\""), "{text}");
}

#[test]
fn app_level_filter_applies_but_security_stream_is_unfiltered_and_separate() {
    let actor: Uuid = new_id();
    let c = capture(LogLevel::Error, || {
        tracing::info!("disaring");
        tracing::error!("lolos");
        security::login_failed(Some(actor), LoginFailure::BadCredentials);
        security::access_denied(
            None,
            "/patient/v1/orders/{order_id}",
            AccessDenial::ForeignResourceId,
        );
    });
    let app = c.app.text();
    assert!(app.contains("lolos") && !app.contains("disaring"), "{app}");
    assert!(
        !app.contains("security_event") && !app.contains("login_failed"),
        "{app}"
    );
    let sec = c.security.lines();
    assert_eq!(sec.len(), 2, "{sec:?}");
    for l in &sec {
        assert!(l.contains("\"log_stream\":\"security\""), "{l}");
        assert!(l.contains("\"component\":\"api\""), "{l}");
    }
    let first = sec.first().unwrap();
    assert!(
        first.contains("\"event_type\":\"login_failed\"")
            && first.contains(&actor.hyphenated().to_string()),
        "{first}"
    );
    assert!(sec.get(1).unwrap().contains("foreign_resource_id"));
}

#[test]
fn account_id_with_all_digit_groups_is_emitted_intact() {
    // TD-21: dulu ~1% UUID acak (kelompok ke-3..5 semuanya digit) tersamar sebagian sebagai kartu 4-4-4-4
    // atau HP, sehingga account_id di security log rusak dan test di atas gagal sesekali.
    for raw in [
        "01a20164-f5ec-7123-8456-123456789012",
        "01a20164-f5ec-7123-9456-081234567890",
        "01a20164-f5ec-7123-8456-628123456789",
    ] {
        let actor = Uuid::parse_str(raw).unwrap();
        let c = capture(LogLevel::Info, || {
            security::login_failed(Some(actor), LoginFailure::BadCredentials);
            tracing::info!(actor_id = %actor, "x");
        });
        let sec = c.security.text();
        assert!(sec.contains(&format!("\"account_id\":\"{raw}\"")), "{sec}");
        let app = c.app.text();
        assert!(app.contains(&format!("\"actor_id\":\"{raw}\"")), "{app}");
    }
}

#[test]
fn security_log_obeys_the_same_denylist() {
    let c = capture(LogLevel::Info, || {
        security::access_denied(
            None,
            "/x?token=route-token-123 jwt=eyJa.bb.cc",
            AccessDenial::RoleNotAllowed,
        );
        // Kejadian di target security dari kode lain tetap melewati redaksi.
        tracing::warn!(
            target: observability::security::SECURITY_TARGET,
            password = "sec-pw-555",
            detail = "Bearer sec-bearer-666",
            "x"
        );
    });
    let text = c.security.text();
    for secret in [
        "route-token-123",
        "eyJa.bb.cc",
        "sec-pw-555",
        "sec-bearer-666",
    ] {
        assert!(!text.contains(secret), "bocor {secret}: {text}");
    }
}

#[test]
fn audit_write_failure_goes_to_security_stream_only() {
    let before = security::audit_write_failures_total();
    let c = capture(LogLevel::Error, || {
        security::audit_write_failed("e_09", security::AuditWriteFailure::StoreError);
        security::audit_write_failed("e_05", security::AuditWriteFailure::InvalidEvent);
    });
    // US-1501 AC3 Fase 1: penghitung naik satu per kegagalan (test lain di biner ini tidak memanggilnya).
    assert_eq!(security::audit_write_failures_total() - before, 2);
    assert!(!c.app.text().contains("audit_write_failed"));
    let sec = c.security.lines();
    assert_eq!(sec.len(), 2, "{sec:?}");
    let first = sec.first().unwrap();
    assert!(
        first.contains("\"event_type\":\"audit_write_failed\""),
        "{first}"
    );
    assert!(first.contains("\"event_code\":\"e_09\""), "{first}");
    assert!(first.contains("\"stage\":\"store_error\""), "{first}");
    assert!(sec.get(1).unwrap().contains("\"stage\":\"invalid_event\""));
}
