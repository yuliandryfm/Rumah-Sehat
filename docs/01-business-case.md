# Business Case: Rumah Sehat

Tanggal: 2026-11-17
Versi: 1.0 (disetujui di gate Tahap 1, 2026-11-17)
Penyusun: Claude (persona Project Manager + Business Analyst)
Pemilik dan pemutus: Executive, yaitu user (project manager yang meneruskan ke CEO dan CTO) `[DOK: 00-project-brief.md]`
Tingkat ketelitian: T2 Standar (dokumen perencanaan; angka biaya dan KPI diperiksa substantif)

Sumber: `[DOK]` merujuk ke `docs/00-project-brief.md` (lulus gate Tahap 0) beserta kode pertanyaan intake di `docs/00-catatan-intake.md` (misalnya `P8.1`). `[USER] T1-Qx` merujuk ke wawancara Tahap 1 di `docs/01-catatan-business-case.md`. Semua angka biaya adalah **estimasi** dan dasar hitungnya ditulis di sampingnya.

Sesuai prinsip PRINCE2 *continued business justification*, dokumen ini dicek ulang di gate besar berikutnya (lihat bagian 9).

## 1. Ringkasan untuk Executive

- **Apa:** aplikasi mobile kesehatan (Flutter, Android dan iOS) dengan tujuh layanan. Rumah Sehat berperan sebagai gateway ke sistem klinik, lab, dan outlet rekanan `[DOK] P1.1, PS6, PS6a, P6.1, P7.4`.
- **Pemilik:** PT. Arunika, perusahaan IT berbadan hukum, dengan pembiayaan sendiri `[USER] T1-Q6` `[DOK] P2.2, P5.2a`.
- **Opsi terpilih:** membangun sendiri. Opsi beli/sewa solusi jadi sudah dipertimbangkan dan ditolak `[USER] T1-Q7, T1-Q7a`.
- **Manfaat terukur:** 10.000 pasien aktif bulanan, 70 dokter dan psikolog, 20 klinik rekanan, dan 10 lab rekanan pada 1 Maret 2028 `[DOK] P8.1, P8.1a` `[USER] T1-Q3`.
- **Pendapatan:** komisi konsultasi, komisi produk, dan biaya dari rekanan `[USER] T1-Q1`. Besarannya belum ditentukan, sehingga kelayakan finansial baru bisa disajikan sebagai simulasi (bagian 5).
- **Biaya yang sudah diketahui:** biaya mobile developer Rp10–20 juta per bulan `[USER] T1-Q5a` dan biaya akun Google Play US$25 sekali bayar `[SRC-01]`. Komponen biaya lain `[TIDAK DIKETAHUI]` dan dipertajam setelah Tahap 2, 4, dan 5 (bagian 6).
- **Waktu:** go-live 1 Maret 2027 untuk mengikuti rencana pemasaran `[DOK] P7.2` `[USER] T1-Q4`, yaitu 104 hari kalender sejak 2026-11-17 (perhitungan tanggal).
- **Risiko terbesar:** ketergantungan pada rekanan yang belum ada (R-01), jadwal dan kapasitas tim (R-02, R-04), serta izin yang belum diketahui (R-03). Rinciannya di bagian 7.
- **Keputusan kelayakan:** **layak dilanjutkan**, disetujui Executive di gate Tahap 1 pada 2026-11-17 `[USER] T1-Gate`.

## 2. Latar belakang dan masalah

Diambil dari Project Brief `[DOK]`:

- Pasien yang punya kendala datang ke klinik atau apotek membutuhkan konsultasi online atau home visit tenaga medis `[DOK] P1.2`.
- Klinik dan lab rekanan membutuhkan perluasan pasar ke pengguna aplikasi `[DOK] P1.2`.
- Calon pasien ingin tahu jumlah biaya sebelum MCU atau pemeriksaan lab `[DOK] P1.2`.
- Kondisi saat ini belum diketahui pasti. Sudah ada aplikasi sejenis (Halodoc), dan sebagian orang menghubungi klinik lewat telepon atau WhatsApp `[DOK] P1.3`.
- Dampak kalau tidak diselesaikan: pasien kesulitan mendapat layanan atau informasi kesehatan, klinik rekanan sulit berekspansi, dan belum ada *one stop solution* untuk pasien yang mobilitasnya terbatas `[DOK] P1.4`.

## 3. Opsi yang dipertimbangkan

