# Backlog

Diisi mulai Tahap 3 (PRD). Format story dan change request: lihat `.claude/skills/runbook/references/templates.md`.

## Definition of Ready

- Acceptance criteria jelas dan bisa diuji
- Merujuk ke bagian PRD dan ADR yang relevan
- Dependensi sudah selesai atau jelas
- Ukurannya muat dalam satu sesi kerja (satu PR yang bisa direview dalam 15 menit)
- Tambahan Tahap 7: story sudah dipecah menjadi task 2–4 jam dengan urutan, dependensi, dan penanda kelas T1/T2 (`docs/07-phase-plan.md` §7.3 untuk Fase 1); gerbang eksternal yang dibutuhkan (`docs/07-phase-plan.md` §9) sudah terbuka, atau task yang bergantung padanya ditandai dan tidak dimulai
- Tambahan Tahap 7: bila sebagian AC bergantung pada story di fase berikutnya, AC itu ditulis eksplisit di kolom Fase dan diverifikasi di fase tujuan `[KEPUTUSAN-179]`

## Definition of Done

- CI hijau: lint, type check, test, security scan
- Mengikuti pola reference implementation
- Lolos review subagent `code-reviewer`
- Audit trail tercatat untuk aksi sensitif
- File migrasi disertakan kalau ada perubahan skema
- Acceptance criteria terverifikasi
- Dependensi mobile baru sudah direview perilaku jaringannya; build rilis lulus uji lalu lintas jaringan terhadap daftar izin host (ADR-021 butir 2)
- Migrasi yang menyentuh database audit, role, grant, trigger penolak, atau RLS sudah direview DBA dan keamanan (ADR-021 butir 4)
- Spesifikasi OpenAPI diperbarui untuk setiap endpoint baru atau berubah, dan contract test lulus (ADR-022)
- Setiap `response_code` baru didaftarkan di `docs/05-response-codes.md`, tiga digit pertamanya sama dengan kode HTTP, dan satu kode kasus tidak dipakai ulang untuk arti berbeda (ADR-023, KEPUTUSAN-84; usulan CR-012, berlaku setelah gate Tahap 5)
- Dependensi baru sudah dicek di registry resmi dan dicatat di `docs/sources.md`; pemindaian dependensi dan image (cargo-deny, cargo-audit, osv-scanner, Trivy) tidak menyisakan advisory Critical atau High yang belum ditangani (ADR-024, ADR-025, KEPUTUSAN-81; usulan CR-012, berlaku setelah gate Tahap 5)
- Perubahan di modul T1 baru dari Tahap 5 (BFF auth di server Next.js, konfigurasi OpenBao, bastion dan skrip deploy production) direview penuh termasuk security, dengan test negatif (ADR-024, ADR-025; usulan CR-012, berlaku setelah gate Tahap 5)
- Tambahan Tahap 7: setiap test yang membuktikan AC memakai penanda `trace: US-xxx ACn` dan kolom "File kode / test" di `docs/traceability.md` diperbarui (KEPUTUSAN-133); task ditandai selesai di backlog beserta commit hash (anti-halu §5)
- Tambahan Tahap 7: layar baru memenuhi aturan aksesibilitas design guideline §7 dan lulus test guideline `flutter_test` (mobile) atau test keyboard/fokus Playwright (web) untuk alur utama (KEPUTUSAN-176)

## Epic dan Story

Sumber: `docs/03-prd.md` v1.3 (CR-021: EP-18, EP-19; fase dari phase plan v0.3 `[USER] C21-P4`); sebelumnya v1.2 final (v1.0 Tahap 3, revisi CR-007 Tahap 4, revisi CR-013 dan klarifikasi AC Fase 1 di Checkpoint 2, CR-017, CR-018, US-110). Acceptance criteria, audit event, dan rujukan lengkap setiap story ada di PRD bagian 5 (`[KEPUTUSAN-03]`). Prioritas adalah usulan penyusun `[UP-01]` yang disetujui di gate Tahap 3, 2026-11-17 (T3-Gate). Kolom Fase diisi di Tahap 7 sebagai usulan `[UP-65]` `[UP-66]`, disetujui di Checkpoint 2 `[USER] C2-Q5, C2-Q6`, dengan Opsi B sebagai baseline sampai gerbang re-plan 2026-12-11 `[USER] C2-Q1`; rincian fase, tanggal, dan task Fase 1 ada di `docs/07-phase-plan.md`. Fase 4 bersifat bersyarat (dikerjakan hanya bila gerbang re-plan 2026-12-11 memilih Opsi A; di Opsi B, US-108 dan US-109 pindah ke Fase 3). Enabler (EN-xx) dan story UI (UI-xx) ada di bagian tersendiri di bawah `[KEPUTUSAN-179]`.

**Definition of Ready story Fase 1 (Checkpoint 2, 2026-11-18).** Klarifikasi AC Fase 1 disetujui `[USER] C2-B4, C2-B14, C2-B15` dan teksnya sudah diterapkan di PRD v1.2 final. Karena itu butir DoR "AC jelas dan bisa diuji" untuk US-101, US-103–US-106, US-201, US-208, US-1501, dan US-1504 berstatus **terpenuhi**. Butir DoR lain (task, dependensi, gerbang eksternal) tetap mengikuti `docs/07-phase-plan.md`. Bagian AC yang masih memuat `[TIDAK DIKETAHUI]` (mekanisme Keycloak, PB-22, daftar kata sandi bocor) diverifikasi di task terkait dan tidak menghalangi dimulainya task. Test case story terdampak diperbarui QA sebelum task dimulai (`docs/checkpoint-2.md` tindak lanjut 4).

Story yang terblokir pertanyaan terbuka: US-1605 dan US-1606 (PT-24, L-06). Story dengan ketergantungan legal yang dapat mengubah AC: US-104, US-108, US-203, US-308, US-309, US-805–US-808, US-1306, US-1309 (lihat PRD §12.3).

### [EP-01] Akun, identitas, dan anggota keluarga (aplikasi pasien)

| Story | Judul | Induk BR | Prioritas | Fase | Status |
| --- | --- | --- | --- | --- | --- |
| US-101 | Registrasi dengan email dan password | BR-001, BR-103 | Must | 1 (DoR AC terpenuhi, PRD v1.2 final; AC5 pendaftaran TOTP dan kode pemulihan pasien dari CR-018) | Backlog |
| US-102 | Dihapus oleh CR-007 (sebelumnya: masuk dengan akun Google atau Apple) | BR-001 | Won't | — | Dihapus |
| US-103 | Masuk, keluar, dan atur ulang password | BR-001 | Must | 1 (DoR AC terpenuhi, PRD v1.2 final; AC3 diputuskan lewat UP-73; AC6 dari CR-017; AC1, AC1b, AC4e, AC7, AC8 dari CR-018; jalur Layanan Pelanggan di AC8 diverifikasi di Fase 2 bersama US-207 AC5). **v0.3 (CR-021, S3):** AC6 (ubah password) dan AC7 (daftar ulang TOTP) realm `pasien`/`admin` diverifikasi di Fase 2 (F1-T75–F1-T79 pindah) `[USER] C21-P4` | Backlog |
| US-104 | Persetujuan pemrosesan data | BR-103, BR-104, BR-107 | Must | 1 (AC4 di Fase 2 bersama US-1001; DoR AC terpenuhi, PRD v1.2 final) | Backlog |
| US-105 | Profil data diri | BR-002 | Must | 1 (DoR AC terpenuhi, PRD v1.2 final; nilai PB-22 menunggu L-08). **v0.3:** AC4 (ubah email, F1-T57) di Fase 2 `[USER] C21-P4` | Backlog |
| US-106 | Unggah NIK dan foto KTP | BR-003 | Must | 1 (reference implementation; DoR AC terpenuhi, PRD v1.2 final) | Backlog |
| US-107 | Pembatasan layanan berdasarkan status verifikasi | BR-003, BR-004 | Must | 2 | Backlog |
| US-108 | Tambah anggota keluarga | BR-002, BR-008, BR-104 | Should | 4 (Opsi B: 3) | Backlog |
| US-109 | Ubah dan hapus anggota keluarga | BR-008, BR-106 | Should | 4 (Opsi B: 3) | Backlog |
| US-110 | Perangkat dan sesi aktif (lintas aplikasi; UP-78 a, ASVS 7.5.2) | BR-001, BR-108 | Must | 2 (aplikasi pasien, aplikasi tenaga kesehatan setelah EN-17, backoffice, portal setelah EN-20); bagian web RME diverifikasi di Fase 3 bersama EN-22 `[USER] C2-Q10` | Backlog |

### [EP-02] Verifikasi dan pengelolaan akun

| Story | Judul | Induk BR | Prioritas | Fase | Status |
| --- | --- | --- | --- | --- | --- |
| US-201 | Verifikasi identitas (KTP/KK) | BR-004, BR-008 | Must | 1 (reference implementation; pemberitahuan AC2 dan AC7 di Fase 2 bersama US-1001; DoR AC terpenuhi, PRD v1.2 final) | Backlog |
| US-202 | Pendaftaran tenaga kesehatan | BR-005, BR-102 | Must | 2 | Backlog |
| US-203 | Verifikasi tenaga kesehatan | BR-005, BR-102 | Must | 2 | Backlog |
| US-204 | Pemantauan masa berlaku SIP | BR-102 | Must | 2 | Backlog |
| US-205 | Onboarding rekanan | BR-006, BR-031 | Must | 2 (PT-47; status integrasi penuh dari US-1405 di Fase 3) | Backlog |
| US-206 | Akun admin mitra | BR-006 | Must | 2 | Backlog |
| US-207 | Menonaktifkan dan mengaktifkan akun | BR-060 | Must | 2 (kaskade ke produk dan home visit di AC2 diverifikasi di Fase 4; AC5 reset MFA pasien oleh Layanan Pelanggan dari CR-018) | Backlog |
| US-208 | Akun dan sub-peran admin Ruang Sehat | BR-060, BR-080 | Must | 1 (DoR AC terpenuhi, PRD v1.2 final; AC8 pemberitahuan sub-peran sensitif di Fase 2 bersama US-1003; bootstrap AC4 production menunggu PT-52). **v0.3:** layar kelola admin backoffice (F1-T67) di Fase 2; AC1–AC3 lewat API di Fase 1 `[USER] C21-P4` | Backlog |

### [EP-03] Konsultasi online

| Story | Judul | Induk BR | Prioritas | Fase | Status |
| --- | --- | --- | --- | --- | --- |
| US-301 | Tarif dan durasi konsultasi (menu portal klinik) | BR-018, BR-064 | Must | 2 | Backlog |
| US-302 | Jadwal praktik online | BR-017 | Must | 2 | Backlog |
| US-303 | Cari dan pilih tenaga kesehatan | BR-010, BR-011, BR-012, BR-014 | Must | 2 | Backlog |
| US-304 | Pesan sesi konsultasi | BR-010, BR-011, BR-012, BR-040 | Must | 2 | Backlog |
| US-305 | Sesi chat | BR-010, BR-016 | Must | 2 | Backlog |
| US-306 | Sesi video | BR-010, BR-016 | Must | 2 | Backlog |
| US-307 | Ketidakhadiran dan pembatalan oleh tenaga kesehatan | BR-045 | Must | 2 | Backlog |
| US-308 | Catatan konsultasi | BR-013, BR-016, BR-114 | Must | 2 (draf di server, UP-72) | Backlog |
| US-309 | Resep digital | BR-015, BR-114 | Must | 2 | Backlog |
| US-310 | Riwayat konsultasi pasien | BR-016, BR-106 | Must | 2 | Backlog |
| US-311 | Daftar sesi tenaga kesehatan | BR-013 | Must | 2 | Backlog |

### [EP-04] Pendaftaran klinik dan lab, check-in, dan antrean

| Story | Judul | Induk BR | Prioritas | Fase | Status |
| --- | --- | --- | --- | --- | --- |
| US-401 | Cari klinik, lab, dan layanannya | BR-020, BR-021 | Must | 3 | Backlog |
| US-402 | Pesan pendaftaran klinik atau lab | BR-020, BR-021, BR-024, BR-040 | Must | 3 | Backlog |
| US-403 | Appointment dan kode check-in | BR-024 | Must | 3 | Backlog |
| US-404 | Check-in lewat sistem rekanan | BR-024, BR-070 | Must | 3 | Backlog |
| US-405 | Antrean real-time | BR-027 | Should | 4 | Backlog |
| US-406 | Riwayat pendaftaran | BR-026 | Must | 3 | Backlog |

### [EP-05] Home visit

| Story | Judul | Induk BR | Prioritas | Fase | Status |
| --- | --- | --- | --- | --- | --- |
| US-501 | Cari dan pesan home visit | BR-022, BR-024, BR-040 | Must | 4 | Backlog |
| US-502 | Penugasan tenaga kesehatan | BR-022, BR-070 | Must | 4 | Backlog |
| US-503 | Status kunjungan oleh tenaga kesehatan | BR-022 | Must | 4 | Backlog |
| US-504 | Pantau kunjungan dan laporkan tenaga kesehatan tidak datang | BR-022, BR-045 | Must | 4 | Backlog |

### [EP-06] Informasi biaya MCU dan lab

| Story | Judul | Induk BR | Prioritas | Fase | Status |
| --- | --- | --- | --- | --- | --- |
| US-601 | Cari dan bandingkan biaya MCU dan pemeriksaan lab | BR-023 | Must | 3 | Backlog |
| US-602 | Lanjut mendaftar dari informasi biaya | BR-023, BR-020, BR-021 | Should | 3 | Backlog |

### [EP-07] Pembelian produk

| Story | Judul | Induk BR | Prioritas | Fase | Status |
| --- | --- | --- | --- | --- | --- |
| US-701 | Jelajah dan cari katalog | BR-030, BR-033 | Must | 4 (gerbang PT-37) | Backlog |
| US-702 | Detail produk | BR-030, BR-031, BR-032, BR-116 | Must | 4 | Backlog |
| US-703 | Keranjang dan checkout | BR-034, BR-040 | Must | 4 (gerbang PT-37) | Backlog |
| US-704 | Status dan pelacakan pesanan | BR-033, BR-034 | Must | 4 (gerbang PT-29, PT-37) | Backlog |
| US-705 | Konfirmasi terima dan klaim barang | BR-045 | Must | 4 (gerbang PT-29, PT-30, PT-33) | Backlog |
| US-706 | Riwayat pesanan | BR-035 | Must | 4 | Backlog |

### [EP-08] Pembayaran, harga, pembagian dana, dan refund (T1)

| Story | Judul | Induk BR | Prioritas | Fase | Status |
| --- | --- | --- | --- | --- | --- |
| US-801 | Ringkasan harga sebelum membayar | BR-042, BR-044, BR-024 | Must | 2 (setelah EN-24; pajak menunggu sisa PT-28) | Backlog |
| US-802 | Pembayaran lewat payment gateway | BR-040, BR-115 | Must | 2 (gerbang PT-36) | Backlog |
| US-803 | Pembagian dana per transaksi | BR-041, BR-044 | Must | 2 (gerbang PT-36) | Backlog |
| US-804 | Bukti pembayaran | BR-040 | Must | 2 | Backlog |
| US-805 | Pembatalan layanan terjadwal oleh pasien | BR-045 | Must | 2 | Backlog |
| US-806 | Refund otomatis | BR-045 | Must | 2 (kejadian produk RF-08/RF-09/RF-15 diverifikasi di Fase 4) | Backlog |
| US-807 | Klaim refund | BR-045 | Must | 3 (klaim produk dan home visit diverifikasi di Fase 4) | Backlog |
| US-808 | Eksekusi dan status refund | BR-045 | Must | 2 | Backlog |
| US-809 | Pengaturan margin per rekanan | BR-042, BR-063 | Must | 2 | Backlog |
| US-810 | Pengaturan biaya rekanan | BR-043, BR-063 | Must | 3 | Backlog |
| US-811 | Laporan transaksi dan rekonsiliasi pembagian dana | BR-046 | Must | 3 | Backlog |
| US-812 | Pengaturan parameter bisnis | BR-045, BR-063 | Must | 2 | Backlog |

### [EP-09] Hasil lab

| Story | Judul | Induk BR | Prioritas | Fase | Status |
| --- | --- | --- | --- | --- | --- |
| US-901 | Penerimaan hasil lab | BR-025, BR-070 | Must | 3 | Backlog |
| US-902 | Lihat dan unduh hasil lab | BR-025, BR-050 | Must | 3 | Backlog |

### [EP-10] Notifikasi

| Story | Judul | Induk BR | Prioritas | Fase | Status |
| --- | --- | --- | --- | --- | --- |
| US-1001 | Notifikasi pasien per kejadian dan kanal | BR-050 | Must | 2 | Backlog |
| US-1002 | Pengingat jadwal | BR-050 | Must | 2 | Backlog |
| US-1003 | Notifikasi tenaga kesehatan dan admin | BR-013, BR-102 | Must | 2 | Backlog |

### [EP-11] Konten dan layanan pelanggan

| Story | Judul | Induk BR | Prioritas | Fase | Status |
| --- | --- | --- | --- | --- | --- |
| US-1101 | Pengelolaan konten (CMS) | BR-061, BR-131 | Must | **1** (v0.3, dipindah dari Fase 3 untuk situs CR-021; F1-T91–F1-T95) `[USER] C21-P4` | Backlog |
| US-1102 | Konten di aplikasi pasien | BR-061 | Must | 3 | Backlog |
| US-1103 | Ajukan keluhan | BR-062 | Must | 3 | Backlog |
| US-1104 | Kelola keluhan | BR-062, BR-064, BR-065 | Must | 3 | Backlog |

### [EP-12] Portal rekanan

| Story | Judul | Induk BR | Prioritas | Fase | Status |
| --- | --- | --- | --- | --- | --- |
| US-1201 | Laporan transaksi dan pembagian dana rekanan | BR-046, BR-064, BR-065 | Must | 3 | Backlog |
| US-1202 | Lihat profil rekanan | BR-064, BR-065 | Must | 3 | Backlog |
| US-1203 | Lihat keluhan terkait rekanan | BR-064, BR-065 | Must | 3 | Backlog |

### [EP-13] Aplikasi RME minimalis (T1)

| Story | Judul | Induk BR | Prioritas | Fase | Status |
| --- | --- | --- | --- | --- | --- |
| US-1301 | Onboarding fasyankes ke RME minimalis | BR-120 | Must | 3 | Backlog |
| US-1302 | Akun dan peran pengguna RME | BR-120 | Must | 3 | Backlog |
| US-1303 | Jadwal, layanan, dan harga | BR-122 | Must | 3 | Backlog |
| US-1304 | Appointment dan check-in | BR-125, BR-024 | Must | 3 | Backlog |
| US-1305 | Antrean | BR-125, BR-027 | Should | 4 | Backlog |
| US-1306 | Catatan rekam medis kunjungan | BR-123 | Must | 3 (AC6 home visit diverifikasi di Fase 4) | Backlog |
| US-1307 | Hasil lab | BR-124, BR-025 | Must | 3 | Backlog |
| US-1308 | Home visit | BR-125, BR-022 | Must | 4 | Backlog |
| US-1309 | Pengiriman data rekam medis ke SATUSEHAT | BR-123 | Must | 3 (uji bergantung PT-32) | Backlog |
| US-1310 | Riwayat rekam medis pasien di fasyankes | BR-123, BR-126 | Must | 3 | Backlog |

### [EP-14] Integrasi rekanan dan pihak ketiga

| Story | Judul | Induk BR | Prioritas | Fase | Status |
| --- | --- | --- | --- | --- | --- |
| US-1401 | Spesifikasi kontrak integrasi dan lingkungan uji | BR-070, BR-121 | Must | 3 | Backlog |
| US-1402 | Sinkronisasi data rekanan | BR-070 | Must | 3 | Backlog |
| US-1403 | Pengiriman pesanan dan pembatalan ke rekanan | BR-070 | Must | 3 | Backlog |
| US-1404 | Penerimaan status dari rekanan | BR-070 | Must | 3 | Backlog |
| US-1405 | Pemantauan dan status integrasi per rekanan | BR-070, BR-090 | Must | 3 | Backlog |
| US-1406 | Integrasi pihak ketiga | BR-071 | Must | 2 (pembayaran, video, email); 4 (kurir) | Backlog |

### [EP-15] Audit trail (fitur)

| Story | Judul | Induk BR | Prioritas | Fase | Status |
| --- | --- | --- | --- | --- | --- |
| US-1501 | Pencatatan event audit | BR-080, BR-081, BR-107 | Must | 1 (event fase berikutnya ditambahkan per story; peringatan email AC3 di Fase 2 setelah EN-16; DoR AC terpenuhi, PRD v1.2 final) | Sebagian: F1-T69 (pustaka + inventaris pemancar); F1-T109 event korporat selesai, commit eb40a2d (tabel cakupan aksi korporat di `docs/traceability.md` bagian "Cakupan audit korporat"; realm `korporat` bawaan pengambil event E-01–E-04 + tenant HR, E-16 dokumen diganti, E-10 gagal untuk keputusan/pengajuan ulang yang ditolak aturan, pemeriksa bersama `assert_corporate_events_clean`, test rantai `corporate_audit_chain.rs`; KEPUTUSAN-380–385; menunggu review T1 security + DBA) |
| US-1502 | Halaman Audit Log | BR-080 | Must | 3 | Backlog |
| US-1503 | Export audit log | BR-080 | Must | 3 | Backlog |
| US-1504 | Perlindungan audit log | BR-080 | Must | 1 (DoR AC terpenuhi, PRD v1.2 final; peringatan email AC3 di Fase 2) | Backlog |

### [EP-16] Persetujuan dan hak subjek data (T1)

| Story | Judul | Induk BR | Prioritas | Fase | Status |
| --- | --- | --- | --- | --- | --- |
| US-1601 | Lihat dan tarik persetujuan | BR-103, BR-106 | Must | 3 | Backlog |
| US-1602 | Ajukan permintaan hak subjek data | BR-106 | Must | 3 | Backlog |
| US-1603 | Proses permintaan hak subjek data | BR-106, BR-111 | Must | 3 | Backlog |
| US-1604 | Salinan data pribadi | BR-106 | Must | 3 | Backlog |
| US-1605 | Penghapusan dan anonimisasi | BR-106, BR-112 | Must | 3 (terblokir PT-24, L-06) | Terblokir |
| US-1606 | Retensi dan pemusnahan otomatis | BR-112 | Must | 3 (terblokir PT-24) | Terblokir |

