//! Perintah `rs-backend bootstrap-super-admin` (F1-T65, US-208 AC4, klarifikasi T-19 a, ADR-021 butir 3,
//! ADR-025): bootstrap Super Admin pertama lewat jalur administrasi production (bastion), hanya setelah
//! persetujuan orang kedua lewat Control Group OpenBao.
//!
//! Prosedur (runbook operasi):
//! 1. Operator login ke OpenBao dengan identitasnya sendiri (MFA), lalu membaca path ber-Control Group
//!    (`BOOTSTRAP_CG_KV_MOUNT`/`BOOTSTRAP_CG_KV_PATH`, bawaan `rs-admin` / `bootstrap/super-admin`, berisi
//!    `purpose=rs_super_admin_bootstrap`). OpenBao menjawab token pembungkus dan `accessor`.
//! 2. Orang kedua (anggota grup approver) menyetujui lewat `sys/control-group/authorize` dengan `accessor`
//!    itu (OpenBao tidak memberi tahu approver; pemberitahuan lewat prosedur di luar OpenBao, KEPUTUSAN-149).
//! 3. Operator menjalankan perintah ini dengan `BOOTSTRAP_OPERATOR_TOKEN`, `BOOTSTRAP_CG_ACCESSOR`,
//!    `BOOTSTRAP_CG_WRAPPING_TOKEN`, dan email + nama calon Super Admin yang sudah disetujui
//!    (`BOOTSTRAP_MODE=create`), atau `BOOTSTRAP_MODE=reissue_invite` untuk undangan ulang setelah undangan
//!    kedaluwarsa. Persetujuan diverifikasi dan dipakai sekali ([`crypto::verify_control_group_approval`])
//!    sebelum apa pun diubah.
//!
//! Kode keluar: 0 berhasil (undangan terkirim), 3 akun dibuat tetapi undangan gagal (jalankan ulang dengan
//! `reissue_invite` dan persetujuan baru), 4 ditolak aturan (sudah ada Super Admin aktif/menunggu
//! aktivasi, atau tidak ada yang dapat diundang ulang), 5 persetujuan tidak sah, 1 galat lain, 78 konfigurasi
//! tidak valid. Keluaran tidak memuat email, nama, token, atau accessor.

use crate::modules::identity::api::{
    AdminAccounts, AdminRuleViolation, BootstrapApproval, IdentityError, IdpRealm, InviteDelivery,
    KeycloakAddress, KeycloakClient, NoAccountCache, RealmCredential,
};
use audit::{AuditContext, Source, SourceComponent};
use foundation::{Clock, CorrelationId, Sensitive, SystemClock};
use observability::{BaseConfig, ConfigError, EnvSource};
use sqlx::postgres::PgPoolOptions;
use std::collections::HashMap;
use std::process::ExitCode;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

/// `client_id` bawaan klien layanan pengelola pengguna realm `admin` (`infra/keycloak/realms/admin.json`).
pub const DEFAULT_ADMIN_MANAGER_CLIENT_ID: &str = "rs-backend-admin-manager";
/// Nilai `purpose` rahasia ber-Control Group.
pub const BOOTSTRAP_PURPOSE: &str = "rs_super_admin_bootstrap";

/// Mode perintah.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootstrapMode {
    /// Membuat Super Admin pertama.
    Create,
    /// Mengirim ulang undangan ke Super Admin yang menunggu aktivasi.
    ReissueInvite,
}

impl FromStr for BootstrapMode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "create" => Ok(Self::Create),
            "reissue_invite" => Ok(Self::ReissueInvite),
            _ => Err(()),
        }
    }
}

/// Konfigurasi perintah bootstrap (KEPUTUSAN-132: bertipe, gagal cepat, tanpa nilai di galat).
#[derive(Debug, Clone)]
pub struct BootstrapConfig {
    /// `LOG_LEVEL`.
    pub base: BaseConfig,
    /// `BOOTSTRAP_MODE`: `create` | `reissue_invite`.
    pub mode: BootstrapMode,
    /// `BOOTSTRAP_EMAIL` (wajib untuk `create`).
    pub email: Option<String>,
    /// `BOOTSTRAP_FULL_NAME` (wajib untuk `create`).
    pub full_name: Option<String>,
    /// `BOOTSTRAP_DATABASE_URL`: login anggota `rs_app` ke `ruang_sehat` (rahasia, kredensial dinamis).
    pub database_url: Sensitive<String>,
    /// `OPENBAO_ADDR`.
    pub openbao_addr: String,
    /// `BOOTSTRAP_OPERATOR_TOKEN`: token login OpenBao operator (rahasia).
    pub operator_token: Sensitive<String>,
    /// `BOOTSTRAP_CG_ACCESSOR`.
    pub accessor: Sensitive<String>,
    /// `BOOTSTRAP_CG_WRAPPING_TOKEN` (rahasia, sekali pakai).
    pub wrapping_token: Sensitive<String>,
    /// `BOOTSTRAP_CG_KV_MOUNT` (bawaan `rs-admin`).
    pub kv_mount: String,
    /// `BOOTSTRAP_CG_KV_PATH` (bawaan `bootstrap/super-admin`).
    pub kv_path: String,
    /// `KEYCLOAK_ADMIN_URL`.
    pub keycloak_url: KeycloakAddress,
    /// `KEYCLOAK_ADMIN_MANAGER_CLIENT_ID` (bawaan [`DEFAULT_ADMIN_MANAGER_CLIENT_ID`]).
    pub keycloak_client_id: String,
    /// `KEYCLOAK_ADMIN_MANAGER_SECRET` (rahasia).
    pub keycloak_secret: Sensitive<String>,
}

