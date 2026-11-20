//! Query sqlx atas tabel milik modul `identity`: `account` dan `rs_admin_role` (A1, kelas M; query default
//! memfilter `deleted_at IS NULL`, ADR-003).

use authz::{AccountRecord, AccountStatus, AccountType, AdminSubRole};
use sqlx::PgPool;

/// Akun aktif maupun tidak untuk `(idp_realm, idp_subject)`, beserta sub-peran admin yang tidak terhapus.
/// Jenis atau status yang tidak dikenal kode = `None` (ditolak `401`, bukan dilewatkan).
pub(super) async fn find_account(
    pool: &PgPool,
    realm: &str,
    subject: &str,
) -> Result<Option<AccountRecord>, sqlx::Error> {
    let Some(row) = sqlx::query!(
        "SELECT id, account_type, status FROM app.account \
         WHERE idp_realm = $1 AND idp_subject = $2 AND deleted_at IS NULL",
        realm,
        subject,
    )
    .fetch_optional(pool)
    .await?
    else {
        return Ok(None);
    };
    let (Some(account_type), Some(status)) = (
        AccountType::parse(&row.account_type),
        AccountStatus::parse(&row.status),
    ) else {
        return Ok(None);
    };
    let sub_roles = if account_type == AccountType::RsAdmin {
        sqlx::query_scalar!(
            "SELECT sub_role FROM app.rs_admin_role WHERE account_id = $1 AND deleted_at IS NULL",
            row.id,
        )
        .fetch_all(pool)
        .await?
        .iter()
        .filter_map(|r| AdminSubRole::parse(r))
        .collect()
    } else {
        Vec::new()
    };
    Ok(Some(AccountRecord {
        account_id: row.id,
        account_type,
        status,
        sub_roles,
    }))
}
