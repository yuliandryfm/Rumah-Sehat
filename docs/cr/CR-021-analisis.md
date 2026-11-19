# CR-021: Analisis — Web Ruang Sehat (company profile) dan member area layanan korporat

Tanggal: 2026-11-19
Versi: 0.3 (2026-11-19: keputusan user putaran 4 `[USER] C21-P4` — S3, K-01–K-12 sesuai usulan, UP-79–UP-87 disetujui, PT-67 Verifikator, PT-79 bukti kewenangan wajib, PT-72 tolak NIB ganda; ADR-027/028 Diterima; koreksi fact-check N-2, N-5; isi diterapkan ke BRD v1.4, PRD v1.3, data dictionary, phase plan v0.3. Dokumen gate kini menjadi sumber kebenaran; berkas ini catatan CR). Versi 0.2 (perbaikan review putaran 1: `docs/review/CR-021-kepatuhan.md` B-1 dan `docs/review/CR-021-security.md` B-01–B-05 beserta sebagian catatan non-blokir; draf untuk persetujuan user; belum mengubah dokumen gate kecuali threat model v1.3). Versi 0.1: draf awal
Penyusun: Claude (persona Business Analyst + Product Manager + Solution Architect + Project Manager), Mode Otomatis
Pemilik dan pemutus: user (project manager, meneruskan ke CEO dan CTO) `[DOK: 00-project-brief.md]`
Tingkat ketelitian: **T1** untuk data karyawan dan hasil kesehatan (§3 BR-143–BR-145, §4 US-1907, US-1908, US-1912–US-1915, §5, §6.3–§6.4), autentikasi dan otorisasi akun korporat (US-1901–US-1904, §6.2–§6.3), dan penagihan (US-1905, US-1906, US-1909, US-1910, §6.5). **T2** untuk bagian lain.
Dasar: wawancara CR-021 putaran 0–3 `[DOK: cr/CR-021-wawancara.md]`, backlog CR-021 dan CR-015 `[DOK: backlog.md]`, BRD v1.3 `[DOK: 02-brd.md]`, PRD v1.2 final `[DOK: 03-prd.md]`, arsitektur v1.0 `[DOK: 04-arsitektur.md]`, ADR-006, ADR-011–ADR-016, ADR-018, ADR-021, ADR-024–ADR-026 `[DOK: adr/]`, data dictionary `[DOK: 04-data-dictionary.md]`, stack `[DOK: 05-stack.md]`, design guideline v0.1 `[DOK: 06-design-guideline.md]`, phase plan v0.2 `[DOK: 07-phase-plan.md]`, catatan Fase 1 `[DOK: fase-1-catatan.md]`.

**Label sumber.** `[USER] Px-y` adalah jawaban user di `docs/cr/CR-021-wawancara.md` (Telegram, 2026-11-19). `[USER] C2-Qx`/`C2-Bn` dan `[USER] Tx-Qy` mengikuti konvensi PRD §0.1. `[DOK: ...]` dokumen proyek yang lulus gate. `BR-`, `US-`, `AB-`, `PB-`, `NFR-`, `E-`, `KU-` lama merujuk ke BRD/PRD dan berfungsi sebagai label `[DOK]`. Nomor **baru** di dokumen ini (BR-130 dst., EP-18, EP-19, US-18xx, US-19xx, AB-80 dst., PB-23 dst., E-18, UP-79 dst., RT-38 dst., PT-59 dst., L-17 dst., ADR-027 dst.) adalah **usulan** yang baru berlaku setelah user menyetujuinya. `[UP-xx]` usulan penyusun untuk detail produk yang belum diputuskan user. `[KEPUTUSAN-xx]` tercatat di `docs/decision-log.md`. `[TIDAK DIKETAHUI]` masuk §13. Angka jam-task adalah **estimasi** dengan dasar hitung di sampingnya, bukan fakta.

Sumber eksternal hanya yang tercatat di `docs/sources.md`: `[SRC-13]`, `[SRC-15]`, `[SRC-18]`, `[SRC-20]` (lewat BRD), `[SRC-246]`, `[SRC-265]`, `[SRC-276]`, `[SRC-50]`, serta yang dibuka reviewer putaran 1: `[SRC-405]`–`[SRC-409]` (OWASP XML/XXE/CSV Injection/File Upload, MDN `Set-Cookie`) dan `[SRC-440]`–`[SRC-441]` (teks UU PDP di JDIH Komdigi; metadata dan abstrak PER-02/MEN/1980). Isi regulasi yang belum diverifikasi diajukan sebagai konfirmasi legal (L-17–L-21).

**Perubahan v0.2 (ringkas).** Kepatuhan B-1: kolom Sensitif data peserta dan rekap kehadiran (§4.1). Keamanan B-01: render Markdown aman, pratinjau sandbox, CSP (US-1802 AC6–AC9, ADR-027 butir 5, F1-T111). B-02: jalur reset MFA HR (US-1904 AC7–AC8, ADR-027 butir 3, usulan revisi ADR-011 butir 5). B-03: invarian kanal turunan (ADR-028 butir 3, US-1909 AC6, US-1912 AC5, US-1914 AC7–AC8). B-04: pengikatan undangan (UP-83 menjadi syarat keamanan, App Links, PT-78, UP-87). B-05: mutasi bank dengan kunci unik global dan alokasi (§5, US-1910, ADR-029 butir 3). Ditambah K-12, PT-76–PT-79, perluasan L-17–L-21, KEPUTUSAN-264–269, dan pembaruan effort serta skenario (§8–§10).

---

## 0. Ringkasan

- **Keputusan user** `[USER] P0-1–P3-4`: situs company profile (sembilan halaman, konten lewat CMS US-1101) dan member area korporat dibangun di Fase 1, dengan konsekuensi Fase 1 mundur atau sebagian task Fase 1 pindah ke Fase 2. Login: HR/admin perusahaan dan karyawan peserta. Layanan: vaksinasi massal, MCU massal, lab massal oleh klinik/lab rekanan, on-site atau di rekanan. Pembayaran lewat invoice B2B tanpa payment gateway.
- **Usulan isi**: 16 BR baru (BR-130–BR-145), 2 epic baru (EP-18 Situs web, EP-19 Layanan korporat) dengan **19 story baru** (US-1801, US-1802, US-1901–US-1917), ditambah US-1101 (CMS) yang dipindah dari Fase 3 ke Fase 1. Usulan data: satu bagian data dictionary baru (A20, 24 tabel) dan perluasan `account`, `cms_content`, `data_key`, `journal_account`.
- **Arsitektur**: dua aplikasi Next.js baru (`web/situs` publik tanpa sesi, `web/korporat` member area HR dengan BFF), realm Keycloak baru `korporat`, relasi otorisasi baru "tenant perusahaan", persetujuan berbagi ringkasan per pesanan, dan penagihan B2B dengan jurnal berpasangan tanpa adaptor payment gateway. Tiga ADR baru diusulkan (ADR-027, ADR-028, ADR-029) dan tujuh ADR yang sudah diterima perlu revisi (§7).
- **Effort tambahan bersih ±340–435 jam-task** (±19–24% dari baseline Opsi B ±1.791) `[KEPUTUSAN-260]`: bagian Fase 1 dihitung dari bawah (31 task, 109 jam-task, F1-T81–F1-T111), sisanya per story dengan kelas KEPUTUSAN-178. Naik dari ±306–399 di v0.1 karena perbaikan review (§8).
- **Tiga skenario jadwal** (§9): S1 Fase 1 diperpanjang 4 hari kerja; S2 Fase 1 tetap selesai 2026-12-11 dengan 7 task Fase 1 dipindah ke Fase 2 dan pesanan/peserta/invoice di Fase 2; S3 seperti S2, tetapi pesanan/peserta/invoice di Fase 3. Reset MFA HR oleh admin ada di Fase 2 di semua skenario. **Rekomendasi penyusun: S3**, karena paling sedikit menambah beban Fase 2 (konsultasi dan pembayaran, layanan yang dipertahankan paling akhir `[USER] T3-Q2`) dan menaruh penagihan setelah dependensi teknisnya tersedia. S3 berarti pemesanan korporat baru tersedia di Fase 3. Itu **menyimpang dari maksud "member area di Fase 1"** dan wajib diputuskan user. Semua skenario tetap menaikkan kebutuhan kecepatan di atas rencana, sehingga di gerbang re-plan 2026-12-11 perlu keputusan kompensasi (opsi scope, Opsi E, atau posisi layanan korporat di urutan pengurangan scope, PT-69).
- **Kontradiksi dengan dokumen gate** (§2): 12 butir. Yang mengubah keputusan: non-scope "versi web untuk pasien" (K-01), PT. Arunika tidak menampung dana mitra (K-02), semua layanan dibayar lewat payment gateway (K-03), portal rekanan hanya baca (K-04), vaksinasi ke rilis berikutnya (K-05), dan persetujuan sebelum pemrosesan BR-103 terhadap unggahan HR (K-12).
- **Pemblokir pembangunan**: penerima dana invoice B2B (PT-59), kanal peserta (PT-60), sumber harga katalog korporat (PT-61), dan konfirmasi legal L-17–L-19. Bagian Fase 1 yang diusulkan (situs, CMS, pendaftaran dan verifikasi perusahaan, akun HR) **tidak** terblokir oleh butir itu, kecuali PT-67 (sub-peran admin yang membuat perusahaan dan yang memulai reset MFA HR).

---

## 1. Ringkasan keputusan user

| Kode | Butir | Keputusan | Sumber |
| --- | --- | --- | --- |
| P0-1 | Penempatan | Company profile **dan** member area korporat dibangun di Fase 1 (pilihan Y); Fase 1 mundur atau sebagian task Fase 1 pindah ke Fase 2, rinciannya diajukan dulu | `[USER] P0-1` |
| P0-2 | Siapa login | HR/admin perusahaan + karyawan peserta (K) | `[USER] P0-2` |
| P1-1 | Halaman company profile | Beranda, Tentang Kami, Layanan (7 layanan + korporat), Untuk Perusahaan, Rekanan, Artikel/berita, Kontak, Unduh aplikasi, Kebijakan privasi & syarat | `[USER] P1-1` |
| P1-2 | Layanan korporat rilis ini | Vaksinasi massal, MCU massal, pemeriksaan lab massal (non-MCU) | `[USER] P1-2` |
| P1-3 | Pelaksana | Klinik/lab rekanan; Ruang Sehat sebagai perantara | `[USER] P1-3` |
| P1-4 | Pembayaran | Invoice B2B (transfer, termin/jatuh tempo), tanpa payment gateway | `[USER] P1-4` |
| P2-1 | Menjadi member | Daftar sendiri di web lalu diverifikasi admin Ruang Sehat (NIB/NPWP), **dan** dibuatkan admin/sales Ruang Sehat setelah kesepakatan | `[USER] P2-1` |
| P2-2 | Alur pesan | HR memilih paket dari katalog harga tetap, lalu langsung memesan (bukan penawaran) | `[USER] P2-2` |
| P2-3 | Data dan akun peserta | HR unggah Excel/CSV (nama, email/HP, tanggal lahir; NIK hanya bila wajib untuk layanan); peserta menerima undangan email dan memakai akun pasien yang sama | `[USER] P2-3` |
| P2-4 | Akses hasil | Hasil lengkap hanya untuk peserta; HR melihat rekap kehadiran; ringkasan (mis. fit/unfit) hanya dengan persetujuan peserta. Tafsiran penyusun "tanpa persetujuan, HR hanya melihat rekap kehadiran" tercatat sebagai tafsiran yang dikonfirmasi di rekap (status konfirmasinya dibawa ke PT-65) | `[USER] P2-4` |
| P3-1 | Harga katalog | Harga dari rekanan (integrasi/portal rekanan) + markup Ruang Sehat, mengikuti aturan markup ADR-015 | `[USER] P3-1` |
| P3-2 | Lokasi | On-site di kantor perusahaan atau di klinik/lab rekanan, dipilih per pesanan | `[USER] P3-2` |
| P3-3 | Syarat invoice bawaan | Terbit saat pesanan dikonfirmasi, jatuh tempo 14 hari, layanan dijadwalkan setelah lunas | `[USER] P3-3` |
| P3-4 | Pengelola konten | Admin Ruang Sehat lewat CMS (US-1101) | `[USER] P3-4` |

---

## 2. Kontradiksi dan ketegangan dengan dokumen gate

Butir K-01–K-05 dan K-12 mengubah keputusan, perilaku, atau peran yang sudah lulus gate (kriteria blokir `aturan-kerja.md` §2 butir 3), sehingga tidak diselesaikan penyusun dan dibawa ke user.

| # | Dokumen gate | Isi yang berbenturan | Keputusan CR-021 | Opsi | Pemutus |
| --- | --- | --- | --- | --- | --- |
| K-01 | BRD §4.2 non-scope "Versi web untuk pasien" `[DOK] P3.6a` | Peserta memakai akun pasien `[USER] P2-3`; peserta ikut login di member area web `[USER] P0-2` | Peserta login di web dengan akun pasien = versi web untuk pasien | (a) Peserta memakai aplikasi pasien (mobile) untuk undangan, persetujuan, jadwal, dan hasil; member area web hanya untuk HR. Konsisten dengan BRD. (b) CR atas non-scope: web terbatas untuk fungsi peserta korporat, dengan klien web baru di realm `pasien`. Menambah ±10–15 jam-task web dan memindahkan ±18 jam-task dari jalur mobile (estimasi penyusun dari porsi layar peserta di US-1908, US-1911, US-1913) | User (PT-60) |
| K-02 | BRD BR-041; PRD §1, AB-21; non-scope "Penampungan dana pasien di rekening PT. Arunika lalu diteruskan sendiri ke mitra" (CR-003) `[USER] T2-Q4` | PT. Arunika tidak menampung bagian rekanan | Invoice B2B dibayar transfer tanpa payment gateway `[USER] P1-4` | (a) Perusahaan membayar seluruh invoice ke rekening PT. Arunika, lalu PT. Arunika membayar rekanan (PT. Arunika menampung dana rekanan; bertentangan dengan BR-041, perlu CR dan legal L-21). (b) Dua tagihan: bagian rekanan dibayar langsung ke rekening rekanan, markup ke PT. Arunika. Opsi ini membuat perusahaan melihat bagian rekanan dan bagian PT. Arunika terpisah, sehingga markup terbuka. Itu bertentangan dengan AB-13 dan US-1905 AC2 ("markup tidak tampil sebagai baris tersendiri"), jadi bila dipilih, AB-13 perlu pengecualian korporat. (c) Virtual account dengan pembagian dana dari penyedia pembayaran (bertentangan dengan "tanpa payment gateway") | User + CEO + legal (PT-59, L-21) |
| K-03 | BRD BR-040, BR-115; PRD AB-20; ADR-015 konteks | Semua layanan dibayar di aplikasi lewat payment gateway berizin BI | Transfer bank langsung atas invoice `[USER] P1-4` | BR baru BR-141 sebagai pengecualian eksplisit untuk layanan korporat; L-04 diperluas | User (persetujuan BR-141) + legal L-21 |
| K-04 | BRD BR-064; PRD AB-62 | Portal rekanan hanya pelaporan dan rekap, ditambah satu menu pengaturan konsultasi | Harga korporat dari rekanan lewat "integrasi/portal rekanan" `[USER] P3-1` | (a) Lewat Partner Integration API (EN-21, Fase 3); (b) lewat portal rekanan (CR atas AB-62, menu baru); (c) sementara diinput admin Ruang Sehat atas nama rekanan sampai integrasi ada (belum pernah diputuskan user) | User (PT-61) |
| K-05 | Backlog CR-015 dan PRD §2.2 Won't: vaksinasi sebagai layanan lab ke **rilis berikutnya** `[USER] C2-B6` | Vaksinasi tidak dibangun di rilis 1 | Vaksinasi massal korporat di Fase 1 `[USER] P1-2` | Tafsiran penyusun: CR-021 hanya menambah vaksinasi lewat kanal korporat dan tidak membuka kembali CR-015 untuk aplikasi pasien. Pertanyaan regulasi dan pencatatan RME/SATUSEHAT CR-015 tetap `[TIDAK DIKETAHUI]` dan kini jatuh tempo lebih awal | User (PT-75), legal (L-19) |
| K-06 | Phase plan v0.2 §4, §8.2: US-1101 (CMS) di Fase 3 | CMS belum ada di Fase 1 | Konten company profile lewat CMS US-1101 `[USER] P3-4` | US-1101 dipindah ke Fase 1 (F1-T91–F1-T95) | User (persetujuan perubahan phase plan) |
| K-07 | ADR-006 butir 5 "Dua web frontend"; ADR-024 butir 2 "Tiga web"; PRD §1.1 komponen | Jumlah komponen web tetap | Situs publik + member area | Revisi ADR-006 dan ADR-024 lewat ADR-027 | User |
| K-08 | ADR-011 butir 3 (lima realm) | Tidak ada populasi HR perusahaan | HR login `[USER] P0-2` | Realm keenam `korporat` (§6.2, ADR-027) | User |
| K-09 | NFR-SEC-08 dan ADR-018: jenis berkas JPG/PNG dan PDF | CSV/XLSX tidak diizinkan | Unggah Excel/CSV `[USER] P2-3` | Revisi NFR-SEC-08 dan ADR-018 (ADR-028 butir 6) | User |
| K-10 | BRD §4.2 non-scope "Pengantaran sampel lab atau pengambilan sampel di rumah" `[DOK] P3.6a` | Pengambilan sampel di luar lab tidak dibangun | MCU/lab on-site di kantor perusahaan `[USER] P3-2` | Tafsiran penyusun: on-site di kantor bukan "di rumah", sehingga tidak bertabrakan langsung, tetapi butuh konfirmasi user dan legal (L-19) | User (PT-75 butir b) |
| K-11 | BRD BR-126: RME minimalis hanya mencatat pasien yang memesan lewat aplikasi Ruang Sehat | Pesanan dibuat pasien di aplikasi | Pesanan dibuat HR di web atas nama peserta | Perlu ketegasan apakah peserta korporat termasuk "pasien lewat Ruang Sehat" bagi rekanan pengguna RME | User (PT-74) |
| K-12 | BRD BR-103: pasien memberikan persetujuan eksplisit sebelum datanya diproses | Persetujuan sebelum pemrosesan | HR mengunggah data peserta sebelum peserta berinteraksi dengan Ruang Sehat `[USER] P2-3` | UU PDP mengenal enam dasar pemrosesan, dan persetujuan hanya salah satunya (Pasal 20 ayat (2) huruf a–f `[SRC-440]`). Dasar untuk data unggahan HR dan peran PT. Arunika terhadap perusahaan ditanyakan di L-17. BR-103 tetap berlaku untuk data setelah peserta tertaut ke akun pasien | Legal (L-17), lalu user |

Ketegangan lain yang tidak mengubah keputusan, tetapi perlu ditulis di PRD bila CR disetujui: AB-17 mengunci harga "saat pembayaran dibuat di payment gateway", padahal pesanan korporat tidak memakai payment gateway (usulan AB-81); matriks refund §3.5 tidak mencakup pembatalan pesanan korporat (PT-63); urutan pengurangan scope §2.3 belum memuat layanan korporat (PT-69); NFR-SEC-02 mendaftar jenis akun ber-MFA tanpa HR, tetapi frasanya "semua akun manusia" (KEPUTUSAN-253).

---

## 3. Kebutuhan bisnis baru (usulan BRD §5.12)

Penomoran melanjutkan pola blok BRD §5: BR-130 dst. untuk area baru "Situs web dan layanan korporat (CR-021)". Tingkat ketelitian: BR-138, BR-141–BR-145 T1.

