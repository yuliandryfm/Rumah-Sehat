# BRD (Business Requirements Document): Rumah Sehat

Tanggal: 2026-11-17
Versi: 1.3 (revisi CR-007 dari Tahap 4: semua penyedia pihak ketiga memproses data di Indonesia; login Google/Apple, push, dan WhatsApp dihapus; disetujui user 2026-11-18 `[USER] T4-Q2`, disetujui di gate Tahap 4, 2026-11-18 `[USER] T4-G5`). Versi 1.2 (revisi CR-006 dari wawancara Tahap 3, disetujui bersama PRD di gate Tahap 3, 2026-11-17 `[USER] T3-Gate`). Versi 1.1 (revisi CR-005) disetujui di gate revisi 2026-11-17 `[USER] T2-Gate-CR005`. Versi 1.0 disetujui di gate Tahap 2, 2026-11-17
Penyusun: Claude (persona Business Analyst + Compliance Officer); review kepatuhan oleh subagent `compliance-reviewer`
Pemilik dan penyetuju: user (project manager, meneruskan ke CEO dan CTO) `[DOK: 00-project-brief.md]`
Tingkat ketelitian: **T1 Kritis** untuk bagian 6 (kepatuhan dan data pribadi/kesehatan), bagian 5.5 (alur uang), dan bagian 5.11 (rekam medis, CR-006); **T2 Standar** untuk bagian lain.

Dokumen ini mendefinisikan *apa* yang dibutuhkan bisnis, belum *bagaimana* membangunnya. Rincian alur, layar, dan acceptance criteria disusun di PRD (Tahap 3).

**Label sumber.** `[DOK] Px.x` merujuk ke `docs/00-project-brief.md` beserta kode pertanyaan intake di `docs/00-catatan-intake.md`. `[DOK: 01-business-case.md]` merujuk ke Business Case (lulus gate Tahap 1); `T1-Qx` ke `docs/01-catatan-business-case.md`. `[USER] T2-Qx` merujuk ke wawancara Tahap 2 di `docs/02-catatan-brd.md`; `[USER] T3-Qx` ke wawancara Tahap 3 di `docs/03-catatan-prd.md`. `[SRC-xx]` tercatat di `docs/sources.md`. Hasil riset kepatuhan lengkap: `docs/compliance/tahap-02-riset.md`.

**Catatan hukum.** Bagian kepatuhan disusun dari sumber resmi yang benar-benar dibuka. Sebagian regulasi kesehatan hanya terbaca metadata atau pernyataan resminya, sehingga temuannya berstatus **indikasi** dan wajib dikonfirmasi tim legal manusia (bagian 6.3). BRD ini bukan nasihat hukum.

## 1. Tujuan bisnis dan KPI

### 1.1 Tujuan bisnis

| ID | Tujuan | Sumber |
| --- | --- | --- |
| TB-01 | Memberi pasien yang kesulitan datang ke klinik atau apotek akses ke konsultasi online dan home visit tenaga kesehatan | `[DOK] P1.2` |
| TB-02 | Memperluas pasar klinik dan lab rekanan ke pengguna aplikasi | `[DOK] P1.2` |
| TB-03 | Memberi calon pasien informasi biaya MCU dan pemeriksaan lab sebelum datang | `[DOK] P1.2` |
| TB-04 | Menjadi *one stop solution* bagi pasien yang mobilitasnya terbatas, termasuk pembelian produk kesehatan dan kecantikan | `[DOK] P1.4, PS1` |

### 1.2 KPI (12 bulan setelah go-live, tenggat 1 Maret 2028)

KPI diambil dari Business Case `[DOK: 01-business-case.md]` §4.1. Definisi pengukuran dilengkapi di Tahap 2 (PT-16 terjawab).

| ID | KPI | Target | Definisi pengukuran | Sumber |
| --- | --- | --- | --- | --- |
| KPI-01 | Pasien aktif bulanan (MAU) | 10.000 | **Angka resmi:** pasien unik yang menyelesaikan minimal satu layanan (konsultasi, pendaftaran klinik/lab, home visit, atau pembelian produk) dalam satu bulan kalender. **Angka pendukung (dilaporkan, bukan target):** pasien unik yang membuka aplikasi dalam keadaan login minimal sekali dalam sebulan kalender | `[DOK: 01-business-case.md]` `[USER] T2-Q10` |
| KPI-02 | Dokter dan psikolog (gabungan) | 70 | Jumlah dokter dan psikolog yang **terverifikasi dan siap melayani**, sama dengan syarat go-live `[DOK] P8.2` | `[DOK: 01-business-case.md]` `[USER] T2-Q11` |
| KPI-03 | Klinik rekanan | 20 | Jumlah klinik rekanan yang **terintegrasi** | `[DOK: 01-business-case.md]` `[USER] T2-Q11` |
| KPI-04 | Lab rekanan | 10 | Jumlah lab rekanan yang **terintegrasi** | `[DOK: 01-business-case.md]` `[USER] T2-Q11` |

### 1.3 Syarat go-live 1 Maret 2027

Tidak berubah dari brief `[DOK] P8.2, P8.3`: minimal 2 klinik, 1 lab, dan 3 outlet rekanan terintegrasi; minimal 5 dokter dan 1 psikolog terverifikasi; ketujuh layanan berjalan; tayang di Play Store dan App Store; izin dan kepatuhan regulasi terpenuhi; lulus audit keamanan; sign-off UAT CEO, CTO, dan konsultan medis. Tanggal tetap. Scope boleh dikurangi, kecuali syarat izin/kepatuhan, audit keamanan, dan UAT. Kebutuhan kepatuhan yang menjadi isi syarat izin dirinci di bagian 6.

Klinik dan lab yang memakai RME minimalis (BR-120) terhubung ke Rumah Sehat lewat integrasi, sehingga terhitung "terintegrasi" untuk syarat go-live 1 dan KPI-03/KPI-04. Ini konsekuensi jawaban `[USER] T3-Q14, T3-Q15`, dikonfirmasi di gate Tahap 3 `[USER] T3-Gate`.

## 2. Stakeholder dan peran pengguna

### 2.1 Stakeholder

| Stakeholder | Kepentingan | Sumber |
| --- | --- | --- |
| PT. Arunika (pemilik dan pembiaya) | Pemilik produk; berperan sebagai **platform (Penyelenggara Sistem Elektronik)**, bukan fasyankes dan bukan penjual barang | `[DOK: 01-business-case.md]` `[USER] T2-Q2, T2-Q3` |
| CEO, CTO | Penyetuju dokumen gate lewat user (PM) | `[DOK] P2.3, P2.4a` |
| Konsultan medis | Penyetuju saat UAT dan go-live | `[DOK] P2.4b` |
| Klinik rekanan (fasyankes) | Menaungi semua tenaga kesehatan yang melayani di Rumah Sehat; menerima pendaftaran dan home visit; dapat menjual obat bebas | `[DOK] PS6a` `[USER] T2-Q2, T2-Q3b` |
| Lab rekanan | Menerima pendaftaran; menyediakan harga, jadwal, dan hasil lab | `[DOK] PS6a, P4.1` |
| Outlet rekanan | Toko kecantikan, toko alat kesehatan, klinik rekanan, dan apotek rekanan yang menjual produk | `[DOK] PS2` `[USER] T2-Q3b` |
| Penyedia pihak ketiga | Payment gateway berizin BI, layanan video, kurir, email. Belum dipilih. Semua wajib memproses dan menyimpan data di Indonesia; WhatsApp, push, dan login Google/Apple dihapus (CR-007) | `[DOK] P4.2` `[USER] T2-Q4` `[USER] T4-Q1, T4-Q2` |
| Regulator dan auditor eksternal | Pihak yang menerima pembuktian kepatuhan. Instansi yang teridentifikasi dari riset ada di bagian 6.1 | `[DOK] P5.3` |

### 2.2 Peran pengguna (9 peran)

| Peran | Kebutuhan utama | Platform | Sumber |
| --- | --- | --- | --- |
| Pasien | Konsultasi online, pendaftaran klinik/lab, home visit, info biaya MCU/lab, pembelian produk | Aplikasi mobile Android dan iOS | `[DOK] P1.2, P6.1` |
| Dokter | Konsultasi online (chat, video), catatan, resep digital | Aplikasi mobile tenaga kesehatan | `[DOK] PS6b, P3.4, P6.2` |
| Psikolog | Konsultasi kesehatan mental (chat, video), catatan | Aplikasi mobile tenaga kesehatan | `[DOK] P2.1, PS6b, P6.2` |
| Fisioterapis | Konsultasi online (chat, video) dan home visit | Aplikasi mobile tenaga kesehatan | `[DOK] P6.2` `[USER] T2-Q5` |
| Tenaga medis | Konsultasi online (chat, video) dan home visit | Aplikasi mobile tenaga kesehatan | `[DOK] P6.2` `[USER] T2-Q5` |
| Admin klinik | Portal: laporan transaksi dan pembagian dana, profil, keluhan | Portal web | `[DOK] PS6c, P6.2` |
| Admin lab | Sama seperti admin klinik, untuk lab rekanan | Portal web | `[DOK] PS6c, P6.2` |
| Admin outlet | Portal untuk toko kecantikan, toko alat kesehatan, dan apotek rekanan: laporan, pembagian dana, profil, keluhan | Portal web | `[DOK] PS6d, P6.2` `[USER] T2-Q3b` |
| Admin Rumah Sehat | Backoffice: verifikasi dan kelola akun, verifikasi KTP manual, laporan dan rekonsiliasi dana, CMS konten, layanan pelanggan, pengaturan markup dan biaya rekanan | Portal web | `[DOK] P4.6b, P8.2a, P6.2` `[USER] T2-Q1, T2-Q6` |

