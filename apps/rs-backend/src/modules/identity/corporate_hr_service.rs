//! Use case kelola anggota HR perusahaan (F1-T107, T1; US-1904 AC2, AC5; ADR-027 butir 3, ADR-028 butir 1).
//!
//! **Nonaktifkan HR** (satu transaksi `audit::in_audited_tx`, KEPUTUSAN-125: tanpa panggilan HTTP selama
//! transaksi terbuka):
//! 1. IP klien wajib (field "Sumber" PRD §7.2, gagal-tertutup). Keanggotaan sasaran diputuskan lewat
//!    `authz::corporate_object` (perusahaan lain dan tidak ada = `404` yang sama dan tercatat di security
//!    log, KU-03).
//! 2. Di transaksi: `corporate::api::lock_members` (pola KEPUTUSAN-290), baca ulang daftar HR perusahaan
//!    pemanggil, aturan murni [`check_deactivation`] (pemanggil masih HR aktif, bukan diri sendiri
//!    `[ASUMSI]`, sasaran belum nonaktif, HR aktif terakhir dijaga), cabut keanggotaan (hapus lunak) dan
//!    ubah status akun ke `inactive`, lalu E-10 (`corporate_admin_account_deactivated`, status) dan E-11
//!    (`corporate_membership_revoked`, keanggotaan) dengan tenant = perusahaan. Penolakan aturan dicatat
//!    E-10 hasil gagal di transaksi tersendiri.
//! 3. Setelah commit: entri cache akun di Valkey ditimpa ([`AccountDirectory::refresh_account`]) dan
//!    pengguna Keycloak dinonaktifkan lewat [`CorporateIdentityProvisioner`].
//!
//! **Penolakan sesi seketika (US-1904 AC5, KU-10).** Permintaan berikutnya HR yang dinonaktifkan ditolak
//! oleh middleware `authz` tanpa menunggu Keycloak: keanggotaan aktif dibaca langsung dari PostgreSQL di
//! setiap permintaan endpoint HR (`CorporateDirectory`, tanpa cache) → `401` `4010001`; status akun
//! `inactive` → `401` `4010002` setelah cache akun ditimpa (paling lambat TTL cache bila penimpaan gagal).
//! Pencatatan `session_revocation` per sesi (F1-T33) tidak dipakai di sini: tabel itu dikunci per ID sesi
//! Keycloak, dan sesi milik HR **lain** tidak diketahui backend (hanya sesi pemanggil yang ada di token).
//! Pengakhiran sesi Keycloak menyusul lewat penyedia (butir 3).
//!
//! **Undang HR lain** (F1-T107 sisa, US-1904 AC2; ADR-030 butir 2, 5, 7; KEPUTUSAN-303 P30-2, KEPUTUSAN-329),
//! satu transaksi `audit::in_audited_tx`:
//! 1. Email divalidasi (ASCII ternormalisasi) sebelum transaksi; IP klien wajib (gagal-tertutup).
//! 2. Di transaksi, **kunci advisory email lebih dulu** (`register_hr_invitation` -> `register_pending_account`:
//!    akun `pending_provisioning` jenis `corporate_admin` + niat `create_user` + job `idp_create_user`), baru
//!    `corporate::api::lock_members` (kunci tabel; urutan tetap advisory -> tabel, tidak pernah sebaliknya,
//!    review T1 ADR-030 catatan 5), baca ulang daftar HR, [`check_invite`] (pemanggil masih HR aktif),
//!    tambah keanggotaan perusahaan **pemanggil** (cakupan selalu dari keanggotaan, tidak dari permintaan),
//!    lalu E-10 (`corporate_admin_account_invited`) dan E-11 (`corporate_membership_granted`) dengan tenant =
//!    perusahaan.
//! 3. Email sudah dipakai akun belum terhapus di realm `korporat`: tidak ada efek (transaksi dibatalkan),
//!    jawaban SERAGAM dengan undangan baru (`202`, tanpa ID), tercatat E-10 hasil gagal `email_in_use` di
//!    transaksi tersendiri (anti-enumerasi, P30-2). Pemberitahuan "Anda sudah punya akun" ke alamat itu
//!    BELUM ada: jalur email aplikasi (outbox, K2) belum dibangun `[TIDAK DIKETAHUI]`.
//! 4. Email aktivasi (password + TOTP, 24 jam, sekali pakai, KEPUTUSAN-306) dikirim worker IdP setelah pengguna
//!    dibuat (`idp_worker.rs` mencatat niat `idp_send_activation` sendiri); backend tidak punya jalur "undang
//!    ulang", jadi tidak ada akun atau niat ganda. Proses `api` tidak pernah memanggil Keycloak (TD-26/TD-30).
//!
//! **Penonaktifan pengguna Keycloak** belum lewat niat `idp_disable_user`: [`CorporateIdentityProvisioner`]
//! hanya antarmuka dan implementasi yang dipasang ([`ProvisionerNotConfigured`]) tidak memanggil Admin REST
//! (akses backend sudah tertutup oleh PostgreSQL, TD-38).

