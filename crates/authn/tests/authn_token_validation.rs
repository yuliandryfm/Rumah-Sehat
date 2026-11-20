//! Test validasi token (F1-T32, T1): skenario positif dan negatif coding-standards §7.3 butir 2 dan
//! ADR-011 butir 8 dan 11, dengan kunci uji yang dibangkitkan saat test (tanpa Keycloak nyata).

#![allow(clippy::unwrap_used, clippy::expect_used)] // berkas test

use authn::testing::{Scheme, StaticJwks, TestRealmKey, jwks_document, unsigned_token};
use authn::{
    Algorithm, AuthError, Authenticator, AuthnConfig, AuthnConfigError, Backend, JwksFetchError,
    Realm, RealmConfig,
};
use envelope::CommonError;
use foundation::{Clock, FixedClock};
use jsonwebtoken::{EncodingKey, Header};
use serde_json::{Value, json};
use std::sync::Arc;
use time::macros::datetime;
use time::{Duration, OffsetDateTime};

const PASIEN_ISS: &str = "https://auth.ruang-sehat.invalid/realms/pasien";
const NAKES_ISS: &str = "https://auth.ruang-sehat.invalid/realms/nakes";
const RME_ISS: &str = "https://auth.ruang-sehat.invalid/realms/rme";
const INTEGRASI_ISS: &str = "https://auth.ruang-sehat.invalid/realms/integrasi";
const PASIEN_AUD: &str = "rs-api-pasien";
const NAKES_AUD: &str = "rs-api-nakes";
const NOW: OffsetDateTime = datetime!(2027-03-01 08:00:00 UTC);

struct Fixture {
    auth: Authenticator,
    clock: Arc<FixedClock>,
    pasien: TestRealmKey,
    nakes: TestRealmKey,
    rme: TestRealmKey,
    integrasi: TestRealmKey,
    pasien_jwks: Arc<StaticJwks>,
}

fn realm(
    realm: Realm,
    issuer: &str,
    aud: &str,
    jwks: Arc<StaticJwks>,
    azp: &[&str],
) -> RealmConfig {
    RealmConfig {
        realm,
        issuer: issuer.into(),
        audiences: vec![aud.into()],
        algorithms: vec![Algorithm::RS256],
        authorized_parties: azp.iter().map(|s| s.to_string()).collect(),
        jwks,
    }
}

fn fixture() -> Fixture {
    let pasien = TestRealmKey::generate("pasien-k1").unwrap();
    let nakes = TestRealmKey::generate("nakes-k1").unwrap();
    let rme = TestRealmKey::generate("rme-k1").unwrap();
    let integrasi = TestRealmKey::generate("integrasi-k1").unwrap();
    let pasien_jwks = Arc::new(StaticJwks::new(jwks_document(&[pasien.jwk()])));
    let nakes_jwks = Arc::new(StaticJwks::new(jwks_document(&[nakes.jwk()])));
    let integrasi_jwks = Arc::new(StaticJwks::new(jwks_document(&[integrasi.jwk()])));
    let clock = Arc::new(FixedClock::new(NOW));
    let cfg = AuthnConfig::with_defaults(
        Backend::RuangSehat,
        vec![
            realm(
                Realm::Pasien,
                PASIEN_ISS,
                PASIEN_AUD,
                pasien_jwks.clone(),
                &[],
            ),
            realm(
                Realm::Nakes,
                NAKES_ISS,
                NAKES_AUD,
                nakes_jwks,
                &["app-nakes"],
            ),
            realm(
                Realm::Integrasi,
                INTEGRASI_ISS,
                "rs-partner-api",
                integrasi_jwks,
                &[],
            ),
        ],
    );
    let auth = Authenticator::new(cfg, clock.clone()).unwrap();
    Fixture {
        auth,
        clock,
        pasien,
        nakes,
        rme,
        integrasi,
        pasien_jwks,
    }
}

