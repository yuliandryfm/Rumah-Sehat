//! Audit trail penolakan akses ke data sensitif (F1-T38, KU-03, PRD §3.1 dan §7.1 E-05, ADR-009 butir 2,
//! coding-standards §4.4 butir 3).
//!
//! [`AuditDenialRecorder`] adalah `authz::DenialAuditor` backend Ruang Sehat: setiap penolakan atas data
//! sensitif ditulis sebagai event E-05 dengan hasil gagal (`access_denied`). Event dibentuk dan
//! difinalisasi sekali per penolakan (`prepare`), lalu ditulis idempoten lewat `audit::insert_failed_access` di
//! transaksi tersendiri, tanpa perubahan bisnis. Dipanggil antrean penolakan `authz` di luar jalur
//! permintaan dan dicoba ulang di sana dengan event yang sama bila gagal sementara.
//!
//! Event wajib membawa alamat IP sumber (PRD §7.2, ditegakkan pustaka audit). Bila IP klien tidak
//! diketahui (middleware IP klien belum terpasang), event tidak dapat dibentuk: kegagalan permanen,
//! dicatat di application log oleh antrean penolakan.

use audit::{
    Actor, AuditContext, AuditEvent, Code, EventCode, FailedAccessRecord, ObjectPolicy, ObjectRef,
    Outcome, Source, SourceComponent, Subject, insert_failed_access,
};
use authz::{
    AccountType, DenialAuditError, DenialAuditFuture, DenialAuditor, DeniedAccess,
    PreparedDenialAudit,
};
use foundation::Clock;
use sqlx::PgPool;
use std::sync::Arc;

/// Objek penerima layanan pada event akses (tanpa nilai field: E-05 hanya mencatat akses).
static CARE_RECIPIENT_ACCESS: ObjectPolicy = ObjectPolicy::non_clinical("care_recipient", &[]);

/// Kebijakan objek audit untuk jenis objek sensitif yang dikenal; jenis lain ditolak (permanen).
fn object_policy(object_type: &str) -> Option<&'static ObjectPolicy> {
    match object_type {
        "care_recipient" => Some(&CARE_RECIPIENT_ACCESS),
        _ => None,
    }
}

fn source_component(t: AccountType) -> SourceComponent {
    match t {
        AccountType::Patient => SourceComponent::PatientApp,
        AccountType::HealthWorker => SourceComponent::PractitionerApp,
        AccountType::RsAdmin => SourceComponent::Backoffice,
        AccountType::PartnerAdmin => SourceComponent::PartnerPortal,
    }
}

/// Membentuk event E-05 hasil gagal untuk satu penolakan atas data sensitif.
fn denied_event(d: &DeniedAccess) -> Result<AuditEvent, DenialAuditError> {
    let target = d.sensitive.ok_or(DenialAuditError::Permanent)?;
    let ip = d.client_ip.ok_or(DenialAuditError::Permanent)?;
    let policy = object_policy(target.object_type).ok_or(DenialAuditError::Permanent)?;
    let code = |v: &str| Code::new(v).map_err(|_| DenialAuditError::Permanent);
    let roles = d
        .sub_roles
        .iter()
        .map(|r| code(r.as_str()))
        .collect::<Result<Vec<_>, _>>()?;
    let actor = Actor::account(d.account_id, code(d.account_type.as_str())?, roles);
    let context = AuditContext::new(
        d.correlation_id,
        Source::component(source_component(d.account_type)).ip(ip),
    );
    Ok(AuditEvent::new(
        EventCode::E05,
        code("access")?,
        actor,
        ObjectRef::with_uuid(policy, target.object_id),
    )
    .subject(Subject::care_recipient(target.subject_care_recipient_id))
    .outcome(Outcome::failure(code("access_denied")?))
    .context(&context))
}

/// Penulis event E-05 hasil gagal ke `audit_outbox` database aplikasi.
pub struct AuditDenialRecorder {
    pool: PgPool,
    clock: Arc<dyn Clock>,
}

impl AuditDenialRecorder {
    /// `pool` = koneksi role aplikasi (`INSERT` di `audit_outbox`); `clock` = jam server (ADR-009 butir 6).
    pub fn new(pool: PgPool, clock: Arc<dyn Clock>) -> Self {
        Self { pool, clock }
    }
}

impl DenialAuditor for AuditDenialRecorder {
    /// Event dibentuk dan difinalisasi sekali (ID event dan waktu server tetap untuk semua percobaan).
    fn prepare(&self, denial: &DeniedAccess) -> Result<PreparedDenialAudit, DenialAuditError> {
        let record = denied_event(denial)?
            .finish_failed_access(self.clock.as_ref())
            .map_err(|_| DenialAuditError::Permanent)?;
        Ok(PreparedDenialAudit::new(record))
    }

    fn write<'a>(&'a self, prepared: &'a PreparedDenialAudit) -> DenialAuditFuture<'a> {
        Box::pin(async move {
            let record = prepared
                .get::<FailedAccessRecord>()
                .ok_or(DenialAuditError::Permanent)?;
            // Idempoten: percobaan ulang dengan ID yang sama tidak menggandakan event.
            insert_failed_access(&self.pool, record)
                .await
                .map_err(|_| DenialAuditError::Retryable)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use authz::SensitiveTarget;
    use foundation::{CorrelationId, Uuid};
    use std::net::{IpAddr, Ipv4Addr};

    fn denial() -> DeniedAccess {
        DeniedAccess {
            account_id: Uuid::from_u128(1),
            account_type: AccountType::Patient,
            sub_roles: vec![],
            route: "/patient/v1/care-recipients/{care_recipient_id}".into(),
            correlation_id: CorrelationId::default(),
            client_ip: Some(IpAddr::V4(Ipv4Addr::new(198, 51, 100, 9))),
            sensitive: Some(SensitiveTarget {
                object_type: "care_recipient",
                object_id: Uuid::from_u128(2),
                subject_care_recipient_id: Uuid::from_u128(2),
            }),
        }
    }

    #[test]
    fn incomplete_denials_are_permanent_failures() {
        assert!(denied_event(&denial()).is_ok());
        let mut no_ip = denial();
        no_ip.client_ip = None;
        assert_eq!(
            denied_event(&no_ip).err(),
            Some(DenialAuditError::Permanent)
        );
        let mut unknown = denial();
        unknown.sensitive = Some(SensitiveTarget {
            object_type: "objek_lain",
            object_id: Uuid::from_u128(3),
            subject_care_recipient_id: Uuid::from_u128(3),
        });
        assert_eq!(
            denied_event(&unknown).err(),
            Some(DenialAuditError::Permanent)
        );
        let mut not_sensitive = denial();
        not_sensitive.sensitive = None;
        assert_eq!(
            denied_event(&not_sensitive).err(),
            Some(DenialAuditError::Permanent)
        );
    }
}
