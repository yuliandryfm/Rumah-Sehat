//! Kebijakan endpoint modul `corporate` (coding-standards §1.3, KEPUTUSAN-127, ADR-012 butir 1).

use authz::{AdminSubRole, Policy, PublicReason};

/// Pendaftaran perusahaan mandiri dan teks persetujuannya (`/v1/public/corporate-registrations`): tanpa login
/// karena calon HR belum punya akun (US-1901, ADR-012 butir 3 diamandemen KEPUTUSAN-323). Pengamannya: batas laju,
/// jawaban seragam (anti-enumerasi), penyamaan waktu respons, persetujuan wajib, teks draf ditolak di production.
pub(super) fn registration() -> Policy {
    Policy::Public(PublicReason::CorporateRegistration)
}

/// Endpoint verifikasi perusahaan (`/backoffice/v1`, realm `admin`): hanya admin Ruang Sehat bersub-peran
/// Verifikator (US-1902, `[USER] C21-P4` 4a). Sub-peran lain termasuk Super Admin dijawab `403` dan token
/// realm lain (termasuk HR realm `korporat`) `401` oleh middleware kebijakan, keduanya tercatat di security
/// log (US-1902 AC5).
pub(super) fn verifier() -> Policy {
    Policy::rs_admin(&[AdminSubRole::Verifier])
}
