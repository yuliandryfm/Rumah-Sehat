//! Konfigurasi proses `api` (KEPUTUSAN-132): struct bertipe, gagal cepat, tanpa membaca env langsung.

use crate::modules::platform::api::FileStorageConfig;
use crate::rate_limit::RateBudget;
use foundation::Sensitive;
use observability::{BaseConfig, ConfigError, EnvSource};
use std::path::PathBuf;

/// Awalan URI `type` RFC 9457 bila `PROBLEM_TYPE_BASE` tidak diisi. Domain dokumentasi jenis masalah
/// `[TIDAK DIKETAHUI]` (docs/05-response-codes.md §4.7); TLD `.invalid` dicadangkan RFC 2606 sehingga
/// tidak pernah menunjuk situs nyata.
pub const DEFAULT_PROBLEM_TYPE_BASE: &str = "https://problems.ruang-sehat.invalid";

/// Konfigurasi `rs-backend api`.
#[derive(Debug)]
pub struct ApiConfig {
    /// `LOG_LEVEL` dan `HTTP_BIND`.
    pub base: BaseConfig,
    /// `PROBLEM_TYPE_BASE`: awalan URI `type` pada respons galat.
    pub problem_type_base: String,
    /// `API_DATABASE_URL`: login `rs_api` (anggota `rs_app`) ke `ruang_sehat` (rahasia, F1-T92).
    pub database_url: Sensitive<String>,
    /// `API_DATABASE_MAX_CONNECTIONS` (bawaan 20 = skema fisik §10.3, 1..=100).
    pub database_max_connections: PoolSize,
    /// Validasi token realm `admin` untuk endpoint backoffice (ADR-011 butir 3 dan 8).
    pub admin_realm: RealmAuthConfig,
    /// Object storage untuk berkas (F1-T29; gambar konten F1-T93).
    pub files: FileStorageConfig,
    /// Anggaran batas laju endpoint publik per menit (F1-T94): `PUBLIC_RATE_LIMIT_PER_SOURCE_PER_MINUTE`
    /// (bawaan 300) dan `PUBLIC_RATE_LIMIT_GLOBAL_PER_MINUTE` (bawaan 3000). Angka bawaan adalah usulan
    /// teknis; lalu lintas situs yang diharapkan `[TIDAK DIKETAHUI]`. Berlaku terpisah untuk konten dan gambar.
    pub public_rate: RateBudget,
}

/// Konfigurasi validasi token satu realm. Alamat JWKS dikonfigurasi terpisah dari issuer dan tidak
/// pernah diturunkan dari token (crate `authn`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RealmAuthConfig {
    /// Nilai `iss` yang diharapkan (sama persis).
    pub issuer: String,
    /// Alamat JWKS realm (saat ini hanya `http://<loopback>:<port>/...`, `authn::HttpJwksSource`).
    pub jwks_url: String,
    /// Audience API ini; token wajib memuatnya. Nilai per lingkungan `[TIDAK DIKETAHUI]` sampai klien
    /// audience backoffice ditetapkan di realm sebagai kode.
    pub audience: String,
}

/// Ukuran pool koneksi `1..=100`; di luar rentang = tidak valid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PoolSize(u32);

impl PoolSize {
    /// Nilai.
    pub fn get(self) -> u32 {
        self.0
    }
}

impl std::str::FromStr for PoolSize {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse::<u32>() {
            Ok(n) if (1..=100).contains(&n) => Ok(Self(n)),
            _ => Err(()),
        }
    }
}

