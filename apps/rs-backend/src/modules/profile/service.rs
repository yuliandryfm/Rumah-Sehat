//! Use case pengajuan identitas pasien (F1-T60, reference implementation US-106; pola untuk F1-T66).
//!
//! Satu jalur per lapis: handler (F1-T61) → fungsi di sini → `repo` (tabel A2) / `api` modul lain
//! (`platform` untuk berkas) / crate bersama (`authz`, `audit`, `crypto`, `status-flow`). Setiap use case
//! yang mengubah data berjalan di **satu** `audit::in_audited_tx` bersama event-nya (ADR-009 butir 2);
//! panggilan ke pihak luar (object storage, OpenBao Transit, Valkey) dilakukan **sebelum** transaksi dibuka
//! (KEPUTUSAN-125).
//!
//! Pengajuan ([`submit`]), urutan dan alasannya:
//! 1. NIK 16 digit (US-106 AC2) dan IP klien wajib ada (field "Sumber" audit PRD §7.2; tanpa IP tidak ada
//!    event yang dapat ditulis, jadi permintaan ditolak lebih awal: pola gagal-tertutup TD-22).
//! 2. Relasi kepemilikan lewat `authz::care_recipient_access` (KU-03: milik orang lain = 404 identik,
//!    dicatat E-05 gagal karena dokumen identitas data sensitif).
//! 3. Penahanan batas NIK berlapis (US-106 AC4, KEPUTUSAN-192) dibaca dari Valkey; tertahan = pesan umum.
//! 4. Foto divalidasi (modul `platform`, ADR-018) **sebelum** registri diperiksa, karena registri hanya
//!    diperiksa saat pengajuan lengkap (US-106 AC4).
//! 5. UP-40: registri NIK pemilik akun dicocokkan dengan blind index **semua** versi kunci aktif (TD-10).
//!    Bentrok = pengajuan tidak disimpan, status tidak berubah, pesan umum, penghitung batas dinaikkan.
//! 6. DEK wadah `identity_document` dibuat lewat Transit, NIK dienkripsi dengan AAD baris dokumen
//!    (KEPUTUSAN-130).
//! 7. Transaksi teraudit: kunci baris penerima layanan, foto terkunci bila masih menunggu (US-106 AC5),
//!    `stored_file` + `data_key` + `identity_document` + permintaan + transisi `pending` (jalur tulis
//!    ADR-008) + penunjuk permintaan efektif, dan event E-10. Gagal = semuanya batal dan berkas tervalidasi
//!    dibuang.
//!
//! Pembatalan ([`cancel`], US-106 AC5): transisi `pending → cancelled` lewat `status-flow` (keputusan
//! Verifikator yang tersimpan lebih dulu menang; yang kalah dijawab `409` dan dicatat hasil gagal), lalu
//! penunjuk permintaan efektif kembali ke permintaan sebelumnya yang tidak dibatalkan (temuan S-04).

use super::api::{IdentityServices, IdentityStatusView, SubmissionAccepted};
use super::domain::{
    ACCOUNT_REJECTIONS_BEFORE_HOLD, DocumentType, GLOBAL_COUNTER_TTL,
    GLOBAL_REJECTION_ALERT_PER_HOUR, IP_REJECTIONS_LIMIT, IdentityStatus, NIK_HOLD,
    NIK_REJECTION_WINDOW, NIK_REJECTIONS_BEFORE_HOLD, TriggerType, VerificationActor,
    VerificationRules, VerificationStatus, owner_nik_conflict,
};
use super::error::ProfileError;
use super::repo::{self, EventDetails, IvrStatusStore, RecipientNik, RepoError};
use crate::modules::platform::api::{self as files, FileContainer, UploadPurpose, ValidatedUpload};
use audit::{
    Actor, AuditContext, AuditEvent, AuditedTxError, Code, EventCode, FieldChange, FieldRule,
    ObjectPolicy, ObjectRef, Outcome, Source, SourceComponent, Subject, Value, in_audited_tx,
};
use authz::{AccountType, CareRecipientOwner, Principal, Sensitivity, care_recipient_access};
use crypto::{Container, ContainerType, CryptoError, EncryptionContext, Nik, insert_data_key};
use envelope::FieldError;
use foundation::{Clock, Sensitive, Uuid, new_id};
use observability::security;
use sqlx::PgPool;
use status_flow::TransitionError;
use std::convert::Infallible;
use std::net::IpAddr;
use std::sync::Arc;

