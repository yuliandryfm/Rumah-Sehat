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
//! **Penyedia identitas belum dipasang.** Undangan HR baru (pembuatan pengguna realm `korporat` dan email
//! aktivasi sekali pakai) dan penonaktifan pengguna Keycloak membutuhkan hak setara `manage-users` atas
//! seluruh realm `korporat` (Keycloak 26.8.2, SRC-674; FGAP v2 tidak mempersempitnya berarti, SRC-830).
//! Menurut TD-26/TD-30 klien itu tidak boleh dipegang proses `api`; ADR-030 (KEPUTUSAN-302) menetapkan
//! aksi itu hanya dijalankan proses `worker` lewat niat dan job outbox berjenis tertutup. Pemasangannya
//! task terpisah: sampai itu [`CorporateIdentityProvisioner`] hanya antarmuka, implementasi yang dipasang
//! ([`ProvisionerNotConfigured`]) tidak memanggil Admin REST, dan endpoint undangan belum dibuat.

use super::corporate_hr_domain::{HrMember, HrRoster, HrRuleViolation, check_deactivation};
use super::corporate_hr_repo;
use super::error::IdentityError;
use super::service::AccountDirectory;
use crate::modules::corporate::api as corporate;
use audit::{
    Actor, AuditContext, AuditEvent, AuditedTxError, Code, EventCode, FieldChange, FieldRule,
    ObjectPolicy, ObjectRef, Outcome, Source, SourceComponent, Value, in_audited_tx,
};
use authn::Realm;
use authz::{AccountStatus, CorporateScope, corporate_object};
use foundation::{Clock, Uuid};
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
    /// Status akun.
    pub status: AccountStatus,
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
        FieldRule::name_only("status_reason"),
        FieldRule::value("corporate_member_id"),
        FieldRule::value("corporate_client_id"),
    ],
);

/// Alasan penonaktifan (`account.status_reason`, `corporate_member.delete_reason`).
const DEACTIVATION_REASON: &str = "deactivated_by_corporate_hr";

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
        let status = AccountStatus::parse(&a.status).ok_or(Work::Db)?;
        out.push(HrMember {
            member_id: m.corporate_member_id,
            account_id: a.id,
            status,
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
            let status = AccountStatus::parse(&a.status).ok_or(IdentityError::Database)?;
            out.push(HrMemberView {
                corporate_member_id: m.corporate_member_id,
                email: a.email.clone(),
                full_name: a.full_name.clone(),
                job_title: m.job_title,
                status,
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
                let subject = accounts
                    .iter()
                    .find(|a| a.id == target.account_id)
                    .map(|a| a.idp_subject.clone())
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
            Err(AuditedTxError::Work(Work::Db) | AuditedTxError::Database { .. }) => {
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

    /// E-10 hasil gagal untuk penolakan aturan, di transaksi tersendiri dan dicoba ulang; kegagalan tidak
    /// mengubah jawaban (coding-standards §4.4 butir 3). Sasaran tidak ditemukan tidak dicatat di sini
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
        let Ok(ev) = code(v.reason_code()).and_then(|reason| {
            event(
                EventCode::E10,
                "corporate_admin_account_deactivated",
                scope,
                target_account,
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
