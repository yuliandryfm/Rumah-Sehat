# ADR-032: MFA HR realm `korporat` memakai kode sekali pakai lewat WhatsApp: autentikator Keycloak kustom yang tipis, `otp-relay` Rust dengan port pengirim, dan kode tetap khusus demo

Status: **Diusulkan** (usulan, 2026-11-28; direvisi 2026-11-28 sesudah user menjawab P32-1–P32-7, lihat "Riwayat revisi"). Arah ditetapkan user `[USER]` CR-041 (2026-11-28). Dokumen ini menunggu review T1 (keamanan, code-reviewer, DBA untuk tabel baru) dan persetujuan user. **Adaptor `whatsapp` di production** diizinkan sebagai pengecualian T4-Q1/CR-007 khusus kode masuk HR (P32-1 = A), tetapi tetap terkunci oleh gerbang pada butir 10 (UU PDP Pasal 56, teks persetujuan HR, BSP, templat Meta).
Mengubah bila diterima: ADR-011 butir 3 (baris realm `korporat`) dan butir 5 (faktor kedua dan pemulihan HR); ADR-027 butir 3 (MFA TOTP dan pemulihan MFA HR); ADR-024 (Java/Maven masuk stack khusus provider Keycloak, image Keycloak kustom); ADR-025 butir 7 (identitas OpenBao `otp-relay`); KEPUTUSAN-253 dan KEPUTUSAN-265; ADR-016 dan CR-007 (pengecualian P32-1 A); **ADR-030** butir 3 dan 4 (nomor belum terverifikasi ikut `idp_create_user`; dua jenis job pemulihan baru `idp_reset_wa_factor` dan `idp_send_wa_reenroll`; kunci Transit `hr-phone`); **ADR-031** (isian nomor di formulir `/daftar`; rute publik ketiga `/bantuan-akses`); ADR-012 (`PublicReason` baru dan izin sub-peran pemulihan); ADR-013 (kunci `hr-phone`); ADR-008 (kelas tabel penyangga dan tabel permintaan pemulihan).
Tidak mengubah: realm `pasien`, `nakes`, `admin`, `rme` (tetap TOTP, keputusan user, butir 9); ADR-026 (jalur email tetap, hanya dipakai untuk jenis job pemberitahuan baru).
Tanggal: 2026-11-28
Tingkat ketelitian: T1 (autentikasi, sesi, dan MFA; konfigurasi realm Keycloak; pipeline dan image; `aturan-kerja.md` §1)

## Konteks

**Keputusan user.**
- Faktor kedua akun HR realm `korporat` berganti dari TOTP aplikasi authenticator menjadi kode sekali pakai lewat WhatsApp, tanpa aplikasi authenticator: "i want move to WhatsApp OTP as MFA. just need develop WA Agent later. no need other apps as authenticator. its cheapest and easy to handle" `[USER]` CR-041.
- Masa transisi: "on demo, OTP should be static value, example \"A1B2C3\"" `[USER]` CR-041.
- Ruang lingkup: "only korporat realm, others stay TOTP" `[USER]` 2026-11-28 (terminal, diteruskan orkestrator).
- Jawaban atas P32-1–P32-7 `[USER]` 2026-11-28 (terminal dan Telegram), ringkas; teks lengkap di tabel "Butir keputusan user (P32-1–P32-7 sudah dijawab)":
  - P32-1 A: WhatsApp dipakai di semua lingkungan termasuk production, pengecualian T4-Q1/CR-007 khusus kode masuk HR;
  - P32-2 A: WhatsApp Business Platform resmi lewat BSP terdaftar Meta;
  - P32-3 B: kode pemulihan opsional, HR yang kehilangan nomor mengajukan tiket ke tim support;
  - P32-4 B: nomor diisi di `/daftar` dan formulir Undang HR, tetap diverifikasi saat login pertama;
  - P32-5 A: hanya +62, awalan 0 dinormalkan ke E.164;
  - P32-6 A: kode tetap hanya `local` dan `staging`;
  - P32-7 B: pemulihan lewat tiket support dimajukan ke Fase 1.

**Keadaan sekarang.**
- Realm `korporat` mewajibkan password lalu OTP atau kode pemulihan (`rs-browser-mfa`), memaksa Configure OTP bila belum ada faktor kedua, meminta OTP sebelum password baru di atur ulang (`rs-reset-credentials-mfa`), dan direct grant mati `[DOK: infra/keycloak/realms/korporat.json]`.
- KEPUTUSAN-253 menetapkan TOTP + kode pemulihan untuk HR; KEPUTUSAN-265 menetapkan jalur pemulihan HR (kode pemulihan dulu, lalu reset yang dimulai Verifikator, dibangun Fase 2) `[DOK: decision-log.md]`.
- NFR-SEC-02 PRD menetapkan "Faktor kedua berupa TOTP dengan kode pemulihan sekali pakai; SMS, OTP email, dan WebAuthn tidak dipakai di rilis 1" `[DOK: 03-prd.md]`.
- Token akses realm pengguna memuat `acr`=1 tanpa `amr` `[SRC-682]`. Backend tidak membaca `acr`/`amr`; jaminan MFA berasal dari konfigurasi realm yang diuji `keycloak-check` (TD-27) `[DOK: infra/keycloak/README.md]`.

**Lokasi data dan WhatsApp.**
- T4-Q1: semua penyedia pihak ketiga memproses dan menyimpan data di Indonesia, tanpa kecuali `[USER] T4-Q1`. Karena itu CR-007 menghapus WhatsApp: lokasi pemrosesan WhatsApp Cloud API tidak dapat dipastikan di Indonesia `[DOK: backlog.md CR-007]` `[DOK: 02-brd.md BR-050]`.
- Dokumentasi Meta: "Cloud API processes messages on servers in Meta data centers" `[SRC-54]`. Dengan *local storage*, isi pesan tetap "may be stored on Meta data centers internationally while being processed", dan "the data-in-use period is up to 60 minutes" (dibuka ulang 2026-11-28) `[SRC-997]`. On-Premises API sudah kedaluwarsa, dan sejak 1 Juli 2024 nomor bisnis hanya dapat didaftarkan untuk Cloud API `[SRC-55]`.
- UU PDP Pasal 56 mengatur transfer data pribadi ke luar wilayah hukum Indonesia secara bersyarat `[SRC-15]`. Nomor telepon HR adalah data pribadi.
- Ketentuan layanan WhatsApp untuk akun biasa melarang akses "through automated or other means" yang tidak sah, "bulk messaging, auto-messaging", dan "non-personal use of our Services unless otherwise authorized by us" `[SRC-998]`.
- Templat pesan autentikasi WhatsApp Business Platform (menurut ringkasan halaman resmi, dibuka 2026-11-28): teks isi preset tetap, dengan pernyataan keamanan dan peringatan kedaluwarsa opsional (1–90 menit), dan tombol salin kode, autofill, atau tanpa tombol. Aplikasi yang mengirim kode lewat WhatsApp memakai templat autentikasi `[SRC-999]`. Isi persis dan aturan templat dipastikan ulang saat A32-T10.

**Standar.**
- NIST SP 800-63B-4 `[SRC-990]`:
  - §3.1.3.1: perangkat out-of-band boleh diautentikasi lewat SIM jaringan seluler hanya bila rahasia dikirim "via the PSTN (i.e., SMS or voice) or an encrypted instant messaging service". "Email SHALL NOT be used for out-of-band authentication."
  - §3.1.3.2: rahasia acak "at least six decimal digits" (atau setara); "the authentication SHALL be considered invalid unless completed within 10 minutes"; diterima "only once during the validity period"; bila rahasia di bawah 64 bit, verifier wajib membatasi laju (§3.2.2); "Generating a new authentication secret SHALL NOT reset the failed authentication count."
  - §3.1.3.3 hanya membatasi PSTN: "Use of the PSTN for out-of-band verification is restricted". Kewajiban "alternative authenticator types are available to all subscribers" dan pertimbangan "device swap, SIM change, number porting" ada di bagian PSTN ini. "Setting or changing the pre-registered telephone number is considered to be the binding of a new authenticator."
  - §3.2.2: kegagalan berturut-turut dibatasi "no more than 100", dan instansi boleh memakai batas lebih rendah.
  - **Koreksi atas catatan CR-041:** pesan instan terenkripsi disebut terpisah dari PSTN dan tidak berstatus *restricted*. Kewajiban jenis autentikator alternatif di §3.1.3.3 melekat pada PSTN, bukan pada WhatsApp. Kode pemulihan bersifat opsional (P32-3 B, butir 7.1): ASVS 6.4.4 dan NIST §4.2 mewajibkan adanya jalur pemulihan dengan pembuktian identitas dan pemberitahuan, bukan kode pemulihan secara khusus (butir 7.1).
- NIST SP 800-63B-4 `[SRC-991]`: penambahan autentikator setelah pendaftaran butuh autentikasi pada AAL tertinggi yang tersedia, dan "the CSP SHALL notify the subscriber via a mechanism independent of the transaction" (§4.1.2.1). Kode pemulihan tersimpan adalah metode pemulihan yang sah, dan wajib dibatalkan setelah dipakai (§4.2.1). Setiap pemulihan akun wajib memicu pemberitahuan ke pelanggan (§4.2).
- OWASP ASVS 5.0.0 `[SRC-992]`:
  - 6.5.1 (L2): kode out-of-band sekali pakai;
  - 6.5.3 (L2): dibangkitkan CSPRNG;
  - 6.5.4 (L2): minimal 20 bit entropi;
  - 6.5.5 (L2): umur maksimum 10 menit;
  - 6.6.1 (L2): khusus PSTN;
  - 6.6.2 (L2): terikat ke permintaan autentikasi asal;
  - 6.6.3 (L2): dilindungi dari brute force.
- ASVS juga menyebut "Unsafe out-of-band authentication mechanisms such as e-mail and VOIP are not permitted" (pengantar V6.6), dan 6.3.3 (L2) mewajibkan MFA untuk mengakses aplikasi `[SRC-265]`. 6.4.4 (L2) mensyaratkan pembuktian identitas setara pendaftaran saat faktor hilang `[SRC-276]`.

**Kemampuan Keycloak 26.8.2** (versi di `docs/05-stack.md` §7).
- Tidak ada autentikator WhatsApp bawaan. Faktor baru di dalam alur Keycloak hanya mungkin lewat Authentication SPI, yang berstatus "an internal SPI. While it was stable in the previous releases, any new release of Keycloak can change it without prior notice" `[SRC-993]`.
- Bentuk SPI-nya `[SRC-993]`:
  - autentikator punya `authenticate()` (kunjungan pertama) dan `action()` (kiriman formulir); `configuredFor()` menentukan apakah pengguna sudah dikonfigurasi; `setRequiredActions()` dipanggil bila belum, bila autentikator wajib, dan bila factory mengizinkan *user setup*;
  - `failureChallenge()` "also records the failure so it can be analyzed by any attack detection service";
  - required action punya `requiredActionChallenge()` dan `processAction()`;
  - paket JAR memuat berkas `META-INF/services` per factory. Provider didaftarkan dengan menyalin JAR ke direktori `providers`, lalu Keycloak di-build ulang (`kc.sh build`).
- Required action dapat mendukung *application-initiated action* (AIA), dengan `getMaxAuthAge()` yang menentukan kapan autentikasi ulang diminta untuk AIA `[SRC-994]`.
- Dokumentasi image: JAR ditambahkan ke `/opt/keycloak/providers/` sebelum `RUN /opt/keycloak/bin/kc.sh build`, lalu start `--optimized` `[SRC-996]`.
- Deteksi brute force berlaku juga untuk OTP dan kode pemulihan `[SRC-273]`. Kode pemulihan adalah fitur stabil `[SRC-68]`.
- Eksekusi autentikasi dapat diberi nilai referensi. Mapper AMR mengisi klaim `amr` dari nilai referensi eksekusi yang selesai (ID `oidc-amr-mapper` baru terlihat di indeks versi nightly; dikonfirmasi untuk 26.8.2 saat task). `acr` diisi mapper `acr loa level`, dan klien dianjurkan memeriksa `acr` sesuai level yang diharapkan `[SRC-994]` `[SRC-995]`.

**Preseden proyek.**
- ADR-026: email Keycloak lewat relay Rust per host di jaringan Docker khusus. `EmailSenderProvider` Java ditolak karena SPI internal dan karena menambah Java/Maven ke stack (mengubah ADR-024) `[DOK: adr/ADR-026-jalur-email-keycloak.md]`.
- Pola penjaga `DEPLOY_ENV`:
  - production tanpa `TRUSTED_PROXY_CIDRS` gagal start (KEPUTUSAN-403);
  - `SES_LOCAL_ENDPOINT` hanya diterima bila `DEPLOY_ENV=local` `[DOK: apps/rs-backend/src/config.rs]`.
- Pemaketan tema Keycloak untuk production (JAR di `providers/`) memang belum ditetapkan, sehingga image Keycloak kustom kemungkinan dibutuhkan juga untuk tema `[DOK: infra/keycloak/README.md]`.

## Keputusan

### 1. Langkah OTP berada di alur Keycloak lewat autentikator kustom yang tipis (opsi A2) `[KEPUTUSAN-460]`

- Provider Java `rs-wa-otp-form` (autentikator) dan `rs-wa-enroll` (required action) dalam satu JAR, hanya dipasang di alur realm `korporat`.
- Autentikator **tidak** membangkitkan kode dan tidak memegang kredensial pengirim. Isi dari `authenticate()`:
  - meminta tantangan ke `otp-relay` (butir 3);
  - menyimpan MAC kode, salt, waktu kedaluwarsa, dan penghitung percobaan di catatan sesi autentikasi Keycloak (nama API diverifikasi saat task `[TIDAK DIKETAHUI]`). Catatan itu diperlakukan seperti kode: hanya di memori sesi autentikasi, tidak boleh dipersistenkan ke database atau dicatat di log. Apakah Keycloak 26.8.2 mempersistenkan catatan sesi autentikasi `[TIDAK DIKETAHUI]`; diuji (dump database dan pencarian log sesudah tantangan), dan bila ya, pilihan pengganti ditetapkan di A32-T4;
  - menampilkan formulir.
- Isi dari `action()`:
  - menormalkan masukan, menghitung MAC, dan membandingkan dalam waktu konstan;
  - bila cocok: menghapus catatan (sekali pakai), lalu `success()`;
  - bila salah: `failureChallenge()`, sehingga kegagalan masuk deteksi brute force realm `[SRC-993]` `[SRC-273]`.
- Kode otorisasi OIDC baru terbit setelah seluruh alur dan required action selesai; perilaku ini sudah diuji untuk OTP dan Configure OTP di realm `pasien`/`admin` `[DOK: infra/keycloak/README.md]` dan diuji ulang untuk provider baru. Karena itu sesi Keycloak, token, dan sesi BFF hanya ada setelah faktor kedua lolos. Tidak ada token "setengah MFA" yang perlu dijaga aplikasi.
- Perbandingan opsi ada di bagian "Opsi yang dibandingkan".

### 2. Alur realm `korporat` `[KEPUTUSAN-460]` `[KEPUTUSAN-466]`

- **Login browser:** password (REQUIRED). Subalur faktor kedua (`conditional-user-configured`) memuat `rs-wa-otp-form` dan `auth-recovery-authn-code-form` sebagai ALTERNATIVE (kode pemulihan hanya berlaku bagi HR yang membuatnya, butir 7.1). Bila atribut `rs_wa_reset_pending` ada (pemulihan lewat tiket, butir 7.3), `rs-wa-otp-form` menolak login password-saja yang tidak berasal dari tautan pemulihan (gagal-tertutup). Bila belum ada faktor kedua, `rs-wa-otp-form` menjadi REQUIRED dengan *user setup*, sehingga `rs-wa-enroll` dipaksa sebelum kode otorisasi terbit (pola `rs-browser-otp-default` sekarang).
- **Atur ulang password:** `rs-wa-otp-form` atau kode pemulihan sebelum `reset-password` (ASVS 6.4.3, KEPUTUSAN-199 tetap berlaku).
- **Direct grant:** tetap mati. Alur cadangan tetap mewajibkan OTP, sehingga HR tanpa kredensial OTP Keycloak tidak dapat masuk lewat jalur itu walaupun klien diaktifkan tanpa sengaja. Perilaku ini diuji `keycloak-check`.
- Required action `CONFIGURE_TOTP` dimatikan **hanya** di realm `korporat`. Realm lain tidak berubah.
- **`amr` (pertahanan berlapis, bersyarat verifikasi):**
  - nilai referensi `pwd` pada eksekusi password, `wa_otp` pada `rs-wa-otp-form`, dan `rcode` pada kode pemulihan; `oidc-amr-mapper` dipasang di klien `rs-korporat-bff` `[SRC-994]` `[SRC-995]`;
  - backend menolak token `korporat` yang `amr`-nya tidak memuat `pwd` dan salah satu dari `wa_otp`/`rcode`;
  - apakah `amr` memuat faktor kedua pada login pertama (saat faktor dipasang lewat required action) `[TIDAK DIKETAHUI]`, diuji `keycloak-check`. Bila tidak, pilihannya: login ulang paksa sesudah pendaftaran nomor, atau tetap memakai kontrol kompensasi TD-27. Pilihan ditetapkan saat task dengan review keamanan.

