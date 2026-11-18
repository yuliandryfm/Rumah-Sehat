# ADR-024: Stack aplikasi, yaitu backend Rust, web Next.js dengan server Next.js sebagai BFF, mobile Flutter, dan satu monorepo

Status: Diusulkan (menunggu gate Tahap 5)
Tanggal: 2026-11-18

## Konteks

- Flutter wajib untuk aplikasi mobile pasien dan tenaga kesehatan. Teknologi backend, portal web, dan cloud tidak dibatasi selain lokasi data di Indonesia, dan dipilih di Tahap 5 dengan persetujuan user `[DOK: 00-project-brief.md]` (`[USER] P7.4, P5.5`).
- Tim terdiri atas project manager, Claude lewat runbook, dan 1 mobile developer. Backend dan web ditulis Claude. Go-live 1 Maret 2027 dengan toleransi waktu 0 `[DOK: 00-project-brief.md]` `[DOK: 01-business-case.md]` (R-02, R-04).
- Mobile developer dibayar penuh waktu sampai go-live saja. Siapa yang memelihara aplikasi setelah go-live belum diketahui (PT-35) `[USER] T5-Q7`.
- Arsitektur yang sudah diterima menuntut dari backend: modular monolith dengan proses `api`, `realtime`, dan `worker`, serta RME minimalis sebagai sistem terpisah yang boleh memakai pustaka bersama lewat monorepo tanpa ketergantungan runtime (ADR-006); SQL eksplisit PostgreSQL, antrean dan outbox di tabel PostgreSQL, dan penguncian job tunggal yang mekanismenya ditetapkan di Tahap 5 (ADR-007); pustaka audit tunggal (ADR-009); validasi token Keycloak menurut RFC 8725 (ADR-011); enkripsi amplop (envelope encryption) di tingkat aplikasi (ADR-013); WebSocket (ADR-016); serta OpenAPI dan envelope `response_code` (ADR-022, ADR-023) `[DOK: adr/]`.
- ADR-011 butir 7 menetapkan pola *backend-for-frontend* (BFF) untuk ketiga web: "Backend menjalankan Authorization Code sebagai klien rahasia, sehingga token tidak pernah sampai ke browser", dengan cookie sesi `HttpOnly`, `Secure`, `SameSite`, perlindungan CSRF, data sesi di Redis, dan token yang dienkripsi dengan kunci dari secret manager serta diikat ke ID sesi `[DOK: adr/ADR-011-target-asvs-autentikasi-sesi-mfa.md]`.
- Pada T5-Q1, penyusun menawarkan empat opsi: (1) TypeScript/Node.js + NestJS dengan web React (rekomendasi penyusun); (2) Go + React; (3) Kotlin/Spring Boot + React; (4) Dart + Flutter Web. User mengusulkan Rust untuk backend, Next.js untuk web, dan Flutter untuk mobile. Penyusun menjelaskan kelebihan Rust (aman memori, kebutuhan memori kecil, compiler ketat) dan kekurangannya (pengembangan lebih lambat, pemelihara harus menguasai Rust), konsekuensi server Next.js sebagai BFF, serta riwayat advisory keamanan Next.js. Opsi kemudian diulang: (1) TypeScript NestJS + Next.js (rekomendasi); (2) Rust + Next.js; (3) Rust + React SPA; (4) TypeScript NestJS + React SPA. User bertanya arti "lebih lambat dikembangkan". Penyusun menjelaskan tiga sebabnya (waktu compile di setiap siklus verifikasi dan CI, compiler ketat yang menuntut lebih banyak putaran perbaikan, lebih banyak komponen yang mungkin ditulis sendiri). Besaran perlambatannya `[TIDAK DIKETAHUI]` dan akan diukur di reference implementation Tahap 8 Fase 1. User menjawab "Rust + Next.js" `[USER] T5-Q1` `[DOK: 05-catatan-stack.md]`.
- Halaman advisory resmi Next.js memuat dua advisory Critical, yaitu RCE di Image Optimization API dengan berkas SVG (14 Okt 2026) dan RCE di `next/og` ImageResponse (22 Sep 2026), serta tujuh advisory Moderate bertanggal 3 Nov 2026. Di antara yang Moderate ada "HttpOnly cookie exposure through internal response header on Next.js Edge runtime routes" dan "Cross-client response injection and disclosure in self-hosted Next.js applications forwarding fetched responses" `[SRC-246]`.

