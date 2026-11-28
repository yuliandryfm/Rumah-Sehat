# ADR-032: MFA HR realm `korporat` memakai kode sekali pakai lewat WhatsApp: autentikator Keycloak kustom yang tipis, `otp-relay` Rust dengan port pengirim, dan kode tetap khusus demo

Status: **Diusulkan** (usulan, 2026-11-28). Arah ditetapkan user `[USER]` CR-041 (2026-11-28). Dokumen ini menunggu review T1 (keamanan, code-reviewer) dan persetujuan user. **Adaptor `whatsapp` di production terblokir** sampai user memutuskan P32-1 (bentrok dengan T4-Q1/CR-007, butir 10).
Mengubah bila diterima: ADR-011 butir 3 (baris realm `korporat`) dan butir 5 (faktor kedua dan pemulihan HR); ADR-027 butir 3 (MFA TOTP dan pemulihan MFA HR); ADR-024 (Java/Maven masuk stack khusus provider Keycloak, image Keycloak kustom); ADR-025 butir 7 (identitas OpenBao `otp-relay`); KEPUTUSAN-253 dan KEPUTUSAN-265; ADR-016 dan CR-007 hanya bila user memberi pengecualian di P32-1.
Tidak mengubah: realm `pasien`, `nakes`, `admin`, `rme` (tetap TOTP, keputusan user, butir 9); ADR-030 (tidak ada jenis job baru di Fase 1; reset MFA HR Fase 2 tetap butuh revisi ADR-030 seperti sudah dicatat di ADR itu); ADR-031 (formulir `/daftar` tidak bertambah isian); ADR-026 (jalur email tetap).
Tanggal: 2026-11-28
Tingkat ketelitian: T1 (autentikasi, sesi, dan MFA; konfigurasi realm Keycloak; pipeline dan image; `aturan-kerja.md` §1)

## Konteks

**Keputusan user.**
- Faktor kedua akun HR realm `korporat` berganti dari TOTP aplikasi authenticator menjadi kode sekali pakai lewat WhatsApp, tanpa aplikasi authenticator: "i want move to WhatsApp OTP as MFA. just need develop WA Agent later. no need other apps as authenticator. its cheapest and easy to handle" `[USER]` CR-041.
- Masa transisi: "on demo, OTP should be static value, example \"A1B2C3\"" `[USER]` CR-041.
- Ruang lingkup: "only korporat realm, others stay TOTP" `[USER]` 2026-11-28 (terminal, diteruskan orkestrator).

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

**Standar.**
- NIST SP 800-63B-4 `[SRC-990]`:
  - §3.1.3.1: perangkat out-of-band boleh diautentikasi lewat SIM jaringan seluler hanya bila rahasia dikirim "via the PSTN (i.e., SMS or voice) or an encrypted instant messaging service". "Email SHALL NOT be used for out-of-band authentication."
  - §3.1.3.2: rahasia acak "at least six decimal digits" (atau setara); "the authentication SHALL be considered invalid unless completed within 10 minutes"; diterima "only once during the validity period"; bila rahasia di bawah 64 bit, verifier wajib membatasi laju (§3.2.2); "Generating a new authentication secret SHALL NOT reset the failed authentication count."
  - §3.1.3.3 hanya membatasi PSTN: "Use of the PSTN for out-of-band verification is restricted". Kewajiban "alternative authenticator types are available to all subscribers" dan pertimbangan "device swap, SIM change, number porting" ada di bagian PSTN ini. "Setting or changing the pre-registered telephone number is considered to be the binding of a new authenticator."
  - §3.2.2: kegagalan berturut-turut dibatasi "no more than 100", dan instansi boleh memakai batas lebih rendah.
  - **Koreksi atas catatan CR-041:** pesan instan terenkripsi disebut terpisah dari PSTN dan tidak berstatus *restricted*. Kewajiban jenis autentikator alternatif di §3.1.3.3 melekat pada PSTN, bukan pada WhatsApp. Kode pemulihan tetap diusulkan (butir 7), dengan dasar ASVS 6.4.4, pemulihan akun NIST §4.2, dan ketersediaan.
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
  - menyimpan MAC kode, salt, waktu kedaluwarsa, dan penghitung percobaan di catatan sesi autentikasi Keycloak (nama API diverifikasi saat task `[TIDAK DIKETAHUI]`);
  - menampilkan formulir.
- Isi dari `action()`:
  - menormalkan masukan, menghitung MAC, dan membandingkan dalam waktu konstan;
  - bila cocok: menghapus catatan (sekali pakai), lalu `success()`;
  - bila salah: `failureChallenge()`, sehingga kegagalan masuk deteksi brute force realm `[SRC-993]` `[SRC-273]`.
- Kode otorisasi OIDC baru terbit setelah seluruh alur dan required action selesai; perilaku ini sudah diuji untuk OTP dan Configure OTP di realm `pasien`/`admin` `[DOK: infra/keycloak/README.md]` dan diuji ulang untuk provider baru. Karena itu sesi Keycloak, token, dan sesi BFF hanya ada setelah faktor kedua lolos. Tidak ada token "setengah MFA" yang perlu dijaga aplikasi.
- Perbandingan opsi ada di bagian "Opsi yang dibandingkan".

### 2. Alur realm `korporat` `[KEPUTUSAN-460]` `[KEPUTUSAN-466]`

- **Login browser:** password (REQUIRED). Subalur faktor kedua (`conditional-user-configured`) memuat `rs-wa-otp-form` dan `auth-recovery-authn-code-form` sebagai ALTERNATIVE. Bila belum ada faktor kedua, `rs-wa-otp-form` menjadi REQUIRED dengan *user setup*, sehingga `rs-wa-enroll` dipaksa sebelum kode otorisasi terbit (pola `rs-browser-otp-default` sekarang).
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
- Relay **membangkitkan** kode dan mengirimnya lewat adaptor. Keycloak hanya menerima `HMAC-SHA256(salt, kode)` dengan salt acak 128 bit per tantangan, tidak pernah teks kodenya.

