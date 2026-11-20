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
| US-809 | Pengaturan markup per rekanan | BR-042, BR-063 | Must | 2 | Backlog |
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
| US-1501 | Pencatatan event audit | BR-080, BR-081, BR-107 | Must | 1 (event fase berikutnya ditambahkan per story; peringatan email AC3 di Fase 2 setelah EN-16; DoR AC terpenuhi, PRD v1.2 final) | Backlog |
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
| US-1901 | Pendaftaran perusahaan mandiri | BR-136 | Must | 1 (F1-T101–F1-T103; bentuk bukti kewenangan G-27) | Backlog |
| US-1902 | Verifikasi perusahaan oleh Verifikator | BR-136 | Must | 1 (F1-T104, F1-T106) | Backlog |
| US-1903 | Perusahaan dibuatkan Verifikator | BR-136 | Must | 1 (F1-T105, F1-T106; persetujuan orang kedua G-27) | Backlog |
| US-1904 | Akun HR: masuk, MFA, pemulihan MFA, kelola anggota HR | BR-132, BR-145 | Must | 1 (F1-T85–F1-T88, F1-T90, F1-T100, F1-T107, F1-T108); AC7–AC9 (daftar ulang TOTP dan reset MFA HR) di Fase 2 | Backlog |
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
| US-1916 | Pengelolaan perusahaan di backoffice | BR-136, BR-060 | Must | 1 (bagian K1, F1-T106); pesanan dan invoice di Fase 3 | Backlog |
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

### CR-004: Pendapatan transaksi berupa markup, menggantikan komisi potongan

Diajukan: 2026-11-17 (`docs/02-catatan-brd.md` Q6, Q6a)
Deskripsi: Harga konsultasi dan produk diambil dari rekanan, lalu Ruang Sehat menambahkan markup berbentuk persen atau nominal yang dapat diatur per rekanan oleh admin Ruang Sehat. Model komisi potongan di Business Case §5.1 diganti. Biaya dari rekanan tetap.
Alasan: keputusan user.
Dampak ke scope / waktu / arsitektur: Business Case §5.1–5.2 diperbarui istilahnya (komisi → markup) setelah gate Tahap 2. Pengaturan markup per rekanan masuk backoffice (BR-063). Pencatatannya di audit trail (BR-080) adalah usulan penyusun berdasarkan ADR-005, disetujui user di gate Tahap 2.
Keputusan: Diterima user (T2-Q6, T2-Q6a), berlaku sejak BRD v1.0

### CR-005: Semua layanan yang dipesan di aplikasi dibayar di aplikasi

Diajukan: 2026-11-17 (setelah gate Tahap 2, `docs/02-catatan-brd.md` Q13–Q20)
Deskripsi: Setiap layanan yang dimulai atau didaftarkan di aplikasi dibayar di aplikasi, termasuk pendaftaran klinik, pendaftaran lab, dan home visit. Pasien yang datang ke klinik atau lab cukup menunjukkan appointment di aplikasi mobile. Membalik BR-024 dan butir non-scope "Pembayaran klinik, lab, dan home visit di aplikasi" di BRD v1.0 §4.2; memperluas BR-040, BR-041, P-04, P-05.
Alasan: keputusan user.
Dampak ke scope / waktu / arsitektur: pembagian dana langsung (CR-003) dan aturan refund (BR-045, PT-27) berlaku juga untuk klinik, lab, dan home visit. Cakupan konfirmasi legal L-04, L-08, L-10, L-12, L-13, dan L-14 meluas; PT-28 (pajak) ditambahkan. Sistem rekanan perlu menerima status lunas dan mencocokkan appointment saat pasien datang (PT-05). Business Case §5 perlu memuat markup klinik, lab, dan home visit saat diperbarui.
Keputusan: Diterima user (T2-Q13 sampai T2-Q20): tindakan tambahan yang tidak tercantum di aplikasi dibayar di luar aplikasi; markup dan aturan biaya transaksi T2-Q12 berlaku untuk semua layanan; biaya rekanan dipakai atau tidak diputuskan CEO (PT-17); pembatalan layanan terjadwal sebelum batas waktu di-refund; informasi biaya MCU/lab menampilkan harga bayar di aplikasi. Berlaku sejak BRD v1.1 (disetujui user di gate revisi 2026-11-17, T2-Gate-CR005)

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
- Syarat final penyedia: DOKU (atau Xendit) final setelah konfirmasi tertulis atas lokasi pemrosesan dan penyimpanan data, nomor izin BI, split settlement untuk markup dan bagian rekanan, refund dari dana yang sudah terbagi (PT-27), dan biaya (PT-36). Lalamove final setelah konfirmasi tertulis lokasi data (PT-37). Penyedia yang memproses data di luar Indonesia tidak dipilih (US-1406 AC2, `[USER] T4-Q1`).
Keputusan: Disetujui user lewat T5-Q1–T5-Q8 dan final di gate Tahap 5 (2026-11-18, T5-G2–G4): Valkey sebagai pengganti "Redis" dikonfirmasi (T5-G2 a); perutean Magic Transit diterima sementara sampai dikonfirmasi tertulis ke Biznet Gio (T5-G3, PT-39)

### CR-013: Pajak dikenakan atas harga rekanan + markup dan ditambahkan ke total

Diajukan: 2026-11-18 (Tahap 5, jawaban PT-28; `docs/05-catatan-stack.md` T5-Q11, T5-Q11a)
Deskripsi: pajak dihitung dari (harga rekanan + markup) setelah markup diterapkan, ditambahkan ke total yang dibayar pasien, dan tampil sebagai baris terpisah di rincian harga `[USER] T5-Q11, T5-Q11a`. Tarif pajak menjadi parameter berlaku-mulai yang diatur Keuangan (pola docs/04-data-dictionary.md §0.8). Jenis pajak, tarif, perlakuan pajak atas ongkos kirim dan biaya transaksi, pihak pemungut/penyetor, dan isi bukti bayar `[TIDAK DIKETAHUI]` (sisa PT-28).
Alasan: jawaban user atas PT-28 ("sama seperti layanan lainnya, karena markup dulu baru dibebakan pajaknya") dan pilihan "Di atas harga rekanan+markup".
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

_Belum ada._
