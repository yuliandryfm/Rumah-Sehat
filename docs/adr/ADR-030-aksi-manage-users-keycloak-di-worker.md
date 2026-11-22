# ADR-030: Aksi Keycloak Admin REST yang membutuhkan `manage-users` hanya dijalankan proses `worker`, lewat niat dan job outbox dengan daftar jenis tertutup

Status: **Diterima** (2026-11-22, `[USER]` Telegram "A"; `[KEPUTUSAN-302]`). Satu butir menunggu keputusan user: butir 4, jalur password pada pendaftaran mandiri (pasien US-101, HR US-1901). Lihat bagian "Butir yang menunggu keputusan user".
Mengubah: ADR-011 butir 2 (klien Admin REST backend), butir 6 (pendaftaran dan pembuatan akun), dan butir 8 (penonaktifan di Keycloak); ADR-027 butir 3 (klien Admin REST realm `korporat`, pendaftaran mandiri dengan kompensasi). Juga KEPUTUSAN-201 kolom "alternatif ditolak" dan `docs/07-klarifikasi-ac-fase-1.md` §2.7 butir 4 poin 2 (pembuatan akun asinkron ditolak), dengan syarat butir 4 ADR ini.
Tidak mengubah: ADR-019 (topologi proses `api`/`realtime`/`worker` tetap), ADR-007 (antrean job di PostgreSQL dipakai apa adanya), ADR-009 (audit lewat outbox).
Tanggal: 2026-11-22
Tingkat ketelitian: T1 (autentikasi dan konfigurasi Keycloak, `aturan-kerja.md` §1)

## Konteks

**Temuan review yang memicu.**
- TD-26 (review F1-T65, High): klien `rs-backend-admin-manager` di realm `admin` memegang `manage-users`, sehingga dapat mereset password dan menghapus OTP semua admin, termasuk Super Admin. Gerbangnya: klien itu belum boleh dipakai proses `api` `[DOK: backlog.md]`.
- TD-30 (review F1-T50, High): klien `rs-backend-registrar` di realm `pasien` memegang `manage-users` di proses `api`. Bila `api` diretas, penyerang dapat mereset password, menghapus OTP, mengubah email, dan menghapus akun semua pasien. Gerbangnya: sebelum production `[DOK: backlog.md]`.
- Pendaftaran HR dan undangan HR di realm `korporat` (F1-T101, F1-T105, F1-T107) akan membutuhkan klien sejenis `[DOK: 07-phase-plan.md]`.

**Kemampuan Keycloak 26.8.2 yang relevan.**
- Endpoint pengguna dan izin yang diperiksanya `[SRC-842]` `[SRC-843]`:
  - butuh `requireManage`: membuat pengguna, mengubah pengguna (termasuk email dan `enabled`), menghapus pengguna, menghapus kredensial, `execute-actions-email`, `send-verify-email`, dan `logout`;
  - butuh `requireResetPassword`: hanya `reset-password`;
  - butuh `requireQuery`: mencari pengguna.
- Tidak ada peran admin realm yang lebih sempit dari `manage-users` untuk membuat pengguna atau mengirim email aksi `[SRC-844]`.
- Uji F1-T65 dengan token klien `manage-users` membuktikan `POST users`, `execute-actions-email`, `PUT enabled=false`, `logout`, dan `DELETE` berhasil, sedangkan akses ke klien, event, impersonasi, dan realm lain ditolak `[SRC-674]`.
- Tautan `execute-actions-email` sekali pakai untuk aksi bawaan dan berlaku sesuai `lifespan` `[SRC-672]`. Admin REST mewajibkan cakupan `kc-admin-api` `[SRC-625]`.
- **Fine-grained admin permissions v2 (FGAP v2)** `[SRC-840]` `[SRC-841]`:
  - diaktifkan per realm;
  - pemegang peran seperti `manage-users` melewati mekanismenya ("will skip the mechanisms provided by this feature"); di kode, `canManage` dan `canResetPassword` langsung `true` bagi pemegang `MANAGE_USERS`;
  - scope pengguna saling independen. `reset-password` jatuh ke `manage` hanya bila tidak ada permission `reset-password`, dan policy dapat berlogika `NEGATIVE`. Jadi permission tolak untuk `reset-password` mungkin dibuat, tetapi hanya berlaku untuk klien yang **tidak** memegang peran `manage-users`;
  - permission dapat dibatasi ke *All Users* atau *Specific Groups*.
