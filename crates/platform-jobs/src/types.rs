//! Tipe nilai job: ID, jenis, identitas pekerja, kunci deduplikasi, kode galat, payload.
//!
//! Kolom `payload` hanya berisi ID dan `last_error` tanpa data pribadi (DD A19.4). Keduanya ditegakkan
//! lewat tipe: payload hanya menerima UUID atau kode `snake_case`, dan galat hanya berupa kode, sehingga
//! teks bebas (email, nama, pesan galat pustaka) tidak dapat masuk ke tabel `job`.

use crate::error::{JobError, JobErrorKind};
use foundation::{Uuid, new_id};
use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::fmt;

/// Panjang maksimum kode (`job_type`, kode galat, kunci payload, nilai kode payload).
pub const MAX_CODE_LEN: usize = 64;
/// Panjang maksimum kunci deduplikasi.
pub const MAX_DEDUPE_KEY_LEN: usize = 200;
/// Jumlah entri maksimum payload.
pub const MAX_PAYLOAD_ENTRIES: usize = 16;

/// `^[a-z][a-z0-9_]{0,63}$`.
pub(crate) fn is_code(s: &str) -> bool {
    let mut chars = s.chars();
    matches!(chars.next(), Some('a'..='z'))
        && s.len() <= MAX_CODE_LEN
        && chars.all(|c| matches!(c, 'a'..='z' | '0'..='9' | '_'))
}

fn code(s: &str) -> Result<String, JobError> {
    if is_code(s) {
        Ok(s.to_string())
    } else {
        Err(JobError::new(JobErrorKind::InvalidCode))
    }
}

/// UUID bentuk berhubung 36 karakter (sama dengan `CorrelationId::parse`).
fn parse_hyphenated_uuid(s: &str) -> Option<Uuid> {
    if s.len() != 36 {
        return None;
    }
    Uuid::parse_str(s).ok()
}

/// ID job (UUIDv7, ADR-002).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct JobId(Uuid);

impl JobId {
    pub(crate) fn new() -> Self {
        Self(new_id())
    }

    pub(crate) fn from_uuid(id: Uuid) -> Self {
        Self(id)
    }

    /// UUID di dalamnya.
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl fmt::Display for JobId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.hyphenated())
    }
}

/// Jenis job, kode `snake_case` (misalnya `pb_08_reminder`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct JobType(String);

impl JobType {
    /// Memvalidasi jenis job.
    pub fn new(s: &str) -> Result<Self, JobError> {
        code(s).map(Self)
    }

    /// Teks kode.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for JobType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Identitas pekerja untuk kolom `locked_by`: `<komponen>:<UUIDv7>`, unik per proses pekerja.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WorkerId(String);

impl WorkerId {
    /// Membuat identitas baru untuk komponen `component` (kode, misalnya `worker`).
    pub fn new(component: &str) -> Result<Self, JobError> {
        let component = code(component)?;
        Ok(Self(format!("{component}:{}", new_id().hyphenated())))
    }

    /// Teks identitas.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for WorkerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Kunci deduplikasi (`dedupe_key`): mencegah job ganda untuk entitas dan tujuan yang sama selama job
/// masih `queued` atau `running` (`uqp_job_dedupe_key`, DD A19.4). Karakter dibatasi `[a-z0-9_:-]`,
/// sehingga tidak dapat memuat email, nama, atau teks bebas.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DedupeKey(String);

impl DedupeKey {
    /// `<tujuan>:<id entitas>`, misalnya `pb_08_reminder:0193...`.
    pub fn for_entity(purpose: &str, entity_id: Uuid) -> Result<Self, JobError> {
        let purpose = code(purpose).map_err(|_| JobError::new(JobErrorKind::InvalidDedupeKey))?;
        Ok(Self(format!("{purpose}:{}", entity_id.hyphenated())))
    }

    /// Memvalidasi kunci bebas: diawali huruf kecil, `[a-z0-9_:-]`, maksimal 200 karakter
    /// (misalnya `sip_daily_check:2027-03-01`).
    pub fn parse(s: &str) -> Result<Self, JobError> {
        let mut chars = s.chars();
        let ok = matches!(chars.next(), Some('a'..='z'))
            && s.len() <= MAX_DEDUPE_KEY_LEN
            && chars.all(|c| matches!(c, 'a'..='z' | '0'..='9' | '_' | ':' | '-'));
        if ok {
            Ok(Self(s.to_string()))
        } else {
            Err(JobError::new(JobErrorKind::InvalidDedupeKey))
        }
    }

