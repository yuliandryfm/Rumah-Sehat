# Project Brief: Rumah Sehat

Tanggal: 2026-11-17
Jalur intake: B (wawancara)
Tingkat ketelitian: T2 Standar (dokumen perencanaan; menyentuh data pribadi/kesehatan, sehingga fakta yang terkait diperiksa substantif)

Setiap butir diberi label sumber. `[USER]` merujuk ke jawaban wawancara di `docs/00-catatan-intake.md`; kode di belakangnya (misalnya `P1.1`) adalah nomor pertanyaan di file itu. Butir yang belum terjawab ditulis `[TIDAK DIKETAHUI]` dan masuk daftar pertanyaan terbuka.

## Ringkasan

Rumah Sehat adalah aplikasi mobile gaya hidup untuk kesehatan, sejenis Halodoc `[USER] P1.1`. Aplikasi ini menghubungkan pasien dengan konsultasi online dokter dan psikolog, pendaftaran ke klinik dan lab rekanan, home visit tenaga medis, informasi biaya MCU dan lab, serta pembelian obat bebas, produk kecantikan, dan produk kesehatan dari outlet rekanan `[USER] P1.1, P1.2, P2.1, PS1`. Rumah Sehat berperan sebagai gateway: data layanan klinik, lab, home visit, dan produk berasal dari integrasi dengan sistem rekanan, sedangkan konsultasi online berlangsung di Rumah Sehat sendiri `[USER] PS6, PS6a, PS6b`. Sasaran utamanya pasien yang kesulitan datang ke klinik atau apotek, sekaligus membuka pasar yang lebih luas bagi klinik dan lab rekanan `[USER] P1.2`.

## Masalah dan tujuan

- **Masalah:**
    - Pasien yang punya kendala untuk datang ke klinik atau apotek membutuhkan konsultasi online atau home visit dari tenaga medis `[USER] P1.2`.
    - Klinik dan lab rekanan membutuhkan perluasan pasar ke pengguna aplikasi `[USER] P1.2`.
    - Calon pasien ingin mengetahui jumlah biaya sebelum melakukan MCU atau pemeriksaan lab `[USER] P1.2`.
- **Kondisi saat ini:** belum diketahui pasti. Sudah ada aplikasi sejenis (Halodoc), dan sebagian orang menghubungi klinik langsung lewat telepon atau WhatsApp `[USER] P1.3`.
- **Dampak kalau tidak diselesaikan:** pasien kesulitan mendapat layanan atau informasi kesehatan, klinik rekanan sulit berekspansi, dan belum ada *one stop solution* untuk pasien yang mobilitasnya terbatas `[USER] P1.4`.
- **Tujuan terukur:** 10.000 user aktif, 70 dokter, 20 klinik rekanan, dan 10 lab rekanan dalam 12 bulan setelah go-live (sekitar Maret 2028) `[USER] P8.1, P8.1a`. Definisi "user aktif" `[TIDAK DIKETAHUI]` (PT-09).

## Pengguna

Sembilan peran di rilis pertama `[USER] P2.1, P3.5, PS3, PS6c, PS6d`:

| Peran | Kebutuhan utama | Platform | Sumber |
| --- | --- | --- | --- |
| Pasien | Konsultasi online, daftar klinik/lab, home visit, info biaya MCU/lab, beli produk | Mobile Android dan iOS | `[USER] P1.2, P6.1` |
| Dokter | Konsultasi online (chat dan video), catatan, resep digital | Aplikasi mobile tenaga kesehatan | `[USER] PS6b, P3.4, P6.2` |
| Psikolog | Konsultasi kesehatan mental (chat dan video) | Aplikasi mobile tenaga kesehatan | `[USER] P2.1, PS6b, P6.2` |
| Tenaga medis | Tugas rinci `[TIDAK DIKETAHUI]` (PT-07); home visit disebut sebagai layanan tenaga medis | Aplikasi mobile tenaga kesehatan | `[USER] P1.2, P6.2` |
| Fisioterapis | Tugas rinci `[TIDAK DIKETAHUI]` (PT-07) | Aplikasi mobile tenaga kesehatan | `[USER] P2.1, P6.2` |
| Admin klinik | Portal di Rumah Sehat (contoh: laporan transaksi dan penerusan dana, profil, keluhan); data layanan tetap lewat integrasi | Portal web | `[USER] PS6c, P6.2` |
| Admin lab | Sama seperti admin klinik, untuk lab rekanan | Portal web | `[USER] PS6c, P6.2` |
| Admin outlet | Portal untuk toko kecantikan dan toko alat kesehatan (laporan, penerusan dana, profil, keluhan); katalog dan pesanan lewat integrasi | Portal web | `[USER] PS6d, P6.2` |
| Admin Rumah Sehat | Backoffice: verifikasi dan kelola akun, verifikasi KTP manual, laporan dan penerusan dana, CMS konten, layanan pelanggan | Portal web | `[USER] P4.6b, P8.2, P8.2a, P6.2` |

