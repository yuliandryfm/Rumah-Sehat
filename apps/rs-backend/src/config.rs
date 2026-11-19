//! Konfigurasi proses `api` (KEPUTUSAN-132): struct bertipe, gagal cepat, tanpa membaca env langsung.

use observability::{BaseConfig, ConfigError, EnvSource};

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

#[cfg(test)]
mod tests {
    use super::*;

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