- Atribut pengguna yang tidak dideklarasikan di user profile diabaikan server. Atribut tak terkelola nonaktif secara bawaan. Izin lihat dan ubah per atribut dapat dibatasi ke Admin `[SRC-845]`. Pencarian pengguna mendukung parameter `q` untuk atribut kustom `[SRC-843]`.
- Admin event dapat diambil lewat `GET admin-events` dengan filter `authClient`, `operationTypes`, `resourcePath`, dan `resourceTypes`, dengan peran `view-events` `[SRC-846]` `[SRC-621]`. Admin event sudah aktif di realm `pasien`, `admin`, dan `korporat` (`adminEventsEnabled: true`) `[DOK: infra/keycloak/realms/*.json]`.

**Topologi.**
- Yang publik hanya load balancer 443 ke Caddy, yang meneruskan ke `api` dan BFF `[DOK: adr/ADR-025-hosting-vps-openbao.md]` butir 3.
- Proses `worker` tidak membuka port HTTP. Saat ini ia menjalankan pengambil event Keycloak (F1-T34) dengan klien `rs-backend-event-reader` (`view-events`) `[DOK: apps/rs-backend/src/cli.rs]`.
- Setiap proses punya identitas OpenBao dengan policy minimum `[DOK: adr/ADR-025-hosting-vps-openbao.md]` butir 7.

**Batasan dari keputusan sebelumnya.**
- KEPUTUSAN-201 dan klarifikasi §2.7 butir 4 menolak pembuatan akun asinkron, karena password teks asli akan tersimpan di muatan job `[DOK: decision-log.md]` `[DOK: 07-klarifikasi-ac-fase-1.md]`.
- US-101 AC3 dan UP-73 mewajibkan respons pendaftaran yang tidak dapat dibedakan dan waktu respons yang ditahan 900 ms `[DOK: 03-prd.md]` `[USER] C2-B4`.
- NFR-PERF-05 menetapkan notifikasi diserahkan ke penyedia kanal ≤ 1 menit setelah kejadian `[DOK: 03-prd.md]`.

## Keputusan

### 1. Rahasia `manage-users` hanya ada di proses `worker` `[KEPUTUSAN-302]`

- Setiap realm punya satu klien layanan ber-`manage-users`:
  - `pasien`: `rs-backend-registrar`;
  - `admin`: `rs-backend-admin-manager`;
  - `korporat`: klien baru dengan pola nama yang sama, ditetapkan di task.

  Rahasianya disimpan di KV OpenBao pada path yang hanya dapat dibaca identitas OpenBao `worker` (ADR-025 butir 7). Identitas `api`, BFF, `email-relay`, dan perintah CLI tidak punya hak baca atas path itu.
- Konfigurasi `api` (`ApiConfig`) tidak memuat rahasia, alamat, atau konstruktor klien Admin REST pengguna. Perintah bootstrap Super Admin (US-208 AC4) juga hanya mencatat niat (butir 2) dan tidak lagi memegang rahasia. Arch-check menolak pemakaian tipe klien Admin REST pengguna di luar modul worker.
- Klien `rs-backend-event-reader` (`view-events`) tidak berubah.
- `worker` tetap tanpa port HTTP masuk dan berjalan minimal dua instans (ADR-019 butir 3). Job diambil dengan `FOR UPDATE SKIP LOCKED` lewat `platform-jobs` (ADR-007).

### 2. Niat dan job dicatat dalam satu transaksi beraudit `[KEPUTUSAN-302]` `[KEPUTUSAN-283]`

- `api` (atau perintah bootstrap) mencatat, di dalam satu `in_audited_tx`:
  - perubahan bisnis (akun berstatus menunggu, butir 5; bukti persetujuan; peran);
  - satu baris **niat penyediaan identitas**: realm, jenis operasi, akun, aktor pemohon, waktu;
  - event audit (E-04, E-10, E-11, E-13 sesuai use case);
  - job di `app.job`.

  Semuanya tersimpan bersama atau batal bersama.
- Muatan job hanya berisi ID niat: tanpa email, nama, password, atau data pribadi lain (sejalan TD-14). Kunci deduplikasi job diturunkan dari ID niat.
- Kelas tabel niat dan riwayat statusnya mengikuti ADR-008, ditetapkan saat task bersama DBA.