Semua dokter, psikolog, fisioterapis, dan tenaga medis berpraktik **atas nama klinik rekanan** `[USER] T2-Q2`. Tenaga kesehatan "mitra langsung" Rumah Sehat tanpa fasyankes tidak ada lagi (CR-001). Apoteker tetap tidak dipakai di rilis pertama `[DOK] P3.5, PS3`. Pesanan obat bebas di klinik rekanan dikelola admin klinik `[DOK] PS3`; pembagian fitur portal admin klinik dan admin outlet untuk klinik yang menjual produk ditetapkan di PRD.

Aplikasi RME minimalis (BR-120, CR-006) dipakai staf klinik dan lab rekanan yang belum punya sistem `[USER] T3-Q15`. Peran rincinya, termasuk operator PT. Arunika untuk onboarding, diusulkan di PRD (UP-26) dan disetujui di gate Tahap 3 `[USER] T3-Gate`.

## 3. Proses bisnis

### 3.1 Kondisi saat ini (*as-is*)

Kondisi saat ini belum diketahui pasti `[DOK] P1.3`. Yang diketahui:

- Sudah ada aplikasi sejenis (Halodoc) `[DOK] P1.3`.
- Sebagian orang menghubungi klinik langsung lewat telepon atau WhatsApp `[DOK] P1.3`.
- Belum ada rekanan yang bergabung, sehingga proses dan sistem internal klinik, lab, dan outlet calon rekanan `[TIDAK DIKETAHUI]` (PT-05) `[DOK] P4.3`.

### 3.2 Kondisi yang diinginkan (*to-be*)

Ringkasan tingkat bisnis. Alur rinci, status, dan pengecualian disusun di PRD.

| Proses | Alur *to-be* | Sumber |
| --- | --- | --- |
| P-01 Registrasi dan verifikasi pasien | Pasien mendaftar dengan email + password (login Google/Apple dihapus oleh CR-007), memberikan persetujuan pemrosesan data, dan mengisi data diri. Sebelum memakai konsultasi, home visit, atau pendaftaran klinik/lab, pasien mengunggah NIK dan foto KTP, lalu admin Rumah Sehat memverifikasinya secara manual (disetujui/ditolak) | `[DOK] P4.4, P4.6a, P4.6b` `[SRC-15]` |
| P-02 Onboarding rekanan dan tenaga kesehatan | Klinik, lab, dan outlet bergabung; admin Rumah Sehat memverifikasi akun dan izin rekanan; sistem rekanan diintegrasikan. Tenaga kesehatan didaftarkan di bawah klinik rekanan dan diverifikasi admin Rumah Sehat, termasuk STR/SIP | `[DOK] P8.2a, PS6a` `[USER] T2-Q2, T2-Q3b` `[SRC-02]` `[SRC-21]` |
| P-03 Konsultasi online | Pasien memilih tenaga kesehatan (dokter, psikolog, fisioterapis, tenaga medis) dan jadwalnya, melihat harga (tarif rekanan + markup, ditambah biaya transaksi untuk metode non-QRIS), membayar di aplikasi, lalu berkonsultasi lewat chat atau video di Rumah Sehat. Tenaga kesehatan menulis catatan; dokter dapat menerbitkan resep digital yang ditebus pasien di apotek mana pun. Payment gateway membagi dana langsung ke rekening klinik rekanan dan Rumah Sehat | `[DOK] P3.2, P3.4, PS6b` `[USER] T2-Q2, T2-Q4, T2-Q5, T2-Q6a, T2-Q8, T2-Q12` `[SRC-24]` |
| P-04 Pendaftaran klinik dan lab | Pasien melihat jadwal dan harga layanan klinik atau lab (dari sistem rekanan), mendaftar, dan membayar di aplikasi (harga rekanan + markup, ditambah biaya transaksi untuk metode non-QRIS). Saat datang, pasien cukup menunjukkan appointment di aplikasi. Tindakan tambahan yang tidak tercantum di aplikasi diselesaikan dan dibayar di luar aplikasi. Payment gateway membagi dana langsung ke rekening rekanan dan Rumah Sehat. Hasil lab dari lab rekanan masuk ke Rumah Sehat dan pasien diberi tahu | `[DOK] PS6a, P4.1, P4.5` `[USER] T2-Q4, T2-Q8, T2-Q12, T2-Q13, T2-Q14, T2-Q15, T2-Q19` `[SRC-24]` (CR-005) |
| P-05 Home visit | Pasien memesan home visit yang disediakan klinik rekanan (data dari sistem rekanan) dan membayar di aplikasi (harga rekanan + markup, ditambah biaya transaksi untuk metode non-QRIS). Kunjungan dilakukan oleh tenaga kesehatan klinik itu, termasuk fisioterapis dan tenaga medis. Tindakan tambahan yang tidak tercantum di aplikasi diselesaikan dan dibayar di luar aplikasi | `[DOK] P1.2, PS6a` `[USER] T2-Q5, T2-Q8, T2-Q12, T2-Q13, T2-Q14, T2-Q15, T2-Q19` `[SRC-24]` (CR-005) |
| P-06 Informasi biaya MCU dan lab | Pasien melihat biaya MCU dan pemeriksaan lab berdasarkan harga dari sistem rekanan. Biaya yang ditampilkan sama dengan harga yang dibayar bila mendaftar di aplikasi (harga rekanan + markup) | `[DOK] P1.2, PS6a` `[USER] T2-Q15, T2-Q20` (CR-005) |
| P-07 Pembelian produk | Pasien memilih produk dari katalog outlet rekanan (dari sistem rekanan), membayar di aplikasi (harga rekanan + markup + ongkir, ditambah biaya transaksi untuk metode non-QRIS), outlet memproses pesanan, kurir pihak ketiga mengirim. Payment gateway membagi dana langsung ke rekening outlet dan Rumah Sehat | `[DOK] PS1, PS4, PS5, PS6` `[USER] T2-Q4, T2-Q6a, T2-Q7, T2-Q8, T2-Q12` `[SRC-24]` |
| P-08 Refund | Refund diberikan bila layanan gagal (contoh: tenaga kesehatan tidak hadir, barang tidak dikirim atau rusak). Untuk layanan terjadwal (konsultasi online, pendaftaran klinik, pendaftaran lab, home visit), pembatalan oleh pasien sebelum batas waktu tertentu juga di-refund. Pembatalan pembelian produk oleh pasien setelah membayar tidak di-refund | `[USER] T2-Q9, T2-Q17, T2-Q18` (CR-005) |
| P-09 Operasional backoffice | Admin Rumah Sehat memverifikasi akun, mengelola konten, menangani layanan pelanggan, mengatur markup dan biaya rekanan, serta memantau dan merekonsiliasi pembagian dana | `[DOK] P8.2a` `[USER] T2-Q1, T2-Q4, T2-Q6` |
| P-10 Permintaan hak data pasien | Pasien mengajukan permintaan akses, perbaikan, penghapusan, atau penarikan persetujuan; Rumah Sehat menanggapi dalam batas waktu yang diatur | `[DOK] P5.4` `[SRC-15]` |

## 4. Scope dan non-scope rilis pertama

### 4.1 In-scope

| # | Layanan / kemampuan | Sumber |
| --- | --- | --- |
| S-01 | Konsultasi online dokter (chat teks dan video) | `[DOK] P3.1, P3.2` |
| S-02 | Konsultasi kesehatan mental dengan psikolog (chat teks dan video) | `[DOK] P2.1, P3.1, P3.2` |
| S-03 | Konsultasi online fisioterapis dan tenaga medis (chat teks dan video) | `[USER] T2-Q5` |
| S-04 | Pendaftaran klinik rekanan | `[DOK] P3.1` |
| S-05 | Pendaftaran lab rekanan, termasuk penerimaan hasil lab | `[DOK] P3.1, P4.1` |
| S-06 | Home visit oleh tenaga kesehatan klinik rekanan, termasuk fisioterapis dan tenaga medis | `[DOK] P1.2, P3.1` `[USER] T2-Q5` |
| S-07 | Informasi biaya MCU dan pemeriksaan lab | `[DOK] P1.2, P3.1` |
| S-08 | Pembelian obat bebas (dari apotek dan klinik rekanan berizin), produk kecantikan, dan produk kesehatan lain (vitamin, suplemen, alat kesehatan) dari outlet rekanan | `[DOK] PS1` `[USER] T2-Q3, T2-Q3b` |
| S-09 | Resep digital oleh dokter, ditebus pasien di apotek mana pun di luar aplikasi | `[DOK] P3.4` |
| S-10 | Pembayaran di aplikasi untuk semua layanan yang dipesan di aplikasi (konsultasi, pendaftaran klinik dan lab, home visit, produk) lewat payment gateway berizin BI, dengan pembagian dana langsung ke rekening rekanan. Bagian klinik, lab, dan home visit di brief P3.3 diganti CR-005 | `[DOK] P3.3, PS4` `[USER] T2-Q4, T2-Q13` (CR-005) |
| S-11 | Integrasi otomatis ke sistem klinik, lab, dan outlet rekanan | `[DOK] PS6a` |
| S-12 | Backoffice admin Rumah Sehat (verifikasi dan kelola akun, laporan dan rekonsiliasi dana, CMS termasuk konten edukasi, layanan pelanggan) | `[DOK] P8.2, P8.2a` |
| S-13 | Portal admin klinik, admin lab, dan admin outlet | `[DOK] PS6c, PS6d` |
| S-14 | Notifikasi email dan notifikasi di dalam aplikasi. Push dan WhatsApp dihapus (CR-007) | `[DOK] P4.5` `[USER] T4-Q1, T4-Q2` |
| S-15 | Audit trail aktivitas sensitif dan transaksi | `[DOK] P5.4` |
| S-16 | Kemampuan kepatuhan pelindungan data pribadi (persetujuan, hak subjek data, retensi) | `[SRC-15]` |
| S-17 | Aplikasi RME minimalis untuk klinik dan lab rekanan yang belum punya sistem, terpisah dari Rumah Sehat dan terhubung lewat integrasi (CR-006) | `[USER] T3-Q1a, T3-Q1b, T3-Q1c, T3-Q14, T3-Q15` |
| S-18 | Antrean real-time klinik (CR-006) | `[USER] T3-Q1a` |
| S-19 | Pemesanan layanan untuk anggota keluarga | `[USER] T3-Q3, T3-Q3a` |

