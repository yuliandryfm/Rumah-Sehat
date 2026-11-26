//! Handler modul `privacy` untuk API pasien (`/patient/v1`, KEPUTUSAN-120). Handler hanya membaca permintaan,
//! memanggil satu fungsi service, dan memetakan hasil ke envelope ADR-023.

use super::error::PrivacyError;
use super::policy;
use super::service::{self, ActiveConsentText};
use crate::app::AppState;
use crate::http_support::{ApiFailure, ApiSuccess, ClientAddr};
use crate::openapi::ErrorEnvelope;
use crate::rate_limit::Verdict;
use authz::PolicyRouter;
use axum::Extension;
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use envelope::Success;
use foundation::CorrelationId;
use observability::security;
use serde::Serialize;
use time::format_description::well_known::Rfc3339;
use utoipa::ToSchema;
use utoipa_axum::routes;

pub(super) fn router() -> PolicyRouter<AppState> {
    PolicyRouter::new().routes(policy::consent_text(), routes!(current_consent_text))
}

/// Satu bagian teks persetujuan.
#[derive(Debug, Serialize, ToSchema)]
pub(super) struct ConsentSectionDto {
    /// Kode tetap bagian Pasal 21 ayat (1) UU PDP: `legal_basis` (a), `purpose` (b), `data_types` (c),
    /// `document_retention` (d), `collected_information` (e), `processing_period` (f), `subject_rights` (g).
    #[schema(example = "purpose")]
    code: String,
    #[schema(example = "(b) Tujuan pemrosesan")]
    title: String,
    /// Isi bagian, teks biasa (bukan HTML).
    body: String,
}

/// Kontak Ruang Sehat.
#[derive(Debug, Serialize, ToSchema)]
pub(super) struct ConsentContactDto {
    #[schema(example = "Kontak Ruang Sehat")]
    title: String,
    body: String,
}

/// Butir persetujuan: satu kotak centang yang tidak tercentang otomatis.
#[derive(Debug, Serialize, ToSchema)]
pub(super) struct ConsentItemDto {
    #[schema(example = "01a2008d-7400-7000-8000-000000000a4a")]
    consent_item_id: String,
    #[schema(example = "data_processing")]
    item_code: String,
    label: String,
    /// Butir utama (penarikannya membuat akun "Persetujuan ditarik", US-1601 AC2).
    is_primary: bool,
    /// Wajib dipilih untuk mendaftar (US-101 AC4).
    is_required: bool,
    sort_order: i16,
}

/// Versi teks persetujuan pemrosesan data yang berlaku (US-104 AC1).
#[derive(Debug, Serialize, ToSchema)]
pub(super) struct ConsentTextDto {
    #[schema(example = "01a2008d-7400-7000-8000-000000000a49")]
    consent_text_version_id: String,
    #[schema(example = "patient_data_processing")]
    consent_type: String,
    #[schema(example = "0.1-draf")]
    version_label: String,
    #[schema(format = DateTime, example = "2026-11-22T00:00:01Z")]
    effective_from: String,
    /// Penanda teks draf yang belum final (`null` untuk teks final yang disetujui legal dan Petugas PDP).
    draft_notice: Option<String>,
    /// Tujuh bagian, urut (a)–(g).
    sections: Vec<ConsentSectionDto>,
    contact: ConsentContactDto,
    /// Butir persetujuan, urut tampilan.
    items: Vec<ConsentItemDto>,
}

impl ConsentTextDto {
    fn from_text(t: ActiveConsentText) -> Result<Self, time::error::Format> {
        Ok(Self {
            consent_text_version_id: t.version_id.to_string(),
            consent_type: t.consent_type,
            version_label: t.version_label,
            effective_from: t.effective_from.format(&Rfc3339)?,
            draft_notice: t.content.draft_notice,
            sections: t
                .content
                .sections
                .into_iter()
                .map(|s| ConsentSectionDto {
                    code: s.code,
                    title: s.title,
                    body: s.body,
                })
                .collect(),
            contact: ConsentContactDto {
                title: t.content.contact.title,
                body: t.content.contact.body,
            },
            items: t
                .items
                .into_iter()
                .map(|i| ConsentItemDto {
                    consent_item_id: i.id.to_string(),
                    item_code: i.item_code,
                    label: i.label,
                    is_primary: i.is_primary,
                    is_required: i.is_required,
                    sort_order: i.sort_order,
                })
                .collect(),
        })
    }
}

/// Bentuk dokumentasi respons teks persetujuan (ADR-023 butir 1); isi sebenarnya `envelope::Success`.
#[derive(ToSchema)]
#[allow(dead_code)] // hanya untuk pembangkitan skema
pub(super) struct ConsentTextEnvelope {
    #[schema(example = "2000000")]
    response_code: String,
    #[schema(example = "Berhasil")]
    response_description: String,
    data: ConsentTextDto,
    #[schema(nullable = false)]
    correlation_id: Option<String>,
}

fn fail(state: &AppState, cid: CorrelationId, e: PrivacyError) -> Response {
    if let PrivacyError::Internal(what) = &e {
        tracing::error!(error = *what, "privacy_request_failed");
    }
    ApiFailure::new(e, cid, state.problem_type_base()).into_response()
}

/// Teks persetujuan pemrosesan data yang berlaku untuk layar pendaftaran (US-104 AC1, AC3): tujuh bagian
/// Pasal 21 ayat (1) UU PDP, kontak, dan butir persetujuan. Tanpa login, dengan batas laju.
#[utoipa::path(
    get,
    path = "/patient/v1/consent-texts/current",
    tag = "consent",
    operation_id = "consent_text_current",
    responses(
        (status = 200, description = "Versi teks yang berlaku.", body = ConsentTextEnvelope, content_type = "application/json"),
        (status = 429, description = "Batas laju terlampaui.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 500, description = "Galat tak terduga.", body = ErrorEnvelope, content_type = "application/problem+json"),
        (status = 503, description = "Belum ada versi teks yang berlaku atau layanan tidak tersedia.", body = ErrorEnvelope, content_type = "application/problem+json")
    )
)]
async fn current_consent_text(
    State(state): State<AppState>,
    Extension(cid): Extension<CorrelationId>,
    addr: ClientAddr,
) -> Response {
    if let Verdict::Limited { first } = state.consent_text_limit().check(addr.0) {
        if first {
            security::rate_limited(None, "/patient/v1/consent-texts/current");
        }
        return fail(&state, cid, PrivacyError::RateLimited);
    }
    let r = service::current_text(state.pool(), state.clock(), state.consent_draft_policy())
        .await
        .and_then(|t| {
            ConsentTextDto::from_text(t).map_err(|_| PrivacyError::Internal("timestamp_format"))
        });
    match r {
        Ok(d) => ApiSuccess(Success::ok(d).with_correlation_id(cid)).into_response(),
        Err(e) => fail(&state, cid, e),
    }
}
