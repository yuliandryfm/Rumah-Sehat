//! Pemilihan subcommand dan siklus hidup proses. Satu biner, satu argumen: nama subcommand.
//!
//! Semua subcommand berfungsi: `api`, `worker` (F1-T34: pengambil event Keycloak, kunci 7001002; antrean job
//! menyusul di task pemakainya), `audit-relay` (F1-T24), `audit-verify` (F1-T25), dan `email-relay` (F1-T36,
//! ADR-026).

use crate::access_audit::AuditDenialRecorder;
use crate::app::{AppServices, AppState, build_router};
use crate::config::{
    ApiConfig, AuditRelayConfig, AuditVerifyConfig, EmailRelayConfig, WorkerConfig,
};
use crate::http_support::EnvelopeRenderer;
use crate::modules::identity::api::AccountDirectory;
use crate::modules::platform::api::{DatabaseReadiness, FileStorage, ReadinessCheck};
use crate::modules::profile::api::IdentityServices;
use crate::rate_limit::RateLimiter;
use audit_store::anchor::{AnchorStore, FileAnchorStore};
use audit_store::verify::{VerifierSettings, VerifyOutcome};
use authn::{
    Algorithm, AuthState, Authenticator, AuthnConfig, Backend, HttpJwksSource, Realm, RealmConfig,
};
use authz::{DEFAULT_DENIAL_AUDIT_CONCURRENCY, DEFAULT_DENIAL_QUEUE_CAPACITY, DenialSink};
use crypto::{DataKeyring, Keyring, KeyringConfig, System, TransitConfig, TransitKek};
use foundation::{Clock, SystemClock};
use observability::{BaseConfig, Component, ConfigError, EnvSource, JsonLogLayer, install};
use platform_jobs::lock::RS_KEYCLOAK_EVENT_PULLER;
use sqlx::postgres::PgPoolOptions;
use std::process::ExitCode;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use valkey::{Valkey, ValkeyConfig};

/// Teks bantuan.
pub const USAGE: &str = "pemakaian: rs-backend <api|worker|audit-relay|audit-verify|email-relay>";

/// Subcommand yang dikenal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    /// Server HTTP API.
    Api,
    /// Pekerja latar: pengambil event Keycloak (F1-T34).
    Worker,
    /// Relay audit (F1-T24).
    AuditRelay,
    /// Verifikasi rantai audit (F1-T25).
    AuditVerify,
    /// Relay SMTP ke SES API v2 (F1-T36, ADR-026).
    EmailRelay,
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
            "email-relay" => Ok(Self::EmailRelay),
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
            Self::EmailRelay => Component::EmailRelay,
        }
    }

    /// Nama subcommand.
    pub fn name(self) -> &'static str {
        match self {
            Self::Api => "api",
            Self::Worker => "worker",
            Self::AuditRelay => "audit-relay",
            Self::AuditVerify => "audit-verify",
            Self::EmailRelay => "email-relay",
        }
    }
}

/// Konfigurasi bertipe per subcommand (KEPUTUSAN-132): setiap proses hanya memuat yang dibutuhkannya.
enum Loaded {
    Api(Box<ApiConfig>),
    Worker(WorkerConfig),
    AuditRelay(AuditRelayConfig),
    AuditVerify(AuditVerifyConfig),
    EmailRelay(Box<EmailRelayConfig>),
}

impl Loaded {
    fn load(command: Command, env: &EnvSource) -> Result<Self, ConfigError> {
        Ok(match command {
            Command::Api => Self::Api(Box::new(ApiConfig::load(env)?)),
            Command::Worker => Self::Worker(WorkerConfig::load(env)?),
            Command::AuditRelay => Self::AuditRelay(AuditRelayConfig::load(env)?),
            Command::AuditVerify => Self::AuditVerify(AuditVerifyConfig::load(env)?),
            Command::EmailRelay => Self::EmailRelay(Box::new(EmailRelayConfig::load(env)?)),
        })
    }

