//! Aturan murni penyediaan identitas lewat worker (ADR-030 butir 2-6, KEPUTUSAN-302, KEPUTUSAN-305). Tanpa I/O.
//!
//! - [`IdpJobKind`]: daftar jenis job **tertutup** (ADR-030 butir 3). Jenis lain tidak pernah dijalankan.
//! - [`IntentPurpose`]: tujuan niat; menentukan susunan required action dan `lifespan` email aksi lewat
//!   [`activation_plan`] (ditetapkan di kode per realm dan per tujuan, tidak pernah dari muatan job atau baris
//!   niat).
//! - [`intent_id_from_payload`]: muatan job hanya ID niat; field lain apa pun = job ditolak tanpa panggilan
//!   Keycloak.
//! - Prasyarat yang diperiksa terhadap **keadaan Keycloak** ([`KcUserFacts`]): atribut `rs_intent_id` dicocokkan
//!   persis, dan pengguna berkredensial tidak pernah dikirimi email aktivasi atau dihapus.

use super::domain::IdpRealm;
use foundation::Uuid;
use platform_jobs::JobPayload;
use status_flow::TransitionRules;

/// Nama atribut user profile Keycloak berisi ID niat pembuat pengguna (ADR-030 butir 6). Dideklarasikan di
/// `infra/keycloak/realms/korporat.json` (hanya Admin, tanpa mapper token).
pub const INTENT_ATTRIBUTE: &str = "rs_intent_id";

/// Satu-satunya kunci muatan job IdP.
pub const PAYLOAD_INTENT_KEY: &str = "intent_id";

/// Jenis job IdP (ADR-030 butir 3). Menambah jenis = revisi ADR-030 + review keamanan T1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IdpJobKind {
    /// `POST users` tanpa kredensial dengan atribut `rs_intent_id`.
    CreateUser,
    /// `PUT users/{id}/execute-actions-email`.
    SendActivation,
    /// `PUT users/{id}` `enabled=false` lalu `POST users/{id}/logout`.
    DisableUser,
    /// `POST users/{id}/logout`.
    LogoutUser,
    /// `DELETE users/{id}` (kompensasi).
    DeleteOrphan,
    /// Pencarian pengguna lewat atribut (hanya baca) dan adopsi `idp_subject`.
    Reconcile,
}

impl IdpJobKind {
    /// Semua jenis.
    pub const ALL: [Self; 6] = [
        Self::CreateUser,
        Self::SendActivation,
        Self::DisableUser,
        Self::LogoutUser,
        Self::DeleteOrphan,
        Self::Reconcile,
    ];

    /// Awalan `job_type` yang dicadangkan untuk job IdP (jenis lain berawalan ini digagalkan penyapu).
    pub const JOB_TYPE_PREFIX: &'static str = "idp_";

    /// Nilai `app.job.job_type`.
    pub fn job_type(self) -> &'static str {
        match self {
            Self::CreateUser => "idp_create_user",
            Self::SendActivation => "idp_send_activation",
            Self::DisableUser => "idp_disable_user",
            Self::LogoutUser => "idp_logout_user",
            Self::DeleteOrphan => "idp_delete_orphan",
            Self::Reconcile => "idp_reconcile",
        }
    }

    /// Jenis dari `job_type`; selain daftar = `None`.
    pub fn from_job_type(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|k| k.job_type() == s)
    }

    /// Nilai kolom `idp_provisioning_intent.operation`.
    pub fn operation(self) -> &'static str {
        match self {
            Self::CreateUser => "create_user",
            Self::SendActivation => "send_activation",
            Self::DisableUser => "disable_user",
            Self::LogoutUser => "logout_user",
            Self::DeleteOrphan => "delete_orphan",
            Self::Reconcile => "reconcile",
        }
    }

    /// Jenis dari kolom `operation`.
    pub fn from_operation(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|k| k.operation() == s)
    }

    /// Operasi ini memeriksa atribut `rs_intent_id` milik niat asal (niat `create_user`).
    pub fn needs_origin(self) -> bool {
        matches!(
            self,
            Self::SendActivation | Self::DeleteOrphan | Self::Reconcile
        )
    }
}