    /// Teks kunci.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Kode galat job untuk `last_error` (tanpa data pribadi, DD A19.4), misalnya `partner_timeout`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ErrorCode(String);

impl ErrorCode {
    /// Kode yang dipakai penyapu untuk job yang sewanya habis.
    pub const LEASE_EXPIRED: &'static str = "lease_expired";
    /// Kode yang dipakai runner untuk payload yang tidak dapat diurai.
    pub const INVALID_PAYLOAD: &'static str = "invalid_payload";

    /// Memvalidasi kode galat.
    pub fn new(s: &str) -> Result<Self, JobError> {
        code(s).map(Self)
    }

    /// Kode `s` bila valid; selain itu [`Self::INVALID_CODE`]. Untuk pemanggil yang memakai kode konstanta dan
    /// tidak boleh panik (lint `panic`/`expect_used`), misalnya worker IdP ADR-030.
    pub fn or_invalid(s: &str) -> Self {
        Self(code(s).unwrap_or_else(|_| Self::INVALID_CODE.to_string()))
    }

    /// Pengganti kode yang tidak valid di [`Self::or_invalid`].
    pub const INVALID_CODE: &'static str = "invalid_error_code";

    /// Teks kode.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Nilai satu entri payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PayloadValue {
    /// ID entitas (UUID).
    Id(Uuid),
    /// Kode `snake_case` (misalnya alasan atau varian tujuan).
    Code(String),
}

/// Payload job: objek JSON yang hanya berisi ID dan kode (DD A19.4 "Hanya ID", ADR-007 butir 3).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct JobPayload(BTreeMap<String, PayloadValue>);

impl JobPayload {
    /// Payload kosong.
    pub fn new() -> Self {
        Self::default()
    }

    fn insert(mut self, key: &str, value: PayloadValue) -> Result<Self, JobError> {
        let key = code(key).map_err(|_| JobError::new(JobErrorKind::InvalidPayload))?;
        self.0.insert(key, value);
        if self.0.len() > MAX_PAYLOAD_ENTRIES {
            return Err(JobError::new(JobErrorKind::InvalidPayload));
        }
        Ok(self)
    }

    /// Menambah entri ID.
    pub fn with_id(self, key: &str, id: Uuid) -> Result<Self, JobError> {
        self.insert(key, PayloadValue::Id(id))
    }

    /// Menambah entri kode.
    pub fn with_code(self, key: &str, value: &str) -> Result<Self, JobError> {
        let value = code(value).map_err(|_| JobError::new(JobErrorKind::InvalidPayload))?;
        self.insert(key, PayloadValue::Code(value))
    }

    /// Jumlah entri (dipakai penangan yang menerima daftar kunci persis, misalnya job IdP ADR-030 yang hanya
    /// menerima ID niat).
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Payload tanpa entri.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// ID pada `key`, bila ada dan berupa ID.
    pub fn id(&self, key: &str) -> Option<Uuid> {
        match self.0.get(key) {
            Some(PayloadValue::Id(id)) => Some(*id),
            _ => None,
        }
    }

    /// Kode pada `key`, bila ada dan berupa kode.
    pub fn code(&self, key: &str) -> Option<&str> {
        match self.0.get(key) {
            Some(PayloadValue::Code(c)) => Some(c),
            _ => None,
        }
    }

    /// Bentuk JSON untuk kolom `payload`.
    pub fn to_json(&self) -> Value {
        let map: Map<String, Value> = self
            .0
            .iter()
            .map(|(k, v)| {
                let v = match v {
                    PayloadValue::Id(id) => Value::String(id.hyphenated().to_string()),
                    PayloadValue::Code(c) => Value::String(c.clone()),
                };
                (k.clone(), v)
            })
            .collect();
        Value::Object(map)
    }

    /// Mengurai nilai kolom `payload` dengan aturan yang sama dengan pembuatannya.
    pub fn from_json(value: &Value) -> Result<Self, JobError> {
        let invalid = || JobError::new(JobErrorKind::InvalidPayload);
        let obj = value.as_object().ok_or_else(invalid)?;
        let mut out = Self::new();
        for (k, v) in obj {
            let s = v.as_str().ok_or_else(invalid)?;
            out = match parse_hyphenated_uuid(s) {
                Some(id) => out.with_id(k, id)?,
                None => out.with_code(k, s)?,
            };
        }
        Ok(out)
    }
}

/// Batas percobaan job (`max_attempts`, CHECK `ck_job_max_attempts`: minimal 1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MaxAttempts(i16);

impl MaxAttempts {
    /// Memvalidasi batas percobaan (minimal 1).
    pub fn new(n: i16) -> Result<Self, JobError> {
        if n >= 1 {
            Ok(Self(n))
        } else {
            Err(JobError::new(JobErrorKind::InvalidMaxAttempts))
        }
    }

