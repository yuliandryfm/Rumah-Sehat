# Review Kepatuhan Tahap 3: PRD Rumah Sehat v1.0 dan BRD v1.2 (CR-006)

Tanggal: 2026-11-17 (review awal dan cek ulang)
Reviewer: compliance reviewer independen (Claude). **Ini bukan nasihat hukum.** Temuan yang sumbernya hanya terbaca sebagian berstatus **indikasi** dan wajib dikonfirmasi tim legal/compliance manusia.
Tingkat ketelitian: **T1** untuk data pribadi dan kesehatan, RME minimalis, audit trail, dan retensi.
Kriteria blokir: `aturan-kerja.md` §2. Non-blokir dibatasi 5 butir terpenting.

Dokumen yang direview:
- `docs/02-brd.md` v1.2, hanya perubahan Tahap 3 (CR-006 dan jawaban T3-Q). Hasil review `docs/compliance/tahap-02.md`, `tahap-02-cr-005.md`, dan `tahap-02-riset.md` yang sudah lulus tidak diulang.
- `docs/03-prd.md` v1.0: §3.1, §3.2, §3.5 (sebatas penandaan L-14), §3.11, US-104, US-106, US-108, US-109, US-201, US-308, US-310, EP-13, EP-14 (aliran data), EP-15, EP-16, §6.4–§6.6, §7, §8.2, §9, §10, §11, §12.3.
- `docs/03-catatan-prd.md`, `docs/sources.md` (SRC-26 sampai SRC-29), `docs/decision-log.md`, `docs/traceability.md`.

**Cara menentukan baris yang berubah.** Reviewer tidak punya akses shell, sehingga `git diff main` tidak dijalankan. Baris BRD yang berubah ditentukan dari penanda "CR-006" dan "T3-Q", kolom status "BRD v1.2" di `docs/traceability.md`, serta catatan wawancara Tahap 3. Baris BRD yang diperiksa: 4, 7, 11, 41, 74, 125–127, 151–159, 170, 175–176, 189–190, 203, 225, 242, 249, 267–277, 289, 327–339, 380, 396, 410, dan 430–434.

Nomor baris di bagian 1 sampai 6 merujuk ke versi dokumen saat review awal. Baris setelah perbaikan dicantumkan di bagian 7.2.

## Ringkasan

| Aspek | Hasil |
| --- | --- |
| SRC-26, SRC-27, SRC-28, SRC-29 | Dibuka ulang hari ini, dan semuanya mendukung klaim yang memakainya. SRC-27 memuat dua syarat WAJIB tambahan yang belum tercermin di PRD (N03) |
| Klaim regulasi baru di BRD v1.2 (BR-123, §6.1) | Sesuai sumber. SRC-18 dibuka ulang dan memuat "Fasyankes wajib terhubung melalui platform terintegrasi dengan SATUSEHAT". Kewajiban PT. Arunika sebagai penyedia RME tidak diklaim pasti dan diteruskan ke L-05 |
| Persetujuan anggota keluarga (BR-008, AB-04, BR-104) | Ditandai L-08 dan tidak diklaim cukup. Untuk anak sejalan dengan Pasal 25. Untuk dewasa ada risiko tinggi terhadap Pasal 20, 22, dan 24, dan akses pemilik akun ke data kesehatan anggota keluarga dewasa belum dibawa ke legal (N01) |
| Kewajiban PDP baru dari CR-006 | DPIA sudah mencakup RME minimalis (PRD §10). Belum tercatat: konsekuensi hubungan pengendali/prosesor, termasuk perjanjian (N02), serta dasar pemrosesan pengiriman data ke SATUSEHAT (N05) |
| EP-16, §7, retensi | Tidak ada klaim yang melebihi sumber. Ada satu kekurangan terhadap sumber: rekam jejak pemrosesan pada hak akses (N04) |
| Layar persetujuan pasien (US-104) | Daftar isi layar tidak lengkap terhadap Pasal 21 ayat (1), padahal BR-103 yang sudah lulus gate mewajibkan "informasi yang diwajibkan UU PDP" (**C3-B01**) |
| L-14 di §3.5 (refund) | Ditandai dengan benar |
| Nomor SRC, KEPUTUSAN, dan UP yang dirujuk | Semuanya terdaftar: SRC-15, 18, 24–29; KEPUTUSAN-03 sampai 15; UP-01 sampai 48 |
| Blokir | Review awal: **1** (C3-B01). Setelah cek ulang: **0** (bagian 7.2) |
| **Status** | **LULUS** setelah cek ulang. Status review awal: TIDAK LULUS |

## 1. Jawaban atas pertanyaan kunci

**1. Apakah kewajiban PT. Arunika sebagai penyedia sistem RME sudah teridentifikasi dan diteruskan ke legal tanpa diklaim pasti?** Sebagian besar sudah.

