//! Use case modul `platform`: kesiapan (F1-T11), berkas unggahan (F1-T29, ADR-018; TD-17), dan URL unduh
//! (F1-T30).
//!
//! Berkas: tidak ada panggilan object storage selama transaksi database terbuka (KEPUTUSAN-125). Isi berkas,
//! URL bertanda tangan, dan kunci objek tidak pernah dicatat di log; yang dicatat hanya ID slot/berkas dan
//! kode alasan (ADR-010).
//!
//! Kunci objek (TD-17): unggahan mentah di `incoming/<uuid>` (kunci slot), hasil kode ulang di
//! `files/<uuid>`. Objek `files/` selalu diklaim di database (`upload_slot.sanitized_object_key`) sebelum
//! ditulis, sehingga rekonsiliasi dapat membedakan objek sah dari objek yatim.

use super::api::{
    DOWNLOAD_URL_TTL, DownloadRequest, DownloadTicket, FILE_RECONCILE_JOB, FileAccess,
    FileContainer, FileStorage, PublicFile, ReadinessCheck, ReconcileReport, SlotPurgeReport,
    UPLOAD_SLOT_PURGE_BATCH, UPLOAD_SLOT_PURGE_JOB, UPLOAD_SLOT_TOMBSTONE_GRACE, UploadInstruction,
    UploadMethod, UploadSlotRequest, UploadTicket, ValidatedUpload,
};
use super::domain::{
    ContentRejection, FILES_PREFIX, INCOMING_PREFIX, LogicalBucket, MAX_PHOTO_BYTES, PhotoFormat,
    UploadPurpose, sanitize_photo,
};
use super::error::FileError;
use super::repo;
use audit::{
    Actor, AuditContext, AuditEvent, AuditedTxError, Code, EventCode, ObjectPolicy, ObjectRef,
    Outcome, Subject, in_audited_tx,
};
use authz::{AccountType, Principal, visible_object};
use envelope::FieldError;
use foundation::{Clock, Uuid, new_id};
use object_storage::{GetOutcome, ObjectStorageError, ResponseHeaders, VersionListCursor};
use observability::security::{AccessDenial, access_denied};
use platform_jobs::{
    ClaimedJob, DedupeKey, ErrorCode, JobOutcome, JobPayload, JobType, MaxAttempts, NewJob, enqueue,
};
use sqlx::{PgConnection, PgPool};
use std::convert::Infallible;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use tokio::sync::OwnedSemaphorePermit;

/// Label use case untuk security log (tidak ada rute HTTP di task ini; endpoint dibuat modul pemakai).
const VALIDATE_ROUTE: &str = "platform.validate_upload";
/// Jumlah baris `stored_file` per halaman rekonsiliasi arah database → bucket.
const RECONCILE_DB_PAGE: i64 = 100;
/// Batas halaman daftar versi per prefiks per rekonsiliasi (pelindung putaran tanpa akhir; sisa kunci
/// diproses rekonsiliasi berikutnya).
const RECONCILE_MAX_PAGES: usize = 1000;
/// Percobaan maksimum job pemeliharaan berkas.
const MAINTENANCE_MAX_ATTEMPTS: i16 = 5;

/// Menjalankan semua pemeriksaan berurutan; mencatat nama yang gagal (tanpa detail) dan
/// mengembalikan `true` bila semuanya lulus (daftar kosong berarti siap).
pub(super) async fn is_ready(checks: &[Arc<dyn ReadinessCheck>]) -> bool {
    let mut ready = true;
    for c in checks {
        if !c.check().await {
            tracing::warn!(check = c.name(), "readiness_check_failed");
            ready = false;
        }
    }
    ready
}

fn db_error(_e: sqlx::Error) -> FileError {
    FileError::Internal("database")
}

fn storage_error(e: ObjectStorageError) -> FileError {
    match e {
        ObjectStorageError::Unavailable | ObjectStorageError::Timeout => {
            FileError::StorageUnavailable
        }
        _ => FileError::Internal("object_storage"),
    }
}

fn not_uploaded() -> FileError {
    FileError::Validation(vec![FieldError::new(
        "upload_slot_id",
        "not_uploaded",
        "Berkas belum diunggah",
    )])
}

fn grace() -> Result<time::Duration, FileError> {
    time::Duration::try_from(UPLOAD_SLOT_TOMBSTONE_GRACE).map_err(|_| FileError::Internal("grace"))
}

