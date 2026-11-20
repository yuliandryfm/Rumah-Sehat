//! Validasi token akses (ADR-011 butir 8, RFC 8725 [SRC-79], coding-standards §5.4).
//!
//! Urutan per token:
//! 1. Ukuran dan bentuk JWS compact; header diurai untuk `alg` dan `kid` saja.
//! 2. `iss` dibaca (belum tepercaya) **hanya** untuk memilih realm di antara realm yang dikonfigurasi;
//!    issuer yang tidak dikenal (termasuk realm milik backend lain) langsung ditolak.
//! 3. `alg` harus ada di daftar izin realm (RFC 8725 §3.1); `none` dan HMAC tidak pernah ada di daftar.
//! 4. Kunci dicari per `kid` di JWKS realm itu saja (RFC 8725 §3.8), dan `alg` header harus sama dengan
//!    `alg` kunci.
//! 5. `jsonwebtoken::decode` memverifikasi tanda tangan, `iss` (sama persis), dan `aud` (wajib ada dan
//!    termasuk audience API, RFC 8725 §3.9). Fungsi pustaka diverifikasi ke source 12.0.0 (SRC-530).
//! 6. Waktu (`exp`, `nbf`, `iat`) diperiksa di sini dengan [`foundation::Clock`] (KEPUTUSAN-131),
//!    bukan jam sistem di dalam pustaka; `typ` harus `Bearer` (SRC-531); `sid` wajib untuk realm akun
//!    manusia; `azp` dicocokkan bila realm membatasi klien.

use crate::context::AuthContext;
use crate::error::{AuthError, AuthnConfigError};
use crate::jwks::{CachePolicy, JwksCache, JwksSource, LookupError};
use crate::realm::{Backend, Realm};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use foundation::Clock;
use http::HeaderValue;
use jsonwebtoken::errors::ErrorKind;
use jsonwebtoken::{Algorithm, AlgorithmFamily, Validation};
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;
use std::sync::Arc;
use time::{Duration, OffsetDateTime};

/// Batas panjang token yang diproses (byte). Token Keycloak jauh lebih kecil; batas ini mencegah
/// pekerjaan dekode besar dari masukan penyerang.
pub const MAX_TOKEN_LEN: usize = 8 * 1024;

/// Nilai klaim `typ` token akses Keycloak (SRC-531).
const ACCESS_TOKEN_TYPE: &str = "Bearer";

/// Konfigurasi satu realm yang diterima backend.
pub struct RealmConfig {
    /// Realm.
    pub realm: Realm,
    /// Nilai `iss` yang diharapkan, sama persis (misalnya `https://<host>/realms/pasien`).
    pub issuer: String,
    /// Audience API ini; token harus memuat minimal satu di klaim `aud`.
    pub audiences: Vec<String>,
    /// Daftar izin algoritma (hanya keluarga RSA dan EC).
    pub algorithms: Vec<Algorithm>,
    /// Klien (`azp`) yang boleh memakai API ini. Kosong = tidak dibatasi (audience tetap wajib).
    pub authorized_parties: Vec<String>,
    /// Sumber JWKS realm. Alamatnya dikonfigurasi, tidak diturunkan dari token.
    pub jwks: Arc<dyn JwksSource>,
}

impl fmt::Debug for RealmConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RealmConfig")
            .field("realm", &self.realm)
            .field("issuer", &self.issuer)
            .field("audiences", &self.audiences)
            .field("algorithms", &self.algorithms)
            .field("authorized_parties", &self.authorized_parties)
            .finish_non_exhaustive()
    }
}

/// Konfigurasi autentikasi satu backend.
#[derive(Debug)]
pub struct AuthnConfig {
    /// Backend pemakai; menentukan realm yang boleh dikonfigurasi (ADR-011 butir 11).
    pub backend: Backend,
    /// Realm yang diterima.
    pub realms: Vec<RealmConfig>,
    /// Masa simpan JWKS di cache. Setelah lewat, JWKS wajib diambil ulang.
    pub jwks_ttl: Duration,
    /// Jeda minimum antar-pengambilan JWKS (`kid` tidak dikenal atau pengambilan gagal).
    pub jwks_min_refresh_interval: Duration,
    /// Toleransi selisih jam untuk `exp`, `nbf`, dan `iat`.
    pub leeway: Duration,
}

