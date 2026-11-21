//! Aturan murni verifikasi identitas (F1-T59, reference implementation US-106/US-201). Tanpa I/O dan tanpa
//! jam sistem (coding-standards §1.2, §5.7).
//!
//! - Status permintaan verifikasi (DD §0.9, A2.6): `pending`, `approved`, `rejected`, `cancelled`, dengan
//!   transisi lewat jalur tulis status ADR-008 (crate `status-flow`, aturan [`VerificationRules`]).
//! - Status identitas yang tampil (AB-05): "Belum diunggah" (tidak ada permintaan efektif), "Menunggu
//!   verifikasi", "Disetujui", "Ditolak" ([`IdentityStatus`]). Permintaan efektif = permintaan terbaru
//!   penerima layanan yang tidak dibatalkan (`care_recipient.latest_verification_request_id`): pembatalan
//!   mengembalikan penunjuk itu ke permintaan efektif sebelumnya, sehingga status kembali seperti sebelum
//!   pengajuan (US-106 AC5, AB-05a b; temuan S-04 klarifikasi AC).
//! - Pengajuan baru diizinkan dari semua status kecuali "Menunggu verifikasi" (US-106 AC1, AC5; AB-05a a).
//! - UP-40: NIK yang sudah disetujui sebagai pemilik akun di penerima layanan lain tidak dapat diajukan
//!   atau disetujui sebagai pemilik akun ([`owner_nik_conflict`]); NIK anggota keluarga boleh di banyak akun.
//! - US-201 AC8: Verifikator tidak memutuskan pengajuan milik akunnya sendiri ([`verifier_may_decide`]).
//! - US-201 AC3/AC3a: alasan penolakan dari daftar tertutup, keterangan wajib untuk `other`, ≤ 500 karakter.

use status_flow::TransitionRules;
use std::time::Duration;

/// Jendela penghitung penolakan NIK (US-106 AC4: "per 24 jam"). Diterapkan sebagai jendela tetap sejak
/// penolakan pertama (usulan teknis F1-T60).
pub(super) const NIK_REJECTION_WINDOW: Duration = Duration::from_secs(24 * 3600);
/// Lama penahanan setelah ambang per akun atau per NIK tercapai (US-106 AC4: "ditahan 24 jam").
pub(super) const NIK_HOLD: Duration = Duration::from_secs(24 * 3600);
/// Per akun: 3 penolakan per 24 jam, lalu pengajuan NIK akun itu ditahan (US-106 AC4).
pub(super) const ACCOUNT_REJECTIONS_BEFORE_HOLD: i64 = 3;
/// Per NIK lintas akun: 3 penolakan NIK yang sama dari akun mana pun per 24 jam (US-106 AC4).
pub(super) const NIK_REJECTIONS_BEFORE_HOLD: i64 = 3;
/// Per alamat IP: 10 penolakan per 24 jam (US-106 AC4).
pub(super) const IP_REJECTIONS_LIMIT: i64 = 10;
/// Seluruh sistem: lebih dari 20 penolakan per jam memicu peringatan di security log (US-106 AC4).
pub(super) const GLOBAL_REJECTION_ALERT_PER_HOUR: i64 = 20;
/// Umur penghitung global per jam (satu jam berjalan + jeda).
pub(super) const GLOBAL_COUNTER_TTL: Duration = Duration::from_secs(2 * 3600);
/// Kuota slot unggah foto KTP per akun per jendela (review F1-T61 N3). `[ASUMSI]` usulan teknis 10 per jam:
/// cukup untuk mengulang foto beberapa kali, membatasi pembuatan slot (baris + URL bertanda tangan) massal.
/// Angka final belum ditetapkan dokumen mana pun.
pub(super) const KTP_UPLOAD_SLOTS_PER_WINDOW: i64 = 10;
/// Jendela kuota slot unggah foto KTP (jendela tetap sejak permintaan pertama). `[ASUMSI]` 1 jam.
pub(super) const KTP_UPLOAD_SLOT_WINDOW: Duration = Duration::from_secs(3600);

