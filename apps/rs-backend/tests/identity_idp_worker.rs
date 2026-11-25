//! ADR-030 langkah 1-2 (T1, CR-026, KEPUTUSAN-302/305): worker IdP terhadap PostgreSQL 18 sekali pakai
//! (`scripts/db-ephemeral.sh`, role runtime `rs_api` = anggota `rs_app` seperti `rs_worker`) dan tiruan Admin REST
//! Keycloak realm `korporat` di 127.0.0.1 yang menyimpan keadaan pengguna (atribut `rs_intent_id`, kredensial,
//! `enabled`) dan mencatat setiap panggilan. Perilaku Keycloak sungguhan untuk endpoint yang sama diuji
//! `scripts/keycloak-check.sh` (`cek_pekerja_idp`). Tanpa variabel `RS_TEST_DB_*` test GAGAL.
//!
//! Job IdP berbagi tabel `app.job`, sehingga test di berkas ini berjalan satu per satu dan setiap test mulai
//! dengan menggagalkan job IdP sisa test lain.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

mod audit_support;

use audit_support::assert_event_complete;
use axum::Router;
use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, Method, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use foundation::{Clock, CorrelationId, FixedClock, Sensitive, SystemClock, Uuid, new_id};
use observability::EnvSource;
use platform_jobs::{
    Backoff, ClaimRequest, ClaimedJob, JobOutcome, JobPayload, JobType, LeaseOutcome, MaxAttempts,
    NewJob, WorkerConfig, WorkerId, claim, enqueue, sweep_expired,
};
use rs_backend::modules::identity::api::{
    CreateOutcome, IdpJobKind, IdpRealm, IdpUserAdmin, IdpWorker, IntentActor, IntentError,
    IntentPurpose, IntentRepoError, IntentRequest, KeycloakAddress, PendingAccount,
    PendingAccountRequest, RealmCredential, idp_job_types, record_intent, register_pending_account,
};
use serde_json::{Value, json};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

static SERIAL: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

const SECRET: &str = "rahasia-uji-idp-worker-korporat-0123456789";
const CLIENT: &str = "rs-backend-idp-worker";
const USERS: &str = "/admin/realms/korporat/users";

// ------------------------------------------------------------------------------- tiruan Keycloak

#[derive(Clone, Debug)]
struct KcUser {
    id: String,
    email: String,
    attrs: Vec<String>,
    enabled: bool,
    creds: Vec<String>,
}

#[derive(Default)]
struct Kc {
    users: Mutex<Vec<KcUser>>,
    calls: Mutex<Vec<(String, String, Value)>>,
    down: AtomicBool,
}

type S = Arc<Kc>;

impl Kc {
    fn calls(&self) -> Vec<(String, String, Value)> {
        self.calls.lock().unwrap().clone()
    }
    fn count(&self, method: &str, contains: &str) -> usize {
        self.calls()
            .iter()
            .filter(|(m, p, _)| m == method && p.contains(contains))
            .count()
    }
    fn user(&self, id: &str) -> Option<KcUser> {
        self.users
            .lock()
            .unwrap()
            .iter()
            .find(|u| u.id == id)
            .cloned()
    }
    fn edit(&self, id: &str, f: impl FnOnce(&mut KcUser)) {
        let mut users = self.users.lock().unwrap();
        f(users.iter_mut().find(|u| u.id == id).unwrap());
    }
    fn add(&self, email: &str, attrs: Vec<String>) -> String {
        let id = new_id().hyphenated().to_string();
        self.users.lock().unwrap().push(KcUser {
            id: id.clone(),
            email: email.to_string(),
            attrs,
            enabled: true,
            creds: Vec::new(),
        });
        id
    }
}

fn rep(u: &KcUser) -> Value {
    json!({"id": u.id, "username": u.email, "email": u.email, "enabled": u.enabled,
           "attributes": {"rs_intent_id": u.attrs}})
}

