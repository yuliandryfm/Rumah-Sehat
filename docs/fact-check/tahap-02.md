# Fact-check Tahap 02: BRD

Tanggal: 2026-11-17
Dokumen diperiksa: `docs/02-brd.md` (utama), `docs/backlog.md` bagian Change Request (CR-001 s.d. CR-004), `docs/traceability.md` (kolom BR). Bukti sumber: `docs/02-catatan-brd.md`, `docs/00-project-brief.md`, `docs/00-catatan-intake.md`, `docs/01-business-case.md`, `docs/01-catatan-business-case.md`, `docs/compliance/tahap-02-riset.md`, `docs/adr/ADR-005-audit-trail-append-only.md`, `docs/sources.md`.
Tingkat ketelitian: T1 untuk BRD bagian 5.5 (pembayaran) dan 6 (kepatuhan); T2 untuk bagian lain.
Hasil: LULUS (putaran 1: tidak ada blokir, 5 non-blokir; cek ulang: tidak ada blokir, 2 non-blokir baru. Lihat bagian "Cek ulang")

## Ringkasan
- Total klaim: sekitar 250 (perkiraan; satu baris tabel bisa memuat beberapa klaim)
- Terverifikasi: sekitar 245
- Tanpa sumber: 1 (non-blokir, butir 5: kalimat portal klinik di §2.2)
- Sumber tidak mendukung: 1 sebagian (non-blokir, butir 1: label `[USER] T2-Q6` untuk audit markup; dasar sah tersedia di ADR-005 yang sudah dikutip di baris yang sama)
- Kontradiksi: 0 yang tidak tercatat sebagai CR dan mengubah keputusan
- Fakta bisnis berlabel asumsi/keputusan: 0 (BRD tidak memakai `[ASUMSI]`/`[KEPUTUSAN]`)

