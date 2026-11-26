//! Aturan murni modul `privacy`: bentuk isi versi teks persetujuan (US-104 AC1) dan pemeriksaan pilihan
//! butir saat pendaftaran (US-101 AC4, US-104 AC2, AC5). Tanpa I/O.
//!
//! Isi kolom `consent_text_version.content` berformat JSON `rs-consent-text/1` (migrasi
//! `20261126200000_privacy_consent_text_draft_seed.sql`): tujuh bagian Pasal 21 ayat (1) UU PDP `[SRC-15]`
//! dengan kode tetap dan urutan tetap, ditambah kontak Ruang Sehat dan penanda draf. Versi yang isinya tidak
//! memenuhi bentuk ini tidak pernah disajikan (galat internal), sehingga layar persetujuan tidak dapat tampil
//! dengan bagian yang hilang.

use foundation::Uuid;
use serde::Deserialize;
use std::collections::BTreeSet;

/// Nilai `format` yang dikenali.
pub(super) const CONTENT_FORMAT: &str = "rs-consent-text/1";

/// Kode tujuh bagian wajib, sesuai urutan huruf (a)–(g) US-104 AC1.
pub const REQUIRED_SECTIONS: [&str; 7] = [
    "legal_basis",
    "purpose",
    "data_types",
    "document_retention",
    "collected_information",
    "processing_period",
    "subject_rights",
];

/// Batas panjang judul bagian (karakter).
const MAX_TITLE_CHARS: usize = 200;
/// Batas panjang isi satu bagian (karakter).
const MAX_BODY_CHARS: usize = 20_000;
/// Batas panjang penanda draf (karakter).
const MAX_NOTICE_CHARS: usize = 1_000;

/// Jenis teks persetujuan (DD A3.1 `consent_type`, CHECK `ck_consent_text_version_consent_type`). Modul lain
/// memilih jenis lewat tipe ini; jenis baru menunggu perluasan skema A3.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ConsentType {
    /// Pemrosesan data pasien saat pendaftaran.
    PatientDataProcessing,
    /// Pernyataan dewasa untuk anggota keluarga.
    FamilyAdultStatement,
    /// Persetujuan wali.
    GuardianConsent,
    /// Persetujuan HR perusahaan saat mendaftarkan perusahaan (US-1901 AC1, F1-T101; PRD L-20).
    CorporateHrRegistration,
}

impl ConsentType {
    /// Nilai kolom `consent_type`.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PatientDataProcessing => "patient_data_processing",
            Self::FamilyAdultStatement => "family_adult_statement",
            Self::GuardianConsent => "guardian_consent",
            Self::CorporateHrRegistration => "corporate_hr_registration",
        }
    }
}

/// Satu bagian teks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsentSection {
    /// Kode tetap bagian (`legal_basis`, ...).
    pub code: String,
    /// Judul tampilan.
    pub title: String,
    /// Isi bagian (teks biasa).
    pub body: String,
}

/// Kontak Ruang Sehat (US-104 AC1 "ditambah kontak Ruang Sehat").
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsentContact {
    /// Judul tampilan.
    pub title: String,
    /// Isi kontak.
    pub body: String,
}

/// Isi versi teks yang sudah divalidasi.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsentContent {
    /// Penanda draf; `None` untuk teks final.
    pub draft_notice: Option<String>,
    /// Tujuh bagian sesuai urutan [`REQUIRED_SECTIONS`].
    pub sections: Vec<ConsentSection>,
    /// Kontak.
    pub contact: ConsentContact,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawContent {
    format: String,
    #[serde(default)]
    draft_notice: Option<String>,
    sections: Vec<RawSection>,
    contact: RawContact,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawSection {
    code: String,
    title: String,
    body: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawContact {
    title: String,
    body: String,
}

/// Alasan isi versi teks ditolak (untuk log; tidak pernah ke respons).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentProblem {
    /// Bukan JSON berbentuk `rs-consent-text/1`.
    Malformed,
    /// Nilai `format` tidak dikenali.
    UnknownFormat,
    /// Bagian tidak tujuh, kodenya lain, atau urutannya lain.
    Sections,
    /// Judul atau isi kosong atau melebihi batas.
    Text,
}

impl ContentProblem {
    /// Kode tetap untuk log.
    pub fn code(self) -> &'static str {
        match self {
            Self::Malformed => "consent_content_malformed",
            Self::UnknownFormat => "consent_content_unknown_format",
            Self::Sections => "consent_content_sections",
            Self::Text => "consent_content_text",
        }
    }
}

