//! Worker IdP (ADR-030 butir 1-6, KEPUTUSAN-302, KEPUTUSAN-305): menjalankan job IdP dari `app.job` di proses
//! `worker`. Satu-satunya pemakai [`IdpUserAdmin`] (arch-check `idp-admin-client`).
//!
//! Alur satu job (tanpa transaksi terbuka selama panggilan Keycloak, KEPUTUSAN-125):
//! 1. `job_type` di luar [`IdpJobKind`] atau muatan selain ID niat → job `failed` tanpa panggilan Keycloak dan
//!    tanpa menyentuh niat. Job `idp_*` tak dikenal yang masih `queued` digagalkan di awal setiap putaran
//!    ([`platform_jobs::fail_unknown_types`]) karena `claim` hanya mengambil jenis yang dikenal.
//! 2. Niat dibaca; niat yang sudah berakhir → job selesai tanpa panggilan Keycloak (job diulang setelah hasil
//!    tercatat).
//! 3. Prasyarat diperiksa terhadap keadaan **Keycloak** (atribut `rs_intent_id`, kredensial, `enabled`), bukan
//!    hanya status di database aplikasi (ADR-030 butir 3).
//! 4. Hasil (status niat, perubahan akun, niat lanjutan, event audit E-10) ditulis dalam SATU transaksi
//!    `audit::in_audited_tx` bersama `platform_jobs::complete`/`fail`. Sewa hilang → transaksi dibatalkan
//!    seluruhnya, sehingga hasil dan audit tidak pernah tercatat dua kali (ADR-030 butir 6).
//!
//! Idempotensi `idp_create_user`: sebelum `POST users`, worker mencari pengguna beratribut `rs_intent_id` = ID
//! niat; bila ada (worker mati setelah `POST` berhasil, atau sewa hilang), pengguna itu diadopsi, tidak dibuat
//! ulang. 409 tanpa pengguna beratribut cocok = email sudah dipakai: niat `failed`, akun dihapus lunak.
//!
//! Kegagalan akhir `idp_create_user` (percobaan habis, prasyarat ditolak): akun `pending_provisioning` dihapus
//! lunak dengan alasan penyediaan gagal dan niat `idp_delete_orphan` dicatat (ADR-030 butir 5); pengguna
//! Keycloak hanya dihapus bila atributnya sama persis dan tanpa kredensial.
//!
//! Belum di sini (ADR-030 langkah 7-8, task lanjutan): pengambil admin event dan pembanding pemanggil, serta
//! rekonsiliasi terjadwal niat yang tertahan `pending` atau job yang digagalkan penyapu sewa.

use super::domain::IdpRealm;
use super::idp_domain::{
    IdpJobKind, IntentPurpose, IntentRules, IntentStatus, Refusal, activation_plan,
    intent_id_from_payload, may_delete_orphan, may_send_activation, valid_combination,
};
use super::idp_repo::{self, IntentEventDetails, IntentRow, IntentStatusStore, RepoError};
use super::idp_service::{IntentActor, IntentError, IntentRequest, record_intent};
use super::idp_worker_client::{CreateOutcome, IdpUserAdmin};
use super::keycloak::KeycloakError;
use audit::{
    Actor, AuditContext, AuditEvent, AuditedTxError, Code, EventCode, FieldChange, FieldRule,
    ObjectPolicy, ObjectRef, Outcome, Source, SourceComponent, Value, in_audited_tx,
};
use foundation::{Clock, CorrelationId, Uuid, new_id};
use platform_jobs::{
    BatchReport, ClaimedJob, ErrorCode, JobError, JobOutcome, JobType, Lease, LeaseOutcome,
    WorkerConfig, complete, fail, fail_unknown_types, run_batch, sweep_expired,
};
use sqlx::{PgConnection, PgPool};
use std::sync::Arc;
use std::time::Duration;

/// Alasan hapus lunak akun yang penyediaannya gagal (`account.delete_reason`, ADR-030 butir 5).
pub const PROVISIONING_FAILED_REASON: &str = "idp_provisioning_failed";

