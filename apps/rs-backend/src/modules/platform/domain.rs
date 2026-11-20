//! Aturan murni berkas unggahan (ADR-018, NFR-SEC-08, DD A19.5/A19.6), tanpa I/O.
//!
//! - Jenis berkas ditentukan dari isi (magic bytes), bukan ekstensi, nama berkas, atau `Content-Type` klien
//!   (ADR-018 butir 2, OWASP File Upload [SRC-50]); isi lalu wajib berhasil didekode sebagai jenis itu.
//! - Foto didekode lalu dikode ulang ke jenis yang sama, sehingga metadata (EXIF termasuk lokasi, XMP,
//!   IPTC, ICC, chunk teks PNG) dan data apa pun di luar gambar (misalnya muatan polyglot setelah penanda
//!   akhir) terbuang. Orientasi EXIF diterapkan ke piksel lebih dulu supaya foto tidak terputar.
//! - Batas ukuran foto 5 MiB = 5.242.880 byte untuk berkas seperti diunggah, sebelum dikode ulang
//!   (NFR-SEC-08, KEPUTUSAN-191).

use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::PngEncoder;
use image::{DynamicImage, ImageDecoder, ImageFormat, ImageReader, Limits};
use std::io::Cursor;

/// Batas ukuran foto JPG/PNG seperti diunggah (NFR-SEC-08: 5 MiB, KEPUTUSAN-191).
pub const MAX_PHOTO_BYTES: u64 = 5_242_880;

/// Batas lebar/tinggi gambar yang didekode (pelindung bom dekompresi). Usulan teknis F1-T29: cukup untuk
/// foto kamera ponsel 108 MP (±12.000 × 9.000 piksel).
pub(crate) const MAX_IMAGE_DIMENSION: u32 = 12_000;

/// Batas alokasi dekoder (byte). Usulan teknis F1-T29.
pub(crate) const MAX_DECODE_ALLOC: u64 = 512 * 1024 * 1024;

/// Kualitas JPEG hasil kode ulang (1–100). Usulan teknis F1-T29.
pub(crate) const JPEG_REENCODE_QUALITY: u8 = 90;

/// Jenis foto yang diizinkan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhotoFormat {
    /// JPEG.
    Jpeg,
    /// PNG.
    Png,
}

impl PhotoFormat {
    /// Nilai `mime_type` (`stored_file`, CHECK DD A19.6) dan `Content-Type` URL unggah.
    pub const fn mime(self) -> &'static str {
        match self {
            Self::Jpeg => "image/jpeg",
            Self::Png => "image/png",
        }
    }

    /// Mengurai tipe MIME yang dinyatakan klien saat meminta slot.
    pub fn from_mime(mime: &str) -> Option<Self> {
        match mime {
            "image/jpeg" => Some(Self::Jpeg),
            "image/png" => Some(Self::Png),
            _ => None,
        }
    }

    fn image_format(self) -> ImageFormat {
        match self {
            Self::Jpeg => ImageFormat::Jpeg,
            Self::Png => ImageFormat::Png,
        }
    }
}

/// Bucket logis (CHECK `ck_upload_slot_bucket` / `ck_stored_file_bucket`, DD A19.6). Nama bucket fisik per
/// lingkungan dibentuk dari konfigurasi.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicalBucket {
    /// Dokumen identitas (foto KTP/KK).
    IdentityDocument,
    /// Berkas klinis.
    Clinical,
    /// Bukti klaim.
    ClaimEvidence,
    /// Dokumen izin.
    LicenseDocument,
    /// Export dan salinan data.
    Export,
    /// Konten CMS.
    PublicContent,
    /// Foto katalog rekanan.
    PublicCatalog,
}

impl LogicalBucket {
    /// Semua nilai, urut sesuai CHECK di migrasi.
    pub const ALL: [Self; 7] = [
        Self::IdentityDocument,
        Self::Clinical,
        Self::ClaimEvidence,
        Self::LicenseDocument,
        Self::Export,
        Self::PublicContent,
        Self::PublicCatalog,
    ];

