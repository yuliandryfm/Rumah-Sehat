//! Konfigurasi bertipe dan gagal cepat (KEPUTUSAN-132, coding-standards §6).
//!
//! Setiap proses memuat konfigurasi ke satu struct saat start. Galat hanya menyebut NAMA
//! variabel dan jenis masalah, tidak pernah nilainya, sehingga rahasia tidak bocor lewat
//! pesan galat. Kode lain tidak boleh membaca variabel lingkungan langsung (lint clippy
//! `disallowed_methods` di `clippy.toml`); [`EnvSource::from_process`] satu-satunya pengecualian.

use foundation::Sensitive;
use std::collections::HashMap;
use std::fmt;
use std::net::SocketAddr;
use std::str::FromStr;

/// Jenis masalah konfigurasi.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigErrorKind {
    /// Variabel wajib tidak ada.
    Missing,
    /// Variabel ada tetapi kosong.
    Empty,
    /// Nilai tidak dapat diuraikan atau di luar yang diizinkan.
    Invalid,
}

/// Galat konfigurasi. Sengaja hanya memuat nama variabel, tidak nilai.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigError {
    kind: ConfigErrorKind,
    name: String,
}

impl ConfigError {
    fn new(kind: ConfigErrorKind, name: &str) -> Self {
        Self {
            kind,
            name: name.to_string(),
        }
    }

    /// Jenis masalah.
    pub fn kind(&self) -> ConfigErrorKind {
        self.kind
    }

    /// Nama variabel yang bermasalah.
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let what = match self.kind {
            ConfigErrorKind::Missing => "tidak ada",
            ConfigErrorKind::Empty => "kosong",
            ConfigErrorKind::Invalid => "tidak valid",
        };
        write!(f, "konfigurasi {}: variabel {what}", self.name)
    }
}

impl std::error::Error for ConfigError {}

/// Sumber variabel konfigurasi. Di production berasal dari lingkungan proses; di test dari pasangan literal.
#[derive(Clone, Default)]
pub struct EnvSource {
    vars: HashMap<String, String>,
}

// Debug tidak mencetak isi: sumber bisa memuat rahasia.
impl fmt::Debug for EnvSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "EnvSource({} variabel)", self.vars.len())
    }
}

impl EnvSource {
    /// Membaca lingkungan proses (nilai non-UTF-8 diabaikan).
    /// Satu-satunya pembacaan variabel lingkungan yang sah (KEPUTUSAN-132).
    #[allow(clippy::disallowed_methods)]
    pub fn from_process() -> Self {
        let vars = std::env::vars_os()
            .filter_map(|(k, v)| Some((k.into_string().ok()?, v.into_string().ok()?)))
            .collect();
        Self { vars }
    }

    /// Membuat sumber dari pasangan nama-nilai (untuk test).
    pub fn from_pairs<'a>(pairs: impl IntoIterator<Item = (&'a str, &'a str)>) -> Self {
        Self {
            vars: pairs
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        }
    }

    fn raw(&self, name: &str) -> Option<&str> {
        self.vars.get(name).map(String::as_str)
    }

    /// Variabel wajib, diuraikan lewat `FromStr`.
    pub fn required<T: FromStr>(&self, name: &str) -> Result<T, ConfigError> {
        match self.raw(name) {
            None => Err(ConfigError::new(ConfigErrorKind::Missing, name)),
            Some("") => Err(ConfigError::new(ConfigErrorKind::Empty, name)),
            // Galat FromStr dibuang: bisa memuat nilai.
            Some(v) => v
                .parse::<T>()
                .map_err(|_| ConfigError::new(ConfigErrorKind::Invalid, name)),
        }
    }

    /// Variabel opsional dengan nilai bawaan; bila ada tetapi tidak valid tetap galat.
    pub fn optional_or<T: FromStr>(&self, name: &str, default: T) -> Result<T, ConfigError> {
        match self.raw(name) {
            None => Ok(default),
            Some(_) => self.required(name),
        }
    }

    /// Variabel rahasia wajib, dibungkus [`Sensitive`].
    pub fn required_secret(&self, name: &str) -> Result<Sensitive<String>, ConfigError> {
        match self.raw(name) {
            None => Err(ConfigError::new(ConfigErrorKind::Missing, name)),
            Some("") => Err(ConfigError::new(ConfigErrorKind::Empty, name)),
            Some(v) => Ok(Sensitive::new(v.to_string())),
        }
    }
}

