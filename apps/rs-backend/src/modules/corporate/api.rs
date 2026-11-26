//! Antarmuka publik modul `corporate` untuk perakitan aplikasi dan modul lain.
//!
//! F1-T100: [`CorporateDirectory`] dipasang ke `authz::Enforcement::with_corporate_scopes` sebagai pembaca
//! keanggotaan aktif HR (ADR-028 butir 1). Dibaca langsung dari PostgreSQL di setiap permintaan endpoint HR,
//! tanpa cache, supaya pencabutan keanggotaan dan penonaktifan perusahaan berlaku pada permintaan berikutnya
//! (US-1904 AC5, US-1916 AC2, KU-10, M-51).
//!
//! F1-T104: verifikasi perusahaan oleh Verifikator (US-1902): antrean, detail, URL dokumen, setujui/tolak,
//! dan daftar alasan penolakan, dengan dependensi [`CorporateVerification`].
//! F1-T107: operasi keanggotaan HR untuk use case kelola anggota HR di modul `identity` (pemilik akun):
//! kunci tabel, daftar keanggotaan aktif per perusahaan, pemilik keanggotaan, dan pencabutan (hapus lunak).

use super::{admin_create, documents, registration, repo, verification};
use crate::modules::platform::api::{DownloadTicket, FileStorage, UploadTicket};
use authz::{
    CorporateClientStatus, CorporateMembership, CorporateScope, CorporateScopeResolver,
    MembershipFuture, Principal, ResolveError,
};
use foundation::{Clock, Sensitive, Uuid};
use sqlx::{PgConnection, PgPool};
use std::fmt;
use time::OffsetDateTime;

pub use super::admin_create::{CreatedCorporateClientView, NewCorporateClientInput};
pub use super::error::CorporateError;
pub use super::registration::{NewCorporateRegistration, RegistrationDeps};

/// Pembaca keanggotaan aktif HR di atas pool role `rs_app`.
#[derive(Clone)]
pub struct CorporateDirectory {
    pool: PgPool,
}

impl CorporateDirectory {
    /// `pool` = login `rs_api` (anggota `rs_app`, hak SELECT pada `corporate_member` dan `corporate_client`).
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Keanggotaan aktif akun `account_id`. Status perusahaan yang tidak dikenal = galat (ditolak `503`),
    /// tidak pernah dianggap Aktif.
    pub async fn membership(
        &self,
        account_id: Uuid,
    ) -> Result<Option<CorporateMembership>, ResolveError> {
        let row = repo::active_membership(&self.pool, account_id)
            .await
            .map_err(|_| {
                tracing::warn!("corporate_membership_read_failed");
                ResolveError
            })?;
        let Some(row) = row else {
            return Ok(None);
        };
        let Some(client_status) = CorporateClientStatus::parse(&row.client_status) else {
            tracing::error!("corporate_client_status_unparseable");
            return Err(ResolveError);
        };
        Ok(Some(CorporateMembership {
            corporate_member_id: row.corporate_member_id,
            corporate_client_id: row.corporate_client_id,
            client_status,
        }))
    }
}

/// Keanggotaan aktif satu HR di perusahaan (F1-T107).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemberRef {
    /// `corporate_member.id`.
    pub corporate_member_id: Uuid,
    /// `account.id` akun HR.
    pub account_id: Uuid,
    /// Jabatan (boleh kosong untuk HR yang diundang).
    pub job_title: Option<String>,
}

/// Mengunci `corporate_member` sampai akhir transaksi pemanggil (KEPUTUSAN-290 diterapkan ke invarian "HR
/// aktif terakhir per perusahaan", ADR-027/ADR-028). **Setiap** transaksi yang mencabut keanggotaan atau
/// menonaktifkan akun HR wajib memanggil ini lebih dulu, lalu membaca ulang [`active_members`] di transaksi
/// yang sama. Pembaca keanggotaan (`ACCESS SHARE`) tidak terhalang.
pub async fn lock_members(conn: &mut PgConnection) -> Result<(), sqlx::Error> {
    repo::lock_members(conn).await
}

