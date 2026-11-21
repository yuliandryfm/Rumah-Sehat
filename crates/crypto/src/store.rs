//! Penyimpanan baris `data_key` (DD A19.7) di database aplikasi (F1-T60). Tabel ini milik crate `crypto`
//! (`apps/rs-backend/module-tables.toml`, coding-standards §1.3): modul bisnis menulis dan membaca baris
//! wadah DEK hanya lewat fungsi di sini, di transaksi use case pemanggil (KEPUTUSAN-114).
//!
//! Role aplikasi hanya punya `SELECT`, `INSERT` (tanpa `UPDATE`/`DELETE`; pemusnahan oleh pekerja privasi,
//! ADR-013 butir 5). Galat database dipetakan ke [`CryptoError::Store`] tanpa pesan atau DETAIL, karena
//! DETAIL pelanggaran constraint dapat memuat `wrapped_dek` (TD-03).

use crate::error::CryptoError;
use crate::keyring::{DataKeyRecord, NewDataKey};
use foundation::{Sensitive, Uuid};
use sqlx::PgConnection;

/// Menyisipkan baris wadah DEK baru ([`crate::Keyring::create_data_key`]) di transaksi pemanggil, sebelum
/// baris bisnis yang merujuknya (`*_data_key_id`).
pub async fn insert_data_key(conn: &mut PgConnection, row: &NewDataKey) -> Result<(), CryptoError> {
    sqlx::query!(
        "INSERT INTO app.data_key (id, container_type, container_id, wrapped_dek, kek_ref, \
                                   retention_category, retention_anchor_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
        row.id,
        row.container_type.as_str(),
        row.container_id,
        row.wrapped_dek.expose().as_slice(),
        row.kek_ref,
        row.retention_category,
        row.retention_anchor_at,
    )
    .execute(conn)
    .await
    .map_err(|_| CryptoError::Store)?;
    Ok(())
}

struct Row {
    id: Uuid,
    container_type: String,
    container_id: Uuid,
    wrapped_dek: Option<Vec<u8>>,
    kek_ref: String,
    shredded_at: Option<time::OffsetDateTime>,
}

/// Membaca baris wadah DEK `data_key_id` untuk [`crate::Keyring::open_data_key`]. `None` bila tidak ada.
/// Wadah yang diharapkan tetap diperiksa keyring saat membuka (bukan di query).
pub async fn load_data_key(
    conn: &mut PgConnection,
    data_key_id: Uuid,
) -> Result<Option<DataKeyRecord>, CryptoError> {
    let row = sqlx::query_as!(
        Row,
        "SELECT id, container_type, container_id, wrapped_dek, kek_ref, shredded_at \
         FROM app.data_key WHERE id = $1",
        data_key_id,
    )
    .fetch_optional(conn)
    .await
    .map_err(|_| CryptoError::Store)?;
    Ok(row.map(|r| DataKeyRecord {
        id: r.id,
        container_type: r.container_type,
        container_id: r.container_id,
        wrapped_dek: r.wrapped_dek.map(Sensitive::new),
        kek_ref: r.kek_ref,
        shredded_at: r.shredded_at,
    }))
}
