# Checklist ASVS 5.0.0 Level 2: Ruang Sehat

Tanggal: 2026-11-18
Versi: 1.0, disetujui di gate Tahap 5, 2026-11-18 `[USER] T5-G4`
Versi 1.1 (2026-11-18, tindak lanjut Checkpoint 2): V6 dan V7 diperbarui untuk MFA wajib pasien, atur ulang password ber-MFA, pemulihan perangkat hilang, ubah password saat login (CR-017), opsi B UP-73 (M-12), layar sesi aktif (US-110), dan kebijakan sesi paralel `[USER] C2-Q9, C2-Q10, C2-Q11, C2-B4, C2-B14`. Butir V6/V7 yang disebut per nomor requirement merujuk teks resmi yang sudah dibuka `[SRC-265]` `[SRC-266]` `[SRC-276]` `[SRC-277]`; batas dokumen di bawah tetap berlaku untuk bab lain
Penyusun: Claude (persona Security Architect)
Tingkat ketelitian: **T1 Kritis** (keamanan, autentikasi/otorisasi, data pribadi spesifik, kunci, pipeline; `aturan-kerja.md` §1)
Dasar: ADR-011 butir 1 (target ASVS 5.0.0 Level 2; "Pemetaan kontrol disusun di Tahap 5 sebagai checklist dan diverifikasi di Tahap 10") `[KEPUTUSAN-26]`; arsitektur §7 dan §11 baris "Checklist ASVS L2" `[DOK: 04-arsitektur.md]`; threat model v1.0 §6.2–§6.3 `[DOK: 04-threat-model.md]`; ADR-009, -010, -012, -013, -014, -016, -018, -021, -023 `[DOK: adr/]`; keputusan Tahap 5 KEPUTUSAN-74–KEPUTUSAN-89 `[DOK: decision-log.md]`; jawaban user T5-Q1–T5-Q8 `[DOK: 05-catatan-stack.md]`.

> **Batas dokumen ini.** Pemetaan dilakukan **per bab** ASVS 5.0.0 (V1–V17), memakai judul bab yang terverifikasi `[SRC-245]`. Teks requirement individual ASVS 5.0.0 **belum dibuka dan belum didaftarkan** di `docs/sources.md`, sehingga dokumen ini tidak mengutip, menomori, atau menyimpulkan isi requirement mana pun. Ruang lingkup tiap bab di bawah adalah **tafsiran penyusun atas judul bab**, bukan isi bab. **Pemetaan per requirement dilakukan di Tahap 8 (Fase 1) setelah teks ASVS 5.0.0 dibuka dan didaftarkan sebagai SRC, lalu diverifikasi di Tahap 10** (usulan U-01, §5). Nilai yang bergantung pada teks requirement (misalnya panjang minimum password, versi TLS minimum) ditandai "Tertunda Tahap 8".

---

## 0. Cara membaca

### 0.1 Label sumber

Mengikuti `anti-halu.md` §2 dan konvensi threat model §0.1:

| Penanda | Arti |
| --- | --- |
| `ADR-xxx butir n` | Butir keputusan ADR yang sudah diterima, setara `[DOK: adr/ADR-xxx-....md]` |
| `KEPUTUSAN-xx` | Keputusan teknis terdaftar di `docs/decision-log.md`. KEPUTUSAN-74–89 berstatus "menunggu gate Tahap 5" |
| `NFR-`, `US-`, `KU-`, `AB-`, `PT-`, `L-` | Butir PRD/BRD, setara `[DOK: 03-prd.md]` |
| `APP-`, `API-`, `KC-`, …, `M-xx`, `H-xx` | Ancaman dan mitigasi threat model, setara `[DOK: 04-threat-model.md]` |
| `[USER] T5-Qx` | Jawaban user Tahap 5 (`docs/05-catatan-stack.md`) |
| `CR-012` | Perubahan hosting ke VPS (`[USER] T5-Q2`), diajukan sebagai CR di Tahap 5 |
| `[SRC-xx]` | Fakta eksternal terverifikasi di `docs/sources.md`. Dokumen ini tidak menambah SRC baru |
| `U-xx` | **Usulan penyusun** dokumen ini (§5). Bukan keputusan; menjadi keputusan bila orkestrator mencatatnya di decision-log dan disetujui di gate |
| `N-xx` | Celah atau risiko keamanan baru (§4), untuk dimasukkan ke risk register oleh orkestrator |
| `[TIDAK DIKETAHUI]` | Belum ada sumber terverifikasi; disertai rencana verifikasi |

### 0.2 Status

| Status | Arti |
| --- | --- |
| **Dirancang** | Kontrol tertulis di ADR/KEPUTUSAN dengan sumber; tinggal dibangun dan dibuktikan di Tahap 8, diaudit di Tahap 10 |
| **Tertunda Tahap 8** | Mekanisme, nilai, atau pustaka belum terverifikasi atau belum dipilih; atau nilainya bergantung pada teks requirement yang belum dibuka |
| **Tidak berlaku** | Tidak ada komponen di stack yang menjadi sasaran kontrol; alasan ditulis |

Butir yang memerlukan keputusan user atau change request ditulis di kolom "Celah" dan di §6.

### 0.3 Kode cara verifikasi

| Kode | Cara | Sumber |
| --- | --- | --- |
| VS-n | Test otomatis di `scripts/verify.sh` langkah n (1 Rust, 2 migrasi dan hak role, 3 kontrak API, 4 test arsitektur, 5 web, 6 mobile, 7 rahasia) | KEPUTUSAN-82 |
| CI | Pemindaian di GitHub Actions (gitleaks, osv-scanner, Trivy, cargo-deny, cargo-audit) dan verifikasi tanda tangan image | KEPUTUSAN-81 `[SRC-151]` `[SRC-152]` `[SRC-153]` `[SRC-154]` `[SRC-183]` `[SRC-184]` |
| RK | Review kode dengan checklist (code-reviewer; ditambah security/DBA untuk modul T1) | `CLAUDE.md` modul kritis; `aturan-kerja.md` §1 |
| KF | Pemeriksaan konfigurasi sebagai kode di `infra/` (realm Keycloak, Caddyfile, Compose, kebijakan OpenBao, konfigurasi LiveKit, Loki) | KEPUTUSAN-78, -85, -89 |
| T10 | Audit dan uji keamanan Tahap 10 terhadap teks requirement ASVS | ADR-011 butir 1 |
| T12 | Syarat rilis Tahap 12 (misalnya uji lalu lintas jaringan build mobile) | ADR-021 butir 2 |

### 0.4 Komponen penanggung jawab (nama pendek)

| Nama | Isi | Sumber |
| --- | --- | --- |
| `rs-backend` | Binary Rust Ruang Sehat: `api`, `realtime`, `worker`, `audit-relay` | KEPUTUSAN-74 |
| `rme-backend` | Binary Rust RME: `api`, `worker` | KEPUTUSAN-74 |
| crate bersama | `audit`, `crypto`, `authz`, `envelope`, `platform-jobs`, `integration-contract` | KEPUTUSAN-74, -89 |
| BFF Next.js | Server tiga aplikasi Next.js (backoffice, portal, web RME), modul T1 autentikasi/sesi | KEPUTUSAN-75 |
| Flutter | Aplikasi pasien dan aplikasi nakes | KEPUTUSAN-76 |
| Keycloak | Keycloak 26.8.2, konfigurasi realm sebagai kode (keycloak-config-cli) | KEPUTUSAN-85 `[SRC-121]` `[SRC-122]` |
| OpenBao | OpenBao 2.7.2: Transit, KV, DB secrets, SSH CA, audit device | KEPUTUSAN-79 `[SRC-185]` |
| Caddy | Reverse proxy dan terminasi TLS di host app | KEPUTUSAN-78 `[SRC-189]` `[SRC-237]` |
| PG | PostgreSQL 18.7: instans (a) aplikasi RS + `keycloak`, (b) audit, (c) RME | KEPUTUSAN-77 `[SRC-118]` |
| Valkey | Valkey 9.1.3, instans RS dan instans RME terpisah | KEPUTUSAN-77 `[SRC-124]` |
| NOS | NEO Object Storage Biznet Gio (bucket per sensitivitas, Object Lock) | KEPUTUSAN-78 `[SRC-97]` `[SRC-99]` |
| LiveKit | LiveKit server 1.13.11 + TURN bawaan, tanpa Egress | KEPUTUSAN-87 `[SRC-126]` |
| Ops | Prometheus, Grafana, Loki, Alertmanager, GlitchTip di host `ops` | KEPUTUSAN-80 |
| Pipeline | GitHub Actions, GHCR, runner staging, skrip deploy production di bastion | KEPUTUSAN-81 |
| Host | VPS Biznet Gio dan Hostinger: OS, firewall, Docker, bastion, WireGuard, NTP | KEPUTUSAN-78 |

---

## 1. Ringkasan per bab

| Bab `[SRC-245]` | Komponen utama | Status ringkas | Celah terbuka |
| --- | --- | --- | --- |
| V1 Encoding and Sanitization | `rs-backend`, BFF Next.js, crate `envelope` | Dirancang sebagian; pustaka sanitasi CMS dan penetral CSV Tertunda Tahap 8 | 3 |
| V2 Validation and Business Logic | `rs-backend`, `rme-backend`, PG | Dirancang; batas anti-otomasi bisnis butuh keputusan user (M-11) | 3 |
| V3 Web Frontend Security | BFF Next.js, tema Keycloak, halaman verifikasi resep | Dirancang sebagian; header keamanan Tertunda Tahap 8 | 3 |
| V4 API and Web Service | `rs-backend`, `rme-backend`, `realtime`, Caddy | Dirancang | 3 |
| V5 File Handling | `rs-backend`, NOS, qpdf | Dirancang sebagian; kemampuan NOS dan antivirus Tertunda/[TIDAK DIKETAHUI] | 5 |
| V6 Authentication | Keycloak, OpenBao (manusia), bastion | Dirancang; MFA wajib semua realm pengguna termasuk `pasien` (v1.1); mekanisme kode pemulihan dan Reset Credentials ber-OTP diverifikasi Tahap 8 | 5 |
| V7 Session Management | BFF Next.js, Valkey, Keycloak, `rs-backend`, Flutter | Dirancang; layar sesi aktif Fase 2 dan sesi paralel terdokumentasi (v1.1); opsi reuse refresh token dan parameter autentikasi ulang Tertunda Tahap 8 | 3 |
| V8 Authorization | crate `authz`, PG (role, RLS), Keycloak admin client, kebijakan OpenBao | Dirancang | 4 |
| V9 Self-contained Tokens | `rs-backend`, `rme-backend` (jsonwebtoken), BFF (JWE) | Dirancang | 1 |
| V10 OAuth and OIDC | Keycloak, BFF, Flutter, OpenBao JWT auth | Dirancang | 3 |
| V11 Cryptography | crate `crypto` (aws-lc-rs), OpenBao Transit, pgBackRest | Dirancang; dua butir pilihan algoritma perlu dicek ke teks V11 | 4 |
| V12 Secure Communication | Caddy, LB NEO, PG/Valkey/OpenBao TLS, WireGuard, NOS | Sebagian besar Tertunda Tahap 8 | 6 |
| V13 Configuration | `infra/`, Pipeline, Host, Keycloak, Next.js | Dirancang sebagian; pengerasan host dan kontainer Tertunda Tahap 8 | 5 |
| V14 Data Protection | crate `crypto`, OpenBao, PG, NOS, Flutter | Dirancang sebagian; enkripsi lapis penyedia VPS/NOS [TIDAK DIKETAHUI] | 5 |
| V15 Secure Coding and Architecture | Monorepo, crate bersama, Pipeline | Dirancang | 3 |
| V16 Security Logging and Error Handling | crate `audit`, Ops (Loki), OpenBao audit, Keycloak event, crate `envelope` | Dirancang; NTP dan perangkat login observability Tertunda Tahap 8 | 5 |
| V17 WebRTC | LiveKit, `rs-backend` (token), Flutter (livekit_client) | Dirancang sebagian; batas peserta dan port publik Tertunda Tahap 8 | 4 |

Tidak ada bab yang seluruhnya "Tidak berlaku": setiap bab punya sasaran di stack ini. Kontrol untuk teknologi yang tidak dipakai (misalnya bila teks bab memuat kontrol GraphQL, XML/SOAP, atau LDAP) akan berstatus Tidak berlaku saat pemetaan per requirement, karena stack hanya memakai REST/JSON dan WebSocket (ADR-014 butir 1, ADR-016 butir 1, KEPUTUSAN-74).

---

## 2. Pemetaan per bab

### V1 Encoding and Sanitization

**Ruang lingkup di sistem ini** (tafsiran judul bab): pembentukan query SQL, pengodean keluaran HTML di tiga web Next.js dan tema login Keycloak, konten CMS dan data rekanan yang tampil di web/aplikasi (WEB-05), ekspor CSV (WEB-10), templat email, log terstruktur, dan pemanggilan proses eksternal (qpdf).

| Area kontrol (rumusan penyusun) | Kontrol desain | Komponen | Verifikasi | Status |
| --- | --- | --- | --- | --- |
| Query database | sqlx dengan query diperiksa saat compile; metadata `.sqlx` di-commit dan `cargo sqlx prepare --check` di CI (KEPUTUSAN-74 `[SRC-165]` `[SRC-166]`). Query dinamis hanya lewat parameter terikat, bukan penggabungan string (usulan U-02) | `rs-backend`, `rme-backend` | VS-1, RK | Dirancang |
| Pengodean keluaran web | React di Next.js 16.4.6 (KEPUTUSAN-75 `[SRC-156]`). Perilaku escape bawaan dan larangan penyisipan HTML mentah ditetapkan di coding standards; perilaku React tidak terdaftar SRC → diverifikasi Tahap 8 | BFF Next.js | RK, VS-5 (lint) | Tertunda Tahap 8 |
| Konten CMS dan data rekanan | Sanitasi konten CMS (H-03). Pustaka sanitasi belum dipilih `[TIDAK DIKETAHUI]` | `rs-backend` (modul Konten), BFF | RK, T10 | Tertunda Tahap 8 |
| Ekspor CSV | Netralkan sel diawali karakter formula, hanya kolom yang dibutuhkan (M-18) | `rs-backend` | VS-1 (test unit penetral), RK | Tertunda Tahap 8 (masuk coding standards) |
| Log | Log JSON terstruktur lewat tracing-subscriber fitur `json` (KEPUTUSAN-74 `[SRC-238]`), sehingga nilai masukan menjadi field, bukan baris log bebas; redaksi daftar tolak (ADR-010 butir 1) | crate bersama, semua proses Rust | VS-4 | Dirancang |
| Proses eksternal | qpdf dipanggil untuk memeriksa PDF (KEPUTUSAN-86 `[SRC-155]`). Usulan U-02: argumen sebagai daftar, tanpa shell, nama berkas buatan aplikasi | `rs-backend` (modul Platform berkas) | RK | Tertunda Tahap 8 |
| Templat email dan tema login | Templat email tanpa data kesehatan rinci (AB-73, M-10); tema login FreeMarker turunan tema bawaan `[SRC-233]` KEPUTUSAN-85. Pengodean variabel di templat diverifikasi Tahap 8 | `rs-backend` (Notifikasi), Keycloak | RK | Tertunda Tahap 8 |

