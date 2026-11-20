//! Kebijakan endpoint modul `platform` (coding-standards §1.3, KEPUTUSAN-127).

use authz::{Policy, PublicReason};

/// Endpoint kesehatan: tanpa login, dibaca pemeriksa luar dan load balancer; respons tanpa data
/// pribadi (F1-T11, ADR-012 butir 3).
pub(super) fn health() -> Policy {
    Policy::Public(PublicReason::Operational)
}
