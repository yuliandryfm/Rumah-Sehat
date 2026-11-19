# Review Keamanan F1-T02: Jalur email Keycloak (spike dan ADR-026)

Tanggal: 2026-11-19
Reviewer: security-architect (independen, tidak ikut menyusun dokumen yang direview)
Tingkat ketelitian: T1 Kritis (autentikasi, jalur token aksi; `aturan-kerja.md` §1), 2 putaran (maksimal 2)
**Status akhir (putaran 2): LULUS DENGAN CATATAN.** B-01 dan B-02 tertutup, C-01–C-05 sudah ditangani di ADR-026, dan Lampiran A diterapkan dengan benar ke threat model v1.2. Rinciannya di §6.
Dokumen yang direview (branch `tahap-08-fase-1`, belum di-commit):
- `docs/spike/F1-T02-email-keycloak.md`
- `docs/adr/ADR-026-jalur-email-keycloak.md` (Diusulkan)
- `docs/decision-log.md` KEPUTUSAN-226–231 (baris 232–237)
- `docs/sources.md` SRC-294–SRC-303

Acuan: RT-32 `[DOK: 04-risk-register.md]`, KEPUTUSAN-183, ADR-011, ADR-016, ADR-024, ADR-025, CR-007/T4-Q1 `[USER] T4-Q1`, T5-Q4 `[USER] T5-Q4`.

Metode: kriteria blokir `aturan-kerja.md` §2. Halaman yang saya buka sendiri hari ini:
- SRC-297 (SES API v2 SendEmail): `ConfigurationOverrides.Tracking`, `EndpointId`, `Destination` (Required: No), respons 200 + `MessageId`, galat 400/429 cocok dengan klaim. Dua kutipan tambahan di bawah (R-2, R-3).
- SRC-298 (SES IAM): `ses:FromAddress`, `ses:Recipients` ("include the To:, "CC", and "BCC" addresses"), `ses:MultiRegionEndpointId` (berlaku untuk `SendEmail`, `SendBulkEmail`), `aws:SourceIp` termasuk kunci global yang didukung. Cocok dengan klaim.
- Kode `ResetCredentialEmail.java` Keycloak tag 26.8.2 (R-1, dasar blokir B-01).

SRC-294–296 dan SRC-299–SRC-303 tidak saya buka ulang; kutipan yang tercatat konsisten dengan klaim di spike dan ADR.

Nomor baris di §1–§5 merujuk ke working tree saat putaran 1. Nomor baris di §6 merujuk ke working tree saat putaran 2.

---

## 1. Verdict

**TIDAK LULUS (putaran 1): 2 blokir (B-01, B-02).** (Hasil putaran 2 di §6.) Keduanya bisa diperbaiki di tingkat teks dan tidak mengubah rekomendasi. Arah rekomendasi Opsi 1 (relay SMTP internal → SES API v2 Jakarta, sinkron, tanpa menyimpan pesan) **aman secara desain** dan saya dukung. Cek ulang cukup memeriksa baris yang berubah.

Ringkasan atas empat pertanyaan:

1. **Keamanan Opsi 1: memadai, dengan catatan.**
   - Token aksi hanya ada di memori relay.
   - Pelacakan klik SES dimatikan, sehingga tautan token tidak dialihkan.
   - Keycloak → relay memakai STARTTLS yang tidak bisa diturunkan, dengan CA internal dan verifikasi hostname.
   - Pembatasan relay berlapis: jaringan, `From`, penerima, ukuran, laju, lalu IAM sebagai lapis kedua.
   - Kunci AWS ada di OpenBao. Debug dilarang dan log tanpa badan email.

   Celah yang tersisa ada di tingkat rincian implementasi (C-01–C-05). Celah topologi menjadi blokir B-02.
2. **Klaim teknis vs SRC: cocok, kecuali satu generalisasi perilaku Keycloak yang salah (B-01).**
3. **Opsi yang ditolak: alasannya benar dan bersumber.**
   - Opsi 2/3: SPI internal (SRC-299, SRC-300) dan perubahan ADR-024.
   - Opsi 3/4: token tersimpan di database.
   - Opsi 6: di luar Indonesia (SRC-294).
   - Opsi 7: `[TIDAK DIKETAHUI]` port 25 dan reputasi.
   - Opsi 8: membangun ulang alur ADR-011.

   Opsi 2, 3, dan 5 dibawa ke user, tidak diputuskan penyusun.
4. **Butir keputusan user: disajikan jujur.**
   - Lokasi data dinyatakan "sebagian terverifikasi", termasuk pengecualian "necessary to provide the services you initiated" dan sifat penyerahan ke server penerima. Penyusun tidak menyimpulkan kepatuhan (KEPUTUSAN-230).
   - TLS wajib vs oportunistik disertai trade-off keterkiriman.

   Satu dampak yang belum disebut ada di C-05.

---

## 2. Blokir

