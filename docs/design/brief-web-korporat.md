# Brief Desain: Web Korporat (Member Area HR Perusahaan) Ruang Sehat

Tanggal: 2026-11-21
Versi: 0.1 (draf untuk diteruskan ke UI/UX Designer)
Penyusun: Claude (persona UI/UX Lead + Product Manager)
Pemilik dan pemutus: user (project manager, meneruskan ke CEO dan CTO)
Sifat dokumen: mandiri. Desainer tidak perlu akses repositori. Semua fakta bersumber dari dokumen proyek yang disebut di Lampiran A. Dua kategori yang harus dibedakan:
1. **Fakta bisnis/konten yang belum ada** (angka, harga, nama rekanan, teks hukum, logo, nama domain, kontak, format NIB/NPWP) ditulis `[TIDAK DIKETAHUI]`. Desainer **tidak boleh mengarang**; pakai konten penanda (placeholder) bertanda jelas.
2. **Langkah alur yang dibutuhkan tetapi belum diatur PRD** (misalnya pemilihan paket, pengisian peserta, permintaan jadwal, pembayaran, pemantauan) sudah diisi dengan rancangan rekomendasi berlabel `[USULAN-DESAIN]`. Label ini berarti: disetujui user sebagai dasar desain, konsisten dengan aturan PRD/ADR (privasi peserta, persetujuan berbagi ringkasan, data rekanan lewat gateway, invoice B2B), dan **bila dipakai untuk build tetap harus lewat change request di `docs/backlog.md`**. Desainer boleh menyempurnakan `[USULAN-DESAIN]` selama tidak melanggar aturan PRD yang ada (§6).

## 0. Cara membaca

Rujukan singkat di akhir kalimat atau kolom "Sumber":

| Penanda | Arti |
| --- | --- |
| `[PRD US-xxxx ACn]`, `[PRD AB-xx]`, `[PRD KU-xx]`, `[PRD NFR-xx]`, `[PRD PT-xx]` | PRD v1.4 (`docs/03-prd.md`): story, acceptance criteria (AC), aturan bisnis, kriteria umum, kebutuhan non-fungsional, pertanyaan terbuka |
| `[BRD BR-xxx]` | BRD (`docs/02-brd.md`) |
| `[ADR-0xx]` | Keputusan arsitektur (ADR-027, ADR-028, ADR-029) |
| `[DG §x]`, `[DG D-K01]` | Design guideline (`docs/06-design-guideline.md`) |
| `[WF K01-n]` | Wireframe draf K01 layar n (`docs/wireframes/k01-korporat-hr.html`) |
| `[PROTO]` | Prototipe HTML korporat (`docs/prototype/korporat/`, tujuh layar) |
| `[USER C21-xx]` | Jawaban user di wawancara CR-021 |
| `[PHASE]` | Rencana fase (`docs/07-phase-plan.md` v0.4) |
| `[TURUNAN]` | Layar yang dibutuhkan alur tetapi **tidak ditulis eksplisit** di PRD. Isi minimalnya diturunkan dari AC yang ada; sisanya `[USULAN-DESAIN]` |
| `[USULAN-DESAIN]` | Rancangan alur/aturan rekomendasi penyusun (praktik umum portal B2B kesehatan) untuk langkah yang belum diatur PRD. Disetujui user sebagai dasar desain. Untuk build wajib change request. Tidak boleh bertentangan dengan PRD/ADR |
| `[TIDAK DIKETAHUI]` | Informasi belum ada, tidak boleh ditebak |

Teks di wireframe dan prototipe yang ada adalah **draf** kecuali dikutip sebagai "teks ditetapkan" di brief ini.

## 1. Ringkasan produk dan peran Web Korporat

**Ruang Sehat** adalah aplikasi kesehatan (Flutter, Android dan iOS) untuk pasien yang sulit datang ke klinik atau apotek. Rilis pertama berisi tujuh layanan: konsultasi online dokter dan psikolog, pendaftaran klinik dan lab rekanan, home visit, informasi biaya MCU dan lab, serta pembelian obat bebas, produk kecantikan, dan produk kesehatan. Ruang Sehat berperan sebagai gateway, data klinik, lab, dan produk berasal dari sistem rekanan. Operator: PT. Arunika `[CLAUDE.md; BRD]`. Antarmuka hanya Bahasa Indonesia `[PRD NFR-LOC-01]`.

**Web Korporat** (`web/korporat`) adalah *member area* web untuk **HR/admin perusahaan klien**. Fungsinya: perusahaan mendaftar dan diverifikasi, lalu HR memesan layanan korporat dari klinik/lab rekanan, yaitu **vaksinasi massal, MCU massal, dan pemeriksaan lab massal (non-MCU)** `[BRD BR-133]`. Pelaksana adalah klinik/lab rekanan, Ruang Sehat berperan sebagai perantara `[BRD BR-134]`. Lokasi dipilih per pesanan, on-site di kantor perusahaan atau di klinik/lab rekanan `[BRD BR-135]`. Pembayaran lewat **invoice B2B (transfer bank)** tanpa payment gateway `[BRD BR-141]`.

Hubungan dengan permukaan lain:

| Permukaan | Hubungan dengan Web Korporat |
| --- | --- |
| Situs publik `web/situs` (company profile, sembilan halaman, tanpa login) | Halaman "Untuk Perusahaan" punya dua jalur ke Web Korporat: "Daftarkan perusahaan" dan "Masuk member area". Situs dan Web Korporat adalah aplikasi dan host terpisah `[DG D-S01; ADR-027 b.1]`. Nama domain `[TIDAK DIKETAHUI]` |
| Backoffice Ruang Sehat | Staf Ruang Sehat (**Verifikator**) memeriksa pendaftaran, menyetujui/menolak, membuat perusahaan, dan memulai reset MFA HR `[PRD US-1902, US-1903, US-1916; USER C21-P4 4a]`. Desain backoffice di luar brief ini (wireframe C13 sudah ada) |
| Aplikasi pasien (mobile, repositori terpisah) | **Karyawan peserta memakai aplikasi pasien**, bukan web: menerima undangan, menautkan akun, memberi/menarik persetujuan berbagi ringkasan, melihat jadwal dan hasil lengkap `[PRD §1.2, EP-19 pengantar; BRD BR-132]`. Tidak ada layar web untuk peserta (non-scope "versi web untuk pasien") `[BRD §4.2]` |
| Keycloak (realm `korporat`) | Layar masuk, TOTP, aktivasi, dan atur ulang password dirender Keycloak dengan tema login terpisah (lihat §8) `[ADR-027 b.3; DG §2]` |

HR yang juga karyawan peserta memegang dua identitas yang tidak ditautkan. Hasil peserta tidak pernah tampil di Web Korporat `[ADR-027 konsekuensi; BRD BR-143]`.

## 2. Tujuan desain dan ukuran keberhasilan

Tujuan (diturunkan dari sumber):
1. HR dapat mendaftarkan perusahaan, mengunggah tiga dokumen wajib, dan memahami status verifikasinya tanpa bantuan `[PRD US-1901, AC6]`.
2. Masuk aman (password + TOTP) tanpa membocorkan apakah akun ada, terkunci, atau nonaktif `[PRD US-1904 AC1; DG §9]`.
3. HR memesan paket dengan **harga yang jelas satu angka** dan rincian pajak terpisah sebelum konfirmasi, tanpa aksi ganda `[PRD US-1905 AC1, US-1906 AC1, KU-05]`.
4. HR mengimpor daftar peserta dengan pratinjau validasi sebelum ada yang tersimpan, dengan pesan galat yang menyebut baris, field, dan kode alasan `[PRD US-1907 AC1, AC12]`.
5. UI **tidak pernah membocorkan data kesehatan individu** ke HR, termasuk lewat kanal turunan (status, penanda, agregat) `[PRD AB-84, AB-87]`.

Ukuran keberhasilan yang **bersumber**:

| Ukuran | Nilai | Sumber |
| --- | --- | --- |
| Aksesibilitas | WCAG 2.2 Level AA, kontras semua teks ≥ 4,5:1, komponen ≥ 3:1 | `[DG §7.1, §3.2]` |
| Waktu tampil halaman web | p95 ≤ 3 detik (berlaku juga untuk member area korporat) | `[PRD NFR-PERF-04]` |
| Sesi | Berakhir 30 menit tanpa aktivitas; batas mutlak 12 jam | `[PRD NFR-SEC-04; ADR-027 b.3]` |
| Peramban | Dua versi stabil terakhir Chrome, Edge, Firefox, Safari | `[PRD NFR-COMP-03]` |
| Batas klien jaringan | Tanpa respons 15 detik (unggah berkas 60 detik) menampilkan "gagal terhubung" + coba lagi | `[PRD KU-02]` |

KPI bisnis khusus Web Korporat (jumlah perusahaan, waktu verifikasi, konversi pendaftaran): `[TIDAK DIKETAHUI]`, tidak ada di sumber. Lama pemeriksaan verifikasi juga `[TIDAK DIKETAHUI]` `[PROTO 05]`, sehingga UI tidak menjanjikan waktu (KR-09).

## 3. Pengguna, peran, dan hak akses

| Pengguna | Memakai | Catatan |
| --- | --- | --- |
| **Calon HR / HR pendaftar (HR pertama)** | Web Korporat | Mendaftarkan perusahaan sendiri, atau menerima undangan aktivasi bila perusahaan dibuatkan Verifikator `[PRD US-1901, US-1903 AC1]` |
| **HR (anggota)** | Web Korporat | Diundang HR lain. **Semua HR berperan sama** sampai PT-68 dijawab `[PRD US-1904 AC2]`. Satu akun HR hanya punya satu perusahaan `[ADR-028 b.1]` |
| **Verifikator** (staf Ruang Sehat) | Backoffice | Bukan pengguna Web Korporat. Disebut karena keputusannya tampil di layar HR |
| **Peserta** (karyawan) | Aplikasi pasien | Bukan pengguna Web Korporat |
| Keuangan (staf Ruang Sehat) | Backoffice | Mencatat transfer dan pelunasan. Hasilnya tampil sebagai status invoice di Web Korporat `[PRD US-1910]` |

**Status perusahaan menentukan menu dan layar HR** `[PRD US-1901 AC6, US-1905 AC4, US-1916 AC2]`:

| Status perusahaan | Yang dilihat HR | Tidak tersedia |
| --- | --- | --- |
| **Menunggu verifikasi** | Status verifikasi, profil perusahaan (baca-saja) | Katalog, pesanan, peserta, invoice |
| **Ditolak** | Status, alasan penolakan, profil; tombol memperbaiki dan mengajukan ulang | Idem |
| **Aktif** | Dasbor, profil, kelola HR; layanan korporat sesuai fase (§4) | Data perusahaan lain (selamanya) |
| **Nonaktif** (oleh Verifikator) | Semua akun HR perusahaan ditolak seketika; sesi terbuka berakhir pada permintaan berikutnya, layar "Akun nonaktif" | Semuanya. Nasib pesanan berjalan `[TIDAK DIKETAHUI]` (PT-63) `[PRD US-1916 AC2, KU-10]` |

Akun HR nonaktif: saat masuk, ditolak dengan pesan seragam login gagal tanpa tanda tambahan; bila sesi terbuka, layar "Akun nonaktif" dengan kontak Layanan Pelanggan `[PRD KU-10]`. Kontak konkret Layanan Pelanggan (nomor/email/jam) `[TIDAK DIKETAHUI]`.

## 4. Ruang lingkup per fase

Skenario S3 `[USER C21-P4]`, jadwal di `[PHASE]`:

