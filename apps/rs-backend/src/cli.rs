//! Pemilihan subcommand dan siklus hidup proses. Satu biner, satu argumen: nama subcommand.
//!
//! Yang berfungsi: `api`, `audit-relay` (F1-T24), dan `audit-verify` (F1-T25). `worker` adalah kerangka:
//! proses mencatat galat `subcommand_not_implemented` lalu keluar dengan kode gagal, supaya tidak ada
//! penyebaran yang mengira proses itu berjalan.

use crate::app::{AppState, build_router};
use crate::config::{ApiConfig, AuditRelayConfig, AuditVerifyConfig};
use audit_store::anchor::{AnchorStore, FileAnchorStore};
use audit_store::verify::{VerifierSettings, VerifyOutcome};
use foundation::{Clock, SystemClock};
use observability::{BaseConfig, Component, ConfigError, EnvSource, JsonLogLayer, install};
use std::process::ExitCode;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;

/// Teks bantuan.
pub const USAGE: &str = "pemakaian: rs-backend <api|worker|audit-relay|audit-verify>";

/// Subcommand yang dikenal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    /// Server HTTP API.
    Api,
    /// Pekerja job (kerangka).
    Worker,
    /// Relay audit (F1-T24).
    AuditRelay,
    /// Verifikasi rantai audit (F1-T25).
    AuditVerify,
}

/// Galat argumen baris perintah. Nilai argumen tidak pernah disertakan di pesan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CliError {
    /// Tidak ada subcommand.
    Missing,
    /// Nama subcommand tidak dikenal.
    Unknown,
    /// Argumen lebih dari satu.
    Extra,
}

impl Command {
    /// Mengurai argumen setelah nama program.
    pub fn parse(args: &[String]) -> Result<Self, CliError> {
        let mut it = args.iter();
        let name = it.next().ok_or(CliError::Missing)?;
        if it.next().is_some() {
            return Err(CliError::Extra);
        }
        match name.as_str() {
            "api" => Ok(Self::Api),
            "worker" => Ok(Self::Worker),
            "audit-relay" => Ok(Self::AuditRelay),
            "audit-verify" => Ok(Self::AuditVerify),
            _ => Err(CliError::Unknown),
        }
    }

    /// Nilai field `component` di log (coding-standards §4.1).
    pub fn component(self) -> Component {
        match self {
            Self::Api => Component::Api,
            Self::Worker => Component::Worker,
            Self::AuditRelay => Component::AuditRelay,
            Self::AuditVerify => Component::AuditVerify,
        }
    }

    /// Nama subcommand.
    pub fn name(self) -> &'static str {
        match self {
            Self::Api => "api",
            Self::Worker => "worker",
            Self::AuditRelay => "audit-relay",
            Self::AuditVerify => "audit-verify",
        }
    }
}

/// Konfigurasi bertipe per subcommand (KEPUTUSAN-132): setiap proses hanya memuat yang dibutuhkannya.
enum Loaded {
    Api(ApiConfig),
    AuditRelay(AuditRelayConfig),
    AuditVerify(AuditVerifyConfig),
    Skeleton(Command, BaseConfig),
}

impl Loaded {
    fn load(command: Command, env: &EnvSource) -> Result<Self, ConfigError> {
        Ok(match command {
            Command::Api => Self::Api(ApiConfig::load(env)?),
            Command::AuditRelay => Self::AuditRelay(AuditRelayConfig::load(env)?),
            Command::AuditVerify => Self::AuditVerify(AuditVerifyConfig::load(env)?),
            other => Self::Skeleton(other, BaseConfig::load(env)?),
        })
    }

    fn base(&self) -> &BaseConfig {
        match self {
            Self::Api(c) => &c.base,
            Self::AuditRelay(c) => &c.base,
            Self::AuditVerify(c) => &c.base,
            Self::Skeleton(_, b) => b,
        }
    }
}

/// Menjalankan proses. `args` adalah argumen setelah nama program.
pub fn run(args: &[String], env: &EnvSource) -> ExitCode {
    let command = match Command::parse(args) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{USAGE} ({e:?})");
            return ExitCode::from(2);
        }
    };
    let loaded = match Loaded::load(command, env) {
        Ok(c) => c,
        Err(e) => {
            // ConfigError tidak memuat nilai, hanya nama variabel dan jenis galat.
            eprintln!("konfigurasi tidak valid: {e}");
            return ExitCode::from(78);
        }
    };
    let layer = JsonLogLayer::new(
        command.component(),
        loaded.base().log_level,
        Arc::new(SystemClock),
    );
    if install(layer).is_err() {
        eprintln!("subscriber log global sudah terpasang");
        return ExitCode::FAILURE;
    }
    match loaded {
        Loaded::Api(config) => run_api(config),
        Loaded::AuditRelay(config) => run_audit_relay(config),
        Loaded::AuditVerify(config) => run_audit_verify(config),
        Loaded::Skeleton(other, base) => not_implemented(other, &base),
    }
}

fn runtime() -> Option<tokio::runtime::Runtime> {
    match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(r) => Some(r),
        Err(_) => {
            tracing::error!("runtime_start_failed");
            None
        }
    }
}

