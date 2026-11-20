//! Daftar izin before/after per jenis objek (ADR-009 butir 7, PRD §7.2–§7.3).
//!
//! Setiap jenis objek punya satu [`ObjectPolicy`] konstan yang menyebut field mana yang boleh dicatat dan
//! bagaimana. Pustaka yang menerapkan aturannya; pemanggil tidak melakukan masking:
//! - [`FieldKind::Value`]: nilai bertipe (kode, angka, Rupiah, UUID, waktu, boolean) dicatat apa adanya.
//! - [`FieldKind::Text`]: teks pendek non-klinis; disaring pola terlarang dan dipotong.
//! - [`FieldKind::Last4`]: nomor (NIK, nomor kartu, nomor rekening): hanya 4 digit terakhir.
//! - [`FieldKind::NameOnly`]: hanya nama field yang tercatat berubah, nilainya tidak.
//!
//! Objek [`ContentClass::ClinicalOrDocument`] (chat, catatan, resep, rekam medis, hasil lab, foto KTP/KK,
//! bukti klaim) hanya boleh punya field `NameOnly`, ditambah versi objek (PRD §7.3).
//!
//! Nama field yang memuat data rahasia (password, PIN, OTP, token, rahasia MFA, kode pemulihan, dan
//! sejenisnya) ditolak dalam aturan apa pun, termasuk `NameOnly`: perubahan kredensial dinyatakan lewat
//! nama aksi E-04, bukan lewat field. Nama field nomor identitas/kartu/rekening wajib `Last4` atau
//! `NameOnly`.

use crate::code::is_valid_code;
use crate::error::{AuditError, AuditErrorKind};

/// Cara satu field dicatat.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FieldKind {
    /// Nilai bertipe non-teks dicatat apa adanya.
    Value,
    /// Teks pendek non-klinis (disaring dan dipotong) atau nilai bertipe.
    Text,
    /// Hanya 4 digit terakhir dari nomor.
    Last4,
    /// Hanya nama field.
    NameOnly,
}

/// Aturan satu field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FieldRule {
    name: &'static str,
    kind: FieldKind,
}

impl FieldRule {
    /// Field bernilai bertipe.
    pub const fn value(name: &'static str) -> Self {
        Self {
            name,
            kind: FieldKind::Value,
        }
    }

    /// Field teks pendek non-klinis.
    pub const fn text(name: &'static str) -> Self {
        Self {
            name,
            kind: FieldKind::Text,
        }
    }

    /// Field nomor yang hanya dicatat 4 digit terakhirnya.
    pub const fn last4(name: &'static str) -> Self {
        Self {
            name,
            kind: FieldKind::Last4,
        }
    }

    /// Field yang hanya dicatat namanya.
    pub const fn name_only(name: &'static str) -> Self {
        Self {
            name,
            kind: FieldKind::NameOnly,
        }
    }

    /// Nama field.
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// Cara pencatatan.
    pub fn kind(&self) -> FieldKind {
        self.kind
    }
}

/// Kelas isi objek.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentClass {
    /// Data non-klinis: nilai before/after boleh dicatat sesuai aturan field.
    NonClinical,
    /// Konten klinis dan dokumen: hanya daftar field yang berubah dan versi (PRD §7.2, §7.3).
    ClinicalOrDocument,
}

/// Daftar izin satu jenis objek.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObjectPolicy {
    object_type: &'static str,
    class: ContentClass,
    fields: &'static [FieldRule],
}

impl ObjectPolicy {
    /// Objek non-klinis.
    pub const fn non_clinical(object_type: &'static str, fields: &'static [FieldRule]) -> Self {
        Self {
            object_type,
            class: ContentClass::NonClinical,
            fields,
        }
    }

    /// Objek klinis atau dokumen: semua field wajib [`FieldKind::NameOnly`].
    pub const fn clinical_or_document(
        object_type: &'static str,
        fields: &'static [FieldRule],
    ) -> Self {
        Self {
            object_type,
            class: ContentClass::ClinicalOrDocument,
            fields,
        }
    }