/// Kebijakan audit objek akun untuk hasil worker IdP (E-10). Email, nama, dan ID pengguna Keycloak tidak dicatat
/// nilainya.
static IDP_ACCOUNT_POLICY: ObjectPolicy = ObjectPolicy::non_clinical(
    "account",
    &[
        FieldRule::value("status"),
        FieldRule::name_only("idp_subject"),
        FieldRule::value("idp_intent_id"),
        FieldRule::value("idp_operation"),
        FieldRule::name_only("activation_email"),
        FieldRule::value("idp_enabled"),
        FieldRule::name_only("idp_sessions"),
        FieldRule::name_only("idp_user"),
        FieldRule::value("idp_user_found"),
        FieldRule::name_only("delete_reason"),
    ],
);

/// Hasil akhir yang akan ditulis.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Finish {
    /// Efek Keycloak terjadi (atau sudah ada). `adopt` = ID pengguna yang diadopsi ke `idp_subject`.
    Succeeded {
        adopt: Option<String>,
        found: Option<bool>,
    },
    /// Prasyarat ditolak.
    Rejected(Refusal),
    /// Gagal permanen.
    Failed(Refusal),
}

/// Galat di dalam transaksi hasil.
#[derive(Debug)]
enum Work {
    LeaseLost,
    Db,
    Audit,
}

impl From<RepoError> for Work {
    fn from(_: RepoError) -> Self {
        Self::Db
    }
}

impl From<IntentError> for Work {
    fn from(_: IntentError) -> Self {
        Self::Db
    }
}

impl From<JobError> for Work {
    fn from(_: JobError) -> Self {
        Self::Db
    }
}

/// Niat yang sudah divalidasi.
#[derive(Debug, Clone)]
struct Intent {
    id: Uuid,
    account_id: Uuid,
    realm: IdpRealm,
    kind: IdpJobKind,
    purpose: IntentPurpose,
    origin: Option<Uuid>,
    correlation: CorrelationId,
    account_status: String,
    account_deleted: bool,
    idp_subject: Option<String>,
}

/// Worker IdP.
pub struct IdpWorker {
    pool: PgPool,
    clock: Arc<dyn Clock>,
    admin: Arc<IdpUserAdmin>,
    cfg: WorkerConfig,
    known: Vec<JobType>,
}

impl std::fmt::Debug for IdpWorker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IdpWorker")
            .field("admin", &self.admin)
            .finish_non_exhaustive()
    }
}

/// Semua `job_type` IdP.
pub fn idp_job_types() -> Result<Vec<JobType>, JobError> {
    IdpJobKind::ALL
        .into_iter()
        .map(|k| JobType::new(k.job_type()))
        .collect()
}

fn code(s: &str) -> ErrorCode {
    // Semua kode di modul ini konstanta yang lolos `ErrorCode::new` (diuji `refusal_codes_are_job_error_codes`).
    ErrorCode::or_invalid(s)
}

fn transient_code(e: KeycloakError) -> &'static str {
    match e {
        KeycloakError::Token(_) => "keycloak_auth",
        KeycloakError::Json | KeycloakError::Body => "keycloak_invalid_response",
        _ => "keycloak_unavailable",
    }
}

/// Konfigurasi worker IdP tidak sah (jenis job tidak tepat daftar IdP).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidIdpWorkerConfig;

impl IdpWorker {
    /// `cfg.job_types()` wajib tepat semua jenis IdP (tidak lebih, tidak kurang).
    pub fn new(
        pool: PgPool,
        clock: Arc<dyn Clock>,
        admin: Arc<IdpUserAdmin>,
        cfg: WorkerConfig,
    ) -> Result<Self, InvalidIdpWorkerConfig> {
        let known = idp_job_types().map_err(|_| InvalidIdpWorkerConfig)?;
        let mut a: Vec<&str> = cfg.job_types().iter().map(JobType::as_str).collect();
        let mut b: Vec<&str> = known.iter().map(JobType::as_str).collect();
        a.sort_unstable();
        b.sort_unstable();
        if a != b {
            return Err(InvalidIdpWorkerConfig);
        }
        Ok(Self {
            pool,
            clock,
            admin,
            cfg,
            known,
        })
    }

    /// Satu putaran: gagalkan job `idp_*` tak dikenal, sapu sewa kedaluwarsa, lalu jalankan satu batch.
    pub async fn round(&self) -> Result<BatchReport, JobError> {
        let clock = self.clock.as_ref();
        let refused = fail_unknown_types(
            &self.pool,
            IdpJobKind::JOB_TYPE_PREFIX,
            &self.known,
            &code(Refusal::UnknownJobType.code()),
            clock,
        )
        .await?;
        for (id, _) in &refused {
            tracing::warn!(job_id = %id, reason_code = Refusal::UnknownJobType.code(), "idp_job_refused");
        }
        sweep_expired(&self.pool, &self.known, clock).await?;
        run_batch(&self.pool, &self.cfg, clock, |job| self.handle(job)).await
    }

