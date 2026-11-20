//! Kebijakan endpoint (ADR-012 butir 1: setiap endpoint menyatakan kebijakannya; default deny).

use crate::principal::Principal;
use crate::role::{AccountType, AdminSubRole};
use authn::Realm;
use std::collections::BTreeSet;

/// Alasan sah sebuah endpoint tanpa login (ADR-012 butir 3). Wajib disebut supaya endpoint publik
/// selalu keputusan yang terlihat di review, bukan kelalaian.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PublicReason {
    /// Endpoint operasional tanpa data (kesehatan proses; F1-T11).
    Operational,
    /// Konten yang memang publik (UP-21).
    PublicContent,
    /// Halaman verifikasi resep dengan kode acak (US-309 AC6–AC8).
    PrescriptionVerification,
}

/// Syarat akses endpoint terautentikasi. Dibangun lewat [`Policy`]; tanpa konstruktor publik supaya
/// syarat selalu memuat minimal satu jenis akun.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Requirement {
    account_types: BTreeSet<AccountType>,
    any_sub_role: BTreeSet<AdminSubRole>,
}

impl Requirement {
    /// Jenis akun yang diizinkan.
    pub fn account_types(&self) -> &BTreeSet<AccountType> {
        &self.account_types
    }

    /// Sub-peran admin yang diizinkan (cukup salah satu). Kosong = jenis akun saja yang diperiksa.
    pub fn any_sub_role(&self) -> &BTreeSet<AdminSubRole> {
        &self.any_sub_role
    }

    /// Realm yang dapat menerbitkan token untuk endpoint ini (dari jenis akun).
    pub fn realms(&self) -> BTreeSet<Realm> {
        self.account_types.iter().map(|t| t.realm()).collect()
    }

    /// Token realm lain tidak diterima endpoint ini (coding-standards §7.3 butir 2: `401`).
    pub(crate) fn accepts_realm(&self, realm: Realm) -> bool {
        self.account_types.iter().any(|t| t.realm() == realm)
    }

    /// Keputusan peran atas pemanggil yang sudah diautentikasi dan aktif.
    pub(crate) fn allows(&self, principal: &Principal) -> bool {
        if !self.account_types.contains(&principal.account_type()) {
            return false;
        }
        if principal.account_type() == AccountType::RsAdmin {
            // Admin Ruang Sehat wajib memegang salah satu sub-peran yang disebut kebijakan.
            return self.any_sub_role.iter().any(|r| principal.has_sub_role(*r));
        }
        true
    }

    pub(crate) fn is_satisfiable(&self) -> bool {
        !self.account_types.is_empty()
            && (!self.account_types.contains(&AccountType::RsAdmin)
                || !self.any_sub_role.is_empty())
    }
}

/// Kebijakan satu endpoint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Policy {
    /// Tanpa login, dengan alasan yang sah (ADR-012 butir 3).
    Public(PublicReason),
    /// Login wajib, dengan syarat jenis akun dan sub-peran.
    Require(Requirement),
}

impl Policy {
    /// Pasien (realm `pasien`). Kepemilikan objek diperiksa di service (F1-T38).
    pub fn patient() -> Self {
        Self::accounts(&[AccountType::Patient])
    }

    /// Tenaga kesehatan (realm `nakes`). Penugasan diperiksa di service.
    pub fn health_worker() -> Self {
        Self::accounts(&[AccountType::HealthWorker])
    }

    /// Admin rekanan (realm `admin`). Cakupan tenant dari sesi (KEPUTUSAN-128).
    pub fn partner_admin() -> Self {
        Self::accounts(&[AccountType::PartnerAdmin])
    }

    /// Admin Ruang Sehat yang memegang **salah satu** sub-peran `roles` (UP-18, hak minimum per fungsi).
    /// Daftar kosong menghasilkan kebijakan yang tidak dapat dipenuhi dan ditolak saat start.
    pub fn rs_admin(roles: &[AdminSubRole]) -> Self {
        Self::Require(Requirement {
            account_types: BTreeSet::from([AccountType::RsAdmin]),
            any_sub_role: roles.iter().copied().collect(),
        })
    }

    /// Gabungan jenis akun bebas (tanpa admin Ruang Sehat; pakai [`Policy::rs_admin`] untuk admin).
    pub fn accounts(types: &[AccountType]) -> Self {
        Self::Require(Requirement {
            account_types: types.iter().copied().collect(),
            any_sub_role: BTreeSet::new(),
        })
    }

    /// Menambahkan admin Ruang Sehat dengan sub-peran `roles` ke kebijakan terautentikasi.
    #[must_use]
    pub fn or_rs_admin(self, roles: &[AdminSubRole]) -> Self {
        match self {
            Self::Require(mut r) => {
                r.account_types.insert(AccountType::RsAdmin);
                r.any_sub_role.extend(roles.iter().copied());
                Self::Require(r)
            }
            public @ Self::Public(_) => public,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn realms_follow_account_types() {
        let Policy::Require(r) = Policy::patient().or_rs_admin(&[AdminSubRole::CustomerService])
        else {
            panic!("bukan Require");
        };
        assert_eq!(r.realms(), BTreeSet::from([Realm::Pasien, Realm::Admin]));
        assert!(r.accepts_realm(Realm::Pasien));
        assert!(!r.accepts_realm(Realm::Nakes));
        assert!(!r.accepts_realm(Realm::Rme));
    }

    #[test]
    fn unsatisfiable_requirements_are_detected() {
        let Policy::Require(r) = Policy::rs_admin(&[]) else {
            panic!("bukan Require");
        };
        assert!(!r.is_satisfiable());
        let Policy::Require(r) = Policy::accounts(&[]) else {
            panic!("bukan Require");
        };
        assert!(!r.is_satisfiable());
        let Policy::Require(r) = Policy::partner_admin() else {
            panic!("bukan Require");
        };
        assert!(r.is_satisfiable());
    }

    #[test]
    fn or_rs_admin_keeps_public_public() {
        assert_eq!(
            Policy::Public(PublicReason::Operational).or_rs_admin(&[AdminSubRole::SuperAdmin]),
            Policy::Public(PublicReason::Operational)
        );
    }
}
