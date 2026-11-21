//! Use case Verifikator (F1-T62, reference implementation US-201; pola untuk F1-T66). Satu jalur per
//! lapis seperti `service.rs`: handler backoffice (`verifier_http.rs`) → fungsi di sini → `repo` / `api`
//! modul `platform` (URL unduh foto) / crate bersama (`audit`, `crypto`, `status-flow`).
//!
//! Akses: semua rute berkebijakan `Policy::rs_admin(&[Verifier])` (sub-peran lain termasuk Super Admin →
//! `403` dari middleware, realm lain → `401`). Verifikator boleh melihat semua pengajuan (bukan data milik
//! akunnya), jadi ID yang tidak ada = `404` (KU-03 tetap: tidak ada pembeda lain).
//!
//! - Antrean ([`queue`], US-201 AC1): pengajuan "Menunggu verifikasi" urut dari yang terlama, cursor =
//!   ID permintaan terakhir halaman sebelumnya. NIK hanya 4 digit terakhir di daftar.
//! - Detail ([`detail`], AC1): NIK lengkap didekripsi hanya untuk Verifikator (PRD §3.1). Membuka NIK
//!   lengkap = akses data sensitif oleh pihak selain subjek data, sehingga E-05 berhasil ditulis dan di-commit
//!   **sebelum** NIK didekripsi dan dikirim; gagal mencatat = data tidak dikirim (coding-standards §4.4 butir 2).
//! - Foto dokumen ([`document_photo_url`], AC1, AC6): URL unduh berumur pendek lewat modul `platform`
//!   (`FileAccess::Staff`, F1-T30): E-05 di-commit sebelum URL terbit.
//! - Keputusan ([`approve`], [`reject`], AC2, AC3, AC3a, AC5, AC7): panggilan ke pihak luar (Transit untuk
//!   membuka DEK NIK atau membuat DEK keterangan) dilakukan sebelum transaksi (KEPUTUSAN-125). Transaksi
//!   teraudit mengunci baris penerima layanan (pengajuan, pembatalan, dan keputusan atas penerima layanan
//!   yang sama berurutan), memeriksa status masih `pending`, lalu transisi lewat `status-flow` (keputusan
//!   pertama menang, AB-36). Pihak yang kalah dijawab `4090204` dan tercatat E-07 hasil gagal di transaksi
//!   tersendiri.
//! - Persetujuan pemilik akun (UP-40, AC7): registri NIK pemilik akun dicocokkan dengan blind index semua
//!   versi kunci aktif (TD-10) untuk penerima layanan **lain**; bentrok → `4090203` (pesan konflik ke
//!   Verifikator) dan E-07 hasil gagal `nik_unavailable`. Bila tidak bentrok, baris registri aktif penerima
//!   layanan yang sama (NIK lama) di-soft-delete lalu baris baru disisipkan dengan blind index versi
//!   terbaru. Dua persetujuan NIK yang sama bersamaan untuk dua akun: index unik parsial registri menolak
//!   yang kedua (`4090203`).
//! - Penolakan: kode alasan dari daftar tertutup (domain) **dan** harus ada aktif di `reason_code` konteks
//!   `identity_rejection` (data migrasi, AC3). Keterangan dienkripsi dengan DEK baru wadah `status_note`
//!   (DD §0.7: wadah = entitas induk riwayat status, yaitu permintaan verifikasi; satu permintaan hanya
//!   punya satu event penolakan, jadi satu DEK per event keterangan), AAD = ID baris event.
//! - Pemisahan tugas (AC8, KEPUTUSAN-294): sebelum setuju/tolak, [`SelfDecisionGuard`] (titik keputusan
//!   tunggal, `self_decision.rs`) memeriksa NIK staf Verifikator terhadap NIK pemilik akun pasien. Ditolak =
//!   `4090205`, status tidak berubah, E-07 hasil gagal `separation_of_duties`. Detail memakai penjaga yang
//!   sama untuk `can_decide`.