| Fase | Isi Web Korporat | Status desain |
| --- | --- | --- |
| **Fase 1** (selesai 2026-12-11 menurut rencana) | Pendaftaran bertahap + unggah dokumen, periksa email, aktivasi dan TOTP, masuk/keluar, menunggu verifikasi, ditolak + ajukan ulang, profil, dasbor, kelola HR, akses ditolak/404, akun nonaktif, galat, peringatan idle `[PHASE F1-T85–F1-T110]` | **Didesain dulu, wajib.** Wireframe K01 dan prototipe sudah ada sebagai titik awal |
| **Fase 2** | Pemulihan MFA HR: masuk dengan kode pemulihan lalu daftar ulang TOTP; reset oleh Verifikator `[PRD US-1904 AC7–AC9; PHASE K1-F2]`. Email pemberitahuan keputusan verifikasi `[PRD US-1902 AC3, US-1001]` | Peta, desain setelah Fase 1 |
| **Fase 3** | Katalog, pemesanan, invoice, impor peserta, undangan peserta, penjadwalan, rekap kehadiran, ringkasan hasil dengan persetujuan (K2 dan K3) `[PHASE §0 v0.3, §8.2]` | Peta lengkap di bawah, dirancang end-to-end. Aturan yang belum diputuskan bisnis (PT-59–PT-79) diisi `[USULAN-DESAIN]`, dan angka/teks hukum/harga tetap `[TIDAK DIKETAHUI]` |

Prioritas semua story korporat: Must `[PRD §5 EP-19]`. Posisi di urutan pengurangan scope `[TIDAK DIKETAHUI]` (PT-69).

### 4.1 Peta alur (journey)

| Kode | Alur | Fase |
| --- | --- | --- |
| J1 | Mendaftarkan perusahaan, verifikasi email, aktivasi TOTP | 1 |
| J2 | Masuk, keluar, sesi, lupa password | 1 |
| J3 | Menunggu verifikasi, ditolak, ajukan ulang | 1 |
| J4 | Perusahaan dibuatkan Verifikator: aktivasi HR pertama | 1 |
| J5 | Dasbor, profil, kelola HR | 1 |
| J6 | Keadaan lintas: akses ditolak, akun nonaktif, galat, offline | 1 |
| J7 | Pemulihan MFA HR | 2 |
| J8 | Memilih layanan dari katalog | 3 |
| J9 | Memesan: lokasi, jumlah peserta, ringkasan, konfirmasi | 3 |
| J10 | Invoice dan pembayaran | 3 |
| J11 | Mengisi dan mengimpor peserta | 3 |
| J12 | Undangan peserta dan status | 3 |
| J13 | Jadwal pelaksanaan | 3 |
| J14 | Memantau pesanan, kehadiran, ekspor | 3 |
| J15 | Ringkasan hasil (dengan persetujuan) | 3 |
| J16 | Pembatalan/perubahan pesanan | 3, `[USULAN-DESAIN]` |

Urutan end-to-end layanan: **J1 → J3 (verifikasi) → J8 → J9 → J10 (lunas) → J11/J12 → J13 → (layanan berlangsung di rekanan) → J14 → J15**. Aturan "layanan hanya dapat dijadwalkan setelah invoice lunas" `[PRD AB-83]` membuat J13 bergantung pada J10.

## 5. Daftar layar dan alur lengkap

Notasi per layar: **Tujuan · Data · Aksi · Keadaan · Teks ditetapkan / menunggu · Sumber**. "Keadaan" memuat kosong (K), memuat (M), galat (G), sukses (S) bila relevan. Pola umum keadaan: validasi per field, isian benar tidak hilang, fokus ke galat pertama `[PRD KU-01; DG §5.3]`; tombol kirim terkunci saat memuat `[PRD KU-05; DG §5.1]`; kerangka (skeleton) untuk muatan > 300 ms `[DG §5.1]`; gagal jaringan "Gagal terhubung" + "Coba lagi" `[PRD KU-02]`.

### Tabel ringkas

| ID | Layar | Fase | Status spesifikasi | Dirender oleh |
| --- | --- | --- | --- | --- |
| KR-01 | Daftar, langkah 1: data perusahaan | 1 | Ada | Web Korporat |
| KR-02 | Daftar, langkah 2: data pendaftar (HR) | 1 | Ada | Web Korporat |
| KR-03 | Daftar, langkah 3: dokumen dan persetujuan, kirim | 1 | Ada | Web Korporat |
| KR-04 | Periksa email Anda | 1 | Ada | Web Korporat |
| KR-05 | Verifikasi email, aktivasi undangan HR (password + TOTP) | 1 | Ada | **Keycloak** |
| KR-06 | Pendaftaran TOTP + 12 kode pemulihan | 1 | Ada | **Keycloak** |
| KR-07 | Masuk (password, lalu TOTP / kode pemulihan) + lupa password | 1 | Ada; lupa password `[USULAN-DESAIN]` | **Keycloak** |
| KR-08 | Dialog sesi: peringatan idle, batas 12 jam, sesi berakhir | 1 | Ada | Web Korporat |
| KR-09 | Status verifikasi: Menunggu verifikasi | 1 | Ada | Web Korporat |
| KR-10 | Status verifikasi: Ditolak + ajukan ulang | 1 | Ada | Web Korporat |
| KR-11 | Profil perusahaan | 1 | Ada (baca-saja); catatan perubahan data `[USULAN-DESAIN]` | Web Korporat |
| KR-12 | Dasbor | 1 (isi lanjutan 3) | Ada untuk Fase 1; Fase 3 `[USULAN-DESAIN]` | Web Korporat |
| KR-13 | Kelola HR: daftar, undang, nonaktifkan | 1 | Ada | Web Korporat |
| KR-14 | Akses ditolak / data tidak ditemukan | 1 | Ada | Web Korporat |
| KR-15 | Akun nonaktif | 1 | Ada | Web Korporat |
| KR-16 | Galat umum dengan kode referensi, gagal terhubung, data sementara tidak tersedia | 1 | Ada | Web Korporat |
| KR-17 | Masuk dengan kode pemulihan, lalu daftar ulang TOTP | 2 | Ada (AC7) | **Keycloak** |
| KR-18 | Reset MFA oleh Verifikator: tautan sekali pakai, password, TOTP baru | 2 | Ada (AC8) | **Keycloak** |
| KR-19 | Konfirmasi permintaan reset MFA oleh HR aktif lain | 2 | `[USULAN-DESAIN]` | Web Korporat |
| KR-20 | Katalog paket | 3 | Ada | Web Korporat |
| KR-21 | Detail paket | 3 | `[TURUNAN]` | Web Korporat |
| KR-22 | Pesan: lokasi, jumlah peserta | 3 | Ada sebagian + `[USULAN-DESAIN]` | Web Korporat |
| KR-23 | Ringkasan pesanan dan konfirmasi | 3 | Ada | Web Korporat |
| KR-24 | Pesanan dibuat, invoice terbit | 3 | Ada | Web Korporat |
| KR-25 | Daftar pesanan | 3 | `[TURUNAN]` | Web Korporat |
| KR-26 | Detail pesanan | 3 | `[TURUNAN]` | Web Korporat |
| KR-27 | Daftar invoice | 3 | `[TURUNAN]` | Web Korporat |
| KR-28 | Detail invoice dan unduh | 3 | Ada sebagian | Web Korporat |
| KR-29 | Pembayaran: cara membayar dan status pembayaran dari sisi HR | 3 | `[USULAN-DESAIN]` | Web Korporat |
| KR-30 | Peserta: unggah berkas (CSV/XLSX) | 3 | Ada | Web Korporat |
| KR-31 | Peserta: pratinjau validasi per baris | 3 | Ada | Web Korporat |
| KR-32 | Peserta: hasil impor | 3 | Ada sebagian | Web Korporat |
| KR-33 | Peserta: tambah/ubah/hapus manual satu per satu | 3 | `[USULAN-DESAIN]` | Web Korporat |
| KR-34 | Daftar peserta pesanan dan status undangan | 3 | Ada sebagian | Web Korporat |
| KR-35 | Detail peserta (data unggahan, status undangan) | 3 | Ada sebagian | Web Korporat |
| KR-36 | Kirim ulang undangan | 3 | Ada sebagian | Web Korporat |
| KR-37 | Jadwal pelaksanaan | 3 | Tampil: ada. Preferensi jadwal oleh HR: `[USULAN-DESAIN]` | Web Korporat |
| KR-38 | Rekap kehadiran dan ekspor CSV | 3 | Ada | Web Korporat |
| KR-39 | Ringkasan hasil peserta (bersyarat persetujuan) | 3 | Ada | Web Korporat |
| KR-40 | Pembatalan atau perubahan pesanan | 3 | `[USULAN-DESAIN]` | Web Korporat |

Total: **40 layar** = Fase 1: KR-01–KR-16 (16); Fase 2: KR-17–KR-19 (3, dua di antaranya halaman Keycloak); Fase 3: KR-20–KR-40 (21). Keadaan lintas (kosong, memuat, galat, dialog konfirmasi) tidak dihitung sebagai layar tersendiri.

---

### J1 Mendaftarkan perusahaan (Fase 1)

Pola: tiga langkah, penanda langkah ("1 Data perusahaan / 2 Data Anda / 3 Dokumen dan persetujuan"), tombol Kembali mempertahankan isian. Isian hanya di memori halaman, **kemajuan tidak disimpan di perangkat**; password dan berkas tidak pernah disimpan di klien, bila halaman dimuat ulang pendaftar mengisi dari awal `[DG D-K01; WF K01-2]`.

**KR-01 Langkah 1: data perusahaan** `[PRD US-1901 AC1, AC3]`
- Data: nama badan usaha, NIB, NPWP, alamat kantor, telepon kantor. Wajib semua (dokumen wajib AC2; "wajib" per field selain yang disebut AC1 mengikuti KU-01).
- Validasi: format NIB dan NPWP salah ditolak per field, **format resmi `[TIDAK DIKETAHUI]`**, diverifikasi ke sumber resmi sebelum task dibangun. Rancang field dengan konten penanda dan jangan menulis aturan panjang/digit.
- Aksi: Lanjut. Tautan "Sudah punya akun? Masuk".
- Keadaan: G per field. Contoh teks galat di wireframe (draf): "Format NIB tidak sesuai. Periksa kembali nomor di dokumen NIB Anda." / "NPWP wajib diisi." `[WF K01-2]`.

**KR-02 Langkah 2: data pendaftar** `[PRD US-1901 AC1]`
- Data: nama, jabatan, email (kerja), password. Hint di wireframe: tautan verifikasi dikirim ke email; password boleh ditempel `[WF K01-3]`.
- Password mengikuti kebijakan: 8–128 karakter, semua karakter boleh, tanpa aturan komposisi, ditolak bila umum/bocor/kata konteks/sama dengan email, paste dan pengelola password diizinkan, isian bermasker dengan opsi menampilkan `[PRD NFR-SEC-03, US-101 AC1a]`. Pesan galat menyebut aturan yang dilanggar (panjang, kata sandi umum atau bocor, kata konteks, sama dengan email) `[PRD US-101 AC1a]`.
- Aksi: Kembali, Lanjut.

**KR-03 Langkah 3: dokumen dan persetujuan** `[PRD US-1901 AC1, AC2, AC5, AC7; NFR-SEC-08]`
- Data: tiga unggahan terpisah wajib, **Dokumen NIB, Dokumen NPWP, Bukti kewenangan pendaftar**. Bukti kewenangan = surat kuasa atau surat penunjukan dari perusahaan, ditandatangani pejabat berwenang `[USER C21-P4 putaran 5, 4b]`. Siapa yang dianggap pejabat berwenang `[TIDAK DIKETAHUI]`.
- Batas berkas: foto JPG/PNG ≤ 5 MiB; PDF ≤ 10 MiB. Pesan ukuran menyebut batas per jenis berkas `[PRD NFR-SEC-08; DG D-K01]`.
- Persetujuan: satu kotak centang untuk teks pemrosesan data pendaftaran perusahaan, **tidak tercentang otomatis**, versi teks tampil `[PRD US-1901 AC1; DG §4]`. **Teks persetujuan HR dan versinya `[TIDAK DIKETAHUI]`, menunggu legal (L-20)**. Tampilkan placeholder bertanda draf, jangan menulis teks hukum.
- Aksi: Kembali, Kirim pendaftaran (terkunci + "Mengirim..." saat memuat).
- Keadaan:
  - G per field: dokumen wajib kosong; jenis/ukuran di luar batas `[PRD AC2]`.
  - G NIB sudah terdaftar: pesan **generik** tanpa nama perusahaan lain, dengan arahan menghubungi Layanan Pelanggan `[PRD US-1901 AC7; USER C21-P4 4c]`. Draf teks: "Pendaftaran tidak dapat diproses. Data yang Anda kirim tidak dapat digunakan untuk mendaftar. Periksa kembali data Anda, atau hubungi Layanan Pelanggan untuk bantuan." `[WF K01-4]`.
  - G pengiriman gagal di tengah proses: "Pendaftaran belum terkirim", dinyatakan belum ada akun dibuat, isian masih ada, tombol "Coba lagi" `[PRD AC5; DG D-K01]`.
  - PRD menyebut NIB yang sudah dipakai "perusahaan terdaftar" `[PRD US-1901 AC7]`. `[USULAN-DESAIN]` UI memperlakukan semua status (Menunggu verifikasi, Ditolak, Aktif, Nonaktif) sama: satu pesan generik, tidak pernah membedakan status perusahaan lain.

