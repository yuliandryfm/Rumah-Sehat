# Review Kepatuhan Tahap 2: BRD Rumah Sehat v0.1

Tanggal: 2026-11-17 (review awal dan cek ulang)
Reviewer: compliance reviewer (Claude). **Bukan nasihat hukum.**
Dokumen yang direview: `docs/02-brd.md` v0.1. Bagian 5.5 dan 6 direview di tingkat **T1**, bagian lain hanya sejauh menyentuh kepatuhan.
Masukan: `docs/compliance/tahap-02-riset.md`, `docs/02-catatan-brd.md` (T2-Q1 sampai Q12), `docs/sources.md`.
Kriteria blokir: `aturan-kerja.md` §2 (hanya yang substantif). Cek ulang hanya memeriksa baris yang berubah.

## Ringkasan

| Aspek | Hasil |
| --- | --- |
| Usulan C-01 sampai C-19 tercermin sebagai BR | **19/19.** Semuanya ada di BR-100 sampai BR-118, dengan SRC yang benar |
| Klaim regulasi di 6.1, 6.2, dan 9 | Tidak ada yang melebihi sumber |
| Blokir review awal | 1 (B-1: biaya transaksi ke pasien bertentangan dengan `[SRC-24]` untuk QRIS) |
| Cek ulang (bagian 5) | B-1 **tertutup**. NB-1 sampai NB-5 sudah ditindaklanjuti |
| **Status akhir** | **LULUS** |

## 1. Pemetaan usulan riset (C-xx) ke BR

| Usulan | BR | SRC di BRD | Status | Catatan |
| --- | --- | --- | --- | --- |
| C-01 Pendaftaran PSE | BR-100 | SRC-12, SRC-13 | Terpenuhi | — |
| C-02 Skema telemedisin | BR-101 | SRC-02, SRC-03, T2-Q2 | Terpenuhi, perlu konfirmasi legal | Sudah disesuaikan dengan keputusan K-1 opsi A (L-01) |
| C-03 STR/SIP | BR-102 | SRC-02, SRC-03, SRC-21 | Terpenuhi, perlu konfirmasi legal | L-07 sudah diperluas untuk fisioterapis dan tenaga medis |
| C-04 Persetujuan eksplisit | BR-103 | SRC-15 | Terpenuhi | — |
| C-05 Data anak | BR-104 | SRC-15 | Terpenuhi | — |
| C-06 Data spesifik | BR-105 | SRC-15 | Terpenuhi | — |
| C-07 Hak subjek data | BR-106 | SRC-15 | Terpenuhi, perlu konfirmasi legal (L-06) | — |
| C-08 Perekaman pemrosesan | BR-107 | SRC-15 | Terpenuhi | — |
| C-09 Keamanan dan kerahasiaan | BR-108 | SRC-15, SRC-13 | Terpenuhi | — |
| C-10 Notifikasi kegagalan | BR-109 | SRC-15, SRC-13 | Terpenuhi | — |
| C-11 DPIA | BR-110 | SRC-15 | Terpenuhi, perlu konfirmasi legal | — |
| C-12 Pejabat PDP | BR-111 | SRC-15, SRC-16 | Terpenuhi, perlu konfirmasi legal | — |
| C-13 Retensi | BR-112 | SRC-15, SRC-25 | Sebagian | Angka `[TIDAK DIKETAHUI]` (PT-24). Masukan retensi PMSE dari SRC-25 sudah dicatat |
| C-14 Lokasi dan transfer data | BR-113 | DOK P5.5, SRC-15, SRC-13 | Terpenuhi, perlu konfirmasi legal (L-09) | — |
| C-15 Rekam medis dan resep | BR-114 | SRC-18, SRC-06, SRC-07 | Perlu konfirmasi legal (L-05) | — |
| C-16 Pembayaran berizin BI | BR-115 | SRC-19, SRC-20 | Terpenuhi, perlu konfirmasi legal (L-04) | — |
| C-17 Peredaran daring dan PSEF | BR-116 | SRC-08, SRC-10, SRC-11 | Perlu konfirmasi legal (L-03) | Kontinjensi PT-21 terbuka (CEO) |
| C-18 Perizinan berusaha | BR-117 | SRC-09, SRC-23 | Perlu konfirmasi legal (L-02) | — |
| C-19 Pemantauan regulasi | BR-118 | SRC-06 | Terpenuhi | — |

## 2. Temuan review awal

### Blokir

| # | Kebutuhan | Sumber | Status awal | Bukti | Rekomendasi |
| --- | --- | --- | --- | --- | --- |
| B-1 | Pembebanan biaya transaksi pembayaran ke pasien | `[SRC-24]` | Belum. BR bertentangan dengan sumber resmi untuk QRIS | BR-044 versi awal (baris 209) membebankan biaya transaksi payment gateway ke pasien, dan §9 (baris 357) memakai kesimpulan itu. Sebaliknya, BI menyatakan: "biaya MDR ini ditanggung oleh merchant dan tidak boleh dibebankan kepada konsumen" `[SRC-24]`. Pernyataan itu khusus QRIS | Bawa ke user, revisi BR-044 dan §9, lalu tambahkan L-12. **Status: tertutup (bagian 5)** |

### Non-blokir

