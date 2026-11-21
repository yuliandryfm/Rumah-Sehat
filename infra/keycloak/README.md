# Keycloak sebagai kode (F1-T31)

Modul kritis T1 (`CLAUDE.md`, ADR-011). Perubahan manual lewat konsol admin dilarang; semua perubahan lewat berkas di sini (`docs/coding-standards.md` §6, KEPUTUSAN-85).

| Berkas | Isi |
| --- | --- |
| `keycloak.conf` | Konfigurasi server: parameter Argon2 19456 KB / 2 iterasi / paralelisme 1 `[SRC-526]` `[SRC-527]` |
| `realms/pasien.json`, `realms/admin.json` | Realm `pasien` dan `admin` untuk keycloak-config-cli 6.5.1 `[SRC-520]` |
| `test/keycloak_check.py`, `test/kc_http.py` | Test konfigurasi (Admin REST) dan perilaku (OIDC, halaman login), pustaka standar Python 3 |
| `themes/ruang-sehat/login/` | Tema login Ruang Sehat (F1-T35): lihat bagian "Tema login" |
| `themes/ruang-sehat/email/` | Tema email Ruang Sehat (TD-27): hanya pesan email pemberitahuan password diubah; lihat bagian "Atur ulang password: sesi dan pemberitahuan" |
| `test/SmtpSink.java` | Penangkap SMTP khusus test tanpa TLS, dipakai mode `compose` (bukan relay ADR-026) |
| `test/SesTiruan.java` | Tiruan SES API v2 HTTPS khusus test (mode `ephemeral`): memvalidasi SigV4, menyimpan MIME yang diterima, jawaban galat dapat diatur (F1-T36) |

Menerapkan dan menguji:

```sh
docker compose -f infra/local/compose.yaml up -d --wait keycloak
docker compose -f infra/local/compose.yaml --profile alat run --rm keycloak-config   # terapkan saja
bash scripts/keycloak-check.sh compose     # terapkan + test ke Keycloak compose (gagal bila tidak sehat)
bash scripts/keycloak-check.sh ephemeral   # Keycloak sekali pakai, untuk CI/verify
```

Mode `ephemeral` menguji jalur email ADR-026 end-to-end tanpa AWS: Keycloak (realm apa adanya, STARTTLS ke `email-relay:2525`) → `rs-backend email-relay` (dibangun `cargo build`, sertifikat dari CA uji yang dibangkitkan `openssl` saat test) → `SesTiruan.java` (HTTPS, memvalidasi SigV4). Kunci akses contoh fiktif AWS dibaca relay dari OpenBao dev sekali pakai. Semua kontainer bantu berbagi namespace jaringan Keycloak (hanya 127.0.0.1). Mode `compose` tetap memakai penangkap SMTP tanpa TLS lewat fixture realm sementara, karena Keycloak compose tidak memuat CA uji.

## Isi kedua realm

Sama di `pasien` dan `admin`, kecuali batas sesi dan klien:

- **Pendaftaran bawaan mati** (`registrationAllowed: false`), email sebagai username, profil user hanya `username` dan `email` yang hanya dapat diubah admin (lewat backend), required action Update Email/Update Profile mati (KEPUTUSAN-201, ADR-011 butir 6).
- **Brute force sementara**: 5 gagal, kunci 900 detik, strategi linear, Max Wait 900 detik, Failure Reset Time 12 jam, Quick Login Check bawaan (KEPUTUSAN-190) `[SRC-521]` `[SRC-525]`. Hitungan gagal kembali nol hanya setelah login berhasil atau Failure Reset Time, sehingga satu gagal setelah kunci berakhir langsung mengunci lagi (US-103 AC3 revisi, KEPUTUSAN-285; test `cek_pasca_kunci`).
- **Token**: Revoke Refresh Token aktif dengan `refreshTokenMaxReuse` 0, token akses 300 detik. Sesi `pasien` 30 hari/90 hari, `admin` 30 menit/12 jam (ADR-011 butir 7).
- **MFA wajib**: alur `rs-browser-mfa` mengikuti pola dokumentasi "Conditional 2FA sub-flow with OTP default" `[SRC-523]`. User yang punya OTP atau kode pemulihan wajib memasukkan salah satunya; user tanpa faktor kedua dipaksa Configure OTP sebelum kode otorisasi terbit `[SRC-524]`. Configure OTP memakai `add-recovery-codes`, sehingga kode pemulihan wajib dibuat setelah TOTP dan tersimpan sebagai required action user (KEPUTUSAN-203) `[SRC-522]`. Kebijakan OTP TOTP SHA1 6 digit 30 detik, kode tidak dapat dipakai ulang (KEPUTUSAN-85).
- **Atur ulang password tidak melewati MFA**: alur `rs-reset-credentials-mfa` meminta OTP atau kode pemulihan setelah tautan email dibuka dan sebelum Reset Password. Langkah bawaan `reset-otp` (yang mengatur ulang OTP) tidak dipakai (KEPUTUSAN-199, US-103 AC4e) `[SRC-521]`. Tautan berlaku 60 menit. Eksekusi `reset-credential-email` memakai `force-login=true` (`rs-reset-email-force-login`), sehingga setelah password baru tersimpan pengguna tidak otomatis masuk dan harus masuk lewat alur biasa (US-103 AC4d, F1-T74) `[SRC-680]`. OTP salah di alur ini dihitung penguncian `[SRC-682]`.
- **Verifikasi email lebih dulu**: required action Verify Email berprioritas 5, sebelum Configure OTP (10), sehingga TOTP tidak dapat didaftarkan pada akun yang emailnya belum terverifikasi (review keamanan F1-T31, cegah pre-account hijack).
- **Konsol akun Keycloak tertutup**: klien `account-console` nonaktif dan default role tanpa `account/manage-account`, supaya ubah password/TOTP tidak lewat konsol akun di luar alur aplikasi.
- **Direct grant**: tidak ada klien dengan grant password (termasuk `admin-cli` realm), dan alur `rs-direct-grant-mfa` tetap mewajibkan OTP bila kelak diaktifkan tanpa sengaja.
- **Pesan login seragam** (US-103 AC2): realm override pesan `invalidUserMessage`, `accountDisabledMessage`, `accountTemporarilyDisabledMessage`, `accountPermanentlyDisabledMessage` ke satu teks `[DOK: wireframes/k01-korporat-hr.html]`. Tema bawaan menampilkan teks itu di posisi berbeda untuk akun nonaktif (banner) dibanding password salah (galat isian); struktur halaman disamakan oleh tema `ruang-sehat` (F1-T35, bagian "Tema login").
- **SMTP** ke relay internal ADR-026: STARTTLS wajib, tanpa AUTH, tanpa `debug`, host/port/pengirim dari variabel lingkungan (KEPUTUSAN-231). Relay: `rs-backend email-relay` (F1-T36, crate `crates/email-relay` + adaptor `crates/ses`). Sertifikat relay dari CA internal dipercaya Keycloak lewat `conf/truststores`/`KC_TRUSTSTORE_PATHS` dengan `tls-hostname-verifier` `DEFAULT` `[SRC-583]`.
- **Event** login dan admin disimpan 7 hari, tanpa detail representasi admin (ADR-011 butir 10). Event login diambil `rs-backend worker` lewat Admin REST dengan klien `rs-backend-event-reader` (F1-T34). Event menyimpan `details.username` (= email) dan ID token; backend tidak menyalinnya ke audit (`apps/rs-backend/src/modules/identity/domain.rs`).

Klien:

| Realm | Klien | Jenis | Catatan |
| --- | --- | --- | --- |
| `pasien` | `rs-pasien-mobile` | Publik | Authorization Code + PKCE S256 wajib, redirect URI tepat satu dari `RS_KC_PASIEN_MOBILE_REDIRECT_URI` (ADR-011 butir 7, RFC 8252) |
| `admin` | `rs-backoffice-bff`, `rs-portal-bff` | Rahasia | Authorization Code + PKCE S256, secret dari `RS_KC_*_BFF_SECRET` (lokal: `infra/local/.env`; staging/production: OpenBao lewat skrip deploy) |
| `pasien`, `admin` | `rs-backend-event-reader` | Rahasia, service account | Pengambil event `rs-backend worker` (F1-T34, ADR-011 butir 10): hanya client credentials, `fullScopeAllowed` mati, peran service account dan scope mapping hanya `realm-management/view-events`, cakupan bawaan hanya `kc-admin-api` (audience Admin REST yang diwajibkan Keycloak 26.8.2). Secret dari `RS_KC_EVENT_READER_<REALM>_SECRET` (satu per realm) |

