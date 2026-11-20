//! Aturan murni akun dan sub-peran admin Ruang Sehat (F1-T65, US-208 AC1–AC7, klarifikasi T-19 / UP-76,
//! ADR-012 butir 7). Tanpa I/O: service membaca keadaan di bawah kunci tabel lalu memanggil aturan ini.
//!
//! - AC1: kombinasi sub-peran apa pun dari tujuh sub-peran, minimal satu.
//! - AC2: Super Admin aktif terakhir tidak dapat dicabut sub-perannya maupun dinonaktifkan.
//! - AC3: admin tidak dapat mengubah sub-peran akunnya sendiri.
//! - AC4: bootstrap hanya bila tidak ada Super Admin aktif maupun menunggu aktivasi; undangan ulang hanya
//!   bila tidak ada Super Admin aktif dan tepat satu menunggu aktivasi.
//! - AC5: admin tidak dapat menonaktifkan akunnya sendiri.
//! - AC6: Auditor eksklusif (tidak digabung dengan sub-peran lain).
//! - AC7: Auditor aktif terakhir tidak dapat dicabut sub-perannya maupun dinonaktifkan.

use authz::{AccountStatus, AdminSubRole};
use foundation::Uuid;
use std::collections::BTreeSet;

/// Umur undangan aktivasi admin: 24 jam (US-208 AC4, 6.4.1 L1 [SRC-265]).
pub const ACTIVATION_INVITE_LIFESPAN_SECONDS: u32 = 24 * 60 * 60;

/// Pelanggaran aturan akun admin (arti bisnis; pemetaan ke kode respons di `error.rs`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdminRuleViolation {
    /// Daftar sub-peran kosong.
    NoSubRole,
    /// Auditor digabung dengan sub-peran lain (AC6).
    AuditorNotExclusive,
    /// Mengubah sub-peran akun sendiri (AC3).
    SelfRoleChange,
    /// Menonaktifkan akun sendiri (AC5).
    SelfDeactivation,
    /// Mencabut/menonaktifkan Super Admin aktif terakhir (AC2).
    LastSuperAdmin,
    /// Mencabut/menonaktifkan Auditor aktif terakhir (AC7).
    LastAuditor,
    /// Pelaku bukan Super Admin aktif (dibaca ulang dari database, M-13).
    ActorNotSuperAdmin,
    /// Akun sasaran sudah nonaktif.
    TargetInactive,
    /// Sub-peran yang diminta sama dengan yang sudah dipegang.
    NoChange,
    /// Bootstrap ditolak: sudah ada Super Admin aktif atau menunggu aktivasi (AC4).
    SuperAdminExists,
    /// Undangan ulang bootstrap ditolak: ada Super Admin aktif, atau tidak tepat satu yang menunggu aktivasi.
    NoPendingBootstrapAccount,
}

/// Akun admin Ruang Sehat seperti tersimpan (hanya akun `rs_admin`, belum dihapus).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdminAccount {
    /// `account.id`.
    pub id: Uuid,
    /// Status akun.
    pub status: AccountStatus,
    /// Sub-peran aktif (`rs_admin_role` tanpa `deleted_at`).
    pub sub_roles: BTreeSet<AdminSubRole>,
}

impl AdminAccount {
    fn is_active_with(&self, role: AdminSubRole) -> bool {
        self.status == AccountStatus::Active && self.sub_roles.contains(&role)
    }
}

/// Seluruh akun admin Ruang Sehat yang belum dihapus, dibaca di bawah kunci tabel `rs_admin_role`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AdminCensus {
    /// Semua akun admin.
    pub accounts: Vec<AdminAccount>,
}

impl AdminCensus {
    /// Akun dengan ID tertentu.
    pub fn get(&self, id: Uuid) -> Option<&AdminAccount> {
        self.accounts.iter().find(|a| a.id == id)
    }

    fn active_with(&self, role: AdminSubRole, except: Uuid) -> usize {
        self.accounts
            .iter()
            .filter(|a| a.id != except && a.is_active_with(role))
            .count()
    }