- **Registrasi SATUSEHAT** teridentifikasi di BR-123, US-1309 AC3, §9, §10, dan PT-32, dengan SRC-27 yang benar. Tidak ada klaim bahwa registrasi itu wajib. BR-123 menulisnya sebagai langkah ("penyedia sistem RME mendaftarkan sistemnya ke SATUSEHAT"), dan halaman SRC-27 memang tidak menyatakan registrasi itu wajib. Kewajiban terhubung ke SATUSEHAT melekat pada fasyankes `[SRC-18]`. Masih ada kekurangan pada daftar syarat berkas (N03).
- **PSE**: kewajiban nomor PSE untuk sistem RME sudah teridentifikasi. Namun BR-100 hanya menyebut "Rumah Sehat", L-02 belum menyebut RME minimalis, dan tenggat L-02 (Tahap 12) lebih lambat daripada PT-32 (N03).
- **Pengendali versus prosesor**: sudah ditanyakan di L-05 ("pengendali data rekam medis di RME minimalis (fasyankes atau PT. Arunika)") tanpa klaim pasti. Konsekuensi untuk tiap kemungkinan jawaban belum diturunkan (N02).
- **Keamanan**: NFR-SEC berlaku untuk RME minimalis, yaitu MFA (NFR-SEC-02), sesi (NFR-SEC-04), dan isolasi antar-fasyankes (NFR-SEC-06, US-1302 AC3). Operator RME tidak dapat melihat rekam medis (§3.1, US-1301 AC3). Survei keamanan SATUSEHAT sudah tercatat sebagai prasyarat. Rekomendasinya ada di bagian 4 butir 6.

**2. Apakah persetujuan anggota keluarga sudah ditandai L-08 dan tidak bertentangan dengan UU PDP?**

- **Penandaan:** sudah. BR-008 (BRD baris 176), AB-04 (PRD baris 141), L-08 (BRD baris 333), dan §12.3 (PRD baris 1861) menyatakan kecukupannya menunggu legal.
- **Anak:** BR-104, AB-04, dan US-108 AC2 sejalan dengan Pasal 25 ayat (2): "wajib mendapat persetujuan dari orang tua anak dan/ atau wali anak sesuai dengan ketentuan peraturan perundang-undangan" `[SRC-15]`.
- **Dewasa:** PRD tidak menyatakan caranya sudah cukup, sehingga tidak ada klaim yang bertentangan dengan sumber. Namun desainnya tegang dengan teks Pasal 20 ayat (2) huruf a, Pasal 22 ayat (1), dan Pasal 24 (N01).
- **Belum ditandai sama sekali:** akses pemilik akun ke data kesehatan anggota keluarga dewasa (N01).

**3. Apakah ada kewajiban PDP baru dari CR-006 yang belum tercatat?**

| Kewajiban | Status | Catatan |
| --- | --- | --- |
| DPIA yang mencakup RME minimalis | Sudah, di PRD §10 baris 1768 ("DPIA, termasuk RME minimalis dan data anggota keluarga") | Pihak yang wajib melakukan DPIA atas data RME bergantung pada L-05. Pasal 34 ayat (1) membebankan DPIA kepada "Pengendali Data Pribadi", dan Pasal 52 tidak menyebut Pasal 34 sebagai kewajiban yang berlaku juga bagi prosesor `[SRC-15]` (N02) |
| Perjanjian pemrosesan dengan fasyankes | Belum | N02 |
| Pengiriman data ke SATUSEHAT | Tercatat di audit (E-17) dan UP-28 | Dasar pemrosesan dan pihak yang memberi informasi kepada pasien belum ditanyakan (N05) |

**4. Apakah hak subjek data, audit trail, dan retensi tidak mengklaim melebihi sumber?** Tidak ada yang melebihi sumber.

- US-1603 AC4 membedakan dengan benar batas 3 x 24 jam yang diatur UU PDP dari target internal UP-47.
- Retensi data dan retensi audit log ditulis `[TIDAK DIKETAHUI]` (NFR-RET-02, §7.4).
- US-1605 dan US-1606 dinyatakan terblokir sampai L-06 dan PT-24 terjawab.
- Kekurangannya justru ke arah sebaliknya: rekam jejak pemrosesan pada hak akses belum dimuat (N04).

## 2. Blokir

| ID | Lokasi | Masalah | Sumber | Usulan perbaikan |
| --- | --- | --- | --- | --- |
| C3-B01 | `docs/03-prd.md` baris 379 (US-104 AC1); terkait `docs/02-brd.md` baris 303 (BR-103) | AC1 menetapkan isi layar persetujuan: tujuan, jenis data, masa retensi, hak subjek data, dan kontak. Pasal 21 ayat (1) mewajibkan pengendali menyampaikan tujuh informasi bila pemrosesan berdasarkan persetujuan. AC1 tidak memuat tiga di antaranya: huruf a "legalitas dari pemrosesan Data Pribadi", huruf e "rincian mengenai Informasi yang dikumpulkan", dan huruf f "jangka waktu pemrosesan Data Pribadi". Unsur "relevansi" di huruf c juga tidak ada. BR-103 yang sudah lulus gate mewajibkan persetujuan "dengan informasi yang diwajibkan UU PDP". Akibatnya, layar yang lulus AC1 bisa tidak memenuhi BR-103. Ini masuk kategori 2 dan 3, di bagian T1 | `[SRC-15]` Pasal 21 ayat (1): "Dalam hal pemrosesan Data Pribadi berdasarkan persetujuan sebagaimana dimaksud dalam Pasal 20 ayat (2) huruf a, Pengendali Data Pribadi wajib menyampaikan Informasi mengenai:" huruf a–g. Halaman dibuka ulang hari ini | Ganti AC1 menjadi: "Given pasien mendaftar, When layar persetujuan tampil, Then layar memuat informasi Pasal 21 ayat (1) UU PDP: legalitas (dasar) pemrosesan, tujuan pemrosesan, jenis dan relevansi data yang diproses termasuk data kesehatan, jangka waktu retensi dokumen yang memuat data pribadi, rincian informasi yang dikumpulkan, jangka waktu pemrosesan, dan hak subjek data, ditambah kontak Rumah Sehat. Teks legalitas menunggu L-08; teks retensi dan jangka waktu pemrosesan menunggu PT-24; semuanya wajib terisi sebelum go-live `[SRC-15]`." Cek ulang cukup memeriksa baris ini. **Status: tertutup (bagian 7.2)** |

