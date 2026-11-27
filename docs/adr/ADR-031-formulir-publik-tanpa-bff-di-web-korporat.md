# ADR-031: Formulir publik tanpa sesi di web korporat memanggil API publik langsung (bukan lewat BFF)

Status: **Diusulkan** (2026-11-27). Arah dipilih user `[USER]` Q30 = C (2026-11-27, Telegram, diteruskan orkestrator); dokumen ADR ini menunggu review T1 dan persetujuan.
Mengubah bila diterima: ADR-027 butir 1 dan butir 6 (amandemen: `web/korporat` juga menyajikan satu halaman publik tanpa sesi, dan Caddy host korporat meneruskan dua rute API publik), ADR-024 butir 3 (pengecualian terbatas: tidak semua panggilan API dari web ber-BFF melewati BFF).
Tidak mengubah: ADR-012 amandemen 2026-11-26 (`PublicReason::CorporateRegistration`, KEPUTUSAN-323), KEPUTUSAN-269 (CSRF synchronizer token untuk semua permintaan bersesi).
Tanggal: 2026-11-27
Tingkat ketelitian: T1 (BFF `web/korporat`, konfigurasi reverse proxy)

## Konteks

- Pendaftaran perusahaan mandiri (US-1901) dilakukan calon HR yang belum punya akun, lewat endpoint publik `POST /v1/public/corporate-registrations` dan `GET /v1/public/corporate-registrations/consent-text` (KEPUTUSAN-323). Backend sudah menerapkan jawaban `202` seragam, penyamaan waktu respons, batas laju, dan persetujuan per versi teks `[DOK: adr/ADR-012-otorisasi-dan-isolasi-data.md]`.
- Halaman formulirnya dirender `web/korporat` (brief web korporat KR-01–KR-04) `[DOK: design/brief-web-korporat.md]`.
- BFF `web/korporat` (`@rs/bff-auth`) hanya meneruskan permintaan bersesi: CSRF berupa synchronizer token yang disimpan di sesi (KEPUTUSAN-269), dan proxy menolak tanpa sesi. Tidak ada pola untuk formulir anonim `[DOK: web/packages/bff-auth/src/routes.ts]`.
- Bila permintaan anonim diteruskan server Next.js, backend melihat alamat peer server Next.js untuk semua pendaftar, sehingga batas laju "per sumber" menjadi batas global. IP klien asli di balik Caddy juga belum terselesaikan (PT-39, KEPUTUSAN-78, TD-22) `[DOK: 05-stack.md]`.
- Tiga opsi diajukan ke user: (A) Route Handler anonim di BFF dengan cek `Origin`/`Sec-Fetch-Site` dan batas laju Valkey; (B) sesi pra-pendaftaran anonim dengan cookie `__Host-` dan synchronizer token; (C) browser memanggil API publik langsung lewat rute Caddy di host korporat. User memilih C `[USER] Q30`.

## Keputusan

1. Halaman `/daftar` di `web/korporat` adalah halaman publik: tanpa penjaga sesi, tanpa cookie, tidak menyentuh Valkey atau OpenBao.
2. Dari browser, formulir memanggil `GET /v1/public/corporate-registrations/consent-text` dan `POST /v1/public/corporate-registrations` **pada asal (origin) web korporat yang sama**, dengan `credentials: "omit"` dan tanpa token CSRF. Tidak lewat Route Handler BFF dan tidak lewat server Next.js.
3. Caddy host korporat meneruskan **hanya** dua rute itu ke API Rust, dicocokkan persis dengan metodenya (POST untuk pendaftaran, GET/HEAD untuk teks persetujuan). Jalur `/v1/*` lain di host korporat (termasuk `/v1/corporate/*` dan `/v1/public/content/*`) tidak diteruskan dan dijawab Next.js. Konfigurasi sebagai kode: `infra/caddy/snippets/korporat-pendaftaran-publik.caddy`.
4. CSP korporat tetap `connect-src 'self'` (ditambah origin unggah dokumen KEPUTUSAN-371 saja). Tidak ada origin API terpisah.
5. CSRF tidak diterapkan pada dua rute ini. Alasannya: tidak ada otoritas ambien (tanpa cookie, tanpa sesi, tanpa header otorisasi), sehingga situs lain yang memicu permintaan dari browser korban tidak mendapat apa pun yang tidak dapat diperolehnya dengan memanggil endpoint publik itu langsung. Badan JSON juga memicu preflight CORS untuk permintaan lintas asal, dan API tidak mengirim header CORS.

## Alternatif yang ditolak

- **(A) Route Handler anonim di BFF**: menambah pola keamanan baru di modul T1 (`@rs/bff-auth`), butuh batas laju sendiri di Valkey dan IP klien tepercaya dari Caddy yang diteruskan ke backend (perubahan backend). Tidak menambah perlindungan berarti dibanding batas laju backend.
- **(B) Sesi pra-pendaftaran anonim**: setiap pengunjung anonim membuat kunci Valkey (permukaan DoS) dan cookie, padahal tidak ada data yang perlu dilindungi sesi.

## Konsekuensi

- Rute Caddy wajib ada di Caddyfile host korporat sebelum formulir dapat dipakai di staging/production. Caddyfile host belum ada di repo (dibuat task deploy); snippet sudah divalidasi dengan `caddy adapt` (caddy 2.11.9).
- **Ketergantungan, bukan bagian ADR ini**: batas laju per IP di backend memakai alamat peer TCP. Di balik Caddy, itu alamat Caddy, sehingga batas per sumber pendaftaran efektif menjadi global sampai backend membaca IP klien asli dari header tepercaya Caddy (PT-39, TD-22, TD-25). Ini dipegang task backend terpisah.
- Pengembangan lokal tanpa Caddy: `next dev` tidak meneruskan `/v1/public/*`, sehingga formulir lokal butuh reverse proxy lokal atau dijalankan lewat test. Tidak ada rewrite Next.js yang ditambahkan, supaya server Next.js tidak menjadi jalur proxy anonim.
- Permukaan server Next.js korporat untuk lalu lintas anonim hanya halaman statis-dinamis `/daftar` (render HTML), tanpa jalur API anonim.