    fn with_status(&self, role: AdminSubRole, status: AccountStatus) -> Vec<&AdminAccount> {
        self.accounts
            .iter()
            .filter(|a| a.status == status && a.sub_roles.contains(&role))
            .collect()
    }
}

/// Validasi kombinasi sub-peran (AC1, AC6).
pub fn validate_sub_roles(roles: &BTreeSet<AdminSubRole>) -> Result<(), AdminRuleViolation> {
    if roles.is_empty() {
        return Err(AdminRuleViolation::NoSubRole);
    }
    if roles.contains(&AdminSubRole::Auditor) && roles.len() > 1 {
        return Err(AdminRuleViolation::AuditorNotExclusive);
    }
    Ok(())
}

/// Pelaku wajib Super Admin aktif menurut database saat ini.
pub fn require_super_admin(census: &AdminCensus, actor: Uuid) -> Result<(), AdminRuleViolation> {
    match census.get(actor) {
        Some(a) if a.is_active_with(AdminSubRole::SuperAdmin) => Ok(()),
        _ => Err(AdminRuleViolation::ActorNotSuperAdmin),
    }
}

/// Sisa pemegang aktif setelah `target` kehilangan `role` (AC2, AC7).
fn guard_last_holder(
    census: &AdminCensus,
    target: &AdminAccount,
    losing: &BTreeSet<AdminSubRole>,
) -> Result<(), AdminRuleViolation> {
    if target.status != AccountStatus::Active {
        return Ok(());
    }
    for (role, violation) in [
        (AdminSubRole::SuperAdmin, AdminRuleViolation::LastSuperAdmin),
        (AdminSubRole::Auditor, AdminRuleViolation::LastAuditor),
    ] {
        if losing.contains(&role) && census.active_with(role, target.id) == 0 {
            return Err(violation);
        }
    }
    Ok(())
}

/// Perubahan sub-peran yang sah: yang ditambahkan dan yang dicabut.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoleDiff {
    /// Sub-peran baru.
    pub granted: BTreeSet<AdminSubRole>,
    /// Sub-peran yang dicabut.
    pub revoked: BTreeSet<AdminSubRole>,
}

/// AC1–AC3, AC6, AC7: Super Admin `actor` mengganti sub-peran akun `target` menjadi `wanted`.
/// Akun sasaran yang tidak ada atau bukan admin Ruang Sehat ditangani service (`NotFound`).
pub fn plan_role_change(
    census: &AdminCensus,
    actor: Uuid,
    target: &AdminAccount,
    wanted: &BTreeSet<AdminSubRole>,
) -> Result<RoleDiff, AdminRuleViolation> {
    require_super_admin(census, actor)?;
    if actor == target.id {
        return Err(AdminRuleViolation::SelfRoleChange);
    }
    if target.status == AccountStatus::Inactive {
        return Err(AdminRuleViolation::TargetInactive);
    }
    validate_sub_roles(wanted)?;
    let granted: BTreeSet<_> = wanted.difference(&target.sub_roles).copied().collect();
    let revoked: BTreeSet<_> = target.sub_roles.difference(wanted).copied().collect();
    if granted.is_empty() && revoked.is_empty() {
        return Err(AdminRuleViolation::NoChange);
    }
    guard_last_holder(census, target, &revoked)?;
    Ok(RoleDiff { granted, revoked })
}

/// AC2, AC5, AC7: Super Admin `actor` menonaktifkan akun `target`.
pub fn check_deactivation(
    census: &AdminCensus,
    actor: Uuid,
    target: &AdminAccount,
) -> Result<(), AdminRuleViolation> {
    require_super_admin(census, actor)?;
    if actor == target.id {
        return Err(AdminRuleViolation::SelfDeactivation);
    }
    if target.status == AccountStatus::Inactive {
        return Err(AdminRuleViolation::TargetInactive);
    }
    guard_last_holder(census, target, &target.sub_roles)
}

/// AC1: Super Admin `actor` membuat akun admin dengan `roles`.
pub fn check_creation(
    census: &AdminCensus,
    actor: Uuid,
    roles: &BTreeSet<AdminSubRole>,
) -> Result<(), AdminRuleViolation> {
    require_super_admin(census, actor)?;
    validate_sub_roles(roles)
}