/// Kebijakan audit objek permintaan verifikasi (daftar izin before/after, ADR-009 butir 7): status dan
/// alasan berupa kode, NIK hanya 4 digit terakhir (PRD §7.3), jenis dokumen.
pub(super) static IDENTITY_VERIFICATION_REQUEST_POLICY: ObjectPolicy = ObjectPolicy::non_clinical(
    "identity_verification_request",
    &[
        FieldRule::value("status"),
        FieldRule::value("reason_code"),
        FieldRule::last4("nik"),
        FieldRule::value("document_type"),
    ],
);

/// Percobaan menulis event hasil gagal di transaksi tersendiri (coding-standards §4.4 butir 3).
const FAILED_AUDIT_ATTEMPTS: usize = 3;

fn repo_error(e: RepoError) -> ProfileError {
    match e {
        RepoError::DuplicateNikInAccount => ProfileError::DuplicateNikInAccount,
        RepoError::RegistryConflict => ProfileError::NikUnavailable,
        RepoError::Corrupt => ProfileError::Internal("corrupt_row"),
        RepoError::Database => ProfileError::Database,
    }
}

fn crypto_error(e: CryptoError) -> ProfileError {
    match e {
        CryptoError::KmsUnavailable | CryptoError::KmsTimeout | CryptoError::Store => {
            ProfileError::Database
        }
        _ => ProfileError::Internal("crypto"),
    }
}

fn audited_error(e: AuditedTxError<ProfileError>) -> ProfileError {
    match e {
        AuditedTxError::Work(e) => e,
        AuditedTxError::Audit(_) => ProfileError::Internal("audit"),
        AuditedTxError::Database { .. } => ProfileError::Database,
    }
}

fn transition_error<E>(e: TransitionError<VerificationStatus, E>) -> ProfileError
where
    E: Into<ProfileError>,
{
    match e {
        TransitionError::NotAllowed { .. } | TransitionError::Contended => {
            ProfileError::TransitionConflict
        }
        TransitionError::ProjectionMismatch => ProfileError::ProjectionMismatch,
        TransitionError::VersionOverflow => ProfileError::Internal("version_overflow"),
        TransitionError::Store(e) => e.into(),
    }
}

impl From<RepoError> for ProfileError {
    fn from(e: RepoError) -> Self {
        repo_error(e)
    }
}

pub(super) fn source_component(t: AccountType) -> SourceComponent {
    match t {
        AccountType::Patient => SourceComponent::PatientApp,
        AccountType::HealthWorker => SourceComponent::PractitionerApp,
        AccountType::RsAdmin => SourceComponent::Backoffice,
        AccountType::PartnerAdmin => SourceComponent::PartnerPortal,
    }
}

/// IP klien wajib (TD-22, gagal-tertutup): tanpa IP, event audit aksi pengguna tidak dapat ditulis
/// (PRD §7.2), sehingga permintaan ditolak sebelum ada efek apa pun.
pub(super) fn client_ip(principal: &Principal, route: &str) -> Result<IpAddr, ProfileError> {
    principal.client_ip().ok_or_else(|| {
        tracing::error!(route = route, "client_ip_missing");
        ProfileError::Internal("client_ip_missing")
    })
}

/// Konteks audit dari pemanggil terotorisasi (field "Sumber" dan ID korelasi PRD §7.2).
pub(super) fn audit_context(principal: &Principal, ip: IpAddr) -> AuditContext {
    AuditContext::new(
        principal.correlation_id(),
        Source::component(source_component(principal.account_type())).ip(ip),
    )
}

pub(super) fn actor(principal: &Principal) -> Result<Actor, ProfileError> {
    let code = |v: &str| Code::new(v).map_err(|_| ProfileError::Internal("actor"));
    let roles = principal
        .sub_roles()
        .iter()
        .map(|r| code(r.as_str()))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Actor::account(
        principal.account_id(),
        code(principal.account_type().as_str())?,
        roles,
    ))
}

