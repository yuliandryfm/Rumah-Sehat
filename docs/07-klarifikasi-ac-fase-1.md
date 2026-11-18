# Klarifikasi Acceptance Criteria Fase 1 (temuan QA test plan §9)

Tanggal: 2026-11-18
Versi: 0.3 (disetujui di Checkpoint 2, 2026-11-18, `docs/checkpoint-2.md`; teks AC sudah diterapkan di PRD v1.2. Pilihan user yang berbeda dari opsi di sini: MFA pasien M1 dan US-103 AC6 diambil sebagai CR-017)
Penyusun: Claude (persona Product Manager, dengan tinjauan Tech Lead), Mode Otomatis
Pemutus: user (project manager, meneruskan ke CEO dan CTO) `[DOK: 00-project-brief.md]`
Tingkat ketelitian: **T1** untuk butir autentikasi, sesi, data identitas, dan pemisahan tugas admin (modul kritis di `CLAUDE.md`): T-01–T-09, T-12 (hanya bagian usia pemilik akun), T-13–T-16, T-18, T-19, T-21, T-22, dan §2. **T2** untuk butir lain.

**Status dokumen.** Isinya usulan. Teks AC di `docs/03-prd.md` **tidak diubah** di sini. AC PRD yang sudah lulus gate baru diubah setelah user menyetujuinya di Checkpoint 2. Butir Teknis mengikuti dokumen gate yang sudah ada dan dicatat sebagai KEPUTUSAN. Butir Bisnis/produk disajikan sebagai opsi, usulan default, dan dampaknya (UP), dan **tidak diputuskan penyusun**.

**Label sumber.** `[DOK: file]` dokumen proyek yang lulus gate. `[SRC-xx]` fakta eksternal terverifikasi di `docs/sources.md`. `[KEPUTUSAN-xx]` keputusan teknis di `docs/decision-log.md`. `[UP-xx]` usulan penyusun yang wajib diputuskan user di Checkpoint 2. `[TIDAK DIKETAHUI]` informasi belum ada. Nomor temuan T-xx merujuk ke `docs/test-plan.md` §9 `[DOK: test-plan.md]`.

---

## 1. Ringkasan

### 1.1 Cakupan

Gate Tahap 7 meminta semua story Fase 1 memenuhi Definition of Ready, yaitu AC yang jelas dan bisa diuji `[DOK: backlog.md]`. Dokumen ini menjawab temuan T-01–T-19, ditambah PT-44/UP-70 dan T7-F5 `[DOK: 07-catatan-phase-plan.md]`.

**Penyimpangan dari instruksi, dengan alasan.** Tiga temuan bernomor di atas T-19 ternyata berlaku di Fase 1:
- T-20 (KU-02) berlaku untuk semua story, termasuk story Fase 1;
- T-21 menyangkut US-1501 dan T-22 menyangkut US-1504, dan keduanya story Fase 1 (`docs/07-phase-plan.md` §4, task F1-T25, F1-T68, F1-T69) `[DOK: 07-phase-plan.md]`.

Tanpa usulan rinci untuk ketiganya, gate DoR Fase 1 tetap tidak terpenuhi. Karena itu ketiganya ditulis rinci di §3, sama seperti T-01–T-19. Temuan T-23–T-35 hanya dipetakan ke fase dan tenggatnya (§4).

### 1.2 Jumlah butir per jenis

| Jenis | Temuan | Jumlah |
| --- | --- | --- |
| Teknis (mengikuti dokumen gate; KEPUTUSAN) | T-01, T-02, T-04, T-05, T-07, T-08, T-09, T-10, T-20, T-21, T-22 | 11 (8 dari T-01–T-19) |
| Bisnis/produk (opsi + usulan default; UP) | T-03, T-11, T-14, T-17, T-18, T-19 | 6 |
| Campuran (bagian teknis + bagian bisnis) | T-06, T-12, T-13, T-15, T-16 | 5 |
| Temuan terkait yang digabung | PT-44/UP-70, T7-F5 | masuk UP-73 |

### 1.3 Keputusan bisnis untuk Checkpoint 2

| UP | Isi | Temuan | Usulan default |
| --- | --- | --- | --- |
| UP-73 | Keputusan keamanan terpadu tentang enumerasi akun dan NIK (§2) | T-03, T-06 (pesan), T-13 (email sudah dipakai), T-16, T-18 (teks alasan), PT-44/UP-70, T7-F5 | Jalur tanpa login tidak memberi sinyal keberadaan akun, dan informasi khusus hanya dikirim ke kotak masuk pemilik. Jalur yang sudah login boleh memberi tahu status akun miliknya sendiri; sinyal tentang akun atau NIK lain diminimalkan dengan pesan umum, batas laju, dan audit |
| UP-74 | Siklus status identitas: pengajuan NIK baru dari status "Disetujui", status setelah batal, status setelah persetujuan gagal karena NIK sudah dipakai | T-14, T-15 (AC5), T-18 | Pengajuan dari "Disetujui" diizinkan dan status menjadi "Menunggu verifikasi" (UP-39). Setelah batal, status kembali ke status sebelum pengajuan. Bila NIK sudah dipakai, Verifikator menolak dengan alasan yang sudah terpilih |
| UP-75 | Data rujukan dikelola lewat migrasi data di rilis 1: versi teks dan butir persetujuan, serta daftar alasan penolakan identitas | T-11, T-17 | Lewat migrasi data yang direview, tanpa layar admin. Daftar alasan awal ada di §3 T-17 |
| UP-76 | Akun admin Ruang Sehat: bootstrap Super Admin pertama, menonaktifkan akun sendiri, kombinasi sub-peran | T-19 | Bootstrap lewat perintah satu kali di jalur administrasi production, dengan persetujuan orang kedua (Control Group OpenBao), hanya bila belum ada Super Admin aktif maupun yang menunggu aktivasi; undangan sekali pakai berlaku 24 jam. Menonaktifkan akun sendiri ditolak. Auditor tidak boleh digabung dengan sub-peran lain, dan Auditor terakhir tidak dapat dicabut. Pemberitahuan pemberian sub-peran sensitif diusulkan karena risiko WEB-07/M-16 masih tersisa. Minimal dua Super Admin aktif sebelum go-live |
| UP-77 | Usia minimum pemilik akun pasien | T-12 (sebagian) | Disediakan parameter usia minimum pemilik akun, dengan nilai dari jawaban legal (L-08). Sampai ada jawaban, nilai diisi user sebelum go-live |
| UP-78 | Celah ASVS Level 2 di manajemen sesi: 7.5.2 (melihat dan mengakhiri sesi aktif) dan 7.1.2 (kebijakan jumlah sesi paralel) | Review keamanan C-14 (§5 S-06) | Layar "Perangkat dan sesi aktif" (daftar dan tombol akhiri) dibangun di Fase 2 untuk semua aplikasi. Jumlah sesi paralel tidak dibatasi, dan kebijakannya didokumentasikan |
| — (S-01) | MFA pasien terhadap target ASVS Level 2. **Perubahan atas ADR-011 yang sudah diterima** | §5 S-01 | Tidak ada usulan default. Opsi M1–M4 disajikan untuk diputuskan user |

### 1.4 Temuan sampingan untuk reviewer keamanan

Temuan di luar T-01–T-35 yang muncul saat menyusun dokumen ini dan saat review keamanan ada di §5. Dua yang perlu disorot:
- **S-01.** ASVS 5.0.0 mewajibkan MFA mulai Level 2 `[SRC-265]`, sedangkan ADR-011 menargetkan Level 2 untuk semua komponen tetapi tidak mewajibkan MFA bagi pasien `[DOK: adr/ADR-011-target-asvs-autentikasi-sesi-mfa.md]`. Penerimaan risiko pasien tanpa MFA sudah ada sejak Tahap 4 (threat model APP-04). Yang keliru adalah klaim kepatuhan Level 2 untuk realm `pasien`. Opsi M1–M4 dibawa ke Checkpoint 2 sebagai perubahan ADR yang sudah diterima, tanpa diputuskan penyusun.
- **S-02.** Penghentian sesi setelah atur ulang password dan ubah email sekarang ditetapkan sebagai `[KEPUTUSAN-198]`.

---

## 2. Keputusan keamanan terpadu: enumerasi akun dan NIK (UP-73, T1)

### 2.1 Masalah

Beberapa AC Fase 1 memberi tahu pihak lain apakah sebuah email atau NIK sudah terdaftar. Ada juga AC yang belum menetapkan hal itu:
- US-101 AC3 mengarahkan pendaftar untuk masuk atau mengatur ulang password bila emailnya sudah terdaftar (T-03, T7-F5);
- pesan akun terkunci belum ditetapkan (T-06, PT-44);
- KU-10 menampilkan pesan "akun nonaktif" saat masuk (T7-F5);
- ubah email ke alamat yang sudah dipakai belum diatur (T-13);
- US-106 AC4 menolak NIK yang sudah dipakai pemilik akun lain (T-16), dan US-201 AC7 memakai alasan "NIK sudah dipakai" (T-18).

Sebaliknya, US-103 AC2 dan AC4 sudah sengaja netral `[DOK: 03-prd.md]`. ADR-011 butir 4 mewajibkan pesan galat login dan atur ulang yang umum, tetapi menyerahkan pendaftaran ke US-101 AC3 `[DOK: adr/ADR-011-target-asvs-autentikasi-sesi-mfa.md]`. Enumerasi lewat pendaftaran tercatat sebagai celah terbuka M-12 `[DOK: 05-asvs-checklist.md]` (V6).

### 2.2 Posisi sumber

| Sumber | Isi yang relevan |
| --- | --- |
| OWASP Authentication Cheat Sheet | Pesan galat umum wajib dipakai, baik bila user ID atau password salah, akun tidak ada, maupun akun terkunci atau dinonaktifkan. Fitur pendaftaran juga perlu diperhatikan. Fitur lupa password tetap dapat dipakai walaupun akun terkunci `[SRC-261]` |
| Keycloak brute force detection | Saat akun terkunci sementara, Keycloak menampilkan pesan bawaan `Invalid username or password` supaya penyerang tidak tahu akun itu dinonaktifkan `[SRC-262]` |
| ASVS 5.0.0 6.3.8 | "Verify that valid users cannot be deduced from failed authentication challenges"; perlindungan yang sama berlaku untuk pendaftaran dan lupa password. Butir ini **Level 3**, sehingga tidak diwajibkan oleh target Level 2 di ADR-011 `[SRC-265]` |
| ASVS 5.0.0 6.3.4 (L2) | Tidak boleh ada jalur autentikasi yang tidak terdokumentasi ("there are no undocumented pathways") `[SRC-265]`. Halaman pendaftaran bawaan Keycloak adalah jalur kedua di luar kontrol UP-73 bila "User Registration" aktif `[SRC-274]` |
| OWASP Forgot Password Cheat Sheet | Pesan yang sama untuk akun yang ada maupun tidak; respons dalam waktu yang konsisten; batas laju per akun `[SRC-269]` |
| ADR-011 butir 4 | "Pesan galat login dan atur ulang password bersifat umum" `[DOK: adr/ADR-011-target-asvs-autentikasi-sesi-mfa.md]` |

Artinya, pencegahan enumerasi adalah **keputusan penerimaan risiko**, bukan kewajiban dari target ASVS proyek ini. Karena itu butir ini dibawa ke user sebagai keputusan produk dan keamanan, setelah direview security.

### 2.3 Prinsip usulan (default)

