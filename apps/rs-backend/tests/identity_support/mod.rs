//! Perangkat test integrasi verifikasi identitas (F1-T60–F1-T62, reference implementation US-106/US-201):
//! PostgreSQL (`scripts/db-ephemeral.sh`, role runtime `rs_api`), SeaweedFS dan Valkey efemeral (Docker),
//! KEK uji di memori (bukan OpenBao: Transit diuji crate `crypto`), kunci blind index acak dua versi, dan
//! `Principal` yang lahir dari jalur autentikasi/otorisasi produksi (`PolicyRouter` + validator token
//! sungguhan dengan kunci realm uji), sehingga service dipanggil dengan pemanggil yang sah.
//! Tanpa variabel database atau Docker, test GAGAL (tidak dilewati). Data sintetis saja.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    dead_code
)]

use crate::files::{self, SeaweedFs, app_pool, jpeg, now, random_hex, upload};
use crate::valkey_support::ValkeyContainer;
use audit_store::{PageCursor, StoredEvent, read_page};
use authn::testing::{StaticJwks, TestRealmKey, jwks_document};
use authn::{
    Algorithm, AuthContext, AuthState, Authenticator, AuthnConfig, Backend, ErrorRenderer, Realm,
    RealmConfig,
};
use authz::{
    AccountRecord, AccountStatus, AccountType, AdminSubRole, ClientIp, DenialSink, Enforcement,
    Policy, PolicyRouter, Principal, PrincipalResolver, ResolveError, ResolveFuture,
    RevocationFuture, SessionGuard,
};
use axum::extract::State;
use axum::http::header::CONTENT_TYPE;
use axum::response::{IntoResponse, Response};
use crypto::{
    CryptoError, GeneratedDek, KeyEncryptionKey, Keyring, KeyringConfig, NikBlindIndexKeyring,
    SecretBytes, System, secret_bytes,
};
use envelope::{ApiError, CommonError};
use foundation::{Clock, CorrelationId, Sensitive, SystemClock, Uuid, new_id};
use platform_jobs::lock::RS_AUDIT_RELAY;
use platform_jobs::{LockAttempt, SingletonLock};
use rs_backend::modules::platform::api::{
    FileStorage, UploadMethod, UploadPurpose, UploadSlotRequest, create_upload_slot,
};
use rs_backend::modules::profile::api::IdentityServices;
use serde_json::json;
use sqlx::{Connection, PgConnection, PgPool};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use utoipa_axum::routes;

pub const PATIENT_ROUTE: &str =
    "/patient/v1/care-recipients/{care_recipient_id}/identity-submissions";
pub const CLIENT_IP: IpAddr = IpAddr::V4(Ipv4Addr::new(198, 51, 100, 23));

// ----- KEK uji -----

/// KEK uji: DEK acak dari RNG pustaka, "terbungkus" berupa heksadesimal dengan awalan. Hanya untuk test
/// (bentuk produksi = OpenBao Transit, diuji di `crates/crypto/tests`).
pub struct TestKek;

impl KeyEncryptionKey for TestKek {
    async fn generate_dek(&self) -> Result<GeneratedDek, CryptoError> {
        let mut dek = vec![0u8; crypto::DEK_LEN];
        crypto::fill_random(&mut dek)?;
        let hex: String = dek.iter().map(|b| format!("{b:02x}")).collect();
        Ok(GeneratedDek {
            plaintext: secret_bytes(dek),
            wrapped: Sensitive::new(format!("uji:{hex}")),
            kek_ref: "uji-kek:v1".to_string(),
        })
    }

    async fn unwrap_dek(&self, wrapped: &str, kek_ref: &str) -> Result<SecretBytes, CryptoError> {
        if kek_ref != "uji-kek:v1" {
            return Err(CryptoError::KekMismatch);
        }
        let hex = wrapped
            .strip_prefix("uji:")
            .ok_or(CryptoError::MalformedWrappedKey)?;
        let bytes = (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| CryptoError::MalformedWrappedKey)?;
        Ok(secret_bytes(bytes))
    }
}

pub fn random_key() -> SecretBytes {
    let mut k = vec![0u8; crypto::BLIND_INDEX_KEY_LEN];
    crypto::fill_random(&mut k).unwrap();
    secret_bytes(k)
}

