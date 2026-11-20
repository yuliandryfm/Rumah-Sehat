//! Kebijakan endpoint modul `privacy` (coding-standards §1.3, KEPUTUSAN-127, ADR-012 butir 1).

use authz::{Policy, PublicReason};

/// Teks persetujuan yang berlaku: tanpa login karena dibaca calon pasien sebelum akun ada (US-104 AC1,
/// ADR-011 butir 6). Isinya teks yang memang publik (ADR-012 butir 3, UP-21); hanya baca, dengan batas laju.
pub(super) fn consent_text() -> Policy {
    Policy::Public(PublicReason::PublicContent)
}
