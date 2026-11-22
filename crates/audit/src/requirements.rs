//! Kelengkapan field per event (US-1501 AC1, PRD §7.2) dan larangan isi (US-1501 AC2, PRD §7.3) yang
//! diperiksa atas payload `audit_outbox` yang sudah jadi (F1-T69).
//!
//! [`check_payload`] dipanggil dua kali:
//! - oleh [`crate::AuditEvent::finish`] atas payload yang baru dibentuk, sehingga event yang kurang field
//!   wajib untuk kodenya ditolak sebelum ditulis dan aksi pemanggil batal (ADR-009 butir 2, KEPUTUSAN-15);
//! - oleh test integrasi modul atas payload yang dibaca dari `audit_outbox` (pemancar yang terdaftar di
//!   `audit-event-coverage.toml`, dijaga aturan arch-check `audit-coverage`).
//!
//! Aturan per kode (PRD §7.1 dibaca bersama §7.2; kolom "Wajib" §7.2):
//! - Semua event: ID event, waktu, aktor, aksi (kode + nama), jenis objek, komponen sumber, hasil (gagal
//!   wajib kode alasan), ID korelasi. Alamat IP wajib kecuali aktor proses otomatis (`system`).
//! - ID objek wajib, kecuali event autentikasi E-01–E-04 dari IdP yang subjeknya tidak dikenal aplikasi
//!   (aktor `system`, mis. login dengan email yang tidak terdaftar): ID itu tidak ada, dan email tidak
//!   boleh menggantikannya (§7.3).
//! - E-01 selalu sukses; E-02 (login gagal, termasuk akun terkunci) selalu gagal dengan kode alasan.
//! - Subjek data wajib bila definisi event §7.1 selalu menyangkut data pribadi pasien: E-05 yang berhasil
//!   (data benar-benar dibuka; percobaan yang ditolak tidak membuka data apa pun), E-06, E-07, E-13, E-15,
//!   E-17.
//! - Daftar field berubah (before/after) wajib untuk event perubahan yang berhasil: E-06, E-07, E-10,
//!   E-11, E-14, E-16. E-04 tidak: nilai kredensial tidak pernah dicatat, perubahan dinyatakan lewat nama
//!   aksi (§7.1 E-04).
//! - E-17 wajib tujuan dan jenis data (ditegakkan juga saat event disusun).
//!
//! Larangan (§7.3), sebagai lapis kedua di belakang daftar izin dan masking pustaka:
//! - Payload hanya berisi kunci bentuk versi 1; kunci lain (mis. `password`) ditolak.
//! - Nama field di `changed_fields`/`before_values`/`after_values` bukan nama terlarang
//!   ([`crate::is_forbidden_field_name`]); field nomor identitas/kartu/rekening hanya `{"last4": "dddd"}`.
//! - Di nilai before/after, kode aksi, jenis objek, dan kode alasan: teks paling banyak 4 digit seluruhnya
//!   (UUID dan waktu RFC 3339 UTC utuh dikecualikan), bilangan paling besar [`crate::IDR_MAX_ABS`]. NIK
//!   (16 digit), nomor kartu, dan nomor rekening lengkap karena itu tidak dapat lolos sebagai teks, juga bila
//!   dipecah pemisah.
//! - User agent: deret digit paling panjang 5 (lebih dari itu sudah disamarkan pustaka), versi aplikasi
//!   paling banyak 8 digit seluruhnya.

use crate::code::{EventCode, is_valid_code};
use crate::error::AuditErrorKind;
use crate::event::{OBJECT_VERSION_MAX, PAYLOAD_VERSION, USER_AGENT_MAX_CHARS};
use crate::policy::{is_forbidden_field_name, is_number_field_name};
use crate::value::IDR_MAX_ABS;
use foundation::Uuid;
use serde_json::{Map, Value as Json};
use std::fmt;
use std::net::IpAddr;

/// Satu pelanggaran: field dan aturan (teks statis dari kode, tidak pernah nilai payload).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Violation {
    field: &'static str,
    rule: &'static str,
    kind: AuditErrorKind,
}