### B-01. Klaim "kegagalan sampai ke pengguna sebagai galat" salah untuk alur "lupa password" (kriteria §2 butir 2)

**Lokasi:**
- `docs/spike/F1-T02-email-keycloak.md:150`: "Kegagalan sampai ke pengguna sebagai galat, dan pengguna dapat meminta kirim ulang".
- `docs/adr/ADR-026-jalur-email-keycloak.md:25`: "Coba ulang dan kirim ulang menjadi tanggung jawab pemanggil Admin REST dan pengguna".
- `docs/adr/ADR-026-jalur-email-keycloak.md:46`: "Keycloak sudah melaporkan kegagalan secara sinkron".
- `docs/adr/ADR-026-jalur-email-keycloak.md:57`: "Kegagalan SES muncul sebagai galat ke pengguna atau backend".
- `docs/decision-log.md:233`: alasan KEPUTUSAN-227.

**Alasan:**
- E1 hanya menguji Admin REST (`send-verify-email`, `execute-actions-email`). Perilaku 500 di jalur itu benar (spike:63).
- Alur "lupa password" di halaman login berbeda. Kode Keycloak 26.8.2 menangkap `EmailException` dan menampilkan pesan sukses yang sama. Kegagalan hanya tercatat sebagai event `EMAIL_SEND_FAILED` dan log server (R-1).
- Akibatnya, untuk alur yang paling sering dipakai pasien, kegagalan relay atau SES (kuota habis, 429) **tidak terlihat** oleh pengguna maupun backend.
- Ini mengubah "risiko yang diterima" yang dibawa ke user (§6 spike, ADR Konsekuensi), serta dasar penolakan opsi 4 di KEPUTUSAN-227.
- Perilaku required action verifikasi email saat login (bukan lewat Admin REST) belum diuji `[TIDAK DIKETAHUI]`.

Dampak keamanan positif: perilaku ini tidak membuka oracle enumerasi akun (konsisten dengan API-10). Jadi yang perlu diubah adalah klaimnya, bukan rancangannya.

**Perbaikan:**
1. Batasi klaim "kegagalan dilaporkan sinkron" hanya untuk Admin REST (pendaftaran, kirim ulang verifikasi, undangan lewat backend F1-T50/F1-T51).
2. Nyatakan bahwa "lupa password" gagal secara diam, dengan sumber kode `ResetCredentialEmail.java` 26.8.2 (dicatat sebagai SRC baru).
3. Tambahkan kontrol deteksi ke ADR-026 (butir baru atau Konsekuensi): peringatan Alertmanager atas laju galat relay/SES, serta atas event Keycloak `EMAIL_SEND_FAILED` yang sudah diambil ke audit (ADR-011 butir 10).
4. Tambahkan verifikasi perilaku required action verifikasi email ke spike §9.
5. Sesuaikan kalimat KEPUTUSAN-227. Rekomendasi tidak perlu berubah.

### B-02. Topologi "satu jaringan Docker internal untuk Keycloak dan Alertmanager" bertentangan dengan ADR-025 (kriteria §2 butir 3)

**Lokasi:**
- `docs/adr/ADR-026-jalur-email-keycloak.md:21` dan `:27`: relay "hanya mendengarkan di jaringan Docker internal yang diikuti Keycloak dan Alertmanager, tidak di antarmuka publik atau jaringan privat antar-host".
- `docs/spike/F1-T02-email-keycloak.md:81` dan `:101`.
- `docs/decision-log.md:235` (KEPUTUSAN-229) dan `:237` (KEPUTUSAN-231).

**Alasan:**
- ADR-025 (diterima) menempatkan Keycloak satu node di **app-1 dan app-2** (`docs/adr/ADR-025-hosting-vps-openbao.md:31`) dan Alertmanager di host **ops** (`:38`).
- Jaringan Docker Compose bersifat lokal per host. Karena itu satu relay tidak dapat sekaligus "hanya di jaringan Docker internal" dan melayani ketiga host.
- Ada dua cara keluar, dan keduanya mengubah perilaku keamanan yang diputuskan:
  - (a) satu relay per host (app-1, app-2, ops, stg-app). Kunci AWS ikut tersebar ke lebih banyak proses dan host, dan daftar `aws:SourceIp` mencakup semua IP keluar;
  - (b) relay dicapai lewat jaringan privat antar-host. Ini bertentangan dengan larangan di butir 4, sehingga AUTH SMTP, TLS, dan daftar izin firewall KEPUTUSAN-144 menjadi wajib, tidak lagi opsional seperti di KEPUTUSAN-231.

**Perbaikan:** tetapkan bentuk deployment di ADR-026 butir 1/4. Usulan reviewer: **(a) relay per host**, dengan rincian berikut.
- Jaringan Docker khusus yang hanya berisi relay dan Keycloak (di ops: relay dan Alertmanager). Kontainer `api`/`worker` tidak ikut jaringan itu.
- Policy OpenBao per host hanya membaca kunci SES.
- `aws:SourceIp` memuat IP keluar app-1, app-2, ops, dan host staging. IP keluar host Biznet yang sebenarnya diverifikasi di F1-T36, termasuk dampak Magic Transit `[TIDAK DIKETAHUI]` (ADR-025:74).