**Port pengirim** (`OtpSender`, trait Rust). Dipilih lewat `OTP_SENDER` tanpa nilai bawaan:
- `static`: kode = nilai konfigurasi, tanpa pengiriman. Hanya `local`/`staging` (butir 4).
- `whatsapp`: kode acak dari CSPRNG OS, dikirim lewat agen WA. Agen dibangun kemudian `[USER]`. Penyedianya `[TIDAK DIKETAHUI]` dan menjadi keputusan user (P32-2).
- Jenis adaptor berupa enum di kode. Nilai lain membuat start gagal.

**Kontrak Keycloak → relay** (nama final ditetapkan saat task):

| Butir | Isi |
| --- | --- |
| Permintaan | `POST /otp/v1/challenges`, header `Authorization: Bearer …` dan `Idempotency-Key` = SHA-256 atas (ID sesi autentikasi, tab, nomor urut tantangan). Badan: `realm`, `account_ref` (ID pengguna Keycloak), `recipient` (E.164), `purpose` (`login` / `enroll` / `change` / `reset`), `locale` |
| Sukses | `201`: `salt`, `mac`, `expires_in_s`, `sender_mode` (`static` / `whatsapp`), `recipient_hint` (4 digit terakhir) |
| Gagal | `401` token salah; `422` nomor ditolak permanen (format, bukan pengguna WhatsApp bila agen dapat mengetahuinya); `429` batas laju, dengan `retry_after_s`; `503` agen gagal sementara atau batas waktu |
| Batas waktu | Keycloak → relay 7 detik. Gagal atau batas waktu menghasilkan pesan umum "Kode belum dapat dikirim, coba lagi", **tanpa jalan pintas** melewati faktor kedua (gagal-tertutup). Kode pemulihan tetap tersedia |
| Idempotensi | Kunci yang sama dalam 10 menit mengembalikan hasil sama tanpa kirim ulang (cache memori per host). Kirim ulang oleh pengguna = tantangan baru dengan kunci baru |

**Kontrak relay → agen WA** (adaptor `whatsapp`; agen dibangun kemudian):

| Butir | Isi |
| --- | --- |
| Permintaan | `POST <agen>/v1/otp-messages`, autentikasi klien (token atau mTLS, ditetapkan bersama agen), `Idempotency-Key` diteruskan dari relay. Badan: `to` (E.164), `code`, `ttl_s`, `locale`, `template` (jenis tetap per `purpose`) |
| Hasil | `202 accepted` + `message_id` = diterima penyedia; `4xx` permanen (nomor tidak valid, bukan pengguna WhatsApp, templat ditolak); `429`/`5xx`/batas waktu = sementara |
| Batas waktu | Sambung 1 detik, total 5 detik. Tanpa coba ulang otomatis di relay; pengguna menekan "Kirim ulang" setelah jeda |
| Idempotensi | Agen wajib menolak kiriman ganda dengan kunci yang sama selama minimal 15 menit |
| Larangan | Tanpa jatuh ke SMS (PSTN memicu ASVS 6.6.1 dan status *restricted* NIST `[SRC-992]` `[SRC-990]`). Agen tidak mencatat kode atau nomor lengkap di log, tidak menyimpan kode setelah dikirim, dan tidak memuat tautan di pesan |
| Teks pesan (draf) | "Kode masuk Ruang Sehat: {kode}. Berlaku 5 menit. Jangan berikan kode ini kepada siapa pun, termasuk petugas Ruang Sehat." Teks final mengikuti templat yang disetujui penyedia `[TIDAK DIKETAHUI]` |
| Status kirim | Laporan terkirim/dibaca/gagal dari penyedia tidak masuk ke relay di Fase 1 (relay tanpa port masuk). Metrik ada di agen |

### 4. Penjaga kode tetap dan masa transisi demo `[KEPUTUSAN-462]`

- **Lapis 1, relay (pola KEPUTUSAN-403):** `DEPLOY_ENV=production` dengan `OTP_SENDER=static` membuat `OtpRelayConfig::load` gagal. Pesannya hanya menyebut nama variabel. `OTP_SENDER` tanpa nilai juga gagal start (tidak ada bawaan `static`).
- **Lapis 2, Keycloak (independen dari relay):** SPI autentikator menerima `deploy-env` dari variabel lingkungan Keycloak. Bila `production` dan relay menjawab `sender_mode=static`, tantangan ditolak dan login gagal-tertutup. Apakah galat di `init()` factory menghentikan start Keycloak `[TIDAK DIKETAHUI]`; penjaga runtime ini berlaku apa pun hasilnya.
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
- Alur di demo sama dengan production: pendaftaran nomor, layar kode, batas percobaan, dan kode pemulihan. Hanya pengirimannya yang tidak ada. Kode tetap tetap sekali pakai per tantangan.

### 5. Parameter OTP `[KEPUTUSAN-463]`

