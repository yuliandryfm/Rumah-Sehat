//! DTO endpoint CMS backoffice. Field JSON snake_case bahasa Inggris (coding-standards §2.5); waktu ISO 8601
//! dengan offset (ADR-017 butir 1); permintaan menolak field yang tidak dikenal (`deny_unknown_fields`).

use super::domain::{
    Channel, ContentFields, ContentType, DisplayStatus, EditorialStatus, LinkTargetType, SitePage,
};
use super::markdown::PUBLIC_IMAGE_PATH;
use super::service::{ContentView, PublicContent};
use crate::openapi::MetaSchema;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;
use utoipa::{IntoParams, ToSchema};

/// Isian simpan draf (buat atau ubah).
#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct ContentRequest {
    /// Jenis konten.
    pub(super) content_type: ContentType,
    /// Judul (1-200 karakter).
    #[schema(example = "Tentang Ruang Sehat", max_length = 200)]
    pub(super) title: String,
    /// Isi Markdown tanpa HTML mentah (US-1802 AC6).
    #[serde(default)]
    #[schema(max_length = 100000)]
    pub(super) body: Option<String>,
    /// Kategori (opsional).
    #[serde(default)]
    #[schema(max_length = 100)]
    pub(super) category: Option<String>,
    /// Kanal tayang: aplikasi, situs, atau keduanya (US-1802 AC1).
    pub(super) channel: Channel,
    /// Halaman situs tujuan; wajib bila kanal memuat situs.
    #[serde(default)]
    pub(super) site_page: Option<SitePage>,
    /// Teks alternatif gambar; wajib sebelum konten bergambar ditayangkan (US-1802 AC5).
    #[serde(default)]
    #[schema(max_length = 300)]
    pub(super) image_alt_text: Option<String>,
    /// Tujuan tautan banner.
    #[serde(default)]
    pub(super) link_target_type: Option<LinkTargetType>,
    /// Rujukan tujuan tautan.
    #[serde(default)]
    #[schema(max_length = 500)]
    pub(super) link_target_ref: Option<String>,
}

impl From<ContentRequest> for ContentFields {
    fn from(r: ContentRequest) -> Self {
        Self {
            content_type: r.content_type,
            title: r.title,
            body: r.body,
            category: r.category,
            channel: r.channel,
            site_page: r.site_page,
            image_alt_text: r.image_alt_text,
            link_target_type: r.link_target_type,
            link_target_ref: r.link_target_ref,
        }
    }
}

/// Jadwal tayang (US-1101 AC2). Tanpa `publish_start_at` = tayang sekarang menurut jam server.
#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct ScheduleRequest {
    /// Waktu mulai tayang, ISO 8601 dengan offset; tidak boleh di masa lalu.
    #[serde(default)]
    #[schema(format = DateTime, example = "2027-03-01T08:00:00+07:00")]
    pub(super) publish_start_at: Option<String>,
    /// Waktu selesai tayang (opsional), ISO 8601 dengan offset; harus setelah waktu mulai.
    #[serde(default)]
    #[schema(format = DateTime)]
    pub(super) publish_end_at: Option<String>,
}

/// Soft delete dengan alasan wajib (ADR-003).
#[derive(Debug, Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct DeleteRequest {
    /// Alasan penghapusan (1-500 karakter).
    #[schema(example = "Konten usang", max_length = 500)]
    pub(super) delete_reason: String,
}

/// Saringan daftar konten.
#[derive(Debug, Deserialize, IntoParams)]
#[serde(deny_unknown_fields)]
#[into_params(parameter_in = Query)]
pub(super) struct ListParams {
    /// Status tampilan; `deleted` menampilkan konten terhapus yang dapat dipulihkan.
    pub(super) status: Option<DisplayStatus>,
    /// Kanal.
    pub(super) channel: Option<Channel>,
    /// Halaman situs.
    pub(super) site_page: Option<SitePage>,
    /// Cursor dari `meta.next_cursor` halaman sebelumnya (UUID).
    #[param(
        pattern = "^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$"
    )]
    pub(super) cursor: Option<String>,
    /// Jumlah item (1-100, bawaan 20).
    #[param(minimum = 1, maximum = 100)]
    pub(super) limit: Option<i64>,
}