    /// Menjalankan satu job yang sudah diambil (dipakai [`Self::round`] dan test).
    pub async fn handle(&self, job: ClaimedJob) -> JobOutcome {
        let lease = job.lease().clone();
        let last_attempt = job.attempt() >= job.max_attempts();
        let Some(kind) = IdpJobKind::from_job_type(job.job_type().as_str()) else {
            return refuse_job(Refusal::UnknownJobType);
        };
        let Ok(payload) = job.payload() else {
            return refuse_job(Refusal::InvalidPayload);
        };
        let intent_id = match intent_id_from_payload(&payload) {
            Ok(id) => id,
            Err(r) => return refuse_job(r),
        };
        let row = {
            let mut conn = match self.pool.acquire().await {
                Ok(c) => c,
                Err(_) => return JobOutcome::Retry(code("database")),
            };
            match idp_repo::load_intent(&mut conn, intent_id).await {
                Ok(Some(r)) => r,
                Ok(None) => return refuse_job(Refusal::IntentNotFound),
                Err(_) => return JobOutcome::Retry(code("database")),
            }
        };
        match row.status.as_deref().map(IntentStatus::parse) {
            Some(Some(IntentStatus::Pending)) => {}
            // Hasil sudah tercatat: selesaikan job tanpa panggilan Keycloak.
            Some(Some(_)) => return JobOutcome::Succeeded,
            _ => return refuse_job(Refusal::InvalidIntent),
        }
        if IdpJobKind::from_operation(&row.operation) != Some(kind) {
            // Job tidak sesuai niatnya: job ditolak, niat (dan job miliknya yang sah) tidak disentuh.
            return refuse_job(Refusal::KindMismatch);
        }
        let intent = match validate(row, kind) {
            Ok(i) => i,
            Err((intent, r)) => {
                return match intent {
                    Some(i) => self.finish(&i, &lease, Finish::Rejected(r)).await,
                    None => refuse_job(r),
                };
            }
        };
        if !self.admin.serves(intent.realm) {
            return self
                .transient(&intent, &lease, last_attempt, "realm_not_configured")
                .await;
        }
        let decision = match intent.kind {
            IdpJobKind::CreateUser => self.create_user(&intent).await,
            IdpJobKind::SendActivation => self.send_activation(&intent).await,
            IdpJobKind::DisableUser => self.disable_user(&intent).await,
            IdpJobKind::LogoutUser => self.logout_user(&intent).await,
            IdpJobKind::DeleteOrphan => self.delete_orphan(&intent).await,
            IdpJobKind::Reconcile => self.reconcile(&intent).await,
        };
        match decision {
            Ok(finish) => self.finish(&intent, &lease, finish).await,
            Err(c) => self.transient(&intent, &lease, last_attempt, c).await,
        }
    }