### [EP-17] Pengukuran KPI

| Story | Judul | Induk BR | Prioritas | Fase | Status |
| --- | --- | --- | --- | --- | --- |
| US-1701 | Dasbor KPI | BR-090 | Must | 3 | Backlog |

### [EP-18] Situs web Ruang Sehat (CR-021)

Sumber: PRD v1.3 EP-18; fase dari phase plan v0.3 (skenario S3 `[USER] C21-P4`).

| Story | Judul | Induk BR | Prioritas | Fase | Status |
| --- | --- | --- | --- | --- | --- |
| US-1801 | Halaman company profile publik | BR-130, BR-131 | Must | 1 (F1-T83, F1-T89, F1-T94, F1-T96, F1-T97, F1-T111) | Backlog |
| US-1802 | Konten situs di CMS | BR-131, BR-061 | Must | 1 (F1-T91–F1-T95, F1-T111) | Backlog |

### [EP-19] Layanan korporat (CR-021, T1)

Sumber: PRD v1.3 EP-19; fase dari phase plan v0.3 (skenario S3 `[USER] C21-P4`). Gerbang K2/K3: G-25, G-26 (`docs/07-phase-plan.md` §9).

| Story | Judul | Induk BR | Prioritas | Fase | Status |
| --- | --- | --- | --- | --- | --- |
| US-1901 | Pendaftaran perusahaan mandiri | BR-136 | Must | 1 (F1-T101–F1-T103; bentuk bukti kewenangan: surat kuasa/penugasan direksi, PDF atau foto bertanda tangan, Q9=A) | Sebagian: backend F1-T101 selesai, commit c0a43e2 (AC1 tanpa lampiran dokumen, AC3–AC7: `POST /v1/public/corporate-registrations` tanpa login, perusahaan Menunggu verifikasi + akun HR `pending_provisioning` tanpa password + bukti persetujuan HR per versi teks + E-10/E-11/E-13, NIB 13 angka dan NPWP 16 angka, jawaban `202` seragam untuk NIB/email terdaftar, batas laju, teks draf ditolak di production; KEPUTUSAN-321–327; menunggu review T1 + DBA); AC2 dokumen backend F1-T102 selesai, commit 4ddef79 (KEPUTUSAN-343–349; HR mengunggah NIB, NPWP, bukti kewenangan setelah aktivasi lewat `/v1/corporate/documents*` dan upload slot, validasi isi + penetralan PDF qpdf [qpdf asli belum dijalankan di mesin ini, TD-48], antrean Verifikator hanya memuat pengajuan lengkap, persetujuan tanpa dokumen lengkap 409 `4090113`; kode `4090112`, `4090113`; migrasi `20261126350000`; menunggu review T1 + DBA); pengajuan ulang setelah ditolak dan profil perusahaan backend F1-T102b selesai, commit 067dcc3, review T1 LULUS DENGAN CATATAN, perbaikan menunggu konfirmasi (AC6 profil di `GET /v1/corporate/documents`, US-1902 AC3 `POST /v1/corporate/verification-resubmissions`, `4090115`, KEPUTUSAN-360/361; menunggu review T1); CR-035 (Q10 = B2) backend selesai, commit 943da6c: NIB terdaftar diproses sama dengan NIB baru (`202` dan email aktivasi identik, TD-45 selesai), migrasi `20261126400000`/`20261126410000`, KEPUTUSAN-362–365, review T1 LULUS DENGAN CATATAN, perbaikan menunggu konfirmasi (KEPUTUSAN-366/367, TD-51); UI HR F1-T103 sebagian selesai, commit cdd2941 (halaman `/verifikasi` web korporat: status Menunggu verifikasi/Ditolak dengan alasan/Aktif, tiga dokumen, unggah/ganti lewat slot langsung ke object storage, unduh, Ajukan ulang; CSP `KORPORAT_UPLOAD_ORIGIN`; KEPUTUSAN-370/371; menunggu review T1); perbaikan review T1 commit 37bcd41 (TD-53); formulir daftar publik `/daftar` dan tombol "Daftarkan perusahaan" di situs (Q30 = C, KEPUTUSAN-372/373, ADR-031 diusulkan; snippet Caddy `infra/caddy/snippets/korporat-pendaftaran-publik.caddy`; review T1 c447a4b TIDAK LULUS, perbaikan B1–B5 commit berikutnya menunggu konfirmasi; **PRASYARAT KERAS sebelum `/daftar` dibuka di production: task backend "IP klien asli dari proxy tepercaya" (PT-39/TD-22: `X-Forwarded-For` hanya dibaca bila peer TCP ada di daftar proxy tepercaya, ambil alamat paling kanan), sebab tanpa itu batas laju global memungkinkan siapa pun memblokir semua pendaftar; usulan sementara: anggaran per sumber = global untuk sumber proxy, ADR-031; kode backend selesai 2026-11-27, KEPUTUSAN-400/401, commit de2dedb: `TRUSTED_PROXY_CIDRS`; tinggal konfigurasi deploy = alamat Caddy dan IP asli sampai Caddy, PT-39**); koreksi data perusahaan saat Ditolak (Q20 = A, CR-036) selesai commit `a66a26f` + perbaikan review `23bd7b3`, `ab2bc5a` (backend + web korporat, KEPUTUSAN-410–419); sisa: email pemberitahuan "sudah punya akun" (modul Notifikasi, Q5=A), `docs/05-response-codes.md` `4090107`–`4090109` belum dipakai |
| US-1902 | Verifikasi perusahaan oleh Verifikator | BR-136 | Must | 1 (F1-T104, F1-T106) | Sebagian: backend F1-T104 dan antrean/panel backoffice F1-T106 selesai (AC1–AC4 dan layar akses ditolak AC5 di backoffice; AC5 ditegakkan backend); AC3 "mengajukan ulang" backend F1-T102b selesai, commit 067dcc3, review T1 LULUS DENGAN CATATAN, perbaikan menunggu konfirmasi (KEPUTUSAN-360; keputusan putaran baru tidak terhalang putaran lama; menunggu review T1); CR-035 (commit 943da6c): penanda "NIB sudah dipakai perusahaan lain" di antrean/detail (backend + `web/backoffice`) dan persetujuan ditolak `4090116` bila NIB dipakai perusahaan Aktif lain (KEPUTUSAN-363/364); CR-036 (commit `a66a26f`): AC3 "memperbaiki" mencakup koreksi data perusahaan oleh HR bersama pengajuan ulang, Verifikator melihat data hasil koreksi di antrean/detail (KEPUTUSAN-410, KEPUTUSAN-414) |
| US-1903 | Perusahaan dibuatkan Verifikator | BR-136 | Must | 1 (F1-T105, F1-T106; persetujuan orang kedua G-27) | Sebagian: backend F1-T105 selesai (AC1–AC4: perusahaan Aktif + undangan HR pertama lewat jalur ADR-030, NIB/email ganda, E-10 sumber dibuat admin; tanpa persetujuan orang kedua sesuai G-27; AC2 diuji di keycloak-check `cek_undangan_hr`); UI F1-T106 "buat perusahaan" selesai di web/backoffice (AC1 dan AC4 di layar: 409 NIB/email HR, 422, 403 non-Verifikator, CSRF); sisa: lampiran dokumen perusahaan (menunggu F1-T102, ditandai jujur di layar) |
| US-1904 | Akun HR: masuk, MFA, pemulihan MFA, kelola anggota HR | BR-132, BR-145 | Must | 1 (F1-T85–F1-T88, F1-T90, F1-T100, F1-T107, F1-T108; F1-T108 di-merge 4be14d5, review T2 LULUS DENGAN CATATAN (tanpa alur undang HR, tombol nonaktif bertanda belum tersedia): AC2 sebagian; sisa undangan HR lain selesai di backend (F1-T107 sisa: `POST /v1/corporate/member-invitations`, KEPUTUSAN-329–331), UI undang HR di web korporat selesai di sisa F1-T108: formulir "Undang HR", pesan seragam KEPUTUSAN-306 untuk email baru dan terpakai); AC7–AC9 (daftar ulang TOTP dan reset MFA HR) di Fase 2 | Backlog |
| US-1905 | Katalog paket korporat | BR-133, BR-137, BR-138 | Must | 3 (setelah EN-21) | Backlog |
| US-1906 | Pesan layanan korporat | BR-135, BR-137, BR-142 | Must | 3 | Backlog |
| US-1907 | Unggah daftar peserta | BR-139, BR-145 | Must | 3 (G-26 untuk data nyata) | Backlog |
| US-1908 | Undangan peserta dan penautan akun pasien | BR-140, BR-145 | Must | 3 (layar di repositori mobile; AC4a menunggu PT-78) | Backlog |
| US-1909 | Invoice B2B | BR-141, BR-142 | Must | 3 (G-25: PT-59, L-21) | Backlog |
| US-1910 | Pencatatan mutasi bank dan pelunasan | BR-141 | Must | 3 (G-25) | Backlog |
| US-1911 | Penjadwalan setelah lunas | BR-142, BR-135 | Must | 3 | Backlog |
| US-1912 | Rekap kehadiran untuk HR | BR-144 | Must | 3 (sumber kehadiran PT-64) | Backlog |
| US-1913 | Persetujuan berbagi ringkasan oleh peserta | BR-144, BR-145 | Must | 3 (teks L-18) | Backlog |
| US-1914 | Ringkasan hasil untuk HR dengan persetujuan | BR-144 | Must | 3 (PT-65) | Backlog |
| US-1915 | Hasil lengkap untuk peserta | BR-143 | Must | 3 (setelah US-901) | Backlog |
| US-1916 | Pengelolaan perusahaan di backoffice | BR-136, BR-060 | Must | 1 (bagian K1, F1-T106); pesanan dan invoice di Fase 3 | Sebagian: antrean verifikasi di F1-T106; daftar perusahaan/anggota HR menunggu endpoint backoffice |
| US-1917 | Penerusan pesanan korporat ke rekanan pelaksana | BR-134, BR-070 | Must | 3 (setelah EN-21; PT-74) | Backlog |

## Enabler (Tahap 4–5)

Story teknis yang menjadi prasyarat story bisnis. Setiap enabler merujuk ADR atau KEPUTUSAN, ditambah BR yang dilayaninya (anti-halu §4). Prioritas enabler adalah usulan penyusun `[UP-65]`; semuanya Must karena menjadi prasyarat story Must atau NFR. Rincian task Fase 1: `docs/07-phase-plan.md` §7.3; perkiraan ukuran enabler Fase 2–4: §6.2.

| ID | Judul | Induk (ADR/KEPUTUSAN; BR) | Prioritas | Fase | Status |
| --- | --- | --- | --- | --- | --- |
| EN-01 | Repo dan kerangka monorepo | ADR-024, KEPUTUSAN-89 | Must | 1 | Backlog |
| EN-02 | `scripts/verify.sh`, git hook, CI GitHub Actions | ADR-021, ADR-025, KEPUTUSAN-81, KEPUTUSAN-82; BR-108 | Must | 1 | Backlog |
| EN-03 | Lingkungan lokal (Docker Compose) | ADR-019, KEPUTUSAN-88 | Must | 1 | Backlog |
| EN-04 | Skema inti dan migrasi (bootstrap role, fungsi `util`, test katalog hak role, tabel Fase 1, CR-014 bagian DB aplikasi) | ADR-001–ADR-004, ADR-008, KEPUTUSAN-74, KEPUTUSAN-185; BR-080 | Must | 1 | Backlog |
| EN-05 | Audit Trail Foundation (DB audit, crate `audit`, outbox, relay, rantai hash, verifikasi) | ADR-005, ADR-009, ADR-010; BR-080, BR-081, BR-107 | Must | 1 | Backlog |
| EN-06 | Autentikasi dan sesi: realm `pasien` dan `admin`, validasi token, penolakan sesi, pengambil event Keycloak; sejak CR-018 termasuk alur MFA wajib, kode pemulihan, dan Reset Credentials ber-OTP di realm `pasien` | ADR-011 (revisi Checkpoint 2), KEPUTUSAN-199, KEPUTUSAN-202, KEPUTUSAN-203; BR-001, BR-060, BR-108 | Must | 1 | Backlog |
| EN-07 | Otorisasi dan RBAC: crate `authz`, kebijakan wajib per endpoint, relasi | ADR-012, KEPUTUSAN-127, KEPUTUSAN-128; BR-060, BR-105, BR-108 | Must | 1 | Backlog |
| EN-08 | Crate dasar, envelope, logging dan security log, OpenAPI, test arsitektur | ADR-010, ADR-022, ADR-023, KEPUTUSAN-110, KEPUTUSAN-113, KEPUTUSAN-121–KEPUTUSAN-124, KEPUTUSAN-132; BR-080 | Must | 1 | Backlog |
| EN-09 | Kriptografi dan kunci data (crate `crypto`, Transit OpenBao, blind index NIK) | ADR-013, KEPUTUSAN-129, KEPUTUSAN-130; BR-105, BR-108 | Must | 1 | Backlog |
| EN-10 | Berkas unggahan (upload slot, URL bertanda tangan, validasi isi) | ADR-018, KEPUTUSAN-184; BR-003 | Must | 1 | Backlog |
| EN-11 | BFF Next.js (`bff-auth`) dan kerangka backoffice | ADR-011 butir 7, ADR-024, KEPUTUSAN-75, KEPUTUSAN-115; BR-060 | Must | 1 | Backlog |
| EN-12 | Kerangka mobile, `rs_core`, `rs_api_client`, `rs_auth`, data di perangkat | ADR-021 butir 1, ADR-024, KEPUTUSAN-76, KEPUTUSAN-116; BR-007 | Must | 1 | Backlog |
| EN-13 | Jalur email Keycloak dan notifikasi (spike, usulan ADR, adaptor SES API) | ADR-016, CR-012, KEPUTUSAN-183; BR-001, BR-050 | Must | 1 | Backlog |
| EN-14 | Reference implementation (US-106 → US-201) dan pengukuran kecepatan | KEPUTUSAN-181, KEPUTUSAN-134; RT-24; BR-003, BR-004 | Must | 1 | Backlog |
| EN-15 | Staging dan CD (runner JIT, OIDC GitHub → OpenBao staging, cosign, deploy bergulir) | ADR-021, ADR-025, KEPUTUSAN-81; BR-113 | Must | 2 (gerbang PT-39) | Backlog |
| EN-16 | Observability (Prometheus, Alertmanager, Loki, Grafana, GlitchTip, peringatan) | ADR-010, ADR-025, KEPUTUSAN-80; BR-017 (NFR-OBS-01) | Must | 2 | Backlog |
| EN-17 | Realm `nakes`, MFA, kunci aplikasi 15 menit (CR-009), kerangka aplikasi tenaga kesehatan | ADR-011, ADR-021 butir 1; BR-007, BR-013 | Must | 2 | Backlog |
| EN-18 | Video LiveKit (token per peserta, Egress tidak terpasang) | ADR-016, KEPUTUSAN-87; BR-010, BR-016 | Must | 2 | Backlog |
| EN-19 | Proses `realtime` WebSocket, Pub/Sub Valkey, AsyncAPI | ADR-016, ADR-022; BR-010 | Must | 2 | Backlog |
| EN-20 | Kerangka portal rekanan (BFF), klien admin rekanan, `PartnerScope` | ADR-011, ADR-012, KEPUTUSAN-128; BR-064, BR-065 | Must | 2 | Backlog |
| EN-21 | Gateway Partner Integration API (realm `integrasi`, HMAC webhook, idempotensi, outbox, pengaman SSRF, sandbox) | ADR-014, KEPUTUSAN-126; BR-070, BR-121 | Must | 3 (gerbang PT-05) | Backlog |
| EN-22 | Fondasi RME (`rme-backend`, realm `rme`, RLS, migrasi C1–C6 termasuk CR-014 bagian DB RME, audit RME, web RME BFF) | ADR-006, ADR-012, ADR-020, KEPUTUSAN-185; BR-120 | Must | 3 | Backlog |
| EN-23 | Pekerja privasi dan retensi (crypto-shredding; kebijakan kosong tidak memproses apa pun) | ADR-013; BR-112 | Must | 3 | Backlog |
| EN-24 | Pajak CR-013: data dictionary A6/A8/A13 dan migrasi, review DBA, sebelum US-801 | CR-013, ADR-015, KEPUTUSAN-185; BR-042 | Must | 2 (awal) | Backlog |
| EN-25 | Uji beban NFR-CAP-04 di staging berukuran production | ADR-019, KEPUTUSAN-78; BR-017 | Must | 3 (Januari 2027) | Backlog |
| EN-26 | Production: host, Compose, Caddy, OpenBao Raft + unseal, pgBackRest + standby, bastion + skrip deploy | ADR-019, ADR-021, ADR-025, KEPUTUSAN-78, KEPUTUSAN-79, KEPUTUSAN-81; BR-113 | Must | Februari 2027 (gerbang PT-38) | Backlog |
| EN-27 | Uji pemulihan NFR-DR-03 | ADR-019, KEPUTUSAN-14; BR-113 | Must | Februari 2027 | Backlog |

## Story UI (Tahap 6)

Turunan `docs/06-design-guideline.md` dan wireframe. Layar per alur dikerjakan di dalam story PRD masing-masing (peta alur di guideline §10.1); story di bawah adalah pekerjaan UI lintas story. Catatan non-blokir QA Tahap 6 N3–N5 dipetakan di `docs/07-phase-plan.md` §11.7.

| ID | Judul | Induk (KEPUTUSAN; BR) | Prioritas | Fase | Status |
| --- | --- | --- | --- | --- | --- |
| UI-01 | Token desain dan komponen dasar web (`web/packages/ui`), skrip kontras, galeri state (QA N4) | KEPUTUSAN-161, KEPUTUSAN-163, KEPUTUSAN-169, KEPUTUSAN-176; BR-060, BR-064 | Must | 1 | Backlog |
| UI-02 | Token desain dan komponen dasar Flutter (`rs_design`), test guideline aksesibilitas | KEPUTUSAN-162, KEPUTUSAN-163, KEPUTUSAN-176; BR-007 | Must | 1 | Backlog |
| UI-03 | Tema login Keycloak Ruang Sehat (pesan seragam login gagal, UP-73; halaman pendaftaran TOTP dan kode pemulihan untuk semua realm termasuk `pasien`, CR-018) | ADR-011 butir 4, 5, 7, KEPUTUSAN-175, KEPUTUSAN-202; BR-001 | Must | 1 | Backlog |
| UI-04 | Template email notifikasi (D-F02) | KEPUTUSAN-175; BR-050 | Must | 2 | Backlog |
| UI-05 | Uji manual pembaca layar TalkBack/VoiceOver alur A01, A05, A07 sebelum UAT | KEPUTUSAN-160, KEPUTUSAN-176; BR-007 | Should | 3 | Backlog |

Aset merek final (logo, ikon aplikasi, aset toko) bukan pekerjaan pengembangan; bergantung pada PT-40 dan PT-45 dan dibutuhkan sebelum pengajuan toko aplikasi (Tahap 12).

## Bug

_Belum ada._

## Change Request

### CR-001: Semua tenaga kesehatan berpraktik atas nama klinik rekanan

Diajukan: 2026-11-17 (wawancara Tahap 2, `docs/02-catatan-brd.md` Q2)
Deskripsi: Dokter, psikolog, fisioterapis, dan tenaga medis yang melayani di Ruang Sehat berpraktik atas nama klinik rekanan (fasyankes). PT. Arunika berperan sebagai platform (PSE). Tenaga kesehatan "mitra langsung" Ruang Sehat tanpa fasyankes ditiadakan. Mengubah brief PS6e.
Alasan: riset kepatuhan Tahap 2 (`docs/compliance/tahap-02-riset.md` temuan 1.1, SRC-02, SRC-03) mengindikasikan telemedisin diselenggarakan fasyankes, sendiri atau bekerja sama dengan PSE terdaftar.
Dampak ke scope / waktu / arsitektur: konsultasi online kini bergantung pada klinik rekanan (R-01 meluas). Model data menghubungkan setiap nakes ke klinik rekanan. Dikonfirmasi legal sebelum Tahap 4 (L-01).
Keputusan: Diterima user (T2-Q2), berlaku sejak BRD v1.0

### CR-002: Apotek rekanan menjadi outlet rekanan untuk obat bebas

Diajukan: 2026-11-17 (`docs/02-catatan-brd.md` Q3, Q3a, Q3b)
Deskripsi: Apotek rekanan ditambahkan sebagai outlet rekanan. Obat bebas dijual oleh apotek dan klinik rekanan yang sudah berizin; barang milik rekanan. Mengubah brief PS2 (apotek bukan outlet) dan memperluas PS2a. Peran apoteker (PS3) tetap ditunda.
Alasan: keputusan user setelah riset PSEF (SRC-08) dan BPOM (SRC-10, SRC-11).
Dampak ke scope / waktu / arsitektur: portal admin outlet juga dipakai apotek. Kewajiban PSEF bagi platform menunggu L-03; tindak lanjut bila wajib di PT-21.
Keputusan: Diterima user (T2-Q3, T2-Q3b), berlaku sejak BRD v1.0

### CR-003: Pembagian dana langsung oleh payment gateway

Diajukan: 2026-11-17 (`docs/02-catatan-brd.md` Q4)
Deskripsi: Pembayaran konsultasi dan produk dibagi langsung oleh payment gateway berizin BI ke rekening rekanan dan rekening PT. Arunika. PT. Arunika tidak menampung dan tidak meneruskan dana rekanan sendiri. Mengubah brief PS4; "penerusan dana" di P8.2a dibaca sebagai pemantauan, laporan, dan rekonsiliasi.
Alasan: riset (SRC-19, SRC-20) menemukan larangan aktivitas sistem pembayaran tanpa izin BI dan pengaturan penerusan pembayaran dari PJP ke penyedia barang/jasa.
Dampak ke scope / waktu / arsitektur: pemilihan payment gateway di Tahap 4–5 harus mendukung pembagian dana. Konfirmasi legal L-04 sebelum Tahap 4.
Keputusan: Diterima user (T2-Q4), berlaku sejak BRD v1.0

