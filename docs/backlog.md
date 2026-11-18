# Backlog

Diisi mulai Tahap 3 (PRD). Format story dan change request: lihat `.claude/skills/runbook/references/templates.md`.

## Definition of Ready

- Acceptance criteria jelas dan bisa diuji
- Merujuk ke bagian PRD dan ADR yang relevan
- Dependensi sudah selesai atau jelas
- Ukurannya muat dalam satu sesi kerja (satu PR yang bisa direview dalam 15 menit)

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

## Epic dan Story

Sumber: `docs/03-prd.md` v1.1 (v1.0 Tahap 3, revisi CR-007 Tahap 4). Acceptance criteria, audit event, dan rujukan lengkap setiap story ada di PRD bagian 5 (`[KEPUTUSAN-03]`). Prioritas adalah usulan penyusun `[UP-01]` yang disetujui di gate Tahap 3, 2026-11-17 (T3-Gate). Kolom Fase diisi di Tahap 7.

Story yang terblokir pertanyaan terbuka: US-1605 dan US-1606 (PT-24, L-06). Story dengan ketergantungan legal yang dapat mengubah AC: US-104, US-108, US-203, US-308, US-309, US-805–US-808, US-1306, US-1309 (lihat PRD §12.3).

### [EP-01] Akun, identitas, dan anggota keluarga (aplikasi pasien)

| Story | Judul | Induk BR | Prioritas | Fase | Status |
| --- | --- | --- | --- | --- | --- |
| US-101 | Registrasi dengan email dan password | BR-001, BR-103 | Must | — | Backlog |
| US-102 | Dihapus oleh CR-007 (sebelumnya: masuk dengan akun Google atau Apple) | BR-001 | Won't | — | Dihapus |
| US-103 | Masuk, keluar, dan atur ulang password | BR-001 | Must | — | Backlog |
| US-104 | Persetujuan pemrosesan data | BR-103, BR-104, BR-107 | Must | — | Backlog |
| US-105 | Profil data diri | BR-002 | Must | — | Backlog |
| US-106 | Unggah NIK dan foto KTP | BR-003 | Must | — | Backlog |
| US-107 | Pembatasan layanan berdasarkan status verifikasi | BR-003, BR-004 | Must | — | Backlog |
| US-108 | Tambah anggota keluarga | BR-002, BR-008, BR-104 | Should | — | Backlog |
| US-109 | Ubah dan hapus anggota keluarga | BR-008, BR-106 | Should | — | Backlog |

### [EP-02] Verifikasi dan pengelolaan akun

| Story | Judul | Induk BR | Prioritas | Fase | Status |
| --- | --- | --- | --- | --- | --- |
| US-201 | Verifikasi identitas (KTP/KK) | BR-004, BR-008 | Must | — | Backlog |
| US-202 | Pendaftaran tenaga kesehatan | BR-005, BR-102 | Must | — | Backlog |
| US-203 | Verifikasi tenaga kesehatan | BR-005, BR-102 | Must | — | Backlog |
| US-204 | Pemantauan masa berlaku SIP | BR-102 | Must | — | Backlog |
| US-205 | Onboarding rekanan | BR-006, BR-031 | Must | — | Backlog |
| US-206 | Akun admin mitra | BR-006 | Must | — | Backlog |
| US-207 | Menonaktifkan dan mengaktifkan akun | BR-060 | Must | — | Backlog |
| US-208 | Akun dan sub-peran admin Rumah Sehat | BR-060, BR-080 | Must | — | Backlog |

### [EP-03] Konsultasi online

| Story | Judul | Induk BR | Prioritas | Fase | Status |
| --- | --- | --- | --- | --- | --- |
| US-301 | Tarif dan durasi konsultasi (menu portal klinik) | BR-018, BR-064 | Must | — | Backlog |
| US-302 | Jadwal praktik online | BR-017 | Must | — | Backlog |
| US-303 | Cari dan pilih tenaga kesehatan | BR-010, BR-011, BR-012, BR-014 | Must | — | Backlog |
| US-304 | Pesan sesi konsultasi | BR-010, BR-011, BR-012, BR-040 | Must | — | Backlog |
| US-305 | Sesi chat | BR-010, BR-016 | Must | — | Backlog |
| US-306 | Sesi video | BR-010, BR-016 | Must | — | Backlog |
| US-307 | Ketidakhadiran dan pembatalan oleh tenaga kesehatan | BR-045 | Must | — | Backlog |
| US-308 | Catatan konsultasi | BR-013, BR-016, BR-114 | Must | — | Backlog |
| US-309 | Resep digital | BR-015, BR-114 | Must | — | Backlog |
| US-310 | Riwayat konsultasi pasien | BR-016, BR-106 | Must | — | Backlog |
| US-311 | Daftar sesi tenaga kesehatan | BR-013 | Must | — | Backlog |

