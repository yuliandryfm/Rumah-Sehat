//! Antarmuka publik modul `corporate` untuk perakitan aplikasi dan modul lain.
//!
//! F1-T100: [`CorporateDirectory`] dipasang ke `authz::Enforcement::with_corporate_scopes` sebagai pembaca
//! keanggotaan aktif HR (ADR-028 butir 1). Dibaca langsung dari PostgreSQL di setiap permintaan endpoint HR,
//! tanpa cache, supaya pencabutan keanggotaan dan penonaktifan perusahaan berlaku pada permintaan berikutnya
//! (US-1904 AC5, US-1916 AC2, KU-10, M-51).

use super::repo;
use authz::{
    CorporateClientStatus, CorporateMembership, CorporateScopeResolver, MembershipFuture,
    ResolveError,
};
use foundation::Uuid;
use sqlx::PgPool;

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

impl CorporateScopeResolver for CorporateDirectory {
    fn active_membership<'a>(&'a self, account_id: Uuid) -> MembershipFuture<'a> {
        Box::pin(self.membership(account_id))
    }
}