- **P1. Jalur tanpa login** (pendaftaran, masuk, terkunci, akun nonaktif saat masuk, atur ulang password): respons ke layar sama, terdaftar atau tidak. Semua kegagalan login (password salah, akun tidak ada, terkunci, nonaktif) memakai **satu pesan seragam yang identik kata per kata**, tanpa kalimat tambahan untuk kondisi tertentu (ADR-011 butir 4). Informasi khusus tentang akun hanya dikirim ke kotak masuk email pemilik.
- **P2. Jalur yang sudah login** (pemegang sesi yang sah):
  - Status akun miliknya sendiri boleh ditampilkan, karena pemegang sesi sudah membuktikan diri. Contohnya layar "Akun nonaktif" di guideline §6 `[DOK: 06-design-guideline.md]`.
  - Sinyal tentang akun atau NIK **orang lain** diminimalkan dengan pesan umum, batas laju, dan audit, karena penyerang harus lebih dulu punya akun dengan email terverifikasi.

### 2.4 Perilaku per alur

| # | Alur (AC terdampak) | Perilaku sekarang | Opsi A: pertahankan, risiko diterima dan dicatat | Opsi B: usulan default | Dampak opsi B |
| --- | --- | --- | --- | --- | --- |
| 1 | Pendaftaran dengan email terdaftar (US-101 AC3; T-03, T7-F5) | Pesan "masuk atau atur ulang password" | Tetap; M-12 dicatat sebagai risiko diterima | Layar sama dengan pendaftaran berhasil ("Cek email Anda untuk melanjutkan"). Bila email belum terdaftar, akun dibuat seperti AC1. Bila sudah terdaftar, akun baru tidak dibuat dan email "Anda sudah punya akun; masuk atau atur ulang password" dikirim ke alamat itu. Kontrol pendukungnya ada di §2.7 (halaman pendaftaran Keycloak mati, kompensasi aman, batas laju per alamat tujuan, penyamaan waktu) | Satu templat email tambahan, cabang logika, dan penyamaan waktu respons di F1-T50 (estimasi penyusun ±2–4 jam-task). Pendaftar sah yang lupa sudah punya akun dipandu lewat email, bukan layar |
| 2 | Login gagal dan akun terkunci (US-103 AC3; T-06, PT-44, UP-70 a) | Pesan belum ditetapkan | Pesan khusus "akun terkunci" (membuka status akun). **Memerlukan revisi ADR-011 butir 4** | Pesan seragam login gagal (US-103 AC2 rumusan ulang, §2.5), sesuai perilaku bawaan Keycloak `[SRC-262]` `[SRC-273]`. Opsi ini menegaskan ADR-011 butir 4 | Tanpa kode tambahan. Pengguna sah yang terkunci tidak tahu sebabnya, dan dibantu butir 3 serta kalimat petunjuk di pesan seragam |
| 3 | Atur ulang password saat terkunci (UP-70 b) | Tidak diatur | — | Tetap dapat dipakai saat terkunci `[SRC-261]`. Perilaku Keycloak diverifikasi di F1-T55 | Tidak ada |
| 4 | Masuk dengan akun nonaktif (bagian "masuk" KU-10; T7-F5) | Pesan "akun nonaktif" | Tetap. **Memerlukan revisi ADR-011 butir 4** | Pesan seragam login gagal yang **identik** dengan butir 2 dan dengan password salah atau akun tidak ada (US-103 AC2 rumusan ulang, §2.5). Kalimat petunjuk Lupa password dan Layanan Pelanggan adalah bagian dari pesan seragam itu, sehingga tampil di semua kegagalan login, bukan hanya untuk akun nonaktif. Pesan bawaan Keycloak untuk akun nonaktif ditimpa lewat message bundle tema `[SRC-233]` (F1-T35). Apakah Keycloak memeriksa password sebelum menampilkan pesan akun nonaktif `[TIDAK DIKETAHUI]`, diverifikasi di F1-T55 | Pemilik akun nonaktif tahu statusnya lewat Layanan Pelanggan, bukan dari layar login. Opsi ini menegaskan ADR-011 butir 4 |
| 5 | Sesi terbuka saat akun dinonaktifkan (bagian "sesi" KU-10) | Pesan "akun nonaktif" | — | Tidak berubah. Pemegang sesi sudah terautentikasi (P2) | Butuh kode respons khusus untuk "akun nonaktif", karena `4010001` juga dipakai untuk sesi berakhir `[DOK: 05-response-codes.md]`. Kodenya didaftarkan di F1-T33 sesuai DoD (§5 S-05) |
| 6 | Batas laju atur ulang password per email (US-103 AC4; T-08) | Tidak diatur | Penolakan 429 per email (membuka keberadaan akun) | Melewati batas per email tidak mengirim email, dan pesan tetap sama dengan AC4. Batas per alamat IP dijawab HTTP 429 oleh proxy di depan Keycloak atau halaman galat bertema, karena halaman lupa password adalah halaman HTML Keycloak, bukan endpoint envelope API, sehingga kode `4290001` tidak berlaku di jalur ini `[KEPUTUSAN-192]` | Tidak ada sinyal per akun |
| 7 | Ubah email ke alamat yang sudah dipakai akun lain (US-105 AC4; T-13) | Tidak diatur | Pesan "email sudah dipakai" ke pemohon | Respons ke pemohon sama dengan perubahan biasa. Alamat tujuan menerima email "alamat ini sudah terdaftar; perubahan tidak dapat dilakukan", dan email akun pemohon tidak berubah. Email itu dibatasi per alamat tujuan (§2.7). Jalur ubah email lewat Keycloak ditutup | Satu templat email tambahan di F1-T57 |
| 8 | NIK yang sudah disetujui sebagai pemilik akun lain (US-106 AC4, T-16; US-201 AC7, T-18) | Pesan menyebut NIK sudah terdaftar di akun lain | Tetap | Pengajuan ditolak dini dengan pesan umum: "NIK ini tidak dapat dipakai untuk akun Anda. Hubungi Layanan Pelanggan bila ini NIK Anda." Pemeriksaan registri hanya dilakukan saat pengajuan lengkap (NIK dan foto), bukan saat NIK diketik. Batasnya berlapis (§2.7 butir 5): per akun, **per NIK lintas akun**, per alamat IP, dan ambang global dengan peringatan. Alasan penolakan Verifikator (T-18) memakai teks yang sama. Angka batas adalah bagian dari keputusan UP-73 ini, bukan keputusan teknis tersendiri | **Sisa risiko**: pesan yang berbeda dari pengajuan berhasil tetap memberi sinyal. Sinyal itu hanya bisa diperoleh lewat akun berpassword dengan email terverifikasi, dan dibatasi juga per NIK, sehingga tidak dapat dilipatgandakan dengan banyak akun. Alternatif tanpa sinyal dini ada di baris 8a |
| 8a | Alternatif butir 8 | — | — | (Opsi C, bukan default) Pengajuan diterima masuk antrean. Verifikator melihat penanda konflik dari registri NIK lalu menolak dengan alasan umum "Data identitas tidak dapat diverifikasi; hubungi Layanan Pelanggan" | Sinyal tertunda dan dibatasi kapasitas manusia, tetapi Verifikator membuka data identitas tambahan (E-05) dan pemilik NIK yang sah menunggu lebih lama |
| 9 | Email pemberitahuan ke pemilik saat akun terkunci (alternatif UP-70) | — | — | Tidak dibangun di rilis 1. Sumber yang dibuka tidak menyebutnya `[SRC-261]` `[SRC-262]`, dan pemberitahuan upaya masuk mencurigakan adalah butir Level 3 (6.3.5) `[SRC-265]` | Bisa ditambahkan lewat change request |

### 2.5 Usulan teks AC bila opsi B dipilih

- **US-101 AC3 (rumusan ulang).** Given email sudah terdaftar, When pasien mendaftar dengan email itu, Then akun baru tidak dibuat, layar yang tampil sama dengan pendaftaran berhasil, dan email berisi petunjuk masuk atau atur ulang password dikirim ke alamat itu. Kode dan isi respons API tidak dapat dibedakan dari pendaftaran baru. Email dikirim lewat antrean job, dan waktu respons dipadankan sesuai §2.7 butir 4. Pendaftaran ini tidak mengubah akun yang ada dan tidak menulis bukti persetujuan baru (§2.7 butir 2).
- **US-103 AC2 (rumusan ulang): pesan seragam login gagal.** Given masuk gagal karena email atau password salah, akun tidak ada, akun terkunci sementara (AC3), atau akun nonaktif (KU-10), When pasien masuk, Then tampil **satu pesan seragam yang identik kata per kata** untuk keempat kondisi. Pesan itu tidak menyebut bagian mana yang salah dan memuat petunjuk tetap "Bila tidak dapat masuk, gunakan Lupa password atau hubungi Layanan Pelanggan". Event login gagal tercatat (E-02).
- **US-103 AC3 (tambahan kalimat).** "... dan pesan yang tampil adalah pesan seragam AC2. Atur ulang password tetap dapat dipakai saat akun terkunci."
- **KU-10 (rumusan ulang).** Given akun pasien, tenaga kesehatan, rekanan, atau admin berstatus nonaktif:
  - When pemiliknya masuk, Then akses ditolak dengan pesan seragam login gagal (US-103 AC2), tanpa kalimat atau tanda tambahan apa pun;
  - When pemiliknya memakai sesi yang masih terbuka, Then permintaan berikutnya ditolak (§3 T-09) dan aplikasi menampilkan layar "Akun nonaktif" dengan kontak Layanan Pelanggan.
- **Uji (F1-T55).** Teks layar keempat kondisi (password salah, akun tidak ada, terkunci, nonaktif) dibandingkan secara persis dan harus identik, di semua realm yang memakai tema: `pasien`, `nakes`, `admin`, dan `rme`.
- **US-105 AC4e** dan **US-106 AC4**: lihat §3 T-13 dan T-16.

### 2.6 Review dan hubungan dengan ADR-011

- Opsi B (default) untuk butir 2 dan 4 **menegaskan** ADR-011 butir 4. Opsi A untuk butir itu **merevisi** ADR-011 butir 4 dan, bila dipilih, dicatat sebagai perubahan ADR yang diterima.
- Bila user memilih opsi A untuk alur mana pun, risikonya dicatat sebagai risiko diterima di `docs/04-risk-register.md` (M-12) beserta pemiliknya.
- Keputusan UP-73 juga berlaku untuk pendaftaran tenaga kesehatan (US-202, realm `nakes`, Fase 2), yang memakai pola pendaftaran yang sama (ADR-011 butir 6). Dengan begitu klarifikasi Fase 2 tidak perlu mengulang keputusan ini.
- Review keamanan putaran 1 ada di `docs/review/tahap-07-security.md` (B-01 diperbaiki di §2.3, §2.4 butir 4, dan §2.5).

### 2.7 Kontrol teknis pendukung UP-73

Kontrol ini berlaku bila opsi B dipilih, dan dicatat sebagai keputusan teknis.

1. **Jalur Keycloak yang melewati UP-73 ditutup** lewat konfigurasi realm sebagai kode di F1-T31, dan diuji di CI:
   - "User Registration" OFF di realm `pasien`, `nakes`, `admin`, dan `rme`, karena bila aktif, tautan Register muncul di halaman login `[SRC-274]` dan menjadi jalur yang tidak terdokumentasi (6.3.4 L2) `[SRC-265]`;
   - pengguna tidak dapat mengubah email lewat Keycloak (ADR-011 butir 6).
   `[KEPUTUSAN-201]`