### [EP-04] Pendaftaran klinik dan lab, check-in, dan antrean

| Story | Judul | Induk BR | Prioritas | Fase | Status |
| --- | --- | --- | --- | --- | --- |
| US-401 | Cari klinik, lab, dan layanannya | BR-020, BR-021 | Must | — | Backlog |
| US-402 | Pesan pendaftaran klinik atau lab | BR-020, BR-021, BR-024, BR-040 | Must | — | Backlog |
| US-403 | Appointment dan kode check-in | BR-024 | Must | — | Backlog |
| US-404 | Check-in lewat sistem rekanan | BR-024, BR-070 | Must | — | Backlog |
| US-405 | Antrean real-time | BR-027 | Should | — | Backlog |
| US-406 | Riwayat pendaftaran | BR-026 | Must | — | Backlog |

### [EP-05] Home visit

| Story | Judul | Induk BR | Prioritas | Fase | Status |
| --- | --- | --- | --- | --- | --- |
| US-501 | Cari dan pesan home visit | BR-022, BR-024, BR-040 | Must | — | Backlog |
| US-502 | Penugasan tenaga kesehatan | BR-022, BR-070 | Must | — | Backlog |
| US-503 | Status kunjungan oleh tenaga kesehatan | BR-022 | Must | — | Backlog |
| US-504 | Pantau kunjungan dan laporkan tenaga kesehatan tidak datang | BR-022, BR-045 | Must | — | Backlog |

### [EP-06] Informasi biaya MCU dan lab

| Story | Judul | Induk BR | Prioritas | Fase | Status |
| --- | --- | --- | --- | --- | --- |
| US-601 | Cari dan bandingkan biaya MCU dan pemeriksaan lab | BR-023 | Must | — | Backlog |
| US-602 | Lanjut mendaftar dari informasi biaya | BR-023, BR-020, BR-021 | Should | — | Backlog |

### [EP-07] Pembelian produk

| Story | Judul | Induk BR | Prioritas | Fase | Status |
| --- | --- | --- | --- | --- | --- |
| US-701 | Jelajah dan cari katalog | BR-030, BR-033 | Must | — | Backlog |
| US-702 | Detail produk | BR-030, BR-031, BR-032, BR-116 | Must | — | Backlog |
| US-703 | Keranjang dan checkout | BR-034, BR-040 | Must | — | Backlog |
| US-704 | Status dan pelacakan pesanan | BR-033, BR-034 | Must | — | Backlog |
| US-705 | Konfirmasi terima dan klaim barang | BR-045 | Must | — | Backlog |
| US-706 | Riwayat pesanan | BR-035 | Must | — | Backlog |

### [EP-08] Pembayaran, harga, pembagian dana, dan refund (T1)

| Story | Judul | Induk BR | Prioritas | Fase | Status |
| --- | --- | --- | --- | --- | --- |
| US-801 | Ringkasan harga sebelum membayar | BR-042, BR-044, BR-024 | Must | — | Backlog |
| US-802 | Pembayaran lewat payment gateway | BR-040, BR-115 | Must | — | Backlog |
| US-803 | Pembagian dana per transaksi | BR-041, BR-044 | Must | — | Backlog |
| US-804 | Bukti pembayaran | BR-040 | Must | — | Backlog |
| US-805 | Pembatalan layanan terjadwal oleh pasien | BR-045 | Must | — | Backlog |
| US-806 | Refund otomatis | BR-045 | Must | — | Backlog |
| US-807 | Klaim refund | BR-045 | Must | — | Backlog |
| US-808 | Eksekusi dan status refund | BR-045 | Must | — | Backlog |
| US-809 | Pengaturan markup per rekanan | BR-042, BR-063 | Must | — | Backlog |
| US-810 | Pengaturan biaya rekanan | BR-043, BR-063 | Must | — | Backlog |
| US-811 | Laporan transaksi dan rekonsiliasi pembagian dana | BR-046 | Must | — | Backlog |
| US-812 | Pengaturan parameter bisnis | BR-045, BR-063 | Must | — | Backlog |