/// Status permintaan verifikasi (`identity_verification_request_status_event.to_status`,
/// `identity_verification_request_state.status`; CHECK di migrasi F1-T18).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum VerificationStatus {
    /// Menunggu verifikasi.
    Pending,
    /// Disetujui.
    Approved,
    /// Ditolak.
    Rejected,
    /// Dibatalkan pasien (US-106 AC5); bukan status tampilan.
    Cancelled,
}

impl VerificationStatus {
    /// Kode di database.
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Cancelled => "cancelled",
        }
    }

    /// Mengurai kode database; kode tidak dikenal = `None` (diperlakukan galat data).
    pub(super) fn parse(code: &str) -> Option<Self> {
        match code {
            "pending" => Some(Self::Pending),
            "approved" => Some(Self::Approved),
            "rejected" => Some(Self::Rejected),
            "cancelled" => Some(Self::Cancelled),
            _ => None,
        }
    }
}

/// Pemicu permintaan verifikasi (`identity_verification_request.trigger_type`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TriggerType {
    /// Pasien mengajukan NIK dan foto dokumen (US-106).
    DocumentSubmitted,
    /// Nama atau tanggal lahir berubah setelah disetujui (US-105 AC3); tidak dapat dibatalkan.
    ProfileChanged,
}

impl TriggerType {
    /// Kode di database.
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::DocumentSubmitted => "document_submitted",
            Self::ProfileChanged => "profile_changed",
        }
    }

    /// Mengurai kode database.
    pub(super) fn parse(code: &str) -> Option<Self> {
        match code {
            "document_submitted" => Some(Self::DocumentSubmitted),
            "profile_changed" => Some(Self::ProfileChanged),
            _ => None,
        }
    }
}

/// Jenis dokumen identitas (`identity_document.document_type`; AB-02, AB-03).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DocumentType {
    /// KTP: pemilik akun dan anggota keluarga dewasa.
    Ktp,
    /// Kartu Keluarga: anak (Verifikator memeriksa pemilik akun tercantum sebagai orang tua/wali, US-201 AC4).
    Kk,
}

impl DocumentType {
    /// Kode di database.
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::Ktp => "ktp",
            Self::Kk => "kk",
        }
    }

    /// Mengurai kode database.
    pub(super) fn parse(code: &str) -> Option<Self> {
        match code {
            "ktp" => Some(Self::Ktp),
            "kk" => Some(Self::Kk),
            _ => None,
        }
    }
}

/// Status identitas yang tampil ke pasien (AB-05).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum IdentityStatus {
    /// Belum diunggah: tidak ada permintaan efektif.
    NotUploaded,
    /// Menunggu verifikasi.
    Pending,
    /// Disetujui.
    Approved,
    /// Ditolak (dengan alasan permintaan efektif).
    Rejected,
}

impl IdentityStatus {
    /// Kode API (snake_case).
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::NotUploaded => "not_uploaded",
            Self::Pending => "pending",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
        }
    }

    /// Status tampilan dari status permintaan efektif. Permintaan efektif tidak pernah `cancelled`
    /// (penunjuk dikembalikan saat batal); bila tetap terjadi, `None` = data tidak konsisten (galat).
    pub(super) fn from_effective(effective: Option<VerificationStatus>) -> Option<Self> {
        match effective {
            None => Some(Self::NotUploaded),
            Some(VerificationStatus::Pending) => Some(Self::Pending),
            Some(VerificationStatus::Approved) => Some(Self::Approved),
            Some(VerificationStatus::Rejected) => Some(Self::Rejected),
            Some(VerificationStatus::Cancelled) => None,
        }
    }

    /// Pengajuan dokumen baru diizinkan dari "Belum diunggah", "Ditolak", dan "Disetujui" (US-106 AC1,
    /// AB-05a a); saat "Menunggu verifikasi" foto terkunci sampai diputuskan atau dibatalkan (US-106 AC5).
    pub(super) fn accepts_new_submission(self) -> bool {
        !matches!(self, Self::Pending)
    }
}

/// Pelaku transisi permintaan verifikasi.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum VerificationActor {
    /// Akun pasien pemilik penerima layanan.
    Patient,
    /// Admin Ruang Sehat bersub-peran Verifikator (US-201).
    Verifier,
}