## 3. Non-blokir (5 terpenting)

| ID | Lokasi | Masalah | Sumber | Rekomendasi |
| --- | --- | --- | --- | --- |
| C3-N01 | PRD baris 126–127 (§3.1), 140–141 (AB-03, AB-04), 302 (AB-71), 403 (US-106 AC4), 420 (US-108 AC1), 644–646 (US-310), 1416–1418 (US-1602), 1435 (US-1604), 1590 (E-05), 1795 dan 1797 (UP-15, UP-17), 1820 (UP-40); BRD baris 176 (BR-008), 333 (L-08) | **Data anggota keluarga dewasa: persetujuan dan akses pemilik akun.**<br>(a) Untuk anggota keluarga dewasa, Rumah Sehat hanya menyimpan pernyataan pemilik akun, bukan persetujuan yang diberikan subjek sendiri. Informasi Pasal 21 juga tidak pernah sampai ke subjek. Butir ini sudah ditandai L-08, tetapi kalau legal menyatakan persetujuan harus datang langsung dari subjek, US-108 dan model datanya berubah.<br>(b) §3.1, AB-71, dan UP-15 memberi pemilik akun akses ke isi chat, catatan, resep, dan hasil lab anggota keluarga dewasa, termasuk konsultasi psikolog. US-1602 AC1 dan US-1604 juga memungkinkan pemilik akun meminta salinan data anggota keluarga dewasa. Sementara itu E-05 (KEPUTUSAN-07) tidak mencatat akses oleh pemilik akun. Alasan UP-15, "Hak akses data (BR-106)", tidak didukung sumber, karena Pasal 7 memberi hak akses atas "Data Pribadi tentang dirinya". Akses ini belum disebut di L-08 maupun §12.3.<br>(c) UP-40 membolehkan satu NIK anggota keluarga tercatat di beberapa akun. Belum diatur apakah akun A dapat melihat data layanan yang dipesan akun B untuk orang yang sama.<br>(d) AB-03 membedakan "dewasa" dan "anak" tanpa batas usia. UU PDP tidak mendefinisikan "anak" (Pasal 1 hanya berisi angka 1–11), dan Pasal 25 ayat (2) merujuk ke "ketentuan peraturan perundang-undangan" | `[SRC-15]`:<br>- Pasal 20 ayat (2) huruf a: "persetujuan yang sah secara eksplisit dari Subjek Data Pribadi ... yang telah disampaikan oleh Pengendali Data Pribadi kepada Subjek Data Pribadi"<br>- Pasal 22 ayat (1): "tertulis atau terekam"; ayat (5): "batal demi hukum"<br>- Pasal 24: "wajib menunjukkan bukti persetujuan yang telah diberikan oleh Subjek Data Pribadi"<br>- Pasal 7: hak akses atas "Data Pribadi tentang dirinya" | (1) Perluas L-08, dan L-05 untuk aspek rahasia kedokteran, dengan:<br>- kecukupan pernyataan pemilik akun terhadap Pasal 24;<br>- akses pemilik akun ke data kesehatan anggota keluarga dewasa;<br>- permintaan hak subjek oleh pemilik akun atas nama anggota keluarga dewasa;<br>- batas usia anak dan perlakuan saat anak menjadi dewasa;<br>- satu NIK yang tercatat di beberapa akun.<br>(2) Untuk anggota keluarga dewasa, ganti alasan UP-15 (jangan merujuk BR-106), dan tampilkan secara eksplisit di gate sebagai keputusan user.<br>(3) Sambil menunggu L-08: catat akses pemilik akun ke data klinis anggota keluarga dewasa di E-05. Siapkan juga di US-108 opsi konfirmasi langsung oleh anggota keluarga dewasa (misalnya tautan ke email atau HP miliknya), supaya jawaban L-08 tidak merombak model data |
| C3-N02 | BRD baris 271–277 (BR-120–BR-126), 330 (L-05); PRD baris 128 (§3.1), 1437 (US-1604 AC3), 1768 (§10), 1859 (§12.3) | **RME minimalis: hubungan pemrosesan PT. Arunika dengan fasyankes belum diturunkan menjadi kebutuhan.**<br>L-05 sudah menanyakan siapa pengendali, dan itu benar. Konsekuensi untuk tiap jawaban belum tercatat:<br>- Bila pengendalinya bersama-sama, perjanjian antar-pengendali wajib ada.<br>- Bila PT. Arunika prosesor, ia memproses berdasarkan perintah fasyankes, wajib meminta persetujuan tertulis sebelum melibatkan prosesor lain (misalnya penyedia cloud), dan memikul sebagian kewajiban pengendali (Pasal 52).<br>Yang juga belum ada:<br>- perjanjian dengan fasyankes sebelum RME aktif;<br>- batasan agar data RME tidak dipakai untuk keperluan Rumah Sehat;<br>- serah terima atau ekspor rekam medis saat fasyankes berhenti memakai RME;<br>- jalur permintaan hak subjek atas data RME. Di sini ada inkonsistensi internal: §3.1 baris 128 menyatakan aksesnya "Lewat hak akses data (EP-16)", sedangkan US-1604 AC3 menyatakan `[TIDAK DIKETAHUI]`;<br>- penjelasan kepada fasyankes bahwa RME minimalis hanya mencatat pasien Rumah Sehat (BR-126), sedangkan kewajiban RME fasyankes `[SRC-18]` berlaku juga untuk pasien lainnya | `[SRC-15]`:<br>- Pasal 1 angka 4 dan 5 (definisi pengendali dan prosesor)<br>- Pasal 18 ayat (2) huruf a: "terdapat perjanjian antara para Pengendali Data Pribadi yang memuat peran, tanggung jawab, dan hubungan antar-Pengendali Data Pribadi"<br>- Pasal 51 ayat (1): "berdasarkan perintah Pengendali Data Pribadi"; ayat (3): tanggung jawab pengendali; ayat (5): "wajib mendapatkan persetujuan tertulis" sebelum melibatkan prosesor lain<br>- Pasal 52: kewajiban di Pasal 29, 31, 35–39 "berlaku juga terhadap Prosesor Data Pribadi"<br>- Pasal 34 ayat (1) | (1) Perluas L-05, atau buat L-15 baru, dengan butir-butir di kolom Masalah.<br>(2) Calon BR untuk diputuskan user di gate:<br>- perjanjian penggunaan RME minimalis dan pemrosesan data dengan setiap fasyankes, sebagai prasyarat US-1301 AC1;<br>- ekspor rekam medis milik fasyankes saat kerja sama berakhir.<br>(3) Samakan §3.1 baris 128 dengan US-1604 AC3: `[TIDAK DIKETAHUI]`, menunggu L-05 |
| C3-N03 | PRD baris 1173 (US-1301 AC1), 1250 (US-1309 AC3), 1756 (§9), 1765 dan 1773 (§10), 1853 (PT-32); BRD baris 300 (BR-100), 327 (L-02) | **Prasyarat registrasi SATUSEHAT dan pendaftaran PSE untuk sistem RME belum lengkap.**<br>(a) SRC-27 memuat dua syarat WAJIB di bagian berkas administrasi yang belum tercantum di PRD: kode registrasi fasyankes untuk fasyankes yang sudah bermitra, dan surat pernyataan resmi bila belum bermitra. Dampaknya, PT-32 membutuhkan daftar fasyankes pengguna atau surat pernyataan, dan fasyankes pengguna RME perlu memiliki kode registrasi fasyankes.<br>(b) Yang didaftarkan ke PSE adalah sistem RME-nya. BR-100 hanya menyebut "Rumah Sehat", dan L-02 tidak menyebut RME minimalis.<br>(c) Tenggat L-02 adalah Tahap 12, padahal PT-32 dimulai setelah gate Tahap 3 dan §10 menempatkan pendaftaran PSE sebelum registrasi SATUSEHAT | `[SRC-27]`, dibuka ulang hari ini:<br>- "WAJIB menyertakan kode registrasi fasyankes terbaru sesuai KMK"<br>- "Apabila belum bermitra dengan fasyankes tertentu, maka WAJIB melampirkan surat pernyataan resmi (diketahui/ditanda tangani oleh Pimpinan)."<br>- "WAJIB mendaftarkan sistem RME-nya ke PSE KOMINFO untuk mendapatkan Nomor PSE." | (1) Lengkapi US-1309 AC3, §9, dan PT-32 dengan dua syarat itu `[SRC-27]`.<br>(2) US-1301 mencatat kode registrasi fasyankes; rinciannya ditetapkan di Tahap 4.<br>(3) BR-100 atau §10 menyebut pendaftaran PSE untuk aplikasi RME minimalis.<br>(4) Perluas L-02 dengan pertanyaan apakah satu pendaftaran PSE mencakup kedua sistem, dan majukan bagian PSE dari L-02 ke Tahap 4 |
| C3-N04 | PRD baris 1435 (US-1604 AC1), 382 (US-104 AC4) | **Hak akses dan perubahan informasi persetujuan.**<br>(a) Isi salinan di US-1604 AC1 tidak memuat rekam jejak pemrosesan.<br>(b) US-104 AC4 baru meminta persetujuan ulang saat pasien membuka aplikasi setelah versi teks diperbarui, padahal perubahan informasi wajib diberitahukan sebelum terjadi.<br>Tidak ada BR yang sudah lulus gate yang dilanggar, sehingga butir ini non-blokir | `[SRC-15]`:<br>- Pasal 32 ayat (1): akses "terhadap Data Pribadi yang diproses beserta rekam jejak pemrosesan Data Pribadi"<br>- Pasal 21 ayat (2): wajib memberitahukan "sebelum terjadi perubahan Informasi" | (1) US-1604 AC1 menambahkan rekam jejak pemrosesan. Datanya dapat bersumber dari field subjek data di audit trail (§7.2) dan event E-17; bentuknya ditetapkan di Tahap 4.<br>(2) US-104 AC4: pemberitahuan perubahan dikirim sebelum tanggal berlaku versi baru, dan tanggal berlakunya ditampilkan |
| C3-N05 | PRD baris 1243–1250 (US-1309), 1272 (EP-14), 1602 (E-17), 1808 (UP-28), 125 (§3.1 NIK) | **Pengiriman data ke SATUSEHAT dan ke sistem rekanan: dasar pemrosesan dan informasi kepada pasien.**<br>E-17 sudah mencatat setiap pengiriman, sesuai BR-107. Yang belum ditanyakan ke legal:<br>- dasar pemrosesan pengiriman rekam medis ke SATUSEHAT oleh RME minimalis atas nama fasyankes. Salah satu kandidatnya Pasal 20 ayat (2) huruf c, mengingat `[SRC-18]`; ini interpretasi untuk dinilai legal, bukan kesimpulan;<br>- siapa yang menyampaikan informasi Pasal 21 kepada pasien tentang aliran data Rumah Sehat → rekanan atau RME → SATUSEHAT.<br>Pada 100.000 karakter pertama SRC-28 tidak ada pernyataan tentang persetujuan pasien; sisa halaman belum dibaca. NIK di RME ditulis `[TIDAK DIKETAHUI]`, dan itu sudah benar | `[SRC-15]` Pasal 20 ayat (2) huruf c: "pemenuhan kewajiban hukum dan Pengendali Data Pribadi sesuai dengan ketentuan peraturan perundang-undangan"; `[SRC-18]`; `[SRC-28]` | (1) Tambahkan ke L-05 dan L-08: dasar pemrosesan pengiriman data ke SATUSEHAT dan ke sistem rekanan, serta apakah teks persetujuan US-104 perlu menyebut penerima data.<br>(2) Di Tahap 4, baca halaman SATUSEHAT tentang pasien (NIK dan IHS number) dan tentang persetujuan sebelum merancang US-1309 |