2. **Kompensasi pendaftaran tidak menyentuh akun yang sudah ada.**
   - Pendaftaran dengan email terdaftar tidak mengubah, menghapus, atau mengganti kredensial user Keycloak maupun akun aplikasi yang ada, dan tidak menulis bukti persetujuan baru.
   - Kompensasi hanya menghapus user berdasarkan ID yang dibuat oleh permintaan itu sendiri, bukan berdasarkan email.
   `[KEPUTUSAN-201]`
3. **Batas laju email "Anda sudah punya akun" dan "alamat ini sudah terdaftar"** per alamat tujuan (alamat dinormalisasi huruf kecil): 1 kali per 60 detik dan 5 kali per 60 menit, diam (tanpa respons berbeda), ditambah batas per alamat IP. Tujuannya mencegah kotak masuk korban dibanjiri dan kuota SES terkuras `[KEPUTUSAN-192]`.
4. **Penyamaan waktu respons pendaftaran.**
   - Untuk email baru maupun email terdaftar, backend menahan respons sampai batas waktu tetap 900 ms sejak permintaan diterima. Batas ini dipilih di bawah target NFR-PERF-02 (p95 ≤ 1 detik). Bila jalur pembuatan akun melebihi 900 ms, selisih yang tersisa tetap terlihat.
   - Pembuatan akun secara asinkron lewat antrean job **ditolak**, karena password harus ikut tersimpan di muatan job, padahal password teks asli tidak boleh disimpan at rest di tabel job (larangan mencatat password di ADR-005 berlaku untuk log; muatan job adalah penyimpanan, sehingga dasar penolakannya larangan menyimpan password teks asli, catatan review keamanan putaran 2).
   - Cara verifikasi di Tahap 8 (F1-T50): test mengukur sebaran waktu respons untuk kedua kasus, dengan minimal 100 permintaan per kasus di lingkungan test yang sama, lalu membandingkan median dan p95.
   - Selisih yang tidak dapat dihilangkan dicatat sebagai risiko sisa, karena 6.3.8 berlevel L3 `[SRC-265]` dan Forgot Password Cheat Sheet menganjurkan "consistent amount of time" `[SRC-269]`.
   `[KEPUTUSAN-201]`
5. **Batas NIK berlapis** (bagian dari keputusan bisnis UP-73 butir 8, bukan keputusan teknis tersendiri):
   - per akun: 3 penolakan `nik_unavailable` per 24 jam, lalu pengajuan NIK akun itu ditahan 24 jam;
   - **per NIK lintas akun**: 3 penolakan untuk NIK yang sama dari akun mana pun per 24 jam, lalu pengajuan NIK itu dari akun mana pun ditolak dengan pesan umum yang sama selama 24 jam;
   - per alamat IP: 10 penolakan per 24 jam;
   - ambang global: lebih dari 20 penolakan per jam di seluruh sistem memicu peringatan di security log.
   Setiap penahanan tercatat sebagai kejadian keamanan `[KEPUTUSAN-192]`.
6. **Batas laju login per alamat IP** terhadap credential stuffing, di proxy depan Keycloak. Penguncian Keycloak bersifat per akun, sehingga tidak menahan satu percobaan untuk banyak akun.
   - Angka usulan: 60 percobaan login per 10 menit per IP, dijawab HTTP 429 oleh proxy.
   - Risiko pengguna sah di balik satu IP publik (NAT operator seluler) diuji sebelum production.
   - Mekanismenya `[TIDAK DIKETAHUI]` (S-03).
   `[KEPUTUSAN-192]`

---

## 3. Klarifikasi per temuan (Fase 1)

Format setiap butir: story/AC terdampak, masalah, usulan teks AC (tambahan atau rumusan ulang, Given/When/Then), jenis, sumber, dan task Fase 1 yang menyesuaikan setelah disetujui.

### T-01 · US-101 AC2 · Teknis · T1

**Masalah.** Masa berlaku tautan verifikasi email belum ditetapkan, begitu pula perlakuan tautan lama setelah kirim ulang.

**Usulan AC.**
- **AC2 (rumusan ulang).** Given tautan verifikasi dibuka paling lambat 24 jam sejak tautan itu dikirim dan belum pernah dipakai, When pasien membukanya, Then email berstatus terverifikasi dan pasien dapat masuk.
  - Given tautan sudah lewat 24 jam atau sudah dipakai, Then status email tidak berubah dan pasien diminta mengirim ulang tautan.
  - Given email akun sudah terverifikasi, When tautan verifikasi lain untuk akun itu dibuka, Then tidak ada perubahan dan pasien diarahkan ke halaman masuk.
- **AC2b (baru).** Given pasien meminta kirim ulang tautan untuk sebuah email, When permintaan dikirim, Then respons sama, terdaftar atau tidak (UP-73 butir 1).
  - Email hanya dikirim bila akun ada dan belum terverifikasi, paling banyak 1 kali per 60 detik dan 5 kali per 60 menit per email. Batas dihitung per alamat yang dinormalisasi ke huruf kecil. Permintaan di atas batas itu tidak mengirim email, tanpa respons yang berbeda.
  - Lebih dari 30 permintaan per 60 menit dari satu alamat IP dijawab `4290001`. Kode ini berlaku karena kirim ulang adalah endpoint backend yang memakai envelope API.
  - Tautan yang terkirim lebih dulu tidak dijamin batal sebelum kedaluwarsa, karena dikirim ke alamat yang sama.

**Sumber.** Verifikasi email wajib `[DOK: adr/ADR-011-target-asvs-autentikasi-sesi-mfa.md]` (butir 4). Masa berlaku 24 jam dan sifat tautan `[KEPUTUSAN-187]`; batas laju `[KEPUTUSAN-192]`. Mekanisme token bergantung pada ADR jalur email (F1-T02) `[KEPUTUSAN-183]`. Kode `4290001` `[DOK: 05-response-codes.md]`.
**Task.** F1-T51, F1-T53, F1-T36.

### T-02 · US-101 AC1, NFR-SEC-03 · Teknis · T1

**Masalah.** "Memenuhi kebijakan NFR-SEC-03" merujuk ke NFR-SEC-01 tanpa angka. Checklist ASVS juga mencatat nilai kebijakan password sebagai "Tertunda Tahap 8 (menunggu teks bab)" `[DOK: 05-asvs-checklist.md]`. Teks bab V6 sekarang sudah dibuka `[SRC-265]`.

**Usulan NFR-SEC-03 (rumusan ulang).** Password yang dibuat atau diubah pengguna:
1. Panjangnya 8–128 karakter (6.2.1 L1; 6.2.9 L2 "at least 64 characters are permitted").
2. Semua karakter diizinkan, termasuk spasi, tanpa aturan jumlah huruf besar, angka, atau simbol (6.2.5 L1).
3. Diperiksa persis seperti dimasukkan, tanpa dipangkas dan tanpa perubahan huruf (6.2.8 L1).
4. Ditolak bila:
   - ada di daftar minimal 3.000 kata sandi paling umum (6.2.4 L1) atau di daftar kata sandi bocor (6.2.12 L2). Kedua daftar berupa **berkas lokal** yang dipakai kebijakan "Password blacklist" Keycloak (pencocokan tanpa membedakan huruf besar-kecil, Bloom filter, positif palsu bawaan 0,01%) `[SRC-270]`. Tidak ada pemeriksaan ke layanan kata sandi bocor eksternal, karena itu melanggar ketentuan penyedia di Indonesia (CR-007, T4-Q1);
   - termasuk daftar kata konteks (6.2.11 L2) yang dimuat di berkas daftar tolak yang sama: "ruang sehat", "ruangsehat", "rumah sehat", "rumahsehat" (nama lama, CR-016), "arunika", beserta variasi umumnya yang disusun di Tahap 8;
   - sama dengan email pengguna (kebijakan "Not email"). Kebijakan ini hanya memeriksa kesamaan, bukan apakah password memuat email `[SRC-270]`.

   Aturan "memuat bagian lokal email" dari versi 0.1 **dihapus**. Keycloak tidak menyediakannya secara bawaan `[SRC-270]`, sehingga aturan itu tidak bisa berlaku sama di semua jalur. Kebijakan password harus sama di pendaftaran backend, halaman atur ulang Keycloak, aktivasi admin, dan ubah password (6.1.3, 6.3.4 L2) `[SRC-265]`. Kebijakan kustom (SPI) adalah pola baru yang memerlukan ADR, sehingga tidak diambil di sini.
5. Tidak ada kewajiban mengganti password secara berkala (6.2.10 L2).
6. Fitur tempel dan pengelola password diizinkan (6.2.7 L1).
7. Isian bermasker, dengan opsi menampilkan password (6.2.6 L1).

Aturan penguncian (15 menit setelah 5 gagal) tetap. Batas atas 128 karakter membatasi biaya hash Argon2 per permintaan.

**Usulan AC.**
- **AC1a (baru).** Given password tidak memenuhi NFR-SEC-03, When formulir dikirim, Then pendaftaran ditolak, pesan menyebut aturan yang dilanggar (panjang, kata sandi umum atau bocor, kata konteks), dan isian lain tidak hilang (KU-01).
- Rumusan AC1a juga berlaku untuk atur ulang password (US-103 AC4b), aktivasi akun admin (US-208), dan ubah password (US-103 AC6 usulan di bawah).

**Ubah password saat login (6.2.2 dan 6.2.3, L1) `[SRC-265]`.** PRD belum punya AC untuk pengguna yang sudah login mengubah passwordnya. Butir ini menambah kemampuan, jadi memerlukan change request kecil. CR dicatat di `docs/backlog.md` bila disetujui di Checkpoint 2 (pertanyaan 12 e).
- **US-103 AC6 (usulan baru).** Given pasien sudah login, When ia mengubah password, Then:
  - password lama dan password baru wajib diisi;
  - autentikasi ulang berlaku (ADR-011 butir 9, `[KEPUTUSAN-195]`);
  - password baru mengikuti NFR-SEC-03;
  - opsi "keluar dari perangkat lain" tampil dan tercentang secara bawaan `[KEPUTUSAN-198]`;
  - event E-04 tercatat tanpa nilai kredensial.
- Berlaku juga untuk realm `nakes`, `admin`, dan `rme`. Mekanisme Keycloak (required action atau konsol akun) `[TIDAK DIKETAHUI]`, diverifikasi di Tahap 8.

**Belum diketahui.** Daftar kata sandi bocor yang dipakai beserta lisensinya, dan cara Keycloak menghitung panjang karakter non-ASCII `[TIDAK DIKETAHUI]`. Keduanya diverifikasi di F1-T31 dan dicatat di `docs/sources.md`, sebelum dipakai.

**Sumber.** `[SRC-265]` (butir yang disebut di atas), `[SRC-270]`, `[KEPUTUSAN-188]`; kebijakan mengikuti ASVS Level 2 `[DOK: adr/ADR-011-target-asvs-autentikasi-sesi-mfa.md]` (butir 4).
**Task.** F1-T31, F1-T35, F1-T50, F1-T52.

### T-03 · US-101 AC3 · Bisnis/produk · T1

Masuk keputusan terpadu UP-73 §2.4 butir 1. Usulan default opsi B, dengan teks AC di §2.5.

### T-04 · US-101 AC1 · Teknis · T1

**Masalah.** AC belum mengatur dua kondisi gagal: sebagian langkah pembuatan akun gagal, atau email verifikasi gagal terkirim. Kompensasinya baru ada di ADR-011 butir 6 dan coding standards §7.3.

