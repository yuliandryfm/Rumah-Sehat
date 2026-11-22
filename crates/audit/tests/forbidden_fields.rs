//! US-1501 AC2, ADR-005, ADR-009 butir 7: event audit tidak pernah memuat password, PIN, OTP, token,
//! rahasia MFA, nomor kartu/rekening lengkap, atau NIK terbuka; field wajib PRD §7.2 ditegakkan.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use audit::{
    Actor, AuditContext, AuditErrorKind, AuditEvent, AuditRecord, Code, Disclosure, EventCode,
    FieldChange, FieldRule, ObjectId, ObjectPolicy, ObjectRef, Outcome, Source, SourceComponent,
    Subject, Value,
};
use foundation::{CorrelationId, FixedClock, Idr, Sensitive, new_id};
use serde_json::json;
use std::net::{IpAddr, Ipv4Addr};
use time::macros::datetime;

const NIK: &str = "3174012345678901";
const CARD: &str = "4111111111111234";
const ACCOUNT_NO: &str = "1234567890987";
const PASSWORD: &str = "P@ssw0rd-Rahasia-77";
const OTP: &str = "482913";
const TOKEN: &str = "eyJhbGciOiJSUzI1NiJ9.eyJzdWIiOiIxIn0.c2lnbmF0dXJl";
const EMAIL: &str = "budi.santoso@example.com";

static CARE_RECIPIENT: ObjectPolicy = ObjectPolicy::non_clinical(
    "care_recipient",
    &[
        FieldRule::value("status"),
        FieldRule::last4("nik"),
        FieldRule::text("relationship_label"),
        FieldRule::name_only("photo"),
    ],
);
static PAYOUT_ACCOUNT: ObjectPolicy = ObjectPolicy::non_clinical(
    "partner_payout_account",
    &[
        FieldRule::last4("account_number"),
        FieldRule::last4("card_number"),
        FieldRule::value("is_active"),
    ],
);
static PRICING: ObjectPolicy = ObjectPolicy::non_clinical(
    "margin_rule",
    &[
        FieldRule::value("margin_percent_bp"),
        FieldRule::value("fee_idr"),
    ],
);
static NOTE: ObjectPolicy =
    ObjectPolicy::clinical_or_document("consultation_note", &[FieldRule::name_only("content")]);
static ACCOUNT: ObjectPolicy = ObjectPolicy::non_clinical("account", &[FieldRule::value("status")]);
static BAD_SECRET_POLICY: ObjectPolicy =
    ObjectPolicy::non_clinical("account", &[FieldRule::name_only("password")]);
static BAD_NIK_POLICY: ObjectPolicy =
    ObjectPolicy::non_clinical("care_recipient", &[FieldRule::text("nik")]);

fn code(s: &str) -> Code {
    Code::new(s).unwrap()
}

fn clock() -> FixedClock {
    FixedClock::new(datetime!(2027-03-01 09:15:00 +7))
}

fn user_ctx() -> AuditContext {
    AuditContext::new(
        CorrelationId::new(),
        Source::component(SourceComponent::PatientApp)
            .ip(IpAddr::V4(Ipv4Addr::new(203, 0, 113, 7)))
            .user_agent(format!(
                "RuangSehat/1.0 (Android 15) Authorization: Bearer {TOKEN} {EMAIL}"
            ))
            .app_version("1.0.3+42"),
    )
}

fn user() -> Actor {
    Actor::account(new_id(), code("patient"), vec![code("account_owner")])
}

fn secret(v: &str) -> Sensitive<String> {
    Sensitive::new(v.to_string())
}

fn assert_clean(rec: &AuditRecord) {
    let text = rec.payload().to_string();
    for forbidden in [
        NIK,
        CARD,
        ACCOUNT_NO,
        PASSWORD,
        OTP,
        TOKEN,
        EMAIL,
        "c2lnbmF0dXJl",
    ] {
        assert!(!text.contains(forbidden), "bocor {forbidden}: {text}");
    }
}

