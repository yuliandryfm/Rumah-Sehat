# ADR-026: Email Keycloak lewat relay SMTP internal ke Amazon SES API Jakarta

Status: Diterima (2026-11-19, `[USER]` lewat Telegram: "1Y 2A 3W"; review keamanan LULUS DENGAN CATATAN putaran 2 `docs/review/F1-T02-security.md`)
Tanggal: 2026-11-19

Rincian temuan, eksperimen, dan perbandingan opsi: `docs/spike/F1-T02-email-keycloak.md`.

## Konteks

- Keycloak mengirim email verifikasi, atur ulang password, dan undangan lewat pengaturan server SMTP per realm `[SRC-264]`. Pengirim bawaan mengirim secara sinkron di dalam request, tanpa antrean `[SRC-301]`. Pada eksperimen E1 (spike §2.3), kegagalan SMTP membuat Admin REST mengembalikan HTTP 500 tanpa coba ulang. Alur "lupa password" di halaman login berbeda: kegagalan hanya dicatat sebagai event `EMAIL_SEND_FAILED` dan log server, lalu pengguna tetap melihat pesan sukses `[SRC-364]` (spike §2.6).
- Badan email Keycloak memuat token aksi dalam bentuk tautan `login-actions/action-token?key=...` (spike §2.3, E1).
- User memilih Amazon SES region Jakarta lewat API, dengan akun AWS khusus email `[USER] T5-Q4`. Endpoint SMTP SES tidak tersedia di Jakarta; endpoint API tersedia `[SRC-84]` `[SRC-294]`. API v2 `SendEmail` menerima pesan MIME mentah (Raw) `[SRC-297]`.
- Semua penyedia memproses data di Indonesia `[USER] T4-Q1` (CR-007). Endpoint SMTP SES di region lain, misalnya Singapura `[SRC-294]`, tidak memenuhi ketentuan ini.
- ADR-016 butir 5 menetapkan email lewat penyedia email atau relay SMTP yang memproses dan menyimpan data di Indonesia `[DOK: adr/ADR-016-konsultasi-realtime-dan-notifikasi.md]`. ADR-024 menetapkan stack Rust, Next.js, dan Flutter `[DOK: adr/ADR-024-stack-rust-nextjs-flutter.md]`.
- Keycloak menyediakan SPI `emailSender` untuk pengirim kustom, tetapi SPI itu internal: dapat berubah tanpa pemberitahuan dan memicu peringatan saat start `[SRC-299]` `[SRC-300]`.
- Alertmanager juga mengirim email lewat SMTP `[SRC-302]`, ke email tim teknis lewat SES `[KEPUTUSAN-80]`.
- Keycloak berjalan satu node di app-1 dan app-2, Alertmanager di host ops `[DOK: adr/ADR-025-hosting-vps-openbao.md]`. Firewall production default deny dengan daftar izin per host dan port `[KEPUTUSAN-144]`.
- Jalur email ini menyentuh autentikasi (modul T1, `CLAUDE.md`) dan merupakan pola baru, sehingga diusulkan sebagai ADR `[KEPUTUSAN-183]`.

## Keputusan

Keycloak (semua realm) dan Alertmanager mengirim email lewat SMTP ke **relay internal buatan sendiri yang dipasang per host**. Relay meneruskan pesan MIME mentah ke Amazon SES API v2 di `ap-southeast-3`, secara sinkron dan tanpa menyimpan pesan `[KEPUTUSAN-226]` `[KEPUTUSAN-232]`.

1. **Bentuk dan topologi.** Proses `email-relay` di workspace Rust, sebagai subcommand `apps/rs-backend` mengikuti pola `audit-relay`; letak final ditetapkan di F1-T36. Adaptor SES v2 berada di pustaka bersama dan dipakai juga oleh modul notifikasi (ADR-016 butir 5) `[KEPUTUSAN-228]`. Satu instans relay **per host** pengirim SMTP: app-1, app-2 (Keycloak), ops (Alertmanager), dan stg-app `[KEPUTUSAN-232]`:
   - relay berada di jaringan Docker khusus yang hanya berisi relay dan Keycloak (di ops: relay dan Alertmanager); kontainer `api`/`worker` tidak ikut jaringan itu;
   - relay tidak pernah dicapai lewat jaringan privat antar-host, sehingga tidak ada port baru di daftar izin firewall `[KEPUTUSAN-144]`;
   - alasan memilih per host dibanding satu relay lintas host: token aksi tidak melintasi jaringan antar-host, dan AUTH SMTP tidak menjadi syarat (butir 6). Kunci AWS memang berada di lebih banyak host, tetapi di app-1/app-2 kunci itu sudah dipegang modul notifikasi (butir 7), sehingga tambahannya hanya ops dan stg-app (mitigasi di butir 5).