| Opsi | Uraian | Biaya | Manfaat | Status | Sumber |
| --- | --- | --- | --- | --- | --- |
| A. Tidak melakukan apa-apa | Rumah Sehat tidak dibangun | Tidak ada biaya proyek | Tidak ada; masalah di bagian 2 tetap ada | Tidak dipilih | `[DOK] P1.4` |
| B. Beli atau sewa solusi jadi | Misalnya platform telemedis *white-label* yang diberi merek Rumah Sehat | `[TIDAK DIKETAHUI]` (tidak ada penawaran vendor yang tercatat) | Tidak dinilai lebih jauh | Sudah dipertimbangkan perusahaan dan ditolak | `[USER] T1-Q7` |
| C. Membangun sendiri | Aplikasi Flutter pasien dan tenaga kesehatan, portal web admin, dan backend, dibangun lewat runbook ini bersama 1 mobile developer | Bagian 6 | Bagian 4 | **Dipilih** | `[USER] T1-Q7` `[DOK] P7.3, P7.4` |

**Alasan memilih opsi C** `[USER] T1-Q7a`:

1. Kendali produk: fitur, prioritas, dan prinsip gateway ditentukan sendiri.
2. Kepemilikan kode dan data: kode dan data milik perusahaan, tanpa ketergantungan pada vendor.
3. Biaya jangka panjang: sewa atau lisensi solusi jadi dinilai lebih mahal dalam jangka panjang.
4. Kapabilitas perusahaan IT: membangun sendiri sesuai kemampuan inti PT. Arunika.

Catatan penyusun: alasan nomor 3 adalah penilaian perusahaan. Dokumen ini tidak memuat angka pembanding biaya opsi B.

## 4. Manfaat dan KPI

### 4.1 KPI utama (12 bulan setelah go-live)

Tenggat KPI 1 Maret 2028 adalah 12 bulan setelah go-live 1 Maret 2027 `[DOK] P8.1a, P7.2`.

| ID | KPI | Target | Tenggat | Cara mengukur | Sumber |
| --- | --- | --- | --- | --- | --- |
| KPI-01 | Pasien aktif bulanan (MAU) | 10.000 | 1 Maret 2028 | Jumlah pasien unik yang memakai aplikasi dalam satu bulan kalender. Kriteria "memakai" masih terbuka (PT-16), ditetapkan sebelum Tahap 3 supaya bisa diukur sistem | `[DOK] P8.1` `[USER] T1-Q3` |
| KPI-02 | Dokter dan psikolog (gabungan) | 70 | 1 Maret 2028 | Jumlah akun dokter dan psikolog di Rumah Sehat | `[DOK] P8.1` `[USER] T1-Q3` |
| KPI-03 | Klinik rekanan | 20 | 1 Maret 2028 | Jumlah klinik rekanan yang bergabung | `[DOK] P8.1` |
| KPI-04 | Lab rekanan | 10 | 1 Maret 2028 | Jumlah lab rekanan yang bergabung | `[DOK] P8.1` |

Status akun yang dihitung di KPI-02 sampai KPI-04 (misalnya "terverifikasi dan aktif" atau "terintegrasi") belum ditentukan user. Hal ini masuk PT-16 bersama definisi MAU.

### 4.2 Tonggak go-live (1 Maret 2027)

Syarat minimum rilis pertama `[DOK] P8.2`: minimal 2 klinik, 1 lab, dan 3 outlet rekanan terintegrasi; minimal 5 dokter dan 1 psikolog terverifikasi; ketujuh layanan berjalan; tayang di Play Store dan App Store; izin dan kepatuhan regulasi terpenuhi; lulus audit keamanan; serta sign-off UAT dari CEO, CTO, dan konsultan medis. Kalau ada syarat yang belum terpenuhi, tanggal tetap dan scope dikurangi, kecuali syarat izin, audit keamanan, dan UAT `[DOK] P8.3`.

### 4.3 Manfaat kualitatif

- Bagi pasien: akses konsultasi online, home visit, pendaftaran klinik dan lab, informasi biaya MCU dan lab, serta pembelian produk dalam satu aplikasi `[DOK] P1.2, P1.4`.
- Bagi klinik dan lab rekanan: perluasan pasar ke pengguna aplikasi `[DOK] P1.2`.

### 4.4 KPI finansial