## 4. Rekomendasi tambahan untuk daftar konfirmasi legal

Semua butir L-01 sampai L-14 tetap berlaku sesuai `docs/compliance/tahap-02.md` bagian 4 dan `tahap-02-cr-005.md` bagian 3. Tambahan dari Tahap 3:

1. **L-05, sebelum Tahap 4, diperluas dengan:**
   - pengendali bersama atau prosesor untuk RME minimalis, dan juga untuk catatan konsultasi di Rumah Sehat;
   - isi perjanjian yang dibutuhkan (Pasal 18, 51, 52);
   - persetujuan tertulis bila memakai sub-prosesor;
   - serah terima rekam medis saat kerja sama berakhir;
   - jalur hak subjek atas data RME;
   - dasar pengiriman ke SATUSEHAT;
   - posisi RME minimalis yang hanya mencatat pasien Rumah Sehat terhadap kewajiban RME fasyankes untuk pasien lain;
   - akses pemilik akun ke data klinis anggota keluarga dewasa dari sisi rahasia kedokteran (N01, N02, N05).
2. **L-08, sebelum Tahap 4, diperluas dengan:**
   - bukti persetujuan anggota keluarga dewasa (Pasal 24) dan penyampaian informasi Pasal 21 kepada mereka;
   - akses pemilik akun ke data kesehatan anggota keluarga dewasa;
   - permintaan hak subjek atas nama anggota keluarga dewasa;
   - batas usia anak dan perlakuan saat anak menjadi dewasa;
   - satu NIK yang tercatat di beberapa akun;
   - minimisasi foto Kartu Keluarga, yang juga memuat data anggota keluarga lain yang tidak memakai layanan;
   - pihak yang wajib melakukan DPIA atas data RME (Pasal 34, Pasal 52);
   - apakah teks persetujuan perlu menyebut penerima data (N01, N04, N05).