/// Permit validasi foto (TD-17): diambil sebelum isi berkas diunduh, dengan batas tunggu antrean supaya
/// permintaan tidak menumpuk tanpa batas saat semua permit terpakai (→ 503).
pub(super) async fn acquire_validation_permit(
    storage: &FileStorage,
) -> Result<OwnedSemaphorePermit, FileError> {
    match tokio::time::timeout(
        storage.queue_timeout,
        Arc::clone(&storage.permits).acquire_owned(),
    )
    .await
    {
        Ok(Ok(permit)) => Ok(permit),
        Ok(Err(_)) => Err(FileError::Internal("validation_permit")),
        Err(_) => {
            tracing::warn!("file_validation_queue_timeout");
            Err(FileError::Busy)
        }
    }
}

pub(super) async fn create_upload_slot(
    pool: &PgPool,
    clock: &dyn Clock,
    storage: &FileStorage,
    request: UploadSlotRequest,
) -> Result<UploadTicket, FileError> {
    let purpose = request.purpose;
    let format = PhotoFormat::from_mime(&request.content_type)
        .filter(|f| purpose.allows(*f))
        .ok_or(FileError::UnsupportedType)?;
    if request.size_bytes == 0 {
        return Err(FileError::Validation(vec![FieldError::new(
            "size_bytes",
            "out_of_range",
            "Ukuran berkas tidak valid",
        )]));
    }
    let max = purpose.max_size_bytes();
    if request.size_bytes > max {
        return Err(FileError::TooLarge);
    }
    let max_i32 = i32::try_from(max).map_err(|_| FileError::Internal("max_size_bytes"))?;
    let now = clock.now();
    let ttl =
        time::Duration::try_from(storage.slot_ttl).map_err(|_| FileError::Internal("slot_ttl"))?;
    let expires_at = now
        .checked_add(ttl)
        .ok_or(FileError::Internal("slot_ttl"))?;
    let id = new_id();
    let object_key = format!("{INCOMING_PREFIX}{}", new_id().hyphenated());
    let bucket = purpose.bucket();
    repo::insert_slot(
        pool,
        &repo::NewSlot {
            id,
            expires_at,
            owner_account_id: request.owner_account_id,
            bucket: bucket.as_str(),
            object_key: &object_key,
            allowed_mime_type: format.mime(),
            max_size_bytes: max_i32,
        },
    )
    .await
    .map_err(db_error)?;

    let physical = storage.bucket_name(bucket);
    let presigner = storage.store.presigner();
    let instruction = match storage.upload_method {
        UploadMethod::Post => {
            let p = presigner
                .presign_post(
                    &physical,
                    &object_key,
                    format.mime(),
                    max,
                    now,
                    storage.slot_ttl,
                )
                .map_err(storage_error)?;
            UploadInstruction::Post {
                url: p.url,
                fields: p.fields,
            }
        }
        UploadMethod::Put => {
            let p = presigner
                .presign_put(
                    &physical,
                    &object_key,
                    format.mime(),
                    request.size_bytes,
                    now,
                    storage.slot_ttl,
                )
                .map_err(storage_error)?;
            UploadInstruction::Put {
                url: p.url,
                headers: p.headers,
            }
        }
    };
    tracing::info!(upload_slot_id = %id, "upload_slot_created");
    Ok(UploadTicket {
        upload_slot_id: id,
        expires_at,
        instruction,
    })
}

/// Menghapus objek mentah slot beserta semua versinya; kegagalan dicatat (pembersih mencoba lagi nanti).
async fn purge_original(
    clock: &dyn Clock,
    storage: &FileStorage,
    physical: &str,
    key: &str,
    slot_id: Uuid,
) -> bool {
    match storage
        .store
        .delete_all_versions(physical, key, clock.now())
        .await
    {
        Ok(()) => true,
        Err(e) => {
            tracing::error!(upload_slot_id = %slot_id, error = %e, "upload_object_purge_failed");
            false
        }
    }
}

/// Menghapus hasil kode ulang beserta semua versinya; kegagalan dicatat.
async fn purge_sanitized(
    clock: &dyn Clock,
    storage: &FileStorage,
    physical: &str,
    key: &str,
    slot_id: Uuid,
) -> bool {
    match storage
        .store
        .delete_all_versions(physical, key, clock.now())
        .await
    {
        Ok(()) => true,
        Err(e) => {
            tracing::error!(upload_slot_id = %slot_id, error = %e, "sanitized_object_purge_failed");
            false
        }
    }
}

