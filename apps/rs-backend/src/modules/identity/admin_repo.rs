//! Query akun dan sub-peran admin Ruang Sehat (F1-T65, US-208; DD A1.1 `account`, A1.2 `rs_admin_role`).
//!
//! Semua penulisan berjalan di transaksi use case (`audit::in_audited_tx`) setelah
//! [`lock_admin_roles`]: kunci tabel `rs_admin_role` mode `SHARE ROW EXCLUSIVE` yang hanya bentrok dengan
//! dirinya sendiri dan dengan penulisan baris (`ROW EXCLUSIVE`), bukan dengan pembacaan resolver akun
//! (`ACCESS SHARE`). Akibatnya pemeriksaan aturan (Super Admin/Auditor terakhir, bootstrap hanya sekali) dan
//! perubahannya berurutan di antara semua pengubah sub-peran: dua bootstrap bersamaan tidak menghasilkan dua
//! akun, dan dua Super Admin yang saling mencabut tidak menyisakan nol (klarifikasi T-19 a-iii). Hak
//! `LOCK ... SHARE ROW EXCLUSIVE` cukup dengan hak `UPDATE` yang dimiliki `rs_app` (SRC-673).
//! Master tanpa DELETE (ADR-003): pencabutan sub-peran = soft delete.

use authz::{AccountStatus, AdminSubRole};
use foundation::Uuid;
use sqlx::PgConnection;
use time::OffsetDateTime;

use super::admin_domain::{AdminAccount, AdminCensus};

/// Mengunci tabel `rs_admin_role` sampai akhir transaksi (lihat dokumentasi modul).
pub async fn lock_admin_roles(conn: &mut PgConnection) -> Result<(), sqlx::Error> {
    sqlx::query!("LOCK TABLE app.rs_admin_role IN SHARE ROW EXCLUSIVE MODE")
        .execute(conn)
        .await?;
    Ok(())
}

/// Semua akun admin Ruang Sehat yang belum dihapus beserta sub-peran aktifnya. Status atau sub-peran yang
/// tidak dikenal membuat baris diabaikan untuk sub-peran (tidak memberi hak), dan status tidak dikenal =
/// galat (CHECK constraint menjamin tidak terjadi).
pub async fn load_census(conn: &mut PgConnection) -> Result<Option<AdminCensus>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT a.id, a.status,
                  COALESCE(array_agg(r.sub_role ORDER BY r.sub_role)
                           FILTER (WHERE r.id IS NOT NULL), '{}') AS "sub_roles!"
           FROM app.account a
           LEFT JOIN app.rs_admin_role r ON r.account_id = a.id AND r.deleted_at IS NULL
           WHERE a.account_type = 'rs_admin' AND a.deleted_at IS NULL
           GROUP BY a.id, a.status"#
    )
    .fetch_all(conn)
    .await?;
    let mut accounts = Vec::with_capacity(rows.len());
    for r in rows {
        let Some(status) = AccountStatus::parse(&r.status) else {
            return Ok(None);
        };
        accounts.push(AdminAccount {
            id: r.id,
            status,
            sub_roles: r
                .sub_roles
                .iter()
                .filter_map(|s| AdminSubRole::parse(s))
                .collect(),
        });
    }
    Ok(Some(AdminCensus { accounts }))
}

/// Subjek Keycloak akun admin (untuk operasi Keycloak setelah commit).
pub async fn admin_subject(
    conn: &mut PgConnection,
    account_id: Uuid,
) -> Result<Option<String>, sqlx::Error> {
    let row = sqlx::query!(
        "SELECT idp_subject FROM app.account \
         WHERE id = $1 AND account_type = 'rs_admin' AND deleted_at IS NULL",
        account_id,
    )
    .fetch_optional(conn)
    .await?;
    // `idp_subject` kosong hanya pada `pending_provisioning` (ADR-030), yang tidak dipakai realm admin.
    Ok(row.and_then(|r| r.idp_subject))
}

/// Akun admin baru berstatus menunggu aktivasi.
pub struct NewAdmin<'a> {
    /// `account.id` (UUIDv7).
    pub id: Uuid,
    /// ID pengguna Keycloak realm `admin`.
    pub idp_subject: &'a str,
    /// Email (salinan untuk notifikasi, KEPUTUSAN-45).
    pub email: &'a str,
    /// Nama lengkap.
    pub full_name: &'a str,
    /// Pembuat; `None` = bootstrap (aktor sistem).
    pub created_by: Option<Uuid>,
    /// Waktu server.
    pub now: OffsetDateTime,
}

/// Menyisipkan akun admin `pending_activation`.
pub async fn insert_admin(conn: &mut PgConnection, a: &NewAdmin<'_>) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO app.account (id, created_by, updated_by, account_type, idp_realm, idp_subject, \
           email, status, status_changed_at, full_name, created_at, updated_at) \
         VALUES ($1, $2, $2, 'rs_admin', 'admin', $3, $4, 'pending_activation', $5, $6, $5, $5)",
        a.id,
        a.created_by,
        a.idp_subject,
        a.email,
        a.now,
        a.full_name,
    )
    .execute(conn)
    .await?;
    Ok(())
}