use super::api::{
    DecisionView, IdentityServices, QueueItemView, RejectionReasonView, VerificationDetailView,
};
use super::domain::{
    DocumentType, RejectionNoteError, RejectionReason, TriggerType, VerificationActor,
    VerificationRules, VerificationStatus, normalize_rejection_note,
};
use super::error::ProfileError;
use super::repo::{self, DecisionTarget, EventDetails, IvrStatusStore, RepoError};
use super::self_decision::SelfDecisionGuard;
use super::service::{
    RequestEvent, audit_context, client_ip, record_failed, request_event, status_change,
};
use crate::modules::platform::api::{
    self as files, DownloadRequest, DownloadTicket, FileAccess, UploadPurpose,
};
use audit::{AuditedTxError, Code, EventCode, FieldChange, Outcome, Subject, Value, in_audited_tx};
use authz::{AdminSubRole, Principal};
use crypto::{
    Container, ContainerType, CryptoError, EncryptionContext, NewDataKey, Nik, insert_data_key,
    load_data_key,
};
use envelope::{FieldError, Meta};
use foundation::{Uuid, new_id};
use observability::security;
use std::convert::Infallible;

/// Sub-peran yang boleh membuka foto dokumen identitas (US-201 AC6). Sama dengan kebijakan rute;
/// diperiksa ulang modul `platform` (`FileAccess::Staff`).
pub(super) const VERIFIER_ROLES: &[AdminSubRole] = &[AdminSubRole::Verifier];

/// Konteks `reason_code` untuk penolakan identitas (CHECK `ck_reason_code_context`).
const REJECTION_CONTEXT: &str = "identity_rejection";

/// Alasan soft delete baris registri NIK lama saat NIK baru penerima layanan yang sama disetujui.
const REGISTRY_SUPERSEDED: &str = "superseded_by_new_approval";

const DEFAULT_PAGE_SIZE: i64 = 20;
const MAX_PAGE_SIZE: i64 = 100;

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

fn page_size(limit: Option<i64>) -> Result<i64, ProfileError> {
    match limit {
        None => Ok(DEFAULT_PAGE_SIZE),
        Some(n) if (1..=MAX_PAGE_SIZE).contains(&n) => Ok(n),
        Some(_) => Err(ProfileError::Validation(vec![FieldError::new(
            "limit",
            "out_of_range",
            "Jumlah item harus 1 sampai 100",
        )])),
    }
}

fn corrupt<T>(v: Option<T>) -> Result<T, ProfileError> {
    v.ok_or(ProfileError::Internal("corrupt_row"))
}

/// US-201 AC1: antrean pengajuan "Menunggu verifikasi", terlama lebih dulu.
pub(super) async fn queue(
    s: &IdentityServices,
    cursor: Option<Uuid>,
    limit: Option<i64>,
) -> Result<(Vec<QueueItemView>, Meta), ProfileError> {
    let limit = page_size(limit)?;
    let mut rows = repo::pending_queue(&s.pool, cursor, limit.saturating_add(1)).await?;
    let has_more = i64::try_from(rows.len()).map_or(true, |n| n > limit);
    rows.truncate(usize::try_from(limit).map_err(|_| ProfileError::Internal("limit"))?);
    let next_cursor = if has_more {
        rows.last().map(|r| r.request_id.hyphenated().to_string())
    } else {
        None
    };
    let items = rows
        .into_iter()
        .map(|r| {
            Ok(QueueItemView {
                identity_verification_request_id: r.request_id,
                care_recipient_id: r.care_recipient_id,
                is_account_owner: r.is_account_owner,
                full_name: r.full_name,
                birth_date: r.birth_date,
                sex: r.sex,
                document_type: corrupt(DocumentType::parse(&r.document_type))?.as_str(),
                trigger_type: corrupt(TriggerType::parse(&r.trigger_type))?.as_str(),
                nik_last4: r.nik_last4,
                submitted_at: r.submitted_at,
            })
        })
        .collect::<Result<Vec<_>, ProfileError>>()?;
    Ok((
        items,
        Meta {
            next_cursor,
            has_more,
        },
    ))
}

