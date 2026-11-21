//! Kebijakan endpoint modul `profile` (coding-standards §1.3, KEPUTUSAN-127, ADR-012 butir 1).

use authz::Policy;

/// Endpoint aplikasi pasien (`/patient/v1`, realm `pasien`): hanya akun berjenis pasien. Relasi ke penerima
/// layanan diperiksa per objek di service (`authz::care_recipient_access`, KU-03), bukan di kebijakan ini.
/// Token realm lain dijawab `401` oleh middleware kebijakan (ADR-011 butir 8).
pub(super) fn patient() -> Policy {
    Policy::patient()
}
