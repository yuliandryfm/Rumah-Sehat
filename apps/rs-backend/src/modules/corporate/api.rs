//! Antarmuka publik modul `corporate` untuk perakitan aplikasi dan modul lain.
//!
//! F1-T100: [`CorporateDirectory`] dipasang ke `authz::Enforcement::with_corporate_scopes` sebagai pembaca
//! keanggotaan aktif HR (ADR-028 butir 1). Dibaca langsung dari PostgreSQL di setiap permintaan endpoint HR,
//! tanpa cache, supaya pencabutan keanggotaan dan penonaktifan perusahaan berlaku pada permintaan berikutnya
//! (US-1904 AC5, US-1916 AC2, KU-10, M-51).
//!
//! F1-T107: operasi keanggotaan HR untuk use case kelola anggota HR di modul `identity` (pemilik akun):
//! kunci tabel, daftar keanggotaan aktif per perusahaan, pemilik keanggotaan, dan pencabutan (hapus lunak).

use super::repo;
use authz::{
    CorporateClientStatus, CorporateMembership, CorporateScopeResolver, MembershipFuture,
    ResolveError,
};
use foundation::Uuid;
use sqlx::{PgConnection, PgPool};
use time::OffsetDateTime;

/// Pembaca keanggotaan aktif HR di atas pool role `rs_app`.
#[derive(Clone)]
pub struct CorporateDirectory {
    pool: PgPool,
}

impl CorporateDirectory {
    /// `pool` = login `rs_api` (anggota `rs_app`, hak SELECT pada `corporate_member` dan `corporate_client`).
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Keanggotaan aktif akun `account_id`. Status perusahaan yang tidak dikenal = galat (ditolak `503`),
    /// tidak pernah dianggap Aktif.
    pub async fn membership(
        &self,
        account_id: Uuid,
    ) -> Result<Option<CorporateMembership>, ResolveError> {
        let row = repo::active_membership(&self.pool, account_id)
            .await
            .map_err(|_| {
                tracing::warn!("corporate_membership_read_failed");
                ResolveError
            })?;
        let Some(row) = row else {
            return Ok(None);
        };
        let Some(client_status) = CorporateClientStatus::parse(&row.client_status) else {
            tracing::error!("corporate_client_status_unparseable");
            return Err(ResolveError);
        };
        Ok(Some(CorporateMembership {
            corporate_member_id: row.corporate_member_id,
            corporate_client_id: row.corporate_client_id,
            client_status,
        }))
    }
}

/// Keanggotaan aktif satu HR di perusahaan (F1-T107).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemberRef {
    /// `corporate_member.id`.
    pub corporate_member_id: Uuid,
    /// `account.id` akun HR.
    pub account_id: Uuid,
    /// Jabatan (boleh kosong untuk HR yang diundang).
    pub job_title: Option<String>,
}

/// Mengunci `corporate_member` sampai akhir transaksi pemanggil (KEPUTUSAN-290 diterapkan ke invarian "HR
/// aktif terakhir per perusahaan", ADR-027/ADR-028). **Setiap** transaksi yang mencabut keanggotaan atau
/// menonaktifkan akun HR wajib memanggil ini lebih dulu, lalu membaca ulang [`active_members`] di transaksi
/// yang sama. Pembaca keanggotaan (`ACCESS SHARE`) tidak terhalang.
pub async fn lock_members(conn: &mut PgConnection) -> Result<(), sqlx::Error> {
    repo::lock_members(conn).await
}

/// Keanggotaan aktif perusahaan `corporate_client_id`.
pub async fn active_members(
    conn: &mut PgConnection,
    corporate_client_id: Uuid,
) -> Result<Vec<MemberRef>, sqlx::Error> {
    Ok(repo::active_members(conn, corporate_client_id)
        .await?
        .into_iter()
        .map(|r| MemberRef {
            corporate_member_id: r.id,
            account_id: r.account_id,
            job_title: r.job_title,
        })
        .collect())
}

/// Perusahaan pemilik keanggotaan aktif `corporate_member_id`, untuk keputusan objek `authz::corporate_object`
/// (perusahaan lain dan tidak ada = `404` yang sama, KU-03).
pub async fn member_owner(
    pool: &PgPool,
    corporate_member_id: Uuid,
) -> Result<Option<Uuid>, sqlx::Error> {
    repo::member_owner(pool, corporate_member_id).await
}

/// Mencabut keanggotaan aktif (hapus lunak dengan alasan) di transaksi pemanggil setelah [`lock_members`].
/// `false` bila tidak ada keanggotaan aktif `corporate_member_id` milik `corporate_client_id`.
pub async fn revoke_member(
    conn: &mut PgConnection,
    corporate_member_id: Uuid,
    corporate_client_id: Uuid,
    by: Uuid,
    reason: &str,
    now: OffsetDateTime,
) -> Result<bool, sqlx::Error> {
    repo::revoke_member(
        conn,
        corporate_member_id,
        corporate_client_id,
        by,
        reason,
        now,
    )
    .await
}

impl CorporateScopeResolver for CorporateDirectory {
    fn active_membership<'a>(&'a self, account_id: Uuid) -> MembershipFuture<'a> {
        Box::pin(self.membership(account_id))
    }
}
