//! Use case akun dan sub-peran admin Ruang Sehat (F1-T65, US-208 AC1–AC7, klarifikasi T-19, ADR-011,
//! ADR-012 butir 7). Semua perubahan lewat `audit::in_audited_tx` (E-10, E-11) setelah kunci tabel
//! `rs_admin_role` ([`admin_repo::lock_admin_roles`]).
//!
//! Urutan dengan Keycloak (tidak ada panggilan HTTP selama transaksi terbuka, KEPUTUSAN-125):
//! - **Buat akun** (Super Admin atau bootstrap): (1) pemeriksaan awal tanpa kunci supaya permintaan yang
//!   pasti ditolak tidak menyentuh Keycloak; (2) pengguna Keycloak dibuat (aktif, tanpa kredensial: tidak
//!   dapat masuk sebelum undangan dijalankan); (3) transaksi berkunci memeriksa ulang aturan, menyisipkan
//!   akun `pending_activation` + sub-peran, dan menulis E-10/E-11; bila ditolak atau gagal, pengguna
//!   Keycloak dihapus (kompensasi); (4) setelah commit, email undangan dikirim (sekali pakai, 24 jam) dan
//!   E-10 `activation_invite_sent` dicatat. Undangan yang gagal terkirim dilaporkan di hasil
//!   ([`InviteDelivery::Failed`]) dan dikirim ulang lewat [`AdminAccounts::resend_invite`] atau, untuk
//!   bootstrap, [`AdminAccounts::bootstrap_reissue`].
//! - **Nonaktifkan / ubah sub-peran**: transaksi berkunci (aturan + perubahan + audit), lalu setelah commit
//!   cache akun disegarkan dan (penonaktifan) pengguna Keycloak dinonaktifkan beserta sesinya. Akses backend
//!   sudah tertutup oleh status di PostgreSQL (resolver akun, KU-10); kegagalan Keycloak dilaporkan di hasil
//!   untuk dicoba ulang dan dicatat di log galat.
//!
//! Aktor bootstrap = "sistem (bootstrap)": `Actor::system()`, aksi `admin_account_bootstrapped`, ditambah
//! ID entitas OpenBao operator dan penyetuju dari Control Group (US-208 AC4). Pemberian sub-peran sensitif
//! (AC8) ditandai `sensitive_grant` di E-11; pemberitahuan ke Super Admin dan Auditor menyusul bersama kanal
//! notifikasi admin US-1003 (Fase 2).
//!
//! NIK staf Verifikator (US-201 AC8, KEPUTUSAN-294 = jawaban user PT-57 opsi A): setiap kali sub-peran
//! Verifikator diberikan (akun baru atau perubahan sub-peran), NIK staf wajib dan disimpan sebagai blind
//! index saja (versi kunci terbaru, `account.staff_nik_bidx`) di transaksi yang sama dengan pemberian
//! sub-peran; E-11 hanya mencatat nama field `staff_nik`, tidak pernah nilainya. Pencabutan Verifikator
//! mengosongkan blind index itu (data minimal). Modul `profile` mencocokkannya lewat [`staff_nik_index`]
//! sebelum Verifikator memutuskan pengajuan identitas.

use super::admin_domain::{
    self, ACTIVATION_INVITE_LIFESPAN_SECONDS, AdminCensus, AdminRuleViolation, RoleDiff,
};
use super::admin_repo::{self, NewAdmin};
use super::domain::IdpRealm;
use super::error::IdentityError;
use super::keycloak::{KeycloakClient, KeycloakError};
use super::service::AccountDirectory;
use audit::{
    Actor, AuditContext, AuditEvent, AuditedTxError, Code, EventCode, FieldChange, FieldRule,
    ObjectPolicy, ObjectRef, Outcome, Value, in_audited_tx,
};
use authn::Realm;
use authz::{AccountStatus, AdminSubRole};
use crypto::{Nik, NikBlindIndexKeyring};
use foundation::{Clock, Sensitive, Uuid, new_id};
use sqlx::{PgConnection, PgPool};
use std::collections::BTreeSet;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use time::OffsetDateTime;

/// Future operasi penyedia identitas.
pub type IdpFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, IdpFailure>> + Send + 'a>>;

/// Kegagalan penyedia identitas (tanpa isi respons).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdpFailure {
    /// Username/email sudah dipakai.
    Conflict,
    /// Penyedia menolak atau tidak tersedia.
    Unavailable,
}