## Hal yang diperiksa dan bersih
- **Label `[USER] T2-Qx`** (Q1–Q11, Q3a, Q3b, Q6a) cocok dengan `02-catatan-brd.md` dan tidak melebihi jawaban user: struktur PSE + fasyankes rekanan (Q2), obat bebas oleh apotek dan klinik rekanan berizin (Q3, Q3b), pembagian dana langsung oleh payment gateway (Q4), fisioterapis/tenaga medis konsultasi + home visit (Q5), markup persen/nominal per rekanan untuk konsultasi dan produk (Q6, Q6a), ongkir dan biaya transaksi ke pasien (Q7, Q8), refund hanya bila layanan gagal (Q9), definisi KPI (Q10, Q11). "Besarannya ditetapkan CEO sebelum go-live" (BR-042, PT-17) sesuai teks opsi Q1 yang dipilih.
- **Label `[DOK] Px.x`/`PSx`** sesuai catatan intake: login (P4.4), KTP (P4.6a/b), data disimpan (P4.1), notifikasi (P4.5), syarat go-live (P8.2, P8.3), non-scope (P3.6, P3.6a), gateway (PS6, PS6a), backoffice (P8.2a), audit trail 10 butir + setiap transaksi (P5.4). `T1-Q1` dan KPI dari Business Case §4.1 cocok.
- **Kontradiksi dengan dokumen lulus gate tercatat sebagai CR**: PS6e (CR-001), PS2/PS2a (CR-002), PS4 dan "penerusan dana" P8.2a (CR-003), komisi potongan Business Case §5.1 (CR-004). Perubahan lain hanya mengisi pertanyaan terbuka yang memang dibiarkan terbuka di dokumen lulus gate (PT-07 tugas fisioterapis/tenaga medis, PT-16 definisi KPI, penanggung ongkir dan biaya transaksi di Business Case §6.1), sehingga tidak memerlukan CR. Audit trail butir (3) dan (5) di BR-080 disesuaikan secara konsisten dengan CR-001/CR-003.
- **Traceability**: 67 BR di BRD (BR-001 s.d. BR-118, dengan celah penomoran yang disengaja) semuanya ada di `traceability.md` dengan label sumber yang sama. (Koreksi saat cek ulang: putaran 1 menulis "72"; hitungan baris di kedua file adalah 67.)
- **Register**: semua SRC yang dirujuk BRD (SRC-02, 03, 04, 06, 07, 08, 09, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23) terdaftar di `sources.md` dengan URL, tanggal akses, dan kutipan. L-01 s.d. L-11 terdefinisi di §6.3; PT-20 s.d. PT-26 di §10.2; CR-001 s.d. CR-004 di backlog; R-01, R-03, R-08 ada di Business Case.
- **Tingkat keyakinan regulasi tidak melebihi sumber**: status "rancangan" untuk SRC-02/SRC-03 disebut di §6.1 dan `sources.md`; regulasi yang hanya terbaca metadata ditulis Sedang/Rendah atau isi pasal `[TIDAK DIKETAHUI]`; kewajiban yang keberlakuannya belum pasti (registrasi Kemenkes, PSEF, DPO, izin BI) diberi "bila diwajibkan"/"dikonfirmasi" dan kolom konfirmasi legal.
- **URL dibuka ulang oleh fact-checker (T1)**, semuanya mendukung klaim:
    - SRC-15 (UU 27/2022, JDIH Komdigi): Pasal 4(2) data kesehatan, data anak, data keuangan pribadi; Pasal 20(2)a "persetujuan yang sah secara eksplisit"; Pasal 21(1) informasi wajib termasuk tujuan dan "jangka waktu retensi"; Pasal 22(1) "tertulis atau terekam"; Pasal 25 persetujuan orang tua/wali; Pasal 30(1) dan 32(2) 3 x 24 jam; Pasal 34 penilaian dampak bila risiko tinggi, termasuk "pemrosesan atas Data Pribadi yang bersifat spesifik"; Pasal 46(1) dan (3); Pasal 53(1) penunjukan pejabat/petugas PDP (bersyarat, sehingga kolom "Kewajiban bagi Rumah Sehat (L-08)" di BR-111 tepat); Pasal 56(1)–(4) transfer bersyarat. Mendukung BR-103 s.d. BR-113.
    - SRC-20 (ringkasan PBI 23/6/PBI/2021): larangan aktivitas sistem pembayaran tanpa izin/penetapan BI; Pasal 35 aspek prudensial bagi Penyelenggara Penunjang yang meneruskan pembayaran dari PJP, termasuk "jangka waktu penampungan dana". Mendukung BR-115 dan L-04.
    - SRC-12 (Permenkominfo 5/2020): Pasal 2(1) wajib daftar; Pasal 2(3) sebelum sistem mulai digunakan; Pasal 7(2) *access blocking*. Mendukung BR-100.
    - SRC-08 (portal PSEF): syarat NIB, Tanda Daftar PSE, IUI/PMSE, apoteker penanggung jawab, komitmen kerja sama apotek, daftar apotek jaringan; rujukan FAQ Permenkes 11/2025. Mendukung BR-116, L-03, PT-21.
    - SRC-06 (Kemenkes, 10/11 Nov 2026): akan mengatur seluruh platform kesehatan digital, telemedisin terintegrasi rekam medis, SIP, "penetapan tarif pelayanan kesehatan digital"; tanpa nomor peraturan. Mendukung BR-118, L-10, risiko baru §9.