### CR-004: Pendapatan transaksi berupa margin, menggantikan komisi potongan

Diajukan: 2026-11-17 (`docs/02-catatan-brd.md` Q6, Q6a)
Deskripsi: Harga konsultasi dan produk diambil dari rekanan, lalu Ruang Sehat menambahkan margin berbentuk persen atau nominal yang dapat diatur per rekanan oleh admin Ruang Sehat. Model komisi potongan di Business Case §5.1 diganti. Biaya dari rekanan tetap.
Alasan: keputusan user.
Dampak ke scope / waktu / arsitektur: Business Case §5.1–5.2 diperbarui istilahnya (komisi → margin) setelah gate Tahap 2. Pengaturan margin per rekanan masuk backoffice (BR-063). Pencatatannya di audit trail (BR-080) adalah usulan penyusun berdasarkan ADR-005, disetujui user di gate Tahap 2.
Keputusan: Diterima user (T2-Q6, T2-Q6a), berlaku sejak BRD v1.0

### CR-005: Semua layanan yang dipesan di aplikasi dibayar di aplikasi

Diajukan: 2026-11-17 (setelah gate Tahap 2, `docs/02-catatan-brd.md` Q13–Q20)
Deskripsi: Setiap layanan yang dimulai atau didaftarkan di aplikasi dibayar di aplikasi, termasuk pendaftaran klinik, pendaftaran lab, dan home visit. Pasien yang datang ke klinik atau lab cukup menunjukkan appointment di aplikasi mobile. Membalik BR-024 dan butir non-scope "Pembayaran klinik, lab, dan home visit di aplikasi" di BRD v1.0 §4.2; memperluas BR-040, BR-041, P-04, P-05.
Alasan: keputusan user.
Dampak ke scope / waktu / arsitektur: pembagian dana langsung (CR-003) dan aturan refund (BR-045, PT-27) berlaku juga untuk klinik, lab, dan home visit. Cakupan konfirmasi legal L-04, L-08, L-10, L-12, L-13, dan L-14 meluas; PT-28 (pajak) ditambahkan. Sistem rekanan perlu menerima status lunas dan mencocokkan appointment saat pasien datang (PT-05). Business Case §5 perlu memuat margin klinik, lab, dan home visit saat diperbarui.
Keputusan: Diterima user (T2-Q13 sampai T2-Q20): tindakan tambahan yang tidak tercantum di aplikasi dibayar di luar aplikasi; margin dan aturan biaya transaksi T2-Q12 berlaku untuk semua layanan; biaya rekanan dipakai atau tidak diputuskan CEO (PT-17); pembatalan layanan terjadwal sebelum batas waktu di-refund; informasi biaya MCU/lab menampilkan harga bayar di aplikasi. Berlaku sejak BRD v1.1 (disetujui user di gate revisi 2026-11-17, T2-Gate-CR005)

### CR-006: RME minimalis sebagai aplikasi terpisah dan antrean real-time klinik

Diajukan: 2026-11-17 (wawancara Tahap 3, `docs/03-catatan-prd.md` T3-Q1, T3-Q1a, T3-Q1b, T3-Q1c, T3-Q14, T3-Q15, T3-Q16)
Deskripsi: Dari empat butir yang belum diputuskan (BRD §4.3, PT-13), RME **minimalis** dan antrean real-time klinik masuk rilis 1; RME lengkap, loyalitas/poin/voucher, dan multi-bahasa tidak masuk. RME minimalis adalah aplikasi web terpisah dari aplikasi dan portal Ruang Sehat, dibangun di proyek ini untuk rilis 1, menjadi "sistem rekanan" bagi klinik dan lab yang belum punya sistem, dan terhubung ke Ruang Sehat lewat integrasi yang sama dengan sistem rekanan lain. Isinya: jadwal, layanan, dan harga; appointment dan check-in; antrean; rekam medis kunjungan yang datanya sesuai ketentuan SATUSEHAT; hasil lab; penugasan home visit. Hanya mencatat pasien yang datang lewat Ruang Sehat. Portal Ruang Sehat untuk rekanan ditegaskan hanya untuk pelaporan dan rekap, ditambah satu menu pengaturan konsultasi online (T3-Q4 tetap, T3-Q16).
Alasan: keputusan user, agar rekanan tanpa sistem dapat bergabung ("stimulus").
Dampak ke scope / waktu / arsitektur: BRD naik ke v1.2 (S-17, S-18, BR-027, BR-120–BR-126, perubahan BR-018, BR-064, BR-070, perluasan L-05). Komponen baru yang besar (aplikasi web RME minimalis dan integrasi SATUSEHAT) di dalam 104 hari dengan 1 mobile developer: risiko jadwal R-02 dan kapasitas R-04 naik; opsi penyesuaian ada di PRD §9 untuk diputuskan user. PT. Arunika berposisi sebagai penyedia sistem RME: registrasi SATUSEHAT memerlukan nomor PSE, nomor afiliasi SNOMED CT, dan survei keamanan `[SRC-27]` (PT-32). Klinik/lab pengguna RME minimalis terhitung "terintegrasi" untuk syarat go-live 1 dan KPI-03/04. Business Case §6 dan R-02/R-04 ditinjau ulang di gate Tahap 4 dan Tahap 7.
Keputusan: Diterima user (T3-Q1a, T3-Q1b, T3-Q1c, T3-Q15, T3-Q16). Berlaku sejak BRD v1.2 dan PRD disetujui di gate Tahap 3, 2026-11-17 (T3-Gate)

### CR-007: Semua penyedia pihak ketiga memproses data di Indonesia; login Google/Apple, push, dan WhatsApp dihapus

Diajukan: 2026-11-18 (Tahap 4, `docs/04-catatan-arsitektur.md` T4-Q1, T4-Q2)
Deskripsi: Keputusan "data wajib di Indonesia" (brief P5.5, BR-113) berlaku untuk semua penyedia pihak ketiga tanpa kecuali (PT-22). Verifikasi sumber resmi menunjukkan lokasi pemrosesan login Google/Apple, push notification (APNs untuk iOS, FCM untuk Android), dan WhatsApp Cloud API tidak dapat dipastikan di Indonesia (`docs/sources.md`). Karena itu: login Google/Apple dihapus (BR-001, US-102); notifikasi push dan WhatsApp dihapus (BR-050); notifikasi pasien dan tenaga kesehatan lewat email dan notifikasi di dalam aplikasi (kotak notifikasi, tampil langsung saat aplikasi terbuka); pengingat jadwal lewat email; panggilan video hanya dapat diterima saat kedua pihak berada di ruang sesi.
Alasan: keputusan user T4-Q1 ("Semua di Indonesia") dan persetujuan CR T4-Q2 ("Setujui CR-007 apa adanya").
Dampak ke scope / waktu / arsitektur: BRD naik ke v1.3 (P-01, S-14, BR-001, BR-050, BR-071, BR-113, §7.3, PT-22); PRD naik ke v1.1 (§2.2, AB-73, US-101, US-102, US-306, US-311, US-405, matriks EP-10, US-1001, US-1003, US-1406, US-1602, NFR-PRIV-02, E-04). Scope berkurang (login sosial, push, WhatsApp), bertambah sedikit (kotak notifikasi di dalam aplikasi). Risiko produk: pasien atau tenaga kesehatan melewatkan jadwal karena tidak ada notifikasi di layar ponsel (dicatat di `docs/04-risk-register.md`). Arsitektur: penyedia video, email, pemantauan, dan pelaporan crash wajib di Indonesia; SDK pihak ketiga yang mengirim data ke luar Indonesia tidak boleh dipakai di aplikasi.
Keputusan: Diterima user 2026-11-18 (T4-Q2). Teks revisi BRD v1.3 dan PRD v1.1 diperiksa di gate Tahap 4

### CR-008: Verifikasi keaslian resep digital oleh apotek non-rekanan

Diajukan: 2026-11-18 (Tahap 4, threat model API-13; `docs/04-catatan-arsitektur.md` T4-Q5)
Deskripsi: PDF resep memuat kode verifikasi acak dan QR ke halaman verifikasi di domain resmi Ruang Sehat (alamat juga dicetak sebagai teks). Halaman publik menampilkan status resep (Terbit/Dibatalkan), tanggal, dokter dan nomor SIP, klinik, daftar obat, dan nama pasien disamarkan; isi final mengikuti L-05/L-08. Batas laju per sumber; setiap verifikasi dicatat E-05 (pemeriksa anonim). Tanda tangan elektronik dipertimbangkan setelah L-05.
Alasan: apotek bukan rekanan, sehingga bukti keaslian harus dapat diperiksa tanpa akun; PDF saja dapat dipalsukan (threat model API-13, Tinggi).
Dampak ke scope / waktu / arsitektur: PRD v1.1 US-309 AC5–AC8; satu halaman publik di backend Ruang Sehat; kolom kode verifikasi di data dictionary A9.8. Penandaan "sudah ditebus" tidak termasuk (tidak dipilih user).
Keputusan: Diterima user 2026-11-18 (T4-Q5)

### CR-009: Kunci aplikasi tenaga kesehatan setelah 15 menit tidak aktif

Diajukan: 2026-11-18 (Tahap 4, threat model APP-02; `docs/04-catatan-arsitektur.md` T4-Q6)
Deskripsi: aplikasi tenaga kesehatan meminta verifikasi pemilik perangkat (biometrik atau PIN perangkat) saat dibuka kembali setelah 15 menit tidak aktif (NFR-SEC-10).
Alasan: mengurangi dampak ponsel tenaga kesehatan yang hilang atau dipinjam saat sesi masih aktif (sesi tenaga kesehatan bertahan sampai 7 hari tanpa aktivitas, ADR-011).
Dampak ke scope / waktu / arsitektur: PRD v1.1 NFR-SEC-10; ADR-021 butir 1; sedikit tambahan pekerjaan di aplikasi tenaga kesehatan.
Keputusan: Diterima user 2026-11-18 (T4-Q6)

### CR-010: Penjelasan AB-36 (pengecualian RF-13 dan RF-14)

Diajukan: 2026-11-18 (Tahap 4, review DBA B05; `docs/04-catatan-arsitektur.md` G3)
Deskripsi: aturan "satu pesanan paling banyak satu refund, kecuali RF-12" juga mengecualikan RF-13 dan RF-14, karena keduanya refund atas dana yang tidak menjadi pembayaran pesanan. Tanpa pengecualian ini, refund wajib (misalnya RF-01 setelah ada RF-14 pada percobaan bayar lain) ditolak constraint.
Alasan: temuan DBA B05 atas jalur sah di PRD §3.6.
Dampak ke scope / waktu / arsitektur: PRD v1.1 AB-36; data dictionary A14.1 (partial unique index); ADR-015 butir 6; KEPUTUSAN-67.
Keputusan: Diterima user 2026-11-18 (gate Tahap 4, G3)

### CR-011: Onboarding mandiri calon rekanan dan langganan berbayar RME

Diajukan: 2026-11-18 (Tahap 4, pertanyaan user; `docs/04-catatan-arsitektur.md` T4-Q7)
Deskripsi: calon rekanan (klinik/lab) dapat mendaftar sendiri (formulir, unggah dokumen izin, persetujuan perjanjian penggunaan RME dan pemrosesan data) dan berlangganan RME minimalis dengan membayar ke PT. Arunika. Verifikasi dokumen izin tetap oleh Verifikator sebelum rekanan tampil (BR-006, US-205) dan perjanjian pemrosesan data tetap syarat aktivasi (UP-49, L-15).
Alasan: permintaan user agar calon rekanan dapat *self-service/self-onboard*.
Dampak ke scope / waktu / arsitektur: kebutuhan bisnis baru (BR dan story baru bila diterima); modul pendaftaran mandiri, penagihan langganan, dan penangguhan RME; menambah risiko jadwal RT-10. Fakta bisnis belum diketahui (PT-34): apakah RME berbayar, model/harga/siklus, cara bayar, perlakuan telat bayar. Arsitektur menyiapkan titik perluasan (`docs/04-arsitektur.md` §13) tanpa membangunnya.
Keputusan: Dicatat; penempatan (rilis 1 atau berikutnya) diputuskan di Tahap 7 bersama estimasi effort (T4-Q7)
Tahap 7 (2026-11-18): estimasi ±108 jam-task (rentang ±90–150), bagian penagihan terblokir PT-34; opsi (a) rilis 1 penuh, (b) rilis 1 hanya pendaftaran mandiri tanpa penagihan (±45 jam-task), (c) rilis berikutnya. Usulan penyusun (c) `[UP-68]`, diputuskan user di Checkpoint 2 (`docs/07-phase-plan.md` §11.1)
Checkpoint 2 (2026-11-18): **rilis berikutnya** `[USER] C2-B5` (UP-68 c). Tidak ada story atau task rilis 1; titik perluasan arsitektur §13 tetap. PT-34 tidak lagi menjadi gerbang rilis 1. Dicatat di PRD §2.2 Won't

### CR-012: Hosting VPS Biznet Gio dan Hostinger, stack Rust/Next.js/Flutter, penyedia pihak ketiga, dan persetujuan deploy di sisi VPS

Diajukan: 2026-11-18 (Tahap 5, `docs/05-catatan-stack.md` T5-Q1–T5-Q8)
Deskripsi:
- Hosting: VPS Biznet Gio sebagai lokasi utama (aplikasi, database utama, Keycloak, Valkey, OpenBao, object storage WORM dan backup) dan Hostinger Jakarta sebagai lokasi kedua (database standby, salinan backup terenkripsi, node ketiga OpenBao, pemeriksa ketersediaan). Layanan terkelola yang tidak tersedia dijalankan sendiri: PostgreSQL 18, OpenBao sebagai pengganti KMS dan secret manager, Loki/Prometheus/Grafana/GlitchTip sebagai observability. Komponen "Redis" diimplementasikan dengan Valkey. Rincian dan biaya di ADR-025.
- Stack: backend Ruang Sehat dan RME minimalis memakai Rust; backoffice, portal rekanan, dan web RME memakai Next.js dengan server Next.js sebagai BFF; aplikasi mobile Flutter; satu monorepo (ADR-024).
- Penyedia pihak ketiga: payment gateway DOKU sebagai kandidat utama dan Xendit sebagai cadangan; email lewat Amazon SES region Jakarta (ap-southeast-3) lewat API, karena endpoint SMTP tidak tersedia di Jakarta (SRC-84, SRC-209); kurir Lalamove saja.
- CI/CD: GitHub untuk kode, CI, dan registry image (hanya kode dan data sintetis). Staging di-deploy otomatis; deploy production dipicu orang kedua yang berwenang lewat bastion dengan sertifikat SSH berumur pendek dari OpenBao (MFA), atas digest yang lulus staging, dan tercatat di log audit OpenBao.
Alasan: pilihan user T5-Q1 (Rust + Next.js), masukan user di tengah T5-Q1 ("vpsnya hostinger dan biznet gio saja") dan T5-Q2 (Biznet utama + Hostinger), T5-Q3 (DOKU utama, Xendit cadangan), T5-Q4 (SES Jakarta), T5-Q5 (Lalamove saja), T5-Q6 (GitHub + agen deploy VPS), dan T5-Q8 (persetujuan di sisi VPS). Persetujuan di sisi VPS dipilih karena required reviewers GitHub hanya tersedia untuk repositori publik pada paket Free, Pro, dan Team (SRC-240).
Dampak ke scope / waktu / arsitektur:
- ADR baru: ADR-024 (stack) dan ADR-025 (hosting, OpenBao, Valkey, observability, CI/CD, rahasia), keduanya berstatus Diusulkan.
- ADR yang diubah sebagian oleh ADR-025: ADR-019 (butir 1–4, alternatif "server virtual yang dikelola sendiri" kini dipilih), ADR-013 (butir 2, 3, 5, 8), ADR-021 (butir 3 dan 4), ADR-010 (penyimpanan log, butir 3–5), ADR-009 (butir 5, 6, 9), ADR-017 (butir 5), ADR-018 butir 5 (enkripsi lapis penyedia dan versi objek NEO Object Storage `[TIDAK DIKETAHUI]`, diverifikasi Tahap 8), ADR-013 butir 1 (tafsiran: lalu lintas di dalam satu host lewat jaringan Docker privat, ditampilkan di gate), dan ADR-007 (Redis menjadi Valkey, menunggu konfirmasi user). ADR-011 butir 7 ditafsirkan oleh ADR-024: BFF dijalankan di server Next.js. Teks ADR lama tidak diubah; baris Status-nya diberi catatan rujukan. Rincian per butir ada di tabel "Butir ADR lama yang diubah" di ADR-025.
- Modul kritis T1 bertambah: server Next.js (autentikasi dan sesi), konfigurasi OpenBao, bastion, dan skrip deploy production. Daftar modul kritis di `CLAUDE.md` diperbarui setelah gate Tahap 5.
- Dokumen yang terdampak: `docs/04-risk-register.md` (RT-21–RT-30 dan catatan perubahan risiko lama), `docs/05-stack.md`, `docs/05-response-codes.md`, Definition of Done di berkas ini (tiga butir baru), dan Business Case §6 (biaya infrastruktur ±Rp8,26 juta/bulan dibandingkan baseline di gate).
- Kewajiban operasional baru: patch keamanan Next.js berderajat Critical atau High dipasang paling lambat 7 hari (KEPUTUSAN-75); unseal OpenBao manual setelah restart oleh pemegang share; promosi standby database manual lintas penyedia.
- Waktu: kecepatan pengembangan Rust lebih lambat dengan besaran `[TIDAK DIKETAHUI]`, diukur di reference implementation Tahap 8 Fase 1 (RT-24). Beban operasional VPS menaikkan Business Case R-04 (RT-21).
- Pertanyaan terbuka baru: PT-35 (pemelihara setelah go-live), PT-36 (konfirmasi tertulis DOKU dan Xendit), PT-37 (konfirmasi tertulis Lalamove), PT-38 (pemegang share unseal OpenBao dan orang kedua), PT-39 (konfirmasi Biznet Gio: protokol load balancer, perutean Magic Transit, PPN).
- **Catatan tafsiran US-703 AC3 (PRD tidak diubah):** dengan kurir Lalamove saja, "pilihan kurir" pada US-703 AC3 dibaca sebagai jenis layanan atau kendaraan Lalamove yang tersedia untuk alamat itu. Alamat di luar area layanan Lalamove membuat checkout tidak dapat dilanjutkan, sesuai kalimat kedua AC3 `[USER] T5-Q5`. Tafsiran ini ditampilkan di gate Tahap 5.
- Syarat final penyedia: DOKU (atau Xendit) final setelah konfirmasi tertulis atas lokasi pemrosesan dan penyimpanan data, nomor izin BI, split settlement untuk margin dan bagian rekanan, refund dari dana yang sudah terbagi (PT-27), dan biaya (PT-36). Lalamove final setelah konfirmasi tertulis lokasi data (PT-37). Penyedia yang memproses data di luar Indonesia tidak dipilih (US-1406 AC2, `[USER] T4-Q1`).
Keputusan: Disetujui user lewat T5-Q1–T5-Q8 dan final di gate Tahap 5 (2026-11-18, T5-G2–G4): Valkey sebagai pengganti "Redis" dikonfirmasi (T5-G2 a); perutean Magic Transit diterima sementara sampai dikonfirmasi tertulis ke Biznet Gio (T5-G3, PT-39)

### CR-013: Pajak dikenakan atas harga rekanan + margin dan ditambahkan ke total

Diajukan: 2026-11-18 (Tahap 5, jawaban PT-28; `docs/05-catatan-stack.md` T5-Q11, T5-Q11a)
Deskripsi: pajak dihitung dari (harga rekanan + margin) setelah margin diterapkan, ditambahkan ke total yang dibayar pasien, dan tampil sebagai baris terpisah di rincian harga `[USER] T5-Q11, T5-Q11a`. Tarif pajak menjadi parameter berlaku-mulai yang diatur Keuangan (pola docs/04-data-dictionary.md §0.8). Jenis pajak, tarif, perlakuan pajak atas ongkos kirim dan biaya transaksi, pihak pemungut/penyetor, dan isi bukti bayar `[TIDAK DIKETAHUI]` (sisa PT-28).
Alasan: jawaban user atas PT-28 ("sama seperti layanan lainnya, karena markup dulu baru dibebakan pajaknya") dan pilihan "Di atas harga rekanan+markup" (kutipan asli user; kini: margin, CR-024).
Dampak ke scope / waktu / arsitektur: rumus harga PRD AB-11/AB-12 dan tampilan rincian harga (US-801, US-812) bertambah komponen pajak; harga terkunci (ADR-015 butir 3) menyimpan tarif dan nominal pajak; jurnal berpasangan menambah akun pajak tanpa perubahan struktur (ADR-015 konsekuensi); data dictionary A6/A8/A13 bertambah kolom/parameter pajak lewat migrasi; laporan keuangan dan rekonsiliasi. Teks PRD dan data dictionary diperbarui lewat CR ini sebelum pengembangan EP-08, setelah sisa PT-28 dijawab.
Keputusan: Disetujui user (T5-Q11a, 2026-11-18) untuk arah rumus; penerapan ke PRD dan data dictionary dijadwalkan di Tahap 7, sebelum EP-08. Disetujui di gate Tahap 5 (2026-11-18, T5-G4)
Tahap 7 (2026-11-18): diterapkan ke PRD v1.2 (draf, menunggu Checkpoint 2): AB-10a, AB-11, AB-13, AB-17, AB-21, AB-27, definisi §3.5, PB-21, US-801 AC1, US-804 AC2, US-811 AC1, US-812 AC4. Data dictionary dan migrasi dijadwalkan sebagai EN-24 di awal Fase 2, sebelum US-801, dengan review DBA `[KEPUTUSAN-185]`. Sisa PT-28 menjadi gerbang G-10
Checkpoint 2 (2026-11-18): teks PRD v1.2 untuk CR-013 disetujui `[USER] C2-B15`; PRD v1.2 final

### CR-014: Perbaikan teknis data dictionary dari review DBA Tahap 5