| ID | Kebutuhan bisnis | Sumber |
| --- | --- | --- |
| BR-130 | PT. Arunika menyediakan situs web Ruang Sehat bergaya company profile dengan halaman Beranda, Tentang Kami, Layanan (tujuh layanan dan layanan korporat), Untuk Perusahaan, Rekanan, Artikel/berita, Kontak, Unduh aplikasi, serta Kebijakan privasi dan syarat | `[USER] P1-1` |
| BR-131 | Konten situs dikelola admin Ruang Sehat lewat CMS backoffice (memperluas BR-061) | `[USER] P3-4` |
| BR-132 | Situs memiliki member area untuk klien korporat. Yang masuk: HR/admin perusahaan dan karyawan peserta. Kanal peserta (web atau aplikasi pasien) menunggu PT-60 karena non-scope BRD §4.2 | `[USER] P0-2` |
| BR-133 | Layanan korporat rilis ini: vaksinasi massal, MCU massal, dan pemeriksaan lab massal (non-MCU) | `[USER] P1-2` |
| BR-134 | Layanan korporat dilaksanakan klinik/lab rekanan; Ruang Sehat berperan sebagai perantara | `[USER] P1-3` |
| BR-135 | Lokasi pelaksanaan dipilih per pesanan: on-site di kantor perusahaan atau di klinik/lab rekanan | `[USER] P3-2` |
| BR-136 | Perusahaan menjadi member dengan dua cara: (1) mendaftar sendiri di web, lalu diverifikasi admin Ruang Sehat dengan dokumen NIB/NPWP; (2) dibuatkan admin/sales Ruang Sehat setelah kesepakatan. Sub-peran admin yang berwenang `[TIDAK DIKETAHUI]` (PT-67) | `[USER] P2-1` |
| BR-137 | HR memesan dengan memilih paket dari katalog harga tetap dan langsung memesan, tanpa alur penawaran | `[USER] P2-2` |
| BR-138 | Harga katalog korporat = harga dari rekanan + markup Ruang Sehat, mengikuti aturan markup (BR-042). Jalur masuk harga rekanan `[TIDAK DIKETAHUI]` (PT-61); apakah besaran markup korporat sama dengan markup layanan pasien `[TIDAK DIKETAHUI]` (PT-62) | `[USER] P3-1` |
| BR-139 | HR mengunggah daftar peserta dalam berkas Excel/CSV berisi nama, email/nomor HP, dan tanggal lahir. NIK hanya diminta bila wajib untuk layanan itu. Layanan yang mewajibkan NIK `[TIDAK DIKETAHUI]` (PT-66) | `[USER] P2-3` |
| BR-140 | Peserta menerima undangan lewat email dan memakai akun pasien Ruang Sehat yang sama dengan pasien lain | `[USER] P2-3` |
| BR-141 | Pesanan korporat dibayar lewat invoice B2B (transfer bank, dengan jatuh tempo), tanpa payment gateway. Ini pengecualian atas BR-040 untuk layanan korporat. Rekening penerima dan hubungannya dengan BR-041 `[TIDAK DIKETAHUI]` (PT-59, L-21) | `[USER] P1-4` |
| BR-142 | Syarat invoice bawaan: invoice terbit saat pesanan dikonfirmasi, jatuh tempo 14 hari, dan layanan dijadwalkan setelah lunas. Apakah "14 hari" hari kalender atau hari kerja, dan apakah syarat dapat diubah per perusahaan, `[TIDAK DIKETAHUI]` (PT-63) | `[USER] P3-3` |
| BR-143 | Hasil lengkap MCU dan lab hanya dapat diakses peserta yang bersangkutan | `[USER] P2-4` |
| BR-144 | HR melihat rekap kehadiran peserta. Ringkasan hasil (misalnya fit/unfit) hanya tampil bagi HR bila peserta memberi persetujuan; tanpa persetujuan, HR hanya melihat rekap kehadiran (kalimat ini tafsiran penyusun atas P2-4, konfirmasi PT-65). Persetujuan dapat ditarik peserta (hak penarikan persetujuan, BR-106) | `[USER] P2-4`; BR-106 `[SRC-15]` |
| BR-145 | Data peserta dan hasil layanan korporat diperlakukan sebagai data pribadi, dan hasil serta ringkasannya sebagai data pribadi spesifik (BR-105). Data setiap perusahaan terisolasi dari perusahaan lain, dan setiap aksi sensitif tercatat di audit trail (BR-080, BR-107) | BR-105, BR-080, BR-107 `[DOK: 02-brd.md]`; `[USER] P2-4` |

**Usulan perubahan BRD lain** (diterapkan setelah persetujuan):
- §2.2 peran: tambah "HR/admin perusahaan" (member area web) dan "Karyawan peserta" (akun pasien) `[USER] P0-2`.
- §4.1 in-scope: S-20 situs web company profile (BR-130, BR-131); S-21 layanan korporat (BR-132–BR-145).
- §4.2 non-scope: butir "Versi web untuk pasien" disesuaikan sesuai jawaban PT-60.
- §6.2: BR-100 (PSE) diperluas ke situs dan member area (L-20).
- §6.3: L-17–L-21 (§13.2).

---

## 4. Epic, story, dan acceptance criteria (usulan PRD)

Kriteria umum KU-01–KU-10 berlaku tanpa ditulis ulang. Prioritas semua story baru **Must**, karena user menyatakan fitur ini "harus" masuk Fase 1 `[USER] P0-1`. Posisi di urutan pengurangan scope §2.3 `[TIDAK DIKETAHUI]` (PT-69).

### 4.1 Aturan bisnis, parameter, dan event audit baru (usulan)

| ID | Aturan | Sumber |
| --- | --- | --- |
| AB-80 | Harga paket korporat yang dibayar perusahaan = harga rekanan + markup Ruang Sehat, dihitung server per satuan dengan pembulatan AB-11a. Pajak mengikuti AB-10a (CR-013) | BR-138, AB-10, AB-10a, AB-11a |
| AB-81 | Harga, markup, tarif pajak, dan parameter yang berlaku untuk pesanan korporat dikunci saat HR mengonfirmasi pesanan (saat invoice terbit), karena pesanan korporat tidak membuat pembayaran di payment gateway | `[UP-81]` (analog AB-17), BR-142 |
| AB-82 | Pesanan korporat tidak memakai payment gateway. Pelunasan dicatat dari transfer bank oleh Keuangan. Rekening penerima menunggu PT-59 | BR-141 |
| AB-83 | Layanan korporat hanya dapat dijadwalkan setelah invoice lunas | BR-142 `[USER] P3-3` |
| AB-84 | HR tidak pernah dapat mengakses hasil lengkap (nilai, PDF, catatan pemeriksa). Ringkasan hanya dapat dibaca HR selama persetujuan peserta untuk pesanan itu aktif | BR-143, BR-144 `[USER] P2-4` |
| AB-85 | Peserta ditautkan ke akun pasien hanya oleh peserta sendiri, lewat tautan undangan yang dibuka setelah masuk di aplikasi pasien. Tidak ada penautan otomatis berdasarkan kecocokan email | BR-140 `[KEPUTUSAN-255]` |
| AB-86 | Setiap HR hanya dapat mengakses data perusahaannya. ID perusahaan diambil dari sesi, bukan dari parameter permintaan | BR-145 `[KEPUTUSAN-254]` |
| AB-87 | HR hanya menerima data yang berasal dari hasil (nilai, ringkasan, ketersediaan hasil, status tindak lanjut, agregat, atau baris tagihan yang bergantung pada hasil) untuk peserta yang persetujuannya aktif saat permintaan. Rilis ini tidak menyediakan agregat hasil bagi HR | BR-144 `[KEPUTUSAN-266]` (review keamanan B-03) |

| ID | Parameter | Nilai awal | Diatur oleh | Sumber |
| --- | --- | --- | --- | --- |
| PB-23 | Batas berkas unggahan peserta: ukuran dan jumlah baris per berkas | `[TIDAK DIKETAHUI]`; nilai teknis awal diusulkan di Tahap 8 dan dibawa ke user | Super Admin | `[UP-82]` |
| PB-24 | Masa berlaku tautan undangan peserta | `[TIDAK DIKETAHUI]` | Super Admin | `[UP-83]` |
| PB-25 | Jatuh tempo invoice B2B bawaan | 14 hari (jenis hari: PT-63) | Keuangan | `[USER] P3-3` |

| Kode | Event (usulan) | Contoh | Sumber |
| --- | --- | --- | --- |
| E-18 | Impor data peserta korporat dan pengiriman undangan (jumlah baris, hasil, tanpa isi baris) | HR mengonfirmasi impor; undangan dikirim ulang | BR-107, BR-145 `[UP-86]` |

Event yang sudah ada dipakai untuk aksi korporat: E-01–E-04 (akun HR), E-05 (HR membaca ringkasan peserta, admin membuka dokumen perusahaan), E-08 (invoice terbit, penerimaan transfer, entri pembalik), E-09 (status pesanan korporat), E-10/E-11 (akun HR dan perusahaan), E-12 (ekspor rekap, unduh invoice), E-13 (persetujuan berbagi ringkasan), E-16 (soft delete konten dan perusahaan), E-17 (data peserta ke rekanan).

**Usulan baris matriks akses PRD §3.1** (T1):

| Data | Pasien (peserta) | HR perusahaan | Admin klinik/lab (portal) | Admin RS | Sensitif |
| --- | --- | --- | --- | --- | --- |
| Data perusahaan dan dokumen NIB/NPWP | — | Perusahaannya (baca) | — | Sub-peran berwenang (PT-67) | Tidak |
| Data peserta (nama, kontak, tanggal lahir, NIK bila ada), termasuk keikutsertaannya dalam pesanan korporat | Dirinya | Peserta perusahaannya; NIK termasking | Peserta pesanan yang dilaksanakannya, data minimal (L-17) | Layanan Pelanggan (baca, tiket terkait); NIK lengkap tidak ada peran | NIK: Ya. Data lain: perlu konfirmasi legal (L-08, L-18); sampai dijawab diperlakukan Ya |
| Rekap kehadiran | Dirinya | Perusahaannya | Pesanannya | Layanan Pelanggan (baca) | Perlu konfirmasi legal (L-08, L-18); sampai dijawab diperlakukan Ya |
| Ringkasan hasil (fit/unfit) | Dirinya | Hanya dengan persetujuan aktif | — | Tidak ada peran | Ya |
| Hasil lengkap | Dirinya | Tidak | Pemeriksa di rekanan (seperti §3.1 baris hasil lab) | Tidak ada peran | Ya |
| Invoice dan penerimaan | — | Perusahaannya | — | Keuangan; Super Admin (baca) | Tidak (data keuangan badan usaha); berkas bukti transfer dapat memuat nomor rekening lengkap pembayar, hanya Keuangan |

Dasar baris "perlu konfirmasi legal": UU PDP mendefinisikan data kesehatan sebagai "catatan atau keterangan individu yang berkaitan dengan kesehatan fisik, kesehatan mental, dan/ atau pelayanan kesehatan" (Penjelasan Pasal 4 ayat (2) huruf a `[SRC-440]`), dan keikutsertaan atau kehadiran seseorang dalam MCU, vaksinasi, atau pemeriksaan lab dapat termasuk definisi itu. L-08 yang sudah lulus gate juga masih menanyakan hal serupa untuk data transaksi `[DOK: 02-brd.md]`. Selama belum dijawab, akses detail kedua data itu oleh pihak selain subjek mengikuti aturan data sensitif (E-05, KU-03). Keputusan user bahwa HR melihat rekap kehadiran `[USER] P2-4` tetap berlaku; yang ditanyakan ke legal adalah dasar pemrosesannya.

### 4.2 EP-18 Situs web Ruang Sehat

| Story | Fitur | BR | Prioritas | Kelas |
| --- | --- | --- | --- | --- |
| US-1801 | Halaman company profile publik | BR-130, BR-131 | Must | L |
| US-1802 | Konten situs di CMS | BR-131, BR-061 | Must | M |
| US-1101 | Pengelolaan konten (CMS), dipindah dari Fase 3 ke Fase 1; AC tidak berubah | BR-061 | Must | M |

#### US-1801 Halaman company profile publik

Sebagai pengunjung, saya ingin membaca profil, layanan, dan informasi Ruang Sehat di web, agar mengenal Ruang Sehat dan layanan korporatnya.
Rujukan: BR-130, BR-131 · Audit: tidak ada aksi sensitif

- AC1. Given konten berstatus "Tayang" untuk kanal situs, When pengunjung tanpa login membuka salah satu dari sembilan halaman BR-130, Then halaman tampil dari konten CMS yang sedang tayang.
- AC2. Given halaman Layanan, When tampil, Then isi dan urutan layanan (tujuh layanan dan layanan korporat) berasal dari konten CMS, tidak ditulis tetap di kode. Status aktif atau tidaknya layanan di aplikasi saat go-live ditentukan isi konten itu.
- AC3. Given konten berstatus Draf, terjadwal, berakhir, diarsipkan, atau terhapus, When URL-nya dibuka, Then halaman "tidak ditemukan" tampil tanpa membocorkan isi atau keberadaan draf.
- AC4. Given halaman Kebijakan privasi dan syarat, When tampil, Then versi dan tanggal berlaku tercantum. Teks final menunggu L-08, L-20, dan PT-24 dan sampai itu ditandai draf.
- AC5. Given pengunjung membuka halaman situs mana pun, When respons diperiksa, Then tidak ada cookie sesi dibuat dan tidak ada data pribadi dimuat `[KEPUTUSAN-251]`.
- AC6. Given halaman situs, When pemeriksaan aksesibilitas otomatis dijalankan, Then lulus aturan design guideline §7. Gambar tanpa teks alternatif tidak dapat ditayangkan (US-1802 AC5).
- AC7. Given layanan konten backend tidak tersedia, When pengunjung membuka halaman, Then halaman galat umum tampil tanpa jejak teknis (KU-08 analog).

#### US-1802 Konten situs di CMS

Sebagai admin Konten, saya ingin mengelola konten halaman situs dari CMS yang sama dengan aplikasi, agar situs selalu mutakhir tanpa perubahan kode.
Rujukan: BR-131, BR-061, US-1101 · Audit: E-16 (hapus dan pulihkan)

- AC1. Given admin Konten membuat konten, When memilih kanal (aplikasi, situs, atau keduanya) dan untuk situs memilih halaman tujuan, Then konten hanya tampil di kanal terpilih setelah berstatus Tayang (aturan jadwal US-1101 AC2).
- AC2. Given halaman situs bertipe halaman tetap (misalnya Tentang Kami), When versi baru ditayangkan, Then hanya satu versi yang tayang untuk halaman itu, dan versi sebelumnya berakhir.
- AC3. Given promo di situs, When tampil, Then hanya berupa informasi dan tidak mengubah harga di aplikasi maupun katalog korporat (US-1101 AC4).
- AC4. Given akun admin tanpa sub-peran Konten, When mencoba membuat, mengubah, atau menayangkan konten situs, Then ditolak (403) dan percobaan tercatat.
- AC5. Given konten memuat gambar tanpa teks alternatif, When admin menayangkannya, Then penayangan ditolak dengan pesan per field (KU-01).
- AC6. (negatif, XSS) Given isi konten memuat HTML mentah (misalnya `<script>`, `<iframe>`, atribut `on*`), tautan dengan skema selain `https`, `mailto`, atau relatif (misalnya `javascript:`, `data:`), atau gambar dari sumber selain endpoint gambar publik backend, When admin menyimpan, Then backend menolak atau menetralkan bagian itu sesuai aturan render aman, sehingga tidak ada markup aktif yang tersimpan sebagai konten siap tayang `[KEPUTUSAN-264]`.
- AC7. Given konten tersimpan, When dirender di situs dan di pratinjau backoffice, Then keduanya memakai aturan render aman yang sama (Markdown tanpa HTML mentah, daftar izin skema URL), dan test payload XSS gagal dieksekusi di keduanya.
- AC8. Given admin membuka pratinjau, When pratinjau tampil, Then isinya dirender di iframe `sandbox` tanpa `allow-same-origin` (origin opak), sehingga skrip apa pun di pratinjau tidak dapat membaca atau memakai sesi backoffice.
- AC9. Given halaman `web/situs` atau backoffice dimuat, When header respons diperiksa, Then Content Security Policy melarang skrip inline dan sumber pihak ketiga; situs tidak memuat skrip, font, atau sematan pihak ketiga (M-48).

### 4.3 EP-19 Layanan korporat (member area)

| Story | Fitur | BR | Prioritas | Kelas | Kelompok |
| --- | --- | --- | --- | --- | --- |
| US-1901 | Pendaftaran perusahaan mandiri | BR-136 | Must | M | K1 |
| US-1902 | Verifikasi perusahaan oleh admin Ruang Sehat | BR-136 | Must | M | K1 |
| US-1903 | Perusahaan dibuatkan admin/sales | BR-136 | Must | S | K1 |
| US-1904 | Akun HR: masuk, MFA, kelola anggota HR | BR-132, BR-145 | Must | L | K1 |
| US-1905 | Katalog paket korporat | BR-133, BR-137, BR-138 | Must | M | K2 |
| US-1906 | Pesan layanan korporat | BR-135, BR-137, BR-142 | Must | L | K2 |
| US-1907 | Unggah daftar peserta | BR-139, BR-145 | Must | L | K2 |
| US-1908 | Undangan peserta dan penautan akun pasien | BR-140, BR-145 | Must | L | K2 |
| US-1909 | Invoice B2B | BR-141, BR-142 | Must | L | K2 |
| US-1910 | Pencatatan transfer dan pelunasan | BR-141 | Must | L | K2 |
| US-1911 | Penjadwalan setelah lunas | BR-142, BR-135 | Must | M | K2 |
| US-1912 | Rekap kehadiran untuk HR | BR-144 | Must | M | K2 |
| US-1913 | Persetujuan berbagi ringkasan oleh peserta | BR-144, BR-145 | Must | M | K2 |
| US-1914 | Ringkasan hasil untuk HR dengan persetujuan | BR-144 | Must | M | K3 |
| US-1915 | Hasil lengkap untuk peserta | BR-143 | Must | M | K3 |
| US-1916 | Pengelolaan perusahaan di backoffice | BR-136, BR-060 | Must | M | K1 |
| US-1917 | Penerusan pesanan korporat ke rekanan pelaksana | BR-134, BR-070 | Must | M | K3 |

Kelompok: **K1** dapat dibangun di Fase 1 di atas fondasi Fase 1; **K2** butuh jurnal (Fase 2) dan jawaban PT-59–PT-63; **K3** butuh Partner Integration API (EN-21) dan alur hasil lab (US-901), keduanya di Fase 3 `[DOK: 07-phase-plan.md]` §8.2.

#### US-1901 Pendaftaran perusahaan mandiri (T1)

Sebagai HR perusahaan, saya ingin mendaftarkan perusahaan saya di web, agar dapat memesan layanan korporat setelah diverifikasi.
Rujukan: BR-136, BR-103 · Audit: E-04, E-10, E-13