async fn any(
    State(f): State<S>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let path = uri.path().to_string();
    if path == "/realms/korporat/protocol/openid-connect/token" {
        let form = String::from_utf8(body.to_vec()).unwrap();
        if form.contains(&format!("client_id={CLIENT}"))
            && form.contains(&format!("client_secret={SECRET}"))
        {
            return (
                StatusCode::OK,
                axum::Json(json!({"access_token": "token-akses-uji-worker", "expires_in": 300})),
            )
                .into_response();
        }
        return StatusCode::UNAUTHORIZED.into_response();
    }
    if headers.get("authorization").and_then(|v| v.to_str().ok())
        != Some("Bearer token-akses-uji-worker")
    {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let json_body: Value = if body.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&body).unwrap()
    };
    f.calls
        .lock()
        .unwrap()
        .push((method.to_string(), uri.to_string(), json_body.clone()));
    if f.down.load(Ordering::SeqCst) {
        return StatusCode::SERVICE_UNAVAILABLE.into_response();
    }
    let rest = path.strip_prefix(USERS).unwrap_or("!");
    let parts: Vec<&str> = rest.trim_start_matches('/').split('/').collect();
    match (method.as_str(), rest, parts.as_slice()) {
        ("POST", "", _) => {
            let email = json_body["email"].as_str().unwrap().to_string();
            if f.users.lock().unwrap().iter().any(|u| u.email == email) {
                return StatusCode::CONFLICT.into_response();
            }
            let attrs: Vec<String> = json_body["attributes"]["rs_intent_id"]
                .as_array()
                .map(|a| a.iter().map(|v| v.as_str().unwrap().to_string()).collect())
                .unwrap_or_default();
            let id = f.add(&email, attrs);
            let mut h = HeaderMap::new();
            h.insert(
                "location",
                format!("http://127.0.0.1{USERS}/{id}").parse().unwrap(),
            );
            (StatusCode::CREATED, h).into_response()
        }
        ("GET", "", _) => {
            let q = uri
                .query()
                .unwrap_or("")
                .split('&')
                .find_map(|kv| kv.strip_prefix("q=rs_intent_id:"))
                .unwrap_or("")
                .to_string();
            let list: Vec<Value> = f
                .users
                .lock()
                .unwrap()
                .iter()
                .filter(|u| u.attrs.contains(&q))
                .map(rep)
                .collect();
            axum::Json(Value::Array(list)).into_response()
        }
        ("GET", _, [id]) => match f.user(id) {
            Some(u) => axum::Json(rep(&u)).into_response(),
            None => StatusCode::NOT_FOUND.into_response(),
        },
        ("GET", _, [id, "credentials"]) => match f.user(id) {
            Some(u) => axum::Json(Value::Array(
                u.creds
                    .iter()
                    .map(|c| json!({"id": "c", "type": c, "createdDate": 1}))
                    .collect(),
            ))
            .into_response(),
            None => StatusCode::NOT_FOUND.into_response(),
        },
        ("PUT", _, [id, "execute-actions-email"]) | ("POST", _, [id, "logout"]) => {
            if f.user(id).is_some() {
                StatusCode::NO_CONTENT.into_response()
            } else {
                StatusCode::NOT_FOUND.into_response()
            }
        }
        ("PUT", _, [id]) => {
            if f.user(id).is_none() {
                return StatusCode::NOT_FOUND.into_response();
            }
            let enabled = json_body["enabled"].as_bool().unwrap_or(true);
            f.edit(id, |u| u.enabled = enabled);
            StatusCode::NO_CONTENT.into_response()
        }
        ("DELETE", _, [id]) => {
            let mut users = f.users.lock().unwrap();
            let before = users.len();
            users.retain(|u| u.id != *id);
            if users.len() < before {
                StatusCode::NO_CONTENT.into_response()
            } else {
                StatusCode::NOT_FOUND.into_response()
            }
        }
        _ => StatusCode::NOT_FOUND.into_response(),
    }
}

// ------------------------------------------------------------------------------------- perakitan

async fn connect(var: &str) -> PgPool {
    let url = EnvSource::from_process()
        .required_secret(var)
        .unwrap_or_else(|_| {
            panic!(
                "{var} tidak diset: jalankan lewat `bash scripts/db-ephemeral.sh cargo test ...`"
            )
        });
    PgPoolOptions::new()
        .max_connections(8)
        .connect(url.expose())
        .await
        .unwrap()
}

struct Env {
    pool: PgPool,
    relay: PgPool,
    kc: S,
    admin: Arc<IdpUserAdmin>,
    worker: IdpWorker,
    /// Jam pengambilan job (di depan jam sistem supaya job baru sudah jatuh tempo).
    claim_clock: FixedClock,
}

