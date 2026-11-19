# Review kepatuhan CR-021 (T1): situs publik dan member area layanan korporat

Tanggal: 2026-11-19
Peninjau: compliance-reviewer (subagent). Bukan pengacara; laporan ini bukan nasihat hukum.
Objek: `docs/cr/CR-021-analisis.md` v0.1, `docs/adr/ADR-027-*.md`, `ADR-028-*.md`, `ADR-029-*.md` (draf), KEPUTUSAN-250–263 (`docs/decision-log.md`), terhadap `docs/cr/CR-021-wawancara.md`, `docs/02-brd.md` (BR-040/041, BR-100–BR-118, L-01–L-16), ADR-013, `docs/03-prd.md` (§3.1, US-104, EP-16).
Kriteria blokir: `aturan-kerja.md` §2.

## Verdict: **TIDAK LULUS** (1 blokir, perbaikan kecil di dokumen)

Setelah B-1 diperbaiki, status menjadi **LULUS DENGAN CATATAN**. Analisis jujur menunda hal yang belum diketahui ke `[TIDAK DIKETAHUI]`/L-17–L-21, tidak memuat isi pasal dari ingatan, dan semua `SRC` yang dirujuk (SRC-15, SRC-50, SRC-63, SRC-246, SRC-265) terdaftar. Pekerjaan K1 Fase 1 (situs, CMS, pendaftaran/verifikasi perusahaan, akun HR) tidak memproses data peserta atau data kesehatan, sehingga temuan di bawah tidak menghambat K1. Temuan itu wajib selesai sebelum gerbang usulan G-26 (data peserta nyata pertama).

---

## 1. Sumber eksternal baru

`docs/sources.md` **belum** saya ubah. Saya hanya punya alat tulis-ulang-penuh (tanpa edit sebagian), dan menulis ulang register ±400 baris berisiko merusaknya. Mohon orkestrator menambahkan dua baris ini apa adanya:

| ID | Judul / nama sumber | URL | Tingkat | Tanggal akses | Mendukung klaim | Dipakai di |
| --- | --- | --- | --- | --- | --- | --- |
| SRC-440 | JDIH Komdigi: UU Nomor 27 Tahun 2022 tentang Pelindungan Data Pribadi (teks HTML), dibuka ulang untuk CR-021 | https://jdih.komdigi.go.id/produk_hukum/view/id/832/t/undangundang+nomor+27+tahun+2022 | 2 | 2026-11-19 | Pasal 1 angka 4: Pengendali "bertindak sendiri-sendiri atau bersama-sama dalam menentukan tujuan dan melakukan kendali pemrosesan Data Pribadi"; Pasal 1 angka 5: Prosesor "melakukan pemrosesan Data Pribadi atas nama Pengendali Data Pribadi"; Pasal 16 ayat (2) huruf a: "pengumpulan Data Pribadi dilakukan secara terbatas dan spesifik, sah secara hukum, dan transparan;"; Pasal 18 ayat (1): "Pemrosesan Data Pribadi dapat dilakukan oleh 2 (dua) atau lebih Pengendali Data Pribadi."; Pasal 20 ayat (2) huruf a–f: persetujuan eksplisit; "pemenuhan kewajiban perjanjian dalam hal Subjek Data Pribadi merupakan salah satu pihak"; "pemenuhan kewajiban hukum dan [dari] Pengendali Data Pribadi sesuai dengan ketentuan peraturan perundang-undangan;"; kepentingan vital; kepentingan umum/pelayanan publik; "pemenuhan kepentingan yang sah lainnya"; Pasal 34 ayat (2) huruf b: "pemrosesan atas Data Pribadi yang bersifat spesifik;" huruf c: "pemrosesan Data Pribadi dalam skala besar;"; Pasal 51 ayat (3): pemrosesan oleh prosesor "termasuk dalam tanggung jawab Pengendali Data Pribadi."; Pasal 52: kewajiban Pengendali Pasal 29, 31, 35–39 "berlaku juga terhadap Prosesor Data Pribadi."; Penjelasan Pasal 4 ayat (2) huruf a: data kesehatan adalah "catatan atau keterangan individu yang berkaitan dengan kesehatan fisik, kesehatan mental, dan/ atau pelayanan kesehatan." Teks halaman hasil OCR (ada salah ketik) | review/CR-021-kepatuhan.md |
| SRC-441 | JDIH Kemnaker: Peraturan Menteri Nomor PER-02/MEN/1980 tentang Pemeriksaan Kesehatan dan Keselamatan Tenaga Kerja dalam Penyelenggaraan Keselamatan Kerja (metadata dan abstrak) | https://jdih.kemnaker.go.id/peraturan/detail/2402 | 2 | 2026-11-19 | Ditetapkan 13 Maret 1980, status "Berlaku". Abstrak: "kewajiban perusahaan dalam menyelenggarakan pemeriksaan kesehatan tenaga kerja"; jenis pemeriksaan sebelum kerja, berkala "sekurang-kurangnya satu tahun sekali", dan khusus "bagi tenaga kerja tertentu"; "pelaporan hasil kepada otoritas terkait". Teks pasal tidak tampil di halaman; penerima dan penyimpan hasil tidak disebut; perubahan sesudahnya tidak diperiksa | review/CR-021-kepatuhan.md |