impl AuthnConfig {
    /// Nilai awal teknis: TTL JWKS 5 menit, jeda ambil ulang 10 detik, toleransi jam 30 detik.
    /// Bukan nilai yang diatur dokumen; dapat diubah lewat konfigurasi.
    pub fn with_defaults(backend: Backend, realms: Vec<RealmConfig>) -> Self {
        Self {
            backend,
            realms,
            jwks_ttl: Duration::minutes(5),
            jwks_min_refresh_interval: Duration::seconds(10),
            leeway: Duration::seconds(30),
        }
    }
}

struct RealmVerifier {
    realm: Realm,
    issuer: String,
    audiences: Vec<String>,
    algorithms: Vec<Algorithm>,
    authorized_parties: Vec<String>,
    cache: JwksCache,
}

/// Validator token untuk satu backend. Murah dibagi lewat `Arc`.
pub struct Authenticator {
    by_issuer: HashMap<String, RealmVerifier>,
    leeway: Duration,
    clock: Arc<dyn Clock>,
}

impl fmt::Debug for Authenticator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut realms: Vec<&str> = self.by_issuer.values().map(|v| v.realm.as_str()).collect();
        realms.sort_unstable();
        f.debug_struct("Authenticator")
            .field("realms", &realms)
            .finish_non_exhaustive()
    }
}

fn validate_issuer(issuer: &str) -> bool {
    let rest = issuer
        .strip_prefix("https://")
        .or_else(|| issuer.strip_prefix("http://"));
    rest.is_some_and(|r| !r.is_empty() && !r.contains(char::is_whitespace))
}

#[derive(Deserialize)]
struct RawHeader {
    alg: String,
    kid: Option<String>,
}

#[derive(Deserialize)]
struct IssuerOnly {
    iss: String,
}

/// Klaim yang dibaca dari token akses. `aud` divalidasi pustaka; tipe numerik yang salah membuat
/// dekode gagal (token ditolak).
#[derive(Deserialize)]
struct AccessClaims {
    sub: String,
    exp: i64,
    nbf: Option<i64>,
    iat: Option<i64>,
    typ: Option<String>,
    azp: Option<String>,
    sid: Option<String>,
    auth_time: Option<i64>,
}

fn b64_json<T: for<'de> Deserialize<'de>>(segment: &str) -> Option<T> {
    let bytes = URL_SAFE_NO_PAD.decode(segment).ok()?;
    serde_json::from_slice(&bytes).ok()
}

fn ts(seconds: i64) -> Result<OffsetDateTime, AuthError> {
    OffsetDateTime::from_unix_timestamp(seconds).map_err(|_| AuthError::MalformedToken)
}

/// Mengambil token dari header `Authorization: Bearer <token>` (RFC 6750 §2.1). Skema tidak peka huruf
/// besar-kecil; isi harus berbentuk `b64token`.
pub fn bearer_token<'a>(
    mut values: impl Iterator<Item = &'a HeaderValue>,
) -> Result<&'a str, AuthError> {
    let value = values.next().ok_or(AuthError::MissingCredentials)?;
    if values.next().is_some() {
        return Err(AuthError::MalformedHeader);
    }
    let text = value.to_str().map_err(|_| AuthError::MalformedHeader)?;
    let (scheme, token) = text.split_once(' ').ok_or(AuthError::MalformedHeader)?;
    if !scheme.eq_ignore_ascii_case("bearer") || token.is_empty() || token.len() > MAX_TOKEN_LEN {
        return Err(AuthError::MalformedHeader);
    }
    let b64token = token
        .trim_end_matches('=')
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_' | b'~' | b'+' | b'/'));
    if !b64token {
        return Err(AuthError::MalformedHeader);
    }
    Ok(token)
}

