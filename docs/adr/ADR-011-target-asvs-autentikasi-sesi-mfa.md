# ADR-011: Target ASVS Level 2, dengan Keycloak sebagai identity provider dan pencabutan sesi seketika di aplikasi

Status: Diterima (gate Tahap 4, 2026-11-18 `[USER] T4-G5`; revisi 2026-11-18 sesuai `[USER] T4-Q4`: autentikasi memakai Keycloak, menggantikan rancangan autentikasi buatan sendiri; revisi Checkpoint 2, 2026-11-18, disetujui user `[USER] C2-Q9, C2-Q10, C2-Q11`: MFA wajib untuk pasien, pemulihan perangkat hilang, manajemen sesi oleh pengguna; rincian di bagian "Riwayat revisi")
Perubahan: butir 7 (BFF) ditafsirkan oleh ADR-024 — BFF dijalankan di server Next.js ([USER] T5-Q1), diterima di gate Tahap 5 (2026-11-18).
Tanggal: 2026-11-18

Label `[USER] C2-Qx` merujuk ke jawaban user di `docs/checkpoint-2.md` bagian "Jawaban user" (butir 1–3). `[USER] C2-Bn` merujuk ke butir n yang didelegasikan user ke usulan penyusun di dokumen yang sama (butir 4–15).

## Konteks

- NFR-SEC-01: kontrol keamanan diverifikasi terhadap OWASP ASVS 5.0.0; level dan pemetaan ditetapkan di Tahap 4 `[DOK: 03-prd.md]` `[KEPUTUSAN-05]`. ASVS 5.0 menyebut Level 2 sebagai level yang "Most applications should be striving to achieve", Level 3 untuk tingkat keamanan tertinggi, dan level dipilih menurut sensitivitas aplikasi `[SRC-48]`.
- User memutuskan memakai Keycloak dan Redis, tanpa message broker `[USER] T4-Q4`. Semua komponen dan penyedia wajib memproses data di Indonesia `[USER] T4-Q1`.
- Login pasien hanya email dan password; login Google/Apple dihapus (CR-007) `[USER] T4-Q2`.
- KEPUTUSAN-06 menetapkan kontrol berikut. MFA tenaga kesehatan diputuskan di Tahap 4 `[KEPUTUSAN-06]`.
  - verifikasi email sebelum akun aktif;
  - penguncian 15 menit setelah 5 kali gagal berturut-turut;
  - tautan atur ulang password sekali pakai 60 menit;
  - MFA wajib untuk admin Ruang Sehat, admin rekanan, dan pengguna RME;
  - sesi web berakhir setelah 30 menit tanpa aktivitas.
- Akun nonaktif harus ditolak termasuk pada sesi yang masih terbuka (KU-10). Untuk pengguna RME, akses berakhir seketika (US-1302 AC4). Keluar mengakhiri sesi di server (US-103 AC5). Akun tidak dibuat sebelum pasien menyetujui persetujuan pemrosesan data (US-101 AC1, AC4; US-104) `[DOK: 03-prd.md]`.
- Kemampuan Keycloak yang terverifikasi (versi 26.8.2, lisensi Apache 2.0 `[SRC-64]`):
  - database PostgreSQL `[SRC-65]` dan klaster multi-node `[SRC-66]`;
  - kebijakan OTP berbasis waktu `[SRC-67]` dan kode pemulihan sebagai fitur stabil `[SRC-68]`;
  - deteksi brute force `[SRC-69]`;
  - Argon2 sebagai hash bawaan non-FIPS, verifikasi email, batas waktu atur ulang password, SSO Session Idle/Max, Access Token Lifespan, back-channel logout, dan realm yang terisolasi `[SRC-70]`;
  - Authorization Code dengan PKCE, client credentials, introspeksi, dan revokasi token `[SRC-71]`;
  - Admin REST API untuk membuat user, mengubah status `enabled`, menghapus semua sesi user, dan mengambil event `[SRC-72]`;
  - penyimpanan event dengan masa simpan `[SRC-73]`.
