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
/// Batas nilai mutlak [`Value::Int`] (6 digit): cukup untuk basis poin, durasi, jumlah baris, dan versi,
/// tetapi NIK, nomor kartu, nomor rekening, dan nomor HP tidak dapat masuk sebagai bilangan
/// (review keamanan F1-T22 BLOKIR-1).
pub const INT_MAX_ABS: u64 = 999_999;
/// Batas nilai mutlak [`Value::Idr`] (12 digit, < Rp1 triliun): NIK (16 digit) dan nomor kartu (13–19
/// digit) tidak dapat masuk sebagai nominal. Nomor rekening 10–12 digit TIDAK dapat dibedakan dari
/// nominal; field nominal wajib bernama nominal (`*_idr`) dan direview di daftar izin.
pub const IDR_MAX_ABS: u64 = 999_999_999_999;

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
        Value::Int(i) if i.unsigned_abs() <= INT_MAX_ABS => json!(i),
        Value::Int(_) => {
            return Err(AuditError::new(
                AuditErrorKind::ValueNotAllowed,
                "int_too_large",
            ));
        }
        Value::Idr(v) if v.get().unsigned_abs() <= IDR_MAX_ABS => json!(v.get()),
        Value::Idr(_) => {
            return Err(AuditError::new(
                AuditErrorKind::ValueNotAllowed,
                "idr_too_large",
            ));
        }
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
/// disamarkan dengan redaksi yang sama dengan log (`observability::redact::scrub_text`), lalu angka
/// dibatasi (lihat [`mask_digits`]) dan dipotong.
pub(crate) fn clean_text(input: &str) -> String {
    let no_control: String = input
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect();
    let scrubbed = observability::redact::scrub_text(&no_control);
    mask_digits(&scrubbed)
        .chars()
        .take(TEXT_MAX_CHARS)
        .collect()
}

/// Pengganti deret angka yang disamarkan di teks.
const DIGITS_REDACTED: &str = "[REDACTED]";

/// Batas angka di teks non-klinis (review keamanan F1-T22 BLOKIR-1 dan non-blokir 1):
/// 1. setiap deret angka 4 digit atau lebih (termasuk OTP/PIN 4–8 digit setelah kata `otp`, `pin`,
///    `sandi`, `password`, `kode`) diganti `[REDACTED]`;
/// 2. bila sisa angka seluruhnya masih 4 atau lebih (nomor yang dipecah spasi atau tanda baca, mis.
///    `3 1 7 4 0 1`), semua angka diganti `#`.
///
/// Angka Unicode non-ASCII (mis. angka lebar penuh) ikut dihitung. Akibatnya teks yang tercatat memuat
/// paling banyak 3 angka; teks label yang sah jarang membutuhkan lebih.
pub(crate) fn mask_digits(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut run = String::new();
    let flush = |run: &mut String, out: &mut String| {
        if run.chars().count() >= 4 {
            out.push_str(DIGITS_REDACTED);
        } else {
            out.push_str(run);
        }
        run.clear();
    };
    for c in input.chars() {
        if c.is_numeric() {
            run.push(c);
        } else {
            flush(&mut run, &mut out);
            out.push(c);
        }
    }
    flush(&mut run, &mut out);
    if out.chars().filter(|c| c.is_numeric()).count() >= 4 {
        out = out
            .chars()
            .map(|c| if c.is_numeric() { '#' } else { c })
            .collect();
    }
    out
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

    /// Probe review keamanan BLOKIR-1: nomor sebagai bilangan di field Value/Text ditolak.
    #[test]
    fn large_numbers_rejected_in_value_and_text_fields() {
        for kind in [FieldKind::Value, FieldKind::Text] {
            for n in [
                3_174_012_345_678_901_i64,
                4_111_111_111_111_111,
                1_234_567_890,
                81_234_567_890,
                1_000_000,
                -1_000_000,
                i64::MIN,
            ] {
                let e = mask(kind, &Value::Int(n)).unwrap_err();
                assert_eq!(e.kind(), AuditErrorKind::ValueNotAllowed, "{kind:?} {n}");
            }
            for n in [0_i64, 10_000, 999_999, -999_999] {
                assert_eq!(mask(kind, &Value::Int(n)).unwrap(), Some(json!(n)));
            }
            for n in [3_174_012_345_678_901_i64, 1_000_000_000_000] {
                assert!(mask(kind, &Value::Idr(Idr::new(n))).is_err(), "{n}");
            }
            assert_eq!(
                mask(kind, &Value::Idr(Idr::new(999_999_999_999))).unwrap(),
                Some(json!(999_999_999_999_i64))
            );
        }
    }

    /// Probe review keamanan non-blokir 1 dan BLOKIR-1: angka di teks dibatasi.
    #[test]
    fn digits_in_text_are_masked() {
        for (input, leak) in [
            ("kode OTP: 482913", "482913"),
            ("pin 4321 jangan dibagikan", "4321"),
            ("password=12345678", "12345678"),
            ("sandi 9876", "9876"),
            ("kode verifikasi 55443322", "55443322"),
            ("rek 1234567890", "1234567890"),
            ("hp 0812 3456 7890", "3456"),
            ("nik 317401 234567 8901", "317401"),
            ("3 1 7 4 0 1 2 3", "3 1 7 4"),
            ("ktp ３１７４０１２３４５６７８９０１", "３１７４"),
            ("12-34-56-78", "12-34"),
        ] {
            let out = clean_text(input);
            assert!(!out.contains(leak), "{input:?} -> {out:?}");
            assert!(
                out.chars().filter(|c| c.is_numeric()).count() <= 3,
                "{input:?} -> {out:?}"
            );
        }
        assert_eq!(clean_text("anak ke-2"), "anak ke-2");
        assert_eq!(clean_text("lantai 12 B"), "lantai 12 B");
    }
}