/// Satu event untuk setiap kode E-01–E-18, masing-masing diberi masukan sensitif lewat semua jalur
/// yang tersedia; payload tidak boleh memuat nilai terlarang mana pun.
#[test]
fn every_event_code_payload_is_free_of_forbidden_values() {
    let c = clock();
    for ec in EventCode::ALL {
        let mut ev = AuditEvent::new(
            ec,
            code("sample_action"),
            user(),
            ObjectRef::with_uuid(&CARE_RECIPIENT, new_id()),
        )
        .subject(Subject::care_recipient(new_id()))
        .change(FieldChange::new(
            "nik",
            Some(Value::digits(secret(NIK))),
            Some(Value::digits(secret("3174019999990000"))),
        ))
        .change(FieldChange::new(
            "relationship_label",
            None,
            Some(Value::text(format!(
                "anak {EMAIL} {NIK} password={PASSWORD}"
            ))),
        ))
        .change(FieldChange::new(
            "photo",
            None,
            Some(Value::text(format!("otp {OTP} token {TOKEN}"))),
        ))
        .outcome(if ec == EventCode::E02 {
            // E-02 (login gagal) selalu hasil gagal (F1-T69, PRD §7.1).
            Outcome::failure(code("invalid_user_credentials"))
        } else {
            Outcome::success()
        })
        .context(&user_ctx());
        if ec == EventCode::E17 {
            ev = ev.disclosure(Disclosure::new(
                code("satusehat"),
                vec![code("encounter"), code("patient_identity")],
            ));
        }
        let rec = ev.finish(&c).unwrap();
        assert_clean(&rec);
        let p = rec.payload();
        assert_eq!(p["event_code"], json!(ec.as_str()));
        assert_eq!(p["after_values"]["nik"], json!({"last4": "0000"}));
        assert_eq!(p["before_values"]["nik"], json!({"last4": "8901"}));
        assert!(p["after_values"].get("photo").is_none());
        assert_eq!(
            p["changed_fields"],
            json!(["nik", "relationship_label", "photo"])
        );
    }
}

#[test]
fn card_and_account_numbers_only_last4() {
    let rec = AuditEvent::new(
        EventCode::E14,
        code("payout_account_changed"),
        user(),
        ObjectRef::with_uuid(&PAYOUT_ACCOUNT, new_id()),
    )
    .change(FieldChange::new(
        "account_number",
        Some(Value::digits(secret(ACCOUNT_NO))),
        Some(Value::digits(secret("9876-5432-1000-4321"))),
    ))
    .change(FieldChange::new(
        "card_number",
        None,
        Some(Value::digits(secret(CARD))),
    ))
    .outcome(Outcome::success())
    .context(&user_ctx())
    .finish(&clock())
    .unwrap();
    assert_clean(&rec);
    let p = rec.payload();
    assert_eq!(
        p["before_values"]["account_number"],
        json!({"last4": "0987"})
    );
    assert_eq!(
        p["after_values"]["account_number"],
        json!({"last4": "4321"})
    );
    assert_eq!(p["after_values"]["card_number"], json!({"last4": "1234"}));
}

#[test]
fn full_number_cannot_enter_as_plain_value_or_text() {
    for v in [
        Value::text(NIK),
        Value::Int(3_174_012_345_678_901),
        Value::code("nik").unwrap(),
    ] {
        let err = AuditEvent::new(
            EventCode::E15,
            code("family_member_changed"),
            user(),
            ObjectRef::with_uuid(&CARE_RECIPIENT, new_id()),
        )
        .change(FieldChange::new("nik", None, Some(v)))
        .outcome(Outcome::success())
        .context(&user_ctx())
        .finish(&clock())
        .unwrap_err();
        assert_eq!(err.kind(), AuditErrorKind::ValueNotAllowed);
    }
    // Field nilai biasa menolak teks bebas dan nomor sensitif.
    for v in [Value::text("bebas"), Value::digits(secret(NIK))] {
        let err = AuditEvent::new(
            EventCode::E09,
            code("status_changed"),
            user(),
            ObjectRef::with_uuid(&CARE_RECIPIENT, new_id()),
        )
        .change(FieldChange::new("status", None, Some(v)))
        .outcome(Outcome::success())
        .context(&user_ctx())
        .finish(&clock())
        .unwrap_err();
        assert_eq!(err.kind(), AuditErrorKind::ValueNotAllowed);
    }
}