impl Violation {
    const fn new(field: &'static str, rule: &'static str, kind: AuditErrorKind) -> Self {
        Self { field, rule, kind }
    }

    /// Nama field payload.
    pub fn field(&self) -> &'static str {
        self.field
    }

    /// Aturan yang dilanggar (mis. `required`, `required_for_event_code`, `digits`).
    pub fn rule(&self) -> &'static str {
        self.rule
    }

    /// Jenis galat bila pelanggaran ini menolak event di [`crate::AuditEvent::finish`].
    pub fn kind(&self) -> AuditErrorKind {
        self.kind
    }
}

impl fmt::Display for Violation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.field, self.rule)
    }
}

/// Kunci payload versi 1 (DD A19.1, `event::Payload`).
const PAYLOAD_KEYS: &[&str] = &[
    "payload_version",
    "event_id",
    "occurred_at",
    "event_code",
    "action_name",
    "actor_type",
    "actor_account_id",
    "actor_account_type",
    "actor_roles",
    "actor_partner_id",
    "object_type",
    "object_id",
    "object_version",
    "subjects",
    "tenant_id",
    "source_component",
    "source_ip",
    "user_agent",
    "app_version",
    "changed_fields",
    "before_values",
    "after_values",
    "disclosure_recipient",
    "disclosure_data_categories",
    "outcome",
    "outcome_reason_code",
    "correlation_id",
];

/// Digit seluruhnya yang boleh ada di teks nilai before/after dan kode (kode ≤ 4 digit seluruhnya, teks
/// `Text` ≤ 3 digit setelah disaring pustaka).
const VALUE_MAX_DIGITS: usize = 4;
/// Deret digit terpanjang di user agent (deret lebih panjang sudah disamarkan pustaka).
const USER_AGENT_MAX_DIGIT_RUN: usize = 5;
/// Digit seluruhnya di versi aplikasi.
const APP_VERSION_MAX_DIGITS: usize = 8;

/// Subjek data wajib untuk kode ini (lihat dokumentasi modul).
fn subject_required(code: EventCode, success: bool) -> bool {
    match code {
        EventCode::E05 => success,
        EventCode::E06 | EventCode::E07 | EventCode::E13 | EventCode::E15 | EventCode::E17 => true,
        _ => false,
    }
}

/// Daftar field berubah wajib untuk kode ini (lihat dokumentasi modul).
fn changes_required(code: EventCode, success: bool) -> bool {
    success
        && matches!(
            code,
            EventCode::E06
                | EventCode::E07
                | EventCode::E10
                | EventCode::E11
                | EventCode::E14
                | EventCode::E16
        )
}

/// ID objek boleh kosong hanya untuk event autentikasi IdP dengan subjek tidak dikenal.
fn object_id_optional(code: EventCode, actor_type: &str) -> bool {
    actor_type == "system"
        && matches!(
            code,
            EventCode::E01 | EventCode::E02 | EventCode::E03 | EventCode::E04
        )
}

fn event_code(s: &str) -> Option<EventCode> {
    EventCode::ALL.into_iter().find(|c| c.as_str() == s)
}

fn is_uuid(v: Option<&Json>) -> bool {
    v.and_then(Json::as_str)
        .is_some_and(|s| s.len() == 36 && Uuid::parse_str(s).is_ok())
}

fn is_code(v: Option<&Json>) -> bool {
    v.and_then(Json::as_str).is_some_and(is_valid_code)
}

fn is_null(v: Option<&Json>) -> bool {
    v.is_none_or(Json::is_null)
}

fn non_empty_str(v: Option<&Json>) -> bool {
    v.and_then(Json::as_str)
        .is_some_and(|s| !s.trim().is_empty())
}