### 4.2 Non-scope rilis pertama

| Butir | Sumber |
| --- | --- |
| Integrasi BPJS dan asuransi swasta | `[DOK] P3.6` |
| Penjualan obat resep | `[DOK] PS1` |
| Peran apoteker | `[DOK] P3.5, PS3` |
| Konsultasi lewat panggilan suara | `[DOK] P3.2` |
| Perekaman atau penyimpanan panggilan video | `[DOK] P4.1` |
| Langganan atau membership berbayar bagi pasien | `[DOK] P3.6a` |
| Chatbot AI atau pemeriksa gejala otomatis | `[DOK] P3.6a` |
| Versi web untuk pasien | `[DOK] P3.6a` |
| Integrasi perangkat kesehatan atau wearable | `[DOK] P3.6a` |
| Pengantaran sampel lab atau pengambilan sampel di rumah | `[DOK] P3.6a` |
| Login pasien dengan nomor HP + OTP; notifikasi SMS | `[DOK] P4.4, P4.5` |
| Login dengan akun Google atau Apple; notifikasi push; notifikasi WhatsApp (dihapus, CR-007) | `[USER] T4-Q1, T4-Q2` |
| Tenaga kesehatan "mitra langsung" Rumah Sehat tanpa fasyankes rekanan (diganti, CR-001) | `[USER] T2-Q2` |
| Penampungan dana pasien di rekening PT. Arunika lalu diteruskan sendiri ke mitra (diganti, CR-003) | `[USER] T2-Q4` |
| Model komisi berupa potongan dari harga rekanan (diganti markup, CR-004) | `[USER] T2-Q6a` |
| Refund atas pembatalan pembelian produk oleh pasien setelah membayar, dan atas pembatalan layanan terjadwal setelah batas waktu pembatalan | `[USER] T2-Q9, T2-Q17, T2-Q18` |
| Pembayaran tindakan tambahan yang tidak tercantum di aplikasi (diselesaikan dan dibayar di luar aplikasi) | `[USER] T2-Q14` |
| Pembayaran klinik, lab, dan home visit langsung ke mitra (diganti pembayaran di aplikasi, CR-005) | `[USER] T2-Q13` |
| Katalog produk milik Rumah Sehat sendiri (katalog dari sistem rekanan) | `[DOK] PS6a` |
| RME lengkap untuk operasional klinik dan lab (diganti RME minimalis, CR-006) | `[USER] T3-Q1a` |
| RME minimalis untuk pasien yang tidak datang lewat Rumah Sehat | `[USER] T3-Q1c` |
| Program loyalitas, poin, dan voucher | `[USER] T3-Q1a` (tidak dipilih) |
| Multi-bahasa | `[USER] T3-Q1a` (tidak dipilih) |
| Rekam medis, check-in, antrean, hasil lab, dan home visit yang diisi di portal Rumah Sehat (portal rekanan hanya untuk pelaporan dan rekap, ditambah menu pengaturan konsultasi online) | `[USER] T3-Q14, T3-Q16` |

### 4.3 Belum diputuskan (diputuskan di Tahap 3)

Keempat butir `[DOK] P3.6a` (PT-13) diputuskan di Tahap 3 (CR-006): RME minimalis sebagai aplikasi terpisah dan antrean real-time klinik masuk rilis 1 (S-17, S-18); RME lengkap, program loyalitas/poin/voucher, dan multi-bahasa tidak masuk (§4.2) `[USER] T3-Q1, T3-Q1a, T3-Q1b, T3-Q1c, T3-Q15`.

## 5. Kebutuhan bisnis

Penomoran dikelompokkan per area dan sengaja diberi celah agar kebutuhan baru bisa disisipkan tanpa menomori ulang. Prioritas (Must/Should/Could) dan urutan pengurangan scope (P8.3) ditetapkan di PRD.

### 5.1 Akun, identitas, dan peran

| ID | Kebutuhan bisnis | Sumber |
| --- | --- | --- |
| BR-001 | Pasien dapat mendaftar dan masuk dengan email + password. Login Google/Apple dihapus karena lokasi pemrosesannya tidak dapat dipastikan di Indonesia (CR-007) | `[DOK] P4.4` `[USER] T4-Q1, T4-Q2` |
| BR-002 | Rumah Sehat menyimpan data identitas dasar pasien (nama, tanggal lahir, jenis kelamin, nomor HP, email, alamat) dan data anggota keluarga pasien. Cara data anggota keluarga dipakai: BR-008 (PT-26 terjawab) | `[DOK] P4.1` `[USER] T3-Q3` |
| BR-003 | NIK dan foto KTP pasien wajib sebelum memakai konsultasi online, home visit, serta pendaftaran klinik dan lab. Tidak wajib untuk pembelian produk | `[DOK] P4.6a` |
| BR-004 | Admin Rumah Sehat memverifikasi NIK dan foto KTP secara manual (disetujui atau ditolak) sebelum layanan yang mewajibkannya dapat dipakai | `[DOK] P4.6b` |
| BR-005 | Akun dokter, psikolog, fisioterapis, dan tenaga medis diverifikasi dan dikelola admin Rumah Sehat. Setiap tenaga kesehatan terhubung ke klinik rekanan yang menaunginya | `[DOK] P8.2a` `[USER] T2-Q2` |
| BR-006 | Akun rekanan (klinik, lab, toko kecantikan, toko alat kesehatan, apotek) dan admin mitranya diverifikasi dan dikelola admin Rumah Sehat, termasuk pemeriksaan izin usaha rekanan saat bergabung | `[DOK] P8.2a` `[USER] T2-Q3, T2-Q3b` |
| BR-007 | Aplikasi pasien tersedia di Android dan iOS; tenaga kesehatan memakai aplikasi mobile yang juga tersedia di Android dan iOS (PT-08 terjawab); para admin memakai portal web | `[DOK] P6.1, P6.2` `[USER] T3-Q9` |
| BR-008 | Pasien dapat memesan konsultasi online, pendaftaran klinik/lab, dan home visit untuk anggota keluarga yang terdaftar di profilnya. Identitas anggota keluarga diverifikasi manual admin Rumah Sehat: dewasa dengan NIK dan foto KTP, anak dengan NIK dan foto Kartu Keluarga yang memuat pemilik akun sebagai orang tua/wali. Pemilik akun menyatakan telah mendapat persetujuan anggota keluarga dewasa, atau memberi persetujuan sebagai orang tua/wali untuk anak. Kecukupan cara persetujuan ini dikonfirmasi legal (L-08) | `[USER] T3-Q3, T3-Q3a` |

### 5.2 Konsultasi online

| ID | Kebutuhan bisnis | Sumber |
| --- | --- | --- |
| BR-010 | Pasien dapat berkonsultasi dengan dokter lewat chat teks dan panggilan video | `[DOK] P3.1, P3.2` |
| BR-011 | Pasien dapat berkonsultasi kesehatan mental dengan psikolog lewat chat teks dan panggilan video | `[DOK] P2.1, P3.1, P3.2` |
| BR-012 | Pasien dapat berkonsultasi dengan fisioterapis dan tenaga medis lewat chat teks dan panggilan video | `[USER] T2-Q5` |
| BR-013 | Konsultasi berlangsung di Rumah Sehat. Tenaga kesehatan memakai aplikasi tenaga kesehatan untuk chat, video, dan catatan konsultasi | `[DOK] PS6b` |
| BR-014 | Setiap konsultasi diberikan oleh tenaga kesehatan atas nama klinik rekanan yang menaunginya; PT. Arunika berperan sebagai platform | `[USER] T2-Q2` |
| BR-015 | Dokter dapat menerbitkan resep digital setelah konsultasi. Pasien menebusnya sendiri di apotek mana pun; obat resep tidak dijual di Rumah Sehat | `[DOK] P3.4, PS1` |
| BR-016 | Isi chat konsultasi (termasuk foto), catatan tenaga kesehatan, dan resep disimpan. Panggilan video tidak direkam atau disimpan | `[DOK] P4.1` |
| BR-017 | Aplikasi tersedia 24 jam, 7 hari. Ketersediaan tenaga kesehatan mengikuti jadwal masing-masing. Target ketersediaan 99,0% per bulan, cara ukurnya di PRD (PT-12 terjawab) | `[DOK] P6.4` `[USER] T3-Q12` |
| BR-018 | Tarif konsultasi berasal dari rekanan. Tarif dan durasi sesi diisi admin klinik di portal; jadwal praktik online diatur tenaga kesehatan di aplikasinya dan dapat diubah admin klinik di portal | `[USER] T2-Q6, T3-Q4, T3-Q16` |