### [EP-09] Hasil lab

| Story | Judul | Induk BR | Prioritas | Fase | Status |
| --- | --- | --- | --- | --- | --- |
| US-901 | Penerimaan hasil lab | BR-025, BR-070 | Must | — | Backlog |
| US-902 | Lihat dan unduh hasil lab | BR-025, BR-050 | Must | — | Backlog |

### [EP-10] Notifikasi

| Story | Judul | Induk BR | Prioritas | Fase | Status |
| --- | --- | --- | --- | --- | --- |
| US-1001 | Notifikasi pasien per kejadian dan kanal | BR-050 | Must | — | Backlog |
| US-1002 | Pengingat jadwal | BR-050 | Must | — | Backlog |
| US-1003 | Notifikasi tenaga kesehatan dan admin | BR-013, BR-102 | Must | — | Backlog |

### [EP-11] Konten dan layanan pelanggan

| Story | Judul | Induk BR | Prioritas | Fase | Status |
| --- | --- | --- | --- | --- | --- |
| US-1101 | Pengelolaan konten (CMS) | BR-061 | Must | — | Backlog |
| US-1102 | Konten di aplikasi pasien | BR-061 | Must | — | Backlog |
| US-1103 | Ajukan keluhan | BR-062 | Must | — | Backlog |
| US-1104 | Kelola keluhan | BR-062, BR-064, BR-065 | Must | — | Backlog |

### [EP-12] Portal rekanan

| Story | Judul | Induk BR | Prioritas | Fase | Status |
| --- | --- | --- | --- | --- | --- |
| US-1201 | Laporan transaksi dan pembagian dana rekanan | BR-046, BR-064, BR-065 | Must | — | Backlog |
| US-1202 | Lihat profil rekanan | BR-064, BR-065 | Must | — | Backlog |
| US-1203 | Lihat keluhan terkait rekanan | BR-064, BR-065 | Must | — | Backlog |

### [EP-13] Aplikasi RME minimalis (T1)

| Story | Judul | Induk BR | Prioritas | Fase | Status |
| --- | --- | --- | --- | --- | --- |
| US-1301 | Onboarding fasyankes ke RME minimalis | BR-120 | Must | — | Backlog |
| US-1302 | Akun dan peran pengguna RME | BR-120 | Must | — | Backlog |
| US-1303 | Jadwal, layanan, dan harga | BR-122 | Must | — | Backlog |
| US-1304 | Appointment dan check-in | BR-125, BR-024 | Must | — | Backlog |
| US-1305 | Antrean | BR-125, BR-027 | Should | — | Backlog |
| US-1306 | Catatan rekam medis kunjungan | BR-123 | Must | — | Backlog |
| US-1307 | Hasil lab | BR-124, BR-025 | Must | — | Backlog |
| US-1308 | Home visit | BR-125, BR-022 | Must | — | Backlog |
| US-1309 | Pengiriman data rekam medis ke SATUSEHAT | BR-123 | Must | — | Backlog |
| US-1310 | Riwayat rekam medis pasien di fasyankes | BR-123, BR-126 | Must | — | Backlog |

### [EP-14] Integrasi rekanan dan pihak ketiga

| Story | Judul | Induk BR | Prioritas | Fase | Status |
| --- | --- | --- | --- | --- | --- |
| US-1401 | Spesifikasi kontrak integrasi dan lingkungan uji | BR-070, BR-121 | Must | — | Backlog |
| US-1402 | Sinkronisasi data rekanan | BR-070 | Must | — | Backlog |
| US-1403 | Pengiriman pesanan dan pembatalan ke rekanan | BR-070 | Must | — | Backlog |
| US-1404 | Penerimaan status dari rekanan | BR-070 | Must | — | Backlog |
| US-1405 | Pemantauan dan status integrasi per rekanan | BR-070, BR-090 | Must | — | Backlog |
| US-1406 | Integrasi pihak ketiga | BR-071 | Must | — | Backlog |