/// Memeriksa payload `audit_outbox` terhadap field wajib PRD §7.2 per kode event dan larangan §7.3.
/// Mengembalikan semua pelanggaran (kosong = lengkap dan bersih).
pub fn check_payload(payload: &Json) -> Vec<Violation> {
    use AuditErrorKind::{ForbiddenField, InvalidCombination, InvalidValue, MissingField};
    let mut out = Vec::new();
    let Some(p) = payload.as_object() else {
        return vec![Violation::new("payload", "not_object", InvalidValue)];
    };
    let mut push = |field, rule, kind| out.push(Violation::new(field, rule, kind));

    for key in p.keys() {
        if !PAYLOAD_KEYS.contains(&key.as_str()) {
            push("payload", "unknown_key", ForbiddenField);
        }
    }
    if p.get("payload_version").and_then(Json::as_u64) != Some(u64::from(PAYLOAD_VERSION)) {
        push("payload_version", "unsupported", InvalidValue);
    }

    // Siapa, apa, kapan.
    if !is_uuid(p.get("event_id")) {
        push("event_id", "required_uuid", MissingField);
    }
    if !p
        .get("occurred_at")
        .and_then(Json::as_str)
        .is_some_and(|s| s.len() >= 20 && s.ends_with('Z'))
    {
        push("occurred_at", "required_utc", MissingField);
    }
    let code = p
        .get("event_code")
        .and_then(Json::as_str)
        .and_then(event_code);
    if code.is_none() {
        push("event_code", "required_prd_7_1", MissingField);
    }
    if !is_code(p.get("action_name")) {
        push("action_name", "required_code", MissingField);
    }
    let actor_type = p.get("actor_type").and_then(Json::as_str).unwrap_or("");
    match actor_type {
        "account" => {
            if !is_uuid(p.get("actor_account_id")) {
                push("actor_account_id", "required_for_account", MissingField);
            }
            if !is_code(p.get("actor_account_type")) {
                push("actor_account_type", "required_for_account", MissingField);
            }
        }
        "partner_system" => {
            if !is_uuid(p.get("actor_partner_id")) {
                push(
                    "actor_partner_id",
                    "required_for_partner_system",
                    MissingField,
                );
            }
        }
        "system" | "ruang_sehat_system" => {
            if !is_null(p.get("actor_account_id")) {
                push("actor_account_id", "not_for_system", InvalidCombination);
            }
        }
        _ => push("actor_type", "required", MissingField),
    }
    match p.get("actor_roles").and_then(Json::as_array) {
        Some(roles) if roles.iter().all(|r| is_code(Some(r))) => {}
        _ => push("actor_roles", "codes", InvalidValue),
    }

    // Objek.
    if !is_code(p.get("object_type")) {
        push("object_type", "required_code", MissingField);
    }
    let object_id = p.get("object_id");
    if is_null(object_id) {
        if !code.is_some_and(|c| object_id_optional(c, actor_type)) {
            push("object_id", "required", MissingField);
        }
    } else if !(is_uuid(object_id) || is_code(object_id)) {
        push("object_id", "uuid_or_code", InvalidValue);
    }
    if let Some(v) = p.get("object_version").filter(|v| !v.is_null())
        && !v
            .as_i64()
            .is_some_and(|n| (0..=i64::from(OBJECT_VERSION_MAX)).contains(&n))
    {
        push("object_version", "range", InvalidValue);
    }

    // Subjek data.
    let subjects = p.get("subjects").and_then(Json::as_array);
    match subjects {
        Some(list)
            if list.iter().all(|s| {
                is_uuid(s.get("subject_id"))
                    && matches!(
                        s.get("subject_kind").and_then(Json::as_str),
                        Some("care_recipient" | "rme_patient")
                    )
            }) => {}
        _ => push("subjects", "shape", InvalidValue),
    }

    // Dari mana.
    if !is_code(p.get("source_component")) {
        push("source_component", "required", MissingField);
    }
    match p.get("source_ip").and_then(Json::as_str) {
        Some(ip) if ip.parse::<IpAddr>().is_ok() => {}
        Some(_) => push("source_ip", "ip_address", InvalidValue),
        None if actor_type == "system" => {}
        None => push(
            "source_ip",
            "required_for_user_and_integration",
            MissingField,
        ),
    }
    if let Some(ua) = p.get("user_agent").filter(|v| !v.is_null()) {
        match ua.as_str() {
            Some(s)
                if s.chars().count() <= USER_AGENT_MAX_CHARS
                    && longest_digit_run(s) <= USER_AGENT_MAX_DIGIT_RUN => {}
            _ => push("user_agent", "digits", ForbiddenField),
        }
    }
    if let Some(av) = p.get("app_version").filter(|v| !v.is_null()) {
        match av.as_str() {
            Some(s) if s.chars().filter(char::is_ascii_digit).count() <= APP_VERSION_MAX_DIGITS => {
            }
            _ => push("app_version", "digits", ForbiddenField),
        }
    }

    // Hasil dan korelasi.
    let outcome = p.get("outcome").and_then(Json::as_str);
    let success = outcome == Some("success");
    match outcome {
        Some("success") => {
            if !is_null(p.get("outcome_reason_code")) && !is_code(p.get("outcome_reason_code")) {
                push("outcome_reason_code", "code", InvalidValue);
            }
        }
        Some("failure") => {
            if !is_code(p.get("outcome_reason_code")) {
                push("outcome_reason_code", "required_for_failure", MissingField);
            }
        }
        _ => push("outcome", "required", MissingField),
    }
    if !non_empty_str(p.get("correlation_id")) {
        push("correlation_id", "required", MissingField);
    }

    // Aturan per kode event.
    if let Some(c) = code {
        if c == EventCode::E01 && outcome != Some("success") {
            push("outcome", "e_01_is_success", InvalidCombination);
        }
        if c == EventCode::E02 && outcome != Some("failure") {
            push("outcome", "e_02_is_failure", InvalidCombination);
        }
        if subject_required(c, success) && subjects.is_none_or(Vec::is_empty) {
            push("subjects", "required_for_event_code", MissingField);
        }
        let changed = p.get("changed_fields").and_then(Json::as_array);
        if changes_required(c, success) && changed.is_none_or(Vec::is_empty) {
            push("changed_fields", "required_for_event_code", MissingField);
        }
        let has_disclosure = non_empty_str(p.get("disclosure_recipient"));
        let categories = p
            .get("disclosure_data_categories")
            .and_then(Json::as_array)
            .is_some_and(|a| !a.is_empty() && a.iter().all(|x| is_code(Some(x))));
        match (c == EventCode::E17, has_disclosure && categories) {
            (true, false) => push("disclosure_recipient", "required_for_e_17", MissingField),
            (false, _) if has_disclosure => {
                push("disclosure_recipient", "only_for_e_17", InvalidCombination);
            }
            _ => {}
        }
    }

    // Larangan §7.3 pada kode bebas pemanggil.
    for key in ["action_name", "outcome_reason_code", "object_type"] {
        if let Some(s) = p.get(key).and_then(Json::as_str)
            && !string_is_clean(s)
        {
            push(key, "digits", ForbiddenField);
        }
    }
    if let Some(list) = p.get("changed_fields").filter(|v| !v.is_null()) {
        match list.as_array() {
            Some(names) => {
                for n in names {
                    match n.as_str() {
                        Some(name) if is_valid_code(name) && !is_forbidden_field_name(name) => {}
                        _ => push(
                            "changed_fields",
                            "forbidden_or_invalid_name",
                            ForbiddenField,
                        ),
                    }
                }
            }
            None => push("changed_fields", "shape", InvalidValue),
        }
    }
    for key in ["before_values", "after_values"] {
        if let Some(v) = p.get(key).filter(|v| !v.is_null()) {
            match v.as_object() {
                Some(map) => check_values(key, map, &mut out),
                None => out.push(Violation::new(key, "shape", InvalidValue)),
            }
        }
    }
    out
}