### 3. `otp-relay` Rust dan port pengirim `[KEPUTUSAN-461]`

**Bentuk.**
- Subcommand baru `rs-backend otp-relay`, pola `email-relay` ADR-026:
  - satu instans per host Keycloak (app-1, app-2, stg-app);
  - jaringan Docker khusus yang hanya berisi relay dan Keycloak;
  - tanpa port publik dan tanpa port di jaringan antar-host;
  - TLS dengan CA internal, ditambah token pembawa (bearer) dari OpenBao di kedua sisi;
  - identitas OpenBao sendiri, yang hanya dapat membaca kredensial agen WA (production) atau kode tetap (staging);
  - tanpa akses database.
- Relay **membangkitkan** kode dan mengirimnya lewat adaptor. Keycloak hanya menerima `mac = HMAC-SHA256(K_host, salt || challenge_id || account_ref || kode)` dengan `salt` acak 128 bit per tantangan, tidak pernah teks kodenya `[KEPUTUSAN-474]`.
- **Kunci MAC `K_host`.**
  - 256 bit acak per host Keycloak (app-1, app-2, stg-app), dibuat dan disimpan di KV OpenBao pada path yang hanya dapat dibaca identitas `otp-relay` dan pemasok konfigurasi Keycloak host itu (ADR-013, ADR-025 butir 7). Bukan salt: salt dikirim bersama MAC dan tidak rahasia.
  - Cara Keycloak menerima nilainya saat start (misalnya berkas atau variabel lingkungan yang dirender agen OpenBao) `[TIDAK DIKETAHUI]`, ditetapkan di A32-T2/A32-T9. Tidak di kode, image, berkas yang di-commit, atau cadangan database.
  - Diputar terjadwal dan saat dugaan bocor. `key_id` ikut di catatan, sehingga dua kunci diterima selama 10 menit.
  - Batas jujur: ruang kode 31^6 ≈ 2^29,7, jadi **tanpa `K_host`** nilai MAC tidak dapat dibalik secara offline, tetapi **pemegang `K_host` dan catatan sesi** (kompromi JVM Keycloak atau relay) memulihkan kode dalam hitungan detik. Selama 5 menit umur kode, nilai MAC setara kode bagi pemegang `K_host`. Karena itu `K_host` dan catatan dijaga seperti kode.

**Port pengirim** (`OtpSender`, trait Rust). Dipilih lewat `OTP_SENDER` tanpa nilai bawaan:
- `static`: kode = nilai konfigurasi, tanpa pengiriman. Hanya `local`/`staging` (butir 4).
- `whatsapp`: kode acak dari CSPRNG OS, dikirim lewat agen WA. Agen dibangun kemudian `[USER]`. Penyedia: BSP terdaftar Meta (P32-2 A); nama, biaya, dan kontrak `[TIDAK DIKETAHUI]`.
- Jenis adaptor berupa enum di kode. Nilai lain membuat start gagal.

**Kontrak Keycloak → relay** (nama final ditetapkan saat task):

| Butir | Isi |
| --- | --- |
| Permintaan | `POST /otp/v1/challenges`, header `Authorization: Bearer …` dan `Idempotency-Key` = SHA-256 atas (ID sesi autentikasi, tab, nomor urut tantangan). Badan: `realm`, `account_ref` (ID pengguna Keycloak), `recipient` (E.164), `purpose` (`login` / `enroll` / `change` / `reset`), `locale` |
| Sukses | `201`: `salt`, `mac`, `key_id`, `expires_in_s`, `sender_mode` (`static` / `whatsapp`), `recipient_hint` (4 digit terakhir) |
| Gagal | `401` token salah; `422` nomor ditolak permanen (format, bukan pengguna WhatsApp bila agen dapat mengetahuinya); `429` batas laju, dengan `retry_after_s`; `503` agen gagal sementara atau batas waktu |
| Batas waktu | Keycloak → relay 7 detik. Gagal atau batas waktu menghasilkan pesan umum "Kode belum dapat dikirim, coba lagi", **tanpa jalan pintas** melewati faktor kedua (gagal-tertutup). Kode pemulihan tersedia hanya bagi HR yang membuatnya (butir 7.1) |
| Idempotensi | Kunci yang sama dalam 10 menit mengembalikan hasil sama tanpa kirim ulang (cache memori per host). Kirim ulang oleh pengguna = tantangan baru dengan kunci baru |

**Kontrak relay → agen WA** (adaptor `whatsapp`; agen dibangun kemudian):

| Butir | Isi |
| --- | --- |
| Permintaan | `POST <agen>/v1/otp-messages`, autentikasi klien (token atau mTLS, ditetapkan bersama agen), `Idempotency-Key` diteruskan dari relay. Badan: **hanya** `to` (E.164) dan `code`. Templat autentikasi, bahasa, dan masa berlaku (5 menit) tetap di konfigurasi agen. Tidak ada `account_ref`, realm, `purpose`, nama, atau teks lain, sehingga BSP tidak dapat mengaitkan kode dengan identitas atau tujuan |
| Hasil | `202 accepted` + `message_id` = diterima penyedia; `4xx` permanen (nomor tidak valid, bukan pengguna WhatsApp, templat ditolak); `429`/`5xx`/batas waktu = sementara |
| Batas waktu | Sambung 1 detik, total 5 detik. Tanpa coba ulang otomatis di relay; pengguna menekan "Kirim ulang" setelah jeda |
| Idempotensi | Agen wajib menolak kiriman ganda dengan kunci yang sama selama minimal 15 menit |
| Larangan | Tanpa jatuh ke SMS (PSTN memicu ASVS 6.6.1 dan status *restricted* NIST `[SRC-992]` `[SRC-990]`). Agen tidak mencatat kode atau nomor lengkap di log, tidak menyimpan kode setelah dikirim, dan tidak memuat tautan di pesan |
| Teks pesan | Memakai templat autentikasi WhatsApp: teks isi preset tetap, pernyataan keamanan dan peringatan kedaluwarsa opsional, tombol salin kode `[SRC-999]`. Kalimat bebas seperti "jangan berikan kode kepada petugas Ruang Sehat" tidak dapat dikarang di pesan; larangan itu ditegakkan di layar kode Keycloak dan SOP support. Teks final dan bahasa `[TIDAK DIKETAHUI]`, mengikuti templat yang disetujui Meta lewat BSP. Adaptor `static` tidak mengirim pesan |
| Status kirim | Laporan terkirim/dibaca/gagal dari penyedia tidak masuk ke relay di Fase 1 (relay tanpa port masuk). Metrik ada di agen |

### 4. Penjaga kode tetap dan masa transisi demo `[KEPUTUSAN-462]`

- **Lapis 1, relay (pola KEPUTUSAN-403):** `DEPLOY_ENV=production` dengan `OTP_SENDER=static` membuat `OtpRelayConfig::load` gagal. Pesannya hanya menyebut nama variabel. `OTP_SENDER` tanpa nilai juga gagal start (tidak ada bawaan `static`).
- **Lapis 2, Keycloak (independen dari relay):** SPI autentikator menerima `deploy-env` dari variabel lingkungan Keycloak. Nilai kosong, tidak ada, atau tidak dikenal diperlakukan sebagai `production` (gagal-tertutup): hanya `local` dan `staging` yang ditulis eksplisit mengizinkan `sender_mode=static`. Bila `production` dan relay menjawab `sender_mode=static`, tantangan ditolak dan login gagal-tertutup. Apakah galat di `init()` factory menghentikan start Keycloak `[TIDAK DIKETAHUI]`; penjaga runtime ini berlaku apa pun hasilnya.
- **Lapis 3, deploy:**
  - checklist dan skrip deploy production menolak berkas lingkungan yang memuat `OTP_SENDER=static` atau variabel kode tetap;
  - policy OpenBao production tidak punya path kode tetap;
  - `keycloak-check` mode production memeriksa drift.
- **Nilai kode tetap:**
  - tidak pernah di kode sumber atau berkas yang di-commit;
  - lokal di `.env.local` (tidak di-commit; `.env.example` hanya memuat nama variabel tanpa nilai);
  - staging di KV OpenBao yang hanya dapat dibaca identitas `otp-relay` staging;
  - format divalidasi `^[A-Z0-9]{6}$`, sehingga contoh user "A1B2C3" sah.
- **"Demo" = lingkungan `local` dan `staging`**, yang hanya berisi data sintetis (ADR-019 butir 1, ADR-025 butir 1). Tafsiran ini dikonfirmasi user: P32-6 = A `[USER]` 2026-11-28.
- Alur di demo sama dengan production: pendaftaran nomor, layar kode, batas percobaan, dan penawaran kode pemulihan. Hanya pengirimannya yang tidak ada. Kode tetap tetap sekali pakai per tantangan.

### 5. Parameter OTP `[KEPUTUSAN-463]`

| Parameter | Nilai | Dasar |
| --- | --- | --- |
| Panjang dan alfabet | 6 karakter dari 31 simbol: angka 2–9 dan huruf A–Z tanpa I, L, O (tanpa 0/1/I/L/O yang mirip). 31^6 ≈ 2^29,7 | ASVS 6.5.4 ≥ 20 bit `[SRC-992]`; NIST ≥ 6 digit desimal atau setara `[SRC-990]`; bentuk alfanumerik mengikuti contoh user `[USER]` |
| Normalisasi masukan | Pangkas spasi dan tanda hubung, ubah ke huruf besar | Kode diketik HR dari ponsel ke web |
| Umur | 5 menit | ≤ 10 menit (NIST §3.1.3.2, ASVS 6.5.5) `[SRC-990]` `[SRC-992]` |
| Sekali pakai | Catatan dihapus saat sukses. Tantangan baru membatalkan yang lama. Hanya berlaku di sesi autentikasi asalnya | ASVS 6.5.1, 6.6.2 `[SRC-992]` |
| Percobaan per kode | Maksimal 5 salah, lalu kode batal dan HR meminta kode baru | ASVS 6.6.3 `[SRC-992]` |
| Penguncian akun | Setiap kode salah dihitung deteksi brute force realm (5 gagal, kunci 15 menit, ADR-011 butir 4). Penghitung per pengguna tidak direset oleh kode baru Realm `korporat` mengisi `maxTemporaryLockouts` lebih dari 0 (di `infra/keycloak/realms/korporat.json` kini 0; nilai awal usulan 3, ditetapkan di A32-T6 dan diverifikasi artinya di `[SRC-273]`): sesudah itu akun terkunci sampai dibuka lewat jalur pemulihan (butir 7.3), sehingga kegagalan berulang tidak berlangsung tanpa batas. NIST §3.2.2 membatasi kegagalan berturut-turut dan mengizinkan batas lebih rendah `[SRC-990]` (angka batas atas tidak dikutip di sini). Kompromi: pemegang password dapat mengunci korban (penolakan layanan); korban memakai jalur pemulihan. Cara membuka kunci oleh `worker` `[TIDAK DIKETAHUI]`, diverifikasi di task dan masuk amandemen ADR-030 bila butuh jenis job baru | NIST §3.1.3.2, §3.2.2 `[SRC-990]`; `[SRC-273]` |
| Jeda kirim ulang | 60 detik (dicek di Keycloak sebelum memanggil relay) | Keputusan teknis |
| Batas laju di relay | Per akun: 5 kirim per 15 menit. Per nomor: 10 kirim per 24 jam. Pendaftaran/ubah nomor: 3 nomor berbeda per akun per 24 jam. Anggaran global per menit dapat dikonfigurasi (biaya pesan). Kunci di-hash, di memori per host, sehingga anggaran efektif ±2 kali di dua host. Saat batas tercapai: `429`, tanpa kirim | ASVS 6.6.3 `[SRC-992]`; pola ADR-026 butir 4; angka adalah usulan teknis |
| Penyimpanan | `HMAC-SHA256(K_host, salt || challenge_id || account_ref || kode)` di catatan sesi autentikasi Keycloak, tidak dipersistenkan dan tidak dicatat di log; tidak di database aplikasi | Keycloak tidak pernah menyimpan teks kode; kunci `K_host` di OpenBao (butir 3) |
| Log | Kode tidak pernah dicatat di relay, Keycloak (detail event), audit, atau security log. Nomor hanya tampil sebagai 4 digit terakhir | ADR-005, ADR-010 |

### 6. Nomor WhatsApp HR `[KEPUTUSAN-464]` `[KEPUTUSAN-471]`

**Pengumpulan (P32-4 = B `[USER]`).**
- Nomor diisi di tiga tempat:
  1. formulir `/daftar` (pendaftar mengisi nomornya sendiri, bagian data diri HR);
  2. formulir "Undang HR" (HR pengundang mengetik nomor rekan);
  3. layar `rs-wa-enroll` pada login pertama, **bila nomor belum ada**. Ini jalur cadangan untuk HR yang dibuat Verifikator (US-1903), karena P32-4 hanya menyebut dua formulir pertama. Menambah isian di formulir Verifikator ditanyakan di P32-12.
- Nomor tetap **dibuktikan** dengan kode saat login pertama (NIST §3.1.3.3: menetapkan nomor = mengikat autentikator baru `[SRC-990]`). HR dapat mengoreksi nomor sebelum verifikasi.
- Tidak ada pesan WhatsApp yang dikirim saat pendaftaran atau undangan. Kode pertama baru dikirim setelah pemilik email mengatur password dan masuk. Jadi formulir publik tidak dapat dipakai untuk mengirim pesan langsung ke nomor korban. Penyerang tetap dapat mendaftar dengan emailnya sendiri dan nomor korban, lalu memicu pengiriman kode ke nomor itu: ini dibatasi batas laju per nomor dan per akun (butir 5) dan tercatat di tabel risiko.

**Aturan nomor belum terverifikasi (invarian, wajib diuji).** Nomor yang diketik di formulir, atau diketik ulang di layar koreksi, tetapi belum dibuktikan dengan kode:
1. hanya boleh menjadi penerima tantangan `purpose=enroll` di `rs-wa-enroll`. Tidak untuk `login`, `change`, `reset`, pemberitahuan, pemasaran, atau tujuan lain;
2. tidak dihitung sebagai faktor kedua: `configuredFor()` hanya melihat `rs_wa_phone` (terverifikasi), tidak pernah `rs_wa_phone_pending`. Selama belum terverifikasi, login password-saja ditolak dengan petunjuk membuka tautan di email (pendaftaran nomor hanya di sesi tautan), bukan diarahkan ke layar kode login;
3. tidak tampil di respons API, daftar anggota HR, backoffice, audit, atau log. Pengundang tidak dapat membacanya kembali setelah dikirim; yang tampil paling banyak 4 digit terakhir (ADR-005, ADR-010);
4. tidak dipakai untuk pencarian, pencocokan, atau uji keunikan lintas akun (menghindari enumerasi nomor);
5. dibuang bersama akun menunggu yang dibatalkan atau digagalkan lewat `idp_delete_orphan` (ADR-030), dan dihapus dari tabel penyangga setelah dipakai (di bawah).

**Validasi dan normalisasi (P32-5 = A `[USER]`).**
- Satu fungsi murni di backend dipakai `api` (kedua formulir) dan `otp-relay`. `rs-wa-enroll` (Java) memakai implementasi sendiri, dan ketiganya diuji dengan **satu berkas vektor uji bersama** agar tidak melenceng.
- Aturan:
  - buang spasi, `-`, `.`, `(`, `)`;
  - terima awalan `+62`, `62`, atau `0`; awalan `0` diganti `+62`;
  - hanya digit sesudah `+`; awalan negara lain (`+65`, `00…`, dst.) ditolak dengan pesan "Fase 1 hanya nomor Indonesia (+62)";
  - panjang mentah paling banyak 20 karakter (usulan teknis);
  - hasil disimpan sebagai E.164 `+62…`.