async fn start() -> Env {
    let pool = connect("RS_TEST_DB_APP_URL").await;
    let relay = connect("RS_TEST_DB_RELAY_URL").await;
    // Job IdP sisa test lain tidak ikut diambil.
    sqlx::query(
        "UPDATE app.job SET status = 'failed', finished_at = now(), locked_until = NULL, \
         last_error = 'test_cleanup' WHERE job_type LIKE 'idp\\_%' AND status IN ('queued', 'running')",
    )
    .execute(&pool)
    .await
    .unwrap();
    let kc: S = Arc::new(Kc::default());
    let app = Router::new().fallback(any).with_state(kc.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    let admin = Arc::new(
        IdpUserAdmin::new(
            format!("http://{addr}").parse::<KeycloakAddress>().unwrap(),
            Duration::from_secs(5),
            HashMap::from([(
                IdpRealm::Korporat,
                RealmCredential {
                    client_id: CLIENT.into(),
                    secret: Sensitive::new(SECRET.into()),
                },
            )]),
        )
        .unwrap(),
    );
    let cfg = WorkerConfig::new(
        WorkerId::new("idp_worker").unwrap(),
        idp_job_types().unwrap(),
        10,
        time::Duration::seconds(60),
        Backoff::new(time::Duration::seconds(1), time::Duration::seconds(5)).unwrap(),
    )
    .unwrap();
    let worker =
        IdpWorker::new(pool.clone(), Arc::new(SystemClock), Arc::clone(&admin), cfg).unwrap();
    let claim_clock = FixedClock::new(SystemClock.now() + time::Duration::hours(1));
    Env {
        pool,
        relay,
        kc,
        admin,
        worker,
        claim_clock,
    }
}

impl Env {
    async fn register(&self, email: &str, purpose: IntentPurpose) -> PendingAccount {
        let mut tx = self.pool.begin().await.unwrap();
        let p = register_pending_account(
            &mut tx,
            &PendingAccountRequest {
                account_type: "corporate_admin",
                realm: IdpRealm::Korporat,
                email,
                full_name: Some("HR Sintetis"),
                purpose,
                actor: IntentActor::SYSTEM,
                correlation_id: CorrelationId::new(),
            },
            &SystemClock,
        )
        .await
        .unwrap();
        tx.commit().await.unwrap();
        p
    }

    async fn claim_as(&self, worker: &WorkerId, clock: &FixedClock) -> Option<ClaimedJob> {
        let types = idp_job_types().unwrap();
        let mut jobs = claim(
            &self.pool,
            &ClaimRequest {
                worker,
                job_types: &types,
                limit: 1,
                lease: time::Duration::seconds(60),
            },
            clock,
        )
        .await
        .unwrap();
        jobs.pop()
    }

    async fn claim(&self) -> ClaimedJob {
        self.claim_as(&WorkerId::new("idp_worker").unwrap(), &self.claim_clock)
            .await
            .expect("ada job IdP jatuh tempo")
    }

    async fn scalar_text(&self, sql: &'static str, id: Uuid) -> Option<String> {
        sqlx::query_scalar::<_, Option<String>>(sql)
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .unwrap()
    }

    async fn account(&self, id: Uuid) -> (String, Option<String>, Option<String>) {
        sqlx::query_as::<_, (String, Option<String>, Option<String>)>(
            "SELECT status, idp_subject, delete_reason FROM app.account WHERE id = $1",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .unwrap()
    }

    async fn intent_status(&self, id: Uuid) -> String {
        self.scalar_text(
            "SELECT status FROM app.idp_provisioning_intent_state WHERE idp_provisioning_intent_id = $1",
            id,
        )
        .await
        .unwrap()
    }

    async fn status_events(&self, id: Uuid) -> Vec<(String, Option<String>)> {
        sqlx::query_as::<_, (String, Option<String>)>(
            "SELECT to_status, reason_code FROM app.idp_provisioning_intent_status_event \
             WHERE idp_provisioning_intent_id = $1 ORDER BY version",
        )
        .bind(id)
        .fetch_all(&self.pool)
        .await
        .unwrap()
    }

    /// Niat lanjutan (operasi) dengan niat asal `origin`.
    async fn follow_up(&self, origin: Uuid, operation: &str) -> Option<Uuid> {
        sqlx::query_scalar::<_, Uuid>(
            "SELECT id FROM app.idp_provisioning_intent WHERE origin_intent_id = $1 AND operation = $2",
        )
        .bind(origin)
        .bind(operation)
        .fetch_optional(&self.pool)
        .await
        .unwrap()
    }

    /// Payload audit di outbox untuk objek akun `account`.
    async fn audit(&self, account: Uuid) -> Vec<Value> {
        let all: Vec<Value> = sqlx::query_scalar::<_, Value>(
            "SELECT payload FROM app.audit_outbox ORDER BY occurred_at, id",
        )
        .fetch_all(&self.relay)
        .await
        .unwrap();
        let id = account.hyphenated().to_string();
        all.into_iter()
            .filter(|p| p["object_id"].as_str() == Some(id.as_str()))
            .collect()
    }

    async fn job_error(&self, id: Uuid) -> (String, Option<String>) {
        sqlx::query_as::<_, (String, Option<String>)>(
            "SELECT status, last_error FROM app.job WHERE id = $1",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .unwrap()
    }

    /// Niat tanpa niat asal (nonaktifkan/akhiri sesi) di transaksi sendiri, seperti pemanggil API.
    async fn record(
        &self,
        account_id: Uuid,
        realm: IdpRealm,
        kind: IdpJobKind,
        purpose: IntentPurpose,
    ) -> Result<Uuid, IntentError> {
        let mut tx = self.pool.begin().await.unwrap();
        let id = record_intent(
            &mut tx,
            &IntentRequest {
                account_id,
                realm,
                kind,
                purpose,
                origin_intent_id: None,
                actor: IntentActor::SYSTEM,
                correlation_id: CorrelationId::new(),
            },
            &SystemClock,
        )
        .await?;
        tx.commit().await.unwrap();
        Ok(id)
    }

    /// Job IdP yang masih antre disingkirkan supaya hanya job berikutnya yang jatuh tempo.
    async fn cancel_queued_idp_jobs(&self) {
        sqlx::query(
            "UPDATE app.job SET status = 'cancelled', finished_at = now() \
             WHERE job_type LIKE 'idp\\_%' AND status = 'queued'",
        )
        .execute(&self.pool)
        .await
        .unwrap();
    }

    /// Menjalankan job `create_user` akun sampai pengguna diadopsi; mengembalikan ID pengguna Keycloak.
    async fn provision(&self, p: &PendingAccount) -> String {
        let job = self.claim().await;
        let out = self.worker.handle(job).await;
        assert_eq!(out, JobOutcome::Completed(LeaseOutcome::Applied));
        self.account(p.account_id).await.1.unwrap()
    }

    /// Tidak ada email (bagian lokal maupun utuh) di `app.job` atau tabel niat (ADR-030 butir 2).
    async fn assert_no_email_in_job_or_intent(&self, email: &str) {
        let local = email.split('@').next().unwrap();
        let dumps: Vec<String> = sqlx::query_scalar::<_, String>(
            "SELECT to_jsonb(j)::text FROM app.job j WHERE job_type LIKE 'idp\\_%' \
             UNION ALL SELECT to_jsonb(i)::text FROM app.idp_provisioning_intent i \
             UNION ALL SELECT to_jsonb(e)::text FROM app.idp_provisioning_intent_status_event e \
             UNION ALL SELECT to_jsonb(s)::text FROM app.idp_provisioning_intent_state s",
        )
        .fetch_all(&self.pool)
        .await
        .unwrap();
        assert!(!dumps.is_empty());
        for d in dumps {
            assert!(
                !d.contains(local) && !d.contains('@') && !d.contains(SECRET),
                "data pribadi/rahasia di job atau niat: {d}"
            );
        }
    }
}

fn email(tag: &str) -> String {
    format!(
        "hr-{tag}-{}@contoh.invalid",
        &new_id().simple().to_string()[24..]
    )
}

// ------------------------------------------------------------------------------------------- test

// trace: ADR-030 butir 6 (adopsi setelah crash, sewa hilang), butir 2 (muatan hanya ID niat), butir 5
#[tokio::test]
async fn create_user_adopts_after_crash_and_lease_loss_without_duplicates() {
    let _g = SERIAL.lock().await;
    let env = start().await;
    let mail = email("adopsi");
    let p = env.register(&mail, IntentPurpose::HrSelfRegistration).await;
    assert_eq!(env.account(p.account_id).await.0, "pending_provisioning");
    assert_eq!(env.intent_status(p.intent_id).await, "pending");

    // Worker A mengambil job lalu "mati" tepat setelah POST users berhasil.
    let a = WorkerId::new("idp_worker").unwrap();
    let job_a = env.claim_as(&a, &env.claim_clock).await.unwrap();
    let created = env
        .admin
        .create_user(IdpRealm::Korporat, &mail, p.intent_id, &SystemClock)
        .await
        .unwrap();
    let CreateOutcome::Created(kc_id) = created else {
        panic!("tiruan menolak pembuatan pertama")
    };

    // Sewa A habis; penyapu mengembalikan job, worker B mengambilnya (percobaan kedua).
    let later = FixedClock::new(env.claim_clock.now() + time::Duration::minutes(5));
    sweep_expired(&env.pool, &idp_job_types().unwrap(), &later)
        .await
        .unwrap();
    let b = WorkerId::new("idp_worker").unwrap();
    let job_b = env.claim_as(&b, &later).await.unwrap();
    assert_eq!(job_b.attempt(), 2);

    // A yang terlambat: efek Keycloak idempoten (adopsi), tetapi hasil dan audit tidak tercatat.
    let out_a = env.worker.handle(job_a).await;
    assert_eq!(out_a, JobOutcome::Completed(LeaseOutcome::LeaseLost));
    assert_eq!(env.account(p.account_id).await.0, "pending_provisioning");
    assert_eq!(env.status_events(p.intent_id).await.len(), 1);
    assert!(env.audit(p.account_id).await.is_empty());

    // B: mengadopsi pengguna yang sama, tanpa POST kedua.
    let out_b = env.worker.handle(job_b).await;
    assert_eq!(out_b, JobOutcome::Completed(LeaseOutcome::Applied));
    assert_eq!(env.kc.count("POST", USERS), 1, "{:?}", env.kc.calls());
    let (status, subject, _) = env.account(p.account_id).await;
    assert_eq!(
        (status.as_str(), subject.as_deref()),
        ("pending_activation", Some(kc_id.as_str()))
    );
    assert_eq!(
        env.status_events(p.intent_id).await,
        vec![
            ("pending".to_string(), None),
            ("succeeded".to_string(), None)
        ]
    );
    let events = env.audit(p.account_id).await;
    assert_eq!(events.len(), 1);
    assert_event_complete(&events[0], "e_10");
    assert_eq!(events[0]["action_name"], "idp_user_provisioned");
    assert!(!events[0].to_string().contains(&mail));

    // Job diulang setelah hasil tercatat: selesai tanpa panggilan Keycloak.
    let calls_before = env.kc.calls().len();
    enqueue(
        &env.pool,
        &NewJob::new(
            JobType::new("idp_create_user").unwrap(),
            JobPayload::new().with_id("intent_id", p.intent_id).unwrap(),
            SystemClock.now(),
            MaxAttempts::new(3).unwrap(),
        ),
    )
    .await
    .unwrap();
    // Job lanjutan send_activation (dicatat di transaksi hasil) dan ulangan create sama-sama jatuh tempo.
    let activation = env.follow_up(p.intent_id, "send_activation").await.unwrap();
    let mut saw_activation = false;
    for _ in 0..2 {
        let job = env.claim().await;
        let intent = job.payload().unwrap().id("intent_id").unwrap();
        let out = env.worker.handle(job).await;
        if intent == activation {
            saw_activation = true;
            assert_eq!(out, JobOutcome::Completed(LeaseOutcome::Applied));
        } else {
            assert_eq!(out, JobOutcome::Succeeded);
        }
    }
    assert!(saw_activation);
    assert_eq!(env.kc.count("POST", USERS), 1);
    let sends: Vec<_> = env
        .kc
        .calls()
        .into_iter()
        .skip(calls_before)
        .filter(|(m, p, _)| m == "PUT" && p.contains("execute-actions-email"))
        .collect();
    assert_eq!(sends.len(), 1);
    assert!(
        sends[0]
            .1
            .ends_with(&format!("{kc_id}/execute-actions-email?lifespan=86400"))
    );
    assert_eq!(
        sends[0].2,
        json!(["VERIFY_EMAIL", "UPDATE_PASSWORD", "CONFIGURE_TOTP"])
    );
    assert_eq!(env.intent_status(activation).await, "succeeded");
    let events = env.audit(p.account_id).await;
    assert_eq!(events.len(), 2);
    assert_event_complete(&events[1], "e_10");
    assert_eq!(events[1]["action_name"], "idp_activation_email_sent");

    env.assert_no_email_in_job_or_intent(&mail).await;
}

// trace: ADR-030 test negatif idp_send_activation / idp_delete_orphan, KEPUTUSAN-201 butir 2
#[tokio::test]
async fn activation_and_orphan_deletion_refuse_credentialed_or_foreign_users() {
    let _g = SERIAL.lock().await;
    let env = start().await;

    // (a) Pengguna sudah berkredensial: email aktivasi ditolak, tanpa panggilan execute-actions-email.
    let p1 = env
        .register(&email("kredensial"), IntentPurpose::HrInvitation)
        .await;
    let kc1 = env.provision(&p1).await;
    env.kc.edit(&kc1, |u| u.creds = vec!["password".into()]);
    let act1 = env
        .follow_up(p1.intent_id, "send_activation")
        .await
        .unwrap();
    let job = env.claim().await;
    let job_id = job.id().as_uuid();
    let out = env.worker.handle(job).await;
    assert!(
        matches!(out, JobOutcome::FailedInTx(LeaseOutcome::Applied, ref c) if c.as_str() == "idp_user_credentialed")
    );
    assert_eq!(env.kc.count("PUT", "execute-actions-email"), 0);
    assert_eq!(
        env.status_events(act1).await.last().unwrap(),
        &(
            "rejected".to_string(),
            Some("idp_user_credentialed".to_string())
        )
    );
    assert_eq!(
        env.job_error(job_id).await,
        (
            "failed".to_string(),
            Some("idp_user_credentialed".to_string())
        )
    );
    let events = env.audit(p1.account_id).await;
    let refused = events.last().unwrap();
    assert_event_complete(refused, "e_10");
    assert_eq!(refused["action_name"], "idp_activation_email_refused");
    assert_eq!(refused["outcome_reason_code"], "idp_user_credentialed");

    // (b) Atribut rs_intent_id milik niat lain: ditolak.
    let p2 = env
        .register(&email("atribut"), IntentPurpose::HrInvitation)
        .await;
    let kc2 = env.provision(&p2).await;
    env.kc
        .edit(&kc2, |u| u.attrs = vec![new_id().hyphenated().to_string()]);
    let job = env.claim().await;
    let out = env.worker.handle(job).await;
    assert!(
        matches!(out, JobOutcome::FailedInTx(_, ref c) if c.as_str() == "intent_attribute_mismatch")
    );
    assert_eq!(env.kc.count("PUT", "execute-actions-email"), 0);

    // (c) Email sudah dipakai pengguna Keycloak lain (tanpa atribut niat ini): tidak diadopsi, tidak dihapus;
    // akun dihapus lunak dan email bebas lagi.
    let mail3 = email("bentrok");
    let other = env.kc.add(&mail3, vec![]);
    let p3 = env
        .register(&mail3, IntentPurpose::HrSelfRegistration)
        .await;
    let job = env.claim().await;
    let out = env.worker.handle(job).await;
    assert!(matches!(out, JobOutcome::FailedInTx(_, ref c) if c.as_str() == "idp_email_taken"));
    let (_, subject, reason) = env.account(p3.account_id).await;
    assert_eq!(
        (subject, reason.as_deref()),
        (None, Some("idp_provisioning_failed"))
    );
    assert!(env.kc.user(&other).is_some());
    assert_eq!(env.kc.count("DELETE", USERS), 0);
    assert!(env.follow_up(p3.intent_id, "delete_orphan").await.is_none());
    let again = env
        .register(&mail3, IntentPurpose::HrSelfRegistration)
        .await;
    assert_ne!(again.account_id, p3.account_id);
    env.kc.users.lock().unwrap().retain(|u| u.id != other);
    for _ in 0..2 {
        // create_user akun baru, lalu email aktivasinya.
        let job = env.claim().await;
        assert_eq!(
            env.worker.handle(job).await,
            JobOutcome::Completed(LeaseOutcome::Applied)
        );
    }

    // (d) Percobaan habis setelah POST berhasil (Keycloak mati): akun dihapus lunak, niat delete_orphan dicatat;
    // pengguna yatim yang sempat berkredensial TIDAK dihapus.
    for (tag, credentialed) in [("yatim-kredensial", true), ("yatim", false)] {
        let mail = email(tag);
        let p = env.register(&mail, IntentPurpose::HrInvitation).await;
        let orphan = match env
            .admin
            .create_user(IdpRealm::Korporat, &mail, p.intent_id, &SystemClock)
            .await
            .unwrap()
        {
            CreateOutcome::Created(id) => id,
            CreateOutcome::Conflict => panic!("bentrok"),
        };
        if credentialed {
            env.kc.edit(&orphan, |u| u.creds = vec!["password".into()]);
        }
        // Percobaan terakhir job create.
        let max = sqlx::query_scalar::<_, i16>(
            "UPDATE app.job SET attempt_count = max_attempts - 1 \
             WHERE job_type = 'idp_create_user' AND status = 'queued' AND payload->>'intent_id' = $1 \
             RETURNING max_attempts",
        )
        .bind(p.intent_id.hyphenated().to_string())
        .fetch_one(&env.pool)
        .await
        .unwrap();
        assert!(max > 1);
        env.kc.down.store(true, Ordering::SeqCst);
        let job = env.claim().await;
        assert_eq!(job.attempt(), job.max_attempts());
        let out = env.worker.handle(job).await;
        env.kc.down.store(false, Ordering::SeqCst);
        assert!(
            matches!(out, JobOutcome::FailedInTx(LeaseOutcome::Applied, ref c) if c.as_str() == "retries_exhausted")
        );
        assert_eq!(
            env.account(p.account_id).await.2.as_deref(),
            Some("idp_provisioning_failed")
        );
        let comp = env.follow_up(p.intent_id, "delete_orphan").await.unwrap();
        let deletes_before = env.kc.count("DELETE", USERS);
        let job = env.claim().await;
        assert_eq!(job.payload().unwrap().id("intent_id"), Some(comp));
        let out = env.worker.handle(job).await;
        if credentialed {
            assert!(
                matches!(out, JobOutcome::FailedInTx(_, ref c) if c.as_str() == "idp_user_credentialed")
            );
            assert_eq!(env.kc.count("DELETE", USERS), deletes_before);
            assert!(env.kc.user(&orphan).is_some());
        } else {
            assert_eq!(out, JobOutcome::Completed(LeaseOutcome::Applied));
            assert_eq!(env.kc.count("DELETE", USERS), deletes_before + 1);
            assert!(env.kc.user(&orphan).is_none());
            assert_eq!(env.intent_status(comp).await, "succeeded");
            let events = env.audit(p.account_id).await;
            assert_eq!(events.len(), 2);
            for e in &events {
                assert_event_complete(e, "e_10");
            }
            assert_eq!(events[0]["action_name"], "idp_user_provisioning_failed");
            assert_eq!(events[1]["action_name"], "idp_orphan_deleted");
        }
    }
    env.assert_no_email_in_job_or_intent(&mail3).await;
}

// trace: ADR-030 butir 3 (prasyarat lapis kedua di database aplikasi), review T1 ADR-030 catatan 2
#[tokio::test]
async fn disable_requires_inactive_account_and_intent_realm_must_match_account() {
    let _g = SERIAL.lock().await;
    let env = start().await;
    let p = env
        .register(&email("nonaktif"), IntentPurpose::HrInvitation)
        .await;
    let kc = env.provision(&p).await;
    env.cancel_queued_idp_jobs().await;

    // (a) NEG: akun masih pending_activation (belum inactive) -> niat disable ditolak tanpa panggilan Keycloak.
    let calls_before = env.kc.calls().len();
    let refused = env
        .record(
            p.account_id,
            IdpRealm::Korporat,
            IdpJobKind::DisableUser,
            IntentPurpose::AccountDeactivation,
        )
        .await
        .unwrap();
    let job = env.claim().await;
    let job_id = job.id().as_uuid();
    let out = env.worker.handle(job).await;
    assert!(
        matches!(out, JobOutcome::FailedInTx(LeaseOutcome::Applied, ref c) if c.as_str() == "account_state"),
        "{out:?}"
    );
    assert_eq!(env.kc.calls().len(), calls_before, "{:?}", env.kc.calls());
    assert!(env.kc.user(&kc).unwrap().enabled);
    assert_eq!(
        env.status_events(refused).await.last().unwrap(),
        &("rejected".to_string(), Some("account_state".to_string()))
    );
    assert_eq!(
        env.job_error(job_id).await,
        ("failed".to_string(), Some("account_state".to_string()))
    );
    let audit = env.audit(p.account_id).await;
    let last = audit.last().unwrap();
    assert_event_complete(last, "e_10");
    assert_eq!(last["action_name"], "idp_user_disable_failed");
    assert_eq!(last["outcome_reason_code"], "account_state");

    // (b) NEG: niat realm korporat untuk akun realm pasien ditolak database (FK komposit realm); tidak ada niat
    // maupun job yang tersisa.
    let patient = new_id();
    sqlx::query(
        "INSERT INTO app.account (id, account_type, idp_realm, idp_subject, email, status, status_changed_at) \
         VALUES ($1, 'patient', 'pasien', $2, $3, 'active', now())",
    )
    .bind(patient)
    .bind(new_id().hyphenated().to_string())
    .bind(email("pasien"))
    .execute(&env.pool)
    .await
    .unwrap();
    let err = env
        .record(
            patient,
            IdpRealm::Korporat,
            IdpJobKind::DisableUser,
            IntentPurpose::AccountDeactivation,
        )
        .await
        .unwrap_err();
    assert!(
        matches!(err, IntentError::Repo(IntentRepoError::Database(Some(ref s))) if s == "23503"),
        "{err}"
    );
    let left: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM app.idp_provisioning_intent WHERE account_id = $1",
    )
    .bind(patient)
    .fetch_one(&env.pool)
    .await
    .unwrap();
    assert_eq!(left, 0);
    assert!(
        env.claim_as(&WorkerId::new("idp_worker").unwrap(), &env.claim_clock)
            .await
            .is_none()
    );
    assert_eq!(env.kc.calls().len(), calls_before);

    // (c) Akun sudah inactive di database aplikasi: pengguna Keycloak dinonaktifkan lalu sesinya diakhiri.
    sqlx::query(
        "UPDATE app.account SET status = 'inactive', status_changed_at = now() WHERE id = $1",
    )
    .bind(p.account_id)
    .execute(&env.pool)
    .await
    .unwrap();
    env.record(
        p.account_id,
        IdpRealm::Korporat,
        IdpJobKind::DisableUser,
        IntentPurpose::AccountDeactivation,
    )
    .await
    .unwrap();
    let job = env.claim().await;
    let out = env.worker.handle(job).await;
    assert_eq!(out, JobOutcome::Completed(LeaseOutcome::Applied));
    assert!(!env.kc.user(&kc).unwrap().enabled);
    assert_eq!(env.kc.count("POST", "/logout"), 1, "{:?}", env.kc.calls());
}

// trace: ADR-030 butir 3 (daftar tertutup), butir 2 (muatan hanya ID niat)
#[tokio::test]
async fn unknown_job_kinds_and_foreign_payloads_fail_without_keycloak_calls() {
    let _g = SERIAL.lock().await;
    let env = start().await;
    let p = env
        .register(&email("asing"), IntentPurpose::HrSelfRegistration)
        .await;
    // Job create sah milik niat ini disingkirkan dulu supaya hanya job uji yang jatuh tempo.
    sqlx::query(
        "UPDATE app.job SET status = 'cancelled', finished_at = now() \
         WHERE job_type LIKE 'idp\\_%' AND status = 'queued'",
    )
    .execute(&env.pool)
    .await
    .unwrap();

    let raw = |job_type: &'static str, payload: Value| {
        let pool = env.pool.clone();
        async move {
            sqlx::query_scalar::<_, Uuid>(
                "INSERT INTO app.job (id, job_type, payload, run_at, status, attempt_count, max_attempts) \
                 VALUES ($1, $2, $3, now(), 'queued', 0, 3) RETURNING id",
            )
            .bind(new_id())
            .bind(job_type)
            .bind(payload)
            .fetch_one(&pool)
            .await
            .unwrap()
        }
    };
    let intent = p.intent_id.hyphenated().to_string();
    let unknown = raw("idp_reset_password", json!({"intent_id": intent})).await;
    let with_email = raw(
        "idp_create_user",
        json!({"intent_id": intent, "email": "hr@contoh.invalid"}),
    )
    .await;
    let with_actions = raw(
        "idp_send_activation",
        json!({"intent_id": intent, "actions": "update_password"}),
    )
    .await;
    let mismatch = raw("idp_delete_orphan", json!({"intent_id": intent})).await;
    let missing = raw(
        "idp_create_user",
        json!({"intent_id": new_id().hyphenated().to_string()}),
    )
    .await;

    // Putaran worker sungguhan (dengan jam sistem; job dibuat jatuh tempo `now()`).
    let report = env.worker.round().await.unwrap();
    assert_eq!(report.claimed, 4, "{report:?}");
    assert_eq!(report.failed, 4, "{report:?}");
    assert!(env.kc.calls().is_empty(), "{:?}", env.kc.calls());
    assert_eq!(
        env.job_error(unknown).await,
        ("failed".to_string(), Some("unknown_job_type".to_string()))
    );
    for (id, code) in [
        (with_email, "invalid_payload"),
        (with_actions, "invalid_payload"),
        (mismatch, "job_kind_mismatch"),
        (missing, "intent_not_found"),
    ] {
        assert_eq!(
            env.job_error(id).await,
            ("failed".to_string(), Some(code.to_string())),
            "{code}"
        );
    }
    // Niat sah tidak tersentuh oleh job palsu.
    assert_eq!(env.status_events(p.intent_id).await.len(), 1);
    assert_eq!(env.intent_status(p.intent_id).await, "pending");
    // Job palsu (salah satunya memuat email) dibuang supaya pemeriksaan "tanpa email di app.job" test lain bersih.
    sqlx::query("DELETE FROM app.job WHERE id = ANY($1)")
        .bind(vec![unknown, with_email, with_actions, mismatch, missing])
        .execute(&env.pool)
        .await
        .unwrap();
}
