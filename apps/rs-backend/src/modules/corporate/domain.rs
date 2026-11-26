//! Aturan murni verifikasi perusahaan (F1-T104, US-1902), tanpa I/O.

use authz::CorporateClientStatus;

/// Konteks `reason_code` penolakan perusahaan (CHECK `ck_reason_code_context`, migrasi
/// `20261124200000_platform_reason_code_corporate_rejection.sql`).
pub(super) const REJECTION_CONTEXT: &str = "corporate_rejection";

/// Keputusan Verifikator (`corporate_verification_decision.decision`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Decision {
    /// Disetujui: perusahaan Aktif (US-1902 AC2).
    Approved,
    /// Ditolak dengan kode alasan: perusahaan Ditolak (US-1902 AC3).
    Rejected,
}

impl Decision {
    /// Kode di database.
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::Approved => "approved",
            Self::Rejected => "rejected",
        }
    }

    /// Status perusahaan setelah keputusan.
    pub(super) const fn resulting_status(self) -> CorporateClientStatus {
        match self {
            Self::Approved => CorporateClientStatus::Active,
            Self::Rejected => CorporateClientStatus::Rejected,
        }
    }
}

/// Keputusan hanya sah atas pengajuan yang masih "Menunggu verifikasi" (US-1902 AC4: keputusan pertama yang
/// berlaku; perusahaan Aktif, Ditolak, atau Nonaktif tidak diputuskan ulang sampai diajukan ulang).
pub(super) fn may_decide(status: CorporateClientStatus) -> bool {
    status == CorporateClientStatus::PendingVerification
}

/// Daftar tertutup alasan penolakan perusahaan. **Draf**: PRD US-1902 AC3 hanya menyebut "bukti kewenangan
/// tidak memadai"; kode lain usulan F1-T104 (lihat migrasi `20261124201000_corporate_rejection_reason_seed.sql`).
/// Kode juga wajib aktif di `reason_code` konteks [`REJECTION_CONTEXT`] saat keputusan disimpan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RejectionReason {
    /// Dokumen NIB tidak terbaca, tidak sah, atau tidak sesuai.
    NibDocumentInvalid,
    /// Dokumen NPWP tidak terbaca, tidak sah, atau tidak sesuai.
    NpwpDocumentInvalid,
    /// Data perusahaan tidak sesuai dengan dokumen.
    CompanyDataMismatch,
    /// Bukti kewenangan pendaftar tidak memadai (US-1902 AC3, `[USER] C21-P4` 4b).
    AuthorityProofInsufficient,
}

impl RejectionReason {
    /// Semua kode, urut daftar migrasi data.
    pub(super) const ALL: [Self; 4] = [
        Self::NibDocumentInvalid,
        Self::NpwpDocumentInvalid,
        Self::CompanyDataMismatch,
        Self::AuthorityProofInsufficient,
    ];

    /// Kode.
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::NibDocumentInvalid => "nib_document_invalid",
            Self::NpwpDocumentInvalid => "npwp_document_invalid",
            Self::CompanyDataMismatch => "company_data_mismatch",
            Self::AuthorityProofInsufficient => "authority_proof_insufficient",
        }
    }

    /// Mengurai kode; di luar daftar = `None`.
    pub(super) fn parse(code: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|r| r.as_str() == code)
    }
}

/// Jenis dokumen perusahaan (`corporate_client_document.document_kind`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DocumentKind {
    /// Dokumen NIB.
    Nib,
    /// Dokumen NPWP.
    Npwp,
    /// Bukti kewenangan pendaftar.
    AuthorityProof,
}

impl DocumentKind {
    /// Ketiga jenis dokumen wajib, urut tampilan (Q8: perusahaan "siap diverifikasi" bila ketiganya ada).
    pub(super) const ALL: [Self; 3] = [Self::Nib, Self::Npwp, Self::AuthorityProof];

    /// Kode di database dan di path endpoint.
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::Nib => "nib",
            Self::Npwp => "npwp",
            Self::AuthorityProof => "authority_proof",
        }
    }

    /// Mengurai kode; di luar daftar = `None`.
    pub(super) fn parse(code: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|k| k.as_str() == code)
    }
}

/// Ketiga jenis dokumen wajib sudah ada di `kinds` (kode jenis dokumen aktif; duplikat tidak menambah hitungan).
pub(super) fn documents_complete<S: AsRef<str>>(kinds: &[S]) -> bool {
    DocumentKind::ALL
        .into_iter()
        .all(|k| kinds.iter().any(|c| c.as_ref() == k.as_str()))
}

