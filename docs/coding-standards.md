# Coding Standards: Ruang Sehat

Tanggal: 2026-11-18
Versi: 1.0, disetujui di gate Tahap 5, 2026-11-18 `[USER] T5-G4`
Penyusun: Claude (persona Principal Engineer / Tech Lead)
Tingkat ketelitian: **T2 Standar**. Bagian yang menyangkut modul kritis (§5.3–§5.7, §7.3, §10) **T1**.
Pembaca: developer di Tahap 8, yaitu Claude (backend Rust, web Next.js) dan mobile developer (Flutter), serta subagent `code-reviewer`.

Dokumen ini mengunci cara menulis kode sebelum baris kode pertama. Kalau dokumen ini bertentangan dengan ADR atau dokumen lain yang sudah lulus gate, yang berlaku dokumen yang lulus gate; laporkan kontradiksinya, jangan diselesaikan diam-diam `[DOK: CLAUDE.md]`. Kalau menemukan pola yang belum ada standarnya, berhenti dan usulkan ADR `[DOK: CLAUDE.md]`.

---

## 0. Cara membaca

### 0.1 Label sumber

| Penanda | Arti |
| --- | --- |
| `ADR-xxx` | Keputusan arsitektur yang sudah diterima `[DOK: adr/]` |
| `US-`, `AB-`, `KU-`, `PB-`, `NFR-`, `E-xx` | Butir PRD v1.1 `[DOK: 03-prd.md]` |
| `[USER] T5-Qx` | Jawaban user Tahap 5 `[DOK: 05-catatan-stack.md]` |
| `[KEPUTUSAN-74]`–`[KEPUTUSAN-89]` | Keputusan teknis Tahap 5 dari lembar keputusan bersama, dicatat orkestrator di `docs/decision-log.md` |
| `[KEPUTUSAN-110]` dan seterusnya | Aturan baru dari dokumen ini. Semuanya terkumpul di §12 beserta alasannya, untuk dicatat orkestrator di `docs/decision-log.md` |
| `[SRC-xx]` | Fakta eksternal terverifikasi di `docs/sources.md`. Dokumen ini tidak menambah SRC baru |
| "diverifikasi Tahap 8" | Fakta eksternal (nama API, opsi lint, perilaku pustaka) yang belum terdaftar di `docs/sources.md`. Wajib diverifikasi ke dokumentasi resmi versi terpasang sebelum dipakai. Daftarnya di §13 |

### 0.2 Status contoh kode

Semua blok kode di dokumen ini adalah **ilustrasi pola**. Nama fungsi, tipe, dan makro pustaka di dalamnya **bukan API final**. API final ditetapkan di reference implementation Tahap 8 Fase 1, dan setiap pemanggilan pustaka pihak ketiga diverifikasi ke dokumentasi resmi versi terpasang `[DOK: .claude/skills/runbook/references/anti-halu.md]` §5.

### 0.3 Kata kunci aturan

- **Wajib**: pelanggaran membuat review gagal.
- **Dilarang**: pelanggaran membuat review gagal.
- **Dianjurkan**: penyimpangan boleh, dengan alasan di deskripsi commit.

---

## 1. Struktur repositori dan layering

### 1.1 Monorepo

Struktur puncak mengikuti `[KEPUTUSAN-89]`:

```
apps/rs-backend/        crate binary Ruang Sehat: subcommand api | realtime | worker | audit-relay |
                        audit-verify | privacy-worker
apps/rme-backend/       crate binary RME minimalis: subcommand api | worker | audit-relay | audit-verify |
                        privacy-worker
crates/                 pustaka Rust bersama: audit, crypto, authz, envelope, platform-jobs,
                        integration-contract, test-support, foundation (lihat KEPUTUSAN-110)
web/backoffice/         aplikasi Next.js backoffice
web/portal/             aplikasi Next.js portal rekanan
web/rme/                aplikasi Next.js web RME
web/packages/           klien API bertipe, BFF auth, komponen UI
mobile/apps/pasien/     aplikasi Flutter pasien
mobile/apps/nakes/      aplikasi Flutter tenaga kesehatan
mobile/packages/        paket Dart/Flutter bersama (pub workspace)
db/migrations/app/      migrasi SQL database aplikasi Ruang Sehat
db/migrations/audit/    migrasi SQL database audit (Ruang Sehat dan RME)
db/migrations/rme/      migrasi SQL database RME
db/roles/               bootstrap klaster idempoten oleh superuser: role, atribut, keanggotaan, skema,
                        ekstensi, pencabutan PUBLIC; tanpa password (hak per tabel ada di migrasi, §5.3)
openapi/                spesifikasi OpenAPI hasil generate (di-commit) + asyncapi/
infra/                  Compose per lingkungan, Caddyfile, LiveKit, realm Keycloak,
                        policy OpenBao, pgBackRest, Prometheus/Loki/alert rules, skrip deploy
scripts/verify.sh       pintu verifikasi tunggal
docs/
```

Aturan:

- Rust memakai satu Cargo workspace dengan toolchain dipin di `rust-toolchain.toml` (Rust stable 1.100.0, edisi 2024) `[KEPUTUSAN-74]` `[SRC-161]`.
- Pustaka bersama dipakai RME lewat workspace, **tanpa ketergantungan runtime** antara Ruang Sehat dan RME (ADR-006 butir 2) `[KEPUTUSAN-74]`. `apps/rme-backend` **dilarang** bergantung pada `apps/rs-backend`, dan sebaliknya.
- Crate `foundation` adalah tambahan dari dokumen ini untuk tipe dasar lintas crate: `Clock`, pembuat ID UUIDv7, `Idr`, `CorrelationId`, dan `Sensitive<T>` `[KEPUTUSAN-110]`. Tanpa crate ini, tipe dasar itu akan tersebar di crate yang tidak sesuai tanggung jawabnya.

| Crate bersama | Tanggung jawab | Modul kritis |
| --- | --- | --- |
| `foundation` | `Clock`, `new_id()` UUIDv7, `Idr`, `CorrelationId`, `Sensitive<T>` | Tidak (tetapi `Sensitive<T>` dipakai modul T1) |
| `envelope` | Tipe respons sukses/galat ADR-023, `ResponseCode`, trait pemetaan galat ke HTTP | Tidak |
| `authz` | Registri kebijakan per endpoint, evaluasi peran + relasi, tipe cakupan tenant | **T1** (ADR-012) |
| `audit` | Tipe event E-01–E-17, masking, penulisan ke `audit_outbox`, logika relay dan hash berantai | **T1** (ADR-009) |
| `crypto` | AES-256-GCM, HMAC-SHA256, SHA-256, RNG, wadah DEK, klien Transit OpenBao, cache DEK | **T1** (ADR-013) |
| `platform-jobs` | Tabel job, pengambilan `FOR UPDATE SKIP LOCKED`, kunci job tunggal `pg_try_advisory_lock` `[SRC-36]` `[SRC-234]` | Tidak (kecuali dipakai relay audit) |
| `integration-contract` | DTO Partner Integration API v1, skema tanda tangan webhook | **T1** (ADR-014) |
| `test-support` | Penyiapan testcontainers, pembangkit data sintetis, `FixedClock`, token uji | Tidak |

### 1.2 Backend Rust: layering

Alur satu permintaan:

```
handler axum (http.rs)  →  service / use case (service.rs)  →  repository sqlx (repo.rs)  →  PostgreSQL
        │                         │
        │                         ├─ authz (relasi objek), audit (outbox), crypto (enkripsi)
        │                         └─ api publik modul lain (api.rs milik modul lain)
        └─ middleware lintas modul: korelasi, validasi token, kebijakan endpoint, pemetaan galat → envelope
```

Tanggung jawab per lapis:

| Lapis | Boleh | Dilarang |
| --- | --- | --- |
| **Handler** (`http.rs`) | Mengambil konteks autentikasi dari ekstraktor; deserialisasi dan validasi bentuk DTO (§5.1); memanggil satu fungsi service; memetakan hasil ke envelope ADR-023; anotasi OpenAPI (utoipa `[SRC-168]`) | SQL, membuka transaksi, logika bisnis, keputusan otorisasi relasi, memanggil modul lain langsung |
| **Service** (`service.rs`) | Validasi domain; pemeriksaan relasi lewat `authz`; satu transaksi per use case (§5.2); memanggil repository modulnya sendiri dan `api` modul lain; menulis event audit dan outbox; enkripsi/dekripsi lewat `crypto` | SQL langsung, membaca tabel modul lain, mengetahui tipe HTTP (status, header) |
| **Repository** (`repo.rs`) | Query sqlx yang diperiksa saat compile `[SRC-166]`; memetakan baris ke tipe domain; filter default `deleted_at IS NULL` (ADR-003) | Logika bisnis, memanggil service, mengakses tabel milik modul lain |
| **Domain** (`domain.rs`) | Tipe dan aturan murni (transisi status, perhitungan harga) tanpa I/O | I/O apa pun, jam sistem langsung (§5.7) |

Dependensi hanya mengalir ke bawah: handler → service → repository. Repository tidak tahu service, service tidak tahu handler.

### 1.3 Modul bisnis Ruang Sehat

Setiap modul bisnis di `docs/04-arsitektur.md` §3 menjadi satu modul Rust di `apps/rs-backend/src/modules/`. Nama modul bahasa Inggris snake_case `[KEPUTUSAN-111]`:

| Modul arsitektur §3 | Modul Rust | Kode modul `response_code` `[KEPUTUSAN-84]` |
| --- | --- | --- |
| Identitas dan akses | `identity` | 01 |
| Profil dan verifikasi | `profile` | 02 |
| Tenaga kesehatan | `practitioner` | 03 |
| Rekanan | `partner` | 04 |
| Katalog salinan | `catalog` | 05 |
| Pemesanan | `ordering` | 06 |
| Konsultasi | `consultation` | 07 |
| Pembayaran | `payment` | 08 |
| Hasil lab | `lab_result` | 09 |
| Notifikasi | `notification` | 10 |
| Konten dan layanan pelanggan | `content_support` | 11 |
| Portal rekanan | `partner_portal` | 12 |
| Gateway integrasi | `integration_gateway` | 13 |
| Audit (halaman Audit Log, export) | `audit_log` | 14 |
| Privasi | `privacy` | 15 |
| KPI | `kpi` | 16 |
| Platform (berkas, job, kalender, parameter) | `platform` | 17 |

Modul RME minimalis (`docs/04-arsitektur.md` §3) mengikuti pola yang sama di `apps/rme-backend/src/modules/`, dengan kode modul 50–60 `[KEPUTUSAN-84]`.

Isi satu modul `[KEPUTUSAN-112]`:

```
modules/ordering/
  mod.rs        // pub(crate) mod api;  mod http; mod service; mod repo; mod domain; mod error; mod policy; mod jobs;
  api.rs        // antarmuka publik untuk modul lain (satu-satunya submodul yang terlihat di luar modul)
  http.rs       // handler axum + DTO + anotasi utoipa
  service.rs    // use case
  repo.rs       // query sqlx atas tabel milik modul ini saja
  domain.rs     // tipe dan aturan murni
  error.rs      // tipe galat domain modul (§3)
  policy.rs     // kebijakan endpoint modul, didaftarkan ke crate authz
  jobs.rs       // handler job terjadwal milik modul (bila ada)
```

Aturan batas modul (ADR-006 butir 1 dan konsekuensinya, `[KEPUTUSAN-74]`):

- **Wajib**: hanya `api` yang dideklarasikan `pub(crate)` di `mod.rs`. Submodul lain privat, dan item di dalamnya paling luas `pub(super)`. Akibatnya modul lain tidak dapat memanggil `repo` atau `service` modul ini; compiler yang menolaknya. Perilaku visibilitas ini dibuktikan dengan test kompilasi di reference implementation (diverifikasi Tahap 8).
- **Dilarang**: modul membaca atau menulis tabel milik modul lain, termasuk lewat `JOIN`. Data modul lain diambil lewat fungsi di `api.rs` modul pemiliknya.
- **Wajib**: kepemilikan tabel dicatat di `apps/rs-backend/module-tables.toml` (dan `apps/rme-backend/module-tables.toml`), diturunkan dari pemetaan modul ke data dictionary di `docs/04-arsitektur.md` §3. Test arsitektur di `scripts/verify.sh` (crate `arch-check`, langkah `test-arsitektur`) memeriksa bahwa setiap nama tabel di literal SQL dan berkas `.sql` suatu modul terdaftar milik modul itu, bahwa modul di `src/modules` terdaftar, bahwa tiap tabel punya satu pemilik, dan bahwa tabel yang dibuat migrasi `db/migrations/<database>/` punya pemilik `[KEPUTUSAN-113]` `[KEPUTUSAN-82]`. Nama tabel dinamis (`{x}`) di literal SQL ditolak. SQL yang dirakit saat runtime dari potongan non-literal tidak terlihat oleh pemeriksa; reviewer menolaknya (§10).
- Pengecualian yang sah: tabel `audit_outbox` hanya ditulis lewat crate `audit`; tabel job dan outbox integrasi lewat `platform-jobs`/modul `platform`; tabel `data_key` lewat crate `crypto`. Pengecualian ini dicatat di `module-tables.toml` sebagai milik crate tersebut.
- Fungsi `api.rs` yang mengubah data **menerima transaksi milik pemanggil** (`&mut Tx`), supaya satu use case lintas modul tetap satu transaksi database (ADR-009 butir 2) `[KEPUTUSAN-114]`.

