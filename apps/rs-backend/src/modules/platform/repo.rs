//! Query sqlx atas tabel milik modul `platform`: `upload_slot` (A19.5, kelas O) dan `stored_file`
//! (A19.6, kelas M). Hak `rs_app`: upload_slot S/I/U/D; stored_file S/I/U tanpa DELETE (ADR-003).

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
        "SELECT id, expires_at, owner_account_id, bucket, object_key, allowed_mime_type, max_size_bytes \
         FROM app.upload_slot WHERE id = $1",
        id,
    )
    .fetch_optional(pool)
    .await
}

/// Menghapus slot (setelah berkasnya dibuang). Mengembalikan `true` bila ada baris terhapus.
pub(super) async fn delete_slot(pool: &PgPool, id: Uuid) -> Result<bool, sqlx::Error> {
    let r = sqlx::query!("DELETE FROM app.upload_slot WHERE id = $1", id)
        .execute(pool)
        .await?;
    Ok(r.rows_affected() == 1)
}

/// Memakai slot sekali di transaksi pemanggil: baris dihapus hanya bila masih ada dan milik `owner`.
pub(super) async fn consume_slot(
    conn: &mut PgConnection,
    id: Uuid,
    owner: Uuid,
) -> Result<bool, sqlx::Error> {
    let r = sqlx::query!(
        "DELETE FROM app.upload_slot WHERE id = $1 AND owner_account_id = $2",
        id,
        owner,
    )
    .execute(&mut *conn)
    .await?;
    Ok(r.rows_affected() == 1)
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
