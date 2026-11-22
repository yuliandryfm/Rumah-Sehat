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
        [Self::Nib, Self::Npwp, Self::AuthorityProof]
            .into_iter()
            .find(|k| k.as_str() == code)
    }
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
}
