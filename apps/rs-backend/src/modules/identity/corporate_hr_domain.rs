//! Aturan murni kelola anggota HR perusahaan (F1-T107, US-1904 AC2, AC5; ADR-027 butir 3, ADR-028 butir 1).
//! Tanpa I/O; dipanggil di transaksi berkunci (`corporate::api::lock_members`, pola KEPUTUSAN-290).
//!
//! - "HR aktif" = keanggotaan belum dicabut **dan** akun berstatus `active`. HR yang diundang tetapi belum
//!   aktivasi (`pending_activation`) atau yang penggunanya belum dibuat worker (`pending_provisioning`,
//!   ADR-030 butir 5) tidak dihitung.
//! - Undangan HR lain (US-1904 AC2): pemanggil wajib masih HR aktif saat transaksi berkunci membaca ulang
//!   daftar ([`check_invite`]); sama dengan penonaktifan, supaya HR yang baru dinonaktifkan tidak sempat
//!   menambah anggota.
//! - HR yang penggunanya belum dibuat (`provisioning`) tidak dapat dinonaktifkan sampai worker menyelesaikan
//!   penyediaan (`TargetProvisioning`): tanpa `idp_subject` belum ada pengguna Keycloak untuk dinonaktifkan dan
//!   adopsi worker akan ditolak untuk akun yang sudah `inactive`.
//! - HR aktif terakhir perusahaan tidak dapat dinonaktifkan (US-1904 AC2). Selama pemanggil wajib HR aktif
//!   dan tidak boleh menonaktifkan dirinya sendiri, aturan ini sudah tersirat; tetap diperiksa eksplisit
//!   ([`active_after_removal`]) supaya invarian tidak bergantung pada urutan pemeriksaan lain.
//! - HR tidak dapat menonaktifkan dirinya sendiri `[ASUMSI]`: PRD US-1904 AC2 hanya menyebut "HR lain";
//!   mengikuti pola US-208 AC5 untuk admin (arahan orkestrator F1-T107, menunggu konfirmasi PRD).
//! - Pemanggil wajib masih HR aktif saat transaksi berkunci membaca ulang daftar: dua HR yang saling
//!   menonaktifkan bersamaan tidak menyisakan nol HR aktif (yang kedua ditolak).

use authz::AccountStatus;
use foundation::Uuid;

/// Satu anggota HR perusahaan beserta status akunnya.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HrMember {
    /// `corporate_member.id`.
    pub member_id: Uuid,
    /// `account.id`.
    pub account_id: Uuid,
    /// Status akun. Akun `pending_provisioning` dinyatakan `PendingActivation` dengan [`Self::provisioning`].
    pub status: AccountStatus,
    /// Akun `pending_provisioning`: pengguna Keycloak belum dibuat worker (ADR-030 butir 5).
    pub provisioning: bool,
}

/// Semua anggota HR (keanggotaan belum dicabut) satu perusahaan.
#[derive(Debug, Clone, Default)]
pub struct HrRoster {
    /// Anggota.
    pub members: Vec<HrMember>,
}

impl HrRoster {
    fn by_member(&self, member_id: Uuid) -> Option<&HrMember> {
        self.members.iter().find(|m| m.member_id == member_id)
    }

    fn by_account(&self, account_id: Uuid) -> Option<&HrMember> {
        self.members.iter().find(|m| m.account_id == account_id)
    }
}

/// Jumlah HR aktif yang tersisa bila keanggotaan `member_id` dicabut.
pub fn active_after_removal(roster: &HrRoster, member_id: Uuid) -> usize {
    roster
        .members
        .iter()
        .filter(|m| m.member_id != member_id && m.status == AccountStatus::Active)
        .count()
}

