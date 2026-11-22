//! F1-T65 (T1; US-208 AC1–AC7, klarifikasi T-19, ADR-011, ADR-012 butir 7, ADR-009 butir 2): akun dan
//! sub-peran admin Ruang Sehat terhadap PostgreSQL 18 sekali pakai (`scripts/db-ephemeral.sh`, role runtime
//! `rs_api`). Tanpa variabel `RS_TEST_DB_*` test GAGAL.
//!
//! Keycloak di test ini adalah tiruan dalam proses dari [`AdminIdentityProvider`] yang mencatat panggilan
//! (buat, undang, nonaktifkan, hapus) dan dapat disetel gagal; perilaku Admin REST Keycloak sungguhan untuk
//! klien `rs-backend-admin-manager` diuji `scripts/keycloak-check.sh` dan klien HTTP-nya di
//! `identity_admin_keycloak_http.rs`. Persetujuan Control Group OpenBao sungguhan diuji di
//! `crates/crypto/tests/crypto_openbao_control_group.rs`; di sini persetujuan yang sudah diverifikasi
//! diberikan langsung.
//!
//! Sensus admin bersifat global (seluruh akun `rs_admin` di database), sehingga test di berkas ini berjalan
//! satu per satu dan setiap test mulai dengan menandai akun admin sisa test lain sebagai terhapus (soft
//! delete, hak `UPDATE` milik `rs_app`).
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

mod audit_support;

use audit::{AuditContext, Source, SourceComponent};
use authz::AdminSubRole;
use foundation::{Clock, CorrelationId, Sensitive, SystemClock, Uuid, new_id};
use observability::EnvSource;
use rs_backend::modules::identity::api::{
    ACTIVATION_INVITE_LIFESPAN_SECONDS, AccountChangeListener, AdminAccounts, AdminActor,
    AdminIdentityProvider, AdminRuleViolation, BootstrapApproval, IdentityError, IdpFailure,
    IdpFuture, InviteDelivery, NewAdminInput,
};
use serde_json::Value;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use std::collections::BTreeSet;
use std::future::Future;
use std::net::IpAddr;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use AdminSubRole::*;

static SERIAL: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

async fn pool(var: &str) -> PgPool {
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

/// Panggilan yang tercatat di Keycloak tiruan.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Call {
    Create(String),
    Invite(String, u32),
    Disable(String),
    Delete(String),
}

type Hook = Box<dyn Fn() -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;

#[derive(Default)]
struct FakeIdp {
    calls: Mutex<Vec<Call>>,
    seq: AtomicUsize,
    conflict: AtomicBool,
    fail_invite: AtomicBool,
    fail_disable: AtomicBool,
    create_delay: Mutex<Option<Duration>>,
    on_create: Mutex<Option<Hook>>,
}

impl FakeIdp {
    fn calls(&self) -> Vec<Call> {
        self.calls.lock().unwrap().clone()
    }
    fn record(&self, c: Call) {
        self.calls.lock().unwrap().push(c);
    }
    fn creates(&self) -> usize {
        self.calls()
            .iter()
            .filter(|c| matches!(c, Call::Create(_)))
            .count()
    }
}

impl AdminIdentityProvider for FakeIdp {
    fn create_user<'a>(&'a self, email: &'a str) -> IdpFuture<'a, String> {
        Box::pin(async move {
            let delay = *self.create_delay.lock().unwrap();
            if let Some(d) = delay {
                tokio::time::sleep(d).await;
            }
            let hook = self.on_create.lock().unwrap().take();
            if let Some(h) = hook {
                h().await;
            }
            if self.conflict.load(Ordering::SeqCst) {
                return Err(IdpFailure::Conflict);
            }
            let n = self.seq.fetch_add(1, Ordering::SeqCst);
            let subject = format!("kc-{}-{n}", new_id().simple());
            self.record(Call::Create(email.to_string()));
            Ok(subject)
        })
    }
    fn send_activation<'a>(&'a self, subject: &'a str, lifespan: u32) -> IdpFuture<'a, ()> {
        Box::pin(async move {
            if self.fail_invite.load(Ordering::SeqCst) {
                return Err(IdpFailure::Unavailable);
            }
            self.record(Call::Invite(subject.to_string(), lifespan));
            Ok(())
        })
    }
    fn disable_user<'a>(&'a self, subject: &'a str) -> IdpFuture<'a, ()> {
        Box::pin(async move {
            if self.fail_disable.load(Ordering::SeqCst) {
                return Err(IdpFailure::Unavailable);
            }
            self.record(Call::Disable(subject.to_string()));
            Ok(())
        })
    }
    fn delete_user<'a>(&'a self, subject: &'a str) -> IdpFuture<'a, ()> {
        Box::pin(async move {
            self.record(Call::Delete(subject.to_string()));
            Ok(())
        })
    }
}

#[derive(Default)]
struct Cache(Mutex<Vec<String>>);

