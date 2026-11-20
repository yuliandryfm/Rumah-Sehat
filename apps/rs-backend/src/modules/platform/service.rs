//! Use case modul `platform`: kesiapan (F1-T11) dan berkas unggahan (F1-T29, ADR-018).
//!
//! Berkas: tidak ada panggilan object storage selama transaksi database terbuka (KEPUTUSAN-125). Isi berkas,
//! URL bertanda tangan, dan kunci objek tidak pernah dicatat di log; yang dicatat hanya ID slot dan kode
//! alasan (ADR-010).

use super::api::{
    FileContainer, FileStorage, PublicFile, ReadinessCheck, UploadInstruction, UploadMethod,
    UploadSlotRequest, UploadTicket, ValidatedUpload,
};
use super::domain::{
    ContentRejection, LogicalBucket, MAX_PHOTO_BYTES, PhotoFormat, UploadPurpose, sanitize_photo,
};
use super::error::FileError;
use super::repo;
use envelope::FieldError;
use foundation::{Clock, Uuid, new_id};
use object_storage::{GetOutcome, ObjectStorageError};
use observability::security::{AccessDenial, access_denied};
use sqlx::{PgConnection, PgPool};
use std::sync::Arc;

/// Label use case untuk security log (tidak ada rute HTTP di task ini; endpoint dibuat modul pemakai).
const VALIDATE_ROUTE: &str = "platform.validate_upload";

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
    let object_key = new_id().hyphenated().to_string();
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

/// Membuang berkas asli slot beserta semua versinya, lalu baris slotnya. Bila objek gagal dihapus, baris
/// slot dibiarkan supaya pembersih slot kedaluwarsa dapat mencoba lagi (ADR-018 butir 2).
async fn discard_slot(
    pool: &PgPool,
    clock: &dyn Clock,
    storage: &FileStorage,
    slot: &repo::SlotRow,
) {
    let Some(bucket) = super::domain::LogicalBucket::parse(&slot.bucket) else {
        tracing::error!(upload_slot_id = %slot.id, "upload_slot_bucket_unknown");
        return;
    };
    let physical = storage.bucket_name(bucket);
    match storage
        .store
        .delete_all_versions(&physical, &slot.object_key, clock.now())
        .await
    {
        Ok(()) => {
            // Pesan galat sqlx tidak dicatat (dapat memuat teks query/nilai, coding-standards §3.4).
            if repo::delete_slot(pool, slot.id).await.is_err() {
                tracing::error!(upload_slot_id = %slot.id, "upload_slot_delete_failed");
            }
        }
        Err(e) => {
            tracing::error!(upload_slot_id = %slot.id, error = %e, "upload_object_purge_failed");
        }
    }
}

async fn reject(
    pool: &PgPool,
    clock: &dyn Clock,
    storage: &FileStorage,
    slot: &repo::SlotRow,
    reason: ContentRejection,
) -> FileError {
    tracing::warn!(upload_slot_id = %slot.id, reason = reason.as_str(), "upload_rejected");
    discard_slot(pool, clock, storage, slot).await;
    match reason {
        ContentRejection::TooLarge => FileError::TooLarge,
        ContentRejection::UnsupportedType => FileError::UnsupportedType,
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
    if slot.expires_at <= clock.now() {
        discard_slot(pool, clock, storage, &slot).await;
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
            return Err(reject(pool, clock, storage, &slot, ContentRejection::TooLarge).await);
        }
        Some(_) => {}
    }
    let max_usize = usize::try_from(max).map_err(|_| FileError::Internal("max_size_bytes"))?;
    let original = match store
        .get_object(&physical, &slot.object_key, max_usize, clock.now())
        .await
        .map_err(storage_error)?
    {
        GetOutcome::NotFound => return Err(not_uploaded()),
        GetOutcome::TooLarge => {
            return Err(reject(pool, clock, storage, &slot, ContentRejection::TooLarge).await);
        }
        GetOutcome::Body(b) => b,
    };

    // Dekode + kode ulang di thread blocking, dibatasi semaphore (memori dekoder, domain::MAX_DECODE_ALLOC).
    let permit = storage
        .permits
        .clone()
        .acquire_owned()
        .await
        .map_err(|_| FileError::Internal("validation_permit"))?;
    let sanitized = tokio::task::spawn_blocking(move || {
        let _permit = permit;
        sanitize_photo(&original, declared, max)
    })
    .await
    .map_err(|_| FileError::Internal("validation_task"))?;
    let sanitized = match sanitized {
        Ok(s) => s,
        Err(reason) => return Err(reject(pool, clock, storage, &slot, reason).await),
    };

    let size_bytes =
        i32::try_from(sanitized.bytes.len()).map_err(|_| FileError::Internal("size_bytes"))?;
    let sha256 = crypto::sha256(&sanitized.bytes);
    let object_key = new_id().hyphenated().to_string();
    store
        .put_object(
            &physical,
            &object_key,
            sanitized.format.mime(),
            sanitized.bytes,
            clock.now(),
        )
        .await
        .map_err(storage_error)?;
    // Berkas asli (dengan metadata) dihapus beserta semua versinya; bila gagal, hasil kode ulang ikut
    // dibuang dan slot tetap ada sehingga konfirmasi dapat diulang.
    if let Err(e) = store
        .delete_all_versions(&physical, &slot.object_key, clock.now())
        .await
    {
        tracing::error!(upload_slot_id = %slot.id, error = %e, "upload_object_purge_failed");
        if let Err(e2) = store
            .delete_all_versions(&physical, &object_key, clock.now())
            .await
        {
            tracing::error!(upload_slot_id = %slot.id, error = %e2, "sanitized_object_purge_failed");
        }
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
    })
}

pub(super) async fn record_stored_file(
    conn: &mut PgConnection,
    upload: &ValidatedUpload,
    container: Option<FileContainer>,
) -> Result<Uuid, FileError> {
    if !repo::consume_slot(conn, upload.upload_slot_id, upload.owner_account_id)
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
    let physical = storage.bucket_name(upload.purpose.bucket());
    storage
        .store
        .delete_all_versions(&physical, &upload.object_key, clock.now())
        .await
        .map_err(storage_error)
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