impl ApiConfig {
    /// Memuat dari sumber; nilai tidak valid menggagalkan start tanpa mencetak nilainya.
    pub fn load(env: &EnvSource) -> Result<Self, ConfigError> {
        Ok(Self {
            base: BaseConfig::load(env)?,
            problem_type_base: env
                .optional_or("PROBLEM_TYPE_BASE", DEFAULT_PROBLEM_TYPE_BASE.to_string())?,
            database_url: env.required_secret("API_DATABASE_URL")?,
            database_max_connections: env
                .optional_or("API_DATABASE_MAX_CONNECTIONS", PoolSize(20))?,
            admin_realm: RealmAuthConfig {
                issuer: env.required("AUTH_ADMIN_ISSUER")?,
                jwks_url: env.required("AUTH_ADMIN_JWKS_URL")?,
                audience: env.required("AUTH_ADMIN_AUDIENCE")?,
            },
            files: FileStorageConfig::load(env)?,
            public_rate: RateBudget {
                per_source: env.optional_or("PUBLIC_RATE_LIMIT_PER_SOURCE_PER_MINUTE", 300)?,
                global: env.optional_or("PUBLIC_RATE_LIMIT_GLOBAL_PER_MINUTE", 3_000)?,
            },
        })
    }
}

/// Ukuran batch relay audit `1..=64` (`audit_store::MAX_BATCH_EVENTS`); di luar rentang = tidak valid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RelayBatchSize(usize);

impl RelayBatchSize {
    /// Nilai.
    pub fn get(self) -> usize {
        self.0
    }
}

impl std::str::FromStr for RelayBatchSize {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse::<usize>() {
            Ok(n) if (1..=audit_store::MAX_BATCH_EVENTS).contains(&n) => Ok(Self(n)),
            _ => Err(()),
        }
    }
}

/// Penanda lingkungan yang mengizinkan jangkar berkas lokal (`DEPLOY_ENV`). Hanya `local` yang diterima:
/// `FileAnchorStore` bukan WORM, dan implementasi Object Lock NEO untuk staging/production belum ada, jadi
/// nilai lain (`staging`, `production`, ...) menggagalkan start (fail-closed, review F1-T24/T25).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalAnchorEnv;

impl std::str::FromStr for LocalAnchorEnv {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "local" { Ok(Self) } else { Err(()) }
    }
}

/// Konfigurasi `rs-backend audit-relay` (F1-T24; ADR-009 butir 3, skema fisik §3.3): proses ini hanya
/// memegang kredensial `rs_audit_relay_login` (database aplikasi) dan `rs_audit_writer_login` (database
/// audit), tidak memegang kredensial lain (coding-standards §1.4).
#[derive(Debug, Clone)]
pub struct AuditRelayConfig {
    /// `LOG_LEVEL` (dan `HTTP_BIND`, tidak dipakai proses ini).
    pub base: BaseConfig,
    /// `AUDIT_RELAY_DATABASE_URL`: login `rs_audit_relay_login` ke `ruang_sehat` (rahasia).
    pub app_database_url: Sensitive<String>,
    /// `AUDIT_WRITER_DATABASE_URL`: login `rs_audit_writer_login` ke `ruang_sehat_audit` (rahasia).
    pub audit_database_url: Sensitive<String>,
    /// `AUDIT_RELAY_BATCH_SIZE` (bawaan 50, 1..=64).
    pub batch_size: RelayBatchSize,
    /// `AUDIT_RELAY_POLL_MS`: jeda saat outbox kosong (bawaan 1000).
    pub poll_ms: u64,
    /// `AUDIT_RELAY_RETRY_MS`: jeda coba ulang setelah galat / saat siaga (bawaan 5000).
    pub retry_ms: u64,
    /// `AUDIT_ANCHOR_DIR`: direktori jangkar kepala rantai (implementasi berkas lokal, F1-T25). Staging
    /// memakai Object Lock NEO (implementasinya belum ada; skema fisik §3.3).
    pub anchor_dir: PathBuf,
    /// `AUDIT_ANCHOR_INTERVAL_SECS`: jeda minimum antar-jangkar (bawaan 3600, ADR-009 butir 5).
    pub anchor_interval_secs: u64,
    /// `DEPLOY_ENV` wajib `local` selama jangkar memakai berkas lokal.
    pub anchor_env: LocalAnchorEnv,
}