**KR-04 Periksa email Anda** `[PRD US-1901 AC4]`
- Layar **identik** untuk email baru dan email yang sudah terdaftar. Beda hanya di kotak masuk. Teks (draf): "Kami mengirim petunjuk ke email yang Anda daftarkan. Buka tautannya untuk melanjutkan. Bila email tidak masuk dalam beberapa menit, periksa folder spam atau hubungi Layanan Pelanggan." `[WF K01-5]`
- Aksi: Kembali ke situs. `[USULAN-DESAIN]` tombol sekunder "Kirim ulang petunjuk" dengan pesan hasil yang sama untuk semua kasus dan jeda sebelum bisa ditekan lagi (nilai jeda ditetapkan saat implementasi).
- Jangan menampilkan email yang diketik dengan cara yang membedakan kasus.

**KR-05 Verifikasi email dan aktivasi (Keycloak)** `[PRD US-1901 AC1, US-1903 AC1, US-1904 AC2]`
- Verifikasi email wajib. Untuk HR yang diundang (oleh Verifikator atau HR lain): undangan aktivasi, mengatur password dan TOTP. Tautan kedaluwarsa atau sudah dipakai ditolak `[PRD US-1903 AC2]`. Layar ini dibuat dengan tema Keycloak (§8).
- `[USULAN-DESAIN]` Tiga keadaan halaman tautan: valid (formulir password lalu lanjut ke KR-06), kedaluwarsa/sudah dipakai (satu pesan netral: "Tautan tidak berlaku. Minta undangan baru kepada HR yang mengundang Anda atau hubungi Layanan Pelanggan", tanpa menyebut nama perusahaan), dan berhasil (lanjut ke KR-06). Masa berlaku tampil sebagai tanggal/jam dari server, angkanya `[TIDAK DIKETAHUI]` (tidak ditulis desainer).

**KR-06 Pendaftaran TOTP dan kode pemulihan (Keycloak)** `[PRD US-1904 AC1; NFR-SEC-02]`
- Wajib saat login pertama. Isi: instruksi memasang aplikasi authenticator, kode QR **dan** kunci teks yang dapat disalin, isian kode 6 digit, tombol Aktifkan; lalu **12 kode pemulihan, tampil sekali**, kotak centang wajib "Saya sudah menyimpan kode pemulihan" sebelum Lanjut `[DG §9, D-K01, WF K01-6]`. QR punya teks alternatif. Menutup layar tidak membawa HR ke dasbor. Kode pemulihan tidak masuk log.
- Peringatan (draf): "Setiap kode hanya berlaku sekali... tanpa kode ini dan perangkat TOTP, akun hanya dapat dipulihkan lewat Verifikator." `[WF K01-6]`
- Catatan: di Fase 1 pemulihan lewat Verifikator belum tersedia (Fase 2). Sampai itu HR yang kehilangan TOTP dan kode pemulihan **tidak dapat dipulihkan** `[ADR-027 konsekuensi]`. Teks peringatan harus jujur soal ini, tanpa janji jalur darurat. `[USULAN-DESAIN]` Teks: "Simpan kode pemulihan di tempat yang aman. Bila perangkat authenticator dan semua kode pemulihan hilang, akun tidak dapat dipulihkan sendiri." Desainer boleh menyempurnakan kalimatnya.

### J2 Masuk, keluar, sesi (Fase 1)

**KR-07 Masuk (Keycloak)** `[PRD US-1904 AC1; US-103 AC2; DG §9]`
- Langkah: email + password, lalu kode TOTP 6 digit (boleh ditempel); tautan "Pakai kode pemulihan"; tautan "Lupa password"; "Belum terdaftar? Daftarkan perusahaan".
- **Pesan galat seragam** untuk password salah, akun tidak ada, terkunci, nonaktif, dan kode salah. Tanpa pesan "akun terkunci". Teks draf: "Tidak dapat masuk. Email, password, atau kode tidak sesuai. Bila tidak dapat masuk, gunakan Lupa password atau hubungi Layanan Pelanggan." Frasa petunjuk "Bila tidak dapat masuk, gunakan Lupa password atau hubungi Layanan Pelanggan" ditetapkan `[DG §9]`.
- Kunci: 5 percobaan gagal berturut-turut mengunci 15 menit `[PRD NFR-SEC-03 / US-103 AC3]`. **Jangan menampilkan** lama kunci atau status terkunci.
- Lupa password: atur ulang password tidak melewati MFA (tetap meminta TOTP) `[PRD NFR-SEC-02; ADR-027 b.3]`. Story HR tidak punya AC khusus (hanya task F1-T88), maka `[USULAN-DESAIN]` mengikuti pola pasien: (1) formulir email; (2) pesan **seragam** "Bila email terdaftar, petunjuk atur ulang password dikirim ke email itu" untuk semua kasus; (3) tautan di email membuka formulir password baru (aturan password §KR-02); (4) setelah selesai, login tetap meminta TOTP; (5) email pemberitahuan "password telah diatur ulang" beserta kontak Layanan Pelanggan `[PRD US-103 AC4b]`.
- Petunjuk kehilangan TOTP (draf): "Hubungi Layanan Pelanggan. Reset oleh Verifikator memerlukan konfirmasi lewat kanal lain (Fase 2)." `[WF K01-1]`
- Keluar: tombol "Keluar" di kepala halaman.

**KR-08 Dialog sesi** `[PRD US-1904 AC6; DG §5.5 KEPUTUSAN-170]`
- Peringatan 2 menit sebelum 30 menit tidak aktif: dialog "Sesi akan berakhir" dengan **Tetap masuk** dan **Keluar** (teks draf `[WF K01-11]`). Dialog menerima fokus, fokus kembali ke pemicu saat ditutup.
- Batas mutlak 12 jam: peringatan tanpa tombol perpanjangan `[DG §5.5; ADR-027 b.3]`.
- Setelah berakhir: diarahkan ke KR-07 tanpa isian terbawa `[WF K01-11]`.

### J3 Verifikasi perusahaan (Fase 1)

**KR-09 Menunggu verifikasi** `[PRD US-1901 AC6, US-1902 AC1]`
- Data: badge "Menunggu verifikasi" (peringatan, teks + simbol), tanggal/jam pengajuan, linimasa (Pendaftaran terkirim, Pemeriksaan dokumen oleh Verifikator, Keputusan), profil baca-saja dengan **NIB dan NPWP termasking** `[DG D-K01; WF K01-7]`.
- Menu hanya: Status verifikasi, Profil perusahaan.
- Email pemberitahuan keputusan **belum ada di Fase 1**, menyusul Fase 2; teks wireframe menyuruh HR memeriksa halaman berkala `[PRD US-1902 AC3; WF K01-7]`. Lama pemeriksaan `[TIDAK DIKETAHUI]`, maka `[USULAN-DESAIN]` UI tidak menampilkan perkiraan waktu atau janji tenggat. Setelah email pemberitahuan ada (Fase 2), tambahkan kalimat bahwa keputusan juga dikirim lewat email.

**KR-10 Ditolak dan ajukan ulang** `[PRD US-1902 AC3, US-1901 AC6]`
- Data: badge/alert "Ditolak" (bahaya), **alasan** dari kode alasan Verifikator, termasuk "bukti kewenangan tidak memadai". Daftar kode alasan final `[TIDAK DIKETAHUI]`: rancang slot "Alasan" yang menampilkan label teks dari kode, plus slot "Keterangan Verifikator" yang tampil hanya bila ada (teks tampil sebagai teks biasa, tidak sebagai HTML) `[USULAN-DESAIN]`.
- Aksi: "Perbaiki dan ajukan ulang" membuka formulir KR-01–KR-03 dengan data sebelumnya terisi, dokumen yang diganti diunggah ulang `[DG D-K01; WF K01-8]`. `[USULAN-DESAIN]` Dokumen lama tidak ditampilkan ulang (hanya tanda "perlu diunggah ulang" per dokumen yang ditolak/diganti); pengajuan ulang tidak punya penghitung percobaan di UI; setelah diajukan ulang HR kembali ke KR-09.

### J4 Perusahaan dibuatkan Verifikator (Fase 1)

Tidak ada layar daftar. HR menerima undangan aktivasi email (password + TOTP) lewat KR-05/KR-06, perusahaan sudah **Aktif** `[PRD US-1903 AC1]`. Layar pertama setelah masuk: KR-12. Persetujuan orang kedua di sisi Ruang Sehat (PT-67 sisa) adalah urusan backoffice dan tidak memengaruhi layar HR `[PRD US-1903 AC1]`. `[USULAN-DESAIN]` Pada login pertama, dasbor menampilkan alert info "Akun perusahaan Anda sudah aktif" sekali, dengan tautan ke Kelola HR.

### J5 Dasbor, profil, kelola HR (Fase 1)

**KR-11 Profil perusahaan** `[PRD §3.1 baris data perusahaan; US-1901 AC6]`
- Baca-saja: nama badan usaha, NIB, NPWP (termasking), alamat, telepon kantor. HR melihat "perusahaannya (baca)" `[PRD §3.1 CR-021]`.
- `[USULAN-DESAIN]` Profil seluruhnya **baca-saja** di semua status (meminimalkan risiko perubahan identitas badan usaha setelah diverifikasi). Di bawah profil tampil catatan "Untuk mengubah data perusahaan, hubungi Layanan Pelanggan" dan daftar dokumen (NIB, NPWP, bukti kewenangan) dengan status "Terunggah" **tanpa tautan unduh** (hanya Verifikator yang membuka dokumen, lewat URL berumur pendek `[PRD US-1902 AC1]`).

**KR-12 Dasbor** `[WF K01-9; PROTO 06]`
- Fase 1: alert "Perusahaan terverifikasi" (sukses), status perusahaan (Aktif), jumlah HR aktif + tautan Kelola HR, kartu "Pesanan berjalan" bertanda belum tersedia. Menu "Layanan korporat" (Pesanan, Peserta, Invoice) tampil **nonaktif dengan alasan tertulis "belum tersedia"** (bukan dihilangkan) `[DG D-K01]`.
- Fase 3 `[USULAN-DESAIN]`: kartu "Perlu tindakan Anda" (pesanan Menunggu pembayaran; invoice Lewat jatuh tempo; pesanan Lunas yang menunggu jadwal; peserta berstatus undangan "Gagal kirim"), kartu "Pesanan berjalan" (daftar singkat dengan status, tautan ke KR-26), kartu "Jadwal terdekat" (tanggal, jam, lokasi), pintasan "Pesan layanan" ke KR-20. Angka kehadiran per pesanan boleh tampil bila bersumber dari status kehadiran saja. **Batas keras:** tidak boleh ada nilai hasil, ringkasan, jumlah/persentase/statistik/grafik yang berasal dari hasil peserta `[PRD US-1914 AC7, AB-87, BR-143]`.