/// Membuka NIK lengkap dokumen identitas (DEK wadah `identity_document`, AAD = baris dokumen).
pub(super) async fn open_document_nik(
    s: &IdentityServices,
    document_id: Uuid,
    data_key_id: Uuid,
    nik_enc: &[u8],
) -> Result<Nik, ProfileError> {
    let mut conn = s.pool.acquire().await.map_err(|_| ProfileError::Database)?;
    let record = load_data_key(&mut conn, data_key_id)
        .await
        .map_err(crypto_error)?
        .ok_or(ProfileError::Internal("data_key_missing"))?;
    drop(conn);
    let key = s
        .keyring
        .open_data_key(
            &record,
            Container::new(ContainerType::IdentityDocument, document_id),
        )
        .await
        .map_err(crypto_error)?;
    let plain = key
        .open(
            EncryptionContext::IDENTITY_DOCUMENT_NIK,
            document_id,
            nik_enc,
        )
        .map_err(crypto_error)?;
    let text = std::str::from_utf8(plain.expose()).map_err(|_| ProfileError::Internal("nik"))?;
    Nik::parse(text).map_err(|_| ProfileError::Internal("nik"))
}

/// US-201 AC1: detail pengajuan dengan NIK lengkap. E-05 di-commit sebelum NIK didekripsi.
pub(super) async fn detail(
    s: &IdentityServices,
    principal: &Principal,
    route: &str,
    request_id: Uuid,
) -> Result<VerificationDetailView, ProfileError> {
    let ip = client_ip(principal, route)?;
    let row = repo::verification_detail(&s.pool, request_id)
        .await?
        .ok_or(ProfileError::NotFound)?;
    let context = audit_context(principal, ip);
    let event = request_event(
        principal,
        RequestEvent {
            code: EventCode::E05,
            action: "identity_document_viewed",
            request_id,
            subject: row.care_recipient_id,
            changes: vec![],
            outcome: Outcome::success(),
            context: &context,
        },
    )?;
    in_audited_tx(&s.pool, s.clock.as_ref(), move |_conn| {
        Box::pin(async move { Ok::<_, Infallible>(((), vec![event])) })
    })
    .await
    .map_err(|_| ProfileError::Internal("audit"))?;
    let nik = open_document_nik(s, row.document_id, row.data_key_id, &row.nik_enc).await?;
    let status = corrupt(VerificationStatus::parse(&row.status))?;
    // US-201 AC8: tombol setuju/tolak tidak tersedia untuk pengajuan yang tidak boleh diputuskan pemanggil.
    let can_decide = status == VerificationStatus::Pending
        && SelfDecisionGuard::new(s)
            .may_decide(
                principal,
                row.owner_account_id,
                row.is_account_owner.then_some(&nik),
            )
            .await?;
    Ok(VerificationDetailView {
        identity_verification_request_id: row.request_id,
        care_recipient_id: row.care_recipient_id,
        is_account_owner: row.is_account_owner,
        full_name: row.full_name,
        birth_date: row.birth_date,
        sex: row.sex,
        document_type: corrupt(DocumentType::parse(&row.document_type))?.as_str(),
        trigger_type: corrupt(TriggerType::parse(&row.trigger_type))?.as_str(),
        nik: nik.expose().to_string(),
        submitted_at: row.submitted_at,
        status: status.as_str(),
        rejection_reason: if status == VerificationStatus::Rejected {
            row.reason_code
        } else {
            None
        },
        decided_at: if matches!(
            status,
            VerificationStatus::Approved | VerificationStatus::Rejected
        ) {
            Some(row.status_changed_at)
        } else {
            None
        },
        can_decide,
    })
}

