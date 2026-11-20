//! Query sqlx atas tabel milik modul `platform`: `upload_slot` (A19.5, kelas O) dan `stored_file`
//! (A19.6, kelas M). Hak `rs_app`: upload_slot S/I/U/D; stored_file S/I/U tanpa DELETE (ADR-003).
//!
//! Siklus slot (TD-17): dibuat → (divalidasi: `sanitized_object_key` diklaim) → dipakai atau ditolak
//! (`consumed_at`, tombstone) → dihapus pembersih setelah `expires_at` + jeda, bersama semua versi objek
//! mentahnya. Selama tombstone, URL unggah lama yang masih berlaku tidak dapat lagi menghasilkan berkas.

use foundation::Uuid;
use sqlx::{PgConnection, PgPool};
use time::OffsetDateTime;

/// Baris `upload_slot`.
pub(super) struct SlotRow {
    pub(super) id: Uuid,
    pub(super) expires_at: OffsetDateTime,
    pub(super) owner_account_id: Option<Uuid>,
    pub(super) bucket: String,
    pub(super) object_key: String,
    pub(super) allowed_mime_type: String,
    pub(super) max_size_bytes: i32,
    pub(super) consumed_at: Option<OffsetDateTime>,
    pub(super) sanitized_object_key: Option<String>,
}

/// Slot baru.
pub(super) struct NewSlot<'a> {
    pub(super) id: Uuid,
    pub(super) expires_at: OffsetDateTime,
    pub(super) owner_account_id: Uuid,
    pub(super) bucket: &'a str,
    pub(super) object_key: &'a str,
    pub(super) allowed_mime_type: &'a str,
    pub(super) max_size_bytes: i32,
}

pub(super) async fn insert_slot(pool: &PgPool, slot: &NewSlot<'_>) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO app.upload_slot \
           (id, expires_at, owner_account_id, bucket, object_key, allowed_mime_type, max_size_bytes) \
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
        slot.id,
        slot.expires_at,
        slot.owner_account_id,
        slot.bucket,
        slot.object_key,
        slot.allowed_mime_type,
        slot.max_size_bytes,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub(super) async fn find_slot(pool: &PgPool, id: Uuid) -> Result<Option<SlotRow>, sqlx::Error> {
    sqlx::query_as!(
        SlotRow,
        "SELECT id, expires_at, owner_account_id, bucket, object_key, allowed_mime_type, max_size_bytes, \
                consumed_at, sanitized_object_key \
         FROM app.upload_slot WHERE id = $1",
        id,
    )
    .fetch_optional(pool)
    .await
}

/// Mengklaim slot untuk satu hasil kode ulang sebelum objeknya ditulis, supaya objek di prefiks `files/`
/// selalu tercatat (rekonsiliasi) dan validasi bersamaan atas slot yang sama hanya satu yang menang.
/// Hanya berhasil bila slot milik `owner`, belum dipakai, belum diklaim, dan belum kedaluwarsa pada `now`.
pub(super) async fn claim_slot(
    pool: &PgPool,
    id: Uuid,
    owner: Uuid,
    sanitized_object_key: &str,
    now: OffsetDateTime,
) -> Result<bool, sqlx::Error> {
    let r = sqlx::query!(
        "UPDATE app.upload_slot SET sanitized_object_key = $3 \
         WHERE id = $1 AND owner_account_id = $2 AND consumed_at IS NULL \
           AND sanitized_object_key IS NULL AND expires_at > $4",
        id,
        owner,
        sanitized_object_key,
        now,
    )
    .execute(pool)
    .await?;
    Ok(r.rows_affected() == 1)
}

/// Melepas klaim setelah objek hasil kode ulang gagal ditulis atau dibuang, sehingga validasi dapat
/// diulang selama berkas mentahnya masih ada.
pub(super) async fn release_claim(
    pool: &PgPool,
    id: Uuid,
    sanitized_object_key: &str,
) -> Result<bool, sqlx::Error> {
    let r = sqlx::query!(
        "UPDATE app.upload_slot SET sanitized_object_key = NULL \
         WHERE id = $1 AND sanitized_object_key = $2 AND consumed_at IS NULL",
        id,
        sanitized_object_key,
    )
    .execute(pool)
    .await?;
    Ok(r.rows_affected() == 1)
}

/// Menutup slot yang berkasnya ditolak (tombstone): tidak dapat divalidasi atau dipakai lagi; baris tetap
/// ada sampai pembersih menghapusnya.
pub(super) async fn close_slot(
    pool: &PgPool,
    id: Uuid,
    now: OffsetDateTime,
) -> Result<bool, sqlx::Error> {
    let r = sqlx::query!(
        "UPDATE app.upload_slot SET consumed_at = $2 WHERE id = $1 AND consumed_at IS NULL",
        id,
        now,
    )
    .execute(pool)
    .await?;
    Ok(r.rows_affected() == 1)
}

/// Memakai slot sekali di transaksi pemanggil (tombstone, bukan DELETE): hanya bila masih milik `owner`,
/// belum dipakai, klaimnya untuk `sanitized_object_key` ini, dan belum kedaluwarsa pada `now`. Syarat
/// kedaluwarsa menjamin pembersih (yang hanya menyentuh slot lewat `expires_at` + jeda) tidak pernah
/// menghapus berkas yang baru dicatat.
pub(super) async fn consume_slot(
    conn: &mut PgConnection,
    id: Uuid,
    owner: Uuid,
    sanitized_object_key: &str,
    now: OffsetDateTime,
) -> Result<bool, sqlx::Error> {
    let r = sqlx::query!(
        "UPDATE app.upload_slot SET consumed_at = $4 \
         WHERE id = $1 AND owner_account_id = $2 AND consumed_at IS NULL \
           AND sanitized_object_key = $3 AND expires_at > $4",
        id,
        owner,
        sanitized_object_key,
        now,
    )
    .execute(&mut *conn)
    .await?;
    Ok(r.rows_affected() == 1)
}

/// Slot yang lewat batas pembersihan, beserta apakah hasil kode ulangnya sudah dicatat di `stored_file`.
pub(super) struct ExpiredSlotRow {
    pub(super) id: Uuid,
    pub(super) bucket: String,
    pub(super) object_key: String,
    pub(super) sanitized_object_key: Option<String>,
    pub(super) recorded: bool,
}

pub(super) async fn expired_slots(
    pool: &PgPool,
    cutoff: OffsetDateTime,
    limit: i64,
) -> Result<Vec<ExpiredSlotRow>, sqlx::Error> {
    sqlx::query_as!(
        ExpiredSlotRow,
        r#"SELECT s.id, s.bucket, s.object_key, s.sanitized_object_key,
                  EXISTS (SELECT 1 FROM app.stored_file f WHERE f.object_key = s.sanitized_object_key)
                    AS "recorded!"
           FROM app.upload_slot s
           WHERE s.expires_at < $1
           ORDER BY s.expires_at
           LIMIT $2"#,
        cutoff,
        limit,
    )
    .fetch_all(pool)
    .await
}

/// Menghapus baris slot yang sudah lewat batas pembersihan (setelah objeknya dihapus).
pub(super) async fn delete_expired_slot(
    pool: &PgPool,
    id: Uuid,
    cutoff: OffsetDateTime,
) -> Result<bool, sqlx::Error> {
    let r = sqlx::query!(
        "DELETE FROM app.upload_slot WHERE id = $1 AND expires_at < $2",
        id,
        cutoff,
    )
    .execute(pool)
    .await?;
    Ok(r.rows_affected() == 1)
}

/// Kunci objek mentah (`incoming/`) yang masih punya slot.
pub(super) async fn known_incoming_keys(
    pool: &PgPool,
    keys: &[String],
) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar!(
        "SELECT object_key FROM app.upload_slot WHERE object_key = ANY($1)",
        keys,
    )
    .fetch_all(pool)
    .await
}