**KR-13 Kelola HR** `[PRD US-1904 AC2, AC3, AC5; WF K01-10]`
- Tabel anggota HR perusahaan itu saja: nama, email, status (Aktif, Menunggu aktivasi, Nonaktif), aksi. Judul tabel/caption: semua HR berperan sama.
- Undang HR: isian email, tombol "Kirim undangan". Pesan sukses **sama** bila email sudah terdaftar di tempat lain (anti-enumerasi) `[WF K01-10 draf; pola PRD US-1901 AC4]`.
- Nonaktifkan: dialog dampak (HR tidak dapat masuk lagi, sesi terbuka diakhiri pada permintaan berikutnya) dengan tombol destruktif "Nonaktifkan" `[DG §4 Dialog]`. **HR aktif terakhir tidak dapat dinonaktifkan** (pesan galat + aksi tidak tersedia dengan alasan tertulis).
- `[USULAN-DESAIN]` Pelengkap: (a) alasan nonaktif tidak diminta; (b) tidak ada tombol Nonaktifkan untuk baris diri sendiri, diganti keterangan "Anda tidak dapat menonaktifkan akun sendiri; minta HR lain"; (c) baris "Menunggu aktivasi" punya aksi "Kirim ulang undangan" dan "Batalkan undangan" (tautan lama menjadi tidak berlaku); (d) tanggal/jam berlaku undangan tampil dari server bila ada, angkanya tidak ditulis desainer; (e) email yang sudah menjadi HR perusahaan lain dijawab dengan pesan sukses yang sama (anti-enumerasi, pola KR-04).
- K: tabel selalu berisi minimal HR yang sedang login.

### J6 Keadaan lintas (Fase 1)

**KR-14 Akses ditolak / data tidak ditemukan.** Objek perusahaan lain (anggota, pesanan, peserta, impor, invoice, rekap), termasuk dengan mengganti ID di URL, dijawab **sama persis dengan "tidak ada"**: satu pesan "Data tidak ditemukan", tidak membedakan "tidak ada" dan "tidak berhak" `[PRD US-1904 AC3, KU-03; DG §5.3]`. Juga dipakai untuk akses HR ke fitur yang belum terbuka sesuai status perusahaan (endpoint ditolak `[PRD US-1905 AC4]`). **Jangan** membuat layar "403 Anda tidak berhak" yang berbeda.

**KR-15 Akun nonaktif** `[PRD KU-10(b); US-1904 AC5; DG §5.3]`: layar penuh "Akun nonaktif" dengan kontak Layanan Pelanggan (kontak konkret `[TIDAK DIKETAHUI]`). Tampil bila sesi masih terbuka saat akun atau perusahaan dinonaktifkan. Tanpa penjelasan sebab. Saat masuk baru: pesan seragam KR-07.

**KR-16 Galat.** (a) Galat server: pesan umum + **kode referensi** (nilai `correlation_id` dari respons galat) untuk dilaporkan ke Layanan Pelanggan `[DG §5.3 KEPUTUSAN-172; ADR-023]`. (b) Gagal jaringan: "Gagal terhubung" + "Coba lagi" (kunci idempotensi sama). (c) Rekanan/pihak ketiga tidak tersedia: "Data sementara tidak tersedia" + waktu pembaruan terakhir bila ada, tombol pesan tidak tersedia `[PRD KU-08]`. (d) Tidak menampilkan jejak teknis. Halaman 404 URL tak dikenal: gunakan pola KR-14.

### J7 Pemulihan MFA HR (Fase 2)

**KR-17 Masuk dengan kode pemulihan lalu daftar ulang TOTP (Keycloak)** `[PRD US-1904 AC7]`: HR yang kehilangan perangkat TOTP tetapi memegang kode pemulihan masuk dengan kode itu, lalu **diarahkan mendaftarkan TOTP baru setelah autentikasi ulang**. Di Fase 1 kode pemulihan sudah dapat dipakai masuk, daftar ulang TOTP baru Fase 2. Beri tahu lebih dulu bahwa login akan diminta lagi `[DG §9 Autentikasi ulang]`.

**KR-18 Reset MFA oleh Verifikator (Keycloak)** `[PRD US-1904 AC8, AC9; ADR-027 b.3]`: HR yang kehilangan TOTP **dan** kode pemulihan menghubungi Layanan Pelanggan. Reset hanya dapat dimulai Verifikator setelah konfirmasi lewat kanal berbeda. Reset berlaku setelah HR membuka tautan sekali pakai di email akun dan memasukkan password. TOTP lama tetap berlaku sampai TOTP baru dan kode pemulihan baru tersimpan. Semua sesi HR dicabut dan semua HR aktif perusahaan diberi tahu. Desainer merancang layar tautan sekali pakai (valid, kedaluwarsa/sudah dipakai, ditolak) dan email pemberitahuan ke HR lain. Super Admin tidak punya jalur reset. Tidak ada admin yang melihat atau menetapkan kredensial HR.

**KR-19 Konfirmasi permintaan reset MFA oleh HR aktif lain** `[USULAN-DESAIN]`. PRD mensyaratkan konfirmasi lewat kanal berbeda: "HR aktif lain di perusahaan yang sama, atau kontak resmi perusahaan yang terdaftar saat verifikasi" `[PRD US-1904 AC8]`, tanpa merinci bentuknya. Rekomendasi: (1) bila ada HR aktif lain, ia menerima email "Ada permintaan reset MFA untuk rekan Anda" dan, setelah masuk, melihat banner di dasbor yang membuka layar konfirmasi berisi nama rekan, waktu permintaan, tombol "Ya, permintaan ini benar" dan "Bukan, tolak"; (2) tidak ada informasi sensitif di email maupun layar selain nama dan waktu; (3) bila HR yang kehilangan TOTP adalah satu-satunya HR aktif, konfirmasi dilakukan Verifikator ke kontak resmi di luar sistem (telepon kantor dari KR-01 berfungsi sebagai kontak resmi) dan tidak ada layar web untuk itu. Pemberitahuan ke semua HR aktif setelah reset berlaku `[PRD US-1904 AC8]` berupa email dan banner informasi. Backoffice mengatur sisi Verifikator (di luar brief).

### J8 Memilih layanan dari katalog (Fase 3)

**KR-20 Katalog paket** `[PRD US-1905 AC1–AC5; AB-80]`
- Hanya untuk perusahaan **Aktif**. Paket tampil **per jenis** (vaksinasi massal, MCU massal, lab massal) dan **rekanan pelaksana**, dengan: isi paket, lokasi yang tersedia (on-site, di rekanan, atau keduanya), dan **harga sebagai satu angka** (margin tidak tampil sebagai baris tersendiri `[PRD AB-13, US-1905 AC2]`, kecuali pengecualian bila legal memilih dua tagihan, L-21).
- **Satuan harga (per peserta atau per paket) dan jumlah peserta minimum belum ditetapkan bisnis** `[TIDAK DIKETAHUI]` (PT-62). `[USULAN-DESAIN]` Rancang slot "Harga" + "Satuan" (teks satuan dari data server) dan baris "Minimal peserta" yang tampil hanya bila datanya ada. Desainer memakai harga contoh bertanda "Rp[harga]" atau angka penanda yang jelas bukan data nyata, tidak mengarang harga.
- Paket yang dihapus rekanan atau rekanan nonaktif **tidak tampil** `[PRD AC3]`. Rekanan tak merespons: KU-08 (data sementara tidak tersedia).
- Kosong: katalog kosong bila belum ada rekanan yang mengirim paket lewat integrasi `[BRD §7 risiko RT-43]`. Pola kosong DG §5.2 (ilustrasi mint + judul + satu kalimat + satu aksi), `[USULAN-DESAIN]` teks "Belum ada paket tersedia" + "Hubungi Layanan Pelanggan".
- Harga berasal dari rekanan lewat integrasi, **tidak dapat diisi lewat portal rekanan** `[PRD AC5]`.
- `[USULAN-DESAIN]` Tata letak: tab per jenis (Vaksinasi massal, MCU massal, Lab massal), di dalamnya kartu paket (nama paket, rekanan pelaksana, ringkasan isi, ikon+teks lokasi tersedia "On-site" / "Di rekanan", harga + satuan). Filter: lokasi (on-site/di rekanan) dan rekanan; pencarian teks nama paket. Urutan bawaan sesuai server. Tombol kartu: "Lihat detail" (KR-21) dan "Pesan". Indikator "data diperbarui" bila memakai KU-08.
- Layanan vaksinasi/MCU/lab nyata di katalog menunggu konfirmasi legal L-19 `[PRD §12.3]`.

**KR-21 Detail paket** `[TURUNAN]`: isi paket, rekanan, lokasi tersedia, harga, tombol "Pesan paket ini". Sumber hanya menyebut "isi paket"; `[USULAN-DESAIN]` rincian pemeriksaan tampil sebagai daftar bertitik dari data rekanan (tidak dikarang desainer). Bila paket menandai NIK wajib (`requires_nik`), tampil catatan "Paket ini memerlukan NIK peserta" karena menentukan kolom impor `[ADR-028 b.5]`; layanan mana yang mewajibkan NIK `[TIDAK DIKETAHUI]` (PT-66), jadi catatan tampil dari data.

### J9 Memesan (Fase 3)

**KR-22 Pesan: lokasi dan jumlah peserta** `[PRD US-1906 AC1, AC5]`
- Data: paket (terpilih), **lokasi**: on-site (isi alamat kantor) atau di rekanan; **jumlah peserta**.
- Tanggal pelaksanaan **tidak dipilih di langkah ini** menurut PRD (ditetapkan setelah lunas, lihat KR-37).
- `[USULAN-DESAIN]` Satu pesanan = satu paket + satu lokasi. Penyusunan: (1) pilihan radio "Di kantor perusahaan (on-site)" atau "Di klinik/lab rekanan" (hanya opsi yang tersedia untuk paket); (2) on-site: alamat dipilih dari alamat kantor di profil (bawaan) atau isi alamat lain di lokasi yang sama sebagai teks, tidak diketik ulang antar langkah (A13); rekanan: nama dan alamat rekanan ditampilkan baca-saja dari data paket; (3) jumlah peserta (angka bulat, batas bawah/atas dari data paket bila ada); (4) tombol "Lihat ringkasan". Penanda langkah pesanan memakai komponen penanda langkah.
- Alamat on-site di luar area layanan rekanan: server menolak dan UI menampilkan galat per field "Alamat ini di luar area layanan rekanan untuk paket ini", dengan pilihan ganti alamat atau pilih "Di klinik/lab rekanan" `[USULAN-DESAIN]`.
- Lebih dari satu lokasi: pesan terpisah per lokasi `[USULAN-DESAIN]` (dengan tombol "Pesan lagi untuk lokasi lain" di KR-24).

**KR-23 Ringkasan pesanan dan konfirmasi** `[PRD US-1906 AC1–AC4; AB-81]`
- **Rincian harga**: harga per satuan, jumlah, **pajak sebagai baris terpisah** (label dan tarif menunggu PT-28), total `[PRD AC1; DG §4 Rincian harga]`. Semua angka dari server. Margin bukan baris.
- Konfirmasi mengunci harga, margin, dan tarif pajak. Pesanan berstatus "Menunggu pembayaran" dan invoice terbit `[PRD AC2]`.
- Negatif: konfirmasi dua kali hanya membuat satu pesanan dan satu invoice (tombol terkunci + "Memproses...") `[PRD AC3, KU-05]`. **Harga/ketersediaan berubah antara ringkasan dan konfirmasi**: tampilkan nilai terbaru dan wajib disetujui ulang (dialog) `[PRD AC4]`.
- `[USULAN-DESAIN]` Tata letak dua kolom: kiri ringkasan paket, rekanan, lokasi, jumlah (dengan tautan "Ubah" per bagian); kanan **Rincian harga** dan tombol utama "Konfirmasi pesanan dan terbitkan invoice". Di atas tombol: kotak centang syarat pemesanan, tidak tercentang otomatis, **teks syarat `[TIDAK DIKETAHUI]` menunggu legal**, pakai penanda "[teks syarat, draf]". Peringatan netral: "Harga dikunci saat Anda mengonfirmasi" `[PRD AB-81]`. Jumlah peserta tidak dapat diubah sendiri setelah invoice terbit; perubahan lewat KR-40.

