//! Pemanggil terotorisasi: identitas dari token yang sudah divalidasi (`authn`) ditambah akun dan
//! sub-peran dari database aplikasi (ADR-011 butir 6: peran disimpan di aplikasi, bukan di token).

use crate::denial::{DenialSink, DeniedAccess};
use crate::role::{AccountStatus, AccountType, AdminSubRole};
use authn::AuthContext;
use foundation::{CorrelationId, Uuid};
use std::collections::BTreeSet;
use std::fmt;
use std::future::Future;
use std::net::IpAddr;
use std::pin::Pin;

/// Data akun yang dibaca resolver dari database aplikasi untuk satu `(realm, subject)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountRecord {
    /// `account.id`.
    pub account_id: Uuid,
    /// `account.account_type`.
    pub account_type: AccountType,
    /// `account.status`.
    pub status: AccountStatus,
    /// Sub-peran admin aktif (`rs_admin_role` dengan `deleted_at IS NULL`); kosong untuk jenis lain.
    pub sub_roles: Vec<AdminSubRole>,
}

/// Kegagalan resolver (misalnya database tidak tersedia). Permintaan ditolak (`503`), tidak pernah
/// dilewatkan.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResolveError;

/// Future hasil [`PrincipalResolver::resolve`].
pub type ResolveFuture<'a> =
    Pin<Box<dyn Future<Output = Result<Option<AccountRecord>, ResolveError>> + Send + 'a>>;

/// Memetakan konteks autentikasi ke akun aplikasi (`account.idp_realm`, `account.idp_subject`, tanpa
/// baris terhapus). `Ok(None)` = akun tidak ada → `401`. Implementasi database dan pemeriksaan sesi
/// yang dicabut menyusul (F1-T33); aksi berisiko membaca status langsung dari database (ADR-011
/// butir 8).
pub trait PrincipalResolver: Send + Sync {
    /// Mencari akun untuk `ctx`.
    fn resolve<'a>(&'a self, ctx: &'a AuthContext) -> ResolveFuture<'a>;
}

/// Pemanggil yang lolos autentikasi, status akun aktif, dan kebijakan endpoint. Hanya dibangun crate
/// ini (field privat), lalu dipasang di extensions permintaan oleh middleware kebijakan; handler
/// mengambilnya dengan ekstraktor [`Principal`]. Membawa konteks permintaan untuk pencatatan penolakan
/// objek (ID korelasi, IP klien, antrean penolakan; F1-T38).
#[derive(Clone)]
pub struct Principal {
    account_id: Uuid,
    account_type: AccountType,
    sub_roles: BTreeSet<AdminSubRole>,
    auth: AuthContext,
    correlation_id: CorrelationId,
    client_ip: Option<IpAddr>,
    denials: DenialSink,
}

impl Principal {
    pub(crate) fn new(
        record: AccountRecord,
        auth: AuthContext,
        correlation_id: CorrelationId,
        client_ip: Option<IpAddr>,
        denials: DenialSink,
    ) -> Self {
        // Sub-peran hanya bermakna untuk admin Ruang Sehat; untuk jenis lain dibuang.
        let sub_roles = if record.account_type == AccountType::RsAdmin {
            record.sub_roles.into_iter().collect()
        } else {
            BTreeSet::new()
        };
        Self {
            account_id: record.account_id,
            account_type: record.account_type,
            sub_roles,
            auth,
            correlation_id,
            client_ip,
            denials,
        }
    }

    /// ID korelasi permintaan.
    pub fn correlation_id(&self) -> CorrelationId {
        self.correlation_id
    }

    /// Alamat IP klien bila middleware IP klien terpasang ([`crate::ClientIp`]).
    pub fn client_ip(&self) -> Option<IpAddr> {
        self.client_ip
    }

    /// Penolakan objek dicatat di latar belakang (tanpa menunggu).
    pub(crate) fn report_denial(&self, denial: DeniedAccess) {
        self.denials.submit(denial);
    }

    /// `account.id` pemanggil.
    pub fn account_id(&self) -> Uuid {
        self.account_id
    }

    /// Jenis akun.
    pub fn account_type(&self) -> AccountType {
        self.account_type
    }

    /// Sub-peran admin yang dipegang.
    pub fn sub_roles(&self) -> &BTreeSet<AdminSubRole> {
        &self.sub_roles
    }

    /// Apakah memegang sub-peran `role`.
    pub fn has_sub_role(&self, role: AdminSubRole) -> bool {
        self.sub_roles.contains(&role)
    }

    /// Konteks autentikasi (realm, subjek, sesi).
    pub fn auth(&self) -> &AuthContext {
        &self.auth
    }
}

impl fmt::Debug for Principal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Principal")
            .field("account_id", &self.account_id)
            .field("account_type", &self.account_type)
            .field("sub_roles", &self.sub_roles)
            .field("auth", &self.auth)
            .field("correlation_id", &self.correlation_id)
            .finish_non_exhaustive()
    }
}
