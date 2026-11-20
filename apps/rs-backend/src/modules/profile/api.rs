//! Antarmuka publik modul `profile` untuk modul lain (F1-T38).
//!
//! Pola pemeriksaan relasi (ADR-012 butir 1 dan 4, KU-03): service pemakai membaca pemilik dengan
//! [`care_recipient_owner`], lalu memutuskan lewat `authz::care_recipient_access`; bukti
//! `authz::OwnedCareRecipient` yang dihasilkan menjadi syarat fungsi data penerima layanan. Query yang
//! sama dijalankan untuk ID yang tidak ada maupun milik orang lain, sehingga keduanya tidak dapat
//! dibedakan dari waktu tanggap.

use super::repo;
use authz::CareRecipientOwner;
use foundation::Uuid;
use sqlx::PgPool;

pub use super::error::ProfileError;

/// Pemilik penerima layanan `care_recipient_id` (`patient.account_id`, `is_account_owner`). `None` bila
/// penerima layanan atau pasiennya tidak ada atau terhapus (ADR-003).
pub async fn care_recipient_owner(
    pool: &PgPool,
    care_recipient_id: Uuid,
) -> Result<Option<CareRecipientOwner>, ProfileError> {
    let row = repo::care_recipient_owner(pool, care_recipient_id)
        .await
        .map_err(|_| ProfileError::Database)?;
    Ok(row.map(|r| CareRecipientOwner {
        account_id: r.account_id,
        is_account_owner: r.is_account_owner,
    }))
}