**KR-24 Pesanan dibuat dan invoice terbit** `[PRD US-1906 AC2, US-1909 AC1, AC7]`: konfirmasi sukses, nomor pesanan (format `[TIDAK DIKETAHUI]`, tampil dari server), status "Menunggu pembayaran", tautan ke invoice (KR-28) dan **langkah berikutnya** (1 Bayar invoice, 2 Unggah daftar peserta, 3 Menunggu jadwal) `[USULAN-DESAIN]`. Tombol: "Lihat invoice", "Unggah peserta", "Pesan lagi". Email invoice **tanpa lampiran** dan mengarah ke member area `[PRD AC7]`.

### J10 Invoice dan pembayaran (Fase 3)

**KR-25 Daftar pesanan** `[TURUNAN]`: sumber hanya menetapkan **status pesanan**: Menunggu pembayaran, Lunas–menunggu jadwal, Terjadwal, Selesai, Dibatalkan `[PRD US-1906 AC2, UP-80]`. Transisi lengkap menunggu PT-63. `[USULAN-DESAIN]` Kolom: nomor pesanan, paket, rekanan, lokasi (On-site/Di rekanan), jumlah peserta, tanggal pelaksanaan (bila ada), status pesanan, status invoice. Filter status; urut terbaru; paginasi; pencarian nomor pesanan. Kosong: "Belum ada pesanan" + "Lihat katalog". Pemetaan nada badge (guideline hanya memetakan status aplikasi pasien `[DG §5.6]`), usulan: Menunggu pembayaran = peringatan; Lunas, menunggu jadwal = info; Terjadwal = info; Selesai = sukses; Dibatalkan = netral. Ikon/teks wajib (A1).

**KR-26 Detail pesanan** `[TURUNAN]`: hub per pesanan: ringkasan (paket, lokasi, jumlah), status, invoice, jadwal (KR-37), peserta (KR-34), rekap (KR-38). Mengikuti pola "daftar + panel detail" `[DG §10.2]`. Isolasi: pesanan perusahaan lain dijawab KR-14 `[PRD US-1904 AC3]`. `[USULAN-DESAIN]` Susunan: kepala (nomor, status, aksi kontekstual), linimasa status, panel "Langkah berikutnya" yang berubah per status (Menunggu pembayaran: "Lihat invoice dan cara membayar" dan "Unggah peserta"; Lunas, menunggu jadwal: "Ajukan preferensi jadwal" dan "Kirim undangan"; Terjadwal: "Lihat rekap"; Selesai: "Lihat rekap"), lalu tab Ringkasan / Invoice / Peserta / Jadwal / Rekap.

**KR-27 Daftar invoice** `[TURUNAN]`: status invoice: Terbit, Lunas, Lewat jatuh tempo, Dibatalkan `[PRD UP-80, US-1909 AC5]`. `[USULAN-DESAIN]` Kolom: nomor invoice, nomor pesanan, tanggal terbit, jatuh tempo, total, status; filter status; urut terbaru; paginasi; kosong "Belum ada invoice". Nada badge: Terbit = info; Lunas = sukses; Lewat jatuh tempo = bahaya; Dibatalkan = netral. Invoice pengganti menampilkan rujukan ke invoice yang digantikan.

**KR-28 Detail invoice dan unduh** `[PRD US-1909 AC1–AC7; ADR-029]`
- Isi: nomor unik (format `[TIDAK DIKETAHUI]`), data perusahaan, paket, harga per satuan, jumlah, pajak (sisa PT-28), total, tanggal terbit, **jatuh tempo (bawaan 14 hari**, hari kalender atau hari kerja `[TIDAK DIKETAHUI]`, PT-63) `[PRD PB-25; USER C21-P3-3]`, dan **rekening tujuan** (menunggu PT-59/L-21, **arah dana belum diputuskan**; tampilkan penanda, jangan tulis nomor/nama bank).
- Aturan UI: **baris invoice tidak menyebut peserta** `[PRD US-1909 AC6]`. Koreksi berupa pembatalan invoice dan invoice pengganti yang merujuk invoice lama (invoice lama tampil Dibatalkan, jangan ada "ubah invoice") `[PRD AC2]`.
- Aksi: Unduh invoice (hanya data perusahaannya). `[USULAN-DESAIN]` Format unduhan PDF, dibangkitkan saat diminta; tampilan layar berisi data yang sama. Faktur pajak/pemotongan pajak `[TIDAK DIKETAHUI]` (PT-73): jangan mendesain bagian itu.
- "Lewat jatuh tempo": akibat bagi pesanan `[TIDAK DIKETAHUI]` (PT-63). `[USULAN-DESAIN]` UI hanya menampilkan status dan tanggal jatuh tempo, tanpa sanksi atau tenggat tambahan yang dikarang.
- `[USULAN-DESAIN]` Tata letak: kepala (nomor, status, tombol Unduh), blok data perusahaan dan paket, tabel baris invoice (tanpa nama peserta) dengan rincian harga, blok "Cara membayar" (lihat KR-29).

**KR-29 Pembayaran, sisi HR.** Pembayaran adalah transfer bank manual. **Lunas hanya dari mutasi bank yang dicatat Keuangan dan diperiksa terhadap rekening koran, bukan dari bukti transfer kiriman perusahaan** `[PRD US-1910 AC6]`. Status berubah ke Lunas setelah Keuangan mengalokasikan (pesanan "Lunas, menunggu jadwal") `[PRD US-1910 AC2]`. Tidak ada notifikasi atau rekonsiliasi otomatis dari payment gateway `[ADR-029 b.5]`. Bayar sebagian atau kelebihan bayar: invoice tidak Lunas otomatis dan ditandai untuk tindak lanjut `[PRD US-1910 AC3]`; kebijakannya `[TIDAK DIKETAHUI]` (PT-63).
`[USULAN-DESAIN]` Rancangan sisi HR (bagian dari KR-28, bukan layar terpisah):
- Blok "Cara membayar": jumlah yang harus dibayar, **rekening tujuan** (nama bank, nama pemilik, nomor, dari konfigurasi server; isinya `[TIDAK DIKETAHUI]` menunggu PT-59, desainer pakai penanda "[rekening tujuan]"), nomor invoice sebagai berita transfer, tombol "Salin nomor rekening" dan "Salin nomor invoice", tanggal jatuh tempo.
- **Tidak ada unggah bukti transfer oleh HR** (menghindari berkas berisi nomor rekening lengkap di luar Keuangan `[ADR-029 b.3]`). Teks: "Status Lunas ditetapkan setelah transfer Anda kami terima dan diperiksa oleh Keuangan Ruang Sehat. Anda tidak perlu mengirim bukti transfer. Bila sudah transfer tetapi status belum berubah, hubungi Layanan Pelanggan dengan menyebut nomor invoice."
- Status tampil: "Terbit" (menunggu pembayaran), "Lunas" (tanggal pelunasan), "Lewat jatuh tempo". Bila ditandai tindak lanjut: label netral "Perlu tindak lanjut Layanan Pelanggan" tanpa menampilkan nominal yang diterima.
- Setelah Lunas: alert sukses "Pembayaran diterima. Pesanan menunggu penjadwalan" dan panel "Langkah berikutnya" KR-26 beralih ke jadwal dan undangan.

### J11 Mengisi dan mengimpor peserta (Fase 3)

**KR-30 Unggah berkas peserta** `[PRD US-1907 AC1, AC3, AC10, AC11; ADR-028 b.6]`
- Dari pesanan miliknya. Berkas **CSV atau XLSX**, kolom: nama, email dan/atau nomor HP, tanggal lahir, serta NIK **hanya bila paket menandai NIK wajib**. Kolom di luar skema **dibuang saat pratinjau** dan HR diberi tahu `[PRD AC10]`.
- Batas ukuran dan jumlah baris `[TIDAK DIKETAHUI]` (PB-23): tampil dari konfigurasi server, desainer pakai penanda. `[USULAN-DESAIN]` Tautan "Unduh templat CSV" (kolom sesuai skema; kolom NIK hanya bila paket menandai NIK wajib; format tanggal ditetapkan saat implementasi), petunjuk singkat tiga butir, area pilih berkas (tombol pilih berkas, tanpa ketergantungan skrip pihak ketiga), tombol "Unggah dan periksa". Akses halaman dari tab Peserta di KR-26.
- Galat **seluruh berkas ditolak** (tidak ada baris tersimpan) bila: bukan CSV/XLSX asli, rusak, terkunci password, melebihi batas, header tidak dikenali, XLSX bermakro/OLE/entitas eksternal/melebihi batas dekompresi. Pesan menjelaskan sebab `[PRD AC3, AC11]`. Daftar teks galat `[TIDAK DIKETAHUI]`.
- Unggah berkas: batas klien 60 detik `[PRD KU-02]`, bilah progres `[DG §4 Unggah berkas]`.
- Teks pemberitahuan pemrosesan data untuk HR saat mengunggah data karyawan: `[TIDAK DIKETAHUI]` (L-17). `[USULAN-DESAIN]` Sediakan slot catatan teks di atas area unggah (konten dari legal, penanda "[teks pemberitahuan, draf]"); tidak ada kotak persetujuan baru karena sumber tidak menetapkannya.

**KR-31 Pratinjau validasi per baris** `[PRD US-1907 AC1, AC2, AC4, AC5, AC6, AC12, AC13]`
- Tabel per baris: valid atau galat dengan **nomor baris, field, kode alasan, tanpa menggemakan isi sel** `[PRD AC12]`. Contoh jenis galat: field wajib kosong, email/HP tidak valid, tanggal lahir tidak valid atau di masa depan, NIK bukan 16 digit padahal wajib, **ganda** (dalam berkas atau dengan peserta pesanan yang sama: email, HP, NIK) `[PRD AC4, AC5]`. Daftar kode alasan final `[TIDAK DIKETAHUI]`: desainer menyediakan kolom "Kode alasan" + teks penjelas dari server `[USULAN-DESAIN]`.
- Peserta di bawah batas usia anak: perlakuan `[TIDAK DIKETAHUI]` (PT-77, menunggu legal); `[USULAN-DESAIN]` ditampilkan memakai kode alasan generik yang sama, tanpa aturan usia yang dikarang.
- NIK **ditampilkan hanya 4 digit terakhir** `[ADR-028 b.4]`.
- Aksi: Konfirmasi impor (hanya baris valid yang diimpor), Batalkan (berkas dan baris pratinjau dihapus; pratinjau kedaluwarsa juga dihapus) `[PRD AC2, AC7]`. Masa berlaku pratinjau tampil sebagai "Pratinjau berlaku sampai [tanggal/jam dari server]" (angkanya `[TIDAK DIKETAHUI]`).
- Jumlah peserta valid berbeda dari jumlah di pesanan: perilaku `[TIDAK DIKETAHUI]` (PT-63). `[USULAN-DESAIN]` Bila jumlah valid **kurang** dari pesanan: peringatan non-blokir "Jumlah peserta valid (X) kurang dari pesanan (Y)", impor tetap bisa dikonfirmasi. Bila **lebih**: konfirmasi ditahan dengan pesan "Jumlah peserta melebihi pesanan; hubungi Layanan Pelanggan untuk mengubah pesanan" (selaras batas undangan aktif ≤ jumlah peserta pesanan `[PRD US-1908 AC1b]`).
- `[USULAN-DESAIN]` Susunan: kartu ringkasan (valid / galat / ganda, hanya angka), tab Semua / Galat / Ganda / Valid (baris galat didahulukan), tabel pratinjau dengan kolom nomor baris, nama, status, kode alasan; dialog konfirmasi menyebut jumlah yang akan diimpor dan yang tidak. Tombol "Unduh daftar galat" boleh berisi hanya nomor baris, field, kode alasan (tanpa isi sel, sel rumus dinetralkan).
- Pratinjau tidak boleh disimpan di cache (data peserta `no-store`) `[PHASE F1-T90]`.

**KR-32 Hasil impor** `[PRD US-1907 AC2]`: ringkasan jumlah valid, galat, ganda. Hanya jumlah, tanpa isi baris. Langkah berikutnya: kirim undangan (KR-36), `[USULAN-DESAIN]` ditampilkan sebagai tombol utama, nonaktif dengan alasan tertulis "Undangan dapat dikirim setelah invoice lunas" bila belum Lunas (selaras usulan keamanan di `[PRD US-1908 AC1]`). Tautan lain: "Lihat daftar peserta" (KR-34), "Impor berkas lain". Unduh laporan galat bila ada (sel rumus dinetralkan `[PRD AC6]`).

