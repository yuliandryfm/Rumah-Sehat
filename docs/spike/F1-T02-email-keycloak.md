# Spike F1-T02: Jalur email Keycloak

Tanggal: 2026-11-19
Task: F1-T02 (EN-13), tingkat ketelitian **T1** (autentikasi; `aturan-kerja.md` §1) `[DOK: 07-phase-plan.md]` §7.3
Status: selesai sebagai usulan; revisi setelah review keamanan putaran 1 (`docs/review/F1-T02-security.md`, B-01, B-02, C-01–C-05); menunggu cek ulang review dan keputusan user atas `docs/adr/ADR-026-jalur-email-keycloak.md` (Diusulkan)
Dasar: RT-32 `[DOK: 04-risk-register.md]`, `[KEPUTUSAN-183]`, ADR-011 butir 4–6, ADR-016 butir 5, `docs/05-stack.md` §13.2 butir 18, jawaban user T5-Q4 (SES Jakarta lewat API, akun AWS khusus email) `[USER] T5-Q4`, CR-007 dan T4-Q1 (semua penyedia memproses data di Indonesia) `[USER] T4-Q1`.

## 1. Pertanyaan spike

1. Bagaimana Keycloak mengirim email verifikasi, atur ulang password, dan undangan, bila Amazon SES di Jakarta hanya menyediakan API?
2. Opsi apa yang tersedia, apa kemampuan teknisnya, apa dampak keamanannya (token di email, log, rahasia), dan berapa effort-nya?
3. Apa pernyataan resmi AWS tentang lokasi pemrosesan dan penyimpanan data SES (stack §13.2 butir 18)?

## 2. Temuan

### 2.1 Amazon SES di Jakarta

| Temuan | Sumber |
| --- | --- |
| Endpoint API SES Jakarta: `email.ap-southeast-3.amazonaws.com` dan `email.ap-southeast-3.api.aws` (HTTPS) | `[SRC-294]` |
| Endpoint SMTP tidak tersedia di Jakarta: "SMTP endpoints are not currently available in ... Asia Pacific (Jakarta)". Singapura (`ap-southeast-1`) punya endpoint SMTP | `[SRC-294]` (menegaskan `[SRC-84]`) |
| Kuota bawaan per region: kirim 200 email per 24 jam dan 1 email per detik; keduanya dapat dinaikkan | `[SRC-294]` |
| SES punya domain pelacakan tautan open/click per region, untuk Jakarta `r.ap-southeast-3.awstrack.me` | `[SRC-294]` |
| API v2 `SendEmail` (`POST /v2/email/outbound-emails`) menerima pesan **Raw** ("A raw, MIME-formatted email message"), Simple, atau Templated. Pelacakan klik dan buka dapat dimatikan per pesan lewat `ConfigurationOverrides.Tracking`. Parameter `EndpointId` adalah "multi-region endpoint (global-endpoint)" | `[SRC-297]` |
| Bawaan SES memakai TLS oportunistik ke server penerima: bila koneksi aman gagal, pesan dikirim tanpa enkripsi. Perilaku ini dapat diubah supaya hanya dikirim bila TLS berhasil | `[SRC-296]` |
| IAM dapat membatasi aksi `ses:SendEmail`/`ses:SendRawEmail`, identitas pengirim lewat ARN identitas, serta kunci kondisi `ses:FromAddress`, `ses:Recipients`, `ses:MultiRegionEndpointId`, `ses:ApiVersion`, dan `aws:SourceIp` | `[SRC-298]` |

### 2.2 Cara Keycloak 26.8.2 mengirim email