Asal dokter dan psikolog: ada yang mitra langsung Rumah Sehat, ada yang tenaga klinik rekanan `[USER] PS6e`.
Apoteker tidak dipakai di rilis pertama, menunggu rilis berikutnya `[USER] P3.5, PS3`.

## Stakeholder dan sign-off

- **Pemilik dan pembiaya:** dibiayai sendiri `[USER] P2.2`. Rumah Sehat adalah produk milik perusahaan IT yang sudah berbadan hukum, bukan bagian grup klinik atau apotek `[USER] P2.2, P5.2a`. Nama perusahaan `[TIDAK DIKETAHUI]` (PT-01).
- **Penyetuju:** CEO, CTO, dan konsultan medis `[USER] P2.3, P2.3a`.
- **Alur persetujuan:** user adalah project manager yang meneruskan dokumen gate ke CEO dan CTO. Persetujuan user di gate runbook dianggap persetujuan CEO dan CTO yang diteruskan `[USER] P2.4a, Konfirmasi P2`.
- **Konsultan medis:** dilibatkan hanya saat UAT dan go-live `[USER] P2.4b`.

## Scope awal

### In-scope rilis pertama

Tujuh layanan `[USER] P3.1, Perubahan scope 1, Konfirmasi PS`:

1. Konsultasi online dengan dokter lewat chat teks dan panggilan video `[USER] P3.1, P3.2`
2. Konsultasi kesehatan mental dengan psikolog lewat chat teks dan panggilan video `[USER] P2.1, P3.1, P3.2`
3. Pendaftaran ke klinik rekanan `[USER] P3.1`
4. Pendaftaran lab rekanan `[USER] P3.1`
5. Home visit tenaga medis `[USER] P1.2, P3.1`
6. Informasi biaya MCU dan pemeriksaan lab `[USER] P1.2, P3.1`
7. Pembelian obat bebas, produk kecantikan, dan produk kesehatan lain (vitamin, suplemen, alat kesehatan) dari outlet rekanan `[USER] PS1`

Ketentuan pendukung:

- **Resep:** dokter menerbitkan resep digital di aplikasi, lalu pasien menebusnya sendiri di apotek mana pun. Obat resep tidak dijual di Rumah Sehat `[USER] P3.4, PS1`.
- **Outlet rekanan:** toko kecantikan, toko alat kesehatan, dan klinik rekanan. Obat bebas dijual oleh klinik rekanan. Apotek tidak termasuk outlet rekanan `[USER] PS2, PS2a`.
- **Pembayaran:** konsultasi online dan pembelian produk dibayar di aplikasi, lalu dana produk diteruskan ke outlet. Klinik, lab, dan home visit dibayar langsung ke mitra `[USER] P3.3, PS4`.
- **Pengiriman produk:** kurir pihak ketiga yang terintegrasi dengan aplikasi `[USER] PS5`.
- **Prinsip gateway:** jadwal dan data klinik, jadwal dan harga lab, home visit, serta katalog, stok, dan pesanan produk diambil dari sistem rekanan lewat integrasi. Rumah Sehat tidak menyimpan katalog sendiri `[USER] PS6, PS6a`.
- **Konsultasi online** berlangsung di Rumah Sehat (aplikasi tenaga kesehatan untuk chat, video, catatan, dan resep) `[USER] PS6b`.
- **Backoffice admin Rumah Sehat:** verifikasi dan kelola akun (pasien, tenaga kesehatan, rekanan), laporan transaksi dan penerusan dana, CMS konten (banner, halaman info, artikel, promo), dan layanan pelanggan `[USER] P8.2, P8.2a`.
- **Konten edukasi kesehatan** masuk rilis pertama lewat CMS `[USER] P8.2a`.
- **Integrasi otomatis ke sistem klinik dan lab rekanan** masuk rilis pertama `[USER] PS6a`.