/// Berkas ditolak (413/415): slot ditutup sebagai tombstone (URL lama masih berlaku sampai kedaluwarsa,
/// sehingga baris tidak dihapus sekarang), lalu berkas mentah dihapus beserta semua versinya.
async fn reject(
    pool: &PgPool,
    clock: &dyn Clock,
    storage: &FileStorage,
    slot: &repo::SlotRow,
    physical: &str,
    reason: ContentRejection,
) -> FileError {
    tracing::warn!(upload_slot_id = %slot.id, reason = reason.as_str(), "upload_rejected");
    if repo::close_slot(pool, slot.id, clock.now()).await.is_err() {
        // Pesan galat sqlx tidak dicatat (dapat memuat teks query/nilai, coding-standards §3.4).
        tracing::error!(upload_slot_id = %slot.id, "upload_slot_close_failed");
    }
    purge_original(clock, storage, physical, &slot.object_key, slot.id).await;
    match reason {
        ContentRejection::TooLarge => FileError::TooLarge,
        ContentRejection::UnsupportedType => FileError::UnsupportedType,
    }
}

/// Membatalkan klaim hasil kode ulang yang gagal diselesaikan: objek `files/` dihapus (bila gagal, slot
/// tetap mengklaimnya sehingga pembersih yang menghapus), lalu klaim dilepas supaya validasi dapat diulang.
async fn abandon_claim(
    pool: &PgPool,
    clock: &dyn Clock,
    storage: &FileStorage,
    slot_id: Uuid,
    physical: &str,
    sanitized_key: &str,
) {
    if purge_sanitized(clock, storage, physical, sanitized_key, slot_id).await
        && repo::release_claim(pool, slot_id, sanitized_key)
            .await
            .is_err()
    {
        tracing::error!(upload_slot_id = %slot_id, "upload_slot_release_failed");
    }
}

pub(super) async fn validate_upload(
    pool: &PgPool,
    clock: &dyn Clock,
    storage: &FileStorage,
    owner_account_id: Uuid,
    upload_slot_id: Uuid,
    purpose: UploadPurpose,
) -> Result<ValidatedUpload, FileError> {
    let slot = repo::find_slot(pool, upload_slot_id)
        .await
        .map_err(db_error)?
        .ok_or(FileError::NotFound)?;
    if slot.owner_account_id != Some(owner_account_id) {
        // Percobaan memakai slot milik akun lain: 404 seperti slot tidak ada (KU-03) + security log.
        access_denied(
            Some(owner_account_id),
            VALIDATE_ROUTE,
            AccessDenial::ForeignResourceId,
        );
        return Err(FileError::NotFound);
    }
    // Tombstone (sudah dipakai/ditolak) dan slot yang sudah divalidasi tidak dapat divalidasi lagi;
    // slot kedaluwarsa dibersihkan job pembersih (TD-17), bukan di jalur permintaan.
    if slot.consumed_at.is_some()
        || slot.sanitized_object_key.is_some()
        || slot.expires_at <= clock.now()
    {
        return Err(FileError::NotFound);
    }
    let max = purpose.max_size_bytes();
    if slot.bucket != purpose.bucket().as_str()
        || u64::try_from(slot.max_size_bytes).ok() != Some(max)
    {
        return Err(FileError::NotFound);
    }
    let declared = PhotoFormat::from_mime(&slot.allowed_mime_type)
        .filter(|f| purpose.allows(*f))
        .ok_or(FileError::Internal("allowed_mime_type"))?;
    let physical = storage.bucket_name(purpose.bucket());
    let store = &storage.store;

    // Ukuran dulu lewat HEAD (KEPUTUSAN-234), tanpa mengunduh berkas yang terlalu besar.
    match store
        .head_object(&physical, &slot.object_key, clock.now())
        .await
        .map_err(storage_error)?
    {
        None => return Err(not_uploaded()),
        Some(size) if size > max => {
            return Err(reject(
                pool,
                clock,
                storage,
                &slot,
                &physical,
                ContentRejection::TooLarge,
            )
            .await);
        }
        Some(_) => {}
    }
    let max_usize = usize::try_from(max).map_err(|_| FileError::Internal("max_size_bytes"))?;

    // Permit sebelum isi diunduh (TD-17): isi + dekode + kode ulang dibatasi semaphore dengan batas tunggu.
    let permit = acquire_validation_permit(storage).await?;
    let original = match store
        .get_object(&physical, &slot.object_key, max_usize, clock.now())
        .await
        .map_err(storage_error)?
    {
        GetOutcome::NotFound => return Err(not_uploaded()),
        GetOutcome::TooLarge => {
            drop(permit);
            return Err(reject(
                pool,
                clock,
                storage,
                &slot,
                &physical,
                ContentRejection::TooLarge,
            )
            .await);
        }
        GetOutcome::Body(b) => b,
    };
    // Dekode + kode ulang di thread blocking; permit dilepas saat thread selesai (domain::MAX_DECODED_BYTES).
    let sanitized = tokio::task::spawn_blocking(move || {
        let _permit = permit;
        sanitize_photo(&original, declared, max)
    })
    .await
    .map_err(|_| FileError::Internal("validation_task"))?;
    let sanitized = match sanitized {
        Ok(s) => s,
        Err(reason) => return Err(reject(pool, clock, storage, &slot, &physical, reason).await),
    };

    let size_bytes =
        i32::try_from(sanitized.bytes.len()).map_err(|_| FileError::Internal("size_bytes"))?;
    let sha256 = crypto::sha256(&sanitized.bytes);
    let object_key = format!("{FILES_PREFIX}{}", new_id().hyphenated());
    // Klaim sebelum objek ditulis: objek `files/` tidak pernah ada tanpa catatan di database, dan dari
    // validasi bersamaan atas slot yang sama hanya satu yang menang.
    if !repo::claim_slot(pool, slot.id, owner_account_id, &object_key, clock.now())
        .await
        .map_err(db_error)?
    {
        return Err(FileError::NotFound);
    }
    if let Err(e) = store
        .put_object(
            &physical,
            &object_key,
            sanitized.format.mime(),
            sanitized.bytes,
            clock.now(),
        )
        .await
    {
        abandon_claim(pool, clock, storage, slot.id, &physical, &object_key).await;
        return Err(storage_error(e));
    }
    // Berkas asli (dengan metadata) dihapus beserta semua versinya; bila gagal, hasil kode ulang ikut
    // dibuang dan klaim dilepas sehingga konfirmasi dapat diulang.
    if let Err(e) = store
        .delete_all_versions(&physical, &slot.object_key, clock.now())
        .await
    {
        tracing::error!(upload_slot_id = %slot.id, error = %e, "upload_object_purge_failed");
        abandon_claim(pool, clock, storage, slot.id, &physical, &object_key).await;
        return Err(storage_error(e));
    }
    tracing::info!(upload_slot_id = %slot.id, "upload_validated");
    Ok(ValidatedUpload {
        upload_slot_id: slot.id,
        owner_account_id,
        purpose,
        object_key,
        format: sanitized.format,
        size_bytes,
        sha256,
        validated_at: clock.now(),
        settled: AtomicBool::new(false),
    })
}

