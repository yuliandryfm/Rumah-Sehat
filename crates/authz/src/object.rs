//! Keputusan atas objek di tingkat service (ADR-012 butir 4, KU-03, coding-standards §3.4).
//!
//! Objek yang tidak boleh diakses dijawab persis seperti objek yang tidak ada (`404` `4040001`), dan
//! percobaannya masuk security log (`access_denied`, `foreign_resource_id`). Pemeriksaan relasi
//! (kepemilikan, penugasan, tenant) dan pencatatan audit E-05 hasil gagal untuk data sensitif dibangun
//! di F1-T38 di atas fungsi ini.

use crate::error::AuthzError;
use crate::principal::Principal;
use observability::security::{self, AccessDenial};

/// Mengembalikan `found` bila ada **dan** `allowed` menyetujuinya; selain itu
/// [`AuthzError::NotFound`] yang sama untuk "tidak ada" dan "tidak boleh". `route` adalah pola rute
/// (misalnya `/patient/v1/orders/{order_id}`), bukan URL berisi ID.
pub fn visible_object<T>(
    principal: &Principal,
    route: &str,
    found: Option<T>,
    allowed: impl FnOnce(&T) -> bool,
) -> Result<T, AuthzError> {
    match found {
        None => Err(AuthzError::NotFound),
        Some(obj) if allowed(&obj) => Ok(obj),
        Some(_) => {
            security::access_denied(
                Some(principal.account_id()),
                route,
                AccessDenial::ForeignResourceId,
            );
            Err(AuthzError::NotFound)
        }
    }
}

/// Penolakan aksi yang dikenal tetapi tidak diizinkan untuk peran pemanggil (`403` `4030001`), dengan
/// security log `access_denied` (`role_not_allowed`). Untuk aturan peran yang hanya dapat diputuskan
/// di service (misalnya pemisahan tugas ADR-012 butir 7).
pub fn deny_action(principal: &Principal, route: &str) -> AuthzError {
    security::access_denied(
        Some(principal.account_id()),
        route,
        AccessDenial::RoleNotAllowed,
    );
    AuthzError::Forbidden
}
