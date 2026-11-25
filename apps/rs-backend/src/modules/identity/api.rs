//! Antarmuka publik modul `identity`.
//!
//! F1-T34: pengambil event Keycloak E-01–E-04 ke `audit_outbox` (ADR-011 butir 10) untuk subcommand
//! `rs-backend worker`: konfigurasi bertipe [`KeycloakEventConfig`], klien Admin REST [`KeycloakClient`],
//! satu putaran [`Puller::round`] (dipakai test), dan perulangan proses [`run_puller`].
//!
//! F1-T65 (US-208): [`AdminAccounts`] untuk bootstrap Super Admin pertama (perintah
//! `rs-backend bootstrap-super-admin`, persetujuan orang kedua lewat Control Group OpenBao), pembuatan akun
//! admin beserta undangan aktivasi sekali pakai 24 jam, perubahan sub-peran, dan penonaktifan, semuanya
//! dengan E-10/E-11 lewat `audit::in_audited_tx`.
//!
//! F1-T107 (US-1904 AC2, AC5): [`CorporateHrAccounts`] daftar dan penonaktifan anggota HR perusahaan
//! (invarian HR aktif terakhir, E-10/E-11), dengan antarmuka penyedia realm `korporat`
//! [`CorporateIdentityProvisioner`] yang belum punya implementasi produksi ([`ProvisionerNotConfigured`]).
//!
//! F1-T33: resolver akun dan penjaga sesi yang dicabut untuk perakitan aplikasi dan modul lain:
//! - [`AccountDirectory`]: dipasang ke `authz::Enforcement::new(auth, directory.clone(), directory, denials)`
//!   sebagai resolver akun dan penjaga sesi yang dicabut.
//! - Pencabutan sesi: [`AccountDirectory::record_revocation`] di transaksi use case pemanggil (misalnya
//!   keluar F1-T54, penonaktifan US-207), lalu [`AccountDirectory::publish_revocation`] setelah commit.
//! - Penulis status akun memanggil [`AccountDirectory::refresh_account`] setelah commit.
//! - Aksi berisiko memakai [`AccountDirectory::resolve_fresh`] (tanpa cache, M-13).
//! - Job berkala: [`AccountDirectory::purge_expired_revocations`]; saat start atau setelah isi Valkey
//!   hilang: [`AccountDirectory::rebuild_revocation_cache`] (juga dipicu otomatis di latar belakang).

use observability::{ConfigError, EnvSource};
use std::collections::HashMap;
use std::num::NonZeroU64;
use std::str::FromStr;
use std::time::Duration;

pub use super::admin_domain::{
    ACTIVATION_INVITE_LIFESPAN_SECONDS, AdminRuleViolation, RoleDiff, is_sensitive,
};
pub use super::admin_service::{
    AccountChangeListener, AdminAccounts, AdminActor, AdminIdentityProvider, BootstrapApproval,
    CreatedAdmin, Deactivated, IdpFailure, IdpFuture, InviteDelivery, KeycloakAdminUsers,
    NewAdminInput, NoAccountCache, StaffNikIndex, staff_nik_index,
};
pub use super::corporate_hr_domain::HrRuleViolation;
pub use super::corporate_hr_service::{
    CorporateHrAccounts, CorporateIdentityProvisioner, HrDeactivated, HrIdpFailure, HrIdpFuture,
    HrMemberView, ProvisionerNotConfigured,
};
pub use super::domain::{IdpRealm, Mapped, PULLED_TYPES, map_event, reason_code};
pub use super::error::IdentityError;
pub use super::idp_config::{
    DEFAULT_IDP_WORKER_CLIENT_ID, IDP_WORKER_ALLOWED_REALMS, IDP_WORKER_KV_FIELD, IdpRealms,
    IdpSecretError, IdpWorkerConfig, load_idp_credentials,
};
pub use super::idp_domain::{
    HR_ACTIVATION_ACTIONS, HR_ACTIVATION_LIFESPAN_SECONDS, INTENT_ATTRIBUTE, IdpJobKind,
    IntentPurpose, IntentStatus, PAYLOAD_INTENT_KEY, Refusal,
};
pub use super::idp_repo::RepoError as IntentRepoError;
pub use super::idp_service::{
    IDP_JOB_DEDUPE_PURPOSE, IDP_JOB_MAX_ATTEMPTS, IntentActor, IntentError, IntentRequest,
    PendingAccount, PendingAccountRequest, record_intent, register_pending_account,
};
pub use super::idp_worker::{
    IdpWorker, InvalidIdpWorkerConfig, PROVISIONING_FAILED_REASON, idp_job_types, run_idp_worker,
};
pub use super::idp_worker_client::{CreateOutcome, IdpUserAdmin};
pub use super::jobs::{
    LOGOUT_REVOCATION_TTL, MAX_PAGES_PER_ROUND, PAGE_SIZE_RANGE, PullError, PullOptions, Puller,
    PullerSettings, RoundReport, SAME_MS_LIMIT, pull_realm, run_puller,
};
pub use super::keycloak::{
    ACTIVATION_ACTIONS, InvalidKeycloakAddress, KeycloakAddress, KeycloakClient, KeycloakError,
    RealmCredential,
};
pub use super::service::{
    AccountDirectory, DEFAULT_ACCOUNT_CACHE_TTL_SECONDS, RecordedRevocation, RevocationReason,
};