    fn base(&self) -> &BaseConfig {
        match self {
            Self::Api(c) => &c.base,
            Self::Worker(c) => &c.base,
            Self::AuditRelay(c) => &c.base,
            Self::AuditVerify(c) => &c.base,
            Self::EmailRelay(c) => &c.base,
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
        Loaded::Api(config) => run_api(*config),
        Loaded::Worker(config) => run_worker(config),
        Loaded::AuditRelay(config) => run_audit_relay(config),
        Loaded::AuditVerify(config) => run_audit_verify(config),
        Loaded::EmailRelay(config) => run_email_relay(*config),
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

/// Batas waktu satu permintaan ke OpenBao saat memuat kredensial SES.
const OPENBAO_TIMEOUT: Duration = Duration::from_secs(5);

/// `email-relay` (F1-T36, ADR-026): memuat sertifikat TLS relay dan kunci akses SES dari OpenBao KV saat start
/// (rotasi kunci = restart proses), lalu melayani SMTP sampai SIGINT/SIGTERM. Galat start keluar dengan kode
/// gagal tanpa mencetak nilai rahasia.
fn run_email_relay(config: EmailRelayConfig) -> ExitCode {
    let Some(runtime) = runtime() else {
        return ExitCode::FAILURE;
    };
    runtime.block_on(async move {
        let tls = match email_relay::server_tls(&config.tls_cert_file, &config.tls_key_file) {
            Ok(t) => t,
            Err(e) => {
                tracing::error!(error = %e, "relay_tls_setup_failed");
                return ExitCode::FAILURE;
            }
        };
        let openbao = crypto::OpenBaoConfig {
            address: config.openbao_addr.clone(),
            token: config.openbao_token.clone(),
            timeout: OPENBAO_TIMEOUT,
        };
        let credentials =
            match ses::load_from_openbao(&openbao, &config.ses_kv_mount, &config.ses_kv_path).await
            {
                Ok(c) => c,
                Err(e) => {
                    tracing::error!(error = %e, "relay_ses_credentials_failed");
                    return ExitCode::FAILURE;
                }
            };
        let client = match ses::SesClient::new(&ses::SesConfig {
            local_endpoint: config.ses_local_endpoint.clone(),
            ca_file: config.ses_ca_file.clone(),
            timeout: Duration::from_millis(config.ses_timeout_ms),
            configuration_set: config.ses_configuration_set.clone(),
            credentials,
        }) {
            Ok(c) => c,
            Err(e) => {
                tracing::error!(error = %e, "relay_ses_setup_failed");
                return ExitCode::FAILURE;
            }
        };
        let sender = email_relay::SesSender::new(client, Arc::new(SystemClock));
        let relay = match email_relay::Relay::new(config.relay_settings(), tls, sender) {
            Ok(r) => r,
            Err(e) => {
                tracing::error!(error = %e, "relay_setup_failed");
                return ExitCode::FAILURE;
            }
        };
        let listener = match TcpListener::bind(config.bind).await {
            Ok(l) => l,
            Err(_) => {
                tracing::error!("relay_bind_failed");
                return ExitCode::FAILURE;
            }
        };
        match listener.local_addr() {
            Ok(addr) => tracing::info!(bind = %addr, "relay_listening"),
            Err(_) => tracing::info!("relay_listening"),
        }
        tokio::select! {
            () = relay.serve(listener) => {}
            () = shutdown_signal() => {}
        }
        tracing::info!("relay_stopped");
        ExitCode::SUCCESS
    })
}

/// `worker`: pengambil event Keycloak E-01–E-04 ke `audit_outbox`, satu aktif (kunci 7001002), instans lain
/// siaga; berhenti pada SIGINT/SIGTERM (ADR-011 butir 10, F1-T34).
fn run_worker(config: WorkerConfig) -> ExitCode {
    use crate::modules::identity::api::{PullerSettings, run_puller};
    let Some(runtime) = runtime() else {
        return ExitCode::FAILURE;
    };
    let events = config.keycloak_events;
    let client = match events.client() {
        Ok(c) => Arc::new(c),
        Err(_) => {
            tracing::error!("keycloak_event_client_invalid");
            return ExitCode::FAILURE;
        }
    };
    let settings = PullerSettings {
        options: events.options(),
        database_url: config.database_url,
        poll_interval: events.poll_interval(),
        retry_interval: events.retry_interval(),
    };
    let clock: Arc<dyn Clock> = Arc::new(SystemClock);
    tracing::info!(
        lock_key = RS_KEYCLOAK_EVENT_PULLER.key(),
        realms = settings.options.realms.len(),
        "worker_starting"
    );
    runtime.block_on(async move {
        tokio::select! {
            () = run_puller(settings, client, clock) => {}
            () = shutdown_signal() => {}
        }
    });
    tracing::info!("worker_stopped");
    ExitCode::SUCCESS
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
    // Galat penyiapan dicatat dengan nama bagian saja (tanpa nilai konfigurasi, KEPUTUSAN-132).
    let setup_failed = |part: &'static str| {
        tracing::error!(part = part, "api_setup_failed");
        std::io::Error::other("api_setup_failed")
    };
    let clock: Arc<dyn Clock> = Arc::new(SystemClock);
    let pool = PgPoolOptions::new()
        // Skema fisik §10.3 (KEPUTUSAN-139): acquire 3 dtk untuk api, idle 10 mnt, umur 30 mnt.
        .max_connections(config.database_max_connections.get())
        .acquire_timeout(Duration::from_secs(3))
        .idle_timeout(Duration::from_secs(600))
        .max_lifetime(Duration::from_secs(1800))
        .connect_lazy(config.database_url.expose())
        .map_err(|_| setup_failed("api_database_url"))?;
    let files =
        Arc::new(FileStorage::new(&config.files).map_err(|_| setup_failed("object_storage"))?);
    let jwks = HttpJwksSource::new(&config.admin_realm.jwks_url, Duration::from_secs(5))
        .map_err(|_| setup_failed("auth_admin_jwks_url"))?;
    let patient_jwks = HttpJwksSource::new(&config.patient_realm.jwks_url, Duration::from_secs(5))
        .map_err(|_| setup_failed("auth_patient_jwks_url"))?;
    let authenticator = Authenticator::new(
        AuthnConfig::with_defaults(
            Backend::RuangSehat,
            vec![
                RealmConfig {
                    realm: Realm::Admin,
                    issuer: config.admin_realm.issuer.clone(),
                    audiences: vec![config.admin_realm.audience.clone()],
                    algorithms: vec![Algorithm::RS256],
                    authorized_parties: Vec::new(),
                    jwks: Arc::new(jwks),
                },
                // Realm pasien (F1-T61): endpoint /patient/v1, pola yang sama dengan realm admin.
                RealmConfig {
                    realm: Realm::Pasien,
                    issuer: config.patient_realm.issuer.clone(),
                    audiences: vec![config.patient_realm.audience.clone()],
                    algorithms: vec![Algorithm::RS256],
                    authorized_parties: Vec::new(),
                    jwks: Arc::new(patient_jwks),
                },
            ],
        ),
        Arc::clone(&clock),
    )
    .map_err(|_| setup_failed("auth_realms"))?;

    let auth = AuthState::new(
        Arc::new(authenticator),
        Arc::new(EnvelopeRenderer::new(&config.problem_type_base)),
    );
    let valkey = Valkey::connect_lazy(&ValkeyConfig {
        url: config.valkey_url.clone(),
        op_timeout: Duration::from_millis(config.valkey_op_timeout_ms),
    })
    .map_err(|_| setup_failed("api_valkey_url"))?;
    let identity = identity_services(
        &config,
        pool.clone(),
        Arc::clone(&clock),
        &files,
        valkey.clone(),
    )
    .await
    .map_err(setup_failed)?;
    // Resolver akun + penjaga sesi yang dicabut (F1-T33): Valkey dengan cadangan PostgreSQL (RT-18).
    let directory = Arc::new(AccountDirectory::new(
        pool.clone(),
        valkey,
        Arc::clone(&clock),
        Duration::from_secs(crate::modules::identity::api::DEFAULT_ACCOUNT_CACHE_TTL_SECONDS),
    ));
    // Penolakan objek data sensitif dicatat E-05 hasil gagal (F1-T38).
    let denials = DenialSink::start(
        DEFAULT_DENIAL_QUEUE_CAPACITY,
        DEFAULT_DENIAL_AUDIT_CONCURRENCY,
        Some(Arc::new(AuditDenialRecorder::new(
            pool.clone(),
            Arc::clone(&clock),
        ))),
    )
    .map_err(|_| setup_failed("denial_sink"))?;
    let readiness: Vec<Arc<dyn ReadinessCheck>> =
        vec![Arc::new(DatabaseReadiness::new(pool.clone()))];
    let state = AppState::new(
        &config.problem_type_base,
        readiness,
        AppServices {
            pool,
            clock,
            files,
            identity: Arc::new(identity),
            public_content_limit: Arc::new(RateLimiter::new(config.public_rate)),
            public_image_limit: Arc::new(RateLimiter::new(config.public_rate)),
        },
    );
    let router = build_router(state, auth, directory.clone(), directory, denials).map_err(|e| {
        // Rute tanpa kebijakan atau kebijakan tidak valid: start gagal (KEPUTUSAN-127).
        tracing::error!("policy_setup_failed: {e}");
        std::io::Error::other("policy_setup_failed")
    })?;
    tracing::info!(bind = %listener.local_addr()?, "api_listening");
    // Alamat peer TCP (ConnectInfo -> authz::ClientIp) untuk sumber audit dan batas laju per sumber. Di balik
    // reverse proxy alamat ini milik proxy; IP klien asli `[TIDAK DIKETAHUI]` (docs/05-stack.md, PT-39).
    router
        .serve_with_peer_addr(listener, shutdown_signal())
        .await
}

/// F1-T61 (KEPUTUSAN-293): keyring DEK lewat OpenBao Transit dan kunci blind index NIK dari OpenBao KV,
/// dimuat saat start (gagal cepat, pola `email-relay`). Galat dikembalikan sebagai nama bagian saja.
async fn identity_services(
    config: &ApiConfig,
    pool: sqlx::PgPool,
    clock: Arc<dyn Clock>,
    files: &Arc<FileStorage>,
    valkey: Valkey,
) -> Result<IdentityServices, &'static str> {
    let keys = &config.identity_keys;
    let openbao = crypto::OpenBaoConfig {
        address: keys.openbao_addr.clone(),
        token: keys.openbao_token.clone(),
        timeout: OPENBAO_TIMEOUT,
    };
    let kek = TransitKek::new(
        &openbao,
        TransitConfig {
            mount: keys.transit_mount.clone(),
            key_name: keys.transit_key.clone(),
        },
    )
    .map_err(|_| "openbao_transit")?;
    let keyring = Keyring::new(
        kek,
        KeyringConfig {
            system: System::RuangSehat,
            cache_ttl: time::Duration::seconds(i64::from(keys.dek_cache_ttl.seconds())),
            cache_capacity: keys.dek_cache_capacity.get(),
        },
        Arc::clone(&clock),
    )
    .map_err(|_| "dek_keyring")?;
    let blind_index = crypto::load_nik_blind_index_keyring(
        &openbao,
        &keys.blind_index_kv_mount,
        &keys.blind_index_kv_path,
    )
    .await
    .map_err(|_| "nik_blind_index_keys")?;
    let keyring: Arc<dyn DataKeyring> = Arc::new(keyring);
    Ok(IdentityServices::new(
        pool,
        clock,
        Arc::clone(files),
        keyring,
        Arc::new(blind_index),
        valkey,
    ))
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
        assert_eq!(
            Command::parse(&a(&["email-relay"])),
            Ok(Command::EmailRelay)
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
            Command::EmailRelay,
        ] {
            assert_eq!(Command::parse(&a(&[c.name()])), Ok(c));
        }
    }
}
