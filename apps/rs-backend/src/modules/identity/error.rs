//! Galat modul `identity` dan pemetaannya ke kode terdaftar (coding-standards §3.2,
//! docs/05-response-codes.md §3 dan §6 modul 01).
//!
//! Kode spesifik modul (F1-T65, US-208): `4090101` aksi atas akun sendiri (AC3, AC5), `4090102` Super Admin
//! aktif terakhir (AC2), `4090103` Auditor aktif terakhir (AC7). Pemisahan tugas Auditor (AC6) memakai
//! `4220001` dengan galat isian `sub_roles`/`auditor_exclusive`.
//!
//! F1-T107 (US-1904 AC2): HR menonaktifkan dirinya sendiri memakai `4090101` yang sama `[ASUMSI]`; HR aktif
//! terakhir perusahaan `4090105`.

use super::admin_domain::AdminRuleViolation;
use super::corporate_hr_domain::HrRuleViolation;
use envelope::{ApiError, FieldError, ResponseCode, codes};
use std::fmt;

/// `4090101`: admin mengubah sub-peran atau menonaktifkan akunnya sendiri (US-208 AC3, AC5).
pub const SELF_ACTION_REJECTED: ResponseCode =
    ResponseCode::new("4090101", "Aksi atas akun sendiri tidak diizinkan");
/// `4090102`: Super Admin aktif terakhir (US-208 AC2).
pub const LAST_SUPER_ADMIN: ResponseCode =
    ResponseCode::new("4090102", "Super Admin aktif terakhir tidak dapat dicabut");
/// `4090103`: Auditor aktif terakhir (US-208 AC7).
pub const LAST_AUDITOR: ResponseCode =
    ResponseCode::new("4090103", "Auditor aktif terakhir tidak dapat dicabut");

/// `4090105`: HR aktif terakhir perusahaan (US-1904 AC2).
pub const LAST_ACTIVE_CORPORATE_HR: ResponseCode = ResponseCode::new(
    "4090105",
    "HR aktif terakhir perusahaan tidak dapat dinonaktifkan",
);

/// Galat use case identitas, sesi, dan akun admin.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdentityError {
    /// PostgreSQL tidak tersedia atau query gagal (503; detail hanya di log).
    Database,
    /// Valkey tidak tersedia saat menulis cache (503). Pemanggil mencoba ulang lewat job.
    CacheUnavailable,
    /// Pencabutan tanpa ID sesi atau dengan `expires_at` yang sudah lewat (500: kesalahan pemanggil).
    InvalidRevocation,
    /// Aturan akun admin dilanggar (US-208).
    AdminRule(AdminRuleViolation),
    /// Aturan kelola anggota HR perusahaan dilanggar (US-1904 AC2).
    HrRule(HrRuleViolation),
    /// Akun admin sasaran tidak ada (atau bukan akun admin Ruang Sehat).
    NotFound,
    /// Isian tidak valid.
    Invalid(Vec<FieldError>),
    /// Badan permintaan bukan JSON yang dapat dibaca (400, F1-T107 undangan HR).
    BadRequest,
    /// Badan permintaan tanpa `Content-Type: application/json` (415).
    UnsupportedMediaType,
    /// Keycloak menolak atau tidak dapat dihubungi (502). Perubahan yang sudah di-commit tetap berlaku;
    /// langkah Keycloak dicoba ulang (lihat dokumentasi use case).
    IdentityProvider,
    /// Pencatatan audit gagal; aksi dibatalkan (500).
    Audit,
}

impl fmt::Display for IdentityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Database => f.write_str("database tidak tersedia"),
            Self::CacheUnavailable => f.write_str("cache tidak tersedia"),
            Self::InvalidRevocation => f.write_str("pencabutan sesi tidak valid"),
            Self::AdminRule(v) => write!(f, "aturan akun admin dilanggar: {v:?}"),
            Self::HrRule(v) => write!(f, "aturan anggota HR dilanggar: {v:?}"),
            Self::NotFound => f.write_str("akun admin tidak ditemukan"),
            Self::Invalid(_) => f.write_str("isian tidak valid"),
            Self::BadRequest => f.write_str("permintaan tidak dapat dibaca"),
            Self::UnsupportedMediaType => f.write_str("jenis media tidak didukung"),
            Self::IdentityProvider => f.write_str("Keycloak gagal"),
            Self::Audit => f.write_str("pencatatan audit gagal"),
        }
    }
}

impl std::error::Error for IdentityError {}

impl IdentityError {
    /// Kode tetap untuk log.
    pub fn code(&self) -> &'static str {
        match self {
            Self::Database => "database",
            Self::CacheUnavailable => "cache_unavailable",
            Self::InvalidRevocation => "invalid_revocation",
            Self::AdminRule(_) => "admin_rule",
            Self::HrRule(_) => "hr_rule",
            Self::NotFound => "not_found",
            Self::Invalid(_) => "invalid",
            Self::BadRequest => "bad_request",
            Self::UnsupportedMediaType => "unsupported_media_type",
            Self::IdentityProvider => "identity_provider",
            Self::Audit => "audit",
        }
    }

    /// Galat isian tunggal.
    pub fn field(field: &str, code: &str, message: &str) -> Self {
        Self::Invalid(vec![FieldError::new(field, code, message)])
    }
}

impl From<AdminRuleViolation> for IdentityError {
    fn from(v: AdminRuleViolation) -> Self {
        // Pelanggaran yang berbentuk isian dinyatakan sebagai galat validasi supaya pesannya tampil di isian.
        match v {
            AdminRuleViolation::AuditorNotExclusive => Self::field(
                "sub_roles",
                "auditor_exclusive",
                "Pemisahan tugas: Auditor tidak dapat digabung dengan sub-peran lain",
            ),
            AdminRuleViolation::NoSubRole => {
                Self::field("sub_roles", "required", "Pilih minimal satu sub-peran")
            }
            AdminRuleViolation::NoChange => Self::field(
                "sub_roles",
                "no_change",
                "Sub-peran sama dengan yang sudah dipegang",
            ),
            other => Self::AdminRule(other),
        }
    }
}

