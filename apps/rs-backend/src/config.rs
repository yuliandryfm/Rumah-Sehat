//! Konfigurasi proses `api` (KEPUTUSAN-132): struct bertipe, gagal cepat, tanpa membaca env langsung.

use foundation::Sensitive;
use observability::{BaseConfig, ConfigError, EnvSource};
use std::path::PathBuf;

/// Awalan URI `type` RFC 9457 bila `PROBLEM_TYPE_BASE` tidak diisi. Domain dokumentasi jenis masalah
/// `[TIDAK DIKETAHUI]` (docs/05-response-codes.md §4.7); TLD `.invalid` dicadangkan RFC 2606 sehingga
/// tidak pernah menunjuk situs nyata.
pub const DEFAULT_PROBLEM_TYPE_BASE: &str = "https://problems.ruang-sehat.invalid";

/// Konfigurasi `rs-backend api`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiConfig {
    /// `LOG_LEVEL` dan `HTTP_BIND`.
    pub base: BaseConfig,
    /// `PROBLEM_TYPE_BASE`: awalan URI `type` pada respons galat.
    pub problem_type_base: String,
}

impl ApiConfig {
    /// Memuat dari sumber; nilai tidak valid menggagalkan start tanpa mencetak nilainya.
    pub fn load(env: &EnvSource) -> Result<Self, ConfigError> {
        Ok(Self {
            base: BaseConfig::load(env)?,
            problem_type_base: env
                .optional_or("PROBLEM_TYPE_BASE", DEFAULT_PROBLEM_TYPE_BASE.to_string())?,
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

    #[test]
    fn defaults() {
        let c = ApiConfig::load(&EnvSource::default()).unwrap();
        assert_eq!(c.problem_type_base, DEFAULT_PROBLEM_TYPE_BASE);
        assert_eq!(c.base.http_bind.port(), 8080);
    }

    #[test]
    fn overrides_and_propagates_base_errors() {
        let env = EnvSource::from_pairs([("PROBLEM_TYPE_BASE", "https://d.example/p")]);
        assert_eq!(
            ApiConfig::load(&env).unwrap().problem_type_base,
            "https://d.example/p"
        );
        let bad = EnvSource::from_pairs([("HTTP_BIND", "bukan-alamat")]);
        assert!(ApiConfig::load(&bad).is_err());
    }
}