/// Operasi pengguna realm `admin` di Keycloak yang dibutuhkan use case ini (implementasi produksi:
/// [`super::keycloak::KeycloakClient`] dengan klien layanan `rs-backend-admin-manager`).
pub trait AdminIdentityProvider: Send + Sync {
    /// Membuat pengguna tanpa kredensial; mengembalikan ID pengguna (`account.idp_subject`).
    fn create_user<'a>(&'a self, email: &'a str) -> IdpFuture<'a, String>;
    /// Mengirim undangan aktivasi sekali pakai berlaku `lifespan_seconds`.
    fn send_activation<'a>(&'a self, subject: &'a str, lifespan_seconds: u32) -> IdpFuture<'a, ()>;
    /// Menonaktifkan pengguna dan mengakhiri semua sesinya.
    fn disable_user<'a>(&'a self, subject: &'a str) -> IdpFuture<'a, ()>;
    /// Menghapus pengguna (kompensasi).
    fn delete_user<'a>(&'a self, subject: &'a str) -> IdpFuture<'a, ()>;
}

/// Dipanggil setelah commit perubahan status/sub-peran akun (penyegaran cache akun, ADR-011 butir 8).
pub trait AccountChangeListener: Send + Sync {
    /// Akun `subject` (realm `admin`) berubah.
    fn account_changed<'a>(
        &'a self,
        subject: &'a str,
    ) -> Pin<Box<dyn Future<Output = bool> + Send + 'a>>;
}

/// Implementasi produksi [`AdminIdentityProvider`]: Admin REST Keycloak realm `admin` dengan klien layanan
/// `rs-backend-admin-manager` (hanya `manage-users`).
pub struct KeycloakAdminUsers {
    client: Arc<KeycloakClient>,
    clock: Arc<dyn Clock>,
}

impl KeycloakAdminUsers {
    /// `client` wajib memegang kredensial realm `admin`.
    pub fn new(client: Arc<KeycloakClient>, clock: Arc<dyn Clock>) -> Self {
        Self { client, clock }
    }
}

fn idp_failure(e: KeycloakError) -> IdpFailure {
    tracing::warn!(error = %e, "keycloak_admin_call_failed");
    match e {
        KeycloakError::Conflict => IdpFailure::Conflict,
        _ => IdpFailure::Unavailable,
    }
}

impl AdminIdentityProvider for KeycloakAdminUsers {
    fn create_user<'a>(&'a self, email: &'a str) -> IdpFuture<'a, String> {
        Box::pin(async move {
            self.client
                .create_user(IdpRealm::Admin, email, self.clock.as_ref())
                .await
                .map_err(idp_failure)
        })
    }

    fn send_activation<'a>(&'a self, subject: &'a str, lifespan_seconds: u32) -> IdpFuture<'a, ()> {
        Box::pin(async move {
            self.client
                .send_activation_email(
                    IdpRealm::Admin,
                    subject,
                    lifespan_seconds,
                    self.clock.as_ref(),
                )
                .await
                .map_err(idp_failure)
        })
    }

    fn disable_user<'a>(&'a self, subject: &'a str) -> IdpFuture<'a, ()> {
        Box::pin(async move {
            self.client
                .disable_user(IdpRealm::Admin, subject, self.clock.as_ref())
                .await
                .map_err(idp_failure)
        })
    }

    fn delete_user<'a>(&'a self, subject: &'a str) -> IdpFuture<'a, ()> {
        Box::pin(async move {
            self.client
                .delete_user(IdpRealm::Admin, subject, self.clock.as_ref())
                .await
                .map_err(idp_failure)
        })
    }
}

impl AccountChangeListener for AccountDirectory {
    fn account_changed<'a>(
        &'a self,
        subject: &'a str,
    ) -> Pin<Box<dyn Future<Output = bool> + Send + 'a>> {
        Box::pin(async move { self.refresh_account(Realm::Admin, subject).await.is_ok() })
    }
}

/// Tanpa cache (proses tanpa Valkey, misalnya perintah bootstrap).
pub struct NoAccountCache;

impl AccountChangeListener for NoAccountCache {
    fn account_changed<'a>(
        &'a self,
        _subject: &'a str,
    ) -> Pin<Box<dyn Future<Output = bool> + Send + 'a>> {
        Box::pin(async { true })
    }
}

/// Persetujuan orang kedua yang sudah diverifikasi lewat Control Group OpenBao (dibangun perintah
/// bootstrap dari `crypto::verify_control_group_approval`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BootstrapApproval {
    /// Entitas OpenBao operator.
    pub operator_entity_id: Uuid,
    /// Entitas OpenBao penyetuju (1..=3, berbeda dari operator).
    pub approver_entity_ids: Vec<Uuid>,
}

/// Admin pelaku (Super Admin) dan konteks audit permintaannya. Hak Super Admin dibaca ulang dari database
/// di dalam transaksi berkunci (M-13), bukan dipercaya dari token/cache.
#[derive(Debug, Clone)]
pub struct AdminActor {
    /// `account.id` pelaku.
    pub account_id: Uuid,
    /// Konteks audit (korelasi, sumber).
    pub context: AuditContext,
}