- Batas panjang digit dan awalan nomor seluler Indonesia (misalnya digit pertama sesudah `+62` harus `8`) **belum ditetapkan**. Panjang maksimum E.164 dan pola nomor seluler Indonesia `[TIDAK DIKETAHUI]`; sumber resmi (ITU-T E.164, rencana penomoran Indonesia) dibuka saat A32-T12 dan dicatat di `docs/sources.md`. Sampai itu, regex final belum dikunci.
- Kegagalan validasi: 422 per field (KU-01), tanpa menyebut akun atau perusahaan lain. Nomor tidak diperiksa apakah terdaftar di WhatsApp sebelum kode dikirim.
- Tidak ada syarat keunikan. Nomor yang sama boleh dipakai beberapa akun HR (P32-13); batas laju per nomor di relay tetap berlaku.

**Jalur data nomor belum terverifikasi (revisi atas ADR-030 dan ADR-031).**
1. Dalam satu `in_audited_tx` yang sama dengan niat (ADR-030 butir 2), `api` menormalisasi nomor dan mengenkripsinya (amplop ADR-013, kunci Transit khusus `hr-phone`: `api` hanya `encrypt`, `worker` hanya `decrypt`, pola ADR-030 butir 4 R2) ke tabel penyangga berumur pendek `corporate_hr_phone_pending` (nama sementara). Kelas tabel menurut ADR-008 ditetapkan bersama DBA `[TIDAK DIKETAHUI]`. Muatan job tetap hanya ID niat.
2. `worker` pada `idp_create_user` mendekripsi dan menulis atribut `rs_wa_phone_pending` **saat membuat** pengguna. Ini pembuatan, bukan perubahan atribut pengguna yang sudah ada, sehingga larangan ADR-030 butir 3 tidak dilanggar. Sesudah berhasil, baris ciphertext dihapus. Baris juga dihapus bila niat gagal, batal, atau kedaluwarsa (batas umur ditetapkan di task, usulan 48 jam).
3. Bila email sudah terdaftar (jawaban seragam anti-enumerasi, US-1901 AC4), tidak ada pengguna dibuat dan nomor dibuang.
4. Risiko sisa: ciphertext ikut WAL dan PITR 35 hari. Risiko ini sama dengan R2 di ADR-030, dan crypto-shredding efektif hanya bila semua salinan sudah habis masa simpannya (TD-11).

**Penyimpanan di Keycloak.**
- Dua atribut user profile, keduanya E.164 +62, izin lihat dan ubah hanya Admin (pola `rs_intent_id` `[SRC-845]`), tanpa mapper token:
  - `rs_wa_phone_pending`: belum terverifikasi. Ditulis `worker` saat pembuatan, atau `rs-wa-enroll` saat koreksi;
  - `rs_wa_phone`: terverifikasi. **Hanya** `rs-wa-enroll` yang menulisnya, setelah kode benar. Pada saat itu `rs_wa_phone_pending` dihapus.
- Nomor terverifikasi diperlakukan sebagai pengikatan autentikator, setara rahasia TOTP yang selama ini disimpan Keycloak. ADR-011 butir 6 tetap terpenuhi. Relay tidak punya akses database.
- Database `keycloak` berada di instans (a) di Indonesia dan ikut PITR 35 hari (ADR-011 butir 2, ADR-025 butir 5).

**Pendaftaran nomor di login pertama (`rs-wa-enroll`).**
1. Di sesi yang dibuka tautan aktivasi (password diatur, lalu langsung `rs-wa-enroll`; KEPUTUSAN-303), layar menampilkan nomor yang sudah diisi (lengkap, karena HR adalah pemiliknya dan sudah terautentikasi password) dengan pilihan "Kirim kode" atau "Ubah nomor". Bila kosong, formulir nomor kosong. Teks layar: "Nomor ini didaftarkan atas nama Anda. Bukan nomor Anda? Ubah sekarang." Ini menutup kasus pengundang salah ketik atau menanam nomor lain.
2. Koreksi: nomor baru divalidasi dan menggantikan `rs_wa_phone_pending`. Batas 3 nomor berbeda per akun per 24 jam (butir 5).
3. Kode dikirim (`purpose=enroll`) dan harus dimasukkan. Bila benar: `rs_wa_phone` ditulis, `rs_wa_phone_pending` dihapus, E-04.
4. Layar kode pemulihan ditawarkan, tidak dipaksa (butir 7).

**Pendaftaran nomor hanya di sesi tautan.**
- Pendaftaran nomor pertama hanya boleh berjalan di sesi yang dibuka tautan aktivasi (atau tautan pemulihan, butir 7.3), bukan di login password biasa. Tanpa aturan ini, pemegang password saja dapat mengikat nomornya sendiri.
- Akun yang password-nya sudah diatur tetapi nomornya belum terverifikasi (HR berhenti di tengah, atau tautan kedaluwarsa) diperlakukan seperti `rs_wa_reset_pending`: login password-saja ditolak dengan petunjuk membuka tautan di email. Tautan baru dikirim lewat jalur kirim ulang aktivasi yang ada (`api`, niat, `idp_send_activation`, batas laju US-101 AC2b), dengan prasyarat worker diperluas: pengguna belum punya `rs_wa_phone` terverifikasi. Apakah autentikator dapat mengirim tautan itu sendiri lewat email Keycloak `[TIDAK DIKETAHUI]`.
- "Ubah nomor WhatsApp" (AIA) hanya tersedia bila nomor terverifikasi sudah ada. Akun tanpa nomor terverifikasi tidak punya jalur itu.
- Sisa risiko yang diterima: siapa pun yang menguasai email pada aktivasi pertama, **dan pada setiap pengiriman ulang tautan untuk akun yang nomornya belum terverifikasi**, dapat mendaftarkan nomornya. Apakah tautan aktivasi (termasuk kirim ulang) juga mengatur password, atau hanya `rs-wa-enroll` bila password sudah ada, `[TIDAK DIKETAHUI]`: ADR-030 butir 4 R1 mengusulkan tautan memuat `VERIFY_EMAIL` dan `UPDATE_PASSWORD`, dan bila itu berlaku kirim ulang berarti pemegang mailbox dapat menetapkan password sekaligus nomor. Kepercayaan awal pada email sama dengan pendaftaran TOTP sebelumnya, dibatasi batas laju kirim ulang (US-101 AC2b) dan verifikasi perusahaan oleh Verifikator. Keputusan isi tautan kirim ulang ditetapkan di A32-T5/A32-T14 dengan review keamanan.

**Biaya pesan dan urutan pembuatan akun.** US-1901 AC1 membuat akun HR saat pendaftaran, sebelum Verifikator menyetujui perusahaan. Akibatnya siapa pun dapat membuat banyak akun dan memicu kode ke nomor sembarang, dibatasi hanya oleh batas laju (3 nomor per akun per 24 jam, 10 per nomor per 24 jam, anggaran global, butir 5). Pilihan A (bawaan draf ini): dipertahankan. Pilihan C: tautan aktivasi dikirim baru sesudah perusahaan Aktif (akun dibuat sesudah persetujuan; mengubah US-1901 AC1, AC5, AC6). Ditanyakan di P32-19.

**Ubah nomor (setelah terverifikasi).**
- Dari member area, tautan "Ubah nomor WhatsApp" memicu `rs-wa-enroll` sebagai AIA.
- Langkahnya:
  1. autentikasi ulang dengan password dan nomor lama, atau kode pemulihan bila HR punya (`getMaxAuthAge` = 0 `[SRC-994]`);
  2. kode dikirim ke nomor baru dan harus dimasukkan;
  3. nomor lama tetap terikat sampai nomor baru terverifikasi (pola H-16 a).
- Sesudahnya: email pemberitahuan ke alamat akun (NIST §4.1.2.1 `[SRC-991]`) dan E-04.
- Mekanisme email pemberitahuan dari required action kustom `[TIDAK DIKETAHUI]`. Bila Keycloak tidak dapat mengirimnya, pemberitahuan dikirim worker dari event yang diambil (jalur email dan jenis job pemberitahuan dibangun di A32-T14).

**Privasi (UU PDP).**
- Tujuan pemrosesan nomor hanya autentikasi (kode masuk dan pemulihan akses): tanpa notifikasi atau pemasaran, sesuai CR-007 untuk notifikasi dan cakupan pengecualian P32-1.
- Nomor rekan yang diketik pengundang adalah data pribadi pihak ketiga. Email aktivasi dan layar `rs-wa-enroll` menyebut tujuan itu. Teks persetujuan pendaftar `/daftar` (menunggu L-20) wajib menyebut pemakaian nomor untuk kode masuk lewat WhatsApp beserta penyedianya `[SRC-15]` (Pasal 21). Teks untuk HR yang didaftarkan orang lain `[TIDAK DIKETAHUI]`, menunggu legal (L-20).
- Nomor terhapus bersama pengguna Keycloak. Prosedur hak subjek data (EP-16) dan inventaris pemrosesan atau DPIA ditambah butir ini.
- Transfer ke penyedia WhatsApp = butir 10.

### 7. Faktor alternatif dan pemulihan (revisi KEPUTUSAN-265) `[KEPUTUSAN-465]` `[KEPUTUSAN-470]`

#### 7.1 Kode pemulihan bersifat opsional (P32-3 = B `[USER]`)

- Sesudah nomor pertama terverifikasi, layar menawarkan "Buat kode pemulihan (disarankan)" dan "Lewati". Memilih lewati tidak menyisakan kredensial apa pun dan tidak menghalangi login.
- HR dapat membuat atau membuat ulang 12 kode pemulihan bawaan Keycloak kapan saja lewat AIA sesudah autentikasi ulang, dan E-04 tercatat. Apakah required action bawaan kode pemulihan dapat dipasang sebagai opsional dari `rs-wa-enroll`, atau harus lewat tombol di member area `[TIDAK DIKETAHUI]`, diverifikasi saat task (KEPUTUSAN-203).
- Subalur faktor kedua tetap memuat kode pemulihan sebagai ALTERNATIVE, tetapi hanya berlaku bagi HR yang punya kodenya. Perilaku ALTERNATIVE untuk pengguna tanpa kredensial itu `[TIDAK DIKETAHUI]`, diuji `keycloak-check`.
- Pengingat di member area selama HR belum punya kode pemulihan: usulan, menunggu P32-15.

**Yang melemah, dan kompensasinya.**
- ASVS 6.4.4 (L2) mensyaratkan pembuktian identitas setara pendaftaran saat faktor hilang `[SRC-276]`. Persyaratan itu tidak mewajibkan kode pemulihan, jadi kode opsional tidak melanggarnya selama jalur pemulihan lewat pembuktian identitas ada (butir 7.3). NIST §4.2 menyebut kode pemulihan tersimpan sebagai salah satu metode pemulihan dan mewajibkan pemberitahuan setiap pemulihan `[SRC-991]`; pemberitahuan itu dipenuhi lewat butir 7.3. Catatan: ADR ini sebelumnya memakai ASVS 6.4.4 dan NIST §4.2 sebagai dasar kode pemulihan wajib. Dasar itu hanya mendukung adanya jalur pemulihan, bukan kewajiban kode.
- Yang melemah:
  1. **Ketersediaan.** HR tanpa kode pemulihan yang kehilangan nomor harus menunggu proses manual dan masa tunggu 24 jam. Bila agen WhatsApp atau BSP gagal, HR tanpa kode tidak dapat masuk sama sekali (gagal-tertutup, tanpa jalan pintas). Dengan kode wajib, HR tetap dapat masuk.
  2. **Kekuatan pemulihan.** Kode pemulihan adalah faktor kepemilikan yang disimpan sendiri oleh HR. Pemulihan manual bergantung pada manusia (petugas dan HR rekan) dan menjadi sasaran rekayasa sosial (tabel risiko).
  3. **Beban operasional.** Tim support dan Verifikator menjadi jalur kritis kepercayaan, dengan beban yang naik bersama jumlah HR.
- Kompensasi: penawaran kode pemulihan yang jelas di langkah pendaftaran dan pengingat (P32-15); jalur pemulihan manual berlapis (butir 7.3); runbook gangguan BSP; metrik dan peringatan agen WA. Penerimaan risiko ini adalah keputusan user (P32-3 B).

#### 7.2 Skenario

| Keadaan | Jalan |
| --- | --- |
| Ganti ponsel, nomor sama | Tanpa reset: kode tetap terkirim ke nomor itu |
| Nomor hilang, HR punya kode pemulihan | Masuk dengan kode pemulihan, lalu "Ubah nomor WhatsApp" (butir 6). Layar menawarkan "Nomor saya tetap", karena kode pemulihan juga dipakai saat WhatsApp sedang tidak dapat diakses |
| Nomor hilang, tanpa kode pemulihan (atau kodenya juga hilang) | Pemulihan lewat tiket support (butir 7.3), Fase 1 |
| WhatsApp atau BSP gagal sementara | HR dengan kode pemulihan masuk dengan kode itu. HR tanpa kode menunggu layanan pulih; tidak ada jalan pintas dan tidak ada reset otomatis |
| HR tidak punya akses ke email akun | Di luar alur ini: perubahan email HR korporat belum ada `[TIDAK DIKETAHUI]`. Dijawab gagal-tertutup; jalan keluarnya penunjukan HR baru oleh Verifikator |
| Perusahaan tanpa HR aktif | Bukan reset. Verifikator menunjuk HR baru dengan bukti kewenangan baru (pola US-1903) |

#### 7.3 Pemulihan lewat tiket support (dimajukan ke Fase 1, P32-7 = B `[USER]`)

**Pengajuan.**
- HR yang tidak dapat masuk mengajukan **permintaan pemulihan akses** lewat halaman publik `/bantuan-akses` di `web/korporat`, memakai pola ADR-031: tanpa sesi, tanpa cookie, `POST /v1/public/corporate-access-recovery-requests` pada asal yang sama (amandemen ADR-031: rute Caddy ketiga dengan rewrite ke path tetap dan batas badan 16 KB), hanya `application/json`, jawaban `202` seragam, penyamaan waktu respons, batas laju per sumber.
- Pengajuan juga dapat dicatat Layanan Pelanggan bila HR menghubungi kanal resmi (telepon atau email Layanan Pelanggan; alamat dan nomornya `[TIDAK DIKETAHUI]`, P32-9). Petugas memakai layar backoffice yang sama, dan kanal asal tercatat.
- Isian: email akun HR, nama lengkap, alasan (nomor hilang, ganti nomor, lainnya), keterangan opsional (paling banyak 500 karakter, tanpa kolom kode atau password). Isian tidak memuat nomor baru. Nomor baru hanya diketik HR sendiri di `rs-wa-enroll` sesudah pemulihan, jadi petugas **tidak pernah menetapkan nomor**.
- Permintaan hanya dibuat bila email cocok dengan akun HR aktif milik perusahaan berstatus Aktif di database aplikasi. Jawaban tetap seragam untuk akun yang ada, tidak ada, atau nonaktif (pola US-101 AC3).
- Status: Baru, Menunggu verifikasi, Terverifikasi, Disetujui (masa tunggu), Dieksekusi, Ditolak, Dibatalkan, Kedaluwarsa, Ditutup (penyalahgunaan; hanya Verifikator). Riwayat status append-only (ADR-004, pola `status-flow`). Permintaan kedaluwarsa dalam 7 hari bila belum disetujui (usulan, P32-10).

**Peran (hak paling kecil, tanpa sub-peran baru; P32-8).**

| Sub-peran | Izin | Tidak boleh |
| --- | --- | --- |
| Layanan Pelanggan | `hr_recovery.verify`: melihat antrean, menghubungi pemohon, mencatat hasil verifikasi (jalur, waktu, hasil, nama penerima telepon) | Menyetujui atau mengeksekusi reset; melihat nomor WhatsApp lengkap, kode, atau password; melihat dokumen perusahaan |
| Verifikator | `hr_recovery.approve` dan `hr_recovery.reject`: menyetujui atau menolak permintaan yang sudah terverifikasi; menutup permintaan sebagai penyalahgunaan; membaca data perusahaan dan bukti kewenangan untuk pencocokan (setiap pembukaan dokumen memancarkan E-05, ADR-012) | Menyetujui permintaan yang ia sendiri verifikasi; menetapkan nomor |
| Super Admin | Tidak ada izin pemulihan (KEPUTUSAN-265: tanpa jalur Super Admin) | Semuanya |
| Auditor | Baca audit saja | Semuanya |

