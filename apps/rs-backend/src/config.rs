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

/// Lingkungan penerapan (`DEPLOY_ENV`) untuk `email-relay`: `local`, `staging`, atau `production`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeployEnv {
    /// Laptop developer dan test.
    Local,
    /// Staging.
    Staging,
    /// Production.
    Production,
}

impl std::str::FromStr for DeployEnv {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "local" => Ok(Self::Local),
            "staging" => Ok(Self::Staging),
            "production" => Ok(Self::Production),
            _ => Err(()),
        }
    }
}

/// Daftar izin pengirim `EMAIL_RELAY_ALLOWED_FROM` (dipisah koma, minimal satu alamat valid).
#[derive(Debug, Clone)]
pub struct AllowedFrom(pub Vec<email_relay::address::Mailbox>);

impl std::str::FromStr for AllowedFrom {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let list = s
            .split(',')
            .map(|a| email_relay::address::Mailbox::parse(a.trim()).ok_or(()))
            .collect::<Result<Vec<_>, _>>()?;
        if list.is_empty() {
            Err(())
        } else {
            Ok(Self(list))
        }
    }
}

/// Konfigurasi `rs-backend email-relay` (F1-T36, ADR-026): relay SMTP per host ke SES API v2 Jakarta.
/// Proses ini hanya memegang token OpenBao yang dapat membaca kunci SES (policy per host, KEPUTUSAN-79) dan
/// sertifikat TLS relay; tidak memegang kredensial database.
#[derive(Debug, Clone)]
pub struct EmailRelayConfig {
    /// `LOG_LEVEL` (dan `HTTP_BIND`, tidak dipakai proses ini).
    pub base: BaseConfig,
    /// `DEPLOY_ENV`: `SES_LOCAL_ENDPOINT` hanya diterima bila `local`.
    pub deploy_env: DeployEnv,
    /// `EMAIL_RELAY_BIND`: alamat listen SMTP di jaringan Docker khusus host (wajib, tanpa bawaan).
    pub bind: std::net::SocketAddr,
    /// `EMAIL_RELAY_HOSTNAME` (bawaan `email-relay`): nama di sapaan/EHLO, sama dengan SAN sertifikat.
    pub hostname: String,
    /// `EMAIL_RELAY_TLS_CERT_FILE`: rantai sertifikat PEM relay (CA internal).
    pub tls_cert_file: PathBuf,
    /// `EMAIL_RELAY_TLS_KEY_FILE`: kunci privat PEM relay.
    pub tls_key_file: PathBuf,
    /// `EMAIL_RELAY_ALLOWED_FROM`: daftar izin pengirim.
    pub allowed_from: AllowedFrom,
    /// `EMAIL_RELAY_MAX_MESSAGE_BYTES` (bawaan 262144).
    pub max_message_bytes: usize,
    /// `EMAIL_RELAY_GLOBAL_PER_MINUTE` (bawaan 60; kuota SES bawaan 1 email/detik [SRC-294]).
    pub global_per_minute: u32,
    /// `EMAIL_RELAY_PER_RECIPIENT_PER_HOUR` (bawaan 5).
    pub per_recipient_per_hour: u32,
    /// `EMAIL_RELAY_MAX_CONNECTIONS` (bawaan 16).
    pub max_connections: usize,
    /// `EMAIL_RELAY_IDLE_TIMEOUT_SECS` (bawaan 30).
    pub idle_timeout_secs: u64,
    /// `EMAIL_RELAY_SESSION_TIMEOUT_SECS` (bawaan 120).
    pub session_timeout_secs: u64,
    /// `SES_CONFIGURATION_SET`: configuration set email Keycloak (wajib TLS ke penerima, KEPUTUSAN-229).
    pub ses_configuration_set: String,
    /// `SES_TIMEOUT_MS` (bawaan 8000, maksimum 9000; < batas tulis Keycloak 10000 ms).
    pub ses_timeout_ms: u64,
    /// `SES_CA_FILE` (bawaan `/etc/ssl/certs/ca-certificates.crt`): CA tepercaya untuk TLS ke SES.
    pub ses_ca_file: PathBuf,
    /// `SES_LOCAL_ENDPOINT` (opsional, hanya `DEPLOY_ENV=local`): tiruan SES HTTPS untuk test.
    pub ses_local_endpoint: Option<String>,
    /// `OPENBAO_ADDR`: alamat OpenBao.
    pub openbao_addr: String,
    /// `OPENBAO_TOKEN`: token OpenBao proses relay (rahasia).
    pub openbao_token: Sensitive<String>,
    /// `SES_CREDENTIALS_KV_MOUNT` (bawaan `secret`): mount KV v2.
    pub ses_kv_mount: String,
    /// `SES_CREDENTIALS_KV_PATH`: path rahasia berisi `access_key_id` dan `secret_access_key`.
    pub ses_kv_path: String,
}