**Riset yang gagal (dicatat agar tidak diulang dan tidak dikutip):**
- Regulasi imunisasi/vaksinasi. Hasil pencarian menampilkan judul "Permenkes 12/2017 Penyelenggaraan Imunisasi" dan "Permenkes 3/2026 Penanggulangan Penyakit". Halaman resmi JDIH BPK (`peraturan.bpk.go.id/Details/111977`, `/Details/350854`) menjawab HTTP 403, PDF di peraturan.go.id tidak dapat diekstrak, dan halaman detail peraturan.go.id terputus (ECONNRESET). Status dan isi keduanya **belum terverifikasi dan tidak boleh dikutip**. Judul ini hanya petunjuk untuk tim legal.
- Teks pasal PER-02/MEN/1980 (penerima hasil, kerahasiaan) tidak tersedia di JDIH Kemnaker. Salinan pihak ketiga tidak dipakai.

---

## 2. Blokir

| # | Temuan | Lokasi | Kriteria | Perbaikan |
| --- | --- | --- | --- | --- |
| B-1 | Matriks akses usulan memberi tanda **"Sensitif: Tidak"** pada "Rekap kehadiran" dan "Data peserta (nama, kontak, tanggal lahir)" (kecuali NIK), tanpa sumber. Menurut PRD §3.1 baris 144, "Sensitif" berarti data pribadi spesifik, termasuk data kesehatan. Penjelasan Pasal 4 ayat (2) huruf a UU PDP mendefinisikan data kesehatan secara luas, termasuk "keterangan individu yang berkaitan dengan ... pelayanan kesehatan" `[SRC-440]`. Kehadiran atau keikutsertaan seseorang dalam MCU, vaksinasi, atau pemeriksaan lab tertentu dapat termasuk definisi itu. L-08 yang sudah lulus gate juga masih menanyakan hal serupa: apakah data transaksi berisi nama layanan klinik/lab "tergolong data kesehatan" `[DOK: 02-brd.md]`. Tanda "Tidak" berarti mendahului jawaban legal dan menghapus kewajiban E-05/KU-03 atas data itu. | `CR-021-analisis.md` §4.1 matriks, baris "Data peserta" dan "Rekap kehadiran" (±baris 136–137) | §2 butir 2 (klasifikasi regulasi tanpa SRC) dan butir 3 (bertentangan dengan pertanyaan terbuka L-08) | Ubah kolom Sensitif kedua baris menjadi "Perlu konfirmasi legal (L-08, L-18); sampai dijawab diperlakukan Ya", lalu tambahkan butir itu ke L-18. Keputusan user bahwa HR melihat rekap kehadiran `[USER] P2-4` tetap berlaku. Yang ditanyakan ke legal adalah dasar pemrosesannya, bukan keputusannya. |

---

## 3. Tabel temuan per area

Status: terpenuhi / sebagian / belum / perlu konfirmasi legal.