| Temuan | Sumber |
| --- | --- |
| Keycloak mengirim email ke pengguna lewat pengaturan server SMTP per realm | `[SRC-264]` |
| Pengirim bawaan (`DefaultEmailSenderProvider`) membuka koneksi SMTP dan memanggil `Transport.sendMessage` di dalam request, tanpa antrean. Opsi yang dibaca: `host`, `port`, `from`, `ssl`, `starttls` (memaksa `mail.smtp.starttls.required`), `auth` dengan `authType` basic atau token (XOAUTH2), `debug` (menyalakan `mail.debug`), serta batas waktu `timeout`/`connectionTimeout`/`writeTimeout` bawaan 10000 ms. TLS keluar memakai truststore Keycloak | `[SRC-301]` |
| Truststore Keycloak diatur dengan `truststore-paths`; `tls-hostname-verifier` (bawaan `DEFAULT`) berlaku untuk "out-going HTTPS and SMTP requests" | `[SRC-301]` |
| Ada SPI `emailSender` (`EmailSenderProvider`) dengan metode `send(config, address, subject, textBody, htmlBody)` dan `validate(config)`, tetapi `EmailSenderSpi.isInternal()` bernilai `true`. Keycloak mencatat peringatan "is implementing the internal SPI ... This SPI is internal and may change without notice" untuk provider SPI internal | `[SRC-299]` |
| Dokumentasi Keycloak untuk SPI internal lain: "This is an internal SPI. ... any new release of Keycloak can change it without prior notice." | `[SRC-300]` |

### 2.3 Eksperimen E1 (2026-11-19, scratchpad, tidak masuk repo)

Tujuan: membuktikan perilaku Keycloak 26.8.2 terhadap server SMTP internal sederhana, isi email, dan perilaku saat SMTP gagal.

Langkah:
1. Image `quay.io/keycloak/keycloak:26.8.2` dijalankan `start-dev` (H2, port 18080). Penangkap SMTP buatan sendiri (Python stdlib, `smtp_sink.py`, hanya mendukung EHLO/HELO, MAIL FROM, RCPT TO, DATA, QUIT, tanpa TLS dan tanpa AUTH) mendengarkan di `127.0.0.1:2525`.
2. Realm `spike` dibuat lewat Admin REST dengan `smtpServer` mengarah ke penangkap. User dibuat, lalu dipanggil `PUT .../users/{id}/send-verify-email` dan `PUT .../users/{id}/execute-actions-email?lifespan=3600` dengan `["UPDATE_PASSWORD","CONFIGURE_TOTP"]` (bentuk undangan).
3. Penangkap dimatikan, lalu `send-verify-email` dipanggil lagi.

Hasil (output dipangkas):

```
GET /admin/serverinfo  -> emailSender ['default'] internal= True ; emailTemplate ['freemarker'] internal= True
send-verify-email 204 ; execute-actions-email 204
=== MAIL FROM:<no-reply@example.test> | RCPT TO:<pasien1@example.test>   (2 pesan, masing-masing 1 penerima)
Subject: Verify email ; Subject: Update Your Account
Content-Type: multipart/alternative (text/plain + text/html, UTF-8)
http://127.0.0.1:18080/realms/spike/login-actions/action-token?key=eyJhbGciOiJIUzUxMiIs...   (token aksi JWT di badan email)
penangkap mati -> {"errorMessage":"Failed to send verify email"} HTTP 500
log Keycloak: KC-SERVICES0029: Failed to send email ... ; jumlah kemunculan "action-token"/"key=eyJ" di log Keycloak: 0
```

Kesimpulan E1:
- Keycloak bekerja dengan server SMTP minimal (subset perintah di atas) dan mengirim satu penerima per pesan dalam bentuk MIME standar. Relay tidak perlu mengurai MIME; cukup meneruskan data mentah.
- Badan email memuat **token aksi** dalam bentuk tautan. Siapa pun yang melihat isi email sebelum token kedaluwarsa dapat memakainya. Untuk atur ulang password, MFA tetap diminta sesudah tautan dibuka (ADR-011 butir 5, `[KEPUTUSAN-199]`), tetapi verifikasi email dan undangan tetap bergantung pada kerahasiaan tautan.
- Pengiriman bersifat sinkron. **Pada jalur Admin REST** (yang diuji E1: pendaftaran, kirim ulang verifikasi, undangan lewat backend F1-T50/F1-T51), bila SMTP gagal Admin REST mengembalikan 500 dan Keycloak tidak mencoba ulang; backend yang menangani kegagalan dan kirim ulang. Jalur lain berperilaku berbeda (§2.6).
- Log Keycloak saat gagal tidak memuat token pada percobaan ini. Opsi `debug` tidak diuji; isinya `[TIDAK DIKETAHUI]`.

### 2.4 Lokasi data SES (stack §13.2 butir 18)