**KR-33 Tambah/ubah/hapus peserta manual** `[USULAN-DESAIN]`. Story US-1907 berjudul "agar tidak mengetik peserta satu per satu" tetapi tidak ada AC untuk input manual, ubah, atau hapus peserta setelah impor. Rekomendasi (validasi dan field sama dengan impor, tanpa field baru, `[PRD US-1907 AC4, AC5]`):
- **Tambah peserta**: formulir satu peserta (nama, email dan/atau nomor HP, tanggal lahir, NIK bila paket mewajibkan). Validasi per field, ganda ditolak dengan pesan per field tanpa membuka data peserta lain. Tidak melebihi jumlah peserta pesanan.
- **Ubah peserta**: sebelum undangan dikirim, semua field boleh diubah. Sesudah undangan dikirim, mengubah email atau nomor HP membatalkan undangan lama dan membuat undangan baru (dialog dampak) agar penautan tetap atas tindakan peserta `[PRD AB-85, US-1908 AC5]`. Mengubah nama atau tanggal lahir setelah tertaut tidak mengubah akun pasien peserta.
- **Hapus peserta** dari pesanan: dialog konfirmasi destruktif, hanya sebelum pesanan Terjadwal; setelahnya lewat Layanan Pelanggan. Undangan yang masih aktif dibatalkan.
- Semua aksi tampil di tab Peserta (KR-34) sebagai tombol "Tambah peserta" dan aksi baris.

### J12 Undangan peserta dan status (Fase 3)

**KR-34 Daftar peserta pesanan** `[PRD US-1908, US-1912 AC1, ADR-028 b.2]`: nama, kontak, tanggal lahir (data unggahan), **status undangan**, **status kehadiran** (hadir, tidak hadir, belum dicatat; sumber data kehadiran `[TIDAK DIKETAHUI]`, PT-64). Status undangan yang disebut PRD: **ditolak** (tanpa alasan dan tanpa data akun pasien) dan **gagal kirim**, serta tautan yang kedaluwarsa, sudah dipakai, atau dibatalkan `[PRD US-1908 AC5, AC6, AC7]`. `[USULAN-DESAIN]` Set status yang ditampilkan: Belum dikirim, Terkirim, Gagal kirim, Tertaut, Ditolak, Kedaluwarsa, Dibatalkan (badge ikon + teks). **Status persetujuan berbagi ringkasan tidak pernah ditampilkan** (HR tidak boleh bisa membedakan "belum setuju", "menarik", dan "belum ada ringkasan", `[PRD US-1914 AC2]`). NIK 4 digit terakhir. `[USULAN-DESAIN]` Tabel: nama, kontak, status undangan, status kehadiran, sesi (bila ada), aksi baris; pencarian nama; filter status undangan dan kehadiran; paginasi; kosong "Belum ada peserta" + "Unggah daftar peserta"; tombol "Tambah peserta" (KR-33) dan "Impor berkas" (KR-30).
- **Jangan menampilkan** apa pun yang berasal dari hasil (status "hasil tersedia", "perlu tindak lanjut", "jadwal ulang karena hasil", penanda serupa) untuk peserta tanpa persetujuan aktif `[PRD US-1912 AC5, AB-87]`.

**KR-35 Detail peserta** `[PRD US-1908 AC8]`: hanya **data yang diunggah HR dan status undangan**. **Data akun pasien peserta tidak tersedia**, jangan ada tautan, foto, atau riwayat. Ringkasan hasil di sini mengikuti KR-39. Mengakses detail peserta oleh HR tercatat sebagai akses sensitif (tidak berefek ke UI selain ketersediaan) `[ADR-028 b.1]`. `[USULAN-DESAIN]` Panel samping: data unggahan, status undangan dengan riwayat singkat (Terkirim, Kirim ulang, Tertaut/Ditolak, tanpa alasan), status kehadiran, blok "Ringkasan" sesuai KR-39. Aksi: Ubah, Kirim ulang undangan, Hapus dari pesanan (KR-33, KR-36).

**KR-36 Kirim ulang undangan** `[PRD US-1908 AC1, AC1b, AC7]`: HR mengirim undangan (waktu pengiriman awal `[TIDAK DIKETAHUI]`, PT-63; usulan keamanan: setelah lunas) dan dapat mengirim ulang. Kirim ulang **dibatasi laju per peserta dan per perusahaan per hari** (angka `[TIDAK DIKETAHUI]`), jumlah undangan aktif tidak melebihi jumlah peserta pesanan: UI perlu pesan "batas tercapai". **Templat email tidak memuat teks bebas dari HR**, jadi **tidak ada kolom pesan** di layar ini. Gagal kirim dicoba ulang otomatis, HR melihat "gagal kirim". Masa berlaku tautan (PB-24) `[TIDAK DIKETAHUI]`, tampil sebagai tanggal/jam dari server. HR tidak melihat apa yang peserta lakukan selain status.
- `[USULAN-DESAIN]` Aksi: "Kirim undangan" untuk semua peserta berstatus Belum dikirim (dialog konfirmasi menyebut jumlah penerima); "Kirim ulang" per peserta (status Gagal kirim, Kedaluwarsa, Dibatalkan). Nonaktif dengan alasan tertulis sebelum invoice Lunas. Pesan batas laju: "Batas kirim ulang tercapai. Coba lagi nanti." (angka batas tidak ditampilkan). Layar menyatakan "Peserta menautkan undangan lewat aplikasi Ruang Sehat; Anda tidak perlu membuatkan akun mereka". Teks email undangan adalah desain email terpisah, memuat pemberitahuan pemrosesan data dari legal `[PRD US-1908 AC1; L-17]` (`[TIDAK DIKETAHUI]`).

### J13 Jadwal pelaksanaan (Fase 3)

**KR-37 Jadwal pelaksanaan** `[PRD US-1911 AC1–AC3, AB-83]`
- **Yang ada di PRD**: jadwal hanya dapat ditetapkan setelah invoice **Lunas** (sebelum itu ditolak). Setelah jadwal ditetapkan pesanan berstatus **Terjadwal**, dan HR serta peserta tertaut **melihat tanggal, jam (dengan zona waktu lokasi, mis. "10.00 WITA"), dan lokasi** `[PRD AC1; DG §6]`.
- Pihak yang menetapkan jadwal dan cara konfirmasi rekanan belum diatur `[PRD US-1911 AC1: "pihak dan cara konfirmasi rekanan: PT-63"]`. **Rekomendasi `[USULAN-DESAIN]`** (konsisten dengan peran Ruang Sehat sebagai perantara `[BRD BR-134]` dan data rekanan lewat gateway):
  1. Pesanan **Lunas, menunggu jadwal**: HR melihat tombol "Ajukan preferensi jadwal". Formulir: satu atau beberapa rentang tanggal yang diinginkan dan, untuk on-site, perkiraan jam mulai/selesai yang diinginkan. Preferensi hanya permintaan, bukan penetapan; label "Preferensi terkirim" tampil di bawah status yang sama (tidak ada status pesanan baru di luar UP-80).
  2. **Rekanan menetapkan jadwal** lewat Ruang Sehat. Pesanan menjadi **Terjadwal** dan HR melihat satu atau lebih **sesi**: tanggal, jam (dengan label zona waktu lokasi, mis. "10.00 WITA", `[PRD AB-40]`), lokasi (alamat kantor untuk on-site, nama/alamat rekanan untuk di rekanan). Bila ada pembagian peserta per sesi, tampil di kolom "Sesi" KR-34.
  3. **HR tidak memilih slot** dan tidak mengubah jadwal sendiri; perubahan jadwal lewat Layanan Pelanggan (tombol "Minta perubahan jadwal" membuka petunjuk kontak, KR-40). Bila rekanan tidak dapat memenuhi preferensi, HR melihat jadwal yang ditetapkan; tidak ada layar "ditolak rekanan" (perilaku penolakan rekanan `[TIDAK DIKETAHUI]`, PT-63).
  4. Keadaan: Lunas–menunggu jadwal tanpa preferensi (tombol Ajukan preferensi); dengan preferensi (ringkasan preferensi baca-saja); Terjadwal (daftar sesi); Selesai (sesi berlalu, baca-saja).
- Pesanan on-site: lokasi = alamat yang dipilih di KR-22. Pesanan di rekanan: nama/alamat rekanan dari data paket; selebihnya tidak dikarang.
- Jadwal bagi peserta tampil di aplikasi pasien `[PRD AC3]` (bukan web).

### J14 Memantau pesanan dan kehadiran (Fase 3)

**KR-38 Rekap kehadiran dan ekspor** `[PRD US-1912 AC1–AC6]`
- Untuk pesanan **Terjadwal atau Selesai**: setiap peserta dengan status undangan dan status kehadiran. **Tidak ada** nilai hasil, PDF, catatan pemeriksa. Ringkasan hanya sesuai KR-39.
- Ekspor CSV: hanya peserta perusahaannya, sel berawalan rumus dinetralkan (aman di Excel), ekspor dibangkitkan saat diminta dan **tidak disimpan di server** `[PRD AC3, AC6]`. Ringkasan di ekspor hanya untuk peserta yang persetujuannya aktif saat ekspor `[PRD US-1914 AC5]`. `[USULAN-DESAIN]` Tombol "Ekspor CSV" membuka dialog sebelum unduh: "Berkas berisi data pribadi peserta. Simpan dengan aman." Kolom ekspor sama dengan tabel (nama, kontak, status undangan, status kehadiran, ringkasan bila persetujuan aktif); nama berkas memuat nomor pesanan.
- Respons rekap `no-store` (tidak ada tombol "tersimpan offline") `[PRD AC6]`. Sumber/definisi kehadiran `[TIDAK DIKETAHUI]` (PT-64). Klasifikasi legal (L-18) membuat akses detail dicatat sebagai akses sensitif.
- K `[USULAN-DESAIN]`: pesanan belum Terjadwal: "Rekap tersedia setelah pesanan terjadwal" + tautan ke KR-26; pesanan tanpa peserta: "Belum ada peserta" + "Unggah daftar peserta". Layar dapat difilter status kehadiran dan dicari nama; sesi ditampilkan bila ada.

### J15 Ringkasan hasil dengan persetujuan (Fase 3)

**KR-39 Ringkasan hasil peserta** `[PRD US-1913, US-1914, AB-84, AB-87; ADR-028 b.3]`
- Persetujuan diberikan/ditarik **peserta di aplikasi pasien** per peserta, per perusahaan, per pesanan, bawaan tidak berbagi `[PRD US-1913 AC1]`. Layar HR tidak punya kontrol persetujuan.
- Bila persetujuan aktif dan ringkasan tersedia: ringkasan tampil di rekap atau detail peserta `[PRD US-1914 AC1]`. **Isi, cakupan, format ringkasan, dan siapa yang menetapkan `[TIDAK DIKETAHUI]`** (PT-65). Contoh "fit/unfit" hanya contoh dari user `[USER C21-P2-4]`, bukan daftar nilai final. Rancang slot generik berlabel `[USULAN-DESAIN]`: teks ringkasan tampil sebagai label teks netral (tanpa kode warna hijau/merah atau ikon yang menyiratkan penilaian kesehatan), dengan keterangan "Dibagikan atas persetujuan peserta" di dekatnya dan di tabel rekap sebagai kolom "Ringkasan".
- Bila persetujuan belum diberikan atau ditarik, **dan bila ringkasan belum ada**: tampilan **seragam "Ringkasan tidak tersedia"** (teks ditetapkan `[PRD US-1914 AC2]`). HR tidak boleh bisa membedakan dua kasus itu.
- **Jangan** mendesain: hasil lengkap, unduh PDF hasil, catatan pemeriksa, jumlah/persentase/grafik/statistik hasil, daftar "perlu tindak lanjut", penanda warna status kesehatan `[PRD US-1914 AC3, AC7, US-1912 AC5]`. Agregat hasil tidak disediakan di rilis ini.
- Persetujuan ditarik saat HR sedang membuka: muat ulang atau URL lama tidak menampilkan ringkasan, tanpa salinan di cache `[PRD US-1914 AC8]`. Teks persetujuan final menyatakan penarikan berlaku ke depan dan tidak menarik ekspor yang sudah diunduh HR; teks final (draf sampai L-18) milik layar aplikasi pasien. `[USULAN-DESAIN]` Di dialog ekspor (KR-38) tambahkan satu kalimat: "Ringkasan yang sudah Anda unduh tidak ikut terhapus bila peserta menarik persetujuan", selaras isi teks persetujuan `[PRD US-1913 AC4]`.

