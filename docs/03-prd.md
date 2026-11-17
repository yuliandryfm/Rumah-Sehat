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
| Sub-peran admin Rumah Sehat: Super Admin, Verifikator, Layanan Pelanggan, Keuangan, Konten, Auditor, Petugas PDP | Backoffice | `[UP-18]` |
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

1. Antrean real-time klinik (EP-04 US-405, EP-13 US-1305)
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

| Data | Pasien | Tenaga kesehatan | Admin klinik/lab/outlet (portal) | Admin RS | Pengguna RME minimalis | Sensitif | Sumber |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Profil dan identitas dasar | Miliknya dan anggota keluarganya | Nama, jenis kelamin, tanggal lahir, dan keluhan awal pasien di sesi/kunjungan yang ditanganinya. Untuk home visit juga alamat kunjungan dan nomor HP pasien, hanya di hari kunjungan sampai kunjungan selesai | Nama pasien di laporan transaksi rekanannya | Layanan Pelanggan, Verifikator, Super Admin; Petugas PDP untuk pemohon hak subjek data | Pasien Rumah Sehat di fasyankesnya | Ya untuk keluhan awal dan data anak; tidak untuk profil dewasa lainnya | BR-002, BR-105 `[UP-13]` `[UP-17]` |
| NIK, foto KTP, foto KK | Miliknya dan anggota keluarganya | Tidak | Tidak | NIK lengkap dan foto hanya Verifikator. Peran lain, termasuk Petugas PDP, melihat NIK termasking | NIK pasien di fasyankesnya bila dibutuhkan pencatatan rekam medis `[TIDAK DIKETAHUI]` (L-05) | Ya | BR-003, BR-004, BR-105 `[UP-17]` |
| Isi chat, catatan konsultasi, resep | Miliknya dan anggota keluarganya (UP-15) | Hanya sesi yang ditanganinya | Tidak | Tidak ada peran admin RS yang dapat melihat. Salinan untuk hak subjek data dibuat otomatis oleh sistem, tanpa dibuka Petugas PDP | Tidak | Ya | BR-016, BR-105 `[UP-15]` `[UP-17]` |
| Hasil lab | Miliknya dan anggota keluarganya (UP-15) | Tidak, di rilis 1 | Tidak | Tidak (seperti baris di atas) | Petugas Lab dan Tenaga Kesehatan Fasyankes di lab/klinik yang memeriksa | Ya | BR-025, BR-105 `[UP-17]` |
| Rekam medis RME minimalis | `[TIDAK DIKETAHUI]`, menunggu L-05 (sama dengan US-1604 AC3) | Tidak | Tidak | Tidak (Operator RME juga tidak) | Tenaga Kesehatan Fasyankes: baca dan tulis. Admin Fasyankes: baca saja. Petugas Lab: tidak | Ya | BR-123 `[UP-17]` `[UP-26]` |
| Foto bukti klaim | Miliknya | Tidak | Tidak | Layanan Pelanggan (klaim yang ditanganinya); Super Admin (klaim tereskalasi) | Tidak | Ya | BR-045 `[UP-36]` |
| Transaksi, pembagian dana, refund | Transaksinya | Tidak | Transaksi rekanannya | Keuangan; Super Admin (baca); Layanan Pelanggan (baca, hanya pesanan terkait klaim atau tiket yang ditanganinya) | Tidak | Ya untuk detail transaksi seorang pasien yang dibuka admin (data keuangan pribadi); tidak untuk laporan berisi banyak transaksi | BR-046, BR-105 `[UP-17]` |
| Permintaan hak subjek data | Miliknya | Tidak | Tidak | Petugas PDP; Super Admin (baca) | Tidak | Tidak (dicatat sebagai E-13) | BR-106 `[UP-18]` |
| Audit log | Tidak di rilis 1 | Tidak | Tidak | Auditor (baca dan export), Super Admin (baca) | Admin Fasyankes: log akses rekam medis fasyankesnya | Tidak | BR-080 `[UP-18]` `[UP-26]` |

**Data sensitif** untuk KU-03 dan E-05 adalah data bertanda "Ya" di kolom Sensitif: data pribadi spesifik (AB-70: data kesehatan termasuk keluhan awal, data anak, data keuangan pribadi), NIK lengkap, dokumen identitas, dan foto bukti klaim. E-05 dicatat saat detail data itu dibuka oleh pihak selain subjek data, bukan saat daftar ringkas tampil. Akses yang ditolak ke data sensitif dicatat dengan hasil "gagal" (KU-03, §7) `[KEPUTUSAN-07]`.

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
| AB-11 | Untuk produk, total = jumlah harga produk (harga rekanan + markup, per unit dikali jumlah) + ongkos kirim | BR-034, BR-042 |
| AB-11a | Markup dihitung per unit layanan atau produk atas harga rekanan per unit. Markup persen dibulatkan per unit ke Rupiah penuh terdekat (setengah ke atas), lalu dikalikan jumlah. Markup nominal berlaku per unit. Markup tidak dikenakan pada ongkos kirim dan biaya transaksi | `[UP-51]` |
| AB-12 | Untuk metode pembayaran non-QRIS, biaya transaksi ditambahkan ke total dan ditampilkan sebelum pasien membayar. Untuk QRIS, pasien tidak dikenai biaya transaksi, dan MDR dipotong dari bagian rekanan. Pembebanan per metode menunggu L-12 | BR-044 `[SRC-24]` |
| AB-13 | Pasien melihat harga layanan sebagai **satu angka** (harga rekanan + markup). Ongkos kirim dan biaya transaksi tampil sebagai baris terpisah. Markup tidak ditampilkan sebagai baris tersendiri | `[UP-07]` (sejalan dengan T2-Q20); keterbukaan rincian harga menunggu L-14 |
| AB-14 | Informasi biaya MCU dan lab menampilkan harga yang sama dengan harga bayar di aplikasi | BR-023 `[USER] T2-Q20` |
| AB-15 | Sebelum pembayaran dibuat, harga dan ketersediaan (slot atau stok) diperiksa ulang ke sistem rekanan. Kalau berbeda, pasien diberi tahu nilai terbaru dan harus menyetujuinya sebelum membayar | `[UP-25]` |
| AB-16 | Layar ringkasan pesanan klinik, lab, dan home visit menyatakan bahwa harga hanya mencakup layanan yang tercantum, dan tindakan tambahan diselesaikan serta dibayar di luar aplikasi | BR-024 (L-14) |
| AB-17 | Harga (harga rekanan atau tarif + markup) dan nilai parameter bisnis yang berlaku untuk suatu pesanan dikunci saat pembayaran dibuat di payment gateway. Perubahan sesudahnya tidak memengaruhi pesanan itu, walaupun pembayaran diselesaikan setelah perubahan berlaku. Perubahan markup atau tarif antara ringkasan harga dan pembuatan pembayaran ditangani seperti AB-15 | ADR-004 `[UP-32]` |

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
- **Bergabung (konsultasi)**: pihak membuka ruang sesi di aplikasinya saat ruang sudah terbuka. Server mencatat waktu pertama kali tiap pihak bergabung. Berlaku sama untuk chat dan video; menjawab panggilan video tidak disyaratkan `[UP-29]`.
- **Batas ketidakhadiran konsultasi**: PB-02 (pasien) atau PB-03 (tenaga kesehatan) sejak jadwal mulai, atau akhir durasi sesi bila lebih dulu `[UP-29]`.
- **Pasien tidak hadir (no-show)**:
    - konsultasi: tenaga kesehatan sudah bergabung, tetapi pasien belum bergabung saat batas ketidakhadiran pasien habis;
    - klinik/lab: appointment yang sudah diterima sistem rekanan tidak di-check-in sampai jadwalnya berakhir;
    - home visit: tenaga kesehatan menandai pasien tidak ada di lokasi setelah berstatus "Tiba".
- **Tenaga kesehatan tidak hadir**:
    - konsultasi: tenaga kesehatan belum bergabung saat batas ketidakhadiran tenaga kesehatan habis `[USER] T3-Q8`;
    - home visit: tenaga kesehatan belum menandai "Tiba" sampai PB-03 habis sejak jadwal mulai, lalu pasien melaporkannya `[UP-13]`.
- **Harga layanan**: harga rekanan + markup (AB-10, AB-11a).
- **Seluruh pembayaran**: harga layanan + ongkos kirim (produk) + biaya transaksi (non-QRIS).

**Matriks refund**

| # | Kejadian | Berlaku untuk | Refund? | Yang dikembalikan ke pasien | Kelompok penanggung (AB-34) | Cara | Sumber |
| --- | --- | --- | --- | --- | --- | --- | --- |
| RF-01 | Pasien membatalkan paling lambat PB-01 sebelum jadwal | Layanan terjadwal | Ya | Harga layanan. Biaya transaksi non-QRIS tidak dikembalikan | Pembatalan pasien | Otomatis | BR-045 `[USER] T3-Q5, T3-Q7, T3-Q8` |
| RF-02 | Pasien membatalkan kurang dari PB-01 sebelum jadwal | Layanan terjadwal | Tidak | — | — | — | BR-045 |
| RF-03 | Pasien tidak hadir | Layanan terjadwal | Tidak | — | Rekanan tetap menerima bagiannya | — | `[USER] T3-Q6` |
| RF-04 | Tenaga kesehatan tidak hadir di sesi konsultasi | Konsultasi | Ya | Seluruh pembayaran | Penyebab rekanan | Otomatis | `[USER] T3-Q7, T3-Q8` |
| RF-05 | Rekanan atau tenaga kesehatan membatalkan, termasuk akibat penonaktifan akun rekanan atau tenaga kesehatan dan SIP berakhir | Layanan terjadwal | Ya | Seluruh pembayaran | Penyebab rekanan | Otomatis | BR-045 `[USER] T3-Q7, T3-Q8` |
| RF-06 | Tenaga kesehatan home visit tidak datang, dilaporkan pasien paling lambat PB-15 setelah jadwal berakhir | Home visit | Ya, bila klaim disetujui | Seluruh pembayaran | Penyebab rekanan | Pengajuan, diputuskan dalam PB-05 | `[USER] T3-Q7` `[UP-13]` `[UP-53]` |
| RF-07 | Layanan gagal karena gangguan platform Rumah Sehat, diajukan paling lambat PB-15 setelah jadwal berakhir (layanan terjadwal) atau PB-04 setelah status terakhir pesanan (produk) | Semua | Ya, bila klaim disetujui | Seluruh pembayaran | Penyebab platform | Pengajuan, diputuskan dalam PB-05 | `[USER] T3-Q7` `[UP-53]` |
| RF-08 | Outlet menolak pesanan (misalnya stok habis), tidak menerimanya sampai PB-06, membatalkannya sebelum diserahkan ke kurir, atau dinonaktifkan sebelum pesanan dikirim | Produk | Ya | Seluruh pembayaran | Penyebab rekanan | Otomatis | `[USER] T3-Q7, T3-Q8` `[UP-03]` |
| RF-09 | Kurir gagal mengirim (status gagal dari kurir) | Produk | Ya | Seluruh pembayaran | Penyebab kurir `[TIDAK DIKETAHUI]` (PT-29) | Otomatis | `[USER] T3-Q7, T3-Q8` |
| RF-10 | Barang rusak, tidak sesuai, atau kurang, diajukan paling lambat PB-04 setelah status Diterima, dengan foto | Produk | Ya, bila klaim disetujui | Seluruh pembayaran. Klaim sebagian barang `[TIDAK DIKETAHUI]` (PT-33) | Penyebab rekanan. Kerusakan dalam pengiriman: penyebab kurir (PT-29). Pengembalian barang fisik `[TIDAK DIKETAHUI]` (PT-30) | Pengajuan, diputuskan dalam PB-05 | BR-045 `[USER] T3-Q7, T3-Q8` |
| RF-11 | Pasien ingin membatalkan pesanan produk setelah membayar | Produk | Tidak. Pembatalan tidak disediakan | — | — | — | BR-045 `[UP-08]` |
| RF-12 | Pasien terdebit dua kali untuk satu pesanan (pembayaran ganda) | Semua | Ya, untuk pembayaran ganda | Seluruh pembayaran ganda | Penyebab platform | Otomatis saat notifikasi berhasil kedua diterima, atau saat terdeteksi di rekonsiliasi | `[USER] T3-Q7` `[UP-34]` |
| RF-13 | Pembayaran berhasil diterima setelah pesanan kedaluwarsa, dan slot atau stok sudah tidak tersedia | Semua | Ya | Seluruh pembayaran | Penyebab platform | Otomatis | `[USER] T3-Q7` `[UP-34]` |
| RF-14 | Nominal di notifikasi pembayaran tidak sama dengan nominal pesanan | Semua | Ya. Pesanan tidak dikonfirmasi dan menjadi Kedaluwarsa | Seluruh nominal yang diterima | Penyebab platform | Keuangan memeriksa dan membuat refund dalam PB-16. Bila PB-16 habis tanpa tindakan, refund dibuat otomatis dan Super Admin diberi tahu | `[UP-52]` |
| RF-15 | Outlet sudah menerima pesanan tetapi tidak menyerahkannya ke kurir sampai PB-17 sejak status Diproses outlet | Produk | Ya | Seluruh pembayaran | Penyebab rekanan | Otomatis | `[USER] T3-Q7` `[UP-52]` |
| RF-16 | Barang tidak sampai: status Dikirim tanpa kabar terkirim atau gagal sampai PB-18, atau kurir menyatakan terkirim tetapi pasien tidak menerima (diajukan paling lambat PB-04 setelah status Diterima, atau paling lambat PB-04 setelah PB-18 habis untuk jalur Dikirim) | Produk | Ya, bila klaim disetujui | Seluruh pembayaran | Penyebab kurir `[TIDAK DIKETAHUI]` (PT-29) | Pengajuan tanpa wajib foto, diputuskan dalam PB-05 | `[USER] T3-Q7` `[UP-52]` |
| RF-17 | Pasien keberatan atas status "Pasien tidak hadir" home visit, diajukan paling lambat PB-15 setelah jadwal berakhir | Home visit | Ya, bila klaim disetujui | Seluruh pembayaran | Penyebab rekanan | Pengajuan, diputuskan dalam PB-05 | `[USER] T3-Q7` `[UP-13]` `[UP-53]` |
| RF-18 | Pesanan klinik, lab, atau home visit yang sudah dibayar belum diterima sistem rekanan saat jadwal mulai | Pendaftaran klinik/lab, home visit | Ya | Seluruh pembayaran, dibuat otomatis saat jadwal mulai | Menunggu penetapan penyebab: Keuangan menetapkan penyebab rekanan (sistem rekanan menolak atau tidak tersedia) atau penyebab platform (gangguan Rumah Sehat) dari log integrasi dalam PB-20. Bila PB-20 habis tanpa penetapan, berlaku penyebab platform | Refund ke pasien otomatis; penetapan penyebab manual | `[USER] T3-Q7` `[UP-56]` |

Ketentuan tambahan:

- **AB-30** Refund dieksekusi lewat payment gateway ke metode bayar asal bila penyedia mendukungnya `[TIDAK DIKETAHUI]` (Tahap 4–5). Waktu sampainya dana ke pasien mengikuti penyedia dan metode pembayaran `[TIDAK DIKETAHUI]`.
- **AB-31** Setiap refund dicatat sebagai entri pembalik yang merujuk transaksi asal (ADR-004), termasuk kewajiban rekanan dan Rumah Sehat menurut AB-34.
- **AB-32** Refund yang gagal dieksekusi berstatus "Gagal" dan masuk antrean tindak lanjut peran Keuangan. Pasien diberi tahu bahwa refund sedang ditangani `[UP-35]`.
- **AB-33** Klaim refund yang tidak diputuskan sampai PB-05 habis dieskalasi ke Super Admin dan ditandai terlambat di laporan. Super Admin dapat memutuskan klaim tereskalasi beserta melihat buktinya. Klaim tidak otomatis disetujui atau ditolak `[UP-36]`.
- **AB-34** Rumus kewajiban pengembalian per kelompok penanggung. "Biaya payment gateway yang tidak kembali" mencakup MDR dan biaya transaksi sejauh tidak dikembalikan penyedia `[TIDAK DIKETAHUI]` (Tahap 4–5).

  | Kelompok | Pasien menerima | Kewajiban rekanan | Kewajiban Rumah Sehat | Sumber |
  | --- | --- | --- | --- | --- |
  | Pembatalan pasien (RF-01) | Harga layanan | Harga rekanan penuh (MDR QRIS tetap ditanggung rekanan) | Markup | `[USER] T3-Q7` |
  | Penyebab rekanan (RF-04, RF-05, RF-06, RF-08, RF-10, RF-15, RF-17; RF-18 bila ditetapkan penyebab rekanan) | Seluruh pembayaran | Harga rekanan penuh + biaya payment gateway yang tidak kembali | Markup | `[USER] T3-Q7` |
  | Penyebab platform (RF-07, RF-12, RF-13, RF-14; RF-18 bila ditetapkan penyebab platform atau PB-20 habis) | Seluruh pembayaran yang di-refund | Bagian yang benar-benar diterimanya dari pembayaran itu (harga rekanan dikurangi MDR bila QRIS) | Markup + semua biaya payment gateway yang tidak kembali | `[USER] T3-Q7` `[UP-34]` |
  | Penyebab kurir (RF-09, RF-16, kerusakan dalam pengiriman) | Seluruh pembayaran | `[TIDAK DIKETAHUI]` (PT-29) | `[TIDAK DIKETAHUI]` (PT-29) | — |

  Ongkos kirim dikembalikan oleh penerima dana ongkos kirim (AB-23, PT-31). Nominal kewajiban dihitung dari harga yang dikunci (AB-17).
- **AB-35** Refund yang berstatus Diproses lebih dari PB-19 tanpa konfirmasi penyedia menjadi "Perlu dicek" dan masuk antrean Keuangan. Status ditanyakan ke penyedia, dan refund tidak dieksekusi ulang sebelum status penyedia diketahui. Keuangan menandai kewajiban rekanan "Selesai" dengan rujukan bukti `[UP-55]`.
- **AB-36** Satu pesanan paling banyak punya satu refund, kecuali RF-12. Bila dua kejadian datang hampir bersamaan, transisi status yang tersimpan lebih dulu yang berlaku, dan kejadian berikutnya yang tidak sah dari status baru ditolak dan tercatat (E-09, hasil gagal) `[UP-10]`.

### 3.6 Status layanan dan pesanan

Status dan transisi berikut usulan penyusun `[UP-10]`. Perubahan status dicatat di audit trail (E-09). Transisi yang tidak ada di tabel ditolak, dan penolakannya tercatat dengan hasil gagal (US-1404 AC2, AB-36). "Status akhir" adalah status tanpa transisi keluar.

**Transisi bersama semua pesanan**

| Dari | Ke | Pemicu | Aktor | Refund |
| --- | --- | --- | --- | --- |
| Menunggu pembayaran | Terkonfirmasi (layanan terjadwal) atau Dibayar (produk) | Notifikasi pembayaran berhasil dengan nominal sama, sebelum PB-07 habis | Sistem | — |
| Menunggu pembayaran | Kedaluwarsa | PB-07 habis, atau nominal notifikasi tidak sama | Sistem | RF-13 atau RF-14 bila dana sudah masuk |
| Menunggu pembayaran | Dibatalkan pasien | Pasien membatalkan sebelum membayar | Pasien | — |
| Kedaluwarsa | Terkonfirmasi atau Dibayar | Notifikasi berhasil datang setelah kedaluwarsa dan slot/stok masih tersedia | Sistem | — |

**Konsultasi online**