fn check_values(key: &'static str, map: &Map<String, Json>, out: &mut Vec<Violation>) {
    use AuditErrorKind::{ForbiddenField, ValueNotAllowed};
    for (name, value) in map {
        if !is_valid_code(name) || is_forbidden_field_name(name) {
            out.push(Violation::new(
                key,
                "forbidden_or_invalid_name",
                ForbiddenField,
            ));
            continue;
        }
        if is_number_field_name(name) {
            if !is_last4(value) {
                out.push(Violation::new(
                    key,
                    "number_field_not_last4",
                    ValueNotAllowed,
                ));
            }
            continue;
        }
        if !value_is_clean(value) {
            out.push(Violation::new(key, "digits", ForbiddenField));
        }
    }
}

/// `{"last4": "dddd"}` (bentuk keluaran field `Last4`).
fn is_last4(v: &Json) -> bool {
    v.as_object().is_some_and(|o| {
        o.len() == 1
            && o.get("last4")
                .and_then(Json::as_str)
                .is_some_and(|s| s.len() == 4 && s.bytes().all(|b| b.is_ascii_digit()))
    })
}

fn value_is_clean(v: &Json) -> bool {
    match v {
        Json::Null | Json::Bool(_) => true,
        Json::Number(n) => match (n.as_i64(), n.as_u64()) {
            (Some(i), _) => i.unsigned_abs() <= IDR_MAX_ABS,
            (None, Some(u)) => u <= IDR_MAX_ABS,
            // Pecahan tidak dihasilkan pustaka.
            (None, None) => false,
        },
        Json::String(s) => string_is_clean(s),
        Json::Array(a) => a.iter().all(value_is_clean),
        Json::Object(o) => o.values().all(value_is_clean),
    }
}

