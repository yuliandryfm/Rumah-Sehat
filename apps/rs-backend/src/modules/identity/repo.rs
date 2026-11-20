//! Query modul `identity` untuk pengambil event Keycloak (F1-T34): posisi baca per realm
//! (`idp_event_cursor`), tanda terima event (`idp_event_receipt`), dan pemetaan subjek Keycloak ke akun
//! (`account`, `rs_admin_role`). Semua tabel milik modul ini (`apps/rs-backend/module-tables.toml`).

use super::domain::IdpRealm;
use foundation::{Uuid, new_id};
use sqlx::PgConnection;
use time::OffsetDateTime;

/// Posisi baca satu realm.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cursor {
    /// Waktu event (milidetik epoch) yang sudah diproses sampai dengan.
    pub position_ms: i64,
    /// Akhir putaran pengambilan terakhir yang berhasil (jam server aplikasi).
    pub last_pulled_at: Option<OffsetDateTime>,
}

/// Membaca posisi baca realm; realm yang belum pernah diambil mulai dari 0 (semua event yang masih
/// disimpan Keycloak, paling lama `eventsExpiration` 7 hari).
pub async fn load_cursor(conn: &mut PgConnection, realm: IdpRealm) -> Result<Cursor, sqlx::Error> {
    let row = sqlx::query!(
        "SELECT position_ms, last_pulled_at FROM app.idp_event_cursor WHERE idp_realm = $1",
        realm.as_str(),
    )
    .fetch_optional(conn)
    .await?;
    Ok(row.map_or(
        Cursor {
            position_ms: 0,
            last_pulled_at: None,
        },
        |r| Cursor {
            position_ms: r.position_ms,
            last_pulled_at: r.last_pulled_at,
        },
    ))
}

/// Memajukan posisi baca (tidak pernah mundur: `GREATEST`). `pulled_at` diisi bila putaran selesai.
pub async fn advance_cursor(
    conn: &mut PgConnection,
    realm: IdpRealm,
    position_ms: i64,
    pulled_at: Option<OffsetDateTime>,
    now: OffsetDateTime,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO app.idp_event_cursor (id, idp_realm, position_ms, last_pulled_at, updated_at) \
         VALUES ($1, $2, $3, $4, $5) \
         ON CONFLICT (idp_realm) DO UPDATE SET \
           position_ms = GREATEST(app.idp_event_cursor.position_ms, EXCLUDED.position_ms), \
           last_pulled_at = COALESCE(EXCLUDED.last_pulled_at, app.idp_event_cursor.last_pulled_at), \
           updated_at = EXCLUDED.updated_at",
        new_id(),
        realm.as_str(),
        position_ms,
        pulled_at,
        now,
    )
    .execute(conn)
    .await?;
    Ok(())
}

/// Mencatat tanda terima event; `false` bila event yang sama sudah pernah dicatat (kiriman ulang).
/// Dipanggil di transaksi teraudit yang sama dengan penulisan `audit_outbox`, sehingga tanda terima dan
/// event audit ter-commit atau batal bersama.
pub async fn insert_receipt(
    conn: &mut PgConnection,
    realm: IdpRealm,
    idp_event_id: Uuid,
    event_time: OffsetDateTime,
) -> Result<bool, sqlx::Error> {
    let done = sqlx::query!(
        "INSERT INTO app.idp_event_receipt (id, idp_realm, idp_event_id, event_time) \
         VALUES ($1, $2, $3, $4) ON CONFLICT (idp_realm, idp_event_id) DO NOTHING",
        new_id(),
        realm.as_str(),
        idp_event_id,
        event_time,
    )
    .execute(conn)
    .await?;
    Ok(done.rows_affected() == 1)
}

/// Menghapus tanda terima yang lebih tua dari `before` (di luar jendela tumpang-tindih).
pub async fn prune_receipts(
    conn: &mut PgConnection,
    realm: IdpRealm,
    before: OffsetDateTime,
) -> Result<u64, sqlx::Error> {
    Ok(sqlx::query!(
        "DELETE FROM app.idp_event_receipt WHERE idp_realm = $1 AND event_time < $2",
        realm.as_str(),
        before,
    )
    .execute(conn)
    .await?
    .rows_affected())
}

/// Akun aplikasi yang dipetakan ke subjek Keycloak (DD A1.1, KEPUTUSAN-45), tanpa data pribadi.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountRef {
    /// `account.id`.
    pub id: Uuid,
    /// `account.account_type`.
    pub account_type: String,
    /// Sub-peran admin aktif (`rs_admin_role.sub_role`), urut.
    pub roles: Vec<String>,
}

/// Mencari akun aktif (belum dihapus) untuk `(idp_realm, idp_subject)`.
pub async fn find_account(
    conn: &mut PgConnection,
    realm: IdpRealm,
    idp_subject: &str,
) -> Result<Option<AccountRef>, sqlx::Error> {
    let Some(acc) = sqlx::query!(
        "SELECT id, account_type FROM app.account \
         WHERE idp_realm = $1 AND idp_subject = $2 AND deleted_at IS NULL",
        realm.as_str(),
        idp_subject,
    )
    .fetch_optional(&mut *conn)
    .await?
    else {
        return Ok(None);
    };
    let roles = sqlx::query_scalar!(
        "SELECT sub_role FROM app.rs_admin_role \
         WHERE account_id = $1 AND deleted_at IS NULL ORDER BY sub_role",
        acc.id,
    )
    .fetch_all(&mut *conn)
    .await?;
    Ok(Some(AccountRef {
        id: acc.id,
        account_type: acc.account_type,
        roles,
    }))
}