/// Keanggotaan aktif perusahaan `corporate_client_id`.
pub async fn active_members(
    conn: &mut PgConnection,
    corporate_client_id: Uuid,
) -> Result<Vec<MemberRef>, sqlx::Error> {
    Ok(repo::active_members(conn, corporate_client_id)
        .await?
        .into_iter()
        .map(|r| MemberRef {
            corporate_member_id: r.id,
            account_id: r.account_id,
            job_title: r.job_title,
        })
        .collect())
}

/// Perusahaan pemilik keanggotaan aktif `corporate_member_id`, untuk keputusan objek `authz::corporate_object`
/// (perusahaan lain dan tidak ada = `404` yang sama, KU-03).
pub async fn member_owner(
    pool: &PgPool,
    corporate_member_id: Uuid,
) -> Result<Option<Uuid>, sqlx::Error> {
    repo::member_owner(pool, corporate_member_id).await
}

/// Mencabut keanggotaan aktif (hapus lunak dengan alasan) di transaksi pemanggil setelah [`lock_members`].
/// `false` bila tidak ada keanggotaan aktif `corporate_member_id` milik `corporate_client_id`.
pub async fn revoke_member(
    conn: &mut PgConnection,
    corporate_member_id: Uuid,
    corporate_client_id: Uuid,
    by: Uuid,
    reason: &str,
    now: OffsetDateTime,
) -> Result<bool, sqlx::Error> {
    repo::revoke_member(
        conn,
        corporate_member_id,
        corporate_client_id,
        by,
        reason,
        now,
    )
    .await
}

/// Menambah keanggotaan HR di transaksi pemanggil setelah [`lock_members`] (undangan HR, F1-T107): satu
/// keanggotaan aktif per akun. Mengembalikan `corporate_member.id`.
pub async fn add_member(
    conn: &mut PgConnection,
    corporate_client_id: Uuid,
    account_id: Uuid,
    by: Uuid,
    now: OffsetDateTime,
) -> Result<Uuid, sqlx::Error> {
    let id = foundation::new_id();
    repo::insert_member(conn, id, corporate_client_id, account_id, by, now).await?;
    Ok(id)
}

impl CorporateScopeResolver for CorporateDirectory {
    fn active_membership<'a>(&'a self, account_id: Uuid) -> MembershipFuture<'a> {
        Box::pin(self.membership(account_id))
    }
}

// ----------------------------------------------------------------------------------------------------
// Verifikasi perusahaan oleh Verifikator (F1-T104, US-1902). Dipanggil handler backoffice modul ini;
// kebijakan rute = sub-peran Verifikator.

/// Dependensi use case verifikasi perusahaan (dipinjam dari keadaan aplikasi per permintaan).
#[derive(Clone, Copy)]
pub struct CorporateVerification<'a> {
    /// Pool role aplikasi (`rs_app`).
    pub pool: &'a PgPool,
    /// Jam server (KEPUTUSAN-131).
    pub clock: &'a dyn Clock,
    /// Layanan berkas (URL unduh dokumen).
    pub files: &'a FileStorage,
}

impl fmt::Debug for CorporateVerification<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CorporateVerification")
            .finish_non_exhaustive()
    }
}

/// Satu pengajuan di antrean Verifikator (US-1902 AC1). Tanpa NPWP (hanya di detail).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CorporateQueueItemView {
    /// Perusahaan.
    pub corporate_client_id: Uuid,
    /// Nama badan usaha.
    pub legal_name: String,
    /// NIB.
    pub nib: String,
    /// `self_registered` | `created_by_admin`.
    pub origin: &'static str,
    /// Putaran pengajuan (naik saat diajukan ulang setelah ditolak).
    pub submission_round: i32,
    /// Waktu pengajuan terakhir.
    pub submitted_at: OffsetDateTime,
    /// NIB, NPWP, dan bukti kewenangan sudah diunggah semua (pengajuan "siap diverifikasi", F1-T102).
    pub documents_complete: bool,
}