/// Konten di backoffice (termasuk isi untuk pratinjau, US-1101 AC1).
#[derive(Debug, Serialize, ToSchema)]
pub(super) struct ContentDto {
    #[schema(example = "0193a1b2-c3d4-7e5f-8a6b-7c8d9e0f1a2b")]
    pub(super) content_id: String,
    pub(super) content_type: ContentType,
    pub(super) title: String,
    pub(super) body: Option<String>,
    pub(super) category: Option<String>,
    pub(super) channel: Channel,
    pub(super) site_page: Option<SitePage>,
    /// ID berkas gambar (bucket konten publik, disajikan lewat endpoint gambar publik bila tayang).
    #[schema(example = "0193a1b2-c3d4-7e5f-8a6b-7c8d9e0f1a2b")]
    pub(super) image_file_id: Option<String>,
    pub(super) image_alt_text: Option<String>,
    pub(super) link_target_type: Option<LinkTargetType>,
    pub(super) link_target_ref: Option<String>,
    #[schema(format = DateTime)]
    pub(super) publish_start_at: Option<String>,
    #[schema(format = DateTime)]
    pub(super) publish_end_at: Option<String>,
    /// Status tersimpan.
    pub(super) editorial_status: EditorialStatus,
    /// Status tampilan menurut jam server (Draf, Tayang (terjadwal), Tayang, Selesai, Diarsipkan, Terhapus).
    pub(super) display_status: DisplayStatus,
    #[schema(format = DateTime)]
    pub(super) created_at: String,
    #[schema(format = DateTime)]
    pub(super) updated_at: String,
    #[schema(format = DateTime)]
    pub(super) deleted_at: Option<String>,
    pub(super) delete_reason: Option<String>,
}

/// Format ISO 8601/RFC 3339 dengan offset (UTC). Gagal format = galat internal (tidak terjadi untuk
/// `timestamptz` yang sah).
pub(super) fn ts(t: OffsetDateTime) -> Result<String, time::error::Format> {
    t.format(&Rfc3339)
}

fn ts_opt(t: Option<OffsetDateTime>) -> Result<Option<String>, time::error::Format> {
    t.map(ts).transpose()
}

impl ContentDto {
    pub(super) fn from_view(v: ContentView) -> Result<Self, time::error::Format> {
        Ok(Self {
            content_id: v.id.hyphenated().to_string(),
            content_type: v.content_type,
            title: v.title,
            body: v.body,
            category: v.category,
            channel: v.channel,
            site_page: v.site_page,
            image_file_id: v.image_file_id.map(|i| i.hyphenated().to_string()),
            image_alt_text: v.image_alt_text,
            link_target_type: v.link_target_type,
            link_target_ref: v.link_target_ref,
            publish_start_at: ts_opt(v.publish_start_at)?,
            publish_end_at: ts_opt(v.publish_end_at)?,
            editorial_status: v.editorial_status,
            display_status: v.display_status,
            created_at: ts(v.created_at)?,
            updated_at: ts(v.updated_at)?,
            deleted_at: ts_opt(v.deleted_at)?,
            delete_reason: v.delete_reason,
        })
    }
}

/// Hasil soft delete.
#[derive(Debug, Serialize, ToSchema)]
pub(super) struct DeletedDto {
    #[schema(example = "0193a1b2-c3d4-7e5f-8a6b-7c8d9e0f1a2b")]
    pub(super) content_id: String,
    pub(super) display_status: DisplayStatus,
}

/// Bentuk dokumentasi respons sukses satu konten (ADR-023 butir 1); isi sebenarnya `envelope::Success`.
#[derive(ToSchema)]
#[allow(dead_code)] // hanya untuk pembangkitan skema
pub(super) struct ContentEnvelope {
    #[schema(example = "2000000")]
    response_code: String,
    #[schema(example = "Berhasil")]
    response_description: String,
    data: ContentDto,
    #[schema(nullable = false)]
    correlation_id: Option<String>,
}

/// Bentuk dokumentasi respons daftar konten.
#[derive(ToSchema)]
#[allow(dead_code)] // hanya untuk pembangkitan skema
pub(super) struct ContentListEnvelope {
    #[schema(example = "2000000")]
    response_code: String,
    #[schema(example = "Berhasil")]
    response_description: String,
    data: Vec<ContentDto>,
    meta: MetaSchema,
    #[schema(nullable = false)]
    correlation_id: Option<String>,
}

