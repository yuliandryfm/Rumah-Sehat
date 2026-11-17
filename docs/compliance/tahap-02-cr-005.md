# Review Kepatuhan Tahap 2, CR-005: BRD Rumah Sehat v1.1

Tanggal: 2026-11-17
Reviewer: compliance reviewer (Claude). **Ini bukan nasihat hukum.** Temuan yang sumbernya hanya terbaca sebagian berstatus **indikasi**.
Dokumen yang direview: `docs/02-brd.md` v1.1 dan `docs/02-catatan-brd.md` (Q13 sampai Q18). Cakupannya **hanya perubahan CR-005**. BRD v1.0 sudah lulus review (`docs/compliance/tahap-02.md`).
Tingkat ketelitian: **T1** (alur uang, kalkulasi harga).
Masukan: `docs/compliance/tahap-02-riset.md`, `docs/sources.md`, `docs/backlog.md` (CR-005), `docs/traceability.md`.
Kriteria blokir: `aturan-kerja.md` §2. Blokir hanya untuk hal substantif, dan catatan non-blokir dibatasi 5 butir.

**Cara menentukan baris yang berubah.** Reviewer tidak punya akses shell, sehingga `git diff main` tidak dijalankan. Baris yang berubah ditentukan dari penanda "(CR-005)" di BRD, status "BRD v1.1 (CR-005)" di `docs/traceability.md`, entri CR-005 di `docs/backlog.md`, catatan Q13 sampai Q18, dan rujukan baris di `docs/compliance/tahap-02.md`. Baris yang diperiksa: 4, 90–96, 114, 140–142, 183, 187, 206–211, 235, 303, 312–313, 337, 345, 353, 366–367, dan 403.

## Ringkasan

| Aspek | Hasil |
| --- | --- |
| Klaim regulasi baru di revisi | Hanya satu, yaitu BR-040 `[SRC-19]`, dan sumbernya mendukung. Tidak ada klaim yang melebihi sumber |
| Fakta bisnis baru | Semuanya berlabel `[USER] T2-Q13` sampai `T2-Q18`, dan tercatat di catatan wawancara |
| Perluasan L-04, L-13, L-14 | Memadai untuk cakupannya. Ada tiga celah: L-10, L-12, dan L-14 perlu diperluas (NB-2, NB-3, NB-5) |
| Kewajiban baru yang belum tertangkap | Tidak ada yang memblokir. Ada dua risiko baru yang perlu dicatat: konsistensi harga di informasi biaya (NB-1) dan data transaksi layanan kesehatan (NB-4) |
| Blokir | **0** |
| **Status** | **LULUS** |

## 1. Temuan

### Blokir

Tidak ada.

### Non-blokir (5 terpenting)