/// Event atas permintaan verifikasi. `subject` = penerima layanan pemilik dokumen (PRD §7.2).
pub(super) struct RequestEvent<'a> {
    pub(super) code: EventCode,
    pub(super) action: &'static str,
    pub(super) request_id: Uuid,
    pub(super) subject: Uuid,
    pub(super) changes: Vec<FieldChange>,
    pub(super) outcome: Outcome,
    pub(super) context: &'a AuditContext,
}

pub(super) fn request_event(
    principal: &Principal,
    e: RequestEvent<'_>,
) -> Result<AuditEvent, ProfileError> {
    let action = Code::new(e.action).map_err(|_| ProfileError::Internal("audit_action"))?;
    let mut event = AuditEvent::new(
        e.code,
        action,
        actor(principal)?,
        ObjectRef::with_uuid(&IDENTITY_VERIFICATION_REQUEST_POLICY, e.request_id),
    )
    .subject(Subject::care_recipient(e.subject))
    .outcome(e.outcome)
    .context(e.context);
    for c in e.changes {
        event = event.change(c);
    }
    Ok(event)
}

pub(super) fn status_change(
    from: Option<VerificationStatus>,
    to: VerificationStatus,
) -> Result<FieldChange, ProfileError> {
    let v = |s: VerificationStatus| {
        Value::code(s.as_str()).map_err(|_| ProfileError::Internal("audit_value"))
    };
    Ok(FieldChange::new(
        "status",
        from.map(v).transpose()?,
        Some(v(to)?),
    ))
}

/// Menulis event hasil gagal di transaksi tersendiri, dicoba ulang; kegagalan tidak mengubah jawaban
/// (coding-standards §4.4 butir 3).
pub(super) async fn record_failed(pool: &PgPool, clock: &dyn Clock, event: AuditEvent) {
    let mut event = Some(event);
    for attempt in 1..=FAILED_AUDIT_ATTEMPTS {
        let Some(ev) = event.take() else { return };
        let retry = ev.clone();
        let result = in_audited_tx(pool, clock, move |_conn| {
            Box::pin(async move { Ok::<_, Infallible>(((), vec![ev])) })
        })
        .await;
        match result {
            Ok(()) => return,
            Err(_) if attempt < FAILED_AUDIT_ATTEMPTS => event = Some(retry),
            Err(_) => tracing::error!("failed_outcome_audit_not_written"),
        }
    }
}

/// Kepemilikan penerima layanan oleh akun pasien pemanggil (ADR-012 butir 1 dan 4, KU-03).
pub(super) async fn owned_recipient(
    pool: &PgPool,
    principal: &Principal,
    route: &str,
    care_recipient_id: Uuid,
    sensitivity: Sensitivity,
) -> Result<authz::OwnedCareRecipient, ProfileError> {
    let owner = repo::care_recipient_owner(pool, care_recipient_id)
        .await
        .map_err(|_| ProfileError::Database)?
        .map(|r| CareRecipientOwner::new(care_recipient_id, r.account_id, r.is_account_owner));
    care_recipient_access(principal, route, care_recipient_id, owner, sensitivity)
        .map_err(|_| ProfileError::NotFound)
}

// ----------------------------------------------------------------------------------------------------
// Batas NIK berlapis (US-106 AC4, KEPUTUSAN-192) di Valkey: penghitung jendela tetap sejak penolakan
// pertama + kunci penahanan. Nilai NIK tidak pernah menjadi kunci: kunci memakai SHA-256 dari blind index
// versi terbaru.

struct LimitKeys {
    account_count: String,
    account_hold: String,
    nik_count: String,
    nik_hold: String,
    ip_count: String,
}

impl LimitKeys {
    fn new(account_id: Uuid, nik_bidx_current: &[u8], ip: IpAddr) -> Self {
        let nik: String = crypto::sha256(nik_bidx_current)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        let account = account_id.hyphenated();
        Self {
            account_count: format!("rs:profile:nik-limit:account:{account}:count"),
            account_hold: format!("rs:profile:nik-limit:account:{account}:hold"),
            nik_count: format!("rs:profile:nik-limit:nik:{nik}:count"),
            nik_hold: format!("rs:profile:nik-limit:nik:{nik}:hold"),
            ip_count: format!("rs:profile:nik-limit:ip:{ip}:count"),
        }
    }
}

