//! Koreksi data perusahaan oleh HR saat pengajuan Ditolak (CR-036, keputusan user Q20 = A 2026-11-27; T1:
//! isolasi tenant korporat dan audit; KEPUTUSAN-410 s.d. KEPUTUSAN-414).
//!
//! **Satu aksi dengan pengajuan ulang** (KEPUTUSAN-410): koreksi nama badan usaha, NIB, NPWP, alamat kantor,
//! dan telepon kantor hanya diterapkan bersama pengajuan ulang (`POST /v1/corporate/company-corrections`), di
//! dalam transaksi pengajuan ulang yang sama (`documents::resubmit_with`): kunci baris perusahaan, status wajib
//! Ditolak, ketiga dokumen wajib lengkap, lalu satu `UPDATE ... WHERE status = 'rejected' AND submission_round
//! = $n` yang mengubah data dan status sekaligus (wajib tepat satu baris). Tidak ada keadaan "data dikoreksi
//! tetapi belum diajukan ulang", tidak ada jalur ubah data di status lain, dan batas laju per akun pengajuan
//! ulang (KEPUTUSAN-389) otomatis berlaku tanpa pembatas baru.
//!
//! **Validasi** sama dengan pendaftaran (KEPUTUSAN-325, pesan sama dengan `registration.rs`): NIB tepat 13
//! angka, NPWP tepat 16 angka (Q6), nama/alamat/telepon wajib dengan batas panjang yang sama. Field yang tidak
//! dikirim tidak diubah (klien tidak perlu memegang NPWP lengkap untuk mengoreksi field lain, Q23 = A); isian
//! kosong sama sekali ditolak 422 (pakai pengajuan ulang biasa).
//!
//! **NIB** (CR-035, KEPUTUSAN-365): NIB baru tidak pernah dibandingkan dengan perusahaan lain di jalur ini,
//! sehingga NIB yang sudah dipakai perusahaan lain diproses persis sama dengan NIB baru. Penanda
//! `nib_used_by_other_company` di antrean Verifikator dihitung saat dibaca (KEPUTUSAN-363), jadi ikut NIB hasil
//! koreksi; keunikan di antara perusahaan Aktif tetap ditegakkan saat persetujuan (`4090116`).
//!
//! **Audit** (ADR-005, KEPUTUSAN-411 diubah KEPUTUSAN-416 setelah review DBA CR-036 C1): E-10
//! `corporate_client_data_corrected` atas objek `corporate_client` untuk field yang benar-benar berubah, dengan
//! nilai sebelum/sesudah menurut [`CORRECTION_POLICY`]: nama badan usaha, alamat, dan telepon sebagai teks pendek
//! (disaring pustaka audit: deret angka 4 digit atau lebih disamarkan), NIB dan NPWP hanya 4 digit terakhir
//! (`last4`; NPWP Terbatas DD A20, NPWP penuh tidak pernah masuk payload). Ditulis di transaksi yang sama dengan
//! E-10 `corporate_verification_resubmitted`. Koreksi yang ditolak aturan
//! (bukan Ditolak, dokumen belum lengkap) tercatat E-10 hasil gagal untuk kedua aksi di transaksi tersendiri.

use super::admin_create::{ADDRESS_MAX, LEGAL_NAME_MAX, PHONE_MAX, field, text};
use super::domain::{nib_format, npwp_format};
use super::error::CorporateError;
use super::repo::{ClientCorrection, ClientProfileRow};
use audit::{FieldChange, FieldRule, ObjectPolicy, Value};
use foundation::Sensitive;
use std::fmt;

/// Nama aksi E-10 koreksi data perusahaan.
pub(super) const CORRECTED_ACTION: &str = "corporate_client_data_corrected";