impl AccountChangeListener for Cache {
    fn account_changed<'a>(
        &'a self,
        subject: &'a str,
    ) -> Pin<Box<dyn Future<Output = bool> + Send + 'a>> {
        Box::pin(async move {
            self.0.lock().unwrap().push(subject.to_string());
            true
        })
    }
}

struct Env {
    app: PgPool,
    relay: PgPool,
    idp: Arc<FakeIdp>,
    cache: Arc<Cache>,
    svc: Arc<AdminAccounts>,
}

async fn env() -> Env {
    let app = pool("RS_TEST_DB_APP_URL").await;
    let relay = pool("RS_TEST_DB_RELAY_URL").await;
    // Akun admin sisa test lain dikeluarkan dari sensus (soft delete; data uji sintetis).
    sqlx::query!(
        "UPDATE app.account SET deleted_at = now(), delete_reason = 'uji_f1_t65_reset' \
         WHERE account_type = 'rs_admin' AND deleted_at IS NULL"
    )
    .execute(&app)
    .await
    .unwrap();
    let idp = Arc::new(FakeIdp::default());
    let cache = Arc::new(Cache::default());
    let clock: Arc<dyn Clock> = Arc::new(SystemClock);
    let svc = Arc::new(
        AdminAccounts::new(app.clone(), idp.clone(), cache.clone(), clock)
            .with_staff_nik_index(Arc::new(staff_nik_keyring())),
    );
    Env {
        app,
        relay,
        idp,
        cache,
        svc,
    }
}

/// Kunci blind index NIK uji (versi 1, nilai tetap; data sintetis).
fn staff_nik_keyring() -> crypto::NikBlindIndexKeyring {
    crypto::NikBlindIndexKeyring::new(vec![(
        1,
        crypto::secret_bytes(vec![7u8; crypto::BLIND_INDEX_KEY_LEN]),
    )])
    .unwrap()
}

/// NIK staf sintetis berawalan `99` (bukan kode wilayah nyata).
const STAFF_NIK: &str = "9900000000000007";

async fn staff_nik_stored(app: &PgPool, id: Uuid) -> Option<(Vec<u8>, i16)> {
    let r = sqlx::query!(
        "SELECT staff_nik_bidx, staff_nik_bidx_key_version FROM app.account WHERE id = $1",
        id
    )
    .fetch_one(app)
    .await
    .unwrap();
    r.staff_nik_bidx.zip(r.staff_nik_bidx_key_version)
}

fn ctx() -> AuditContext {
    AuditContext::new(
        CorrelationId::new(),
        Source::component(SourceComponent::Backoffice).ip(IpAddr::from([198, 51, 100, 7])),
    )
}

fn actor(id: Uuid) -> AdminActor {
    AdminActor {
        account_id: id,
        context: ctx(),
    }
}

fn roles(r: &[AdminSubRole]) -> BTreeSet<AdminSubRole> {
    r.iter().copied().collect()
}

fn approval() -> BootstrapApproval {
    BootstrapApproval {
        operator_entity_id: Uuid::from_u128(0x0a),
        approver_entity_ids: vec![Uuid::from_u128(0x0b)],
    }
}

/// Akun admin sintetis langsung di database (data uji, email domain contoh RFC 2606).
async fn seed(app: &PgPool, status: &str, r: &[AdminSubRole]) -> Uuid {
    let id = new_id();
    let subject = format!("kc-seed-{}", id.simple());
    sqlx::query!(
        "INSERT INTO app.account (id, account_type, idp_realm, idp_subject, email, status, status_changed_at, full_name) \
         VALUES ($1, 'rs_admin', 'admin', $2, $3, $4, now(), 'Admin Uji')",
        id,
        subject,
        format!("{subject}@example.com"),
        status,
    )
    .execute(app)
    .await
    .unwrap();
    for role in r {
        sqlx::query!(
            "INSERT INTO app.rs_admin_role (id, account_id, sub_role) VALUES ($1, $2, $3)",
            new_id(),
            id,
            role.as_str()
        )
        .execute(app)
        .await
        .unwrap();
    }
    id
}

async fn status(app: &PgPool, id: Uuid) -> String {
    sqlx::query_scalar!("SELECT status FROM app.account WHERE id = $1", id)
        .fetch_one(app)
        .await
        .unwrap()
}

async fn active_roles(app: &PgPool, id: Uuid) -> Vec<String> {
    sqlx::query_scalar!(
        "SELECT sub_role FROM app.rs_admin_role WHERE account_id = $1 AND deleted_at IS NULL ORDER BY sub_role",
        id
    )
    .fetch_all(app)
    .await
    .unwrap()
}

async fn admin_count(app: &PgPool) -> i64 {
    sqlx::query_scalar!(
        r#"SELECT count(*) AS "n!" FROM app.account WHERE account_type = 'rs_admin' AND deleted_at IS NULL"#
    )
    .fetch_one(app)
    .await
    .unwrap()
}