/// Teks nilai/kode: waktu RFC 3339 UTC utuh, atau paling banyak 4 digit seluruhnya di luar UUID.
fn string_is_clean(s: &str) -> bool {
    is_utc_timestamp(s)
        || strip_uuids(s).bytes().filter(u8::is_ascii_digit).count() <= VALUE_MAX_DIGITS
}

/// `YYYY-MM-DDThh:mm:ss[.f{1,9}]Z` (bentuk `rfc3339` pustaka untuk nilai waktu).
fn is_utc_timestamp(s: &str) -> bool {
    let b = s.as_bytes();
    let digit = |i: usize| b.get(i).is_some_and(u8::is_ascii_digit);
    let fixed = b.len() >= 20
        && [0, 1, 2, 3, 5, 6, 8, 9, 11, 12, 14, 15, 17, 18]
            .iter()
            .all(|&i| digit(i))
        && b.get(4) == Some(&b'-')
        && b.get(7) == Some(&b'-')
        && b.get(10) == Some(&b'T')
        && b.get(13) == Some(&b':')
        && b.get(16) == Some(&b':')
        && b.last() == Some(&b'Z');
    if !fixed {
        return false;
    }
    match b.get(19..b.len() - 1) {
        Some([]) => true,
        Some([b'.', frac @ ..]) => {
            (1..=9).contains(&frac.len()) && frac.iter().all(u8::is_ascii_digit)
        }
        _ => false,
    }
}

/// Deret digit terpanjang di `s` (untuk user agent).
fn longest_digit_run(s: &str) -> usize {
    let mut longest = 0;
    let mut run = 0;
    for c in s.chars() {
        if c.is_ascii_digit() {
            run += 1;
            longest = longest.max(run);
        } else {
            run = 0;
        }
    }
    longest
}

/// Mengganti setiap UUID berbentuk 8-4-4-4-12 heksadesimal dengan spasi.
fn strip_uuids(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(b.len());
    let mut i = 0;
    while i < b.len() {
        if let Some(w) = b.get(i..i + 36)
            && is_uuid_bytes(w)
        {
            out.push(b' ');
            i += 36;
        } else {
            out.push(b.get(i).copied().unwrap_or(b' '));
            i += 1;
        }
    }
    String::from_utf8(out).unwrap_or_default()
}