Bila (b) yang dipilih, butir 4 dan KEPUTUSAN-231 harus diubah supaya AUTH wajib. Mekanisme penyimpanan password SMTP realm (§9 butir 2) lalu menjadi syarat, bukan pilihan.

---

## 3. Catatan non-blokir (5 terpenting)

**C-01. Relay harus menyusun permintaan SES sendiri, bukan mempercayai header MIME.**
- SRC-297: `Destination` "Required: No". Untuk Raw: "you have to specify all of the message headers".
- Halaman yang sama menyebut header `X-SES-SOURCE-ARN` dan `X-SES-FROM-ARN` "specified in raw email message content" (R-2), jadi SES membaca header khusus dari isi Raw.
- Bila `Destination` kosong dan SES memakai header To/Cc/Bcc (§9 butir 4, `[TIDAK DIKETAHUI]`), kontrol "satu penerima" di amplop dapat dilewati lewat header.
- Usulan (AC F1-T36):
  - relay selalu mengisi `Destination.ToAddresses` dari RCPT, `FromEmailAddress` dari daftar izin, dan `ConfigurationSetName` secara eksplisit;
  - relay menolak pesan bila header `From` berbeda dengan amplop atau daftar izin, bila To/Cc/Bcc memuat alamat selain penerima amplop, atau bila ada header `X-SES-*`;
  - test negatif untuk masing-masing kondisi;
  - pemisahan CRLF ketat dan batas panjang baris di pengurai.

**C-02. Teks balasan SMTP relay harus tetap, tanpa menggemakan masukan.**
- Pada alur "lupa password", Keycloak menyimpan `e.getMessage()` sebagai detail `REASON` event `EMAIL_SEND_FAILED` (R-1). Event ini diambil ke audit (ADR-011 butir 10).
- Balasan relay yang memuat potongan header atau badan dapat membawa alamat atau token ke audit (ADR-005).
- Usulan: kode dan teks balasan tetap per kelas galat, ditambah ID korelasi acak. Rinciannya hanya ada di metrik relay.

**C-03. DoS kuota dan email bombing.**
- Kuota SES per akun dan region dipakai bersama oleh modul notifikasi (ADR-026 butir 7). Banjir permintaan "lupa password" atau kirim ulang verifikasi dapat menghabiskan kuota. Akibatnya registrasi gagal dan reset password gagal secara diam (B-01).
- Usulan:
  - batas laju per penerima di memori relay (kunci di-hash, tanpa persistensi);
  - saat batas per penerima tercapai, relay membalas 250 dan membuang pesan (dihitung di metrik), supaya tidak menjadi sinyal yang berbeda bagi pemanggil;
  - peringatan pemakaian kuota;
  - batas laju WAF/Caddy pada endpoint reset Keycloak (ADR-019 butir 3).

**C-04. Email aplikasi yang memuat token (ADR-026 butir 7) tidak ikut perlindungan yang sama.**
- Tautan konfirmasi reset MFA (KEPUTUSAN-204) dikirim modul notifikasi lewat outbox dengan coba ulang (ADR-016). Token bisa tersimpan di database dan backup, dan dikirim dengan TLS oportunistik karena configuration set "wajib TLS" hanya untuk email Keycloak.
- Kalimat Konsekuensi "Token aksi tidak pernah ditulis ke disk oleh sistem Ruang Sehat" (ADR-026:54) hanya benar untuk token Keycloak.
- Usulan:
  - outbox hanya menyimpan referensi, dan token dibangkitkan saat pengiriman (database hanya menyimpan hash);
  - configuration set wajib TLS juga berlaku untuk email bertoken aplikasi;
  - kalimat Konsekuensi dipersempit.
- Masuk ke rancangan US-207 AC5 (H-16) di Fase 2.

**C-05. Keputusan TLS wajib: satu dampak belum disebut ke user. Kunci AWS: rotasi.**
- SRC-297: "It's possible for Amazon SES to accept a message without sending it" (R-3). Kegagalan TLS ke server penerima terjadi setelah SES menerima pesan, sehingga relay dan Keycloak tetap melihat sukses dan pengguna tidak tahu emailnya tidak terkirim.
- Mendeteksinya memerlukan event publishing SES, yang menyimpan alamat penerima di layanan AWS lain. Ini menambah pertanyaan lokasi data KEPUTUSAN-230.
- Usulan: tambahkan ke spike §8 butir 3 dan ADR "Butir yang menunggu keputusan user" butir 3.
- Kunci akses IAM bersifat jangka panjang. Tambahkan prosedur rotasi dan pencabutan, serta policy OpenBao per proses (lihat B-02). Apakah OpenBao dapat menerbitkan kredensial AWS dinamis untuk kasus ini `[TIDAK DIKETAHUI]`, diperiksa di F1-T36.

