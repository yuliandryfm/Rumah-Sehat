//! Use case modul `privacy`.
//!
//! Bukti persetujuan dan event E-13 di transaksi teraudit pemanggil (US-104 AC2, F1-T49).
//!
//! F1-T49: versi teks persetujuan yang berlaku beserta butirnya (US-104 AC1, AC3). Versi dipilih menurut jam
//! server aplikasi (`Clock`, KEPUTUSAN-131), bukan jam database, sehingga versi baru tampil tepat mulai
//! `effective_from` dan perilaku itu dapat diuji dengan jam tetap.

use super::domain::{ConsentContent, ConsentItem, ConsentType, parse_content};
use super::error::PrivacyError;
use super::repo::{self, NewConsentRecord};
use crate::modules::platform::api::{ReadinessCheck, ReadinessFuture};
use audit::{
    Actor, AuditContext, AuditEvent, Code, EventCode, FieldChange, FieldRule, ObjectPolicy,
    ObjectRef, Outcome, Subject, Value,
};
use foundation::{Clock, Uuid, new_id};
use sqlx::{PgConnection, PgPool};
use std::sync::Arc;
use time::OffsetDateTime;

/// Objek E-13: satu bukti persetujuan (DD A3.3); nilai yang boleh dicatat hanya kode butir dan aksinya.
pub(super) static CONSENT_RECORD_OBJECT: ObjectPolicy = ObjectPolicy::non_clinical(
    "consent_record",
    &[FieldRule::value("item_code"), FieldRule::value("action")],
);

/// Objek E-13 untuk persetujuan HR perusahaan (F1-T101, KEPUTUSAN-322): jenis objek berbeda dari pasien supaya
/// pengecualian subjek di `audit::requirements` (E-13 tanpa subjek orang perseorangan terdaftar) hanya berlaku
/// untuknya dan hanya bila `tenant_id` (perusahaan) terisi.
pub(super) static CORPORATE_HR_CONSENT_RECORD_OBJECT: ObjectPolicy = ObjectPolicy::non_clinical(
    audit::CORPORATE_HR_CONSENT_OBJECT_TYPE,
    &[FieldRule::value("item_code"), FieldRule::value("action")],
);

/// Versi teks persetujuan yang berlaku beserta butirnya.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveConsentText {
    /// `consent_text_version.id`.
    pub version_id: Uuid,
    /// Jenis persetujuan.
    pub consent_type: String,
    /// Label versi.
    pub version_label: String,
    /// Mulai berlaku.
    pub effective_from: OffsetDateTime,
    /// Isi tervalidasi (tujuh bagian + kontak).
    pub content: ConsentContent,
    /// Butir, urut tampilan.
    pub items: Vec<ConsentItem>,
}

/// Versi teks persetujuan jenis `consent_type` yang berlaku pada `now`, di koneksi `conn` (koneksi pool atau
/// transaksi pemanggil). `None` bila belum ada versi berlaku.
pub(super) async fn active_text(
    conn: &mut PgConnection,
    consent_type: ConsentType,
    now: OffsetDateTime,
    drafts: DraftPolicy,
) -> Result<Option<ActiveConsentText>, PrivacyError> {
    let Some(v) = repo::active_version(&mut *conn, consent_type.as_str(), now)
        .await
        .map_err(|_| PrivacyError::Unavailable)?
    else {
        return Ok(None);
    };
    let content = parse_content(&v.content).map_err(|p| {
        tracing::error!(error = p.code(), "consent_text_invalid");
        PrivacyError::Internal("consent_content")
    })?;
    if drafts == DraftPolicy::Reject && content.draft_notice.is_some() {
        // Production tidak menyajikan teks draf (KEPUTUSAN-289): sama dengan tidak ada versi berlaku (503),
        // isi draf tidak dibocorkan.
        tracing::error!("consent_text_draft_rejected");
        return Err(PrivacyError::Unavailable);
    }
    let items = repo::items(&mut *conn, v.id)
        .await
        .map_err(|_| PrivacyError::Unavailable)?;
    if !items.iter().any(|i| i.is_required) {
        // Versi tanpa butir wajib akan meloloskan pendaftaran tanpa persetujuan (US-101 AC4): tidak disajikan.
        tracing::error!("consent_text_without_required_item");
        return Err(PrivacyError::Internal("consent_items"));
    }
    Ok(Some(ActiveConsentText {
        version_id: v.id,
        consent_type: v.consent_type,
        version_label: v.version_label,
        effective_from: v.effective_from,
        content,
        items: items
            .into_iter()
            .map(|i| ConsentItem {
                id: i.id,
                item_code: i.item_code,
                label: i.label,
                is_primary: i.is_primary,
                is_required: i.is_required,
                sort_order: i.sort_order,
            })
            .collect(),
    }))
}

