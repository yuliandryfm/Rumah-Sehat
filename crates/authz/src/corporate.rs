//! Cakupan tenant perusahaan untuk akun HR (F1-T100, modul kritis T1; ADR-028 butir 1, ADR-012 butir 1
//! dan 4, KEPUTUSAN-254, threat model KOR-05–KOR-07).
//!
//! - `corporate_client_id` **selalu** diambil dari keanggotaan aktif akun HR (`corporate_member` dengan
//!   `deleted_at IS NULL`, satu per akun lewat `uqp_corporate_member_account_id`, M-51), tidak pernah dari
//!   parameter permintaan. Middleware penegak membaca keanggotaan itu lewat [`CorporateScopeResolver`] di
//!   setiap permintaan endpoint HR, lalu memasang [`CorporateScope`] di extensions.
//! - [`CorporateScope`] hanya dibangun crate ini (field privat). Repository modul Korporat menerimanya
//!   sebagai parameter wajib, sehingga tidak ada query data perusahaan tanpa cakupan (pola KEPUTUSAN-128,
//!   seperti [`crate::OwnedCareRecipient`]).
//! - Keputusan per permintaan ([`decide`]): tanpa keanggotaan aktif `401` `4010001`; perusahaan nonaktif
//!   `401` `4010002` (US-1904 AC5, US-1916 AC2, KU-10: layar "Akun nonaktif"); perusahaan menunggu
//!   verifikasi atau ditolak hanya lolos endpoint [`CorporateAccess::AnyVerificationState`] (US-1901 AC6),
//!   endpoint lain `403` `4030001`.
//! - Objek milik perusahaan lain dijawab `404` identik dengan objek tidak ada ([`corporate_object`],
//!   US-1904 AC3, KU-03), dan percobaannya dicatat di latar belakang (security log `access_denied`
//!   `foreign_resource_id`).
//! - Namespace API korporat [`CORPORATE_PATH_PREFIX`]: endpoint HR hanya di namespace ini, dan endpoint
//!   terautentikasi di namespace ini hanya untuk HR (ADR-027 butir 2). Ditegakkan saat start oleh
//!   [`crate::PolicyRouter::finish`].
//!
//! Belum di sini (task lain): akses data sensitif peserta (subjek `corporate_participant`, ADR-028 butir 3
//! dan 7) yang mencatat E-05; [`crate::SensitiveTarget`] saat ini hanya membawa subjek `care_recipient`.

use crate::error::AuthzError;
use crate::object::visible_object;
use crate::principal::{Principal, ResolveError};
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::response::Response;
use envelope::CommonError;
use foundation::Uuid;
use std::fmt;
use std::future::Future;
use std::pin::Pin;

/// Awalan path API korporat (ADR-027 butir 2, KEPUTUSAN-251).
pub const CORPORATE_PATH_PREFIX: &str = "/v1/corporate/";

/// Apakah pola path `path` berada di namespace API korporat.
pub fn is_corporate_path(path: &str) -> bool {
    path.starts_with(CORPORATE_PATH_PREFIX) || path == "/v1/corporate"
}

/// Status perusahaan (`corporate_client.status`, `ck_corporate_client_status`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CorporateClientStatus {
    /// Menunggu verifikasi.
    PendingVerification,
    /// Aktif (lolos verifikasi).
    Active,
    /// Ditolak Verifikator (dapat diajukan ulang, US-1902 AC3).
    Rejected,
    /// Dinonaktifkan (US-1916 AC2).
    Inactive,
}

impl CorporateClientStatus {
    /// Kode di database.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PendingVerification => "pending_verification",
            Self::Active => "active",
            Self::Rejected => "rejected",
            Self::Inactive => "inactive",
        }
    }

    /// Mengurai kode database; kode tidak dikenal = `None` (ditolak).
    pub fn parse(code: &str) -> Option<Self> {
        match code {
            "pending_verification" => Some(Self::PendingVerification),
            "active" => Some(Self::Active),
            "rejected" => Some(Self::Rejected),
            "inactive" => Some(Self::Inactive),
            _ => None,
        }
    }
}