**Usulan AC.**
- **AC1b (baru).** Given salah satu langkah pembuatan akun gagal (user Keycloak, akun aplikasi, atau bukti persetujuan), When pendaftaran diproses, Then:
  - tidak ada akun setengah jadi, dan user Keycloak yang dibuat oleh permintaan itu dihapus kembali berdasarkan ID-nya, bukan berdasarkan email;
  - user Keycloak, akun aplikasi, dan bukti persetujuan milik akun lain yang sudah ada tidak berubah (§2.7 butir 2);
  - pasien melihat pesan gagal dengan tombol coba lagi (KU-02);
  - setelah kompensasi selesai, pendaftaran ulang dengan email yang sama berhasil;
  - bila penghapusan kompensasi gagal, penghapusan diulang dari antrean job sampai berhasil dan memicu entri security log;
  - event E-04 dengan hasil gagal dicoba dicatat (KU-06).
- **AC1c (baru).** Given akun berhasil dibuat tetapi email verifikasi gagal terkirim, Then:
  - akun tetap berstatus "email belum diverifikasi" dan pendaftaran tidak dibatalkan;
  - pengiriman dicoba ulang otomatis dari antrean job;
  - pasien tetap dapat meminta kirim ulang (AC2b).

**Sumber.** Kompensasi `[DOK: adr/ADR-011-target-asvs-autentikasi-sesi-mfa.md]` (butir 6) dan `[DOK: coding-standards.md]` (§7.3). Pengulangan lewat job `[KEPUTUSAN-189]`; kompensasi berdasarkan ID `[KEPUTUSAN-201]`. Antrean job di PostgreSQL `[DOK: 04-data-dictionary.md]` (§0.10).
**Task.** F1-T50, F1-T36.

### T-05 · US-103 AC1 · Teknis · T1

**Masalah.** Perilaku masuk dengan email yang belum terverifikasi belum ditetapkan.

**Usulan AC.**
- **AC1a (baru).** Given email dan password benar tetapi email belum terverifikasi, When pasien masuk, Then:
  - pasien tidak masuk ke beranda;
  - layar meminta verifikasi email, dengan tombol kirim ulang tautan (US-101 AC2b);
  - event login tercatat sesuai pemetaan event Keycloak (ADR-011 butir 10).
- Given password salah, Then berlaku AC2, tanpa petunjuk status verifikasi.

Petunjuk "email belum diverifikasi" hanya muncul setelah password benar, sehingga tidak membuka keberadaan akun (P1, §2.3). Urutan pemeriksaan di Keycloak (required action setelah password diterima) diverifikasi di F1-T54.

**Sumber.** Verifikasi email wajib sebelum akun dipakai `[DOK: adr/ADR-011-target-asvs-autentikasi-sesi-mfa.md]` (butir 4); required action Verify Email `[SRC-70]`.
**Task.** F1-T54, F1-T55, F1-T35.

### T-06 · US-103 AC3 · Campuran (waktu: Teknis; pesan: Bisnis) · T1

**Masalah.** Titik awal 15 menit tidak jelas, arti "berturut-turut" tidak terukur, dan pesan akun terkunci menunggu PT-44.

**Usulan AC3 (rumusan ulang).** Given 5 percobaan masuk gagal berturut-turut untuk satu akun, When percobaan berikutnya dilakukan, Then akun terkunci sementara selama 15 menit sejak percobaan gagal ke-5, walaupun password benar.
- "Berturut-turut" berarti tanpa login berhasil di antaranya, dan setiap kegagalan terjadi tidak lebih dari 12 jam sejak kegagalan sebelumnya.
- Percobaan selama akun terkunci tidak menambah hitungan dan tidak memperpanjang penguncian.
- Setelah 15 menit, password benar dapat dipakai masuk dan hitungan kembali nol.
- Lama penguncian tetap 15 menit juga untuk penguncian kedua dan seterusnya. Keycloak dapat memperpanjang masa kunci selama serangan berlanjut `[SRC-273]`, sehingga Wait Increment dan Max Wait sama-sama diatur 15 menit dengan strategi linear. Test F1-T55 mencakup 5 kegagalan baru setelah penguncian pertama berakhir.
- Pesan yang tampil adalah pesan seragam AC2 (UP-73 butir 2), dan atur ulang password tetap tersedia (butir 3).
- Percobaan yang lebih cepat dari 1 detik dapat memicu penundaan 1 menit (perlindungan otomasi bawaan Keycloak), dan test F1-T55 memberi jeda lebih dari 1 detik antarpercobaan.

**Sumber.**
- Nilai 5 gagal dan 15 menit `[DOK: adr/ADR-011-target-asvs-autentikasi-sesi-mfa.md]` (butir 4) `[KEPUTUSAN-06]`.
- Perilaku Keycloak `[SRC-262]` (dibuka ulang 2026-11-18): "Failure Reset Time" bawaan 12 jam dan dihitung dari kegagalan terakhir; hitungan tidak bertambah selama akun terkunci sementara; Quick Login Check bawaan 1000 ms dengan penundaan minimal 1 menit.
- Masa kunci bertambah selama serangan berlanjut, dengan Max Wait dan strategi kelipatan atau linear `[SRC-273]`.
- Pilihan parameter `[KEPUTUSAN-190]`.
- **Belum diketahui:** apakah percobaan selama terkunci memperpanjang penguncian `[TIDAK DIKETAHUI]`. Dokumentasi tidak menyebutnya `[SRC-262]`, jadi perilakunya diuji di F1-T55. Bila Keycloak memperpanjang, rumusan AC disesuaikan.
- Pesan: UP-73.

**Task.** F1-T31, F1-T35, F1-T55.

### T-07 · NFR-SEC-08, US-106 AC3 · Teknis · T1 (berkas identitas)

**Masalah.** "5 MB" dan "10 MB" tidak menyatakan apakah 10^6 atau 2^20 byte.

**Usulan.**
- **NFR-SEC-08:** foto JPG/PNG paling besar 5 MiB (5.242.880 byte); dokumen PDF paling besar 10 MiB (10.485.760 byte). Batas berlaku untuk berkas seperti diunggah, sebelum dikode ulang.
- **US-106 AC3 (tambahan):**
  - Given berkas berukuran tepat 5.242.880 byte dan bertipe JPG/PNG menurut isinya, Then diterima.
  - Given berkas berukuran 5.242.881 byte, Then ditolak dengan `4130001`.
  - Given jenis menurut isi bukan JPG/PNG (termasuk ekstensi palsu), Then ditolak dengan `4150001`.
  - Pesan menyebut jenis dan ukuran yang diizinkan.

**Sumber.** `[KEPUTUSAN-08]`, satuan `[KEPUTUSAN-191]`, kode respons `[DOK: 05-response-codes.md]` `[KEPUTUSAN-84]`, validasi jenis dari isi `[DOK: 05-asvs-checklist.md]` (V5).
**Task.** F1-T29, F1-T30, F1-T60.

### T-08 · US-103 AC4 · Teknis · T1

**Masalah.** AC belum mengatur tautan kedaluwarsa atau sudah dipakai, password baru yang melanggar kebijakan, dan batas laju permintaan.

**Usulan AC.**
- **AC4a.** Given tautan atur ulang sudah lewat 60 menit sejak dikirim atau sudah dipakai, When dibuka, Then password tidak dapat diubah dan pasien diminta meminta tautan baru.
- **AC4b.** Given password baru tidak memenuhi NFR-SEC-03, When disimpan, Then ditolak dengan pesan aturan yang dilanggar, dan pasien dapat mencoba lagi selama tautan masih berlaku.
- **AC4c.** Given lebih dari 5 permintaan atur ulang untuk email yang sama (dinormalisasi huruf kecil) dalam 60 menit, Then permintaan berikutnya tidak mengirim email dan pesan tetap sama dengan AC4 (UP-73 butir 6). Given lebih dari 30 permintaan per 60 menit dari satu alamat IP, Then permintaan dijawab HTTP 429 oleh proxy di depan Keycloak atau ditampilkan halaman galat bertema. Kode `4290001` tidak dipakai di jalur ini, karena halaman lupa password adalah halaman HTML Keycloak, bukan endpoint envelope API (ADR-023).
- **AC4d.** Given password berhasil diatur ulang, Then:
  - event E-04 tercatat tanpa nilai kredensial (ADR-005), dan password lama tidak dapat dipakai;
  - **semua sesi pengguna diakhiri**, yaitu sesi Keycloak lewat Admin REST API (logout user) `[SRC-72]` ditambah catatan `session_revocation` di aplikasi (ADR-011 butir 8). Sesi berakhir paling lambat 2 menit setelah atur ulang berhasil, karena event Keycloak diambil setiap menit (ADR-011 butir 10). Jeda nyatanya diukur di F1-T54;
  - pengguna tidak otomatis masuk, dan masuk lewat alur biasa;
  - email pemberitahuan "password Anda telah diatur ulang", beserta kontak Layanan Pelanggan, dikirim ke email akun.
  `[KEPUTUSAN-198]` `[KEPUTUSAN-200]`
- **AC4e.** Given akun memakai MFA (realm `admin`, `nakes`, `rme`; juga `pasien` bila opsi S-01 M1 atau M2 dipilih), When password diatur ulang lewat tautan email, Then OTP tetap diminta sebelum password baru berlaku dan sebelum akses diberikan. Atur ulang tidak boleh melewati MFA (6.4.3 L2) `[SRC-265]` `[KEPUTUSAN-199]`.

**Belum diketahui.**
- Mekanisme batas laju di halaman lupa password Keycloak `[TIDAK DIKETAHUI]`, diverifikasi di F1-T31 (§5 S-03).
- Apakah alur Reset Credentials bawaan meminta OTP `[TIDAK DIKETAHUI]`. Dokumentasi hanya menyebut atur ulang "after email address confirmation" `[SRC-272]`, sehingga alurnya dikonfigurasi dan diuji di F1-T31/F1-T55.
- Apakah alur bawaan mengakhiri sesi lain atau langsung memasukkan pengguna `[TIDAK DIKETAHUI]`. Karena itu penghentian sesi dijalankan backend, tanpa bergantung pada perilaku bawaan.

**Sumber.** Tautan sekali pakai 60 menit `[KEPUTUSAN-06]` `[SRC-70]`; batas laju `[KEPUTUSAN-192]`; kontrol anti-otomasi wajib terdokumentasi dan diterapkan (6.1.1, 6.3.1) `[SRC-265]`; opsi mengakhiri sesi lain setelah perubahan kredensial (7.4.3 L2) `[SRC-266]`; Forgot Password Cheat Sheet "invalidate the sessions automatically", "Don't automatically log the user in", email pemberitahuan `[SRC-269]`.
**Task.** F1-T31, F1-T54, F1-T55, F1-T35.

### T-09 · US-103 AC5, KU-10 (juga US-1302 AC4 di Fase 3 dan US-207 AC1 di Fase 2) · Teknis · T1

**Masalah.** AC5 merujuk KU-10 (akun nonaktif) untuk keluar. Kata "seketika" dan "ditolak" tidak punya batas waktu yang terukur.

**Usulan AC.**
- **US-103 AC5 (rumusan ulang).** Given pasien memilih keluar, When proses keluar selesai, Then sesi di server diakhiri, dan setiap permintaan API berikutnya yang memakai token akses atau token penyegar sesi itu ditolak dengan `4010001`, walaupun token belum kedaluwarsa. Rujukan KU-10 diganti ADR-011 butir 8.
- **Definisi "berikutnya" dan "seketika"**, berlaku untuk KU-10, US-207 AC1, dan US-1302 AC4: setiap permintaan yang mulai diproses setelah aksi keluar, penonaktifan, atau perubahan peran mengembalikan respons sukses ke pelakunya.
  - Permintaan itu ditolak, atau dinilai dengan peran baru.
  - Koneksi WebSocket terbuka milik sesi itu diputus.
  - Test: aksi selesai, lalu permintaan pertama dengan token lama harus ditolak.
