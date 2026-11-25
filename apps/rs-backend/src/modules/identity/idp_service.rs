//! API internal pencatatan niat penyediaan identitas (ADR-030 butir 2, 5, 7; KEPUTUSAN-305). Dipanggil use
//! case di **transaksi pemanggil** (`audit::in_audited_tx`, bersama event audit use case itu): niat, status
//! `pending`, dan job ber-muatan ID niat saja ter-commit bersama perubahan bisnisnya, atau tidak sama sekali.
//! Tidak ada panggilan Keycloak di sini (KEPUTUSAN-125); efek Keycloak hanya dijalankan worker IdP.
//!
//! Pemakai yang direncanakan: pendaftaran mandiri HR (F1-T101), undangan HR (F1-T105, F1-T107 undangan),
//! penonaktifan dan pencabutan sesi HR. Worker sendiri memakai [`record_intent`] untuk niat lanjutan
//! (email aktivasi setelah pengguna dibuat, kompensasi setelah penyediaan gagal).

use super::domain::IdpRealm;
use super::idp_domain::{
    IdpJobKind, IntentPurpose, IntentRules, IntentStatus, PAYLOAD_INTENT_KEY, valid_combination,
};
use super::idp_repo::{
    self, IntentEventDetails, IntentStatusStore, NewIntent, NewPendingAccount, RepoError,
};
use foundation::{Clock, CorrelationId, Uuid, new_id};
use platform_jobs::{DedupeKey, JobPayload, JobType, MaxAttempts, NewJob, enqueue};
use sqlx::PgConnection;

/// Percobaan maksimum job IdP (backoff `platform_jobs::Backoff` pekerja). Angka belum ditetapkan dokumen
/// `[TIDAK DIKETAHUI]`; usulan teknis KEPUTUSAN-306 (Keycloak tidak tersedia beberapa menit tetap tertangani).
pub const IDP_JOB_MAX_ATTEMPTS: i16 = 8;

/// Tujuan kunci deduplikasi job IdP (`idp_intent:<id niat>`): satu job aktif per niat.
pub const IDP_JOB_DEDUPE_PURPOSE: &str = "idp_intent";

/// Galat pencatatan niat (tanpa isi baris atau email).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntentError {
    /// Kombinasi jenis/tujuan/realm atau niat asal tidak sah.
    InvalidIntent,
    /// Sudah ada akun belum terhapus dengan email yang sama di realm itu.
    EmailInUse,
    /// Database.
    Repo(RepoError),
    /// Antrean job.
    Queue,
}

impl std::fmt::Display for IntentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidIntent => f.write_str("niat penyediaan identitas tidak sah"),
            Self::EmailInUse => f.write_str("email sudah dipakai akun lain di realm ini"),
            Self::Repo(e) => write!(f, "{e}"),
            Self::Queue => f.write_str("job IdP gagal diantrekan"),
        }
    }
}

impl From<RepoError> for IntentError {
    fn from(e: RepoError) -> Self {
        Self::Repo(e)
    }
}

/// Pelaku pencatat niat (kolom `actor_type`/`actor_account_id` riwayat status).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IntentActor {
    /// `patient`, `health_worker`, `rs_admin`, `partner_admin`, `corporate_admin`, atau `system`.
    pub actor_type: &'static str,
    /// Akun pelaku (`None` untuk `system` atau pendaftar yang belum punya akun aktif).
    pub account_id: Option<Uuid>,
}

impl IntentActor {
    /// Proses sistem (worker).
    pub const SYSTEM: Self = Self {
        actor_type: "system",
        account_id: None,
    };
}

/// Permintaan niat.
#[derive(Debug, Clone, Copy)]
pub struct IntentRequest {
    /// Akun sasaran.
    pub account_id: Uuid,
    /// Realm akun.
    pub realm: IdpRealm,
    /// Operasi.
    pub kind: IdpJobKind,
    /// Tujuan.
    pub purpose: IntentPurpose,
    /// Niat `create_user` asal (wajib untuk aktivasi, kompensasi, rekonsiliasi).
    pub origin_intent_id: Option<Uuid>,
    /// Pelaku.
    pub actor: IntentActor,
    /// Korelasi permintaan pemicu (diteruskan ke audit hasil worker).
    pub correlation_id: CorrelationId,
}