- **Pemisahan tugas:** pencatat verifikasi (Layanan Pelanggan) harus berbeda dari penyetuju (Verifikator), dan keduanya bukan pemohon. Permintaan atas akun HR yang terkait staf sendiri ditolak (PRD US-1904 AC9). API yang menegakkan ini, bukan hanya UI.
- Pembagian ini menjaga ketentuan `[USER] C21-P4` (4a) bahwa keputusan reset ada pada Verifikator, sekaligus memakai tim support untuk penerimaan dan verifikasi seperti jawaban user P32-3. Apakah user menghendaki pembagian dua orang ini ditanyakan di P32-8.

**Verifikasi identitas (sebelum reset).** Email atau isian formulir saja tidak pernah cukup, dan telepon balik saja (J2) tidak pernah cukup sendirian. Dua cara yang sah, urut dari yang terkuat:
- **J1, konfirmasi HR aktif lain di perusahaan yang sama (utama).** Sistem mengirim email dan menampilkan pemberitahuan di member area kepada semua HR aktif lain. Satu HR aktif lain, setelah masuk dengan MFA-nya sendiri dan autentikasi ulang (`getMaxAuthAge` 0), membuka "Permintaan pemulihan rekan", melihat nama pemohon, ID permintaan, dan waktu, lalu menekan Konfirmasi atau Tolak. Layar memuat instruksi: "Hubungi rekan Anda langsung lewat kanal yang Anda kenal (telepon atau pesan internal) dan pastikan ia sendiri yang mengajukan permintaan ini. Jangan konfirmasi hanya karena menerima email ini." Petugas tidak dapat mengonfirmasi atas nama HR. Satu penolakan oleh HR aktif lain menghentikan permintaan (ditolak, dan Verifikator ditandai untuk menyelidiki).
- **J3, jalur tanpa konfirmasi rekan** (HR tunggal, atau perusahaan banyak HR yang tidak mendapat J1 dalam 48 jam, usulan P32-10). Semua syarat berikut harus terpenuhi:
  1. J2 berhasil: Layanan Pelanggan menelepon balik ke **nomor telepon kantor pada data perusahaan yang sudah disetujui Verifikator** (US-1901 `office_phone`), bukan nomor yang diberikan pemohon, dan mencatat nama penerima, waktu, dan hasil terhadap ID permintaan. Nomor kantor yang diubah lewat koreksi data perusahaan dalam 30 hari terakhir (usulan) tidak dipakai; permintaan dieskalasi ke Verifikator;
  2. nama pemohon sama dengan pendaftar yang bukti kewenangannya sudah ada di berkas perusahaan (US-1901 AC1), dicocokkan Verifikator. Setiap Verifikator membuka bukti kewenangan itu lewat URL berumur pendek (ADR-018) dengan event audit E-05 (ADR-012);
  3. masa tunggu 24 jam.

  Perusahaan banyak-HR tanpa J1 diperlakukan sama dengan HR tunggal: J2 tanpa syarat 2 **tidak** cukup (P32-18). Bila syarat 2 tidak terpenuhi (misalnya HR tunggal adalah HR yang diundang, tanpa bukti kewenangan atas namanya), permintaan **ditolak** dan perusahaan diarahkan ke penunjukan HR baru lewat Verifikator dengan bukti kewenangan baru. Verifikator hanya dapat menolak atau mengarahkan ke penunjukan itu, dan tidak dapat menyetujui tanpa J1 atau syarat 2. Tidak ada jalur lain: tidak ada izin Super Admin, tidak ada konfirmasi hanya lewat telepon tanpa pencocokan berkas, tidak ada pengecualian "mendesak" (P32-14). Telepon tidak terjawab, berkas tidak cocok, atau salah satu langkah tidak tercatat berarti permintaan tidak maju dan akhirnya kedaluwarsa.
- Yang bukan bukti identitas: informasi yang disebut penelepon sendiri, nada suara, desakan waktu, chat atau email dari alamat yang tidak ada di berkas. Petugas dilarang meminta atau menerima kode OTP, password, atau kode pemulihan, dan layar petugas tidak punya kolom untuk itu.

**Masa tunggu, pembatalan, dan eksekusi.**
1. Setelah Verifikator menyetujui, masa tunggu 24 jam dimulai (usulan, P32-10). Dalam masa itu terkirim: email ke alamat akun (dengan tautan sekali pakai "Bukan saya, batalkan"), email ke semua HR aktif lain, dan notifikasi di member area. Pembatalan oleh pemilik akun, HR aktif lain, Layanan Pelanggan, atau Verifikator menghentikan permintaan, menandai akun, dan mencatat E-04. Permintaan yang masih terbuka **tidak** mengunci login HR; HR yang menemukan ponselnya lagi dapat masuk dan membatalkan sendiri.
2. Setelah masa tunggu, `worker` menjalankan job `idp_reset_wa_factor`, lalu `idp_send_wa_reenroll` (amandemen ADR-030, di bawah). Hasilnya:
   - atribut `rs_wa_phone` dan `rs_wa_phone_pending` dihapus;
   - kredensial kode pemulihan dihapus;
   - semua sesi HR dicabut;
   - atribut `rs_wa_reset_pending` (berisi ID niat) ditulis;
   - email berisi tautan sekali pakai (berlaku 24 jam, pola KEPUTUSAN-204) dikirim ke alamat akun, dengan required action `rs-wa-enroll`. Tautan **tidak** membawa `UPDATE_PASSWORD` dan tidak menggantikan password: sesi tautan **mewajibkan password saat ini** sebelum `rs-wa-enroll` berjalan.
3. **Selama `rs_wa_reset_pending` aktif**, autentikator menolak (gagal-tertutup) login password-saja yang tidak berasal dari tautan itu, sehingga pemegang password yang bocor tidak dapat mendaftarkan nomor miliknya sendiri. Nomor baru hanya dapat diikat dari sesi tautan, lewat `rs-wa-enroll` dengan verifikasi kode ke nomor baru yang diketik HR. Sesudah itu `rs_wa_reset_pending` dihapus, kode pemulihan baru ditawarkan lagi (opsional), E-04. Bahwa sesi aksi-token Keycloak tidak menerbitkan token tanpa faktor kedua, dan bahwa `amr` memuat `wa_otp` sesudahnya, `[TIDAK DIKETAHUI]`, diuji `keycloak-check` (butir 2).
   Tambahan: `rs-wa-enroll` mode pemulihan menulis `rs_wa_recovery_bound_at` (waktu nomor terikat lewat pemulihan). Selama `rs_wa_reset_pending` aktif dan **7 hari** sesudah `rs_wa_recovery_bound_at` (usulan, P32-10), alur atur ulang password lewat email (`rs-reset-credentials-mfa`) diblokir untuk akun itu. HR yang lupa password dalam masa itu mengajukan permintaan pemulihan baru (jalur support). Tanpa blokir ini, pemegang mailbox yang berhasil mengikat nomor miliknya dapat mengatur ulang password lewat email dan kode WhatsApp miliknya sendiri.
4. Tautan tidak dibuka dalam 24 jam: kedaluwarsa. HR tidak dapat masuk sampai permintaan baru diajukan (gagal-tertutup).

**Pemberitahuan (NIST §4.2 `[SRC-991]`).**
- Email ke alamat akun: permintaan dibuat, disetujui (mulai masa tunggu), dieksekusi, ditolak atau dibatalkan.
- Email ke semua HR aktif lain: permintaan dibuat dan dieksekusi.
- Pemberitahuan ke nomor WhatsApp lama **tidak** dikirim secara bawaan: P32-1 membatasi pengecualian pada kode masuk HR, dan pemberitahuan memakai kategori pesan lain. Mengizinkannya ditanyakan di P32-11.
- Email dikirim lewat jalur ADR-026 dengan jenis job pemberitahuan berisi templat tetap, tanpa data pribadi di muatan job.

**Batas laju dan penyalahgunaan (usulan teknis).**
- Formulir publik: batas per sumber (pola ADR-031) dan per email akun: 1 permintaan terbuka dan 3 permintaan per 30 hari. Sesudahnya hanya Layanan Pelanggan atau Verifikator yang dapat membuka permintaan baru. Permintaan yang ditutup Verifikator sebagai "penyalahgunaan" (status Ditutup) **tidak dihitung** ke kuota akun itu, supaya pihak lain tidak dapat menghabiskan kuota korban dengan permintaan palsu; permintaan palsu dihitung ke batas per sumber saja.
- Per perusahaan: paling banyak 3 permintaan terbuka. Email ke HR lain paling banyak 1 per permintaan.
- Eksekusi `worker`: batas harian per realm (usulan 5) dan peringatan bila terlampaui.
- Anggaran pengiriman email dan WhatsApp global tetap berlaku (butir 5).

**Amandemen ADR-030 (analisis ancaman).** Dua jenis job ditambahkan. Keduanya melanggar larangan ADR-030 butir 3 ("menghapus kredensial", "mengubah atribut pengguna yang sudah ada"), sehingga dibatasi ketat. ADR-030 sendiri mencatat bahwa reset MFA butuh analisis ancaman tersendiri `[DOK: adr/ADR-030-aksi-manage-users-keycloak-di-worker.md]`.

| Jenis | Panggilan Keycloak | Prasyarat yang diperiksa worker | Dampak bila `api` diretas lalu menyisipkan job ini |
| --- | --- | --- | --- |
| `idp_reset_wa_factor` | `PUT users/{id}` (atribut), `DELETE users/{id}/credentials/{credentialId}` untuk kode pemulihan, lalu `POST users/{id}/logout` `[SRC-842]`. Perilaku `PUT users` terhadap atribut yang tidak dikirim `[TIDAK DIKETAHUI]`, diuji | Pengguna ada di realm `korporat` dan `enabled`; catatan persetujuan lengkap (pemverifikasi, penyetuju, pemohon berbeda); masa tunggu lewat; permintaan belum dibatalkan; batas harian belum terlampaui | Faktor kedua beberapa akun terhapus (penolakan layanan, dan penurunan sementara ke satu faktor). Pemegang password yang bocor **tidak** dapat mengikat nomor sendiri karena `rs_wa_reset_pending` dan tautan email (butir 3 di atas). Peringatan lonjakan eksekusi |
| `idp_send_wa_reenroll` | `PUT users/{id}/execute-actions-email` dengan required action `rs-wa-enroll`, `lifespan` 24 jam, ke alamat akun yang tersimpan di Keycloak `[SRC-672]` | `rs_wa_reset_pending` di Keycloak sama dengan ID niat; email tidak diambil dari muatan job | Email aksi terkirim ke pemilik yang sah. Tidak ada pengambilalihan |

- Catatan jujur: prasyarat "catatan persetujuan lengkap" hanya dapat diperiksa di database aplikasi yang dapat ditulis `api`, sehingga tidak berdiri sendiri sebagai kontrol terhadap `api` yang diretas. Kontrol utamanya adalah penahan di sisi Keycloak (`rs_wa_reset_pending` dan tautan ke email pemilik), batas harian, dan peringatan. Jenis job ini wajib review keamanan T1.

**Audit (ADR-005, append-only).**
- E-04 untuk: permintaan dibuat (kanal), verifikasi dicatat (jalur, pencatat), konfirmasi atau penolakan HR rekan, persetujuan atau penolakan (penyetuju), pembatalan (siapa), kedaluwarsa, eksekusi (`worker`), pendaftaran nomor baru pascapemulihan. Aksi petugas yang ditolak (peran salah, pemisahan tugas) tercatat sebagai akses ditolak (KU-03).
- Isi: ID permintaan, ID perusahaan, ID aktor dan perannya, jalur, hasil. **Tidak pernah** kode, password, kode pemulihan, atau nomor lengkap (paling banyak 4 digit terakhir). Catatan bebas petugas dibatasi 500 karakter dan tidak memuat rahasia.
- Event dipancarkan `api` untuk langkah aplikasi, dan oleh event Keycloak yang diambil worker untuk langkah di IdP (pola butir 8).

**Nilai atribut di event Keycloak.**
- Admin event: `adminEventsDetailsEnabled` (opsi "include representation") **false** di realm `korporat`. Berkas realm sudah memuatnya, dan `keycloak-check` menolak realm yang menyalakannya. Bila menyala, representasi pengguna (termasuk `rs_wa_phone`, `rs_wa_phone_pending`) tertulis penuh di admin event `[SRC-846]`.
- Event pengguna seperti `UPDATE_PROFILE` dapat memuat nilai atribut sebelum dan sesudah `[TIDAK DIKETAHUI]`; diuji, dan bila memuat, atribut nomor dikecualikan dari detail atau event itu dimatikan di realm `korporat`.
- Atribut `rs_wa_phone*`, `rs_wa_reset_pending`, `rs_wa_recovery_bound_at` tidak boleh muncul di log atau event apa pun (pemeriksaan pola).

### 8. Audit, log, dan pemantauan `[KEPUTUSAN-467]`

**Audit.**
- E-01/E-02: dari event login Keycloak yang diambil worker (ADR-011 butir 10), dengan detail faktor (`wa_otp` atau `rcode`) tanpa kode dan tanpa nomor.
- E-04: pengikatan nomor pertama, ubah nomor, penghapusan pengikatan (reset lewat tiket, butir 7.3), pembuatan ulang kode pemulihan, dan seluruh langkah pemulihan lewat tiket (butir 7.3).
- Jenis event yang dipancarkan required action atau autentikator kustom dan cara menambah detailnya `[TIDAK DIKETAHUI]`. Pemetaannya ditetapkan per versi saat task (pola ADR-011 butir 10) dan diamati `keycloak-check`.

**Security log (ADR-010).**
- Relay mencatat hasil per tantangan: mode, `purpose`, kelas hasil, latensi, dan ID korelasi. Tidak mencatat kode, nomor, atau badan permintaan.

**Peringatan.**
- Laju galat agen WA.
- Lonjakan permintaan per akun atau global (indikasi penyalahgunaan biaya pesan).
- Kegagalan lapis 2 butir 4 di production (`sender_mode=static` ditolak).
- `otp-relay` tidak terjangkau.
- Lonjakan permintaan pemulihan, eksekusi reset di atas batas harian, atau pembatalan oleh pemilik akun (indikasi rekayasa sosial).
- Permintaan pemulihan terbuka lebih lama dari 48 jam tanpa tindakan petugas.

### 9. Ruang lingkup: hanya realm `korporat` `[USER]`

Realm `pasien`, `nakes`, `admin`, dan `rme` tetap TOTP dengan kode pemulihan: "only korporat realm, others stay TOTP" `[USER]` 2026-11-28. JAR provider terpasang di deployment Keycloak bersama, tetapi hanya dirujuk alur realm `korporat`. `keycloak-check` menguji bahwa alur realm lain tidak merujuk `rs-wa-otp-form` atau `rs-wa-enroll`.

### 10. Gerbang production adaptor `whatsapp`: pengecualian T4-Q1 dan CR-007 (P32-1 = A) `[KEPUTUSAN-469]`

- Mengirim kode lewat WhatsApp berarti nomor HR dan isi pesan diproses Meta. Isi pesan dapat tersimpan di pusat data Meta di luar Indonesia selama diproses, hingga 60 menit, walaupun *local storage* Indonesia dipakai `[SRC-54]` `[SRC-997]`. Ini bertentangan dengan T4-Q1 "tanpa kecuali" dan dasar CR-007 `[USER] T4-Q1` `[DOK: backlog.md CR-007]`.
- **Keputusan user:** WhatsApp dipakai di semua lingkungan termasuk production, sebagai pengecualian T4-Q1/CR-007 **khusus kode masuk HR korporat** `[USER]` P32-1 A. Cakupan pengecualian: nomor HR dan kode (tanpa data kesehatan). Pemakaian lain, misalnya pemberitahuan ke nomor lama (butir 7.3), notifikasi, atau pemasaran, di luar pengecualian dan butuh keputusan terpisah (P32-11).
- **Gerbang sebelum adaptor `whatsapp` aktif di production (semua wajib):**
  1. Petugas PDP meninjau syarat Pasal 56 untuk transfer ke Meta dan BSP `[SRC-15]`;
  2. teks persetujuan dan pemberitahuan HR (L-20) menyebut pemakaian nomor untuk kode masuk lewat WhatsApp beserta penyedianya, termasuk untuk HR yang didaftarkan rekan;
  3. BSP terdaftar Meta dipilih dan dikontrak (P32-2 A). Syarat kontrak minimum: ketentuan keamanan BSP dan perjanjian pemrosesan data (DPA) yang memuat daftar sub-pemroses, kontrol akses staf BSP dan Meta ke isi pesan, masa simpan isi pesan dan log, pemberitahuan insiden, hak audit, dan lokasi pemrosesan. Nama mitra, biaya per pesan, kontrak, peran pemrosesan data, dan lokasi pemrosesan oleh BSP `[TIDAK DIKETAHUI]`, dipilih sebelum A32-T10. Agen hanya menerima `to` dan `code` (butir 3);
  4. templat autentikasi disetujui Meta `[SRC-999]`;
  5. A32-T10 selesai dan lulus uji terhadap lingkungan uji BSP;
  6. PRD, BRD, CR-007, dan ADR-016 memuat catatan pengecualian (A32-T11).