**Celah terbuka V1:** (1) pustaka sanitasi konten CMS belum dipilih; (2) aturan pengodean di React/Next.js dan FreeMarker belum terverifikasi ke sumber resmi; (3) M-18 belum ditulis di coding standards.

### V2 Validation and Business Logic

**Ruang lingkup:** validasi masukan API aplikasi, portal, Partner Integration API, dan WebSocket; aturan bisnis yang dapat dimanipulasi (harga, nominal, status, waktu); idempotensi dan kondisi balapan; batas anti-otomasi.

| Area kontrol | Kontrol desain | Komponen | Verifikasi | Status |
| --- | --- | --- | --- | --- |
| Validasi skema masukan | Kontrak OpenAPI 3.1 dibangkitkan utoipa dan dibandingkan dengan berkas `openapi/`; Schemathesis terhadap backend lokal (KEPUTUSAN-82 langkah 3, KEPUTUSAN-83 `[SRC-168]` `[SRC-148]`). Pesan rekanan divalidasi skema dan ditolak seluruhnya bila satu bagian salah (ADR-014 butir 4). Galat per isian di `errors` (ADR-023 butir 1) | `rs-backend`, `rme-backend` | VS-3, VS-1 | Dirancang |
| Penolakan field tak dikenal (mass assignment) | Tipe permintaan eksplisit per endpoint (serde `[SRC-167]`). Usulan U-02: field tak dikenal ditolak; atribut serde yang dipakai diverifikasi Tahap 8 | `rs-backend` | RK, VS-1 | Tertunda Tahap 8 |
| Harga, nominal, waktu | Harga dihitung dan dikunci di server; nominal pembayaran = total terkunci; waktu dari jam server (ADR-015 butir 1, 3, 5; ADR-016 butir 3; ADR-017 butir 5) | `rs-backend` (Pemesanan, Pembayaran) | VS-1 (test negatif T1), T10 | Dirancang |
| Transisi status | Transisi divalidasi terhadap tabel transisi; konflik → `4090001` (ADR-008; KEPUTUSAN-84) | `rs-backend` | VS-1 | Dirancang |
| Idempotensi dan balapan | Kunci idempotensi 7 hari dengan hash isi (`4090002`); antrean job `FOR UPDATE SKIP LOCKED` `[SRC-36]`, job tunggal `pg_try_advisory_lock` `[SRC-234]` (KEPUTUSAN-74); refund satu per pesanan lewat partial unique index (PG-04) | `rs-backend`, crate `platform-jobs`, PG | VS-1, VS-2 | Dirancang |
| Anti-otomasi | Batas laju aplikasi di Valkey (ADR-007 butir 6; pola INCR+EXPIRE atomik `[SRC-75]`), per rekanan (ADR-014 butir 3), halaman verifikasi resep per sumber dan global (ADR-012 butir 3); batas per IP di Caddy (modul diverifikasi Tahap 8, KEPUTUSAN-78). Batas pesanan "Menunggu pembayaran" per akun belum ada (M-11) | `rs-backend`, Caddy, Valkey | VS-1, T10 | Dirancang sebagian; M-11 butuh keputusan user |

**Celah terbuka V2:** (1) M-11 nilai batas adalah keputusan bisnis; (2) modul batas laju Caddy belum terverifikasi, dan tanpa WAF terkelola batas per IP bergantung padanya (CR-012, N-03); (3) alamat IP klien yang sampai ke Caddy lewat LB NEO `[TIDAK DIKETAHUI]` (N-04), padahal batas laju per sumber membutuhkannya.

### V3 Web Frontend Security

**Ruang lingkup:** tiga web Next.js (backoffice, portal rekanan, web RME), halaman login Keycloak (dibuka di browser sistem oleh aplikasi mobile dan web), halaman verifikasi resep publik, Swagger UI di staging. Aplikasi Flutter bukan frontend web; penerapannya ke Flutter ditinjau ulang setelah teks bab dibuka.

| Area kontrol | Kontrol desain | Komponen | Verifikasi | Status |
| --- | --- | --- | --- | --- |
| Atribut cookie sesi | Cookie `HttpOnly`, `Secure`, `SameSite`; token tidak sampai ke browser (ADR-011 butir 7, KEPUTUSAN-75) | BFF Next.js | VS-5 (test unit), Playwright e2e `[SRC-133]`, T10 | Dirancang |
| CSRF | Token CSRF untuk aksi yang mengubah data (ADR-011 butir 7, WEB-04) | BFF Next.js | Playwright e2e, T10 | Dirancang |
| Header keamanan | Content Security Policy untuk ketiga web (H-03); halaman verifikasi resep tidak di-cache, tidak diindeks, tanpa sumber daya pihak ketiga, tanpa kode di header perujuk (ADR-012 butir 3, M-31 e). Usulan U-03: satu set header (CSP, larangan framing, HSTS, `nosniff`, `Referrer-Policy`) ditetapkan di satu tempat (Caddy atau Next.js); mekanisme belum terverifikasi | BFF Next.js, Caddy | KF, Playwright e2e, T10 | Tertunda Tahap 8 |
| Pengerasan Next.js | `images.unoptimized: true` `[SRC-159]`; tanpa `next/og`, Edge runtime, `use cache`, middleware untuk keputusan otorisasi; versi dipin; patch Critical/High ≤ 7 hari (KEPUTUSAN-75, riwayat advisory `[SRC-246]`) | BFF Next.js | KF (`next.config`), RK, CI (osv-scanner) | Dirancang |
| Penerusan respons BFF | Route Handler meneruskan panggilan ke API Rust (KEPUTUSAN-75). Advisory "Cross-client response injection and disclosure in self-hosted Next.js applications forwarding fetched responses" (GHSA-pc58-hwjq-p385) `[SRC-246]` ditambal sejak 16.4.5 (terdampak ">= 16.0.0, <= 16.4.4") `[SRC-248]`; versi dipin 16.4.6 tidak terdampak | BFF Next.js | CI (osv-scanner), RK | Dirancang |
| Kunci Server Functions multi-server | Halaman self-hosting memuat bagian "Multi-Server Deployments (Server Functions encryption key, …)" `[SRC-158]`; tiap aplikasi berjalan di app-1 dan app-2 (KEPUTUSAN-78) sehingga konfigurasinya harus konsisten dan kuncinya dari OpenBao | BFF Next.js, OpenBao | KF | Tertunda Tahap 8 |
| Swagger UI | Hanya lokal dan staging di belakang login staging, tidak di production (KEPUTUSAN-83) | Pipeline, Caddy | KF | Dirancang |

**Celah terbuka V3:** (1) mekanisme header keamanan dan CSP belum dipilih; (2) — (GHSA-pc58-hwjq-p385 tertutup: 16.4.6 tidak terdampak `[SRC-248]`); (3) CSP untuk tema login Keycloak belum dirancang; (4) kunci Server Functions untuk deployment dua host belum dikonfigurasi.

### V4 API and Web Service

**Ruang lingkup:** REST/JSON untuk aplikasi, web (lewat BFF), Partner Integration API v1, API RME; WebSocket `realtime`; webhook keluar dan masuk (payment gateway, kurir).

| Area kontrol | Kontrol desain | Komponen | Verifikasi | Status |
| --- | --- | --- | --- | --- |
| Kontrak dan format | OpenAPI per API di `openapi/` sebagai kontrak; envelope `response_code` dengan 3 digit pertama = kode HTTP; galat `application/problem+json` (ADR-022, ADR-023, KEPUTUSAN-83, -84 `[SRC-47]`) | `rs-backend`, `rme-backend`, crate `envelope` | VS-3 | Dirancang |
| Isi galat | Tanpa stack trace, query, data pribadi, rahasia; 404 untuk objek terlarang (ADR-023 butir 5, ADR-012 butir 4) | crate `envelope` | VS-1, VS-3, T10 | Dirancang |
| Namespace per kanal | Namespace API dan audience terpisah per kanal (ADR-006 butir 5, WEB-06); Swagger tidak di production | `rs-backend` | VS-1 (test negatif per realm/klien, ADR-011 butir 8) | Dirancang |
| WebSocket | Autentikasi saat koneksi dibuka, otorisasi per langganan kanal, putus saat token kedaluwarsa atau pencabutan, cadangan `LISTEN/NOTIFY` dan cek ulang 60 detik (ADR-016 butir 1 `[SRC-74]`); didokumentasikan AsyncAPI 3.1.0 (KEPUTUSAN-83 `[SRC-147]`); Caddy meneruskan WebSocket `[SRC-237]`. Usulan U-04: batas ukuran pesan dan laju per koneksi | `realtime`, Caddy | VS-1 (test KU-10), T10 | Dirancang; U-04 Tertunda Tahap 8 |
| Webhook keluar | HMAC-SHA256 atas stempel waktu dan isi, rahasia per rekanan, tolak > 5 menit, `event_id` unik (ADR-014 butir 3–4 `[SRC-46]`) | `rs-backend` (Gateway integrasi) | VS-1, skenario uji rekanan (ADR-014 butir 8, H-08) | Dirancang |
| Webhook masuk pihak ketiga | Payment gateway: verifikasi sesuai penyedia (NFR-INT-04); DOKU menyediakan public key, algoritma `[TIDAK DIKETAHUI]` `[SRC-207]`; Xendit memakai token callback statis `[SRC-199]`. Kurir Lalamove: autentikasi API HMAC-SHA256 `[SRC-216]`, verifikasi keaslian webhook `[TIDAK DIKETAHUI]` (M-30). Usulan U-07: status dikonfirmasi lewat API "tanya status" sebelum memicu konfirmasi pesanan, refund, atau status kirim bila verifikasi webhook tidak kriptografis (perluasan H-09) | `rs-backend` (Pembayaran, Pemesanan) | VS-1, T10 | Tertunda Tahap 8 |
| SSRF | Validasi URL endpoint rekanan saat disimpan dan dipanggil; tolak alamat privat/loopback/link-local; ikat koneksi; tanpa redirect; egress worker dibatasi (ADR-014 butir 3 `[SRC-81]`). Di VPS, alamat metadata penyedia dan rentang jaringan privat Biznet (`[SRC-95]` private network) ikut daftar tolak; pembatasan egress lewat firewall host (KEPUTUSAN-78) | `rs-backend` (`worker`), Host | VS-1, KF | Dirancang; rentang privat Biznet/WireGuard ditetapkan Tahap 8 |

**Celah terbuka V4:** (1) mekanisme verifikasi webhook DOKU dan Lalamove belum terverifikasi (PT-36, PT-37); (2) U-04 batas ukuran/laju WebSocket; (3) daftar tolak SSRF harus memuat rentang jaringan privat Biznet dan alamat WireGuard yang belum ditetapkan.

### V5 File Handling

**Ruang lingkup:** unggah foto KTP/KK, lampiran chat, bukti klaim, PDF hasil lab dan dokumen izin; unduh lewat URL bertanda tangan; salinan data pribadi dan ekspor; konten CMS publik (ADR-018).

| Area kontrol | Kontrol desain | Komponen | Verifikasi | Status |
| --- | --- | --- | --- | --- |
| Jenis, ukuran, nama | Foto JPG/PNG ≤ 5 MB, PDF ≤ 10 MB (NFR-SEC-08); jenis dari isi, bukan header; nama objek UUID; berkas baru dapat dirujuk setelah lolos validasi (ADR-018 butir 2 `[SRC-50]`); `4130001`/`4150001` (KEPUTUSAN-84) | `rs-backend` (Platform berkas) | VS-1, T10 | Dirancang |
| Pembatasan URL unggah | ADR-018 butir 2 meminta URL unggah dibatasi jenis dan ukuran "sejauh kemampuan penyedia". NOS: URL bertanda tangan terverifikasi untuk `get_object`; dukungan PUT/POST bertanda tangan tidak dinyatakan `[SRC-98]` → `[TIDAK DIKETAHUI]` | NOS, `rs-backend` | KF, VS-1 (test integrasi staging) | Tertunda Tahap 8 |
| Penimpaan setelah validasi | Salin ke kunci final yang tidak pernah punya URL unggah, cocokkan checksum (M-21) | `rs-backend` | VS-1 | Tertunda Tahap 8 (revisi ADR-018 butir 2) |
| Metadata gambar | Dekode dan kode ulang foto (ADR-018 butir 2). Pustaka gambar Rust belum dipilih `[TIDAK DIKETAHUI]` | `rs-backend` | VS-1 | Tertunda Tahap 8 |
| PDF berbahaya | qpdf `--json` menolak PDF berkonten aktif (JavaScript, OpenAction, Launch, EmbeddedFile, AcroForm/XFA); CDR/render ulang tidak tersedia saat go-live → jalur penolakan (KEPUTUSAN-86, ADR-018 butir 6 `[SRC-155]`); antivirus `[TIDAK DIKETAHUI]` | `rs-backend` | VS-1 (korpus PDF uji), T10 | Dirancang (fallback); antivirus Tertunda |
| Unduh | Otorisasi → audit E-05/E-12 → URL 5 menit sebagai lampiran (ADR-018 butir 3). Endpoint NOS harus HTTPS; contoh resmi memakai `http://nos.wjv-1.neo.id` `[SRC-98]` sehingga dukungan HTTPS `[TIDAK DIKETAHUI]` (N-16) | `rs-backend`, NOS | VS-1, KF, T10 | Tertunda Tahap 8 |
| Bucket dan versi | Bucket privat per sensitivitas; hanya CMS dan katalog publik (ADR-018 butir 1); pemeriksaan otomatis kebijakan bucket (H-11); versi objek (ADR-018 butir 5) — fitur versi NOS tidak dinyatakan di SRC-97/SRC-99 `[TIDAK DIKETAHUI]`; Object Lock Compliance tersedia `[SRC-99]` | NOS, `infra/` | KF, VS (test kebijakan bucket, H-11) | Tertunda Tahap 8 |

