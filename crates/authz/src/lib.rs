//! Otorisasi endpoint Ruang Sehat dan RME (F1-T37, modul kritis T1, ADR-012, coding-standards §5.4).
//!
//! - Registri kebijakan per endpoint ([`PolicyRegistry`]); setiap endpoint menyatakan kebijakannya
//!   ([`Policy`]) dan permintaan tanpa kebijakan ditolak (default deny, ADR-012 butir 1).
//! - Rute hanya didaftarkan lewat [`PolicyRouter`]; [`PolicyRouter::finish`] memeriksa cakupan saat
//!   start dan memasang middleware penegak (KEPUTUSAN-127).
//! - Peran: jenis akun dan sub-peran admin Ruang Sehat ([`AdminSubRole`], UP-18) dari database aplikasi
//!   lewat [`PrincipalResolver`] (ADR-011 butir 6), tanpa sub-peran yang otomatis memegang hak lain.
//! - Respons: token tidak sah atau realm lain `401`, peran tidak diizinkan `403`, objek terlarang `404`
//!   yang identik dengan objek tidak ada ([`visible_object`], ADR-012 butir 4, KU-03).
//!
//! Belum di sini (task lain): relasi kepemilikan/penugasan/tenant dan E-05 hasil gagal (F1-T38),
//! status akun dari database dan sesi yang dicabut (F1-T33), cakupan tenant `PartnerScope`
//! (KEPUTUSAN-128), kebijakan klien realm `integrasi` (ADR-014), realm `korporat` (F1-T86/F1-T100).

mod enforce;
mod error;
mod object;
mod policy;
mod principal;
mod registry;
mod role;
mod router;

pub use error::{AuthzError, PolicySetupError};
pub use object::{deny_action, visible_object};
pub use policy::{Policy, PublicReason, Requirement};
pub use principal::{AccountRecord, Principal, PrincipalResolver, ResolveError, ResolveFuture};
pub use registry::{EndpointKey, PolicyRegistry, check_coverage};
pub use role::{AccountStatus, AccountType, AdminSubRole};
pub use router::{Enforcement, PolicyRouter};