- **Tambahan revisi Checkpoint 2.** ASVS 5.0.0 mewajibkan MFA mulai Level 2: "L2 applications must force the use of multi-factor authentication (MFA)" dan 6.3.3 (L2) `[SRC-265]`. Versi awal ADR ini menargetkan Level 2 untuk semua komponen (butir 1), tetapi tidak mewajibkan MFA bagi pasien, sehingga klaim Level 2 untuk realm `pasien` tidak benar (`docs/07-klarifikasi-ac-fase-1.md` §5.1; `docs/review/tahap-07-security.md` §3). User memilih opsi M1, yaitu MFA wajib untuk pasien dengan TOTP dan kode pemulihan `[USER] C2-Q9`.
- ASVS 5.0.0 juga mensyaratkan, di Level 2:
  - pembuktian identitas setara saat pendaftaran bila faktor MFA hilang: "if a multi-factor authentication factor is lost, evidence of identity proofing is performed at the same level as during enrollment" (6.4.4) `[SRC-276]`;
  - pengguna dapat melihat dan, setelah autentikasi ulang dengan minimal satu faktor, mengakhiri sesi aktifnya (7.5.2) `[SRC-277]`;
  - dokumentasi jumlah sesi paralel per akun (7.1.2) `[SRC-277]`.

## Keputusan

**1. Target verifikasi ASVS 5.0.0 Level 2** untuk semua komponen, termasuk konfigurasi Keycloak. Pemetaan kontrol disusun di Tahap 5 sebagai checklist dan diverifikasi di Tahap 10 `[KEPUTUSAN-26]`.

**2. Keycloak sebagai identity provider** untuk semua akun manusia Ruang Sehat dan RME, serta untuk klien integrasi rekanan `[USER] T4-Q4` `[KEPUTUSAN-24]`:
- dijalankan sendiri di cloud Indonesia, minimal 2 instans terklaster `[SRC-66]`;
- memakai database PostgreSQL tersendiri `[SRC-65]` di instans database aplikasi Ruang Sehat, ikut PITR (ADR-019);
- versi yang dipakai ditetapkan di Tahap 5.
- **Hanya endpoint OIDC realm pengguna yang publik** (login, token, logout, dan halaman yang diperlukan). Konsol admin, realm master, dan Admin REST API hanya dapat dicapai dari jaringan internal (proses backend dan jalur administrasi ber-MFA, ADR-021 butir 3). Cara pemisahan endpoint diverifikasi ke dokumentasi Keycloak di Tahap 5 (threat model KC-06, M-05).
- Akun admin realm master seminimal mungkin dan ber-MFA. Backend Ruang Sehat dan backend RME memakai klien Admin REST API terpisah per realm, dengan peran admin minimum. Backend Ruang Sehat tidak punya hak di realm `rme`, dan sebaliknya.
- Admin event yang mengubah konfigurasi realm memicu peringatan di security log.

**3. Realm terpisah per populasi.** Realm terisolasi satu sama lain `[SRC-70]`. Pembagiannya `[KEPUTUSAN-24]`:

| Realm | Pengguna | Kebijakan utama |
| --- | --- | --- |
| `pasien` | Pasien | Pendaftaran lewat backend (butir 6), verifikasi email, MFA TOTP wajib dengan kode pemulihan (butir 5) `[USER] C2-Q9`. Sebelum revisi Checkpoint 2: tanpa MFA |
| `nakes` | Dokter, psikolog, fisioterapis, tenaga medis | Pendaftaran lewat backend (US-202), MFA TOTP wajib `[KEPUTUSAN-25]` |
| `admin` | Admin Ruang Sehat dan admin rekanan | Hanya undangan, MFA TOTP wajib |
| `rme` | Pengguna RME minimalis, termasuk Operator RME | Hanya undangan, MFA TOTP wajib. Backend RME hanya menerima token realm ini |
| `integrasi` | Klien sistem rekanan dan fasyankes RME | Client credentials, satu klien per rekanan/fasyankes (ADR-014) |