/// Teks persetujuan yang berlaku untuk layar pendaftaran (US-104 AC1). Tanpa versi berlaku = 503.
pub(super) async fn current_text(
    pool: &PgPool,
    clock: &dyn Clock,
    drafts: DraftPolicy,
) -> Result<ActiveConsentText, PrivacyError> {
    let mut conn = pool
        .acquire()
        .await
        .map_err(|_| PrivacyError::Unavailable)?;
    match active_text(
        &mut conn,
        ConsentType::PatientDataProcessing,
        clock.now(),
        drafts,
    )
    .await?
    {
        Some(t) => Ok(t),
        None => {
            tracing::error!("consent_text_missing");
            Err(PrivacyError::Unavailable)
        }
    }
}

/// Apakah teks berstatus draf (`draft_notice` terisi) boleh disajikan. Production memakai `Reject`
/// (KEPUTUSAN-289); local/staging/demo memakai `Allow`. Pemanggil `privacy::api` mengambilnya dari
/// `AppState::consent_draft_policy`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DraftPolicy {
    /// Draf disajikan.
    Allow,
    /// Draf ditolak sebagai [`PrivacyError::Unavailable`] (503), isi tidak dibocorkan.
    Reject,
}

/// Versi teks persetujuan jenis `consent_type` yang berlaku menurut `clock`. `None` bila belum ada. Dengan
/// `DraftPolicy::Reject`, versi berlaku yang masih draf ditolak sebagai `Unavailable`.
pub async fn active_consent_text(
    pool: &PgPool,
    clock: &dyn Clock,
    consent_type: ConsentType,
    drafts: DraftPolicy,
) -> Result<Option<ActiveConsentText>, PrivacyError> {
    let mut conn = pool
        .acquire()
        .await
        .map_err(|_| PrivacyError::Unavailable)?;
    active_text(&mut conn, consent_type, clock.now(), drafts).await
}

/// `true` bila versi berlaku ada dan isinya final (tanpa penanda draf, KEPUTUSAN-289).
pub fn is_final_text(text: Option<&ActiveConsentText>) -> bool {
    text.is_some_and(|t| t.content.draft_notice.is_none())
}

/// Pemeriksaan kesiapan proses `api` production (KEPUTUSAN-289): versi teks persetujuan pemrosesan data yang
/// berlaku harus ada, terbaca utuh, dan bukan draf "menunggu L-08/PT-24". Selama belum terpenuhi, endpoint
/// kesiapan menjawab tidak siap. Ini hanya lapis pertama (probe); penolakan draf yang sebenarnya ada di jalur
/// layanan ([`DraftPolicy::Reject`] lewat `AppState::reject_consent_drafts`), sehingga teks draf hasil seed
/// tidak pernah dijawab 200 di production. Hanya dipasang untuk `DEPLOY_ENV=production`; local/staging/demo tetap memakai teks draf.
pub struct ConsentTextReadiness {
    pool: PgPool,
    clock: Arc<dyn Clock>,
}

impl ConsentTextReadiness {
    /// Pemeriksaan atas pool dan jam proses `api`.
    pub fn new(pool: PgPool, clock: Arc<dyn Clock>) -> Self {
        Self { pool, clock }
    }
}

impl ReadinessCheck for ConsentTextReadiness {
    fn name(&self) -> &'static str {
        "consent_text_final"
    }

    fn check(&self) -> ReadinessFuture<'_> {
        Box::pin(async move {
            match active_consent_text(
                &self.pool,
                self.clock.as_ref(),
                ConsentType::PatientDataProcessing,
                DraftPolicy::Allow,
            )
            .await
            {
                Ok(text) => is_final_text(text.as_ref()),
                Err(_) => false,
            }
        })
    }
}

/// Pihak yang memberi persetujuan. `consent_record` (DD A3.3) kini berkunci ke pasien (`patient_id` NOT
/// NULL, FK ke `app.patient`), sehingga hanya pasien yang dapat dicatat. Pihak lain (mis. HR perusahaan,
/// F1-T101) menambah varian di sini bersamaan dengan perluasan skema A3 yang direview DBA; enum
/// `non_exhaustive` supaya penambahan itu tidak memecah pemanggil.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ConsentSubject {
    /// Pasien `patient_id` yang bertindak dengan akun `account_id` (kanal `patient_app`, peran audit
    /// `patient`).
    Patient {
        /// `app.patient.id`.
        patient_id: Uuid,
        /// `app.account.id` pelaku.
        account_id: Uuid,
        /// Penerima layanan pemilik akun (`is_account_owner`): subjek data event E-13 (PRD §7.2, event
        /// persetujuan selalu menyangkut data pribadi, F1-T69). Bukti `consent_record` tetap tingkat pasien.
        owner_care_recipient_id: Uuid,
    },
    /// HR perusahaan pendaftar (F1-T101, KEPUTUSAN-322): akun `account_id` realm `korporat` yang baru dibuat
    /// di transaksi pemanggil (kanal `corporate_portal`, peran audit `corporate_admin`). Bukti terikat ke
    /// akun, bukan ke pasien. E-13 memakai `tenant_id` = `corporate_client_id` dan tanpa subjek penerima
    /// layanan (HR bukan pasien).
    CorporateHr {
        /// `app.account.id` akun HR pendaftar.
        account_id: Uuid,
        /// `app.corporate_client.id` perusahaan yang didaftarkan (tenant event).
        corporate_client_id: Uuid,
    },
}