- AC1. Given calon HR mengisi data perusahaan dan data dirinya `[UP-79]`, mengunggah dokumen NIB dan NPWP, dan menyetujui teks pemrosesan data, When mengirim, Then perusahaan berstatus "Menunggu verifikasi", akun HR dibuat di realm `korporat` dengan verifikasi email wajib, dan bukti persetujuan tersimpan per versi teks.
- AC2. Given dokumen NIB atau NPWP tidak diunggah, atau jenis/ukurannya di luar NFR-SEC-08, When mengirim, Then pengiriman ditolak per field (KU-01).
- AC3. Given format NIB atau NPWP salah, When mengirim, Then ditolak per field. Format resmi keduanya `[TIDAK DIKETAHUI]` dan diverifikasi ke sumber resmi sebelum task dimulai.
- AC4. (negatif) Given email sudah terdaftar di realm `korporat`, When mendaftar, Then layar yang tampil sama dengan pendaftaran berhasil dan petunjuk dikirim ke email itu (pola US-101 AC3, ADR-011 butir 4).
- AC5. (negatif) Given salah satu langkah pendaftaran gagal (akun Keycloak, data perusahaan, atau bukti persetujuan), When kegagalan terjadi, Then tidak ada akun HR tanpa perusahaan dan persetujuan (kompensasi pola ADR-011 butir 6).
- AC6. Given perusahaan berstatus Menunggu verifikasi atau Ditolak, When HR masuk, Then HR hanya melihat status verifikasi, alasan penolakan bila ada, dan profil perusahaannya; katalog, pesanan, dan peserta tidak tersedia.
- AC7. Given NIB yang sama sudah dipakai perusahaan aktif lain, When mendaftar, Then perilakunya `[TIDAK DIKETAHUI]` (PT-72).

#### US-1902 Verifikasi perusahaan oleh admin Ruang Sehat (T1)

Sebagai admin Ruang Sehat yang berwenang, saya ingin memeriksa data dan dokumen perusahaan, agar hanya badan usaha yang sah dapat memesan.
Rujukan: BR-136 · Audit: E-05 (dokumen dibuka), E-10 (keputusan)

- AC1. Given antrean verifikasi perusahaan, When admin berwenang (PT-67) membukanya, Then pengajuan tampil urut dari yang terlama, dengan data perusahaan dan dokumen yang hanya dibuka lewat URL berumur pendek setelah otorisasi (ADR-018).
- AC2. Given admin menyetujui, When disimpan, Then perusahaan berstatus Aktif dan akun HR pertama dapat memakai menu pesanan.
- AC3. Given admin menolak dengan kode alasan, When disimpan, Then perusahaan berstatus Ditolak, HR melihat alasannya saat masuk, dan dapat memperbaiki data serta mengajukan ulang. Pemberitahuan email atas keputusan menyusul bersama US-1001 (Fase 2), seperti US-201 AC2.
- AC4. (negatif) Given dua admin memutuskan pengajuan yang sama hampir bersamaan, When keputusan kedua disimpan, Then ditolak karena keputusan pertama yang berlaku (pola US-201 AC5), dan tercatat.
- AC5. (negatif) Given admin tanpa sub-peran berwenang atau akun HR, When memanggil endpoint verifikasi, Then ditolak (403 untuk admin, 404 untuk HR) dan percobaan tercatat.

#### US-1903 Perusahaan dibuatkan admin/sales (T1)

Sebagai admin/sales Ruang Sehat, saya ingin membuat akun perusahaan setelah kesepakatan, agar HR langsung dapat masuk tanpa mendaftar sendiri.
Rujukan: BR-136 · Audit: E-10, E-04

- AC1. Given admin berwenang (PT-67) mengisi data perusahaan, dokumen, dan email HR pertama, When menyimpan, Then perusahaan berstatus Aktif dan HR menerima undangan aktivasi (password dan TOTP) di realm `korporat`.
- AC2. (negatif) Given tautan undangan kedaluwarsa atau sudah dipakai, When dibuka, Then ditolak dan HR diarahkan meminta undangan baru lewat admin.
- AC3. Given perusahaan dibuat admin, When dicatat, Then event E-10 memuat pelaku admin dan sumber "dibuat admin".

#### US-1904 Akun HR: masuk, MFA, kelola anggota HR (T1)

Sebagai HR perusahaan, saya ingin masuk dengan aman dan mengelola rekan HR di perusahaan saya, agar member area dipakai orang yang berhak.
Rujukan: BR-132, BR-145, NFR-SEC-02, NFR-SEC-04 · Audit: E-01–E-04, E-10, E-11

- AC1. Given HR dengan akun aktif, When masuk, Then halaman login realm `korporat` meminta password dan TOTP; pendaftaran TOTP dan kode pemulihan wajib saat login pertama `[KEPUTUSAN-253]`.
- AC2. Given HR aktif, When mengundang HR lain dengan email, Then undangan aktivasi terkirim dan akun baru terikat ke perusahaan pengundang. HR dapat menonaktifkan HR lain di perusahaannya; HR aktif terakhir tidak dapat dinonaktifkan (pola US-208 AC2). Peran di dalam perusahaan `[TIDAK DIKETAHUI]` (PT-68); sampai dijawab, semua HR berperan sama.
- AC3. (negatif, isolasi) Given HR perusahaan A, When mengakses perusahaan, anggota HR, pesanan, peserta, batch impor, invoice, atau rekap milik perusahaan B, termasuk dengan mengganti ID di permintaan, Then dijawab seperti objek tidak ada (404), dan percobaan ke data peserta tercatat dengan hasil gagal (KU-03).
- AC4. (negatif) Given token realm `korporat`, When dipakai ke API pasien, backoffice, atau portal rekanan, Then ditolak; token realm lain ditolak di API korporat (ADR-011 butir 8).
- AC5. Given akun HR atau perusahaannya dinonaktifkan, When HR memakai sesi yang masih terbuka, Then permintaan berikutnya ditolak (KU-10).
- AC6. Given HR tidak aktif 30 menit, When kembali, Then sesi web berakhir (NFR-SEC-04).
- AC7. Given HR kehilangan perangkat TOTP tetapi masih memegang kode pemulihan, When masuk dengan kode pemulihan, Then HR diarahkan mendaftarkan TOTP baru setelah autentikasi ulang (jalur mandiri, pola US-103 AC7–AC8).
- AC8. Given HR kehilangan TOTP dan kode pemulihan, When meminta reset, Then reset hanya dapat dimulai sub-peran admin berwenang (PT-67) setelah konfirmasi lewat kanal yang berbeda dari kanal permintaan (HR aktif lain di perusahaan yang sama, atau kontak resmi perusahaan yang terdaftar saat verifikasi); reset baru berlaku setelah HR membuka tautan sekali pakai di email akun dan memasukkan password (pola KEPUTUSAN-204); TOTP lama tetap berlaku sampai TOTP baru dan kode pemulihan baru tersimpan (H-16 a); semua sesi HR itu dicabut, semua HR aktif perusahaan diberi tahu, dan E-04 tercatat. Super Admin tidak punya jalur reset kecuali user memutuskannya. Tidak ada admin yang melihat atau menetapkan kredensial HR `[KEPUTUSAN-265]`. AC ini diverifikasi di Fase 2 bersama US-207 AC5 (KEPUTUSAN-179).
- AC9. (negatif) Given admin tanpa sub-peran berwenang, atau permintaan reset tanpa konfirmasi kanal kedua, When reset dimulai, Then ditolak dan tercatat; reset atas akun HR milik staf sendiri ditolak (H-16 e).

#### US-1905 Katalog paket korporat (T1, harga)

Sebagai HR perusahaan aktif, saya ingin melihat paket vaksinasi, MCU, dan lab massal beserta harganya, agar dapat memilih paket.
Rujukan: BR-133, BR-137, BR-138, AB-80 · Audit: tidak ada aksi sensitif

- AC1. Given perusahaan Aktif, When HR membuka katalog, Then paket tampil per jenis (vaksinasi massal, MCU massal, lab massal) dan rekanan pelaksana, dengan isi paket, lokasi yang tersedia (on-site, di rekanan, atau keduanya), dan harga sebagai satu angka (AB-13). Satuan harga (per peserta atau per paket) dan jumlah peserta minimum `[TIDAK DIKETAHUI]` (PT-62).
- AC2. Given harga rekanan dan aturan markup yang berlaku, When harga tampil, Then harga dihitung server sesuai AB-80, dan markup tidak tampil sebagai baris tersendiri.
- AC3. Given paket dihapus rekanan atau rekanan nonaktif, When katalog dibuka, Then paket tidak tampil (KU-04). Bila data rekanan tidak dapat dikonfirmasi, berlaku KU-08.
- AC4. (negatif) Given perusahaan Menunggu verifikasi, Ditolak, atau Nonaktif, When HR memanggil endpoint katalog, Then ditolak.
- AC5. Sumber harga rekanan mengikuti jawaban PT-61.

#### US-1906 Pesan layanan korporat (T1, harga)

Sebagai HR, saya ingin memesan paket dengan lokasi dan jumlah peserta, agar layanan dapat dijadwalkan.
Rujukan: BR-135, BR-137, BR-142, AB-81 · Audit: E-09

- AC1. Given HR memilih paket, lokasi (on-site dengan alamat kantor, atau di rekanan), dan jumlah peserta, When membuka ringkasan, Then rincian tampil: harga per satuan, jumlah, pajak sebagai baris terpisah (AB-10a, sisa PT-28), dan total.
- AC2. Given HR mengonfirmasi, When disimpan, Then harga, markup, dan tarif pajak dikunci (AB-81), pesanan berstatus "Menunggu pembayaran" `[UP-80]`, dan invoice terbit (US-1909).
- AC3. (negatif) Given HR menekan konfirmasi dua kali atau permintaan terkirim ulang, When diterima server, Then hanya satu pesanan dan satu invoice yang dibuat (KU-05).
- AC4. (negatif) Given harga atau ketersediaan berubah antara ringkasan dan konfirmasi, When HR mengonfirmasi, Then nilai terbaru ditampilkan dan harus disetujui ulang (pola AB-15).
- AC5. Given alamat on-site di luar area layanan rekanan, When HR memilih lokasi, Then perilakunya `[TIDAK DIKETAHUI]` (PT-74).
- AC6. Penetapan tanggal pelaksanaan, perubahan jumlah peserta setelah invoice terbit, dan pembatalan oleh perusahaan `[TIDAK DIKETAHUI]` (PT-63).

#### US-1907 Unggah daftar peserta (T1, data karyawan)

Sebagai HR, saya ingin mengunggah daftar peserta dari Excel/CSV, agar tidak mengetik peserta satu per satu.
Rujukan: BR-139, BR-145, NFR-SEC-08 (direvisi) · Audit: E-18 `[UP-86]`

- AC1. Given HR membuka pesanan miliknya, When mengunggah berkas CSV atau XLSX dengan kolom nama, email dan/atau nomor HP, tanggal lahir, serta NIK hanya bila paket menandai NIK wajib, Then sistem menampilkan pratinjau validasi per baris (valid atau galat dengan alasan) sebelum ada data yang tersimpan `[UP-82]`.
- AC2. Given pratinjau, When HR mengonfirmasi, Then hanya baris valid yang diimpor; jumlah baris valid, galat, dan ganda tercatat (E-18) tanpa isi baris.
- AC3. (negatif, berkas rusak) Given berkas bukan CSV/XLSX asli (isi tidak sesuai ekstensi), rusak, terkunci password, melebihi PB-23, atau header kolom tidak dikenali, When diunggah, Then seluruh berkas ditolak dengan pesan yang menjelaskan sebabnya, dan tidak ada baris tersimpan.
- AC4. (negatif, baris rusak) Given baris dengan field wajib kosong, email atau nomor HP tidak valid, tanggal lahir tidak valid atau di masa depan, atau NIK bukan 16 digit angka padahal wajib (format NIK ADR-013 §4), When divalidasi, Then baris ditandai galat dengan alasan per field dan tidak diimpor.
- AC5. (negatif, ganda) Given dua baris dalam satu berkas, atau satu baris dan peserta yang sudah terdaftar di pesanan yang sama, punya email yang sama, nomor HP yang sama, atau NIK yang sama (dibandingkan lewat blind index), When divalidasi, Then baris kedua ditandai ganda dan tidak diimpor dua kali.
- AC6. (negatif) Given sel diawali karakter yang dapat ditafsirkan sebagai rumus oleh aplikasi spreadsheet, When disimpan dan kemudian diekspor (US-1912 AC3), Then isinya diperlakukan sebagai teks dan dinetralkan saat ekspor `[KEPUTUSAN-256]`.
- AC7. Given impor selesai atau pratinjau dibatalkan atau kedaluwarsa, When dicek, Then berkas asli dan baris pratinjau terhapus; log aplikasi tidak memuat isi baris `[KEPUTUSAN-256]`.
- AC8. (negatif, isolasi) Given HR perusahaan lain, When membuka batch impor atau pratinjau dengan ID-nya, Then 404.
- AC9. Given jumlah peserta valid berbeda dari jumlah di pesanan, When HR mengonfirmasi, Then perilakunya `[TIDAK DIKETAHUI]` (PT-63).
- AC10. Given berkas memuat kolom di luar skema (misalnya jabatan atau golongan darah), When diproses, Then kolom itu dibuang saat pratinjau, tidak pernah disimpan, dan HR diberi tahu bahwa kolom diabaikan (minimisasi, Pasal 16 ayat (2) huruf a `[SRC-440]`).
- AC11. (negatif) Given XLSX berisi makro (`.xlsm`, `vbaProject.bin`), wadah OLE, DTD atau entitas eksternal, atau ukuran setelah dekompresi, jumlah entri, atau rasio kompresinya melebihi batas, When diproses, Then berkas ditolak sebelum diurai penuh; penguraian berjalan di worker dengan batas waktu dan memori, bukan di proses `api` `[SRC-405]` `[SRC-406]` `[SRC-409]` `[KEPUTUSAN-269]`.
- AC12. Given baris ditolak, When pesan galat tampil, Then pesan menyebut nomor baris, field, dan kode alasan, tanpa menggemakan isi sel.

#### US-1908 Undangan peserta dan penautan akun pasien (T1, data karyawan)

Sebagai peserta, saya ingin menerima undangan dan menautkannya ke akun pasien saya, agar jadwal dan hasil saya tersedia di akun saya sendiri.
Rujukan: BR-140, BR-145, AB-85, AB-73 · Audit: E-18, E-10, E-13

- AC1. Given peserta diimpor, When undangan dikirim (waktu pengiriman PT-63; usulan keamanan: setelah lunas, M-50), Then peserta menerima email berisi tautan sekali pakai yang berlaku PB-24, tanpa data kesehatan (AB-73), dan berisi pemberitahuan pemrosesan data: siapa yang mengirim datanya, tujuan, dan cara mengajukan hak subjek data. Teks pemberitahuan menunggu L-17 dan PT-76.
- AC1a. Given tautan undangan, When dibuka di perangkat, Then tautan memakai App Links/Universal Links terverifikasi untuk domain resmi, bukan skema URL kustom, dan token hanya ditukar setelah peserta masuk di realm `pasien` `[KEPUTUSAN-267]`.
- AC1b. Given HR mengirim ulang undangan, When dikirim, Then jumlah undangan aktif tidak melebihi jumlah peserta pesanan, kirim ulang dibatasi laju per peserta dan per perusahaan per hari, dan templat tidak memuat teks bebas dari HR; nama peserta dikodekan sebagai teks (M-50, M-54).
- AC2. Given peserta belum punya akun pasien, When membuka tautan, Then diarahkan mendaftar (US-101, termasuk persetujuan US-104 dan MFA CR-018) lalu kembali ke undangan; peserta yang sudah punya akun cukup masuk.
- AC3. Given peserta yang sudah masuk di realm `pasien` membuka tautannya, When menerima undangan, Then peserta tertaut ke akun itu (AB-85). Penautan tidak pernah terjadi tanpa tindakan peserta.
- AC4. (negatif) Given akun pasien yang membuka tautan memakai email terverifikasi yang berbeda dari email undangan, When menerima, Then penautan ditolak dengan pesan netral, dan peserta diminta meminta HR mengirim ulang ke email akunnya. Aturan ini syarat keamanan (UP-83, M-43); bila user menolaknya, dicatat sebagai risiko sisa yang diterima user `[KEPUTUSAN-267]`.
- AC4a. Bila user menyetujui PT-78: Given peserta menerima undangan, When diminta mengetik tanggal lahirnya, Then penautan hanya terjadi bila cocok dengan data unggahan; percobaan dibatasi, dan setelah batas habis undangan terkunci dan HR melihat status netral.
- AC5. (negatif) Given tautan kedaluwarsa, sudah dipakai, atau milik undangan yang dibatalkan, When dibuka, Then ditolak tanpa membocorkan nama perusahaan atau pesanan.
- AC6. Given peserta menolak undangan, When disimpan, Then HR melihat status "ditolak" tanpa alasan dan tanpa data akun pasien.
- AC7. Given email gagal terkirim, When kegagalan terjadi, Then pengiriman dicoba ulang (US-1001 AC3) dan HR melihat status "gagal kirim" serta dapat mengirim ulang.
- AC8. (negatif) Given HR, When mencoba melihat data akun pasien peserta (profil, identitas, riwayat layanan lain), Then tidak tersedia; HR hanya melihat data yang diunggahnya dan status undangan.

#### US-1909 Invoice B2B (T1, penagihan)

Sebagai HR, saya ingin menerima invoice saat pesanan dikonfirmasi, agar perusahaan dapat membayar lewat transfer.
Rujukan: BR-141, BR-142, AB-81, AB-82, PB-25 · Audit: E-08, E-12

- AC1. Given HR mengonfirmasi pesanan, When invoice terbit, Then invoice memuat nomor unik, data perusahaan, paket, harga per satuan, jumlah, pajak (sisa PT-28), total, tanggal terbit, jatuh tempo PB-25, dan rekening tujuan (PT-59).
- AC2. Given invoice sudah terbit, When perlu koreksi, Then invoice lama dibatalkan dan invoice pengganti terbit; baris invoice tidak pernah diubah atau dihapus (ADR-004).
- AC3. Given HR mengunduh invoice, When berkas dibuat, Then berkas berisi data perusahaannya saja dan E-12 tercatat.
- AC4. (negatif, isolasi) Given HR perusahaan lain, When membuka invoice dengan ID atau nomornya, Then 404.
- AC5. Given jatuh tempo lewat tanpa pelunasan, When dicek, Then invoice berstatus "Lewat jatuh tempo" `[UP-80]`; akibatnya bagi pesanan (batal otomatis, tetap menunggu, pengingat) `[TIDAK DIKETAHUI]` (PT-63).
- AC6. (negatif, kanal turunan) Given ada layanan tambahan yang muncul karena hasil seorang peserta (misalnya pemeriksaan lanjutan; ada atau tidaknya `[TIDAK DIKETAHUI]`, PT-63), When invoice atau invoice pengganti terbit, Then baris invoice tidak menyebut peserta dan tidak memungkinkan HR mengaitkan baris itu dengan peserta yang persetujuannya tidak aktif (AB-87).
- AC7. Given rekening tujuan invoice, When ditampilkan, Then berasal dari konfigurasi yang perubahannya tercatat E-14 dan, bila user menyetujui UP-84, disetujui orang kedua; email pemberitahuan invoice tidak berlampiran dan mengarah ke member area (M-56; berlaku setelah PT-59).

#### US-1910 Pencatatan transfer dan pelunasan (T1, uang)

Sebagai Keuangan, saya ingin mencatat transfer yang diterima atas invoice, agar pesanan dapat dijadwalkan dan pembukuan seimbang.
Rujukan: BR-141, AB-82, ADR-015 butir 4 · Audit: E-08