- Adaptor `static` tetap dilarang di production (butir 4). Jadi **login HR production terblokir sampai gerbang di atas terpenuhi**. Demo `local` dan `staging` tidak terblokir, dan rancangan butir 1–9 tidak bergantung pada pilihan BSP.
- Ketentuan layanan WhatsApp untuk akun biasa `[SRC-998]` tidak dipakai sebagai dasar: pengirim memakai WhatsApp Business Platform lewat BSP resmi (P32-2 A), bukan otomasi akun biasa. Risiko sisa: Meta atau BSP dapat menangguhkan nomor pengirim atau menolak templat. Akibatnya HR dengan kode pemulihan masuk dengan kode itu, HR tanpa kode menunggu (butir 7.1).

### 11. Toolchain Java dan image Keycloak `[KEPUTUSAN-468]`

- Modul Maven di `infra/keycloak/providers/rs-wa-otp/`:
  - artefak SPI Keycloak 26.8.2 dengan scope `provided`, ditambah JDK saja (klien HTTP `java.net.http`);
  - tanpa dependensi runtime pihak ketiga;
  - nama artefak, versi JDK runtime image Keycloak, dan versi Maven dicek di registry resmi saat task `[TIDAK DIKETAHUI]` (anti-halu §5).
- Image Keycloak kustom multi-tahap: build JAR, lalu `ADD` ke `/opt/keycloak/providers/` sebelum `kc.sh build`, lalu start `--optimized` `[SRC-996]`. Basis tetap `quay.io/keycloak/keycloak:26.8.2` yang dipin digest. Tema dapat ikut dipaketkan sebagai JAR di langkah yang sama.
- Image dibangun CI, dipindai Trivy dan osv-scanner (`pom.xml`), dirujuk lewat digest, dan ditandatangani cosign (ADR-025 butir 10).
- Langkah build dan test Maven masuk `scripts/verify.sh`. Ini menambah langkah, tidak melonggarkan; izin user tetap diminta bila dianggap mengubah verifikasi.
- Setiap upgrade Keycloak (termasuk patch keamanan) wajib menjalankan ulang `keycloak-check` untuk realm `korporat`, karena SPI-nya internal `[SRC-993]`.

### 12. Model data tabel baru (usulan untuk review DBA) `[KEPUTUSAN-473]`

Nama dan kolom sementara; migrasi ditetapkan di A32-T12/A32-T13 bersama DBA. Semua `id` UUIDv7 (ADR-002), snake_case tunggal (ADR-001), kelas mengikuti ADR-008. Kelas tabel adalah keputusan teknis penyusun dan diusulkan, bukan keputusan bisnis.

**Tabel dan kelas.**

| Tabel | Kelas | Kolom dan constraint penting |
| --- | --- | --- |
| `corporate_hr_phone_pending` | Operasional | `id`, `intent_id` (FK ke `idp_provisioning_intent`, UNIQUE), `ciphertext`, `key_version`, `created_at`, `expires_at NOT NULL` |
| `corporate_access_recovery_request` | Transaksi | `id`, `created_at`, `created_by` (NULL bila pengaju anonim), `corporate_client_id`, `hr_account_id`, `channel`, `reason_code`, `requester_name_enc`, `note_enc`; UNIQUE `(id, hr_account_id)` dan `(id, corporate_client_id)` sebagai sasaran FK anak; FK komposit `(hr_account_id, corporate_client_id)` ke `corporate_member (account_id, corporate_client_id)` |
| `corporate_access_recovery_request_status_event` | Riwayat status | Pola ADR-008: UNIQUE `(request_id, version)`, `from_status`, `to_status`, `reason_code` (kode saja), `actor_*`. Status: Baru, Menunggu verifikasi, Terverifikasi, Disetujui, Dieksekusi, Ditolak, Dibatalkan, Kedaluwarsa, Ditutup (penyalahgunaan) |
| `corporate_access_recovery_request_state` | Proyeksi | PK `request_id`, `status`, `version`, `hr_account_id`, `corporate_client_id`, `updated_at` |
| `corporate_access_recovery_verification` | Transaksi | `id`, `request_id`, `corporate_client_id`, `verified_by` (staf), `route` (J1, J3), `outcome`, `peer_response_id` (opsional, untuk J1), `contact_name_enc` (nama penerima telepon), `created_at`; UNIQUE `(id, request_id, verified_by)`; FK komposit `(request_id, corporate_client_id)` ke `..._request`; FK komposit `(peer_response_id, request_id)` ke `..._peer_response (id, request_id)` |
| `corporate_access_recovery_approval` | Transaksi | `id`, `request_id` (UNIQUE: satu keputusan), `corporate_client_id`, `verification_id`, `verified_by`, `approved_by`, `requester_account_id`, `decision`, `created_at` |
| `corporate_access_recovery_peer_response` | Transaksi | `id`, `request_id`, UNIQUE `(id, request_id)`, `requester_account_id`, `hr_account_id`, `corporate_client_id`, `response`, `created_at` |
| `corporate_access_recovery_cancel_token` | Operasional | `id`, `request_id`, `token_hash` (SHA-256 dari token acak 256 bit, UNIQUE), `created_at`, `expires_at NOT NULL`, `used_at` |

**1. Tabel penyangga nomor `corporate_hr_phone_pending`.**
- Hak: `api` hanya `INSERT`; `worker` hanya `SELECT` dan `DELETE`; **tidak ada `UPDATE` untuk siapa pun**. Ini menyempitkan hak standar kelas Operasional (ADR-008) dan dicatat sebagai pengecualian terkontrol.
- Pemisahan role `worker` dari `api` di tabel ini berbeda dari KEPUTUSAN-305 (tabel niat tanpa role worker terpisah). Apakah dipakai role baru atau hak per tabel pada grup yang ada `[TIDAK DIKETAHUI]`, diputuskan DBA.
- Pembersihan: indeks pada `expires_at`; `worker` menghapus bertahap (batch dengan `LIMIT` dan `FOR UPDATE SKIP LOCKED`) baris dengan `expires_at` lewat (waktu dari Clock aplikasi, KEPUTUSAN-131) dan baris yang niatnya sudah selesai; metrik dan peringatan jumlah baris kedaluwarsa. `expires_at NOT NULL` dengan CHECK `expires_at > created_at`.
- Enkripsi: Transit langsung (kunci `hr-phone`, `key_version` tercatat) dengan **context = `intent_id`**, sehingga ciphertext tidak dapat dipindahkan ke niat lain. Tanpa DEK subjek karena baris berumur pendek dan tidak terikat subjek yang sudah ada. Ini pengecualian dari amplop ADR-013 dan perlu persetujuan review keamanan.

**2. Tabel pemulihan dan hak (pola ADR-008).**

| Tabel | `api` | `worker` | Pekerja privasi |
| --- | --- | --- | --- |
| `..._request` | `SELECT`, `INSERT` | `SELECT` | Hapus kunci DEK (crypto-shredding), tanpa `UPDATE` baris |
| `..._status_event` | `SELECT`, `INSERT` | `SELECT`, `INSERT` (transisi Dieksekusi, Kedaluwarsa) | Tidak ada |
| `..._state` | `SELECT`, `INSERT`, `UPDATE` | `SELECT`, `UPDATE` (transisi Dieksekusi, Kedaluwarsa, dengan syarat `version` lama) | Tidak ada |
| `..._verification`, `..._approval`, `..._peer_response` | `SELECT`, `INSERT` | `SELECT` (memeriksa prasyarat job) | Tidak ada |
| `..._cancel_token` | `SELECT`, `INSERT`, `UPDATE` hanya kolom `used_at` | `SELECT`, `DELETE` (pembersihan) | Tidak ada |

Role `api` dan `worker` mengikuti catatan butir 1 tentang pemisahan role. Role Auditor dan Super Admin tidak punya hak tabel (akses lewat use case dan audit).

**3. Satu permintaan terbuka per akun, ditegakkan di database.** Partial unique index pada proyeksi: `UNIQUE (hr_account_id) WHERE status IN (Baru, Menunggu verifikasi, Terverifikasi, Disetujui)`. Batas 3 permintaan terbuka per perusahaan: dalam transaksi pembuatan, baris `corporate_client` dikunci `SELECT ... FOR UPDATE`, lalu jumlah terbuka dihitung dari proyeksi sebelum `INSERT`. Status Ditutup (penyalahgunaan) tidak termasuk status terbuka dan tidak dihitung ke kuota akun (butir 7.3).

**4. Aturan dua orang di database.**
- `..._approval`: FK komposit `(verification_id, request_id, verified_by)` ke `..._verification (id, request_id, verified_by)` (persetujuan tidak dapat menunjuk verifikasi permintaan lain); FK komposit `(request_id, corporate_client_id)` ke `..._request`; FK komposit `(request_id, requester_account_id)` ke `..._request (id, hr_account_id)`; `CHECK (approved_by <> verified_by AND approved_by <> requester_account_id)`.
- `..._peer_response`: UNIQUE `(request_id, hr_account_id)`; `CHECK (hr_account_id <> requester_account_id)`; dua FK komposit `(request_id, corporate_client_id)` ke `..._request (id, corporate_client_id)` dan `(hr_account_id, corporate_client_id)` ke `corporate_member`, sehingga rekan yang menjawab berasal dari perusahaan yang sama; FK komposit `(request_id, requester_account_id)` ke `..._request (id, hr_account_id)`.
- Pemisahan peran sub-peran (Layanan Pelanggan memverifikasi, Verifikator menyetujui) tetap ditegakkan API, karena database tidak mengetahui sub-peran.

**5. Konsistensi tenant.** Semua tabel anak membawa `corporate_client_id` dan FK komposit ke induknya `(id, corporate_client_id)`; `(hr_account_id, corporate_client_id)` diikat ke `corporate_member`, sehingga HR perusahaan lain tidak dapat dicatat pada permintaan perusahaan ini. Sasaran FK di `corporate_member` butuh UNIQUE `(account_id, corporate_client_id)`; apakah riwayat keanggotaan (soft delete lalu tambah ulang) mengizinkannya `[TIDAK DIKETAHUI]`. Bila tidak, DBA memilih sasaran lain (misalnya `(id, corporate_client_id)` dengan kolom `corporate_member_id`).

**6. PII di tabel yang tidak dapat diubah.**
- Tidak menyalin email: tabel hanya menyimpan FK akun, email dibaca lewat `account` pada saat dibutuhkan.
- `requester_name_enc`, `note_enc`, dan `contact_name_enc` dienkripsi dengan DEK subjek (akun HR target) sehingga dapat di-crypto-shred (ADR-013, ADR-008). Nama penerima telepon adalah data pribadi pihak ketiga.
- Kolom dan klasifikasinya ditambah ke data dictionary (`docs/04-data-dictionary.md`).

**7. Token "Bukan saya, batalkan".** Tabel Operasional `..._cancel_token`: hanya hash token (SHA-256) yang disimpan, token mentah hanya ada di email. Sekali pakai: pemakaian dilakukan dengan `UPDATE ... SET used_at = ... WHERE token_hash = ... AND used_at IS NULL AND expires_at > <waktu Clock aplikasi>` dan hasil nol baris berarti ditolak. `expires_at NOT NULL` (sampai akhir masa tunggu). Pembersihan bertahap seperti butir 1.

**Butir terbuka untuk keputusan.**
- Masa retensi tabel pemulihan dan event audit E-04 pemulihan `[TIDAK DIKETAHUI]`: keputusan bisnis dan hukum (L-20, kepatuhan).
- Pemisahan role `worker` (butir 1) dan sasaran FK `corporate_member` (butir 5): keputusan DBA.
- Pengecualian Transit langsung untuk tabel penyangga (butir 1): keputusan review keamanan.
- Nama tabel, kolom, dan nomor migrasi: ditetapkan di task bersama DBA.

## Opsi yang dibandingkan

| Kriteria | A1: autentikator Java memegang semua (kode, pengirim, kode tetap) | **A2 (usulan): autentikator Java tipis + `otp-relay` Rust** | B: langkah OTP di BFF/backend setelah login Keycloak | Status quo: TOTP |
| --- | --- | --- | --- | --- |
| Tidak dapat dilewati; token baru ada setelah OTP | Ya: kode otorisasi terbit setelah alur selesai | Ya, sama dengan A1 | Tidak secara struktural. Token Keycloak bernilai 1 faktor dan sah di API sampai backend menolak lewat penanda MFA per sesi (`sid`) pada **setiap** endpoint dan proses; satu endpoint terlewat = jalan pintas (pola TD-15) | Ya |
| Kesesuaian ADR-011 (`acr`/`amr`) | `amr` lewat nilai referensi `[SRC-995]` | Sama dengan A1 | `amr`/`acr` dari Keycloak hanya `pwd`; status MFA hidup di luar token | `acr`=1 tanpa `amr` `[SRC-682]` (TD-27) |
| Brute force, kode pemulihan, atur ulang ber-MFA | Memakai bawaan Keycloak `[SRC-273]` `[SRC-68]` | Sama dengan A1 | Harus dibangun ulang di backend: penguncian, kode pemulihan (hash sesuai ASVS 6.5.2), dan atur ulang password Keycloak yang kini tanpa faktor kedua sebelum password baru (melemahkan KEPUTUSAN-199) | Bawaan |
| Penjaga kode tetap | Di Java; perilaku gagal start Keycloak `[TIDAK DIKETAHUI]`; tidak memakai pola `DEPLOY_ENV` Rust | Pola KEPUTUSAN-403 di Rust, ditambah lapis Keycloak independen | Pola Rust | Tidak perlu |
| Radius dampak T1 | Kode kustom berjalan di JVM Keycloak bersama semua realm (ketersediaan semua realm bila provider gagal dimuat). Kredensial agen WA ada di Keycloak | JVM sama dengan A1, tetapi logika di Java minimal. Kredensial agen WA hanya di relay. Kompromi relay = kode dapat dipalsukan (butuh password juga) | Tanpa Java. Tetapi logika MFA tersebar di BFF, backend, dan proses `realtime` | Tidak ada perubahan |
| Pola dan toolchain baru | Java/Maven, image kustom | Java/Maven, image kustom, satu subcommand Rust (pola `email-relay`) | Status sesi MFA baru di authn backend dan BFF | — |
| Estimasi (sesudah P32-3, P32-4, P32-7) | ±104 jam-task (perkiraan penyusun: ±45 ditambah ±59 untuk nomor di formulir, pemulihan lewat tiket, dan pengerasan) | ±107 jam-task (bagian "Rencana task") | ±103 jam-task (perkiraan penyusun: ±44 ditambah ±59), ditambah penguatan atur ulang yang belum terancang | 0 |
| Memenuhi keputusan user | Ya | Ya | Ya | Tidak (user memilih WhatsApp) |

A2 dipilih karena memberi sifat keamanan A1 (faktor kedua di dalam IdP, memakai ulang brute force, kode pemulihan, dan atur ulang ber-MFA) dengan effort yang setara B. A2 juga menaruh bagian yang paling sering berubah (pengirim, kode tetap, batas laju biaya) di Rust, yang punya pola penjaga, test, dan review yang sudah berjalan.

## Alternatif yang ditolak