/// Token versi dokumen aktif (review T1 F1-T102): SHA-256 (heksadesimal) atas ID baris dokumen aktif terurut.
/// Penggantian dokumen selalu membuat baris baru (ID baru), sehingga token berubah pada setiap unggah atau
/// penggantian. Verifikator mengirim token dari detail yang dilihatnya bersama keputusan; keputusan atas dokumen
/// yang berbeda dari yang dilihat ditolak (`4090114`).
pub(super) fn documents_version(ids: &[foundation::Uuid]) -> String {
    let mut sorted = ids.to_vec();
    sorted.sort();
    let mut bytes = Vec::with_capacity(sorted.len() * 16);
    for id in &sorted {
        bytes.extend_from_slice(id.as_bytes());
    }
    crypto::sha256(&bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// Dokumen hanya dapat diunggah, diganti, dan dilihat HR saat perusahaan "Menunggu verifikasi" atau "Ditolak"
/// (US-1901 AC6, keputusan user Q8 2026-11-26). Perusahaan Aktif tidak mengubah dokumen yang sudah diverifikasi;
/// perusahaan Nonaktif sudah ditolak middleware (`401`).
pub(super) fn may_manage_documents(status: CorporateClientStatus) -> bool {
    matches!(
        status,
        CorporateClientStatus::PendingVerification | CorporateClientStatus::Rejected
    )
}

/// Pengajuan ulang oleh HR hanya sah dari status "Ditolak" (US-1902 AC3 "dapat memperbaiki serta mengajukan
/// ulang", TD-47, F1-T102b): perusahaan Menunggu verifikasi belum diputuskan, Aktif sudah disetujui, Nonaktif
/// sudah ditolak middleware. Pengajuan ulang menaikkan `submission_round` sehingga keputusan putaran baru tidak
/// terhalang keputusan putaran lama (UQ `(corporate_client_id, submission_round)`).
pub(super) fn may_resubmit(status: CorporateClientStatus) -> bool {
    status == CorporateClientStatus::Rejected
}

/// Kuota slot unggah dokumen per akun HR per jam (usulan teknis F1-T102; angka bisnis `[TIDAK DIKETAHUI]`):
/// cukup untuk tiga jenis dengan beberapa percobaan ulang, jauh di bawah yang dibutuhkan penyalahgunaan.
pub(super) const DOCUMENT_SLOTS_PER_WINDOW: i64 = 30;
/// Jendela kuota slot unggah dokumen.
pub(super) const DOCUMENT_SLOT_WINDOW: std::time::Duration =
    crate::modules::platform::api::UPLOAD_SLOT_QUOTA_RETENTION;

/// Panjang NIB: tiga belas digit angka `[SRC-854]` (Peraturan BKPM 3/2021, KEPUTUSAN-325). Angka pengaman
/// (digit ke-13) dan tanggal (enam digit pertama) TIDAK divalidasi: algoritma angka pengaman tidak
/// dipublikasikan; keaslian diperiksa Verifikator.
pub(super) const NIB_DIGITS: usize = 13;
/// Panjang NPWP Wajib Pajak Badan: enam belas digit `[SRC-855]` (PMK 112/PMK.03/2022 Pasal 2 ayat (1) huruf b;
/// KEPUTUSAN-325). NPWP 15 digit lama tidak diterima dan tidak diberi angka 0 di depan secara diam-diam.
pub(super) const NPWP_DIGITS: usize = 16;

/// `raw` dipangkas lalu harus tepat `n` angka ASCII (angka Unicode lain, spasi di tengah, dan pemisah ditolak).
fn exact_digits(raw: &str, n: usize) -> Option<String> {
    let t = raw.trim();
    (t.len() == n && t.bytes().all(|b| b.is_ascii_digit())).then(|| t.to_string())
}

/// NIB sah secara format: tepat 13 angka.
pub(super) fn nib_format(raw: &str) -> Option<String> {
    exact_digits(raw, NIB_DIGITS)
}

/// NPWP badan sah secara format: tepat 16 angka.
pub(super) fn npwp_format(raw: &str) -> Option<String> {
    exact_digits(raw, NPWP_DIGITS)
}

/// Asal perusahaan (`corporate_client.origin`).
pub(super) fn parse_origin(code: &str) -> Option<&'static str> {
    match code {
        "self_registered" => Some("self_registered"),
        "created_by_admin" => Some("created_by_admin"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // trace: US-1902 AC4 (keputusan hanya atas pengajuan yang masih menunggu)
    #[test]
    fn only_pending_submissions_can_be_decided() {
        assert!(may_decide(CorporateClientStatus::PendingVerification));
        for s in [
            CorporateClientStatus::Active,
            CorporateClientStatus::Rejected,
            CorporateClientStatus::Inactive,
        ] {
            assert!(!may_decide(s), "{s}");
        }
        assert_eq!(
            Decision::Approved.resulting_status(),
            CorporateClientStatus::Active
        );
        assert_eq!(
            Decision::Rejected.resulting_status(),
            CorporateClientStatus::Rejected
        );
    }

    // trace: US-1902 AC3 (daftar tertutup; bukti kewenangan tidak memadai wajib ada)
    #[test]
    fn rejection_reasons_are_a_closed_list() {
        for r in RejectionReason::ALL {
            assert_eq!(RejectionReason::parse(r.as_str()), Some(r));
            assert!(audit::Code::new(r.as_str()).is_ok());
        }
        assert_eq!(
            RejectionReason::parse("authority_proof_insufficient"),
            Some(RejectionReason::AuthorityProofInsufficient)
        );
        for bad in ["", "other", "photo_unreadable", "NIB_DOCUMENT_INVALID"] {
            assert_eq!(RejectionReason::parse(bad), None, "{bad}");
        }
    }

    // trace: US-1901 AC3 (NIB 13 angka, NPWP badan 16 angka; KEPUTUSAN-325)
    #[test]
    fn nib_is_exactly_thirteen_digits() {
        assert_eq!(
            nib_format("1234567890123").as_deref(),
            Some("1234567890123")
        );
        assert_eq!(
            nib_format(" 1234567890123\n").as_deref(),
            Some("1234567890123")
        );
        for bad in [
            "",
            "123456789012",
            "12345678901234",
            "12345678901a3",
            "12345 7890123",
            "1234567890-23",
            "\u{0661}\u{0662}\u{0663}\u{0664}\u{0665}\u{0666}\u{0667}\u{0668}\u{0669}\u{0660}\u{0661}\u{0662}\u{0663}",
            "1234567890123\u{0}",
        ] {
            assert_eq!(nib_format(bad), None, "{bad:?}");
        }
    }

    // trace: US-1901 AC3 (NPWP badan 16 angka; 15 angka lama ditolak tanpa diubah diam-diam)
    #[test]
    fn npwp_is_exactly_sixteen_digits_and_fifteen_is_not_padded() {
        assert_eq!(
            npwp_format("0123456789012345").as_deref(),
            Some("0123456789012345")
        );
        for bad in [
            "",
            "012345678901234",
            "01234567890123456",
            "01.234.567.8-901.234",
            "012345678901234x",
            "012345678901 345",
        ] {
            assert_eq!(npwp_format(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn document_kinds_match_schema() {
        for (k, s) in [
            (DocumentKind::Nib, "nib"),
            (DocumentKind::Npwp, "npwp"),
            (DocumentKind::AuthorityProof, "authority_proof"),
        ] {
            assert_eq!(k.as_str(), s);
            assert_eq!(DocumentKind::parse(s), Some(k));
        }
        assert_eq!(DocumentKind::parse("ktp"), None);
        assert_eq!(parse_origin("self_registered"), Some("self_registered"));
        assert_eq!(parse_origin("x"), None);
    }
    // trace: Q8 (perusahaan siap diverifikasi setelah ketiga jenis dokumen lengkap)
    #[test]
    fn completeness_needs_all_three_kinds() {
        assert!(documents_complete(&["authority_proof", "nib", "npwp"]));
        assert!(documents_complete(&[
            "npwp",
            "nib",
            "authority_proof",
            "nib"
        ]));
        for partial in [
            vec![],
            vec!["nib"],
            vec!["nib", "npwp"],
            vec!["nib", "authority_proof"],
            vec!["npwp", "authority_proof"],
            vec!["nib", "nib", "nib"],
            vec!["ktp", "nib", "npwp"],
        ] {
            assert!(!documents_complete(&partial), "{partial:?}");
        }
    }

    // trace: review T1 F1-T102 (keputusan terikat versi dokumen yang dilihat Verifikator)
    #[test]
    fn documents_version_is_order_independent_and_changes_with_any_document() {
        let (a, b, c) = (
            foundation::new_id(),
            foundation::new_id(),
            foundation::new_id(),
        );
        let v = documents_version(&[a, b, c]);
        assert_eq!(v.len(), 64);
        assert!(v.bytes().all(|x| x.is_ascii_hexdigit()));
        assert_eq!(v, documents_version(&[c, a, b]));
        assert_ne!(
            v,
            documents_version(&[a, b, foundation::new_id()]),
            "dokumen diganti"
        );
        assert_ne!(v, documents_version(&[a, b]), "dokumen dihapus");
        assert_ne!(documents_version(&[]), documents_version(&[a]));
    }

    // trace: US-1901 AC6, Q8 (dokumen hanya saat menunggu verifikasi atau ditolak)
    #[test]
    fn documents_are_managed_only_while_pending_or_rejected() {
        assert!(may_manage_documents(
            CorporateClientStatus::PendingVerification
        ));
        assert!(may_manage_documents(CorporateClientStatus::Rejected));
        assert!(!may_manage_documents(CorporateClientStatus::Active));
        assert!(!may_manage_documents(CorporateClientStatus::Inactive));
    }

    // trace: US-1902 AC3, TD-47 (pengajuan ulang hanya dari Ditolak)
    #[test]
    fn only_rejected_submissions_can_be_resubmitted() {
        assert!(may_resubmit(CorporateClientStatus::Rejected));
        for s in [
            CorporateClientStatus::PendingVerification,
            CorporateClientStatus::Active,
            CorporateClientStatus::Inactive,
        ] {
            assert!(!may_resubmit(s), "{s}");
        }
    }
}