impl fmt::Display for CorporateClientStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Keanggotaan aktif satu akun HR, dibaca resolver dari database aplikasi.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CorporateMembership {
    /// `corporate_member.id`.
    pub corporate_member_id: Uuid,
    /// `corporate_member.corporate_client_id`.
    pub corporate_client_id: Uuid,
    /// `corporate_client.status`.
    pub client_status: CorporateClientStatus,
}

/// Future hasil [`CorporateScopeResolver::active_membership`].
pub type MembershipFuture<'a> =
    Pin<Box<dyn Future<Output = Result<Option<CorporateMembership>, ResolveError>> + Send + 'a>>;

/// Pembaca keanggotaan aktif akun HR. Implementasi database ada di modul `corporate` backend.
///
/// Wajib membaca PostgreSQL langsung di setiap permintaan (tanpa cache), supaya pencabutan keanggotaan dan
/// penonaktifan perusahaan berlaku pada permintaan berikutnya (KU-10, M-51). `Ok(None)` = tidak ada
/// keanggotaan aktif atau perusahaannya terhapus lunak (ADR-003); galat → `503`, tidak pernah dilewatkan.
pub trait CorporateScopeResolver: Send + Sync {
    /// Keanggotaan aktif akun `account_id`.
    fn active_membership<'a>(&'a self, account_id: Uuid) -> MembershipFuture<'a>;
}

/// Syarat status perusahaan untuk endpoint HR. Perusahaan nonaktif selalu ditolak.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CorporateAccess {
    /// Hanya perusahaan Aktif (bawaan; menu pesanan dan seterusnya, US-1902 AC2).
    ActiveClient,
    /// Perusahaan Aktif, Menunggu verifikasi, atau Ditolak: status verifikasi, alasan penolakan, dan profil
    /// perusahaan sendiri (US-1901 AC6).
    AnyVerificationState,
}

/// Cakupan perusahaan pemanggil HR. Hanya dibangun crate ini; handler mengambilnya dengan ekstraktor
/// [`CorporateScope`], repository modul Korporat menerimanya sebagai parameter wajib.
#[derive(Clone)]
pub struct CorporateScope {
    principal: Principal,
    corporate_member_id: Uuid,
    corporate_client_id: Uuid,
    client_status: CorporateClientStatus,
}

impl CorporateScope {
    /// `corporate_client.id` perusahaan pemanggil (dari keanggotaan aktif, bukan dari permintaan).
    pub fn corporate_client_id(&self) -> Uuid {
        self.corporate_client_id
    }

    /// `corporate_member.id` keanggotaan aktif pemanggil.
    pub fn corporate_member_id(&self) -> Uuid {
        self.corporate_member_id
    }

    /// Status perusahaan saat permintaan diperiksa.
    pub fn client_status(&self) -> CorporateClientStatus {
        self.client_status
    }

    /// `account.id` akun HR pemanggil.
    pub fn account_id(&self) -> Uuid {
        self.principal.account_id()
    }

    /// Pemanggil terotorisasi (konteks permintaan, pencatatan penolakan).
    pub fn principal(&self) -> &Principal {
        &self.principal
    }
}

impl fmt::Debug for CorporateScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CorporateScope")
            .field("account_id", &self.principal.account_id())
            .field("corporate_member_id", &self.corporate_member_id)
            .field("corporate_client_id", &self.corporate_client_id)
            .field("client_status", &self.client_status)
            .finish()
    }
}

/// Penolakan cakupan perusahaan: alasan untuk application log dan respons.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ScopeRejection {
    pub(crate) reason: &'static str,
    pub(crate) error: CommonError,
}