    async fn origin_status(
        &self,
        origin: Uuid,
    ) -> Result<Option<(String, Option<String>)>, &'static str> {
        let mut conn = self.pool.acquire().await.map_err(|_| "database")?;
        idp_repo::origin_state(&mut conn, origin)
            .await
            .map_err(|_| "database")
    }

    async fn account_email(&self, account: Uuid) -> Result<Option<String>, &'static str> {
        let mut conn = self.pool.acquire().await.map_err(|_| "database")?;
        idp_repo::account_email(&mut conn, account)
            .await
            .map_err(|_| "database")
    }

    async fn create_user(&self, i: &Intent) -> Result<Finish, &'static str> {
        if i.account_deleted
            || i.account_status != "pending_provisioning"
            || i.idp_subject.is_some()
        {
            return Ok(Finish::Rejected(Refusal::AccountState));
        }
        let clock = self.clock.as_ref();
        // Adopsi dulu: pengguna dari percobaan sebelumnya yang sudah dibuat tidak pernah dibuat ulang.
        if let Some(id) = self
            .admin
            .find_by_intent(i.realm, i.id, clock)
            .await
            .map_err(transient_code)?
        {
            return Ok(Finish::Succeeded {
                adopt: Some(id),
                found: None,
            });
        }
        let Some(email) = self.account_email(i.account_id).await? else {
            return Ok(Finish::Rejected(Refusal::AccountState));
        };
        match self
            .admin
            .create_user(i.realm, &email, i.id, clock)
            .await
            .map_err(transient_code)?
        {
            CreateOutcome::Created(id) => Ok(Finish::Succeeded {
                adopt: Some(id),
                found: None,
            }),
            CreateOutcome::Conflict => {
                // Balapan dengan percobaan lain niat yang sama: cari lagi menurut atribut.
                match self
                    .admin
                    .find_by_intent(i.realm, i.id, clock)
                    .await
                    .map_err(transient_code)?
                {
                    Some(id) => Ok(Finish::Succeeded {
                        adopt: Some(id),
                        found: None,
                    }),
                    None => Ok(Finish::Failed(Refusal::EmailTaken)),
                }
            }
        }
    }

    async fn send_activation(&self, i: &Intent) -> Result<Finish, &'static str> {
        let Some(origin) = i.origin else {
            return Ok(Finish::Rejected(Refusal::OriginState));
        };
        match self.origin_status(origin).await? {
            Some((op, Some(st))) if op == "create_user" && st == "succeeded" => {}
            _ => return Ok(Finish::Rejected(Refusal::OriginState)),
        }
        if i.account_deleted || i.account_status != "pending_activation" {
            return Ok(Finish::Rejected(Refusal::AccountState));
        }
        let Some(subject) = i.idp_subject.as_deref() else {
            return Ok(Finish::Rejected(Refusal::AccountState));
        };
        let Some(plan) = activation_plan(i.realm, i.purpose) else {
            return Ok(Finish::Rejected(Refusal::InvalidIntent));
        };
        let clock = self.clock.as_ref();
        let Some(facts) = self
            .admin
            .user_facts(i.realm, subject, clock)
            .await
            .map_err(transient_code)?
        else {
            return Ok(Finish::Rejected(Refusal::IdpUserMissing));
        };
        if let Err(r) = may_send_activation(&facts, origin) {
            return Ok(Finish::Rejected(r));
        }
        match self
            .admin
            .send_actions_email(i.realm, subject, plan, clock)
            .await
        {
            Ok(()) => Ok(Finish::Succeeded {
                adopt: None,
                found: None,
            }),
            Err(KeycloakError::NotFound) => Ok(Finish::Rejected(Refusal::IdpUserMissing)),
            Err(e) => Err(transient_code(e)),
        }
    }

    async fn disable_user(&self, i: &Intent) -> Result<Finish, &'static str> {
        let Some(subject) = i.idp_subject.as_deref() else {
            return Ok(Finish::Rejected(Refusal::AccountState));
        };
        let clock = self.clock.as_ref();
        match self.admin.disable(i.realm, subject, clock).await {
            Ok(()) | Err(KeycloakError::NotFound) => {}
            Err(e) => return Err(transient_code(e)),
        }
        match self.admin.logout(i.realm, subject, clock).await {
            Ok(()) | Err(KeycloakError::NotFound) => Ok(Finish::Succeeded {
                adopt: None,
                found: None,
            }),
            Err(e) => Err(transient_code(e)),
        }
    }

    async fn logout_user(&self, i: &Intent) -> Result<Finish, &'static str> {
        let Some(subject) = i.idp_subject.as_deref() else {
            return Ok(Finish::Rejected(Refusal::AccountState));
        };
        match self
            .admin
            .logout(i.realm, subject, self.clock.as_ref())
            .await
        {
            Ok(()) | Err(KeycloakError::NotFound) => Ok(Finish::Succeeded {
                adopt: None,
                found: None,
            }),
            Err(e) => Err(transient_code(e)),
        }
    }

    async fn delete_orphan(&self, i: &Intent) -> Result<Finish, &'static str> {
        let Some(origin) = i.origin else {
            return Ok(Finish::Rejected(Refusal::OriginState));
        };
        match self.origin_status(origin).await? {
            Some((op, Some(st))) if op == "create_user" && (st == "failed" || st == "rejected") => {
            }
            _ => return Ok(Finish::Rejected(Refusal::OriginState)),
        }
        let clock = self.clock.as_ref();
        // Sasaran dicari menurut atribut niat asal, tidak pernah dari `idp_subject` atau muatan job.
        let Some(user) = self
            .admin
            .find_by_intent(i.realm, origin, clock)
            .await
            .map_err(transient_code)?
        else {
            return Ok(Finish::Succeeded {
                adopt: None,
                found: Some(false),
            });
        };
        let Some(facts) = self
            .admin
            .user_facts(i.realm, &user, clock)
            .await
            .map_err(transient_code)?
        else {
            return Ok(Finish::Succeeded {
                adopt: None,
                found: Some(false),
            });
        };
        if let Err(r) = may_delete_orphan(&facts, origin) {
            return Ok(Finish::Rejected(r));
        }
        self.admin
            .delete(i.realm, &user, clock)
            .await
            .map_err(transient_code)?;
        Ok(Finish::Succeeded {
            adopt: None,
            found: Some(true),
        })
    }

    async fn reconcile(&self, i: &Intent) -> Result<Finish, &'static str> {
        let Some(origin) = i.origin else {
            return Ok(Finish::Rejected(Refusal::OriginState));
        };
        match self.origin_status(origin).await? {
            Some((op, _)) if op == "create_user" => {}
            _ => return Ok(Finish::Rejected(Refusal::OriginState)),
        }
        let found = self
            .admin
            .find_by_intent(i.realm, origin, self.clock.as_ref())
            .await
            .map_err(transient_code)?;
        let adoptable = i.account_status == "pending_provisioning"
            && !i.account_deleted
            && i.idp_subject.is_none();
        Ok(Finish::Succeeded {
            found: Some(found.is_some()),
            adopt: found.filter(|_| adoptable),
        })
    }

    /// Gagal sementara: dicoba ulang, atau hasil akhir `failed` bila percobaan habis.
    async fn transient(
        &self,
        i: &Intent,
        lease: &Lease,
        last_attempt: bool,
        error: &'static str,
    ) -> JobOutcome {
        if last_attempt {
            tracing::warn!(intent_id = %i.id, error_code = error, "idp_job_retries_exhausted");
            self.finish(i, lease, Finish::Failed(Refusal::RetriesExhausted))
                .await
        } else {
            JobOutcome::Retry(code(error))
        }
    }

    /// Menulis hasil dalam satu transaksi bersama penandaan job.
    async fn finish(&self, i: &Intent, lease: &Lease, finish: Finish) -> JobOutcome {
        if let Finish::Rejected(r) | Finish::Failed(r) = &finish {
            tracing::warn!(
                intent_id = %i.id,
                job_type = i.kind.job_type(),
                reason_code = r.code(),
                security_relevant = r.is_security_relevant(),
                "idp_intent_refused"
            );
        }
        let intent = i.clone();
        let tx_lease = lease.clone();
        let clock = Arc::clone(&self.clock);
        let result = in_audited_tx(&self.pool, self.clock.as_ref(), move |conn| {
            Box::pin(
                async move { write_result(conn, &intent, &tx_lease, finish, clock.as_ref()).await },
            )
        })
        .await;
        match result {
            Ok(Some(c)) => JobOutcome::FailedInTx(LeaseOutcome::Applied, c),
            Ok(None) => JobOutcome::Completed(LeaseOutcome::Applied),
            Err(AuditedTxError::Work(Work::LeaseLost)) => {
                JobOutcome::Completed(LeaseOutcome::LeaseLost)
            }
            Err(_) => JobOutcome::Retry(code("result_write")),
        }
    }
}

