# Fact-check Tahap 02, revisi CR-005: BRD v1.0 ke v1.1

Tanggal: 2026-11-17
Dokumen diperiksa: `docs/02-brd.md` (bagian yang berubah karena CR-005, ditambah pemindaian seluruh file untuk sisa kalimat lama), `docs/backlog.md` CR-005, `docs/traceability.md` (baris berstatus "BRD v1.1 (CR-005)" dan jumlah baris), `docs/STATUS.md` baris Tahap 2. Bukti sumber: `docs/02-catatan-brd.md` (Q4, Q8, Q9, Q12, Q13–Q18), `docs/00-catatan-intake.md` (P3.3, PS4, PS6a), `docs/00-project-brief.md` baris 67, `docs/01-business-case.md` §5.1, `docs/fact-check/tahap-02.md`.
Tingkat ketelitian: T1 untuk §5.5 (alur uang) dan klausul uang di P-04, P-05, P-08; T2 untuk bagian lain.
Hasil: LULUS setelah cek ulang. Putaran 1 TIDAK LULUS (1 blokir, 5 non-blokir); cek ulang menutup blokir dan menyisakan 2 non-blokir baru (lihat "Cek ulang")

Catatan metode: fact-checker tidak punya akses shell, jadi `git diff main` tidak bisa dijalankan. Bagian yang berubah ditentukan dari penanda `(CR-005)` dan label `T2-Q13`–`T2-Q18` di BRD, baris traceability berstatus "BRD v1.1 (CR-005)", isi CR-005 di backlog, dan nomor baris di `tahap-02.md`. Untuk pemeriksaan sisa kalimat lama, seluruh `02-brd.md` dipindai, bukan hanya bagian yang berubah.

## Ringkasan (putaran 1)
- Total klaim: sekitar 60 klaim baru atau berubah (BRD, backlog, traceability)
- Terverifikasi: sekitar 58
- Tanpa sumber: 0
- Sumber tidak mendukung: 2 klausul (P-04 dan P-05, "ditambah biaya transaksi untuk metode non-QRIS"), digabung menjadi blokir B-1
- Kontradiksi: 0 yang belum tercatat. Brief baris 67 (P3.3) dan Business Case §5.1 baris 90 ("Pembayaran klinik, lab, dan home visit dilakukan langsung ke mitra") memang bertentangan dengan v1.1, tetapi sudah tercatat sebagai dokumen terdampak CR-005 (BRD §8 baris 353, backlog baris 66–68, §9 baris 366)
- Fakta bisnis berlabel asumsi/keputusan: 0

## Hal yang diperiksa dan bersih
- **Label T2-Q13–T2-Q18 ada dan cocok** dengan `02-catatan-brd.md` baris 92–113, dan tidak melebihi ucapan user:
    - Q13 (semua layanan yang dipesan di aplikasi dibayar di aplikasi; pasien cukup menunjukkan appointment) mendukung BR-024, BR-040, S-10, P-04, P-05, dan butir non-scope baris 142.
    - Q15 ("Ya, semua") mendukung BR-042 dan baris §9 di baris 366.
    - Q16 ("Diputuskan CEO", opsi "sistem tetap menyediakan pengaturannya") mendukung BR-043 dan PT-17.
    - Q17 dan Q18 mendukung BR-045, P-08, butir non-scope baris 140, dan L-14. Kalimat "pembatalan setelah batas waktu itu tidak di-refund" adalah sisa aturan dasar T2-Q9, jadi bukan klaim baru. Pembelian produk tetap mengikuti T2-Q9, sesuai catatan Q18.