| Dari | Ke | Pemicu | Aktor | Refund |
| --- | --- | --- | --- | --- |
| Terkonfirmasi | Dibatalkan pasien | US-805 | Pasien | RF-01 atau RF-02 |
| Terkonfirmasi | Dibatalkan rekanan | Tenaga kesehatan membatalkan; akun tenaga kesehatan atau rekanan dinonaktifkan; SIP berakhir | Tenaga kesehatan, admin RS, sistem | RF-05 |
| Terkonfirmasi | Berlangsung | Pihak pertama bergabung (§3.5) | Sistem | — |
| Terkonfirmasi atau Berlangsung | Tenaga kesehatan tidak hadir | Batas ketidakhadiran tenaga kesehatan habis dan tenaga kesehatan belum bergabung (berlaku juga bila keduanya tidak bergabung) | Sistem | RF-04 |
| Berlangsung | Pasien tidak hadir | Tenaga kesehatan sudah bergabung dan batas ketidakhadiran pasien habis tanpa pasien bergabung | Sistem | — (RF-03) |
| Berlangsung | Selesai | Durasi sesi habis dan kedua pihak pernah bergabung | Sistem | — |

**Pendaftaran klinik dan lab**

| Dari | Ke | Pemicu | Aktor | Refund |
| --- | --- | --- | --- | --- |
| Terkonfirmasi | Dibatalkan pasien | US-805 | Pasien | RF-01 atau RF-02 |
| Terkonfirmasi | Dibatalkan rekanan | Rekanan membatalkan lewat integrasi; akun rekanan dinonaktifkan | Rekanan, admin RS | RF-05 |
| Terkonfirmasi | Dibatalkan sistem | Pesanan belum diterima sistem rekanan saat jadwal mulai | Sistem | RF-18 |
| Terkonfirmasi | Check-in | Status check-in dari sistem rekanan | Rekanan | — |
| Terkonfirmasi | Pasien tidak hadir | Jadwal berakhir tanpa check-in, dan pesanan sudah diterima sistem rekanan | Sistem | — (RF-03) |
| Check-in | Selesai | Status selesai dari sistem rekanan, atau akhir hari kalender rekanan | Rekanan, sistem | — |

Untuk lab, hasil lab adalah objek tersendiri yang terhubung ke pendaftaran (EP-09). Pembatalan oleh rekanan setelah Check-in ditolak.

**Home visit**

| Dari | Ke | Pemicu | Aktor | Refund |
| --- | --- | --- | --- | --- |
| Terkonfirmasi atau Ditugaskan | Dibatalkan pasien | US-805 | Pasien | RF-01 atau RF-02 |
| Terkonfirmasi atau Ditugaskan | Dibatalkan rekanan | Klinik membatalkan; akun klinik dinonaktifkan | Rekanan, admin RS | RF-05 |
| Terkonfirmasi | Dibatalkan sistem | Pesanan belum diterima sistem klinik saat jadwal mulai | Sistem | RF-18 |
| Terkonfirmasi | Ditugaskan | Penugasan valid dari sistem klinik (US-502) | Rekanan | — |
| Ditugaskan | Ditugaskan | Penugasan ulang ke tenaga kesehatan lain | Rekanan | — |
| Ditugaskan | Terkonfirmasi | Tenaga kesehatan yang ditugaskan dinonaktifkan atau SIP-nya berakhir; menunggu penugasan ulang | Sistem | — |
| Ditugaskan | Berangkat | Tenaga kesehatan menandai | Tenaga kesehatan | — |
| Berangkat | Tiba | Tenaga kesehatan menandai | Tenaga kesehatan | — |
| Tiba | Selesai | Tenaga kesehatan menandai | Tenaga kesehatan | — |
| Tiba | Selesai | Belum ditandai Selesai sampai akhir hari kalender klinik | Sistem | — |
| Terkonfirmasi, Ditugaskan, atau Berangkat | Ditutup tanpa konfirmasi | PB-15 habis setelah jadwal berakhir tanpa status Tiba dan tanpa laporan pasien; ditandai untuk ditindaklanjuti Layanan Pelanggan dan tidak dihitung di KPI-01 | Sistem | — |
| Tiba | Pasien tidak hadir | Tenaga kesehatan menandai pasien tidak ada di lokasi, dengan keterangan | Tenaga kesehatan | — (RF-03) |
| Terkonfirmasi, Ditugaskan, atau Berangkat | Dilaporkan tidak datang | Pasien melapor setelah PB-03 habis sejak jadwal mulai, paling lambat PB-15 setelah jadwal berakhir | Pasien | Klaim RF-06 |
| Dilaporkan tidak datang | Tenaga kesehatan tidak hadir | Klaim disetujui | Layanan Pelanggan, Super Admin | RF-06 |
| Dilaporkan tidak datang | Klaim ditolak | Klaim ditolak | Layanan Pelanggan, Super Admin | — |
| Pasien tidak hadir | Keberatan diajukan | Pasien keberatan paling lambat PB-15 setelah jadwal berakhir | Pasien | Klaim RF-17 |
| Keberatan diajukan | Tenaga kesehatan tidak hadir | Klaim disetujui | Layanan Pelanggan, Super Admin | RF-17 |
| Keberatan diajukan | Pasien tidak hadir | Klaim ditolak | Layanan Pelanggan, Super Admin | — |

Status dari tenaga kesehatan yang datang setelah pasien melapor "tidak datang" ditolak (AB-36). Keputusan klaim yang diambil sesudahnya memakai bukti dari kedua pihak.

**Pesanan produk**

| Dari | Ke | Pemicu | Aktor | Refund |
| --- | --- | --- | --- | --- |
| Dibayar | Diproses outlet | Outlet menerima pesanan | Rekanan | — |
| Dibayar | Ditolak outlet | Outlet menolak, PB-06 habis, atau outlet dinonaktifkan | Rekanan, sistem, admin RS | RF-08 |
| Diproses outlet | Ditolak outlet | Outlet membatalkan, atau outlet dinonaktifkan | Rekanan, admin RS | RF-08 |
| Diproses outlet | Tidak diserahkan | PB-17 habis tanpa barang diserahkan ke kurir | Sistem | RF-15 |
| Diproses outlet | Dikirim | Nomor resi dan status kurir diterima | Rekanan, kurir | — |
| Dikirim | Diterima | Kurir mengirim status terkirim, atau pasien menekan "Pesanan diterima" | Kurir, pasien | — |
| Dikirim | Gagal kirim | Kurir mengirim status gagal | Kurir | RF-09 |
| Dikirim | Klaim | PB-18 habis tanpa status terkirim atau gagal, dan pasien mengajukan "barang tidak sampai" | Pasien | Klaim RF-16 |
| Diterima | Klaim | Pasien mengajukan klaim dalam PB-04 (rusak, tidak sesuai, kurang, atau tidak sampai) | Pasien | Klaim RF-10 atau RF-16 |
| Diterima | Selesai | PB-04 habis tanpa klaim | Sistem | — |
| Klaim | Klaim disetujui | Klaim disetujui | Layanan Pelanggan, Super Admin | RF-10 atau RF-16 |
| Klaim | Selesai | Klaim ditolak | Layanan Pelanggan, Super Admin | — |

Pesanan yang sudah Dikirim tetap berjalan walaupun outlet dinonaktifkan.

**Refund**

| Dari | Ke | Pemicu |
| --- | --- | --- |
| Diproses | Berhasil | Penyedia mengonfirmasi berhasil |
| Diproses | Gagal | Penyedia mengembalikan galat (AB-32) |
| Diproses | Perlu dicek | PB-19 habis tanpa konfirmasi penyedia (AB-35) |
| Perlu dicek | Berhasil atau Gagal | Status penyedia diketahui |
| Gagal | Diproses | Keuangan memproses ulang |

**Klaim:** Diajukan → Disetujui atau Ditolak (dengan alasan). Klaim aktif adalah klaim berstatus Diajukan. Setelah klaim ditolak, klaim baru untuk pesanan yang sama tidak dapat diajukan `[UP-36]`.

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
| PB-15 | Batas pengajuan klaim layanan terjadwal (RF-06, RF-07, RF-17) | 2 x 24 jam setelah jadwal berakhir | Super Admin | `[UP-53]` |
| PB-16 | Batas pemeriksaan Keuangan atas notifikasi pembayaran yang nominalnya tidak sama (RF-14) | 1 hari kerja | Super Admin | `[UP-52]` |
| PB-17 | Batas outlet menyerahkan barang ke kurir sejak status Diproses outlet (RF-15) | 2 x 24 jam | Super Admin | `[UP-52]` |
| PB-18 | Batas status Dikirim tanpa kabar kurir sebelum pasien dapat mengajukan "barang tidak sampai" (RF-16) | 7 hari | Super Admin | `[UP-52]` |
| PB-19 | Batas refund berstatus Diproses tanpa konfirmasi penyedia sebelum menjadi "Perlu dicek" (AB-35) | 2 x 24 jam | Super Admin | `[UP-55]` |
| PB-20 | Batas Keuangan menetapkan penyebab RF-18 sebelum berlaku penyebab platform | 3 hari kerja | Super Admin | `[UP-56]` |

"Hari kerja" mengikuti kalender hari kerja yang diatur Super Admin. Batas berbasis hari kerja (PB-05, PB-16, PB-20) dihitung dari waktu kejadian dan berakhir pukul 23.59 WIB pada hari kerja ke-N setelah hari kejadian; hari libur tidak dihitung `[UP-37]`.

### 3.8 Zona waktu

| ID | Aturan | Sumber |
| --- | --- | --- |
| AB-40 | Jadwal klinik, lab, dan home visit tampil dalam zona waktu lokasi rekanan, dengan label zona (misalnya "10.00 WITA") | `[USER] T3-Q10` |
| AB-41 | Jadwal konsultasi online tampil dalam zona waktu perangkat masing-masing pengguna | `[USER] T3-Q10` |
| AB-42 | Laporan di portal rekanan memakai zona waktu rekanan; laporan dan backoffice Rumah Sehat memakai WIB | `[USER] T3-Q10` |
| AB-43 | Batas waktu berbasis jam atau hari kalender (PB-01–PB-04, PB-06, PB-07, PB-15, PB-17–PB-19; PB-20 berbasis hari kerja seperti PB-05) dihitung dari waktu absolut, tidak terpengaruh zona waktu tampilan | `[KEPUTUSAN-11]` |

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
| AB-71 | Pasien dapat melihat catatan konsultasi dan resep miliknya dan milik anggota keluarganya. Untuk anggota keluarga dewasa, akses pemilik akun adalah keputusan produk yang dibawa ke gate dan kebolehannya ditanyakan ke legal (L-05 rahasia kedokteran, L-08). Akses pemilik akun ke data klinis anggota keluarga, dewasa maupun anak, dicatat di audit trail (E-05) | `[UP-15]` |
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

- AC1. Given pasien mendaftar, When layar persetujuan tampil, Then layar memuat ketujuh informasi Pasal 21 ayat (1) UU PDP: (a) legalitas pemrosesan, (b) tujuan pemrosesan, (c) jenis dan relevansi data yang diproses termasuk data kesehatan, (d) jangka waktu retensi dokumen yang memuat data pribadi, (e) rincian informasi yang dikumpulkan, (f) jangka waktu pemrosesan, dan (g) hak subjek data, ditambah kontak Rumah Sehat `[SRC-15]`. Teks legalitas menunggu L-08; teks retensi dan jangka waktu pemrosesan menunggu PT-24. Semuanya wajib terisi sebelum go-live.
- AC2. Given pasien menyetujui, When ia melanjutkan, Then bukti persetujuan tersimpan: versi teks, butir yang disetujui, waktu, dan kanal.
- AC3. Given layar mendukung lebih dari satu butir persetujuan, When legal menetapkan dasar pemrosesan per jenis data (L-08), Then butir baru dapat ditambahkan tanpa mengubah aplikasi `[KEPUTUSAN-12]`.
- AC4. Given teks persetujuan akan diperbarui ke versi baru dengan tanggal berlaku, When versi baru dijadwalkan, Then pemberitahuan perubahan beserta tanggal berlakunya dikirim ke pasien sebelum tanggal itu (Pasal 21 ayat (2) `[SRC-15]`). Setelah tanggal berlaku, pasien diminta menyetujui versi baru sebelum dapat memesan layanan.
- AC5. Given pasien tidak menyetujui, When ia mencoba melanjutkan, Then akun tidak dibuat (registrasi) atau layanan tidak dapat dipesan (versi baru).

#### US-105 Profil data diri

Sebagai pasien, saya ingin mengisi dan mengubah data diri, agar data layanan dan pengiriman saya benar.
Rujukan: BR-002 · Audit: E-10 (perubahan data identitas)

- AC1. Given pasien membuka profil, When ia mengisi nama, tanggal lahir, jenis kelamin, nomor HP, email, dan alamat, Then data tersimpan.
- AC2. Given pasien menambah lebih dari satu alamat, When ia menyimpannya, Then alamat tersimpan di daftar alamat dan dapat dipilih saat pengiriman produk atau home visit `[UP-38]`.
- AC3. Given identitas pasien sudah disetujui, When ia mengubah nama atau tanggal lahir (perubahan NIK lewat US-106), Then status identitas kembali ke "Menunggu verifikasi", dan layanan yang mewajibkan identitas tidak dapat dipesan sampai disetujui lagi. Pesanan yang sudah dibayar tetap berlaku `[UP-39]`.
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
- AC6. Given pasien mengisi tanggal lahir anggota keluarga, When formulir tampil, Then alur dewasa (KTP) atau anak (KK) ditentukan sistem dari tanggal lahir, bukan dipilih pasien. Batas usianya `[TIDAK DIKETAHUI]` (L-08) dan diatur sebagai parameter sebelum go-live.

#### US-109 Ubah dan hapus anggota keluarga

Sebagai pasien, saya ingin mengubah atau menghapus anggota keluarga, agar datanya tetap benar.
Rujukan: BR-008, BR-106 · Audit: E-15, E-16

- AC1. Given anggota keluarga sudah disetujui, When pasien mengubah nama, tanggal lahir, NIK, atau dokumen, Then statusnya kembali ke "Menunggu verifikasi" `[UP-39]`.
- AC2. Given anggota keluarga punya pesanan terjadwal yang sudah dibayar dan belum selesai, When pasien menghapusnya, Then penghapusan ditolak sampai pesanan selesai atau dibatalkan.
- AC3. Given tidak ada pesanan aktif, When pasien menghapus anggota keluarga, Then anggota keluarga tidak lagi tampil (soft delete). Riwayat transaksi dan rekam medis yang terkait tetap tersimpan mengikuti ketentuan retensi (PT-24, L-06).
- AC4. Given anggota keluarga dengan NIK yang sama pernah dihapus, When pasien menambahkannya kembali, Then data dibuat sebagai entri baru berstatus "Menunggu verifikasi", dan riwayat layanan entri lama tetap dapat dilihat (US-310 AC3).

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
- AC7. Given NIK sudah disetujui sebagai pemilik akun di akun lain, When Verifikator menyetujui pengajuan pemilik akun dengan NIK yang sama, Then persetujuan ditolak dengan alasan NIK sudah dipakai `[UP-40]`.

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
- AC3. Given akun dinonaktifkan karena SIP berakhir, When penonaktifan terjadi, Then sesi konsultasi terbayar di masa depan berstatus "Dibatalkan rekanan" dengan refund RF-05 otomatis, home visit yang ditugaskan kepadanya kembali ke "Terkonfirmasi" untuk ditugaskan ulang oleh klinik (§3.6), dan pasien serta klinik diberi tahu.
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
- AC2. Given tenaga kesehatan atau rekanan punya pesanan terbayar yang belum selesai, When akunnya dinonaktifkan, Then: sesi konsultasi dan pendaftaran/home visit rekanan itu yang belum mulai menjadi "Dibatalkan rekanan" (RF-05); home visit yang ditugaskan ke tenaga kesehatan itu kembali ke "Terkonfirmasi" untuk ditugaskan ulang; pesanan produk outlet itu yang belum Dikirim menjadi "Ditolak outlet" (RF-08), sedangkan yang sudah Dikirim tetap berjalan.
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
- AC3. Given tarif diubah, When pembayaran dibuat setelah perubahan, Then harga baru berlaku. Pesanan yang pembayarannya sudah dibuat sebelum perubahan tidak berubah (AB-17).
- AC4. Given admin klinik mencoba mengatur tenaga kesehatan klinik lain, When ia mengirim permintaan, Then ditolak (KU-03).
- AC5. Given pasien melihat harga, When harga tampil, Then nilainya = tarif + markup rekanan itu (AB-10).

#### US-302 Jadwal praktik online

Sebagai tenaga kesehatan, saya ingin mengatur jadwal praktik online saya, agar pasien dapat memesan di waktu yang saya sediakan.
Rujukan: BR-017, AB-52, AB-55 `[USER] T3-Q4` · Audit: E-14

- AC1. Given tenaga kesehatan berstatus aktif, When ia membuat jadwal mingguan berulang (hari, jam mulai, jam selesai) dan pengecualian tanggal, Then slot terbentuk sesuai durasi per jenis (US-301) dalam zona waktu perangkatnya (AB-41).
- AC2. Given jadwal baru tumpang tindih dengan jadwal yang ada, When disimpan, Then ditolak (AB-52).
- AC3. Given slot sudah dipesan dan dibayar, When tenaga kesehatan atau admin klinik menghapus jadwal yang memuat slot itu, Then penghapusan ditolak. Sesi itu harus dibatalkan lebih dulu (US-307 AC4).
- AC4. Given admin klinik membuka jadwal tenaga kesehatan kliniknya di portal, When ia mengubahnya, Then aturan AC1–AC3 berlaku sama, dan perubahan tercatat atas nama admin klinik.
- AC5. Given perubahan jam, durasi, atau penghapusan memengaruhi slot yang sudah dipesan dan dibayar, When disimpan, Then perubahan ditolak sampai sesi itu dibatalkan (RF-05).

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

- AC1. Given sesi chat "Terkonfirmasi", When waktu mulai tiba, Then ruang chat terbuka untuk pasien dan tenaga kesehatan sesi itu saja. Waktu tiap pihak bergabung (§3.5) dicatat server, dan status menjadi "Berlangsung" saat pihak pertama bergabung.
- AC2. Given ruang chat terbuka, When salah satu pihak mengirim teks atau foto (KEPUTUSAN-08), Then pesan sampai ke penerima yang online dalam p95 2 detik dan tersimpan (BR-016).
- AC3. Given koneksi pengirim terputus, When ia mengirim pesan, Then pesan berstatus "belum terkirim" dan terkirim otomatis saat koneksi pulih selama sesi masih berjalan.
- AC4. Given durasi sesi habis, When waktu berakhir, Then ruang chat menjadi baca-saja dan sesi tidak diperpanjang karena gangguan jaringan. Status menjadi "Selesai" hanya bila kedua pihak pernah bergabung; selain itu berlaku US-307.
- AC5. Given pengguna lain (termasuk admin Rumah Sehat), When mencoba membuka isi chat, Then ditolak (§3.1, KU-03).

#### US-306 Sesi video

Sebagai pasien dan tenaga kesehatan, kami ingin melakukan panggilan video selama sesi, agar konsultasi dapat dilakukan tatap muka.
Rujukan: BR-010, BR-016, AB-74 `[UP-29]` · Audit: E-09

- AC1. Given sesi video "Terkonfirmasi" dan waktu mulai tiba, When pihak membuka ruang sesi, Then ia tercatat bergabung (§3.5), status menjadi "Berlangsung" saat pihak pertama bergabung, dan salah satu pihak dapat memulai panggilan yang diterima pihak lain sebagai panggilan masuk.
- AC2. Given panggilan terputus, When salah satu pihak menyambung ulang sebelum durasi habis, Then panggilan tersambung kembali tanpa menambah durasi.
- AC3. Given durasi habis, When waktu berakhir, Then panggilan diakhiri. Status menjadi "Selesai" hanya bila kedua pihak pernah bergabung; selain itu berlaku US-307.
- AC4. Given sesi berlangsung, When diperiksa, Then aplikasi tidak menyediakan fitur rekam, dan fitur rekam di layanan video pihak ketiga dinonaktifkan (AB-74). Cara memverifikasi pengaturan penyedia ditetapkan di Tahap 4.