impl EmailRelayConfig {
    /// Memuat dari sumber; gagal cepat tanpa mencetak nilai.
    pub fn load(env: &EnvSource) -> Result<Self, ConfigError> {
        let deploy_env: DeployEnv = env.required("DEPLOY_ENV")?;
        let ses_local_endpoint: Option<String> =
            match env.optional_or("SES_LOCAL_ENDPOINT", String::new())? {
                v if v.is_empty() => None,
                // Endpoint selain SES Jakarta hanya untuk tiruan SES lokal (ADR-026: region ap-southeast-3).
                _ if deploy_env != DeployEnv::Local => {
                    return Err(ConfigError::invalid("SES_LOCAL_ENDPOINT"));
                }
                v => Some(v),
            };
        Ok(Self {
            base: BaseConfig::load(env)?,
            deploy_env,
            bind: env.required("EMAIL_RELAY_BIND")?,
            hostname: env.optional_or("EMAIL_RELAY_HOSTNAME", "email-relay".to_string())?,
            tls_cert_file: env.required("EMAIL_RELAY_TLS_CERT_FILE")?,
            tls_key_file: env.required("EMAIL_RELAY_TLS_KEY_FILE")?,
            allowed_from: env.required("EMAIL_RELAY_ALLOWED_FROM")?,
            max_message_bytes: env.optional_or("EMAIL_RELAY_MAX_MESSAGE_BYTES", 262_144)?,
            global_per_minute: env.optional_or("EMAIL_RELAY_GLOBAL_PER_MINUTE", 60)?,
            per_recipient_per_hour: env.optional_or("EMAIL_RELAY_PER_RECIPIENT_PER_HOUR", 5)?,
            max_connections: env.optional_or("EMAIL_RELAY_MAX_CONNECTIONS", 16)?,
            idle_timeout_secs: env.optional_or("EMAIL_RELAY_IDLE_TIMEOUT_SECS", 30)?,
            session_timeout_secs: env.optional_or("EMAIL_RELAY_SESSION_TIMEOUT_SECS", 120)?,
            ses_configuration_set: env.required("SES_CONFIGURATION_SET")?,
            ses_timeout_ms: env.optional_or("SES_TIMEOUT_MS", 8_000)?,
            ses_ca_file: env.optional_or(
                "SES_CA_FILE",
                PathBuf::from("/etc/ssl/certs/ca-certificates.crt"),
            )?,
            ses_local_endpoint,
            openbao_addr: env.required("OPENBAO_ADDR")?,
            openbao_token: env.required_secret("OPENBAO_TOKEN")?,
            ses_kv_mount: env.optional_or("SES_CREDENTIALS_KV_MOUNT", "secret".to_string())?,
            ses_kv_path: env.required("SES_CREDENTIALS_KV_PATH")?,
        })
    }