- **B (BFF/backend):** lihat tabel. Selisih effort kecil, sedangkan token 1 faktor yang beredar dan pelemahan atur ulang password adalah mundur dari ADR-011 butir 5 dan 8.
- **A1:** kredensial agen WA dan kode tetap berada di Keycloak, yang dipakai semua realm. Penjaga production tidak mengikuti pola `DEPLOY_ENV` yang sudah teruji.
- **OTP email:** dilarang NIST §3.1.3.1 dan pengantar ASVS V6.6 `[SRC-990]` `[SRC-265]`.
- **SMS penyedia Indonesia** (bila WhatsApp tidak diizinkan): PSTN berstatus *restricted* (NIST §3.1.3.3) dan ASVS 6.6.1 mensyaratkan opsi lebih kuat seperti TOTP ikut ditawarkan `[SRC-990]` `[SRC-992]`. Tidak diusulkan, tetapi tetap disebut di P32-1 sebagai pilihan user.
- **Pengirim WhatsApp langsung dari Keycloak tanpa relay:** sama dengan A1.
- **Tetap TOTP:** bertentangan dengan keputusan user CR-041. Tidak dipilih (P32-1 A).

## Konsekuensi

**Positif.**
- HR tidak perlu memasang aplikasi authenticator. Login pertama: atur password, periksa atau koreksi nomor WhatsApp (sudah diisi di formulir bila ada), masukkan kode, lalu pilih membuat kode pemulihan atau melewatinya. Login harian: password lalu kode WhatsApp.
- Token korporat dapat membawa bukti faktor kedua (`amr`), yang menutup kesenjangan TD-27 untuk realm ini bila verifikasi butir 2 berhasil.
- Kode tetap demo mustahil aktif di production lewat tiga lapis independen.

**Negatif.**
- Java/Maven dan image Keycloak kustom masuk stack, sehingga ADR-024 berubah. Ini membalik alasan ADR-026 untuk kasus ini, karena tidak ada cara memasang faktor baru di alur Keycloak tanpa SPI.
- Setiap upgrade Keycloak, termasuk patch keamanan, butuh uji ulang provider karena SPI internal `[SRC-993]`. Patch darurat dapat tertahan bila SPI berubah.
- Komponen baru (`otp-relay`, agen WA) menjadi syarat login HR. Bila agen WA mati, hanya HR yang membuat kode pemulihan (opsional, P32-3 B) yang dapat masuk. HR lain menunggu layanan pulih.
- Effort Fase 1 bertambah ±107 jam-task (±48 untuk MFA WhatsApp, ±59 untuk nomor di formulir, pemulihan lewat tiket, dan pengerasan), sebagian menggantikan bagian TOTP F1-T85/F1-T87/F1-T88 yang sudah dibangun.
- Pemulihan lewat tiket menjadikan Layanan Pelanggan dan Verifikator jalur kritis kepercayaan dan sasaran rekayasa sosial, dan menambah beban operasional di Fase 1 (butir 7.1).
- Dua jenis job baru di ADR-030 melonggarkan larangan "menghapus kredensial" dan "mengubah atribut pengguna yang sudah ada" untuk `worker`, dengan kontrol kompensasi yang tidak sepenuhnya berdiri sendiri (butir 7.3).
- Nomor HR (data pribadi, termasuk milik pihak ketiga yang diketik pengundang) kini melewati `api` dan tersimpan sementara terenkripsi di database aplikasi.

### Risiko sisa

| Risiko | Penanganan | Register |
| --- | --- | --- |
| Bentrok T4-Q1/CR-007: isi pesan diproses Meta di luar Indonesia `[SRC-997]` | Pengecualian diputuskan user (P32-1 A); gerbang production butir 10: tinjauan Petugas PDP (Pasal 56 `[SRC-15]`), teks persetujuan HR, kontrak BSP | Risk register (baru) |
| Meta atau BSP menangguhkan nomor pengirim atau menolak templat autentikasi; BSP gagal `[SRC-999]` | P32-2 A (jalur resmi); HR yang membuat kode pemulihan tetap dapat masuk; HR tanpa kode menunggu (gagal-tertutup, tanpa jalan pintas); runbook gangguan BSP; peringatan laju galat | Baru |
| Kompromi `otp-relay` memungkinkan pemalsuan kode (penyerang tetap butuh password) | Per host, jaringan Docker khusus, tanpa port masuk, TLS + token pembawa, identitas OpenBao minimum, peringatan `sender_mode=static` di production | Threat model (baru) |
| Pengambilalihan akun WhatsApp atau nomor (ganti SIM, porting) memberi faktor kedua ke penyerang | Password tetap wajib, penguncian 5/15, pemberitahuan ubah nomor; indikator risiko ganti SIM bergantung pada penyedia `[TIDAK DIKETAHUI]` | KOR-01 (diperbarui) |
| Phishing seketika atas password dan kode | Sama dengan TOTP; faktor tahan phishing hanya wajib di L3 `[SRC-276]` | KOR-01 |
| Penyalahgunaan biaya pesan lewat pendaftaran nomor sembarang (akun HR dapat dibuat mandiri lewat `/daftar`) | Batas 3 nomor per akun per 24 jam, batas per nomor dan global, peringatan lonjakan. Tidak ada pesan dikirim saat pendaftaran atau undangan, hanya setelah pemilik email mengatur password (butir 6) | Baru |
| Perubahan SPI Keycloak saat upgrade | Provider minimal, `keycloak-check` wajib per upgrade, versi dipin | Baru |
| Provider gagal dimuat sehingga Keycloak gagal start untuk semua realm | Uji start image di CI dan staging sebelum production; rollback ke digest sebelumnya | Baru |
| Kode tetap demo bocor | Hanya lingkungan data sintetis; nilai per lingkungan di `.env.local`/OpenBao; dapat diganti kapan saja | — |
| Nomor korban ditanam di `/daftar` atau undangan (typo atau sengaja) sehingga korban menerima kode tak diminta | Tidak ada pengiriman sebelum login pertama; batas per nomor (10 per 24 jam) dan per akun; nomor belum terverifikasi hanya untuk `purpose=enroll`; layar enrol meminta HR memeriksa nomor | Baru |
| Rekayasa sosial atas support: penyerang mengaku HR (pretext, desakan waktu) dan meminta reset nomor | Email atau isian tidak pernah cukup; J1/J2/J3 (butir 7.3); telepon balik ke nomor kantor pada berkas, bukan nomor penelepon; pemisahan tugas Layanan Pelanggan dan Verifikator; masa tunggu 24 jam; notifikasi ke pemilik dan HR lain; petugas tidak pernah menetapkan nomor atau menerima kode | Threat model (baru) |
| Penyerang menguasai **hanya email** akun HR (prasyarat: mailbox saja, tanpa password), mengajukan reset, lolos verifikasi lewat rekayasa sosial, menerima tautan, lalu mencoba mengikat nomor sendiri dan mengatur ulang password lewat email dan WhatsApp | Tautan hanya terbit sesudah J1 atau (J2 + syarat J3-2 + masa tunggu) yang tidak bergantung pada email itu; sesi tautan mewajibkan password saat ini; `rs-reset-credentials-mfa` diblokir selama pending dan 7 hari sesudah nomor terikat lewat pemulihan; notifikasi ke HR lain. Risiko sisa: pemegang mailbox yang juga mengetahui password dan melewati verifikasi manusia (HR tunggal: telepon balik dan berkas). Pemegang mailbox yang password-nya tidak diketahui tidak dapat menyelesaikan pemulihan | Threat model (baru), KOR-01 |
| Orang dalam support atau Verifikator disuap atau akunnya diambil alih | Dua peran terpisah harus sepakat, MFA admin, audit E-04, masa tunggu, notifikasi ke pemilik dan HR lain, peringatan lonjakan eksekusi, tidak ada jalur Super Admin. Risiko sisa: kolusi dua petugas | Threat model (baru) |
| HR rekan jahat mengonfirmasi permintaan palsu untuk mengambil alih akun rekan | Orang dalam perusahaan yang sama dan berperan sama (PT-68); masa tunggu dan notifikasi ke pemilik akun memberi waktu membatalkan. Risiko sisa diterima | Threat model (baru) |
| Penyerang mengubah nomor telepon kantor (koreksi data perusahaan) lalu lolos telepon balik J2 | Nomor kantor yang diubah dalam 30 hari terakhir tidak dipakai dan dieskalasi ke Verifikator. Apakah koreksi nomor kantor butuh persetujuan Verifikator `[TIDAK DIKETAHUI]` | Threat model (baru) |
| Telepon kantor dijawab pihak tak berwenang (resepsionis) | Penerima dicatat; J3 mewajibkan pencocokan nama dengan bukti kewenangan di berkas; J1 diutamakan | Baru |
| Enumerasi akun HR lewat `/bantuan-akses` | Jawaban `202` seragam dan waktu disamakan (pola US-101 AC3, ADR-031); batas laju per sumber dan per email | Baru |
| Permintaan palsu membanjiri HR lain atau support, atau memblokir permintaan sah | Batas permintaan terbuka per akun dan per perusahaan, batas email per permintaan; permintaan terbuka tidak mengunci login HR | Baru |
| Petugas diminta atau menerima kode OTP, password, atau kode pemulihan dari penelepon | Larangan di SOP, layar petugas tanpa kolom itu, petugas tidak punya kemampuan melihat kode | Baru |
| Pemegang password yang bocor masuk saat reset berjalan dan mengikat nomor miliknya | `rs_wa_reset_pending` menolak login password-saja; nomor baru hanya dapat diikat dari sesi tautan email (butir 7.3) | Baru |
| `api` diretas lalu menyisipkan job reset massal | Prasyarat di sisi Keycloak, batas harian, peringatan; downgrade tidak berujung pengambilalihan (butir 7.3). Prasyarat catatan persetujuan tidak berdiri sendiri | Baru (review T1) |
| HR tanpa kode pemulihan terkunci saat BSP gagal atau nomor hilang (konsekuensi P32-3 B) | Pemulihan lewat tiket; penawaran dan pengingat kode pemulihan; runbook gangguan BSP. Risiko sisa diterima user | KOR-02 (diperbarui) |
| Pemegang password saja mengikat nomor miliknya pada akun yang nomornya belum terverifikasi lewat "Ubah nomor" | Pendaftaran nomor hanya di sesi tautan; akun tanpa nomor terverifikasi diperlakukan seperti `rs_wa_reset_pending`; AIA ubah nomor butuh nomor terverifikasi (butir 6) | Baru |
| BSP atau Meta terkompromi, atau staf BSP/Meta mengakses isi pesan (nomor dan kode 5 menit) | Agen hanya mengirim `to` dan `code` (tanpa identitas atau tujuan); kode sekali pakai, 5 menit, password tetap wajib; syarat keamanan BSP dan DPA masuk gerbang production butir 10 item 3. Risiko sisa: pihak dengan akses BSP/Meta yang juga memegang password | Threat model (baru) |
| Nilai `rs_wa_phone*` bocor lewat admin event "include representation" atau event `UPDATE_PROFILE` | Representasi dimatikan di realm `korporat`, dijaga `keycloak-check`; uji event (butir 8) | Baru |
| Penyerang menghabiskan kuota permintaan korban dengan permintaan palsu | Permintaan berstatus Ditutup (penyalahgunaan) tidak dihitung ke kuota akun (butir 7.3) | Baru |
| Pemegang password mengunci akun korban lewat kode salah berulang (`maxTemporaryLockouts`) | Korban memakai jalur pemulihan; nilai batas dikalibrasi di A32-T6. Risiko sisa: penolakan layanan terarah | Baru |
| Mailbox HR diambil alih dan penyerang menguasai akun | Di luar alur pemulihan (butir 7.2). Penanganan: P32-20 | Baru |

## Dampak ke checklist ASVS (`docs/05-asvs-checklist.md` V6)

| Requirement | Sebelum (realm `korporat`) | Sesudah |
| --- | --- | --- |
| Ruang lingkup V6 | Tidak menyebut `korporat` | Tambah realm `korporat` dan `otp-relay` |
| 6.3.3 (L2) | TOTP + kode pemulihan | Password + kode WhatsApp (out-of-band) atau kode pemulihan (opsional); tetap MFA `[SRC-265]` |
| 6.5.1 (L2) | TOTP sekali pakai | Kode OOB sekali pakai per tantangan; kode pemulihan sekali pakai `[SRC-992]` |
| 6.5.3 (L2) | Seed TOTP oleh Keycloak | Kode oleh CSPRNG OS di relay (crate dicek registry saat task) |
| 6.5.4 (L2) | — | ±29,7 bit ≥ 20 bit |
| 6.5.5 (L2) | TOTP 30 detik | Kode OOB 5 menit ≤ 10 menit |
| 6.6.1 (L2) | Tidak berlaku | Tetap tidak berlaku selama tanpa SMS; kontrak agen melarang jatuh ke SMS |
| 6.6.2 (L2) | Tidak berlaku | Kode terikat sesi autentikasi asal |
| 6.6.3 (L2) | Tidak berlaku | Batas 5 per kode + brute force realm + batas laju kirim |
| 6.4.3 (L2) | OTP sebelum password baru | Kode WhatsApp atau kode pemulihan (bila HR punya) sebelum password baru |
| 6.4.4 (L2) | KEPUTUSAN-265 (TOTP) | Butir 7.3: pembuktian identitas lewat konfirmasi HR lain atau kontak resmi perusahaan (J1–J3), bukan lewat kode pemulihan; kode pemulihan opsional (butir 7.1) |
| 6.1.3, 6.3.4 (L2) | Alur terdokumentasi | Jalur baru (pendaftaran nomor, ubah nomor lewat AIA) didokumentasikan; direct grant tetap tertutup |
| Celah N-20 (TOTP SHA1) | Berlaku | Tidak berlaku untuk `korporat`; tetap untuk realm lain |

## Dampak ke dokumen lain (diubah setelah ADR ini diterima; PRD belum diubah sekarang)