fn claims(iss: &str, aud: Value) -> Value {
    let now = NOW.unix_timestamp();
    json!({
        "iss": iss,
        "aud": aud,
        "sub": "6f1c1d1e-0000-4000-8000-000000000001",
        "typ": "Bearer",
        "azp": "app-pasien",
        "sid": "sesi-uji-1",
        "iat": now - 10,
        "exp": now + 300,
        "auth_time": now - 20,
    })
}

fn pasien_claims() -> Value {
    claims(PASIEN_ISS, json!(PASIEN_AUD))
}

async fn verify(f: &Fixture, token: &str) -> Result<authn::AuthContext, AuthError> {
    f.auth.verify(token).await
}

#[tokio::test]
async fn valid_pasien_token_yields_context() {
    let f = fixture();
    let ctx = verify(&f, &f.pasien.sign(&pasien_claims()).unwrap())
        .await
        .unwrap();
    assert_eq!(ctx.realm(), Realm::Pasien);
    assert_eq!(ctx.subject(), "6f1c1d1e-0000-4000-8000-000000000001");
    assert_eq!(ctx.session_id(), Some("sesi-uji-1"));
    assert_eq!(ctx.client_id(), Some("app-pasien"));
    assert_eq!(ctx.expires_at(), NOW + Duration::seconds(300));
    assert_eq!(ctx.auth_time(), Some(NOW - Duration::seconds(20)));
    // Audience berupa larik yang memuat audience API juga diterima.
    let mut c = pasien_claims();
    c["aud"] = json!(["account", PASIEN_AUD]);
    assert!(verify(&f, &f.pasien.sign(&c).unwrap()).await.is_ok());
}

#[tokio::test]
async fn token_from_rme_realm_is_rejected_by_rs_backend() {
    // ADR-011 butir 11: issuer realm `rme` tidak dikonfigurasi di backend Ruang Sehat.
    let f = fixture();
    let t = f.rme.sign(&claims(RME_ISS, json!("rme-api"))).unwrap();
    assert_eq!(verify(&f, &t).await.unwrap_err(), AuthError::UnknownIssuer);
}

#[tokio::test]
async fn token_signed_by_other_realm_key_is_rejected() {
    let f = fixture();
    // iss = pasien, tetapi ditandatangani kunci realm nakes dengan kid nakes: kid tidak ada di JWKS pasien.
    let forged = f.nakes.sign(&pasien_claims()).unwrap();
    assert_eq!(
        verify(&f, &forged).await.unwrap_err(),
        AuthError::UnknownKey
    );
    // Kunci nakes yang memakai kid pasien: tanda tangan tidak cocok dengan kunci pasien (RFC 8725 §3.8).
    let header = json!({"alg": "RS256", "typ": "JWT", "kid": f.pasien.kid()});
    let forged = f
        .nakes
        .sign_raw(&header, &pasien_claims(), Scheme::Rs256)
        .unwrap();
    assert_eq!(
        verify(&f, &forged).await.unwrap_err(),
        AuthError::InvalidSignature
    );
}

#[tokio::test]
async fn token_of_other_realm_audience_is_rejected() {
    let f = fixture();
    // Token realm pasien yang ditujukan ke API nakes.
    let t = f
        .pasien
        .sign(&claims(PASIEN_ISS, json!(NAKES_AUD)))
        .unwrap();
    assert_eq!(verify(&f, &t).await.unwrap_err(), AuthError::WrongAudience);
}

#[tokio::test]
async fn wrong_or_missing_audience_is_rejected() {
    let f = fixture();
    for aud in [json!("account"), json!([]), json!(["account", "lain"])] {
        let t = f.pasien.sign(&claims(PASIEN_ISS, aud.clone())).unwrap();
        assert_eq!(
            verify(&f, &t).await.unwrap_err(),
            AuthError::WrongAudience,
            "{aud}"
        );
    }
    let mut c = pasien_claims();
    c.as_object_mut().unwrap().remove("aud");
    let t = f.pasien.sign(&c).unwrap();
    assert_eq!(verify(&f, &t).await.unwrap_err(), AuthError::WrongAudience);
}

