//! Konfigurasi worker IdP (ADR-030 butir 1, langkah 3; KEPUTUSAN-132 struct bertipe, gagal cepat).
//!
//! Rahasia klien `manage-users` per realm TIDAK pernah lewat variabel lingkungan: proses `worker` membacanya dari
//! OpenBao KV v2 saat start dengan token identitas proses worker (policy `infra/openbao/policies/rs-worker.hcl`).
//! Token proses `api` ditolak policy untuk path itu (`rs-api.hcl`; diuji `identity_idp_openbao_policy.rs`), dan
//! `ApiConfig` tidak punya field rahasia ini sama sekali (KEPUTUSAN-305).
//!
//! `IDP_WORKER_REALMS` kosong (bawaan) = worker IdP tidak dijalankan; job IdP tetap `queued` sampai dikonfigurasi.
//! Realm yang diisi wajib lengkap (OpenBao, path KV); kekurangan menggagalkan start tanpa mencetak nilai.

use super::domain::IdpRealm;
use super::keycloak::{KeycloakAddress, RealmCredential};
use foundation::Sensitive;
use observability::{ConfigError, EnvSource};
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;
use std::time::Duration;

/// `client_id` bawaan klien worker IdP (`infra/keycloak/realms/korporat.json`).
pub const DEFAULT_IDP_WORKER_CLIENT_ID: &str = "rs-backend-idp-worker";

/// Nama field rahasia di entri KV.
pub const IDP_WORKER_KV_FIELD: &str = "client_secret";

/// Realm yang boleh disediakan worker pada langkah ini (ADR-030 fokus realm `korporat`, CR-026; realm `pasien`
/// dan `admin` tetap memakai alur F1-T50/T51/T65 sampai dipindahkan).
pub const IDP_WORKER_ALLOWED_REALMS: &[IdpRealm] = &[IdpRealm::Korporat];

/// Daftar realm worker IdP (boleh kosong).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct IdpRealms(Vec<IdpRealm>);

impl IdpRealms {
    /// Realm, urut sesuai konfigurasi.
    pub fn realms(&self) -> &[IdpRealm] {
        &self.0
    }
}

impl FromStr for IdpRealms {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut out = Vec::new();
        for part in s.split(',').map(str::trim).filter(|p| !p.is_empty()) {
            let r: IdpRealm = part.parse()?;
            if out.contains(&r) || !IDP_WORKER_ALLOWED_REALMS.contains(&r) {
                return Err(());
            }
            out.push(r);
        }
        Ok(Self(out))
    }
}

/// Ukuran batch 1..=100.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IdpBatchSize(u16);

impl FromStr for IdpBatchSize {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse::<u16>() {
            Ok(n) if (1..=platform_jobs::MAX_BATCH_SIZE).contains(&n) => Ok(Self(n)),
            _ => Err(()),
        }
    }
}

/// Bagian OpenBao + KV (hanya bila ada realm).
#[derive(Clone)]
pub struct IdpWorkerSecrets {
    /// `OPENBAO_ADDR`.
    pub openbao_addr: String,
    /// `OPENBAO_TOKEN`: token identitas proses worker (rahasia).
    pub openbao_token: Sensitive<String>,
    /// `IDP_WORKER_KV_MOUNT` (bawaan `secret`).
    pub kv_mount: String,
    /// `IDP_WORKER_KV_PATH_<REALM>` per realm.
    pub kv_paths: HashMap<IdpRealm, String>,
}

impl fmt::Debug for IdpWorkerSecrets {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IdpWorkerSecrets")
            .field("openbao_addr", &self.openbao_addr)
            .field("kv_mount", &self.kv_mount)
            .field("kv_paths", &self.kv_paths)
            .finish_non_exhaustive()
    }
}