| # | Kebutuhan | Sumber | Status | Bukti | Rekomendasi |
| --- | --- | --- | --- | --- | --- |
| NB-1 | Harga yang ditampilkan sebelum mendaftar harus sama dengan harga yang ditagih | `[TIDAK DIKETAHUI]` untuk ketentuannya. Rujukan konteksnya `[SRC-23]` (Permendag 19/2026 mengatur perlindungan konsumen, isi pasal tidak terbaca) | Perlu keputusan user dan konfirmasi legal | P-06 (baris 93) dan BR-023 (baris 186) menampilkan "biaya MCU dan pemeriksaan lab dari sistem rekanan". Sejak CR-005, BR-042 (baris 208) menetapkan harga pendaftaran lab = harga rekanan + markup. Di v1.0 keduanya konsisten karena lab dibayar langsung ke mitra. Sekarang informasi biaya bisa menampilkan harga rekanan, sedangkan yang ditagih saat mendaftar harga rekanan + markup | Bawa ke user, sebaiknya di gate v1.1, atau catat sebagai PT baru untuk Tahap 3: apakah BR-023 menampilkan harga yang akan dibayar (rekanan + markup)? Usulan teks bila user memilih harga bayar: "BR-023 Pasien dapat melihat informasi biaya MCU dan pemeriksaan lab sebelum mendaftar. Biaya yang ditampilkan sama dengan harga yang dibayar bila mendaftar di aplikasi (harga rekanan + markup, BR-042)." Masukkan juga ke L-14 |
| NB-2 | Pengaturan tarif oleh Kemenkes kini menyentuh semua layanan bermarkup, tidak hanya konsultasi | `[SRC-06]` (indikasi; pernyataan rencana, belum ada nomor peraturan) | Perlu konfirmasi legal | L-10 (baris 309) masih berbunyi "Pengaturan tarif konsultasi atau markup platform oleh Kemenkes". Halaman resmi `[SRC-06]` yang dibuka ulang hari ini menyebut regulasi mencakup "penetapan tarif layanan" dan "penetapan tarif pelayanan kesehatan digital". Halaman itu tidak membatasi pengaturan ke telemedisin. Sejak CR-005, markup (BR-042) berlaku juga untuk klinik, lab, dan home visit. Pencarian peraturan tarif platform atau batasan markup atas tarif fasyankes tidak menemukan peraturan yang sudah terbit (lihat bagian 4) | Perluas L-10 menjadi: "Pengaturan tarif pelayanan kesehatan digital atau markup platform oleh Kemenkes, untuk konsultasi, pendaftaran klinik, pendaftaran lab, dan home visit (CR-005); serta kebolehan menambahkan markup di atas tarif fasyankes rekanan". Tambahkan BR-024 ke BR terkait. Tenggat Tahap 12 boleh tetap karena BR-118 sudah memantau, tetapi dampaknya ke Business Case kini lebih besar |
| NB-3 | MDR QRIS untuk transaksi klinik, lab, dan home visit; siapa *merchant*-nya | `[SRC-24]` (dibuka ulang hari ini, kutipan tarif diusulkan masuk register, lihat bagian 4) | Sebagian. BR-044 sudah berlaku umum untuk semua transaksi QRIS, sehingga tidak ada pelanggaran | BR-044 (baris 210) tidak membedakan layanan, jadi otomatis mencakup transaksi baru. Namun `[SRC-24]` menunjukkan tarif MDR QRIS bergantung pada **kategori usaha merchant dan nominal transaksi**: Usaha Mikro "0%" sampai Rp500.000 dan "0,3%" di atasnya; Usaha Kecil, Menengah, dan Besar "0%" sampai Rp100.000 dan "0,7%" di atasnya. Tidak ada kategori khusus kesehatan. Transaksi klinik dan lab cenderung bernilai lebih besar, sehingga tarif yang berlaku ditentukan oleh siapa *merchant*-nya (rekanan atau PT. Arunika). Ini persis pertanyaan L-12 (baris 311). Teks L-12 belum diubah, padahal catatan Q13 (catatan-brd baris 94) dan `docs/backlog.md` baris 68 menyatakan "cakupan L-04, L-12, L-14 meluas" | Perluas L-12: "... dan siapa *merchant of record* dalam pembagian dana langsung, untuk semua layanan termasuk klinik, lab, dan home visit (CR-005). Kategori usaha merchant menentukan tarif MDR QRIS `[SRC-24]`." Tambahkan ke PT-27 (baris 403): siapa menanggung MDR atas transaksi yang di-refund. Catatan bisnis: MDR QRIS ditanggung rekanan (T2-Q12), sehingga potongan bagi rekanan atas transaksi bernilai besar dapat memengaruhi minat rekanan (R-01) |
| NB-4 | Data transaksi layanan kesehatan dapat mengungkap data kesehatan | `[SRC-15]` (Pasal 4 ayat (2): data spesifik meliputi "data dan informasi kesehatan") | Perlu konfirmasi legal | BR-105 (baris 279) memetakan data kesehatan sebagai catatan tenaga kesehatan, isi chat, resep, dan hasil lab. Sejak CR-005, catatan pembayaran juga memuat nama pemeriksaan lab, layanan klinik, atau home visit, dan data itu mengalir ke payment gateway (pihak ketiga). Apakah data transaksi semacam ini termasuk "data dan informasi kesehatan" `[TIDAK DIKETAHUI]` | Perluas L-08: "... termasuk apakah data transaksi layanan kesehatan (nama layanan klinik, pemeriksaan lab, home visit) tergolong data kesehatan, dan data minimal yang boleh dikirim ke payment gateway". Untuk PRD dan Tahap 4: deskripsi transaksi ke payment gateway dibuat seminimal mungkin. Payment gateway masuk cakupan PT-22 dan L-09 bila memproses data di luar Indonesia |
| NB-5 | Pasien perlu tahu apa yang tercakup dalam harga di aplikasi; interpretasi Q14 perlu dikonfirmasi | `[SRC-23]` (perlindungan konsumen PMSE, isi pasal `[TIDAK DIKETAHUI]`) | Perlu konfirmasi legal, dan interpretasi perlu ditampilkan di gate | L-14 (baris 313) sudah diperluas ke refund pembatalan, tetapi belum menyebut keterbukaan informasi bahwa tindakan tambahan dibayar di luar aplikasi (BR-024, baris 187; P-04 dan P-05, baris 91–92). Pewawancara juga menerapkan jawaban Q14 (yang menyebut pendaftaran klinik) ke lab dan home visit dengan alasan "pernyataan user berbentuk umum" (catatan-brd baris 98). Ini interpretasi penyusun | (a) Perluas L-14: "... serta informasi bahwa harga di aplikasi hanya mencakup layanan yang tercantum, sedangkan tindakan tambahan dibayar di luar aplikasi; dan konsistensi harga di informasi biaya dengan harga yang dibayar (NB-1)". (b) Di gate v1.1, tampilkan secara eksplisit bahwa aturan tindakan tambahan diterapkan juga ke lab dan home visit, supaya menjadi keputusan user dan bukan interpretasi. (c) Kosmetik: tambahkan BR-024 dan BR-040 ke kolom BR terkait L-04 (baris 303) |