impl Authenticator {
    /// Memvalidasi konfigurasi dan membangun validator. Konfigurasi tidak valid = start gagal.
    pub fn new(config: AuthnConfig, clock: Arc<dyn Clock>) -> Result<Self, AuthnConfigError> {
        if config.realms.is_empty() {
            return Err(AuthnConfigError::NoRealms);
        }
        let positive = |d: Duration| d > Duration::ZERO;
        if !positive(config.jwks_ttl) || config.jwks_ttl > Duration::hours(24) {
            return Err(AuthnConfigError::InvalidDuration("jwks_ttl"));
        }
        if !positive(config.jwks_min_refresh_interval)
            || config.jwks_min_refresh_interval > config.jwks_ttl
        {
            return Err(AuthnConfigError::InvalidDuration(
                "jwks_min_refresh_interval",
            ));
        }
        if config.leeway < Duration::ZERO || config.leeway > Duration::minutes(2) {
            return Err(AuthnConfigError::InvalidDuration("leeway"));
        }
        let policy = CachePolicy {
            ttl: config.jwks_ttl,
            min_refresh_interval: config.jwks_min_refresh_interval,
        };
        let mut by_issuer: HashMap<String, RealmVerifier> = HashMap::new();
        let mut seen: Vec<Realm> = Vec::new();
        for rc in config.realms {
            let name = rc.realm.as_str();
            if !config.backend.accepts(rc.realm) {
                return Err(AuthnConfigError::RealmNotAccepted(name));
            }
            if seen.contains(&rc.realm) {
                return Err(AuthnConfigError::DuplicateRealm(name));
            }
            seen.push(rc.realm);
            if !validate_issuer(&rc.issuer) {
                return Err(AuthnConfigError::InvalidIssuer(name));
            }
            if rc.audiences.is_empty() || rc.audiences.iter().any(String::is_empty) {
                return Err(AuthnConfigError::NoAudience(name));
            }
            if rc.algorithms.is_empty() {
                return Err(AuthnConfigError::NoAlgorithm(name));
            }
            if rc
                .algorithms
                .iter()
                .any(|a| !matches!(a.family(), AlgorithmFamily::Rsa | AlgorithmFamily::Ec))
            {
                return Err(AuthnConfigError::AlgorithmNotAllowed(name));
            }
            if by_issuer.contains_key(&rc.issuer) {
                return Err(AuthnConfigError::DuplicateIssuer(name));
            }
            let cache = JwksCache::new(rc.jwks, rc.algorithms.clone(), policy);
            by_issuer.insert(
                rc.issuer.clone(),
                RealmVerifier {
                    realm: rc.realm,
                    issuer: rc.issuer,
                    audiences: rc.audiences,
                    algorithms: rc.algorithms,
                    authorized_parties: rc.authorized_parties,
                    cache,
                },
            );
        }
        Ok(Self {
            by_issuer,
            leeway: config.leeway,
            clock,
        })
    }

    /// Realm yang diterima validator ini.
    pub fn realms(&self) -> Vec<Realm> {
        let mut r: Vec<Realm> = self.by_issuer.values().map(|v| v.realm).collect();
        r.sort_unstable();
        r
    }

