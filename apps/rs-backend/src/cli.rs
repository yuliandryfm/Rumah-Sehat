//! Pemilihan subcommand dan siklus hidup proses. Satu biner, satu argumen: nama subcommand.
//!
//! Hanya `api` yang berfungsi. `worker`, `audit-relay`, dan `audit-verify` adalah kerangka: proses
//! mencatat galat `subcommand_not_implemented` lalu keluar dengan kode gagal, supaya tidak ada
//! penyebaran yang mengira proses itu berjalan.

use crate::app::{AppState, build_router};
use crate::config::ApiConfig;
use foundation::SystemClock;
use observability::{BaseConfig, Component, EnvSource, JsonLogLayer, install};
use std::process::ExitCode;
use std::sync::Arc;
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
    /// Relay audit (kerangka).
    AuditRelay,
    /// Verifikasi rantai audit (kerangka).
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

/// Menjalankan proses. `args` adalah argumen setelah nama program.
pub fn run(args: &[String], env: &EnvSource) -> ExitCode {
    let command = match Command::parse(args) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{USAGE} ({e:?})");
            return ExitCode::from(2);
        }
    };
    let config = match ApiConfig::load(env) {
        Ok(c) => c,
        Err(e) => {
            // ConfigError tidak memuat nilai, hanya nama variabel dan jenis galat.
            eprintln!("konfigurasi tidak valid: {e}");
            return ExitCode::from(78);
        }
    };
    let layer = JsonLogLayer::new(
        command.component(),
        config.base.log_level,
        Arc::new(SystemClock),
    );
    if install(layer).is_err() {
        eprintln!("subscriber log global sudah terpasang");
        return ExitCode::FAILURE;
    }
    match command {
        Command::Api => run_api(config),
        other => not_implemented(other, &config.base),
    }
}

fn not_implemented(command: Command, _base: &BaseConfig) -> ExitCode {
    tracing::error!(
        subcommand = command.name(),
        "subcommand_not_implemented: kerangka F1-T11, belum berfungsi"
    );
    ExitCode::FAILURE
}

fn run_api(config: ApiConfig) -> ExitCode {
    let runtime = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(r) => r,
        Err(_) => {
            tracing::error!("runtime_start_failed");
            return ExitCode::FAILURE;
        }
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
    axum::serve(listener, build_router(state))
        .with_graceful_shutdown(shutdown_signal())
        .await
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