/// Memberi satu sub-peran.
pub async fn grant_role(
    conn: &mut PgConnection,
    id: Uuid,
    account_id: Uuid,
    role: AdminSubRole,
    by: Option<Uuid>,
    now: OffsetDateTime,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO app.rs_admin_role (id, created_by, updated_by, account_id, sub_role, created_at, updated_at) \
         VALUES ($1, $2, $2, $3, $4, $5, $5)",
        id,
        by,
        account_id,
        role.as_str(),
        now,
    )
    .execute(conn)
    .await?;
    Ok(())
}

/// Mencabut satu sub-peran (soft delete, ADR-003). `false` bila tidak ada baris aktif.
pub async fn revoke_role(
    conn: &mut PgConnection,
    account_id: Uuid,
    role: AdminSubRole,
    by: Uuid,
    reason: &str,
    now: OffsetDateTime,
) -> Result<bool, sqlx::Error> {
    let done = sqlx::query!(
        "UPDATE app.rs_admin_role SET deleted_at = $4, deleted_by = $3, delete_reason = $5, \
           updated_at = $4, updated_by = $3 \
         WHERE account_id = $1 AND sub_role = $2 AND deleted_at IS NULL",
        account_id,
        role.as_str(),
        by,
        now,
        reason,
    )
    .execute(conn)
    .await?;
    Ok(done.rows_affected() == 1)
}

/// Menyimpan (atau mengosongkan dengan `None`) blind index NIK staf akun admin (US-201 AC8,
/// KEPUTUSAN-294). Blind index saja; NIK staf tidak pernah disimpan dalam bentuk yang dapat dibaca kembali.
pub async fn set_staff_nik(
    conn: &mut PgConnection,
    account_id: Uuid,
    nik: Option<(&[u8], i16)>,
    by: Option<Uuid>,
    now: OffsetDateTime,
) -> Result<bool, sqlx::Error> {
    let done = sqlx::query!(
        "UPDATE app.account \
         SET staff_nik_bidx = $2, staff_nik_bidx_key_version = $3, updated_at = $5, updated_by = $4 \
         WHERE id = $1 AND account_type = 'rs_admin' AND deleted_at IS NULL",
        account_id,
        nik.map(|n| n.0),
        nik.map(|n| n.1),
        by,
        now,
    )
    .execute(conn)
    .await?;
    Ok(done.rows_affected() == 1)
}

/// Blind index NIK staf akun admin aktif maupun tidak (`None` bila akun tidak ada atau tanpa NIK staf).
pub async fn staff_nik(
    pool: &sqlx::PgPool,
    account_id: Uuid,
) -> Result<Option<(Vec<u8>, i16)>, sqlx::Error> {
    let row = sqlx::query!(
        "SELECT staff_nik_bidx, staff_nik_bidx_key_version FROM app.account \
         WHERE id = $1 AND account_type = 'rs_admin' AND deleted_at IS NULL",
        account_id,
    )
    .fetch_optional(pool)
    .await?;
    Ok(
        row.and_then(|r| match (r.staff_nik_bidx, r.staff_nik_bidx_key_version) {
            (Some(b), Some(v)) => Some((b, v)),
            _ => None,
        }),
    )
}

/// Menonaktifkan akun admin. `false` bila akun tidak ada atau sudah nonaktif.
pub async fn deactivate(
    conn: &mut PgConnection,
    account_id: Uuid,
    by: Uuid,
    reason: &str,
    now: OffsetDateTime,
) -> Result<bool, sqlx::Error> {
    let done = sqlx::query!(
        "UPDATE app.account SET status = 'inactive', status_reason = $3, status_changed_at = $4, \
           updated_at = $4, updated_by = $2 \
         WHERE id = $1 AND account_type = 'rs_admin' AND status <> 'inactive' AND deleted_at IS NULL",
        account_id,
        by,
        reason,
        now,
    )
    .execute(conn)
    .await?;
    Ok(done.rows_affected() == 1)
}

/// Aktivasi pertama admin (`pending_activation` → `active`) setelah login pertama yang berhasil di realm
/// `admin` (password + TOTP wajib, ADR-011). `false` bila akun bukan admin menunggu aktivasi.
pub async fn activate_pending_admin(
    conn: &mut PgConnection,
    account_id: Uuid,
    now: OffsetDateTime,
) -> Result<bool, sqlx::Error> {
    let done = sqlx::query!(
        "UPDATE app.account SET status = 'active', status_changed_at = $2, updated_at = $2 \
         WHERE id = $1 AND account_type = 'rs_admin' AND status = 'pending_activation' \
           AND deleted_at IS NULL",
        account_id,
        now,
    )
    .execute(conn)
    .await?;
    Ok(done.rows_affected() == 1)
}
