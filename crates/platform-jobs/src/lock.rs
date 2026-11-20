//! Kunci job tunggal: kunci advisory tingkat sesi `pg_try_advisory_lock` (KEPUTUSAN-74, KEPUTUSAN-103,
//! docs/05-physical-schema.md §6.4) [SRC-234].
//!
//! Kunci dipegang di **koneksi khusus di luar pool** ([`PgConnection`] milik [`SingletonLock`]), dan
//! pekerjaan job tunggal (misalnya batch relay audit) dijalankan **lewat koneksi yang sama**
//! ([`SingletonLock::connection`]). Bila koneksi putus, server melepas kunci saat sesi berakhir, termasuk
//! saat klien terputus tidak wajar [SRC-512], dan pekerjaan yang sedang berjalan ikut gagal, sehingga dua
//! pemegang tidak pernah bekerja bersamaan. Instans siaga menyimpan koneksinya ([`LockAttempt::Busy`]) dan
//! mencoba ulang berkala.

use crate::error::{JobError, JobErrorKind};
use sqlx::PgConnection;
use std::fmt;

/// Satu kunci di registri (§6.4). Nilai tidak pernah dipakai ulang untuk pemegang lain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LockKey {
    key: i64,
    database: &'static str,
    holder: &'static str,
}

impl LockKey {
    const fn new(key: i64, database: &'static str, holder: &'static str) -> Self {
        Self {
            key,
            database,
            holder,
        }
    }

    /// Nilai kunci `bigint`.
    pub fn key(&self) -> i64 {
        self.key
    }

    /// Database tempat kunci diambil.
    pub fn database(&self) -> &'static str {
        self.database
    }

    /// Pemegang (kode, untuk log).
    pub fn holder(&self) -> &'static str {
        self.holder
    }
}

impl fmt::Display for LockKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}({})", self.holder, self.key)
    }
}

/// `rs-backend audit-relay` di `ruang_sehat`.
pub const RS_AUDIT_RELAY: LockKey = LockKey::new(7_001_001, "ruang_sehat", "rs_audit_relay");
/// Pengambil event Keycloak (di `rs-backend worker`) di `ruang_sehat`.
pub const RS_KEYCLOAK_EVENT_PULLER: LockKey =
    LockKey::new(7_001_002, "ruang_sehat", "rs_keycloak_event_puller");
/// `rs-backend audit-verify` di `ruang_sehat_audit`.
pub const RS_AUDIT_VERIFY: LockKey =
    LockKey::new(7_001_003, "ruang_sehat_audit", "rs_audit_verify");
/// `rs-backend privacy-worker` di `ruang_sehat`.
pub const RS_PRIVACY_WORKER: LockKey = LockKey::new(7_001_004, "ruang_sehat", "rs_privacy_worker");
/// `rme-backend audit-relay` di `rme`.
pub const RME_AUDIT_RELAY: LockKey = LockKey::new(7_002_001, "rme", "rme_audit_relay");
/// Dicadangkan untuk pengambil event Keycloak realm `rme`; pemegangnya belum ditetapkan (§6.4).
pub const RME_KEYCLOAK_EVENT_PULLER_RESERVED: LockKey =
    LockKey::new(7_002_002, "rme", "rme_keycloak_event_puller_reserved");
/// `rme-backend audit-verify` di `rme_audit`.
pub const RME_AUDIT_VERIFY: LockKey = LockKey::new(7_002_003, "rme_audit", "rme_audit_verify");
/// `rme-backend privacy-worker` di `rme`.
pub const RME_PRIVACY_WORKER: LockKey = LockKey::new(7_002_004, "rme", "rme_privacy_worker");

/// Seluruh registri kunci advisory (docs/05-physical-schema.md §6.4, KEPUTUSAN-103, KEPUTUSAN-153).
pub const REGISTRY: &[LockKey] = &[
    RS_AUDIT_RELAY,
    RS_KEYCLOAK_EVENT_PULLER,
    RS_AUDIT_VERIFY,
    RS_PRIVACY_WORKER,
    RME_AUDIT_RELAY,
    RME_KEYCLOAK_EVENT_PULLER_RESERVED,
    RME_AUDIT_VERIFY,
    RME_PRIVACY_WORKER,
];