/// Pilihan isi antrean verifikasi menurut kelengkapan dokumen (F1-T102, keputusan user Q8).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum QueueReadiness {
    /// Hanya pengajuan yang ketiga jenis dokumennya lengkap ("siap diverifikasi"); bawaan.
    #[default]
    Ready,
    /// Hanya pengajuan yang dokumennya belum lengkap.
    Incomplete,
    /// Semua pengajuan menunggu verifikasi.
    All,
}

/// Detail perusahaan untuk Verifikator (US-1902 AC1). NPWP Terbatas (DD A20): dibungkus [`Sensitive`],
/// `Debug` manual tanpa NPWP.
#[derive(Clone)]
pub struct CorporateDetailView {
    /// Perusahaan.
    pub corporate_client_id: Uuid,
    /// Nama badan usaha.
    pub legal_name: String,
    /// NIB.
    pub nib: String,
    /// NPWP (Terbatas: tidak pernah ke log, audit, atau pesan galat).
    pub npwp: Sensitive<String>,
    /// Alamat kantor.
    pub office_address: String,
    /// Telepon kantor.
    pub office_phone: String,
    /// `pending_verification` | `active` | `rejected` | `inactive`.
    pub status: &'static str,
    /// `self_registered` | `created_by_admin`.
    pub origin: &'static str,
    /// Putaran pengajuan berjalan.
    pub submission_round: i32,
    /// Waktu pengajuan terakhir.
    pub submitted_at: OffsetDateTime,
    /// Jenis dokumen aktif (`authority_proof`, `nib`, `npwp`); lengkap bila ketiganya ada (persetujuan ditolak
    /// `4090113` bila belum, F1-T102).
    pub documents: Vec<&'static str>,
    /// Token versi dokumen aktif; wajib dikirim kembali saat menyetujui atau menolak (`4090114` bila dokumen
    /// berubah sejak detail dilihat).
    pub documents_version: String,
    /// Kode alasan penolakan putaran berjalan (hanya bila `rejected`).
    pub rejection_reason: Option<String>,
    /// Waktu keputusan putaran berjalan, bila sudah diputuskan.
    pub decided_at: Option<OffsetDateTime>,
    /// Setuju/tolak tersedia (masih menunggu verifikasi).
    pub can_decide: bool,
}

impl fmt::Debug for CorporateDetailView {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CorporateDetailView")
            .field("corporate_client_id", &self.corporate_client_id)
            .field("npwp", &self.npwp)
            .field("status", &self.status)
            .field("submission_round", &self.submission_round)
            .finish_non_exhaustive()
    }
}

/// Hasil keputusan Verifikator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CorporateDecisionView {
    /// Perusahaan.
    pub corporate_client_id: Uuid,
    /// `active` | `rejected`.
    pub status: &'static str,
    /// Putaran yang diputuskan.
    pub submission_round: i32,
    /// Kode alasan (hanya penolakan).
    pub rejection_reason: Option<&'static str>,
}

/// Satu alasan penolakan aktif (data migrasi, US-1902 AC3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RejectionReasonView {
    /// Kode alasan.
    pub code: &'static str,
    /// Label tampilan.
    pub label: String,
}

/// US-1902 AC1: antrean "Menunggu verifikasi" terlama lebih dulu (`cursor` = ID perusahaan terakhir
/// halaman sebelumnya, `limit` 1-100).
pub async fn corporate_verification_queue(
    deps: &CorporateVerification<'_>,
    cursor: Option<Uuid>,
    limit: Option<i64>,
    readiness: QueueReadiness,
) -> Result<(Vec<CorporateQueueItemView>, envelope::Meta), CorporateError> {
    verification::queue(deps, cursor, limit, readiness).await
}

/// US-1902 AC1: detail perusahaan termasuk NPWP dan jenis dokumen aktif.
pub async fn corporate_verification_detail(
    deps: &CorporateVerification<'_>,
    corporate_client_id: Uuid,
) -> Result<CorporateDetailView, CorporateError> {
    verification::detail(deps, corporate_client_id).await
}