## Keputusan

**1. Backend Rumah Sehat dan backend RME minimalis memakai Rust** `[USER] T5-Q1` `[KEPUTUSAN-74]`:
- Rust stable 1.100.0 edisi 2024, dipin di `rust-toolchain.toml` `[SRC-161]`. Upgrade toolchain (siklus 6 minggu) dievaluasi, tidak otomatis.
- HTTP dan async: tokio 1.54.1 `[SRC-162]`, axum 0.8.9 `[SRC-163]`, tower-http 0.7.1 `[SRC-164]`, serde `[SRC-167]`, reqwest 0.13.5 untuk panggilan keluar `[SRC-170]`.
- Database: sqlx 0.9.1 untuk PostgreSQL, dengan query yang diperiksa saat compile. Metadata offline `.sqlx` di-commit, dan `cargo sqlx prepare --check` dijalankan di CI `[SRC-165]` `[SRC-166]`. Migrasi berupa SQL murni lewat sqlx-cli 0.9.1 `[SRC-182]`, hanya maju (forward-only), dengan pola *expand/contract* (ADR-019 butir 2).
- OpenAPI 3.1 dihasilkan utoipa 6.0.0 dari kode `[SRC-168]` (ADR-022).
- Validasi JWT: jsonwebtoken 12.0.0 `[SRC-169]`, dengan daftar izin algoritma, issuer per realm, dan audience sesuai RFC 8725 `[SRC-79]` (ADR-011 butir 8). JWKS realm diambil dari Keycloak dan di-cache.
- Kriptografi: aws-lc-rs 1.19.0 sebagai satu-satunya pustaka kripto untuk AES-256-GCM, HMAC-SHA256 (blind index, tanda tangan webhook), SHA-256 (hash berantai audit), dan pembangkit bilangan acak `[SRC-173]`.
- UUIDv7 dibuat aplikasi dengan uuid 1.29.3 fitur `std` dan `v7` (`Uuid::now_v7`) `[SRC-176]` `[SRC-238]`, bukan nilai bawaan kolom (ADR-002).
- Waktu: crate `time` 0.3.55 `[SRC-177]`. Log: tracing 0.1.44 dan tracing-subscriber 0.3.23 dengan format JSON `[SRC-174]` `[SRC-175]` `[SRC-238]`.
- Klien Valkey: crate `redis` 1.8.2 versi stabil, bukan 2.0.0-rc `[SRC-171]`.
- **Antrean job** (ADR-007) dibangun tipis sendiri di modul Platform: tabel job dengan `SELECT ... FOR UPDATE SKIP LOCKED` `[SRC-36]`. Job tunggal (relay audit, pengambil event Keycloak) memakai `pg_try_advisory_lock` tingkat sesi, yang langsung mengembalikan `false` bila kunci sedang dipegang dan dilepas saat sesi berakhir `[SRC-234]`. Ini menjawab "mekanisme penguncinya ditetapkan di Tahap 5" pada konsekuensi ADR-007.
- **Workspace Cargo**: `rs-backend` adalah satu binary dengan subcommand `api`, `realtime`, `worker`, dan `audit-relay`; `rme-backend` adalah satu binary dengan subcommand `api` dan `worker`. Pustaka bersama (`audit`, `crypto`, `authz`, `envelope`, `platform-jobs`, `integration-contract`) dipakai RME lewat workspace, tanpa ketergantungan runtime (ADR-006 butir 2).
- **Batas modul** (konsekuensi ADR-006): setiap modul bisnis Rumah Sehat adalah modul Rust yang hanya mengekspos antarmuka publiknya. Tabel milik modul hanya diakses dari repository modul itu. Aturan ini ditegakkan dengan visibilitas Rust (`pub(crate)`), checklist code review, dan test arsitektur di `scripts/verify.sh` yang memeriksa nama tabel di SQL per modul.
- Test dan rantai pasok: `cargo test` dengan testcontainers 0.28.0 untuk PostgreSQL 18, Valkey, dan Keycloak `[SRC-181]`; `cargo fmt --check`; `cargo clippy -- -D warnings`; cargo-deny 0.21.0 (lisensi, advisory, sumber) dan cargo-audit 0.23.0 `[SRC-183]` `[SRC-184]`.

