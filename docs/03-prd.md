# PRD (Product Requirements Document): Rumah Sehat

Tanggal: 2026-11-17
Versi: 1.0 (draf untuk gate Tahap 3)
Penyusun: Claude (persona Product Manager + Tech Lead). Review acceptance criteria oleh subagent `qa-engineer`, review kepatuhan CR-006 oleh `compliance-reviewer`, dan pemeriksaan fakta oleh `fact-checker`
Pemilik dan penyetuju: user (project manager, meneruskan ke CEO dan CTO) `[DOK: 00-project-brief.md]`
Dasar: BRD v1.2 `[DOK: 02-brd.md]`. BRD v1.1 sudah lulus gate; revisi v1.2 (CR-006) diajukan bersama PRD ini di gate Tahap 3.
Tingkat ketelitian:
- **T1 Kritis**: §3.1 (akses), §3.3–§3.5 (harga, pembayaran, refund), EP-08 (pembayaran), EP-13 (RME minimalis), EP-15 (audit trail), EP-16 (data pribadi), §6.4–§6.6 (keamanan, privasi, retensi), §7 (audit trail).
- **T2 Standar**: bagian lain.

PRD menetapkan *apa* yang dibangun dan bagaimana mengujinya. *Bagaimana* membangunnya ditetapkan di Tahap 4 (arsitektur) dan Tahap 5 (stack).

## 0. Cara membaca

### 0.1 Label sumber

| Penanda | Arti |
| --- | --- |
| `BR-xxx` | Kebutuhan bisnis di BRD v1.2 (dokumen yang lulus gate atau diajukan bersama PRD ini). Di tabel dan baris "Rujukan" story, penanda ini berfungsi sebagai label `[DOK: 02-brd.md]` |
| `[USER] T3-Qx` | Jawaban user di wawancara Tahap 3, tercatat di `docs/03-catatan-prd.md`. `[USER] T2-Qx` merujuk ke `docs/02-catatan-brd.md` |
| `[DOK] Px.x` | Project Brief dan catatan intake (`docs/00-project-brief.md`, `docs/00-catatan-intake.md`) |
| `[UP-xx]` | **Usulan penyusun** untuk detail produk yang belum diputuskan user. Semua UP terdaftar di §11 dan dibawa ke gate Tahap 3 untuk disetujui atau diubah user. Setelah disetujui, statusnya menjadi `[USER] T3-Gate` |
| `[KEPUTUSAN-xx]` | Keputusan teknis penyusun, tercatat di `docs/decision-log.md` |
| `[SRC-xx]` | Fakta eksternal yang sudah diverifikasi, tercatat di `docs/sources.md` |
| `[TIDAK DIKETAHUI]` | Belum ada informasi. Masuk pertanyaan terbuka (§12) beserta pemiliknya |

### 0.2 Penomoran

| Awalan | Isi |
| --- | --- |
| `EP-xx` | Epic, yaitu modul fitur |
| `US-xxx` | User story. Tiga digit pertama mengikuti epic (misalnya US-301 ada di EP-03) |
| `AB-xx` | Aturan bisnis lintas modul (§3) |
| `PB-xx` | Parameter bisnis yang nilainya bisa diatur (§3.7) |
| `KU-xx` | Kriteria umum yang berlaku untuk semua story (§4) |
| `NFR-xxx-nn` | Kebutuhan non-fungsional (§6, §7) |
| `UP-xx` | Usulan penyusun (§11) |

Acceptance criteria hanya ditulis di PRD ini. `docs/backlog.md` memuat indeks epic dan story yang merujuk ke PRD, supaya tidak ada dua versi kriteria yang bisa berbeda `[KEPUTUSAN-03]`.

## 1. Ringkasan produk

Rumah Sehat adalah platform yang menghubungkan pasien dengan klinik, lab, dan outlet rekanan. PT. Arunika berperan sebagai Penyelenggara Sistem Elektronik, bukan fasyankes, bukan penjual barang, dan tidak menampung dana mitra (BRD §2.1, §7.2). Data klinik, lab, home visit, dan produk berasal dari integrasi dengan sistem rekanan, sedangkan konsultasi online berlangsung di Rumah Sehat (BR-013, BR-033, BR-070). Rumah Sehat hanya platform yang mengintegrasikan pasien dengan rekanan `[USER] T3-Q14`.

Tujuan bisnis dan KPI tidak berubah dari BRD §1 (TB-01–TB-04, KPI-01–KPI-04).

### 1.1 Komponen yang dibangun di rilis 1

| Komponen | Pengguna | Platform | Sumber |
| --- | --- | --- | --- |
| Aplikasi pasien | Pasien | Android dan iOS (Flutter) | BR-007 `[DOK] P6.1, P7.4` |
| Aplikasi tenaga kesehatan | Dokter, psikolog, fisioterapis, tenaga medis | Android dan iOS (Flutter) | BR-007 `[USER] T3-Q9` |
| Backoffice admin Rumah Sehat | Admin Rumah Sehat | Web | BR-007, BR-060–BR-063 |
| Portal rekanan | Admin klinik, admin lab, admin outlet | Web. Isinya pelaporan dan rekap, ditambah satu menu pengaturan konsultasi online untuk klinik | BR-064, BR-065 `[USER] T3-Q4, T3-Q14, T3-Q16` |
| Aplikasi RME minimalis | Staf klinik dan lab rekanan yang belum punya sistem | Web, terpisah dari aplikasi dan portal Rumah Sehat, terhubung lewat integrasi | BR-120, BR-121 `[USER] T3-Q15` |
| Backend, integrasi, dan audit trail | Semua komponen | Server di Indonesia | BR-070, BR-071, BR-080, BR-113 |

Apakah aplikasi tenaga kesehatan dibuat terpisah atau digabung dengan aplikasi pasien diputuskan di Tahap 4 `[USER] T3-Q9`.

### 1.2 Peran pengguna

Sembilan peran Rumah Sehat mengikuti BRD §2.2. Tambahan dari PRD ini:

| Peran | Komponen | Sumber |
| --- | --- | --- |
| Sub-peran admin Rumah Sehat: Super Admin, Verifikator, Layanan Pelanggan, Keuangan, Konten, Auditor | Backoffice | `[UP-18]` |
| Admin Fasyankes, Tenaga Kesehatan Fasyankes, Petugas Lab | Aplikasi RME minimalis, untuk klinik/lab rekanan pengguna RME | BR-120 `[UP-26]` |
| Operator RME (staf PT. Arunika untuk onboarding fasyankes ke RME minimalis; tidak dapat melihat isi rekam medis) | Aplikasi RME minimalis | BR-120 `[UP-26]` |

## 2. Scope rilis 1

### 2.1 In-scope

Layanan S-01 sampai S-16 mengikuti BRD §4.1. Tambahan CR-006 dan Tahap 3:

| # | Kemampuan | Sumber |
| --- | --- | --- |
| S-17 | Aplikasi RME minimalis untuk klinik dan lab rekanan yang belum punya sistem, terpisah dari Rumah Sehat dan terhubung lewat integrasi | BR-120–BR-126 `[USER] T3-Q1a, T3-Q1b, T3-Q1c, T3-Q14, T3-Q15` |
| S-18 | Antrean real-time klinik | BR-027 `[USER] T3-Q1a` |
| S-19 | Pemesanan layanan untuk anggota keluarga | BR-008 `[USER] T3-Q3, T3-Q3a` |

### 2.2 Won't (tidak dibangun di rilis 1)

Semua butir non-scope BRD §4.2 tetap berlaku. Tambahan:

| Butir | Sumber |
| --- | --- |
| RME lengkap untuk operasional klinik dan lab (diganti RME minimalis) | `[USER] T3-Q1a` |
| RME minimalis untuk pasien yang tidak datang lewat Rumah Sehat (walk-in) | `[USER] T3-Q1c` |
| Program loyalitas, poin, dan voucher | `[USER] T3-Q1a` (tidak dipilih) |
| Multi-bahasa; antarmuka hanya Bahasa Indonesia | `[USER] T3-Q1a` (tidak dipilih) |
| Jadwal ulang (reschedule) layanan terjadwal | `[USER] T2-Q17, T3-Q6` (opsi tidak dipilih) |
| Pengisian katalog, stok, atau pesanan produk di portal oleh outlet tanpa sistem | BR-033 `[USER] T3-Q1b` (tidak disebut user) |
| Rekam medis, check-in, antrean, hasil lab, atau home visit yang diisi di portal Rumah Sehat | `[USER] T3-Q14` |
| Pembatalan pesanan produk oleh pasien setelah membayar | BR-045 `[UP-08]` |

### 2.3 Urutan pengurangan scope

Kalau layanan belum siap pada 1 Maret 2027, tanggal tetap dan layanan dilepas berurutan, dimulai dari nomor 1 `[USER] T3-Q2` `[DOK] P8.3`:

1. Antrean real-time klinik (EP-04 US-405, EP-13 US-1304)
2. Obat bebas, lalu seluruh pembelian produk (EP-07)
3. Home visit (EP-05)
4. Konsultasi online fisioterapis dan tenaga medis (bagian EP-03)
5. Pendaftaran lab dan informasi biaya MCU/lab (EP-04 bagian lab, EP-06, EP-09)
6. Pendaftaran klinik (EP-04 bagian klinik)

Konsultasi online dokter dan psikolog dipertahankan paling akhir `[USER] T3-Q2`. Syarat izin dan kepatuhan, audit keamanan, dan UAT tidak bisa dikurangi `[DOK] P8.3`. RME minimalis tidak diberi nomor sendiri. Fiturnya ikut dilepas bersama layanan yang memakainya (misalnya penugasan home visit ikut nomor 3) `[UP-33]`.

### 2.4 Prioritas MoSCoW

- Ketujuh layanan rilis 1 berprioritas Must, karena "ketujuh layanan berjalan" adalah syarat go-live 3 `[DOK] P8.2`.
- Fitur kepatuhan (EP-15, EP-16) dan pembayaran (EP-08) berprioritas Must, karena syarat izin dan kepatuhan tidak bisa dikurangi `[DOK] P8.3`.
- Antrean real-time berprioritas Should, sesuai posisinya di urutan pertama pengurangan scope `[USER] T3-Q2`.
- MoSCoW per story di bagian 5 adalah usulan penyusun `[UP-01]`.

## 3. Aturan bisnis lintas modul

### 3.1 Akses data per peran (T1)

| Data | Pasien | Tenaga kesehatan | Admin klinik/lab/outlet (portal) | Admin RS | Pengguna RME minimalis | Sumber |
| --- | --- | --- | --- | --- | --- | --- |
| Profil dan identitas dasar | Miliknya dan anggota keluarganya | Nama, jenis kelamin, tanggal lahir, dan keluhan awal pasien di sesi/kunjungan yang ditanganinya | Nama pasien di laporan transaksi rekanannya | Layanan Pelanggan, Verifikator, Super Admin | Pasien Rumah Sehat di fasyankesnya | BR-002, BR-105 `[UP-17]` |
| NIK, foto KTP, foto KK | Miliknya dan anggota keluarganya | Tidak | Tidak | Hanya Verifikator. Peran lain melihat NIK termasking | NIK pasien di fasyankesnya bila dibutuhkan pencatatan rekam medis `[TIDAK DIKETAHUI]` (L-05) | BR-003, BR-004, BR-105 `[UP-17]` |
| Isi chat, catatan konsultasi, resep | Miliknya dan anggota keluarganya | Hanya sesi yang ditanganinya | Tidak | Tidak ada peran admin RS yang dapat melihat | Tidak | BR-016, BR-105 `[UP-15]` `[UP-17]` |
| Hasil lab | Miliknya dan anggota keluarganya | Tidak, di rilis 1 | Tidak | Tidak | Petugas Lab dan Tenaga Kesehatan Fasyankes di lab/klinik yang memeriksa | BR-025, BR-105 `[UP-17]` |
| Rekam medis RME minimalis | Lewat hak akses data (EP-16) | Tidak | Tidak | Tidak (Operator RME juga tidak) | Tenaga Kesehatan Fasyankes dan Admin Fasyankes di fasyankes itu | BR-123 `[UP-17]` `[UP-26]` |
| Transaksi, pembagian dana, refund | Transaksinya | Tidak | Transaksi rekanannya | Keuangan, Super Admin (baca) | Tidak | BR-046 |
| Audit log | Tidak di rilis 1 | Tidak | Tidak | Auditor (baca dan export), Super Admin (baca) | Tidak | BR-080 `[UP-18]` |

Akses yang ditolak ke data sensitif dicatat di audit trail (KU-03, §7).

### 3.2 Verifikasi identitas dan pembatasan layanan

| ID | Aturan | Sumber |
| --- | --- | --- |
| AB-01 | Konsultasi online, home visit, serta pendaftaran klinik dan lab hanya dapat dipesan untuk orang yang identitasnya sudah **disetujui** admin. Pembelian produk tidak mewajibkan identitas | BR-003, BR-004 |
| AB-02 | Pemilik akun memverifikasi dirinya dengan NIK dan foto KTP | BR-003 |
| AB-03 | Anggota keluarga dewasa diverifikasi dengan NIK dan foto KTP. Anak diverifikasi dengan NIK dan foto Kartu Keluarga yang memuat pemilik akun sebagai orang tua/wali. Semuanya diverifikasi manual oleh admin | BR-008 `[USER] T3-Q3a` |
| AB-04 | Saat menambah anggota keluarga, pemilik akun wajib menyatakan telah mendapat persetujuan anggota keluarga dewasa, atau memberi persetujuan sebagai orang tua/wali untuk anak. Pernyataan disimpan sebagai bukti. Kecukupan cara ini menunggu konfirmasi legal (L-08) | BR-008, BR-104 `[USER] T3-Q3a` |
| AB-05 | Status verifikasi identitas: Belum diunggah, Menunggu verifikasi, Disetujui, Ditolak (dengan alasan). Identitas yang ditolak dapat diunggah ulang | BR-004 `[UP-10]` |