**Celah terbuka V5:** (1) URL unggah bertanda tangan NOS dan pembatasannya; (2) HTTPS NOS; (3) versi objek NOS; (4) antivirus/sandbox tidak tersedia (risiko lama ADR-018 butir 6); (5) pustaka kode ulang gambar.

### V6 Authentication

**Ruang lingkup:** autentikasi manusia di Keycloak (realm `pasien`, `nakes`, `admin`, `rme`), klien integrasi (realm `integrasi`), autentikasi manusia ke OpenBao dan bastion, autentikasi proses ke OpenBao.

| Area kontrol | Kontrol desain | Komponen | Verifikasi | Status |
| --- | --- | --- | --- | --- |
| Penyimpanan password | Argon2 bawaan Keycloak; opsi SPI `memory`, `iterations`, `parallelism` diatur minimal 19 MiB, 2, 1 (KEPUTUSAN-85 `[SRC-232]` `[SRC-49]`) | Keycloak | KF, test konfigurasi (VS-1 testcontainers Keycloak) | Dirancang |
| Kebijakan password | Nilai NFR-SEC-03 v1.2: 8–128 karakter, tanpa aturan komposisi, diperiksa persis, daftar kata sandi umum/bocor/kata konteks sebagai berkas lokal "Password blacklist", "Not email", tanpa rotasi berkala (6.2.1, 6.2.4–6.2.12) `[KEPUTUSAN-188]` `[SRC-265]` `[SRC-270]`; sama di semua jalur (6.1.3, 6.3.4). Daftar bocor dan lisensinya `[TIDAK DIKETAHUI]`, diverifikasi F1-T31 | Keycloak, `rs-backend` | KF, VS-1, T10 | Dirancang (v1.1; sebelumnya Tertunda Tahap 8) |
| Ubah password (6.2.2, 6.2.3 L1 `[SRC-276]`) | Pengguna yang login dapat mengubah password; password lama dan baru wajib; autentikasi ulang; opsi keluar dari perangkat lain tercentang bawaan (US-103 AC6, CR-017 `[USER] C2-B14`; KEPUTUSAN-198). Mekanisme Keycloak `[TIDAK DIKETAHUI]`, diverifikasi Tahap 8 | Keycloak, `rs-backend`, Flutter, BFF | VS-1, Playwright, T10 | Dirancang (v1.1; sebelumnya celah) |
| Brute force | Mode sementara, 5 gagal, kunci 15 menit (ADR-011 butir 4 `[SRC-69]`); Max Wait 15 menit, strategi linear `[KEPUTUSAN-190]`; berlaku juga untuk OTP dan kode pemulihan `[SRC-273]`; test "5 kali berturut-turut" | Keycloak | VS-1 (testcontainers Keycloak) | Dirancang |
| MFA (6.3.3 L2) | TOTP wajib di **semua realm pengguna: `pasien`, `nakes`, `admin`, `rme`** dengan alur yang mewajibkan OTP; Configure OTP saat login pertama (pasien: setelah email terverifikasi); 12 kode pemulihan wajib dibuat saat pendaftaran MFA (ADR-011 butir 5 revisi Checkpoint 2 `[USER] C2-Q9` `[SRC-68]` `[SRC-70]`; KEPUTUSAN-202, KEPUTUSAN-203). **6.3.3 kini dirancang memenuhi untuk realm `pasien` (terpenuhi setelah diimplementasi dan diuji di Fase 1)** `[SRC-265]`; sebelum revisi, klaim L2 realm `pasien` tidak benar. Faktor perangkat keras tahan phishing hanya wajib di L3 `[SRC-276]`. Parameter: SHA1 (default Keycloak), 6 digit, 30 detik demi kompatibilitas authenticator (KEPUTUSAN-85 `[SRC-67]`); SRC-67 menyebut SHA256/SHA512 "more secure" → dicek ke teks V6/V11 (N-20). Mekanisme mewajibkan kode pemulihan `[TIDAK DIKETAHUI]`, diverifikasi Tahap 8 | Keycloak, tema login | VS-1, uji dua aplikasi authenticator (Tahap 8), T10 | Dirancang; pilihan algoritma dan mekanisme kode pemulihan dicek Tahap 8 |
| Pendaftaran dan pemulihan | Verifikasi email wajib; tautan atur ulang 60 menit (ADR-011 butir 4 `[SRC-70]`). **M-12 ditutup lewat UP-73 opsi B** `[USER] C2-B4`: pendaftaran dengan email terdaftar menampilkan layar sama dan petunjuk dikirim ke email itu (US-101 AC3); pesan seragam untuk semua kegagalan login (US-103 AC2); halaman pendaftaran Keycloak OFF dan kompensasi berdasarkan ID `[KEPUTUSAN-201]`; batas laju diam per alamat `[KEPUTUSAN-192]`. Selisih waktu respons yang tersisa dicatat sebagai risiko sisa (6.3.8 berlevel L3). H-04 sebagian: pemberitahuan setelah atur ulang password dan ubah email `[KEPUTUSAN-200]`; pemberitahuan login perangkat baru dan penguncian tidak dibangun | Keycloak, `rs-backend` | VS-1, uji sebaran waktu F1-T50, T10 | Dirancang (v1.1); H-04 sebagian |
| Atur ulang tidak melewati MFA (6.4.3 L2) | Setelah tautan email dibuka, OTP atau kode pemulihan tetap diminta sebelum password baru berlaku, di semua realm pengguna termasuk `pasien` (US-103 AC4e) `[KEPUTUSAN-199]` `[USER] C2-Q9`. Dokumentasi Keycloak hanya menyebut "after email address confirmation" `[SRC-272]`; konfigurasi alur Reset Credentials diuji F1-T31/F1-T55, dan bila tidak dapat dikonfigurasi, dibawa ke user | Keycloak | VS-1, T10 | Dirancang; konfigurasi diverifikasi Tahap 8 |
| Faktor hilang (6.4.4 L2 `[SRC-276]`) | Jalur mandiri dengan kode pemulihan, lalu daftar ulang TOTP (US-103 AC7, AC8). Jalur pasien lewat Layanan Pelanggan: tautan konfirmasi sekali pakai 60 menit ke email akun + masuk dengan password, tanpa admin melihat atau memilih kredensial, semua sesi diakhiri, pemberitahuan email, E-04 (US-207 AC5; KEPUTUSAN-204). Bukti setara pendaftaran pasien (kendali email) ditambah password. Jalur tenaga kesehatan, admin, RME: prosedur H-05. Kasus perangkat + kode pemulihan + email/password hilang bersamaan: bukti pengganti `[TIDAK DIKETAHUI]` (RT-37) | Keycloak, `rs-backend`, backoffice, prosedur | VS-1, review prosedur, T10 | Dirancang; prosedur H-05 (mencakup pasien) sebelum go-live |
| Reset MFA | Admin berwenang setelah verifikasi identitas, E-04, autentikasi ulang (ADR-011 butir 5, 9); reset mengakhiri semua sesi (KEPUTUSAN-198 butir 4); untuk pasien oleh Layanan Pelanggan (US-207 AC5); prosedur H-05 sebelum go-live | Keycloak, `rs-backend`, prosedur | Review prosedur, T10 | Tertunda (prosedur sebelum go-live) |
| Klien integrasi | Client credentials per rekanan/fasyankes, token pendek (ADR-014 butir 3 `[SRC-45]`). mTLS "dipertimbangkan di Tahap 5" (ADR-014 butir 3) belum diputuskan di KEPUTUSAN-74–89 | Keycloak (`integrasi`), `rs-backend` | VS-1, T10 | Dirancang; mTLS belum diputuskan |
| Manusia ke infrastruktur | Login OpenBao dengan Login MFA metode TOTP `[SRC-251]` → sertifikat SSH berumur pendek untuk bastion `[SRC-225]` (ADR-025, `[USER] T5-Q8`). Perlindungan brute force passcode TOTP tidak aktif bawaan; dokumentasi menganjurkan batas laju pada jalur login dan `/sys/mfa/validate`, `max_validation_attempts` bawaan 5 `[SRC-251]` | OpenBao, bastion | KF, T10 | Dirancang (konfigurasi batas laju Tahap 8) |
| Proses ke OpenBao | Identitas per proses dengan policy minimum; AppRole atau JWT dipilih Tahap 8; runner staging lewat OIDC GitHub dibatasi `sub` repo + ref `main` (KEPUTUSAN-79, -81 `[SRC-223]` `[SRC-239]`) | OpenBao, Pipeline | KF | Tertunda Tahap 8 |

**Celah terbuka V6 (v1.1):** (1) daftar kata sandi bocor dan lisensinya, serta mekanisme Keycloak untuk ubah password, kode pemulihan wajib, dan Reset Credentials ber-OTP (diverifikasi Tahap 8); (2) batas laju jalur login dan validasi MFA OpenBao (mekanisme MFA sendiri sudah ditetapkan, `[SRC-251]`), serta mekanisme batas laju di halaman dan proxy Keycloak (S-03); (3) keputusan mTLS klien integrasi; (4) H-04 sebagian, H-05 (prosedur yang kini mencakup pasien), dan bukti identitas pengganti kasus RT-37; (5) TOTP SHA1 perlu dibandingkan dengan teks requirement. Celah lama "nilai kebijakan password" dan M-12 tertutup di v1.1.

### V7 Session Management

**Ruang lingkup:** sesi BFF web, sesi IdP Keycloak, sesi mobile (token di perangkat), daftar pencabutan, koneksi WebSocket, kunci aplikasi nakes.

| Area kontrol | Kontrol desain | Komponen | Verifikasi | Status |
| --- | --- | --- | --- | --- |
| Penyimpanan sesi BFF | Data sesi di Valkey (ioredis 6.0.0 `[SRC-116]`); token dienkripsi JWE (jose 6.2.17 `[SRC-114]`) dengan kunci dari OpenBao dan diikat ke ID sesi (KEPUTUSAN-75, ADR-011 butir 7, M-06) | BFF Next.js, Valkey, OpenBao | VS-5, RK, T10 | Dirancang |
| Batas waktu | Web idle 30 menit, mutlak 12 jam; pasien mobile 30/90 hari; nakes mobile 7/30 hari; token akses 5 menit (ADR-011 butir 7 `[SRC-70]`, NFR-SEC-04) | Keycloak, BFF | VS-1 (konfigurasi realm), Playwright | Dirancang |
| Sesi paralel (7.1.2 L2 `[SRC-277]`) | **Kebijakan terdokumentasi:** jumlah sesi paralel per akun tidak dibatasi di semua realm pengguna (`pasien`, `nakes`, `admin`, `rme`) `[USER] C2-Q11` (NFR-SEC-11, ADR-011 butir 12). Tidak ada batas maksimum, sehingga tidak ada perilaku saat batas tercapai. Alasan dan kompensasi: setiap sesi dapat dilihat dan diakhiri pemiliknya (US-110); batas idle dan mutlak per realm tetap berlaku; sesi lain diakhiri setelah atur ulang password, ubah email, dan reset MFA, serta dapat diakhiri saat ubah password atau daftar ulang TOTP (KEPUTUSAN-198); penonaktifan akun mengakhiri semua sesi (ADR-011 butir 8). Pilihan membatasi satu sesi web per akun admin ditolak user | Keycloak, BFF | KF (tanpa pembatas sesi di konfigurasi realm), T10 | Dirancang (terdokumentasi, v1.1) |
| Melihat dan mengakhiri sesi (7.5.2 L2 `[SRC-277]`) | Layar "Perangkat dan sesi aktif" di aplikasi pasien, aplikasi tenaga kesehatan, backoffice, portal rekanan (Fase 2), dan web RME (diverifikasi Fase 3) (US-110 `[USER] C2-Q10`). Daftar dari Admin REST API "Get sessions associated with the user"; pengakhiran lewat "Remove a specific user session" `[SRC-275]` ditambah `session_revocation`. Mengakhiri sesi wajib autentikasi ulang (jendela 5 menit) sesuai syarat "having authenticated again with at least one factor" `[KEPUTUSAN-205]` `[KEPUTUSAN-195]`. Pengakhiran sesi BFF (back-channel logout atau pencabutan di Valkey) diverifikasi Tahap 8 | Keycloak, `rs-backend`, BFF, Flutter | VS-1, Playwright, uji widget Flutter, T10 | Dirancang (v1.1; sebelumnya celah H-01) |
| Mengakhiri sesi setelah perubahan kredensial (7.4.3 L2 `[SRC-266]`) | Atur ulang password, ubah email, dan reset MFA mengakhiri semua sesi; ubah password dan daftar ulang TOTP menampilkan opsi keluar dari perangkat lain yang tercentang bawaan (KEPUTUSAN-198, KEPUTUSAN-204; US-103 AC4d, AC6, AC7; US-105 AC4d; US-207 AC5) | Keycloak, `rs-backend` | VS-1 (jeda ≤ 2 menit diukur F1-T54), T10 | Dirancang (v1.1) |
| Rotasi token penyegar | "Revoke Refresh Token" aktif di semua realm pengguna (KEPUTUSAN-85 `[SRC-231]`); jumlah pemakaian ulang 0, nama opsi `[TIDAK DIKETAHUI]`; pencabutan rantai saat token lama dipakai ulang (H-01) diverifikasi Tahap 8 | Keycloak | VS-1 (test pemakaian ulang) | Tertunda Tahap 8 |
| Pencabutan seketika | Status akun dan sesi dicabut di DB (sumber kebenaran) + cache Valkey; penonaktifan di Keycloak + hapus sesi lewat Admin REST API; aksi berisiko baca DB langsung (ADR-011 butir 8 `[SRC-72]`, M-13) | `rs-backend`, `rme-backend`, Valkey, Keycloak | VS-1 (KU-10, US-1302 AC4) | Dirancang |
| Keluar | Logout Keycloak/revokasi + catatan sesi dicabut di tabel Operasional (ADR-011 butir 8 `[SRC-71]`); back-channel logout tersedia `[SRC-70]` | Keycloak, BFF, `rs-backend` | VS-1, Playwright | Dirancang |
| Autentikasi ulang | Aksi berisiko meminta login ulang dan memeriksa waktu autentikasi (ADR-011 butir 9); parameter `max_age`/`prompt=login`/`auth_time` belum terdaftar SRC (KEPUTUSAN-85) | Keycloak, `rs-backend`, BFF, Flutter | VS-1, T10 | Tertunda Tahap 8 |
| Mobile | Token di flutter_secure_storage 11.2.0 `[SRC-136]`; kunci aplikasi nakes 15 menit lewat local_auth `[SRC-137]` (ADR-021 butir 1, NFR-SEC-10); prosedur perangkat hilang (ADR-021 butir 1) | Flutter | VS-6, uji perangkat Tahap 9 | Dirancang |
| WebSocket | Diputus saat token kedaluwarsa dan saat pencabutan (ADR-016 butir 1) | `realtime` | VS-1 | Dirancang |