### 5.3 Klinik, lab, home visit, dan informasi biaya

| ID | Kebutuhan bisnis | Sumber |
| --- | --- | --- |
| BR-020 | Pasien dapat mendaftar ke klinik rekanan berdasarkan jadwal, data klinik, dan harga layanan dari sistem rekanan | `[DOK] P3.1, PS6a` `[USER] T2-Q13` |
| BR-021 | Pasien dapat mendaftar pemeriksaan di lab rekanan berdasarkan jadwal dan harga dari sistem rekanan | `[DOK] P3.1, PS6a` |
| BR-022 | Pasien dapat memesan home visit yang disediakan klinik rekanan, dilayani tenaga kesehatan klinik itu termasuk fisioterapis dan tenaga medis. Data home visit berasal dari sistem rekanan | `[DOK] P1.2, PS6a` `[USER] T2-Q5` |
| BR-023 | Pasien dapat melihat informasi biaya MCU dan pemeriksaan lab sebelum mendaftar, berdasarkan harga dari sistem rekanan. Biaya yang ditampilkan sama dengan harga yang dibayar bila mendaftar di aplikasi (harga rekanan + markup, BR-042) | `[DOK] P1.2, PS6a` `[USER] T2-Q20` |
| BR-024 | Pendaftaran klinik, pendaftaran lab, dan home visit yang dipesan di aplikasi dibayar di aplikasi. Saat datang ke klinik atau lab, pasien cukup menunjukkan appointment di aplikasi. Tindakan tambahan yang tidak tercantum di aplikasi diselesaikan dan dibayar di luar aplikasi (user menjawab untuk klinik; penerapan ke lab dan home visit adalah tafsiran penyusun atas pernyataan umum user, disetujui user di gate revisi `[USER] T2-Gate-CR005`). Menggantikan keputusan brief P3.3 (CR-005) | `[USER] T2-Q13, T2-Q14` |
| BR-025 | Hasil lab dari lab rekanan diterima lewat integrasi, disimpan, dan diberitahukan ke pasien | `[DOK] P4.1, P4.5, PS6a` |
| BR-026 | Riwayat pendaftaran klinik, lab, dan home visit pasien disimpan | `[DOK] P4.1` |
| BR-027 | Pasien yang sudah check-in di klinik rekanan dapat melihat antrean secara real-time, dari data antrean sistem klinik (termasuk RME minimalis) lewat integrasi (CR-006) | `[USER] T3-Q1a, T3-Q1b, T3-Q14` |

### 5.4 Pembelian produk

| ID | Kebutuhan bisnis | Sumber |
| --- | --- | --- |
| BR-030 | Pasien dapat membeli obat bebas, produk kecantikan, dan produk kesehatan lain (vitamin, suplemen, alat kesehatan) dari outlet rekanan | `[DOK] PS1` |
| BR-031 | Outlet rekanan terdiri atas toko kecantikan, toko alat kesehatan, klinik rekanan, dan apotek rekanan. Obat bebas hanya dijual oleh apotek dan klinik rekanan yang sudah berizin. Barang adalah milik rekanan | `[DOK] PS2` `[USER] T2-Q3, T2-Q3b` |
| BR-032 | Obat resep tidak dijual di Rumah Sehat | `[DOK] PS1` |
| BR-033 | Katalog, stok, dan pesanan produk berasal dari sistem outlet rekanan lewat integrasi. Rumah Sehat tidak menyimpan katalog sendiri | `[DOK] PS6, PS6a` |
| BR-034 | Produk dikirim kurir pihak ketiga yang terintegrasi. Ongkos kirim dibayar pasien dan ditambahkan ke total pembayaran | `[DOK] PS5` `[USER] T2-Q7` |
| BR-035 | Riwayat pesanan produk pasien disimpan | `[DOK] P4.1` |

### 5.5 Pembayaran dan pendapatan (T1)

| ID | Kebutuhan bisnis | Sumber |
| --- | --- | --- |
| BR-040 | Semua layanan yang dipesan di aplikasi (konsultasi online, pendaftaran klinik, pendaftaran lab, home visit, dan pembelian produk) dibayar di aplikasi melalui payment gateway berizin Bank Indonesia. Bagian klinik, lab, dan home visit di brief P3.3 diganti CR-005 | `[DOK] P3.3, PS4` `[USER] T2-Q4, T2-Q13` `[SRC-19]` |
| BR-041 | Payment gateway membagi dana setiap pembayaran langsung: bagian rekanan ke rekening rekanan, bagian Rumah Sehat ke rekening PT. Arunika. PT. Arunika tidak menampung dana bagian rekanan | `[USER] T2-Q4` |
| BR-042 | Harga yang dibayar pasien untuk konsultasi, pendaftaran klinik, pendaftaran lab, home visit, dan produk = harga dari rekanan + markup Rumah Sehat. Markup berbentuk persen atau nominal dan dapat diatur per rekanan oleh admin Rumah Sehat. Besarannya ditetapkan CEO sebelum go-live (PT-17) | `[USER] T2-Q1, T2-Q6, T2-Q6a, T2-Q15` |
| BR-043 | Biaya dari rekanan (langganan atau biaya per pendaftaran yang ditagihkan ke klinik dan lab) dapat diatur admin Rumah Sehat. Dipakai atau tidak, bentuk, dan besarannya diputuskan CEO sebelum go-live (PT-17) | `[DOK: 01-business-case.md]` `[USER] T1-Q1, T2-Q1, T2-Q16` |
| BR-044 | Biaya MDR untuk pembayaran QRIS ditanggung rekanan (dipotong dari bagian rekanan) dan tidak dibebankan ke pasien. Biaya transaksi metode pembayaran lain dibebankan ke pasien dan ditampilkan di total pembayaran, menunggu konfirmasi legal per metode (L-12). Aturan ini berlaku untuk semua layanan yang dibayar di aplikasi, termasuk pendaftaran klinik, lab, dan home visit (CR-005) | `[USER] T2-Q8, T2-Q12, T2-Q19` `[SRC-24]` |
| BR-045 | Refund diberikan bila layanan gagal terjadi (contoh: tenaga kesehatan tidak hadir, barang tidak dikirim atau rusak). Untuk layanan terjadwal (konsultasi online, pendaftaran klinik, pendaftaran lab, home visit), pembatalan oleh pasien sebelum batas waktu tertentu juga di-refund; pembatalan setelah batas waktu itu tidak di-refund. Pembatalan pembelian produk oleh pasien setelah membayar tidak di-refund. Daftar kondisi, batas waktu pembatalan, perlakuan pasien yang tidak datang (no-show), pihak yang memutuskan, dan batas waktu refund ditetapkan di PRD §3.5 dengan jawaban user T3-Q5 sampai T3-Q8 | `[USER] T2-Q9, T2-Q17, T2-Q18, T3-Q5, T3-Q6, T3-Q7, T3-Q8` |
| BR-046 | Admin Rumah Sehat, admin klinik, admin lab, dan admin outlet dapat melihat laporan transaksi dan pembagian dana sesuai haknya; admin Rumah Sehat memantau dan merekonsiliasi pembagian dana oleh payment gateway | `[DOK] P8.2a, PS6c, PS6d` `[USER] T2-Q4` |

### 5.6 Notifikasi

| ID | Kebutuhan bisnis | Sumber |
| --- | --- | --- |
| BR-050 | Rumah Sehat mengirim pemberitahuan ke pasien (konfirmasi, pengingat, status pesanan, hasil lab) lewat email dan notifikasi di dalam aplikasi, tanpa SMS. Push dan WhatsApp dihapus karena pemrosesannya tidak dapat dipastikan di Indonesia (CR-007) | `[DOK] P4.5` `[USER] T4-Q1, T4-Q2` |

### 5.7 Backoffice dan portal mitra

| ID | Kebutuhan bisnis | Sumber |
| --- | --- | --- |
| BR-060 | Backoffice admin Rumah Sehat menyediakan verifikasi dan pengelolaan akun (pasien, tenaga kesehatan, rekanan) | `[DOK] P8.2a` |
| BR-061 | Backoffice admin Rumah Sehat menyediakan CMS untuk banner, halaman info, artikel/konten edukasi kesehatan, dan promo | `[DOK] P8.2a` |
| BR-062 | Backoffice admin Rumah Sehat menyediakan layanan pelanggan, termasuk penanganan keluhan | `[DOK] P8.2a` |
| BR-063 | Backoffice admin Rumah Sehat menyediakan pengaturan markup per rekanan dan biaya rekanan | `[USER] T2-Q1, T2-Q6` |
| BR-064 | Portal admin klinik dan admin lab menyediakan laporan transaksi dan pembagian dana, profil, dan keluhan. Untuk klinik ditambah satu menu pengaturan konsultasi online (tarif, durasi, jadwal tenaga kesehatan, BR-018). Selain itu portal rekanan hanya untuk pelaporan dan rekap; profil dan keluhan hanya dapat dilihat, tanggapan dan perubahan lewat Layanan Pelanggan | `[DOK] PS6c` `[USER] T3-Q14, T3-Q16, T3-Gate G3` |
| BR-065 | Portal admin outlet (toko kecantikan, toko alat kesehatan, apotek) menyediakan laporan, pembagian dana, profil, dan keluhan. Profil dan keluhan hanya dapat dilihat, seperti BR-064 | `[DOK] PS6d` `[USER] T2-Q3b, T3-Gate G3` |

### 5.8 Integrasi