Diajukan: 2026-11-18 (Tahap 5, `docs/review/tahap-05-dba.md` F-P04 dan N04)
Deskripsi:
- Tabel sekali-isi `data_key_retention_anchor` (database A) dan `rme_data_key_retention_anchor` (database C, dengan `fasyankes_id`, FK komposit, RLS), kelas Transaksi: `data_key_id` PK/FK, `retention_category`, `retention_anchor_at`, `created_at`, `created_by`; role aplikasi hanya `SELECT`/`INSERT`, pekerja privasi `SELECT`. Melengkapi `data_key.retention_anchor_at`/`retention_category`: bila nilai di `data_key` sudah diisi saat INSERT, nilai itu yang berlaku; bila tanggal acuan baru diketahui belakangan, acuan dicatat sekali di tabel ini (dua sumber digabung dengan `coalesce`, desain di `docs/05-physical-schema.md` §4.12). Role pekerja RME tidak diberi hak atas tabel acuan RME. Acuan kosong berarti wadah tidak memenuhi syarat retensi (gagal-tertutup).
- Hak pembersihan `rme_slot_hold` dan `rme_upload_slot` untuk `rme_worker` (`SELECT` kolom yang diperlukan + `DELETE` dengan kebijakan RLS pembersihan baris kedaluwarsa), karena data dictionary §D.2 mewajibkan pembersihan tetapi hak `rme_worker` di §0.6 C tidak memuatnya.
Alasan: celah teknis yang ditemukan DBA. Memberi `UPDATE` ke role aplikasi atas `data_key` ditolak karena bertentangan dengan ADR-013 §5 dan KEPUTUSAN-66.
Dampak ke scope / waktu / arsitektur: data dictionary A19 dan C (tabel baru dan hak role) diperbarui setelah gate; desain fisik di `docs/05-physical-schema.md`. Tanpa perubahan PRD atau BRD. Keputusan teknis penyusun, ditampilkan di gate.
Keputusan: Disetujui di gate Tahap 5 (2026-11-18, T5-G2 e); data dictionary A19 dan C diperbarui saat migrasi terkait dibuat di Tahap 8
Tahap 7 (2026-11-18): dijadwalkan di task F1-T17 (DB aplikasi, `data_key_retention_anchor`) dan EN-22 Fase 3 (DB RME, `rme_data_key_retention_anchor` dan hak pembersihan `rme_worker`), keduanya dengan review DBA dan keamanan `[KEPUTUSAN-185]`

### CR-015: Vaksinasi dan suntik/infus vitamin sebagai layanan lab rekanan

Diajukan: 2026-11-18 (setelah gate Tahap 5, sebelum Tahap 6; pernyataan user saat membahas pamflet `docs/input/Rumah Sehat Apps.jpeg`)
Deskripsi: layanan vaksinasi dan suntik/infus vitamin yang tampil di pamflet masuk ke scope lab: ditawarkan lab rekanan, dipesan dan dibayar di aplikasi seperti pendaftaran lab (P-04, BR-024) `[USER] "sebenarnya vaksinasi atau infus masuk ke scope lab, mirip secara bisnis"`.
Alasan: secara bisnis serupa dengan pendaftaran lab (menurut user).
Dampak ke scope / waktu / arsitektur:
- Brief, BRD, PRD, dan backlog saat ini tidak menyebut vaksinasi maupun infus (dicek dengan pencarian teks 2026-11-18). Jenis layanan yang dikenal US-1303 AC1, PRD model data, dan data dictionary: layanan klinik, pemeriksaan lab, paket MCU, home visit.
- Jenis "pemeriksaan lab" terhubung ke alur hasil lab (US-901, US-902, US-1307) dan pengodean LOINC (ADR-020 butir 5). Vaksinasi dan infus tidak menghasilkan nilai lab, sehingga perlu diputuskan: jenis layanan baru, atau jenis yang ada tanpa alur hasil.
- Pencatatan tindakan dan vaksin/obat yang diberikan di RME minimalis serta pengirimannya ke SATUSEHAT: `[TIDAK DIKETAHUI]`, perlu dicek ke sumber resmi bila CR diterima.
- Kepatuhan: apakah laboratorium klinik rekanan boleh melakukan vaksinasi dan infus, atau harus fasyankes lain: `[TIDAK DIKETAHUI]`, diusulkan menjadi pertanyaan legal baru (menunggu PT-20).
- Tempat layanan: di lokasi lab, atau home visit sesuai regulasi `[USER] "di lab, atau home visit sesuai regulasi"` (2026-11-18). Varian home visit oleh lab memperluas BRD P-05 (saat ini home visit hanya disediakan klinik rekanan) dan bersinggungan dengan butir di luar scope BRD §4.2 "Pengantaran sampel lab atau pengambilan sampel di rumah" `[DOK] P3.6a`, sehingga perlu keputusan eksplisit: hanya vaksinasi/infus yang boleh di rumah, sedangkan pengambilan sampel tetap di luar scope, atau keduanya. Syarat regulasi home visit vaksinasi/infus oleh lab: `[TIDAK DIKETAHUI]`, masuk pertanyaan legal yang sama.
- Fakta bisnis yang belum diketahui: apakah klinik rekanan juga boleh menawarkan vaksinasi/infus.
Keputusan: Dicatat; dampak ke BRD/PRD dan penempatan (rilis 1 atau berikutnya) diputuskan di Checkpoint 2 / Tahap 7. Wireframe Tahap 6 boleh menampilkannya hanya sebagai butir katalog lab bertanda "menunggu CR-015"
Tahap 7 (2026-11-18): estimasi ±50–60 jam-task untuk varian di lokasi lab saja, tambahan ±60–100 jam-task bila termasuk home visit oleh lab; pencatatan tindakan/vaksin di RME dan SATUSEHAT serta syarat regulasi tetap `[TIDAK DIKETAHUI]`. Opsi (a) rilis 1 di lokasi lab, (b) rilis 1 termasuk home visit, (c) rilis berikutnya. Usulan penyusun (c) `[UP-69]`, diputuskan user di Checkpoint 2 (`docs/07-phase-plan.md` §11.2)
Checkpoint 2 (2026-11-18): **rilis berikutnya** `[USER] C2-B6` (UP-69 c). Ubin vaksinasi dan infus tidak tampil di beranda (UP-63); butir katalog lab "menunggu CR-015" di wireframe tidak dibangun di rilis 1. Pertanyaan regulasi dan pencatatan RME/SATUSEHAT tetap `[TIDAK DIKETAHUI]` untuk rilis berikutnya. Dicatat di PRD §2.2 Won't

### CR-016: Nama aplikasi menjadi "Ruang Sehat"

Diajukan: 2026-11-18 (setelah gate Tahap 6, sebelum Tahap 7)
Deskripsi: nama aplikasi berubah dari "Rumah Sehat" menjadi "Ruang Sehat" `[USER] "nama aplikasinya berubah, bukan "Rumah Sehat", tetapi "Ruang Sehat" sesuaikan semuanya ya"`. Menggantikan jawaban T5-Q12 (`docs/05-catatan-stack.md`), yang menetapkan nama tetap "Rumah Sehat".
Alasan: keputusan user (disebut user sebagai keputusan major).
Dampak ke scope / waktu / arsitektur:
- Tanpa perubahan fitur, alur, aturan bisnis, atau arsitektur. Nama diganti di semua dokumen hidup: `CLAUDE.md`, brief, Business Case, BRD, PRD, arsitektur, ERD, data dictionary, risk register, threat model, ADR, stack, skema fisik, response codes, checklist ASVS, coding standards, design guideline, catatan Tahap 6 (bagian yang masih menunggu Checkpoint 2), wireframe, backlog, decision log, traceability.
- Identifier teknis ikut diganti selagi belum ada kode dan migrasi: database `rumah_sehat` → `ruang_sehat`, `rumah_sehat_audit` → `ruang_sehat_audit`, kolom `rumah_sehat_order_id`, `rumah_sehat_partner_id`, `rumah_sehat_health_worker_id`, dan nilai `rumah_sehat_system` → awalan `ruang_sehat`. Singkatan "RS" dan awalan paket `rs_` (KEPUTUSAN-116) tetap berlaku.
- Tidak diubah karena merupakan rekam jejak: catatan wawancara Tahap 0–5 (`docs/0x-catatan-*.md`, termasuk kutipan jawaban user), laporan `docs/fact-check/`, `docs/qa/`, `docs/review/`, `docs/compliance/`, catatan status Tahap 5 di `docs/STATUS.md`, dan riwayat git. Nama berkas `docs/input/Rumah Sehat Apps.jpeg` serta PDF di `docs/ringkasan/` juga tetap; isi PDF masih memakai nama lama sampai dibuat ulang.
- Di luar repositori dan tidak diubah tanpa persetujuan user: nama folder proyek lokal dan nama repositori GitHub `Rumah-Sehat`.
- Pertanyaan baru: PT-45 (wordmark pamflet "RuangSehat" dan slogan dipakai atau "Ruang Sehat" dengan spasi), PT-46 (cek merek DJKI dan domain), di `docs/06-design-guideline.md` §12.
Keputusan: Diterima user 2026-11-18; diterapkan di dokumen hidup pada tanggal yang sama
Jawaban lanjutan 2026-11-18: PT-45 "dengan spasi" `[USER]` (nama selalu "Ruang Sehat" dengan spasi, wordmark pamflet "RuangSehat" tidak dipakai; slogan pamflet ditanyakan di Checkpoint 2); PT-46 "belum dicek" `[USER]` (cek merek DJKI dan domain masih terbuka, tenggat sebelum Tahap 12).

### CR-017: Ubah password saat login

Diajukan: 2026-11-18 (Tahap 7, temuan review keamanan C-07; `docs/07-klarifikasi-ac-fase-1.md` T-02, "US-103 AC6 usulan")
Deskripsi: pengguna yang sudah login dapat mengubah passwordnya. Password lama dan baru wajib diisi. Autentikasi ulang berlaku (ADR-011 butir 9, KEPUTUSAN-195). Password baru mengikuti NFR-SEC-03. Opsi "keluar dari perangkat lain" tampil dan tercentang bawaan (KEPUTUSAN-198). E-04 tercatat tanpa nilai kredensial. Berlaku di realm `pasien`, `nakes`, `admin`, dan `rme`.
Alasan: ASVS 5.0.0 6.2.2 (L1) "Verify that users can change their password." dan 6.2.3 (L1) "...requires the user's current and new password" `[SRC-276]`. PRD v1.1 belum punya AC untuk kemampuan ini.
Dampak ke scope / waktu / arsitektur:
- PRD v1.2 final US-103 AC6. Tanpa perubahan BRD (induk BR-001) dan tanpa ADR baru.
- Tambahan task Fase 1 untuk aplikasi pasien dan backoffice. Aplikasi tenaga kesehatan (Fase 2) dan web RME (Fase 3) mengikutinya. Ukuran dinilai di phase plan.
- Mekanisme Keycloak (required action atau konsol akun) `[TIDAK DIKETAHUI]`, diverifikasi di Tahap 8.
Keputusan: Diterima user 2026-11-18 lewat delegasi Checkpoint 2 butir 14 `[USER] C2-B14`; diterapkan ke PRD v1.2 final

### CR-018: MFA wajib untuk pasien (revisi ADR-011)

Diajukan: 2026-11-18 (Tahap 7, temuan S-01/T7-F9; `docs/07-klarifikasi-ac-fase-1.md` §5.1; `docs/review/tahap-07-security.md` §3)
Deskripsi:
- Pasien wajib MFA dengan TOTP, ditambah kode pemulihan sekali pakai (opsi M1).
- Pendaftaran TOTP dan pembuatan kode pemulihan terjadi saat login pertama setelah email terverifikasi.
- Atur ulang password tidak melewati MFA di semua realm, termasuk `pasien`.
- Pemulihan perangkat hilang: jalur mandiri dengan kode pemulihan; jalur Layanan Pelanggan dengan konfirmasi lewat email akun dan password (prosedur H-05 meluas ke pasien).
- WebAuthn, SMS, dan OTP email tidak dipakai.
Alasan: ASVS 5.0.0 mewajibkan MFA mulai Level 2 ("L2 applications must force the use of multi-factor authentication (MFA)", 6.3.3) `[SRC-265]`, sedangkan ADR-011 menargetkan Level 2 tetapi tidak mewajibkan MFA bagi pasien. User memilih M1 dari opsi M1–M4 `[USER] C2-Q9`.
Dampak ke scope / waktu / arsitektur:
- **ADR-011 direvisi** (butir 3, 4, 5, 9, butir 12 baru, alternatif, konsekuensi, riwayat revisi), status tetap Diterima. KEPUTUSAN-202–KEPUTUSAN-205.
- **PRD v1.2 final:** NFR-SEC-02, US-101 AC5, US-103 AC1, AC1b, AC4e, AC7, AC8, US-207 AC5.
- **Scope bertambah.**
  - Fase 1: konfigurasi alur MFA wajib dan Reset Credentials ber-OTP di realm `pasien` (EN-06); halaman pendaftaran TOTP dan kode pemulihan di tema login (UI-03); layar pasien untuk mendaftarkan ulang TOTP dan perangkat hilang; test negatif.
  - Fase 2: reset MFA pasien oleh Layanan Pelanggan di backoffice (US-207 AC5) dan prosedur operasi Layanan Pelanggan.
  - Ukuran task dinilai ulang di phase plan. Kebutuhan jam-task per hari kerja ikut naik (RT-10, RT-33).
- **Risiko:** dampak adopsi MFA wajib bagi pasien `[TIDAK DIKETAHUI]` (RT-35); beban Layanan Pelanggan dan rekayasa sosial (RT-36); pasien tanpa jalur pemulihan bila perangkat, kode pemulihan, dan email/password hilang bersamaan (RT-37). Risiko sisa threat model APP-04 ("pasien tanpa MFA") tertutup. Threat model diperbarui oleh pemiliknya.
- **Checklist ASVS:** 6.3.3 dirancang memenuhi untuk realm `pasien`; 6.4.3 dan 6.4.4 dirancang (`docs/05-asvs-checklist.md` V6).
- **Pertanyaan terbuka:** bukti identitas pengganti untuk kasus RT-37 `[TIDAK DIKETAHUI]`.
Keputusan: Diterima user 2026-11-18 `[USER] C2-Q9` (perubahan ADR yang sudah diterima, diputuskan langsung oleh user). Review keamanan atas revisi ADR-011, pembaruan test case oleh QA, dan fact-check dijalankan sebelum Tahap 8 (`docs/checkpoint-2.md` tindak lanjut 4)

### CR-019: Aplikasi mobile di repositori GitHub terpisah

Diajukan: 2026-11-19 (Tahap 8 Fase 1), `[USER]` lewat Telegram: "kita fokus pada BE kemudian web based. flutter dijalankan terpisah di laptop" dan "sebaiknya terpisah saja. untuk mobile di github berbeda saja"
Deskripsi:
- Kode Flutter (aplikasi pasien dan tenaga kesehatan) berada di repositori GitHub terpisah, dikerjakan mobile developer di laptop.
- Repositori ini (backend Rust, web Next.js, infra, db, docs) tidak memuat folder `mobile/`.
- Antarmuka antar-repositori adalah kontrak API OpenAPI di `openapi/` repositori ini.
Alasan: keputusan user.
Dampak ke scope / waktu / arsitektur:
- **Mengubah keputusan lama:** "satu monorepo" (KEPUTUSAN-82, KEPUTUSAN-89; `docs/05-stack.md` §6; `docs/coding-standards.md` §1.1; `CLAUDE.md` bagian Stack dan Perintah). Dokumen itu diperbarui di task tersendiri, termasuk perintah mobile di `CLAUDE.md`.
- Skrip verifikasi: langkah mobile di repositori ini dihapus dari daftar "BELUM DITERAPKAN" dan diganti penjaga yang GAGAL bila `mobile/` muncul di repositori ini (bukan pelonggaran: komponen pindah repositori). Standar hijau setara (format, analyze, test, aksesibilitas, osv-scanner untuk pub) wajib ada di repositori mobile; penyiapannya `[TIDAK DIKETAHUI]` (nama repositori, CI, hook) dan menjadi tanggung jawab mobile developer, dengan panduan dari dokumen ini.
- Task Fase 1 mobile (F1-T44, T45, T47, T48, T52, T53, T54, T58, T63, T77, T78) dikerjakan di repositori mobile. Review `code-reviewer` dan test plan tetap berlaku; cara review lintas repositori `[TIDAK DIKETAHUI]`.
- Jalur kritis Fase 1 (F1-T58, F1-T63) dan demo F1-T70 bergantung pada repositori mobile.
- `docs/test-plan.md` dan `docs/traceability.md`: kolom berkas kode untuk story mobile merujuk repositori mobile.
Keputusan: disetujui user 2026-11-19 `[USER]`
Status: disetujui; pembaruan dokumen dan skrip verifikasi dijadwalkan sebagai task orkestrator di Fase 1

### CR-020: Pengerasan bootstrap role database (temuan review F1-T13)

Diajukan: 2026-11-19 (Tahap 8 Fase 1, F1-T13; review DBA LULUS DENGAN CATATAN dan review keamanan putaran 1 BLOKIR High)
Deskripsi (perubahan teknis atas `docs/05-physical-schema.md` yang sudah lulus gate):
- §3.5: bootstrap wajib berjalan dengan `search_path = pg_catalog, pg_temp` di awal dan setelah setiap `\connect`, dan fungsi di blok DO dikualifikasi `pg_catalog.`, karena search_path tingkat database juga berlaku untuk sesi superuser (pola CVE-2018-1058; repro: pemilik skema menanam `app.format(text,text)` → role login menjadi SUPERUSER saat bootstrap dijalankan ulang).
- §2.1 butir 4 / §3.5 butir 2: `REVOKE CONNECT, TEMPORARY ON DATABASE template1 FROM PUBLIC`; penjaga encoding UTF8.
- §3.4: `openbao_*_admin` diberi `log_min_duration_statement = -1`, `log_min_duration_sample = -1` (CREATE ROLE ... PASSWORD tidak tercatat di log durasi), dan `createrole_self_grant = ''`.
- §3.4: self-grant ADMIN → SET → SET ROLE oleh `openbao_*_admin` dicatat sebagai risiko residu (sejalan dengan "setara pemilik skema"); kontrol detektif: log audit OpenBao dan pemeriksaan katalog berkala T-05.
Alasan: keamanan (temuan High) dan pengerasan DBA; tidak mengubah scope bisnis.
Dampak: dokumen skema fisik diperbarui; kode sudah menerapkan di F1-T13 (commit f0a40bd) dengan test negatif umpan.
Keputusan: keputusan teknis orkestrator (tanpa dampak bisnis), dilaporkan di Checkpoint 3
Status: diterapkan di kode; pembaruan teks `docs/05-physical-schema.md` menyusul (task dokumen orkestrator)

### CR-021: Web Ruang Sehat (company profile) dengan member area layanan korporat

Diajukan: 2026-11-19 (Tahap 8 Fase 1), `[USER]` lewat Telegram: "ada tambahan requirement. kita butuh web ruang sehat. web tersebut merupakan semacam company profile. tetapi ada member area untuk layanan corporate seperti vaksin masal, mcu masal dan semacamnya"
Deskripsi:
- Situs web publik Ruang Sehat bergaya company profile.
- Member area untuk klien korporat dengan layanan seperti vaksinasi massal dan MCU massal ("dan semacamnya").
Alasan: kebutuhan bisnis baru dari user.
Dampak ke scope / waktu / arsitektur (awal, belum dianalisis rinci):
- Scope baru di luar BRD/PRD v1.2 dan di luar tujuh layanan rilis 1. Tidak ada BR induk; perlu BR, story, dan AC baru.
- Bersinggungan dengan CR-015 (vaksinasi, sudah diputuskan rilis berikutnya) dan layanan MCU/lab (EP-09, info biaya MCU; dilepas di Opsi C).
- Pengguna baru (akun korporat, kemungkinan admin perusahaan dan peserta) → dampak ke autentikasi (realm baru atau pemakaian realm yang ada, ADR-011), otorisasi dan isolasi data (ADR-012), persetujuan data karyawan (UU PDP), pembayaran/penagihan korporat (ADR-015).
- Kapasitas: phase plan v0.2 sudah menyatakan scope penuh kemungkinan besar tidak layak sampai 1 Maret 2027 (Fase 2 ±43 jam-task per hari kerja).
Fakta bisnis `[TIDAK DIKETAHUI]`: penempatan rilis; isi company profile dan pengelola kontennya; siapa klien korporat dan peserta; alur pemesanan, penjadwalan, dan penagihan layanan massal; daftar "semacamnya"; integrasi dengan rekanan lab/klinik; regulasi vaksinasi massal.
Keputusan `[USER]` 2026-11-19 (Telegram): "1A, 2K" lalu "Web dan fitur ini pun masuk fase 1 dan harus"; setelah klarifikasi memilih **Y**: company profile **dan** member area korporat dibangun di Fase 1, dengan konsekuensi Fase 1 mundur atau sebagian task Fase 1 pindah ke Fase 2 (rincian dihitung dan diajukan ke user dulu). Peran login member area: HR/admin perusahaan + karyawan peserta (2K).
Status: analisis disetujui user 2026-11-19 (skenario S3, K-01–K-12, UP-79–UP-87, PT-67 Verifikator, PT-79 wajib bukti kewenangan, PT-72 tolak NIB ganda; `docs/cr/CR-021-wawancara.md` putaran 4). ADR-027/028 Diterima; ADR-029 Diusulkan (menunggu PT-59/L-21). Pembaruan BRD/PRD/phase plan/data dictionary menyusul (task dokumen orkestrator)
Diterapkan 2026-11-19 (F1-T81/F1-T82): BRD v1.4 (BR-130–BR-145, L-16–L-21), PRD v1.3 (EP-18, EP-19, AB-80–AB-87, PB-23–PB-25, E-18, UP-79–UP-87), data dictionary v1.1 (A20 rancangan dengan catatan DBA R2-01), ADR-006/011/018/024 direvisi (status tetap Diterima), threat model v1.3 (lingkup CR-021 LULUS), phase plan v0.3, traceability. Analisis: `docs/cr/CR-021-analisis.md` v0.3.
Status: **disetujui dan dijadwalkan** `[USER] C21-P4`. Fase 1: F1-T83–F1-T111 (situs, CMS, pendaftaran dan verifikasi perusahaan, akun HR); Fase 2: tujuh task pindahan Fase 1 dan reset MFA HR; Fase 3: pemesanan, peserta, invoice, ringkasan, hasil (gerbang G-25, G-26). Pertanyaan yang masih terbuka: PRD §12.6

