//! DTO endpoint profil dan alamat pasien (`/patient/v1`, F1-T56, US-105). Field JSON snake_case bahasa
//! Inggris (coding-standards §2.5); permintaan menolak field yang tidak dikenal. DTO berisi data pribadi
//! memakai `Debug` manual tanpa nilai (KEPUTUSAN-122, ADR-005).

use super::api::{AddressInput, AddressView, ProfileInput, ProfileView};
use super::dto::{DtoError, IdentityStatusDto};
use serde::{Deserialize, Serialize};
use std::fmt;
use time::format_description::well_known::Iso8601;
use utoipa::ToSchema;

/// Jenis kelamin (CHECK `ck_care_recipient_sex`).
#[derive(Clone, Copy, PartialEq, Eq, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub(super) enum SexDto {
    /// Laki-laki.
    Male,
    /// Perempuan.
    Female,
}

impl SexDto {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Male => "male",
            Self::Female => "female",
        }
    }

    fn parse(code: &str) -> Option<Self> {
        match code {
            "male" => Some(Self::Male),
            "female" => Some(Self::Female),
            _ => None,
        }
    }
}

/// Simpan profil (US-105 AC1, AC1a, AC1b). Penggantian penuh: `phone` yang tidak dikirim atau `null`
/// mengosongkan nomor HP. Perubahan NIK bukan lewat sini (US-106).
#[derive(Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct UpdateProfileRequest {
    /// 1-100 karakter setelah dipangkas; huruf, spasi, titik, koma, apostrof, tanda hubung.
    #[schema(example = "Budi Santoso", max_length = 100)]
    pub(super) full_name: String,
    /// Tanggal sah `TTTT-BB-HH`; tidak di masa depan (tanggal WIB), tidak lebih dari 120 tahun lalu, usia
    /// minimum pemilik akun (PB-22, nilai uji Fase 1).
    #[schema(example = "1990-01-31", format = Date)]
    pub(super) birth_date: String,
    pub(super) sex: SexDto,
    /// Spasi dan tanda hubung dibuang; awalan `+62` atau `0`; 8-13 digit setelah kode negara; disimpan
    /// sebagai `+62...`.
    #[schema(example = "0812-3456-7890", nullable)]
    pub(super) phone: Option<String>,
}

impl fmt::Debug for UpdateProfileRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UpdateProfileRequest")
            .finish_non_exhaustive()
    }
}

impl UpdateProfileRequest {
    pub(super) fn input(&self) -> ProfileInput<'_> {
        ProfileInput {
            full_name: &self.full_name,
            birth_date: &self.birth_date,
            sex: self.sex.as_str(),
            phone: self.phone.as_deref(),
        }
    }
}

/// Profil pemilik akun.
#[derive(Serialize, ToSchema)]
pub(super) struct ProfileDto {
    #[schema(example = "0193a1b2-c3d4-7e5f-8a6b-7c8d9e0f1a2b")]
    pub(super) care_recipient_id: String,
    #[schema(example = "Budi Santoso")]
    pub(super) full_name: String,
    #[schema(example = "1990-01-31", format = Date)]
    pub(super) birth_date: String,
    pub(super) sex: SexDto,
    #[schema(example = "+6281234567890", nullable)]
    pub(super) phone: Option<String>,
    /// Status identitas (AB-05). Berubah ke `pending` bila nama atau tanggal lahir diubah saat `approved`
    /// (US-105 AC3).
    pub(super) identity_status: IdentityStatusDto,
}

impl fmt::Debug for ProfileDto {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ProfileDto").finish_non_exhaustive()
    }
}

impl ProfileDto {
    pub(super) fn from_view(v: ProfileView) -> Result<Self, DtoError> {
        Ok(Self {
            care_recipient_id: v.care_recipient_id.hyphenated().to_string(),
            full_name: v.full_name,
            birth_date: v.birth_date.format(&Iso8601::DATE).map_err(|_| DtoError)?,
            sex: SexDto::parse(&v.sex).ok_or(DtoError)?,
            phone: v.phone,
            identity_status: IdentityStatusDto::parse(v.identity_status).ok_or(DtoError)?,
        })
    }
}

/// Bentuk dokumentasi respons profil.
#[derive(ToSchema)]
#[allow(dead_code)] // hanya untuk pembangkitan skema
pub(super) struct ProfileEnvelope {
    #[schema(example = "2000000")]
    response_code: String,
    #[schema(example = "Berhasil")]
    response_description: String,
    data: ProfileDto,
    #[schema(nullable = false)]
    correlation_id: Option<String>,
}