/// Masukan pembuatan akun admin.
#[derive(Debug, Clone)]
pub struct NewAdminInput {
    /// Email (username Keycloak).
    pub email: String,
    /// Nama lengkap.
    pub full_name: String,
    /// Sub-peran.
    pub sub_roles: BTreeSet<AdminSubRole>,
    /// NIK staf 16 digit; wajib bila `sub_roles` memuat Verifikator (US-201 AC8, KEPUTUSAN-294), diabaikan
    /// bila tidak. Hanya blind index-nya yang disimpan.
    pub staff_nik: Option<Sensitive<String>>,
}

/// Blind index NIK staf akun admin (US-201 AC8, KEPUTUSAN-294) untuk dicocokkan modul `profile`.
#[derive(Clone, PartialEq, Eq)]
pub struct StaffNikIndex {
    /// Blind index HMAC 32 byte.
    pub bidx: Vec<u8>,
    /// Versi kunci blind index.
    pub key_version: i16,
}

impl std::fmt::Debug for StaffNikIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StaffNikIndex")
            .field("key_version", &self.key_version)
            .finish_non_exhaustive()
    }
}

/// Blind index NIK staf akun admin `account_id`; `None` bila akun tidak ada atau belum punya NIK staf
/// (Verifikator tanpa NIK staf tidak boleh memutuskan pengajuan identitas: gagal-tertutup di pemanggil).
pub async fn staff_nik_index(
    pool: &PgPool,
    account_id: Uuid,
) -> Result<Option<StaffNikIndex>, IdentityError> {
    admin_repo::staff_nik(pool, account_id)
        .await
        .map(|r| r.map(|(bidx, key_version)| StaffNikIndex { bidx, key_version }))
        .map_err(|_| IdentityError::Database)
}

/// Status pengiriman undangan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InviteDelivery {
    /// Terkirim (berlaku 24 jam, sekali pakai).
    Sent,
    /// Gagal terkirim; akun tetap menunggu aktivasi, undangan dikirim ulang.
    Failed,
}

/// Hasil pembuatan akun admin.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CreatedAdmin {
    /// `account.id`.
    pub account_id: Uuid,
    /// Status undangan.
    pub invite: InviteDelivery,
}

/// Hasil penonaktifan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Deactivated {
    /// Pengguna Keycloak sudah dinonaktifkan dan sesinya diakhiri.
    pub identity_provider_synced: bool,
}

/// Kebijakan audit objek akun admin (E-10, E-11). Email dan nama tidak dicatat; alasan penonaktifan hanya
/// nama field-nya.
static ADMIN_ACCOUNT_POLICY: ObjectPolicy = ObjectPolicy::non_clinical(
    "account",
    &[
        FieldRule::value("status"),
        FieldRule::value("account_type"),
        FieldRule::value("sub_role"),
        FieldRule::value("sensitive_grant"),
        FieldRule::name_only("staff_nik"),
        FieldRule::name_only("status_reason"),
        FieldRule::value("invite_expires_at"),
        FieldRule::value("bootstrap_operator_entity_id"),
        FieldRule::value("bootstrap_approver_1_entity_id"),
        FieldRule::value("bootstrap_approver_2_entity_id"),
        FieldRule::value("bootstrap_approver_3_entity_id"),
    ],
);

/// Alasan pencabutan sub-peran (`rs_admin_role.delete_reason`).
const ROLE_REVOKE_REASON: &str = "sub_role_revoked_by_super_admin";

/// Use case akun admin.
pub struct AdminAccounts {
    pool: PgPool,
    idp: Arc<dyn AdminIdentityProvider>,
    cache: Arc<dyn AccountChangeListener>,
    clock: Arc<dyn Clock>,
    staff_nik_index: Option<Arc<NikBlindIndexKeyring>>,
}

enum Work {
    Rule(AdminRuleViolation),
    StaffNikRequired,
    NotFound,
    Db,
    Audit,
}

impl From<sqlx::Error> for Work {
    fn from(_: sqlx::Error) -> Self {
        Self::Db
    }
}

fn map_tx(e: AuditedTxError<Work>) -> IdentityError {
    match e {
        AuditedTxError::Work(Work::Rule(v)) => v.into(),
        AuditedTxError::Work(Work::NotFound) => IdentityError::NotFound,
        AuditedTxError::Work(Work::StaffNikRequired) => staff_nik_required(),
        AuditedTxError::Work(Work::Db) | AuditedTxError::Database { .. } => IdentityError::Database,
        AuditedTxError::Work(Work::Audit) | AuditedTxError::Audit(_) => IdentityError::Audit,
    }
}

fn code(s: &str) -> Result<Code, Work> {
    Code::new(s).map_err(|_| Work::Audit)
}

fn role_codes(roles: &BTreeSet<AdminSubRole>) -> Result<Vec<Code>, Work> {
    roles.iter().map(|r| code(r.as_str())).collect()
}

async fn census(conn: &mut PgConnection) -> Result<AdminCensus, Work> {
    admin_repo::load_census(conn).await?.ok_or(Work::Db)
}

