//! Query akun HR perusahaan (F1-T107, US-1904; DD A1.1 `account`, jenis `corporate_admin` realm
//! `korporat`). Keanggotaan (`corporate_member`) milik modul `corporate` dan hanya dibaca/diubah lewat
//! `corporate::api`. Penulisan berjalan di transaksi use case setelah `corporate::api::lock_members`.

use foundation::Uuid;
use sqlx::PgConnection;
use time::OffsetDateTime;

/// Akun HR (belum dihapus) beserta data yang ditampilkan ke HR perusahaan yang sama.
pub(super) struct HrAccountRow {
    pub(super) id: Uuid,
    /// Kosong hanya untuk akun `pending_provisioning` (`ck_account_idp_subject`).
    pub(super) idp_subject: Option<String>,
    pub(super) email: String,
    pub(super) full_name: Option<String>,
    pub(super) status: String,
}

/// Akun HR `ids` (hanya jenis `corporate_admin`, belum dihapus lunak). Akun yang tidak memenuhi tidak
/// dikembalikan (pemanggil menganggapnya bukan HR). Akun `pending_provisioning` (ADR-030 butir 5, undangan yang
/// penggunanya belum dibuat worker, `idp_subject` kosong) ikut dikembalikan supaya tampil di daftar HR dan
/// dihitung use case undangan (F1-T105/F1-T107); akun yang penyediaannya gagal sudah dihapus lunak dan tidak
/// muncul.
pub(super) async fn hr_accounts(
    conn: &mut PgConnection,
    ids: &[Uuid],
) -> Result<Vec<HrAccountRow>, sqlx::Error> {
    sqlx::query_as!(
        HrAccountRow,
        r#"SELECT id, idp_subject, email, full_name, status FROM app.account
         WHERE id = ANY($1) AND account_type = 'corporate_admin' AND idp_realm = 'korporat'
           AND deleted_at IS NULL"#,
        ids,
    )
    .fetch_all(conn)
    .await
}

/// Menonaktifkan akun HR. `false` bila akun bukan HR, sudah nonaktif, atau terhapus.
pub(super) async fn deactivate_hr(
    conn: &mut PgConnection,
    account_id: Uuid,
    by: Uuid,
    reason: &str,
    now: OffsetDateTime,
) -> Result<bool, sqlx::Error> {
    let done = sqlx::query!(
        "UPDATE app.account SET status = 'inactive', status_reason = $3, status_changed_at = $4, \
           updated_at = $4, updated_by = $2 \
         WHERE id = $1 AND account_type = 'corporate_admin' AND status <> 'inactive' \
           AND deleted_at IS NULL",
        account_id,
        by,
        reason,
        now,
    )
    .execute(conn)
    .await?;
    Ok(done.rows_affected() == 1)
}
