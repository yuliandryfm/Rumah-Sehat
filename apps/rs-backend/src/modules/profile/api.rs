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

use super::{profile_service, repo, service, verification};
use authz::{CareRecipientOwner, Principal};
use crypto::{DataKeyring, NikBlindIndexKeyring};
use foundation::{Clock, Uuid};
use sqlx::PgPool;
use std::fmt;
use std::sync::Arc;
use time::OffsetDateTime;
use valkey::Valkey;

pub use super::error::ProfileError;
use crate::modules::platform::api::{DownloadTicket, FileStorage, UploadTicket};

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
    /// Keterangan penolakan dari Verifikator (hanya bila `rejected` dan diisi, US-201 AC3a).
    pub rejection_note: Option<String>,
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

// ----------------------------------------------------------------------------------------------------
// Verifikator (F1-T62, reference implementation US-201). Dipanggil handler backoffice modul ini;
// kebijakan rute = sub-peran Verifikator.

/// Satu pengajuan di antrean Verifikator (US-201 AC1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueueItemView {
    /// Permintaan verifikasi.
    pub identity_verification_request_id: Uuid,
    /// Penerima layanan pemilik dokumen.
    pub care_recipient_id: Uuid,
    /// Pemilik akun (bukan anggota keluarga).
    pub is_account_owner: bool,
    /// Nama di profil.
    pub full_name: String,
    /// Tanggal lahir di profil.
    pub birth_date: time::Date,
    /// `male` | `female`.
    pub sex: String,
    /// `ktp` | `kk`.
    pub document_type: &'static str,
    /// `document_submitted` | `profile_changed`.
    pub trigger_type: &'static str,
    /// Empat digit terakhir NIK.
    pub nik_last4: String,
    /// Waktu pengajuan.
    pub submitted_at: OffsetDateTime,
}

/// Detail pengajuan untuk Verifikator dengan NIK lengkap (US-201 AC1, PRD §3.1).
#[derive(Clone, PartialEq, Eq)]
pub struct VerificationDetailView {
    /// Permintaan verifikasi.
    pub identity_verification_request_id: Uuid,
    /// Penerima layanan pemilik dokumen.
    pub care_recipient_id: Uuid,
    /// Pemilik akun (bukan anggota keluarga).
    pub is_account_owner: bool,
    /// Nama di profil.
    pub full_name: String,
    /// Tanggal lahir di profil.
    pub birth_date: time::Date,
    /// `male` | `female`.
    pub sex: String,
    /// `ktp` | `kk`.
    pub document_type: &'static str,
    /// `document_submitted` | `profile_changed`.
    pub trigger_type: &'static str,
    /// NIK lengkap (rahasia: tidak pernah dicatat di log).
    pub nik: String,
    /// Waktu pengajuan.
    pub submitted_at: OffsetDateTime,
    /// `pending` | `approved` | `rejected` | `cancelled`.
    pub status: &'static str,
    /// Kode alasan (hanya bila `rejected`).
    pub rejection_reason: Option<String>,
    /// Waktu keputusan (hanya bila `approved`/`rejected`).
    pub decided_at: Option<OffsetDateTime>,
    /// Setuju/tolak tersedia untuk pemanggil.
    pub can_decide: bool,
}

impl fmt::Debug for VerificationDetailView {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("VerificationDetailView")
            .field(
                "identity_verification_request_id",
                &self.identity_verification_request_id,
            )
            .field("nik", &"[REDACTED]")
            .field("status", &self.status)
            .finish_non_exhaustive()
    }
}

/// Hasil keputusan Verifikator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecisionView {
    /// Permintaan verifikasi.
    pub identity_verification_request_id: Uuid,
    /// `approved` | `rejected`.
    pub status: &'static str,
    /// Kode alasan (hanya penolakan).
    pub rejection_reason: Option<String>,
}

/// Satu alasan penolakan aktif (data migrasi, US-201 AC3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RejectionReasonView {
    /// Kode alasan.
    pub code: &'static str,
    /// Label tampilan.
    pub label: String,
}