Catatan kecil lain:
- Batas waktu panggilan SES di relay sebaiknya lebih pendek dari batas tulis Keycloak 10000 ms (SRC-301). Dengan begitu relay tidak membalas sukses setelah Keycloak menyerah.
- Daftar supresi SES menyimpan alamat sampai dihapus (SRC-296). Masukkan ke prosedur penghapusan hak subjek data (EP-16).

---

## 4. Threat model

Saya tidak mengubah `docs/04-threat-model.md`. Sesi review ini tidak punya alat edit parsial, dan menulis ulang seluruh berkas T1 yang sudah lulus gate (±600 baris panjang) berisiko merusak isinya. Teks yang siap disisipkan ada di **Lampiran A**, termasuk perhitungan ulang §5 dan riwayat revisi v1.2. Orkestrator atau pelaksana perlu menerapkannya dengan alat edit.

Ringkasan dampak:
- 9 ancaman baru, EML-01–EML-09 (§4.18 baru): 0 Tinggi, 8 Sedang, 1 Rendah.
- Ancaman Tinggi tetap 55, semuanya Tercermin. Status threat model **tetap LULUS**.
- Ancaman Sedang yang terbuka bertambah lima Sebagian (EML-01, 02, 03, 04, 08) dan satu Belum (EML-09).

---

## 5. Sumber baru yang perlu dicatat pelaksana di `docs/sources.md`

Saya tidak menambah nomor SRC karena hanya berkas ini yang boleh saya ubah. Nomor SRC dipilih pelaksana.

| Ref | Judul | URL | Akses | Kutipan |
| --- | --- | --- | --- | --- |
| R-1 | Keycloak kode sumber tag 26.8.2: `ResetCredentialEmail.java` | https://raw.githubusercontent.com/keycloak/keycloak/26.8.2/services/src/main/java/org/keycloak/authentication/authenticators/resetcred/ResetCredentialEmail.java | 2026-11-19 | `if (user == null) { context.forkWithSuccessMessage(new FormMessage(Messages.EMAIL_SENT)); return; }`; `} catch (EmailException e) { event.clone().event(EventType.SEND_RESET_PASSWORD).detail(Details.REASON, e.getMessage()) ... .error(Errors.EMAIL_SEND_FAILED); ServicesLogger.LOGGER.failedToSendPwdResetEmail(e); context.forkWithSuccessMessage(new FormMessage(Messages.EMAIL_SENT)); }` |
| R-2 | SES API v2 SendEmail (halaman SRC-297), kutipan tambahan | https://docs.aws.amazon.com/ses/latest/APIReference-V2/API_SendEmail.html | 2026-11-19 | "For Raw emails, the `FromEmailAddressIdentityArn` value overrides the X-SES-SOURCE-ARN and X-SES-FROM-ARN headers specified in raw email message content." `Destination`: "Required: No" |
| R-3 | Sama dengan R-2 | sama | 2026-11-19 | "It's possible for Amazon SES to accept a message without sending it." |

---

## 6. Putaran 2 (cek ulang, 2026-11-19)

Cakupan sesuai instruksi orkestrator: B-01, B-02, C-01–C-05, dan penerapan Lampiran A (dibandingkan dengan salinan sebelum perubahan `threat-model.before.md` di scratchpad sesi). Hanya baris yang berubah yang diperiksa.

### 6.1 Verdict putaran 2

**LULUS DENGAN CATATAN.** Kedua blokir tertutup dan tidak ada blokir baru. Catatan di §6.4 tidak memblokir dan tidak memicu putaran baru.

### 6.2 Status blokir

| Blokir | Status | Bukti |
| --- | --- | --- |
| B-01 | **Tertutup** | Klaim dibatasi ke Admin REST: spike:63, :166; ADR-026:10, :29–32, :70; KEPUTUSAN-227 (decision-log:233). "Lupa password" dinyatakan gagal diam dengan `[SRC-364]`, dan SRC-364 tercatat dengan URL kode yang sama dengan R-1. Kontrol deteksi baru ada di ADR-026 butir 8 (:52) dan spike:89, :174. Perilaku required action verifikasi email masuk spike §9 butir 7 (:190) dan ADR-026:32 sebagai `[TIDAK DIKETAHUI]`. Penolakan opsi 4 dirumuskan ulang dengan jujur (ADR-026:58) |
| B-02 | **Tertutup** | Relay per host (app-1, app-2, ops, stg-app) di jaringan Docker khusus, tanpa jalur antar-host: ADR-026:17, :22, :24–27, :34; KEPUTUSAN-232 (decision-log:238) dengan alternatif (b) dicatat; KEPUTUSAN-229 dan KEPUTUSAN-231 disesuaikan. AUTH wajib bila kelak lintas host (ADR-026:50). Lintas host ditolak dengan alasan yang benar (:60). Biaya kunci AWS di empat host diakui (:27, :71) dan dimitigasi (:45–46) |