### 3. Daftar jenis job tertutup `[KEPUTUSAN-302]`

Jenis job adalah enum di kode. Jenis yang tidak dikenal digagalkan dan memicu peringatan. Susunan required action, `lifespan`, `client_id`, dan `redirect_uri` ditetapkan di kode per realm dan per tujuan, tidak pernah diambil dari muatan job atau dari baris niat.

Setiap prasyarat diperiksa terhadap **keadaan di Keycloak**, bukan hanya terhadap database aplikasi, karena database aplikasi dapat ditulis oleh `api` yang diretas.

| Jenis | Panggilan Keycloak | Prasyarat yang diperiksa worker | Dampak bila `api` diretas lalu menyisipkan job jenis ini |
| --- | --- | --- | --- |
| `idp_create_user` | `POST users`: username = email, `enabled: true`, `emailVerified: false`, atribut `rs_intent_id` = ID niat, tanpa kredensial kecuali diputuskan lain di butir 4 `[SRC-843]` `[SRC-674]` | Niat berstatus menunggu; akun berstatus `pending_provisioning` tanpa `idp_subject` | Pengguna tanpa kredensial dibuat (spam, penguasaan alamat email sebelum pemiliknya mendaftar). Tidak ada akses ke akun yang sudah ada |
| `idp_send_activation` | `PUT users/{id}/execute-actions-email?lifespan=…` `[SRC-672]` | `rs_intent_id` di Keycloak sama dengan niat; pengguna belum punya kredensial (tanpa password, atau `emailVerified: false` bila butir 4 memilih R2); batas kirim ulang US-101 AC2b | Email aksi terkirim ke alamat pemilik yang sah. Tidak ada pengambilalihan |
| `idp_disable_user` | `PUT users/{id}` `enabled: false`, lalu `POST users/{id}/logout` `[SRC-72]` `[SRC-674]` | Akun di database aplikasi sudah `inactive`, yang menurut KU-10 sudah menolak akses (ADR-011 butir 8) | Penolakan layanan. Hak ini tidak melebihi hak tulis database yang sudah dipegang `api` |
| `idp_logout_user` | `POST users/{id}/logout` | Alasan tercatat, misalnya pencabutan sesi setelah atur ulang (TD-33) | Sesi pengguna berakhir (penolakan layanan) |
| `idp_delete_orphan` (kompensasi) | `DELETE users/{id}` | `rs_intent_id` sama dengan niat yang gagal atau batal; pengguna tanpa kredensial (daftar `credentials`, butuh `view`, termasuk dalam `manage-users` `[SRC-841]` `[SRC-842]`); akun aplikasi belum aktif | Hanya pengguna yang belum pernah aktif dan dibuat oleh niat itu yang terhapus. Akun yang sudah ada tidak tersentuh (KEPUTUSAN-201 butir 2) |
| `idp_reconcile` (terjadwal) | `GET users?q=rs_intent_id:…` atau `GET users?email=…&exact=true` dengan `briefRepresentation=false` `[SRC-843]` | Nilai atribut dicocokkan persis di worker, tidak mengandalkan semantik pencarian | Hanya baca. Hasilnya diteruskan ke `idp_delete_orphan` atau dipakai untuk mengadopsi `idp_subject` |

**Dilarang bagi klien worker dan tidak termasuk daftar:**
- `reset-password`;
- menghapus kredensial;
- mengubah email, username, atau atribut pengguna yang sudah ada;
- pemetaan peran, keanggotaan grup, dan impersonasi;
- semua resource selain `users`.

Menambah jenis job berarti merevisi ADR ini dan menjalani review keamanan T1. Tiga fitur mendatang membutuhkan revisi itu:
- ubah email (US-105 AC4) dan reset MFA oleh Layanan Pelanggan (US-207 AC5). Keduanya dapat dipakai untuk mengambil alih akun, sehingga butuh analisis ancaman tersendiri;
- akhiri satu sesi (US-110).

### 4. Password pada pendaftaran mandiri: menunggu keputusan user

