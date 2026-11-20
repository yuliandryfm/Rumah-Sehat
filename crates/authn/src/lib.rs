//! Validasi token akses Keycloak di backend (F1-T32, modul kritis T1).
//!
//! - ADR-011 butir 8: tanda tangan, audience realm, dan masa berlaku diperiksa di setiap permintaan;
//!   hanya algoritma dalam daftar izin, kunci milik issuer realm yang diharapkan, audience wajib
//!   (RFC 8725 [SRC-79]).
//! - ADR-011 butir 11: backend Ruang Sehat menolak realm `rme`, backend RME hanya menerima `rme`
//!   ([`Backend`], ditegakkan saat konfigurasi).
//! - JWKS per realm di-cache dengan TTL, gagal-tertutup ([`jwks`]); alamat JWKS dari konfigurasi.
//! - [`AuthContext`] hanya dibangun oleh [`Authenticator`]; ekstraktor axum di [`extract`].
//!
//! Status akun dan daftar sesi yang dicabut (ADR-011 butir 8 bagian kedua, KU-10) diperiksa F1-T33;
//! peran dan kebijakan endpoint di crate `authz` (F1-T37).

mod context;
mod error;
pub mod extract;
pub mod jwks;
mod realm;
#[cfg(feature = "testing")]
pub mod testing;
mod verify;

pub use context::AuthContext;
pub use error::{AuthError, AuthnConfigError};
pub use extract::{AuthState, ErrorRenderer};
pub use jsonwebtoken::Algorithm;
pub use jwks::{HttpJwksSource, JwksFetchError, JwksFuture, JwksSource, MAX_JWKS_BYTES};
pub use realm::{Backend, Realm};
pub use verify::{Authenticator, AuthnConfig, MAX_TOKEN_LEN, RealmConfig, bearer_token};