| Temuan | Sumber |
| --- | --- |
| Komitmen umum AWS: "You choose the AWS Region(s) in which your content is stored." dan "We will not move or replicate your content outside of your chosen AWS Region(s) without your agreement", dengan pengecualian "except as necessary to provide the services you initiated" atau "to comply with the law or a binding order of a governmental body" | `[SRC-295]` |
| Halaman perlindungan data SES merujuk ke Data Privacy FAQ dan tidak memuat pernyataan lokasi khusus SES. SES mengenkripsi data saat disimpan secara bawaan | `[SRC-296]` |
| Data pribadi yang dapat tersimpan di SES: identitas pengirim terverifikasi, alamat di daftar supresi tingkat akun (tetap ada sampai dihapus), data event email bila dialirkan ke layanan lain, dan templat | `[SRC-296]` |

Penilaian:
- Pernyataan **khusus SES** tentang lokasi pemrosesan tidak ditemukan. Yang ada adalah komitmen umum AWS di atas, termasuk pengecualian "necessary to provide the services you initiated" `[SRC-295]`. Status butir 18: **sebagian terverifikasi**.
- Pengiriman email ke kotak surat penerima mengharuskan pesan diserahkan ke server email penerima, misalnya penyedia email publik yang servernya bisa berada di luar Indonesia. Ini sifat email itu sendiri dan berlaku untuk **semua opsi** di dokumen ini, termasuk penyedia SMTP di Indonesia. Lokasi server penyedia email penerima `[TIDAK DIKETAHUI]`.
- Apakah komitmen umum AWS dan sifat penyerahan email ke server penerima memenuhi ketentuan "data wajib disimpan di Indonesia" dan T4-Q1 adalah **penilaian bisnis/legal**, bukan keputusan teknis. Penilaian itu dibawa ke user dan tim legal (§8) `[KEPUTUSAN-230]`.

### 2.5 Pengirim SMTP lain di stack

Alertmanager mengirim peringatan ke email tim teknis lewat SES `[DOK: 05-stack.md]` §4 `[KEPUTUSAN-80]`. Penerima `email_configs` Alertmanager memakai SMTP ("The SMTP host through which emails are sent"), dengan `smtp_require_tls` bawaan `true` `[SRC-302]`. Jadi Alertmanager menghadapi masalah yang sama dengan Keycloak. Karena Keycloak berjalan di app-1/app-2 dan Alertmanager di host ops `[DOK: adr/ADR-025-hosting-vps-openbao.md]`, relay opsi 1 dipasang per host, termasuk satu di ops untuk Alertmanager `[KEPUTUSAN-231]` `[KEPUTUSAN-232]`.

### 2.6 Kegagalan email pada alur "lupa password" (temuan review B-01)

Alur "lupa password" di halaman login Keycloak 26.8.2 menangkap `EmailException`, mencatat event `SEND_RESET_PASSWORD` dengan galat `EMAIL_SEND_FAILED` (detail `REASON` berisi pesan exception) dan log server, lalu **tetap menampilkan pesan sukses yang sama** dengan pengiriman berhasil. Pesan sukses yang sama juga dipakai bila user tidak ditemukan `[SRC-364]`. Pesan exception dari pengirim bawaan bernilai tetap ("Error when attempting to send the email to the server..."), sedangkan penyebabnya dicatat ke log server `[SRC-301]`.

Akibatnya:
- Kegagalan relay atau SES (kuota habis, 429) pada alur ini **tidak terlihat** oleh pengguna maupun backend. Perilaku ini tidak membuka oracle enumerasi akun, sehingga yang diubah adalah klaim dan kontrol deteksi, bukan rancangan.
- Kontrol deteksi diperlukan: peringatan Alertmanager atas laju galat relay/SES dan atas event Keycloak `EMAIL_SEND_FAILED` yang sudah diambil worker ke audit dan security log (ADR-011 butir 10) `[KEPUTUSAN-227]`.
- Perilaku required action verifikasi email yang dipicu saat login (bukan lewat Admin REST) belum diuji `[TIDAK DIKETAHUI]` (§9).

## 3. Opsi