Target pendapatan, titik impas, atau masa balik modal belum ditetapkan `[TIDAK DIKETAHUI]`, karena besaran komisi dan biaya rekanan belum diputuskan (PT-17). Bagian 5 menyajikan simulasinya.

## 5. Model pendapatan dan simulasi

### 5.1 Sumber pendapatan

| Sumber | Dasar | Sumber |
| --- | --- | --- |
| Komisi konsultasi | Potongan dari biaya konsultasi dokter dan psikolog yang dibayar pasien di aplikasi | `[USER] T1-Q1` `[DOK] P3.3` |
| Komisi produk | Potongan dari penjualan produk outlet sebelum dana diteruskan ke outlet | `[USER] T1-Q1` `[DOK] PS4` |
| Biaya dari rekanan | Langganan atau biaya per pendaftaran yang ditagihkan ke klinik dan lab rekanan. Bentuk yang dipakai belum dipilih | `[USER] T1-Q1` |

Pembayaran klinik, lab, dan home visit dilakukan langsung ke mitra, tidak lewat aplikasi `[DOK] P3.3`, sehingga untuk layanan itu Rumah Sehat tidak memegang dana pasien yang bisa dipotong komisi. Dari ketiga sumber di atas, yang berlaku untuk pendaftaran klinik dan lab adalah biaya dari rekanan `[USER] T1-Q1`. Klinik rekanan tetap bisa menghasilkan komisi produk sebagai penjual obat bebas `[DOK] PS2a, PS4`.

### 5.2 Simulasi

**Ini simulasi, bukan keputusan dan bukan proyeksi.** Tabel ini dibuat atas permintaan user `[USER] T1-Q2`. Persentase dan nominal di kolom parameter adalah angka bulat ilustratif yang dipilih penyusun, bukan acuan pasar `[KEPUTUSAN-01]`. Volume transaksi riil `[TIDAK DIKETAHUI]` (PT-10), sehingga simulasi dinyatakan per satuan nilai transaksi. Angka final ditetapkan CEO (PT-17).

**Tabel A. Pendapatan komisi per Rp100 juta nilai transaksi (konsultasi atau produk) per bulan**

Rumus: pendapatan = persentase komisi × nilai transaksi.

| Komisi | 5% | 10% | 15% | 20% |
| --- | --- | --- | --- | --- |
| Pendapatan per Rp100 juta transaksi | Rp5 juta | Rp10 juta | Rp15 juta | Rp20 juta |

**Tabel B. Nilai transaksi per bulan yang dibutuhkan hanya untuk menutup biaya mobile developer**

Rumus: nilai transaksi = biaya bulanan ÷ persentase komisi. Biaya bulanan mengikuti rentang `[USER] T1-Q5a`. Tabel ini **hanya menutup satu komponen biaya**. Komponen lain di bagian 6 belum bernilai, sehingga nilai impas sebenarnya lebih tinggi.

| Komisi | 5% | 10% | 15% | 20% |
| --- | --- | --- | --- | --- |
| Biaya developer Rp10 juta/bulan | Rp200 juta | Rp100 juta | ±Rp66,7 juta | Rp50 juta |
| Biaya developer Rp20 juta/bulan | Rp400 juta | Rp200 juta | ±Rp133,3 juta | Rp100 juta |

**Tabel C. Pendapatan biaya rekanan per bulan (bentuk langganan bulanan)**

Rumus: pendapatan = biaya per rekanan × jumlah klinik dan lab. Jumlah rekanan memakai angka minimum go-live (2 klinik + 1 lab = 3) `[DOK] P8.2` dan target KPI (20 klinik + 10 lab = 30) `[DOK] P8.1`. Outlet tidak dihitung karena opsi Q1 menyebut klinik dan lab `[USER] T1-Q1`.

| Biaya per rekanan per bulan | Rp250 ribu | Rp500 ribu | Rp1 juta |
| --- | --- | --- | --- |
| 3 rekanan (minimum go-live) | Rp750 ribu | Rp1,5 juta | Rp3 juta |
| 30 rekanan (target KPI) | Rp7,5 juta | Rp15 juta | Rp30 juta |

## 6. Perkiraan biaya dan waktu

### 6.1 Biaya (kasar, dipertajam setelah Tahap 2, 4, dan 5)

Anggaran total `[TIDAK DIKETAHUI]` `[DOK] P7.1` (PT-11). Periode pembangunan sampai go-live adalah 2026-11-17 sampai 2027-03-01, yaitu 104 hari atau sekitar 3,4 bulan (perhitungan tanggal).