| Parameter | Nilai | Dasar |
| --- | --- | --- |
| Panjang dan alfabet | 6 karakter dari 31 simbol: angka 2–9 dan huruf A–Z tanpa I, L, O (tanpa 0/1/I/L/O yang mirip). 31^6 ≈ 2^29,7 | ASVS 6.5.4 ≥ 20 bit `[SRC-992]`; NIST ≥ 6 digit desimal atau setara `[SRC-990]`; bentuk alfanumerik mengikuti contoh user `[USER]` |
| Normalisasi masukan | Pangkas spasi dan tanda hubung, ubah ke huruf besar | Kode diketik HR dari ponsel ke web |
| Umur | 5 menit | ≤ 10 menit (NIST §3.1.3.2, ASVS 6.5.5) `[SRC-990]` `[SRC-992]` |
| Sekali pakai | Catatan dihapus saat sukses. Tantangan baru membatalkan yang lama. Hanya berlaku di sesi autentikasi asalnya | ASVS 6.5.1, 6.6.2 `[SRC-992]` |
| Percobaan per kode | Maksimal 5 salah, lalu kode batal dan HR meminta kode baru | ASVS 6.6.3 `[SRC-992]` |
| Penguncian akun | Setiap kode salah dihitung deteksi brute force realm (5 gagal, kunci 15 menit, ADR-011 butir 4). Penghitung per pengguna tidak direset oleh kode baru | NIST §3.1.3.2, §3.2.2 `[SRC-990]`; `[SRC-273]` |
| Jeda kirim ulang | 60 detik (dicek di Keycloak sebelum memanggil relay) | Keputusan teknis |
| Batas laju di relay | Per akun: 5 kirim per 15 menit. Per nomor: 10 kirim per 24 jam. Pendaftaran/ubah nomor: 3 nomor berbeda per akun per 24 jam. Anggaran global per menit dapat dikonfigurasi (biaya pesan). Kunci di-hash, di memori per host, sehingga anggaran efektif ±2 kali di dua host. Saat batas tercapai: `429`, tanpa kirim | ASVS 6.6.3 `[SRC-992]`; pola ADR-026 butir 4; angka adalah usulan teknis |
| Penyimpanan | `HMAC-SHA256(salt 128 bit, kode)` di catatan sesi autentikasi Keycloak; tidak di database aplikasi | Keycloak tidak pernah menyimpan teks kode |
| Log | Kode tidak pernah dicatat di relay, Keycloak (detail event), audit, atau security log. Nomor hanya tampil sebagai 4 digit terakhir | ADR-005, ADR-010 |

### 6. Nomor WhatsApp HR `[KEPUTUSAN-464]`

**Pengumpulan.**
- Nomor **tidak** diminta di `/daftar` (ADR-031) maupun di formulir undangan HR atau pembuatan perusahaan oleh Verifikator.
- HR memasukkan nomornya sendiri di required action `rs-wa-enroll` pada login pertama, setelah password diatur lewat tautan aktivasi (KEPUTUSAN-303). Posisinya sama dengan pendaftaran TOTP sekarang.
- Kepemilikan nomor dibuktikan saat itu juga: kode dikirim ke nomor itu dan harus dimasukkan sebelum nomor terikat (NIST §3.1.3.3: menetapkan nomor = mengikat autentikator baru `[SRC-990]`).
- Akibatnya HR pengundang tidak perlu tahu nomor rekannya. Nomor juga tidak melewati `api`, `worker`, baris niat, atau database aplikasi, sehingga ADR-030 dan ADR-031 tidak berubah.

**Penyimpanan.**
- Atribut user profile Keycloak `rs_wa_phone`:
  - format E.164, Fase 1 hanya `+62` (P32-5);
  - izin lihat dan ubah hanya Admin (pola `rs_intent_id` `[SRC-845]`);
  - tanpa mapper token;
  - ditulis oleh required action setelah verifikasi.
- Nomor diperlakukan sebagai pengikatan autentikator, setara rahasia TOTP yang selama ini disimpan Keycloak. ADR-011 butir 6 ("Keycloak hanya menyimpan data identitas minimal ... dan kredensial") tetap terpenuhi.
- Database `keycloak` berada di instans (a) di Indonesia dan ikut PITR 35 hari (ADR-011 butir 2, ADR-025 butir 5).

**Ubah nomor.**
- Dari member area, tautan "Ubah nomor WhatsApp" memicu `rs-wa-enroll` sebagai AIA.
- Langkahnya:
  1. autentikasi ulang dengan password dan nomor lama atau kode pemulihan (`getMaxAuthAge` = 0 `[SRC-994]`);
  2. kode dikirim ke nomor baru dan harus dimasukkan;
  3. nomor lama tetap terikat sampai nomor baru terverifikasi (pola H-16 a).
- Sesudahnya: email pemberitahuan ke alamat akun (NIST §4.1.2.1 `[SRC-991]`) dan E-04.
- Mekanisme email pemberitahuan dari required action kustom `[TIDAK DIKETAHUI]`. Bila Keycloak tidak dapat mengirimnya, pemberitahuan dikirim worker dari event yang diambil, bersama US-1001 (Fase 2), dan kesenjangan itu dicatat sebagai risiko sisa.

**Privasi (UU PDP).**
- Tujuan pemrosesan nomor hanya autentikasi: tanpa notifikasi atau pemasaran, sesuai CR-007 untuk notifikasi.
- Teks persetujuan atau pemberitahuan HR (menunggu L-20) wajib menyebut pemakaian nomor untuk kode masuk lewat WhatsApp beserta penyedianya `[SRC-15]` (Pasal 21).
- Nomor terhapus bersama pengguna Keycloak. Prosedur hak subjek data (EP-16) dan inventaris pemrosesan atau DPIA ditambah butir ini.
- Transfer ke penyedia WhatsApp = butir 10.

### 7. Faktor alternatif dan pemulihan (revisi KEPUTUSAN-265) `[KEPUTUSAN-465]`

- **Kode pemulihan bawaan Keycloak (12 kode sekali pakai) wajib dibuat** setelah nomor pertama terverifikasi, di login yang sama (`rs-wa-enroll` menambahkan required action kode pemulihan; mekanismenya diverifikasi saat task, KEPUTUSAN-203). Kode pemulihan adalah jenis autentikator kedua tanpa aplikasi tambahan dan berlaku sebagai ALTERNATIVE di login dan atur ulang password.
- **Ganti ponsel dengan nomor yang sama:** HR tidak perlu reset; kode tetap terkirim ke nomor itu.
- **Nomor hilang, kode pemulihan masih ada:** masuk dengan kode pemulihan, lalu "Ubah nomor WhatsApp" (butir 6). Setelah login dengan kode pemulihan, layar menawarkan ganti nomor dengan pilihan "Nomor saya tetap", karena kode pemulihan juga dipakai saat WhatsApp sedang tidak dapat diakses.
- **Nomor dan kode pemulihan sama-sama hilang:** jalur KEPUTUSAN-265 tetap, dengan kata "TOTP" diganti "nomor WhatsApp":
  - reset hanya dimulai Verifikator, setelah konfirmasi lewat kanal berbeda (HR aktif lain atau kontak resmi perusahaan);
  - reset berlaku setelah HR membuka tautan sekali pakai di email akun dan memasukkan password;
  - nomor lama tetap terikat sampai nomor baru terverifikasi;
  - semua sesi dicabut, semua HR aktif diberi tahu, dan E-04 tercatat;
  - tidak ada jalur Super Admin;
  - dibangun di Fase 2 bersama US-207 AC5, dengan revisi ADR-030 seperti sudah dicatat di ADR itu.