#[test]
fn secret_field_names_rejected_even_name_only() {
    let err = AuditEvent::new(
        EventCode::E04,
        code("password_changed"),
        user(),
        ObjectRef::with_uuid(&BAD_SECRET_POLICY, new_id()),
    )
    .change(FieldChange::changed("password"))
    .outcome(Outcome::success())
    .context(&user_ctx())
    .finish(&clock())
    .unwrap_err();
    assert_eq!(err.kind(), AuditErrorKind::ForbiddenField);

    let err = AuditEvent::new(
        EventCode::E15,
        code("family_member_changed"),
        user(),
        ObjectRef::with_uuid(&BAD_NIK_POLICY, new_id()),
    )
    .outcome(Outcome::success())
    .context(&user_ctx())
    .finish(&clock())
    .unwrap_err();
    assert_eq!(err.kind(), AuditErrorKind::InvalidPolicy);
}

#[test]
fn credential_change_is_recorded_by_action_name_without_value() {
    let rec = AuditEvent::new(
        EventCode::E04,
        code("password_changed"),
        user(),
        ObjectRef::with_uuid(&ACCOUNT, new_id()),
    )
    .outcome(Outcome::success())
    .context(&user_ctx())
    .finish(&clock())
    .unwrap();
    assert_clean(&rec);
    let p = rec.payload();
    assert_eq!(p["action_name"], json!("password_changed"));
    assert!(p["before_values"].is_null() && p["after_values"].is_null());
}

#[test]
fn field_outside_allowlist_rejected() {
    let err = AuditEvent::new(
        EventCode::E10,
        code("account_changed"),
        user(),
        ObjectRef::with_uuid(&ACCOUNT, new_id()),
    )
    .change(FieldChange::new(
        "email",
        None,
        Some(Value::code("x").unwrap()),
    ))
    .outcome(Outcome::success())
    .context(&user_ctx())
    .finish(&clock())
    .unwrap_err();
    assert_eq!(err.kind(), AuditErrorKind::FieldNotAllowed);
}

#[test]
fn clinical_content_only_field_names_and_version() {
    let rec = AuditEvent::new(
        EventCode::E06,
        code("note_updated"),
        Actor::account(new_id(), code("practitioner"), vec![code("doctor")]),
        ObjectRef::with_uuid(&NOTE, new_id()).version(3),
    )
    .subject(Subject::care_recipient(new_id()))
    .change(FieldChange::new(
        "content",
        Some(Value::text("keluhan lama: demam")),
        Some(Value::text("diagnosis: tifoid")),
    ))
    .outcome(Outcome::success())
    .context(&user_ctx())
    .finish(&clock())
    .unwrap();
    let p = rec.payload();
    let text = p.to_string();
    assert!(
        !text.contains("demam") && !text.contains("tifoid"),
        "{text}"
    );
    assert_eq!(p["changed_fields"], json!(["content"]));
    assert_eq!(p["object_version"], json!(3));
    assert!(p["before_values"].is_null() && p["after_values"].is_null());
}

#[test]
fn required_fields_enforced() {
    let base = || {
        AuditEvent::new(
            EventCode::E09,
            code("status_changed"),
            user(),
            ObjectRef::with_uuid(&ACCOUNT, new_id()),
        )
    };
    // Tanpa konteks (sumber + ID korelasi).
    let e = base().outcome(Outcome::success()).finish(&clock());
    assert_eq!(e.unwrap_err().what(), "context");
    // Tanpa hasil.
    let e = base().context(&user_ctx()).finish(&clock());
    assert_eq!(e.unwrap_err().what(), "outcome");
    // Aksi pengguna tanpa alamat IP.
    let no_ip = AuditContext::new(
        CorrelationId::new(),
        Source::component(SourceComponent::Backoffice),
    );
    let e = base()
        .outcome(Outcome::success())
        .context(&no_ip)
        .finish(&clock());
    assert_eq!(e.unwrap_err().what(), "source_ip");
    // Integrasi rekanan tanpa alamat IP.
    let e = AuditEvent::new(
        EventCode::E09,
        code("status_changed"),
        Actor::partner_system(new_id()),
        ObjectRef::with_uuid(&ACCOUNT, new_id()),
    )
    .outcome(Outcome::success())
    .context(&AuditContext::new(
        CorrelationId::new(),
        Source::component(SourceComponent::Integration),
    ))
    .finish(&clock());
    assert_eq!(e.unwrap_err().what(), "source_ip");
    // Proses otomatis cukup nama komponen.
    let rec = AuditEvent::new(
        EventCode::E16,
        code("anonymized_by_retention"),
        Actor::system(),
        ObjectRef::with_uuid(&ACCOUNT, new_id()),
    )
    .change(FieldChange::new(
        "status",
        Some(Value::code("active").unwrap()),
        Some(Value::code("anonymized").unwrap()),
    ))
    .outcome(Outcome::success())
    .context(&AuditContext::new(
        CorrelationId::new(),
        Source::component(SourceComponent::Worker),
    ))
    .finish(&clock())
    .unwrap();
    assert_eq!(rec.payload()["actor_type"], json!("system"));
    assert!(rec.payload()["source_ip"].is_null());
}