### [EP-15] Audit trail (fitur)

| Story | Judul | Induk BR | Prioritas | Fase | Status |
| --- | --- | --- | --- | --- | --- |
| US-1501 | Pencatatan event audit | BR-080, BR-081, BR-107 | Must | — | Backlog |
| US-1502 | Halaman Audit Log | BR-080 | Must | — | Backlog |
| US-1503 | Export audit log | BR-080 | Must | — | Backlog |
| US-1504 | Perlindungan audit log | BR-080 | Must | — | Backlog |

### [EP-16] Persetujuan dan hak subjek data (T1)

| Story | Judul | Induk BR | Prioritas | Fase | Status |
| --- | --- | --- | --- | --- | --- |
| US-1601 | Lihat dan tarik persetujuan | BR-103, BR-106 | Must | — | Backlog |
| US-1602 | Ajukan permintaan hak subjek data | BR-106 | Must | — | Backlog |
| US-1603 | Proses permintaan hak subjek data | BR-106, BR-111 | Must | — | Backlog |
| US-1604 | Salinan data pribadi | BR-106 | Must | — | Backlog |
| US-1605 | Penghapusan dan anonimisasi | BR-106, BR-112 | Must | — | Backlog |
| US-1606 | Retensi dan pemusnahan otomatis | BR-112 | Must | — | Backlog |

### [EP-17] Pengukuran KPI

| Story | Judul | Induk BR | Prioritas | Fase | Status |
| --- | --- | --- | --- | --- | --- |
| US-1701 | Dasbor KPI | BR-090 | Must | — | Backlog |

## Bug

_Belum ada._

## Change Request

### CR-001: Semua tenaga kesehatan berpraktik atas nama klinik rekanan

Diajukan: 2026-11-17 (wawancara Tahap 2, `docs/02-catatan-brd.md` Q2)
Deskripsi: Dokter, psikolog, fisioterapis, dan tenaga medis yang melayani di Rumah Sehat berpraktik atas nama klinik rekanan (fasyankes). PT. Arunika berperan sebagai platform (PSE). Tenaga kesehatan "mitra langsung" Rumah Sehat tanpa fasyankes ditiadakan. Mengubah brief PS6e.
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
Deskripsi: Harga konsultasi dan produk diambil dari rekanan, lalu Rumah Sehat menambahkan markup berbentuk persen atau nominal yang dapat diatur per rekanan oleh admin Rumah Sehat. Model komisi potongan di Business Case §5.1 diganti. Biaya dari rekanan tetap.
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
Deskripsi: Dari empat butir yang belum diputuskan (BRD §4.3, PT-13), RME **minimalis** dan antrean real-time klinik masuk rilis 1; RME lengkap, loyalitas/poin/voucher, dan multi-bahasa tidak masuk. RME minimalis adalah aplikasi web terpisah dari aplikasi dan portal Rumah Sehat, dibangun di proyek ini untuk rilis 1, menjadi "sistem rekanan" bagi klinik dan lab yang belum punya sistem, dan terhubung ke Rumah Sehat lewat integrasi yang sama dengan sistem rekanan lain. Isinya: jadwal, layanan, dan harga; appointment dan check-in; antrean; rekam medis kunjungan yang datanya sesuai ketentuan SATUSEHAT; hasil lab; penugasan home visit. Hanya mencatat pasien yang datang lewat Rumah Sehat. Portal Rumah Sehat untuk rekanan ditegaskan hanya untuk pelaporan dan rekap, ditambah satu menu pengaturan konsultasi online (T3-Q4 tetap, T3-Q16).
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
Deskripsi: PDF resep memuat kode verifikasi acak dan QR ke halaman verifikasi di domain resmi Rumah Sehat (alamat juga dicetak sebagai teks). Halaman publik menampilkan status resep (Terbit/Dibatalkan), tanggal, dokter dan nomor SIP, klinik, daftar obat, dan nama pasien disamarkan; isi final mengikuti L-05/L-08. Batas laju per sumber; setiap verifikasi dicatat E-05 (pemeriksa anonim). Tanda tangan elektronik dipertimbangkan setelah L-05.
Alasan: apotek bukan rekanan, sehingga bukti keaslian harus dapat diperiksa tanpa akun; PDF saja dapat dipalsukan (threat model API-13, Tinggi).
Dampak ke scope / waktu / arsitektur: PRD v1.1 US-309 AC5–AC8; satu halaman publik di backend Rumah Sehat; kolom kode verifikasi di data dictionary A9.8. Penandaan "sudah ditebus" tidak termasuk (tidak dipilih user).
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