| # | Opsi | Inti teknis | Data di Indonesia | Mengubah ADR diterima / menambah penyedia |
| --- | --- | --- | --- | --- |
| 1 | **Relay SMTP internal → SES API Jakarta** | Keycloak memakai SMTP bawaan ke relay buatan sendiri di jaringan internal. Relay meneruskan MIME mentah ke `SendEmail` Raw di `ap-southeast-3` `[SRC-297]` | Relay di VPS Indonesia; SES Jakarta (§2.4) | Tidak. ADR-016 butir 5 sudah menyebut "relay SMTP"; T5-Q4 (SES lewat API) tetap. Menambah satu proses (perlu ADR-026) |
| 2 | Provider `EmailSenderProvider` kustom (Java) → SES API langsung | Mengganti pengirim bawaan Keycloak dengan provider Java yang memanggil SES | Sama dengan opsi 1 | Ya: menambah Java/Maven ke stack (ADR-024). SPI internal `[SRC-299]` |
| 3 | Provider `EmailSenderProvider` kustom → endpoint internal backend → outbox notifikasi → adaptor SES | Keycloak menyerahkan email ke aplikasi; aplikasi mengirim lewat adaptor SES yang sama dengan notifikasi | Sama dengan opsi 1 | Ya: Java/Maven (ADR-024); token aksi tersimpan di database (ADR-005) |
| 4 | Relay SMTP → outbox aplikasi (asinkron dengan coba ulang) | Seperti opsi 1, tetapi relay menyimpan pesan ke outbox lalu worker mengirim | Sama dengan opsi 1 | Tidak mengubah stack; token aksi tersimpan di database |
| 5 | Penyedia SMTP lain di Indonesia | Keycloak (dan Alertmanager) memakai SMTP penyedia lain | Bergantung penyedia; kandidat `[TIDAK DIKETAHUI]` | Ya: menambah penyedia dan biaya, atau mengganti T5-Q4 |
| 6 | Endpoint SMTP SES region lain (misalnya Singapura) | Konfigurasi SMTP langsung `[SRC-294]` | **Tidak**: data diproses di luar Indonesia | Melanggar T4-Q1/CR-007. Ditolak |
| 7 | MTA sendiri langsung ke server penerima | Postfix atau sejenisnya di VPS mengirim langsung | Ya, sampai server penerima | Tanpa penyedia baru; reputasi IP, PTR, dan kebijakan port 25 keluar di Biznet Gio `[TIDAK DIKETAHUI]`. Ditolak (beban operasi dan risiko keterkiriman) |
| 8 | Keycloak tidak mengirim email; backend membuat token sendiri | Verifikasi email dan atur ulang password dibangun ulang di aplikasi | Sama dengan opsi 1 | Membangun ulang alur autentikasi yang ADR-011 serahkan ke Keycloak (butir 4–5), termasuk tautan "lupa password" di halaman login. Ditolak (risiko T1) |

### 3.1 Rincian opsi 1 (relay SMTP internal → SES API)

- **Bentuk**: proses `email-relay` di workspace Rust (subcommand `apps/rs-backend`, pola yang sama dengan `audit-relay`; letak final ditetapkan di F1-T36) `[KEPUTUSAN-228]`. Adaptor SES v2 dibuat sekali di pustaka bersama dan dipakai juga oleh modul notifikasi (ADR-016 butir 5).
- **Topologi (revisi B-02)**: satu relay **per host** yang menjalankan pengirim SMTP: app-1 dan app-2 (Keycloak, ADR-025 menempatkan satu node Keycloak per host), ops (Alertmanager), dan stg-app. Relay berada di jaringan Docker khusus yang hanya berisi relay dan Keycloak (di ops: relay dan Alertmanager); kontainer `api`/`worker` tidak ikut jaringan itu. Relay tidak pernah dicapai lewat jaringan privat antar-host, sehingga tidak ada port baru di daftar izin firewall `[KEPUTUSAN-144]` `[KEPUTUSAN-232]`.
  - Alasan memilih per host dibanding satu relay lintas host: token aksi tidak melintasi jaringan antar-host; AUTH SMTP tetap opsional sehingga mekanisme penyimpanan password realm (§9 butir 2) tidak menjadi syarat; kegagalan satu relay hanya memengaruhi host itu.
  - Biayanya: kunci AWS berada di lebih banyak host. Di app-1/app-2 kunci itu sudah dipegang modul notifikasi (butir 7 ADR-026), sehingga tambahan nyatanya hanya ops dan stg-app. Mitigasi: policy OpenBao per host yang hanya membaca kunci SES, `aws:SourceIp` memuat IP keluar semua host itu, serta prosedur rotasi dan pencabutan kunci (H-17). IP keluar host Biznet yang sebenarnya, termasuk dampak Magic Transit, `[TIDAK DIKETAHUI]` dan diverifikasi di F1-T36.