**4. Kebijakan kredensial** `[KEPUTUSAN-24]`:
- Hash password memakai Argon2, bawaan Keycloak untuk deployment non-FIPS `[SRC-70]`. Parameternya minimal setara OWASP Password Storage Cheat Sheet (19 MiB, 2 iterasi, paralelisme 1) `[SRC-49]`; kemampuan mengatur parameter diverifikasi di Tahap 5. Kebijakan panjang dan isi password mengikuti ASVS Level 2.
- Verifikasi email wajib sebelum akun dipakai `[SRC-70]`.
- Tautan atur ulang password berlaku 60 menit `[SRC-70]`.
- Deteksi brute force dalam mode sementara: maksimal 5 kegagalan, penguncian 15 menit (KEPUTUSAN-06) `[SRC-69]`. Kesesuaian perilakunya dengan "5 kali gagal berturut-turut" diuji otomatis di Tahap 8.
- Pesan galat login dan atur ulang password bersifat umum. Pendaftaran dengan email yang sudah terdaftar mengikuti US-101 AC3.
- Revisi Checkpoint 2 (UP-73 opsi B, menegaskan butir ini) `[USER] C2-B4`:
  - semua kegagalan login (password salah, akun tidak ada, terkunci sementara, nonaktif) memakai satu pesan seragam yang identik kata per kata di semua realm bertema (US-103 AC2);
  - pendaftaran dengan email terdaftar menampilkan layar yang sama dengan pendaftaran berhasil, dan petunjuknya dikirim ke email itu (US-101 AC3);
  - kontrol pendukungnya `[KEPUTUSAN-192]` `[KEPUTUSAN-201]`;
  - email pemberitahuan ke pemilik saat akun terkunci tidak dibangun di rilis 1.

**5. MFA dengan TOTP** (RFC 6238 `[SRC-43]`) wajib untuk realm `nakes`, `admin`, dan `rme` `[KEPUTUSAN-25]`, dan sejak revisi Checkpoint 2 juga untuk realm `pasien` `[USER] C2-Q9`:
- Alur autentikasi realm disesuaikan supaya OTP wajib bagi semua pengguna. Alur bawaan Keycloak bersifat bersyarat, hanya meminta OTP dari pengguna yang sudah mengonfigurasinya `[SRC-70]`.
- Required action "Configure OTP" dijalankan saat login pertama. Untuk pasien, login pertama terjadi setelah email terverifikasi (US-101 AC5) `[KEPUTUSAN-202]`.
- Algoritma, jumlah digit, dan periode `[SRC-67]` ditetapkan di Tahap 5 dengan memperhatikan kompatibilitas aplikasi authenticator (KEPUTUSAN-85). Nilai yang sama dipakai realm `pasien`.
- Kode pemulihan memakai fitur bawaan Keycloak (12 kode sekali pakai) `[SRC-68]` `[SRC-70]`. Kode pemulihan wajib dibuat saat pendaftaran MFA pertama, termasuk oleh pasien. Mekanisme Keycloak untuk mewajibkannya `[TIDAK DIKETAHUI]`, diverifikasi di Tahap 8 `[KEPUTUSAN-203]`.
- **Atur ulang password tidak melewati MFA di semua realm pengguna**, termasuk `pasien` (6.4.3 L2) `[SRC-265]`. Setelah tautan email dibuka, OTP atau kode pemulihan tetap diminta sebelum password baru berlaku dan sebelum akses diberikan (US-103 AC4e) `[KEPUTUSAN-199]`.
- Reset MFA oleh admin yang berwenang setelah verifikasi identitas, dan tercatat (E-04). Reset MFA mengakhiri semua sesi pengguna `[KEPUTUSAN-198]`.
- **Pemulihan perangkat hilang** (6.4.4 L2 `[SRC-276]`; prosedur H-05 meluas ke pasien `[USER] C2-Q9`) `[KEPUTUSAN-204]`:
  - Jalur mandiri: pengguna masuk dengan kode pemulihan, lalu mendaftarkan TOTP baru setelah autentikasi ulang.
  - Jalur pasien lewat Layanan Pelanggan, bila kode pemulihan juga hilang:
    1. Layanan Pelanggan memulai reset MFA di backoffice.
    2. Sistem mengirim tautan konfirmasi sekali pakai ke email akun, yaitu kanal yang berbeda dari kanal permintaan.
    3. Reset baru berlaku setelah pasien membuka tautan itu dan memasukkan password yang benar. Bukti ini setara dengan pembuktian saat pendaftaran (kendali atas email akun), ditambah pengetahuan password. Mekanisme pemeriksaan password di langkah ini, tanpa meminta TOTP lama, `[TIDAK DIKETAHUI]`; dirancang di Tahap 8 bersama ADR jalur email (KEPUTUSAN-183), dan bila butuh komponen atau pola baru, diusulkan sebagai ADR.
    4. Layanan Pelanggan tidak pernah melihat, memilih, atau menetapkan kredensial pasien.
    5. Setelah reset, semua sesi diakhiri, email pemberitahuan dikirim ke email akun, dan TOTP baru wajib didaftarkan saat masuk berikutnya.
  - Jalur tenaga kesehatan, admin, dan pengguna RME mengikuti prosedur H-05 yang sudah ada (sub-peran berwenang, verifikasi lewat kanal berbeda, pemberitahuan, pencabutan sesi).
  - Pasien yang kehilangan perangkat, kode pemulihan, dan akses ke email atau password sekaligus tidak tercakup jalur di atas. Bukti identitas pengganti untuk kasus itu `[TIDAK DIKETAHUI]` (pertanyaan terbuka, risk register RT-37), ditetapkan sebelum prosedur H-05 final sebelum go-live.