/// Pelanggaran aturan kelola anggota HR.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HrRuleViolation {
    /// Pemanggil tidak lagi HR aktif perusahaan itu (dinonaktifkan HR lain sesaat sebelumnya).
    ActorNotActive,
    /// Keanggotaan sasaran tidak ada di perusahaan pemanggil (perusahaan lain atau sudah dicabut).
    TargetNotFound,
    /// HR menonaktifkan dirinya sendiri `[ASUMSI]` (lihat dokumentasi modul).
    SelfDeactivation,
    /// Akun sasaran sudah nonaktif.
    TargetInactive,
    /// Sasaran adalah HR aktif terakhir perusahaan (US-1904 AC2).
    LastActiveHr,
    /// Sasaran masih menunggu pembuatan pengguna Keycloak oleh worker (`pending_provisioning`).
    TargetProvisioning,
}

impl HrRuleViolation {
    /// Kode alasan untuk audit hasil gagal dan log.
    pub const fn reason_code(self) -> &'static str {
        match self {
            Self::ActorNotActive => "actor_not_active",
            Self::TargetNotFound => "target_not_found",
            Self::SelfDeactivation => "self_deactivation",
            Self::TargetInactive => "target_inactive",
            Self::LastActiveHr => "last_active_hr",
            Self::TargetProvisioning => "target_provisioning",
        }
    }
}

/// Keputusan penonaktifan keanggotaan `target_member` oleh akun `actor`. Mengembalikan anggota sasaran.
pub fn check_deactivation(
    roster: &HrRoster,
    actor: Uuid,
    target_member: Uuid,
) -> Result<HrMember, HrRuleViolation> {
    let actor_active = roster
        .by_account(actor)
        .is_some_and(|m| m.status == AccountStatus::Active);
    if !actor_active {
        return Err(HrRuleViolation::ActorNotActive);
    }
    let target = roster
        .by_member(target_member)
        .ok_or(HrRuleViolation::TargetNotFound)?;
    if target.account_id == actor {
        return Err(HrRuleViolation::SelfDeactivation);
    }
    if target.provisioning {
        return Err(HrRuleViolation::TargetProvisioning);
    }
    if target.status == AccountStatus::Inactive {
        return Err(HrRuleViolation::TargetInactive);
    }
    if active_after_removal(roster, target.member_id) == 0 {
        return Err(HrRuleViolation::LastActiveHr);
    }
    Ok(target.clone())
}