/// `audit-relay`: satu relay aktif (kunci 7001001), instans lain siaga; berhenti pada SIGINT/SIGTERM.
fn run_audit_relay(config: AuditRelayConfig) -> ExitCode {
    let Some(runtime) = runtime() else {
        return ExitCode::FAILURE;
    };
    let anchors: Arc<dyn AnchorStore> = match FileAnchorStore::new(&config.anchor_dir) {
        Ok(s) => Arc::new(s),
        Err(_) => {
            tracing::error!("audit_anchor_dir_unusable");
            return ExitCode::FAILURE;
        }
    };
    let settings = audit::relay::RelaySettings {
        lock_key: platform_jobs::lock::RS_AUDIT_RELAY,
        app_database_url: config.app_database_url,
        audit_database_url: config.audit_database_url,
        batch_size: config.batch_size.get(),
        poll_interval: Duration::from_millis(config.poll_ms),
        retry_interval: Duration::from_millis(config.retry_ms),
        anchors,
        anchor_interval: Duration::from_secs(config.anchor_interval_secs),
    };
    let clock: Arc<dyn Clock> = Arc::new(SystemClock);
    tracing::info!(lock_key = settings.lock_key.key(), "audit_relay_starting");
    runtime.block_on(async move {
        tokio::select! {
            () = audit::relay::run_relay(settings, clock) => {}
            () = shutdown_signal() => {}
        }
    });
    tracing::info!("audit_relay_stopped");
    ExitCode::SUCCESS
}

fn not_implemented(command: Command, _base: &BaseConfig) -> ExitCode {
    tracing::error!(
        subcommand = command.name(),
        "subcommand_not_implemented: kerangka F1-T11, belum berfungsi"
    );
    ExitCode::FAILURE
}

/// `audit-verify`: verifikasi rantai harian (kunci 7001003) atau sekali (`AUDIT_VERIFY_ONCE`).
fn run_audit_verify(config: AuditVerifyConfig) -> ExitCode {
    let Some(runtime) = runtime() else {
        return ExitCode::FAILURE;
    };
    let anchors: Arc<dyn AnchorStore> = match FileAnchorStore::new(&config.anchor_dir) {
        Ok(s) => Arc::new(s),
        Err(_) => {
            tracing::error!("audit_anchor_dir_unusable");
            return ExitCode::FAILURE;
        }
    };
    let settings = VerifierSettings {
        lock_key: platform_jobs::lock::RS_AUDIT_VERIFY,
        database_url: config.database_url,
        anchors,
        interval: Duration::from_secs(config.interval_secs),
        retry_interval: Duration::from_millis(config.retry_ms),
    };
    let clock: Arc<dyn Clock> = Arc::new(SystemClock);
    tracing::info!(
        lock_key = settings.lock_key.key(),
        once = config.once,
        "audit_verify_starting"
    );
    if config.once {
        let result = runtime.block_on(audit_store::verify::verify_once(&settings, clock.as_ref()));
        return match result {
            Ok((report, Some(_))) if report.outcome == VerifyOutcome::Match => ExitCode::SUCCESS,
            _ => ExitCode::FAILURE,
        };
    }
    runtime.block_on(async move {
        tokio::select! {
            () = audit_store::verify::run_verifier(settings, clock) => {}
            () = shutdown_signal() => {}
        }
    });
    tracing::info!("audit_verify_stopped");
    ExitCode::SUCCESS
}

fn run_api(config: ApiConfig) -> ExitCode {
    let Some(runtime) = runtime() else {
        return ExitCode::FAILURE;
    };
    match runtime.block_on(serve(config)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => {
            tracing::error!("api_stopped_with_error");
            ExitCode::FAILURE
        }
    }
}

async fn serve(config: ApiConfig) -> std::io::Result<()> {
    let listener = TcpListener::bind(config.base.http_bind).await?;
    tracing::info!(bind = %listener.local_addr()?, "api_listening");
    let state = AppState::new(&config.problem_type_base, Vec::new());
    let router = build_router(state).map_err(|e| {
        // Rute tanpa kebijakan atau kebijakan tidak valid: start gagal (KEPUTUSAN-127).
        tracing::error!("policy_setup_failed: {e}");
        std::io::Error::other("policy_setup_failed")
    })?;
    router.serve(listener, shutdown_signal()).await
}

/// Selesai pada SIGINT atau SIGTERM (Docker Compose mengirim SIGTERM).
async fn shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};
        match signal(SignalKind::terminate()) {
            Ok(mut term) => {
                tokio::select! {
                    _ = tokio::signal::ctrl_c() => {}
                    _ = term.recv() => {}
                }
            }
            Err(_) => {
                let _ = tokio::signal::ctrl_c().await;
            }
        }
    }
    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn a(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| (*s).to_string()).collect()
    }

    #[test]
    fn parses_known_commands() {
        assert_eq!(Command::parse(&a(&["api"])), Ok(Command::Api));
        assert_eq!(Command::parse(&a(&["worker"])), Ok(Command::Worker));
        assert_eq!(
            Command::parse(&a(&["audit-relay"])),
            Ok(Command::AuditRelay)
        );
        assert_eq!(
            Command::parse(&a(&["audit-verify"])),
            Ok(Command::AuditVerify)
        );
    }

    #[test]
    fn rejects_missing_unknown_and_extra() {
        assert_eq!(Command::parse(&[]), Err(CliError::Missing));
        assert_eq!(Command::parse(&a(&["realtime"])), Err(CliError::Unknown));
        assert_eq!(Command::parse(&a(&["API"])), Err(CliError::Unknown));
        assert_eq!(Command::parse(&a(&["api", "x"])), Err(CliError::Extra));
    }

    #[test]
    fn names_roundtrip() {
        for c in [
            Command::Api,
            Command::Worker,
            Command::AuditRelay,
            Command::AuditVerify,
        ] {
            assert_eq!(Command::parse(&a(&[c.name()])), Ok(c));
        }
    }
}