#[test]
fn disclosure_rules_for_e17() {
    let ev = |ec| {
        AuditEvent::new(
            ec,
            code("sent_to_partner"),
            Actor::system(),
            ObjectRef::new(&ACCOUNT, Some(ObjectId::Code(code("export_batch")))),
        )
        .subject(Subject::care_recipient(new_id()))
        .outcome(Outcome::success())
        .context(&AuditContext::new(
            CorrelationId::new(),
            Source::component(SourceComponent::Worker),
        ))
    };
    assert_eq!(
        ev(EventCode::E17).finish(&clock()).unwrap_err().what(),
        "disclosure"
    );
    let d = Disclosure::new(code("partner_system"), vec![]);
    assert_eq!(
        ev(EventCode::E17)
            .disclosure(d)
            .finish(&clock())
            .unwrap_err()
            .what(),
        "disclosure_data_categories"
    );
    let d = Disclosure::new(code("partner_system"), vec![code("patient_identity")]);
    assert_eq!(
        ev(EventCode::E09)
            .disclosure(d.clone())
            .finish(&clock())
            .unwrap_err()
            .kind(),
        AuditErrorKind::InvalidCombination
    );
    let rec = ev(EventCode::E17).disclosure(d).finish(&clock()).unwrap();
    assert_eq!(
        rec.payload()["disclosure_recipient"],
        json!("partner_system")
    );
    assert_eq!(
        rec.payload()["disclosure_data_categories"],
        json!(["patient_identity"])
    );
    assert_eq!(rec.payload()["object_id"], json!("export_batch"));
}

#[test]
fn failure_requires_reason_and_payload_has_prd_7_2_fields() {
    let c = clock();
    let ctx = user_ctx();
    let rec = AuditEvent::new(
        EventCode::E05,
        code("chat_history_read"),
        Actor::account(new_id(), code("practitioner"), vec![code("doctor")]),
        ObjectRef::with_uuid(&ACCOUNT, new_id()),
    )
    .subject(Subject::care_recipient(new_id()))
    .tenant(new_id())
    .outcome(Outcome::failure(code("access_denied")))
    .context(&ctx)
    .finish(&c)
    .unwrap();
    let p = rec.payload();
    assert_eq!(p["outcome"], json!("failure"));
    assert_eq!(p["outcome_reason_code"], json!("access_denied"));
    assert_eq!(
        p["event_id"],
        json!(rec.event_id().hyphenated().to_string())
    );
    assert_eq!(rec.event_id().get_version_num(), 7);
    assert_eq!(p["occurred_at"], json!("2027-03-01T02:15:00Z"));
    assert_eq!(p["source_component"], json!("patient_app"));
    assert_eq!(p["source_ip"], json!("203.0.113.7"));
    assert_eq!(p["app_version"], json!("1.0.3+42"));
    assert_eq!(p["actor_account_type"], json!("practitioner"));
    assert_eq!(p["actor_roles"], json!(["doctor"]));
    assert_eq!(p["subjects"].as_array().map(Vec::len), Some(1));
    assert_eq!(p["payload_version"], json!(1));
    for key in [
        "event_id",
        "occurred_at",
        "event_code",
        "action_name",
        "actor_type",
        "object_type",
        "source_component",
        "outcome",
        "correlation_id",
    ] {
        assert!(!p[key].is_null(), "field wajib {key} kosong");
    }
    // Field yang tidak dikenal DD B1 tidak muncul.
    for key in ["password", "token", "otp", "pin", "nik"] {
        assert!(p.get(key).is_none(), "{key}");
    }
}

