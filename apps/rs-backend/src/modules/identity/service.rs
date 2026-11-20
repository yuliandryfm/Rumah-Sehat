//! Resolver akun dan penolakan sesi seketika (F1-T33, ADR-011 butir 8, KU-10, US-103 AC5, RT-18).
//!
//! Sumber kebenaran adalah PostgreSQL; Valkey hanya cache. Aturan per jalur:
//!
//! **Akun** (`authz::PrincipalResolver`): Valkey `GET` → kena: dipakai; tidak ada: baca PostgreSQL lalu
//! `SET NX` ber-TTL pendek; Valkey gagal atau habis waktu: baca PostgreSQL (RT-18 mitigasi 2). Penulis
//! status (penonaktifan, perubahan peran) memanggil [`AccountDirectory::refresh_account`] setelah commit,
//! yang **menimpa** entri (bukan menghapus), sehingga pembaca yang membaca data lama sebelum commit tidak
//! dapat menulis ulang nilai lama (`SET NX` pembaca kalah). Bila penyegaran gagal, entri lama paling lama
//! hidup selama TTL; pemanggil wajib mencoba ulang lewat job dan memicu peringatan (ADR-011 butir 8).
//!
//! **Sesi dicabut** (`authz::SessionGuard`): pencabutan dicatat di `session_revocation` (transaksi
//! pemanggil) lalu disalin ke Valkey setelah commit. Pemeriksaan membaca `MGET [penanda, sid]` dalam satu
//! putaran: penanda ada → hasil Valkey dipakai; penanda tidak ada (isi Valkey hilang, misalnya restart
//! tanpa persistensi) → PostgreSQL, dan salinan dibangun ulang di latar belakang; Valkey gagal →
//! PostgreSQL. Penyalinan satu pencabutan yang gagal: (1) sid disimpan di daftar tunggu proses ini, yang
//! diperiksa sebelum Valkey (proses ini langsung menolak); (2) penghitung kegagalan dinaikkan dan penanda
//! dihapus (memaksa jalur PostgreSQL di semua proses); (3) penyalinan dicoba ulang di latar belakang
//! sampai berhasil atau pencabutan berakhir; galat dilaporkan ke pemanggil (ADR-011 butir 8: dicoba
//! ulang dan memicu peringatan). Pembangunan ulang hanya memasang penanda bila penghitung tidak berubah
//! sejak snapshot PostgreSQL. Sisa risiko: bila Valkey kembali terjangkau sebelum langkah (2) maupun
//! percobaan ulang berhasil, proses API **lain** dapat meloloskan sesi itu sampai percobaan ulang
//! berikutnya (jeda maksimal [`PUBLISH_RETRY_MAX_DELAY`]); batas atasnya umur token akses.
//! PostgreSQL gagal di jalur cadangan = permintaan ditolak `503` (gagal-tertutup).

use super::cache::{
    REVOCATIONS_EPOCH, REVOCATIONS_LOADED, account_key, decode_account, encode_account,
    revoked_session_key,
};
use super::error::IdentityError;
use super::repo;
use authn::{AuthContext, Realm};
use authz::{
    AccountRecord, AccountStatus, AccountType, AdminSubRole, PrincipalResolver, ResolveError,
    ResolveFuture, RevocationFuture, SessionGuard,
};
use foundation::{Clock, Uuid, new_id};
use sqlx::{PgConnection, PgPool};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use time::OffsetDateTime;
use valkey::Valkey;

/// Bawaan TTL entri akun di Valkey (detik). Usulan teknis F1-T33: ADR-011 butir 8 hanya menyebut "TTL
/// pendek"; 30 detik membatasi umur entri lama bila penyegaran oleh penulis status gagal.
pub const DEFAULT_ACCOUNT_CACHE_TTL_SECONDS: u64 = 30;

/// Jeda awal percobaan ulang penyalinan pencabutan ke Valkey.
pub const PUBLISH_RETRY_MIN_DELAY: Duration = Duration::from_millis(200);
/// Jeda maksimal percobaan ulang penyalinan pencabutan ke Valkey.
pub const PUBLISH_RETRY_MAX_DELAY: Duration = Duration::from_secs(2);

/// Pencabutan yang gagal disalin ke Valkey: sid → batas berlaku.
type Pending = Arc<Mutex<HashMap<String, OffsetDateTime>>>;

fn pending_lock(p: &Pending) -> std::sync::MutexGuard<'_, HashMap<String, OffsetDateTime>> {
    p.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Alasan pencabutan (`ck_session_revocation_reason_code`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RevocationReason {
    /// Pengguna keluar (US-103 AC5).
    Logout,
    /// Sesi diakhiri admin atau dari layar sesi aktif (US-110).
    AdminRevoke,
    /// Akun dinonaktifkan (KU-10).
    AccountDeactivated,
    /// Peran berubah (US-1302 AC4).
    RoleChanged,
}

impl RevocationReason {
    /// Kode di database.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Logout => "logout",
            Self::AdminRevoke => "admin_revoke",
            Self::AccountDeactivated => "account_deactivated",
            Self::RoleChanged => "role_changed",
        }
    }
}

