//! DTO endpoint Verifikator (`/backoffice/v1`, F1-T62, US-201). Field JSON snake_case bahasa Inggris
//! (coding-standards §2.5); waktu ISO 8601 dengan offset (ADR-017 butir 1); tanggal lahir `YYYY-MM-DD`;
//! permintaan menolak field yang tidak dikenal. NIK lengkap hanya di detail (PRD §3.1: hanya Verifikator).

use super::api::{DecisionView, QueueItemView, RejectionReasonView, VerificationDetailView};
use super::dto::DtoError;
use crate::modules::platform::api::DownloadTicket;
use crate::openapi::MetaSchema;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use utoipa::{IntoParams, ToSchema};

fn ts(t: OffsetDateTime) -> Result<String, DtoError> {
    t.format(&Rfc3339).map_err(|_| DtoError)
}

fn date(d: time::Date) -> Result<String, DtoError> {
    let year = d.year();
    if !(1..=9999).contains(&year) {
        return Err(DtoError);
    }
    Ok(format!(
        "{year:04}-{:02}-{:02}",
        u8::from(d.month()),
        d.day()
    ))
}

/// Halaman antrean.
#[derive(Debug, Deserialize, IntoParams)]
#[serde(deny_unknown_fields)]
#[into_params(parameter_in = Query)]
pub(super) struct QueueParams {
    /// Cursor dari `meta.next_cursor` halaman sebelumnya (UUID).
    #[param(
        pattern = "^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$"
    )]
    pub(super) cursor: Option<String>,
    /// Jumlah item (1-100, bawaan 20).
    #[param(minimum = 1, maximum = 100)]
    pub(super) limit: Option<i64>,
}

/// Satu pengajuan di antrean (US-201 AC1).
#[derive(Debug, Serialize, ToSchema)]
pub(super) struct QueueItemDto {
    #[schema(example = "0193a1b2-c3d4-7e5f-8a6b-7c8d9e0f1a2b")]
    pub(super) identity_verification_request_id: String,
    #[schema(example = "0193a1b2-c3d4-7e5f-8a6b-7c8d9e0f1a2c")]
    pub(super) care_recipient_id: String,
    /// Pemilik akun (bukan anggota keluarga).
    pub(super) is_account_owner: bool,
    #[schema(example = "Pasien Sintetis")]
    pub(super) full_name: String,
    #[schema(format = Date, example = "1990-01-01")]
    pub(super) birth_date: String,
    /// `male` | `female`.
    #[schema(example = "female")]
    pub(super) sex: String,
    /// `ktp` | `kk`.
    #[schema(example = "ktp")]
    pub(super) document_type: String,
    /// `document_submitted` | `profile_changed`.
    #[schema(example = "document_submitted")]
    pub(super) trigger_type: String,
    /// Empat digit terakhir NIK (NIK lengkap hanya di detail).
    #[schema(example = "0001")]
    pub(super) nik_last4: String,
    #[schema(format = DateTime)]
    pub(super) submitted_at: String,
}

impl QueueItemDto {
    pub(super) fn from_view(v: QueueItemView) -> Result<Self, DtoError> {
        Ok(Self {
            identity_verification_request_id: v
                .identity_verification_request_id
                .hyphenated()
                .to_string(),
            care_recipient_id: v.care_recipient_id.hyphenated().to_string(),
            is_account_owner: v.is_account_owner,
            full_name: v.full_name,
            birth_date: date(v.birth_date)?,
            sex: v.sex,
            document_type: v.document_type.to_string(),
            trigger_type: v.trigger_type.to_string(),
            nik_last4: v.nik_last4,
            submitted_at: ts(v.submitted_at)?,
        })
    }
}

/// Bentuk dokumentasi respons antrean.
#[derive(ToSchema)]
#[allow(dead_code)] // hanya untuk pembangkitan skema
pub(super) struct QueueEnvelope {
    #[schema(example = "2000000")]
    response_code: String,
    #[schema(example = "Berhasil")]
    response_description: String,
    data: Vec<QueueItemDto>,
    meta: MetaSchema,
    #[schema(nullable = false)]
    correlation_id: Option<String>,
}

