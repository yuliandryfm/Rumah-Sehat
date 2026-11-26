//! Pencatatan undangan akun HR realm `korporat` (F1-T105 undangan HR pertama, F1-T107 undang HR lain; US-1903
//! AC1, US-1904 AC2; ADR-030 butir 2, 5, 6, 7; KEPUTUSAN-303, KEPUTUSAN-306).
//!
//! Dipakai dua pemanggil, keduanya di **transaksi beraudit milik mereka** (`audit::in_audited_tx`, tanpa
//! panggilan Keycloak, KEPUTUSAN-125): Verifikator membuat perusahaan (modul `corporate`, `admin_create.rs`)
//! dan HR aktif mengundang HR lain (`corporate_hr_service.rs`).
//!
//! Satu fungsi, [`register_hr_invitation`]: memvalidasi email (ASCII ternormalisasi, kewajiban pemanggil
//! `register_pending_account`, review T1 ADR-030 catatan 5), lalu `register_pending_account` yang mengambil
//! kunci advisory (realm, email) lebih dulu, memeriksa email belum dipakai akun belum terhapus, menyisipkan akun
//! `pending_provisioning` jenis `corporate_admin`, dan mencatat niat `create_user` beserta job `idp_create_user`
//! (tujuan `hr_invitation`). Email aktivasi TIDAK dicatat di sini: worker IdP mencatat niat `idp_send_activation`
//! sendiri setelah pengguna Keycloak dibuat dan `idp_subject` diadopsi (`idp_worker.rs`), sehingga satu undangan
//! = satu niat pembuatan + satu niat email, dan tidak ada jalur "undang ulang" yang membuat akun atau niat
//! ganda: email yang sama ditolak `EmailInUse` selama akun belum terhapus.
//!
//! Kedaluwarsa (24 jam, KEPUTUSAN-306) dan sekali pakai tautan aktivasi ditangani token aksi Keycloak, bukan
//! backend; diuji di `infra/keycloak/test/keycloak_check.py` (`cek_undangan_hr`) dan bukan di test Rust.

use super::admin_domain::normalize_email;
use super::domain::IdpRealm;
use super::idp_domain::IntentPurpose;
use super::idp_service::{
    IntentActor, IntentError, PendingAccount, PendingAccountRequest, register_pending_account,
};
use foundation::{Clock, CorrelationId};
use sqlx::PgConnection;

/// `account.account_type` akun HR perusahaan.
pub(super) const CORPORATE_ADMIN_ACCOUNT_TYPE: &str = "corporate_admin";

/// Galat pencatatan undangan (tanpa isi email).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HrInviteError {
    /// Email bukan alamat ASCII yang sah (3-254 karakter, `lokal@domain`).
    InvalidEmail,
    /// Sudah ada akun belum terhapus dengan email itu di realm `korporat`.
    EmailInUse,
    /// Database tidak tersedia atau niat tidak sah (detail hanya di log).
    Database,
    /// Antrean job.
    Queue,
}

/// Memvalidasi email undangan HR; hasilnya ternormalisasi (ASCII, huruf kecil, tanpa spasi tepi).
pub fn normalize_hr_email(raw: &str) -> Option<String> {
    normalize_email(raw)
}

/// Mencatat undangan HR di transaksi pemanggil. `raw_email` divalidasi di sini; kunci advisory email adalah
/// kunci PERTAMA di transaksi (pemanggil tidak boleh memegang kunci baris atau tabel lebih dulu).
pub async fn register_hr_invitation(
    conn: &mut PgConnection,
    raw_email: &str,
    actor: IntentActor,
    correlation_id: CorrelationId,
    clock: &dyn Clock,
) -> Result<PendingAccount, HrInviteError> {
    let email = normalize_hr_email(raw_email).ok_or(HrInviteError::InvalidEmail)?;
    register_pending_account(
        conn,
        &PendingAccountRequest {
            account_type: CORPORATE_ADMIN_ACCOUNT_TYPE,
            realm: IdpRealm::Korporat,
            email: &email,
            full_name: None,
            purpose: IntentPurpose::HrInvitation,
            actor,
            correlation_id,
        },
        clock,
    )
    .await
    .map_err(|e| match e {
        IntentError::EmailInUse => HrInviteError::EmailInUse,
        IntentError::Queue => HrInviteError::Queue,
        IntentError::InvalidIntent | IntentError::Repo(_) => HrInviteError::Database,
    })
}

/// Mencatat pendaftaran mandiri HR (F1-T101, US-1901; tujuan `hr_self_registration`) di transaksi pemanggil, dengan
/// aturan yang sama dengan [`register_hr_invitation`] (email ASCII ternormalisasi, kunci advisory email PERTAMA,
/// `EmailInUse` bila email sudah dipakai akun belum terhapus). Pelaku niat = `corporate_admin` tanpa ID akun
/// (pendaftar belum punya akun sebelum fungsi ini). `full_name` disimpan di `account.full_name` (anonimkan).
pub async fn register_hr_self_registration(
    conn: &mut PgConnection,
    raw_email: &str,
    full_name: &str,
    correlation_id: CorrelationId,
    clock: &dyn Clock,
) -> Result<PendingAccount, HrInviteError> {
    let email = normalize_hr_email(raw_email).ok_or(HrInviteError::InvalidEmail)?;
    register_pending_account(
        conn,
        &PendingAccountRequest {
            account_type: CORPORATE_ADMIN_ACCOUNT_TYPE,
            realm: IdpRealm::Korporat,
            email: &email,
            full_name: Some(full_name),
            purpose: IntentPurpose::HrSelfRegistration,
            actor: IntentActor {
                actor_type: CORPORATE_ADMIN_ACCOUNT_TYPE,
                account_id: None,
            },
            correlation_id,
        },
        clock,
    )
    .await
    .map_err(|e| match e {
        IntentError::EmailInUse => HrInviteError::EmailInUse,
        IntentError::Queue => HrInviteError::Queue,
        IntentError::InvalidIntent | IntentError::Repo(_) => HrInviteError::Database,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // trace: ADR-030 butir 5 (kewajiban pemanggil register_pending_account: email ASCII ternormalisasi)
    #[test]
    fn email_is_validated_ascii_and_normalized() {
        assert_eq!(
            normalize_hr_email("  HR.Baru@Perusahaan.Example "),
            Some("hr.baru@perusahaan.example".to_string())
        );
        for bad in [
            "",
            "tanpa-at",
            "a@b",
            "hr baru@x.example",
            "hr@x.exa\nmple",
            "hr@@x.example",
            "hr@.example",
            "hr@x.example.",
            "h\u{fc}r@x.example",
            "hr@\u{130}.example",
            "\u{ff21}@x.example",
        ] {
            assert_eq!(normalize_hr_email(bad), None, "{bad:?}");
        }
        let long = format!("{}@x.example", "a".repeat(250));
        assert_eq!(normalize_hr_email(&long), None);
    }
}