**Celah terbuka V7 (v1.1):** (1) opsi pemakaian ulang refresh token dan perilaku pencabutan rantai (H-01); (2) parameter autentikasi ulang (dipakai juga oleh US-110); (3) bila Valkey RS hilang, semua pengguna web harus login ulang (ADR-007 butir 6) — tetap, tidak berubah oleh CR-012. Celah lama "keluar dari semua perangkat" (H-01) tertutup lewat US-110 dan KEPUTUSAN-198; 7.1.2 terdokumentasi. Sampai US-110 selesai di Fase 2, 7.5.2 belum terpenuhi pada build Fase 1 (bukan rilis production).

### V8 Authorization

**Ruang lingkup:** otorisasi per endpoint dan per objek, isolasi rekanan dan fasyankes, sub-peran admin, hak database, hak admin Keycloak, kebijakan OpenBao, hak manusia ke infrastruktur.

| Area kontrol | Kontrol desain | Komponen | Verifikasi | Status |
| --- | --- | --- | --- | --- |
| Model dan titik kebijakan | RBAC + relasi (kepemilikan, penugasan berbatas waktu, tenant, sub-peran); default deny; satu modul kebijakan (ADR-012 butir 1) di crate `authz` (KEPUTUSAN-74); pemeriksaan per objek `[SRC-80]` | crate `authz`, `rs-backend`, `rme-backend` | VS-1 (test penolakan per endpoint, DoD), RK | Dirancang |
| Data sensitif | Fungsi akses yang menulis E-05 sebelum data dikirim (ADR-012 butir 2, ADR-009 butir 2) | crate `authz`, crate `audit` | VS-1 | Dirancang |
| Isolasi Ruang Sehat | `partner_id` dari sesi lewat repository ber-tenant; test isolasi otomatis setiap endpoint portal dan Partner Integration API (ADR-012 butir 5, H-08) | `rs-backend` | VS-1 | Dirancang; H-08 terbuka |
| Isolasi RME | RLS `FORCE`, role aplikasi bukan pemilik dan tanpa `BYPASSRLS` `[SRC-34]`; role operator tanpa `SELECT` klinis (ADR-012 butir 6); RLS gagal-tertutup bila konteks kosong dan test kebocoran konteks di pool (H-12); batas hak operator di database (M-24) | PG (c), `rme-backend` | VS-2 (test RLS FORCE, KEPUTUSAN-82), VS-1 | Dirancang; H-12, M-24 terbuka |
| Isolasi log akses audit RME | `audit_tenant_reader` ber-RLS per `tenant_id`, atau fungsi baca per tenant bila RLS pada tabel terpartisi tidak memadai (ADR-012 butir 9); H-13 | PG (b) | VS-2 | Tertunda Tahap 8 (verifikasi RLS tabel terpartisi) |
| Pemisahan tugas | Larangan ubah peran sendiri, Super Admin/Admin Fasyankes terakhir, persetujuan rekening oleh Verifikator (ADR-012 butir 7); pemberitahuan pemberian sub-peran sensitif (M-16) | `rs-backend` | VS-1 | Dirancang; M-16 terbuka |
| Hak database | Role per kelas tabel, trigger penolak, pemilik skema hanya saat migrasi (ADR-008, ADR-012 butir 8); kredensial pemilik skema dinamis berumur pendek dari OpenBao (KEPUTUSAN-79 `[SRC-224]`) | PG, OpenBao, `db/roles/` | VS-2 | Dirancang |
| Admin Keycloak | Klien Admin REST per realm, peran minimum; `/admin/` dan `/realms/master/` hanya internal, `hostname-admin` terpisah (ADR-011 butir 2, KEPUTUSAN-85 `[SRC-229]` `[SRC-230]`) | Keycloak, Caddy | KF, VS-1 (test akses lintas realm), T10 (uji dari internet) | Dirancang |
| Kebijakan OpenBao dan manusia | Policy minimum per proses (api, realtime, worker, audit-relay, RME, pekerja privasi, rotasi kunci, BFF); manusia tanpa hak tetap (KEPUTUSAN-79, ADR-021 butir 3). Root token hanya lewat `generate-root` dengan kuorum pemegang share, lalu dicabut segera setelah dipakai `[SRC-252]`; persetujuan orang kedua untuk akses darurat lewat Control Groups `[SRC-253]` (ADR-025). OpenBao tidak memberi tahu penyetuju tentang permintaan tertunda `[SRC-253]`, sehingga pemberitahuan perlu dibuat terpisah | OpenBao | KF, review akses berkala, latihan break-glass | Dirancang |

**Celah terbuka V8:** (1) H-08, H-12, H-13, M-16, M-24; (2) RLS pada tabel audit terpartisi belum diverifikasi; (3) pemberitahuan ke penyetuju Control Groups (OpenBao tidak mengirimnya `[SRC-253]`); (4) RS dan RME berbagi host app-1/app-2 sehingga batas sistem di tingkat host hanya oleh kontainer (N-06).

### V9 Self-contained Tokens

**Ruang lingkup:** JWT akses dari Keycloak yang divalidasi backend; JWE di data sesi BFF; token bergabung LiveKit; token OIDC GitHub ke OpenBao.

| Area kontrol | Kontrol desain | Komponen | Verifikasi | Status |
| --- | --- | --- | --- | --- |
| Validasi JWT backend | jsonwebtoken 12.0.0 dengan daftar izin algoritma, issuer per realm, audience; JWKS realm di-cache (KEPUTUSAN-74 `[SRC-169]`, ADR-011 butir 8 `[SRC-79]`); backend RS menolak token `rme` dan sebaliknya (ADR-011 butir 11) | `rs-backend`, `rme-backend` | VS-1 (test negatif per kombinasi realm/klien, H-06) | Dirancang |
| JWE sesi BFF | jose 6.2.17, kunci dari OpenBao, diikat ke ID sesi (KEPUTUSAN-75 `[SRC-114]`) | BFF Next.js | VS-5, RK | Dirancang |
| Token video | Per peserta per sesi, berumur pendek, dalam jendela sesi (ADR-016 butir 4, KEPUTUSAN-87) | `rs-backend`, LiveKit | VS-1 | Dirancang |
| Token OIDC pipeline | Klaim `sub` repo + ref `main` diikat di role OpenBao (`bound_claims`/`bound_subject`) (KEPUTUSAN-81 `[SRC-223]` `[SRC-239]`) | OpenBao, Pipeline | KF | Dirancang |

**Celah terbuka V9:** (1) perilaku rotasi kunci penandatangan realm (JWKS) dan masa cache JWKS belum ditetapkan — diverifikasi Tahap 8.

### V10 OAuth and OIDC

**Ruang lingkup:** Keycloak sebagai authorization server; Flutter sebagai klien publik native; BFF Next.js sebagai klien rahasia; klien integrasi; OpenBao sebagai pihak yang menerima token OIDC GitHub.

| Area kontrol | Kontrol desain | Komponen | Verifikasi | Status |
| --- | --- | --- | --- | --- |
| Klien native | Authorization Code + PKCE lewat browser sistem dengan flutter_appauth (ADR-011 butir 7 `[SRC-77]`, KEPUTUSAN-76 `[SRC-135]`); `pkce-enforcer` S256 `[SRC-71]` | Flutter, Keycloak | KF, VS-6, T10 | Dirancang |
| Klien rahasia BFF | Authorization Code dengan openid-client 6.9.0 (KEPUTUSAN-75 `[SRC-115]`); rahasia klien dari OpenBao | BFF Next.js, Keycloak | VS-5, Playwright, T10 | Dirancang |
| Klien integrasi | Client credentials per rekanan `[SRC-45]`; status `revoked`; prosedur rotasi rahasia dengan tumpang tindih (M-20) | Keycloak, `rs-backend` | VS-1 | Dirancang; M-20 terbuka |
| Redirect URI dan konfigurasi realm | Konfigurasi realm sebagai kode dengan keycloak-config-cli 6.5.1 (KEPUTUSAN-85 `[SRC-122]`); test MFA wajib, penguncian, batas sesi, penolakan token realm lain (CI-04) | Keycloak, Pipeline | KF, VS-1 | Dirancang |
| Endpoint publik Keycloak | Hanya `/realms/{realm pengguna}/`, `/resources/`, `/.well-known/` lewat proxy (KEPUTUSAN-85 `[SRC-229]`) | Caddy, Keycloak | KF, T10 | Dirancang |

**Celah terbuka V10:** (1) pemetaan jenis event Keycloak per versi (ADR-011 butir 10) ditetapkan Tahap 8; (2) keycloak-config-cli rilis terakhir ~6 bulan lalu `[SRC-122]`, dipantau (KEPUTUSAN-85); (3) M-20.

### V11 Cryptography

**Ruang lingkup:** enkripsi kolom data spesifik (envelope), blind index NIK, tanda tangan webhook, hash berantai audit, kode acak (verifikasi resep), pengelolaan dan rotasi kunci, enkripsi backup.

| Area kontrol | Kontrol desain | Komponen | Verifikasi | Status |
| --- | --- | --- | --- | --- |
| Satu pustaka kripto | aws-lc-rs 1.19.0 untuk AES-256-GCM, HMAC-SHA256, SHA-256, RNG; `aes-gcm` dan `ring` tidak dipakai (KEPUTUSAN-74 `[SRC-173]`) | crate `crypto` | VS-1 (test vektor), cargo-deny (larangan crate kripto lain, usulan U-02), RK | Dirancang |
| Envelope encryption | DEK per wadah dibangkitkan lewat Transit `datakey`, dibungkus KEK; rotasi KEK dengan versi, `min_decryption_version`, `rewrap` (KEPUTUSAN-79 `[SRC-222]`, ADR-013 butir 3, 5); DEK di-cache di memori dengan batas waktu | crate `crypto`, OpenBao | VS-1 (testcontainers OpenBao dev — ketersediaan image diverifikasi Tahap 8), RK | Dirancang |
| Blind index dan kode acak | HMAC-SHA256 dengan kunci terpisah berversi `nik_bidx_key_version` (ADR-013 butir 4 `[SRC-46]`, KEPUTUSAN-88); kode QR resep ≥ 128 bit acak dari RNG kriptografis (ADR-012 butir 3, M-31 a) | crate `crypto` | VS-1 | Dirancang |
| Hash berantai audit | `event_hash = SHA-256(prev_hash ‖ bentuk kanonis)`; kepala rantai tiap jam ke NOS Object Lock Compliance (ADR-009 butir 5 `[SRC-99]`) | `audit-relay`, NOS | VS-1, job verifikasi harian | Dirancang |
| TOTP | SHA1 untuk kompatibilitas (KEPUTUSAN-85 `[SRC-67]`) | Keycloak | T10 | Dirancang; dicek ke teks bab (N-20) |
| Enkripsi backup | pgBackRest `repo-cipher-type` aes-256-cbc (KEPUTUSAN-78 `[SRC-235]`); opsi yang tercantum di SRC hanya `none` dan `aes-256-cbc`. Mode CBC tidak memberi integritas sendiri; kecukupannya dicek ke teks bab V11 (N-21) | pgBackRest, Host | KF, uji pemulihan | Dirancang; dicek Tahap 8 |
| Kunci material | KEK di OpenBao (seal Shamir, tanpa auto-unseal `[SRC-228]`); snapshot Raft terenkripsi ke NOS (perintah diverifikasi Tahap 12); prosedur rotasi KEK dan blind index tertulis sebelum go-live (ADR-013 butir 8, KEPUTUSAN-88) | OpenBao | Review prosedur, uji pemulihan termasuk dekripsi (KMS-02) | Dirancang; prosedur Tertunda |

**Celah terbuka V11:** (1) TOTP SHA1; (2) pgBackRest CBC; (3) KEK disimpan perangkat lunak di VM, tanpa HSM (N-02); (4) pemegang share unseal (PT-38).

### V12 Secure Communication

**Ruang lingkup:** TLS dari internet (LB NEO → Caddy), lalu lintas antar-host di jaringan privat Biznet, jalur Biznet ↔ Hostinger, koneksi ke pihak ketiga (SES, DOKU, Lalamove, SATUSEHAT), media WebRTC, klaster Keycloak.