3. **L-02 diperluas:** pendaftaran PSE untuk aplikasi RME minimalis (apakah satu pendaftaran cukup untuk dua sistem), dan perizinan berusaha/KBLI bagi PT. Arunika sebagai penyedia sistem RME. **Bagian PSE dimajukan ke Tahap 4**, karena menjadi prasyarat PT-32 (N03).
4. **Usulan L-15 (opsional, bisa digabung ke L-05):** perjanjian pemrosesan data antara PT. Arunika dan setiap fasyankes pengguna RME minimalis, serta antara PT. Arunika dan klinik rekanan untuk data konsultasi.
5. **PT-20 perlu dieskalasi.** Tim legal belum ditunjuk, padahal L-05 dan L-08 kini menahan story Must (EP-13), story Should (US-108), dan pengurusan registrasi SATUSEHAT yang lama prosesnya `[TIDAK DIKETAHUI]`.
6. **Bukan butir legal, untuk Tahap 4 dan 10:** cakupan audit keamanan NFR-SEC-09 menyebut RME minimalis secara eksplisit, dan diselaraskan dengan survei keamanan registrasi SATUSEHAT `[SRC-27]`.

## 5. Verifikasi sumber

| SRC | Hasil buka ulang hari ini | Dipakai di | Penilaian |
| --- | --- | --- | --- |
| SRC-26 | "SATUSEHAT menggunakan HL7 FHIR dalam pengimplementasian standar data model dan Application Programming Interface (API)." Versi FHIR tidak disebut. Prerequisites: Organization, Location, Practitioner, Patient | BR-123, US-1302 AC2, US-1309 | Didukung. US-1302 AC2 didukung sebatas Practitioner sebagai prasyarat, dan rinciannya sudah benar ditulis `[TIDAK DIKETAHUI]` |
| SRC-27 | "WAJIB mendaftarkan sistem RME-nya ke PSE KOMINFO untuk mendapatkan Nomor PSE."; registrasi "memerlukan nomor afiliasi SNOMED CT"; tiga proses registrasi; verifikasi; dua WAJIB tambahan di bagian berkas administrasi (N03). Halaman tidak menyatakan registrasi ke SATUSEHAT itu sendiri wajib | BR-123, US-1309 AC3, §9, §10, PT-32, catatan T3-Q15 | Didukung. Tidak ada klaim "wajib registrasi SATUSEHAT". Daftar syarat belum lengkap (N03) |
| SRC-28 | Playbook rawat jalan sebagai panduan teknis bagi fasyankes; 29 kelompok data. Halaman tidak menyebut kelompok data mana yang minimal atau wajib. Kata "WAJIB" hanya dipakai untuk elemen bertanda bintang pada pemetaan FHIR. Hanya 100.000 karakter pertama yang dibaca | UP-27, US-1306 AC1–AC2, US-308 AC4 | Didukung. UP-27 sudah benar berlabel usulan, dan kecukupan subsetnya ditanyakan di L-05. Catatan untuk Tahap 4: elemen wajib per resource ada di bagian pemetaan yang belum dibaca |
| SRC-29 | "Get the latest stable version of the ASVS (5.0.0) from the Downloads page."; "Published 30 May 2025" | NFR-SEC-01, KEPUTUSAN-05 | Didukung |
| SRC-18 (dibuka ulang karena BR-123 memperkuat klaimnya) | "Fasyankes wajib terhubung melalui platform terintegrasi dengan SATUSEHAT yang telah disediakan oleh Kementerian Kesehatan." Tidak membahas kerja sama dengan PSE atau pihak ketiga | BR-123, UP-28, US-1306, US-1309 | Didukung |
| SRC-15 (dibuka ulang) | Pasal 1 angka 4–5, 4 ayat (2), 7, 13, 18, 20, 21, 22, 24, 25, 32, 34, 51, 52. Pasal 1 tidak memuat definisi "anak" | US-104, AB-04, BR-104, AB-70, EP-16 | Lihat B01, N01, N02, N04, N05 |

