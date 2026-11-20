//! Membaca rahasia string dari OpenBao KV v2: `GET /v1/<mount>/data/<path>` [SRC-488].
//!
//! Dipakai untuk rahasia pihak ketiga yang dipegang satu proses saja, misalnya kunci akses AWS akun email
//! untuk relay SES (F1-T36, ADR-026 butir 5, KEPUTUSAN-79). Policy OpenBao per proses/host membatasi path
//! yang dapat dibaca; crate ini hanya membaca path yang diminta pemanggil. Nilai dibungkus [`Sensitive`] dan
//! salinan sementara dinolkan; tidak ada nilai atau nama field di galat.

use crate::error::CryptoError;
use crate::openbao::{OpenBaoClient, OpenBaoConfig, valid_path};
use foundation::Sensitive;
use http::Method;
use serde::Deserialize;
use std::collections::BTreeMap;
use zeroize::Zeroize;

#[derive(Deserialize)]
struct KvEnvelope {
    data: KvData,
}

#[derive(Deserialize)]
struct KvData {
    data: BTreeMap<String, String>,
}

/// Isi satu rahasia KV v2: nama field ke nilai. Nilai tidak pernah muncul di `Debug`.
pub struct KvSecret {
    fields: BTreeMap<String, Sensitive<String>>,
}

impl std::fmt::Debug for KvSecret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "KvSecret({} field)", self.fields.len())
    }
}

impl KvSecret {
    /// Mengambil (memindahkan) satu field; `None` bila tidak ada atau kosong.
    pub fn take(&mut self, name: &str) -> Option<Sensitive<String>> {
        self.fields.remove(name).filter(|v| !v.expose().is_empty())
    }

    fn from_data(data: BTreeMap<String, String>) -> Self {
        let fields = data
            .into_iter()
            .map(|(k, v)| (k, Sensitive::new(v)))
            .collect();
        Self { fields }
    }
}

impl Drop for KvSecret {
    fn drop(&mut self) {
        for v in std::mem::take(&mut self.fields).into_values() {
            let mut s = v.into_inner();
            s.zeroize();
        }
    }
}

/// Membaca satu rahasia KV v2. Mount dan path hanya `[a-z0-9_-]` (path boleh ber-`/`).
pub async fn read_kv_secret(
    openbao: &OpenBaoConfig,
    mount: &str,
    path: &str,
) -> Result<KvSecret, CryptoError> {
    if !valid_path(mount, false) || !valid_path(path, true) {
        return Err(CryptoError::InvalidConfig);
    }
    let client = OpenBaoClient::new(openbao)?;
    let raw = client
        .call(Method::GET, &format!("/v1/{mount}/data/{path}"), None)
        .await?;
    let parsed: KvEnvelope =
        serde_json::from_slice(&raw).map_err(|_| CryptoError::KmsInvalidResponse)?;
    Ok(KvSecret::from_data(parsed.data.data))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn take_moves_field_and_rejects_empty() {
        let mut data = BTreeMap::new();
        data.insert("a".to_string(), "nilai-a".to_string());
        data.insert("kosong".to_string(), String::new());
        let mut s = KvSecret::from_data(data);
        assert!(!format!("{s:?}").contains("nilai-a"));
        assert_eq!(
            s.take("a").map(|v| v.expose().clone()),
            Some("nilai-a".to_string())
        );
        assert!(s.take("a").is_none());
        assert!(s.take("kosong").is_none());
        assert!(s.take("tidak-ada").is_none());
    }

    #[tokio::test]
    async fn invalid_mount_or_path_is_rejected_before_any_request() {
        let cfg = OpenBaoConfig {
            address: "http://127.0.0.1:1".to_string(),
            token: Sensitive::new("uji".to_string()),
            timeout: std::time::Duration::from_secs(1),
        };
        for (m, p) in [("a/b", "x"), ("kv", "../x"), ("kv", ""), ("KV", "x")] {
            assert_eq!(
                read_kv_secret(&cfg, m, p).await.err(),
                Some(CryptoError::InvalidConfig),
                "{m} {p}"
            );
        }
    }
}