| Area kontrol | Kontrol desain | Komponen | Verifikasi | Status |
| --- | --- | --- | --- | --- |
| TLS eksternal | Caddy dengan HTTPS otomatis dan pengalihan HTTP→HTTPS `[SRC-237]`; LB NEO diteruskan sebagai TCP bila tersedia, protokol listener belum terverifikasi (KEPUTUSAN-78 `[SRC-102]`, PT-39). Versi minimum dan cipher "mengikuti checklist ASVS L2" (ADR-013 butir 1) → usulan U-05: TLS 1.2 minimum dengan TLS 1.3 diutamakan, final setelah teks bab V12 dibuka | Caddy, LB NEO | KF, T10 (pemindaian TLS) | Tertunda Tahap 8 |
| Antar-host Biznet | ADR-013 butir 1: semua lalu lintas internal memakai TLS. Berlaku untuk PG, Valkey, OpenBao, Keycloak, API antar-host. Dukungan dan konfigurasi TLS Valkey dan PostgreSQL belum terdaftar SRC → Tahap 8. Usulan U-06 soal lalu lintas di dalam satu host (§6 butir 2) | Host, PG, Valkey, OpenBao | KF, T10 | Tertunda Tahap 8 |
| Biznet ↔ Hostinger | WireGuard untuk replikasi standby, Raft OpenBao bao-3, salinan backup (KEPUTUSAN-78); perangkat WireGuard belum terdaftar SRC, diverifikasi Tahap 8 Fase 1 | Host | KF | Tertunda Tahap 8 |
| Klaster Keycloak | 2 node (app-1, app-2); mode klaster/cache antar-node diverifikasi Tahap 8 (KEPUTUSAN-85 `[SRC-66]`); enkripsi lalu lintas antar-node `[TIDAK DIKETAHUI]` | Keycloak | KF | Tertunda Tahap 8 |
| Pihak ketiga | SES lewat API HTTPS region ap-southeast-3 (`[USER] T5-Q4` `[SRC-84]` `[SRC-209]`); SATUSEHAT TLS, URL dasar resmi (ADR-020, SS-03); DOKU/Lalamove HTTPS | `rs-backend`, `rme-backend` | RK, KF | Dirancang |
| Object storage | Endpoint NOS HTTPS `[TIDAK DIKETAHUI]` (contoh resmi memakai http `[SRC-98]`, N-16) | NOS | KF, uji Tahap 8 | Tertunda Tahap 8 |
| Perutean Magic Transit | Anti-DDoS Biznet "Cloudflare Magic Transit" `[SRC-95]`; TLS diterminasi di Caddy sehingga isi tetap terenkripsi, lokasi perutean `[TIDAK DIKETAHUI]` (KEPUTUSAN-78, PT-39; butir gate T4-Q1) | LB NEO, Host | Konfirmasi tertulis Biznet Gio | Tertunda (keputusan user) |

**Celah terbuka V12:** (1) versi TLS minimum dan cipher; (2) TLS internal PG/Valkey/OpenBao/Keycloak; (3) WireGuard; (4) HTTPS NOS; (5) Magic Transit (N-05); (6) penanganan TLS di dalam satu host (§6 butir 2).

### V13 Configuration

**Ruang lingkup:** konfigurasi sebagai kode, rahasia per lingkungan, pengerasan host VPS, Docker, Caddy, Keycloak, Next.js, rantai pasok dependensi dan image, pemisahan lingkungan.

| Area kontrol | Kontrol desain | Komponen | Verifikasi | Status |
| --- | --- | --- | --- | --- |
| Konfigurasi sebagai kode | Compose per lingkungan, Caddyfile, realm Keycloak, policy OpenBao, LiveKit, pgBackRest, aturan Prometheus/Loki di `infra/` (KEPUTUSAN-78, -89); perangkat IaC provisioning VM `[TIDAK DIKETAHUI]` (kandidat OpenTofu `[SRC-127]`) | `infra/` | KF, RK | Dirancang; IaC Tertunda Tahap 8 |
| Rahasia | Semua rahasia staging/production di OpenBao; tidak di image, repo, Compose, variabel CI, kecuali kunci privat cosign di secret GitHub (KEPUTUSAN-88 `[SRC-82]`); gitleaks (KEPUTUSAN-82 langkah 7 `[SRC-151]`) | OpenBao, Pipeline | VS-7, CI | Dirancang |
| Rantai pasok | Versi dipin, lockfile, `pnpm install --frozen-lockfile`, cargo-deny/cargo-audit, osv-scanner, Trivy; image lewat digest dan ditandatangani cosign (KEPUTUSAN-81, -82 `[SRC-242]` `[SRC-154]`); usulan U-08: SBOM per rilis | Pipeline | VS-1, VS-5, CI | Dirancang; U-08 Tertunda |
| Pengerasan host | Firewall publik: hanya LB (443) dan bastion (22, dibatasi) (KEPUTUSAN-78 `[SRC-95]`). Firewall dan security group production berupa **daftar izin per host sumber dan port**, bukan "jaringan privat" umum (misalnya PostgreSQL hanya dari app-1, app-2, node OpenBao, dan bastion jalur darurat; Valkey hanya dari app-1/app-2); staging di jaringan terpisah dari production (ADR-025). Kemampuan pemisahan jaringan Biznet Gio diverifikasi Tahap 8. Pernyataan firewall publik tidak menyebut port publik host `video` untuk WebRTC/TURN (§6 butir 1). Patch OS, Docker 29.10.0, Compose 5.8.0 (`[SRC-187]` `[SRC-188]`) menjadi tanggung jawab tim (N-07); distribusi OS dan mekanisme patch `[TIDAK DIKETAHUI]` | Host | KF, T10 (pemindaian port dari internet) | Tertunda Tahap 8 |
| Pengerasan kontainer | Usulan U-06: kontainer non-root, root filesystem baca-saja bila memungkinkan, socket Docker hanya untuk agen deploy, jaringan Docker terpisah untuk RS dan RME, core dump dimatikan (M-25) | Host, `infra/` | KF, Trivy (konfigurasi) | Tertunda Tahap 8 |
| Fitur tak dipakai | Swagger UI tidak di production (KEPUTUSAN-83); Next.js tanpa image optimization, `next/og`, Edge, Cache Components (KEPUTUSAN-75); LiveKit tanpa Egress (KEPUTUSAN-87 `[SRC-220]`); konsol admin Keycloak di host admin terpisah `[SRC-230]` | Semua | KF | Dirancang |
| Pemisahan lingkungan | Data production tidak pernah disalin; CI hanya data sintetis (ADR-019 butir 1–2, `[USER] T5-Q6`); production tanpa runner dan tanpa kredensial di GitHub (KEPUTUSAN-81); staging di jaringan terpisah dari production (ADR-025) | Pipeline, Host | KF, T10 (uji jangkauan dari host staging) | Dirancang |
| Runner staging | Runner self-hosted JIT sekali pakai per job ("perform at most one job before being automatically removed") `[SRC-254]`, hanya untuk workflow `main` (U-12), OpenBao staging hanya berisi rahasia sintetis (ADR-025). GitHub mengingatkan kehati-hatian juga untuk repo privat `[SRC-254]` | Pipeline | KF | Dirancang (mekanisme JIT diuji Tahap 8) |

**Celah terbuka V13:** (1) perangkat IaC; (2) pengerasan host dan rezim patch (N-07); (3) pengerasan kontainer U-06; (4) SBOM U-08; (5) port publik host video tidak tercakup aturan firewall (§6 butir 1); (6) — (runner staging tertutup di desain: JIT sekali pakai `[SRC-254]`, jaringan staging terpisah; tinggal pembuktian Tahap 8).

### V14 Data Protection

**Ruang lingkup:** klasifikasi dan perlindungan data spesifik, minimisasi data keluar, data di perangkat, cache, retensi dan pemusnahan, backup dan replika.

| Area kontrol | Kontrol desain | Komponen | Verifikasi | Status |
| --- | --- | --- | --- | --- |
| Enkripsi tingkat aplikasi | Kolom data spesifik dienkripsi per wadah (ADR-013 butir 3, KEPUTUSAN-43); NIK tiga bentuk (ADR-013 butir 4) | crate `crypto`, `rs-backend`, `rme-backend` | VS-1, RK (DBA + security) | Dirancang |
| Enkripsi lapis penyedia | ADR-013 butir 2 mengandalkan enkripsi disk/backup/object storage penyedia dengan KMS. Di VPS: enkripsi disk Biznet Gio/Hostinger dan enkripsi sisi server NOS tidak terdaftar SRC `[TIDAK DIKETAHUI]`; backup pgBackRest terenkripsi `[SRC-235]` (N-09) | Host, NOS | Konfirmasi penyedia, KF | Tertunda (butir review security-architect) |
| Minimisasi data keluar | Payment gateway, rekanan, notifikasi minimal; E-17 (ADR-013 butir 7, ADR-014 butir 9); email tanpa data kesehatan rinci (M-10) | `rs-backend` | RK, VS-1 | Dirancang; M-10 terbuka |
| Data di perangkat | Isi klinis tidak persisten; antrean offline terenkripsi; token di flutter_secure_storage `[SRC-136]`; kecualikan dari cadangan dan sembunyikan layar di latar belakang — mekanisme platform `[TIDAK DIKETAHUI]` (ADR-021 butir 1, KEPUTUSAN-76) | Flutter | VS-6, uji perangkat | Tertunda Tahap 8 |
| Cache respons | Usulan U-03: respons API dan halaman yang memuat data pribadi memakai header larangan simpan cache; halaman verifikasi resep tidak di-cache (ADR-012 butir 3) | `rs-backend`, BFF, Caddy | VS-3 (pemeriksaan header), T10 | Tertunda Tahap 8 |
| Valkey | Hanya data turunan tanpa data pribadi selain ID; token sesi BFF terenkripsi (ADR-007 butir 6, ADR-011 butir 7) | Valkey, BFF | RK, T10 | Dirancang |
| Retensi dan pemusnahan | Tabel kebijakan retensi, crypto-shredding, pekerja privasi (ADR-013 butir 5–6); angka retensi `[TIDAK DIKETAHUI]` (PT-24); Loki retensi per tenant dengan compactor diaktifkan (KEPUTUSAN-80 `[SRC-236]`) | `rs-backend`, PG, Ops | VS-1, KF | Dirancang (PT-24 terbuka) |
| Backup dan salinan penyedia | pgBackRest repo1 NOS + repo2 Hostinger, terenkripsi, retensi 35 hari (KEPUTUSAN-78, `[USER] T4-G2`). Backup mingguan gratis Hostinger `[SRC-88]` dan snapshot Biznet `[SRC-95]` dapat menyimpan salinan VM di luar enkripsi pgBackRest dan di luar kebijakan retensi/crypto-shredding (N-10) | Host | KF, konfirmasi penyedia | Tertunda Tahap 8 |
| Monitoring tanpa PII | GlitchTip tanpa data pribadi, `sendDefaultPii` mati, default `false` `[SRC-255]` (KEPUTUSAN-76, -80 `[SRC-150]`); uji lalu lintas build rilis (ADR-021 butir 2) | Flutter, Ops | T12, VS-6 | Dirancang; metode uji lalu lintas Tertunda |

**Celah terbuka V14:** (1) enkripsi lapis penyedia VPS dan NOS (N-09); (2) salinan backup/snapshot penyedia (N-10); (3) mekanisme platform pengecualian cadangan dan penyembunyian layar; (4) metode uji lalu lintas jaringan build rilis (M-02) belum ditetapkan; (5) M-10.

### V15 Secure Coding and Architecture

**Ruang lingkup:** arsitektur modular dan batas modul, dokumentasi arsitektur dan threat model, praktik kode aman Rust/TypeScript/Dart, pengelolaan dependensi, konkurensi.

| Area kontrol | Kontrol desain | Komponen | Verifikasi | Status |
| --- | --- | --- | --- | --- |
| Batas modul | Modul Rust per modul bisnis, `pub(crate)`, tabel milik modul hanya dari repository modul; test arsitektur nama tabel per modul (KEPUTUSAN-74, ADR-006) | `rs-backend` | VS-4, RK | Dirancang |
| Disiplin kode | `cargo clippy -D warnings`, larangan `unwrap()` di jalur produksi, `cargo fmt`; `tsc --noEmit`; `flutter analyze` (KEPUTUSAN-82) | Semua | VS-1, VS-4, VS-5, VS-6 | Dirancang |
| Pustaka kritis bersama | `audit`, `crypto`, `authz` sebagai crate bersama RS dan RME, tanpa ketergantungan runtime (KEPUTUSAN-74, ADR-006 butir 2); modul T1 wajib review penuh (`CLAUDE.md`) | crate bersama | RK (security, DBA) | Dirancang |
| Dependensi | Paket diverifikasi di registry sebelum ditambah (anti-halu §5); pustaka 0.x atau jarang rilis dipantau: openapi-fetch `[SRC-132]`, web_socket_channel `[SRC-142]`, keycloak-config-cli `[SRC-122]` (KEPUTUSAN-75, -76, -85); `time` dipakai, `chrono` tidak (KEPUTUSAN-74) | Pipeline | CI, RK | Dirancang |
| Dokumentasi keamanan | Threat model, ADR, checklist ini; pembaruan threat model untuk CR-012 (§3.4) | `docs/` | Review gate | Dirancang; pembaruan threat model diusulkan |

**Celah terbuka V15:** (1) threat model belum diperbarui untuk topologi VPS (§3.4); (2) checklist code review per modul T1 belum ditulis (coding standards); (3) dependensi berstatus "dipantau" perlu pemicu evaluasi ulang tertulis.

### V16 Security Logging and Error Handling

**Ruang lingkup:** audit trail, application log, security log; log audit OpenBao; event Keycloak; galat API; pemantauan dan peringatan; sinkronisasi waktu.