| Dokumen | Bagian | Perubahan |
| --- | --- | --- |
| PRD | NFR-SEC-02 | Faktor kedua HR = kode sekali pakai lewat WhatsApp; kode pemulihan opsional. Realm lain tetap TOTP. "SMS, OTP email, WebAuthn tidak dipakai" tetap |
| PRD | US-1904 AC1, AC7, AC8, AC9; AC baru | AC1: password + kode WhatsApp; nomor sudah diisi di formulir (bila ada) dan diverifikasi di login pertama; kode pemulihan **ditawarkan, tidak wajib**. AC7: kode pemulihan (bila punya), lalu ubah nomor. AC8/AC9: reset lewat tiket support Fase 1 (butir 7.3: Layanan Pelanggan memverifikasi, Verifikator menyetujui, masa tunggu 24 jam), tidak lagi Fase 2. AC baru: ubah nomor (autentikasi ulang, verifikasi nomor baru, pemberitahuan, E-04), batas kirim ulang dan percobaan, serta pembuatan ulang kode pemulihan |
| PRD | US baru "Pemulihan akses HR lewat tiket" (Fase 1) | Pengajuan publik dan oleh petugas; verifikasi J1/J2/J3; pemisahan tugas; masa tunggu; pembatalan; eksekusi; pemberitahuan; audit; batas laju. Permintaan konfirmasi rekan di member area. Bagian matriks izin §3.1: Layanan Pelanggan `hr_recovery.verify`, Verifikator `hr_recovery.approve`; PT-67 (4a) dan BR-136 `[USER] C21-P4`: keputusan reset tetap di Verifikator |
| PRD | US-1903 AC1 | "undangan aktivasi (password dan TOTP)" diganti "undangan aktivasi (password); nomor WhatsApp diperiksa atau diisi saat login pertama" |
| PRD | US-1901 (AC1, AC2, AC baru) | AC1: data diri HR bertambah "nomor WhatsApp (+62)"; teks persetujuan menyebut pemakaian nomor untuk kode masuk. AC2: nomor di luar +62 atau format salah ditolak per field. AC baru (negatif): nomor yang diisi tidak dipakai untuk hal lain selain kode verifikasi di login pertama, dan tidak ada pesan terkirim saat pendaftaran. AC4: email terdaftar, jawaban seragam, nomor dibuang. Penyebutan "password, TOTP" di rujukan aktivasi diganti sesuai butir 6 |
| PRD | US-1904 AC2 (undangan HR) | HR pengundang mengisi email **dan nomor WhatsApp rekan (+62)**; nomor tidak dapat dibaca kembali; nomor tidak valid ditolak per field; email terdaftar tetap dijawab seragam. Teks email aktivasi menyebut pemakaian nomor |
| PRD dan BRD | §2.2 non-scope, BR-050, BR-071, CR-007, PT-22 | WhatsApp dipakai **khusus kode masuk HR** (P32-1 A), bukan notifikasi. Pengecualian dicatat |
| `openapi/` (hanya daftar, belum diubah) | `public.json`: `CorporateRegistrationRequest`; `korporat.json`: `HrInvitationRequest` | Tambah field nomor WhatsApp (usulan `hr_whatsapp_number` di pendaftaran, `whatsapp_number` di undangan), wajib, string (maks 20 karakter mentah), dinormalkan ke E.164 oleh server, respons tidak memuatnya. Perbarui deskripsi `hr_email` dan `email` ("password dan TOTP" menjadi password). Tambah kode galat per field untuk nomor tidak valid di `docs/05-response-codes.md` |
| `openapi/` (hanya daftar) | Endpoint baru pemulihan (nama final saat task) | `public.json`: `POST /v1/public/corporate-access-recovery-requests` (202 seragam) dan pembatalan oleh pemilik dengan token tautan; `korporat.json`: daftar permintaan yang menunggu konfirmasi rekan, konfirmasi, penolakan; `backoffice.json`: daftar antrean, catat verifikasi, setujui, tolak, batalkan permintaan pemulihan. `POST /backoffice/v1/corporate-clients` tidak berubah kecuali P32-12 |
| ADR-011 | Butir 3 (baris `korporat`), butir 5, Konsekuensi | Faktor kedua HR; pemulihan; WhatsApp dikecualikan dari kalimat "SMS dan OTP email tidak dipakai" hanya untuk `korporat` |
| ADR-027 | Butir 3 | MFA TOTP diganti kode WhatsApp, kode pemulihan opsional; pemulihan MFA HR mengikuti butir 7 |
| ADR-024 | Stack | Java/Maven khusus provider Keycloak; image Keycloak kustom |
| ADR-025 | Butir 2, 7 | Kontainer `otp-relay` per host; identitas dan policy OpenBao baru; path KV kode tetap hanya staging |
| ADR-016 | Konteks dan alternatif | Catatan pengecualian MFA (P32-1 A) |
| KEPUTUSAN-253, -265, -85, -203 | — | KEPUTUSAN-253 digantikan untuk `korporat`; KEPUTUSAN-265 direvisi (butir 7); parameter TOTP KEPUTUSAN-85 tidak berlaku untuk `korporat`; kode pemulihan opsional (ditawarkan lewat `rs-wa-enroll` atau member area, bukan wajib) |
| Threat model | KOR-01, KOR-02; ancaman baru | Mitigasi KOR-01/KOR-02 diperbarui. Ancaman baru: kompromi `otp-relay`, kode tetap aktif di production, penyalahgunaan biaya pesan, agen WA mati atau diblokir, pengambilalihan nomor WhatsApp, rekayasa sosial atas support, orang dalam, pengambilalihan email akun, job reset massal dari `api` yang diretas |
| `docs/05-asvs-checklist.md` | V6 | Tabel di atas |
| `docs/05-stack.md` | §6 (verify), §7 (Keycloak), §5.3 (deploy) | Langkah Maven, image kustom, `OTP_SENDER` ≠ `static` di production |
| `docs/coding-standards.md` | Bagian baru | Aturan minimum Java provider (tanpa dependensi runtime, larangan log kode atau nomor) |
| `CLAUDE.md` | Modul kritis | Tambah JAR provider Keycloak, `otp-relay`, dan konfigurasi penjaga kode tetap |
| Design guideline dan wireframe K01 | Layar MFA HR | Layar pendaftaran nomor, masukkan kode, kirim ulang, dan kode pemulihan menggantikan layar TOTP |
| Kepatuhan (DPIA/inventaris pemrosesan) | HR | Nomor WhatsApp HR, tujuan, penyimpanan, penyedia |
| `infra/keycloak/README.md` | TD-27 | Kontrol kompensasi diganti pemeriksaan `amr` untuk `korporat` bila butir 2 terverifikasi |
| Phase plan | F1-T85, F1-T87, F1-T88; task baru | Lihat bagian berikut |
| ADR-030 | Butir 3 dan 4 | `idp_create_user` membawa nomor belum terverifikasi (dari tabel penyangga terenkripsi) ke atribut `rs_wa_phone_pending`; dua jenis job baru `idp_reset_wa_factor` dan `idp_send_wa_reenroll` dengan analisis ancaman butir 7.3; jenis job pemberitahuan email. Larangan "menghapus kredensial" dan "mengubah atribut" dikecualikan hanya untuk jenis ini |
| ADR-031 | Konteks, Keputusan butir 2–3 | Formulir `/daftar` bertambah isian nomor (sebelumnya "tidak bertambah isian"); halaman `/bantuan-akses` dan rute Caddy ketiga dengan rewrite ke path tetap, batas badan 16 KB, test snippet |
| ADR-012, ADR-013, ADR-008 | `PublicReason`, izin sub-peran, kunci Transit, kelas tabel | `PublicReason` untuk permintaan pemulihan; izin `hr_recovery.verify`/`approve`/`reject`; kunci `hr-phone`; tabel `corporate_hr_phone_pending` dan tabel permintaan pemulihan dengan riwayat status append-only |
| Design guideline dan wireframe | Web korporat, member area, backoffice | Isian nomor di `/daftar` dan Undang HR; layar `/bantuan-akses`; layar konfirmasi rekan; antrean pemulihan backoffice |
| `docs/04-data-dictionary.md` | Kelas tabel dan klasifikasi PII | Tabel butir 12 beserta kolom `*_enc` (nama pemohon, nama penerima telepon, keterangan petugas) dan klasifikasinya |
| ADR-012 | Event audit | Verifikator membuka bukti kewenangan di J3 syarat 2 memancarkan E-05 |
| `infra/keycloak/realms/korporat.json` dan `keycloak-check` | `maxTemporaryLockouts`, `adminEventsDetailsEnabled`, `rs-reset-credentials-mfa`, atribut baru | Nilai dan pemeriksaan butir 5, 7.3, dan 8 |

## Rencana task Fase 1 (semua T1; reviewer keamanan + code-reviewer; DBA untuk tabel baru) `[KEPUTUSAN-472]`

Nomor sementara `A32-Tn` diberi nomor F1 oleh orkestrator di phase plan. A32-T4, A32-T5, A32-T6, dan A32-T11 diperluas oleh revisi 2026-11-28; A32-T12–A32-T16 baru.

| Task | Isi | Jam-task | Bergantung |
| --- | --- | --- | --- |
| A32-T1 | Toolchain Java/Maven: cek registry (artefak SPI 26.8.2, JDK, Maven, pustaka test), kerangka modul, langkah build/test di `verify.sh` dan CI, osv-scanner `pom.xml` | 4 | ADR ini diterima |
| A32-T2 | Image Keycloak kustom (multi-tahap, `providers/`, `kc.sh build`, `--optimized`, digest, cosign); compose lokal dan `keycloak-check ephemeral` memakai image ini; opsional tema sebagai JAR | 4 | A32-T1 |
| A32-T3 | `otp-relay` Rust: konfigurasi + penjaga `DEPLOY_ENV` (lapis 1), trait `OtpSender` + adaptor `static`, server HTTP di jaringan khusus, TLS + token pembawa, batas laju, idempotensi, redaksi log, test negatif Tambahan: `K_host` (HMAC berkunci, rotasi dengan `key_id`), larangan mempersistenkan atau mencatat catatan sesi, agen hanya menerima `to` dan `code`. | 7 | — |
| A32-T4 | Autentikator `rs-wa-otp-form`: panggilan relay, MAC + catatan sesi, perbandingan waktu konstan, batas 5 per kode, jeda kirim ulang, `failureChallenge`, lapis 2 penjaga, templat FTL, **penolakan login password-saja saat `rs_wa_reset_pending`**, `configuredFor()` hanya membaca `rs_wa_phone`, test Tambahan: verifikasi MAC berkunci, `deploy-env` kosong = production, uji catatan sesi tidak dipersistenkan, `maxTemporaryLockouts`. | 8 | A32-T1, A32-T3 |
| A32-T5 | Required action `rs-wa-enroll`: **nomor terisi (`rs_wa_phone_pending`) tampil, bisa dikoreksi, atau kosong; verifikasi; penulisan `rs_wa_phone`**; **kode pemulihan ditawarkan opsional (bukan wajib) dan dapat dibuat ulang lewat AIA**; AIA ubah nomor (`getMaxAuthAge` 0); tawaran ganti nomor setelah login dengan kode pemulihan; **mode enrol dari tautan pemulihan**; pemberitahuan (bila mekanisme tersedia) Tambahan: pendaftaran nomor hanya di sesi tautan (akun tanpa nomor terverifikasi diperlakukan seperti `rs_wa_reset_pending`), mode pemulihan mewajibkan password saat ini, menulis `rs_wa_recovery_bound_at`. | 8 | A32-T4 |
| A32-T6 | Realm `korporat` sebagai kode: alur browser/reset/direct grant, atribut `rs_wa_phone`, **`rs_wa_phone_pending`, `rs_wa_reset_pending`**, konfigurasi SPI, `CONFIGURE_TOTP` mati hanya di `korporat`, nilai referensi + `oidc-amr-mapper`; skenario `keycloak-check` (test negatif di bawah); regresi realm lain Tambahan: blokir `rs-reset-credentials-mfa` (pending dan 7 hari), `maxTemporaryLockouts`, `adminEventsDetailsEnabled=false` dijaga `keycloak-check`, uji event `UPDATE_PROFILE`. | 7 | A32-T2, A32-T5 |
| A32-T7 | Backend: pemetaan event E-01/E-02/E-04 (detail faktor), metrik dan peringatan relay; pemeriksaan `amr` di authn bila butir 2 terverifikasi | 4 | A32-T6 |
| A32-T8 | Tema dan UX: layar nomor, kode, kirim ulang (hitung mundur), petunjuk 4 digit terakhir, tawaran kode pemulihan; DG dan wireframe K01 | 4 | A32-T4, A32-T5 |
| A32-T9 | Konfigurasi demo: `.env.example` tanpa nilai, KV OpenBao staging + policy `otp-relay`, checklist deploy §5.3 dan pemeriksaan skrip production (lapis 3) Tambahan: `K_host` di KV OpenBao dan cara Keycloak menerimanya. | 3 | A32-T3 |
| A32-T10 | Adaptor `whatsapp` lewat BSP sesuai kontrak butir 3 dan templat autentikasi, diuji terhadap agen tiruan dan lingkungan uji BSP. **Hanya setelah BSP dipilih (P32-2 A)**; aktif di production hanya setelah gerbang butir 10. Estimasi bergantung API BSP `[TIDAK DIKETAHUI]` | 3 | A32-T3, BSP dipilih |
| A32-T11 | Dokumen setelah persetujuan (PRD termasuk US-1901/US-1904/US-1903 dan US baru, ADR-011/024/025/027/030/031, threat model, ASVS, DPIA, 05-stack, coding standards, CLAUDE.md, phase plan, risk register, `openapi/` daftar di atas bila diputuskan di task kode) | 5 | ADR ini diterima |
| A32-T12 | **Nomor di formulir:** fungsi normalisasi + validasi `+62` (Rust) + berkas vektor uji bersama dengan Java; field nomor di `/daftar` dan Undang HR (`api`, `openapi`, `web/korporat`); tabel penyangga `corporate_hr_phone_pending` + migrasi + kunci Transit `hr-phone`; `idp_create_user` menulis `rs_wa_phone_pending`; sumber resmi format nomor dibuka dan dicatat di `docs/sources.md`; test negatif (butir 6) Tambahan: pembersihan bertahap tabel penyangga (indeks `expires_at`), Transit langsung dengan context `intent_id`, batas DB butir 12, sumber resmi format nomor (pengganti P32-17), jalur kirim ulang aktivasi untuk akun tanpa nomor terverifikasi. | 12 | ADR ini diterima, A32-T6, DBA |
| A32-T13 | **Backend pemulihan:** modul permintaan pemulihan (tabel, riwayat status append-only, `status-flow`), endpoint publik/korporat/backoffice, izin `hr_recovery.*` dan pemisahan tugas, J1/J2/J3, masa tunggu, pembatalan, kedaluwarsa, batas laju, E-04, jawaban seragam Tambahan: enam tabel butir 12 beserta constraint DB (satu permintaan terbuka per akun, aturan dua orang, tenant), token pembatalan Operasional, enkripsi nama dan catatan dengan DEK subjek, status Ditutup (penyalahgunaan), E-05 untuk pembukaan bukti kewenangan, klasifikasi data dictionary. | 16 | A32-T6, A32-T12, DBA |
| A32-T14 | **Worker pemulihan:** `idp_reset_wa_factor`, `idp_send_wa_reenroll`, jenis job pemberitahuan email (jalur ADR-026, templat tetap), prasyarat terhadap keadaan Keycloak, batas harian dan peringatan, amandemen teks ADR-030, test skenario `api` diretas Tambahan: prasyarat kirim ulang aktivasi untuk akun tanpa nomor terverifikasi, pembukaan kunci `maxTemporaryLockouts` bila butuh job. | 9 | A32-T13, A32-T5 |
| A32-T15 | **Web pemulihan:** `/bantuan-akses` + rute Caddy ketiga (snippet, test `caddy-snippet`), layar konfirmasi rekan di member area, antrean pemulihan di backoffice, DG dan wireframe | 8 | A32-T13 |
| A32-T16 | **Uji pemulihan ujung ke ujung:** skenario `keycloak-check` (reset pending, tautan, `amr`), suite rekayasa sosial (test negatif butir 7.3), draf SOP support (skrip verifikasi, larangan meminta kode), entri threat model Tambahan: uji sesi tautan tanpa password, blokir atur ulang password, J2 tanpa J3 syarat 2, constraint DB. | 5 | A32-T14, A32-T15 |
| | **Total** | **107** | |

Demo dengan kode tetap butuh A32-T1 sampai A32-T9 dan A32-T12 (±61 jam-task). Pemulihan lewat tiket (A32-T13–A32-T16, ±38 jam-task) wajib selesai sebelum Fase 1 ditutup dan sebelum production. A32-T10 dan A32-T11 (±8 jam-task) menutup total (61 + 38 + 8 = 107). BSP dan agen WA (di luar estimasi, `[TIDAK DIKETAHUI]`) dibutuhkan sebelum production. Estimasi A32-T12–A32-T16 adalah perkiraan penyusun (±25%) dan belum dikalibrasi dengan kecepatan task sebelumnya.

## Test negatif wajib

- Login HR tanpa kode, dengan kode salah, kode kedaluwarsa (> 5 menit), kode bekas, atau kode dari sesi autentikasi atau tab lain: tidak ada kode otorisasi.
- Lima kode salah: tantangan batal dan penghitung brute force realm naik (penguncian 15 menit, E-02). Kode baru tidak mereset penghitung.
- Kirim ulang sebelum 60 detik tidak memanggil relay. Batas per akun, per nomor, dan 3 nomor per 24 jam menghasilkan `429` tanpa kirim.
- Relay:
  - `DEPLOY_ENV=production` + `OTP_SENDER=static` gagal start;
  - `OTP_SENDER` kosong gagal start;
  - kode tetap bukan `^[A-Z0-9]{6}$` gagal start;
  - nilai kode tidak muncul di pesan galat atau log.
- Keycloak dengan `deploy-env=production`, atau `deploy-env` kosong, tidak ada, atau tidak dikenal, dan jawaban relay `sender_mode=static`: login gagal-tertutup.
- MAC: tanpa `K_host` nilai catatan sesi tidak cocok dengan kode mana pun yang diuji offline; `K_host` dan catatan sesi tidak ada di dump database, log, atau cadangan; dua kunci diterima 10 menit saat rotasi.
- Admin event dan event `UPDATE_PROFILE` tidak memuat nilai `rs_wa_phone*` (realm dengan representasi menyala ditolak `keycloak-check`).
- `maxTemporaryLockouts` tercapai: akun terkunci sampai jalur pemulihan; kode baru tidak mereset penghitung.
- Akun dengan password diatur dan nomor belum terverifikasi: login password-saja ditolak, "Ubah nomor" tidak tersedia, pendaftaran nomor hanya dari sesi tautan.
- Relay tanpa token atau dengan token salah menjawab `401` tanpa kirim. Relay mati: login gagal-tertutup; hanya HR yang punya kode pemulihan dapat masuk dengan kode itu.
- Ubah nomor: tanpa autentikasi ulang ditolak. Nomor lama tetap berlaku sampai nomor baru terverifikasi. E-04 tercatat.
- Atur ulang password tanpa kode WhatsApp atau kode pemulihan: password baru tidak tersimpan.
- Direct grant tidak dapat memberi token HR walaupun klien diaktifkan tanpa sengaja.
- Kode pemulihan bekas ditolak dan dihitung penguncian.
- Nomor lengkap dan kode tidak muncul di log relay, detail event Keycloak, audit, atau security log (pemeriksaan pola).
- Bila pemeriksaan `amr` dipakai: token `korporat` tanpa faktor kedua di `amr` ditolak backend.
- Realm `pasien` dan `admin` tetap meminta TOTP (regresi).
- **Nomor di formulir:**
  - `0812…`, `62812…`, dan `+62812…` (contoh format, bukan nomor nyata) menjadi E.164 `+62…`; `+65…`, `00…`, huruf, dan terlalu panjang ditolak per field;
  - pendaftaran dan undangan tidak memicu pengiriman WhatsApp;
  - respons API, daftar anggota HR, backoffice, audit, dan log tidak memuat nomor lengkap;
  - `rs_wa_phone_pending` tidak pernah dipakai untuk `purpose=login` dan tidak membuat `configuredFor()` bernilai benar;
  - email sudah terdaftar: jawaban seragam dan nomor dibuang; baris ciphertext terhapus setelah `idp_create_user`, atau saat niat gagal atau kedaluwarsa;
  - HR dapat mengoreksi nomor sebelum verifikasi; nomor berbeda keempat dalam 24 jam menghasilkan `429`;
  - HR yang dibuat Verifikator (tanpa nomor terisi) tetap dapat mendaftarkan nomor di login pertama.
