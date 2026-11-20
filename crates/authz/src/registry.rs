//! Registri kebijakan per endpoint `(metode, pola path)` dan pemeriksaan cakupan saat start
//! (KEPUTUSAN-127).

use crate::error::PolicySetupError;
use crate::policy::Policy;
use axum::http::Method;
use std::collections::BTreeMap;
use utoipa::openapi::OpenApi;
use utoipa::openapi::path::PathItem;

/// Kunci registri: metode (huruf besar) dan pola path axum/OpenAPI (`/x/{id}`).
pub type EndpointKey = (String, String);

/// Kebijakan semua endpoint yang terdaftar.
#[derive(Debug, Default, Clone)]
pub struct PolicyRegistry {
    entries: BTreeMap<EndpointKey, Policy>,
}

fn key(method: &Method, path: &str) -> EndpointKey {
    // HEAD dilayani handler GET oleh axum, sehingga memakai kebijakan GET.
    let m = if method == Method::HEAD {
        Method::GET
    } else {
        method.clone()
    };
    (m.as_str().to_string(), path.to_string())
}

/// Metode operasi yang dideklarasikan sebuah `PathItem` OpenAPI.
pub(crate) fn operations(item: &PathItem) -> Vec<Method> {
    let mut out = Vec::new();
    let pairs = [
        (item.get.is_some(), Method::GET),
        (item.put.is_some(), Method::PUT),
        (item.post.is_some(), Method::POST),
        (item.delete.is_some(), Method::DELETE),
        (item.options.is_some(), Method::OPTIONS),
        (item.head.is_some(), Method::HEAD),
        (item.patch.is_some(), Method::PATCH),
        (item.trace.is_some(), Method::TRACE),
    ];
    for (present, m) in pairs {
        if present {
            out.push(m);
        }
    }
    // Operasi QUERY (OpenAPI 3.2) ikut dihitung supaya tidak lolos dari pemeriksaan cakupan.
    if item.query.is_some()
        && let Ok(m) = Method::from_bytes(b"QUERY")
    {
        out.push(m);
    }
    out
}

impl PolicyRegistry {
    /// Mendaftarkan kebijakan satu endpoint. Endpoint ganda adalah galat penyiapan.
    pub fn insert(
        &mut self,
        method: &Method,
        path: &str,
        policy: Policy,
    ) -> Result<(), PolicySetupError> {
        let k = key(method, path);
        if self.entries.contains_key(&k) {
            return Err(PolicySetupError::DuplicatePolicy {
                method: k.0,
                path: k.1,
            });
        }
        self.entries.insert(k, policy);
        Ok(())
    }

    /// Kebijakan untuk permintaan `method` ke pola `path`; `None` = tidak terdaftar (ditolak).
    pub fn get(&self, method: &Method, path: &str) -> Option<&Policy> {
        self.entries.get(&key(method, path))
    }

    /// Semua entri.
    pub fn iter(&self) -> impl Iterator<Item = (&EndpointKey, &Policy)> {
        self.entries.iter()
    }

    /// Jumlah entri.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Registri kosong.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Pemeriksaan saat start (KEPUTUSAN-127): setiap operasi di spesifikasi OpenAPI router punya
/// kebijakan, dan setiap kebijakan menunjuk operasi yang ada. Spesifikasi dan rute axum dibangun dari
/// anotasi yang sama (utoipa-axum), sehingga operasi di spesifikasi = rute yang terpasang.
pub fn check_coverage(spec: &OpenApi, registry: &PolicyRegistry) -> Result<(), PolicySetupError> {
    let mut declared: Vec<EndpointKey> = Vec::new();
    for (path, item) in &spec.paths.paths {
        for m in operations(item) {
            let k = key(&m, path);
            if !registry.entries.contains_key(&k) {
                return Err(PolicySetupError::MissingPolicy {
                    method: k.0,
                    path: k.1,
                });
            }
            declared.push(k);
        }
    }
    for k in registry.entries.keys() {
        if !declared.contains(k) {
            return Err(PolicySetupError::StalePolicy {
                method: k.0.clone(),
                path: k.1.clone(),
            });
        }
    }
    Ok(())
}