| Komponen | Sampai go-live | 12 bulan operasi setelah go-live | Dasar hitung / sumber | Dipertajam di |
| --- | --- | --- | --- | --- |
| Mobile developer (1 orang) | Rp34–68 juta | Rp120–240 juta | Rp10–20 juta per bulan `[USER] T1-Q5a` × 3,4 bulan dan × 12 bulan. Berlaku bila developer dibayar penuh sepanjang periode. Status itu `[TIDAK DIKETAHUI]` (PT-19) | Gate Tahap 1 (PT-19) |
| Project manager (user) | `[TIDAK DIKETAHUI]` | `[TIDAK DIKETAHUI]` | Tidak disebut `[USER] T1-Q5a` (PT-18) | PT-18 |
| Langganan Claude untuk runbook | `[TIDAK DIKETAHUI]` | `[TIDAK DIKETAHUI]` | Tidak disebut (PT-18) | PT-18 |
| Akun Google Play Console | US$25 sekali bayar | — | `[SRC-01]` | — |
| Akun Apple Developer Program | `[TIDAK DIKETAHUI]` | `[TIDAK DIKETAHUI]` | Halaman resmi Apple gagal dibuka saat riset 2026-11-17 (lihat `docs/sources.md`) | Tahap 5 |
| Cloud dan hosting di Indonesia | `[TIDAK DIKETAHUI]` | `[TIDAK DIKETAHUI]` | Penyedia belum dipilih `[DOK] P4.2, P5.5` | Tahap 4–5 |
| Layanan panggilan video | `[TIDAK DIKETAHUI]` | `[TIDAK DIKETAHUI]` | Penyedia belum dipilih `[DOK] P4.2` | Tahap 4–5 |
| Payment gateway dan penerusan dana | `[TIDAK DIKETAHUI]` | `[TIDAK DIKETAHUI]` | Penyedia belum dipilih `[DOK] P4.2`. Biaya per transaksi mengurangi pendapatan komisi di bagian 5 | Tahap 4–5 |
| WhatsApp, email, notifikasi push | `[TIDAK DIKETAHUI]` | `[TIDAK DIKETAHUI]` | Penyedia belum dipilih `[DOK] P4.2, P4.5` | Tahap 4–5 |
| Kurir pihak ketiga | `[TIDAK DIKETAHUI]` | `[TIDAK DIKETAHUI]` | Penyedia dan siapa yang menanggung ongkir belum diketahui `[DOK] PS5` | Tahap 2–5 |
| Audit keamanan eksternal | `[TIDAK DIKETAHUI]` | `[TIDAK DIKETAHUI]` | Syarat go-live 6 `[DOK] P8.2`; auditor eksternal `[DOK] P5.3` | Tahap 10 |
| Perizinan, legal, dan kepatuhan | `[TIDAK DIKETAHUI]` | `[TIDAK DIKETAHUI]` | Izin belum diketahui `[DOK] P5.2` (PT-02, PT-03) | Tahap 2 |
| Staf operasional admin Rumah Sehat (verifikasi KTP manual, layanan pelanggan, CMS) | `[TIDAK DIKETAHUI]` | `[TIDAK DIKETAHUI]` | Fungsi wajib `[DOK] P4.6b, P8.2a`; jumlah staf tidak disebut (PT-18) | PT-18 |
| Pemasaran | `[TIDAK DIKETAHUI]` | `[TIDAK DIKETAHUI]` | Ada rencana pemasaran `[USER] T1-Q4`; apakah masuk anggaran proyek `[TIDAK DIKETAHUI]` (PT-11) | PT-11 |

**Total yang sudah bernilai:** Rp34–68 juta ditambah US$25 sampai go-live, dan Rp120–240 juta untuk 12 bulan operasi. Total ini **bukan total biaya proyek**, karena sebagian besar komponen belum bernilai.

**Toleransi biaya:** +20% dari perkiraan biaya terakhir yang disetujui `[USER] T1-Q8`. Karena perkiraan saat ini belum lengkap, baseline biaya yang menjadi acuan toleransi baru bisa ditetapkan setelah komponen pihak ketiga dihitung di Tahap 4–5 (lihat bagian 9).

### 6.2 Waktu

