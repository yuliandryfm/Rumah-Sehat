//! Pembangkitan spesifikasi OpenAPI 3.1 dengan utoipa (ADR-022, docs/05-stack.md §1.1 [SRC-168]).
//!
//! Spesifikasi dibangun dari anotasi `#[utoipa::path]` pada handler lewat `utoipa-axum`, yang juga
//! memasang rute di router axum dari anotasi yang sama; endpoint di kode dan di spesifikasi karena itu
//! tidak dapat berbeda. Berkas hasilnya di-commit di `openapi/` dan harus sama dengan hasil generate
//! (test `tests/openapi_sync.rs`, langkah `openapi-sinkron` di `scripts/verify.sh`).
//!
//! Berkas yang dihasilkan: `ops.json` (endpoint operasional tanpa audiens: kesehatan) dan `backoffice.json`
//! (F1-T92, `/backoffice/v1`), `pasien.json` (F1-T61, `/patient/v1`), `public.json` (F1-T94,
//! `/v1/public/content`). Berkas audiens lain (`nakes`, `portal`, `partner-v1`, `webhook`,
//! `rme`; docs/05-stack.md §7) ditambahkan task yang memperkenalkan endpoint audiens itu.

use serde::Serialize;
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi, ToSchema};

/// Satu galat per isian (`errors[]`), sama dengan `envelope::FieldError`.
#[derive(Debug, Serialize, ToSchema)]
pub struct FieldErrorSchema {
    /// Path isian.
    #[schema(example = "email")]
    pub field: String,
    /// Kode galat isian, snake_case Inggris.
    #[schema(example = "invalid_format")]
    pub code: String,
    /// Pesan Bahasa Indonesia untuk pengguna.
    #[schema(example = "Format email tidak valid")]
    pub message: String,
}

/// Bentuk dokumentasi respons galat (RFC 9457 yang kompatibel dengan ADR-023 butir 3). Isi sebenarnya
/// dibangun `envelope::ErrorBody`; test `tests/openapi_schema_drift.rs` menjaga keduanya sama.
#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorEnvelope {
    /// URI jenis masalah: `<awalan>/<response_code>`.
    #[serde(rename = "type")]
    #[schema(example = "https://problems.ruang-sehat.invalid/4040001")]
    pub type_uri: String,
    /// Kode 7 digit `HHHMMCC` (docs/05-response-codes.md).
    #[schema(example = "4040001")]
    pub response_code: String,
    /// Deskripsi Bahasa Indonesia untuk kode itu.
    #[schema(example = "Data tidak ditemukan")]
    pub response_description: String,
    /// Galat per isian; hanya pada galat validasi.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<FieldErrorSchema>,
    /// Pola rute (bukan URL dengan ID).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(nullable = false)]
    pub instance: Option<String>,
    /// UUID korelasi permintaan; selalu ada pada galat.
    #[schema(example = "01936f8e-9c01-7e02-8f03-1a2b3c4d5e6f")]
    pub correlation_id: String,
}

/// Pagination cursor `meta` (ADR-023 butir 1), sama dengan `envelope::Meta`.
#[derive(Debug, Serialize, ToSchema)]
pub struct MetaSchema {
    /// Cursor halaman berikutnya; `null` bila tidak ada.
    pub next_cursor: Option<String>,
    /// Apakah masih ada data setelah halaman ini.
    pub has_more: bool,
}

/// Skema keamanan `bearer`: token akses Keycloak di header `Authorization` (diteruskan BFF backoffice,
/// ADR-011 butir 7; token tidak pernah sampai ke browser).
struct BearerAuth;

impl Modify for BearerAuth {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            "bearer",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .build(),
            ),
        );
    }
}

/// Dasar spesifikasi operasional; path dan skema ditambahkan dari anotasi handler.
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Ruang Sehat: API operasional",
        description = "Endpoint operasional tanpa autentikasi (kesehatan). Format respons: ADR-023.",
        version = env!("CARGO_PKG_VERSION")
    ),
    tags((name = "ops", description = "Operasional"))
)]
pub struct OpsApiDoc;

/// Dasar spesifikasi backoffice (admin Ruang Sehat, realm `admin`); path dari anotasi handler modul.
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Ruang Sehat: API backoffice",
        description = "Endpoint backoffice admin Ruang Sehat (realm admin, sub-peran per endpoint, UP-18). Format respons: ADR-023.",
        version = env!("CARGO_PKG_VERSION")
    ),
    tags((name = "cms", description = "Konten (CMS) aplikasi dan situs")),
    modifiers(&BearerAuth)
)]
pub struct BackofficeApiDoc;

/// Dasar spesifikasi aplikasi pasien (realm `pasien`, `/patient/v1`); path dari anotasi handler modul.
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Ruang Sehat: API aplikasi pasien",
        description = "Endpoint aplikasi mobile pasien (realm pasien, akun berjenis pasien; relasi ke penerima layanan diperiksa per objek, KU-03). Kontrak antar-repo untuk repositori mobile (CR-019). Format respons: ADR-023.",
        version = env!("CARGO_PKG_VERSION")
    ),
    tags((name = "identity", description = "NIK dan foto KTP (US-106)")),
    modifiers(&BearerAuth)
)]
pub struct PatientApiDoc;

/// Dasar spesifikasi konten publik tanpa login (situs, ADR-027 butir 2 dan 5).
#[derive(OpenApi)]
#[openapi(
    info(
        title = "Ruang Sehat: API konten publik",
        description = "Konten situs yang sedang tayang tanpa login (ADR-012 butir 3, ADR-027 butir 5), dengan batas laju. Format respons: ADR-023.",
        version = env!("CARGO_PKG_VERSION")
    ),
    tags((name = "public-content", description = "Konten situs yang sedang tayang"))
)]
pub struct PublicApiDoc;

/// Seluruh spesifikasi yang dihasilkan sebagai pasangan (nama berkas, isi JSON). Isi diakhiri satu
/// baris baru; urutan kunci deterministik (utoipa memakai BTreeMap tanpa fitur `preserve_order`).
/// `ops.json` selalu pertama (dipakai test `openapi_schema_drift`).
pub fn generate() -> Result<Vec<(&'static str, String)>, serde_json::Error> {
    let specs = [
        ("ops.json", crate::app::ops_router().into_openapi()),
        (
            "backoffice.json",
            crate::app::backoffice_router().into_openapi(),
        ),
        ("pasien.json", crate::app::patient_router().into_openapi()),
        ("public.json", crate::app::public_router().into_openapi()),
    ];
    let mut out = Vec::with_capacity(specs.len());
    for (name, mut spec) in specs {
        // utoipa mengisi lisensi kosong bawaan; lisensi proyek [TIDAK DIKETAHUI], jadi tidak diklaim.
        spec.info.license = None;
        let mut json = spec.to_pretty_json()?;
        json.push('\n');
        out.push((name, json));
    }
    Ok(out)
}