pub(super) async fn record_stored_file(
    conn: &mut PgConnection,
    clock: &dyn Clock,
    upload: &ValidatedUpload,
    container: Option<FileContainer>,
) -> Result<Uuid, FileError> {
    upload.mark_settled();
    if !repo::consume_slot(
        conn,
        upload.upload_slot_id,
        upload.owner_account_id,
        &upload.object_key,
        clock.now(),
    )
    .await
    .map_err(db_error)?
    {
        return Err(FileError::NotFound);
    }
    let id = new_id();
    repo::insert_stored_file(
        conn,
        &repo::NewStoredFile {
            id,
            actor_account_id: upload.owner_account_id,
            bucket: upload.purpose.bucket().as_str(),
            object_key: &upload.object_key,
            category: upload.purpose.category(),
            owner_account_id: upload.owner_account_id,
            container_type: container.map(|c| c.container_type.as_str()),
            container_id: container.map(|c| c.container_id),
            mime_type: upload.format.mime(),
            size_bytes: upload.size_bytes,
            sha256: &upload.sha256,
            validated_at: upload.validated_at,
        },
    )
    .await
    .map_err(db_error)?;
    Ok(id)
}

pub(super) async fn discard_validated(
    clock: &dyn Clock,
    storage: &FileStorage,
    upload: ValidatedUpload,
) -> Result<(), FileError> {
    upload.mark_settled();
    let physical = storage.bucket_name(upload.purpose.bucket());
    storage
        .store
        .delete_all_versions(&physical, &upload.object_key, clock.now())
        .await
        .map_err(storage_error)
}