#[test]
fn invalid_app_version_and_codes_rejected() {
    let ctx = AuditContext::new(
        CorrelationId::new(),
        Source::component(SourceComponent::PatientApp)
            .ip(IpAddr::V4(Ipv4Addr::LOCALHOST))
            .app_version("1.0 <script>"),
    );
    let e = AuditEvent::new(
        EventCode::E01,
        code("login"),
        user(),
        ObjectRef::with_uuid(&ACCOUNT, new_id()),
    )
    .outcome(Outcome::success())
    .context(&ctx)
    .finish(&clock());
    assert_eq!(e.unwrap_err().what(), "app_version");
    assert!(Code::new(EMAIL).is_err());
    assert!(Code::new(NIK).is_err());
    assert!(Code::new(&format!("x{NIK}")).is_err());
}

#[test]
fn money_values_recorded_as_integers() {
    let rec = AuditEvent::new(
        EventCode::E14,
        code("margin_changed"),
        user(),
        ObjectRef::with_uuid(&PRICING, new_id()),
    )
    .change(FieldChange::new(
        "margin_percent_bp",
        Some(Value::Int(1000)),
        Some(Value::Int(1250)),
    ))
    .change(FieldChange::new(
        "fee_idr",
        None,
        Some(Value::Idr(Idr::new(15_000))),
    ))
    .outcome(Outcome::success())
    .context(&user_ctx())
    .finish(&clock())
    .unwrap();
    let p = rec.payload();
    assert_eq!(p["before_values"]["margin_percent_bp"], json!(1000));
    assert_eq!(p["after_values"]["fee_idr"], json!(15000));
}

#[test]
fn duplicate_change_and_subjects_handled() {
    let s = new_id();
    let e = AuditEvent::new(
        EventCode::E09,
        code("status_changed"),
        user(),
        ObjectRef::with_uuid(&ACCOUNT, new_id()),
    )
    .change(FieldChange::new(
        "status",
        None,
        Some(Value::code("a").unwrap()),
    ))
    .change(FieldChange::new(
        "status",
        None,
        Some(Value::code("b").unwrap()),
    ))
    .outcome(Outcome::success())
    .context(&user_ctx())
    .finish(&clock());
    assert_eq!(e.unwrap_err().kind(), AuditErrorKind::InvalidCombination);
    let rec = AuditEvent::new(
        EventCode::E12,
        code("report_exported"),
        user(),
        ObjectRef::with_uuid(&ACCOUNT, new_id()),
    )
    .subject(Subject::care_recipient(s))
    .subject(Subject::care_recipient(s))
    .subject(Subject::rme_patient(new_id()))
    .outcome(Outcome::success())
    .context(&user_ctx())
    .finish(&clock())
    .unwrap();
    assert_eq!(rec.payload()["subjects"].as_array().map(Vec::len), Some(2));
}

#[test]
fn debug_output_of_event_and_record_does_not_leak() {
    let ev = AuditEvent::new(
        EventCode::E15,
        code("family_member_changed"),
        user(),
        ObjectRef::with_uuid(&CARE_RECIPIENT, new_id()),
    )
    .subject(Subject::care_recipient(new_id()))
    .change(FieldChange::new(
        "nik",
        None,
        Some(Value::digits(secret(NIK))),
    ))
    .change(FieldChange::new(
        "relationship_label",
        None,
        Some(Value::text(EMAIL)),
    ))
    .outcome(Outcome::success())
    .context(&user_ctx());
    let dbg_ev = format!("{ev:?}");
    let rec = ev.finish(&clock()).unwrap();
    let dbg_rec = format!("{rec:?}");
    for out in [dbg_ev, dbg_rec] {
        for forbidden in [NIK, EMAIL, TOKEN, "203.0.113.7"] {
            assert!(!out.contains(forbidden), "bocor {forbidden}: {out}");
        }
    }
}

static CONTACT: ObjectPolicy = ObjectPolicy::non_clinical(
    "contact_preference",
    &[FieldRule::value("phone"), FieldRule::text("note")],
);

