# Fact-check Tahap 03: PRD + NFR (dan BRD v1.2, CR-006)

Tanggal: 2026-11-17
Dokumen diperiksa: `docs/03-prd.md` (v1.0 draf); bagian `docs/02-brd.md` yang berubah ke v1.2 (§1.3, §2.2, §4.1–§4.3, BR-002, BR-007, BR-008, BR-017, BR-018, BR-027, BR-045, BR-064, BR-070, BR-100, §5.11 BR-120–BR-126, §6.3 L-02/L-05/L-07/L-08/L-14/L-15, §8, §9, §10); `docs/backlog.md` (indeks epic/story, CR-006); `docs/traceability.md` (kolom Story); `docs/decision-log.md` (KEPUTUSAN-03–KEPUTUSAN-16); `docs/sources.md` (SRC-26–SRC-29, perluasan SRC-15, SRC-18, SRC-27, SRC-28)
Sumber kebenaran: `docs/03-catatan-prd.md` (T3-Q1–T3-Q16), `docs/02-catatan-brd.md`, `docs/00-project-brief.md`, `docs/01-business-case.md` (PT-18), ADR-005
Tingkat ketelitian: T1 untuk §3.1, §3.3–§3.5, EP-08, EP-13, EP-15, EP-16, §6.4–§6.6, §7; T2 untuk bagian lain
Hasil: **TIDAK LULUS** (2 blokir: FC3-B01, FC3-B02; 5 non-blokir)

Catatan metode:
- Fact-checker tidak punya akses shell, jadi BRD v1.1 di `main` tidak bisa di-diff. Bagian BRD yang berubah ditentukan dari penanda CR-006, T3-Q, dan v1.2, lalu dicocokkan dengan catatan wawancara.
- `docs/03-prd.md` berubah selama pemeriksaan, kemungkinan karena perbaikan hasil cek ulang QA. Yang berubah antara lain RF-14, RF-18 (kini memakai UP-56 dan PB-20), AB-34 baris platform, dan AB-43. Nomor baris di laporan ini mengacu ke salinan yang dibaca terakhir, sesaat sebelum laporan ditulis. Bagian yang berubah sudah dibaca ulang.

## Ringkasan
- Total klaim: sekitar 580 klaim berlabel, dihitung per label. Rinciannya: PRD 98 baris `[USER]`, 29 `[SRC]`, 37 `[KEPUTUSAN]`, sekitar 150 `[UP]` di luar §11, 41 `[TIDAK DIKETAHUI]`, 97 story dengan induk BR, dan sekitar 70 aturan/NFR/entitas berlabel BR, ADR, atau `[DOK]`. Ditambah sekitar 35 baris BRD v1.2 yang berubah dan sekitar 25 entri register.
- Terverifikasi: sekitar 578
- Tanpa sumber: 0
- Sumber tidak mendukung: 2 (FC3-B01, FC3-B02)
- Kontradiksi: 0 yang blokir. Ada 1 ketegangan dengan rumusan terbaru user, dicatat non-blokir (FC3-N02)
- Fakta bisnis berlabel asumsi/keputusan: 0. Tidak ada `[ASUMSI]`. KEPUTUSAN-03–KEPUTUSAN-16 semuanya teknis atau proses.
- TANPA INDUK: 0

## Hal yang diperiksa dan bersih
- **Angka dan aturan `[USER] T3-Qx` cocok dengan `03-catatan-prd.md`:**
    - PB-01 24 jam untuk semua jenis layanan (T3-Q5);
    - PB-03 15 menit untuk konsultasi, PB-04 2 x 24 jam setelah barang diterima, PB-05 3 hari kerja, refund otomatis plus klaim lewat admin (T3-Q8);
    - definisi no-show dan "rekanan tetap menerima bagiannya" (T3-Q6);
    - nominal refund RF-01, RF-04, RF-05, RF-08, RF-09, RF-10, baris AB-34 untuk pembatalan pasien dan penyebab platform, serta penanggung kurir yang ditulis `[TIDAK DIKETAHUI]` karena memang tidak tercakup T3-Q7;
    - 99,0%, sekitar 7,2 jam per 30 hari, pemeliharaan diumumkan 48 jam sebelumnya di jam sepi, gangguan pihak ketiga dicatat terpisah (T3-Q12);
    - 30.000, 1.000, dan 100 sebagai target desain dan uji beban (T3-Q11);
    - paket standar: 500 ms, 1 detik, 2 detik, 3 detik, 1 menit, RPO 15 menit, RTO 4 jam, beserta pengecualian video dan notifikasi di perangkat (T3-Q13);
    - zona waktu AB-40 dan AB-41 (T3-Q10);
    - urutan pengurangan scope (T3-Q2), dan RME minimalis yang tidak diberi nomor (catatan T3-Q2, UP-33);
    - anggota keluarga: dewasa dengan KTP, anak dengan KK, pernyataan persetujuan (T3-Q3, T3-Q3a);
    - tarif, durasi, dan jadwal konsultasi di menu portal sebagai pengecualian (T3-Q4, T3-Q16);
    - RME minimalis terpisah dan terhubung lewat integrasi, hanya untuk pasien Rumah Sehat (T3-Q1b, T3-Q1c, T3-Q14, T3-Q15).

    AB-64 (RME dihitung "terintegrasi") sudah ditandai sebagai konsekuensi yang dimintakan konfirmasi di gate. Hitungan "104 hari" (2026-11-17 sampai 2027-03-01) dan "1 mobile developer" (P7.3) benar.