fn refuse_job(r: Refusal) -> JobOutcome {
    tracing::warn!(reason_code = r.code(), "idp_job_refused");
    JobOutcome::Fail(code(r.code()))
}

fn validate(row: IntentRow, kind: IdpJobKind) -> Result<Intent, (Option<Box<Intent>>, Refusal)> {
    let realm: Option<IdpRealm> = row.realm.parse().ok();
    let purpose = IntentPurpose::parse(&row.purpose);
    let correlation = CorrelationId::parse(&row.correlation_id.hyphenated().to_string()).ok();
    let (Some(realm), Some(purpose), Some(correlation)) = (realm, purpose, correlation) else {
        return Err((None, Refusal::InvalidIntent));
    };
    let intent = Intent {
        id: row.id,
        account_id: row.account_id,
        realm,
        kind,
        purpose,
        origin: row.origin_intent_id,
        correlation,
        account_status: row.account_status,
        account_deleted: row.account_deleted,
        idp_subject: row.idp_subject,
    };
    if !valid_combination(kind, purpose, realm) || kind.needs_origin() != intent.origin.is_some() {
        return Err((Some(Box::new(intent)), Refusal::InvalidIntent));
    }
    Ok(intent)
}

fn audit_code(s: &str) -> Result<Code, Work> {
    Code::new(s).map_err(|_| Work::Audit)
}