// ----- lingkungan -----

pub struct Env {
    pub pool: PgPool,
    pub seaweed: SeaweedFs,
    pub valkey: ValkeyContainer,
    pub storage: Arc<FileStorage>,
    pub keyring: Arc<Keyring<TestKek>>,
    /// Kunci blind index versi 1 (lama) dan 2 (terbaru) aktif bersamaan (rotasi, TD-10).
    pub blind_index: Arc<NikBlindIndexKeyring>,
    /// Keyring yang hanya memuat versi 1 (untuk menulis registri "lama").
    pub blind_index_v1: NikBlindIndexKeyring,
    pub clock: Arc<dyn Clock>,
}

impl Env {
    pub async fn start() -> Self {
        let pool = app_pool().await;
        let seaweed = SeaweedFs::start().await;
        let storage = Arc::new(
            seaweed
                .file_storage(UploadMethod::Post, Duration::from_secs(600))
                .await,
        );
        let valkey = ValkeyContainer::start();
        let clock: Arc<dyn Clock> = Arc::new(SystemClock);
        let keyring = Arc::new(
            Keyring::new(
                TestKek,
                KeyringConfig {
                    system: System::RuangSehat,
                    cache_ttl: time::Duration::minutes(5),
                    cache_capacity: 64,
                },
                Arc::clone(&clock),
            )
            .unwrap(),
        );
        let k1 = random_key();
        let k1_copy = secret_bytes(k1.expose().to_vec());
        let blind_index =
            Arc::new(NikBlindIndexKeyring::new(vec![(1, k1), (2, random_key())]).unwrap());
        let blind_index_v1 = NikBlindIndexKeyring::new(vec![(1, k1_copy)]).unwrap();
        Self {
            pool,
            seaweed,
            valkey,
            storage,
            keyring,
            blind_index,
            blind_index_v1,
            clock,
        }
    }

    pub fn services(&self) -> IdentityServices {
        IdentityServices::new(
            self.pool.clone(),
            Arc::clone(&self.clock),
            Arc::clone(&self.storage),
            self.keyring.clone(),
            Arc::clone(&self.blind_index),
            self.valkey.client(Duration::from_millis(500)),
        )
    }

    /// Slot unggah dengan foto JPEG sah sudah terunggah milik `account_id`.
    pub async fn uploaded_slot(&self, account_id: Uuid) -> Uuid {
        let bytes = jpeg(64, 48);
        let ticket = create_upload_slot(
            &self.pool,
            self.clock.as_ref(),
            &self.storage,
            UploadSlotRequest {
                owner_account_id: account_id,
                purpose: UploadPurpose::KtpPhoto,
                content_type: "image/jpeg".to_string(),
                size_bytes: bytes.len() as u64,
            },
        )
        .await
        .unwrap();
        assert_eq!(upload(&ticket.instruction, &bytes), 204);
        ticket.upload_slot_id
    }
}

/// NIK sintetis 16 digit acak berawalan `99` (bukan kode wilayah nyata; data uji, coding-standards §7.2).
pub fn synthetic_nik() -> String {
    let mut b = [0u8; 14];
    crypto::fill_random(&mut b).unwrap();
    let digits: String = b.iter().map(|x| char::from(b'0' + x % 10)).collect();
    format!("99{digits}")
}

// ----- data akun -----

pub struct Patient {
    pub account_id: Uuid,
    pub subject: String,
    pub owner_care_recipient_id: Uuid,
}

/// Akun pasien + `patient` + penerima layanan pemilik akun (data sintetis).
pub async fn new_patient(pool: &PgPool) -> Patient {
    let account_id = files::new_account(pool).await;
    let subject: String = sqlx::query_scalar!(
        r#"SELECT idp_subject AS "idp_subject!" FROM app.account WHERE id = $1"#,
        account_id
    )
    .fetch_one(pool)
    .await
    .unwrap();
    let patient_id = new_id();
    sqlx::query!(
        "INSERT INTO app.patient (id, account_id, processing_status) VALUES ($1, $2, 'normal')",
        patient_id,
        account_id,
    )
    .execute(pool)
    .await
    .unwrap();
    let care_recipient_id = new_id();
    sqlx::query!(
        "INSERT INTO app.care_recipient (id, patient_id, is_account_owner, full_name, birth_date, sex) \
         VALUES ($1, $2, true, 'Pasien Sintetis', '1990-01-01', 'female')",
        care_recipient_id,
        patient_id,
    )
    .execute(pool)
    .await
    .unwrap();
    Patient {
        account_id,
        subject,
        owner_care_recipient_id: care_recipient_id,
    }
}