Klaim lain yang diperiksa dan sesuai sumber:

| Klaim | Lokasi | Sumber yang mendukung |
| --- | --- | --- |
| Data kesehatan, data anak, dan data keuangan pribadi adalah data spesifik | AB-70 | Pasal 4 ayat (2) huruf a, e, f |
| Penghentian pemrosesan setelah persetujuan ditarik dan batas 3 x 24 jam | US-1601 AC2, US-1603 AC4 | Pasal 30 ayat (1), 32 ayat (2), 40 ayat (2), 41 ayat (1), menurut entri SRC-15 |
| Salinan dalam format yang dapat dibaca mesin | US-1604 | Pasal 13 ayat (1): "dapat dibaca oleh sistem elektronik" |
| Retensi PMSE beserta catatan bahwa status PP 80/2019 belum diverifikasi | NFR-RET-02 | `[SRC-25]` |

Bagian berikut juga sesuai dengan BR-081, BR-105, BR-107, BR-108, BR-113, dan ADR-005:
- masking dan larangan data di audit trail (§7.3);
- audit log append-only (US-1504);
- pencatatan pengiriman data ke pihak luar (E-17);
- data RME minimalis disimpan di Indonesia (NFR-PRIV-01);
- isolasi data antar-fasyankes.

**L-14 (refund).** Penandaannya benar:
- Pengantar §3.5 (PRD baris 172) menyatakan kebijakan refund menunggu L-04 dan L-14. AB-13, AB-16, dan §12.3 baris 1862 juga merujuk L-14.
- BRD L-14 (baris 339) mencatat bahwa tenggat Tahap 3 terlewat karena PT-20, dan bahwa perubahan dari legal akan masuk sebagai change request.
- RF-11/UP-08 tercakup oleh "kebijakan refund" di L-14.
- Tidak ada klaim bahwa aturan refund sudah sesuai ketentuan perlindungan konsumen.