### CR-012: Hosting VPS Biznet Gio dan Hostinger, stack Rust/Next.js/Flutter, penyedia pihak ketiga, dan persetujuan deploy di sisi VPS

Diajukan: 2026-11-18 (Tahap 5, `docs/05-catatan-stack.md` T5-Q1–T5-Q8)
Deskripsi:
- Hosting: VPS Biznet Gio sebagai lokasi utama (aplikasi, database utama, Keycloak, Valkey, OpenBao, object storage WORM dan backup) dan Hostinger Jakarta sebagai lokasi kedua (database standby, salinan backup terenkripsi, node ketiga OpenBao, pemeriksa ketersediaan). Layanan terkelola yang tidak tersedia dijalankan sendiri: PostgreSQL 18, OpenBao sebagai pengganti KMS dan secret manager, Loki/Prometheus/Grafana/GlitchTip sebagai observability. Komponen "Redis" diimplementasikan dengan Valkey. Rincian dan biaya di ADR-025.
- Stack: backend Rumah Sehat dan RME minimalis memakai Rust; backoffice, portal rekanan, dan web RME memakai Next.js dengan server Next.js sebagai BFF; aplikasi mobile Flutter; satu monorepo (ADR-024).
- Penyedia pihak ketiga: payment gateway DOKU sebagai kandidat utama dan Xendit sebagai cadangan; email lewat Amazon SES region Jakarta (ap-southeast-3) lewat API, karena endpoint SMTP tidak tersedia di Jakarta (SRC-84, SRC-209); kurir Lalamove saja.
- CI/CD: GitHub untuk kode, CI, dan registry image (hanya kode dan data sintetis). Staging di-deploy otomatis; deploy production dipicu orang kedua yang berwenang lewat bastion dengan sertifikat SSH berumur pendek dari OpenBao (MFA), atas digest yang lulus staging, dan tercatat di log audit OpenBao.
Alasan: pilihan user T5-Q1 (Rust + Next.js), masukan user di tengah T5-Q1 ("vpsnya hostinger dan biznet gio saja") dan T5-Q2 (Biznet utama + Hostinger), T5-Q3 (DOKU utama, Xendit cadangan), T5-Q4 (SES Jakarta), T5-Q5 (Lalamove saja), T5-Q6 (GitHub + agen deploy VPS), dan T5-Q8 (persetujuan di sisi VPS). Persetujuan di sisi VPS dipilih karena required reviewers GitHub hanya tersedia untuk repositori publik pada paket Free, Pro, dan Team (SRC-240).
Dampak ke scope / waktu / arsitektur:
- ADR baru: ADR-024 (stack) dan ADR-025 (hosting, OpenBao, Valkey, observability, CI/CD, rahasia), keduanya berstatus Diusulkan.
- ADR yang diubah sebagian oleh ADR-025: ADR-019 (butir 1–4, alternatif "server virtual yang dikelola sendiri" kini dipilih), ADR-013 (butir 2, 3, 5, 8), ADR-021 (butir 3 dan 4), ADR-010 (penyimpanan log, butir 3–5), ADR-009 (butir 5, 6, 9), ADR-017 (butir 5), dan ADR-007 (Redis menjadi Valkey, menunggu konfirmasi user). ADR-011 butir 7 ditafsirkan oleh ADR-024: BFF dijalankan di server Next.js. Teks ADR lama tidak diubah; baris Status-nya diberi catatan rujukan. Rincian per butir ada di tabel "Butir ADR lama yang diubah" di ADR-025.
- Modul kritis T1 bertambah: server Next.js (autentikasi dan sesi), konfigurasi OpenBao, bastion, dan skrip deploy production. Daftar modul kritis di `CLAUDE.md` diperbarui setelah gate Tahap 5.
- Dokumen yang terdampak: `docs/04-risk-register.md` (RT-21–RT-30 dan catatan perubahan risiko lama), `docs/05-stack.md`, `docs/05-response-codes.md`, Definition of Done di berkas ini (tiga butir baru), dan Business Case §6 (biaya infrastruktur ±Rp8,26 juta/bulan dibandingkan baseline di gate).
- Kewajiban operasional baru: patch keamanan Next.js berderajat Critical atau High dipasang paling lambat 7 hari (KEPUTUSAN-75); unseal OpenBao manual setelah restart oleh pemegang share; promosi standby database manual lintas penyedia.
- Waktu: kecepatan pengembangan Rust lebih lambat dengan besaran `[TIDAK DIKETAHUI]`, diukur di reference implementation Tahap 8 Fase 1 (RT-24). Beban operasional VPS menaikkan Business Case R-04 (RT-21).
- Pertanyaan terbuka baru: PT-35 (pemelihara setelah go-live), PT-36 (konfirmasi tertulis DOKU dan Xendit), PT-37 (konfirmasi tertulis Lalamove), PT-38 (pemegang share unseal OpenBao dan orang kedua), PT-39 (konfirmasi Biznet Gio: protokol load balancer, perutean Magic Transit, PPN).
- **Catatan tafsiran US-703 AC3 (PRD tidak diubah):** dengan kurir Lalamove saja, "pilihan kurir" pada US-703 AC3 dibaca sebagai jenis layanan atau kendaraan Lalamove yang tersedia untuk alamat itu. Alamat di luar area layanan Lalamove membuat checkout tidak dapat dilanjutkan, sesuai kalimat kedua AC3 `[USER] T5-Q5`. Tafsiran ini ditampilkan di gate Tahap 5.
- Syarat final penyedia: DOKU (atau Xendit) final setelah konfirmasi tertulis atas lokasi pemrosesan dan penyimpanan data, nomor izin BI, split settlement untuk markup dan bagian rekanan, refund dari dana yang sudah terbagi (PT-27), dan biaya (PT-36). Lalamove final setelah konfirmasi tertulis lokasi data (PT-37). Penyedia yang memproses data di luar Indonesia tidak dipilih (US-1406 AC2, `[USER] T4-Q1`).
Keputusan: Disetujui user lewat T5-Q1–T5-Q8 (2026-11-18). Final di gate Tahap 5, termasuk konfirmasi Valkey sebagai pengganti "Redis" (T4-Q4) dan penilaian user atas perutean Magic Transit terhadap T4-Q1