pub(super) async fn purge_expired_upload_slots(
    pool: &PgPool,
    clock: &dyn Clock,
    storage: &FileStorage,
    limit: u16,
) -> Result<SlotPurgeReport, FileError> {
    let cutoff = clock
        .now()
        .checked_sub(grace()?)
        .ok_or(FileError::Internal("grace"))?;
    let rows = repo::expired_slots(pool, cutoff, i64::from(limit))
        .await
        .map_err(db_error)?;
    let mut report = SlotPurgeReport::default();
    for row in rows {
        let Some(bucket) = LogicalBucket::parse(&row.bucket) else {
            tracing::error!(upload_slot_id = %row.id, "upload_slot_bucket_unknown");
            report.failed += 1;
            continue;
        };
        let physical = storage.bucket_name(bucket);
        // Objek mentah: URL unggah sudah kedaluwarsa + jeda, jadi penghapusan ini final.
        if !purge_original(clock, storage, &physical, &row.object_key, row.id).await {
            report.failed += 1;
            continue;
        }
        // Hasil kode ulang yang tidak pernah dicatat (transaksi pemanggil gagal, proses mati, atau
        // ValidatedUpload dibuang tanpa discard_validated).
        if let Some(key) = row.sanitized_object_key.as_deref()
            && !row.recorded
            && !purge_sanitized(clock, storage, &physical, key, row.id).await
        {
            report.failed += 1;
            continue;
        }
        match repo::delete_expired_slot(pool, row.id, cutoff).await {
            Ok(_) => report.purged += 1,
            Err(_) => {
                tracing::error!(upload_slot_id = %row.id, "upload_slot_delete_failed");
                report.failed += 1;
            }
        }
    }
    if report.purged > 0 || report.failed > 0 {
        tracing::info!(
            purged = report.purged,
            failed = report.failed,
            "upload_slots_purged"
        );
    }
    Ok(report)
}

/// Kunci yang ada di `keys` tetapi tidak di `known`.
fn unknown_keys(keys: Vec<String>, known: &[String]) -> Vec<String> {
    keys.into_iter().filter(|k| !known.contains(k)).collect()
}

/// Menghapus objek yatim di bawah satu prefiks. `known` mengembalikan kunci yang masih tercatat.
async fn reconcile_prefix(
    pool: &PgPool,
    clock: &dyn Clock,
    storage: &FileStorage,
    physical: &str,
    prefix: &str,
    report: &mut ReconcileReport,
) -> Result<(), FileError> {
    let mut cursor: Option<VersionListCursor> = None;
    for _ in 0..RECONCILE_MAX_PAGES {
        let page = storage
            .store
            .list_version_keys(physical, prefix, cursor.as_ref(), clock.now())
            .await
            .map_err(storage_error)?;
        if !page.keys.is_empty() {
            let known = if prefix == INCOMING_PREFIX {
                repo::known_incoming_keys(pool, &page.keys).await
            } else {
                repo::known_file_keys(pool, &page.keys).await
            }
            .map_err(db_error)?;
            for key in unknown_keys(page.keys, &known) {
                match storage
                    .store
                    .delete_all_versions(physical, &key, clock.now())
                    .await
                {
                    Ok(()) => report.orphans_deleted += 1,
                    Err(e) => {
                        tracing::error!(prefix, error = %e, "orphan_object_purge_failed");
                        report.orphans_failed += 1;
                    }
                }
            }
        }
        match page.next {
            Some(next) => cursor = Some(next),
            None => return Ok(()),
        }
    }
    tracing::warn!(prefix, "reconcile_page_limit_reached");
    Ok(())
}

pub(super) async fn reconcile_bucket(
    pool: &PgPool,
    clock: &dyn Clock,
    storage: &FileStorage,
    bucket: LogicalBucket,
) -> Result<ReconcileReport, FileError> {
    let physical = storage.bucket_name(bucket);
    let mut report = ReconcileReport::default();
    for prefix in [INCOMING_PREFIX, FILES_PREFIX] {
        reconcile_prefix(pool, clock, storage, &physical, prefix, &mut report).await?;
    }
    // Arah database → bucket: berkas tercatat yang objeknya hilang (integritas; tidak mengubah data).
    let mut after: Option<Uuid> = None;
    loop {
        let rows = repo::live_files_page(pool, bucket.as_str(), after, RECONCILE_DB_PAGE)
            .await
            .map_err(db_error)?;
        let Some(last) = rows.last() else {
            break;
        };
        after = Some(last.id);
        for row in &rows {
            let present = storage
                .store
                .head_object(&physical, &row.object_key, clock.now())
                .await
                .map_err(storage_error)?;
            if present.is_none() {
                tracing::error!(stored_file_id = %row.id, "stored_file_object_missing");
                report.missing_objects += 1;
            }
        }
    }
    tracing::info!(
        bucket = bucket.as_str(),
        orphans_deleted = report.orphans_deleted,
        orphans_failed = report.orphans_failed,
        missing_objects = report.missing_objects,
        "file_bucket_reconciled"
    );
    Ok(report)
}