### 6.3 Status catatan C-01–C-05

| Catatan | Status | Lokasi |
| --- | --- | --- |
| C-01 | Ditangani | ADR-026:35, :37 (`Destination`, `FromEmailAddress`, `ConfigurationSetName` eksplisit; tolak header `From`/To/Cc/Bcc tidak cocok dan `X-SES-*`; `[SRC-365]`); test negatif di :69 |
| C-02 | Ditangani | ADR-026:39 (teks balasan tetap + ID korelasi); test di :69 |
| C-03 | Ditangani | ADR-026:36 (per penerima, 250 lalu buang), :52 (peringatan kuota, batas laju Caddy/WAF) |
| C-04 | Ditangani sebagai usulan Fase 2 | ADR-026:51, :67 (Konsekuensi dipersempit ke token Keycloak); spike:167 |
| C-05 | Ditangani | ADR-026:84 (`[SRC-366]`, dampak ke lokasi data); rotasi dan policy per host :46; batas waktu SES < 10000 ms :29; daftar supresi ke EP-16 :52 |

SRC-364–SRC-366 tercatat dengan URL dan kutipan yang sama dengan R-1–R-3 (sources.md:360–362).

### 6.4 Penerapan Lampiran A ke threat model v1.2

**Diterapkan dengan benar.**
- Header dan Dasar (:3–4, :7), paragraf "Perubahan v1.2" (:23), AS-10 (:93), diagram (:147, :186–187), TB-11 dan TB-14 (:206, :209), §4.18 (:419–433), §5.1–§5.3 (:441–448, :478, :492), M-34–M-39 (:549–554), H-17 (:576), §8 (:612), dan §9 (:624) cocok dengan Lampiran A.
- Rujukan R-1/R-2 sudah diganti dengan SRC-364/SRC-365.
- Isi lain tidak berubah. Jumlah baris tabel ber-ID naik dari 200 ke 217, sama dengan 17 baris yang ditambahkan (TB-14, 9 EML, 6 M, 1 H), dan bagian lain yang saya periksa identik dengan versi sebelumnya.
- Status tetap LULUS: 55 Tinggi Tercermin, total 122.

### 6.5 Catatan non-blokir putaran 2

1. **Status EML bisa dinaikkan setelah ADR-026 diterima.** Mitigasi inti EML-01, EML-03, dan EML-04 kini tertulis di ADR-026 (Diusulkan). Ketiganya dapat dinaikkan ke "Tercermin (bersyarat ADR-026 diterima)", seperti EML-05, di revisi threat model berikutnya. Paragraf pengantar §4.18 (:421) sudah menjelaskan alasan status dibiarkan dulu.
2. **Butir IAM `aws:SourceIp` ditulis dua kali** di ADR-026:44 dan :45. Satukan.
3. **Alasan teks balasan tetap kurang lengkap.** Alasan di ADR-026:39 dan KEPUTUSAN-229 hanya menyebut "masuk log server Keycloak". Alasan utamanya, detail `REASON` event `EMAIL_SEND_FAILED` yang diambil ke audit (ADR-011 butir 10, `[SRC-364]`), sebaiknya disebut juga.
4. **Klaim "jaringan Docker Compose lokal per host" belum bersumber.** Klaim ini ada di KEPUTUSAN-232 (dan di B-02 putaran 1). Isinya tidak diragukan, tetapi sebaiknya diberi SRC dokumentasi Docker saat F1-T36 menetapkan konfigurasi jaringan.
5. **Estimasi effort opsi 1 belum disesuaikan.** Spike §5 (:145) belum memperhitungkan empat instans dan test negatif tambahan (header, `X-SES-*`, teks balasan). Nilai ulang saat F1-T36 dipecah (sudah disebut di ADR-026:77).

---

## Lampiran A. Teks siap sisip untuk `docs/04-threat-model.md` v1.2

**A.1 Header (baris 3–4).** Ganti tanggal menjadi 2026-11-19 dan versi menjadi "1.2 (2026-11-19, review keamanan F1-T02: jalur email Keycloak ADR-026 Diusulkan; menunggu gate berikutnya). Sebelumnya 1.1 ...". Tambahkan ke baris Dasar: "Dasar tambahan v1.2: `docs/spike/F1-T02-email-keycloak.md`, ADR-026 (Diusulkan), KEPUTUSAN-226–231, `docs/review/F1-T02-security.md`".

**A.2 Paragraf baru setelah "Perubahan v1.1".**

> **Perubahan v1.2 (F1-T02).** Komponen baru relay email Keycloak (ADR-026, berstatus Diusulkan) dinilai di §4.18 (EML-01–EML-09), dengan boundary baru TB-14 dan AS-10 yang diperluas. Ancaman berlaku bila ADR-026 diterima. Bila user memilih opsi lain, EML-01, EML-03, EML-05, dan EML-07 dinilai ulang. EML-02, EML-04, EML-06, EML-08, dan EML-09 berlaku untuk opsi mana pun. Tidak ada ancaman Tinggi baru. Status tetap LULUS.