- **Yang berlaku apa pun pilihannya:** password teks asli tidak pernah disimpan di muatan job, baris niat, tabel mana pun, atau log. Dasar KEPUTUSAN-201 tetap berlaku.
- **Pilihan R1 (usulan):** formulir pendaftaran pasien (US-101) dan HR (US-1901) tidak lagi memuat password.
  - Tautan aktivasi dari `idp_send_activation` memuat `VERIFY_EMAIL` dan `UPDATE_PASSWORD`, sama seperti undangan admin yang sudah diuji `[SRC-674]`. Kebijakan password NFR-SEC-03 ditegakkan di halaman Keycloak, seperti aktivasi admin (US-101 AC1a sudah mencakup jalur itu) `[DOK: 03-prd.md]`.
  - Password tidak pernah melewati `api`.
  - Apakah `CONFIGURE_TOTP` ikut di tautan itu, atau tetap di login pertama (US-101 AC5), ditetapkan bersama keputusan ini.
- **Pilihan R2:** password tetap diisi di formulir.
  - `api` mengenkripsi password dengan kunci Transit khusus. `api` hanya punya hak `encrypt`, sedangkan `worker` hanya `decrypt`.
  - Ciphertext disimpan di tabel terpisah berumur pendek, lalu dihapus setelah dipakai atau kedaluwarsa. Job hanya membawa ID.
  - Risiko sisanya: ciphertext ikut WAL dan backup PITR selama 35 hari, sehingga kompromi identitas `worker` ditambah salinan backup dapat membuka password. Pilihan ini juga menafsirkan ulang dasar KEPUTUSAN-201 dan wajib direview keamanan.
- **Sampai diputuskan:**
  - jalur undangan (US-208, US-1903, US-1904 AC2), yang tidak membawa password, dapat dikerjakan;
  - pendaftaran pasien dan HR tetap memakai rancangan lama hanya di lokal dan staging. Gerbang TD-30 (sebelum production) tetap berlaku.

### 5. Keadaan akun menunggu tanpa `idp_subject` `[KEPUTUSAN-302]`

- Status akun ditambah `pending_provisioning`, dan `idp_subject` boleh kosong hanya pada status itu (constraint CHECK, ditetapkan lewat file migrasi bersama DBA).
- Akun di status ini tidak dapat masuk, karena belum ada user Keycloak.
- Email di `app.account` hanya salinan tanpa constraint unik. Keunikan email tetap ditangani Keycloak per realm `[KEPUTUSAN-46]` `[USER] T4-G3`. Agar dua pendaftaran bersamaan dengan email yang sama tidak sama-sama membuat niat dan bukti persetujuan, `api` mengambil kunci advisory transaksi atas (realm, email ternormalisasi) sebelum memeriksa akun yang belum terhapus di realm itu (butir 7). Mekanismenya dikonfirmasi DBA saat task. Kasus sisa, yaitu user Keycloak ada tetapi akun aplikasinya tidak, tertangkap lewat 409 di worker.
- Transisinya:
  - `pending_provisioning` → `pending_activation` setelah `idp_create_user` berhasil dan `idp_subject` terisi (aktivasi admin dan HR, serta email belum terverifikasi untuk pasien), lalu → `active`;
  - bila percobaan ulang habis atau galat permanen: akun dihapus lunak dengan `delete_reason` penyediaan gagal (ADR-003), `idp_delete_orphan` dijadwalkan bila user Keycloak sempat terbentuk, dan email dapat dipakai mendaftar ulang (US-101 AC1b).
- Perlakuan bukti persetujuan (ADR-004, immutable) milik akun yang gagal dibuat, misalnya entri pembalik, ditetapkan saat task bersama DBA dan Petugas PDP.

### 6. Idempotensi `[KEPUTUSAN-302]`

- **Satu niat, satu efek.** Kunci deduplikasi job per niat. Kirim ulang tautan dicatat sebagai niat baru dengan batas laju US-101 AC2b.
- **Atribut `rs_intent_id`** dideklarasikan di user profile tiap realm dengan izin lihat dan ubah hanya Admin, dan tidak dipetakan ke token `[SRC-845]`. Bila `POST users` gagal batas waktu, 5xx, atau 409:
  - worker mencari pengguna lewat atribut atau email;
  - worker mengadopsi `idp_subject` hanya bila nilai atributnya sama persis dengan ID niat;
  - bila tidak sama, kasusnya diperlakukan sebagai "email sudah terdaftar" (butir 7).