### Out-of-scope rilis pertama

| Butir | Sumber |
| --- | --- |
| Integrasi BPJS | `[USER] P3.6` |
| Integrasi asuransi swasta | `[USER] P3.6` |
| Penjualan obat resep | `[USER] PS1` |
| Peran apoteker | `[USER] P3.5, PS3` |
| Konsultasi lewat panggilan suara | `[USER] P3.2` |
| Langganan atau membership berbayar | `[USER] P3.6a` |
| Chatbot AI atau pemeriksa gejala otomatis | `[USER] P3.6a` |
| Versi web untuk pasien | `[USER] P3.6a` |
| Integrasi perangkat kesehatan atau wearable | `[USER] P3.6a` |
| Pengantaran sampel lab atau pengambilan sampel di rumah | `[USER] P3.6a` |
| Rekaman panggilan video (tidak disimpan) | `[USER] P4.1` |

### Belum diputuskan (dibahas di Tahap 3)

Butir berikut tidak otomatis masuk scope `[USER] P3.6a`:

- Rekam medis elektronik (RME) lengkap untuk klinik dan lab
- Program loyalitas, poin, dan voucher
- Antrean real-time di klinik
- Multi-bahasa (selain Bahasa Indonesia)

## Data dan integrasi

- **Data pasien yang disimpan** `[USER] P4.1`: identitas dasar (nama, tanggal lahir, jenis kelamin, nomor HP, email, alamat), NIK dan foto KTP, riwayat dan isi chat konsultasi termasuk foto, catatan dokter dan psikolog, resep digital, hasil lab dari lab rekanan, riwayat pendaftaran klinik, lab, dan home visit, data anggota keluarga, serta riwayat pembayaran dan pesanan produk.
- **Klasifikasi sensitivitas data:** `[TIDAK DIKETAHUI]`. User belum menyatakan data mana yang tergolong sensitif, dan kategori hukum serta kewajibannya diteliti oleh compliance reviewer di Tahap 2 (PT-03). Usulan penyusun untuk diperiksa di Tahap 2 (bukan fakta): NIK dan foto KTP, catatan dokter dan psikolog (termasuk kesehatan mental), resep, hasil lab, data anggota keluarga, dan data pembayaran.
- **NIK/KTP:** wajib untuk konsultasi online, home visit, serta pendaftaran klinik dan lab. Tidak wajib untuk pembelian produk. Diverifikasi manual oleh admin Rumah Sehat sebelum layanan dipakai `[USER] P4.6a, P4.6b`.
- **Login pasien:** email dan password, atau akun Google/Apple. Tanpa login nomor HP dan OTP `[USER] P4.4`.
- **Notifikasi ke pasien:** push, email, WhatsApp. Tanpa SMS `[USER] P4.5`.
- **Sistem yang harus terhubung** `[USER] PS4, PS5, PS6a, P3.2, P4.4, P4.5`:
    - Sistem klinik, lab, toko kecantikan, dan toko alat kesehatan rekanan (jadwal, harga, home visit, katalog, stok, pesanan, hasil lab)
    - Payment gateway, termasuk penerusan dana ke mitra
    - Layanan panggilan video
    - Kurir pihak ketiga
    - WhatsApp, email, dan notifikasi push
    - Login Google dan Apple
- **Penyedia pihak ketiga:** belum ada yang dipilih atau dikontrak. Pemilihan dilakukan di Tahap 4–5 dan dibawa ke user untuk disetujui `[USER] P4.2`.
- **Rekanan:** belum ada rekanan yang bergabung, dicari paralel dengan pembangunan aplikasi. Kemampuan integrasi sistem rekanan `[TIDAK DIKETAHUI]` `[USER] P4.3` (PT-05).

## Kepatuhan