/// Anggota keluarga dewasa (bukan pemilik akun) di akun `patient`.
pub async fn new_family_member(pool: &PgPool, account_id: Uuid) -> Uuid {
    let id = new_id();
    sqlx::query!(
        "INSERT INTO app.care_recipient (id, patient_id, is_account_owner, relationship, full_name, birth_date, sex) \
         SELECT $1, p.id, false, 'spouse', 'Keluarga Sintetis', '1991-02-02', 'male' \
         FROM app.patient p WHERE p.account_id = $2",
        id,
        account_id,
    )
    .execute(pool)
    .await
    .unwrap();
    id
}

/// Akun admin Ruang Sehat aktif dengan sub-peran `roles`.
pub async fn new_admin(pool: &PgPool, roles: &[AdminSubRole]) -> (Uuid, String) {
    let id = new_id();
    let subject = format!("uji-admin-{}", random_hex(8));
    sqlx::query!(
        "INSERT INTO app.account (id, account_type, idp_realm, idp_subject, email, status, status_changed_at, full_name) \
         VALUES ($1, 'rs_admin', 'admin', $2, $3, 'active', now(), 'Admin Sintetis')",
        id,
        subject,
        format!("{subject}@example.com"),
    )
    .execute(pool)
    .await
    .unwrap();
    for r in roles {
        sqlx::query!(
            "INSERT INTO app.rs_admin_role (id, account_id, sub_role) VALUES ($1, $2, $3)",
            new_id(),
            id,
            r.as_str(),
        )
        .execute(pool)
        .await
        .unwrap();
    }
    (id, subject)
}

// ----- pemanggil terotorisasi dari jalur produksi -----

fn iss(realm: &str) -> String {
    format!("https://auth.ruang-sehat.invalid/realms/{realm}")
}

fn aud(realm: &str) -> String {
    format!("rs-api-{realm}")
}

struct Renderer;

impl ErrorRenderer for Renderer {
    fn render(&self, error: CommonError, correlation_id: CorrelationId) -> Response {
        let body = error.to_error_body(correlation_id, "https://d.example/problems");
        (
            error.http_status(),
            [(CONTENT_TYPE, "application/problem+json")],
            serde_json::to_vec(&body).unwrap(),
        )
            .into_response()
    }
}

#[derive(Default)]
struct MemResolver {
    accounts: Mutex<HashMap<(Realm, String), AccountRecord>>,
}

impl SessionGuard for MemResolver {
    fn is_revoked<'a>(&'a self, _ctx: &'a AuthContext) -> RevocationFuture<'a> {
        Box::pin(async move { Ok(false) })
    }
}

impl PrincipalResolver for MemResolver {
    fn resolve<'a>(&'a self, ctx: &'a AuthContext) -> ResolveFuture<'a> {
        let rec = self
            .accounts
            .lock()
            .unwrap()
            .get(&(ctx.realm(), ctx.subject().to_string()))
            .cloned();
        Box::pin(async move { Ok::<_, ResolveError>(rec) })
    }
}

type Captured = Arc<Mutex<Option<Principal>>>;

#[utoipa::path(get, path = "/patient/v1/uji-principal", responses((status = 204)))]
async fn capture_patient(State(c): State<Captured>, p: Principal) -> Response {
    *c.lock().unwrap() = Some(p);
    axum::http::StatusCode::NO_CONTENT.into_response()
}

#[utoipa::path(get, path = "/backoffice/v1/uji-principal", responses((status = 204)))]
async fn capture_admin(State(c): State<Captured>, p: Principal) -> Response {
    *c.lock().unwrap() = Some(p);
    axum::http::StatusCode::NO_CONTENT.into_response()
}