    /// Memvalidasi header `Authorization` permintaan.
    pub async fn authenticate_headers<'a>(
        &self,
        authorization: impl Iterator<Item = &'a HeaderValue>,
    ) -> Result<AuthContext, AuthError> {
        let token = bearer_token(authorization)?;
        self.verify(token).await
    }

    /// Memvalidasi satu token akses.
    pub async fn verify(&self, token: &str) -> Result<AuthContext, AuthError> {
        if token.len() > MAX_TOKEN_LEN {
            return Err(AuthError::MalformedToken);
        }
        let mut parts = token.split('.');
        let (Some(header_b64), Some(payload_b64), Some(_sig), None) =
            (parts.next(), parts.next(), parts.next(), parts.next())
        else {
            return Err(AuthError::MalformedToken);
        };
        let header: RawHeader = b64_json(header_b64).ok_or(AuthError::MalformedToken)?;
        let unverified: IssuerOnly = b64_json(payload_b64).ok_or(AuthError::MalformedToken)?;
        let realm = self
            .by_issuer
            .get(&unverified.iss)
            .ok_or(AuthError::UnknownIssuer)?;

        let alg = Algorithm::from_str(&header.alg).map_err(|_| AuthError::AlgorithmNotAllowed)?;
        if !realm.algorithms.contains(&alg) {
            return Err(AuthError::AlgorithmNotAllowed);
        }
        let kid = header.kid.ok_or(AuthError::UnknownKey)?;
        let key = realm
            .cache
            .key(&kid, self.clock.as_ref())
            .await
            .map_err(|e| match e {
                LookupError::UnknownKey => AuthError::UnknownKey,
                LookupError::Unavailable => AuthError::KeysUnavailable,
            })?;
        if key.alg != alg {
            return Err(AuthError::AlgorithmNotAllowed);
        }

        let mut validation = Validation::new(key.alg);
        validation.set_issuer(&[realm.issuer.as_str()]);
        validation.set_audience(&realm.audiences);
        validation.set_required_spec_claims(&["exp", "iss", "aud", "sub"]);
        validation.validate_aud = true;
        // Waktu diperiksa di bawah dengan Clock (KEPUTUSAN-131), bukan jam sistem di pustaka.
        validation.validate_exp = false;
        validation.validate_nbf = false;
        let claims = jsonwebtoken::decode::<AccessClaims>(token, &key.key, &validation)
            .map_err(|e| match e.kind() {
                ErrorKind::InvalidSignature => AuthError::InvalidSignature,
                ErrorKind::InvalidIssuer => AuthError::WrongIssuer,
                ErrorKind::InvalidAudience => AuthError::WrongAudience,
                ErrorKind::InvalidAlgorithm
                | ErrorKind::InvalidAlgorithmName
                | ErrorKind::MissingAlgorithm => AuthError::AlgorithmNotAllowed,
                ErrorKind::MissingRequiredClaim(c) if c == "aud" => AuthError::WrongAudience,
                _ => AuthError::MalformedToken,
            })?
            .claims;

        self.check_claims(realm, &claims)?;
        Ok(AuthContext::new(
            realm.realm,
            claims.sub,
            claims.sid,
            claims.azp,
            claims.auth_time.map(ts).transpose()?,
            ts(claims.exp)?,
        ))
    }

    fn check_claims(&self, realm: &RealmVerifier, c: &AccessClaims) -> Result<(), AuthError> {
        if c.typ.as_deref() != Some(ACCESS_TOKEN_TYPE) {
            return Err(AuthError::WrongTokenType);
        }
        if c.sub.is_empty() {
            return Err(AuthError::MalformedToken);
        }
        let now = self.clock.now();
        let past = now.checked_sub(self.leeway).ok_or(AuthError::Expired)?;
        let future = now.checked_add(self.leeway).ok_or(AuthError::NotYetValid)?;
        if ts(c.exp)? <= past {
            return Err(AuthError::Expired);
        }
        for t in [c.nbf, c.iat].into_iter().flatten() {
            if ts(t)? > future {
                return Err(AuthError::NotYetValid);
            }
        }
        if realm.realm.is_human() && c.sid.as_deref().is_none_or(str::is_empty) {
            return Err(AuthError::MissingSession);
        }
        if !realm.authorized_parties.is_empty()
            && !c
                .azp
                .as_ref()
                .is_some_and(|azp| realm.authorized_parties.contains(azp))
        {
            return Err(AuthError::UnauthorizedParty);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hv(s: &str) -> HeaderValue {
        HeaderValue::from_str(s).unwrap()
    }

    #[test]
    fn bearer_header_parsing_rules() {
        let ok = hv("Bearer aaa.bbb.ccc");
        assert_eq!(bearer_token([&ok].into_iter()), Ok("aaa.bbb.ccc"));
        let lower = hv("bearer aaa.bbb.ccc");
        assert_eq!(bearer_token([&lower].into_iter()), Ok("aaa.bbb.ccc"));
        assert_eq!(
            bearer_token(std::iter::empty()),
            Err(AuthError::MissingCredentials)
        );
        for bad in [
            "Basic dXNlcjpwYXNz",
            "Bearer",
            "Bearer ",
            "Bearer a b",
            "Bearer a\"b",
            "Bearer  aaa",
            "Token aaa.bbb.ccc",
        ] {
            let v = hv(bad);
            assert_eq!(
                bearer_token([&v].into_iter()),
                Err(AuthError::MalformedHeader),
                "{bad}"
            );
        }
        let two = [hv("Bearer a.b.c"), hv("Bearer d.e.f")];
        assert_eq!(bearer_token(two.iter()), Err(AuthError::MalformedHeader));
        let long = hv(&format!("Bearer {}", "a".repeat(MAX_TOKEN_LEN + 1)));
        assert_eq!(
            bearer_token([&long].into_iter()),
            Err(AuthError::MalformedHeader)
        );
    }

    #[test]
    fn issuer_must_be_http_url() {
        assert!(validate_issuer("https://auth.example/realms/pasien"));
        assert!(validate_issuer("http://127.0.0.1:18080/realms/pasien"));
        assert!(!validate_issuer(""));
        assert!(!validate_issuer("https://"));
        assert!(!validate_issuer("pasien"));
        assert!(!validate_issuer("https://a b"));
    }
}