/// Pencabutan yang sudah tercatat di database dan menunggu disalin ke Valkey setelah commit.
#[derive(Debug, Clone)]
pub struct RecordedRevocation {
    idp_session_id: String,
    expires_at: OffsetDateTime,
}

/// Resolver akun dan penjaga sesi untuk middleware penegak `authz`.
pub struct AccountDirectory {
    pool: PgPool,
    valkey: Valkey,
    clock: Arc<dyn Clock>,
    account_ttl: Duration,
    rebuilding: Arc<AtomicBool>,
    pending: Pending,
}

fn parse_row(row: repo::AccountRow) -> Option<AccountRecord> {
    let account_type = AccountType::parse(&row.account_type)?;
    let status = AccountStatus::parse(&row.status)?;
    // Sub-peran tidak dikenal diabaikan (tidak memberi hak), sama dengan AdminSubRole::parse.
    let sub_roles = row
        .sub_roles
        .iter()
        .filter_map(|r| AdminSubRole::parse(r))
        .collect();
    Some(AccountRecord {
        account_id: row.id,
        account_type,
        status,
        sub_roles,
    })
}

fn remaining(until: OffsetDateTime, now: OffsetDateTime) -> Option<Duration> {
    let d = until - now;
    if d.is_positive() {
        Duration::try_from(d).ok()
    } else {
        None
    }
}