fn text_ok(s: &str, max: usize) -> bool {
    !s.trim().is_empty() && s.chars().count() <= max
}

/// Mengurai dan memvalidasi isi versi teks.
pub fn parse_content(raw: &str) -> Result<ConsentContent, ContentProblem> {
    let parsed: RawContent = serde_json::from_str(raw).map_err(|_| ContentProblem::Malformed)?;
    if parsed.format != CONTENT_FORMAT {
        return Err(ContentProblem::UnknownFormat);
    }
    let codes: Vec<&str> = parsed.sections.iter().map(|s| s.code.as_str()).collect();
    if codes != REQUIRED_SECTIONS {
        return Err(ContentProblem::Sections);
    }
    let sections_ok = parsed
        .sections
        .iter()
        .all(|s| text_ok(&s.title, MAX_TITLE_CHARS) && text_ok(&s.body, MAX_BODY_CHARS));
    let contact_ok = text_ok(&parsed.contact.title, MAX_TITLE_CHARS)
        && text_ok(&parsed.contact.body, MAX_BODY_CHARS);
    let notice_ok = parsed
        .draft_notice
        .as_deref()
        .is_none_or(|n| text_ok(n, MAX_NOTICE_CHARS));
    if !(sections_ok && contact_ok && notice_ok) {
        return Err(ContentProblem::Text);
    }
    Ok(ConsentContent {
        draft_notice: parsed.draft_notice,
        sections: parsed
            .sections
            .into_iter()
            .map(|s| ConsentSection {
                code: s.code,
                title: s.title,
                body: s.body,
            })
            .collect(),
        contact: ConsentContact {
            title: parsed.contact.title,
            body: parsed.contact.body,
        },
    })
}

/// Butir persetujuan satu versi.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConsentItem {
    /// `consent_item.id`.
    pub id: Uuid,
    /// Kode butir.
    pub item_code: String,
    /// Teks kotak centang.
    pub label: String,
    /// Butir utama (US-1601 AC2).
    pub is_primary: bool,
    /// Wajib untuk pendaftaran (US-101 AC4).
    pub is_required: bool,
    /// Urutan tampilan.
    pub sort_order: i16,
}

/// Hasil pemeriksaan pilihan butir terhadap versi aktif.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectionCheck {
    /// Versi sesuai, semua butir milik versi itu, dan semua butir wajib dipilih. Isinya butir terpilih
    /// sesuai urutan tampilan.
    Accepted(Vec<Uuid>),
    /// Versi yang dikirim bukan versi aktif, atau ada butir yang bukan milik versi aktif: klien perlu
    /// memuat ulang teks (US-104 AC3).
    Outdated,
    /// Belum ada butir dipilih atau butir wajib tidak dipilih (US-101 AC4, US-104 AC5).
    MissingRequired,
}