### J16 Pembatalan/perubahan pesanan (Fase 3)

**KR-40 Pembatalan atau perubahan pesanan.** Status "Dibatalkan" ada `[PRD UP-80]`, tetapi pembatalan oleh perusahaan, refund, perubahan jumlah peserta setelah invoice, bayar sebagian/kelebihan bayar, penolakan rekanan, dan perusahaan nonaktif saat pesanan berjalan belum diatur `[PRD PT-63]`. Matriks refund aplikasi pasien **tidak berlaku** untuk pesanan korporat `[ADR-029 konsekuensi]`. `[USULAN-DESAIN]` Rekomendasi yang tidak membuat aturan uang baru:
- **Pesanan "Menunggu pembayaran"** (belum ada pembayaran): tombol "Batalkan pesanan" dengan dialog destruktif yang menyebut dampak (pesanan Dibatalkan, invoice dibatalkan, tidak dapat dipulihkan). Invoice yang dibatalkan tampil Dibatalkan (tidak diubah, `[PRD US-1909 AC2]`).
- **Status lain** (Lunas, Terjadwal, Selesai) dan permintaan **ubah jumlah peserta, ubah jadwal, atau batalkan setelah bayar**: tidak ada tindakan mandiri. Tombol "Hubungi Layanan Pelanggan untuk perubahan" membuka panel petunjuk kontak (isi kontak `[TIDAK DIKETAHUI]`) dengan nomor pesanan yang dapat disalin.
- **Tidak ada layar refund** dan tidak ada teks yang menjanjikan pengembalian dana.
- Perusahaan dinonaktifkan: lihat KR-15, pesanan tidak diakses.

## 6. Aturan keamanan dan privasi yang memengaruhi UI

| # | Aturan | Dampak desain | Sumber |
| --- | --- | --- | --- |
| S1 | **Anti-enumerasi.** Pendaftaran dengan email yang sudah terdaftar menampilkan layar yang sama dengan sukses; undangan HR dijawab sama; login gagal satu pesan seragam | Teks dan layar identik, tanpa petunjuk selisih waktu/kata | `[PRD US-1901 AC4; US-103 AC2; DG §9]` |
| S2 | NIB ganda: pesan generik tanpa nama perusahaan lain + arahan ke Layanan Pelanggan | Tidak ada "perusahaan X sudah terdaftar" | `[PRD US-1901 AC7]` |
| S3 | HR hanya melihat data perusahaannya. Objek perusahaan lain dijawab "Data tidak ditemukan" | Tidak ada layar 403 terpisah, tidak ada pencarian lintas perusahaan | `[PRD AB-86, US-1904 AC3, KU-03]` |
| S4 | NIB/NPWP termasking di profil; NIK peserta hanya 4 digit terakhir; nomor rekening hanya 4 digit terakhir | Pola masking: `************1234` | `[DG §6; ADR-028 b.4; PRD §3.1]` |
| S5 | HR tidak pernah melihat hasil lengkap; ringkasan hanya bila persetujuan aktif; tidak ada kanal turunan (status, penanda, agregat, baris invoice) | Lihat KR-34, KR-38, KR-39, KR-28 | `[PRD AB-84, AB-87]` |
| S6 | HR tidak melihat data akun pasien peserta | Tidak ada tautan ke profil pasien | `[PRD US-1908 AC8]` |
| S7 | Respons berisi rekap atau ringkasan **tidak di-cache** | Jangan desain "simpan offline" atau "kembali ke hasil sebelumnya" | `[PRD US-1912 AC6, US-1914 AC8]` |
| S8 | Pesan galat impor tidak menggemakan isi sel | Hanya nomor baris, field, kode alasan | `[PRD US-1907 AC12]` |
| S9 | Kotak persetujuan tidak tercentang otomatis; satu butir satu kotak | Form daftar KR-03 | `[DG §4 Checkbox; PRD US-1901 AC1]` |
| S10 | Kode pemulihan tampil sekali; password dan berkas tidak disimpan di klien | KR-06 dan KR-01–03 | `[DG §9, D-K01]` |
| S11 | Sesi 30 menit idle, 12 jam mutlak; sesi HR nonaktif berakhir pada permintaan berikutnya | KR-08, KR-15 | `[PRD NFR-SEC-04, US-1904 AC5, AC6]` |
| S12 | Password dan TOTP boleh ditempel; tanpa CAPTCHA teka-teki | Tanpa `paste` diblokir | `[DG §7.2 A14]` |
| S13 | Email undangan peserta dan invoice tanpa data kesehatan dan (invoice) tanpa lampiran | Pengaruh ke desain email, bukan layar | `[PRD US-1908 AC1, US-1909 AC7]` |
| S14 | Aspek teknis yang **tidak perlu** dipikirkan desainer: CSRF, token, cookie sesi, isolasi tenant di server | Tidak ada elemen UI khusus | `[ADR-027 b.1, b.6; ADR-028 b.1]` |

## 7. Sistem desain

Semua nilai berikut sudah ada di design guideline (v1.2) dan **disetujui user** sebagai arah (UP-58–UP-64) `[DG §13]`, kecuali ditandai.

**Prinsip**: nama tampil "Ruang Sehat" dengan spasi di semua tempat; wordmark "RuangSehat" tidak dipakai; slogan pamflet tidak dipakai (statusnya `[TIDAK DIKETAHUI]`) `[DG §3.5]`. Satu sistem token bersama di `web/packages/ui` (web) `[DG §1 P7, §2]`. Mode gelap tidak dirancang di rilis 1 `[DG §3.1 UP-61]`.

**Warna (token)** `[DG §3.1]`: `color.brand.primary #087F78`, `color.brand.mint #DFF4ED`, `color.bg.app #FAFBF7`, `color.bg.surface #FFFFFF`, turunan: `primaryStrong #065F59`, `text.primary #1A2B2E`, `text.secondary #5C6B6F`, `border.control #7A898C`, `border.divider #DCE3E1` (dekoratif saja), `bg.neutral #EEF1F0`, `danger #B3261E / dangerBg #FDECEA`, `success #1E7A3E`, `warning #8A5300 / warningBg #FFF4DB`, `text.onPrimary #FFFFFF`. **Aturan kontras:** semua teks ≥ 4,5:1; `brand.primary` **tidak** dipakai untuk teks di atas mint (4,24:1), pakai `primaryStrong`; komponen UI ≥ 3:1; status tidak hanya warna `[DG §3.2, §7.2 A1–A3]`.

**Tipografi** `[DG §3.3]`: font sistem (`system-ui, -apple-system, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif`), tanpa berkas font tambahan. Skala web: `display 28/36 700`, `titleLg 24/32 600`, `title 18/24 600`, `subtitle 16/24 600`, `body 14/20 400`, `bodySm 13/18 400`, `label 14/20 600`, `caption 12/16 400` (tidak untuk informasi wajib sebelum bertindak). Tinggi baris ≥ 1,3×. Angka uang/tabel pakai angka tabular. Plus Jakarta Sans masih opsi bila disetujui `[DG UP-59]`, **belum diputuskan sebagai font merek**.

**Spasi/radius/elevasi** `[DG §3.4]`: kelipatan 4 (4–64), jarak halaman web 24; radius md 8 (isian, tombol web), lg 12 (alert), xl 16 (kartu, dialog), full 999 (chip, badge); kartu `elevation.1`, dialog `elevation.3`.

**Ikon** `[DG §3.5]`: **SVG sebaris** yang disimpan lokal di `web/packages/ui`; sumber gambar SVG dan lisensinya `[TIDAK DIKETAHUI]`. Ikon tanpa teks wajib punya label; ikon dekoratif disembunyikan dari pembaca layar. Ilustrasi keadaan kosong: lingkaran mint + satu ikon (tanpa ilustrasi orang).

**Komponen yang dipakai ulang** `[DG §4]`: Tombol (utama/sekunder/teks/destruktif; satu tombol utama per layar; tombol utama **tidak dinonaktifkan** karena validasi), Isian teks, Pilihan, Checkbox, Radio, Kartu, Badge status (ikon + teks), Tab, Navigasi samping (menu hanya memuat fitur yang diizinkan), App bar/kepala, Dialog (aksi destruktif berlabel aksinya), Peringatan, Toast (bukan untuk galat yang butuh tindakan), **Rincian harga**, Linimasa, **Unggah berkas**, Tabel web, Paginasi, Kerangka. Kosong: pola DG §5.2. Galat: DG §5.3. Waktu/sesi: DG §5.5.

**Komponen yang belum ada di guideline dan perlu diusulkan desainer** (tandai usulan, **tanpa pustaka pihak ketiga**, komponen dibangun sendiri `[DG §2]`): penanda langkah pendaftaran (ada di wireframe sebagai `steps`), tabel pratinjau impor dengan status per baris, ringkasan angka "valid/galat/ganda", pemilih lokasi (on-site/rekanan), kartu paket katalog, badge status pesanan dan invoice (pemetaan nada).

**Bahasa dan nada** `[DG §6; PRD NFR-LOC-01/02]`: Bahasa Indonesia saja; sapaan "Anda"; tombol berupa kata kerja ("Kirim pendaftaran", "Unduh invoice"); istilah PRD dipakai apa adanya ("perusahaan", "pesanan", "rekanan", "peserta", "invoice"). Uang: `Rp150.000` (tanpa spasi, titik ribuan, tanpa desimal). Tanggal `Sen, 1 Mar 2027`; jam `10.00` dengan label zona untuk jadwal layanan (`10.00 WITA`) `[PRD AB-40; DG §6]`. Zona waktu untuk waktu administratif (pengajuan, penerbitan invoice, jatuh tempo): `[USULAN-DESAIN]` WIB dengan label "WIB" (seperti contoh wireframe); jadwal layanan memakai zona lokasi dengan label. Pesan galat menyebut cara memperbaiki. Tidak ada istilah klinis detail di notifikasi.

**Aksesibilitas (WCAG 2.2 AA)** `[DG §7.2]`: keyboard penuh dan urutan fokus logis (A7); cincin fokus 2 px `primaryStrong` + jarak 2 px, tidak tertutup kepala/toast/dialog (A8); target web ≥ 24×24 px, standar komponen 40 px (A9); label untuk setiap kontrol (A11); galat teks per field, ditandai `aria-invalid` (A12); pesan status diumumkan tanpa memindah fokus (A15); teks dapat diperbesar 200% (A4); reflow 320 px tanpa gulir dua arah kecuali tabel data (A5); spasi teks tanpa terpotong (A6); `lang="id"`; landmark `header/nav/main/footer`; tautan "Lewati ke isi". Batas waktu sesi: SC 2.2.1 dengan peringatan + "Tetap masuk" (A16, §5.5); batas 12 jam diterima sebagai pengecualian essential `[DG PT-43 terjawab]`. Autentikasi aksesibel: tanpa CAPTCHA, paste boleh (A14). Uji: navigasi keyboard Playwright untuk alur utama web `[DG §7.4]`.