/// Hasil [`SingletonLock::try_acquire`].
#[derive(Debug)]
pub enum LockAttempt {
    /// Kunci dipegang; pekerjaan job tunggal boleh berjalan lewat koneksi ini.
    Acquired(SingletonLock),
    /// Kunci dipegang sesi lain; koneksi dikembalikan untuk dicoba ulang nanti.
    Busy(PgConnection),
}

/// Kunci job tunggal yang sedang dipegang beserta koneksinya.
#[derive(Debug)]
pub struct SingletonLock {
    conn: PgConnection,
    key: LockKey,
}

impl SingletonLock {
    /// Mencoba mengambil `key` di koneksi khusus `conn` tanpa menunggu.
    ///
    /// `conn` harus koneksi tersendiri, bukan pinjaman pool: tipe [`PgConnection`] memastikannya
    /// (koneksi pool hanya dapat dipakai setelah dilepas dari pool dengan `detach`).
    pub async fn try_acquire(
        mut conn: PgConnection,
        key: LockKey,
    ) -> Result<LockAttempt, JobError> {
        let locked =
            sqlx::query_scalar!(r#"SELECT pg_try_advisory_lock($1) AS "locked!""#, key.key)
                .fetch_one(&mut conn)
                .await?;
        if locked {
            tracing::info!(
                lock_holder = key.holder,
                lock_key = key.key,
                "singleton_lock_acquired"
            );
            Ok(LockAttempt::Acquired(Self { conn, key }))
        } else {
            Ok(LockAttempt::Busy(conn))
        }
    }

    /// Kunci yang dipegang.
    pub fn key(&self) -> LockKey {
        self.key
    }

    /// Koneksi pemegang kunci: pekerjaan job tunggal wajib lewat koneksi ini (§6.4 butir 1).
    pub fn connection(&mut self) -> &mut PgConnection {
        &mut self.conn
    }

    /// Melepas kunci (`pg_advisory_unlock`) dan mengembalikan koneksinya. Bila server menyatakan
    /// kunci tidak dipegang sesi ini, hasilnya [`JobErrorKind::LockNotHeld`].
    pub async fn release(mut self) -> Result<PgConnection, JobError> {
        let released = sqlx::query_scalar!(
            r#"SELECT pg_advisory_unlock($1) AS "released!""#,
            self.key.key
        )
        .fetch_one(&mut self.conn)
        .await?;
        if released {
            tracing::info!(
                lock_holder = self.key.holder,
                lock_key = self.key.key,
                "singleton_lock_released"
            );
            Ok(self.conn)
        } else {
            Err(JobError::new(JobErrorKind::LockNotHeld))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn registry_keys_are_unique_and_match_physical_schema() {
        let keys: BTreeSet<i64> = REGISTRY.iter().map(LockKey::key).collect();
        assert_eq!(keys.len(), REGISTRY.len(), "kunci advisory dipakai ulang");
        let holders: BTreeSet<&str> = REGISTRY.iter().map(LockKey::holder).collect();
        assert_eq!(holders.len(), REGISTRY.len());
        // Nilai persis docs/05-physical-schema.md §6.4.
        let expected = [
            (7_001_001, "ruang_sehat"),
            (7_001_002, "ruang_sehat"),
            (7_001_003, "ruang_sehat_audit"),
            (7_001_004, "ruang_sehat"),
            (7_002_001, "rme"),
            (7_002_002, "rme"),
            (7_002_003, "rme_audit"),
            (7_002_004, "rme"),
        ];
        let actual: Vec<(i64, &str)> = REGISTRY.iter().map(|k| (k.key(), k.database())).collect();
        assert_eq!(actual, expected);
    }

    #[test]
    fn display_shows_holder_and_key() {
        assert_eq!(RS_AUDIT_RELAY.to_string(), "rs_audit_relay(7001001)");
    }
}