- Go-live tetap 1 Maret 2027 `[DOK] P7.2, P8.3`. Tanggal ini terkait rencana pemasaran `[USER] T1-Q4`.
- Waktu tersedia 104 hari kalender untuk sisa runbook: Tahap 1–7 (perencanaan), Tahap 8 (pembangunan), Tahap 9–11 (QA, audit keamanan, UAT), dan Tahap 12–13 (persiapan rilis dan deploy) `[DOK: .claude/skills/runbook/SKILL.md]`.
- Tim: user (PM), Claude lewat runbook, dan 1 mobile developer `[DOK] P7.3`.
- Kegiatan di luar pembangunan yang ikut memakan waktu ini, dengan durasi yang belum diketahui: perekrutan dan integrasi rekanan (PT-05), pengurusan izin (PT-02), kontrak penyedia pihak ketiga `[DOK] P4.2`, verifikasi akun developer dan review toko aplikasi (R-08), audit keamanan eksternal, dan UAT bersama konsultan medis `[DOK] P2.4b`.
- Pembagian waktu per tahap dan per fase disusun di Tahap 7 (Phase Plan) setelah scope PRD dan arsitektur jelas.

## 7. Risiko utama dan toleransi

### 7.1 Toleransi proyek

| Aspek | Toleransi | Sumber |
| --- | --- | --- |
| Waktu | 0 hari. Go-live tetap 1 Maret 2027 | `[DOK] P7.2, P8.3` |
| Scope | Boleh dikurangi (layanan yang belum siap menyusul), kecuali syarat izin, audit keamanan, dan sign-off UAT | `[DOK] P8.3` |
| Biaya | +20% dari perkiraan terakhir yang disetujui | `[USER] T1-Q8` |
| Kualitas | Lulus audit keamanan dan sign-off UAT CEO, CTO, konsultan medis | `[DOK] P8.2` |

### 7.2 Register risiko

Penilaian kemungkinan dan dampak (Tinggi/Sedang/Rendah) adalah penilaian penyusun berdasarkan fakta di kolom Dasar `[KEPUTUSAN-02]`, dan dikonfirmasi user di gate. Respons yang menyangkut keputusan bisnis ditulis sebagai usulan untuk Executive.

| # | Risiko | Dasar | Kemungkinan | Dampak | Usulan respons |
| --- | --- | --- | --- | --- | --- |
| R-01 | Lima dari tujuh layanan (klinik, lab, home visit, informasi biaya MCU/lab, pembelian produk) bergantung pada integrasi dengan rekanan yang belum ada dan belum diketahui kemampuan integrasinya. Dua layanan konsultasi online berjalan di Rumah Sehat sendiri | `[DOK] PS6, PS6a, PS6b, P4.3, P8.2` | Tinggi | Tinggi | Rekrut rekanan paralel dan jawab PT-05 sebelum Tahap 4. Kalau scope harus dikurangi (P8.3), urutan layanan yang dipertahankan diputuskan Executive di Tahap 3 atau 7 |
| R-02 | Jadwal 104 hari untuk 7 layanan, 9 peran, 2 aplikasi mobile, portal web, dan banyak integrasi, dengan toleransi waktu 0 | `[DOK] P7.2, P7.3, P6.1, P6.2, P8.3` | Tinggi | Tinggi | Pakai mekanisme pengurangan scope P8.3 dengan prioritas eksplisit di PRD. Pantau di setiap gate |
| R-03 | Kebutuhan badan usaha terpisah dan izin belum diketahui, padahal izin adalah syarat go-live yang tidak bisa dikurangi. Penjualan obat bebas oleh klinik lewat aplikasi juga belum dipastikan boleh (PT-04) | `[DOK] P5.2, P8.2, P8.3, PS2a` | `[TIDAK DIKETAHUI]` (belum diriset) | Tinggi | Jadikan PT-02 dan PT-04 prioritas pertama compliance-reviewer di awal Tahap 2. Kalau hasilnya mengubah biaya atau jadwal secara besar, Business Case ditinjau ulang |
| R-04 | Kapasitas tim (PM, Claude, 1 mobile developer) terhadap scope. Peran QA manusia, DevOps, dan UI/UX tidak disebut | `[DOK] P7.3` | Tinggi | Tinggi | Tinjau ulang setelah arsitektur (Tahap 4) dan phase plan (Tahap 7), ketika volume pekerjaan sudah terukur |
| R-05 | Besaran komisi dan biaya rekanan belum ditentukan, sehingga kelayakan finansial belum bisa dihitung | `[USER] T1-Q2` | Sudah terjadi (isu) | Sedang | CEO menetapkan angka (PT-17) sebelum Tahap 2, karena BRD memuat aturan bisnis biaya |
| R-06 | Penyedia pihak ketiga (payment gateway, video, kurir, WhatsApp) belum dipilih. Waktu kontrak dan onboarding belum diketahui | `[DOK] P4.2` | Sedang | Tinggi | Pemilihan di Tahap 4–5 dibawa ke user. Proses kontrak dimulai segera setelah disetujui |
| R-07 | Tanggal go-live terkait rencana pemasaran. Kalau scope dikurangi, materi dan kampanye pemasaran harus menyesuaikan | `[USER] T1-Q4` `[DOK] P8.3` | Sedang | Sedang | Tim pemasaran diberi tahu prioritas layanan setelah Tahap 7 dan diperbarui di setiap gate |
| R-08 | Publikasi di Play Store dan App Store (syarat go-live 4) membutuhkan akun developer yang terverifikasi. Google Play mewajibkan verifikasi identitas sesuai jenis akun `[SRC-01]`; syarat dan lama verifikasi Apple `[TIDAK DIKETAHUI]` | `[DOK] P8.2` `[SRC-01]` | Sedang | Tinggi | Usulan: daftarkan akun developer atas nama PT. Arunika jauh sebelum Tahap 12. Keputusan dan waktunya ada di Executive |