- Setiap pemulihan memicu pemberitahuan ke HR (NIST §4.2 `[SRC-991]`).

### 8. Audit, log, dan pemantauan `[KEPUTUSAN-467]`

**Audit.**
- E-01/E-02: dari event login Keycloak yang diambil worker (ADR-011 butir 10), dengan detail faktor (`wa_otp` atau `rcode`) tanpa kode dan tanpa nomor.
- E-04: pengikatan nomor pertama, ubah nomor, penghapusan pengikatan (reset Fase 2), dan pembuatan ulang kode pemulihan.
- Jenis event yang dipancarkan required action atau autentikator kustom dan cara menambah detailnya `[TIDAK DIKETAHUI]`. Pemetaannya ditetapkan per versi saat task (pola ADR-011 butir 10) dan diamati `keycloak-check`.

**Security log (ADR-010).**
- Relay mencatat hasil per tantangan: mode, `purpose`, kelas hasil, latensi, dan ID korelasi. Tidak mencatat kode, nomor, atau badan permintaan.

**Peringatan.**
- Laju galat agen WA.
- Lonjakan permintaan per akun atau global (indikasi penyalahgunaan biaya pesan).
- Kegagalan lapis 2 butir 4 di production (`sender_mode=static` ditolak).
- `otp-relay` tidak terjangkau.

### 9. Ruang lingkup: hanya realm `korporat` `[USER]`

Realm `pasien`, `nakes`, `admin`, dan `rme` tetap TOTP dengan kode pemulihan: "only korporat realm, others stay TOTP" `[USER]` 2026-11-28. JAR provider terpasang di deployment Keycloak bersama, tetapi hanya dirujuk alur realm `korporat`. `keycloak-check` menguji bahwa alur realm lain tidak merujuk `rs-wa-otp-form` atau `rs-wa-enroll`.

### 10. Gerbang production adaptor `whatsapp`: bentrok dengan T4-Q1 dan CR-007 `[KEPUTUSAN-469]`

- Mengirim kode lewat WhatsApp berarti nomor HR dan isi pesan diproses Meta. Isi pesan dapat tersimpan di pusat data Meta di luar Indonesia selama diproses, hingga 60 menit, walaupun *local storage* Indonesia dipakai `[SRC-54]` `[SRC-997]`.
- Ini bertentangan dengan T4-Q1 "tanpa kecuali" dan dengan dasar CR-007 yang menghapus WhatsApp `[USER] T4-Q1` `[DOK: backlog.md CR-007]`. Agen WA buatan sendiri tidak mengubah fakta ini, karena nomor bisnis hanya dapat memakai Cloud API `[SRC-55]`.
- **Akibatnya:**
  - adaptor `whatsapp` tidak boleh aktif di production sampai user memutuskan P32-1, dan, bila pengecualian diberikan, Petugas PDP meninjau syarat Pasal 56 `[SRC-15]`;
  - adaptor `static` juga tidak boleh di production (butir 4);
  - jadi **login HR di production terblokir sampai P32-1 diputuskan**;
  - rancangan butir 1–8 tidak bergantung pada pilihan penyedia dan dapat dikerjakan untuk demo.
- Agen tidak resmi yang mengotomatiskan akun WhatsApp biasa bertentangan dengan ketentuan layanan WhatsApp `[SRC-998]`. Nomor pengirim dapat diblokir, dan bila itu terjadi, semua HR hanya dapat masuk dengan kode pemulihan (P32-2).

### 11. Toolchain Java dan image Keycloak `[KEPUTUSAN-468]`

- Modul Maven di `infra/keycloak/providers/rs-wa-otp/`:
  - artefak SPI Keycloak 26.8.2 dengan scope `provided`, ditambah JDK saja (klien HTTP `java.net.http`);
  - tanpa dependensi runtime pihak ketiga;
  - nama artefak, versi JDK runtime image Keycloak, dan versi Maven dicek di registry resmi saat task `[TIDAK DIKETAHUI]` (anti-halu §5).
- Image Keycloak kustom multi-tahap: build JAR, lalu `ADD` ke `/opt/keycloak/providers/` sebelum `kc.sh build`, lalu start `--optimized` `[SRC-996]`. Basis tetap `quay.io/keycloak/keycloak:26.8.2` yang dipin digest. Tema dapat ikut dipaketkan sebagai JAR di langkah yang sama.
- Image dibangun CI, dipindai Trivy dan osv-scanner (`pom.xml`), dirujuk lewat digest, dan ditandatangani cosign (ADR-025 butir 10).
- Langkah build dan test Maven masuk `scripts/verify.sh`. Ini menambah langkah, tidak melonggarkan; izin user tetap diminta bila dianggap mengubah verifikasi.
- Setiap upgrade Keycloak (termasuk patch keamanan) wajib menjalankan ulang `keycloak-check` untuk realm `korporat`, karena SPI-nya internal `[SRC-993]`.

## Opsi yang dibandingkan

