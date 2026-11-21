//! Titik keputusan tunggal pemisahan tugas Verifikator (US-201 AC8, F1-T62).
//!
//! Penentu "Verifikator juga pemilik akun pasien" = KEPUTUSAN-294 (jawaban user PT-57/T-38 = opsi A):
//! NIK staf Verifikator disimpan sebagai blind index saja (tanpa ciphertext) saat sub-peran Verifikator
//! diberikan (modul `identity`, US-208), lalu dicocokkan dengan NIK pemilik akun pasien. Aturan
//! [`SelfDecisionGuard::may_decide`]:
//!
//! 1. ID akun Verifikator sama dengan akun pemilik: ditolak (tidak terjadi lintas realm; tetap diperiksa).
//! 2. Verifikator tanpa NIK staf tidak boleh memutuskan apa pun (gagal-tertutup).
//! 3. NIK staf cocok dengan NIK pemilik akun pasien pemilik pengajuan: (a) NIK pemilik akun yang sudah
//!    disetujui (baris registri aktif penerima layanan pemilik akun), dan (b) NIK pada pengajuan itu sendiri
//!    bila pengajuan milik pemilik akun. Butir (b) ditambahkan karena pengajuan pertama belum punya NIK
//!    disetujui: tanpa (b) Verifikator dapat menyetujui pengajuan pertamanya sendiri. Pengajuan anggota
//!    keluarga tercakup karena pemeriksaan memakai akun pemilik, bukan penerima layanan.
//!
//! Pencocokan: blind index staf tersimpan di satu versi kunci, sedangkan NIK pemilik dapat tercatat di versi
//! lain. NIK pemilik karena itu didekripsi (DEK wadah `identity_document`) lalu dihitung ulang di versi kunci
//! blind index staf, sehingga rotasi kunci (TD-10) tidak membuka celah. Versi kunci staf yang tidak lagi
//! termuat = tidak dapat dibandingkan = ditolak (gagal-tertutup; NIK staf didaftarkan ulang lewat pemberian
//! sub-peran).
//!
//! Bila tidak ada Verifikator aktif lain, pengajuan tetap menunggu; Super Admin tidak memutuskan
//! (kebijakan rute hanya Verifikator).

use super::api::IdentityServices;
use super::error::ProfileError;
use super::repo;
use super::verification::open_document_nik;
use crate::modules::identity::api as identity;
use authz::Principal;
use crypto::{Nik, constant_time_eq};
use foundation::Uuid;

/// Penjaga pemisahan tugas atas satu pengajuan.
pub(super) struct SelfDecisionGuard<'a> {
    services: &'a IdentityServices,
}

impl<'a> SelfDecisionGuard<'a> {
    pub(super) fn new(services: &'a IdentityServices) -> Self {
        Self { services }
    }

    /// `true` bila `principal` boleh memutuskan pengajuan milik akun pasien `owner_account_id`.
    /// `owner_request_nik` = NIK pengajuan itu bila pengajuan milik pemilik akun (sudah didekripsi pemanggil).
    pub(super) async fn may_decide(
        &self,
        principal: &Principal,
        owner_account_id: Uuid,
        owner_request_nik: Option<&Nik>,
    ) -> Result<bool, ProfileError> {
        let s = self.services;
        let verifier = principal.account_id();
        if !super::domain::verifier_may_decide(verifier, owner_account_id) {
            return Ok(false);
        }
        let Some(staff) = identity::staff_nik_index(&s.pool, verifier)
            .await
            .map_err(|_| ProfileError::Database)?
        else {
            return Ok(false);
        };
        let approved = match repo::approved_owner_document(&s.pool, owner_account_id).await? {
            Some(d) => Some(open_document_nik(s, d.document_id, d.data_key_id, &d.nik_enc).await?),
            None => None,
        };
        for nik in owner_request_nik.into_iter().chain(approved.as_ref()) {
            let Ok(b) = s.blind_index.compute_with_version(nik, staff.key_version) else {
                tracing::error!("staff_nik_key_version_unavailable");
                return Ok(false);
            };
            if constant_time_eq(&b.value, &staff.bidx) {
                return Ok(false);
            }
        }
        Ok(true)
    }
}