- **KU-10 bagian sesi:** lihat rumusan di §2.5.

**Sumber.** Penolakan seketika lewat status akun dan `session_revocation` `[DOK: adr/ADR-011-target-asvs-autentikasi-sesi-mfa.md]` (butir 8) `[DOK: 04-data-dictionary.md]` (A1.4); pemutusan WebSocket `[DOK: 05-asvs-checklist.md]` (V7, ADR-016 butir 1); definisi `[KEPUTUSAN-193]`.
**Task.** F1-T33, F1-T54. US-207 dan US-1302 menerapkannya di fasenya masing-masing.

### T-10 · US-104 AC1 · Teknis (tanpa perubahan teks AC) · T2

**Masalah.** Teks final legalitas, retensi, dan jangka waktu pemrosesan belum ada (L-08, PT-24).

**Usulan.** Teks AC tidak diubah, karena AC1 sudah menyatakan semuanya wajib terisi sebelum go-live.
- Di Fase 1, AC1 dianggap memenuhi DoR dan diverifikasi lewat strukturnya: ketujuh bagian Pasal 21 ayat (1) dan kontak tampil, dan seed versi draf bertanda "menunggu L-08/PT-24" (F1-T49).
- Pemeriksaan isi final menjadi pintu go-live (TC-US-104-2).

**Sumber.** `[DOK: 03-prd.md]` (US-104 AC1), `[DOK: 07-phase-plan.md]` (F1-T49), `[DOK: test-plan.md]` (§9 T-10).

### T-11 · US-104 AC3 (dan AC4 di Fase 2) · Bisnis/produk · T2

**Masalah.** Belum ditetapkan siapa yang menerbitkan versi teks dan butir persetujuan baru, dan bagaimana caranya.

**Opsi.**
- (a) **Migrasi data** yang direview dan dijalankan pipeline deploy, tanpa layar admin. Isi teks disetujui legal dan Petugas PDP sebelum migrasi dibuat.
- (b) Layar backoffice untuk Petugas PDP atau Super Admin.

**Usulan default (UP-75 a).** Opsi (a).
- "Tanpa mengubah aplikasi" ditafsirkan sebagai tanpa rilis baru aplikasi mobile atau web.
- Opsi ini sudah sejalan dengan seed di F1-T49 dan dengan larangan mengubah versi yang sudah terbit `[DOK: 04-data-dictionary.md]` (A3.1).

**Dampak.**
- Opsi (a) tidak menambah scope, tetapi setiap perubahan teks memerlukan siklus deploy.
- Opsi (b) menambah satu story backoffice beserta audit E-14, yang ukurannya ditaksir bila dipilih.

**Usulan AC3 (bila a).** Given legal menetapkan butir atau teks versi baru, When versi dan butirnya dimasukkan lewat migrasi data yang direview dan dijalankan pipeline (tanpa rilis baru aplikasi), Then:
- layar persetujuan menampilkan teks dan butir baru mulai `effective_from`;
- versi yang sudah terbit tidak dapat diubah, dan percobaan mengubahnya ditolak trigger.

### T-12 · US-105 AC1 · Campuran (aturan validasi: Teknis; usia pemilik akun: Bisnis) · T2 (bagian usia: T1, data anak)

**Masalah.** Aturan validasi field profil belum ada; yang ada hanya KU-01 yang generik.

**Usulan tabel validasi (AC1a baru).** Field wajib mengikuti kolom `NOT NULL` di data dictionary `[DOK: 04-data-dictionary.md]` (A2.1–A2.3). Aturan format `[KEPUTUSAN-194]`.

| Field | Wajib | Aturan |
| --- | --- | --- |
| Nama lengkap | Ya | 1–100 karakter setelah spasi di awal dan akhir dipangkas; tidak boleh hanya spasi; huruf (aksara apa pun), spasi, titik, koma, apostrof, dan tanda hubung |
| Tanggal lahir | Ya | Tanggal sah; tidak di masa depan (tanggal WIB); tidak lebih dari 120 tahun sebelum hari ini. Usia minimum pemilik akun mengikuti UP-77 |
| Jenis kelamin | Ya | Laki-laki atau Perempuan (CHECK `male`/`female`, A2.2) |
| Nomor HP (profil) | Tidak (A2.1 `phone` boleh kosong) | Spasi dan tanda hubung dibuang; awalan "+62" atau "0"; disimpan dalam bentuk +62; 8–13 digit setelah kode negara. Kepemilikan nomor tidak diverifikasi |
| Alamat: nama penerima, nomor HP penerima, alamat lengkap, provinsi, kota | Ya (A2.3) | Nama penerima seperti aturan nama; nomor HP seperti aturan di atas; alamat lengkap 5–500 karakter. Provinsi dan kota berupa teks 2–100 karakter sampai sistem kode wilayah ditetapkan (KEPUTUSAN-60, US-1401) |
| Alamat: label, kode pos, patokan | Tidak | Label ≤ 30 karakter; kode pos hanya angka; patokan ≤ 200 karakter |
| Alamat utama | — | Tepat satu alamat utama per pasien (A2.3 index) |

Setiap pelanggaran ditolak dengan `4220001` dan pesan per field (KU-01).

**Butir bisnis (UP-77): usia minimum pemilik akun.**
- **Masalah.** PRD tidak membatasi usia pendaftar. UU PDP mewajibkan persetujuan orang tua atau wali untuk data anak `[SRC-15]` (Pasal 25), dan batas usia anak masih menunggu L-08 (US-108 AC6) `[DOK: 03-prd.md]`.
- **Opsi.**
  - (a) Tanpa batas usia sampai L-08 terjawab. Risikonya, anak dapat mendaftar sendiri tanpa persetujuan orang tua.
  - (b) Parameter usia minimum pemilik akun, ditolak saat pendaftaran atau pengisian profil bila usia di bawah nilai parameter. Nilainya dari legal (L-08), atau diisi user sebelum go-live.
- **Usulan default.** Opsi (b). Di Fase 1 parameter dibuat dan diisi nilai uji.
- **Dampak.** Menambah satu parameter (pola PB) dan satu aturan validasi. Nilai final `[TIDAK DIKETAHUI]` (L-08).

**Task.** F1-T56, F1-T58.

### T-13 · US-105 AC4 · Campuran (Teknis + pesan email sudah dipakai: Bisnis, UP-73 butir 7) · T1

**Masalah.** AC belum mengatur email baru yang sudah dipakai akun lain, tautan kedaluwarsa, batas waktu, dan autentikasi ulang.

**Usulan AC.**
- **AC4a.** Given autentikasi terakhir pasien lebih dari 5 menit sebelum permintaan, When ia mengajukan perubahan email, Then ia diminta masuk ulang di Keycloak sebelum permintaan diproses. Permintaan tanpa autentikasi ulang ditolak.
- **AC4b.** Given perubahan email diajukan, Then:
  - tautan verifikasi dikirim ke email baru, berlaku 24 jam, dan sekali pakai `[KEPUTUSAN-187]`;
  - email lama tetap dipakai untuk masuk dan notifikasi sampai tautan dibuka `[KEPUTUSAN-200]`;
  - hanya tautan dari permintaan terakhir yang berlaku, dan permintaan baru membatalkan tautan sebelumnya `[KEPUTUSAN-187]`.
- **AC4c.** Given tautan kedaluwarsa, sudah dipakai, atau sudah digantikan, When dibuka, Then email tidak berubah dan pasien diminta mengajukan ulang.
- **AC4d.** Given tautan yang berlaku dibuka, Then:
  - email untuk masuk dan notifikasi berganti ke email baru di Keycloak dan di aplikasi;
  - email lama menerima pemberitahuan perubahan beserta kontak Layanan Pelanggan `[KEPUTUSAN-200]`. Pemberitahuan ini butir ASVS 6.3.7 Level 3 `[SRC-265]`, jadi dipilih sebagai praktik baik berbiaya kecil, bukan kewajiban target Level 2;
  - semua sesi lain pengguna diakhiri, karena email adalah kanal pemulihan akun `[KEPUTUSAN-198]`;
  - event E-04 tercatat.
- **AC4e (UP-73 butir 7).** Given email baru sudah dipakai akun lain, Then respons ke pasien sama dengan AC4b, alamat tujuan menerima pemberitahuan bahwa alamat itu sudah terdaftar, dan email akun pasien tidak berubah. Email pemberitahuan dibatasi per alamat tujuan (§2.7 butir 3).

Pembatalan tautan lama penting untuk keamanan. Tanpanya, tautan ke alamat yang salah ketik masih bisa dibuka penerima yang salah, dan alamat itu menjadi email akun sehingga dapat dipakai mengambil alih akun lewat atur ulang password.

**Sumber.**
- Autentikasi ulang untuk mengubah email `[DOK: adr/ADR-011-target-asvs-autentikasi-sesi-mfa.md]` (butir 9), sesuai ASVS 7.5.1 L2 `[SRC-266]`; jendela 5 menit `[KEPUTUSAN-195]`. Parameter `max_age`/`auth_time` diverifikasi di Tahap 8 `[KEPUTUSAN-85]`.
- Email hanya diubah lewat API aplikasi `[DOK: adr/ADR-011-target-asvs-autentikasi-sesi-mfa.md]` (butir 6) `[KEPUTUSAN-46]`.
- Tautan `[KEPUTUSAN-187]`. Mekanisme token bergantung pada ADR jalur email `[KEPUTUSAN-183]`.

**Task.** F1-T57, F1-T58.

### T-14 · US-106 AC1 vs US-105 AC3 · Bisnis/produk · T1

**Masalah.** US-105 AC3 menyatakan perubahan NIK dilakukan lewat US-106, tetapi US-106 AC1 hanya berlaku untuk status "Belum diunggah" dan "Ditolak".

**Opsi.**
- (a) Pasien yang sudah "Disetujui" dapat mengajukan NIK dan KTP baru. Status menjadi "Menunggu verifikasi", sesuai UP-39 yang sudah disetujui.
- (b) Perubahan NIK hanya lewat Layanan Pelanggan atau permintaan perbaikan hak subjek data (US-1603 AC5, Fase 3).
- (c) Seperti (a), tetapi status "Disetujui" lama tetap berlaku sampai keputusan baru diambil.

**Usulan default (UP-74 a).** Opsi (a).
- Sejalan dengan UP-39 dan registri NIK, yang menonaktifkan NIK lama hanya saat NIK baru disetujui `[DOK: 04-data-dictionary.md]` (A2.8).
- Opsi (c) bertentangan dengan UP-39.

**Dampak.**
- Opsi (a): pasien kehilangan akses layanan berbatas selama menunggu, sehingga perlu dialog konfirmasi sebelum mengirim.
- Opsi (b): pasien tidak dapat mengubah sendiri, dan beban Layanan Pelanggan bertambah.

**Usulan AC1 (bila a).** Given identitas berstatus "Belum diunggah", "Ditolak", atau "Disetujui", When pasien mengisi NIK dan mengunggah foto KTP, Then status menjadi "Menunggu verifikasi" dan pasien diberi tahu bahwa verifikasi dilakukan manual. Bila status sebelumnya "Disetujui":
- dialog konfirmasi lebih dulu menjelaskan bahwa layanan yang mewajibkan identitas tidak dapat dipesan sampai disetujui lagi;
- pesanan yang sudah dibayar tetap berlaku (UP-39);
- NIK lama tetap tercatat sebagai NIK pemilik akun ini sampai NIK baru disetujui.

### T-15 · US-106 AC2, AC5 · Campuran (AC2: Teknis; AC5: Bisnis) · T1