- Hasil (`idp_subject`, status niat, event audit E-04/E-10) dicatat dalam satu transaksi dengan penandaan job selesai. `complete` menerima transaksi pemanggil `[DOK: crates/platform-jobs/src/queue.rs]`, sehingga hasil tidak tercatat dua kali bila sewa job hilang.

### 7. Respons API tetap seragam `[KEPUTUSAN-302]` `[USER] C2-B4`

- **Pendaftaran mandiri (US-101, US-1901):**
  - selalu `202` dengan isi yang sama ("Cek email Anda untuk melanjutkan"), ditahan sampai 900 ms (US-101 AC3, US-1901 AC4);
  - bila di realm itu sudah ada akun belum terhapus dengan email yang sama (butir 5), `api` tidak membuat niat maupun bukti persetujuan. Ia hanya menjadwalkan email pemberitahuan "Anda sudah punya akun" lewat jalur email aplikasi, dengan batas per alamat KEPUTUSAN-192 (US-101 AC3: akun lama tidak berubah, tidak ada persetujuan baru);
  - bila user Keycloak ada tetapi akun aplikasinya tidak (yatim tanpa `rs_intent_id` yang cocok), worker menerima 409, menjadwalkan pemberitahuan yang sama, lalu menghapus lunak akun menunggu. Bukti persetujuan yang sempat tercatat diperlakukan menurut butir 5. Kasus ini adalah anomali yang juga dibersihkan rekonsiliasi.

  Karena `api` tidak lagi memanggil Keycloak di jalur ini, selisih waktu dari panggilan Keycloak (catatan P2-1 review Tahap 7) hilang.
- **Undangan admin dan HR (US-208, US-1903, US-1904 AC2):**
  - `202` dengan ID akun dan status penyediaan;
  - backoffice menampilkan status "undangan diproses", "terkirim", atau "gagal";
  - konflik email di realm `admin` tetap ditampilkan ke Super Admin, sama dengan perilaku F1-T65 untuk pengguna internal;
  - perilaku undangan HR ke email yang sudah terdaftar di realm `korporat` `[TIDAK DIKETAHUI]` (lihat bagian di bawah).

### 8. Jeda dan target penyerahan

- Pengguna akan menerima email aktivasi beberapa detik setelah respons. Target teknisnya: email diserahkan ke Keycloak, lalu ke `email-relay`, ≤ 1 menit setelah niat tercatat (NFR-PERF-05).
- Interval polling job jenis IdP ditetapkan di task, dan p95 jedanya diukur di uji integrasi.

### 9. Pemantauan dan peringatan `[KEPUTUSAN-302]`

Pengambil event F1-T34 (klien `view-events`) juga mengambil admin event per realm dengan filter `authClient` = klien worker `[SRC-846]`. Peringatan keamanan (security log, ADR-010) dibunyikan bila:
- (a) pasangan `operationType`/`resourcePath` di luar peta izin per jenis job. Contohnya `reset-password`, `credentials`, `role-mappings`, `groups`, atau resource selain `users`;
- (b) ada admin event klien worker tanpa catatan operasi worker yang cocok (ID niat, ID pengguna Keycloak, operasi, jendela waktu). Ini mendeteksi rahasia klien yang dicuri lalu dipakai di luar worker;
- (c) niat tertahan di `pending_provisioning` melewati ambang, job IdP gagal permanen, atau rekonsiliasi menemukan pengguna yatim.

Nilai `operationType`, `resourcePath`, dan `authClient` (ID internal atau `clientId`) untuk setiap panggilan diamati lewat `scripts/keycloak-check.sh`, tidak diasumsikan. Masa simpan admin event mengikuti ADR-011 butir 10.

### 10. FGAP v2 sebagai pelengkap opsional, bukan syarat penutupan TD-26/TD-30

- Klien worker dapat melepas peran `manage-users`, lalu diberi permission FGAP v2 `view` dan `manage` *All Users* ditambah permission `reset-password` berpolicy `NEGATIVE`. Dengan begitu `PUT reset-password` tertolak `[SRC-840]` `[SRC-841]`.
- Manfaatnya kecil, karena `manage` tetap mengizinkan `execute-actions-email`, penghapusan kredensial, dan perubahan email `[SRC-842]`, yang cukup untuk mengambil alih akun.
- Pembatasan per grup juga tidak mengurangi jangkauan, karena semua pengguna realm dibuat lewat worker dan termasuk satu kelas yang sama.
- Diputuskan saat task berdasarkan biaya konfigurasi (fitur per realm, policy sebagai kode, uji di `keycloak-check`).

