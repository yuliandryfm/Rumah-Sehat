# Traceability Matrix

Menghubungkan setiap kebutuhan bisnis sampai ke kode dan test-nya. Fitur atau story tanpa induk BR adalah tanda ada yang dikarang dan harus dihapus atau dikonfirmasi user.

Diisi bertahap: kolom BR di Tahap 2, story di Tahap 3, test case di Tahap 7, file kode/test di Tahap 8.

| BR | Kebutuhan bisnis | Sumber BR | Story | Test case | File kode / test | Status |
| --- | --- | --- | --- | --- | --- | --- |
| BR-001 | Pasien dapat mendaftar dan masuk dengan email + password. Login Google/Apple dihapus (CR-007) | `[DOK] P4.4` `[USER] T4-Q1, T4-Q2` | US-101, US-103 (US-102 dihapus CR-007) |  |  | BRD v1.3 (CR-007/Tahap 4) |
| BR-002 | Rumah Sehat menyimpan data identitas dasar pasien (nama, tanggal lahir, jenis kelamin, nomor HP, email,… | `[DOK] P4.1` `[USER] T3-Q3` | US-105, US-108 |  |  | BRD v1.2 (CR-006/Tahap 3) |
| BR-003 | NIK dan foto KTP pasien wajib sebelum memakai konsultasi online, home visit, serta pendaftaran klinik da… | `[DOK] P4.6a` | US-106, US-107 |  |  | BRD v1.0 |
| BR-004 | Admin Rumah Sehat memverifikasi NIK dan foto KTP secara manual (disetujui atau ditolak) sebelum layanan… | `[DOK] P4.6b` | US-106, US-107, US-201 |  |  | BRD v1.0 |
| BR-005 | Akun dokter, psikolog, fisioterapis, dan tenaga medis diverifikasi dan dikelola admin Rumah Sehat. Setia… | `[DOK] P8.2a` `[USER] T2-Q2` | US-202, US-203 |  |  | BRD v1.0 |
| BR-006 | Akun rekanan (klinik, lab, toko kecantikan, toko alat kesehatan, apotek) dan admin mitranya diverifikasi… | `[DOK] P8.2a` `[USER] T2-Q3, T2-Q3b` | US-205, US-206 |  |  | BRD v1.0 |
| BR-007 | Aplikasi pasien tersedia di Android dan iOS; tenaga kesehatan memakai aplikasi mobile yang juga tersedia… | `[DOK] P6.1, P6.2` `[USER] T3-Q9` | NFR-COMP-01 |  |  | BRD v1.2 (CR-006/Tahap 3) |
| BR-008 | Pasien dapat memesan konsultasi online, pendaftaran klinik/lab, dan home visit untuk anggota keluarga ya… | `[USER] T3-Q3, T3-Q3a` | US-107, US-108, US-109, US-201, US-310, US-902 |  |  | BRD v1.2 (CR-006/Tahap 3) |
| BR-010 | Pasien dapat berkonsultasi dengan dokter lewat chat teks dan panggilan video | `[DOK] P3.1, P3.2` | US-303, US-304, US-305, US-306 |  |  | BRD v1.0 |
| BR-011 | Pasien dapat berkonsultasi kesehatan mental dengan psikolog lewat chat teks dan panggilan video | `[DOK] P2.1, P3.1, P3.2` | US-303, US-304 |  |  | BRD v1.0 |
| BR-012 | Pasien dapat berkonsultasi dengan fisioterapis dan tenaga medis lewat chat teks dan panggilan video | `[USER] T2-Q5` | US-303, US-304 |  |  | BRD v1.0 |
| BR-013 | Konsultasi berlangsung di Rumah Sehat. Tenaga kesehatan memakai aplikasi tenaga kesehatan untuk chat, vi… | `[DOK] PS6b` | US-308, US-311, US-1003 |  |  | BRD v1.0 |
| BR-014 | Setiap konsultasi diberikan oleh tenaga kesehatan atas nama klinik rekanan yang menaunginya; PT. Arunika… | `[USER] T2-Q2` | US-303, US-804 |  |  | BRD v1.0 |
| BR-015 | Dokter dapat menerbitkan resep digital setelah konsultasi. Pasien menebusnya sendiri di apotek mana pun;… | `[DOK] P3.4, PS1` | US-309 |  |  | BRD v1.0 |
| BR-016 | Isi chat konsultasi (termasuk foto), catatan tenaga kesehatan, dan resep disimpan. Panggilan video tidak… | `[DOK] P4.1` | US-305, US-306, US-308, US-310 |  |  | BRD v1.0 |
| BR-017 | Aplikasi tersedia 24 jam, 7 hari. Ketersediaan tenaga kesehatan mengikuti jadwal masing-masing. Target k… | `[DOK] P6.4` `[USER] T3-Q12` | US-302, NFR-AVL-01 |  |  | BRD v1.2 (CR-006/Tahap 3) |
| BR-018 | Tarif konsultasi berasal dari rekanan. Tarif dan durasi sesi diisi admin klinik di portal; jadwal prakti… | `[USER] T2-Q6, T3-Q4, T3-Q16` | US-301 |  |  | BRD v1.2 (CR-006/Tahap 3) |
| BR-020 | Pasien dapat mendaftar ke klinik rekanan berdasarkan jadwal, data klinik, dan harga layanan dari sistem… | `[DOK] P3.1, PS6a` `[USER] T2-Q13` | US-401, US-402, US-602 |  |  | BRD v1.1 (CR-005) |
| BR-021 | Pasien dapat mendaftar pemeriksaan di lab rekanan berdasarkan jadwal dan harga dari sistem rekanan | `[DOK] P3.1, PS6a` | US-401, US-402, US-602 |  |  | BRD v1.0 |
| BR-022 | Pasien dapat memesan home visit yang disediakan klinik rekanan, dilayani tenaga kesehatan klinik itu ter… | `[DOK] P1.2, PS6a` `[USER] T2-Q5` | US-501, US-502, US-503, US-504, US-1308 |  |  | BRD v1.0 |
| BR-023 | Pasien dapat melihat informasi biaya MCU dan pemeriksaan lab sebelum mendaftar, berdasarkan harga dari s… | `[DOK] P1.2, PS6a` `[USER] T2-Q20` | US-601, US-602 |  |  | BRD v1.1 (CR-005) |
| BR-024 | Pendaftaran klinik, pendaftaran lab, dan home visit yang dipesan di aplikasi dibayar di aplikasi. Saat d… | `[USER] T2-Q13, T2-Q14` | US-402, US-403, US-404, US-501, US-801, US-1304 |  |  | BRD v1.1 (CR-005) |
| BR-025 | Hasil lab dari lab rekanan diterima lewat integrasi, disimpan, dan diberitahukan ke pasien | `[DOK] P4.1, P4.5, PS6a` | US-901, US-902, US-1307 |  |  | BRD v1.0 |
| BR-026 | Riwayat pendaftaran klinik, lab, dan home visit pasien disimpan | `[DOK] P4.1` | US-406 |  |  | BRD v1.0 |
| BR-027 | Pasien yang sudah check-in di klinik rekanan dapat melihat antrean secara real-time, dari data antrean s… | `[USER] T3-Q1a, T3-Q1b, T3-Q14` | US-405, US-1305 |  |  | BRD v1.2 (CR-006/Tahap 3) |
| BR-030 | Pasien dapat membeli obat bebas, produk kecantikan, dan produk kesehatan lain (vitamin, suplemen, alat k… | `[DOK] PS1` | US-701, US-702 |  |  | BRD v1.0 |
| BR-031 | Outlet rekanan terdiri atas toko kecantikan, toko alat kesehatan, klinik rekanan, dan apotek rekanan. Ob… | `[DOK] PS2` `[USER] T2-Q3, T2-Q3b` | US-205, US-702 |  |  | BRD v1.0 |
| BR-032 | Obat resep tidak dijual di Rumah Sehat | `[DOK] PS1` | US-702 |  |  | BRD v1.0 |
| BR-033 | Katalog, stok, dan pesanan produk berasal dari sistem outlet rekanan lewat integrasi. Rumah Sehat tidak… | `[DOK] PS6, PS6a` | US-701, US-704, US-1402 |  |  | BRD v1.0 |
| BR-034 | Produk dikirim kurir pihak ketiga yang terintegrasi. Ongkos kirim dibayar pasien dan ditambahkan ke tota… | `[DOK] PS5` `[USER] T2-Q7` | US-703, US-704 |  |  | BRD v1.0 |
| BR-035 | Riwayat pesanan produk pasien disimpan | `[DOK] P4.1` | US-706 |  |  | BRD v1.0 |
| BR-040 | Semua layanan yang dipesan di aplikasi (konsultasi online, pendaftaran klinik, pendaftaran lab, home vis… | `[DOK] P3.3, PS4` `[USER] T2-Q4, T2-Q13` `[SRC-19]` | US-304, US-402, US-501, US-703, US-802, US-804 |  |  | BRD v1.1 (CR-005) |
| BR-041 | Payment gateway membagi dana setiap pembayaran langsung: bagian rekanan ke rekening rekanan, bagian Ruma… | `[USER] T2-Q4` | US-803 |  |  | BRD v1.1 (cakupan meluas, CR-005) |
| BR-042 | Harga yang dibayar pasien untuk konsultasi, pendaftaran klinik, pendaftaran lab, home visit, dan produk… | `[USER] T2-Q1, T2-Q6, T2-Q6a, T2-Q15` | US-801, US-809 |  |  | BRD v1.1 (CR-005) |
| BR-043 | Biaya dari rekanan (langganan atau biaya per pendaftaran yang ditagihkan ke klinik dan lab) dapat diatur… | `[DOK: 01-business-case.md]` `[USER] T1-Q1, T2-Q1, T2-Q16` | US-810 |  |  | BRD v1.1 (CR-005) |
| BR-044 | Biaya MDR untuk pembayaran QRIS ditanggung rekanan (dipotong dari bagian rekanan) dan tidak dibebankan k… | `[USER] T2-Q8, T2-Q12, T2-Q19` `[SRC-24]` | US-801, US-803 |  |  | BRD v1.1 (CR-005) |
| BR-045 | Refund diberikan bila layanan gagal terjadi (contoh: tenaga kesehatan tidak hadir, barang tidak dikirim… | `[USER] T2-Q9, T2-Q17, T2-Q18, T3-Q5, T3-Q6, T3-Q7, T3-Q8` | US-307, US-504, US-705, US-805, US-806, US-807, US-808, US-812, US-1403 |  |  | BRD v1.2 (CR-006/Tahap 3) |
| BR-046 | Admin Rumah Sehat, admin klinik, admin lab, dan admin outlet dapat melihat laporan transaksi dan pembagi… | `[DOK] P8.2a, PS6c, PS6d` `[USER] T2-Q4` | US-811, US-1201 |  |  | BRD v1.0 |
| BR-050 | Rumah Sehat mengirim pemberitahuan ke pasien lewat email dan notifikasi di dalam aplikasi; push dan WhatsApp dihapus (CR-007) | `[DOK] P4.5` `[USER] T4-Q1, T4-Q2` | US-902, US-1001, US-1002 |  |  | BRD v1.3 (CR-007/Tahap 4) |
| BR-060 | Backoffice admin Rumah Sehat menyediakan verifikasi dan pengelolaan akun (pasien, tenaga kesehatan, reka… | `[DOK] P8.2a` | US-207, US-208 |  |  | BRD v1.0 |
| BR-061 | Backoffice admin Rumah Sehat menyediakan CMS untuk banner, halaman info, artikel/konten edukasi kesehata… | `[DOK] P8.2a` | US-1101, US-1102 |  |  | BRD v1.0 |
| BR-062 | Backoffice admin Rumah Sehat menyediakan layanan pelanggan, termasuk penanganan keluhan | `[DOK] P8.2a` | US-1003, US-1103, US-1104 |  |  | BRD v1.0 |
| BR-063 | Backoffice admin Rumah Sehat menyediakan pengaturan markup per rekanan dan biaya rekanan | `[USER] T2-Q1, T2-Q6` | US-809, US-810, US-812 |  |  | BRD v1.0 |
| BR-064 | Portal admin klinik dan admin lab menyediakan laporan transaksi dan pembagian dana, profil, dan keluhan.… | `[DOK] PS6c` `[USER] T3-Q14, T3-Q16` | US-301, US-1104, US-1201, US-1202, US-1203 |  |  | BRD v1.2 (CR-006/Tahap 3) |
| BR-065 | Portal admin outlet (toko kecantikan, toko alat kesehatan, apotek) menyediakan laporan, pembagian dana,… | `[DOK] PS6d` `[USER] T2-Q3b` | US-1104, US-1201, US-1202, US-1203 |  |  | BRD v1.0 |
| BR-070 | Rumah Sehat terintegrasi otomatis dengan sistem klinik, lab, dan outlet rekanan untuk jadwal, harga dan… | `[DOK] PS6a, P4.3` `[USER] T2-Q6, T2-Q13, T3-Q14` | US-404, US-502, US-901, US-1304, US-1401, US-1402, US-1403, US-1404, US-1405 |  |  | BRD v1.2 (CR-006/Tahap 3) |
| BR-071 | Rumah Sehat terintegrasi dengan payment gateway (dengan pembagian dana), layanan panggilan video, kurir, dan email; semua penyedia di Indonesia (CR-007) | `[DOK] P4.2` `[USER] T2-Q4, T4-Q1, T4-Q2` | US-1406 |  |  | BRD v1.3 (CR-007/Tahap 4) |
| BR-080 | Audit trail append-only mencatat: (1) login, logout, dan percobaan login gagal; (2) akses ke data pasien… | `[DOK] P5.4` `[DOK: adr/ADR-005-audit-trail-append-only.md]` | US-208, US-1501, US-1502, US-1503, US-1504, PRD §7 |  |  | BRD v1.0 |
| BR-081 | Audit trail tidak boleh memuat password, PIN, OTP, token, atau nomor kartu/rekening lengkap | `[DOK: adr/ADR-005-audit-trail-append-only.md]` | US-1501, PRD §7 |  |  | BRD v1.0 |
| BR-090 | Rumah Sehat menyediakan data untuk mengukur KPI-01 sampai KPI-04 sesuai definisi di bagian 1.2: MAU bert… | `[USER] T2-Q10, T2-Q11` `[DOK: 01-business-case.md]` | US-1405, US-1701 |  |  | BRD v1.0 |
| BR-100 | Rumah Sehat terdaftar sebagai Penyelenggara Sistem Elektronik Lingkup Privat sebelum aplikasi digunakan… | `[SRC-12]` `[SRC-13]` `[SRC-27]` | PRD §10 |  |  | BRD v1.2 (CR-006/Tahap 3) |
| BR-101 | Konsultasi online berjalan dalam skema telemedisin yang sesuai ketentuan Kemenkes: klinik rekanan (fasya… | `[SRC-02]` `[SRC-03]` `[USER] T2-Q2` | PRD §10 |  |  | BRD v1.0 |
| BR-102 | Hanya tenaga kesehatan dengan STR dan SIP yang masih berlaku yang dapat diaktifkan melayani. Masa berlak… | `[SRC-02]` `[SRC-03]` `[SRC-21]` | US-202, US-203, US-204, US-1003 |  |  | BRD v1.0 |
| BR-103 | Pasien memberikan persetujuan eksplisit, tertulis atau terekam, sebelum datanya diproses, dengan informa… | `[SRC-15]` | US-101, US-104, US-1601 |  |  | BRD v1.0 |
| BR-104 | Data anggota keluarga yang masih anak hanya diproses dengan persetujuan orang tua atau wali | `[SRC-15]` | US-104, US-108 |  |  | BRD v1.0 |
| BR-105 | Data kesehatan (catatan tenaga kesehatan, isi chat konsultasi, resep, hasil lab), data anak, dan data ke… | `[SRC-15]` | NFR-SEC-05, NFR-PRIV-03 |  |  | BRD v1.0 |
| BR-106 | Pasien dapat menjalankan hak subjek data: informasi, akses dan salinan, perbaikan, penarikan persetujuan… | `[SRC-15]` `[DOK] P5.4` | US-109, US-310, US-1601, US-1602, US-1603, US-1604, US-1605 |  |  | BRD v1.0 |
| BR-107 | Seluruh kegiatan pemrosesan data pribadi direkam, selaras dengan audit trail BR-080 | `[SRC-15]` | US-104, US-1501, PRD §7 |  |  | BRD v1.0 |
| BR-108 | Rumah Sehat menjaga keamanan dan kerahasiaan data pribadi yang diprosesnya | `[SRC-15]` `[SRC-13]` | NFR-SEC-05, NFR-SEC-06 |  |  | BRD v1.0 |
| BR-109 | Tersedia prosedur pemberitahuan kegagalan pelindungan data pribadi secara tertulis paling lambat 3 x 24… | `[SRC-15]` `[SRC-13]` | PRD §10 |  |  | BRD v1.0 |
| BR-110 | Penilaian dampak pelindungan data pribadi (DPIA) dilakukan sebelum go-live | `[SRC-15]` | PRD §10 |  |  | BRD v1.0 |
| BR-111 | Rumah Sehat menunjuk pejabat/petugas pelindungan data pribadi | `[SRC-15]` `[SRC-16]` | US-1603, PRD §10 |  |  | BRD v1.0 |
| BR-112 | Setiap jenis data memiliki masa retensi; setelah masa retensi berakhir, data diakhiri pemrosesannya, dih… | `[SRC-15]` `[SRC-25]` | US-1605, US-1606, NFR-RET-01 |  |  | BRD v1.0 |
| BR-113 | Data disimpan di Indonesia; penyedia pihak ketiga juga memproses dan menyimpan data pribadi di Indonesia, tanpa kecuali | `[DOK] P5.5` `[SRC-15]` `[SRC-13]` `[USER] T4-Q1` | US-1406, NFR-PRIV-01, NFR-PRIV-02, NFR-DR-04 |  |  | BRD v1.3 (CR-007/Tahap 4) |
| BR-114 | Catatan konsultasi dan resep digital memenuhi ketentuan rekam medis elektronik dan resep elektronik yang… | `[SRC-18]` `[SRC-06]` `[SRC-07]` `[USER] T2-Q2` | US-308, US-309, PRD §10 |  |  | BRD v1.0 |
| BR-115 | Pembayaran hanya diproses lewat penyedia jasa pembayaran berizin BI. Model pembagian dana langsung (BR-0… | `[SRC-19]` `[SRC-20]` `[USER] T2-Q4` | US-802, US-1406, NFR-INT-04 |  |  | BRD v1.0 |
| BR-116 | Obat bebas, produk kecantikan, dan produk kesehatan yang dijual lewat Rumah Sehat memenuhi ketentuan per… | `[SRC-08]` `[SRC-10]` `[SRC-11]` `[USER] T2-Q3` | US-702, PRD §10 |  |  | BRD v1.0 |
| BR-117 | PT. Arunika memiliki perizinan berusaha yang sesuai untuk kegiatan Rumah Sehat sebelum go-live. Jenis iz… | `[SRC-09]` `[SRC-23]` | PRD §10 |  |  | BRD v1.0 |
| BR-118 | Perkembangan regulasi kesehatan digital dipantau sampai go-live, karena Kemenkes menyatakan akan mengatu… | `[SRC-06]` | PRD §10 |  |  | BRD v1.0 |
| BR-120 | PT. Arunika menyediakan aplikasi RME minimalis, terpisah dari aplikasi dan portal Rumah Sehat, untuk kli… | `[USER] T3-Q1a, T3-Q14, T3-Q15` | US-1301, US-1302 |  |  | BRD v1.2 (CR-006/Tahap 3) |
| BR-121 | RME minimalis terhubung ke Rumah Sehat lewat integrasi yang sama dengan sistem rekanan lain (BR-070). Ru… | `[USER] T3-Q14` | US-1301, US-1401 |  |  | BRD v1.2 (CR-006/Tahap 3) |
| BR-122 | RME minimalis menyediakan pengelolaan jadwal, daftar layanan, dan harga klinik/lab, termasuk layanan hom… | `[USER] T3-Q1b` | US-1303 |  |  | BRD v1.2 (CR-006/Tahap 3) |
| BR-123 | RME minimalis menyediakan pencatatan rekam medis kunjungan. Data rekam medis sesuai ketentuan SATUSEHAT… | `[USER] T3-Q1b` `[SRC-18]` `[SRC-26]` `[SRC-27]` | US-1306, US-1309, US-1310, PRD §10 |  |  | BRD v1.2 (CR-006/Tahap 3) |
| BR-124 | RME minimalis menyediakan pengisian dan unggah hasil pemeriksaan lab | `[USER] T3-Q1b` | US-1307 |  |  | BRD v1.2 (CR-006/Tahap 3) |
| BR-125 | RME minimalis menyediakan penerimaan appointment dari Rumah Sehat, check-in pasien, antrean real-time kl… | `[USER] T3-Q1b, T2-Q13` | US-1304, US-1305, US-1308 |  |  | BRD v1.2 (CR-006/Tahap 3) |
| BR-126 | RME minimalis hanya mencatat pasien yang memesan layanan lewat aplikasi Rumah Sehat | `[USER] T3-Q1c` | US-1306, US-1310 |  |  | BRD v1.2 (CR-006/Tahap 3) |

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
| EP-01 | US-101 A01; US-103 A01; US-104 A01, A13; US-105 A03; US-106 A03; US-107 A02, A03, A05; US-108 A04; US-109 A04 |
| EP-02 | US-201 C02; US-202 B01; US-203 C02 (pola); US-204 — (B01 peringatan SIP); US-205 D-C04; US-206 D-C04, C01; US-207 D-C05; US-208 D-C05, C01 |
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
| NFR | NFR-SEC-02 C01, B01; NFR-SEC-04 C01; NFR-SEC-08 A03, A04, B01; NFR-SEC-10 B01; PRD §6.11 aksesibilitas → 06-design-guideline.md §7 (KEPUTUSAN-160) |