/// Kebijakan audit koreksi data perusahaan (ADR-005 before/after; KEPUTUSAN-416): nama, alamat, dan telepon
/// sebagai teks pendek yang disaring pustaka (`text`), NIB dan NPWP hanya 4 digit terakhir (`last4`; NPWP
/// Terbatas, DD A20). Satu-satunya event korporat yang memuat nilai data perusahaan; pemeriksa test
/// `assert_corporate_correction_event_clean` mengecualikan field teks ini secara sempit (KEPUTUSAN-385 tetap
/// berlaku untuk event lain).
pub(super) static CORRECTION_POLICY: ObjectPolicy = ObjectPolicy::non_clinical(
    "corporate_client",
    &[
        FieldRule::text("legal_name"),
        FieldRule::last4("nib"),
        FieldRule::last4("npwp"),
        FieldRule::text("office_address"),
        FieldRule::text("office_phone"),
    ],
);

/// Nilai teks untuk field `text`.
fn text_value(v: &str) -> Value {
    Value::text(v)
}

/// Nilai nomor untuk field `last4`. Nilai tersimpan yang bukan deret angka ASCII minimal 4 digit (mis. data lama
/// di luar format KEPUTUSAN-325) dicatat `Null`, bukan menggagalkan pencatatan audit (gagal-tertutup tetap untuk
/// galat pustaka lain). Nilai baru selalu sudah divalidasi 13/16 angka.
fn digits_value(v: &str) -> Value {
    if v.len() >= 4 && v.bytes().all(|b| b.is_ascii_digit()) {
        Value::digits(Sensitive::new(v.to_string()))
    } else {
        Value::Null
    }
}

/// Isian koreksi dari HR. `None` = field tidak dikoreksi. NPWP (Terbatas) dibungkus [`Sensitive`]; `Debug`
/// manual tanpa isian.
#[derive(Default)]
pub struct CompanyCorrectionInput {
    /// Nama badan usaha.
    pub legal_name: Option<String>,
    /// NIB: tepat 13 angka.
    pub nib: Option<String>,
    /// NPWP badan: tepat 16 angka.
    pub npwp: Option<Sensitive<String>>,
    /// Alamat kantor.
    pub office_address: Option<String>,
    /// Telepon kantor.
    pub office_phone: Option<String>,
}

impl fmt::Debug for CompanyCorrectionInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CompanyCorrectionInput")
            .finish_non_exhaustive()
    }
}

/// Koreksi tervalidasi (nilai dipangkas). Tanpa `Debug` turunan (memuat NPWP).
pub(super) struct ValidatedCorrection {
    legal_name: Option<String>,
    nib: Option<String>,
    npwp: Option<String>,
    office_address: Option<String>,
    office_phone: Option<String>,
}

impl fmt::Debug for ValidatedCorrection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ValidatedCorrection")
            .finish_non_exhaustive()
    }
}

impl ValidatedCorrection {
    /// Parameter `UPDATE` (`repo::resubmit_client`).
    pub(super) fn as_repo(&self) -> ClientCorrection<'_> {
        ClientCorrection {
            legal_name: self.legal_name.as_deref(),
            nib: self.nib.as_deref(),
            npwp: self.npwp.as_deref(),
            office_address: self.office_address.as_deref(),
            office_phone: self.office_phone.as_deref(),
        }
    }

    /// Perubahan audit (before/after menurut [`CORRECTION_POLICY`]) untuk field yang berubah, urut tetap.
    pub(super) fn changes(&self, current: &ClientProfileRow) -> Vec<FieldChange> {
        self.changed_fields(current)
            .into_iter()
            .map(|name| {
                let (old, new, value): (&str, &str, fn(&str) -> Value) = match name {
                    "legal_name" => (
                        &current.legal_name,
                        self.legal_name.as_deref().unwrap_or_default(),
                        text_value,
                    ),
                    "nib" => (
                        &current.nib,
                        self.nib.as_deref().unwrap_or_default(),
                        digits_value,
                    ),
                    "npwp" => (
                        &current.npwp,
                        self.npwp.as_deref().unwrap_or_default(),
                        digits_value,
                    ),
                    "office_address" => (
                        &current.office_address,
                        self.office_address.as_deref().unwrap_or_default(),
                        text_value,
                    ),
                    _ => (
                        &current.office_phone,
                        self.office_phone.as_deref().unwrap_or_default(),
                        text_value,
                    ),
                };
                FieldChange::new(name, Some(value(old)), Some(value(new)))
            })
            .collect()
    }

    /// Nama field yang nilainya berbeda dari baris tersimpan `current` (urut tetap). Field yang dikirim dengan
    /// nilai sama tidak dihitung berubah.
    pub(super) fn changed_fields(&self, current: &ClientProfileRow) -> Vec<&'static str> {
        let differs = |new: &Option<String>, old: &str| new.as_deref().is_some_and(|n| n != old);
        [
            ("legal_name", differs(&self.legal_name, &current.legal_name)),
            ("nib", differs(&self.nib, &current.nib)),
            ("npwp", differs(&self.npwp, &current.npwp)),
            (
                "office_address",
                differs(&self.office_address, &current.office_address),
            ),
            (
                "office_phone",
                differs(&self.office_phone, &current.office_phone),
            ),
        ]
        .into_iter()
        .filter_map(|(name, changed)| changed.then_some(name))
        .collect()
    }
}