### 11. Realm lain

Realm `nakes` (US-202) belum punya klien `manage-users`. Saat dibangun, ADR ini berlaku sebagai pola bawaan kecuali diputuskan lain lewat ADR. Backend RME adalah sistem terpisah dan memutuskan sendiri, dengan ADR ini sebagai rujukan.

## Alternatif yang ditolak

- **B: FGAP v2 saja, klien tetap di `api`.** Permission tolak hanya menutup `reset-password`. Klien yang memegang `manage` tetap dapat mengirim `UPDATE_PASSWORD`, menghapus OTP, dan mengubah email `[SRC-840]` `[SRC-842]`. Klien yang memegang peran `manage-users` malah melewati FGAP sepenuhnya `[SRC-841]`. Pengurangannya kecil, sehingga hanya dipakai sebagai pelengkap (butir 10) `[USER]` (KEPUTUSAN-302).
- **C: menerima risiko di `api` dengan peringatan admin event saja.** Kompromi `api`, satu-satunya proses backend yang menerima lalu lintas internet, langsung memberi kendali atas semua akun di tiga realm. Peringatan baru berbunyi setelah pengambilalihan terjadi `[USER]` (KEPUTUSAN-302).
- **D: layanan penyedia identitas terpisah dengan HTTP internal sinkron dari `api`.** `api` yang diretas tetap dapat memanggil operasi layanan itu, sehingga wewenangnya setara dengan menyisipkan job. Pola ini menambah layanan jaringan dan autentikasi antarproses baru, tanpa pengurangan risiko dibanding butir 3.
- **E: pendaftaran bawaan Keycloak ("User Registration" ON).** Bertentangan dengan KEPUTUSAN-201 butir 1 (jalur tidak terdokumentasi, 6.3.4 L2) dan dengan kewajiban persetujuan sebelum akun dibuat (US-101 AC4).
- **Status quo (pembuatan sinkron di `api`, worker hanya untuk kompensasi).** Rahasia `manage-users` tetap di proses yang menghadap internet (TD-26, TD-30).

## Konsekuensi

**Positif.**
- Kompromi `api` tidak lagi memberi kemampuan reset password, hapus OTP, ubah email, atau hapus akun yang sudah aktif. Dampak terburuk jenis job tertutup adalah penolakan layanan dan pembuatan pengguna tanpa kredensial (butir 3).
- Jalur pendaftaran tidak lagi bergantung pada ketersediaan Keycloak saat respons. Penyamaan waktu 900 ms lebih mudah dipenuhi.

**Negatif dan risiko sisa.**
- `worker` kini memegang `manage-users` di tiga realm. Kompromi `worker` (butuh pergerakan lateral, tanpa port masuk) setara dengan kondisi lama di `api`. Mitigasinya: rahasia hanya di identitas OpenBao `worker`, peringatan butir 9 (b), dan rotasi rahasia.
- Pengguna menunggu beberapa detik sebelum email tiba. Kegagalan penyediaan tidak lagi tampil sebagai pesan "gagal, coba lagi" di layar pendaftaran.
- Rancangan bertambah satu status akun, satu tabel niat, enam jenis job, atribut user profile, rekonsiliasi, dan peringatan baru. Semuanya masuk modul kritis T1 (Autentikasi, sesi, dan MFA).

**Dampak ke dokumen lain.**

