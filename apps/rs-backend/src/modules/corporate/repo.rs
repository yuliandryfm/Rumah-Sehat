//! Akses data modul Korporat (F1-T100).

use foundation::Uuid;
use sqlx::PgPool;

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
