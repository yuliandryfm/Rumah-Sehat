//! Dokumen NIB, NPWP, dan bukti kewenangan perusahaan oleh HR (F1-T102, US-1901 AC2, T1; ADR-018, ADR-028
//! butir 1; keputusan user Q8 dan Q9, 2026-11-26).
//!
//! **Alur** (tiga langkah ADR-018): HR yang sudah aktivasi akun (password dan TOTP) dan masuk lewat sesi
//! korporat (1) meminta slot unggah, (2) mengunggah langsung ke object storage, (3) mengonfirmasi dengan jenis
//! dokumen. Tidak ada slot unggah anonim: pendaftaran publik (F1-T101) tidak memuat berkas, dan perusahaan
//! berstatus "Menunggu verifikasi" baru tampil di antrean Verifikator sebagai "siap diverifikasi" setelah ketiga
//! jenis dokumen ada (`verification.rs`). Bukti kewenangan = surat kuasa atau surat penugasan dari direksi
//! berbentuk PDF atau foto yang ditandatangani; sistem tidak memeriksa tanda tangan (tugas Verifikator, Q9).
//!
//! **Isolasi tenant.** Perusahaan selalu dari keanggotaan aktif sesi (`authz::CorporateScope`), tidak pernah dari
//! isi permintaan; tidak ada ID dokumen atau perusahaan di permintaan. Slot unggah milik akun pembuatnya
//! (modul `platform`): slot akun lain dijawab `404` yang sama dengan slot tidak ada (KU-03). Status perusahaan
//! diperiksa dua kali: dari cakupan sesi (cepat) dan ulang di transaksi yang mengunci baris perusahaan, sehingga
//! keputusan Verifikator yang bersamaan (yang mengunci baris yang sama) tidak menyelipkan perubahan dokumen.
//!
//! **Penggantian.** Satu dokumen aktif per jenis (`uqp_corp_client_doc_corporate_client_id_document_kind`):
//! mengonfirmasi jenis yang sudah ada menghapus lunak baris dokumen lama dan `stored_file`-nya (alasan
//! `replaced`, ADR-003) di transaksi yang sama dengan penyisipan yang baru; objek lama dimusnahkan retensi.
//!
//! **Audit** (modul kritis, ADR-005/ADR-009): E-10 `corporate_document_uploaded` (jenis dokumen dan apakah
//! mengganti; tanpa nama berkas, ukuran, isi, atau kunci objek) di transaksi yang sama dengan penulisan; unggahan
//! yang ditolak validasi isi (413/415, termasuk PDF berkonten aktif) tercatat E-10 hasil gagal dengan kode
//! tertutup di transaksi tersendiri. Unduhan HR dicatat E-12 oleh modul `platform` sebelum URL terbit.

use super::api::{CorporateDocuments, DocumentStatusView, DocumentsOverviewView};
use super::domain::{
    self, DOCUMENT_SLOT_WINDOW, DOCUMENT_SLOTS_PER_WINDOW, DocumentKind, may_manage_documents,
};
use super::error::CorporateError;
use super::repo::{self, NewDocument};
use crate::modules::platform::api::{
    self as files, DownloadRequest, DownloadTicket, FileAccess, FileError, UploadPurpose,
    UploadSlotRequest, UploadTicket, ValidatedUpload,
};
use audit::{
    Actor, AuditContext, AuditEvent, AuditedTxError, Code, EventCode, FieldChange, FieldRule,
    ObjectPolicy, ObjectRef, Outcome, Source, SourceComponent, Value, in_audited_tx,
};
use authz::{CorporateClientStatus, CorporateScope};
use envelope::FieldError;
use foundation::{Clock, FixedClock, Uuid, new_id};
use observability::security;
use sqlx::PgPool;
use std::convert::Infallible;
use std::net::IpAddr;
use std::sync::Arc;

/// Percobaan menulis event hasil gagal di transaksi tersendiri (coding-standards §4.4 butir 3).
const FAILED_AUDIT_ATTEMPTS: usize = 3;
const UPLOADED_ACTION: &str = "corporate_document_uploaded";
/// Alasan penghapusan lunak dokumen yang digantikan (kolom `delete_reason`).
const REPLACED_REASON: &str = "replaced";