## 2. Jawaban atas pertanyaan review

**1. Kewajiban atau risiko baru yang belum tertangkap di BR-100–BR-118 dan L-01–L-14.**

| Area yang ditanyakan | Sudah tertangkap? | Catatan |
| --- | --- | --- |
| Markup atas tarif pelayanan kesehatan fasyankes | Sebagian, lewat L-13 yang sudah diperluas | Aspek pengaturan tarif oleh Kemenkes belum diperluas di L-10 (NB-2). Tidak ditemukan sumber resmi yang melarang atau mengatur markup atas tarif fasyankes. Statusnya `[TIDAK DIKETAHUI]`, bukan "boleh" |
| Pembayaran layanan fasyankes lewat platform | Ya, lewat L-04, BR-115, dan L-12 | L-12 perlu menyebut transaksi baru secara eksplisit (NB-3). Tidak ditemukan kewajiban baru yang khusus untuk pembayaran layanan fasyankes lewat platform |
| Perlindungan konsumen PMSE (rincian harga, refund) | Sebagian, lewat L-14 | Konsistensi harga di informasi biaya (NB-1) dan keterbukaan soal tindakan tambahan (NB-5) belum tercakup |
| MDR QRIS untuk transaksi baru | Ya, BR-044 berlaku umum | Tarif MDR bergantung pada kategori usaha merchant dan nominal `[SRC-24]`, sehingga jawaban L-12 makin menentukan (NB-3) |
| Risiko lain | Belum | Data transaksi layanan kesehatan dan pengirimannya ke payment gateway (NB-4) |

**2. Kecukupan perluasan L-04, L-13, L-14.** L-04 dan L-13 memadai. L-14 perlu ditambah dua hal (NB-1, NB-5). Yang belum diperluas tetapi terdampak: L-10 (NB-2), L-12 (NB-3), dan L-08 (NB-4). Tidak perlu butir L baru, karena cukup memperluas butir yang ada. BR kepatuhan baru juga tidak diperlukan. Bila user memilih menampilkan harga bayar di informasi biaya, cukup ubah BR-023 (NB-1).

