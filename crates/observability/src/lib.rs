//! Observabilitas dasar backend Ruang Sehat dan RME (F1-T10): konfigurasi bertipe (KEPUTUSAN-132),
//! log JSON dengan redaksi (ADR-005, ADR-010), aliran security log (KEPUTUSAN-124), dan middleware
//! korelasi (KEPUTUSAN-123). Lihat docs/coding-standards.md §4 dan §6.

pub mod config;
pub mod http;
pub mod log;
pub mod redact;
pub mod security;

pub use config::{BaseConfig, ConfigError, ConfigErrorKind, EnvSource, LogLevel};
pub use http::{CORRELATION_HEADER, correlation_middleware, resolve_correlation_id};
pub use log::{
    Component, InstallError, JsonLogLayer, LogSink, MemorySink, StdoutSink, install, subscriber,
};