/// AC4: bootstrap hanya bila tidak ada Super Admin aktif maupun menunggu aktivasi.
pub fn check_bootstrap(census: &AdminCensus) -> Result<(), AdminRuleViolation> {
    let exists = census.accounts.iter().any(|a| {
        a.sub_roles.contains(&AdminSubRole::SuperAdmin)
            && matches!(
                a.status,
                AccountStatus::Active | AccountStatus::PendingActivation
            )
    });
    if exists {
        Err(AdminRuleViolation::SuperAdminExists)
    } else {
        Ok(())
    }
}

/// AC4 (undangan kedaluwarsa): undangan ulang lewat perintah bootstrap hanya bila belum ada Super Admin
/// aktif dan tepat satu Super Admin menunggu aktivasi; mengembalikan akun itu.
pub fn bootstrap_reissue_target(census: &AdminCensus) -> Result<Uuid, AdminRuleViolation> {
    if !census
        .with_status(AdminSubRole::SuperAdmin, AccountStatus::Active)
        .is_empty()
    {
        return Err(AdminRuleViolation::NoPendingBootstrapAccount);
    }
    match census
        .with_status(AdminSubRole::SuperAdmin, AccountStatus::PendingActivation)
        .as_slice()
    {
        [only] => Ok(only.id),
        _ => Err(AdminRuleViolation::NoPendingBootstrapAccount),
    }
}

/// Sub-peran sensitif yang pemberiannya wajib diberitahukan ke semua Super Admin dan Auditor aktif
/// (US-208 AC8, UP-76 d-2). Kanal notifikasi admin (US-1003) ada di Fase 2; sampai saat itu pemberian
/// tercatat di E-11 dan ditandai `sensitive_grant` supaya dapat ditinjau.
pub fn is_sensitive(role: AdminSubRole) -> bool {
    matches!(
        role,
        AdminSubRole::SuperAdmin
            | AdminSubRole::Auditor
            | AdminSubRole::Verifier
            | AdminSubRole::Finance
            | AdminSubRole::PrivacyOfficer
    )
}

/// Alamat email admin: bentuk sederhana `lokal@domain` (validasi akhir oleh Keycloak), 3–254 karakter,
/// tanpa spasi atau karakter kontrol, huruf kecil (Keycloak menyimpan username huruf kecil).
pub fn normalize_email(raw: &str) -> Option<String> {
    let e = raw.trim().to_ascii_lowercase();
    let (local, domain) = e.split_once('@')?;
    let ok = (3..=254).contains(&e.len())
        && !local.is_empty()
        && local.len() <= 64
        && domain.contains('.')
        && !domain.starts_with('.')
        && !domain.ends_with('.')
        && !domain.contains('@')
        && e.chars().all(|c| c.is_ascii_graphic());
    ok.then_some(e)
}

/// Nama lengkap admin: 1–200 karakter setelah dipangkas, tanpa karakter kontrol.
pub fn normalize_full_name(raw: &str) -> Option<String> {
    let n = raw.trim();
    let len = n.chars().count();
    ((1..=200).contains(&len) && !n.chars().any(char::is_control)).then(|| n.to_string())
}