## Penilaian butir yang ditandai penyusun "perlu dikonfirmasi user di gate"
- **BR-080, perubahan pengaturan markup dan biaya rekanan di audit trail** (baris 241, catatan baris 244): bukan keputusan yang disembunyikan, dan dibawa ke gate. Menambah event audit bersifat konservatif dan punya dasar sah di ADR-005/`CLAUDE.md` ("setiap aksi sensitif"). Penandaan **cukup**, tetapi alasan dan label perlu dirapikan (non-blokir butir 1).
- **Catatan PT-06, pembayaran nakes oleh klinik rekanan** (baris 373, 381): turunan langsung T2-Q2 (nakes atas nama klinik) dan T2-Q4 (bagian klinik masuk rekening klinik, PT. Arunika tidak menampung dana). Hanya berada di status PT, bukan BR. Penandaan **cukup**; dasar sebaiknya ditulis T2-Q2 + T2-Q4.
- **Portal untuk klinik rekanan yang menjual produk** (baris 70): kalimat ini menunda keputusan ke PRD, bukan mengambilnya, sehingga bukan kategori blokir 4. Namun kalimatnya tanpa label dan tidak menyebut dasar yang sudah ada (non-blokir butir 5).

## Temuan
Tidak ada temuan blokir.

| # | Lokasi (file, bagian) | Klaim | Masalah | Usulan |
| --- | --- | --- | --- | --- |
| 1 (non-blokir) | `02-brd.md` §5.9 BR-080 (baris 241) dan catatan (baris 244); `traceability.md` baris 54; `backlog.md` CR-004 "Dampak" (baris 60) | "Perubahan pengaturan markup dan biaya rekanan termasuk butir yang dicatat" `[USER] T2-Q6`; catatan: penerapan butir (5) dan "setiap transaksi" | T2-Q6 hanya menetapkan keberadaan pengaturan markup, tidak menyebut audit trail. Perubahan pengaturan bukan "pembayaran/refund/pembagian dana" (butir 5) maupun "transaksi". CR-004 menulisnya seolah bagian dari keputusan yang diterima user. Dasar yang sah sudah ada di baris yang sama: ADR-005/`CLAUDE.md` (setiap aksi sensitif), karena markup menentukan harga yang dibayar pasien (T1) | Ubah alasan di catatan menjadi "aksi sensitif menurut ADR-005; keberadaan pengaturan dari T2-Q6". Di CR-004 tulis "audit trail: usulan, dikonfirmasi di gate". Pastikan butir ini masuk rekap gate |
| 2 (non-blokir) | `02-brd.md` §5.5 BR-044 (baris 209), BR-045 (baris 210); §6.3 L-04 (baris 302) | Biaya transaksi payment gateway dibebankan ke pasien; refund bila layanan gagal | Celah di alur uang (T1), bukan klaim yang salah. (a) BR-044 tidak masuk daftar konfirmasi legal. Media sekunder (mis. bisnis.com, kontan.co.id) memberitakan BI melarang merchant membebankan biaya MDR QRIS ke konsumen. Ini **belum diverifikasi** ke sumber primer: ringkasan resmi PBI 23/6 (SRC-20) tidak memuatnya, jadi tidak boleh dikutip sebagai fakta. (b) Dengan pembagian dana langsung (BR-041), siapa yang mendanai refund (rekanan mengembalikan bagiannya? markup dan biaya PG ikut dikembalikan?) belum menjadi pertanyaan terbuka bisnis. L-04 hanya mencakup sisi hukumnya | Tambahkan BR-044 ke L-04 ("bolehkah biaya PG dibebankan ke pasien, per metode pembayaran") dan verifikasi ke sumber primer BI saat riset. Tambahkan PT baru: pendanaan dan komponen refund setelah dana terbagi (pemilik CEO, tenggat Tahap 3) |
| 3 (non-blokir) | `02-brd.md` §1.2 KPI-02 (baris 33), BR-090 (baris 250) | KPI-02 menghitung dokter dan psikolog "**terverifikasi dan aktif**" `[USER] T2-Q11` | User memilih "Sama syarat go-live". Syarat go-live di brief butir 2: "terverifikasi dan siap melayani". Kata "aktif" bisa terbaca sebagai opsi yang **tidak dipilih** user ("aktif melayani minimal satu transaksi per bulan"), sehingga bisa mengubah angka KPI saat diukur | Tulis "terverifikasi dan siap melayani (akun aktif, tanpa syarat jumlah transaksi)", sama dengan brief P8.2. Samakan di BR-090 |
| 4 (non-blokir) | `02-brd.md` §9 baris "Model pendapatan" (baris 356) | Rumus Tabel A–B Business Case "tetap berlaku secara matematis" bila komisi dibaca sebagai markup terhadap harga rekanan | Hanya benar bila "nilai transaksi" berarti total harga rekanan **sebelum** markup. Kalau dibaca sebagai total yang dibayar pasien, pendapatan = m/(1+m) × nilai (markup 20% → Rp16,7 juta per Rp100 juta, bukan Rp20 juta). Rumus juga tidak berlaku untuk markup nominal | Tambahkan syarat "nilai transaksi = total harga rekanan sebelum markup; markup nominal dihitung terpisah" |
| 5 (non-blokir) | `02-brd.md` §2.2 kalimat terakhir (baris 70) | "Klinik rekanan yang menjual produk memakai portal admin klinik atau admin outlet; pembagiannya ditetapkan di PRD" | Tanpa label. Jawaban intake PS3 sudah mencatat "pesanan obat bebas dikelola admin klinik", dan BR-065 sudah mengecualikan klinik dari portal outlet. Membuka ulang pilihan tanpa menyebut dasar ini berisiko bergeser dari keputusan yang lulus gate | Rujuk `[DOK] PS3`: default admin klinik. Kalau PRD ingin memakai portal outlet, bawa ke user (CR bila mengubah PS3) |