```rust
// Ilustrasi, API final di reference implementation Tahap 8
// modules/payment/api.rs
pub(crate) async fn record_payment_attempt(
    tx: &mut Tx<'_>,            // transaksi milik use case pemanggil (modul ordering)
    ctx: &RequestContext,
    order: LockedOrderRef,
    idempotency_key: IdempotencyKey,
) -> Result<PaymentAttemptId, PaymentError> { /* ... */ }
```

### 1.4 Proses dan subcommand

- Binary `rs-backend` dengan subcommand `api | realtime | worker | audit-relay | audit-verify | privacy-worker`; `rme-backend` dengan `api | worker | audit-relay | audit-verify | privacy-worker` `[KEPUTUSAN-74]` `[DOK: review/tahap-05-dba.md]`.
- Setiap subcommand dijalankan sebagai deployment sendiri dengan **identitas OpenBao sendiri** dan policy minimum; kredensial database hanya yang dibutuhkan deployment itu `[KEPUTUSAN-79]` (ADR-021 butir 3).
- `audit-relay` hanya memegang kredensial `audit_relay` dan `audit_writer`; proses lain **dilarang** memuat kredensial itu (ADR-009 butir 3). Relay **tidak** memegang kredensial `audit_verifier`; kredensial itu hanya milik `audit-verify` (ADR-009 butir 4). Kredensial role pekerja privasi hanya milik `privacy-worker` (ADR-013 butir 5).
- Job tunggal (relay audit, pengambil event Keycloak) memakai `pg_try_advisory_lock` tingkat sesi `[SRC-234]` lewat `platform-jobs` `[KEPUTUSAN-74]`. Job biasa diambil dengan `SELECT ... FOR UPDATE SKIP LOCKED` `[SRC-36]` dan **wajib idempoten** (ADR-007 konsekuensi).

### 1.5 Web Next.js

Tiga aplikasi Next.js 16.4.6 App Router `[SRC-156]` terpisah, masing-masing `output: 'standalone'` `[SRC-157]` dan self-hosted `[SRC-158]` `[KEPUTUSAN-75]`.

```
web/backoffice/
  app/                      // App Router
    (protected)/...         // halaman yang butuh sesi
    login/ callback/ logout/
    bff/[...path]/route.ts  // Route Handler BFF: meneruskan panggilan ke API Rust
  lib/server/               // kode khusus server: sesi, klien API dengan token, CSRF
  components/               // komponen khusus aplikasi
  next.config.ts            // images.unoptimized: true; output: 'standalone'
web/packages/
  api-client/               // tipe openapi-typescript + openapi-fetch per API
  bff-auth/                 // OIDC (openid-client), sesi Valkey (ioredis), JWE (jose), CSRF
  ui/                       // komponen UI (pustaka dan token ditetapkan Tahap 6)
```

Aturan:

- **BFF** `[KEPUTUSAN-75]`: server Next.js menjalankan OIDC Authorization Code sebagai klien rahasia dengan openid-client 6.9.0 `[SRC-115]`; data sesi di Valkey lewat ioredis 6.0.0 `[SRC-116]`; token dienkripsi JWE dengan jose 6.2.17 `[SRC-114]` memakai kunci dari OpenBao dan diikat ke ID sesi; cookie `HttpOnly`, `Secure`, `SameSite`, plus token CSRF (ADR-011 butir 7). Token **dilarang** sampai ke browser.
- **Komponen server** boleh memanggil API Rust lewat klien bertipe di `lib/server/` dengan token sesi. **Komponen klien** hanya memanggil Route Handler `bff/` milik aplikasinya sendiri dengan cookie sesi dan token CSRF.
- **Mutasi hanya lewat Route Handler BFF.** Server Actions/Server Functions tidak dipakai untuk mutasi, supaya pemeriksaan CSRF, korelasi, dan penerusan ke API Rust lewat satu jalur `[KEPUTUSAN-115]`.
- **Dilarang** menaruh logika otorisasi data di Next.js. Server Next.js hanya memeriksa "ada sesi atau tidak" untuk pengalihan tampilan; keputusan akses data selalu di backend Rust (ADR-012) `[KEPUTUSAN-75]`. Middleware Next.js **dilarang** dipakai untuk keputusan otorisasi `[KEPUTUSAN-75]`.
- **Dilarang** menyimpan data bisnis di server Next.js, termasuk cache halaman/data. `use cache`/Cache Components, `next/og`, dan Edge runtime tidak dipakai; `images.unoptimized: true` `[SRC-159]` `[KEPUTUSAN-75]` (latar: advisory Critical di Image Optimization dan `next/og` `[SRC-246]`). Pengambilan data server memakai mode tanpa cache (opsi persisnya diverifikasi Tahap 8).
- Server Next.js termasuk modul kritis T1 untuk bagian autentikasi dan sesi (`web/packages/bff-auth`, `lib/server/`, `bff/`) `[KEPUTUSAN-75]`.
- **Dilarang** menaruh rahasia di variabel lingkungan yang diekspos ke bundel browser (prefiks `NEXT_PUBLIC_`; perilaku diverifikasi Tahap 8).
- Klien API dibangkitkan dengan openapi-typescript 7.13.0 + openapi-fetch 0.17.0 `[SRC-131]` `[SRC-132]` dari berkas di `openapi/`; TanStack Query 5.108.0 `[SRC-130]` untuk status data di komponen klien `[KEPUTUSAN-75]`.

### 1.6 Mobile Flutter

Flutter 3.50.0 / Dart 3.14.0 `[SRC-134]` dengan Dart pub workspaces `[SRC-144]` `[KEPUTUSAN-76]`.

```
mobile/
  pubspec.yaml                  // akar workspace
  apps/pasien/lib/
    main.dart
    app/                        // router (go_router), tema, provider akar (riverpod), flavor/lingkungan
    features/<fitur>/
      data/                     // repository: memanggil klien API bersama, memetakan ke model fitur
      domain/                   // model dan aturan tampilan (opsional untuk fitur sederhana)
      presentation/             // layar, widget, controller/provider
  apps/nakes/lib/ ...           // struktur sama
  packages/
    rs_api_client/              // klien API Dart hasil generate dari openapi/ (generator dipilih Tahap 8)
    rs_auth/                    // flutter_appauth + flutter_secure_storage + penyegaran token
    rs_core/                    // pembacaan envelope ADR-023, pemetaan response_code, Clock, pembungkus crash report
    rs_realtime/                // klien WebSocket (web_socket_channel) + penyambungan ulang
    rs_design/                  // komponen dan token desain (Tahap 6)
```

Nama paket bersama memakai awalan `rs_` `[KEPUTUSAN-116]`. Pustaka mengikuti `[KEPUTUSAN-76]`: flutter_appauth `[SRC-135]`, flutter_secure_storage 11.2.0 `[SRC-136]`, local_auth `[SRC-137]`, livekit_client `[SRC-138]`, dio `[SRC-139]`, go_router `[SRC-140]`, flutter_riverpod `[SRC-141]`, web_socket_channel 3.0.3 `[SRC-142]`, sentry_flutter 10.5.0 dengan `sendDefaultPii` mati, dikirim ke GlitchTip `[SRC-143]` `[SRC-255]` `[SRC-150]`. Mobile developer boleh mengusulkan pengganti pustaka state management lewat CR sebelum Tahap 8 `[KEPUTUSAN-76]`.

Aturan lapisan:

- `presentation` hanya memanggil `data` lewat provider; **dilarang** memanggil `rs_api_client` atau dio langsung dari widget.
- `data` adalah satu-satunya lapis yang tahu bentuk DTO API dan `response_code`.
- Fitur di satu aplikasi **dilarang** mengimpor fitur lain secara langsung; yang dipakai bersama dipindah ke `packages/`.
- Aplikasi pasien **dilarang** bergantung pada kode khusus tenaga kesehatan, dan sebaliknya (ADR-006 butir 3).

Aturan data di perangkat (ADR-021 butir 1, T1 untuk `rs_auth`):

- Token hanya di flutter_secure_storage `[SRC-136]`. Login lewat browser sistem dengan Authorization Code + PKCE, **bukan** webview (ADR-011 butir 7, RFC 8252 `[SRC-77]`).
- **Dilarang** menyimpan isi klinis (chat, keluhan awal, catatan, resep, hasil lab, lampiran) di penyimpanan persisten aplikasi. Pengecualian: berkas yang sengaja diunduh pasien dan kode/QR appointment (US-403 AC2). Antrean pesan keluar yang belum terkirim disimpan terenkripsi dengan kunci dari penyimpanan aman, lalu dihapus setelah terkirim atau saat keluar.
- Pengecualian dari cadangan perangkat dan penyembunyian layar di latar belakang wajib diterapkan; mekanisme platformnya diverifikasi di Tahap 8 `[KEPUTUSAN-76]`.
- Kunci aplikasi tenaga kesehatan setelah 15 menit tidak aktif memakai local_auth `[SRC-137]` (CR-009).
- **Dilarang** memanggil host di luar daftar izin ADR-021 butir 2 (backend Ruang Sehat, Keycloak, server media video, layanan crash di Indonesia).

---

## 2. Penamaan

### 2.1 Database (PostgreSQL)

| Objek | Aturan | Contoh | Sumber |
| --- | --- | --- | --- |
| Tabel, kolom, index, constraint, nama berkas migrasi | snake_case | `patient_order`, `created_at` | ADR-001 |
| Nama tabel | Tunggal, bahasa Inggris, bukan kata kunci SQL | `patient_order` (bukan `order`), `account` (bukan `user`) | ADR-008, `[KEPUTUSAN-38]`, `[KEPUTUSAN-39]` |
| Riwayat status dan proyeksi | `<entitas>_status_event`, `<entitas>_state` | `refund_status_event`, `refund_state` | ADR-008 |
| Primary key | `id uuid`, UUIDv7 dari aplikasi; proyeksi memakai `<entitas>_id` sebagai PK | — | ADR-002, ADR-008 |
| Foreign key | `<tabel_rujukan>_id` | `partner_id`, `care_recipient_id` | Data dictionary §0.2 |
| Uang | `bigint`, akhiran `_idr` | `total_idr` | ADR-015 butir 1 |
| Persen | `integer` basis poin, akhiran `_bp` | `percent_bp` | `[KEPUTUSAN-59]` |
| Instan waktu | `timestamptz`, akhiran `_at` | `occurred_at` | ADR-017 |
| Tanggal kalender | `date`, akhiran `_on` atau `_date` | — | Data dictionary §0.2 |
| Kolom terenkripsi aplikasi | `<nama>_enc bytea` + `data_key_id` | `body_enc`, `nik_enc` | ADR-013, data dictionary §0.2 |
| Blind index | `<nama>_bidx` + `<nama>_bidx_key_version` | `nik_bidx` | ADR-013 butir 4 |
| Empat digit terakhir | `<nama>_last4 char(4)` | `nik_last4`, `account_number_last4` | Data dictionary §0.2 |
| Boolean | Berbentuk predikat | `is_active`, `requires_expiry` | Data dictionary |
| Enum | `text` + `CHECK`, kode bahasa Inggris snake_case | `'awaiting_payment'` | `[KEPUTUSAN-39]` |
| Constraint dan index | Awalan `pk_`, `fk_`, `uq_` (unik penuh), `uqp_` (partial unique), `ix_`, `ck_`, `tg_` (trigger), ditambah `ex_`, `fn_`, `pol_` dan aturan panjang di `docs/05-physical-schema.md` §1, diikuti `<tabel>_<kolom/arti>` | `uqp_account_owner_nik_registry_nik_bidx` | `[KEPUTUSAN-117]` |

Data dictionary §0.2 menyerahkan format nama constraint ke Tahap 5. Kalau `docs/05-physical-schema.md` (disusun paralel) menetapkan format berbeda, perbedaan itu dilaporkan dan salah satu dokumen dikoreksi di gate Tahap 5.

### 2.2 Rust

Mengikuti konvensi bahasa Rust (rujukan Rust API Guidelines, diverifikasi Tahap 8):

