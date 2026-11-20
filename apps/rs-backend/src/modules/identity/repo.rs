//! Query sqlx atas tabel milik modul `identity` (DD A1): `account` (A1.1, M), `rs_admin_role` (A1.2, M),
//! `session_revocation` (A1.4, O). Hak `rs_app`: account dan rs_admin_role S/I/U tanpa DELETE (ADR-003);
//! session_revocation S/I/U/D (kelas O, pembersihan setelah kedaluwarsa).

use foundation::Uuid;
use sqlx::{PgConnection, PgPool};
use time::OffsetDateTime;

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