/// Tujuan niat (kolom `purpose`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IntentPurpose {
    /// Pendaftaran mandiri HR (US-1901, KEPUTUSAN-303 R1).
    HrSelfRegistration,
    /// Undangan HR oleh Verifikator atau HR lain (US-1903 AC1, US-1904 AC2).
    HrInvitation,
    /// Penonaktifan akun.
    AccountDeactivation,
    /// Pencabutan sesi sisi server (TD-33).
    SessionRevocation,
    /// Kompensasi penyediaan yang gagal.
    ProvisioningCompensation,
    /// Rekonsiliasi.
    Reconciliation,
}

impl IntentPurpose {
    /// Semua tujuan.
    pub const ALL: [Self; 6] = [
        Self::HrSelfRegistration,
        Self::HrInvitation,
        Self::AccountDeactivation,
        Self::SessionRevocation,
        Self::ProvisioningCompensation,
        Self::Reconciliation,
    ];

    /// Nilai kolom `purpose`.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::HrSelfRegistration => "hr_self_registration",
            Self::HrInvitation => "hr_invitation",
            Self::AccountDeactivation => "account_deactivation",
            Self::SessionRevocation => "session_revocation",
            Self::ProvisioningCompensation => "provisioning_compensation",
            Self::Reconciliation => "reconciliation",
        }
    }

    /// Tujuan dari kolom `purpose`.
    pub fn parse(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|p| p.as_str() == s)
    }
}

/// Pasangan jenis, tujuan, dan realm yang sah (sama dengan CHECK `ck_idp_provisioning_intent_purpose*` di
/// migrasi 20261125100000; diperiksa lagi di kode supaya baris yang disisipkan langsung tetap ditolak).
pub fn valid_combination(kind: IdpJobKind, purpose: IntentPurpose, realm: IdpRealm) -> bool {
    use IdpJobKind as K;
    use IntentPurpose as P;
    let pair = match kind {
        K::CreateUser | K::SendActivation => {
            matches!(purpose, P::HrSelfRegistration | P::HrInvitation)
        }
        K::DisableUser => purpose == P::AccountDeactivation,
        K::LogoutUser => purpose == P::SessionRevocation,
        K::DeleteOrphan => purpose == P::ProvisioningCompensation,
        K::Reconcile => purpose == P::Reconciliation,
    };
    let hr = matches!(purpose, P::HrSelfRegistration | P::HrInvitation);
    pair && (!hr || realm == IdpRealm::Korporat)
}

/// Susunan email aksi per realm dan tujuan (ADR-030 butir 3 dan butir 4 R1, KEPUTUSAN-303).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActivationPlan {
    /// Required action, urut.
    pub actions: &'static [&'static str],
    /// Masa berlaku tautan (detik).
    pub lifespan_seconds: u32,
}

/// Required action tautan aktivasi HR: verifikasi email, atur password, daftar TOTP (KEPUTUSAN-303: password
/// dibuat saat membuka tautan, R1; US-1903 AC1 "password dan TOTP").
pub const HR_ACTIVATION_ACTIONS: &[&str] = &["VERIFY_EMAIL", "UPDATE_PASSWORD", "CONFIGURE_TOTP"];

/// Masa berlaku tautan aktivasi HR: 24 jam, sama dengan undangan admin US-208 AC1
/// (`ACTIVATION_INVITE_LIFESPAN_SECONDS`). PRD US-1901/US-1903/US-1904 tidak menyebut angka
/// `[TIDAK DIKETAHUI]`: nilai ini usulan teknis KEPUTUSAN-306 yang dikonfirmasi saat F1-T101/F1-T105 memakai
/// jalur ini.
pub const HR_ACTIVATION_LIFESPAN_SECONDS: u32 = 86_400;