fn base_event(i: &Intent, action: &str, outcome: Outcome) -> Result<AuditEvent, Work> {
    let ctx = AuditContext::new(i.correlation, Source::component(SourceComponent::Worker));
    Ok(AuditEvent::new(
        EventCode::E10,
        audit_code(action)?,
        Actor::system(),
        ObjectRef::with_uuid(&IDP_ACCOUNT_POLICY, i.account_id),
    )
    .outcome(outcome)
    .context(&ctx)
    .change(FieldChange::new(
        "idp_intent_id",
        None,
        Some(Value::Uuid(i.id)),
    ))
    .change(FieldChange::new(
        "idp_operation",
        None,
        Some(Value::Code(audit_code(i.kind.operation())?)),
    )))
}

fn status_value(s: &str) -> Result<Value, Work> {
    Ok(Value::Code(audit_code(s)?))
}

async fn transition(
    conn: &mut PgConnection,
    i: &Intent,
    to: IntentStatus,
    reason: Option<&'static str>,
) -> Result<(), Work> {
    let mut store = IntentStatusStore {
        conn,
        intent_id: i.id,
        details: IntentEventDetails {
            event_id: new_id(),
            reason_code: reason,
            actor_type: "system",
            actor_account_id: None,
            correlation_id: i.correlation.as_uuid(),
        },
    };
    status_flow::transition(&mut store, &IntentRules, to)
        .await
        .map_err(|_| Work::Db)?;
    Ok(())
}

/// Isi transaksi hasil. `Ok(Some(code))` = job ditandai `failed` dengan kode itu.
async fn write_result(
    conn: &mut PgConnection,
    i: &Intent,
    lease: &Lease,
    finish: Finish,
    clock: &dyn Clock,
) -> Result<(Option<ErrorCode>, Vec<AuditEvent>), Work> {
    let mut finish = finish;
    // Adopsi hanya bila akun masih menunggu penyediaan (dikunci baris; keadaan bisa berubah sejak dibaca).
    if let Finish::Succeeded {
        adopt: Some(subject),
        ..
    } = &finish
    {
        let adopted = idp_repo::adopt_subject(conn, i.account_id, subject, clock.now()).await?;
        if !adopted {
            if i.kind == IdpJobKind::CreateUser {
                finish = Finish::Rejected(Refusal::AccountState);
            } else {
                // Rekonsiliasi: akun sudah tidak menunggu; tidak ada yang diadopsi.
                finish = Finish::Succeeded {
                    adopt: None,
                    found: Some(true),
                };
            }
        }
    }
    let mut events = Vec::new();
    let rejected = matches!(finish, Finish::Rejected(_));
    let failed = match finish {
        Finish::Succeeded { adopt, found } => {
            transition(conn, i, IntentStatus::Succeeded, None).await?;
            let mut e = base_event(i, success_action(i.kind), Outcome::success())?;
            if adopt.is_some() {
                e = e
                    .change(FieldChange::new(
                        "status",
                        Some(status_value("pending_provisioning")?),
                        Some(status_value("pending_activation")?),
                    ))
                    .change(FieldChange::changed("idp_subject"));
                if matches!(
                    i.purpose,
                    IntentPurpose::HrSelfRegistration | IntentPurpose::HrInvitation
                ) {
                    record_intent(
                        conn,
                        &IntentRequest {
                            account_id: i.account_id,
                            realm: i.realm,
                            kind: IdpJobKind::SendActivation,
                            purpose: i.purpose,
                            origin_intent_id: Some(origin_of(i)),
                            actor: IntentActor::SYSTEM,
                            correlation_id: i.correlation,
                        },
                        clock,
                    )
                    .await?;
                }
            }
            e = match i.kind {
                IdpJobKind::SendActivation => e.change(FieldChange::changed("activation_email")),
                IdpJobKind::DisableUser => e
                    .change(FieldChange::new(
                        "idp_enabled",
                        None,
                        Some(Value::Bool(false)),
                    ))
                    .change(FieldChange::changed("idp_sessions")),
                IdpJobKind::LogoutUser => e.change(FieldChange::changed("idp_sessions")),
                IdpJobKind::DeleteOrphan => e
                    .change(FieldChange::new(
                        "idp_user_found",
                        None,
                        Some(Value::Bool(found.unwrap_or(false))),
                    ))
                    .change(FieldChange::changed("idp_user")),
                IdpJobKind::Reconcile => e.change(FieldChange::new(
                    "idp_user_found",
                    None,
                    Some(Value::Bool(found.unwrap_or(false))),
                )),
                IdpJobKind::CreateUser => e,
            };
            events.push(e);
            None
        }
        Finish::Rejected(r) | Finish::Failed(r) => {
            let to = if rejected {
                IntentStatus::Rejected
            } else {
                IntentStatus::Failed
            };
            transition(conn, i, to, Some(r.code())).await?;
            let mut e = base_event(
                i,
                failure_action(i.kind),
                Outcome::failure(audit_code(r.code())?),
            )?;
            if i.kind == IdpJobKind::CreateUser {
                // ADR-030 butir 5: akun tidak pernah tertinggal `pending_provisioning`; email bebas lagi.
                if idp_repo::abandon_pending_account(conn, i.account_id, PROVISIONING_FAILED_REASON)
                    .await?
                {
                    e = e.change(FieldChange::changed("delete_reason"));
                }
                if r != Refusal::EmailTaken {
                    record_intent(
                        conn,
                        &IntentRequest {
                            account_id: i.account_id,
                            realm: i.realm,
                            kind: IdpJobKind::DeleteOrphan,
                            purpose: IntentPurpose::ProvisioningCompensation,
                            origin_intent_id: Some(i.id),
                            actor: IntentActor::SYSTEM,
                            correlation_id: i.correlation,
                        },
                        clock,
                    )
                    .await?;
                }
            }
            events.push(e);
            Some(code(r.code()))
        }
    };
    let outcome = match &failed {
        None => complete(&mut *conn, lease, clock).await?,
        Some(c) => fail(&mut *conn, lease, c, clock).await?,
    };
    if outcome == LeaseOutcome::LeaseLost {
        return Err(Work::LeaseLost);
    }
    Ok((failed, events))
}

