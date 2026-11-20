//! Keputusan atas objek di tingkat service (ADR-012 butir 1 dan 4, KU-03, coding-standards §3.4, §5.4).
//!
//! Objek yang tidak boleh diakses dijawab persis seperti objek yang tidak ada (`404` `4040001`).
//! Percobaannya dicatat di latar belakang lewat [`crate::DenialSink`] (security log `access_denied`
//! `foreign_resource_id`; data sensitif juga event audit E-05 hasil gagal), tanpa menambah waktu tanggap
//! jalur penolakan (F1-T38, TD-15 Low).
//!
//! Relasi kepemilikan pasien (F1-T38): penerima layanan (`care_recipient`) dimiliki akun pasien lewat
//! `patient.account_id` (DD A2.1, A2.2, KEPUTUSAN-41). Hanya akun pasien pemiliknya yang lolos
//! [`care_recipient_access`]; bukti yang dihasilkan ([`OwnedCareRecipient`]) hanya dapat dibangun crate
//! ini, sehingga repository data penerima layanan yang menerimanya tidak dapat dipanggil tanpa
//! pemeriksaan (pola KEPUTUSAN-128).

use crate::denial::{DeniedAccess, SensitiveTarget};
use crate::error::AuthzError;
use crate::principal::Principal;
use crate::role::AccountType;
use foundation::Uuid;
use observability::security::{self, AccessDenial};

/// Apakah objek yang diperiksa termasuk data sensitif (PRD §3.1, kolom "Sensitif" data dictionary).
/// Data sensitif yang ditolak wajib tercatat di audit trail dengan hasil gagal (KU-03, E-05).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sensitivity {
    /// Bukan data sensitif: penolakan hanya masuk security log.
    NonSensitive,
    /// Data sensitif: penolakan masuk security log dan audit trail (E-05, hasil gagal).
    Sensitive,
}

fn deny(principal: &Principal, route: &str, sensitive: Option<SensitiveTarget>) {
    principal.report_denial(DeniedAccess {
        account_id: principal.account_id(),
        account_type: principal.account_type(),
        sub_roles: principal.sub_roles().iter().copied().collect(),
        route: route.to_string(),
        correlation_id: principal.correlation_id(),
        client_ip: principal.client_ip(),
        sensitive,
    });
}

/// Mengembalikan `found` bila ada **dan** `allowed` menyetujuinya; selain itu
/// [`AuthzError::NotFound`] yang sama untuk "tidak ada" dan "tidak boleh". `route` adalah pola rute
/// (misalnya `/patient/v1/orders/{order_id}`), bukan URL berisi ID. Untuk data sensitif pakai
/// [`visible_sensitive_object`].
pub fn visible_object<T>(
    principal: &Principal,
    route: &str,
    found: Option<T>,
    allowed: impl FnOnce(&T) -> bool,
) -> Result<T, AuthzError> {
    match found {
        None => Err(AuthzError::NotFound),
        Some(obj) if allowed(&obj) => Ok(obj),
        Some(_) => {
            deny(principal, route, None);
            Err(AuthzError::NotFound)
        }
    }
}

/// Seperti [`visible_object`] untuk data sensitif: penolakan juga dicatat sebagai event audit E-05
/// hasil gagal dengan subjek data dari `target` (KU-03).
pub fn visible_sensitive_object<T>(
    principal: &Principal,
    route: &str,
    found: Option<T>,
    allowed: impl FnOnce(&T) -> bool,
    target: impl FnOnce(&T) -> SensitiveTarget,
) -> Result<T, AuthzError> {
    match found {
        None => Err(AuthzError::NotFound),
        Some(obj) if allowed(&obj) => Ok(obj),
        Some(obj) => {
            deny(principal, route, Some(target(&obj)));
            Err(AuthzError::NotFound)
        }
    }
}

/// Pemilik penerima layanan menurut database (dibaca modul profil): `patient.account_id` dan
/// `care_recipient.is_account_owner`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CareRecipientOwner {
    /// `patient.account_id` pemilik.
    pub account_id: Uuid,
    /// Penerima layanan adalah pemilik akun sendiri (bukan anggota keluarga).
    pub is_account_owner: bool,
}

/// Bukti bahwa pemanggil adalah akun pasien pemilik penerima layanan ini. Hanya dibangun
/// [`care_recipient_access`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OwnedCareRecipient {
    care_recipient_id: Uuid,
    account_id: Uuid,
    is_account_owner: bool,
}

impl OwnedCareRecipient {
    /// `care_recipient.id`.
    pub fn care_recipient_id(&self) -> Uuid {
        self.care_recipient_id
    }

    /// Akun pasien pemilik (= pemanggil).
    pub fn account_id(&self) -> Uuid {
        self.account_id
    }

    /// `true` bila penerima layanan adalah pemilik akun sendiri. Bila `false` (anggota keluarga),
    /// membuka data klinisnya wajib dicatat E-05 berhasil sebelum data dikirim (PRD §3.1, E-05).
    pub fn is_account_owner(&self) -> bool {
        self.is_account_owner
    }
}

/// Relasi kepemilikan pasien atas penerima layanan (ADR-012 butir 1, KU-03). `owner` = hasil baca
/// pemilik dari modul profil (`None` = tidak ada atau terhapus). Lolos hanya bila pemanggil akun pasien
/// dan `owner.account_id` sama dengan akunnya; selain itu `404` yang identik dengan "tidak ada", dan
/// penolakan atas penerima layanan yang ada dicatat (data sensitif: E-05 hasil gagal).
pub fn care_recipient_access(
    principal: &Principal,
    route: &str,
    care_recipient_id: Uuid,
    owner: Option<CareRecipientOwner>,
    sensitivity: Sensitivity,
) -> Result<OwnedCareRecipient, AuthzError> {
    let Some(owner) = owner else {
        return Err(AuthzError::NotFound);
    };
    if principal.account_type() == AccountType::Patient
        && owner.account_id == principal.account_id()
    {
        return Ok(OwnedCareRecipient {
            care_recipient_id,
            account_id: owner.account_id,
            is_account_owner: owner.is_account_owner,
        });
    }
    let target = match sensitivity {
        Sensitivity::NonSensitive => None,
        Sensitivity::Sensitive => Some(SensitiveTarget {
            object_type: "care_recipient",
            object_id: care_recipient_id,
            subject_care_recipient_id: care_recipient_id,
        }),
    };
    deny(principal, route, target);
    Err(AuthzError::NotFound)
}

/// Penolakan aksi yang dikenal tetapi tidak diizinkan untuk peran pemanggil (`403` `4030001`), dengan
/// security log `access_denied` (`role_not_allowed`). Untuk aturan peran yang hanya dapat diputuskan
/// di service (misalnya pemisahan tugas ADR-012 butir 7). Keberadaan fungsinya tidak rahasia, jadi
/// dicatat langsung.
pub fn deny_action(principal: &Principal, route: &str) -> AuthzError {
    security::access_denied(
        Some(principal.account_id()),
        route,
        AccessDenial::RoleNotAllowed,
    );
    AuthzError::Forbidden
}