/// Event audit untuk objek `id`, urut waktu.
async fn events(relay: &PgPool, id: Uuid) -> Vec<Value> {
    let all: Vec<Value> =
        sqlx::query_scalar!("SELECT payload FROM app.audit_outbox ORDER BY occurred_at, id")
            .fetch_all(relay)
            .await
            .unwrap();
    let id = id.to_string();
    all.into_iter()
        .filter(|p| p["object_id"].as_str() == Some(id.as_str()))
        .collect()
}

fn actions(evs: &[Value]) -> Vec<(String, String)> {
    evs.iter()
        .map(|e| {
            (
                e["event_code"].as_str().unwrap().to_string(),
                e["action_name"].as_str().unwrap().to_string(),
            )
        })
        .collect()
}

fn assert_no_personal_data(evs: &[Value], email: &str, name: &str) {
    let text = serde_json::to_string(evs).unwrap();
    assert!(!text.contains(email), "email bocor ke audit");
    assert!(!text.contains(name), "nama bocor ke audit");
    assert!(!text.contains("kc-"), "subjek Keycloak tidak dicatat");
}

fn rule(e: &IdentityError) -> Option<AdminRuleViolation> {
    match e {
        IdentityError::AdminRule(v) => Some(*v),
        _ => None,
    }
}

fn field_code(e: &IdentityError) -> Option<String> {
    match e {
        IdentityError::Invalid(f) => f.first().map(|x| x.code.clone()),
        _ => None,
    }
}

// ---------------------------------------------------------------- AC4 bootstrap

#[tokio::test(flavor = "multi_thread")]
async fn bootstrap_creates_one_pending_super_admin_with_invite_and_audit() {
    let _g = SERIAL.lock().await;
    let e = env().await;
    let email = "calon.super@example.com";
    let name = "Calon Super Admin Uji";
    let created = e
        .svc
        .bootstrap(approval(), " Calon.Super@Example.com ", name, &ctx())
        .await
        .unwrap();
    assert_eq!(created.invite, InviteDelivery::Sent);
    assert_eq!(
        status(&e.app, created.account_id).await,
        "pending_activation"
    );
    assert_eq!(
        active_roles(&e.app, created.account_id).await,
        ["super_admin"]
    );
    let calls = e.idp.calls();
    assert_eq!(calls.first(), Some(&Call::Create(email.to_string())));
    assert!(
        matches!(calls.get(1), Some(Call::Invite(_, l)) if *l == ACTIVATION_INVITE_LIFESPAN_SECONDS)
    );
    assert_eq!(ACTIVATION_INVITE_LIFESPAN_SECONDS, 86_400);

    let evs = events(&e.relay, created.account_id).await;
    audit_support::assert_events_complete(&evs, &["e_10", "e_11"]);
    assert_eq!(
        actions(&evs),
        [
            ("e_10".into(), "admin_account_bootstrapped".into()),
            ("e_11".into(), "sub_role_granted".into()),
            ("e_10".into(), "activation_invite_requested".into()),
        ]
    );
    for ev in &evs {
        assert_eq!(ev["actor_type"], "system", "aktor sistem (bootstrap)");
        assert_eq!(
            ev["after_values"]["bootstrap_operator_entity_id"],
            Uuid::from_u128(0x0a).to_string()
        );
        assert_eq!(
            ev["after_values"]["bootstrap_approver_1_entity_id"],
            Uuid::from_u128(0x0b).to_string()
        );
    }
    assert_eq!(evs[0]["after_values"]["status"], "pending_activation");
    assert_eq!(evs[1]["after_values"]["sub_role"], "super_admin");
    assert_eq!(evs[1]["after_values"]["sensitive_grant"], true);
    assert_no_personal_data(&evs, email, name);
}