async fn with_client_ip(mut req: axum::extract::Request, next: axum::middleware::Next) -> Response {
    req.extensions_mut().insert(ClientIp(CLIENT_IP));
    next.run(req).await
}

/// Penerbit `Principal` uji: server kecil dengan validator token dan penegak kebijakan produksi.
pub struct Principals {
    pasien: TestRealmKey,
    admin: TestRealmKey,
    resolver: Arc<MemResolver>,
    captured: Captured,
    with_ip: SocketAddr,
    without_ip: SocketAddr,
}

impl Principals {
    pub async fn start() -> Self {
        let pasien = TestRealmKey::generate("kid-pasien").unwrap();
        let admin = TestRealmKey::generate("kid-admin").unwrap();
        let resolver = Arc::new(MemResolver::default());
        let captured: Captured = Arc::new(Mutex::new(None));
        let mut addrs = Vec::new();
        for ip in [true, false] {
            let realm = |r: Realm, key: &TestRealmKey| RealmConfig {
                realm: r,
                issuer: iss(r.as_str()),
                audiences: vec![aud(r.as_str())],
                algorithms: vec![Algorithm::RS256],
                authorized_parties: vec![],
                jwks: Arc::new(StaticJwks::new(jwks_document(&[key.jwk()]))),
            };
            let authenticator = Authenticator::new(
                AuthnConfig::with_defaults(
                    Backend::RuangSehat,
                    vec![realm(Realm::Pasien, &pasien), realm(Realm::Admin, &admin)],
                ),
                Arc::new(SystemClock),
            )
            .unwrap();
            let auth = AuthState::new(Arc::new(authenticator), Arc::new(Renderer));
            let (router, _) = PolicyRouter::<Captured>::new()
                .routes(Policy::patient(), routes!(capture_patient))
                .routes(Policy::rs_admin(&AdminSubRole::ALL), routes!(capture_admin))
                .finish(Enforcement::new(
                    auth,
                    resolver.clone(),
                    resolver.clone(),
                    DenialSink::start(64, 1, None).unwrap(),
                ))
                .unwrap();
            let mut app =
                router
                    .with_state(Arc::clone(&captured))
                    .layer(axum::middleware::from_fn(
                        observability::correlation_middleware,
                    ));
            if ip {
                app = app.layer(axum::middleware::from_fn(with_client_ip));
            }
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            addrs.push(listener.local_addr().unwrap());
            tokio::spawn(async move { app.serve(listener, std::future::pending()).await.unwrap() });
        }
        Self {
            pasien,
            admin,
            resolver,
            captured,
            with_ip: addrs[0],
            without_ip: addrs[1],
        }
    }

    async fn fetch(&self, addr: SocketAddr, path: &str, token: &str) -> Principal {
        let mut s = TcpStream::connect(addr).await.unwrap();
        let req = format!(
            "GET {path} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\nContent-Length: 0\r\n\
             Authorization: Bearer {token}\r\n\r\n"
        );
        s.write_all(req.as_bytes()).await.unwrap();
        let mut buf = Vec::new();
        s.read_to_end(&mut buf).await.unwrap();
        let text = String::from_utf8_lossy(&buf);
        assert!(
            text.starts_with("HTTP/1.1 204"),
            "principal uji gagal: {text}"
        );
        self.captured.lock().unwrap().take().unwrap()
    }

    fn token(key: &TestRealmKey, realm: &str, sub: &str) -> String {
        let now = now().unix_timestamp();
        key.sign(&json!({
            "iss": iss(realm), "aud": aud(realm), "sub": sub, "typ": "Bearer",
            "sid": "sesi-uji", "azp": "app", "iat": now - 5, "exp": now + 300,
        }))
        .unwrap()
    }

    /// Pemanggil pasien `p` (dengan IP klien bila `with_ip`).
    pub async fn patient(&self, p: &Patient, with_ip: bool) -> Principal {
        self.resolver.accounts.lock().unwrap().insert(
            (Realm::Pasien, p.subject.clone()),
            AccountRecord {
                account_id: p.account_id,
                account_type: AccountType::Patient,
                status: AccountStatus::Active,
                sub_roles: vec![],
            },
        );
        let addr = if with_ip {
            self.with_ip
        } else {
            self.without_ip
        };
        let token = Self::token(&self.pasien, "pasien", &p.subject);
        self.fetch(addr, "/patient/v1/uji-principal", &token).await
    }