#### US-307 Ketidakhadiran dan pembatalan oleh tenaga kesehatan

Sebagai pasien, saya ingin ketidakhadiran ditangani otomatis sesuai aturan, agar saya mendapat refund bila tenaga kesehatan tidak hadir.
Rujukan: BR-045, RF-03, RF-04, RF-05, PB-02, PB-03 `[USER] T3-Q6, T3-Q8` `[UP-29]` · Audit: E-08, E-09

- AC1. Given tenaga kesehatan belum bergabung saat batas ketidakhadiran tenaga kesehatan habis (PB-03 sejak jadwal mulai, atau akhir durasi bila lebih dulu), When batas habis, Then status menjadi "Tenaga kesehatan tidak hadir", refund RF-04 berjalan otomatis, dan pasien diberi tahu.
- AC2. Given tenaga kesehatan sudah bergabung dan pasien belum bergabung saat batas ketidakhadiran pasien habis (PB-02 sejak jadwal mulai, atau akhir durasi bila lebih dulu), When batas habis, Then status menjadi "Pasien tidak hadir" tanpa refund (RF-03), dan pasien diberi tahu.
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
Rujukan: BR-016, BR-106, BR-008, AB-71 `[UP-15]` · Audit: E-05 (pemilik akun membuka data anggota keluarga, dewasa maupun anak), E-12 (unduh)

- AC1. Given pasien membuka riwayat, When ia memilih dirinya atau anggota keluarga, Then daftar sesi tampil dengan tanggal, tenaga kesehatan, jenis, dan status.
- AC2. Given sesi selesai, When pasien membukanya, Then ia melihat chat (baca-saja), catatan final, dan resep.
- AC3. Given anggota keluarga sudah dihapus, When pemilik akun membuka riwayat, Then riwayat sesi anggota keluarga itu tetap dapat dilihat selama datanya disimpan (PT-24).

#### US-311 Daftar sesi tenaga kesehatan

Sebagai tenaga kesehatan, saya ingin melihat sesi hari ini dan yang akan datang, agar siap melayani tepat waktu.
Rujukan: BR-013 · Audit: E-05 (membuka detail sesi berisi keluhan awal)

- AC1. Given tenaga kesehatan masuk, When ia membuka daftar sesi, Then tampil sesi terkonfirmasi dengan waktu (zona waktu perangkat), jenis, dan nama pasien.
- AC2. Given ada pesanan baru atau waktu pengingat PB-08 tiba, When kejadian terjadi, Then tenaga kesehatan menerima notifikasi push (US-1003).
- AC3. Given waktu mulai sesi belum tiba, When ia menekan tombol bergabung, Then tombol belum aktif.
- AC4. Given tenaga kesehatan membuka detail sesi yang ditanganinya, When detail tampil, Then keluhan awal pasien tampil dan akses tercatat (E-05).

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
- AC3. Given pengiriman ke sistem rekanan gagal, When percobaan ulang otomatis juga gagal sampai batas waktu integrasi (Tahap 4), Then pendaftaran ditandai untuk tindak lanjut Layanan Pelanggan. Kalau rekanan menyatakan tidak dapat melayani, status menjadi "Dibatalkan rekanan" (RF-05). Kalau sampai jadwal mulai pesanan belum diterima sistem rekanan, status menjadi "Dibatalkan sistem" dan refund RF-18 berjalan otomatis.

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
- AC3. Given pesanan sudah diterima sistem rekanan dan tidak ada check-in sampai jadwal berakhir, When batas terlewati, Then status menjadi "Pasien tidak hadir" tanpa refund (RF-03). Pesanan yang belum diterima sistem rekanan mengikuti US-402 AC3 (RF-18).
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
- AC3. Given pesanan berstatus "Ditugaskan", When klinik mengirim penugasan ke tenaga kesehatan lain, Then penugasan diganti, tenaga kesehatan lama dan baru serta pasien diberi tahu.

#### US-503 Status kunjungan oleh tenaga kesehatan

Sebagai tenaga kesehatan, saya ingin melihat tugas home visit dan memperbarui statusnya, agar pasien dan klinik tahu perkembangan kunjungan.
Rujukan: BR-022, RF-03, RF-17 `[UP-13]` · Audit: E-09. Alamat dan nomor HP bukan data sensitif menurut §3.1, sehingga tidak dicatat sebagai E-05; aksesnya dibatasi waktu (AC1)

- AC1. Given tenaga kesehatan punya tugas, When ia membuka aplikasi, Then tampil jadwal, alamat (dengan tautan peta), nama pasien, layanan, dan nomor HP pasien. Nomor HP hanya tampil di hari kunjungan sampai kunjungan selesai.
- AC2. Given tugas "Ditugaskan", When tenaga kesehatan menandai "Berangkat", lalu "Tiba", lalu "Selesai", Then setiap status tersimpan berurutan dan pasien diberi tahu saat "Berangkat" dan "Tiba". Urutan tidak boleh dilompati.
- AC3. Given status "Tiba", When tenaga kesehatan menandai "Pasien tidak ada di lokasi" dengan keterangan, Then status menjadi "Pasien tidak hadir" tanpa refund (RF-03), dan pasien dapat mengajukan keberatan paling lambat PB-15 setelah jadwal berakhir sebagai klaim RF-17 (US-807). Penandaan dari status selain "Tiba" ditolak.

#### US-504 Pantau kunjungan dan laporkan tenaga kesehatan tidak datang

Sebagai pasien, saya ingin memantau status kunjungan dan melaporkan bila tenaga kesehatan tidak datang, agar mendapat refund.
Rujukan: BR-022, BR-045, RF-06, PB-03, PB-05 `[UP-13]` · Audit: E-08, E-09

- AC1. Given pesanan home visit aktif, When pasien membukanya, Then tampil linimasa status dan nama tenaga kesehatan yang ditugaskan.
- AC2. Given status "Terkonfirmasi", "Ditugaskan", atau "Berangkat" saat PB-03 habis sejak jadwal mulai, When pasien membuka pesanan sebelum PB-15 habis setelah jadwal berakhir, Then tombol "Laporkan tenaga kesehatan tidak datang" aktif, dan laporan menjadi klaim RF-06 (US-807).
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
- AC2. Given outlet menolak pesanan, tidak menerimanya sampai PB-06, atau membatalkannya sebelum diserahkan ke kurir, When kondisi terjadi, Then status menjadi "Ditolak outlet", refund RF-08 berjalan otomatis, dan pasien diberi tahu.
- AC3. Given barang diserahkan ke kurir, When nomor resi dan status kurir diterima, Then status menjadi "Dikirim" dan pasien dapat melihat riwayat pelacakan.
- AC4. Given kurir mengirim status gagal kirim, When diterima, Then status menjadi "Gagal kirim", refund RF-09 berjalan otomatis, dan pasien diberi tahu.
- AC5. Given status "Diproses outlet" dan PB-17 habis tanpa barang diserahkan ke kurir, When batas habis, Then status menjadi "Tidak diserahkan", refund RF-15 berjalan otomatis, dan pasien diberi tahu.
- AC6. Given status "Dikirim" dan PB-18 habis tanpa status terkirim atau gagal dari kurir, When pasien membuka pesanan, Then tombol "Barang tidak sampai" aktif dan pengajuannya menjadi klaim RF-16 tanpa wajib foto (US-807).

#### US-705 Konfirmasi terima dan klaim barang

Sebagai pasien, saya ingin mengonfirmasi barang diterima atau mengajukan klaim bila rusak atau tidak sesuai, agar masalah pesanan ditangani.
Rujukan: BR-045, RF-10, PB-04 `[USER] T3-Q8` · Audit: E-08, E-09

- AC1. Given kurir mengirim status terkirim atau pasien menekan "Pesanan diterima", When kejadian pertama terjadi, Then status menjadi "Diterima".
- AC2. Given status "Diterima" dan PB-04 belum habis, When pasien mengajukan klaim dengan alasan rusak, tidak sesuai, atau kurang (minimal satu foto), atau tidak sampai (foto tidak wajib), Then klaim tercatat (US-807) dan status menjadi "Klaim".
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
- AC2. Given markup berbentuk persen, When harga dihitung, Then markup dihitung dan dibulatkan per unit ke Rupiah penuh terdekat (setengah ke atas), lalu dikalikan jumlah. Given markup nominal, Then markup berlaku per unit (AB-11a) `[UP-43]` `[UP-51]`.
- AC3. Given pesanan layanan terjadwal, When ringkasan tampil, Then ringkasan aturan pembatalan (PB-01, no-show) tampil. Untuk klinik, lab, dan home visit, pernyataan AB-16 juga tampil.
- AC4. Given pesanan produk, When ringkasan tampil, Then pernyataan bahwa pesanan tidak dapat dibatalkan setelah dibayar tampil (RF-11).
- AC5. Given markup atau tarif berubah setelah ringkasan tampil dan sebelum pembayaran dibuat, When pasien menekan bayar, Then total baru tampil dan pasien harus menyetujuinya sebelum pembayaran dibuat (AB-15, AB-17).

#### US-802 Pembayaran lewat payment gateway

Sebagai pasien, saya ingin membayar dengan metode yang tersedia, agar pesanan saya terkonfirmasi.
Rujukan: BR-040, BR-115, AB-20, AB-25, PB-07, RF-12, RF-13, RF-14 `[UP-34]` `[UP-52]` · Audit: E-08

- AC1. Given pasien memilih metode dan menekan bayar, When pembayaran dibuat di payment gateway, Then nominalnya sama dengan total di ringkasan, dan pasien diarahkan ke instruksi atau halaman pembayaran.
- AC2. Given notifikasi pembayaran berhasil diterima dan terverifikasi berasal dari payment gateway, When diproses, Then pesanan dikonfirmasi satu kali saja walaupun notifikasi diterima berulang (KU-05).
- AC3. Given nominal di notifikasi tidak sama dengan nominal pesanan, When diproses, Then pesanan tidak dikonfirmasi dan menjadi "Kedaluwarsa", lalu masuk antrean Keuangan. Keuangan memeriksa dan membuat refund RF-14 sebesar seluruh nominal yang diterima. Bila PB-16 habis tanpa tindakan Keuangan, refund RF-14 dibuat otomatis dan Super Admin diberi tahu.
- AC4. Given pembayaran gagal atau pasien berganti metode, When pasien kembali ke aplikasi, Then ia dapat mencoba metode lain selama PB-07 belum habis, dan instruksi pembayaran sebelumnya dibatalkan di payment gateway bila penyedia mendukungnya.
- AC5. Given notifikasi berhasil datang setelah pesanan kedaluwarsa, When slot atau stok masih tersedia, Then pesanan dikonfirmasi. Kalau tidak tersedia, refund RF-13 berjalan otomatis `[UP-34]`.
- AC6. Given pesanan sudah dibayar, When notifikasi berhasil untuk pembayaran lain atas pesanan yang sama diterima, Then refund RF-12 dibuat langsung tanpa menunggu rekonsiliasi.

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

- AC1. Given layanan berstatus "Terkonfirmasi" (atau "Ditugaskan" untuk home visit) dan sisa waktu sebelum jadwal minimal PB-01, When pasien menekan batal, Then pasien melihat nominal refund (harga layanan; biaya transaksi non-QRIS tidak kembali) dan harus mengonfirmasi.
- AC2. Given pasien mengonfirmasi, When pembatalan disimpan, Then status menjadi "Dibatalkan pasien", slot dilepas, rekanan (dan tenaga kesehatan yang sudah ditugaskan untuk home visit) diberi tahu, dan refund RF-01 berjalan otomatis.
- AC3. Given sisa waktu kurang dari PB-01, When pasien menekan batal, Then pasien diberi tahu tidak ada refund (RF-02). Kalau tetap dibatalkan, status menjadi "Dibatalkan pasien" tanpa refund dan slot dilepas.
- AC4. Given konsultasi "Berlangsung", klinik/lab "Check-in", home visit "Berangkat" atau sesudahnya, atau status akhir, When pasien mencari tombol batal, Then tombol tidak tersedia.

#### US-806 Refund otomatis

Sebagai pasien, saya ingin refund berjalan otomatis untuk kejadian yang tercatat sistem, agar tidak perlu mengajukan klaim.
Rujukan: BR-045, RF-01, RF-04, RF-05, RF-08, RF-09, RF-12, RF-13, RF-15, RF-18, AB-34 `[USER] T3-Q7, T3-Q8` · Audit: E-08

- AC1. Given salah satu kejadian RF-01, RF-04, RF-05, RF-08, RF-09, RF-12, RF-13, RF-15, atau RF-18 terjadi, When sistem mencatatnya, Then refund ke pasien dibuat dengan nominal sesuai matriks §3.5 tanpa persetujuan manual, dan pasien diberi tahu. Kewajiban per pihak (AB-34) langsung tercatat, kecuali RF-18 yang berstatus "Menunggu penetapan penyebab" sampai ditetapkan Keuangan atau PB-20 habis.
- AC2. Given refund untuk satu pesanan sudah dibuat, When kejadian lain atau kejadian yang sama tercatat untuk pesanan itu, Then refund kedua tidak dibuat, kecuali RF-12 (AB-36, KU-05).
- AC3. Given refund RF-01, When nominal dihitung, Then pasien menerima harga layanan tanpa biaya transaksi non-QRIS, kewajiban rekanan = harga rekanan penuh, dan kewajiban Rumah Sehat = markup. Given refund kelompok penyebab rekanan atau penyebab platform, Then pasien menerima seluruh pembayaran dan kewajiban per pihak mengikuti AB-34.

#### US-807 Klaim refund

Sebagai pasien, saya ingin mengajukan klaim untuk kejadian yang perlu diperiksa, agar mendapat refund bila klaim saya benar.
Rujukan: BR-045, RF-06, RF-07, RF-10, RF-16, RF-17, PB-04, PB-05, PB-15 `[USER] T3-Q8` `[UP-36]` `[UP-52]` `[UP-53]` · Audit: E-05 (membuka bukti), E-08

- AC1. Given kejadian RF-06, RF-07, RF-10, RF-16, atau RF-17 dan masih dalam batas pengajuannya (PB-15 setelah jadwal berakhir untuk RF-06, RF-07 layanan terjadwal, dan RF-17; PB-04 setelah status Diterima untuk RF-07 produk, RF-10, dan RF-16; PB-04 setelah PB-18 habis untuk RF-16 jalur Dikirim), When pasien mengirim klaim dengan alasan dan bukti (foto wajib untuk RF-10), Then klaim berstatus "Diajukan" dan pasien melihat batas keputusan PB-05. Di luar batas pengajuan, tombol klaim tidak tersedia.
- AC2. Given pesanan sudah punya klaim aktif, When pasien mengajukan klaim lagi, Then ditolak (KU-05).
- AC3. Given Layanan Pelanggan (atau Super Admin untuk klaim tereskalasi) menyetujui klaim, When keputusan disimpan, Then refund dibuat dengan nominal sesuai matriks dan kewajiban per pihak sesuai AB-34, dan pasien diberi tahu.
- AC4. Given Layanan Pelanggan atau Super Admin menolak klaim, When alasan diisi, Then pasien diberi tahu alasannya, dan klaim baru untuk pesanan itu tidak dapat diajukan. Penolakan tanpa alasan tidak dapat disimpan.
- AC5. Given PB-05 habis tanpa keputusan, When batas terlewati, Then klaim ditandai terlambat dan dieskalasi ke Super Admin, tanpa disetujui atau ditolak otomatis (AB-33).

#### US-808 Eksekusi dan status refund

Sebagai Keuangan, saya ingin refund dieksekusi dan dipantau sampai berhasil, agar dana pasien kembali dan pencatatan lengkap.
Rujukan: BR-045, AB-30–AB-35, PB-19, ADR-004 `[UP-35]` `[UP-55]` · Audit: E-08

- AC1. Given refund dibuat, When dieksekusi lewat payment gateway, Then status "Diproses" lalu "Berhasil" setelah dikonfirmasi penyedia, dan entri pembalik untuk kewajiban rekanan dan Rumah Sehat (AB-34) tercatat (AB-31).
- AC2. Given eksekusi gagal, When penyedia mengembalikan galat, Then status menjadi "Gagal", refund masuk antrean Keuangan, pasien diberi tahu bahwa refund sedang ditangani, dan Keuangan dapat memproses ulang.
- AC3. Given refund menimbulkan kewajiban rekanan (AB-34), When kewajiban tercatat (saat refund dibuat, atau untuk RF-18 saat penyebab ditetapkan), Then nominalnya tampil di laporan rekanan (US-1201) dan laporan Keuangan berstatus "Terbuka". Kewajiban RF-18 yang belum ditetapkan tampil di laporan Keuangan sebagai "Menunggu penetapan penyebab" dan tidak tampil di laporan rekanan. Mekanisme penarikan dana ditetapkan di Tahap 4–5 (PT-27) `[UP-35]` `[UP-56]`.
- AC4. Given refund berstatus "Diproses" lebih dari PB-19 tanpa konfirmasi penyedia, When batas habis, Then status menjadi "Perlu dicek", refund masuk antrean Keuangan, dan refund tidak dapat dieksekusi ulang sebelum status penyedia diketahui (AB-35).
- AC5. Given kewajiban rekanan sudah dilunasi, When Keuangan menandainya "Selesai" dengan rujukan bukti, Then status kewajiban berubah di kedua laporan dan tercatat (E-08). Penandaan tanpa rujukan bukti ditolak.
- AC6. Given refund RF-18 berstatus "Menunggu penetapan penyebab", When Keuangan menetapkan penyebab rekanan atau platform dengan rujukan log integrasi dalam PB-20, Then kewajiban per pihak dihitung sesuai AB-34 dan tercatat (E-08). Given PB-20 habis tanpa penetapan, Then berlaku penyebab platform dan Super Admin diberi tahu.

#### US-809 Pengaturan markup per rekanan

Sebagai Keuangan, saya ingin mengatur markup per rekanan, agar harga pasien sesuai keputusan CEO.
Rujukan: BR-042, BR-063, PB-11 `[UP-44]` · Audit: E-14

- AC1. Given Keuangan memilih rekanan, When ia mengisi jenis (persen atau nominal), nilai (persen 0–100; nominal Rupiah 0 atau lebih), dan waktu berlaku (segera atau tanggal mendatang), Then pengaturan tersimpan dan riwayat perubahan tetap ada.
- AC2. Given markup diubah dengan waktu berlaku tertentu, When pembayaran suatu pesanan dibuat sebelum waktu berlaku, Then harga pesanan itu memakai markup lama walaupun pembayarannya diselesaikan sesudahnya (AB-17).
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

- AC1. Given Super Admin mengubah parameter PB-01–PB-10 atau PB-14–PB-20, When nilainya valid (angka positif dengan satuan yang ditentukan; untuk PB-14 daftar dokumen tidak kosong), Then perubahan tersimpan dengan riwayat.
- AC2. Given parameter diubah, When pesanan yang pembayarannya dibuat sebelum perubahan dievaluasi (misalnya batas pembatalan), Then nilai yang berlaku saat pembayaran dibuat yang dipakai (AB-17) `[UP-32]`.
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
Rujukan: BR-025, BR-050, BR-008, AB-71 `[UP-15]` · Audit: E-05 (pemilik akun membuka data anggota keluarga, dewasa maupun anak), E-12 (unduh)

- AC1. Given hasil tersedia, When pasien membukanya, Then hasil tampil di aplikasi dan dapat diunduh sebagai PDF.
- AC2. Given pengguna lain, When mencoba membuka hasil lab pasien, Then ditolak (§3.1, KU-03).