impl AccountDirectory {
    /// `account_ttl` = umur entri akun di Valkey.
    pub fn new(pool: PgPool, valkey: Valkey, clock: Arc<dyn Clock>, account_ttl: Duration) -> Self {
        Self {
            pool,
            valkey,
            clock,
            account_ttl,
            rebuilding: Arc::new(AtomicBool::new(false)),
            pending: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn account_from_db(
        &self,
        realm: Realm,
        subject: &str,
    ) -> Result<Option<AccountRecord>, IdentityError> {
        let row = repo::account_by_subject(&self.pool, realm.as_str(), subject)
            .await
            .map_err(|_| IdentityError::Database)?;
        Ok(match row {
            None => None,
            Some(row) => {
                let parsed = parse_row(row);
                if parsed.is_none() {
                    tracing::error!(realm = realm.as_str(), "account_row_unparseable");
                }
                parsed
            }
        })
    }

    async fn resolve_cached(
        &self,
        ctx: &AuthContext,
    ) -> Result<Option<AccountRecord>, ResolveError> {
        let key = account_key(ctx.realm(), ctx.subject());
        let mut overwrite = false;
        match self.valkey.get(&key).await {
            Ok(Some(bytes)) => {
                if let Some(rec) = decode_account(&bytes) {
                    return Ok(Some(rec));
                }
                tracing::warn!("account_cache_value_invalid");
                overwrite = true;
            }
            Ok(None) => {}
            Err(e) => {
                tracing::warn!(
                    error_kind = e.code(),
                    "valkey_unavailable_account_db_fallback"
                );
            }
        }
        let rec = self
            .account_from_db(ctx.realm(), ctx.subject())
            .await
            .map_err(|_| ResolveError)?;
        if let Some(r) = &rec {
            let value = encode_account(r);
            // Nilai rusak ditimpa; selain itu SET NX supaya nilai lama tidak menimpa penyegaran penulis.
            let filled = if overwrite {
                self.valkey.set(&key, &value, self.account_ttl).await
            } else {
                self.valkey
                    .set_if_absent(&key, &value, self.account_ttl)
                    .await
                    .map(|_| ())
            };
            if let Err(e) = filled {
                tracing::warn!(error_kind = e.code(), "account_cache_fill_failed");
            }
        }
        Ok(rec)
    }

    /// Akun langsung dari PostgreSQL tanpa cache, untuk aksi berisiko (ekspor, refund, perubahan peran,
    /// akses dokumen identitas; ADR-011 butir 8, M-13).
    pub async fn resolve_fresh(
        &self,
        ctx: &AuthContext,
    ) -> Result<Option<AccountRecord>, IdentityError> {
        self.account_from_db(ctx.realm(), ctx.subject()).await
    }

    /// Menyegarkan entri akun di Valkey dari PostgreSQL setelah status atau peran berubah (dipanggil
    /// penulis status **setelah commit**). Akun tidak ada lagi = entri dihapus. Galat = pemanggil mencoba
    /// ulang lewat job dan memicu peringatan (ADR-011 butir 8).
    pub async fn refresh_account(&self, realm: Realm, subject: &str) -> Result<(), IdentityError> {
        let key = account_key(realm, subject);
        let result = match self.account_from_db(realm, subject).await? {
            Some(rec) => {
                self.valkey
                    .set(&key, &encode_account(&rec), self.account_ttl)
                    .await
            }
            None => self.valkey.delete(&key).await,
        };
        result.map_err(|e| {
            tracing::error!(error_kind = e.code(), "account_cache_refresh_failed");
            IdentityError::CacheUnavailable
        })
    }

    async fn revoked_in_db(&self, sid: &str) -> Result<bool, ResolveError> {
        repo::is_revoked(&self.pool, sid, self.clock.now())
            .await
            .map_err(|_| ResolveError)
    }

    async fn check_revoked(&self, ctx: &AuthContext) -> Result<bool, ResolveError> {
        let Some(sid) = ctx.session_id().filter(|s| !s.is_empty()) else {
            // Realm akun manusia wajib membawa `sid` (authn); tanpa itu sesi tidak dapat diperiksa.
            return Ok(true);
        };
        if self.pending_contains(sid) {
            return Ok(true);
        }
        let key = revoked_session_key(sid);
        match self.valkey.get_many(&[REVOCATIONS_LOADED, &key]).await {
            Ok(values) => match values.as_slice() {
                [Some(_), entry] => return Ok(entry.is_some()),
                _ => {
                    tracing::warn!("revocation_cache_not_loaded_db_fallback");
                    self.spawn_rebuild();
                }
            },
            Err(e) => {
                tracing::warn!(
                    error_kind = e.code(),
                    "valkey_unavailable_revocation_db_fallback"
                );
            }
        }
        self.revoked_in_db(sid).await
    }

    fn pending_contains(&self, sid: &str) -> bool {
        let now = self.clock.now();
        let mut pending = pending_lock(&self.pending);
        pending.retain(|_, until| *until > now);
        pending.contains_key(sid)
    }

    fn spawn_rebuild(&self) {
        if self
            .rebuilding
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return;
        }
        let pool = self.pool.clone();
        let valkey = self.valkey.clone();
        let clock = Arc::clone(&self.clock);
        let flag = Arc::clone(&self.rebuilding);
        tokio::spawn(async move {
            if let Err(e) = rebuild(&pool, &valkey, clock.as_ref()).await {
                tracing::warn!(error_kind = e.code(), "revocation_cache_rebuild_failed");
            }
            flag.store(false, Ordering::Release);
        });
    }

    /// Menyalin semua pencabutan aktif dari PostgreSQL ke Valkey lalu memasang penanda (saat start atau
    /// setelah isi Valkey hilang). Mengembalikan jumlah pencabutan yang disalin.
    pub async fn rebuild_revocation_cache(&self) -> Result<usize, IdentityError> {
        rebuild(&self.pool, &self.valkey, self.clock.as_ref()).await
    }

    /// Mencatat pencabutan sesi `idp_session_id` milik `account_id` di transaksi pemanggil (KEPUTUSAN-114;
    /// pemanggil dapat menjalankannya di `audit::in_audited_tx` bersama event E-03/E-10). Setelah commit,
    /// pemanggil wajib memanggil [`AccountDirectory::publish_revocation`]. `expires_at` = saat token akses
    /// terakhir sesi itu pasti kedaluwarsa (DD A1.4); harus setelah sekarang.
    pub async fn record_revocation(
        &self,
        conn: &mut PgConnection,
        account_id: Uuid,
        idp_session_id: &str,
        reason: RevocationReason,
        expires_at: OffsetDateTime,
    ) -> Result<RecordedRevocation, IdentityError> {
        if idp_session_id.is_empty() || expires_at <= self.clock.now() {
            return Err(IdentityError::InvalidRevocation);
        }
        let effective = repo::insert_revocation(
            conn,
            new_id(),
            account_id,
            idp_session_id,
            reason.as_str(),
            expires_at,
        )
        .await
        .map_err(|_| IdentityError::Database)?;
        Ok(RecordedRevocation {
            idp_session_id: idp_session_id.to_string(),
            expires_at: effective,
        })
    }

    /// Menyalin pencabutan yang sudah di-commit ke Valkey. Gagal → penanda salinan lengkap dihapus supaya
    /// semua pemeriksaan membaca PostgreSQL sampai salinan dibangun ulang; galat dikembalikan supaya
    /// pemanggil mencoba ulang lewat job dan memicu peringatan.
    pub async fn publish_revocation(&self, rec: &RecordedRevocation) -> Result<(), IdentityError> {
        let Some(ttl) = remaining(rec.expires_at, self.clock.now()) else {
            return Ok(());
        };
        let key = revoked_session_key(&rec.idp_session_id);
        match self.valkey.set(&key, b"1", ttl).await {
            Ok(()) => Ok(()),
            Err(e) => {
                tracing::error!(error_kind = e.code(), "revocation_cache_publish_failed");
                pending_lock(&self.pending).insert(rec.idp_session_id.clone(), rec.expires_at);
                if let Err(e2) = self.valkey.incr(REVOCATIONS_EPOCH).await {
                    tracing::error!(error_kind = e2.code(), "revocation_cache_epoch_bump_failed");
                }
                if let Err(e2) = self.valkey.delete(REVOCATIONS_LOADED).await {
                    tracing::error!(
                        error_kind = e2.code(),
                        "revocation_cache_marker_clear_failed"
                    );
                }
                self.spawn_publish_retry(rec.clone());
                Err(IdentityError::CacheUnavailable)
            }
        }
    }

    fn spawn_publish_retry(&self, rec: RecordedRevocation) {
        let valkey = self.valkey.clone();
        let clock = Arc::clone(&self.clock);
        let pending = Arc::clone(&self.pending);
        tokio::spawn(async move {
            let key = revoked_session_key(&rec.idp_session_id);
            let mut delay = PUBLISH_RETRY_MIN_DELAY;
            loop {
                tokio::time::sleep(delay).await;
                let Some(ttl) = remaining(rec.expires_at, clock.now()) else {
                    break;
                };
                if valkey.set(&key, b"1", ttl).await.is_ok() {
                    tracing::info!("revocation_cache_publish_retried");
                    break;
                }
                delay = (delay * 2).min(PUBLISH_RETRY_MAX_DELAY);
            }
            pending_lock(&pending).remove(&rec.idp_session_id);
        });
    }

    /// Mencatat (transaksi sendiri, tanpa event audit) lalu menyalin pencabutan. Untuk pemanggil yang
    /// mencatat audit sendiri, pakai [`AccountDirectory::record_revocation`] di transaksinya.
    pub async fn revoke_session(
        &self,
        account_id: Uuid,
        idp_session_id: &str,
        reason: RevocationReason,
        expires_at: OffsetDateTime,
    ) -> Result<(), IdentityError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|_| IdentityError::Database)?;
        let rec = self
            .record_revocation(&mut tx, account_id, idp_session_id, reason, expires_at)
            .await?;
        tx.commit().await.map_err(|_| IdentityError::Database)?;
        self.publish_revocation(&rec).await
    }

    /// Menghapus baris pencabutan yang sudah lewat (DD A1.4; dijalankan job berkala).
    pub async fn purge_expired_revocations(&self) -> Result<u64, IdentityError> {
        repo::purge_expired(&self.pool, self.clock.now())
            .await
            .map_err(|_| IdentityError::Database)
    }
}