impl BootstrapConfig {
    /// Memuat dari sumber.
    pub fn load(env: &EnvSource) -> Result<Self, ConfigError> {
        let mode: BootstrapMode = env.required("BOOTSTRAP_MODE")?;
        let (email, full_name) = match mode {
            BootstrapMode::Create => (
                Some(env.required::<String>("BOOTSTRAP_EMAIL")?),
                Some(env.required::<String>("BOOTSTRAP_FULL_NAME")?),
            ),
            BootstrapMode::ReissueInvite => (None, None),
        };
        Ok(Self {
            base: BaseConfig::load(env)?,
            mode,
            email,
            full_name,
            database_url: env.required_secret("BOOTSTRAP_DATABASE_URL")?,
            openbao_addr: env.required("OPENBAO_ADDR")?,
            operator_token: env.required_secret("BOOTSTRAP_OPERATOR_TOKEN")?,
            accessor: env.required_secret("BOOTSTRAP_CG_ACCESSOR")?,
            wrapping_token: env.required_secret("BOOTSTRAP_CG_WRAPPING_TOKEN")?,
            kv_mount: env.optional_or("BOOTSTRAP_CG_KV_MOUNT", "rs-admin".to_string())?,
            kv_path: env
                .optional_or("BOOTSTRAP_CG_KV_PATH", "bootstrap/super-admin".to_string())?,
            keycloak_url: env.required("KEYCLOAK_ADMIN_URL")?,
            keycloak_client_id: env.optional_or(
                "KEYCLOAK_ADMIN_MANAGER_CLIENT_ID",
                DEFAULT_ADMIN_MANAGER_CLIENT_ID.to_string(),
            )?,
            keycloak_secret: env.required_secret("KEYCLOAK_ADMIN_MANAGER_SECRET")?,
        })
    }
}

const TIMEOUT: Duration = Duration::from_secs(10);

fn exit_for(e: &IdentityError) -> ExitCode {
    match e {
        IdentityError::AdminRule(
            AdminRuleViolation::SuperAdminExists | AdminRuleViolation::NoPendingBootstrapAccount,
        ) => ExitCode::from(4),
        _ => ExitCode::FAILURE,
    }
}

/// Menjalankan perintah bootstrap (dipanggil `cli` setelah log terpasang).
pub fn run(config: BootstrapConfig) -> ExitCode {
    let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(r) => r,
        Err(_) => return ExitCode::FAILURE,
    };
    runtime.block_on(execute(config))
}