/// Rencana email aksi; `None` = realm/tujuan tidak punya email aktivasi (job ditolak).
pub fn activation_plan(realm: IdpRealm, purpose: IntentPurpose) -> Option<ActivationPlan> {
    match (realm, purpose) {
        (IdpRealm::Korporat, IntentPurpose::HrSelfRegistration | IntentPurpose::HrInvitation) => {
            Some(ActivationPlan {
                actions: HR_ACTIVATION_ACTIONS,
                lifespan_seconds: HR_ACTIVATION_LIFESPAN_SECONDS,
            })
        }
        _ => None,
    }
}

/// Status niat (ADR-008 jalur tulis status).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntentStatus {
    /// Menunggu worker.
    Pending,
    /// Efek Keycloak sudah terjadi dan hasilnya tercatat.
    Succeeded,
    /// Prasyarat ditolak (keamanan), tidak dicoba ulang.
    Rejected,
    /// Gagal permanen atau percobaan habis.
    Failed,
}

impl IntentStatus {
    /// Nilai kolom status.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Succeeded => "succeeded",
            Self::Rejected => "rejected",
            Self::Failed => "failed",
        }
    }

    /// Status dari kolom.
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(Self::Pending),
            "succeeded" => Some(Self::Succeeded),
            "rejected" => Some(Self::Rejected),
            "failed" => Some(Self::Failed),
            _ => None,
        }
    }
}

/// Transisi niat: dibuat `pending`, lalu satu hasil akhir. Hasil pertama yang tersimpan berlaku.
pub struct IntentRules;

impl TransitionRules<IntentStatus> for IntentRules {
    fn allows(&self, from: Option<IntentStatus>, to: IntentStatus) -> bool {
        use IntentStatus as S;
        matches!(
            (from, to),
            (None, S::Pending) | (Some(S::Pending), S::Succeeded | S::Rejected | S::Failed)
        )
    }
}

/// Alasan penolakan atau kegagalan (kode `reason_code` dan alasan security log; daftar tertutup).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    /// `job_type` di luar [`IdpJobKind`].
    UnknownJobType,
    /// Muatan job memuat selain ID niat.
    InvalidPayload,
    /// Niat tidak ditemukan.
    IntentNotFound,
    /// Jenis job tidak sama dengan operasi niat.
    KindMismatch,
    /// Kombinasi realm/tujuan/operasi tidak sah, atau realm tanpa klien worker.
    InvalidIntent,
    /// Keadaan akun di database tidak memenuhi prasyarat.
    AccountState,
    /// Niat asal tidak sah (bukan `create_user`, atau belum berakhir/berhasil sesuai operasi).
    OriginState,
    /// Pengguna Keycloak tidak ditemukan.
    IdpUserMissing,
    /// Atribut `rs_intent_id` di Keycloak kosong atau tidak sama dengan niat.
    IntentAttributeMismatch,
    /// Pengguna Keycloak sudah punya kredensial.
    Credentialed,
    /// Pengguna Keycloak nonaktif.
    IdpUserDisabled,
    /// Email sudah dipakai pengguna Keycloak lain (409 tanpa atribut yang cocok).
    EmailTaken,
    /// Percobaan habis.
    RetriesExhausted,
}

impl Refusal {
    /// Kode `reason_code`/security log.
    pub fn code(self) -> &'static str {
        match self {
            Self::UnknownJobType => "unknown_job_type",
            Self::InvalidPayload => "invalid_payload",
            Self::IntentNotFound => "intent_not_found",
            Self::KindMismatch => "job_kind_mismatch",
            Self::InvalidIntent => "invalid_intent",
            Self::AccountState => "account_state",
            Self::OriginState => "origin_state",
            Self::IdpUserMissing => "idp_user_missing",
            Self::IntentAttributeMismatch => "intent_attribute_mismatch",
            Self::Credentialed => "idp_user_credentialed",
            Self::IdpUserDisabled => "idp_user_disabled",
            Self::EmailTaken => "idp_email_taken",
            Self::RetriesExhausted => "retries_exhausted",
        }
    }

    /// Penolakan ini tanda penyalahgunaan (peringatan keamanan, ADR-030 butir 3 dan 9), bukan kegagalan biasa.
    pub fn is_security_relevant(self) -> bool {
        !matches!(self, Self::RetriesExhausted | Self::EmailTaken)
    }
}