- SMS dan OTP email tidak dipakai sebagai faktor kedua. WebAuthn tidak dipakai di rilis 1, karena user memilih TOTP dan kode pemulihan `[USER] C2-Q9`. Penambahannya dapat diajukan lewat change request.

**6. Pendaftaran dan pembuatan akun** `[KEPUTUSAN-24]`:
- **Pasien**: formulir pendaftaran native di aplikasi, termasuk layar persetujuan US-104. Backend memeriksa persetujuan, membuat user di realm `pasien` lewat Admin REST API `[SRC-72]` dengan required action verifikasi email, lalu menyimpan akun aplikasi dan bukti persetujuan di database aplikasi. Bila salah satu langkah gagal, user Keycloak yang sudah terbuat dihapus kembali (kompensasi), sehingga tidak ada akun tanpa persetujuan (US-101 AC4).
- **Tenaga kesehatan**: pola yang sama di realm `nakes`. Akun aplikasi berstatus "Menunggu verifikasi" (US-202).
- **Admin dan pengguna RME**: dibuat lewat backoffice atau RME. User Keycloak dibuat lewat Admin REST API, lalu undangan aktivasi dikirim untuk mengatur password dan TOTP (US-206, US-208, US-1302).
- Keycloak hanya menyimpan data identitas minimal (email sebagai username, dan kredensial). Profil, data identitas, dan peran disimpan di database aplikasi.
- Perubahan email mandiri di konsol akun Keycloak dinonaktifkan. Email hanya diubah lewat API aplikasi (US-105 AC4), yang memperbarui Keycloak lewat Admin REST API `[SRC-72]`, sehingga salinan email di aplikasi tidak tertinggal.

**7. Alur login** `[KEPUTUSAN-24]`:
- **Aplikasi mobile**: OIDC Authorization Code dengan PKCE lewat browser sistem, bukan webview di dalam aplikasi, sesuai RFC 8252 `[SRC-77]` `[SRC-71]`. Halaman login memakai tema Ruang Sehat; kemampuan tema diverifikasi di Tahap 5 dan desainnya di Tahap 6. Token disimpan di penyimpanan aman platform perangkat (pustaka ditetapkan Tahap 5).
- **Web** (backoffice, portal, RME): pola *backend-for-frontend* (BFF). Backend menjalankan Authorization Code sebagai klien rahasia, sehingga token tidak pernah sampai ke browser. Browser memegang cookie sesi `HttpOnly`, `Secure`, dan `SameSite`, dengan perlindungan CSRF. Data sesi BFF disimpan di Redis (ADR-007). Token di dalamnya dienkripsi dengan kunci dari secret manager dan diikat ke ID sesi, sehingga pembaca Redis tidak memperoleh token yang dapat dipakai (M-06).
- **Batas sesi (nilai awal)**: diatur lewat SSO Session Idle/Max dan Access Token Lifespan per realm `[SRC-70]`.
  - Web: tidak aktif 30 menit (NFR-SEC-04), mutlak 12 jam.
  - Pasien mobile: 30 hari dan 90 hari.
  - Tenaga kesehatan mobile: 7 hari dan 30 hari.
  - Token akses: 5 menit.
  - Rotasi token penyegar mengikuti pengaturan Keycloak, diverifikasi di Tahap 5.