#[tokio::test]
async fn wrong_issuer_is_rejected() {
    let f = fixture();
    for iss in [
        "https://auth.ruang-sehat.invalid/realms/pasien/",
        "https://evil.invalid/realms/pasien",
        "http://auth.ruang-sehat.invalid/realms/pasien",
    ] {
        let t = f.pasien.sign(&claims(iss, json!(PASIEN_AUD))).unwrap();
        assert_eq!(
            verify(&f, &t).await.unwrap_err(),
            AuthError::UnknownIssuer,
            "{iss}"
        );
    }
    let mut c = pasien_claims();
    c["iss"] = json!([PASIEN_ISS]);
    let t = f.pasien.sign(&c).unwrap();
    assert_eq!(verify(&f, &t).await.unwrap_err(), AuthError::MalformedToken);
}

#[tokio::test]
async fn expired_and_not_yet_valid_tokens_are_rejected_with_leeway() {
    let f = fixture();
    let t = f.pasien.sign(&pasien_claims()).unwrap();
    // exp = NOW+300; toleransi 30 detik.
    f.clock.set(NOW + Duration::seconds(329));
    assert!(verify(&f, &t).await.is_ok());
    f.clock.set(NOW + Duration::seconds(330));
    assert_eq!(verify(&f, &t).await.unwrap_err(), AuthError::Expired);
    f.clock.set(NOW);

    let mut c = pasien_claims();
    c["nbf"] = json!(NOW.unix_timestamp() + 31);
    assert_eq!(
        verify(&f, &f.pasien.sign(&c).unwrap()).await.unwrap_err(),
        AuthError::NotYetValid
    );
    let mut c = pasien_claims();
    c["iat"] = json!(NOW.unix_timestamp() + 31);
    assert_eq!(
        verify(&f, &f.pasien.sign(&c).unwrap()).await.unwrap_err(),
        AuthError::NotYetValid
    );
    let mut c = pasien_claims();
    c.as_object_mut().unwrap().remove("exp");
    assert_eq!(
        verify(&f, &f.pasien.sign(&c).unwrap()).await.unwrap_err(),
        AuthError::MalformedToken
    );
    let mut c = pasien_claims();
    c["exp"] = json!("besok");
    assert_eq!(
        verify(&f, &f.pasien.sign(&c).unwrap()).await.unwrap_err(),
        AuthError::MalformedToken
    );
}

#[tokio::test]
async fn alg_none_is_rejected() {
    let f = fixture();
    for alg in ["none", "None", "NONE"] {
        let t = unsigned_token(
            &json!({"alg": alg, "typ": "JWT", "kid": f.pasien.kid()}),
            &pasien_claims(),
        );
        assert_eq!(
            verify(&f, &t).await.unwrap_err(),
            AuthError::AlgorithmNotAllowed,
            "{alg}"
        );
    }
}

#[tokio::test]
async fn hs256_signed_with_public_key_is_rejected() {
    // Serangan kebingungan algoritma: kunci publik RSA (modulus mentah, atau JWK-nya) dipakai sebagai
    // rahasia HMAC.
    let f = fixture();
    let jwk_bytes = serde_json::to_vec(&f.pasien.jwk()).unwrap();
    for secret in [f.pasien.public_modulus(), jwk_bytes] {
        let mut header = Header::new(jsonwebtoken::Algorithm::HS256);
        header.kid = Some(f.pasien.kid().to_string());
        let t = jsonwebtoken::encode(
            &header,
            &pasien_claims(),
            &EncodingKey::from_secret(&secret).unwrap(),
        )
        .unwrap();
        assert_eq!(
            verify(&f, &t).await.unwrap_err(),
            AuthError::AlgorithmNotAllowed
        );
    }
}

