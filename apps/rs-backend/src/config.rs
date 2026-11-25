//! Konfigurasi proses `api` (KEPUTUSAN-132): struct bertipe, gagal cepat, tanpa membaca env langsung.

use crate::modules::identity::api::{IdpWorkerConfig, KeycloakEventConfig};
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
    /// `API_VALKEY_URL`: Valkey aplikasi untuk cache akun dan pencabutan sesi (F1-T33; rahasia). Bila Valkey
    /// tidak tersedia, resolver memakai PostgreSQL (RT-18).
    pub valkey_url: Sensitive<String>,
    /// `API_VALKEY_OP_TIMEOUT_MS` (bawaan `valkey::DEFAULT_OP_TIMEOUT_MS`).
    pub valkey_op_timeout_ms: u64,
    /// Validasi token realm `admin` untuk endpoint backoffice (ADR-011 butir 3 dan 8).
    pub admin_realm: RealmAuthConfig,
    /// Validasi token realm `pasien` untuk endpoint `/patient/v1` (F1-T61, ADR-011 butir 3 dan 8).
    pub patient_realm: RealmAuthConfig,
    /// Validasi token realm `korporat` untuk endpoint `/v1/corporate` (F1-T100, ADR-027 butir 2 dan 3, ADR-011
    /// butir 8). Realm Keycloak korporat sebagai kode dibuat F1-T85.
    pub corporate_realm: RealmAuthConfig,
    /// Kunci data identitas (F1-T61, KEPUTUSAN-293): OpenBao Transit untuk DEK dan kunci blind index NIK
    /// dari OpenBao KV yang dimuat saat start (gagal cepat).
    pub identity_keys: IdentityKeysConfig,
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

/// Kunci data identitas proses `api` (F1-T61, KEPUTUSAN-293, ADR-013, KEPUTUSAN-79).
#[derive(Debug)]
pub struct IdentityKeysConfig {
    /// `OPENBAO_ADDR`: alamat OpenBao.
    pub openbao_addr: String,
    /// `OPENBAO_TOKEN`: token OpenBao proses api (identitas proses, KEPUTUSAN-79; rahasia).
    pub openbao_token: Sensitive<String>,
    /// `API_DEK_TRANSIT_MOUNT` (bawaan `transit`): mount Transit.
    pub transit_mount: String,
    /// `API_DEK_TRANSIT_KEY` (bawaan `rs-dek-kek`, skema fisik §7.3): kunci Transit KEK Ruang Sehat.
    pub transit_key: String,
    /// `NIK_BLIND_INDEX_KV_MOUNT` (bawaan `secret`): mount KV v2 kunci blind index NIK.
    pub blind_index_kv_mount: String,
    /// `NIK_BLIND_INDEX_KV_PATH`: path KV kunci blind index NIK (wajib; tanpa nilai bawaan).
    pub blind_index_kv_path: String,
    /// `API_DEK_CACHE_TTL_SECONDS` (1..=3600): umur DEK di cache memori. Wajib eksplisit: nilai production
    /// belum ditetapkan dokumen mana pun (crate `crypto`, `KeyringConfig`).
    pub dek_cache_ttl: DekCacheTtl,
    /// `API_DEK_CACHE_CAPACITY` (1..=100000): jumlah DEK maksimum di cache. Wajib eksplisit.
    pub dek_cache_capacity: DekCacheCapacity,
}

/// Umur cache DEK dalam detik `1..=3600`; di luar rentang = tidak valid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DekCacheTtl(u32);

impl DekCacheTtl {
    /// Nilai dalam detik.
    pub fn seconds(self) -> u32 {
        self.0
    }
}

impl std::str::FromStr for DekCacheTtl {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse::<u32>() {
            Ok(n) if (1..=3_600).contains(&n) => Ok(Self(n)),
            _ => Err(()),
        }
    }
}

/// Kapasitas cache DEK `1..=100000`; di luar rentang = tidak valid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DekCacheCapacity(usize);

impl DekCacheCapacity {
    /// Nilai.
    pub fn get(self) -> usize {
        self.0
    }
}

