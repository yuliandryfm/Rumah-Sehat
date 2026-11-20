//! Pembangkitan spesifikasi OpenAPI 3.1 dengan utoipa (ADR-022, docs/05-stack.md §1.1 [SRC-168]).
//!
//! Spesifikasi dibangun dari anotasi `#[utoipa::path]` pada handler lewat `utoipa-axum`, yang juga
//! memasang rute di router axum dari anotasi yang sama; endpoint di kode dan di spesifikasi karena itu
//! tidak dapat berbeda. Berkas hasilnya di-commit di `openapi/` dan harus sama dengan hasil generate
//! (test `tests/openapi_sync.rs`, langkah `openapi-sinkron` di `scripts/verify.sh`).
//!
//! Berkas yang dihasilkan: `ops.json` (endpoint operasional tanpa audiens: kesehatan). Berkas per
//! audiens (`pasien`, `nakes`, `backoffice`, `portal`, `partner-v1`, `webhook`, `rme`;
//! docs/05-stack.md §7) ditambahkan task yang memperkenalkan endpoint audiens itu.

use serde::Serialize;
use utoipa::{OpenApi, ToSchema};

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

/// Seluruh spesifikasi yang dihasilkan sebagai pasangan (nama berkas, isi JSON). Isi diakhiri satu
/// baris baru; urutan kunci deterministik (utoipa memakai BTreeMap tanpa fitur `preserve_order`).
pub fn generate() -> Result<Vec<(&'static str, String)>, serde_json::Error> {
    let mut ops = crate::app::ops_router().into_openapi();
    // utoipa mengisi lisensi kosong bawaan; lisensi proyek [TIDAK DIKETAHUI], jadi tidak diklaim.
    ops.info.license = None;
    let mut json = ops.to_pretty_json()?;
    json.push('\n');
    Ok(vec![("ops.json", json)])
}