| # | Kebutuhan | Sumber | Status | Bukti | Rekomendasi |
| --- | --- | --- | --- | --- | --- |
| 1 | Peran PT. Arunika terhadap perusahaan klien (pengendali, pengendali bersama, prosesor) untuk data unggahan HR | Pasal 1 angka 4–5, Pasal 18 ayat (1), Pasal 51–52 `[SRC-440]`; Pasal 18 ayat (2) huruf a, Pasal 51 ayat (5) `[SRC-15]` | Perlu konfirmasi legal | L-17; §6.4; ADR-028 Konteks | Rumusan L-17 benar. Tambahkan akibat tiap pilihan agar legal bisa memutus. (a) Bila PT. Arunika prosesor: pemrosesan hanya atas perintah perusahaan, keterlibatan rekanan pelaksana butuh persetujuan tertulis perusahaan `[SRC-15]`, dan kewajiban keamanan serta perekaman ikut berlaku `[SRC-440]` Pasal 52. (b) Bila pengendali bersama: wajib perjanjian peran dan tanggung jawab `[SRC-15]`. Tambahkan juga **peran rekanan pelaksana** atas data peserta. L-15 hanya mencakup RME dan konsultasi, belum layanan korporat. |
| 2 | Dasar pemrosesan data karyawan sebelum karyawan berinteraksi | Pasal 20 ayat (2) huruf a–f `[SRC-440]` | Perlu konfirmasi legal | §6.4 butir 1; L-17 | Penyajiannya jujur. Ini **juga bertentangan dengan BR-103** ("Pasien memberikan persetujuan eksplisit ... sebelum datanya diproses"), tetapi tidak tercantum di tabel K. Tambahkan sebagai K-12. Tulis di L-17 bahwa ada enam dasar, sehingga persetujuan bukan satu-satunya pilihan. Contohnya kewajiban hukum pemberi kerja (lihat butir 4). |
| 3 | Data kesehatan sebagai data pribadi spesifik; ringkasan fit/unfit | Pasal 4 ayat (2) `[SRC-15]`; Penjelasan Pasal 4 ayat (2) huruf a `[SRC-440]` | Terpenuhi (ringkasan dan hasil), belum (rekap, B-1) | BR-145, KEPUTUSAN-258, wadah `corporate_result_summary` | Klaim "ringkasan fit/unfit = data kesehatan" didukung definisi Penjelasan. Tambahkan `[SRC-440]` di §6.4 dan KEPUTUSAN-258. |
| 4 | Persetujuan peserta untuk berbagi ringkasan ke HR | Pasal 20 ayat (2) huruf a, Pasal 22 ayat (1), Pasal 24 `[SRC-15]`; PER-02/MEN/1980 `[SRC-441]` | Sebagian, perlu konfirmasi legal | US-1913 (bawaan tidak berbagi, per pesanan, dapat ditarik, bukti per versi), KEPUTUSAN-257, UP-85, L-18 | Desain teknisnya kuat. Perlu ditambah: (a) L-18 menyebut eksplisit bahwa pemberi kerja **wajib** menyelenggarakan pemeriksaan kesehatan tenaga kerja dan ada "pelaporan hasil kepada otoritas terkait" `[SRC-441]`. Untuk MCU yang menjadi kewajiban pemberi kerja, dasar yang tepat mungkin bukan persetujuan, dan BR-143 ("hasil lengkap hanya untuk peserta") dapat berbenturan dengan kewajiban lapor. Isi pasalnya `[TIDAK DIKETAHUI]`. (b) Teks persetujuan wajib menyebut bahwa penarikan berlaku ke depan dan **tidak menarik ekspor CSV yang sudah diunduh HR** (US-1914 AC5). (c) Keabsahan persetujuan dalam relasi kerja sudah ditanyakan di L-18, dan itu tepat. |
| 5 | Minimisasi data unggahan HR; NIK hanya bila wajib | Pasal 16 ayat (2) huruf a `[SRC-440]`; ADR-013 §4, §7 | Terpenuhi (desain), perlu konfirmasi legal (daftar layanan ber-NIK) | BR-139, `requires_nik`, PT-66, ADR-028 butir 5–6, KEPUTUSAN-256 (berkas asli dan pratinjau dihapus, log tanpa isi baris, E-18 hanya jumlah) | Sudah memadai. Perlu ditambah: kolom di luar skema (misalnya jabatan atau golongan darah di berkas HR) ditolak atau dibuang saat pratinjau dan tidak pernah disimpan. Saat ini AC3 hanya menyebut "header tidak dikenali". |
| 6 | Hak subjek data peserta yang **belum atau tidak pernah** tertaut ke akun pasien | Pasal 5–13 `[SRC-15]`; BR-106 | **Belum** | EP-16 dan US-1602 hanya untuk pasien yang masuk aplikasi. Peserta yang menolak undangan atau tidak pernah membukanya tidak punya jalur akses, perbaikan, penghapusan, atau keberatan atas data yang diunggah HR. PT-71 hanya menanyakan "perlakuan data" | Tambahkan PT baru atau perluas PT-71: kanal hak subjek data bagi peserta tanpa akun, serta siapa yang menanggapi (PT. Arunika atau perusahaan, bergantung L-17). Tambahkan juga isi pemberitahuan pemrosesan di email undangan. Saat ini US-1908 AC1 hanya mengatur "tanpa data kesehatan". |
| 7 | DPIA | Pasal 34 ayat (2) huruf b–c `[SRC-440]`; BR-110 | Sebagian | ADR-028 Konsekuensi "DPIA mencakup data karyawan dan ringkasan" | MCU massal memenuhi dua pemicu sekaligus (data spesifik dan berpotensi skala besar). Jadikan DPIA bagian CR-021 sebagai syarat G-26, bukan hanya syarat go-live. |
| 8 | Audit trail append-only dan masking (ADR-005) | BR-080, BR-107; Pasal 31 `[SRC-15]` | Terpenuhi (desain) | E-05 (HR membaca ringkasan, dengan subjek), E-13, E-17, E-18 tanpa isi; subjek `corporate_participant` sebelum tertaut (ADR-028 butir 7); token undangan disimpan sebagai hash; rekening 4 digit (ADR-029 butir 3) | Bukti transfer (`stored_file_id`) hampir pasti memuat nomor rekening lengkap pembayar. Tetapkan kelas berkas dan aksesnya di ADR-029, dan pastikan isi bukti tidak pernah masuk log maupun audit. |
| 9 | Retensi dan anonimisasi (ADR-003, ADR-013 §5–6) | BR-112; SRC-25 (belum terverifikasi status) | Sebagian | §5: retensi tabel baru mengikuti PT-24; `corporate_participant` kelas M dapat dianonimkan; pratinjau berumur pendek | Data peserta tanpa undangan diterima tidak punya pemicu akhir (PT-71). Tambahkan kategori retensi "peserta korporat tidak tertaut" ke PT-24. Invoice B2B mengikuti retensi data keuangan, yang angkanya `[TIDAK DIKETAHUI]` (PT-24, L-05). |
| 10 | Vaksinasi massal dan pelayanan on-site (L-19) | SRC-18 (fasyankes wajib RME dan terhubung SATUSEHAT, berita Kemenkes); regulasi imunisasi `[TIDAK DIKETAHUI]` | Perlu konfirmasi legal | L-19, RT-46, K-05, K-10, PT-75 | Yang dapat dinyatakan dari sumber resmi hanya ini: fasyankes wajib menjalankan RME dan terhubung ke SATUSEHAT `[SRC-18]`, sehingga klinik/lab rekanan pelaksana kemungkinan mencatat layanan korporat di sistemnya. Rincian kewajibannya untuk layanan di luar gedung `[TIDAK DIKETAHUI]`. Selain itu `[TIDAK DIKETAHUI]`: penyelenggara dan izin vaksinasi, pelaksanaan di luar fasyankes, pengambilan spesimen di kantor, pengadaan dan rantai dingin vaksin, serta pencatatan imunisasi. Rumusan L-19 sudah benar. Tambahkan dua hal: (a) kebolehan PT. Arunika menjual paket vaksinasi dengan markup (analog L-13); (b) kaitan dengan PSEF/BPOM bila vaksin dianggap produk yang diedarkan lewat platform (L-03). |
| 11 | Situs dan member area sebagai PSE (L-20) | Pasal 2 ayat (5) PP 71/2019: PSE lingkup privat mencakup penyelenggara yang "memiliki portal, situs, atau aplikasi" `[SRC-13]` | Perlu konfirmasi legal | L-20, BR-100 | Rumusan L-20 benar. Dapat ditambah dasar `[SRC-13]`: situs dan portal termasuk konsep sistem elektronik PSE. Yang ditanyakan cukup apakah pendaftaran yang ada mencakupnya. Teks persetujuan HR saat mendaftar (US-1901 AC1) juga perlu disetujui legal; masukkan ke L-20. |
| 12 | Invoice B2B dan dana rekanan vs BR-041/CR-003 (K-02) | BR-041 `[USER] T2-Q4`; BR-115, L-04; SRC-20 ("BI dapat menetapkan aspek prudensial kepada Penyelenggara Penunjang yang melakukan penerusan pembayaran ... kepada Penyedia Barang dan/atau Jasa") | Perlu konfirmasi legal dan keputusan user | K-02, K-03, BR-141, ADR-029 butir 6, PT-59, L-21 | Penyajiannya jujur dan tidak diputuskan sepihak. ADR-029 juga tidak dapat diterima sebelum PT-59/L-21, dan itu benar. Tambahkan dua hal: (a) L-21 menyebut eksplisit risiko opsi (a) terhadap BR-115 dan `[SRC-20]`. Apakah menerima transfer B2B lalu meneruskan bagian rekanan termasuk aktivitas sistem pembayaran yang butuh izin BI `[TIDAK DIKETAHUI]`. (b) Opsi (b) dua tagihan membuat perusahaan melihat bagian rekanan dan bagian PT. Arunika secara terpisah, sehingga markup terbuka. Ini bertentangan dengan US-1905 AC2 dan AB-13 ("markup tidak tampil sebagai baris tersendiri") dan perlu dicatat di K-02. |
| 13 | Peserta anak | BR-104; Pasal 25 `[SRC-15]` | Belum | Unggahan memuat tanggal lahir, tetapi tidak ada aturan untuk peserta berusia di bawah batas usia anak | Tambahkan PT: apakah peserta anak (misalnya pemagang, atau keluarga karyawan pada vaksinasi) boleh diunggah, dan bagaimana persetujuan wali. Batas usia anak masih menunggu L-08. |
| 14 | Email undangan peserta lewat SES | L-16 | Sebagian | §6.8 | L-16 kini juga mencakup email karyawan pihak ketiga yang belum menjadi pengguna. Cukup dicatat di L-16. |