**8. Penolakan seketika di aplikasi** (KU-10, US-1302 AC4, US-103 AC5) `[KEPUTUSAN-24]`:
- Setiap permintaan ke backend memvalidasi tanda tangan, audience realm, dan masa berlaku token. Sesuai RFC 8725: hanya algoritma dalam daftar izin; kunci harus milik issuer realm yang diharapkan; token ditolak bila audience tidak ada atau bukan untuk API itu `[SRC-79]`. Test negatif disiapkan untuk setiap kombinasi realm dan klien. Backend juga memeriksa status akun aplikasi dan daftar sesi yang dicabut.
- Pemeriksaan memakai cache Redis yang diperbarui saat status berubah. Sumber kebenarannya database aplikasi. Bila Redis tidak tersedia, backend membaca status langsung dari database.
- **Penonaktifan akun atau perubahan peran**:
  - status di database aplikasi diubah dan dicatat (E-10, E-11);
  - cache Redis diperbarui seketika;
  - user di Keycloak dinonaktifkan (`enabled = false`) dan semua sesinya dihapus lewat Admin REST API `[SRC-72]`.
  Akses ditolak sejak langkah kedua, walaupun token lama belum kedaluwarsa. Bila langkah Redis atau Keycloak gagal, langkah itu dicoba ulang lewat job dan memicu peringatan. Entri cache ber-TTL pendek. Aksi berisiko (ekspor, refund, perubahan peran, akses dokumen identitas) membaca status langsung dari database (M-13).
- **Keluar**: aplikasi memanggil logout Keycloak atau endpoint revokasi `[SRC-71]`. Backend mencatat sesi itu sebagai dicabut di tabel Operasional database aplikasi, lalu menyalinnya ke cache Redis. Barisnya dibersihkan setelah token terkait pasti kedaluwarsa. Dengan begitu pencabutan tetap berlaku walaupun isi Redis hilang (fact-check FC4-N2-2).

**9. Autentikasi ulang** untuk aksi berisiko (mengajukan hak subjek data, mengunduh salinan data, mengubah email atau password, reset MFA, mendaftarkan ulang TOTP, dan mengakhiri sesi dari layar "Perangkat dan sesi aktif" (butir 12)): backend meminta login ulang di Keycloak dan memeriksa waktu autentikasi terakhir di token. Mekanisme parameternya diverifikasi di Tahap 5 `[KEPUTUSAN-24]`.

**10. Event autentikasi ke audit trail** `[KEPUTUSAN-24]`:
- Keycloak menyimpan event login dan admin event dengan masa simpan, nilai awal 7 hari sebagai penyangga `[SRC-73]`.
- Worker mengambil event lewat Admin REST API `[SRC-72]` setiap menit, memetakannya ke E-01–E-04, lalu menulisnya ke outbox audit secara idempoten (ADR-009). Event yang sama juga diteruskan ke security log (ADR-010).
- Sebagian jenis event Keycloak berstatus usang `[SRC-73]`, sehingga pemetaan jenis event ditetapkan per versi di Tahap 5.
- Keterlambatan pengambilan lebih dari 5 menit memicu peringatan keamanan.
- Pembuatan akun aplikasi, perubahan peran, dan penonaktifan dicatat langsung oleh aplikasi (E-04, E-10, E-11).

**11. Satu deployment Keycloak untuk Ruang Sehat dan RME**, dengan realm dan admin realm terpisah. Backend Ruang Sehat tidak menerima token realm `rme`, dan sebaliknya `[KEPUTUSAN-24]`.

