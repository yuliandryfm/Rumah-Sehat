//! Nilai before/after dan masking-nya (PRD §7.2–§7.3, ADR-009 butir 7).

use crate::code::Code;
use crate::error::{AuditError, AuditErrorKind};
use crate::policy::FieldKind;
use foundation::{Idr, Sensitive, Uuid};
use serde_json::{Map, Value as Json, json};
use std::fmt;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

/// Batas panjang nilai teks non-klinis setelah disaring (karakter).
pub const TEXT_MAX_CHARS: usize = 200;

/// Nilai satu field. Nomor identitas/kartu/rekening hanya dapat masuk sebagai [`Value::Digits`]
/// (dibungkus [`Sensitive`]) dan hanya keluar sebagai 4 digit terakhir.
#[derive(Clone)]
#[non_exhaustive]
pub enum Value {
    /// Kosong.
    Null,
    /// Boolean.
    Bool(bool),
    /// Bilangan bulat (termasuk basis poin, jumlah, durasi).
    Int(i64),
    /// Rupiah.
    Idr(Idr),
    /// UUID (rujukan objek lain).
    Uuid(Uuid),
    /// Kode tertutup (status, jenis).
    Code(Code),
    /// Instan waktu (UTC).
    Timestamp(OffsetDateTime),
    /// Teks pendek non-klinis; hanya untuk field [`FieldKind::Text`].
    Text(String),
    /// Nomor (NIK, kartu, rekening); hanya untuk field [`FieldKind::Last4`].
    Digits(Sensitive<String>),
}

// Debug tidak mencetak isi teks (bisa data pribadi); Digits sudah tersamar oleh Sensitive.
impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Null => f.write_str("Null"),
            Self::Bool(_) => f.write_str("Bool(..)"),
            Self::Int(_) => f.write_str("Int(..)"),
            Self::Idr(_) => f.write_str("Idr(..)"),
            Self::Uuid(_) => f.write_str("Uuid(..)"),
            Self::Code(_) => f.write_str("Code(..)"),
            Self::Timestamp(_) => f.write_str("Timestamp(..)"),
            Self::Text(_) => f.write_str("Text([REDACTED])"),
            Self::Digits(_) => f.write_str("Digits([REDACTED])"),
        }
    }
}

impl Value {
    /// Kode tertutup dari teks (divalidasi).
    pub fn code(value: &str) -> Result<Self, AuditError> {
        Code::new(value).map(Self::Code)
    }

    /// Teks pendek non-klinis.
    pub fn text(value: impl Into<String>) -> Self {
        Self::Text(value.into())
    }

    /// Nomor sensitif; hanya 4 digit terakhir yang akan tercatat.
    pub fn digits(value: Sensitive<String>) -> Self {
        Self::Digits(value)
    }
}

/// Perubahan satu field (before/after). Nama field wajib ada di daftar izin jenis objeknya.
#[derive(Debug, Clone)]
pub struct FieldChange {
    pub(crate) field: String,
    pub(crate) before: Option<Value>,
    pub(crate) after: Option<Value>,
}

impl FieldChange {
    /// Perubahan nilai `before` → `after` (`None` = tidak ada nilai di sisi itu, mis. pembuatan).
    pub fn new(field: &str, before: Option<Value>, after: Option<Value>) -> Self {
        Self {
            field: field.to_string(),
            before,
            after,
        }
    }

    /// Hanya mencatat bahwa field berubah (tanpa nilai).
    pub fn changed(field: &str) -> Self {
        Self::new(field, None, None)
    }
}

/// Memasking satu nilai sesuai aturan field. `None` berarti nilai tidak boleh dicatat sama sekali.
pub(crate) fn mask(kind: FieldKind, value: &Value) -> Result<Option<Json>, AuditError> {
    match kind {
        FieldKind::NameOnly => Ok(None),
        FieldKind::Last4 => match value {
            Value::Null => Ok(Some(Json::Null)),
            Value::Digits(d) => last4(d).map(Some),
            _ => Err(AuditError::new(
                AuditErrorKind::ValueNotAllowed,
                "last4_requires_digits",
            )),
        },
        FieldKind::Value => match value {
            Value::Text(_) => Err(AuditError::new(
                AuditErrorKind::ValueNotAllowed,
                "text_not_allowed",
            )),
            Value::Digits(_) => Err(AuditError::new(
                AuditErrorKind::ValueNotAllowed,
                "digits_not_allowed",
            )),
            other => typed(other).map(Some),
        },
        FieldKind::Text => match value {
            Value::Text(t) => Ok(Some(Json::String(clean_text(t)))),
            Value::Digits(_) => Err(AuditError::new(
                AuditErrorKind::ValueNotAllowed,
                "digits_not_allowed",
            )),
            other => typed(other).map(Some),
        },
    }
}