| ID | Kebutuhan bisnis | Sumber |
| --- | --- | --- |
| BR-070 | Rumah Sehat terintegrasi otomatis dengan sistem klinik, lab, dan outlet rekanan untuk jadwal, harga dan tarif, home visit, katalog, stok, pesanan, hasil lab, serta status lunas dan appointment yang dibayar di aplikasi (CR-005); mekanisme pencocokan appointment saat pasien datang ditetapkan di PRD dan Tahap 4; portal rekanan tidak dipakai untuk itu (T3-Q14). Klinik dan lab tanpa sistem memakai RME minimalis yang terhubung lewat integrasi yang sama (BR-121). Kemampuan integrasi sistem rekanan `[TIDAK DIKETAHUI]` (PT-05) | `[DOK] PS6a, P4.3` `[USER] T2-Q6, T2-Q13, T3-Q14` |
| BR-071 | Rumah Sehat terintegrasi dengan payment gateway (dengan pembagian dana), layanan panggilan video, kurir pihak ketiga, dan email. Semua penyedia wajib memproses dan menyimpan data di Indonesia; WhatsApp, notifikasi push, dan login Google/Apple dihapus (CR-007). Penyedia dipilih di Tahap 4–5 dengan persetujuan user | `[DOK] P4.2` `[USER] T2-Q4` `[USER] T4-Q1, T4-Q2` |

### 5.9 Audit trail

| ID | Kebutuhan bisnis | Sumber |
| --- | --- | --- |
| BR-080 | Audit trail append-only mencatat: (1) login, logout, dan percobaan login gagal; (2) akses ke data pasien (catatan medis, hasil lab, KTP); (3) pembuatan dan perubahan catatan tenaga kesehatan serta resep; (4) verifikasi KTP; (5) pembayaran, refund, dan pembagian dana ke mitra; (6) perubahan status pendaftaran, home visit, dan pesanan; (7) penambahan, perubahan, dan penonaktifan akun tenaga kesehatan dan admin mitra; (8) perubahan hak akses atau peran; (9) ekspor atau unduh data; (10) permintaan pasien untuk melihat atau menghapus datanya; ditambah setiap transaksi. Perubahan pengaturan markup dan biaya rekanan diusulkan ikut dicatat (lihat catatan) | `[DOK] P5.4` `[DOK: adr/ADR-005-audit-trail-append-only.md]` |
| BR-081 | Audit trail tidak boleh memuat password, PIN, OTP, token, atau nomor kartu/rekening lengkap | `[DOK: adr/ADR-005-audit-trail-append-only.md]` |

Catatan: pencatatan perubahan pengaturan markup dan biaya rekanan (pengaturan baru dari T2-Q6 dan T2-Q1) adalah **usulan penyusun** berdasarkan ADR-005 ("setiap story yang menyentuh aksi sensitif wajib mencatat event"), karena pengaturan ini menentukan uang yang dibayar pasien. Tidak disebut user di P5.4; disetujui user di gate Tahap 2 `[USER] T2-Gate`.

### 5.10 Pengukuran KPI

| ID | Kebutuhan bisnis | Sumber |
| --- | --- | --- |
| BR-090 | Rumah Sehat menyediakan data untuk mengukur KPI-01 sampai KPI-04 sesuai definisi di bagian 1.2: MAU bertransaksi (resmi) dan MAU login (pendukung), jumlah dokter dan psikolog terverifikasi dan siap melayani, serta jumlah klinik dan lab terintegrasi | `[USER] T2-Q10, T2-Q11` `[DOK: 01-business-case.md]` |

### 5.11 RME minimalis (CR-006)

| ID | Kebutuhan bisnis | Sumber |
| --- | --- | --- |
| BR-120 | PT. Arunika menyediakan aplikasi RME minimalis, terpisah dari aplikasi dan portal Rumah Sehat, untuk klinik dan lab rekanan yang belum memiliki sistem, agar dapat bergabung dengan Rumah Sehat. Dibangun di proyek ini untuk rilis 1 | `[USER] T3-Q1a, T3-Q14, T3-Q15` |
| BR-121 | RME minimalis terhubung ke Rumah Sehat lewat integrasi yang sama dengan sistem rekanan lain (BR-070). Rumah Sehat tetap hanya platform yang mengintegrasikan pasien dengan rekanan | `[USER] T3-Q14` |
| BR-122 | RME minimalis menyediakan pengelolaan jadwal, daftar layanan, dan harga klinik/lab, termasuk layanan home visit dan paket MCU | `[USER] T3-Q1b` |
| BR-123 | RME minimalis menyediakan pencatatan rekam medis kunjungan. Data rekam medis sesuai ketentuan SATUSEHAT Kemenkes. PMK 24/2022 mewajibkan fasyankes menjalankan rekam medis elektronik yang terhubung dengan SATUSEHAT; SATUSEHAT memakai HL7 FHIR; penyedia sistem RME mendaftarkan sistemnya ke SATUSEHAT dan wajib memiliki nomor PSE | `[USER] T3-Q1b` `[SRC-18]` `[SRC-26]` `[SRC-27]` |
| BR-124 | RME minimalis menyediakan pengisian dan unggah hasil pemeriksaan lab | `[USER] T3-Q1b` |
| BR-125 | RME minimalis menyediakan penerimaan appointment dari Rumah Sehat, check-in pasien, antrean real-time klinik, dan penugasan home visit | `[USER] T3-Q1b, T2-Q13` |
| BR-126 | RME minimalis hanya mencatat pasien yang memesan layanan lewat aplikasi Rumah Sehat | `[USER] T3-Q1c` |

## 6. Kebutuhan kepatuhan (T1)

### 6.1 Regulasi dan instansi yang teridentifikasi

Hasil riset dari sumber resmi `[DOK: compliance/tahap-02-riset.md]`. Daftar ini menjawab sebagian PT-03. Apakah setiap kewajiban berlaku bagi Rumah Sehat wajib dikonfirmasi legal.

| Area | Regulasi / sumber | Instansi | Keyakinan | Sumber |
| --- | --- | --- | --- | --- |
| Pelindungan data pribadi | UU 27/2022 tentang PDP | Lembaga PDP (status pembentukan `[TIDAK DIKETAHUI]`) | Tinggi (teks pasal terbaca) | `[SRC-15]` `[SRC-16]` |
| Penyelenggara sistem elektronik | PP 71/2019; Permenkominfo 5/2020 | Komdigi | Tinggi | `[SRC-12]` `[SRC-13]` `[SRC-14]` |
| Telemedisin, rekam medis, perbekalan kesehatan | UU 17/2023; PP 28/2024; PMK 24/2022; Permenkes 5/2026; pernyataan Kemenkes; dokumentasi SATUSEHAT Platform (CR-006) | Kemenkes | Sedang–Rendah (teks pasal tidak terbaca; sebagian berupa rancangan). Dokumentasi SATUSEHAT terbaca langsung | `[SRC-02]` `[SRC-03]` `[SRC-04]` `[SRC-06]` `[SRC-07]` `[SRC-17]` `[SRC-18]` `[SRC-26]` `[SRC-27]` |
| Peredaran daring obat, kosmetik, suplemen | Peraturan BPOM 14/2024 jo. 30/2025 | BPOM | Tinggi untuk ruang lingkup; isi pasal `[TIDAK DIKETAHUI]` | `[SRC-10]` `[SRC-11]` |
| Platform farmasi (PSEF) | Portal PSEF Kemenkes (portal menautkan FAQ Permenkes 11/2025; isi FAQ tidak terbaca, sehingga dasar hukum syarat PSEF belum terverifikasi) | Kemenkes | Tinggi untuk daftar syarat; keberlakuan bagi Rumah Sehat `[TIDAK DIKETAHUI]` | `[SRC-08]` `[SRC-09]` |
| Sistem pembayaran | PBI 23/6/PBI/2021; ketentuan MDR QRIS | Bank Indonesia | Tinggi (ringkasan dan halaman resmi BI); ketentuan MDR non-QRIS `[TIDAK DIKETAHUI]` | `[SRC-19]` `[SRC-20]` `[SRC-24]` |
| Perdagangan melalui sistem elektronik | Permendag 19/2026; PP 80/2019 (status terkini belum diverifikasi) | Kemendag | Rendah (abstrak dan berita resmi Setkab) | `[SRC-22]` `[SRC-23]` `[SRC-25]` |
| Izin praktik tenaga kesehatan | STR/SIP; SIP diterbitkan pemerintah kabupaten/kota | Kemenkes, pemda | Sedang | `[SRC-21]` |

### 6.2 Kebutuhan bisnis kepatuhan