Sumber sekunder yang dibaca untuk butir 2 (hanya sebagai petunjuk, bukan bukti): hasil pencarian "Bank Indonesia QRIS merchant dilarang membebankan MDR surcharge" (bisnis.com, kontan.co.id, idxchannel.com), 2026-11-17. Ringkasan resmi BI https://www.bi.go.id/id/publikasi/peraturan/Pages/PBI_230621.aspx dibuka ulang dan tidak memuat larangan surcharge oleh merchant.

## Cek ulang (2026-11-17)

Cakupan: hanya baris yang berubah sejak putaran 1 (aturan-kerja §2), yaitu perbaikan lima non-blokir dan perubahan setelah review kepatuhan (blokir B-1) beserta jawaban user T2-Q12. Nomor baris di bawah mengacu ke `docs/02-brd.md` versi cek ulang.

**Penutupan non-blokir putaran 1**

| # | Status | Bukti |
| --- | --- | --- |
| 1 | Ditutup | BR-080 (baris 241) kini tanpa `[USER] T2-Q6` dan menulis "diusulkan ikut dicatat". Catatan (baris 244) menyebutnya **usulan penyusun** berdasarkan ADR-005, "tidak disebut user di P5.4; dikonfirmasi user di gate". CR-004 di backlog (baris 60) sudah disesuaikan. Traceability baris 54 sudah sama |
| 2 | Ditutup | BR-044 masuk L-04 (baris 302); L-12 baru (baris 310); PT-27 pendanaan refund (baris 399) |
| 3 | Ditutup | KPI-02 (baris 33) "terverifikasi dan siap melayani … `[DOK] P8.2`"; BR-090 (baris 250) sudah sama |
| 4 | Ditutup | §9 (baris 359): setara hanya untuk markup persen dengan nilai transaksi = harga rekanan sebelum markup; markup nominal = nominal × jumlah transaksi. Aritmetikanya benar |
| 5 | Ditutup | Baris 70: "Pesanan obat bebas di klinik rekanan dikelola admin klinik `[DOK] PS3`"; pembagian fitur portal ditunda ke PRD |