### 3.3 Harga (T1)

| ID | Aturan | Sumber |
| --- | --- | --- |
| AB-10 | Harga layanan yang dibayar pasien = harga rekanan + markup Rumah Sehat. Markup berbentuk persen atau nominal, diatur per rekanan oleh admin Rumah Sehat | BR-042 |
| AB-11 | Untuk produk, total = harga produk (harga rekanan + markup) + ongkos kirim | BR-034, BR-042 |
| AB-12 | Untuk metode pembayaran non-QRIS, biaya transaksi ditambahkan ke total dan ditampilkan sebelum pasien membayar. Untuk QRIS, pasien tidak dikenai biaya transaksi, dan MDR dipotong dari bagian rekanan. Pembebanan per metode menunggu L-12 | BR-044 `[SRC-24]` |
| AB-13 | Pasien melihat harga layanan sebagai **satu angka** (harga rekanan + markup). Ongkos kirim dan biaya transaksi tampil sebagai baris terpisah. Markup tidak ditampilkan sebagai baris tersendiri | `[UP-07]` (sejalan dengan T2-Q20); keterbukaan rincian harga menunggu L-14 |
| AB-14 | Informasi biaya MCU dan lab menampilkan harga yang sama dengan harga bayar di aplikasi | BR-023 `[USER] T2-Q20` |
| AB-15 | Sebelum pembayaran dibuat, harga dan ketersediaan (slot atau stok) diperiksa ulang ke sistem rekanan. Kalau berbeda, pasien diberi tahu nilai terbaru dan harus menyetujuinya sebelum membayar | `[UP-25]` |
| AB-16 | Layar ringkasan pesanan klinik, lab, dan home visit menyatakan bahwa harga hanya mencakup layanan yang tercantum, dan tindakan tambahan diselesaikan serta dibayar di luar aplikasi | BR-024 (L-14) |
| AB-17 | Perubahan markup atau tarif rekanan tidak mengubah harga pesanan yang sudah dibayar. Harga dikunci saat pembayaran dibuat | ADR-004 `[UP-32]` |

### 3.4 Pembayaran dan pembagian dana (T1)

| ID | Aturan | Sumber |
| --- | --- | --- |
| AB-20 | Semua layanan yang dipesan di aplikasi dibayar di aplikasi lewat payment gateway berizin Bank Indonesia | BR-040, BR-115 |
| AB-21 | Setiap pembayaran dibagi langsung oleh payment gateway: bagian rekanan ke rekening rekanan, markup ke rekening PT. Arunika. PT. Arunika tidak menampung bagian rekanan | BR-041 |
| AB-22 | Bagian rekanan = harga rekanan, dikurangi MDR bila dibayar dengan QRIS | BR-041, BR-044 |
| AB-23 | Penerima dana ongkos kirim (outlet atau kurir) dan cara penyalurannya dalam pembagian dana `[TIDAK DIKETAHUI]` (PT-31). Ditetapkan di Tahap 4–5 bersama pemilihan kurir dan payment gateway | BR-034, BR-041 |
| AB-24 | Cara penagihan biaya rekanan (dipotong dari pembagian dana atau ditagih terpisah) `[TIDAK DIKETAHUI]` (PT-17). Rilis 1 menyediakan pengaturan dan perhitungan tagihannya | BR-043 |
| AB-25 | Metode pembayaran yang tersedia mengikuti payment gateway terpilih (Tahap 4–5) `[TIDAK DIKETAHUI]` | BR-071 |
| AB-26 | Pembayaran, pembagian dana, dan refund dicatat immutable. Koreksi dilakukan lewat entri pembalik | ADR-004 |
| AB-27 | Perlakuan pajak atas markup dan pembagian dana `[TIDAK DIKETAHUI]` (PT-28) | BR-042 |

### 3.5 Pembatalan, ketidakhadiran, dan refund (T1)

Aturan pokok BR-045 dirinci dengan jawaban `[USER] T3-Q5, T3-Q6, T3-Q7, T3-Q8`. Mekanisme penarikan dana dari rekanan ditetapkan di Tahap 4–5 (PT-27). Kebijakan ini menunggu konfirmasi legal L-04 dan L-14.

**Definisi** (sumber: `[USER] T3-Q6` untuk definisi no-show yang disetujui bersama pertanyaan):

- **Layanan terjadwal**: konsultasi online, pendaftaran klinik, pendaftaran lab, home visit.
- **Pasien tidak hadir (no-show)**:
    - konsultasi: pasien tidak bergabung ke sesi sampai PB-02 habis sejak jadwal mulai;
    - klinik/lab: appointment tidak di-check-in sampai jadwalnya berakhir;
    - home visit: tenaga kesehatan menandai pasien tidak ada di lokasi.
- **Tenaga kesehatan tidak hadir**:
    - konsultasi: tenaga kesehatan tidak bergabung ke sesi sampai PB-03 habis sejak jadwal mulai `[USER] T3-Q8`;
    - home visit: tenaga kesehatan belum menandai "tiba" sampai PB-03 habis sejak jadwal mulai, lalu pasien melaporkannya `[UP-13]`.
- **Harga layanan**: harga rekanan + markup (AB-10).
- **Seluruh pembayaran**: harga layanan + ongkos kirim (produk) + biaya transaksi (non-QRIS).

**Matriks refund**

| # | Kejadian | Berlaku untuk | Refund? | Yang dikembalikan ke pasien | Penanggung | Cara | Sumber |
| --- | --- | --- | --- | --- | --- | --- | --- |
| RF-01 | Pasien membatalkan paling lambat PB-01 sebelum jadwal | Layanan terjadwal | Ya | Harga layanan. Biaya transaksi non-QRIS tidak dikembalikan | Bagian rekanan dikembalikan rekanan; markup dikembalikan Rumah Sehat; MDR QRIS tetap ditanggung rekanan | Otomatis | BR-045 `[USER] T3-Q5, T3-Q7, T3-Q8` |
| RF-02 | Pasien membatalkan kurang dari PB-01 sebelum jadwal | Layanan terjadwal | Tidak | — | — | — | BR-045 |
| RF-03 | Pasien tidak hadir | Layanan terjadwal | Tidak | — | Rekanan tetap menerima bagiannya | — | `[USER] T3-Q6` |
| RF-04 | Tenaga kesehatan tidak bergabung ke sesi konsultasi sampai PB-03 habis | Konsultasi | Ya | Seluruh pembayaran | Rekanan mengembalikan bagiannya dan menanggung biaya payment gateway yang tidak kembali; Rumah Sehat mengembalikan markup | Otomatis | `[USER] T3-Q7, T3-Q8` |
| RF-05 | Rekanan atau tenaga kesehatan membatalkan | Layanan terjadwal | Ya | Seluruh pembayaran | Seperti RF-04 | Otomatis | BR-045 `[USER] T3-Q7, T3-Q8` |
| RF-06 | Tenaga kesehatan home visit tidak datang, dilaporkan pasien | Home visit | Ya, bila klaim disetujui | Seluruh pembayaran | Seperti RF-04 | Pengajuan, diputuskan admin RS dalam PB-05 | `[USER] T3-Q7` `[UP-13]` |
| RF-07 | Layanan gagal karena gangguan platform Rumah Sehat | Semua | Ya, bila klaim disetujui | Seluruh pembayaran | Rekanan mengembalikan bagiannya; Rumah Sehat mengembalikan markup dan menanggung biaya payment gateway yang tidak kembali | Pengajuan, diputuskan admin RS dalam PB-05 | `[USER] T3-Q7` |
| RF-08 | Outlet menolak pesanan (misalnya stok habis) atau tidak memprosesnya sampai PB-06 | Produk | Ya | Seluruh pembayaran | Outlet mengembalikan bagiannya dan menanggung biaya payment gateway yang tidak kembali; Rumah Sehat mengembalikan markup | Otomatis | `[USER] T3-Q7, T3-Q8` `[UP-03]` |
| RF-09 | Kurir gagal mengirim (status gagal dari kurir) | Produk | Ya | Seluruh pembayaran | `[TIDAK DIKETAHUI]` (PT-29): T3-Q7 hanya menyebut rekanan atau Rumah Sehat sebagai penyebab | Otomatis | `[USER] T3-Q7, T3-Q8` |
| RF-10 | Barang rusak atau tidak sesuai, diajukan paling lambat PB-04 setelah diterima, dengan foto | Produk | Ya, bila klaim disetujui | Seluruh pembayaran | Outlet, seperti RF-08. Kerusakan dalam pengiriman `[TIDAK DIKETAHUI]` (PT-29). Pengembalian barang fisik `[TIDAK DIKETAHUI]` (PT-30) | Pengajuan, diputuskan admin RS dalam PB-05 | BR-045 `[USER] T3-Q7, T3-Q8` |
| RF-11 | Pasien ingin membatalkan pesanan produk setelah membayar | Produk | Tidak. Pembatalan tidak disediakan | — | — | — | BR-045 `[UP-08]` |
| RF-12 | Pasien terdebit dua kali untuk satu pesanan (pembayaran ganda) | Semua | Ya, untuk pembayaran ganda | Seluruh pembayaran ganda | Rumah Sehat (gangguan platform) | Otomatis setelah terdeteksi saat rekonsiliasi | `[USER] T3-Q7` `[UP-34]` |

Ketentuan tambahan:

- **AB-30** Refund dieksekusi lewat payment gateway ke metode bayar asal bila penyedia mendukungnya `[TIDAK DIKETAHUI]` (Tahap 4–5). Waktu sampainya dana ke pasien mengikuti penyedia dan metode pembayaran `[TIDAK DIKETAHUI]`.
- **AB-31** Setiap refund dicatat sebagai entri pembalik yang merujuk transaksi asal (ADR-004), termasuk bagian rekanan dan markup yang dikembalikan.
- **AB-32** Refund yang gagal dieksekusi berstatus "Refund gagal" dan masuk antrean tindak lanjut peran Keuangan. Pasien diberi tahu bahwa refund sedang ditangani `[UP-35]`.
- **AB-33** Klaim refund yang tidak diputuskan sampai PB-05 habis dieskalasi ke Super Admin dan ditandai terlambat di laporan. Klaim tidak otomatis disetujui atau ditolak `[UP-36]`.

### 3.6 Status layanan dan pesanan

Status berikut usulan penyusun `[UP-10]`. Perubahan status dicatat di audit trail (BR-080 butir 6).

**Konsultasi online:** Menunggu pembayaran → Terkonfirmasi → Berlangsung → Selesai. Cabang:
- Kedaluwarsa (tidak dibayar sampai PB-07)
- Dibatalkan pasien (refund mengikuti RF-01 atau RF-02)
- Dibatalkan rekanan (RF-05)
- Tenaga kesehatan tidak hadir (RF-04)
- Pasien tidak hadir (RF-03)

**Pendaftaran klinik/lab:** Menunggu pembayaran → Terkonfirmasi → Check-in → Selesai. Cabang:
- Kedaluwarsa, Dibatalkan pasien, Dibatalkan rekanan (seperti konsultasi)
- Pasien tidak hadir (tidak check-in sampai jadwal berakhir)

Untuk lab, hasil lab adalah objek tersendiri yang terhubung ke pendaftaran (EP-09).

**Home visit:** Menunggu pembayaran → Terkonfirmasi → Ditugaskan → Berangkat → Tiba → Selesai. Cabang:
- Kedaluwarsa, Dibatalkan pasien, Dibatalkan rekanan
- Pasien tidak hadir (ditandai tenaga kesehatan)
- Dilaporkan tidak datang → Klaim disetujui (RF-06) atau Klaim ditolak

**Pesanan produk:** Menunggu pembayaran → Dibayar → Diproses outlet → Dikirim → Diterima → Selesai (setelah PB-04 habis tanpa klaim). Cabang:
- Kedaluwarsa (tidak dibayar)
- Ditolak outlet, atau Tidak diproses sampai PB-06 (RF-08)
- Gagal kirim (RF-09)
- Klaim → Klaim disetujui (RF-10) atau Klaim ditolak → Selesai

**Refund:** Diproses → Berhasil, atau Diproses → Gagal → Diproses ulang oleh Keuangan.

**Klaim:** Diajukan → Disetujui atau Ditolak (dengan alasan).

### 3.7 Parameter bisnis

Parameter diatur di backoffice. Setiap perubahan dicatat di audit trail sebagai perubahan konfigurasi.