### CR-022: Scope penuh diutamakan, go-live boleh mundur

Diajukan: 2026-11-19, `[USER]` Telegram: "semua CR tetap. masuk, jalankan paralel opsinua. tidak mundur" lalu "baik, kita kerjakan semaksimal mungkin. jika harus mundur kita ambil resikonya tetapi semua fitur harus berjalan dan komplit"
Deskripsi: semua fitur harus berjalan dan lengkap; bila tidak muat, tanggal go-live boleh mundur (risiko diterima user).
**Mengubah keputusan lama:** toleransi waktu 0 dan go-live tetap 1 Maret 2027 (`docs/01-business-case.md` §7.1; P8.3 brief); baseline Opsi B dan urutan pengurangan scope PRD §2.3 (UP-67, C2-Q1); kerangka gerbang re-plan 2026-12-11 (UP-66) yang memilih opsi pengurangan scope.
Dampak (awal, sebelum klarifikasi; **digantikan klarifikasi di bawah**: baseline tetap Opsi B + CR-021): baseline scope menjadi Opsi A (Fase 4 dikerjakan) ditambah CR-021; gerbang 2026-12-11 berubah dari "pilih opsi potong scope" menjadi "tetapkan tanggal go-live baru berdasarkan kecepatan terukur + keputusan tambahan tenaga (PT-53)". Dampak biaya (toleransi +20%, T1-Q8), jendela audit/toko aplikasi (RT-34), dan komitmen pemasaran (R-07) perlu ditinjau. Cakupan "semua fitur" (apakah CR-011 dan CR-015 yang ditunda ikut) `[TIDAK DIKETAHUI]`, ditanyakan ke user.
Klarifikasi `[USER]` 2026-11-19 23:21 UTC: "maksud ku tetap fase 1 selesaikan. kemudian selesaikan CR tambahan yang semalem baru di tambahkan".
Tafsiran orkestrator (dilaporkan ke user): (1) **urutan pengerjaan**: task baseline Fase 1 (F1-T01–F1-T80 yang tetap di Fase 1) diselesaikan lebih dulu, lalu task CR-021 Fase 1 (F1-T81–F1-T111) — mengubah penjadwalan S3 yang menyisipkan CR-021 di tengah Fase 1; (2) bila karenanya Fase 1 melewati 2026-12-11, risiko mundur diterima user; (3) cakupan rilis tidak diperluas oleh klarifikasi ini (pertanyaan A/B "semua fitur" dianggap terjawab: yang dimaksud penyelesaian Fase 1 + CR-021, bukan menambah CR-011/CR-015 atau membatalkan baseline Opsi B); keputusan scope akhir tetap di gerbang re-plan.
Keputusan: disetujui user 2026-11-19
Status: berlaku untuk urutan task Tahap 8; phase plan diperbarui (task dokumen orkestrator)

### CR-023: Jenis layanan dari data master Ruang Sehat, rekanan memilih saat onboarding

Diajukan: 2026-11-20, `[USER]` Telegram: "jenis layanan di ruang sehat lebih baik dari data master saja, bukan dari rekanan. nanti saat onboard rekanan mereka memilih layanan apa saja yg tersedia. ini lebih menghemat resources API dan waktu"
Deskripsi: katalog jenis layanan (konsultasi, pemeriksaan lab, paket MCU, vaksinasi korporat, dsb.) dikelola sebagai data master Ruang Sehat; rekanan saat onboarding memilih jenis layanan yang mereka sediakan dari master itu, alih-alih jenis layanan dikirim rekanan lewat Partner Integration API.
**Mengubah keputusan lama** (perlu analisis dampak): ADR-014 (kontrak integrasi rekanan) dan EP-04/EP-06/EP-09 bila jenis layanan bersumber dari rekanan; K-04 CR-021 (harga katalog korporat lewat integrasi rekanan, opsi a); peran portal rekanan (AB-62 read-only).
`[TIDAK DIKETAHUI]`: apakah **harga** juga ditetapkan di master atau tetap dari rekanan (per rekanan per layanan); apakah rekanan boleh mengusulkan jenis layanan baru; granularitas master (jenis vs item pemeriksaan per lab); siapa pengelola master (sub-peran admin).
Keputusan `[USER]` 2026-11-20 (Telegram "R"): **jenis layanan dari master Ruang Sehat; harga ditetapkan masing-masing rekanan per layanan yang dipilih (lewat portal/onboarding), lalu margin Ruang Sehat ditambahkan (ADR-015)**. Akibat: portal rekanan tidak lagi hanya-baca untuk pemilihan layanan dan harga (AB-62 direvisi); Partner Integration API tidak lagi menjadi sumber jenis layanan/harga katalog (ADR-014 direvisi); K-04 CR-021 berubah dari (a) integrasi ke portal. Butir terbuka lain (rekanan mengusulkan jenis baru, granularitas master, sub-peran pengelola master) dijawab saat analisis.
Status: disetujui; analisis dampak dokumen (BRD/PRD/ADR-014/AB-62/phase plan) dijadwalkan sebelum task katalog/onboarding rekanan (Fase 2–3)

### CR-024: Istilah "markup" diganti "margin" (rumus tetap)

Diajukan dan diputuskan: 2026-11-20, `[USER]` Telegram: "ubah semua istilah markup menjadi margin." lalu, atas klarifikasi, "I": **hanya istilah** yang berubah di semua dokumen, UI, dan kode; **rumus tetap** persentase dari harga rekanan (harga ke pasien = harga rekanan + persentase × harga rekanan, ADR-015), bukan margin akuntansi atas harga jual.
Dampak: penggantian istilah di BRD, PRD, ADR-015 dan ADR lain, data dictionary (nama kolom yang memuat "markup" — bila sudah ada di migrasi, perubahan nama kolom lewat migrasi expand/contract; bila belum dimigrasi, ganti di dokumen saja), phase plan, backlog, test plan, design guideline/wireframe, CLAUDE.md. Setiap dokumen menyertakan definisi: "margin = persentase dari harga rekanan (bukan dari harga jual)" agar tidak disalahartikan.
Status: diterapkan di dokumen 2026-11-20. Cakupan: dokumen gate dan acuan hidup (BRD, PRD, ADR-015, ADR-029, DD, ERD, arsitektur, threat model, stack, skema fisik, kode respons, design guideline, wireframe, phase plan, coding standards, test plan, traceability, STATUS, CLAUDE.md). Sengaja tidak diubah (rekaman sejarah dan kutipan): `docs/sources.md`, catatan wawancara (`docs/*catatan*`, `docs/cr/CR-021-*`), laporan `docs/fact-check/`, `docs/qa/`, `docs/compliance/`, `docs/review/`, `docs/checkpoint-2.md`, kutipan `[USER]` di entri backlog ini dan CR-013. Kode (`crates/`, `apps/`, `web/`) dan `db/test/kelas-app.sql` (memuat nama tabel `markup_rule`) diubah task developer; belum ada migrasi yang membuat tabel/kolom bernama markup.

### CR-025: Hasil yang bisa dipresentasikan lebih awal (hibrida per aplikasi)

Diajukan dan diputuskan: 2026-11-20, `[USER]` Telegram: "bagaimana bila. pembagaiannya per aplikasi, sehingga wujudnya ada dan bisa kita present ke CEO, bahkan tim marketing bisa mulai menjajaki pasar" lalu memilih **H** (hibrida).
Isi: (1) prototipe klik HTML statis dari wireframe yang ada (pasien, nakes, backoffice, situs, member area) untuk presentasi CEO, segera; (2) situs publik company profile + CMS (task CR-021 F1-T89–F1-T97 terkait situs/CMS) dimajukan menjadi hasil pertama yang berjalan di staging untuk marketing, paralel dengan fondasi; (3) fondasi backend tetap berjalan; setiap fase berikutnya menghasilkan satu aplikasi/alur yang bisa didemokan.
**Mengubah keputusan lama:** urutan CR-022 (Fase 1 asli dulu, baru CR-021) — khusus situs publik + CMS kini paralel/dimajukan; sisa CR-021 (realm korporat, pendaftaran perusahaan) tetap setelah baseline.
Ketergantungan: domain dan merek (PT-46), konten situs (Tentang Kami, layanan, rekanan; `[TIDAK DIKETAHUI]`), staging (EN-15, PT-39 Biznet Gio) — tanpa staging, situs hanya bisa didemokan di server pengembangan.
Status: disetujui; prototipe klik dikerjakan 2026-11-20; task situs/CMS dijadwalkan di pemanggilan /runbook auto berikutnya (batas 45 task sesi ini tercapai)

### CR-026: Aksi Keycloak berhak tinggi lewat worker; pendaftaran tanpa password di formulir

Diajukan dan diputuskan: 2026-11-22, `[USER]` Telegram "A" (KEPUTUSAN-302) dan "R1" (KEPUTUSAN-303); rincian ADR-030.
Isi: semua aksi Keycloak Admin REST ber-`manage-users` (realm pasien, admin, korporat) dipindah dari proses `api` ke `worker` (niat + job tertutup, akun `pending_provisioning`, alarm admin event); formulir pendaftaran mandiri pasien (US-101) dan HR (US-1901) tanpa password — password dibuat lewat tautan aktivasi email; undangan HR ke email terdaftar ditampilkan seragam.
**Mengubah keputusan lama:** KEPUTUSAN-201 dan klarifikasi §2.7 butir 4 (pembuatan akun sinkron); AC PRD US-101, US-208, US-1901, US-1903, US-1904 perlu revisi (tabel dampak di ADR-030); menutup TD-26 dan TD-30 setelah implementasi langkah 1–8 ADR-030.
Status: disetujui; implementasi dijadwalkan setelah registrasi F1-T49–T51 masuk branch utama
Progres: ADR-030 langkah 1–4 untuk realm `korporat` selesai di commit d5bdca9 (2026-11-25, menunggu review T1 penuh: code-reviewer + security + DBA): migrasi `pending_provisioning` + niat/riwayat/proyeksi, modul worker IdP enam jenis job, rahasia klien worker hanya di OpenBao (`rs-worker.hcl`), atribut `rs_intent_id` + klien `rs-backend-idp-worker` (KEPUTUSAN-305/306). Sisa: langkah 1–4 untuk realm `pasien`/`admin` (F1-T50/T51/T65, bootstrap), langkah 5–8 (pengambil admin event + alarm, rekonsiliasi niat `pending` dan job yang digagalkan sweeper), use case F1-T101/T105/T107 memakai `register_pending_account`/`record_intent`, fungsi security log untuk penolakan job IdP, metode autentikasi proses ke OpenBao `[TIDAK DIKETAHUI]`. Temuan di luar scope: realm `korporat` tanpa `eventsListeners` email, sehingga 2 pemeriksaan TD-27 di `keycloak-check.sh` gagal (sudah ada sebelum commit ini)
Review T1 gabungan (keamanan + DBA + kode) langkah 1–4: LULUS DENGAN CATATAN; catatan 2 (prasyarat `inactive` untuk `idp_disable_user`, FK komposit realm niat = realm akun) dan catatan 3 (batch worker IdP 1, validasi sewa saat start) diperbaiki di commit lanjutan. Syarat yang belum dikerjakan:
- Catatan 1 (wajib sebelum `idp_reconcile` punya pemanggil, langkah 8): rekonsiliasi yang mengadopsi selagi niat asal `create_user` masih `pending` membuat job `create_user` berikutnya menjadwalkan `idp_delete_orphan` dan menghapus pengguna yang baru diadopsi.
- Catatan 4 (syarat penutupan langkah 7): akurasi audit bila efek Keycloak terjadi tetapi transaksi hasil gagal ditutup lewat pencocokan admin event; alarm admin event ADR-030 butir 9 wajib aktif sebelum alur korporat dipakai di production.
- Catatan 5 (pemanggil F1-T101/F1-T105): `normalize_email` (trim + huruf kecil Unicode) tidak sama dengan `lower(email)` PostgreSQL tanpa trim, sehingga pemanggil wajib memvalidasi email ASCII ternormalisasi; kunci advisory email diambil pertama, pemanggil tidak memegang kunci baris lain sebelumnya (aturan tertulis di doc `register_pending_account`).

### CR-027: Hook pre-commit mengizinkan docs/ belum di-stage padahal langkah verifikasi membaca docs/

Diajukan: 2026-11-25 (Tahap 8 Fase 1, review T1 KEPUTUSAN-301, catatan non-blokir N-5)
Deskripsi: `.githooks/pre-commit` menolak perubahan belum di-stage/belum dilacak di luar `docs/`, tetapi `scripts/verify.sh` memeriksa pohon kerja, dan beberapa langkah membaca berkas `docs/` (mis. registry `docs/05-response-codes.md` di test Rust dan web, data dictionary). Akibatnya commit dapat diverifikasi terhadap isi `docs/` di pohon kerja yang berbeda dari yang di-commit: hijau di lokal, tetapi isi yang ter-commit tidak pernah diverifikasi lokal (CI tetap memverifikasi isi commit). Ini sudah ada sebelum KEPUTUSAN-301; cache sidik tidak memperburuknya karena sidik dihitung dari pohon kerja yang sama dengan yang diperiksa.
Usulan: hook menolak juga perubahan `docs/` yang belum di-stage bila commit memuat kode (atau verifikasi dijalankan atas salinan index). Mengubah `.githooks/` memerlukan persetujuan user (CLAUDE.md, aturan nomor dua).
Alasan: integritas gerbang "hanya commit hijau"; tidak mengubah scope bisnis.
Keputusan: menunggu user
Status: diajukan; `.githooks/` tidak diubah

### CR-028: Perbaikan tiga bug demo publik (komponen client, audience mapper, koneksi Valkey BFF)

Diajukan: 2026-11-26 (persiapan demo publik); disetujui user via Telegram 2026-11-26
Deskripsi: (1) `web/packages/ui/src/Button.tsx` merender `onClick` tanpa `"use client"` dan dipakai dari server component `KerangkaKorporat`/`KerangkaBackoffice`: semua halaman terautentikasi gagal 500 ("Event handlers cannot be passed to Client Component props"); satu-satunya komponen `@rs/ui` dengan handler adalah Button. (2) Realm-as-code tidak memuat audience mapper sehingga API menolak token BFF (`wrong_audience`). (3) `ValkeyBackend` memeriksa status `wait` saja, sehingga permintaan pertama setelah start dan yang serentak gagal "Stream isn't writeable and enableOfflineQueue options is false".
Perbaikan: (1) `"use client"` di Button + test statis `web/scripts/batas-client.test.ts` (komponen dengan handler event/hook wajib berdirektif, lintas ui, korporat, backoffice, portal, rme). (2) `oidc-audience-mapper` pada `rs-korporat-bff` (audiens `rs-corporate-api`) dan `rs-backoffice-bff` (`rs-backoffice-api`), hanya access token, nilai sama dengan `AUTH_CORPORATE_AUDIENCE`/`AUTH_ADMIN_AUDIENCE`; test `web/scripts/realm-audience.test.ts` dan pemeriksaan hidup di `keycloak_check.py` (`cek_konfigurasi`). (3) `ValkeyBackend` menunggu koneksi siap dengan batas `timeoutMs`, penantian dibagi bersama; gagal dilempar `BackendUnavailableError` (rute BFF menjawab 503); test `web/packages/bff-auth/src/store.test.ts`.
Alasan: perbaikan cacat; tidak mengubah scope bisnis atau pengaturan keamanan lain.
Keputusan: disetujui user
Status: selesai (commit tercatat di riwayat git cabang tahap-08-fase-1 setelah merge)

### CR-030: Integritas persetujuan di database: FK gabungan butir-versi dan jenis teks di bukti

Diajukan: 2026-11-26 (Tahap 8 Fase 1, review DBA F1-T101, butir b)
Deskripsi: kesesuaian butir dengan versinya dan jenis teks dengan subjek bukti persetujuan kini hanya dijaga aplikasi (`privacy::api::record_granted_consent`, KEPUTUSAN-321). Usulan penguatan di database: (1) FK gabungan `consent_record (consent_item_id, consent_text_version_id)` ke `consent_item (id, consent_text_version_id)` (butuh UNIQUE `(id, consent_text_version_id)` di `consent_item`); (2) opsional kolom `consent_type` di `consent_record` dengan CHECK yang mengikat jenis ke subjek (HR hanya `corporate_hr_registration`, pasien tiga jenis lain) dan FK gabungan ke `consent_text_version (id, consent_type)`. Migrasi expand dengan review DBA; pada tabel Log besar memakai NOT VALID lalu VALIDATE di migrasi terpisah.
Alasan: pertahanan berlapis bila jalur tulis lain menyisipkan bukti tanpa lewat `privacy::api`.
Keputusan: menunggu user
Status: diajukan

### CR-031: Dokumen perusahaan diunggah HR setelah aktivasi akun; bentuk bukti kewenangan

Diajukan dan diputuskan: 2026-11-26, `[USER]` Telegram Q8=A dan Q9=A (KEPUTUSAN-343).
Isi: (1) dokumen NIB, NPWP, dan bukti kewenangan diunggah HR **setelah** aktivasi akun (password + TOTP) dari sesi korporat, bukan saat formulir pendaftaran dikirim; tidak ada slot unggah anonim; perusahaan self-registered baru muncul di antrean Verifikator sebagai "siap diverifikasi" setelah ketiga jenis dokumen lengkap, dan persetujuan tanpa dokumen lengkap ditolak; (2) bukti kewenangan = surat kuasa atau surat penugasan dari direksi, PDF atau foto, bertanda tangan; sistem tidak memvalidasi tanda tangan, itu tugas Verifikator.
**Mengubah teks lama:** PRD US-1901 AC1 ("mengunggah dokumen NIB, NPWP, dan bukti kewenangan pendaftar ... When mengirim" dan kalimat "Bentuk bukti kewenangan ... dikonfirmasi user") dan AC2 ("pengiriman ditolak per field" untuk dokumen tidak lengkap, kini dokumen diunggah di langkah terpisah dan kelengkapan dinilai di antrean/persetujuan); US-1902 AC1 (antrean memuat pengajuan siap diverifikasi); PT-79 terjawab. PRD diperbarui pada revisi PRD berikutnya; sampai itu keputusan ini yang berlaku.
Status: disetujui user; diimplementasikan di F1-T102 (backend); UI di F1-T103

### CR-032: Tautan ke member area korporat dari situs publik

Diajukan: 2026-11-26 (Tahap 8 Fase 1); disetujui user via Telegram 2026-11-26 ("jalankan")
Deskripsi: `web/situs` memuat tautan ke web korporat (member area HR): blok "Sudah terdaftar?" dengan tombol "Masuk member area" di halaman Untuk Perusahaan (mengikuti `docs/prototype/situs/04-perusahaan.html` dan `docs/wireframes/s01-situs-publik.html`) dan tautan kecil "Masuk HR" di footer. Alamat dari env server `SITUS_KORPORAT_URL` (domain member area `[TIDAK DIKETAHUI]`), divalidasi fail-closed; tidak valid atau kosong → tautan tidak dirender (KEPUTUSAN-350, KEPUTUSAN-351).
Catatan: tombol "Daftarkan perusahaan" TIDAK dibuat di task ini; menyusul bersama F1-T103 karena halaman tujuannya belum ada.
Alasan: pengunjung HR perlu jalan masuk ke member area dari situs publik tanpa membakukan domain di build.
Keputusan: disetujui user
Status: selesai (commit tercatat di riwayat git cabang tahap-08-fase-1 setelah merge)

### CR-033: Perombakan visual situs publik tahap pertama (C-01, C-02, C-03)

Diajukan: 2026-11-26 (arahan user: tampilan situs harus "lebih fancy, profesional, eye catching; sekarang sangat kaku dan jelek"; `docs/design/brief-situs.md` v0.1 §7, §10); disetujui user via Telegram 2026-11-26 ("jalankan rekomendasimu")
Keputusan user: Q15 = konsep A "Ruang Teduh" + seksi gelap ala konsep B untuk blok Untuk Perusahaan (Beranda) dan hero halaman Untuk Perusahaan; Q16 = kerjakan C-01, C-02, C-03 `[USER]`
Cakupan:
- C-01 gaya visual konsep A di `web/situs`: token situs `--situs-*` (skala tipografi, ritme seksi, kontainer, radius; DG v1.3 §3.3a/§3.4a), gradien `bg.app → mint`, ornamen SVG sebaris, kartu sudut 20–24 + bayangan token, hover/transisi CSS yang menghormati `prefers-reduced-motion`, header lengket, pita judul halaman isi, footer empat kolom, seksi gelap `primaryStrong`, normalisasi tingkat judul Markdown (KEPUTUSAN-353, -357, -358)
- C-02 kartu layanan berikon di Beranda dan halaman Layanan dari konvensi Markdown `services` (`##` = kartu, paragraf pertama = ringkasan, urutan dari CMS, fallback Markdown biasa), ikon SVG buatan sendiri dengan pemetaan kata kunci + ikon generik (KEPUTUSAN-354, -355)
- C-03 hero Beranda: mockup ponsel ilustratif di atas blob daun bila CMS tanpa gambar, dua kartu mengambang dan tiga chip nilai faktual, CTA ganda tetap; bingkai gambar baru menggantikan kotak putus-putus (D3) (KEPUTUSAN-359)
Perbaikan review keamanan+kode (LULUS DENGAN CATATAN, commit lanjutan): teks pita judul memakai `text-primary` dan ornamen pita judul disembunyikan di bawah 1024 px (kontras di atas daun ornamen), halaman draf (Kebijakan) tidak memindah paragraf pembuka sehingga spanduk "Draf." tampil sebelum isi (US-1801 AC4), test negatif masukan berbahaya di kartu/sorotan Beranda, isi contoh `services` berbentuk `##` agar `periksa-build` melewati jalur kartu.
Di luar cakupan: C-04–C-16 kecuali C-13 (font merek, ditambahkan lewat Q18; penanda kosong publik C-11 tetap di luar cakupan), perubahan backend/CMS/OpenAPI, `web/packages/ui`.
Alasan: dampak visual terbesar tanpa perubahan model konten (brief §10 saran urutan).
Keputusan: disetujui user
Keputusan lanjutan user 2026-11-26 (Telegram, "eksekusi sesuai rekomendasimu") `[USER]`: Q17 = kelima teks klaim hero (tiga chip nilai, dua kartu mengambang) disetujui apa adanya (KEPUTUSAN-359); Q18 = font Plus Jakarta Sans di-host sendiri untuk situs (KEPUTUSAN-356, SRC-881/882, DG §3.3a).
Status: selesai (commit tercatat di riwayat git cabang tahap-08-fase-1 setelah merge). Tindak lanjut yang perlu keputusan user: aturan tinggi baris 1,3 untuk judul (DG §3.3a), pemeriksaan visual di peramban sungguhan (belum dilakukan di server: pustaka sistem peramban headless tidak terpasang)