/// US-1902 AC1: URL unduh dokumen perusahaan berumur pendek (E-05 dulu, `FileAccess::StaffCorporate`).
pub async fn corporate_document_url(
    deps: &CorporateVerification<'_>,
    principal: &Principal,
    route: &str,
    corporate_client_id: Uuid,
    document_kind: &str,
) -> Result<DownloadTicket, CorporateError> {
    verification::document_url(deps, principal, route, corporate_client_id, document_kind).await
}

/// US-1902 AC2, AC4: menyetujui pengajuan.
pub async fn approve_corporate_client(
    deps: &CorporateVerification<'_>,
    principal: &Principal,
    route: &str,
    corporate_client_id: Uuid,
    documents_version: &str,
) -> Result<CorporateDecisionView, CorporateError> {
    verification::approve(
        deps,
        principal,
        route,
        corporate_client_id,
        documents_version,
    )
    .await
}

/// US-1902 AC3, AC4: menolak pengajuan dengan kode alasan.
pub async fn reject_corporate_client(
    deps: &CorporateVerification<'_>,
    principal: &Principal,
    route: &str,
    corporate_client_id: Uuid,
    reason_code: &str,
    documents_version: &str,
) -> Result<CorporateDecisionView, CorporateError> {
    verification::reject(
        deps,
        principal,
        route,
        corporate_client_id,
        reason_code,
        documents_version,
    )
    .await
}

/// US-1902 AC3: daftar alasan penolakan aktif.
pub async fn corporate_rejection_reasons(
    deps: &CorporateVerification<'_>,
) -> Result<Vec<RejectionReasonView>, CorporateError> {
    verification::rejection_reasons(deps).await
}

// ----------------------------------------------------------------------------------------------------
// Dokumen perusahaan oleh HR (F1-T102, US-1901 AC2). Dipanggil handler `/v1/corporate` modul ini; kebijakan
// rute = HR perusahaan Menunggu verifikasi, Ditolak, atau Aktif (status dibatasi lagi oleh use case:
// hanya Menunggu verifikasi dan Ditolak). Perusahaan selalu dari cakupan sesi (`CorporateScope`).

/// Dependensi use case dokumen perusahaan (dipinjam dari keadaan aplikasi per permintaan).
#[derive(Clone, Copy)]
pub struct CorporateDocuments<'a> {
    /// Pool role aplikasi (`rs_app`).
    pub pool: &'a PgPool,
    /// Jam server (KEPUTUSAN-131).
    pub clock: &'a dyn Clock,
    /// Layanan berkas (slot unggah, validasi isi, URL unduh).
    pub files: &'a FileStorage,
}

impl fmt::Debug for CorporateDocuments<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CorporateDocuments").finish_non_exhaustive()
    }
}

/// Status satu jenis dokumen wajib.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentStatusView {
    /// `nib` | `npwp` | `authority_proof`.
    pub kind: &'static str,
    /// Dokumen aktif jenis ini sudah diunggah.
    pub uploaded: bool,
    /// Waktu dokumen aktif dicatat.
    pub uploaded_at: Option<OffsetDateTime>,
}

/// Status verifikasi dan kelengkapan dokumen perusahaan pemanggil (US-1901 AC6, Q8).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentsOverviewView {
    /// `pending_verification` | `rejected`.
    pub status: &'static str,
    /// Putaran pengajuan berjalan.
    pub submission_round: i32,
    /// Kode alasan penolakan putaran berjalan (hanya bila `rejected`).
    pub rejection_reason: Option<String>,
    /// Ketiga jenis dokumen wajib, urut `nib`, `npwp`, `authority_proof`.
    pub documents: Vec<DocumentStatusView>,
    /// Ketiganya sudah diunggah (pengajuan "siap diverifikasi").
    pub complete: bool,
}