| Area kontrol | Kontrol desain | Komponen | Verifikasi | Status |
| --- | --- | --- | --- | --- |
| Audit trail | Pustaka audit, outbox atomik, relay terpisah, DB audit append-only dengan trigger penolak, hash berantai, WORM (ADR-009 `[SRC-40]` `[SRC-99]`); masking di pustaka dengan test (ADR-009 butir 7) | crate `audit`, `audit-relay`, PG (b), NOS | VS-1, VS-2 | Dirancang |
| Pemisahan log | Loki tenant `app` 30 hari dan `security` 1 tahun lewat `X-Scope-OrgID`; akses tenant `security` hanya peran keamanan (KEPUTUSAN-80 `[SRC-236]`, ADR-010). Karena header hanya mengidentifikasi tenant `[SRC-236]`, Loki (push dan query), Prometheus, Grafana, dan GlitchTip ditempatkan di belakang login ber-MFA dan hanya dapat dicapai dari host yang diizinkan (ADR-025); perangkat autentikasinya dipilih Tahap 8 | Ops | KF, T10 | Dirancang (perangkat Tahap 8) |
| Redaksi | Daftar tolak (password, token, `Authorization`, cookie, NIK, rekening, body, nilai sesi, parameter tanda tangan URL, kode verifikasi resep) dan test pola (ADR-010 butir 1, 5 `[SRC-83]`, M-31 d) | Semua proses | VS-4 | Dirancang |
| Log akses manusia ke infrastruktur | Audit device OpenBao → security log, string di-hash HMAC-SHA256 bersalt (KEPUTUSAN-79 `[SRC-226]`). Jalur `sys/seal`, `sys/unseal`, `sys/init` tidak diaudit `[SRC-226]` → usulan U-13; peringatan cakupan HMAC dibaca penuh sebelum diandalkan | OpenBao, Ops | KF, review prosedur | Dirancang; U-13 Tertunda |
| Event IdP | Event Keycloak disimpan 7 hari, diambil tiap menit ke outbox audit dan security log; peringatan > 5 menit (ADR-011 butir 10 `[SRC-73]`) | `worker`, Keycloak | VS-1 | Dirancang |
| Galat | Envelope dengan `correlation_id`, tanpa stack trace (ADR-023 butir 1, 5) | crate `envelope` | VS-3 | Dirancang |
| Peringatan | Metrik dan aturan (laju galat, umur outbox, jeda relay, keterlambatan event Keycloak, sertifikat, disk, replikasi, umur backup); Alertmanager ke email lewat SES; pemeriksa ketersediaan dari Hostinger (KEPUTUSAN-80); penguncian beruntun nakes/admin (M-14) dan volume E-05 tak wajar (H-02) terbuka | Ops | KF, uji peringatan | Dirancang; satu kanal peringatan (N-17) |
| Waktu | NTP (chrony) di setiap host; sumber dan perangkat diverifikasi Tahap 8 (KEPUTUSAN-78, menggantikan layanan waktu penyedia di ADR-009 butir 6) | Host | KF, metrik selisih `recorded_at − occurred_at` (ADR-009 butir 6) | Tertunda Tahap 8 |

**Celah terbuka V16:** (1) pemilihan perangkat login ber-MFA di depan observability (keputusan sudah ada, ADR-025); (2) operasi unseal tidak tercatat audit OpenBao (U-13); (3) peringatan hanya lewat email SES (N-17); (4) M-14, H-02; (5) sumber NTP.

### V17 WebRTC

**Ruang lingkup:** panggilan video konsultasi lewat LiveKit (SFU) dan TURN bawaan di host `video` Biznet Gio; token bergabung; larangan rekaman.

| Area kontrol | Kontrol desain | Komponen | Verifikasi | Status |
| --- | --- | --- | --- | --- |
| Lokasi media | LiveKit dijalankan sendiri di Biznet Gio (Indonesia `[SRC-91]`); LiveKit Cloud ditolak karena tanpa region Indonesia `[SRC-217]` (KEPUTUSAN-87, `[USER] T4-Q1`) | LiveKit, Host | KF | Dirancang |
| TURN | TURN bawaan diaktifkan (default `enabled: false`), mendukung TLS dan port 3478 atau 443 (`[SRC-126]` `[SRC-219]`) | LiveKit | KF | Dirancang; konfigurasi port Tertunda |
| Akses ruang | Token per peserta per sesi, berumur pendek, jendela sesi, ruang ditutup saat durasi habis (ADR-016 butir 4); batas dua peserta di konfigurasi (H-14) — opsi LiveKit belum terverifikasi | `rs-backend`, LiveKit | VS-1 | Dirancang; H-14 Tertunda |
| Tanpa rekaman | Egress tidak dipasang `[SRC-220]`; test konfigurasi (US-306 AC4, KEPUTUSAN-87); pemeriksaan berkala tidak ada berkas rekaman (H-14) | LiveKit | KF, VS-1 | Dirancang |
| Kredensial API | Kunci/rahasia API LiveKit di OpenBao KV (KEPUTUSAN-79, -88); M-26 terpenuhi karena fitur rekam tidak dipasang | LiveKit, OpenBao | KF | Dirancang |
| Enkripsi media | Perilaku enkripsi media dan apakah SFU mendekripsi media `[TIDAK DIKETAHUI]` (tidak ada SRC) | LiveKit, Flutter | T10 | Tertunda Tahap 8 |
| Paparan jaringan | Host `video` harus menerima lalu lintas WebRTC/TURN dari internet; tidak tercantum di aturan firewall KEPUTUSAN-78 (§6 butir 1); kapasitas 100 sesi `[TIDAK DIKETAHUI]`, uji beban (KEPUTUSAN-78) | Host, LiveKit | KF, T10 (pemindaian port), uji beban | Tertunda Tahap 8 |

**Celah terbuka V17:** (1) port publik host video; (2) batas dua peserta (H-14); (3) enkripsi media; (4) satu host video tanpa redundansi (N-15).

---

## 3. Mitigasi threat model yang diserahkan ke Tahap 5

Kolom "Status Tahap 5": **Dijawab** (stack/hosting baru memberi mekanisme terverifikasi), **Sebagian**, **Tertunda Tahap 8**, **Terbuka** (butuh keputusan user, CR, atau bukan wewenang stack). Status threat model sendiri tidak diubah oleh dokumen ini; pembaruannya ada di §3.4 sebagai usulan.

### 3.1 Mitigasi disarankan §6.2 (status "Terbuka", "Baru", atau "Sebagian diterapkan")

| ID | Ancaman | Jawaban di stack/hosting baru | Status Tahap 5 |
| --- | --- | --- | --- |
| M-10 | API-05, DB-03 | Bukan keputusan stack. Templat email lewat SES (`[USER] T5-Q4`); teks di Tahap 6. Isi email tersimpan di AWS region Jakarta dan kotak surat penerima `[SRC-84]` | Terbuka (Tahap 6, DPIA) |
| M-11 | API-07 | Mekanisme tersedia: penghitung Valkey `[SRC-75]` + batas per IP Caddy (modul Tahap 8). Nilai batas = keputusan bisnis | Terbuka (CR ke user) |
| M-12 | API-10 | Bukan keputusan stack; teks pendaftaran native (ADR-011 butir 6) | Terbuka (kemungkinan CR kecil) |
| M-14 | KC-07 | Aturan peringatan di Alertmanager dari event Keycloak di security log (KEPUTUSAN-80); prosedur buka kunci belum ada. Tanpa WAF terkelola, batas laju login hanya Caddy + brute force Keycloak (risiko berubah, §3.4) | Sebagian |
| M-16 | WEB-07 | Pemberitahuan lewat modul Notifikasi (email SES); tidak tergantung stack | Terbuka (revisi ADR-012 butir 7) |
| M-18 | WEB-10 | Penetral CSV di `rs-backend`, test unit di VS-1 | Tertunda Tahap 8 (coding standards) |
| M-20 | INT-06 | Rahasia HMAC terenkripsi per wadah (ADR-013); rahasia klien di Keycloak; rotasi KEK lewat Transit `[SRC-222]`. Prosedur tumpang tindih dua rahasia belum ditulis | Sebagian |
| M-21 | OS-04 | Salin ke kunci final di NOS (S3-compatible `[SRC-97]`) setelah validasi; kemampuan URL unggah NOS `[TIDAK DIKETAHUI]` `[SRC-98]` | Tertunda Tahap 8 |
| M-22 | DB-05, AUD-01 | Job harian di `worker` (crate `platform-jobs`); tidak tergantung stack | Terbuka (revisi ADR-009 butir 5) |
| M-24 | RME-04 | Ditegakkan lewat fungsi/kebijakan di `db/roles/` RME (KEPUTUSAN-89) | Terbuka (revisi ADR-012 butir 6) |
| M-25 | KMS-03 | Core dump proses kontainer dimatikan (U-06); berlaku juga untuk OpenBao di bao-1..3 karena KEK kini di memori VM (risiko berubah, §3.4) | Tertunda Tahap 8 |
| M-26 | VID-04 | LiveKit dijalankan sendiri, Egress tidak dipasang `[SRC-220]` (KEPUTUSAN-87) | **Dijawab** |
| M-27 | SS-02 | Tidak tergantung stack; rahasia SATUSEHAT terenkripsi per wadah | Terbuka (revisi ADR-020 butir 1) |
| M-28 | SS-04 | Tidak tergantung stack | Terbuka (revisi ADR-020) |
| M-29 | CI-05 | Kunci penandatangan aplikasi mobile: ADR-021 butir 4 menyebut secret manager → kini OpenBao (KEPUTUSAN-79); proses penandatanganan rilis mobile belum dirancang | Sebagian (Tahap 12) |
| M-30 | EXT-02 | Lalamove: autentikasi API HMAC-SHA256 `[SRC-216]`; verifikasi keaslian webhook `[TIDAK DIKETAHUI]`; U-07 konfirmasi status lewat API | Tertunda Tahap 8 (PT-37) |
| M-31 | RX-01 | (a) RNG aws-lc-rs ≥ 128 bit (KEPUTUSAN-74); (c) batas laju global di Valkey + peringatan; (d) redaksi di log aplikasi dan log akses Caddy; (e) header lewat U-03, mekanisme Tahap 8 | Sebagian |
| M-32 | RX-02 | Domain verifikasi dan prosedur operasional; tidak tergantung stack | Terbuka (disarankan sebelum go-live) |
| M-33 | RX-03 | Risk register; tidak tergantung stack | Terbuka |

### 3.2 Penguatan §6.3 (status "Terbuka")

| ID | Ancaman | Jawaban di stack/hosting baru | Status Tahap 5 |
| --- | --- | --- | --- |
| H-01 | APP-01 | "Revoke Refresh Token" diaktifkan `[SRC-231]` (KEPUTUSAN-85); opsi pemakaian ulang 0 dan pencabutan rantai `[TIDAK DIKETAHUI]`; "keluar dari semua perangkat" lewat Admin REST `[SRC-72]` belum diputuskan | Sebagian |
| H-02 | API-03, WEB-03 | Aturan peringatan di Alertmanager dari metrik E-05; batasan Verifikator perlu konfirmasi produk | Terbuka |
| H-03 | API-08, WEB-05 | Dipetakan di V1 (sqlx compile-checked, sanitasi CMS), V2 (validasi skema, Schemathesis), V3 (CSP U-03) | Sebagian (CSP dan sanitasi Tertunda Tahap 8) |
| H-04 | APP-04, KC-01, API-15 | Email lewat SES; event Keycloak sudah diambil worker (ADR-011 butir 10). MFA sukarela pasien = keputusan user | Terbuka |
| H-05 | KC-02 | Prosedur; tidak tergantung stack | Terbuka (sebelum go-live) |
| H-07 | WEB-09 | Tidak tergantung stack | Terbuka (konfirmasi produk) |
| H-08 | INT-02, INT-03 | Test isolasi Partner API di VS-1; skenario uji rekanan di sandbox staging (KEPUTUSAN-83) | Tertunda Tahap 8 |
| H-09 | PG-01, PG-04 | DOKU: public key, algoritma `[TIDAK DIKETAHUI]` `[SRC-207]`; idempotensi DOKU tidak terverifikasi (SRC-208 dibatalkan); Xendit: token callback statis `[SRC-199]`; operasi "tanya status" sebagai cadangan (U-07) | Tertunda Tahap 8 (PT-36) |
| H-11 | OS-02 | Kebijakan bucket NOS di `infra/` + test otomatis; perangkat IaC `[TIDAK DIKETAHUI]` | Tertunda Tahap 8 |
| H-12 | RME-02 | Kebijakan RLS gagal-tertutup + test pool di VS-2 (KEPUTUSAN-82 langkah 2) | Tertunda Tahap 8 |
| H-13 | AUD-04 | `tenant_id` diisi crate `audit` dari konteks server; verifikasi RLS pada tabel terpartisi | Tertunda Tahap 8 |
| H-14 | VID-01, VID-02 | Egress tidak dipasang (sebagian terjawab); batas dua peserta di LiveKit belum terverifikasi | Sebagian |
| H-15 | SS-01 | Job di `rme-backend` mencocokkan Organization ID; test lintas fasyankes | Tertunda Tahap 8 |

### 3.3 Butir lain di threat model dan ADR yang menyebut "Tahap 5"