async fn rebuild(
    pool: &PgPool,
    valkey: &Valkey,
    clock: &dyn Clock,
) -> Result<usize, IdentityError> {
    let epoch = valkey
        .get(REVOCATIONS_EPOCH)
        .await
        .map_err(|_| IdentityError::CacheUnavailable)?;
    let now = clock.now();
    let rows = repo::active_revocations(pool, now)
        .await
        .map_err(|_| IdentityError::Database)?;
    let items: Vec<(String, Vec<u8>, Duration)> = rows
        .iter()
        .filter_map(|r| {
            remaining(r.expires_at, now)
                .map(|ttl| (revoked_session_key(&r.idp_session_id), b"1".to_vec(), ttl))
        })
        .collect();
    let n = items.len();
    // Batch supaya satu pipeline tidak terlalu besar.
    for chunk in items.chunks(500) {
        valkey
            .set_many(chunk)
            .await
            .map_err(|_| IdentityError::CacheUnavailable)?;
    }
    let marked = valkey
        .set_persistent_if_unchanged(
            REVOCATIONS_LOADED,
            b"1",
            REVOCATIONS_EPOCH,
            epoch.as_deref(),
        )
        .await
        .map_err(|_| IdentityError::CacheUnavailable)?;
    if !marked {
        // Ada pencabutan yang gagal disalin selama pembangunan ulang: penanda tidak dipasang, pemeriksaan
        // tetap lewat PostgreSQL dan pembangunan ulang berikutnya mengambil snapshot baru.
        tracing::warn!("revocation_cache_rebuild_superseded");
        return Err(IdentityError::CacheUnavailable);
    }
    tracing::info!(count = n, "revocation_cache_rebuilt");
    Ok(n)
}

impl PrincipalResolver for AccountDirectory {
    fn resolve<'a>(&'a self, ctx: &'a AuthContext) -> ResolveFuture<'a> {
        Box::pin(self.resolve_cached(ctx))
    }
}

impl SessionGuard for AccountDirectory {
    fn is_revoked<'a>(&'a self, ctx: &'a AuthContext) -> RevocationFuture<'a> {
        Box::pin(self.check_revoked(ctx))
    }
}