- AC1. Given Keuangan mencatat satu transfer masuk sebagai mutasi bank (rekening tujuan, tanggal valuta, referensi bank, nominal, bukti), When disimpan, Then mutasi tercatat immutable sebagai baris tersendiri dengan kunci unik global atas rekening tujuan, tanggal valuta, referensi, dan nominal `[KEPUTUSAN-268]`. Komposisi kunci diverifikasi terhadap format mutasi bank yang dipakai saat task.
- AC1a. Given mutasi tercatat, When Keuangan mengalokasikannya ke satu atau beberapa invoice, Then setiap alokasi tercatat immutable dan jurnal berpasangan yang seimbang terbentuk; jumlah semua alokasi dari satu mutasi tidak pernah melebihi nominal mutasi, ditegakkan di aplikasi dan di database.
- AC2. Given total alokasi ke sebuah invoice sama dengan total invoice, When tersimpan, Then invoice berstatus Lunas dan pesanan berstatus "Lunas, menunggu jadwal" `[UP-80]`.
- AC3. (negatif) Given total alokasi kurang atau lebih dari sisa tagihan, When tersimpan, Then invoice tidak menjadi Lunas otomatis dan ditandai untuk tindak lanjut Keuangan. Perlakuan bayar sebagian dan kelebihan bayar `[TIDAK DIKETAHUI]` (PT-63).
- AC4. (negatif, transfer ganda) Given mutasi dengan kunci yang sama sudah tercatat, When dicatat lagi, untuk invoice yang sama atau invoice lain, Then ditolak. Given mutasi yang sisa nominalnya sudah habis dialokasikan, When dialokasikan lagi ke invoice lain, Then ditolak.
- AC5. Given pencatatan atau alokasi salah, When dikoreksi, Then koreksi berupa entri pembalik yang merujuk baris asal, termasuk jurnal pembalik (ADR-004, ADR-015 butir 4).
- AC6. Given pemisahan tugas antara pencatat dan penyetuju, dan status Lunas hanya dari mutasi yang diperiksa Keuangan terhadap rekening koran (bukan dari bukti transfer kiriman perusahaan), When berlaku, Then mengikuti keputusan user atas `[UP-84]`; bila ditolak, dicatat sebagai risiko sisa yang diterima user (B2B-01).
- AC7. (negatif) Given akun tanpa sub-peran Keuangan, When mencatat, mengalokasikan, atau membalik, Then ditolak (403) dan tercatat.
- AC8. Given berkas bukti transfer, When disimpan atau dibuka, Then hanya dapat dibuka Keuangan lewat URL berumur pendek, dan isinya (termasuk nomor rekening pembayar) tidak pernah masuk log maupun audit trail; audit hanya merujuk ID berkas.

#### US-1911 Penjadwalan setelah lunas

Sebagai HR, saya ingin layanan dijadwalkan setelah invoice lunas, agar peserta tahu kapan dan di mana layanan berlangsung.
Rujukan: BR-142, BR-135, AB-83 · Audit: E-09

- AC1. Given invoice Lunas, When jadwal ditetapkan (pihak yang menetapkan dan cara konfirmasi rekanan: PT-63), Then pesanan berstatus Terjadwal, dan HR serta peserta yang sudah menerima undangan melihat tanggal, jam (zona waktu lokasi, AB-40), dan lokasi.
- AC2. (negatif) Given invoice belum Lunas, When jadwal ditetapkan, Then ditolak (AB-83).
- AC3. Given peserta tertaut, When membuka aplikasi pasien (atau web sesuai PT-60), Then jadwal layanan korporatnya tampil.

#### US-1912 Rekap kehadiran untuk HR (T1, data karyawan)

Sebagai HR, saya ingin melihat siapa yang sudah dan belum dilayani, agar dapat mengatur karyawan.
Rujukan: BR-144, AB-84 · Audit: E-12 (ekspor)

- AC1. Given pesanan Terjadwal atau Selesai, When HR membuka rekap, Then setiap peserta tampil dengan status undangan dan status kehadiran (hadir, tidak hadir, belum dicatat). Sumber data kehadiran `[TIDAK DIKETAHUI]` (PT-64).
- AC2. Given rekap tampil, When diperiksa, Then tidak ada nilai hasil, PDF, atau catatan pemeriksa, dan ringkasan hanya tampil sesuai US-1914.
- AC3. Given HR mengekspor rekap, When berkas dibuat, Then berkas CSV hanya berisi peserta perusahaannya, sel yang dapat ditafsirkan sebagai rumus dinetralkan, dan E-12 tercatat.
- AC4. (negatif, isolasi) Given HR perusahaan lain, When membuka atau mengekspor rekap pesanan itu, Then 404.
- AC5. (negatif, kanal turunan) Given peserta tanpa persetujuan aktif, When HR membuka rekap, mengekspornya, atau memanggil endpoint daftar dan detail peserta, Then tidak ada data yang berasal dari hasil: tidak ada status "hasil tersedia", "perlu tindak lanjut", "jadwal ulang karena hasil", atau penanda serupa; status kehadiran hanya berasal dari pencatatan kehadiran (AB-87).
- AC6. Given rekap ditampilkan atau diekspor, When respons dikirim, Then BFF dan browser tidak menyimpan respons itu di cache (`Cache-Control: no-store`), dan berkas ekspor dibangkitkan saat diminta tanpa disimpan di server.

#### US-1913 Persetujuan berbagi ringkasan oleh peserta (T1, persetujuan, EP-16)

Sebagai peserta, saya ingin memutuskan sendiri apakah ringkasan hasil saya dibagikan ke perusahaan, dan dapat menariknya kapan saja.
Rujukan: BR-144, BR-145, BR-103, BR-106, NFR-PRIV-05 · Audit: E-13

- AC1. Given peserta tertaut ke pesanan korporat, When membuka pesanan di akun pasiennya, Then pertanyaan persetujuan tampil dengan jenis ringkasan (PT-65), nama perusahaan penerima, dan tujuannya; kotak tidak tercentang otomatis, dan bawaannya tidak berbagi.
- AC2. Given peserta memberi atau menarik persetujuan, When tersimpan, Then bukti tersimpan per versi teks. Penarikan berlaku ke depan: sejak penarikan tersimpan, HR tidak lagi dapat membaca ringkasan itu (US-1914 AC2).
- AC3. (negatif) Given HR, admin Ruang Sehat, atau akun pasien lain, When memanggil endpoint pemberian persetujuan untuk peserta itu, Then ditolak; hanya akun pasien yang tertaut ke peserta itu yang dapat memberi atau menarik.
- AC4. Teks persetujuan final menunggu L-18; sampai itu ditandai draf seperti US-104 AC1. Teks wajib menyatakan bahwa penarikan berlaku ke depan dan tidak menarik berkas ekspor yang sudah diunduh HR sebelumnya.

#### US-1914 Ringkasan hasil untuk HR dengan persetujuan (T1, data kesehatan)

Sebagai HR, saya ingin melihat ringkasan hasil peserta yang setuju membagikannya, agar dapat menindaklanjuti sesuai kebijakan perusahaan.
Rujukan: BR-144, AB-84 · Audit: E-05

- AC1. Given peserta memberi persetujuan aktif dan ringkasan tersedia, When HR membuka rekap atau detail peserta, Then ringkasan tampil dan E-05 tercatat dengan subjek data peserta itu.
- AC2. (negatif, tanpa persetujuan) Given peserta belum memberi atau sudah menarik persetujuan, When HR membuka rekap, detail peserta, ekspor, atau memanggil endpoint ringkasan langsung dengan ID peserta atau ID ringkasan, Then ringkasan tidak dikembalikan, tampilan mengikuti `[UP-85]`, dan percobaan lewat endpoint langsung tercatat dengan hasil gagal (KU-03).
- AC3. (negatif) Given persetujuan aktif, When HR meminta hasil lengkap (nilai, PDF, catatan pemeriksa), Then tetap ditolak (AB-84).
- AC4. (negatif) Given admin Ruang Sehat sub-peran apa pun, When mencoba membaca ringkasan, Then ditolak (sejalan PRD §3.1 baris hasil lab).
- AC5. Given HR mengekspor rekap, When berkas dibuat, Then ringkasan hanya dimuat untuk peserta yang persetujuannya aktif saat ekspor.
- AC6. Sumber, format, dan pihak yang menetapkan ringkasan `[TIDAK DIKETAHUI]` (PT-65); bergantung pada kontrak integrasi (US-1917).
- AC7. (negatif, agregat) Given endpoint atau layar apa pun untuk HR, When diperiksa, Then tidak ada jumlah, persentase, atau statistik yang dihitung dari hasil peserta tanpa persetujuan aktif. Bila agregat hasil ditambahkan lewat CR, agregat hanya dihitung atas peserta yang ringkasannya terlihat HR dan tidak ditampilkan untuk kelompok di bawah ambang minimum (nilai ditetapkan bersama CR itu) (AB-87).
- AC8. (negatif, persetujuan ditarik) Given HR sudah membuka ringkasan, lalu peserta menarik persetujuan, When HR memuat ulang halaman, membuka tab lain, atau memakai URL lama, Then ringkasan tidak lagi tampil; tidak ada salinan ringkasan di cache BFF, cache browser, atau berkas di server.

#### US-1915 Hasil lengkap untuk peserta (T1, data kesehatan)

Sebagai peserta, saya ingin melihat dan mengunduh hasil MCU atau lab saya, agar dapat memakainya.
Rujukan: BR-143, US-901, US-902 · Audit: E-05, E-12 (pola US-902)

- AC1. Given rekanan mengirim hasil untuk peserta pesanan korporat lewat alur US-901, When diterima, Then hasil tampil hanya di akun pasien peserta yang tertaut, dengan perilaku US-902.
- AC2. (negatif) Given HR, perusahaan, atau akun pasien lain, When mencoba membuka hasil, Then ditolak (KU-03).
- AC3. Given hasil datang untuk peserta yang belum menautkan akun, When diterima, Then perlakuannya `[TIDAK DIKETAHUI]` (PT-71).
- AC4. Bila user menyetujui UP-87: Given hasil peserta tersedia, When akan dirilis ke akun tertaut, Then hasil hanya dirilis bila kehadiran peserta itu tercatat dengan konfirmasi identitas di lokasi layanan (cara pencocokan mengikuti PT-64 dan PT-66); bila tidak, hasil ditahan dan Layanan Pelanggan menindaklanjuti.

#### US-1916 Pengelolaan perusahaan di backoffice

Sebagai admin Ruang Sehat yang berwenang, saya ingin melihat dan mengelola perusahaan member, agar dapat melayani klien korporat.
Rujukan: BR-136, BR-060 · Audit: E-10, E-12

- AC1. Given admin berwenang, When membuka daftar perusahaan, Then tampil status verifikasi, anggota HR, dan (setelah K2) pesanan dan invoice, tanpa ringkasan maupun hasil peserta.
- AC2. Given admin menonaktifkan perusahaan, When tersimpan, Then semua akun HR perusahaan itu ditolak seketika (KU-10). Nasib pesanan yang sedang berjalan `[TIDAK DIKETAHUI]` (PT-63).
- AC3. (negatif) Given sub-peran tanpa hak, When mengakses, Then 403.

#### US-1917 Penerusan pesanan korporat ke rekanan pelaksana (T1, integrasi)

Sebagai Ruang Sehat, saya ingin meneruskan pesanan korporat terjadwal ke rekanan pelaksana, agar rekanan menyiapkan layanan.
Rujukan: BR-134, BR-070, ADR-014 · Audit: E-17

- AC1. Given pesanan Terjadwal, When dikirim ke sistem rekanan atau RME minimalis lewat Partner Integration API, Then pesan memuat data peserta minimal (menunggu L-17) dan E-17 tercatat tanpa isi.
- AC2. Given rekanan menolak atau tidak menerima, When batas waktu habis, Then perilakunya `[TIDAK DIKETAHUI]` (PT-63).
- AC3. (negatif) Given rekanan lain, When meminta data peserta pesanan itu, Then ditolak (ADR-014 butir 3).
- AC4. Kehadiran (PT-64), ringkasan (PT-65), dan hasil (US-901) diterima kembali lewat kontrak yang sama.

### 4.4 Usulan penyusun (detail produk, menunggu user)

| UP | Usulan | Alasan |
| --- | --- | --- |
| UP-79 | Formulir pendaftaran perusahaan: nama badan usaha, NIB, NPWP, alamat kantor, nomor telepon kantor; data HR: nama, jabatan, email, password | Data minimal untuk verifikasi NIB/NPWP `[USER] P2-1`; field lain tidak disebut user |
| UP-80 | Status pesanan korporat: Menunggu pembayaran, Lunas–menunggu jadwal, Terjadwal, Selesai, Dibatalkan; status invoice: Terbit, Lunas, Lewat jatuh tempo, Dibatalkan | Turunan BR-142; transisi lengkap disusun di PRD setelah PT-63 |
| UP-81 | Harga dikunci saat konfirmasi pesanan (AB-81) | Tanpa payment gateway, titik "pembayaran dibuat" AB-17 tidak ada; invoice harus berisi harga yang tidak berubah |
| UP-82 | Impor dua langkah: pratinjau validasi, lalu HR mengonfirmasi impor baris valid saja | Mencegah data rusak tersimpan tanpa sepengetahuan HR; baris galat tetap terlihat untuk diperbaiki |
| UP-83 | **Syarat keamanan** (review keamanan B-04, M-43): undangan hanya dapat diterima akun pasien yang email terverifikasinya sama dengan email undangan. Bila berbeda, peserta meminta HR mengirim ulang ke email akunnya. Bila user menolak, dicatat sebagai risiko sisa yang diterima user | Mencegah tautan yang diteruskan atau disadap menautkan peserta ke akun orang lain (RT-40, KOR-12). Tidak menutup salah ketik email ke alamat milik orang lain yang nyata; untuk itu PT-78 dan UP-87 |
| UP-84 | Pemisahan tugas atas uang manual: (a) mutasi dan alokasi berlaku setelah dicatat satu orang Keuangan dan disetujui orang kedua (Keuangan lain atau Super Admin); (b) rekonsiliasi harian terhadap rekening koran oleh orang selain pencatat; (c) status Lunas hanya dari mutasi yang diperiksa terhadap rekening koran, bukan dari bukti transfer kiriman perusahaan; (d) perubahan rekening tujuan invoice disetujui orang kedua | Pola ADR-012 butir 7; M-45, M-56. Menambah beban operasional; bila user menolak, B2B-01 dicatat sebagai risiko sisa yang diterima user |
| UP-85 | Untuk peserta tanpa persetujuan aktif, HR melihat teks seragam "Ringkasan tidak tersedia", sama dengan saat ringkasan belum ada | Mengurangi tekanan pada peserta dan kebocoran status persetujuan dalam relasi kerja (L-18) |
| UP-86 | Event audit baru E-18 untuk impor peserta dan undangan | Masuknya data pribadi pihak ketiga dalam jumlah besar adalah aksi sensitif (BR-107) yang tidak tercakup E-01–E-17 |
| UP-87 | Hasil lengkap (K3) baru dirilis ke akun tertaut setelah kehadiran peserta tercatat dengan konfirmasi identitas di lokasi layanan | Menutup jalur salah tautan yang lolos UP-83 (salah ketik email ke alamat orang lain yang nyata; M-43 butir 3). Bergantung pada proses rekanan (PT-64, PT-66) |

---

## 5. Kebutuhan data (usulan untuk data dictionary)

Usulan bagian baru **A20. Layanan korporat** (24 tabel; v0.1 menulis "18 tabel" padahal daftarnya 23 tabel; v0.2 mengganti `b2b_payment_receipt` dengan `bank_statement_entry` dan `b2b_payment_allocation`) di database aplikasi Ruang Sehat, mengikuti konvensi §0.2–§0.7 data dictionary (snake_case, UUIDv7, kelas ADR-008, uang `_idr`, PII di Master yang dapat dianonimkan, isi spesifik terenkripsi per wadah) `[DOK: 04-data-dictionary.md]`. Kolom standar (STD-x) tidak ditulis ulang. Rancangan rinci, index, dan hak role ditetapkan di F1-T82 dengan review DBA dan keamanan `[KEPUTUSAN-262]`.

| Tabel (usulan) | Kelas | Isi utama | Sensitivitas | Kelompok |
| --- | --- | --- | --- | --- |
| `corporate_client` | M | nama badan usaha, `nib`, `npwp`, alamat, telepon, `status` (`pending_verification`, `active`, `rejected`, `inactive`), `origin` (`self_registered`, `created_by_admin`) | — (data badan usaha) | K1 |
| `corporate_client_document` | M | `corporate_client_id`, `document_kind` (`nib`, `npwp`), `stored_file_id` | Dapat memuat nama pengurus: PII | K1 |
| `corporate_verification_decision` | T | keputusan, `reason_code`, pemutus; keputusan pertama menang lewat UQ | — | K1 |
| `corporate_member` | M | `account_id` (jenis `corporate_admin`), `corporate_client_id`, `job_title`; UQ parsial satu keanggotaan aktif per akun; pencabutan atau perpindahan keanggotaan mencabut sesi (M-51) | PII (lewat akun) | K1 |
| `corporate_package` | M (salinan rekanan) | `partner_id`, `external_id`, `package_kind` (`mass_vaccination`, `mass_mcu`, `mass_lab`), nama, isi, `location_options`, `partner_price_idr`, `requires_nik`, `synced_at` | — | K2 |
| `corporate_order` | T | `corporate_client_id`, `corporate_package_id`, `partner_id`, `location_kind` (`onsite`, `partner_site`), alamat on-site, jumlah peserta | — | K2 |
| `corporate_order_status_event` / `corporate_order_state` | R / P | transisi UP-80 | — | K2 |
| `corporate_order_price_lock` | T | harga rekanan per satuan, `markup_rule_id` dan nilainya, markup per satuan, jumlah, tarif dan nominal pajak, total | — | K2 |
| `corporate_participant` | M | `corporate_client_id`, `full_name`, `email`, `phone`, `birth_date`, `nik_enc`/`nik_bidx`/`nik_last4` (opsional) | PII (Anonimkan); NIK Spesifik+Enc | K2 |
| `corporate_order_participant` | T | keikutsertaan peserta dalam pesanan; pencabutan lewat baris pembalik | — | K2 |
| `participant_import_batch` | L | pesanan, pengunggah, jumlah valid/galat/ganda, status; tanpa isi baris | — | K2 |
| `participant_import_row` | O (berumur) | baris pratinjau terenkripsi, hasil validasi; dihapus saat konfirmasi/kedaluwarsa | PII, Enc | K2 |
| `corporate_invitation` / `corporate_invitation_response` | T / T | hash token (bukan token), `expires_at`; respons diterima/ditolak dan `care_recipient_id` yang ditautkan | — | K2 |
| `corporate_attendance_record` | T | status hadir, sumber (PT-64), pencatat | — | K2 |
| `b2b_invoice`, `b2b_invoice_line`, `b2b_invoice_status_event`, `b2b_invoice_state` | T / T / R / P | nomor invoice, jatuh tempo (PB-25), total, pembatalan dan pengganti | — | K2 |
| `bank_statement_entry` | T | satu baris per transfer masuk: rekening tujuan, tanggal valuta, referensi bank, nominal, `stored_file_id` bukti, pencatat; **UQ global** atas (rekening tujuan, tanggal valuta, referensi, nominal), komposisi diverifikasi terhadap format mutasi bank saat task; pembalik lewat baris pembalik `[KEPUTUSAN-268]` | Nomor rekening pembayar hanya 4 digit terakhir di kolom (§7.3 PRD); berkas bukti dapat memuat rekening lengkap, akses hanya Keuangan, tidak pernah masuk log/audit | K2 |
| `b2b_payment_allocation` | T | `bank_statement_entry_id`, `b2b_invoice_id`, nominal alokasi, pembalik; jumlah alokasi per mutasi ≤ nominal mutasi, ditegakkan aplikasi dan database (mekanisme constraint diverifikasi saat task) `[KEPUTUSAN-268]` | — | K2 |