/// Aktor audit admin dari sensus (sub-peran saat aksi).
fn admin_actor(census: &AdminCensus, id: Uuid) -> Result<Actor, Work> {
    let roles = census
        .get(id)
        .map(|a| a.sub_roles.clone())
        .unwrap_or_default();
    Ok(Actor::account(id, code("rs_admin")?, role_codes(&roles)?))
}

fn event(
    ev: EventCode,
    action: &str,
    actor: &Actor,
    target: Uuid,
    ctx: &AuditContext,
) -> Result<AuditEvent, Work> {
    Ok(AuditEvent::new(
        ev,
        code(action)?,
        actor.clone(),
        ObjectRef::with_uuid(&ADMIN_ACCOUNT_POLICY, target),
    )
    .outcome(Outcome::success())
    .context(ctx))
}

fn role_value(r: AdminSubRole) -> Result<Value, Work> {
    Value::code(r.as_str()).map_err(|_| Work::Audit)
}

fn status_value(s: AccountStatus) -> Result<Value, Work> {
    Value::code(s.as_str()).map_err(|_| Work::Audit)
}

fn grant_event(
    actor: &Actor,
    target: Uuid,
    role: AdminSubRole,
    ctx: &AuditContext,
) -> Result<AuditEvent, Work> {
    let mut e = event(EventCode::E11, "sub_role_granted", actor, target, ctx)?
        .change(FieldChange::new("sub_role", None, Some(role_value(role)?)))
        .change(FieldChange::new(
            "sensitive_grant",
            None,
            Some(Value::Bool(admin_domain::is_sensitive(role))),
        ));
    if role == AdminSubRole::Verifier {
        // US-201 AC8: hanya nama field (NIK staf terdaftar), tanpa nilai apa pun (PRD §7.3).
        e = e.change(FieldChange::new("staff_nik", None, Some(Value::Bool(true))));
    }
    Ok(e)
}

fn staff_nik_required() -> IdentityError {
    IdentityError::field(
        "staff_nik",
        "required",
        "NIK staf 16 digit wajib untuk sub-peran Verifikator (pemisahan tugas verifikasi identitas)",
    )
}

/// Blind index NIK staf siap disimpan.
#[derive(Clone)]
struct StaffNik {
    bidx: Vec<u8>,
    key_version: i16,
}

fn revoke_event(
    actor: &Actor,
    target: Uuid,
    role: AdminSubRole,
    ctx: &AuditContext,
) -> Result<AuditEvent, Work> {
    Ok(
        event(EventCode::E11, "sub_role_revoked", actor, target, ctx)?.change(FieldChange::new(
            "sub_role",
            Some(role_value(role)?),
            None,
        )),
    )
}

fn bootstrap_fields(mut e: AuditEvent, approval: &BootstrapApproval) -> AuditEvent {
    e = e.change(FieldChange::new(
        "bootstrap_operator_entity_id",
        None,
        Some(Value::Uuid(approval.operator_entity_id)),
    ));
    for (i, id) in approval.approver_entity_ids.iter().enumerate() {
        let field = match i {
            0 => "bootstrap_approver_1_entity_id",
            1 => "bootstrap_approver_2_entity_id",
            _ => "bootstrap_approver_3_entity_id",
        };
        e = e.change(FieldChange::new(field, None, Some(Value::Uuid(*id))));
    }
    e
}

/// Pembuat akun: Super Admin atau bootstrap.
#[derive(Clone)]
enum Creator {
    Admin(Uuid),
    Bootstrap(BootstrapApproval),
}