#[tokio::test(flavor = "multi_thread")]
async fn bootstrap_is_rejected_without_change_when_super_admin_active_or_pending() {
    let _g = SERIAL.lock().await;
    for st in ["active", "pending_activation"] {
        let e = env().await;
        seed(&e.app, st, &[SuperAdmin]).await;
        let before = admin_count(&e.app).await;
        let err = e
            .svc
            .bootstrap(approval(), "lain@example.com", "Orang Lain", &ctx())
            .await
            .unwrap_err();
        assert_eq!(
            rule(&err),
            Some(AdminRuleViolation::SuperAdminExists),
            "{st}"
        );
        assert_eq!(admin_count(&e.app).await, before);
        assert!(e.idp.calls().is_empty(), "Keycloak tidak disentuh ({st})");
    }
    // Super Admin nonaktif atau admin lain tidak menghalangi bootstrap.
    let e = env().await;
    seed(&e.app, "inactive", &[SuperAdmin]).await;
    seed(&e.app, "active", &[Verifier]).await;
    assert!(
        e.svc
            .bootstrap(approval(), "baru@example.com", "Baru", &ctx())
            .await
            .is_ok()
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn concurrent_bootstraps_create_exactly_one_account_and_compensate_the_loser() {
    let _g = SERIAL.lock().await;
    let e = env().await;
    // Keduanya lolos pemeriksaan awal sebelum salah satu commit.
    *e.idp.create_delay.lock().unwrap() = Some(Duration::from_millis(300));
    let (c1, c2) = (ctx(), ctx());
    let (a, b) = tokio::join!(
        e.svc.bootstrap(approval(), "satu@example.com", "Satu", &c1),
        e.svc.bootstrap(approval(), "dua@example.com", "Dua", &c2),
    );
    let ok = [a.is_ok(), b.is_ok()].iter().filter(|x| **x).count();
    assert_eq!(ok, 1, "{a:?} {b:?}");
    let err = a.err().or(b.err()).unwrap();
    assert_eq!(rule(&err), Some(AdminRuleViolation::SuperAdminExists));
    assert_eq!(admin_count(&e.app).await, 1);
    assert_eq!(e.idp.creates(), 2);
    let deletes = e
        .idp
        .calls()
        .iter()
        .filter(|c| matches!(c, Call::Delete(_)))
        .count();
    assert_eq!(deletes, 1, "pengguna Keycloak pihak yang kalah dihapus");
}

#[tokio::test(flavor = "multi_thread")]
async fn bootstrap_reissue_only_for_single_pending_super_admin() {
    let _g = SERIAL.lock().await;
    let e = env().await;
    e.idp.fail_invite.store(true, Ordering::SeqCst);
    let created = e
        .svc
        .bootstrap(
            approval(),
            "kedaluwarsa@example.com",
            "Undangan Gagal",
            &ctx(),
        )
        .await
        .unwrap();
    assert_eq!(created.invite, InviteDelivery::Failed);
    assert_eq!(
        status(&e.app, created.account_id).await,
        "pending_activation"
    );
    // F1-T69 (US-1501 AC1, AC3): undangan dicatat sebelum dikirim; kegagalan Keycloak dicatat sebagai aksi
    // yang sama dengan hasil gagal.
    let evs = events(&e.relay, created.account_id).await;
    let invites: Vec<_> = evs
        .iter()
        .filter(|p| p["action_name"] == "activation_invite_requested")
        .collect();
    assert_eq!(invites.len(), 2, "{invites:?}");
    assert_eq!(invites[0]["outcome"], "success");
    assert_eq!(invites[1]["outcome"], "failure");
    assert_eq!(
        invites[1]["outcome_reason_code"],
        "identity_provider_unavailable"
    );
    audit_support::assert_events_complete(&evs, &["e_10", "e_11"]);
    // Perintah bootstrap biasa tetap ditolak karena ada Super Admin menunggu aktivasi.
    let err = e
        .svc
        .bootstrap(approval(), "lain@example.com", "Lain", &ctx())
        .await
        .unwrap_err();
    assert_eq!(rule(&err), Some(AdminRuleViolation::SuperAdminExists));

    e.idp.fail_invite.store(false, Ordering::SeqCst);
    let (target, invite) = e.svc.bootstrap_reissue(approval(), &ctx()).await.unwrap();
    assert_eq!((target, invite), (created.account_id, InviteDelivery::Sent));
    let evs = events(&e.relay, created.account_id).await;
    assert_eq!(
        actions(&evs).last(),
        Some(&("e_10".into(), "activation_invite_requested".into()))
    );

    // Setelah ada Super Admin aktif, undangan ulang lewat bootstrap ditolak.
    seed(&e.app, "active", &[SuperAdmin]).await;
    let err = e
        .svc
        .bootstrap_reissue(approval(), &ctx())
        .await
        .unwrap_err();
    assert_eq!(
        rule(&err),
        Some(AdminRuleViolation::NoPendingBootstrapAccount)
    );
    let e2 = env().await;
    let err = e2
        .svc
        .bootstrap_reissue(approval(), &ctx())
        .await
        .unwrap_err();
    assert_eq!(
        rule(&err),
        Some(AdminRuleViolation::NoPendingBootstrapAccount)
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn bootstrap_without_distinct_second_party_or_with_bad_input_is_rejected_before_keycloak() {
    let _g = SERIAL.lock().await;
    let e = env().await;
    let op = Uuid::from_u128(0x0a);
    for approvers in [vec![], vec![op], vec![Uuid::from_u128(1); 4]] {
        let a = BootstrapApproval {
            operator_entity_id: op,
            approver_entity_ids: approvers,
        };
        let err = e
            .svc
            .bootstrap(a, "x@example.com", "X", &ctx())
            .await
            .unwrap_err();
        assert_eq!(field_code(&err).as_deref(), Some("no_second_party"));
    }
    let err = e
        .svc
        .bootstrap(approval(), "bukan-email", "", &ctx())
        .await
        .unwrap_err();
    assert!(matches!(err, IdentityError::Invalid(ref f) if f.len() == 2));
    assert!(e.idp.calls().is_empty());
    assert_eq!(admin_count(&e.app).await, 0);
}

// ---------------------------------------------------------------- AC1, AC6 buat akun

#[tokio::test(flavor = "multi_thread")]
async fn super_admin_creates_admin_with_sub_roles_and_audit() {
    let _g = SERIAL.lock().await;
    let e = env().await;
    let sa = seed(&e.app, "active", &[SuperAdmin]).await;
    let created = e
        .svc
        .create_admin(
            &actor(sa),
            NewAdminInput {
                email: "verifikator@example.com".into(),
                full_name: "Verifikator Uji".into(),
                sub_roles: roles(&[Verifier, CustomerService]),
                staff_nik: Some(Sensitive::new(STAFF_NIK.to_string())),
            },
        )
        .await
        .unwrap();
    assert_eq!(
        status(&e.app, created.account_id).await,
        "pending_activation"
    );
    assert_eq!(
        active_roles(&e.app, created.account_id).await,
        ["customer_service", "verifier"]
    );
    let evs = events(&e.relay, created.account_id).await;
    audit_support::assert_events_complete(&evs, &["e_10", "e_11"]);
    assert_eq!(
        actions(&evs),
        [
            ("e_10".into(), "admin_account_created".into()),
            ("e_11".into(), "sub_role_granted".into()),
            ("e_11".into(), "sub_role_granted".into()),
            ("e_10".into(), "activation_invite_requested".into()),
        ]
    );
    for ev in &evs {
        assert_eq!(ev["actor_account_id"], sa.to_string());
        assert_eq!(ev["actor_roles"], serde_json::json!(["super_admin"]));
    }
    let sensitive: Vec<_> = evs
        .iter()
        .filter(|e| e["action_name"] == "sub_role_granted")
        .map(|e| {
            (
                e["after_values"]["sub_role"].as_str().unwrap().to_string(),
                e["after_values"]["sensitive_grant"].as_bool().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        sensitive,
        [
            ("verifier".into(), true),
            ("customer_service".into(), false)
        ]
    );
    assert_no_personal_data(&evs, "verifikator@example.com", "Verifikator Uji");
    // US-201 AC8 (KEPUTUSAN-294): NIK staf tersimpan sebagai blind index versi terbaru saja; E-11 hanya
    // mencatat bahwa NIK staf terdaftar.
    let expected = staff_nik_keyring()
        .compute(&crypto::Nik::parse(STAFF_NIK).unwrap())
        .unwrap();
    assert_eq!(
        staff_nik_stored(&e.app, created.account_id).await,
        Some((expected.value.to_vec(), 1))
    );
    let verifier_grant = evs
        .iter()
        .find(|e| e["after_values"]["sub_role"] == "verifier")
        .unwrap();
    assert!(
        verifier_grant["changed_fields"]
            .as_array()
            .unwrap()
            .iter()
            .any(|f| f == "staff_nik")
    );
    assert!(verifier_grant["after_values"].get("staff_nik").is_none());
    let raw = serde_json::to_string(&evs).unwrap();
    assert!(!raw.contains(STAFF_NIK) && !raw.contains(&STAFF_NIK[12..]));
}

// trace: US-201 AC8, KEPUTUSAN-294 (NIK staf wajib saat Verifikator diberikan, format 16 digit, dikosongkan
// saat Verifikator dicabut)
#[tokio::test(flavor = "multi_thread")]
async fn verifier_grant_requires_staff_nik() {
    let _g = SERIAL.lock().await;
    let e = env().await;
    let sa = seed(&e.app, "active", &[SuperAdmin]).await;
    let input = |nik: Option<&str>| NewAdminInput {
        email: "verifikator2@example.com".into(),
        full_name: "Verifikator Dua".into(),
        sub_roles: roles(&[Verifier]),
        staff_nik: nik.map(|n| Sensitive::new(n.to_string())),
    };
    let before = admin_count(&e.app).await;
    for (nik, code) in [(None, "required"), (Some("12345"), "invalid_format")] {
        let err = e
            .svc
            .create_admin(&actor(sa), input(nik))
            .await
            .unwrap_err();
        assert_eq!(field_code(&err).as_deref(), Some(code), "{nik:?}");
    }
    assert!(
        e.idp.calls().is_empty(),
        "ditolak sebelum Keycloak disentuh"
    );
    assert_eq!(admin_count(&e.app).await, before);

    let staff = seed(&e.app, "active", &[Content]).await;
    let err = e
        .svc
        .change_sub_roles(&actor(sa), staff, roles(&[Content, Verifier]))
        .await
        .unwrap_err();
    assert_eq!(field_code(&err).as_deref(), Some("required"));
    assert_eq!(active_roles(&e.app, staff).await, ["content"]);
    e.svc
        .change_sub_roles_with_staff_nik(
            &actor(sa),
            staff,
            roles(&[Content, Verifier]),
            Some(Sensitive::new(STAFF_NIK.to_string())),
        )
        .await
        .unwrap();
    assert!(staff_nik_stored(&e.app, staff).await.is_some());
    e.svc
        .change_sub_roles(&actor(sa), staff, roles(&[Content]))
        .await
        .unwrap();
    assert_eq!(
        staff_nik_stored(&e.app, staff).await,
        None,
        "pencabutan Verifikator mengosongkan NIK staf"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn creation_is_rejected_for_non_super_admin_auditor_combination_and_duplicate_email() {
    let _g = SERIAL.lock().await;
    let e = env().await;
    let sa = seed(&e.app, "active", &[SuperAdmin]).await;
    let input = |r: &[AdminSubRole]| NewAdminInput {
        email: "baru@example.com".into(),
        full_name: "Baru".into(),
        sub_roles: roles(r),
        staff_nik: None,
    };
    for (who, st) in [
        (
            seed(&e.app, "active", &[Verifier, Finance]).await,
            "bukan SA",
        ),
        (
            seed(&e.app, "pending_activation", &[SuperAdmin]).await,
            "SA menunggu",
        ),
        (seed(&e.app, "inactive", &[SuperAdmin]).await, "SA nonaktif"),
        (new_id(), "tidak ada"),
    ] {
        let err = e
            .svc
            .create_admin(&actor(who), input(&[Content]))
            .await
            .unwrap_err();
        assert_eq!(
            rule(&err),
            Some(AdminRuleViolation::ActorNotSuperAdmin),
            "{st}"
        );
    }
    let err = e
        .svc
        .create_admin(&actor(sa), input(&[Auditor, Content]))
        .await
        .unwrap_err();
    assert_eq!(field_code(&err).as_deref(), Some("auditor_exclusive"));
    let err = e
        .svc
        .create_admin(&actor(sa), input(&[]))
        .await
        .unwrap_err();
    assert_eq!(field_code(&err).as_deref(), Some("required"));
    assert!(e.idp.calls().is_empty(), "ditolak sebelum Keycloak");

    e.idp.conflict.store(true, Ordering::SeqCst);
    let err = e
        .svc
        .create_admin(&actor(sa), input(&[Content]))
        .await
        .unwrap_err();
    assert_eq!(field_code(&err).as_deref(), Some("already_registered"));
    // Super Admin + tiga akun pelaku yang ditolak; tidak ada akun baru.
    assert_eq!(admin_count(&e.app).await, 4);
}

#[tokio::test(flavor = "multi_thread")]
async fn rejection_after_keycloak_user_creation_compensates_and_changes_nothing() {
    let _g = SERIAL.lock().await;
    let e = env().await;
    let sa = seed(&e.app, "active", &[SuperAdmin]).await;
    seed(&e.app, "active", &[SuperAdmin]).await;
    // Saat pengguna Keycloak dibuat, Super Admin pelaku dinonaktifkan orang lain.
    let app = e.app.clone();
    *e.idp.on_create.lock().unwrap() = Some(Box::new(move || {
        let app = app.clone();
        Box::pin(async move {
            sqlx::query!(
                "UPDATE app.account SET status = 'inactive' WHERE id = $1",
                sa
            )
            .execute(&app)
            .await
            .unwrap();
        })
    }));
    let before = admin_count(&e.app).await;
    let err = e
        .svc
        .create_admin(
            &actor(sa),
            NewAdminInput {
                email: "telat@example.com".into(),
                full_name: "Telat".into(),
                sub_roles: roles(&[Content]),
                staff_nik: None,
            },
        )
        .await
        .unwrap_err();
    assert_eq!(rule(&err), Some(AdminRuleViolation::ActorNotSuperAdmin));
    assert_eq!(admin_count(&e.app).await, before);
    let calls = e.idp.calls();
    assert!(
        matches!(calls.as_slice(), [Call::Create(_), Call::Delete(_)]),
        "{calls:?}"
    );
}

// ---------------------------------------------------------------- AC2, AC3, AC6, AC7 sub-peran

#[tokio::test(flavor = "multi_thread")]
async fn sub_role_change_rules_and_audit() {
    let _g = SERIAL.lock().await;
    let e = env().await;
    let sa = seed(&e.app, "active", &[SuperAdmin]).await;
    let staff = seed(&e.app, "active", &[Verifier]).await;
    let auditor = seed(&e.app, "active", &[Auditor]).await;

    // AC3: sub-peran sendiri.
    let err = e
        .svc
        .change_sub_roles(&actor(sa), sa, roles(&[SuperAdmin, Content]))
        .await
        .unwrap_err();
    assert_eq!(rule(&err), Some(AdminRuleViolation::SelfRoleChange));
    // AC6 dua arah.
    let err = e
        .svc
        .change_sub_roles(&actor(sa), staff, roles(&[Verifier, Auditor]))
        .await
        .unwrap_err();
    assert_eq!(field_code(&err).as_deref(), Some("auditor_exclusive"));
    let err = e
        .svc
        .change_sub_roles(&actor(sa), auditor, roles(&[Auditor, Finance]))
        .await
        .unwrap_err();
    assert_eq!(field_code(&err).as_deref(), Some("auditor_exclusive"));
    // AC7: Auditor aktif terakhir.
    let err = e
        .svc
        .change_sub_roles(&actor(sa), auditor, roles(&[Content]))
        .await
        .unwrap_err();
    assert_eq!(rule(&err), Some(AdminRuleViolation::LastAuditor));
    // Bukan Super Admin.
    let err = e
        .svc
        .change_sub_roles(&actor(staff), auditor, roles(&[Content]))
        .await
        .unwrap_err();
    assert_eq!(rule(&err), Some(AdminRuleViolation::ActorNotSuperAdmin));
    // Sasaran tidak ada.
    let err = e
        .svc
        .change_sub_roles(&actor(sa), new_id(), roles(&[Content]))
        .await
        .unwrap_err();
    assert_eq!(err, IdentityError::NotFound);
    assert_eq!(active_roles(&e.app, staff).await, ["verifier"]);
    assert_eq!(active_roles(&e.app, auditor).await, ["auditor"]);
    assert!(
        events(&e.relay, staff).await.is_empty(),
        "penolakan tanpa perubahan"
    );

    // Sukses: Verifikator → Keuangan + Konten.
    let diff = e
        .svc
        .change_sub_roles(&actor(sa), staff, roles(&[Finance, Content]))
        .await
        .unwrap();
    assert_eq!(diff.granted, roles(&[Finance, Content]));
    assert_eq!(diff.revoked, roles(&[Verifier]));
    assert_eq!(active_roles(&e.app, staff).await, ["content", "finance"]);
    let evs = events(&e.relay, staff).await;
    audit_support::assert_events_complete(&evs, &["e_11"]);
    assert_eq!(
        actions(&evs),
        [
            ("e_11".into(), "sub_role_revoked".into()),
            ("e_11".into(), "sub_role_granted".into()),
            ("e_11".into(), "sub_role_granted".into()),
        ]
    );
    assert_eq!(evs[0]["before_values"]["sub_role"], "verifier");
    let revoked_by: Option<Uuid> = sqlx::query_scalar!(
        "SELECT deleted_by FROM app.rs_admin_role WHERE account_id = $1 AND sub_role = 'verifier'",
        staff
    )
    .fetch_one(&e.app)
    .await
    .unwrap();
    assert_eq!(
        revoked_by,
        Some(sa),
        "soft delete atas nama pelaku (ADR-003)"
    );
    assert_eq!(e.cache.0.lock().unwrap().len(), 1, "cache akun disegarkan");

    // Auditor kedua ada: Auditor pertama boleh diganti.
    seed(&e.app, "active", &[Auditor]).await;
    assert!(
        e.svc
            .change_sub_roles(&actor(sa), auditor, roles(&[Content]))
            .await
            .is_ok()
    );
}

// ---------------------------------------------------------------- AC2, AC5, AC7 penonaktifan

#[tokio::test(flavor = "multi_thread")]
async fn deactivation_rules_audit_and_keycloak_sync() {
    let _g = SERIAL.lock().await;
    let e = env().await;
    let sa = seed(&e.app, "active", &[SuperAdmin]).await;
    let auditor = seed(&e.app, "active", &[Auditor]).await;
    let staff = seed(&e.app, "active", &[Content]).await;

    let err = e
        .svc
        .deactivate_admin(&actor(sa), sa, "keluar")
        .await
        .unwrap_err();
    assert_eq!(rule(&err), Some(AdminRuleViolation::SelfDeactivation));
    let err = e
        .svc
        .deactivate_admin(&actor(sa), auditor, "pindah")
        .await
        .unwrap_err();
    assert_eq!(rule(&err), Some(AdminRuleViolation::LastAuditor));
    let err = e
        .svc
        .deactivate_admin(&actor(sa), staff, "  ")
        .await
        .unwrap_err();
    assert_eq!(field_code(&err).as_deref(), Some("invalid"));
    assert_eq!(status(&e.app, sa).await, "active");
    assert_eq!(status(&e.app, auditor).await, "active");

    let out = e
        .svc
        .deactivate_admin(&actor(sa), staff, "Kontrak berakhir")
        .await
        .unwrap();
    assert!(out.identity_provider_synced);
    assert_eq!(status(&e.app, staff).await, "inactive");
    let reason: Option<String> =
        sqlx::query_scalar!("SELECT status_reason FROM app.account WHERE id = $1", staff)
            .fetch_one(&e.app)
            .await
            .unwrap();
    assert_eq!(reason.as_deref(), Some("Kontrak berakhir"));
    let evs = events(&e.relay, staff).await;
    assert_eq!(
        actions(&evs),
        [("e_10".into(), "admin_account_deactivated".into())]
    );
    audit_support::assert_events_complete(&evs, &["e_10"]);
    assert_eq!(evs[0]["before_values"]["status"], "active");
    assert_eq!(evs[0]["after_values"]["status"], "inactive");
    assert!(
        !serde_json::to_string(&evs).unwrap().contains("Kontrak"),
        "alasan hanya nama field"
    );
    assert!(matches!(e.idp.calls().last(), Some(Call::Disable(_))));
    // Sudah nonaktif.
    let err = e
        .svc
        .deactivate_admin(&actor(sa), staff, "lagi")
        .await
        .unwrap_err();
    assert_eq!(rule(&err), Some(AdminRuleViolation::TargetInactive));

    // Keycloak gagal: status di PostgreSQL tetap nonaktif (akses backend tertutup), hasil melaporkan.
    let other = seed(&e.app, "active", &[Finance]).await;
    e.idp.fail_disable.store(true, Ordering::SeqCst);
    let out = e
        .svc
        .deactivate_admin(&actor(sa), other, "Uji Keycloak gagal")
        .await
        .unwrap();
    assert!(!out.identity_provider_synced);
    assert_eq!(status(&e.app, other).await, "inactive");
}

#[tokio::test(flavor = "multi_thread")]
async fn two_super_admins_deactivating_each_other_concurrently_leave_one_active() {
    let _g = SERIAL.lock().await;
    for _ in 0..5 {
        let e = env().await;
        let a = seed(&e.app, "active", &[SuperAdmin]).await;
        let b = seed(&e.app, "active", &[SuperAdmin]).await;
        let (act_a, act_b) = (actor(a), actor(b));
        let (ra, rb) = tokio::join!(
            e.svc.deactivate_admin(&act_a, b, "saling"),
            e.svc.deactivate_admin(&act_b, a, "saling"),
        );
        let ok = [ra.is_ok(), rb.is_ok()].iter().filter(|x| **x).count();
        assert_eq!(ok, 1, "{ra:?} {rb:?}");
        let loser = ra.err().or(rb.err()).unwrap();
        assert_eq!(rule(&loser), Some(AdminRuleViolation::ActorNotSuperAdmin));
        let active: i64 = sqlx::query_scalar!(
            r#"SELECT count(*) AS "n!" FROM app.account WHERE id = ANY($1) AND status = 'active'"#,
            &[a, b][..]
        )
        .fetch_one(&e.app)
        .await
        .unwrap();
        assert_eq!(active, 1, "Super Admin aktif tidak pernah habis");
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn concurrent_revocation_of_two_auditors_keeps_one() {
    let _g = SERIAL.lock().await;
    for _ in 0..5 {
        let e = env().await;
        let s1 = seed(&e.app, "active", &[SuperAdmin]).await;
        let s2 = seed(&e.app, "active", &[SuperAdmin]).await;
        let a1 = seed(&e.app, "active", &[Auditor]).await;
        let a2 = seed(&e.app, "active", &[Auditor]).await;
        let (act1, act2) = (actor(s1), actor(s2));
        let (r1, r2) = tokio::join!(
            e.svc.change_sub_roles(&act1, a1, roles(&[Content])),
            e.svc.deactivate_admin(&act2, a2, "bersamaan"),
        );
        let ok = [r1.is_ok(), r2.is_ok()].iter().filter(|x| **x).count();
        assert_eq!(ok, 1);
        let err = r1.err().or(r2.err()).unwrap();
        assert_eq!(rule(&err), Some(AdminRuleViolation::LastAuditor));
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn resend_invite_only_for_pending_accounts_by_super_admin() {
    let _g = SERIAL.lock().await;
    let e = env().await;
    let sa = seed(&e.app, "active", &[SuperAdmin]).await;
    let pending = seed(&e.app, "pending_activation", &[Content]).await;
    let active = seed(&e.app, "active", &[Content]).await;
    assert_eq!(
        e.svc.resend_invite(&actor(sa), pending).await.unwrap(),
        InviteDelivery::Sent
    );
    let evs = events(&e.relay, pending).await;
    assert_eq!(
        actions(&evs),
        [("e_10".into(), "activation_invite_requested".into())]
    );
    audit_support::assert_events_complete(&evs, &["e_10"]);
    let err = e.svc.resend_invite(&actor(sa), active).await.unwrap_err();
    assert_eq!(rule(&err), Some(AdminRuleViolation::TargetInactive));
    let err = e
        .svc
        .resend_invite(&actor(active), pending)
        .await
        .unwrap_err();
    assert_eq!(rule(&err), Some(AdminRuleViolation::ActorNotSuperAdmin));
    let err = e.svc.resend_invite(&actor(sa), new_id()).await.unwrap_err();
    assert_eq!(err, IdentityError::NotFound);
}