#[tokio::test]
async fn rsa_algorithm_outside_allowlist_is_rejected() {
    let f = fixture();
    for (alg, scheme) in [("RS384", Scheme::Rs384), ("PS256", Scheme::Ps256)] {
        let header = json!({"alg": alg, "typ": "JWT", "kid": f.pasien.kid()});
        let t = f
            .pasien
            .sign_raw(&header, &pasien_claims(), scheme)
            .unwrap();
        assert_eq!(
            verify(&f, &t).await.unwrap_err(),
            AuthError::AlgorithmNotAllowed,
            "{alg}"
        );
    }
}

#[tokio::test]
async fn header_alg_must_match_key_alg() {
    // Daftar izin realm RS256 + PS256, kunci di JWKS terikat RS256: token PS256 dengan kid itu ditolak.
    let key = TestRealmKey::generate("k").unwrap();
    let jwks = Arc::new(StaticJwks::new(jwks_document(&[key.jwk()])));
    let mut rc = realm(Realm::Pasien, PASIEN_ISS, PASIEN_AUD, jwks, &[]);
    rc.algorithms = vec![Algorithm::RS256, Algorithm::PS256];
    let auth = Authenticator::new(
        AuthnConfig::with_defaults(Backend::RuangSehat, vec![rc]),
        Arc::new(FixedClock::new(NOW)),
    )
    .unwrap();
    let header = json!({"alg": "PS256", "typ": "JWT", "kid": "k"});
    let t = key
        .sign_raw(&header, &pasien_claims(), Scheme::Ps256)
        .unwrap();
    assert_eq!(
        auth.verify(&t).await.unwrap_err(),
        AuthError::AlgorithmNotAllowed
    );
    assert!(
        auth.verify(&key.sign(&pasien_claims()).unwrap())
            .await
            .is_ok()
    );
}

#[tokio::test]
async fn unknown_or_missing_kid_is_rejected_and_refresh_is_rate_limited() {
    let f = fixture();
    assert!(
        verify(&f, &f.pasien.sign(&pasien_claims()).unwrap())
            .await
            .is_ok()
    );
    assert_eq!(f.pasien_jwks.fetches(), 1);

    let header = json!({"alg": "RS256", "typ": "JWT", "kid": "tidak-dikenal"});
    let t = f
        .pasien
        .sign_raw(&header, &pasien_claims(), Scheme::Rs256)
        .unwrap();
    // Tepat setelah pengambilan, kid baru tidak memicu pengambilan ulang (interval minimum 10 detik).
    assert_eq!(verify(&f, &t).await.unwrap_err(), AuthError::UnknownKey);
    assert_eq!(f.pasien_jwks.fetches(), 1);
    // Setelah interval, kid baru memicu satu pengambilan ulang (rotasi kunci), lalu ditolak.
    f.clock.advance(Duration::seconds(11));
    assert_eq!(verify(&f, &t).await.unwrap_err(), AuthError::UnknownKey);
    assert_eq!(f.pasien_jwks.fetches(), 2);
    // Dalam interval minimum, kid acak tidak memicu pengambilan lagi.
    for _ in 0..5 {
        assert_eq!(verify(&f, &t).await.unwrap_err(), AuthError::UnknownKey);
    }
    assert_eq!(f.pasien_jwks.fetches(), 2);

    let header = json!({"alg": "RS256", "typ": "JWT"});
    let t = f
        .pasien
        .sign_raw(&header, &pasien_claims(), Scheme::Rs256)
        .unwrap();
    assert_eq!(verify(&f, &t).await.unwrap_err(), AuthError::UnknownKey);
}

#[tokio::test]
async fn key_rotation_is_picked_up_after_min_interval() {
    let f = fixture();
    assert!(
        verify(&f, &f.pasien.sign(&pasien_claims()).unwrap())
            .await
            .is_ok()
    );
    let rotated = TestRealmKey::generate("pasien-k2").unwrap();
    f.pasien_jwks
        .set(Ok(jwks_document(&[f.pasien.jwk(), rotated.jwk()])));
    f.clock.advance(Duration::seconds(11));
    let ctx = verify(&f, &rotated.sign(&pasien_claims()).unwrap()).await;
    assert!(ctx.is_ok());
    assert_eq!(f.pasien_jwks.fetches(), 2);
}

