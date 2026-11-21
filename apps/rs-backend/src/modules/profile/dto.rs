//! DTO endpoint identitas pasien (`/patient/v1`, F1-T61). Field JSON snake_case bahasa Inggris
//! (coding-standards §2.5); waktu ISO 8601 dengan offset (ADR-017 butir 1); permintaan menolak field yang
//! tidak dikenal (`deny_unknown_fields`). Nilai NIK tidak pernah ada di respons, hanya 4 digit terakhir.

use super::api::{IdentityStatusView, SubmissionAccepted};
use crate::modules::platform::api::{UploadInstruction, UploadTicket};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use utoipa::ToSchema;

fn ts(t: OffsetDateTime) -> Result<String, time::error::Format> {
    t.format(&Rfc3339)
}

/// Permintaan slot unggah foto KTP (US-106 AC3).
#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct KtpUploadSlotRequest {
    /// Jenis yang dinyatakan: `image/jpeg` atau `image/png` (isi tetap diperiksa saat pengajuan).
    #[schema(example = "image/jpeg")]
    pub(super) content_type: String,
    /// Ukuran berkas (byte), maksimal 5.242.880 (KEPUTUSAN-191).
    #[schema(example = 204800, minimum = 1, maximum = 5242880)]
    pub(super) size_bytes: u64,
}

/// Pasangan nama-nilai (field formulir POST atau header PUT).
#[derive(Debug, Serialize, ToSchema)]
pub(super) struct NameValueDto {
    pub(super) name: String,
    pub(super) value: String,
}

/// Cara unggah langsung ke object storage (rahasia berumur pendek; tidak boleh dicatat di log).
#[derive(Debug, Clone, Copy, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub(super) enum UploadMethodDto {
    /// `multipart/form-data` POST ke `url` dengan semua `fields`, berkas sebagai field `file` terakhir.
    Post,
    /// `PUT` ke `url` dengan `headers` persis.
    Put,
}

/// Slot unggah yang diterbitkan.
#[derive(Debug, Serialize, ToSchema)]
pub(super) struct UploadSlotDto {
    #[schema(example = "0193a1b2-c3d4-7e5f-8a6b-7c8d9e0f1a2b")]
    pub(super) upload_slot_id: String,
    #[schema(format = DateTime)]
    pub(super) expires_at: String,
    pub(super) method: UploadMethodDto,
    pub(super) url: String,
    /// Field formulir (POST); kosong untuk PUT.
    pub(super) fields: Vec<NameValueDto>,
    /// Header wajib (PUT); kosong untuk POST.
    pub(super) headers: Vec<NameValueDto>,
}

fn pairs(v: Vec<(String, String)>) -> Vec<NameValueDto> {
    v.into_iter()
        .map(|(name, value)| NameValueDto { name, value })
        .collect()
}

impl UploadSlotDto {
    pub(super) fn from_ticket(t: UploadTicket) -> Result<Self, time::error::Format> {
        let (method, url, fields, headers) = match t.instruction {
            UploadInstruction::Post { url, fields } => {
                (UploadMethodDto::Post, url, pairs(fields), Vec::new())
            }
            UploadInstruction::Put { url, headers } => {
                (UploadMethodDto::Put, url, Vec::new(), pairs(headers))
            }
        };
        Ok(Self {
            upload_slot_id: t.upload_slot_id.hyphenated().to_string(),
            expires_at: ts(t.expires_at)?,
            method,
            url,
            fields,
            headers,
        })
    }
}

/// Bentuk dokumentasi respons slot unggah.
#[derive(ToSchema)]
#[allow(dead_code)] // hanya untuk pembangkitan skema
pub(super) struct UploadSlotEnvelope {
    #[schema(example = "2010000")]
    response_code: String,
    #[schema(example = "Berhasil dibuat")]
    response_description: String,
    data: UploadSlotDto,
    #[schema(nullable = false)]
    correlation_id: Option<String>,
}

/// Pengajuan NIK dan foto KTP (US-106 AC1, AC2).
#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct IdentitySubmissionRequest {
    /// NIK tepat 16 digit angka (US-106 AC2). Tidak pernah dikembalikan di respons atau dicatat di log.
    #[schema(
        example = "9900000000000001",
        min_length = 16,
        max_length = 16,
        pattern = "^[0-9]{16}$"
    )]
    pub(super) nik: String,
    /// ID slot dari `POST /patient/v1/ktp-photos/upload-slots` yang berkasnya sudah diunggah.
    #[schema(example = "0193a1b2-c3d4-7e5f-8a6b-7c8d9e0f1a2b")]
    pub(super) upload_slot_id: String,
}