    /// Jenis objek (DD B1 `object_type`).
    pub fn object_type(&self) -> &'static str {
        self.object_type
    }

    /// Kelas isi.
    pub fn class(&self) -> ContentClass {
        self.class
    }

    /// Aturan field.
    pub fn fields(&self) -> &'static [FieldRule] {
        self.fields
    }

    /// Memeriksa kebijakan: jenis objek dan nama field berupa kode sah, tanpa duplikat, tanpa nama
    /// terlarang, nomor identitas hanya `Last4`/`NameOnly`, objek klinis hanya `NameOnly`.
    /// Dipanggil pustaka di setiap pencatatan; modul pemilik juga dianjurkan memanggilnya di unit test.
    pub fn validate(&self) -> Result<(), AuditError> {
        if !is_valid_code(self.object_type) {
            return Err(AuditError::new(
                AuditErrorKind::InvalidPolicy,
                "object_type",
            ));
        }
        for (i, rule) in self.fields.iter().enumerate() {
            if !is_valid_code(rule.name) {
                return Err(AuditError::new(AuditErrorKind::InvalidPolicy, "field_name"));
            }
            if self
                .fields
                .iter()
                .take(i)
                .any(|other| other.name == rule.name)
            {
                return Err(AuditError::new(
                    AuditErrorKind::InvalidPolicy,
                    "duplicate_field",
                ));
            }
            if is_forbidden_field_name(rule.name) {
                return Err(AuditError::new(
                    AuditErrorKind::ForbiddenField,
                    "forbidden_field",
                ));
            }
            if is_number_field_name(rule.name)
                && !matches!(rule.kind, FieldKind::Last4 | FieldKind::NameOnly)
            {
                return Err(AuditError::new(
                    AuditErrorKind::InvalidPolicy,
                    "number_field_must_be_last4",
                ));
            }
            if self.class == ContentClass::ClinicalOrDocument && rule.kind != FieldKind::NameOnly {
                return Err(AuditError::new(
                    AuditErrorKind::InvalidPolicy,
                    "clinical_field_must_be_name_only",
                ));
            }
        }
        Ok(())
    }

    pub(crate) fn rule(&self, field: &str) -> Option<FieldRule> {
        self.fields.iter().copied().find(|r| r.name == field)
    }
}

/// Token nama yang menandai data yang tidak pernah boleh dicatat (PRD §7.3 baris 1, ADR-005).
const FORBIDDEN_TOKENS: &[&str] = &[
    "password",
    "passwd",
    "pwd",
    "passphrase",
    "passcode",
    "sandi",
    "pin",
    "otp",
    "totp",
    "hotp",
    "token",
    "jwt",
    "bearer",
    "authorization",
    "cookie",
    "secret",
    "mfa",
    "recovery",
    "credential",
    "credentials",
    "cvv",
    "cvc",
    "apikey",
    "signature",
];

/// Potongan pada nama gabungan (tanpa pemisah) yang juga terlarang.
const FORBIDDEN_SUBSTRINGS: &[&str] = &[
    "password",
    "passwd",
    "passphrase",
    "passcode",
    "katasandi",
    "token",
    "secret",
    "credential",
    "authorization",
    "apikey",
    "privatekey",
    "sessionid",
    "resetlink",
    "confirmationlink",
];

/// Pasangan token terlarang: (salah satu dari kiri) dan (salah satu dari kanan).
const FORBIDDEN_PAIRS: &[(&[&str], &[&str])] = &[
    (
        &["api", "private", "signing", "access", "refresh", "session"],
        &["key", "id", "code"],
    ),
    (
        &["reset", "confirmation", "konfirmasi"],
        &["link", "url", "tautan"],
    ),
    (&["auth"], &["code"]),
];

/// Token nama nomor identitas, kartu, atau rekening (PRD §7.3 baris 2–3): wajib 4 digit terakhir.
const NUMBER_TOKENS: &[&str] = &["nik", "ktp", "pan", "card", "kartu", "rekening", "norek"];

const NUMBER_PAIRS: &[(&[&str], &[&str])] = &[(
    &["account", "bank", "rek", "va"],
    &["number", "no", "nomor", "num"],
)];

fn tokens(name: &str) -> Vec<&str> {
    name.split('_').filter(|t| !t.is_empty()).collect()
}

fn matches_any(tokens: &[&str], set: &[&str]) -> bool {
    tokens.iter().any(|t| set.contains(t))
}

fn matches_pairs(tokens: &[&str], pairs: &[(&[&str], &[&str])]) -> bool {
    pairs
        .iter()
        .any(|(l, r)| matches_any(tokens, l) && matches_any(tokens, r))
}

/// Benar bila nama field termasuk data yang tidak pernah boleh dicatat.
pub fn is_forbidden_field_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    let t = tokens(&lower);
    let joined: String = t.concat();
    matches_any(&t, FORBIDDEN_TOKENS)
        || matches_pairs(&t, FORBIDDEN_PAIRS)
        || FORBIDDEN_SUBSTRINGS.iter().any(|s| joined.contains(s))
}