fn global_key(clock: &dyn Clock) -> String {
    format!(
        "rs:profile:nik-limit:global:{}",
        clock.now().unix_timestamp().div_euclid(3600)
    )
}

fn as_count(v: Option<&Vec<u8>>) -> i64 {
    v.and_then(|b| std::str::from_utf8(b).ok())
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(0)
}

/// Apakah pengajuan NIK tertahan (per akun, per NIK lintas akun, atau per IP). Valkey tidak tersedia =
/// gagal-tertutup (`503`): kontrol anti-enumerasi tidak dilewati diam-diam.
async fn is_held(s: &IdentityServices, keys: &LimitKeys) -> Result<bool, ProfileError> {
    let values = s
        .valkey
        .get_many(&[
            keys.account_hold.as_str(),
            keys.nik_hold.as_str(),
            keys.ip_count.as_str(),
        ])
        .await
        .map_err(|_| ProfileError::Database)?;
    let held = values.first().is_some_and(Option::is_some)
        || values.get(1).is_some_and(Option::is_some)
        || as_count(values.get(2).and_then(Option::as_ref)) >= IP_REJECTIONS_LIMIT;
    Ok(held)
}

/// Mencatat satu penolakan `nik_unavailable`: penahanan dipasang tepat saat ambang tercapai dan dicatat
/// sebagai kejadian keamanan; ambang global memicu peringatan sekali per jam. Galat Valkey hanya dicatat
/// di log (jawaban ke pasien tetap penolakan yang sama).
async fn record_rejection(s: &IdentityServices, account_id: Uuid, keys: &LimitKeys) {
    let v = &s.valkey;
    let mut failed = false;
    match v
        .incr_with_ttl(&keys.account_count, NIK_REJECTION_WINDOW)
        .await
    {
        Ok(n) if n == ACCOUNT_REJECTIONS_BEFORE_HOLD => {
            failed |= v.set(&keys.account_hold, b"1", NIK_HOLD).await.is_err();
            security::identity_submission_held(Some(account_id), "account");
        }
        Ok(_) => {}
        Err(_) => failed = true,
    }
    match v.incr_with_ttl(&keys.nik_count, NIK_REJECTION_WINDOW).await {
        Ok(n) if n == NIK_REJECTIONS_BEFORE_HOLD => {
            failed |= v.set(&keys.nik_hold, b"1", NIK_HOLD).await.is_err();
            security::identity_submission_held(Some(account_id), "identity_number");
        }
        Ok(_) => {}
        Err(_) => failed = true,
    }
    match v.incr_with_ttl(&keys.ip_count, NIK_REJECTION_WINDOW).await {
        Ok(n) if n == IP_REJECTIONS_LIMIT => {
            security::identity_submission_held(Some(account_id), "ip");
        }
        Ok(_) => {}
        Err(_) => failed = true,
    }
    match v
        .incr_with_ttl(&global_key(s.clock.as_ref()), GLOBAL_COUNTER_TTL)
        .await
    {
        Ok(n) if n == GLOBAL_REJECTION_ALERT_PER_HOUR + 1 => {
            security::identity_rejection_surge(n.unsigned_abs());
        }
        Ok(_) => {}
        Err(_) => failed = true,
    }
    if failed {
        tracing::error!("identity_rejection_counter_failed");
    }
}

// ----------------------------------------------------------------------------------------------------

async fn discard(s: &IdentityServices, upload: Arc<ValidatedUpload>) {
    match Arc::try_unwrap(upload) {
        Ok(u) => {
            if files::discard_validated(s.clock.as_ref(), &s.files, u)
                .await
                .is_err()
            {
                // Objek tetap diklaim slot dan dibersihkan job pembersih (TD-17).
                tracing::warn!("identity_upload_discard_failed");
            }
        }
        Err(_) => tracing::error!("identity_upload_still_shared"),
    }
}