/// Tingkat log yang diizinkan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    /// Hanya galat.
    Error,
    /// Peringatan ke atas.
    Warn,
    /// Informasi ke atas (bawaan).
    Info,
    /// Debug ke atas.
    Debug,
    /// Semua, termasuk trace.
    Trace,
}

impl FromStr for LogLevel {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "error" => Ok(Self::Error),
            "warn" => Ok(Self::Warn),
            "info" => Ok(Self::Info),
            "debug" => Ok(Self::Debug),
            "trace" => Ok(Self::Trace),
            _ => Err(()),
        }
    }
}

impl LogLevel {
    pub(crate) fn as_tracing(self) -> tracing::Level {
        match self {
            Self::Error => tracing::Level::ERROR,
            Self::Warn => tracing::Level::WARN,
            Self::Info => tracing::Level::INFO,
            Self::Debug => tracing::Level::DEBUG,
            Self::Trace => tracing::Level::TRACE,
        }
    }
}

/// Konfigurasi dasar setiap proses backend: `LOG_LEVEL` (bawaan `info`) dan `HTTP_BIND` (bawaan `127.0.0.1:8080`).
/// Konfigurasi per proses menambah field sendiri dengan pola yang sama (rahasia bertipe `Sensitive<T>`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaseConfig {
    /// Tingkat application log.
    pub log_level: LogLevel,
    /// Alamat listen HTTP.
    pub http_bind: SocketAddr,
}

impl BaseConfig {
    /// Memuat dari sumber; gagal cepat pada nilai tidak valid, tanpa mencetak nilai.
    pub fn load(env: &EnvSource) -> Result<Self, ConfigError> {
        Ok(Self {
            log_level: env.optional_or("LOG_LEVEL", LogLevel::Info)?,
            http_bind: env.optional_or("HTTP_BIND", SocketAddr::from(([127, 0, 0, 1], 8080)))?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_apply_when_absent() {
        let c = BaseConfig::load(&EnvSource::default()).unwrap();
        assert_eq!(c.log_level, LogLevel::Info);
        assert_eq!(c.http_bind.port(), 8080);
    }

    #[test]
    fn reads_values() {
        let env = EnvSource::from_pairs([("LOG_LEVEL", "debug"), ("HTTP_BIND", "0.0.0.0:9000")]);
        let c = BaseConfig::load(&env).unwrap();
        assert_eq!(c.log_level, LogLevel::Debug);
        assert_eq!(c.http_bind.port(), 9000);
    }

    #[test]
    fn invalid_value_fails_fast_without_echoing_value() {
        let env = EnvSource::from_pairs([("LOG_LEVEL", "SuperSecretValue-9f3a")]);
        let err = BaseConfig::load(&env).unwrap_err();
        assert_eq!(err.kind(), ConfigErrorKind::Invalid);
        assert_eq!(err.name(), "LOG_LEVEL");
        let shown = format!("{err} | {err:?}");
        assert!(!shown.contains("SuperSecretValue"), "{shown}");
        assert!(shown.contains("LOG_LEVEL"));
    }

    #[test]
    fn invalid_bind_does_not_echo_value() {
        let env = EnvSource::from_pairs([("HTTP_BIND", "pw=hunter2-not-an-addr")]);
        let err = BaseConfig::load(&env).unwrap_err();
        assert!(!format!("{err} {err:?}").contains("hunter2"));
    }

    #[test]
    fn missing_and_empty_required_are_distinguished() {
        let env = EnvSource::from_pairs([("DB_PASSWORD", "")]);
        assert_eq!(
            env.required::<String>("NOPE").unwrap_err().kind(),
            ConfigErrorKind::Missing
        );
        assert_eq!(
            env.required::<String>("DB_PASSWORD").unwrap_err().kind(),
            ConfigErrorKind::Empty
        );
        assert_eq!(
            env.required_secret("DB_PASSWORD").unwrap_err().kind(),
            ConfigErrorKind::Empty
        );
    }

    #[test]
    fn secret_is_sensitive_and_source_debug_hides_values() {
        let env = EnvSource::from_pairs([("DB_PASSWORD", "s3cr3t-value-123")]);
        let secret = env.required_secret("DB_PASSWORD").unwrap();
        assert_eq!(secret.expose(), "s3cr3t-value-123");
        assert!(!format!("{secret:?} {secret} {env:?}").contains("s3cr3t"));
    }
}