- **Tidak ada sisa kalimat lama di seluruh `02-brd.md`.** Pemindaian frasa "langsung ke mitra", "dibayar langsung", "konsultasi dan produk", "hanya bila/kalau", "layanan gagal", "komisi", dan semua kemunculan "markup" tidak menemukan kalimat yang masih menyatakan:
    - klinik, lab, atau home visit dibayar ke mitra (satu-satunya kemunculan ada di baris 142, sebagai butir non-scope "diganti");
    - markup hanya untuk konsultasi dan produk;
    - refund "hanya bila layanan gagal" tanpa pengecualian layanan terjadwal.

    Bagian berikut sudah konsisten: §2.2, §3.2, §4.1 S-10, §4.2, §5.3, §5.5, §6.3 L-04/L-13/L-14, §7.3, §8, §9, §10 PT-17/PT-27. Butir non-scope v1.0 "Pembayaran klinik, lab, dan home visit di aplikasi" sudah dihapus.
- **Traceability**: masih 67 baris, sama dengan jumlah BR di BRD (tidak ada BR baru). Label dan awal teks BR-020, BR-024, BR-040, BR-042, BR-043, BR-045, dan BR-070 cocok dengan BRD.
- **Register**: tidak ada SRC, ASUMSI, atau KEPUTUSAN baru. SRC-19 dan SRC-24 yang dirujuk sudah terdaftar dan sudah diverifikasi di `tahap-02.md`.
- **Backlog CR-005** cocok dengan BRD: deskripsi, keputusan Q13–Q18, status "menunggu persetujuan user", dan dampak (pembagian dana, refund/PT-27, integrasi status lunas/PT-05, Business Case §5).

## Penilaian butir turunan penyusun (putaran 1)
| Butir | Dasar | Ditandai jujur? | Perlu persetujuan eksplisit di gate? |
| --- | --- | --- | --- |
| Biaya transaksi non-QRIS ke pasien dan MDR QRIS ke rekanan, kini berlaku untuk klinik, lab, dan home visit (P-04, P-05; BR-044 secara implisit) | T2-Q8/Q12 dijawab saat transaksi di aplikasi hanya konsultasi dan produk (konteks Q8 merujuk Q4). Dampak yang disampaikan ke user di Q13 (catatan baris 94) hanya "pembagian dana langsung, markup, dan refund", tanpa biaya transaksi. Markup diperlakukan perlu ditanyakan ulang (Q15), tetapi biaya transaksi tidak | **Tidak.** Di P-04 dan P-05 klausul ini berlabel T2-Q13/Q14/Q15, padahal ketiganya tidak menyebut biaya transaksi. Tidak ada catatan turunan | **Ya, blokir B-1** |
| "Tindakan tambahan di luar aplikasi" diterapkan juga ke lab dan home visit (P-04, P-05, BR-024, §4.2) | Ucapan user di Q14 berbentuk umum ("tindakan tambahan yang tidak tercantum di aplikasi"), walaupun pertanyaannya tentang klinik | Ya di catatan wawancara (baris 98). Di BRD tidak ditandai | Dianjurkan disebut di rekap gate (non-blokir N-1) |
| BR-070: sistem rekanan menerima status lunas dan appointment lewat integrasi | Konsekuensi T2-Q13 ditambah prinsip integrasi PS6a ("berlaku untuk semua"). Dampaknya sudah disampaikan ke user di Q13 | Cukup | Tidak wajib (non-blokir N-2) |