| ID | Parameter | Nilai awal | Diatur oleh | Sumber |
| --- | --- | --- | --- | --- |
| PB-01 | Batas waktu pembatalan berbayar-kembali, per jenis layanan terjadwal | 24 jam sebelum jadwal (semua jenis) | Super Admin | `[USER] T3-Q5` |
| PB-02 | Toleransi keterlambatan pasien bergabung ke sesi konsultasi | 15 menit | Super Admin | `[UP-02]` |
| PB-03 | Toleransi tenaga kesehatan bergabung ke sesi konsultasi atau tiba di lokasi home visit | 15 menit | Super Admin | `[USER] T3-Q8` (konsultasi) `[UP-13]` (home visit) |
| PB-04 | Batas pengajuan klaim barang rusak atau tidak sesuai | 2 x 24 jam setelah status Diterima | Super Admin | `[USER] T3-Q8` |
| PB-05 | Batas waktu keputusan admin atas klaim refund | 3 hari kerja | Super Admin | `[USER] T3-Q8` |
| PB-06 | Batas waktu outlet menerima pesanan sebelum refund otomatis | 24 jam setelah pembayaran | Super Admin | `[UP-03]` |
| PB-07 | Batas waktu pembayaran sebelum pesanan kedaluwarsa dan slot/stok dilepas | 30 menit, dibatasi kemampuan payment gateway | Super Admin | `[UP-04]` |
| PB-08 | Waktu pengingat jadwal | 24 jam dan 1 jam sebelum jadwal | Super Admin | `[UP-05]` |
| PB-09 | Peringatan sebelum SIP berakhir | 30 hari sebelum tanggal berakhir | Super Admin | `[UP-06]` |
| PB-10 | Notifikasi antrean | Saat giliran tinggal 3 nomor | Super Admin | `[UP-12]` |
| PB-11 | Markup per rekanan (persen atau nominal) | `[TIDAK DIKETAHUI]`, ditetapkan CEO sebelum go-live (PT-17) | Keuangan | BR-042, BR-063 |
| PB-12 | Biaya rekanan (tidak dipakai, langganan, atau per pendaftaran; besaran) | `[TIDAK DIKETAHUI]`, diputuskan CEO (PT-17) | Keuangan | BR-043, BR-063 |
| PB-13 | Tarif dan durasi sesi konsultasi per tenaga kesehatan dan per jenis (chat/video) | Diisi klinik | Admin klinik (portal) | BR-018 `[USER] T3-Q4, T3-Q16` |
| PB-14 | Dokumen izin wajib per jenis tenaga kesehatan | STR dan SIP untuk semua jenis, menunggu L-07 | Super Admin | BR-102 `[UP-19]` |

"Hari kerja" mengikuti kalender hari kerja yang diatur Super Admin `[UP-37]`.

### 3.8 Zona waktu

| ID | Aturan | Sumber |
| --- | --- | --- |
| AB-40 | Jadwal klinik, lab, dan home visit tampil dalam zona waktu lokasi rekanan, dengan label zona (misalnya "10.00 WITA") | `[USER] T3-Q10` |
| AB-41 | Jadwal konsultasi online tampil dalam zona waktu perangkat masing-masing pengguna | `[USER] T3-Q10` |
| AB-42 | Laporan di portal rekanan memakai zona waktu rekanan; laporan dan backoffice Rumah Sehat memakai WIB | `[USER] T3-Q10` |
| AB-43 | Batas waktu berbasis jam (PB-01, PB-02, PB-03, PB-04, PB-06, PB-07) dihitung dari waktu absolut, tidak terpengaruh zona waktu tampilan | `[KEPUTUSAN-11]` |

### 3.9 Tenaga kesehatan

| ID | Aturan | Sumber |
| --- | --- | --- |
| AB-50 | Setiap tenaga kesehatan terhubung ke satu klinik rekanan yang menaunginya, dan melayani atas nama klinik itu | BR-005, BR-014 |
| AB-51 | Tenaga kesehatan hanya dapat diaktifkan bila dokumen wajib (PB-14) terverifikasi dan masih berlaku. Akun dinonaktifkan otomatis saat SIP berakhir | BR-102 |
| AB-52 | Satu tenaga kesehatan melayani satu sesi konsultasi pada satu waktu. Slot jadwalnya tidak boleh tumpang tindih | `[UP-11]` |
| AB-53 | Hanya dokter yang dapat menerbitkan resep digital | BR-015 |
| AB-54 | Tenaga kesehatan berstatus **siap melayani** (KPI-02) bila akunnya aktif, terverifikasi, dokumen izinnya berlaku, terhubung ke klinik rekanan yang aktif, dan tarif konsultasinya sudah diatur | BR-090 `[UP-24]` |
| AB-55 | Jadwal praktik online diatur tenaga kesehatan di aplikasinya. Admin klinik dapat melihat dan mengubahnya di portal | `[USER] T3-Q4, T3-Q16` |

### 3.10 Sumber data rekanan

| ID | Aturan | Sumber |
| --- | --- | --- |
| AB-60 | Jadwal, layanan, harga, home visit, katalog, stok, pesanan produk, status appointment, check-in, antrean, dan hasil lab berasal dari sistem rekanan lewat integrasi | BR-033, BR-070 |
| AB-61 | Klinik dan lab yang belum punya sistem memakai aplikasi RME minimalis. RME minimalis terhubung ke Rumah Sehat lewat integrasi yang sama dengan sistem rekanan lain | BR-120, BR-121 `[USER] T3-Q14, T3-Q15` |
| AB-62 | Portal rekanan hanya untuk pelaporan dan rekap, ditambah satu menu pengaturan konsultasi online untuk klinik (PB-13, AB-55) | BR-064, BR-065 `[USER] T3-Q14, T3-Q16` |
| AB-63 | Outlet wajib terhubung lewat integrasi sistemnya sendiri. RME minimalis tidak menyediakan katalog produk | BR-033 `[USER] T3-Q1b` |
| AB-64 | Klinik/lab yang memakai RME minimalis dihitung "terintegrasi" untuk syarat go-live 1 dan KPI-03/KPI-04, karena terhubung lewat integrasi | `[USER] T3-Q14, T3-Q15` (konsekuensi yang dimintakan konfirmasi di gate) |
| AB-65 | Klinik rekanan yang juga menjual produk memakai satu akun rekanan dengan dua kemampuan (klinik dan outlet). Portalnya menampilkan laporan keduanya | `[UP-20]` |

### 3.11 Data klinis dan kerahasiaan

| ID | Aturan | Sumber |
| --- | --- | --- |
| AB-70 | Data kesehatan (catatan, isi chat, resep, hasil lab, rekam medis), data anak, dan data keuangan pribadi adalah data pribadi spesifik dengan akses lebih ketat (§3.1) | BR-105 `[SRC-15]` |
| AB-71 | Pasien dapat melihat catatan konsultasi dan resep miliknya dan milik anggota keluarganya | BR-106 `[UP-15]` |
| AB-72 | Catatan konsultasi memakai struktur data yang selaras dengan SATUSEHAT, sama dengan rekam medis RME minimalis, supaya siap kalau legal menyatakan catatan konsultasi termasuk rekam medis (L-05) | BR-114 `[UP-16]` |
| AB-73 | Isi notifikasi push, email, dan WhatsApp tidak memuat data kesehatan rinci (diagnosis, nama obat, nilai hasil lab, isi chat) | BR-105 `[UP-23]` |
| AB-74 | Panggilan video tidak direkam dan tidak disimpan | BR-016 |
| AB-75 | Data yang dikirim ke payment gateway dibatasi pada yang diperlukan untuk pembayaran. Batas datanya menunggu L-08 | BR-105 (L-08) |

## 4. Kriteria umum (berlaku untuk semua story)

Kriteria ini berlaku di setiap story tanpa ditulis ulang `[KEPUTUSAN-04]`. Story hanya menulis kondisi khususnya.

| ID | Kriteria |
| --- | --- |
| KU-01 | **Validasi.** Given field wajib kosong atau formatnya salah, When pengguna menyimpan atau mengirim, Then aksi ditolak, pesan kesalahan tampil di setiap field yang salah, dan isian yang sudah benar tidak hilang |
| KU-02 | **Gagal jaringan.** Given koneksi terputus atau server tidak merespons dalam batas waktu, When pengguna melakukan aksi, Then pesan "gagal terhubung" dan tombol coba lagi tampil, dan percobaan ulang tidak menghasilkan aksi ganda (idempoten), terutama untuk pembayaran, refund, dan pemesanan |
| KU-03 | **Akses ditolak.** Given pengguna tidak berhak atas suatu data atau aksi, When ia mencoba mengaksesnya (termasuk dengan mengganti ID di permintaan), Then akses ditolak tanpa membocorkan keberadaan data, dan percobaan akses ke data sensitif (§3.1) dicatat di audit trail dengan hasil "gagal" |
| KU-04 | **Data terhapus.** Data master yang di-soft delete tidak tampil di daftar dan pencarian default, dan tidak bisa dipilih untuk transaksi baru (ADR-003) |
| KU-05 | **Kiriman ganda.** Given pengguna menekan kirim dua kali atau permintaan terkirim ulang, When server menerimanya, Then hanya satu entitas yang dibuat |
| KU-06 | **Audit.** Setiap aksi yang termasuk event audit (§7.1) mencatat event dengan field §7.2, baik sukses maupun gagal |
| KU-07 | **Waktu.** Semua waktu tampil mengikuti AB-40–AB-42 |
| KU-08 | **Rekanan atau pihak ketiga tidak tersedia.** Given sistem rekanan atau penyedia pihak ketiga tidak merespons, When pasien membuka data atau memesan, Then aplikasi menyatakan data sementara tidak tersedia, tidak menampilkan data lama seolah terbaru (menampilkan waktu pembaruan terakhir bila ada), dan tidak membuat pembayaran untuk layanan yang tidak bisa dikonfirmasi ke rekanan |
| KU-09 | **Rebutan slot atau stok.** Given dua pasien memilih slot atau stok terakhir yang sama, When keduanya melanjutkan ke pembayaran, Then hanya satu yang berhasil, dan yang lain diberi tahu bahwa slot/stok tidak tersedia sebelum membayar |
| KU-10 | **Akun nonaktif.** Given akun pasien, tenaga kesehatan, rekanan, atau admin berstatus nonaktif, When pemiliknya masuk atau memakai sesi yang masih terbuka, Then akses ditolak dengan pesan akun nonaktif |

## 5. Fitur per modul dan user story

Setiap story mencantumkan rujukan BR (induk), prioritas, event audit (§7.1), dan acceptance criteria (AC) format Given–When–Then. Kriteria umum KU-01–KU-10 berlaku tanpa ditulis ulang. Prioritas per story adalah usulan penyusun `[UP-01]`.

### EP-01 Akun, identitas, dan anggota keluarga (aplikasi pasien)

| Story | Fitur | BR | Prioritas |
| --- | --- | --- | --- |
| US-101 | Registrasi dengan email dan password | BR-001, BR-103 | Must |
| US-102 | Masuk dengan akun Google atau Apple | BR-001 | Must |
| US-103 | Masuk, keluar, dan atur ulang password | BR-001 | Must |
| US-104 | Persetujuan pemrosesan data | BR-103, BR-104, BR-107 | Must |
| US-105 | Profil data diri | BR-002 | Must |
| US-106 | Unggah NIK dan foto KTP | BR-003 | Must |
| US-107 | Pembatasan layanan berdasarkan status verifikasi | BR-003, BR-004 | Must |
| US-108 | Tambah anggota keluarga | BR-002, BR-008, BR-104 | Should |
| US-109 | Ubah dan hapus anggota keluarga | BR-008, BR-106 | Should |

#### US-101 Registrasi dengan email dan password

Sebagai pasien, saya ingin mendaftar dengan email dan password, agar dapat memakai layanan Rumah Sehat.
Rujukan: BR-001, BR-103 · Audit: E-04

- AC1. Given email belum terdaftar dan password memenuhi kebijakan NFR-SEC-03, When pasien mengisi formulir, menyetujui persetujuan pemrosesan data (US-104), dan mengirim, Then akun dibuat berstatus "email belum diverifikasi" dan tautan verifikasi dikirim ke email itu `[KEPUTUSAN-06]`.
- AC2. Given tautan verifikasi masih berlaku, When pasien membukanya, Then email berstatus terverifikasi dan pasien dapat masuk. Given tautan sudah kedaluwarsa atau sudah dipakai, Then pasien diminta mengirim ulang tautan.
- AC3. Given email sudah terdaftar (termasuk lewat Google/Apple), When pasien mendaftar dengan email itu, Then akun baru tidak dibuat dan pasien diarahkan untuk masuk atau mengatur ulang password.
- AC4. Given kotak persetujuan belum dipilih, When pasien mengirim formulir, Then pendaftaran ditolak dan pasien diberi tahu bahwa persetujuan wajib (BR-103).

#### US-102 Masuk dengan akun Google atau Apple

Sebagai pasien, saya ingin masuk dengan akun Google atau Apple, agar tidak perlu membuat password baru.
Rujukan: BR-001 · Audit: E-01, E-02, E-04

- AC1. Given pasien belum punya akun, When ia memilih Google atau Apple dan otorisasi berhasil, Then akun dibuat setelah pasien menyetujui persetujuan pemrosesan data (US-104).
- AC2. Given pasien sudah punya akun dari Google/Apple yang sama, When ia masuk lagi, Then ia masuk ke akun yang sama.
- AC3. Given email dari Google/Apple sama dengan akun email-password yang sudah ada, When pasien masuk dengan Google/Apple, Then akun tidak otomatis digabung, dan pasien diminta memasukkan password akun lama untuk menautkannya `[KEPUTUSAN-06]`.
- AC4. Given pasien membatalkan atau otorisasi gagal di halaman Google/Apple, When ia kembali ke aplikasi, Then ia berada di layar masuk tanpa akun dibuat, dan percobaan dicatat sebagai login gagal.

