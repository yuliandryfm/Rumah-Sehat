//! Jenis akun dan sub-peran admin Ruang Sehat. Kode sama dengan CHECK di migrasi
//! `db/migrations/app/20261119100000_identity_account.sql` (`ck_account_account_type`,
//! `ck_account_idp_realm`, `ck_account_account_type_idp_realm`, `ck_account_status`,
//! `ck_rs_admin_role_sub_role`).

use authn::Realm;
use std::fmt;

/// Jenis akun aplikasi (`account.account_type`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AccountType {
    /// Pasien (realm `pasien`).
    Patient,
    /// Tenaga kesehatan (realm `nakes`).
    HealthWorker,
    /// Admin Ruang Sehat (realm `admin`), dengan sub-peran UP-18.
    RsAdmin,
    /// Admin rekanan (realm `admin`).
    PartnerAdmin,
}

impl AccountType {
    /// Kode di database.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Patient => "patient",
            Self::HealthWorker => "health_worker",
            Self::RsAdmin => "rs_admin",
            Self::PartnerAdmin => "partner_admin",
        }
    }

    /// Mengurai kode database; kode tidak dikenal = `None` (ditolak).
    pub fn parse(code: &str) -> Option<Self> {
        match code {
            "patient" => Some(Self::Patient),
            "health_worker" => Some(Self::HealthWorker),
            "rs_admin" => Some(Self::RsAdmin),
            "partner_admin" => Some(Self::PartnerAdmin),
            _ => None,
        }
    }

    /// Realm Keycloak akun jenis ini (`ck_account_account_type_idp_realm`).
    pub const fn realm(self) -> Realm {
        match self {
            Self::Patient => Realm::Pasien,
            Self::HealthWorker => Realm::Nakes,
            Self::RsAdmin | Self::PartnerAdmin => Realm::Admin,
        }
    }
}

impl fmt::Display for AccountType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Status akun (`account.status`). Hanya `Active` yang boleh mengakses endpoint terautentikasi (KU-10).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountStatus {
    /// Menunggu aktivasi.
    PendingActivation,
    /// Aktif.
    Active,
    /// Nonaktif.
    Inactive,
}

impl AccountStatus {
    /// Mengurai kode database; kode tidak dikenal = `None` (ditolak).
    pub fn parse(code: &str) -> Option<Self> {
        match code {
            "pending_activation" => Some(Self::PendingActivation),
            "active" => Some(Self::Active),
            "inactive" => Some(Self::Inactive),
            _ => None,
        }
    }
}

/// Sub-peran admin Ruang Sehat (PRD §1.2 `[UP-18]`, `rs_admin_role.sub_role`). Tidak ada sub-peran
/// yang otomatis memegang hak sub-peran lain: Super Admin pun hanya lolos kebijakan yang menyebutnya
/// (PRD §3.1: misalnya tidak ada admin yang melihat isi klinis). PRD v1.3 tidak menambah sub-peran
/// untuk CR-021 (Verifikator memperoleh tugas korporat; PT-67 masih terbuka soal sub-peran baru).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AdminSubRole {
    /// Super Admin.
    SuperAdmin,
    /// Verifikator.
    Verifier,
    /// Layanan Pelanggan.
    CustomerService,
    /// Keuangan.
    Finance,
    /// Konten.
    Content,
    /// Auditor.
    Auditor,
    /// Petugas PDP.
    PrivacyOfficer,
}

impl AdminSubRole {
    /// Semua sub-peran.
    pub const ALL: [AdminSubRole; 7] = [
        Self::SuperAdmin,
        Self::Verifier,
        Self::CustomerService,
        Self::Finance,
        Self::Content,
        Self::Auditor,
        Self::PrivacyOfficer,
    ];

    /// Kode di database.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SuperAdmin => "super_admin",
            Self::Verifier => "verifier",
            Self::CustomerService => "customer_service",
            Self::Finance => "finance",
            Self::Content => "content",
            Self::Auditor => "auditor",
            Self::PrivacyOfficer => "privacy_officer",
        }
    }

    /// Mengurai kode database; kode tidak dikenal = `None` (diabaikan, tidak memberi hak).
    pub fn parse(code: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|r| r.as_str() == code)
    }
}

impl fmt::Display for AdminSubRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_match_database_check_constraints() {
        let codes: Vec<&str> = AdminSubRole::ALL.iter().map(|r| r.as_str()).collect();
        assert_eq!(
            codes,
            [
                "super_admin",
                "verifier",
                "customer_service",
                "finance",
                "content",
                "auditor",
                "privacy_officer"
            ]
        );
        for r in AdminSubRole::ALL {
            assert_eq!(AdminSubRole::parse(r.as_str()), Some(r));
        }
        assert_eq!(AdminSubRole::parse("Super_Admin"), None);
        assert_eq!(AdminSubRole::parse(""), None);
        for t in [
            AccountType::Patient,
            AccountType::HealthWorker,
            AccountType::RsAdmin,
            AccountType::PartnerAdmin,
        ] {
            assert_eq!(AccountType::parse(t.as_str()), Some(t));
        }
        assert_eq!(AccountType::parse("admin"), None);
        assert_eq!(AccountStatus::parse("active"), Some(AccountStatus::Active));
        assert_eq!(AccountStatus::parse("ACTIVE"), None);
    }

    #[test]
    fn account_type_realm_matches_database_constraint() {
        assert_eq!(AccountType::Patient.realm(), Realm::Pasien);
        assert_eq!(AccountType::HealthWorker.realm(), Realm::Nakes);
        assert_eq!(AccountType::RsAdmin.realm(), Realm::Admin);
        assert_eq!(AccountType::PartnerAdmin.realm(), Realm::Admin);
    }
}
