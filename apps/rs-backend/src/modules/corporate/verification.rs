//! Use case verifikasi perusahaan oleh Verifikator (F1-T104, US-1902), mengikuti reference implementation
//! US-201 (`docs/reference-implementation.md`): handler backoffice (`http.rs`) → fungsi di sini → `repo` /
//! `api` modul `platform` (URL unduh dokumen, daftar `reason_code`) / crate bersama (`audit`, `authz`).
//!
//! Akses: semua rute berkebijakan `Policy::rs_admin(&[Verifier])` (`policy.rs`). Verifikator boleh melihat
//! semua perusahaan (bukan data milik akunnya), jadi ID yang tidak ada atau terhapus = `404` (KU-03 tetap:
//! tidak ada pembeda lain).
//!
//! - Antrean ([`queue`], AC1): perusahaan "Menunggu verifikasi" urut `submitted_at` terlama, cursor = ID
//!   perusahaan terakhir halaman sebelumnya. Tanpa NPWP. F1-T102 (keputusan user Q8): bawaan hanya pengajuan
//!   yang ketiga jenis dokumennya lengkap ("siap diverifikasi"); [`QueueReadiness`] memilih yang belum lengkap
//!   atau semuanya, dan setiap item antrean memuat `documents_complete`.
//! - Detail ([`detail`], AC1): data perusahaan termasuk NPWP (klasifikasi Terbatas, DD A20: tidak pernah ke
//!   log, payload audit, atau pesan galat; dibawa sebagai `Sensitive<String>` sampai DTO), jenis dokumen aktif,
//!   dan keputusan putaran berjalan. PRD menetapkan E-05 untuk **dokumen dibuka**, bukan detail.
//! - Dokumen ([`document_url`], AC1): URL unduh berumur pendek lewat modul `platform`
//!   (`FileAccess::StaffCorporate`, ADR-018 butir 3): E-05 di-commit sebelum URL terbit; gagal mencatat =
//!   tidak ada URL.
//! - Keputusan ([`approve`], [`reject`], AC2-AC4): satu `audit::in_audited_tx` mengunci baris perusahaan
//!   (`FOR UPDATE`), memeriksa status masih "Menunggu verifikasi" (`domain::may_decide`), menyisipkan
//!   `corporate_verification_decision` untuk putaran berjalan (`ON CONFLICT DO NOTHING` atas UQ
//!   `(corporate_client_id, submission_round)`: keputusan pertama menang, AB-36), memindahkan status, dan
//!   menulis E-10 (PRD US-1902: "E-10 (keputusan)"). Pihak yang kalah dijawab `4090001` dan tercatat E-10
//!   hasil gagal `already_decided` di transaksi tersendiri (AC4 "ditolak dan tercatat").
//! - Status perusahaan berada di kolom `corporate_client.status` (Master, DD A20 Rincian K1) dengan riwayat
//!   keputusan di tabel Transaksi `corporate_verification_decision`; skema F1-T98 tidak memakai pasangan
//!   `_status_event`/`_state`, sehingga jalur `status-flow` (ADR-008 butir 2) tidak berlaku. Penjaga
//!   konkurensi pengganti: kunci baris + UQ per putaran + `UPDATE ... WHERE status = 'pending_verification'
//!   AND submission_round = $n` yang wajib mengubah tepat satu baris.
//! - Pengajuan ulang setelah ditolak (AC3): `submission_round` naik dan status kembali "Menunggu verifikasi"
//!   (ditulis jalur HR, F1-T101/F1-T103); keputusan putaran baru tidak terhalang keputusan putaran lama.
//! - Penolakan: kode alasan dari daftar tertutup (`domain::RejectionReason`) **dan** aktif di `reason_code`
//!   konteks `corporate_rejection` (migrasi data, daftar draf).

use super::api::{
    CorporateDecisionView, CorporateDetailView, CorporateQueueItemView, CorporateVerification,
    QueueReadiness, RejectionReasonView,
};
use super::domain::{
    self, Decision, DocumentKind, REJECTION_CONTEXT, RejectionReason, parse_origin,
};
use super::error::CorporateError;
use super::repo::{self, NewDecision};
use crate::modules::platform::api::{
    self as files, DownloadRequest, DownloadTicket, FileAccess, UploadPurpose,
};
use audit::{
    Actor, AuditContext, AuditEvent, AuditedTxError, Code, EventCode, FieldChange, FieldRule,
    ObjectPolicy, ObjectRef, Outcome, Source, SourceComponent, Value, in_audited_tx,
};
use authz::{AdminSubRole, CorporateClientStatus, Principal};
use envelope::{FieldError, Meta};
use foundation::{Clock, Sensitive, Uuid, new_id};
use sqlx::PgPool;
use std::convert::Infallible;
use std::net::IpAddr;