impl AuditRelayConfig {
    /// Memuat dari sumber; gagal cepat tanpa mencetak nilai.
    pub fn load(env: &EnvSource) -> Result<Self, ConfigError> {
        Ok(Self {
            base: BaseConfig::load(env)?,
            app_database_url: env.required_secret("AUDIT_RELAY_DATABASE_URL")?,
            audit_database_url: env.required_secret("AUDIT_WRITER_DATABASE_URL")?,
            batch_size: env.optional_or(
                "AUDIT_RELAY_BATCH_SIZE",
                RelayBatchSize(audit::relay::DEFAULT_BATCH_SIZE),
            )?,
            poll_ms: env.optional_or("AUDIT_RELAY_POLL_MS", 1_000)?,
            retry_ms: env.optional_or("AUDIT_RELAY_RETRY_MS", 5_000)?,
            anchor_dir: env.required("AUDIT_ANCHOR_DIR")?,
            anchor_interval_secs: env.optional_or("AUDIT_ANCHOR_INTERVAL_SECS", 3_600)?,
            anchor_env: env.required("DEPLOY_ENV")?,
        })
    }
}

/// Konfigurasi `rs-backend audit-verify` (F1-T25; ADR-009 butir 5, KEPUTUSAN-141, skema fisik §3.3):
/// hanya kredensial `rs_audit_verifier_login` dan akses baca jangkar WORM.
#[derive(Debug, Clone)]
pub struct AuditVerifyConfig {
    /// `LOG_LEVEL` (dan `HTTP_BIND`, tidak dipakai proses ini).
    pub base: BaseConfig,
    /// `AUDIT_VERIFIER_DATABASE_URL`: login `rs_audit_verifier_login` ke `ruang_sehat_audit` (rahasia).
    pub database_url: Sensitive<String>,
    /// `AUDIT_ANCHOR_DIR`: direktori jangkar kepala rantai (berkas lokal).
    pub anchor_dir: PathBuf,
    /// `DEPLOY_ENV` wajib `local` selama jangkar memakai berkas lokal.
    pub anchor_env: LocalAnchorEnv,
    /// `AUDIT_VERIFY_INTERVAL_SECS`: jeda antar-verifikasi (bawaan 86400, harian).
    pub interval_secs: u64,
    /// `AUDIT_VERIFY_RETRY_MS`: jeda coba ulang setelah galat / saat siaga (bawaan 60000).
    pub retry_ms: u64,
    /// `AUDIT_VERIFY_ONCE`: `true` = satu verifikasi lalu keluar (pemicu manual; kode keluar 0 hanya bila
    /// `match`). Bawaan `false`.
    pub once: bool,
}