/// US-1901 AC2, AC6: status verifikasi, alasan penolakan, dan kelengkapan dokumen perusahaan pemanggil.
/// Perusahaan Aktif: [`CorporateError::DocumentsNotManageable`].
pub async fn corporate_documents_overview(
    deps: &CorporateDocuments<'_>,
    scope: &CorporateScope,
) -> Result<DocumentsOverviewView, CorporateError> {
    documents::overview(deps, scope).await
}

/// US-1901 AC2, langkah 1: slot unggah untuk akun HR pemanggil (JPG/PNG ≤ 5 MiB atau PDF ≤ 10 MiB; kuota
/// per akun per jam).
pub async fn create_corporate_document_slot(
    deps: &CorporateDocuments<'_>,
    scope: &CorporateScope,
    route: &str,
    content_type: String,
    size_bytes: u64,
) -> Result<UploadTicket, CorporateError> {
    documents::create_slot(deps, scope, route, content_type, size_bytes).await
}

/// US-1901 AC2, langkah 3: memvalidasi berkas yang sudah diunggah dan mencatatnya sebagai dokumen aktif
/// `document_kind` (menggantikan yang lama), dengan E-10.
pub async fn confirm_corporate_document(
    deps: &CorporateDocuments<'_>,
    scope: &CorporateScope,
    route: &str,
    document_kind: &str,
    upload_slot_id: Uuid,
) -> Result<DocumentsOverviewView, CorporateError> {
    documents::confirm(deps, scope, route, document_kind, upload_slot_id).await
}

/// URL unduh berumur pendek untuk dokumen aktif `document_kind` perusahaan pemanggil (E-12 dulu).
pub async fn corporate_document_download_url(
    deps: &CorporateDocuments<'_>,
    scope: &CorporateScope,
    route: &str,
    document_kind: &str,
) -> Result<DownloadTicket, CorporateError> {
    documents::download_url(deps, scope, route, document_kind).await
}

/// US-1903 AC1-AC4 (F1-T105): Verifikator membuat perusahaan langsung Aktif dan mengundang HR pertama
/// (akun `pending_provisioning` + niat penyediaan; email aktivasi dikirim worker IdP, ADR-030). NIB atau
/// email HR ganda = `409` tanpa efek, tercatat E-10 hasil gagal.
pub async fn create_corporate_client_by_admin(
    deps: &CorporateVerification<'_>,
    principal: &Principal,
    route: &str,
    input: &NewCorporateClientInput,
) -> Result<CreatedCorporateClientView, CorporateError> {
    admin_create::create(deps, principal, route, input).await
}

/// US-1901 AC1, AC3-AC7 (F1-T101): mendaftarkan perusahaan mandiri tanpa login. `Ok(())` juga untuk NIB atau
/// email yang sudah terdaftar (jawaban seragam, KEPUTUSAN-324; tanpa efek data, hanya audit gagal).
pub async fn register_corporate_client(
    deps: &RegistrationDeps<'_>,
    input: &NewCorporateRegistration,
) -> Result<(), CorporateError> {
    registration::register(deps, input).await
}

/// Teks persetujuan HR yang berlaku untuk layar pendaftaran (US-1901 AC1, US-104 AC1), tanpa login. Teks
/// draf ditolak sebagai tidak tersedia bila `drafts` = `Reject` (production, KEPUTUSAN-289).
pub async fn corporate_registration_consent_text(
    pool: &PgPool,
    clock: &dyn Clock,
    drafts: crate::modules::privacy::api::DraftPolicy,
) -> Result<crate::modules::privacy::api::ActiveConsentText, CorporateError> {
    use crate::modules::privacy::api::{ConsentType, PrivacyError, active_consent_text};
    match active_consent_text(pool, clock, ConsentType::CorporateHrRegistration, drafts).await {
        Ok(Some(t)) => Ok(t),
        Ok(None) => {
            tracing::error!("corporate_consent_text_missing");
            Err(CorporateError::Database)
        }
        Err(PrivacyError::Internal(w)) => Err(CorporateError::Internal(w)),
        Err(_) => Err(CorporateError::Database),
    }
}