/// Keputusan undangan HR lain oleh akun `actor` (US-1904 AC2): pemanggil masih HR aktif perusahaan itu.
pub fn check_invite(roster: &HrRoster, actor: Uuid) -> Result<(), HrRuleViolation> {
    let actor_active = roster
        .by_account(actor)
        .is_some_and(|m| m.status == AccountStatus::Active && !m.provisioning);
    if actor_active {
        Ok(())
    } else {
        Err(HrRuleViolation::ActorNotActive)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use foundation::new_id;

    fn member(status: AccountStatus) -> HrMember {
        HrMember {
            member_id: new_id(),
            account_id: new_id(),
            status,
            provisioning: false,
        }
    }

    fn roster(members: &[&HrMember]) -> HrRoster {
        HrRoster {
            members: members.iter().map(|m| (*m).clone()).collect(),
        }
    }

    // trace: US-1904 AC2 (HR menonaktifkan HR lain, termasuk undangan yang belum aktivasi)
    #[test]
    fn active_hr_may_deactivate_another_active_or_invited_hr() {
        let a = member(AccountStatus::Active);
        let b = member(AccountStatus::Active);
        let c = member(AccountStatus::PendingActivation);
        let r = roster(&[&a, &b, &c]);
        assert_eq!(check_deactivation(&r, a.account_id, b.member_id), Ok(b));
        assert_eq!(check_deactivation(&r, a.account_id, c.member_id), Ok(c));
    }

    // trace: US-1904 AC2 (diri sendiri ditolak, [ASUMSI] pola US-208 AC5)
    #[test]
    fn self_deactivation_is_rejected_even_with_other_active_hr() {
        let a = member(AccountStatus::Active);
        let b = member(AccountStatus::Active);
        let r = roster(&[&a, &b]);
        assert_eq!(
            check_deactivation(&r, a.account_id, a.member_id),
            Err(HrRuleViolation::SelfDeactivation)
        );
    }

    // trace: US-1904 AC2 (HR aktif terakhir tidak dapat dinonaktifkan)
    #[test]
    fn active_count_after_removal_ignores_invited_and_inactive_hr() {
        let only = member(AccountStatus::Active);
        let invited = member(AccountStatus::PendingActivation);
        let gone = member(AccountStatus::Inactive);
        let r = roster(&[&only, &invited, &gone]);
        assert_eq!(active_after_removal(&r, only.member_id), 0);
        assert_eq!(active_after_removal(&r, invited.member_id), 1);
        let second = member(AccountStatus::Active);
        let r = roster(&[&only, &second, &invited]);
        assert_eq!(active_after_removal(&r, only.member_id), 1);
    }

    // trace: US-1904 AC2, KEPUTUSAN-290 (pemanggil yang baru dinonaktifkan tidak dapat lanjut)
    #[test]
    fn actor_no_longer_active_is_rejected() {
        let a = member(AccountStatus::Inactive);
        let b = member(AccountStatus::Active);
        assert_eq!(
            check_deactivation(&roster(&[&a, &b]), a.account_id, b.member_id),
            Err(HrRuleViolation::ActorNotActive)
        );
        // Keanggotaan pemanggil sudah dicabut (tidak ada di daftar): yang tersisa HR aktif terakhir.
        assert_eq!(
            check_deactivation(&roster(&[&b]), a.account_id, b.member_id),
            Err(HrRuleViolation::ActorNotActive)
        );
    }

    // trace: US-1904 AC3 (keanggotaan perusahaan lain tidak ada di daftar pemanggil)
    #[test]
    fn unknown_or_foreign_member_is_not_found_and_inactive_target_conflicts() {
        let a = member(AccountStatus::Active);
        let gone = member(AccountStatus::Inactive);
        let r = roster(&[&a, &gone]);
        assert_eq!(
            check_deactivation(&r, a.account_id, new_id()),
            Err(HrRuleViolation::TargetNotFound)
        );
        assert_eq!(
            check_deactivation(&r, a.account_id, gone.member_id),
            Err(HrRuleViolation::TargetInactive)
        );
    }
}

#[cfg(test)]
mod invite_tests {
    use super::*;
    use foundation::new_id;

    fn member(status: AccountStatus, provisioning: bool) -> HrMember {
        HrMember {
            member_id: new_id(),
            account_id: new_id(),
            status,
            provisioning,
        }
    }

    fn roster(members: &[&HrMember]) -> HrRoster {
        HrRoster {
            members: members.iter().map(|m| (*m).clone()).collect(),
        }
    }

    // trace: US-1904 AC2 (hanya HR aktif yang mengundang; undangan, nonaktif, dan bukan anggota ditolak)
    #[test]
    fn only_an_active_member_may_invite() {
        let active = member(AccountStatus::Active, false);
        let invited = member(AccountStatus::PendingActivation, false);
        let provisioning = member(AccountStatus::PendingActivation, true);
        let gone = member(AccountStatus::Inactive, false);
        let r = roster(&[&active, &invited, &provisioning, &gone]);
        assert_eq!(check_invite(&r, active.account_id), Ok(()));
        for m in [&invited, &provisioning, &gone] {
            assert_eq!(
                check_invite(&r, m.account_id),
                Err(HrRuleViolation::ActorNotActive)
            );
        }
        assert_eq!(
            check_invite(&r, new_id()),
            Err(HrRuleViolation::ActorNotActive)
        );
    }

    // trace: ADR-030 butir 5 (HR yang penggunanya belum dibuat tidak dapat dinonaktifkan; tidak dihitung aktif)
    #[test]
    fn provisioning_hr_cannot_be_deactivated_and_is_not_active() {
        let a = member(AccountStatus::Active, false);
        let p = member(AccountStatus::PendingActivation, true);
        let r = roster(&[&a, &p]);
        assert_eq!(
            check_deactivation(&r, a.account_id, p.member_id),
            Err(HrRuleViolation::TargetProvisioning)
        );
        assert_eq!(active_after_removal(&r, a.member_id), 0);
    }
}