## Temuan (putaran 1)
| # | Lokasi (file, bagian) | Klaim | Masalah | Usulan |
| --- | --- | --- | --- | --- |
| B-1 (blokir, kategori 1 dan 4; T1) | `02-brd.md` §3.2 P-04 (baris 91) dan P-05 (baris 92); secara implisit BR-044 (baris 210) dan §9 baris 362 | Pendaftaran klinik/lab dan home visit dibayar "harga rekanan + markup, ditambah biaya transaksi untuk metode non-QRIS". Akibat ikutannya lewat BR-044: MDR QRIS untuk klinik, lab, dan home visit dipotong dari bagian rekanan | Ini fakta alur uang baru: pasien membayar biaya transaksi atas pendaftaran klinik/lab/home visit, dan klinik/lab kini menanggung MDR QRIS untuk layanan yang di v1.0 dibayar langsung tanpa payment gateway. (1) Labelnya, T2-Q13/Q14/Q15, tidak memuat hal ini. (2) T2-Q8/Q12 dijawab saat konteksnya hanya konsultasi dan produk. (3) Dampak ini tidak disampaikan ke user di Q13. (4) BRD tidak menandainya sebagai turunan penyusun. Jadi keputusan ini diambil penyusun dan belum dibawa ke user. Ini juga berdampak ke R-01, karena menambah beban biaya bagi klinik dan lab rekanan | Pilih salah satu. (a) **Disarankan**: ajukan satu pertanyaan ke user, misalnya "Apakah aturan T2-Q12 (MDR QRIS ditanggung rekanan, biaya transaksi metode lain ke pasien) juga berlaku untuk pendaftaran klinik, lab, dan home visit?". Catat sebagai T2-Q19, lalu beri label `T2-Q8, T2-Q12, T2-Q19` `[SRC-24]` di P-04/P-05. (b) Tandai klausul itu di P-04/P-05 sebagai "turunan penyusun dari T2-Q8/Q12, dikonfirmasi di gate", masukkan ke rekap gate sebagai butir tersendiri, dan perbarui labelnya. Cek ulang cukup pada baris 91, 92, dan catatan wawancara |
| N-1 (non-blokir) | `02-brd.md` P-04, P-05, BR-024, §4.2 baris 141 | Tindakan tambahan di luar aplikasi berlaku juga untuk lab dan home visit `[USER] T2-Q14` | Rumusan user memang umum, jadi klaim tidak melebihi ucapan user. Namun pertanyaannya tentang klinik, dan BRD tidak menyebut bahwa penerapan ke lab dan home visit adalah tafsiran pewawancara (catatan baris 98) | Sebut di rekap gate: "tindakan tambahan di luar aplikasi juga berlaku untuk lab dan home visit (tafsiran dari T2-Q14)" |
| N-2 (non-blokir) | `02-brd.md` BR-070 (baris 235), §9 baris 367 | Integrasi otomatis untuk "status lunas dan appointment" | Wajar sebagai konsekuensi T2-Q13 dan PS6a. Namun PT-05 (kemampuan integrasi rekanan) masih `[TIDAK DIKETAHUI]`, dan ada alternatif seperti pencocokan appointment lewat portal admin klinik/lab | Tambahkan "mekanisme pencocokan ditetapkan di PRD/Tahap 4" agar BRD tidak mengunci cara teknisnya |
| N-3 (non-blokir) | `02-brd.md` BR-040 (baris 206), S-10 (baris 114) | Semua layanan dibayar di aplikasi `[DOK] P3.3` ... `[USER] T2-Q13` | P3.3 ("Campuran") menyatakan kebalikannya untuk klinik, lab, dan home visit. Kalimat yang sama sudah ditulis dengan benar di BR-024 ("Menggantikan keputusan brief P3.3") | Tulis `[DOK] P3.3 (konsultasi; klinik/lab/home visit diganti CR-005)` atau pindahkan P3.3 ke keterangan |
| N-4 (non-blokir) | `02-brd.md` §8 CR-005 (baris 353); `backlog.md` CR-005 (baris 66, 68); `traceability.md` BR-041, BR-044 | Daftar dokumen dan bagian terdampak | §8 tidak menyebut BR-020, BR-070, P-04, P-05, P-08, S-10, L-04, L-13, dan L-14. Backlog menyebut "L-12 meluas", padahal teks L-12 tidak berubah, sedangkan L-13 berubah. Status BR-041 dan BR-044 masih "BRD v1.0", padahal cakupannya bertambah | Lengkapi daftar terdampak. Opsional: ubah status BR-041 dan BR-044 menjadi "BRD v1.0 (cakupan diperluas CR-005)" |
| N-5 (non-blokir) | `02-brd.md` BR-045 (baris 211) | Aturan refund layanan terjadwal | Q17 secara eksplisit menanyakan pasien yang tidak datang (*no-show*). BRD tidak menyebutnya. Secara logika *no-show* tidak di-refund (tidak membatalkan sebelum batas waktu), tetapi hal ini tidak tertulis | Tambahkan *no-show* ke daftar yang ditetapkan di PRD, atau tulis eksplisit bila user menyetujuinya di gate |