**12. Manajemen sesi oleh pengguna** (revisi Checkpoint 2, UP-78):
- **Melihat dan mengakhiri sesi aktif (7.5.2 L2 `[SRC-277]`).**
  - Layar "Perangkat dan sesi aktif" dibangun di Fase 2 untuk semua aplikasi: aplikasi pasien, aplikasi tenaga kesehatan, backoffice, portal rekanan, dan web RME (US-110) `[USER] C2-Q10`. Bagian web RME diverifikasi di Fase 3 bersama fondasi RME (EN-22).
  - Daftar sesi diambil dari Keycloak lewat Admin REST API ("Get sessions associated with the user"). Satu sesi diakhiri lewat "Remove a specific user session" `[SRC-275]`, ditambah catatan `session_revocation` di aplikasi (butir 8).
  - Mengakhiri sesi mewajibkan autentikasi ulang (butir 9) `[KEPUTUSAN-205]`.
- **Sesi paralel (7.1.2 L2 `[SRC-277]`).**
  - Jumlah sesi paralel per akun tidak dibatasi di semua realm pengguna (`pasien`, `nakes`, `admin`, `rme`) `[USER] C2-Q11`. Karena tidak ada batas maksimum, tidak ada perilaku saat batas tercapai.
  - Kompensasinya: setiap sesi tampil dan dapat diakhiri lewat US-110; batas idle dan mutlak per realm (butir 7) tetap berlaku; sesi lain diakhiri setelah perubahan kredensial atau email `[KEPUTUSAN-198]`.
  - Kebijakan ini didokumentasikan di `docs/05-asvs-checklist.md` V7.

## Alternatif yang ditolak

- **Autentikasi dibangun sendiri** (rancangan awal ADR ini): user memilih komponen yang sudah teruji `[USER] T4-Q4`. Autentikasi juga modul kritis, sehingga mengurangi kode buatan sendiri mengurangi risiko.
- **Login di webview di dalam aplikasi atau grant password langsung**: RFC 8252 mewajibkan aplikasi native memakai user-agent eksternal `[SRC-77]`.
- **Satu realm untuk semua pengguna**: kebijakan pendaftaran, MFA, dan sesi berbeda per populasi, sehingga satu kesalahan konfigurasi bisa berdampak lintas populasi.
- **Deployment Keycloak terpisah untuk RME**: biaya operasional ganda. Dievaluasi ulang bila jawaban L-05 menuntut pemisahan infrastruktur.
- **Hanya mengandalkan masa berlaku token**: penonaktifan baru berlaku setelah token kedaluwarsa, sehingga KU-10 dan US-1302 AC4 tidak terpenuhi.
- **ASVS Level 3 untuk seluruh sistem**: tidak realistis dalam jadwal rilis 1. Kontrol Level 3 tertentu dapat diambil di Tahap 5 tanpa mengubah target.
- **MFA pasien opsi M2, M3, dan M4** (revisi Checkpoint 2), yaitu MFA sukarela dengan step-up untuk aksi berisiko, tanpa MFA dengan penyimpangan 6.3.3 terdokumentasi, dan target V6 realm `pasien` diturunkan ke Level 1. Ketiganya tidak memenuhi 6.3.3 penuh untuk akses dasar, dan user memilih M1 `[USER] C2-Q9` (opsi lengkap di `docs/07-klarifikasi-ac-fase-1.md` §5.1).
- **WebAuthn sebagai faktor kedua di rilis 1**: tidak termasuk pilihan user `[USER] C2-Q9`. Lokasi pemrosesan passkey yang disinkronkan lewat layanan awan platform juga `[TIDAK DIKETAHUI]` terhadap ketentuan T4-Q1.
- **Hanya tombol "keluar dari semua perangkat", atau menunda layar sesi aktif**: tidak memenuhi 7.5.2 penuh; user memilih layar lengkap di Fase 2 `[USER] C2-Q10`.
- **Membatasi jumlah sesi web per akun admin**: tidak dipilih; sesi paralel tidak dibatasi di semua realm `[USER] C2-Q11`.