/// US-106 AC1-AC5: pengajuan NIK + foto KTP untuk penerima layanan milik pemanggil.
pub(super) async fn submit(
    s: &IdentityServices,
    principal: &Principal,
    route: &str,
    care_recipient_id: Uuid,
    nik_input: &str,
    upload_slot_id: Uuid,
) -> Result<SubmissionAccepted, ProfileError> {
    let nik = Nik::parse(nik_input).map_err(|_| {
        ProfileError::Validation(vec![FieldError::new(
            "nik",
            "invalid_format",
            "NIK harus 16 digit angka",
        )])
    })?;
    let ip = client_ip(principal, route)?;
    let owned = owned_recipient(
        &s.pool,
        principal,
        route,
        care_recipient_id,
        Sensitivity::Sensitive,
    )
    .await?;
    let account_id = principal.account_id();
    let current = s.blind_index.compute(&nik).map_err(crypto_error)?;
    let keys = LimitKeys::new(account_id, &current.value, ip);
    if is_held(s, &keys).await? {
        security::rate_limited(Some(account_id), route);
        return Err(ProfileError::NikUnavailable);
    }

    let upload = files::validate_upload(
        &s.pool,
        s.clock.as_ref(),
        &s.files,
        account_id,
        upload_slot_id,
        UploadPurpose::KtpPhoto,
    )
    .await
    .map_err(ProfileError::File)?;
    let upload = Arc::new(upload);

    if owned.is_account_owner() {
        let candidates: Vec<(Vec<u8>, i16)> = s
            .blind_index
            .lookup_candidates(&nik)
            .into_iter()
            .map(|b| (b.value.to_vec(), b.key_version))
            .collect();
        let holder = match s.pool.acquire().await {
            Ok(mut c) => repo::registry_holder(&mut c, &candidates).await,
            Err(_) => Err(RepoError::Database),
        };
        let holder = match holder {
            Ok(h) => h,
            Err(e) => {
                discard(s, upload).await;
                return Err(repo_error(e));
            }
        };
        if owner_nik_conflict(true, care_recipient_id, holder) {
            discard(s, upload).await;
            record_rejection(s, account_id, &keys).await;
            return Err(ProfileError::NikUnavailable);
        }
    }

    let document_id = new_id();
    let request_id = new_id();
    let (data_key_row, data_key) = match s
        .keyring
        .create_data_key(Container::new(ContainerType::IdentityDocument, document_id))
        .await
    {
        Ok(v) => v,
        Err(e) => {
            discard(s, upload).await;
            return Err(crypto_error(e));
        }
    };
    let nik_enc = match data_key.seal(
        EncryptionContext::IDENTITY_DOCUMENT_NIK,
        document_id,
        nik.expose().as_bytes(),
    ) {
        Ok(c) => c,
        Err(e) => {
            discard(s, upload).await;
            return Err(crypto_error(e));
        }
    };
    let last4 = nik.last4();
    let nik_digits = Sensitive::new(nik.expose().to_string());
    let context = audit_context(principal, ip);
    let event_actor = principal.clone();
    let clock = Arc::clone(&s.clock);
    let work_upload = Arc::clone(&upload);
    let bidx = current;

    let result = in_audited_tx(&s.pool, s.clock.as_ref(), move |conn| {
        Box::pin(async move {
            let locked = repo::lock_care_recipient(conn, care_recipient_id)
                .await?
                .ok_or(ProfileError::NotFound)?;
            if let Some(latest) = locked.latest_verification_request_id {
                let state = repo::request_state(conn, latest)
                    .await?
                    .ok_or(ProfileError::Internal("dangling_latest_request"))?;
                let shown = IdentityStatus::from_effective(Some(state.status))
                    .ok_or(ProfileError::Internal("cancelled_effective_request"))?;
                if !shown.accepts_new_submission() {
                    return Err(ProfileError::SubmissionLocked);
                }
            }
            let file_id = files::record_stored_file(
                conn,
                clock.as_ref(),
                &work_upload,
                Some(FileContainer {
                    container_type: ContainerType::IdentityDocument,
                    container_id: document_id,
                }),
            )
            .await
            .map_err(ProfileError::File)?;
            insert_data_key(conn, &data_key_row)
                .await
                .map_err(crypto_error)?;
            repo::insert_identity_document(
                conn,
                &repo::NewIdentityDocument {
                    id: document_id,
                    created_by: account_id,
                    care_recipient_id,
                    document_type: DocumentType::Ktp,
                    nik_enc: &nik_enc,
                    nik_bidx: &bidx.value,
                    nik_bidx_key_version: bidx.key_version,
                    nik_last4: &last4,
                    data_key_id: data_key_row.id,
                    file_id,
                },
            )
            .await?;
            repo::insert_request(
                conn,
                request_id,
                account_id,
                care_recipient_id,
                document_id,
                TriggerType::DocumentSubmitted,
            )
            .await?;
            let step = {
                let mut store = IvrStatusStore {
                    conn: &mut *conn,
                    request_id,
                    details: EventDetails {
                        actor_type: "patient",
                        actor_account_id: account_id,
                        reason_code: None,
                        note_enc: None,
                        note_data_key_id: None,
                        correlation_id: event_actor.correlation_id(),
                    },
                };
                status_flow::transition(
                    &mut store,
                    &VerificationRules::new(
                        VerificationActor::Patient,
                        TriggerType::DocumentSubmitted,
                    ),
                    VerificationStatus::Pending,
                )
                .await
                .map_err(transition_error)?
            };
            repo::set_effective_request(
                conn,
                care_recipient_id,
                Some(request_id),
                Some(RecipientNik {
                    bidx: &bidx.value,
                    key_version: bidx.key_version,
                    last4: &last4,
                }),
                account_id,
            )
            .await?;
            let document_type = Value::code(DocumentType::Ktp.as_str())
                .map_err(|_| ProfileError::Internal("audit_value"))?;
            let event = request_event(
                &event_actor,
                RequestEvent {
                    code: EventCode::E10,
                    action: "identity_document_submitted",
                    request_id,
                    subject: care_recipient_id,
                    changes: vec![
                        status_change(step.from, step.to)?,
                        FieldChange::new("nik", None, Some(Value::digits(nik_digits))),
                        FieldChange::new("document_type", None, Some(document_type)),
                    ],
                    outcome: Outcome::success(),
                    context: &context,
                },
            )?;
            Ok((
                SubmissionAccepted {
                    identity_verification_request_id: request_id,
                    identity_document_id: document_id,
                    status: IdentityStatus::Pending.as_str(),
                },
                vec![event],
            ))
        })
    })
    .await;

    match result {
        Ok(accepted) => {
            drop(upload);
            Ok(accepted)
        }
        Err(e) => {
            discard(s, upload).await;
            let e = audited_error(e);
            if matches!(e, ProfileError::ProjectionMismatch) {
                security::status_projection_mismatch("identity_verification_request", request_id);
            }
            Err(e)
        }
    }
}