| Item | Aturan | Contoh |
| --- | --- | --- |
| Crate, modul, fungsi, variabel, field | snake_case | `lab_result`, `record_payment_attempt` |
| Tipe, trait, varian enum | PascalCase | `OrderError`, `InvalidTransition` |
| Konstanta | SCREAMING_SNAKE_CASE | `MAX_PAGE_SIZE` |
| Field DTO | Sama dengan field JSON (snake_case bahasa Inggris), tanpa penggantian nama | `total_idr` |

Tipe domain yang membawa makna diberi newtype supaya tidak tertukar: `Idr`, `PartnerId`, `PatientOrderId`, dan sejenisnya `[KEPUTUSAN-118]`.

### 2.3 TypeScript (Next.js)

| Item | Aturan |
| --- | --- |
| Variabel, fungsi | camelCase |
| Komponen React, tipe, interface | PascalCase |
| Berkas komponen | PascalCase (`OrderTable.tsx`); berkas lain kebab-case (`session-store.ts`) |
| Folder rute App Router | kebab-case, sesuai URL |
| Field data API | **snake_case apa adanya** dari tipe hasil generate openapi-typescript. Dilarang memetakan ulang ke camelCase `[KEPUTUSAN-119]` |

TypeScript dijalankan dengan mode ketat; konfigurasi persisnya ditetapkan di Tahap 8 bersama ESLint `[KEPUTUSAN-75]`.

### 2.4 Dart (Flutter)

Mengikuti panduan gaya Effective Dart (diverifikasi Tahap 8): tipe UpperCamelCase, anggota dan variabel lowerCamelCase, berkas dan paket lowercase_with_underscores. Nama field Dart di model hasil generate mengikuti generator yang dipilih di Tahap 8, dengan kunci JSON tetap snake_case sesuai kontrak `[KEPUTUSAN-76]` `[KEPUTUSAN-119]`.

### 2.5 API HTTP

- Field JSON dan kode bahasa Inggris snake_case (ADR-023 butir 4, `[KEPUTUSAN-39]`). Pesan untuk pengguna Bahasa Indonesia (NFR-LOC-01).
- Waktu ISO 8601 dengan offset (ADR-017 butir 1). Uang bilangan bulat Rupiah dengan akhiran `_idr` (ADR-015). ID berupa UUID (ADR-002).
- Awalan path per audiens, selaras dengan berkas di `openapi/` `[KEPUTUSAN-83]` dan namespace terpisah ADR-006 butir 5 `[KEPUTUSAN-120]`:

| Berkas OpenAPI | Awalan path |
| --- | --- |
| pasien | `/patient/v1` |
| nakes | `/practitioner/v1` |
| backoffice | `/backoffice/v1` |
| portal | `/portal/v1` |
| partner-v1 | `/partner/v1` (ADR-014 butir 1) |
| webhook | `/webhook/<penyedia>` |
| rme | `/rme/v1` (backend RME) |

- Segmen path kata benda jamak kebab-case; parameter path dan query snake_case. Contoh: `GET /patient/v1/care-recipients/{care_recipient_id}`, `?cursor=...&limit=...` `[KEPUTUSAN-120]`.
- Daftar memakai pagination cursor (`meta.next_cursor`, `meta.has_more`, ADR-023 butir 1).

---

## 3. Galat dan format respons API

### 3.1 Envelope

Semua respons HTTP memakai envelope ADR-023 butir 1: `response_code` (7 digit `HHHMMCC`), `response_description` (Bahasa Indonesia), `data` (sukses), `meta` (daftar), `errors` (validasi: `{ field, code, message }`), `correlation_id` (wajib di semua galat). Kode HTTP tetap dipakai di header dan 3 digit pertama `response_code` sama dengan kode HTTP (ADR-023 butir 2, konsekuensi). Galat memakai media type `application/problem+json` dengan member `type` RFC 9457 `[SRC-47]`; sukses `application/json` (ADR-023 butir 3). `type` = `https://<domain-dokumentasi>/problems/<response_code>` `[KEPUTUSAN-84]`.

Registry kode ada di **`docs/05-response-codes.md`** `[KEPUTUSAN-84]`. Kode yang tidak terdaftar di registry **dilarang** dipakai; `scripts/verify.sh` memeriksa setiap `response_code` di spesifikasi terdaftar dan konsisten dengan kode HTTP `[KEPUTUSAN-82]`. Kode spesifik modul ditambahkan story per story di Tahap 8 sebagai bagian Definition of Done; satu kode kasus tidak dipakai ulang untuk arti berbeda (ADR-023 butir 6).

Contoh sukses dan galat validasi:

```json
{ "response_code": "2000600", "response_description": "Berhasil",
  "data": { "patient_order_id": "0193...", "total_idr": 250000, "created_at": "2027-03-02T09:15:00+07:00" } }
```

```json
{ "type": "https://<domain-dokumentasi>/problems/4220001",
  "response_code": "4220001", "response_description": "Data tidak valid",
  "errors": [ { "field": "chief_complaint", "code": "too_long", "message": "Keluhan maksimal 500 karakter" } ],
  "correlation_id": "0193..." }
```

### 3.2 Tipe galat domain

- Setiap modul punya satu enum galat di `error.rs`. Varian menyatakan **arti bisnis** (misalnya `InvalidTransition`, `PaymentAmountMismatch`), bukan kode HTTP.
- Varian `Internal` membungkus sumber galat teknis untuk log; isinya **dilarang** sampai ke respons.
- Pemetaan ke HTTP dan `response_code` dilakukan di satu tempat per modul lewat trait dari crate `envelope`, bukan di handler.
- `ResponseCode::new` (dan alias `ResponseCode as ...`) hanya boleh muncul di crate `envelope` dan di `error.rs` modul (`apps/*/src/modules/*/error.rs`); di tempat lain (termasuk `http.rs`) test arsitektur menolaknya (F1-T12). Kode baru tetap wajib terdaftar di `docs/05-response-codes.md`.

```rust
// Ilustrasi, API final di reference implementation Tahap 8
pub(super) enum OrderError {
    NotFound,                                      // termasuk objek yang tidak boleh diakses (ADR-012 butir 4)
    Forbidden,                                     // aksi dikenal, peran tidak diizinkan
    InvalidTransition { from: OrderStatus, to: OrderStatus },
    Validation(Vec<FieldError>),
    PartnerTimeout,                                // pemeriksaan sebelum bayar > 5 detik (ADR-014 butir 5)
    Internal(InternalError),                       // dicatat di log, tidak pernah ke respons
}

impl envelope::ApiError for OrderError {
    fn http_and_code(&self) -> (StatusCode, ResponseCode) {
        match self {
            Self::NotFound => (StatusCode::NOT_FOUND, codes::NOT_FOUND),                 // 4040001
            Self::Forbidden => (StatusCode::FORBIDDEN, codes::FORBIDDEN),                // 4030001
            Self::InvalidTransition { .. } => (StatusCode::CONFLICT, codes::STATUS_CONFLICT), // 4090001
            Self::Validation(_) => (StatusCode::UNPROCESSABLE_ENTITY, codes::INVALID),   // 4220001
            Self::PartnerTimeout => (StatusCode::GATEWAY_TIMEOUT, codes::UPSTREAM_TIMEOUT), // 5040001
            Self::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, codes::INTERNAL),   // 5000001
        }
    }
}
```

### 3.3 Pemetaan kasus umum

Kode umum modul 00 dari `[KEPUTUSAN-84]` (rincian dan deskripsi di `docs/05-response-codes.md`):

| Situasi | HTTP | `response_code` | Catatan |
| --- | --- | --- | --- |
| Body tidak dapat dibaca (JSON rusak) | 400 | `4000001` | |
| Tanpa token, token tidak valid atau kedaluwarsa, sesi dicabut | 401 | `4010001` | Pesan umum, tanpa alasan rinci |
| Aksi dikenal tetapi peran tidak diizinkan | 403 | `4030001` | ADR-012 butir 4 |
| Objek tidak ada **atau tidak boleh diakses** | 404 | `4040001` | KU-03, ADR-012 butir 4, ADR-023 butir 5 |
| Transisi status kalah atau tidak sah | 409 | `4090001` | AB-36, ADR-008 butir 3 |
| Kunci idempotensi dipakai dengan isi berbeda | 409 | `4090002` | ADR-014 butir 4 |
| Berkas terlalu besar / jenis tidak didukung | 413 / 415 | `4130001` / `4150001` | ADR-018 |
| Validasi gagal | 422 | `4220001` + `errors` | KU-01 |
| Batas laju | 429 | `4290001` | |
| Galat tak terduga | 500 | `5000001` | Rinciannya hanya di log dengan `correlation_id` yang sama |
| Pihak ketiga membalas tidak benar / tidak tersedia / habis waktu | 502 / 503 / 504 | `5020001` / `5030001` / `5040001` | KU-08 |

### 3.4 Larangan

- **Dilarang** `unwrap()`, `expect()`, `panic!`, `todo!`, `unimplemented!`, dan pengindeksan slice/peta yang dapat panik (`v[i]`) di kode non-test. Gunakan `?`, `ok_or(...)`, atau `.get(...)`. Lint clippy `unwrap_used`, `expect_used`, `panic`, `todo`, `unimplemented`, `indexing_slicing`, dan `dbg_macro` diaktifkan sebagai galat untuk seluruh workspace lewat `[workspace.lints.clippy]` di `Cargo.toml` root, dengan `allow-*-in-tests` di `clippy.toml` root `[KEPUTUSAN-82]` `[SRC-387]` (F1-T12). Setiap crate wajib `[lints] workspace = true` dan dilarang punya `[lints.*]` atau `clippy.toml` sendiri; crate `arch-check` menolak pelemahan itu. Pengecualian hanya untuk invarian yang terbukti saat compile atau saat start (misalnya konstanta yang divalidasi test), dengan atribut `allow` lokal ber-komentar alasan dan disetujui reviewer `[KEPUTUSAN-121]`. Atribut `allow`/`expect` atas lint larangan di kode non-test hanya sah bila tercatat di `architecture-exceptions.toml` (berkas, fungsi, jumlah kemunculan persis, alasan); kemunculan di luar catatan atau catatan usang menggagalkan test arsitektur. Di test, `unwrap`/`expect` boleh: `allow-*-in-tests` mencakup fungsi `#[test]` dan `#[cfg(test)]`, sedangkan fungsi pembantu di `tests/*.rs` memakai `#![allow(clippy::unwrap_used, clippy::expect_used)]` di berkas itu.
- **Dilarang** memuat stack trace, teks query, pesan galat pustaka, data pribadi, atau rahasia di respons (ADR-023 butir 5, ADR-010).
- **Dilarang** membedakan "tidak ada" dan "tidak boleh" di respons, waktu tanggap yang mencolok sengaja, atau pesan galat (KU-03).
- Pesan galat login dan atur ulang password bersifat umum (ADR-011 butir 4).
- Aritmetika uang **wajib** memakai operasi yang memeriksa luapan (`checked_*`) lewat tipe `Idr`; luapan = galat `Internal`, bukan pembulatan diam-diam `[KEPUTUSAN-118]`.

### 3.5 Galat di Next.js dan Flutter

- Klien membaca `response_code` untuk logika dan menampilkan `response_description` atau `errors[].message` ke pengguna (ADR-023 konsekuensi). **Dilarang** mengurai teks deskripsi untuk logika.
- Route Handler BFF meneruskan envelope galat dari API Rust apa adanya. Galat BFF sendiri (misalnya CSRF gagal, sesi habis) memakai envelope dan kode dari registry yang sama.
- Galat tak terduga di klien dilaporkan ke GlitchTip tanpa data pribadi `[KEPUTUSAN-80]`.

---

## 4. Logging dan audit trail

Tiga log terpisah (ADR-010): **audit trail** (fitur produk, DB audit), **application log** (debugging, Loki tenant `app`, 30 hari), **security log** (deteksi ancaman, Loki tenant `security`, 1 tahun) `[KEPUTUSAN-80]` `[SRC-236]`. Audit trail **tidak** pernah ditulis ke Loki, dan application log **tidak** pernah dipakai sebagai pengganti audit trail `[KEPUTUSAN-80]`.

### 4.1 Application log