### EP-10 Notifikasi

| Story | Fitur | BR | Prioritas |
| --- | --- | --- | --- |
| US-1001 | Notifikasi pasien per kejadian dan kanal | BR-050 | Must |
| US-1002 | Pengingat jadwal | BR-050 | Must |
| US-1003 | Notifikasi tenaga kesehatan dan admin | BR-013, BR-102 | Must |

Matriks kanal notifikasi pasien `[UP-23]`. Tidak ada SMS (BR-050).

| Kejadian | Push | Email | WhatsApp |
| --- | --- | --- | --- |
| Verifikasi identitas diputuskan (disetujui atau ditolak) | Ya | Ya | Tidak |
| Pembayaran berhasil dan bukti bayar | Ya | Ya | Ya |
| Pengingat jadwal (PB-08) | Ya | Tidak | Ya |
| Pesanan dibatalkan rekanan, tenaga kesehatan tidak hadir, refund dibuat | Ya | Ya | Ya |
| Perubahan status lain (ditugaskan, berangkat, tiba, dikirim, diterima) | Ya | Tidak | Tidak |
| Pesan chat baru saat aplikasi tidak dibuka | Ya | Tidak | Tidak |
| Antrean mendekati giliran (PB-10) | Ya | Tidak | Tidak |
| Hasil lab tersedia atau dikoreksi | Ya | Ya | Ya |
| Klaim diputuskan, refund berhasil atau gagal | Ya | Ya | Tidak |
| Permintaan hak subjek data ditanggapi | Tidak | Ya | Tidak |

#### US-1001 Notifikasi pasien per kejadian dan kanal

Sebagai pasien, saya ingin diberi tahu setiap kejadian penting lewat kanal yang sesuai, agar tidak melewatkan layanan.
Rujukan: BR-050, AB-73, NFR-PERF-05 `[UP-23]` · Audit: tidak ada aksi sensitif

- AC1. Given kejadian di matriks terjadi, When diproses, Then notifikasi diserahkan ke penyedia kanal sesuai matriks paling lambat 1 menit setelah kejadian.
- AC2. Given notifikasi dibuat, When isinya diperiksa, Then tidak memuat diagnosis, nama obat, nilai hasil lab, atau isi chat (AB-73).
- AC3. Given penyedia kanal menolak atau gagal mengirim, When kegagalan terjadi, Then pengiriman dicoba ulang dan kegagalan akhirnya tercatat. Kegagalan satu kanal tidak menghentikan kanal lain.
- AC4. Given pasien tidak mengizinkan notifikasi push, When kejadian terjadi, Then kanal email dan WhatsApp tetap dikirim sesuai matriks.
- AC5. Syarat persetujuan penerima dan template pesan WhatsApp mengikuti penyedia terpilih (Tahap 4–5) `[TIDAK DIKETAHUI]`.

#### US-1002 Pengingat jadwal

Sebagai pasien, saya ingin diingatkan sebelum jadwal layanan, agar tidak terlambat atau lupa.
Rujukan: BR-050, PB-08 `[UP-05]` · Audit: tidak ada aksi sensitif

- AC1. Given layanan terjadwal "Terkonfirmasi", When waktu PB-08 tiba, Then pengingat dikirim ke pasien sesuai matriks.
- AC2. Given layanan dipesan setelah waktu pengingat pertama lewat, When pemesanan dikonfirmasi, Then hanya pengingat yang belum lewat yang dikirim.
- AC3. Given layanan dibatalkan, When waktu pengingat tiba, Then pengingat tidak dikirim.

#### US-1003 Notifikasi tenaga kesehatan dan admin

Sebagai tenaga kesehatan dan admin, saya ingin diberi tahu kejadian yang menjadi tugas saya, agar dapat menindaklanjutinya.
Rujukan: BR-013, BR-102, BR-062 `[UP-23]` · Audit: tidak ada aksi sensitif

- AC1. Given ada pesanan konsultasi baru, pembatalan, atau tugas home visit, When kejadian terjadi, Then tenaga kesehatan menerima push dan email.
- AC2. Given waktu pengingat PB-08 tiba, When tenaga kesehatan punya sesi atau kunjungan, Then ia menerima push.
- AC3. Given SIP akan berakhir (PB-09) atau ada keluhan baru terkait rekanan, When kejadian terjadi, Then tenaga kesehatan atau admin rekanan terkait menerima email.

### EP-11 Konten dan layanan pelanggan

| Story | Fitur | BR | Prioritas |
| --- | --- | --- | --- |
| US-1101 | Pengelolaan konten (CMS) | BR-061 | Must |
| US-1102 | Konten di aplikasi pasien | BR-061 | Must |
| US-1103 | Ajukan keluhan | BR-062 | Must |
| US-1104 | Kelola keluhan | BR-062, BR-064, BR-065 | Must |

#### US-1101 Pengelolaan konten (CMS)

Sebagai admin Konten, saya ingin membuat dan menayangkan banner, halaman info, artikel edukasi kesehatan, dan promo, agar pasien mendapat informasi.
Rujukan: BR-061 · Audit: E-16 (hapus dan pulihkan)

- AC1. Given admin Konten membuat banner, halaman info, artikel (judul, isi, gambar, kategori), atau promo, When ia menyimpan, Then konten berstatus "Draf" dan dapat dipratinjau.
- AC2. Given konten dijadwalkan, When waktu tayang tiba, Then status menjadi "Tayang". When waktu selesai tiba atau konten diarsipkan, Then konten tidak tampil lagi.
- AC3. Given admin menghapus konten, When disimpan, Then konten di-soft delete dan dapat dipulihkan (ADR-003).
- AC4. Promo hanya berupa informasi. Potongan harga dan voucher tidak didukung di rilis 1 (§2.2), sehingga promo tidak boleh mengubah harga di aplikasi.

#### US-1102 Konten di aplikasi pasien

Sebagai pengguna, saya ingin melihat banner, artikel, dan halaman info, agar mendapat informasi kesehatan dan layanan.
Rujukan: BR-061 `[UP-21]` · Audit: tidak ada aksi sensitif

- AC1. Given konten berstatus "Tayang", When pengguna (boleh belum masuk) membuka beranda atau daftar artikel, Then konten tampil urut sesuai jadwal tayang.
- AC2. Given banner menautkan ke layanan atau artikel di aplikasi, When pengguna mengetuknya, Then halaman tujuan terbuka. Kalau tujuan tidak lagi tersedia, pesan "konten tidak tersedia" tampil.

#### US-1103 Ajukan keluhan

Sebagai pasien, saya ingin mengajukan keluhan dan memantau penanganannya, agar masalah saya diselesaikan.
Rujukan: BR-062 `[UP-30]` · Audit: tidak ada aksi sensitif

- AC1. Given pasien mengisi kategori (layanan, pembayaran, akun, lainnya), pesanan terkait (opsional), deskripsi, dan lampiran foto (opsional), When ia mengirim, Then tiket berstatus "Baru" dan nomor tiket tampil.
- AC2. Given admin membalas, When balasan tersimpan, Then pasien diberi tahu dan dapat membalas di tiket yang sama.
- AC3. Klaim refund diajukan lewat US-807, dan permintaan hak subjek data lewat US-1602. Bila pasien memilih kategori yang sesuai, aplikasi mengarahkan ke alur itu.

#### US-1104 Kelola keluhan

Sebagai Layanan Pelanggan, saya ingin mengelola tiket keluhan, agar setiap keluhan ditanggapi.
Rujukan: BR-062, BR-064, BR-065 `[UP-30]` · Audit: tidak ada aksi sensitif

- AC1. Given tiket baru, When Layanan Pelanggan membuka antrean, Then tiket tampil urut dari yang terlama dan dapat diubah statusnya: Baru, Diproses, Menunggu pasien, Selesai.
- AC2. Given tiket terkait pesanan rekanan, When Layanan Pelanggan meneruskannya ke rekanan, Then admin rekanan itu melihat tiket di portal (US-1203) tanpa data klinis pasien.
- AC3. Given tiket ditutup, When pasien membalas dalam 7 hari, Then tiket dibuka kembali `[UP-30]`.

### EP-12 Portal rekanan

Portal rekanan hanya untuk pelaporan dan rekap, ditambah menu pengaturan konsultasi online untuk klinik (US-301, US-302 AC4) `[USER] T3-Q14, T3-Q16`.

| Story | Fitur | BR | Prioritas |
| --- | --- | --- | --- |
| US-1201 | Laporan transaksi dan pembagian dana rekanan | BR-046, BR-064, BR-065 | Must |
| US-1202 | Profil rekanan | BR-064, BR-065 | Must |
| US-1203 | Keluhan terkait rekanan | BR-064, BR-065 | Must |

#### US-1201 Laporan transaksi dan pembagian dana rekanan

Sebagai admin klinik, lab, atau outlet, saya ingin melihat laporan transaksi dan pembagian dana rekanan saya, agar dapat mencocokkannya dengan pembukuan.
Rujukan: BR-046, BR-064, BR-065, AB-42, AB-65 `[UP-45]` · Audit: E-12 (export)

- AC1. Given admin rekanan memilih periode, jenis layanan, dan status, When laporan tampil, Then setiap transaksi memuat tanggal (zona waktu rekanan), nomor, layanan atau produk, nama penerima layanan, harga rekanan, MDR, bagian rekanan, refund, kewajiban pengembalian (US-808 AC3), dan tagihan biaya rekanan (US-810), beserta rekap harian dan bulanan.
- AC2. Given laporan tampil, When diperiksa, Then markup Rumah Sehat dan total bayar pasien tidak ditampilkan `[UP-45]`, dan tidak ada data klinis.
- AC3. Given klinik juga outlet (AB-65), When admin membuka laporan, Then transaksi layanan klinik dan penjualan produk dapat dilihat terpisah atau gabungan.
- AC4. Given admin mengekspor laporan, When file dibuat, Then file CSV hanya berisi data rekanannya dan event export tercatat.

#### US-1202 Profil rekanan

Sebagai admin rekanan, saya ingin melihat profil rekanan dan mengajukan perubahan, agar data rekanan tetap benar.
Rujukan: BR-064, BR-065 `[UP-31]` · Audit: E-10

- AC1. Given admin rekanan membuka profil, When tampil, Then memuat nama, jenis, alamat, zona waktu, kontak, rekening (nomor termasking), sumber sistem, dan status integrasi.
- AC2. Given admin rekanan mengajukan perubahan, When dikirim, Then perubahan tidak langsung berlaku, dan menjadi pengajuan yang diproses admin Rumah Sehat.
- AC3. Given pengajuan menyangkut rekening penerima, When diproses, Then perubahan hanya berlaku setelah disetujui Verifikator.

#### US-1203 Keluhan terkait rekanan

Sebagai admin rekanan, saya ingin melihat dan menanggapi keluhan pasien terkait rekanan saya, agar keluhan selesai lebih cepat.
Rujukan: BR-064, BR-065 `[UP-30]` · Audit: tidak ada aksi sensitif

- AC1. Given Layanan Pelanggan meneruskan tiket (US-1104 AC2), When admin rekanan membuka menu keluhan, Then tiket tampil dan dapat ditanggapi dengan teks.
- AC2. Given tiket milik rekanan lain, When admin rekanan mencoba membukanya, Then ditolak (KU-03).

### EP-13 Aplikasi RME minimalis (T1)

Aplikasi web terpisah dari aplikasi dan portal Rumah Sehat, untuk klinik dan lab rekanan yang belum punya sistem. Aplikasi ini terhubung ke Rumah Sehat lewat kontrak integrasi yang sama dengan sistem rekanan lain (EP-14), dan hanya mencatat pasien yang datang lewat Rumah Sehat `[USER] T3-Q1b, T3-Q1c, T3-Q14, T3-Q15`.

| Story | Fitur | BR | Prioritas |
| --- | --- | --- | --- |
| US-1301 | Onboarding fasyankes ke RME minimalis | BR-120 | Must |
| US-1302 | Akun dan peran pengguna RME | BR-120 | Must |
| US-1303 | Jadwal, layanan, dan harga | BR-122 | Must |
| US-1304 | Appointment dan check-in | BR-125, BR-024 | Must |
| US-1305 | Antrean | BR-125, BR-027 | Should |
| US-1306 | Catatan rekam medis kunjungan | BR-123 | Must |
| US-1307 | Hasil lab | BR-124, BR-025 | Must |
| US-1308 | Home visit | BR-125, BR-022 | Must |
| US-1309 | Pengiriman data rekam medis ke SATUSEHAT | BR-123 | Must |
| US-1310 | Riwayat rekam medis pasien di fasyankes | BR-123, BR-126 | Must |

#### US-1301 Onboarding fasyankes ke RME minimalis

Sebagai Operator RME, saya ingin mendaftarkan klinik atau lab rekanan ke RME minimalis, agar mereka dapat melayani pasien Rumah Sehat tanpa sistem sendiri.
Rujukan: BR-120, BR-121 `[UP-26]` · Audit: E-10

- AC1. Given rekanan klinik atau lab sudah terverifikasi di Rumah Sehat dengan sumber sistem "RME minimalis" (US-205) dan perjanjian penggunaan RME minimalis serta pemrosesan data dengan fasyankes itu sudah tercatat `[UP-49]`, When Operator RME mendaftarkannya beserta kode registrasi fasyankesnya (rincian Tahap 4) dan membuat akun Admin Fasyankes pertama, Then fasyankes aktif di RME minimalis dan koneksi integrasinya ke Rumah Sehat siap diuji.
- AC2. Given rekanan belum terverifikasi atau sumber sistemnya "sistem sendiri", When Operator RME mencoba mendaftarkannya, Then ditolak.
- AC3. Given Operator RME, When ia mencoba membuka data pasien atau rekam medis fasyankes, Then ditolak (§3.1).
- AC4. Given fasyankes berhenti memakai RME minimalis, When Admin Fasyankes meminta ekspor, Then rekam medis milik fasyankes itu dapat diekspor sebelum akses ditutup `[UP-50]`. Perlakuan data setelahnya menunggu L-05.

#### US-1302 Akun dan peran pengguna RME

Sebagai Admin Fasyankes, saya ingin mengelola akun staf fasyankes saya, agar setiap staf hanya punya akses yang dibutuhkan.
Rujukan: BR-120 `[UP-26]` `[KEPUTUSAN-06]` · Audit: E-10, E-11

- AC1. Given Admin Fasyankes membuat akun dengan peran Tenaga Kesehatan Fasyankes, Petugas Lab, atau Admin Fasyankes, When disimpan, Then undangan aktivasi dikirim, dan MFA wajib saat aktivasi.
- AC2. Given akun Tenaga Kesehatan Fasyankes, When dibuat, Then identitas profesi yang dibutuhkan SATUSEHAT wajib diisi. Rinciannya ditetapkan di Tahap 4 `[SRC-26]` `[TIDAK DIKETAHUI]`.
- AC3. Given pengguna fasyankes A, When ia mencoba mengakses data fasyankes B, Then ditolak dan tercatat (KU-03).
- AC4. Given Admin Fasyankes menonaktifkan akun atau mengubah perannya dengan alasan, When disimpan, Then akses berakhir seketika termasuk sesi yang masih terbuka (KU-10), dan tercatat (E-10, E-11).
- AC5. Given hanya tersisa satu Admin Fasyankes aktif, When akunnya akan dinonaktifkan atau perannya dicabut, Then aksi ditolak.
- AC6. Given akun Tenaga Kesehatan Fasyankes milik orang yang juga tenaga kesehatan Rumah Sehat di klinik itu, When Admin Fasyankes menghubungkan kedua akun, Then tenaga kesehatan itu dapat menulis rekam medis home visit yang ditugaskan kepadanya (US-1306 AC6) `[UP-26]`.

#### US-1303 Jadwal, layanan, dan harga

Sebagai Admin Fasyankes, saya ingin mengelola jadwal, layanan, dan harga, agar pasien Rumah Sehat dapat memesan layanan fasyankes saya.
Rujukan: BR-122, AB-17 `[USER] T3-Q1b` `[UP-46]` · Audit: E-14

- AC1. Given Admin Fasyankes mengisi layanan (nama, jenis: layanan klinik, pemeriksaan lab, paket MCU beserta daftar pemeriksaannya, atau home visit beserta area layanannya), harga rekanan, dan jadwal (hari, jam, kuota per slot, hari libur), When disimpan, Then data terkirim ke Rumah Sehat lewat integrasi dan tampil di aplikasi pasien paling lambat 15 menit kemudian (NFR-INT-02).
- AC2. Given harga diubah, When ada pesanan terbayar dengan harga lama, Then pesanan itu tidak berubah (AB-17).
- AC3. Given slot dengan pesanan terbayar dihapus, When disimpan, Then ditolak sampai pesanan dibatalkan rekanan (RF-05).
- AC4. Given perubahan jam, kuota, atau durasi memengaruhi slot yang sudah dipesan dan dibayar, When disimpan, Then perubahan ditolak sampai pesanan itu dibatalkan rekanan (RF-05).

#### US-1304 Appointment dan check-in

Sebagai Admin Fasyankes, saya ingin melihat appointment dari Rumah Sehat dan mencatat kedatangan pasien, agar status pasien di aplikasi benar.
Rujukan: BR-125, BR-024, BR-070 `[UP-14]` · Audit: E-09

- AC1. Given pesanan terbayar dari Rumah Sehat, When diterima lewat integrasi, Then appointment tampil di daftar harian dengan nama pasien, layanan, jadwal, dan status lunas.
- AC2. Given pasien datang, When Admin Fasyankes memindai QR atau memasukkan kode, Then appointment berstatus check-in dan status terkirim ke Rumah Sehat (US-404).
- AC3. Given kode milik fasyankes lain, tanggal lain, atau pesanan yang batal, When dimasukkan, Then check-in ditolak dengan alasannya.
- AC4. Given layanan selesai, When Admin Fasyankes menandai "Selesai", Then status terkirim ke Rumah Sehat.

#### US-1305 Antrean

Sebagai Admin Fasyankes, saya ingin menjalankan antrean pasien yang sudah check-in, agar pasien dapat memantaunya di aplikasi.
Rujukan: BR-125, BR-027 `[UP-12]` · Audit: tidak ada aksi sensitif

- AC1. Given pasien check-in, When check-in tersimpan, Then pasien mendapat nomor antrean berurutan per layanan per hari.
- AC2. Given Admin Fasyankes menekan "panggil berikutnya", When disimpan, Then nomor yang sedang dilayani berubah dan terkirim ke Rumah Sehat (US-405).
- AC3. Given pasien yang dipanggil tidak ada, When Admin Fasyankes menandainya dilewati, Then nomor berikutnya dipanggil, dan pasien yang dilewati dapat dipanggil ulang.

#### US-1306 Catatan rekam medis kunjungan

Sebagai Tenaga Kesehatan Fasyankes, saya ingin mencatat rekam medis kunjungan pasien Rumah Sehat sesuai ketentuan SATUSEHAT, agar fasyankes memenuhi kewajiban rekam medis elektronik.
Rujukan: BR-123, BR-126 `[USER] T3-Q1b` `[SRC-18]` `[SRC-28]` `[UP-27]` · Audit: E-05, E-06