**A.3 AS-10 (§1).** Tambahkan pada kolom Aset: ", token aksi Keycloak di badan email (verifikasi email, atur ulang password, undangan aktivasi) (v1.2)". Tambahkan pada kolom Lokasi utama: ", memori relay email selama satu request, kotak surat penerima".

**A.4 Diagram §3.1.** Tambahkan node `ER[relay email]` di subgraph `APPZ`, lalu dua baris berikut:
```
  KC -->|TB-14 SMTP STARTTLS| ER
  ER -->|TB-11 SES API| EM
```

**A.5 Baris baru §3.2.**

| TB | Batas | Yang melintas | Kontrol utama di desain | Ancaman terkait |
| --- | --- | --- | --- | --- |
| TB-14 | Keycloak (dan Alertmanager) ke relay email, di jaringan kontainer lokal host (v1.2; topologi per host menunggu perbaikan B-02 `docs/review/F1-T02-security.md`) | Pesan MIME berisi token aksi, alamat penerima, peringatan operasional | STARTTLS wajib dengan CA internal dan verifikasi hostname, `debug` dilarang (ADR-026 butir 6); jaringan khusus, satu penerima, daftar izin `From`, batas ukuran dan laju, tanpa log isi (ADR-026 butir 4) | EML-01, EML-03, EML-05, EML-07 |

Tambahkan juga di sel "Yang melintas" TB-11: ", pesan MIME email Keycloak lewat relay (v1.2)". Tambahkan ke "Ancaman terkait" TB-11: ", EML-02, EML-04, EML-06, EML-08".

**A.6 Bagian baru setelah §4.17.**

> ### 4.18 Relay email Keycloak dan jalur SES (TB-14, TB-11), baru di v1.2
>
> Dasar: ADR-026 (Diusulkan), spike F1-T02 (eksperimen E1: token aksi `login-actions/action-token?key=...` ada di badan email), `docs/review/F1-T02-security.md` (B-01, B-02, C-01–C-05).