## 8. Pertanyaan terbuka

### 8.1 Status pertanyaan terbuka dari Project Brief yang bertenggat Tahap 1

| # | Status | Jawaban / tindak lanjut |
| --- | --- | --- |
| PT-01 | Terjawab | PT. Arunika `[USER] T1-Q6` |
| PT-09 | Terjawab sebagian | Bulanan; 70 mencakup dokter dan psikolog `[USER] T1-Q3`. Kriteria "aktif" dilanjutkan di PT-16 |
| PT-11 | Terjawab sebagian | Anggaran total belum ada; biaya developer Rp10–20 juta/bulan `[USER] T1-Q5, T1-Q5a`. Cakupan anggaran (termasuk pemasaran) tetap terbuka |
| PT-14 | Terjawab | Rencana pemasaran `[USER] T1-Q4` |

PT-02 sampai PT-08, PT-10, PT-12, PT-13, dan PT-15 tetap sesuai brief dengan tenggat masing-masing.

### 8.2 Pertanyaan terbuka baru

Kolom Pemilik dan Tenggat adalah usulan penyusun, dikonfirmasi user di gate Tahap 1.

| # | Pertanyaan | Pemilik | Tenggat |
| --- | --- | --- | --- |
| PT-16 | Kriteria "pasien aktif" untuk KPI-01 (cukup membuka aplikasi/login, atau harus bertransaksi) dan status yang dihitung untuk KPI-02 sampai KPI-04 (terdaftar, terverifikasi, atau terintegrasi) | CEO | Sebelum Tahap 3 |
| PT-17 | Besaran komisi konsultasi, komisi produk, dan bentuk serta besaran biaya rekanan | CEO | Sebelum Tahap 2 |
| PT-18 | Biaya tim selain mobile developer: PM, langganan Claude, dan staf operasional admin Rumah Sehat (verifikasi KTP, layanan pelanggan, CMS) | User (PM) dan CEO | Sebelum gate Tahap 4 |
| PT-19 | Apakah mobile developer dibayar penuh waktu selama periode pembangunan, dan apakah tetap dipertahankan setelah go-live? | User (PM) | Gate Tahap 1 |

## 9. Rencana tinjauan ulang Business Case

| Titik tinjauan | Yang dicek ulang |
| --- | --- |
| Gate Tahap 2 (BRD) | Dampak hasil riset izin (PT-02, PT-04) pada biaya, jadwal, dan scope; besaran pendapatan (PT-17) |
| Gate Tahap 4–5 (Arsitektur, Stack) | Biaya pihak ketiga dihitung dan **baseline biaya ditetapkan** sebagai acuan toleransi +20% |
| Gate Tahap 7 (Phase Plan) | Kelayakan jadwal 1 Maret 2027 terhadap volume pekerjaan; prioritas pengurangan scope |
| Tahap 12 (Pre-deploy) | Status syarat go-live dan kesiapan KPI untuk diukur |
| Hypercare dan 1 Maret 2028 | Pencapaian KPI-01 sampai KPI-04 |