Audience API backend dan pemeta klaim belum diatur; ditetapkan bersama validasi token F1-T32.

Impor keycloak-config-cli dijalankan tanpa cache, tetapi hanya menimpa atribut yang ada di JSON: klien atau IdP tambahan buatan manual tidak terhapus. `scripts/keycloak-check.sh` GAGAL bila ada klien di luar konfigurasi (ditambah klien bawaan Keycloak) atau IdP apa pun.

## Tema login (F1-T35)

Tema `ruang-sehat` (jenis `login`, parent `keycloak.v2`, SRC-630, SRC-631), dipakai realm `pasien` dan `admin` (`loginTheme`), satu bahasa Indonesia (`supportedLocales: ["id"]`, tanpa pemilih bahasa). Mencakup login, Lupa password, OTP dan kode pemulihan, pendaftaran TOTP, verifikasi email, dan ubah password (aktivasi undangan) lewat templat bawaan yang distel oleh `messages_id.properties` dan CSS.

- `theme.properties`: `parent=keycloak.v2`, `locales=id`, `darkMode=false`, `styles` = `css/styles.css css/tokens.css css/ruang-sehat.css`.
- `resources/css/tokens.css`: salinan persis `web/packages/ui/src/tokens.css`; test membandingkan isi yang disajikan Keycloak dengan berkas sumber. Perbarui salinan ini bila token berubah.
- `login.ftl`: satu-satunya templat yang ditimpa. Galat login digambar sebagai satu blok `#kc-login-error` (`role="alert"`) untuk semua kegagalan (password salah, akun tidak ada, terkunci, nonaktif), tanpa galat per isian, sehingga DOM identik (UP-73, TD-16). Pesan dari kunci tema `rsLoginGagal`; realm override pesan tetap dipasang sebagai lapis kedua.
- Tanpa skrip, font, atau aset pihak ketiga: semua sumber daya dari Keycloak sendiri; font sistem. CSP Keycloak tidak diubah (`frame-src 'self'; frame-ancestors 'self'; object-src 'none';`, diuji).
- Dipasang dengan mount `infra/keycloak/themes/ruang-sehat` ke `/opt/keycloak/themes/ruang-sehat` (compose dan `scripts/keycloak-check.sh ephemeral`).

## MFA pasien dan tema pendaftaran TOTP (F1-T72, F1-T73)

- Konfigurasi MFA realm `pasien` sama dengan `admin` (bagian "Isi kedua realm"); `keycloak_check.py` menguji untuk keduanya: login pasien tanpa OTP tidak menerbitkan kode otorisasi, OTP salah dan kode pemulihan salah dihitung penguncian (5 gagal mengunci, E-02 `user_temporarily_disabled` tercatat), kode pemulihan bekas ditolak, meninggalkan pendaftaran TOTP atau kode pemulihan tidak memberi akses.
- Tema menimpa `login-otp.ftl` dan `login-recovery-authn-code-input.ftl` untuk petunjuk menghubungi Layanan Pelanggan (US-103 AC8; tanpa nomor atau alamat, kontak final `[TIDAK DIKETAHUI]`), serta `login-recovery-authn-code-config.ftl` untuk teks pembaca layar dan format tanggal berbahasa Indonesia `[SRC-681]`. Kode QR, kunci teks (mode manual), dan 12 kode pemulihan yang ditampilkan sekali memakai templat bawaan; hasilnya diuji.
- Klaim `acr` token akses bernilai `1` untuk login password+OTP dan tanpa `amr` `[SRC-682]`: backend tidak dapat memastikan OTP dari token. Pemetaan Level of Assurance (acr ke LoA) belum dikonfigurasi; kontrol kompensasinya di bagian "Kontrol kompensasi `acr`=1 tanpa `amr`" di bawah.