| Dokumen | Butir | Perubahan |
| --- | --- | --- |
| ADR-011 | Butir 2 | Klien Admin REST per realm dengan peran minimum dipegang proses `worker`, bukan backend secara umum |
| ADR-011 | Butir 6 | Pembuatan user Keycloak dan kompensasinya dijalankan worker dari niat. Urutannya berubah: akun dan bukti persetujuan tercatat lebih dulu dengan status `pending_provisioning` |
| ADR-011 | Butir 8 | Penonaktifan dan logout di Keycloak lewat job `idp_disable_user`. Penolakan seketika tetap lewat database aplikasi dan cache (tidak berubah) |
| ADR-027 | Butir 3 | Klien Admin REST realm `korporat` hanya di `worker`. Pendaftaran mandiri dan undangan HR mengikuti butir 2–7 |
| ADR-025 | Butir 7 | Policy identitas `worker` ditambah baca KV rahasia klien `manage-users` per realm. Policy `api` tidak boleh memuatnya |
| KEPUTUSAN-201, klarifikasi §2.7 butir 4 | Alternatif ditolak | Pembuatan akun asinkron kini dipilih, dengan syarat password tidak disimpan (butir 4) |
| KEPUTUSAN-288 | Alasan | Endpoint pendaftaran tidak lagi memanggil Keycloak secara sinkron. Keycloak efemeral di langkah Schemathesis ditinjau ulang di task dan tidak dihapus tanpa izin user (melonggarkan verifikasi) |
| PRD US-101 | AC1, AC1b, AC2b; AC1a bila R1 | AC1: email dikirim beberapa detik setelah respons. AC1b: pesan gagal dan tombol coba lagi diganti dengan "email tidak tiba, daftar ulang atau kirim ulang". Penghapusan kompensasi tetap lewat antrean job. AC2b: kirim ulang lewat worker. Bila R1: formulir tanpa password, dan AC1a pindah ke halaman aktivasi Keycloak |
| PRD US-208 | AC1, AC4 | Status "undangan diproses" sebelum "menunggu aktivasi". Bootstrap mencatat niat, dan undangan dikirim worker |
| PRD US-1901, US-1903, US-1904 | US-1901 AC1/AC5, US-1903 AC1, US-1904 AC2/AC5 | Pendaftaran dan undangan asinkron. Penonaktifan di Keycloak menyusul, sedangkan penolakan sesi tetap seketika (KU-10). Bila R1, password HR diatur lewat tautan aktivasi |
| Backlog | TD-26, TD-30 | Tertutup setelah butir "Perubahan implementasi" 1–8 selesai dengan test negatif lulus. Sub-butir TD-26 lain (ikatan accessor dan wrapping token, audit persetujuan terpakai, runbook COMMIT tak pasti) tetap terbuka. TD-33 mendapat jalur logout sisi server (`idp_logout_user`). TD-31 terbuka setelah rute US-208 dipasang tanpa klien di `api` |
| ADR-019 | — | Tidak berubah |

## Perubahan implementasi yang dibutuhkan (task berikutnya, semuanya T1, review keamanan dan DBA)

1. **Migrasi (DBA):** status `pending_provisioning`, `idp_subject` nullable dengan CHECK, tabel niat beserta riwayat status (ADR-008), hak role database, dan konfirmasi mekanisme kunci advisory per (realm, email) (butir 5). Pertimbangkan role terpisah untuk worker IdP, sehingga status hasil hanya dapat dicatat worker.
2. **Modul worker IdP:**
   - enum jenis job (butir 3);
   - handler idempoten dengan pemeriksaan prasyarat ke Keycloak;
   - adopsi lewat `rs_intent_id`;
   - pencatatan hasil dan audit dalam satu transaksi dengan `complete`;
   - interval polling dan pengukuran p95 jeda (butir 8).
3. **Konfigurasi dan OpenBao:** `ApiConfig` dan bootstrap tanpa rahasia klien `manage-users`; `WorkerConfig` memuat rahasia per realm dari KV; policy OpenBao per identitas di `infra/openbao/`; arch-check yang menolak klien Admin REST pengguna di luar modul worker.
4. **Keycloak sebagai kode:** atribut `rs_intent_id` di user profile realm `pasien`, `admin`, dan `korporat` (hanya Admin, tanpa mapper token); klien `korporat` baru; `keycloak-check` memverifikasi atribut tidak muncul di token, nilai `operationType`/`resourcePath`/`authClient` per panggilan, dan pencarian `q`.
5. **Ubah alur F1-T65 (US-208) dan bootstrap** ke niat dan job, lalu pasang rute kelola admin di `api` (menutup gerbang TD-26 dan membuka TD-31).
6. **Ubah alur F1-T50/F1-T51 (US-101) dan F1-T101/F1-T105/F1-T107 (korporat)** ke niat dan job. Jalur password menunggu keputusan butir 4.
7. **Pengambil event F1-T34:** ambil admin event klien worker, terapkan peta izin dan pencocokan catatan operasi, lalu peringatan (butir 9).
8. **Rekonsiliasi terjadwal** (`idp_reconcile`): pengguna yatim dan niat tertahan, beserta metrik dan peringatan.
9. (Opsional, butir 10) Evaluasi FGAP v2 untuk klien worker.