**2. Tiga web (backoffice, portal rekanan, web RME) memakai Next.js** `[USER] T5-Q1` `[KEPUTUSAN-75]`:
- Next.js 16.4.6 App Router `[SRC-156]`, React 19.3.1 `[SRC-160]`, TypeScript 7.0.2 `[SRC-112]`, pnpm 12.21.0 `[SRC-113]`, dan runtime Node.js 26 LTS (Active LTS, EOL 2029-04-30) `[SRC-107]`.
- Tiga aplikasi terpisah, masing-masing dibangun dengan `output: 'standalone'` `[SRC-157]` dan dijalankan di kontainer sendiri secara self-hosted `[SRC-158]` di server Indonesia (ADR-025), tidak di Vercel. Pemisahan ini mengikuti ADR-006 butir 2 dan 5.
- **Pengerasan karena riwayat advisory** `[SRC-246]`: image optimization dimatikan (`images.unoptimized: true`) `[SRC-159]`. `next/og`, Edge runtime, `use cache`/Cache Components, dan middleware untuk keputusan otorisasi tidak dipakai. Versi dipin. Patch keamanan Next.js berderajat Critical atau High dipasang paling lambat 7 hari setelah terbit `[KEPUTUSAN-75]`.
- Klien API bertipe dibangkitkan dari OpenAPI dengan openapi-typescript 7.13.0 dan openapi-fetch 0.17.0 (masih 0.x, dipantau) `[SRC-131]` `[SRC-132]`. Pengambilan data dengan TanStack Query 5.108.0 `[SRC-130]`. Test end-to-end dengan Playwright 1.65.1 `[SRC-133]`.
- Pustaka UI dan design token ditetapkan di Tahap 6. ESLint dan pustaka test unit web diverifikasi saat ditambahkan di Tahap 8.

**3. BFF dijalankan di server Next.js (tafsiran ADR-011 butir 7)** `[USER] T5-Q1` `[KEPUTUSAN-75]`. Kata "Backend" pada ADR-011 butir 7 ditafsirkan sebagai **lapisan server milik Rumah Sehat yang berdiri di depan browser**, yaitu proses server Next.js setiap web, bukan backend Rust. Semua sifat keamanan butir 7 tetap berlaku:

| Syarat ADR-011 butir 7 | Cara dipenuhi di server Next.js | Sumber |
| --- | --- | --- |
| Authorization Code sebagai klien rahasia | Server Next.js menjalankan OIDC Authorization Code sebagai klien rahasia dengan openid-client 6.9.0. Rahasia klien diambil dari OpenBao (ADR-025) | `[SRC-115]` `[KEPUTUSAN-75]` |
| Token tidak pernah sampai ke browser | Token akses dan token penyegar hanya ada di server Next.js dan di penyimpanan sesi. Browser hanya memegang cookie sesi | `[KEPUTUSAN-75]` |
| Cookie `HttpOnly`, `Secure`, `SameSite`, perlindungan CSRF | Cookie sesi dengan ketiga atribut, ditambah token CSRF pada setiap permintaan yang mengubah data | `[KEPUTUSAN-75]` |
| Data sesi BFF di Redis (ADR-007) | Data sesi disimpan di Valkey, implementasi komponen "Redis" (ADR-025), lewat ioredis 6.0.0. Web RME memakai instans Valkey RME | `[SRC-116]` `[KEPUTUSAN-77]` |
| Token dienkripsi dengan kunci dari secret manager dan diikat ke ID sesi | Token dienkripsi (JWE) dengan jose 6.2.17, memakai kunci yang diambil dari OpenBao, dan diikat ke ID sesi, sehingga pembaca Valkey tidak memperoleh token yang dapat dipakai (M-06) | `[SRC-114]` `[KEPUTUSAN-75]` `[KEPUTUSAN-79]` |
| Akses ke API | Route Handler Next.js meneruskan panggilan ke API Rust dengan token akses pengguna. API Rust tetap memvalidasi token (ADR-011 butir 8) dan memeriksa otorisasi data (ADR-012) pada setiap permintaan | `[KEPUTUSAN-75]` |