- **Protokol**: subset SMTP yang terbukti cukup untuk Keycloak di E1 (EHLO, MAIL FROM, RCPT TO, DATA, QUIT), ditambah STARTTLS. AUTH dipakai bila password SMTP di konfigurasi realm dapat disimpan tanpa menulis rahasia ke berkas realm (§9 butir 2) `[KEPUTUSAN-231]`.
- **Sinkron, tanpa penyimpanan**: relay membalas sukses ke Keycloak hanya setelah SES mengembalikan HTTP 200 dan `MessageId` `[SRC-297]`. Galat SES 429/5xx dipetakan ke balasan sementara (4xx SMTP), galat 400 ke balasan permanen (5xx SMTP). Pesan hanya ada di memori `[KEPUTUSAN-227]`. Batas waktu panggilan SES di relay lebih pendek dari batas tulis Keycloak 10000 ms `[SRC-301]`, supaya relay tidak membalas sukses setelah Keycloak menyerah. Teks balasan SMTP tetap per kelas galat ditambah ID korelasi acak, tanpa menggemakan header, badan, atau alamat, karena penyebab exception masuk log server Keycloak (§2.6) `[KEPUTUSAN-229]`.
- **Pembatasan**: hanya mendengarkan di jaringan Docker khusus per host; satu penerima per pesan (sesuai perilaku Keycloak di E1); daftar izin alamat `From`; batas ukuran pesan dan panjang baris, pemisah CRLF ketat; batas laju global dan per penerima (di memori, kunci di-hash; saat batas per penerima tercapai relay membalas 250 dan membuang pesan, dihitung di metrik, supaya tidak ada sinyal berbeda bagi pemanggil); `EndpointId` multi-region tidak dipakai; pelacakan klik dan buka dimatikan `[KEPUTUSAN-229]`.
- **Permintaan SES disusun relay, bukan dari header MIME (revisi C-01)**: SES membaca header khusus dari isi pesan Raw (`X-SES-SOURCE-ARN`, `X-SES-FROM-ARN`) dan `Destination` tidak wajib `[SRC-365]`. Relay selalu mengisi `Destination.ToAddresses` dari RCPT, `FromEmailAddress` dari daftar izin, dan `ConfigurationSetName` secara eksplisit, lalu menolak pesan bila header `From` tidak cocok dengan amplop/daftar izin, bila To/Cc/Bcc memuat alamat selain penerima amplop, atau bila ada header `X-SES-*` `[KEPUTUSAN-229]`.
- **Pustaka**: relay pihak ketiga dan crate server SMTP yang diperiksa tidak aktif dirilis (pembaruan terakhir 2022–2024) `[SRC-303]`, sehingga subset SMTP ditulis sendiri. Pustaka TLS dan penandatangan SigV4 dipilih di F1-T36 dengan cek registry dan dicatat di `docs/sources.md` `[KEPUTUSAN-228]`.

### 3.2 Rincian opsi 2 dan 3 (provider Keycloak kustom)

- Antarmuka `EmailSenderProvider` kecil (`send`, `validate`), sehingga provider-nya sederhana `[SRC-299]`.
- SPI ini internal: dapat berubah tanpa pemberitahuan di rilis baru, dan Keycloak mencatat peringatan saat start `[SRC-299]` `[SRC-300]`. Setiap upgrade Keycloak perlu memeriksa ulang provider.
- Membutuhkan toolchain Java/Maven, build JAR, dan image Keycloak kustom. Stack saat ini Rust, Next.js, dan Flutter (ADR-024), sehingga ADR-024 harus diubah dan pemindaian dependensi (osv-scanner, verify.sh) diperluas ke ekosistem Maven.
- Opsi 3 menambah kredensial Keycloak → backend (klien atau rahasia bersama), dan menyimpan badan email yang memuat token aksi di tabel outbox (§4).

### 3.3 Rincian opsi 5 (penyedia SMTP di Indonesia)

Penyedia SMTP yang memproses data di Indonesia tidak diteliti dalam spike ini. Biznet Gio tidak ditemukan menyediakan layanan email `[DOK: sources.md]` (Pencarian tanpa hasil, 2026-11-18). Kandidat, harga, dan pernyataan lokasi data `[TIDAK DIKETAHUI]`. Opsi ini menambah penyedia dan biaya, sehingga hanya bisa dipilih user (§8).