## Konsekuensi

- Ada komponen tambahan yang dioperasikan, dipantau, di-upgrade, dan di-backup: klaster Keycloak dan databasenya. Konfigurasi realm disimpan sebagai kode, mekanismenya ditetapkan di Tahap 5.
- Login berpindah ke halaman Keycloak di browser sistem. Tema login perlu didesain (Tahap 6) dan dibangun.
- Keycloak, konfigurasi realm, dan validasi token di backend adalah modul kritis (T1).
- Tenaga kesehatan memerlukan aplikasi authenticator. Kehilangan perangkat MFA diselesaikan lewat prosedur reset oleh admin yang ditulis sebelum go-live.
- Nilai batas sesi adalah nilai awal teknis yang diselaraskan dengan checklist ASVS di Tahap 5.
- **Konsekuensi revisi Checkpoint 2:**
  - Pasien juga memerlukan aplikasi authenticator. Pendaftaran dan login pasien bertambah satu langkah. Dampaknya terhadap adopsi dan KPI-01 `[TIDAK DIKETAHUI]` (risk register RT-35).
  - Pemulihan perangkat hilang pasien lewat Layanan Pelanggan menambah beban operasional. Layanan Pelanggan juga menjadi sasaran rekayasa sosial seperti KC-02 (risk register RT-36). Prosedur H-05 yang mencakup pasien ditulis sebelum go-live.
  - Realm `pasien` kini dapat diklaim memenuhi 6.3.3. Risiko sisa threat model APP-04 ("pasien tanpa MFA") tertutup. TOTP bukan faktor perangkat keras yang tahan phishing, yang baru diwajibkan 6.3.3 di L3 `[SRC-276]`, sehingga phishing seketika atas password dan kode TOTP tetap dicatat sebagai risiko sisa.
  - Scope rilis 1 bertambah: pendaftaran TOTP dan kode pemulihan pasien di Fase 1 (CR-018), reset MFA pasien oleh Layanan Pelanggan di Fase 2 (US-207 AC5), layar sesi aktif di Fase 2 (US-110), dan ubah password saat login (CR-017). Ukuran task dinilai di phase plan.

## Riwayat revisi

| Tanggal | Revisi | Sumber |
| --- | --- | --- |
| 2026-11-18 | Diterima di gate Tahap 4 | `[USER] T4-G5` |
| 2026-11-18 | Autentikasi memakai Keycloak, menggantikan rancangan buatan sendiri | `[USER] T4-Q4` |
| 2026-11-18 | Butir 7 (BFF) ditafsirkan ADR-024: BFF di server Next.js | `[USER] T5-Q1`, gate Tahap 5 |
| 2026-11-18 | **Revisi Checkpoint 2**, status tetap Diterima:<br>- Konteks: kewajiban MFA Level 2, 6.4.4, 7.5.2, 7.1.2;<br>- butir 3: realm `pasien` MFA wajib;<br>- butir 4: pesan seragam login gagal dan pendaftaran, ditegaskan lewat UP-73 opsi B; email pemberitahuan penguncian tidak dibangun;<br>- butir 5: MFA wajib pasien (TOTP dan kode pemulihan); atur ulang password tidak melewati MFA di semua realm, termasuk `pasien`; pemulihan perangkat hilang, prosedur H-05 diperluas ke pasien lewat Layanan Pelanggan; WebAuthn tidak dipakai di rilis 1;<br>- butir 9: aksi yang memerlukan autentikasi ulang ditambah;<br>- butir 12 baru: layar sesi aktif di Fase 2 untuk semua aplikasi; sesi paralel tidak dibatasi untuk semua realm, terdokumentasi;<br>- alternatif ditolak dan konsekuensi ditambah | `[USER] C2-Q9, C2-Q10, C2-Q11`, `[USER] C2-B4` (`docs/checkpoint-2.md`); `[KEPUTUSAN-198]`, `[KEPUTUSAN-199]`, `[KEPUTUSAN-202]`–`[KEPUTUSAN-205]` |