| ID | STRIDE | Aktor | Ancaman | K | D | Tingkat | Mitigasi dalam desain | Tercermin di | Status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| EML-01 | S, E | AK-07, penyerang yang sudah menguasai satu kontainer di jaringan internal | Relay disalahgunakan sebagai open relay internal untuk mengirim phishing ber-DKIM sah atas nama Ruang Sehat ke alamat sembarang, termasuk lewat header To/Cc/Bcc tambahan atau header `X-SES-*` yang dibaca SES dari pesan Raw `[SRC-297]` | R | T | Sedang | Jaringan kontainer khusus; satu penerima; daftar izin `From`; batas ukuran dan laju; IAM `ses:FromAddress`, ARN identitas Jakarta, `aws:SourceIp`, penolakan `ses:MultiRegionEndpointId` `[SRC-298]`. Belum: topologi per host (B-02); permintaan SES disusun dari amplop dan header divalidasi (C-01) | ADR-026 butir 4–5 (Diusulkan) | **Sebagian** (M-34) |
| EML-02 | I | AK-05 dengan posisi jaringan antara SES dan server penerima, AK-06 | Token aksi di badan email terbaca selama pengiriman (TLS oportunistik bawaan SES `[SRC-296]`) atau lewat domain pelacakan tautan `[SRC-294]`. Undangan aktivasi nakes atau admin yang dicuri memungkinkan penyerang menetapkan password dan TOTP sendiri | R | T | Sedang | Pelacakan klik dan buka dimatikan `[SRC-297]`; configuration set wajib TLS diusulkan (menunggu user); atur ulang password tetap meminta OTP (ADR-011 butir 5); undangan Super Admin pertama sekali pakai 24 jam (US-208 AC4). Belum: masa berlaku undangan umum di semua realm | ADR-026 butir 5 (Diusulkan); ADR-011 butir 5; US-208 AC4 | **Sebagian** (M-38; bila user memilih TLS oportunistik, dicatat sebagai risiko sisa diterima user) |
| EML-03 | I | AK-07 | Isi email atau token bocor lewat log relay, opsi `debug` SMTP Keycloak (`mail.debug` `[SRC-301]`), atau teks balasan galat relay yang disimpan Keycloak sebagai detail `REASON` event `EMAIL_SEND_FAILED` lalu diambil ke audit (kode `ResetCredentialEmail.java` 26.8.2, R-1 di `docs/review/F1-T02-security.md`) | R | T | Sedang | Relay tidak mencatat badan, subjek, alamat; `debug` dilarang dan diperiksa (ADR-005, ADR-010). Belum: teks balasan SMTP tetap tanpa menggemakan masukan | ADR-026 butir 4, 6 (Diusulkan) | **Sebagian** (M-35) |
| EML-04 | D | AK-05 | Banjir permintaan "lupa password" atau kirim ulang verifikasi, atau lonjakan notifikasi aplikasi pada akun SES yang sama, menghabiskan kuota/laju SES `[SRC-294]`. Registrasi gagal, dan reset password gagal **diam-diam** karena Keycloak tetap menampilkan pesan sukses (R-1); email bombing ke satu korban | S | S | Sedang | Kenaikan kuota sebelum staging dipakai (prasyarat spike §7); batas laju relay; WAF (ADR-019 butir 3). Belum: batas per penerima, peringatan `EMAIL_SEND_FAILED` dan laju galat relay | ADR-026 butir 4, Konsekuensi (Diusulkan) | **Sebagian** (M-36) |
| EML-05 | S, I | AK-07, kontainer terkompromi | Relay palsu atau penyadap di jaringan internal menerima email Keycloak dan memanen token aksi | R | T | Sedang | `starttls` memaksa `mail.smtp.starttls.required`, sertifikat relay dari CA internal lewat `truststore-paths`, `tls-hostname-verifier` `DEFAULT` `[SRC-301]`; test negatif koneksi tanpa STARTTLS | ADR-026 butir 6, Konsekuensi (Diusulkan) | Tercermin (bersyarat ADR-026 diterima; kompatibilitas Alertmanager diverifikasi F1-T36) |
| EML-06 | S, I | AK-07, AK-05 | Kunci akses AWS akun email bocor, lalu dipakai mengirim email atas nama domain resmi | R | T | Sedang | Akun AWS khusus email `[USER] T5-Q4`; kunci di OpenBao KV hanya untuk relay dan modul notifikasi `[KEPUTUSAN-79]`; IAM hanya kirim, ARN identitas, `ses:FromAddress`, `aws:SourceIp` `[SRC-298]` | ADR-026 butir 5 (Diusulkan); ADR-013 butir 8 | Tercermin; penguatan H-17 |
| EML-07 | T, D | AK-01, AK-05 (lewat nilai yang dikendalikan pengguna, misalnya nama dan alamat email, yang masuk ke pesan) | Pengurai SMTP buatan sendiri salah menangani batas baris, akhir DATA, atau ukuran, sehingga terjadi injeksi perintah atau relay berhenti (semua email autentikasi gagal) | R | S | Rendah | Subset perintah kecil; batas ukuran; hanya terjangkau dari jaringan internal; test negatif wajib (modul T1) | ADR-026 butir 2, 4, Konsekuensi (Diusulkan) | Tercermin; detail CRLF ketat di M-34 |
| EML-08 | I | AK-06 | Alamat dan isi email diproses atau disimpan di luar Indonesia: komitmen AWS bersifat umum dengan pengecualian "necessary to provide the services you initiated" `[SRC-295]`; daftar supresi menyimpan alamat sampai dihapus `[SRC-296]`; server penerima di lokasi `[TIDAK DIKETAHUI]` `[USER] T4-Q1` | S | S | Sedang | Region `ap-southeast-3`; tanpa `EndpointId` multi-region dan penolakan IAM-nya `[SRC-297]` `[SRC-298]`. Kesesuaian dengan T4-Q1/CR-007 dibawa ke user dan tim legal | ADR-026 butir 4–5 (Diusulkan); KEPUTUSAN-230 | **Sebagian** (menunggu user dan legal; M-39) |
| EML-09 | I | AK-07, AK-05 | Email aplikasi bertoken (tautan konfirmasi reset MFA KEPUTUSAN-204) dikirim modul notifikasi lewat outbox dengan coba ulang (ADR-016), sehingga token tersimpan di database dan backup, dan dikirim dengan TLS oportunistik karena configuration set wajib TLS hanya untuk email Keycloak | S | S | Sedang | Tautan sekali pakai 60 menit dan password tetap diminta (KEPUTUSAN-204), sehingga token saja tidak cukup | KEPUTUSAN-204; ADR-026 butir 7 (Diusulkan) | **Belum** (M-37) |

**A.7 §5.1.** Ganti kalimat pembuka menjadi: "Angka v1.2. Dibanding v1.1: EML-01–EML-09 ditambahkan (8 Sedang, 1 Rendah)." Ganti tabelnya:

| Tingkat | Jumlah | Tercermin | Sebagian | Belum |
| --- | --- | --- | --- | --- |
| Tinggi | 55 | 55 | 0 | 0 |
| Sedang | 61 | 40 | 16 | 5 |
| Rendah | 6 | 5 | 1 | 0 |
| **Total** | **122** | **100** | **17** | **5** |

**A.8 §5.2.** Tambahkan baris `| 4.18 Relay email Keycloak dan SES | 9 | 0 | — |`. Pada §5.3, baris "Kebocoran lewat log, notifikasi, URL bertanda tangan", tambahkan "EML-03, EML-09" di kolom Ancaman dan "; EML-03 Sebagian, EML-09 Belum (Sedang)" di kolom Status.