**Catatan untuk DBA (review keamanan putaran 2, R2-01)** atas `bank_statement_entry`, diputuskan saat task bersama review DBA: (i) nominal ikut menjadi bagian kunci unik, sehingga salah ketik nominal untuk referensi yang sama lolos sebagai mutasi kedua. Pilih kunci tanpa nominal, atau peringatan bila referensi sama muncul dengan nominal berbeda. (ii) Mutasi yang dibalik lalu dicatat ulang dengan kunci identik akan ditolak UQ. Pilih UQ yang mengecualikan mutasi yang sudah dibalik, atau koreksi lewat alokasi pembalik tanpa mencatat ulang mutasi.
| `corporate_share_consent_record` | L | peserta tertaut, perusahaan, pesanan, `consent_text_version_id`, aksi (beri/tarik) | Merujuk orang lewat ID (KEPUTUSAN-42) | K2 |
| `corporate_result_summary` | T | peserta, pesanan, `summary_enc`, `data_key_id`, versi | Spesifik, Enc | K3 |

Perubahan atas tabel yang sudah ada (migrasi *expand*, ADR-019 butir 2):
- `account.account_type` ditambah `corporate_admin`; `account.idp_realm` ditambah `korporat`; CHECK pasangan `corporate_admin`→`korporat` (A1.1).
- `cms_content` ditambah kanal (`app`, `site`) dan halaman tujuan situs (A15.4).
- `data_key.container_type` ditambah `corporate_participant_identity` (NIK peserta), `corporate_import_preview`, dan `corporate_result_summary` `[KEPUTUSAN-258]`.
- `journal_account` ditambah akun piutang korporat per perusahaan, pendapatan markup korporat, dan (bila PT-59 memilih opsi a) kewajiban kepada rekanan atas pesanan korporat (A13.1, ADR-029).
- `audit_event_subject` (B3): subjek data dapat berupa `corporate_participant` sebelum peserta tertaut ke `care_recipient`.

Retensi semua tabel baru mengikuti PT-24 (kebijakan kosong berarti tidak diproses, ADR-013 §6). Diusulkan kategori retensi baru "peserta korporat tidak tertaut" di PT-24; data peserta yang tidak pernah menerima undangan: PT-71. Invoice dan mutasi bank mengikuti retensi data keuangan, yang angkanya `[TIDAK DIKETAHUI]` (PT-24).

---

## 6. Dampak arsitektur

### 6.1 Komponen

| Komponen | Usulan | Dasar |
| --- | --- | --- |
| `web/situs` | Aplikasi Next.js publik untuk sembilan halaman, dirender server dari endpoint konten publik backend. Tanpa sesi, tanpa cookie sesi, tanpa akses Valkey, dan tanpa kunci JWE | `[KEPUTUSAN-251]`; pengerasan ADR-024 butir 2 berlaku |
| `web/korporat` | Aplikasi Next.js member area HR dengan BFF (klien rahasia realm `korporat`, cookie `HttpOnly`/`Secure`/`SameSite`, CSRF, sesi Valkey, token JWE berkunci OpenBao). T1 seperti BFF lain | ADR-011 butir 7, ADR-024 butir 3 |
| Backend Rust | Modul baru "Korporat" (perusahaan, anggota HR, pesanan, peserta, undangan, kehadiran, ringkasan) dan perluasan modul Pembayaran (invoice B2B), Konten (kanal situs), Platform (impor berkas) | ADR-006 butir 1 |
| Namespace API | `/v1/corporate/*` dengan audience sesi sendiri; `/v1/public/content/*` tanpa login | ADR-006 butir 5 (pola namespace + audience), ADR-012 butir 3 |
| Deployment | Dua kontainer Next.js tambahan, dua identitas OpenBao tambahan (`web/korporat` memegang kunci; `web/situs` tidak), rute Caddy dan nama host baru (domain `[TIDAK DIKETAHUI]`, PT-46) | ADR-025 |
| Aplikasi pasien (repositori mobile, CR-019) | Layar undangan, pesanan korporat, persetujuan berbagi ringkasan, dan jadwal; hasil memakai layar US-902. Tautan undangan memakai App Links/Universal Links terverifikasi; berkas asosiasi domain disajikan di domain resmi (`web/situs`), mekanismenya diverifikasi ke dokumentasi platform saat task `[KEPUTUSAN-267]`. Bila PT-60 memilih web, layar ini pindah ke aplikasi web terpisah dari `web/korporat` (N-03) | CR-019 `[USER]` |

Alasan memisahkan situs publik dari member area: server Next.js yang memegang sesi adalah modul T1, dan riwayat advisory Next.js memuat RCE `[SRC-246]` (via ADR-024). Situs publik menerima lalu lintas anonim terbanyak, sehingga dipisah agar kompromi situs tidak memberi akses ke sesi HR atau kunci JWE. Pola ini sejalan dengan alasan ADR-006 menolak satu web untuk backoffice dan portal `[KEPUTUSAN-251]`.

Pengerasan tambahan dari review keamanan (M-46–M-48) `[KEPUTUSAN-269]`: `web/situs` berada di jaringan Docker sendiri dan hanya menjangkau backend lewat Caddy pada `/v1/public/*`, tanpa egress lain; cookie sesi `web/korporat` berprefiks `__Host-` (wajib `Secure`, tanpa atribut `Domain`, `Path=/`, sehingga hanya dikirim ke host yang menetapkannya `[SRC-408]`); CSRF memakai synchronizer token, bukan double-submit cookie; situs tanpa skrip, font, atau sematan pihak ketiga; respons konten publik boleh di-cache Caddy dengan TTL pendek dan anggaran batas laju terpisah dari API pasien (keputusan cache di Tahap 8 dengan review keamanan). Konten CMS dirender dengan aturan render aman yang sama di situs dan pratinjau backoffice, pratinjau di iframe `sandbox` tanpa `allow-same-origin`, dan CSP tanpa skrip inline untuk situs dan backoffice (US-1802 AC6–AC9, M-40) `[KEPUTUSAN-264]`.

### 6.2 Realm Keycloak untuk HR (T1)

| Opsi | Isi | Kelebihan | Kekurangan | Penilaian |
| --- | --- | --- | --- | --- |
| A. Realm baru `korporat` | Pendaftaran mandiri lewat backend (pola realm `pasien`) dan undangan (pola realm `admin`), MFA TOTP wajib, sesi web 30 menit/12 jam | Kebijakan pendaftaran dan sesi khusus populasi; kesalahan konfigurasi tidak menyebar ke admin atau pasien; sesuai alasan ADR-011 menolak satu realm untuk semua | Satu realm lagi untuk dikonfigurasi, ditemakan, dan diuji (±9 jam-task: F1-T85–F1-T87) | **Dipilih** `[KEPUTUSAN-250]` |
| B. Realm `admin` | HR sebagai jenis akun baru di realm admin RS dan admin rekanan | Tanpa realm baru | Realm `admin` "hanya undangan" (ADR-011 butir 3), padahal perusahaan dapat mendaftar sendiri `[USER] P2-1`; pengguna eksternal anonim dapat membuat akun di realm yang sama dengan Super Admin | Ditolak |
| C. Realm `pasien` | HR memakai akun pasien dengan peran tambahan | HR yang juga peserta cukup satu akun | Batas sesi pasien 30/90 hari (ADR-011 butir 7) tidak cocok untuk web perusahaan; mencampur populasi; HR terpaksa membuat akun pasien | Ditolak |

Konsekuensi opsi A: HR yang juga karyawan peserta memegang dua identitas (realm `korporat` untuk mengelola, realm `pasien` untuk hasilnya sendiri), dan keduanya tidak ditautkan. Ini konsisten dengan BR-143: hasil peserta tidak pernah tampil di member area HR.

**Pemulihan MFA HR (review keamanan B-02, M-41)** `[KEPUTUSAN-265]`: jalur mandiri dengan kode pemulihan didahulukan. Bila kode pemulihan juga hilang, reset dimulai sub-peran admin berwenang (PT-67) setelah konfirmasi lewat kanal berbeda (HR aktif lain di perusahaan yang sama, atau kontak resmi perusahaan yang terdaftar saat verifikasi), lalu mengikuti pola KEPUTUSAN-204: tautan sekali pakai ke email akun ditambah password, TOTP lama tetap berlaku sampai TOTP baru tersimpan (H-16 a), semua sesi dicabut, semua HR aktif perusahaan diberi tahu, E-04 tercatat. Super Admin tidak punya jalur reset kecuali user memutuskannya, sejalan dengan penutupan blokir review Checkpoint 2 untuk pasien (hanya Layanan Pelanggan). HR aktif terakhir yang kehilangan semua faktor hanya dapat dipulihkan lewat jalur ini, dengan konfirmasi ke kontak resmi perusahaan. Diusulkan sebagai revisi ADR-011 butir 5 (daftar jalur "tenaga kesehatan, admin, dan pengguna RME" ditambah realm `korporat`) dan diterapkan di Fase 2 bersama US-207 AC5.

### 6.3 Otorisasi dan isolasi per perusahaan (T1, ADR-012)

- Relasi baru **tenant perusahaan**: HR atas data perusahaannya. `corporate_client_id` diambil dari sesi (keanggotaan aktif `corporate_member`), bukan parameter, mengikuti ADR-012 butir 5 untuk portal rekanan `[KEPUTUSAN-254]`.
- Isolasi di lapisan aplikasi dengan repository yang mewajibkan `CorporateScope`, ditambah test isolasi otomatis untuk setiap endpoint `/v1/corporate/*` di `scripts/verify.sh` (NFR-SEC-06 diperluas). RLS tidak dipakai di database aplikasi, konsisten dengan ADR-012 butir 5 dan alternatif yang ditolak di sana `[KEPUTUSAN-254]`.
- Relasi kepemilikan pasien (ADR-012 butir 1) diperluas: akun pasien atas peserta korporat yang tertaut ke `care_recipient` miliknya.
- Ringkasan hasil adalah data sensitif (§4.1 matriks). Ringkasan hanya dibaca lewat fungsi akses sensitif yang (1) memeriksa persetujuan aktif di `corporate_share_consent_record` pada saat baca, (2) menulis E-05 sebelum data dikirim, dan (3) mencatat penolakan sebagai hasil gagal (ADR-012 butir 2) `[KEPUTUSAN-257]`.
- Admin Ruang Sehat tidak punya jalur baca ringkasan maupun hasil (sejalan §3.1 baris hasil lab).
- **Invarian kanal turunan** (review keamanan B-03, M-42) `[KEPUTUSAN-266]`: HR hanya menerima data yang berasal dari hasil (nilai, ringkasan, ketersediaan hasil, status tindak lanjut, agregat, atau baris tagihan yang bergantung pada hasil) untuk peserta yang persetujuannya aktif saat permintaan. Akibatnya: rekap kehadiran tidak memuat status hasil; rilis ini tidak menyediakan agregat hasil bagi HR, dan agregat yang ditambahkan kelak hanya dihitung atas data yang memang terlihat HR dengan ambang kelompok minimum; baris invoice tidak dapat dikaitkan ke peserta tanpa persetujuan; BFF tidak menyimpan respons berisi ringkasan atau rekap di cache, dan ekspor tidak disimpan di server. Setiap kanal punya test negatif (US-1909 AC6, US-1912 AC5–AC6, US-1914 AC7–AC8).
- Satu akun HR hanya punya satu keanggotaan aktif (UQ parsial), dan pencabutan atau perpindahan keanggotaan mencabut sesi (M-51). Harness isolasi mencakup ekspor, unduhan berkas, pratinjau impor, endpoint ID anak, job asinkron pembangkit berkas, dan route handler BFF `web/korporat` (H-18).

### 6.4 Persetujuan dan data karyawan (T1, UU PDP)

- Ada dua aliran data pribadi yang berbeda dasar pemrosesannya: (1) data peserta yang diunggah HR **sebelum** peserta berinteraksi dengan Ruang Sehat; (2) data peserta setelah tertaut ke akun pasien, yang tunduk pada persetujuan US-104. Dasar pemrosesan aliran (1) dan peran PT. Arunika terhadap perusahaan (pengendali, pengendali bersama, atau prosesor) `[TIDAK DIKETAHUI]` (L-17).
- UU PDP menggolongkan data kesehatan sebagai data pribadi spesifik `[SRC-15]` (via BRD BR-105), dan mendefinisikannya sebagai "catatan atau keterangan individu yang berkaitan dengan kesehatan fisik, kesehatan mental, dan/ atau pelayanan kesehatan" `[SRC-440]`. Ringkasan fit/unfit termasuk definisi itu, sehingga dienkripsi tingkat aplikasi dengan wadah sendiri `[KEPUTUSAN-258]`. Status keikutsertaan dan kehadiran ditanyakan ke legal (§4.1, L-18).
- **DPIA**: UU PDP menyebut pemrosesan data spesifik dan pemrosesan skala besar sebagai pemicu penilaian dampak (Pasal 34 ayat (2) huruf b–c `[SRC-440]`). MCU massal dapat memenuhi keduanya, sehingga DPIA bagian CR-021 diusulkan menjadi syarat gerbang G-26 (data peserta nyata pertama), bukan hanya syarat go-live (BR-110).
- **Hak subjek data peserta yang tidak tertaut**: EP-16 hanya untuk pasien yang masuk aplikasi. Peserta yang menolak atau tidak pernah membuka undangan belum punya kanal akses, perbaikan, penghapusan, atau keberatan atas data unggahan HR; kanal dan pihak yang menanggapi bergantung L-17 (PT-76).
- **Peserta anak**: tanggal lahir diunggah, tetapi aturan untuk peserta di bawah batas usia anak (BR-104, batasnya menunggu L-08) belum ada (PT-77).
- Persetujuan berbagi ringkasan per pesanan, bawaan tidak berbagi, dapat ditarik, berlaku ke depan `[KEPUTUSAN-257]`. Keabsahan persetujuan dalam relasi kerja dan kewajiban medis pemberi kerja atas hasil MCU `[TIDAK DIKETAHUI]` (L-18).
- Minimisasi: NIK hanya bila `requires_nik` paket bernilai benar (PT-66); NIK disimpan dalam tiga bentuk ADR-013 §4; HR hanya melihat 4 digit terakhir.
- Berkas unggahan tidak disimpan setelah impor; pratinjau terenkripsi dan berumur pendek `[KEPUTUSAN-256]`.

### 6.5 Invoice B2B dan jurnal tanpa payment gateway (T1, ADR-015)

- Invoice adalah entitas transaksi immutable dengan riwayat status dan proyeksi (ADR-008); koreksi lewat pembatalan dan invoice pengganti `[KEPUTUSAN-259]`.
- Harga dikunci saat konfirmasi (AB-81) di `corporate_order_price_lock`, memakai aturan markup berlaku-mulai yang sama (data dictionary §0.8).
- Penerimaan transfer dicatat manual oleh Keuangan; jurnal berpasangan: penerbitan invoice (debit piutang korporat, kredit pendapatan markup dan bagian rekanan atau kewajiban kepada rekanan, kredit pajak bila ada), pelunasan (debit kas/bank, kredit piutang). Struktur akun bergantung PT-59 `[KEPUTUSAN-259]`.
- Tidak ada adaptor, notifikasi, atau rekonsiliasi otomatis payment gateway untuk pesanan korporat. Ketersediaan data mutasi bank untuk rekonsiliasi otomatis `[TIDAK DIKETAHUI]`; rilis ini mengandalkan pencatatan manual dan laporan piutang.
- **Mutasi bank dan alokasi** (review keamanan B-05, M-44) `[KEPUTUSAN-268]`: setiap transfer masuk dicatat sekali sebagai `bank_statement_entry` dengan kunci unik global, lalu dialokasikan ke invoice lewat `b2b_payment_allocation`. Jumlah alokasi per mutasi tidak boleh melebihi nominal mutasi, sehingga satu transfer tidak dapat melunasi dua invoice melebihi nilainya. Pemisahan tugas dan rekonsiliasi rekening koran mengikuti keputusan user atas UP-84.
- Berkas bukti transfer hampir pasti memuat nomor rekening lengkap pembayar: disimpan di bucket privat dengan akses hanya Keuangan, dan isinya tidak pernah masuk log maupun audit (US-1910 AC8).
- Refund dan pembatalan korporat tidak tercakup matriks RF-01–RF-18 (PT-63).
- Pajak: komponen pajak sebagai parameter tanpa nilai sampai sisa PT-28 terjawab (G-10); kewajiban faktur pajak B2B dan pemotongan oleh perusahaan klien `[TIDAK DIKETAHUI]` (PT-73, L-21).

### 6.6 Integrasi katalog dan pesanan rekanan (ADR-014)

- Kontrak Partner Integration API v1 belum diterbitkan (EN-21 di Fase 3), sehingga perluasan dapat masuk v1 tanpa versi baru: sumber daya paket korporat (jenis, isi, opsi lokasi, harga, area on-site), webhook pesanan korporat terjadwal dengan daftar peserta minimal, serta status kehadiran, ringkasan, dan hasil per peserta.
- Bila PT-61 memilih portal rekanan, AB-62 harus direvisi. Bila memilih input admin sementara, perlu story backoffice baru (±12 jam-task, estimasi kelas M) dan aturan siapa yang bertanggung jawab atas kebenaran harga.
- RME minimalis sebagai klien kontrak pertama (ADR-014 konsekuensi) harus mendukung pesanan korporat bila rekanan RME boleh melayani korporat (PT-74, BR-126).

### 6.7 Berkas (ADR-018)

- Jenis baru: CSV dan XLSX untuk impor peserta. Validasi isi (bukan ekstensi atau header `Content-Type`), batas ukuran dan baris (PB-23), penolakan berkas terenkripsi, dan pembatasan sumber daya saat membaca XLSX. Pustaka pembaca XLSX diverifikasi di registry resmi saat task (aturan anti-halu §5), belum dipilih.
- Gambar konten situs publik disajikan lewat endpoint publik backend yang hanya melayani gambar milik konten Tayang, bukan bucket publik `[KEPUTUSAN-252]`. Koreksi v0.2: ADR-018 butir 1 sebenarnya **membolehkan** bucket konten CMS dan katalog dibaca publik `[DOK: adr/ADR-018-berkas-unggahan.md]`. Keputusan ini lebih ketat (draf dan gambar konten yang belum atau tidak lagi tayang tidak pernah dapat dibaca publik), sehingga ADR-018 butir 1, H-11 threat model, dan checklist ASVS yang menyebut bucket publik diusulkan direvisi agar tidak tersisa izin bucket publik untuk konten CMS.
- Impor XLSX (review keamanan N-04, M-53–M-55) `[KEPUTUSAN-269]`: penguraian di worker dengan batas waktu dan memori; batas ukuran setelah dekompresi, jumlah entri, dan rasio kompresi diperiksa sebelum mengurai `[SRC-409]`; pemrosesan DTD dimatikan `[SRC-405]` `[SRC-406]`; `.xlsm`, wadah OLE, dan `vbaProject.bin` ditolak; berkas mentah di bucket impor tanpa versi objek dengan aturan kedaluwarsa siklus hidup, dikecualikan dari backup object storage atau berumur sama; pesan galat tidak menggemakan isi sel; penetralan karakter awal rumus (`=`, `+`, `-`, `@`, tab, CR, LF, termasuk varian lebar penuhnya) di semua ekspor CSV `[SRC-407]`.

### 6.8 Email dan notifikasi

- Email Keycloak (verifikasi email, undangan aktivasi HR, atur ulang password) memakai jalur ADR-026 yang sudah diterima.
- L-16 (lokasi data jalur email) kini juga mencakup email karyawan pihak ketiga yang belum menjadi pengguna (undangan peserta).
- Email aplikasi (undangan peserta, invoice terbit, keputusan verifikasi perusahaan) butuh outbox notifikasi dan templat (A19.3, US-1001, UI-04 di Fase 2). Karena itu undangan peserta ada di K2, dan pemberitahuan keputusan verifikasi perusahaan ditunda ke Fase 2 seperti US-201 AC2.