/// US-201 AC1, AC6: URL unduh foto dokumen (E-05 dulu, modul `platform` F1-T30).
pub(super) async fn document_photo_url(
    s: &IdentityServices,
    principal: &Principal,
    route: &str,
    request_id: Uuid,
) -> Result<DownloadTicket, ProfileError> {
    let ip = client_ip(principal, route)?;
    let target = repo::decision_target(&s.pool, request_id)
        .await?
        .ok_or(ProfileError::NotFound)?;
    let context = audit_context(principal, ip);
    files::issue_download_url(
        &s.pool,
        s.clock.as_ref(),
        &s.files,
        principal,
        &context,
        DownloadRequest {
            stored_file_id: target.file_id,
            purpose: UploadPurpose::KtpPhoto,
            route,
            access: FileAccess::Staff {
                roles: VERIFIER_ROLES,
                subject: Subject::care_recipient(target.care_recipient_id),
            },
        },
    )
    .await
    .map_err(ProfileError::File)
}

/// Daftar alasan penolakan aktif hasil migrasi data (US-201 AC3), urut daftar PRD.
pub(super) async fn rejection_reasons(
    s: &IdentityServices,
) -> Result<Vec<RejectionReasonView>, ProfileError> {
    let mut conn = s.pool.acquire().await.map_err(|_| ProfileError::Database)?;
    let rows = files::active_reason_codes(&mut conn, REJECTION_CONTEXT)
        .await
        .map_err(|_| ProfileError::Database)?;
    Ok(rows
        .into_iter()
        // Hanya kode yang juga dikenal domain yang dapat dipakai menolak.
        .filter_map(|r| {
            RejectionReason::parse(&r.code).map(|c| RejectionReasonView {
                code: c.as_str(),
                label: r.label,
            })
        })
        .collect())
}

/// Bahan NIK untuk persetujuan pemilik akun, dihitung sebelum transaksi.
struct OwnerNik {
    candidates: Vec<(Vec<u8>, i16)>,
    current: Vec<u8>,
    current_version: i16,
}

fn verifier_details(principal: &Principal, event_id: Uuid) -> EventDetails {
    EventDetails {
        event_id,
        actor_type: "rs_admin",
        actor_account_id: principal.account_id(),
        reason_code: None,
        note_enc: None,
        note_data_key_id: None,
        correlation_id: principal.correlation_id(),
    }
}

/// Keputusan yang kalah atau ditolak aturan dicatat E-07 hasil gagal di transaksi tersendiri.
async fn record_failed_decision(
    s: &IdentityServices,
    principal: &Principal,
    context: &audit::AuditContext,
    action: &'static str,
    request_id: Uuid,
    care_recipient_id: Uuid,
    reason: &'static str,
) {
    let Ok(code) = Code::new(reason) else { return };
    if let Ok(event) = request_event(
        principal,
        RequestEvent {
            code: EventCode::E07,
            action,
            request_id,
            subject: care_recipient_id,
            changes: vec![],
            outcome: Outcome::failure(code),
            context,
        },
    ) {
        record_failed(&s.pool, s.clock.as_ref(), event).await;
    }
}

/// Galat keputusan yang tercatat E-07 hasil gagal beserta kodenya.
fn failure_code(e: &ProfileError) -> Option<&'static str> {
    match e {
        ProfileError::AlreadyDecided => Some("already_decided"),
        ProfileError::OwnerNikTaken => Some("nik_unavailable"),
        ProfileError::SeparationOfDuties => Some("separation_of_duties"),
        _ => None,
    }
}

/// NIK pengajuan bila milik pemilik akun (anggota keluarga = `None`; tidak masuk registri, dan penjaga AC8
/// memakai NIK pemilik akun yang disetujui).
async fn owner_request_nik(
    s: &IdentityServices,
    target: &DecisionTarget,
) -> Result<Option<Nik>, ProfileError> {
    if !target.is_account_owner {
        return Ok(None);
    }
    open_document_nik(s, target.document_id, target.data_key_id, &target.nik_enc)
        .await
        .map(Some)
}