/// Alasan penonaktifan: 1–500 karakter setelah dipangkas, tanpa karakter kontrol selain baris baru.
pub fn normalize_reason(raw: &str) -> Option<String> {
    let n = raw.trim();
    let len = n.chars().count();
    ((1..=500).contains(&len) && !n.chars().any(|c| c.is_control() && c != '\n'))
        .then(|| n.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use AdminSubRole::*;

    fn id(n: u128) -> Uuid {
        Uuid::from_u128(n)
    }

    fn acc(n: u128, status: AccountStatus, roles: &[AdminSubRole]) -> AdminAccount {
        AdminAccount {
            id: id(n),
            status,
            sub_roles: roles.iter().copied().collect(),
        }
    }

    fn set(roles: &[AdminSubRole]) -> BTreeSet<AdminSubRole> {
        roles.iter().copied().collect()
    }

    fn census(accounts: Vec<AdminAccount>) -> AdminCensus {
        AdminCensus { accounts }
    }

    const A: AccountStatus = AccountStatus::Active;
    const P: AccountStatus = AccountStatus::PendingActivation;
    const I: AccountStatus = AccountStatus::Inactive;

    #[test]
    fn sub_role_combinations_follow_ac1_and_ac6() {
        assert_eq!(
            validate_sub_roles(&set(&[])),
            Err(AdminRuleViolation::NoSubRole)
        );
        assert_eq!(validate_sub_roles(&set(&[Auditor])), Ok(()));
        for other in AdminSubRole::ALL.into_iter().filter(|r| *r != Auditor) {
            assert_eq!(
                validate_sub_roles(&set(&[Auditor, other])),
                Err(AdminRuleViolation::AuditorNotExclusive),
                "{other}"
            );
            assert_eq!(validate_sub_roles(&set(&[other])), Ok(()));
        }
        // Kombinasi lain diizinkan di rilis 1 (T-19 c).
        assert_eq!(
            validate_sub_roles(&set(&[SuperAdmin, Verifier, Finance, Content])),
            Ok(())
        );
    }

    #[test]
    fn only_active_super_admin_may_manage() {
        let c = census(vec![
            acc(1, A, &[SuperAdmin]),
            acc(2, P, &[SuperAdmin]),
            acc(3, I, &[SuperAdmin]),
            acc(4, A, &[Verifier]),
            acc(5, A, &[Auditor]),
        ]);
        assert_eq!(require_super_admin(&c, id(1)), Ok(()));
        for n in [2, 3, 4, 5, 99] {
            assert_eq!(
                require_super_admin(&c, id(n)),
                Err(AdminRuleViolation::ActorNotSuperAdmin),
                "{n}"
            );
            assert_eq!(
                check_creation(&c, id(n), &set(&[Verifier])),
                Err(AdminRuleViolation::ActorNotSuperAdmin)
            );
        }
        assert_eq!(check_creation(&c, id(1), &set(&[Verifier])), Ok(()));
        assert_eq!(
            check_creation(&c, id(1), &set(&[Auditor, Content])),
            Err(AdminRuleViolation::AuditorNotExclusive)
        );
    }

    #[test]
    fn own_roles_cannot_be_changed_and_own_account_cannot_be_deactivated() {
        let c = census(vec![acc(1, A, &[SuperAdmin]), acc(2, A, &[SuperAdmin])]);
        let me = c.get(id(1)).cloned().unwrap();
        assert_eq!(
            plan_role_change(&c, id(1), &me, &set(&[SuperAdmin, Verifier])),
            Err(AdminRuleViolation::SelfRoleChange)
        );
        assert_eq!(
            check_deactivation(&c, id(1), &me),
            Err(AdminRuleViolation::SelfDeactivation)
        );
    }

    #[test]
    fn last_active_super_admin_is_protected() {
        let c = census(vec![
            acc(1, A, &[SuperAdmin, Verifier]),
            acc(2, A, &[SuperAdmin]),
            acc(3, P, &[SuperAdmin]),
            acc(4, I, &[SuperAdmin]),
        ]);
        let two = c.get(id(2)).cloned().unwrap();
        // Masih ada Super Admin aktif lain (1): boleh.
        assert!(plan_role_change(&c, id(1), &two, &set(&[Content])).is_ok());
        assert_eq!(check_deactivation(&c, id(1), &two), Ok(()));

        // Hanya 2 yang aktif; yang menunggu aktivasi (3) dan nonaktif (4) tidak dihitung.
        let c = census(vec![
            acc(1, A, &[Verifier]),
            acc(2, A, &[SuperAdmin]),
            acc(3, P, &[SuperAdmin]),
            acc(4, I, &[SuperAdmin]),
            acc(5, A, &[SuperAdmin]),
        ]);
        let two = c.get(id(2)).cloned().unwrap();
        // Pelaku 5 menurunkan 2: 5 sendiri masih Super Admin aktif → boleh.
        assert!(plan_role_change(&c, id(5), &two, &set(&[Verifier])).is_ok());
        let c2 = census(vec![acc(2, A, &[SuperAdmin]), acc(3, P, &[SuperAdmin])]);
        let three = c2.get(id(3)).cloned().unwrap();
        // Super Admin menunggu aktivasi boleh dicabut (bukan Super Admin aktif).
        assert!(plan_role_change(&c2, id(2), &three, &set(&[Content])).is_ok());
        assert_eq!(check_deactivation(&c2, id(2), &three), Ok(()));
    }

    #[test]
    fn last_super_admin_guard_counts_only_other_active_holders() {
        // Satu-satunya Super Admin aktif lain = pelaku sendiri, jadi menurunkan target tetap menyisakan pelaku.
        // Kasus "terakhir" murni: target satu-satunya Super Admin aktif dan pelaku... tidak mungkin Super
        // Admin (pelaku wajib Super Admin aktif). Karena itu guard diuji langsung.
        let target = acc(2, A, &[SuperAdmin, Auditor]);
        let c = census(vec![
            target.clone(),
            acc(3, P, &[SuperAdmin]),
            acc(4, I, &[Auditor]),
        ]);
        assert_eq!(
            guard_last_holder(&c, &target, &set(&[SuperAdmin])),
            Err(AdminRuleViolation::LastSuperAdmin)
        );
        assert_eq!(
            guard_last_holder(&c, &target, &set(&[Auditor])),
            Err(AdminRuleViolation::LastAuditor)
        );
        assert_eq!(guard_last_holder(&c, &target, &set(&[Verifier])), Ok(()));
        let pending = acc(3, P, &[SuperAdmin]);
        assert_eq!(guard_last_holder(&c, &pending, &set(&[SuperAdmin])), Ok(()));
    }

    #[test]
    fn last_active_auditor_is_protected() {
        let c = census(vec![
            acc(1, A, &[SuperAdmin]),
            acc(2, A, &[SuperAdmin]),
            acc(10, A, &[Auditor]),
            acc(11, P, &[Auditor]),
            acc(12, I, &[Auditor]),
        ]);
        let auditor = c.get(id(10)).cloned().unwrap();
        assert_eq!(
            plan_role_change(&c, id(1), &auditor, &set(&[Content])),
            Err(AdminRuleViolation::LastAuditor)
        );
        assert_eq!(
            check_deactivation(&c, id(1), &auditor),
            Err(AdminRuleViolation::LastAuditor)
        );
        let mut c2 = c.clone();
        c2.accounts.push(acc(13, A, &[Auditor]));
        assert!(plan_role_change(&c2, id(1), &auditor, &set(&[Content])).is_ok());
        assert_eq!(check_deactivation(&c2, id(1), &auditor), Ok(()));
    }

    #[test]
    fn auditor_exclusivity_applies_both_ways_on_change() {
        let c = census(vec![
            acc(1, A, &[SuperAdmin]),
            acc(2, A, &[Verifier]),
            acc(3, A, &[Auditor]),
            acc(4, A, &[Auditor]),
        ]);
        let verifier = c.get(id(2)).cloned().unwrap();
        let auditor = c.get(id(3)).cloned().unwrap();
        assert_eq!(
            plan_role_change(&c, id(1), &verifier, &set(&[Verifier, Auditor])),
            Err(AdminRuleViolation::AuditorNotExclusive)
        );
        assert_eq!(
            plan_role_change(&c, id(1), &auditor, &set(&[Auditor, Content])),
            Err(AdminRuleViolation::AuditorNotExclusive)
        );
        // Mengganti Verifikator menjadi Auditor saja: sah.
        let d = plan_role_change(&c, id(1), &verifier, &set(&[Auditor])).unwrap();
        assert_eq!(d.granted, set(&[Auditor]));
        assert_eq!(d.revoked, set(&[Verifier]));
    }

    #[test]
    fn role_change_rejects_inactive_target_empty_and_no_change() {
        let c = census(vec![
            acc(1, A, &[SuperAdmin]),
            acc(2, I, &[Verifier]),
            acc(3, A, &[Verifier]),
        ]);
        let inactive = c.get(id(2)).cloned().unwrap();
        let three = c.get(id(3)).cloned().unwrap();
        assert_eq!(
            plan_role_change(&c, id(1), &inactive, &set(&[Content])),
            Err(AdminRuleViolation::TargetInactive)
        );
        assert_eq!(
            check_deactivation(&c, id(1), &inactive),
            Err(AdminRuleViolation::TargetInactive)
        );
        assert_eq!(
            plan_role_change(&c, id(1), &three, &set(&[])),
            Err(AdminRuleViolation::NoSubRole)
        );
        assert_eq!(
            plan_role_change(&c, id(1), &three, &set(&[Verifier])),
            Err(AdminRuleViolation::NoChange)
        );
    }

    #[test]
    fn bootstrap_only_without_active_or_pending_super_admin() {
        assert_eq!(check_bootstrap(&AdminCensus::default()), Ok(()));
        let others = census(vec![
            acc(1, A, &[Verifier]),
            acc(2, I, &[SuperAdmin]),
            acc(3, P, &[Auditor]),
        ]);
        assert_eq!(check_bootstrap(&others), Ok(()));
        for status in [A, P] {
            let c = census(vec![acc(9, status, &[SuperAdmin, Content])]);
            assert_eq!(
                check_bootstrap(&c),
                Err(AdminRuleViolation::SuperAdminExists)
            );
        }
    }

    #[test]
    fn bootstrap_reissue_needs_exactly_one_pending_and_no_active_super_admin() {
        assert_eq!(
            bootstrap_reissue_target(&AdminCensus::default()),
            Err(AdminRuleViolation::NoPendingBootstrapAccount)
        );
        let one = census(vec![acc(7, P, &[SuperAdmin]), acc(8, I, &[SuperAdmin])]);
        assert_eq!(bootstrap_reissue_target(&one), Ok(id(7)));
        let with_active = census(vec![acc(7, P, &[SuperAdmin]), acc(9, A, &[SuperAdmin])]);
        assert_eq!(
            bootstrap_reissue_target(&with_active),
            Err(AdminRuleViolation::NoPendingBootstrapAccount)
        );
        let two = census(vec![acc(7, P, &[SuperAdmin]), acc(9, P, &[SuperAdmin])]);
        assert_eq!(
            bootstrap_reissue_target(&two),
            Err(AdminRuleViolation::NoPendingBootstrapAccount)
        );
    }

    #[test]
    fn sensitive_sub_roles_follow_ac8() {
        let sensitive: Vec<_> = AdminSubRole::ALL
            .into_iter()
            .filter(|r| is_sensitive(*r))
            .collect();
        assert_eq!(
            sensitive,
            vec![SuperAdmin, Verifier, Finance, Auditor, PrivacyOfficer]
        );
    }

    #[test]
    fn inputs_are_normalized_or_rejected() {
        assert_eq!(
            normalize_email("  Admin.Satu@Example.Invalid "),
            Some("admin.satu@example.invalid".into())
        );
        for bad in [
            "",
            "tanpa-at",
            "@example.invalid",
            "a@b",
            "a b@example.invalid",
            "a@@b.c",
            "a@.b",
            "a@b.",
            "a@b\t.c",
        ] {
            assert_eq!(normalize_email(bad), None, "{bad:?}");
        }
        assert_eq!(
            normalize_email(&format!("{}@example.invalid", "a".repeat(65))),
            None
        );
        assert_eq!(normalize_full_name("  Budi Uji "), Some("Budi Uji".into()));
        assert_eq!(normalize_full_name("   "), None);
        assert_eq!(normalize_full_name("a\u{7}b"), None);
        assert_eq!(normalize_full_name(&"x".repeat(201)), None);
        assert_eq!(
            normalize_reason("Pindah tugas\nper 1 Maret"),
            Some("Pindah tugas\nper 1 Maret".into())
        );
        assert_eq!(normalize_reason(""), None);
        assert_eq!(normalize_reason(&"x".repeat(501)), None);
    }
}