| Kriteria | A1: autentikator Java memegang semua (kode, pengirim, kode tetap) | **A2 (usulan): autentikator Java tipis + `otp-relay` Rust** | B: langkah OTP di BFF/backend setelah login Keycloak | Status quo: TOTP |
| --- | --- | --- | --- | --- |
| Tidak dapat dilewati; token baru ada setelah OTP | Ya: kode otorisasi terbit setelah alur selesai | Ya, sama dengan A1 | Tidak secara struktural. Token Keycloak bernilai 1 faktor dan sah di API sampai backend menolak lewat penanda MFA per sesi (`sid`) pada **setiap** endpoint dan proses; satu endpoint terlewat = jalan pintas (pola TD-15) | Ya |
| Kesesuaian ADR-011 (`acr`/`amr`) | `amr` lewat nilai referensi `[SRC-995]` | Sama dengan A1 | `amr`/`acr` dari Keycloak hanya `pwd`; status MFA hidup di luar token | `acr`=1 tanpa `amr` `[SRC-682]` (TD-27) |
| Brute force, kode pemulihan, atur ulang ber-MFA | Memakai bawaan Keycloak `[SRC-273]` `[SRC-68]` | Sama dengan A1 | Harus dibangun ulang di backend: penguncian, kode pemulihan (hash sesuai ASVS 6.5.2), dan atur ulang password Keycloak yang kini tanpa faktor kedua sebelum password baru (melemahkan KEPUTUSAN-199) | Bawaan |
| Penjaga kode tetap | Di Java; perilaku gagal start Keycloak `[TIDAK DIKETAHUI]`; tidak memakai pola `DEPLOY_ENV` Rust | Pola KEPUTUSAN-403 di Rust, ditambah lapis Keycloak independen | Pola Rust | Tidak perlu |
| Radius dampak T1 | Kode kustom berjalan di JVM Keycloak bersama semua realm (ketersediaan semua realm bila provider gagal dimuat). Kredensial agen WA ada di Keycloak | JVM sama dengan A1, tetapi logika di Java minimal. Kredensial agen WA hanya di relay. Kompromi relay = kode dapat dipalsukan (butuh password juga) | Tanpa Java. Tetapi logika MFA tersebar di BFF, backend, dan proses `realtime` | Tidak ada perubahan |
| Pola dan toolchain baru | Java/Maven, image kustom | Java/Maven, image kustom, satu subcommand Rust (pola `email-relay`) | Status sesi MFA baru di authn backend dan BFF | — |
| Estimasi | ±45 jam-task | ±48 jam-task (bagian "Rencana task") | ±44 jam-task, ditambah penguatan atur ulang yang belum terancang | 0 |
| Memenuhi keputusan user | Ya | Ya | Ya | Tidak (user memilih WhatsApp) |

A2 dipilih karena memberi sifat keamanan A1 (faktor kedua di dalam IdP, memakai ulang brute force, kode pemulihan, dan atur ulang ber-MFA) dengan effort yang setara B. A2 juga menaruh bagian yang paling sering berubah (pengirim, kode tetap, batas laju biaya) di Rust, yang punya pola penjaga, test, dan review yang sudah berjalan.

## Alternatif yang ditolak

- **B (BFF/backend):** lihat tabel. Selisih effort kecil, sedangkan token 1 faktor yang beredar dan pelemahan atur ulang password adalah mundur dari ADR-011 butir 5 dan 8.
- **A1:** kredensial agen WA dan kode tetap berada di Keycloak, yang dipakai semua realm. Penjaga production tidak mengikuti pola `DEPLOY_ENV` yang sudah teruji.
- **OTP email:** dilarang NIST §3.1.3.1 dan pengantar ASVS V6.6 `[SRC-990]` `[SRC-265]`.
- **SMS penyedia Indonesia** (bila WhatsApp tidak diizinkan): PSTN berstatus *restricted* (NIST §3.1.3.3) dan ASVS 6.6.1 mensyaratkan opsi lebih kuat seperti TOTP ikut ditawarkan `[SRC-990]` `[SRC-992]`. Tidak diusulkan, tetapi tetap disebut di P32-1 sebagai pilihan user.
- **Pengirim WhatsApp langsung dari Keycloak tanpa relay:** sama dengan A1.
- **Tetap TOTP:** bertentangan dengan keputusan user CR-041. Tetap menjadi jalan keluar bila P32-1 = B.

## Konsekuensi

**Positif.**
- HR tidak perlu memasang aplikasi authenticator. Login pertama: atur password, masukkan nomor WhatsApp, masukkan kode, simpan kode pemulihan. Login harian: password lalu kode WhatsApp.
- Token korporat dapat membawa bukti faktor kedua (`amr`), yang menutup kesenjangan TD-27 untuk realm ini bila verifikasi butir 2 berhasil.
- Kode tetap demo mustahil aktif di production lewat tiga lapis independen.

**Negatif.**
- Java/Maven dan image Keycloak kustom masuk stack, sehingga ADR-024 berubah. Ini membalik alasan ADR-026 untuk kasus ini, karena tidak ada cara memasang faktor baru di alur Keycloak tanpa SPI.
- Setiap upgrade Keycloak, termasuk patch keamanan, butuh uji ulang provider karena SPI internal `[SRC-993]`. Patch darurat dapat tertahan bila SPI berubah.
- Komponen baru (`otp-relay`, agen WA) menjadi syarat login HR. Bila agen WA mati, HR hanya dapat masuk dengan kode pemulihan.
- Effort Fase 1 bertambah ±48 jam-task, sebagian menggantikan bagian TOTP F1-T85/F1-T87/F1-T88 yang sudah dibangun.

### Risiko sisa