/// Kebijakan audit objek dokumen perusahaan (daftar izin before/after, ADR-009 butir 7): hanya jenis dokumen
/// dan penanda penggantian. Nama berkas, ukuran, checksum, kunci objek, NIB, dan NPWP tidak terdaftar sehingga
/// pustaka menolak event yang mencoba mencatatnya.
static DOCUMENT_POLICY: ObjectPolicy = ObjectPolicy::non_clinical(
    "corporate_client_document",
    &[
        FieldRule::value("document_kind"),
        FieldRule::value("replaced"),
    ],
);
/// Kebijakan audit objek perusahaan untuk event hasil gagal (tanpa perubahan field).
static CLIENT_POLICY: ObjectPolicy = ObjectPolicy::non_clinical("corporate_client", &[]);

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

/// IP klien wajib (TD-22, gagal-tertutup): tanpa IP, event audit aksi pengguna tidak dapat ditulis (PRD §7.2).
fn client_ip(scope: &CorporateScope, route: &str) -> Result<IpAddr, CorporateError> {
    scope.principal().client_ip().ok_or_else(|| {
        tracing::error!(route = route, "client_ip_missing");
        CorporateError::Internal("client_ip_missing")
    })
}

/// Konteks audit HR: komponen member area korporat (PRD §7.2, F1-T100).
fn audit_context(scope: &CorporateScope, ip: IpAddr) -> AuditContext {
    AuditContext::new(
        scope.principal().correlation_id(),
        Source::component(SourceComponent::CorporatePortal).ip(ip),
    )
}

