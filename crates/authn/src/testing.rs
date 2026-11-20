//! Pembantu test: kunci realm uji, JWKS, dan token bertanda tangan (coding-standards §7.2: token uji dari
//! kunci uji yang dibangkitkan saat test, tanpa Keycloak nyata).
//!
//! Hanya ada dengan fitur `testing`, yang dipasang lewat `[dev-dependencies]`; tidak ikut biner
//! produksi. Modul ini **tidak** dapat membangun [`crate::AuthContext`] tanpa validasi: test membuat
//! token lalu memvalidasinya lewat [`crate::Authenticator`] sungguhan.

use crate::jwks::{JwksFetchError, JwksFuture, JwksSource};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use crypto::test_keys::{RsaTestKey, TestRsaScheme};
use serde_json::{Value, json};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};

pub use crypto::test_keys::TestRsaScheme as Scheme;

/// Kunci tanda tangan satu realm uji.
#[derive(Debug)]
pub struct TestRealmKey {
    key: RsaTestKey,
    kid: String,
}

/// Galat pembantu test (pembangkitan kunci atau tanda tangan gagal).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TestKeyError;

fn b64(bytes: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(bytes)
}

impl TestRealmKey {
    /// Membangkitkan kunci RSA 2048 bit baru dengan `kid`.
    pub fn generate(kid: &str) -> Result<Self, TestKeyError> {
        Ok(Self {
            key: RsaTestKey::generate().map_err(|_| TestKeyError)?,
            kid: kid.to_string(),
        })
    }

    /// `kid` kunci ini.
    pub fn kid(&self) -> &str {
        &self.kid
    }

    /// Modulus publik mentah (untuk test serangan HS256 dengan kunci publik).
    pub fn public_modulus(&self) -> Vec<u8> {
        self.key.modulus()
    }

    /// JWK publik bergaya Keycloak (`use: sig`, `alg: RS256`).
    pub fn jwk(&self) -> Value {
        self.jwk_with_alg("RS256")
    }

    /// JWK publik dengan `alg` tertentu.
    pub fn jwk_with_alg(&self, alg: &str) -> Value {
        json!({
            "kid": self.kid,
            "kty": "RSA",
            "alg": alg,
            "use": "sig",
            "n": b64(&self.key.modulus()),
            "e": b64(&self.key.exponent()),
        })
    }

    /// Token compact RS256 dengan header `{alg: RS256, typ: JWT, kid}`.
    pub fn sign(&self, claims: &Value) -> Result<String, TestKeyError> {
        let header = json!({"alg": "RS256", "typ": "JWT", "kid": self.kid});
        self.sign_raw(&header, claims, Scheme::Rs256)
    }

    /// Token compact dengan header dan skema tanda tangan bebas (untuk test negatif).
    pub fn sign_raw(
        &self,
        header: &Value,
        claims: &Value,
        scheme: TestRsaScheme,
    ) -> Result<String, TestKeyError> {
        let h = serde_json::to_vec(header).map_err(|_| TestKeyError)?;
        let c = serde_json::to_vec(claims).map_err(|_| TestKeyError)?;
        let input = format!("{}.{}", b64(&h), b64(&c));
        let sig = self
            .key
            .sign(scheme, input.as_bytes())
            .map_err(|_| TestKeyError)?;
        Ok(format!("{input}.{}", b64(&sig)))
    }
}

/// Token tanpa tanda tangan dengan header bebas (misalnya `alg: none`).
pub fn unsigned_token(header: &Value, claims: &Value) -> String {
    let h = serde_json::to_vec(header).unwrap_or_default();
    let c = serde_json::to_vec(claims).unwrap_or_default();
    format!("{}.{}.", b64(&h), b64(&c))
}

/// Dokumen JWKS `{ "keys": [...] }`.
pub fn jwks_document(keys: &[Value]) -> Vec<u8> {
    serde_json::to_vec(&json!({ "keys": keys })).unwrap_or_default()
}

/// Sumber JWKS dalam memori yang dapat diganti isinya atau dibuat gagal, dan menghitung pengambilan.
#[derive(Debug)]
pub struct StaticJwks {
    response: Mutex<Result<Vec<u8>, JwksFetchError>>,
    fetches: AtomicUsize,
}

impl StaticJwks {
    /// Sumber yang mengembalikan `doc`.
    pub fn new(doc: Vec<u8>) -> Self {
        Self {
            response: Mutex::new(Ok(doc)),
            fetches: AtomicUsize::new(0),
        }
    }

    /// Sumber yang selalu gagal.
    pub fn failing(error: JwksFetchError) -> Self {
        Self {
            response: Mutex::new(Err(error)),
            fetches: AtomicUsize::new(0),
        }
    }

    /// Mengganti respons berikutnya.
    pub fn set(&self, response: Result<Vec<u8>, JwksFetchError>) {
        *self.response.lock().unwrap_or_else(|e| e.into_inner()) = response;
    }

    /// Jumlah pengambilan sejauh ini.
    pub fn fetches(&self) -> usize {
        self.fetches.load(Ordering::SeqCst)
    }
}

impl JwksSource for StaticJwks {
    fn fetch(&self) -> JwksFuture<'_> {
        self.fetches.fetch_add(1, Ordering::SeqCst);
        let r = self
            .response
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone();
        Box::pin(async move { r })
    }
}