/// Probe review keamanan F1-T22 BLOKIR-1: nomor yang masuk lewat nilai (bukan nama field) ditolak atau
/// disamarkan.
#[test]
fn numbers_smuggled_through_values_are_blocked() {
    let base = |change: FieldChange| {
        AuditEvent::new(
            EventCode::E10,
            code("contact_changed"),
            user(),
            ObjectRef::with_uuid(&CONTACT, new_id()),
        )
        .change(change)
        .outcome(Outcome::success())
        .context(&user_ctx())
        .finish(&clock())
    };
    // Bilangan besar di field Value dan Text.
    for (field, n) in [
        ("phone", 3_174_012_345_678_901_i64),
        ("phone", 81_234_567_890),
        ("note", 4_111_111_111_111_111),
    ] {
        let e = base(FieldChange::new(field, None, Some(Value::Int(n)))).unwrap_err();
        assert_eq!(e.kind(), AuditErrorKind::ValueNotAllowed, "{field} {n}");
    }
    // Kode dengan digit dipecah pemisah.
    assert!(Value::code("n_31740_12345_67890_1").is_err());
    assert!(Code::new("otp_48291").is_err());
    // Teks dengan OTP/PIN dan nomor dipecah.
    for t in [
        "otp 482913",
        "PIN: 4321",
        "kode 1234 5678",
        "3174 01 23 45 67 89 01",
    ] {
        let rec = base(FieldChange::new("note", None, Some(Value::text(t)))).unwrap();
        let out = rec.payload()["after_values"]["note"].to_string();
        assert!(
            out.chars().filter(char::is_ascii_digit).count() <= 3,
            "{t} -> {out}"
        );
    }
    // Versi objek dan versi aplikasi yang memuat nomor.
    let e = AuditEvent::new(
        EventCode::E06,
        code("note_updated"),
        user(),
        ObjectRef::with_uuid(&NOTE, new_id()).version(1_234_567_890),
    )
    .outcome(Outcome::success())
    .context(&user_ctx())
    .finish(&clock())
    .unwrap_err();
    assert_eq!(e.what(), "object_version");
    let ctx = AuditContext::new(
        CorrelationId::new(),
        Source::component(SourceComponent::PatientApp)
            .ip(IpAddr::V4(Ipv4Addr::LOCALHOST))
            .app_version("3174.0123.4567.8901"),
    );
    let e = AuditEvent::new(
        EventCode::E01,
        code("login"),
        user(),
        ObjectRef::with_uuid(&ACCOUNT, new_id()),
    )
    .outcome(Outcome::success())
    .context(&ctx)
    .finish(&clock())
    .unwrap_err();
    assert_eq!(e.what(), "app_version");
    // User agent: deret angka panjang disamarkan.
    let ctx = AuditContext::new(
        CorrelationId::new(),
        Source::component(SourceComponent::PatientApp)
            .ip(IpAddr::V4(Ipv4Addr::LOCALHOST))
            .user_agent("App/1.0 rek 1234567890 hp 081234567"),
    );
    let rec = AuditEvent::new(
        EventCode::E01,
        code("login"),
        user(),
        ObjectRef::with_uuid(&ACCOUNT, new_id()),
    )
    .outcome(Outcome::success())
    .context(&ctx)
    .finish(&clock())
    .unwrap();
    let ua = rec.payload()["user_agent"].to_string();
    assert!(
        !ua.contains("1234567890") && !ua.contains("081234567"),
        "{ua}"
    );
    assert!(ua.contains("App/1.0"), "{ua}");
}