use super::corporate_hr_domain::{
    HrMember, HrRoster, HrRuleViolation, check_deactivation, check_invite,
};
use super::corporate_hr_invite::{HrInviteError, normalize_hr_email, register_hr_invitation};
use super::corporate_hr_repo;
use super::error::IdentityError;
use super::idp_service::IntentActor;
use super::service::AccountDirectory;
use crate::modules::corporate::api as corporate;
use audit::{
    Actor, AuditContext, AuditEvent, AuditedTxError, Code, EventCode, FieldChange, FieldRule,
    ObjectPolicy, ObjectRef, Outcome, Source, SourceComponent, Value, in_audited_tx,
};
use authn::Realm;
use authz::{AccountStatus, CorporateScope, corporate_object};
use foundation::{Clock, FixedClock, Uuid};
use sqlx::{PgConnection, PgPool};
use std::convert::Infallible;
use std::future::Future;
use std::net::IpAddr;
use std::pin::Pin;
use std::sync::Arc;

/// Future operasi penyedia identitas realm `korporat`.
pub type HrIdpFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, HrIdpFailure>> + Send + 'a>>;

/// Kegagalan penyedia identitas (tanpa isi respons).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HrIdpFailure {
    /// Username/email sudah dipakai.
    Conflict,
    /// Penyedia menolak, tidak tersedia, atau belum dipasang.
    Unavailable,
}

/// Operasi pengguna realm `korporat` yang dibutuhkan kelola anggota HR (US-1904 AC2). Dirancang supaya dapat
/// dijalankan proses lain (mis. pekerja khusus pemegang klien `manage-users`, ADR-030) lewat antrean, sehingga
/// proses `api` tidak memegang rahasia klien itu (TD-26/TD-30). Implementasi produksi belum ada.
pub trait CorporateIdentityProvisioner: Send + Sync {
    /// Membuat pengguna tanpa kredensial (undangan HR baru); mengembalikan ID pengguna (`account.idp_subject`).
    fn create_user<'a>(&'a self, email: &'a str) -> HrIdpFuture<'a, String>;
    /// Mengirim undangan aktivasi sekali pakai berlaku `lifespan_seconds` (verifikasi email, password, TOTP).
    fn send_activation<'a>(
        &'a self,
        subject: &'a str,
        lifespan_seconds: u32,
    ) -> HrIdpFuture<'a, ()>;
    /// Menonaktifkan pengguna dan mengakhiri semua sesinya.
    fn disable_user<'a>(&'a self, subject: &'a str) -> HrIdpFuture<'a, ()>;
}