## Test negatif wajib

- `api` tidak dapat memperoleh token klien `manage-users`: konfigurasi `api` tanpa rahasia gagal-tertutup, policy OpenBao `api` ditolak membaca path rahasia, dan arch-check gagal bila tipe klien dipakai di modul `api`.
- Job berjenis tidak dikenal, atau muatan berisi field selain ID niat, digagalkan dan memicu peringatan tanpa panggilan Keycloak.
- `idp_send_activation` untuk pengguna aktif berkredensial, atau dengan `rs_intent_id` berbeda, ditolak tanpa email. Susunan aksi atau `redirect_uri` dari muatan diabaikan.
- `idp_delete_orphan` untuk pengguna berkredensial, pengguna tanpa atribut, atau pengguna dengan atribut niat lain ditolak, dan akun korban utuh (KEPUTUSAN-201 butir 2).
- Pendaftaran dengan email terdaftar: respons dan waktu tidak dapat dibedakan (≥ 100 permintaan per kasus, median dan p95, klarifikasi §2.7 butir 4). Tidak ada niat baru, dan akun lama tidak berubah.
- Crash atau batas waktu setelah `POST users` sukses: percobaan ulang mengadopsi pengguna yang sama, tanpa duplikat dan tanpa email ganda.
- Galat permanen: akun dihapus lunak, kompensasi menghapus user Keycloak berdasarkan ID, dan pendaftaran ulang dengan email yang sama berhasil.
- Sewa job hilang di tengah eksekusi: hasil dan audit tidak tercatat dua kali.
- Admin event `reset-password`, `credentials`, atau `role-mappings` dari klien worker, dan admin event tanpa catatan operasi yang cocok, memicu peringatan keamanan.
- Penonaktifan akun: sesi terbuka ditolak seketika walaupun job `idp_disable_user` belum jalan atau Keycloak mati (KU-10).
- Password teks asli tidak muncul di `app.job`, tabel niat, log, atau audit (pemeriksaan pola di test integrasi).

## Butir yang menunggu keputusan user

| No | Pertanyaan | Pilihan | Usulan | Pemilik |
| --- | --- | --- | --- | --- |
| P30-1 | Jalur password pendaftaran mandiri pasien dan HR (butir 4) | R1: password diatur lewat tautan aktivasi, formulir tanpa password. R2: password terenkripsi Transit di tabel berumur pendek | R1. Password tidak pernah melewati `api`, polanya sudah teruji di realm `admin` `[SRC-674]`, dan tidak ada ciphertext password di backup | User (perubahan perilaku US-101 dan US-1901) — **Diputuskan R1** `[USER]` 2026-11-22 (KEPUTUSAN-303) |
| P30-2 | Tampilan bagi HR yang mengundang email yang sudah terdaftar di realm `korporat` (US-1904 AC2) | Tampilan sama dengan undangan baru dan pemberitahuan dikirim ke alamat itu; atau pesan "sudah terdaftar" | Tampilan sama, karena pesan "sudah terdaftar" membuka keberadaan akun HR perusahaan lain `[TIDAK DIKETAHUI]` | User / pemilik PRD — **Diputuskan: tampilan sama + pemberitahuan ke alamat itu** (usulan default; arahan user "jawab sesuai rekomendasimu", KEPUTUSAN-303) |

## Riwayat revisi

| Tanggal | Revisi | Sumber |
| --- | --- | --- |
| 2026-11-22 | Diterima: aksi `manage-users` dipindah ke `worker`, niat dan job tertutup, keadaan `pending_provisioning`, idempotensi, peringatan admin event, FGAP v2 opsional. Butir 4 menunggu P30-1 | `[USER]` Telegram "A" (KEPUTUSAN-302) |

| 2026-11-22 | P30-1 diputuskan R1 (formulir pendaftaran mandiri tanpa password; password diatur lewat tautan aktivasi), P30-2 tampilan seragam `[USER]` (KEPUTUSAN-303) |
