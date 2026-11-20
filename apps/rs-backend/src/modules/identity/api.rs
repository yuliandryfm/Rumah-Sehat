//! Antarmuka publik modul `identity`: resolver akun dari database aplikasi untuk middleware kebijakan
//! crate `authz` (F1-T37). Permintaan dengan akun yang tidak ada, terhapus, berjenis/berstatus tidak dikenal
//! dijawab `401` oleh middleware; akun nonaktif juga `401` (KU-10). Database tidak tersedia = `503`
//! (gagal-tertutup, `authz::ResolveError`).

use super::repo;
use authn::AuthContext;
use authz::{PrincipalResolver, ResolveError, ResolveFuture};
use sqlx::PgPool;

/// Resolver `(realm, subject)` → akun + sub-peran admin aktif (`account`, `rs_admin_role`; tanpa baris
/// terhapus). Pemeriksaan sesi yang dicabut menyusul di F1-T33.
#[derive(Debug, Clone)]
pub struct DbPrincipalResolver {
    pool: PgPool,
}

impl DbPrincipalResolver {
    /// Resolver di atas pool role runtime `rs_app`.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl PrincipalResolver for DbPrincipalResolver {
    fn resolve<'a>(&'a self, ctx: &'a AuthContext) -> ResolveFuture<'a> {
        Box::pin(async move {
            // Pesan galat sqlx tidak diteruskan (dapat memuat teks query, coding-standards §3.4).
            repo::find_account(&self.pool, ctx.realm().as_str(), ctx.subject())
                .await
                .map_err(|_| ResolveError)
        })
    }
}