/// `client_id` bawaan klien layanan pembaca event di setiap realm (`infra/keycloak/realms/*.json`).
pub const DEFAULT_EVENT_CLIENT_ID: &str = "rs-backend-event-reader";

/// Daftar realm yang diambil: dipisah koma, tanpa duplikat, minimal satu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RealmList(Vec<IdpRealm>);

impl RealmList {
    /// Realm, urut sesuai konfigurasi.
    pub fn realms(&self) -> &[IdpRealm] {
        &self.0
    }
}

impl FromStr for RealmList {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut out = Vec::new();
        for part in s.split(',') {
            let r: IdpRealm = part.trim().parse()?;
            if out.contains(&r) {
                return Err(());
            }
            out.push(r);
        }
        if out.is_empty() {
            Err(())
        } else {
            Ok(Self(out))
        }
    }
}

/// Ukuran halaman Admin REST, [`PAGE_SIZE_RANGE`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageSize(usize);

impl FromStr for PageSize {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse::<usize>() {
            Ok(n) if PAGE_SIZE_RANGE.contains(&n) => Ok(Self(n)),
            _ => Err(()),
        }
    }
}

/// Batas waktu satu permintaan ke Keycloak, 1–30 detik.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeoutSecs(u64);

impl FromStr for TimeoutSecs {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse::<u64>() {
            Ok(n) if (1..=30).contains(&n) => Ok(Self(n)),
            _ => Err(()),
        }
    }
}

/// Konfigurasi pengambil event Keycloak (KEPUTUSAN-132: struct bertipe, gagal cepat, tanpa nilai di galat).
#[derive(Debug, Clone)]
pub struct KeycloakEventConfig {
    /// `KEYCLOAK_ADMIN_URL`: `http://<ip-loopback>:<port>` (TLS lintas host belum diterapkan).
    pub keycloak_url: KeycloakAddress,
    /// `KEYCLOAK_EVENT_REALMS` (bawaan `pasien,admin`, realm yang sudah ada sebagai kode).
    pub realms: RealmList,
    /// `KEYCLOAK_EVENT_CLIENT_ID` (bawaan [`DEFAULT_EVENT_CLIENT_ID`]).
    pub client_id: String,
    /// `KEYCLOAK_EVENT_SECRET_<REALM>` per realm (rahasia, dari OpenBao lewat skrip deploy).
    pub credentials: HashMap<IdpRealm, RealmCredential>,
    /// `KEYCLOAK_EVENT_POLL_SECS` (bawaan 60, ADR-011 butir 10 "setiap menit").
    pub poll_secs: NonZeroU64,
    /// `KEYCLOAK_EVENT_OVERLAP_SECS` (bawaan 120): jendela pembacaan ulang.
    pub overlap_secs: NonZeroU64,
    /// `KEYCLOAK_EVENT_PAGE_SIZE` (bawaan 100).
    pub page_size: PageSize,
    /// `KEYCLOAK_EVENT_LAG_ALERT_SECS` (bawaan 300, ADR-011 butir 10 "lebih dari 5 menit").
    pub lag_alert_secs: NonZeroU64,
    /// `KEYCLOAK_EVENT_TIMEOUT_SECS` (bawaan 10).
    pub timeout_secs: TimeoutSecs,
    /// `KEYCLOAK_EVENT_RETRY_SECS` (bawaan 15): jeda coba ulang sesi dan saat siaga.
    pub retry_secs: NonZeroU64,
}

fn nz(v: u64) -> NonZeroU64 {
    NonZeroU64::new(v).unwrap_or(NonZeroU64::MIN)
}

impl KeycloakEventConfig {
    /// Memuat dari sumber; rahasia wajib untuk setiap realm yang dikonfigurasi.
    pub fn load(env: &EnvSource) -> Result<Self, ConfigError> {
        let realms: RealmList = env.optional_or(
            "KEYCLOAK_EVENT_REALMS",
            RealmList(vec![IdpRealm::Pasien, IdpRealm::Admin]),
        )?;
        let client_id: String = env.optional_or(
            "KEYCLOAK_EVENT_CLIENT_ID",
            DEFAULT_EVENT_CLIENT_ID.to_string(),
        )?;
        let mut credentials = HashMap::new();
        for realm in realms.realms() {
            let name = format!(
                "KEYCLOAK_EVENT_SECRET_{}",
                realm.as_str().to_ascii_uppercase()
            );
            credentials.insert(
                *realm,
                RealmCredential {
                    client_id: client_id.clone(),
                    secret: env.required_secret(&name)?,
                },
            );
        }
        Ok(Self {
            keycloak_url: env.required("KEYCLOAK_ADMIN_URL")?,
            realms,
            client_id,
            credentials,
            poll_secs: env.optional_or("KEYCLOAK_EVENT_POLL_SECS", nz(60))?,
            overlap_secs: env.optional_or("KEYCLOAK_EVENT_OVERLAP_SECS", nz(120))?,
            page_size: env.optional_or("KEYCLOAK_EVENT_PAGE_SIZE", PageSize(100))?,
            lag_alert_secs: env.optional_or("KEYCLOAK_EVENT_LAG_ALERT_SECS", nz(300))?,
            timeout_secs: env.optional_or("KEYCLOAK_EVENT_TIMEOUT_SECS", TimeoutSecs(10))?,
            retry_secs: env.optional_or("KEYCLOAK_EVENT_RETRY_SECS", nz(15))?,
        })
    }