/// US-201 AC8: titik keputusan tunggal pemisahan tugas sebelum keputusan apa pun. Ditolak = `4090205` dan
/// E-07 hasil gagal `separation_of_duties` dengan aktor Verifikator; status tidak berubah.
async fn guard_decision(
    s: &IdentityServices,
    principal: &Principal,
    context: &audit::AuditContext,
    action: &'static str,
    request_id: Uuid,
    target: &DecisionTarget,
    request_nik: Option<&Nik>,
) -> Result<(), ProfileError> {
    if SelfDecisionGuard::new(s)
        .may_decide(principal, target.owner_account_id, request_nik)
        .await?
    {
        return Ok(());
    }
    let e = ProfileError::SeparationOfDuties;
    if let Some(code) = failure_code(&e) {
        record_failed_decision(
            s,
            principal,
            context,
            action,
            request_id,
            target.care_recipient_id,
            code,
        )
        .await;
    }
    Err(e)
}

/// US-201 AC2, AC5, AC7: Verifikator menyetujui pengajuan.
pub(super) async fn approve(
    s: &IdentityServices,
    principal: &Principal,
    route: &str,
    request_id: Uuid,
) -> Result<DecisionView, ProfileError> {
    let ip = client_ip(principal, route)?;
    let target = repo::decision_target(&s.pool, request_id)
        .await?
        .ok_or(ProfileError::NotFound)?;
    let context = audit_context(principal, ip);
    let care_recipient_id = target.care_recipient_id;

    // NIK pengajuan pemilik akun dibuka di luar transaksi (Transit): dipakai penjaga pemisahan tugas (AC8)
    // dan registri NIK pemilik akun (UP-40; hanya pemilik akun yang masuk registri).
    let request_nik = owner_request_nik(s, &target).await?;
    guard_decision(
        s,
        principal,
        &context,
        "identity_verification_approved",
        request_id,
        &target,
        request_nik.as_ref(),
    )
    .await?;
    let owner_nik = if let Some(nik) = &request_nik {
        let current = s.blind_index.compute(nik).map_err(crypto_error)?;
        Some(OwnerNik {
            candidates: s
                .blind_index
                .lookup_candidates(nik)
                .into_iter()
                .map(|b| (b.value.to_vec(), b.key_version))
                .collect(),
            current: current.value.to_vec(),
            current_version: current.key_version,
        })
    } else {
        None
    };

    let event_actor = principal.clone();
    let event_context = context.clone();
    let actor_id = principal.account_id();
    let result = in_audited_tx(&s.pool, s.clock.as_ref(), move |conn| {
        Box::pin(async move {
            repo::lock_care_recipient(conn, care_recipient_id)
                .await?
                .ok_or(ProfileError::NotFound)?;
            let state = repo::request_state(conn, request_id)
                .await?
                .ok_or(ProfileError::NotFound)?;
            if state.status != VerificationStatus::Pending {
                return Err(ProfileError::AlreadyDecided);
            }
            if let Some(nik) = &owner_nik {
                if repo::other_registry_holder(conn, &nik.candidates, care_recipient_id)
                    .await?
                    .is_some()
                {
                    return Err(ProfileError::OwnerNikTaken);
                }
                repo::retire_registry(conn, care_recipient_id, actor_id, REGISTRY_SUPERSEDED)
                    .await?;
                repo::insert_registry(
                    conn,
                    &repo::NewRegistry {
                        id: new_id(),
                        created_by: actor_id,
                        nik_bidx: &nik.current,
                        nik_bidx_key_version: nik.current_version,
                        care_recipient_id,
                        identity_verification_request_id: request_id,
                    },
                )
                .await
                .map_err(|e| match e {
                    RepoError::RegistryConflict => ProfileError::OwnerNikTaken,
                    e => e.into(),
                })?;
            }
            let step = {
                let mut store = IvrStatusStore {
                    conn: &mut *conn,
                    request_id,
                    details: verifier_details(&event_actor, new_id()),
                };
                status_flow::transition(
                    &mut store,
                    &VerificationRules::new(VerificationActor::Verifier, state.trigger),
                    VerificationStatus::Approved,
                )
                .await
                .map_err(decision_transition_error)?
            };
            let event = request_event(
                &event_actor,
                RequestEvent {
                    code: EventCode::E07,
                    action: "identity_verification_approved",
                    request_id,
                    subject: care_recipient_id,
                    changes: vec![status_change(step.from, step.to)?],
                    outcome: Outcome::success(),
                    context: &event_context,
                },
            )?;
            Ok((
                DecisionView {
                    identity_verification_request_id: request_id,
                    status: VerificationStatus::Approved.as_str(),
                    rejection_reason: None,
                },
                vec![event],
            ))
        })
    })
    .await;

    finish_decision(
        s,
        principal,
        &context,
        "identity_verification_approved",
        request_id,
        care_recipient_id,
        result,
    )
    .await
}