/// Penyedia yang belum dipasang: tidak pernah memanggil Keycloak, selalu [`HrIdpFailure::Unavailable`].
/// Dipakai proses `api` sampai pekerja ADR-030 dipasang (lihat dokumentasi modul).
#[derive(Debug, Clone, Copy, Default)]
pub struct ProvisionerNotConfigured;

impl CorporateIdentityProvisioner for ProvisionerNotConfigured {
    fn create_user<'a>(&'a self, _email: &'a str) -> HrIdpFuture<'a, String> {
        Box::pin(async { Err(HrIdpFailure::Unavailable) })
    }

    fn send_activation<'a>(
        &'a self,
        _subject: &'a str,
        _lifespan_seconds: u32,
    ) -> HrIdpFuture<'a, ()> {
        Box::pin(async { Err(HrIdpFailure::Unavailable) })
    }

    fn disable_user<'a>(&'a self, _subject: &'a str) -> HrIdpFuture<'a, ()> {
        Box::pin(async { Err(HrIdpFailure::Unavailable) })
    }
}

/// Satu anggota HR di daftar perusahaan pemanggil.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HrMemberView {
    /// `corporate_member.id`.
    pub corporate_member_id: Uuid,
    /// Email akun (salinan, KEPUTUSAN-45).
    pub email: String,
    /// Nama lengkap (boleh kosong untuk HR yang diundang).
    pub full_name: Option<String>,
    /// Jabatan.
    pub job_title: Option<String>,
    /// Status akun (akun `pending_provisioning` = `PendingActivation` dengan [`Self::invitation_processing`]).
    pub status: AccountStatus,
    /// Undangan masih diproses: pengguna Keycloak belum dibuat worker (ADR-030 butir 5, 7).
    pub invitation_processing: bool,
    /// Anggota ini adalah pemanggil.
    pub is_self: bool,
}

/// Hasil penonaktifan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HrDeactivated {
    /// `corporate_member.id` yang dicabut.
    pub corporate_member_id: Uuid,
    /// Pengguna Keycloak sudah dinonaktifkan dan sesinya diakhiri. `false` selama penyedia belum dipasang;
    /// akses backend sudah tertutup oleh PostgreSQL.
    pub identity_provider_synced: bool,
}

/// Kebijakan audit objek akun HR (E-10, E-11). Email dan nama tidak dicatat; alasan hanya nama field.
static CORPORATE_ADMIN_ACCOUNT_POLICY: ObjectPolicy = ObjectPolicy::non_clinical(
    "account",
    &[
        FieldRule::value("status"),
        FieldRule::value("account_type"),
        FieldRule::name_only("status_reason"),
        FieldRule::value("corporate_member_id"),
        FieldRule::value("corporate_client_id"),
    ],
);

/// Alasan penonaktifan (`account.status_reason`, `corporate_member.delete_reason`).
const DEACTIVATION_REASON: &str = "deactivated_by_corporate_hr";

const DEACTIVATE_ACTION: &str = "corporate_admin_account_deactivated";
const INVITE_ACTION: &str = "corporate_admin_account_invited";
const GRANT_ACTION: &str = "corporate_membership_granted";
/// Alasan E-10 hasil gagal undangan ke email yang sudah dipakai (jawaban ke HR tetap seragam).
const EMAIL_IN_USE_REASON: &str = "email_in_use";

/// Percobaan menulis event hasil gagal (coding-standards §4.4 butir 3).
const FAILED_AUDIT_ATTEMPTS: u32 = 3;

/// Use case kelola anggota HR.
pub struct CorporateHrAccounts {
    pool: PgPool,
    clock: Arc<dyn Clock>,
    directory: Arc<AccountDirectory>,
    idp: Arc<dyn CorporateIdentityProvisioner>,
}

enum Work {
    Rule(HrRuleViolation, Uuid),
    /// Email undangan sudah dipakai akun lain di realm `korporat` (jawaban seragam, tanpa efek).
    EmailInUse,
    Db,
    Audit,
}