fn actor(scope: &CorporateScope) -> Result<Actor, CorporateError> {
    let principal = scope.principal();
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

/// Jenis dokumen dari isi permintaan; di luar daftar = `422` per field.
fn parse_kind(raw: &str) -> Result<DocumentKind, CorporateError> {
    DocumentKind::parse(raw).ok_or_else(|| {
        CorporateError::Validation(vec![FieldError::new(
            "document_kind",
            "invalid_choice",
            "Pilih jenis dokumen: nib, npwp, atau authority_proof",
        )])
    })
}

/// Gerbang cepat dari cakupan sesi: dokumen hanya untuk perusahaan Menunggu verifikasi atau Ditolak.
fn gate(scope: &CorporateScope) -> Result<(), CorporateError> {
    if may_manage_documents(scope.client_status()) {
        Ok(())
    } else {
        Err(CorporateError::DocumentsNotManageable)
    }
}

/// Ringkasan status dan dokumen perusahaan pemanggil (US-1901 AC6: status verifikasi dan alasan penolakan).
pub(super) async fn overview(
    s: &CorporateDocuments<'_>,
    scope: &CorporateScope,
) -> Result<DocumentsOverviewView, CorporateError> {
    gate(scope)?;
    let client_id = scope.corporate_client_id();
    let client = repo::client_status(s.pool, client_id)
        .await
        .map_err(db_error)?
        .ok_or(CorporateError::NotFound)?;
    let status = corrupt(CorporateClientStatus::parse(&client.status))?;
    if !may_manage_documents(status) {
        return Err(CorporateError::DocumentsNotManageable);
    }
    read_overview(s.pool, client_id, status, client.submission_round)
        .await
        .map_err(db_error)
}

/// Membaca dokumen aktif dan alasan penolakan lalu menyusun ringkasan. `executor` boleh pool atau koneksi
/// transaksi pemanggil (respons konfirmasi dibangun dari data di dalam transaksi, review T1 F1-T102).
async fn read_overview(
    executor: &PgPool,
    client_id: Uuid,
    status: CorporateClientStatus,
    submission_round: i32,
) -> Result<DocumentsOverviewView, sqlx::Error> {
    let docs = repo::active_documents(executor, client_id).await?;
    let rejection_reason = if status == CorporateClientStatus::Rejected {
        repo::round_decision(executor, client_id, submission_round)
            .await?
            .filter(|d| d.decision == domain::Decision::Rejected.as_str())
            .and_then(|d| d.reason_code)
    } else {
        None
    };
    Ok(build_overview(
        status,
        submission_round,
        &docs,
        rejection_reason,
    ))
}

/// Ringkasan dari data yang sudah dibaca.
fn build_overview(
    status: CorporateClientStatus,
    submission_round: i32,
    docs: &[repo::DocumentRow],
    rejection_reason: Option<String>,
) -> DocumentsOverviewView {
    let documents: Vec<DocumentStatusView> = DocumentKind::ALL
        .into_iter()
        .map(|k| {
            let found = docs.iter().find(|d| d.document_kind == k.as_str());
            DocumentStatusView {
                kind: k.as_str(),
                uploaded: found.is_some(),
                uploaded_at: found.map(|d| d.created_at),
            }
        })
        .collect();
    let complete = documents.iter().all(|d| d.uploaded);
    DocumentsOverviewView {
        status: status.as_str(),
        submission_round,
        rejection_reason,
        documents,
        complete,
    }
}

/// Langkah 1: slot unggah untuk akun HR pemanggil. Jenis dan ukuran dinyatakan diperiksa modul `platform`
/// (JPG/PNG ≤ 5 MiB atau PDF ≤ 10 MiB, NFR-SEC-08); kuota per akun per jam dihitung dari `upload_slot`.
pub(super) async fn create_slot(
    s: &CorporateDocuments<'_>,
    scope: &CorporateScope,
    route: &str,
    content_type: String,
    size_bytes: u64,
) -> Result<UploadTicket, CorporateError> {
    gate(scope)?;
    let account_id = scope.account_id();
    let recent = files::count_recent_upload_slots(
        s.pool,
        s.clock,
        account_id,
        UploadPurpose::CorporateDocument,
        DOCUMENT_SLOT_WINDOW,
    )
    .await
    .map_err(CorporateError::File)?;
    if recent >= DOCUMENT_SLOTS_PER_WINDOW {
        security::rate_limited(Some(account_id), route);
        return Err(CorporateError::RateLimited);
    }
    files::create_upload_slot(
        s.pool,
        s.clock,
        s.files,
        UploadSlotRequest {
            owner_account_id: account_id,
            purpose: UploadPurpose::CorporateDocument,
            content_type,
            size_bytes,
        },
    )
    .await
    .map_err(CorporateError::File)
}

async fn discard(s: &CorporateDocuments<'_>, upload: Arc<ValidatedUpload>) {
    match Arc::try_unwrap(upload) {
        Ok(u) => {
            if files::discard_validated(s.clock, s.files, u).await.is_err() {
                // Objek tetap diklaim slot dan dibersihkan job pembersih (TD-17).
                tracing::warn!("corporate_document_discard_failed");
            }
        }
        Err(_) => tracing::error!("corporate_document_upload_still_shared"),
    }
}

/// Event hasil gagal unggahan yang ditolak isinya, tanpa data berkas.
fn rejected_event(
    scope: &CorporateScope,
    context: &AuditContext,
    reason: &'static str,
) -> Result<AuditEvent, CorporateError> {
    let action =
        Code::new(UPLOADED_ACTION).map_err(|_| CorporateError::Internal("audit_action"))?;
    let reason = Code::new(reason).map_err(|_| CorporateError::Internal("audit_reason"))?;
    let client_id = scope.corporate_client_id();
    Ok(AuditEvent::new(
        EventCode::E10,
        action,
        actor(scope)?,
        ObjectRef::with_uuid(&CLIENT_POLICY, client_id),
    )
    .tenant(client_id)
    .outcome(Outcome::failure(reason))
    .context(context))
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

/// Alasan tertutup untuk audit penolakan unggahan; hanya penolakan isi (bukan galat infrastruktur).
fn rejection_reason(e: &FileError) -> Option<&'static str> {
    match e {
        FileError::TooLarge => Some("file_too_large"),
        FileError::UnsupportedType => Some("file_type_rejected"),
        _ => None,
    }
}

/// Langkah 3: memvalidasi berkas yang sudah diunggah, lalu mencatatnya sebagai dokumen aktif `kind`
/// (menggantikan yang lama) bersama E-10 dalam satu transaksi.
pub(super) async fn confirm(
    s: &CorporateDocuments<'_>,
    scope: &CorporateScope,
    route: &str,
    kind: &str,
    upload_slot_id: Uuid,
) -> Result<DocumentsOverviewView, CorporateError> {
    let kind = parse_kind(kind)?;
    let ip = client_ip(scope, route)?;
    gate(scope)?;
    let context = audit_context(scope, ip);
    let account_id = scope.account_id();
    let client_id = scope.corporate_client_id();

    let upload = match files::validate_upload(
        s.pool,
        s.clock,
        s.files,
        account_id,
        upload_slot_id,
        UploadPurpose::CorporateDocument,
    )
    .await
    {
        Ok(u) => Arc::new(u),
        Err(e) => {
            if let Some(reason) = rejection_reason(&e)
                && let Ok(event) = rejected_event(scope, &context, reason)
            {
                record_failed(s.pool, s.clock, event).await;
            }
            return Err(CorporateError::File(e));
        }
    };

    let tx_upload = Arc::clone(&upload);
    let tx_actor = actor(scope)?;
    let tx_context = context.clone();
    // Jam dibekukan di awal transaksi: future transaksi tidak boleh meminjam `s.clock` (tidak statis).
    let now = s.clock.now();
    let result = in_audited_tx(s.pool, s.clock, move |conn| {
        Box::pin(async move {
            let clock = &FixedClock::new(now);
            // Kunci baris perusahaan: status diperiksa ulang dan unggahan serta keputusan Verifikator
            // (juga mengunci baris ini) berurutan.
            let locked = repo::lock_client(conn, client_id)
                .await
                .map_err(db_error)?
                .ok_or(CorporateError::NotFound)?;
            let status = corrupt(CorporateClientStatus::parse(&locked.status))?;
            if !may_manage_documents(status) {
                return Err(CorporateError::DocumentsNotManageable);
            }
            let current = repo::active_documents(&mut *conn, client_id)
                .await
                .map_err(db_error)?;
            let old = current.iter().find(|d| d.document_kind == kind.as_str());
            if let Some(old) = old {
                if !repo::soft_delete_document(
                    conn,
                    old.id,
                    client_id,
                    account_id,
                    REPLACED_REASON,
                    now,
                )
                .await
                .map_err(db_error)?
                {
                    return Err(CorporateError::Internal("document_not_replaced"));
                }
                files::soft_delete_stored_file(
                    conn,
                    clock,
                    old.stored_file_id,
                    account_id,
                    REPLACED_REASON,
                )
                .await
                .map_err(CorporateError::File)?;
            }
            // Wadah retensi berkas: DD §0.7 belum memuat wadah dokumen perusahaan, jadi tanpa wadah (NULL);
            // pemilik berkas = akun HR pengunggah.
            let stored_file_id = files::record_stored_file(conn, clock, &tx_upload, None)
                .await
                .map_err(CorporateError::File)?;
            let document_id = new_id();
            repo::insert_document(
                conn,
                &NewDocument {
                    id: document_id,
                    corporate_client_id: client_id,
                    document_kind: kind.as_str(),
                    stored_file_id,
                    by: account_id,
                    now,
                },
            )
            .await
            .map_err(db_error)?;
            let action =
                Code::new(UPLOADED_ACTION).map_err(|_| CorporateError::Internal("audit_action"))?;
            let event = AuditEvent::new(
                EventCode::E10,
                action,
                tx_actor,
                ObjectRef::with_uuid(&DOCUMENT_POLICY, document_id),
            )
            .tenant(client_id)
            .outcome(Outcome::success())
            .context(&tx_context)
            .change(FieldChange::new(
                "document_kind",
                None,
                Some(code_value(kind.as_str())?),
            ))
            .change(FieldChange::new(
                "replaced",
                None,
                Some(Value::Bool(old.is_some())),
            ));
            // Respons dibangun dari data di dalam transaksi (baris perusahaan masih terkunci), bukan dibaca
            // ulang setelah commit.
            let docs = repo::active_documents(&mut *conn, client_id)
                .await
                .map_err(db_error)?;
            let rejection_reason = if status == CorporateClientStatus::Rejected {
                repo::round_decision(&mut *conn, client_id, locked.submission_round)
                    .await
                    .map_err(db_error)?
                    .filter(|d| d.decision == domain::Decision::Rejected.as_str())
                    .and_then(|d| d.reason_code)
            } else {
                None
            };
            let view = build_overview(status, locked.submission_round, &docs, rejection_reason);
            Ok((view, vec![event]))
        })
    })
    .await;
    match result {
        Ok(view) => {
            tracing::info!(document_kind = kind.as_str(), "corporate_document_uploaded");
            Ok(view)
        }
        Err(e) => {
            // Transaksi gagal: berkas tervalidasi dibuang (slot dapat dipakai ulang tidak: tombstone/klaim).
            discard(s, upload).await;
            Err(audited_error(e))
        }
    }
}

/// URL unduh dokumen aktif `kind` milik perusahaan pemanggil (E-12 dulu, modul `platform`). Jenis di luar
/// daftar atau dokumen belum ada = `404` yang sama.
pub(super) async fn download_url(
    s: &CorporateDocuments<'_>,
    scope: &CorporateScope,
    route: &str,
    kind: &str,
) -> Result<DownloadTicket, CorporateError> {
    gate(scope)?;
    let ip = client_ip(scope, route)?;
    let kind = DocumentKind::parse(kind).ok_or(CorporateError::NotFound)?;
    let client_id = scope.corporate_client_id();
    let stored_file_id = repo::document_file(s.pool, client_id, kind.as_str())
        .await
        .map_err(db_error)?
        .ok_or(CorporateError::NotFound)?;
    let context = audit_context(scope, ip);
    files::issue_download_url(
        s.pool,
        s.clock,
        s.files,
        scope.principal(),
        &context,
        DownloadRequest {
            stored_file_id,
            purpose: UploadPurpose::CorporateDocument,
            route,
            access: FileAccess::CorporateHr {
                corporate_client_id: client_id,
            },
        },
    )
    .await
    .map_err(CorporateError::File)
}

#[cfg(test)]
mod tests {
    use super::*;

    // trace: ADR-005, DD A20 (audit dokumen tidak memuat berkas, NIB, atau NPWP)
    #[test]
    fn audit_policies_admit_no_file_or_company_identifier() {
        assert!(DOCUMENT_POLICY.validate().is_ok());
        assert!(CLIENT_POLICY.validate().is_ok());
        for f in [
            "npwp",
            "nib",
            "legal_name",
            "file_name",
            "size_bytes",
            "sha256",
            "object_key",
            "stored_file_id",
            "mime_type",
        ] {
            assert!(
                DOCUMENT_POLICY.fields().iter().all(|r| r.name() != f),
                "{f}"
            );
        }
        assert!(Code::new(UPLOADED_ACTION).is_ok());
        for reason in ["file_too_large", "file_type_rejected"] {
            assert!(Code::new(reason).is_ok(), "{reason}");
        }
    }

    #[test]
    fn only_content_rejections_are_audited_as_failed_uploads() {
        assert_eq!(
            rejection_reason(&FileError::TooLarge),
            Some("file_too_large")
        );
        assert_eq!(
            rejection_reason(&FileError::UnsupportedType),
            Some("file_type_rejected")
        );
        for e in [
            FileError::NotFound,
            FileError::StorageUnavailable,
            FileError::Busy,
            FileError::Internal("x"),
            FileError::Validation(vec![]),
        ] {
            assert_eq!(rejection_reason(&e), None, "{e}");
        }
    }

    #[test]
    fn unknown_document_kind_is_a_field_error() {
        for bad in ["", "ktp", "NIB", "authority-proof"] {
            assert!(
                matches!(parse_kind(bad), Err(CorporateError::Validation(_))),
                "{bad}"
            );
        }
        assert!(parse_kind("npwp").is_ok());
    }
}