/// Mengambil ID niat dari muatan job. Muatan wajib tepat satu entri `intent_id` berupa UUID; entri lain apa pun
/// (misalnya email, `redirect_uri`, susunan aksi) menolak job (ADR-030 butir 2 dan 3).
pub fn intent_id_from_payload(payload: &JobPayload) -> Result<Uuid, Refusal> {
    if payload.len() != 1 {
        return Err(Refusal::InvalidPayload);
    }
    payload
        .id(PAYLOAD_INTENT_KEY)
        .ok_or(Refusal::InvalidPayload)
}

/// Fakta pengguna Keycloak yang dibaca worker (bukan dari database aplikasi).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KcUserFacts {
    /// Nilai atribut `rs_intent_id` apa adanya (bisa kosong atau lebih dari satu).
    pub intent_attribute: Vec<String>,
    /// Jenis kredensial yang terdaftar (`password`, `otp`, ...).
    pub credential_types: Vec<String>,
    /// `enabled`.
    pub enabled: bool,
}

impl KcUserFacts {
    /// Atribut berisi tepat satu nilai yang sama persis dengan `intent` (bentuk UUID berhubung huruf kecil).
    pub fn created_by(&self, intent: Uuid) -> bool {
        let want = intent.hyphenated().to_string();
        matches!(self.intent_attribute.as_slice(), [v] if *v == want)
    }
}

/// Prasyarat `idp_send_activation` (ADR-030 butir 3): pengguna dibuat oleh niat asal, aktif, dan belum
/// punya kredensial apa pun.
pub fn may_send_activation(user: &KcUserFacts, origin: Uuid) -> Result<(), Refusal> {
    if !user.created_by(origin) {
        return Err(Refusal::IntentAttributeMismatch);
    }
    if !user.credential_types.is_empty() {
        return Err(Refusal::Credentialed);
    }
    if !user.enabled {
        return Err(Refusal::IdpUserDisabled);
    }
    Ok(())
}

/// Prasyarat `idp_delete_orphan` (ADR-030 butir 3, KEPUTUSAN-201 butir 2): pengguna dibuat oleh niat asal dan
/// tanpa kredensial. Pengguna tanpa atribut atau dengan atribut niat lain tidak pernah dihapus.
pub fn may_delete_orphan(user: &KcUserFacts, origin: Uuid) -> Result<(), Refusal> {
    if !user.created_by(origin) {
        return Err(Refusal::IntentAttributeMismatch);
    }
    if !user.credential_types.is_empty() {
        return Err(Refusal::Credentialed);
    }
    Ok(())
}

/// Email ternormalisasi untuk kunci advisory dan pencarian akun per realm: tanpa spasi tepi, huruf kecil
/// (Keycloak menyimpan username/email huruf kecil).
pub fn normalize_email(email: &str) -> String {
    email.trim().to_lowercase()
}

/// Ruang nama kunci advisory transaksi dua-int4 per (realm, email ternormalisasi) (modul 01 identitas, kunci
/// 01; pola `pg_advisory_xact_lock(1101, ..)` modul konten; KEPUTUSAN-305).
pub const EMAIL_LOCK_NAMESPACE: i32 = 101;