/// Memeriksa versi dan butir yang disetujui pendaftar terhadap versi aktif `active_version` beserta
/// butirnya. Duplikat dalam pilihan dianggap satu.
pub fn check_selection(
    active_version: Uuid,
    items: &[ConsentItem],
    sent_version: Uuid,
    selected: &[Uuid],
) -> SelectionCheck {
    if sent_version != active_version {
        return SelectionCheck::Outdated;
    }
    let chosen: BTreeSet<Uuid> = selected.iter().copied().collect();
    let known: BTreeSet<Uuid> = items.iter().map(|i| i.id).collect();
    if !chosen.is_subset(&known) {
        return SelectionCheck::Outdated;
    }
    let required_missing = items
        .iter()
        .any(|i| i.is_required && !chosen.contains(&i.id));
    if chosen.is_empty() || required_missing {
        return SelectionCheck::MissingRequired;
    }
    SelectionCheck::Accepted(
        items
            .iter()
            .filter(|i| chosen.contains(&i.id))
            .map(|i| i.id)
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn content(sections: serde_json::Value) -> String {
        json!({
            "format": CONTENT_FORMAT,
            "draft_notice": "DRAF",
            "sections": sections,
            "contact": {"title": "Kontak", "body": "Isi kontak"},
        })
        .to_string()
    }

    fn seven() -> serde_json::Value {
        serde_json::Value::Array(
            REQUIRED_SECTIONS
                .iter()
                .map(|c| json!({"code": c, "title": format!("Judul {c}"), "body": "Isi"}))
                .collect(),
        )
    }

    // trace: US-104 AC1
    #[test]
    fn content_with_seven_sections_and_contact_is_accepted() {
        let c = parse_content(&content(seven())).unwrap();
        assert_eq!(c.sections.len(), 7);
        assert_eq!(c.sections[0].code, "legal_basis");
        assert_eq!(c.sections[6].code, "subject_rights");
        assert_eq!(c.draft_notice.as_deref(), Some("DRAF"));
        assert_eq!(c.contact.title, "Kontak");
    }

    // trace: US-104 AC1
    #[test]
    fn content_missing_or_reordered_section_is_rejected() {
        let mut six = seven();
        six.as_array_mut().unwrap().pop();
        assert_eq!(parse_content(&content(six)), Err(ContentProblem::Sections));
        let mut swapped = seven();
        swapped.as_array_mut().unwrap().swap(0, 1);
        assert_eq!(
            parse_content(&content(swapped)),
            Err(ContentProblem::Sections)
        );
    }

    #[test]
    fn content_with_empty_text_or_unknown_shape_is_rejected() {
        let mut empty = seven();
        empty[2]["body"] = json!("   ");
        assert_eq!(parse_content(&content(empty)), Err(ContentProblem::Text));
        assert_eq!(parse_content("bukan json"), Err(ContentProblem::Malformed));
        let unknown = json!({"format": "lain/1", "sections": seven(), "contact": {"title": "k", "body": "k"}});
        assert_eq!(
            parse_content(&unknown.to_string()),
            Err(ContentProblem::UnknownFormat)
        );
        let extra = json!({"format": CONTENT_FORMAT, "sections": seven(), "contact": {"title": "k", "body": "k"}, "x": 1});
        assert_eq!(
            parse_content(&extra.to_string()),
            Err(ContentProblem::Malformed)
        );
    }

    #[test]
    fn content_without_draft_notice_is_final() {
        let c = json!({"format": CONTENT_FORMAT, "sections": seven(), "contact": {"title": "k", "body": "k"}});
        assert_eq!(parse_content(&c.to_string()).unwrap().draft_notice, None);
    }

    fn item(n: u128, required: bool) -> ConsentItem {
        ConsentItem {
            id: Uuid::from_u128(n),
            item_code: format!("butir_{n}"),
            label: "Label".into(),
            is_primary: required,
            is_required: required,
            sort_order: i16::try_from(n).unwrap(),
        }
    }

    // trace: US-101 AC4
    #[test]
    fn selection_without_required_item_is_rejected() {
        let v = Uuid::from_u128(100);
        let items = [item(1, true), item(2, false)];
        assert_eq!(
            check_selection(v, &items, v, &[]),
            SelectionCheck::MissingRequired
        );
        assert_eq!(
            check_selection(v, &items, v, &[Uuid::from_u128(2)]),
            SelectionCheck::MissingRequired
        );
        assert_eq!(
            check_selection(v, &items, v, &[Uuid::from_u128(2), Uuid::from_u128(1)]),
            SelectionCheck::Accepted(vec![Uuid::from_u128(1), Uuid::from_u128(2)])
        );
        assert_eq!(
            check_selection(v, &items, v, &[Uuid::from_u128(1), Uuid::from_u128(1)]),
            SelectionCheck::Accepted(vec![Uuid::from_u128(1)])
        );
    }

    // trace: US-104 AC3
    #[test]
    fn selection_for_other_version_or_foreign_item_is_outdated() {
        let v = Uuid::from_u128(100);
        let items = [item(1, true)];
        assert_eq!(
            check_selection(v, &items, Uuid::from_u128(101), &[Uuid::from_u128(1)]),
            SelectionCheck::Outdated
        );
        assert_eq!(
            check_selection(v, &items, v, &[Uuid::from_u128(1), Uuid::from_u128(9)]),
            SelectionCheck::Outdated
        );
    }
}