/// Mencatat niat + status `pending` + job di transaksi pemanggil. Mengembalikan ID niat.
pub async fn record_intent(
    conn: &mut PgConnection,
    req: &IntentRequest,
    clock: &dyn Clock,
) -> Result<Uuid, IntentError> {
    if !valid_combination(req.kind, req.purpose, req.realm)
        || req.kind.needs_origin() != req.origin_intent_id.is_some()
    {
        return Err(IntentError::InvalidIntent);
    }
    let id = new_id();
    idp_repo::insert_intent(
        conn,
        &NewIntent {
            id,
            created_by: req.actor.account_id,
            account_id: req.account_id,
            realm: req.realm,
            kind: req.kind,
            purpose: req.purpose,
            origin_intent_id: req.origin_intent_id,
            correlation_id: req.correlation_id,
        },
    )
    .await?;
    let mut store = IntentStatusStore {
        conn: &mut *conn,
        intent_id: id,
        details: IntentEventDetails {
            event_id: new_id(),
            reason_code: None,
            actor_type: req.actor.actor_type,
            actor_account_id: req.actor.account_id,
            correlation_id: req.correlation_id.as_uuid(),
        },
    };
    status_flow::transition(&mut store, &IntentRules, IntentStatus::Pending)
        .await
        .map_err(|e| match e {
            status_flow::TransitionError::Store(r) => IntentError::Repo(r),
            _ => IntentError::Repo(RepoError::Corrupt),
        })?;
    enqueue_intent_job(conn, req.kind, id, clock).await?;
    Ok(id)
}

async fn enqueue_intent_job(
    conn: &mut PgConnection,
    kind: IdpJobKind,
    intent: Uuid,
    clock: &dyn Clock,
) -> Result<(), IntentError> {
    let job = NewJob::new(
        JobType::new(kind.job_type()).map_err(|_| IntentError::Queue)?,
        JobPayload::new()
            .with_id(PAYLOAD_INTENT_KEY, intent)
            .map_err(|_| IntentError::Queue)?,
        clock.now(),
        MaxAttempts::new(IDP_JOB_MAX_ATTEMPTS).map_err(|_| IntentError::Queue)?,
    )
    .dedupe_key(
        DedupeKey::for_entity(IDP_JOB_DEDUPE_PURPOSE, intent).map_err(|_| IntentError::Queue)?,
    );
    enqueue(&mut *conn, &job)
        .await
        .map_err(|_| IntentError::Queue)?;
    Ok(())
}

/// Akun baru yang menunggu penyediaan identitas.
#[derive(Debug, Clone, Copy)]
pub struct PendingAccountRequest<'a> {
    /// `account_type` (misalnya `corporate_admin`).
    pub account_type: &'a str,
    /// Realm.
    pub realm: IdpRealm,
    /// Email (disimpan ternormalisasi di `app.account`, tidak pernah di niat atau job).
    pub email: &'a str,
    /// Nama lengkap (opsional).
    pub full_name: Option<&'a str>,
    /// Tujuan (`hr_self_registration` atau `hr_invitation`).
    pub purpose: IntentPurpose,
    /// Pelaku.
    pub actor: IntentActor,
    /// Korelasi.
    pub correlation_id: CorrelationId,
}

/// Akun `pending_provisioning` beserta niat `create_user`-nya.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PendingAccount {
    /// ID akun.
    pub account_id: Uuid,
    /// ID niat `create_user` (nilai atribut `rs_intent_id` di Keycloak).
    pub intent_id: Uuid,
}

/// Kunci advisory transaksi per (realm, email ternormalisasi), periksa tidak ada akun belum terhapus dengan
/// email itu, sisipkan akun `pending_provisioning` tanpa `idp_subject`, lalu catat niat `create_user`
/// (ADR-030 butir 5). Semua di transaksi pemanggil.
pub async fn register_pending_account(
    conn: &mut PgConnection,
    req: &PendingAccountRequest<'_>,
    clock: &dyn Clock,
) -> Result<PendingAccount, IntentError> {
    if !valid_combination(IdpJobKind::CreateUser, req.purpose, req.realm) {
        return Err(IntentError::InvalidIntent);
    }
    idp_repo::lock_email(conn, req.realm, req.email).await?;
    if idp_repo::live_account_with_email(conn, req.realm, req.email).await? {
        return Err(IntentError::EmailInUse);
    }
    let account_id = new_id();
    idp_repo::insert_pending_account(
        conn,
        &NewPendingAccount {
            id: account_id,
            account_type: req.account_type,
            realm: req.realm,
            email: req.email,
            full_name: req.full_name,
            created_by: req.actor.account_id,
            now: clock.now(),
        },
    )
    .await?;
    let intent_id = record_intent(
        conn,
        &IntentRequest {
            account_id,
            realm: req.realm,
            kind: IdpJobKind::CreateUser,
            purpose: req.purpose,
            origin_intent_id: None,
            actor: req.actor,
            correlation_id: req.correlation_id,
        },
        clock,
    )
    .await?;
    Ok(PendingAccount {
        account_id,
        intent_id,
    })
}