## 4. Dampak keamanan

| Aspek | Opsi 1 (relay sinkron) | Opsi 2 (provider → SES) | Opsi 3 / 4 (lewat outbox) | Opsi 5 (SMTP penyedia lain) |
| --- | --- | --- | --- | --- |
| Token aksi di email (E1) | Hanya di memori relay selama satu request | Hanya di memori Keycloak | **Tersimpan** di tabel outbox dan ikut PITR/backup sampai baris dibersihkan; perlu enkripsi kolom dan penghapusan segera setelah terkirim | Melewati infrastruktur penyedia lain |
| Log | Relay tidak mencatat badan, subjek, atau alamat; cukup hasil, ukuran, dan `MessageId` SES. Opsi `debug` SMTP Keycloak dilarang di berkas realm (`mail.debug` `[SRC-301]`; isinya `[TIDAK DIKETAHUI]`) | Provider tidak mencatat badan; `debug` tidak relevan | Seperti opsi 1, ditambah aturan untuk tabel outbox | `debug` dilarang |
| Rahasia | Kunci akses AWS (akun khusus email `[USER] T5-Q4`) hanya dipegang proses relay (per host: app-1, app-2, ops, stg-app) dan modul notifikasi, dari OpenBao KV dengan policy per host yang hanya membaca kunci SES `[KEPUTUSAN-79]` `[KEPUTUSAN-232]`. Kunci IAM jangka panjang: perlu prosedur rotasi dan pencabutan; kredensial AWS dinamis dari OpenBao `[TIDAK DIKETAHUI]`. Bila AUTH SMTP dipakai, password relay perlu disimpan di konfigurasi realm (§9 butir 2) | Kunci AWS dipegang proses Keycloak; memperluas permukaan rahasia Keycloak | Kredensial Keycloak → backend; kunci AWS tetap di backend | Kredensial SMTP penyedia di konfigurasi realm |
| Penyalahgunaan sebagai open relay | Dicegah: jaringan Docker khusus per host saja, permintaan SES disusun dari amplop dan header divalidasi, daftar izin `From`, satu penerima, batas ukuran dan laju. Lapis kedua IAM: `ses:FromAddress`, ARN identitas Jakarta, `aws:SourceIp`, `ses:MultiRegionEndpointId` `[SRC-298]` | Tidak ada port SMTP baru | Endpoint internal backend perlu autentikasi klien | Bergantung penyedia |
| Pelacakan tautan SES | Dimatikan per pesan (`ConfigurationOverrides.Tracking`) dan di configuration set, supaya tautan token tidak dialihkan lewat domain pelacakan `[SRC-297]` `[SRC-294]` | Sama | Sama | Bergantung penyedia |
| TLS ke server penerima | Configuration set untuk email Keycloak diusulkan **wajib TLS** (bukan oportunistik) karena memuat token `[SRC-296]`; dampaknya ke keterkiriman dipantau `[KEPUTUSAN-229]` | Sama | Sama | Bergantung penyedia |
| Kegagalan tidak terlihat | Alur "lupa password" menampilkan sukses walau gagal `[SRC-364]`; dideteksi lewat peringatan laju galat relay/SES dan event `EMAIL_SEND_FAILED` | Sama (perilaku Keycloak) | Coba ulang outbox menutupi sebagian kegagalan, tetapi token tersimpan | Sama (perilaku Keycloak) |
| Permukaan serangan baru | Pengurai SMTP buatan sendiri (T1, test negatif wajib), hanya terjangkau dari jaringan Docker khusus di host yang sama | Kode Java di dalam proses Keycloak (berjalan dengan hak penuh Keycloak) | Endpoint HTTP internal baru | Tidak ada di sisi kita |

## 5. Effort (estimasi penyusun, jam-task, dasar `[ASUMSI-01]`)