impl std::str::FromStr for DekCacheCapacity {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse::<usize>() {
            Ok(n) if (1..=100_000).contains(&n) => Ok(Self(n)),
            _ => Err(()),
        }
    }
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
            valkey_url: env.required_secret("API_VALKEY_URL")?,
            valkey_op_timeout_ms: env
                .optional_or("API_VALKEY_OP_TIMEOUT_MS", valkey::DEFAULT_OP_TIMEOUT_MS)?,
            admin_realm: RealmAuthConfig {
                issuer: env.required("AUTH_ADMIN_ISSUER")?,
                jwks_url: env.required("AUTH_ADMIN_JWKS_URL")?,
                audience: env.required("AUTH_ADMIN_AUDIENCE")?,
            },
            patient_realm: RealmAuthConfig {
                issuer: env.required("AUTH_PATIENT_ISSUER")?,
                jwks_url: env.required("AUTH_PATIENT_JWKS_URL")?,
                audience: env.required("AUTH_PATIENT_AUDIENCE")?,
            },
            corporate_realm: RealmAuthConfig {
                issuer: env.required("AUTH_CORPORATE_ISSUER")?,
                jwks_url: env.required("AUTH_CORPORATE_JWKS_URL")?,
                audience: env.required("AUTH_CORPORATE_AUDIENCE")?,
            },
            identity_keys: IdentityKeysConfig {
                openbao_addr: env.required("OPENBAO_ADDR")?,
                openbao_token: env.required_secret("OPENBAO_TOKEN")?,
                transit_mount: env.optional_or("API_DEK_TRANSIT_MOUNT", "transit".to_string())?,
                transit_key: env.optional_or("API_DEK_TRANSIT_KEY", "rs-dek-kek".to_string())?,
                blind_index_kv_mount: env
                    .optional_or("NIK_BLIND_INDEX_KV_MOUNT", "secret".to_string())?,
                blind_index_kv_path: env.required("NIK_BLIND_INDEX_KV_PATH")?,
                dek_cache_ttl: env.required("API_DEK_CACHE_TTL_SECONDS")?,
                dek_cache_capacity: env.required("API_DEK_CACHE_CAPACITY")?,
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

/// Konfigurasi `rs-backend worker` (F1-T34): login `rs_worker` ke `ruang_sehat` dan pengambil event Keycloak
/// (ADR-011 butir 10). Proses ini tidak memegang kredensial relay/penulis audit (ADR-009 butir 3).
#[derive(Debug, Clone)]
pub struct WorkerConfig {
    /// `LOG_LEVEL` (dan `HTTP_BIND`, tidak dipakai proses ini).
    pub base: BaseConfig,
    /// `WORKER_DATABASE_URL`: login `rs_worker` (anggota `rs_app`) ke `ruang_sehat` (rahasia).
    pub database_url: Sensitive<String>,
    /// Pengambil event Keycloak (`KEYCLOAK_ADMIN_URL`, `KEYCLOAK_EVENT_*`).
    pub keycloak_events: KeycloakEventConfig,
    /// `WORKER_VALKEY_URL`: Valkey aplikasi yang sama dengan `API_VALKEY_URL`, untuk menyalin pencabutan sesi
    /// dari event LOGOUT Keycloak (TD-27; rahasia). Valkey mati tidak menghentikan pengambil: pencabutan tetap
    /// di PostgreSQL dan API memakai jalur cadangan (RT-18).
    pub valkey_url: Sensitive<String>,
    /// `WORKER_VALKEY_OP_TIMEOUT_MS` (bawaan `valkey::DEFAULT_OP_TIMEOUT_MS`).
    pub valkey_op_timeout_ms: u64,
    /// Worker IdP (ADR-030): realm `IDP_WORKER_REALMS`; rahasia klien `manage-users` dibaca dari OpenBao KV saat
    /// start, tidak dari lingkungan. `ApiConfig` tidak punya padanannya (KEPUTUSAN-305).
    pub idp: IdpWorkerConfig,
}

impl WorkerConfig {
    /// Memuat dari sumber; gagal cepat tanpa mencetak nilai.
    pub fn load(env: &EnvSource) -> Result<Self, ConfigError> {
        Ok(Self {
            base: BaseConfig::load(env)?,
            database_url: env.required_secret("WORKER_DATABASE_URL")?,
            keycloak_events: KeycloakEventConfig::load(env)?,
            valkey_url: env.required_secret("WORKER_VALKEY_URL")?,
            valkey_op_timeout_ms: env
                .optional_or("WORKER_VALKEY_OP_TIMEOUT_MS", valkey::DEFAULT_OP_TIMEOUT_MS)?,
            idp: IdpWorkerConfig::load(env)?,
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
            ("API_VALKEY_URL", "redis://127.0.0.1:16379"),
            (
                "AUTH_ADMIN_ISSUER",
                "https://auth.ruang-sehat.invalid/realms/admin",
            ),
            (
                "AUTH_ADMIN_JWKS_URL",
                "http://127.0.0.1:18081/realms/admin/protocol/openid-connect/certs",
            ),
            ("AUTH_ADMIN_AUDIENCE", "rs-backoffice-api"),
            (
                "AUTH_PATIENT_ISSUER",
                "https://auth.ruang-sehat.invalid/realms/pasien",
            ),
            (
                "AUTH_PATIENT_JWKS_URL",
                "http://127.0.0.1:18081/realms/pasien/protocol/openid-connect/certs",
            ),
            ("AUTH_PATIENT_AUDIENCE", "rs-patient-api"),
            (
                "AUTH_CORPORATE_ISSUER",
                "https://auth.ruang-sehat.invalid/realms/korporat",
            ),
            (
                "AUTH_CORPORATE_JWKS_URL",
                "http://127.0.0.1:18081/realms/korporat/protocol/openid-connect/certs",
            ),
            ("AUTH_CORPORATE_AUDIENCE", "rs-corporate-api"),
            ("OPENBAO_ADDR", "http://127.0.0.1:18200"),
            ("OPENBAO_TOKEN", "token-uji-sintetis-api"),
            ("NIK_BLIND_INDEX_KV_PATH", "ruang-sehat/blind-index/nik"),
            ("API_DEK_CACHE_TTL_SECONDS", "300"),
            ("API_DEK_CACHE_CAPACITY", "64"),
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

    // trace: ADR-030 langkah 3 (api tanpa rahasia manage-users)
    #[test]
    fn api_config_never_carries_idp_worker_secret() {
        let c = ApiConfig::load(&api_env(&[
            ("IDP_WORKER_REALMS", "korporat"),
            (
                "IDP_WORKER_KV_PATH_KORPORAT",
                "rs/keycloak/korporat/idp-worker",
            ),
            ("IDP_WORKER_CLIENT_SECRET", "rahasia-sintetis-manage-users"),
        ]))
        .unwrap();
        let dump = format!("{c:?}");
        assert!(!dump.contains("rahasia-sintetis-manage-users"));
        assert!(!dump.contains("idp-worker"));
        assert!(!dump.contains("IdpWorker"));
    }

    #[test]
    fn worker_idp_is_disabled_unless_configured_and_then_fails_closed() {
        let base = [
            (
                "WORKER_DATABASE_URL",
                "postgres://rs_worker@127.0.0.1/ruang_sehat",
            ),
            ("WORKER_VALKEY_URL", "redis://127.0.0.1:16379"),
            ("KEYCLOAK_ADMIN_URL", "http://127.0.0.1:18080"),
            ("KEYCLOAK_EVENT_SECRET_PASIEN", "s1"),
            ("KEYCLOAK_EVENT_SECRET_ADMIN", "s2"),
        ];
        let c = WorkerConfig::load(&EnvSource::from_pairs(base)).unwrap();
        assert!(!c.idp.enabled());
        let mut with_realm: Vec<(&str, &str)> = base.to_vec();
        with_realm.push(("IDP_WORKER_REALMS", "korporat"));
        let e = WorkerConfig::load(&EnvSource::from_pairs(with_realm.clone())).unwrap_err();
        assert_eq!(e.kind(), ConfigErrorKind::Missing);
        with_realm.push((
            "IDP_WORKER_KV_PATH_KORPORAT",
            "rs/keycloak/korporat/idp-worker",
        ));
        with_realm.push(("OPENBAO_ADDR", "http://127.0.0.1:18200"));
        with_realm.push(("OPENBAO_TOKEN", "token-uji-sintetis-worker"));
        let c = WorkerConfig::load(&EnvSource::from_pairs(with_realm)).unwrap();
        assert!(c.idp.enabled());
        assert!(!format!("{c:?}").contains("token-uji-sintetis-worker"));
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
        let c = ApiConfig::load(&api_env(&[])).unwrap();
        assert_eq!(c.problem_type_base, DEFAULT_PROBLEM_TYPE_BASE);
        assert_eq!(c.base.http_bind.port(), 8080);
        assert_eq!(c.database_max_connections.get(), 20);
        assert_eq!(c.admin_realm.audience, "rs-backoffice-api");
        assert_eq!(c.patient_realm.audience, "rs-patient-api");
        assert_eq!(c.corporate_realm.audience, "rs-corporate-api");
        assert_eq!(c.identity_keys.transit_mount, "transit");
        assert_eq!(c.identity_keys.transit_key, "rs-dek-kek");
        assert_eq!(c.identity_keys.blind_index_kv_mount, "secret");
        assert_eq!(c.identity_keys.dek_cache_ttl.seconds(), 300);
        assert_eq!(c.identity_keys.dek_cache_capacity.get(), 64);
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
        assert!(!dbg.contains("token-uji-sintetis-api"), "{dbg}");
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
            "API_VALKEY_URL",
            "AUTH_ADMIN_ISSUER",
            "AUTH_ADMIN_JWKS_URL",
            "AUTH_ADMIN_AUDIENCE",
            "AUTH_PATIENT_ISSUER",
            "AUTH_PATIENT_JWKS_URL",
            "AUTH_PATIENT_AUDIENCE",
            "AUTH_CORPORATE_ISSUER",
            "AUTH_CORPORATE_JWKS_URL",
            "AUTH_CORPORATE_AUDIENCE",
            "OPENBAO_ADDR",
            "OPENBAO_TOKEN",
            "NIK_BLIND_INDEX_KV_PATH",
            "API_DEK_CACHE_TTL_SECONDS",
            "API_DEK_CACHE_CAPACITY",
            "OBJECT_STORAGE_ENDPOINT",
        ] {
            let pairs: Vec<(&str, &str)> = [
                ("API_DATABASE_URL", "postgres://x"),
                ("API_VALKEY_URL", "redis://127.0.0.1:1"),
                ("AUTH_ADMIN_ISSUER", "https://i"),
                ("AUTH_ADMIN_JWKS_URL", "http://127.0.0.1:1/j"),
                ("AUTH_ADMIN_AUDIENCE", "a"),
                ("AUTH_PATIENT_ISSUER", "https://p"),
                ("AUTH_PATIENT_JWKS_URL", "http://127.0.0.1:1/p"),
                ("AUTH_PATIENT_AUDIENCE", "p"),
                ("AUTH_CORPORATE_ISSUER", "https://k"),
                ("AUTH_CORPORATE_JWKS_URL", "http://127.0.0.1:1/k"),
                ("AUTH_CORPORATE_AUDIENCE", "k"),
                ("OPENBAO_ADDR", "http://127.0.0.1:1"),
                ("OPENBAO_TOKEN", "t"),
                ("NIK_BLIND_INDEX_KV_PATH", "rs/nik"),
                ("API_DEK_CACHE_TTL_SECONDS", "300"),
                ("API_DEK_CACHE_CAPACITY", "64"),
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
        for (k, bad) in [
            ("API_DEK_CACHE_TTL_SECONDS", "0"),
            ("API_DEK_CACHE_TTL_SECONDS", "3601"),
            ("API_DEK_CACHE_CAPACITY", "0"),
            ("API_DEK_CACHE_CAPACITY", "100001"),
        ] {
            let e = ApiConfig::load(&api_env(&[(k, bad)])).unwrap_err();
            assert_eq!(e.name(), k, "{k}={bad}");
        }
    }

    #[test]
    fn worker_needs_database_and_keycloak_event_credentials() {
        let ok = EnvSource::from_pairs([
            (
                "WORKER_DATABASE_URL",
                "postgres://rs_worker@localhost/ruang_sehat",
            ),
            ("KEYCLOAK_ADMIN_URL", "http://127.0.0.1:18081"),
            ("KEYCLOAK_EVENT_SECRET_PASIEN", "rahasia-pasien"),
            ("KEYCLOAK_EVENT_SECRET_ADMIN", "rahasia-admin"),
            (
                "WORKER_VALKEY_URL",
                "redis://:rahasia-valkey@127.0.0.1:16379",
            ),
        ]);
        let c = WorkerConfig::load(&ok).unwrap();
        assert_eq!(c.keycloak_events.options().realms.len(), 2);
        assert_eq!(c.valkey_op_timeout_ms, valkey::DEFAULT_OP_TIMEOUT_MS);
        let dbg = format!("{c:?}");
        assert!(!dbg.contains("postgres://") && !dbg.contains("rahasia"));
        let no_valkey = EnvSource::from_pairs([
            ("WORKER_DATABASE_URL", "postgres://x"),
            ("KEYCLOAK_ADMIN_URL", "http://127.0.0.1:18081"),
            ("KEYCLOAK_EVENT_SECRET_PASIEN", "x"),
            ("KEYCLOAK_EVENT_SECRET_ADMIN", "x"),
        ]);
        assert_eq!(
            WorkerConfig::load(&no_valkey).unwrap_err().name(),
            "WORKER_VALKEY_URL"
        );
        let no_db = EnvSource::from_pairs([("KEYCLOAK_ADMIN_URL", "http://127.0.0.1:18081")]);
        assert_eq!(
            WorkerConfig::load(&no_db).unwrap_err().name(),
            "WORKER_DATABASE_URL"
        );
        let no_secret = EnvSource::from_pairs([
            ("WORKER_DATABASE_URL", "postgres://x"),
            ("KEYCLOAK_ADMIN_URL", "http://127.0.0.1:18081"),
            ("KEYCLOAK_EVENT_SECRET_PASIEN", "x"),
        ]);
        assert_eq!(
            WorkerConfig::load(&no_secret).unwrap_err().name(),
            "KEYCLOAK_EVENT_SECRET_ADMIN"
        );
    }
}