#[tokio::test]
async fn jwks_unavailable_fails_closed() {
    let key = TestRealmKey::generate("k").unwrap();
    let jwks = Arc::new(StaticJwks::failing(JwksFetchError::Connect));
    let clock = Arc::new(FixedClock::new(NOW));
    let auth = Authenticator::new(
        AuthnConfig::with_defaults(
            Backend::RuangSehat,
            vec![realm(
                Realm::Pasien,
                PASIEN_ISS,
                PASIEN_AUD,
                jwks.clone(),
                &[],
            )],
        ),
        clock.clone(),
    )
    .unwrap();
    let t = key.sign(&pasien_claims()).unwrap();
    let e = auth.verify(&t).await.unwrap_err();
    assert_eq!(e, AuthError::KeysUnavailable);
    assert_eq!(e.to_common_error(), CommonError::Unavailable);
    // Kegagalan berulang tidak membanjiri Keycloak dalam interval minimum.
    assert_eq!(
        auth.verify(&t).await.unwrap_err(),
        AuthError::KeysUnavailable
    );
    assert_eq!(jwks.fetches(), 1);

    // Status non-200, isi rusak, dan JWKS tanpa kunci yang dapat dipakai juga gagal-tertutup.
    for resp in [
        Err(JwksFetchError::Status(500)),
        Err(JwksFetchError::Timeout),
        Ok(b"<html>".to_vec()),
        Ok(jwks_document(&[])),
        Ok(jwks_document(&[key.jwk_with_alg("RSA-OAEP")])),
    ] {
        jwks.set(resp);
        clock.advance(Duration::seconds(11));
        assert_eq!(
            auth.verify(&t).await.unwrap_err(),
            AuthError::KeysUnavailable
        );
    }

    // Pulih setelah JWKS kembali tersedia.
    jwks.set(Ok(jwks_document(&[key.jwk()])));
    clock.advance(Duration::seconds(11));
    let fresh = clock.now();
    let mut c = pasien_claims();
    c["iat"] = json!(fresh.unix_timestamp());
    c["exp"] = json!(fresh.unix_timestamp() + 300);
    assert!(auth.verify(&key.sign(&c).unwrap()).await.is_ok());
}

#[tokio::test]
async fn expired_cache_is_not_used_when_refresh_fails() {
    let f = fixture();
    assert!(
        verify(&f, &f.pasien.sign(&pasien_claims()).unwrap())
            .await
            .is_ok()
    );
    f.pasien_jwks.set(Err(JwksFetchError::Connect));
    // TTL bawaan 5 menit lewat: kunci lama tidak dipakai lagi.
    f.clock.set(NOW + Duration::minutes(5));
    let now = f.clock.now().unix_timestamp();
    let mut c = pasien_claims();
    c["iat"] = json!(now);
    c["exp"] = json!(now + 300);
    assert_eq!(
        verify(&f, &f.pasien.sign(&c).unwrap()).await.unwrap_err(),
        AuthError::KeysUnavailable
    );
}

#[tokio::test]
async fn tampered_payload_fails_signature() {
    let f = fixture();
    let t = f.pasien.sign(&pasien_claims()).unwrap();
    use base64::Engine;
    let mut parts: Vec<String> = t.split('.').map(str::to_string).collect();
    let mut c = pasien_claims();
    c["sub"] = json!("6f1c1d1e-0000-4000-8000-000000000999");
    parts[1] =
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(serde_json::to_vec(&c).unwrap());
    let forged = parts.join(".");
    assert_eq!(
        verify(&f, &forged).await.unwrap_err(),
        AuthError::InvalidSignature
    );
}

