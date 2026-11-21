//! Otorisasi endpoint Ruang Sehat dan RME (F1-T37, modul kritis T1, ADR-012, coding-standards §5.4).
//!
//! - Registri kebijakan per endpoint ([`PolicyRegistry`]); setiap endpoint menyatakan kebijakannya
//!   ([`Policy`]) dan permintaan tanpa kebijakan ditolak (default deny, ADR-012 butir 1).
//! - Rute hanya didaftarkan lewat [`PolicyRouter`]; [`PolicyRouter::finish`] memeriksa cakupan saat
//!   start dan memasang middleware penegak (KEPUTUSAN-127), lalu mengembalikan [`EnforcedRouter`] yang
//!   tertutup untuk penambahan rute (TD-15).
//! - Peran: jenis akun dan sub-peran admin Ruang Sehat ([`AdminSubRole`], UP-18) dari database aplikasi
//!   lewat [`PrincipalResolver`] (ADR-011 butir 6), tanpa sub-peran yang otomatis memegang hak lain.
//! - Respons: token tidak sah atau realm lain `401`, peran tidak diizinkan `403`, objek terlarang `404`
//!   yang identik dengan objek tidak ada ([`visible_object`], ADR-012 butir 4, KU-03).
//!
//! - Status akun dan sesi yang dicabut (F1-T33, ADR-011 butir 8, KU-10): akun nonaktif `401` `4010002`
//!   (layar "Akun nonaktif"), sesi dicabut `401` `4010001` ([`SessionGuard`]).
//! - Relasi kepemilikan pasien atas penerima layanan ([`care_recipient_access`], bukti
//!   [`OwnedCareRecipient`]) dan penolakan objek yang dicatat di latar belakang ([`DenialSink`]:
//!   security log, dan untuk data sensitif event audit E-05 hasil gagal lewat [`DenialAuditor`]) tanpa
//!   menambah waktu tanggap jalur `404` (F1-T38, KU-03, TD-15).
//! - Permintaan ubah/hapus ke jalur audit tanpa kebijakan ditolak (`401` tanpa token sah, selain itu
//!   `404`) dan dicatat sebagai kejadian keamanan `audit_modification_attempt` ([`is_audit_path`],
//!   US-1504 AC2, ADR-009; F1-T68).
//!
//! - Tenant perusahaan (F1-T100, ADR-028 butir 1): akun HR realm `korporat` ([`AccountType::CorporateAdmin`])
//!   hanya di namespace `/v1/corporate/` ([`Policy::corporate_hr`]); cakupan [`CorporateScope`] dari
//!   keanggotaan aktif yang dibaca [`CorporateScopeResolver`] di setiap permintaan; objek perusahaan lain
//!   `404` ([`corporate_object`]).
//!
//! Belum di sini (task lain): relasi penugasan, cakupan tenant `PartnerScope` (KEPUTUSAN-128), kebijakan
//! klien realm `integrasi` (ADR-014), akses data sensitif peserta korporat (ADR-028 butir 3).

mod audit_guard;
mod corporate;
mod denial;
mod enforce;
mod error;
mod object;
mod policy;
mod principal;
mod registry;
mod role;
mod router;

pub use audit_guard::is_audit_path;
pub use corporate::{
    CORPORATE_PATH_PREFIX, CorporateAccess, CorporateClientStatus, CorporateMembership,
    CorporateScope, CorporateScopeResolver, MembershipFuture, corporate_object, is_corporate_path,
};
pub use denial::{
    ClientIp, DEFAULT_DENIAL_AUDIT_CONCURRENCY, DEFAULT_DENIAL_QUEUE_CAPACITY, DROP_REPORT_WINDOW,
    DenialAuditError, DenialAuditFuture, DenialAuditor, DenialSink, DenialSinkError, DeniedAccess,
    PreparedDenialAudit, SensitiveTarget,
};
pub use error::{AuthzError, PolicySetupError};
pub use object::{
    CareRecipientOwner, OwnedCareRecipient, Sensitivity, care_recipient_access, deny_action,
    visible_object, visible_sensitive_object,
};
pub use policy::{Policy, PublicReason, Requirement};
pub use principal::{
    AccountRecord, Principal, PrincipalResolver, ResolveError, ResolveFuture, RevocationFuture,
    SessionGuard,
};
pub use registry::{EndpointKey, PolicyRegistry, check_coverage};
pub use role::{AccountStatus, AccountType, AdminSubRole};
pub use router::{EnforcedRouter, Enforcement, PolicyRouter};