/// Sub-peran yang boleh membuka dokumen perusahaan (US-1902 AC1). Sama dengan kebijakan rute; diperiksa
/// ulang modul `platform` (`FileAccess::StaffCorporate`).
const VERIFIER_ROLES: &[AdminSubRole] = &[AdminSubRole::Verifier];

const DEFAULT_PAGE_SIZE: i64 = 20;
const MAX_PAGE_SIZE: i64 = 100;

/// Percobaan menulis event hasil gagal di transaksi tersendiri (coding-standards §4.4 butir 3).
const FAILED_AUDIT_ATTEMPTS: usize = 3;

const APPROVED_ACTION: &str = "corporate_verification_approved";
const REJECTED_ACTION: &str = "corporate_verification_rejected";

/// Kebijakan audit objek perusahaan (daftar izin before/after, ADR-009 butir 7): hanya status, kode alasan,
/// dan putaran pengajuan. NPWP, NIB, nama, alamat, dan telepon sengaja tidak terdaftar, sehingga pustaka
/// menolak event yang mencoba mencatatnya (NPWP Terbatas, DD A20).
static CORPORATE_CLIENT_POLICY: ObjectPolicy = ObjectPolicy::non_clinical(
    "corporate_client",
    &[
        FieldRule::value("status"),
        FieldRule::value("reason_code"),
        FieldRule::value("submission_round"),
    ],
);

fn db_error(_: sqlx::Error) -> CorporateError {
    // Pesan sqlx dapat memuat isi baris (DETAIL); tidak diteruskan ke log maupun respons (TD-03).
    CorporateError::Database
}

fn audited_error(e: AuditedTxError<CorporateError>) -> CorporateError {
    match e {
        AuditedTxError::Work(e) => e,
        AuditedTxError::Audit(_) => CorporateError::Internal("audit"),
        AuditedTxError::Database { .. } => CorporateError::Database,
    }
}

fn corrupt<T>(v: Option<T>) -> Result<T, CorporateError> {
    v.ok_or(CorporateError::Internal("corrupt_row"))
}

fn page_size(limit: Option<i64>) -> Result<i64, CorporateError> {
    match limit {
        None => Ok(DEFAULT_PAGE_SIZE),
        Some(n) if (1..=MAX_PAGE_SIZE).contains(&n) => Ok(n),
        Some(_) => Err(CorporateError::Validation(vec![FieldError::new(
            "limit",
            "out_of_range",
            "Jumlah item harus 1 sampai 100",
        )])),
    }
}

/// IP klien wajib (TD-22, gagal-tertutup): tanpa IP, event audit aksi pengguna tidak dapat ditulis
/// (PRD §7.2), sehingga permintaan ditolak sebelum ada efek apa pun.
fn client_ip(principal: &Principal, route: &str) -> Result<IpAddr, CorporateError> {
    principal.client_ip().ok_or_else(|| {
        tracing::error!(route = route, "client_ip_missing");
        CorporateError::Internal("client_ip_missing")
    })
}

/// Konteks audit Verifikator (field "Sumber" dan ID korelasi PRD §7.2). Endpoint ini hanya untuk admin
/// Ruang Sehat, sehingga komponennya backoffice.
fn audit_context(principal: &Principal, ip: IpAddr) -> AuditContext {
    AuditContext::new(
        principal.correlation_id(),
        Source::component(SourceComponent::Backoffice).ip(ip),
    )
}