#### US-103 Masuk, keluar, dan atur ulang password

Sebagai pasien, saya ingin masuk, keluar, dan mengatur ulang password, agar akun saya aman dan tetap bisa diakses.
Rujukan: BR-001 · Audit: E-01, E-02, E-03, E-04

- AC1. Given email dan password benar dan email terverifikasi, When pasien masuk, Then ia masuk ke beranda dan event login berhasil tercatat.
- AC2. Given email atau password salah, When pasien masuk, Then muncul pesan umum yang tidak menyebut bagian mana yang salah, dan event login gagal tercatat.
- AC3. Given terjadi 5 percobaan gagal berturut-turut untuk satu akun, When percobaan berikutnya dilakukan dalam 15 menit, Then akun terkunci sementara sampai 15 menit berlalu, walaupun password benar `[KEPUTUSAN-06]`.
- AC4. Given pasien meminta atur ulang password, When ia memasukkan email, Then pesan yang sama tampil, terdaftar atau tidak. Kalau terdaftar, tautan sekali pakai yang berlaku 60 menit dikirim `[KEPUTUSAN-06]`.
- AC5. Given pasien memilih keluar, When proses selesai, Then sesi di server diakhiri, dan memakai token lama ditolak (KU-10).

#### US-104 Persetujuan pemrosesan data

Sebagai pasien, saya ingin membaca dan memberikan persetujuan pemrosesan data, agar saya tahu data apa yang diproses dan untuk apa.
Rujukan: BR-103, BR-104, BR-107 `[SRC-15]` · Audit: E-13

- AC1. Given pasien mendaftar, When layar persetujuan tampil, Then layar memuat tujuan pemrosesan, jenis data yang diproses termasuk data kesehatan, masa retensi, hak subjek data, dan kontak Rumah Sehat. Teks masa retensi menunggu PT-24 dan wajib terisi sebelum go-live.
- AC2. Given pasien menyetujui, When ia melanjutkan, Then bukti persetujuan tersimpan: versi teks, butir yang disetujui, waktu, dan kanal.
- AC3. Given layar mendukung lebih dari satu butir persetujuan, When legal menetapkan dasar pemrosesan per jenis data (L-08), Then butir baru dapat ditambahkan tanpa mengubah aplikasi `[UP-36]`.
- AC4. Given teks persetujuan diperbarui ke versi baru, When pasien membuka aplikasi berikutnya, Then ia diminta menyetujui versi baru sebelum dapat memesan layanan.
- AC5. Given pasien tidak menyetujui, When ia mencoba melanjutkan, Then akun tidak dibuat (registrasi) atau layanan tidak dapat dipesan (versi baru).

#### US-105 Profil data diri

Sebagai pasien, saya ingin mengisi dan mengubah data diri, agar data layanan dan pengiriman saya benar.
Rujukan: BR-002 · Audit: E-10 (perubahan data identitas)

- AC1. Given pasien membuka profil, When ia mengisi nama, tanggal lahir, jenis kelamin, nomor HP, email, dan alamat, Then data tersimpan.
- AC2. Given pasien menambah lebih dari satu alamat, When ia menyimpannya, Then alamat tersimpan di daftar alamat dan dapat dipilih saat pengiriman produk atau home visit `[UP-38]`.
- AC3. Given identitas pasien sudah disetujui, When ia mengubah nama, tanggal lahir, atau NIK, Then status identitas kembali ke "Menunggu verifikasi", dan layanan yang mewajibkan identitas tidak dapat dipesan sampai disetujui lagi `[UP-39]`.
- AC4. Given pasien mengubah email, When ia menyimpan, Then email baru baru berlaku setelah diverifikasi lewat tautan, dan email lama diberi tahu `[KEPUTUSAN-06]`.

#### US-106 Unggah NIK dan foto KTP

Sebagai pasien, saya ingin mengunggah NIK dan foto KTP, agar dapat memakai konsultasi, home visit, serta pendaftaran klinik dan lab.
Rujukan: BR-003, BR-004 · Audit: E-07 (saat diputuskan), E-10

- AC1. Given identitas berstatus "Belum diunggah" atau "Ditolak", When pasien mengisi NIK dan mengunggah foto KTP dari kamera atau galeri, Then status menjadi "Menunggu verifikasi" dan pasien diberi tahu bahwa verifikasi dilakukan manual oleh admin.
- AC2. Given NIK berisi karakter selain angka, When dikirim, Then ditolak (KU-01). Validasi panjang dan format NIK ditetapkan di Tahap 4 dari sumber resmi `[TIDAK DIKETAHUI]`.
- AC3. Given berkas bukan JPG/PNG atau melebihi batas ukuran, When diunggah, Then ditolak dengan pesan jenis dan ukuran yang diizinkan `[KEPUTUSAN-08]`.
- AC4. Given NIK sudah disetujui sebagai **pemilik akun** di akun lain, When pasien mengirim NIK itu, Then pengiriman ditolak dan pasien diarahkan ke layanan pelanggan. NIK anggota keluarga boleh tercatat di lebih dari satu akun, misalnya anak yang didaftarkan kedua orang tuanya `[UP-40]`.
- AC5. Given status "Menunggu verifikasi", When pasien membuka profil, Then foto tidak dapat diganti sampai diputuskan, kecuali pasien membatalkan pengiriman.

#### US-107 Pembatasan layanan berdasarkan status verifikasi

Sebagai Rumah Sehat, saya ingin layanan tertentu hanya dapat dipesan untuk identitas yang sudah disetujui, agar sesuai ketentuan BR-003.
Rujukan: BR-003, BR-004, BR-008, AB-01 · Audit: tidak ada aksi sensitif

- AC1. Given penerima layanan (pemilik akun atau anggota keluarga) belum berstatus "Disetujui", When pasien memesan konsultasi, home visit, atau pendaftaran klinik/lab untuknya, Then pemesanan dihentikan sebelum pembayaran, dengan pesan status verifikasi dan tautan ke langkah berikutnya.
- AC2. Given identitas belum disetujui, When pasien membeli produk, Then pembelian tetap dapat dilakukan.
- AC3. Given identitas disetujui, When pasien memesan layanan, Then pemesanan berlanjut ke pemilihan jadwal.

#### US-108 Tambah anggota keluarga

Sebagai pasien, saya ingin menambahkan anggota keluarga, agar dapat memesankan layanan untuk mereka.
Rujukan: BR-002, BR-008, BR-104, AB-03, AB-04 · Audit: E-15

- AC1. Given pasien menambah anggota keluarga dewasa, When ia mengisi nama, hubungan, tanggal lahir, jenis kelamin, NIK, foto KTP, dan menyatakan telah mendapat persetujuan yang bersangkutan, Then anggota keluarga tersimpan berstatus "Menunggu verifikasi".
- AC2. Given pasien menambah anak, When ia mengisi data anak, NIK anak, foto Kartu Keluarga, dan memberikan persetujuan sebagai orang tua/wali, Then anggota keluarga tersimpan berstatus "Menunggu verifikasi" dan persetujuan orang tua/wali tersimpan sebagai bukti (BR-104).
- AC3. Given pernyataan atau persetujuan tidak dipilih, When dikirim, Then penambahan ditolak.
- AC4. Given NIK anggota keluarga sama dengan NIK pemilik akun, When dikirim, Then ditolak.
- AC5. Given NIK yang sama sudah tercatat sebagai anggota keluarga di akun yang sama, When dikirim, Then ditolak sebagai duplikat (KU-05).

#### US-109 Ubah dan hapus anggota keluarga

Sebagai pasien, saya ingin mengubah atau menghapus anggota keluarga, agar datanya tetap benar.
Rujukan: BR-008, BR-106 · Audit: E-15, E-16

- AC1. Given anggota keluarga sudah disetujui, When pasien mengubah nama, tanggal lahir, NIK, atau dokumen, Then statusnya kembali ke "Menunggu verifikasi" `[UP-39]`.
- AC2. Given anggota keluarga punya pesanan terjadwal yang sudah dibayar dan belum selesai, When pasien menghapusnya, Then penghapusan ditolak sampai pesanan selesai atau dibatalkan.
- AC3. Given tidak ada pesanan aktif, When pasien menghapus anggota keluarga, Then anggota keluarga tidak lagi tampil (soft delete). Riwayat transaksi dan rekam medis yang terkait tetap tersimpan mengikuti ketentuan retensi (PT-24, L-06).

### EP-02 Verifikasi dan pengelolaan akun

| Story | Fitur | BR | Prioritas |
| --- | --- | --- | --- |
| US-201 | Verifikasi identitas (KTP/KK) | BR-004, BR-008 | Must |
| US-202 | Pendaftaran tenaga kesehatan | BR-005, BR-102 | Must |
| US-203 | Verifikasi tenaga kesehatan | BR-005, BR-102 | Must |
| US-204 | Pemantauan masa berlaku SIP | BR-102 | Must |
| US-205 | Onboarding rekanan | BR-006, BR-031 | Must |
| US-206 | Akun admin mitra | BR-006 | Must |
| US-207 | Menonaktifkan dan mengaktifkan akun | BR-060 | Must |
| US-208 | Akun dan sub-peran admin Rumah Sehat | BR-060, BR-080 | Must |

#### US-201 Verifikasi identitas (KTP/KK)

Sebagai Verifikator, saya ingin memeriksa NIK dan foto KTP/KK, agar hanya identitas yang sah yang dapat memakai layanan berbatas.
Rujukan: BR-004, BR-008, AB-03, AB-05 · Audit: E-05 (membuka foto), E-07

- AC1. Given ada pengajuan "Menunggu verifikasi", When Verifikator membuka antrean, Then pengajuan tampil urut dari yang terlama, dengan data diri, NIK, dan foto dokumen.
- AC2. Given Verifikator menyetujui, When keputusan disimpan, Then status menjadi "Disetujui" dan pasien diberi tahu (US-1001).
- AC3. Given Verifikator menolak, When ia memilih alasan dari daftar dan dapat menambah keterangan, Then status menjadi "Ditolak", alasan tampil ke pasien, dan pasien dapat mengunggah ulang. Penolakan tanpa alasan tidak dapat disimpan.
- AC4. Given pengajuan anak memakai KK, When Verifikator memeriksa, Then ia dapat memastikan pemilik akun tercantum sebagai orang tua/wali. Kalau tidak tercantum, pengajuan ditolak dengan alasan itu.
- AC5. Given dua Verifikator membuka pengajuan yang sama, When keduanya memutuskan, Then hanya keputusan pertama yang tersimpan, dan yang kedua diberi tahu bahwa pengajuan sudah diputuskan.
- AC6. Given pengguna bukan Verifikator, When ia mencoba membuka foto dokumen, Then akses ditolak dan tercatat (KU-03).

#### US-202 Pendaftaran tenaga kesehatan

Sebagai tenaga kesehatan, saya ingin mendaftar di aplikasi tenaga kesehatan atas nama klinik rekanan saya, agar dapat melayani pasien Rumah Sehat.
Rujukan: BR-005, BR-102, AB-50, PB-14 `[UP-19]` · Audit: E-10

- AC1. Given klinik rekanan sudah aktif, When tenaga kesehatan mengisi data diri, jenis (dokter, psikolog, fisioterapis, tenaga medis), klinik penaung, dan dokumen wajib PB-14 (nomor dan foto STR; nomor, tanggal berakhir, dan foto SIP), serta dokumen penugasan dari klinik, Then akun tersimpan berstatus "Menunggu verifikasi".
- AC2. Given akun berstatus "Menunggu verifikasi" atau "Ditolak", When tenaga kesehatan masuk, Then ia hanya dapat melihat status dan melengkapi dokumen, dan tidak tampil untuk pasien.
- AC3. Given tanggal berakhir SIP sudah lewat, When dikirim, Then pendaftaran ditolak dengan pesan SIP sudah berakhir.
- AC4. Given klinik yang dipilih tidak aktif, When tenaga kesehatan mencari klinik, Then klinik itu tidak tampil di daftar.

#### US-203 Verifikasi tenaga kesehatan

Sebagai Verifikator, saya ingin memeriksa dokumen izin dan afiliasi tenaga kesehatan, agar hanya tenaga kesehatan berizin yang melayani.
Rujukan: BR-005, BR-102, AB-51 `[UP-19]` · Audit: E-05, E-10

- AC1. Given pengajuan lengkap, When Verifikator memeriksa dokumen dan mencatat konfirmasi afiliasi dari klinik (tanggal, cara konfirmasi, nama petugas klinik), lalu menyetujui, Then akun menjadi aktif. Statusnya menjadi siap melayani setelah tarif diatur klinik (AB-54).
- AC2. Given konfirmasi afiliasi klinik belum dicatat, When Verifikator menyetujui, Then persetujuan ditolak.
- AC3. Given Verifikator menolak, When alasan diisi, Then tenaga kesehatan diberi tahu alasannya dan dapat memperbaiki pengajuan.
- AC4. Kanal resmi untuk memeriksa keabsahan STR/SIP `[TIDAK DIKETAHUI]` (L-07). Sampai ada jawaban legal, pemeriksaan dilakukan atas dokumen yang diunggah dan konfirmasi klinik.