/// Status permintaan verifikasi untuk Verifikator.
#[derive(Debug, Clone, Copy, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub(super) enum RequestStatusDto {
    /// Menunggu verifikasi.
    Pending,
    /// Disetujui.
    Approved,
    /// Ditolak.
    Rejected,
    /// Dibatalkan pasien.
    Cancelled,
}

impl RequestStatusDto {
    fn parse(code: &str) -> Result<Self, DtoError> {
        match code {
            "pending" => Ok(Self::Pending),
            "approved" => Ok(Self::Approved),
            "rejected" => Ok(Self::Rejected),
            "cancelled" => Ok(Self::Cancelled),
            _ => Err(DtoError),
        }
    }
}

/// Detail pengajuan dengan NIK lengkap (US-201 AC1; membuka detail tercatat E-05).
#[derive(Debug, Serialize, ToSchema)]
pub(super) struct VerificationDetailDto {
    #[schema(example = "0193a1b2-c3d4-7e5f-8a6b-7c8d9e0f1a2b")]
    pub(super) identity_verification_request_id: String,
    #[schema(example = "0193a1b2-c3d4-7e5f-8a6b-7c8d9e0f1a2c")]
    pub(super) care_recipient_id: String,
    pub(super) is_account_owner: bool,
    #[schema(example = "Pasien Sintetis")]
    pub(super) full_name: String,
    #[schema(format = Date, example = "1990-01-01")]
    pub(super) birth_date: String,
    #[schema(example = "female")]
    pub(super) sex: String,
    #[schema(example = "ktp")]
    pub(super) document_type: String,
    #[schema(example = "document_submitted")]
    pub(super) trigger_type: String,
    /// NIK lengkap 16 digit (hanya Verifikator, PRD §3.1). Jangan disimpan atau dicatat di log klien.
    #[schema(example = "9900000000000001")]
    pub(super) nik: String,
    #[schema(format = DateTime)]
    pub(super) submitted_at: String,
    pub(super) status: RequestStatusDto,
    /// Kode alasan penolakan (hanya bila `rejected`).
    #[schema(example = "photo_unreadable")]
    pub(super) rejection_reason: Option<String>,
    /// Waktu keputusan (hanya bila `approved` atau `rejected`).
    #[schema(format = DateTime)]
    pub(super) decided_at: Option<String>,
    /// Setuju/tolak tersedia (masih menunggu).
    pub(super) can_decide: bool,
}

impl VerificationDetailDto {
    pub(super) fn from_view(v: VerificationDetailView) -> Result<Self, DtoError> {
        Ok(Self {
            identity_verification_request_id: v
                .identity_verification_request_id
                .hyphenated()
                .to_string(),
            care_recipient_id: v.care_recipient_id.hyphenated().to_string(),
            is_account_owner: v.is_account_owner,
            full_name: v.full_name,
            birth_date: date(v.birth_date)?,
            sex: v.sex,
            document_type: v.document_type.to_string(),
            trigger_type: v.trigger_type.to_string(),
            nik: v.nik,
            submitted_at: ts(v.submitted_at)?,
            status: RequestStatusDto::parse(v.status)?,
            rejection_reason: v.rejection_reason,
            decided_at: v.decided_at.map(ts).transpose()?,
            can_decide: v.can_decide,
        })
    }
}

/// Bentuk dokumentasi respons detail.
#[derive(ToSchema)]
#[allow(dead_code)] // hanya untuk pembangkitan skema
pub(super) struct VerificationDetailEnvelope {
    #[schema(example = "2000000")]
    response_code: String,
    #[schema(example = "Berhasil")]
    response_description: String,
    data: VerificationDetailDto,
    #[schema(nullable = false)]
    correlation_id: Option<String>,
}