/// Konfigurasi worker IdP.
#[derive(Debug, Clone)]
pub struct IdpWorkerConfig {
    /// `IDP_WORKER_REALMS` (bawaan kosong = nonaktif).
    pub realms: IdpRealms,
    /// `KEYCLOAK_ADMIN_URL` (sama dengan pengambil event).
    pub keycloak_url: Option<KeycloakAddress>,
    /// `IDP_WORKER_CLIENT_ID` (bawaan [`DEFAULT_IDP_WORKER_CLIENT_ID`]).
    pub client_id: String,
    /// OpenBao dan path KV (`None` bila nonaktif).
    pub secrets: Option<IdpWorkerSecrets>,
    /// `IDP_WORKER_TIMEOUT_SECS` (bawaan 10).
    pub timeout_secs: u64,
    /// `IDP_WORKER_POLL_MS` (bawaan 2000).
    pub poll_ms: u64,
    /// `IDP_WORKER_BATCH_SIZE` (bawaan 10).
    pub batch_size: IdpBatchSize,
    /// `IDP_WORKER_LEASE_SECS` (bawaan 120; > batas waktu Keycloak x jumlah panggilan per job).
    pub lease_secs: u32,
}

impl IdpWorkerConfig {
    /// Memuat dari sumber; realm yang diisi wajib lengkap.
    pub fn load(env: &EnvSource) -> Result<Self, ConfigError> {
        let realms: IdpRealms = env.optional_or("IDP_WORKER_REALMS", IdpRealms::default())?;
        let enabled = !realms.realms().is_empty();
        let secrets = if enabled {
            let mut kv_paths = HashMap::new();
            for realm in realms.realms() {
                let name = format!("IDP_WORKER_KV_PATH_{}", realm.as_str().to_ascii_uppercase());
                kv_paths.insert(*realm, env.required::<String>(&name)?);
            }
            Some(IdpWorkerSecrets {
                openbao_addr: env.required("OPENBAO_ADDR")?,
                openbao_token: env.required_secret("OPENBAO_TOKEN")?,
                kv_mount: env.optional_or("IDP_WORKER_KV_MOUNT", "secret".to_string())?,
                kv_paths,
            })
        } else {
            None
        };
        let lease_secs: u32 = env.optional_or("IDP_WORKER_LEASE_SECS", 120)?;
        if !(30..=3_600).contains(&lease_secs) {
            return Err(ConfigError::invalid("IDP_WORKER_LEASE_SECS"));
        }
        let timeout_secs: u64 = env.optional_or("IDP_WORKER_TIMEOUT_SECS", 10)?;
        if !(1..=30).contains(&timeout_secs) {
            return Err(ConfigError::invalid("IDP_WORKER_TIMEOUT_SECS"));
        }
        let poll_ms: u64 = env.optional_or("IDP_WORKER_POLL_MS", 2_000)?;
        if !(100..=60_000).contains(&poll_ms) {
            return Err(ConfigError::invalid("IDP_WORKER_POLL_MS"));
        }
        Ok(Self {
            keycloak_url: if enabled {
                Some(env.required("KEYCLOAK_ADMIN_URL")?)
            } else {
                None
            },
            realms,
            client_id: env.optional_or(
                "IDP_WORKER_CLIENT_ID",
                DEFAULT_IDP_WORKER_CLIENT_ID.to_string(),
            )?,
            secrets,
            timeout_secs,
            poll_ms,
            batch_size: env.optional_or("IDP_WORKER_BATCH_SIZE", IdpBatchSize(10))?,
            lease_secs,
        })
    }

    /// Worker IdP dijalankan.
    pub fn enabled(&self) -> bool {
        self.secrets.is_some()
    }

    /// Batas waktu Keycloak.
    pub fn timeout(&self) -> Duration {
        Duration::from_secs(self.timeout_secs)
    }

    /// Jeda antar-putaran.
    pub fn poll_interval(&self) -> Duration {
        Duration::from_millis(self.poll_ms)
    }

    /// Ukuran batch.
    pub fn batch_size(&self) -> u16 {
        self.batch_size.0
    }
}

/// Kegagalan memuat rahasia worker (tanpa nilai, path, atau isi respons).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdpSecretError {
    /// Worker IdP tidak dikonfigurasi.
    Disabled,
    /// OpenBao menolak (policy) atau tidak terjangkau.
    OpenBao,
    /// Entri KV tidak memuat field rahasia.
    MissingField,
}