    /// Pengaturan relay (divalidasi lagi oleh `email_relay::Relay::new`).
    pub fn relay_settings(&self) -> email_relay::RelaySettings {
        email_relay::RelaySettings {
            hostname: self.hostname.clone(),
            allowed_from: self.allowed_from.0.clone(),
            max_message_bytes: self.max_message_bytes,
            global_per_minute: self.global_per_minute,
            per_recipient_per_hour: self.per_recipient_per_hour,
            max_connections: self.max_connections,
            idle_timeout: std::time::Duration::from_secs(self.idle_timeout_secs),
            session_timeout: std::time::Duration::from_secs(self.session_timeout_secs),
        }
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

    fn email_relay_env(extra: &[(&'static str, &'static str)]) -> EnvSource {
        let mut v = vec![
            ("DEPLOY_ENV", "local"),
            ("EMAIL_RELAY_BIND", "127.0.0.1:2525"),
            ("EMAIL_RELAY_TLS_CERT_FILE", "/run/relay/cert.pem"),
            ("EMAIL_RELAY_TLS_KEY_FILE", "/run/relay/key.pem"),
            (
                "EMAIL_RELAY_ALLOWED_FROM",
                "noreply@uji.invalid, alert@uji.invalid",
            ),
            ("SES_CONFIGURATION_SET", "rs-keycloak"),
            ("OPENBAO_ADDR", "http://127.0.0.1:8200"),
            ("OPENBAO_TOKEN", "token-uji-sintetis"),
            ("SES_CREDENTIALS_KV_PATH", "ruang-sehat/ses/email-relay"),
        ];
        v.extend_from_slice(extra);
        EnvSource::from_pairs(v)
    }

    #[test]
    fn email_relay_loads_with_defaults_and_hides_token() {
        let c = EmailRelayConfig::load(&email_relay_env(&[])).unwrap();
        assert_eq!(c.allowed_from.0.len(), 2);
        assert_eq!((c.ses_timeout_ms, c.max_message_bytes), (8_000, 262_144));
        assert_eq!((c.global_per_minute, c.per_recipient_per_hour), (60, 5));
        assert_eq!(c.ses_kv_mount, "secret");
        assert!(c.ses_local_endpoint.is_none());
        assert!(c.relay_settings().validate().is_ok());
        assert!(!format!("{c:?}").contains("token-uji-sintetis"));
    }

    #[test]
    fn email_relay_rejects_invalid_values_by_name() {
        for (k, v) in [
            ("EMAIL_RELAY_ALLOWED_FROM", "bukan-alamat"),
            ("EMAIL_RELAY_ALLOWED_FROM", "a@uji.invalid,,b@uji.invalid"),
            ("EMAIL_RELAY_BIND", "semua"),
            ("DEPLOY_ENV", "dev"),
            ("SES_TIMEOUT_MS", "x"),
        ] {
            let e = EmailRelayConfig::load(&email_relay_env(&[(k, v)])).unwrap_err();
            assert_eq!(
                (e.kind(), e.name()),
                (ConfigErrorKind::Invalid, k),
                "{k}={v}"
            );
        }
        let missing = EnvSource::from_pairs([("DEPLOY_ENV", "local")]);
        assert_eq!(
            EmailRelayConfig::load(&missing).unwrap_err().kind(),
            ConfigErrorKind::Missing
        );
    }

    #[test]
    fn ses_local_endpoint_only_in_local() {
        let ok = EmailRelayConfig::load(&email_relay_env(&[(
            "SES_LOCAL_ENDPOINT",
            "https://127.0.0.1:18443",
        )]))
        .unwrap();
        assert_eq!(
            ok.ses_local_endpoint.as_deref(),
            Some("https://127.0.0.1:18443")
        );
        for d in ["staging", "production"] {
            let env = EnvSource::from_pairs([
                ("DEPLOY_ENV", d),
                ("SES_LOCAL_ENDPOINT", "https://127.0.0.1:18443"),
            ]);
            let e = EmailRelayConfig::load(&env).unwrap_err();
            assert_eq!(e.name(), "SES_LOCAL_ENDPOINT", "{d}");
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