/// Niat `create_user` pembuat pengguna: niat ini sendiri (create) atau asalnya (rekonsiliasi).
fn origin_of(i: &Intent) -> Uuid {
    i.origin.unwrap_or(i.id)
}

fn success_action(kind: IdpJobKind) -> &'static str {
    match kind {
        IdpJobKind::CreateUser => "idp_user_provisioned",
        IdpJobKind::SendActivation => "idp_activation_email_sent",
        IdpJobKind::DisableUser => "idp_user_disabled",
        IdpJobKind::LogoutUser => "idp_sessions_ended",
        IdpJobKind::DeleteOrphan => "idp_orphan_deleted",
        IdpJobKind::Reconcile => "idp_user_reconciled",
    }
}

fn failure_action(kind: IdpJobKind) -> &'static str {
    match kind {
        IdpJobKind::CreateUser => "idp_user_provisioning_failed",
        IdpJobKind::SendActivation => "idp_activation_email_refused",
        IdpJobKind::DisableUser => "idp_user_disable_failed",
        IdpJobKind::LogoutUser => "idp_sessions_end_failed",
        IdpJobKind::DeleteOrphan => "idp_orphan_delete_refused",
        IdpJobKind::Reconcile => "idp_user_reconcile_failed",
    }
}

/// Perulangan proses `worker`: satu putaran tiap `poll_interval`; galat putaran dicatat lalu dicoba lagi.
pub async fn run_idp_worker(worker: IdpWorker, poll_interval: Duration) {
    loop {
        match worker.round().await {
            Ok(r) if r.claimed > 0 => {
                tracing::info!(
                    claimed = r.claimed,
                    succeeded = r.succeeded,
                    requeued = r.requeued,
                    failed = r.failed,
                    lease_lost = r.lease_lost,
                    "idp_worker_round"
                );
            }
            Ok(_) => {}
            Err(e) => tracing::warn!(error = %e, "idp_worker_round_failed"),
        }
        tokio::time::sleep(poll_interval).await;
    }
}