/// Kunci objek berkas (`files/`) yang tercatat di `stored_file` atau masih diklaim slot.
pub(super) async fn known_file_keys(
    pool: &PgPool,
    keys: &[String],
) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar!(
        r#"SELECT object_key AS "object_key!" FROM app.stored_file WHERE object_key = ANY($1)
           UNION
           SELECT sanitized_object_key AS "object_key!" FROM app.upload_slot
           WHERE sanitized_object_key = ANY($1)"#,
        keys,
    )
    .fetch_all(pool)
    .await
}

/// Baris `stored_file` yang objeknya seharusnya ada (belum dimusnahkan), untuk rekonsiliasi arah
/// database → bucket.
pub(super) struct LiveFileRow {
    pub(super) id: Uuid,
    pub(super) object_key: String,
}

pub(super) async fn live_files_page(
    pool: &PgPool,
    bucket: &str,
    after: Option<Uuid>,
    limit: i64,
) -> Result<Vec<LiveFileRow>, sqlx::Error> {
    sqlx::query_as!(
        LiveFileRow,
        "SELECT id, object_key FROM app.stored_file \
         WHERE bucket = $1 AND purged_at IS NULL AND ($2::uuid IS NULL OR id > $2) \
         ORDER BY id LIMIT $3",
        bucket,
        after,
        limit,
    )
    .fetch_all(pool)
    .await
}

/// Berkas yang dapat diunduh (belum dihapus lunak dan belum dimusnahkan, ADR-003, ADR-018 butir 5).
pub(super) struct FileForDownload {
    pub(super) id: Uuid,
    pub(super) bucket: String,
    pub(super) object_key: String,
    pub(super) category: String,
    pub(super) owner_account_id: Option<Uuid>,
    pub(super) mime_type: String,
    pub(super) size_bytes: i32,
}

pub(super) async fn find_file_for_download(
    pool: &PgPool,
    id: Uuid,
) -> Result<Option<FileForDownload>, sqlx::Error> {
    sqlx::query_as!(
        FileForDownload,
        "SELECT id, bucket, object_key, category, owner_account_id, mime_type, size_bytes \
         FROM app.stored_file WHERE id = $1 AND deleted_at IS NULL AND purged_at IS NULL",
        id,
    )
    .fetch_optional(pool)
    .await
}

/// Baris `stored_file` baru (lolos validasi).
pub(super) struct NewStoredFile<'a> {
    pub(super) id: Uuid,
    pub(super) actor_account_id: Uuid,
    pub(super) bucket: &'a str,
    pub(super) object_key: &'a str,
    pub(super) category: &'a str,
    pub(super) owner_account_id: Uuid,
    pub(super) container_type: Option<&'a str>,
    pub(super) container_id: Option<Uuid>,
    pub(super) mime_type: &'a str,
    pub(super) size_bytes: i32,
    pub(super) sha256: &'a [u8],
    pub(super) validated_at: OffsetDateTime,
}

pub(super) async fn insert_stored_file(
    conn: &mut PgConnection,
    f: &NewStoredFile<'_>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO app.stored_file \
           (id, created_by, updated_by, bucket, object_key, category, owner_account_id, \
            container_type, container_id, mime_type, size_bytes, sha256, validated_at) \
         VALUES ($1, $2, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)",
        f.id,
        f.actor_account_id,
        f.bucket,
        f.object_key,
        f.category,
        f.owner_account_id,
        f.container_type,
        f.container_id,
        f.mime_type,
        f.size_bytes,
        f.sha256,
        f.validated_at,
    )
    .execute(&mut *conn)
    .await?;
    Ok(())
}