/// Validasi per field, sama dengan pendaftaran (KEPUTUSAN-325; pesan sama dengan `registration::validate`).
/// Galat dilaporkan per field (422 `4220001`); isian tanpa satu field pun = galat `$`.
pub(super) fn validate(
    input: &CompanyCorrectionInput,
) -> Result<ValidatedCorrection, CorporateError> {
    let mut errors = Vec::new();
    let mut take = |name: &str, raw: Option<&str>, max: usize, message: &str| {
        raw.and_then(|r| {
            let v = text(r, max, false);
            if v.is_none() {
                errors.push(field(name, "invalid", message));
            }
            v
        })
    };
    let legal_name = take(
        "legal_name",
        input.legal_name.as_deref(),
        LEGAL_NAME_MAX,
        "Nama badan usaha wajib, maksimal 200 karakter",
    );
    let office_address = take(
        "office_address",
        input.office_address.as_deref(),
        ADDRESS_MAX,
        "Alamat kantor wajib, maksimal 500 karakter",
    );
    let office_phone = take(
        "office_phone",
        input.office_phone.as_deref(),
        PHONE_MAX,
        "Telepon kantor wajib, maksimal 32 karakter",
    );
    let nib = input.nib.as_deref().and_then(|r| {
        let v = nib_format(r);
        if v.is_none() {
            errors.push(field("nib", "invalid_format", "NIB harus tepat 13 angka"));
        }
        v
    });
    let npwp = input.npwp.as_ref().and_then(|r| {
        let v = npwp_format(r.expose());
        if v.is_none() {
            errors.push(field(
                "npwp",
                "invalid_format",
                "NPWP harus tepat 16 angka (NPWP 15 angka lama: tambahkan angka 0 di depan)",
            ));
        }
        v
    });
    let empty = input.legal_name.is_none()
        && input.nib.is_none()
        && input.npwp.is_none()
        && input.office_address.is_none()
        && input.office_phone.is_none();
    if empty {
        errors.push(field(
            "$",
            "required",
            "Isi minimal satu data perusahaan yang dikoreksi",
        ));
    }
    if errors.is_empty() {
        Ok(ValidatedCorrection {
            legal_name,
            nib,
            npwp,
            office_address,
            office_phone,
        })
    } else {
        Err(CorporateError::Validation(errors))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::OffsetDateTime;

    fn current() -> ClientProfileRow {
        ClientProfileRow {
            legal_name: "PT Lama".to_string(),
            nib: "9900000000001".to_string(),
            npwp: "9900000000000001".to_string(),
            office_address: "Jl. Lama".to_string(),
            office_phone: "0210000000".to_string(),
            status: "rejected".to_string(),
            submission_round: 1,
            submitted_at: OffsetDateTime::UNIX_EPOCH,
        }
    }

    fn errors_of(r: Result<ValidatedCorrection, CorporateError>) -> Vec<String> {
        match r {
            Err(CorporateError::Validation(e)) => e.iter().map(|f| f.field.clone()).collect(),
            other => panic!("harus galat validasi: {other:?}"),
        }
    }

    // trace: CR-036 (validasi sama dengan pendaftaran KEPUTUSAN-325, Q6)
    #[test]
    fn present_fields_are_validated_like_registration_and_trimmed() {
        let v = validate(&CompanyCorrectionInput {
            legal_name: Some("  PT Baru  ".to_string()),
            nib: Some(" 9900000000002 ".to_string()),
            npwp: Some(Sensitive::new("9900000000000002".to_string())),
            ..CompanyCorrectionInput::default()
        })
        .unwrap();
        let r = v.as_repo();
        assert_eq!(r.legal_name, Some("PT Baru"));
        assert_eq!(r.nib, Some("9900000000002"));
        assert_eq!(r.npwp, Some("9900000000000002"));
        assert_eq!(r.office_address, None);
        assert_eq!(r.office_phone, None);

        let bad = validate(&CompanyCorrectionInput {
            legal_name: Some("   ".to_string()),
            nib: Some("123456789012".to_string()),
            npwp: Some(Sensitive::new("990000000000001".to_string())),
            office_address: Some("x".repeat(501)),
            office_phone: Some("0".repeat(33)),
        });
        assert_eq!(
            errors_of(bad),
            [
                "legal_name",
                "office_address",
                "office_phone",
                "nib",
                "npwp"
            ]
        );
        for nib in ["99000000000０2", "9900-00000002", "99000000000021"] {
            let r = validate(&CompanyCorrectionInput {
                nib: Some(nib.to_string()),
                ..CompanyCorrectionInput::default()
            });
            assert_eq!(errors_of(r), ["nib"], "{nib}");
        }
    }

    // trace: CR-036 (isian kosong bukan koreksi; pakai pengajuan ulang biasa)
    #[test]
    fn empty_correction_is_rejected() {
        assert_eq!(
            errors_of(validate(&CompanyCorrectionInput::default())),
            ["$"]
        );
    }

    // trace: CR-036, KEPUTUSAN-411 (audit hanya nama field yang benar-benar berubah)
    #[test]
    fn only_really_changed_fields_are_reported() {
        let v = validate(&CompanyCorrectionInput {
            legal_name: Some("PT Lama".to_string()),
            nib: Some("9900000000009".to_string()),
            office_phone: Some(" 0210000000 ".to_string()),
            npwp: Some(Sensitive::new("9900000000000009".to_string())),
            ..CompanyCorrectionInput::default()
        })
        .unwrap();
        assert_eq!(v.changed_fields(&current()), ["nib", "npwp"]);
    }

    // trace: ADR-005 (before/after), DD A20 (NPWP Terbatas: hanya 4 digit terakhir), KEPUTUSAN-416
    #[test]
    fn correction_policy_keeps_npwp_and_nib_to_last4_and_debug_hides_values() {
        assert!(CORRECTION_POLICY.validate().is_ok());
        let kind = |f: &str| {
            CORRECTION_POLICY
                .fields()
                .iter()
                .find(|r| r.name() == f)
                .map(|r| r.kind())
        };
        assert_eq!(kind("npwp"), Some(audit::FieldKind::Last4));
        assert_eq!(kind("nib"), Some(audit::FieldKind::Last4));
        for f in ["legal_name", "office_address", "office_phone"] {
            assert_eq!(kind(f), Some(audit::FieldKind::Text), "{f}");
        }
        assert!(matches!(digits_value("9900000000000009"), Value::Digits(_)));
        for bad in ["", "123", "12-3456", "abcd"] {
            assert!(matches!(digits_value(bad), Value::Null), "{bad}");
        }
        assert!(audit::Code::new(CORRECTED_ACTION).is_ok());
        let input = CompanyCorrectionInput {
            npwp: Some(Sensitive::new("9900000000000009".to_string())),
            legal_name: Some("PT Rahasia".to_string()),
            ..CompanyCorrectionInput::default()
        };
        let d = format!("{input:?} {:?}", validate(&input).unwrap());
        assert!(
            !d.contains("9900000000000009") && !d.contains("Rahasia"),
            "{d}"
        );
    }
}