fn typed(value: &Value) -> Result<Json, AuditError> {
    Ok(match value {
        Value::Null => Json::Null,
        Value::Bool(b) => Json::Bool(*b),
        Value::Int(i) => json!(i),
        Value::Idr(v) => json!(v.get()),
        Value::Uuid(u) => Json::String(u.hyphenated().to_string()),
        Value::Code(c) => Json::String(c.as_str().to_string()),
        Value::Timestamp(t) => Json::String(rfc3339(*t)?),
        Value::Text(_) | Value::Digits(_) => {
            return Err(AuditError::new(
                AuditErrorKind::ValueNotAllowed,
                "typed_value",
            ));
        }
    })
}

pub(crate) fn rfc3339(t: OffsetDateTime) -> Result<String, AuditError> {
    t.to_offset(time::UtcOffset::UTC)
        .format(&Rfc3339)
        .map_err(|_| AuditError::new(AuditErrorKind::Serialization, "timestamp"))
}

/// `{"last4": "1234"}` dari nomor; pemisah spasi/tanda hubung diabaikan, karakter lain ditolak.
fn last4(d: &Sensitive<String>) -> Result<Json, AuditError> {
    let raw = d.expose();
    let mut digits = Vec::with_capacity(raw.len());
    for c in raw.chars() {
        if c.is_ascii_digit() {
            digits.push(c);
        } else if c != ' ' && c != '-' {
            return Err(AuditError::new(AuditErrorKind::InvalidValue, "digits"));
        }
    }
    if digits.len() < 4 {
        return Err(AuditError::new(AuditErrorKind::InvalidValue, "digits"));
    }
    let tail: String = digits.iter().skip(digits.len() - 4).collect();
    let mut m = Map::new();
    m.insert("last4".to_string(), Json::String(tail));
    Ok(Json::Object(m))
}

/// Teks non-klinis: karakter kontrol dibuang, pola terlarang (token, email, deret nomor panjang, HP)
/// disamarkan dengan redaksi yang sama dengan log (`observability::redact::scrub_text`), lalu dipotong.
pub(crate) fn clean_text(input: &str) -> String {
    let no_control: String = input
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect();
    let scrubbed = observability::redact::scrub_text(&no_control);
    scrubbed.chars().take(TEXT_MAX_CHARS).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &str) -> Sensitive<String> {
        Sensitive::new(v.to_string())
    }

    #[test]
    fn last4_keeps_only_tail() {
        let out = mask(FieldKind::Last4, &Value::digits(s("3174012345678901"))).unwrap();
        assert_eq!(out, Some(json!({"last4": "8901"})));
        let out = mask(FieldKind::Last4, &Value::digits(s("4111 1111-1111 1234"))).unwrap();
        assert_eq!(out, Some(json!({"last4": "1234"})));
    }

    #[test]
    fn last4_rejects_short_or_non_digit() {
        for bad in ["123", "", "12a456", "budi@example.com"] {
            assert!(
                mask(FieldKind::Last4, &Value::digits(s(bad))).is_err(),
                "{bad}"
            );
        }
        assert!(mask(FieldKind::Last4, &Value::Int(31740123)).is_err());
        assert!(mask(FieldKind::Last4, &Value::text("3174012345678901")).is_err());
    }

    #[test]
    fn value_kind_rejects_text_and_digits() {
        assert!(mask(FieldKind::Value, &Value::text("x")).is_err());
        assert!(mask(FieldKind::Value, &Value::digits(s("1234567890123456"))).is_err());
        assert_eq!(
            mask(FieldKind::Value, &Value::Idr(Idr::new(250_000))).unwrap(),
            Some(json!(250_000))
        );
    }

    #[test]
    fn name_only_never_emits_value() {
        assert_eq!(
            mask(FieldKind::NameOnly, &Value::digits(s("3174012345678901"))).unwrap(),
            None
        );
        assert_eq!(
            mask(FieldKind::NameOnly, &Value::text("isi")).unwrap(),
            None
        );
    }

    #[test]
    fn text_is_scrubbed_and_truncated() {
        let t = "hubungi budi@example.com\nNIK 3174012345678901 Bearer abc.def.ghi";
        let out = clean_text(t);
        assert!(!out.contains("budi@example.com"), "{out}");
        assert!(!out.contains("3174012345678901"), "{out}");
        assert!(!out.contains("abc.def.ghi"), "{out}");
        assert!(!out.contains('\n'));
        let long = "a".repeat(1000);
        assert_eq!(clean_text(&long).chars().count(), TEXT_MAX_CHARS);
    }

    #[test]
    fn debug_never_prints_contents() {
        let v = format!(
            "{:?} {:?}",
            Value::text("Budi Santoso"),
            Value::digits(s("3174012345678901"))
        );
        assert!(!v.contains("Budi") && !v.contains("3174"), "{v}");
    }
}