### 6.9 Modul kritis tambahan (usulan untuk `CLAUDE.md`, setelah disetujui)

Realm `korporat` (termasuk jalur reset MFA HR) dan BFF `web/korporat`; render aman konten CMS dan pratinjau backoffice; otorisasi tenant perusahaan; persetujuan berbagi ringkasan, fungsi akses ringkasan, dan invarian kanal turunan; impor data peserta; undangan dan penautan peserta; invoice B2B, mutasi bank, alokasi, dan jurnal korporat; migrasi A20 dan hak role.

---

## 7. ADR baru dan revisi yang diusulkan

| ADR | Status | Isi | Berkas |
| --- | --- | --- | --- |
| ADR-027 (baru) | Diusulkan | Situs publik dan member area korporat: dua aplikasi Next.js, realm `korporat`, penyajian konten dan gambar publik | `docs/adr/ADR-027-situs-publik-dan-member-area-korporat.md` |
| ADR-028 (baru) | Diusulkan | Tenant perusahaan, data peserta, impor berkas, penautan ke akun pasien, persetujuan berbagi ringkasan | `docs/adr/ADR-028-tenant-perusahaan-dan-data-peserta.md` |
| ADR-029 (baru) | Diusulkan; **keputusan inti menunggu PT-59** | Invoice B2B tanpa payment gateway, harga terkunci saat konfirmasi, penerimaan manual, jurnal berpasangan | `docs/adr/ADR-029-invoice-b2b-tanpa-payment-gateway.md` |
| ADR-006 (revisi) | — | Butir 5: empat web Ruang Sehat (backoffice, portal, situs, korporat) | lewat ADR-027 |
| ADR-011 (revisi) | — | Butir 3: realm keenam `korporat`; butir 5: jalur pemulihan perangkat hilang (kini untuk "tenaga kesehatan, admin, dan pengguna RME") ditambah realm `korporat` dengan konfirmasi lewat HR aktif lain atau kontak resmi perusahaan, pola KEPUTUSAN-204, tanpa jalur Super Admin kecuali diputuskan user (KEPUTUSAN-265); butir 7: batas sesi web berlaku untuk `korporat`; bila PT-60 memilih web, klien web realm `pasien` di aplikasi terpisah | lewat ADR-027 |
| ADR-012 (revisi) | — | Butir 1: relasi tenant perusahaan; butir 2: ringkasan sebagai data sensitif dengan syarat persetujuan | lewat ADR-028 |
| ADR-013 (revisi) | — | Wadah data baru (§5) | lewat ADR-028 |
| ADR-015 (revisi) | — | Konteks "semua layanan lewat payment gateway" diberi pengecualian korporat; akun jurnal baru | lewat ADR-029 |
| ADR-018 (revisi) | — | Jenis berkas CSV/XLSX dan pengerasan impor; butir 1: izin bucket konten CMS dibaca publik dicabut, gambar konten publik lewat endpoint backend; bucket impor tanpa versi objek (butir 5) | lewat ADR-027 dan ADR-028 |
| ADR-024 (revisi) | — | Butir 2: lima web Next.js (tiga lama + situs + korporat) | lewat ADR-027 |
| ADR-014 (perluasan) | — | Sumber daya paket korporat dan pesanan korporat di kontrak v1 | dirinci di EN-21 (Fase 3) |

Perubahan atas ADR yang sudah diterima diputuskan langsung oleh user (pola CR-018).

---

## 8. Estimasi effort

Metode `[KEPUTUSAN-178]`, diterapkan `[KEPUTUSAN-260]`:
- Bagian Fase 1 dihitung dari bawah: 31 task berukuran 2–4 jam-task (§10), **109 jam-task** (v0.1: 30 task, 105; ditambah F1-T111 render aman CMS untuk B-01).
- Bagian lain per story dengan kelas S = 6, M = 12, L = 21, XL = 36 jam-task, termasuk layar mobile atau web story itu.
- Satu jam-task = ukuran usaha task termasuk `scripts/verify.sh` dan satu putaran perbaikan review `[ASUMSI-01]`, bukan jam kalender.

| Kelompok | Isi | Jam-task | Dasar |
| --- | --- | --- | --- |
| K1 Fase 1 (dari bawah) | Dokumen gate, desain, realm `korporat`, dua web baru, CMS (US-1101 dipindah + US-1802, termasuk render aman), situs (US-1801), pendaftaran dan verifikasi perusahaan, akun HR, audit, e2e | 109 (31 task) | §10 |
| — dikurangi US-1101 yang sudah ada di Fase 3 | | −12 | `[DOK: 07-phase-plan.md]` §8.2 (US-1101 M) |
| K1-F2 reset MFA HR (US-1904 AC7–AC9), selalu di Fase 2 bersama US-207 AC5 | Jalur reset oleh admin, konfirmasi kanal kedua, pemberitahuan semua HR, test negatif | 12 (M) | kelas; memakai mekanisme US-207 AC5 |
| K2 | US-1905 M 12, US-1906 L 21, US-1907 L 21, US-1908 L 21, US-1909 L 21, US-1910 L 21, US-1911 M 12, US-1912 M 12, US-1913 M 12; enabler jurnal korporat M 12, email aplikasi korporat (outbox, templat undangan dan invoice) M 12; tambahan v0.2: mutasi bank dan alokasi S 6, pengerasan undangan (App Links, batas kirim ulang, konfirmasi tanggal lahir bila disetujui) S 6, test kanal turunan S 6 | 195 (porsi mobile ±18) | kelas per story |
| K3 | US-1914 M 12, US-1915 M 12, US-1917 M 12 | 36 (porsi mobile ±6) | kelas per story |
| **Tambahan bersih (campuran)** | | **±340** | 109 − 12 + 12 + 195 + 36 |
| Pembanding semua dari atas | K1 per kelas: realm M 12, dua web + BFF L 21, migrasi A20 L 21, otorisasi tenant M 12, desain M 12, dokumen S 6, e2e S 6, US-1801 L 21, US-1802 M 12 + render aman S 6, US-1901 M 12, US-1902 M 12, US-1903 S 6, US-1904 L 21, US-1916 M 12 = 192 | ±435 | 192 + 12 + 195 + 36 |

- **Rentang tambahan bersih ±340–435 jam-task**, atau ±19–24% dari baseline Opsi B ±1.791 `[DOK: 07-phase-plan.md]` §6.6 (v0.1: ±306–399). Angka dari bawah cenderung rendah, seperti catatan phase plan untuk 277 jam-task Fase 1 (§7.3).
- Porsi jalur mobile (repositori terpisah, CR-019): ±24 jam-task di K2–K3. Bila PT-60 memilih web untuk peserta, porsi ini pindah ke jalur backend/web, ditambah ±10–15 jam-task untuk klien web realm `pasien` (estimasi penyusun).
- Belum termasuk: story tambahan bila PT-61 memilih input admin (±12) atau menu portal rekanan (±12–21); perubahan bila L-17–L-19 menuntut alur lain; pembatalan/refund korporat (PT-63).
- 19 dari 31 task Fase 1 berkelas T1 (§10).

---

## 9. Dampak ke Fase 1 dan skenario penjadwalan

### 9.1 Kondisi Fase 1 saat ini

- Fase 1 berjalan sejak 2026-11-18, lebih awal dari kalender 2026-11-23 `[DOK: STATUS.md]`. Menurut catatan Fase 1 dan riwayat git branch `tahap-08-fase-1`, task yang sudah ter-commit atau ter-merge per 2026-11-19: F1-T01, T02 (ADR-026 diterima), T03, T04, T05, T07, T08, T09, T10, T11, T12, T13, T14, T15, yaitu 14 task, **±45 jam-task** dari 277 (ukuran §7.3) `[DOK: fase-1-catatan.md]` (v0.3, koreksi fact-check N-2: v0.2 menulis 12 task dan ±37 jam-task, terlewat F1-T14 dan F1-T15 yang sudah di-merge di commit 2487775). F1-T11 disimpulkan dari merge sebelum KEPUTUSAN-243/246 yang bertahap F1-T11 dan dari F1-T12 yang bergantung padanya; review beberapa task belum tercatat lengkap di catatan. Angka ini **indikatif** dan bukan `v` gerbang re-plan, yang dirata-rata atas 10 hari kerja terakhir Fase 1 (§6.5) `[KEPUTUSAN-261]`.
- Sisa baseline Fase 1: **232 jam-task** (backend/web 188, mobile 44). Sisa hari kerja di jendela Fase 1, 2026-11-20 – 2026-12-11, tanpa libur `[DOK: 07-phase-plan.md]` §6.2a: **16 hari kerja**.
- Arahan user: backend dulu, lalu web; Flutter dikerjakan terpisah di laptop dan di repositori terpisah `[USER]` (CR-019, `fase-1-catatan.md`). Seluruh K1 ada di jalur backend/web.
- Laju rencana (bukan laju terukur), sebagai pembanding: Fase 1 18,5 jam-task/hari kerja (backend/web 15,5; mobile 2,9), Fase 2 43,0 (30,8; 12,2), Fase 3 Opsi B 50,3 (≤ 42,3; ≥ 8,0) `[DOK: 07-phase-plan.md]` §6.4–§6.5.

### 9.2 Task Fase 1 yang dapat dipindah ke Fase 2 (dipakai S2 dan S3)

Kriteria: bukan fondasi T1 yang dipakai K1 atau MVP, tidak di jalur kritis, dan story-nya boleh selesai bertahap per AC `[KEPUTUSAN-179]` `[KEPUTUSAN-262]`.

| Task | Isi | Jalur | Jam | Akibat |
| --- | --- | --- | --- | --- |
| F1-T57 | US-105 AC4 ubah email | backend | 3 | US-105 AC4 diverifikasi di Fase 2 |
| F1-T67 | Layar kelola admin backoffice | web | 3 | US-208 AC1–AC3 lewat API dan bootstrap dulu; layar di Fase 2 |
| F1-T75 | Daftar ulang TOTP realm `pasien` dan `admin` | backend | 4 | US-103 AC7 ke Fase 2, bersama varian realm `nakes` (KEPUTUSAN-219) |
| F1-T76 | Ubah password realm `pasien` dan `admin` | backend | 4 | US-103 AC6 ke Fase 2 |
| F1-T79 | Web ubah password dan daftar ulang TOTP backoffice | web | 4 | Mengikuti F1-T75/T76 |
| F1-T77 | Mobile layar "Keamanan akun" dan daftar ulang TOTP | mobile | 4 | Bergantung F1-T75 |
| F1-T78 | Mobile ubah password | mobile | 3 | Bergantung F1-T76 |
| **Jumlah** | | | **25** (backend/web 18, mobile 7) | |

F1-T80 (uji end-to-end CR-018) didefinisikan ulang tanpa langkah ubah password dan daftar ulang TOTP, dan tidak lagi bergantung pada F1-T78/F1-T79. Pendaftaran TOTP, kode pemulihan, masuk dengan OTP, dan atur ulang password ber-OTP (F1-T72–F1-T74) **tetap** di Fase 1, sehingga MFA wajib pasien CR-018 tetap ada di MVP. Risiko: AC ASVS 6.2.2/6.2.3 dan US-103 AC7 baru terpenuhi di Fase 2, masih sebelum go-live.

### 9.3 Skenario

Semua angka dalam jam-task per hari kerja, dengan hari libur dikurangi (§6.2a: 24–25 Desember 2026, 1 dan 5 Januari 2027 di Fase 2). Tanggal pembekuan fitur 2027-01-29 dan go-live tetap `[DOK: 01-business-case.md]` §7.1 (toleransi waktu 0).

**S1 — Fase 1 diperpanjang.** Seluruh K1 masuk Fase 1, tidak ada task Fase 1 yang dipindah. Dengan laju backend/web rencana 15,5, sisa backend/web 188 + 109 = 297 butuh 19,2 hari kerja, sehingga **Fase 1 diperpanjang 4 hari kerja sampai 2026-12-17**. Fase 2 menyusut menjadi 12 hari kerja (2026-12-18 – 2027-01-08) dan menampung K2 dan K1-F2. K3 di Fase 3.

**S2 — Fase 1 tetap selesai 2026-12-11, task dipindah, K2 di Fase 2.** K1 masuk Fase 1; tujuh task §9.2 pindah ke Fase 2; K2 di Fase 2; K3 di Fase 3.

**S3 — Fase 1 tetap selesai 2026-12-11, task dipindah, K2 di Fase 3.** Sama dengan S2, tetapi K2 ditaruh di Fase 3 setelah dependensinya tersedia: jurnal (Fase 2), Partner Integration API (EN-21), dan alur hasil lab (US-901) `[DOK: 07-phase-plan.md]` §8.1–§8.2. Developer tambahan Opsi E membantu mulai Fase 3 `[USER] C2-Q8`.

| Ukuran | Baseline Opsi B | S1 | S2 | S3 |
| --- | --- | --- | --- | --- |
| Akhir Fase 1 | 2026-12-11 | **2026-12-17** (+4 hari kerja) | 2026-12-11 | 2026-12-11 |
| Fase 1, backend/web per hari kerja | 15,5 (sisa: 188 ÷ 16 = 11,8) | 14,9 (297 ÷ 20) | **17,4** (188 − 18 + 109 = 279 ÷ 16) | 17,4 |
| Fase 1, total per hari kerja | 18,5 | 17,1 ((297 + 44) ÷ 20) | 19,8 ((279 + 37) ÷ 16) | 19,8 |
| Hari kerja Fase 2 | 16 | 12 | 16 | 16 |
| Fase 2, total per hari kerja | 43,0 (688 ÷ 16) | **74,6** ((688 + 195 + 12) ÷ 12) | **57,5** ((688 + 25 + 195 + 12) ÷ 16) | **45,3** ((688 + 25 + 12) ÷ 16) |
| Fase 2, backend/web / mobile | 30,8 / 12,2 | 56,8 / 17,8 | 43,8 / 13,8 | 32,7 / 12,6 |
| Fase 3, total per hari kerja (15 hari kerja) | 50,3 (754 ÷ 15) | 51,9 ((754 − 12 + 36) ÷ 15) | 51,9 | **64,9** ((754 − 12 + 195 + 36) ÷ 15) |
| Fase 3, backend/web / mobile | ≤ 42,3 / ≥ 8,0 | ≤ 43,5 / ≥ 8,4 | ≤ 43,5 / ≥ 8,4 | ≤ 55,3 / ≥ 9,6 |
| Yang tersedia di akhir Fase 1 | MVP pasien terverifikasi | MVP + situs, CMS, pendaftaran dan verifikasi perusahaan, akun HR (pemulihan MFA HR mandiri lewat kode pemulihan) | sama dengan S1, tanpa ubah password dan daftar ulang TOTP | sama dengan S2 |
| Reset MFA HR oleh admin (K1-F2) | — | Fase 2 | Fase 2 | Fase 2 |
| Pemesanan, peserta, invoice tersedia | — | akhir Fase 2 (2027-01-08) | akhir Fase 2 (2027-01-08) | akhir Fase 3 (2027-01-29) |
| Ringkasan, hasil, penerusan ke rekanan (K3) | — | Fase 3 | Fase 3 | Fase 3 |

Catatan hitung: 25 = jam-task tujuh task §9.2 yang pindah ke Fase 2 (18 backend/web + 7 mobile); 12 = US-1101 yang keluar dari Fase 3, dan juga K1-F2 yang masuk Fase 2; K2 = 195 (backend/web 177, mobile 18); backend/web Fase 2 S2 = 493 + 18 + 177 + 12 = 700 ÷ 16; mobile Fase 2 S2 = 195 + 7 + 18 = 220 ÷ 16; backend/web Fase 3 S3 = 634 − 12 + 177 + 30 = 829 ÷ 15; mobile Fase 3 S3 = 120 + 18 + 6 = 144 ÷ 15 `[KEPUTUSAN-261]`.

### 9.4 Dampak ke jalur kritis dan gerbang re-plan

- **Jalur kritis Fase 1 (longest path) tetap 65 jam-task** di ketiga skenario. Rantai K1 terpanjang adalah F1-T01 → T04 → T05 → T07 → T13 → T14 → T15 → T17 → F1-T98 → F1-T99 → F1-T100 → F1-T104 → F1-T105 → F1-T106 → F1-T110, selesai paling awal di jam-task ke-51, dengan slack ±11 terhadap F1-T70 (selesai ke-62). F1-T111 (render aman CMS, selesai ke-37) tidak mengubah rantai itu: F1-T95 tetap ke-41 dan F1-T96/F1-T97 ke-41/ke-45. F1-T71 kini bergantung juga pada F1-T110. Dihitung dengan cara §7.4 (earliest finish dari kolom Jam dan Bergantung pada), dengan asumsi persetujuan user atas analisis ini dan ADR-027 tidak menunda F1-T81/F1-T82 `[KEPUTUSAN-261]`.
- **Pembatas nyata adalah kapasitas jalur backend/web**, bukan longest path: K1 seluruhnya di jalur Claude. Di S2 dan S3, task yang dipindah tidak ada di jalur kritis; rantai MFA kehilangan cabang T75 → T77 → T78, sehingga F1-T80 ditentukan oleh F1-T74 (ke-40) dan F1-T73.
- **Persetujuan user menjadi gerbang baru** di depan K1: persetujuan analisis ini (F1-T81), ADR-027 dan ADR-028 (F1-T82, F1-T85), dan PT-67 (F1-T105). Bila tertunda lebih dari ±11 jam-task kerja, rantai K1 menjadi jalur kritis.
- **Gerbang re-plan 2026-12-11.** S2 dan S3: gerbang berjalan dengan Fase 1 lengkap; `v` mencakup task K1 dengan satuan yang sama. S1: gerbang tetap dilaksanakan 2026-12-11 dengan data yang ada (aturan §6.5), sedangkan F1-T71 dan laporan fase pindah ke 2026-12-17. Di semua skenario, gerbang perlu masukan tambahan: total kebutuhan Opsi B + CR-021 (±2.131–2.226 jam-task), keputusan PT-59–PT-61, dan posisi layanan korporat di urutan pengurangan scope (PT-69). Sebagai skala: penghematan Opsi C ±73 dan Opsi D ±454 jam-task `[DOK: 07-phase-plan.md]` §6.6, sehingga menyerap CR-021 tanpa tambahan kapasitas setara dengan pengurangan sebesar Opsi D.

### 9.5 Rekomendasi penyusun

**S3**, dengan syarat:
1. Fase 1 tetap ditutup 2026-12-11 dan gerbang re-plan memakai data Fase 1 yang lengkap.
2. Fase 2 (konsultasi dan pembayaran, layanan yang dipertahankan paling akhir `[USER] T3-Q2`) hanya naik dari 43,0 ke 45,3, dibanding 57,5 (S2) dan 74,6 (S1).
3. K2 dibangun di atas jurnal, Partner Integration API, dan alur hasil yang sudah ada, sehingga harga rekanan tidak perlu jalur sementara (K-04) dan penagihan tidak dibangun sebelum PT-59 dan L-21 terjawab.
4. Beban Fase 3 (64,9, di atas Opsi A 62,0) diputuskan di gerbang re-plan bersama Opsi E (PT-53) dan PT-69.

**Konsekuensi yang harus diterima user:** di S3, member area di Fase 1 berisi pendaftaran, verifikasi, dan akun HR. Pemesanan, peserta, dan invoice baru tersedia di Fase 3 (sebelum pembekuan 2027-01-29). Bila user menghendaki pemesanan sebelum Fase 3, **S2** adalah alternatifnya, dengan risiko Fase 2 naik ±34% (43,0 → 57,5). S1 tidak direkomendasikan karena memotong Fase 2 menjadi 12 hari kerja.