### CR-035: NIB terdaftar tetap boleh diajukan; NIB unik hanya di antara perusahaan Aktif

Diajukan dan diputuskan: 2026-11-26, `[USER]` keputusan Q10 = B2 (diteruskan orkestrator); diimplementasikan di task Q10-B2 (KEPUTUSAN-362–365).
Isi: (1) pendaftaran perusahaan mandiri dengan NIB yang sudah dipakai perusahaan lain (status apa pun) **tidak ditolak**: pengajuan dibuat seperti biasa (perusahaan Menunggu verifikasi, akun HR, persetujuan), jawaban `202` dan email aktivasi yang diterima pendaftar **identik** dengan pendaftaran NIB baru (anti-enumerasi; tidak membocorkan bahwa NIB sudah terdaftar); (2) pengajuan itu masuk antrean Verifikator dengan penanda "NIB sudah dipakai perusahaan lain" (`nib_used_by_other_company`, antrean dan detail); (3) NIB unik **hanya di antara perusahaan Aktif** yang belum dihapus lunak (`uqp_corporate_client_nib_active`), bukan di antara pengajuan Menunggu verifikasi atau Ditolak; persetujuan atas pengajuan yang NIB-nya sudah dipakai perusahaan Aktif lain ditolak `409` `4090116`; (4) akibat langsung butir 3 (tafsiran pelaksana, mohon konfirmasi): Verifikator yang membuat perusahaan (US-1903) hanya ditolak `4090110` bila NIB sudah dipakai perusahaan **Aktif**, tidak lagi oleh pengajuan Menunggu verifikasi/Ditolak/Nonaktif.
**Mengubah teks lama:** BRD BR-136 ("NIB yang sudah terdaftar ditolak" dan "cara menangani NIB ganda (diarahkan ke Layanan Pelanggan)"); PRD US-1901 AC7 ("Given NIB yang sama sudah dipakai perusahaan terdaftar, When mendaftar, Then pendaftaran ditolak dengan pesan generik ... dan pendaftar diarahkan menghubungi Layanan Pelanggan"); PRD US-1903 AC4 ("ditolak seperti US-1901 AC7", kini hanya NIB perusahaan Aktif); PT-72 ("NIB terdaftar ditolak"); DD A20 `corporate_client` ("UQp `nib WHERE deleted_at IS NULL` (NIB ganda ditolak)"); design guideline §7 baris galat NIB ganda K01 (tidak lagi tampil di pendaftaran mandiri). BRD/PRD/DD diperbarui pada revisi dokumen berikutnya; sampai itu CR ini yang berlaku. Menutup TD-45 dan bagian penguncian NIB TD-46.
Status: disetujui user; diimplementasikan commit 943da6c (backend, migrasi, kontrak backoffice, penanda minimal di `web/backoffice`); review T1 (keamanan + DBA) LULUS DENGAN CATATAN, perbaikan menunggu konfirmasi: data dictionary A20, kode respons `4090110`, dan BR-136 di traceability disinkronkan; test jalur penjaga index unik deterministik; kursor antrean `(submitted_at, id)` (KEPUTUSAN-366); aturan deploy expand/contract (KEPUTUSAN-367); kriteria jalur aktivasi masa depan (TD-51)

### CR-036: Koreksi data perusahaan oleh HR saat Ditolak, lalu ajukan ulang

Diajukan dan diputuskan: 2026-11-27, `[USER]` keputusan Q20 = A (diteruskan orkestrator; menjawab pertanyaan P-T102b-1). Belum diimplementasikan (butuh backend; `apps/rs-backend/**` di luar kepemilikan F1-T103).
Isi: HR boleh mengoreksi data perusahaan (nama badan usaha, NIB, NPWP, alamat kantor, telepon kantor) **hanya saat pengajuan berstatus Ditolak**, lalu mengajukan ulang; setiap koreksi diaudit; Verifikator melihat data terbaru saat memeriksa pengajuan ulang.
Cakupan usulan: (1) backend: endpoint HR koreksi data perusahaan (hanya dari Ditolak, validasi sama dengan pendaftaran KEPUTUSAN-325, kebijakan NIB CR-035), E-10 dengan before/after tanpa nilai NPWP lengkap (pola ADR-005), kontrak `openapi/korporat.json`, data terbaru di antrean dan panel Verifikator; (2) web korporat: form koreksi di halaman `/verifikasi` saat Ditolak (prefill data `company`, validasi klien NIB 13/NPWP 16 seperti `/daftar`, NPWP tetap tersamar saat tidak diedit sesuai Q23 = A, galat per field, lalu "Ajukan ulang"); teks sementara Q31 ("Bila ada data yang salah, hubungi Layanan Pelanggan") dihapus saat fitur ini tayang; (3) backoffice: panel Verifikator menampilkan penanda "data dikoreksi HR" bila diputuskan perlu (`[TIDAK DIKETAHUI]`, tanyakan saat task).
Status: disetujui user; diimplementasikan (backend + OpenAPI + web korporat), commit `a66a26f`, menunggu review T1 (keamanan + DBA). Keputusan pelaksanaan KEPUTUSAN-410–414: koreksi digabung dengan pengajuan ulang dalam satu aksi `POST /v1/corporate/company-corrections` (satu `UPDATE` bersyarat status + putaran), E-10 `corporate_client_data_corrected` nama field saja, batas laju bersama pengajuan ulang, NIB perusahaan lain diproses sama dengan NIB baru, UI di bagian "Ajukan ulang" `/verifikasi` (NIB/NPWP tetap tersamar, teks sementara Q31 diganti). Butir (3) penanda "data dikoreksi HR" di panel Verifikator tidak dibuat (KEPUTUSAN-414, butuh perubahan skema; pertanyaan ke user). Test: `apps/rs-backend/tests/corporate_correction.rs`, `corporate_isolation.rs` (kasus H-18), `web/korporat/src/verifikasi-e2e.test.ts`, `src/komponen/verifikasi/{verifikasi,a11y}.test.tsx`, `src/lib/verifikasi/koreksi.test.ts`. Perbaikan review T1 (DBA B1/C1/C2, keamanan T-01/T-02; KEPUTUSAN-415–419): token keputusan Verifikator terikat putaran, audit koreksi before/after (NIB/NPWP `last4`), `submitted_at` setelah kunci, teks bersama menolak bidi/lebar-nol, isian koreksi diselaraskan setelah berhasil; C3 dan NFC dicatat TD-57; menunggu review ulang.

### CR-037: E-05 saat Verifikator membuka detail perusahaan (data Terbatas)

Diajukan dan diputuskan: 2026-11-27, `[USER]` keputusan Q29 = A (Telegram, diteruskan orkestrator); diimplementasikan di batch perbaikan F1-T109.
Isi: setiap kali Verifikator membuka detail perusahaan (`GET /backoffice/v1/corporate-verifications/{corporate_client_id}`, memuat NPWP berklasifikasi Terbatas, DD A20) dicatat E-05 `corporate_detail_opened` atas objek `corporate_client_detail` (ID = perusahaan, `tenant_id` = perusahaan, tanpa subjek dan tanpa field). Event di-commit sebelum detail dikembalikan; gagal mencatat = `500` tanpa detail; perusahaan tidak ada = `404` tanpa event. NPWP, NIB, nama, alamat, dan telepon tidak pernah masuk payload. Keputusan teknis: pustaka audit (`audit::check_payload`) mendapat pengecualian subjek SEMPIT gaya KEPUTUSAN-304 untuk jenis objek `corporate_client_detail` hanya bila `tenant_id` = `object_id` (`crates/audit/src/requirements.rs`, test unit negatif). Test: `apps/rs-backend/tests/corporate_verification.rs::opening_company_detail_records_e05_without_restricted_data`.
**Mengubah teks lama:** PRD §7.1 E-05 ("akses data pasien") diperluas ke data Terbatas perusahaan; revisi PRD menyusul (belum diubah di task ini). Kontrak OpenAPI backoffice tidak diubah (respons `500` endpoint detail belum dicantumkan karena menuntut regenerasi klien di `web/**`).
Status: disetujui user; diimplementasikan, menunggu review T1 (keamanan, pustaka audit).
### CR-029: CI test-verify.sh dipecah jadi job matrix paralel per kelompok fixture

Diajukan: 2026-11-26 (review CI; run 45733028822 berhenti di `timeout-minutes: 60`); opsi A dipilih user via Telegram 2026-11-26
Deskripsi: CI menjalankan seluruh `scripts/test/test-verify.sh` dalam satu job. Skrip itu menjalankan verify.sh lengkap untuk hampir setiap fixture (fixture pertama sekitar 23 menit di runner bersih, fixture berikutnya belasan menit), sehingga job tidak pernah selesai dalam batas 60 menit dan status CI tidak bisa dibaca hijau atau merah (SRC-858).
Perbaikan: fixture dikelompokkan (`kelompok_blok`), dipilih lewat `RS_TEST_VERIFY_KELOMPOK` (tanpa pilihan: semua, cakupan sama atau lebih luas, durasi lokal bertambah sekitar 5 run verify.sh (KEPUTUSAN-390); nama tidak dikenal gagal); job matrix `test-verify` di `.github/workflows/ci.yml` (runner `ubuntu-24.04` standar, `fail-fast: false`, batas 60 menit tidak dinaikkan); job `verify` dipertahankan; test kelengkapan `scripts/test/test-verify-kelompok.sh` (gabungan kelompok = seluruh fixture; matrix = kelompok skrip). KEPUTUSAN-340 sampai KEPUTUSAN-342; SRC-856 sampai SRC-858; `docs/05-stack.md` §3.2.
Alasan: perbaikan alur CI; tidak mengubah `scripts/verify.sh`, tidak mengubah cakupan test, tanpa runner berbayar.
Keputusan: disetujui user (opsi A)
Status: selesai (commit 27242c0, 5ed67c6, 1a2252f, d10d744, 55f5cfa di cabang agen; run CI nyata keempat 45844577354 hijau seluruhnya, 31 job; perbaikan review dan merge di commit 3b15c9c, run CI nyata kelima 45949956430 hijau seluruhnya, 32 job termasuk `test-verify-hasil`, job terpanjang 51,3 menit, SRC-958; perbaikan verifikasi ulang R-1 dan R-2 di commit lanjutan). Catatan sejarah: run CI nyata pertama menemukan dua cacat laten test-verify.sh (lisensi `--offline`, path cache di cache-isi) dan kelompok terlalu panjang (dipecah menjadi satu fixture penuh per kelompok, 30 kelompok); ukuran dan data run di KEPUTUSAN-342 Perbaikan CI 2026-11-27: commit abb6cff (merge `tahap-08-fase-1`) dan 64d53d4 (diagnostik galat `test-verify.sh`, TD-61, KEPUTUSAN-395); run CI nyata keenam 46013122998 hijau seluruhnya, 32 job, job terpanjang 48,5 menit.

### CR-038: Header situs: deretan tautan navigasi dan tautan toko aplikasi di pojok kanan

Diajukan dan diputuskan: 2026-11-27, arahan user via Telegram `[USER]`: "menu di pojok atas situs tidak bagus. lebih baik menu link yg dipasang berurutan di header. kemudian pojok kanan lebih baik logo android dan apple yg merupakan link ke playstore dan appstore".
Isi: (1) menu lipat "Menu" diganti deretan tujuh tautan di header (di layar sempit baris kedua yang dapat digulir mendatar), halaman aktif `aria-current="page"` (KEPUTUSAN-420, 421); (2) tautan Google Play dan App Store di pojok kanan; (3) alamat toko dari env `SITUS_PLAYSTORE_URL` / `SITUS_APPSTORE_URL`, fail-closed, fallback ke `/unduh` (KEPUTUSAN-423, 424).
**Berbeda dari permintaan:** pedoman merek resmi (SRC-945–947) melarang logo Apple berdiri sendiri dan mengarahkan lencana resmi untuk ajakan unduh; aset lencana hanya terunduh setelah menyetujui perjanjian lisensi atas nama pemegang akun pengembang (Apple: hanya untuk aplikasi yang sudah ada di App Store). Agent tidak dapat menyetujuinya, sehingga sementara tautan berupa teks "Google Play" dan "App Store"; lencana resmi menunggu aset dari pemilik akun (KEPUTUSAN-422).
Status: sebagian diimplementasikan (butir 1 dan 3 selesai, butir 2 teks sementara); menunggu review T1 dan keputusan user soal aset lencana.

### CR-039: Web korporat "sederhana dan memandu" dengan lima menu (Beranda, Pesanan, Peserta, Tagihan, Tim HR)

Diajukan dan diputuskan: 2026-11-27, arahan user `[USER]` (Q32 "ya" untuk arah tampilan "sederhana dan memandu"; "Ya" untuk lima menu). Persona: HR perusahaan pelanggan, sibuk, bukan orang teknis, "tidak mau yang ribet".
Isi: (1) Beranda menampilkan status perusahaan dan satu tombol langkah berikutnya, bukan dasbor angka (KEPUTUSAN-431); (2) menu Beranda, Pesanan, Peserta, Tagihan, Tim HR; fitur yang belum ada abu-abu "segera hadir"; "Invoice" menjadi "Tagihan", "Kelola HR" menjadi "Tim HR" (KEPUTUSAN-430); (3) status verifikasi tetap di `/verifikasi`, dicapai dari Beranda, ditambah penanda "Langkah x dari 3" (KEPUTUSAN-432); (4) target sentuh 44 px dan mobile-first (KEPUTUSAN-433). Tanpa perubahan backend, OpenAPI, BFF, dan dependensi.
Status: diimplementasikan di `web/korporat`, commit `2eaa8e8`; menunggu review T1 (BFF). Catatan: PRD US-1904 dan wireframe K01 layar 9 menyebut "Dasbor" dan "Invoice"; nama baru mengikuti keputusan user, penyelarasan PRD/wireframe bila diperlukan dilakukan orkestrator.
Keputusan lanjutan `[USER]` 2026-11-27 (Telegram): Q1 = A, Beranda Aktif tetap satu tombol utama "Undang rekan HR" (KEPUTUSAN-440); Q2 = A, tiga dokumen diunggah dalam satu halaman dengan "Langkah x dari 3" (KEPUTUSAN-441). Penyelarasan selesai: wireframe K01 layar 7-11 (menu Beranda, Pesanan, Peserta, Tagihan, Tim HR; layar 9 menjadi Beranda; layar 10 menjadi Tim HR) dan brief web korporat (KR-12, KR-13, J5); teks PRD US-1904 tidak memuat nama menu "Dasbor"/"Invoice" sehingga tidak berubah.

## Tech Debt