- AC1. Given appointment sudah check-in, When Tenaga Kesehatan Fasyankes mengisi rekam medis kunjungan dengan kelompok data minimal (keluhan/anamnesis, pemeriksaan fisik termasuk tanda vital, diagnosis, tindakan, peresepan obat, permintaan pemeriksaan lab, dan resume) lalu memfinalkannya, Then rekam medis tersimpan atas nama penulis dan fasyankes `[UP-27]`.
- AC2. Given kelompok data dan kode yang dipakai, When diperiksa, Then mengikuti dokumentasi SATUSEHAT. Daftar final kelompok data, variabel, dan terminologinya ditetapkan di Tahap 4 setelah L-05 `[SRC-28]` `[TIDAK DIKETAHUI]`.
- AC3. Given rekam medis sudah final, When penulis mengoreksinya, Then versi baru dibuat dengan alasan koreksi, dan versi lama tetap tersimpan.
- AC4. Given pasien tidak datang lewat Rumah Sehat (tanpa appointment Rumah Sehat), When pengguna mencoba membuat rekam medis untuknya, Then aplikasi tidak menyediakan pendaftaran pasien baru (BR-126).
- AC5. Given pengguna berperan Admin Fasyankes, Petugas Lab, atau Operator RME, When mencoba membuat atau mengubah rekam medis kunjungan, Then ditolak. Admin Fasyankes hanya dapat membaca (§3.1) `[UP-26]`.
- AC6. Given home visit dari klinik pengguna RME berstatus "Tiba" atau "Selesai", When Tenaga Kesehatan Fasyankes yang terhubung ke tenaga kesehatan yang ditugaskan (US-1302 AC6) mengisi rekam medis, Then rekam medis kunjungan home visit tersimpan dengan aturan AC1–AC3.

#### US-1307 Hasil lab

Sebagai Petugas Lab, saya ingin mengisi atau mengunggah hasil pemeriksaan, agar hasil sampai ke pasien lewat Rumah Sehat.
Rujukan: BR-124, BR-025 `[USER] T3-Q1b` · Audit: E-06, E-17

- AC1. Given pemeriksaan lab pasien Rumah Sehat sudah check-in, When Petugas Lab mengisi nilai per pemeriksaan (nilai, satuan, nilai rujukan) dan/atau mengunggah PDF, lalu memfinalkannya, Then hasil terkirim ke Rumah Sehat (US-901).
- AC2. Given hasil sudah terkirim, When Petugas Lab mengoreksinya, Then versi baru terkirim dengan alasan koreksi (US-901 AC3).
- AC3. Apakah hasil wajib divalidasi penanggung jawab lab sebelum dikirim `[TIDAK DIKETAHUI]` (L-05).

#### US-1308 Home visit

Sebagai Admin Fasyankes, saya ingin melihat pesanan home visit dan menugaskan tenaga kesehatan, agar kunjungan dilayani.
Rujukan: BR-125, BR-022 `[USER] T3-Q1b` `[UP-13]` · Audit: E-09

- AC1. Given pesanan home visit terbayar diterima, When Admin Fasyankes membuka daftar, Then pesanan tampil dengan jadwal, alamat, layanan, dan nama pasien.
- AC2. Given Admin Fasyankes memilih tenaga kesehatan dari daftar tenaga kesehatan aktif klinik itu di Rumah Sehat, When penugasan disimpan, Then penugasan terkirim ke Rumah Sehat (US-502).
- AC3. Given status kunjungan diperbarui tenaga kesehatan di aplikasi Rumah Sehat (US-503), When status diterima, Then status tampil di RME minimalis.

#### US-1309 Pengiriman data rekam medis ke SATUSEHAT

Sebagai fasyankes pengguna RME minimalis, saya ingin rekam medis yang sudah final terkirim ke SATUSEHAT, agar kewajiban keterhubungan SATUSEHAT terpenuhi.
Rujukan: BR-123 `[SRC-18]` `[SRC-26]` `[SRC-27]` `[UP-28]` · Audit: E-17

- AC1. Given rekam medis difinalkan, When fasyankes sudah mengisi kredensial dan identitas SATUSEHAT-nya, Then status pengiriman "Antre" lalu "Terkirim" atau "Gagal" (dengan alasan). Given kredensial belum diisi, Then status "Menunggu kredensial", dan rekam medis otomatis masuk antrean setelah kredensial diisi `[KEPUTUSAN-16]`.
- AC2. Given pengiriman gagal karena gangguan, When dicoba ulang, Then percobaan ulang otomatis berjalan sampai 24 jam sejak percobaan pertama, lalu status menjadi "Gagal" dan Admin Fasyankes diberi tahu. Kegagalan karena data klinis tidak valid dikembalikan ke penulis untuk dikoreksi (US-1306 AC3); kegagalan karena data identitas fasyankes diperbaiki Admin Fasyankes `[KEPUTUSAN-16]`.
- AC3. Prasyarat: RME minimalis terdaftar sebagai sistem RME mitra di SATUSEHAT. Pendaftaran memerlukan nomor PSE untuk sistem RME, nomor afiliasi SNOMED CT, kode registrasi fasyankes mitra (atau surat pernyataan resmi bila belum bermitra), survei keamanan, dan verifikasi `[SRC-27]` (PT-32). Cara memperoleh kredensial per fasyankes `[TIDAK DIKETAHUI]`.
- AC4. Given rekam medis yang sudah "Terkirim" dikoreksi (US-1306 AC3), When versi baru difinalkan, Then versi baru dikirim sebagai pembaruan atas data yang sama di SATUSEHAT, dengan status pengiriman sendiri `[KEPUTUSAN-16]`.
- AC5. Given status pengiriman "Gagal", When penyebabnya sudah diperbaiki dan Admin Fasyankes menekan kirim ulang, Then status kembali ke "Antre" dan aturan AC1–AC2 berlaku `[KEPUTUSAN-16]`.

#### US-1310 Riwayat rekam medis pasien di fasyankes

Sebagai Tenaga Kesehatan Fasyankes, saya ingin melihat riwayat rekam medis pasien di fasyankes saya, agar pelayanan berkesinambungan.
Rujukan: BR-123, BR-126 `[UP-17]` · Audit: E-05

- AC1. Given pasien punya kunjungan sebelumnya di fasyankes yang sama, When Tenaga Kesehatan Fasyankes membuka pasien itu, Then riwayat rekam medis di fasyankes itu tampil.
- AC2. Given data pasien di Rumah Sehat (isi chat, catatan konsultasi online, hasil dari fasyankes lain), When Tenaga Kesehatan Fasyankes mencarinya, Then data itu tidak tersedia.
- AC3. Given Admin Fasyankes membuka log akses rekam medis fasyankesnya, When tampil, Then setiap akses tercatat dengan siapa, kapan, dan pasien mana `[UP-26]`.

### EP-14 Integrasi rekanan dan pihak ketiga

Kontrak integrasi rekanan berlaku sama untuk sistem milik rekanan dan RME minimalis (AB-61). Mekanisme teknis (API, format, keamanan) ditetapkan di Tahap 4. Kemampuan sistem rekanan `[TIDAK DIKETAHUI]` (PT-05).

| Data | Arah | Klinik | Lab | Outlet | Rujukan |
| --- | --- | --- | --- | --- | --- |
| Layanan, harga rekanan, paket MCU | Rekanan → Rumah Sehat | Ya | Ya | — | BR-020, BR-021, BR-023, BR-070 |
| Jadwal, slot, kuota | Rekanan → Rumah Sehat | Ya | Ya | — | BR-020, BR-021 |
| Layanan dan area home visit | Rekanan → Rumah Sehat | Ya | — | — | BR-022 `[UP-22]` |
| Katalog, harga, stok | Rekanan → Rumah Sehat | — | — | Ya | BR-033 |
| Pemeriksaan harga dan ketersediaan sebelum bayar | Rumah Sehat → rekanan (permintaan) | Ya | Ya | Ya | AB-15 `[UP-25]` |
| Pesanan terbayar beserta status lunas dan data penerima layanan minimal | Rumah Sehat → rekanan | Ya | Ya | Ya | BR-070 (data minimal: Tahap 4, L-08) |
| Pembatalan | Dua arah | Ya | Ya | Ya | BR-045, BR-070 |
| Check-in, selesai | Rekanan → Rumah Sehat | Ya | Ya | — | BR-024, BR-070 `[UP-14]` |
| Antrean | Rekanan → Rumah Sehat | Ya | — | — | BR-027 |
| Daftar tenaga kesehatan aktif klinik | Rumah Sehat → rekanan | Ya | — | — | BR-005, BR-022 |
| Penugasan home visit | Rekanan → Rumah Sehat | Ya | — | — | BR-022 `[UP-13]` |
| Status kunjungan home visit | Rumah Sehat → rekanan | Ya | — | — | BR-022 |
| Hasil lab | Rekanan → Rumah Sehat | — | Ya | — | BR-025 |
| Status pesanan produk (diterima, ditolak, diproses, diserahkan ke kurir) | Rekanan → Rumah Sehat | — | — | Ya | BR-033 |

| Story | Fitur | BR | Prioritas |
| --- | --- | --- | --- |
| US-1401 | Spesifikasi kontrak integrasi dan lingkungan uji | BR-070, BR-121 | Must |
| US-1402 | Sinkronisasi data rekanan | BR-070 | Must |
| US-1403 | Pengiriman pesanan dan pembatalan ke rekanan | BR-070 | Must |
| US-1404 | Penerimaan status dari rekanan | BR-070 | Must |
| US-1405 | Pemantauan dan status integrasi per rekanan | BR-070, BR-090 | Must |
| US-1406 | Integrasi pihak ketiga | BR-071 | Must |

#### US-1401 Spesifikasi kontrak integrasi dan lingkungan uji

Sebagai rekanan, saya ingin spesifikasi integrasi dan lingkungan uji, agar sistem saya dapat terhubung ke Rumah Sehat.
Rujukan: BR-070, BR-121 · Audit: tidak ada aksi sensitif

- AC1. Given kontrak integrasi diterbitkan, When rekanan membacanya, Then setiap data di tabel di atas punya definisi format, arah, aturan validasi, dan kode galat.
- AC2. Given rekanan menguji di lingkungan uji, When semua skenario wajib lulus, Then status integrasinya dapat diubah menjadi "Aktif" oleh admin Rumah Sehat (US-1405).
- AC3. Given RME minimalis, When diuji dengan skenario yang sama, Then lulus tanpa jalur khusus.

#### US-1402 Sinkronisasi data rekanan

Sebagai pasien, saya ingin data layanan, harga, jadwal, dan katalog rekanan selalu mutakhir, agar yang saya pesan sesuai kenyataan.
Rujukan: BR-070, BR-033 `[UP-46]` · Audit: tidak ada aksi sensitif

- AC1. Given rekanan mengubah data, When perubahan dikirim atau diambil, Then data tampil di aplikasi paling lambat 15 menit kemudian (NFR-INT-02).
- AC2. Given pesan integrasi tidak valid, When diterima, Then pesan ditolak seluruhnya (tidak diterapkan sebagian) dan galat tercatat.
- AC3. Given rekanan tidak dikenal atau kredensialnya salah, When mengirim data, Then ditolak dan tercatat sebagai kejadian keamanan.

#### US-1403 Pengiriman pesanan dan pembatalan ke rekanan

Sebagai rekanan, saya ingin menerima pesanan terbayar dan pembatalan dari Rumah Sehat, agar dapat melayani pasien.
Rujukan: BR-070, BR-045 · Audit: E-17

- AC1. Given pesanan dibayar, When dikirim ke rekanan, Then pesanan memuat status lunas, layanan atau produk, jadwal atau alamat, dan data penerima layanan minimal.
- AC2. Given pengiriman gagal, When dicoba ulang, Then rekanan tidak menerima pesanan ganda (idempoten).
- AC3. Given pesanan dibatalkan pasien atau sistem, When pembatalan dikirim, Then rekanan menerimanya dan slot atau stok dapat dilepas.

#### US-1404 Penerimaan status dari rekanan

Sebagai Rumah Sehat, saya ingin menerima status dari sistem rekanan, agar status pesanan pasien benar.
Rujukan: BR-070 · Audit: E-09

- AC1. Given rekanan mengirim status (check-in, selesai, antrean, penugasan, status pesanan produk, hasil lab), When diterima, Then status pesanan diperbarui sesuai §3.6.
- AC2. Given status tidak sesuai urutan (misalnya "selesai" untuk pesanan yang dibatalkan), When diterima, Then ditolak dan galat dikembalikan ke rekanan.
- AC3. Given rekanan membatalkan pesanan terbayar, When diterima, Then status berubah sesuai tabel §3.6: layanan terjadwal menjadi "Dibatalkan rekanan" dengan RF-05; pesanan produk menjadi "Ditolak outlet" dengan RF-08. Pembatalan dari status yang tidak diizinkan (misalnya setelah Check-in atau setelah Dikirim) ditolak sesuai AC2.

#### US-1405 Pemantauan dan status integrasi per rekanan

Sebagai admin Rumah Sehat, saya ingin memantau kesehatan integrasi setiap rekanan, agar gangguan cepat diketahui.
Rujukan: BR-070, BR-090 `[UP-46]` · Audit: E-10 (perubahan status integrasi)

- AC1. Given admin membuka pemantauan integrasi, When tampil, Then setiap rekanan memuat status integrasi (Belum, Uji, Aktif, Terganggu, Nonaktif), waktu sinkron terakhir yang berhasil, dan jumlah galat 24 jam terakhir.
- AC2. Given integrasi satu rekanan gagal terus selama 30 menit, When batas terlewati, Then status menjadi "Terganggu" dan admin Rumah Sehat menerima peringatan (NFR-INT-03).
- AC3. Given status integrasi "Aktif", When KPI-03/KPI-04 dihitung, Then rekanan itu terhitung terintegrasi, termasuk pengguna RME minimalis (AB-64).

#### US-1406 Integrasi pihak ketiga

Sebagai Rumah Sehat, saya ingin terhubung dengan penyedia pihak ketiga, agar login, pembayaran, video, kurir, dan notifikasi berjalan.
Rujukan: BR-071, BR-113, BR-115 · Audit: mengikuti story pemakai

- AC1. Given penyedia (login Google dan Apple, payment gateway dengan pembagian dana, video, kurir, WhatsApp, email, push) dipilih di Tahap 4–5 dan disetujui user, When integrasi dibangun, Then setiap fitur pemakainya (US-102, US-802, US-306, US-703, US-1001) lulus AC-nya.
- AC2. Given penyedia memproses data pribadi di luar Indonesia, When dipilih, Then pemilihan mengikuti keputusan PT-22 dan konfirmasi L-09 (BR-113).
- AC3. Given satu penyedia gangguan, When fitur lain dipakai, Then fitur lain tetap berjalan (KU-08).

### EP-15 Audit trail (fitur)

Spesifikasi isi audit trail ada di §7. Story di bawah adalah fiturnya.

| Story | Fitur | BR | Prioritas |
| --- | --- | --- | --- |
| US-1501 | Pencatatan event audit | BR-080, BR-081, BR-107 | Must |
| US-1502 | Halaman Audit Log | BR-080 | Must |
| US-1503 | Export audit log | BR-080 | Must |
| US-1504 | Perlindungan audit log | BR-080 | Must |

#### US-1501 Pencatatan event audit

Sebagai Auditor, saya ingin setiap aksi sensitif tercatat lengkap, agar dapat ditelusuri siapa melakukan apa.
Rujukan: BR-080, BR-081, BR-107, ADR-005 `[KEPUTUSAN-07]` `[KEPUTUSAN-15]` · Audit: §7

- AC1. Given aksi yang termasuk event §7.1 terjadi di komponen mana pun (aplikasi pasien, aplikasi tenaga kesehatan, backoffice, portal, RME minimalis, proses otomatis, integrasi), When aksi selesai atau gagal, Then satu event tercatat dengan semua field wajib §7.2 terisi, dan field bersyarat terisi bila kondisinya berlaku.
- AC2. Given event dicatat, When isinya diperiksa, Then tidak memuat data terlarang §7.3.
- AC3. Given event audit untuk suatu aksi §7.1 tidak dapat dicatat, When aksi dijalankan, Then: aksi pengguna dibatalkan dan pesan gagal tampil; aksi sistem atau integrasi tidak dianggap selesai dan dicoba ulang dari antrean sampai event tercatat (pesan integrasi masuk dibalas galat sementara agar dikirim ulang); efek eksternal yang sudah terjadi (misalnya refund yang sudah dieksekusi penyedia) dicatat dari antrean dan tidak boleh hilang. Kegagalan mencatat event memicu peringatan keamanan. Event aksi gagal (misalnya login gagal) tetap dicoba dicatat tanpa mengubah hasil aksinya `[KEPUTUSAN-15]`.

#### US-1502 Halaman Audit Log

Sebagai Auditor, saya ingin mencari dan memfilter audit log, agar investigasi dan pemeriksaan kepatuhan cepat.
Rujukan: BR-080, ADR-005 `[UP-18]` · Audit: tidak ada aksi sensitif

- AC1. Given Auditor atau Super Admin membuka Audit Log, When ia memfilter rentang waktu, aktor, peran, jenis event, jenis dan ID objek, subjek data, hasil, atau alamat IP, Then hasil tampil urut terbaru, berhalaman.
- AC2. Given Auditor membuka satu event, When detail tampil, Then semua field §7.2 tampil dengan nilai termasking sesuai §7.3.
- AC3. Given pengguna selain Auditor dan Super Admin, When mencoba membuka Audit Log, Then ditolak (KU-03).

#### US-1503 Export audit log

Sebagai Auditor, saya ingin mengekspor hasil filter audit log, agar dapat diserahkan ke auditor eksternal.
Rujukan: BR-080 butir 9 · Audit: E-12

- AC1. Given Auditor menerapkan filter, When ia mengekspor, Then file CSV berisi hasil filter dengan masking §7.3, dan event export (siapa, kapan, filter, jumlah baris) tercatat.
- AC2. Given Super Admin, When ia mencari tombol export, Then tombol tidak tersedia (§3.1).

#### US-1504 Perlindungan audit log

Sebagai Auditor, saya ingin audit log tidak dapat diubah atau dihapus siapa pun, agar bisa dipercaya sebagai bukti.
Rujukan: BR-080, ADR-005 · Audit: kejadian keamanan

- AC1. Given pengguna mana pun, termasuk Super Admin, When mencari fitur ubah atau hapus audit log, Then fitur tidak tersedia di aplikasi maupun API.
- AC2. Given ada permintaan langsung ke API untuk mengubah atau menghapus event, When diterima, Then ditolak dan tercatat sebagai kejadian keamanan.
- AC3. Mekanisme penyimpanan append-only dan deteksi manipulasi ditetapkan di Tahap 4 (ADR-005).

### EP-16 Persetujuan dan hak subjek data (T1)

| Story | Fitur | BR | Prioritas |
| --- | --- | --- | --- |
| US-1601 | Lihat dan tarik persetujuan | BR-103, BR-106 | Must |
| US-1602 | Ajukan permintaan hak subjek data | BR-106 | Must |
| US-1603 | Proses permintaan hak subjek data | BR-106, BR-111 | Must |
| US-1604 | Salinan data pribadi | BR-106 | Must |
| US-1605 | Penghapusan dan anonimisasi | BR-106, BR-112 | Must |
| US-1606 | Retensi dan pemusnahan otomatis | BR-112 | Must |

#### US-1601 Lihat dan tarik persetujuan

Sebagai pasien, saya ingin melihat persetujuan yang pernah saya berikan dan menariknya, agar kendali atas data saya tetap di tangan saya.
Rujukan: BR-103, BR-106 `[SRC-15]` `[UP-47]` · Audit: E-13

