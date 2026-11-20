//! Query sqlx atas tabel milik modul `profile` (DD A2): `patient` (A2.1, M) dan `care_recipient`
//! (A2.2, M). Query default memfilter baris terhapus (ADR-003).

use foundation::Uuid;
use sqlx::PgPool;

/// Pemilik satu penerima layanan.
pub(super) struct OwnerRow {
    pub(super) account_id: Uuid,
    pub(super) is_account_owner: bool,
}

pub(super) async fn care_recipient_owner(
    pool: &PgPool,
    care_recipient_id: Uuid,
) -> Result<Option<OwnerRow>, sqlx::Error> {
    sqlx::query_as!(
        OwnerRow,
        "SELECT p.account_id, cr.is_account_owner \
         FROM app.care_recipient cr \
         JOIN app.patient p ON p.id = cr.patient_id \
         WHERE cr.id = $1 AND cr.deleted_at IS NULL AND p.deleted_at IS NULL",
        care_recipient_id,
    )
    .fetch_optional(pool)
    .await
}