/// Benar bila nama field menunjuk nomor identitas, kartu, atau rekening.
pub fn is_number_field_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    let t = tokens(&lower);
    let joined: String = t.concat();
    matches_any(&t, NUMBER_TOKENS)
        || matches_pairs(&t, NUMBER_PAIRS)
        || ["accountnumber", "cardnumber", "nomorrekening", "nomorkartu"]
            .iter()
            .any(|s| joined.contains(s))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forbidden_names_cover_prd_7_3_secrets() {
        for name in [
            "password",
            "new_password",
            "password_hash",
            "pin",
            "transaction_pin",
            "otp",
            "otp_code",
            "totp_seed",
            "access_token",
            "refresh_token",
            "api_token",
            "pg_token",
            "session_token",
            "session_id",
            "mfa_secret",
            "client_secret",
            "recovery_code",
            "recovery_codes",
            "reset_link",
            "mfa_reset_confirmation_link",
            "api_key",
            "private_key",
            "auth_code",
            "cvv",
            "kata_sandi",
            "credentials",
        ] {
            assert!(is_forbidden_field_name(name), "{name}");
        }
    }

    #[test]
    fn ordinary_names_are_not_forbidden() {
        for name in [
            "status",
            "margin_percent_bp",
            "price_idr",
            "display_order",
            "is_active",
            "duration_minutes",
            "nik_last4",
            "role",
            "spinner",
        ] {
            assert!(!is_forbidden_field_name(name), "{name}");
        }
    }

    #[test]
    fn number_names_detected() {
        for name in [
            "nik",
            "nik_last4",
            "card_number",
            "account_number",
            "bank_account_number",
            "rekening",
            "va_number",
            "ktp_number",
        ] {
            assert!(is_number_field_name(name), "{name}");
        }
        for name in ["status", "account_type", "bank_name"] {
            assert!(!is_number_field_name(name), "{name}");
        }
    }

    const OK: ObjectPolicy = ObjectPolicy::non_clinical(
        "care_recipient",
        &[
            FieldRule::value("status"),
            FieldRule::text("relationship_note"),
            FieldRule::last4("nik"),
            FieldRule::name_only("photo"),
        ],
    );

    #[test]
    fn valid_policy_passes() {
        assert_eq!(OK.validate(), Ok(()));
        assert_eq!(OK.rule("nik").map(|r| r.kind()), Some(FieldKind::Last4));
        assert_eq!(OK.rule("unknown"), None);
    }

    fn kind_of(p: ObjectPolicy) -> Option<AuditErrorKind> {
        p.validate().err().map(|e| e.kind())
    }

    #[test]
    fn invalid_policies_rejected() {
        const PW: &[FieldRule] = &[FieldRule::name_only("password")];
        const PW_HASH: &[FieldRule] = &[FieldRule::value("password_hash")];
        const NIK_PLAIN: &[FieldRule] = &[FieldRule::value("nik")];
        const CARD_TEXT: &[FieldRule] = &[FieldRule::text("card_number")];
        const DUP: &[FieldRule] = &[FieldRule::value("status"), FieldRule::text("status")];
        const BAD_NAME: &[FieldRule] = &[FieldRule::value("Status")];
        const CLIN_VALUE: &[FieldRule] = &[FieldRule::text("diagnosis_text")];
        assert_eq!(
            kind_of(ObjectPolicy::non_clinical("account", PW)),
            Some(AuditErrorKind::ForbiddenField)
        );
        assert_eq!(
            kind_of(ObjectPolicy::non_clinical("account", PW_HASH)),
            Some(AuditErrorKind::ForbiddenField)
        );
        assert_eq!(
            kind_of(ObjectPolicy::non_clinical("care_recipient", NIK_PLAIN)),
            Some(AuditErrorKind::InvalidPolicy)
        );
        assert_eq!(
            kind_of(ObjectPolicy::non_clinical("payout", CARD_TEXT)),
            Some(AuditErrorKind::InvalidPolicy)
        );
        assert_eq!(
            kind_of(ObjectPolicy::non_clinical("x", DUP)),
            Some(AuditErrorKind::InvalidPolicy)
        );
        assert_eq!(
            kind_of(ObjectPolicy::non_clinical("x", BAD_NAME)),
            Some(AuditErrorKind::InvalidPolicy)
        );
        assert_eq!(
            kind_of(ObjectPolicy::non_clinical("Bad Type", &[])),
            Some(AuditErrorKind::InvalidPolicy)
        );
        assert_eq!(
            kind_of(ObjectPolicy::clinical_or_document(
                "consultation_note",
                CLIN_VALUE
            )),
            Some(AuditErrorKind::InvalidPolicy)
        );
    }
}