    /// Pemanggil admin Ruang Sehat.
    pub async fn admin(
        &self,
        account_id: Uuid,
        subject: &str,
        roles: &[AdminSubRole],
    ) -> Principal {
        self.resolver.accounts.lock().unwrap().insert(
            (Realm::Admin, subject.to_string()),
            AccountRecord {
                account_id,
                account_type: AccountType::RsAdmin,
                status: AccountStatus::Active,
                sub_roles: roles.to_vec(),
            },
        );
        let token = Self::token(&self.admin, "admin", subject);
        self.fetch(self.with_ip, "/backoffice/v1/uji-principal", &token)
            .await
    }
}

// ----- audit: relay ke database audit lalu baca lewat verifier -----

async fn connection(var: &str) -> PgConnection {
    let env = observability::EnvSource::from_process();
    let url = env
        .required_secret(var)
        .unwrap_or_else(|_| panic!("{var} tidak diset (jalankan lewat scripts/db-ephemeral.sh)"));
    PgConnection::connect(url.expose())
        .await
        .unwrap_or_else(|_| panic!("koneksi {var} gagal"))
}

/// Memindahkan semua event outbox ke database audit lewat relay, lalu mengembalikan event berkode
/// `event_code` (misalnya `e_10`) yang subjeknya `subject`.
///
/// F1-T69 (US-1501 AC1–AC2): sebelum dipindahkan, SEMUA payload di `audit_outbox` diperiksa
/// `audit::check_payload` (field wajib PRD §7.2 per kode event, larangan §7.3); test gagal bila ada event
/// yang kurang field atau memuat data terlarang, termasuk event hasil gagal dan E-05 yang dipancarkan
/// sepanjang test itu.
pub async fn audit_events(event_code: &str, subject: Uuid) -> Vec<StoredEvent> {
    for p in &pending_payloads().await {
        assert_payload_complete(p, audit::check_payload(p));
    }
    relay_and_read(|e| {
        e.event.body.event_code == event_code && e.event.body.subject_id == Some(subject)
    })
    .await
}

/// Seperti [`audit_events`] (termasuk pemeriksaan `audit::check_payload` atas semua payload outbox), tetapi
/// untuk event berkode `event_code` atas objek `object_id` (event tanpa subjek, misalnya percobaan gagal
/// atas objek yang tidak ditemukan).
pub async fn audit_events_for_object(event_code: &str, object_id: Uuid) -> Vec<StoredEvent> {
    for p in &pending_payloads().await {
        assert_payload_complete(p, audit::check_payload(p));
    }
    let object_id = object_id.hyphenated().to_string();
    relay_and_read(|e| {
        e.event.body.event_code == event_code
            && e.event.body.object_id.as_deref() == Some(object_id.as_str())
    })
    .await
}

async fn pending_payloads() -> Vec<serde_json::Value> {
    let mut conn = connection("RS_TEST_DB_RELAY_URL").await;
    sqlx::query_scalar!("SELECT payload FROM app.audit_outbox ORDER BY occurred_at, id")
        .fetch_all(&mut conn)
        .await
        .unwrap()
}

fn assert_payload_complete(p: &serde_json::Value, violations: Vec<audit::Violation>) {
    assert!(
        violations.is_empty(),
        "event {} `{}` tidak lengkap/terlarang: {:?}",
        p["event_code"],
        p["action_name"],
        violations
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
    );
}

