# Keycloak sebagai kode (F1-T31)

Modul kritis T1 (`CLAUDE.md`, ADR-011). Perubahan manual lewat konsol admin dilarang; semua perubahan lewat berkas di sini (`docs/coding-standards.md` §6, KEPUTUSAN-85).

| Berkas | Isi |
| --- | --- |
| `keycloak.conf` | Konfigurasi server: parameter Argon2 19456 KB / 2 iterasi / paralelisme 1 `[SRC-526]` `[SRC-527]` |
| `realms/pasien.json`, `realms/admin.json` | Realm `pasien` dan `admin` untuk keycloak-config-cli 6.5.1 `[SRC-520]` |
| `test/keycloak_check.py`, `test/kc_http.py` | Test konfigurasi (Admin REST) dan perilaku (OIDC, halaman login), pustaka standar Python 3 |
| `test/SmtpSink.java` | Penangkap SMTP khusus test (bukan relay ADR-026) |

Menerapkan dan menguji:

```sh
docker compose -f infra/local/compose.yaml up -d --wait keycloak
docker compose -f infra/local/compose.yaml --profile alat run --rm keycloak-config   # terapkan saja
bash scripts/keycloak-check.sh compose     # terapkan + test ke Keycloak compose (gagal bila tidak sehat)
bash scripts/keycloak-check.sh ephemeral   # Keycloak sekali pakai, untuk CI/verify
```

## Isi kedua realm

Sama di `pasien` dan `admin`, kecuali batas sesi dan klien:

- **Pendaftaran bawaan mati** (`registrationAllowed: false`), email sebagai username, profil user hanya `username` dan `email` yang hanya dapat diubah admin (lewat backend), required action Update Email/Update Profile mati (KEPUTUSAN-201, ADR-011 butir 6).
- **Brute force sementara**: 5 gagal, kunci 900 detik, strategi linear, Max Wait 900 detik, Failure Reset Time 12 jam, Quick Login Check bawaan (KEPUTUSAN-190) `[SRC-521]` `[SRC-525]`. Hitungan gagal kembali nol hanya setelah login berhasil atau Failure Reset Time, sehingga satu gagal setelah kunci berakhir langsung mengunci lagi (US-103 AC3 revisi, KEPUTUSAN-285; test `cek_pasca_kunci`).
- **Token**: Revoke Refresh Token aktif dengan `refreshTokenMaxReuse` 0, token akses 300 detik. Sesi `pasien` 30 hari/90 hari, `admin` 30 menit/12 jam (ADR-011 butir 7).
- **MFA wajib**: alur `rs-browser-mfa` mengikuti pola dokumentasi "Conditional 2FA sub-flow with OTP default" `[SRC-523]`. User yang punya OTP atau kode pemulihan wajib memasukkan salah satunya; user tanpa faktor kedua dipaksa Configure OTP sebelum kode otorisasi terbit `[SRC-524]`. Configure OTP memakai `add-recovery-codes`, sehingga kode pemulihan wajib dibuat setelah TOTP dan tersimpan sebagai required action user (KEPUTUSAN-203) `[SRC-522]`. Kebijakan OTP TOTP SHA1 6 digit 30 detik, kode tidak dapat dipakai ulang (KEPUTUSAN-85).
- **Atur ulang password tidak melewati MFA**: alur `rs-reset-credentials-mfa` meminta OTP atau kode pemulihan setelah tautan email dibuka dan sebelum Reset Password. Langkah bawaan `reset-otp` (yang mengatur ulang OTP) tidak dipakai (KEPUTUSAN-199, US-103 AC4e) `[SRC-521]`. Tautan berlaku 60 menit.
- **Verifikasi email lebih dulu**: required action Verify Email berprioritas 5, sebelum Configure OTP (10), sehingga TOTP tidak dapat didaftarkan pada akun yang emailnya belum terverifikasi (review keamanan F1-T31, cegah pre-account hijack).
- **Konsol akun Keycloak tertutup**: klien `account-console` nonaktif dan default role tanpa `account/manage-account`, supaya ubah password/TOTP tidak lewat konsol akun di luar alur aplikasi.
- **Direct grant**: tidak ada klien dengan grant password (termasuk `admin-cli` realm), dan alur `rs-direct-grant-mfa` tetap mewajibkan OTP bila kelak diaktifkan tanpa sengaja.
- **Pesan login seragam** (US-103 AC2): realm override pesan `invalidUserMessage`, `accountDisabledMessage`, `accountTemporarilyDisabledMessage`, `accountPermanentlyDisabledMessage`, `invalidEmailMessage` ke satu teks `[DOK: wireframes/k01-korporat-hr.html]`. Tema bawaan menampilkan teks itu di posisi berbeda untuk akun nonaktif (banner) dibanding password salah (galat isian); penyamaan struktur halaman menjadi bagian tema (F1-T35).
- **SMTP** ke relay internal ADR-026: STARTTLS wajib, tanpa AUTH, tanpa `debug`, host/port/pengirim dari variabel lingkungan (KEPUTUSAN-231). Sampai relay F1-T36 ada, pengiriman email gagal (aman).
- **Event** login dan admin disimpan 7 hari, tanpa detail representasi admin (ADR-011 butir 10). Event login diambil `rs-backend worker` lewat Admin REST dengan klien `rs-backend-event-reader` (F1-T34). Event menyimpan `details.username` (= email) dan ID token; backend tidak menyalinnya ke audit (`apps/rs-backend/src/modules/identity/domain.rs`).