| # | Kebutuhan | Sumber | Bukti | Rekomendasi | Tindak lanjut (cek ulang) |
| --- | --- | --- | --- | --- | --- |
| NB-1 | Status hukum markup | Tidak ada sumber resmi yang dibuka | Baris 47, 207, 326 | Tambahkan butir legal | L-13 ditambahkan |
| NB-2 | Perlindungan konsumen PMSE (refund, rincian harga) | `[SRC-23]` `[SRC-25]` `[SRC-20]` | BR-045, P-08 | Tambahkan butir legal | L-14 ditambahkan |
| NB-3 | Retensi data PMSE | `[SRC-25]` | BR-112 | Jadikan masukan untuk PT-24 dan L-05/L-06 | BR-112 dan 6.1 diperbarui, dengan catatan status PP 80/2019 belum diverifikasi |
| NB-4 | Telemedisin oleh fisioterapis dan tenaga medis | `[SRC-02]` `[SRC-03]` | BR-012, BR-102 | Perluas L-07 | L-07 diperluas |
| NB-5 | Rujukan PSEF ke Permenkes 11/2025 | `[SRC-08]` `[SRC-09]` | 6.1 baris 264 | Rapikan rumusan | Dirapikan: portal menautkan FAQ Permenkes 11/2025, dan dasar hukumnya belum terverifikasi |

## 3. Apakah ada butir kepatuhan yang memblokir PRD? (diperbarui setelah cek ulang)

| Butir | Memblokir PRD? | Alasan |
| --- | --- | --- |
| K-1 skema telemedisin (opsi A) | Tidak | Sudah diputuskan user. L-01 dijawab sebelum Tahap 4 |
| K-2 obat bebas (apotek dan klinik berizin) | Tidak | Obat bebas bisa dilepas lewat P8.3. PRD sebaiknya menandainya sebagai bagian yang bisa dilepas tanpa memengaruhi produk kecantikan dan kesehatan |
| K-3 pembagian dana langsung (opsi A) | Tidak | L-04 sebelum Tahap 4 |
| B-1 biaya transaksi | **Tidak lagi** | Diputuskan user (T2-Q12). PRD dapat menulis rincian total pembayaran: MDR QRIS ditanggung rekanan, biaya non-QRIS ke pasien. Acceptance criteria untuk metode non-QRIS sebaiknya dibuat bisa diubah lewat konfigurasi, karena menunggu L-12 |

## 4. Daftar konfirmasi legal

L-01 sampai L-14 di BRD §6.3 wajib dijawab tim legal/compliance manusia. L-07 sudah mencakup fisioterapis dan tenaga medis. Urutan terdesak:

1. **Sebelum Tahap 3 (PRD):** L-07 (verifikasi nakes per profesi) dan L-14 (refund dan rincian harga).
2. **Sebelum Tahap 4:** L-01, L-04, L-05, L-06, L-08, L-12 (MDR non-QRIS dan *merchant of record*), dan L-13 (markup).
3. **Sebelum Tahap 7:** L-03 (obat bebas dan PSEF).
4. **Sebelum Tahap 10–12:** L-02, L-09, L-10, L-11.

## 5. Cek ulang (hanya baris yang berubah)

Dasar keputusan: `docs/02-catatan-brd.md` bagian "Putaran tambahan setelah review kepatuhan" (Q12, jawaban "C. QRIS rekanan, lain pasien").

| Baris `docs/02-brd.md` | Perubahan | Hasil |
| --- | --- | --- |
| 209 (BR-044) | MDR QRIS ditanggung rekanan (dipotong dari bagian rekanan) dan tidak dibebankan ke pasien. Metode lain dibebankan ke pasien sambil menunggu L-12. Sumber T2-Q8, T2-Q12, SRC-24 | **Sesuai sumber.** QRIS tidak lagi dibebankan ke konsumen, sesuai `[SRC-24]`. Metode non-QRIS ditandai menunggu legal, dan itu benar karena `[SRC-24]` hanya membahas QRIS. Keputusan berlabel `[USER]`. **B-1 tertutup** |
| 90 (P-03), 94 (P-07) | "Biaya transaksi" kini hanya untuk metode non-QRIS | Konsisten dengan BR-044. Catatan kosmetik: kolom sumber belum menyertakan T2-Q12 (non-blokir) |
| 265 (6.1 sistem pembayaran) | Tambah kebijakan MDR QRIS dan SRC-24. Ketentuan non-QRIS ditulis `[TIDAK DIKETAHUI]` | Sesuai sumber |
| 266 (6.1 PMSE) | Tambah PP 80/2019 (status belum diverifikasi) dan SRC-25. Keyakinan "Rendah" | Sesuai sumber |
| 264 (6.1 PSEF) | Rumusan dirapikan | Sesuai NB-5 |
| 285 (BR-112) | Masukan retensi PMSE 10/5 tahun, dengan catatan status PP belum diverifikasi | Sesuai `[SRC-25]` |
| 302 (L-04), 305 (L-07), 310–312 (L-12 sampai L-14) | Perluasan dan penambahan butir legal | Sesuai rekomendasi |
| 338 (7.3), 392 (PT-20) | Rujukan "L-01 sampai L-14" | Konsisten |
| 360 (§9 biaya transaksi dan ongkir) | Kesimpulan bahwa biaya transaksi tidak mengurangi pendapatan Rumah Sehat kini bersyarat "selama konfirmasi legal L-12 tidak mengubahnya". Sumber T2-Q7, Q8, Q12, SRC-24 | Sesuai. MDR QRIS ditanggung rekanan, bukan Rumah Sehat |

Catatan untuk legal, tidak memblokir: `[SRC-24]` menyebut MDR ditanggung "merchant". Dalam pembagian dana langsung, apakah rekanan atau PT. Arunika yang berstatus merchant sudah masuk L-12.

**Status akhir: LULUS.** Tidak ada blokir tersisa. Satu catatan kosmetik (label T2-Q12 di P-03/P-07) tidak memicu putaran baru. Semua butir L-01 sampai L-14 tetap wajib dikonfirmasi tim legal/compliance manusia sesuai tenggat di bagian 4.