impl AdminAccounts {
    /// Use case dengan penyedia identitas dan pendengar perubahan akun.
    pub fn new(
        pool: PgPool,
        idp: Arc<dyn AdminIdentityProvider>,
        cache: Arc<dyn AccountChangeListener>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            pool,
            idp,
            cache,
            clock,
            staff_nik_index: None,
        }
    }

    /// Kunci blind index NIK (versi terbaru) untuk NIK staf Verifikator (US-201 AC8). Tanpa ini, pemberian
    /// sub-peran Verifikator ditolak (`503`, gagal-tertutup).
    #[must_use]
    pub fn with_staff_nik_index(mut self, index: Arc<NikBlindIndexKeyring>) -> Self {
        self.staff_nik_index = Some(index);
        self
    }

    /// Mengurai dan menghitung blind index NIK staf (tanpa I/O). `None` bila tidak diberikan.
    fn prepare_staff_nik(
        &self,
        raw: Option<&Sensitive<String>>,
    ) -> Result<Option<StaffNik>, IdentityError> {
        let Some(raw) = raw else { return Ok(None) };
        let nik = Nik::parse(raw.expose()).map_err(|_| {
            IdentityError::field("staff_nik", "invalid_format", "NIK harus 16 digit angka")
        })?;
        let Some(index) = &self.staff_nik_index else {
            tracing::error!("staff_nik_index_not_configured");
            return Err(IdentityError::Database);
        };
        let b = index.compute(&nik).map_err(|_| IdentityError::Database)?;
        Ok(Some(StaffNik {
            bidx: b.value.to_vec(),
            key_version: b.key_version,
        }))
    }

    async fn census_unlocked(&self) -> Result<AdminCensus, IdentityError> {
        let mut conn = self
            .pool
            .acquire()
            .await
            .map_err(|_| IdentityError::Database)?;
        admin_repo::load_census(&mut conn)
            .await
            .map_err(|_| IdentityError::Database)?
            .ok_or(IdentityError::Database)
    }

    /// US-208 AC1: Super Admin membuat akun admin dengan satu atau lebih sub-peran; akun menunggu aktivasi
    /// (password + TOTP lewat undangan, MFA wajib di realm `admin`).
    pub async fn create_admin(
        &self,
        actor: &AdminActor,
        input: NewAdminInput,
    ) -> Result<CreatedAdmin, IdentityError> {
        let (email, full_name) = validate_identity(&input.email, &input.full_name)?;
        let pre = self.census_unlocked().await?;
        admin_domain::check_creation(&pre, actor.account_id, &input.sub_roles)?;
        let staff_nik = if input.sub_roles.contains(&AdminSubRole::Verifier) {
            Some(
                self.prepare_staff_nik(input.staff_nik.as_ref())?
                    .ok_or_else(staff_nik_required)?,
            )
        } else {
            None
        };
        self.create(
            Creator::Admin(actor.account_id),
            &email,
            &full_name,
            input.sub_roles,
            staff_nik,
            &actor.context,
        )
        .await
    }

    /// US-208 AC4: bootstrap Super Admin pertama (perintah satu kali, persetujuan orang kedua sudah
    /// diverifikasi). Ditolak bila sudah ada Super Admin aktif atau menunggu aktivasi.
    pub async fn bootstrap(
        &self,
        approval: BootstrapApproval,
        email: &str,
        full_name: &str,
        context: &AuditContext,
    ) -> Result<CreatedAdmin, IdentityError> {
        validate_approval(&approval)?;
        let (email, full_name) = validate_identity(email, full_name)?;
        admin_domain::check_bootstrap(&self.census_unlocked().await?)?;
        let roles = BTreeSet::from([AdminSubRole::SuperAdmin]);
        self.create(
            Creator::Bootstrap(approval),
            &email,
            &full_name,
            roles,
            None,
            context,
        )
        .await
    }

    async fn create(
        &self,
        creator: Creator,
        email: &str,
        full_name: &str,
        roles: BTreeSet<AdminSubRole>,
        staff_nik: Option<StaffNik>,
        context: &AuditContext,
    ) -> Result<CreatedAdmin, IdentityError> {
        let subject = match self.idp.create_user(email).await {
            Ok(s) => s,
            Err(IdpFailure::Conflict) => {
                return Err(IdentityError::field(
                    "email",
                    "already_registered",
                    "Email sudah terdaftar di realm admin",
                ));
            }
            Err(IdpFailure::Unavailable) => return Err(IdentityError::IdentityProvider),
        };
        let account_id = new_id();
        let now = self.clock.now();
        let tx = {
            let subject = subject.clone();
            let email = email.to_string();
            let full_name = full_name.to_string();
            let ctx = context.clone();
            let creator = creator.clone();
            in_audited_tx(&self.pool, self.clock.as_ref(), move |conn| {
                Box::pin(async move {
                    admin_repo::lock_admin_roles(conn).await?;
                    let census = census(conn).await?;
                    let (actor, created_by, action) = match &creator {
                        Creator::Admin(id) => {
                            admin_domain::check_creation(&census, *id, &roles)
                                .map_err(Work::Rule)?;
                            (
                                admin_actor(&census, *id)?,
                                Some(*id),
                                "admin_account_created",
                            )
                        }
                        Creator::Bootstrap(_) => {
                            admin_domain::check_bootstrap(&census).map_err(Work::Rule)?;
                            (Actor::system(), None, "admin_account_bootstrapped")
                        }
                    };
                    admin_repo::insert_admin(
                        conn,
                        &NewAdmin {
                            id: account_id,
                            idp_subject: &subject,
                            email: &email,
                            full_name: &full_name,
                            created_by,
                            now,
                        },
                    )
                    .await?;
                    let mut created = event(EventCode::E10, action, &actor, account_id, &ctx)?
                        .change(FieldChange::new(
                            "account_type",
                            None,
                            Some(Value::code("rs_admin").map_err(|_| Work::Audit)?),
                        ))
                        .change(FieldChange::new(
                            "status",
                            None,
                            Some(status_value(AccountStatus::PendingActivation)?),
                        ));
                    if let Creator::Bootstrap(a) = &creator {
                        created = bootstrap_fields(created, a);
                    }
                    let mut events = vec![created];
                    if roles.contains(&AdminSubRole::Verifier) {
                        let n = staff_nik.as_ref().ok_or(Work::StaffNikRequired)?;
                        admin_repo::set_staff_nik(
                            conn,
                            account_id,
                            Some((&n.bidx, n.key_version)),
                            created_by,
                            now,
                        )
                        .await?;
                    }
                    for role in &roles {
                        admin_repo::grant_role(conn, new_id(), account_id, *role, created_by, now)
                            .await?;
                        let mut e = grant_event(&actor, account_id, *role, &ctx)?;
                        if let Creator::Bootstrap(a) = &creator {
                            e = bootstrap_fields(e, a);
                        }
                        events.push(e);
                    }
                    Ok(((), events))
                })
            })
            .await
        };
        if let Err(e) = tx {
            // Kompensasi: pengguna Keycloak tanpa akun aplikasi dihapus (tidak memberi akses apa pun, tetapi
            // menahan email dan mengganggu percobaan berikutnya).
            if self.idp.delete_user(&subject).await.is_err() {
                tracing::error!("admin_idp_user_compensation_failed");
            }
            return Err(map_tx(e));
        }
        let invite = self
            .send_invite(account_id, &subject, &creator, context)
            .await;
        Ok(CreatedAdmin { account_id, invite })
    }

    async fn send_invite(
        &self,
        account_id: Uuid,
        subject: &str,
        creator: &Creator,
        context: &AuditContext,
    ) -> InviteDelivery {
        if self
            .idp
            .send_activation(subject, ACTIVATION_INVITE_LIFESPAN_SECONDS)
            .await
            .is_err()
        {
            tracing::error!(account_id = %account_id, "admin_activation_invite_failed");
            return InviteDelivery::Failed;
        }
        let expires_at = self.clock.now()
            + time::Duration::seconds(i64::from(ACTIVATION_INVITE_LIFESPAN_SECONDS));
        let creator = creator.clone();
        let ctx = context.clone();
        let recorded = in_audited_tx(&self.pool, self.clock.as_ref(), move |conn| {
            Box::pin(async move {
                let actor = match &creator {
                    Creator::Admin(id) => admin_actor(&census(conn).await?, *id)?,
                    Creator::Bootstrap(_) => Actor::system(),
                };
                let mut e = event(
                    EventCode::E10,
                    "activation_invite_sent",
                    &actor,
                    account_id,
                    &ctx,
                )?
                .change(FieldChange::new(
                    "invite_expires_at",
                    None,
                    Some(Value::Timestamp(expires_at)),
                ));
                if let Creator::Bootstrap(a) = &creator {
                    e = bootstrap_fields(e, a);
                }
                Ok::<_, Work>(((), vec![e]))
            })
        })
        .await;
        if recorded.is_err() {
            // Undangan sudah terkirim; kegagalan catatan dilaporkan ke log galat (security log oleh pustaka).
            tracing::error!(account_id = %account_id, "admin_activation_invite_audit_failed");
        }
        InviteDelivery::Sent
    }

    /// Mengirim ulang undangan untuk akun admin yang masih menunggu aktivasi (undangan lama kedaluwarsa atau
    /// gagal terkirim). Hanya Super Admin aktif; bukan untuk akun sendiri.
    pub async fn resend_invite(
        &self,
        actor: &AdminActor,
        target: Uuid,
    ) -> Result<InviteDelivery, IdentityError> {
        let census = self.census_unlocked().await?;
        admin_domain::require_super_admin(&census, actor.account_id)?;
        match census.get(target) {
            None => return Err(IdentityError::NotFound),
            Some(a) if a.status != AccountStatus::PendingActivation => {
                return Err(AdminRuleViolation::TargetInactive.into());
            }
            Some(_) => {}
        }
        let subject = self.subject(target).await?;
        Ok(self
            .send_invite(
                target,
                &subject,
                &Creator::Admin(actor.account_id),
                &actor.context,
            )
            .await)
    }

    /// US-208 AC4 (undangan kedaluwarsa): perintah bootstrap yang sama, dengan persetujuan orang kedua baru,
    /// mengirim ulang undangan ke satu-satunya Super Admin yang menunggu aktivasi.
    pub async fn bootstrap_reissue(
        &self,
        approval: BootstrapApproval,
        context: &AuditContext,
    ) -> Result<(Uuid, InviteDelivery), IdentityError> {
        validate_approval(&approval)?;
        let target = admin_domain::bootstrap_reissue_target(&self.census_unlocked().await?)?;
        let subject = self.subject(target).await?;
        let invite = self
            .send_invite(target, &subject, &Creator::Bootstrap(approval), context)
            .await;
        Ok((target, invite))
    }

    async fn subject(&self, account_id: Uuid) -> Result<String, IdentityError> {
        let mut conn = self
            .pool
            .acquire()
            .await
            .map_err(|_| IdentityError::Database)?;
        admin_repo::admin_subject(&mut conn, account_id)
            .await
            .map_err(|_| IdentityError::Database)?
            .ok_or(IdentityError::NotFound)
    }

    /// US-208 AC1–AC3, AC6, AC7: Super Admin mengganti sub-peran akun admin lain menjadi `wanted`, tanpa NIK
    /// staf (pemberian Verifikator ditolak; pakai [`Self::change_sub_roles_with_staff_nik`]).
    pub async fn change_sub_roles(
        &self,
        actor: &AdminActor,
        target: Uuid,
        wanted: BTreeSet<AdminSubRole>,
    ) -> Result<RoleDiff, IdentityError> {
        self.change_sub_roles_with_staff_nik(actor, target, wanted, None)
            .await
    }

    /// Seperti [`Self::change_sub_roles`]; `staff_nik` wajib bila sub-peran Verifikator baru diberikan
    /// (US-201 AC8, KEPUTUSAN-294). Pencabutan Verifikator mengosongkan blind index NIK staf.
    pub async fn change_sub_roles_with_staff_nik(
        &self,
        actor: &AdminActor,
        target: Uuid,
        wanted: BTreeSet<AdminSubRole>,
        staff_nik: Option<Sensitive<String>>,
    ) -> Result<RoleDiff, IdentityError> {
        let staff_nik = if wanted.contains(&AdminSubRole::Verifier) {
            self.prepare_staff_nik(staff_nik.as_ref())?
        } else {
            None
        };
        let actor_id = actor.account_id;
        let ctx = actor.context.clone();
        let now = self.clock.now();
        let (diff, subject) = in_audited_tx(&self.pool, self.clock.as_ref(), move |conn| {
            Box::pin(async move {
                admin_repo::lock_admin_roles(conn).await?;
                let census = census(conn).await?;
                admin_domain::require_super_admin(&census, actor_id).map_err(Work::Rule)?;
                let account = census.get(target).cloned().ok_or(Work::NotFound)?;
                let diff = admin_domain::plan_role_change(&census, actor_id, &account, &wanted)
                    .map_err(Work::Rule)?;
                let actor = admin_actor(&census, actor_id)?;
                let mut events = Vec::new();
                if diff.granted.contains(&AdminSubRole::Verifier) {
                    let n = staff_nik.as_ref().ok_or(Work::StaffNikRequired)?;
                    admin_repo::set_staff_nik(
                        conn,
                        target,
                        Some((&n.bidx, n.key_version)),
                        Some(actor_id),
                        now,
                    )
                    .await?;
                } else if diff.revoked.contains(&AdminSubRole::Verifier) {
                    admin_repo::set_staff_nik(conn, target, None, Some(actor_id), now).await?;
                }
                for role in &diff.revoked {
                    admin_repo::revoke_role(conn, target, *role, actor_id, ROLE_REVOKE_REASON, now)
                        .await?;
                    events.push(revoke_event(&actor, target, *role, &ctx)?);
                }
                for role in &diff.granted {
                    admin_repo::grant_role(conn, new_id(), target, *role, Some(actor_id), now)
                        .await?;
                    events.push(grant_event(&actor, target, *role, &ctx)?);
                }
                let subject = admin_repo::admin_subject(conn, target)
                    .await?
                    .ok_or(Work::NotFound)?;
                Ok(((diff, subject), events))
            })
        })
        .await
        .map_err(map_tx)?;
        if !self.cache.account_changed(&subject).await {
            tracing::error!(account_id = %target, "admin_account_cache_refresh_failed");
        }
        Ok(diff)
    }

    /// US-208 AC2, AC5, AC7: Super Admin menonaktifkan akun admin lain dengan alasan.
    pub async fn deactivate_admin(
        &self,
        actor: &AdminActor,
        target: Uuid,
        reason: &str,
    ) -> Result<Deactivated, IdentityError> {
        let reason = admin_domain::normalize_reason(reason).ok_or_else(|| {
            IdentityError::field(
                "reason",
                "invalid",
                "Alasan wajib diisi, maksimal 500 karakter",
            )
        })?;
        let actor_id = actor.account_id;
        let ctx = actor.context.clone();
        let now = self.clock.now();
        let subject = in_audited_tx(&self.pool, self.clock.as_ref(), move |conn| {
            Box::pin(async move {
                admin_repo::lock_admin_roles(conn).await?;
                let census = census(conn).await?;
                admin_domain::require_super_admin(&census, actor_id).map_err(Work::Rule)?;
                let account = census.get(target).cloned().ok_or(Work::NotFound)?;
                admin_domain::check_deactivation(&census, actor_id, &account)
                    .map_err(Work::Rule)?;
                if !admin_repo::deactivate(conn, target, actor_id, &reason, now).await? {
                    return Err(Work::Rule(AdminRuleViolation::TargetInactive));
                }
                let actor = admin_actor(&census, actor_id)?;
                let e = event(
                    EventCode::E10,
                    "admin_account_deactivated",
                    &actor,
                    target,
                    &ctx,
                )?
                .change(FieldChange::new(
                    "status",
                    Some(status_value(account.status)?),
                    Some(status_value(AccountStatus::Inactive)?),
                ))
                .change(FieldChange::changed("status_reason"));
                let subject = admin_repo::admin_subject(conn, target)
                    .await?
                    .ok_or(Work::NotFound)?;
                Ok((subject, vec![e]))
            })
        })
        .await
        .map_err(map_tx)?;
        if !self.cache.account_changed(&subject).await {
            tracing::error!(account_id = %target, "admin_account_cache_refresh_failed");
        }
        let synced = self.idp.disable_user(&subject).await.is_ok();
        if !synced {
            tracing::error!(account_id = %target, "admin_idp_disable_failed");
        }
        Ok(Deactivated {
            identity_provider_synced: synced,
        })
    }
}