| ID | Isi | Asal | Target |
| --- | --- | --- | --- |
| TD-01 | Pemeriksa teks migrasi (T-23) tidak dapat menutup SQL dinamis: 7 bentuk lolos (dollar-quote, `chr()`, `concat()`, `format('%s')`, `E'\x..'`, `convert_from(decode())`). Usulan: (a) tolak `EXECUTE` di berkas migrasi kecuali templat §6.2 yang cocok persis (allowlist); (b) event trigger runtime yang menolak DISABLE/DROP trigger penjaga (KEPUTUSAN-158) sebagai kontrol utama; review DBA wajib atas setiap berkas migrasi tetap berlaku | Review keamanan F1-T14/T15 putaran 2 (2026-11-19), Medium non-blokir | Sebelum migrasi pertama yang memakai `EXECUTE` (paling lambat F1-T16–F1-T19) |
| TD-02 | Test flaky: commit merge F1-T12 sempat MERAH di `cargo-test`, ulang tanpa perubahan HIJAU; test penyebab belum teridentifikasi | Merge F1-T12 (2026-11-19) | Task berikutnya yang menyentuh test HTTP/Rust |
| TD-03 | Pelanggaran CHECK di `data_key` menaruh baris lengkap (termasuk `wrapped_dek`) di DETAIL galat PostgreSQL → log server dan galat ke klien. Usulan: validasi sebelum INSERT di aplikasi, masking galat sqlx, dan/atau `log_error_verbosity` untuk role aplikasi | Review keamanan F1-T17 (Low) | Task enkripsi/kunci data (EN-09) |
| TD-04 | `stored_file.purged_at`: pekerja privasi menghapus objek tetapi tidak punya UPDATE; pengisi `purged_at` `[TIDAK DIKETAHUI]`. Usulan CR teknis: `GRANT UPDATE (purged_at, updated_at, updated_by)` ke `rs_privacy_worker` + pengecualian T-08 | Review DBA F1-T17 | Sebelum task pekerja privasi (EN-23) |
| TD-05 | `rs_privacy_worker` dapat membaca `wrapped_dek` padahal tugasnya memusnahkan; pertimbangkan hak per kolom lewat CR/ADR | Review keamanan F1-T17 | Task kunci data |
| TD-06 | Kesalahan sebelumnya: commit merge F1-T12 merah sekali di `cargo-test`, dan merge F1-T06 merah di `migrasi-db`; penyebab kedua dipastikan: variabel `GIT_*` warisan hook pre-merge-commit. Langkah DB kini dibungkus `tanpa_env_git`; TD-02 kemungkinan sebab yang sama (sebelum F1-T06 cargo-test belum dibungkus) | Merge F1-T06 (2026-11-19) | Dipantau; TD-02 ditutup bila tidak terulang |
| TD-07 | CHECK amplop `*_enc` bergantung urutan evaluasi `AND`; bentuk aman: `x IS NULL OR (octet_length(x) >= 29 AND substring(x FROM 1 FOR 1) = '\x01'::bytea)`; perlu keputusan teknis untuk skema fisik §4.1 butir 5 + sesuaikan `uji_t20` | Review F1-T18 | Sebelum tabel `*_enc` berikutnya |
| TD-08 | FK komposit `(id, care_recipient_id)` untuk ivr→identity_document, `care_recipient.latest_verification_request_id`, registri→ivr (cegah kaitan lintas penerima); registri hanya untuk pemilik akun. Usulan KEPUTUSAN baru (pola KEPUTUSAN-142); dapat ditambah tanpa backfill | Review F1-T18 | Sebelum F1-T59 (keputusan verifikasi) |
| TD-09 | Kontradiksi DD §0.2 (FK komposit `note_data_key_id` ke `data_key (id, container_id)`) vs templat §4.6 + migrasi (FK tunggal); pilih satu | Review F1-T18 | Task dokumen orkestrator |
| TD-10 | DD A2.3/A2.4/A2.8 belum mencantumkan `anonymized_at` yang diwajibkan §0.2/§0.6/T-07 (sudah ada di migrasi); `actor_partner_id` STD-R di ivr_status_event ditunda sampai A5; `reason_code` milik modul platform belum dicatat di 04-arsitektur §3; CHECK pasangan `nik_bidx`/`nik_bidx_key_version` di care_recipient; aplikasi wajib memeriksa semua versi kunci blind index aktif saat rotasi (UP-40); penyimpanan penghitung batas NIK berlapis `[TIDAK DIKETAHUI]` | Review F1-T18 | Task dokumen + task aplikasi US-106/US-201 |
| TD-11 | Crate crypto (review F1-T26–T28, Low): `wrapped_dek` tidak terikat `data_key_id` lewat context Transit (crypto-shredding efektif hanya bila semua salinan termasuk backup/replika habis masa simpan selagi KEK versi itu masih bisa dekripsi — masukkan ke prosedur retensi); `generate_dek` tanpa cek pulang-pergi; `DataKey` yang di-clone hidup melewati TTL (dokumentasikan: jangan disimpan melewati satu use case); token OpenBao/JSON galat tidak di-zeroize; normalisasi NIK membuang semua tanda baca (konfirmasi validasi API); klien OpenBao belum TLS/autentikasi proses (AppRole/JWT) untuk staging/production | Review F1-T26–T28 | Task EN-15/EN-16 (staging) dan task retensi |
| TD-12 | Pemilik skema audit masih bisa `DISABLE TRIGGER USER` lalu DELETE, atau DETACH/DROP partisi (terbukti); trigger penolak hanya kontrol sisi aplikasi. Kontrol utama: kredensial pemilik dinamis OpenBao lewat bastion + orang kedua, jangkar WORM; event trigger KEPUTUSAN-158 dilacak sebagai item. F1-T25 wajib memuat metrik/peringatan "partisi masa depan < 6 bulan" (§6.2 butir 2) karena tanpa partisi DEFAULT relay berhenti 2030-01. Validasi rencana `fn_chain_head` (seq scan 38 partisi) di uji beban NFR-CAP-04. FK komposit consent_record (butir milik versi, care_recipient milik pasien) lewat CR | Review F1-T19/T21 | F1-T25, EN-25, task deploy |
| TD-13 | Arch-check `tx-control` bisa dilewati (`concat!("COM","MIT")`, `AssertSqlSafe(format!(..))`, komentar SQL sebelum COMMIT/ROLLBACK); lapis 2 (`TransactionEscaped`) hanya mendeteksi setelah data bisnis ter-commit. Perbaikan: buang komentar SQL sebelum pencocokan, tandai `AssertSqlSafe`/`concat!` di panggilan SQL di luar allowlist, lunakkan klaim `outbox.rs:12-16`. Plus catatan coding-standards §4.4: nilai dalam batas (Idr 12 digit, Int 6 digit, teks tanpa digit) tetap lolos — hanya kebijakan nama field + review yang menahan | Review audit putaran 2 (Medium/Low) | Sebelum modul pertama yang memakai `in_audited_tx` (US-101/US-201) |
| TD-14 | platform-jobs (review F1-T20): satu sewa untuk seluruh batch — lewati job bila `clock.now() >= locked_until` sebelum handler, atau tetapkan sewa > batch × durasi; belum ada heartbeat; selaraskan skema fisik §9.1 (`now()` server) dengan KEPUTUSAN-131 (Clock aplikasi) + sewa minimum jauh di atas skew NTP; kode payload sebaiknya daftar tetap per job_type (pola `[a-z0-9_]` masih bisa memuat nomor berawalan huruf); `sweep_expired` wajib dijadwalkan di deployment | Review F1-T20 | Sebelum job pertama yang memanggil pihak ketiga (notifikasi/email) |
| TD-15 | authn/authz (review F1-T32/T37): (Medium) `finish()` mengembalikan `axum::Router` biasa — rute yang ditambah sesudahnya tanpa penegak (terbukti); ganti newtype tertutup + arch-check melarang `.route/.merge/.nest/.fallback` di `apps/*/src` setelah finish; (Medium) ekstraktor `AuthContext` di endpoint Public memvalidasi token tanpa resolver akun (KU-10) — hanya terima konteks dari middleware; (Low) selisih timing 404 milik-orang-lain vs tidak-ada (log sinkron) — log non-blocking + test timing release di F1-T38; (Low) arch-check penjaga fitur test (`authn/testing`, `crypto/test-keys`) tidak ikut build produksi; (Info) OPTIONS 404 vs preflight CORS; klien JWKS belum TLS | Review F1-T32/T37 | F1-T38 (paling lambat sebelum endpoint pasien pertama) |
| TD-16 | `scripts/keycloak-check.sh ephemeral` (150 pemeriksaan, ±4,5 menit) belum tersambung ke verifikasi otomatis. Usulan: job CI wajib terpisah (bukan pre-commit lokal) dengan izin user bila menambah langkah skrip verifikasi; juga dijalankan untuk staging/production sebagai deteksi drift. Temuan terbuka Keycloak: struktur halaman akun nonaktif (F1-T35, KU-10 a), reset langsung menerbitkan kode otorisasi vs US-103 AC4d (F1-T74), blacklist password (NFR-SEC-03, KEPUTUSAN-188), `sslRequired` production, alamat pengirim & skema redirect mobile final | Review F1-T31 | F1-T35, F1-T36, F1-T74, EN-15 |
| TD-17 | **Berkas (review F1-T29, Medium, prioritas tinggi — data KTP):** objek mentah yatim di bucket identitas: URL POST/PUT dapat dipakai ulang sampai TTL setelah slot dikonsumsi; unggahan di sela list/delete versi; crash setelah put hasil kode ulang; `ValidatedUpload` dibuang tanpa `discard_validated`. Perbaikan: slot sebagai tombstone (`consumed_at`) sampai `expires_at` + jeda lalu pembersih menghapus semua versi kunci asli; prefiks `incoming/` vs `files/`; rekonsiliasi berkala bucket vs `stored_file`; `#[must_use]`/penjaga Drop. Juga (Low) memori dekoder ±0,9 GB per validasi: turunkan batas konkurensi maks 16 atau anggaran piksel×2, permit sebelum `get_object`, timeout antrean; TLS ke NEO (klien server baru `http://`, staging gagal start); field POST tambahan di NEO belum diuji | Review F1-T29 | Sebelum F1-T60 (unggah KTP pasien) |
| TD-18 | Audit (review F1-T24/T25): AnchorStore NEO Object Lock wajib sebelum staging; verifikasi mulai dari titik `audit_archive_manifest` setelah partisi di-detach (§6.5); runbook event racun di outbox (fail-closed menahan antrean); bedakan alarm jangkar "hanya DB" vs "WORM" | Review F1-T24/T25 | EN-15/EN-16, task arsip audit |
| TD-19 | CR-024 di kode: ganti `markup` → `margin` di `crates/audit/src/policy.rs`, `crates/audit/tests/forbidden_fields.rs`, dan `db/test/kelas-app.sql:13` (`markup_rule` → `margin_rule`) | CR-024 | Task developer berikutnya yang menyentuh audit/DB test |
| TD-20 | Berkas (review F1-T30/TD-17): (Medium) pembersih slot dan rekonsiliasi bucket belum dijadwalkan — subcommand `worker` harus menjalankan `enqueue_file_maintenance` berkala (wajib sebelum F1-T60); (Medium) `FileAccess::Owner` dibatasi per `UploadPurpose` — tujuan baru wajib E-12 (unduh resep/hasil lab) atau E-05 (pemilik akun membuka data anggota keluarga, AB-71); (Low) subjek E-05 Staff diisi pemanggil — cocokkan dengan pemilik berkas; batasi durasi transaksi pemanggil < jeda pembersih 15 menit; rekonsiliasi bucket besar (batas 1000 halaman dari awal); oracle waktu E-05 gagal (retry) | Review F1-T30/TD-17 | Sebelum F1-T60 |
| TD-21 | Batas laju per akun untuk lupa password dan kirim ulang verifikasi di Keycloak/Caddy (ADR-019; review F1-T36) belum ada; test flaky `observability::log_redaction::app_level_filter` (sekali gagal, 6× lulus) dan `audit relay_pg` (sekali gagal) — selidiki; `keycloak-check ephemeral` dijalankan ulang setelah perbaikan relay | Review F1-T36, laporan F1-T38 | Task berikutnya (F1-T55/keamanan login) |
| TD-22 | **SEBAGIAN SELESAI 2026-11-27 (KEPUTUSAN-400/401, commit de2dedb):** kode IP klien asli dari proxy tepercaya selesai: env `TRUSTED_PROXY_CIDRS` (tidak diisi = peer TCP; nilai tidak valid = start gagal), middleware IP klien `app.rs` membaca `X-Forwarded-For` hanya bila peer TCP termasuk daftar dan mengambil alamat paling kanan yang bukan proxy tepercaya; hasilnya `authz::ClientIp` yang dipakai batas laju per sumber (pendaftaran perusahaan, teks persetujuan HR/pasien, konten/gambar publik), `source_ip` audit, dan E-05 (`apps/rs-backend/src/http_support.rs` `TrustedProxies`; test `apps/rs-backend/tests/http_trusted_proxy.rs`). `ClientIp` selalu terisi saat dilayani lewat `serve_with_peer_addr` (proses `api`), jadi E-05 tidak lagi kosong. **Sisa (di luar kode backend):** (a) isi `TRUSTED_PROXY_CIDRS` = alamat Caddy `/32` di skrip deploy staging/production (production kini gagal start tanpa nilai ini, KEPUTUSAN-403) (checklist deploy docs/05-stack.md §5.3); (b) IP asli sampai ke Caddy bila ada LB NEO di depannya (PT-39; LB L4 tanpa PROXY protocol membuat Caddy melihat alamat LB); (c) BFF Next.js belum meneruskan IP klien: dipindah ke TD-58 (syarat reviewer di sana); (d) usulan fail-closed endpoint Require tanpa `ClientIp` → 500 belum diterapkan (tidak terjadi di proses `api`). Riwayat: E-05 hasil gagal tidak akan tertulis di production bila `authz::ClientIp` kosong; prasyarat keras pembukaan `/daftar` korporat di production (ADR-031, review T1 F1-T103 B3) | Review F1-T38, F1-T41/T42, F1-T103 | Sisa (a) sebelum `/daftar` dibuka di production; (b) bersama PT-39; (c) sebelum endpoint pasien lewat BFF di production |
| TD-23 | Situs (review F1-T89): mode `SITUS_KONTEN_SUMBER=contoh` tidak dijaga di production — tolak bila `NODE_ENV=production` tanpa flag eksplisit kedua (periksa-build memakai mode ini); image production wajib `NODE_ENV=production`. Tema login (review F1-T35, Low): pesan galat non-kredensial (sesi kedaluwarsa, cookie) tertutup pesan login; `&laquo;` tampil literal; `invalidEmailMessage` menyesatkan; pemaketan tema JAR untuk production; uji visual/pembaca layar manual. Event Keycloak (F1-T34): DD untuk `idp_event_cursor`/`idp_event_receipt`; nilai `actor_type` anonim (E-02 akun tak dikenal dicatat aktor system tanpa ID); IP penuh di `source_ip` sesuai PRD §7.2 | Review F1-T89, F1-T35, laporan F1-T34 | F1-T96/97, task deploy, task dokumen |
| TD-24 | Sesi (review F1-T33): publish revocation ke Valkey lewat platform-jobs (tahan restart; kini proses mati di antara commit dan publish → sesi lolos sampai token kedaluwarsa); konfirmasi dokumentasi PostgreSQL bahwa `ON CONFLICT (kolom)` butuh SELECT atas kolom target (dipakai KEPUTUSAN-284 dan `insert_failed_access`, SRC-470) | Review F1-T33 | Sebelum endpoint pasien pertama |
| TD-25 | CMS (review F1-T91–T94/T111): batas laju publik di memori per proses dan memakai IP peer (di balik Caddy/BFF = IP proxy) — pindah ke Caddy/Valkey dengan IP klien asli (TD-22; IP klien asli dari proxy tepercaya sudah di kode, KEPUTUSAN-400) sebelum rilis; daftar publik tanpa `body` penuh; header `Retry-After` pada 429; renderer situs/pratinjau wajib allowlist Markdown yang sama (F1-T95/T96) | Review CMS | Sebelum rilis situs |
| TD-26 | Keycloak (review F1-T65, High): klien `rs-backend-admin-manager` memegang `manage-users` = bisa reset password/hapus OTP semua admin termasuk Super Admin (Keycloak 26.8.2: admin role mengalahkan permission). **Gerbang**: sebelum klien ini dipakai proses `api` — batasi dengan FGAP v2 per grup atau pisahkan ke worker khusus + alert admin event reset-password/credential. Juga: ikat accessor–wrapping token (`sys/wrapping/lookup` creation_path), audit persetujuan terpakai saat kirim undangan gagal, runbook pemulihan bila COMMIT tak pasti saat bootstrap | Review F1-T65 | Sebelum endpoint kelola admin dipasang di api |
| TD-27 | US-103 AC4d belum lengkap (review F1-T74): email pemberitahuan "password diatur ulang" dan pengakhiran semua sesi setelah reset (event Keycloak → pencabutan sesi F1-T33/F1-T34). AC4d jangan ditandai selesai sebelum ini; kontrol kompensasi `acr`=1 tanpa `amr` (token hanya lewat flow browser ber-OTP, direct grant mati) dicatat di README Keycloak/ADR-011 | Review F1-T72–T74 | Sebelum penutupan Fase 1 (F1-T70/T80) |
| TD-28 | Batas NIK (review F1-T60): celah jendela tetap (2+2 penolakan di batas jendela); penghitung IP per alamat — IPv6 sebaiknya per /64; cek hold di luar transaksi (burst paralel, dibatasi slot unggah); uq (patient_id, nik_bidx) tanpa versi kunci → setelah rotasi duplikat lintas versi lolos sampai re-index (cek `lookup_candidates` atau job re-index); aturan "registri NIK hanya pemilik akun" belum di DB; FK tunggal lama berlebih (contract) | Review F1-T60 | Sebelum rotasi kunci pertama / Fase 2 |
| TD-29 | OpenBao untuk proses `api` (review F1-T61): `OPENBAO_TOKEN` statis tanpa perpanjangan (token kedaluwarsa → Transit gagal) dan belum ada policy minimum di repo (`transit/datakey/plaintext/rs-dek-kek`, `transit/decrypt/rs-dek-kek`, read KV blind index) — menunggu metode auth proses ADR-025; nilai production cache DEK (TTL, kapasitas) `[TIDAK DIKETAHUI]`; Schemathesis hanya melihat 401 di `/patient/v1` (skenario bertoken dicakup test integrasi); `docker logs` saat OpenBao dev gagal start bisa mencetak token root dev sekali pakai | Review F1-T61 | Sebelum staging |
| TD-30 | Keycloak (review F1-T50, High): klien `rs-backend-registrar` (realm pasien) memegang `manage-users` di proses `api` — bila api diretas: reset password, hapus OTP, ubah email, hapus akun semua pasien. Peran lebih sempit tidak ada di Keycloak 26.8.2 (SRC-643). **Gerbang sebelum production**: alarm admin event Keycloak dari klien registrar selain create/delete/send-verify-email; rahasia hanya di OpenBao + rotasi; kaji FGAP v2 atau pemisahan ke worker khusus. Juga: TD-22 (IP klien asli) kini wajib sebelum deploy karena batas laju pendaftaran per IP; ukur ulang waktu respons enumerasi dengan Keycloak sungguhan (test kini memakai tiruan) | Review F1-T49–T51 | Sebelum production |
| TD-31 | Verifikator (review F1-T62): belum ada jalur production untuk mengisi NIK staf (`with_staff_nik_index` hanya dipakai test; rute US-208 kelola admin belum dipasang karena gerbang TD-26) → semua Verifikator ditolak `SelfDecisionGuard` (gagal-tertutup) sehingga US-201 belum bisa dipakai. Pemilik: task endpoint kelola admin US-208 + F1-T64 (panel). Risiko bawaan KEPUTUSAN-294 opsi A: akun pasien Verifikator yang belum pernah mengajukan NIK sendiri tidak bisa dikenali | Review F1-T62 | Sebelum F1-T70 (demo E2E) |
| TD-32 | Pemeriksa lisensi npm otomatis di `web/` belum ada (review F1-T96): lisensi di luar daftar izin (KEPUTUSAN-136/297) tidak tertangkap verifikasi; tambah langkah pemeriksa lisensi dependensi pnpm (runtime vs dev dibedakan) — memerlukan perubahan skrip verifikasi (menambah, bukan melonggarkan) | Review F1-T96 | Sebelum penutupan Fase 1 |
| TD-33 | Sesi setelah reset (review TD-27, Medium/Low): pencabutan sesi hanya dipicu event LOGOUT dari Keycloak; bila pemegang tautan reset membuang `logout-sessions=on` dari form, sesi lain tetap hidup sampai token refresh habis. Tambah pertahanan sisi server: cabut semua sesi akun berdasarkan E-04 password (not-before per akun = waktu event) atau logout Admin REST saat klien berhak minimal tersedia (TD-26/TD-30). Jendela AC4d bergantung pada worker event (poll 60 s; alarm `idp_event_pull_delayed` 5 menit) | Review TD-27 | Sebelum production |
| TD-34 | Harness isolasi korporat (review F1-T100, Low): sidik data penulisan hanya mencakup tabel `app` ber-`corporate_client_id` + `corporate_client`; penulisan lintas tenant ke tabel turunan tanpa kolom itu atau ke `app.account` HR perusahaan lain tidak terdeteksi — perluas sidik saat F1-T101/F1-T107 menambah tabel/endpoint; DD belum menyebut member area korporat sebagai `source_component` tersendiri (`corporate_portal`) | Review F1-T100 | Bersama F1-T101/F1-T107 |
| TD-35 | Foto KTP via URL bertanda tangan (review F1-T64, Low): pastikan object storage mengirim `Cache-Control: no-store`/`private` untuk URL unduh dokumen identitas (ADR-018) agar tidak tersimpan di cache HTTP browser; isi `BACKOFFICE_DOCUMENT_ORIGIN` di konfigurasi deploy | Review F1-T64 | Sebelum staging |
| TD-36 | Field audit (review F1-T69): before/after "bila ada nilai yang berubah" belum ditegakkan untuk E-08 refund, E-09, E-13, E-15 — wajibkan saat pemancarnya dibuat; pengecualian ID objek dipersempit ke E-01..E-04 gagal; inventaris pemancar bisa dilewati lewat alias/helper lintas berkas; `record_invite_failure` bisa mencatat dobel bila balasan commit hilang dan jejak audit tidak memuat kegagalan akhir setelah 3 percobaan; doc `admin_service.rs` masih menyebut urutan lama; ekspor `audit_write_failures_total` ke metrik menunggu agen metrik | Review F1-T69 | Saat pemancar E-08/E-09/E-13/E-15 dibuat |
| TD-37 | Verifikasi perusahaan (review F1-T104, Low): arch-check membatasi pemakaian `CORPORATE_DOCUMENT_OBJECT_TYPE` ke `platform/service.rs`; job rekonsiliasi status `corporate_client` vs keputusan terakhir (ADR-008 butir 4, opsional Master); test bahwa penolakan 401/403 tercatat di security log (US-1902 AC5); 3 dari 4 kode alasan penolakan perusahaan masih draf — konfirmasi user sebelum go-live; bucket `license_document` untuk dokumen perusahaan bucket dan kategori dipakai F1-T102 tetapi retensi dan pemusnahannya belum ditetapkan (TD-49); PRD US-1902 AC5 menyebut 404 untuk HR, implementasi 401 (penolakan realm seragam di `/backoffice`) — selaraskan PRD lewat CR-026 | Review F1-T104 | Sebelum F1-T110 |
| TD-38 | Profil pasien (review F1-T56): asumsi yang perlu dikonfirmasi user/legal — usia minimum 17 tahun (nilai uji; PB-22/L-08), maksimal 20 alamat aktif (tanpa sumber), T-42: alamat pertama otomatis utama, alamat utama yang dihapus diganti alamat aktif tertua, alamat utama tidak bisa dijadikan bukan-utama; perjelas TC-US-105-1 ("nama field; before/after hanya untuk nilai bertipe"); HR yang dinonaktifkan masih bisa login Keycloak sampai worker ADR-030 aktif (review F1-T107) — gerbang sebelum production | Review F1-T56, F1-T107 | Sebelum penutupan Fase 1 |
| TD-39 | Backoffice F1-T106 (sisa): UI "buat perusahaan" (US-1903) dan daftar perusahaan serta anggota HR (US-1916 K1) belum dibuat karena endpoint backoffice-nya belum ada di `openapi/` (F1-T105 menunggu ADR-030); panel verifikasi menandai fitur ini belum tersedia. Juga: `openapi/backoffice.json` memakai nama skema sama (`QueueItemDto`, `DecisionDto`, dll.) untuk modul identitas dan perusahaan sehingga tipe hasil generator untuk antrean/keputusan perusahaan keliru; `web/backoffice/src/lib/perusahaan/model.ts` menulis dua tipe lokal sampai spesifikasi diberi nama unik (di luar kepemilikan web) | F1-T106 | Setelah F1-T105 |
| TD-F1T49-A | Syarat F1-T50 (review F1-T49 catatan 4, US-1501 AC2): test integrasi yang menulis event E-13 dari `privacy::api::record_granted_consent` ke outbox di transaksi yang sama dengan `consent_record` dan membuktikan pembatalan seluruh transaksi bila penulisan audit gagal. Kini test F1-T49 hanya memeriksa kelengkapan payload event | Pemanggil pertama (pendaftaran pasien, F1-T50 atau penggantinya, ADR-030) | F1-T49 (22be15c) |
| TD-F1T49-B | Syarat F1-T50 (review F1-T49 catatan 5): probe `/health/ready` production menjalankan kueri teks persetujuan per panggilan (`ConsentTextReadiness`); batasi probe ke jaringan internal atau cache hasilnya sebelum production | Deploy production | F1-T49 (22be15c) |
| TD-39 | Backoffice F1-T106 (sisa): UI "buat perusahaan" (US-1903) sudah dibuat; daftar perusahaan serta anggota HR (US-1916 K1) belum dibuat karena endpoint daftarnya belum ada di `openapi/backoffice.json`; panel verifikasi menandai ini belum tersedia. Juga: `openapi/backoffice.json` memakai nama skema sama (`QueueItemDto`, `DecisionDto`, dll.) untuk modul identitas dan perusahaan sehingga tipe hasil generator untuk antrean/keputusan perusahaan keliru; `web/backoffice/src/lib/perusahaan/model.ts` menulis dua tipe lokal sampai spesifikasi diberi nama unik (di luar kepemilikan web) | F1-T106 | Setelah F1-T105 |
| TD-40 | Undangan HR (F1-T107 sisa, Medium): tidak ada batas jumlah atau laju undangan per HR/perusahaan (AC tidak menyebutnya); HR dapat memicu email aktivasi ke banyak alamat berbeda (penyalahgunaan sebagai pengirim email). Tetapkan batas (per HR, per perusahaan, per alamat; KEPUTUSAN-192/US-101 AC2b sebagai pola) bersama "pengerasan undangan" K2; juga pemberitahuan "Anda sudah punya akun" dan penyeragaman waktu respons (KEPUTUSAN-329) | Review F1-T107 sisa | Sebelum production |
| TD-41 | Worker IdP (`idp_worker.rs`, review F1-T107 sisa, Low): saat penyediaan undangan HR gagal permanen, akun dihapus lunak tetapi baris `corporate_member` tetap aktif (yatim); cabut keanggotaan (hapus lunak dengan alasan) di transaksi yang sama lewat `corporate::api` | Review F1-T107 sisa | Sebelum penutupan Fase 1 |
| TD-42 | Pendaftaran perusahaan mandiri (F1-T101, Medium): bila penyediaan HR gagal permanen (`idp_worker.rs`, akun dihapus lunak), perusahaan `pending_verification` dan keanggotaan HR hasil pendaftaran mandiri tertinggal tanpa HR dan NIB-nya tetap terkunci (jalur asinkron AC5; seperti TD-41). Tambahkan di transaksi hasil worker: bila tujuan niat `hr_self_registration` dan akun ditinggalkan, hapus lunak keanggotaan dan perusahaan yang belum punya HR lain lewat `corporate::api`, dengan alasan penyediaan gagal | Task F1-T101 | Sebelum penutupan Fase 1 (bersama TD-41) |
| TD-43 | **SELESAI 2026-11-26 oleh keputusan user Q8=A dan Q9=A (KEPUTUSAN-343, F1-T102): dokumen diunggah HR setelah aktivasi akun lewat sesi korporat, tanpa slot anonim; antrean Verifikator hanya memuat pengajuan dengan ketiga dokumen; PRD US-1901 AC1 diselaraskan lewat CR-031.** Pendaftaran perusahaan mandiri (F1-T101/F1-T102): PRD US-1901 AC1 meminta unggah dokumen NIB, NPWP, dan bukti kewenangan saat mengirim, padahal akun HR baru ada setelah worker membuat pengguna dan HR membuka tautan aktivasi (ADR-030). Perusahaan masuk antrean Verifikator tanpa dokumen. F1-T102 perlu memutuskan: unggah dokumen setelah HR aktivasi (AC6 menyebut HR hanya melihat status) atau slot unggah tanpa akun; antrean Verifikator memfilter pengajuan tanpa dokumen lengkap. Perlu keputusan user (G-27, bentuk bukti kewenangan) | Task F1-T101 | Sebelum F1-T102 |
| TD-44 | Pendaftaran perusahaan mandiri (F1-T101, Low): (a) probe `/health/ready` `ConsentTextReadiness` hanya memeriksa teks pasien; teks HR draf di production ditolak di jalur layanan (503) tetapi tidak membuat proses "tidak siap"; (b) batas laju dalam proses (per instans, "sumber" = BFF di balik proxy, PT-39): satu klien jahat yang menghabiskan anggaran per sumber memblokir semua pendaftar sah (DoS; review keamanan F1-T101 Medium-3). Wajib sebelum production: IP klien asli dari proxy tepercaya (PT-39; kode selesai KEPUTUSAN-400, tinggal `TRUSTED_PROXY_CIDRS` di deploy) atau batas per IP di Caddy; belum ada batas per email/NIB di Valkey. Anggaran GET teks persetujuan sudah dipisah dari POST (commit perbaikan review); (c) `PRD` US-1901 AC7 menyebut "ditolak dengan pesan generik", implementasi jawaban seragam (KEPUTUSAN-324); selaraskan PRD lewat CR-026; (d) `4090107`–`4090109` dicadangkan, belum dipakai | Task F1-T101 | Sebelum production |
| TD-45 | **SELESAI 2026-11-26 (CR-035, keputusan user Q10 = B2):** NIB terdaftar diproses persis sama dengan NIB baru (perusahaan Menunggu verifikasi + akun HR + email aktivasi yang sama), sehingga ada-tidaknya email tidak lagi membuka NIB; diuji `corporate_registration_registered_nib_is_processed_exactly_like_a_new_one`. Sisa pemantauan p99 waktu respons terhadap 900 ms tetap berlaku (kedua jalur kini melakukan penulisan yang sama). Teks lama: Pendaftaran perusahaan mandiri (review keamanan F1-T101 Medium-1 dan Low-5): **menunggu keputusan user**. Jawaban `202` seragam, tetapi ada-tidaknya email aktivasi membuka oracle: pendaftar tahu NIB sudah terdaftar bila email tidak tiba (Q7=A hanya menerima oracle email HR, bukan NIB). Pesan respons tidak lagi menjelaskan email yang tidak tiba. Opsi perbaikan: kirim email arahan ke alamat pendaftar untuk jalur NIB terdaftar setelah modul Notifikasi ada (isi tanpa membuka perusahaan lain), sehingga email selalu tiba. Juga: penyamaan waktu 900 ms hanya batas bawah; pantau p99 waktu respons jalur baru terhadap 900 ms (jalur baru melakukan lebih banyak penulisan daripada jalur terdaftar; bila p99 melewati 900 ms, selisih menjadi pembeda) | Review F1-T101 | Sebelum production |
| TD-46 | **Sebagian selesai 2026-11-26 (CR-035):** NIB tidak lagi terkunci oleh pengajuan Menunggu verifikasi/Ditolak (keunikan hanya di antara perusahaan Aktif, `uqp_corporate_client_nib_active`), sehingga squatting tidak menghalangi perusahaan sah mendaftar atau dibuatkan; Verifikator melihat penanda NIB dipakai. **Sisa:** kontrol anti-otomasi (CAPTCHA atau batas per email/NIB di Valkey) dan pembersihan pengajuan yatim/terbengkalai (N hari `[TIDAK DIKETAHUI]`), serta aturan reaktivasi perusahaan Nonaktif yang NIB-nya sudah dipakai perusahaan Aktif baru (US-1916, pertanyaan P-Q10-2). Teks lama: Pendaftaran perusahaan mandiri (review keamanan F1-T101 Medium-2): squatting NIB dan penguncian NIB. Pendaftaran tanpa verifikasi kepemilikan NIB mengunci NIB (`uqp_corporate_client_nib`, status apa pun, termasuk `rejected`) selamanya dan perusahaan sah tidak dapat mendaftar. Perlu sebelum production: prosedur pembebasan NIB (hapus lunak beralasan saat penolakan final, atau saat HR tidak mengaktivasi dalam N hari; N `[TIDAK DIKETAHUI]`) dan kontrol anti-otomasi (CAPTCHA atau batas per email/NIB di Valkey). Berkaitan dengan TD-42 (perusahaan yatim saat penyediaan gagal) Catatan F1-T102: antrean `ready` menyembunyikan pengajuan tanpa dokumen lengkap (memperkuat squatting NIB). | Review F1-T101 | Sebelum production |
| TD-47 | **SELESAI 2026-11-26 (F1-T102b, commit 067dcc3, KEPUTUSAN-360/361), kecuali koreksi data perusahaan saat pengajuan ulang (pertanyaan user P-T102b-1, PRD tidak mengatur):** (a) `POST /v1/corporate/verification-resubmissions` (tombol eksplisit, hanya dari Ditolak, dokumen lengkap wajib `4090115`, putaran +1, E-10 `corporate_verification_resubmitted`); (b) objek `company` di `GET /v1/corporate/documents`. Teks lama: Pengajuan ulang dan profil HR (F1-T102, Medium): (a) US-1902 AC3 "dapat memperbaiki serta mengajukan ulang": belum ada endpoint HR yang menaikkan `submission_round` dan mengembalikan status Ditolak ke Menunggu verifikasi (F1-T102 hanya mengizinkan unggah dan ganti dokumen saat Ditolak); perlu keputusan kapan pengajuan ulang terjadi (tombol eksplisit setelah dokumen lengkap, atau otomatis) dan audit E-10; (b) US-1901 AC6: HR hanya melihat status verifikasi, alasan penolakan, dan **profil perusahaan**, tetapi `GET /v1/corporate/documents` hanya memuat status, alasan, dan kelengkapan dokumen, belum profil (nama, NIB, alamat, telepon) **Keputusan user Q14=A (2026-11-26): (a) endpoint pengajuan ulang dan (b) profil perusahaan di `GET /v1/corporate/documents` dikerjakan sebagai task kecil terpisah setelah F1-T102 merge, tidak di F1-T102.** | Task F1-T102 | F1-T103 atau task baru sebelum UI ditolak |
| TD-48 | Penetralan PDF (F1-T102, High untuk go-live): (a) biner qpdf 12.4.2 belum terpasang di mesin dev/CI/image api, sehingga test dengan qpdf asli (`corporate_documents_pdf_qpdf.rs`) bertanda `#[ignore]` dan belum pernah dijalankan; unggahan PDF bergantung pada qpdf (`PDF_INSPECTOR_QPDF_PATH`), tanpa itu slot PDF ditolak 415; pemasangan wajib memverifikasi sha256 rilis (SRC-862) dan menjalankan test itu sebelum PDF dipakai di production; (b) CDR/render ulang dan antivirus belum ada (PDF lolos disimpan apa adanya, sisa data setelah `%%EOF` tetap); (c) PDF bertanda tangan digital (memuat `/AcroForm`) tetap ditolak: **keputusan user Q12=A (2026-11-26)**, pengguna mengunggah pindaian atau foto; (d) pembatasan sumber daya proses qpdf: batas waktu 10 detik, keluaran 32 MiB, dan `ulimit -v`/`-t` lewat pembungkus `sh` (upaya terbaik; dikerjakan tanpa `unsafe` karena `pre_exec`/`setrlimit` langsung memerlukan `unsafe` yang dilarang standar kode; platform tanpa `ulimit -v` melewatinya diam-diam); tambahkan batas cgroup di deploy; (e) daftar izin `/S` menolak PDF bertag dengan tipe struktur khusus (RoleMap) dan kunci `/S` ganda dalam satu kamus tidak terlihat di JSON qpdf (perilaku qpdf `[TIDAK DIKETAHUI]`, diuji di test qpdf yang `#[ignore]`); (f) izin konkurensi pemeriksa dan klaim slot atomik sudah ada, tetapi permintaan yang dibatalkan meninggalkan klaim sampai slot kedaluwarsa (klien membuat slot baru) | Task F1-T102 | Task CI terpisah setelah F1-T102 (keputusan user Q11=B: memasang paket qpdf Ubuntu di CI dan menjalankan `corporate_documents_pdf_qpdf.rs` dengan `--ignored`; tetap `#[ignore]` di lokal), task deploy |
| TD-49 | Dokumen perusahaan (F1-T102, Medium): (a) `stored_file` dokumen yang diganti atau perusahaan yang dihapus lunak belum dimusnahkan (objek tetap di bucket `license_document`); pekerja privasi dan kategori retensi dokumen perusahaan `[TIDAK DIKETAHUI]`; (b) wadah retensi `container_type` NULL karena DD §0.7 belum memuat dokumen perusahaan; (c) tanpa enkripsi lapis aplikasi (hanya lapis penyedia, ADR-018 butir 5), padahal bukti kewenangan dapat memuat data pribadi pengurus (KTP, tanda tangan); perlu keputusan legal/Petugas PDP; (d) bucket `license_document` dipakai bersama dokumen izin rekanan (usulan teknis F1-T104); (e) berkas milik perusahaan tidak boleh ikut dimusnahkan atau diekspor lewat `stored_file.owner_account_id` (akun HR pengunggah) saat akun HR dihapus atau salinan data pribadinya diekspor (US-1604): `owner_account_id` bukan subjek data atas dokumen perusahaan; jalur pekerja privasi dan ekspor wajib mengecualikan kategori `corporate_document`. **GERBANG sebelum production: (a) dan (c) harus diputuskan dan dikerjakan sebelum dokumen perusahaan dipakai di production** (f) **Risiko diterima (review DBA F1-T102):** pembersih menahan baris slot sampai `created_at` + 1 jam demi kuota, sehingga objek mentah `incoming/` di semua bucket (termasuk foto KTP ber-EXIF, US-106) kini tertahan sampai itu (sebelumnya `expires_at` + 15 menit); alternatif masa depan: hapus objek pada `expires_at` + jeda dan beri penanda "objek sudah dibersihkan" di baris slot (migrasi expand), lalu tahan baris saja untuk kuota. | Task F1-T102 | Fase 2 privasi, update ADR-013/ADR-018 |
| TD-50 | Keputusan perusahaan terikat versi dokumen (F1-T102 review): token `documents_version` (SHA-256 atas putaran pengajuan dan ID dokumen aktif terurut, sejak KEPUTUSAN-415) dicek di bawah kunci baris, tetapi ID/token dokumen yang diputuskan **tidak disimpan** di `corporate_verification_decision` maupun payload audit E-10; bukti dokumen mana yang dilihat saat keputusan hanya hilang setelah dokumen diganti lagi. Usulan: kolom `documents_version` (atau daftar ID dokumen) di baris keputusan (migrasi expand, Transaksi append-only) dan di E-10; ubah skema keputusan sekarang dinilai di luar ruang lingkup | Review F1-T102 | Sebelum production korporat / review DBA berikutnya |
| TD-51 | Jalur masa depan yang menulis `corporate_client.status = 'active'` atau mengosongkan `deleted_at` (reaktivasi perusahaan Nonaktif US-1916, pemulihan hapus lunak) belum ada. Kriteria wajib saat dibuat (review T1 CR-035, DBA 2 + keamanan L-3): (a) kunci baris perusahaan (`FOR UPDATE`) di transaksi yang sama; (b) cek perusahaan Aktif lain ber-NIB sama (`repo::other_active_client_with_nib`) sebelum menulis; (c) petakan 23505 `uqp_corporate_client_nib_active` (`repo::is_active_nib_conflict`) ke 409 dengan pesan umum dan event gagal, bukan 500; (d) test negatif balapan seperti `corporate_nib_shared.rs`. Aturan bisnis: `[USER]` Q22 = A (2026-11-27): reaktivasi **ditolak** bila NIB sudah dipakai perusahaan Aktif lain, Verifikator menyelesaikannya manual; kode respons ditetapkan di task reaktivasi; DD A20 Rincian K1 memuat catatan yang sama | Review T1 CR-035 | Task reaktivasi US-1916 atau pemulihan hapus lunak perusahaan |
| TD-52 | Tenant audit akun HR terikat waktu event (F1-T109 review DBA, Medium bila perpindahan dibuka): `corporate::repo::hr_account_tenant` memilih keanggotaan aktif atau terakhir tanpa melihat waktu event, sah hanya selama invarian KEPUTUSAN-387 (satu akun HR hanya pernah punya satu keanggotaan; dijaga test `corporate_hr_invite.rs::a_deactivated_hr_cannot_gain_a_second_membership_through_an_invitation`). Bila perpindahan perusahaan atau pengaktifan ulang keanggotaan diaktifkan, tenant wajib dipilih per waktu event: `created_at <= $2 AND (deleted_at IS NULL OR deleted_at >= $2)` (rumus DBA; `$2` = waktu event Keycloak atau waktu niat IdP), dengan index yang sesuai dan test event sebelum/sesudah perpindahan | Review T1 F1-T109 (DBA butir 2) | Task yang membuka perpindahan keanggotaan HR atau pengaktifan ulang HR |
| TD-53 | NIB dan NPWP perusahaan dikirim mentah ke browser HR (review T1 F1-T103, Low): `GET /v1/corporate/documents` (objek `company`, juga respons unggah dan pengajuan ulang) memuat `nib` dan `npwp` lengkap, sedangkan layar `/verifikasi` hanya menampilkan versi termasking (`termasking`, DG §6, brief S4). NPWP berklasifikasi Terbatas (DD A20). Perbaikan: backend (atau BFF korporat) mengirim nilai termasking (misalnya `nib_last4`, `npwp_last4`) untuk layar status, dan nilai lengkap hanya bila memang dibutuhkan; perubahan kontrak `openapi/korporat.json` + klien web. Di luar kepemilikan F1-T103 (`apps/rs-backend/**`) | Review T1 F1-T103 | Sebelum production korporat |
| TD-54 | CI test-verify (CR-029, review N-5, Medium untuk keandalan CI): (a) **flake `web-test` pada run CI ketiga**: kelompok `cache-ci` merah (job 139937481027, run 45828121248, commit d10d744) karena `web-test` GAGAL di run prima `verify.sh`; ringkasan galat memuat hanya 40 baris terakhir yang berisi satu uji lulus, jadi uji yang gagal tidak terlihat; run keempat (45844577354) hijau seluruhnya tanpa perubahan kode. **Penyebab [TIDAK DIKETAHUI]**. Pemicu tindak lanjut: bila `web-test` merah lagi pada run CI mana pun tanpa perubahan kode di web, jadikan task penelusuran (usulan: langkah CI menyimpan log lengkap `web-test` sebagai artefak, lihat TD-55); (b) margin waktu rangkaian cache: run keempat sekitar 22% (terpanjang `cache-batal` 46,6 dan `cache-isi` 46,3 menit, SRC-957), run kelima (45949956430, SRC-958) hanya sekitar 15% (`cache-penuh` 51,3, `cache-batal` 50,5, `cache-isi` 47,1 menit; waktu per job bervariasi antar run), dirujuk ke KEPUTUSAN-342; **ambang tindakan**: job terpanjang melewati 54 menit (margin kurang dari 10%) pada run mana pun: bila ada kelompok melebihi 60 menit, kelompok diperkecil lagi atau batas dinaikkan lewat keputusan user | Review keamanan+kode CR-029 (N-5), SRC-956, SRC-957 | Saat `web-test` flake terulang atau waktu kelompok melewati 54 menit |
| TD-55 | Langkah `web-build` di `scripts/verify.sh` gagal dua kali di dalam git hook saat merge (2026-11-27, F1-T102b dan F1-T103), tetapi lulus saat `verify.sh` dijalankan sendiri dan lulus saat hook diulang; **log galat tidak tersimpan**, sehingga penyebabnya [TIDAK DIKETAHUI]. Pengamatan orkestrator. Dugaan (belum diverifikasi): `web/*/scripts/periksa-build.mjs` menunggu server standalone paling lama 10 detik dan terlampaui saat mesin sibuk (beberapa agen berjalan paralel). **Usulan, belum dikerjakan dan butuh persetujuan user karena menyentuh `scripts/verify.sh` (modul kritis T1)**: (1) `verify.sh` menyimpan log lengkap setiap langkah yang gagal ke berkas (mis. di bawah direktori sementara bernama tetap yang dicetak di ringkasan) supaya kegagalan di dalam hook dapat dianalisis; (2) setelah penyebab terbukti, putuskan perbaikan (mis. pemeriksaan kesiapan server dengan polling dan batas waktu lebih longgar). Tidak ada perubahan `verify.sh` atau batas waktu pada task ini | Pengamatan orkestrator 2026-11-27 (F1-T102b, F1-T103) | Sebelum merge berikutnya yang sering gagal di hook, atau Tahap 8 Fase 1 selesai |
| TD-61 | Flake `schemathesis` pada run pertama kelompok `cache-isi` (Medium untuk keandalan CI; run 45973841577, job 140420555018, commit a8c1d95): `verify.sh` run pertama merah di langkah `schemathesis` (24 lulus, 1 gagal), run kedua dan ketiga di job yang sama dan 31 job lain hijau. **Fakta terbukti dari log**: ringkasan galat `test-verify.sh` hanya memuat baris vitest (penyaring `×`/`Timeout`/`ditolak` menangkap keluaran langkah lain), jadi keluaran schemathesis yang gagal tidak tercatat; **penyebab [TIDAK DIKETAHUI]** (kandidat: kegagalan `docker pull` image schemathesis di run pertama yang dicetak `tidak tersedia` dan tidak masuk penyaring, kasus 5xx sesaat dari backend efemeral). Perbaikan diagnostik (commit 64d53d4): `test-verify.sh` kini mencetak 60 baris keluaran sebelum setiap penanda `==> [langkah] GAGAL` (maks. 3), sehingga flake berikutnya terlihat penyebabnya. Pemicu tindak lanjut: flake terulang, lalu usulan di `verify.sh` (ulang `docker pull` beberapa kali, pull image sebelum backend dibangun) menunggu persetujuan user | CI run 45973841577 (CR-029) | Saat flake terulang |
| TD-56 | **SELESAI 2026-11-27 (KEPUTUSAN-402):** kunci batas laju per sumber untuk IPv6 diagregasi per /64 (`rate_limit::RateKey`, `IPV6_RATE_KEY_PREFIX`), sehingga satu klien IPv6 yang berganti alamat di dalam /64-nya tidak memperoleh anggaran baru; IPv4 dan IPv4-dipetakan-IPv6 tetap per alamat (dinormalkan); `source_ip` audit tetap alamat penuh. Diuji unit `rate_limit.rs` dan integrasi `tests/http_trusted_proxy.rs`. **Tetap perlu:** batas per email/NIB di Valkey untuk pendaftaran (TD-44b), karena batas per IP tidak menahan penyerang dengan banyak alamat/prefiks | Task TD-22 (2026-11-27), review keamanan T1 | Selesai |
| TD-57 | Validasi data perusahaan (review DBA CR-036 C3, review keamanan CR-036 T-01; Low): (a) jalur pembuatan perusahaan oleh Verifikator (`admin_create.rs`, `NIB_MAX`/`NPWP_MAX` 64, satu token) belum memakai format KEPUTUSAN-325 (NIB 13 angka, NPWP 16 angka) seperti pendaftaran mandiri dan koreksi HR; setelah disatukan, tambahkan CHECK format `nib`/`npwp` di `corporate_client` lewat migrasi expand (validasi data lama dulu, `NOT VALID` lalu `VALIDATE`), dan perbarui DD A20; (b) teks bebas (nama badan usaha, alamat, telepon, jabatan) belum dinormalisasi Unicode NFC sebelum disimpan dan dibandingkan (bentuk komposisi berbeda tampak sama di panel Verifikator dan dihitung "berubah" di koreksi); kontrol bidi dan lebar-nol sudah ditolak (KEPUTUSAN-418). NFC butuh crate normalisasi (cek registry + `docs/sources.md`) | Review T1 CR-036 | Sebelum production korporat |
| TD-58 | BFF Next.js (`web/backoffice`, `web/korporat`, dan BFF lain) belum meneruskan IP klien ke `api`: permintaan lewat BFF tercatat dengan alamat BFF di `source_ip` audit dan batas laju. Syarat (review keamanan T1 TD-22): BFF hanya meneruskan `X-Forwarded-For` yang ditetapkan Caddy (nilai dari Caddy + alamat Caddy di kanan, tidak pernah nilai kiriman browser tanpa lewat Caddy), BFF tidak dapat dijangkau tanpa lewat Caddy (tanpa `ports:` publik), lalu alamat BFF (`/32`) baru ditambahkan ke `TRUSTED_PROXY_CIDRS`. **Wajib sebelum audit aksi portal (backoffice, korporat) dianggap memenuhi PRD §7.2 "Sumber"** | Review keamanan T1 TD-22 (2026-11-27) | Sebelum production portal web (task `web/**` terpisah) |
| TD-64 | CI qpdf (KEPUTUSAN-450): paket Ubuntu noble memakai qpdf 11.9.0 (SRC-980), sedangkan kode, komentar test, dan `docs/05-stack.md` §1.4 merujuk 12.4.2 (SRC-862); test `--ignored` di CI membuktikan perilaku 11.9.0, bukan 12.4.2. Kandidat tindak lanjut: (a) bila ada test yang gagal karena beda versi, putuskan memasang biner rilis 12.4.2 (sha256, SRC-862) di CI dan di image api; (b) production/image api harus memakai versi yang sama dengan yang diuji; (c) job `test-qpdf` belum menjadi required check | Task CI qpdf | Task deploy/image api; keputusan user bila beda perilaku |

_Belum ada._