- AC1. Given pasien membuka pengaturan privasi, When tampil, Then daftar persetujuan (butir, versi, tanggal) tampil.
- AC2. Given pasien menarik butir persetujuan utama, When ia mengonfirmasi setelah membaca akibatnya, Then akun berstatus "Persetujuan ditarik": pasien hanya dapat masuk untuk melihat riwayat dan status, mengajukan hak subjek data, dan memberi persetujuan kembali; pemesanan baru ditolak; data tidak lagi dikirim ke rekanan atau kanal notifikasi, kecuali untuk menuntaskan pesanan yang tetap diselesaikan (AC3) dan memenuhi kewajiban simpan (L-06). Perubahan ini berlaku paling lambat 3 x 24 jam `[UP-54]`.
- AC3. Given pasien punya pesanan terbayar yang belum selesai, When ia menarik butir persetujuan utama, Then sebelum konfirmasi ia diberi tahu bahwa pesanan terjadwal akan dibatalkan dengan aturan pembatalan pasien (RF-01 atau RF-02), sedangkan pesanan produk yang sudah dibayar tetap diselesaikan sampai diterima. Kelayakan pembatalan tanpa refund (RF-02) karena penarikan persetujuan ditanyakan ke legal (L-08, L-14) `[UP-47]` `[UP-54]`.
- AC4. Given pasien menarik butir persetujuan tambahan (KEPUTUSAN-12), When dikonfirmasi, Then hanya pemrosesan untuk tujuan butir itu yang berhenti, dan layanan lain tetap dapat dipakai `[UP-54]`.

#### US-1602 Ajukan permintaan hak subjek data

Sebagai pasien, saya ingin mengajukan permintaan hak atas data saya, agar dapat menjalankan hak yang diatur UU PDP.
Rujukan: BR-106 `[SRC-15]` · Audit: E-13

- AC1. Given pasien memilih jenis permintaan (informasi, akses dan salinan, perbaikan, penghapusan, penundaan atau pembatasan, keberatan, portabilitas) untuk dirinya atau anggota keluarganya, When ia mengirim setelah autentikasi ulang (password untuk akun email, atau otorisasi ulang Google/Apple untuk akun yang masuk lewat Google/Apple), Then permintaan berstatus "Diterima" dan batas waktunya tampil.
- AC2. Given permintaan masih diproses, When pasien membuka status, Then tahap penanganan tampil.
- AC3. Cara anggota keluarga dewasa menjalankan haknya sendiri tanpa lewat pemilik akun `[TIDAK DIKETAHUI]` (L-08).

#### US-1603 Proses permintaan hak subjek data

Sebagai Petugas PDP, saya ingin memproses permintaan hak subjek data dalam batas waktu, agar Rumah Sehat patuh UU PDP.
Rujukan: BR-106, BR-111 `[SRC-15]` `[UP-18]` `[UP-47]` · Audit: E-13

- AC1. Given permintaan masuk, When Petugas PDP membuka antrean, Then setiap permintaan menampilkan sisa waktu menuju batas 3 x 24 jam.
- AC2. Given batas waktu tinggal 12 jam, When belum selesai, Then permintaan ditandai mendesak dan Super Admin diberi tahu `[UP-47]`.
- AC3. Given permintaan diselesaikan atau ditolak, When Petugas PDP mengisi tanggapan dan alasan, Then pasien menerima tanggapan lewat email dan aplikasi.
- AC4. Batas 3 x 24 jam ditetapkan UU PDP untuk akses, perbaikan, penundaan/pembatasan, dan penghentian pemrosesan setelah persetujuan ditarik `[SRC-15]`. Untuk jenis lain, batas yang sama dipakai sebagai target internal `[UP-47]`.
- AC5. Given permintaan perbaikan, When Petugas PDP menyetujuinya, Then data yang diminta diubah lewat backoffice; perubahan nama, tanggal lahir, atau NIK mengembalikan status identitas ke "Menunggu verifikasi" (US-105 AC3); perubahan tercatat (E-10, E-13).
- AC6. Given permintaan penundaan/pembatasan atau keberatan disetujui, When Petugas PDP menerapkannya, Then akun berstatus "Pemrosesan dibatasi" dengan efek yang sama seperti US-1601 AC2, sampai Petugas PDP mencabutnya atas permintaan pasien atau setelah keberatan diputuskan; pencabutan tercatat (E-13) `[UP-54]`.
- AC7. Given permintaan informasi, When diselesaikan, Then tanggapan tertulis beserta waktu dan isi tanggapan tersimpan dan terkirim ke pasien. Pengerjaan manual di luar sistem diperbolehkan, tetapi sistem tetap mencatat permintaan, tanggapan, bukti, dan waktunya.

#### US-1604 Salinan data pribadi

Sebagai pasien, saya ingin mengunduh salinan data pribadi saya dalam format yang dapat dibaca mesin, agar dapat memakai hak akses dan portabilitas.
Rujukan: BR-106 `[SRC-15]` · Audit: E-12

- AC1. Given permintaan akses atau portabilitas disetujui, When salinan dibuat, Then file berisi profil, identitas, anggota keluarga, transaksi, isi chat, catatan konsultasi, resep, hasil lab, dan rekam jejak pemrosesan data pribadinya (Pasal 32 ayat (1) `[SRC-15]`; bersumber dari field subjek data audit trail §7.2 dan event E-17, bentuknya ditetapkan di Tahap 4) dalam format yang dapat dibaca mesin dan format yang dapat dibaca orang, paling lambat 3 x 24 jam sejak permintaan.
- AC2. Given salinan siap, When pasien mengunduhnya dari aplikasi setelah autentikasi ulang (seperti US-1602 AC1), Then unduhan tercatat. Tautan unduhan berlaku 7 hari `[UP-47]`.
- AC3. Rekam medis yang disimpan fasyankes di RME minimalis termasuk atau tidak `[TIDAK DIKETAHUI]` (L-05).

#### US-1605 Penghapusan dan anonimisasi

Sebagai pasien, saya ingin data saya dihapus bila saya meminta, sejauh tidak ada kewajiban menyimpannya.
Rujukan: BR-106, BR-112, ADR-003, ADR-004 · Audit: E-13, E-16

- AC1. Given permintaan penghapusan, When Petugas PDP memeriksa kewajiban retensi per jenis data, Then data tanpa kewajiban simpan dihapus atau dianonimkan, sedangkan catatan transaksi tetap utuh dengan data pribadinya dianonimkan (ADR-003, ADR-004).
- AC2. Given ada data yang wajib disimpan, When tanggapan dikirim, Then pasien diberi tahu data mana yang tetap disimpan, alasannya, dan sampai kapan.
- AC3. Story ini **terblokir** sampai L-06 dan PT-24 terjawab, karena aturan benturan hak hapus dengan retensi rekam medis belum diketahui.

#### US-1606 Retensi dan pemusnahan otomatis

Sebagai Rumah Sehat, saya ingin data diakhiri pemrosesannya, dihapus, atau dimusnahkan saat masa retensinya habis, agar sesuai BR-112.
Rujukan: BR-112 `[SRC-15]` · Audit: E-16

- AC1. Given masa retensi per jenis data diatur, When masa retensi suatu data habis, Then proses terjadwal mengakhiri pemrosesannya, menghapus, atau memusnahkannya sesuai aturan jenis datanya, dan laporan hasilnya tersimpan.
- AC2. Masa retensi per jenis data `[TIDAK DIKETAHUI]` (PT-24). Story ini terblokir sampai PT-24 terjawab.

### EP-17 Pengukuran KPI

| Story | Fitur | BR | Prioritas |
| --- | --- | --- | --- |
| US-1701 | Dasbor KPI | BR-090 | Must |

#### US-1701 Dasbor KPI

Sebagai Super Admin, saya ingin melihat KPI-01 sampai KPI-04, agar kemajuan menuju target 1 Maret 2028 terpantau.
Rujukan: BR-090, KPI-01–KPI-04, AB-54, AB-64 `[UP-24]` · Audit: E-12 (export)

- AC1. Given bulan kalender dipilih (WIB), When dasbor tampil, Then KPI-01 resmi = jumlah pemilik akun unik yang punya minimal satu layanan berstatus akhir berhasil di bulan itu: konsultasi "Selesai", klinik/lab "Check-in" atau "Selesai", home visit "Selesai", atau produk "Diterima" `[UP-24]`.
- AC2. Given dasbor tampil, When angka pendukung dilihat, Then tampil jumlah pemilik akun unik yang membuka aplikasi dalam keadaan masuk minimal sekali di bulan itu.
- AC3. Given dasbor tampil, When KPI-02 dilihat, Then tampil jumlah dokter dan psikolog yang siap melayani (AB-54) pada akhir bulan dan saat ini.
- AC4. Given dasbor tampil, When KPI-03 dan KPI-04 dilihat, Then tampil jumlah klinik dan lab berstatus integrasi "Aktif", termasuk pengguna RME minimalis (AB-64).
- AC5. Given Super Admin mengekspor, When file dibuat, Then CSV berisi angka agregat tanpa data pribadi, dan event export tercatat.

## 6. Kebutuhan non-fungsional (NFR)

### 6.1 Performa

Diukur di sisi server pada beban NFR-CAP, tanpa waktu panggilan ke pihak ketiga dan sistem rekanan `[USER] T3-Q13`.

| ID | Kebutuhan | Target | Sumber |
| --- | --- | --- | --- |
| NFR-PERF-01 | Waktu respons API untuk membaca data | p95 ≤ 500 ms | `[USER] T3-Q13` |
| NFR-PERF-02 | Waktu respons API untuk menulis data | p95 ≤ 1 detik | `[USER] T3-Q13` |
| NFR-PERF-03 | Pesan chat sampai ke penerima yang sedang online | p95 ≤ 2 detik | `[USER] T3-Q13` |
| NFR-PERF-04 | Halaman web tampil: portal rekanan; juga backoffice dan RME minimalis | p95 ≤ 3 detik | `[USER] T3-Q13` (portal) `[UP-48]` (backoffice, RME minimalis) |
| NFR-PERF-05 | Notifikasi diserahkan ke penyedia kanal setelah kejadian | ≤ 1 menit | `[USER] T3-Q13` |

Tidak dijadikan target karena bergantung pada penyedia: waktu tersambung panggilan video dan waktu sampainya notifikasi di perangkat `[USER] T3-Q13`.

### 6.2 Kapasitas

| ID | Kebutuhan | Target | Sumber |
| --- | --- | --- | --- |
| NFR-CAP-01 | Pasien aktif bulanan yang dilayani tanpa penurunan di bawah NFR-PERF | 30.000 | `[USER] T3-Q11` |
| NFR-CAP-02 | Pengguna aktif bersamaan, yaitu pengguna yang mengirim permintaan dalam jendela 1 menit yang sama | 1.000 | `[USER] T3-Q11` `[KEPUTUSAN-13]` (definisi) |
| NFR-CAP-03 | Sesi konsultasi (chat atau video) bersamaan | 100 | `[USER] T3-Q11` |
| NFR-CAP-04 | Uji beban pada NFR-CAP-01–03 dilakukan sebelum go-live, dan hasilnya memenuhi NFR-PERF | Lulus | `[USER] T3-Q11` |

### 6.3 Ketersediaan dan pemulihan

| ID | Kebutuhan | Target | Sumber |
| --- | --- | --- | --- |
| NFR-AVL-01 | Ketersediaan bulanan layanan inti: login, pemesanan, pembayaran, konsultasi, portal | 99,0% (gangguan maksimal sekitar 7,2 jam per 30 hari) | `[USER] T3-Q12` |
| NFR-AVL-02 | Ketersediaan bulanan RME minimalis | 99,0% | `[UP-48]` |
| NFR-AVL-03 | Pemeliharaan terjadwal tidak dihitung sebagai gangguan bila diumumkan paling lambat 48 jam sebelumnya dan dilakukan di jam sepi | — | `[USER] T3-Q12` |
| NFR-AVL-04 | Gangguan sistem rekanan dan penyedia pihak ketiga dicatat terpisah dan tidak dihitung di NFR-AVL-01 | — | `[USER] T3-Q12` |
| NFR-DR-01 | Batas kehilangan data saat bencana (RPO) | ≤ 15 menit | `[USER] T3-Q13` |
| NFR-DR-02 | Batas waktu pemulihan (RTO) | ≤ 4 jam | `[USER] T3-Q13` |
| NFR-DR-03 | Uji pemulihan dari backup dilakukan minimal sekali sebelum go-live, dan hasilnya memenuhi NFR-DR-01 dan NFR-DR-02 | Lulus | `[KEPUTUSAN-14]` |
| NFR-DR-04 | Backup disimpan di Indonesia | — | BR-113 |

### 6.4 Keamanan (T1)

| ID | Kebutuhan | Sumber |
| --- | --- | --- |
| NFR-SEC-01 | Kontrol keamanan aplikasi diverifikasi terhadap OWASP ASVS 5.0.0. Level dan pemetaan kontrol ditetapkan di Tahap 4, verifikasi di Tahap 10 | `[SRC-29]` `[KEPUTUSAN-05]` |
| NFR-SEC-02 | MFA wajib untuk akun admin Rumah Sehat, admin rekanan, dan pengguna RME minimalis. MFA untuk tenaga kesehatan ditetapkan di Tahap 4 | `[KEPUTUSAN-06]` |
| NFR-SEC-03 | Kebijakan password mengikuti NFR-SEC-01. Akun terkunci 15 menit setelah 5 percobaan masuk gagal berturut-turut | `[KEPUTUSAN-06]` |
| NFR-SEC-04 | Sesi web (backoffice, portal, RME minimalis) berakhir setelah 30 menit tanpa aktivitas | `[KEPUTUSAN-06]` |
| NFR-SEC-05 | Data dienkripsi saat dikirim dan saat disimpan. Data pribadi spesifik (AB-70) mendapat perlindungan tambahan yang ditetapkan di Tahap 4 | BR-105, BR-108 |
| NFR-SEC-06 | Isolasi data antar-rekanan dan antar-fasyankes diuji, termasuk percobaan mengakses data lain lewat ID (KU-03) | BR-105, BR-108 |
| NFR-SEC-07 | Hak akses mengikuti prinsip hak minimum per peran dan sub-peran (§3.1) | BR-105 `[UP-17]` |
| NFR-SEC-08 | Berkas unggahan dibatasi jenis dan ukurannya (foto JPG/PNG ≤ 5 MB; dokumen PDF ≤ 10 MB) dan hanya dapat diakses pihak yang berhak | `[KEPUTUSAN-08]` |
| NFR-SEC-09 | Lulus audit keamanan sebelum go-live | `[DOK] P8.2` (syarat go-live 6) |

### 6.5 Privasi dan lokasi data (T1)

| ID | Kebutuhan | Sumber |
| --- | --- | --- |
| NFR-PRIV-01 | Data dan backup disimpan di server atau cloud yang berlokasi di Indonesia, termasuk data RME minimalis | BR-113 `[DOK] P5.5` |
| NFR-PRIV-02 | Pengiriman data pribadi ke penyedia di luar Indonesia hanya bila sesuai keputusan PT-22 dan konfirmasi L-09 | BR-113 |
| NFR-PRIV-03 | Data yang dikirim ke payment gateway, rekanan, dan kanal notifikasi dibatasi pada yang diperlukan (AB-73, AB-75) | BR-105 |
| NFR-PRIV-04 | Panggilan video tidak direkam (AB-74) | BR-016 |
| NFR-PRIV-05 | Bukti persetujuan tersimpan per versi teks (US-104) | BR-103 |

### 6.6 Retensi (T1)

| ID | Kebutuhan | Sumber |
| --- | --- | --- |
| NFR-RET-01 | Setiap jenis data punya masa retensi yang dapat diatur, dan dijalankan otomatis (US-1606) | BR-112 |
| NFR-RET-02 | Angka retensi per jenis data, termasuk audit log dan rekam medis RME minimalis | `[TIDAK DIKETAHUI]` (PT-24, L-05). Masukan dari BRD BR-112: berita resmi Setkab tentang PP 80/2019 menyebut 10 tahun untuk data PMSE terkait transaksi keuangan dan 5 tahun untuk lainnya; status terkini PP itu belum diverifikasi `[SRC-25]` |

### 6.7 Kompatibilitas

| ID | Kebutuhan | Sumber |
| --- | --- | --- |
| NFR-COMP-01 | Aplikasi pasien dan aplikasi tenaga kesehatan berjalan di Android dan iOS | BR-007 `[USER] T3-Q9` |
| NFR-COMP-02 | Versi OS minimum ditetapkan di Tahap 5 dari dokumentasi resmi Flutter | `[TIDAK DIKETAHUI]` |
| NFR-COMP-03 | Web (backoffice, portal, RME minimalis) mendukung dua versi stabil terakhir Chrome, Edge, Firefox, dan Safari | `[KEPUTUSAN-09]` |

### 6.8 Lokalisasi

| ID | Kebutuhan | Sumber |
| --- | --- | --- |
| NFR-LOC-01 | Antarmuka hanya Bahasa Indonesia | `[USER] T3-Q1a` (multi-bahasa tidak dipilih) |
| NFR-LOC-02 | Mata uang Rupiah tanpa desimal; zona waktu mengikuti AB-40–AB-42 | `[USER] T3-Q10` `[UP-43]` |

### 6.9 Observabilitas

| ID | Kebutuhan | Sumber |
| --- | --- | --- |
| NFR-OBS-01 | Ketersediaan dipantau otomatis dan gangguan memicu peringatan, sebagai dasar pengukuran NFR-AVL | `[USER] T3-Q12` |
| NFR-OBS-02 | Application log, security log, dan audit trail dipisahkan | ADR-005 |
| NFR-OBS-03 | Application log dan security log juga tidak memuat data terlarang §7.3 | ADR-005 `[KEPUTUSAN-07]` |

### 6.10 Integrasi

| ID | Kebutuhan | Sumber |
| --- | --- | --- |
| NFR-INT-01 | Pesan integrasi diproses idempoten dan dicoba ulang otomatis bila gagal sementara | KU-02, KU-05 |
| NFR-INT-02 | Perubahan data rekanan tampil di aplikasi paling lambat 15 menit | `[UP-46]` |
| NFR-INT-03 | Integrasi rekanan yang gagal terus selama 30 menit memicu peringatan ke admin Rumah Sehat | `[UP-46]` |
| NFR-INT-04 | Notifikasi dari payment gateway diverifikasi keasliannya sebelum diproses | BR-115 `[KEPUTUSAN-06]` |

### 6.11 Aksesibilitas

Standar aksesibilitas ditetapkan di Tahap 6 dari WCAG. Saat ini `[TIDAK DIKETAHUI]`.

## 7. Spesifikasi audit trail (NFR, T1)

Dasar: BR-080, BR-081, BR-107, ADR-005. Fitur halaman Audit Log ada di EP-15.

### 7.1 Event yang dicatat

| Kode | Event | Contoh | Sumber |
| --- | --- | --- | --- |
| E-01 | Login berhasil | Semua jenis akun dan komponen | BR-080 butir 1 |
| E-02 | Login gagal, termasuk akun terkunci | | BR-080 butir 1 |
| E-03 | Logout | | BR-080 butir 1 |
| E-04 | Pembuatan akun dan perubahan kredensial (nilai tidak dicatat) | Registrasi, atur ulang password, aktivasi MFA, penautan Google/Apple | ADR-005 `[KEPUTUSAN-07]` |
| E-05 | Akses ke data pasien sensitif oleh pihak selain subjek data, termasuk pemilik akun yang membuka data klinis anggota keluarga (dewasa maupun anak) | Tenaga kesehatan membuka riwayat chat, Verifikator membuka foto KTP/KK, Tenaga Kesehatan Fasyankes membuka rekam medis, admin membuka bukti klaim | BR-080 butir 2 `[KEPUTUSAN-07]` |
| E-06 | Pembuatan dan perubahan catatan tenaga kesehatan, resep, rekam medis RME, dan hasil lab | | BR-080 butir 3, BR-123 |
| E-07 | Verifikasi identitas disetujui atau ditolak | | BR-080 butir 4 |
| E-08 | Pembayaran, pembagian dana, refund, dan keputusan klaim | | BR-080 butir 5 |
| E-09 | Perubahan status konsultasi, pendaftaran, home visit, dan pesanan | | BR-080 butir 6 |
| E-10 | Penambahan, perubahan, verifikasi, dan penonaktifan akun tenaga kesehatan, admin mitra, rekanan, admin Rumah Sehat, pengguna RME, dan pasien | | BR-080 butir 7 |
| E-11 | Perubahan hak akses atau peran | | BR-080 butir 8 |
| E-12 | Ekspor atau unduh data | Export laporan dan audit log, unduh salinan data pribadi, unduh resep dan hasil lab | BR-080 butir 9 |
| E-13 | Persetujuan diberikan atau ditarik; permintaan hak subjek data diajukan dan ditanggapi | | BR-080 butir 10, BR-103 |
| E-14 | Perubahan pengaturan yang memengaruhi uang atau aturan: markup, biaya rekanan, tarif dan durasi konsultasi, jadwal, parameter bisnis | | BR-080 (catatan), BR-063 |
| E-15 | Penambahan, perubahan, dan penghapusan anggota keluarga beserta pernyataan persetujuannya | | BR-104 |
| E-16 | Soft delete dan pemulihan data master; penghapusan dan anonimisasi atas permintaan atau retensi | | ADR-003, BR-112 |
| E-17 | Pengiriman data pribadi ke pihak luar: sistem rekanan, SATUSEHAT, payment gateway (jenis data dan tujuan, tanpa isi) | | BR-107 |