#### US-204 Pemantauan masa berlaku SIP

Sebagai Rumah Sehat, saya ingin masa berlaku SIP dipantau otomatis, agar tidak ada tenaga kesehatan melayani dengan SIP yang sudah berakhir.
Rujukan: BR-102, AB-51, PB-09 · Audit: E-09, E-10

- AC1. Given SIP berakhir dalam PB-09, When pemeriksaan harian berjalan, Then tenaga kesehatan, admin kliniknya (email), dan Verifikator diberi tahu.
- AC2. Given tanggal berakhir SIP terlewati, When pemeriksaan harian berjalan, Then akun dinonaktifkan otomatis, tidak tampil untuk pasien, dan event tercatat dengan aktor "sistem".
- AC3. Given akun dinonaktifkan karena SIP berakhir dan masih ada sesi terbayar di masa depan, When penonaktifan terjadi, Then sesi itu berstatus "Dibatalkan rekanan", refund RF-05 berjalan otomatis, dan pasien diberi tahu.
- AC4. Given tenaga kesehatan mengunggah SIP baru, When Verifikator menyetujuinya, Then akun aktif kembali.

#### US-205 Onboarding rekanan

Sebagai Verifikator, saya ingin mendaftarkan dan memverifikasi rekanan, agar hanya rekanan berizin yang tampil di aplikasi.
Rujukan: BR-006, BR-031, AB-60–AB-65 · Audit: E-05, E-10

- AC1. Given data rekanan lengkap (nama, jenis: klinik, lab, toko kecantikan, toko alat kesehatan, atau apotek; klinik dapat sekaligus outlet; alamat dan zona waktu; dokumen izin usaha; rekening penerima; sumber sistem: sistem sendiri atau RME minimalis), When Verifikator menyimpan, Then rekanan berstatus "Menunggu verifikasi".
- AC2. Given dokumen izin diperiksa dan disetujui, markup sudah diatur (US-809), dan status integrasi "Aktif" (US-1405), When ketiga syarat terpenuhi, Then rekanan tampil untuk pasien `[UP-44]`. Kalau salah satu belum, rekanan tidak tampil.
- AC3. Given rekanan berjenis toko kecantikan atau toko alat kesehatan, When katalognya memuat produk berkategori obat bebas, Then produk itu tidak ditampilkan (US-702), karena obat bebas hanya dijual apotek dan klinik berizin (BR-031).
- AC4. Jenis dokumen izin wajib per jenis rekanan `[TIDAK DIKETAHUI]` (L-02, L-03). Daftar dokumen wajib dapat diatur Super Admin `[UP-19]`.
- AC5. Cara verifikasi rekening dan pendaftaran rekening ke payment gateway mengikuti penyedia terpilih (Tahap 4–5) `[TIDAK DIKETAHUI]`.

#### US-206 Akun admin mitra

Sebagai Verifikator, saya ingin membuat dan mengelola akun admin klinik, admin lab, dan admin outlet, agar rekanan dapat memakai portal.
Rujukan: BR-006 · Audit: E-10, E-11

- AC1. Given rekanan sudah terverifikasi, When Verifikator membuat akun admin mitra dengan email, Then undangan aktivasi dikirim, dan admin mitra wajib mengatur password serta MFA saat aktivasi `[KEPUTUSAN-06]`.
- AC2. Given akun admin mitra, When ia masuk ke portal, Then ia hanya melihat data rekanannya sendiri (KU-03).
- AC3. Given admin mitra mencoba membuat akun admin lain, When ia mencari menu itu, Then menu tidak tersedia. Akun admin mitra hanya dikelola admin Rumah Sehat (BR-006).

#### US-207 Menonaktifkan dan mengaktifkan akun

Sebagai admin Rumah Sehat, saya ingin menonaktifkan dan mengaktifkan kembali akun pasien, tenaga kesehatan, dan rekanan, agar penyalahgunaan bisa dihentikan.
Rujukan: BR-060 · Audit: E-10

- AC1. Given admin berhak (Super Admin; Verifikator untuk tenaga kesehatan dan rekanan; Layanan Pelanggan untuk pasien) mengisi alasan, When ia menonaktifkan akun, Then akun tidak dapat masuk (KU-10) dan alasan tersimpan.
- AC2. Given tenaga kesehatan atau rekanan punya pesanan terbayar di masa depan, When akunnya dinonaktifkan, Then pesanan itu berstatus "Dibatalkan rekanan" dan refund RF-05 berjalan otomatis.
- AC3. Given pasien punya pesanan terbayar yang belum selesai, When admin mencoba menonaktifkannya, Then penonaktifan ditolak sampai pesanan selesai atau dibatalkan `[UP-41]`.
- AC4. Given akun nonaktif, When admin mengaktifkannya kembali dengan alasan, Then akun dapat dipakai lagi.

#### US-208 Akun dan sub-peran admin Rumah Sehat

Sebagai Super Admin, saya ingin mengelola akun admin Rumah Sehat dan sub-perannya, agar setiap staf hanya punya akses yang dibutuhkan.
Rujukan: BR-060, BR-080 butir 8 `[UP-18]` · Audit: E-10, E-11

- AC1. Given Super Admin membuat akun admin, When ia memilih satu atau lebih sub-peran (Super Admin, Verifikator, Layanan Pelanggan, Keuangan, Konten, Auditor, Petugas PDP), Then akses akun mengikuti §3.1 dan MFA wajib diaktifkan saat pertama masuk.
- AC2. Given hanya tersisa satu Super Admin aktif, When peran Super Admin-nya akan dicabut atau akunnya dinonaktifkan, Then aksi ditolak.
- AC3. Given admin mencoba mengubah peran akunnya sendiri, When ia menyimpan, Then aksi ditolak.

### EP-03 Konsultasi online

| Story | Fitur | BR | Prioritas |
| --- | --- | --- | --- |
| US-301 | Tarif dan durasi konsultasi (menu portal klinik) | BR-018, BR-064 | Must |
| US-302 | Jadwal praktik online | BR-017 | Must |
| US-303 | Cari dan pilih tenaga kesehatan | BR-010, BR-011, BR-012, BR-014 | Must |
| US-304 | Pesan sesi konsultasi | BR-010, BR-011, BR-012, BR-040 | Must |
| US-305 | Sesi chat | BR-010, BR-016 | Must |
| US-306 | Sesi video | BR-010, BR-016 | Must |
| US-307 | Ketidakhadiran dan pembatalan oleh tenaga kesehatan | BR-045 | Must |
| US-308 | Catatan konsultasi | BR-013, BR-016, BR-114 | Must |
| US-309 | Resep digital | BR-015, BR-114 | Must |
| US-310 | Riwayat konsultasi pasien | BR-016, BR-106 | Must |
| US-311 | Daftar sesi tenaga kesehatan | BR-013 | Must |

Konsultasi online fisioterapis dan tenaga medis memakai story yang sama dan berada di nomor 4 urutan pengurangan scope (§2.3).

#### US-301 Tarif dan durasi konsultasi

Sebagai admin klinik, saya ingin mengatur tarif dan durasi sesi konsultasi per tenaga kesehatan dan per jenis, agar pasien melihat harga dan durasi yang benar.
Rujukan: BR-018, BR-064, PB-13 `[USER] T3-Q4, T3-Q16` · Audit: E-14

- AC1. Given admin klinik membuka menu pengaturan konsultasi online, When ia mengisi tarif (Rupiah, lebih dari 0) dan durasi (kelipatan 5 menit, 10–120 menit) untuk chat dan/atau video seorang tenaga kesehatan kliniknya, Then pengaturan tersimpan `[UP-29]`.
- AC2. Given suatu jenis belum diberi tarif, When pasien melihat tenaga kesehatan itu, Then jenis tersebut tidak dapat dipesan.
- AC3. Given tarif diubah, When pasien memesan setelahnya, Then harga baru berlaku. Pesanan yang sudah dibayar tidak berubah (AB-17).
- AC4. Given admin klinik mencoba mengatur tenaga kesehatan klinik lain, When ia mengirim permintaan, Then ditolak (KU-03).
- AC5. Given pasien melihat harga, When harga tampil, Then nilainya = tarif + markup rekanan itu (AB-10).

#### US-302 Jadwal praktik online

Sebagai tenaga kesehatan, saya ingin mengatur jadwal praktik online saya, agar pasien dapat memesan di waktu yang saya sediakan.
Rujukan: BR-017, AB-52, AB-55 `[USER] T3-Q4` · Audit: E-14

- AC1. Given tenaga kesehatan berstatus aktif, When ia membuat jadwal mingguan berulang (hari, jam mulai, jam selesai) dan pengecualian tanggal, Then slot terbentuk sesuai durasi per jenis (US-301) dalam zona waktu perangkatnya (AB-41).
- AC2. Given jadwal baru tumpang tindih dengan jadwal yang ada, When disimpan, Then ditolak (AB-52).
- AC3. Given slot sudah dipesan dan dibayar, When tenaga kesehatan atau admin klinik menghapus jadwal yang memuat slot itu, Then penghapusan ditolak. Sesi itu harus dibatalkan lebih dulu (US-307 AC4).
- AC4. Given admin klinik membuka jadwal tenaga kesehatan kliniknya di portal, When ia mengubahnya, Then aturan AC1–AC3 berlaku sama, dan perubahan tercatat atas nama admin klinik.

#### US-303 Cari dan pilih tenaga kesehatan

Sebagai pasien, saya ingin mencari tenaga kesehatan dan melihat profil, harga, serta jadwalnya, agar dapat memilih yang sesuai.
Rujukan: BR-010, BR-011, BR-012, BR-014, AB-54 · Audit: tidak ada aksi sensitif

- AC1. Given pasien membuka konsultasi, When ia memilih jenis (dokter, psikolog, fisioterapis, tenaga medis) atau mencari nama, Then hasil hanya memuat tenaga kesehatan yang siap melayani (AB-54) dan punya minimal satu slot di masa depan.
- AC2. Given hasil tampil, When pasien melihat kartu tenaga kesehatan, Then kartu memuat nama, jenis, klinik penaung (BR-014), harga chat dan/atau video (AB-10), dan slot terdekat.
- AC3. Given tidak ada hasil, When pencarian selesai, Then pesan "tidak ada tenaga kesehatan tersedia" tampil.

#### US-304 Pesan sesi konsultasi

Sebagai pasien, saya ingin memesan sesi konsultasi untuk saya atau anggota keluarga, agar mendapat konsultasi pada waktu yang dipilih.
Rujukan: BR-010, BR-011, BR-012, BR-040, AB-01, AB-15, PB-07 · Audit: E-08, E-09

- AC1. Given penerima layanan sudah terverifikasi (US-107), When pasien memilih jenis, slot, dan penerima layanan, mengisi keluhan awal (opsional, maksimal 500 karakter), lalu melanjutkan, Then slot ditahan selama PB-07 dan pasien masuk ke ringkasan harga (US-801).
- AC2. Given pembayaran berhasil (US-802), When notifikasi dari payment gateway diterima, Then sesi berstatus "Terkonfirmasi", dan pasien serta tenaga kesehatan diberi tahu.
- AC3. Given slot dimulai kurang dari PB-01 lagi, When pasien memesan, Then sebelum membayar pasien diberi tahu bahwa pembatalan sesi ini tidak dapat di-refund.
- AC4. Given pembayaran tidak selesai sampai PB-07, When batas habis, Then sesi berstatus "Kedaluwarsa" dan slot dilepas.

#### US-305 Sesi chat

Sebagai pasien dan tenaga kesehatan, kami ingin berkomunikasi lewat chat teks dan foto selama sesi, agar konsultasi berjalan.
Rujukan: BR-010, BR-016, NFR-PERF-03 `[UP-29]` · Audit: E-05 (akses tenaga kesehatan ke riwayat chat setelah sesi), E-09

- AC1. Given sesi chat "Terkonfirmasi", When waktu mulai tiba, Then ruang chat terbuka untuk pasien dan tenaga kesehatan sesi itu saja, dan status menjadi "Berlangsung" saat salah satu pihak bergabung.
- AC2. Given ruang chat terbuka, When salah satu pihak mengirim teks atau foto (KEPUTUSAN-08), Then pesan sampai ke penerima yang online dalam p95 2 detik dan tersimpan (BR-016).
- AC3. Given koneksi pengirim terputus, When ia mengirim pesan, Then pesan berstatus "belum terkirim" dan terkirim otomatis saat koneksi pulih selama sesi masih berjalan.
- AC4. Given durasi sesi habis, When waktu berakhir, Then ruang chat menjadi baca-saja, status menjadi "Selesai", dan sesi tidak diperpanjang karena gangguan jaringan.
- AC5. Given pengguna lain (termasuk admin Rumah Sehat), When mencoba membuka isi chat, Then ditolak (§3.1, KU-03).

#### US-306 Sesi video

Sebagai pasien dan tenaga kesehatan, kami ingin melakukan panggilan video selama sesi, agar konsultasi dapat dilakukan tatap muka.
Rujukan: BR-010, BR-016, AB-74 `[UP-29]` · Audit: E-09

