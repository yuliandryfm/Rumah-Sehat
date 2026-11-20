//! Query sqlx atas tabel milik modul `identity` (DD A1; `apps/rs-backend/module-tables.toml`).
//!
//! - F1-T34 (pengambil event Keycloak): posisi baca per realm (`idp_event_cursor`), tanda terima event
//!   (`idp_event_receipt`), dan pemetaan subjek Keycloak ke akun (`account`, `rs_admin_role`).
//! - F1-T33 (resolver akun dan pencabutan sesi): `account` (A1.1, M), `rs_admin_role` (A1.2, M),
//!   `session_revocation` (A1.4, O). Hak `rs_app`: account dan rs_admin_role S/I/U tanpa DELETE
//!   (ADR-003); session_revocation S/I/U/D (kelas O, pembersihan setelah kedaluwarsa).

use super::domain::IdpRealm;
use foundation::{Uuid, new_id};
use sqlx::{PgConnection, PgPool};
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

// ----- F1-T33: resolver akun dan pencabutan sesi -----

/// Akun untuk satu `(realm, subject)` beserta sub-peran admin aktif.
pub(super) struct AccountRow {
    pub(super) id: Uuid,
    pub(super) account_type: String,
    pub(super) status: String,
    pub(super) sub_roles: Vec<String>,
}

/// Akun aktif (belum terhapus) untuk subjek Keycloak `subject` di realm `realm`, dengan sub-peran admin
/// yang belum terhapus (ADR-003). Satu putaran ke database per permintaan yang tidak kena cache.
pub(super) async fn account_by_subject(
    pool: &PgPool,
    realm: &str,
    subject: &str,
) -> Result<Option<AccountRow>, sqlx::Error> {
    sqlx::query_as!(
        AccountRow,
        r#"SELECT a.id, a.account_type, a.status,
                  COALESCE(array_agg(r.sub_role ORDER BY r.sub_role)
                           FILTER (WHERE r.sub_role IS NOT NULL), '{}') AS "sub_roles!"
           FROM app.account a
           LEFT JOIN app.rs_admin_role r ON r.account_id = a.id AND r.deleted_at IS NULL
           WHERE a.idp_realm = $1 AND a.idp_subject = $2 AND a.deleted_at IS NULL
           GROUP BY a.id"#,
        realm,
        subject,
    )
    .fetch_optional(pool)
    .await
}

/// Mencatat sesi `idp_session_id` sebagai dicabut di transaksi milik pemanggil. Sesi yang sudah tercatat
/// memakai `expires_at` terlama (pencabutan tidak pernah dipersingkat). Mengembalikan `expires_at`
/// yang berlaku.
pub(super) async fn insert_revocation(
    conn: &mut PgConnection,
    id: Uuid,
    account_id: Uuid,
    idp_session_id: &str,
    reason_code: &str,
    expires_at: OffsetDateTime,
) -> Result<OffsetDateTime, sqlx::Error> {
    sqlx::query_scalar!(
        "INSERT INTO app.session_revocation (id, account_id, idp_session_id, reason_code, expires_at) \
         VALUES ($1, $2, $3, $4, $5) \
         ON CONFLICT ON CONSTRAINT uq_session_revocation_idp_session_id \
         DO UPDATE SET expires_at = GREATEST(app.session_revocation.expires_at, EXCLUDED.expires_at) \
         RETURNING expires_at",
        id,
        account_id,
        idp_session_id,
        reason_code,
        expires_at,
    )
    .fetch_one(conn)
    .await
}

/// Apakah sesi tercatat dicabut dan pencabutannya belum lewat pada `now`.
pub(super) async fn is_revoked(
    pool: &PgPool,
    idp_session_id: &str,
    now: OffsetDateTime,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar!(
        r#"SELECT EXISTS (
             SELECT 1 FROM app.session_revocation WHERE idp_session_id = $1 AND expires_at > $2
           ) AS "revoked!""#,
        idp_session_id,
        now,
    )
    .fetch_one(pool)
    .await
}

/// Pencabutan yang masih berlaku pada `now` (untuk membangun ulang cache Valkey).
pub(super) struct ActiveRevocation {
    pub(super) idp_session_id: String,
    pub(super) expires_at: OffsetDateTime,
}

pub(super) async fn active_revocations(
    pool: &PgPool,
    now: OffsetDateTime,
) -> Result<Vec<ActiveRevocation>, sqlx::Error> {
    sqlx::query_as!(
        ActiveRevocation,
        "SELECT idp_session_id, expires_at FROM app.session_revocation WHERE expires_at > $1",
        now,
    )
    .fetch_all(pool)
    .await
}

/// Menghapus pencabutan yang token terakhirnya pasti sudah kedaluwarsa (DD A1.4).
pub(super) async fn purge_expired(pool: &PgPool, now: OffsetDateTime) -> Result<u64, sqlx::Error> {
    let r = sqlx::query!(
        "DELETE FROM app.session_revocation WHERE expires_at <= $1",
        now
    )
    .execute(pool)
    .await?;
    Ok(r.rows_affected())
}