---

## 4. Kelengkapan dan rumusan L-17–L-21 dan PT-59–PT-75

- **L-17**: benar. Tambahkan peran rekanan pelaksana, kanal hak subjek peserta tanpa akun, kewajiban pemberitahuan kegagalan pelindungan kepada perusahaan bila PT. Arunika prosesor, dan daftar enam dasar Pasal 20 ayat (2) `[SRC-440]`.
- **L-18**: benar. Tambahkan PER-02/MEN/1980 `[SRC-441]` sebagai rujukan awal, klasifikasi rekap kehadiran (B-1), dan akibat penarikan persetujuan atas ekspor yang sudah diunduh.
- **L-19**: benar. Tambahkan markup paket vaksinasi (L-13) dan PSEF/BPOM untuk vaksin (L-03).
- **L-20**: benar. Tambahkan dasar `[SRC-13]` dan teks persetujuan HR.
- **L-21**: benar. Tambahkan BR-115/`[SRC-20]` untuk opsi (a) dan keterbukaan markup untuk opsi (b).
- **PT-59–PT-75**: lengkap untuk aspek produk. Kurang tiga butir: hak subjek data peserta tanpa akun (butir 6), peserta anak (butir 13), dan kolom di luar skema pada berkas unggahan (butir 5).

## 5. Kontradiksi K-01–K-11