- Rust memakai tracing 0.1.44 + tracing-subscriber 0.3.23 dengan format JSON (fitur `json`) `[SRC-174]` `[SRC-175]` `[SRC-238]` `[KEPUTUSAN-74]`.
- Field standar setiap baris (ADR-010): waktu, level, `component` (`api`, `realtime`, `worker`, `audit-relay`, `audit-verify`, `privacy-worker`, `rme-api`, `rme-worker`, `web-backoffice`, dan seterusnya), `correlation_id`, `route` berupa **pola** rute (misalnya `/patient/v1/orders/{order_id}`), status, `duration_ms`, jenis galat.
- Hanya ID internal yang boleh muncul. **Dilarang** mencatat: isi body permintaan atau respons, header `Authorization`, cookie, token apa pun, nilai sesi, parameter tanda tangan URL, password, PIN, OTP, rahasia MFA, NIK, nomor rekening/kartu, email, nama, nomor HP, alamat, isi klinis, kode verifikasi resep, dan nilai kolom `_enc` setelah didekripsi (ADR-005, ADR-010 butir 1 dan 5, ADR-012 butir 3, PRD §7.3).
- **Dilarang** mencatat objek domain utuh (ADR-010 butir 1). Yang dicatat field ID satu per satu.
- Nilai sensitif di kode dibungkus `Sensitive<T>` dari crate `foundation`. Implementasi `Debug`/`Display`-nya menulis `[REDACTED]`, dan tipe ini tidak mengimplementasikan serialisasi umum, sehingga tidak bisa ikut masuk log tanpa sengaja `[KEPUTUSAN-122]`. Tipe yang memuat field sensitif **dilarang** memakai `derive(Debug)` biasa.
- Test otomatis memeriksa contoh keluaran log terhadap pola data terlarang (ADR-010 butir 1), dijalankan di `scripts/verify.sh` langkah test arsitektur `[KEPUTUSAN-82]`. Sejak F1-T12 test arsitektur juga menolak di kode non-test: field log berformat Debug (`?x`, `{x:?}`), field atau nilai yang namanya memuat segmen data terlarang (`password`, `pin`, `otp`, `token`, `secret`, `authorization`, `cookie`, `nik`, `email`, `phone`, `address`, `body`, `payload`, dan sejenisnya), `#[instrument]` tanpa `skip_all`, serta `println!`/`eprintln!`/`dbg!`.
- Next.js: logger server menulis JSON dengan field yang sama dan daftar tolak yang sama; pustakanya dipilih Tahap 8. `console.log` data permintaan dilarang.
- Flutter: **dilarang** `print`/`debugPrint` data pengguna di build rilis; laporan crash lewat sentry_flutter ke GlitchTip dengan `sendDefaultPii` mati `[SRC-143]` `[SRC-255]` `[SRC-150]` `[KEPUTUSAN-76]`.

```rust
// Ilustrasi, API final di reference implementation Tahap 8
tracing::info!(
    component = "api",
    correlation_id = %ctx.correlation_id,
    route = "/patient/v1/orders/{order_id}",   // pola, bukan URL berisi ID
    status = 200,
    duration_ms = elapsed.as_millis() as u64,
    "request_completed"
);
// Dilarang: tracing::debug!(?order, ...)  -> objek domain utuh
// Dilarang: tracing::info!(email = %account.email, ...)
```

### 4.2 ID korelasi

- Header `X-Correlation-Id` `[KEPUTUSAN-123]`. Middleware API menerima nilai masuk hanya bila berbentuk UUID yang valid; bila tidak ada atau tidak valid, dibuat UUIDv7 baru. Dengan begitu nilai dari luar tidak dapat dipakai menyisipkan teks ke log.
- Server Next.js meneruskan header ini ke API Rust; aplikasi mobile mengirim UUID baru per permintaan.
- `correlation_id` ikut tersimpan di baris job, outbox, event audit (field "ID korelasi" PRD §7.2), dan header panggilan keluar ke rekanan, sehingga satu kejadian di audit trail, application log, dan security log dapat dihubungkan (ADR-010 butir 2).
- Nilai yang sama dikembalikan di field `correlation_id` envelope galat (ADR-023 butir 1).

### 4.3 Security log

- Dipancarkan lewat satu fungsi khusus per jenis kejadian (misalnya login gagal dari event Keycloak, akses ditolak termasuk percobaan ganti ID, batas laju, tanda tangan webhook atau kredensial rekanan tidak valid, keterlambatan relay audit, ketidakcocokan proyeksi status, perubahan hak akses; daftar lengkap ADR-010) dengan field `log_stream = "security"`. Agen pengirim log merutekan field itu ke tenant Loki `security` `[KEPUTUSAN-124]` `[KEPUTUSAN-80]`. Agen dan aturan perutean dipilih di Tahap 8 Fase 1 `[KEPUTUSAN-80]`.
- Isi security log tunduk pada daftar tolak yang sama dengan application log.

### 4.4 Memanggil audit trail

Audit trail dibuat aplikasi lewat **satu pustaka**, crate `audit`, yang dipakai semua modul Ruang Sehat dan RME (ADR-009 butir 1). Aturan pemanggilan:

1. **Aksi yang mengubah data**: event ditulis ke `audit_outbox` **di transaksi yang sama** dengan perubahan bisnisnya. Kalau penulisan event gagal, seluruh transaksi batal dan pengguna menerima galat (ADR-009 butir 2, KEPUTUSAN-15).
2. **Akses baca data sensitif (E-05)**: event ditulis dan di-commit **sebelum** data didekripsi dan dikirim. Kalau gagal, akses ditolak (ADR-009 butir 2, ADR-012 butir 2).
3. **Aksi yang gagal atau ditolak** (misalnya akses ditolak menurut KU-03): event ditulis di transaksi tersendiri dan dicoba ulang, tanpa mengubah hasil aksi (ADR-009 butir 2).
4. **Aksi sistem dan integrasi**: job tidak ditandai selesai sebelum event-nya tertulis (ADR-009 butir 2).
5. **Pemanggil tidak melakukan masking.** Masking dan larangan PRD §7.3 diterapkan pustaka (ADR-009 butir 7). API pustaka **tidak menyediakan** tempat untuk password, PIN, OTP, token, atau rahasia MFA; nilai before/after hanya untuk field di daftar izin per jenis objek; NIK dan nomor rekening hanya 4 digit terakhir; field klinis dan dokumen hanya nama field dan versinya.
6. Field wajib PRD §7.2 (aktor, aksi, objek, sumber, hasil, ID korelasi, dan subjek data bila ada) diisi dari `RequestContext`, bukan disusun ulang di setiap handler.
7. Kode event mengikuti PRD §7.1 (E-01–E-17). Story yang menyentuh aksi sensitif wajib mencatat event-nya (ADR-005 konsekuensi, Definition of Done).

```rust
// Ilustrasi, API final di reference implementation Tahap 8
// (1) Transisi status pesanan + E-09 dalam satu transaksi (ADR-008 butir 2, ADR-009 butir 2)
let mut tx = db.begin().await?;
let change = ordering_repo::apply_transition(&mut tx, order_id, expected_version, to_status, clock.now()).await?;
audit::record(
    &mut tx,
    AuditEvent::new(EventCode::E09, ctx.actor(), ObjectRef::patient_order(order_id))
        .subject(change.care_recipient_id)
        .change(FieldChange::status(change.from_status, change.to_status))  // daftar izin per jenis objek
        .outcome(Outcome::Success)
        .context(&ctx),                                                  // sumber + correlation_id
).await?;                     // gagal di sini → tx batal → pengguna menerima galat
tx.commit().await?;
```

```rust
// Ilustrasi, API final di reference implementation Tahap 8
// (2) Tenaga kesehatan membuka riwayat chat: E-05 di-commit sebelum data dikirim
match authz.check(&ctx, Action::ReadChatHistory, Resource::consultation(session_id)).await {
    Decision::Allow => {}
    Decision::Deny(denial) => {
        audit::record_detached(&db, AuditEvent::denied(EventCode::E05, &ctx, ObjectRef::consultation(session_id))).await; // transaksi sendiri, dicoba ulang
        security_log::access_denied(&ctx, "consultation", denial);
        return Err(ConsultationError::NotFound);                       // 404, keberadaan tidak bocor
    }
}
audit::record_committed(&db, AuditEvent::read(EventCode::E05, &ctx, ObjectRef::consultation(session_id))
    .subject(patient_id)).await?;  // gagal → akses ditolak
let messages = consultation_repo::chat_messages(&db, session_id, cursor).await?;
let plaintext = crypto.open_all(&messages, Container::ConsultationChat(session_id)).await?;
```

**Larangan audit**: tidak ada endpoint atau fungsi untuk mengubah atau menghapus event (US-1504); role aplikasi hanya punya `INSERT` di `audit_outbox` (ADR-009 butir 2); test otomatis memastikan event tidak memuat field terlarang (US-1501 AC2, ADR-009 butir 7).

---

## 5. Validasi, transaksi, otorisasi, enkripsi, dan waktu

### 5.1 Validasi input

| Lapis | Yang divalidasi |
| --- | --- |
| **Tepi (handler + skema OpenAPI)** | Bentuk: tipe, field wajib, panjang, format (UUID, ISO 8601 dengan offset, email), enum, rentang angka, ukuran body. DTO menolak field yang tidak dikenal (atribut serde yang setara diverifikasi Tahap 8). Skema OpenAPI dibangkitkan dari tipe DTO yang sama lewat utoipa `[SRC-168]`, sehingga dokumentasi dan validasi tidak menyimpang (ADR-022 butir 3) |
| **Service (domain)** | Aturan bisnis: transisi status (PRD §3.6), batas waktu PB, keunikan NIK lewat blind index (UP-40), kepemilikan dan relasi (lewat `authz`), harga dan ketersediaan (AB-15) |
| **Database** | Pertahanan terakhir: `CHECK`, `UNIQUE`/partial unique, FK, hak role, trigger penolak (ADR-007 butir 2, ADR-008) |

Aturan khusus:

- **Uang**: bilangan bulat Rupiah (`i64` dalam newtype `Idr`; JSON integer), tanpa float atau desimal (ADR-015 butir 1, NFR-LOC-02). Semua perhitungan harga di server; klien **dilarang** menghitung harga yang dikirim balik ke server (ADR-015 butir 1). Persen dalam basis poin (`[KEPUTUSAN-59]`).
- **Waktu**: instan dari klien wajib ISO 8601 dengan offset; tanpa offset ditolak (ADR-017 butir 1).
- **Zona waktu**: nama IANA (misalnya `Asia/Jakarta`), divalidasi terhadap basis data zona (ADR-017 butir 2). Pustaka basis data zona untuk Rust `[TIDAK DIKETAHUI]`, dipilih dan diverifikasi di Tahap 8.
- **NIK**: 16 digit angka, divalidasi sebelum enkripsi (ADR-013 butir 4, `[SRC-63]`).
- **Teks terenkripsi**: panjang divalidasi aplikasi sebelum enkripsi karena database tidak dapat memeriksa isi `_enc` (data dictionary §0.2).
- **Berkas** (ADR-018, `[SRC-50]`): klien meminta slot unggah; backend memeriksa hak, membuat nama objek UUID dan URL unggah bertanda tangan berumur pendek. Setelah unggah, backend memvalidasi ukuran (foto JPG/PNG ≤ 5 MB, PDF ≤ 10 MB), jenis dari **isi berkas** (bukan header `Content-Type` atau ekstensi), mendekode dan mengode ulang foto supaya metadata terbuang, dan memeriksa PDF dengan qpdf 12.4.2 `--json` `[SRC-155]` lalu menolak PDF berkonten aktif (JavaScript, OpenAction, Launch, EmbeddedFile, AcroForm/XFA) `[KEPUTUSAN-86]`. Berkas baru dapat dirujuk setelah lolos validasi. Nama asli berkas tidak dipakai sebagai nama objek.
- **URL endpoint rekanan**: pemeriksaan SSRF saat disimpan dan saat dipanggil (HTTPS wajib, semua alamat hasil resolve publik, koneksi diikat ke alamat tervalidasi, redirect tidak diikuti) (ADR-014 butir 3, `[SRC-81]`).
- **Pesan integrasi masuk**: divalidasi skema dan diterapkan dalam satu transaksi; satu bagian tidak valid menolak seluruh pesan (ADR-014 butir 4, US-1402 AC2).

### 5.2 Transaksi database

