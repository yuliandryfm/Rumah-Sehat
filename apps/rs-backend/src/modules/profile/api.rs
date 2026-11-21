//! Antarmuka publik modul `profile` untuk modul lain dan untuk perakitan aplikasi.
//!
//! - F1-T38: pemilik penerima layanan untuk relasi kepemilikan. Pola pemeriksaan relasi (ADR-012 butir 1
//!   dan 4, KU-03): service pemakai membaca pemilik dengan [`care_recipient_owner`], lalu memutuskan lewat
//!   `authz::care_recipient_access`; bukti `authz::OwnedCareRecipient` yang dihasilkan menjadi syarat
//!   fungsi data penerima layanan. Query yang sama dijalankan untuk ID yang tidak ada maupun milik orang
//!   lain, sehingga keduanya tidak dapat dibedakan dari waktu tanggap.
//! - F1-T60 (reference implementation US-106): pengajuan identitas ([`submit_identity_document`]),
//!   pembatalan ([`cancel_identity_submission`]), dan status ([`identity_status`]) dengan dependensinya
//!   dalam satu wadah [`IdentityServices`] yang dirakit aplikasi.

use super::{repo, service};
use authz::{CareRecipientOwner, Principal};
use crypto::{DataKeyring, NikBlindIndexKeyring};
use foundation::{Clock, Uuid};
use sqlx::PgPool;
use std::fmt;
use std::sync::Arc;
use time::OffsetDateTime;
use valkey::Valkey;

pub use super::error::ProfileError;
use crate::modules::platform::api::{FileStorage, UploadTicket};

/// Pemilik penerima layanan `care_recipient_id` (`patient.account_id`, `is_account_owner`). `None` bila
/// penerima layanan atau pasiennya tidak ada atau terhapus (ADR-003).
pub async fn care_recipient_owner(
    pool: &PgPool,
    care_recipient_id: Uuid,
) -> Result<Option<CareRecipientOwner>, ProfileError> {
    let row = repo::care_recipient_owner(pool, care_recipient_id)
        .await
        .map_err(|_| ProfileError::Database)?;
    Ok(row.map(|r| CareRecipientOwner::new(care_recipient_id, r.account_id, r.is_account_owner)))
}

/// Dependensi use case identitas (dirakit sekali saat start; murah dibagi lewat `Arc`).
pub struct IdentityServices {
    pub(super) pool: PgPool,
    pub(super) clock: Arc<dyn Clock>,
    pub(super) files: Arc<FileStorage>,
    pub(super) keyring: Arc<dyn DataKeyring>,
    pub(super) blind_index: Arc<NikBlindIndexKeyring>,
    pub(super) valkey: Valkey,
}

impl IdentityServices {
    /// `pool` = role aplikasi; `keyring` = DEK lewat OpenBao Transit; `blind_index` = kunci blind index
    /// NIK semua versi aktif; `valkey` = penghitung batas NIK berlapis (US-106 AC4).
    pub fn new(
        pool: PgPool,
        clock: Arc<dyn Clock>,
        files: Arc<FileStorage>,
        keyring: Arc<dyn DataKeyring>,
        blind_index: Arc<NikBlindIndexKeyring>,
        valkey: Valkey,
    ) -> Self {
        Self {
            pool,
            clock,
            files,
            keyring,
            blind_index,
            valkey,
        }
    }
}

impl fmt::Debug for IdentityServices {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IdentityServices").finish_non_exhaustive()
    }
}

/// Pengajuan yang diterima (status "Menunggu verifikasi").
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubmissionAccepted {
    /// Permintaan verifikasi baru.
    pub identity_verification_request_id: Uuid,
    /// Dokumen identitas baru.
    pub identity_document_id: Uuid,
    /// Kode status tampilan (`pending`).
    pub status: &'static str,
}

/// Status identitas yang tampil ke pemilik (AB-05).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentityStatusView {
    /// `not_uploaded` | `pending` | `approved` | `rejected`.
    pub status: &'static str,
    /// Permintaan efektif (tidak ada bila "Belum diunggah").
    pub identity_verification_request_id: Option<Uuid>,
    /// Kode alasan penolakan (hanya bila `rejected`, US-201 AC3).
    pub rejection_reason: Option<String>,
    /// Empat digit terakhir NIK permintaan efektif.
    pub nik_last4: Option<String>,
    /// Waktu pengajuan efektif.
    pub submitted_at: Option<OffsetDateTime>,
    /// Pengajuan baru diizinkan (US-106 AC1, AC5).
    pub can_submit: bool,
    /// Pengajuan yang menunggu dapat dibatalkan (US-106 AC5).
    pub can_cancel: bool,
}

/// US-106: pengajuan NIK + foto KTP (slot unggah dari modul `platform`) untuk penerima layanan milik
/// pemanggil. `route` = pola rute untuk security log.
pub async fn submit_identity_document(
    services: &IdentityServices,
    principal: &Principal,
    route: &str,
    care_recipient_id: Uuid,
    nik: &str,
    upload_slot_id: Uuid,
) -> Result<SubmissionAccepted, ProfileError> {
    service::submit(
        services,
        principal,
        route,
        care_recipient_id,
        nik,
        upload_slot_id,
    )
    .await
}

/// US-106 AC5: pembatalan pengajuan dokumen yang masih menunggu.
pub async fn cancel_identity_submission(
    services: &IdentityServices,
    principal: &Principal,
    route: &str,
    care_recipient_id: Uuid,
) -> Result<IdentityStatusView, ProfileError> {
    service::cancel(services, principal, route, care_recipient_id).await
}

/// AB-05: status identitas penerima layanan milik pemanggil.
pub async fn identity_status(
    services: &IdentityServices,
    principal: &Principal,
    route: &str,
    care_recipient_id: Uuid,
) -> Result<IdentityStatusView, ProfileError> {
    service::status(services, principal, route, care_recipient_id).await
}

/// US-106 AC3 (F1-T61): slot unggah foto KTP untuk akun pemanggil (modul `platform`, ADR-018).
pub async fn create_ktp_upload_slot(
    services: &IdentityServices,
    principal: &Principal,
    route: &str,
    content_type: String,
    size_bytes: u64,
) -> Result<UploadTicket, ProfileError> {
    service::create_ktp_upload_slot(services, principal, route, content_type, size_bytes).await
}