- **Regulasi atau standar:** `[TIDAK DIKETAHUI]`. Diteliti dari sumber resmi oleh compliance reviewer di Tahap 2 `[USER] P5.1` (PT-03).
- **Badan usaha dan izin:** Rumah Sehat dimiliki perusahaan IT yang sudah berbadan hukum `[USER] P5.2a`. Apakah dibutuhkan badan usaha terpisah, serta izin apa saja yang diperlukan dan status kepemilikannya, `[TIDAK DIKETAHUI]` `[USER] P5.2` (PT-02).
- **Kebolehan penjualan obat bebas oleh klinik rekanan secara online:** `[TIDAK DIKETAHUI]`, diteliti di Tahap 2 `[USER] PS2a` (PT-04).
- **Kebutuhan audit trail:** kesepuluh aktivitas berikut ditambah setiap transaksi yang terjadi `[USER] P5.4, Konfirmasi P5`:
    1. Login, logout, dan percobaan login gagal
    2. Akses ke data pasien (catatan medis, hasil lab, KTP)
    3. Pembuatan dan perubahan catatan dokter atau psikolog, serta resep
    4. Verifikasi KTP oleh admin (disetujui atau ditolak)
    5. Pembayaran, refund, dan penerusan dana ke mitra
    6. Perubahan status pendaftaran, home visit, dan pesanan
    7. Penambahan, perubahan, dan penonaktifan akun tenaga kesehatan dan admin mitra
    8. Perubahan hak akses atau peran
    9. Ekspor atau unduh data
    10. Permintaan pasien untuk menghapus atau melihat datanya
- **Pihak auditor:** auditor eksternal, serta regulator atau instansi pemerintah `[USER] P5.3`. Nama auditor, standar sertifikasi, dan instansinya `[TIDAK DIKETAHUI]` (PT-03).
- **Lokasi data:** wajib di server atau cloud yang berlokasi di Indonesia `[USER] P5.5`.

## Kebutuhan non-fungsional

- **Platform:** aplikasi pasien di Android dan iOS sekaligus `[USER] P6.1`. Tenaga kesehatan memakai aplikasi mobile, para admin memakai portal web `[USER] P6.2`. Platform aplikasi tenaga kesehatan (Android, iOS, atau keduanya) `[TIDAK DIKETAHUI]` (PT-08).
- **Ketersediaan:** 24 jam, 7 hari. Ketersediaan dokter mengikuti jadwal masing-masing `[USER] P6.4`. Target dalam angka `[TIDAK DIKETAHUI]`, dibahas di Tahap 3.
- **Wilayah:** seluruh Indonesia, di mana pun ada rekanan `[USER] P6.5`.
- **Volume pengguna dan transaksi:** `[TIDAK DIKETAHUI]` `[USER] P6.3` (PT-10).

## Batasan

- **Anggaran:** `[TIDAK DIKETAHUI]` `[USER] P7.1`. Estimasi disusun di Tahap 1 (PT-11).
- **Tenggat:** go-live 1 Maret 2027 `[USER] P7.2`.
- **Tim:** project manager (user), Claude lewat runbook ini, dan 1 mobile developer `[USER] P7.3`.
- **Teknologi wajib:** Flutter untuk aplikasi mobile pasien dan tenaga kesehatan `[USER] P7.4, Konfirmasi P7`. Teknologi lain (backend, portal web, cloud) tidak dibatasi selain lokasi data di Indonesia, dan dipilih di Tahap 5 dengan persetujuan user `[USER] P5.5, Konfirmasi P7`.
- **Teknologi dilarang:** tidak disebut `[USER] P7.4`.

## KPI dan go-live

- **Ukuran sukses (12 bulan setelah go-live):** 10.000 user aktif, 70 dokter, 20 klinik, dan 10 lab `[USER] P8.1, P8.1a`.
- **Syarat rilis pertama (1 Maret 2027)** `[USER] P8.2`:
    1. Minimal 2 klinik, 1 lab, dan 3 outlet rekanan terintegrasi
    2. Minimal 5 dokter dan 1 psikolog terverifikasi dan siap melayani
    3. Ketujuh layanan berjalan
    4. Aplikasi tayang di Play Store dan App Store
    5. Izin dan kepatuhan regulasi (hasil Tahap 2) terpenuhi
    6. Lulus audit keamanan
    7. Sign-off UAT dari CEO, CTO, dan konsultan medis
- **Kalau syarat belum terpenuhi:** tanggal tetap dan scope dikurangi (layanan yang belum siap menyusul). Syarat 5, 6, dan 7 tidak bisa dikurangi `[USER] P8.3, Konfirmasi P8`.

## Risiko yang dicatat untuk dinilai di Tahap 1

Risiko ini adalah catatan pewawancara. Fakta yang mendasarinya bersumber dari jawaban user, sedangkan penilaian kemungkinan dan dampaknya dilakukan di Tahap 1.