fn validate_identity(email: &str, full_name: &str) -> Result<(String, String), IdentityError> {
    let mut errors = Vec::new();
    let e = admin_domain::normalize_email(email);
    if e.is_none() {
        errors.push(envelope::FieldError::new(
            "email",
            "invalid_format",
            "Format email tidak valid",
        ));
    }
    let n = admin_domain::normalize_full_name(full_name);
    if n.is_none() {
        errors.push(envelope::FieldError::new(
            "full_name",
            "invalid",
            "Nama lengkap wajib diisi, maksimal 200 karakter",
        ));
    }
    match (e, n) {
        (Some(e), Some(n)) => Ok((e, n)),
        _ => Err(IdentityError::Invalid(errors)),
    }
}

fn validate_approval(a: &BootstrapApproval) -> Result<(), IdentityError> {
    let ok = (1..=3).contains(&a.approver_entity_ids.len())
        && !a.approver_entity_ids.contains(&a.operator_entity_id);
    if ok {
        Ok(())
    } else {
        Err(IdentityError::field(
            "approval",
            "no_second_party",
            "Persetujuan orang kedua tidak sah",
        ))
    }
}

/// Aktivasi pertama admin saat login berhasil pertama di realm `admin` (dipanggil pengambil event
/// Keycloak di transaksi teraudit event itu). Mengembalikan event E-10 `admin_account_activated` bila akun
/// berpindah dari `pending_activation` ke `active`.
pub(super) async fn activate_on_first_login(
    conn: &mut PgConnection,
    account_id: Uuid,
    roles: Vec<Code>,
    context: &AuditContext,
    at: OffsetDateTime,
) -> Result<Option<AuditEvent>, ActivationError> {
    if !admin_repo::activate_pending_admin(conn, account_id, at)
        .await
        .map_err(ActivationError::Db)?
    {
        return Ok(None);
    }
    let build = || -> Result<AuditEvent, Work> {
        let actor = Actor::account(account_id, code("rs_admin")?, roles);
        Ok(event(
            EventCode::E10,
            "admin_account_activated",
            &actor,
            account_id,
            context,
        )?
        .change(FieldChange::new(
            "status",
            Some(status_value(AccountStatus::PendingActivation)?),
            Some(status_value(AccountStatus::Active)?),
        )))
    };
    build().map(Some).map_err(|_| ActivationError::Audit)
}