Status putaran 1: TIDAK LULUS karena B-1.

## Cek ulang (2026-11-17)

Cakupan (aturan-kerja §2): hanya baris yang berubah sejak putaran 1, yaitu:
- penutupan B-1 lewat T2-Q19;
- penerapan N-1 sampai N-5;
- perluasan dari review kepatuhan CR-005 (L-04, L-08, L-10, L-12, L-14, PT-27, PT-28, `sources.md` SRC-06/SRC-24 dan baris "Pencarian tanpa hasil");
- T2-Q20 (P-06, BR-023).

Nomor baris mengacu ke `docs/02-brd.md` versi cek ulang.

**Penutupan temuan putaran 1**

| # | Status | Bukti |
| --- | --- | --- |
| B-1 | Ditutup | Q19 ada di `02-catatan-brd.md` baris 115–117. Konteksnya menyebut temuan B-1, ada tiga opsi (sama; MDR QRIS dari markup; semua dari markup), dan jawabannya "Ya, sama". Keputusan diambil user tanpa rekomendasi pewawancara. P-04 (baris 91), P-05 (baris 92), dan BR-044 (baris 210) kini berlabel `[USER] T2-Q8, T2-Q12, T2-Q19` `[SRC-24]`. Kalimat tambahan di BR-044 ("berlaku untuk semua layanan yang dibayar di aplikasi, termasuk pendaftaran klinik, lab, dan home visit") sesuai catatan Q19. Traceability baris 42 sama |
| N-1 | Ditutup | BR-024 (baris 187): "user menjawab untuk klinik; penerapan ke lab dan home visit adalah tafsiran penyusun atas pernyataan umum user, diajukan di gate" |
| N-2 | Ditutup | BR-070 (baris 235): mekanisme pencocokan appointment (integrasi atau portal admin klinik/lab) ditetapkan di PRD dan Tahap 4 |
| N-3 | Ditutup | BR-040 (baris 206) dan S-10 (baris 114): "Bagian klinik, lab, dan home visit di brief P3.3 diganti CR-005" |
| N-4 | Ditutup | §8 baris 353 kini menyebut P-04, P-05, P-06, P-08, S-10, §4.2, BR-020, BR-023, BR-024, BR-040, BR-042 s.d. BR-045, BR-070, L-04, L-13, L-14, PT-17, PT-27, dengan label `T2-Q13–T2-Q20`. Backlog CR-005 baris 68 menyebut L-04, L-08, L-10, L-12, L-13, L-14, dan PT-28. Traceability BR-041 berstatus "BRD v1.1 (cakupan meluas, CR-005)" dan BR-044 "BRD v1.1 (CR-005)" |
| N-5 | Ditutup | BR-045 (baris 211): "perlakuan pasien yang tidak datang (no-show)" masuk daftar yang ditetapkan di PRD. Tidak ada klaim perlakuan yang dikarang |