## Atur ulang password: sesi dan pemberitahuan (TD-27, US-103 AC4d)

Rantai setelah password baru tersimpan lewat tautan atur ulang (realm `pasien` dan `admin`):

1. **Sesi Keycloak berakhir seketika.** Bawaan `keycloak.v2` menampilkan kotak "keluar dari perangkat lain" yang **tidak** tercentang, sehingga atur ulang tidak mengakhiri sesi lain kecuali pengguna mencentangnya. Tema menimpa `login-update-password.ftl`: bila bukan aksi dari aplikasi (atur ulang, aktivasi undangan), formulir selalu mengirim `logout-sessions=on` sebagai isian tersembunyi, tanpa pilihan, dengan kalimat pemberitahuan; bila aksi dari aplikasi (`kc_action`, US-103 AC6 kelak), kotak tampil dan tercentang bawaan (KEPUTUSAN-198 butir 2). `UpdatePassword` 26.8.2 lalu mengakhiri semua sesi lain pengguna, online dan offline (fitur `logout-all-sessions:v1` tidak aktif), dan memancarkan satu event `LOGOUT` per sesi dengan `sessionId` dan detail `logout_triggered_by_required_action=UPDATE_PASSWORD` `[SRC-770]`. Token penyegar sesi lama langsung ditolak (`invalid_grant`, diuji).
2. **Token akses sesi lama ditolak backend paling lambat satu putaran pengambil event.** `rs-backend worker` (F1-T34, tiap 60 detik) mencatat `session_revocation` (alasan `logout`) untuk setiap event `LOGOUT` sukses bersesi milik akun yang dikenal, di transaksi yang sama dengan E-03, lalu menyalinnya ke Valkey (`WORKER_VALKEY_URL`). Berlaku sampai waktu event + 360 detik (umur token akses 300 detik + kelonggaran jam). Valkey mati: baris PostgreSQL tetap dibaca API lewat jalur cadangan (RT-18); salinan dibangun ulang setiap kali worker memperoleh kunci pengambil (`apps/rs-backend/src/modules/identity/jobs.rs`, test `apps/rs-backend/tests/identity_password_reset_sessions.rs`).
3. **E-04** tercatat dari `UPDATE_CREDENTIAL` (`credential_type=password`, aksi `password_changed`) tanpa nilai kredensial; salinan usang `UPDATE_PASSWORD` tidak diambil supaya tidak tercatat ganda (`domain.rs`). Event `RESET_PASSWORD` sukses tidak dipancarkan saat password baru tersimpan lewat alur ini (diamati `scripts/keycloak-check.sh`, `[SRC-772]`), sehingga aksi audit tidak membedakan atur ulang dari ubah password; keduanya E-04.
4. **Email pemberitahuan** lewat listener event bawaan `email` (realm `eventsListeners: ["jboss-logging", "email"]`, `emailTheme: ruang-sehat`), dikirim Keycloak lewat SMTP realm ke relay ADR-026. `keycloak.conf` membatasi listener ke event `UPDATE_PASSWORD` saja (`spi-events-listener--email--include-events`); tanpa itu listener juga mengirim email untuk `LOGIN_ERROR` dan perubahan OTP `[SRC-771]`. Email dikirim setelah transaksi Keycloak selesai, hanya bila email akun terverifikasi; kegagalan kirim hanya tercatat di log server Keycloak (tanpa coba ulang, sama dengan email Keycloak lain, ADR-026). Teks di `themes/ruang-sehat/email/messages/messages_id.properties` bertanda DRAF (belum ditetapkan PRD): tanpa waktu (zona JVM bukan WIB), tanpa alamat IP, tanpa tautan; kontak Layanan Pelanggan final `[TIDAK DIKETAHUI]`. Email yang sama juga terkirim saat aktivasi undangan admin dan, kelak, ubah password (AC6), karena ketiganya memakai required action yang sama; teksnya netral ("diatur ulang atau diubah").