/// Galat aktivasi pertama.
#[derive(Debug)]
pub(super) enum ActivationError {
    /// Query gagal.
    Db(sqlx::Error),
    /// Event tidak dapat disusun.
    Audit,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audit_policy_is_valid_and_has_no_personal_fields() {
        assert!(ADMIN_ACCOUNT_POLICY.validate().is_ok());
        for f in ADMIN_ACCOUNT_POLICY.fields() {
            assert!(!["email", "full_name"].contains(&f.name()), "{}", f.name());
        }
    }

    #[test]
    fn approval_needs_a_distinct_second_party() {
        let op = Uuid::from_u128(1);
        let ok = BootstrapApproval {
            operator_entity_id: op,
            approver_entity_ids: vec![Uuid::from_u128(2)],
        };
        assert!(validate_approval(&ok).is_ok());
        for bad in [
            vec![],
            vec![op],
            vec![Uuid::from_u128(2), op],
            vec![Uuid::from_u128(2); 4],
        ] {
            let a = BootstrapApproval {
                operator_entity_id: op,
                approver_entity_ids: bad,
            };
            assert!(validate_approval(&a).is_err());
        }
    }

    #[test]
    fn identity_validation_reports_each_field() {
        match validate_identity("bukan-email", " ") {
            Err(IdentityError::Invalid(e)) => {
                let fields: Vec<_> = e.iter().map(|x| x.field.as_str()).collect();
                assert_eq!(fields, ["email", "full_name"]);
            }
            other => panic!("{other:?}"),
        }
    }
}
