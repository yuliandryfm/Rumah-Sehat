//! DTO endpoint kesehatan. Isinya sengaja minimal dan tanpa data pribadi, versi, atau nama
//! dependensi (docs/05-stack.md §7: endpoint kesehatan dibaca pemeriksa luar).

use serde::Serialize;
use utoipa::ToSchema;

/// Status kesehatan.
#[derive(Debug, Clone, Copy, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub(super) enum HealthStatus {
    /// Proses sehat atau siap.
    Ok,
}

/// Isi `data` respons kesehatan.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub(super) struct HealthData {
    pub(super) status: HealthStatus,
}

/// Bentuk dokumentasi respons sukses kesehatan (ADR-023 butir 1). Dipakai hanya untuk skema
/// OpenAPI; isi sebenarnya dibangun `envelope::Success`. Test `tests/openapi_schema_drift.rs`
/// menjaga keduanya tetap sama.
#[derive(ToSchema)]
#[allow(dead_code)] // hanya untuk pembangkitan skema
pub(super) struct HealthEnvelope {
    /// Kode 7 digit `HHHMMCC` (docs/05-response-codes.md).
    #[schema(example = "2000000")]
    response_code: String,
    /// Deskripsi Bahasa Indonesia untuk kode itu.
    #[schema(example = "Berhasil")]
    response_description: String,
    data: HealthData,
    /// UUID korelasi permintaan.
    #[schema(example = "01936f8e-9c01-7e02-8f03-1a2b3c4d5e6f", nullable = false)]
    correlation_id: Option<String>,
}