impl From<sqlx::Error> for Work {
    fn from(_: sqlx::Error) -> Self {
        Self::Db
    }
}

fn code(s: &str) -> Result<Code, Work> {
    Code::new(s).map_err(|_| Work::Audit)
}

fn status_value(s: AccountStatus) -> Result<Value, Work> {
    Value::code(s.as_str()).map_err(|_| Work::Audit)
}

/// Daftar HR perusahaan (keanggotaan aktif + status akun) di koneksi pemanggil.
async fn roster(
    conn: &mut PgConnection,
    client_id: Uuid,
) -> Result<
    (
        Vec<corporate::MemberRef>,
        Vec<corporate_hr_repo::HrAccountRow>,
    ),
    sqlx::Error,
> {
    let members = corporate::active_members(conn, client_id).await?;
    let ids: Vec<Uuid> = members.iter().map(|m| m.account_id).collect();
    let accounts = corporate_hr_repo::hr_accounts(conn, &ids).await?;
    Ok((members, accounts))
}

/// Status akun dan penanda `pending_provisioning` (ADR-030 butir 5); nilai lain tidak dikenal = `None`.
fn account_status(raw: &str) -> Option<(AccountStatus, bool)> {
    if raw == "pending_provisioning" {
        Some((AccountStatus::PendingActivation, true))
    } else {
        AccountStatus::parse(raw).map(|s| (s, false))
    }
}

/// Status tidak dikenal = galat (CHECK constraint menjamin tidak terjadi); akun yang bukan HR diabaikan.
fn to_roster(
    members: &[corporate::MemberRef],
    accounts: &[corporate_hr_repo::HrAccountRow],
) -> Result<HrRoster, Work> {
    let mut out = Vec::with_capacity(members.len());
    for m in members {
        let Some(a) = accounts.iter().find(|a| a.id == m.account_id) else {
            tracing::error!("corporate_member_without_hr_account");
            continue;
        };
        let (status, provisioning) = account_status(&a.status).ok_or(Work::Db)?;
        out.push(HrMember {
            member_id: m.corporate_member_id,
            account_id: a.id,
            status,
            provisioning,
        });
    }
    Ok(HrRoster { members: out })
}

fn client_ip(scope: &CorporateScope, route: &str) -> Result<IpAddr, IdentityError> {
    scope.principal().client_ip().ok_or_else(|| {
        tracing::error!(route = route, "client_ip_missing");
        IdentityError::Audit
    })
}

fn context(scope: &CorporateScope, ip: IpAddr) -> AuditContext {
    AuditContext::new(
        scope.principal().correlation_id(),
        Source::component(SourceComponent::CorporatePortal).ip(ip),
    )
}

fn hr_actor(scope: &CorporateScope) -> Result<Actor, Work> {
    Ok(Actor::account(
        scope.account_id(),
        code(scope.principal().account_type().as_str())?,
        Vec::new(),
    ))
}

fn event(
    ev: EventCode,
    action: &str,
    scope: &CorporateScope,
    target_account: Uuid,
    outcome: Outcome,
    ctx: &AuditContext,
) -> Result<AuditEvent, Work> {
    Ok(AuditEvent::new(
        ev,
        code(action)?,
        hr_actor(scope)?,
        ObjectRef::with_uuid(&CORPORATE_ADMIN_ACCOUNT_POLICY, target_account),
    )
    .tenant(scope.corporate_client_id())
    .outcome(outcome)
    .context(ctx))
}

impl CorporateHrAccounts {
    /// `directory` = resolver akun proses `api` (penimpaan cache setelah commit); `idp` = penyedia realm
    /// `korporat` ([`ProvisionerNotConfigured`] sampai pekerja ADR-030 dipasang).
    pub fn new(
        pool: PgPool,
        clock: Arc<dyn Clock>,
        directory: Arc<AccountDirectory>,
        idp: Arc<dyn CorporateIdentityProvisioner>,
    ) -> Self {
        Self {
            pool,
            clock,
            directory,
            idp,
        }
    }