| ID | Kebutuhan bisnis | Sumber | Konfirmasi legal |
| --- | --- | --- | --- |
| BR-100 | Rumah Sehat terdaftar sebagai Penyelenggara Sistem Elektronik Lingkup Privat sebelum aplikasi digunakan pengguna (sebelum go-live). Aplikasi RME minimalis juga didaftarkan ke PSE untuk memperoleh nomor PSE yang menjadi prasyarat registrasi SATUSEHAT (CR-006) | `[SRC-12]` `[SRC-13]` `[SRC-27]` | Kategori PSE; apakah satu pendaftaran mencakup kedua sistem (L-02) |
| BR-101 | Konsultasi online berjalan dalam skema telemedisin yang sesuai ketentuan Kemenkes: klinik rekanan (fasyankes) sebagai penyelenggara pelayanan, PT. Arunika sebagai PSE terdaftar, beserta registrasi ke Kemenkes bila diwajibkan | `[SRC-02]` `[SRC-03]` `[USER] T2-Q2` | Ya, kritis (L-01) |
| BR-102 | Hanya tenaga kesehatan dengan STR dan SIP yang masih berlaku yang dapat diaktifkan melayani. Masa berlaku SIP dipantau, dan akun dinonaktifkan saat SIP berakhir. Kewajiban STR/SIP bagi psikolog `[TIDAK DIKETAHUI]` | `[SRC-02]` `[SRC-03]` `[SRC-21]` | Ya (L-07) |
| BR-103 | Pasien memberikan persetujuan eksplisit, tertulis atau terekam, sebelum datanya diproses, dengan informasi yang diwajibkan UU PDP (antara lain tujuan dan masa retensi). Bukti persetujuan disimpan | `[SRC-15]` | Dasar pemrosesan per jenis data (L-08) |
| BR-104 | Data anggota keluarga yang masih anak hanya diproses dengan persetujuan orang tua atau wali | `[SRC-15]` | Ya (L-08) |
| BR-105 | Data kesehatan (catatan tenaga kesehatan, isi chat konsultasi, resep, hasil lab), data anak, dan data keuangan pribadi diperlakukan sebagai data pribadi spesifik dengan perlindungan dan pembatasan akses yang lebih ketat | `[SRC-15]` | Pemetaan data (L-08) |
| BR-106 | Pasien dapat menjalankan hak subjek data: informasi, akses dan salinan, perbaikan, penarikan persetujuan, penghapusan, penundaan/pembatasan, keberatan, dan portabilitas. Permintaan yang diatur batas waktunya (antara lain akses dan perbaikan) ditanggapi paling lambat 3 x 24 jam | `[SRC-15]` `[DOK] P5.4` | Benturan hak hapus dengan retensi rekam medis (L-06) |
| BR-107 | Seluruh kegiatan pemrosesan data pribadi direkam, selaras dengan audit trail BR-080 | `[SRC-15]` | Tidak |
| BR-108 | Rumah Sehat menjaga keamanan dan kerahasiaan data pribadi yang diprosesnya | `[SRC-15]` `[SRC-13]` | Tidak |
| BR-109 | Tersedia prosedur pemberitahuan kegagalan pelindungan data pribadi secara tertulis paling lambat 3 x 24 jam kepada subjek data dan lembaga, serta masyarakat dalam hal tertentu | `[SRC-15]` `[SRC-13]` | Prosedur (L-08) |
| BR-110 | Penilaian dampak pelindungan data pribadi (DPIA) dilakukan sebelum go-live | `[SRC-15]` | Kewajiban bagi Rumah Sehat (L-08) |
| BR-111 | Rumah Sehat menunjuk pejabat/petugas pelindungan data pribadi | `[SRC-15]` `[SRC-16]` | Kewajiban bagi Rumah Sehat (L-08); penunjukan oleh CEO (PT-23) |
| BR-112 | Setiap jenis data memiliki masa retensi; setelah masa retensi berakhir, data diakhiri pemrosesannya, dihapus, atau dimusnahkan. Angka retensi `[TIDAK DIKETAHUI]` (PT-24). Masukan untuk legal: berita resmi Setkab tentang PP 80/2019 menyebut penyimpanan data PMSE terkait transaksi keuangan paling singkat 10 tahun dan lainnya paling singkat 5 tahun; status terkini PP itu belum diverifikasi | `[SRC-15]` `[SRC-25]` | Ya, terutama rekam medis (L-05) |
| BR-113 | Data disimpan di server atau cloud yang berlokasi di Indonesia. Penyedia pihak ketiga juga wajib memproses dan menyimpan data pribadi di Indonesia, tanpa kecuali (PT-22 terjawab `[USER] T4-Q1`) | `[DOK] P5.5` `[SRC-15]` `[SRC-13]` `[USER] T4-Q1` | Ya (L-09) |
| BR-114 | Catatan konsultasi dan resep digital memenuhi ketentuan rekam medis elektronik dan resep elektronik yang berlaku, termasuk integrasi dengan sistem Kemenkes bila diwajibkan. Status catatan konsultasi sebagai rekam medis, pengendalinya (klinik rekanan atau PT. Arunika), dan rinciannya `[TIDAK DIKETAHUI]` | `[SRC-18]` `[SRC-06]` `[SRC-07]` `[USER] T2-Q2` | Ya, kritis (L-05) |
| BR-115 | Pembayaran hanya diproses lewat penyedia jasa pembayaran berizin BI. Model pembagian dana langsung (BR-041) dikonfirmasi tidak memerlukan izin atau penetapan BI bagi PT. Arunika sebelum go-live | `[SRC-19]` `[SRC-20]` `[USER] T2-Q4` | Ya, kritis (L-04) |
| BR-116 | Obat bebas, produk kecantikan, dan produk kesehatan yang dijual lewat Rumah Sehat memenuhi ketentuan peredaran daring BPOM. Kewajiban PSEF bagi Rumah Sehat dikonfirmasi; bila wajib, tindak lanjutnya diputuskan CEO (PT-21) | `[SRC-08]` `[SRC-10]` `[SRC-11]` `[USER] T2-Q3` | Ya, kritis (L-03) |
| BR-117 | PT. Arunika memiliki perizinan berusaha yang sesuai untuk kegiatan Rumah Sehat sebelum go-live. Jenis izin, KBLI, dan perlu-tidaknya badan usaha terpisah `[TIDAK DIKETAHUI]` | `[SRC-09]` `[SRC-23]` | Ya (L-02) |
| BR-118 | Perkembangan regulasi kesehatan digital dipantau sampai go-live, karena Kemenkes menyatakan akan mengatur seluruh platform kesehatan digital termasuk tarif. Perubahan regulasi yang berdampak dicatat sebagai change request | `[SRC-06]` | Pengaturan tarif (L-10) |

### 6.3 Daftar konfirmasi legal

Wajib dijawab tim legal/compliance manusia. Rincian pertanyaan ada di `docs/compliance/tahap-02-riset.md` bagian (d).

| # | Topik | BR terkait | Dibutuhkan sebelum |
| --- | --- | --- | --- |
| L-01 | Skema telemedisin: PT. Arunika sebagai PSE dengan klinik rekanan sebagai penyelenggara; registrasi Kemenkes dan *regulatory sandbox* | BR-014, BR-101 | Tahap 4 (arsitektur peran dan data) |
| L-02 | Perizinan berusaha, KBLI, kategori PSE, perlu-tidaknya badan usaha terpisah (PT-02). Meluas karena CR-006: pendaftaran PSE untuk aplikasi RME minimalis (apakah satu pendaftaran cukup untuk dua sistem) dan perizinan PT. Arunika sebagai penyedia sistem RME | BR-100, BR-117, BR-120 | Tahap 12 (syarat go-live 5); lebih awal kalau memengaruhi biaya. Bagian PSE diusulkan dimajukan ke Tahap 4 karena menjadi prasyarat registrasi SATUSEHAT `[SRC-27]` (usulan review kepatuhan Tahap 3, disetujui di gate Tahap 3 `[USER] T3-Gate`) |
| L-03 | Obat bebas lewat platform: ketentuan BPOM dan kewajiban PSEF (apoteker penanggung jawab, jaringan apotek) (PT-04) | BR-031, BR-116 | Tahap 7 (agar urutan pengurangan scope jelas) |
| L-04 | Pembagian dana langsung oleh payment gateway, kini untuk semua layanan termasuk klinik, lab, dan home visit (CR-005): perlu-tidaknya izin atau penetapan BI bagi PT. Arunika; ketentuan refund; kebolehan membebankan biaya transaksi payment gateway ke pasien (termasuk per metode pembayaran) | BR-024, BR-040, BR-041, BR-044, BR-045, BR-115 | Tahap 4 |
| L-05 | Status catatan konsultasi sebagai rekam medis, pengendalinya, masa retensinya, SATUSEHAT, resep elektronik, rahasia kedokteran. Meluas karena CR-006: kewajiban PT. Arunika sebagai penyedia sistem RME minimalis, pengendali data rekam medis di RME minimalis (fasyankes atau PT. Arunika), kecukupan kelompok data minimal, dasar dan kewajiban pengiriman ke SATUSEHAT, validasi hasil lab, isi perjanjian pengendali/prosesor dan serah terima rekam medis saat kerja sama berakhir, serta akses pemilik akun ke data klinis anggota keluarga dewasa dari sisi rahasia kedokteran (rincian `docs/compliance/tahap-03.md` bagian 4) | BR-112, BR-114, BR-120–BR-126, BR-008 | Tahap 4 (model data) |
| L-06 | Hak hapus pasien terhadap kewajiban retensi rekam medis | BR-106 | Tahap 4 |
| L-07 | STR/SIP: status psikolog, kanal verifikasi resmi, SIP yang berlaku untuk telemedisin; kebolehan konsultasi daring oleh fisioterapis dan tenaga medis, serta profesi yang dimaksud "tenaga medis" | BR-012, BR-102 | Tahap 3 (alur verifikasi nakes). Belum terjawab di Tahap 3 karena tim legal belum ditunjuk (PT-20); PRD memakai dokumen wajib yang dapat diatur |
| L-08 | PDP: dasar pemrosesan per jenis data, DPIA, DPO, lembaga penerima notifikasi, data anggota keluarga/anak; apakah data transaksi yang memuat nama layanan klinik, pemeriksaan lab, atau home visit tergolong data kesehatan, dan batas data minimal yang boleh dikirim ke payment gateway (CR-005). Meluas karena BR-008: kecukupan pernyataan persetujuan anggota keluarga dewasa oleh pemilik akun terhadap kewajiban bukti persetujuan dari subjek, persetujuan orang tua/wali lewat Kartu Keluarga, akses pemilik akun ke data kesehatan anggota keluarga dewasa, cara anggota keluarga dewasa menjalankan haknya, batas usia anak, satu NIK di beberapa akun, minimisasi foto KK, pihak yang wajib DPIA atas data RME, dan dasar pengiriman data ke rekanan dan SATUSEHAT (rincian `docs/compliance/tahap-03.md` bagian 4) | BR-008, BR-103–BR-105, BR-109–BR-111, BR-040 | Tahap 4 |
| L-09 | Lokasi data sektor kesehatan; syarat transfer ke penyedia pihak ketiga di luar negeri | BR-113 | Tahap 4–5 (pemilihan penyedia) |
| L-10 | Pengaturan tarif pelayanan kesehatan digital atau markup platform oleh Kemenkes, untuk konsultasi, pendaftaran klinik, pendaftaran lab, dan home visit (CR-005), serta kebolehan menambahkan markup di atas tarif fasyankes rekanan. Peraturan yang sudah terbit tidak ditemukan; yang ada pernyataan rencana `[SRC-06]` | BR-024, BR-042, BR-118 | Tahap 12 |
| L-11 | Instansi pengawas dan auditor yang akan memeriksa Rumah Sehat (PT-03) | Bagian 6.1 | Tahap 10 |
| L-12 | Pembebanan biaya transaksi (MDR) per instrumen pembayaran (kartu, virtual account, e-wallet) ke pasien, dan siapa *merchant of record* dalam pembagian dana langsung, untuk semua layanan termasuk klinik, lab, dan home visit (CR-005). Kategori usaha merchant menentukan tarif MDR QRIS `[SRC-24]` | BR-044 | Tahap 4 |
| L-13 | Kesesuaian markup atas harga barang dan tarif rekanan (konsultasi, klinik, lab, home visit) dengan posisi PT. Arunika sebagai platform yang bukan penjual barang dan bukan fasyankes | BR-042, BR-031 | Tahap 4 |
| L-14 | Kebijakan refund (bila layanan gagal, ditambah pembatalan layanan terjadwal sebelum batas waktu), penyajian rincian harga, dan keterbukaan bahwa harga di aplikasi hanya mencakup layanan yang tercantum sedangkan tindakan tambahan dibayar di luar aplikasi, terhadap ketentuan perlindungan konsumen PMSE | BR-023, BR-024, BR-042, BR-044, BR-045 | Tahap 3 (aturan refund di PRD). Belum terjawab di Tahap 3 (PT-20); aturan refund ditetapkan user di PRD §3.5, dan perubahan dari legal dicatat sebagai change request |
| L-15 | Perjanjian pemrosesan data antara PT. Arunika dan setiap fasyankes pengguna RME minimalis, serta antara PT. Arunika dan klinik rekanan untuk data konsultasi (usulan review kepatuhan Tahap 3, disetujui di gate Tahap 3 `[USER] T3-Gate`) | BR-014, BR-120, BR-123 | Tahap 4 |