Disajikan jujur. Setiap butir menyebut dokumen gate, isi yang berbenturan, opsi, dan pemutus. Tafsiran penyusun (K-05, K-10) ditandai sebagai tafsiran dan dibawa ke user lewat PT-75. Kurang dua butir: **K-12** (BR-103, persetujuan sebelum pemrosesan; butir 2) dan, bila legal menjawab rekap kehadiran tergolong data kesehatan, benturan P2-4 dengan dasar pemrosesan (B-1). Keduanya sudah tersirat di §6.4 dan L-17, tetapi belum ada di tabel K.

## 6. Non-blokir (paling penting)

1. K-12 untuk BR-103 (butir 2).
2. Kanal hak subjek data peserta tanpa akun dan pemberitahuan pemrosesan di undangan (butir 6).
3. Rujukan `[SRC-441]` dan risiko benturan BR-143 dengan kewajiban lapor hasil MCU di L-18 (butir 4).
4. K-02 opsi (b) membuka markup, bertentangan dengan AB-13 (butir 12).
5. Bukti transfer berisi rekening lengkap: kelas berkas dan larangan masuk log/audit (butir 8).

## 7. Wajib dikonfirmasi tim legal/compliance manusia

1. Peran PT. Arunika, perusahaan klien, dan rekanan pelaksana atas data peserta, beserta perjanjiannya (L-17).
2. Dasar pemrosesan data unggahan HR, dan apakah MCU berkala/khusus adalah kewajiban hukum pemberi kerja yang mengubah dasar dan hak atas hasil (L-17, L-18, `[SRC-441]`).
3. Apakah kehadiran atau keikutsertaan dalam layanan kesehatan tergolong data kesehatan (B-1, L-08).
4. Keabsahan persetujuan karyawan dalam relasi kerja, rahasia kedokteran, dan pihak berwenang menyatakan fit/unfit (L-18).
5. Regulasi vaksinasi massal dan pelayanan MCU/lab on-site, termasuk pencatatan dan pelaporan (L-19). Belum ada sumber resmi yang berhasil dibuka.
6. Cakupan PSE untuk situs dan member area (L-20).
7. Penerimaan dana invoice B2B terhadap BR-041, BR-115, dan izin BI; faktur dan pemotongan pajak (L-21, PT-73).
8. Kanal hak subjek data bagi peserta tanpa akun, dan peserta anak.