    /// Nilai kolom `bucket`.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::IdentityDocument => "identity_document",
            Self::Clinical => "clinical",
            Self::ClaimEvidence => "claim_evidence",
            Self::LicenseDocument => "license_document",
            Self::Export => "export",
            Self::PublicContent => "public_content",
            Self::PublicCatalog => "public_catalog",
        }
    }

    /// Mengurai nilai kolom `bucket`.
    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|b| b.as_str() == value)
    }
}

/// Tujuan unggahan: menentukan bucket, kategori, jenis, dan batas ukuran. F1-T29 foto KTP (US-106); F1-T93
/// gambar konten CMS (US-1802).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UploadPurpose {
    /// Foto KTP pemilik akun (US-106 AC3).
    KtpPhoto,
    /// Gambar konten CMS (US-1802, F1-T93): bucket `public_content` yang tetap privat; disajikan endpoint
    /// gambar publik backend hanya untuk konten yang sedang tayang (ADR-027 butir 5).
    ContentImage,
}

impl UploadPurpose {
    /// Bucket logis.
    pub const fn bucket(self) -> LogicalBucket {
        match self {
            Self::KtpPhoto => LogicalBucket::IdentityDocument,
            Self::ContentImage => LogicalBucket::PublicContent,
        }
    }

    /// Nilai kolom `stored_file.category` (DD A19.6).
    pub const fn category(self) -> &'static str {
        match self {
            Self::KtpPhoto => "ktp_photo",
            Self::ContentImage => "cms_image",
        }
    }

    /// Batas ukuran berkas seperti diunggah.
    pub const fn max_size_bytes(self) -> u64 {
        match self {
            Self::KtpPhoto | Self::ContentImage => MAX_PHOTO_BYTES,
        }
    }

    /// Apakah jenis foto diizinkan untuk tujuan ini.
    pub fn allows(self, format: PhotoFormat) -> bool {
        match self {
            Self::KtpPhoto | Self::ContentImage => {
                matches!(format, PhotoFormat::Jpeg | PhotoFormat::Png)
            }
        }
    }
}

/// Alasan penolakan isi (dipetakan ke `4130001` / `4150001`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ContentRejection {
    /// Ukuran melebihi batas.
    TooLarge,
    /// Jenis menurut isi tidak diizinkan, tidak cocok dengan yang dinyatakan, atau isi tidak dapat didekode.
    UnsupportedType,
}

impl ContentRejection {
    /// Kode alasan untuk log (daftar tertutup, tanpa isi berkas).
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::TooLarge => "too_large",
            Self::UnsupportedType => "unsupported_type",
        }
    }
}

/// Foto yang sudah dikode ulang.
pub(crate) struct SanitizedPhoto {
    pub(crate) bytes: Vec<u8>,
    pub(crate) format: PhotoFormat,
}

/// Jenis foto menurut magic bytes: JPEG `FF D8 FF`, PNG `89 50 4E 47 0D 0A 1A 0A` [SRC-555].
pub(crate) fn sniff_photo_format(bytes: &[u8]) -> Option<PhotoFormat> {
    const PNG: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        Some(PhotoFormat::Jpeg)
    } else if bytes.starts_with(&PNG) {
        Some(PhotoFormat::Png)
    } else {
        None
    }
}

fn decode_limits() -> Limits {
    let mut limits = Limits::default();
    limits.max_image_width = Some(MAX_IMAGE_DIMENSION);
    limits.max_image_height = Some(MAX_IMAGE_DIMENSION);
    limits.max_alloc = Some(MAX_DECODE_ALLOC);
    limits
}