- **Satu transaksi per use case**, dibuka di service. Repository menerima transaksi (`&mut Tx`), tidak membuka transaksi sendiri. Fungsi `api` modul lain yang menulis ikut transaksi pemanggil `[KEPUTUSAN-114]`.
- **Jalur tulis status** (ADR-008 butir 2) wajib lewat pustaka status bersama yang dibuat sekali di reference implementation (ADR-008 konsekuensi): (a) baca `version` proyeksi; (b) validasi transisi terhadap tabel transisi PRD §3.6; (c) `INSERT` event dengan `version + 1`; (d) `UPDATE` proyeksi dengan syarat `version` lama; (e) tulis outbox audit E-09. Pelanggaran constraint unik `(<entitas>_id, version)` dievaluasi ulang dari status baru; bila tidak sah, transisi ditolak dengan `4090001` dan dicatat hasil gagal (ADR-008 butir 3, AB-36).
- **Tabel transaksi, riwayat status, dan log**: hanya `INSERT`. Koreksi lewat entri pembalik (ADR-004). Kode **dilarang** mencoba `UPDATE`/`DELETE` atas tabel ini; database juga menolaknya lewat hak role dan trigger penolak (ADR-008).
- **Data master**: soft delete dengan `deleted_at`, `deleted_by`, `delete_reason`; query default memfilter `deleted_at IS NULL`; delete dan restore dicatat E-16 (ADR-003).
- **Outbox** (ADR-007 butir 3): baris `audit_outbox`, outbox integrasi keluar, dan outbox notifikasi ditulis di transaksi yang sama dengan perubahan bisnisnya. Payload outbox integrasi hanya berisi rujukan (jenis pesan + ID entitas), bukan isi klinis atau data pribadi teks terbuka.
- **Tidak ada panggilan HTTP ke pihak ketiga** (payment gateway, rekanan, Keycloak, SES, Lalamove, OpenBao selain cache DEK) **selama transaksi database terbuka** `[KEPUTUSAN-125]`. Polanya: catat niat di transaksi (misalnya percobaan pembayaran beserta kunci idempotensinya **sebelum** penyedia dipanggil, ADR-015 butir 5), commit, panggil pihak ketiga, lalu catat hasil di transaksi baru atau lewat job.
- **Idempotensi**:
  - Pesan masuk rekanan yang mengubah data wajib membawa header `Idempotency-Key` `[KEPUTUSAN-126]`. Kunci disimpan 7 hari (kelas Operasional) bersama hash SHA-256 isi permintaan dan respons yang dikembalikan, dalam transaksi yang sama dengan perubahan bisnisnya. Kunci sama + isi sama → respons yang sama tanpa diproses ulang. Kunci sama + isi berbeda → `4090002` (ADR-014 butir 4).
  - Pesan keluar membawa `event_id` unik (ADR-014 butir 4).
  - Pembuatan entitas dari klien yang dapat dikirim ulang memakai ID dari klien (UUIDv7), misalnya pesan chat (ADR-016 butir 2, KU-05).
  - Refund dieksekusi job dengan kunci idempotensi = ID refund (ADR-015 butir 6). Notifikasi payment gateway unik per ID kejadian penyedia (ADR-015 butir 5).
  - Job wajib idempoten karena dapat dijalankan ulang (ADR-007 konsekuensi).
- **SQL**: hanya lewat makro query sqlx yang diperiksa saat compile; metadata offline `.sqlx` di-commit dan `cargo sqlx prepare --check` di CI `[SRC-166]` `[KEPUTUSAN-74]`. **Dilarang** menyusun SQL dengan penggabungan string dari input. **Dilarang** `SELECT *`; kolom disebut eksplisit.
- **ID**: UUIDv7 dibuat aplikasi lewat `foundation::new_id()` yang memanggil `Uuid::now_v7` (uuid 1.29.3 fitur `std` + `v7`) `[SRC-176]` `[SRC-238]`; bukan nilai bawaan kolom (ADR-002, `[KEPUTUSAN-74]`).

### 5.3 Migrasi skema (T1)

- Hanya lewat berkas SQL murni di `db/migrations/<app|audit|rme>/` dengan sqlx-cli 0.9.1 `[SRC-182]`; maju saja (forward-only), pola expand/contract (ADR-019 butir 2) `[KEPUTUSAN-74]`. Perubahan manual di database dilarang `[DOK: CLAUDE.md]`.
- Bagian deskripsi nama berkas snake_case (ADR-001); format awalan mengikuti sqlx-cli (diverifikasi Tahap 8).
- **Dilarang** mengubah berkas migrasi yang sudah di-commit ke `main`; koreksi lewat migrasi baru.
- **Hak per tabel** (`GRANT`/`REVOKE` sesuai kelas ADR-008) ditulis di berkas migrasi yang membuat atau mengubah tabelnya. `db/roles/` hanya berisi **bootstrap klaster** yang idempoten dan dijalankan superuser: role, atribut role, keanggotaan, skema, ekstensi, dan pencabutan hak `PUBLIC`; **tanpa password** (kredensial dari OpenBao, `[KEPUTUSAN-79]`) `[DOK: review/tahap-05-dba.md]` (N10, F-P08–F-P10).
- **Dilarang** `DISABLE TRIGGER` atau `DROP TRIGGER tg_*_reject_*` di migrasi. Koreksi data kelas Transaksi dan Riwayat status hanya lewat baris pembalik atau versi baru (ADR-004), bukan dengan melepas trigger penolak. `scripts/verify.sh` menolak pola tersebut di berkas migrasi `[DOK: review/tahap-05-dba.md]`.
- Setiap tabel baru: kelasnya tercatat di data dictionary (ADR-008); hak role sesuai kelas di migrasinya; trigger penolak untuk kelas Transaksi, Riwayat status, dan Log; partial unique `WHERE deleted_at IS NULL` untuk Master (ADR-003); RLS `FORCE` untuk tabel tenant RME (ADR-012 butir 6).
- Migrasi yang menyentuh DB audit, role, grant, trigger penolak, atau RLS wajib direview DBA dan keamanan (ADR-021 butir 4, `[KEPUTUSAN-81]`).
- `scripts/verify.sh` menerapkan semua migrasi ke PostgreSQL 18 kosong untuk ketiga database dan menguji hak role (role aplikasi tidak bisa `UPDATE`/`DELETE` tabel T/R/L; trigger penolak aktif; RLS `FORCE` di RME) `[KEPUTUSAN-82]`.

### 5.4 Otorisasi (T1)

- **Default deny** (ADR-012 butir 1): setiap endpoint menyatakan kebijakannya; permintaan tanpa kebijakan ditolak. Route hanya dapat didaftarkan lewat pembungkus router yang mewajibkan kebijakan, dan test saat start memeriksa bahwa setiap route terdaftar punya kebijakan `[KEPUTUSAN-127]`.
- Kebijakan dievaluasi di crate `authz`, **tidak tersebar di handler** (ADR-012 butir 1). Modul mendaftarkan kebijakannya di `policy.rs`.
- Relasi yang diperiksa: kepemilikan, penugasan (dalam batas waktunya), tenant, dan sub-peran admin (ADR-012 butir 1).
- **Validasi token** di backend: tanda tangan dengan daftar izin algoritma, issuer per realm, audience, masa berlaku (RFC 8725 `[SRC-79]`), dengan jsonwebtoken 12.0.0 `[SRC-169]`; JWKS realm di-cache `[KEPUTUSAN-74]`. Backend Ruang Sehat **menolak** token realm `rme`, dan backend RME hanya menerima realm `rme` (ADR-011 butir 3 dan 11). Backend juga memeriksa status akun dan daftar sesi yang dicabut; aksi berisiko membaca status langsung dari database (ADR-011 butir 8).
- **Cakupan tenant dari sesi, bukan dari permintaan** (ADR-012 butir 5, ADR-014 butir 3): tipe `PartnerScope` dan `FasyankesScope` hanya dapat dibangun oleh crate `authz` dari konteks autentikasi yang sudah divalidasi. Fungsi repository untuk data per tenant **wajib** menerima tipe ini, bukan `Uuid` mentah, sehingga lupa membatasi tenant menjadi galat compile `[KEPUTUSAN-128]`. `partner_id` di body atau query **diabaikan** untuk otorisasi.
- **RME**: selain pemeriksaan aplikasi, setiap transaksi memasang pengaturan sesi database `fasyankes_id` untuk RLS (ADR-012 butir 6). Mekanisme pengaturan lokal-transaksi diverifikasi ke dokumentasi PostgreSQL di Tahap 8. Jalur Operator RME memakai role database terpisah tanpa `SELECT` atas tabel klinis.
- **Respons penolakan**: objek terlarang → 404 `4040001`; aksi dikenal tetapi tidak diizinkan → 403 `4030001` (ADR-012 butir 4). Penolakan atas data sensitif dicatat dengan hasil gagal (ADR-012 butir 2) dan masuk security log.
- **WebSocket**: token, status akun, dan daftar pencabutan diperiksa saat koneksi dibuka; setiap langganan kanal diperiksa `authz`; koneksi diputus saat token kedaluwarsa atau sinyal penonaktifan diterima (ADR-016 butir 1). Pesan kanal didokumentasikan dengan AsyncAPI 3.1.0 `[SRC-147]` `[KEPUTUSAN-83]`.
- **Pemisahan tugas** ADR-012 butir 7 (admin tidak mengubah perannya sendiri; Super Admin dan Admin Fasyankes terakhir tidak dapat dinonaktifkan) diuji sebagai aturan domain.
- **Test penolakan wajib** untuk setiap endpoint baru (ADR-012 konsekuensi): lihat §7.3.

### 5.5 Enkripsi data spesifik (T1)

- **Satu pustaka kripto**: aws-lc-rs 1.19.0 untuk AES-256-GCM, HMAC-SHA256, SHA-256, dan RNG `[SRC-173]`; `aes-gcm` dan `ring` tidak dipakai `[KEPUTUSAN-74]`. Hanya crate `crypto` yang boleh bergantung langsung pada aws-lc-rs; kode lain memakai API crate `crypto` `[KEPUTUSAN-129]`.
- **Wadah DEK** (ADR-013 butir 3, `[KEPUTUSAN-68]`): setiap wadah data (misalnya `consultation_chat` dan `consultation_clinical` untuk satu sesi, satu hasil lab, satu dokumen identitas, `status_note`) punya DEK sendiri. DEK dibangkitkan lewat Transit `datakey` OpenBao dan disimpan terbungkus KEK di tabel `data_key` `[SRC-222]` `[KEPUTUSAN-79]`. Pustaka **wajib** memeriksa `container_type` saat membuka data, karena `consultation_chat` dan `consultation_clinical` berbagi `container_id` dan pembedanya tidak ditegakkan FK (data dictionary §0.2); pemeriksaan ini wajib dicakup test.
- **Format ciphertext**: setiap nilai `_enc` memuat versi format, nonce acak 96-bit dari RNG pustaka (tidak pernah dipakai ulang), dan ciphertext + tag. Data tambahan terautentikasi (AAD) mengikat ciphertext ke tabel, kolom, ID baris, dan wadahnya, sehingga ciphertext yang dipindah ke baris atau wadah lain gagal dibuka `[KEPUTUSAN-130]`. Dukungan AAD di API aws-lc-rs diverifikasi Tahap 8.
- **Blind index** NIK dan kode verifikasi resep: HMAC-SHA256 dengan kunci terpisah dari OpenBao KV, disimpan bersama versi kuncinya (ADR-013 butir 4, data dictionary §0.2) `[KEPUTUSAN-79]`.
- **Cache DEK** di memori proses dengan batas waktu (ADR-013 konsekuensi, `[KEPUTUSAN-79]`). Plaintext DEK dan data hasil dekripsi dibungkus `Sensitive<T>` dan **dilarang** ditulis ke log, cache Valkey, atau outbox.
- **Dilarang** mengimplementasikan primitif kriptografi sendiri, memakai fungsi kriptografi PostgreSQL untuk data aplikasi (ADR-013 alternatif ditolak), atau menghapus DEK dari kode selain pekerja privasi (ADR-013 butir 5).
- Crypto-shredding, anonimisasi, dan retensi hanya oleh role pekerja privasi dan dicatat E-16 (ADR-013 butir 5–6).

### 5.6 Integrasi keluar dan webhook (T1)

- Webhook dan panggilan sinkron keluar ditandatangani HMAC-SHA256 atas stempel waktu dan isi, dengan rahasia per rekanan; penerima menolak pesan lebih tua dari 5 menit (ADR-014 butir 3, `[SRC-46]`). Perbandingan tanda tangan memakai fungsi perbandingan waktu-konstan dari crate `crypto`.
- Pada penerima webhook (RME), identitas pengirim dan tenant diambil dari tanda tangan yang terverifikasi, bukan dari isi pesan (ADR-014 butir 4).
- Pemeriksaan sebelum bayar: batas waktu 5 detik tanpa percobaan ulang di dalam permintaan pasien (ADR-014 butir 5).
- Percobaan ulang dari outbox dengan jeda membesar 1, 2, 4, ... menit, maksimal 30 menit (ADR-014 butir 4).
- Setiap pengiriman data pribadi ke pihak luar dicatat E-17 (jenis data dan tujuan, tanpa isi) (ADR-013 butir 7).
- Klien HTTP keluar memakai reqwest 0.13.5 `[SRC-170]` dengan batas waktu eksplisit di setiap panggilan `[KEPUTUSAN-74]`.

### 5.7 Waktu

