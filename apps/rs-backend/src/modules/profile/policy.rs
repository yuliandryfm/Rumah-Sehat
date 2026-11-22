//! Kebijakan endpoint modul `profile` (coding-standards §1.3, KEPUTUSAN-127, ADR-012 butir 1).

use authz::{AdminSubRole, Policy};

/// Endpoint aplikasi pasien (`/patient/v1`, realm `pasien`): hanya akun berjenis pasien. Relasi ke penerima
/// layanan diperiksa per objek di service (`authz::care_recipient_access`, KU-03), bukan di kebijakan ini.
/// Token realm lain dijawab `401` oleh middleware kebijakan (ADR-011 butir 8).
pub(super) fn patient() -> Policy {
    Policy::patient()
}

/// Endpoint Verifikator (`/backoffice/v1`, realm `admin`): hanya admin Ruang Sehat bersub-peran Verifikator
/// (US-201, UP-18). Sub-peran lain termasuk Super Admin tidak otomatis memegang hak ini; penolakan dijawab
/// `403` dan dicatat di security log oleh middleware kebijakan (pola CMS `content_support::policy`).
pub(super) fn verifier() -> Policy {
    Policy::rs_admin(&[AdminSubRole::Verifier])
}