| # | Risiko | Dasar |
| --- | --- | --- |
| R-01 | Prinsip gateway bergantung pada sistem rekanan yang belum ada dan belum diketahui kemampuan integrasinya. Tanpa rekanan terintegrasi, layanan klinik, lab, home visit, dan produk tidak bisa tampil | `[USER] PS6a, P4.3, P8.2` |
| R-02 | Jadwal sekitar 3,5 bulan (2026-11-17 sampai 2027-03-01) untuk 7 layanan, 9 peran, aplikasi mobile pasien (Android dan iOS) dan aplikasi mobile tenaga kesehatan (platform belum diketahui, PT-08), portal web, dan banyak integrasi | `[USER] P7.2, P7.3, P6.1, P6.2` |
| R-03 | Kebutuhan badan usaha terpisah dan izin belum diketahui, padahal izin adalah syarat go-live yang tidak bisa dikurangi | `[USER] P5.2, P8.2, P8.3` |
| R-04 | Kapasitas tim (PM, Claude, 1 mobile developer) terhadap scope | `[USER] P7.3` |

## Asumsi

Tidak ada. Semua butir di brief ini bersumber dari jawaban user atau ditulis `[TIDAK DIKETAHUI]`.

## Pertanyaan terbuka

Kolom Pemilik dan Tenggat adalah usulan penyusun, disesuaikan dengan tahap yang membutuhkan jawabannya, dan dikonfirmasi user di gate Tahap 0.

| # | Pertanyaan | Pemilik | Tenggat |
| --- | --- | --- | --- |
| PT-01 | Nama badan hukum perusahaan IT pemilik Rumah Sehat | User (PM), konfirmasi ke CEO | Sebelum gate Tahap 1 |
| PT-02 | Apakah layanan Rumah Sehat harus dijalankan oleh badan usaha terpisah, atau boleh oleh perusahaan IT yang ada? Izin apa saja yang dibutuhkan, dan bagaimana statusnya? (prioritas tinggi) | Compliance reviewer (riset sumber resmi), keputusan oleh CEO | Tahap 2. Bisa dimajukan ke Tahap 1 kalau memengaruhi Business Case |
| PT-03 | Regulasi dan standar yang berlaku (layanan kesehatan digital, data pribadi dan kesehatan, lokasi data), beserta instansi pengawas dan auditornya | Compliance reviewer, disetujui user | Tahap 2 |
| PT-04 | Apakah penjualan obat bebas oleh klinik rekanan lewat aplikasi diperbolehkan regulasi? | Compliance reviewer, keputusan oleh CEO | Tahap 2 |
| PT-05 | Rekanan pertama (minimal 2 klinik, 1 lab, 3 outlet), sistem yang mereka pakai, dan kesiapan integrasinya | User (PM) dan CEO | Sebelum Tahap 4 (arsitektur integrasi) |
| PT-06 | Mekanisme dan jadwal penerusan dana ke outlet dan mitra, serta pembayaran ke dokter dan psikolog mitra langsung | CEO | Tahap 2 |
| PT-07 | Tugas fisioterapis dan tenaga medis (konsultasi online, home visit, atau keduanya) dan asal mereka (mitra langsung atau dari klinik rekanan) | User (PM), konfirmasi ke CEO | Tahap 2 |
| PT-08 | Platform aplikasi tenaga kesehatan: Android, iOS, atau keduanya? | User (PM) dan CTO | Tahap 3 |
| PT-09 | Definisi "user aktif" (harian atau bulanan), dan apakah 70 dokter sudah termasuk psikolog | CEO | Tahap 1 |
| PT-10 | Perkiraan volume pengguna dan transaksi (pasien, konsultasi per hari, pesanan) | CEO dan CTO | Tahap 3 (NFR) |
| PT-11 | Anggaran pembangunan sampai rilis pertama, dan apa saja yang termasuk di dalamnya | CEO | Tahap 1 |
| PT-12 | Target ketersediaan dalam angka (misalnya persen uptime) | CTO | Tahap 3 (NFR) |
| PT-13 | Butir yang belum diputuskan: RME lengkap, loyalitas/poin/voucher, antrean real-time, multi-bahasa | User (PM), keputusan oleh CEO | Tahap 3 |
| PT-14 | Alasan pemilihan tanggal go-live 1 Maret 2027 (misalnya investor, pemasaran, kontrak) | User (PM) | Tahap 1 |
| PT-15 | Zona waktu acuan untuk jadwal konsultasi, klinik, lab, dan home visit, karena layanan mencakup seluruh Indonesia | User (PM) dan CTO | Tahap 3 (NFR) |
