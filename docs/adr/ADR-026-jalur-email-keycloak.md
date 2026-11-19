# ADR-026: Email Keycloak lewat relay SMTP internal ke Amazon SES API Jakarta

Status: Diusulkan (hasil spike F1-T02, 2026-11-19; menunggu review keamanan dan keputusan user `[KEPUTUSAN-183]`)
Tanggal: 2026-11-19

Rincian temuan, eksperimen, dan perbandingan opsi: `docs/spike/F1-T02-email-keycloak.md`.

## Konteks

- Keycloak mengirim email verifikasi, atur ulang password, dan undangan lewat pengaturan server SMTP per realm `[SRC-264]`. Pengirim bawaan mengirim secara sinkron di dalam request, tanpa antrean `[SRC-301]`. Pada eksperimen E1 (spike §2.3), kegagalan SMTP membuat Admin REST mengembalikan HTTP 500 tanpa coba ulang.
- Badan email Keycloak memuat token aksi dalam bentuk tautan `login-actions/action-token?key=...` (spike §2.3, E1).
- User memilih Amazon SES region Jakarta lewat API, dengan akun AWS khusus email `[USER] T5-Q4`. Endpoint SMTP SES tidak tersedia di Jakarta; endpoint API tersedia `[SRC-84]` `[SRC-294]`. API v2 `SendEmail` menerima pesan MIME mentah (Raw) `[SRC-297]`.
- Semua penyedia memproses data di Indonesia `[USER] T4-Q1` (CR-007). Endpoint SMTP SES di region lain, misalnya Singapura `[SRC-294]`, tidak memenuhi ketentuan ini.
- ADR-016 butir 5 menetapkan email lewat penyedia email atau relay SMTP yang memproses dan menyimpan data di Indonesia `[DOK: adr/ADR-016-konsultasi-realtime-dan-notifikasi.md]`. ADR-024 menetapkan stack Rust, Next.js, dan Flutter `[DOK: adr/ADR-024-stack-rust-nextjs-flutter.md]`.
- Keycloak menyediakan SPI `emailSender` untuk pengirim kustom, tetapi SPI itu internal: dapat berubah tanpa pemberitahuan dan memicu peringatan saat start `[SRC-299]` `[SRC-300]`.
- Alertmanager juga mengirim email lewat SMTP `[SRC-302]`, ke email tim teknis lewat SES `[KEPUTUSAN-80]`.
- Jalur email ini menyentuh autentikasi (modul T1, `CLAUDE.md`) dan merupakan pola baru, sehingga diusulkan sebagai ADR `[KEPUTUSAN-183]`.

## Keputusan

Keycloak (semua realm) dan Alertmanager mengirim email lewat SMTP ke **relay internal buatan sendiri**. Relay meneruskan pesan MIME mentah ke Amazon SES API v2 di `ap-southeast-3`, secara sinkron dan tanpa menyimpan pesan `[KEPUTUSAN-226]`.

1. **Bentuk.** Proses `email-relay` di workspace Rust, sebagai subcommand `apps/rs-backend` mengikuti pola `audit-relay`; letak final ditetapkan di F1-T36. Adaptor SES v2 berada di pustaka bersama dan dipakai juga oleh modul notifikasi (ADR-016 butir 5) `[KEPUTUSAN-228]`.
2. **Protokol.** Relay mengimplementasikan subset SMTP yang cukup untuk Keycloak (EHLO, MAIL FROM, RCPT TO, DATA, QUIT; spike E1), ditambah STARTTLS. Pustaka TLS dan penandatangan SigV4 dipilih di F1-T36 dengan cek registry dan dicatat di `docs/sources.md`. Relay pihak ketiga dan crate server SMTP yang diperiksa tidak dipakai karena tidak aktif dirilis `[SRC-303]` `[KEPUTUSAN-228]`.
3. **Sinkron, tanpa penyimpanan.** Relay membalas sukses hanya setelah SES mengembalikan HTTP 200 dan `MessageId` `[SRC-297]`. Galat SES 429 dan 5xx menjadi balasan SMTP sementara (4xx); galat 400 menjadi balasan permanen (5xx). Pesan hanya ada di memori. Coba ulang dan kirim ulang menjadi tanggung jawab pemanggil Admin REST dan pengguna (US-101 AC2) `[KEPUTUSAN-227]`.
4. **Pembatasan di relay** `[KEPUTUSAN-229]`:
   - hanya mendengarkan di jaringan Docker internal yang diikuti Keycloak dan Alertmanager, tidak di antarmuka publik atau jaringan privat antar-host;
   - satu penerima per pesan, daftar izin alamat `From`, batas ukuran pesan, batas laju;
   - tidak memakai `EndpointId` multi-region `[SRC-297]`;
   - tidak mencatat badan, subjek, atau alamat; log berisi hasil, ukuran, dan `MessageId` SES (ADR-005, ADR-010).
5. **Pengaturan SES** `[KEPUTUSAN-229]`:
   - pelacakan klik dan buka dimatikan lewat `ConfigurationOverrides.Tracking` dan configuration set, supaya tautan token tidak dialihkan lewat domain pelacakan `[SRC-297]` `[SRC-294]`;
   - configuration set email Keycloak mewajibkan TLS ke server penerima, bukan TLS oportunistik bawaan `[SRC-296]` (menunggu keputusan user, lihat bagian akhir);
   - kebijakan IAM akun email: hanya `ses:SendEmail`/`ses:SendRawEmail` pada ARN identitas di `ap-southeast-3`, dengan kondisi `ses:FromAddress`, `aws:SourceIp` (IP keluar VPS), dan penolakan `ses:MultiRegionEndpointId` `[SRC-298]`;
   - kunci akses AWS disimpan di OpenBao KV dan hanya diberikan ke proses relay dan modul notifikasi `[KEPUTUSAN-79]`.