**Perubahan baru yang diperiksa**
- **T2-Q20** (`02-catatan-brd.md` baris 119–121): ada, dengan tiga opsi. Rekomendasi pewawancara ditandai terbuka sebagai rekomendasi, dan jawabannya "Harga bayar di app". P-06 (baris 93) dan BR-023 (baris 186) menyatakan biaya yang ditampilkan = harga yang dibayar bila mendaftar di aplikasi (harga rekanan + markup). Label `[USER] T2-Q15, T2-Q20` dan `T2-Q20` sah dan tidak melebihi ucapan user. Traceability baris 28 sama.
- **SRC-24** dibuka ulang (https://www.bi.go.id/id/publikasi/ruang-media/cerita-bi/Pages/mdr-QRIS.aspx) dan kutipan tambahannya cocok. Tarif berbeda per kategori merchant: UMI 0% sampai Rp500.000 dan 0,3% di atasnya; UKE/UME/UBE 0% sampai Rp100.000 dan 0,7% di atasnya; Pendidikan 0%/0,6%; SPBU 0%/0,4%; BLU/PSO/G2P/P2G 0%. Tidak ada kategori khusus kesehatan. Halaman juga memuat "ditanggung oleh merchant dan tidak boleh dibebankan kepada konsumen" dan "berlaku efektif mulai 1 Oktober 2026". Kalimat L-12 "Kategori usaha merchant menentukan tarif MDR QRIS `[SRC-24]`" didukung sumber.
- **SRC-06** dibuka ulang (https://www.kemkes.go.id/id/wamenkes-dante-seluruh-platform-kesehatan-digital-harus-diregulasi). Halaman memuat "akan mengatur seluruh platform pelayanan kesehatan digital" dan "Regulasi tersebut mencakup praktik telemedisin, pencatatan rekam medis, hingga penetapan tarif layanan". Halaman tidak menyebut nomor peraturan yang sudah terbit dan tidak menyebut klinik, laboratorium, atau home visit. Kutipan di `sources.md` akurat. L-10 (baris 309) hanya berupa pertanyaan legal, dan "peraturan yang sudah terbit tidak ditemukan; yang ada pernyataan rencana" tidak melebihi sumber.
- **Baris "Pencarian tanpa hasil"** di `sources.md` baris 51 (tarif platform/markup Kemenkes) jujur, karena hasilnya `[TIDAK DIKETAHUI]` dan dirujukkan ke L-10/L-13.
- **L-04, L-08, L-12, L-14** (baris 303, 307, 311, 313) dan **PT-27** (baris 403): semuanya berbentuk pertanyaan legal atau terbuka, bukan keputusan bisnis. Rujukan BR di kolom "BR terkait" ada di BRD.
- **PT-28** (baris 404): pertanyaan pajak, ditandai "usulan review kepatuhan CR-005" dan ketentuannya `[TIDAK DIKETAHUI]`. Ini bukan fakta bisnis yang diasumsikan.
- Tidak ada SRC, ASUMSI, atau KEPUTUSAN baru. Traceability tetap 67 baris.

**Temuan cek ulang**: tidak ada blokir.

| # | Lokasi | Masalah | Usulan |
| --- | --- | --- | --- |
| R-1 (non-blokir) | `02-brd.md` §10.2 kalimat pengantar (baris 392) dan PT-28 (baris 404) | Pengantar menyatakan kolom Pemilik dan Tenggat "disetujui user di gate Tahap 2". Pemilik dan tenggat PT-28 (CEO + konsultan pajak, gate Tahap 4) baru diusulkan setelah gate itu | Masukkan pemilik dan tenggat PT-28 ke rekap gate v1.1, atau beri catatan "PT-28: diusulkan, dikonfirmasi di gate v1.1" |
| R-2 (non-blokir) | `02-brd.md` §8 baris 353; §9 baris 362 | Daftar terdampak §8 belum memuat L-08, L-10, L-12, dan PT-28 yang ikut diperluas atau ditambahkan setelah review kepatuhan (backlog sudah memuatnya). Label baris §9 "Biaya transaksi dan ongkir" belum menyertakan T2-Q19 | Samakan dengan backlog CR-005; tambahkan `T2-Q19` di baris 362 |

**Status akhir: LULUS.** B-1 ditutup lewat jawaban user T2-Q19 dengan label yang sah. N-1 sampai N-5 sudah ditutup. Perubahan dari review kepatuhan dan T2-Q20 punya label dan sumber yang mendukung. R-1 dan R-2 tidak memicu putaran ulang. Butir yang tetap perlu persetujuan eksplisit di gate v1.1:
- tafsiran tindakan tambahan untuk lab dan home visit (BR-024);
- pemilik dan tenggat PT-28.
