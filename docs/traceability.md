# Traceability Matrix

Menghubungkan setiap kebutuhan bisnis sampai ke kode dan test-nya. Fitur atau story tanpa induk BR adalah tanda ada yang dikarang dan harus dihapus atau dikonfirmasi user.

Diisi bertahap: kolom BR di Tahap 2, story di Tahap 3, test case di Tahap 7, file kode/test di Tahap 8. Pembaruan 2026-11-19 (CR-021): BR-130–BR-145 dan EP-18/EP-19 (BRD v1.4, PRD v1.3, phase plan v0.3 `[USER] C21-P4`); test case story CR-021 belum dibuat dan disusun QA dari PRD v1.3 sebelum task terkait. File kode story mobile merujuk repositori mobile (CR-019).

| BR | Kebutuhan bisnis | Sumber BR | Story | Test case | File kode / test | Status |
| --- | --- | --- | --- | --- | --- | --- |
| BR-001 | Pasien dapat mendaftar dan masuk dengan email + password. Login Google/Apple dihapus (CR-007) | `[DOK] P4.4` `[USER] T4-Q1, T4-Q2` | US-101, US-103, US-110 (US-102 dihapus CR-007) | TC-US-101-1–5, 7–30; TC-US-103-1–51; TC-US-110-1–12 | US-103 AC4d (TD-27): sesi diakhiri setelah atur ulang `infra/keycloak/themes/ruang-sehat/login/login-update-password.ftl`, pencabutan dari event LOGOUT `apps/rs-backend/src/modules/identity/jobs.rs`, `domain.rs`, `service.rs`, `apps/rs-backend/src/cli.rs`; email pemberitahuan `infra/keycloak/keycloak.conf`, `infra/keycloak/realms/*.json`, `infra/keycloak/themes/ruang-sehat/email/`; test `apps/rs-backend/tests/identity_password_reset_sessions.rs`, `infra/keycloak/test/keycloak_check.py` (`cek_atur_ulang_tidak_melewati_otp`) | BRD v1.3 (CR-007/Tahap 4); PRD v1.2 final (Checkpoint 2: CR-017, CR-018, US-110) |
| BR-002 | Ruang Sehat menyimpan data identitas dasar pasien (nama, tanggal lahir, jenis kelamin, nomor HP, email,… | `[DOK] P4.1` `[USER] T3-Q3` | US-105, US-108 | TC-US-105-1–25; TC-US-108-1–7 | US-105 AC1-AC3 (F1-T56): `apps/rs-backend/src/modules/profile/profile_service.rs`, `profile_rules.rs`, `profile_http.rs`, `profile_dto.rs`, `repo.rs`; kontrak `openapi/pasien.json`; test `apps/rs-backend/tests/profile_patient_profile_http.rs` dan unit `profile_rules.rs`. AC4 (ubah email) = F1-T57 (Fase 2) | BRD v1.2 (CR-006/Tahap 3) |
| BR-003 | NIK dan foto KTP pasien wajib sebelum memakai konsultasi online, home visit, serta pendaftaran klinik da… | `[DOK] P4.6a` | US-106, US-107 | TC-US-106-1–34; TC-US-107-1–5 | US-106 AC1-AC5 (F1-T60, F1-T61): `apps/rs-backend/src/modules/profile/service.rs`, `repo.rs`, `domain.rs`, `http.rs`, `dto.rs`; kontrak `openapi/pasien.json`; test `apps/rs-backend/tests/profile_identity_submission.rs`, `apps/rs-backend/tests/profile_identity_http.rs`, unit `profile/domain.rs` | BRD v1.0 |
| BR-004 | Admin Ruang Sehat memverifikasi NIK dan foto KTP secara manual (disetujui atau ditolak) sebelum layanan… | `[DOK] P4.6b` | US-106, US-107, US-201 | TC-US-106-1–34; TC-US-107-1–5; TC-US-201-1–27 | US-106 AC1-AC5 (F1-T60, F1-T61): `apps/rs-backend/src/modules/profile/service.rs`, `repo.rs`, `domain.rs`, `http.rs`, `dto.rs`; kontrak `openapi/pasien.json`; test `apps/rs-backend/tests/profile_identity_submission.rs`, `apps/rs-backend/tests/profile_identity_http.rs`, unit `profile/domain.rs`; US-201 AC5–AC8 domain (F1-T59): unit `profile/domain.rs`; US-201 AC1–AC3a, AC5–AC8 (F1-T62): `apps/rs-backend/src/modules/profile/verification.rs`, `self_decision.rs`, `verifier_http.rs`, `verifier_dto.rs`, `repo.rs`, `modules/platform/api.rs` (`active_reason_codes`), `modules/identity/admin_service.rs` (NIK staf, KEPUTUSAN-294), migrasi `db/migrations/app/20261123300000_profile_identity_rejection_reason_seed.sql`, `20261123310000_identity_account_staff_nik_bidx.sql`; kontrak `openapi/backoffice.json`; test `apps/rs-backend/tests/profile_identity_verifier.rs`, `identity_admin_accounts.rs` (NIK staf); US-201 AC1–AC3a, AC5–AC8 sisi backoffice (F1-T64): `web/backoffice/src/komponen/verifikasi/PanelVerifikasi.tsx`, `LayarTanpaAksesVerifikator.tsx`, `lib/verifikasi/klien.ts`, `lib/verifikasi/model.ts`, `app/(terautentikasi)/verifikasi/page.tsx`, `lib/navigasi.ts`, `keamanan.ts` (img-src origin foto), test `komponen/verifikasi/verifikasi.test.tsx`; e2e Playwright belum (F1-T70); TC-US-201-26 (F4), -6/-23 (F2) belum | BRD v1.0 |
| BR-005 | Akun dokter, psikolog, fisioterapis, dan tenaga medis diverifikasi dan dikelola admin Ruang Sehat. Setia… | `[DOK] P8.2a` `[USER] T2-Q2` | US-202, US-203 | TC-US-202-1–6; TC-US-203-1–5 |  | BRD v1.0 |
| BR-006 | Akun rekanan (klinik, lab, toko kecantikan, toko alat kesehatan, apotek) dan admin mitranya diverifikasi… | `[DOK] P8.2a` `[USER] T2-Q3, T2-Q3b` | US-205, US-206 | TC-US-205-1–7; TC-US-206-1–5 |  | BRD v1.0 |
| BR-007 | Aplikasi pasien tersedia di Android dan iOS; tenaga kesehatan memakai aplikasi mobile yang juga tersedia… | `[DOK] P6.1, P6.2` `[USER] T3-Q9` | NFR-COMP-01 | TC-NFR-COMP-1; TC-NFR-COMP-3 |  | BRD v1.2 (CR-006/Tahap 3) |
| BR-008 | Pasien dapat memesan konsultasi online, pendaftaran klinik/lab, dan home visit untuk anggota keluarga ya… | `[USER] T3-Q3, T3-Q3a` | US-107, US-108, US-109, US-201, US-310, US-902 | TC-US-107-1–5; TC-US-108-1–7; TC-US-109-1–4; TC-US-201-1–27; TC-US-310-1–5; TC-US-902-1–3 | US-201 pemilik akun (F1-T62): lihat BR-004; anggota keluarga (US-108) di F4 | BRD v1.2 (CR-006/Tahap 3) |
| BR-010 | Pasien dapat berkonsultasi dengan dokter lewat chat teks dan panggilan video | `[DOK] P3.1, P3.2` | US-303, US-304, US-305, US-306 | TC-US-303-1–4; TC-US-304-1–8; TC-US-305-1–10; TC-US-306-1–6 |  | BRD v1.0 |
| BR-011 | Pasien dapat berkonsultasi kesehatan mental dengan psikolog lewat chat teks dan panggilan video | `[DOK] P2.1, P3.1, P3.2` | US-303, US-304 | TC-US-303-1–4; TC-US-304-1–8 |  | BRD v1.0 |
| BR-012 | Pasien dapat berkonsultasi dengan fisioterapis dan tenaga medis lewat chat teks dan panggilan video | `[USER] T2-Q5` | US-303, US-304 | TC-US-303-1–4; TC-US-304-1–8 |  | BRD v1.0 |
| BR-013 | Konsultasi berlangsung di Ruang Sehat. Tenaga kesehatan memakai aplikasi tenaga kesehatan untuk chat, vi… | `[DOK] PS6b` | US-308, US-311, US-1003 | TC-US-308-1–6; TC-US-311-1–5; TC-US-1003-1–4 |  | BRD v1.0 |
| BR-014 | Setiap konsultasi diberikan oleh tenaga kesehatan atas nama klinik rekanan yang menaunginya; PT. Arunika… | `[USER] T2-Q2` | US-303, US-804 | TC-US-303-1–4; TC-US-804-1–3 |  | BRD v1.0 |
| BR-015 | Dokter dapat menerbitkan resep digital setelah konsultasi. Pasien menebusnya sendiri di apotek mana pun;… | `[DOK] P3.4, PS1` | US-309 | TC-US-309-1–12 |  | BRD v1.0 |
| BR-016 | Isi chat konsultasi (termasuk foto), catatan tenaga kesehatan, dan resep disimpan. Panggilan video tidak… | `[DOK] P4.1` | US-305, US-306, US-308, US-310 | TC-US-305-1–10; TC-US-306-1–6; TC-US-308-1–6; TC-US-310-1–5 |  | BRD v1.0 |
| BR-017 | Aplikasi tersedia 24 jam, 7 hari. Ketersediaan tenaga kesehatan mengikuti jadwal masing-masing. Target k… | `[DOK] P6.4` `[USER] T3-Q12` | US-302, NFR-AVL-01 | TC-US-302-1–7; TC-NFR-AVL-1 |  | BRD v1.2 (CR-006/Tahap 3) |
| BR-018 | Tarif konsultasi berasal dari rekanan. Tarif dan durasi sesi diisi admin klinik di portal; jadwal prakti… | `[USER] T2-Q6, T3-Q4, T3-Q16` | US-301 | TC-US-301-1–6 |  | BRD v1.2 (CR-006/Tahap 3) |
| BR-020 | Pasien dapat mendaftar ke klinik rekanan berdasarkan jadwal, data klinik, dan harga layanan dari sistem… | `[DOK] P3.1, PS6a` `[USER] T2-Q13` | US-401, US-402, US-602 | TC-US-401-1–4; TC-US-402-1–7; TC-US-602-1–2 |  | BRD v1.1 (CR-005) |
| BR-021 | Pasien dapat mendaftar pemeriksaan di lab rekanan berdasarkan jadwal dan harga dari sistem rekanan | `[DOK] P3.1, PS6a` | US-401, US-402, US-602 | TC-US-401-1–4; TC-US-402-1–7; TC-US-602-1–2 |  | BRD v1.0 |
| BR-022 | Pasien dapat memesan home visit yang disediakan klinik rekanan, dilayani tenaga kesehatan klinik itu ter… | `[DOK] P1.2, PS6a` `[USER] T2-Q5` | US-501, US-502, US-503, US-504, US-1308 | TC-US-501-1–4; TC-US-502-1–3; TC-US-503-1–5; TC-US-504-1–3; TC-US-1308-1–3 |  | BRD v1.0 |
| BR-023 | Pasien dapat melihat informasi biaya MCU dan pemeriksaan lab sebelum mendaftar, berdasarkan harga dari s… | `[DOK] P1.2, PS6a` `[USER] T2-Q20` | US-601, US-602 | TC-US-601-1–3; TC-US-602-1–2 |  | BRD v1.1 (CR-005) |
| BR-024 | Pendaftaran klinik, pendaftaran lab, dan home visit yang dipesan di aplikasi dibayar di aplikasi. Saat d… | `[USER] T2-Q13, T2-Q14` | US-402, US-403, US-404, US-501, US-801, US-1304 | TC-US-402-1–7; TC-US-403-1–4; TC-US-404-1–6; TC-US-501-1–4; TC-US-801-1–9; TC-US-1304-1–4 |  | BRD v1.1 (CR-005) |
| BR-025 | Hasil lab dari lab rekanan diterima lewat integrasi, disimpan, dan diberitahukan ke pasien | `[DOK] P4.1, P4.5, PS6a` | US-901, US-902, US-1307 | TC-US-901-1–5; TC-US-902-1–3; TC-US-1307-1–4 |  | BRD v1.0 |
| BR-026 | Riwayat pendaftaran klinik, lab, dan home visit pasien disimpan | `[DOK] P4.1` | US-406 | TC-US-406-1–2 |  | BRD v1.0 |
| BR-027 | Pasien yang sudah check-in di klinik rekanan dapat melihat antrean secara real-time, dari data antrean s… | `[USER] T3-Q1a, T3-Q1b, T3-Q14` | US-405, US-1305 | TC-US-405-1–3; TC-US-1305-1–3 |  | BRD v1.2 (CR-006/Tahap 3) |
| BR-030 | Pasien dapat membeli obat bebas, produk kecantikan, dan produk kesehatan lain (vitamin, suplemen, alat k… | `[DOK] PS1` | US-701, US-702 | TC-US-701-1–3; TC-US-702-1–3 |  | BRD v1.0 |
| BR-031 | Outlet rekanan terdiri atas toko kecantikan, toko alat kesehatan, klinik rekanan, dan apotek rekanan. Ob… | `[DOK] PS2` `[USER] T2-Q3, T2-Q3b` | US-205, US-702 | TC-US-205-1–7; TC-US-702-1–3 |  | BRD v1.0 |
| BR-032 | Obat resep tidak dijual di Ruang Sehat | `[DOK] PS1` | US-702 | TC-US-702-1–3 |  | BRD v1.0 |
| BR-033 | Katalog, stok, dan pesanan produk berasal dari sistem outlet rekanan lewat integrasi. Ruang Sehat tidak… | `[DOK] PS6, PS6a` | US-701, US-704, US-1402 | TC-US-701-1–3; TC-US-704-1–7; TC-US-1402-1–7 |  | BRD v1.0 |
| BR-034 | Produk dikirim kurir pihak ketiga yang terintegrasi. Ongkos kirim dibayar pasien dan ditambahkan ke tota… | `[DOK] PS5` `[USER] T2-Q7` | US-703, US-704 | TC-US-703-1–7; TC-US-704-1–7 |  | BRD v1.0 |
| BR-035 | Riwayat pesanan produk pasien disimpan | `[DOK] P4.1` | US-706 | TC-US-706-1–2 |  | BRD v1.0 |
| BR-040 | Semua layanan yang dipesan di aplikasi (konsultasi online, pendaftaran klinik, pendaftaran lab, home vis… | `[DOK] P3.3, PS4` `[USER] T2-Q4, T2-Q13` `[SRC-19]` | US-304, US-402, US-501, US-703, US-802, US-804 | TC-US-304-1–8; TC-US-402-1–7; TC-US-501-1–4; TC-US-703-1–7; TC-US-802-1–15; TC-US-804-1–3 |  | BRD v1.1 (CR-005) |
| BR-041 | Payment gateway membagi dana setiap pembayaran langsung: bagian rekanan ke rekening rekanan, bagian Ruma… | `[USER] T2-Q4` | US-803 | TC-US-803-1–7 |  | BRD v1.1 (cakupan meluas, CR-005) |
| BR-042 | Harga yang dibayar pasien untuk konsultasi, pendaftaran klinik, pendaftaran lab, home visit, dan produk… | `[USER] T2-Q1, T2-Q6, T2-Q6a, T2-Q15` | US-801, US-809 | TC-US-801-1–9; TC-US-809-1–6 |  | BRD v1.1 (CR-005) |
| BR-043 | Biaya dari rekanan (langganan atau biaya per pendaftaran yang ditagihkan ke klinik dan lab) dapat diatur… | `[DOK: 01-business-case.md]` `[USER] T1-Q1, T2-Q1, T2-Q16` | US-810 | TC-US-810-1–3 |  | BRD v1.1 (CR-005) |
| BR-044 | Biaya MDR untuk pembayaran QRIS ditanggung rekanan (dipotong dari bagian rekanan) dan tidak dibebankan k… | `[USER] T2-Q8, T2-Q12, T2-Q19` `[SRC-24]` | US-801, US-803 | TC-US-801-1–9; TC-US-803-1–7 |  | BRD v1.1 (CR-005) |
| BR-045 | Refund diberikan bila layanan gagal terjadi (contoh: tenaga kesehatan tidak hadir, barang tidak dikirim… | `[USER] T2-Q9, T2-Q17, T2-Q18, T3-Q5, T3-Q6, T3-Q7, T3-Q8` | US-307, US-504, US-705, US-805, US-806, US-807, US-808, US-812, US-1403 | TC-US-307-1–9; TC-US-504-1–3; TC-US-705-1–4; TC-US-805-1–7; TC-US-806-1–10; TC-US-807-1–10; TC-US-808-1–11; TC-US-812-1–7; TC-US-1403-1–4 |  | BRD v1.2 (CR-006/Tahap 3) |
| BR-046 | Admin Ruang Sehat, admin klinik, admin lab, dan admin outlet dapat melihat laporan transaksi dan pembagi… | `[DOK] P8.2a, PS6c, PS6d` `[USER] T2-Q4` | US-811, US-1201 | TC-US-811-1–5; TC-US-1201-1–5 |  | BRD v1.0 |
| BR-050 | Ruang Sehat mengirim pemberitahuan ke pasien lewat email dan notifikasi di dalam aplikasi; push dan WhatsApp dihapus (CR-007) | `[DOK] P4.5` `[USER] T4-Q1, T4-Q2` | US-902, US-1001, US-1002 | TC-US-902-1–3; TC-US-1001-1–8; TC-US-1002-1–4 |  | BRD v1.3 (CR-007/Tahap 4) |
| BR-060 | Backoffice admin Ruang Sehat menyediakan verifikasi dan pengelolaan akun (pasien, tenaga kesehatan, reka… | `[DOK] P8.2a` | US-207, US-208 | TC-US-207-1–17; TC-US-208-1–27 |  | BRD v1.0 |
| BR-061 | Backoffice admin Ruang Sehat menyediakan CMS untuk banner, halaman info, artikel/konten edukasi kesehata… | `[DOK] P8.2a` | US-1101, US-1102 | TC-US-1101-1–6; TC-US-1102-1–2 |  | BRD v1.0 |
| BR-062 | Backoffice admin Ruang Sehat menyediakan layanan pelanggan, termasuk penanganan keluhan | `[DOK] P8.2a` | US-1003, US-1103, US-1104 | TC-US-1003-1–4; TC-US-1103-1–4; TC-US-1104-1–4 |  | BRD v1.0 |
| BR-063 | Backoffice admin Ruang Sehat menyediakan pengaturan margin per rekanan dan biaya rekanan | `[USER] T2-Q1, T2-Q6` | US-809, US-810, US-812 | TC-US-809-1–6; TC-US-810-1–3; TC-US-812-1–7 |  | BRD v1.0 |
| BR-064 | Portal admin klinik dan admin lab menyediakan laporan transaksi dan pembagian dana, profil, dan keluhan.… | `[DOK] PS6c` `[USER] T3-Q14, T3-Q16` | US-301, US-1104, US-1201, US-1202, US-1203 | TC-US-301-1–6; TC-US-1104-1–4; TC-US-1201-1–5; TC-US-1202-1–3; TC-US-1203-1–2 |  | BRD v1.2 (CR-006/Tahap 3) |
| BR-065 | Portal admin outlet (toko kecantikan, toko alat kesehatan, apotek) menyediakan laporan, pembagian dana,… | `[DOK] PS6d` `[USER] T2-Q3b` | US-1104, US-1201, US-1202, US-1203 | TC-US-1104-1–4; TC-US-1201-1–5; TC-US-1202-1–3; TC-US-1203-1–2 |  | BRD v1.0 |
| BR-070 | Ruang Sehat terintegrasi otomatis dengan sistem klinik, lab, dan outlet rekanan untuk jadwal, harga dan… | `[DOK] PS6a, P4.3` `[USER] T2-Q6, T2-Q13, T3-Q14` | US-404, US-502, US-901, US-1304, US-1401, US-1402, US-1403, US-1404, US-1405 | TC-US-404-1–6; TC-US-502-1–3; TC-US-901-1–5; TC-US-1304-1–4; TC-US-1401-1–3; TC-US-1402-1–7; TC-US-1403-1–4; TC-US-1404-1–5; TC-US-1405-1–4 |  | BRD v1.2 (CR-006/Tahap 3) |
| BR-071 | Ruang Sehat terintegrasi dengan payment gateway (dengan pembagian dana), layanan panggilan video, kurir, dan email; semua penyedia di Indonesia (CR-007) | `[DOK] P4.2` `[USER] T2-Q4, T4-Q1, T4-Q2` | US-1406 | TC-US-1406-1–4 |  | BRD v1.3 (CR-007/Tahap 4) |
| BR-080 | Audit trail append-only mencatat: (1) login, logout, dan percobaan login gagal; (2) akses ke data pasien… | `[DOK] P5.4` `[DOK: adr/ADR-005-audit-trail-append-only.md]` | US-208, US-1501, US-1502, US-1503, US-1504, PRD §7 | TC-US-208-1–27; TC-US-1501-1–18; TC-US-1502-1–5; TC-US-1503-1–3; TC-US-1504-1–11; TC-AUD-1–4; TC-DB-1–5 | US-1501 (F1-T69): `crates/audit/src/requirements.rs`, `crates/audit/tests/forbidden_fields.rs`, `crates/arch-check/src/audit_coverage.rs`, `crates/arch-check/tests/negative_audit_coverage.rs`, `audit-event-coverage.toml`, `apps/rs-backend/tests/audit_support/mod.rs`; F1-T109 (event korporat): `apps/rs-backend/src/modules/corporate/{documents,verification,repo,api}.rs`, `apps/rs-backend/src/modules/identity/{jobs,idp_worker,domain,api}.rs`, `apps/rs-backend/tests/audit_support/mod.rs` (`assert_corporate_events_clean`, `relay_and_verify_tenant_chain`), `apps/rs-backend/tests/corporate_audit_chain.rs`, `apps/rs-backend/tests/identity_keycloak_events.rs`, `apps/rs-backend/tests/corporate_*.rs`, `audit-event-coverage.toml` | BRD v1.0 |
| BR-081 | Audit trail tidak boleh memuat password, PIN, OTP, token, atau nomor kartu/rekening lengkap | `[DOK: adr/ADR-005-audit-trail-append-only.md]` | US-1501, PRD §7 | TC-US-1501-1–18; TC-AUD-1–4 | US-1501 (F1-T69): `crates/audit/src/requirements.rs`, `crates/audit/tests/forbidden_fields.rs`, `crates/arch-check/src/audit_coverage.rs`, `crates/arch-check/tests/negative_audit_coverage.rs`, `audit-event-coverage.toml`, `apps/rs-backend/tests/audit_support/mod.rs`; F1-T109 (event korporat): `apps/rs-backend/src/modules/corporate/{documents,verification,repo,api}.rs`, `apps/rs-backend/src/modules/identity/{jobs,idp_worker,domain,api}.rs`, `apps/rs-backend/tests/audit_support/mod.rs` (`assert_corporate_events_clean`, `relay_and_verify_tenant_chain`), `apps/rs-backend/tests/corporate_audit_chain.rs`, `apps/rs-backend/tests/identity_keycloak_events.rs`, `apps/rs-backend/tests/corporate_*.rs`, `audit-event-coverage.toml` | BRD v1.0 |
| BR-090 | Ruang Sehat menyediakan data untuk mengukur KPI-01 sampai KPI-04 sesuai definisi di bagian 1.2: MAU bert… | `[USER] T2-Q10, T2-Q11` `[DOK: 01-business-case.md]` | US-1405, US-1701 | TC-US-1405-1–4; TC-US-1701-1–5 |  | BRD v1.0 |
| BR-100 | Ruang Sehat terdaftar sebagai Penyelenggara Sistem Elektronik Lingkup Privat sebelum aplikasi digunakan… | `[SRC-12]` `[SRC-13]` `[SRC-27]` | PRD §10 | — (non-perangkat lunak, di luar test plan) |  | BRD v1.2 (CR-006/Tahap 3) |
| BR-101 | Konsultasi online berjalan dalam skema telemedisin yang sesuai ketentuan Kemenkes: klinik rekanan (fasya… | `[SRC-02]` `[SRC-03]` `[USER] T2-Q2` | PRD §10 | — (non-perangkat lunak, di luar test plan) |  | BRD v1.0 |
| BR-102 | Hanya tenaga kesehatan dengan STR dan SIP yang masih berlaku yang dapat diaktifkan melayani. Masa berlak… | `[SRC-02]` `[SRC-03]` `[SRC-21]` | US-202, US-203, US-204, US-1003 | TC-US-202-1–6; TC-US-203-1–5; TC-US-204-1–6; TC-US-1003-1–4 |  | BRD v1.0 |
| BR-103 | Pasien memberikan persetujuan eksplisit, tertulis atau terekam, sebelum datanya diproses, dengan informa… | `[SRC-15]` | US-101, US-104, US-1601 | TC-US-101-1–5, 7–30; TC-US-104-1–13; TC-US-1601-1–6 | US-104 AC1, AC3 (F1-T49): `apps/rs-backend/src/modules/privacy/`, `db/migrations/app/20261126200000_privacy_consent_text_draft_seed.sql`, `apps/rs-backend/tests/privacy_consent_text.rs`; E-13 (`record_granted_consent`) | BRD v1.0 |
| BR-104 | Data anggota keluarga yang masih anak hanya diproses dengan persetujuan orang tua atau wali | `[SRC-15]` | US-104, US-108 | TC-US-104-1–13; TC-US-108-1–7 |  | BRD v1.0 |
| BR-105 | Data kesehatan (catatan tenaga kesehatan, isi chat konsultasi, resep, hasil lab), data anak, dan data ke… | `[SRC-15]` | NFR-SEC-05, NFR-PRIV-03 | TC-NFR-SEC-05-1; TC-NFR-SEC-05-2; TC-NFR-SEC-05-3; TC-NFR-PRIV-03-1; TC-SEC-1–9; TC-DB-1–5 |  | BRD v1.0 |
| BR-106 | Pasien dapat menjalankan hak subjek data: informasi, akses dan salinan, perbaikan, penarikan persetujuan… | `[SRC-15]` `[DOK] P5.4` | US-109, US-310, US-1601, US-1602, US-1603, US-1604, US-1605 | TC-US-109-1–4; TC-US-310-1–5; TC-US-1601-1–6; TC-US-1602-1–4; TC-US-1603-1–8; TC-US-1604-1–3; TC-US-1605-1–3 |  | BRD v1.0 |
| BR-107 | Seluruh kegiatan pemrosesan data pribadi direkam, selaras dengan audit trail BR-080 | `[SRC-15]` | US-104, US-1501, PRD §7 | TC-US-104-1–13; TC-US-1501-1–18; TC-AUD-1–4 | US-1501 (F1-T69): `crates/audit/src/requirements.rs`, `crates/audit/tests/forbidden_fields.rs`, `crates/arch-check/src/audit_coverage.rs`, `crates/arch-check/tests/negative_audit_coverage.rs`, `audit-event-coverage.toml`, `apps/rs-backend/tests/audit_support/mod.rs` | BRD v1.0 |
| BR-108 | Ruang Sehat menjaga keamanan dan kerahasiaan data pribadi yang diprosesnya | `[SRC-15]` `[SRC-13]` | US-103 (MFA dan perangkat hilang, CR-018), US-110, NFR-SEC-02, NFR-SEC-05, NFR-SEC-06, NFR-SEC-11 | TC-NFR-SEC-02-1–5; TC-NFR-SEC-05-1; TC-NFR-SEC-05-2; TC-NFR-SEC-05-3; TC-NFR-SEC-06-1; TC-NFR-SEC-11-1–2; TC-SEC-1–9; TC-DB-1–5; TC-US-103-20–51 (MFA, sesi, CR-017/CR-018); TC-US-110-1–12; TC-US-207-9–17 |  | BRD v1.0; PRD v1.2 final (Checkpoint 2) |
| BR-109 | Tersedia prosedur pemberitahuan kegagalan pelindungan data pribadi secara tertulis paling lambat 3 x 24… | `[SRC-15]` `[SRC-13]` | PRD §10 | — (non-perangkat lunak, di luar test plan) |  | BRD v1.0 |
| BR-110 | Penilaian dampak pelindungan data pribadi (DPIA) dilakukan sebelum go-live | `[SRC-15]` | PRD §10 | — (non-perangkat lunak, di luar test plan) |  | BRD v1.0 |
| BR-111 | Ruang Sehat menunjuk pejabat/petugas pelindungan data pribadi | `[SRC-15]` `[SRC-16]` | US-1603, PRD §10 | TC-US-1603-1–8 |  | BRD v1.0 |
| BR-112 | Setiap jenis data memiliki masa retensi; setelah masa retensi berakhir, data diakhiri pemrosesannya, dih… | `[SRC-15]` `[SRC-25]` | US-1605, US-1606, NFR-RET-01 | TC-US-1605-1–3; TC-US-1606-1–3; TC-NFR-RET-01-1 |  | BRD v1.0 |
| BR-113 | Data disimpan di Indonesia; penyedia pihak ketiga juga memproses dan menyimpan data pribadi di Indonesia, tanpa kecuali | `[DOK] P5.5` `[SRC-15]` `[SRC-13]` `[USER] T4-Q1` | US-1406, NFR-PRIV-01, NFR-PRIV-02, NFR-DR-04 | TC-US-1406-1–4; TC-NFR-PRIV-01-1; TC-NFR-DR-1 |  | BRD v1.3 (CR-007/Tahap 4) |
| BR-114 | Catatan konsultasi dan resep digital memenuhi ketentuan rekam medis elektronik dan resep elektronik yang… | `[SRC-18]` `[SRC-06]` `[SRC-07]` `[USER] T2-Q2` | US-308, US-309, PRD §10 | TC-US-308-1–6; TC-US-309-1–12 |  | BRD v1.0 |
| BR-115 | Pembayaran hanya diproses lewat penyedia jasa pembayaran berizin BI. Model pembagian dana langsung (BR-0… | `[SRC-19]` `[SRC-20]` `[USER] T2-Q4` | US-802, US-1406, NFR-INT-04 | TC-US-802-1–15; TC-US-1406-1–4; TC-NFR-INT-1; TC-NFR-INT-2 |  | BRD v1.0 |
| BR-116 | Obat bebas, produk kecantikan, dan produk kesehatan yang dijual lewat Ruang Sehat memenuhi ketentuan per… | `[SRC-08]` `[SRC-10]` `[SRC-11]` `[USER] T2-Q3` | US-702, PRD §10 | TC-US-702-1–3 |  | BRD v1.0 |
| BR-117 | PT. Arunika memiliki perizinan berusaha yang sesuai untuk kegiatan Ruang Sehat sebelum go-live. Jenis iz… | `[SRC-09]` `[SRC-23]` | PRD §10 | — (non-perangkat lunak, di luar test plan) |  | BRD v1.0 |
| BR-118 | Perkembangan regulasi kesehatan digital dipantau sampai go-live, karena Kemenkes menyatakan akan mengatu… | `[SRC-06]` | PRD §10 | — (non-perangkat lunak, di luar test plan) |  | BRD v1.0 |
| BR-120 | PT. Arunika menyediakan aplikasi RME minimalis, terpisah dari aplikasi dan portal Ruang Sehat, untuk kli… | `[USER] T3-Q1a, T3-Q14, T3-Q15` | US-1301, US-1302 | TC-US-1301-1–6; TC-US-1302-1–6 |  | BRD v1.2 (CR-006/Tahap 3) |
| BR-121 | RME minimalis terhubung ke Ruang Sehat lewat integrasi yang sama dengan sistem rekanan lain (BR-070). Ru… | `[USER] T3-Q14` | US-1301, US-1401 | TC-US-1301-1–6; TC-US-1401-1–3 |  | BRD v1.2 (CR-006/Tahap 3) |
| BR-122 | RME minimalis menyediakan pengelolaan jadwal, daftar layanan, dan harga klinik/lab, termasuk layanan hom… | `[USER] T3-Q1b` | US-1303 | TC-US-1303-1–5 |  | BRD v1.2 (CR-006/Tahap 3) |
| BR-123 | RME minimalis menyediakan pencatatan rekam medis kunjungan. Data rekam medis sesuai ketentuan SATUSEHAT… | `[USER] T3-Q1b` `[SRC-18]` `[SRC-26]` `[SRC-27]` | US-1306, US-1309, US-1310, PRD §10 | TC-US-1306-1–9; TC-US-1309-1–9; TC-US-1310-1–4 |  | BRD v1.2 (CR-006/Tahap 3) |
| BR-124 | RME minimalis menyediakan pengisian dan unggah hasil pemeriksaan lab | `[USER] T3-Q1b` | US-1307 | TC-US-1307-1–4 |  | BRD v1.2 (CR-006/Tahap 3) |
| BR-125 | RME minimalis menyediakan penerimaan appointment dari Ruang Sehat, check-in pasien, antrean real-time kl… | `[USER] T3-Q1b, T2-Q13` | US-1304, US-1305, US-1308 | TC-US-1304-1–4; TC-US-1305-1–3; TC-US-1308-1–3 |  | BRD v1.2 (CR-006/Tahap 3) |
| BR-126 | RME minimalis hanya mencatat pasien yang memesan layanan lewat aplikasi Ruang Sehat | `[USER] T3-Q1c` | US-1306, US-1310 | TC-US-1306-1–9; TC-US-1310-1–4 |  | BRD v1.2 (CR-006/Tahap 3) |
| BR-130 | Situs web company profile sembilan halaman | `[USER] C21-P1-1` | US-1801 | — (dibuat QA dari PRD v1.3) | CR-033 (C-01–C-03, tampilan; US-1801 AC2 urutan layanan dari CMS, AC6 aksesibilitas): `web/situs/src/app/situs.css`, `web/situs/src/konten/struktur.ts`, `web/situs/src/komponen/Beranda.tsx`, `Layanan.tsx`, `HalamanKonten.tsx`, `Artikel.tsx`, `KerangkaSitus.tsx`, `Markdown.tsx` (normalisasi judul), `Ikon.tsx`, `Ornamen.tsx`, `MasukMemberArea.tsx`, `web/situs/src/app/layanan/page.tsx`, `untuk-perusahaan/page.tsx`; test `web/situs/src/konten/struktur.test.ts`, `web/situs/src/komponen/tampilan-cr033.test.tsx`, `a11y.test.tsx`, `markdown.test.tsx`, `web/situs/src/kontras.test.ts`, `web/situs/scripts/periksa-build.mjs` | BRD v1.4 (CR-021) |
| BR-131 | Konten situs lewat CMS backoffice | `[USER] C21-P3-4` | US-1101, US-1801, US-1802 | — (dibuat QA) |  | BRD v1.4 (CR-021) |
| BR-132 | Member area korporat: HR di web, peserta di aplikasi pasien | `[USER] C21-P0-2, C21-P4` | US-1904 | — (dibuat QA) | F1-T107 (US-1904 AC2, AC5; undangan HR lain selesai di commit task F1-T105/F1-T107 sisa, KEPUTUSAN-329–331, endpoint `POST /v1/corporate/member-invitations`, `identity/corporate_hr_invite.rs`; test `apps/rs-backend/tests/corporate_hr_invite.rs`, kasus harness `corporate_isolation.rs`, `infra/keycloak/test/keycloak_check.py` `cek_undangan_hr`): `apps/rs-backend/src/modules/identity/corporate_hr_{domain,repo,service,http}.rs`, `apps/rs-backend/src/modules/corporate/{api,repo}.rs`, `openapi/korporat.json`, `web/packages/api-client/src/index.ts`; test `apps/rs-backend/tests/corporate_hr_members.rs`, `corporate_isolation.rs`; F1-T108 (US-1904 AC2 termasuk formulir Undang HR dengan pesan seragam KEPUTUSAN-306, AC5, AC6): `web/korporat/src/komponen/hr/{KelolaHr,GalatAkses}.tsx`, `web/korporat/src/lib/hr/{klien,model}.ts`, `web/korporat/src/app/(terautentikasi)/{page,kelola-hr/page}.tsx`; test `web/korporat/src/komponen/hr/hr.test.tsx`, `web/korporat/src/hr-e2e.test.ts`. CR-039 (Q32, tampilan "sederhana dan memandu", lima menu, commit `2eaa8e8`): `web/korporat/src/lib/navigasi.ts`, `web/korporat/src/komponen/{NavSamping,KerangkaKorporat}.tsx`, `web/korporat/src/komponen/beranda/Beranda.tsx` (menggantikan `hr/DasborHr.tsx`), `web/korporat/src/lib/beranda/model.ts`, `web/korporat/src/lib/verifikasi/model.ts` (`hitungTerunggah`), `web/korporat/src/app/korporat.css`; perbaikan review KEPUTUSAN-434–439; keputusan Q1/Q2 = A KEPUTUSAN-440–441, penyelarasan wireframe `docs/wireframes/k01-korporat-hr.html` dan `docs/design/brief-web-korporat.md`; test `web/korporat/src/komponen/beranda/{beranda,a11y}.test.tsx`, `komponen/komponen.test.tsx`, `lib/lib.test.ts`. ADR-030 langkah 1–4 (CR-026, fondasi worker IdP realm korporat untuk undangan/pendaftaran HR, commit d5bdca9): `db/migrations/app/20261125100000_identity_idp_provisioning_intent.sql`, `apps/rs-backend/src/modules/identity/idp_{domain,repo,service,config,worker,worker_client}.rs`, `crates/platform-jobs/src/{queue,runner,types}.rs`, `crates/arch-check/src/sources.rs` (`idp-admin-client`), `infra/openbao/policies/rs-{api,worker}.hcl`, `infra/keycloak/realms/korporat.json` (`rs_intent_id`, `rs-backend-idp-worker`); test `apps/rs-backend/tests/identity_idp_worker.rs`, `identity_idp_openbao_policy.rs`, `db/test/tabel-app-a1.sql`, `crates/arch-check/tests/negative_sources.rs`, `infra/keycloak/test/keycloak_check.py` (`cek_pekerja_idp`) CR-028 (perbaikan demo): `web/packages/ui/src/Button.tsx` (`"use client"`), `web/packages/bff-auth/src/store.ts`, `infra/keycloak/realms/{korporat,admin}.json` (audience mapper); test `web/scripts/batas-client.test.ts`, `web/scripts/realm-audience.test.ts`, `web/packages/bff-auth/src/store.test.ts`, `infra/keycloak/test/keycloak_check.py` | BRD v1.4 (CR-021) |
| BR-133 | Layanan korporat: vaksinasi, MCU, lab massal | `[USER] C21-P1-2, C21-P4` | US-1905 | — (dibuat QA) |  | BRD v1.4 (CR-021) |
| BR-134 | Pelaksana klinik/lab rekanan, Ruang Sehat perantara | `[USER] C21-P1-3` | US-1917 | — (dibuat QA) |  | BRD v1.4 (CR-021) |
| BR-135 | Lokasi on-site atau di rekanan per pesanan | `[USER] C21-P3-2` | US-1906, US-1911 | — (dibuat QA) |  | BRD v1.4 (CR-021) |
| BR-136 | Perusahaan daftar sendiri + verifikasi (NIB/NPWP, bukti kewenangan, Verifikator) atau dibuatkan Verifikator; NIB ganda ditolak (**diubah CR-035**, keputusan user Q10 = B2: NIB unik hanya di antara perusahaan Aktif; NIB terdaftar tetap boleh diajukan dan ditandai di antrean Verifikator) | `[USER] C21-P2-1, C21-P4` | US-1901, US-1902, US-1903, US-1916 | — (dibuat QA) | US-1902 AC1–AC5 backend (F1-T104): `apps/rs-backend/src/modules/corporate/verification.rs`, `domain.rs`, `repo.rs`, `http.rs`, `dto.rs`, `policy.rs`, `error.rs`, `api.rs`; `modules/platform/api.rs` (`FileAccess::StaffCorporate`), `platform/domain.rs` (`UploadPurpose::CorporateDocument`); migrasi `db/migrations/app/20261124200000_platform_reason_code_corporate_rejection.sql`, `20261124201000_corporate_rejection_reason_seed.sql` (daftar alasan draf); kontrak `openapi/backoffice.json`; test `apps/rs-backend/tests/corporate_verification.rs`, `db/test/tabel-app-a2.sql`, unit `corporate/domain.rs`, `corporate/dto.rs`, `corporate/verification.rs`; panel backoffice F1-T106 (US-1902 AC1–AC5): `web/backoffice/src/komponen/perusahaan/PanelPerusahaan.tsx`, `src/lib/perusahaan/{klien,model}.ts`, `src/app/(terautentikasi)/verifikasi-perusahaan/page.tsx`, test `src/komponen/perusahaan/perusahaan.test.tsx`; buat perusahaan (US-1903 AC1/AC4 di UI): `web/backoffice/src/komponen/perusahaan/FormBuatPerusahaan.tsx` (+ `buat` di `lib/perusahaan/klien.ts`, `validasiForm` di `model.ts`), test `perusahaan.test.tsx`; daftar perusahaan/HR belum (TD-39); US-1903 AC1–AC4 backend (F1-T105, KEPUTUSAN-328/330/332–334): `apps/rs-backend/src/modules/corporate/{admin_create,admin_create_http}.rs`, `corporate/{api,repo,error,mod}.rs` (4090110, 4090111), `identity/corporate_hr_invite.rs`, kontrak `openapi/backoffice.json` (`POST /backoffice/v1/corporate-clients`); test `apps/rs-backend/tests/corporate_admin_create.rs`, `infra/keycloak/test/keycloak_check.py` (`cek_undangan_hr`: kedaluwarsa 24 jam dan sekali pakai, AC2); US-1901 AC2 dokumen backend (F1-T102, KEPUTUSAN-343–349): `apps/rs-backend/src/modules/corporate/{documents,documents_http}.rs`, `corporate/{api,repo,domain,error,policy,verification,dto,http,mod}.rs` (4090112, 4090113; antrean `readiness`), `modules/platform/{pdf,domain,service,api,repo}.rs` (`FileFormat` + PDF, `PdfInspector`/`QpdfInspector`, `FileAccess::CorporateHr` E-12, `soft_delete_stored_file`, `count_recent_upload_slots`), `app.rs`, `config.rs`, `cli.rs` (`PDF_INSPECTOR_QPDF_PATH`); migrasi `db/migrations/app/20261126350000_platform_upload_slot_owner_index.sql`, `db/test/tabel-app-a19.sql`; kontrak `openapi/korporat.json`, `openapi/backoffice.json`; test `apps/rs-backend/tests/corporate_documents_{access,http}.rs`, `corporate_documents_pdf_qpdf.rs` (qpdf asli, `#[ignore]`, TD-48), `corporate_documents_support/mod.rs`, `corporate_isolation.rs` (empat kasus H-18), unit `platform/pdf.rs`, `platform/domain.rs`, `corporate/{domain,documents,documents_http,error}.rs`; CR-035 NIB unik hanya di antara perusahaan Aktif (Q10 = B2, KEPUTUSAN-362–365): `apps/rs-backend/src/modules/corporate/{registration,admin_create,verification,repo,api,dto,error,http}.rs` (`4090116`); migrasi `db/migrations/app/20261126400000_corporate_client_nib_unique_active.sql`, `20261126410000_corporate_client_nib_unique_any_status_drop.sql`, `db/test/tabel-app-a20.sql`; kontrak `openapi/backoffice.json` (`nib_used_by_other_company`), klien `web/packages/api-client/src/generated/backoffice.ts`; penanda backoffice `web/backoffice/src/komponen/perusahaan/PanelPerusahaan.tsx`, `src/lib/perusahaan/{klien,model}.ts`, test `perusahaan.test.tsx`; test `apps/rs-backend/tests/corporate_nib_shared.rs` (termasuk jalur penjaga index unik deterministik), `corporate_registration_http.rs` (NIB terdaftar identik dengan NIB baru), `corporate_admin_create.rs`; kursor antrean `(submitted_at, id)` (review T1 CR-035, KEPUTUSAN-366): `corporate/{domain,verification,repo,http,dto,api}.rs` (`QueueCursor`), test `apps/rs-backend/tests/corporate_verification.rs` (pengajuan ulang di antara dua halaman), unit `corporate/domain.rs`; US-1901 AC6 profil perusahaan dan US-1902 AC3 pengajuan ulang backend (F1-T102b, KEPUTUSAN-360/361): `apps/rs-backend/src/modules/corporate/{documents,documents_http,api,repo,domain,error}.rs` (`4090115`), kontrak `openapi/korporat.json` (`POST /v1/corporate/verification-resubmissions`, objek `company`), klien `web/packages/api-client/src/generated/korporat.ts`; test `apps/rs-backend/tests/corporate_resubmission.rs`, `corporate_isolation.rs` (kasus H-18), unit `corporate/{domain,documents,documents_http,error}.rs`; US-1901 AC1, AC3–AC7 backend (F1-T101, KEPUTUSAN-321–327): `apps/rs-backend/src/modules/corporate/{registration,registration_http}.rs`, `corporate/{domain,policy,error,api,repo,mod}.rs` (4090106), `identity/corporate_hr_invite.rs` (`register_hr_self_registration`), `privacy/{domain,repo,service}.rs` (`ConsentSubject::CorporateHr`), `crates/audit/src/requirements.rs` (E-13 HR), `crates/authz/src/policy.rs` (`PublicReason::CorporateRegistration`), `app.rs` (`RegistrationGuard`); migrasi `db/migrations/app/20261126300000_privacy_consent_corporate_hr_subject.sql`, `20261126310000_privacy_consent_corporate_hr_text_draft_seed.sql`; kontrak `openapi/public.json`; test `apps/rs-backend/tests/corporate_registration_http.rs`, `db/test/tabel-app-a3.sql`, unit `corporate/domain.rs`, `corporate/registration.rs`, `crates/audit`; UI HR F1-T103 (US-1901 AC2/AC6, US-1902 AC3: status verifikasi, unggah/ganti/unduh dokumen, ajukan ulang): `web/korporat/src/komponen/verifikasi/StatusVerifikasi.tsx`, `src/lib/verifikasi/{klien,model}.ts`, `src/app/(terautentikasi)/verifikasi/page.tsx`, `src/keamanan.ts` + `src/proxy.ts` (`KORPORAT_UPLOAD_ORIGIN`), `src/komponen/hr/GalatAkses.tsx`, `src/lib/navigasi.ts`; test `web/korporat/src/verifikasi-e2e.test.ts` (BFF: CSRF hilang/salah, 4090115/4090001/4090112, 415 PDF, sesi kedaluwarsa), `src/komponen/verifikasi/{verifikasi,a11y}.test.tsx`, `src/lib/verifikasi/model.test.ts`; formulir daftar publik F1-T103 (US-1901 AC1, AC3–AC5, Q30 = C): `web/korporat/src/app/(publik)/daftar/page.tsx`, `src/komponen/daftar/FormDaftar.tsx`, `src/lib/daftar/{klien,model}.ts`, `infra/caddy/snippets/korporat-pendaftaran-publik.caddy`, ADR-031 (diusulkan); test `src/lib/daftar/daftar.test.ts`, `src/komponen/daftar/daftar.test.tsx` (versi persetujuan basi 4090106, 422, gagal jaringan/429/503, axe, HTML SSR terkunci dan POST), `web/scripts/caddy-snippet.test.ts` (rewrite path tetap dan batas badan 16 KB sebelum reverse_proxy); tombol situs `web/situs/src/komponen/MasukMemberArea.tsx`, test `member-area.test.tsx`; IP klien asli dari proxy tepercaya untuk batas laju pendaftaran dan sumber audit (TD-22, KEPUTUSAN-400/401, prasyarat ADR-031): `apps/rs-backend/src/http_support.rs` (`TrustedProxies`), `app.rs` (middleware IP klien), `config.rs` + `cli.rs` (`TRUSTED_PROXY_CIDRS`); test `apps/rs-backend/tests/http_trusted_proxy.rs` (termasuk E-05 lewat proxy dan IPv6 /64), unit `http_support.rs`, `config.rs` (production wajib, KEPUTUSAN-403); kunci batas laju IPv6 /64 `apps/rs-backend/src/rate_limit.rs` (`RateKey`, KEPUTUSAN-402, TD-56), unit `rate_limit.rs`; CR-036 koreksi data perusahaan saat Ditolak bersama pengajuan ulang (Q20 = A, KEPUTUSAN-410–414): `apps/rs-backend/src/modules/corporate/correction.rs` (validasi, kebijakan audit nama field), `corporate/{documents,documents_http,repo,api,mod}.rs` (`POST /v1/corporate/company-corrections`, `repo::resubmit_client` dengan `ClientCorrection`), `app.rs` (hitungan rute di test); kontrak `openapi/korporat.json` (+ 429 pengajuan ulang), klien `web/packages/api-client/src/generated/korporat.ts`; UI `web/korporat/src/komponen/verifikasi/StatusVerifikasi.tsx` (`FormKoreksi`), `src/lib/verifikasi/{klien,koreksi}.ts`, `src/lib/daftar/model.ts` (`periksaSatu` diekspor); test `apps/rs-backend/tests/corporate_correction.rs`, `corporate_isolation.rs` (kasus H-18), unit `corporate/correction.rs`, `web/korporat/src/verifikasi-e2e.test.ts` (BFF: CSRF hilang/salah, 422 per field, 409/429, sesi kedaluwarsa), `src/komponen/verifikasi/{verifikasi,a11y}.test.tsx`, `src/lib/verifikasi/koreksi.test.ts` | BRD v1.4 (CR-021) |
| BR-137 | Pesan dari katalog harga tetap | `[USER] C21-P2-2` | US-1905, US-1906 | — (dibuat QA) |  | BRD v1.4 (CR-021) |
| BR-138 | Harga korporat = harga rekanan (lewat integrasi) + margin | `[USER] C21-P3-1, C21-P4` | US-1905 | — (dibuat QA) |  | BRD v1.4 (CR-021) |
| BR-139 | HR unggah peserta Excel/CSV; NIK hanya bila wajib | `[USER] C21-P2-3` | US-1907 | — (dibuat QA) |  | BRD v1.4 (CR-021) |
| BR-140 | Undangan email; akun pasien yang sama | `[USER] C21-P2-3` | US-1908 | — (dibuat QA) |  | BRD v1.4 (CR-021) |
| BR-141 | Invoice B2B tanpa payment gateway (pengecualian BR-040) | `[USER] C21-P1-4, C21-P4` | US-1909, US-1910 | — (dibuat QA) |  | BRD v1.4 (CR-021) |
| BR-142 | Invoice saat konfirmasi, jatuh tempo 14 hari, jadwal setelah lunas | `[USER] C21-P3-3` | US-1906, US-1909, US-1911 | — (dibuat QA) |  | BRD v1.4 (CR-021) |
| BR-143 | Hasil lengkap hanya untuk peserta | `[USER] C21-P2-4` | US-1915 | — (dibuat QA) |  | BRD v1.4 (CR-021) |
| BR-144 | HR: rekap kehadiran; ringkasan hanya dengan persetujuan | `[USER] C21-P2-4` | US-1912, US-1913, US-1914 | — (dibuat QA) |  | BRD v1.4 (CR-021) |
| BR-145 | Data peserta dan hasil sebagai data pribadi (spesifik); isolasi per perusahaan; audit | BR-105, BR-080, BR-107 `[SRC-440]` | US-1904, US-1907, US-1908, US-1913, NFR-SEC-06 | — (dibuat QA) | F1-T107: endpoint `/v1/corporate/members` terdaftar di harness isolasi, sidik data diperluas ke akun HR (TD-34); test `apps/rs-backend/tests/corporate_isolation.rs` | BRD v1.4 (CR-021) |

## NFR → arsitektur (Tahap 4)

Pemetaan setiap NFR PRD §6 dan spesifikasi audit trail §7 ke jawaban arsitektur, ADR, dan cara pembuktiannya ada di `docs/04-arsitektur.md` §10. Ringkasan per kelompok:

| Kelompok NFR | ADR |
| --- | --- |
| NFR-PERF-01–05, NFR-CAP-01–04 | ADR-006, ADR-007, ADR-014, ADR-016, ADR-019 |
| NFR-AVL-01–04, NFR-DR-01–04 | ADR-019, ADR-010 |
| NFR-SEC-01–09 | ADR-011, ADR-012, ADR-013, ADR-018, ADR-009 |
| NFR-PRIV-01–05 | ADR-013, ADR-014, ADR-016, ADR-019, ADR-008 |
| NFR-RET-01–02 | ADR-013, ADR-009 |
| NFR-COMP-01–03, NFR-LOC-01–02 | ADR-006, ADR-015, ADR-017 |
| NFR-OBS-01–03 | ADR-010 |
| NFR-INT-01–04 | ADR-014, ADR-015, ADR-007 |
| Audit trail PRD §7 (BR-080, BR-081, BR-107) | ADR-009, ADR-010 |
| Model data (semua BR) | `docs/04-erd.md`, `docs/04-data-dictionary.md`, ADR-008 |

## NFR dan butir arsitektur → stack (Tahap 5)

Rincian per butir ada di `docs/05-stack.md` §12 dan `docs/05-asvs-checklist.md`. Kolom kode/test diisi di Tahap 8.

| NFR / butir | Komponen stack | Dokumen |
| --- | --- | --- |
| NFR-PERF-01–05, NFR-CAP-01–04 | Rust (axum, tokio), Valkey Pub/Sub, LiveKit self-hosted; ukuran VPS divalidasi uji beban | ADR-024, ADR-025, 05-stack.md §2 |
| NFR-AVL-01–04, NFR-DR-01–04 | Dua host aplikasi, standby PostgreSQL di Hostinger, pgBackRest PITR (repo Biznet + Hostinger), pemeriksa ketersediaan dari Hostinger | ADR-025, 05-stack.md §2, §4 |
| NFR-SEC-01 (ASVS L2) | Checklist per bab V1–V17 | 05-asvs-checklist.md |
| NFR-SEC-02–04, NFR-SEC-10 | Keycloak 26.8.2 (konfigurasi realm sebagai kode), BFF Next.js, flutter_appauth, local_auth | 05-stack.md §7, ADR-024 |
| NFR-SEC-05, NFR-RET-01 | OpenBao Transit (KEK), aws-lc-rs (AES-256-GCM), format ciphertext | ADR-025, 05-physical-schema.md, coding-standards.md |
| NFR-SEC-06–07 | Repository ber-scope tenant (Rust), RLS RME, role database | 05-physical-schema.md, coding-standards.md |
| NFR-SEC-08 | Object storage NEO, URL bertanda tangan, qpdf | 05-stack.md, KEPUTUSAN-86 |
| NFR-PRIV-01–02 | Semua host dan penyedia di Indonesia; konfirmasi tertulis vendor (PT-36, PT-37, PT-39) | CR-012, 05-stack.md §10 |
| NFR-PRIV-04 | LiveKit tanpa Egress | KEPUTUSAN-87 |
| NFR-OBS-01–03 | Prometheus, Loki (tenant app/security), Grafana, GlitchTip, tracing JSON dengan redaksi | ADR-025, 05-stack.md §4, coding-standards.md |
| NFR-INT-01–04 | Antrean job PostgreSQL (SKIP LOCKED, advisory lock), adaptor DOKU, Lalamove | 05-stack.md, KEPUTUSAN-74, CR-012 |
| ADR-022, ADR-023 | utoipa OpenAPI 3.1, Swagger UI staging, AsyncAPI, registry `response_code`, contract test di skrip verifikasi | 05-stack.md §6, §8, 05-response-codes.md |
| AB-11, AB-12, US-801, US-812 (CR-013 pajak) | Komponen pajak di harga terkunci dan jurnal; diterapkan sebelum EP-08 | backlog.md CR-013 |

## Story → wireframe (Tahap 6)

Peta alur lengkap dan deskripsi terstruktur: `docs/06-design-guideline.md` §10. Berkas HTML di `docs/wireframes/`. "D-xx" = deskripsi terstruktur di §10.2; "—" = tanpa layar sendiri (§10.3).

| Epic | Story → alur/wireframe |
| --- | --- |
| EP-01 | US-101 A01, A15 (D-A15); US-103 A01, A15 (D-A15); US-104 A01, A13; US-105 A03; US-106 A03; US-107 A02, A03, A05; US-108 A04; US-109 A04; US-110 G01 (D-G01) |
| EP-02 | US-201 C02; US-202 B01; US-203 C02 (pola); US-204 — (B01 peringatan SIP); US-205 D-C04; US-206 D-C04, C01; US-207 D-C05 (termasuk AC5 reset MFA pasien); US-208 D-C05, C01 |
| EP-03 | US-301 D01; US-302 B02, D01; US-303 A05; US-304 A05; US-305 A06, B02; US-306 A06, B02; US-307 A06, B02; US-308 B02, A06; US-309 B02, A06, D-F01; US-310 A06; US-311 B02 |
| EP-04 | US-401 A08; US-402 A08, A07; US-403 A08; US-404 — (A08, E01); US-405 A08; US-406 A08 |
| EP-05 | US-501 A10; US-502 D-E05, B03; US-503 B03; US-504 A10 |
| EP-06 | US-601 A09; US-602 A09 |
| EP-07 | US-701–US-705 A11; US-706 A11, A08 |
| EP-08 | US-801 A07, A11; US-802 A07, C03; US-803 — (C03, D01); US-804 A07; US-805 A12; US-806 — (A12); US-807 A12, A10, A11, C03; US-808 C03, A12; US-809 D-C06; US-810 D-C06; US-811 C03; US-812 D-C06 |
| EP-09 | US-901 — (A02, A08); US-902 A08 |
| EP-10 | US-1001 A02, D-F02; US-1002 D-F02; US-1003 B02, D-F02 |
| EP-11 | US-1101 D-C07; US-1102 A02; US-1103 D-A14; US-1104 D-C08 |
| EP-12 | US-1201 D01; US-1202 D-D02; US-1203 D-D02 |
| EP-13 | US-1301 D-E02; US-1302 D-E02, C01; US-1303 D-E03; US-1304 E01; US-1305 E01; US-1306 E01; US-1307 D-E04; US-1308 D-E05; US-1309 E01; US-1310 D-E06 |
| EP-14 | US-1401–US-1404 —; US-1405 D-C09; US-1406 — |
| EP-15 | US-1501 —; US-1502 D-C10; US-1503 D-C10; US-1504 — |
| EP-16 | US-1601 A13; US-1602 A13; US-1603 D-C11; US-1604 A13; US-1605 D-C11 (terblokir L-06/PT-24); US-1606 — |
| EP-17 | US-1701 D-C12 |
| NFR | NFR-SEC-02 C01, B01, A15 (D-A15, pasien); NFR-SEC-04 C01; NFR-SEC-11 G01 (D-G01); NFR-SEC-08 A03, A04, B01; NFR-SEC-10 B01; PRD §6.11 aksesibilitas → 06-design-guideline.md §7 (KEPUTUSAN-160) |

## BR → story → fase (Tahap 7)

Pembagian fase `[UP-65]` `[UP-66]`, disetujui di Checkpoint 2 `[USER] C2-Q5, C2-Q6`. Rincian: `docs/07-phase-plan.md`. "F1"–"F4" = Fase 1–4 Tahap 8; Fase 4 bersyarat (gerbang re-plan 2026-12-11). Story dengan AC yang diverifikasi di fase lain ditandai di `docs/backlog.md` kolom Fase. Butir kepatuhan non-perangkat lunak (PRD §10) tidak masuk fase Tahap 8 dan dipantau sebagai gerbang (`docs/07-phase-plan.md` §9).

| BR | Story (fase) | Enabler / UI terkait (fase) |
| --- | --- | --- |
| BR-001 | US-101 (F1), US-103 (F1; jalur Layanan Pelanggan AC8 di F2; v0.3: AC6, AC7 realm `pasien`/`admin` di F2), US-110 (F2; web RME F3) | EN-06, EN-13, UI-03 (F1) |
| BR-002 | US-105 (F1), US-108 (F4) | — |
| BR-003 | US-106 (F1), US-107 (F2) | EN-10, EN-14 (F1) |
| BR-004 | US-106 (F1), US-201 (F1), US-107 (F2) | EN-14 (F1) |
| BR-005 | US-202, US-203 (F2) | — |
| BR-006 | US-205, US-206 (F2) | — |
| BR-007 | NFR-COMP-01 | EN-12, UI-02 (F1); EN-17 (F2); UI-05 (F3) |
| BR-008 | US-201 (F1), US-107, US-310 (F2), US-902 (F3), US-108, US-109 (F4) | — |
| BR-010 | US-303, US-304, US-305, US-306 (F2) | EN-18, EN-19 (F2) |
| BR-011 | US-303, US-304 (F2) | — |
| BR-012 | US-303, US-304 (F2; nomor 4 urutan pengurangan) | — |
| BR-013 | US-308, US-311, US-1003 (F2) | EN-17 (F2) |
| BR-014 | US-303, US-804 (F2) | — |
| BR-015 | US-309 (F2) | — |
| BR-016 | US-305, US-306, US-308, US-310 (F2) | EN-18 (F2) |
| BR-017 | US-302 (F2), NFR-AVL-01 | EN-16 (F2), EN-25 (F3) |
| BR-018 | US-301 (F2) | — |
| BR-020 | US-401, US-402, US-602 (F3) | — |
| BR-021 | US-401, US-402, US-602 (F3) | — |
| BR-022 | US-501, US-502, US-503, US-504, US-1308 (F4) | — |
| BR-023 | US-601, US-602 (F3) | — |
| BR-024 | US-801 (F2), US-402, US-403, US-404, US-1304 (F3), US-501 (F4) | — |
| BR-025 | US-901, US-902, US-1307 (F3) | — |
| BR-026 | US-406 (F3) | — |
| BR-027 | US-405, US-1305 (F4) | — |
| BR-030 | US-701, US-702 (F4) | — |
| BR-031 | US-205 (F2), US-702 (F4) | — |
| BR-032 | US-702 (F4) | — |
| BR-033 | US-1402 (F3), US-701, US-704 (F4) | — |
| BR-034 | US-703, US-704 (F4) | — |
| BR-035 | US-706 (F4) | — |
| BR-040 | US-304, US-802, US-804 (F2), US-402 (F3), US-501, US-703 (F4) | — |
| BR-041 | US-803 (F2) | — |
| BR-042 | US-801, US-809 (F2) | EN-24 pajak CR-013 (F2) |
| BR-043 | US-810 (F3) | — |
| BR-044 | US-801, US-803 (F2) | — |
| BR-045 | US-307, US-805, US-806, US-808, US-812 (F2), US-807, US-1403 (F3), US-504, US-705 (F4) | — |
| BR-046 | US-811, US-1201 (F3) | — |
| BR-050 | US-1001, US-1002 (F2), US-902 (F3) | EN-13 (F1), UI-04 (F2) |
| BR-060 | US-208 (F1; AC8 di F2), US-207 (F2; termasuk AC5 reset MFA pasien, CR-018) | EN-06, EN-07, EN-11, UI-01 (F1) |
| BR-061 | US-1101 (F1 sejak v0.3, CR-021), US-1102 (F3) | — |
| BR-062 | US-1003 (F2), US-1103, US-1104 (F3) | — |
| BR-063 | US-809, US-812 (F2), US-810 (F3) | — |
| BR-064 | US-301 (F2), US-1104, US-1201, US-1202, US-1203 (F3) | EN-20 (F2) |
| BR-065 | US-1104, US-1201, US-1202, US-1203 (F3) | EN-20 (F2) |
| BR-070 | US-404, US-901, US-1304, US-1401, US-1402, US-1403, US-1404, US-1405 (F3), US-502 (F4) | EN-21 (F3) |
| BR-071 | US-1406 (F2 pembayaran, video, email; F4 kurir) | — |
| BR-080 | US-208, US-1501, US-1504 (F1), US-1502, US-1503 (F3), PRD §7 | EN-04, EN-05, EN-08 (F1) |
| BR-081 | US-1501 (F1), PRD §7 | EN-05 (F1) |
| BR-090 | US-1405, US-1701 (F3) | — |
| BR-100, BR-101, BR-109, BR-110, BR-117, BR-118 | PRD §10 (non-perangkat lunak) | Gerbang G-14 dan pemantauan di setiap gate |
| BR-102 | US-202, US-203, US-204, US-1003 (F2) | — |
| BR-103 | US-101, US-104 (F1), US-1601 (F3) | — |
| BR-104 | US-104 (F1), US-108 (F4) | — |
| BR-105 | NFR-SEC-05, NFR-PRIV-03 | EN-07, EN-09 (F1) |
| BR-106 | US-310 (F2), US-1601–US-1605 (F3; US-1605 terblokir PT-24), US-109 (F4) | — |
| BR-107 | US-104, US-1501 (F1), PRD §7 | EN-05 (F1) |
| BR-108 | US-103 (F1, CR-018), US-110 (F2; web RME F3), NFR-SEC-02, NFR-SEC-05, NFR-SEC-06, NFR-SEC-11 | EN-02, EN-06, EN-07, EN-09 (F1) |
| BR-111 | US-1603 (F3), PRD §10 | — |
| BR-112 | US-1605, US-1606 (F3, terblokir PT-24), NFR-RET-01 | EN-23 (F3) |
| BR-113 | US-1406 (F2), NFR-PRIV-01, NFR-PRIV-02, NFR-DR-04 | EN-15 (F2), EN-26, EN-27 (Februari 2027) |
| BR-114 | US-308, US-309 (F2), PRD §10 | — |
| BR-115 | US-802, US-1406 (F2), NFR-INT-04 | — |
| BR-116 | US-702 (F4), PRD §10 | — |
| BR-120 | US-1301, US-1302 (F3) | EN-22 (F3) |
| BR-121 | US-1301, US-1401 (F3) | EN-21 (F3) |
| BR-122 | US-1303 (F3) | — |
| BR-123 | US-1306, US-1309, US-1310 (F3), PRD §10 | EN-22 (F3) |
| BR-124 | US-1307 (F3) | — |
| BR-125 | US-1304 (F3), US-1305, US-1308 (F4) | — |
| BR-126 | US-1306, US-1310 (F3) | — |
| BR-130 | US-1801 (F1) | F1-T83, F1-T89, F1-T111 (F1) |
| BR-131 | US-1101, US-1801, US-1802 (F1) | F1-T91–F1-T95 (F1) |
| BR-132 | US-1904 (F1; AC7–AC9 di F2) | F1-T85–F1-T90 (F1) |
| BR-133, BR-137, BR-138 | US-1905 (F3), US-1906 (F3) | EN-21 (F3) |
| BR-134 | US-1917 (F3) | EN-21 (F3) |
| BR-135 | US-1906, US-1911 (F3) | — |
| BR-136 | US-1901, US-1902, US-1903, US-1916 (F1; pesanan dan invoice US-1916 di F3) | F1-T98, F1-T101–F1-T106 (F1; F1-T102 dokumen) |
| BR-139, BR-140 | US-1907, US-1908 (F3) | — |
| BR-141, BR-142 | US-1909, US-1910, US-1911 (F3) | Gerbang G-25 |
| BR-143 | US-1915 (F3) | — |
| BR-144 | US-1912, US-1913, US-1914 (F3) | Gerbang G-26 |
| BR-145 | US-1904 (F1), US-1907, US-1908, US-1913 (F3), NFR-SEC-06 | F1-T100, F1-T109 (F1) |

Hitungan v0.3 (CR-021, S3 `[USER] C21-P4`): F1 17 (9 + US-1101 dari F3 + US-1801, US-1802, US-1901–US-1904, US-1916), F2 32, F3 52 (41 − US-1101 + US-1905–US-1915, US-1917), F4 15; total 116 story aktif. Hitungan v0.2 (sebelum CR-021): F1 9, F2 32, F3 41, F4 15 (total 97 story aktif; US-102 dihapus CR-007; US-110 ditambahkan di Checkpoint 2 dan dihitung di F2, dengan bagian web RME diverifikasi di F3). Fase 4 tetap bersyarat; dengan baseline Opsi B `[USER] C2-Q1`, pemindahan story Fase 4 mengikuti `docs/07-phase-plan.md`. Enabler EN-01–EN-27 dan story UI UI-01–UI-05 beserta induk ADR/KEPUTUSAN ada di `docs/backlog.md`.

## Story → test case (Tahap 7)

Sumber: `docs/test-plan.md` (versi 0.2, disesuaikan dengan PRD v1.2 final setelah Checkpoint 2; belum dieksekusi). ID test case berbentuk `TC-<story>-<n>`. "Fase" = fase pengembangan tempat test case dijalankan (sebagian AC story ditunda ke fase lain, lihat test-plan §9 T-29). "T1" = jumlah test case bertanda modul kritis T1; "Neg." = jumlah test case bertipe negatif. Setiap story memiliki minimal satu test case per AC yang dapat diuji; AC berisi `[TIDAK DIKETAHUI]` dicatat di test-plan §9 T-28.

| Story | Fase | Test case | Jumlah | T1 | Neg. |
| --- | --- | --- | --- | --- | --- |
| US-101 | F1 | TC-US-101-1–5, 7–30 | 29 | 29 | 19 |
| US-103 | F1, F2, F3 | TC-US-103-1–51 | 51 | 50 | 27 |
| US-104 | F1, F2, Tahap 12 | TC-US-104-1–13 | 13 | 13 | 7 |
| US-105 | F1, F2 | TC-US-105-1–25 | 25 | 17 | 9 |
| US-106 | F1, F4 | TC-US-106-1–34 | 34 | 34 | 19 |
| US-107 | F2, F4 | TC-US-107-1–5 | 5 | 3 | 3 |
| US-108 | F4 | TC-US-108-1–7 | 7 | 6 | 3 |
| US-109 | F4 | TC-US-109-1–4 | 4 | 4 | 1 |
| US-110 | F2, F3 | TC-US-110-1–12 | 12 | 12 | 6 |
| US-201 | F1, F2, F4 | TC-US-201-1–27 | 27 | 26 | 17 |
| US-202 | F2 | TC-US-202-1–6 | 6 | 5 | 4 |
| US-203 | F2 | TC-US-203-1–5 | 5 | 5 | 4 |
| US-204 | F2 | TC-US-204-1–6 | 6 | 5 | 1 |
| US-205 | F2, F4 | TC-US-205-1–7 | 7 | 5 | 5 |
| US-206 | F2 | TC-US-206-1–5 | 5 | 5 | 4 |
| US-207 | F2, F4 | TC-US-207-1–17 | 17 | 17 | 11 |
| US-208 | F1, F2 | TC-US-208-1–27 | 27 | 27 | 20 |
| US-301 | F2 | TC-US-301-1–6 | 6 | 3 | 2 |
| US-302 | F2 | TC-US-302-1–7 | 7 | 4 | 4 |
| US-303 | F2 | TC-US-303-1–4 | 4 | 0 | 3 |
| US-304 | F2 | TC-US-304-1–8 | 8 | 7 | 3 |
| US-305 | F2 | TC-US-305-1–10 | 10 | 5 | 5 |
| US-306 | F2 | TC-US-306-1–6 | 6 | 2 | 3 |
| US-307 | F2 | TC-US-307-1–9 | 9 | 9 | 3 |
| US-308 | F2 | TC-US-308-1–6 | 6 | 6 | 4 |
| US-309 | F2 | TC-US-309-1–12 | 12 | 11 | 6 |
| US-310 | F2, F4 | TC-US-310-1–5 | 5 | 3 | 1 |
| US-311 | F2 | TC-US-311-1–5 | 5 | 2 | 1 |
| US-401 | F3 | TC-US-401-1–4 | 4 | 1 | 3 |
| US-402 | F3 | TC-US-402-1–7 | 7 | 7 | 5 |
| US-403 | F3 | TC-US-403-1–4 | 4 | 1 | 3 |
| US-404 | F3 | TC-US-404-1–6 | 6 | 6 | 4 |
| US-405 | F4 | TC-US-405-1–3 | 3 | 0 | 1 |
| US-406 | F3 | TC-US-406-1–2 | 2 | 0 | 0 |
| US-501 | F4 | TC-US-501-1–4 | 4 | 2 | 2 |
| US-502 | F4 | TC-US-502-1–3 | 3 | 3 | 1 |
| US-503 | F4 | TC-US-503-1–5 | 5 | 5 | 3 |
| US-504 | F4 | TC-US-504-1–3 | 3 | 1 | 1 |
| US-601 | F3 | TC-US-601-1–3 | 3 | 0 | 1 |
| US-602 | F3 | TC-US-602-1–2 | 2 | 0 | 1 |
| US-701 | F4 | TC-US-701-1–3 | 3 | 1 | 1 |
| US-702 | F4 | TC-US-702-1–3 | 3 | 2 | 2 |
| US-703 | F4 | TC-US-703-1–7 | 7 | 5 | 3 |
| US-704 | F4 | TC-US-704-1–7 | 7 | 7 | 2 |
| US-705 | F4 | TC-US-705-1–4 | 4 | 4 | 3 |
| US-706 | F4 | TC-US-706-1–2 | 2 | 0 | 0 |
| US-801 | F2, F4 | TC-US-801-1–9 | 9 | 7 | 3 |
| US-802 | F2 | TC-US-802-1–15 | 15 | 15 | 9 |
| US-803 | F2 | TC-US-803-1–7 | 7 | 7 | 4 |
| US-804 | F2 | TC-US-804-1–3 | 3 | 1 | 1 |
| US-805 | F2 | TC-US-805-1–7 | 7 | 7 | 3 |
| US-806 | F2, F3, F4 | TC-US-806-1–10 | 10 | 10 | 3 |
| US-807 | F3, F4 | TC-US-807-1–10 | 10 | 10 | 6 |
| US-808 | F2, F3 | TC-US-808-1–11 | 11 | 11 | 6 |
| US-809 | F2 | TC-US-809-1–6 | 6 | 6 | 3 |
| US-810 | F3 | TC-US-810-1–3 | 3 | 3 | 1 |
| US-811 | F3 | TC-US-811-1–5 | 5 | 5 | 4 |
| US-812 | F2 | TC-US-812-1–7 | 7 | 7 | 3 |
| US-901 | F3 | TC-US-901-1–5 | 5 | 5 | 3 |
| US-902 | F3, F4 | TC-US-902-1–3 | 3 | 3 | 1 |
| US-1001 | F2 | TC-US-1001-1–8 | 8 | 3 | 5 |
| US-1002 | F2 | TC-US-1002-1–4 | 4 | 1 | 2 |
| US-1003 | F2 | TC-US-1003-1–4 | 4 | 1 | 1 |
| US-1101 | F3 | TC-US-1101-1–6 | 6 | 2 | 3 |
| US-1102 | F3 | TC-US-1102-1–2 | 2 | 0 | 1 |
| US-1103 | F3 | TC-US-1103-1–4 | 4 | 1 | 1 |
| US-1104 | F3 | TC-US-1104-1–4 | 4 | 2 | 2 |
| US-1201 | F3 | TC-US-1201-1–5 | 5 | 4 | 3 |
| US-1202 | F3 | TC-US-1202-1–3 | 3 | 3 | 2 |
| US-1203 | F3 | TC-US-1203-1–2 | 2 | 2 | 1 |
| US-1301 | F3 | TC-US-1301-1–6 | 6 | 6 | 4 |
| US-1302 | F3 | TC-US-1302-1–6 | 6 | 6 | 4 |
| US-1303 | F3 | TC-US-1303-1–5 | 5 | 4 | 3 |
| US-1304 | F3 | TC-US-1304-1–4 | 4 | 4 | 1 |
| US-1305 | F4 | TC-US-1305-1–3 | 3 | 0 | 0 |
| US-1306 | F3, F4 | TC-US-1306-1–9 | 9 | 9 | 6 |
| US-1307 | F3 | TC-US-1307-1–4 | 4 | 4 | 3 |
| US-1308 | F4 | TC-US-1308-1–3 | 3 | 2 | 1 |
| US-1309 | F3 | TC-US-1309-1–9 | 9 | 9 | 5 |
| US-1310 | F3 | TC-US-1310-1–4 | 4 | 4 | 2 |
| US-1401 | F3 | TC-US-1401-1–3 | 3 | 3 | 1 |
| US-1402 | F3 | TC-US-1402-1–7 | 7 | 7 | 6 |
| US-1403 | F3 | TC-US-1403-1–4 | 4 | 4 | 2 |
| US-1404 | F3 | TC-US-1404-1–5 | 5 | 5 | 3 |
| US-1405 | F3 | TC-US-1405-1–4 | 4 | 1 | 1 |
| US-1406 | F2, F4 | TC-US-1406-1–4 | 4 | 4 | 2 |
| US-1501 | F1, F2 | TC-US-1501-1–18 | 18 | 18 | 11 |
| US-1502 | F3 | TC-US-1502-1–5 | 5 | 4 | 2 |
| US-1503 | F3 | TC-US-1503-1–3 | 3 | 3 | 2 |
| US-1504 | F1, F2 | TC-US-1504-1–11 | 11 | 11 | 10 |
| US-1601 | F3 | TC-US-1601-1–6 | 6 | 6 | 3 |
| US-1602 | F3 | TC-US-1602-1–4 | 4 | 4 | 2 |
| US-1603 | F3 | TC-US-1603-1–8 | 8 | 8 | 3 |
| US-1604 | F3 | TC-US-1604-1–3 | 3 | 3 | 1 |
| US-1605 | F3 | TC-US-1605-1–3 | 3 | 3 | 1 |
| US-1606 | F3 | TC-US-1606-1–3 | 3 | 3 | 2 |
| US-1701 | F3 | TC-US-1701-1–5 | 5 | 1 | 1 |

Test case lintas story dan NFR:

| Kelompok | Test case | Jumlah |
| --- | --- | --- |
| Kriteria umum KU-01–KU-10 | TC-KU-01-1 … TC-KU-10-2 | 16 |
| Skenario wajib endpoint dan isolasi | TC-SEC-1 … TC-SEC-9 | 9 |
| Kebutuhan non-fungsional (NFR) | TC-NFR-SEC-02-1 … TC-NFR-A11Y-2 | 47 |
| Audit trail (PRD §7) | TC-AUD-1 … TC-AUD-4 | 4 |
| Fondasi database dan hak role | TC-DB-1 … TC-DB-5 | 5 |
| MVP end-to-end Fase 1 | TC-MVP-1 … TC-MVP-2 | 2 |

Hitungan total ada di `docs/test-plan.md` §2.9. Kolom "File kode / test" diisi di Tahap 8 lewat penanda `trace:` (coding standards §7.4).

Klarifikasi AC Fase 1 (temuan test-plan §9 T-01–T-22; US-101, US-103–US-106, US-201, US-208, US-1501, US-1504, KU-02, KU-10, NFR-SEC-03, NFR-SEC-08) ada di `docs/07-klarifikasi-ac-fase-1.md`. Pemetaan T-23–T-35 ke fase ada di §4 dokumen itu.

**Perubahan AC setelah Checkpoint 2 (PRD v1.2 final, 2026-11-18).** Baris berikut menandai AC yang berubah atau baru. Test case-nya sudah diperbarui di `docs/test-plan.md` versi 0.2 (kolom "Test case"), dan kolom jumlah di tabel di atas sudah disesuaikan (total 805 test case, test-plan §2.9). Temuan baru dari AC ini: test-plan §9 T-36–T-43.

| Story / kriteria | AC berubah atau baru | Asal | Fase | Test case (test-plan v0.2) |
| --- | --- | --- | --- | --- |
| KU-02, KU-10 | Rumusan ulang (batas waktu klien; pesan seragam saat masuk dan layar "Akun nonaktif" saat sesi) | T-20, T-09, UP-73 | Semua | TC-KU-02-1, -3; TC-KU-10-1, -2; TC-SEC-3 |
| US-101 | AC1a, AC1b, AC1c, AC2, AC2b, AC3 (rumusan ulang); AC5 baru (pendaftaran TOTP dan kode pemulihan) | T-01–T-04, UP-73; CR-018 | F1 | TC-US-101-2, -4, -7, -8, -11, -12 (diubah); TC-US-101-16–30 (baru) |
| US-103 | AC1 (dengan TOTP), AC1a, AC1b baru, AC2, AC3, AC4a–AC4e, AC5; AC6 baru (CR-017); AC7, AC8 baru (CR-018) | T-05, T-06, T-08, T-09, UP-73; CR-017; CR-018 | F1 (AC8 jalur Layanan Pelanggan: F2) | TC-US-103-1–3, -6, -7, -9, -10, -12–14, -16, -17, -19 (diubah); TC-US-103-20–51 (baru) |
| US-104 | AC3 (migrasi data) | T-11, UP-75 | F1 | TC-US-104-6 (diubah); TC-US-104-12, -13 (baru) |
| US-105 | AC1a (tabel validasi), AC1b (usia minimum PB-22), AC4a–AC4e | T-12, T-13, UP-77, UP-73 | F1 | TC-US-105-2, -10, -12–14 (diubah); TC-US-105-15–25 (baru) |
| US-106 | AC1, AC2, AC3, AC4 (batas NIK berlapis), AC5 | T-07, T-14–T-16, UP-73, UP-74 | F1 | TC-US-106-4, -6–10, -12, -16, -17 (diubah); TC-US-106-23–34 (baru) |
| US-110 | AC1–AC8 (story baru) | UP-78 a, CR Checkpoint 2 | F2 (web RME: F3) | TC-US-110-1–12 (baru) |
| US-201 | AC3 (daftar alasan awal), AC3a, AC7, AC8 baru (Verifikator tidak memutuskan pengajuan akunnya sendiri) | T-17, T-18, T-19 d, UP-74, UP-75, UP-76 (PT-56) | F1 | TC-US-201-4, -9, -14, -17 (diubah); TC-US-201-20–27 (baru, termasuk AC8) |
| US-207 | AC5 baru (reset MFA pasien oleh Layanan Pelanggan) | CR-018 | F2 | TC-US-207-4 (diubah, KU-10); TC-US-207-9–17 (baru; reset MFA hanya Layanan Pelanggan, Super Admin ditolak: TC-US-207-15) |
| US-208 | AC4–AC7, AC8 (pemberitahuan sub-peran sensitif) | T-19, UP-76 | F1 (AC8: F2) | TC-US-208-4, -12 (diubah); TC-US-208-15–27 (baru) |
| US-1501 | AC3 (peringatan Fase 1/Fase 2) | T-21 | F1, F2 | TC-US-1501-6, -9 (diubah); TC-US-1501-17, -18 (baru) |
| US-1504 | AC3 (deteksi manipulasi, pengganti) | T-22 | F1 | TC-US-1504-6–9 (diubah); TC-US-1504-10, -11 (baru) |
| NFR-SEC-02, -03, -08, -11 | MFA semua akun; nilai kebijakan password; MiB; sesi paralel tidak dibatasi | CR-018, T-02, T-07, UP-78 b | F1, F2 | TC-NFR-SEC-02-1, -2, TC-NFR-SEC-03-2, TC-NFR-SEC-08-1, TC-AUD-2 (diubah); TC-NFR-SEC-02-3–5, TC-NFR-SEC-03-3–7, TC-NFR-SEC-08-3, TC-NFR-SEC-11-1, -2 (baru) |

## Cakupan audit korporat (F1-T109, US-1501, Tahap 8)

Pemetaan setiap aksi korporat yang sudah ada ke event PRD §7.1 dan field §7.2 (KEPUTUSAN-380–385). Semua event di bawah: ID event, waktu server, aktor (ID akun, jenis akun, sub-peran; HR `corporate_admin` tanpa sub-peran sampai PT-68, atau `system`), kode + nama aksi, jenis + ID objek, sumber (komponen + IP; worker IdP hanya komponen), hasil + kode alasan bila gagal, ID korelasi; subjek data kosong (HR dan perusahaan bukan penerima layanan; pengecualian E-13 HR KEPUTUSAN-322 dan E-05 dokumen perusahaan KEPUTUSAN-304); `tenant_id` = perusahaan kecuali disebut lain. Nilai before/after hanya field di daftar izin kebijakan objek; NIB, NPWP (Terbatas, DD A20), nama, alamat, telepon, email, jabatan, nama berkas, ukuran, checksum, kunci objek, dan URL tidak pernah terdaftar. Pemeriksa test: `assert_corporate_events_clean` (`apps/rs-backend/tests/audit_support/mod.rs`).

| Aksi korporat (task) | Event §7.1 | Aksi audit (objek; field) | Status F1-T109 | Test |
| --- | --- | --- | --- | --- |
| Pendaftaran perusahaan mandiri (F1-T101) | E-10, E-10, E-04, E-11, E-13 | `corporate_client_self_registered` (corporate_client; status, origin, submission_round), `corporate_admin_account_registered` (account; account_type, status), `account_created` E-04 (account; account_type; `[USER]` Q28 = A), `corporate_membership_granted` (account; corporate_member_id, corporate_client_id), persetujuan HR per butir (corporate_hr_consent_record) | sudah ada; pemeriksa bersama ditambah | `corporate_registration_http.rs` |
| Pendaftaran dengan email HR terdaftar (F1-T101) | E-10 gagal | `corporate_registration_rejected` (corporate_registration acak; `hr_email_in_use`; aktor `system`, **tanpa tenant**) | sudah ada; pemeriksa bersama ditambah | `corporate_registration_http.rs` |
| NIB yang sudah dipakai (CR-035) | sama dengan pendaftaran baru | sama persis (KEPUTUSAN-365) | sudah ada | `corporate_registration_http.rs` |
| Unggah dokumen NIB/NPWP/bukti kewenangan (F1-T102) | E-10; gagal E-10 | `corporate_document_uploaded` (corporate_client_document; document_kind, replaced); isi ditolak: `file_too_large`/`file_type_rejected` | sudah ada; pemeriksa bersama ditambah | `corporate_documents_http.rs` |
| Ganti dokumen (F1-T102) | E-16 | `corporate_document_soft_deleted` (dokumen lama; document_kind sebelum = sesudah, deleted_at, delete_reason `replaced`) | **ditambah** (KEPUTUSAN-384) | `corporate_documents_http.rs::replacing_a_document_soft_deletes_the_old_one_and_concurrent_confirms_keep_one_active` |
| Unduh dokumen oleh HR (F1-T102) | E-12 | `stored_file_downloaded` (corporate_document_file) | sudah ada (E-12, bukan E-05: data milik perusahaan pemanggil) | `corporate_documents_http.rs` |
| Pengajuan ulang setelah ditolak (F1-T102b) | E-10; gagal E-10 | `corporate_verification_resubmitted` (corporate_client; status, submission_round); gagal `documents_incomplete`, `not_resubmittable`; batas laju per akun (429, tanpa event, KEPUTUSAN-389) | sukses sudah ada; **gagal ditambah** (KEPUTUSAN-383) | `corporate_resubmission.rs`, `corporate_audit_chain.rs` |
| Koreksi data perusahaan saat Ditolak + pengajuan ulang (CR-036) | E-10, E-10; gagal E-10 | `corporate_client_data_corrected` (corporate_client; field berubah dengan before/after: legal_name, office_address, office_phone teks tersaring; nib, npwp hanya `last4`, KEPUTUSAN-416) + `corporate_verification_resubmitted`; gagal kedua aksi `documents_incomplete`, `not_resubmittable`; 422 dan batas laju tanpa event | **ditambah** (KEPUTUSAN-410–412, KEPUTUSAN-416) | `corporate_correction.rs` |
| Verifikator membuka dokumen perusahaan (F1-T104) | E-05; gagal E-05 | `stored_file_opened` (corporate_document_file, tanpa subjek); ditolak `access_denied` | sudah ada; pemeriksa bersama ditambah | `corporate_verification.rs::document_url_is_issued_only_after_e05_is_recorded` |
| Verifikator membuka detail perusahaan (F1-T104, memuat NPWP) | E-05 | `corporate_detail_opened` (corporate_client_detail = perusahaan, tanpa subjek, tanpa field) | **ditambah** (CR-037, `[USER]` Q29 = A); antrean (tanpa NPWP) tetap tanpa event | `corporate_verification.rs::opening_company_detail_records_e05_without_restricted_data` |
| Keputusan Verifikator setuju/tolak (F1-T104/F1-T106) | E-10; gagal E-10 | `corporate_verification_approved`/`_rejected` (corporate_client; status, reason_code, submission_round); gagal `already_decided`, `nib_in_use_by_active_company`, `documents_incomplete`, `documents_changed` | sukses + kalah sudah ada; **gagal aturan dokumen ditambah** (KEPUTUSAN-382) | `corporate_verification.rs`, `corporate_documents_access.rs` |
| Perusahaan dibuatkan Verifikator (US-1903, F1-T105) | E-10, E-10, E-04, E-11; gagal E-10 | `corporate_client_created_by_admin` (origin `created_by_admin`), `corporate_admin_account_invited`, `account_created` E-04 (Q28 = A), `corporate_membership_granted`; gagal `corporate_client_create_rejected` (objek akun Verifikator; `nib_already_registered`/`hr_email_in_use`; **tanpa tenant**) | sudah ada; pemeriksa bersama ditambah | `corporate_admin_create.rs` |
| Undangan HR oleh HR (F1-T107) | E-10, E-04, E-11; gagal E-10 | `corporate_admin_account_invited`, `account_created` E-04 (Q28 = A), `corporate_membership_granted`; email terpakai `email_in_use` | sudah ada; pemeriksa bersama ditambah | `corporate_hr_invite.rs` |
| Penonaktifan HR lain (F1-T107) | E-10, E-11; gagal E-10 | `corporate_admin_account_deactivated` (status, status_reason nama saja), `corporate_membership_revoked`; ditolak aturan (`self_deactivation`, `last_active_hr`) | sudah ada; pemeriksa bersama ditambah; hapus lunak keanggotaan tidak digandakan E-16 (KEPUTUSAN-385) | `corporate_hr_members.rs` |
| Worker IdP: penyediaan, email aktivasi, nonaktif, akhiri sesi, kompensasi (ADR-030) | E-10; gagal E-10 | `idp_user_provisioned`, `idp_activation_email_sent`, dst. (account; status, idp_intent_id, idp_operation, nama field saja untuk idp_subject/activation_email); aktor `system`, komponen `worker` | sudah ada; **tenant ditambah** (KEPUTUSAN-381) | `identity_idp_worker.rs` |
| Login, login gagal/terkunci, logout HR; atur password dan TOTP saat aktivasi, kode pemulihan, reset password (realm `korporat`, BFF `rs-korporat-bff`) | E-01, E-02, E-03, E-04 | `login`, `login_failed`, `logout`, `password_changed`, `otp_configured`, `recovery_codes_created`, ... (account) dari pengambil event Keycloak | **ditambah**: realm `korporat` bawaan + tenant + komponen `corporate_portal` (KEPUTUSAN-380/381); subjek tidak dikenal tanpa objek dan tanpa tenant | `identity_keycloak_events.rs::hr_events_from_korporat_realm_are_recorded_with_the_company_as_tenant` |
| Aktivasi status akun HR `pending_activation` → `active` | (E-10 bila ada) | — | **tidak ada transisi di kode** (akun HR tetap `pending_activation`; akses memakai `status <> 'inactive'`); di luar scope F1-T109, dilaporkan | — |
| Reset MFA HR oleh Verifikator (US-1904 AC8), impor dan undangan peserta (E-18), persetujuan berbagi ringkasan | E-04, E-18, E-13 | — | belum ada aksinya (Fase 2/3) | — |

Perbaikan review T1 (KEPUTUSAN-386–389, TD-52): index riwayat keanggotaan `20261127100000` + satu query tenant; invarian satu keanggotaan per akun HR (`corporate_hr_invite.rs::a_deactivated_hr_cannot_gain_a_second_membership_through_an_invitation`); komponen sumber diikat ke realm (`identity/domain.rs::client_component_is_bound_to_the_realm_of_the_event`, `identity_keycloak_events.rs::other_realm_events_with_an_hr_subject_do_not_become_hr_events`); batas laju pengajuan ulang (`corporate_resubmission.rs::repeated_resubmissions_are_rate_limited_per_hr_account`); daftar realm eksplisit yang menghilangkan realm ber-secret ditolak (`identity/api.rs`).

Rantai audit: event korporat dipindahkan relay ke database audit dan diverifikasi hash ulang + sambungan `prev_hash` (`corporate_audit_chain.rs`, `relay_and_verify_tenant_chain`); mekanisme rantai tetap ADR-009 butir 5, tanpa mekanisme baru.