Klien:

| Realm | Klien | Jenis | Catatan |
| --- | --- | --- | --- |
| `pasien` | `rs-pasien-mobile` | Publik | Authorization Code + PKCE S256 wajib, redirect URI tepat satu dari `RS_KC_PASIEN_MOBILE_REDIRECT_URI` (ADR-011 butir 7, RFC 8252) |
| `admin` | `rs-backoffice-bff`, `rs-portal-bff` | Rahasia | Authorization Code + PKCE S256, secret dari `RS_KC_*_BFF_SECRET` (lokal: `infra/local/.env`; staging/production: OpenBao lewat skrip deploy) |
| `pasien`, `admin` | `rs-backend-event-reader` | Rahasia, service account | Pengambil event `rs-backend worker` (F1-T34, ADR-011 butir 10): hanya client credentials, `fullScopeAllowed` mati, peran service account dan scope mapping hanya `realm-management/view-events`, cakupan bawaan hanya `kc-admin-api` (audience Admin REST yang diwajibkan Keycloak 26.8.2). Secret dari `RS_KC_EVENT_READER_<REALM>_SECRET` (satu per realm) |

Audience API backend dan pemeta klaim belum diatur; ditetapkan bersama validasi token F1-T32.

Impor keycloak-config-cli dijalankan tanpa cache, tetapi hanya menimpa atribut yang ada di JSON: klien atau IdP tambahan buatan manual tidak terhapus. `scripts/keycloak-check.sh` GAGAL bila ada klien di luar konfigurasi (ditambah klien bawaan Keycloak) atau IdP apa pun.

## Belum diketahui dan temuan terbuka

- Struktur halaman akun nonaktif (banner) berbeda dari password salah (galat isian) walau teksnya identik: tema F1-T35.
- Alur atur ulang password langsung menerbitkan kode otorisasi setelah password baru, berbeda dengan US-103 AC4d ("tidak otomatis masuk"): F1-T74.

- Daftar kata sandi umum/bocor dan lisensinya untuk kebijakan "Password blacklist" (KEPUTUSAN-188) `[TIDAK DIKETAHUI]`; kebijakan `passwordBlacklist` belum dipasang.
- Dukungan resmi keycloak-config-cli 6.5.1 (build klien 26.5.5) untuk Keycloak 26.8.2 `[TIDAK DIKETAHUI]`; kecocokan dibuktikan `scripts/keycloak-check.sh`.
- `sslRequired` tetap `external` (sama dengan bawaan) agar lokal dapat memakai HTTP; nilai untuk production di belakang Caddy ditetapkan bersama skrip deploy.