/// Bentuk dokumentasi respons soft delete.
#[derive(ToSchema)]
#[allow(dead_code)] // hanya untuk pembangkitan skema
pub(super) struct DeletedEnvelope {
    #[schema(example = "2000000")]
    response_code: String,
    #[schema(example = "Berhasil")]
    response_description: String,
    data: DeletedDto,
    #[schema(nullable = false)]
    correlation_id: Option<String>,
}

// ---------------------------------------------------------------- jalur publik (F1-T94)

/// Saringan daftar konten situs yang sedang tayang.
#[derive(Debug, Deserialize, IntoParams)]
#[serde(deny_unknown_fields)]
#[into_params(parameter_in = Query)]
pub(super) struct PublicListParams {
    /// Halaman situs.
    pub(super) site_page: Option<SitePage>,
    /// Jenis konten.
    pub(super) content_type: Option<ContentType>,
    /// Cursor dari `meta.next_cursor` halaman sebelumnya (UUID).
    #[param(
        pattern = "^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$"
    )]
    pub(super) cursor: Option<String>,
    /// Jumlah item (1-100, bawaan 20).
    #[param(minimum = 1, maximum = 100)]
    pub(super) limit: Option<i64>,
}

/// Gambar konten tayang; disajikan endpoint gambar publik backend (bukan bucket publik, ADR-027 butir 5).
#[derive(Debug, Serialize, ToSchema)]
pub(super) struct PublicImageDto {
    /// Jalur relatif endpoint gambar publik.
    #[schema(example = "/v1/public/content/images/0193a1b2-c3d4-7e5f-8a6b-7c8d9e0f1a2b")]
    pub(super) url: String,
    /// Teks alternatif (wajib untuk konten tayang, US-1802 AC5).
    pub(super) alt_text: String,
}

/// Konten situs yang sedang tayang (tanpa status editorial, pelaku, atau data pribadi; US-1801 AC5).
#[derive(Debug, Serialize, ToSchema)]
pub(super) struct PublicContentDto {
    #[schema(example = "0193a1b2-c3d4-7e5f-8a6b-7c8d9e0f1a2b")]
    pub(super) content_id: String,
    pub(super) content_type: ContentType,
    pub(super) title: String,
    /// Markdown yang lolos aturan render aman (F1-T111); dirender tanpa HTML mentah.
    pub(super) body: Option<String>,
    pub(super) category: Option<String>,
    pub(super) site_page: Option<SitePage>,
    pub(super) image: Option<PublicImageDto>,
    pub(super) link_target_type: Option<LinkTargetType>,
    pub(super) link_target_ref: Option<String>,
    /// Mulai tayang (urutan tampil).
    #[schema(format = DateTime)]
    pub(super) published_at: String,
}

impl PublicContentDto {
    pub(super) fn from_public(p: PublicContent) -> Result<Self, time::error::Format> {
        Ok(Self {
            content_id: p.id.hyphenated().to_string(),
            content_type: p.content_type,
            title: p.title,
            body: p.body,
            category: p.category,
            site_page: p.site_page,
            image: p.image.map(|(id, alt)| PublicImageDto {
                url: format!("{PUBLIC_IMAGE_PATH}{}", id.hyphenated()),
                alt_text: alt,
            }),
            link_target_type: p.link_target_type,
            link_target_ref: p.link_target_ref,
            published_at: ts(p.publish_start_at)?,
        })
    }
}

/// Bentuk dokumentasi respons satu konten publik.
#[derive(ToSchema)]
#[allow(dead_code)] // hanya untuk pembangkitan skema
pub(super) struct PublicContentEnvelope {
    #[schema(example = "2000000")]
    response_code: String,
    #[schema(example = "Berhasil")]
    response_description: String,
    data: PublicContentDto,
    #[schema(nullable = false)]
    correlation_id: Option<String>,
}

/// Bentuk dokumentasi respons daftar konten publik.
#[derive(ToSchema)]
#[allow(dead_code)] // hanya untuk pembangkitan skema
pub(super) struct PublicContentListEnvelope {
    #[schema(example = "2000000")]
    response_code: String,
    #[schema(example = "Berhasil")]
    response_description: String,
    data: Vec<PublicContentDto>,
    meta: MetaSchema,
    #[schema(nullable = false)]
    correlation_id: Option<String>,
}