| Risiko | Penanganan | Register |
| --- | --- | --- |
| Bentrok T4-Q1/CR-007: isi pesan diproses Meta di luar Indonesia `[SRC-997]` | Gerbang production butir 10; P32-1; tinjauan Petugas PDP (Pasal 56 `[SRC-15]`) bila ada pengecualian | Risk register (baru) |
| Agen tidak resmi diblokir WhatsApp `[SRC-998]` | P32-2: rekomendasi API resmi; kode pemulihan sebagai cadangan | Baru |
| Kompromi `otp-relay` memungkinkan pemalsuan kode (penyerang tetap butuh password) | Per host, jaringan Docker khusus, tanpa port masuk, TLS + token pembawa, identitas OpenBao minimum, peringatan `sender_mode=static` di production | Threat model (baru) |
| Pengambilalihan akun WhatsApp atau nomor (ganti SIM, porting) memberi faktor kedua ke penyerang | Password tetap wajib, penguncian 5/15, pemberitahuan ubah nomor; indikator risiko ganti SIM bergantung pada penyedia `[TIDAK DIKETAHUI]` | KOR-01 (diperbarui) |
| Phishing seketika atas password dan kode | Sama dengan TOTP; faktor tahan phishing hanya wajib di L3 `[SRC-276]` | KOR-01 |
| Penyalahgunaan biaya pesan lewat pendaftaran nomor sembarang (akun HR dapat dibuat mandiri lewat `/daftar`) | Batas 3 nomor per akun per 24 jam, batas per nomor dan global, peringatan lonjakan | Baru |
| Perubahan SPI Keycloak saat upgrade | Provider minimal, `keycloak-check` wajib per upgrade, versi dipin | Baru |
| Provider gagal dimuat sehingga Keycloak gagal start untuk semua realm | Uji start image di CI dan staging sebelum production; rollback ke digest sebelumnya | Baru |
| Kode tetap demo bocor | Hanya lingkungan data sintetis; nilai per lingkungan di `.env.local`/OpenBao; dapat diganti kapan saja | — |

## Dampak ke checklist ASVS (`docs/05-asvs-checklist.md` V6)

| Requirement | Sebelum (realm `korporat`) | Sesudah |
| --- | --- | --- |
| Ruang lingkup V6 | Tidak menyebut `korporat` | Tambah realm `korporat` dan `otp-relay` |
| 6.3.3 (L2) | TOTP + kode pemulihan | Password + kode WhatsApp (out-of-band) atau kode pemulihan; tetap MFA `[SRC-265]` |
| 6.5.1 (L2) | TOTP sekali pakai | Kode OOB sekali pakai per tantangan; kode pemulihan sekali pakai `[SRC-992]` |
| 6.5.3 (L2) | Seed TOTP oleh Keycloak | Kode oleh CSPRNG OS di relay (crate dicek registry saat task) |
| 6.5.4 (L2) | — | ±29,7 bit ≥ 20 bit |
| 6.5.5 (L2) | TOTP 30 detik | Kode OOB 5 menit ≤ 10 menit |
| 6.6.1 (L2) | Tidak berlaku | Tetap tidak berlaku selama tanpa SMS; kontrak agen melarang jatuh ke SMS |
| 6.6.2 (L2) | Tidak berlaku | Kode terikat sesi autentikasi asal |
| 6.6.3 (L2) | Tidak berlaku | Batas 5 per kode + brute force realm + batas laju kirim |
| 6.4.3 (L2) | OTP sebelum password baru | Kode WhatsApp atau kode pemulihan sebelum password baru |
| 6.4.4 (L2) | KEPUTUSAN-265 (TOTP) | Butir 7 (nomor WhatsApp) |
| 6.1.3, 6.3.4 (L2) | Alur terdokumentasi | Jalur baru (pendaftaran nomor, ubah nomor lewat AIA) didokumentasikan; direct grant tetap tertutup |
| Celah N-20 (TOTP SHA1) | Berlaku | Tidak berlaku untuk `korporat`; tetap untuk realm lain |

## Dampak ke dokumen lain (diubah setelah ADR ini diterima; PRD belum diubah sekarang)

| Dokumen | Bagian | Perubahan |
| --- | --- | --- |
| PRD | NFR-SEC-02 | Faktor kedua HR = kode sekali pakai lewat WhatsApp + kode pemulihan. Realm lain tetap TOTP. "SMS, OTP email, WebAuthn tidak dipakai" tetap |
| PRD | US-1904 AC1, AC7, AC8, AC9; AC baru | AC1: password + kode WhatsApp; pendaftaran nomor dan kode pemulihan di login pertama. AC7: kode pemulihan, lalu ubah nomor. AC8/AC9: kata TOTP diganti nomor WhatsApp. AC baru: ubah nomor (autentikasi ulang, verifikasi nomor baru, pemberitahuan, E-04) dan batas kirim ulang/percobaan |
| PRD | US-1903 AC1 | "undangan aktivasi (password dan TOTP)" diganti "undangan aktivasi (password); nomor WhatsApp didaftarkan saat login pertama" |
| PRD | US-1901 | Berubah: P32-4 = B (nomor WhatsApp di formulir `/daftar` dan undangan HR) |
| PRD dan BRD | §2.2 non-scope, BR-050, BR-071, CR-007, PT-22 | Hanya bila P32-1 = A: WhatsApp dipakai **khusus kode masuk HR**, bukan notifikasi |
| ADR-011 | Butir 3 (baris `korporat`), butir 5, Konsekuensi | Faktor kedua HR; pemulihan; WhatsApp dikecualikan dari kalimat "SMS dan OTP email tidak dipakai" hanya untuk `korporat` |
| ADR-027 | Butir 3 | MFA TOTP diganti kode WhatsApp + kode pemulihan; pemulihan MFA HR mengikuti butir 7 |
| ADR-024 | Stack | Java/Maven khusus provider Keycloak; image Keycloak kustom |
| ADR-025 | Butir 2, 7 | Kontainer `otp-relay` per host; identitas dan policy OpenBao baru; path KV kode tetap hanya staging |
| ADR-016 | Konteks dan alternatif | Hanya bila P32-1 = A: catatan pengecualian MFA |
| KEPUTUSAN-253, -265, -85, -203 | — | KEPUTUSAN-253 digantikan untuk `korporat`; KEPUTUSAN-265 direvisi (butir 7); parameter TOTP KEPUTUSAN-85 tidak berlaku untuk `korporat`; mekanisme kode pemulihan wajib lewat `rs-wa-enroll` |
| Threat model | KOR-01, KOR-02; ancaman baru | Mitigasi KOR-01/KOR-02 diperbarui. Ancaman baru: kompromi `otp-relay`, kode tetap aktif di production, penyalahgunaan biaya pesan, agen WA mati atau diblokir, pengambilalihan nomor WhatsApp |
| `docs/05-asvs-checklist.md` | V6 | Tabel di atas |
| `docs/05-stack.md` | §6 (verify), §7 (Keycloak), §5.3 (deploy) | Langkah Maven, image kustom, `OTP_SENDER` ≠ `static` di production |
| `docs/coding-standards.md` | Bagian baru | Aturan minimum Java provider (tanpa dependensi runtime, larangan log kode atau nomor) |
| `CLAUDE.md` | Modul kritis | Tambah JAR provider Keycloak, `otp-relay`, dan konfigurasi penjaga kode tetap |
| Design guideline dan wireframe K01 | Layar MFA HR | Layar pendaftaran nomor, masukkan kode, kirim ulang, dan kode pemulihan menggantikan layar TOTP |
| Kepatuhan (DPIA/inventaris pemrosesan) | HR | Nomor WhatsApp HR, tujuan, penyimpanan, penyedia |
| `infra/keycloak/README.md` | TD-27 | Kontrol kompensasi diganti pemeriksaan `amr` untuk `korporat` bila butir 2 terverifikasi |
| Phase plan | F1-T85, F1-T87, F1-T88; task baru | Lihat bagian berikut |