Batas yang diketahui:
- **Logout Admin REST (KEPUTUSAN-198 butir 1) tidak dipakai.** Realm `pasien` belum memiliki klien berhak `manage-users` di cabang ini, dan klien `rs-backend-admin-manager` realm `admin` masih bergerbang TD-26; menambah klien `manage-users` baru dilarang (TD-26/TD-30). Pengakhiran sesi Keycloak memakai mekanisme bawaan `UpdatePassword` (butir 1). Isian tersembunyi itu dapat dihapus oleh pengguna yang sedang mengatur ulang (sudah lolos email dan OTP) dengan memanipulasi permintaan; akibatnya hanya sesi miliknya sendiri yang tetap hidup.
- Atur ulang lewat Admin REST `reset-password` (bukan alur email) tidak mengakhiri sesi dan tidak dipakai backend saat ini.
- Batas 2 menit bergantung pada pengambil event sehat; keterlambatan lebih dari 5 menit memicu `idp_event_pull_delayed` (ADR-011 butir 10).

## Kontrol kompensasi `acr`=1 tanpa `amr` (TD-27)

Token akses realm `pasien` dan `admin` memuat `acr`=`1` setelah password+OTP dan tidak memuat `amr` `[SRC-682]`, sehingga backend tidak dapat membuktikan dari token bahwa OTP sudah dimasukkan. Backend tidak membaca `acr`/`amr`; jaminan MFA berasal dari konfigurasi realm yang diuji `scripts/keycloak-check.sh` dan tidak dapat diubah di luar berkas ini:

- Token untuk pengguna hanya terbit lewat alur browser `rs-browser-mfa` (Authorization Code + PKCE S256), yang mewajibkan OTP atau kode pemulihan, dan memaksa Configure OTP bagi pengguna tanpa faktor kedua sebelum kode otorisasi terbit.
- Direct grant (grant password) mati di semua klien termasuk `admin-cli`; alur `rs-direct-grant-mfa` tetap mewajibkan OTP bila kelak diaktifkan tanpa sengaja. Implicit, device, dan CIBA mati di semua klien buatan proyek.
- Atur ulang password (`rs-reset-credentials-mfa`) meminta OTP sebelum password baru dan tidak menerbitkan token (`force-login`).
- Tanpa IdP eksternal, tanpa pendaftaran bawaan, konsol akun mati; klien layanan (`rs-backend-*`) hanya client credentials tanpa pengguna manusia.
- Deteksi drift: `keycloak-check.sh` GAGAL bila ada klien atau IdP di luar konfigurasi, atau alur/klien di atas berubah.

Sisa risiko: perubahan manual di konsol admin di antara dua pemeriksaan drift. Pemetaan LoA (`acr` 2 untuk OTP) dapat menggantikan kontrol ini bila reviewer keamanan memintanya.

## Belum diketahui dan temuan terbuka

- Pemaketan tema untuk production (JAR dengan `META-INF/keycloak-themes.json` di `providers/`, SRC-630) dan nilai cache tema di luar `start-dev`: ditetapkan bersama skrip deploy `[TIDAK DIKETAHUI]`; lokal dan CI memasang direktori tema ke `/opt/keycloak/themes/ruang-sehat`.
- Tema belum diuji dengan peramban sungguhan (kontras, reflow 320 px, pembaca layar): hanya DOM, header, dan aset yang diuji otomatis; uji manual `[TIDAK DIKETAHUI]`.

- Daftar kata sandi umum/bocor dan lisensinya untuk kebijakan "Password blacklist" (KEPUTUSAN-188) `[TIDAK DIKETAHUI]`; kebijakan `passwordBlacklist` belum dipasang.
- Dukungan resmi keycloak-config-cli 6.5.1 (build klien 26.5.5) untuk Keycloak 26.8.2 `[TIDAK DIKETAHUI]`; kecocokan dibuktikan `scripts/keycloak-check.sh`.
- `sslRequired` tetap `external` (sama dengan bawaan) agar lokal dapat memakai HTTP; nilai untuk production di belakang Caddy ditetapkan bersama skrip deploy.