2. **Protokol.** Relay mengimplementasikan subset SMTP yang cukup untuk Keycloak (EHLO, MAIL FROM, RCPT TO, DATA, QUIT; spike E1), ditambah STARTTLS. Pustaka TLS dan penandatangan SigV4 dipilih di F1-T36 dengan cek registry dan dicatat di `docs/sources.md`. Relay pihak ketiga dan crate server SMTP yang diperiksa tidak dipakai karena tidak aktif dirilis `[SRC-303]` `[KEPUTUSAN-228]`.
3. **Sinkron, tanpa penyimpanan.** Relay membalas sukses hanya setelah SES mengembalikan HTTP 200 dan `MessageId` `[SRC-297]`. Galat SES 429 dan 5xx menjadi balasan SMTP sementara (4xx); galat 400 menjadi balasan permanen (5xx). Pesan hanya ada di memori. Batas waktu panggilan SES lebih pendek dari batas tulis Keycloak 10000 ms `[SRC-301]`. Akibat bagi pengguna berbeda per jalur `[KEPUTUSAN-227]`:
   - jalur Admin REST (pendaftaran, kirim ulang verifikasi, undangan lewat backend): kegagalan sampai ke backend sebagai HTTP 500; backend menampilkan galat dan pengguna dapat meminta kirim ulang (US-101 AC2);
   - alur "lupa password" di halaman login: kegagalan tidak terlihat oleh pengguna maupun backend karena Keycloak tetap menampilkan pesan sukses `[SRC-364]`. Kegagalan dideteksi lewat butir 8;
   - required action verifikasi email yang dipicu saat login: perilakunya `[TIDAK DIKETAHUI]`, diuji di F1-T36.
4. **Pembatasan di relay** `[KEPUTUSAN-229]`:
   - hanya mendengarkan di jaringan Docker khusus di host yang sama (butir 1), tidak di antarmuka publik atau jaringan privat antar-host;
   - satu penerima per pesan, daftar izin alamat `From`, batas ukuran pesan dan panjang baris, pemisah CRLF ketat;
   - batas laju global dan per penerima (di memori, kunci di-hash). Saat batas per penerima tercapai, relay membalas 250 dan membuang pesan (dihitung di metrik), supaya tidak muncul sinyal berbeda bagi pemanggil;
   - permintaan SES disusun relay: `Destination.ToAddresses` dari RCPT, `FromEmailAddress` dari daftar izin, dan `ConfigurationSetName` selalu diisi eksplisit. Pesan ditolak bila header `From` tidak cocok dengan amplop atau daftar izin, bila To/Cc/Bcc memuat alamat selain penerima amplop, atau bila ada header `X-SES-*`, karena SES membaca header khusus dari isi pesan Raw `[SRC-365]`;
   - tidak memakai `EndpointId` multi-region `[SRC-297]`;
   - teks balasan SMTP tetap per kelas galat ditambah ID korelasi acak, tanpa menggemakan header, badan, atau alamat, karena penyebab exception masuk log server Keycloak `[SRC-301]` `[SRC-364]`;
   - tidak mencatat badan, subjek, atau alamat; log berisi hasil, ukuran, dan `MessageId` SES (ADR-005, ADR-010).
5. **Pengaturan SES** `[KEPUTUSAN-229]`:
   - pelacakan klik dan buka dimatikan lewat `ConfigurationOverrides.Tracking` dan configuration set, supaya tautan token tidak dialihkan lewat domain pelacakan `[SRC-297]` `[SRC-294]`;
   - configuration set email Keycloak mewajibkan TLS ke server penerima, bukan TLS oportunistik bawaan `[SRC-296]` (menunggu keputusan user, lihat bagian akhir);
   - kebijakan IAM akun email: hanya `ses:SendEmail`/`ses:SendRawEmail` pada ARN identitas di `ap-southeast-3`, dengan kondisi `ses:FromAddress`, `aws:SourceIp` (IP keluar VPS), dan penolakan `ses:MultiRegionEndpointId` `[SRC-298]`;
   - kebijakan IAM: `aws:SourceIp` memuat IP keluar semua host relay dan notifikasi (app-1, app-2, ops, stg-app); IP keluar yang sebenarnya, termasuk dampak Magic Transit, `[TIDAK DIKETAHUI]` dan diverifikasi di F1-T36;
   - kunci akses AWS disimpan di OpenBao KV dan hanya diberikan ke proses relay dan modul notifikasi, dengan policy per host yang hanya membaca kunci SES `[KEPUTUSAN-79]`. Kunci IAM berumur panjang, sehingga prosedur rotasi dan pencabutan ditulis sebelum go-live. Kemungkinan kredensial AWS dinamis dari OpenBao `[TIDAK DIKETAHUI]`, diperiksa di F1-T36.