/// Aturan transisi permintaan verifikasi untuk satu pelaku dan pemicu (tabel transisi AB-05/AB-05a):
///
/// | Dari | Ke | Pelaku | Syarat |
/// | --- | --- | --- | --- |
/// | (baru) | `pending` | pasien | — |
/// | `pending` | `approved`, `rejected` | Verifikator | — |
/// | `pending` | `cancelled` | pasien | pemicu `document_submitted` (US-106 AC5) |
///
/// Status `approved`, `rejected`, dan `cancelled` adalah status akhir permintaan; pengajuan berikutnya
/// adalah permintaan baru.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct VerificationRules {
    actor: VerificationActor,
    trigger: TriggerType,
}

impl VerificationRules {
    pub(super) const fn new(actor: VerificationActor, trigger: TriggerType) -> Self {
        Self { actor, trigger }
    }
}

impl TransitionRules<VerificationStatus> for VerificationRules {
    fn allows(&self, from: Option<VerificationStatus>, to: VerificationStatus) -> bool {
        use VerificationActor as A;
        use VerificationStatus as S;
        match (from, to, self.actor) {
            (None, S::Pending, A::Patient) => true,
            (Some(S::Pending), S::Approved | S::Rejected, A::Verifier) => true,
            (Some(S::Pending), S::Cancelled, A::Patient) => {
                self.trigger == TriggerType::DocumentSubmitted
            }
            _ => false,
        }
    }
}

/// UP-40, US-106 AC4, US-201 AC7: NIK pemilik akun yang diajukan atau disetujui bentrok bila registri
/// NIK pemilik akun sudah memuatnya untuk penerima layanan **lain**. `registry_holder` = penerima layanan
/// pemegang baris registri aktif yang cocok (semua versi kunci blind index, TD-10). Anggota keluarga
/// tidak diperiksa (NIK anggota keluarga boleh di lebih dari satu akun).
pub(super) fn owner_nik_conflict(
    is_account_owner: bool,
    care_recipient_id: foundation::Uuid,
    registry_holder: Option<foundation::Uuid>,
) -> bool {
    is_account_owner && registry_holder.is_some_and(|holder| holder != care_recipient_id)
}

/// US-201 AC8 (pemisahan tugas): Verifikator tidak boleh memutuskan pengajuan penerima layanan yang
/// dimiliki akunnya sendiri (pemilik akun maupun anggota keluarga di akunnya).
pub(super) fn verifier_may_decide(
    verifier_account_id: foundation::Uuid,
    owner_account_id: foundation::Uuid,
) -> bool {
    verifier_account_id != owner_account_id
}

/// Batas keterangan penolakan (US-201 AC3a), dihitung dalam karakter Unicode.
pub(super) const REJECTION_NOTE_MAX_CHARS: usize = 500;

/// Alasan penolakan konteks `identity_rejection` (US-201 AC3, daftar awal UP-75 b lewat migrasi data).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RejectionReason {
    /// Foto dokumen buram atau tidak terbaca.
    PhotoUnreadable,
    /// Dokumen bukan KTP/KK, tidak utuh, atau tidak sesuai jenis yang diminta.
    DocumentInvalid,
    /// NIK yang diisi tidak sama dengan NIK di dokumen.
    NikMismatch,
    /// Nama atau tanggal lahir di profil tidak sama dengan dokumen.
    ProfileMismatch,
    /// Pemilik akun tidak tercantum sebagai orang tua/wali di KK (US-201 AC4).
    GuardianNotListed,
    /// NIK tidak dapat dipakai (teks umum US-106 AC4; US-201 AC7).
    NikUnavailable,
    /// Lainnya; keterangan wajib (AC3a).
    Other,
}

impl RejectionReason {
    /// Semua alasan, urutan daftar US-201 AC3.
    pub(super) const ALL: [Self; 7] = [
        Self::PhotoUnreadable,
        Self::DocumentInvalid,
        Self::NikMismatch,
        Self::ProfileMismatch,
        Self::GuardianNotListed,
        Self::NikUnavailable,
        Self::Other,
    ];