/// US-201 AC1: antrean "Menunggu verifikasi" terlama lebih dulu (`cursor` = ID permintaan terakhir halaman
/// sebelumnya, `limit` 1-100).
pub async fn verification_queue(
    services: &IdentityServices,
    cursor: Option<Uuid>,
    limit: Option<i64>,
) -> Result<(Vec<QueueItemView>, envelope::Meta), ProfileError> {
    verification::queue(services, cursor, limit).await
}

/// US-201 AC1: detail pengajuan dengan NIK lengkap; E-05 dicatat lebih dulu.
pub async fn verification_detail(
    services: &IdentityServices,
    principal: &Principal,
    route: &str,
    request_id: Uuid,
) -> Result<VerificationDetailView, ProfileError> {
    verification::detail(services, principal, route, request_id).await
}

/// US-201 AC1, AC6: URL unduh foto dokumen (E-05 dulu, `FileAccess::Staff`).
pub async fn verification_document_photo_url(
    services: &IdentityServices,
    principal: &Principal,
    route: &str,
    request_id: Uuid,
) -> Result<DownloadTicket, ProfileError> {
    verification::document_photo_url(services, principal, route, request_id).await
}

/// US-201 AC2, AC5, AC7: menyetujui pengajuan.
pub async fn approve_identity(
    services: &IdentityServices,
    principal: &Principal,
    route: &str,
    request_id: Uuid,
) -> Result<DecisionView, ProfileError> {
    verification::approve(services, principal, route, request_id).await
}

/// US-201 AC3, AC3a, AC5: menolak pengajuan dengan kode alasan dan keterangan opsional.
pub async fn reject_identity(
    services: &IdentityServices,
    principal: &Principal,
    route: &str,
    request_id: Uuid,
    reason_code: &str,
    note: Option<&str>,
) -> Result<DecisionView, ProfileError> {
    verification::reject(services, principal, route, request_id, reason_code, note).await
}

/// US-201 AC3: daftar alasan penolakan aktif.
pub async fn identity_rejection_reasons(
    services: &IdentityServices,
) -> Result<Vec<RejectionReasonView>, ProfileError> {
    verification::rejection_reasons(services).await
}

// ----------------------------------------------------------------------------------------------------
// Profil dan alamat pasien (F1-T56, US-105). Tipe berisi data pribadi (nama, tanggal lahir, nomor HP,
// alamat) memakai `Debug` manual tanpa nilai (KEPUTUSAN-122, ADR-005).

/// Profil pemilik akun (US-105 AC1) beserta status identitas yang tampil (AB-05).
#[derive(Clone, PartialEq, Eq)]
pub struct ProfileView {
    /// Penerima layanan pemilik akun.
    pub care_recipient_id: Uuid,
    /// Nama lengkap.
    pub full_name: String,
    /// Tanggal lahir.
    pub birth_date: time::Date,
    /// `male` | `female`.
    pub sex: String,
    /// Nomor HP berbentuk `+62...`; kosong bila belum diisi.
    pub phone: Option<String>,
    /// `not_uploaded` | `pending` | `approved` | `rejected`.
    pub identity_status: &'static str,
}

impl fmt::Debug for ProfileView {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProfileView")
            .field("care_recipient_id", &self.care_recipient_id)
            .field("identity_status", &self.identity_status)
            .finish_non_exhaustive()
    }
}

/// Isian simpan profil (belum divalidasi).
#[derive(Clone, Copy)]
pub struct ProfileInput<'a> {
    /// Nama lengkap.
    pub full_name: &'a str,
    /// Tanggal lahir `YYYY-MM-DD`.
    pub birth_date: &'a str,
    /// `male` | `female`.
    pub sex: &'a str,
    /// Nomor HP (opsional; kosong = dikosongkan).
    pub phone: Option<&'a str>,
}

impl fmt::Debug for ProfileInput<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProfileInput").finish_non_exhaustive()
    }
}