impl fmt::Display for IdpSecretError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Disabled => f.write_str("worker IdP tidak dikonfigurasi"),
            Self::OpenBao => {
                f.write_str("rahasia klien worker IdP tidak dapat dibaca dari OpenBao")
            }
            Self::MissingField => f.write_str("entri KV rahasia klien worker IdP tidak lengkap"),
        }
    }
}

/// Membaca rahasia klien per realm dari OpenBao KV (gagal-tertutup: satu realm gagal = tidak ada kredensial).
pub async fn load_idp_credentials(
    cfg: &IdpWorkerConfig,
) -> Result<HashMap<IdpRealm, RealmCredential>, IdpSecretError> {
    let secrets = cfg.secrets.as_ref().ok_or(IdpSecretError::Disabled)?;
    let openbao = crypto::OpenBaoConfig {
        address: secrets.openbao_addr.clone(),
        token: secrets.openbao_token.clone(),
        timeout: cfg.timeout(),
    };
    let mut out = HashMap::new();
    for realm in cfg.realms.realms() {
        let path = secrets
            .kv_paths
            .get(realm)
            .ok_or(IdpSecretError::MissingField)?;
        let mut kv = crypto::read_kv_secret(&openbao, &secrets.kv_mount, path)
            .await
            .map_err(|_| IdpSecretError::OpenBao)?;
        let secret = kv
            .take(IDP_WORKER_KV_FIELD)
            .ok_or(IdpSecretError::MissingField)?;
        out.insert(
            *realm,
            RealmCredential {
                client_id: cfg.client_id.clone(),
                secret,
            },
        );
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_by_default_without_openbao() {
        let c = IdpWorkerConfig::load(&EnvSource::from_pairs([])).unwrap();
        assert!(!c.enabled());
        assert!(c.realms.realms().is_empty());
        assert_eq!(c.client_id, DEFAULT_IDP_WORKER_CLIENT_ID);
    }

    // trace: ADR-030 langkah 3 (gagal-tertutup)
    #[test]
    fn configured_realm_requires_openbao_and_kv_path() {
        let full = [
            ("IDP_WORKER_REALMS", "korporat"),
            ("KEYCLOAK_ADMIN_URL", "http://127.0.0.1:18080"),
            ("OPENBAO_ADDR", "http://127.0.0.1:18200"),
            ("OPENBAO_TOKEN", "token-uji-sintetis-worker"),
            (
                "IDP_WORKER_KV_PATH_KORPORAT",
                "rs/keycloak/korporat/idp-worker",
            ),
        ];
        let c = IdpWorkerConfig::load(&EnvSource::from_pairs(full)).unwrap();
        assert!(c.enabled());
        assert!(!format!("{c:?}").contains("token-uji-sintetis-worker"));
        for missing in [
            "OPENBAO_ADDR",
            "OPENBAO_TOKEN",
            "IDP_WORKER_KV_PATH_KORPORAT",
            "KEYCLOAK_ADMIN_URL",
        ] {
            let pairs: Vec<(&str, &str)> = full
                .iter()
                .copied()
                .filter(|(k, _)| *k != missing)
                .collect();
            assert!(
                IdpWorkerConfig::load(&EnvSource::from_pairs(pairs)).is_err(),
                "{missing}"
            );
        }
    }

    #[test]
    fn only_allowed_realms_are_accepted() {
        for bad in ["pasien", "admin", "korporat,korporat", "rme", "Korporat"] {
            assert!(bad.parse::<IdpRealms>().is_err(), "{bad}");
        }
        assert!("".parse::<IdpRealms>().unwrap().realms().is_empty());
    }

    #[tokio::test]
    async fn credentials_are_not_loaded_when_disabled() {
        let c = IdpWorkerConfig::load(&EnvSource::from_pairs([])).unwrap();
        assert_eq!(
            load_idp_credentials(&c).await.err(),
            Some(IdpSecretError::Disabled)
        );
    }
}