Di ketiga skenario, kebutuhan kecepatan tetap di atas rencana, dan kelayakannya baru dapat dinilai dengan `v` terukur di gerbang 2026-12-11 (§6.5).

---

## 10. Usulan task Fase 1 tambahan (F1-T81–F1-T111)

Kolom mengikuti phase plan §7.3. **Jam** = ukuran 2/3/4 jam-task. **Kelas** T1 memakai Opus effort high dan review penuh dengan test negatif; T2 satu reviewer. **Review** = reviewer selain `code-reviewer`. Semua task baru dimulai setelah user menyetujui CR-021 hasil analisis ini `[KEPUTUSAN-263]`.

| ID | Task | Untuk | Jam | Bergantung pada | Kelas | Review |
| --- | --- | --- | --- | --- | --- | --- |
| F1-T81 | Dokumen gate (1): BRD v1.4 (BR-130–BR-145, peran, scope, non-scope sesuai PT-60, L-17–L-21) dan PRD v1.3 (EP-18, EP-19, AB-80–AB-87, PB-23–PB-25, E-18, §3.1, §1.1, NFR-SEC-08, US-1101 ke Fase 1) | CR-021 | 4 | Persetujuan user atas analisis | T1 | fact-checker, compliance-reviewer |
| F1-T82 | Dokumen gate (2): data dictionary A20 dan perubahan A1.1, A13.1, A15.4, A19.7, B3; skema fisik; ADR-027/ADR-028 ke status Diterima; revisi ADR-011 butir 5 dan ADR-018 butir 1; daftar modul kritis `CLAUDE.md` | CR-021 | 4 | F1-T81, persetujuan ADR | T1 | DBA, Sec |
| F1-T83 | Desain: deskripsi terstruktur dan wireframe sembilan halaman situs (responsif, aksesibilitas guideline §7) | US-1801 | 3 | F1-T81 | T2 | qa-engineer |
| F1-T84 | Desain: member area HR (daftar, menunggu verifikasi, dasbor, kelola HR) dan layar backoffice verifikasi perusahaan serta CMS kanal situs | US-1901–US-1904, US-1802 | 3 | F1-T81 | T2 | qa-engineer |
| F1-T85 | Realm `korporat` sebagai kode: MFA TOTP wajib + kode pemulihan (pola F1-T72), brute force 5/15 menit, sesi web 30 menit/12 jam, Revoke Refresh Token, klien rahasia BFF. Test negatif: login tanpa OTP tidak menerbitkan token, kode pemulihan bekas ditolak; daftar ulang TOTP setelah masuk dengan kode pemulihan (US-1904 AC7) memakai mekanisme F1-T75 dan ikut pindah ke Fase 2 (v0.3, S3) | US-1904, EN-06 | 4 | F1-T31, F1-T72, F1-T82 | T1 | Sec |
| F1-T86 | Validasi token realm `korporat` di backend (issuer, audience namespace `/v1/corporate`), ekstraktor konteks HR. Test: token realm lain dan audience lain ditolak di dua arah | US-1904 | 3 | F1-T32, F1-T85 | T1 | Sec |
| F1-T87 | Tema login realm `korporat` (memakai ulang tema F1-T35/F1-T73), pesan galat seragam UP-73 | US-1904, UI-03 | 2 | F1-T35, F1-T73, F1-T85 | T1 | Sec |
| F1-T88 | Atur ulang password ber-OTP dan uji penguncian di realm `korporat` (pola F1-T55/F1-T74), test negatif | US-1904 | 3 | F1-T55, F1-T74, F1-T85 | T1 | Sec |
| F1-T89 | `web/situs`: kerangka Next.js publik tanpa sesi (standalone, pengerasan ADR-024, CSP tanpa skrip inline dan tanpa sumber pihak ketiga), tanpa identitas OpenBao dan Valkey; jaringan Docker sendiri yang hanya menjangkau backend lewat Caddy `/v1/public/*` (M-46); `verify.sh` mencakup aplikasi baru | US-1801 | 3 | F1-T39, F1-T40 | T2 | Sec |
| F1-T90 | `web/korporat`: kerangka + BFF realm `korporat` (memakai ulang `bff-auth`, cookie berprefiks `__Host-`, CSRF synchronizer token, idle 30 menit, `Cache-Control: no-store` untuk respons data peserta). Test: sesi korporat tidak dapat memanggil API backoffice, portal, atau pasien; token tidak sampai browser | US-1904, EN-11 | 4 | F1-T42, F1-T86, F1-T40 | T1 | Sec |
| F1-T91 | Migrasi `cms_content` dengan kanal dan halaman situs + hak role | US-1101, US-1802 | 3 | F1-T15, F1-T82 | T1 | DBA |
| F1-T92 | Backend CMS (US-1101 AC1–AC4, US-1802 AC1–AC4): draf, pratinjau, jadwal, arsip, soft delete dan pulihkan (E-16), kebijakan sub-peran Konten | US-1101, US-1802 | 4 | F1-T91, F1-T37, F1-T23 | T2 | Sec |
| F1-T93 | Gambar konten lewat upload slot, teks alternatif wajib (US-1802 AC5); endpoint publik gambar tayang lewat backend (bukan bucket publik), batas laju | US-1802 | 3 | F1-T29, F1-T92 | T2 | Sec |
| F1-T94 | Endpoint konten publik tanpa login: hanya konten Tayang, batas laju, ADR-012 butir 3; OpenAPI dan contract test; test draf tidak bocor | US-1801, US-1802 | 3 | F1-T92 | T2 | Sec |
| F1-T95 | Backoffice layar CMS (D-C07 + kanal situs), pratinjau di iframe `sandbox` tanpa `allow-same-origin` dengan render aman F1-T111 (US-1802 AC8); e2e Playwright termasuk payload XSS di pratinjau | US-1101, US-1802 | 4 | F1-T43, F1-T92, F1-T93, F1-T84, F1-T111 | T2 | Sec |
| F1-T96 | Situs halaman Beranda, Tentang Kami, Layanan, Untuk Perusahaan dari endpoint publik dengan render aman F1-T111; halaman tidak ditemukan dan galat umum | US-1801 | 4 | F1-T89, F1-T94, F1-T83, F1-T111 | T2 | — |
| F1-T97 | Situs halaman Rekanan, Artikel (daftar dan detail), Kontak, Unduh aplikasi, Kebijakan privasi dan syarat (bertanda draf); uji aksesibilitas otomatis dan uji tanpa cookie sesi (US-1801 AC5) | US-1801 | 4 | F1-T96 | T2 | — |
| F1-T98 | Migrasi A20 (1): `corporate_client`, `corporate_client_document`, `corporate_verification_decision` + hak role + test katalog | US-1901, US-1902 | 4 | F1-T15, F1-T17, F1-T82 | T1 | DBA, Sec |
| F1-T99 | Migrasi A20 (2): `corporate_member`; *expand* `account` (`corporate_admin`, `korporat`) + hak role | US-1904 | 3 | F1-T16, F1-T98 | T1 | DBA, Sec |
| F1-T100 | `authz` tenant perusahaan: `CorporateScope` dari sesi, kebijakan HR, 404 untuk objek perusahaan lain, harness test isolasi otomatis semua endpoint `/v1/corporate` di `verify.sh` | US-1904 AC3, EN-07 | 4 | F1-T37, F1-T38, F1-T99, F1-T86 | T1 | Sec |
| F1-T101 | Backend pendaftaran perusahaan mandiri (US-1901 AC1, AC3–AC6): akun HR lewat Admin REST + kompensasi, bukti persetujuan, pesan seragam email terdaftar, batas laju; E-04/E-10/E-13. Format NIB/NPWP diverifikasi ke sumber resmi lebih dulu | US-1901 | 4 | F1-T99, F1-T36, F1-T49, F1-T85, F1-T33 | T1 | Sec |
| F1-T102 | Unggah dokumen NIB/NPWP (US-1901 AC2) lewat upload slot, validasi isi, penetralan PDF ADR-018 | US-1901 | 3 | F1-T29, F1-T101 | T1 | Sec |
| F1-T103 | Web korporat: formulir daftar bertahap, unggah dokumen, layar menunggu verifikasi dan ditolak; e2e | US-1901 | 4 | F1-T90, F1-T101, F1-T102, F1-T84 | T2 | Sec |
| F1-T104 | Backend verifikasi perusahaan (US-1902): antrean terlama, buka dokumen (E-05), setujui/tolak dengan `reason_code`, keputusan pertama berlaku, aktivasi menu HR, E-10; test negatif HR dan sub-peran tanpa hak | US-1902 | 4 | F1-T98, F1-T100, F1-T65 | T1 | Sec, DBA |
| F1-T105 | Backend perusahaan dibuatkan admin (US-1903): perusahaan Aktif + undangan aktivasi HR pertama; test undangan kedaluwarsa dan dipakai ulang. Sub-peran sesuai PT-67 | US-1903 | 3 | F1-T104, F1-T36, PT-67 | T1 | Sec |
| F1-T106 | Backoffice: antrean verifikasi perusahaan, panel dokumen, buat perusahaan, daftar perusahaan dan anggota HR (US-1902, US-1903, US-1916 AC1–AC3 bagian K1); e2e | US-1902, US-1903, US-1916 | 4 | F1-T43, F1-T104, F1-T105, F1-T84 | T2 | Sec |
| F1-T107 | Backend akun HR (US-1904 AC2, AC5): undang HR lain, nonaktifkan, HR terakhir, penolakan sesi seketika (`session_revocation`, KU-10), E-10/E-11 | US-1904 | 4 | F1-T100, F1-T33, F1-T36 | T1 | Sec |
| F1-T108 | Web korporat: masuk/keluar, dasbor perusahaan, kelola HR, layar akses ditolak dan akun nonaktif; e2e termasuk sesi idle 30 menit | US-1904 | 4 | F1-T90, F1-T107, F1-T87 | T2 | Sec |
| F1-T109 | Event audit korporat (E-04, E-05, E-10, E-11, E-13, E-16) dengan field PRD §7.2; subjek data HR dan pengurus; test field terlarang tidak masuk | US-1501 | 3 | F1-T22, F1-T101, F1-T104, F1-T107 | T1 | Sec |
| F1-T110 | Uji end-to-end CR-021 Fase 1: situs menampilkan konten CMS → daftar perusahaan → verifikasi email → daftar TOTP → Verifikator menyetujui → HR masuk → undang HR kedua; test isolasi dua perusahaan; rantai audit terverifikasi | CR-021 | 3 | F1-T97, F1-T95, F1-T103, F1-T106, F1-T108, F1-T109, F1-T88 | T1 | Sec |
| F1-T111 | Render aman konten CMS (B-01, M-40): validasi backend saat simpan (HTML mentah ditolak atau dinetralkan, daftar izin skema URL `https`/`mailto`/relatif, gambar hanya dari endpoint publik backend), satu aturan render Markdown tanpa HTML mentah yang dipakai situs dan pratinjau backoffice, CSP tanpa skrip inline untuk backoffice; pustaka Markdown/sanitasi dipilih dengan cek registry dan dicatat di `docs/sources.md`; test payload XSS (US-1802 AC6, AC7, AC9) | US-1802, US-1801 | 4 | F1-T92 | T1 | Sec |

Jumlah: **31 task, 109 jam-task**; 19 berkelas T1 (F1-T81, T82, T85–T88, T90, T91, T98–T102, T104, T105, T107, T109–T111). Semua di jalur backend/web. Perubahan dependensi task lama: F1-T71 bergantung juga pada F1-T110; di S2/S3, F1-T80 tidak lagi bergantung pada F1-T78/F1-T79.

---

## 11. Usulan perubahan dokumen gate (belum diterapkan)

Menunggu persetujuan user. `docs/07-phase-plan.md` dan `docs/03-prd.md` **tidak** diubah dalam analisis ini.

| Dokumen | Usulan |
| --- | --- |
| `docs/02-brd.md` → v1.4 | §3 BR-130–BR-145 dan perubahan lain di §3 analisis ini |
| `docs/03-prd.md` → v1.3 | §4 analisis ini; §1.1 komponen; §2.1 S-20, S-21; §2.3 posisi layanan korporat (PT-69); NFR-SEC-08 jenis berkas CSV/XLSX; NFR-PERF-04 dan NFR-COMP-03 berlaku untuk situs dan member area; §8.1 entitas baru; §12 PT dan L baru |
| `docs/07-phase-plan.md` → v0.3 | F1-T81–F1-T111 di §7.3; K1-F2 (reset MFA HR) di Fase 2; pemindahan F1-T57, T67, T75–T79 ke Fase 2 dan redefinisi F1-T80 (S2/S3); US-1101 ke Fase 1; K2 ke Fase 2 (S2) atau Fase 3 (S3); K3 ke Fase 3; §6 angka effort dan kebutuhan; §7.4 jalur kritis (tetap 65); §9 gerbang baru G-24 (persetujuan CR-021 dan ADR-027/028), G-25 (PT-59–PT-61 sebelum K2), G-26 (L-17–L-19 dan DPIA bagian CR-021 sebelum peserta nyata diproses) |
| `docs/04-data-dictionary.md`, `docs/05-physical-schema.md` | §5 analisis ini (F1-T82) |
| ADR-006, ADR-011, ADR-012, ADR-013, ADR-015, ADR-018, ADR-024 | §7 analisis ini |
| `docs/06-design-guideline.md` | §2 permukaan baru (situs, member area korporat); §10 alur dan deskripsi terstruktur baru (F1-T83, F1-T84) |
| `docs/04-risk-register.md` | RT-38–RT-47 |
| `docs/04-threat-model.md` | **Sudah diperbarui ke v1.3** (2026-11-19) dari Lampiran A `docs/review/CR-021-security.md`: §4.19–§4.22 (SIT, KOR, IMP, B2B), AS-19–AS-21, AK-08, TB-15, TB-16, M-40–M-56, H-18; status diperbarui setelah cek ulang putaran 2 (LULUS DENGAN CATATAN): lingkup CR-021 LULUS BERSYARAT keputusan user atas UP-83, PT-78/UP-87 (KOR-12) dan UP-84 (B2B-01) |
| `docs/05-asvs-checklist.md` | Pustaka sanitasi CMS (F1-T111); pencabutan izin bucket CMS publik |
| `docs/traceability.md`, `docs/test-plan.md` | BR-130–BR-145 → story → test case setelah PRD v1.3 |
| `CLAUDE.md` | Ringkasan proyek (situs dan layanan korporat), modul kritis §6.9 |

---

## 12. Risiko baru

| ID | Risiko | Dampak | Kemungkinan | Mitigasi | Pemilik |
| --- | --- | --- | --- | --- | --- |
| RT-38 | CR-021 menambah ±340–435 jam-task tanpa tambahan waktu (toleransi 0) | Fase 2 atau Fase 3 tidak muat; konsultasi dokter dan psikolog (inti) ikut terancam | Tinggi (kebutuhan sudah di atas rencana sebelum CR, phase plan §6.5) | S3; keputusan kompensasi di gerbang 2026-12-11 (opsi scope, Opsi E, PT-69) | User/CEO |
| RT-39 | Ringkasan atau hasil kesehatan karyawan sampai ke HR tanpa persetujuan sah | Pelanggaran data pribadi spesifik; kepercayaan karyawan | Sedang | AB-84; fungsi akses sensitif yang memeriksa persetujuan saat baca; test negatif US-1914; L-18; DPIA | Tim + Petugas PDP |
| RT-40 | Peserta tertaut ke akun yang salah (email salah ketik HR, tautan diteruskan) | Jadwal, ringkasan, atau hasil sampai ke orang lain | Sedang | AB-85, UP-83, tautan sekali pakai berumur PB-24 | Tim |
| RT-41 | Model penagihan bertentangan dengan BR-041 atau regulasi pembayaran dan pajak | Penagihan dirancang ulang; risiko izin | Sedang | PT-59, L-21 sebelum K2; ADR-029 belum diterima | CEO + legal |
| RT-42 | Pencatatan transfer manual salah atau terlambat; layanan dijadwalkan sebelum lunas | Kerugian uang, sengketa dengan perusahaan | Sedang | AB-83 ditegakkan server; UP-84; entri pembalik; laporan piutang | Keuangan |
| RT-43 | Katalog korporat kosong karena rekanan belum dapat mengirim harga (PT-05, PT-61) | Member area tidak dapat dipakai memesan | Tinggi sampai PT-05 terjawab | PT-61; S3 menaruh katalog setelah EN-21 | User/CEO |
| RT-44 | Peserta enggan membuat akun pasien dengan MFA TOTP wajib (CR-018) untuk layanan massal | Undangan tidak diterima; hasil tidak dapat disampaikan; HR tidak mendapat ringkasan | `[TIDAK DIKETAHUI]` (terkait RT-35) | Panduan di undangan; PT-71 untuk peserta tanpa akun | User |
| RT-45 | Permukaan serangan baru: situs publik, pendaftaran perusahaan anonim, BFF korporat | Spam pendaftaran, enumerasi, kompromi sesi HR (RT-25) | Sedang | Pemisahan `web/situs` tanpa sesi; batas laju; pesan seragam; patch Next.js ≤ 7 hari (KEPUTUSAN-75) | Tim |
| RT-46 | Vaksinasi massal atau layanan on-site oleh klinik/lab tidak diizinkan atau butuh izin/pencatatan khusus | Layanan V/on-site tidak dapat dijual saat go-live | `[TIDAK DIKETAHUI]` | L-19 sebelum katalog berisi layanan nyata | Legal/CEO |
| RT-47 | Berkas CSV/XLSX berbahaya (rumus, berkas terkompresi besar, isi palsu) | Gangguan layanan, injeksi rumus ke HR | Sedang | KEPUTUSAN-256; validasi isi; batas sumber daya; netralisasi ekspor | Tim |

---

## 13. Pertanyaan terbuka dan konfirmasi legal

### 13.1 Pertanyaan terbuka baru (PT-59 dst.)