## Rencana task Fase 1 (semua T1; reviewer keamanan + code-reviewer) `[KEPUTUSAN-469]`

Nomor sementara `A32-Tn` diberi nomor F1 oleh orkestrator di phase plan.

| Task | Isi | Jam-task | Bergantung |
| --- | --- | --- | --- |
| A32-T1 | Toolchain Java/Maven: cek registry (artefak SPI 26.8.2, JDK, Maven, pustaka test), kerangka modul, langkah build/test di `verify.sh` dan CI, osv-scanner `pom.xml` | 4 | ADR ini diterima |
| A32-T2 | Image Keycloak kustom (multi-tahap, `providers/`, `kc.sh build`, `--optimized`, digest, cosign); compose lokal dan `keycloak-check ephemeral` memakai image ini; opsional tema sebagai JAR | 4 | A32-T1 |
| A32-T3 | `otp-relay` Rust: konfigurasi + penjaga `DEPLOY_ENV` (lapis 1), trait `OtpSender` + adaptor `static`, server HTTP di jaringan khusus, TLS + token pembawa, batas laju, idempotensi, redaksi log, test negatif | 6 | — |
| A32-T4 | Autentikator `rs-wa-otp-form`: panggilan relay, MAC + catatan sesi, perbandingan waktu konstan, batas 5 per kode, jeda kirim ulang, `failureChallenge`, lapis 2 penjaga, templat FTL, test | 6 | A32-T1, A32-T3 |
| A32-T5 | Required action `rs-wa-enroll`: daftar nomor, verifikasi, kode pemulihan wajib, AIA ubah nomor (`getMaxAuthAge` 0), tawaran ganti nomor setelah login dengan kode pemulihan, pemberitahuan (bila mekanisme tersedia) | 5 | A32-T4 |
| A32-T6 | Realm `korporat` sebagai kode: alur browser/reset/direct grant, atribut `rs_wa_phone`, konfigurasi SPI, `CONFIGURE_TOTP` mati hanya di `korporat`, nilai referensi + `oidc-amr-mapper`; skenario `keycloak-check` (test negatif di bawah); regresi realm lain | 6 | A32-T2, A32-T5 |
| A32-T7 | Backend: pemetaan event E-01/E-02/E-04 (detail faktor), metrik dan peringatan relay; pemeriksaan `amr` di authn bila butir 2 terverifikasi | 4 | A32-T6 |
| A32-T8 | Tema dan UX: layar nomor, kode, kirim ulang (hitung mundur), petunjuk 4 digit terakhir, kode pemulihan; DG dan wireframe K01 | 4 | A32-T4, A32-T5 |
| A32-T9 | Konfigurasi demo: `.env.example` tanpa nilai, KV OpenBao staging + policy `otp-relay`, checklist deploy §5.3 dan pemeriksaan skrip production (lapis 3) | 2 | A32-T3 |
| A32-T10 | Adaptor `whatsapp` sesuai kontrak butir 3, diuji terhadap agen tiruan. **Hanya setelah P32-1 dan P32-2 diputuskan** | 3 | A32-T3, P32-1, P32-2 |
| A32-T11 | Dokumen setelah persetujuan (PRD, ADR-011/024/025/027, threat model, ASVS, DPIA, 05-stack, coding standards, CLAUDE.md, phase plan, risk register) | 4 | ADR ini diterima |
| | **Total** | **48** | |

Demo dengan kode tetap butuh A32-T1 sampai A32-T9 (±41 jam-task). A32-T10 dan agen WA (di luar estimasi ini, `[TIDAK DIKETAHUI]`) dibutuhkan sebelum production.

## Test negatif wajib

- Login HR tanpa kode, dengan kode salah, kode kedaluwarsa (> 5 menit), kode bekas, atau kode dari sesi autentikasi atau tab lain: tidak ada kode otorisasi.
- Lima kode salah: tantangan batal dan penghitung brute force realm naik (penguncian 15 menit, E-02). Kode baru tidak mereset penghitung.
- Kirim ulang sebelum 60 detik tidak memanggil relay. Batas per akun, per nomor, dan 3 nomor per 24 jam menghasilkan `429` tanpa kirim.
- Relay:
  - `DEPLOY_ENV=production` + `OTP_SENDER=static` gagal start;
  - `OTP_SENDER` kosong gagal start;
  - kode tetap bukan `^[A-Z0-9]{6}$` gagal start;
  - nilai kode tidak muncul di pesan galat atau log.
