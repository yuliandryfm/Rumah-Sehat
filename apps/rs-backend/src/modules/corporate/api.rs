//! Antarmuka publik modul `corporate` untuk perakitan aplikasi dan modul lain.
//!
//! F1-T100: [`CorporateDirectory`] dipasang ke `authz::Enforcement::with_corporate_scopes` sebagai pembaca
//! keanggotaan aktif HR (ADR-028 butir 1). Dibaca langsung dari PostgreSQL di setiap permintaan endpoint HR,
//! tanpa cache, supaya pencabutan keanggotaan dan penonaktifan perusahaan berlaku pada permintaan berikutnya
//! (US-1904 AC5, US-1916 AC2, KU-10, M-51).
//!
//! F1-T104: verifikasi perusahaan oleh Verifikator (US-1902): antrean, detail, URL dokumen, setujui/tolak,
//! dan daftar alasan penolakan, dengan dependensi [`CorporateVerification`].

use super::{repo, verification};
use crate::modules::platform::api::{DownloadTicket, FileStorage};
use authz::{
    CorporateClientStatus, CorporateMembership, CorporateScopeResolver, MembershipFuture,
    Principal, ResolveError,
};
use foundation::{Clock, Sensitive, Uuid};
use sqlx::PgPool;
use std::fmt;
use time::OffsetDateTime;

pub use super::error::CorporateError;

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
    /// Jenis dokumen aktif (`authority_proof`, `nib`, `npwp`).
    pub documents: Vec<&'static str>,
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
) -> Result<(Vec<CorporateQueueItemView>, envelope::Meta), CorporateError> {
    verification::queue(deps, cursor, limit).await
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
) -> Result<CorporateDecisionView, CorporateError> {
    verification::approve(deps, principal, route, corporate_client_id).await
}

/// US-1902 AC3, AC4: menolak pengajuan dengan kode alasan.
pub async fn reject_corporate_client(
    deps: &CorporateVerification<'_>,
    principal: &Principal,
    route: &str,
    corporate_client_id: Uuid,
    reason_code: &str,
) -> Result<CorporateDecisionView, CorporateError> {
    verification::reject(deps, principal, route, corporate_client_id, reason_code).await
}

/// US-1902 AC3: daftar alasan penolakan aktif.
pub async fn corporate_rejection_reasons(
    deps: &CorporateVerification<'_>,
) -> Result<Vec<RejectionReasonView>, CorporateError> {
    verification::rejection_reasons(deps).await
}