/// F1-T69 (US-1501 AC1, TC-US-1501-4): field wajib per kode event ditegakkan saat event dibentuk, sehingga
/// event yang kurang tidak pernah ditulis dan aksi pemanggil batal.
#[test]
fn per_event_code_requirements_reject_incomplete_events() {
    let c = clock();
    let ev = |ec: EventCode| {
        AuditEvent::new(
            ec,
            code("sample_action"),
            user(),
            ObjectRef::with_uuid(&ACCOUNT, new_id()),
        )
        .context(&user_ctx())
    };
    let status = || {
        FieldChange::new(
            "status",
            Some(Value::code("pending").unwrap()),
            Some(Value::code("approved").unwrap()),
        )
    };
    // E-07 tanpa subjek data.
    let e = ev(EventCode::E07)
        .change(status())
        .outcome(Outcome::success())
        .finish(&c)
        .unwrap_err();
    assert_eq!(
        (e.kind(), e.what()),
        (AuditErrorKind::MissingField, "subjects")
    );
    // E-07 tanpa nilai berubah.
    let e = ev(EventCode::E07)
        .subject(Subject::care_recipient(new_id()))
        .outcome(Outcome::success())
        .finish(&c)
        .unwrap_err();
    assert_eq!(e.what(), "changed_fields");
    // E-05 yang membuka data tanpa subjek; penolakan tanpa subjek boleh.
    let e = ev(EventCode::E05)
        .outcome(Outcome::success())
        .finish(&c)
        .unwrap_err();
    assert_eq!(e.what(), "subjects");
    ev(EventCode::E05)
        .outcome(Outcome::failure(code("access_denied")))
        .finish(&c)
        .unwrap();
    // E-02 selalu gagal, E-01 selalu sukses.
    let e = ev(EventCode::E02)
        .outcome(Outcome::success())
        .finish(&c)
        .unwrap_err();
    assert_eq!(
        (e.kind(), e.what()),
        (AuditErrorKind::InvalidCombination, "outcome")
    );
    let e = ev(EventCode::E01)
        .outcome(Outcome::failure(code("invalid_user_credentials")))
        .finish(&c)
        .unwrap_err();
    assert_eq!(e.what(), "outcome");
    // E-10/E-11/E-16 berhasil wajib before/after; hasil gagal tidak.
    for ec in [EventCode::E10, EventCode::E11, EventCode::E16] {
        let e = ev(ec).outcome(Outcome::success()).finish(&c).unwrap_err();
        assert_eq!(e.what(), "changed_fields", "{ec}");
        ev(ec)
            .outcome(Outcome::failure(code("transition_conflict")))
            .finish(&c)
            .unwrap();
        ev(ec)
            .change(status())
            .outcome(Outcome::success())
            .finish(&c)
            .unwrap();
    }
    // ID objek wajib kecuali autentikasi IdP dengan subjek tidak dikenal (aktor sistem).
    let worker = AuditContext::new(
        CorrelationId::new(),
        Source::component(SourceComponent::Worker),
    );
    let e = AuditEvent::new(
        EventCode::E10,
        code("account_changed"),
        user(),
        ObjectRef::new(&ACCOUNT, None),
    )
    .change(status())
    .outcome(Outcome::success())
    .context(&user_ctx())
    .finish(&c)
    .unwrap_err();
    assert_eq!(e.what(), "object_id");
    let rec = AuditEvent::new(
        EventCode::E02,
        code("login_failed"),
        Actor::system(),
        ObjectRef::new(&ACCOUNT, None),
    )
    .outcome(Outcome::failure(code("user_not_found")))
    .context(&worker)
    .finish(&c)
    .unwrap();
    assert!(rec.payload()["object_id"].is_null());
}

/// F1-T69 (US-1501 AC1–AC2): setiap payload yang lolos `finish` juga lolos pemeriksa yang dipakai test
/// integrasi atas payload tersimpan; payload yang dirusak sesudahnya terdeteksi.
#[test]
fn finished_payloads_pass_check_payload() {
    let rec = AuditEvent::new(
        EventCode::E14,
        code("payout_account_changed"),
        user(),
        ObjectRef::with_uuid(&PAYOUT_ACCOUNT, new_id()),
    )
    .change(FieldChange::new(
        "account_number",
        None,
        Some(Value::digits(secret(ACCOUNT_NO))),
    ))
    .outcome(Outcome::success())
    .context(&user_ctx())
    .finish(&clock())
    .unwrap();
    assert_eq!(audit::check_payload(rec.payload()), []);
    let mut tampered = rec.payload().clone();
    tampered["after_values"]["account_number"] = json!(ACCOUNT_NO);
    assert!(!audit::check_payload(&tampered).is_empty());
    let mut tampered = rec.payload().clone();
    tampered["otp"] = json!(OTP);
    assert!(!audit::check_payload(&tampered).is_empty());
}