/// Tambah atau ubah alamat (US-105 AC1a, AC2). Pada ubah (`PUT`) semua field dikirim ulang.
#[derive(Deserialize, ToSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct AddressRequest {
    /// Paling banyak 30 karakter, mis. "Rumah".
    #[schema(example = "Rumah", max_length = 30, nullable)]
    pub(super) label: Option<String>,
    /// Aturan nama seperti nama lengkap.
    #[schema(example = "Siti Aminah", max_length = 100)]
    pub(super) recipient_name: String,
    /// Aturan nomor HP seperti profil.
    #[schema(example = "081234567890")]
    pub(super) recipient_phone: String,
    /// 5-500 karakter.
    #[schema(
        example = "Jl. Contoh No. 1, RT 01/RW 02",
        min_length = 5,
        max_length = 500
    )]
    pub(super) address_line: String,
    /// 2-100 karakter (sampai sistem kode wilayah ditetapkan).
    #[schema(example = "DKI Jakarta", min_length = 2, max_length = 100)]
    pub(super) province_name: String,
    /// 2-100 karakter (sampai sistem kode wilayah ditetapkan).
    #[schema(example = "Jakarta Selatan", min_length = 2, max_length = 100)]
    pub(super) city_name: String,
    /// Hanya angka.
    #[schema(example = "12345", pattern = "^[0-9]+$", max_length = 10, nullable)]
    pub(super) postal_code: Option<String>,
    /// Patokan alamat, paling banyak 200 karakter.
    #[schema(example = "Sebelah warung biru", max_length = 200, nullable)]
    pub(super) notes: Option<String>,
    /// Nama zona waktu IANA; bawaan `Asia/Jakarta`.
    #[schema(example = "Asia/Jakarta", nullable)]
    pub(super) time_zone: Option<String>,
    /// `true` menjadikan alamat ini utama (yang lama dilepas). Alamat pertama selalu utama. `false` pada
    /// alamat utama ditolak: pilih alamat lain sebagai utama.
    #[schema(nullable)]
    pub(super) is_default: Option<bool>,
}

impl fmt::Debug for AddressRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AddressRequest").finish_non_exhaustive()
    }
}

impl AddressRequest {
    pub(super) fn input(&self) -> AddressInput<'_> {
        AddressInput {
            label: self.label.as_deref(),
            recipient_name: &self.recipient_name,
            recipient_phone: &self.recipient_phone,
            address_line: &self.address_line,
            province_name: &self.province_name,
            city_name: &self.city_name,
            postal_code: self.postal_code.as_deref(),
            notes: self.notes.as_deref(),
            time_zone: self.time_zone.as_deref(),
            is_default: self.is_default,
        }
    }
}

/// Satu alamat.
#[derive(Serialize, ToSchema)]
pub(super) struct AddressDto {
    #[schema(example = "0193a1b2-c3d4-7e5f-8a6b-7c8d9e0f1a2b")]
    pub(super) id: String,
    #[schema(nullable)]
    pub(super) label: Option<String>,
    pub(super) recipient_name: String,
    #[schema(example = "+6281234567890")]
    pub(super) recipient_phone: String,
    pub(super) address_line: String,
    pub(super) province_name: String,
    pub(super) city_name: String,
    #[schema(nullable)]
    pub(super) postal_code: Option<String>,
    #[schema(nullable)]
    pub(super) notes: Option<String>,
    #[schema(example = "Asia/Jakarta")]
    pub(super) time_zone: String,
    pub(super) is_default: bool,
}

impl fmt::Debug for AddressDto {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AddressDto").finish_non_exhaustive()
    }
}

impl From<AddressView> for AddressDto {
    fn from(a: AddressView) -> Self {
        Self {
            id: a.id.hyphenated().to_string(),
            label: a.label,
            recipient_name: a.recipient_name,
            recipient_phone: a.recipient_phone,
            address_line: a.address_line,
            province_name: a.province_name,
            city_name: a.city_name,
            postal_code: a.postal_code,
            notes: a.notes,
            time_zone: a.time_zone,
            is_default: a.is_default,
        }
    }
}

/// Daftar alamat aktif: alamat utama pertama, lalu urutan dibuat. Tepat satu `is_default` bila tidak
/// kosong.
#[derive(Serialize, ToSchema)]
pub(super) struct AddressListDto {
    pub(super) items: Vec<AddressDto>,
}

impl fmt::Debug for AddressListDto {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AddressListDto")
            .field("count", &self.items.len())
            .finish()
    }
}

impl AddressListDto {
    pub(super) fn from_views(v: Vec<AddressView>) -> Self {
        Self {
            items: v.into_iter().map(AddressDto::from).collect(),
        }
    }
}

/// Bentuk dokumentasi respons daftar alamat (200).
#[derive(ToSchema)]
#[allow(dead_code)] // hanya untuk pembangkitan skema
pub(super) struct AddressListEnvelope {
    #[schema(example = "2000000")]
    response_code: String,
    #[schema(example = "Berhasil")]
    response_description: String,
    data: AddressListDto,
    #[schema(nullable = false)]
    correlation_id: Option<String>,
}

/// Bentuk dokumentasi respons daftar alamat setelah tambah (201).
#[derive(ToSchema)]
#[allow(dead_code)] // hanya untuk pembangkitan skema
pub(super) struct AddressListCreatedEnvelope {
    #[schema(example = "2010000")]
    response_code: String,
    #[schema(example = "Berhasil dibuat")]
    response_description: String,
    data: AddressListDto,
    #[schema(nullable = false)]
    correlation_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // trace: KEPUTUSAN-122, ADR-005 (data pribadi profil tidak ada di Debug)
    #[test]
    fn debug_hides_personal_data() {
        let req = UpdateProfileRequest {
            full_name: "Nama Rahasia".to_string(),
            birth_date: "1990-01-01".to_string(),
            sex: SexDto::Male,
            phone: Some("0812-0000".to_string()),
        };
        let d = format!("{req:?}");
        assert!(
            !d.contains("Rahasia") && !d.contains("1990") && !d.contains("0812"),
            "{d}"
        );
        let a = AddressRequest {
            label: None,
            recipient_name: "Penerima Rahasia".to_string(),
            recipient_phone: "0812".to_string(),
            address_line: "Jalan Rahasia".to_string(),
            province_name: "P".to_string(),
            city_name: "K".to_string(),
            postal_code: None,
            notes: None,
            time_zone: None,
            is_default: None,
        };
        assert!(!format!("{a:?}").contains("Rahasia"));
    }
}
