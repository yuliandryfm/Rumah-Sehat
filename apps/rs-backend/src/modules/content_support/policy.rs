//! Kebijakan endpoint modul `content_support` (coding-standards §1.3, KEPUTUSAN-127, ADR-012 butir 1).

use authz::{AdminSubRole, Policy, PublicReason};

/// Pengelolaan konten CMS: hanya admin Ruang Sehat bersub-peran Konten (UP-18, US-1101, US-1802 AC4).
/// Sub-peran lain, termasuk Super Admin, tidak otomatis memegang hak ini (crate `authz`, `AdminSubRole`);
/// penolakan dijawab `403` dan dicatat di security log oleh middleware kebijakan.
pub(super) fn manage_content() -> Policy {
    Policy::rs_admin(&[AdminSubRole::Content])
}

/// Konten situs yang sedang tayang: tanpa login, alasan sah "konten yang memang publik" (ADR-012 butir 3,
/// UP-21); hanya baca, dengan batas laju (ADR-027 butir 5).
pub(super) fn public_content() -> Policy {
    Policy::Public(PublicReason::PublicContent)
}