## 7. Asumsi, batasan, dan dependensi

### 7.1 Asumsi

Tidak ada asumsi atas fakta bisnis. Keputusan Tahap 2 yang bergantung pada konfirmasi legal (Q2 skema telemedisin, Q3 obat bebas, Q4 pembagian dana) adalah **keputusan user**, bukan asumsi penyusun, dan konfirmasi legalnya dicatat di bagian 6.3.

### 7.2 Batasan

| Batasan | Sumber |
| --- | --- |
| Go-live 1 Maret 2027, toleransi waktu 0; scope boleh dikurangi kecuali syarat izin, audit keamanan, UAT | `[DOK] P7.2, P8.3` `[DOK: 01-business-case.md]` |
| Tim: user (PM), Claude lewat runbook, 1 mobile developer | `[DOK] P7.3` |
| Flutter wajib untuk aplikasi mobile pasien dan tenaga kesehatan | `[DOK] P7.4` |
| Data wajib disimpan di Indonesia | `[DOK] P5.5` |
| Toleransi biaya +20% dari perkiraan terakhir yang disetujui; anggaran total `[TIDAK DIKETAHUI]` | `[DOK: 01-business-case.md]` |
| PT. Arunika berperan sebagai platform: bukan fasyankes, bukan penjual barang, tidak menampung dana mitra | `[USER] T2-Q2, T2-Q3, T2-Q4` |

### 7.3 Dependensi

| Dependensi | Dampak bila terlambat | Sumber |
| --- | --- | --- |
| Rekanan (min. 2 klinik, 1 lab, 3 outlet) bergabung dan sistemnya bisa diintegrasikan (PT-05) | Layanan S-04 sampai S-08 dan semua konsultasi (karena nakes di bawah klinik rekanan) tidak bisa berjalan | `[DOK] P4.3, P8.2` `[USER] T2-Q2` |
| Payment gateway berizin BI yang mendukung pembagian dana langsung ke rekening rekanan | BR-041 tidak bisa dipenuhi untuk semua layanan berbayar; ketersediaan fitur diverifikasi di Tahap 4–5 | `[USER] T2-Q4, T2-Q13` |
| Penyedia video, kurir, dan email yang memproses dan menyimpan data di Indonesia (T4-Q1) | Pemilihan penyedia di Tahap 4–5 tertahan | `[DOK] P4.2, P5.5` |
| Konfirmasi legal L-01 sampai L-15 | Lihat kolom "Dibutuhkan sebelum" di bagian 6.3 | Bagian 6.3 |
| Pendaftaran PSE, perizinan berusaha, dan ketentuan Kemenkes yang mungkin terbit sebelum go-live | Syarat go-live 5 tidak terpenuhi | `[SRC-06]` `[SRC-12]` |
| Akun developer Google Play dan Apple atas nama PT. Arunika | Syarat go-live 4 | `[DOK: 01-business-case.md]` R-08 |

## 8. Perubahan terhadap dokumen yang sudah lulus gate

Dicatat sebagai change request di `docs/backlog.md`. Semuanya keputusan user: CR-001 sampai CR-004 di wawancara Tahap 2, CR-005 di putaran setelah gate Tahap 2, CR-006 di wawancara Tahap 3.

| CR | Perubahan | Dokumen terdampak | Sumber |
| --- | --- | --- | --- |
| CR-001 | Semua tenaga kesehatan berpraktik atas nama klinik rekanan; tidak ada lagi mitra langsung tanpa fasyankes | Brief PS6e | `[USER] T2-Q2` |
| CR-002 | Apotek rekanan menjadi outlet rekanan; obat bebas dijual apotek dan klinik rekanan berizin | Brief PS2, PS2a | `[USER] T2-Q3, T2-Q3b` |
| CR-003 | Pembayaran dibagi langsung oleh payment gateway ke rekening rekanan; PT. Arunika tidak menampung dan meneruskan dana sendiri | Brief PS4, P8.2a | `[USER] T2-Q4` |
| CR-004 | Pendapatan transaksi berupa markup di atas harga rekanan (persen/nominal, per rekanan), menggantikan komisi potongan | Business Case §5.1, §5.2 Tabel A–B | `[USER] T2-Q6, T2-Q6a` |
| CR-005 | Semua layanan yang dipesan di aplikasi dibayar di aplikasi, termasuk pendaftaran klinik, lab, dan home visit; pasien cukup menunjukkan appointment; tindakan tambahan di luar aplikasi. Markup dan aturan biaya transaksi berlaku untuk semua layanan; refund atas pembatalan layanan terjadwal sebelum batas waktu. Diajukan setelah gate Tahap 2 | Brief P3.3; BRD v1.0 P-04, P-05, P-06, P-08, S-10, §4.2, BR-020, BR-023, BR-024, BR-040, BR-042, BR-043, BR-044, BR-045, BR-070, L-04, L-08, L-10, L-12, L-13, L-14, PT-17, PT-27, PT-28; Business Case §5 | `[USER] T2-Q13–T2-Q20` |
| CR-006 | RME minimalis sebagai aplikasi terpisah untuk klinik dan lab tanpa sistem, terhubung lewat integrasi, data rekam medis sesuai SATUSEHAT, hanya pasien Rumah Sehat; antrean real-time klinik masuk rilis 1; RME lengkap, loyalitas/poin/voucher, dan multi-bahasa tidak masuk; portal rekanan hanya untuk pelaporan dan rekap ditambah menu pengaturan konsultasi online | BRD v1.1 §1.3, §2.2, §4.1–§4.3, BR-018, BR-064, BR-070, L-05; Business Case §6 (biaya) dan R-02, R-04 | `[USER] T3-Q1–T3-Q1c, T3-Q14–T3-Q16` |
| CR-007 | Semua penyedia pihak ketiga memproses dan menyimpan data pribadi di Indonesia tanpa kecuali (PT-22). Login Google/Apple, notifikasi push, dan WhatsApp dihapus karena lokasi pemrosesannya tidak dapat dipastikan di Indonesia; notifikasi lewat email dan di dalam aplikasi. Diajukan di Tahap 4 | Brief P4.4, P4.5; BRD v1.2 §2.1, P-01, S-14, BR-001, BR-050, BR-071, BR-113, §7.3, PT-22; PRD v1.0 | `[USER] T4-Q1, T4-Q2` |

Bagian BRD v1.1 yang diubah oleh CR-006 dan jawaban Tahap 3 lainnya: §1.3, §2.2, §4.1–§4.3, BR-002, BR-007 (BR-008 baru), BR-017, BR-018, BR-027 baru, BR-045, BR-064, BR-070, BR-100, §5.11 (BR-120–BR-126 baru), §6.1, L-02, L-05, L-07, L-08, L-14, L-15 baru, §9, §10.