- **Kode pemulihan opsional:** HR yang melewati tawaran login normal; "Lewati" tidak membuat kredensial; kode dapat dibuat atau dibuat ulang setelah autentikasi ulang; tanpa kode pemulihan dan relay mati, login gagal-tertutup tanpa jalan pintas.
- **Pemulihan lewat tiket:**
  - permintaan tanpa verifikasi atau tanpa persetujuan tidak dapat dieksekusi; pemverifikasi sama dengan penyetuju ditolak; Super Admin, Auditor, dan sub-peran lain ditolak dan tercatat; permintaan atas akun staf sendiri ditolak;
  - eksekusi sebelum masa tunggu berakhir ditolak; pembatalan oleh pemilik akun, HR rekan, atau petugas menghentikan permintaan;
  - satu penolakan HR aktif lain menghentikan permintaan; petugas tidak dapat mengonfirmasi atas nama HR;
  - HR tunggal yang bukan pendaftar berkas ditolak (J3 syarat 2); telepon balik memakai nomor kantor pada berkas, bukan nomor pemohon; nomor kantor yang baru diubah tidak dipakai;
  - login password-saja saat `rs_wa_reset_pending` tidak menghasilkan token dan tidak dapat mengikat nomor; tautan bekas atau kedaluwarsa ditolak; kode pemulihan lama ditolak setelah reset;
  - `/bantuan-akses` menjawab `202` seragam dengan waktu disamakan untuk akun ada, tidak ada, atau nonaktif; batas laju dan batas permintaan terbuka ditegakkan;
  - job reset tanpa catatan persetujuan lengkap, atau melewati batas harian, ditolak `worker`;
  - nomor lengkap, kode, dan password tidak muncul di audit, log, atau layar petugas; E-04 tercatat untuk setiap langkah;
  - sesi tautan pemulihan tanpa password saat ini tidak dapat mengikat nomor; `rs-reset-credentials-mfa` ditolak selama `rs_wa_reset_pending` dan 7 hari sesudah nomor terikat lewat pemulihan;
  - J2 saja (tanpa J3 syarat 2) pada perusahaan banyak-HR atau HR tunggal tidak dapat disetujui; Verifikator tidak dapat menyetujui tanpa J1 atau syarat 2;
  - permintaan yang ditutup sebagai penyalahgunaan tidak mengurangi kuota akun;
  - pembukaan bukti kewenangan oleh Verifikator memancarkan E-05;
  - DB menolak: dua permintaan terbuka untuk satu akun, penyetuju sama dengan pemverifikasi atau pemohon, konfirmasi rekan ganda atau dari pemohon, HR lintas perusahaan, token pembatalan dipakai dua kali atau kedaluwarsa, ciphertext penyangga dipindah ke niat lain.

## Butir keputusan user (P32-1–P32-7 sudah dijawab)

| No | Pertanyaan | Pilihan | Usulan |
| --- | --- | --- | --- |
| P32-1 | WhatsApp memproses isi pesan di server Meta, termasuk di luar Indonesia hingga 60 menit, sedangkan T4-Q1 "tanpa kecuali" dan CR-007 menghapus WhatsApp karena alasan ini | A: pengecualian terbatas T4-Q1 khusus kode masuk HR (nomor HR + kode, tanpa data kesehatan), dengan tinjauan Petugas PDP (Pasal 56) dan pemberitahuan di teks persetujuan HR. B: tanpa pengecualian, HR production tetap TOTP, dan WhatsApp/kode tetap hanya demo. C: SMS penyedia Indonesia (*restricted*, wajib menawarkan TOTP juga) | A bila Petugas PDP menyetujui; bila tidak, B  **Dijawab** `[USER]` 2026-11-28: "OTP WhatsApp diterapkan di semuanya", diperjelas "all environments" = A (kode masuk HR korporat lewat WhatsApp di semua lingkungan termasuk production; pengecualian T4-Q1/CR-007 khusus kode masuk HR). Kewajiban transfer lintas batas UU PDP Pasal 56 dan teks persetujuan HR tetap harus dipenuhi sebelum production (gerbang butir 10). |
| P32-2 | Pengirim WhatsApp | A: WhatsApp Business Platform resmi (langsung atau lewat mitra; ada biaya per pesan dan kontrak). B: agen yang mengotomatiskan akun WhatsApp biasa (melanggar ketentuan layanan `[SRC-998]`, nomor dapat diblokir) | A  **Dijawab** `[USER]` 2026-11-28: "use 3rd party" (terminal), diperjelas "P32-2 A" (Telegram) = WhatsApp Business Platform resmi lewat mitra (Business Solution Provider terdaftar Meta). Nama mitra, biaya per pesan, dan kontrak `[TIDAK DIKETAHUI]`, dipilih sebelum A32-T10. |
| P32-3 | Kode pemulihan wajib disimpan HR saat login pertama | A: wajib (satu layar tambahan). B: opsional (HR yang kehilangan nomor harus lewat Verifikator, Fase 2) | A **Dijawab** `[USER]` 2026-11-28 (terminal): "P32-3 B. HR can raise the ticket to ruang sehat support team" = kode pemulihan opsional; HR yang kehilangan nomor mengajukan tiket ke tim support Ruang Sehat. Dampak: jalur pemulihan lewat support (verifikasi identitas sebelum reset faktor kedua, rawan rekayasa sosial) harus dirancang; waktunya bergantung P32-7. |
| P32-4 | Kapan HR memasukkan nomor WhatsApp | A: saat login pertama setelah membuat password. B: di formulir `/daftar` dan undangan (pengundang mengetik nomor rekan), tetap diverifikasi saat login pertama | A  **Dijawab** `[USER]` 2026-11-28 (terminal): "B" = nomor WhatsApp diisi di formulir `/daftar` (pendaftar) dan di formulir Undang HR (pengundang mengetik nomor rekan), lalu tetap diverifikasi dengan kode saat login pertama; HR dapat mengoreksi nomor sebelum verifikasi. Dampak: PRD US-1901 dan alur undangan berubah, kontrak API pendaftaran/undangan bertambah field nomor, nomor belum terverifikasi tidak boleh dipakai untuk apa pun selain pengiriman kode verifikasi. |
| P32-5 | Nomor yang diterima | A: hanya nomor Indonesia (+62) di Fase 1. B: semua negara | A  **Dijawab** `[USER]` 2026-11-28 (terminal): "A" = hanya nomor Indonesia (+62) di Fase 1; formulir menerima awalan 0 dan menormalkannya ke E.164. |
| P32-6 | "Demo" memakai kode tetap di lingkungan mana | A: lokal dan staging (data sintetis). B: lokal saja | A  Catatan `[USER]` 2026-11-28 (Telegram): "saat demo pakai static" (menegaskan kode tetap di demo). **Dijawab** `[USER]` 2026-11-28 (terminal): "P32-6 A" = kode tetap berlaku di `local` dan `staging` (data sintetis); production selalu WhatsApp. |
| P32-7 | Jalur pemulihan oleh Verifikator (nomor dan kode pemulihan hilang) | A: tetap Fase 2 seperti rencana. B: dimajukan ke Fase 1 | A  **Dijawab** `[USER]` 2026-11-28 (terminal): "P32-7 B" = pemulihan lewat tiket ke tim support dimajukan ke Fase 1 (konsekuensi P32-3 B). Wajib verifikasi identitas sebelum reset nomor (konfirmasi HR aktif lain di perusahaan yang sama atau kontak terdaftar), audit trail, dan notifikasi ke pemilik akun; rincian dan task tambahan disusun di revisi ADR ini. |

### Penerapan jawaban user pada revisi ini

| Butir | Diterapkan di |
| --- | --- |
| P32-1 A | Butir 10 (cakupan pengecualian dan gerbang), header, risiko |
| P32-2 A | Butir 3 (templat autentikasi `[SRC-999]`), butir 10, A32-T10 |
| P32-3 B | Butir 2, 6, 7.1 (yang melemah dan kompensasi), butir 7.2, tabel risiko, ASVS, test negatif |
| P32-4 B | Butir 6 (aturan nomor belum terverifikasi, jalur data, validasi), daftar dampak PRD dan `openapi/`, A32-T12 |
| P32-5 A | Butir 6 (validasi dan normalisasi) |
| P32-6 A | Butir 4 (tidak berubah: kode tetap hanya `local` dan `staging`) |
| P32-7 B | Butir 7.3, tabel risiko, A32-T13–A32-T16 |

### Pertanyaan lanjutan (dari revisi 2026-11-28; belum dijawab, tidak mengubah usulan di atas)

| No | Pertanyaan | Usulan penyusun |
| --- | --- | --- |
| P32-8 | Siapa mengeksekusi pemulihan: dua orang (Layanan Pelanggan memverifikasi, Verifikator menyetujui) atau Layanan Pelanggan saja? Dua orang menjaga `[USER] C21-P4` (4a, reset di Verifikator) dan lebih tahan rekayasa sosial, tetapi menambah beban Verifikator | Dua orang |
| P32-9 | Kanal tiket bagi HR yang tidak dapat masuk: formulir publik `/bantuan-akses` (usulan) dan/atau email atau telepon Layanan Pelanggan. Alamat dan nomor kanal `[TIDAK DIKETAHUI]`. Jam layanan dan target waktu (SLA) support `[TIDAK DIKETAHUI]` | Formulir publik ditambah kanal petugas |
| P32-10 | Masa tunggu eksekusi 24 jam, kedaluwarsa permintaan 7 hari, jendela J1 48 jam sebelum J2: setuju? Masa tunggu berarti HR tanpa akses menunggu paling cepat 1 hari | Setuju; tinjau setelah ada data operasi |
| P32-11 | Boleh memberi tahu nomor WhatsApp lama saat pemulihan (pesan di luar kode masuk, di luar cakupan P32-1)? Bawaan: hanya email | Email saja di Fase 1 |
| P32-12 | Apakah formulir Verifikator membuat perusahaan (US-1903) juga memuat nomor WhatsApp HR pertama? Bawaan: tidak, HR mengisinya di login pertama | Tidak |
| P32-13 | Boleh satu nomor dipakai beberapa akun HR (tanpa syarat keunikan, menghindari enumerasi)? | Boleh |
| P32-14 | HR tunggal yang bukan pendaftar berkas (tanpa bukti kewenangan atas namanya) ditolak dan diarahkan ke penunjukan HR baru lewat Verifikator. Setuju tanpa jalur lain? | Setuju |
| P32-15 | Pengingat membuat kode pemulihan (banner di member area) selama HR belum punya? | Ya |
| P32-16 | Kebijakan koreksi nomor telepon kantor (butuh persetujuan Verifikator?) dan siapa yang boleh membekukan akun HR dalam keadaan darurat (perangkat dicuri)? Keduanya `[TIDAK DIKETAHUI]` di PRD yang ada | Koreksi nomor kantor butuh persetujuan Verifikator; Verifikator dapat menonaktifkan HR (US-207 AC1) |
| P32-17 | **Dicabut**: format dan panjang nomor seluler Indonesia dan batas E.164 adalah pekerjaan teknis, dikerjakan di A32-T12 (sumber resmi dibuka dan dicatat di `docs/sources.md`) | Bukan pertanyaan ke user |
| P32-18 | **Blokir.** Di perusahaan dengan lebih dari satu HR aktif, bolehkah J2 (telepon balik ke nomor kantor) saja menyetujui pemulihan tanpa konfirmasi HR rekan dan tanpa pencocokan nama dengan bukti kewenangan (J3 syarat 2)? Telepon dapat dijawab pihak tak berwenang | Tidak boleh (draf ini): J2 selalu butuh J3 syarat 2, atau Verifikator menolak dan mengarahkan ke penunjukan HR baru |
| P32-19 | **Blokir.** Akun HR dibuat saat `/daftar` (sebelum Verifikator menyetujui perusahaan), sehingga siapa pun dapat memicu kode WhatsApp ke nomor sembarang sampai batas laju. Pilihan A: dipertahankan dengan batas laju dan anggaran global. C: tautan aktivasi dikirim baru sesudah perusahaan Aktif (HR tidak dapat masuk untuk melihat status sebelum disetujui; mengubah US-1901 AC1, AC5, AC6) | A di draf ini; C bila user menerima HR tidak dapat melihat status sebelum persetujuan |
| P32-20 | **Blokir.** Siapa menangani HR yang mailbox-nya sudah diambil alih (alur ini tidak mencakupnya)? Belum ada perubahan email HR korporat | Verifikator menonaktifkan akun lama dan menunjuk HR baru dengan bukti kewenangan baru |

## Riwayat revisi

| Tanggal | Revisi | Sumber |
| --- | --- | --- |
| 2026-11-28 | Diusulkan: autentikator Keycloak tipis + `otp-relay` Rust, port pengirim `static`/`whatsapp`, penjaga kode tetap tiga lapis, parameter OTP, nomor sebagai atribut Keycloak yang didaftarkan HR saat login pertama, kode pemulihan wajib, revisi KEPUTUSAN-265, gerbang production T4-Q1. Ruang lingkup hanya `korporat` | `[USER]` CR-041; `[USER]` 2026-11-28 "only korporat realm, others stay TOTP"; KEPUTUSAN-460–469 |
| 2026-11-28 | Revisi atas jawaban P32-1–P32-7: pengecualian WhatsApp di production dengan gerbang (butir 10); BSP resmi dan templat autentikasi; kode pemulihan opsional dengan analisis yang melemah (butir 7.1); nomor di `/daftar` dan Undang HR dengan invarian nomor belum terverifikasi, jalur data terenkripsi, dan atribut `rs_wa_phone_pending` (butir 6); hanya +62 dengan normalisasi E.164; kode tetap hanya `local`/`staging`; pemulihan lewat tiket support di Fase 1 (butir 7.3) dengan amandemen ADR-030 dan ADR-031; tabel risiko, ASVS, dampak dokumen, test negatif, dan rencana task (A32-T12–A32-T16, total ±107 jam-task) diperbarui; pertanyaan lanjutan P32-8–P32-17 | `[USER]` 2026-11-28 jawaban P32-1–P32-7; SRC-999; KEPUTUSAN-464, -465, -469, -470, -471 |
| 2026-11-28 | Revisi atas temuan review keamanan dan DBA (TIDAK LULUS): sesi tautan pemulihan wajib password saat ini dan blokir `rs-reset-credentials-mfa` 7 hari; J2 tidak pernah cukup sendirian dan layar J1 memuat instruksi kanal langsung; pendaftaran nomor hanya di sesi tautan; MAC berkunci `K_host` (OpenBao) dan catatan sesi tidak dipersistenkan; risiko BSP/Meta, DPA di gerbang, agen hanya menerima `to` dan `code`; admin event tanpa representasi; `deploy-env` kosong = production; `maxTemporaryLockouts`; kuota tidak menghitung permintaan palsu; konsistensi dokumen; butir 12 model data (tabel, hak, batas DB, pembersihan, enkripsi); estimasi menjadi ±107 jam-task; pertanyaan P32-18–P32-20, P32-17 dicabut | Review security-architect dan dba-reviewer; KEPUTUSAN-472–474 |