- AC1. Given sesi video "Terkonfirmasi" dan waktu mulai tiba, When salah satu pihak memulai panggilan, Then pihak lain menerima panggilan masuk dan status menjadi "Berlangsung".
- AC2. Given panggilan terputus, When salah satu pihak menyambung ulang sebelum durasi habis, Then panggilan tersambung kembali tanpa menambah durasi.
- AC3. Given durasi habis, When waktu berakhir, Then panggilan diakhiri dan status menjadi "Selesai".
- AC4. Given sesi berlangsung, When diperiksa, Then aplikasi tidak menyediakan fitur rekam, dan fitur rekam di layanan video pihak ketiga dinonaktifkan (AB-74). Cara memverifikasi pengaturan penyedia ditetapkan di Tahap 4.

#### US-307 Ketidakhadiran dan pembatalan oleh tenaga kesehatan

Sebagai pasien, saya ingin ketidakhadiran ditangani otomatis sesuai aturan, agar saya mendapat refund bila tenaga kesehatan tidak hadir.
Rujukan: BR-045, RF-03, RF-04, RF-05, PB-02, PB-03 `[USER] T3-Q6, T3-Q8` `[UP-29]` · Audit: E-08, E-09

- AC1. Given tenaga kesehatan belum bergabung sampai PB-03 habis sejak jadwal mulai, When batas habis, Then status menjadi "Tenaga kesehatan tidak hadir", refund RF-04 berjalan otomatis, dan pasien diberi tahu.
- AC2. Given tenaga kesehatan sudah bergabung dan pasien belum bergabung sampai PB-02 habis, When batas habis, Then status menjadi "Pasien tidak hadir" tanpa refund (RF-03), dan pasien diberi tahu.
- AC3. Given keduanya tidak bergabung, When batas habis, Then berlaku AC1, karena kehadiran tenaga kesehatan adalah syarat sesi dapat berlangsung.
- AC4. Given tenaga kesehatan membatalkan sesi terkonfirmasi sebelum mulai dengan mengisi alasan, When pembatalan disimpan, Then status menjadi "Dibatalkan rekanan", refund RF-05 berjalan otomatis, dan pasien diberi tahu.

#### US-308 Catatan konsultasi

Sebagai tenaga kesehatan, saya ingin menulis catatan konsultasi, agar hasil konsultasi terdokumentasi.
Rujukan: BR-013, BR-016, BR-114, AB-72 `[UP-16]` · Audit: E-06

- AC1. Given sesi berstatus "Berlangsung" atau "Selesai", When tenaga kesehatan sesi itu mengisi catatan terstruktur (keluhan, temuan, diagnosis atau kesan, rencana atau saran) lalu memfinalkannya, Then catatan tersimpan dan dapat dilihat pasien (AB-71).
- AC2. Given catatan sudah final, When penulisnya mengubah, Then versi baru dibuat, versi lama tetap tersimpan dan terlihat sebagai riwayat, dan alasan perubahan wajib diisi.
- AC3. Given pengguna selain penulis, When ia mencoba mengubah catatan, Then ditolak.
- AC4. Struktur dan kode data (misalnya kode diagnosis) diselaraskan dengan SATUSEHAT di Tahap 4 `[SRC-28]`. Status catatan konsultasi sebagai rekam medis dan masa simpannya `[TIDAK DIKETAHUI]` (L-05).

#### US-309 Resep digital

Sebagai dokter, saya ingin menerbitkan resep digital setelah konsultasi, agar pasien dapat menebus obat di apotek mana pun.
Rujukan: BR-015, BR-114, AB-53 `[UP-42]` · Audit: E-06, E-12 (unduh)

- AC1. Given pengguna berjenis dokter dan sesinya "Berlangsung" atau "Selesai", When ia mengisi daftar obat (nama, kekuatan/dosis, jumlah, aturan pakai) lalu menerbitkan, Then resep bernomor unik terbit memuat identitas pasien, nama dokter dan nomor SIP, klinik, dan tanggal.
- AC2. Given pengguna bukan dokter, When ia mencari fitur resep, Then fitur tidak tersedia (AB-53).
- AC3. Given resep sudah terbit, When dokter perlu mengoreksi, Then resep tidak dapat diubah. Dokter membatalkannya dengan alasan lalu menerbitkan resep baru, dan resep batal tetap tersimpan berstatus "Dibatalkan".
- AC4. Given resep terbit, When pasien membukanya, Then ia dapat melihat dan mengunduhnya sebagai PDF untuk ditunjukkan di apotek.
- AC5. Format, tanda tangan elektronik, dan kewajiban integrasi resep elektronik `[TIDAK DIKETAHUI]` (L-05) `[SRC-07]`. Kanal apotek untuk memeriksa keaslian resep `[TIDAK DIKETAHUI]`.

#### US-310 Riwayat konsultasi pasien

Sebagai pasien, saya ingin melihat riwayat konsultasi beserta catatan, resep, dan chat, agar bisa merujuknya kembali.
Rujukan: BR-016, BR-106, AB-71 `[UP-15]` · Audit: E-12 (unduh)

- AC1. Given pasien membuka riwayat, When ia memilih dirinya atau anggota keluarga, Then daftar sesi tampil dengan tanggal, tenaga kesehatan, jenis, dan status.
- AC2. Given sesi selesai, When pasien membukanya, Then ia melihat chat (baca-saja), catatan final, dan resep.
- AC3. Given anggota keluarga sudah dihapus, When pemilik akun membuka riwayat, Then riwayat sesi anggota keluarga itu tetap dapat dilihat selama datanya disimpan (PT-24).

#### US-311 Daftar sesi tenaga kesehatan

Sebagai tenaga kesehatan, saya ingin melihat sesi hari ini dan yang akan datang, agar siap melayani tepat waktu.
Rujukan: BR-013 · Audit: tidak ada aksi sensitif

- AC1. Given tenaga kesehatan masuk, When ia membuka daftar sesi, Then tampil sesi terkonfirmasi dengan waktu (zona waktu perangkat), jenis, nama pasien, dan keluhan awal.
- AC2. Given ada pesanan baru atau waktu pengingat PB-08 tiba, When kejadian terjadi, Then tenaga kesehatan menerima notifikasi push (US-1003).
- AC3. Given waktu mulai sesi belum tiba, When ia menekan tombol bergabung, Then tombol belum aktif.

### EP-04 Pendaftaran klinik dan lab, check-in, dan antrean

| Story | Fitur | BR | Prioritas |
| --- | --- | --- | --- |
| US-401 | Cari klinik, lab, dan layanannya | BR-020, BR-021 | Must |
| US-402 | Pesan pendaftaran klinik atau lab | BR-020, BR-021, BR-024, BR-040 | Must |
| US-403 | Appointment dan kode check-in | BR-024 | Must |
| US-404 | Check-in lewat sistem rekanan | BR-024, BR-070 | Must |
| US-405 | Antrean real-time | BR-027 | Should |
| US-406 | Riwayat pendaftaran | BR-026 | Must |

#### US-401 Cari klinik, lab, dan layanannya

Sebagai pasien, saya ingin mencari klinik atau lab beserta layanan, harga, dan jadwalnya, agar dapat memilih tempat pemeriksaan.
Rujukan: BR-020, BR-021, AB-10, AB-40 · Audit: tidak ada aksi sensitif

- AC1. Given pasien mencari berdasarkan kota dan nama layanan/pemeriksaan (izin lokasi perangkat opsional), When hasil tampil, Then hasil hanya memuat rekanan aktif dan terintegrasi, beserta layanan, harga (AB-10), dan jadwal tersedia dalam zona waktu rekanan berlabel (AB-40).
- AC2. Given data rekanan tidak dapat diambil, When pasien membuka rekanan itu, Then berlaku KU-08.
- AC3. Given izin lokasi ditolak, When pasien mencari, Then pencarian tetap dapat dilakukan berdasarkan kota.

#### US-402 Pesan pendaftaran klinik atau lab

Sebagai pasien, saya ingin mendaftar ke klinik atau lab dan membayar di aplikasi, agar saat datang cukup menunjukkan appointment.
Rujukan: BR-020, BR-021, BR-024, BR-040, AB-01, AB-15, AB-16 · Audit: E-08, E-09

- AC1. Given penerima layanan terverifikasi, When pasien memilih layanan dan slot, Then slot ditahan selama PB-07 dan ringkasan harga tampil, termasuk pernyataan bahwa tindakan tambahan dibayar di luar aplikasi (AB-16).
- AC2. Given pembayaran berhasil, When notifikasi diterima, Then pendaftaran berstatus "Terkonfirmasi" dan dikirim ke sistem rekanan dengan status lunas (US-1403).
- AC3. Given pengiriman ke sistem rekanan gagal, When percobaan ulang otomatis juga gagal sampai batas waktu integrasi (Tahap 4), Then pendaftaran ditandai untuk tindak lanjut admin. Kalau rekanan menyatakan tidak dapat melayani, status menjadi "Dibatalkan rekanan" dan refund RF-05 berjalan otomatis.

#### US-403 Appointment dan kode check-in

Sebagai pasien, saya ingin melihat appointment beserta kode check-in, agar dapat menunjukkannya saat datang.
Rujukan: BR-024 `[UP-14]` · Audit: tidak ada aksi sensitif

- AC1. Given pendaftaran "Terkonfirmasi", When pasien membuka appointment, Then tampil kode dan QR check-in, nama rekanan, alamat, layanan, jadwal (AB-40), dan nama penerima layanan.
- AC2. Given perangkat sedang offline, When pasien membuka appointment yang pernah dibuka, Then kode dan QR tetap dapat ditampilkan.
- AC3. Given pendaftaran dibatalkan atau kedaluwarsa, When appointment dibuka, Then kode ditandai tidak berlaku.

#### US-404 Check-in lewat sistem rekanan

Sebagai pasien, saya ingin kedatangan saya tercatat saat menunjukkan appointment, agar status pendaftaran saya benar.
Rujukan: BR-024, BR-070, RF-03 `[UP-14]` · Audit: E-09

- AC1. Given rekanan memindai atau memasukkan kode di sistemnya (sistem sendiri atau RME minimalis), When sistem rekanan mengirim status check-in, Then status di Rumah Sehat menjadi "Check-in".
- AC2. Given sistem rekanan mengirim status selesai, When diterima, Then status menjadi "Selesai". Kalau status selesai tidak dikirim, pendaftaran yang sudah check-in otomatis menjadi "Selesai" pada akhir hari kalender rekanan.
- AC3. Given tidak ada check-in sampai jadwal berakhir, When batas terlewati, Then status menjadi "Pasien tidak hadir" tanpa refund (RF-03).
- AC4. Given kode milik rekanan lain, tanggal lain, atau pendaftaran yang tidak berlaku, When dikirim ke Rumah Sehat, Then check-in ditolak dan alasannya dikembalikan ke sistem rekanan.

#### US-405 Antrean real-time

Sebagai pasien yang sudah check-in, saya ingin melihat antrean secara langsung, agar tahu kapan giliran saya.
Rujukan: BR-027, PB-10 `[UP-12]` · Audit: tidak ada aksi sensitif

- AC1. Given rekanan mengirim data antrean, When pasien sudah check-in, Then aplikasi menampilkan nomor antrean pasien dan nomor yang sedang dilayani, diperbarui otomatis dengan keterlambatan maksimal 30 detik dari data rekanan.
- AC2. Given giliran pasien tinggal PB-10 nomor, When kondisi terpenuhi, Then pasien menerima notifikasi push.
- AC3. Given sistem rekanan tidak menyediakan data antrean, When pasien membuka appointment, Then tampilan antrean tidak muncul.

#### US-406 Riwayat pendaftaran

Sebagai pasien, saya ingin melihat riwayat pendaftaran klinik, lab, dan home visit, agar tahu layanan yang pernah saya pakai.
Rujukan: BR-026 · Audit: tidak ada aksi sensitif

- AC1. Given pasien membuka riwayat, When ia memilih dirinya atau anggota keluarga, Then daftar pendaftaran tampil dengan rekanan, layanan, jadwal, status, dan total pembayaran.
- AC2. Given pasien memfilter status, When filter diterapkan, Then hanya pendaftaran berstatus itu yang tampil.

### EP-05 Home visit

| Story | Fitur | BR | Prioritas |
| --- | --- | --- | --- |
| US-501 | Cari dan pesan home visit | BR-022, BR-024, BR-040 | Must |
| US-502 | Penugasan tenaga kesehatan | BR-022, BR-070 | Must |
| US-503 | Status kunjungan oleh tenaga kesehatan | BR-022 | Must |
| US-504 | Pantau kunjungan dan laporkan tenaga kesehatan tidak datang | BR-022, BR-045 | Must |

#### US-501 Cari dan pesan home visit

Sebagai pasien, saya ingin memesan home visit dari klinik rekanan, agar dapat dilayani di rumah.
Rujukan: BR-022, BR-024, BR-040, AB-01, AB-15, AB-16 `[UP-22]` · Audit: E-08, E-09

- AC1. Given pasien memilih alamat kunjungan, When ia mencari home visit, Then hanya layanan home visit dari klinik yang area layanannya mencakup alamat itu yang tampil, beserta harga (AB-10) dan jadwal (AB-40).
- AC2. Given penerima layanan terverifikasi, When pasien memilih layanan dan jadwal lalu membayar, Then pesanan berstatus "Terkonfirmasi" dan dikirim ke sistem klinik (US-1403).
- AC3. Given alamat di luar area layanan semua klinik, When pencarian selesai, Then pesan bahwa home visit belum tersedia di alamat itu tampil.