- Keycloak dengan `deploy-env=production` dan jawaban relay `sender_mode=static`: login gagal-tertutup.
- Relay tanpa token atau dengan token salah menjawab `401` tanpa kirim. Relay mati: login gagal-tertutup, kode pemulihan tetap dapat dipakai.
- Ubah nomor: tanpa autentikasi ulang ditolak. Nomor lama tetap berlaku sampai nomor baru terverifikasi. E-04 tercatat.
- Atur ulang password tanpa kode WhatsApp atau kode pemulihan: password baru tidak tersimpan.
- Direct grant tidak dapat memberi token HR walaupun klien diaktifkan tanpa sengaja.
- Kode pemulihan bekas ditolak dan dihitung penguncian.
- Nomor lengkap dan kode tidak muncul di log relay, detail event Keycloak, audit, atau security log (pemeriksaan pola).
- Bila pemeriksaan `amr` dipakai: token `korporat` tanpa faktor kedua di `amr` ditolak backend.
- Realm `pasien` dan `admin` tetap meminta TOTP (regresi).

## Butir yang menunggu keputusan user

| No | Pertanyaan | Pilihan | Usulan |
| --- | --- | --- | --- |
| P32-1 | WhatsApp memproses isi pesan di server Meta, termasuk di luar Indonesia hingga 60 menit, sedangkan T4-Q1 "tanpa kecuali" dan CR-007 menghapus WhatsApp karena alasan ini | A: pengecualian terbatas T4-Q1 khusus kode masuk HR (nomor HR + kode, tanpa data kesehatan), dengan tinjauan Petugas PDP (Pasal 56) dan pemberitahuan di teks persetujuan HR. B: tanpa pengecualian, HR production tetap TOTP, dan WhatsApp/kode tetap hanya demo. C: SMS penyedia Indonesia (*restricted*, wajib menawarkan TOTP juga) | A bila Petugas PDP menyetujui; bila tidak, B  **Dijawab** `[USER]` 2026-11-28: "OTP WhatsApp diterapkan di semuanya", diperjelas "all environments" = A (kode masuk HR korporat lewat WhatsApp di semua lingkungan termasuk production; pengecualian T4-Q1/CR-007 khusus kode masuk HR). Kewajiban transfer lintas batas UU PDP Pasal 56 dan teks persetujuan HR tetap harus dipenuhi sebelum production (gerbang butir 10). |
| P32-2 | Pengirim WhatsApp | A: WhatsApp Business Platform resmi (langsung atau lewat mitra; ada biaya per pesan dan kontrak). B: agen yang mengotomatiskan akun WhatsApp biasa (melanggar ketentuan layanan `[SRC-998]`, nomor dapat diblokir) | A  **Dijawab** `[USER]` 2026-11-28: "use 3rd party" (terminal), diperjelas "P32-2 A" (Telegram) = WhatsApp Business Platform resmi lewat mitra (Business Solution Provider terdaftar Meta). Nama mitra, biaya per pesan, dan kontrak `[TIDAK DIKETAHUI]`, dipilih sebelum A32-T10. |
| P32-3 | Kode pemulihan wajib disimpan HR saat login pertama | A: wajib (satu layar tambahan). B: opsional (HR yang kehilangan nomor harus lewat Verifikator, Fase 2) | A **Dijawab** `[USER]` 2026-11-28 (terminal): "P32-3 B. HR can raise the ticket to ruang sehat support team" = kode pemulihan opsional; HR yang kehilangan nomor mengajukan tiket ke tim support Ruang Sehat. Dampak: jalur pemulihan lewat support (verifikasi identitas sebelum reset faktor kedua, rawan rekayasa sosial) harus dirancang; waktunya bergantung P32-7. |
| P32-4 | Kapan HR memasukkan nomor WhatsApp | A: saat login pertama setelah membuat password. B: di formulir `/daftar` dan undangan (pengundang mengetik nomor rekan), tetap diverifikasi saat login pertama | A  **Dijawab** `[USER]` 2026-11-28 (terminal): "B" = nomor WhatsApp diisi di formulir `/daftar` (pendaftar) dan di formulir Undang HR (pengundang mengetik nomor rekan), lalu tetap diverifikasi dengan kode saat login pertama; HR dapat mengoreksi nomor sebelum verifikasi. Dampak: PRD US-1901 dan alur undangan berubah, kontrak API pendaftaran/undangan bertambah field nomor, nomor belum terverifikasi tidak boleh dipakai untuk apa pun selain pengiriman kode verifikasi. |
| P32-5 | Nomor yang diterima | A: hanya nomor Indonesia (+62) di Fase 1. B: semua negara | A |
| P32-6 | "Demo" memakai kode tetap di lingkungan mana | A: lokal dan staging (data sintetis). B: lokal saja | A  Catatan `[USER]` 2026-11-28 (Telegram): "saat demo pakai static" (menegaskan kode tetap di demo). **Dijawab** `[USER]` 2026-11-28 (terminal): "P32-6 A" = kode tetap berlaku di `local` dan `staging` (data sintetis); production selalu WhatsApp. |
| P32-7 | Jalur pemulihan oleh Verifikator (nomor dan kode pemulihan hilang) | A: tetap Fase 2 seperti rencana. B: dimajukan ke Fase 1 | A  **Dijawab** `[USER]` 2026-11-28 (terminal): "P32-7 B" = pemulihan lewat tiket ke tim support dimajukan ke Fase 1 (konsekuensi P32-3 B). Wajib verifikasi identitas sebelum reset nomor (konfirmasi HR aktif lain di perusahaan yang sama atau kontak terdaftar), audit trail, dan notifikasi ke pemilik akun; rincian dan task tambahan disusun di revisi ADR ini. |

## Riwayat revisi

| Tanggal | Revisi | Sumber |
| --- | --- | --- |
| 2026-11-28 | Diusulkan: autentikator Keycloak tipis + `otp-relay` Rust, port pengirim `static`/`whatsapp`, penjaga kode tetap tiga lapis, parameter OTP, nomor sebagai atribut Keycloak yang didaftarkan HR saat login pertama, kode pemulihan wajib, revisi KEPUTUSAN-265, gerbang production T4-Q1. Ruang lingkup hanya `korporat` | `[USER]` CR-041; `[USER]` 2026-11-28 "only korporat realm, others stay TOTP"; KEPUTUSAN-460–469 |