async fn execute(config: BootstrapConfig) -> ExitCode {
    let approval = match crypto::verify_control_group_approval(&crypto::ControlGroupRequest {
        address: config.openbao_addr.clone(),
        operator_token: config.operator_token.clone(),
        accessor: config.accessor.expose().clone(),
        wrapping_token: config.wrapping_token.clone(),
        kv_mount: config.kv_mount.clone(),
        kv_path: config.kv_path.clone(),
        purpose: BOOTSTRAP_PURPOSE.to_string(),
        timeout: TIMEOUT,
    })
    .await
    {
        Ok(a) => BootstrapApproval {
            operator_entity_id: a.operator_entity_id,
            approver_entity_ids: a.approver_entity_ids,
        },
        Err(e) => {
            tracing::error!(error = %e, "bootstrap_approval_rejected");
            return ExitCode::from(5);
        }
    };
    let pool = match PgPoolOptions::new()
        .max_connections(2)
        .acquire_timeout(TIMEOUT)
        .connect(config.database_url.expose())
        .await
    {
        Ok(p) => p,
        Err(_) => {
            tracing::error!("bootstrap_database_unavailable");
            return ExitCode::FAILURE;
        }
    };
    let clock: Arc<dyn Clock> = Arc::new(SystemClock);
    let client = match KeycloakClient::new(
        config.keycloak_url,
        TIMEOUT,
        HashMap::from([(
            IdpRealm::Admin,
            RealmCredential {
                client_id: config.keycloak_client_id.clone(),
                secret: config.keycloak_secret.clone(),
            },
        )]),
    ) {
        Ok(c) => Arc::new(c),
        Err(_) => return ExitCode::FAILURE,
    };
    let accounts = AdminAccounts::new(
        pool,
        Arc::new(crate::modules::identity::api::KeycloakAdminUsers::new(
            client,
            Arc::clone(&clock),
        )),
        Arc::new(NoAccountCache),
        clock,
    );
    let ctx = AuditContext::new(
        CorrelationId::new(),
        Source::component(SourceComponent::Worker),
    );
    let result = match config.mode {
        BootstrapMode::Create => accounts
            .bootstrap(
                approval,
                config.email.as_deref().unwrap_or_default(),
                config.full_name.as_deref().unwrap_or_default(),
                &ctx,
            )
            .await
            .map(|c| (c.account_id, c.invite)),
        BootstrapMode::ReissueInvite => accounts.bootstrap_reissue(approval, &ctx).await,
    };
    match result {
        Ok((account_id, InviteDelivery::Sent)) => {
            tracing::info!(account_id = %account_id, "bootstrap_super_admin_invited");
            ExitCode::SUCCESS
        }
        Ok((account_id, InviteDelivery::Failed)) => {
            tracing::error!(account_id = %account_id, "bootstrap_super_admin_invite_failed");
            ExitCode::from(3)
        }
        Err(e) => {
            tracing::error!(error_kind = e.code(), "bootstrap_super_admin_rejected");
            exit_for(&e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use observability::ConfigErrorKind;

    fn base(mode: &'static str) -> Vec<(&'static str, &'static str)> {
        vec![
            ("BOOTSTRAP_MODE", mode),
            (
                "BOOTSTRAP_DATABASE_URL",
                "postgres://rs@127.0.0.1/ruang_sehat",
            ),
            ("OPENBAO_ADDR", "http://127.0.0.1:8200"),
            ("BOOTSTRAP_OPERATOR_TOKEN", "token-operator-rahasia"),
            ("BOOTSTRAP_CG_ACCESSOR", "accessor-uji"),
            ("BOOTSTRAP_CG_WRAPPING_TOKEN", "token-pembungkus-rahasia"),
            ("KEYCLOAK_ADMIN_URL", "http://127.0.0.1:18081"),
            ("KEYCLOAK_ADMIN_MANAGER_SECRET", "rahasia-klien"),
        ]
    }

    #[test]
    fn create_needs_email_and_name_reissue_does_not() {
        let e = BootstrapConfig::load(&EnvSource::from_pairs(base("create"))).unwrap_err();
        assert_eq!(
            (e.kind(), e.name()),
            (ConfigErrorKind::Missing, "BOOTSTRAP_EMAIL")
        );
        let mut v = base("create");
        v.push(("BOOTSTRAP_EMAIL", "calon@example.invalid"));
        v.push(("BOOTSTRAP_FULL_NAME", "Calon Super Admin"));
        let c = BootstrapConfig::load(&EnvSource::from_pairs(v)).unwrap();
        assert_eq!(c.mode, BootstrapMode::Create);
        assert_eq!(c.kv_mount, "rs-admin");
        assert_eq!(c.kv_path, "bootstrap/super-admin");
        assert_eq!(c.keycloak_client_id, DEFAULT_ADMIN_MANAGER_CLIENT_ID);
        let dbg = format!("{c:?}");
        for secret in [
            "token-operator-rahasia",
            "token-pembungkus-rahasia",
            "rahasia-klien",
            "accessor-uji",
        ] {
            assert!(!dbg.contains(secret), "{secret}");
        }
        let r = BootstrapConfig::load(&EnvSource::from_pairs(base("reissue_invite"))).unwrap();
        assert_eq!((r.mode, r.email), (BootstrapMode::ReissueInvite, None));
    }

    #[test]
    fn unknown_mode_and_missing_approval_fail_fast() {
        let e = BootstrapConfig::load(&EnvSource::from_pairs(base("hapus"))).unwrap_err();
        assert_eq!(e.name(), "BOOTSTRAP_MODE");
        for missing in [
            "BOOTSTRAP_OPERATOR_TOKEN",
            "BOOTSTRAP_CG_ACCESSOR",
            "BOOTSTRAP_CG_WRAPPING_TOKEN",
        ] {
            let mut v = base("reissue_invite");
            v.retain(|(k, _)| *k != missing);
            let e = BootstrapConfig::load(&EnvSource::from_pairs(v)).unwrap_err();
            assert_eq!(e.name(), missing);
        }
    }
}