- **Keterlacakan**: 97 story di PRD (EP-01–EP-17). Semuanya punya induk BR yang ada di BRD v1.2, dan induknya sama di tabel epic PRD, di `backlog.md`, dan di kolom Story `traceability.md`. Tidak ada TANPA INDUK.
- **UP**: semua UP yang dirujuk (UP-01–UP-56) terdaftar di §11. Tidak ada UP yang bertentangan dengan jawaban user atau dengan BRD yang lulus gate. UP-08, UP-19, UP-24, dan UP-45 sudah dicek terhadap BR-045, BR-005, KPI-01, dan BR-046.
- **KEPUTUSAN**: semua nomor yang dirujuk (03–16) terdaftar di `decision-log.md`. Tidak ada yang dipakai untuk fakta bisnis.
- **SRC**: semua SRC yang dirujuk (07, 15, 18, 24, 25, 26, 27, 28, 29) terdaftar. Klaim penting berikut sudah dibuka ke sumbernya dan didukung:
    - **SRC-27**: nomor PSE wajib; nomor afiliasi SNOMED CT; kode registrasi fasyankes sesuai KMK, atau surat pernyataan resmi bila belum bermitra; tiga tahap (kategori institusi, data sistem RME, survei keamanan); lalu verifikasi. Klaim ini dipakai di US-1309 AC3, §9, §10, PT-32, BR-100, BR-123, dan backlog CR-006. Halaman tidak menyatakan registrasi ke SATUSEHAT itu sendiri wajib, dan PRD maupun BRD juga tidak mengklaimnya.
    - **SRC-28**: tepat 29 kelompok data. Kelompok UP-27 (anamnesis, pemeriksaan fisik, laboratorium, diagnosis, tindakan, peresepan, resume) termasuk di dalamnya. Halaman tidak menetapkan kelompok minimal, sehingga `[TIDAK DIKETAHUI]` di US-1306 AC2 jujur.
    - **SRC-26**: "SATUSEHAT menggunakan HL7 FHIR...", dan Practitioner tercantum sebagai *prerequisite* (US-1302 AC2).
    - **SRC-29**: ASVS 5.0.0, "Published 30 May 2025" (NFR-SEC-01, KEPUTUSAN-05).
    - **SRC-15**: Pasal 7, 18 ayat (2) huruf a, 21 ayat (1) huruf a–g dan ayat (2), 24, 30 ayat (1), 32 ayat (1)–(2), 40 ayat (2), 41 ayat (1), dan 51 ayat (1) dan (5). Pasal-pasal ini mendukung US-104 AC1/AC4, US-1603 AC4, US-1604 AC1, UP-15, UP-49, dan L-08.
    - **SRC-18**: "Fasyankes wajib terhubung ... dengan SATUSEHAT" (UP-28, US-1309).
- **Tidak ada klaim "terbukti" atau "teruji"** yang melebihi bukti. "Lulus" di NFR-CAP-04 dan NFR-DR-03 adalah syarat yang akan diuji, bukan klaim hasil.

## Temuan blokir