### CR-013: Pajak dikenakan atas harga rekanan + markup dan ditambahkan ke total

Diajukan: 2026-11-18 (Tahap 5, jawaban PT-28; `docs/05-catatan-stack.md` T5-Q11, T5-Q11a)
Deskripsi: pajak dihitung dari (harga rekanan + markup) setelah markup diterapkan, ditambahkan ke total yang dibayar pasien, dan tampil sebagai baris terpisah di rincian harga `[USER] T5-Q11, T5-Q11a`. Tarif pajak menjadi parameter berlaku-mulai yang diatur Keuangan (pola docs/04-data-dictionary.md §0.8). Jenis pajak, tarif, perlakuan pajak atas ongkos kirim dan biaya transaksi, pihak pemungut/penyetor, dan isi bukti bayar `[TIDAK DIKETAHUI]` (sisa PT-28).
Alasan: jawaban user atas PT-28 ("sama seperti layanan lainnya, karena markup dulu baru dibebakan pajaknya") dan pilihan "Di atas harga rekanan+markup".
Dampak ke scope / waktu / arsitektur: rumus harga PRD AB-11/AB-12 dan tampilan rincian harga (US-801, US-812) bertambah komponen pajak; harga terkunci (ADR-015 butir 3) menyimpan tarif dan nominal pajak; jurnal berpasangan menambah akun pajak tanpa perubahan struktur (ADR-015 konsekuensi); data dictionary A6/A8/A13 bertambah kolom/parameter pajak lewat migrasi; laporan keuangan dan rekonsiliasi. Teks PRD dan data dictionary diperbarui lewat CR ini sebelum pengembangan EP-08, setelah sisa PT-28 dijawab.
Keputusan: Disetujui user (T5-Q11a, 2026-11-18) untuk arah rumus; penerapan ke PRD dan data dictionary dijadwalkan di Tahap 7, sebelum EP-08. Final di gate Tahap 5

## Tech Debt

_Belum ada._