impl AuditVerifyConfig {
    /// Memuat dari sumber; gagal cepat tanpa mencetak nilai.
    pub fn load(env: &EnvSource) -> Result<Self, ConfigError> {
        Ok(Self {
            base: BaseConfig::load(env)?,
            database_url: env.required_secret("AUDIT_VERIFIER_DATABASE_URL")?,
            anchor_dir: env.required("AUDIT_ANCHOR_DIR")?,
            anchor_env: env.required("DEPLOY_ENV")?,
            interval_secs: env.optional_or("AUDIT_VERIFY_INTERVAL_SECS", 86_400)?,
            retry_ms: env.optional_or("AUDIT_VERIFY_RETRY_MS", 60_000)?,
            once: env.optional_or("AUDIT_VERIFY_ONCE", false)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use observability::ConfigErrorKind;

    #[test]
    fn audit_relay_requires_both_credentials_and_bounds_batch_size() {
        let ok = EnvSource::from_pairs([
            (
                "AUDIT_RELAY_DATABASE_URL",
                "postgres://relay@localhost/ruang_sehat",
            ),
            (
                "AUDIT_WRITER_DATABASE_URL",
                "postgres://writer@localhost/ruang_sehat_audit",
            ),
            ("AUDIT_ANCHOR_DIR", "/var/lib/ruang-sehat/anchor"),
            ("DEPLOY_ENV", "local"),
        ]);
        let c = AuditRelayConfig::load(&ok).unwrap();
        assert_eq!(c.batch_size.get(), 50);
        assert_eq!((c.poll_ms, c.retry_ms), (1_000, 5_000));
        assert_eq!(c.anchor_interval_secs, 3_600);
        // Debug tidak membocorkan URL berkredensial.
        assert!(!format!("{c:?}").contains("postgres://"));

        let missing = EnvSource::from_pairs([("AUDIT_RELAY_DATABASE_URL", "postgres://x")]);
        let e = AuditRelayConfig::load(&missing).unwrap_err();
        assert_eq!(
            (e.kind(), e.name()),
            (ConfigErrorKind::Missing, "AUDIT_WRITER_DATABASE_URL")
        );

        for bad in ["0", "65", "-1", "x"] {
            let env = EnvSource::from_pairs([
                ("AUDIT_RELAY_DATABASE_URL", "postgres://x"),
                ("AUDIT_WRITER_DATABASE_URL", "postgres://y"),
                ("AUDIT_RELAY_BATCH_SIZE", bad),
                ("AUDIT_ANCHOR_DIR", "/tmp/a"),
                ("DEPLOY_ENV", "local"),
            ]);
            let e = AuditRelayConfig::load(&env).unwrap_err();
            assert_eq!(e.kind(), ConfigErrorKind::Invalid, "batch {bad}");
        }
    }

    #[test]
    fn audit_verify_needs_verifier_credential_and_anchor_dir() {
        let ok = EnvSource::from_pairs([
            (
                "AUDIT_VERIFIER_DATABASE_URL",
                "postgres://verifier@localhost/a",
            ),
            ("AUDIT_ANCHOR_DIR", "/var/lib/ruang-sehat/anchor"),
            ("DEPLOY_ENV", "local"),
        ]);
        let c = AuditVerifyConfig::load(&ok).unwrap();
        assert_eq!(
            (c.interval_secs, c.retry_ms, c.once),
            (86_400, 60_000, false)
        );
        assert!(!format!("{c:?}").contains("postgres://"));
        let no_dir =
            EnvSource::from_pairs([("AUDIT_VERIFIER_DATABASE_URL", "postgres://verifier@x/a")]);
        assert_eq!(
            AuditVerifyConfig::load(&no_dir).unwrap_err().name(),
            "AUDIT_ANCHOR_DIR"
        );
        let bad_once = EnvSource::from_pairs([
            ("AUDIT_VERIFIER_DATABASE_URL", "postgres://verifier@x/a"),
            ("AUDIT_ANCHOR_DIR", "/tmp/a"),
            ("DEPLOY_ENV", "local"),
            ("AUDIT_VERIFY_ONCE", "ya"),
        ]);
        assert_eq!(
            AuditVerifyConfig::load(&bad_once).unwrap_err().kind(),
            ConfigErrorKind::Invalid
        );
    }

    #[test]
    fn file_anchor_store_is_rejected_outside_local() {
        for deploy in [Some("staging"), Some("production"), Some("Local"), None] {
            let mut relay = vec![
                ("AUDIT_RELAY_DATABASE_URL", "postgres://x"),
                ("AUDIT_WRITER_DATABASE_URL", "postgres://y"),
                ("AUDIT_ANCHOR_DIR", "/tmp/a"),
            ];
            let mut verify = vec![
                ("AUDIT_VERIFIER_DATABASE_URL", "postgres://z"),
                ("AUDIT_ANCHOR_DIR", "/tmp/a"),
            ];
            if let Some(d) = deploy {
                relay.push(("DEPLOY_ENV", d));
                verify.push(("DEPLOY_ENV", d));
            }
            let e = AuditRelayConfig::load(&EnvSource::from_pairs(relay)).unwrap_err();
            assert_eq!(e.name(), "DEPLOY_ENV", "{deploy:?}");
            let e = AuditVerifyConfig::load(&EnvSource::from_pairs(verify)).unwrap_err();
            assert_eq!(e.name(), "DEPLOY_ENV", "{deploy:?}");
        }
    }

    fn api_env(extra: &[(&'static str, &'static str)]) -> EnvSource {
        let mut all = vec![
            ("API_DATABASE_URL", "postgres://127.0.0.1:15432/ruang_sehat"),
            (
                "AUTH_ADMIN_ISSUER",
                "https://auth.ruang-sehat.invalid/realms/admin",
            ),
            (
                "AUTH_ADMIN_JWKS_URL",
                "http://127.0.0.1:18081/realms/admin/protocol/openid-connect/certs",
            ),
            ("AUTH_ADMIN_AUDIENCE", "rs-backoffice-api"),
            ("DEPLOY_ENV", "local"),
            ("OBJECT_STORAGE_ENDPOINT", "http://127.0.0.1:18333"),
            ("OBJECT_STORAGE_REGION", "us-east-1"),
            ("OBJECT_STORAGE_ADDRESSING", "path"),
            ("OBJECT_STORAGE_ACCESS_KEY_ID", "sintetis-lokal-s3-akses"),
            (
                "OBJECT_STORAGE_SECRET_ACCESS_KEY",
                "sintetis-uji-rahasia-0123456789abcdef",
            ),
            ("OBJECT_STORAGE_BUCKET_PREFIX", "rs-local-"),
        ];
        for (k, v) in extra {
            all.retain(|(ek, _)| ek != k);
            all.push((k, v));
        }
        EnvSource::from_pairs(all)
    }

    #[test]
    fn defaults() {
        let c = ApiConfig::load(&api_env(&[])).unwrap();
        assert_eq!(c.problem_type_base, DEFAULT_PROBLEM_TYPE_BASE);
        assert_eq!(c.base.http_bind.port(), 8080);
        assert_eq!(c.database_max_connections.get(), 20);
        assert_eq!(c.admin_realm.audience, "rs-backoffice-api");
        assert_eq!(
            c.public_rate,
            RateBudget {
                per_source: 300,
                global: 3_000
            }
        );
        // Rahasia tidak muncul di Debug (KEPUTUSAN-132).
        let dbg = format!("{c:?}");
        assert!(!dbg.contains("postgres://"), "{dbg}");
        assert!(!dbg.contains("0123456789abcdef"), "{dbg}");
    }

    #[test]
    fn overrides_and_propagates_base_errors() {
        let env = api_env(&[("PROBLEM_TYPE_BASE", "https://d.example/p")]);
        assert_eq!(
            ApiConfig::load(&env).unwrap().problem_type_base,
            "https://d.example/p"
        );
        let bad = api_env(&[("HTTP_BIND", "bukan-alamat")]);
        assert!(ApiConfig::load(&bad).is_err());
    }

    #[test]
    fn api_requires_database_auth_and_storage_and_bounds_pool() {
        for missing in [
            "API_DATABASE_URL",
            "AUTH_ADMIN_ISSUER",
            "AUTH_ADMIN_JWKS_URL",
            "AUTH_ADMIN_AUDIENCE",
            "OBJECT_STORAGE_ENDPOINT",
        ] {
            let pairs: Vec<(&str, &str)> = [
                ("API_DATABASE_URL", "postgres://x"),
                ("AUTH_ADMIN_ISSUER", "https://i"),
                ("AUTH_ADMIN_JWKS_URL", "http://127.0.0.1:1/j"),
                ("AUTH_ADMIN_AUDIENCE", "a"),
                ("DEPLOY_ENV", "local"),
                ("OBJECT_STORAGE_ENDPOINT", "http://127.0.0.1:18333"),
                ("OBJECT_STORAGE_REGION", "us-east-1"),
                ("OBJECT_STORAGE_ADDRESSING", "path"),
                ("OBJECT_STORAGE_ACCESS_KEY_ID", "k"),
                (
                    "OBJECT_STORAGE_SECRET_ACCESS_KEY",
                    "sintetis-uji-rahasia-0123456789abcdef",
                ),
                ("OBJECT_STORAGE_BUCKET_PREFIX", "rs-local-"),
            ]
            .into_iter()
            .filter(|(k, _)| *k != missing)
            .collect();
            let e = ApiConfig::load(&EnvSource::from_pairs(pairs)).unwrap_err();
            assert_eq!(
                (e.kind(), e.name()),
                (ConfigErrorKind::Missing, missing),
                "{missing}"
            );
        }
        for bad in ["0", "101", "x"] {
            let e =
                ApiConfig::load(&api_env(&[("API_DATABASE_MAX_CONNECTIONS", bad)])).unwrap_err();
            assert_eq!(e.name(), "API_DATABASE_MAX_CONNECTIONS", "{bad}");
        }
    }
}