/// Penanda data event: subjek orang (pasien) atau tenant perusahaan (HR).
enum SubjectOrTenant {
    Subject(Subject),
    Tenant(Uuid),
}

impl ConsentSubject {
    /// Jenis teks yang sah untuk subjek ini.
    fn allowed_consent_types(&self) -> &'static [ConsentType] {
        match self {
            Self::Patient { .. } => &[
                ConsentType::PatientDataProcessing,
                ConsentType::FamilyAdultStatement,
                ConsentType::GuardianConsent,
            ],
            Self::CorporateHr { .. } => &[ConsentType::CorporateHrRegistration],
        }
    }
}

/// Menulis bukti persetujuan `granted` untuk setiap butir terpilih `items` versi `version_id` oleh `subject`
/// di transaksi pemanggil (KEPUTUSAN-114), lalu mengembalikan event E-13 per butir untuk ditulis pustaka
/// audit di transaksi yang sama (ADR-009 butir 2). Pemanggil lebih dulu memeriksa pilihan dengan
/// [`super::domain::check_selection`]. Waktu bukti = jam server database (trigger
/// `tg_consent_record_server_time`).
pub async fn record_granted_consent(
    conn: &mut PgConnection,
    subject: ConsentSubject,
    version_id: Uuid,
    items: &[ConsentItem],
    ctx: &AuditContext,
    correlation_id: Uuid,
) -> Result<Vec<AuditEvent>, PrivacyError> {
    let audit_err = |_| PrivacyError::Internal("audit_event");
    // Semua butir wajib milik `version_id` dan aktif: butir asing ditolak sebelum ada yang ditulis.
    let wanted: Vec<Uuid> = items.iter().map(|i| i.id).collect();
    // Jenis teks harus cocok dengan subjek: teks HR hanya untuk akun HR, teks pasien hanya untuk pasien
    // (KEPUTUSAN-321; database tidak menegakkannya lintas tabel).
    let allowed: Vec<String> = subject
        .allowed_consent_types()
        .iter()
        .map(|t| t.as_str().to_string())
        .collect();
    let owned = repo::count_items_of_version(&mut *conn, version_id, &wanted, &allowed)
        .await
        .map_err(|_| PrivacyError::Unavailable)?;
    let distinct = wanted
        .iter()
        .collect::<std::collections::BTreeSet<_>>()
        .len();
    if owned != i64::try_from(distinct).unwrap_or(i64::MAX) {
        tracing::error!("consent_item_not_in_version");
        return Err(PrivacyError::Internal("consent_item_foreign"));
    }
    let mut events = Vec::with_capacity(items.len());
    for item in items {
        let id = new_id();
        let (patient_id, record_account, channel) = match subject {
            ConsentSubject::Patient { patient_id, .. } => (Some(patient_id), None, "patient_app"),
            ConsentSubject::CorporateHr { account_id, .. } => {
                (None, Some(account_id), "corporate_portal")
            }
        };
        repo::insert_record(
            &mut *conn,
            &NewConsentRecord {
                id,
                patient_id,
                account_id: record_account,
                consent_text_version_id: version_id,
                consent_item_id: item.id,
                action: "granted",
                channel,
                correlation_id,
            },
        )
        .await
        .map_err(|_| PrivacyError::Unavailable)?;
        let (actor_account, actor_type, object, subject_or_tenant) = match subject {
            ConsentSubject::Patient {
                account_id,
                owner_care_recipient_id,
                ..
            } => (
                account_id,
                "patient",
                &CONSENT_RECORD_OBJECT,
                SubjectOrTenant::Subject(Subject::care_recipient(owner_care_recipient_id)),
            ),
            ConsentSubject::CorporateHr {
                account_id,
                corporate_client_id,
            } => (
                account_id,
                "corporate_admin",
                &CORPORATE_HR_CONSENT_RECORD_OBJECT,
                SubjectOrTenant::Tenant(corporate_client_id),
            ),
        };
        let actor = Actor::account(
            actor_account,
            Code::new(actor_type).map_err(audit_err)?,
            Vec::new(),
        );
        let event = AuditEvent::new(
            EventCode::E13,
            Code::new("consent_granted").map_err(audit_err)?,
            actor,
            ObjectRef::with_uuid(object, id),
        )
        .change(FieldChange::new(
            "item_code",
            None,
            Some(Value::code(&item.item_code).map_err(audit_err)?),
        ))
        .change(FieldChange::new(
            "action",
            None,
            Some(Value::code("granted").map_err(audit_err)?),
        ))
        .outcome(Outcome::success())
        .context(ctx);
        events.push(match subject_or_tenant {
            SubjectOrTenant::Subject(s) => event.subject(s),
            SubjectOrTenant::Tenant(t) => event.tenant(t),
        });
    }
    Ok(events)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consent_record_audit_policy_is_valid() {
        assert!(CONSENT_RECORD_OBJECT.validate().is_ok());
    }
}