## 6. Sumber yang dibuka di review ini (usulan pembaruan `docs/sources.md`, tidak diedit oleh reviewer)

Tidak ada SRC baru. Semua halaman yang dibuka sudah terdaftar. Usulan perluasan kolom "Mendukung klaim", dengan tambahan `compliance/tahap-03.md` di kolom "Dipakai di":

| ID | Usulan tambahan kutipan (dibuka 2026-11-17) |
| --- | --- |
| SRC-15 | Pasal 1 angka 4: Pengendali "menentukan tujuan dan melakukan kendali pemrosesan Data Pribadi"; angka 5: Prosesor "melakukan pemrosesan Data Pribadi atas nama Pengendali Data Pribadi"; Pasal 1 tidak mendefinisikan "anak".<br>Pasal 7: hak akses dan salinan "Data Pribadi tentang dirinya".<br>Pasal 18 ayat (2) huruf a: perjanjian antar-pengendali.<br>Pasal 21 ayat (1): pengendali "wajib menyampaikan Informasi mengenai" huruf a "legalitas dari pemrosesan Data Pribadi", b "tujuan", c "jenis dan relevansi", d "jangka waktu retensi dokumen", e "rincian mengenai Informasi yang dikumpulkan", f "jangka waktu pemrosesan", g "hak Subjek Data Pribadi"; ayat (2): pemberitahuan "sebelum terjadi perubahan Informasi".<br>Pasal 22 ayat (5): "batal demi hukum".<br>Pasal 24: "wajib menunjukkan bukti persetujuan yang telah diberikan oleh Subjek Data Pribadi".<br>Pasal 32 ayat (1): akses "beserta rekam jejak pemrosesan".<br>Pasal 34 ayat (2): daftar pemrosesan berisiko tinggi, antara lain huruf b "Data Pribadi yang bersifat spesifik" dan huruf c "skala besar".<br>Pasal 51 ayat (1), (3), (5) dan Pasal 52 tentang prosesor |
| SRC-18 | Kutipan persis: "Fasyankes wajib terhubung melalui platform terintegrasi dengan SATUSEHAT yang telah disediakan oleh Kementerian Kesehatan." Artikel tidak membahas penyelenggaraan RME bersama pihak ketiga atau PSE |
| SRC-27 | Tambahan: "WAJIB menyertakan kode registrasi fasyankes terbaru sesuai KMK"; "Apabila belum bermitra dengan fasyankes tertentu, maka WAJIB melampirkan surat pernyataan resmi (diketahui/ditanda tangani oleh Pimpinan)." Versi dokumentasi 7.23 |
| SRC-28 | Tambahan: halaman tidak menetapkan kelompok data minimal atau wajib; "WAJIB" hanya dipakai untuk elemen bertanda bintang pada pemetaan FHIR; tidak ditemukan pernyataan persetujuan pasien di 100.000 karakter pertama |

## 7. Status

### 7.1 Review awal

**TIDAK LULUS**, karena satu blokir: **C3-B01** (US-104 AC1 tidak lengkap terhadap Pasal 21 ayat (1) dan BR-103). Perbaikannya satu baris. Setelah AC1 diganti, cek ulang cukup memeriksa baris itu, tanpa review penuh.

Kelima butir non-blokir (C3-N01 sampai C3-N05) tidak memicu putaran ulang. Dua di antaranya sebaiknya dibawa ke user di gate Tahap 3:
- N01(b), karena akses pemilik akun ke data kesehatan anggota keluarga dewasa adalah keputusan produk yang berdampak privasi;
- N03, karena syarat registrasi SATUSEHAT memengaruhi jadwal PT-32.

### 7.2 Cek ulang (2026-11-17)

**Cakupan.** Yang diperiksa hanya baris yang berubah sesuai pemberitahuan orkestrator. Baris dibaca langsung di dokumen, tanpa shell. C3-B01 diperiksa penuh. Tindak lanjut non-blokir hanya dilihat sekilas, sesuai `aturan-kerja.md` §2.