/// US-106 AC5: pasien membatalkan pengajuan dokumen yang masih menunggu.
pub(super) async fn cancel(
    s: &IdentityServices,
    principal: &Principal,
    route: &str,
    care_recipient_id: Uuid,
) -> Result<IdentityStatusView, ProfileError> {
    let ip = client_ip(principal, route)?;
    owned_recipient(
        &s.pool,
        principal,
        route,
        care_recipient_id,
        Sensitivity::NonSensitive,
    )
    .await?;
    let account_id = principal.account_id();
    let context = audit_context(principal, ip);
    let event_actor = principal.clone();
    let failed_context = context.clone();

    let result = in_audited_tx(&s.pool, s.clock.as_ref(), move |conn| {
        Box::pin(async move {
            let locked = repo::lock_care_recipient(conn, care_recipient_id)
                .await?
                .ok_or(ProfileError::NotFound)?;
            let latest = locked
                .latest_verification_request_id
                .ok_or(ProfileError::TransitionConflict)?;
            let state = repo::request_state(conn, latest)
                .await?
                .ok_or(ProfileError::Internal("dangling_latest_request"))?;
            let step = {
                let mut store = IvrStatusStore {
                    conn: &mut *conn,
                    request_id: state.id,
                    details: EventDetails {
                        actor_type: "patient",
                        actor_account_id: account_id,
                        reason_code: None,
                        note_enc: None,
                        note_data_key_id: None,
                        correlation_id: event_actor.correlation_id(),
                    },
                };
                status_flow::transition(
                    &mut store,
                    &VerificationRules::new(VerificationActor::Patient, state.trigger),
                    VerificationStatus::Cancelled,
                )
                .await
                .map_err(transition_error)?
            };
            let previous =
                repo::previous_effective_request(conn, care_recipient_id, state.id).await?;
            repo::set_effective_request(
                conn,
                care_recipient_id,
                previous.as_ref().map(|p| p.request_id),
                previous.as_ref().map(|p| RecipientNik {
                    bidx: &p.nik_bidx,
                    key_version: p.nik_bidx_key_version,
                    last4: &p.nik_last4,
                }),
                account_id,
            )
            .await?;
            let event = request_event(
                &event_actor,
                RequestEvent {
                    code: EventCode::E10,
                    action: "identity_submission_cancelled",
                    request_id: state.id,
                    subject: care_recipient_id,
                    changes: vec![status_change(step.from, step.to)?],
                    outcome: Outcome::success(),
                    context: &context,
                },
            )?;
            Ok(((), vec![event]))
        })
    })
    .await;

    match result {
        Ok(()) => identity_status_unchecked(&s.pool, care_recipient_id).await,
        Err(e) => {
            let e = audited_error(e);
            if matches!(e, ProfileError::TransitionConflict) {
                // AB-36: transisi yang kalah dicatat dengan hasil gagal di transaksi tersendiri.
                if let Ok(event) =
                    cancel_conflict_event(principal, care_recipient_id, &failed_context)
                {
                    record_failed(&s.pool, s.clock.as_ref(), event).await;
                }
            }
            Err(e)
        }
    }
}