/// Keputusan cakupan untuk pemanggil HR yang sudah lolos autentikasi, status akun, sesi, dan peran.
pub(crate) fn decide(
    principal: Principal,
    membership: Option<CorporateMembership>,
    access: CorporateAccess,
) -> Result<CorporateScope, ScopeRejection> {
    let Some(m) = membership else {
        return Err(ScopeRejection {
            reason: "corporate_membership_missing",
            error: CommonError::Unauthenticated,
        });
    };
    match (m.client_status, access) {
        (CorporateClientStatus::Inactive, _) => Err(ScopeRejection {
            reason: "corporate_client_inactive",
            error: CommonError::AccountInactive,
        }),
        (
            CorporateClientStatus::PendingVerification | CorporateClientStatus::Rejected,
            CorporateAccess::ActiveClient,
        ) => Err(ScopeRejection {
            reason: "corporate_client_not_active",
            error: CommonError::Forbidden,
        }),
        (CorporateClientStatus::Active, _)
        | (
            CorporateClientStatus::PendingVerification | CorporateClientStatus::Rejected,
            CorporateAccess::AnyVerificationState,
        ) => Ok(CorporateScope {
            principal,
            corporate_member_id: m.corporate_member_id,
            corporate_client_id: m.corporate_client_id,
            client_status: m.client_status,
        }),
    }
}

/// Mengembalikan objek perusahaan `found` bila ada **dan** dimiliki perusahaan pemanggil
/// (`owner(&obj) == scope.corporate_client_id()`); selain itu [`AuthzError::NotFound`] yang identik untuk
/// "tidak ada" dan "milik perusahaan lain" (US-1904 AC3, KU-03). Percobaan atas objek perusahaan lain
/// dicatat di latar belakang (security log `access_denied` `foreign_resource_id`). `route` adalah pola rute,
/// bukan URL berisi ID.
///
/// Repository tetap wajib memfilter dengan `scope.corporate_client_id()`; fungsi ini adalah pemeriksaan
/// kedua untuk objek yang dibaca lewat ID (pertahanan berlapis).
pub fn corporate_object<T>(
    scope: &CorporateScope,
    route: &str,
    found: Option<T>,
    owner: impl FnOnce(&T) -> Uuid,
) -> Result<T, AuthzError> {
    let own = scope.corporate_client_id;
    visible_object(&scope.principal, route, found, |obj| owner(obj) == own)
}

/// Mengambil [`CorporateScope`] yang dipasang middleware penegak. Tidak ada (endpoint bukan HR atau
/// middleware tidak terpasang) = kesalahan pemrograman → `500`, tidak pernah dilewatkan.
impl<S> FromRequestParts<S> for CorporateScope
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        if let Some(s) = parts.extensions.get::<CorporateScope>() {
            return Ok(s.clone());
        }
        tracing::error!("corporate_scope_missing");
        Err(authn::missing_context_response(parts))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_status_codes_match_database_check_constraint() {
        let all = [
            CorporateClientStatus::PendingVerification,
            CorporateClientStatus::Active,
            CorporateClientStatus::Rejected,
            CorporateClientStatus::Inactive,
        ];
        let codes: Vec<&str> = all.iter().map(|s| s.as_str()).collect();
        assert_eq!(
            codes,
            ["pending_verification", "active", "rejected", "inactive"]
        );
        for s in all {
            assert_eq!(CorporateClientStatus::parse(s.as_str()), Some(s));
        }
        assert_eq!(CorporateClientStatus::parse("Active"), None);
        assert_eq!(CorporateClientStatus::parse(""), None);
    }

    #[test]
    fn corporate_namespace_is_exact_prefix() {
        assert!(is_corporate_path("/v1/corporate/members"));
        assert!(is_corporate_path(
            "/v1/corporate/members/{corporate_member_id}"
        ));
        assert!(is_corporate_path("/v1/corporate"));
        assert!(!is_corporate_path("/v1/corporatex/members"));
        assert!(!is_corporate_path("/v1/corporate-members"));
        assert!(!is_corporate_path("/backoffice/v1/corporate/members"));
        assert!(!is_corporate_path("/v1/public/content/pages"));
    }
}
