//! Akses data modul Korporat (F1-T100; F1-T107 anggota HR).

use foundation::Uuid;
use sqlx::{PgConnection, PgPool};
use time::OffsetDateTime;

/// Keanggotaan aktif satu akun HR beserta status perusahaannya.
pub(super) struct MembershipRow {
    pub(super) corporate_member_id: Uuid,
    pub(super) corporate_client_id: Uuid,
    pub(super) client_status: String,
}

/// Keanggotaan aktif akun `account_id`: baris `corporate_member` yang belum dihapus lunak (paling banyak
/// satu, `uqp_corporate_member_account_id`) dengan perusahaan yang belum dihapus lunak (ADR-003). Index
/// `uqp_corporate_member_account_id` dan PK `corporate_client` melayani query ini.
pub(super) async fn active_membership(
    pool: &PgPool,
    account_id: Uuid,
) -> Result<Option<MembershipRow>, sqlx::Error> {
    sqlx::query_as!(
        MembershipRow,
        "SELECT m.id AS corporate_member_id, m.corporate_client_id, c.status AS client_status \
         FROM app.corporate_member m \
         JOIN app.corporate_client c ON c.id = m.corporate_client_id \
         WHERE m.account_id = $1 AND m.deleted_at IS NULL AND c.deleted_at IS NULL",
        account_id,
    )
    .fetch_optional(pool)
    .await
}

// ----- F1-T107: kelola anggota HR (US-1904 AC2, AC5) -----

/// Keanggotaan aktif (belum dihapus lunak) satu perusahaan.
pub(super) struct MemberRow {
    pub(super) id: Uuid,
    pub(super) account_id: Uuid,
    pub(super) job_title: Option<String>,
}

/// Mengunci tabel `corporate_member` mode `SHARE ROW EXCLUSIVE` sampai akhir transaksi (pola
/// KEPUTUSAN-290): bentrok dengan dirinya sendiri dan dengan penulisan baris (`ROW EXCLUSIVE`), tidak dengan
/// pembacaan pembaca keanggotaan (`ACCESS SHARE`). Hak cukup dengan `UPDATE` milik `rs_app` (SRC-673).
pub(super) async fn lock_members(conn: &mut PgConnection) -> Result<(), sqlx::Error> {
    sqlx::query!("LOCK TABLE app.corporate_member IN SHARE ROW EXCLUSIVE MODE")
        .execute(conn)
        .await?;
    Ok(())
}

/// Semua keanggotaan aktif perusahaan `client_id`, urut ID (index `ix_corporate_member_corporate_client_id`).
pub(super) async fn active_members(
    conn: &mut PgConnection,
    client_id: Uuid,
) -> Result<Vec<MemberRow>, sqlx::Error> {
    sqlx::query_as!(
        MemberRow,
        "SELECT id, account_id, job_title FROM app.corporate_member \
         WHERE corporate_client_id = $1 AND deleted_at IS NULL ORDER BY id",
        client_id,
    )
    .fetch_all(conn)
    .await
}

/// Perusahaan pemilik keanggotaan aktif `member_id` (`None` bila tidak ada atau sudah dicabut).
pub(super) async fn member_owner(
    pool: &PgPool,
    member_id: Uuid,
) -> Result<Option<Uuid>, sqlx::Error> {
    sqlx::query_scalar!(
        "SELECT corporate_client_id FROM app.corporate_member WHERE id = $1 AND deleted_at IS NULL",
        member_id,
    )
    .fetch_optional(pool)
    .await
}

/// Mencabut keanggotaan (hapus lunak, ADR-003). Selalu difilter perusahaan pemanggil (pertahanan
/// berlapis). `false` bila tidak ada baris aktif yang cocok.
pub(super) async fn revoke_member(
    conn: &mut PgConnection,
    member_id: Uuid,
    client_id: Uuid,
    by: Uuid,
    reason: &str,
    now: OffsetDateTime,
) -> Result<bool, sqlx::Error> {
    let done = sqlx::query!(
        "UPDATE app.corporate_member SET deleted_at = $4, deleted_by = $3, delete_reason = $5, \
           updated_at = $4, updated_by = $3 \
         WHERE id = $1 AND corporate_client_id = $2 AND deleted_at IS NULL",
        member_id,
        client_id,
        by,
        now,
        reason,
    )
    .execute(conn)
    .await?;
    Ok(done.rows_affected() == 1)
}