Ketentuan tambahan:
- Server Next.js tidak menyimpan data bisnis dan tidak mengambil keputusan otorisasi data. Keputusan itu tetap di backend Rust (ADR-012) `[KEPUTUSAN-75]`.
- Server Next.js setiap web punya identitas OpenBao sendiri dengan policy minimum (ADR-025) `[KEPUTUSAN-79]`.
- **Server Next.js masuk modul kritis T1** pada kategori autentikasi dan sesi. Daftar modul kritis di `CLAUDE.md` diperbarui setelah gate Tahap 5.
- Tafsiran ini ditampilkan kepada user di gate Tahap 5. Teks ADR-011 tidak diubah; baris Status-nya diberi catatan rujukan ke ADR ini.

**4. Aplikasi mobile memakai Flutter** `[USER] P7.4, T5-Q1` `[KEPUTUSAN-76]`:
- Flutter 3.50.0 dengan Dart 3.14.0 `[SRC-134]`. Dua aplikasi (Pasien dan Tenaga Kesehatan, ADR-006 butir 3) dan paket bersama dikelola sebagai Dart pub workspace `[SRC-144]`.
- Login: flutter_appauth (Authorization Code + PKCE lewat browser sistem, RFC 8252) `[SRC-135]` `[SRC-77]`. Token disimpan dengan flutter_secure_storage 11.2.0 `[SRC-136]` (ADR-011 butir 7).
- Kunci aplikasi tenaga kesehatan setelah 15 menit tidak aktif (CR-009) memakai local_auth `[SRC-137]`.
- Video: livekit_client `[SRC-138]`. HTTP: dio `[SRC-139]`. Navigasi: go_router `[SRC-140]`. State: flutter_riverpod `[SRC-141]`. WebSocket: web_socket_channel 3.0.3, rilis terakhir 2025-04, dipantau `[SRC-142]`.
- Laporan crash: sentry_flutter 10.5.0 mengirim ke GlitchTip yang dijalankan sendiri (ADR-025), dengan `sendDefaultPii` dimatikan `[SRC-143]` `[SRC-150]` (ADR-010 butir 4).
- Klien API Dart dibangkitkan dari OpenAPI. Generatornya diverifikasi dan dipilih di Tahap 8 Fase 1. Mobile developer boleh mengusulkan pengganti pustaka state management lewat change request sebelum Tahap 8.
- Pengecualian dari cadangan perangkat dan penyembunyian layar saat aplikasi di latar belakang (ADR-021 butir 1): mekanisme platformnya `[TIDAK DIKETAHUI]` hari ini dan diverifikasi di Tahap 8.

**5. Satu monorepo** `[KEPUTUSAN-89]`:

```
apps/rs-backend/        crate binary Rumah Sehat (api, realtime, worker, audit-relay)
apps/rme-backend/       crate binary RME (api, worker)
crates/                 pustaka Rust bersama (audit, crypto, authz, envelope, platform-jobs, integration-contract, test-support)
web/backoffice/ web/portal/ web/rme/   tiga aplikasi Next.js; web/packages/ (klien API bertipe, BFF auth, komponen UI)
mobile/apps/pasien/ mobile/apps/nakes/ mobile/packages/  (pub workspace)
db/migrations/app/ db/migrations/audit/ db/migrations/rme/   SQL murni, sqlx-cli
db/roles/               SQL role dan grant per database (dijalankan pemilik skema)
openapi/                spesifikasi OpenAPI hasil generate (di-commit) + asyncapi/
infra/                  Compose per lingkungan, Caddyfile, konfigurasi LiveKit, realm Keycloak, policy OpenBao, pgBackRest, aturan Prometheus/Loki, skrip provisioning dan deploy
scripts/verify.sh       pintu verifikasi tunggal
docs/
```

Batas antara Rumah Sehat dan RME di monorepo hanya berupa kode bersama di `crates/` dan `web/packages/`. Tidak ada pemanggilan langsung antar-proses selain Partner Integration API (ADR-006 butir 2, ADR-014).

## Alternatif yang ditolak

- **TypeScript/Node.js + NestJS untuk backend, dengan web Next.js atau React (rekomendasi awal penyusun)**: opsi ini direkomendasikan penyusun pada kedua putaran pilihan T5-Q1. Pembandingnya yang disampaikan ke user adalah Rust yang lebih lambat dikembangkan dan menuntut pemelihara yang menguasai Rust `[DOK: 05-catatan-stack.md]`. User memilih "Rust + Next.js" setelah mendengar penjelasan kelebihan Rust, risiko pengembangan yang lebih lambat, dan kebutuhan pemelihara yang menguasai Rust `[USER] T5-Q1`. Alasan pilihan user di luar penjelasan itu tidak dinyatakan `[TIDAK DIKETAHUI]`.
- **Go + React, Kotlin/Spring Boot + React, Dart + Flutter Web**: termasuk opsi awal T5-Q1 dan tidak dipilih user `[USER] T5-Q1`.
- **Rust + React SPA, TypeScript NestJS + React SPA**: termasuk opsi ulang T5-Q1 dan tidak dipilih user `[USER] T5-Q1`.
- **BFF di backend Rust**: menambah satu permukaan sesi web di backend dan satu lapisan proksi tambahan, padahal server Next.js sudah berjalan sebagai proses server di depan browser `[KEPUTUSAN-75]`. Akibat pilihan ini, server Next.js menjadi modul T1.
- **Next.js tanpa pengerasan**: dua advisory Critical berupa RCE dalam dua bulan berada pada fitur `next/og` dan Image Optimization `[SRC-246]`. Mematikan fitur yang tidak dipakai mengurangi permukaan serangan `[KEPUTUSAN-75]`.
- **Hosting Next.js di Vercel**: semua penyedia wajib memproses data di Indonesia `[USER] T4-Q1`, dan lokasi pemrosesan Vercel tidak diverifikasi. Self-hosting didukung resmi `[SRC-158]`.
- **apalis sebagai antrean job**: lisensi crate `apalis-postgres` di crates.io tercatat "non-standard", sedangkan berkas LICENSE di repositorinya MIT `[SRC-179]`. Relay audit juga butuh urutan yang spesifik (ADR-009 butir 3) `[KEPUTUSAN-74]`.
- **chrono**: berstatus tidak dipelihara menurut RUSTSEC-2026-0543 (catatan riset di `[SRC-177]`) `[KEPUTUSAN-74]`.
- **aes-gcm atau ring di samping aws-lc-rs**: satu pustaka kripto memperkecil permukaan yang harus direview di modul T1 `[KEPUTUSAN-74]`.
- **Alat migrasi di luar ekosistem sqlx (misalnya Flyway atau dbmate)**: sqlx-cli memakai format migrasi yang sama dengan pustaka yang memeriksa query saat compile, sehingga satu alat cukup `[KEPUTUSAN-74]`.
- **Repositori terpisah per komponen**: pustaka bersama (audit, kripto, envelope) harus dipakai Rumah Sehat dan RME tanpa duplikasi (ADR-006 butir 2) `[KEPUTUSAN-89]`.