    /// Anggota HR perusahaan pemanggil (US-1904 AC2), urut ID keanggotaan.
    pub async fn list_members(
        &self,
        scope: &CorporateScope,
    ) -> Result<Vec<HrMemberView>, IdentityError> {
        let mut conn = self
            .pool
            .acquire()
            .await
            .map_err(|_| IdentityError::Database)?;
        let (members, accounts) = roster(&mut conn, scope.corporate_client_id())
            .await
            .map_err(|_| IdentityError::Database)?;
        let mut out = Vec::with_capacity(members.len());
        for m in members {
            let Some(a) = accounts.iter().find(|a| a.id == m.account_id) else {
                tracing::error!("corporate_member_without_hr_account");
                continue;
            };
            let (status, provisioning) =
                account_status(&a.status).ok_or(IdentityError::Database)?;
            out.push(HrMemberView {
                corporate_member_id: m.corporate_member_id,
                email: a.email.clone(),
                full_name: a.full_name.clone(),
                job_title: m.job_title,
                status,
                invitation_processing: provisioning,
                is_self: a.id == scope.account_id(),
            });
        }
        Ok(out)
    }

    /// Menonaktifkan HR lain di perusahaan pemanggil (US-1904 AC2, AC5). Lihat dokumentasi modul.
    pub async fn deactivate_member(
        &self,
        scope: &CorporateScope,
        route: &str,
        corporate_member_id: Uuid,
    ) -> Result<HrDeactivated, IdentityError> {
        let ip = client_ip(scope, route)?;
        let owner = corporate::member_owner(&self.pool, corporate_member_id)
            .await
            .map_err(|_| IdentityError::Database)?;
        corporate_object(scope, route, owner, |c| *c)
            .map_err(|_| IdentityError::HrRule(HrRuleViolation::TargetNotFound))?;

        let ctx = context(scope, ip);
        let now = self.clock.now();
        let client = scope.corporate_client_id();
        let actor_id = scope.account_id();
        let tx_scope = scope.clone();
        let tx_ctx = ctx.clone();
        let result = in_audited_tx(&self.pool, self.clock.as_ref(), move |conn| {
            Box::pin(async move {
                corporate::lock_members(conn).await?;
                let (members, accounts) = roster(conn, client).await?;
                let r = to_roster(&members, &accounts)?;
                let target =
                    check_deactivation(&r, actor_id, corporate_member_id).map_err(|v| {
                        let account = r
                            .members
                            .iter()
                            .find(|m| m.member_id == corporate_member_id)
                            .map_or(Uuid::nil(), |m| m.account_id);
                        Work::Rule(v, account)
                    })?;
                let revoked = corporate::revoke_member(
                    conn,
                    corporate_member_id,
                    client,
                    actor_id,
                    DEACTIVATION_REASON,
                    now,
                )
                .await?;
                let deactivated = corporate_hr_repo::deactivate_hr(
                    conn,
                    target.account_id,
                    actor_id,
                    DEACTIVATION_REASON,
                    now,
                )
                .await?;
                if !revoked || !deactivated {
                    // Tidak mungkin di bawah kunci tabel (baris sudah diperiksa); batalkan seluruhnya.
                    return Err(Work::Db);
                }
                let e10 = event(
                    EventCode::E10,
                    "corporate_admin_account_deactivated",
                    &tx_scope,
                    target.account_id,
                    Outcome::success(),
                    &tx_ctx,
                )?
                .change(FieldChange::new(
                    "status",
                    Some(status_value(target.status)?),
                    Some(status_value(AccountStatus::Inactive)?),
                ))
                .change(FieldChange::changed("status_reason"));
                let e11 = event(
                    EventCode::E11,
                    "corporate_membership_revoked",
                    &tx_scope,
                    target.account_id,
                    Outcome::success(),
                    &tx_ctx,
                )?
                .change(FieldChange::new(
                    "corporate_member_id",
                    Some(Value::Uuid(corporate_member_id)),
                    None,
                ))
                .change(FieldChange::new(
                    "corporate_client_id",
                    Some(Value::Uuid(client)),
                    None,
                ));
                // Sasaran `pending_provisioning` sudah ditolak aturan, jadi `idp_subject` pasti terisi.
                let subject = accounts
                    .iter()
                    .find(|a| a.id == target.account_id)
                    .and_then(|a| a.idp_subject.clone())
                    .ok_or(Work::Db)?;
                Ok((subject, vec![e10, e11]))
            })
        })
        .await;
        let subject = match result {
            Ok(s) => s,
            Err(AuditedTxError::Work(Work::Rule(v, target_account))) => {
                self.record_rejected(scope, &ctx, v, target_account).await;
                return Err(IdentityError::HrRule(v));
            }
            Err(
                AuditedTxError::Work(Work::Db | Work::EmailInUse) | AuditedTxError::Database { .. },
            ) => {
                return Err(IdentityError::Database);
            }
            Err(AuditedTxError::Work(Work::Audit) | AuditedTxError::Audit(_)) => {
                return Err(IdentityError::Audit);
            }
        };
        if self
            .directory
            .refresh_account(Realm::Korporat, &subject)
            .await
            .is_err()
        {
            tracing::error!("corporate_hr_account_cache_refresh_failed");
        }
        let synced = self.idp.disable_user(&subject).await.is_ok();
        if !synced {
            // Akses backend sudah tertutup (keanggotaan dan status di PostgreSQL); sesi Keycloak menunggu
            // penyedia dipasang (lihat dokumentasi modul).
            tracing::warn!("corporate_hr_idp_disable_pending");
        }
        Ok(HrDeactivated {
            corporate_member_id,
            identity_provider_synced: synced,
        })
    }