### 7.2 Field per event

Field minimum mengikuti ADR-005: siapa, apa, kapan, dari mana, nilai before/after, dan hasil. Kolom "Wajib" membedakan field yang selalu terisi dari field bersyarat (US-1501 AC1).

| Field | Isi | Wajib | Sumber |
| --- | --- | --- | --- |
| ID event | Unik | Ya | ADR-005 |
| Waktu | Waktu server. Disimpan dalam satu acuan baku (Tahap 4), ditampilkan dalam WIB | Ya | ADR-005, AB-42 |
| Aktor | ID akun, jenis akun, peran dan sub-peran, atau "sistem" untuk proses otomatis | Ya | ADR-005 |
| Aksi | Kode event (§7.1) dan nama aksi | Ya | ADR-005 |
| Objek | Jenis dan ID objek | Ya | ADR-005 |
| Subjek data | ID pasien atau anggota keluarga yang datanya terkena, bila ada | Bila ada data pribadi yang terkena | BR-106, BR-107 `[KEPUTUSAN-07]` |
| Sumber | Alamat IP, komponen (aplikasi atau web), versi aplikasi, perangkat atau user agent | Ya untuk aksi pengguna dan integrasi; untuk proses otomatis cukup nama komponen | ADR-005 |
| Before/after | Nilai field non-klinis sebelum dan sesudah, termasking sesuai §7.3. Untuk konten klinis dan dokumen hanya daftar field yang berubah dan versi | Bila ada nilai yang berubah | ADR-005 `[KEPUTUSAN-07]` |
| Hasil | Sukses atau gagal, dengan kode alasan | Ya | ADR-005 |
| ID korelasi | Penghubung event dalam satu permintaan atau proses | Ya | `[KEPUTUSAN-07]` |

### 7.3 Data yang dilarang atau di-masking

| Data | Perlakuan | Sumber |
| --- | --- | --- |
| Password, PIN, OTP, token (sesi, refresh, API, payment gateway), rahasia MFA | Tidak pernah dicatat | BR-081, ADR-005 |
| Nomor kartu dan nomor rekening | Tidak pernah lengkap; hanya 4 digit terakhir | BR-081, ADR-005 |
| NIK | Hanya 4 digit terakhir | `[KEPUTUSAN-07]` |
| Isi chat, catatan, resep, rekam medis, nilai hasil lab, foto KTP/KK, foto bukti klaim | Tidak disalin ke audit trail; hanya referensi objek dan versi | BR-105 `[KEPUTUSAN-07]` |

### 7.4 Retensi

Masa simpan audit trail `[TIDAK DIKETAHUI]` (PT-24). Audit trail tidak dihapus sebelum masa retensinya berakhir.

### 7.5 Hak akses

| Peran | Hak | Sumber |
| --- | --- | --- |
| Auditor | Baca dan export | `[UP-18]` |
| Super Admin | Baca | `[UP-18]` |
| Admin Fasyankes | Baca log akses rekam medis fasyankesnya di RME minimalis | `[UP-26]` |
| Semua peran | Tidak ada yang dapat mengubah atau menghapus | ADR-005 |

### 7.6 Perlindungan

Audit trail disimpan append-only, terpisah dari tabel aplikasi, application log, dan security log (ADR-005). Mekanisme penyimpanan, perlindungan dari manipulasi, dan sumber waktu ditetapkan di Tahap 4 (ADR-005).

## 8. Model data konseptual

Daftar entitas bisnis dan relasinya, tanpa detail teknis. Klasifikasi master/transaksi/log adalah klasifikasi awal untuk ADR-003 dan ADR-004, dan difinalkan di Tahap 4 `[KEPUTUSAN-10]`.

### 8.1 Rumah Sehat

| Entitas | Keterangan | Klasifikasi | BR |
| --- | --- | --- | --- |
| Akun | Identitas masuk untuk semua jenis pengguna | Master | BR-001 |
| Pasien | Pemilik akun aplikasi pasien beserta data dirinya | Master | BR-002 |
| Alamat | Alamat pasien untuk pengiriman dan home visit | Master | BR-002 |
| Anggota keluarga | Orang yang dapat dipesankan layanan oleh pasien | Master | BR-008 |
| Dokumen identitas | NIK, foto KTP atau KK, dan status verifikasinya | Master | BR-003, BR-008 |
| Persetujuan | Bukti persetujuan per versi teks dan per butir | Log | BR-103, BR-104 |
| Tenaga kesehatan | Dokter, psikolog, fisioterapis, tenaga medis | Master | BR-005 |
| Dokumen izin tenaga kesehatan | STR, SIP, masa berlaku | Master | BR-102 |
| Rekanan | Klinik, lab, toko kecantikan, toko alat kesehatan, apotek; sumber sistem; status integrasi | Master | BR-006 |
| Dokumen izin rekanan, rekening rekanan | | Master | BR-006, BR-041 |
| Admin (Rumah Sehat, rekanan) dan peran | | Master | BR-060 |
| Pengaturan tarif konsultasi, jadwal praktik online, slot | | Master | BR-017, BR-018 |
| Layanan rekanan | Layanan klinik, pemeriksaan lab, paket MCU, home visit, beserta harga dan jadwal dari integrasi | Master (salinan dari rekanan) | BR-020–BR-023 |
| Produk | Salinan katalog outlet dari integrasi | Master (salinan dari rekanan) | BR-033 |
| Pesanan | Konsultasi, pendaftaran klinik/lab, home visit, atau produk; harga terkunci | Transaksi | BR-040 |
| Riwayat status pesanan | Setiap perubahan status | Transaksi | BR-080 |
| Sesi konsultasi, pesan chat, lampiran | | Transaksi | BR-010, BR-016 |
| Catatan konsultasi (berversi) | | Transaksi | BR-013, BR-114 |
| Resep dan item resep | | Transaksi | BR-015 |
| Appointment, check-in, antrean | | Transaksi | BR-024, BR-027 |
| Kunjungan home visit dan penugasan | | Transaksi | BR-022 |
| Hasil lab (berversi) | | Transaksi | BR-025 |
| Pembayaran | | Transaksi | BR-040 |
| Pembagian dana | Bagian rekanan, markup, biaya | Transaksi | BR-041, BR-044 |
| Refund dan entri pembalik | | Transaksi | BR-045 |
| Klaim | | Transaksi | BR-045 |
| Kewajiban pengembalian rekanan | | Transaksi | BR-045 |
| Pengaturan markup dan biaya rekanan (berlaku mulai) | | Master | BR-042, BR-043 |
| Tagihan biaya rekanan | | Transaksi | BR-043 |
| Parameter bisnis | | Master | §3.7 |
| Notifikasi | | Log | BR-050 |
| Konten CMS | | Master | BR-061 |
| Tiket keluhan | | Transaksi | BR-062 |
| Permintaan hak subjek data | | Transaksi | BR-106 |
| Event audit | | Log append-only | BR-080 |
| Status integrasi dan galat integrasi | | Log | BR-070 |

### 8.2 RME minimalis

| Entitas | Keterangan | Klasifikasi | BR |
| --- | --- | --- | --- |
| Fasyankes | Rekanan klinik atau lab pengguna RME minimalis | Master | BR-120 |
| Pengguna RME dan peran | | Master | BR-120 |
| Layanan, jadwal, harga | | Master | BR-122 |
| Pasien fasyankes | Hanya pasien yang datang lewat Rumah Sehat | Master | BR-126 |
| Kunjungan | Dari appointment atau home visit Rumah Sehat | Transaksi | BR-123, BR-125 |
| Rekam medis kunjungan (berversi) | Kelompok data UP-27 | Transaksi | BR-123 |
| Permintaan dan hasil lab (berversi) | | Transaksi | BR-124 |
| Antrean | | Transaksi | BR-125 |
| Penugasan home visit | | Transaksi | BR-125 |
| Pengiriman SATUSEHAT | Status per rekam medis | Log | BR-123 |
| Event audit RME | | Log append-only | BR-080 |

### 8.3 Relasi utama

| Relasi | Kardinalitas |
| --- | --- |
| Pasien – Anggota keluarga | Satu pasien punya nol atau banyak anggota keluarga; satu NIK anggota keluarga dapat tercatat di lebih dari satu akun (UP-40) |
| Pasien/Anggota keluarga – Dokumen identitas | Satu orang punya satu dokumen identitas aktif dan riwayat pengajuan |
| Rekanan (klinik) – Tenaga kesehatan | Satu klinik menaungi banyak tenaga kesehatan; satu tenaga kesehatan dinaungi satu klinik (AB-50) |
| Tenaga kesehatan – Slot – Sesi konsultasi | Satu tenaga kesehatan punya banyak slot; satu slot paling banyak satu sesi terbayar (AB-52) |
| Pesanan – Penerima layanan | Satu pesanan terjadwal untuk satu orang (pemilik akun atau anggota keluarga) |
| Pesanan – Pembayaran – Pembagian dana | Satu pesanan punya satu pembayaran berhasil (pembayaran ganda menjadi RF-12); satu pembayaran punya satu set pembagian dana |
| Pesanan – Refund – Klaim | Satu pesanan punya nol atau satu refund dan nol atau satu klaim aktif |
| Sesi konsultasi – Catatan – Resep | Satu sesi punya nol atau satu catatan (berversi) dan nol atau banyak resep |
| Pendaftaran lab – Hasil lab | Satu pendaftaran lab punya nol atau banyak hasil (berversi) |
| Rekanan – Fasyankes RME | Satu rekanan bersumber sistem RME minimalis punya tepat satu fasyankes di RME |
| Kunjungan RME – Pesanan Rumah Sehat | Setiap kunjungan RME berasal dari tepat satu pesanan Rumah Sehat (BR-126) |

```mermaid
erDiagram
  PASIEN ||--o{ ANGGOTA_KELUARGA : memiliki
  PASIEN ||--o{ PESANAN : memesan
  REKANAN ||--o{ TENAGA_KESEHATAN : menaungi
  TENAGA_KESEHATAN ||--o{ SLOT : membuka
  PESANAN ||--o| SESI_KONSULTASI : menjadi
  SESI_KONSULTASI ||--o| CATATAN_KONSULTASI : menghasilkan
  SESI_KONSULTASI ||--o{ RESEP : menghasilkan
  PESANAN ||--|| PEMBAYARAN : dibayar
  PEMBAYARAN ||--|{ PEMBAGIAN_DANA : dibagi
  PESANAN ||--o| REFUND : dikembalikan
  REKANAN ||--o{ LAYANAN_REKANAN : menyediakan
  REKANAN ||--o{ PRODUK : menjual
  PESANAN ||--o{ HASIL_LAB : menghasilkan
  REKANAN ||--o| FASYANKES_RME : memakai
  FASYANKES_RME ||--o{ KUNJUNGAN_RME : mencatat
  KUNJUNGAN_RME ||--o| REKAM_MEDIS : berisi
```

## 9. Tinjauan Tech Lead atas NFR dan scope

Angka NFR berlabel `[USER]` tidak diubah. Butir yang berisiko ditandai untuk diputuskan user di gate.

| Butir | Penilaian | Risiko | Opsi untuk diputuskan user |
| --- | --- | --- | --- |
| NFR-AVL-01 99,0% | Realistis. Lebih longgar dari rekomendasi 99,5%, sehingga lebih mudah dicapai tim kecil | Rendah (teknis). Gangguan sampai sekitar 7,2 jam per bulan terasa bagi pasien yang terjadwal | Tetap; tidak ada usulan perubahan |
| NFR-DR-02 RTO 4 jam | Tercapai bila pemulihan diotomasi dan dilatih (NFR-DR-03) | **Sedang.** Tanpa petugas siaga 24 jam (biaya staf belum diketahui, PT-18), gangguan malam hari bisa melewati 4 jam sebelum ada yang menangani | (a) Tetap 4 jam, dengan jadwal siaga yang ditetapkan sebelum go-live; (b) 4 jam di jam kerja, 8 jam di luar jam kerja |
| NFR-CAP-03 100 sesi video bersamaan | Tercapai secara teknis dengan penyedia video terkelola | Sedang (biaya). Biaya per menit dan batas kapasitas penyedia `[TIDAK DIKETAHUI]` sampai Tahap 4–5 | Tetap; diverifikasi saat pemilihan penyedia |
| NFR-PERF-01/02 | Realistis | Rendah. Pemeriksaan ulang ke sistem rekanan saat checkout (AB-15) menambah waktu tunggu yang dirasakan pasien, tetapi tidak dihitung di NFR | Tetap; batas waktu tunggu ke rekanan ditetapkan di Tahap 4 |
| NFR-PERF-03 chat 2 detik | Realistis dengan koneksi real-time | Rendah | Tetap |
| **Scope rilis 1 setelah CR-006** | Rilis 1 kini memuat 2 aplikasi mobile, backoffice, portal, **aplikasi web RME minimalis**, integrasi SATUSEHAT, dan integrasi rekanan, dengan 104 hari dan 1 mobile developer | **Sangat tinggi** (R-02, R-04). Urutan pengurangan scope (§2.3) tidak mencakup RME minimalis, padahal RME minimalis adalah komponen baru yang besar | (a) Tetap, dan kelayakan dinilai di Tahap 7 dengan estimasi effort; (b) bertahap di dalam RME minimalis: data layanan, appointment/check-in, dan hasil lab dulu (agar rekanan tanpa sistem bisa bergabung), rekam medis dan pengiriman SATUSEHAT menyusul; (c) tambah kapasitas tim |
| Registrasi SATUSEHAT (US-1309) | Prasyarat di luar kendali tim: nomor PSE untuk sistem RME, nomor afiliasi SNOMED CT, kode registrasi fasyankes mitra atau surat pernyataan resmi, survei keamanan, verifikasi `[SRC-27]` | **Tinggi.** Lama prosesnya `[TIDAK DIKETAHUI]`, dan nomor PSE juga menunggu pendaftaran PSE (BR-100) | Mulai pengurusan segera setelah gate (PT-32) |
| Ketergantungan pada sistem rekanan (EP-14) | Check-in, antrean, penugasan home visit, dan hasil lab membutuhkan kemampuan sistem rekanan | Tinggi (PT-05). Rekanan dengan sistem sendiri mungkin tidak mendukung semua data | Dinilai di Tahap 4 setelah PT-05 |

## 10. Kebutuhan kepatuhan di luar perangkat lunak

Butir ini syarat go-live 5, tetapi bukan fitur aplikasi. Dicatat agar tidak hilang dari perencanaan.

| BR | Kebutuhan | Pemilik | Dibutuhkan sebelum |
| --- | --- | --- | --- |
| BR-100 | Pendaftaran PSE Lingkup Privat untuk Rumah Sehat dan untuk aplikasi RME minimalis; pendaftaran PSE sistem RME adalah prasyarat registrasi SATUSEHAT. Apakah satu pendaftaran mencakup kedua sistem ditanyakan di L-02 | CEO | Go-live; untuk RME minimalis sebelum registrasi SATUSEHAT `[SRC-27]` |
| BR-101 | Skema telemedisin dan registrasi Kemenkes bila diwajibkan (L-01) | CEO, legal | Tahap 4 |
| BR-109 | Prosedur pemberitahuan kegagalan pelindungan data 3 x 24 jam | Petugas PDP (PT-23) | Tahap 10 |
| BR-110 | DPIA, termasuk RME minimalis dan data anggota keluarga | Petugas PDP, legal | Go-live |
| BR-111 | Penunjukan pejabat/petugas PDP (PT-23) | CEO | Tahap 10 |
| BR-116 | Kewajiban PSEF dan ketentuan BPOM (L-03, PT-21) | CEO, legal | Tahap 7 |
| BR-117 | Perizinan berusaha (L-02) | CEO, legal | Tahap 12 |
| BR-118 | Pemantauan regulasi kesehatan digital | User (PM) | Setiap gate sampai go-live |
| BR-123 | Registrasi RME minimalis sebagai sistem RME mitra di SATUSEHAT (PT-32) | User (PM), CEO | Sebelum pengujian US-1309 |

## 11. Usulan penyusun (UP) untuk disetujui di gate

Usulan ini mengisi detail produk yang belum diputuskan user. Usulan ditawarkan untuk disetujui sekaligus di gate Tahap 3, dan user boleh menyebut butir yang ingin diubah.