| Opsi | Rincian | Total |
| --- | --- | --- |
| 1 | Relay subset SMTP + STARTTLS + pembatasan (4); adaptor SES v2 Raw dengan SigV4 dan pemetaan galat, dipakai bersama notifikasi (3); konfigurasi SMTP 5 realm, truststore, Compose, IAM, configuration set (2); test integrasi Keycloak (testcontainers) → relay → tiruan SES, termasuk negatif: `From` asing, lebih dari satu penerima, ukuran, SES 429 (3–4) | ±12–13 |
| 2 | Provider Java (3); Maven di verify.sh/CI dan image Keycloak kustom (4); test (3); revisi ADR-024; adaptor SES ganda (Java dan Rust) | ±10–12, ditambah revisi ADR dan beban upgrade |
| 3 | Provider Java (2); Maven dan image (4); endpoint internal, autentikasi klien, outbox terenkripsi (4); test (4); revisi ADR-024 | ±14 |
| 4 | Opsi 1 ditambah outbox terenkripsi dan worker (3–4) | ±16 |
| 5 | Konfigurasi (1–2); pemilihan dan kontrak penyedia di luar pekerjaan teknis `[TIDAK DIKETAHUI]` | `[TIDAK DIKETAHUI]` |

F1-T36 di phase plan berukuran 4 jam-task `[DOK: 07-phase-plan.md]` §7.3. Dengan opsi 1, F1-T36 perlu dipecah (keputusan orkestrator di phase plan, di luar scope spike ini).

## 6. Rekomendasi

**Opsi 1: relay SMTP internal → SES API Jakarta, sinkron dan tanpa penyimpanan** `[KEPUTUSAN-226]`.

Alasan:
1. Tetap pada pilihan user: SES Jakarta lewat API `[USER] T5-Q4`, tanpa penyedia baru dan tanpa biaya penyedia tambahan.
2. Keycloak tetap memakai konfigurasi SMTP bawaan yang didukung `[SRC-264]` `[SRC-301]`, tanpa provider di SPI internal yang dapat berubah tanpa pemberitahuan `[SRC-299]` `[SRC-300]`.
3. Tidak menambah bahasa ke stack (ADR-024 tetap) dan sejalan dengan ADR-016 butir 5 yang sudah menyebut relay SMTP.
4. Token aksi Keycloak tidak pernah disimpan di disk atau database oleh sistem Ruang Sehat (berbeda dengan opsi 3/4).
5. Komponen relay yang sama juga melayani Alertmanager (instans di host ops) `[SRC-302]` `[KEPUTUSAN-231]` `[KEPUTUSAN-232]`.

Risiko yang diterima:
- Pengurai SMTP buatan sendiri, dimitigasi dengan subset kecil, jaringan Docker khusus per host, dan test negatif.
- Tidak ada coba ulang otomatis untuk email Keycloak. Pada jalur Admin REST (pendaftaran, kirim ulang verifikasi, undangan), kegagalan sampai ke backend sebagai HTTP 500, lalu backend menampilkan galat dan pengguna dapat meminta kirim ulang (US-101 AC2). Pada alur "lupa password", kegagalan **tidak terlihat** oleh pengguna maupun backend karena Keycloak tetap menampilkan pesan sukses `[SRC-364]`. Pengguna hanya menyadari karena email tidak datang. Kontrol deteksinya: peringatan laju galat relay/SES dan event `EMAIL_SEND_FAILED` (§7) `[KEPUTUSAN-227]`.
- Perlindungan token di ADR-026 hanya berlaku untuk email Keycloak. Email aplikasi bertoken (tautan konfirmasi reset MFA `[KEPUTUSAN-204]`) dikirim modul notifikasi lewat outbox dengan coba ulang (ADR-016), sehingga token dapat tersimpan di database dan dikirim dengan TLS oportunistik. Usulan untuk rancangan US-207 AC5 (Fase 2): outbox hanya menyimpan referensi dan token dibangkitkan saat pengiriman; configuration set wajib TLS (bila dipilih user) juga untuk email bertoken aplikasi (review C-04).

## 7. Prasyarat sebelum F1-T36