fn decision_transition_error(
    e: status_flow::TransitionError<VerificationStatus, RepoError>,
) -> ProfileError {
    use status_flow::TransitionError as T;
    match e {
        T::NotAllowed { .. } | T::Contended => ProfileError::AlreadyDecided,
        T::ProjectionMismatch => ProfileError::ProjectionMismatch,
        T::VersionOverflow => ProfileError::Internal("version_overflow"),
        T::Store(e) => e.into(),
    }
}

async fn finish_decision(
    s: &IdentityServices,
    principal: &Principal,
    context: &audit::AuditContext,
    action: &'static str,
    request_id: Uuid,
    care_recipient_id: Uuid,
    result: Result<DecisionView, AuditedTxError<ProfileError>>,
) -> Result<DecisionView, ProfileError> {
    match result {
        Ok(v) => Ok(v),
        Err(e) => {
            let e = audited_error(e);
            if matches!(e, ProfileError::ProjectionMismatch) {
                security::status_projection_mismatch("identity_verification_request", request_id);
            }
            if let Some(code) = failure_code(&e) {
                record_failed_decision(
                    s,
                    principal,
                    context,
                    action,
                    request_id,
                    care_recipient_id,
                    code,
                )
                .await;
            }
            Err(e)
        }
    }
}

fn note_error(e: RejectionNoteError) -> ProfileError {
    ProfileError::Validation(vec![match e {
        RejectionNoteError::Required => FieldError::new(
            "note",
            "required",
            "Keterangan wajib diisi untuk alasan Lainnya",
        ),
        RejectionNoteError::TooLong => {
            FieldError::new("note", "too_long", "Keterangan paling panjang 500 karakter")
        }
    }])
}

fn invalid_reason() -> ProfileError {
    ProfileError::Validation(vec![FieldError::new(
        "reason_code",
        "invalid_choice",
        "Pilih alasan penolakan dari daftar",
    )])
}

/// Keterangan terenkripsi siap disisipkan (DEK baru wadah `status_note`).
struct SealedNote {
    event_id: Uuid,
    data_key: NewDataKey,
    note_enc: Vec<u8>,
}