| ID | Pertanyaan | Memblokir | Pemilik | Tenggat usulan |
| --- | --- | --- | --- | --- |
| PT-59 | Ke rekening siapa perusahaan membayar invoice, dan bagaimana bagian rekanan sampai ke rekanan, mengingat BR-041 (PT. Arunika tidak menampung dana mitra)? Opsi di K-02 | K2 penagihan (US-1909, US-1910), ADR-029 | User + CEO + legal | Sebelum K2 dimulai |
| PT-60 | Peserta memakai aplikasi pasien (mobile) saja, atau juga login di member area web (perlu CR atas non-scope "versi web untuk pasien")? | Kanal US-1908, US-1911, US-1913, US-1915; pembagian jalur mobile/web | User | Gerbang re-plan 2026-12-11 |
| PT-61 | Harga paket korporat masuk lewat integrasi (Fase 3), portal rekanan (CR atas AB-62), atau diinput admin Ruang Sehat sementara? | US-1905; jadwal K2 | User | Gerbang re-plan 2026-12-11 |
| PT-62 | Markup korporat sama dengan markup layanan pasien per rekanan, atau aturan tersendiri? Satuan harga (per peserta atau per paket) dan jumlah peserta minimum? | US-1905, US-1906 | CEO | Sebelum K2 |
| PT-63 | Siapa menetapkan tanggal pelaksanaan dan bagaimana rekanan mengonfirmasi; "14 hari" kalender atau kerja; syarat dapat diubah per perusahaan; akibat lewat jatuh tempo; bayar sebagian atau lebih; perubahan jumlah peserta setelah invoice; pembatalan dan refund oleh perusahaan; penolakan oleh rekanan; nasib pesanan bila perusahaan dinonaktifkan; kapan undangan dikirim (usulan keamanan: setelah lunas, M-50); apakah ada layanan tambahan yang muncul karena hasil (pemeriksaan lanjutan) dan cara menagihnya tanpa membuka hasil ke HR (AB-87) | US-1906–US-1911, US-1916, US-1917 | User | Sebelum K2 |
| PT-64 | Siapa mencatat kehadiran peserta (rekanan lewat integrasi/RME, petugas di lokasi, atau HR), dan apa definisi hadir? | US-1912 | User | Sebelum K2 |
| PT-65 | Apa isi ringkasan (fit/unfit, kategori lain), untuk layanan mana (MCU saja atau juga lab dan vaksinasi), siapa yang menetapkannya (dokter rekanan), dan konfirmasi tafsiran "tanpa persetujuan, HR hanya melihat rekap kehadiran" | US-1913, US-1914 | User + konsultan medis | Sebelum K3 |
| PT-66 | Layanan korporat mana yang mewajibkan NIK, dan apakah peserta wajib verifikasi identitas KTP (AB-01) sebelum dilayani? | US-1905, US-1907, US-1908 | User + legal | Sebelum K2 |
| PT-67 | Sub-peran admin mana yang memverifikasi perusahaan, yang membuat perusahaan ("sales" tidak ada di tujuh sub-peran UP-18), dan yang memulai reset MFA HR (US-1904 AC8)? Perlu sub-peran baru? Apakah perusahaan buatan admin/sales disetujui orang kedua (M-49)? Super Admin tidak diberi jalur reset MFA HR kecuali user memutuskan lain | F1-T104, F1-T105 (Fase 1); K1-F2 (Fase 2) | User | Sebelum F1-T104 |
| PT-68 | Apakah ada lebih dari satu peran di dalam perusahaan (misalnya HR dan keuangan perusahaan), dan berapa HR per perusahaan? | US-1904 AC2 (bawaan: semua HR sama) | User | Sebelum F1-T107 |
| PT-69 | Di posisi mana layanan korporat dalam urutan pengurangan scope P8.3 bila Fase 2–3 tidak muat? | Gerbang re-plan | User/CEO | 2026-12-11 |
| PT-70 | Isi company profile (teks, gambar), rekanan yang boleh ditampilkan beserta izin logonya, apakah halaman Kontak memuat formulir atau hanya informasi, dan domain situs (lanjutan PT-46) | Konten situs saat go-live; formulir kontak di luar story saat ini | User | Sebelum UAT |
| PT-71 | Perlakuan data peserta yang tidak menerima undangan atau tidak jadi ikut, hasil untuk peserta yang belum tertaut, dan satu email di beberapa perusahaan; usulan kategori retensi "peserta korporat tidak tertaut" di PT-24 | US-1908, US-1915, retensi (PT-24) | User + legal | Sebelum K2 |
| PT-72 | Apa yang terjadi bila NIB/NPWP sudah terdaftar perusahaan lain, dan bagaimana perusahaan cabang didaftarkan? | US-1901 AC7 | User | Sebelum F1-T101 |
| PT-73 | Kewajiban faktur pajak dan pemotongan pajak oleh perusahaan klien atas invoice B2B (lanjutan PT-28) | US-1909 isi invoice | CEO | Sebelum K2 |
| PT-74 | Area layanan on-site rekanan, dan apakah rekanan pengguna RME minimalis menerima pesanan korporat (BR-126)? | US-1906 AC5, US-1917 | User | Sebelum K3 |
| PT-75 | Konfirmasi tafsiran: (a) vaksinasi massal korporat tidak membuka kembali CR-015 untuk aplikasi pasien; (b) MCU/lab on-site di kantor tidak termasuk non-scope "pengambilan sampel di rumah" | Scope K2; K-05, K-10 | User | Gerbang re-plan |
| PT-76 | Kanal hak subjek data (akses, perbaikan, penghapusan, keberatan) bagi peserta yang belum atau tidak pernah tertaut ke akun pasien, siapa yang menanggapi (PT. Arunika atau perusahaan, bergantung L-17), dan isi pemberitahuan pemrosesan di email undangan | US-1908 AC1; EP-16 | User + legal | Sebelum K2 (G-26) |
| PT-77 | Apakah peserta di bawah batas usia anak (misalnya pemagang, atau keluarga karyawan pada vaksinasi) boleh diunggah, dan bagaimana persetujuan wali? Batas usia anak menunggu L-08 | US-1907 validasi tanggal lahir; BR-104 | User + legal | Sebelum K2 |
| PT-78 | Usulan keamanan (M-43 butir 2): saat menerima undangan, peserta mengetik tanggal lahirnya untuk dicocokkan dengan data unggahan, dengan batas percobaan; bila gagal, tidak ditautkan dan HR melihat status netral. Disetujui atau ditolak? Bila ditolak, risiko salah tautan karena salah ketik email (KOR-12) dicatat sebagai risiko sisa yang diterima user, kecuali UP-87 disetujui | US-1908 AC4a | User | Sebelum K2 |
| PT-79 | Apakah verifikasi perusahaan menuntut bukti kewenangan pendaftar (misalnya surat penunjukan atau konfirmasi lewat telepon kantor dari sumber independen), selain NIB/NPWP (M-49)? | US-1902; KOR-03 | User | Sebelum F1-T104 |

### 13.2 Konfirmasi legal baru (L-17 dst., lanjutan BRD §6.3)

| # | Topik | BR terkait | Dibutuhkan sebelum |
| --- | --- | --- | --- |
| L-17 | Dasar pemrosesan data karyawan yang diunggah HR sebelum karyawan berinteraksi dengan Ruang Sehat. UU PDP mengenal enam dasar (Pasal 20 ayat (2) huruf a–f `[SRC-440]`): persetujuan, perjanjian, kewajiban hukum, kepentingan vital, kepentingan umum, dan kepentingan sah lainnya, sehingga persetujuan bukan satu-satunya pilihan (K-12). Peran PT. Arunika terhadap perusahaan klien, beserta akibatnya: (a) bila **prosesor**, pemrosesan hanya atas perintah perusahaan, pelibatan rekanan pelaksana butuh persetujuan tertulis perusahaan `[SRC-15]`, kewajiban keamanan dan perekaman ikut berlaku (Pasal 52 `[SRC-440]`), sedangkan kewajiban memberi tahu perusahaan bila terjadi kegagalan pelindungan `[TIDAK DIKETAHUI]` (Pasal 52 tidak memperluas Pasal 46 ke prosesor; umumnya diatur di perjanjian pemrosesan); (b) bila **pengendali bersama** (Pasal 18 ayat (1) `[SRC-440]`), wajib perjanjian peran dan tanggung jawab `[SRC-15]`; (c) bila **pengendali** sendiri, kewajiban pengendali penuh. **Peran rekanan pelaksana** atas data peserta (L-15 hanya mencakup RME dan konsultasi). Kanal hak subjek data bagi peserta yang tidak tertaut (PT-76). Batas data minimal (termasuk NIK) yang boleh diterima dan diteruskan ke rekanan | BR-139, BR-140, BR-145, BR-103 | Data peserta nyata pertama diproses (K2, G-26) |
| L-18 | Persetujuan karyawan untuk membagikan ringkasan hasil MCU/lab ke pemberi kerja: keabsahan persetujuan dalam relasi kerja, rahasia kedokteran, dan pihak yang berwenang menyatakan fit/unfit. Rujukan awal: PER-02/MEN/1980 (berstatus "Berlaku") menyebut kewajiban perusahaan menyelenggarakan pemeriksaan kesehatan tenaga kerja (sebelum kerja, berkala "sekurang-kurangnya satu tahun sekali", khusus) dan "pelaporan hasil kepada otoritas terkait" `[SRC-441]`. Teks pasalnya `[TIDAK DIKETAHUI]`. Untuk MCU yang merupakan kewajiban pemberi kerja, dasar yang tepat mungkin bukan persetujuan, dan BR-143 ("hasil lengkap hanya untuk peserta") **dapat berbenturan** dengan kewajiban lapor itu. Ditanyakan juga: (i) apakah keikutsertaan dan kehadiran dalam layanan kesehatan tergolong data kesehatan (§4.1, kepatuhan B-1, bersama L-08); (ii) akibat penarikan persetujuan atas ekspor yang sudah diunduh HR | BR-143, BR-144, BR-145 | K2 untuk butir (i); K3 (US-1913, US-1914) untuk lainnya |
| L-19 | Regulasi vaksinasi massal (penyelenggara, izin fasyankes, pelaksanaan di luar fasyankes, pengadaan dan rantai dingin vaksin, pencatatan dan pelaporan imunisasi termasuk SATUSEHAT) dan pelayanan MCU/lab on-site oleh klinik/lab di kantor perusahaan, termasuk pengambilan spesimen di kantor. Dari sumber resmi baru diketahui bahwa fasyankes wajib menjalankan RME yang terhubung dengan SATUSEHAT `[SRC-18]`; rincian untuk layanan di luar gedung `[TIDAK DIKETAHUI]`. Juga: kebolehan PT. Arunika menjual paket vaksinasi dengan markup (analog L-13), dan kaitan dengan PSEF/BPOM bila vaksin dianggap produk yang diedarkan lewat platform (L-03). Kaitan dengan pertanyaan CR-015 | BR-133–BR-135, BR-138 | Katalog berisi layanan nyata |
| L-20 | Apakah situs web dan member area korporat tercakup pendaftaran PSE yang sama (L-02) atau perlu pendaftaran/perubahan. PSE lingkup privat mencakup penyelenggara yang "memiliki portal, situs, atau aplikasi" `[SRC-13]`, sehingga yang ditanyakan cukup cakupan pendaftaran yang ada. Kewajiban isi kebijakan privasi dan syarat situs, dan teks persetujuan HR saat mendaftar (US-1901 AC1) | BR-100, BR-130 | Go-live (lebih awal bila memengaruhi pendaftaran PSE); teks persetujuan HR sebelum pendaftaran perusahaan nyata |
| L-21 | Penagihan B2B tanpa payment gateway: (a) opsi seluruh dana ke PT. Arunika berisiko terhadap BR-041 dan BR-115. BI dapat menetapkan aspek prudensial bagi penyelenggara penunjang yang meneruskan pembayaran kepada penyedia barang dan/atau jasa `[SRC-20]`; apakah menerima transfer B2B lalu meneruskan bagian rekanan termasuk aktivitas sistem pembayaran yang butuh izin BI `[TIDAK DIKETAHUI]`. (b) Opsi dua tagihan membuka markup kepada perusahaan (K-02, AB-13). Juga: markup B2B atas tarif rekanan (L-13), kewajiban faktur dan pemotongan pajak (PT-73), dan perjanjian dengan perusahaan klien serta rekanan | BR-138, BR-141, BR-142, BR-115 | K2 penagihan |

L-16 (lokasi data jalur email) diperluas: email undangan dikirim ke karyawan pihak ketiga yang belum menjadi pengguna Ruang Sehat.

---

## 14. `[TIDAK DIKETAHUI]` yang memblokir pembangunan

| Butir | Yang terblokir | Yang tetap dapat dibangun |
| --- | --- | --- |
| PT-59 penerima dana invoice + L-21 | US-1909, US-1910, akun jurnal korporat, ADR-029 | Struktur invoice dan harga terkunci dapat dirancang, tetapi tidak difinalkan |
| PT-60 kanal peserta | Layar peserta (mobile atau web) di US-1908, US-1911, US-1913, US-1915 | Backend undangan dan penautan |
| PT-61 sumber harga | US-1905 dan jadwal K2 | Tabel `corporate_package` dan perhitungan harga |
| PT-63 aturan pesanan dan invoice | Transisi status lengkap, pembatalan, jatuh tempo | Alur utama sampai Lunas |
| PT-64, PT-65 kehadiran dan ringkasan | US-1912 sumber data, US-1914 | Persetujuan US-1913 |
| PT-67 sub-peran admin | F1-T105 (Fase 1); kebijakan F1-T104; reset MFA HR (K1-F2) | F1-T104 dengan sub-peran Verifikator sebagai bawaan sementara bila user menyetujui |
| PT-76, PT-77 hak subjek peserta tidak tertaut, peserta anak | Pemrosesan data peserta nyata (G-26) | Pembangunan dengan data uji |
| PT-78, UP-83, UP-87 pengikatan undangan | Status Tercermin KOR-12; rilis hasil ke akun tertaut | Penautan dengan syarat email sama (UP-83) |
| UP-84 pemisahan tugas uang manual | Status Tercermin B2B-01; ADR-029 Diterima | Mutasi, alokasi, dan jurnal |
| L-17, L-18, L-19 | Pemrosesan data peserta nyata, ringkasan untuk HR, penjualan vaksinasi dan layanan on-site | Pembangunan dengan data uji |
| Format resmi NIB dan NPWP | Validasi format di F1-T101 | Validasi lain; format diverifikasi ke sumber resmi saat task |

---

## 15. Register keputusan teknis analisis ini

KEPUTUSAN-250–KEPUTUSAN-269 tercatat di `docs/decision-log.md` dengan status "diusulkan; menunggu persetujuan user atas CR-021": realm `korporat` (250); dua web baru dan situs tanpa sesi (251); konten dan gambar publik lewat endpoint backend (252, alasan dikoreksi di v0.2); MFA wajib HR (253); isolasi tenant perusahaan di aplikasi tanpa RLS (254); data peserta terpisah dan penautan hanya oleh peserta (255); penanganan berkas impor (256); persetujuan berbagi ringkasan dan fungsi akses (257); wadah enkripsi baru (258); invoice dan jurnal tanpa payment gateway (259); metode estimasi (260); dasar hitung skenario (261); model data A20 dan task yang dipindah (262); kelas, reviewer, dan gerbang task CR-021 (263). Baru di v0.2: render aman CMS, pratinjau sandbox, dan CSP (264); jalur reset MFA HR (265); invarian kanal turunan (266); pengikatan undangan (267); mutasi bank dan alokasi (268); pengerasan situs, BFF korporat, dan impor XLSX (269).

---

## 16. Yang diminta dari user

1. Setujui atau koreksi BR-130–BR-145, EP-18/EP-19, dan UP-79–UP-87 (UP-83 adalah syarat keamanan; penolakan UP-83 atau UP-84 dicatat sebagai risiko sisa yang Anda terima).
2. Putuskan K-01–K-05 lewat PT-59, PT-60, PT-61, PT-75, serta PT-78 (konfirmasi tanggal lahir saat menerima undangan) dan PT-79 (bukti kewenangan pendaftar).
3. Pilih skenario jadwal: S3 (rekomendasi), S2, atau S1.
4. Setujui ADR-027 dan ADR-028 (diperlukan sebelum F1-T85 dan F1-T98) setelah cek ulang review keamanan; ADR-029 menunggu PT-59 dan UP-84. Setujui juga usulan revisi ADR-011 butir 5 (jalur reset MFA HR) dan ADR-018 butir 1.
5. Jawab PT-67 sebelum F1-T104 (termasuk sub-peran yang memulai reset MFA HR).
6. Tunjuk pemilik konfirmasi legal L-17–L-21 (melanjutkan PT-20).

Setelah persetujuan, orkestrator menjalankan F1-T81/F1-T82 (pembaruan dokumen gate), fact-check, dan review keamanan serta DBA atas ADR-027–ADR-029 sesuai T1.

---

## 17. Keputusan yang dibutuhkan

Ringkasan untuk user (2026-11-19). Review keamanan putaran 2: LULUS DENGAN CATATAN. Review kepatuhan: B-1 sudah diperbaiki.

**1. Jadwal.** Satuannya jam-task per hari kerja. Rencana awal: Fase 2 butuh 43,0, Fase 3 butuh 50,3 (§9.3).
- **S1**, Fase 1 diperpanjang sampai 2026-12-17: Fase 2 menyusut jadi 12 hari kerja dan butuh 74,6.
- **S2**, Fase 1 tetap selesai 2026-12-11 dan 7 task pindah ke Fase 2: pemesanan, peserta, dan invoice jadi di Fase 2. Fase 2 butuh 57,5 (+34%).
- **S3 (usulan penyusun)**: sama dengan S2, tetapi pemesanan, peserta, dan invoice jadi di Fase 3. Fase 2 butuh 45,3, Fase 3 butuh 64,9.
- Di ketiga skenario: situs, CMS, pendaftaran dan verifikasi perusahaan, dan akun HR jadi di Fase 1. Tambahan kerja ±340–435 jam-task (+19–24%). Keputusan kompensasi diambil di gerbang 2026-12-11.

**2. Kontradiksi dengan keputusan lama** (pilihan → usulan penyusun):
- **K-01**, versi web untuk pasien: (a) peserta memakai aplikasi mobile, atau (b) CR untuk web peserta → **(a)**.
- **K-02**, PT. Arunika tidak menampung dana mitra: (a) semua dana ke PT. Arunika, (b) dua tagihan, atau (c) virtual account → tunggu L-21. Bila izin BI dibutuhkan untuk (a), pilih **(b)**, yang juga butuh pengecualian AB-13 karena markup terlihat.
- **K-03**, semua layanan dibayar lewat payment gateway → **setujui BR-141** sebagai pengecualian untuk korporat.
- **K-04**, portal rekanan hanya baca: harga lewat (a) integrasi, (b) portal, atau (c) input admin → **(a)**, sejalan dengan S3.
- **K-05**, vaksinasi dijadwalkan ke rilis berikutnya → **konfirmasi** vaksinasi korporat saja; vaksinasi untuk pasien tetap mengikuti CR-015 (PT-75).
- **K-12**, persetujuan sebelum pemrosesan (BR-103) → dasar pemrosesan unggahan HR ditetapkan legal (L-17); BR-103 tetap berlaku setelah peserta tertaut ke akun.
- **K-06–K-11** → **setujui**: CMS pindah ke Fase 1, revisi ADR-006/011/018/024, unggah CSV/XLSX, tafsiran on-site; BR-126 ditanyakan lewat PT-74.

**3. Usulan detail produk.** Default usulan: setujui semua.
- UP-79 formulir pendaftaran perusahaan.
- UP-80 status pesanan dan invoice.
- UP-81 harga dikunci saat konfirmasi.
- UP-82 impor dua langkah.
- UP-83 email harus sama saat menerima undangan (syarat keamanan).
- UP-84 pemisahan tugas pencatatan uang manual.
- UP-85 teks seragam "ringkasan tidak tersedia".
- UP-86 event audit E-18.
- UP-87 hasil baru dirilis setelah identitas dicocokkan di lokasi.
- Menolak UP-83, UP-84, atau UP-87 berarti risiko sisanya dicatat sebagai risiko yang Anda terima. Status keamanan KOR-12 dan B2B-01 bergantung pada keputusan ini.

**4. Yang memblokir pembangunan Fase 1:**
- Persetujuan analisis ini dan skenario jadwal (F1-T81).
- Persetujuan ADR-027 dan ADR-028, serta revisi ADR-011 butir 5 dan ADR-018 butir 1 (F1-T82, F1-T85, F1-T98).
- **PT-67**: sub-peran admin yang memverifikasi perusahaan, membuatnya, dan memulai reset MFA HR (F1-T104, F1-T105).
- **PT-79**: perlukah bukti kewenangan pendaftar (F1-T104).
- **PT-72**: NIB yang sudah terdaftar (F1-T101).
- PT-68 (peran di dalam perusahaan) punya bawaan "semua HR sama", jadi tidak memblokir.
- PT-59–PT-66, PT-76–PT-78, dan L-17–L-21 baru memblokir Fase 2/3 (atau G-26).