    /// Pengaturan putaran.
    pub fn options(&self) -> PullOptions {
        PullOptions {
            realms: self.realms.realms().to_vec(),
            page_size: self.page_size.0,
            overlap: time::Duration::seconds(i64::try_from(self.overlap_secs.get()).unwrap_or(120)),
            lag_alert: time::Duration::seconds(
                i64::try_from(self.lag_alert_secs.get()).unwrap_or(300),
            ),
        }
    }

    /// Klien Admin REST.
    pub fn client(&self) -> Result<KeycloakClient, InvalidKeycloakAddress> {
        KeycloakClient::new(
            self.keycloak_url,
            Duration::from_secs(self.timeout_secs.0),
            self.credentials.clone(),
        )
    }

    /// Jeda antar-putaran.
    pub fn poll_interval(&self) -> Duration {
        Duration::from_secs(self.poll_secs.get())
    }

    /// Jeda coba ulang.
    pub fn retry_interval(&self) -> Duration {
        Duration::from_secs(self.retry_secs.get())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use observability::ConfigErrorKind;

    fn base<'a>() -> Vec<(&'a str, &'a str)> {
        vec![
            ("KEYCLOAK_ADMIN_URL", "http://127.0.0.1:18081"),
            ("KEYCLOAK_EVENT_SECRET_PASIEN", "rahasia-pasien"),
            ("KEYCLOAK_EVENT_SECRET_ADMIN", "rahasia-admin"),
        ]
    }

    #[test]
    fn defaults_follow_adr_011() {
        let c = KeycloakEventConfig::load(&EnvSource::from_pairs(base())).unwrap();
        assert_eq!(c.realms.realms(), &[IdpRealm::Pasien, IdpRealm::Admin]);
        assert_eq!(c.client_id, DEFAULT_EVENT_CLIENT_ID);
        assert_eq!(c.poll_interval(), Duration::from_secs(60));
        let o = c.options();
        assert_eq!(o.lag_alert, time::Duration::minutes(5));
        assert_eq!(o.overlap, time::Duration::seconds(120));
        assert_eq!(o.page_size, 100);
        assert!(!format!("{c:?}").contains("rahasia"), "Debug tanpa rahasia");
    }

    #[test]
    fn each_configured_realm_needs_its_own_secret() {
        let mut v = base();
        v.push(("KEYCLOAK_EVENT_REALMS", "pasien,admin,nakes"));
        let e = KeycloakEventConfig::load(&EnvSource::from_pairs(v)).unwrap_err();
        assert_eq!(
            (e.kind(), e.name()),
            (ConfigErrorKind::Missing, "KEYCLOAK_EVENT_SECRET_NAKES")
        );
        let only = EnvSource::from_pairs([
            ("KEYCLOAK_ADMIN_URL", "http://127.0.0.1:18081"),
            ("KEYCLOAK_EVENT_REALMS", "admin"),
            ("KEYCLOAK_EVENT_SECRET_ADMIN", "x"),
        ]);
        assert_eq!(
            KeycloakEventConfig::load(&only).unwrap().credentials.len(),
            1
        );
    }

    #[test]
    fn invalid_values_fail_fast() {
        for (k, v) in [
            ("KEYCLOAK_EVENT_REALMS", "pasien,pasien"),
            ("KEYCLOAK_EVENT_REALMS", "rme"),
            ("KEYCLOAK_EVENT_REALMS", ""),
            ("KEYCLOAK_EVENT_PAGE_SIZE", "0"),
            ("KEYCLOAK_EVENT_PAGE_SIZE", "501"),
            ("KEYCLOAK_EVENT_TIMEOUT_SECS", "31"),
            ("KEYCLOAK_EVENT_POLL_SECS", "0"),
            ("KEYCLOAK_EVENT_LAG_ALERT_SECS", "-1"),
            ("KEYCLOAK_ADMIN_URL", "https://keycloak.example:8443"),
            ("KEYCLOAK_ADMIN_URL", "http://10.0.0.5:8080"),
        ] {
            let mut env = base();
            env.retain(|(n, _)| *n != k);
            env.push((k, v));
            let e = KeycloakEventConfig::load(&EnvSource::from_pairs(env)).unwrap_err();
            assert!(
                matches!(e.kind(), ConfigErrorKind::Invalid | ConfigErrorKind::Empty),
                "{k}={v}: {e}"
            );
            assert_eq!(e.name(), k);
        }
    }
}