**Responsif** `[DG §8]`: ≥ 1280 px kepala 56 px + navigasi samping 240 px + isi, daftar + panel detail berdampingan; 768–1279 px navigasi dapat diciutkan, panel detail di bawah daftar; < 768 hingga 320 px navigasi menjadi tombol menu, satu kolom, tabel boleh gulir horizontal. Perangkat utama HR tidak ditetapkan di sumber; `[USULAN-DESAIN]` rancang **desktop-first** (HR bekerja dengan tabel dan unggah berkas, dan guideline §8 memakai tata letak ≥ 1280 px sebagai dasar web), tetapi semua layar tetap utuh di tablet dan lebar sempit. Tema Keycloak: satu kolom, lebar maksimum 400 px, utuh pada 320 px `[DG §8]` (catatan: wireframe K01-6 memakai lebar 520 px untuk layar TOTP; `[USULAN-DESAIN]` ikuti guideline 400 px sebagai dasar; lebar sedikit lebih besar untuk layar yang memuat QR boleh dipakai bila perlu, dengan tetap utuh di 320 px).

## 8. Batasan teknis untuk desainer

- **Tumpukan:** Next.js (aplikasi `web/korporat`) dengan server Next.js sebagai BFF; komponen di paket UI bersama; token sebagai CSS custom properties `[ADR-027 b.1; DG §2]`.
- **Tidak ada pustaka UI pihak ketiga.** Komponen dibangun sendiri `[DG §2 KEPUTUSAN-163]`.
- **Tanpa font, skrip, atau sematan pihak ketiga** (peta, widget chat, video, tag analitik), dan **tidak ada cookie pihak ketiga**. CSP ketat melarang skrip inline dan sumber pihak ketiga (ditetapkan untuk situs dan backoffice, diterapkan serupa pada member area `[ADR-027 b.5–b.6; PRD US-1802 AC9]`). **Ikon, gambar, dan aset harus lokal** (SVG sebaris atau berkas milik proyek). Tidak ada analitik/pelacakan yang diputuskan `[TIDAK DIKETAHUI]`. Anggap tidak ada. Jangan merancang spanduk persetujuan cookie untuk analitik. Satu-satunya cookie adalah cookie sesi pihak pertama.
- **Layar masuk, TOTP, aktivasi undangan, atur ulang password, dan verifikasi email dirender Keycloak** dengan tema login yang memperluas tema bawaan (stylesheet dan template sendiri), tidak di Next.js `[DG §2; ADR-027 b.3]`. Konsekuensi: desain layar itu dibatasi pola form dan alur bawaan Keycloak (urutan langkah, tombol, pesan sistem tertentu tidak sepenuhnya bebas). Serahkan desain sebagai skin (warna, tipografi, jarak, teks yang dapat diatur), dan tandai bagian yang butuh konfirmasi teknis. Cakupan fleksibilitas tema `[TIDAK DIKETAHUI]`. Dicek Tech Lead saat implementasi.
- **Tiap permintaan ke server:** idempotensi (`KU-05`), batas klien 15 detik, unggah 60 detik `[PRD KU-02]`. Kode referensi galat = `correlation_id` `[DG §5.3]`.
- **Tidak ada optimistic UI untuk konfirmasi pesanan/pembayaran**: status berubah setelah server mengonfirmasi `[DG §5.1]`.
- **Ukuran berkas:** JPG/PNG ≤ 5 MiB, PDF ≤ 10 MiB `[PRD NFR-SEC-08]`.
- **Dukungan peramban:** dua versi stabil terakhir Chrome, Edge, Firefox, Safari `[PRD NFR-COMP-03]`.

## 9. Deliverable yang diminta dari desainer (usulan)

Semua butir di bawah **usulan**. Dapat disesuaikan user.

1. **Alur pengguna (user flow)** J1–J16 dalam satu diagram, dengan titik keputusan berdasarkan status perusahaan, dan tanda fase.
2. **Wireframe hi-fi** per layar **Fase 1 (KR-01–KR-16)**, desktop (≥ 1280), tablet (≈ 768), dan sempit (320–480), **dengan semua keadaan**: kosong, memuat, galat per field, galat sistem, sukses, dialog.
3. **Skin tema Keycloak** untuk KR-05–KR-07 (dan KR-17, KR-18 Fase 2).
4. **Wireframe Fase 2 (KR-17–KR-19) dan Fase 3 (KR-20–KR-40)** dengan keadaan lengkap, mengikuti rancangan `[USULAN-DESAIN]` di §5 (boleh disempurnakan desainer). Tandai setiap aturan `[USULAN-DESAIN]` di wireframe. Angka, harga, nama rekanan, rekening, dan teks hukum memakai penanda yang jelas bukan data nyata.
5. **Spesifikasi komponen baru** (§7), menggunakan token yang ada, dengan semua state (default, hover, fokus, nonaktif, galat, memuat).
6. **Catatan aksesibilitas** per layar utama: urutan fokus, label, peran ARIA/landmark, pengumuman pesan status, hasil hitung kontras pasangan warna baru bila ada.
7. **Spesifikasi teks (copy deck)**: seluruh teks UI dalam Bahasa Indonesia, dengan kolom "ditetapkan" (kutipan dari brief) / "draf desainer" / "menunggu keputusan".
8. **Prototipe klik** untuk alur J1 → J3 → J5 (Fase 1) minimal. Fase 3 opsional setelah keputusan.
9. **Catatan kebutuhan aset/konten** yang ditemukan desainer saat bekerja (bukan pertanyaan alur; alur sudah diputuskan lewat `[USULAN-DESAIN]` di §5).

Aset yang tidak boleh dibuat/diisi desainer: logo final, ikon aplikasi, teks hukum, nama domain, kontak, daftar rekanan, harga, rekening tujuan.

## 10. Aset, konten, dan parameter bisnis yang masih dibutuhkan

Bagian ini **hanya** memuat aset, konten, dan parameter bisnis. Alur dan perilaku layar sudah ditetapkan di §5 (fakta PRD atau `[USULAN-DESAIN]`) dan tidak ditanyakan ulang. Sampai butir di bawah tersedia, desainer memakai penanda yang jelas bertanda draf. Pemilik mengikuti PRD §12 bila ada nomor rujukan.

| # | Aset/konten/parameter | Dipakai di | Rujukan | Pemilik |
| --- | --- | --- | --- | --- |
| 1 | Logo final "Ruang Sehat", ikon aplikasi, nilai warna mint yang benar, slogan dipakai atau tidak | Semua layar, email | PT-40, PT-41, UP-60 | User |
| 2 | Nama domain situs dan Web Korporat | Tautan situs ke Web Korporat, email, tema Keycloak | PT-46, PT-70 | User, legal |
| 3 | Kontak Layanan Pelanggan (nomor, email, jam layanan) | KR-04, KR-07, KR-15, KR-16, KR-29, KR-40 | `[TIDAK DIKETAHUI]` | User |
| 4 | Teks persetujuan pemrosesan data HR (versi, tanggal berlaku) | KR-03 | L-20 | Legal |
| 5 | Teks kebijakan privasi dan syarat; teks syarat pemesanan/invoice | Situs, KR-23 | L-08, L-20, PT-24 | Legal |
| 6 | Teks pemberitahuan pemrosesan data untuk HR saat unggah peserta dan untuk email undangan | KR-30, email undangan | L-17, PT-76 | Legal |
| 7 | Format resmi NIB dan NPWP (panjang, pola) | KR-01 | US-1901 AC3 | Tim produk, verifikasi ke sumber resmi |
| 8 | Daftar kode alasan penolakan beserta label teksnya | KR-10 | US-1902 AC3 | User, Verifikator |
| 9 | Definisi "pejabat berwenang" untuk bukti kewenangan | KR-03 | `[TIDAK DIKETAHUI]` | User, legal |
| 10 | Rekening tujuan invoice dan arah dana | KR-28, KR-29 | PT-59, L-21 | User, CEO, legal |
| 11 | Satuan harga, jumlah peserta minimum, nama dan logo rekanan yang boleh tampil, isi paket dan harga (dari integrasi, bukan dikarang) | KR-20, KR-21 | PT-62, PT-70, L-19 | CEO, rekanan |
| 12 | Format nomor pesanan dan nomor invoice; faktur pajak dan tarif pajak | KR-24, KR-28 | PT-28, PT-73 | CEO |
| 13 | Batas ukuran dan jumlah baris berkas peserta (PB-23); masa berlaku undangan (PB-24); batas kirim ulang undangan; masa berlaku tautan undangan HR | KR-13, KR-30, KR-36 | PB-23, PB-24 | Super Admin, tim produk |
| 14 | Definisi dan sumber kehadiran; isi, cakupan, format ringkasan hasil | KR-38, KR-39 | PT-64, PT-65, L-18 | User, konsultan medis |
| 15 | Daftar layanan yang mewajibkan NIK; perlakuan peserta di bawah batas usia anak | KR-21, KR-31 | PT-66, PT-77 | User, legal |
| 16 | Sumber SVG ikon web dan lisensinya; keputusan font merek (sistem atau Plus Jakarta Sans) | Semua layar | DG §3.3, §3.5, UP-59 | Desainer, user |
| 17 | Ketentuan kebijakan bisnis pembatalan/perubahan pesanan, penolakan rekanan, dan perusahaan nonaktif saat pesanan berjalan (hanya bila user ingin mengganti `[USULAN-DESAIN]` KR-40) | KR-40 | PT-63 | User |

Catatan: butir 17 bukan pertanyaan alur; rancangan KR-40 tetap berlaku sampai ada perubahan.

---

## Lampiran A. Daftar sumber

Dokumen proyek (semua di `docs/`; **desainer tidak memerlukan akses**, isi relevan sudah dikutip di atas; user dapat melampirkan bila diminta):

| Kode di brief | Dokumen | Bagian yang dipakai |
| --- | --- | --- |
| PRD | `03-prd.md` v1.4 | §1.1–§1.2 (komponen dan peran), §2.2 (non-scope), §3.1 (matriks akses CR-021), §3.12 (AB-80–AB-87), §4 (KU-01–KU-10), §5 EP-19 US-1901–US-1917, §6 NFR-PERF-04, NFR-SEC-02/03/04/08, NFR-COMP-03, NFR-LOC-01/02, §11 (UP-80 status), §12.3 (L-17–L-21), §12.6 (PT-59–PT-79) |
| BRD | `02-brd.md` | §2.2 peran, §4.2 non-scope, BR-130–BR-145, BR-040/041 |
| CR-021 | `cr/CR-021-analisis.md` v0.3, `cr/CR-021-wawancara.md` | Putaran 0–5 (`[USER C21-Px-y]`), kontradiksi K-01–K-12 |
| ADR | `adr/ADR-027-...`, `ADR-028-...`, `ADR-029-...` | Dua aplikasi Next.js, realm `korporat`, tenant, impor, persetujuan, invoice B2B |
| DG | `06-design-guideline.md` v1.2 | §2, §3 token, §4 komponen, §5 pola kondisi, §6 konten, §7 aksesibilitas, §8 responsif, §9 modul T1, §10.2 D-K01 dan D-S01, §12 |
| WF | `wireframes/k01-korporat-hr.html` | Layar K01-1–K01-11 (draf F1-T84) |
| PROTO | `prototype/korporat/01…07-*.html` | Tujuh layar prototipe |
| PHASE | `07-phase-plan.md` v0.4 | F1-T85–F1-T111, pembagian S3 (K1/K2/K3) |

Sumber eksternal (WCAG 2.2 dan lainnya) tercatat di `docs/sources.md` dan dipakai lewat design guideline; brief ini tidak mengutip fakta eksternal baru.

## Lampiran B. Catatan konsistensi sumber

- PRD v1.4 (tanggal 2026-11-20) hanya mengganti istilah "markup" menjadi "margin" `[PRD §14]`; wawancara CR-021 masih memakai "markup". Brief memakai **"margin"** dan, sesuai AB-13, margin tidak ditampilkan sebagai baris.
- Wireframe K01 menyebut aktor "Verifikator" dan sub-peran. Sub-peran baru atau tidak serta persetujuan orang kedua adalah urusan backoffice, `[TIDAK DIKETAHUI]` (PT-67 sisa), tidak memengaruhi layar HR.
- Wireframe K01-6 memakai bingkai TOTP selebar 520 px, sedangkan guideline §8 membatasi tema login 400 px. Ditandai di §7.
- Teks sebagai kutipan "draf" di wireframe/prototipe belum final dan belum disetujui user, kecuali yang ditandai "ditetapkan".