/// URL unduh foto dokumen berumur pendek (ADR-018 butir 3). URL adalah rahasia: jangan dicatat.
#[derive(Debug, Serialize, ToSchema)]
pub(super) struct DocumentPhotoDto {
    pub(super) url: String,
    #[schema(format = DateTime)]
    pub(super) expires_at: String,
    #[schema(example = "image/jpeg")]
    pub(super) mime_type: String,
    #[schema(example = 204_800)]
    pub(super) size_bytes: i32,
}

impl DocumentPhotoDto {
    pub(super) fn from_ticket(t: DownloadTicket) -> Result<Self, DtoError> {
        Ok(Self {
            url: t.url,
            expires_at: ts(t.expires_at)?,
            mime_type: t.mime_type,
            size_bytes: t.size_bytes,
        })
    }
}

/// Bentuk dokumentasi respons URL foto.
#[derive(ToSchema)]
#[allow(dead_code)] // hanya untuk pembangkitan skema
pub(super) struct DocumentPhotoEnvelope {
    #[schema(example = "2000000")]
    response_code: String,
    #[schema(example = "Berhasil")]
    response_description: String,
    data: DocumentPhotoDto,
    #[schema(nullable = false)]
    correlation_id: Option<String>,
}

/// Penolakan (US-201 AC3, AC3a).
#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct RejectRequest {
    /// Kode alasan dari `GET /backoffice/v1/identity-verification-reasons`.
    #[schema(example = "photo_unreadable")]
    pub(super) reason_code: String,
    /// Keterangan tambahan yang tampil ke pasien; wajib untuk `other`; paling panjang 500 karakter.
    #[schema(example = "Sudut kanan foto terpotong", max_length = 500)]
    pub(super) note: Option<String>,
}

/// Hasil keputusan.
#[derive(Debug, Serialize, ToSchema)]
pub(super) struct DecisionDto {
    #[schema(example = "0193a1b2-c3d4-7e5f-8a6b-7c8d9e0f1a2b")]
    pub(super) identity_verification_request_id: String,
    pub(super) status: RequestStatusDto,
    /// Kode alasan (hanya penolakan).
    #[schema(example = "photo_unreadable")]
    pub(super) rejection_reason: Option<String>,
}

impl DecisionDto {
    pub(super) fn from_view(v: DecisionView) -> Result<Self, DtoError> {
        Ok(Self {
            identity_verification_request_id: v
                .identity_verification_request_id
                .hyphenated()
                .to_string(),
            status: RequestStatusDto::parse(v.status)?,
            rejection_reason: v.rejection_reason,
        })
    }
}

/// Bentuk dokumentasi respons keputusan.
#[derive(ToSchema)]
#[allow(dead_code)] // hanya untuk pembangkitan skema
pub(super) struct DecisionEnvelope {
    #[schema(example = "2000000")]
    response_code: String,
    #[schema(example = "Berhasil")]
    response_description: String,
    data: DecisionDto,
    #[schema(nullable = false)]
    correlation_id: Option<String>,
}

/// Satu alasan penolakan (data migrasi, US-201 AC3).
#[derive(Debug, Serialize, ToSchema)]
pub(super) struct RejectionReasonDto {
    #[schema(example = "photo_unreadable")]
    pub(super) code: String,
    #[schema(example = "Foto dokumen buram atau tidak terbaca")]
    pub(super) label: String,
}

impl RejectionReasonDto {
    pub(super) fn from_view(v: RejectionReasonView) -> Self {
        Self {
            code: v.code.to_string(),
            label: v.label,
        }
    }
}

/// Bentuk dokumentasi respons daftar alasan.
#[derive(ToSchema)]
#[allow(dead_code)] // hanya untuk pembangkitan skema
pub(super) struct RejectionReasonsEnvelope {
    #[schema(example = "2000000")]
    response_code: String,
    #[schema(example = "Berhasil")]
    response_description: String,
    data: Vec<RejectionReasonDto>,
    #[schema(nullable = false)]
    correlation_id: Option<String>,
}