/// Satu alamat aktif pasien.
#[derive(Clone, PartialEq, Eq)]
pub struct AddressView {
    /// ID alamat.
    pub id: Uuid,
    /// Label, misalnya "Rumah".
    pub label: Option<String>,
    /// Nama penerima.
    pub recipient_name: String,
    /// Nomor HP penerima (`+62...`).
    pub recipient_phone: String,
    /// Alamat lengkap.
    pub address_line: String,
    /// Provinsi.
    pub province_name: String,
    /// Kota.
    pub city_name: String,
    /// Kode pos.
    pub postal_code: Option<String>,
    /// Patokan alamat.
    pub notes: Option<String>,
    /// Nama zona waktu IANA (ADR-017 butir 2).
    pub time_zone: String,
    /// Alamat utama.
    pub is_default: bool,
}

impl fmt::Debug for AddressView {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AddressView")
            .field("id", &self.id)
            .field("is_default", &self.is_default)
            .finish_non_exhaustive()
    }
}

/// Isian tambah atau ubah alamat (belum divalidasi).
#[derive(Clone, Copy)]
pub struct AddressInput<'a> {
    /// Label (opsional).
    pub label: Option<&'a str>,
    /// Nama penerima.
    pub recipient_name: &'a str,
    /// Nomor HP penerima.
    pub recipient_phone: &'a str,
    /// Alamat lengkap.
    pub address_line: &'a str,
    /// Provinsi.
    pub province_name: &'a str,
    /// Kota.
    pub city_name: &'a str,
    /// Kode pos (opsional).
    pub postal_code: Option<&'a str>,
    /// Patokan (opsional).
    pub notes: Option<&'a str>,
    /// Zona waktu IANA (opsional; bawaan `Asia/Jakarta`).
    pub time_zone: Option<&'a str>,
    /// Jadikan alamat utama.
    pub is_default: Option<bool>,
}

impl fmt::Debug for AddressInput<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AddressInput").finish_non_exhaustive()
    }
}

/// US-105 AC1: profil akun pemanggil. Akun tanpa profil = `NotFound`.
pub async fn get_profile(
    services: &IdentityServices,
    principal: &Principal,
) -> Result<ProfileView, ProfileError> {
    profile_service::get_profile(services, principal).await
}

/// US-105 AC1, AC1a, AC1b, AC3: simpan profil akun pemanggil. Perubahan nama atau tanggal lahir pada
/// identitas "Disetujui" mengembalikannya ke "Menunggu verifikasi".
pub async fn update_profile(
    services: &IdentityServices,
    principal: &Principal,
    route: &str,
    input: &ProfileInput<'_>,
) -> Result<ProfileView, ProfileError> {
    profile_service::update_profile(services, principal, route, input).await
}

/// US-105 AC2: daftar alamat aktif akun pemanggil.
pub async fn list_addresses(
    services: &IdentityServices,
    principal: &Principal,
) -> Result<Vec<AddressView>, ProfileError> {
    profile_service::list_addresses(services, principal).await
}

/// US-105 AC2: tambah alamat; mengembalikan daftar alamat sesudahnya.
pub async fn create_address(
    services: &IdentityServices,
    principal: &Principal,
    route: &str,
    input: &AddressInput<'_>,
) -> Result<Vec<AddressView>, ProfileError> {
    profile_service::create_address(services, principal, route, input).await
}

/// US-105 AC2: ubah alamat milik pemanggil; mengembalikan daftar alamat sesudahnya.
pub async fn update_address(
    services: &IdentityServices,
    principal: &Principal,
    route: &str,
    address_id: Uuid,
    input: &AddressInput<'_>,
) -> Result<Vec<AddressView>, ProfileError> {
    profile_service::update_address(services, principal, route, address_id, input).await
}

/// US-105 AC2, KU-04: hapus (soft delete) alamat milik pemanggil; mengembalikan daftar sesudahnya.
pub async fn delete_address(
    services: &IdentityServices,
    principal: &Principal,
    route: &str,
    address_id: Uuid,
) -> Result<Vec<AddressView>, ProfileError> {
    profile_service::delete_address(services, principal, route, address_id).await
}