/// Kebijakan audit objek `stored_file` (dokumen: hanya nama field, PRD §7.3). Akses baca tidak mengubah
/// field, sehingga daftar field kosong.
static STORED_FILE_POLICY: ObjectPolicy = ObjectPolicy::clinical_or_document("stored_file", &[]);
/// Nama aksi E-05 membuka berkas.
const FILE_OPENED_ACTION: &str = "stored_file_opened";
/// Kode alasan E-05 hasil gagal.
const ACCESS_DENIED_REASON: &str = "access_denied";
/// Percobaan menulis E-05 hasil gagal (coding-standards §4.4 butir 3: "dicoba ulang").
const DENIED_AUDIT_ATTEMPTS: usize = 3;
/// `Cache-Control` URL unduh: tidak disimpan cache peramban atau perantara.
const DOWNLOAD_CACHE_CONTROL: &str = "private, no-store";

fn audit_error<E>(_e: AuditedTxError<E>) -> FileError {
    FileError::Internal("audit")
}

fn actor(principal: &Principal) -> Result<Actor, FileError> {
    let account_type =
        Code::new(principal.account_type().as_str()).map_err(|_| FileError::Internal("actor"))?;
    let roles = principal
        .sub_roles()
        .iter()
        .map(|r| Code::new(r.as_str()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| FileError::Internal("actor"))?;
    Ok(Actor::account(principal.account_id(), account_type, roles))
}

fn file_event(
    principal: &Principal,
    audit: &AuditContext,
    file_id: Uuid,
    subject: Option<Subject>,
    outcome: Outcome,
) -> Result<AuditEvent, FileError> {
    let action = Code::new(FILE_OPENED_ACTION).map_err(|_| FileError::Internal("audit_action"))?;
    let mut event = AuditEvent::new(
        EventCode::E05,
        action,
        actor(principal)?,
        ObjectRef::with_uuid(&STORED_FILE_POLICY, file_id),
    )
    .outcome(outcome)
    .context(audit);
    if let Some(s) = subject {
        event = event.subject(s);
    }
    Ok(event)
}

async fn write_events(
    pool: &PgPool,
    clock: &dyn Clock,
    events: Vec<AuditEvent>,
) -> Result<(), AuditedTxError<Infallible>> {
    in_audited_tx(pool, clock, move |_conn| {
        Box::pin(async move { Ok::<_, Infallible>(((), events)) })
    })
    .await
}

/// Percobaan akses yang ditolak atas berkas yang ada: E-05 hasil gagal di transaksi tersendiri, dicoba
/// ulang; kegagalan tidak mengubah jawaban 404 (coding-standards §4.4 butir 3).
async fn record_denied(
    pool: &PgPool,
    clock: &dyn Clock,
    principal: &Principal,
    audit: &AuditContext,
    file_id: Uuid,
    subject: Option<Subject>,
) {
    let Ok(reason) = Code::new(ACCESS_DENIED_REASON) else {
        return;
    };
    for _ in 0..DENIED_AUDIT_ATTEMPTS {
        let Ok(event) = file_event(
            principal,
            audit,
            file_id,
            subject,
            Outcome::failure(reason.clone()),
        ) else {
            break;
        };
        if write_events(pool, clock, vec![event]).await.is_ok() {
            return;
        }
    }
    tracing::error!(stored_file_id = %file_id, "file_access_denied_audit_failed");
}

fn extension(mime: &str) -> &'static str {
    match mime {
        "image/jpeg" => "jpg",
        "image/png" => "png",
        "application/pdf" => "pdf",
        "text/csv" => "csv",
        "application/json" => "json",
        _ => "bin",
    }
}

fn may_access(principal: &Principal, access: FileAccess, file: &repo::FileForDownload) -> bool {
    match access {
        FileAccess::Owner => file.owner_account_id == Some(principal.account_id()),
        FileAccess::Staff { roles, .. } => {
            principal.account_type() == AccountType::RsAdmin
                && roles.iter().any(|r| principal.has_sub_role(*r))
        }
    }
}