**Perubahan baru yang diperiksa**
- **T2-Q12** (`02-catatan-brd.md` baris 71–77): ada, dengan konteks, opsi A/B/C, dan jawaban "C. QRIS rekanan, lain pasien". Pertimbangan pewawancara ditulis sebagai "bukan keputusan". Keputusan ada di tangan user.
- **BR-044** (baris 209): sesuai Q12, yaitu MDR QRIS dipotong dari bagian rekanan, dan metode lain dibebankan ke pasien menunggu L-12. Label `[USER] T2-Q8, T2-Q12` `[SRC-24]` sah. Traceability baris 42 sama. Q12 mengoreksi Q8, jadi bukan kontradiksi dengan dokumen yang sudah lulus gate. Business Case §6.1 sudah ditangani di §9 (baris 360).
- **SRC-24** dibuka ulang (https://www.bi.go.id/id/publikasi/ruang-media/cerita-bi/Pages/mdr-QRIS.aspx). Halaman memuat "biaya MDR ini ditanggung oleh merchant dan tidak boleh dibebankan kepada konsumen", merujuk Siaran Pers No. 28/159/DKom (17 Agustus 2026), terbit 13 Oktober 2026, diperbarui 19 Oktober 2026, dan hanya membahas QRIS. Entri di `sources.md` akurat. Dengan begitu, baris 6.1 "ketentuan MDR non-QRIS `[TIDAK DIKETAHUI]`" (baris 265) tepat.
- **SRC-25** dibuka ulang (https://setkab.go.id/?p=176339). Berita 4 Desember 2019 mencatat PP 80/2019 ditandatangani 20 November 2019, dengan retensi data PMSE terkait transaksi keuangan "paling singkat 10 (sepuluh) tahun" dan data lainnya "paling singkat 5 (lima) tahun". BR-112 (baris 285) dan 6.1 PMSE (baris 266) menyatakan "status terkini belum diverifikasi" dengan keyakinan Rendah, sehingga tidak melebihi sumber.
- **6.1 PSEF** (baris 264) kini menyatakan dasar hukum syarat PSEF belum terverifikasi karena FAQ tidak terbaca. Ini sesuai SRC-08 dan catatan "Pencarian tanpa hasil".
- **L-07 diperluas** (baris 305), **L-12 s.d. L-14** (baris 310–312), PT-20 menjadi "L-01 sampai L-14" (baris 392), dependensi "L-01 sampai L-14" (baris 338): semuanya terdefinisi dan konsisten. L-13 dan L-14 hanya berupa pertanyaan legal, bukan keputusan.
- **Traceability dibuat ulang**: 67 baris BR, sama dengan 67 BR di BRD. Label BR-044, BR-080, BR-090, dan BR-112 cocok dengan BRD.

**Temuan cek ulang**: tidak ada blokir.

| # | Lokasi | Masalah | Usulan |
| --- | --- | --- | --- |
| R-1 (non-blokir) | `02-brd.md` P-03 (baris 90) dan P-07 (baris 94) | Rumus harga sudah memuat "biaya transaksi untuk metode non-QRIS", tetapi label sumber masih `T2-Q8` saja. Aturan non-QRIS berasal dari T2-Q12 | Tambahkan `T2-Q12` dan `[SRC-24]` pada kedua baris |
| R-2 (non-blokir) | `02-catatan-brd.md` baris 73 | Tanggal "(berlaku 1 Oktober 2026)" ditempelkan pada kalimat "ditanggung oleh merchant…". Di SRC-24, tanggal itu berlaku untuk skema tarif MDR QRIS, bukan untuk kalimat beban merchant. BRD 6.1 (baris 265) sudah benar ("kebijakan MDR QRIS (berlaku 1 Oktober 2026)") | Opsional: rapikan konteks catatan. Tidak mengubah keputusan |

**Status akhir: LULUS.** Semua non-blokir putaran 1 sudah ditutup, blokir B-1 dari review kepatuhan sudah dijawab user lewat T2-Q12 dengan label dan sumber yang sah, dan tidak ada blokir baru. R-1 dan R-2 tidak memicu putaran ulang.