    /// Nilai batas.
    pub fn get(&self) -> i16 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn or_invalid_keeps_valid_codes_and_replaces_others() {
        assert_eq!(
            ErrorCode::or_invalid("keycloak_auth").as_str(),
            "keycloak_auth"
        );
        assert_eq!(
            ErrorCode::or_invalid("Bukan Kode").as_str(),
            ErrorCode::INVALID_CODE
        );
        assert!(ErrorCode::new(ErrorCode::INVALID_CODE).is_ok());
    }

    #[test]
    fn code_accepts_snake_case_only() {
        for ok in ["a", "pb_08_reminder", "x1", &"a".repeat(64)] {
            assert!(is_code(ok), "{ok}");
        }
        for bad in [
            "",
            "1abc",
            "_a",
            "Abc",
            "a-b",
            "a b",
            "a.b",
            "user@example.com",
            "é",
            &"a".repeat(65),
        ] {
            assert!(!is_code(bad), "{bad}");
            assert_eq!(
                JobType::new(bad).map_err(|e| e.kind()),
                Err(JobErrorKind::InvalidCode)
            );
            assert_eq!(
                ErrorCode::new(bad).map_err(|e| e.kind()),
                Err(JobErrorKind::InvalidCode)
            );
        }
    }

    #[test]
    fn worker_id_has_component_and_unique_suffix() {
        let a = WorkerId::new("worker").unwrap();
        let b = WorkerId::new("worker").unwrap();
        assert!(a.as_str().starts_with("worker:"));
        assert_ne!(a, b);
        assert!(WorkerId::new("Worker 1").is_err());
    }

    #[test]
    fn dedupe_key_rejects_free_text() {
        let id = new_id();
        let k = DedupeKey::for_entity("pb_08_reminder", id).unwrap();
        assert_eq!(k.as_str(), format!("pb_08_reminder:{}", id.hyphenated()));
        assert!(DedupeKey::parse("sip_daily_check:2027-03-01").is_ok());
        for bad in [
            "",
            "Pb:1",
            "a b",
            "user@example.com",
            "budi santoso",
            "a/b",
            "1a",
            &format!("a{}", "b".repeat(200)),
        ] {
            assert_eq!(
                DedupeKey::parse(bad).map_err(|e| e.kind()),
                Err(JobErrorKind::InvalidDedupeKey),
                "{bad}"
            );
        }
        assert!(DedupeKey::for_entity("Bad Purpose", id).is_err());
    }

    #[test]
    fn payload_roundtrips_ids_and_codes() {
        let id = new_id();
        let p = JobPayload::new()
            .with_id("patient_order_id", id)
            .unwrap()
            .with_code("reason", "payment_timeout")
            .unwrap();
        let v = p.to_json();
        assert_eq!(
            v,
            json!({"patient_order_id": id.hyphenated().to_string(), "reason": "payment_timeout"})
        );
        let back = JobPayload::from_json(&v).unwrap();
        assert_eq!(back, p);
        assert_eq!(back.id("patient_order_id"), Some(id));
        assert_eq!(back.code("reason"), Some("payment_timeout"));
        assert_eq!(back.id("reason"), None);
        assert_eq!(back.code("missing"), None);
    }

    #[test]
    fn payload_rejects_personal_data_and_non_scalar_values() {
        // NEG: payload hanya ID/kode (DD A19.4); teks bebas, angka, objek, dan larik ditolak.
        for bad in [
            json!({"email": "budi@example.com"}),
            json!({"name": "Budi Santoso"}),
            json!({"nik": 3174012345678901_u64}),
            json!({"nik": "3174012345678901"}),
            json!({"nested": {"id": "x"}}),
            json!({"ids": ["a"]}),
            json!({"Key": "abc"}),
            json!(["not", "object"]),
            json!("text"),
            json!(null),
        ] {
            assert_eq!(
                JobPayload::from_json(&bad).map_err(|e| e.kind()),
                Err(JobErrorKind::InvalidPayload),
                "{bad}"
            );
        }
        assert!(JobPayload::new().with_code("email", "a@b.c").is_err());
        assert!(JobPayload::new().with_id("Bad Key", new_id()).is_err());
    }

    #[test]
    fn payload_entry_limit_is_enforced() {
        let mut p = JobPayload::new();
        for i in 0..MAX_PAYLOAD_ENTRIES {
            p = p.with_id(&format!("id_{i}"), new_id()).unwrap();
        }
        assert_eq!(
            p.with_id("one_more", new_id()).map_err(|e| e.kind()),
            Err(JobErrorKind::InvalidPayload)
        );
    }

    #[test]
    fn max_attempts_must_be_positive() {
        assert_eq!(MaxAttempts::new(1).unwrap().get(), 1);
        for bad in [0, -1, i16::MIN] {
            assert_eq!(
                MaxAttempts::new(bad).map_err(|e| e.kind()),
                Err(JobErrorKind::InvalidMaxAttempts)
            );
        }
    }
}