6. **Konfigurasi Keycloak** `[KEPUTUSAN-231]`:
   - setiap realm mengarah ke relay dengan `starttls` aktif; sertifikat relay diterbitkan CA internal yang dipercaya lewat `truststore-paths`, dengan `tls-hostname-verifier` tetap `DEFAULT` `[SRC-301]`;
   - opsi `debug` SMTP dilarang dan diperiksa di berkas realm (`mail.debug` `[SRC-301]`);
   - AUTH SMTP dipakai bila password realm dapat disimpan tanpa menulis rahasia ke berkas realm keycloak-config-cli (diverifikasi F1-T36). Bila tidak, relay tanpa AUTH dan bergantung pada jaringan Docker khusus per host (butir 1), TLS, dan pembatasan butir 4. Bila kelak relay harus dicapai lintas host, AUTH menjadi wajib bersama daftar izin firewall `[KEPUTUSAN-144]`.
7. **Email aplikasi** (notifikasi ADR-016, tautan konfirmasi reset MFA `[KEPUTUSAN-204]`) tetap dikirim langsung oleh modul notifikasi lewat adaptor SES yang sama, tidak lewat relay. Perlindungan token di ADR ini tidak otomatis berlaku untuk email aplikasi bertoken. Untuk rancangan US-207 AC5 (Fase 2) diusulkan: outbox hanya menyimpan referensi dan token dibangkitkan saat pengiriman (database hanya menyimpan hash), serta configuration set wajib TLS (bila dipilih user) juga untuk email bertoken aplikasi.
8. **Deteksi kegagalan** `[KEPUTUSAN-227]`: peringatan Alertmanager atas laju galat relay/SES, atas event Keycloak `EMAIL_SEND_FAILED` yang sudah diambil worker ke audit dan security log (ADR-011 butir 10), dan atas pemakaian kuota SES. Endpoint "lupa password" dan kirim ulang verifikasi diberi batas laju di Caddy/WAF (ADR-019 butir 3). Alamat di daftar supresi SES `[SRC-296]` masuk prosedur penghapusan hak subjek data (EP-16).

## Alternatif yang ditolak

- **Provider `EmailSenderProvider` kustom (Java) yang memanggil SES langsung**: SPI internal yang dapat berubah tanpa pemberitahuan `[SRC-299]` `[SRC-300]`; menambah Java/Maven ke stack (mengubah ADR-024) dan menduplikasi adaptor SES dalam dua bahasa.
- **Provider Keycloak kustom → endpoint internal backend → outbox notifikasi**: masalah SPI dan Java yang sama, ditambah token aksi tersimpan di tabel outbox dan ikut backup sampai baris dibersihkan.
- **Relay SMTP → outbox aplikasi dengan coba ulang asinkron**: memberi coba ulang, tetapi menyimpan token aksi di database. Pada jalur Admin REST, Keycloak sudah melaporkan kegagalan secara sinkron (spike E1), sehingga backend dapat mencoba ulang tanpa menyimpan pesan. Pada alur "lupa password", kegagalan tidak terlihat `[SRC-364]`; coba ulang outbox akan menutupinya, tetapi dengan harga token tersimpan. Penyusun memilih deteksi (butir 8) dan kirim ulang oleh pengguna.
- **Penyedia SMTP lain di Indonesia**: menambah penyedia dan biaya, atau mengganti pilihan user T5-Q4. Kandidat dan pernyataan lokasi datanya `[TIDAK DIKETAHUI]`. Dapat dipilih user bila ADR ini ditolak.
- **Satu relay yang dicapai lintas host lewat jaringan privat**: token aksi melintasi jaringan antar-host, AUTH SMTP menjadi wajib (padahal penyimpanan password realm tanpa rahasia di berkas realm belum terverifikasi), dan perlu port baru di daftar izin firewall `[KEPUTUSAN-144]`.
- **Endpoint SMTP SES di region lain**: data diproses di luar Indonesia `[SRC-294]`, melanggar T4-Q1.
- **MTA sendiri yang mengirim langsung ke server penerima**: reputasi IP, PTR, dan kebijakan port 25 keluar di Biznet Gio `[TIDAK DIKETAHUI]`; beban operasi dan risiko keterkiriman tinggi untuk tim kecil.
- **Keycloak tidak mengirim email; backend membuat token sendiri**: membangun ulang alur autentikasi yang ADR-011 serahkan ke Keycloak (butir 4–5), termasuk "lupa password" di halaman login.