**AC2 (Teknis).** Given NIK bukan tepat 16 digit angka, When dikirim, Then ditolak dengan `4220001` dan pesan "NIK harus 16 digit angka" (KU-01). Tidak ada validasi struktur lain (misalnya kode wilayah), karena sumber yang dibuka hanya menyebut 16 digit angka. Kalimat "ditetapkan di Tahap 4 ... `[TIDAK DIKETAHUI]`" dihapus.
- **Sumber.** `[DOK: 04-data-dictionary.md]` (A2.4, "Format 16 digit divalidasi aplikasi sebelum enkripsi"), `[SRC-63]`.

**AC5 (Bisnis, UP-74 b): status setelah pasien membatalkan pengiriman.**
- **Masalah.** Data dictionary punya status teknis `cancelled` (A2.6), sedangkan AB-05 tidak punya status tampilan "Dibatalkan".
- **Opsi.**
  - (a) Kembali ke status sebelum pengajuan itu:
    - "Belum diunggah" bila belum pernah ada keputusan;
    - "Ditolak" dengan alasan terakhir bila keputusan terakhir adalah penolakan;
    - "Disetujui" dengan NIK lama bila pengajuan dibuat dari status "Disetujui" (UP-74 a).
  - (b) Selalu "Belum diunggah".
- **Usulan default.** Opsi (a). Pengajuan yang dipicu perubahan nama atau tanggal lahir (US-105 AC3, `trigger_type = profile_changed`) tidak dapat dibatalkan, karena datanya sudah berubah.
- **Dampak.** Opsi (a) menjaga status sah yang sudah ada. Proyeksi status saat batal perlu dicek DBA (§5 S-04).

**Usulan AC5 (bila a).** Given status "Menunggu verifikasi" dari pengajuan dokumen, When pasien membatalkan pengiriman, Then status kembali seperti sebelum pengajuan itu dan foto dapat diganti. Bila pembatalan dan keputusan Verifikator terjadi bersamaan, transisi yang tersimpan lebih dulu berlaku dan pihak lain diberi tahu (UP-10, US-201 AC5).

### T-16 · US-106 AC4 · Campuran (pesan: Bisnis UP-73 butir 8; status: Teknis) · T1

**Status setelah ditolak (Teknis).** Pengajuan tidak disimpan dan status identitas tidak berubah. Ini mengikuti data dictionary, karena registri NIK diperiksa saat pengajuan untuk menolak lebih awal `[DOK: 04-data-dictionary.md]` (A2.8).

**Usulan AC4 (bila opsi B UP-73).** Given NIK sudah disetujui sebagai pemilik akun di akun lain, When pasien mengirim NIK itu sebagai pemilik akun, Then:
- pengajuan tidak disimpan dan status identitas tidak berubah;
- pesan umum tampil: "NIK ini tidak dapat dipakai untuk akun Anda. Hubungi Layanan Pelanggan bila ini NIK Anda.";
- pemeriksaan registri hanya dilakukan saat pengajuan lengkap (NIK dan foto);
- batas berlapis per akun, per NIK lintas akun, per alamat IP, dan ambang global berlaku sesuai §2.7 butir 5, dan setiap penahanan tercatat sebagai kejadian keamanan.

NIK anggota keluarga tetap boleh tercatat di lebih dari satu akun (UP-40). Angka batas NIK adalah bagian dari keputusan bisnis UP-73, dan disetujui bersama UP-73, bukan lewat paket keputusan teknis.

**Sumber.** `[UP-40]`, `[KEPUTUSAN-192]` (bagian NIK bergantung pada UP-73), §2.

### T-17 · US-201 AC3 · Bisnis/produk · T2

**Masalah.** Isi dan pengelolaan daftar alasan penolakan belum didefinisikan. Tabel `reason_code` dengan konteks `identity_rejection` sudah ada `[DOK: 04-data-dictionary.md]` (A6.7).

**Usulan daftar awal (UP-75 b; isi adalah keputusan operasional user):**

| Kode | Label untuk pasien |
| --- | --- |
| `photo_unreadable` | Foto dokumen buram atau tidak terbaca |
| `document_invalid` | Dokumen bukan KTP/KK, tidak utuh, atau tidak sesuai jenis yang diminta |
| `nik_mismatch` | NIK yang diisi tidak sama dengan NIK di dokumen |
| `profile_mismatch` | Nama atau tanggal lahir di profil tidak sama dengan dokumen |
| `guardian_not_listed` | Pemilik akun tidak tercantum sebagai orang tua/wali di KK (US-201 AC4) |
| `nik_unavailable` | Teks umum UP-73 butir 8 (US-201 AC7) |
| `other` | Lainnya; keterangan wajib |

**Usulan AC3a (baru).** Given Verifikator memilih alasan "Lainnya", When keterangan kosong, Then penolakan tidak dapat disimpan. Keterangan paling panjang 500 karakter dan tampil ke pasien.

Daftar dikelola lewat migrasi data (UP-75), seperti T-11.

**Dampak.** Daftar lain atau layar pengelolaan menambah scope. Label final diselaraskan dengan guideline teks UI.

### T-18 · US-201 AC7 · Bisnis/produk · T1

**Masalah.** Status pengajuan setelah persetujuan gagal karena NIK sudah dipakai belum ditetapkan.

**Opsi.**
- (a) Persetujuan Verifikator gagal dengan pesan konflik. Verifikator lalu menolak dengan alasan `nik_unavailable` yang sudah terpilih, sehingga status menjadi "Ditolak" dan pasien dapat menghubungi Layanan Pelanggan.
- (b) Sistem otomatis menolak dengan aktor "sistem".
- (c) Status tetap "Menunggu" sampai ditangani.

**Usulan default (UP-74 c).** Opsi (a). Keputusan tetap di tangan manusia (E-07 dengan aktor Verifikator), dan constraint unik registri tetap menjadi penjaga terakhir `[DOK: 04-data-dictionary.md]` (A2.8).

**Dampak.** Opsi (b) lebih cepat tetapi memerlukan aktor sistem di E-07. Opsi (c) membuat pengajuan menggantung.

**Usulan AC7 (bila a).** Given NIK sudah disetujui sebagai pemilik akun di akun lain, When Verifikator menyetujui pengajuan pemilik akun dengan NIK yang sama, Then:
- persetujuan ditolak dengan pesan konflik ke Verifikator;
- formulir penolakan terbuka dengan alasan `nik_unavailable` terpilih;
- setelah disimpan, status menjadi "Ditolak" dengan teks umum UP-73 butir 8. Pemberitahuan ke pasien di Fase 2 bersama US-1001.

### T-19 · US-208 · Bisnis/produk · T1

**Masalah.** US-208 belum mengatur tiga hal:
- (a) pembuatan Super Admin pertama (bootstrap baru ada di F1-T65);
- (b) admin yang menonaktifkan akunnya sendiri saat masih ada Super Admin lain;
- (c) kombinasi sub-peran yang melanggar pemisahan tugas.

**Opsi dan usulan default (UP-76):**