/// Kunci kedua: 4 byte pertama SHA-256 dari `<realm>:<email ternormalisasi>`. Tabrakan hanya membuat dua
/// pendaftaran berbeda berjalan berurutan, tidak pernah melewatkan penguncian.
pub fn email_lock_key(realm: IdpRealm, email: &str) -> i32 {
    let digest =
        crypto::sha256(format!("{}:{}", realm.as_str(), normalize_email(email)).as_bytes());
    let mut b = [0u8; 4];
    for (dst, src) in b.iter_mut().zip(digest.iter()) {
        *dst = *src;
    }
    i32::from_be_bytes(b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use foundation::new_id;

    fn facts(attr: &[&str], creds: &[&str], enabled: bool) -> KcUserFacts {
        KcUserFacts {
            intent_attribute: attr.iter().map(|s| (*s).to_string()).collect(),
            credential_types: creds.iter().map(|s| (*s).to_string()).collect(),
            enabled,
        }
    }

    #[test]
    fn job_types_are_a_closed_list_with_reserved_prefix() {
        for k in IdpJobKind::ALL {
            assert_eq!(IdpJobKind::from_job_type(k.job_type()), Some(k));
            assert_eq!(IdpJobKind::from_operation(k.operation()), Some(k));
            assert!(k.job_type().starts_with(IdpJobKind::JOB_TYPE_PREFIX));
            assert!(platform_jobs::JobType::new(k.job_type()).is_ok());
        }
        for bad in [
            "idp_reset_password",
            "idp_remove_credentials",
            "idp_update_email",
            "idp_role_mapping",
            "IDP_CREATE_USER",
            "",
        ] {
            assert_eq!(IdpJobKind::from_job_type(bad), None, "{bad}");
        }
    }

    // trace: ADR-030 butir 2 (muatan hanya ID niat)
    #[test]
    fn payload_must_hold_only_the_intent_id() {
        let id = new_id();
        let ok = JobPayload::new().with_id(PAYLOAD_INTENT_KEY, id).unwrap();
        assert_eq!(intent_id_from_payload(&ok), Ok(id));
        let extra = JobPayload::new()
            .with_id(PAYLOAD_INTENT_KEY, id)
            .unwrap()
            .with_code("actions", "update_password")
            .unwrap();
        assert_eq!(intent_id_from_payload(&extra), Err(Refusal::InvalidPayload));
        let other_key = JobPayload::new().with_id("account_id", id).unwrap();
        assert_eq!(
            intent_id_from_payload(&other_key),
            Err(Refusal::InvalidPayload)
        );
        let as_code = JobPayload::new()
            .with_code(PAYLOAD_INTENT_KEY, "bukan_uuid")
            .unwrap();
        assert_eq!(
            intent_id_from_payload(&as_code),
            Err(Refusal::InvalidPayload)
        );
        assert_eq!(
            intent_id_from_payload(&JobPayload::new()),
            Err(Refusal::InvalidPayload)
        );
    }

    #[test]
    fn combinations_follow_the_migration_checks() {
        use IdpJobKind as K;
        use IntentPurpose as P;
        assert!(valid_combination(
            K::CreateUser,
            P::HrInvitation,
            IdpRealm::Korporat
        ));
        assert!(valid_combination(
            K::SendActivation,
            P::HrSelfRegistration,
            IdpRealm::Korporat
        ));
        assert!(!valid_combination(
            K::CreateUser,
            P::HrInvitation,
            IdpRealm::Pasien
        ));
        assert!(!valid_combination(
            K::DeleteOrphan,
            P::HrInvitation,
            IdpRealm::Korporat
        ));
        assert!(valid_combination(
            K::DisableUser,
            P::AccountDeactivation,
            IdpRealm::Admin
        ));
        assert!(!valid_combination(
            K::LogoutUser,
            P::AccountDeactivation,
            IdpRealm::Korporat
        ));
        assert!(valid_combination(
            K::Reconcile,
            P::Reconciliation,
            IdpRealm::Korporat
        ));
    }

    #[test]
    fn activation_plan_is_fixed_in_code_per_realm_and_purpose() {
        let p = activation_plan(IdpRealm::Korporat, IntentPurpose::HrInvitation).unwrap();
        assert_eq!(
            p.actions,
            &["VERIFY_EMAIL", "UPDATE_PASSWORD", "CONFIGURE_TOTP"]
        );
        assert_eq!(p.lifespan_seconds, 86_400);
        assert!(activation_plan(IdpRealm::Pasien, IntentPurpose::HrInvitation).is_none());
        assert!(activation_plan(IdpRealm::Korporat, IntentPurpose::AccountDeactivation).is_none());
    }

    #[test]
    fn intent_transitions_allow_one_terminal_result() {
        use IntentStatus as S;
        let r = IntentRules;
        assert!(r.allows(None, S::Pending));
        assert!(r.allows(Some(S::Pending), S::Succeeded));
        assert!(r.allows(Some(S::Pending), S::Rejected));
        assert!(r.allows(Some(S::Pending), S::Failed));
        for from in [S::Succeeded, S::Rejected, S::Failed] {
            for to in [S::Pending, S::Succeeded, S::Rejected, S::Failed] {
                assert!(!r.allows(Some(from), to), "{from:?} -> {to:?}");
            }
        }
        assert!(!r.allows(None, S::Succeeded));
    }

    // trace: ADR-030 test negatif idp_send_activation
    #[test]
    fn activation_requires_matching_attribute_and_no_credentials() {
        let origin = new_id();
        let id = origin.hyphenated().to_string();
        assert_eq!(
            may_send_activation(&facts(&[&id], &[], true), origin),
            Ok(())
        );
        assert_eq!(
            may_send_activation(&facts(&[&id], &["password"], true), origin),
            Err(Refusal::Credentialed)
        );
        assert_eq!(
            may_send_activation(&facts(&[&id], &["otp"], true), origin),
            Err(Refusal::Credentialed)
        );
        let other = new_id().hyphenated().to_string();
        assert_eq!(
            may_send_activation(&facts(&[&other], &[], true), origin),
            Err(Refusal::IntentAttributeMismatch)
        );
        assert_eq!(
            may_send_activation(&facts(&[], &[], true), origin),
            Err(Refusal::IntentAttributeMismatch)
        );
        assert_eq!(
            may_send_activation(&facts(&[&id, &id], &[], true), origin),
            Err(Refusal::IntentAttributeMismatch)
        );
        assert_eq!(
            may_send_activation(&facts(&[&id.to_uppercase()], &[], true), origin),
            Err(Refusal::IntentAttributeMismatch)
        );
        assert_eq!(
            may_send_activation(&facts(&[&id], &[], false), origin),
            Err(Refusal::IdpUserDisabled)
        );
    }

    // trace: ADR-030 test negatif idp_delete_orphan, KEPUTUSAN-201 butir 2
    #[test]
    fn orphan_deletion_never_touches_foreign_or_credentialed_users() {
        let origin = new_id();
        let id = origin.hyphenated().to_string();
        assert_eq!(may_delete_orphan(&facts(&[&id], &[], true), origin), Ok(()));
        assert_eq!(
            may_delete_orphan(&facts(&[&id], &[], false), origin),
            Ok(())
        );
        assert_eq!(
            may_delete_orphan(&facts(&[&id], &["password"], true), origin),
            Err(Refusal::Credentialed)
        );
        assert_eq!(
            may_delete_orphan(&facts(&[], &[], true), origin),
            Err(Refusal::IntentAttributeMismatch)
        );
        assert_eq!(
            may_delete_orphan(
                &facts(&[&new_id().hyphenated().to_string()], &[], true),
                origin
            ),
            Err(Refusal::IntentAttributeMismatch)
        );
    }

    #[test]
    fn email_lock_key_is_per_realm_and_normalized() {
        let a = email_lock_key(IdpRealm::Korporat, "  HR@Contoh.Invalid ");
        assert_eq!(a, email_lock_key(IdpRealm::Korporat, "hr@contoh.invalid"));
        assert_ne!(a, email_lock_key(IdpRealm::Pasien, "hr@contoh.invalid"));
        assert_ne!(a, email_lock_key(IdpRealm::Korporat, "hr2@contoh.invalid"));
    }

    #[test]
    fn refusal_codes_are_job_error_codes() {
        for r in [
            Refusal::UnknownJobType,
            Refusal::InvalidPayload,
            Refusal::IntentNotFound,
            Refusal::KindMismatch,
            Refusal::InvalidIntent,
            Refusal::AccountState,
            Refusal::OriginState,
            Refusal::IdpUserMissing,
            Refusal::IntentAttributeMismatch,
            Refusal::Credentialed,
            Refusal::IdpUserDisabled,
            Refusal::EmailTaken,
            Refusal::RetriesExhausted,
        ] {
            assert!(platform_jobs::ErrorCode::new(r.code()).is_ok(), "{r:?}");
        }
    }
}