| ID | File dan baris | Klaim | Masalah | Kategori §2 | Usulan perbaikan |
| --- | --- | --- | --- | --- | --- |
| FC3-B01 | `docs/03-prd.md` §3.5 AB-34 baris 226, kelompok "Penyebab rekanan". Ikut terbawa ke US-806 AC3 (baris 1034), US-807 AC3, US-808 AC3 | Kewajiban rekanan = "Harga rekanan penuh + biaya payment gateway yang tidak kembali" `[USER] T3-Q7`. Baris 221 mendefinisikan biaya payment gateway yang tidak kembali mencakup MDR | T3-Q7 (catatan baris 58): "Bagian rekanan dikembalikan rekanan; markup dikembalikan Rumah Sehat; biaya payment gateway yang tidak kembali ditanggung pihak penyebab". Bagian rekanan adalah harga rekanan dikurangi MDR bila QRIS (AB-22). "Harga rekanan penuh" sudah memuat porsi MDR, lalu MDR ditambahkan lagi lewat "biaya payment gateway yang tidak kembali". Contoh QRIS: harga rekanan Rp100.000, markup Rp10.000, MDR 0,7% x Rp110.000 = Rp770 (tarif UKE/UME/UBE, SRC-24), sehingga rekanan menerima Rp99.230. Pasien menerima Rp110.000. Menurut T3-Q7, rekanan menanggung 99.230 + 770 = Rp100.000 dan Rumah Sehat Rp10.000, total Rp110.000. Menurut AB-34, rekanan menanggung 100.000 + 770 = Rp100.770, total Rp110.770, lebih besar dari dana yang dikembalikan. Untuk non-QRIS kedua rumus sama hasilnya. Baris "Penyebab platform" (227) sudah benar, karena memakai "bagian yang benar-benar diterimanya". Rumusan baris 226 berasal dari usulan QA-B04 | 1. Label `[USER] T3-Q7` tidak mendukung rumus itu, dan nominal kewajiban rekanan berubah (T1, alur uang) | Ubah kolom kewajiban rekanan di baris 226 menjadi "Bagian rekanan yang diterimanya (AB-22) + semua biaya payment gateway yang tidak kembali". Hasilnya sama dengan harga rekanan penuh ditambah biaya transaksi non-QRIS yang tidak kembali. Kalau penyusun memang bermaksud rumus lain, bawa ke user sebagai UP. Beri tahu QA agar cek ulang QA-B04 selaras. Cek ulang cukup pada baris 221–227 dan US-806 AC3 |
| FC3-B02 | `docs/02-brd.md` §2.2 baris 74 (perubahan v1.2, CR-006) | "Aplikasi RME minimalis ... dipakai staf klinik dan lab rekanan yang belum punya sistem, serta operator PT. Arunika untuk onboarding. ... `[USER] T3-Q15`" | T3-Q15 (catatan baris 95–97) hanya memutuskan bahwa RME minimalis dibangun di proyek ini sebagai aplikasi terpisah untuk rilis 1. Tidak ada jawaban user di T3-Q1–T3-Q16 tentang operator PT. Arunika. Di PRD, peran yang sama sudah benar diberi label `[UP-26]` (§1.2 baris 69, US-1301). Pengguna adalah fakta bisnis (anti-halu §1 butir 3). Peran ini juga substantif: staf PT. Arunika masuk ke sistem rekam medis fasyankes, yang berkaitan dengan L-05 dan L-15 | 1. Fakta bisnis (pengguna) berlabel `[USER]` yang tidak sah | Ubah menjadi "...yang belum punya sistem `[USER] T3-Q15`. Peran operator PT. Arunika untuk onboarding diusulkan di PRD (UP-26) dan diputuskan di gate Tahap 3", atau hapus frasa operator dari BRD. Cek ulang cukup pada baris 74 |

## Temuan non-blokir (5 butir terpenting)