/// Status identitas penerima layanan milik pemanggil (AB-05).
pub(super) async fn status(
    s: &IdentityServices,
    principal: &Principal,
    route: &str,
    care_recipient_id: Uuid,
) -> Result<IdentityStatusView, ProfileError> {
    owned_recipient(
        &s.pool,
        principal,
        route,
        care_recipient_id,
        Sensitivity::NonSensitive,
    )
    .await?;
    identity_status_unchecked(&s.pool, care_recipient_id).await
}

/// Status tanpa pemeriksaan relasi: hanya dipanggil setelah relasi diperiksa di use case yang sama.
async fn identity_status_unchecked(
    pool: &PgPool,
    care_recipient_id: Uuid,
) -> Result<IdentityStatusView, ProfileError> {
    let row = repo::identity_status(pool, care_recipient_id)
        .await?
        .ok_or(ProfileError::NotFound)?;
    let status = row
        .status
        .as_deref()
        .map(|c| VerificationStatus::parse(c).ok_or(ProfileError::Internal("corrupt_row")))
        .transpose()?;
    let trigger = row
        .trigger_type
        .as_deref()
        .map(|c| TriggerType::parse(c).ok_or(ProfileError::Internal("corrupt_row")))
        .transpose()?;
    let shown = IdentityStatus::from_effective(status)
        .ok_or(ProfileError::Internal("cancelled_effective_request"))?;
    Ok(IdentityStatusView {
        status: shown.as_str(),
        identity_verification_request_id: row.request_id,
        rejection_reason: if shown == IdentityStatus::Rejected {
            row.reason_code
        } else {
            None
        },
        nik_last4: row.request_id.and(row.nik_last4),
        submitted_at: row.submitted_at,
        can_submit: shown.accepts_new_submission(),
        can_cancel: shown == IdentityStatus::Pending
            && trigger == Some(TriggerType::DocumentSubmitted),
    })
}

/// Objek penerima layanan untuk event hasil gagal yang permintaannya tidak dapat dipastikan (pembatalan
/// yang kalah atau tanpa pengajuan yang menunggu). Tanpa nilai field.
static CARE_RECIPIENT_OBJECT: ObjectPolicy = ObjectPolicy::non_clinical("care_recipient", &[]);

fn cancel_conflict_event(
    principal: &Principal,
    care_recipient_id: Uuid,
    context: &AuditContext,
) -> Result<AuditEvent, ProfileError> {
    let code = |v: &str| Code::new(v).map_err(|_| ProfileError::Internal("audit_action"));
    Ok(AuditEvent::new(
        EventCode::E10,
        code("identity_submission_cancelled")?,
        actor(principal)?,
        ObjectRef::with_uuid(&CARE_RECIPIENT_OBJECT, care_recipient_id),
    )
    .subject(Subject::care_recipient(care_recipient_id))
    .outcome(Outcome::failure(code("transition_conflict")?))
    .context(context))
}