impl ApiError for IdentityError {
    fn response_code(&self) -> ResponseCode {
        match self {
            Self::Database | Self::CacheUnavailable => codes::UNAVAILABLE,
            Self::InvalidRevocation | Self::Audit => codes::INTERNAL,
            Self::NotFound => codes::NOT_FOUND,
            Self::Invalid(_) => codes::INVALID,
            Self::BadRequest => codes::BAD_REQUEST,
            Self::UnsupportedMediaType => codes::UNSUPPORTED_MEDIA_TYPE,
            Self::IdentityProvider => codes::UPSTREAM_BAD_GATEWAY,
            Self::HrRule(v) => match v {
                HrRuleViolation::SelfDeactivation => SELF_ACTION_REJECTED,
                HrRuleViolation::LastActiveHr => LAST_ACTIVE_CORPORATE_HR,
                HrRuleViolation::TargetInactive | HrRuleViolation::TargetProvisioning => {
                    codes::STATUS_CONFLICT
                }
                HrRuleViolation::TargetNotFound => codes::NOT_FOUND,
                // Pemanggil baru saja dinonaktifkan HR lain: sama dengan penolakan middleware (KU-10).
                HrRuleViolation::ActorNotActive => codes::UNAUTHENTICATED,
            },
            Self::AdminRule(v) => match v {
                AdminRuleViolation::SelfRoleChange | AdminRuleViolation::SelfDeactivation => {
                    SELF_ACTION_REJECTED
                }
                AdminRuleViolation::LastSuperAdmin => LAST_SUPER_ADMIN,
                AdminRuleViolation::LastAuditor => LAST_AUDITOR,
                AdminRuleViolation::ActorNotSuperAdmin => codes::FORBIDDEN,
                AdminRuleViolation::TargetInactive
                | AdminRuleViolation::SuperAdminExists
                | AdminRuleViolation::NoPendingBootstrapAccount => codes::STATUS_CONFLICT,
                // Dipetakan ke Invalid lewat From; bila dibangun langsung tetap 422.
                AdminRuleViolation::AuditorNotExclusive
                | AdminRuleViolation::NoSubRole
                | AdminRuleViolation::NoChange => codes::INVALID,
            },
        }
    }

    fn field_errors(&self) -> &[FieldError] {
        match self {
            Self::Invalid(e) => e,
            _ => &[],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn errors_map_to_registered_codes() {
        assert_eq!(IdentityError::Database.response_code().as_str(), "5030001");
        assert_eq!(
            IdentityError::CacheUnavailable.response_code().as_str(),
            "5030001"
        );
        assert_eq!(
            IdentityError::InvalidRevocation.response_code().as_str(),
            "5000001"
        );
        let code = |v: AdminRuleViolation| IdentityError::from(v).response_code().as_str();
        assert_eq!(code(AdminRuleViolation::SelfRoleChange), "4090101");
        assert_eq!(code(AdminRuleViolation::SelfDeactivation), "4090101");
        assert_eq!(code(AdminRuleViolation::LastSuperAdmin), "4090102");
        assert_eq!(code(AdminRuleViolation::LastAuditor), "4090103");
        assert_eq!(code(AdminRuleViolation::ActorNotSuperAdmin), "4030001");
        assert_eq!(code(AdminRuleViolation::TargetInactive), "4090001");
        assert_eq!(code(AdminRuleViolation::AuditorNotExclusive), "4220001");
        assert_eq!(IdentityError::NotFound.response_code().as_str(), "4040001");
        let hr = |v: HrRuleViolation| IdentityError::HrRule(v).response_code().as_str();
        assert_eq!(hr(HrRuleViolation::SelfDeactivation), "4090101");
        assert_eq!(hr(HrRuleViolation::LastActiveHr), "4090105");
        assert_eq!(hr(HrRuleViolation::TargetInactive), "4090001");
        assert_eq!(hr(HrRuleViolation::TargetProvisioning), "4090001");
        assert_eq!(
            IdentityError::BadRequest.response_code().as_str(),
            "4000001"
        );
        assert_eq!(
            IdentityError::UnsupportedMediaType.response_code().as_str(),
            "4150001"
        );
        assert_eq!(hr(HrRuleViolation::TargetNotFound), "4040001");
        assert_eq!(hr(HrRuleViolation::ActorNotActive), "4010001");
        assert_eq!(
            IdentityError::IdentityProvider.response_code().as_str(),
            "5020001"
        );
    }

    #[test]
    fn auditor_exclusivity_is_a_field_error_with_separation_of_duties_message() {
        let e = IdentityError::from(AdminRuleViolation::AuditorNotExclusive);
        let f = e.field_errors();
        assert_eq!(f.len(), 1);
        assert_eq!(
            f.first().map(|x| x.code.as_str()),
            Some("auditor_exclusive")
        );
        assert!(
            f.first()
                .is_some_and(|x| x.message.contains("Pemisahan tugas"))
        );
    }

    #[test]
    fn specific_codes_belong_to_module_01_and_match_http() {
        for c in [
            SELF_ACTION_REJECTED,
            LAST_SUPER_ADMIN,
            LAST_AUDITOR,
            LAST_ACTIVE_CORPORATE_HR,
        ] {
            assert!(c.as_str().starts_with("40901"), "{}", c.as_str());
        }
    }
}