async fn relay_and_read(keep: impl Fn(&StoredEvent) -> bool) -> Vec<StoredEvent> {
    let mut conn = connection("RS_TEST_DB_RELAY_URL").await;
    let mut lock = None;
    for _ in 0..300 {
        match SingletonLock::try_acquire(conn, RS_AUDIT_RELAY)
            .await
            .unwrap()
        {
            LockAttempt::Acquired(l) => {
                lock = Some(l);
                break;
            }
            LockAttempt::Busy(c) => {
                conn = c;
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }
    }
    let mut lock = lock.expect("kunci relay");
    let mut writer = connection("RS_TEST_DB_AUDIT_WRITER_URL").await;
    loop {
        let b = audit::relay::relay_batch(&mut lock, &mut writer, 50)
            .await
            .unwrap();
        if b.fetched == 0 {
            break;
        }
    }
    lock.release().await.unwrap();
    let mut verifier = connection("RS_TEST_DB_AUDIT_VERIFIER_URL").await;
    let mut out = Vec::new();
    let mut cursor = PageCursor::START;
    loop {
        let page = read_page(&mut verifier, cursor, 500).await.unwrap();
        let Some(last) = page.last() else {
            break;
        };
        cursor = PageCursor {
            seq: last.event.seq,
            event_id: last.event.body.event_id,
        };
        out.extend(page.into_iter().filter(|e| keep(e)));
    }
    out
}

/// Persetujuan tiruan langsung di database lewat jalur tulis yang sama dengan Verifikator (event v2 +
/// proyeksi + registri NIK pemilik akun dengan blind index versi `bidx_version`); keputusan Verifikator
/// sungguhan diuji di F1-T62.
pub async fn approve_in_db(
    env: &Env,
    care_recipient_id: Uuid,
    request_id: Uuid,
    nik: &str,
    v1_key: bool,
) {
    let nik = crypto::Nik::parse(nik).unwrap();
    let b = if v1_key {
        env.blind_index_v1.compute(&nik).unwrap()
    } else {
        env.blind_index.compute(&nik).unwrap()
    };
    let mut tx = env.pool.begin().await.unwrap();
    sqlx::query!(
        "INSERT INTO app.identity_verification_request_status_event \
             (id, identity_verification_request_id, version, from_status, to_status, actor_type, correlation_id) \
         VALUES ($1, $2, 2, 'pending', 'approved', 'rs_admin', $3)",
        foundation::new_id(),
        request_id,
        foundation::new_id(),
    )
    .execute(&mut *tx)
    .await
    .unwrap();
    sqlx::query!(
        "UPDATE app.identity_verification_request_state SET status = 'approved', version = 2 \
         WHERE identity_verification_request_id = $1 AND version = 1",
        request_id
    )
    .execute(&mut *tx)
    .await
    .unwrap();
    sqlx::query!(
        "INSERT INTO app.account_owner_nik_registry \
             (id, nik_bidx, nik_bidx_key_version, care_recipient_id, identity_verification_request_id) \
         VALUES ($1, $2, $3, $4, $5)",
        foundation::new_id(),
        &b.value[..],
        b.key_version,
        care_recipient_id,
        request_id,
    )
    .execute(&mut *tx)
    .await
    .unwrap();
    tx.commit().await.unwrap();
}

// ----- bacaan tabel untuk pemeriksaan -----

pub async fn request_count(pool: &PgPool, care_recipient_id: Uuid) -> i64 {
    sqlx::query_scalar!(
        r#"SELECT count(*) AS "n!" FROM app.identity_verification_request WHERE care_recipient_id = $1"#,
        care_recipient_id
    )
    .fetch_one(pool)
    .await
    .unwrap()
}

pub async fn slot_consumed_or_gone(pool: &PgPool, slot: Uuid) -> bool {
    files::slot_consumed(pool, slot).await || !files::slot_exists(pool, slot).await
}

/// NIK staf akun admin sebagai blind index versi `key_version` (US-201 AC8, KEPUTUSAN-294); jalur produksi
/// lewat `AdminAccounts` (diuji `identity_admin_accounts`).
pub async fn set_staff_nik(env: &Env, account_id: Uuid, nik: &str, key_version: i16) {
    let b = env
        .blind_index
        .compute_with_version(&crypto::Nik::parse(nik).unwrap(), key_version)
        .unwrap();
    sqlx::query!(
        "UPDATE app.account SET staff_nik_bidx = $2, staff_nik_bidx_key_version = $3 WHERE id = $1",
        account_id,
        &b.value[..],
        b.key_version,
    )
    .execute(&env.pool)
    .await
    .unwrap();
}