fn actor(principal: &Principal) -> Result<Actor, CorporateError> {
    let code = |v: &str| Code::new(v).map_err(|_| CorporateError::Internal("actor"));
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

fn code_value(v: &str) -> Result<Value, CorporateError> {
    Value::code(v).map_err(|_| CorporateError::Internal("audit_value"))
}

/// Event E-10 atas perusahaan: objek `corporate_client`, tanpa subjek data pribadi (badan usaha), tenant =
/// perusahaan (ADR-028 butir 1).
fn decision_event(
    principal: &Principal,
    context: &AuditContext,
    action: &'static str,
    corporate_client_id: Uuid,
    changes: Vec<FieldChange>,
    outcome: Outcome,
) -> Result<AuditEvent, CorporateError> {
    let action = Code::new(action).map_err(|_| CorporateError::Internal("audit_action"))?;
    let mut event = AuditEvent::new(
        EventCode::E10,
        action,
        actor(principal)?,
        ObjectRef::with_uuid(&CORPORATE_CLIENT_POLICY, corporate_client_id),
    )
    .tenant(corporate_client_id)
    .outcome(outcome)
    .context(context);
    for c in changes {
        event = event.change(c);
    }
    Ok(event)
}

/// Menulis event hasil gagal di transaksi tersendiri, dicoba ulang; kegagalan tidak mengubah jawaban
/// (coding-standards §4.4 butir 3).
async fn record_failed(pool: &PgPool, clock: &dyn Clock, event: AuditEvent) {
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

/// US-1902 AC1: antrean "Menunggu verifikasi", terlama lebih dulu.
pub(super) async fn queue(
    s: &CorporateVerification<'_>,
    cursor: Option<Uuid>,
    limit: Option<i64>,
    readiness: QueueReadiness,
) -> Result<(Vec<CorporateQueueItemView>, Meta), CorporateError> {
    let limit = page_size(limit)?;
    let complete_only = match readiness {
        QueueReadiness::Ready => Some(true),
        QueueReadiness::Incomplete => Some(false),
        QueueReadiness::All => None,
    };
    let mut rows = repo::pending_queue(s.pool, cursor, complete_only, limit.saturating_add(1))
        .await
        .map_err(db_error)?;
    let has_more = i64::try_from(rows.len()).map_or(true, |n| n > limit);
    rows.truncate(usize::try_from(limit).map_err(|_| CorporateError::Internal("limit"))?);
    let next_cursor = if has_more {
        rows.last().map(|r| r.id.hyphenated().to_string())
    } else {
        None
    };
    let items = rows
        .into_iter()
        .map(|r| {
            Ok(CorporateQueueItemView {
                corporate_client_id: r.id,
                legal_name: r.legal_name,
                nib: r.nib,
                origin: corrupt(parse_origin(&r.origin))?,
                submission_round: r.submission_round,
                submitted_at: r.submitted_at,
                documents_complete: r.documents_complete,
            })
        })
        .collect::<Result<Vec<_>, CorporateError>>()?;
    Ok((
        items,
        Meta {
            next_cursor,
            has_more,
        },
    ))
}

/// US-1902 AC1: detail perusahaan untuk Verifikator.
pub(super) async fn detail(
    s: &CorporateVerification<'_>,
    corporate_client_id: Uuid,
) -> Result<CorporateDetailView, CorporateError> {
    let row = repo::client_detail(s.pool, corporate_client_id)
        .await
        .map_err(db_error)?
        .ok_or(CorporateError::NotFound)?;
    let npwp = Sensitive::new(row.npwp);
    let status = corrupt(CorporateClientStatus::parse(&row.status))?;
    let decision = repo::round_decision(s.pool, row.id, row.submission_round)
        .await
        .map_err(db_error)?;
    let documents = repo::active_document_kinds(s.pool, row.id)
        .await
        .map_err(db_error)?
        .iter()
        .map(|k| corrupt(DocumentKind::parse(k)).map(DocumentKind::as_str))
        .collect::<Result<Vec<_>, _>>()?;
    let (rejection_reason, decided_at) = match (&decision, status) {
        (Some(d), CorporateClientStatus::Rejected) if d.decision == Decision::Rejected.as_str() => {
            (d.reason_code.clone(), Some(d.created_at))
        }
        (Some(d), _) => (None, Some(d.created_at)),
        (None, _) => (None, None),
    };
    Ok(CorporateDetailView {
        corporate_client_id: row.id,
        legal_name: row.legal_name,
        nib: row.nib,
        npwp,
        office_address: row.office_address,
        office_phone: row.office_phone,
        status: status.as_str(),
        origin: corrupt(parse_origin(&row.origin))?,
        submission_round: row.submission_round,
        submitted_at: row.submitted_at,
        documents,
        rejection_reason,
        decided_at,
        can_decide: domain::may_decide(status),
    })
}

/// US-1902 AC1: URL unduh dokumen perusahaan (E-05 dulu, modul `platform` F1-T30). Jenis di luar daftar,
/// perusahaan tidak ada, atau dokumen jenis itu belum ada = `404` yang sama.
pub(super) async fn document_url(
    s: &CorporateVerification<'_>,
    principal: &Principal,
    route: &str,
    corporate_client_id: Uuid,
    document_kind: &str,
) -> Result<DownloadTicket, CorporateError> {
    let ip = client_ip(principal, route)?;
    let kind = DocumentKind::parse(document_kind).ok_or(CorporateError::NotFound)?;
    let stored_file_id = repo::document_file(s.pool, corporate_client_id, kind.as_str())
        .await
        .map_err(db_error)?
        .ok_or(CorporateError::NotFound)?;
    let context = audit_context(principal, ip);
    files::issue_download_url(
        s.pool,
        s.clock,
        s.files,
        principal,
        &context,
        DownloadRequest {
            stored_file_id,
            purpose: UploadPurpose::CorporateDocument,
            route,
            access: FileAccess::StaffCorporate {
                roles: VERIFIER_ROLES,
                corporate_client_id,
            },
        },
    )
    .await
    .map_err(CorporateError::File)
}

/// Daftar alasan penolakan aktif hasil migrasi data (US-1902 AC3), urut daftar migrasi; hanya kode yang
/// juga dikenal domain.
pub(super) async fn rejection_reasons(
    s: &CorporateVerification<'_>,
) -> Result<Vec<RejectionReasonView>, CorporateError> {
    let mut conn = s.pool.acquire().await.map_err(db_error)?;
    let rows = files::active_reason_codes(&mut conn, REJECTION_CONTEXT)
        .await
        .map_err(|_| CorporateError::Database)?;
    Ok(rows
        .into_iter()
        .filter_map(|r| {
            RejectionReason::parse(&r.code).map(|c| RejectionReasonView {
                code: c.as_str(),
                label: r.label,
            })
        })
        .collect())
}

fn invalid_reason() -> CorporateError {
    CorporateError::Validation(vec![FieldError::new(
        "reason_code",
        "invalid_choice",
        "Pilih alasan penolakan dari daftar",
    )])
}

/// US-1902 AC2: Verifikator menyetujui pengajuan; perusahaan Aktif.
pub(super) async fn approve(
    s: &CorporateVerification<'_>,
    principal: &Principal,
    route: &str,
    corporate_client_id: Uuid,
) -> Result<CorporateDecisionView, CorporateError> {
    decide(s, principal, route, corporate_client_id, None).await
}

/// US-1902 AC3: Verifikator menolak pengajuan dengan kode alasan; perusahaan Ditolak.
pub(super) async fn reject(
    s: &CorporateVerification<'_>,
    principal: &Principal,
    route: &str,
    corporate_client_id: Uuid,
    reason_code: &str,
) -> Result<CorporateDecisionView, CorporateError> {
    let reason = RejectionReason::parse(reason_code).ok_or_else(invalid_reason)?;
    decide(s, principal, route, corporate_client_id, Some(reason)).await
}

/// Jalur keputusan bersama: `reason = None` = setuju, `Some` = tolak.
async fn decide(
    s: &CorporateVerification<'_>,
    principal: &Principal,
    route: &str,
    corporate_client_id: Uuid,
    reason: Option<RejectionReason>,
) -> Result<CorporateDecisionView, CorporateError> {
    let ip = client_ip(principal, route)?;
    let context = audit_context(principal, ip);
    let (decision, action) = match reason {
        None => (Decision::Approved, APPROVED_ACTION),
        Some(_) => (Decision::Rejected, REJECTED_ACTION),
    };
    let actor_id = principal.account_id();
    let event_actor = principal.clone();
    let event_context = context.clone();
    let result = in_audited_tx(s.pool, s.clock, move |conn| {
        Box::pin(async move {
            let locked = repo::lock_client(conn, corporate_client_id)
                .await
                .map_err(db_error)?
                .ok_or(CorporateError::NotFound)?;
            let from = corrupt(CorporateClientStatus::parse(&locked.status))?;
            if !domain::may_decide(from) {
                return Err(CorporateError::AlreadyDecided);
            }
            if reason.is_none() {
                // F1-T102 (Q8): persetujuan hanya untuk pengajuan yang ketiga dokumennya lengkap. Dibaca di
                // transaksi yang sama dengan kunci baris, sehingga unggahan HR yang bersamaan (yang juga
                // mengunci baris ini) tidak menyelipkan perubahan di antara pemeriksaan dan keputusan.
                let kinds = repo::active_document_kinds(&mut *conn, corporate_client_id)
                    .await
                    .map_err(db_error)?;
                if !domain::documents_complete(&kinds) {
                    return Err(CorporateError::DocumentsIncomplete);
                }
            }
            if let Some(r) = reason {
                // AC3: daftar alasan dikelola lewat migrasi data; kode yang tidak aktif di sana ditolak.
                let active = files::active_reason_codes(conn, REJECTION_CONTEXT)
                    .await
                    .map_err(|_| CorporateError::Database)?;
                if !active.iter().any(|a| a.code == r.as_str()) {
                    return Err(invalid_reason());
                }
            }
            let inserted = repo::insert_decision(
                conn,
                &NewDecision {
                    id: new_id(),
                    corporate_client_id,
                    submission_round: locked.submission_round,
                    decision: decision.as_str(),
                    reason_code: reason.map(RejectionReason::as_str),
                    decided_by: actor_id,
                },
            )
            .await
            .map_err(db_error)?;
            if !inserted {
                return Err(CorporateError::AlreadyDecided);
            }
            let to = decision.resulting_status();
            let changed = repo::set_decided_status(
                conn,
                corporate_client_id,
                locked.submission_round,
                to.as_str(),
                actor_id,
            )
            .await
            .map_err(db_error)?;
            if changed != 1 {
                // Baris terkunci dan statusnya sudah diperiksa; selisih berarti data tidak konsisten.
                return Err(CorporateError::Internal("status_not_updated"));
            }
            let mut changes = vec![
                FieldChange::new(
                    "status",
                    Some(code_value(from.as_str())?),
                    Some(code_value(to.as_str())?),
                ),
                FieldChange::new(
                    "submission_round",
                    None,
                    Some(Value::Int(i64::from(locked.submission_round))),
                ),
            ];
            if let Some(r) = reason {
                changes.push(FieldChange::new(
                    "reason_code",
                    None,
                    Some(code_value(r.as_str())?),
                ));
            }
            let event = decision_event(
                &event_actor,
                &event_context,
                action,
                corporate_client_id,
                changes,
                Outcome::success(),
            )?;
            Ok((
                CorporateDecisionView {
                    corporate_client_id,
                    status: to.as_str(),
                    submission_round: locked.submission_round,
                    rejection_reason: reason.map(RejectionReason::as_str),
                },
                vec![event],
            ))
        })
    })
    .await;
    match result {
        Ok(v) => Ok(v),
        Err(e) => {
            let e = audited_error(e);
            if matches!(e, CorporateError::AlreadyDecided) {
                record_lost_decision(s, principal, &context, action, corporate_client_id).await;
            }
            Err(e)
        }
    }
}

/// AC4: keputusan yang kalah tercatat E-10 hasil gagal `already_decided` di transaksi tersendiri.
async fn record_lost_decision(
    s: &CorporateVerification<'_>,
    principal: &Principal,
    context: &AuditContext,
    action: &'static str,
    corporate_client_id: Uuid,
) {
    let Ok(code) = Code::new("already_decided") else {
        return;
    };
    if let Ok(event) = decision_event(
        principal,
        context,
        action,
        corporate_client_id,
        vec![],
        Outcome::failure(code),
    ) {
        record_failed(s.pool, s.clock, event).await;
    }
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

    // trace: DD A20 (NPWP Terbatas tidak pernah masuk payload audit), ADR-009 butir 7
    #[test]
    fn audit_policy_admits_no_identifying_company_field() {
        assert!(CORPORATE_CLIENT_POLICY.validate().is_ok());
        for f in [
            "npwp",
            "nib",
            "legal_name",
            "office_address",
            "office_phone",
        ] {
            assert!(
                CORPORATE_CLIENT_POLICY
                    .fields()
                    .iter()
                    .all(|r| r.name() != f),
                "{f}"
            );
        }
        for a in [APPROVED_ACTION, REJECTED_ACTION, "already_decided"] {
            assert!(Code::new(a).is_ok(), "{a}");
        }
    }
}