/// US-201 AC3, AC3a, AC5, AC7 (penolakan `nik_unavailable`): Verifikator menolak pengajuan.
pub(super) async fn reject(
    s: &IdentityServices,
    principal: &Principal,
    route: &str,
    request_id: Uuid,
    reason_code: &str,
    note: Option<&str>,
) -> Result<DecisionView, ProfileError> {
    let reason = RejectionReason::parse(reason_code).ok_or_else(invalid_reason)?;
    let note = normalize_rejection_note(reason, note).map_err(note_error)?;
    let ip = client_ip(principal, route)?;
    let target = repo::decision_target(&s.pool, request_id)
        .await?
        .ok_or(ProfileError::NotFound)?;
    let context = audit_context(principal, ip);
    let care_recipient_id = target.care_recipient_id;
    let request_nik = owner_request_nik(s, &target).await?;
    guard_decision(
        s,
        principal,
        &context,
        "identity_verification_rejected",
        request_id,
        &target,
        request_nik.as_ref(),
    )
    .await?;

    let event_id = new_id();
    let sealed = match note {
        Some(text) => {
            let (row, key) = s
                .keyring
                .create_data_key(Container::new(ContainerType::StatusNote, request_id))
                .await
                .map_err(crypto_error)?;
            let note_enc = key
                .seal(
                    EncryptionContext::IDENTITY_VERIFICATION_REQUEST_STATUS_EVENT_NOTE,
                    event_id,
                    text.as_bytes(),
                )
                .map_err(crypto_error)?;
            Some(SealedNote {
                event_id,
                data_key: row,
                note_enc,
            })
        }
        None => None,
    };

    let event_actor = principal.clone();
    let event_context = context.clone();
    let result = in_audited_tx(&s.pool, s.clock.as_ref(), move |conn| {
        Box::pin(async move {
            repo::lock_care_recipient(conn, care_recipient_id)
                .await?
                .ok_or(ProfileError::NotFound)?;
            let state = repo::request_state(conn, request_id)
                .await?
                .ok_or(ProfileError::NotFound)?;
            if state.status != VerificationStatus::Pending {
                return Err(ProfileError::AlreadyDecided);
            }
            // AC3: daftar alasan dikelola lewat migrasi data; kode yang tidak aktif di sana ditolak.
            let active = files::active_reason_codes(conn, REJECTION_CONTEXT)
                .await
                .map_err(|_| ProfileError::Database)?;
            if !active.iter().any(|r| r.code == reason.as_str()) {
                return Err(invalid_reason());
            }
            let mut details = verifier_details(&event_actor, event_id);
            details.reason_code = Some(reason.as_str());
            if let Some(n) = &sealed {
                insert_data_key(conn, &n.data_key)
                    .await
                    .map_err(crypto_error)?;
                details.event_id = n.event_id;
                details.note_enc = Some(n.note_enc.clone());
                details.note_data_key_id = Some(n.data_key.id);
            }
            let step = {
                let mut store = IvrStatusStore {
                    conn: &mut *conn,
                    request_id,
                    details,
                };
                status_flow::transition(
                    &mut store,
                    &VerificationRules::new(VerificationActor::Verifier, state.trigger),
                    VerificationStatus::Rejected,
                )
                .await
                .map_err(decision_transition_error)?
            };
            let reason_value =
                Value::code(reason.as_str()).map_err(|_| ProfileError::Internal("audit_value"))?;
            let event = request_event(
                &event_actor,
                RequestEvent {
                    code: EventCode::E07,
                    action: "identity_verification_rejected",
                    request_id,
                    subject: care_recipient_id,
                    changes: vec![
                        status_change(step.from, step.to)?,
                        FieldChange::new("reason_code", None, Some(reason_value)),
                    ],
                    outcome: Outcome::success(),
                    context: &event_context,
                },
            )?;
            Ok((
                DecisionView {
                    identity_verification_request_id: request_id,
                    status: VerificationStatus::Rejected.as_str(),
                    rejection_reason: Some(reason.as_str().to_string()),
                },
                vec![event],
            ))
        })
    })
    .await;

    finish_decision(
        s,
        principal,
        &context,
        "identity_verification_rejected",
        request_id,
        care_recipient_id,
        result,
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_size_bounds() {
        assert_eq!(page_size(None).ok(), Some(DEFAULT_PAGE_SIZE));
        assert_eq!(page_size(Some(1)).ok(), Some(1));
        assert_eq!(page_size(Some(100)).ok(), Some(100));
        assert!(page_size(Some(0)).is_err());
        assert!(page_size(Some(101)).is_err());
    }

    #[test]
    fn only_lost_decisions_and_owner_nik_conflicts_are_recorded_as_failed_e07() {
        assert_eq!(
            failure_code(&ProfileError::AlreadyDecided),
            Some("already_decided")
        );
        assert_eq!(
            failure_code(&ProfileError::OwnerNikTaken),
            Some("nik_unavailable")
        );
        assert_eq!(failure_code(&ProfileError::NotFound), None);
        assert_eq!(failure_code(&ProfileError::Validation(vec![])), None);
        for c in ["already_decided", "nik_unavailable"] {
            assert!(Code::new(c).is_ok());
        }
    }
}