## Konsekuensi

- **Kecepatan pengembangan.** Rust menambah waktu compile pada setiap siklus `scripts/verify.sh` dan CI, menuntut lebih banyak putaran perbaikan, dan beberapa komponen (antrean job) ditulis sendiri. Besaran perlambatannya `[TIDAK DIKETAHUI]` dan diukur di reference implementation Tahap 8 Fase 1. Hasilnya menjadi masukan estimasi effort Tahap 7 dan Tahap 8, serta urutan pengurangan scope PRD §2.3 `[DOK: 03-prd.md]`. Risikonya dicatat sebagai RT-24.
- **Pemeliharaan setelah go-live (PT-35).** Sistem ini menuntut kemampuan Rust, TypeScript/Next.js, dan Flutter/Dart. Mobile developer hanya dikontrak sampai go-live `[USER] T5-Q7`. Siapa yang memelihara backend Rust, ketiga web, dan aplikasi mobile setelah go-live belum diketahui (PT-35, pemilik user/CEO, tenggat Tahap 7). Risikonya dicatat sebagai RT-29.
- **Riwayat advisory Next.js** `[SRC-246]`. Server Next.js memegang sesi admin, rekanan, dan pengguna RME, sehingga RCE di server Next.js berdampak langsung pada sesi. Kewajiban yang timbul: pemantauan halaman advisory, patch Critical/High paling lambat 7 hari, dan review keamanan atas setiap fitur Next.js baru yang dipakai. Advisory "Cross-client response injection and disclosure in self-hosted Next.js applications forwarding fetched responses" `[SRC-246]` relevan langsung dengan pola Route Handler yang meneruskan respons API Rust. Apakah Next.js 16.4.6 sudah memuat perbaikan untuk advisory itu `[TIDAK DIKETAHUI]`, karena versi terdampak tidak tercatat di SRC-246. Hal ini wajib diperiksa di halaman advisory sebelum versi dipin di Tahap 8 Fase 1. Risikonya dicatat sebagai RT-25.
- **Server Next.js menjadi modul T1** (autentikasi dan sesi). Perubahan di BFF auth (`web/packages/`) dan konfigurasi sesi wajib review penuh oleh code-reviewer dan security, ditambah test negatif: cookie tanpa token CSRF ditolak, sesi kedaluwarsa ditolak, token tidak bocor ke browser, dan sesi portal tidak dapat memanggil API backoffice (ADR-006 butir 5).
- **Tiga toolchain** (Rust, Node.js/pnpm, Flutter/Dart) dijalankan di `scripts/verify.sh` dan CI. Waktu verifikasi penuh bertambah. Mode cepat lokal hanya untuk iterasi, sedangkan git hook dan CI selalu menjalankan mode penuh `[KEPUTUSAN-82]`.
- **Relay audit** berasal dari binary yang sama dengan proses lain, tetapi berjalan sebagai kontainer terpisah dengan identitas OpenBao sendiri, dan hanya memegang kredensial `audit_relay` dan `audit_writer` (ADR-009 butir 3) `[KEPUTUSAN-74]` `[KEPUTUSAN-79]`. Test dan review memastikan proses lain tidak memperoleh kredensial itu.
- **Kompatibilitas klien Valkey** (crate `redis` dan ioredis) belum dibuktikan. Klaim kompatibilitas di beranda Valkey berasal dari pihak ketiga yang dikutip `[SRC-247]`, sehingga kompatibilitas dibuktikan lewat test integrasi Tahap 8.
- **Pustaka 0.x atau yang jarang dirilis** (openapi-fetch, web_socket_channel, keycloak-config-cli) dipantau. Penggantinya diajukan lewat change request bila pemeliharaannya berhenti.
- **Yang ditunda ke Tahap 8**: generator klien API Dart, ESLint dan test unit web, mekanisme pengecualian cadangan perangkat dan penyembunyian layar (ADR-021 butir 1 semula menyebut Tahap 5), dan versi pasti paket mobile yang belum dipin di sini.