| Butir | Asal | Jawaban di stack/hosting baru | Status Tahap 5 |
| --- | --- | --- | --- |
| Metode uji lalu lintas jaringan build rilis mobile | APP-05, M-02, ADR-021 butir 2 | Belum ditetapkan di KEPUTUSAN-76/81 | **Terbuka** (perlu ditetapkan; diusulkan masuk Tahap 8 Fase 1) |
| Pustaka penyimpanan aman dan kunci aplikasi | ADR-011 butir 7, ADR-021 butir 1 | flutter_secure_storage `[SRC-136]`, local_auth `[SRC-137]` | Dijawab |
| Kecualikan cadangan, sembunyikan layar | ADR-021 butir 1 | Mekanisme platform `[TIDAK DIKETAHUI]` (KEPUTUSAN-76) | Tertunda Tahap 8 |
| Pemisahan endpoint admin Keycloak | KC-06, M-05, ADR-011 butir 2 | Path proxy `[SRC-229]` + `hostname-admin` `[SRC-230]` (KEPUTUSAN-85) | Dijawab |
| Parameter Argon2 | ADR-011 butir 4 | Opsi SPI `[SRC-232]` (KEPUTUSAN-85) | Dijawab |
| Parameter OTP | ADR-011 butir 5 | SHA1/6/30 `[SRC-67]` (KEPUTUSAN-85) | Dijawab (dicek ke teks bab, N-20) |
| Rotasi token penyegar | ADR-011 butir 7 | `[SRC-231]` | Sebagian (opsi reuse) |
| Autentikasi ulang | ADR-011 butir 9 | Parameter OIDC belum terdaftar SRC | Tertunda Tahap 8 |
| Pemetaan jenis event | ADR-011 butir 10 | Ditetapkan per versi saat implementasi (KEPUTUSAN-85) | Tertunda Tahap 8 |
| Konfigurasi realm sebagai kode | ADR-011 konsekuensi | keycloak-config-cli `[SRC-122]` | Dijawab |
| mTLS klien integrasi | INT-01, ADR-014 butir 3 | Tidak dibahas di KEPUTUSAN-74–89 | **Terbuka** (perlu keputusan; lihat §6 butir 4) |
| Mekanisme verifikasi notifikasi PG | PG-01, NFR-INT-04 | Lihat H-09 | Tertunda Tahap 8 |
| Bukti lokasi media video | VID-03 | Server sendiri di Biznet Gio `[SRC-91]` `[SRC-126]` | Dijawab |
| Autentikasi domain pengirim email | EXT-01 | SES Jakarta mencantumkan DKIM domain khusus `[SRC-210]`; SPF/DMARC belum terdaftar SRC | Sebagian |
| Kemampuan IAM, KMS, log audit penyedia | KMS-01, ADR-021 butir 3 | Diganti OpenBao (policy, audit device, SSH CA, DB dinamis) `[SRC-222]`–`[SRC-226]` | Dijawab dengan perubahan model (CR-012; §3.4) |
| Identitas pipeline, proteksi branch, persetujuan deploy | CI-01, CI-06, ADR-021 butir 4 | OIDC GitHub → OpenBao untuk staging; runner staging JIT sekali pakai per job `[SRC-254]` di jaringan staging terpisah (ADR-025); production lewat bastion oleh orang kedua (`[USER] T5-Q8`, KEPUTUSAN-81). Proteksi branch "bila paket mendukung" | Sebagian (review yang tidak dapat dilewati belum terjamin; §6 butir 3) |
| MFA dan break-glass manusia di OpenBao | KMS-01, ADR-021 butir 3 | Login MFA TOTP `[SRC-251]`; root token hanya lewat `generate-root` berkuorum lalu dicabut `[SRC-252]`; persetujuan orang kedua lewat Control Groups `[SRC-253]` (ADR-025) | Dijawab (pemegang share dan orang kedua: PT-38) |
| Penandatanganan image | ADR-021 butir 4 | cosign dengan pasangan kunci `[SRC-154]`, digest `[SRC-242]` | Dijawab (kunci privat di GitHub, N-12) |
| Versi TLS dan cipher | ADR-013 butir 1 | U-05 | Tertunda Tahap 8 |
| Enkripsi lapis penyedia dengan KMS | ADR-013 butir 2 | `[TIDAK DIKETAHUI]` di VPS/NOS | **Terbuka** (N-09; butir review) |
| Rotasi KEK tanpa membungkus ulang | ADR-013 butir 5 | Transit versi + `min_decryption_version` + `rewrap` `[SRC-222]` | Dijawab |
| Prosedur cadangan kunci | ADR-013 konsekuensi, KMS-02 | Snapshot Raft terenkripsi ke NOS (perintah Tahap 12) + share Shamir (PT-38) | Sebagian |
| WORM penyedia | ADR-009 butir 5 | NOS Object Lock Compliance `[SRC-99]` | Dijawab |
| Sinkronisasi waktu | ADR-009 butir 6, ADR-017 butir 5 | NTP sendiri (chrony), diverifikasi Tahap 8 | Tertunda Tahap 8 |
| RLS pada tabel audit terpartisi | ADR-012 butir 9, AUD-04 | Belum diverifikasi | Tertunda Tahap 8 |
| Penyimpanan log | ADR-010 tabel | Loki self-hosted di host `ops` `[SRC-192]` `[SRC-236]` | Dijawab |
| Pembatasan URL bertanda tangan, versi objek | ADR-018 butir 2, 5, konsekuensi | URL GET `[SRC-98]`; PUT/POST dan versi `[TIDAK DIKETAHUI]` | Tertunda Tahap 8 |
| Pustaka penetralan PDF | ADR-018 butir 6 | qpdf `[SRC-155]` (penolakan); CDR tidak tersedia | Dijawab (fallback) |
| Konfigurasi rekaman video | ADR-016 butir 4, US-306 AC4 | Egress tidak dipasang `[SRC-220]` | Dijawab |
| WAF dan batas laju di penyeimbang beban | ADR-019 butir 3; dirujuk APP-04, API-07, KC-07, PG-08, RD-01 (M-15) | Tidak ada WAF terkelola; Caddy + batas laju aplikasi + batas per IP Caddy (modul Tahap 8); OWASP CRS dievaluasi Tahap 8 (KEPUTUSAN-78) | **Sebagian** (risiko berubah, §3.4) |

### 3.4 Ancaman yang risikonya berubah karena CR-012 (butir review security-architect)

Penilaian arah di bawah adalah pendapat penyusun untuk direview; tingkat resmi tetap mengikuti threat model sampai diperbarui.

| Ancaman | Perubahan karena CR-012 / stack | Arah | Usulan tindak lanjut |
| --- | --- | --- | --- |
| **KMS-01** akses manusia ke kunci dan secret | KMS penyedia diganti OpenBao di VM sendiri. Kini dirancang (ADR-025): login manusia ber-MFA TOTP `[SRC-251]`; root token hanya lewat `generate-root` dengan kuorum pemegang share dan dicabut setelah dipakai `[SRC-252]`; akses darurat dengan persetujuan orang kedua lewat Control Groups `[SRC-253]`; observability di belakang login ber-MFA. Sisa: pemegang root OS host bao-1..3; jalur darurat ke host saat OpenBao tersegel; `sys/unseal` tidak diaudit `[SRC-226]`; penyetuju Control Groups tidak diberi tahu otomatis `[SRC-253]` | Naik, sebagian besar tertutup di desain | PT-38; U-13; batasi root OS host bao ke jalur darurat; pemberitahuan penyetuju; review akses berkala |
| **KMS-02** KEK hilang atau tak tersedia | Seal Shamir tanpa auto-unseal `[SRC-228]`: restart node perlu unseal manual; kehilangan share melebihi ambang = data terenkripsi tidak dapat dibuka (KEPUTUSAN-79). Raft 3 node lintas penyedia `[SRC-227]` | Naik | N-01; uji pemulihan termasuk unseal dan dekripsi; dampak RTO |
| **KMS-03** DEK/KEK di memori | KEK kini di memori proses OpenBao di VM yang hypervisor-nya dikelola Biznet Gio/Hostinger; DEK di memori proses aplikasi seperti sebelumnya | Naik | M-25 diperluas ke OpenBao; N-02 |
| **DB-02** pembaca database/backup | Enkripsi tingkat aplikasi tetap. Enkripsi lapis penyedia `[TIDAK DIKETAHUI]`; replika penuh di Hostinger; salinan backup/snapshot penyedia (`[SRC-88]` `[SRC-95]`) | Naik (lapis kedua belum terbukti) | N-09, N-10, U-14 |
| **OS-01 / OS-02 / OS-05** berkas di object storage | NOS: URL GET bertanda tangan ada `[SRC-98]` tetapi contoh memakai http; enkripsi sisi server, versi objek, dan URL unggah tidak terverifikasi; perangkat IaC untuk kebijakan bucket belum dipilih | Naik (belum terbukti) | N-16; H-11; opsi enkripsi berkas oleh aplikasi saat salin ke kunci final (§6 butir 5) |
| **APP-04, API-07, KC-07, PG-08** banjir permintaan dan credential stuffing | Mitigasi menyebut "WAF dan batas laju" di penyeimbang beban (ADR-019 butir 3). Kini tanpa WAF terkelola; batas per IP Caddy belum terverifikasi; alamat IP klien lewat LB NEO `[TIDAK DIKETAHUI]`; anti-DDoS jaringan lewat Magic Transit `[SRC-95]` | Naik | N-03, N-04; evaluasi OWASP CRS Tahap 8; M-14 |
| **RD-01** Redis/Valkey tidak tersedia | Cadangan batas laju "beralih ke penyeimbang beban" (ADR-007 butir 6, M-15) tidak lagi tersedia; Valkey satu host `cache` | Naik | Cadangan diganti batas per IP Caddy (bila modul tersedia) |
| **RD-02** Valkey diracuni/dibaca | Pembatasan "hanya subnet proses aplikasi" sulit dipenuhi karena proses RS dan RME berbagi host app-1/app-2 dan kedua instans Valkey berbagi host `cache` (KEPUTUSAN-78); TLS Valkey belum terverifikasi | Naik | Autentikasi per instans, jaringan Docker terpisah (U-06), TLS antar-host (Tertunda Tahap 8); N-06 |
| **CI-01** pipeline terkompromi | Membaik: production tanpa runner dan kredensial di GitHub; migrasi dengan kredensial dinamis `[SRC-224]`; runner staging JIT sekali pakai `[SRC-254]` di jaringan terpisah dan firewall production berupa daftar izin per host:port (ADR-025). Tersisa: kunci privat cosign di secret GitHub dan staging deploy otomatis; review yang tidak dapat dilewati di `main` bergantung paket GitHub (`[SRC-240]` untuk environment; ketersediaan proteksi branch repo privat tidak terdaftar SRC) | Berubah; tersisa N-12 dan §6 butir 3 | N-12; U-11; §6 butir 3 |
| **CI-06** (ancaman baru, review keamanan Tahap 5) runner staging menjadi pijakan ke jaringan production | Runner JIT sekali pakai per job `[SRC-254]`; staging di jaringan terpisah dari production; firewall production daftar izin per host sumber dan port; Loki, Prometheus, Grafana, GlitchTip di belakang login ber-MFA (ADR-025) | Tertutup di desain; dibuktikan Tahap 8 | Uji jangkauan dari host staging ke port production (T10); verifikasi kemampuan pemisahan jaringan Biznet Gio |
| **KMS-05** rahasia di repo/CI | Satu rahasia kini sengaja disimpan di GitHub (kunci cosign) | Naik sedikit | Akses minimum, rotasi kunci cosign tertulis |
| **KC-06** endpoint admin Keycloak | Pembatasan kini sepenuhnya di Caddy milik sendiri (path) + `hostname-admin` `[SRC-229]` `[SRC-230]`, tanpa lapisan penyeimbang beban terkelola | Tetap, bergantung konfigurasi | KF + uji dari internet di Tahap 10 |
| **KC-09** patch Keycloak terlambat | Kini juga patch OS, Docker, PostgreSQL, Valkey, OpenBao, Caddy, LiveKit, Loki/Grafana menjadi tanggung jawab tim tanpa DevOps (PT-35) | Naik | N-07 |
| **AUD-02 / AUD-03** manipulasi audit | WORM tetap ada lewat NOS Object Lock Compliance `[SRC-99]`; waktu dari NTP sendiri (belum terverifikasi); kredensial penulis kepala rantai di OpenBao | Tetap (dengan syarat NTP) | Sumber NTP Tahap 8; kredensial NOS khusus kepala rantai |
| **VID-03** media di luar Indonesia | LiveKit dijalankan sendiri di Biznet Gio | Turun | — |
| **VID-04** rekaman lewat kredensial API | Egress tidak dipasang `[SRC-220]` | Turun | — |
| **VID-05** kapasitas video | Satu host `video`, kapasitas 100 sesi `[TIDAK DIKETAHUI]`; host harus terbuka ke internet untuk WebRTC/TURN | Naik | N-14, N-15; uji beban |
| **Seluruh TB-1** (baru) metadata lalu lintas | Lalu lintas dapat dirutekan lewat jaringan Cloudflare (Magic Transit) dengan lokasi `[TIDAK DIKETAHUI]`; isi tetap terenkripsi TLS ke Caddy | Baru | N-05; butir gate T4-Q1, PT-39 |
| **TB-13** akses administratif | IAM penyedia diganti bastion + sertifikat SSH OpenBao (`[USER] T5-Q8`); konsol penyedia VPS (Biznet Gio, Hostinger) tetap ada sebagai jalur administratif yang belum dibahas (MFA konsol penyedia `[TIDAK DIKETAHUI]`) | Berubah | Tambahkan akun konsol penyedia ke ADR-021 butir 3 (MFA, pemegang, log) |

Usulan: threat model diperbarui ke v1.1 untuk topologi VPS (TB-1, TB-8, TB-10, TB-13 dan komponen baru OpenBao, Caddy, bastion, host `ops`, WireGuard, Magic Transit) sebelum Tahap 8 Fase 1.

---

## 4. Celah dan risiko keamanan baru akibat CR-012 dan stack (untuk risk register)

Daftar ini diserahkan ke orkestrator untuk dimasukkan ke `docs/04-risk-register.md`; dokumen ini tidak menulis risk register. Kemungkinan dan dampak dinilai oleh pemilik risk register.