/// Memvalidasi dan mengode ulang foto. `declared` adalah jenis yang dinyatakan saat slot diminta (dan
/// ditandatangani di URL unggah); isi wajib berjenis sama.
pub(crate) fn sanitize_photo(
    bytes: &[u8],
    declared: PhotoFormat,
    max_size_bytes: u64,
) -> Result<SanitizedPhoto, ContentRejection> {
    let len = u64::try_from(bytes.len()).map_err(|_| ContentRejection::TooLarge)?;
    if len > max_size_bytes {
        return Err(ContentRejection::TooLarge);
    }
    match sniff_photo_format(bytes) {
        Some(f) if f == declared => {}
        _ => return Err(ContentRejection::UnsupportedType),
    }
    let mut reader = ImageReader::with_format(Cursor::new(bytes), declared.image_format());
    reader.limits(decode_limits());
    let mut decoder = reader
        .into_decoder()
        .map_err(|_| ContentRejection::UnsupportedType)?;
    // EXIF rusak tidak menggagalkan: orientasi dianggap normal (EXIF tetap dibuang).
    let orientation = decoder.orientation().ok();
    let mut image =
        DynamicImage::from_decoder(decoder).map_err(|_| ContentRejection::UnsupportedType)?;
    if let Some(o) = orientation {
        image.apply_orientation(o);
    }
    let mut out = Vec::new();
    let encoded = match declared {
        PhotoFormat::Jpeg => {
            // JPEG tanpa kanal alfa; warna dikonversi bila perlu (L8 tetap L8).
            let image = match image {
                DynamicImage::ImageLuma8(_) | DynamicImage::ImageRgb8(_) => image,
                other => DynamicImage::ImageRgb8(other.to_rgb8()),
            };
            image.write_with_encoder(JpegEncoder::new_with_quality(
                &mut out,
                JPEG_REENCODE_QUALITY,
            ))
        }
        PhotoFormat::Png => image.write_with_encoder(PngEncoder::new(&mut out)),
    };
    encoded.map_err(|_| ContentRejection::UnsupportedType)?;
    Ok(SanitizedPhoto {
        bytes: out,
        format: declared,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgb, RgbImage, Rgba, RgbaImage};

    fn jpeg(w: u32, h: u32) -> Vec<u8> {
        let img = RgbImage::from_fn(w, h, |x, y| Rgb([(x * 40) as u8, (y * 40) as u8, 128]));
        let mut out = Vec::new();
        DynamicImage::ImageRgb8(img)
            .write_with_encoder(JpegEncoder::new_with_quality(&mut out, 90))
            .unwrap();
        out
    }

    fn png(w: u32, h: u32) -> Vec<u8> {
        let img = RgbaImage::from_fn(w, h, |x, y| Rgba([(x * 40) as u8, (y * 40) as u8, 7, 200]));
        let mut out = Vec::new();
        DynamicImage::ImageRgba8(img)
            .write_with_encoder(PngEncoder::new(&mut out))
            .unwrap();
        out
    }

    #[test]
    fn sniffing_uses_magic_bytes_only() {
        assert_eq!(sniff_photo_format(&jpeg(2, 2)), Some(PhotoFormat::Jpeg));
        assert_eq!(sniff_photo_format(&png(2, 2)), Some(PhotoFormat::Png));
        assert_eq!(
            sniff_photo_format(b"<svg xmlns=\"http://www.w3.org/2000/svg\"/>"),
            None
        );
        assert_eq!(sniff_photo_format(b"GIF89a"), None);
        assert_eq!(sniff_photo_format(&[0xFF, 0xD8]), None);
        assert_eq!(sniff_photo_format(&[]), None);
    }

    #[test]
    fn valid_photos_are_reencoded_as_declared_type() {
        let out = sanitize_photo(&jpeg(4, 3), PhotoFormat::Jpeg, MAX_PHOTO_BYTES).unwrap();
        assert_eq!(sniff_photo_format(&out.bytes), Some(PhotoFormat::Jpeg));
        let out = sanitize_photo(&png(4, 3), PhotoFormat::Png, MAX_PHOTO_BYTES).unwrap();
        assert_eq!(sniff_photo_format(&out.bytes), Some(PhotoFormat::Png));
        assert_eq!(out.format, PhotoFormat::Png);
    }

    #[test]
    fn declared_type_mismatch_is_rejected() {
        assert!(matches!(
            sanitize_photo(&png(2, 2), PhotoFormat::Jpeg, MAX_PHOTO_BYTES),
            Err(ContentRejection::UnsupportedType)
        ));
        assert!(matches!(
            sanitize_photo(&jpeg(2, 2), PhotoFormat::Png, MAX_PHOTO_BYTES),
            Err(ContentRejection::UnsupportedType)
        ));
    }

    #[test]
    fn magic_bytes_without_decodable_image_are_rejected() {
        let mut fake = vec![0xFF, 0xD8, 0xFF, 0xE0];
        fake.extend_from_slice(b"<html><script>alert(1)</script></html>");
        assert!(matches!(
            sanitize_photo(&fake, PhotoFormat::Jpeg, MAX_PHOTO_BYTES),
            Err(ContentRejection::UnsupportedType)
        ));
        let mut fake = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
        fake.extend_from_slice(b"<svg onload=alert(1)>");
        assert!(matches!(
            sanitize_photo(&fake, PhotoFormat::Png, MAX_PHOTO_BYTES),
            Err(ContentRejection::UnsupportedType)
        ));
        // PNG terpotong (dekoder JPEG mode tidak ketat dapat mengisi sisa JPEG terpotong; hasilnya tetap
        // dikode ulang sebagai gambar baru).
        let full = png(64, 64);
        assert!(
            sanitize_photo(&full[..full.len() / 2], PhotoFormat::Png, MAX_PHOTO_BYTES).is_err()
        );
    }

    #[test]
    fn size_limit_is_checked_before_decoding() {
        let img = jpeg(2, 2);
        let len = img.len() as u64;
        assert!(sanitize_photo(&img, PhotoFormat::Jpeg, len).is_ok());
        assert!(matches!(
            sanitize_photo(&img, PhotoFormat::Jpeg, len - 1),
            Err(ContentRejection::TooLarge)
        ));
    }

    #[test]
    fn oversized_dimensions_are_rejected() {
        // Header PNG mengklaim 20.000 × 1 piksel (di atas MAX_IMAGE_DIMENSION).
        let img = RgbImage::from_pixel(MAX_IMAGE_DIMENSION + 1, 1, Rgb([1, 2, 3]));
        let mut out = Vec::new();
        DynamicImage::ImageRgb8(img)
            .write_with_encoder(PngEncoder::new(&mut out))
            .unwrap();
        assert!(matches!(
            sanitize_photo(&out, PhotoFormat::Png, MAX_PHOTO_BYTES),
            Err(ContentRejection::UnsupportedType)
        ));
    }

    #[test]
    fn purpose_and_bucket_values_match_schema() {
        assert_eq!(
            UploadPurpose::KtpPhoto.bucket().as_str(),
            "identity_document"
        );
        assert_eq!(UploadPurpose::KtpPhoto.category(), "ktp_photo");
        assert_eq!(
            UploadPurpose::ContentImage.bucket(),
            LogicalBucket::PublicContent
        );
        assert_eq!(UploadPurpose::ContentImage.category(), "cms_image");
        assert_eq!(UploadPurpose::ContentImage.max_size_bytes(), 5_242_880);
        assert_eq!(UploadPurpose::KtpPhoto.max_size_bytes(), 5_242_880);
        assert!(UploadPurpose::KtpPhoto.allows(PhotoFormat::Png));
        for b in LogicalBucket::ALL {
            assert_eq!(LogicalBucket::parse(b.as_str()), Some(b));
        }
        assert_eq!(PhotoFormat::from_mime("image/svg+xml"), None);
        assert_eq!(PhotoFormat::from_mime("image/png"), Some(PhotoFormat::Png));
    }
}