## Konsekuensi

- Positif: pilihan SES Jakarta lewat API `[USER] T5-Q4` tetap, tanpa penyedia dan bahasa baru. Keycloak memakai konfigurasi SMTP bawaan yang didukung, sehingga upgrade Keycloak tidak bergantung pada SPI internal. Token aksi **Keycloak** tidak pernah ditulis ke disk oleh sistem Ruang Sehat (email aplikasi bertoken diatur terpisah, butir 7). Alertmanager mendapat jalur email yang sama.
- Negatif:
  - ada satu jenis proses baru (empat instans, satu per host) dan pengurai SMTP buatan sendiri yang termasuk modul T1. Wajib test negatif: `From` di luar daftar izin, lebih dari satu penerima di amplop atau header, header `X-SES-*`, pesan melebihi batas, baris tanpa CRLF yang benar, koneksi tanpa STARTTLS, SES 429 dan 400, teks balasan tidak memuat alamat;
  - tidak ada coba ulang otomatis untuk email Keycloak. Pada jalur Admin REST kegagalan muncul sebagai galat ke backend; pada alur "lupa password" kegagalan tidak terlihat oleh pengguna dan hanya terdeteksi lewat peringatan butir 8 `[SRC-364]`;
  - kunci AWS berada di empat host (butir 1, butir 5);
  - kuota SES bawaan (200 email/24 jam, 1 email/detik `[SRC-294]`) harus dinaikkan sebelum staging dipakai pengguna nyata dan sebelum uji beban; bila tidak, pengiriman sinkron langsung gagal saat beban tinggi.
- Yang harus dijaga:
  - relay tidak boleh terjangkau dari luar jaringan Docker khusus di host-nya; test deploy memeriksa bahwa port relay tidak terbuka di antarmuka host;
  - log relay dan Keycloak tidak memuat badan email (ADR-005, ADR-010);
  - daftar komponen di `docs/04-arsitektur.md` dan `docs/05-stack.md` diperbarui setelah ADR ini diterima;
  - ukuran F1-T36 (4 jam-task) dinilai ulang; estimasi spike ±12–13 jam-task (spike §5).
- Lokasi data: komitmen AWS yang ditemukan bersifat umum, yaitu AWS tidak memindahkan konten ke luar region pilihan "except as necessary to provide the services you initiated" `[SRC-295]`. Pernyataan khusus SES tidak ditemukan `[SRC-296]`. Setiap email pasti diserahkan ke server penerima yang lokasinya `[TIDAK DIKETAHUI]`. Hal ini berlaku untuk semua opsi dan untuk email notifikasi aplikasi `[KEPUTUSAN-230]`.

## Butir yang menunggu keputusan user

1. Menerima ADR ini (relay internal ke SES API), atau memilih alternatif yang mengubah ADR-024 (provider Keycloak kustom) atau menambah penyedia (SMTP di Indonesia).
2. Menilai, bersama tim legal, apakah komitmen umum AWS dan penyerahan email ke server penerima memenuhi ketentuan data di Indonesia (T4-Q1, CR-007) `[KEPUTUSAN-230]`.
3. TLS wajib ke server penerima untuk email Keycloak (usulan) atau TLS oportunistik `[KEPUTUSAN-229]`. Catatan: kegagalan TLS terjadi setelah SES menerima pesan ("It's possible for Amazon SES to accept a message without sending it" `[SRC-366]`), sehingga relay, Keycloak, dan pengguna tetap melihat sukses. Mendeteksinya memerlukan event publishing SES, yang menyimpan alamat penerima di layanan AWS lain dan menambah pertanyaan lokasi data butir 2.

## Keputusan user (2026-11-19)

| Butir | Keputusan `[USER]` |
| --- | --- |
| 1 | ADR ini diterima: relay SMTP internal per host ke SES API v2 Jakarta |
| 2 | Lanjut; kepatuhan lokasi data jalur email dinilai tim legal sebagai L-16 (`docs/02-brd.md`), sebelum go-live |
| 3 | TLS wajib ke server penerima untuk email Keycloak (KEPUTUSAN-229) |