pub(super) async fn issue_download_url(
    pool: &PgPool,
    clock: &dyn Clock,
    storage: &FileStorage,
    principal: &Principal,
    audit: &AuditContext,
    request: DownloadRequest<'_>,
) -> Result<DownloadTicket, FileError> {
    let purpose = request.purpose;
    let found = repo::find_file_for_download(pool, request.stored_file_id)
        .await
        .map_err(db_error)?
        // Berkas kategori atau bucket lain diperlakukan seperti tidak ada (tanpa security log: bukan
        // percobaan atas objek milik pihak lain lewat endpoint ini).
        .filter(|f| f.category == purpose.category() && f.bucket == purpose.bucket().as_str());
    let exists = found.is_some();
    let subject = match request.access {
        FileAccess::Staff { subject, .. } => Some(subject),
        FileAccess::Owner => None,
    };
    let file = match visible_object(principal, request.route, found, |f| {
        may_access(principal, request.access, f)
    }) {
        Ok(f) => f,
        Err(_) => {
            if exists {
                record_denied(
                    pool,
                    clock,
                    principal,
                    audit,
                    request.stored_file_id,
                    subject,
                )
                .await;
            }
            return Err(FileError::NotFound);
        }
    };
    if let FileAccess::Staff { subject, .. } = request.access {
        let event = file_event(principal, audit, file.id, Some(subject), Outcome::success())?;
        // E-05 di-commit sebelum URL dibuat; gagal → akses ditolak (coding-standards §4.4 butir 2).
        write_events(pool, clock, vec![event])
            .await
            .map_err(audit_error)?;
    }
    let bucket = LogicalBucket::parse(&file.bucket).ok_or(FileError::Internal("bucket"))?;
    let disposition = format!(
        "attachment; filename=\"{}.{}\"",
        file.id.hyphenated(),
        extension(&file.mime_type)
    );
    let presigned = storage
        .store
        .presigner()
        .presign_get(
            &storage.bucket_name(bucket),
            &file.object_key,
            &ResponseHeaders {
                content_type: &file.mime_type,
                content_disposition: &disposition,
                cache_control: DOWNLOAD_CACHE_CONTROL,
            },
            clock.now(),
            DOWNLOAD_URL_TTL,
        )
        .map_err(storage_error)?;
    tracing::info!(stored_file_id = %file.id, "download_url_issued");
    Ok(DownloadTicket {
        url: presigned.url,
        expires_at: presigned.expires_at,
        mime_type: file.mime_type,
        size_bytes: file.size_bytes,
    })
}

fn job_error(_e: platform_jobs::JobError) -> FileError {
    FileError::Internal("job")
}

pub(super) async fn enqueue_file_maintenance(
    pool: &PgPool,
    clock: &dyn Clock,
) -> Result<(), FileError> {
    let now = clock.now();
    let attempts = MaxAttempts::new(MAINTENANCE_MAX_ATTEMPTS).map_err(job_error)?;
    let purge = NewJob::new(
        JobType::new(UPLOAD_SLOT_PURGE_JOB).map_err(job_error)?,
        JobPayload::new(),
        now,
        attempts,
    )
    .dedupe_key(DedupeKey::parse(UPLOAD_SLOT_PURGE_JOB).map_err(job_error)?);
    enqueue(pool, &purge).await.map_err(job_error)?;
    for bucket in LogicalBucket::ALL {
        let job = NewJob::new(
            JobType::new(FILE_RECONCILE_JOB).map_err(job_error)?,
            JobPayload::new()
                .with_code("bucket", bucket.as_str())
                .map_err(job_error)?,
            now,
            attempts,
        )
        .dedupe_key(
            DedupeKey::parse(&format!("{FILE_RECONCILE_JOB}:{}", bucket.as_str()))
                .map_err(job_error)?,
        );
        enqueue(pool, &job).await.map_err(job_error)?;
    }
    Ok(())
}

/// Kode galat job tetap (diuji sah di unit test). Bila tetap tidak sah, job ditandai selesai dan galat
/// dicatat: job pemeliharaan diantrekan ulang berkala, jadi tidak ada pekerjaan yang hilang.
fn outcome(code: &str, retryable: bool) -> JobOutcome {
    match ErrorCode::new(code) {
        Ok(c) if retryable => JobOutcome::Retry(c),
        Ok(c) => JobOutcome::Fail(c),
        Err(_) => {
            tracing::error!("file_maintenance_error_code_invalid");
            JobOutcome::Succeeded
        }
    }
}

fn retry(code: &str) -> JobOutcome {
    outcome(code, true)
}

fn fail(code: &str) -> JobOutcome {
    outcome(code, false)
}

fn maintenance_error_code(e: &FileError) -> &'static str {
    match e {
        FileError::StorageUnavailable | FileError::Busy => "object_storage_unavailable",
        _ => "file_maintenance_failed",
    }
}