| Baris (versi setelah perbaikan) | Perubahan | Hasil |
| --- | --- | --- |
| `docs/03-prd.md` 379 (US-104 AC1) | Layar memuat ketujuh informasi Pasal 21 ayat (1) huruf a–g ditambah kontak Rumah Sehat, dengan label `[SRC-15]`. Teks legalitas menunggu L-08; teks retensi dan jangka waktu pemrosesan menunggu PT-24; semuanya wajib terisi sebelum go-live | **Sesuai** Pasal 21 ayat (1) dan BR-103. Ketujuh huruf cocok dengan kutipan yang dibuka di review awal. Entri SRC-15 di `docs/sources.md` baris 23 kini memuat "legalitas". **C3-B01 tertutup** |
| 382 (US-104 AC4) | Pemberitahuan perubahan beserta tanggal berlakunya dikirim sebelum tanggal berlaku. Setelah tanggal itu, pasien wajib menyetujui versi baru sebelum memesan layanan | Sesuai Pasal 21 ayat (2). N04(b) ditindaklanjuti |
| 1436 (US-1604 AC1) | Salinan memuat rekam jejak pemrosesan (Pasal 32 ayat (1)), bersumber dari field subjek data §7.2 dan E-17; bentuknya ditetapkan di Tahap 4 | Sesuai Pasal 32 ayat (1). N04(a) ditindaklanjuti |
| 128 (§3.1, baris rekam medis RME) | Akses pasien ke rekam medis RME menjadi `[TIDAK DIKETAHUI]`, menunggu L-05 (sama dengan US-1604 AC3) | Inkonsistensi internal di N02 tertutup |
| 1173 (US-1301 AC1), 1251 (US-1309 AC3), 1757 (§9), 1766 (§10 BR-100), 1856 (PT-32) | Syarat SRC-27 lengkap: nomor PSE untuk sistem RME, nomor afiliasi SNOMED CT, kode registrasi fasyankes mitra atau surat pernyataan resmi, survei keamanan, dan verifikasi. US-1301 mencatat kode registrasi fasyankes. §10 menyebut pendaftaran PSE untuk aplikasi RME minimalis | Sesuai `[SRC-27]`; entri di `docs/sources.md` baris 35 kini memuat syarat kode registrasi fasyankes. N03 ditindaklanjuti |
| 302 (AB-71), 1796 (UP-15), 1591 (E-05), 642 (US-310), 1012 (US-902) | Akses pemilik akun ke data klinis anggota keluarga dewasa menjadi keputusan produk yang dibawa ke gate, dengan alasan Pasal 7 dan kebolehannya ditanyakan ke L-05 dan L-08. Akses itu kini dicatat di E-05 | Sesuai Pasal 7 ("Data Pribadi tentang dirinya"). Kata "hanya" di UP-15 adalah tafsiran yang wajar, dan tafsiran itu sudah diteruskan ke legal. N01(b) ditindaklanjuti. Keputusan akhirnya tetap di user (gate) dan tim legal |
| 1173 (US-1301 AC1), 1176 (US-1301 AC4), 1830–1831 (UP-49, UP-50) | Perjanjian dengan fasyankes menjadi prasyarat aktivasi RME, dan fasyankes dapat mengekspor rekam medisnya saat berhenti memakai RME | Keduanya berlabel UP dan dibawa ke gate, sehingga bukan pelanggaran kategori 4. Ada catatan non-blokir pada rumusan alasan UP-49, lihat di bawah tabel |
| `docs/03-prd.md` 1862, 1864, 1867, 1868 (§12.3); `docs/02-brd.md` 327 (L-02), 330 (L-05), 333 (L-08), 340 (L-15) | L-02, L-05, dan L-08 diperluas; L-15 ditambahkan. Pemajuan tenggat bagian PSE dari L-02 ke Tahap 4 dan L-15 diberi status "usulan, dimintakan persetujuan di gate" | Sesuai rekomendasi bagian 4. N02 dan N05 ditindaklanjuti lewat L-05, L-08, dan L-15. N01(a), (c), dan (d) masuk L-08 |

Catatan non-blokir sisa, yang tidak memicu putaran ulang:

1. **Rumusan alasan UP-49 (PRD baris 1830).** Teksnya menyebut "kewajiban perjanjian antar-pengendali atau pengendali–prosesor (Pasal 18, 51)".
   - Pasal 18 ayat (2) huruf a memang mensyaratkan perjanjian antar-pengendali.
   - Pasal 51 tidak menyebut perjanjian. Isinya: pemrosesan "berdasarkan perintah Pengendali Data Pribadi" (ayat (1)) dan "persetujuan tertulis" sebelum melibatkan prosesor lain (ayat (5)) `[SRC-15]`.
   - Usulan rumusan: "Pasal 18 ayat (2) huruf a (perjanjian antar-pengendali) atau Pasal 51 ayat (1) dan (5) (pemrosesan berdasarkan perintah pengendali; persetujuan tertulis untuk prosesor lain)".
   - Butir ini tidak mengubah perilaku, karena UP-49 tetap usulan produk yang diputuskan di gate.
2. **BRD BR-100 (baris 300) masih hanya menyebut "Rumah Sehat".** Dampaknya sudah ditutup oleh PRD §10 baris 1766 dan L-02 yang diperluas. Ini kosmetik.

**Hasil cek ulang: blokir 0. Status akhir: LULUS.**

**Wajib dikonfirmasi tim legal/compliance manusia:**
- L-01 sampai L-14 sesuai tenggat masing-masing;
- L-02, L-05, dan L-08 yang diperluas;
- L-15.

L-05, L-08, dan L-15 dibutuhkan sebelum Tahap 4. Bagian PSE dari L-02 diusulkan dimajukan ke Tahap 4. Pemajuan tenggat itu dan butir L-15 masih menunggu persetujuan user di gate Tahap 3, begitu juga keputusan produk akses pemilik akun ke data klinis anggota keluarga dewasa (UP-15) serta UP-49 dan UP-50. PT-20 (penunjukan tim legal) tetap perlu dieskalasi.