| ID | Lokasi | Masalah | Usulan |
| --- | --- | --- | --- |
| FC3-N01 | `03-prd.md` §2.2 baris 93; AB-50 baris 369 dan §8.3 baris 1829; RF-05 baris 200 | Ada tafsiran penyusun yang diberi label `[USER]` atau BR tanpa UP. (a) Jadwal ulang ditulis Won't dengan label `[USER] T2-Q17, T3-Q6`. Keduanya pertanyaan pilihan tunggal tentang refund dan no-show; user tidak pernah ditanya apakah jadwal ulang disediakan. (b) AB-50 menulis "satu klinik", padahal BR-005 dan T2-Q2 tidak menyebut jumlah. Aturan ini menentukan apakah dokter yang praktik di dua klinik rekanan perlu dua akun, sehingga ikut memengaruhi KPI-02. (c) RF-05 menggolongkan penonaktifan akun rekanan atau tenaga kesehatan oleh admin Rumah Sehat, serta SIP berakhir, sebagai "penyebab rekanan". T3-Q7 hanya menyebut "rekanan membatalkan". Transisinya sudah tercakup UP-10, tetapi penetapan penanggungnya belum | Sebut ketiganya di rekap gate, atau tambahkan ke §11 sebagai UP |
| FC3-N02 | `03-prd.md` US-1202 AC2, US-1203 AC1, AB-62; `02-brd.md` BR-064 baris 242 | User menulis portal rekanan "hanya perihal pelaporan atau rekap saja" (T3-Q14), dan catatan T3-Q16 menyebut "selebihnya portal hanya untuk laporan dan rekap". PRD mempertahankan dua fungsi tulis dari brief PS6c, yaitu menanggapi keluhan dan mengajukan perubahan profil, lewat UP-30 dan UP-31. Ini bukan kontradiksi dengan dokumen yang lulus gate, tetapi berbeda dengan rumusan terbaru user | Tanyakan eksplisit di gate: "Selain menu konsultasi online, apakah portal boleh dipakai untuk menanggapi keluhan dan mengajukan perubahan profil?" |
| FC3-N03 | `03-prd.md` NFR-OBS-01 baris 1676; §2.4 baris 115; AB-42 baris 362 | Label kurang presisi. NFR-OBS-01 berlabel `[USER] T3-Q12`, padahal pemantauan otomatis dan peringatan adalah turunan teknis; user hanya memilih target dan cara ukur. "Antrean berprioritas Should" berlabel `[USER] T3-Q2`, padahal user menetapkan urutan, dan prioritasnya turunan yang tercakup UP-01. AB-42 menambahkan "backoffice" ke aturan WIB, sedangkan T3-Q10 hanya menyebut laporan Rumah Sehat | Ganti labelnya: KEPUTUSAN baru untuk NFR-OBS-01, `[UP-01]` untuk prioritas antrean, dan UP untuk backoffice WIB |
| FC3-N04 | `decision-log.md` KEPUTUSAN-11; `traceability.md` baris 10, 15, 24, 25, 45, 52, 54, 59; `02-brd.md` §8 baris 381, §7.3 baris 366, PT-20 baris 425 | Register dan rujukan silang belum sinkron. KEPUTUSAN-11 masih menyebut "PB-01–PB-07", berbeda dengan AB-43. Kolom ringkasan dan sumber di traceability belum memuat label T3 atau SRC-27 v1.2 (misalnya BR-018 masih "Tarif konsultasi berasal dari rekanan" `[USER] T2-Q6`, dan BR-100 belum memuat SRC-27). §8 CR-006 belum menyebut BR-027, BR-100, L-02, L-08, L-14, dan L-15. §7.3 dan PT-20 masih menulis "L-01 sampai L-14" padahal L-15 sudah ditambahkan | Samakan semua rujukan itu |
| FC3-N05 | `sources.md` SRC-15 baris 23 | Pasal 18 ayat (2) huruf a dan Pasal 51–52 dicatat "menurut laporan compliance/tahap-03.md", bukan sebagai kutipan langsung dari halaman sumber. Saya sudah membuka halamannya dan isinya mendukung UP-49: Pasal 18 ayat (2) huruf a mengatur perjanjian antar-pengendali; Pasal 51 ayat (1) mengatur pemrosesan berdasarkan perintah pengendali, dan ayat (5) mewajibkan persetujuan tertulis pengendali sebelum melibatkan prosesor lain | Ganti dengan kutipan langsung dari halaman sumber beserta tanggal akses |

## URL yang benar-benar dibuka (2026-11-17)
1. https://satusehat.kemkes.go.id/platform/docs/id/registration-guide/regis-vendor/ (SRC-27)
2. https://satusehat.kemkes.go.id/platform/docs/id/interoperability/rme-rawat-jalan/ (SRC-28). Hanya 100.000 karakter pertama yang dibaca; daftar 29 kelompok data ada di bagian itu
3. https://satusehat.kemkes.go.id/platform/docs/id/fhir/ (SRC-26)
4. https://asvs.owasp.org/ (SRC-29)
5. https://jdih.komdigi.go.id/produk_hukum/view/id/832/t/undangundang+nomor+27+tahun+2022 (SRC-15)
6. https://www.kemkes.go.id/id/%20fasyankes-wajib-terapkan-rekam-medis-elektronik (SRC-18)

## Butir yang perlu disebut eksplisit di gate (di luar §11)
- AB-64: klinik/lab pengguna RME minimalis dihitung "terintegrasi". Sudah ditandai di PRD dan BRD §1.3.
- FC3-N01 dan FC3-N02 di atas.