**A.9 Baris baru §6.2.**

| ID | Ancaman | Mitigasi konkret yang diusulkan | Usulan penempatan | Status |
| --- | --- | --- | --- | --- |
| M-34 | EML-01, EML-07 | (a) Satu relay per host di jaringan kontainer khusus yang hanya berisi relay dan Keycloak (di ops: relay dan Alertmanager), atau bila relay lintas host, AUTH wajib dan daftar izin firewall KEPUTUSAN-144. (b) Relay mengisi `Destination.ToAddresses` dari RCPT, `FromEmailAddress` dari daftar izin, dan `ConfigurationSetName` secara eksplisit. (c) Pesan ditolak bila header `From` tidak cocok, To/Cc/Bcc memuat alamat selain penerima amplop, atau ada header `X-SES-*` `[SRC-297]`. (d) CRLF ketat dan batas panjang baris. (e) Test negatif untuk (b)–(d) | Revisi ADR-026 butir 1, 4 (B-02, C-01); AC F1-T36 | Baru |
| M-35 | EML-03 | Kode dan teks balasan SMTP relay tetap per kelas galat ditambah ID korelasi acak, tanpa potongan header, badan, atau alamat; test bahwa detail event `EMAIL_SEND_FAILED` Keycloak tidak memuat alamat atau token | Revisi ADR-026 butir 4 (C-02); AC F1-T36 | Baru |
| M-36 | EML-04 | Batas laju per penerima di memori relay (kunci di-hash); saat batas tercapai, relay membalas 250 dan membuang pesan supaya tidak muncul sinyal berbeda; peringatan Alertmanager untuk laju galat relay/SES, event `EMAIL_SEND_FAILED`, dan pemakaian kuota; batas laju di endpoint reset Keycloak | Revisi ADR-026 (B-01, C-03); tabel ADR-010; ADR-019 butir 3 | Baru |
| M-37 | EML-09 | Outbox notifikasi hanya menyimpan referensi, token dibangkitkan saat pengiriman dan database hanya menyimpan hash; configuration set wajib TLS (bila dipilih user) juga untuk email bertoken aplikasi; kalimat ADR-026 Konsekuensi dipersempit ke token Keycloak | Revisi ADR-026 butir 7 (C-04); rancangan US-207 AC5 bersama H-16 (Fase 2) | Baru |
| M-38 | EML-02 | Masa berlaku undangan `execute-actions-email` untuk semua realm ditetapkan (usulan ≤ 24 jam, setara US-208 AC4) dan sifat sekali pakainya diverifikasi di Keycloak 26.8.2; aktivasi undangan memberi tahu admin pembuat; keputusan TLS wajib dari user | ADR-011 butir 4 atau ADR-026; F1-T50/F1-T51; keputusan user ADR-026 butir 3 | Baru |
| M-39 | EML-08 | Penilaian lokasi data oleh user dan legal (KEPUTUSAN-230); alamat di daftar supresi SES dihapus sebagai bagian dari penghapusan/anonimisasi hak subjek data; event publishing SES hanya ke tujuan di region Jakarta, bila dipakai | Keputusan user; EP-16; ADR-013 | Baru |

**A.10 Baris baru §6.3.**

| ID | Ancaman | Penguatan | Penempatan | Status |
| --- | --- | --- | --- | --- |
| H-17 | EML-06 | Prosedur rotasi dan pencabutan kunci akses AWS akun email; policy OpenBao per proses/host hanya membaca kunci SES; `aws:SourceIp` memuat semua IP keluar host relay dan notifikasi; kemungkinan kredensial AWS dinamis dari OpenBao diperiksa (`[TIDAK DIKETAHUI]`) | ADR-026 butir 5; ADR-025; F1-T36 | Terbuka |

**A.11 §8 Status.** Tambahkan:

> Versi 1.2: seluruh 55 ancaman Tinggi tetap berstatus Tercermin. Sembilan ancaman relay email (EML-01–EML-09, 8 Sedang dan 1 Rendah) ditambahkan. Lima di antaranya Sebagian dan satu Belum, dengan mitigasi M-34–M-39 dan H-17. Butir yang bergantung pada keputusan user: penerimaan ADR-026, TLS wajib (EML-02), dan lokasi data (EML-08). Status **LULUS**.

**A.12 §9 Riwayat revisi.**

| Versi | Tanggal | Perubahan | Sumber |
| --- | --- | --- | --- |
| 1.2 | 2026-11-19 | Review keamanan F1-T02:<br>- §4.18 baru (EML-01–EML-09);<br>- TB-14 baru, TB-11 diperluas;<br>- AS-10 diperluas;<br>- M-34–M-39 dan H-17 baru;<br>- §5 dihitung ulang (122 ancaman).<br>Status tetap LULUS | `docs/spike/F1-T02-email-keycloak.md`; ADR-026 (Diusulkan); KEPUTUSAN-226–231; `docs/review/F1-T02-security.md` |