fn is_uuid_bytes(w: &[u8]) -> bool {
    w.iter().enumerate().all(|(k, c)| match k {
        8 | 13 | 18 | 23 => *c == b'-',
        _ => c.is_ascii_hexdigit(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn base() -> Json {
        json!({
            "payload_version": 1,
            "event_id": "0190f5c2-7c1a-7000-8000-000000000001",
            "occurred_at": "2027-03-01T02:15:00Z",
            "event_code": "e_09",
            "action_name": "order_status_changed",
            "actor_type": "account",
            "actor_account_id": "0190f5c2-7c1a-7000-8000-000000000002",
            "actor_account_type": "patient",
            "actor_roles": [],
            "actor_partner_id": null,
            "object_type": "patient_order",
            "object_id": "0190f5c2-7c1a-7000-8000-000000000003",
            "object_version": null,
            "subjects": [],
            "tenant_id": null,
            "source_component": "patient_app",
            "source_ip": "203.0.113.7",
            "user_agent": null,
            "app_version": null,
            "changed_fields": null,
            "before_values": null,
            "after_values": null,
            "disclosure_recipient": null,
            "disclosure_data_categories": null,
            "outcome": "success",
            "outcome_reason_code": null,
            "correlation_id": "c-1"
        })
    }

    fn rules(p: &Json) -> Vec<(&'static str, &'static str)> {
        check_payload(p)
            .iter()
            .map(|v| (v.field(), v.rule()))
            .collect()
    }

    fn with(mut p: Json, key: &str, v: Json) -> Json {
        if let Some(o) = p.as_object_mut() {
            o.insert(key.to_string(), v);
        }
        p
    }

    #[test]
    fn complete_payload_passes() {
        assert_eq!(rules(&base()), []);
    }

    #[test]
    fn each_mandatory_field_is_required() {
        for (key, field) in [
            ("event_id", "event_id"),
            ("occurred_at", "occurred_at"),
            ("event_code", "event_code"),
            ("action_name", "action_name"),
            ("actor_type", "actor_type"),
            ("actor_account_id", "actor_account_id"),
            ("actor_account_type", "actor_account_type"),
            ("object_type", "object_type"),
            ("object_id", "object_id"),
            ("source_component", "source_component"),
            ("source_ip", "source_ip"),
            ("outcome", "outcome"),
            ("correlation_id", "correlation_id"),
        ] {
            let r = rules(&with(base(), key, Json::Null));
            assert!(r.iter().any(|(f, _)| *f == field), "{key}: {r:?}");
        }
        let r = rules(&with(base(), "outcome", json!("failure")));
        assert_eq!(r, [("outcome_reason_code", "required_for_failure")]);
    }

    #[test]
    fn system_actor_needs_component_only() {
        let p = with(base(), "actor_type", json!("system"));
        let p = with(p, "actor_account_id", Json::Null);
        let p = with(p, "actor_account_type", Json::Null);
        let p = with(p, "source_ip", Json::Null);
        assert_eq!(rules(&p), []);
        // ID objek tetap wajib untuk proses otomatis di luar autentikasi IdP.
        assert_eq!(
            rules(&with(p.clone(), "object_id", Json::Null)),
            [("object_id", "required")]
        );
        let p = with(p, "event_code", json!("e_02"));
        let p = with(p, "outcome", json!("failure"));
        let p = with(p, "outcome_reason_code", json!("user_not_found"));
        assert_eq!(rules(&with(p, "object_id", Json::Null)), []);
    }

    #[test]
    fn per_code_rules() {
        let failed = |p: Json| {
            let p = with(p, "outcome", json!("failure"));
            with(p, "outcome_reason_code", json!("access_denied"))
        };
        assert_eq!(
            rules(&with(base(), "event_code", json!("e_02"))),
            [("outcome", "e_02_is_failure")]
        );
        assert_eq!(
            rules(&failed(with(base(), "event_code", json!("e_01")))),
            [("outcome", "e_01_is_success")]
        );
        for c in ["e_06", "e_07"] {
            let r = rules(&with(base(), "event_code", json!(c)));
            assert_eq!(
                r,
                [
                    ("subjects", "required_for_event_code"),
                    ("changed_fields", "required_for_event_code")
                ],
                "{c}"
            );
        }
        // E-05: subjek wajib bila data dibuka; penolakan tanpa subjek boleh.
        let e05 = with(base(), "event_code", json!("e_05"));
        assert_eq!(rules(&e05), [("subjects", "required_for_event_code")]);
        assert_eq!(rules(&failed(e05)), []);
        for c in ["e_10", "e_11", "e_14", "e_16"] {
            let p = with(base(), "event_code", json!(c));
            assert_eq!(
                rules(&p),
                [("changed_fields", "required_for_event_code")],
                "{c}"
            );
            assert_eq!(rules(&failed(p)), [], "{c} gagal tanpa perubahan");
        }
        let e17 = with(base(), "event_code", json!("e_17"));
        let e17 = with(
            e17,
            "subjects",
            json!([{"subject_id": "0190f5c2-7c1a-7000-8000-000000000009", "subject_kind": "care_recipient"}]),
        );
        assert_eq!(rules(&e17), [("disclosure_recipient", "required_for_e_17")]);
        let e17 = with(e17, "disclosure_recipient", json!("satusehat"));
        let e17 = with(e17, "disclosure_data_categories", json!(["encounter"]));
        assert_eq!(rules(&e17), []);
        assert_eq!(
            rules(&with(base(), "disclosure_recipient", json!("satusehat"))),
            [("disclosure_recipient", "only_for_e_17")]
        );
    }

    #[test]
    fn forbidden_content_is_detected() {
        let p = with(base(), "password", json!("x"));
        assert_eq!(rules(&p), [("payload", "unknown_key")]);
        for (name, value) in [
            ("access_token", json!("abc")),
            ("otp_code", json!("12")),
            ("recovery_codes", json!(true)),
            ("note", json!("NIK 3174012345678901")),
            ("label", json!("kartu 4111 1111 1111 1234")),
            ("amount_idr", json!(1_000_000_000_000_u64)),
            ("nik", json!("3174012345678901")),
            ("account_number", json!({"last4": "12345"})),
        ] {
            let p = with(base(), "after_values", json!({ name: value }));
            let p = with(p, "changed_fields", json!([name]));
            assert!(!rules(&p).is_empty(), "{name} lolos");
        }
        let p = with(base(), "changed_fields", json!(["new_password"]));
        assert_eq!(rules(&p), [("changed_fields", "forbidden_or_invalid_name")]);
        let p = with(base(), "outcome_reason_code", json!("x3174012345678901"));
        assert!(!rules(&p).is_empty());
        let p = with(base(), "user_agent", json!("App/1 3174012345678901"));
        assert_eq!(rules(&p), [("user_agent", "digits")]);
    }

    #[test]
    fn allowed_values_pass() {
        let p = with(
            base(),
            "after_values",
            json!({
                "status": "approved",
                "nik": {"last4": "8901"},
                "deleted_at": "2027-03-01T01:00:00.123456789Z",
                "replaced_by": "00000000-0000-0000-0000-000000000001",
                "fee_idr": 999_999_999_999_u64,
                "attempts": 3,
                "label": "anak ke 2"
            }),
        );
        let p = with(
            p,
            "user_agent",
            json!("Mozilla/5.0 (X11; Linux x86_64) Chrome/131.0.6778.85"),
        );
        let p = with(p, "app_version", json!("1.0.3+2027"));
        assert_eq!(rules(&p), []);
    }

    #[test]
    fn strings_allow_uuids_and_timestamps_only() {
        assert!(string_is_clean("00000000-0000-0000-0000-000000000001"));
        assert!(string_is_clean("2027-03-01T01:00:00.123456789Z"));
        assert!(string_is_clean("2027-03-01T01:00:00Z"));
        assert!(string_is_clean("v1234"));
        assert!(!string_is_clean("a12345"));
        assert!(!string_is_clean("4111 1111 1111 1234"));
        assert!(!string_is_clean("2027-03-01T01:00:00.1234567890Z"));
        assert!(!string_is_clean("3174012345678901Z"));
        assert_eq!(longest_digit_run("Chrome/131.0.6778.85"), 4);
        assert_eq!(longest_digit_run("a 123456 b"), 6);
    }
}