/// Status tampilan identitas (AB-05).
#[derive(Debug, Clone, Copy, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub(super) enum IdentityStatusDto {
    /// Belum diunggah.
    NotUploaded,
    /// Menunggu verifikasi.
    Pending,
    /// Disetujui.
    Approved,
    /// Ditolak.
    Rejected,
}

impl IdentityStatusDto {
    fn parse(code: &str) -> Option<Self> {
        match code {
            "not_uploaded" => Some(Self::NotUploaded),
            "pending" => Some(Self::Pending),
            "approved" => Some(Self::Approved),
            "rejected" => Some(Self::Rejected),
            _ => None,
        }
    }
}

/// Galat pemetaan DTO (selalu galat internal).
#[derive(Debug)]
pub(super) struct DtoError;

impl From<time::error::Format> for DtoError {
    fn from(_: time::error::Format) -> Self {
        Self
    }
}

/// Pengajuan yang diterima.
#[derive(Debug, Serialize, ToSchema)]
pub(super) struct SubmissionDto {
    #[schema(example = "0193a1b2-c3d4-7e5f-8a6b-7c8d9e0f1a2b")]
    pub(super) identity_verification_request_id: String,
    #[schema(example = "0193a1b2-c3d4-7e5f-8a6b-7c8d9e0f1a2c")]
    pub(super) identity_document_id: String,
    /// Selalu `pending` ("Menunggu verifikasi"; verifikasi dilakukan manual oleh admin, US-106 AC1).
    pub(super) status: IdentityStatusDto,
}

impl SubmissionDto {
    pub(super) fn from_accepted(a: SubmissionAccepted) -> Result<Self, DtoError> {
        Ok(Self {
            identity_verification_request_id: a
                .identity_verification_request_id
                .hyphenated()
                .to_string(),
            identity_document_id: a.identity_document_id.hyphenated().to_string(),
            status: IdentityStatusDto::parse(a.status).ok_or(DtoError)?,
        })
    }
}

/// Bentuk dokumentasi respons pengajuan.
#[derive(ToSchema)]
#[allow(dead_code)] // hanya untuk pembangkitan skema
pub(super) struct SubmissionEnvelope {
    #[schema(example = "2010000")]
    response_code: String,
    #[schema(example = "Berhasil dibuat")]
    response_description: String,
    data: SubmissionDto,
    #[schema(nullable = false)]
    correlation_id: Option<String>,
}

/// Status identitas penerima layanan (AB-05, US-106 AC1, AC5).
#[derive(Debug, Serialize, ToSchema)]
pub(super) struct IdentityDto {
    pub(super) status: IdentityStatusDto,
    /// Permintaan efektif; `null` bila "Belum diunggah".
    #[schema(example = "0193a1b2-c3d4-7e5f-8a6b-7c8d9e0f1a2b")]
    pub(super) identity_verification_request_id: Option<String>,
    /// Kode alasan penolakan (hanya bila `rejected`, US-201 AC3).
    #[schema(example = "photo_unreadable")]
    pub(super) rejection_reason: Option<String>,
    /// Empat digit terakhir NIK permintaan efektif.
    #[schema(example = "0001")]
    pub(super) nik_last4: Option<String>,
    /// Waktu pengajuan efektif.
    #[schema(format = DateTime)]
    pub(super) submitted_at: Option<String>,
    /// Pengajuan baru diizinkan (US-106 AC1, AC5).
    pub(super) can_submit: bool,
    /// Pengajuan yang menunggu dapat dibatalkan (US-106 AC5).
    pub(super) can_cancel: bool,
}

impl IdentityDto {
    pub(super) fn from_view(v: IdentityStatusView) -> Result<Self, DtoError> {
        Ok(Self {
            status: IdentityStatusDto::parse(v.status).ok_or(DtoError)?,
            identity_verification_request_id: v
                .identity_verification_request_id
                .map(|id| id.hyphenated().to_string()),
            rejection_reason: v.rejection_reason,
            nik_last4: v.nik_last4,
            submitted_at: v.submitted_at.map(ts).transpose()?,
            can_submit: v.can_submit,
            can_cancel: v.can_cancel,
        })
    }
}

/// Bentuk dokumentasi respons status identitas.
#[derive(ToSchema)]
#[allow(dead_code)] // hanya untuk pembangkitan skema
pub(super) struct IdentityEnvelope {
    #[schema(example = "2000000")]
    response_code: String,
    #[schema(example = "Berhasil")]
    response_description: String,
    data: IdentityDto,
    #[schema(nullable = false)]
    correlation_id: Option<String>,
}