- Server berjalan dalam UTC; instan disimpan `timestamptz` `[SRC-41]`; API mengirim ISO 8601 dengan offset; konversi tampilan dilakukan di klien atau lapisan presentasi laporan (ADR-017 butir 1–2).
- Pustaka waktu Rust: time 0.3.55 `[SRC-177]`; chrono tidak dipakai `[KEPUTUSAN-74]`.
- **Jam yang dapat diatur**: waktu sekarang hanya diambil lewat trait `Clock` dari crate `foundation` yang diinjeksi ke service dan job. Pemanggilan jam sistem langsung hanya boleh di implementasi `SystemClock`; test memakai `FixedClock`/jam manual dari `test-support` `[KEPUTUSAN-131]`. Penegakannya (F1-T12) berlapis: `disallowed-methods` di `clippy.toml` root (`OffsetDateTime::now_utc`, `OffsetDateTime::now_local`, `UtcDateTime::now`, `SystemTime::now`) `[SRC-387]` `[SRC-389]`; test arsitektur menolak pemanggilan yang sama secara teks, fitur `local-offset` crate time, dan atribut `allow` baru. Satu-satunya pengecualian adalah `SystemClock::now` (`architecture-exceptions.toml`). `Instant::now` untuk mengukur durasi tetap boleh.
- Jam perangkat klien **tidak pernah** dipakai untuk aturan bisnis atau audit (ADR-017 butir 5, ADR-009 butir 6).
- Batas waktu PB dihitung sebagai instan absolut lalu dijadwalkan sebagai job; batas hari kerja memakai tabel kalender hari kerja dan berakhir 23.59 WIB pada hari kerja ke-N (ADR-017 butir 4).
- Test yang menyentuh waktu mencakup minimal dua zona waktu Indonesia (ADR-017 konsekuensi).

---

## 6. Konfigurasi dan secrets

Mengikuti `[KEPUTUSAN-88]`, ADR-013 butir 8, ADR-021 butir 4:

- **Dilarang** ada rahasia di kode, repositori, image, berkas Compose, variabel CI, atau log. Satu-satunya pengecualian adalah kunci privat cosign sebagai secret GitHub berakses minimum `[KEPUTUSAN-81]` `[KEPUTUSAN-88]`.
- **Lokal**: berkas `.env.local` berisi nilai **sintetis**, tidak di-commit (masuk `.gitignore`); OpenBao mode dev di Compose lokal `[KEPUTUSAN-88]`. `.env.example` di-commit berisi nama variabel dan nilai contoh sintetis saja `[DOK: .claude/skills/runbook/references/tahap-08-development.md]`.
- **Staging dan production**: semua rahasia di OpenBao 2.7.2 `[SRC-185]`; proses mengambil rahasia saat start dan saat rotasi. Setiap deployment punya identitas OpenBao sendiri dengan policy minimum (`api`, `realtime`, `worker`, `audit-relay`, `audit-verify`, `privacy-worker` untuk Ruang Sehat dan RME, rotasi kunci, BFF Next.js) `[KEPUTUSAN-79]` (§1.4). Kredensial database dinamis berumur pendek `[SRC-224]`.
- **Konfigurasi non-rahasia** lewat variabel lingkungan di berkas Compose per lingkungan di `infra/` `[KEPUTUSAN-88]`.
- **Konfigurasi bertipe dan gagal cepat**: setiap proses memuat konfigurasi ke satu struct bertipe saat start dan berhenti dengan galat yang jelas (tanpa mencetak nilai) bila ada yang hilang atau tidak valid; kode lain menerima konfigurasi lewat struct itu, tidak membaca variabel lingkungan langsung `[KEPUTUSAN-132]`. Larangan `std::env::var`, `var_os`, `vars`, `vars_os` berlaku untuk seluruh workspace (`disallowed-methods` di `clippy.toml` root, ditambah pemeriksaan teks di test arsitektur); satu-satunya pengecualian `EnvSource::from_process` (F1-T12). Field rahasia bertipe `Sensitive<T>`.
- **Keycloak**: konfigurasi realm hanya sebagai kode di `infra/` lewat keycloak-config-cli 6.5.1 `[SRC-122]` `[KEPUTUSAN-85]`; perubahan manual lewat konsol admin dilarang. Perubahan ini termasuk modul kritis T1.
- **Aplikasi mobile**: semua yang ada di paket aplikasi dianggap publik. **Dilarang** menaruh rahasia klien, kunci API, atau kredensial apa pun di kode atau aset aplikasi. Kunci penandatangan aplikasi disimpan di luar repositori dengan akses minimal yang tercatat (ADR-021 butir 4).
- gitleaks 8.30.1 memindai rahasia di setiap perubahan `[SRC-151]` `[KEPUTUSAN-82]`.

---

## 7. Konvensi test

### 7.1 Piramida dan alat

| Jenis | Cakupan | Alat | Sumber |
| --- | --- | --- | --- |
| Unit | Domain murni: transisi status, perhitungan harga dan markup, masking audit, format ciphertext, validasi | `cargo test` | `[KEPUTUSAN-74]` |
| Integrasi | Service + repository di PostgreSQL 18, Valkey, dan Keycloak sungguhan, dengan migrasi dan **role aplikasi** (bukan pemilik skema) sehingga hak tabel ikut teruji | testcontainers 0.28.0 | `[SRC-181]` `[KEPUTUSAN-74]` |
| Kontrak | OpenAPI hasil generate identik dengan `openapi/`; Schemathesis terhadap backend efemeral; `response_code` terdaftar dan konsisten | utoipa, Schemathesis 4.33.4 | `[SRC-168]` `[SRC-148]` `[KEPUTUSAN-82]` |
| Arsitektur | Batas modul (tabel per modul), pola logging terlarang, larangan `unwrap`, pemakai langsung aws-lc-rs, pemanggilan jam sistem | skrip di `scripts/verify.sh` | `[KEPUTUSAN-82]` |
| Web unit | Logika BFF (sesi, CSRF, penerusan), komponen | Pustaka dipilih Tahap 8 | `[KEPUTUSAN-75]` |
| E2E web | Alur utama per aplikasi web terhadap stack lokal | Playwright 1.65.1 | `[SRC-133]` |
| Mobile | Widget test per layar/komponen; integration test alur utama | `flutter test` (paket integration test diverifikasi Tahap 8) | `[KEPUTUSAN-82]` |

Semua jenis di atas dijalankan oleh `scripts/verify.sh` sesuai rancangan `[KEPUTUSAN-82]`. Test **dilarang** di-skip atau di-ignore untuk membuat verifikasi hijau; test yang belum dapat dijalankan dicatat sebagai task, tidak dibungkam.

### 7.2 Data uji

- **Hanya data sintetis**, di semua lingkungan non-production, termasuk contoh di spesifikasi OpenAPI dan Swagger UI staging (ADR-019 butir 1–2, ADR-022 butir 4). Data production tidak pernah disalin ke staging atau lokal.
- Pembangkit data sintetis ada di crate `test-support` (dan padanannya di web dan mobile). NIK sintetis dibuat oleh pembangkit itu dengan pola yang didokumentasikan di kodenya; **dilarang** menyalin NIK, nama, nomor HP, atau email orang nyata. Email uji memakai domain contoh yang dicadangkan untuk dokumentasi (RFC 2606, diverifikasi Tahap 8).
- Token uji dibuat dari Keycloak di testcontainers atau dari kunci uji di `test-support`, tidak pernah dari lingkungan nyata.

### 7.3 Skenario wajib

**Untuk setiap endpoint baru** (ADR-012 konsekuensi, Definition of Done):

1. Tanpa token → 401.
2. Token realm lain, audience salah, issuer salah, algoritma di luar daftar izin, kedaluwarsa → 401 (ADR-011 butir 8: test negatif untuk setiap kombinasi realm dan klien).
3. Akun nonaktif atau sesi dicabut dengan token yang masih berlaku → ditolak (KU-10).
4. Peran yang tidak diizinkan → 403.
5. Objek milik pengguna atau tenant lain lewat ganti ID → 404, dan percobaan atas data sensitif tercatat (KU-03).

**Isolasi tenant** (NFR-SEC-06, ADR-012 butir 5–6): untuk setiap endpoint portal rekanan dan RME, test membuat dua tenant A dan B, lalu memastikan pengguna A tidak dapat membaca, mengubah, atau melihat dalam daftar data milik B, termasuk dengan menyelipkan `partner_id`/`fasyankes_id` B di body atau query. Untuk RME juga diuji di tingkat database: role aplikasi dengan pengaturan sesi tenant A tidak mendapat baris B, dan tanpa pengaturan sesi tidak mendapat baris tenant mana pun.

**Skenario negatif modul kritis T1** (daftar modul di `CLAUDE.md`, `aturan-kerja.md` §1). Minimal:

| Modul T1 | Skenario negatif minimum |
| --- | --- |
| Autentikasi, sesi, MFA (Keycloak, validasi token, BFF) | Kombinasi token tidak sah di atas; token realm `rme` ke backend Ruang Sehat dan sebaliknya; autentikasi ulang wajib untuk aksi berisiko (ADR-011 butir 9); cookie tanpa CSRF ditolak; token tidak pernah muncul di respons browser; penguncian setelah 5 gagal (ADR-011 butir 4) |
| Otorisasi dan isolasi (RLS, role RME) | Endpoint tanpa kebijakan ditolak; isolasi tenant; admin mengubah perannya sendiri ditolak; Super Admin/Admin Fasyankes terakhir tidak dapat dinonaktifkan; Operator RME tidak dapat membaca tabel klinis |
| Harga, pembayaran, jurnal, refund, klaim, kewajiban | Notifikasi ganda hanya mengonfirmasi sekali; tanda tangan notifikasi tidak valid; nominal berbeda → RF-14; refund ganda ditolak; refund melebihi dana diterima ditolak; kegagalan dan habis waktu penyedia; jurnal tidak seimbang ditolak; luapan aritmetika uang (ADR-015 konsekuensi) |
| Pustaka audit, relay, DB audit | Field terlarang tidak pernah masuk event (US-1501 AC2); gagal tulis outbox membatalkan aksi; gagal E-05 menolak akses; `UPDATE`/`DELETE`/`TRUNCATE` di `audit_event` ditolak; perubahan satu event terdeteksi verifikasi hash berantai; dua relay tidak berjalan bersamaan |
| Enkripsi, kunci, pekerja privasi, retensi | `container_type` salah ditolak; ciphertext yang diubah atau dipindah baris gagal dibuka; DEK yang dimusnahkan tidak dapat dihidupkan kembali; role aplikasi tidak dapat menghapus DEK; retensi kosong tidak memproses apa pun (ADR-013 butir 6) |
| Persetujuan dan hak subjek data | Akun tidak terbuat tanpa persetujuan, termasuk kompensasi bila langkah Keycloak gagal (ADR-011 butir 6); pengajuan tanpa autentikasi ulang ditolak |
| Gateway Partner Integration API | Kredensial salah dicatat di security log; tanda tangan salah dan stempel waktu > 5 menit ditolak; kunci idempotensi dengan isi berbeda → `4090002`; `partner_id` di isi pesan diabaikan; satu bagian tidak valid menolak seluruh pesan; URL ke alamat privat/loopback/metadata ditolak; redirect tidak diikuti |
| Berkas dokumen identitas dan klinis | Jenis palsu (isi tidak cocok ekstensi/header) ditolak; melebihi batas ukuran; PDF berkonten aktif ditolak; metadata foto terbuang; URL unduh tidak terbit tanpa izin dan E-05/E-12 |
| Rekam medis RME dan SATUSEHAT | RLS lintas fasyankes; pengiriman ulang tidak menggandakan |
| Migrasi dan hak role | Role aplikasi tidak bisa `UPDATE`/`DELETE` tabel T/R/L; trigger penolak aktif; `FORCE ROW LEVEL SECURITY` aktif di RME |
| Akses administratif dan pipeline | gitleaks bersih; tidak ada kredensial production di konfigurasi CI (review) |

**Waktu**: memakai `Clock` yang dapat diatur dan minimal dua zona waktu Indonesia (ADR-017 konsekuensi).

**Real-time**: akun yang dinonaktifkan saat terhubung tidak menerima pesan berikutnya dan koneksinya terputus (ADR-016 butir 1).

### 7.4 Penamaan dan keterlacakan test

- Rust: fungsi test snake_case bahasa Inggris berpola `<unit>_<kondisi>_<hasil>`, misalnya `transition_with_stale_version_is_rejected`. Test integrasi di `tests/<modul>_<topik>.rs`.
- TypeScript: `describe('<unit>')` + `test('<kondisi> → <hasil>')`. Dart: `group` + `test`/`testWidgets` dengan pola yang sama.
- Setiap test yang membuktikan acceptance criteria diberi penanda `trace: US-xxx ACn` (komentar Rust/Dart/TS) supaya kolom test di `docs/traceability.md` dapat diisi dan diperiksa dengan pencarian teks `[KEPUTUSAN-133]`.