    /// Kode (`reason_code.code`, `..._status_event.reason_code`).
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::PhotoUnreadable => "photo_unreadable",
            Self::DocumentInvalid => "document_invalid",
            Self::NikMismatch => "nik_mismatch",
            Self::ProfileMismatch => "profile_mismatch",
            Self::GuardianNotListed => "guardian_not_listed",
            Self::NikUnavailable => "nik_unavailable",
            Self::Other => "other",
        }
    }

    /// Mengurai kode; tidak dikenal = `None`.
    pub(super) fn parse(code: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|r| r.as_str() == code)
    }
}

/// Galat validasi keputusan tolak (US-201 AC3a).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RejectionNoteError {
    /// Alasan `other` tanpa keterangan.
    Required,
    /// Keterangan melebihi [`REJECTION_NOTE_MAX_CHARS`].
    TooLong,
}

/// Menormalkan keterangan penolakan: spasi di awal/akhir dipangkas, kosong = tidak ada. `other` wajib
/// berketerangan; keterangan apa pun paling panjang 500 karakter (US-201 AC3, AC3a).
pub(super) fn normalize_rejection_note(
    reason: RejectionReason,
    note: Option<&str>,
) -> Result<Option<String>, RejectionNoteError> {
    let trimmed = note.map(str::trim).filter(|n| !n.is_empty());
    match trimmed {
        None if reason == RejectionReason::Other => Err(RejectionNoteError::Required),
        None => Ok(None),
        Some(n) if n.chars().count() > REJECTION_NOTE_MAX_CHARS => Err(RejectionNoteError::TooLong),
        Some(n) => Ok(Some(n.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use foundation::new_id;

    const ALL_STATUS: [Option<VerificationStatus>; 5] = [
        None,
        Some(VerificationStatus::Pending),
        Some(VerificationStatus::Approved),
        Some(VerificationStatus::Rejected),
        Some(VerificationStatus::Cancelled),
    ];

    fn allowed(
        actor: VerificationActor,
        trigger: TriggerType,
    ) -> Vec<(&'static str, &'static str)> {
        let rules = VerificationRules::new(actor, trigger);
        let mut out = Vec::new();
        for from in ALL_STATUS {
            for to in ALL_STATUS.into_iter().flatten() {
                if rules.allows(from, to) {
                    out.push((from.map_or("-", VerificationStatus::as_str), to.as_str()));
                }
            }
        }
        out
    }

    #[test]
    fn codes_match_database_check_constraints() {
        for s in ALL_STATUS.into_iter().flatten() {
            assert_eq!(VerificationStatus::parse(s.as_str()), Some(s));
        }
        assert_eq!(VerificationStatus::parse("Pending"), None);
        for t in [TriggerType::DocumentSubmitted, TriggerType::ProfileChanged] {
            assert_eq!(TriggerType::parse(t.as_str()), Some(t));
        }
        for d in [DocumentType::Ktp, DocumentType::Kk] {
            assert_eq!(DocumentType::parse(d.as_str()), Some(d));
        }
        assert_eq!(DocumentType::parse("sim"), None);
    }

    // trace: US-106 AC1, US-201 AC2, AC3, US-106 AC5 (tabel transisi lengkap per pelaku)
    #[test]
    fn transition_table_is_exactly_ab05() {
        use TriggerType::{DocumentSubmitted, ProfileChanged};
        use VerificationActor::{Patient, Verifier};
        assert_eq!(
            allowed(Patient, DocumentSubmitted),
            [("-", "pending"), ("pending", "cancelled")]
        );
        assert_eq!(allowed(Patient, ProfileChanged), [("-", "pending")]);
        for trigger in [DocumentSubmitted, ProfileChanged] {
            assert_eq!(
                allowed(Verifier, trigger),
                [("pending", "approved"), ("pending", "rejected")]
            );
        }
    }

    // trace: US-106 AC5 (pengajuan karena perubahan profil tidak dapat dibatalkan)
    #[test]
    fn profile_change_request_cannot_be_cancelled() {
        let rules = VerificationRules::new(VerificationActor::Patient, TriggerType::ProfileChanged);
        assert!(!rules.allows(
            Some(VerificationStatus::Pending),
            VerificationStatus::Cancelled
        ));
    }

    // trace: US-201 AC5 (keputusan kedua setelah keputusan pertama ditolak), US-106 AC5 (batal kalah)
    #[test]
    fn decisions_and_cancel_are_final() {
        let verifier =
            VerificationRules::new(VerificationActor::Verifier, TriggerType::DocumentSubmitted);
        let patient =
            VerificationRules::new(VerificationActor::Patient, TriggerType::DocumentSubmitted);
        for done in [
            VerificationStatus::Approved,
            VerificationStatus::Rejected,
            VerificationStatus::Cancelled,
        ] {
            for to in [
                VerificationStatus::Approved,
                VerificationStatus::Rejected,
                VerificationStatus::Pending,
            ] {
                assert!(!verifier.allows(Some(done), to));
            }
            assert!(!patient.allows(Some(done), VerificationStatus::Cancelled));
        }
    }

    // trace: US-106 AC1, AC5, AB-05 (status tampilan dan kunci foto)
    #[test]
    fn display_status_and_submission_lock() {
        assert_eq!(
            IdentityStatus::from_effective(None),
            Some(IdentityStatus::NotUploaded)
        );
        assert_eq!(
            IdentityStatus::from_effective(Some(VerificationStatus::Cancelled)),
            None
        );
        let accepts: Vec<_> = [
            IdentityStatus::NotUploaded,
            IdentityStatus::Pending,
            IdentityStatus::Approved,
            IdentityStatus::Rejected,
        ]
        .into_iter()
        .filter(|s| s.accepts_new_submission())
        .map(IdentityStatus::as_str)
        .collect();
        assert_eq!(accepts, ["not_uploaded", "approved", "rejected"]);
    }

    // trace: US-106 AC4, US-201 AC7, UP-40
    #[test]
    fn owner_nik_conflict_only_for_other_holder_and_account_owner() {
        let me = new_id();
        let other = new_id();
        assert!(owner_nik_conflict(true, me, Some(other)));
        assert!(
            !owner_nik_conflict(true, me, Some(me)),
            "NIK sendiri boleh diajukan ulang"
        );
        assert!(!owner_nik_conflict(true, me, None));
        assert!(
            !owner_nik_conflict(false, me, Some(other)),
            "NIK anggota keluarga boleh di banyak akun"
        );
    }

    // trace: US-201 AC8
    #[test]
    fn verifier_cannot_decide_own_account_submission() {
        let verifier = new_id();
        assert!(!verifier_may_decide(verifier, verifier));
        assert!(verifier_may_decide(verifier, new_id()));
    }

    // trace: US-201 AC3, AC3a
    #[test]
    fn rejection_reasons_and_note_rules() {
        let codes: Vec<_> = RejectionReason::ALL.iter().map(|r| r.as_str()).collect();
        assert_eq!(
            codes,
            [
                "photo_unreadable",
                "document_invalid",
                "nik_mismatch",
                "profile_mismatch",
                "guardian_not_listed",
                "nik_unavailable",
                "other"
            ]
        );
        for r in RejectionReason::ALL {
            assert_eq!(RejectionReason::parse(r.as_str()), Some(r));
        }
        assert_eq!(RejectionReason::parse("OTHER"), None);

        use RejectionReason::{NikMismatch, Other};
        assert_eq!(
            normalize_rejection_note(Other, None),
            Err(RejectionNoteError::Required)
        );
        assert_eq!(
            normalize_rejection_note(Other, Some("   ")),
            Err(RejectionNoteError::Required)
        );
        assert_eq!(normalize_rejection_note(NikMismatch, Some(" ")), Ok(None));
        assert_eq!(
            normalize_rejection_note(Other, Some("  foto terpotong ")),
            Ok(Some("foto terpotong".to_string()))
        );
        let max = "é".repeat(REJECTION_NOTE_MAX_CHARS);
        assert_eq!(
            normalize_rejection_note(Other, Some(&max)),
            Ok(Some(max.clone()))
        );
        let over = format!("{max}a");
        assert_eq!(
            normalize_rejection_note(NikMismatch, Some(&over)),
            Err(RejectionNoteError::TooLong)
        );
    }
}