#[tokio::test]
async fn non_access_token_types_are_rejected() {
    // Keycloak: token akses `typ: Bearer`, ID token `ID`, refresh token `Refresh` (SRC-531).
    let f = fixture();
    for typ in [json!("ID"), json!("Refresh"), json!("bearer"), Value::Null] {
        let mut c = pasien_claims();
        if typ.is_null() {
            c.as_object_mut().unwrap().remove("typ");
        } else {
            c["typ"] = typ.clone();
        }
        assert_eq!(
            verify(&f, &f.pasien.sign(&c).unwrap()).await.unwrap_err(),
            AuthError::WrongTokenType,
            "{typ}"
        );
    }
}

#[tokio::test]
async fn human_realm_requires_session_machine_realm_does_not() {
    let f = fixture();
    for sid in [Value::Null, json!("")] {
        let mut c = pasien_claims();
        if sid.is_null() {
            c.as_object_mut().unwrap().remove("sid");
        } else {
            c["sid"] = sid;
        }
        assert_eq!(
            verify(&f, &f.pasien.sign(&c).unwrap()).await.unwrap_err(),
            AuthError::MissingSession
        );
    }
    let mut c = claims(INTEGRASI_ISS, json!("rs-partner-api"));
    c.as_object_mut().unwrap().remove("sid");
    let ctx = verify(&f, &f.integrasi.sign(&c).unwrap()).await.unwrap();
    assert_eq!(ctx.realm(), Realm::Integrasi);
    assert_eq!(ctx.session_id(), None);
}

#[tokio::test]
async fn missing_subject_is_rejected() {
    let f = fixture();
    let mut c = pasien_claims();
    c.as_object_mut().unwrap().remove("sub");
    assert_eq!(
        verify(&f, &f.pasien.sign(&c).unwrap()).await.unwrap_err(),
        AuthError::MalformedToken
    );
    let mut c = pasien_claims();
    c["sub"] = json!("");
    assert_eq!(
        verify(&f, &f.pasien.sign(&c).unwrap()).await.unwrap_err(),
        AuthError::MalformedToken
    );
}

#[tokio::test]
async fn client_outside_authorized_parties_is_rejected() {
    // Realm nakes dibatasi klien `app-nakes`.
    let f = fixture();
    let mut c = claims(NAKES_ISS, json!(NAKES_AUD));
    c["azp"] = json!("app-pasien");
    assert_eq!(
        verify(&f, &f.nakes.sign(&c).unwrap()).await.unwrap_err(),
        AuthError::UnauthorizedParty
    );
    c.as_object_mut().unwrap().remove("azp");
    assert_eq!(
        verify(&f, &f.nakes.sign(&c).unwrap()).await.unwrap_err(),
        AuthError::UnauthorizedParty
    );
    c["azp"] = json!("app-nakes");
    assert_eq!(
        verify(&f, &f.nakes.sign(&c).unwrap())
            .await
            .unwrap()
            .realm(),
        Realm::Nakes
    );
}

#[tokio::test]
async fn malformed_tokens_are_rejected() {
    let f = fixture();
    let long = "a".repeat(authn::MAX_TOKEN_LEN + 1);
    for t in ["", "abc", "a.b", "a.b.c.d", "!!.??.**", long.as_str()] {
        assert_eq!(
            verify(&f, t).await.unwrap_err(),
            AuthError::MalformedToken,
            "{t}"
        );
    }
}

fn base_cfg(backend: Backend, realms: Vec<RealmConfig>) -> AuthnConfig {
    AuthnConfig::with_defaults(backend, realms)
}

fn jwks() -> Arc<StaticJwks> {
    Arc::new(StaticJwks::new(jwks_document(&[])))
}

fn build(cfg: AuthnConfig) -> Result<Authenticator, AuthnConfigError> {
    Authenticator::new(cfg, Arc::new(FixedClock::new(NOW)))
}