### 7.5 Cakupan minimum

Tidak ada angka cakupan standar industri yang diklaim di sini. Usulan penyusun `[KEPUTUSAN-134]`:

- Ukuran utama adalah **cakupan requirement**: setiap acceptance criteria punya minimal satu test bertanda `trace:`, setiap endpoint punya test penolakan §7.3, dan setiap modul T1 punya skenario negatif §7.3.
- Cakupan baris diukur dan dilaporkan di CI (alat diverifikasi Tahap 8). Angka minimum ditetapkan dari baseline reference implementation Tahap 8 Fase 1 dan **tidak boleh turun** setelahnya (ratchet).
- Usulan titik awal untuk crate T1 (`audit`, `crypto`, `authz`, `integration-contract`): **80% baris**. Angka ini **usulan penyusun**, bukan standar, dan dikonfirmasi user di gate Tahap 5 atau disesuaikan setelah baseline Fase 1.

---

## 8. Commit, branch, dan PR

Mengikuti runbook `[DOK: .claude/skills/runbook/SKILL.md]`, `aturan-kerja.md` §5, dan `tahap-08-development.md`:

- **Hanya commit hijau.** Git hook pre-commit dan pre-merge-commit menjalankan `scripts/verify.sh`. **Dilarang** `--no-verify`/`-n`, mengubah `core.hooksPath`, mengubah `.githooks/`, atau melonggarkan `scripts/verify.sh` `[DOK: CLAUDE.md]`. Kalau merah dan belum dapat diperbaiki: jangan commit, laporkan.
- **Satu commit per task Tahap 8**, setelah `scripts/verify.sh` hijau dan `code-reviewer` lolos; lalu push (`aturan-kerja.md` §5).
- **Format pesan commit** `[KEPUTUSAN-135]`, selaras dengan pola commit tahap yang sudah ada (`Tahap 5 (Stack): ...`):
  - Story: `Tahap 8 (EP-xx/US-xxx): <ringkasan dalam Bahasa Indonesia>`. Contoh: `Tahap 8 (EP-08/US-802): verifikasi notifikasi payment gateway dan konfirmasi sekali`.
  - Task non-story (kerangka, infrastruktur, tech debt): `Tahap 8 (Fase n/<ID task phase plan>): <ringkasan>`.
  - Perbaikan bug atau change request: `Tahap 8 (BUG-xxx): ...` / `Tahap 8 (CR-xxx): ...`, memakai nomor dari `docs/backlog.md`.
  - Badan commit memuat: apa yang berubah, ringkasan hasil `scripts/verify.sh` (perintah dan jumlah test lulus), status review `code-reviewer`, dan migrasi yang disertakan bila ada. Klaim di pesan commit wajib sesuai hasil verifikasi yang benar-benar dijalankan.
- **Branch** `[KEPUTUSAN-135]`: `tahap-08-fase-<n>` per fase, mengikuti pola `tahap-0x-<nama>` yang sudah dipakai. Commit dan push per task ke branch fase. Merge ke `main` minimal saat fase ditutup; merge lebih awal boleh untuk sekumpulan task yang perlu di-deploy ke staging (staging otomatis dari `main` hijau, `[KEPUTUSAN-81]`).
- **PR**: PR GitHub dari branch fase ke `main` dianjurkan sebagai catatan review. Gerbang review per task adalah subagent `code-reviewer`; perubahan modul T1 ditambah reviewer keamanan dan/atau DBA sesuai bidangnya `[DOK: CLAUDE.md]`. Proteksi branch `main` diatur user bila paket GitHub mendukung, dan bukan pengaman utama `[KEPUTUSAN-81]`.
- Status task di backlog hanya berubah dengan bukti: commit hash dan hasil `scripts/verify.sh` (anti-halu §5).
- **Ukuran**: satu task muat dalam satu PR yang dapat direview dalam 15 menit (Definition of Ready, `docs/backlog.md`).

### 8.1 Definition of Done (dari `docs/backlog.md`, ditambah rujukan dokumen ini)

- CI hijau: lint, type check, test, security scan.
- Mengikuti pola reference implementation dan dokumen ini.
- Lolos review `code-reviewer`.
- Audit trail tercatat untuk aksi sensitif (§4.4).
- File migrasi disertakan kalau ada perubahan skema (§5.3).
- Acceptance criteria terverifikasi, dengan penanda `trace:` (§7.4).
- Dependensi mobile baru sudah direview perilaku jaringannya; build rilis lulus uji lalu lintas jaringan (ADR-021 butir 2).
- Migrasi yang menyentuh DB audit, role, grant, trigger penolak, atau RLS sudah direview DBA dan keamanan (ADR-021 butir 4).
- Spesifikasi OpenAPI diperbarui untuk endpoint baru atau berubah, dan contract test lulus (ADR-022).
- Butir yang ditegaskan dokumen ini: endpoint baru punya kebijakan dan test penolakan (§5.4, §7.3); `response_code` baru terdaftar di `docs/05-response-codes.md` (§3.1); dependensi baru tercatat di `docs/sources.md` (§9).

---

## 9. Dependensi

### 9.1 Menambah dependensi

1. **Periksa registry resmi** dengan nama yang persis: crates.io (Rust), npm (TypeScript), pub.dev (Dart). Periksa maintainer, tanggal rilis terakhir, lisensi, dan riwayat advisory (RustSec, GitHub Advisory Database, OSV) `[DOK: .claude/skills/runbook/references/anti-halu.md]` §5. Nama paket yang dikarang dapat didaftarkan penyerang.
2. **Catat** di `docs/sources.md` (URL registry, tanggal akses, versi, lisensi) sebelum dipakai.
3. **Lisensi**: daftar izin awal adalah lisensi pustaka yang sudah dipilih di stack Tahap 5: MIT, Apache-2.0, BSD-3-Clause, ISC, dan kombinasinya (misalnya `MIT OR Apache-2.0`). Lisensi di luar daftar ini, atau copyleft yang ditautkan ke kode aplikasi (GPL, LGPL, AGPL, SSPL), **dibawa ke user** sebelum dipakai karena menyangkut keputusan legal `[KEPUTUSAN-136]`. Daftar izin ditegakkan cargo-deny 0.21.0 untuk Rust `[SRC-183]`; untuk npm dan pub mekanismenya diverifikasi Tahap 8.
4. **Kunci versi**: lockfile di-commit (`Cargo.lock`, `pnpm-lock.yaml`, `pubspec.lock` untuk aplikasi). Cargo dijalankan dengan `--locked` dan pnpm dengan `--frozen-lockfile` di verifikasi `[DOK: .claude/skills/runbook/references/tahap-08-development.md]` `[KEPUTUSAN-82]`. Next.js dan React dipin versi persis di manifest `[KEPUTUSAN-75]`; hal yang sama berlaku untuk pustaka autentikasi dan kriptografi (jsonwebtoken, aws-lc-rs, openid-client, jose, flutter_appauth, flutter_secure_storage) `[KEPUTUSAN-137]`. Integritas diperiksa dengan hash lockfile (ADR-021 butir 4).
5. **Pindai**: cargo-deny dan cargo-audit 0.23.0 `[SRC-183]` `[SRC-184]`, osv-scanner 2.7.1 `[SRC-152]`, Trivy 0.76.2 untuk image `[SRC-153]` di CI `[KEPUTUSAN-81]`.
6. **Mobile**: setiap dependensi atau plugin mobile baru **wajib** direview perilaku jaringannya dan data yang dikumpulkannya sebelum dipakai, dan inventaris dependensi kedua aplikasi diperbarui (ADR-021 butir 2). SDK analitik atau crash pihak ketiga yang mengirim data ke luar Indonesia **dilarang** (ADR-010 butir 4).
7. **Hindari dependensi baru** bila fungsinya sudah ada di stack atau cukup beberapa baris kode. Dependensi baru yang menyentuh modul T1 direview keamanan.

### 9.2 SLA patch advisory

- Next.js: patch keamanan Critical/High dipasang ≤ 7 hari `[KEPUTUSAN-75]`.
- Usulan perluasan untuk semua dependensi di jalur production (Rust, npm, pub, image dasar, komponen infrastruktur) `[KEPUTUSAN-138]`: Critical/High ≤ 7 hari; Moderate pada rilis terjadwal berikutnya, paling lambat 30 hari; Low dikumpulkan dalam pembaruan rutin. Angka 30 hari adalah usulan penyusun yang dikonfirmasi user di gate Tahap 5. Advisory tanpa patch dicatat di risk register beserta mitigasinya.
- Pustaka yang dipantau karena jarang rilis atau masih 0.x (openapi-fetch, web_socket_channel, keycloak-config-cli) ditinjau setiap fase `[KEPUTUSAN-75]` `[KEPUTUSAN-76]` `[KEPUTUSAN-85]`.
- Upgrade toolchain Rust dievaluasi per 6 minggu, tidak otomatis `[KEPUTUSAN-74]`.

---

## 10. Checklist code review

Dipakai `code-reviewer` bersama Definition of Done. Temuan wajib menyebut berkas, baris, dan cara mereproduksi (anti-halu §6).

**Keamanan**
- [ ] Endpoint baru punya kebijakan di `policy.rs`; tidak ada route tanpa kebijakan (§5.4).
- [ ] Cakupan tenant dari sesi lewat `PartnerScope`/`FasyankesScope`, bukan dari input (§5.4).
- [ ] Objek terlarang dijawab 404; tidak ada perbedaan "tidak ada" dan "tidak boleh" (§3.4).
- [ ] Tidak ada rahasia di kode, konfigurasi, test, atau log (§6); variabel `NEXT_PUBLIC_` tidak memuat rahasia.
- [ ] Tidak ada logika otorisasi data di Next.js atau middleware Next.js (§1.5).
- [ ] Kriptografi hanya lewat crate `crypto`; `container_type` diperiksa (§5.5).
- [ ] Input divalidasi di tepi dan di service; berkas divalidasi dari isinya (§5.1).
- [ ] Aplikasi mobile: tidak ada isi klinis di penyimpanan persisten; tidak ada host baru di luar daftar izin (§1.6).

**Data**
- [ ] Perubahan skema lewat migrasi baru; migrasi lama tidak diubah (§5.3).
- [ ] Tabel baru punya kelas ADR-008, hak role, trigger penolak (T/R/L), partial unique (Master), RLS `FORCE` (tenant RME).
- [ ] Tidak ada `UPDATE`/`DELETE` atas tabel T/R/L; status lewat jalur tulis ADR-008 butir 2.
- [ ] Hak per tabel ada di migrasi; tidak ada `DISABLE TRIGGER`/`DROP TRIGGER tg_*_reject_*`; `db/roles/` tanpa password (§5.3).
- [ ] Query master memfilter `deleted_at IS NULL`; tidak ada `SELECT *`; tidak ada SQL dari penggabungan string.
- [ ] Modul tidak menyentuh tabel modul lain; `module-tables.toml` diperbarui (§1.3).
- [ ] Satu transaksi per use case; tidak ada panggilan HTTP pihak ketiga selama transaksi terbuka (§5.2).
- [ ] Uang `Idr`/`_idr` bilangan bulat dengan aritmetika terperiksa; waktu UTC lewat `Clock` (§3.4, §5.7).
- [ ] Penamaan sesuai §2.

**Audit dan log**
- [ ] Aksi sensitif mencatat event yang benar (E-01–E-17) di transaksi yang sama (§4.4).
- [ ] E-05 di-commit sebelum data sensitif dikirim; penolakan dicatat hasil gagal.
- [ ] Tidak ada field terlarang ADR-005/PRD §7.3 di audit, log, atau security log; tidak ada objek domain utuh di log (§4.1).
- [ ] `correlation_id` diteruskan ke job, outbox, dan panggilan keluar (§4.2).

**API**
- [ ] Envelope ADR-023; `response_code` terdaftar di `docs/05-response-codes.md` dan 3 digit pertamanya sama dengan HTTP (§3).
- [ ] Spesifikasi OpenAPI di `openapi/` diperbarui dan identik dengan hasil generate (ADR-022).
- [ ] Field snake_case bahasa Inggris; waktu ISO 8601 dengan offset; path sesuai §2.5.
- [ ] Tidak ada stack trace, query, atau data pribadi di respons galat.
- [ ] Endpoint yang mengubah data dari rekanan memakai `Idempotency-Key` (§5.2).