- SES production access dan kenaikan kuota sebelum staging dipakai pengguna nyata dan sebelum uji beban Januari 2027. Kuota bawaan 200 email/24 jam dan 1 email/detik `[SRC-294]` akan menolak email saat beban tinggi, dan karena pengiriman sinkron, penolakan itu langsung menjadi galat registrasi, atau kegagalan diam pada alur "lupa password" (§2.6).
- Identitas domain pengirim terverifikasi dan DKIM di `ap-southeast-3` `[SRC-210]`.
- Akun AWS khusus email dengan kebijakan IAM minimum `[USER] T5-Q4` `[SRC-298]`, ditambah prosedur rotasi dan pencabutan kunci.
- Peringatan Alertmanager (tabel ADR-010): laju galat relay/SES, event Keycloak `EMAIL_SEND_FAILED` dari worker event (ADR-011 butir 10), dan pemakaian kuota SES `[KEPUTUSAN-227]`.
- Batas laju di endpoint "lupa password" dan kirim ulang verifikasi (Caddy/WAF, ADR-019 butir 3), karena banjir permintaan dapat menghabiskan kuota SES yang dipakai bersama modul notifikasi.
- Alamat di daftar supresi SES (tetap ada sampai dihapus `[SRC-296]`) masuk prosedur penghapusan hak subjek data (EP-16).

## 8. Butuh keputusan user

1. **Persetujuan ADR-026** (opsi 1). Opsi 2 dan 3 mengubah ADR-024; opsi 5 menambah penyedia dan biaya. Ketiganya hanya bisa dipilih user.
2. **Lokasi data email** (§2.4): komitmen AWS yang ditemukan bersifat umum, dengan pengecualian "necessary to provide the services you initiated" `[SRC-295]`, dan setiap email pasti diserahkan ke server penerima yang lokasinya `[TIDAK DIKETAHUI]`. Apakah ini diterima terhadap ketentuan data di Indonesia (T4-Q1, CR-007) perlu dinilai user dan tim legal. Penilaian ini juga berlaku untuk email notifikasi aplikasi (ADR-016), bukan hanya Keycloak.
3. **Wajib TLS ke server penerima** untuk email Keycloak `[KEPUTUSAN-229]`: lebih aman bagi token, tetapi email ke server penerima tanpa TLS tidak terkirim. Kegagalan itu terjadi setelah SES menerima pesan ("It's possible for Amazon SES to accept a message without sending it" `[SRC-366]`), sehingga relay, Keycloak, dan pengguna tetap melihat sukses. Mendeteksinya memerlukan event publishing SES, yang menyimpan alamat penerima di layanan AWS lain dan menambah pertanyaan lokasi data butir 2 `[SRC-296]`. Usulan penyusun: wajib TLS. User dapat memilih oportunistik.

## 9. `[TIDAK DIKETAHUI]` dan verifikasi di F1-T36

1. Isi keluaran opsi `debug` SMTP Keycloak (`mail.debug`). Opsi ini tetap dilarang.
2. Cara menyimpan password SMTP realm tanpa menulisnya di berkas realm keycloak-config-cli (dukungan vault Keycloak untuk field SMTP, atau substitusi variabel keycloak-config-cli). Bila tidak ada, relay tidak memakai AUTH dan bergantung pada jaringan Docker khusus per host dan TLS `[KEPUTUSAN-231]`. Karena relay per host, butir ini bukan syarat `[KEPUTUSAN-232]`.
3. Kompatibilitas Alertmanager dengan relay (STARTTLS dan sertifikat CA internal).
4. Perilaku SES untuk pesan Raw bila `Destination` diisi dari amplop SMTP dan header To berbeda; diuji terhadap SES sandbox. Relay tetap memvalidasi header (§3.1).
7. Perilaku required action verifikasi email yang dipicu saat login (bukan lewat Admin REST) ketika SMTP gagal: galat ke pengguna atau sukses diam seperti §2.6.
8. Apakah OpenBao dapat menerbitkan kredensial AWS dinamis untuk akun email; IP keluar host Biznet (Magic Transit) untuk `aws:SourceIp`.
5. Penyedia SMTP di Indonesia (opsi 5) dan kebijakan port 25 keluar Biznet Gio (opsi 7).
6. Lokasi server email penerima (di luar kendali Ruang Sehat).

## 10. Register

- Sumber: SRC-294–SRC-303, SRC-364–SRC-366 (`docs/sources.md`).
- Keputusan: KEPUTUSAN-226–KEPUTUSAN-232 (`docs/decision-log.md`).
- Threat model: `docs/04-threat-model.md` v1.2 §4.18 (EML-01–EML-09, M-34–M-39, H-17).
- Bukti eksekusi: Eksperimen E1 (§2.3). Skrip penangkap SMTP disimpan di scratchpad sesi dan tidak masuk repo.