#[test]
fn configuration_enforces_realm_separation_and_allowlist() {
    // ADR-011 butir 11 di tingkat konfigurasi.
    assert_eq!(
        build(base_cfg(
            Backend::RuangSehat,
            vec![realm(Realm::Rme, RME_ISS, "rme-api", jwks(), &[])]
        ))
        .unwrap_err(),
        AuthnConfigError::RealmNotAccepted("rme")
    );
    assert_eq!(
        build(base_cfg(
            Backend::Rme,
            vec![realm(Realm::Pasien, PASIEN_ISS, PASIEN_AUD, jwks(), &[])]
        ))
        .unwrap_err(),
        AuthnConfigError::RealmNotAccepted("pasien")
    );
    assert!(
        build(base_cfg(
            Backend::Rme,
            vec![realm(Realm::Rme, RME_ISS, "rme-api", jwks(), &[])]
        ))
        .is_ok()
    );

    for (algs, expected) in [
        (
            vec![Algorithm::HS256],
            AuthnConfigError::AlgorithmNotAllowed("pasien"),
        ),
        (
            vec![Algorithm::RS256, Algorithm::HS512],
            AuthnConfigError::AlgorithmNotAllowed("pasien"),
        ),
        (
            vec![Algorithm::EdDSA],
            AuthnConfigError::AlgorithmNotAllowed("pasien"),
        ),
        (vec![], AuthnConfigError::NoAlgorithm("pasien")),
    ] {
        let mut rc = realm(Realm::Pasien, PASIEN_ISS, PASIEN_AUD, jwks(), &[]);
        rc.algorithms = algs;
        assert_eq!(
            build(base_cfg(Backend::RuangSehat, vec![rc])).unwrap_err(),
            expected
        );
    }

    let mut rc = realm(Realm::Pasien, PASIEN_ISS, PASIEN_AUD, jwks(), &[]);
    rc.audiences = vec![];
    assert_eq!(
        build(base_cfg(Backend::RuangSehat, vec![rc])).unwrap_err(),
        AuthnConfigError::NoAudience("pasien")
    );
    let rc = realm(Realm::Pasien, "pasien", PASIEN_AUD, jwks(), &[]);
    assert_eq!(
        build(base_cfg(Backend::RuangSehat, vec![rc])).unwrap_err(),
        AuthnConfigError::InvalidIssuer("pasien")
    );
    assert_eq!(
        build(base_cfg(
            Backend::RuangSehat,
            vec![
                realm(Realm::Pasien, PASIEN_ISS, PASIEN_AUD, jwks(), &[]),
                realm(Realm::Nakes, PASIEN_ISS, NAKES_AUD, jwks(), &[]),
            ]
        ))
        .unwrap_err(),
        AuthnConfigError::DuplicateIssuer("nakes")
    );
    assert_eq!(
        build(base_cfg(
            Backend::RuangSehat,
            vec![
                realm(Realm::Pasien, PASIEN_ISS, PASIEN_AUD, jwks(), &[]),
                realm(Realm::Pasien, NAKES_ISS, NAKES_AUD, jwks(), &[]),
            ]
        ))
        .unwrap_err(),
        AuthnConfigError::DuplicateRealm("pasien")
    );
    assert_eq!(
        build(base_cfg(Backend::RuangSehat, vec![])).unwrap_err(),
        AuthnConfigError::NoRealms
    );

    let mut cfg = base_cfg(
        Backend::RuangSehat,
        vec![realm(Realm::Pasien, PASIEN_ISS, PASIEN_AUD, jwks(), &[])],
    );
    cfg.leeway = Duration::minutes(5);
    assert_eq!(
        build(cfg).unwrap_err(),
        AuthnConfigError::InvalidDuration("leeway")
    );
    let mut cfg = base_cfg(
        Backend::RuangSehat,
        vec![realm(Realm::Pasien, PASIEN_ISS, PASIEN_AUD, jwks(), &[])],
    );
    cfg.jwks_ttl = Duration::ZERO;
    assert_eq!(
        build(cfg).unwrap_err(),
        AuthnConfigError::InvalidDuration("jwks_ttl")
    );
}