| ID | Usulan | Alasan singkat |
| --- | --- | --- |
| UP-01 | Prioritas MoSCoW per story seperti di bagian 5 | Mengikuti syarat go-live 3, urutan T3-Q2, dan kewajiban kepatuhan |
| UP-02 | PB-02 toleransi keterlambatan pasien: nilai awal 15 menit | Disamakan dengan toleransi tenaga kesehatan (T3-Q8) |
| UP-03 | PB-06 batas outlet menerima pesanan: nilai awal 24 jam | Pasien tidak menunggu tanpa kepastian; refund otomatis RF-08 |
| UP-04 | PB-07 batas pembayaran: nilai awal 30 menit, dibatasi kemampuan payment gateway | Slot dan stok tidak tertahan terlalu lama |
| UP-05 | PB-08 pengingat 24 jam dan 1 jam sebelum jadwal | Mengurangi no-show |
| UP-06 | PB-09 peringatan SIP 30 hari sebelum berakhir | Waktu cukup untuk memperpanjang SIP |
| UP-07 | Harga layanan tampil sebagai satu angka; markup tidak ditampilkan terpisah | Sejalan dengan T2-Q20; keterbukaan menunggu L-14 |
| UP-08 | Pesanan produk tidak dapat dibatalkan pasien setelah dibayar | BR-045 tidak memberi refund, sehingga pembatalan tidak berguna dan membingungkan |
| UP-09 | Satu checkout produk hanya untuk satu outlet | Pembagian dana dan pengiriman per outlet |
| UP-10 | Status dan tabel transisi layanan, pesanan, refund, klaim, dan verifikasi (§3.2, §3.6); transisi yang tersimpan lebih dulu berlaku; satu pesanan satu refund kecuali RF-12 (AB-36) | Dasar acceptance criteria, penolakan status dari rekanan, dan audit perubahan status |
| UP-11 | Satu tenaga kesehatan melayani satu sesi pada satu waktu | Dasar kapasitas T3-Q11 dan mutu layanan |
| UP-12 | Perilaku antrean: nomor per layanan per hari, keterlambatan maksimal 30 detik, PB-10 = 3 nomor, fitur tersembunyi bila rekanan tidak mengirim antrean | Rincian BR-027 |
| UP-13 | Alur home visit: penugasan dan penugasan ulang oleh klinik, status Berangkat–Tiba–Selesai, nomor HP dan alamat pasien hanya di hari kunjungan, "pasien tidak ada di lokasi" hanya dari status Tiba, "tenaga kesehatan tidak datang" (RF-06) dan keberatan "pasien tidak hadir" (RF-17) lewat klaim, PB-03 berlaku untuk tiba di lokasi, penugasan kembali ke Terkonfirmasi bila tenaga kesehatan dinonaktifkan; Tiba ditutup otomatis menjadi Selesai di akhir hari; tanpa Tiba dan tanpa laporan sampai PB-15 habis menjadi "Ditutup tanpa konfirmasi" untuk ditindaklanjuti Layanan Pelanggan | Kehadiran di lokasi tidak tercatat otomatis seperti sesi online |
| UP-14 | Check-in dengan kode/QR di sistem rekanan; QR tetap tampil offline; otomatis "Selesai" di akhir hari setelah check-in | Mekanisme pencocokan appointment yang diserahkan BR-070 ke PRD |
| UP-15 | Pasien dapat melihat catatan konsultasi, resep, dan hasil lab miliknya dan anggota keluarganya. **Untuk anggota keluarga dewasa ini keputusan produk yang berdampak privasi**: hak akses UU PDP hanya atas data "tentang dirinya" (Pasal 7 `[SRC-15]`), sehingga kebolehannya ditanyakan ke legal (L-05, L-08). Akses pemilik akun ke data klinis anggota keluarga (dewasa maupun anak) dicatat di E-05 | Untuk diri sendiri: hak akses data (BR-106). Untuk anggota keluarga: kebutuhan pemesanan atas nama keluarga (BR-008), menunggu legal |
| UP-16 | Catatan konsultasi terstruktur, berversi, dan selaras SATUSEHAT | Siap bila L-05 menyatakan catatan konsultasi rekam medis |
| UP-17 | Matriks akses §3.1: tidak ada admin Rumah Sehat yang melihat isi klinis; foto KTP/KK hanya Verifikator; tenaga kesehatan hanya data sesi yang ditanganinya; riwayat dari tenaga kesehatan lain tidak tersedia di rilis 1 | Minimisasi akses data spesifik (BR-105) |
| UP-18 | Sub-peran admin Rumah Sehat: Super Admin, Verifikator, Layanan Pelanggan, Keuangan, Konten, Auditor, Petugas PDP | Hak minimum per fungsi backoffice |
| UP-19 | Tenaga kesehatan mendaftar sendiri di aplikasinya dengan dokumen penugasan klinik; Verifikator mencatat konfirmasi afiliasi klinik; dokumen wajib per jenis tenaga kesehatan dan per jenis rekanan dapat diatur (nilai awal STR + SIP) | Portal hanya laporan (T3-Q14); L-07 belum terjawab |
| UP-20 | Klinik yang juga outlet memakai satu akun rekanan dengan dua kemampuan | Diserahkan BRD §2.2 ke PRD |
| UP-21 | Informasi biaya MCU/lab, katalog produk, dan konten dapat dilihat tanpa masuk; pemesanan wajib masuk | Tidak ada data pribadi yang ditampilkan |
| UP-22 | Home visit hanya tampil untuk alamat di area layanan yang dikirim klinik | Mencegah pesanan yang tidak dapat dilayani |
| UP-23 | Matriks kanal notifikasi (EP-10); isi notifikasi tanpa data kesehatan rinci | Minimisasi data di kanal pihak ketiga |
| UP-24 | KPI: tenaga kesehatan "siap melayani" (AB-54); KPI-01 menghitung pemilik akun dengan layanan berstatus akhir berhasil (US-1701 AC1) | Definisi T2-Q10/Q11 perlu bentuk yang dapat dihitung sistem |
| UP-25 | Harga dan ketersediaan diperiksa ulang ke sistem rekanan sebelum pembayaran | Mencegah pasien membayar layanan atau barang yang tidak tersedia |
| UP-26 | Peran RME minimalis: Admin Fasyankes (baca rekam medis, tidak menulis), Tenaga Kesehatan Fasyankes (baca dan tulis), Petugas Lab, Operator RME (tanpa akses rekam medis); akun Tenaga Kesehatan Fasyankes dapat dihubungkan ke akun tenaga kesehatan Rumah Sehat untuk rekam medis home visit; Admin Fasyankes dapat melihat log akses rekam medis fasyankesnya | Hak minimum; fasyankes dapat memantau akses |
| UP-27 | Kelompok data rekam medis minimal: anamnesis, pemeriksaan fisik/tanda vital, diagnosis, tindakan, peresepan, permintaan lab, resume; final di Tahap 4 | Subset dari 29 kelompok data SATUSEHAT rawat jalan `[SRC-28]`, sesuai "RME minimalis" |
| UP-28 | RME minimalis mengirim rekam medis ke SATUSEHAT atas nama fasyankes pengguna | Fasyankes wajib terhubung ke SATUSEHAT `[SRC-18]`, dan user meminta kesesuaian dengan ketentuan SATUSEHAT (T3-Q1b) |
| UP-29 | Aturan sesi konsultasi: durasi kelipatan 5 menit (10–120); "bergabung" = membuka ruang sesi, dicatat server; batas ketidakhadiran = PB-02/PB-03 atau akhir durasi bila lebih dulu; Selesai hanya bila kedua pihak pernah bergabung; chat baca-saja setelah durasi habis; tidak ada perpanjangan; video dapat disambung ulang; bila keduanya tidak hadir berlaku "tenaga kesehatan tidak hadir" | Perilaku sesi yang dapat diuji |
| UP-30 | Alur tiket keluhan, penerusan ke rekanan, dibuka kembali bila pasien membalas dalam 7 hari | Rincian BR-062 dan menu keluhan portal |
| UP-31 | Profil rekanan di portal hanya dilihat; perubahan diajukan ke admin Rumah Sehat; perubahan rekening wajib disetujui Verifikator | Portal hanya laporan; mencegah pengalihan dana |
| UP-32 | Harga, markup, dan parameter dikunci saat pembayaran dibuat | Kepastian bagi pasien dan rekanan |
| UP-33 | RME minimalis tidak diberi nomor di urutan pengurangan scope; fiturnya ikut dilepas bersama layanan yang memakainya | Disampaikan setelah T3-Q2 |
| UP-34 | Pembayaran ganda (RF-12) dan pembayaran yang berhasil setelah kedaluwarsa bila slot/stok sudah tidak ada (RF-13) di-refund penuh otomatis sebagai penyebab platform: rekanan mengembalikan bagian yang benar-benar diterimanya dari pembayaran itu, Rumah Sehat menanggung semua biaya payment gateway yang tidak kembali (AB-34) | Kesalahan platform, sesuai prinsip penyebab menanggung (T3-Q7); rekanan tidak memberi layanan tambahan atas pembayaran itu |
| UP-35 | Refund gagal masuk antrean Keuangan; kewajiban pengembalian rekanan dicatat berstatus Terbuka dan tampil di laporan sampai Keuangan menandainya Selesai dengan rujukan bukti | Dana pasien tidak tertahan diam-diam; dasar rekonsiliasi PT-27 |
| UP-36 | Klaim diputuskan Layanan Pelanggan; klaim yang melewati PB-05 dieskalasi ke Super Admin yang dapat memutuskannya, tanpa disetujui atau ditolak otomatis; setelah klaim ditolak, klaim baru untuk pesanan yang sama tidak dapat diajukan | Keputusan uang tetap oleh manusia; mencegah klaim berulang |
| UP-37 | Kalender hari kerja diatur Super Admin; batas hari kerja berakhir pukul 23.59 WIB pada hari kerja ke-N setelah hari kejadian | PB-05 dan PB-16 memakai satuan hari kerja |
| UP-38 | Pasien dapat menyimpan lebih dari satu alamat | Pengiriman produk dan home visit ke alamat berbeda |
| UP-39 | Perubahan nama, tanggal lahir, NIK, atau dokumen setelah disetujui mengembalikan status ke "Menunggu verifikasi" | Verifikasi tetap sah |
| UP-40 | NIK pemilik akun tidak boleh dipakai sebagai pemilik akun di akun lain; NIK anggota keluarga boleh di lebih dari satu akun | Mencegah akun ganda; anak dapat didaftarkan kedua orang tua |
| UP-41 | Pasien dengan pesanan terbayar yang belum selesai tidak dapat dinonaktifkan admin | Menghindari kasus uang yang tidak diatur |
| UP-42 | Resep bernomor unik, tidak dapat diubah, dikoreksi dengan membatalkan dan menerbitkan ulang | Jejak resep utuh |
| UP-43 | Hasil markup persen dibulatkan ke Rupiah penuh terdekat (setengah ke atas) | Rupiah tanpa desimal |
| UP-44 | Rekanan tidak tampil ke pasien sebelum markup diatur (nilai 0 boleh) | Harga selalu lengkap sebelum dijual |
| UP-45 | Laporan portal rekanan tidak menampilkan markup Rumah Sehat dan total bayar pasien | Rekanan menerima harga rekanan; informasi markup milik Rumah Sehat |
| UP-46 | Sinkronisasi data rekanan paling lambat 15 menit; peringatan bila integrasi gagal 30 menit | Data cukup mutakhir; harga tetap dijaga AB-15 |
| UP-47 | Hak subjek data: target internal 3 x 24 jam untuk semua jenis permintaan; tanda mendesak 12 jam sebelum batas; tautan salinan data berlaku 7 hari; menarik persetujuan membatalkan pesanan aktif dengan aturan pembatalan pasien | Batas UU PDP untuk sebagian jenis `[SRC-15]`; perlakuan seragam |
| UP-48 | Target performa web dan ketersediaan juga berlaku untuk backoffice dan RME minimalis | T3-Q12/Q13 menyebut layanan inti dan portal |
| UP-49 | Perjanjian penggunaan RME minimalis dan pemrosesan data dengan setiap fasyankes wajib tercatat sebelum fasyankes aktif (US-1301 AC1) | Review kepatuhan C3-N02: Pasal 18 ayat (2) huruf a (perjanjian antar-pengendali) atau Pasal 51 ayat (1) dan (5) (pemrosesan berdasarkan perintah pengendali; persetujuan tertulis untuk prosesor lain) `[SRC-15]`; isi perjanjian menunggu L-05/L-15 |
| UP-50 | Fasyankes yang berhenti memakai RME minimalis dapat mengekspor rekam medisnya sebelum akses ditutup (US-1301 AC4) | Review kepatuhan C3-N02: rekam medis milik fasyankes tidak tertahan di sistem PT. Arunika |
| UP-51 | Markup dihitung per unit atas harga rekanan per unit; markup persen dibulatkan per unit lalu dikalikan jumlah; markup nominal berlaku per unit; markup tidak dikenakan pada ongkos kirim dan biaya transaksi (AB-11a) | Review QA QA-B03: tanpa basis yang jelas, dua perhitungan memberi total berbeda |
| UP-52 | Kejadian uang tambahan: RF-14 nominal notifikasi tidak sama (Keuangan memeriksa dan membuat refund seluruh nominal dalam PB-16 = 1 hari kerja; bila lewat, refund otomatis); RF-15 outlet tidak menyerahkan barang ke kurir dalam PB-17 = 2 x 24 jam (refund otomatis); RF-16 barang tidak sampai (klaim tanpa wajib foto setelah PB-18 = 7 hari tanpa kabar kurir, atau kurir menyatakan terkirim tetapi tidak diterima) | Review QA QA-B05: kejadian ini tidak punya hasil di matriks |
| UP-53 | PB-15 batas pengajuan klaim layanan terjadwal (RF-06, RF-07, RF-17): 2 x 24 jam setelah jadwal berakhir; keberatan "pasien tidak hadir" home visit menjadi RF-17 dan bila disetujui diperlakukan seperti RF-06 | Review QA QA-B07; disamakan dengan PB-04 |
| UP-54 | Efek menarik persetujuan utama, pembatasan, dan keberatan yang disetujui: akun hanya dapat melihat riwayat, mengajukan hak subjek data, dan memberi persetujuan kembali; pemesanan baru ditolak; data tidak dikirim ke pihak luar kecuali untuk menuntaskan pesanan produk yang sudah dibayar dan kewajiban simpan; butir tambahan hanya menghentikan tujuan butir itu | Review QA QA-B17, QA-B18: "pemrosesan dihentikan" harus teramati |
| UP-55 | PB-19: refund berstatus Diproses lebih dari 2 x 24 jam tanpa konfirmasi penyedia menjadi "Perlu dicek" dan tidak dieksekusi ulang sebelum status penyedia diketahui | Review QA QA-B10: mencegah refund tertahan atau ganda |
| UP-56 | RF-18: refund ke pasien otomatis saat jadwal mulai; penyebab (rekanan atau platform) ditetapkan Keuangan dari log integrasi dalam PB-20 = 3 hari kerja; bila tidak ditetapkan, berlaku penyebab platform | Review QA QA-B19: pasien tidak menunggu penetapan penyebab; Rumah Sehat sebagai operator integrasi menanggung bila penyebab tidak dapat dipastikan |

## 12. Ketergantungan, konfirmasi legal, dan pertanyaan terbuka

### 12.1 Status pertanyaan terbuka yang bertenggat Tahap 3

| # | Status | Jawaban |
| --- | --- | --- |
| PT-08 | Terjawab | Aplikasi tenaga kesehatan Android dan iOS `[USER] T3-Q9` |
| PT-10 | Terjawab sebagai target desain | 30.000 pasien aktif bulanan, 1.000 pengguna bersamaan, 100 sesi bersamaan `[USER] T3-Q11`. Volume riil tetap tidak diketahui |
| PT-12 | Terjawab | 99,0% per bulan `[USER] T3-Q12` |
| PT-13 | Terjawab | RME minimalis (aplikasi terpisah) dan antrean real-time masuk rilis 1; RME lengkap, loyalitas/poin/voucher, dan multi-bahasa tidak `[USER] T3-Q1, T3-Q1a, T3-Q1b, T3-Q1c, T3-Q15` (CR-006) |
| PT-15 | Terjawab | Ikut lokasi layanan `[USER] T3-Q10` |
| PT-26 | Terjawab | Pasien dapat memesankan layanan untuk anggota keluarga; verifikasi KTP (dewasa) atau KK (anak) `[USER] T3-Q3, T3-Q3a` |
| PT-27 | Terjawab untuk aturan bisnis | Penyebab menanggung (§3.5) `[USER] T3-Q7`. Mekanisme penarikan dana dari rekanan di Tahap 4–5 |

### 12.2 Pertanyaan terbuka baru

Pemilik dan tenggat adalah usulan penyusun yang dimintakan persetujuan di gate.

| # | Pertanyaan | Pemilik | Tenggat |
| --- | --- | --- | --- |
| PT-29 | Siapa menanggung biaya payment gateway yang tidak kembali dan nilai barang bila kegagalan atau kerusakan disebabkan kurir (RF-09, RF-10)? | CEO | Tahap 7 (sebelum pengembangan EP-07) |
| PT-30 | Apakah barang wajib dikembalikan ke outlet saat klaim barang rusak atau tidak sesuai disetujui, dan siapa menanggung ongkos kirim baliknya? | CEO | Tahap 7 |
| PT-31 | Ke mana dana ongkos kirim disalurkan (outlet atau kurir) dan bagaimana dalam pembagian dana payment gateway? | User (PM), CTO; konfirmasi L-04 | Tahap 4–5 |
| PT-32 | Pengurusan registrasi RME minimalis sebagai sistem RME mitra di SATUSEHAT (nomor PSE sistem RME, nomor afiliasi SNOMED CT, kode registrasi fasyankes mitra atau surat pernyataan resmi bila belum bermitra, survei keamanan) dan cara fasyankes pengguna memperoleh kredensial SATUSEHAT `[SRC-27]` | User (PM), CEO | Dimulai setelah gate Tahap 3; status dilaporkan di gate Tahap 4 |
| PT-33 | Bila klaim barang hanya menyangkut sebagian isi pesanan (satu dari beberapa barang rusak atau kurang), apakah refund tetap seluruh pembayaran atau sebanding dengan barang yang diklaim? | CEO | Tahap 7 (sebelum pengembangan EP-07) |

### 12.3 Konfirmasi legal yang berdampak ke PRD

| # | Dampak ke PRD | Status |
| --- | --- | --- |
| L-05 (meluas, CR-006) | Status catatan konsultasi sebagai rekam medis (US-308); kewajiban PT. Arunika sebagai penyedia sistem RME; pengendali bersama atau prosesor untuk data RME minimalis dan catatan konsultasi, beserta isi perjanjiannya, persetujuan tertulis sub-prosesor, serah terima rekam medis saat kerja sama berakhir, dan jalur hak subjek atas data RME; kecukupan kelompok data minimal (UP-27); dasar dan kewajiban pengiriman ke SATUSEHAT (US-1309); posisi RME yang hanya mencatat pasien Rumah Sehat terhadap kewajiban RME fasyankes untuk pasien lain; validasi hasil lab (US-1307); resep elektronik (US-309); akses pemilik akun ke data klinis anggota keluarga dewasa dari sisi rahasia kedokteran (UP-15) | Menunggu tim legal (PT-20); dibutuhkan sebelum Tahap 4 |
| L-07 | Dokumen izin wajib per jenis tenaga kesehatan (PB-14) dan kanal verifikasi STR/SIP (US-203) | Tenggat BRD "Tahap 3" belum terpenuhi karena legal belum ditunjuk (PT-20). PRD memakai dokumen wajib yang dapat diatur, nilai awal STR + SIP |
| L-08 (meluas) | Pernyataan persetujuan anggota keluarga dewasa oleh pemilik akun terhadap kewajiban bukti persetujuan dari subjek (Pasal 24) dan persetujuan orang tua/wali lewat KK (AB-04); akses pemilik akun ke data kesehatan anggota keluarga dewasa (UP-15); permintaan hak subjek atas nama anggota keluarga dewasa (US-1602); batas usia anak dan perlakuan saat anak menjadi dewasa; satu NIK di beberapa akun (UP-40); minimisasi foto KK; pihak yang wajib DPIA atas data RME; butir persetujuan per jenis data dan apakah perlu menyebut penerima data (US-104); dasar pengiriman data ke rekanan; data minimal ke payment gateway dan rekanan (AB-75) | Menunggu tim legal |
| L-14 | Aturan refund §3.5 dan penyajian harga (AB-13, AB-16) terhadap perlindungan konsumen | Tenggat BRD "Tahap 3" belum terpenuhi (PT-20). Aturan refund ditetapkan user; kalau legal meminta perubahan, dicatat sebagai change request |
| L-04, L-12 | Mekanisme refund dan penarikan dana rekanan; biaya transaksi per metode; *merchant of record* di bukti bayar | Menunggu tim legal; sebelum Tahap 4 |
| L-02 (meluas) | Pendaftaran PSE untuk aplikasi RME minimalis (apakah satu pendaftaran cukup untuk dua sistem) dan perizinan berusaha PT. Arunika sebagai penyedia sistem RME. Bagian PSE diusulkan dimajukan ke Tahap 4, karena menjadi prasyarat PT-32 | Usulan review kepatuhan, dimintakan persetujuan di gate |
| L-15 (usulan baru) | Perjanjian pemrosesan data antara PT. Arunika dan setiap fasyankes pengguna RME minimalis, serta antara PT. Arunika dan klinik rekanan untuk data konsultasi (UP-49) | Usulan review kepatuhan, dimintakan persetujuan di gate; sebelum Tahap 4 |

### 12.4 Ketergantungan lain

PT-05 (kemampuan sistem rekanan), PT-17 (markup dan biaya rekanan), PT-22 (lokasi data penyedia pihak ketiga), PT-24 (retensi; memblokir US-1605 dan US-1606), dan PT-28 (pajak) tetap terbuka sesuai BRD §10.

## 13. Keterlacakan

Pemetaan lengkap BR → story ada di `docs/traceability.md`. Setiap story di PRD ini punya minimal satu induk BR. Kebutuhan kepatuhan non-perangkat lunak dipetakan ke §10.