| ID | Celah/risiko | Sumber | Usulan mitigasi |
| --- | --- | --- | --- |
| N-01 | Kehilangan share unseal OpenBao melebihi ambang membuat data terenkripsi tidak dapat dibuka; node yang restart tidak melayani sampai di-unseal manual (dampak RTO) | KEPUTUSAN-79 `[SRC-228]` | PT-38; prosedur unseal dan uji pemulihan; pemegang share di lokasi berbeda |
| N-02 | KEK disimpan perangkat lunak di VM (tanpa HSM), terpapar pada pemegang root host dan pengelola hypervisor penyedia. Klaim sertifikasi Biznet Gio berasal dari siaran pers, ruang lingkup tidak dibuka `[SRC-103]` | KEPUTUSAN-79 | Batasi root host bao; core dump mati; konfirmasi ruang lingkup sertifikasi penyedia |
| N-03 | Tanpa WAF terkelola; perlindungan lapis aplikasi bergantung Caddy, validasi masukan, dan batas laju aplikasi; modul batas laju Caddy dan OWASP CRS belum terverifikasi | KEPUTUSAN-78 | Evaluasi Tahap 8 Fase 1; bila tidak tersedia, risiko diterima user |
| N-04 | Alamat IP klien setelah LB NEO (listener TCP) `[TIDAK DIKETAHUI]`; bila hilang, batas laju per IP, `audit_event.source_ip`, dan security log mencatat alamat LB | `[SRC-102]`, PT-39 | U-10 |
| N-05 | Perutean Magic Transit lewat Cloudflare dengan lokasi `[TIDAK DIKETAHUI]` (metadata koneksi; isi terenkripsi) terhadap T4-Q1 "tanpa kecuali" | `[SRC-95]`, KEPUTUSAN-78 | Butir gate user; konfirmasi tertulis Biznet Gio (PT-39) |
| N-06 | Proses RS, RME, Keycloak, dan tiga BFF berbagi host app-1/app-2; kedua Valkey berbagi host `cache`. Lolos dari satu kontainer dapat menembus batas sistem RS–RME (ADR-006 butir 4) | KEPUTUSAN-78 | U-06; evaluasi host terpisah untuk RME bila jawaban L-05 menuntut pemisahan |
| N-07 | Patch dan pengerasan OS, Docker, PostgreSQL, Valkey, OpenBao, Keycloak, Caddy, LiveKit, Loki/Grafana dijalankan tim sendiri tanpa DevOps; pemelihara setelah go-live belum ada | CR-012, `[USER] T5-Q7`, PT-35 | SLA patch tertulis per komponen; pemantauan advisory; PT-35 |
| N-08 | Riwayat advisory Next.js (dua Critical RCE, beberapa Moderate termasuk penerusan respons pada self-hosted) pada komponen BFF yang memegang token | `[SRC-246]`, KEPUTUSAN-75 | Pengerasan KEPUTUSAN-75; patch ≤ 7 hari. GHSA-pc58-hwjq-p385 ditambal sejak 16.4.5, versi dipin 16.4.6 tidak terdampak `[SRC-248]` |
| N-09 | Enkripsi lapis penyedia untuk disk VPS dan NOS tidak terverifikasi, padahal ADR-013 butir 2 dan ADR-018 butir 5 mengandalkannya; foto KTP/KK dan PDF klinis di NOS tidak dienkripsi aplikasi | ADR-013, ADR-018, `[SRC-97]` | §6 butir 5 (pilihan untuk security-architect/user) |
| N-10 | Backup/snapshot tingkat penyedia (Hostinger backup mingguan gratis, snapshot Biznet) dapat menyimpan salinan host DB/OpenBao di luar enkripsi pgBackRest, di luar retensi 35 hari dan crypto-shredding | `[SRC-88]` `[SRC-89]` `[SRC-95]` | U-14 |
| N-11 | Replikasi lintas penyedia lewat WireGuard (perangkat belum terverifikasi); failover manual; replika penuh di Hostinger | KEPUTUSAN-78 | Verifikasi Tahap 8 Fase 1; runbook failover |
| N-12 | GitHub menjadi akar kepercayaan rantai pasok: kunci privat cosign di secret GitHub, staging deploy otomatis; image bertanda tangan yang jahat dapat lulus staging. Gerbang production bergantung pada pemeriksaan orang kedua | KEPUTUSAN-81 `[SRC-154]` | U-11; rotasi kunci cosign; akses minimum secret |
| N-13 | **Tertutup di desain.** Runner self-hosted di stg-app menjalankan kode workflow; GitHub mengingatkan kehati-hatian juga untuk repo privat `[SRC-254]`. Kini: runner JIT sekali pakai per job `[SRC-254]`, staging di jaringan terpisah, firewall production daftar izin per host:port (ADR-025) | KEPUTUSAN-81, ADR-025 | U-12; pembuktian Tahap 8 |
| N-14 | Host `video` harus menerima WebRTC/TURN dari internet; aturan firewall KEPUTUSAN-78 hanya menyebut LB 443 dan bastion 22 | KEPUTUSAN-78, -87 `[SRC-126]` | §6 butir 1 |
| N-15 | Satu host `video` tanpa redundansi; kapasitas 100 sesi `[TIDAK DIKETAHUI]` | KEPUTUSAN-78 | Uji beban NFR-CAP-03/04; rencana host kedua bila perlu |
| N-16 | Endpoint NOS HTTPS dan URL unggah bertanda tangan belum terverifikasi (contoh resmi memakai http, hanya `get_object`) | `[SRC-98]` | Verifikasi Tahap 8 Fase 1 sebelum EP berkas |
| N-17 | Peringatan hanya lewat email SES; gangguan akun AWS/SES atau host `ops` menghilangkan peringatan; satu host `ops` menyimpan security log | KEPUTUSAN-80 | Kanal peringatan kedua; pemeriksa eksternal di Hostinger memantau host `ops` |
| N-18 | **Tertutup di desain.** Tenant Loki hanya diidentifikasi lewat `X-Scope-OrgID` `[SRC-236]`; kini Loki, Prometheus, Grafana, dan GlitchTip di belakang login ber-MFA (ADR-025) | `[SRC-236]`, ADR-025 | Perangkat login dipilih Tahap 8 |
| N-19 | Keaslian notifikasi DOKU (algoritma `[TIDAK DIKETAHUI]`), Xendit (token statis), dan webhook Lalamove (`[TIDAK DIKETAHUI]`) | `[SRC-207]` `[SRC-199]` `[SRC-216]` | U-07; PT-36, PT-37 |
| N-20 | TOTP memakai SHA1 demi kompatibilitas; dokumentasi Keycloak menyebut SHA256/SHA512 lebih aman | `[SRC-67]`, KEPUTUSAN-85 | Bandingkan dengan teks bab V6/V11 di Tahap 8 |
| N-21 | Enkripsi backup pgBackRest hanya `aes-256-cbc` (tanpa integritas bawaan mode) | `[SRC-235]` | Bandingkan dengan teks bab V11; integritas backup lewat uji pemulihan |
| N-22 | Operasi `sys/unseal`, `sys/seal`, `sys/init` tidak tercatat di log audit OpenBao; cakupan hash HMAC punya peringatan. Root token kini hanya lewat `generate-root` berkuorum dan dicabut setelah dipakai `[SRC-252]` (ADR-025), sehingga yang tersisa adalah pencatatan unseal | `[SRC-226]` | U-13 |
| N-23 | Ketergantungan melingkar: akses SSH ke host diterbitkan OpenBao. Persetujuan orang kedua untuk akses darurat kini lewat Control Groups `[SRC-253]` (ADR-025), tetapi mekanisme itu juga butuh OpenBao yang ter-unseal; akses darurat ke host saat OpenBao tersegel tetap butuh jalur lain | KEPUTUSAN-79, ADR-025 | Prosedur akses host saat OpenBao tersegel sebelum go-live (PT-38) |
| N-24 | Sinkronisasi waktu dengan NTP sendiri belum terverifikasi; memengaruhi stempel audit, masa token, TOTP, dan stempel webhook 5 menit | KEPUTUSAN-78 | Verifikasi sumber NTP Tahap 8; metrik selisih jam |
| N-25 | Lalu lintas antar-node klaster Keycloak dan TLS internal PG/Valkey belum terverifikasi | KEPUTUSAN-77, -85 | Verifikasi Tahap 8 |
| N-26 | Konsol penyedia VPS (Biznet Gio, Hostinger) adalah jalur administratif yang dapat mematikan, menyalin snapshot, atau mengakses konsol VM; MFA dan pencatatannya `[TIDAK DIKETAHUI]` | CR-012 | Masukkan ke ADR-021 butir 3: MFA, pemegang minimal, tinjauan berkala |

---

## 5. Usulan penyusun (U-xx)

Usulan ini bukan keputusan. Bila diterima, orkestrator mencatatnya sebagai KEPUTUSAN di `docs/decision-log.md` dan membawanya ke gate.

| ID | Usulan | Alasan/rujukan |
| --- | --- | --- |
| U-01 | Di Tahap 8 Fase 1, teks requirement ASVS 5.0.0 per bab dibuka dari repositori resmi (`[SRC-245]`), didaftarkan sebagai SRC, lalu tabel ini diperluas per requirement (ID, teks ringkas, kontrol, bukti); verifikasi di Tahap 10 | ADR-011 butir 1; batas dokumen ini |
| U-02 | Coding standards memuat: query hanya lewat parameter terikat sqlx; field tak dikenal ditolak di permintaan; proses eksternal tanpa shell; cargo-deny melarang crate kripto selain aws-lc-rs | V1, V2, V11; KEPUTUSAN-74 |
| U-03 | Satu set header keamanan web (CSP, larangan framing, HSTS, `nosniff`, `Referrer-Policy`) dan header larangan cache untuk respons berdata pribadi, ditetapkan di satu tempat; mekanisme diverifikasi Tahap 8 | H-03, M-31 e, V3, V14 |
| U-04 | Batas ukuran pesan dan laju pesan per koneksi WebSocket | V4; API-07 |
| U-05 | TLS eksternal minimum 1.2 dengan 1.3 diutamakan, final setelah teks bab V12 dibuka | ADR-013 butir 1 |
| U-06 | Kontainer non-root, root filesystem baca-saja bila memungkinkan, socket Docker hanya untuk agen deploy, jaringan Docker terpisah RS/RME, core dump dimatikan untuk proses aplikasi dan OpenBao | M-25, N-06, V13 |
| U-07 | Bila verifikasi notifikasi pihak ketiga tidak kriptografis atau belum terbukti, status dikonfirmasi lewat API "tanya status" sebelum mengonfirmasi pesanan, refund, atau status kirim | H-09, M-30, N-19 |
| U-08 | SBOM per rilis untuk backend, web, dan mobile sebagai bentuk inventaris dependensi; perangkat dipilih Tahap 8 | ADR-021 butir 2, V13 |
| U-09 | Threat model diperbarui ke v1.1 untuk topologi VPS sebelum Tahap 8 Fase 1 | §3.4 |
| U-10 | Alamat IP klien wajib sampai ke Caddy (mekanisme seperti PROXY protocol pada LB NEO, bila didukung); bila tidak, opsi tanpa LB NEO dievaluasi | N-04, PT-39 |
| U-11 | Checklist gerbang production untuk orang kedua: digest ↔ commit ↔ hasil `verify.sh` hijau ↔ hasil staging; daftar migrasi yang menyentuh audit/role/grant/trigger/RLS beserta bukti review DBA dan keamanan | N-12, ADR-021 butir 4 |
| U-12 | Runner staging hanya untuk workflow yang dipicu push ke `main`, tidak untuk pull request (melengkapi runner JIT sekali pakai `[SRC-254]` yang sudah masuk ADR-025); mekanisme diverifikasi ke dokumentasi GitHub di Tahap 8 | N-13, `[SRC-241]` `[SRC-254]` |
| U-13 | Unseal, pembuatan root token, dan akses darurat dicatat manual (siapa, kapan, alasan, saksi) ke security log karena tidak tercatat di audit OpenBao | N-22, `[SRC-226]` |
| U-14 | Backup/snapshot tingkat penyedia dimatikan untuk host DB dan OpenBao, atau dimasukkan ke kebijakan retensi dan enkripsi; kemampuan mematikannya diverifikasi ke penyedia | N-10 |

---

## 6. Butir untuk review security-architect dan gate

1. **Inkonsistensi aturan firewall (KEPUTUSAN-78).** Pernyataan "hanya LB (443) dan bastion (22, dibatasi) yang publik" tidak memuat host `video`, padahal WebRTC dan TURN bawaan LiveKit memerlukan port yang dapat dicapai klien dari internet (TURN 3478 atau 443 `[SRC-126]`). Perlu dikoreksi di `docs/05-stack.md` dengan daftar port host `video` setelah diverifikasi Tahap 8.
2. **Tafsiran ADR-013 butir 1 ("semua lalu lintas internal memakai TLS").** Di topologi VPS, Caddy, BFF, dan proses Rust berada di host yang sama dan berkomunikasi lewat jaringan Docker. Perlu keputusan: TLS juga di dalam host, atau TLS wajib antar-host saja dengan jaringan Docker terisolasi di dalam host. Bila yang kedua, ADR-013 butir 1 perlu diperjelas lewat ADR-025.
3. **ADR-021 butir 4 ("review yang tidak dapat dilewati" sebelum `main`).** KEPUTUSAN-81 menjadikan proteksi branch opsional "bila paket mendukung". Dalam alur proyek, merge ke `main` dilakukan Claude setelah `verify.sh` hijau dan review subagent. Gerbang manusia hanya ada di production (orang kedua mendeploy digest). Perlu dinilai apakah ini memenuhi ADR-021 butir 4 dan `[SRC-82]`, atau dicatat sebagai penyesuaian di ADR-025 dengan U-11 sebagai kompensasi.
4. **mTLS klien integrasi (ADR-014 butir 3)** belum diputuskan di Tahap 5. Usulan penyusun: tidak di rilis 1 (client credentials berumur pendek + batas laju + pencatatan), dievaluasi ulang setelah PT-05. Keputusan ini perlu dicatat.
5. **Enkripsi berkas di NOS (N-09).** Pilihan untuk dibawa ke user: (a) konfirmasi tertulis Biznet Gio tentang enkripsi sisi server NOS; (b) aplikasi mengenkripsi berkas dengan DEK wadahnya saat menyalin ke kunci final (bergabung dengan M-21), dengan konsekuensi unduhan tidak lagi lewat URL bertanda tangan langsung dan mengubah ADR-018 butir 3; (c) menerima risiko. Untuk disk VPS: konfirmasi penyedia atau enkripsi disk di tingkat OS (yang menambah langkah buka kunci saat boot).
6. **Metode uji lalu lintas jaringan build rilis mobile (M-02)** belum ditetapkan, padahal ADR-021 butir 2 menyerahkannya ke Tahap 5.

---

## 7. Status

**v1.1 (2026-11-18, tindak lanjut Checkpoint 2):** V6 dan V7 diperbarui (lihat baris Versi di kepala dokumen). SRC baru yang dirujuk: SRC-275–SRC-277. KEPUTUSAN baru: KEPUTUSAN-202–KEPUTUSAN-205. Ringkasan per bab §1 untuk V6 dan V7 disesuaikan. Review keamanan atas revisi ADR-011 dijalankan sebelum Tahap 8 (`docs/checkpoint-2.md` tindak lanjut 4).

Riwayat sebelumnya: **Draf v0.2** (perbaikan setelah review keamanan `docs/review/tahap-05-security.md`: GHSA-pc58-hwjq-p385, MFA dan break-glass OpenBao, runner staging, jaringan staging terpisah, firewall daftar izin, observability ber-MFA). Seluruh 17 bab sudah dipetakan sampai tingkat bab. Belum ada pemetaan per requirement (U-01). Ada 26 celah/risiko baru (N-01–N-26) untuk risk register (N-13 dan N-18 tertutup di desain; N-22 dan N-23 menyempit), 14 usulan (U-01–U-14) yang perlu dicatat orkestrator bila diterima, dan 6 butir review/gate (§6). Dokumen ini tidak menambah SRC dan tidak mengubah ADR.