#### US-502 Penugasan tenaga kesehatan

Sebagai klinik rekanan, saya ingin menugaskan tenaga kesehatan klinik saya ke pesanan home visit, agar kunjungan dilayani.
Rujukan: BR-022, BR-070, AB-50, AB-51 `[UP-13]` · Audit: E-09

- AC1. Given klinik menugaskan tenaga kesehatan di sistemnya (sistem sendiri atau RME minimalis), When penugasan diterima Rumah Sehat, Then status menjadi "Ditugaskan", tenaga kesehatan menerima notifikasi, dan pasien melihat nama serta jenis tenaga kesehatan.
- AC2. Given tenaga kesehatan yang ditugaskan bukan tenaga kesehatan aktif klinik itu di Rumah Sehat, When penugasan diterima, Then penugasan ditolak dan alasannya dikembalikan ke sistem klinik.

#### US-503 Status kunjungan oleh tenaga kesehatan

Sebagai tenaga kesehatan, saya ingin melihat tugas home visit dan memperbarui statusnya, agar pasien dan klinik tahu perkembangan kunjungan.
Rujukan: BR-022, RF-03 `[UP-13]` · Audit: E-09

- AC1. Given tenaga kesehatan punya tugas, When ia membuka aplikasi, Then tampil jadwal, alamat (dengan tautan peta), nama pasien, layanan, dan nomor HP pasien. Nomor HP hanya tampil di hari kunjungan sampai kunjungan selesai.
- AC2. Given tugas "Ditugaskan", When tenaga kesehatan menandai "Berangkat", lalu "Tiba", lalu "Selesai", Then setiap status tersimpan berurutan dan pasien diberi tahu saat "Berangkat" dan "Tiba". Urutan tidak boleh dilompati.
- AC3. Given tenaga kesehatan menandai "Pasien tidak ada di lokasi" dengan keterangan, When disimpan, Then status menjadi "Pasien tidak hadir" tanpa refund (RF-03), dan pasien dapat mengajukan keberatan sebagai klaim (US-807).

#### US-504 Pantau kunjungan dan laporkan tenaga kesehatan tidak datang

Sebagai pasien, saya ingin memantau status kunjungan dan melaporkan bila tenaga kesehatan tidak datang, agar mendapat refund.
Rujukan: BR-022, BR-045, RF-06, PB-03, PB-05 `[UP-13]` · Audit: E-08, E-09

- AC1. Given pesanan home visit aktif, When pasien membukanya, Then tampil linimasa status dan nama tenaga kesehatan yang ditugaskan.
- AC2. Given status belum "Tiba" sampai PB-03 habis sejak jadwal mulai, When pasien membuka pesanan, Then tombol "Laporkan tenaga kesehatan tidak datang" aktif, dan laporan menjadi klaim RF-06 (US-807).
- AC3. Given status sudah "Tiba", When pasien membuka pesanan, Then tombol laporan tidak tersedia. Keluhan lain diajukan lewat layanan pelanggan (US-1103).

### EP-06 Informasi biaya MCU dan lab

| Story | Fitur | BR | Prioritas |
| --- | --- | --- | --- |
| US-601 | Cari dan bandingkan biaya MCU dan pemeriksaan lab | BR-023 | Must |
| US-602 | Lanjut mendaftar dari informasi biaya | BR-023, BR-020, BR-021 | Should |

#### US-601 Cari dan bandingkan biaya MCU dan pemeriksaan lab

Sebagai calon pasien, saya ingin melihat dan membandingkan biaya MCU dan pemeriksaan lab, agar tahu biayanya sebelum datang.
Rujukan: BR-023, AB-14 `[UP-21]` · Audit: tidak ada aksi sensitif

- AC1. Given pengguna, termasuk yang belum masuk, mencari nama pemeriksaan atau paket MCU dan kota, When hasil tampil, Then setiap rekanan yang menyediakannya ditampilkan dengan harga bayar di aplikasi (AB-14), diurutkan dari harga terendah.
- AC2. Given pengguna membuka paket MCU, When rincian tersedia dari rekanan, Then daftar pemeriksaan dalam paket dan informasi persiapan tampil.
- AC3. Given data harga rekanan tidak dapat diambil, When pengguna membuka hasil, Then berlaku KU-08.

#### US-602 Lanjut mendaftar dari informasi biaya

Sebagai calon pasien, saya ingin langsung mendaftar dari halaman informasi biaya, agar tidak perlu mencari ulang.
Rujukan: BR-023, BR-020, BR-021 · Audit: mengikuti US-402

- AC1. Given pengguna menekan "Daftar" di hasil, When ia sudah masuk dan terverifikasi, Then alur US-402 terbuka dengan rekanan dan layanan terpilih.
- AC2. Given pengguna belum masuk atau belum terverifikasi, When ia menekan "Daftar", Then ia diarahkan masuk atau melengkapi verifikasi, lalu kembali ke layanan yang sama.

### EP-07 Pembelian produk

| Story | Fitur | BR | Prioritas |
| --- | --- | --- | --- |
| US-701 | Jelajah dan cari katalog | BR-030, BR-033 | Must |
| US-702 | Detail produk | BR-030, BR-031, BR-032, BR-116 | Must |
| US-703 | Keranjang dan checkout | BR-034, BR-040 | Must |
| US-704 | Status dan pelacakan pesanan | BR-033, BR-034 | Must |
| US-705 | Konfirmasi terima dan klaim barang | BR-045 | Must |
| US-706 | Riwayat pesanan | BR-035 | Must |

Obat bebas berada di nomor 2 urutan pengurangan scope dan dapat dilepas tanpa melepas produk lain (§2.3, PT-21).

#### US-701 Jelajah dan cari katalog

Sebagai pasien, saya ingin menjelajah dan mencari produk, agar menemukan barang yang saya butuhkan.
Rujukan: BR-030, BR-033 `[UP-21]` · Audit: tidak ada aksi sensitif

- AC1. Given pengguna (boleh belum masuk) membuka belanja, When ia memilih kategori (obat bebas, kecantikan, kesehatan lain), outlet, atau mencari nama, Then produk dari outlet aktif dan terintegrasi tampil dengan harga (AB-10) dan status stok.
- AC2. Given stok produk habis, When produk tampil, Then produk ditandai "habis" dan tidak dapat dimasukkan ke keranjang.
- AC3. Given katalog outlet tidak dapat diambil, When pengguna membuka outlet itu, Then berlaku KU-08.

#### US-702 Detail produk

Sebagai pasien, saya ingin melihat detail produk, agar yakin sebelum membeli.
Rujukan: BR-030, BR-031, BR-032, BR-116 · Audit: tidak ada aksi sensitif

- AC1. Given pasien membuka produk, When detail tampil, Then memuat nama, foto, deskripsi, kategori, harga (AB-10), stok, nama outlet, dan jenis outlet.
- AC2. Given produk berkategori obat bebas dari outlet yang bukan apotek atau klinik berizin, When data katalog diterima, Then produk tidak ditampilkan dan dilaporkan ke admin Rumah Sehat (BR-031).
- AC3. Given data rekanan menandai produk sebagai obat resep, When katalog diterima, Then produk ditolak dan tidak pernah ditampilkan (BR-032).
- AC4. Informasi produk yang wajib ditampilkan menurut ketentuan BPOM `[TIDAK DIKETAHUI]` (L-03).

#### US-703 Keranjang dan checkout

Sebagai pasien, saya ingin memasukkan produk ke keranjang dan membayar beserta ongkos kirim, agar barang dikirim ke alamat saya.
Rujukan: BR-034, BR-040, AB-11, AB-15 `[UP-09]` `[UP-25]` · Audit: E-08, E-09

- AC1. Given pasien menambah produk, When produk berasal dari outlet berbeda, Then keranjang dipisah per outlet, dan setiap checkout hanya memuat satu outlet.
- AC2. Given jumlah melebihi stok, When ditambahkan, Then jumlah dibatasi sesuai stok dengan pesan.
- AC3. Given pasien memilih alamat, When ia melanjutkan, Then pilihan kurir dan ongkos kirim dari kurir terintegrasi tampil. Kalau kurir tidak melayani alamat itu, checkout tidak dapat dilanjutkan dan pesannya tampil.
- AC4. Given pasien menekan bayar, When harga atau stok di sistem outlet berbeda dari tampilan, Then pasien diberi tahu nilai terbaru dan harus menyetujuinya sebelum pembayaran dibuat (AB-15).
- AC5. Given pembayaran berhasil, When notifikasi diterima, Then pesanan berstatus "Dibayar" dan dikirim ke sistem outlet (US-1403).

#### US-704 Status dan pelacakan pesanan

Sebagai pasien, saya ingin melihat status dan pelacakan pesanan, agar tahu kapan barang tiba.
Rujukan: BR-033, BR-034, RF-08, RF-09, PB-06 · Audit: E-08, E-09

- AC1. Given outlet menerima pesanan, When status diterima dari sistem outlet, Then status menjadi "Diproses outlet".
- AC2. Given outlet menolak pesanan atau tidak menerimanya sampai PB-06, When kondisi terjadi, Then status menjadi "Ditolak outlet" atau "Tidak diproses", refund RF-08 berjalan otomatis, dan pasien diberi tahu.
- AC3. Given barang diserahkan ke kurir, When nomor resi dan status kurir diterima, Then status menjadi "Dikirim" dan pasien dapat melihat riwayat pelacakan.
- AC4. Given kurir mengirim status gagal kirim, When diterima, Then status menjadi "Gagal kirim", refund RF-09 berjalan otomatis, dan pasien diberi tahu.

#### US-705 Konfirmasi terima dan klaim barang

Sebagai pasien, saya ingin mengonfirmasi barang diterima atau mengajukan klaim bila rusak atau tidak sesuai, agar masalah pesanan ditangani.
Rujukan: BR-045, RF-10, PB-04 `[USER] T3-Q8` · Audit: E-08, E-09

- AC1. Given kurir mengirim status terkirim atau pasien menekan "Pesanan diterima", When kejadian pertama terjadi, Then status menjadi "Diterima".
- AC2. Given status "Diterima" dan PB-04 belum habis, When pasien mengajukan klaim dengan alasan (rusak, tidak sesuai, kurang) dan minimal satu foto, Then klaim tercatat (US-807) dan status menjadi "Klaim".
- AC3. Given PB-04 sudah habis, When pasien mencari tombol klaim, Then tombol tidak tersedia dan status menjadi "Selesai".
- AC4. Given pasien mencoba membatalkan pesanan setelah membayar, When ia mencari fitur pembatalan, Then fitur tidak tersedia (RF-11).

#### US-706 Riwayat pesanan

Sebagai pasien, saya ingin melihat riwayat pesanan produk, agar dapat melacak pembelian lama.
Rujukan: BR-035 · Audit: tidak ada aksi sensitif

- AC1. Given pasien membuka riwayat pesanan, When daftar tampil, Then setiap pesanan memuat tanggal, outlet, produk, total, dan status.
- AC2. Given pasien membuka detail, When tampil, Then rincian pembayaran (US-801) dan riwayat status ikut tampil.

### EP-08 Pembayaran, harga, pembagian dana, dan refund (T1)

| Story | Fitur | BR | Prioritas |
| --- | --- | --- | --- |
| US-801 | Ringkasan harga sebelum membayar | BR-042, BR-044, BR-024 | Must |
| US-802 | Pembayaran lewat payment gateway | BR-040, BR-115 | Must |
| US-803 | Pembagian dana per transaksi | BR-041, BR-044 | Must |
| US-804 | Bukti pembayaran | BR-040 | Must |
| US-805 | Pembatalan layanan terjadwal oleh pasien | BR-045 | Must |
| US-806 | Refund otomatis | BR-045 | Must |
| US-807 | Klaim refund | BR-045 | Must |
| US-808 | Eksekusi dan status refund | BR-045 | Must |
| US-809 | Pengaturan markup per rekanan | BR-042, BR-063 | Must |
| US-810 | Pengaturan biaya rekanan | BR-043, BR-063 | Must |
| US-811 | Laporan transaksi dan rekonsiliasi pembagian dana | BR-046 | Must |
| US-812 | Pengaturan parameter bisnis | BR-045, BR-063 | Must |

#### US-801 Ringkasan harga sebelum membayar

Sebagai pasien, saya ingin melihat rincian harga sebelum membayar, agar tahu persis apa yang saya bayar.
Rujukan: BR-042, BR-044, BR-024, AB-10–AB-16 `[UP-07]` `[UP-43]` · Audit: tidak ada aksi sensitif

- AC1. Given pasien di ringkasan pesanan, When ia memilih metode pembayaran, Then tampil harga layanan (satu angka), ongkos kirim (produk), biaya transaksi metode itu (Rp0 untuk QRIS), dan total, semuanya dihitung server.
- AC2. Given markup berbentuk persen, When harga dihitung, Then hasilnya dibulatkan ke Rupiah penuh terdekat (setengah ke atas) `[UP-43]`.
- AC3. Given pesanan layanan terjadwal, When ringkasan tampil, Then ringkasan aturan pembatalan (PB-01, no-show) tampil. Untuk klinik, lab, dan home visit, pernyataan AB-16 juga tampil.
- AC4. Given pesanan produk, When ringkasan tampil, Then pernyataan bahwa pesanan tidak dapat dibatalkan setelah dibayar tampil (RF-11).