| Butir | Opsi | Usulan default | Dampak |
| --- | --- | --- | --- |
| (a) Bootstrap | (1) Perintah satu kali lewat jalur administrasi production (bastion, kredensial dinamis OpenBao, ADR-021, ADR-025), hanya berjalan bila belum ada Super Admin; (2) seed di file migrasi | (1), dengan kontrol berikut. File migrasi tidak boleh memuat data pribadi. **Siapa Super Admin pertama dan siapa orang kedua yang menyetujuinya** `[TIDAK DIKETAHUI]`, diputuskan user (terkait PT-38). Kontrol: (i) kredensial perintah hanya terbit lewat Control Group OpenBao, sehingga butuh persetujuan orang kedua `[SRC-253]`; (ii) perintah ditolak bila ada Super Admin aktif **atau** akun Super Admin yang menunggu aktivasi; (iii) pemeriksaan dan pembuatan berjalan dalam satu transaksi berkunci, sehingga dua eksekusi bersamaan tidak menghasilkan dua akun; (iv) undangan aktivasi sekali pakai dan berlaku 24 jam (6.4.1 L1 `[SRC-265]`). Undangan yang kedaluwarsa membuat akun tetap menunggu aktivasi, dan undangan ulang dibuat lewat perintah yang sama dengan persetujuan orang kedua | Prosedur tercatat di runbook operasi; F1-T65 sudah mencakup perintahnya, ditambah konfigurasi Control Group (Tahap 8) |
| (a') Jumlah minimum | Satu Super Admin; minimal dua sebelum go-live | Minimal dua Super Admin aktif sebelum go-live (butir checklist go-live, bukan AC) | Mencegah kehilangan akses bila satu orang tidak tersedia, karena AC2 menolak pencabutan Super Admin terakhir |
| (b) Menonaktifkan akun sendiri | Ditolak; diizinkan bila masih ada Super Admin lain | Ditolak, konsisten dengan AC3 (tidak mengubah peran sendiri) dan ADR-012 butir 7 | Penonaktifan selalu dilakukan orang lain dan tercatat atas nama orang itu |
| (c) Kombinasi sub-peran | (1) Bebas (status quo AC1); (2) Auditor eksklusif; (3) pembatasan lebih luas, misalnya Keuangan dengan Verifikator | (2) Auditor tidak dapat digabung dengan sub-peran lain. Kombinasi lain diizinkan di rilis 1, dan pemisahan pengusul–penyetuju per transaksi diatur di T-34 (Fase 3) | Butuh minimal satu orang khusus Auditor. Jumlah staf admin `[TIDAK DIKETAHUI]` |
| (d) Risiko tersisa WEB-07/M-16 | (1) Dicatat sebagai risiko terbuka; (2) pemberitahuan setiap pemberian sub-peran sensitif (Super Admin, Auditor, Verifikator, Keuangan, Petugas PDP) ke semua Super Admin dan Auditor, ditambah tinjauan akses berkala; (3) larangan mencabut Auditor terakhir | Auditor eksklusif **belum menutup** WEB-07/M-16: Super Admin masih dapat membuat akun boneka, termasuk akun Auditor yang ia kendalikan. Usulan default: (2) dan (3). Verifikator yang juga pasien dapat memutuskan pengajuan identitas akunnya sendiri; risiko konflik kepentingan ini dicatat, dan larangan memutuskan pengajuan milik sendiri diusulkan sebagai AC tambahan US-201 | (2) menambah satu jenis notifikasi admin (Fase 2, bersama US-1003); (3) satu aturan domain. Tanpa (2), risiko dicatat terbuka di threat model |

**Usulan AC (bila default dipilih).**
- **AC4.** Given belum ada Super Admin aktif maupun yang menunggu aktivasi, When operator menjalankan perintah bootstrap lewat jalur administrasi production, dengan kredensial yang terbit setelah persetujuan orang kedua, dan dengan email calon Super Admin yang sudah disetujui, Then:
  - satu akun admin dengan sub-peran Super Admin dibuat berstatus menunggu aktivasi;
  - undangan aktivasi (password dan MFA) dikirim, sekali pakai, dan berlaku 24 jam;
  - E-10 dan E-11 tercatat dengan aktor "sistem (bootstrap)" beserta identitas operator dan penyetuju.
- **AC4 (lanjutan).** Given sudah ada Super Admin aktif atau yang menunggu aktivasi, When perintah bootstrap dijalankan, Then perintah ditolak tanpa perubahan. Given dua perintah dijalankan bersamaan, Then hanya satu akun yang dibuat.
- **AC5.** Given admin mencoba menonaktifkan akunnya sendiri, When disimpan, Then aksi ditolak.
- **AC6.** Given akun memegang sub-peran Auditor, When sub-peran lain ditambahkan (atau sebaliknya), Then aksi ditolak dengan pesan pemisahan tugas.
- **AC7 (bila d-3).** Given hanya tersisa satu Auditor aktif, When sub-perannya akan dicabut atau akunnya dinonaktifkan, Then aksi ditolak.

**Sumber.** `[DOK: adr/ADR-012-otorisasi-dan-isolasi-data.md]` (butir 7), `[DOK: 04-data-dictionary.md]` (A1.2), `[DOK: 07-phase-plan.md]` (F1-T65), `[UP-18]`, ADR-021 butir 3 dan ADR-025 (Control Group), threat model WEB-07/M-16 `[DOK: 04-threat-model.md]`, review keamanan C-09 dan C-10 `[DOK: review/tahap-07-security.md]`.

### T-20 · KU-02 (berlaku untuk semua story, termasuk Fase 1) · Teknis · T2

**Masalah.** "Batas waktu" respons server tidak berangka.

**Usulan KU-02 (rumusan ulang).** Given koneksi terputus, atau server tidak merespons dalam 15 detik (unggah berkas: 60 detik), When pengguna melakukan aksi, Then pesan "gagal terhubung" dan tombol coba lagi tampil, dan percobaan ulang tidak menghasilkan aksi ganda (KU-05).

Angka ini batas sisi klien. Target server tetap NFR-PERF-01/02, yaitu p95 ≤ 500 ms untuk baca dan ≤ 1 detik untuk tulis, sehingga 15 detik memberi ruang untuk jaringan seluler.

**Sumber.** `[DOK: 03-prd.md]` (§6.1), `[KEPUTUSAN-196]`.
**Task.** F1-T46 (`rs_core`), F1-T42 (BFF).

### T-21 · US-1501 AC3 (story Fase 1) · Teknis · T1

**Masalah.** "Peringatan keamanan" belum punya ambang, penerima, dan saluran.

**Usulan AC3 (tambahan).** Given penulisan event audit gagal:
- **Fase 1:** setiap kegagalan menghasilkan satu entri security log berkategori kegagalan audit, tanpa data terlarang §7.3, dan menaikkan metrik penghitung.
- **Fase 2 (setelah EN-16):** Alertmanager mengirim peringatan ke email tim teknis paling lambat 5 menit setelah kegagalan pertama.

Penundaan bagian peringatan ke Fase 2 dicatat di kolom Fase backlog setelah disetujui `[KEPUTUSAN-179]`. Penundaan ini bergantung pada persetujuan KEPUTUSAN-179, yang juga masih menunggu Checkpoint 2. Bila disetujui, definisi F1-T69 (yang masih memuat "peringatan keamanan") disesuaikan. Daftar alamat penerima `[TIDAK DIKETAHUI]`, diisi user sebelum staging. Butir ini tidak memblokir Fase 1.

**Sumber.** Peringatan lewat Alertmanager ke email tim teknis `[DOK: 05-stack.md]` (§4.3) `[KEPUTUSAN-80]`; aliran security log `[DOK: adr/ADR-010-pemisahan-tiga-jenis-log.md]`; ambang 5 menit `[KEPUTUSAN-197]`; observability di Fase 2 `[DOK: 07-phase-plan.md]` (§4).
**Task.** F1-T23, F1-T69.

### T-22 · US-1504 AC3 (story Fase 1) · Teknis · T1

**Masalah.** AC3 bukan kriteria uji, karena isinya "ditetapkan di Tahap 4". Perilaku yang dapat diuji sudah ada di ADR-009 butir 5.

**Usulan AC3 (pengganti).**
- Given satu event di `audit_event` diubah, dihapus, atau disisipkan langsung di database (di lingkungan uji, dengan trigger penolak dilewati superuser), When job verifikasi berjalan (terjadwal harian atau dipicu manual), Then ketidakcocokan terdeteksi, hasil verifikasi menyebut `seq` pertama yang tidak cocok, dan peringatan keamanan dikirim. Di Fase 1 berupa entri security log; peringatan email di Fase 2 seperti T-21.
- Given kepala rantai di jangkar WORM berbeda dengan hasil hitung ulang, Then hasilnya sama: terdeteksi dan diperingatkan.
- Given tidak ada perubahan, Then verifikasi lulus dan hasilnya tercatat.

**Sumber.** `[DOK: adr/ADR-009-mekanisme-audit-trail.md]` (butir 5); F1-T25 (jangkar lokal berupa berkas, staging memakai Object Lock NEO) `[DOK: 07-phase-plan.md]`.
**Task.** F1-T25, F1-T68.

---

## 4. Temuan T-23–T-35 (fase 2–4): fase dan tenggat

Tanpa usulan teks rinci. Tenggat adalah sebelum fase tempat story itu dikerjakan dimulai. Tanggal fase mengikuti usulan UP-65 `[DOK: 07-phase-plan.md]` (§4). Pemilik butir bisnis adalah user; butir teknis ditangani Tech Lead di putaran klarifikasi fase itu.

| Temuan | Story / kebutuhan | Fase | Tenggat penyelesaian | Jenis awal | Catatan |
| --- | --- | --- | --- | --- | --- |
| T-23 | US-205 AC2 (status integrasi dari US-1405) | 2 (US-205), 3 (US-1405) | Sebelum Fase 2 (2026-12-14) | Teknis (penempatan AC) | Terkait PT-47 (bisnis) |
| T-24 | US-801 AC1, AC5; US-804 AC2; US-812 AC4; AB-10a; AB-27 (pajak) | 2 (EN-24) | Sisa PT-28 dan perilaku "tarif belum diatur" sebelum EN-24 di awal Fase 2 (gerbang G-10) | Bisnis | Sudah di pertanyaan Checkpoint 2 nomor 10 |
| T-25 | US-1605, US-1606 (PT-24, L-06) | 3 | Sebelum Fase 3 (2027-01-11) | Bisnis/legal | Terblokir |
| T-26 | US-1406 AC2 (evaluasi penyedia) | 2 (pembayaran, video, email), 4 (kurir) | Sebelum Fase 2; bagian kurir sebelum Fase 4 (2027-01-18) | Teknis (checklist proses) | — |
| T-27 | US-1001 AC5 (lokasi data penyedia email) | 2 | Sebelum Fase 2 | Teknis (checklist dokumen) | Terkait verifikasi lokasi data SES di F1-T02 |
| T-28 | US-203 AC4, US-205 AC4–AC5, US-308 AC4, US-309 AC5, US-803 AC4 | 2 | Sebelum Fase 2 | Bisnis/legal | AC berisi `[TIDAK DIKETAHUI]` |
| T-28 | US-810 AC3, US-1307 AC3, US-1602 AC3, US-1604 AC3 | 3 | Sebelum Fase 3 | Bisnis/legal | — |
| T-28 | US-702 AC4 | 4 | Sebelum Fase 4 | Bisnis/legal | — |
| T-29 | US-808 AC3, US-806 AC1, matriks US-1001, US-801 AC3 (AC yang tertunda ke fase lain) | 2 | Sebelum Fase 2 (penandaan AC tertunda di backlog, KEPUTUSAN-179) | Teknis | — |
| T-30 | NFR-CAP-04 (alat uji beban) | 3 (EN-25) | Sebelum Fase 3; uji beban Januari 2027 | Teknis | Alat dipilih dengan cek registry dan SRC |
| T-31 | NFR-DR-02 (jam kerja dan kalender) | Februari (EN-27 uji pemulihan) | Sebelum production disiapkan (2027-02-01) | Bisnis | Jam kerja adalah fakta bisnis |
| T-32 | US-1405 AC2, NFR-INT-03 | 3 | Sebelum Fase 3 | Teknis | — |
| T-33 | US-309 AC8 (batas laju verifikasi resep) | 2 | Sebelum Fase 2 | Teknis | — |
| T-34 | US-1202 AC3 (pengusul dan penyetuju rekening berbeda) | 3 | Sebelum Fase 3 | Bisnis (pemisahan tugas) | Terkait UP-76 butir (c) |
| T-35 | US-1101 AC1, US-1103 AC1 (sanitasi konten dan lampiran) | 3 | Sebelum Fase 3 | Teknis | — |

---

## 5. Temuan sampingan (di luar T-01–T-35)

| # | Temuan | Mengapa tidak diselesaikan di sini | Usulan tindak lanjut |
| --- | --- | --- | --- |
| S-01 | MFA pasien terhadap target ASVS Level 2. Rinciannya di §5.1 | Menyangkut ADR yang sudah diterima | Pertanyaan Checkpoint 2 berlabel perubahan ADR yang sudah diterima, dengan opsi M1–M4 (§5.1) |
| S-02 | Sesi lain setelah atur ulang password atau ubah email (H-01) `[DOK: 05-asvs-checklist.md]` (V7) | **Selesai sebagai keputusan teknis.** ASVS 7.4.3 berlevel L2 `[SRC-266]`, sehingga status quo tidak memenuhi target ADR-011 | `[KEPUTUSAN-198]`: atur ulang password dan ubah email mengakhiri semua sesi lain; ubah password saat login menampilkan opsi keluar dari perangkat lain yang tercentang secara bawaan; reset MFA oleh admin mengakhiri semua sesi. Diterapkan di T-08 AC4d, T-13 AC4d, dan US-103 AC6 usulan |
| S-03 | Mekanisme batas laju di halaman lupa password Keycloak dan di depan Keycloak `[TIDAK DIKETAHUI]` | Butuh verifikasi kemampuan produk. Bila tidak tersedia, mungkin diperlukan komponen atau konfigurasi baru di stack | Diverifikasi di F1-T31. Bila butuh komponen baru, diusulkan sebagai keputusan teknis atau ADR terpisah, tidak diambil di dokumen ini |
| S-04 | Proyeksi status identitas saat pengajuan dibatalkan (`latest_verification_request_id` menunjuk permintaan berstatus `cancelled`) belum menghasilkan status tampilan UP-74 b `[DOK: 04-data-dictionary.md]` (A2.2, A2.6) | Bergantung pada keputusan UP-74 b | Dicek DBA di F1-T59. Bila data dictionary perlu diubah, dicatat sebagai change request |
| S-05 | Kode respons `4010001` dipakai untuk sesi berakhir maupun akun dinonaktifkan `[DOK: 05-response-codes.md]`, padahal aplikasi perlu membedakan layar "Akun nonaktif" (KU-10 bagian sesi) | Penambahan kode respons mengikuti DoD Tahap 8 | Kode baru didaftarkan di F1-T33 |
| S-06 | Celah ASVS Level 2 di manajemen sesi yang belum dirancang (H-01): 7.5.2, yaitu pengguna dapat melihat dan mengakhiri sesi aktifnya, dan 7.1.2, yaitu jumlah sesi paralel terdokumentasi `[SRC-266]`. Berlaku untuk semua realm | 7.5.2 menambah kemampuan yang terlihat pengguna (scope); ADR tidak menetapkan desainnya | **UP-78** (§5.2) |

### 5.1 S-01: MFA pasien terhadap ASVS 5.0.0 Level 2 (pertanyaan Checkpoint 2, perubahan ADR yang sudah diterima)

**Teks resmi.** Tiga tempat di bab V6 menyatakan MFA wajib mulai Level 2 `[SRC-265]`:
- pengantar V6.2: "From L2 onwards, multi-factor authentication mechanisms are required, where passwords may be one of those factors.";
- pengantar V6.3: "L2 applications must force the use of multi-factor authentication (MFA).";
- 6.3.3 (L2): "either a multi-factor authentication mechanism or a combination of single-factor authentication mechanisms, must be used in order to access the application". Butir yang sama menyatakan pelonggaran membutuhkan "a fully documented rationale and a comprehensive set of mitigating controls".

Frasa "combination of single-factor authentication mechanisms" tidak didefinisikan di bab V6 `[SRC-265]`. Klaim bahwa password ditambah kontrol lain sudah memenuhi 6.3.3 tidak punya dasar `[TIDAK DIKETAHUI]`.

ASVS mengizinkan fork yang menyesuaikan requirement asalkan ketertelusurannya dijaga `[SRC-267]`, dan asesor diminta memberi pendapat atas alasan pengecualian requirement `[SRC-268]`.

**Posisi sekarang.**
- ADR-011 butir 1 menargetkan Level 2 untuk semua komponen. Butir 5 dan tabel realm di butir 3 menyatakan pasien tidak diwajibkan MFA `[DOK: adr/ADR-011-target-asvs-autentikasi-sesi-mfa.md]`.
- Risiko pasien tanpa MFA sudah diterima sejak Tahap 4 (threat model APP-04) `[DOK: 04-threat-model.md]`.
- Yang keliru adalah **klaim kepatuhan Level 2** untuk realm `pasien`, bukan penerimaan risikonya.
- Opsi mana pun memerlukan revisi ADR-011 (butir 1 atau butir 3 dan 5), NFR-SEC-02, baris MFA di `docs/05-asvs-checklist.md`, dan penilaian ulang APP-04.

**Faktor kedua yang tersedia tanpa penyedia di luar Indonesia** (CR-007, `[USER] T4-Q1`):

| Faktor | Ketersediaan | Posisi ASVS | Lokasi data |
| --- | --- | --- | --- |
| TOTP (aplikasi authenticator) | Ada di Keycloak 26.8.2 `[SRC-67]`; sudah dipakai realm `nakes`, `admin`, `rme` | Faktor "something you have" `[SRC-265]` | Kode dihitung di perangkat; tanpa penyedia pihak ketiga di sisi Ruang Sehat |
| Kode pemulihan | Ada (12 kode sekali pakai) `[SRC-68]` | Lookup secret `[SRC-265]` | Tanpa pihak ketiga |
| WebAuthn (kunci keamanan, authenticator platform) | Ada: "{project_name} uses WebAuthn for two-factor authentication" `[SRC-271]` | Faktor perangkat | Kunci terikat perangkat tidak melibatkan pihak ketiga. **Passkey yang disinkronkan lewat layanan awan platform: lokasi pemrosesannya `[TIDAK DIKETAHUI]`**, dan apakah itu termasuk "penyedia" menurut T4-Q1 adalah keputusan user. Status dukungan passkey tidak disebut di halaman resmi `[SRC-271]` |
| OTP email | Tidak boleh: "Unsafe out-of-band authentication mechanisms such as e-mail and VOIP are not permitted." `[SRC-265]` | — | Sudah dilarang ADR-011 butir 5 |
| SMS | Dikecualikan ADR-011 butir 5; 6.6.1 (L2) mengizinkan PSTN dengan syarat `[SRC-265]` | — | Butuh penyedia SMS di Indonesia `[TIDAK DIKETAHUI]` |

Keycloak juga mendukung step-up berbasis tingkat autentikasi (`acr_values`, sub-flow "Conditional - Level Of Authentication") `[SRC-272]`, sehingga faktor kedua dapat diwajibkan hanya untuk aksi tertentu.

**Opsi (tidak diputuskan penyusun).** Sumber opsi: review keamanan §3.4 `[DOK: review/tahap-07-security.md]`.

| Opsi | Isi | Status L2 realm `pasien` | Dampak |
| --- | --- | --- | --- |
| M1. MFA wajib untuk pasien | TOTP wajib (alur seperti realm `nakes`) dan kode pemulihan; WebAuthn sebagai alternatif bila disetujui | Memenuhi 6.3.3 | Friksi pendaftaran dan login bagi populasi pasien (dampak adopsi `[TIDAK DIKETAHUI]`). Pasien yang kehilangan perangkat butuh pembuktian identitas setara saat pendaftaran (6.4.4 L2), sehingga beban Layanan Pelanggan bertambah dan prosedur H-05 meluas ke pasien. Revisi ADR-011 butir 3 dan 5, NFR-SEC-02, US-101, US-103 |
| M2. MFA sukarela + step-up wajib untuk aksi berisiko | Pasien boleh mengaktifkan TOTP. Ubah email, unduh salinan data, dan hak subjek data mewajibkan faktor kedua lewat step-up; pasien tanpa faktor kedua diminta mendaftarkannya saat itu | Penyimpangan 6.3.3 untuk akses dasar, wajib terdokumentasi | Melindungi salinan data dan pengambilalihan lewat ubah email. Butuh konfigurasi alur LoA dan test. Revisi ADR-011 butir 1 (pengecualian tercatat) dan butir 5 |
| M3. Tanpa MFA, penyimpangan terdokumentasi dengan kompensasi | Klaim "L2 dengan penyimpangan 6.3.3 untuk realm `pasien`", dengan alasan dan kontrol mitigasi: pemeriksaan kata sandi bocor (6.2.12), batas laju login per IP (§2.7 butir 6), penghentian sesi setelah atur ulang (KEPUTUSAN-198), autentikasi ulang aksi berisiko, dan pemberitahuan email ganti password (KEPUTUSAN-200) | Bukan L2 penuh, tetapi jujur dan tertelusur | Biaya terendah. Risiko phishing dan pemakaian ulang kredensial tetap ada. Dicatat di risk register dan DPIA (APP-04) |
| M4. Target realm `pasien` diturunkan ke L1 untuk V6 | L1 untuk autentikasi pasien, L2 untuk komponen lain | L1 | Kontrol teknis sama dengan M3, dengan label yang lebih eksplisit. ASVS tidak membahas level berbeda per komponen, jadi ini keputusan proyek `[SRC-267]` `[SRC-268]`. Revisi ADR-011 butir 1 |

Panjang minimum password 15 karakter ("strongly recommended", 6.2.1) boleh menjadi bagian M3 atau M4. Angka itu tidak menahan phishing atau pemakaian ulang kredensial, jadi **tidak boleh disajikan sebagai pemenuhan 6.3.3**.

### 5.2 S-06: celah Level 2 manajemen sesi (UP-78, pertanyaan Checkpoint 2)

| Butir | Opsi | Usulan default | Dampak |
| --- | --- | --- | --- |
| 7.5.2 melihat dan mengakhiri sesi aktif `[SRC-266]` | (a) Layar "Perangkat dan sesi aktif" (daftar sesi, waktu terakhir aktif, tombol akhiri satu atau semua) di aplikasi pasien, aplikasi tenaga kesehatan, dan web; (b) hanya tombol "keluar dari semua perangkat", sehingga 7.5.2 menjadi penyimpangan terdokumentasi; (c) ditunda ke rilis berikutnya sebagai penyimpangan terdokumentasi | (a) di Fase 2, bersama aplikasi tenaga kesehatan (EN-17). Sumber daftar sesi adalah Keycloak dan `session_revocation`, dan mekanisme daftar sesi lewat Admin REST API diverifikasi di Tahap 8 | Menambah satu story lintas aplikasi; ukurannya ditaksir bila dipilih. Opsi (b) dan (c) tidak memenuhi L2 penuh |
| 7.1.2 kebijakan sesi paralel terdokumentasi `[SRC-266]` | (a) Tidak dibatasi, didokumentasikan di checklist ASVS beserta alasannya (pasien memakai satu aplikasi per perangkat; web dibatasi idle 30 menit dan mutlak 12 jam, ADR-011 butir 7); (b) dibatasi, misalnya satu sesi web aktif per akun admin | (a) untuk pasien dan tenaga kesehatan. Untuk admin, (a) atau (b) diputuskan user | Opsi (a) hanya berupa dokumentasi; opsi (b) menambah aturan sesi di BFF |

---

## 6. Dampak ke task Fase 1 setelah disetujui

Phase plan tidak diubah di putaran ini. Setelah Checkpoint 2:
- teks AC PRD diperbarui sesuai keputusan;
- test case story terdampak diperbarui QA;
- task berikut menyesuaikan definisinya: F1-T25, F1-T29–F1-T31, F1-T33, F1-T35, F1-T36, F1-T42, F1-T46, F1-T50, F1-T51, F1-T53–F1-T60, F1-T62, F1-T63, F1-T65, F1-T68, F1-T69.

Tidak ada task baru, kecuali bila user memilih opsi yang menambah scope:
- UP-75 opsi (b) layar backoffice;
- email pemberitahuan penguncian (UP-73 butir 9);
- ubah password saat login (US-103 AC6, CR kecil);
- layar sesi aktif (UP-78 a);
- pemberitahuan sub-peran sensitif (UP-76 d-2);
- opsi MFA pasien S-01 M1 atau M2.

Ukuran task dinilai ulang saat AC disetujui.

## 7. Register yang dipakai

- **UP-73–UP-78** (§1.3; dicatat di `docs/07-catatan-phase-plan.md`).
- **KEPUTUSAN-198–KEPUTUSAN-201** (putaran perbaikan review keamanan):
  - 198: penghentian sesi setelah perubahan kredensial atau email;
  - 199: atur ulang password tidak melewati MFA;
  - 200: email lama tetap berlaku sampai verifikasi, dan email pemberitahuan setelah ubah email atau atur ulang password;
  - 201: pengerasan jalur pendaftaran (User Registration Keycloak OFF, jalur ubah email Keycloak ditutup, kompensasi berdasarkan ID, penyamaan waktu respons).
- KEPUTUSAN-188, -190, dan -192 diperbarui di putaran ini.
- **SRC-266–SRC-274** dari review keamanan §8 (R-2–R-10); kutipan R-1 ditambahkan ke SRC-265.
- **KEPUTUSAN-187–KEPUTUSAN-197** (`docs/decision-log.md`):
  - 187: tautan verifikasi email;
  - 188: kebijakan password;
  - 189: pengulangan kompensasi dan email lewat job;
  - 190: parameter penguncian;
  - 191: satuan MiB;
  - 192: batas laju jalur akun;
  - 193: definisi penolakan seketika;
  - 194: validasi profil;
  - 195: jendela autentikasi ulang;
  - 196: batas waktu klien KU-02;
  - 197: ambang peringatan kegagalan audit.
- **SRC-265** OWASP ASVS 5.0.0 bab V6 (teks requirement).
- **SRC-262** dibuka ulang untuk kutipan tambahan (tanpa nomor baru).
- SRC-261, SRC-263, dan SRC-264 tidak berubah.

## 8. Riwayat dokumen

| Versi | Tanggal | Perubahan |
| --- | --- | --- |
| 0.1 | 2026-11-18 | Draf awal putaran perbaikan gate Tahap 7: T-01–T-22 rinci, T-23–T-35 dipetakan, keputusan terpadu enumerasi (UP-73), temuan sampingan S-01–S-05 |
| 0.2 | 2026-11-18 | Perbaikan atas review keamanan putaran 1. **B-01**: pesan seragam login gagal untuk keempat kondisi (§2.3 P1, §2.4 butir 2 dan 4, §2.5 US-103 AC2/AC3 dan KU-10). Catatan C-01–C-14 ditindaklanjuti: §2.7 kontrol pendukung UP-73; T-02 berkas daftar tolak lokal dan US-103 AC6; T-06 Max Wait; T-08 AC4c–AC4e; T-13 KEPUTUSAN-200; T-16 batas per NIK; T-19 kontrol bootstrap dan risiko M-16; §5.1 opsi M1–M4; §5.2 UP-78. Rujukan R-x diganti nomor SRC |