pub(super) async fn run_file_maintenance_job(
    pool: &PgPool,
    clock: &dyn Clock,
    storage: &FileStorage,
    job: &ClaimedJob,
) -> JobOutcome {
    let Ok(payload) = job.payload() else {
        return fail(ErrorCode::INVALID_PAYLOAD);
    };
    match job.job_type().as_str() {
        UPLOAD_SLOT_PURGE_JOB => {
            match purge_expired_upload_slots(pool, clock, storage, UPLOAD_SLOT_PURGE_BATCH).await {
                // Sisa slot (lebih dari satu batch atau yang gagal) diproses putaran job berikutnya.
                Ok(r) if r.failed == 0 => JobOutcome::Succeeded,
                Ok(_) => retry("upload_slot_purge_incomplete"),
                Err(e) => retry(maintenance_error_code(&e)),
            }
        }
        FILE_RECONCILE_JOB => {
            let Some(bucket) = payload.code("bucket").and_then(LogicalBucket::parse) else {
                return fail(ErrorCode::INVALID_PAYLOAD);
            };
            match reconcile_bucket(pool, clock, storage, bucket).await {
                Ok(r) if r.orphans_failed == 0 => JobOutcome::Succeeded,
                Ok(_) => retry("orphan_purge_incomplete"),
                Err(e) => retry(maintenance_error_code(&e)),
            }
        }
        _ => fail("unknown_job_type"),
    }
}

pub(super) async fn read_public_content_file(
    pool: &PgPool,
    clock: &dyn Clock,
    storage: &FileStorage,
    stored_file_id: Uuid,
) -> Result<Option<PublicFile>, FileError> {
    let Some(row) = repo::find_public_content_file(pool, stored_file_id)
        .await
        .map_err(db_error)?
    else {
        return Ok(None);
    };
    let Some(format) = PhotoFormat::from_mime(&row.mime_type) else {
        return Ok(None);
    };
    // Batas baca = ukuran tercatat (hasil kode ulang), dibatasi batas foto x2 sebagai pengaman memori.
    let max = usize::try_from(row.size_bytes)
        .ok()
        .zip(usize::try_from(MAX_PHOTO_BYTES.saturating_mul(2)).ok())
        .map(|(size, cap)| size.min(cap))
        .ok_or(FileError::Internal("size_bytes"))?;
    let physical = storage.bucket_name(LogicalBucket::PublicContent);
    match storage
        .store
        .get_object(&physical, &row.object_key, max, clock.now())
        .await
        .map_err(storage_error)?
    {
        GetOutcome::Body(bytes) => Ok(Some(PublicFile {
            mime_type: format.mime(),
            bytes,
        })),
        GetOutcome::NotFound => {
            tracing::error!(stored_file_id = %stored_file_id, "public_content_object_missing");
            Ok(None)
        }
        GetOutcome::TooLarge => Err(FileError::Internal("public_content_size")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_keys_keeps_only_untracked() {
        let keys = vec!["files/a".to_string(), "files/b".to_string()];
        assert_eq!(
            unknown_keys(keys, &["files/b".to_string()]),
            vec!["files/a".to_string()]
        );
    }

    #[test]
    fn stored_file_audit_policy_is_a_valid_document_policy() {
        assert!(STORED_FILE_POLICY.validate().is_ok());
        assert_eq!(STORED_FILE_POLICY.object_type(), "stored_file");
        assert!(Code::new(FILE_OPENED_ACTION).is_ok());
        assert!(Code::new(ACCESS_DENIED_REASON).is_ok());
    }

    #[test]
    fn download_extension_follows_validated_mime_type() {
        assert_eq!(extension("image/jpeg"), "jpg");
        assert_eq!(extension("image/png"), "png");
        assert_eq!(extension("application/pdf"), "pdf");
        assert_eq!(extension("text/html"), "bin");
    }

    #[test]
    fn tombstone_grace_is_fifteen_minutes() {
        assert_eq!(grace().ok(), Some(time::Duration::minutes(15)));
    }

    #[test]
    fn maintenance_job_codes_are_valid() {
        for code in [
            "upload_slot_purge_incomplete",
            "orphan_purge_incomplete",
            "object_storage_unavailable",
            "file_maintenance_failed",
            "unknown_job_type",
        ] {
            assert!(ErrorCode::new(code).is_ok(), "{code}");
        }
        assert!(JobType::new(UPLOAD_SLOT_PURGE_JOB).is_ok());
        assert!(JobType::new(FILE_RECONCILE_JOB).is_ok());
        for b in LogicalBucket::ALL {
            assert!(DedupeKey::parse(&format!("{FILE_RECONCILE_JOB}:{}", b.as_str())).is_ok());
        }
    }
}