#### US-802 Pembayaran lewat payment gateway

Sebagai pasien, saya ingin membayar dengan metode yang tersedia, agar pesanan saya terkonfirmasi.
Rujukan: BR-040, BR-115, AB-20, AB-25, PB-07 `[UP-34]` · Audit: E-08

- AC1. Given pasien memilih metode dan menekan bayar, When pembayaran dibuat di payment gateway, Then nominalnya sama dengan total di ringkasan, dan pasien diarahkan ke instruksi atau halaman pembayaran.
- AC2. Given notifikasi pembayaran berhasil diterima dan terverifikasi berasal dari payment gateway, When diproses, Then pesanan dikonfirmasi satu kali saja walaupun notifikasi diterima berulang (KU-05).
- AC3. Given nominal di notifikasi tidak sama dengan nominal pesanan, When diproses, Then pesanan tidak dikonfirmasi dan ditandai untuk peran Keuangan.
- AC4. Given pembayaran gagal, When pasien kembali ke aplikasi, Then ia dapat mencoba metode lain selama PB-07 belum habis.
- AC5. Given notifikasi berhasil datang setelah pesanan kedaluwarsa, When slot atau stok masih tersedia, Then pesanan dikonfirmasi. Kalau tidak tersedia, refund penuh berjalan otomatis dengan penanggung Rumah Sehat `[UP-34]`.

#### US-803 Pembagian dana per transaksi

Sebagai Rumah Sehat, saya ingin setiap pembayaran dibagi langsung oleh payment gateway dan tercatat, agar bagian rekanan tidak pernah ditampung PT. Arunika.
Rujukan: BR-041, BR-044, AB-21–AB-23, AB-26 · Audit: E-08

- AC1. Given pembayaran berhasil, When transaksi dicatat, Then tercatat bagian rekanan (AB-22), markup Rumah Sehat, MDR atau biaya transaksi, dan rekening tujuan, sebagai instruksi pembagian ke payment gateway.
- AC2. Given payment gateway melaporkan hasil pembagian dana, When laporan diterima, Then hasil per pihak tercatat dan dapat dicocokkan (US-811).
- AC3. Given catatan transaksi sudah dibuat, When ada koreksi, Then catatan asal tidak diubah, dan koreksi dibuat sebagai entri pembalik (AB-26).
- AC4. Penyaluran dana ongkos kirim `[TIDAK DIKETAHUI]` (AB-23, PT-31).

#### US-804 Bukti pembayaran

Sebagai pasien, saya ingin menerima bukti pembayaran, agar punya catatan transaksi.
Rujukan: BR-040, BR-014 · Audit: tidak ada aksi sensitif

- AC1. Given pembayaran berhasil, When bukti dibuat, Then bukti tersedia di aplikasi dan dikirim ke email. Isinya: nomor transaksi, tanggal, rincian harga (US-801), metode, nama rekanan pemberi layanan atau penjual, dan pernyataan bahwa Rumah Sehat berperan sebagai platform.
- AC2. Kolom pajak dan identitas *merchant of record* `[TIDAK DIKETAHUI]` (PT-28, L-12).

#### US-805 Pembatalan layanan terjadwal oleh pasien

Sebagai pasien, saya ingin membatalkan layanan terjadwal, agar slot dapat dipakai orang lain dan saya mendapat refund bila masih dalam batas waktu.
Rujukan: BR-045, RF-01, RF-02, PB-01 `[USER] T3-Q5` · Audit: E-08, E-09

- AC1. Given layanan "Terkonfirmasi" dan sisa waktu sebelum jadwal minimal PB-01, When pasien menekan batal, Then pasien melihat nominal refund (harga layanan; biaya transaksi non-QRIS tidak kembali) dan harus mengonfirmasi.
- AC2. Given pasien mengonfirmasi, When pembatalan disimpan, Then status menjadi "Dibatalkan pasien", slot dilepas, rekanan diberi tahu lewat integrasi, dan refund RF-01 berjalan otomatis.
- AC3. Given sisa waktu kurang dari PB-01, When pasien menekan batal, Then pasien diberi tahu tidak ada refund (RF-02). Kalau tetap dibatalkan, status menjadi "Dibatalkan pasien" tanpa refund dan slot dilepas.
- AC4. Given layanan sudah dimulai, check-in, atau berstatus akhir, When pasien mencari tombol batal, Then tombol tidak tersedia.

#### US-806 Refund otomatis

Sebagai pasien, saya ingin refund berjalan otomatis untuk kejadian yang tercatat sistem, agar tidak perlu mengajukan klaim.
Rujukan: BR-045, RF-01, RF-04, RF-05, RF-08, RF-09, RF-12 `[USER] T3-Q7, T3-Q8` · Audit: E-08

- AC1. Given salah satu kejadian RF-01, RF-04, RF-05, RF-08, RF-09, atau RF-12 terjadi, When sistem mencatatnya, Then refund dibuat dengan nominal dan penanggung sesuai matriks §3.5 tanpa persetujuan manual, dan pasien diberi tahu.
- AC2. Given refund untuk satu pesanan sudah dibuat, When kejadian yang sama tercatat ulang, Then refund kedua tidak dibuat (KU-05).
- AC3. Given refund RF-01, When nominal dihitung, Then nominalnya harga layanan tanpa biaya transaksi non-QRIS. Given RF-04, RF-05, RF-08, RF-09, atau RF-12, Then nominalnya seluruh pembayaran.

#### US-807 Klaim refund

Sebagai pasien, saya ingin mengajukan klaim untuk kejadian yang perlu diperiksa, agar mendapat refund bila klaim saya benar.
Rujukan: BR-045, RF-06, RF-07, RF-10, PB-04, PB-05 `[USER] T3-Q8` `[UP-36]` · Audit: E-05 (membuka bukti), E-08

- AC1. Given kejadian RF-06, RF-07, atau RF-10 dan masih dalam batas pengajuan, When pasien mengirim klaim dengan alasan dan bukti (foto wajib untuk RF-10), Then klaim berstatus "Diajukan" dan pasien melihat batas keputusan PB-05.
- AC2. Given pesanan sudah punya klaim aktif, When pasien mengajukan klaim lagi, Then ditolak (KU-05).
- AC3. Given Layanan Pelanggan menyetujui klaim, When keputusan disimpan, Then refund dibuat dengan nominal dan penanggung sesuai matriks, dan pasien diberi tahu.
- AC4. Given Layanan Pelanggan menolak klaim, When alasan diisi, Then pasien diberi tahu alasannya. Penolakan tanpa alasan tidak dapat disimpan.
- AC5. Given PB-05 habis tanpa keputusan, When batas terlewati, Then klaim ditandai terlambat dan dieskalasi ke Super Admin, tanpa disetujui atau ditolak otomatis (AB-33).

#### US-808 Eksekusi dan status refund

Sebagai Keuangan, saya ingin refund dieksekusi dan dipantau sampai berhasil, agar dana pasien kembali dan pencatatan lengkap.
Rujukan: BR-045, AB-30–AB-32, ADR-004 `[UP-35]` · Audit: E-08

- AC1. Given refund dibuat, When dieksekusi lewat payment gateway, Then status "Diproses" lalu "Berhasil" setelah dikonfirmasi penyedia, dan entri pembalik untuk bagian rekanan dan markup tercatat (AB-31).
- AC2. Given eksekusi gagal, When penyedia mengembalikan galat, Then status menjadi "Gagal", refund masuk antrean Keuangan, pasien diberi tahu bahwa refund sedang ditangani, dan Keuangan dapat memproses ulang.
- AC3. Given refund menimbulkan kewajiban rekanan mengembalikan bagiannya, When refund dibuat, Then nominal kewajiban tercatat dan tampil di laporan rekanan (US-1201) dan laporan Keuangan sampai dinyatakan selesai. Mekanisme penarikan dana ditetapkan di Tahap 4–5 (PT-27) `[UP-35]`.

#### US-809 Pengaturan markup per rekanan

Sebagai Keuangan, saya ingin mengatur markup per rekanan, agar harga pasien sesuai keputusan CEO.
Rujukan: BR-042, BR-063, PB-11 `[UP-44]` · Audit: E-14

- AC1. Given Keuangan memilih rekanan, When ia mengisi jenis (persen atau nominal), nilai (persen 0–100; nominal Rupiah 0 atau lebih), dan waktu berlaku (segera atau tanggal mendatang), Then pengaturan tersimpan dan riwayat perubahan tetap ada.
- AC2. Given markup diubah, When pesanan dibayar sebelum waktu berlaku, Then harga pesanan itu tidak berubah (AB-17).
- AC3. Given rekanan belum punya pengaturan markup, When Verifikator mencoba menampilkannya ke pasien, Then rekanan tidak tampil sampai markup diatur. Nilai 0 diperbolehkan (US-205 AC2).
- AC4. Given pengguna bukan Keuangan, When ia mencoba mengubah markup, Then ditolak (KU-03).

#### US-810 Pengaturan biaya rekanan

Sebagai Keuangan, saya ingin mengatur biaya rekanan untuk klinik dan lab, agar tagihan biaya rekanan dapat dihitung bila CEO memutuskan memakainya.
Rujukan: BR-043, BR-063, PB-12, AB-24 · Audit: E-14

- AC1. Given Keuangan memilih klinik atau lab, When ia mengisi jenis (tidak ada, langganan per bulan, atau per pendaftaran), nilai, dan waktu berlaku, Then pengaturan tersimpan dengan riwayat.
- AC2. Given jenis per pendaftaran atau langganan, When periode bulan berakhir, Then sistem menghitung tagihan biaya rekanan per rekanan dan menampilkannya di laporan Keuangan dan portal rekanan.
- AC3. Cara penagihan dan penyetoran `[TIDAK DIKETAHUI]` (AB-24, PT-17).

#### US-811 Laporan transaksi dan rekonsiliasi pembagian dana

Sebagai Keuangan, saya ingin melihat laporan transaksi dan mencocokkan pembagian dana, agar selisih segera diketahui.
Rujukan: BR-046 · Audit: E-12 (export)

- AC1. Given Keuangan memilih periode, rekanan, jenis layanan, dan status, When laporan tampil, Then setiap transaksi memuat total, bagian rekanan, markup, MDR atau biaya transaksi, refund, dan status pembagian dari payment gateway, dalam WIB (AB-42).
- AC2. Given rekonsiliasi harian berjalan, When data penyedia dibandingkan dengan catatan Rumah Sehat, Then selisih nominal, pembagian yang tidak terlaporkan, dan pembayaran ganda (RF-12) ditandai untuk ditindaklanjuti.
- AC3. Given Keuangan mengekspor laporan, When file dibuat, Then file berformat CSV dan event export tercatat.

#### US-812 Pengaturan parameter bisnis

Sebagai Super Admin, saya ingin mengatur parameter bisnis, agar aturan operasional dapat disesuaikan tanpa mengubah aplikasi.
Rujukan: BR-045, BR-063, §3.7 `[UP-32]` · Audit: E-14

- AC1. Given Super Admin mengubah parameter PB-01–PB-10 atau PB-14, When nilainya valid (angka positif dengan satuan yang ditentukan), Then perubahan tersimpan dengan riwayat.
- AC2. Given parameter diubah, When pesanan yang dibayar sebelum perubahan dievaluasi (misalnya batas pembatalan), Then nilai yang berlaku saat pesanan dibayar yang dipakai `[UP-32]`.
- AC3. Given nilai tidak valid (kosong, negatif, atau nol untuk batas waktu), When disimpan, Then ditolak (KU-01).

### EP-09 Hasil lab

| Story | Fitur | BR | Prioritas |
| --- | --- | --- | --- |
| US-901 | Penerimaan hasil lab | BR-025, BR-070 | Must |
| US-902 | Lihat dan unduh hasil lab | BR-025, BR-050 | Must |

#### US-901 Penerimaan hasil lab

Sebagai Rumah Sehat, saya ingin menerima hasil lab dari sistem lab rekanan, agar pasien mendapat hasilnya di aplikasi.
Rujukan: BR-025, BR-070, AB-73 · Audit: E-06, E-17

- AC1. Given sistem lab (sistem sendiri atau RME minimalis) mengirim hasil untuk pendaftaran lab yang valid, When diterima, Then hasil tersimpan (dokumen PDF dan/atau nilai terstruktur) dan pasien diberi tahu tanpa menyebut nilai hasil (AB-73).
- AC2. Given hasil dikirim untuk pendaftaran yang tidak ada atau milik rekanan lain, When diterima, Then hasil ditolak dan galat integrasi tercatat.
- AC3. Given lab mengirim koreksi hasil, When diterima, Then versi baru tersimpan, versi lama tetap ada dan ditandai sudah dikoreksi, dan pasien diberi tahu ada pembaruan.

#### US-902 Lihat dan unduh hasil lab

Sebagai pasien, saya ingin melihat dan mengunduh hasil lab saya dan anggota keluarga, agar dapat memakainya.
Rujukan: BR-025, BR-050, AB-71 · Audit: E-12 (unduh)

- AC1. Given hasil tersedia, When pasien membukanya, Then hasil tampil di aplikasi dan dapat diunduh sebagai PDF.
- AC2. Given pengguna lain, When mencoba membuka hasil lab pasien, Then ditolak (§3.1, KU-03).