    /// Mengundang HR lain ke perusahaan pemanggil (US-1904 AC2). Lihat dokumentasi modul. Email yang sudah
    /// dipakai di realm `korporat` menghasilkan `Ok(())` yang sama tanpa efek (P30-2).
    pub async fn invite_member(
        &self,
        scope: &CorporateScope,
        route: &str,
        raw_email: &str,
    ) -> Result<(), IdentityError> {
        let ip = client_ip(scope, route)?;
        let email = normalize_hr_email(raw_email).ok_or_else(|| {
            IdentityError::field(
                "email",
                "invalid",
                "Email harus berupa alamat email yang sah",
            )
        })?;
        let ctx = context(scope, ip);
        let now = self.clock.now();
        let client = scope.corporate_client_id();
        let actor_id = scope.account_id();
        let correlation = scope.principal().correlation_id();
        let tx_scope = scope.clone();
        let tx_ctx = ctx.clone();
        let result = in_audited_tx(&self.pool, self.clock.as_ref(), move |conn| {
            Box::pin(async move {
                // Urutan kunci: advisory email PERTAMA, lalu kunci tabel keanggotaan (ADR-030 catatan 5).
                let invitee = register_hr_invitation(
                    conn,
                    &email,
                    IntentActor {
                        actor_type: "corporate_admin",
                        account_id: Some(actor_id),
                    },
                    correlation,
                    &FixedClock::new(now),
                )
                .await
                .map_err(|e| match e {
                    HrInviteError::EmailInUse => Work::EmailInUse,
                    HrInviteError::InvalidEmail
                    | HrInviteError::Database
                    | HrInviteError::Queue => Work::Db,
                })?;
                corporate::lock_members(conn).await?;
                let (members, accounts) = roster(conn, client).await?;
                let r = to_roster(&members, &accounts)?;
                check_invite(&r, actor_id).map_err(|v| Work::Rule(v, actor_id))?;
                let member_id =
                    corporate::add_member(conn, client, invitee.account_id, actor_id, now).await?;
                let invited = event(
                    EventCode::E10,
                    INVITE_ACTION,
                    &tx_scope,
                    invitee.account_id,
                    Outcome::success(),
                    &tx_ctx,
                )?
                .change(FieldChange::new(
                    "account_type",
                    None,
                    Some(Value::code("corporate_admin").map_err(|_| Work::Audit)?),
                ))
                .change(FieldChange::new(
                    "status",
                    None,
                    Some(Value::code("pending_provisioning").map_err(|_| Work::Audit)?),
                ));
                let granted = event(
                    EventCode::E11,
                    GRANT_ACTION,
                    &tx_scope,
                    invitee.account_id,
                    Outcome::success(),
                    &tx_ctx,
                )?
                .change(FieldChange::new(
                    "corporate_member_id",
                    None,
                    Some(Value::Uuid(member_id)),
                ))
                .change(FieldChange::new(
                    "corporate_client_id",
                    None,
                    Some(Value::Uuid(client)),
                ));
                Ok(((), vec![invited, granted]))
            })
        })
        .await;
        match result {
            Ok(()) => Ok(()),
            Err(AuditedTxError::Work(Work::EmailInUse)) => {
                // Jawaban seragam dengan undangan baru; jejaknya hanya di audit internal.
                self.record_failed(
                    scope,
                    &ctx,
                    INVITE_ACTION,
                    EMAIL_IN_USE_REASON,
                    scope.account_id(),
                )
                .await;
                Ok(())
            }
            Err(AuditedTxError::Work(Work::Rule(v, account))) => {
                self.record_failed(scope, &ctx, INVITE_ACTION, v.reason_code(), account)
                    .await;
                Err(IdentityError::HrRule(v))
            }
            Err(AuditedTxError::Work(Work::Db) | AuditedTxError::Database { .. }) => {
                Err(IdentityError::Database)
            }
            Err(AuditedTxError::Work(Work::Audit) | AuditedTxError::Audit(_)) => {
                Err(IdentityError::Audit)
            }
        }
    }