**3. Pernyataan berlebihan atau tanpa sumber.** Tidak ditemukan.
- BR-040 merujuk `[SRC-19]` untuk "payment gateway berizin Bank Indonesia", dan sumbernya mendukung.
- Butir L yang diperluas berbentuk pertanyaan, bukan klaim.
- BR-115 tetap berbentuk kewajiban konfirmasi, bukan pernyataan bahwa izin tidak diperlukan.
- Klaim di §9 ("sumber pendapatan transaksi bertambah", kebutuhan integrasi status lunas) adalah turunan bisnis dari T2-Q13 dan Q15, dan dibawa ke gate.
- Ada satu ketidaksesuaian kecil antara dokumen. Catatan Q13 dan backlog menyebut "L-04, L-12, L-14 meluas", sedangkan BRD memperluas L-04, L-13, dan L-14 (NB-3).

## 3. Daftar wajib konfirmasi tim legal/compliance manusia (dampak CR-005)

Semua L-01 sampai L-14 tetap berlaku sesuai `docs/compliance/tahap-02.md` bagian 4. Tambahan dan penegasan dari CR-005:

1. **Sebelum Tahap 3:** L-14 yang diperluas (refund pembatalan layanan terjadwal, konsistensi harga di informasi biaya, keterbukaan soal tindakan tambahan di luar aplikasi).
2. **Sebelum Tahap 4:**
   - L-04 dan L-12 untuk klinik, lab, dan home visit, termasuk siapa *merchant of record* dan dampaknya ke kategori MDR QRIS.
   - L-13 (markup atas tarif fasyankes).
   - L-08 (status data transaksi layanan kesehatan dan data minimal ke payment gateway).
3. **Sebelum Tahap 12, dipantau lewat BR-118:** L-10 yang diperluas ke tarif semua layanan kesehatan digital.
4. **Usulan pertanyaan terbuka untuk CEO dan konsultan pajak (di luar cakupan riset regulasi ini):** perlakuan pajak atas markup dan atas pembagian dana layanan kesehatan. Ketentuannya `[TIDAK DIKETAHUI]`. Tidak ada klaim pajak yang dibuat di laporan ini.

## 4. Usulan entri `docs/sources.md` (tidak diedit oleh reviewer)

| ID | Usulan |
| --- | --- |
| SRC-24 (perluasan kutipan) | Dibuka ulang 2026-11-17. Tambahkan ke kolom "Mendukung klaim": tarif MDR QRIS per kategori: "Usaha Mikro (UMI)" 0% untuk "Nominal Transaksi ≤Rp500.000,00" dan 0,3% di atasnya; "Usaha Kecil (UKE), Usaha Menengah (UME), dan Usaha Besar (UBE)" 0% untuk ≤Rp100.000,00 dan 0,7% di atasnya; "Pendidikan" 0%/0,6%; "SPBU" 0%/0,4%; BLU, PSO, G2P, P2G 0%. Tidak ada kategori khusus kesehatan. Tambahkan `compliance/tahap-02-cr-005.md` ke kolom "Dipakai di" |
| SRC-06 (perluasan kutipan) | Dibuka ulang 2026-11-17. Tambahkan: "Regulasi tersebut mencakup praktik telemedisin, pencatatan rekam medis, hingga penetapan tarif layanan" dan "penetapan tarif pelayanan kesehatan digital". Tidak menyebut klinik, laboratorium, atau kunjungan rumah. Tambahkan `compliance/tahap-02-cr-005.md` ke kolom "Dipakai di" |
| Pencarian tanpa hasil (baris baru) | 2026-11-17. Yang dicari: peraturan Kemenkes tentang tarif platform kesehatan digital atau batasan markup atas tarif fasyankes (pencarian web, kemkes.go.id). Hasil: hanya pernyataan rencana `[SRC-06]` yang ditemukan; peraturan yang sudah terbit tidak ditemukan. Tindak lanjut: `[TIDAK DIKETAHUI]`, masuk ke L-10 dan L-13 |

## 5. Status

**LULUS.** Tidak ada blokir. Lima catatan non-blokir (NB-1 sampai NB-5) tidak memicu putaran review baru. NB-1 dan NB-5(b) sebaiknya dibawa ke user di gate v1.1 karena menyangkut keputusan bisnis tentang harga yang ditampilkan ke pasien. Semua butir L tetap wajib dijawab tim legal/compliance manusia sebelum tenggat masing-masing.