**Test**
- [ ] Setiap acceptance criteria punya test bertanda `trace:` (§7.4).
- [ ] Test penolakan endpoint dan isolasi tenant ada (§7.3).
- [ ] Perubahan modul T1 punya skenario negatif §7.3.
- [ ] Data uji sintetis; waktu memakai `Clock` yang dapat diatur dengan dua zona waktu.
- [ ] `scripts/verify.sh` dijalankan reviewer sendiri dan hijau; tidak ada test yang di-skip untuk menghijaukan.

**Dependensi dan anti-halu**
- [ ] Setiap fungsi atau API pustaka yang dipanggil benar-benar ada di versi terpasang.
- [ ] Dependensi baru tercatat di `docs/sources.md` dan lockfile; lisensi dalam daftar izin; dependensi mobile sudah direview perilaku jaringannya (§9).
- [ ] Tidak ada `unwrap`/`expect`/`panic!` di kode non-test tanpa pengecualian beralasan (§3.4); tidak ada kode mati atau TODO yang tertinggal.

---

## 11. Hal yang ditetapkan di Tahap 8 Fase 1

Dokumen ini sengaja tidak mengunci hal berikut; ditetapkan saat reference implementation dibangun dan dicatat kembali di dokumen ini:

- API final crate `audit`, `authz`, `crypto`, `envelope`, `foundation`, dan pustaka jalur tulis status.
- Ditetapkan di F1-T12: lint larangan, `disallowed-methods`, dan `module-tables.toml` (§1.3, §3.4, §5.7, §6) dengan pengecualian di `architecture-exceptions.toml`; `ResponseCode::new` hanya di crate `envelope` dan `error.rs` modul (§3.2).
- Generator klien API Dart `[KEPUTUSAN-76]`; pustaka test unit web dan konfigurasi ESLint `[KEPUTUSAN-75]`.
- Pustaka basis data zona waktu IANA untuk Rust `[TIDAK DIKETAHUI]`.
- Pustaka logger server Next.js.
- Agen pengirim log dan perutean `log_stream` ke tenant Loki `[KEPUTUSAN-80]`.
- Alat pengukur cakupan dan angka baseline (§7.5).
- Metode uji lalu lintas jaringan build rilis mobile (ADR-021 butir 2) bila belum ditetapkan di `docs/05-stack.md`.

---

## 12. Keputusan baru dari dokumen ini

Untuk dicatat orkestrator di `docs/decision-log.md`. Semua adalah keputusan teknis penyusun di antara opsi yang valid; tidak ada fakta bisnis yang diputuskan di sini. Butir yang menyebut angka (KEPUTUSAN-134, KEPUTUSAN-138) adalah usulan yang dikonfirmasi user di gate.

| Nomor | Keputusan | Alasan |
| --- | --- | --- |
| KEPUTUSAN-110 | Crate bersama tambahan `foundation` (`Clock`, `new_id()` UUIDv7, `Idr`, `CorrelationId`, `Sensitive<T>`), melengkapi daftar crate KEPUTUSAN-89 | Tipe dasar ini dipakai semua crate lain; menaruhnya di `envelope` atau `crypto` mencampur tanggung jawab |
| KEPUTUSAN-111 | Nama modul Rust bahasa Inggris snake_case dengan pemetaan tetap ke modul arsitektur §3 dan kode modul `response_code` | Konsisten dengan KEPUTUSAN-39 (identifier bahasa Inggris) dan KEPUTUSAN-84 |
| KEPUTUSAN-112 | Struktur internal modul tetap (`api`, `http`, `service`, `repo`, `domain`, `error`, `policy`, `jobs`); hanya `api` yang `pub(crate)` | Batas modul ADR-006 ditegakkan compiler, bukan hanya review; struktur seragam memudahkan review dan consistency review |
| KEPUTUSAN-113 | Kepemilikan tabel dicatat di `module-tables.toml` per backend dan diperiksa test arsitektur | Test arsitektur KEPUTUSAN-74/-82 butuh sumber kebenaran kepemilikan yang dapat dibaca mesin |
| KEPUTUSAN-114 | Fungsi `api` modul yang menulis menerima transaksi milik pemanggil; repository tidak membuka transaksi sendiri | Satu use case lintas modul tetap satu transaksi, syarat outbox audit atomik ADR-009 butir 2 |
| KEPUTUSAN-115 | Next.js: mutasi hanya lewat Route Handler BFF, tanpa Server Actions/Functions | Satu jalur untuk CSRF, korelasi, dan penerusan token; mengurangi permukaan serangan di komponen T1 |
| KEPUTUSAN-116 | Paket Flutter bersama berawalan `rs_` dengan pembagian `rs_api_client`, `rs_auth`, `rs_core`, `rs_realtime`, `rs_design`; fitur dibagi `data`/`domain`/`presentation` | Memisahkan kode T1 (`rs_auth`) dan memudahkan berbagi antar dua aplikasi (ADR-006 butir 3) |
| KEPUTUSAN-117 | Awalan nama constraint dan index: `pk_`, `fk_`, `uq_`, `uqp_`, `ix_`, `ck_`, `tg_` + `<tabel>_<kolom/arti>` | Data dictionary §0.2 menyerahkan format ke Tahap 5; nama yang dapat ditebak memudahkan pembacaan galat constraint dan review migrasi. Diselaraskan dengan `docs/05-physical-schema.md` |
| KEPUTUSAN-118 | Newtype untuk ID dan uang (`Idr` dengan aritmetika terperiksa) | Mencegah ID atau nominal tertukar dan pembulatan/luapan diam-diam di modul T1 harga dan pembayaran |
| KEPUTUSAN-119 | Field data API di TypeScript dipakai snake_case apa adanya dari tipe hasil generate; Dart mengikuti generator dengan kunci JSON snake_case | Lapisan pemetaan nama menambah kode dan peluang salah tanpa manfaat keamanan |
| KEPUTUSAN-120 | Awalan path per audiens (`/patient/v1`, `/practitioner/v1`, `/backoffice/v1`, `/portal/v1`, `/partner/v1`, `/webhook/<penyedia>`, `/rme/v1`), segmen kebab-case jamak, parameter snake_case | Selaras dengan berkas OpenAPI KEPUTUSAN-83 dan namespace terpisah ADR-006 butir 5; memudahkan aturan proxy dan batas laju per audiens |
| KEPUTUSAN-121 | `unwrap`/`expect`/`panic!`/`todo!`/`unimplemented!`/indeks yang dapat panik dilarang di kode non-test; pengecualian hanya dengan `allow` lokal beralasan | Panik di proses `api` atau `worker` memutus permintaan lain dan menyulitkan pemetaan galat ke envelope; melengkapi KEPUTUSAN-82 |
| KEPUTUSAN-122 | Pembungkus `Sensitive<T>` dengan `Debug`/`Display` tersamar dan tanpa serialisasi umum untuk nilai rahasia dan data pribadi | Redaksi ADR-010 butir 1 ditegakkan di tingkat tipe, tidak hanya oleh daftar tolak logger |
| KEPUTUSAN-123 | Header `X-Correlation-Id`, diterima hanya bila UUID valid, selain itu dibuat UUIDv7 baru; diteruskan ke job, outbox, event audit, dan panggilan keluar | Menghubungkan tiga log (ADR-010 butir 2) tanpa membuka celah penyisipan teks ke log |
| KEPUTUSAN-124 | Security log dipancarkan lewat fungsi khusus dengan field `log_stream = "security"` yang dirutekan agen ke tenant Loki `security` | Pemisahan aliran ADR-010 dengan satu mekanisme logging; perutean final di Tahap 8 Fase 1 (KEPUTUSAN-80) |
| KEPUTUSAN-125 | Tidak ada panggilan HTTP ke pihak ketiga selama transaksi database terbuka | Mencegah kunci baris tertahan saat pihak ketiga lambat dan efek eksternal tanpa catatan bila transaksi batal; sejalan dengan ADR-015 butir 5 |
| KEPUTUSAN-126 | Kunci idempotensi dikirim lewat header `Idempotency-Key`; disimpan bersama hash SHA-256 isi permintaan dan respons | ADR-014 butir 4 tidak menetapkan nama dan cara pembandingan; hash memungkinkan kode `4090002` tanpa menyimpan isi pesan |
| KEPUTUSAN-127 | Route hanya didaftarkan lewat pembungkus router yang mewajibkan kebijakan, ditambah test saat start | Menegakkan default deny ADR-012 butir 1 secara mekanis |
| KEPUTUSAN-128 | Tipe `PartnerScope`/`FasyankesScope` hanya dibangun `authz` dari konteks autentikasi dan wajib menjadi parameter repository data tenant | ADR-012 butir 5 ("`partner_id` dari sesi, bukan dari parameter") ditegakkan compiler |
| KEPUTUSAN-129 | Hanya crate `crypto` yang bergantung langsung pada aws-lc-rs | Satu titik review untuk kode kriptografi T1; melengkapi "satu pustaka kripto" KEPUTUSAN-74 |
| KEPUTUSAN-130 | Format ciphertext berversi dengan nonce acak 96-bit dan AAD yang mengikat tabel, kolom, ID baris, dan wadah | Mencegah ciphertext dipindah ke baris atau wadah lain tanpa terdeteksi; versi format memungkinkan perubahan algoritma kelak |
| KEPUTUSAN-131 | Waktu sekarang hanya lewat trait `Clock` yang diinjeksi; jam sistem langsung hanya di `SystemClock` | ADR-017 konsekuensi mewajibkan test dengan jam yang dapat diatur |
| KEPUTUSAN-132 | Konfigurasi dimuat ke satu struct bertipe saat start, gagal cepat tanpa mencetak nilai; kode lain tidak membaca variabel lingkungan langsung | Galat konfigurasi terlihat saat deploy, bukan saat permintaan; rahasia tidak bocor lewat pesan galat |
| KEPUTUSAN-133 | Penanda keterlacakan `trace: US-xxx ACn` di test | Mengisi dan memeriksa kolom test di `docs/traceability.md` secara mekanis |
| KEPUTUSAN-134 | Cakupan diukur terutama per requirement; cakupan baris dengan ratchet dari baseline Fase 1; usulan awal 80% baris untuk crate T1 | Tidak ada angka standar industri yang dapat dirujuk; ratchet mencegah penurunan tanpa mengarang target. Angka 80% usulan, dikonfirmasi user |
| KEPUTUSAN-135 | Format commit `Tahap 8 (EP-xx/US-xxx): ...` (dan varian Fase/BUG/CR); branch `tahap-08-fase-<n>`, commit dan push per task, merge ke `main` minimal saat fase ditutup | Selaras dengan pola commit dan branch yang sudah dipakai proyek dan aturan commit per task `aturan-kerja.md` §5 |
| KEPUTUSAN-136 | Daftar izin lisensi awal MIT, Apache-2.0, BSD-3-Clause, ISC (dan kombinasinya); lisensi lain atau copyleft yang ditautkan dibawa ke user | Lisensi baru adalah keputusan legal/bisnis; daftar awal diambil dari lisensi stack yang sudah dipilih |
| KEPUTUSAN-137 | Pustaka autentikasi dan kriptografi dipin versi persis di manifest, seperti Next.js/React | Perubahan versi di modul T1 harus disengaja dan direview, bukan ikut rentang semver |
| KEPUTUSAN-138 | SLA patch untuk semua dependensi jalur production: Critical/High ≤ 7 hari, Moderate ≤ 30 hari, Low di pembaruan rutin | Memperluas SLA Next.js KEPUTUSAN-75 ke seluruh stack; angka Moderate usulan, dikonfirmasi user |

---

## 13. Fakta eksternal yang diverifikasi Tahap 8

Belum terdaftar di `docs/sources.md`; dipakai di dokumen ini hanya sebagai aturan yang harus dibuktikan, bukan sebagai fakta:

| Butir | Dipakai di |
| --- | --- |
| Aturan visibilitas modul Rust (submodul privat tidak terlihat dari modul saudara) | §1.3 |
| Nama dan konfigurasi lint clippy untuk `unwrap`/`expect` dan metode terlarang | §3.4, §5.7 |
| Atribut serde untuk menolak field tak dikenal | §5.1 |
| Dukungan AAD pada API AES-GCM aws-lc-rs | §5.5 |
| Mekanisme pengaturan sesi PostgreSQL lokal-transaksi untuk RLS | §5.4 |
| Format nama berkas migrasi sqlx-cli | §5.3 |
| Opsi pengambilan data tanpa cache di Next.js; perilaku variabel `NEXT_PUBLIC_` | §1.5 |
| Panduan gaya Rust API Guidelines dan Effective Dart | §2.2, §2.4 |
| Paket integration test Flutter | §7.1 |
| Domain contoh yang dicadangkan (RFC 2606) | §7.2 |
| Pustaka basis data zona waktu IANA untuk Rust | §5.1 |
| Mekanisme daftar izin lisensi untuk npm dan pub | §9.1 |