6. **Konfigurasi Keycloak** `[KEPUTUSAN-231]`:
   - setiap realm mengarah ke relay dengan `starttls` aktif; sertifikat relay diterbitkan CA internal yang dipercaya lewat `truststore-paths`, dengan `tls-hostname-verifier` tetap `DEFAULT` `[SRC-301]`;
   - opsi `debug` SMTP dilarang dan diperiksa di berkas realm (`mail.debug` `[SRC-301]`);
   - AUTH SMTP dipakai bila password realm dapat disimpan tanpa menulis rahasia ke berkas realm keycloak-config-cli (diverifikasi F1-T36). Bila tidak, relay tanpa AUTH dan bergantung pada isolasi jaringan, TLS, dan pembatasan butir 4.
7. **Email aplikasi** (notifikasi ADR-016, tautan konfirmasi reset MFA `[KEPUTUSAN-204]`) tetap dikirim langsung oleh modul notifikasi lewat adaptor SES yang sama, tidak lewat relay.

## Alternatif yang ditolak

- **Provider `EmailSenderProvider` kustom (Java) yang memanggil SES langsung**: SPI internal yang dapat berubah tanpa pemberitahuan `[SRC-299]` `[SRC-300]`; menambah Java/Maven ke stack (mengubah ADR-024) dan menduplikasi adaptor SES dalam dua bahasa.
- **Provider Keycloak kustom → endpoint internal backend → outbox notifikasi**: masalah SPI dan Java yang sama, ditambah token aksi tersimpan di tabel outbox dan ikut backup sampai baris dibersihkan.
- **Relay SMTP → outbox aplikasi dengan coba ulang asinkron**: memberi coba ulang, tetapi menyimpan token aksi di database. Keycloak sudah melaporkan kegagalan secara sinkron (spike E1), sehingga pemanggil dapat mencoba ulang tanpa menyimpan pesan.
- **Penyedia SMTP lain di Indonesia**: menambah penyedia dan biaya, atau mengganti pilihan user T5-Q4. Kandidat dan pernyataan lokasi datanya `[TIDAK DIKETAHUI]`. Dapat dipilih user bila ADR ini ditolak.
- **Endpoint SMTP SES di region lain**: data diproses di luar Indonesia `[SRC-294]`, melanggar T4-Q1.
- **MTA sendiri yang mengirim langsung ke server penerima**: reputasi IP, PTR, dan kebijakan port 25 keluar di Biznet Gio `[TIDAK DIKETAHUI]`; beban operasi dan risiko keterkiriman tinggi untuk tim kecil.
- **Keycloak tidak mengirim email; backend membuat token sendiri**: membangun ulang alur autentikasi yang ADR-011 serahkan ke Keycloak (butir 4–5), termasuk "lupa password" di halaman login.

## Konsekuensi

- Positif: pilihan SES Jakarta lewat API `[USER] T5-Q4` tetap, tanpa penyedia dan bahasa baru. Keycloak memakai konfigurasi SMTP bawaan yang didukung, sehingga upgrade Keycloak tidak bergantung pada SPI internal. Token aksi tidak pernah ditulis ke disk oleh sistem Ruang Sehat. Alertmanager mendapat jalur email yang sama.
- Negatif:
  - ada satu proses baru dan pengurai SMTP buatan sendiri yang termasuk modul T1. Wajib test negatif: `From` di luar daftar izin, lebih dari satu penerima, pesan melebihi batas, koneksi tanpa STARTTLS, SES 429 dan 400;
  - tidak ada coba ulang otomatis untuk email Keycloak. Kegagalan SES muncul sebagai galat ke pengguna atau backend;
  - kuota SES bawaan (200 email/24 jam, 1 email/detik `[SRC-294]`) harus dinaikkan sebelum staging dipakai pengguna nyata dan sebelum uji beban; bila tidak, pengiriman sinkron langsung gagal saat beban tinggi.
- Yang harus dijaga:
  - relay tidak boleh terjangkau dari luar jaringan Docker internal;
  - log relay dan Keycloak tidak memuat badan email (ADR-005, ADR-010);
  - daftar komponen di `docs/04-arsitektur.md` dan `docs/05-stack.md` diperbarui setelah ADR ini diterima;
  - ukuran F1-T36 (4 jam-task) dinilai ulang; estimasi spike ±12–13 jam-task (spike §5).
- Lokasi data: komitmen AWS yang ditemukan bersifat umum, yaitu AWS tidak memindahkan konten ke luar region pilihan "except as necessary to provide the services you initiated" `[SRC-295]`. Pernyataan khusus SES tidak ditemukan `[SRC-296]`. Setiap email pasti diserahkan ke server penerima yang lokasinya `[TIDAK DIKETAHUI]`. Hal ini berlaku untuk semua opsi dan untuk email notifikasi aplikasi `[KEPUTUSAN-230]`.

## Butir yang menunggu keputusan user

1. Menerima ADR ini (relay internal ke SES API), atau memilih alternatif yang mengubah ADR-024 (provider Keycloak kustom) atau menambah penyedia (SMTP di Indonesia).
2. Menilai, bersama tim legal, apakah komitmen umum AWS dan penyerahan email ke server penerima memenuhi ketentuan data di Indonesia (T4-Q1, CR-007) `[KEPUTUSAN-230]`.
3. TLS wajib ke server penerima untuk email Keycloak (usulan) atau TLS oportunistik `[KEPUTUSAN-229]`.