## 9. Tinjauan ulang Business Case (gate Tahap 2)

Sesuai rencana tinjauan `[DOK: 01-business-case.md]` §9:

| Butir | Dampak | Sumber |
| --- | --- | --- |
| Model pendapatan | Komisi potongan diganti markup (CR-004). Rumus Tabel A–B di Business Case §5.2 hanya setara untuk markup **persen** bila "nilai transaksi" dibaca sebagai harga rekanan sebelum markup; untuk markup **nominal**, pendapatan = nominal × jumlah transaksi. Istilah dan rumusnya perlu diperbarui. Besaran tetap `[TIDAK DIKETAHUI]` (PT-17) | `[USER] T2-Q6a` |
| Biaya transaksi dan ongkir | Ongkir dan biaya transaksi non-QRIS dibebankan ke pasien; MDR QRIS ditanggung rekanan. Dengan begitu biaya transaksi tidak lagi mengurangi pendapatan Rumah Sehat seperti dicatat di Business Case §6.1, selama konfirmasi legal L-12 tidak mengubahnya | `[USER] T2-Q7, T2-Q8, T2-Q12, T2-Q19` `[SRC-24]` |
| Biaya perizinan, legal, dan kepatuhan | Bertambah komponen yang belum bernilai: konsultan hukum, pendaftaran PSE, perizinan berusaha, kemungkinan PSEF, DPIA, petugas PDP (PT-25) | Bagian 6 |
| Risiko R-03 (izin) | Sudah diriset. Kebutuhan teridentifikasi (bagian 6.2), tetapi beberapa butir kritis menunggu konfirmasi legal (L-01, L-03, L-04, L-05). Penilaian ulang kemungkinan dilakukan di gate | `[DOK: compliance/tahap-02-riset.md]` |
| Risiko R-01 (rekanan) | Bertambah cakupannya: konsultasi online kini juga bergantung pada klinik rekanan karena semua nakes berpraktik di bawahnya | `[USER] T2-Q2` |
| Model pendapatan (CR-005) | Markup kini juga berlaku untuk pendaftaran klinik, lab, dan home visit, sehingga sumber pendapatan transaksi bertambah. Biaya rekanan dipakai atau tidak diputuskan CEO (PT-17). Business Case §5 perlu memuat keduanya saat diperbarui | `[USER] T2-Q15, T2-Q16` |
| Risiko R-01 (rekanan), tambahan CR-005 | Pembayaran klinik, lab, dan home visit di aplikasi menambah kebutuhan integrasi: sistem rekanan perlu menerima status lunas dan mencocokkan appointment (BR-070, PT-05) | `[USER] T2-Q13` |
| Risiko baru: regulasi berubah | Kemenkes menyatakan akan mengatur seluruh platform kesehatan digital, termasuk tarif, sehingga aturan baru bisa terbit sebelum go-live | `[SRC-06]` |
| Scope dan jadwal (CR-006, Tahap 3) | Aplikasi web RME minimalis dan integrasi SATUSEHAT menambah komponen besar di dalam 104 hari dengan 1 mobile developer (R-02, R-04). Biaya dan kelayakan jadwal dinilai ulang di gate Tahap 4 (baseline biaya) dan Tahap 7 (phase plan) | `[USER] T3-Q15` |

## 10. Pertanyaan terbuka

### 10.1 Status pertanyaan terbuka sebelumnya

| # | Status | Jawaban / tindak lanjut | Memblokir PRD? |
| --- | --- | --- | --- |
| PT-02 | Terjawab sebagian | Riset: tidak ditemukan sumber resmi yang mewajibkan badan usaha khusus; izin, KBLI, dan kategori PSE menunggu L-02 | Tidak (memengaruhi syarat go-live 5) |
| PT-03 | Terjawab sebagian | Regulasi dan instansi di bagian 6.1; auditor dan instansi pemeriksa menunggu L-11 | Tidak |
| PT-04 | Terjawab sebagian | Keputusan user: obat bebas tetap dijual apotek dan klinik rekanan berizin (T2-Q3, T2-Q3b); kewajiban PSEF menunggu L-03; tindak lanjut bila wajib di PT-21 | Tidak |
| PT-05 | Terbuka | Tetap; tenggat sebelum Tahap 4 | Tidak (memblokir Tahap 4) |
| PT-06 | Terjawab sebagian | Mekanisme: pembagian langsung oleh payment gateway (T2-Q4). Jadwal pencairan mengikuti penyedia, ditetapkan Tahap 4–5. Pembayaran ke tenaga kesehatan menjadi urusan klinik rekanan yang menaunginya (konsekuensi T2-Q2) | Tidak |
| PT-07 | Terjawab | Konsultasi online + home visit; berpraktik atas nama klinik rekanan (T2-Q2, T2-Q5) | — |
| PT-08, PT-10, PT-12, PT-13, PT-15 | Terjawab di Tahap 3 | PT-08: T3-Q9; PT-10: target desain T3-Q11; PT-12: T3-Q12; PT-13: T3-Q1–Q1c, T3-Q15 (CR-006); PT-15: T3-Q10. Rincian di PRD §12.1 | — |
| PT-16 | Terjawab | T2-Q10, T2-Q11 (bagian 1.2) | — |
| PT-17 | Terbuka | Besaran markup, serta dipakai-tidaknya, bentuk, dan besaran biaya rekanan ditetapkan CEO sebelum go-live; sistem menyediakan pengaturannya (T2-Q1, T2-Q16) | Tidak |
| PT-18 | Terbuka | Tenggat dipindah ke gate Tahap 5 bersama baseline biaya `[USER] T4-G4` | Tidak |
| PT-19 | Terbuka | Status mobile developer penuh waktu belum dijawab. Tenggat baru: gate Tahap 5 bersama baseline biaya (dipindah di gate Tahap 4 `[USER] T4-G4`) | Tidak |

Catatan PT-06: kalimat "pembayaran ke tenaga kesehatan menjadi urusan klinik rekanan" adalah turunan logis dari T2-Q2 dan disetujui user di gate Tahap 2 `[USER] T2-Gate`.

### 10.2 Pertanyaan terbuka baru

Kolom Pemilik dan Tenggat diusulkan penyusun dan disetujui user di gate Tahap 2 `[USER] T2-Gate`.

| # | Pertanyaan | Pemilik | Tenggat | Memblokir PRD? |
| --- | --- | --- | --- | --- |
| PT-20 | Penunjukan konsultan hukum/tim legal untuk menjawab L-01 sampai L-15 (L-15 ditambahkan di Tahap 3) | User (PM) dan CEO | Segera; L-01, L-04, L-05, L-06, L-08, L-12, L-13 sebelum Tahap 4 | Tidak |
| PT-21 | Kalau legal menyatakan Rumah Sehat wajib PSEF (dengan apoteker penanggung jawab), apa tindakannya: obat bebas ditunda lewat P8.3, atau PSEF dipenuhi? | CEO | Setelah jawaban L-03 | Tidak (obat bebas dapat dilepas lewat P8.3) |
| PT-22 | Apakah keputusan "data di Indonesia" juga berlaku untuk penyedia video, email, WhatsApp, push, dan login Google/Apple? **Terjawab:** semua di Indonesia tanpa kecuali `[USER] T4-Q1`; akibatnya CR-007 `[USER] T4-Q2` | User (PM), CEO, CTO | Sebelum Tahap 4 | Tidak |
| PT-23 | Siapa pejabat/petugas PDP dan pemilik proses notifikasi insiden (internal atau eksternal)? | CEO | Sebelum Tahap 10 | Tidak |
| PT-24 | Masa retensi per jenis data (setelah L-05) | CEO, berdasarkan pendapat legal | Sebelum Tahap 4 (model data) | Tidak |
| PT-25 | Anggaran dan waktu perizinan, legal, dan kepatuhan | CEO | Gate Tahap 5 bersama baseline biaya (dipindah di gate Tahap 4 `[USER] T4-G4`) | Tidak |
| PT-26 | Bagaimana data anggota keluarga dipakai: apakah pasien dapat memesan konsultasi, pendaftaran, home visit, atau produk untuk anggota keluarga? **Terjawab:** BR-008 `[USER] T3-Q3, T3-Q3a` | User (PM) | Tahap 3 | — |
| PT-27 | Refund setelah dana terbagi langsung ke rekening rekanan (BR-041, BR-045), kini untuk semua layanan termasuk pembatalan sebelum batas waktu (CR-005): siapa yang menanggung, termasuk MDR atas transaksi yang di-refund, dan bagaimana dana ditarik kembali dari rekanan? | CEO; mekanisme teknis di Tahap 4–5 | Tahap 3 (aturan bisnis), Tahap 4–5 (mekanisme). **Aturan bisnis terjawab:** penyebab menanggung, PRD §3.5 `[USER] T3-Q7` | Tidak |
| PT-28 | Perlakuan pajak atas markup dan pembagian dana langsung untuk semua layanan (usulan review kepatuhan CR-005, pemilik dan tenggat diusulkan penyusun dan disetujui user di gate revisi `[USER] T2-Gate-CR005`; ketentuannya `[TIDAK DIKETAHUI]`) | CEO, dengan konsultan pajak | Gate Tahap 5 bersama baseline biaya (dipindah di gate Tahap 4 `[USER] T4-G4`) | Tidak |

Pertanyaan terbuka baru dari Tahap 3 (PT-29 sampai PT-33) tercatat di `docs/03-prd.md` §12.2.