    /// E-10 hasil gagal untuk penolakan aturan penonaktifan. Sasaran tidak ditemukan tidak dicatat di sini
    /// (sudah tercatat di security log oleh `corporate_object`).
    async fn record_rejected(
        &self,
        scope: &CorporateScope,
        ctx: &AuditContext,
        v: HrRuleViolation,
        target_account: Uuid,
    ) {
        if v == HrRuleViolation::TargetNotFound || target_account.is_nil() {
            return;
        }
        self.record_failed(
            scope,
            ctx,
            DEACTIVATE_ACTION,
            v.reason_code(),
            target_account,
        )
        .await;
    }

    /// E-10 hasil gagal di transaksi tersendiri dan dicoba ulang; kegagalan tidak mengubah jawaban
    /// (coding-standards §4.4 butir 3).
    async fn record_failed(
        &self,
        scope: &CorporateScope,
        ctx: &AuditContext,
        action: &str,
        reason: &str,
        object_account: Uuid,
    ) {
        let Ok(ev) = code(reason).and_then(|reason| {
            event(
                EventCode::E10,
                action,
                scope,
                object_account,
                Outcome::failure(reason),
                ctx,
            )
        }) else {
            tracing::error!("failed_outcome_audit_not_built");
            return;
        };
        for attempt in 1..=FAILED_AUDIT_ATTEMPTS {
            let e = ev.clone();
            let r = in_audited_tx(&self.pool, self.clock.as_ref(), move |_conn| {
                Box::pin(async move { Ok::<_, Infallible>(((), vec![e])) })
            })
            .await;
            match r {
                Ok(()) => return,
                Err(_) if attempt < FAILED_AUDIT_ATTEMPTS => {}
                Err(_) => tracing::error!("failed_outcome_audit_not_written"),
            }
        }
    }
}
