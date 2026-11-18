# Test Plan dan Test Case: Ruang Sehat

Tanggal: 2026-11-18
Versi: 0.2 (tindak lanjut Checkpoint 2, disesuaikan dengan PRD v1.2 final)
Penyusun: subagent `qa-engineer` (independen dari developer), tingkat ketelitian T2; T1 untuk test autentikasi, MFA, sesi, dan identitas; test case modul kritis T1 memuat skenario negatif (`aturan-kerja.md` §1)
Dasar: PRD v1.2 final `[DOK: 03-prd.md]` (riwayat §14; klarifikasi AC Fase 1 `[DOK: 07-klarifikasi-ac-fase-1.md]`, CR-017, CR-018, US-110) (acceptance criteria, AB, RF, PB, KU, NFR, §7), phase plan v0.1 `[DOK: 07-phase-plan.md]`, backlog `[DOK: backlog.md]`, coding standards §7 `[DOK: coding-standards.md]`, stack `[DOK: 05-stack.md]`, checklist ASVS `[DOK: 05-asvs-checklist.md]`
Kode belum ada. Seluruh test case disusun dari PRD, bukan dari kode. Hasil eksekusi (lulus/gagal) belum ada dan dicatat di laporan fase Tahap 8.

**Label sumber.** `[DOK: ...]` dokumen proyek. `[USULAN-QA]` usulan penyusun dokumen ini yang perlu disetujui di Checkpoint 2. `[TIDAK DIKETAHUI]` tidak ada di spesifikasi, dicatat di §9 (Temuan), tidak ditebak. Setiap test case menyebut AC sumbernya di kolom "AC" (misalnya "US-103 AC3"). Tidak ada test case untuk perilaku yang tidak ada di PRD; perilaku penting yang belum diatur ada di §9.

---

## 1. Tujuan dan ruang lingkup

- Membuktikan bahwa setiap acceptance criteria (AC) story Must, dan story Should yang ada di fase yang direncanakan, terpenuhi, termasuk kondisi gagal, batas nilai, dan penolakan akses.
- Membuktikan kriteria umum KU-01–KU-10 (PRD §4), NFR (PRD §6), dan spesifikasi audit trail (PRD §7).
- Dipakai F1-T71 (functional test QA penutupan Fase 1) dan task penutupan fase berikutnya `[DOK: 07-phase-plan.md]` §4, §7.3.
- Di luar dokumen ini: audit keamanan eksternal (Tahap 10), UAT (Tahap 11), dan kewajiban kepatuhan non-perangkat lunak (PRD §10). Dokumen ini menyiapkan bukti untuk keduanya.
- Cakupan: 97 story aktif (92 Must termasuk US-110 yang ditambahkan di Checkpoint 2, 5 Should: US-108, US-109, US-405, US-602, US-1305). US-102 berstatus Won't (CR-007) dan tidak diuji.

## 2. Strategi

### 2.1 Level test dan alat

Alat mengikuti yang sudah ditetapkan, tanpa alat baru `[DOK: coding-standards.md]` §7.1 `[DOK: 05-stack.md]`.

| Level (kolom "Level") | Cakupan | Alat |
| --- | --- | --- |
| unit | Domain murni: transisi status, harga/markup/pajak, batas waktu, masking audit, validasi, format ciphertext | `cargo test`, `flutter test`, test unit web |
| integrasi | Service + repository pada PostgreSQL 18, Valkey, Keycloak, OpenBao sungguhan dengan role aplikasi (bukan pemilik skema); endpoint HTTP; kontrak OpenAPI | testcontainers, Schemathesis (kontrak) |
| E2E | Alur utama per aplikasi web dan mobile terhadap stack lokal | Playwright (web), integration test Flutter (mobile) |
| manual | Yang tidak praktis diotomasi: kamera/galeri, TalkBack/VoiceOver, perangkat nyata, tinjauan dokumen/konfigurasi penyedia | Eksekusi QA mengikuti langkah di test case; bukti tangkapan layar/log |

Uji beban NFR-CAP-04: alat uji beban belum ditetapkan di stack `[TIDAK DIKETAHUI]` (§9, T-30). Verifikasi akhir tiap fase memakai `bash scripts/verify.sh`, output ditunjukkan di laporan fase `[DOK: CLAUDE.md]`.

Penanda keterlacakan: setiap test otomatis diberi komentar `trace: US-xxx ACn` dan `tc: TC-...` `[DOK: coding-standards.md]` §7.4. Kolom "Test case" di `docs/traceability.md` diisi dari ID dokumen ini.

### 2.2 Lingkungan

| Lingkungan | Kegunaan | Syarat |
| --- | --- | --- |
| Lokal (`infra/local/compose.yaml`) dan CI (testcontainers) | Semua test unit, integrasi, E2E Fase 1 | PostgreSQL 18, Valkey, Keycloak (realm sebagai kode), OpenBao dev; penangkap email untuk test (F1-T36) `[DOK: 07-phase-plan.md]` |
| Staging | Mulai Fase 2: E2E lintas komponen, sandbox DOKU, LiveKit, uji beban (Januari 2027) | PT-39 `[DOK: 07-phase-plan.md]` §9 |
| Sandbox pihak ketiga | DOKU (PT-36), SATUSEHAT (PT-32), kurir (PT-37), email | Bila sandbox belum ada, test memakai dobel uji (fake) yang meniru kontrak publik penyedia, ditandai "menunggu sandbox", tidak dihapus |
| Production | Tidak ada test fungsional; hanya smoke test Tahap 13 | Tidak ada data uji di production |

### 2.3 Data uji

- Hanya data sintetis di semua lingkungan non-production; tidak ada data pribadi asli dan tidak ada salinan data production `[DOK: coding-standards.md]` §7.2.
- Pembangkit di crate `test-support` (dan padanan web/mobile). NIK sintetis 16 digit dibuat pembangkit itu. Email uji memakai domain yang dicadangkan RFC 2606 (mengikuti coding standards §7.2; dicek ulang di Tahap 8). Foto KTP/KK uji adalah gambar sintetis tanpa wajah atau identitas nyata.
- Data minimal: pasien A dan B; satu pasien per status AB-05; Verifikator 1 dan 2; satu akun per sub-peran admin; (Fase 2+) rekanan A dan B, dua tenaga kesehatan per jenis; (Fase 3) dua fasyankes RME.
- Waktu memakai `Clock` yang dapat diatur dan minimal dua zona waktu Indonesia (WIB, WITA) `[DOK: coding-standards.md]` §7.3. Pengecualian: jam internal Keycloak (penguncian 15 menit, kedaluwarsa tautan) diuji dengan jam nyata atau konfigurasi uji; bila terlalu lambat untuk `verify.sh` dicatat sebagai task, tidak di-skip `[DOK: coding-standards.md]` §7.1.
- Token uji dari Keycloak di testcontainers atau kunci uji di `test-support`.
- Kode TOTP uji dibangkitkan dari rahasia TOTP akun uji oleh pembangkit di `test-support` (MFA wajib semua realm, NFR-SEC-02). Pustakanya dipilih di Tahap 8 dengan cek registry resmi dan catatan di `docs/sources.md`; belum ada dependensi yang ditetapkan di dokumen ini. Kode pemulihan uji hanya ada di lingkungan uji dan tidak pernah dicatat di log test.

### 2.4 Tingkat ketelitian dan penanda T1

- Kolom "T1" bernilai `T1` bila test case menguji modul kritis T1 (`CLAUDE.md`): autentikasi/sesi/MFA, otorisasi dan isolasi data, harga/pembayaran/refund/klaim/kewajiban rekanan, audit, enkripsi/retensi, persetujuan dan hak subjek data, Partner Integration API, berkas identitas/klinis, RME dan SATUSEHAT, migrasi dan hak role. Selain itu `-`.
- Setiap story yang menyentuh T1 memuat minimal satu test `negatif` per AC relevan, ditambah skenario wajib endpoint (§3.2) dan skenario negatif modul T1 (coding standards §7.3).
- Tipe: `positif` (jalur sukses), `negatif` (penolakan, kegagalan, akses ditolak), `batas` (nilai atau waktu di tepi aturan).

### 2.5 Cara membaca test case

Kolom: ID `TC-<story>-<n>` | AC sumber | skenario ringkas (`G:` Given, `W:` When, `T:` Then) | tipe | level | T1 | fase.

Fase mengikuti `docs/07-phase-plan.md` §4. AC yang diverifikasi di fase lain ditandai di kolom fase `[DOK: 07-phase-plan.md]` prinsip 6 (KEPUTUSAN-179). Test case yang bergantung pada keputusan terbuka ditandai `(menunggu: ...)` dan dicatat di §9.

### 2.6 Pelaporan bug dan severity `[USULAN-QA]`

Setiap bug memuat: judul, langkah reproduksi, hasil diharapkan, hasil aktual, severity, rujukan story/BR/AC, dan bukti (output perintah atau log).

| Severity | Definisi |
| --- | --- |
| Critical | Kebocoran atau perubahan data lintas pengguna/tenant; bypass autentikasi/otorisasi; data terlarang (§7.3) masuk audit/log; uang salah (pembayaran, refund, pembagian); audit dapat diubah; kehilangan data |
| High | AC Must tidak terpenuhi di jalur utama; aksi sensitif tanpa audit; kegagalan tanpa pemulihan |
| Medium | AC terpenuhi sebagian atau gagal hanya di kondisi tepi; pesan galat salah |
| Low | Tampilan, teks, atau ketidaksesuaian kecil tanpa dampak fungsi |

### 2.7 Kriteria masuk dan keluar per fase `[USULAN-QA]`

| Fase | Kriteria masuk | Kriteria keluar |
| --- | --- | --- |
| Fase 1 | Checkpoint 2 disetujui; `scripts/verify.sh` disetujui user (F1-T05); stack lokal jalan; pembangkit data sintetis ada | Semua TC Fase 1 Lulus; 0 bug Critical/High terbuka; bug Medium/Low punya keputusan; `verify.sh` hijau (output ditunjukkan); tiap AC Fase 1 punya test bertanda `trace:`; semua TC T1 negatif Lulus; demo end-to-end MVP dengan rantai audit terverifikasi; kolom "Test case" di traceability cocok dengan kode (pencarian teks) |
| Fase 2 | Fase 1 keluar; keputusan gerbang re-plan; staging (PT-39); sandbox DOKU (PT-36) atau fake bertanda "menunggu sandbox"; EN-24 pajak selesai sebelum test US-801 | Seperti Fase 1 untuk TC Fase 2, ditambah regresi TC Fase 1; uji uang memakai nominal tepat tanpa pembulatan tersembunyi; TC chat/video lulus di staging |
| Fase 3 | Fase 2 keluar; PT-05 atau sandbox rekanan/RME; PT-32 untuk US-1309 (bila belum, TC "Menunggu kredensial") | Seperti Fase 1, ditambah isolasi tenant RME dan portal lulus, skenario kontrak gateway lulus untuk sistem rekanan dan RME, uji beban NFR-CAP-04 memenuhi NFR-PERF |
| Fase 4 | Gerbang re-plan memilih Opsi A (atau Opsi B dengan US-108/US-109 di Fase 3); PT-29/30/33/37 terjawab | Seperti Fase 1, ditambah regresi |
| Tahap 9 (QA + SIT) | Pembekuan fitur 2027-01-29 | Regresi seluruh TC lulus; SIT dengan rekanan pertama; seluruh NFR di §3.3 terverifikasi |

TC yang AC-nya terblokir keputusan eksternal (US-1605, US-1606; teks final US-104 AC1) berstatus "Terblokir" dan tidak menghalangi keluar fase selama blokirnya tercatat di gerbang `[DOK: 07-phase-plan.md]` §9 (G-13, G-14); ia menghalangi go-live.

### 2.8 Peran QA di akhir fase

QA independen dari developer dan tidak mengubah kode aplikasi. Di akhir tiap fase, dalam task penutupan fase (F1-T71 untuk Fase 1), QA:
1. Menjalankan `bash scripts/verify.sh` dan melampirkan output.
2. Menjalankan seluruh TC fase (otomatis lewat `verify.sh`; manual sesuai langkah), mencatat Lulus/Gagal/Terblokir dengan bukti.
3. Memeriksa keterlacakan: tiap AC fase punya test bertanda `trace:`; ID di kolom "Test case" `docs/traceability.md` ada di kode (pencarian teks).
4. Pengujian eksploratori edge case (input kosong, batas nilai, duplikat, hak akses ditolak, jaringan gagal, operasi bersamaan, soft delete lalu restore).
5. Sampling audit trail: untuk tiap aksi sensitif yang diuji, field §7.2 benar dan tidak ada data terlarang §7.3.
6. Melaporkan bug dan celah spesifikasi (tidak memperbaiki kode), memperbarui §9, dan menandatangani kriteria keluar di laporan fase.

### 2.9 Ringkasan jumlah

Dihitung dari baris `| TC-` di dokumen ini dengan skrip pencarian teks (jumlah dapat bergeser bila test case ditambah di Tahap 8).

| Ukuran | Jumlah |
| --- | --- |
| Total test case | 805 (versi 0.1: 674; +131) |
| Fase 1 (termasuk TC lintas story F1 di §3; TC bertanda "F1–F3" dihitung di F1) | 276 (T1: 259; negatif: 165) |
| Fase 2 | 245 |
| Fase 3 | 205 (4 terblokir PT-24/L-06, lihat T-25) |
| Fase 4 | 77 |
| Di luar fase pengembangan (pemeriksaan go-live, uji pemulihan) | 2 |
| Bertanda T1 | 678 |
| Tipe negatif | 431 (T1 negatif: 397) |
| Tipe batas | 132 |
| Tipe positif | 242 |
| Level: integrasi / E2E / unit / manual | 633 / 104 / 52 / 16 |
| Story Must tanpa test case | 0 dari 92 (5 story Should juga punya test case) |
| AC tanpa test case | 10 AC yang isinya `[TIDAK DIKETAHUI]` (T-28): US-203 AC4, US-205 AC4 dan AC5, US-308 AC4, US-309 AC5, US-702 AC4, US-803 AC4, US-810 AC3, US-1307 AC3, US-1602 AC3, US-1604 AC3 |
| TC menunggu keputusan (ditandai "menunggu") | TC-US-201-24–26 (T-38, blokir); TC-US-106-27, -29 (T-37); TC-US-101-22 (T-39); TC-US-101-29, TC-US-103-42, -47 (mekanisme Keycloak, Tahap 8); TC-US-1501-18, TC-US-1504-11 (daftar penerima); TC-NFR-SEC-03-2 (daftar kata sandi bocor) |

Tambahan versi 0.2 per story: US-101 +15, US-103 +32, US-104 +2, US-105 +11, US-106 +12, US-110 +12 (baru), US-201 +8, US-207 +9, US-208 +13, US-1501 +2, US-1504 +2; lintas story KU +2, NFR +11.

---

## 3. Test case lintas story dan NFR

### 3.1 Kriteria umum KU-01–KU-10 (PRD §4)

KU diuji sebagai pola umum di Fase 1, lalu diulang per story baru lewat test ber-`trace:` story itu. Kolom fase = fase pertama KU dapat diuji.

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-KU-01-1 | KU-01 | G: formulir dengan field wajib kosong dan satu field berformat salah; W: simpan; T: ditolak, pesan di setiap field salah, isian yang benar tidak hilang | negatif | E2E | - | F1 |
| TC-KU-01-2 | KU-01 | G: permintaan API dengan body tidak valid (tipe salah, field hilang, string sangat panjang); W: kirim; T: ditolak dengan `response_code` validasi dan pesan per field, tanpa galat 500 | negatif | integrasi | - | F1 |
| TC-KU-02-1 | KU-02 | G: koneksi diputus; W: aksi pengguna; T: pesan "gagal terhubung" dan tombol coba lagi tampil | negatif | E2E | - | F1 |
| TC-KU-02-3 | KU-02 | G: server uji menunda respons; W: aksi biasa dengan tunda 14 dan 16 detik, unggah berkas dengan tunda 59 dan 61 detik; T: tunda 16 detik dan 61 detik memunculkan pesan "gagal terhubung" dan tombol coba lagi; tunda 14 dan 59 detik tidak; coba lagi tidak menghasilkan aksi ganda | batas | E2E | - | F1 |
| TC-KU-02-2 | KU-02 | G: permintaan mutasi gagal di tengah jalan; W: dikirim ulang dengan kunci idempotensi sama; T: tidak ada aksi ganda (satu entitas, satu event audit) | negatif | integrasi | T1 | F1 |
| TC-KU-03-1 | KU-03 | G: pengguna A, data milik B; W: A mengganti ID di path/query/body; T: ditolak dengan respons tak dapat dibedakan dari "tidak ada" | negatif | integrasi | T1 | F1 |
| TC-KU-03-2 | KU-03 | G: seperti di atas pada data sensitif (§3.1); W: akses ditolak; T: event audit hasil "gagal" tercatat dengan aktor, objek, subjek data | negatif | integrasi | T1 | F1 |
| TC-KU-04-1 | KU-04 | G: data master di-soft delete (`deleted_at`, `deleted_by`, `delete_reason` terisi); W: buka daftar, cari, pilih untuk transaksi baru; T: tidak tampil dan tidak bisa dipilih; data tidak terhapus fisik | negatif | integrasi | T1 | F1 |
| TC-KU-04-2 | KU-04 | G: data master terhapus; W: dipulihkan peran berhak; T: tampil kembali, E-16 tercatat, riwayat utuh | positif | integrasi | T1 | F1 |
| TC-KU-05-1 | KU-05 | G: dua permintaan identik paralel (tekan kirim dua kali); W: server menerima; T: satu entitas dibuat | negatif | integrasi | T1 | F1 |
| TC-KU-06-1 | KU-06 | G: tiap aksi pada tabel §7.1 yang aktif; W: sukses dan gagal; T: event dengan semua field wajib §7.2, hasil sesuai (matriks event × hasil diperiksa otomatis) | positif | integrasi | T1 | F1 |
| TC-KU-07-1 | KU-07 | G: jadwal konsultasi dan jadwal rekanan WITA; W: dilihat di perangkat WIB; T: konsultasi di zona perangkat, rekanan dengan label zona rekanan, backoffice WIB | batas | unit | - | F1 |
| TC-KU-08-1 | KU-08 | G: sistem rekanan atau penyedia tidak merespons; W: pasien membuka data atau memesan; T: pesan "sementara tidak tersedia", data lama tidak tampil sebagai terbaru (waktu pembaruan terakhir bila ada), tidak ada pembayaran dibuat | negatif | integrasi | T1 | F3 |
| TC-KU-09-1 | KU-09 | G: dua pasien memilih slot/stok terakhir; W: keduanya lanjut ke pembayaran bersamaan; T: hanya satu berhasil, yang lain diberi tahu sebelum membayar | negatif | integrasi | T1 | F2 |
| TC-KU-10-1 | KU-10 | G: akun pasien atau admin (F1), tenaga kesehatan dan rekanan (F2) dinonaktifkan; W: (a) pemiliknya masuk; (b) memakai sesi yang masih terbuka; T: (a) pesan seragam login gagal US-103 AC2, identik kata per kata, tanpa kalimat atau tanda tambahan (dibandingkan dengan password salah); (b) permintaan berikutnya ditolak dengan kode respons akun nonaktif, aplikasi menampilkan layar "Akun nonaktif" dengan kontak Layanan Pelanggan, WebSocket sesi diputus | negatif | integrasi | T1 | F1 |
| TC-KU-10-2 | KU-10 | G: sesi aktif; W: permintaan pertama yang mulai diproses tepat setelah aksi penonaktifan mengembalikan sukses ke pelakunya (dikirim segera setelah respons diterima), diulang 50 kali; T: setiap permintaan itu ditolak, tidak ada yang lolos walaupun token belum kedaluwarsa | batas | integrasi | T1 | F1 |

### 3.2 Skenario wajib endpoint dan isolasi (coding standards §7.3, NFR-SEC-06, NFR-SEC-07)

Dijalankan untuk setiap endpoint baru di setiap fase (test parametrik). Fase 1 mencakup endpoint US-101–US-208, US-1501, US-1504.

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-SEC-1 | NFR-SEC-07 | G: endpoint; W: tanpa token; T: 401 | negatif | integrasi | T1 | F1 |
| TC-SEC-2 | NFR-SEC-01 | G: endpoint; W: token realm lain, audience salah, issuer salah, algoritma di luar daftar izin, kedaluwarsa, token realm `rme` ke backend Ruang Sehat dan sebaliknya (satu test per kombinasi); T: 401 | negatif | integrasi | T1 | F1 |
| TC-SEC-3 | KU-10 | G: akun dinonaktifkan atau sesi dicabut, token masih berlaku; W: panggil endpoint; T: ditolak; akun nonaktif mendapat kode respons akun nonaktif (layar "Akun nonaktif"), sesi dicabut mendapat `4010001` | negatif | integrasi | T1 | F1 |
| TC-SEC-4 | NFR-SEC-07 | G: token sah, peran tidak diizinkan; W: panggil endpoint; T: 403 | negatif | integrasi | T1 | F1 |
| TC-SEC-5 | KU-03 | G: objek milik pengguna/tenant lain; W: ganti ID; T: 404 dan, untuk data sensitif, event gagal tercatat | negatif | integrasi | T1 | F1 |
| TC-SEC-6 | NFR-SEC-06 | G: dua tenant A dan B (portal rekanan, RME); W: pengguna A membaca, mengubah, atau mencari data B, termasuk menyelipkan `partner_id`/`fasyankes_id` B di body/query; T: tidak terbaca, tidak berubah, tidak muncul di daftar | negatif | integrasi | T1 | F2 |
| TC-SEC-7 | NFR-SEC-06 | G: RME dengan konteks tenant A; W: query tanpa konteks, dan dengan konteks A terhadap baris B; T: tanpa konteks tidak ada baris; konteks A tidak mendapat baris B (RLS dipaksa) | negatif | integrasi | T1 | F3 |
| TC-SEC-8 | NFR-SEC-07 | G: endpoint baru tanpa kebijakan otorisasi terdaftar; W: aplikasi dijalankan; T: ditolak atau gagal start (default deny) | negatif | integrasi | T1 | F1 |
| TC-SEC-9 | NFR-OBS-03 | G: permintaan berisi password, token, NIK; W: diproses dan galat; T: tidak ada di application log, security log, audit | negatif | integrasi | T1 | F1 |

### 3.3 Kebutuhan non-fungsional dan audit (PRD §6, §7)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-NFR-SEC-02-1 | NFR-SEC-02 | G: admin Ruang Sehat baru; W: masuk pertama; T: wajib aktivasi MFA TOTP dengan kode pemulihan sebelum akses apa pun; tanpa MFA tidak ada sesi backoffice | negatif | E2E | T1 | F1 |
| TC-NFR-SEC-02-3 | NFR-SEC-02 | G: pasien terverifikasi tanpa TOTP; W: masuk; T: wajib pendaftaran TOTP dan kode pemulihan sebelum beranda (TC-US-101-25–28); tidak ada jalur masuk realm `pasien` tanpa faktor kedua | negatif | integrasi | T1 | F1 |
| TC-NFR-SEC-02-4 | NFR-SEC-02 | G: konfigurasi realm sebagai kode (`pasien`, `admin`; `nakes`, `rme` saat aktif); W: periksa authenticator dan required action yang aktif, lalu coba memilih faktor lain di halaman login; T: hanya TOTP dan kode pemulihan; SMS, OTP email, dan WebAuthn tidak tersedia | negatif | integrasi | T1 | F1 |
| TC-NFR-SEC-02-2 | NFR-SEC-02 | G: tenaga kesehatan dan admin rekanan; W: aktivasi dan masuk; T: MFA TOTP dengan kode pemulihan wajib; atur ulang password tetap meminta faktor kedua (TC-US-103-36) | negatif | integrasi | T1 | F2 |
| TC-NFR-SEC-02-5 | NFR-SEC-02 | G: pengguna RME minimalis; W: aktivasi dan masuk; T: MFA TOTP dengan kode pemulihan wajib; atur ulang password tetap meminta faktor kedua | negatif | integrasi | T1 | F3 |
| TC-NFR-SEC-03-1 | NFR-SEC-03 | G: akun; W: 5 gagal berturut-turut lalu password benar; T: terkunci walau benar; setelah 15 menit dapat masuk (lihat US-103 AC3) | batas | integrasi | T1 | F1 |
| TC-NFR-SEC-03-2 | NFR-SEC-03 butir 4 | G: password dari daftar kata sandi umum (≥ 3.000 entri), dari daftar kata sandi bocor (berkas uji sampai daftar resmi diverifikasi), kata konteks dan variasinya ("ruang sehat", "RuangSehat", "rumahsehat", "arunika"), dan sama dengan email; W: registrasi; T: ditolak dengan pesan aturan yang dilanggar (menunggu: daftar bocor dan lisensinya, Tahap 8) | negatif | integrasi | T1 | F1 |
| TC-NFR-SEC-03-3 | NFR-SEC-03 butir 1 | G: password 7, 8, 128, dan 129 karakter (di luar daftar); W: registrasi; T: 8 dan 128 diterima; 7 dan 129 ditolak | batas | integrasi | T1 | F1 |
| TC-NFR-SEC-03-4 | NFR-SEC-03 butir 2, 3 | G: password berisi spasi, emoji, dan aksara non-Latin tanpa huruf besar atau angka; password dengan spasi di awal dan akhir; W: registrasi lalu masuk dengan versi yang dipangkas dan versi huruf berbeda; T: password diterima tanpa aturan komposisi; masuk hanya berhasil dengan password persis seperti dimasukkan | batas | integrasi | T1 | F1 |
| TC-NFR-SEC-03-5 | NFR-SEC-03 | G: satu set password lemah dan kuat yang sama; W: dipakai di pendaftaran (US-101 AC1a), atur ulang (US-103 AC4b), aktivasi admin (US-208), dan ubah password (US-103 AC6); aktivasi admin mitra (US-206) di F2; T: hasil terima/tolak identik di setiap jalur | negatif | integrasi | T1 | F1 |
| TC-NFR-SEC-03-6 | NFR-SEC-03 butir 4 | G: akses jaringan keluar dari Keycloak dan backend diblokir; W: pemeriksaan password dijalankan; T: pemeriksaan tetap bekerja (ketiga daftar berkas lokal), tidak ada permintaan ke layanan eksternal | negatif | integrasi | T1 | F1 |
| TC-NFR-SEC-03-7 | NFR-SEC-03 butir 5, 6 | G: isian password di aplikasi pasien, backoffice, dan halaman login bertema; W: menempel password, memakai pengelola password, dan memeriksa konfigurasi kebijakan password realm; T: tempel dan pengisian otomatis berfungsi; isian bermasker dengan opsi menampilkan; tidak ada kewajiban ganti password berkala di konfigurasi realm | positif | manual | T1 | F1 |
| TC-NFR-SEC-04-1 | NFR-SEC-04 | G: sesi web backoffice; W: tanpa aktivitas 30 menit; T: sesi berakhir dan permintaan berikutnya ditolak; pada menit 29 masih berlaku | batas | E2E | T1 | F1 |
| TC-NFR-SEC-04-2 | NFR-SEC-04 | G: portal rekanan dan RME; W: idle 30 menit; T: sesi berakhir | batas | E2E | T1 | F2 |
| TC-NFR-SEC-05-1 | NFR-SEC-05 | G: lalu lintas klien ke server; W: koneksi HTTP biasa atau TLS lemah; T: ditolak/dialihkan, tidak ada data terkirim tanpa TLS | negatif | integrasi | T1 | F1 |
| TC-NFR-SEC-05-2 | NFR-SEC-05 | G: kolom data spesifik (NIK, berkas); W: inspeksi langsung di database; T: ciphertext, bukan teks asli; ciphertext diubah satu bit atau dipindah ke baris lain gagal dibuka; DEK yang dimusnahkan tidak dapat dibuka | negatif | integrasi | T1 | F1 |
| TC-NFR-SEC-05-3 | NFR-SEC-05 | G: role aplikasi; W: menghapus/mengubah DEK atau memakai `container_type` salah; T: ditolak | negatif | integrasi | T1 | F1 |
| TC-NFR-SEC-06-1 | NFR-SEC-06 | G: pasien A dan B; W: A mengakses profil, identitas, berkas B lewat ID; T: ditolak (lihat TC-SEC-5) | negatif | integrasi | T1 | F1 |
| TC-NFR-SEC-07-1 | NFR-SEC-07 | G: tujuh sub-peran admin; W: tiap sub-peran memanggil tiap endpoint admin; T: hanya kombinasi yang diizinkan §3.1 berhasil (matriks peran × endpoint, default deny) | negatif | integrasi | T1 | F1 |
| TC-NFR-SEC-08-1 | NFR-SEC-08 | G: unggahan; W: foto JPG/PNG 5.242.880 dan 5.242.881 byte; PDF 10.485.760 dan 10.485.761 byte; T: batas diterima, lewat ditolak `4130001` | batas | integrasi | T1 | F1 |
| TC-NFR-SEC-08-3 | NFR-SEC-08 | G: foto 5.242.881 byte yang setelah dikode ulang menjadi lebih kecil, dan foto 5.242.880 byte yang setelah dikode ulang menjadi lebih besar; W: unggah; T: yang pertama ditolak, yang kedua diterima (batas berlaku untuk berkas seperti diunggah) | batas | integrasi | T1 | F1 |
| TC-NFR-SEC-08-2 | NFR-SEC-08 | G: berkas tersimpan; W: pihak tidak berhak meminta URL unduh, atau URL kedaluwarsa dipakai; T: tidak ada URL tanpa izin; URL berumur pendek tak berlaku setelah kedaluwarsa | negatif | integrasi | T1 | F1 |
| TC-NFR-SEC-10-1 | NFR-SEC-10 | G: aplikasi tenaga kesehatan; W: ditinggalkan 15 menit lalu dibuka; T: minta biometrik/PIN perangkat; pada menit 14 tidak meminta; gagal verifikasi tidak membuka data | batas | manual | T1 | F2 |
| TC-NFR-SEC-11-1 | NFR-SEC-11 | G: satu akun di realm `pasien` dan `admin` (F1; `nakes`, rekanan F2; `rme` F3); W: masuk dari 5 perangkat/klien berbeda berturut-turut; T: kelima sesi tetap berlaku (token tiap sesi diterima), tidak ada sesi lama yang dikeluarkan | positif | integrasi | T1 | F1 |
| TC-NFR-SEC-11-2 | NFR-SEC-11, NFR-SEC-04 | G: dua sesi web paralel satu akun backoffice; W: sesi pertama idle 30 menit sementara sesi kedua aktif; T: sesi pertama berakhir, sesi kedua tetap berlaku (batas idle per sesi tetap berlaku walau sesi paralel tidak dibatasi) | batas | E2E | T1 | F2 |
| TC-NFR-PRIV-01-1 | NFR-PRIV-01 | G: konfigurasi staging dan production; W: tinjau lokasi database, backup, object storage, penyedia; T: seluruhnya di Indonesia (bukti dokumen penyedia) | positif | manual | T1 | F2 |
| TC-NFR-PRIV-03-1 | NFR-PRIV-03 | G: payload ke payment gateway, rekanan, kanal email; W: diperiksa; T: hanya data yang diperlukan, tanpa diagnosis/nama obat/nilai lab/isi chat | negatif | integrasi | T1 | F2 |
| TC-NFR-PRIV-04-1 | NFR-PRIV-04 | G: sesi video berlangsung; W: periksa aplikasi dan konfigurasi LiveKit; T: tidak ada fitur rekam dan tidak ada berkas rekaman | negatif | integrasi | T1 | F2 |
| TC-NFR-PRIV-05-1 | NFR-PRIV-05 | G: dua versi teks persetujuan; W: pasien menyetujui versi 1 lalu 2; T: bukti tersimpan per versi, versi 1 tetap utuh | positif | integrasi | T1 | F1 |
| TC-NFR-RET-01-1 | NFR-RET-01 | G: kebijakan retensi kosong; W: pekerja privasi berjalan; T: tidak ada data diproses (ADR-013 butir 6); angka retensi menunggu PT-24 | negatif | integrasi | T1 | F3 |
| TC-NFR-OBS-02-1 | NFR-OBS-02 | G: aksi sensitif; W: periksa application log, security log, audit; T: tiga aliran terpisah, audit di database terpisah dari tabel aplikasi (§7.6) | positif | integrasi | T1 | F1 |
| TC-NFR-OBS-03-1 | NFR-OBS-03 | G: skenario galat dan login gagal; W: periksa application log dan security log; T: tanpa password, PIN, OTP, token, NIK lengkap, nomor kartu/rekening lengkap | negatif | integrasi | T1 | F1 |
| TC-NFR-PERF-01-1 | NFR-PERF-01 | G: beban NFR-CAP; W: endpoint baca; T: p95 ≤ 500 ms (server, tanpa pihak ketiga) | batas | integrasi | - | F3 |
| TC-NFR-PERF-02-1 | NFR-PERF-02 | G: beban NFR-CAP; W: endpoint tulis; T: p95 ≤ 1 detik | batas | integrasi | - | F3 |
| TC-NFR-PERF-03-1 | NFR-PERF-03 | G: 100 sesi bersamaan; W: pesan chat ke penerima online; T: p95 ≤ 2 detik | batas | integrasi | - | F3 |
| TC-NFR-PERF-04-1 | NFR-PERF-04 | G: portal, backoffice, RME; W: halaman dibuka; T: p95 ≤ 3 detik | batas | E2E | - | F3 |
| TC-NFR-PERF-05-1 | NFR-PERF-05 | G: kejadian notifikasi; W: diproses; T: diserahkan ke penyedia ≤ 1 menit | batas | integrasi | - | F2 |
| TC-NFR-CAP-1 | NFR-CAP-01–04 | G: staging ukuran production; W: 30.000 pasien aktif bulanan, 1.000 pengguna aktif per menit, 100 sesi konsultasi bersamaan; T: NFR-PERF terpenuhi, hasil dicatat (alat: T-30) | batas | integrasi | - | F3 |
| TC-NFR-AVL-1 | NFR-AVL-01, NFR-OBS-01 | G: pemantauan ketersediaan; W: matikan satu layanan inti di staging; T: peringatan terkirim; gangguan rekanan/penyedia tercatat terpisah (NFR-AVL-04) | positif | integrasi | - | F2 |
| TC-NFR-DR-1 | NFR-DR-01–04 | G: backup; W: uji pemulihan; T: RPO ≤ 15 menit, RTO ≤ 4 jam (jam kerja) atau ≤ 8 jam (luar jam kerja; jam kerja belum ditetapkan, T-31), backup di Indonesia | batas | manual | T1 | Februari 2027 (EN-27) |
| TC-NFR-INT-1 | NFR-INT-01 | G: pesan integrasi gagal sementara; W: dicoba ulang; T: diproses idempoten, tanpa duplikasi | negatif | integrasi | T1 | F3 |
| TC-NFR-INT-2 | NFR-INT-04 | G: notifikasi payment gateway; W: tanda tangan salah atau tanpa tanda tangan; T: ditolak, tidak diproses, tercatat di security log | negatif | integrasi | T1 | F2 |
| TC-NFR-COMP-1 | NFR-COMP-01 | G: aplikasi pasien dan tenaga kesehatan; W: alur utama di Android dan iOS (versi OS minimum dari stack); T: berjalan | positif | manual | - | F2 |
| TC-NFR-COMP-3 | NFR-COMP-03 | G: web backoffice, portal, RME; W: alur utama di dua versi stabil terakhir Chrome, Edge, Firefox, Safari; T: berjalan | positif | manual | - | F3 |
| TC-NFR-LOC-1 | NFR-LOC-01, NFR-LOC-02 | G: seluruh layar fase aktif; W: tinjau teks dan mata uang; T: hanya Bahasa Indonesia; Rupiah tanpa desimal | positif | E2E | - | F1 |
| TC-NFR-A11Y-1 | PRD §6.11 (standar di design guideline §7) | G: komponen web dan Flutter Fase 1; W: skrip kontras dan test guideline aksesibilitas; T: lolos ambang design guideline | positif | unit | - | F1 |
| TC-NFR-A11Y-2 | PRD §6.11 | G: alur A01, A05, A07; W: TalkBack dan VoiceOver; T: dapat diselesaikan (UI-05) | positif | manual | - | F3 |
| TC-AUD-1 | PRD §7.2 | G: satu event contoh tiap kode E-01–E-17 yang aktif di fase; W: periksa field; T: ID, waktu (UTC di penyimpanan, WIB di tampilan), aktor, aksi, objek, sumber, hasil, ID korelasi terisi; subjek data dan before/after terisi bila berlaku | positif | integrasi | T1 | F1 |
| TC-AUD-2 | PRD §7.3 | G: aksi membawa password, PIN, OTP, token, rahasia MFA, kode pemulihan, tautan konfirmasi reset MFA, nomor kartu/rekening, NIK, isi chat/catatan/resep/rekam medis/nilai lab, foto; W: event dicatat; T: yang terlarang tidak ada; rekening dan kartu 4 digit terakhir; NIK 4 digit terakhir; konten klinis hanya referensi objek dan versi | negatif | unit | T1 | F1 |
| TC-AUD-3 | PRD §7.5 | G: peran Auditor, Super Admin, lainnya; W: akses Audit Log; T: Auditor baca dan ekspor, Super Admin baca, lainnya ditolak (lihat US-1502, US-1503) | negatif | integrasi | T1 | F3 |
| TC-AUD-4 | PRD §7.4 | G: audit trail; W: periksa proses penghapusan; T: tidak ada job yang menghapus audit sebelum retensi (angka menunggu PT-24) | negatif | integrasi | T1 | F1 |

### 3.4 Fondasi database, migrasi, dan hak role

Sumber: AB-26 (transaksi immutable), KU-04 (soft delete), dan daftar skenario negatif modul T1 migrasi/hak role `[DOK: coding-standards.md]` §7.3. Tidak ada AC PRD tersendiri.

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-DB-1 | AB-26, KU-04 | G: migrasi dijalankan ke PostgreSQL 18 kosong; W: katalog hak role diperiksa; T: role aplikasi tanpa `UPDATE`/`DELETE` pada tabel transaksi, log, dan audit; trigger penolak aktif; PUBLIC dicabut | negatif | integrasi | T1 | F1 |
| TC-DB-2 | AB-26 | G: tabel transaksi; W: `UPDATE` atau `DELETE` dengan role aplikasi; T: ditolak database | negatif | integrasi | T1 | F1 |
| TC-DB-3 | KU-04 | G: tabel master; W: hapus lewat aplikasi; T: hanya `deleted_at`, `deleted_by`, `delete_reason` terisi, bukan `DELETE` fisik | positif | integrasi | T1 | F1 |
| TC-DB-4 | AB-26 | G: bootstrap role dan migrasi; W: dijalankan dua kali; T: idempoten, tanpa password di berkas | negatif | integrasi | T1 | F1 |
| TC-DB-5 | NFR-SEC-06 | G: RME; W: periksa `FORCE ROW LEVEL SECURITY` dan hak Operator RME pada tabel klinis; T: aktif, Operator RME tidak dapat membaca | negatif | integrasi | T1 | F3 |

---

## 4. Fase 1: test case per story (fondasi + MVP "pasien terverifikasi")

Story Fase 1: US-101, US-103 (jalur Layanan Pelanggan AC8 lewat US-207 AC5 di Fase 2), US-104 (kecuali AC4), US-105, US-106, US-201 (kecuali pemberitahuan di AC2 dan AC7), US-208 (kecuali AC8), US-1501 (peringatan email AC3 di Fase 2), US-1504 `[DOK: 07-phase-plan.md]` §4. Semuanya T1. Setiap endpoint story ini juga menjalankan TC-SEC-1–TC-SEC-5, TC-SEC-8, dan TC-SEC-9 (§3.2). Alur end-to-end MVP: TC-MVP-1 di akhir bagian ini.

### US-101 Registrasi dengan email dan password (BR-001, BR-103)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-101-1 | US-101 AC1 | G: email belum terdaftar, password memenuhi NFR-SEC-03, persetujuan dicentang; W: kirim formulir; T: akun berstatus "email belum diverifikasi", email tautan verifikasi tertangkap di penangkap email, E-04 tercatat tanpa password | positif | integrasi | T1 | F1 |
| TC-US-101-2 | US-101 AC1a | G: password melanggar satu aturan NFR-SEC-03 (satu test per aturan: panjang, kata sandi umum, kata sandi bocor, kata konteks, sama dengan email); W: kirim; T: ditolak, pesan menyebut aturan yang dilanggar, isian lain tidak hilang (KU-01), tidak ada akun | negatif | integrasi | T1 | F1 |
| TC-US-101-3 | US-101 AC1 | G: email kosong, tanpa "@", atau melebihi panjang wajar; W: kirim; T: ditolak per field (KU-01), tidak ada akun | negatif | unit | T1 | F1 |
| TC-US-101-4 | US-101 AC1b | G: langkah pembuatan user Keycloak berhasil tetapi akun aplikasi atau bukti persetujuan gagal (satu test per langkah gagal); W: kirim; T: tidak ada akun setengah jadi, user Keycloak permintaan itu dihapus berdasarkan ID-nya, tidak ada `consent_record` yatim, pasien melihat pesan gagal dengan tombol coba lagi; setelah kompensasi, pendaftaran ulang dengan email yang sama berhasil; E-04 hasil gagal dicoba dicatat | negatif | integrasi | T1 | F1 |
| TC-US-101-16 | US-101 AC1b | G: kompensasi berjalan tetapi penghapusan user Keycloak gagal (Admin API dibuat gagal sementara); W: pendaftaran diproses; T: penghapusan diulang dari antrean job sampai berhasil, setiap kegagalan memicu entri security log tanpa password atau token | negatif | integrasi | T1 | F1 |
| TC-US-101-17 | US-101 AC1b | G: akun B sudah ada (user Keycloak, akun aplikasi, bukti persetujuan); pendaftaran akun A gagal di tengah dan dikompensasi, termasuk saat dua pendaftaran berjalan bersamaan; W: kompensasi selesai; T: user Keycloak, akun aplikasi, dan bukti persetujuan B tidak berubah (dibandingkan sebelum dan sesudah) | negatif | integrasi | T1 | F1 |
| TC-US-101-5 | US-101 AC1, KU-05 | G: dua permintaan registrasi identik paralel; W: server menerima; T: satu akun, satu email verifikasi | negatif | integrasi | T1 | F1 |
| TC-US-101-18 | US-101 AC1c | G: penyedia email gagal saat tautan verifikasi dikirim; W: pendaftaran selesai; T: akun tetap "email belum diverifikasi" dan tidak dibatalkan, pengiriman dicoba ulang otomatis dari antrean job sampai tertangkap di penangkap email, pasien tetap dapat meminta kirim ulang (AC2b) | negatif | integrasi | T1 | F1 |
| TC-US-101-7 | US-101 AC2, AC5 | G: tautan verifikasi berlaku; W: dibuka lalu pasien masuk; T: email terverifikasi, masuk pertama diarahkan ke pendaftaran TOTP (AC5), lalu ke beranda (E2E dari registrasi sampai beranda) | positif | E2E | T1 | F1 |
| TC-US-101-8 | US-101 AC2 | G: tautan dikirim pada waktu t (jam uji Keycloak, §2.3); W: dibuka pada t + 23 jam 59 menit dan, untuk tautan lain, pada t + 24 jam 1 menit; T: yang pertama memverifikasi email; yang kedua ditolak, status email tidak berubah, pasien diminta mengirim ulang tautan | batas | integrasi | T1 | F1 |
| TC-US-101-9 | US-101 AC2 | G: tautan sudah dipakai sekali; W: dibuka lagi; T: ditolak, diminta kirim ulang; status tidak berubah | negatif | integrasi | T1 | F1 |
| TC-US-101-10 | US-101 AC2 | G: tautan milik akun A; W: dimodifikasi (token diubah satu karakter atau dipakai untuk akun B); T: ditolak, tanpa membocorkan akun | negatif | integrasi | T1 | F1 |
| TC-US-101-21 | US-101 AC2 | G: email akun sudah terverifikasi, ada tautan verifikasi lain yang belum dipakai dan belum lewat 24 jam; W: tautan itu dibuka; T: tidak ada perubahan pada akun, pasien diarahkan ke halaman masuk | batas | integrasi | T1 | F1 |
| TC-US-101-11 | US-101 AC2b | G: akun belum terverifikasi; W: kirim ulang pada detik 0, 30, 61, lalu berulang sampai 7 kali dalam 60 menit, dengan variasi huruf besar/kecil email; T: email terkirim pada detik 0 dan 61 tetapi tidak pada detik 30; paling banyak 5 email per 60 menit per email ternormalisasi; respons setiap permintaan identik, termasuk yang tidak mengirim email | batas | integrasi | T1 | F1 |
| TC-US-101-19 | US-101 AC2b | G: satu alamat IP; W: 30 lalu 31 permintaan kirim ulang dalam 60 menit (email berbeda-beda); T: permintaan ke-31 dijawab `4290001`; permintaan ke-30 tidak | batas | integrasi | T1 | F1 |
| TC-US-101-20 | US-101 AC2b | G: email tidak terdaftar, email akun yang sudah terverifikasi, dan email akun belum terverifikasi; W: minta kirim ulang; T: kode HTTP dan isi respons ketiganya identik; email hanya terkirim untuk akun yang belum terverifikasi | negatif | integrasi | T1 | F1 |
| TC-US-101-12 | US-101 AC3 | G: email sudah terdaftar; W: daftar dengan email itu; T: akun baru tidak dibuat, layar sama dengan pendaftaran berhasil ("Cek email Anda untuk melanjutkan"), email berisi petunjuk masuk atau atur ulang password tertangkap di alamat itu; kode dan isi respons API identik dengan pendaftaran baru (perbandingan byte isi respons di luar ID korelasi) | negatif | integrasi | T1 | F1 |
| TC-US-101-22 | US-101 AC3 | G: email terdaftar dan email baru; W: masing-masing didaftarkan 50 kali, waktu respons diukur dari server; T: tidak ada respons kedua kasus yang dikirim sebelum 900 ms sejak permintaan diterima (toleransi dan kasus pemrosesan > 900 ms: T-39) | batas | integrasi | T1 | F1 |
| TC-US-101-23 | US-101 AC3 | G: email terdaftar; W: didaftarkan ulang 7 kali dalam 60 menit, dua di antaranya berselang < 60 detik; T: email petunjuk paling banyak 1 per 60 detik dan 5 per 60 menit ke alamat itu; respons tetap sama dengan pendaftaran berhasil | batas | integrasi | T1 | F1 |
| TC-US-101-24 | US-101 AC3 | G: akun terdaftar dengan status, password, dan bukti persetujuan tertentu; W: pendaftaran ulang dengan email itu dan password lain; T: password lama tetap berlaku, status dan data akun tidak berubah, tidak ada `consent_record` baru | negatif | integrasi | T1 | F1 |
| TC-US-101-13 | US-101 AC4 | G: kotak persetujuan tidak dipilih; W: kirim; T: ditolak, pesan persetujuan wajib, tidak ada akun, tidak ada user Keycloak, tidak ada `consent_record` | negatif | E2E | T1 | F1 |
| TC-US-101-14 | US-101 AC4 | G: permintaan API langsung tanpa field persetujuan atau dengan versi teks yang tidak aktif; W: kirim; T: ditolak dengan hasil yang sama seperti lewat UI | negatif | integrasi | T1 | F1 |
| TC-US-101-15 | US-101 AC1, AC1b, KU-06 | G: registrasi sukses dan gagal; W: periksa audit; T: E-04 (sukses) dan hasil gagal (ditolak) dengan field §7.2; tanpa password, token, atau tautan | negatif | integrasi | T1 | F1 |
| TC-US-101-25 | US-101 AC5 | G: email terverifikasi, TOTP belum terdaftar; W: masuk pertama dengan password benar, memindai kode QR dengan pembangkit TOTP uji, mengirim kode benar; T: halaman login bertema menampilkan kode QR dan kunci teks; TOTP tersimpan; 12 kode pemulihan sekali pakai tampil sekali beserta petunjuk menyimpannya; lalu masuk ke beranda | positif | E2E | T1 | F1 |
| TC-US-101-26 | US-101 AC5 | G: langkah pendaftaran TOTP; W: kirim kode TOTP salah; T: tidak ada TOTP tersimpan, tidak masuk ke beranda, pasien dapat mencoba lagi dan kode benar berikutnya berhasil | negatif | integrasi | T1 | F1 |
| TC-US-101-27 | US-101 AC5 | G: langkah pendaftaran TOTP tampil; W: pasien menutup browser atau membatalkan, lalu masuk lagi; T: tidak masuk ke beranda; masuk berikutnya kembali meminta pendaftaran TOTP | negatif | E2E | T1 | F1 |
| TC-US-101-28 | US-101 AC5, NFR-SEC-02 | G: pasien berhenti di langkah pendaftaran TOTP; W: klien mencoba menyelesaikan alur OIDC atau memanggil API backend dengan token apa pun yang diperolehnya; T: tidak ada token yang diterima backend (401), tidak ada data pasien terbaca | negatif | integrasi | T1 | F1 |
| TC-US-101-29 | US-101 AC5 | G: kode pemulihan sudah ditampilkan sekali; W: pasien memuat ulang halaman, menekan kembali, atau masuk lagi; T: kode pemulihan tidak ditampilkan lagi; jumlah kode yang terbit tepat 12 (menunggu: mekanisme Keycloak, KEPUTUSAN-203) | batas | E2E | T1 | F1 |
| TC-US-101-30 | US-101 AC5, KU-06 | G: pendaftaran TOTP sukses dan gagal; W: periksa audit, application log, security log; T: E-04 tercatat; rahasia TOTP, isi kode QR, kunci teks, kode TOTP, dan kode pemulihan tidak ada di mana pun (§7.3) | negatif | integrasi | T1 | F1 |

### US-103 Masuk, keluar, atur ulang dan ubah password, serta MFA (BR-001, BR-108)

Kode TOTP di test dibangkitkan pembangkit TOTP di `test-support` (§2.3). AC6 dan AC7 berlaku juga di realm `nakes` dan `admin` (dan `rme`); test case realm itu dijalankan saat realm dan aplikasinya aktif.

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-103-1 | US-103 AC1 | G: email dan password benar, email terverifikasi, TOTP terdaftar; W: masuk lewat browser sistem (PKCE) dan memasukkan kode TOTP benar; T: masuk ke beranda, E-01 tercatat | positif | E2E | T1 | F1 |
| TC-US-103-20 | US-103 AC1 | G: seperti TC-US-103-1; W: memasukkan satu kode pemulihan yang belum dipakai sebagai pengganti TOTP; T: masuk ke beranda, E-01 tercatat, kode itu tercatat terpakai | positif | integrasi | T1 | F1 |
| TC-US-103-2 | US-103 AC1a | G: email belum terverifikasi, password benar; W: masuk; T: tidak masuk ke beranda, layar meminta verifikasi email dengan tombol kirim ulang tautan (US-101 AC2b), event login tercatat sesuai pemetaan event Keycloak (ADR-011 butir 10) | negatif | E2E | T1 | F1 |
| TC-US-103-21 | US-103 AC1a, AC2 | G: email belum terverifikasi; W: masuk dengan password salah; T: pesan seragam AC2, tanpa petunjuk status verifikasi; bentuk respons identik dengan password salah pada akun terverifikasi (urutan pemeriksaan Keycloak: T-05, diverifikasi Tahap 8) | negatif | integrasi | T1 | F1 |
| TC-US-103-22 | US-103 AC1b | G: password benar; W: kirim kode TOTP salah; T: tidak masuk, layar kode menampilkan pesan umum dan dapat dicoba lagi, E-02 tercatat | negatif | integrasi | T1 | F1 |
| TC-US-103-24 | US-103 AC1b, AC8 | G: password benar; W: kirim kode pemulihan salah, dan kode pemulihan yang sudah pernah dipakai; T: keduanya ditolak dengan pesan umum yang sama, tidak masuk, E-02 tercatat | negatif | integrasi | T1 | F1 |
| TC-US-103-23 | US-103 AC1b, AC3 | G: akun aktif; W: 3 password salah lalu 2 kode TOTP salah (dan variasi dengan kode pemulihan salah) berturut-turut, lalu masuk dengan password dan TOTP benar; T: akun terkunci (kegagalan TOTP dan kode pemulihan ikut dihitung), percobaan benar ditolak dengan pesan seragam AC2 | batas | integrasi | T1 | F1 |
| TC-US-103-3 | US-103 AC2 | G: empat kondisi: password salah, akun tidak ada, akun terkunci sementara, akun nonaktif; W: masuk; T: teks pesan identik kata per kata untuk keempatnya, memuat petunjuk tetap "Bila tidak dapat masuk, gunakan Lupa password atau hubungi Layanan Pelanggan", tidak menyebut bagian yang salah; kode HTTP dan struktur halaman identik; E-02 tercatat tanpa password | negatif | integrasi | T1 | F1 |
| TC-US-103-4 | US-103 AC2 | G: login gagal untuk email tidak terdaftar; W: periksa audit; T: E-02 tercatat dengan hasil gagal dan kode alasan; respons tidak membocorkan bahwa akun tidak ada | negatif | integrasi | T1 | F1 |
| TC-US-103-25 | US-103 AC2, KU-10 | G: empat kondisi TC-US-103-3; W: diulang di realm `pasien`, `admin` (F1), `nakes` (F2), dan `rme` (F3); T: di setiap realm teks keempat kondisi identik, dan teks antar-realm juga identik | negatif | integrasi | T1 | F1–F3 |
| TC-US-103-5 | US-103 AC3 | G: 4 percobaan gagal berturut-turut; W: percobaan ke-5 benar; T: berhasil masuk, hitungan gagal direset | batas | integrasi | T1 | F1 |
| TC-US-103-6 | US-103 AC3 | G: 5 percobaan gagal berturut-turut; W: percobaan berikutnya dengan password benar dalam 15 menit; T: ditolak dengan pesan seragam AC2, E-02 dengan alasan akun terkunci | negatif | integrasi | T1 | F1 |
| TC-US-103-7 | US-103 AC3 | G: akun terkunci; W: masuk dengan password benar pada 14 menit 59 detik dan 15 menit 1 detik sejak percobaan gagal ke-5; T: yang pertama ditolak, yang kedua berhasil | batas | integrasi | T1 | F1 |
| TC-US-103-8 | US-103 AC3 | G: 5 gagal dari alamat IP berbeda untuk satu akun; W: percobaan berikutnya; T: akun terkunci (penguncian per akun) | negatif | integrasi | T1 | F1 |
| TC-US-103-9 | US-103 AC3 | G: akun terkunci; W: pasien meninjau pesan dan meminta atur ulang password; T: pesan identik dengan pesan seragam AC2; atur ulang password tetap dapat dipakai (tautan terkirim); tidak ada email pemberitahuan penguncian yang tertangkap | negatif | E2E | T1 | F1 |
| TC-US-103-26 | US-103 AC3 | G: 4 percobaan gagal; W: kegagalan ke-5 terjadi 11 jam 59 menit sesudah kegagalan ke-4 (akun X), dan 12 jam 1 menit sesudahnya (akun Y); T: akun X terkunci; akun Y tidak terkunci dan hitungannya mulai dari 1 | batas | integrasi | T1 | F1 |
| TC-US-103-27 | US-103 AC3 | G: akun terkunci sejak gagal ke-5 pada waktu t; W: 10 percobaan lagi selama terkunci, lalu password benar pada t + 15 menit 1 detik; T: berhasil masuk (penguncian tidak diperpanjang). Bila Keycloak memperpanjang, test gagal dan temuan dibawa ke user sesuai AC3 | negatif | integrasi | T1 | F1 |
| TC-US-103-28 | US-103 AC3 | G: penguncian pertama berakhir; W: 4 gagal (tidak terkunci), lalu gagal ke-5; T: setelah 4 gagal belum terkunci (hitungan kembali nol); setelah gagal ke-5 terkunci lagi tepat 15 menit (diuji di 14:59 dan 15:01) | batas | integrasi | T1 | F1 |
| TC-US-103-29 | US-103 AC3 | G: 4 gagal, lalu login berhasil; W: 4 gagal lagi; T: akun tidak terkunci (login berhasil memutus "berturut-turut") | batas | integrasi | T1 | F1 |
| TC-US-103-10 | US-103 AC4 | G: email terdaftar; W: minta atur ulang password; T: pesan netral tampil, email tautan sekali pakai tertangkap | positif | integrasi | T1 | F1 |
| TC-US-103-11 | US-103 AC4 | G: email tidak terdaftar; W: minta atur ulang; T: pesan identik dengan kasus terdaftar (isi, kode HTTP, bentuk), tidak ada email terkirim | negatif | integrasi | T1 | F1 |
| TC-US-103-12 | US-103 AC4, AC4a | G: tautan atur ulang diterbitkan; W: dibuka di menit 59 dan menit 61; T: menit 59 berlaku, menit 61 ditolak; tautan hanya bisa dipakai sekali | batas | integrasi | T1 | F1 |
| TC-US-103-13 | US-103 AC4a | G: tautan atur ulang sudah dipakai atau kedaluwarsa; W: dipakai lagi; T: password tidak dapat diubah, pasien diminta meminta tautan baru | negatif | integrasi | T1 | F1 |
| TC-US-103-14 | US-103 AC4b | G: tautan berlaku; W: simpan password baru yang melanggar NFR-SEC-03, lalu password yang memenuhi dengan tautan yang sama; T: yang pertama ditolak dengan pesan aturan yang dilanggar; yang kedua berhasil selama tautan masih berlaku | negatif | integrasi | T1 | F1 |
| TC-US-103-30 | US-103 AC4c | G: satu email terdaftar; W: 6 permintaan atur ulang dalam 60 menit dengan variasi huruf besar/kecil; T: paling banyak 5 email tertangkap; permintaan ke-6 tidak mengirim email dan pesannya sama dengan AC4 | batas | integrasi | T1 | F1 |
| TC-US-103-31 | US-103 AC4c | G: satu alamat IP; W: 30 lalu 31 permintaan atur ulang dalam 60 menit; T: permintaan ke-31 dijawab HTTP 429 oleh proxy atau halaman galat bertema; kode `4290001` tidak muncul di jalur halaman Keycloak | batas | integrasi | T1 | F1 |
| TC-US-103-32 | US-103 AC4d | G: pasien punya sesi aktif di dua perangkat lain; W: password berhasil diatur ulang; T: paling lambat 2 menit kemudian sesi Keycloak kedua perangkat berakhir dan tercatat di `session_revocation`; permintaan dengan token akses atau token penyegar sesi itu ditolak `4010001` | batas | integrasi | T1 | F1 |
| TC-US-103-33 | US-103 AC4d | G: atur ulang berhasil; W: periksa sesudahnya; T: pasien tidak otomatis masuk dan harus masuk lewat alur biasa termasuk TOTP; password lama ditolak; email "password Anda telah diatur ulang" dengan kontak Layanan Pelanggan tertangkap; E-04 tanpa nilai kredensial | positif | integrasi | T1 | F1 |
| TC-US-103-34 | US-103 AC4e | G: akun dengan TOTP terdaftar; W: membuka tautan atur ulang; T: kode TOTP atau kode pemulihan diminta sebelum password baru berlaku dan sebelum akses diberikan; tanpa kode atau dengan kode salah, password tidak berubah dan tidak ada sesi | negatif | E2E | T1 | F1 |
| TC-US-103-35 | US-103 AC4e | G: penyerang hanya menguasai kotak email (tautan atur ulang) tanpa TOTP; W: melewati langkah kode dengan memanggil langsung endpoint aksi Keycloak atau mengulang parameter alur; T: password tidak berubah, tidak ada token terbit, kegagalan kode dihitung untuk penguncian (AC1b). Bila Reset Credentials tidak dapat dikonfigurasi begitu, temuan dibawa ke user sesuai AC4e | negatif | integrasi | T1 | F1 |
| TC-US-103-36 | US-103 AC4e, NFR-SEC-02 | G: akun di realm `admin` (F1), `nakes` (F2), `rme` (F3); W: atur ulang password; T: TOTP atau kode pemulihan tetap diminta seperti TC-US-103-34 | negatif | integrasi | T1 | F1–F3 |
| TC-US-103-15 | US-103 AC5 | G: pasien masuk; W: keluar; T: sesi di server diakhiri, E-03 tercatat | positif | E2E | T1 | F1 |
| TC-US-103-16 | US-103 AC5 | G: pasien sudah keluar, token akses dan token penyegar belum kedaluwarsa; W: permintaan API pertama yang mulai diproses setelah keluar mengembalikan sukses memakai token itu; T: keduanya ditolak `4010001`, tidak ada token baru | negatif | integrasi | T1 | F1 |
| TC-US-103-17 | US-103 AC5, KU-10 | G: akun dinonaktifkan admin saat sesi aktif; W: permintaan berikutnya dan penyegaran token; T: ditolak, aplikasi menampilkan layar "Akun nonaktif" dengan kontak Layanan Pelanggan | negatif | integrasi | T1 | F1 |
| TC-US-103-18 | US-103 AC1, AC5 | G: token dari semua kombinasi tidak sah (realm lain, audience/issuer salah, algoritma di luar daftar, kedaluwarsa); W: panggil endpoint terlindungi; T: 401 (TC-SEC-2) | negatif | integrasi | T1 | F1 |
| TC-US-103-19 | US-103 AC1, AC1b, AC2 | G: login sukses dan gagal, dengan TOTP dan kode pemulihan; W: periksa log dan audit; T: tidak ada password, token, kode TOTP, rahasia TOTP, atau kode pemulihan di application log, security log, audit | negatif | integrasi | T1 | F1 |
| TC-US-103-37 | US-103 AC6 | G: pasien login, autentikasi terakhir < 5 menit; W: ubah password dengan password lama benar dan password baru memenuhi NFR-SEC-03; T: berhasil, sesi yang sedang dipakai tetap berjalan, password lama ditolak saat masuk berikutnya, E-04 tanpa nilai kredensial | positif | E2E | T1 | F1 |
| TC-US-103-38 | US-103 AC6 | G: pasien login; W: ubah password dengan password lama kosong, password lama salah, atau password baru kosong (UI dan API); T: ditolak, password tidak berubah | negatif | integrasi | T1 | F1 |
| TC-US-103-39 | US-103 AC6 | G: autentikasi terakhir 4 menit 59 detik dan 5 menit 1 detik sebelum permintaan; W: ubah password; T: yang pertama tidak meminta autentikasi ulang; yang kedua meminta masuk ulang dulu, dan bila autentikasi ulang gagal atau dibatalkan password tidak berubah; permintaan API tanpa autentikasi ulang ditolak | batas | integrasi | T1 | F1 |
| TC-US-103-40 | US-103 AC6, AC1a (US-101) | G: pasien login; W: password baru melanggar tiap aturan NFR-SEC-03; T: ditolak dengan pesan aturan yang dilanggar seperti US-101 AC1a | negatif | integrasi | T1 | F1 |
| TC-US-103-41 | US-103 AC6 | G: pasien punya sesi di dua perangkat lain; W: membuka layar ubah password lalu menyimpan tanpa mengubah opsi; T: opsi "keluar dari perangkat lain" tampil tercentang; sesudah simpan, token sesi perangkat lain ditolak `4010001` dan sesi yang sedang dipakai tetap berjalan (batas waktu: T-40) | positif | E2E | T1 | F1 |
| TC-US-103-42 | US-103 AC6 | G: akun di realm `admin` (F1), `nakes` (F2), `rme` (F3); W: TC-US-103-37–41 diulang di aplikasi masing-masing; T: perilaku sama (menunggu: mekanisme Keycloak, diverifikasi Tahap 8) | positif | integrasi | T1 | F1–F3 |
| TC-US-103-43 | US-103 AC7 | G: pasien login dengan TOTP lama dan 12 kode pemulihan lama; W: memilih daftar ulang TOTP, autentikasi ulang, mendaftarkan TOTP baru; T: TOTP baru tersimpan, 12 kode pemulihan baru tampil sekali, opsi "keluar dari perangkat lain" tampil tercentang, E-04 tanpa rahasia | positif | E2E | T1 | F1 |
| TC-US-103-44 | US-103 AC7 | G: TOTP baru sudah tersimpan; W: masuk dengan kode dari TOTP lama, dan dengan kode pemulihan lama yang belum pernah dipakai; T: keduanya ditolak, E-02 tercatat | negatif | integrasi | T1 | F1 |
| TC-US-103-45 | US-103 AC7 | G: pasien memulai daftar ulang TOTP; W: autentikasi ulang gagal atau dibatalkan, autentikasi terakhir > 5 menit tanpa autentikasi ulang (lewat API), atau kode TOTP baru salah; T: tidak ada perubahan: TOTP lama dan kode pemulihan lama tetap berlaku | negatif | integrasi | T1 | F1 |
| TC-US-103-46 | US-103 AC7 | G: pasien punya sesi di perangkat lain; W: daftar ulang TOTP dengan opsi bawaan; T: token sesi perangkat lain ditolak `4010001`, sesi yang sedang dipakai tetap berjalan | positif | integrasi | T1 | F1 |
| TC-US-103-47 | US-103 AC7 | G: akun di realm `admin` (F1), `nakes` (F2), `rme` (F3); W: TC-US-103-43–46 diulang; T: perilaku sama (menunggu: mekanisme Keycloak, diverifikasi Tahap 8) | positif | integrasi | T1 | F1–F3 |
| TC-US-103-48 | US-103 AC8 | G: pasien kehilangan perangkat authenticator, punya kode pemulihan; W: masuk dengan password dan satu kode pemulihan yang belum dipakai; T: masuk ke beranda, aplikasi mengarahkan ke daftar ulang TOTP (AC7); kode itu ditolak bila dipakai lagi | positif | E2E | T1 | F1 |
| TC-US-103-49 | US-103 AC8 | G: layar kode TOTP saat masuk; W: pasien meninjau layar; T: petunjuk menghubungi Layanan Pelanggan bila kode pemulihan juga hilang tampil (pemulihan lewat US-207 AC5 diuji di Fase 2) | positif | E2E | - | F1 |
| TC-US-103-50 | US-103 AC8 | G: satu kode pemulihan dikirim dari dua permintaan login bersamaan; W: server memproses; T: paling banyak satu login berhasil, yang lain ditolak; kode tercatat terpakai satu kali | negatif | integrasi | T1 | F1 |
| TC-US-103-51 | US-103 AC8, KU-06 | G: login dengan kode pemulihan sukses dan gagal; W: periksa audit dan log; T: E-01/E-02 tercatat, nilai kode pemulihan tidak ada di audit, application log, security log | negatif | integrasi | T1 | F1 |

### US-104 Persetujuan pemrosesan data (BR-103, BR-104, BR-107)

AC4 dan AC5 versi baru diverifikasi di Fase 2 bersama US-1001 `[DOK: 07-phase-plan.md]`.

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-104-1 | US-104 AC1 | G: pasien mendaftar; W: layar persetujuan tampil; T: memuat tujuh informasi (a)–(g) Pasal 21 ayat (1) UU PDP dan kontak Ruang Sehat; teks draf bertanda "menunggu L-08/PT-24" (isi final: T-10) | positif | E2E | T1 | F1 |
| TC-US-104-2 | US-104 AC1 | G: salah satu dari tujuh bagian kosong atau teks versi aktif masih bertanda draf; W: pemeriksaan sebelum go-live; T: pemeriksaan gagal (semuanya wajib terisi sebelum go-live) | negatif | manual | T1 | Tahap 12 |
| TC-US-104-3 | US-104 AC2 | G: pasien menyetujui; W: melanjutkan; T: bukti tersimpan: versi teks, butir yang disetujui, waktu, kanal; E-13 tercatat | positif | integrasi | T1 | F1 |
| TC-US-104-4 | US-104 AC2 | G: bukti persetujuan tersimpan; W: coba `UPDATE`/`DELETE` bukti dengan role aplikasi; T: ditolak (data transaksi immutable, AB-26) | negatif | integrasi | T1 | F1 |
| TC-US-104-5 | US-104 AC2 | G: permintaan menyetujui versi teks yang tidak aktif atau tidak ada; W: kirim; T: ditolak | negatif | integrasi | T1 | F1 |
| TC-US-104-6 | US-104 AC3 | G: versi teks dan butir baru dimasukkan lewat file migrasi data yang dijalankan pipeline, tanpa build baru aplikasi mobile atau web; W: layar persetujuan dibuka setelah `effective_from`; T: teks dan butir baru tampil dan dapat disetujui, tersimpan di bukti | positif | integrasi | T1 | F1 |
| TC-US-104-12 | US-104 AC3 | G: versi baru dengan `effective_from` = t; W: layar dibuka pada t − 1 detik dan t + 1 detik (`FixedClock`); T: sebelum t versi lama tampil; sesudah t versi baru tampil | batas | integrasi | T1 | F1 |
| TC-US-104-13 | US-104 AC3 | G: versi teks yang sudah terbit; W: `UPDATE` teks, butir, atau `effective_from`-nya (role aplikasi dan role migrasi); T: ditolak trigger, versi tetap utuh | negatif | integrasi | T1 | F1 |
| TC-US-104-7 | US-104 AC3, AC5 | G: butir wajib belum dipilih (satu dari beberapa butir); W: lanjut; T: ditolak, tidak ada akun | negatif | E2E | T1 | F1 |
| TC-US-104-8 | US-104 AC4 | G: versi baru dijadwalkan dengan tanggal berlaku; W: waktu sebelum tanggal berlaku; T: pemberitahuan perubahan dan tanggal berlaku terkirim ke pasien sebelum tanggal itu | positif | integrasi | T1 | F2 |
| TC-US-104-9 | US-104 AC4, AC5 | G: tanggal berlaku lewat, pasien belum menyetujui versi baru; W: memesan layanan; T: diblokir sampai menyetujui; layanan lain yang tidak memesan tetap dapat dibuka (riwayat) | negatif | E2E | T1 | F2 |
| TC-US-104-10 | US-104 AC5 | G: pasien tidak menyetujui saat registrasi; W: mencoba melanjutkan; T: akun tidak dibuat | negatif | E2E | T1 | F1 |
| TC-US-104-11 | US-104 AC2, KU-06 | G: persetujuan diberikan; W: periksa event E-13; T: field §7.2 lengkap, subjek data = pasien, tanpa data terlarang | positif | integrasi | T1 | F1 |

### US-105 Profil data diri (BR-002)

Semua penolakan validasi AC1a dijawab `4220001` dengan pesan per field (KU-01).

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-105-1 | US-105 AC1 | G: pasien membuka profil; W: mengisi nama, tanggal lahir, jenis kelamin, nomor HP, email, alamat; T: data tersimpan dan tampil kembali; E-10 dengan before/after non-klinis termasking | positif | E2E | T1 | F1 |
| TC-US-105-2 | US-105 AC1a | G: nama kosong, hanya spasi, 101 karakter setelah dipangkas, atau berisi angka atau simbol di luar yang diizinkan (misalnya "@", "#", "1"); W: simpan; T: ditolak `4220001` dengan pesan di field nama, isian benar tidak hilang | negatif | unit | - | F1 |
| TC-US-105-15 | US-105 AC1a | G: nama 1 karakter, 100 karakter setelah dipangkas, nama dengan spasi di awal dan akhir, nama beraksara non-Latin, nama dengan titik, koma, apostrof, dan tanda hubung; W: simpan; T: diterima, spasi awal dan akhir dipangkas | batas | unit | - | F1 |
| TC-US-105-16 | US-105 AC1a | G: tanggal lahir: tanggal tidak sah (31 Februari), besok menurut tanggal WIB, hari ini, tepat 120 tahun lalu, 120 tahun lalu kurang satu hari; dan perangkat WITA pukul 00.30 (tanggal WITA sudah berganti, WIB belum) mengirim tanggal WITA hari itu; W: simpan; T: tanggal tidak sah, besok WIB, lebih dari 120 tahun, dan tanggal WITA yang masih "besok" menurut WIB ditolak; hari ini dan tepat 120 tahun diterima (usia minimum: TC-US-105-20) | batas | unit | - | F1 |
| TC-US-105-17 | US-105 AC1a | G: jenis kelamin di luar "Laki-laki" dan "Perempuan" atau kosong; nomor HP "0812-3456 7890", "+6281234567", awalan selain "+62"/"0", 7 dan 14 digit setelah kode negara, 8 dan 13 digit, serta kosong; W: simpan; T: jenis kelamin tidak sah ditolak; nomor HP dibersihkan dari spasi dan tanda hubung lalu disimpan berbentuk +62; 8 dan 13 digit diterima, 7 dan 14 digit serta awalan lain ditolak; nomor HP kosong diterima | batas | unit | - | F1 |
| TC-US-105-18 | US-105 AC1a | G: alamat dengan field wajib kosong (nama penerima, nomor HP penerima, alamat lengkap, provinsi, kota), alamat lengkap 4/5/500/501 karakter, provinsi dan kota 1/2/100/101 karakter, label 30/31 karakter, kode pos berisi huruf, patokan 200/201 karakter; W: simpan; T: field wajib kosong dan nilai di luar batas ditolak per field; nilai tepat di batas diterima; field opsional kosong diterima | batas | unit | - | F1 |
| TC-US-105-19 | US-105 AC1a | G: pasien dengan dua alamat; W: menandai alamat kedua sebagai utama, dan mengirim permintaan yang membuat nol atau dua alamat utama (termasuk dua permintaan bersamaan); T: setelah setiap operasi tepat satu alamat utama (perilaku saat alamat utama dihapus: T-42) | batas | integrasi | - | F1 |
| TC-US-105-20 | US-105 AC1b | G: PB-22 berisi nilai uji N tahun; W: simpan tanggal lahir yang menghasilkan usia tepat N tahun hari ini, dan N tahun kurang satu hari, di pendaftaran (bila field ada) dan di profil; T: tepat N diterima; kurang satu hari ditolak dengan pesan usia minimum (nilai final PB-22: L-08) | batas | integrasi | T1 | F1 |
| TC-US-105-3 | US-105 AC1 | G: pasien A; W: A membaca atau mengubah profil pasien B dengan mengganti ID; T: 404 tanpa bocor keberadaan, tidak ada perubahan | negatif | integrasi | T1 | F1 |
| TC-US-105-4 | US-105 AC1 | G: dua permintaan simpan identik paralel; W: server menerima; T: satu pembaruan, satu event | negatif | integrasi | - | F1 |
| TC-US-105-5 | US-105 AC2 | G: pasien menambah alamat kedua dan ketiga; W: simpan; T: ketiganya di daftar alamat dan dapat dipilih (pemilihan saat home visit/produk diverifikasi di US-501/US-703) | positif | integrasi | - | F1 |
| TC-US-105-6 | US-105 AC2, KU-04 | G: alamat dihapus (soft delete); W: buka daftar alamat dan pilih alamat; T: alamat terhapus tidak tampil dan tidak dapat dipilih | negatif | integrasi | T1 | F1 |
| TC-US-105-7 | US-105 AC3 | G: identitas "Disetujui"; W: ubah nama; T: status kembali "Menunggu verifikasi", E-10 tercatat | positif | integrasi | T1 | F1 |
| TC-US-105-8 | US-105 AC3 | G: identitas "Disetujui"; W: ubah tanggal lahir; T: status kembali "Menunggu verifikasi" | positif | integrasi | T1 | F1 |
| TC-US-105-9 | US-105 AC3 | G: identitas "Disetujui"; W: ubah nomor HP atau alamat saja; T: status tetap "Disetujui" (hanya nama dan tanggal lahir yang menurunkan status) | batas | integrasi | T1 | F1 |
| TC-US-105-10 | US-105 AC3, US-106 AC1 | G: identitas "Disetujui"; W: ubah NIK lewat endpoint profil; T: ditolak; perubahan NIK hanya lewat pengajuan US-106 AC1 (TC-US-106-23) | negatif | integrasi | T1 | F1 |
| TC-US-105-11 | US-105 AC3 | G: status kembali "Menunggu verifikasi" dan ada pesanan terbayar; W: pasien memesan layanan wajib identitas, dan pesanan lama dijalankan; T: pemesanan baru diblokir (US-107), pesanan terbayar tetap berlaku | negatif | integrasi | T1 | F2 |
| TC-US-105-21 | US-105 AC4a | G: autentikasi terakhir 4 menit 59 detik dan 5 menit 1 detik sebelum permintaan; W: ajukan perubahan email lewat aplikasi, dan lewat API langsung tanpa autentikasi ulang; T: yang pertama diproses; yang kedua meminta masuk ulang di Keycloak dulu; permintaan API tanpa autentikasi ulang ditolak dan email tidak berubah | batas | integrasi | T1 | F1 |
| TC-US-105-12 | US-105 AC4, AC4b | G: pasien mengubah email; W: simpan; T: tautan verifikasi tertangkap di email baru; email lama tetap dipakai untuk masuk dan notifikasi sampai tautan dibuka | positif | integrasi | T1 | F1 |
| TC-US-105-22 | US-105 AC4b, AC4c | G: pasien mengajukan perubahan ke email X lalu ke email Y; W: tautan X dibuka, lalu tautan Y dibuka; T: tautan X tidak mengubah email (sudah digantikan), pasien diminta mengajukan ulang; tautan Y berlaku | negatif | integrasi | T1 | F1 |
| TC-US-105-23 | US-105 AC4b, AC4c | G: tautan perubahan email dikirim pada waktu t; W: dibuka pada t + 23 jam 59 menit (tautan A) dan t + 24 jam 1 menit (tautan B, permintaan lain); T: A berlaku; B tidak mengubah email | batas | integrasi | T1 | F1 |
| TC-US-105-14 | US-105 AC4c | G: tautan verifikasi email baru kedaluwarsa, sudah dipakai, atau sudah digantikan; W: dibuka; T: email tidak berubah dan pasien diminta mengajukan ulang | negatif | integrasi | T1 | F1 |
| TC-US-105-13 | US-105 AC4d | G: pasien punya sesi lain di perangkat kedua; W: tautan yang berlaku dibuka; T: email masuk dan notifikasi berganti di Keycloak dan aplikasi; email lama tak lagi dapat masuk; email lama menerima pemberitahuan dengan kontak Layanan Pelanggan; token sesi perangkat kedua ditolak `4010001`; E-04 tanpa nilai kredensial | positif | integrasi | T1 | F1 |
| TC-US-105-24 | US-105 AC4e | G: email Z sudah dipakai akun lain; W: pasien mengajukan perubahan ke Z; T: respons ke pasien identik dengan AC4b (kode dan isi), alamat Z menerima pemberitahuan bahwa alamat itu sudah terdaftar, email akun pasien dan akun pemilik Z tidak berubah | negatif | integrasi | T1 | F1 |
| TC-US-105-25 | US-105 AC4e | G: email Z sudah dipakai akun lain; W: 7 pengajuan ke Z dalam 60 menit, dua di antaranya berselang < 60 detik; T: pemberitahuan ke Z paling banyak 1 per 60 detik dan 5 per 60 menit; respons ke pasien tetap sama | batas | integrasi | T1 | F1 |

### US-106 Unggah NIK dan foto KTP (BR-003, BR-004)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-106-1 | US-106 AC1 | G: status "Belum diunggah"; W: isi NIK sintetis 16 digit dan unggah JPG KTP dari galeri; T: status "Menunggu verifikasi", pasien diberi tahu verifikasi manual oleh admin, E-10 tercatat | positif | E2E | T1 | F1 |
| TC-US-106-2 | US-106 AC1 | G: kamera perangkat nyata; W: ambil foto KTP dengan kamera lalu kirim; T: foto terunggah, status "Menunggu verifikasi" | positif | manual | T1 | F1 |
| TC-US-106-3 | US-106 AC1 | G: status "Ditolak" dengan alasan; W: unggah ulang NIK dan foto baru; T: status "Menunggu verifikasi" | positif | integrasi | T1 | F1 |
| TC-US-106-23 | US-106 AC1 | G: status "Disetujui" dengan NIK lama A; W: pasien mengajukan NIK baru B dan foto baru; T: dialog konfirmasi tampil lebih dulu dan menjelaskan layanan wajib identitas tidak dapat dipesan sampai disetujui lagi; setelah dikonfirmasi status "Menunggu verifikasi" | positif | E2E | T1 | F1 |
| TC-US-106-24 | US-106 AC1, AC4 | G: pengajuan NIK B dari status "Disetujui" (NIK lama A) sedang menunggu; W: akun lain mengajukan NIK A sebagai pemilik akun; T: ditolak dengan pesan umum AC4 (A tetap tercatat milik akun ini sampai B disetujui); setelah B disetujui, NIK A tidak lagi tercatat sebagai NIK pemilik akun ini | negatif | integrasi | T1 | F1 |
| TC-US-106-25 | US-106 AC1 | G: status "Disetujui"; W: pasien membatalkan dialog konfirmasi; T: tidak ada pengajuan, status tetap "Disetujui", NIK tidak berubah | batas | E2E | T1 | F1 |
| TC-US-106-4 | US-106 AC1 | G: status "Menunggu verifikasi"; W: kirim pengajuan baru (UI dan API); T: ditolak, status dan pengajuan yang ada tidak berubah | negatif | integrasi | T1 | F1 |
| TC-US-106-5 | US-106 AC1 | G: NIK terisi tetapi foto tidak dilampirkan, atau sebaliknya; W: kirim; T: ditolak per field, status tidak berubah | negatif | unit | T1 | F1 |
| TC-US-106-6 | US-106 AC2 | G: NIK berisi huruf, simbol, spasi di tengah, atau kosong; W: kirim; T: ditolak `4220001` dengan pesan "NIK harus 16 digit angka", tidak ada pengajuan | negatif | unit | T1 | F1 |
| TC-US-106-7 | US-106 AC2 | G: NIK 15, 16, dan 17 digit angka, serta 16 digit yang strukturnya tidak lazim (misalnya semua nol); W: kirim; T: 15 dan 17 digit ditolak `4220001`; semua 16 digit angka lolos validasi format (tidak ada validasi struktur lain) | batas | unit | T1 | F1 |
| TC-US-106-8 | US-106 AC3 | G: berkas PDF, GIF, HEIC, atau teks; W: unggah; T: ditolak `4150001` dengan pesan jenis dan ukuran yang diizinkan | negatif | integrasi | T1 | F1 |
| TC-US-106-9 | US-106 AC3, NFR-SEC-08 | G: JPG dan PNG berukuran tepat 5.242.880 byte dan 5.242.881 byte; W: unggah; T: 5.242.880 diterima; 5.242.881 ditolak `4130001` dengan pesan ukuran | batas | integrasi | T1 | F1 |
| TC-US-106-10 | US-106 AC3 | G: berkas berekstensi .jpg/.png tetapi isinya bukan JPEG/PNG (executable, PDF, HTML); W: unggah; T: ditolak `4150001` berdasarkan isi, bukan ekstensi | negatif | integrasi | T1 | F1 |
| TC-US-106-11 | US-106 AC3 | G: JPG dengan metadata EXIF (lokasi GPS, perangkat); W: unggah dan simpan; T: berkas tersimpan tanpa metadata (ADR-018) | positif | integrasi | T1 | F1 |
| TC-US-106-12 | US-106 AC4 | G: NIK X sudah "Disetujui" sebagai pemilik akun di akun lain; W: pasien baru mengirim NIK X dan foto; T: pengajuan tidak disimpan, status tidak berubah, pesan tepat "NIK ini tidak dapat dipakai untuk akun Anda. Hubungi Layanan Pelanggan bila ini NIK Anda." | negatif | integrasi | T1 | F1 |
| TC-US-106-13 | US-106 AC4 | G: NIK X pernah diajukan akun lain tetapi masih "Menunggu verifikasi" atau "Ditolak"; W: pasien baru mengirim NIK X; T: pengiriman diterima (hanya yang "Disetujui" memblokir) | batas | integrasi | T1 | F1 |
| TC-US-106-14 | US-106 AC4 | G: dua akun mengirim NIK X bersamaan; W: Verifikator menyetujui keduanya (lihat TC-US-201-14); T: hanya satu "Disetujui" | negatif | integrasi | T1 | F1 |
| TC-US-106-26 | US-106 AC4 | G: NIK X sudah "Disetujui" di akun lain, dan NIK Y belum pernah dipakai; W: pasien mengetik X atau Y tanpa melampirkan foto, lalu mengirim (UI dan API); T: tidak ada panggilan pemeriksaan registri saat mengetik; pengiriman tanpa foto ditolak validasi dengan respons identik untuk X dan Y | negatif | integrasi | T1 | F1 |
| TC-US-106-27 | US-106 AC4 | G: satu akun; W: 3 pengajuan lengkap ditolak karena NIK terpakai dalam 24 jam, lalu pengajuan ke-4 dengan NIK lain yang sah, lalu pengajuan sesudah 24 jam; T: pengajuan ke-4 ditahan, penahanan tercatat sebagai kejadian keamanan; sesudah 24 jam pengajuan diproses lagi (pesan saat ditahan dan jendela 24 jam: T-37) | batas | integrasi | T1 | F1 |
| TC-US-106-28 | US-106 AC4 | G: NIK X sudah "Disetujui" di akun lain; W: akun A, B, C masing-masing ditolak sekali untuk NIK X dalam 24 jam, lalu akun D (belum pernah ditolak) mengirim NIK X; T: D ditolak dengan pesan umum yang sama selama 24 jam; penahanan tercatat sebagai kejadian keamanan | batas | integrasi | T1 | F1 |
| TC-US-106-29 | US-106 AC4 | G: satu alamat IP, beberapa akun; W: 10 lalu 11 penolakan dalam 24 jam; T: batas 10 per IP berlaku dan tercatat sebagai kejadian keamanan (menunggu: konsekuensi setelah batas IP, T-37) | batas | integrasi | T1 | F1 |
| TC-US-106-30 | US-106 AC4 | G: penolakan dari banyak akun dan IP; W: 20 lalu 21 penolakan dalam satu jam di seluruh sistem; T: pada 20 tidak ada peringatan; pada 21 peringatan tercatat di security log | batas | integrasi | T1 | F1 |
| TC-US-106-31 | US-106 AC4, KU-06 | G: penolakan dan penahanan TC-US-106-27–30; W: periksa security log, application log, audit; T: setiap penahanan tercatat; NIK lengkap tidak ada di log mana pun, audit hanya 4 digit terakhir (§7.3) | negatif | integrasi | T1 | F1 |
| TC-US-106-32 | US-106 AC4 | G: NIK anggota keluarga X tercatat di akun A; W: akun B menambah anggota keluarga dengan NIK X (US-108); T: diterima; batas AC4 hanya berlaku untuk NIK pemilik akun | batas | integrasi | T1 | F4 |
| TC-US-106-15 | US-106 AC5 | G: status "Menunggu verifikasi"; W: buka profil dan coba ganti foto (UI dan API); T: tidak dapat diganti | negatif | E2E | T1 | F1 |
| TC-US-106-16 | US-106 AC5 | G: pengajuan dokumen menunggu, dibuat dari status "Belum diunggah", dari "Ditolak" (alasan R), dan dari "Disetujui" (NIK lama A) (tiga test); W: pasien membatalkan; T: status kembali berturut-turut ke "Belum diunggah", "Ditolak" dengan alasan R, dan "Disetujui" dengan NIK A; foto dapat diganti | positif | integrasi | T1 | F1 |
| TC-US-106-33 | US-106 AC5, US-105 AC3 | G: status "Menunggu verifikasi" karena pasien mengubah nama atau tanggal lahir; W: pasien membatalkan (UI dan API); T: ditolak, status tetap "Menunggu verifikasi" | negatif | integrasi | T1 | F1 |
| TC-US-106-34 | US-106 AC5, US-201 AC5 | G: pengajuan menunggu; W: pembatalan pasien dan keputusan Verifikator dikirim bersamaan (diulang 50 kali); T: hanya transisi yang tersimpan lebih dulu berlaku, pihak lain diberi tahu, tidak ada keadaan campuran, audit konsisten dengan status akhir | negatif | integrasi | T1 | F1 |
| TC-US-106-17 | US-106 AC1, KU-02 | G: koneksi putus saat mengunggah foto, atau server tidak merespons dalam 60 detik; W: coba lagi; T: pesan "gagal terhubung" dan coba lagi; tidak ada pengajuan ganda atau berkas yatim; slot unggah kedaluwarsa ditolak | negatif | integrasi | T1 | F1 |
| TC-US-106-18 | US-106 AC1, KU-05 | G: tombol kirim ditekan dua kali; W: server menerima; T: satu pengajuan | negatif | integrasi | T1 | F1 |
| TC-US-106-19 | US-106 AC1, KU-03 | G: pasien B; W: B meminta URL foto KTP pasien A dengan mengganti ID, atau memakai URL A yang kedaluwarsa; T: 404/ditolak, tidak ada URL terbit, E-05 hasil gagal tercatat | negatif | integrasi | T1 | F1 |
| TC-US-106-20 | US-106 AC1 | G: NIK tersimpan; W: inspeksi database, audit, log; T: NIK terenkripsi di database (`nik_enc`), 4 digit terakhir saja di tampilan dan audit; NIK lengkap tidak di log | negatif | integrasi | T1 | F1 |
| TC-US-106-21 | US-106 AC1 | G: ciphertext NIK pasien A dipindah ke baris pasien B; W: aplikasi membukanya; T: gagal dibuka (AAD tidak cocok) | negatif | integrasi | T1 | F1 |
| TC-US-106-22 | US-106 AC1 | G: pengajuan NIK sama dari akun yang sama dua kali; W: kirim; T: blind index mendeteksi keunikan di dalam satu akun, tidak ada duplikat | negatif | integrasi | T1 | F1 |

### US-201 Verifikasi identitas KTP/KK (BR-004, BR-008)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-201-1 | US-201 AC1 | G: tiga pengajuan "Menunggu verifikasi" dengan waktu berbeda; W: Verifikator membuka antrean; T: urut dari yang terlama, memuat data diri, NIK lengkap, foto dokumen | positif | E2E | T1 | F1 |
| TC-US-201-2 | US-201 AC1 | G: tidak ada pengajuan menunggu; W: buka antrean; T: keadaan kosong tampil tanpa galat | batas | E2E | - | F1 |
| TC-US-201-3 | US-201 AC1, KU-06 | G: Verifikator membuka foto dokumen; W: permintaan URL unduh; T: E-05 tercatat lebih dulu (aktor, subjek data, objek) lalu URL berumur pendek terbit; waktu di UI berlabel WIB | positif | integrasi | T1 | F1 |
| TC-US-201-4 | US-201 AC1, US-1501 AC3 | G: penulisan E-05 gagal (outbox tidak dapat ditulis); W: Verifikator membuka foto; T: akses ditolak, tidak ada URL terbit, entri security log kegagalan audit (TC-US-1501-17) | negatif | integrasi | T1 | F1 |
| TC-US-201-5 | US-201 AC2 | G: pengajuan menunggu; W: Verifikator menyetujui; T: status "Disetujui", E-07 tercatat | positif | E2E | T1 | F1 |
| TC-US-201-6 | US-201 AC2 | G: persetujuan disimpan; W: periksa pemberitahuan; T: pasien menerima email dan notifikasi di dalam aplikasi (US-1001) | positif | integrasi | T1 | F2 |
| TC-US-201-7 | US-201 AC3 | G: pengajuan menunggu; W: Verifikator memilih alasan dari daftar, menambah keterangan, menolak; T: status "Ditolak", alasan tampil ke pasien, pasien dapat unggah ulang, E-07 | positif | E2E | T1 | F1 |
| TC-US-201-8 | US-201 AC3 | G: Verifikator menolak tanpa memilih alasan (UI dan API); W: simpan; T: tidak dapat disimpan, status tidak berubah | negatif | integrasi | T1 | F1 |
| TC-US-201-9 | US-201 AC3 | G: kode alasan di luar tujuh kode awal (misalnya `fraud`, string kosong, huruf besar `PHOTO_UNREADABLE`); W: kirim; T: ditolak, status tidak berubah | negatif | integrasi | T1 | F1 |
| TC-US-201-20 | US-201 AC3 | G: daftar alasan hasil migrasi data; W: Verifikator menolak dengan setiap kode `photo_unreadable`, `document_invalid`, `nik_mismatch`, `profile_mismatch`, `guardian_not_listed`, `nik_unavailable`, `other` (satu test per kode); T: tepat tujuh kode tersedia, setiap penolakan tersimpan dan labelnya tampil ke pasien (`guardian_not_listed` untuk pengajuan anak diuji ulang di F4) | positif | integrasi | T1 | F1 |
| TC-US-201-21 | US-201 AC3 | G: semua sub-peran admin termasuk Super Admin; W: mencari layar atau endpoint untuk menambah, mengubah, atau menghapus daftar alasan; T: tidak ada (OpenAPI dan UI), permintaan langsung ditolak 404/405 | negatif | integrasi | T1 | F1 |
| TC-US-201-22 | US-201 AC3a | G: Verifikator memilih `other`; W: simpan dengan keterangan kosong, keterangan 500 karakter, dan 501 karakter; T: kosong tidak dapat disimpan; 500 karakter tersimpan dan tampil utuh ke pasien; 501 karakter ditolak | batas | integrasi | T1 | F1 |
| TC-US-201-10 | US-201 AC4 | G: pengajuan anak dengan KK, pemilik akun tidak tercantum; W: Verifikator menolak dengan alasan `guardian_not_listed`; T: status "Ditolak" dengan alasan tersebut | negatif | E2E | T1 | F4 |
| TC-US-201-11 | US-201 AC4 | G: pengajuan anak, pemilik akun tercantum; W: Verifikator menyetujui; T: "Disetujui" (setelah US-108) | positif | E2E | T1 | F4 |
| TC-US-201-12 | US-201 AC5 | G: dua Verifikator membuka pengajuan yang sama; W: keduanya memutuskan hampir bersamaan (satu setuju, satu tolak); T: hanya keputusan pertama tersimpan, yang kedua diberi tahu sudah diputuskan, status tidak berubah, kejadian ditolak tercatat hasil gagal | negatif | integrasi | T1 | F1 |
| TC-US-201-13 | US-201 AC5 | G: keputusan sudah ada; W: Verifikator lain memutuskan lagi setelahnya; T: ditolak, satu `E-07` sukses dan satu gagal | negatif | integrasi | T1 | F1 |
| TC-US-201-14 | US-201 AC7 | G: dua akun mengajukan NIK X yang sama, keduanya menunggu; W: Verifikator menyetujui keduanya bersamaan; T: hanya satu "Disetujui"; persetujuan kedua mendapat pesan konflik dan diselesaikan seperti TC-US-201-17 | negatif | integrasi | T1 | F1 |
| TC-US-201-17 | US-201 AC7 | G: NIK X sudah "Disetujui" sebagai pemilik akun di akun lain; W: Verifikator menyetujui pengajuan pemilik akun dengan NIK X, lalu menyimpan formulir penolakan yang terbuka; T: persetujuan ditolak dengan pesan konflik; formulir penolakan terbuka dengan `nik_unavailable` terpilih; setelah disimpan status "Ditolak" dengan teks umum US-106 AC4; E-07 dengan aktor Verifikator | negatif | E2E | T1 | F1 |
| TC-US-201-23 | US-201 AC7 | G: penolakan `nik_unavailable` tersimpan; W: periksa pemberitahuan; T: pasien menerima pemberitahuan berisi teks umum US-106 AC4 (US-1001), tanpa menyebut akun lain | negatif | integrasi | T1 | F2 |
| TC-US-201-15 | US-201 AC6 | G: peran Layanan Pelanggan, Keuangan, Konten, Auditor, Petugas PDP, Super Admin, pasien lain, rekanan; W: tiap peran meminta foto dokumen; T: ditolak, tidak ada URL, E-05 hasil gagal tercatat (satu test per peran) | negatif | integrasi | T1 | F1 |
| TC-US-201-16 | US-201 AC6 | G: peran bukan Verifikator (terutama Petugas PDP); W: membuka data pengajuan atau profil; T: NIK hanya 4 digit terakhir (§3.1); foto tidak tersedia | negatif | integrasi | T1 | F1 |
| TC-US-201-24 | US-201 AC8 | G: Verifikator V juga pemilik akun pasien P yang punya pengajuan menunggu; W: V membuka pengajuan P di backoffice; T: tombol setuju dan tolak tidak tersedia (menunggu: penentu "pemilik akun yang sama", T-38) | negatif | E2E | T1 | F1 |
| TC-US-201-25 | US-201 AC8 | G: seperti TC-US-201-24; W: V memanggil API setuju dan API tolak untuk pengajuan P secara langsung; T: keduanya ditolak dengan pesan pemisahan tugas, status tidak berubah, E-07 hasil gagal tercatat dengan aktor V (menunggu: T-38) | negatif | integrasi | T1 | F1 |
| TC-US-201-26 | US-201 AC8 | G: V pemilik akun pasien yang punya anggota keluarga dengan pengajuan menunggu (US-108); W: V memutuskan pengajuan anggota keluarga itu (UI dan API); T: ditolak seperti TC-US-201-25 (menunggu: T-38) | negatif | integrasi | T1 | F4 |
| TC-US-201-27 | US-201 AC8 | G: pengajuan P yang tidak dapat diputuskan V; W: Verifikator lain W memutuskan; T: keputusan W tersimpan, E-07 dengan aktor W | positif | integrasi | T1 | F1 |
| TC-US-201-18 | US-201 AC1, KU-10 | G: Verifikator dinonaktifkan atau sesi dicabut; W: membuka antrean dengan token lama; T: ditolak | negatif | integrasi | T1 | F1 |
| TC-US-201-19 | US-201 AC2, AC3 | G: Verifikator mengambil keputusan; W: periksa audit; T: E-07 memuat aktor Verifikator, objek pengajuan, subjek data pasien, hasil, tanpa NIK lengkap dan tanpa isi foto | negatif | integrasi | T1 | F1 |

### US-208 Akun dan sub-peran admin Ruang Sehat (BR-060, BR-080)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-208-1 | US-208 AC1 | G: Super Admin; W: buat akun admin dengan sub-peran Verifikator; T: akun terbuat, undangan aktivasi terkirim, akses sesuai §3.1, E-10 dan E-11 tercatat | positif | E2E | T1 | F1 |
| TC-US-208-2 | US-208 AC1 | G: Super Admin; W: buat akun dengan dua sub-peran (Verifikator dan Keuangan); T: hak = gabungan keduanya, tidak lebih | positif | integrasi | T1 | F1 |
| TC-US-208-3 | US-208 AC1 | G: Super Admin; W: buat akun tanpa memilih sub-peran, atau dengan sub-peran di luar tujuh yang ada; T: ditolak (KU-01) | negatif | integrasi | T1 | F1 |
| TC-US-208-4 | US-208 AC1 | G: admin baru; W: masuk pertama kali; T: wajib aktivasi MFA TOTP dan tidak dapat melewatinya (lihat TC-NFR-SEC-02-1); password aktivasi mengikuti NFR-SEC-03 (TC-NFR-SEC-03-5) | negatif | E2E | T1 | F1 |
| TC-US-208-5 | US-208 AC1 | G: sub-peran selain Super Admin (tiap satu); W: membuat, mengubah, atau menonaktifkan akun admin; T: 403 | negatif | integrasi | T1 | F1 |
| TC-US-208-6 | US-208 AC1 | G: akun tiap sub-peran; W: akses data per §3.1 (NIK lengkap hanya Verifikator, audit log hanya Auditor/Super Admin, dst); T: sesuai matriks, selebihnya ditolak (TC-NFR-SEC-07-1) | negatif | integrasi | T1 | F1 |
| TC-US-208-7 | US-208 AC2 | G: dua Super Admin aktif; W: cabut peran Super Admin salah satu; T: berhasil, E-11 tercatat | batas | integrasi | T1 | F1 |
| TC-US-208-8 | US-208 AC2 | G: hanya satu Super Admin aktif; W: cabut perannya, atau nonaktifkan akunnya; T: ditolak, tetap satu Super Admin | negatif | integrasi | T1 | F1 |
| TC-US-208-9 | US-208 AC2 | G: dua Super Admin; W: keduanya saling mencabut peran bersamaan; T: tidak pernah tersisa nol Super Admin (satu aksi ditolak) | negatif | integrasi | T1 | F1 |
| TC-US-208-10 | US-208 AC3 | G: admin (termasuk Super Admin); W: mengubah peran akunnya sendiri; T: ditolak, E-11 hasil gagal | negatif | integrasi | T1 | F1 |
| TC-US-208-11 | US-208 AC3 | G: admin; W: memanggil API ubah peran dengan ID akunnya sendiri lewat jalur lain (ganti ID/peran di body); T: ditolak | negatif | integrasi | T1 | F1 |
| TC-US-208-12 | US-208 AC1, KU-10 | G: admin dinonaktifkan; W: memakai sesi atau token yang masih berlaku; T: permintaan berikutnya ditolak, layar "Akun nonaktif" tampil | negatif | integrasi | T1 | F1 |
| TC-US-208-13 | US-208 AC1 | G: pembuatan, perubahan, dan penonaktifan akun admin; W: periksa audit; T: E-10 dan E-11 dengan before/after peran, aktor, subjek; tanpa password atau kode MFA | negatif | integrasi | T1 | F1 |
| TC-US-208-14 | US-208 AC1 | G: admin tanpa MFA selesai (aktivasi belum); W: memanggil API backoffice dengan token yang ada; T: ditolak (NFR-SEC-02) | negatif | integrasi | T1 | F1 |
| TC-US-208-15 | US-208 AC4 | G: belum ada Super Admin aktif maupun menunggu aktivasi; kredensial perintah terbit lewat Control Group OpenBao setelah orang kedua menyetujui; W: operator menjalankan perintah bootstrap dengan email yang disetujui; T: tepat satu akun admin sub-peran Super Admin berstatus menunggu aktivasi; undangan aktivasi (password dan MFA) tertangkap; E-10 dan E-11 dengan aktor "sistem (bootstrap)" beserta identitas operator dan penyetuju | positif | integrasi | T1 | F1 |
| TC-US-208-16 | US-208 AC4 | G: (a) ada Super Admin aktif; (b) ada Super Admin menunggu aktivasi (dua test); W: perintah bootstrap dijalankan; T: ditolak, jumlah akun, peran, dan undangan tidak berubah | negatif | integrasi | T1 | F1 |
| TC-US-208-17 | US-208 AC4 | G: belum ada Super Admin; W: dua perintah bootstrap dijalankan bersamaan (diulang 20 kali); T: setiap kali hanya satu akun yang dibuat | negatif | integrasi | T1 | F1 |
| TC-US-208-18 | US-208 AC4 | G: permintaan kredensial perintah bootstrap belum disetujui orang kedua, atau disetujui oleh pemohon sendiri; W: operator mencoba menjalankan perintah; T: kredensial tidak terbit, perintah tidak dapat berjalan, tidak ada akun | negatif | integrasi | T1 | F1 |
| TC-US-208-19 | US-208 AC4 | G: undangan bootstrap terkirim pada waktu t; W: dibuka pada t + 23 jam 59 menit, dipakai ulang setelah berhasil, dan (akun lain) dibuka pada t + 24 jam 1 menit, lalu perintah dijalankan ulang dengan persetujuan orang kedua; T: yang pertama berlaku; pemakaian ulang ditolak; yang kedaluwarsa ditolak dan akun tetap menunggu aktivasi; perintah ulang menerbitkan undangan baru tanpa akun kedua | batas | integrasi | T1 | F1 |
| TC-US-208-20 | US-208 AC4 | G: seluruh file migrasi `db/migrations/`; W: dipindai (pola email, nama, NIK, nomor HP); T: tidak ada data pribadi | negatif | integrasi | T1 | F1 |
| TC-US-208-21 | US-208 AC5 | G: admin mana pun, termasuk Super Admin saat ada Super Admin lain; W: menonaktifkan akunnya sendiri lewat UI dan lewat API dengan ID sendiri; T: ditolak, akun tetap aktif | negatif | integrasi | T1 | F1 |
| TC-US-208-22 | US-208 AC6 | G: akun memegang Auditor; W: Super Admin menambahkan sub-peran lain (tiap satu); T: ditolak dengan pesan pemisahan tugas, peran tidak berubah | negatif | integrasi | T1 | F1 |
| TC-US-208-23 | US-208 AC6, AC1 | G: akun memegang sub-peran selain Auditor, dan pembuatan akun baru dengan Auditor plus sub-peran lain sekaligus; W: Auditor ditambahkan atau akun dibuat; T: ditolak dengan pesan pemisahan tugas | negatif | integrasi | T1 | F1 |
| TC-US-208-24 | US-208 AC7 | G: hanya satu Auditor aktif; W: cabut sub-peran Auditor-nya, atau nonaktifkan akunnya; T: ditolak, tetap satu Auditor | negatif | integrasi | T1 | F1 |
| TC-US-208-25 | US-208 AC7 | G: dua Auditor aktif; W: pencabutan salah satu (berhasil), dan dua permintaan pencabutan bersamaan untuk keduanya; T: tidak pernah tersisa nol Auditor | batas | integrasi | T1 | F1 |
| TC-US-208-26 | US-208 AC8 | G: dua Super Admin dan dua Auditor aktif, satu Super Admin nonaktif; W: sub-peran Super Admin, Auditor, Verifikator, Keuangan, atau Petugas PDP diberikan ke sebuah akun (satu test per sub-peran); T: setiap Super Admin dan Auditor aktif menerima pemberitahuan lewat kanal US-1003 berisi pemberi, penerima, sub-peran, dan waktu; akun nonaktif tidak menerima | positif | integrasi | T1 | F2 |
| TC-US-208-27 | US-208 AC8 | G: pemberitahuan sub-peran sensitif terkirim; W: periksa kotak notifikasi akun admin yang bukan Super Admin atau Auditor (Verifikator, Layanan Pelanggan, Keuangan, Konten, Petugas PDP) dan isi pemberitahuan; T: akun itu tidak menerima; isi tanpa password, kode MFA, atau tautan undangan | negatif | integrasi | T1 | F2 |

### US-1501 Pencatatan event audit (BR-080, BR-081, BR-107)

Event Fase 1: E-01–E-05, E-07, E-10, E-11, E-13 `[DOK: 07-phase-plan.md]` §7.1.

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-1501-1 | US-1501 AC1 | G: aksi untuk tiap event E-01–E-05, E-07, E-10, E-11, E-13 dari tiap komponen yang ada (aplikasi pasien, backoffice, proses otomatis); W: aksi selesai; T: satu event dengan semua field wajib §7.2 terisi, field bersyarat terisi bila berlaku | positif | integrasi | T1 | F1 |
| TC-US-1501-2 | US-1501 AC1 | G: aksi gagal (login gagal, akses ditolak, validasi ditolak); W: aksi gagal; T: event dengan hasil "gagal" dan kode alasan | positif | integrasi | T1 | F1 |
| TC-US-1501-3 | US-1501 AC1 | G: proses otomatis; W: aksi sistem; T: aktor "sistem", sumber hanya nama komponen | positif | integrasi | T1 | F1 |
| TC-US-1501-4 | US-1501 AC1 | G: pustaka audit; W: menyusun event tanpa field wajib (aktor, aksi, objek, hasil, ID korelasi); T: ditolak saat dibangun, tidak ada event setengah jadi | negatif | unit | T1 | F1 |
| TC-US-1501-5 | US-1501 AC1 | G: satu permintaan yang menghasilkan beberapa event; W: periksa ID korelasi; T: ID korelasi sama di semua event permintaan itu | positif | integrasi | T1 | F1 |
| TC-US-1501-6 | US-1501 AC2 | G: payload berisi password, PIN, OTP, token, rahasia MFA, kode pemulihan, tautan konfirmasi reset MFA, nomor kartu/rekening, NIK lengkap, isi chat/catatan, foto; W: dicatat; T: yang terlarang tidak masuk; NIK dan rekening 4 digit terakhir (TC-AUD-2) | negatif | unit | T1 | F1 |
| TC-US-1501-7 | US-1501 AC2 | G: fuzz kunci dan nilai tersembunyi (kunci bertingkat, huruf besar/kecil, alias `pwd`/`pass`); W: dicatat; T: tidak ada yang lolos | negatif | unit | T1 | F1 |
| TC-US-1501-8 | US-1501 AC3 | G: `audit_outbox` tidak dapat ditulis; W: aksi pengguna (misalnya ubah profil); T: aksi dibatalkan (transaksi rollback), pesan gagal tampil, tidak ada perubahan data | negatif | integrasi | T1 | F1 |
| TC-US-1501-9 | US-1501 AC3 | G: aksi sistem/integrasi dan event tidak dapat dicatat; W: proses berjalan; T: tidak dianggap selesai, diulang dari antrean sampai tercatat | negatif | integrasi | T1 | F1 |
| TC-US-1501-17 | US-1501 AC3 | G: pencatatan event gagal sebanyak 3 kali (aksi pengguna, aksi sistem, login gagal); W: periksa security log dan metrik; T: tepat 3 entri security log berkategori kegagalan audit (satu per kegagalan), tanpa data terlarang §7.3; metrik penghitung naik 3 | negatif | integrasi | T1 | F1 |
| TC-US-1501-18 | US-1501 AC3 | G: staging dengan Alertmanager (EN-16); W: pencatatan event gagal pertama pada waktu t; T: email peringatan ke tim teknis tertangkap paling lambat t + 5 menit (menunggu: daftar alamat penerima, diisi user sebelum staging) | batas | integrasi | T1 | F2 |
| TC-US-1501-10 | US-1501 AC3 | G: event aksi gagal (login gagal) tidak dapat dicatat; W: login gagal; T: hasil aksi tidak berubah (tetap ditolak), pencatatan tetap dicoba | negatif | integrasi | T1 | F1 |
| TC-US-1501-11 | US-1501 AC3 | G: pengambil event Keycloak mengalami gangguan lalu pulih; W: event E-01–E-04 menunggu; T: semua terkirim urut tanpa kehilangan dan tanpa duplikat | negatif | integrasi | T1 | F1 |
| TC-US-1501-12 | US-1501 AC1 | G: relay mengirim event dari outbox; W: periksa urutan; T: `seq` berurutan tanpa celah, `prev_hash`/`event_hash` sambung, event ke-n merujuk n-1 (rantai hash) | positif | integrasi | T1 | F1 |
| TC-US-1501-13 | US-1501 AC1 | G: dua relay dijalankan bersamaan; W: outbox berisi event; T: hanya satu relay aktif, tidak ada `seq` ganda atau celah | negatif | integrasi | T1 | F1 |
| TC-US-1501-14 | US-1501 AC1 | G: relay gagal setelah menulis event sebelum menghapus outbox; W: relay jalan ulang; T: event tidak ganda (idempoten) | negatif | integrasi | T1 | F1 |
| TC-US-1501-15 | US-1501 AC1 | G: role aplikasi; W: menulis langsung ke tabel `audit_event`; T: ditolak, hanya lewat outbox dan relay (ADR-005) | negatif | integrasi | T1 | F1 |
| TC-US-1501-16 | US-1501 AC1 | G: aksi dilakukan di zona waktu WITA; W: periksa waktu event; T: waktu server dalam satu acuan baku dan ditampilkan WIB | batas | unit | T1 | F1 |

### US-1504 Perlindungan audit log (BR-080)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-1504-1 | US-1504 AC1 | G: semua peran termasuk Super Admin; W: periksa OpenAPI dan UI untuk fitur ubah/hapus audit log; T: tidak ada endpoint dan tidak ada tombol | negatif | integrasi | T1 | F1 |
| TC-US-1504-2 | US-1504 AC2 | G: token Super Admin dan Auditor; W: kirim `PUT`/`PATCH`/`DELETE`/`POST` ubah ke URL event audit; T: ditolak (403/404/405), tidak ada perubahan | negatif | integrasi | T1 | F1 |
| TC-US-1504-3 | US-1504 AC2 | G: permintaan ubah/hapus ditolak; W: periksa log; T: tercatat sebagai kejadian keamanan (security log), tanpa data sensitif | negatif | integrasi | T1 | F1 |
| TC-US-1504-4 | US-1504 AC2 | G: permintaan tanpa token atau token tidak sah; W: coba ubah/hapus; T: 401 dan kejadian keamanan tercatat | negatif | integrasi | T1 | F1 |
| TC-US-1504-5 | US-1504 AC3 | G: role database `audit_writer`, `audit_relay`, aplikasi; W: `UPDATE`, `DELETE`, `TRUNCATE` pada `audit_event`; T: semuanya ditolak database | negatif | integrasi | T1 | F1 |
| TC-US-1504-6 | US-1504 AC3 | G: event dengan `seq` = k diubah langsung di database (superuser melewati trigger penolak, lingkungan uji); W: job verifikasi dipicu manual; T: ketidakcocokan terdeteksi, hasil verifikasi menyebut `seq` = k sebagai yang pertama tidak cocok, entri security log peringatan keamanan tercatat | negatif | integrasi | T1 | F1 |
| TC-US-1504-7 | US-1504 AC3 | G: event `seq` = k dihapus dari tengah rantai, atau `prev_hash`-nya diganti; W: job verifikasi; T: terdeteksi, hasil menyebut `seq` pertama yang tidak cocok (k atau k+1 sesuai kasus), entri security log tercatat | negatif | integrasi | T1 | F1 |
| TC-US-1504-10 | US-1504 AC3 | G: event palsu disisipkan langsung di database di antara dua event; W: job verifikasi; T: terdeteksi, `seq` pertama yang tidak cocok disebut, entri security log tercatat | negatif | integrasi | T1 | F1 |
| TC-US-1504-8 | US-1504 AC3 | G: event terakhir dihapus (pemotongan ujung), sehingga kepala rantai di jangkar WORM berbeda dengan hasil hitung ulang; W: job verifikasi; T: terdeteksi lewat jangkar dan diperingatkan (entri security log) | negatif | integrasi | T1 | F1 |
| TC-US-1504-9 | US-1504 AC3 | G: rantai utuh; W: job verifikasi terjadwal harian dan pemicu manual; T: verifikasi lulus dan hasilnya tercatat (syarat demo MVP) | positif | integrasi | T1 | F1 |
| TC-US-1504-11 | US-1504 AC3, US-1501 AC3 | G: staging dengan Alertmanager; W: manipulasi seperti TC-US-1504-6 lalu verifikasi; T: peringatan email ke tim teknis terkirim seperti TC-US-1501-18 (menunggu: daftar penerima) | negatif | integrasi | T1 | F2 |

### MVP end-to-end Fase 1

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-MVP-1 | US-101 AC1–AC2, AC5, US-103 AC1, US-104 AC2, US-106 AC1, US-201 AC2, US-1504 AC3 | G: stack lokal bersih, data sintetis; W: pasien daftar dan setuju, verifikasi email, masuk dan mendaftarkan TOTP, unggah NIK dan foto KTP; Verifikator masuk dengan MFA dan menyetujui; T: status pasien "Disetujui", E-01/E-04/E-05/E-07/E-10/E-13 tercatat, verifikasi rantai audit lulus (demo F1-T70) | positif | E2E | T1 | F1 |
| TC-MVP-2 | US-201 AC3, US-106 AC1, US-103 AC1 | G: seperti TC-MVP-1; W: Verifikator menolak dengan alasan; pasien masuk dengan TOTP, melihat alasan, dan mengunggah ulang; Verifikator menyetujui; T: status akhir "Disetujui", dua siklus tercatat di audit | positif | E2E | T1 | F1 |

---

## 5. Fase 2: konsultasi online, pembayaran, notifikasi

Story Fase 2: US-107, US-202–US-207, US-301–US-311, US-801–US-806, US-808, US-809, US-812, US-1001–US-1003, US-1406 (pembayaran, video, email), ditambah US-104 AC4 dan US-201 AC2 (di §4) `[DOK: 07-phase-plan.md]` §4. Uji uang memakai nominal tepat; semua uang diuji sebagai bilangan bulat Rupiah. Kewajiban per pihak diuji lewat jumlah dan arah entri pembalik. TC yang bergantung pada sandbox DOKU ditandai "menunggu sandbox" sampai PT-36 terjawab (fake kontrak dipakai sebelum itu). Setiap endpoint menjalankan TC-SEC-1–TC-SEC-5, TC-SEC-8, TC-SEC-9; endpoint portal rekanan menjalankan TC-SEC-6.

### US-107 Pembatasan layanan berdasarkan status verifikasi (BR-003, BR-004, BR-008)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-107-1 | US-107 AC1 | G: penerima layanan berstatus Belum diunggah, Menunggu verifikasi, atau Ditolak (satu test per status); W: pasien memesan konsultasi (juga home visit dan klinik/lab saat tersedia); T: dihentikan sebelum pembayaran, pesan status dan tautan ke langkah berikutnya; tidak ada pembayaran dibuat | negatif | integrasi | T1 | F2 |
| TC-US-107-2 | US-107 AC1 | G: pesanan dikirim langsung ke API tanpa lewat UI untuk penerima belum "Disetujui"; W: kirim; T: ditolak oleh server | negatif | integrasi | T1 | F2 |
| TC-US-107-3 | US-107 AC2 | G: identitas belum disetujui; W: membeli produk; T: pembelian tetap dapat dilakukan | positif | integrasi | - | F4 |
| TC-US-107-4 | US-107 AC3 | G: identitas "Disetujui"; W: memesan layanan; T: berlanjut ke pemilihan jadwal | positif | E2E | - | F2 |
| TC-US-107-5 | US-107 AC1 | G: identitas diturunkan ke "Menunggu verifikasi" setelah ubah nama (US-105 AC3) saat ada pesanan terbayar; W: pesan layanan baru; T: diblokir, pesanan terbayar tetap berlaku | negatif | integrasi | T1 | F2 |

### US-110 Perangkat dan sesi aktif (BR-001, BR-108)

Story baru Checkpoint 2. Berlaku di aplikasi pasien, aplikasi tenaga kesehatan, backoffice, dan portal rekanan (F2); bagian web RME diverifikasi di F3 bersama EN-22. Test case dijalankan parametrik per aplikasi. Setiap endpoint juga menjalankan TC-SEC-1–TC-SEC-5, TC-SEC-8, TC-SEC-9.

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-110-1 | US-110 AC1 | G: akun dengan tiga sesi aktif di aplikasi berbeda (misalnya aplikasi pasien di dua perangkat, atau backoffice di dua browser); W: membuka "Perangkat dan sesi aktif"; T: ketiga sesi tampil dengan nama aplikasi, alamat IP, waktu mulai, waktu terakhir aktif (sesuai KU-07), penanda "Sesi ini" hanya pada sesi yang dipakai, dan nama perangkat bila tersedia; isi daftar cocok dengan sesi di Keycloak | positif | E2E | T1 | F2 |
| TC-US-110-2 | US-110 AC1, KU-03 | G: akun A dan B masing-masing punya sesi; W: A membuka daftar sesi, termasuk dengan menyelipkan ID pengguna B di query/body; T: hanya sesi milik A yang tampil | negatif | integrasi | T1 | F2 |
| TC-US-110-3 | US-110 AC2 | G: autentikasi terakhir 4 menit 59 detik dan 5 menit 1 detik sebelum permintaan; W: memilih mengakhiri satu sesi lain; T: yang pertama langsung diproses; yang kedua meminta masuk ulang lebih dulu | batas | integrasi | T1 | F2 |
| TC-US-110-4 | US-110 AC2 | G: autentikasi ulang berhasil; W: mengakhiri sesi S di perangkat lain; T: S berakhir di Keycloak dan tercatat di `session_revocation`; permintaan pertama yang mulai diproses setelah aksi mengembalikan sukses, dengan token akses atau token penyegar S, ditolak `4010001`; WebSocket S diputus; S tidak lagi tampil di daftar | positif | integrasi | T1 | F2 |
| TC-US-110-5 | US-110 AC2 | G: autentikasi terakhir > 5 menit; W: autentikasi ulang gagal atau dibatalkan, atau API akhiri sesi dipanggil langsung tanpa autentikasi ulang; T: tidak ada sesi yang diakhiri, token S tetap diterima | negatif | integrasi | T1 | F2 |
| TC-US-110-6 | US-110 AC3 | G: akun dengan sesi di tiga perangkat lain dan "Sesi ini"; W: "Akhiri semua sesi lain" setelah autentikasi ulang; T: ketiga sesi lain berakhir dengan perilaku TC-US-110-4; "Sesi ini" tetap berjalan | positif | integrasi | T1 | F2 |
| TC-US-110-7 | US-110 AC4 | G: pengguna di "Sesi ini"; W: memilih mengakhiri "Sesi ini" dan mengonfirmasi; T: perilaku sama dengan keluar (US-103 AC5): token sesi ini ditolak `4010001`, E-03 tercatat | positif | E2E | T1 | F2 |
| TC-US-110-8 | US-110 AC5 | G: ID sesi milik akun B, dan ID sesi acak yang tidak ada; W: akun A meminta mengakhirinya; T: kedua permintaan ditolak dengan respons identik (tidak membocorkan keberadaan sesi B); sesi B tetap aktif | negatif | integrasi | T1 | F2 |
| TC-US-110-9 | US-110 AC6 | G: sesi backoffice dan portal rekanan di browser X; W: sesi itu diakhiri dari perangkat lain, lalu browser X mengirim permintaan berikutnya; T: BFF menolak permintaan itu dan menampilkan halaman masuk; cookie sesi lama tidak lagi memberi akses | negatif | E2E | T1 | F2 |
| TC-US-110-10 | US-110 AC7, NFR-SEC-11 | G: pengguna punya sesi aktif di perangkat P1; W: masuk di perangkat P2, P3, dan P4; T: sesi P1–P3 tetap aktif dan tampil di daftar, tidak ada sesi yang dikeluarkan karena batas jumlah | positif | integrasi | T1 | F2 |
| TC-US-110-11 | US-110 AC8 | G: dua sesi diakhiri lewat "Akhiri semua sesi lain" dan satu lewat akhiri satu sesi; W: periksa audit dan log; T: tiga E-03 (satu per sesi) dengan keterangan "diakhiri dari daftar sesi"; tidak ada nilai token di audit, application log, security log | negatif | integrasi | T1 | F2 |
| TC-US-110-12 | US-110 AC1–AC8 | G: web RME (realm `rme`); W: TC-US-110-1–11 diulang untuk sesi web RME, termasuk AC6 lewat BFF RME; T: perilaku sama | negatif | integrasi | T1 | F3 |

### US-202 Pendaftaran tenaga kesehatan (BR-005, BR-102)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-202-1 | US-202 AC1 | G: klinik rekanan aktif; W: tenaga kesehatan mengisi data diri, jenis, klinik penaung, nomor dan foto STR, nomor/tanggal berakhir/foto SIP, dokumen penugasan; T: akun "Menunggu verifikasi", E-10 | positif | E2E | T1 | F2 |
| TC-US-202-2 | US-202 AC1 | G: salah satu dokumen wajib (PB-14) tidak dilampirkan; W: kirim; T: ditolak per field | negatif | integrasi | T1 | F2 |
| TC-US-202-3 | US-202 AC1 | G: berkas STR/SIP berjenis palsu, PDF berkonten aktif, atau melebihi batas (foto 5 MB, PDF 10 MB); W: unggah; T: ditolak | negatif | integrasi | T1 | F2 |
| TC-US-202-4 | US-202 AC2 | G: akun "Menunggu verifikasi" atau "Ditolak"; W: masuk ke aplikasi tenaga kesehatan; T: hanya status dan melengkapi dokumen; tidak tampil di pencarian pasien | negatif | integrasi | T1 | F2 |
| TC-US-202-5 | US-202 AC3 | G: tanggal berakhir SIP kemarin, hari ini, dan besok; W: kirim; T: kemarin ditolak dengan pesan SIP berakhir; hari ini dan besok mengikuti definisi "sudah lewat" (batas hari ini diverifikasi) | batas | unit | T1 | F2 |
| TC-US-202-6 | US-202 AC4 | G: klinik tidak aktif atau di-soft delete; W: tenaga kesehatan mencari klinik; T: tidak tampil di daftar | negatif | integrasi | - | F2 |

### US-203 Verifikasi tenaga kesehatan (BR-005, BR-102)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-203-1 | US-203 AC1 | G: pengajuan lengkap, Verifikator mencatat konfirmasi afiliasi (tanggal, cara, nama petugas klinik); W: setujui; T: akun aktif; E-05 dan E-10 tercatat; siap melayani hanya setelah tarif diatur (AB-54) | positif | E2E | T1 | F2 |
| TC-US-203-2 | US-203 AC2 | G: konfirmasi afiliasi belum dicatat; W: Verifikator menyetujui (UI dan API); T: persetujuan ditolak | negatif | integrasi | T1 | F2 |
| TC-US-203-3 | US-203 AC3 | G: Verifikator menolak dengan alasan; W: simpan; T: tenaga kesehatan diberi tahu alasan dan dapat memperbaiki; penolakan tanpa alasan tidak tersimpan | negatif | integrasi | T1 | F2 |
| TC-US-203-4 | US-203 AC1 | G: peran selain Verifikator; W: membuka dokumen STR/SIP atau menyetujui; T: ditolak, E-05 hasil gagal tercatat | negatif | integrasi | T1 | F2 |
| TC-US-203-5 | US-203 AC1 | G: dua Verifikator memutuskan bersamaan; W: setuju dan tolak; T: keputusan pertama berlaku, kedua ditolak dan tercatat | negatif | integrasi | T1 | F2 |

### US-204 Pemantauan masa berlaku SIP (BR-102)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-204-1 | US-204 AC1 | G: SIP berakhir tepat 30 hari lagi (PB-09), dan 31 hari lagi; W: pemeriksaan harian berjalan (`FixedClock`); T: pada 30 hari tenaga kesehatan, admin klinik (email), dan Verifikator diberi tahu; pada 31 hari tidak; tidak ada pemberitahuan ganda pada pemeriksaan hari berikutnya | batas | unit | - | F2 |
| TC-US-204-2 | US-204 AC2 | G: tanggal berakhir SIP terlewati; W: pemeriksaan harian; T: akun dinonaktifkan otomatis, tidak tampil untuk pasien, event dengan aktor "sistem" | positif | integrasi | T1 | F2 |
| TC-US-204-3 | US-204 AC2 | G: pemeriksaan harian dijalankan dua kali bersamaan; W: kedua berjalan; T: penonaktifan satu kali, satu event (job tunggal) | negatif | integrasi | T1 | F2 |
| TC-US-204-4 | US-204 AC3 | G: tenaga kesehatan dinonaktifkan karena SIP berakhir, punya sesi konsultasi terbayar di masa depan dan home visit ditugaskan; W: penonaktifan; T: sesi menjadi "Dibatalkan rekanan" dengan refund RF-05 otomatis, home visit kembali "Terkonfirmasi" (home visit diverifikasi di F4), pasien dan klinik diberi tahu | positif | integrasi | T1 | F2 |
| TC-US-204-5 | US-204 AC3 | G: sesi konsultasi sedang berlangsung saat SIP berakhir; W: penonaktifan; T: sesi yang sudah mulai tidak dibatalkan (hanya yang di masa depan) | batas | integrasi | T1 | F2 |
| TC-US-204-6 | US-204 AC4 | G: akun nonaktif karena SIP; W: tenaga kesehatan mengunggah SIP baru dan Verifikator menyetujui; T: akun aktif kembali, E-10 | positif | integrasi | T1 | F2 |

### US-205 Onboarding rekanan (BR-006, BR-031)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-205-1 | US-205 AC1 | G: data rekanan lengkap (nama, jenis, alamat, zona waktu, izin, rekening, sumber sistem); W: Verifikator menyimpan; T: "Menunggu verifikasi", E-10; klinik dapat sekaligus outlet | positif | E2E | T1 | F2 |
| TC-US-205-2 | US-205 AC1 | G: field wajib kosong, zona waktu di luar WIB/WITA/WIT; W: simpan; T: ditolak per field | negatif | unit | - | F2 |
| TC-US-205-3 | US-205 AC2 | G: izin disetujui, markup diatur, integrasi "Aktif"; W: pasien mencari; T: rekanan tampil (integrasi Aktif tersedia di F3; di F2 memakai stub status, T-23) | positif | integrasi | T1 | F2 |
| TC-US-205-4 | US-205 AC2 | G: tepat satu dari tiga syarat tidak terpenuhi (tiga test); W: pasien mencari; T: rekanan tidak tampil | negatif | integrasi | T1 | F2 |
| TC-US-205-5 | US-205 AC3 | G: toko kecantikan atau toko alat kesehatan dengan produk berkategori obat bebas; W: katalog diterima; T: produk obat bebas tidak ditampilkan | negatif | integrasi | - | F4 |
| TC-US-205-6 | US-205 AC1 | G: peran selain Verifikator; W: membuat atau menyetujui rekanan; T: 403 | negatif | integrasi | T1 | F2 |
| TC-US-205-7 | US-205 AC1 | G: rekanan di-soft delete atau dinonaktifkan; W: pasien mencari; T: tidak tampil, tidak dapat dipilih (KU-04) | negatif | integrasi | T1 | F2 |

### US-206 Akun admin mitra (BR-006)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-206-1 | US-206 AC1 | G: rekanan terverifikasi; W: Verifikator membuat admin mitra; T: undangan aktivasi terkirim, admin wajib atur password dan MFA, E-10 dan E-11 | positif | E2E | T1 | F2 |
| TC-US-206-2 | US-206 AC1 | G: rekanan belum terverifikasi; W: buat admin mitra; T: ditolak | negatif | integrasi | T1 | F2 |
| TC-US-206-3 | US-206 AC1 | G: tautan aktivasi kedaluwarsa atau sudah dipakai; W: dibuka; T: ditolak | negatif | integrasi | T1 | F2 |
| TC-US-206-4 | US-206 AC2 | G: admin mitra rekanan A; W: akses data rekanan B lewat ID, daftar, dan query (TC-SEC-6); T: tidak terbaca, 404 | negatif | integrasi | T1 | F2 |
| TC-US-206-5 | US-206 AC3 | G: admin mitra; W: mencari menu atau memanggil API buat/ubah admin lain; T: menu tidak ada, API 403 | negatif | integrasi | T1 | F2 |

### US-207 Menonaktifkan dan mengaktifkan akun (BR-060)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-207-1 | US-207 AC1 | G: Super Admin, Verifikator (nakes dan rekanan), Layanan Pelanggan (pasien); W: nonaktifkan akun dengan alasan; T: akun tidak dapat masuk (KU-10), alasan tersimpan, E-10 | positif | integrasi | T1 | F2 |
| TC-US-207-2 | US-207 AC1 | G: Verifikator mencoba menonaktifkan pasien, Layanan Pelanggan mencoba menonaktifkan nakes, Keuangan mencoba apa pun; W: kirim; T: 403 | negatif | integrasi | T1 | F2 |
| TC-US-207-3 | US-207 AC1 | G: penonaktifan tanpa alasan; W: simpan; T: ditolak | negatif | integrasi | T1 | F2 |
| TC-US-207-4 | US-207 AC1, KU-10 | G: akun punya sesi aktif; W: dinonaktifkan; T: permintaan berikutnya (definisi KU-10) ditolak untuk token dan refresh, aplikasi menampilkan layar "Akun nonaktif" dengan kontak Layanan Pelanggan, koneksi WebSocket diputus | negatif | integrasi | T1 | F2 |
| TC-US-207-5 | US-207 AC2 | G: nakes/rekanan dengan sesi konsultasi dan pendaftaran terbayar yang belum mulai; W: dinonaktifkan; T: menjadi "Dibatalkan rekanan" dengan RF-05 otomatis (pendaftaran klinik di F3) | positif | integrasi | T1 | F2 |
| TC-US-207-6 | US-207 AC2 | G: home visit ditugaskan ke nakes; W: nakes dinonaktifkan; T: kembali "Terkonfirmasi" (F4); pesanan produk outlet belum Dikirim menjadi "Ditolak outlet" dengan RF-08, yang sudah Dikirim tetap berjalan (F4) | batas | integrasi | T1 | F4 |
| TC-US-207-7 | US-207 AC3 | G: pasien punya pesanan terbayar belum selesai; W: admin menonaktifkan pasien; T: ditolak | negatif | integrasi | T1 | F2 |
| TC-US-207-8 | US-207 AC4 | G: akun nonaktif; W: admin mengaktifkan kembali dengan alasan (dan tanpa alasan); T: dengan alasan akun dapat dipakai; tanpa alasan ditolak; E-10 | positif | integrasi | T1 | F2 |
| TC-US-207-9 | US-207 AC5 | G: pasien dengan TOTP dan sesi aktif di dua perangkat; W: Layanan Pelanggan memulai reset MFA dengan alasan, pasien membuka tautan di email akun dan memasukkan password benar; T: reset berlaku; semua sesi pasien berakhir (token ditolak `4010001`); email pemberitahuan dengan kontak Layanan Pelanggan tertangkap; masuk berikutnya mewajibkan pendaftaran TOTP baru (US-101 AC5) | positif | E2E | T1 | F2 |
| TC-US-207-10 | US-207 AC5 | G: Layanan Pelanggan memasukkan alamat email lain (misalnya alamat yang disebut penelepon) di permintaan API reset; W: reset dimulai; T: tautan hanya terkirim ke email akun, alamat lain diabaikan atau permintaan ditolak; tidak ada email ke alamat lain | negatif | integrasi | T1 | F2 |
| TC-US-207-11 | US-207 AC5 | G: reset dimulai, tautan belum dibuka; W: pasien masuk dengan TOTP lama; T: TOTP lama tetap berlaku, tidak ada perubahan faktor | negatif | integrasi | T1 | F2 |
| TC-US-207-12 | US-207 AC5 | G: tautan reset dibuka; W: password salah dimasukkan; T: reset tidak berlaku, TOTP lama tetap berlaku (penghitungan penguncian dan nasib tautan: T-41) | negatif | integrasi | T1 | F2 |
| TC-US-207-13 | US-207 AC5 | G: tautan reset dikirim pada waktu t; W: dibuka pada t + 59 menit (berhasil), dibuka ulang setelah dipakai, dan (tautan lain) dibuka pada t + 61 menit; T: yang pertama berlaku; pemakaian ulang dan yang kedaluwarsa tidak mengubah apa pun | batas | integrasi | T1 | F2 |
| TC-US-207-14 | US-207 AC5 | G: Layanan Pelanggan di layar dan API reset MFA; W: mencoba melihat, memilih, atau menetapkan password, rahasia TOTP, atau kode pemulihan pasien (field tambahan di body, endpoint lain); T: tidak ada field atau endpoint yang menampilkan atau menerimanya; permintaan dengan field itu ditolak, kredensial pasien tidak berubah | negatif | integrasi | T1 | F2 |
| TC-US-207-15 | US-207 AC5, KU-03 | G: sub-peran Super Admin, Verifikator, Keuangan, Konten, Auditor, Petugas PDP (satu test per sub-peran, termasuk akun Super Admin yang juga memegang sub-peran lain selain Layanan Pelanggan), dan Layanan Pelanggan sebagai kontrol; W: memulai reset MFA pasien lewat UI dan API langsung; T: keenam sub-peran ditolak 403, tidak ada tautan terkirim dan TOTP pasien tidak berubah; hanya Layanan Pelanggan berhasil | negatif | integrasi | T1 | F2 |
| TC-US-207-16 | US-207 AC5, KU-01 | G: Layanan Pelanggan; W: memulai reset MFA tanpa alasan; T: ditolak per field, tidak ada tautan | negatif | integrasi | T1 | F2 |
| TC-US-207-17 | US-207 AC5, KU-06 | G: reset dimulai dan berlaku; W: periksa audit, application log, security log; T: E-04 dengan aktor Layanan Pelanggan dan alasan; tidak ada password, rahasia TOTP, kode pemulihan, atau tautan konfirmasi reset MFA (§7.3) | negatif | integrasi | T1 | F2 |

### US-301 Tarif dan durasi konsultasi (BR-018, BR-064)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-301-1 | US-301 AC1 | G: admin klinik; W: isi tarif Rp1 dan durasi 10, 15, 120 menit untuk chat dan video; T: tersimpan, E-14 | positif | integrasi | - | F2 |
| TC-US-301-2 | US-301 AC1 | G: tarif Rp0, negatif, desimal; durasi 5, 9, 12, 125 menit; W: simpan; T: ditolak (tarif > 0; durasi kelipatan 5 antara 10 dan 120) | batas | unit | - | F2 |
| TC-US-301-3 | US-301 AC2 | G: jenis belum diberi tarif; W: pasien melihat nakes; T: jenis itu tidak dapat dipesan | negatif | integrasi | - | F2 |
| TC-US-301-4 | US-301 AC3 | G: pembayaran dibuat dengan tarif lama lalu tarif diubah; W: pasien membayar setelah perubahan; T: pesanan lama tetap tarif lama; pembayaran baru memakai tarif baru | batas | integrasi | T1 | F2 |
| TC-US-301-5 | US-301 AC4 | G: admin klinik A; W: mengatur nakes klinik B lewat ID; T: ditolak (404), TC-SEC-6 | negatif | integrasi | T1 | F2 |
| TC-US-301-6 | US-301 AC5 | G: tarif dan markup rekanan; W: pasien melihat harga; T: harga = tarif + markup, satu angka | positif | unit | T1 | F2 |

### US-302 Jadwal praktik online (BR-017)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-302-1 | US-302 AC1 | G: nakes aktif; W: buat jadwal mingguan berulang dan pengecualian tanggal; T: slot terbentuk sesuai durasi per jenis di zona perangkat, E-14 | positif | integrasi | - | F2 |
| TC-US-302-2 | US-302 AC1 | G: nakes belum aktif; W: buat jadwal; T: ditolak | negatif | integrasi | - | F2 |
| TC-US-302-3 | US-302 AC2 | G: jadwal baru tumpang tindih (sebagian, sama persis, bersebelahan tepat); W: simpan; T: tumpang tindih ditolak, bersebelahan tepat diterima | batas | unit | - | F2 |
| TC-US-302-4 | US-302 AC3 | G: slot sudah dipesan dan dibayar; W: hapus jadwal yang memuatnya; T: ditolak sampai sesi dibatalkan (US-307 AC4) | negatif | integrasi | T1 | F2 |
| TC-US-302-5 | US-302 AC4 | G: admin klinik membuka jadwal nakes kliniknya; W: ubah jadwal; T: aturan AC1–AC3 berlaku, perubahan tercatat atas nama admin klinik | positif | integrasi | T1 | F2 |
| TC-US-302-6 | US-302 AC4 | G: admin klinik A; W: ubah jadwal nakes klinik B; T: ditolak | negatif | integrasi | T1 | F2 |
| TC-US-302-7 | US-302 AC5 | G: slot dipesan dan dibayar; W: ubah jam atau durasi atau hapus; T: ditolak sampai sesi dibatalkan (RF-05) | negatif | integrasi | T1 | F2 |

### US-303 Cari dan pilih tenaga kesehatan (BR-010, BR-011, BR-012, BR-014)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-303-1 | US-303 AC1 | G: nakes siap melayani (AB-54) dengan slot, nakes aktif tanpa tarif, nakes nonaktif, nakes tanpa slot masa depan; W: pasien memilih jenis atau mencari nama; T: hanya yang siap dan punya minimal satu slot masa depan | negatif | integrasi | - | F2 |
| TC-US-303-2 | US-303 AC2 | G: hasil tampil; W: lihat kartu; T: nama, jenis, klinik penaung, harga chat dan/atau video (tarif + markup), slot terdekat | positif | E2E | - | F2 |
| TC-US-303-3 | US-303 AC3 | G: tidak ada hasil; W: pencarian selesai; T: pesan "tidak ada tenaga kesehatan tersedia" | negatif | E2E | - | F2 |
| TC-US-303-4 | US-303 AC1 | G: klinik penaung dinonaktifkan atau markup belum diatur; W: pencarian; T: nakesnya tidak tampil | negatif | integrasi | - | F2 |

### US-304 Pesan sesi konsultasi (BR-040)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-304-1 | US-304 AC1 | G: penerima terverifikasi; W: pilih jenis, slot, penerima, isi keluhan 500 karakter, lanjut; T: slot ditahan selama PB-07, masuk ringkasan harga (US-801) | positif | E2E | T1 | F2 |
| TC-US-304-2 | US-304 AC1 | G: keluhan 501 karakter; W: lanjut; T: ditolak; keluhan kosong diterima | batas | unit | - | F2 |
| TC-US-304-3 | US-304 AC1, KU-09 | G: dua pasien memilih slot terakhir yang sama; W: keduanya lanjut; T: satu berhasil menahan, yang lain diberi tahu sebelum membayar | negatif | integrasi | T1 | F2 |
| TC-US-304-4 | US-304 AC2 | G: pembayaran berhasil; W: notifikasi gateway diterima; T: sesi "Terkonfirmasi", pasien dan nakes diberi tahu, E-08 dan E-09 | positif | integrasi | T1 | F2 |
| TC-US-304-5 | US-304 AC3 | G: slot mulai 23 jam 59 menit lagi, tepat 24 jam, dan 24 jam 1 menit; W: pesan; T: kurang dari PB-01 diberi tahu pembatalan tidak di-refund sebelum membayar; tepat 24 jam tidak diberi peringatan (batas "kurang dari") | batas | unit | T1 | F2 |
| TC-US-304-6 | US-304 AC4 | G: pembayaran tidak selesai sampai PB-07 (tepat 30 menit dan 30 menit 1 detik); W: batas habis; T: "Kedaluwarsa", slot dilepas dan dapat dipesan pasien lain | batas | integrasi | T1 | F2 |
| TC-US-304-7 | US-304 AC1 | G: pasien memesan untuk penerima yang bukan miliknya (ID penerima orang lain); W: kirim; T: ditolak (404) | negatif | integrasi | T1 | F2 |
| TC-US-304-8 | US-304 AC1, KU-05 | G: lanjut ditekan dua kali; W: server menerima; T: satu pesanan, satu slot ditahan | negatif | integrasi | T1 | F2 |

### US-305 Sesi chat (BR-010, BR-016)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-305-1 | US-305 AC1 | G: sesi chat "Terkonfirmasi"; W: waktu mulai tiba, pihak pertama membuka ruang; T: ruang terbuka hanya untuk pasien dan nakes sesi itu, waktu bergabung dicatat server, status "Berlangsung" | positif | integrasi | T1 | F2 |
| TC-US-305-2 | US-305 AC1 | G: sebelum waktu mulai; W: pihak membuka ruang; T: ruang belum terbuka | negatif | integrasi | - | F2 |
| TC-US-305-3 | US-305 AC2 | G: ruang terbuka, penerima online; W: kirim teks dan foto (JPG/PNG ≤ 5 MB); T: sampai dengan p95 ≤ 2 detik, tersimpan | positif | integrasi | - | F2 |
| TC-US-305-4 | US-305 AC2 | G: foto berjenis palsu atau melebihi 5 MB; W: kirim; T: ditolak | negatif | integrasi | T1 | F2 |
| TC-US-305-5 | US-305 AC3 | G: koneksi pengirim terputus; W: kirim pesan; T: berstatus "belum terkirim" dan terkirim otomatis saat pulih selama sesi berjalan; tidak ganda | negatif | integrasi | - | F2 |
| TC-US-305-6 | US-305 AC3 | G: koneksi baru pulih setelah sesi berakhir; W: pesan tertunda; T: tidak terkirim | batas | integrasi | - | F2 |
| TC-US-305-7 | US-305 AC4 | G: durasi habis; W: waktu berakhir (tidak diperpanjang karena jaringan); T: ruang baca-saja; status "Selesai" bila kedua pihak pernah bergabung, selain itu berlaku US-307 | batas | integrasi | - | F2 |
| TC-US-305-8 | US-305 AC5 | G: pengguna lain (pasien lain, nakes lain, admin Ruang Sehat semua sub-peran); W: membuka isi chat lewat ID; T: ditolak, E-05 hasil gagal tercatat | negatif | integrasi | T1 | F2 |
| TC-US-305-9 | US-305 AC1, KU-10 | G: akun dinonaktifkan saat terhubung; W: pesan berikutnya; T: tidak menerima pesan, koneksi terputus | negatif | integrasi | T1 | F2 |
| TC-US-305-10 | US-305 AC5 | G: nakes sesi membuka riwayat chat setelah sesi selesai; W: buka; T: dapat dibuka dan E-05 tercatat | positif | integrasi | T1 | F2 |

### US-306 Sesi video (BR-010, BR-016)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-306-1 | US-306 AC1 | G: sesi video "Terkonfirmasi", waktu mulai tiba; W: pihak membuka ruang sesi dan memulai panggilan; T: tercatat bergabung, status "Berlangsung", pihak lain di ruang menerima panggilan masuk | positif | manual | - | F2 |
| TC-US-306-2 | US-306 AC1 | G: pihak lain tidak di ruang sesi / aplikasi tertutup; W: panggilan dimulai; T: tidak ada panggilan masuk (tanpa push) | negatif | manual | - | F2 |
| TC-US-306-3 | US-306 AC1 | G: token ruang video; W: pihak lain/di luar sesi meminta token; T: ditolak, token per peserta per sesi | negatif | integrasi | T1 | F2 |
| TC-US-306-4 | US-306 AC2 | G: panggilan terputus; W: menyambung ulang sebelum durasi habis; T: tersambung kembali tanpa menambah durasi | batas | manual | - | F2 |
| TC-US-306-5 | US-306 AC3 | G: durasi habis; W: waktu berakhir; T: panggilan diakhiri; "Selesai" hanya bila kedua pihak pernah bergabung | batas | integrasi | - | F2 |
| TC-US-306-6 | US-306 AC4 | G: sesi berlangsung; W: periksa aplikasi dan konfigurasi LiveKit; T: tidak ada fitur rekam, rekam dinonaktifkan di penyedia, tidak ada berkas rekaman | negatif | integrasi | T1 | F2 |

### US-307 Ketidakhadiran dan pembatalan oleh tenaga kesehatan (BR-045)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-307-1 | US-307 AC1 | G: nakes belum bergabung; W: batas PB-03 (15 menit) habis; T: "Tenaga kesehatan tidak hadir", refund RF-04 (seluruh pembayaran) otomatis, pasien diberi tahu | positif | integrasi | T1 | F2 |
| TC-US-307-2 | US-307 AC1 | G: nakes bergabung di menit 14:59, dan menit 15:01; W: batas diperiksa; T: 14:59 tidak ada ketidakhadiran; 15:01 menjadi tidak hadir | batas | unit | T1 | F2 |
| TC-US-307-3 | US-307 AC1 | G: durasi sesi 10 menit (lebih pendek dari PB-03); W: nakes tidak bergabung sampai akhir durasi; T: batas = akhir durasi, "Tenaga kesehatan tidak hadir" dan refund RF-04 | batas | unit | T1 | F2 |
| TC-US-307-4 | US-307 AC2 | G: nakes bergabung, pasien belum; W: batas PB-02 habis; T: "Pasien tidak hadir", tanpa refund (RF-03), pasien diberi tahu | positif | integrasi | T1 | F2 |
| TC-US-307-5 | US-307 AC2 | G: pasien bergabung di menit 14:59 dan 15:01; W: batas PB-02; T: 14:59 sesi berlangsung; 15:01 "Pasien tidak hadir" | batas | unit | T1 | F2 |
| TC-US-307-6 | US-307 AC3 | G: keduanya tidak bergabung; W: batas habis; T: berlaku AC1 (nakes tidak hadir, refund RF-04) | negatif | integrasi | T1 | F2 |
| TC-US-307-7 | US-307 AC4 | G: sesi "Terkonfirmasi" belum mulai; W: nakes membatalkan dengan alasan; T: "Dibatalkan rekanan", refund RF-05 otomatis, pasien diberi tahu | positif | integrasi | T1 | F2 |
| TC-US-307-8 | US-307 AC4 | G: pembatalan tanpa alasan, atau sesi sudah "Berlangsung"; W: batal; T: ditolak | negatif | integrasi | T1 | F2 |
| TC-US-307-9 | US-307 AC1, AB-36 | G: refund RF-04 sudah dibuat; W: kejadian lain untuk pesanan sama (nakes batal); T: refund kedua tidak dibuat, transisi tidak sah ditolak dan E-09 hasil gagal | negatif | integrasi | T1 | F2 |

### US-308 Catatan konsultasi (BR-013, BR-016, BR-114)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-308-1 | US-308 AC1 | G: sesi "Berlangsung" atau "Selesai"; W: nakes sesi mengisi catatan terstruktur dan memfinalkan; T: tersimpan, terlihat pasien (dan keluarga bila relevan), E-06 hanya daftar field dan versi | positif | integrasi | T1 | F2 |
| TC-US-308-2 | US-308 AC1 | G: sesi "Terkonfirmasi" atau "Dibatalkan"; W: isi catatan; T: ditolak | negatif | integrasi | T1 | F2 |
| TC-US-308-3 | US-308 AC1 | G: nakes bukan penangani sesi; W: isi/baca catatan; T: ditolak, E-05 gagal | negatif | integrasi | T1 | F2 |
| TC-US-308-4 | US-308 AC2 | G: catatan final; W: penulis mengubah (dengan dan tanpa alasan); T: dengan alasan terbit versi baru, versi lama tetap dan tampak sebagai riwayat; tanpa alasan ditolak | positif | integrasi | T1 | F2 |
| TC-US-308-5 | US-308 AC3 | G: pengguna selain penulis (nakes lain, admin klinik, admin Ruang Sehat); W: mengubah; T: ditolak | negatif | integrasi | T1 | F2 |
| TC-US-308-6 | US-308 AC1 | G: catatan dengan diagnosis; W: periksa audit dan log; T: tidak ada isi catatan di audit/log, hanya referensi dan versi | negatif | integrasi | T1 | F2 |

### US-309 Resep digital (BR-015, BR-114)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-309-1 | US-309 AC1 | G: dokter, sesi "Berlangsung" atau "Selesai"; W: isi daftar obat (nama, dosis, jumlah, aturan pakai) dan terbitkan; T: resep bernomor unik memuat identitas pasien, nama dokter dan SIP, klinik, tanggal; E-06 | positif | integrasi | T1 | F2 |
| TC-US-309-2 | US-309 AC1 | G: daftar obat kosong, atau field obat kosong; W: terbitkan; T: ditolak | negatif | unit | - | F2 |
| TC-US-309-3 | US-309 AC1 | G: dua penerbitan paralel; W: server menerima; T: dua nomor unik tanpa tabrakan atau satu resep (idempoten untuk kiriman ganda) | negatif | integrasi | T1 | F2 |
| TC-US-309-4 | US-309 AC2 | G: psikolog, fisioterapis, tenaga medis (satu test per jenis); W: cari atau panggil API resep; T: fitur tidak ada, API 403 | negatif | integrasi | T1 | F2 |
| TC-US-309-5 | US-309 AC3 | G: resep terbit; W: dokter mengubah isi; T: ditolak; dibatalkan dengan alasan lalu terbit resep baru; resep batal tersimpan berstatus "Dibatalkan" | negatif | integrasi | T1 | F2 |
| TC-US-309-6 | US-309 AC4 | G: resep terbit; W: pasien membuka dan mengunduh PDF; T: tampil dan PDF terunduh, E-12 tercatat; pengguna lain ditolak | positif | integrasi | T1 | F2 |
| TC-US-309-7 | US-309 AC6 | G: resep terbit; W: PDF dibuat; T: memuat kode verifikasi acak, QR ke halaman verifikasi di domain resmi, alamat dalam teks | positif | integrasi | T1 | F2 |
| TC-US-309-8 | US-309 AC6 | G: dua resep; W: bandingkan kode verifikasi; T: acak, tidak berurutan dan tidak dapat ditebak dari nomor resep | negatif | unit | T1 | F2 |
| TC-US-309-9 | US-309 AC7 | G: kode valid (Terbit dan Dibatalkan); W: buka halaman verifikasi publik tanpa login; T: status, tanggal terbit, dokter dan SIP, klinik, daftar obat, nama pasien disamarkan | positif | E2E | T1 | F2 |
| TC-US-309-10 | US-309 AC7 | G: kode tidak valid; W: buka halaman; T: "resep tidak ditemukan", tanpa membedakan format salah dan tidak ada | negatif | integrasi | T1 | F2 |
| TC-US-309-11 | US-309 AC8 | G: banyak percobaan verifikasi dari sumber yang sama; W: melewati batas laju (nilai: T-33); T: ditolak sementara, lalu pulih | batas | integrasi | T1 | F2 |
| TC-US-309-12 | US-309 AC8 | G: verifikasi menampilkan data resep; W: periksa audit; T: E-05 dengan aktor "pemeriksa anonim" dan alamat IP | positif | integrasi | T1 | F2 |

### US-310 Riwayat konsultasi pasien (BR-016, BR-106, BR-008)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-310-1 | US-310 AC1 | G: pasien dengan sesi diri; W: buka riwayat; T: daftar tanggal, nakes, jenis, status (anggota keluarga ditambah di F4) | positif | E2E | - | F2 |
| TC-US-310-2 | US-310 AC1, AC2 | G: pemilik akun membuka data anggota keluarga (dewasa dan anak); W: buka; T: tampil dan E-05 tercatat | positif | integrasi | T1 | F4 |
| TC-US-310-3 | US-310 AC2 | G: sesi selesai; W: buka detail; T: chat baca-saja, catatan final, resep | positif | E2E | - | F2 |
| TC-US-310-4 | US-310 AC1 | G: pasien lain; W: buka riwayat pasien A lewat ID; T: 404, E-05 gagal | negatif | integrasi | T1 | F2 |
| TC-US-310-5 | US-310 AC3 | G: anggota keluarga di-soft delete; W: pemilik akun membuka riwayat; T: riwayat tetap dapat dilihat | positif | integrasi | T1 | F4 |

### US-311 Daftar sesi tenaga kesehatan (BR-013)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-311-1 | US-311 AC1 | G: nakes dengan sesi terkonfirmasi (juga di zona waktu perangkat berbeda); W: buka daftar; T: sesi hari ini dan mendatang dengan waktu zona perangkat, jenis, nama pasien; sesi nakes lain tidak tampil | positif | E2E | - | F2 |
| TC-US-311-2 | US-311 AC2 | G: pesanan baru atau pengingat PB-08; W: kejadian terjadi; T: email dan notifikasi di dalam aplikasi (US-1003) | positif | integrasi | - | F2 |
| TC-US-311-3 | US-311 AC3 | G: waktu mulai belum tiba (sebelum dan tepat saat mulai); W: tekan bergabung; T: sebelum belum aktif, tepat saat mulai aktif | batas | E2E | - | F2 |
| TC-US-311-4 | US-311 AC4 | G: nakes membuka detail sesi yang ditanganinya; W: buka; T: keluhan awal tampil dan E-05 tercatat | positif | integrasi | T1 | F2 |
| TC-US-311-5 | US-311 AC4 | G: nakes membuka detail sesi nakes lain (ganti ID); W: buka; T: 404, E-05 gagal | negatif | integrasi | T1 | F2 |

### US-801 Ringkasan harga sebelum membayar (BR-042, BR-044, BR-024)

Nilai pajak nyata dan perilaku saat tarif belum diatur belum ditetapkan (sisa PT-28, T-24); test memakai tarif uji sebagai parameter. EN-24 selesai sebelum test ini.

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-801-1 | US-801 AC1 | G: harga rekanan 100.000, markup 10.000, metode non-QRIS dengan biaya transaksi uji, tarif pajak uji; W: pilih metode; T: harga layanan satu angka (110.000), pajak sebagai baris terpisah dihitung dari harga rekanan + markup, ongkos kirim (produk), biaya transaksi, total, semua dihitung server | positif | unit | T1 | F2 |
| TC-US-801-2 | US-801 AC1 | G: metode QRIS; W: ringkasan; T: biaya transaksi Rp0 | positif | unit | T1 | F2 |
| TC-US-801-3 | US-801 AC1 | G: klien mengirim total atau harga sendiri di permintaan; W: server menghitung; T: nilai klien diabaikan, server memakai perhitungan sendiri | negatif | integrasi | T1 | F2 |
| TC-US-801-4 | US-801 AC2 | G: markup persen (misalnya 10% dari 12.345, 12,5% dari 10.001, 0%, 100%); W: hitung; T: dibulatkan per unit ke Rupiah penuh terdekat (setengah ke atas), baru dikalikan jumlah | batas | unit | T1 | F2 |
| TC-US-801-5 | US-801 AC2 | G: markup nominal 5.000 dengan jumlah 3; W: hitung; T: 5.000 per unit, total markup 15.000; nilai besar tidak meluap | batas | unit | T1 | F2 |
| TC-US-801-6 | US-801 AC2 | G: nilai harga dan jumlah ekstrem (luapan aritmetika uang); W: hitung; T: ditolak atau diperlakukan aman, tidak ada nilai salah diam-diam | negatif | unit | T1 | F2 |
| TC-US-801-7 | US-801 AC3 | G: layanan terjadwal; W: ringkasan tampil; T: aturan pembatalan (PB-01, no-show) tampil; untuk klinik/lab/home visit pernyataan AB-16 juga tampil (F3/F4) | positif | E2E | - | F2 |
| TC-US-801-8 | US-801 AC4 | G: pesanan produk; W: ringkasan; T: pernyataan tidak dapat dibatalkan setelah dibayar | positif | E2E | - | F4 |
| TC-US-801-9 | US-801 AC5 | G: markup atau tarif berubah setelah ringkasan tampil; W: pasien menekan bayar; T: total baru tampil dan harus disetujui sebelum pembayaran dibuat; tanpa persetujuan tidak ada pembayaran | negatif | integrasi | T1 | F2 |

### US-802 Pembayaran lewat payment gateway (BR-040, BR-115)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-802-1 | US-802 AC1 | G: pasien memilih metode dan bayar; W: pembayaran dibuat di gateway (sandbox); T: nominal sama dengan total ringkasan, pasien diarahkan ke instruksi/halaman bayar, E-08 | positif | integrasi | T1 | F2 |
| TC-US-802-2 | US-802 AC1 | G: pembayaran dibuat dua kali (kirim ganda atau coba ulang karena jaringan); W: server menerima; T: satu pembayaran per pesanan dan metode (idempoten, KU-02/KU-05) | negatif | integrasi | T1 | F2 |
| TC-US-802-3 | US-802 AC2 | G: notifikasi berhasil sah dari gateway; W: diproses; T: pesanan dikonfirmasi satu kali | positif | integrasi | T1 | F2 |
| TC-US-802-4 | US-802 AC2 | G: notifikasi yang sama diterima 3 kali (paralel dan berurutan); W: diproses; T: dikonfirmasi satu kali, satu catatan transaksi, satu pemberitahuan | negatif | integrasi | T1 | F2 |
| TC-US-802-5 | US-802 AC2 | G: notifikasi dengan tanda tangan salah, tanpa tanda tangan, atau dari sumber lain; W: diproses; T: ditolak, pesanan tidak berubah, tercatat di security log (NFR-INT-04) | negatif | integrasi | T1 | F2 |
| TC-US-802-6 | US-802 AC3 | G: nominal di notifikasi berbeda (lebih kecil, lebih besar, selisih Rp1); W: diproses; T: pesanan tidak dikonfirmasi, menjadi "Kedaluwarsa", masuk antrean Keuangan | negatif | integrasi | T1 | F2 |
| TC-US-802-7 | US-802 AC3 | G: Keuangan membuat refund RF-14 sebesar seluruh nominal diterima dalam PB-16 (1 hari kerja); W: refund dibuat; T: refund = nominal yang diterima, kewajiban penyebab platform | positif | integrasi | T1 | F2 |
| TC-US-802-8 | US-802 AC3 | G: PB-16 habis tanpa tindakan Keuangan (batas 23.59 WIB hari kerja ke-1; akhir pekan/libur tidak dihitung); W: batas terlewati; T: refund RF-14 dibuat otomatis, Super Admin diberi tahu | batas | integrasi | T1 | F2 |
| TC-US-802-9 | US-802 AC4 | G: pembayaran gagal atau pasien berganti metode sebelum PB-07 habis; W: coba metode lain; T: berhasil dan instruksi sebelumnya dibatalkan di gateway bila didukung; setelah PB-07 tidak bisa | batas | integrasi | T1 | F2 |
| TC-US-802-10 | US-802 AC5 | G: notifikasi berhasil datang setelah pesanan "Kedaluwarsa" dan slot masih tersedia; W: diproses; T: pesanan dikonfirmasi (Kedaluwarsa menjadi Terkonfirmasi) | positif | integrasi | T1 | F2 |
| TC-US-802-11 | US-802 AC5 | G: notifikasi berhasil setelah kedaluwarsa dan slot sudah diambil pasien lain; W: diproses; T: refund RF-13 otomatis seluruh pembayaran, pesanan tidak dikonfirmasi | negatif | integrasi | T1 | F2 |
| TC-US-802-12 | US-802 AC6 | G: pesanan sudah dibayar; W: notifikasi berhasil untuk pembayaran lain atas pesanan sama; T: refund RF-12 seluruh pembayaran ganda dibuat langsung; pesanan tetap terkonfirmasi satu kali | negatif | integrasi | T1 | F2 |
| TC-US-802-13 | US-802 AC1 | G: gateway gagal atau tidak merespons (galat, timeout); W: pasien menekan bayar; T: pesan gagal dan coba lagi, tidak ada pesanan terkonfirmasi, tidak ada pembayaran ganda | negatif | integrasi | T1 | F2 |
| TC-US-802-14 | US-802 AC2 | G: notifikasi dengan nominal sebagai angka sangat besar atau negatif, atau ID pesanan tidak ada; W: diproses; T: ditolak aman, tidak ada galat 500, tercatat | negatif | integrasi | T1 | F2 |
| TC-US-802-15 | US-802 AC1 | G: pembayaran dibuat; W: periksa data yang dikirim ke gateway; T: hanya data yang diperlukan (AB-75, batas menunggu L-08), tanpa diagnosis/keluhan | negatif | integrasi | T1 | F2 |

### US-803 Pembagian dana per transaksi (BR-041, BR-044)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-803-1 | US-803 AC1 | G: pembayaran QRIS berhasil (harga rekanan 100.000, markup 10.000, MDR uji); W: transaksi dicatat; T: bagian rekanan = harga rekanan dikurangi MDR, markup ke Ruang Sehat, MDR, rekening tujuan sebagai instruksi pembagian; jumlah seimbang | positif | unit | T1 | F2 |
| TC-US-803-2 | US-803 AC1 | G: pembayaran non-QRIS dengan biaya transaksi; W: transaksi dicatat; T: bagian rekanan = harga rekanan penuh; biaya transaksi tercatat; jumlah seimbang | positif | unit | T1 | F2 |
| TC-US-803-3 | US-803 AC1 | G: jurnal/catatan transaksi tidak seimbang (debit tidak sama kredit); W: simpan; T: ditolak | negatif | unit | T1 | F2 |
| TC-US-803-4 | US-803 AC1 | G: rekening tujuan; W: periksa audit dan log; T: nomor rekening hanya 4 digit terakhir | negatif | integrasi | T1 | F2 |
| TC-US-803-5 | US-803 AC2 | G: gateway melaporkan hasil pembagian dana; W: laporan diterima; T: hasil per pihak tercatat dan dapat dicocokkan (US-811); laporan ganda tidak menggandakan | positif | integrasi | T1 | F2 |
| TC-US-803-6 | US-803 AC3 | G: catatan transaksi sudah dibuat; W: koreksi (`UPDATE`/`DELETE` atau koreksi lewat aplikasi); T: `UPDATE`/`DELETE` ditolak; koreksi hanya lewat entri pembalik yang merujuk transaksi asal | negatif | integrasi | T1 | F2 |
| TC-US-803-7 | US-803 AC1 | G: instruksi pembagian dan harga dikunci saat pembayaran dibuat; W: markup berubah kemudian; T: transaksi lama tidak berubah (AB-17) | negatif | integrasi | T1 | F2 |

### US-804 Bukti pembayaran (BR-040, BR-014)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-804-1 | US-804 AC1 | G: pembayaran berhasil; W: bukti dibuat; T: tersedia di aplikasi dan terkirim ke email; memuat nomor transaksi, tanggal, rincian harga, metode, nama rekanan, pernyataan Ruang Sehat berperan sebagai platform | positif | integrasi | - | F2 |
| TC-US-804-2 | US-804 AC1 | G: pasien lain; W: membuka bukti pasien A lewat ID; T: 404 | negatif | integrasi | T1 | F2 |
| TC-US-804-3 | US-804 AC2 | G: rincian harga memuat pajak; W: bukti dibuat; T: baris pajak sama dengan rincian harga (isi wajib pajak dan merchant of record: T-24, belum diuji) | positif | integrasi | - | F2 |

### US-805 Pembatalan layanan terjadwal oleh pasien (BR-045)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-805-1 | US-805 AC1 | G: konsultasi "Terkonfirmasi", sisa waktu 24 jam 0 menit, dan 48 jam; W: pasien menekan batal; T: nominal refund (harga layanan, tanpa biaya transaksi non-QRIS) tampil dan perlu konfirmasi | batas | integrasi | T1 | F2 |
| TC-US-805-2 | US-805 AC2 | G: pasien mengonfirmasi pembatalan; W: simpan; T: "Dibatalkan pasien", slot dilepas, nakes diberi tahu, refund RF-01 otomatis, E-08 dan E-09 | positif | integrasi | T1 | F2 |
| TC-US-805-3 | US-805 AC3 | G: sisa waktu 23 jam 59 menit; W: tekan batal; T: diberi tahu tanpa refund (RF-02); bila tetap batal, "Dibatalkan pasien" tanpa refund dan slot dilepas | batas | integrasi | T1 | F2 |
| TC-US-805-4 | US-805 AC3 | G: PB-01 diubah super admin setelah pembayaran dibuat; W: batal; T: nilai saat pembayaran dibuat yang dipakai (AB-17) | batas | integrasi | T1 | F2 |
| TC-US-805-5 | US-805 AC4 | G: konsultasi "Berlangsung", atau klinik/lab "Check-in", home visit "Berangkat" atau sesudahnya, atau status akhir; W: cari tombol batal dan panggil API batal; T: tombol tidak ada, API menolak | negatif | integrasi | T1 | F2 |
| TC-US-805-6 | US-805 AC2 | G: batal ditekan dua kali (atau dua perangkat); W: server menerima; T: satu pembatalan, satu refund | negatif | integrasi | T1 | F2 |
| TC-US-805-7 | US-805 AC2 | G: pasien lain; W: membatalkan pesanan pasien A lewat ID; T: 404 | negatif | integrasi | T1 | F2 |

### US-806 Refund otomatis (BR-045)

Kejadian RF-08, RF-09, RF-15 berlaku untuk produk (F4); RF-18 untuk klinik/lab/home visit (F3) (T-29).

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-806-1 | US-806 AC1 | G: kejadian RF-01, RF-04, RF-05, RF-12, RF-13 (satu test per kejadian); W: sistem mencatat; T: refund dibuat otomatis dengan nominal sesuai matriks §3.5, tanpa persetujuan manual, pasien diberi tahu, kewajiban per pihak tercatat (AB-34) | positif | integrasi | T1 | F2 |
| TC-US-806-2 | US-806 AC1 | G: RF-18 (pesanan belum diterima rekanan saat jadwal mulai); W: jadwal mulai; T: refund seluruh pembayaran dibuat otomatis, kewajiban "Menunggu penetapan penyebab" | positif | integrasi | T1 | F3 |
| TC-US-806-3 | US-806 AC1 | G: RF-08, RF-09, RF-15; W: kejadian; T: refund otomatis seluruh pembayaran | positif | integrasi | T1 | F4 |
| TC-US-806-4 | US-806 AC2 | G: refund untuk satu pesanan sudah dibuat; W: kejadian lain atau kejadian sama tercatat (tersimpan lebih dulu yang berlaku); T: refund kedua tidak dibuat; transisi tidak sah ditolak dan E-09 hasil gagal | negatif | integrasi | T1 | F2 |
| TC-US-806-5 | US-806 AC2 | G: kejadian refund yang sama datang dua kali paralel; W: diproses; T: satu refund (KU-05) | negatif | integrasi | T1 | F2 |
| TC-US-806-6 | US-806 AC2 | G: refund pertama RF-12 atau RF-13 atau RF-14 sudah ada; W: refund atas pesanan yang sama untuk dana berbeda; T: diizinkan (pengecualian AB-36) | batas | integrasi | T1 | F2 |
| TC-US-806-7 | US-806 AC3 | G: RF-01 dengan QRIS dan non-QRIS (harga rekanan 100.000, markup 10.000, biaya transaksi uji); W: hitung; T: pasien menerima harga layanan tanpa biaya transaksi non-QRIS, kewajiban rekanan = harga rekanan penuh, kewajiban Ruang Sehat = markup | positif | unit | T1 | F2 |
| TC-US-806-8 | US-806 AC3 | G: kelompok penyebab rekanan (RF-04, RF-05) dan penyebab platform (RF-12, RF-13); W: hitung; T: pasien menerima seluruh pembayaran; kewajiban per pihak mengikuti AB-34 (tabel uji per kelompok dan metode) | positif | unit | T1 | F2 |
| TC-US-806-9 | US-806 AC3 | G: refund melebihi dana yang diterima; W: buat; T: ditolak | negatif | unit | T1 | F2 |
| TC-US-806-10 | US-806 AC3 | G: nominal dihitung dari harga yang dikunci; W: markup/tarif berubah setelahnya; T: nominal refund memakai harga terkunci (AB-17) | batas | unit | T1 | F2 |

### US-808 Eksekusi dan status refund (BR-045)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-808-1 | US-808 AC1 | G: refund dibuat; W: dieksekusi lewat gateway (sandbox) dan penyedia mengonfirmasi; T: "Diproses" lalu "Berhasil", entri pembalik kewajiban rekanan dan Ruang Sehat tercatat (AB-31), E-08 | positif | integrasi | T1 | F2 |
| TC-US-808-2 | US-808 AC1 | G: refund dieksekusi; W: eksekusi dikirim ulang karena timeout; T: satu penarikan dana (idempoten), tidak ada refund ganda | negatif | integrasi | T1 | F2 |
| TC-US-808-3 | US-808 AC2 | G: penyedia mengembalikan galat; W: eksekusi; T: "Gagal", masuk antrean Keuangan, pasien diberi tahu refund sedang ditangani; Keuangan memproses ulang dan status "Diproses" | negatif | integrasi | T1 | F2 |
| TC-US-808-4 | US-808 AC2 | G: peran selain Keuangan; W: memproses ulang refund; T: 403 | negatif | integrasi | T1 | F2 |
| TC-US-808-5 | US-808 AC4 | G: refund "Diproses" tepat 2 x 24 jam dan 2 x 24 jam + 1 menit tanpa konfirmasi; W: batas diperiksa (PB-19); T: lewat batas menjadi "Perlu dicek", masuk antrean Keuangan | batas | integrasi | T1 | F2 |
| TC-US-808-6 | US-808 AC4 | G: refund "Perlu dicek"; W: eksekusi ulang dicoba; T: ditolak sebelum status penyedia diketahui; setelah diketahui menjadi "Berhasil" atau "Gagal" | negatif | integrasi | T1 | F2 |
| TC-US-808-7 | US-808 AC3 | G: refund menimbulkan kewajiban rekanan; W: kewajiban tercatat; T: nominal tampil di laporan rekanan (US-1201) dan laporan Keuangan "Terbuka"; RF-18 belum ditetapkan hanya di laporan Keuangan sebagai "Menunggu penetapan penyebab" | positif | integrasi | T1 | F3 |
| TC-US-808-8 | US-808 AC5 | G: kewajiban rekanan dilunasi; W: Keuangan menandai "Selesai" dengan rujukan bukti; T: status berubah di kedua laporan, E-08; tanpa rujukan bukti ditolak | negatif | integrasi | T1 | F2 |
| TC-US-808-9 | US-808 AC6 | G: refund RF-18 "Menunggu penetapan penyebab"; W: Keuangan menetapkan penyebab rekanan atau platform dengan rujukan log integrasi dalam PB-20 (3 hari kerja); T: kewajiban dihitung sesuai AB-34 dan E-08 | positif | integrasi | T1 | F3 |
| TC-US-808-10 | US-808 AC6 | G: PB-20 habis tanpa penetapan (batas 23.59 WIB hari kerja ke-3); W: batas terlewati; T: berlaku penyebab platform, Super Admin diberi tahu | batas | integrasi | T1 | F3 |
| TC-US-808-11 | US-808 AC1 | G: pembalikan kewajiban; W: ubah atau hapus catatan asal; T: ditolak (immutable), hanya entri pembalik | negatif | integrasi | T1 | F2 |

### US-809 Pengaturan markup per rekanan (BR-042, BR-063)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-809-1 | US-809 AC1 | G: Keuangan; W: atur markup persen 0, 50, 100 atau nominal 0, 1.000 dengan waktu berlaku segera dan tanggal mendatang; T: tersimpan, riwayat tetap ada, E-14 | positif | integrasi | T1 | F2 |
| TC-US-809-2 | US-809 AC1 | G: persen -1, 100,01, kosong; nominal negatif, desimal; waktu berlaku di masa lalu; W: simpan; T: ditolak (KU-01) | batas | unit | T1 | F2 |
| TC-US-809-3 | US-809 AC2 | G: markup diubah dengan waktu berlaku besok; W: pembayaran dibuat sebelum waktu berlaku lalu diselesaikan sesudahnya; T: harga memakai markup lama (AB-17) | batas | integrasi | T1 | F2 |
| TC-US-809-4 | US-809 AC3 | G: rekanan belum punya markup; W: Verifikator mencoba menampilkan ke pasien; T: tidak tampil; markup 0 diperbolehkan | negatif | integrasi | T1 | F2 |
| TC-US-809-5 | US-809 AC4 | G: Super Admin, Verifikator, Layanan Pelanggan, Konten, Auditor, Petugas PDP, admin rekanan; W: ubah markup; T: ditolak (403) | negatif | integrasi | T1 | F2 |
| TC-US-809-6 | US-809 AC1 | G: dua perubahan markup bersamaan untuk rekanan sama; W: simpan; T: dua versi berurutan tanpa tabrakan, riwayat utuh | negatif | integrasi | T1 | F2 |

### US-812 Pengaturan parameter bisnis (BR-045, BR-063)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-812-1 | US-812 AC1 | G: Super Admin; W: ubah PB-01–PB-10 dan PB-14–PB-20 dengan nilai valid; T: tersimpan dengan riwayat, E-14 | positif | integrasi | T1 | F2 |
| TC-US-812-2 | US-812 AC1 | G: PB-14 dengan daftar dokumen kosong; W: simpan; T: ditolak | negatif | unit | T1 | F2 |
| TC-US-812-3 | US-812 AC2 | G: parameter diubah (misalnya PB-01 dari 24 ke 12 jam); W: pesanan dengan pembayaran dibuat sebelum perubahan dievaluasi; T: nilai yang berlaku saat pembayaran dibuat yang dipakai | batas | integrasi | T1 | F2 |
| TC-US-812-4 | US-812 AC3 | G: nilai kosong, negatif, nol untuk batas waktu, bukan angka; W: simpan; T: ditolak (KU-01) | batas | unit | T1 | F2 |
| TC-US-812-5 | US-812 AC1 | G: peran selain Super Admin; W: ubah parameter; T: 403 | negatif | integrasi | T1 | F2 |
| TC-US-812-6 | US-812 AC4 | G: Keuangan; W: atur tarif pajak PB-21 dengan waktu berlaku segera dan mendatang; T: tersimpan dengan riwayat, E-14, pesanan yang pembayarannya dibuat sebelum waktu berlaku memakai tarif lama (batas nilai sah: T-24) | batas | integrasi | T1 | F2 |
| TC-US-812-7 | US-812 AC4 | G: Super Admin (bukan Keuangan); W: ubah PB-21; T: ditolak | negatif | integrasi | T1 | F2 |

### US-1001 Notifikasi pasien per kejadian dan kanal (BR-050)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-1001-1 | US-1001 AC1 | G: tiap kejadian di matriks (satu test per baris, sesuai story yang sudah ada: T-29); W: kejadian terjadi; T: email diserahkan ke penyedia dan notifikasi di dalam aplikasi tersedia sesuai matriks ≤ 1 menit; baris "Ya/Tidak" kanal benar | positif | integrasi | - | F2 |
| TC-US-1001-2 | US-1001 AC1 | G: kejadian berstatus hanya in-app (ditugaskan, berangkat, tiba, dikirim, diterima, chat baru, antrean); W: terjadi; T: tidak ada email | negatif | integrasi | - | F2 |
| TC-US-1001-3 | US-1001 AC2 | G: notifikasi hasil lab, resep, refund, klaim; W: periksa isi email dan in-app; T: tanpa diagnosis, nama obat, nilai hasil lab, isi chat | negatif | integrasi | T1 | F2 |
| TC-US-1001-4 | US-1001 AC3 | G: penyedia email menolak atau gagal; W: kirim; T: dicoba ulang, kegagalan akhir tercatat, notifikasi in-app tetap tersedia | negatif | integrasi | - | F2 |
| TC-US-1001-5 | US-1001 AC3 | G: kanal in-app gagal; W: kirim; T: email tetap terkirim (kegagalan satu kanal tidak menghentikan kanal lain) | negatif | integrasi | - | F2 |
| TC-US-1001-6 | US-1001 AC4 | G: ada notifikasi; W: pasien membuka kotak notifikasi; T: urut terbaru, yang belum dibaca ditandai; ditandai dibaca setelah dibuka | positif | E2E | - | F2 |
| TC-US-1001-7 | US-1001 AC4 | G: pasien B; W: membuka notifikasi pasien A lewat ID; T: 404 | negatif | integrasi | T1 | F2 |
| TC-US-1001-8 | US-1001 AC5 | G: penyedia email terpilih; W: tinjau dokumen lokasi pemrosesan dan penyimpanan data; T: di Indonesia (bukti dokumen, T-27) | positif | manual | T1 | F2 |

### US-1002 Pengingat jadwal (BR-050)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-1002-1 | US-1002 AC1 | G: layanan "Terkonfirmasi"; W: waktu 24 jam dan 1 jam sebelum jadwal tiba (PB-08, `FixedClock`); T: pengingat email dan in-app terkirim, masing-masing sekali | positif | integrasi | - | F2 |
| TC-US-1002-2 | US-1002 AC2 | G: layanan dipesan 23 jam sebelum jadwal, dan 30 menit sebelum jadwal; W: dikonfirmasi; T: 23 jam hanya pengingat 1 jam; 30 menit tidak ada pengingat | batas | unit | - | F2 |
| TC-US-1002-3 | US-1002 AC3 | G: layanan dibatalkan; W: waktu pengingat tiba; T: tidak dikirim | negatif | integrasi | - | F2 |
| TC-US-1002-4 | US-1002 AC1 | G: job pengingat dijalankan dua kali bersamaan; W: berjalan; T: pengingat tidak ganda | negatif | integrasi | T1 | F2 |

### US-1003 Notifikasi tenaga kesehatan dan admin (BR-013, BR-102, BR-062)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-1003-1 | US-1003 AC1 | G: pesanan konsultasi baru, pembatalan; W: terjadi; T: nakes menerima email dan notifikasi in-app (tugas home visit di F4) | positif | integrasi | - | F2 |
| TC-US-1003-2 | US-1003 AC2 | G: nakes punya sesi; W: waktu PB-08 tiba; T: email dan in-app | positif | integrasi | - | F2 |
| TC-US-1003-3 | US-1003 AC3 | G: SIP akan berakhir (PB-09); W: terjadi; T: nakes dan admin kliniknya menerima email (keluhan baru terkait rekanan di F3) | positif | integrasi | - | F2 |
| TC-US-1003-4 | US-1003 AC1 | G: nakes lain / admin rekanan lain; W: menerima notifikasi bukan untuknya; T: tidak ada, tidak ada kebocoran lintas penerima | negatif | integrasi | T1 | F2 |

### US-1406 Integrasi pihak ketiga (BR-071), bagian Fase 2: pembayaran, video, email

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-1406-1 | US-1406 AC1 | G: penyedia pembayaran, video, email terpilih dan disetujui; W: seluruh TC US-802, US-306, US-1001 dijalankan terhadap penyedia (sandbox) | positif | integrasi | T1 | F2 |
| TC-US-1406-2 | US-1406 AC2 | G: calon penyedia memproses data di luar Indonesia; W: evaluasi pemilihan; T: tidak dipilih (bukti dokumen penyedia, T-26) | negatif | manual | T1 | F2 |
| TC-US-1406-3 | US-1406 AC3 | G: satu penyedia dimatikan (video, email, atau gateway; tiga test); W: fitur lain dipakai; T: fitur lain tetap berjalan (KU-08) | negatif | integrasi | T1 | F2 |
| TC-US-1406-4 | US-1406 AC1 | G: kurir terpilih; W: seluruh TC US-703 dijalankan; T: lulus | positif | integrasi | T1 | F4 |

---

## 6. Fase 3: integrasi rekanan, RME minimalis, klinik/lab, kepatuhan

Story Fase 3: US-401–US-404, US-406, US-601, US-602 (Should), US-807, US-810, US-811, US-901, US-902, US-1101–US-1104, US-1201–US-1203, US-1301–US-1304, US-1306, US-1307, US-1309, US-1310, US-1401–US-1405, US-1502, US-1503, US-1601–US-1606, US-1701 `[DOK: 07-phase-plan.md]` §4. Lebih ringkas dari Fase 1; setiap AC punya minimal satu test case. Test integrasi memakai sistem rekanan tiruan (fake sandbox) dan RME minimalis dengan kontrak yang sama; sistem rekanan nyata di SIT Tahap 9 (PT-05). Setiap endpoint menjalankan TC-SEC-1–TC-SEC-5, TC-SEC-8, TC-SEC-9; portal dan RME juga TC-SEC-6 dan TC-SEC-7.

### US-401 Cari klinik, lab, dan layanannya (BR-020, BR-021)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-401-1 | US-401 AC1 | G: rekanan aktif+terintegrasi, rekanan nonaktif, rekanan integrasi "Belum/Uji/Terganggu"; W: cari kota dan layanan; T: hanya aktif dan terintegrasi, dengan layanan, harga (rekanan + markup), jadwal berlabel zona (misalnya "10.00 WITA") | positif | integrasi | - | F3 |
| TC-US-401-2 | US-401 AC1 | G: kota atau layanan tidak ada; W: cari; T: keadaan kosong tanpa galat | negatif | E2E | - | F3 |
| TC-US-401-3 | US-401 AC2 | G: data rekanan tidak dapat diambil; W: buka rekanan; T: KU-08 (tidak tampil data lama sebagai terbaru) | negatif | integrasi | T1 | F3 |
| TC-US-401-4 | US-401 AC3 | G: izin lokasi ditolak; W: cari berdasarkan kota; T: pencarian tetap berjalan | negatif | manual | - | F3 |

### US-402 Pesan pendaftaran klinik atau lab (BR-020, BR-021, BR-024, BR-040)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-402-1 | US-402 AC1 | G: penerima terverifikasi; W: pilih layanan dan slot; T: slot ditahan selama PB-07, ringkasan harga memuat pernyataan AB-16 (tindakan tambahan dibayar di luar aplikasi) | positif | E2E | T1 | F3 |
| TC-US-402-2 | US-402 AC1 | G: harga atau ketersediaan di sistem rekanan berubah; W: pasien menekan bayar; T: nilai terbaru tampil dan harus disetujui (AB-15); slot tidak tersedia memblokir sebelum bayar | negatif | integrasi | T1 | F3 |
| TC-US-402-3 | US-402 AC2 | G: pembayaran berhasil; W: notifikasi diterima; T: "Terkonfirmasi" dan dikirim ke sistem rekanan dengan status lunas (US-1403), E-08/E-09/E-17 | positif | integrasi | T1 | F3 |
| TC-US-402-4 | US-402 AC3 | G: pengiriman ke rekanan gagal dan percobaan ulang otomatis juga gagal sampai batas waktu integrasi; W: batas habis; T: ditandai untuk tindak lanjut Layanan Pelanggan | negatif | integrasi | T1 | F3 |
| TC-US-402-5 | US-402 AC3 | G: rekanan menyatakan tidak dapat melayani; W: diterima; T: "Dibatalkan rekanan" dengan RF-05 otomatis | negatif | integrasi | T1 | F3 |
| TC-US-402-6 | US-402 AC3 | G: sampai jadwal mulai pesanan belum diterima sistem rekanan; W: jadwal mulai; T: "Dibatalkan sistem", refund RF-18 otomatis | negatif | integrasi | T1 | F3 |
| TC-US-402-7 | US-402 AC1, KU-09 | G: dua pasien memesan slot terakhir; W: bersamaan; T: satu berhasil | negatif | integrasi | T1 | F3 |

### US-403 Appointment dan kode check-in (BR-024)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-403-1 | US-403 AC1 | G: pendaftaran "Terkonfirmasi"; W: buka appointment; T: kode dan QR check-in, nama rekanan, alamat, layanan, jadwal (zona rekanan), nama penerima layanan | positif | E2E | - | F3 |
| TC-US-403-2 | US-403 AC2 | G: appointment pernah dibuka; W: perangkat offline lalu buka; T: kode dan QR tetap tampil | negatif | manual | - | F3 |
| TC-US-403-3 | US-403 AC3 | G: pendaftaran dibatalkan atau kedaluwarsa; W: buka appointment; T: kode ditandai tidak berlaku | negatif | integrasi | - | F3 |
| TC-US-403-4 | US-403 AC1 | G: pasien lain; W: membuka appointment pasien A lewat ID; T: 404 | negatif | integrasi | T1 | F3 |

### US-404 Check-in lewat sistem rekanan (BR-024, BR-070)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-404-1 | US-404 AC1 | G: rekanan mengirim status check-in untuk pesanan miliknya; W: diterima; T: status "Check-in", E-09 | positif | integrasi | T1 | F3 |
| TC-US-404-2 | US-404 AC2 | G: rekanan mengirim "selesai"; W: diterima; T: "Selesai". Bila tidak dikirim, otomatis "Selesai" pada akhir hari kalender rekanan (23.59 zona rekanan, WITA berbeda dari WIB) | batas | integrasi | T1 | F3 |
| TC-US-404-3 | US-404 AC3 | G: pesanan sudah diterima rekanan dan tidak ada check-in sampai jadwal berakhir; W: batas terlewati; T: "Pasien tidak hadir" tanpa refund (RF-03) | negatif | integrasi | T1 | F3 |
| TC-US-404-4 | US-404 AC3 | G: pesanan belum diterima rekanan; W: jadwal berakhir; T: mengikuti US-402 AC3 (RF-18), bukan "Pasien tidak hadir" | negatif | integrasi | T1 | F3 |
| TC-US-404-5 | US-404 AC4 | G: kode milik rekanan lain, tanggal lain, atau pendaftaran dibatalkan; W: dikirim; T: check-in ditolak, alasan dikembalikan ke rekanan | negatif | integrasi | T1 | F3 |
| TC-US-404-6 | US-404 AC1 | G: pesan check-in diulang dua kali; W: diterima; T: satu transisi, idempoten | negatif | integrasi | T1 | F3 |

### US-405 Antrean real-time (BR-027), Should

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-405-1 | US-405 AC1 | G: pasien check-in dan rekanan mengirim data antrean; W: data berubah; T: nomor antrean pasien dan nomor yang dilayani tampil dan diperbarui dalam ≤ 30 detik | batas | integrasi | - | F4 |
| TC-US-405-2 | US-405 AC2 | G: giliran tinggal 3 nomor (PB-10), dan 4 nomor; W: kondisi terpenuhi; T: pada 3 notifikasi in-app muncul; pada 4 tidak; aplikasi tertutup: tersedia di kotak notifikasi | batas | integrasi | - | F4 |
| TC-US-405-3 | US-405 AC3 | G: sistem rekanan tidak menyediakan data antrean; W: buka appointment; T: tampilan antrean tidak muncul | negatif | E2E | - | F4 |

### US-406 Riwayat pendaftaran (BR-026)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-406-1 | US-406 AC1 | G: pasien dengan pendaftaran; W: buka riwayat; T: rekanan, layanan, jadwal, status, total pembayaran; milik pasien lain tidak tampil | positif | E2E | - | F3 |
| TC-US-406-2 | US-406 AC2 | G: riwayat dengan beberapa status; W: filter status; T: hanya status itu | positif | integrasi | - | F3 |

### US-601 Cari dan bandingkan biaya MCU dan lab (BR-023)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-601-1 | US-601 AC1 | G: pengguna belum masuk; W: cari pemeriksaan/paket MCU dan kota; T: tiap rekanan dengan harga bayar di aplikasi (sama dengan harga pada saat memesan, AB-14), urut harga terendah; harga sama berurutan stabil | positif | integrasi | - | F3 |
| TC-US-601-2 | US-601 AC2 | G: paket MCU dengan rincian dari rekanan; W: buka paket; T: daftar pemeriksaan dan informasi persiapan | positif | E2E | - | F3 |
| TC-US-601-3 | US-601 AC3 | G: data harga tidak dapat diambil; W: buka hasil; T: KU-08 | negatif | integrasi | - | F3 |

### US-602 Lanjut mendaftar dari informasi biaya (BR-023), Should

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-602-1 | US-602 AC1 | G: pengguna masuk dan terverifikasi; W: tekan "Daftar"; T: alur US-402 terbuka dengan rekanan dan layanan terpilih | positif | E2E | - | F3 |
| TC-US-602-2 | US-602 AC2 | G: belum masuk, atau belum terverifikasi; W: tekan "Daftar"; T: diarahkan masuk/verifikasi lalu kembali ke layanan yang sama | negatif | E2E | - | F3 |

### US-807 Klaim refund (BR-045)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-807-1 | US-807 AC1 | G: kejadian RF-07 layanan terjadwal; W: klaim dikirim tepat PB-15 (2 x 24 jam) setelah jadwal berakhir, dan 1 menit sesudahnya; T: tepat batas "Diajukan" dan pasien melihat batas keputusan PB-05; lewat batas ditolak, tombol tidak ada | batas | integrasi | T1 | F3 |
| TC-US-807-2 | US-807 AC1 | G: klaim RF-10 produk tanpa foto; W: kirim; T: ditolak (foto wajib) | negatif | integrasi | T1 | F4 |
| TC-US-807-3 | US-807 AC1 | G: klaim RF-06, RF-17 home visit, RF-16 produk; W: kirim dalam dan di luar batas masing-masing; T: sesuai batas AC (PB-15, PB-04, PB-04 setelah PB-18) | batas | integrasi | T1 | F4 |
| TC-US-807-4 | US-807 AC2 | G: pesanan sudah punya klaim aktif; W: klaim lagi (dan kirim ganda paralel); T: ditolak, satu klaim | negatif | integrasi | T1 | F3 |
| TC-US-807-5 | US-807 AC3 | G: klaim "Diajukan"; W: Layanan Pelanggan (atau Super Admin untuk tereskalasi) menyetujui; T: refund nominal sesuai matriks dan kewajiban per pihak AB-34, pasien diberi tahu, E-08 | positif | integrasi | T1 | F3 |
| TC-US-807-6 | US-807 AC3 | G: peran lain (Keuangan, Konten, Petugas PDP, admin rekanan) memutuskan klaim; W: kirim; T: 403 | negatif | integrasi | T1 | F3 |
| TC-US-807-7 | US-807 AC4 | G: penolakan dengan alasan, dan tanpa alasan; W: simpan; T: dengan alasan tersimpan, pasien diberi tahu, klaim baru untuk pesanan itu ditolak; tanpa alasan tidak tersimpan | negatif | integrasi | T1 | F3 |
| TC-US-807-8 | US-807 AC5 | G: PB-05 (3 hari kerja) habis tanpa keputusan (batas 23.59 WIB hari kerja ke-3, libur tidak dihitung); W: batas terlewati; T: ditandai terlambat dan dieskalasi ke Super Admin, tidak otomatis disetujui atau ditolak | batas | integrasi | T1 | F3 |
| TC-US-807-9 | US-807 AC1 | G: membuka bukti klaim (foto); W: Layanan Pelanggan/Super Admin membuka; T: E-05 tercatat; peran lain ditolak dengan E-05 gagal | negatif | integrasi | T1 | F3 |
| TC-US-807-10 | US-807 AC1 | G: pasien lain; W: mengajukan klaim untuk pesanan pasien A; T: 404 | negatif | integrasi | T1 | F3 |

### US-810 Pengaturan biaya rekanan (BR-043)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-810-1 | US-810 AC1 | G: Keuangan; W: atur biaya (tidak ada, langganan per bulan, per pendaftaran), nilai, waktu berlaku; T: tersimpan dengan riwayat, E-14 | positif | integrasi | T1 | F3 |
| TC-US-810-2 | US-810 AC1 | G: nilai negatif atau kosong, jenis tidak dikenal, peran selain Keuangan; W: simpan; T: ditolak | negatif | integrasi | T1 | F3 |
| TC-US-810-3 | US-810 AC2 | G: jenis per pendaftaran atau langganan; W: periode bulan berakhir (batas 31 hari dan 28 hari); T: tagihan per rekanan dihitung dan tampil di laporan Keuangan dan portal rekanan; pendaftaran dibatalkan tidak dihitung bila ketentuan jelas | batas | integrasi | T1 | F3 |

### US-811 Laporan transaksi dan rekonsiliasi (BR-046)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-811-1 | US-811 AC1 | G: Keuangan memilih periode, rekanan, jenis, status; W: laporan tampil; T: tiap transaksi memuat total, bagian rekanan, markup, pajak, MDR/biaya transaksi, refund, status pembagian, dalam WIB | positif | integrasi | T1 | F3 |
| TC-US-811-2 | US-811 AC1 | G: peran lain (Konten, Auditor); W: buka laporan; T: ditolak; Layanan Pelanggan hanya pesanan terkait klaim/tiket miliknya; Super Admin baca | negatif | integrasi | T1 | F3 |
| TC-US-811-3 | US-811 AC2 | G: data penyedia berbeda dari catatan (selisih nominal, pembagian tidak terlaporkan, pembayaran ganda); W: rekonsiliasi harian berjalan; T: ketiganya ditandai untuk tindak lanjut; pembayaran ganda memicu RF-12 | negatif | integrasi | T1 | F3 |
| TC-US-811-4 | US-811 AC2 | G: rekonsiliasi berjalan dua kali pada hari yang sama; W: berjalan; T: tidak ada penandaan ganda | negatif | integrasi | T1 | F3 |
| TC-US-811-5 | US-811 AC3 | G: Keuangan mengekspor; W: file dibuat; T: CSV dan E-12 (siapa, kapan, filter, jumlah baris); sel yang diawali `=`, `+`, `-`, `@` aman dari injeksi rumus | negatif | integrasi | T1 | F3 |

### US-901 Penerimaan hasil lab (BR-025, BR-070)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-901-1 | US-901 AC1 | G: sistem lab (sendiri atau RME) mengirim hasil (PDF dan/atau nilai terstruktur) untuk pendaftaran valid; W: diterima; T: tersimpan terenkripsi, pasien diberi tahu tanpa menyebut nilai, E-06 dan E-17 | positif | integrasi | T1 | F3 |
| TC-US-901-2 | US-901 AC1 | G: PDF berkonten aktif atau melebihi 10 MB; W: diterima; T: ditolak | negatif | integrasi | T1 | F3 |
| TC-US-901-3 | US-901 AC2 | G: hasil untuk pendaftaran tidak ada atau milik rekanan lain; W: diterima; T: ditolak, galat integrasi tercatat | negatif | integrasi | T1 | F3 |
| TC-US-901-4 | US-901 AC3 | G: lab mengirim koreksi; W: diterima; T: versi baru, versi lama tetap dan ditandai sudah dikoreksi, pasien diberi tahu ada pembaruan | positif | integrasi | T1 | F3 |
| TC-US-901-5 | US-901 AC1 | G: kiriman hasil yang sama diulang; W: diterima; T: tidak ada hasil ganda (idempoten) | negatif | integrasi | T1 | F3 |

### US-902 Lihat dan unduh hasil lab (BR-025, BR-050, BR-008)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-902-1 | US-902 AC1 | G: hasil tersedia; W: pasien membuka dan mengunduh PDF; T: tampil dan terunduh, E-12 | positif | E2E | T1 | F3 |
| TC-US-902-2 | US-902 AC2 | G: pengguna lain (pasien lain, nakes, semua sub-peran admin, admin rekanan); W: membuka hasil lewat ID; T: ditolak, E-05 hasil gagal | negatif | integrasi | T1 | F3 |
| TC-US-902-3 | US-902 AC1 | G: pemilik akun membuka hasil anggota keluarga; W: buka; T: tampil dan E-05 tercatat (F4 untuk anggota keluarga) | positif | integrasi | T1 | F4 |

### US-1101 Pengelolaan konten CMS (BR-061)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-1101-1 | US-1101 AC1 | G: admin Konten; W: buat banner, halaman info, artikel, promo; T: berstatus "Draf" dan dapat dipratinjau | positif | E2E | - | F3 |
| TC-US-1101-2 | US-1101 AC1 | G: peran selain Konten; W: buat/ubah konten; T: 403 | negatif | integrasi | - | F3 |
| TC-US-1101-3 | US-1101 AC2 | G: konten dijadwalkan; W: waktu tayang tiba, lalu waktu selesai atau diarsipkan; T: "Tayang" lalu tidak tampil lagi (pada detik tepat) | batas | unit | - | F3 |
| TC-US-1101-4 | US-1101 AC3 | G: admin menghapus konten; W: simpan lalu pulihkan; T: soft delete lalu dapat dipulihkan, E-16 | positif | integrasi | T1 | F3 |
| TC-US-1101-5 | US-1101 AC4 | G: promo; W: periksa harga di aplikasi; T: promo tidak mengubah harga | negatif | integrasi | - | F3 |
| TC-US-1101-6 | US-1101 AC1 | G: konten berisi skrip atau HTML berbahaya; W: simpan dan tampil; T: disanitasi, tidak dieksekusi | negatif | integrasi | T1 | F3 |

### US-1102 Konten di aplikasi pasien (BR-061)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-1102-1 | US-1102 AC1 | G: konten "Tayang" dan "Draf"; W: pengguna belum masuk membuka beranda/artikel; T: hanya yang tayang, urut jadwal tayang | positif | E2E | - | F3 |
| TC-US-1102-2 | US-1102 AC2 | G: banner menautkan ke layanan/artikel yang kemudian dihapus; W: ketuk; T: tujuan terbuka bila ada, pesan "konten tidak tersedia" bila tidak | negatif | E2E | - | F3 |

### US-1103 Ajukan keluhan (BR-062)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-1103-1 | US-1103 AC1 | G: pasien mengisi kategori, pesanan terkait (opsional), deskripsi, foto (opsional); W: kirim; T: tiket "Baru" dan nomor tiket tampil | positif | E2E | - | F3 |
| TC-US-1103-2 | US-1103 AC1 | G: deskripsi kosong, kategori tak dikenal, pesanan milik pasien lain, foto berjenis palsu atau > 5 MB; W: kirim; T: ditolak | negatif | integrasi | T1 | F3 |
| TC-US-1103-3 | US-1103 AC2 | G: admin membalas; W: simpan; T: pasien diberi tahu dan dapat membalas di tiket yang sama | positif | integrasi | - | F3 |
| TC-US-1103-4 | US-1103 AC3 | G: pasien memilih kategori klaim refund atau hak subjek data; W: kirim; T: diarahkan ke US-807 atau US-1602 | positif | E2E | - | F3 |

### US-1104 Kelola keluhan (BR-062, BR-064, BR-065)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-1104-1 | US-1104 AC1 | G: tiket baru; W: Layanan Pelanggan membuka antrean dan ubah status; T: urut terlama; status Baru, Diproses, Menunggu pasien, Selesai; transisi di luar daftar ditolak | positif | E2E | - | F3 |
| TC-US-1104-2 | US-1104 AC2 | G: tiket terkait pesanan rekanan; W: diteruskan; T: admin rekanan itu melihat tiket di portal tanpa data klinis pasien; rekanan lain tidak | negatif | integrasi | T1 | F3 |
| TC-US-1104-3 | US-1104 AC3 | G: tiket ditutup; W: pasien membalas pada hari ke-7 dan ke-8; T: hari ke-7 dibuka kembali; hari ke-8 tidak | batas | unit | - | F3 |
| TC-US-1104-4 | US-1104 AC1 | G: peran selain Layanan Pelanggan; W: mengelola tiket; T: ditolak | negatif | integrasi | T1 | F3 |

### US-1201 Laporan transaksi dan pembagian dana rekanan (BR-046, BR-064, BR-065)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-1201-1 | US-1201 AC1 | G: admin rekanan memilih periode, jenis, status; W: laporan tampil; T: tanggal (zona rekanan), nomor, layanan/produk, nama penerima, harga rekanan, MDR, bagian rekanan, refund, kewajiban pengembalian, tagihan biaya rekanan, rekap harian dan bulanan | positif | integrasi | T1 | F3 |
| TC-US-1201-2 | US-1201 AC2 | G: laporan tampil; W: periksa respons API dan UI; T: tidak ada markup, total bayar pasien, dan data klinis | negatif | integrasi | T1 | F3 |
| TC-US-1201-3 | US-1201 AC3 | G: klinik yang juga outlet; W: buka laporan; T: terpisah atau gabungan | positif | integrasi | - | F3 |
| TC-US-1201-4 | US-1201 AC4 | G: admin rekanan A mengekspor; W: file dibuat; T: CSV hanya data A, E-12, tanpa injeksi rumus | negatif | integrasi | T1 | F3 |
| TC-US-1201-5 | US-1201 AC1 | G: admin rekanan A; W: menyelipkan `partner_id` B di query/body; T: data B tidak muncul (TC-SEC-6) | negatif | integrasi | T1 | F3 |

### US-1202 Lihat profil rekanan (BR-064, BR-065)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-1202-1 | US-1202 AC1 | G: admin rekanan; W: buka profil; T: nama, jenis, alamat, zona waktu, kontak, rekening termasking (4 digit terakhir), sumber sistem, status integrasi | positif | E2E | T1 | F3 |
| TC-US-1202-2 | US-1202 AC2 | G: admin rekanan; W: cari fitur ubah dan panggil API ubah; T: tidak ada fitur, API 403, petunjuk ke Layanan Pelanggan tampil | negatif | integrasi | T1 | F3 |
| TC-US-1202-3 | US-1202 AC3 | G: admin Ruang Sehat mengubah rekening penerima; W: simpan; T: tidak berlaku sebelum disetujui Verifikator; E-10 | negatif | integrasi | T1 | F3 |

### US-1203 Lihat keluhan terkait rekanan (BR-064, BR-065)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-1203-1 | US-1203 AC1 | G: tiket diteruskan; W: admin rekanan membuka menu keluhan; T: tiket, status, riwayat tanggapan tampil, tanpa fitur menanggapi | positif | E2E | T1 | F3 |
| TC-US-1203-2 | US-1203 AC2 | G: tiket milik rekanan lain; W: membuka lewat ID; T: ditolak (404) | negatif | integrasi | T1 | F3 |

### US-1301 Onboarding fasyankes ke RME minimalis (BR-120, BR-121)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-1301-1 | US-1301 AC1 | G: rekanan terverifikasi dengan sumber "RME minimalis" dan perjanjian tercatat; W: Operator RME mendaftarkan beserta kode registrasi dan akun Admin Fasyankes pertama; T: fasyankes aktif, koneksi integrasi siap diuji, E-10 | positif | E2E | T1 | F3 |
| TC-US-1301-2 | US-1301 AC1 | G: perjanjian belum tercatat; W: mendaftarkan; T: ditolak | negatif | integrasi | T1 | F3 |
| TC-US-1301-3 | US-1301 AC2 | G: rekanan belum terverifikasi, atau sumber "sistem sendiri"; W: mendaftarkan; T: ditolak | negatif | integrasi | T1 | F3 |
| TC-US-1301-4 | US-1301 AC3 | G: Operator RME; W: membuka data pasien/rekam medis fasyankes (API, RLS); T: ditolak di aplikasi dan database | negatif | integrasi | T1 | F3 |
| TC-US-1301-5 | US-1301 AC4 | G: fasyankes berhenti memakai RME; W: Admin Fasyankes meminta ekspor; T: rekam medis fasyankes itu dapat diekspor sebelum akses ditutup, E-12; hanya milik fasyankes itu | positif | integrasi | T1 | F3 |
| TC-US-1301-6 | US-1301 AC4 | G: Admin fasyankes B; W: meminta ekspor milik A; T: ditolak | negatif | integrasi | T1 | F3 |

### US-1302 Akun dan peran pengguna RME (BR-120)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-1302-1 | US-1302 AC1 | G: Admin Fasyankes; W: buat akun Tenaga Kesehatan Fasyankes, Petugas Lab, Admin Fasyankes; T: undangan aktivasi, MFA wajib saat aktivasi, E-10 dan E-11 | positif | E2E | T1 | F3 |
| TC-US-1302-2 | US-1302 AC2 | G: akun Tenaga Kesehatan Fasyankes tanpa identitas profesi SATUSEHAT; W: simpan; T: ditolak (rincian field menunggu Tahap 4) | negatif | integrasi | T1 | F3 |
| TC-US-1302-3 | US-1302 AC3 | G: pengguna fasyankes A; W: akses data fasyankes B; T: ditolak dan tercatat (TC-SEC-6, TC-SEC-7) | negatif | integrasi | T1 | F3 |
| TC-US-1302-4 | US-1302 AC4, KU-10 | G: Admin Fasyankes menonaktifkan akun atau mengubah peran dengan alasan; W: simpan; T: akses berakhir termasuk sesi terbuka: permintaan berikutnya (definisi KU-10) ditolak, E-10/E-11; tanpa alasan ditolak | negatif | integrasi | T1 | F3 |
| TC-US-1302-5 | US-1302 AC5 | G: tersisa satu Admin Fasyankes aktif; W: nonaktifkan atau cabut peran; T: ditolak (termasuk dua admin saling mencabut bersamaan) | negatif | integrasi | T1 | F3 |
| TC-US-1302-6 | US-1302 AC6 | G: akun Tenaga Kesehatan Fasyankes milik orang yang juga nakes Ruang Sehat di klinik itu; W: Admin Fasyankes menghubungkan; T: nakes dapat menulis rekam medis home visit yang ditugaskan kepadanya (F4); penghubungan dengan nakes klinik lain ditolak | positif | integrasi | T1 | F3 |

### US-1303 Jadwal, layanan, dan harga (BR-122)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-1303-1 | US-1303 AC1 | G: Admin Fasyankes mengisi layanan (jenis klinik, lab, MCU dengan daftar, home visit dengan area), harga rekanan, jadwal (hari, jam, kuota, libur); W: simpan; T: terkirim ke Ruang Sehat dan tampil di aplikasi pasien ≤ 15 menit, E-14 | positif | integrasi | T1 | F3 |
| TC-US-1303-2 | US-1303 AC1 | G: harga ≤ 0, kuota negatif, jam selesai sebelum jam mulai; W: simpan; T: ditolak | negatif | unit | - | F3 |
| TC-US-1303-3 | US-1303 AC2 | G: pesanan terbayar dengan harga lama; W: harga diubah; T: pesanan lama tidak berubah (AB-17) | batas | integrasi | T1 | F3 |
| TC-US-1303-4 | US-1303 AC3 | G: slot dengan pesanan terbayar; W: hapus slot; T: ditolak sampai pesanan dibatalkan rekanan (RF-05) | negatif | integrasi | T1 | F3 |
| TC-US-1303-5 | US-1303 AC4 | G: perubahan jam, kuota, durasi memengaruhi slot terbayar; W: simpan; T: ditolak sampai dibatalkan | negatif | integrasi | T1 | F3 |

### US-1304 Appointment dan check-in RME (BR-125, BR-024)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-1304-1 | US-1304 AC1 | G: pesanan terbayar dari Ruang Sehat; W: diterima lewat integrasi; T: appointment di daftar harian dengan nama pasien, layanan, jadwal, status lunas | positif | E2E | T1 | F3 |
| TC-US-1304-2 | US-1304 AC2 | G: pasien datang; W: Admin Fasyankes memindai QR atau memasukkan kode; T: check-in dan status terkirim ke Ruang Sehat (US-404), E-09 | positif | E2E | T1 | F3 |
| TC-US-1304-3 | US-1304 AC3 | G: kode fasyankes lain, tanggal lain, pesanan batal; W: dimasukkan; T: ditolak dengan alasan | negatif | integrasi | T1 | F3 |
| TC-US-1304-4 | US-1304 AC4 | G: layanan selesai; W: tandai "Selesai"; T: status terkirim ke Ruang Sehat | positif | integrasi | T1 | F3 |

### US-1305 Antrean RME (BR-125, BR-027), Should

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-1305-1 | US-1305 AC1 | G: dua pasien check-in untuk layanan sama di hari sama; W: check-in tersimpan; T: nomor berurutan per layanan per hari; hari berikutnya mulai ulang | positif | integrasi | - | F4 |
| TC-US-1305-2 | US-1305 AC2 | G: Admin Fasyankes; W: "panggil berikutnya"; T: nomor dilayani berubah dan terkirim ke Ruang Sehat; saat antrean habis tidak galat | batas | integrasi | - | F4 |
| TC-US-1305-3 | US-1305 AC3 | G: pasien dipanggil tidak ada; W: tandai dilewati lalu panggil ulang; T: nomor berikutnya dipanggil, yang dilewati dapat dipanggil ulang | positif | integrasi | - | F4 |

### US-1306 Catatan rekam medis kunjungan (BR-123)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-1306-1 | US-1306 AC1 | G: appointment sudah check-in; W: Tenaga Kesehatan Fasyankes mengisi kelompok data minimal dan memfinalkan; T: tersimpan atas nama penulis dan fasyankes, E-06 hanya referensi dan versi | positif | E2E | T1 | F3 |
| TC-US-1306-2 | US-1306 AC1 | G: appointment belum check-in; W: isi rekam medis; T: ditolak | negatif | integrasi | T1 | F3 |
| TC-US-1306-3 | US-1306 AC2 | G: kode dan terminologi; W: periksa terhadap dokumentasi SATUSEHAT; T: sesuai (daftar final menunggu Tahap 4 setelah L-05) | positif | integrasi | T1 | F3 |
| TC-US-1306-4 | US-1306 AC3 | G: rekam medis final; W: penulis mengoreksi dengan dan tanpa alasan; T: dengan alasan versi baru, versi lama tetap; tanpa alasan ditolak | negatif | integrasi | T1 | F3 |
| TC-US-1306-5 | US-1306 AC4 | G: pasien tanpa appointment Ruang Sehat; W: membuat rekam medis; T: tidak tersedia pendaftaran pasien baru | negatif | integrasi | T1 | F3 |
| TC-US-1306-6 | US-1306 AC5 | G: Admin Fasyankes, Petugas Lab, Operator RME; W: buat atau ubah rekam medis; T: ditolak (Admin Fasyankes hanya baca) | negatif | integrasi | T1 | F3 |
| TC-US-1306-7 | US-1306 AC6 | G: home visit "Tiba" atau "Selesai" dan nakes terhubung; W: isi rekam medis; T: tersimpan dengan aturan AC1–AC3; status lain atau nakes tidak terhubung ditolak | positif | integrasi | T1 | F4 |
| TC-US-1306-8 | US-1306 AC1 | G: pengguna fasyankes A; W: baca atau ubah rekam medis fasyankes B; T: ditolak, tercatat (RLS) | negatif | integrasi | T1 | F3 |
| TC-US-1306-9 | US-1306 AC1 | G: rekam medis disimpan; W: inspeksi database; T: terenkripsi, tidak ada di log dan audit | negatif | integrasi | T1 | F3 |

### US-1307 Hasil lab RME (BR-124, BR-025)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-1307-1 | US-1307 AC1 | G: pemeriksaan pasien Ruang Sehat sudah check-in; W: Petugas Lab mengisi nilai, satuan, nilai rujukan dan/atau unggah PDF lalu finalkan; T: hasil terkirim ke Ruang Sehat (US-901), E-06, E-17 | positif | E2E | T1 | F3 |
| TC-US-1307-2 | US-1307 AC1 | G: pemeriksaan belum check-in, nilai tanpa satuan, PDF palsu atau > 10 MB; W: finalkan; T: ditolak | negatif | integrasi | T1 | F3 |
| TC-US-1307-3 | US-1307 AC2 | G: hasil terkirim; W: Petugas Lab mengoreksi dengan dan tanpa alasan; T: versi baru terkirim dengan alasan; tanpa alasan ditolak | negatif | integrasi | T1 | F3 |
| TC-US-1307-4 | US-1307 AC1 | G: peran selain Petugas Lab (Admin Fasyankes, Tenaga Kesehatan Fasyankes); W: mengisi hasil; T: ditolak | negatif | integrasi | T1 | F3 |

### US-1308 Home visit RME (BR-125, BR-022)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-1308-1 | US-1308 AC1 | G: pesanan home visit terbayar diterima; W: Admin Fasyankes membuka daftar; T: jadwal, alamat, layanan, nama pasien | positif | E2E | T1 | F4 |
| TC-US-1308-2 | US-1308 AC2 | G: Admin Fasyankes memilih nakes aktif klinik itu; W: simpan; T: penugasan terkirim (US-502); nakes klinik lain atau nonaktif tidak ada di daftar | negatif | integrasi | T1 | F4 |
| TC-US-1308-3 | US-1308 AC3 | G: nakes memperbarui status di aplikasi Ruang Sehat; W: diterima; T: status tampil di RME | positif | integrasi | - | F4 |

### US-1309 Pengiriman data rekam medis ke SATUSEHAT (BR-123)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-1309-1 | US-1309 AC1 | G: rekam medis difinalkan dan kredensial diisi; W: kirim ke SATUSEHAT (sandbox, PT-32); T: "Antre" lalu "Terkirim", atau "Gagal" dengan alasan; E-17 jenis data dan tujuan tanpa isi | positif | integrasi | T1 | F3 |
| TC-US-1309-2 | US-1309 AC1 | G: kredensial belum diisi; W: finalkan lalu isi kredensial; T: "Menunggu kredensial", otomatis masuk antrean setelah diisi | batas | integrasi | T1 | F3 |
| TC-US-1309-3 | US-1309 AC2 | G: SATUSEHAT gangguan; W: percobaan ulang otomatis; T: berjalan sampai 24 jam sejak percobaan pertama lalu "Gagal" dan Admin Fasyankes diberi tahu (pada 23:59 masih mencoba, setelah 24 jam berhenti) | batas | integrasi | T1 | F3 |
| TC-US-1309-4 | US-1309 AC2 | G: kegagalan data klinis tidak valid; W: ditolak SATUSEHAT; T: dikembalikan ke penulis untuk koreksi; kegagalan identitas fasyankes ke Admin Fasyankes | negatif | integrasi | T1 | F3 |
| TC-US-1309-5 | US-1309 AC2 | G: pengiriman diulang karena timeout; W: percobaan ulang; T: tidak menggandakan data di SATUSEHAT (idempoten) | negatif | integrasi | T1 | F3 |
| TC-US-1309-6 | US-1309 AC4 | G: rekam medis "Terkirim" dikoreksi; W: versi baru difinalkan; T: dikirim sebagai pembaruan atas data yang sama dengan status pengiriman sendiri | positif | integrasi | T1 | F3 |
| TC-US-1309-7 | US-1309 AC5 | G: status "Gagal" dan penyebab diperbaiki; W: Admin Fasyankes menekan kirim ulang; T: kembali "Antre"; peran lain tidak dapat | negatif | integrasi | T1 | F3 |
| TC-US-1309-8 | US-1309 AC1 | G: kredensial SATUSEHAT fasyankes; W: periksa log, audit, respons; T: tidak ada kredensial atau token di log/audit; terenkripsi di penyimpanan | negatif | integrasi | T1 | F3 |
| TC-US-1309-9 | US-1309 AC3 | G: RME belum terdaftar sebagai mitra SATUSEHAT (PT-32); W: kirim; T: bukan pengiriman nyata; status "Menunggu kredensial" (KEPUTUSAN-16) | negatif | integrasi | T1 | F3 |

### US-1310 Riwayat rekam medis pasien di fasyankes (BR-123, BR-126)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-1310-1 | US-1310 AC1 | G: pasien punya kunjungan sebelumnya di fasyankes sama; W: Tenaga Kesehatan Fasyankes membuka pasien; T: riwayat tampil, E-05 tercatat | positif | E2E | T1 | F3 |
| TC-US-1310-2 | US-1310 AC1 | G: kunjungan di fasyankes lain; W: membuka riwayat; T: tidak tampil (RLS) | negatif | integrasi | T1 | F3 |
| TC-US-1310-3 | US-1310 AC2 | G: isi chat, catatan konsultasi online, hasil dari fasyankes lain; W: Tenaga Kesehatan Fasyankes mencari; T: tidak tersedia | negatif | integrasi | T1 | F3 |
| TC-US-1310-4 | US-1310 AC3 | G: Admin Fasyankes membuka log akses; W: tampil; T: siapa, kapan, pasien mana untuk fasyankesnya saja; fasyankes lain tidak terlihat; semua akses di TC-US-1310-1 muncul | positif | integrasi | T1 | F3 |

### US-1401 Spesifikasi kontrak integrasi dan lingkungan uji (BR-070, BR-121)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-1401-1 | US-1401 AC1 | G: kontrak integrasi diterbitkan; W: bandingkan dengan tabel EP-14; T: setiap data punya format, arah, aturan validasi, kode galat; OpenAPI hasil generate identik dengan `openapi/` | positif | integrasi | T1 | F3 |
| TC-US-1401-2 | US-1401 AC2 | G: rekanan lulus semua skenario wajib di sandbox; W: admin mengubah status ke "Aktif"; T: berhasil; bila ada skenario gagal, "Aktif" ditolak | negatif | integrasi | T1 | F3 |
| TC-US-1401-3 | US-1401 AC3 | G: RME minimalis; W: diuji dengan skenario yang sama; T: lulus tanpa jalur khusus | positif | integrasi | T1 | F3 |

### US-1402 Sinkronisasi data rekanan (BR-070, BR-033)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-1402-1 | US-1402 AC1 | G: rekanan mengubah layanan, harga, jadwal; W: dikirim atau diambil; T: tampil di aplikasi ≤ 15 menit (14:59 lulus, 15:01 gagal) | batas | integrasi | T1 | F3 |
| TC-US-1402-2 | US-1402 AC2 | G: pesan dengan satu bagian tidak valid di antara bagian valid; W: diterima; T: seluruh pesan ditolak, tidak ada yang diterapkan sebagian, galat tercatat | negatif | integrasi | T1 | F3 |
| TC-US-1402-3 | US-1402 AC3 | G: kredensial salah atau klien tidak dikenal; W: kirim; T: ditolak, tercatat sebagai kejadian keamanan | negatif | integrasi | T1 | F3 |
| TC-US-1402-4 | US-1402 AC3 | G: tanda tangan webhook salah, stempel waktu lebih dari 5 menit, kunci idempotensi sama dengan isi berbeda; W: kirim; T: ditolak (tanda tangan/stempel), kunci sama isi beda mendapat `4090002` | negatif | integrasi | T1 | F3 |
| TC-US-1402-5 | US-1402 AC2 | G: `partner_id` di isi pesan berbeda dari klien terotentikasi; W: kirim; T: `partner_id` di isi diabaikan, data milik klien terotentikasi yang diproses | negatif | integrasi | T1 | F3 |
| TC-US-1402-6 | US-1402 AC2 | G: URL callback ke alamat privat, loopback, metadata, atau berpindah lewat redirect; W: didaftarkan atau dipanggil; T: ditolak, redirect tidak diikuti | negatif | integrasi | T1 | F3 |
| TC-US-1402-7 | US-1402 AC1 | G: pesan yang sama dikirim ulang; W: diterima; T: satu perubahan (idempoten) | negatif | integrasi | T1 | F3 |

### US-1403 Pengiriman pesanan dan pembatalan ke rekanan (BR-070, BR-045)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-1403-1 | US-1403 AC1 | G: pesanan dibayar; W: dikirim ke rekanan; T: memuat status lunas, layanan/produk, jadwal/alamat, data penerima minimal (tanpa data berlebih, E-17) | positif | integrasi | T1 | F3 |
| TC-US-1403-2 | US-1403 AC2 | G: pengiriman gagal lalu dicoba ulang; W: rekanan menerima; T: tidak ada pesanan ganda (idempoten) | negatif | integrasi | T1 | F3 |
| TC-US-1403-3 | US-1403 AC3 | G: pesanan dibatalkan pasien atau sistem; W: dikirim; T: rekanan menerima dan slot/stok dapat dilepas | positif | integrasi | T1 | F3 |
| TC-US-1403-4 | US-1403 AC1 | G: rekanan A meminta pesanan milik B; W: panggil; T: tidak terlihat | negatif | integrasi | T1 | F3 |

### US-1404 Penerimaan status dari rekanan (BR-070)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-1404-1 | US-1404 AC1 | G: rekanan mengirim check-in, selesai, antrean, penugasan, status produk, hasil lab; W: diterima; T: status diperbarui sesuai §3.6, E-09 | positif | integrasi | T1 | F3 |
| TC-US-1404-2 | US-1404 AC2 | G: status tidak sesuai urutan (selesai untuk pesanan dibatalkan; check-in setelah Dibatalkan); W: diterima; T: ditolak, galat ke rekanan, E-09 hasil gagal, satu transisi saja yang berlaku | negatif | integrasi | T1 | F3 |
| TC-US-1404-3 | US-1404 AC3 | G: rekanan membatalkan pesanan terbayar (layanan terjadwal, produk); W: diterima; T: "Dibatalkan rekanan" + RF-05, atau "Ditolak outlet" + RF-08 (produk F4) | positif | integrasi | T1 | F3 |
| TC-US-1404-4 | US-1404 AC3 | G: pembatalan dari status tidak diizinkan (setelah Check-in, setelah Dikirim); W: diterima; T: ditolak (AC2) | negatif | integrasi | T1 | F3 |
| TC-US-1404-5 | US-1404 AC1 | G: dua status dari rekanan datang bersamaan; W: diterima; T: yang tersimpan lebih dulu berlaku, yang lain ditolak dan tercatat (AB-36) | negatif | integrasi | T1 | F3 |

### US-1405 Pemantauan dan status integrasi per rekanan (BR-070, BR-090)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-1405-1 | US-1405 AC1 | G: admin membuka pemantauan; W: tampil; T: status (Belum, Uji, Aktif, Terganggu, Nonaktif), waktu sinkron terakhir berhasil, jumlah galat 24 jam terakhir; perubahan status E-10 | positif | E2E | - | F3 |
| TC-US-1405-2 | US-1405 AC2 | G: integrasi gagal terus 29 menit dan 30 menit (definisi "gagal terus": T-32); W: batas terlewati; T: pada 30 menit "Terganggu" dan peringatan ke admin; pada 29 menit belum | batas | integrasi | - | F3 |
| TC-US-1405-3 | US-1405 AC3 | G: rekanan berstatus "Aktif", termasuk RME; W: KPI-03/KPI-04 dihitung; T: terhitung terintegrasi; "Terganggu"/"Nonaktif" tidak terhitung | positif | integrasi | - | F3 |
| TC-US-1405-4 | US-1405 AC1 | G: peran tanpa hak (Konten, Petugas PDP); W: ubah status integrasi; T: ditolak | negatif | integrasi | T1 | F3 |

### US-1502 Halaman Audit Log (BR-080)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-1502-1 | US-1502 AC1 | G: Auditor atau Super Admin; W: filter rentang waktu, aktor, peran, jenis event, jenis dan ID objek, subjek data, hasil, IP; T: hasil urut terbaru, berhalaman, filter kombinasi benar | positif | E2E | T1 | F3 |
| TC-US-1502-2 | US-1502 AC2 | G: Auditor membuka satu event; W: detail tampil; T: semua field §7.2, nilai termasking sesuai §7.3 | positif | integrasi | T1 | F3 |
| TC-US-1502-3 | US-1502 AC3 | G: peran selain Auditor dan Super Admin; W: membuka Audit Log; T: ditolak (TC-AUD-3) | negatif | integrasi | T1 | F3 |
| TC-US-1502-4 | US-1502 AC1 | G: filter dengan input berisi karakter khusus/injeksi; W: cari; T: tidak ada injeksi, hasil aman | negatif | integrasi | T1 | F3 |
| TC-US-1502-5 | US-1502 AC1 | G: halaman terakhir dan di luar rentang; W: paginasi; T: batas halaman benar tanpa galat | batas | integrasi | - | F3 |

### US-1503 Export audit log (BR-080)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-1503-1 | US-1503 AC1 | G: Auditor menerapkan filter; W: ekspor; T: CSV hasil filter dengan masking §7.3; E-12 (siapa, kapan, filter, jumlah baris) | positif | integrasi | T1 | F3 |
| TC-US-1503-2 | US-1503 AC2 | G: Super Admin; W: cari tombol ekspor dan panggil API ekspor; T: tidak ada tombol, API 403 | negatif | integrasi | T1 | F3 |
| TC-US-1503-3 | US-1503 AC1 | G: ekspor yang mengandung sel diawali `=`/`+`/`-`/`@`; W: file dibuka; T: aman dari injeksi rumus | negatif | integrasi | T1 | F3 |

### US-1601 Lihat dan tarik persetujuan (BR-103, BR-106)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-1601-1 | US-1601 AC1 | G: pasien; W: buka pengaturan privasi; T: daftar persetujuan (butir, versi, tanggal) tampil | positif | E2E | T1 | F3 |
| TC-US-1601-2 | US-1601 AC2 | G: pasien menarik butir utama dan mengonfirmasi; W: simpan; T: "Persetujuan ditarik": hanya riwayat/status, hak subjek data, beri persetujuan kembali; pemesanan baru ditolak; data tidak lagi ke rekanan/kanal notifikasi (paling lambat 3 x 24 jam), E-13 | negatif | integrasi | T1 | F3 |
| TC-US-1601-3 | US-1601 AC2 | G: status "Persetujuan ditarik"; W: pemesanan baru lewat API, dan akses layanan lain; T: ditolak; riwayat dan hak subjek data tetap | negatif | integrasi | T1 | F3 |
| TC-US-1601-4 | US-1601 AC3 | G: pasien punya pesanan terbayar; W: menarik butir utama; T: diberi tahu sebelum konfirmasi: terjadwal dibatalkan (RF-01 atau RF-02), produk tetap diselesaikan | positif | integrasi | T1 | F3 |
| TC-US-1601-5 | US-1601 AC4 | G: pasien menarik butir tambahan; W: konfirmasi; T: hanya tujuan butir itu berhenti, layanan lain tetap | positif | integrasi | T1 | F3 |
| TC-US-1601-6 | US-1601 AC2 | G: penarikan tanpa konfirmasi, atau oleh pasien lain; W: kirim; T: ditolak | negatif | integrasi | T1 | F3 |

### US-1602 Ajukan permintaan hak subjek data (BR-106)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-1602-1 | US-1602 AC1 | G: pasien memilih jenis (informasi, akses dan salinan, perbaikan, penghapusan, penundaan/pembatasan, keberatan, portabilitas) untuk diri sendiri atau anggota keluarga; W: kirim setelah autentikasi ulang password; T: "Diterima" dan batas waktu tampil, E-13 | positif | E2E | T1 | F3 |
| TC-US-1602-2 | US-1602 AC1 | G: pengajuan tanpa autentikasi ulang, atau password salah; W: kirim; T: ditolak | negatif | integrasi | T1 | F3 |
| TC-US-1602-3 | US-1602 AC1 | G: pengajuan untuk anggota keluarga milik akun lain; W: kirim; T: ditolak | negatif | integrasi | T1 | F3 |
| TC-US-1602-4 | US-1602 AC2 | G: permintaan diproses; W: pasien membuka status; T: tahap penanganan tampil | positif | E2E | T1 | F3 |

### US-1603 Proses permintaan hak subjek data (BR-106, BR-111)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-1603-1 | US-1603 AC1, AC4 | G: permintaan masuk; W: Petugas PDP membuka antrean; T: tiap permintaan menampilkan sisa waktu menuju 3 x 24 jam | positif | E2E | T1 | F3 |
| TC-US-1603-2 | US-1603 AC2 | G: sisa waktu 12 jam 1 menit dan 12 jam; W: batas diperiksa; T: pada 12 jam ditandai mendesak dan Super Admin diberi tahu; sebelum itu tidak | batas | unit | T1 | F3 |
| TC-US-1603-3 | US-1603 AC3 | G: Petugas PDP menyelesaikan atau menolak dengan tanggapan dan alasan; W: simpan; T: pasien menerima email dan in-app; tanpa alasan penolakan ditolak | negatif | integrasi | T1 | F3 |
| TC-US-1603-4 | US-1603 AC5 | G: permintaan perbaikan disetujui; W: Petugas PDP mengubah nama/tanggal lahir/NIK; T: data berubah lewat backoffice, status identitas kembali "Menunggu verifikasi", E-10 dan E-13 | positif | integrasi | T1 | F3 |
| TC-US-1603-5 | US-1603 AC6 | G: penundaan/pembatasan atau keberatan disetujui; W: diterapkan lalu dicabut; T: "Pemrosesan dibatasi" dengan efek seperti US-1601 AC2, pencabutan E-13 | positif | integrasi | T1 | F3 |
| TC-US-1603-6 | US-1603 AC7 | G: permintaan informasi; W: diselesaikan; T: tanggapan tertulis, waktu, isi tersimpan dan terkirim ke pasien | positif | integrasi | T1 | F3 |
| TC-US-1603-7 | US-1603 AC1 | G: peran selain Petugas PDP (Super Admin hanya baca); W: memproses; T: ditolak | negatif | integrasi | T1 | F3 |
| TC-US-1603-8 | US-1603 AC1 | G: Petugas PDP membuka data pemohon; W: lihat NIK dan konten klinis; T: NIK termasking; konten klinis tidak dapat dibuka (§3.1) | negatif | integrasi | T1 | F3 |

### US-1604 Salinan data pribadi (BR-106)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-1604-1 | US-1604 AC1 | G: permintaan akses/portabilitas disetujui; W: salinan dibuat; T: memuat profil, identitas, anggota keluarga, transaksi, isi chat, catatan, resep, hasil lab, rekam jejak pemrosesan, format dapat dibaca mesin dan orang, ≤ 3 x 24 jam; dibuat otomatis tanpa dibuka Petugas PDP | positif | integrasi | T1 | F3 |
| TC-US-1604-2 | US-1604 AC1 | G: salinan dibuat; W: periksa isinya; T: hanya data subjek itu, tidak ada data pasien lain | negatif | integrasi | T1 | F3 |
| TC-US-1604-3 | US-1604 AC2 | G: salinan siap; W: pasien mengunduh setelah autentikasi ulang; T: unduhan tercatat (E-12); tautan berlaku 7 hari (hari ke-7 berlaku, hari ke-8 tidak); tanpa autentikasi ulang atau oleh pengguna lain ditolak | batas | integrasi | T1 | F3 |

### US-1605 Penghapusan dan anonimisasi (BR-106, BR-112), terblokir PT-24/L-06

TC di bawah berstatus "Terblokir" dan ditulis agar dapat dijalankan segera setelah aturan retensi diketahui (T-25).

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-1605-1 | US-1605 AC1 | G: permintaan penghapusan; W: Petugas PDP memeriksa kewajiban retensi per jenis; T: data tanpa kewajiban dihapus/dianonimkan; transaksi tetap utuh dengan data pribadi dianonimkan (menunggu: PT-24, L-06) | positif | integrasi | T1 | F3 (terblokir) |
| TC-US-1605-2 | US-1605 AC2 | G: ada data wajib simpan; W: tanggapan dikirim; T: pasien diberi tahu data yang disimpan, alasan, sampai kapan (menunggu: PT-24) | positif | integrasi | T1 | F3 (terblokir) |
| TC-US-1605-3 | US-1605 AC3 | G: anonimisasi; W: periksa `UPDATE` oleh role pekerja privasi dan role aplikasi; T: hanya kolom bertanda "Anonimkan" yang berubah; role aplikasi tidak dapat menghapus; DEK dimusnahkan tidak dapat dihidupkan kembali; E-16 (menunggu: PT-24) | negatif | integrasi | T1 | F3 (terblokir) |

### US-1606 Retensi dan pemusnahan otomatis (BR-112), terblokir PT-24

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-1606-1 | US-1606 AC1 | G: masa retensi per jenis data diatur; W: masa habis dan proses terjadwal berjalan; T: pemrosesan berakhir/dihapus/dimusnahkan sesuai aturan jenis, laporan hasil tersimpan, E-16 (menunggu: PT-24) | positif | integrasi | T1 | F3 (terblokir) |
| TC-US-1606-2 | US-1606 AC1 | G: kebijakan retensi kosong atau data belum habis masa; W: proses berjalan; T: tidak ada data diproses (ADR-013 butir 6); data masih wajib simpan tidak disentuh | negatif | integrasi | T1 | F3 |
| TC-US-1606-3 | US-1606 AC2 | G: audit trail dalam masa retensi; W: proses berjalan; T: audit tidak dihapus sebelum masa habis | negatif | integrasi | T1 | F3 |

### US-1701 Dasbor KPI (BR-090)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-1701-1 | US-1701 AC1 | G: data dengan konsultasi "Selesai", klinik/lab "Check-in/Selesai", home visit "Selesai", produk "Diterima", dan status lain, termasuk pesanan pada 00.00 WIB hari pertama dan 23.59 hari terakhir bulan; W: pilih bulan; T: KPI-01 = pemilik akun unik dengan minimal satu layanan berstatus akhir berhasil, satu orang dengan banyak layanan dihitung satu, batas bulan WIB benar | batas | unit | - | F3 |
| TC-US-1701-2 | US-1701 AC2 | G: pemilik akun masuk sekali dan berkali-kali; W: dasbor; T: pemilik akun unik yang membuka aplikasi dalam keadaan masuk minimal sekali | positif | integrasi | - | F3 |
| TC-US-1701-3 | US-1701 AC3 | G: nakes siap melayani (AB-54) dan yang tidak; W: KPI-02; T: dokter dan psikolog siap melayani pada akhir bulan dan saat ini | positif | integrasi | - | F3 |
| TC-US-1701-4 | US-1701 AC4 | G: rekanan "Aktif" termasuk RME dan non-aktif; W: KPI-03 dan KPI-04; T: hanya "Aktif" terhitung | positif | integrasi | - | F3 |
| TC-US-1701-5 | US-1701 AC5 | G: Super Admin mengekspor; W: file dibuat; T: CSV agregat tanpa data pribadi, E-12; peran lain ditolak | negatif | integrasi | T1 | F3 |

---

## 7. Fase 4: layanan yang dilepas pertama (bersyarat)

Story Fase 4: US-108, US-109 (Should; di Opsi B dipindah ke Fase 3), US-405, US-1305 (Should), US-501–US-504, US-701–US-706, US-1308, US-1406 (kurir). Test case US-405, US-1305, US-1308, dan bagian F4 dari story lain ada di §4–§6 dengan fase F4. Dikerjakan bila gerbang re-plan memilih Opsi A (`[DOK: 07-phase-plan.md]` §4, §6.6).

### US-108 Tambah anggota keluarga (BR-002, BR-008, BR-104), Should

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-108-1 | US-108 AC1 | G: pasien menambah anggota dewasa dengan nama, hubungan, tanggal lahir, jenis kelamin, NIK, foto KTP, pernyataan persetujuan; W: kirim; T: tersimpan "Menunggu verifikasi", E-15 | positif | E2E | T1 | F4 |
| TC-US-108-2 | US-108 AC2 | G: anak dengan data, NIK, foto KK, persetujuan orang tua/wali; W: kirim; T: "Menunggu verifikasi", persetujuan tersimpan sebagai bukti (BR-104) | positif | integrasi | T1 | F4 |
| TC-US-108-3 | US-108 AC3 | G: pernyataan atau persetujuan tidak dipilih; W: kirim; T: ditolak, tidak ada data | negatif | integrasi | T1 | F4 |
| TC-US-108-4 | US-108 AC4 | G: NIK sama dengan NIK pemilik akun; W: kirim; T: ditolak | negatif | integrasi | T1 | F4 |
| TC-US-108-5 | US-108 AC5 | G: NIK sama sudah tercatat sebagai anggota di akun yang sama; W: kirim (dan kirim ganda paralel); T: ditolak sebagai duplikat, satu entri | negatif | integrasi | T1 | F4 |
| TC-US-108-6 | US-108 AC6 | G: tanggal lahir tepat di batas usia dewasa/anak (parameter, nilai menunggu L-08), sehari sebelum dan sesudahnya; W: formulir tampil; T: alur KTP atau KK ditentukan sistem, bukan dipilih pasien | batas | unit | - | F4 |
| TC-US-108-7 | US-108 AC1 | G: NIK anggota keluarga yang juga tercatat di akun lain; W: kirim; T: diterima (boleh di lebih dari satu akun, UP-40) | batas | integrasi | T1 | F4 |

### US-109 Ubah dan hapus anggota keluarga (BR-008, BR-106), Should

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-109-1 | US-109 AC1 | G: anggota "Disetujui"; W: ubah nama, tanggal lahir, NIK, atau dokumen (satu test per field); T: kembali "Menunggu verifikasi", E-15 | positif | integrasi | T1 | F4 |
| TC-US-109-2 | US-109 AC2 | G: anggota punya pesanan terjadwal terbayar belum selesai; W: hapus; T: ditolak sampai selesai atau dibatalkan | negatif | integrasi | T1 | F4 |
| TC-US-109-3 | US-109 AC3 | G: tidak ada pesanan aktif; W: hapus; T: soft delete (tidak tampil, tidak dapat dipilih), transaksi dan rekam medis tetap tersimpan, E-16 | positif | integrasi | T1 | F4 |
| TC-US-109-4 | US-109 AC4 | G: NIK anggota yang pernah dihapus; W: ditambahkan kembali; T: entri baru "Menunggu verifikasi", riwayat entri lama tetap terlihat (US-310 AC3) | positif | integrasi | T1 | F4 |

### US-501 Cari dan pesan home visit (BR-022)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-501-1 | US-501 AC1 | G: alamat kunjungan di dalam area klinik A dan di luar klinik B; W: cari home visit; T: hanya klinik yang areanya mencakup alamat, dengan harga dan jadwal (zona rekanan) | positif | integrasi | - | F4 |
| TC-US-501-2 | US-501 AC2 | G: penerima terverifikasi; W: pilih layanan dan jadwal, bayar; T: "Terkonfirmasi" dan dikirim ke sistem klinik (US-1403), AB-16 tampil, E-08/E-09 | positif | E2E | T1 | F4 |
| TC-US-501-3 | US-501 AC3 | G: alamat di luar semua area; W: cari; T: pesan home visit belum tersedia | negatif | E2E | - | F4 |
| TC-US-501-4 | US-501 AC2 | G: alamat milik pasien lain; W: pilih alamat itu lewat ID; T: ditolak | negatif | integrasi | T1 | F4 |

### US-502 Penugasan tenaga kesehatan (BR-022, BR-070)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-502-1 | US-502 AC1 | G: klinik menugaskan nakes aktifnya; W: diterima; T: "Ditugaskan", nakes menerima notifikasi, pasien melihat nama dan jenis, E-09 | positif | integrasi | T1 | F4 |
| TC-US-502-2 | US-502 AC2 | G: nakes bukan nakes aktif klinik itu (klinik lain, nonaktif, SIP berakhir); W: penugasan diterima; T: ditolak dengan alasan ke sistem klinik | negatif | integrasi | T1 | F4 |
| TC-US-502-3 | US-502 AC3 | G: "Ditugaskan"; W: klinik mengirim nakes lain; T: diganti, nakes lama, baru, dan pasien diberi tahu | positif | integrasi | T1 | F4 |

### US-503 Status kunjungan oleh tenaga kesehatan (BR-022)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-503-1 | US-503 AC1 | G: nakes punya tugas; W: buka aplikasi pada hari kunjungan dan hari lain; T: jadwal, alamat (tautan peta), nama pasien, layanan, nomor HP; nomor HP hanya hari kunjungan sampai selesai | batas | integrasi | T1 | F4 |
| TC-US-503-2 | US-503 AC2 | G: "Ditugaskan"; W: tandai Berangkat, Tiba, Selesai berurutan, dan coba melompati (Ditugaskan ke Tiba); T: berurutan tersimpan, pasien diberi tahu saat Berangkat dan Tiba; lompat ditolak | negatif | integrasi | T1 | F4 |
| TC-US-503-3 | US-503 AC3 | G: status "Tiba"; W: tandai "Pasien tidak ada di lokasi" dengan keterangan; T: "Pasien tidak hadir" tanpa refund (RF-03), pasien dapat mengajukan keberatan paling lambat PB-15 (RF-17) | positif | integrasi | T1 | F4 |
| TC-US-503-4 | US-503 AC3 | G: status selain "Tiba"; W: tandai pasien tidak ada; T: ditolak | negatif | integrasi | T1 | F4 |
| TC-US-503-5 | US-503 AC1 | G: nakes lain; W: membuka tugas nakes A lewat ID; T: 404 | negatif | integrasi | T1 | F4 |

### US-504 Pantau kunjungan dan laporkan tenaga kesehatan tidak datang (BR-022, BR-045)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-504-1 | US-504 AC1 | G: pesanan aktif; W: pasien membuka; T: linimasa status dan nama nakes | positif | E2E | - | F4 |
| TC-US-504-2 | US-504 AC2 | G: status Terkonfirmasi, Ditugaskan, Berangkat; W: PB-03 habis (14:59 dan 15:01) dan sebelum/sesudah PB-15; T: tombol aktif hanya setelah PB-03 habis dan sebelum PB-15 habis; laporan menjadi klaim RF-06 | batas | integrasi | T1 | F4 |
| TC-US-504-3 | US-504 AC3 | G: status "Tiba"; W: pasien membuka; T: tombol laporan tidak tersedia; keluhan lain lewat US-1103 | negatif | E2E | - | F4 |

### US-701 Jelajah dan cari katalog (BR-030, BR-033)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-701-1 | US-701 AC1 | G: pengguna belum masuk; W: pilih kategori, outlet, atau cari nama; T: produk dari outlet aktif dan terintegrasi dengan harga (rekanan + markup) dan status stok | positif | integrasi | - | F4 |
| TC-US-701-2 | US-701 AC2 | G: stok 0 dan 1; W: tampil; T: 0 ditandai "habis" dan tidak dapat masuk keranjang; 1 dapat | batas | integrasi | - | F4 |
| TC-US-701-3 | US-701 AC3 | G: katalog outlet tidak dapat diambil; W: buka outlet; T: KU-08 | negatif | integrasi | T1 | F4 |

### US-702 Detail produk (BR-030, BR-031, BR-032, BR-116)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-702-1 | US-702 AC1 | G: pasien membuka produk; W: detail tampil; T: nama, foto, deskripsi, kategori, harga, stok, nama dan jenis outlet | positif | E2E | - | F4 |
| TC-US-702-2 | US-702 AC2 | G: produk obat bebas dari outlet bukan apotek/klinik berizin; W: katalog diterima; T: tidak ditampilkan dan dilaporkan ke admin | negatif | integrasi | T1 | F4 |
| TC-US-702-3 | US-702 AC3 | G: rekanan menandai produk sebagai obat resep; W: katalog diterima; T: ditolak dan tidak pernah ditampilkan | negatif | integrasi | T1 | F4 |

### US-703 Keranjang dan checkout (BR-034, BR-040)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-703-1 | US-703 AC1 | G: produk dari dua outlet; W: tambah ke keranjang; T: keranjang dipisah per outlet, satu checkout satu outlet | positif | integrasi | - | F4 |
| TC-US-703-2 | US-703 AC2 | G: stok 5; W: tambah 5, 6; T: 5 diterima, 6 dibatasi 5 dengan pesan | batas | integrasi | - | F4 |
| TC-US-703-3 | US-703 AC3 | G: alamat dilayani kurir, dan tidak; W: lanjut; T: pilihan kurir dan ongkos kirim dari kurir terintegrasi; tidak dilayani: tidak dapat lanjut, pesan tampil | negatif | integrasi | T1 | F4 |
| TC-US-703-4 | US-703 AC4 | G: harga atau stok di outlet berbeda; W: tekan bayar; T: nilai terbaru dan wajib disetujui (AB-15) | negatif | integrasi | T1 | F4 |
| TC-US-703-5 | US-703 AC5 | G: pembayaran berhasil; W: notifikasi; T: "Dibayar" dan dikirim ke sistem outlet (US-1403) | positif | integrasi | T1 | F4 |
| TC-US-703-6 | US-703 AC1, KU-09 | G: dua pasien membeli stok terakhir; W: bersamaan; T: satu berhasil | negatif | integrasi | T1 | F4 |
| TC-US-703-7 | US-703 AC3 | G: total = jumlah harga produk + pajak + ongkos kirim (AB-11); W: hitung; T: sesuai; ongkos kirim tidak dikenai markup (AB-11a) | batas | unit | T1 | F4 |

### US-704 Status dan pelacakan pesanan (BR-033, BR-034)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-704-1 | US-704 AC1 | G: outlet menerima; W: status diterima; T: "Diproses outlet" | positif | integrasi | T1 | F4 |
| TC-US-704-2 | US-704 AC2 | G: outlet menolak, tidak menerima sampai PB-06 (24 jam, 23:59 dan 24:01), atau membatalkan sebelum diserahkan ke kurir; W: kondisi terjadi; T: "Ditolak outlet", refund RF-08 otomatis, pasien diberi tahu | batas | integrasi | T1 | F4 |
| TC-US-704-3 | US-704 AC3 | G: barang diserahkan ke kurir; W: resi dan status diterima; T: "Dikirim" dan riwayat pelacakan | positif | integrasi | T1 | F4 |
| TC-US-704-4 | US-704 AC4 | G: kurir mengirim gagal kirim; W: diterima; T: "Gagal kirim", refund RF-09 otomatis, pasien diberi tahu | negatif | integrasi | T1 | F4 |
| TC-US-704-5 | US-704 AC5 | G: "Diproses outlet" dan PB-17 (2 x 24 jam) habis tanpa serah ke kurir; W: batas habis; T: "Tidak diserahkan", RF-15 otomatis | batas | integrasi | T1 | F4 |
| TC-US-704-6 | US-704 AC6 | G: "Dikirim" dan PB-18 (7 hari) habis tanpa terkirim atau gagal; W: pasien membuka; T: tombol "Barang tidak sampai" aktif (sebelum 7 hari belum), klaim RF-16 tanpa wajib foto | batas | integrasi | T1 | F4 |
| TC-US-704-7 | US-704 AC1 | G: status kurir/outlet yang tidak sah dari status saat ini; W: diterima; T: ditolak dan tercatat (AB-36) | negatif | integrasi | T1 | F4 |

### US-705 Konfirmasi terima dan klaim barang (BR-045)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-705-1 | US-705 AC1 | G: kurir terkirim dan pasien menekan diterima hampir bersamaan; W: diproses; T: "Diterima" oleh kejadian pertama saja | negatif | integrasi | T1 | F4 |
| TC-US-705-2 | US-705 AC2 | G: "Diterima" dan PB-04 belum habis; W: klaim rusak/tidak sesuai/kurang dengan satu foto, dan tanpa foto; T: dengan foto "Klaim" (US-807), tanpa foto ditolak; "tidak sampai" tanpa foto boleh | negatif | integrasi | T1 | F4 |
| TC-US-705-3 | US-705 AC3 | G: PB-04 (2 x 24 jam) habis; W: cari tombol klaim; T: tombol tidak ada, status "Selesai"; klaim tepat sebelum batas diterima | batas | integrasi | T1 | F4 |
| TC-US-705-4 | US-705 AC4 | G: pesanan produk dibayar; W: cari fitur batal dan panggil API batal; T: tidak tersedia (RF-11) | negatif | integrasi | T1 | F4 |

### US-706 Riwayat pesanan (BR-035)

| ID | AC | Skenario | Tipe | Level | T1 | Fase |
| --- | --- | --- | --- | --- | --- | --- |
| TC-US-706-1 | US-706 AC1 | G: pasien dengan pesanan; W: buka riwayat; T: tanggal, outlet, produk, total, status; pesanan pasien lain tidak tampil | positif | E2E | - | F4 |
| TC-US-706-2 | US-706 AC2 | G: buka detail; W: tampil; T: rincian pembayaran (US-801) dan riwayat status | positif | E2E | - | F4 |

---

## 8. Pemetaan story ke test case

Pemetaan lengkap (story ke ID test case) ada di `docs/traceability.md`, bagian "Story → test case (Tahap 7)", dan kolom "Test case" di tabel BR. Pemeriksaan cakupan AC dilakukan dengan skrip pencarian teks: setiap AC dari setiap story di PRD muncul di kolom "AC" minimal satu test case, kecuali AC yang berisi `[TIDAK DIKETAHUI]` (§9, T-28).

## 9. Temuan

Daftar acceptance criteria dan kebutuhan yang tidak bisa diuji, ambigu, saling bertentangan, atau belum mencakup kondisi gagal. Tidak ada yang diperbaiki di sini; PRD, backlog, dan phase plan tidak diubah. Versi 0.2: kolom "Status" menandai temuan yang tertutup oleh klarifikasi AC Fase 1 yang diterapkan di PRD v1.2 final (`docs/07-klarifikasi-ac-fase-1.md`, Checkpoint 2); T-36–T-43 adalah temuan baru dari PRD v1.2 final. Klasifikasi blokir/non-blokir mengikuti `aturan-kerja.md` §2. Jenis: TU = tidak bisa diuji, AM = ambigu, BT = bertentangan atau tidak konsisten, KG = belum mencakup kondisi gagal, CS = celah spesifikasi. File acuan: `docs/03-prd.md` kecuali disebut lain. Usulan rumusan adalah saran QA untuk diputuskan di Checkpoint 2 atau gate berikutnya.

| ID | Story / AC | Jenis | Alasan | Usulan rumusan | Status (PRD v1.2 final) |
| --- | --- | --- | --- | --- | --- |
| T-01 | US-101 AC2 | TU, KG | Masa berlaku tautan verifikasi email tidak ditetapkan (hanya atur ulang yang 60 menit, US-103 AC4). Tidak ada ketentuan tautan lama setelah kirim ulang | Tambah: "tautan verifikasi berlaku N menit; tautan yang lebih lama menjadi tidak berlaku setelah kirim ulang" | Tertutup: US-101 AC2, AC2b → TC-US-101-8, -11, -19–21 |
| T-02 | US-101 AC1, NFR-SEC-03 | TU | "Memenuhi kebijakan NFR-SEC-03" merujuk NFR-SEC-01 (ASVS) tanpa angka (panjang minimum/maksimum, daftar kata sandi umum). Batas tidak dapat diuji | Tetapkan nilai kebijakan password di PRD atau rujuk butir ASVS 5.0.0 spesifik (diverifikasi di Tahap 8) | Tertutup sebagian: nilai kebijakan di NFR-SEC-03 → TC-NFR-SEC-03-2–7; daftar kata sandi bocor dan lisensinya `[TIDAK DIKETAHUI]` (Tahap 8) |
| T-03 | US-101 AC3 | AM, CS | Mengarahkan pasien "masuk atau atur ulang password" menyatakan email sudah terdaftar, sementara US-103 AC2 dan AC4 sengaja netral. Ada celah enumerasi akun | Putuskan: pesan netral di registrasi (email konfirmasi ke pemilik) atau terima risikonya dan catat sebagai keputusan | Tertutup: US-101 AC3 (UP-73) → TC-US-101-12, -22–24; sisa pengukuran: T-39 |
| T-04 | US-101 AC1 | KG | Tidak ada AC untuk kegagalan: pengiriman email verifikasi gagal, atau sebagian langkah pembuatan akun gagal (kompensasi ADR-011 butir 6 hanya ada di ADR dan coding standards) | Tambah AC kegagalan: "tidak ada akun setengah jadi dan pasien dapat mengulang" serta perilaku saat email gagal terkirim | Tertutup: US-101 AC1b, AC1c → TC-US-101-4, -16–18 |
| T-05 | US-103 AC1 | KG | Perilaku masuk dengan email belum terverifikasi tidak ditetapkan (pesan, kirim ulang tautan) | Tambah AC: "email belum terverifikasi tidak dapat masuk, pesan memandu kirim ulang tautan" | Tertutup: US-103 AC1a → TC-US-103-2, -21; urutan pemeriksaan Keycloak diverifikasi Tahap 8 |
| T-06 | US-103 AC3 | AM | Titik awal 15 menit tidak jelas (dari gagal ke-5 atau dari gagal terakhir); pesan akun terkunci menunggu PT-44; belum dinyatakan apakah pesan terkunci boleh membuka keberadaan akun. ADR-011 butir 4 menyebut perilaku Keycloak akan diuji | Rumuskan: "terkunci selama 15 menit sejak percobaan gagal ke-5" dan tetapkan pesan setelah PT-44 | Tertutup: US-103 AC3; PT-44 digantikan pesan seragam AC2 → TC-US-103-6–9, -26–29 |
| T-07 | NFR-SEC-08, US-106 AC3 | AM | "5 MB" dan "10 MB" tidak menyatakan 10^6 atau 2^20 byte | Tetapkan satuan, misalnya 5 MiB = 5.242.880 byte | Tertutup: NFR-SEC-08 (MiB), US-106 AC3 (`4130001`, `4150001`) → TC-US-106-8–10, TC-NFR-SEC-08-1, -3 |
| T-08 | US-103 AC4 | KG | Tidak ada AC untuk tautan atur ulang yang kedaluwarsa, sudah dipakai, atau password baru melanggar kebijakan; tidak ada batas laju permintaan atur ulang | Tambah AC kondisi gagal dan batas laju | Tertutup: US-103 AC4a–AC4d → TC-US-103-12–14, -30–33 |
| T-09 | US-103 AC5, US-1302 AC4, US-207 AC1 | AM, BT | AC5 merujuk KU-10 (akun nonaktif) untuk token setelah keluar, padahal KU-10 mengatur akun nonaktif. Kata "seketika" (US-1302 AC4) dan "ditolak" tanpa batas waktu tidak terukur | Rujuk "token sesi yang diakhiri ditolak"; definisikan batas, misalnya "paling lambat pada permintaan berikutnya" | Tertutup: KU-10 (definisi "berikutnya"), US-103 AC5 (`4010001`); "seketika" di US-1302 AC4 dibaca menurut KU-10 → TC-KU-10-1, -2, TC-US-103-16, TC-US-207-4, TC-US-1302-4 |
| T-10 | US-104 AC1 | TU | Teks final (legalitas L-08; retensi dan jangka waktu PT-24) belum ada; hanya struktur tujuh bagian dan kontak yang dapat diuji | Biarkan; TC-US-104-2 menjadi pintu go-live yang memeriksa teks final | Terbuka (pintu go-live, TC-US-104-2) |
| T-11 | US-104 AC3, AC4 | CS, TU | Tidak ada story atau AC untuk siapa yang menerbitkan versi teks dan butir persetujuan baru dan bagaimana (data, bukan aplikasi). Di Fase 1 hanya seed; "tanpa mengubah aplikasi" tidak punya mekanisme yang dapat diuji selain pemasukan data | Tambah story atau AC pengelolaan versi persetujuan, atau nyatakan eksplisit dikelola lewat migrasi data | Tertutup: US-104 AC3 (migrasi data) → TC-US-104-6, -12, -13 |
| T-12 | US-105 AC1 | TU | Aturan validasi nama, tanggal lahir (rentang), nomor HP (format), alamat (field wajib) tidak ada; hanya KU-01 generik | Tambah tabel validasi field profil | Tertutup sebagian: US-105 AC1a, AC1b → TC-US-105-2, -15–20; nilai PB-22 `[TIDAK DIKETAHUI]` (L-08); sisa: T-42 |
| T-13 | US-105 AC4 | KG | Tidak ada AC: email baru sudah dipakai akun lain, tautan kedaluwarsa, batas waktu; autentikasi ulang sebelum ubah email (ada di ADR-011 butir 9 dan F1-T57) tidak ada di AC | Tambah AC kondisi gagal dan syarat autentikasi ulang | Tertutup: US-105 AC4a–AC4e → TC-US-105-12–14, -21–25 |
| T-14 | US-106 AC1 vs US-105 AC3 | BT | US-105 AC3 menyatakan perubahan NIK lewat US-106, tetapi US-106 AC1 hanya berlaku untuk status "Belum diunggah" atau "Ditolak". Cara pasien "Disetujui" mengubah NIK tidak ditetapkan | Tambah ke US-106: perilaku pengajuan saat status "Disetujui" (status kembali "Menunggu verifikasi" atau ditolak) | Tertutup: US-106 AC1, AB-05a → TC-US-106-23–25, TC-US-105-10 |
| T-15 | US-106 AC2, AC5 | TU, AM | Panjang NIK "ditetapkan di Tahap 4": sekarang 16 digit di data dictionary A2.4 tetapi AC belum diperbarui. Status setelah "membatalkan pengiriman" tidak ditetapkan | Perbarui AC2 dengan 16 digit; tambah di AC5 status sesudah batal | Tertutup: US-106 AC2, AC5 → TC-US-106-6, -7, -16, -33, -34 |
| T-16 | US-106 AC4 | AM, CS | Pesan penolakan NIK yang sudah dipakai memberi tahu bahwa NIK itu terdaftar di akun lain (enumerasi NIK); status pengajuan setelah ditolak tidak ditetapkan | Putuskan pesan netral dan status pengajuan; tinjauan privasi | Tertutup: US-106 AC4 (pesan umum, batas berlapis) → TC-US-106-12, -26–31; sisa ambiguitas: T-37 |
| T-17 | US-201 AC3 | TU | Isi dan pengelolaan daftar alasan penolakan tidak didefinisikan (tabel `reason_code` ada di phase plan tetapi tidak di PRD) | Tambah daftar alasan awal atau rujuk data dictionary | Tertutup: US-201 AC3, AC3a → TC-US-201-9, -20–22 |
| T-18 | US-201 AC7 | AM | Status pengajuan setelah persetujuan ditolak karena NIK sudah dipakai tidak ditetapkan (tetap "Menunggu" atau otomatis "Ditolak") | Tentukan status akhir dan pemberitahuan | Tertutup: US-201 AC7 → TC-US-201-14, -17, -23 |
| T-19 | US-208 | CS | (a) Pembuatan Super Admin pertama tidak diatur (bootstrap ada di F1-T65, bukan PRD); (b) menonaktifkan akun sendiri saat masih ada Super Admin lain tidak diatur; (c) kombinasi sub-peran yang melanggar pemisahan tugas (misalnya Auditor dengan Super Admin) tidak diatur | Tambah AC untuk (a)–(c) atau nyatakan diperbolehkan | Tertutup: US-208 AC4–AC8, US-201 AC8 → TC-US-208-15–27, TC-US-201-24–27; sisa: T-38, PT-52 |
| T-20 | KU-02 | TU | "Batas waktu" respons server tidak berangka | Tetapkan batas (misalnya p99 klien) per jenis permintaan | Tertutup: KU-02 (15 dan 60 detik) → TC-KU-02-3 |
| T-21 | US-1501 AC3 | AM | "Peringatan keamanan" tidak punya ambang, penerima, dan saluran | Tambah: penerima, saluran, dan waktu paling lambat | Tertutup sebagian: US-1501 AC3 → TC-US-1501-17, -18; daftar penerima email `[TIDAK DIKETAHUI]` (sebelum staging) |
| T-22 | US-1504 AC3 | TU | Bukan kriteria: "mekanisme ditetapkan di Tahap 4". Perilaku yang dapat diuji ada di ADR-005/ADR-009 (hash berantai, jangkar) tetapi tidak ada di AC | Ganti AC3 dengan kriteria uji: perubahan atau penghapusan satu event terdeteksi verifikasi harian | Tertutup: US-1504 AC3 → TC-US-1504-6–11 |
| T-23 | US-205 AC2 vs fase | BT | AC2 mensyaratkan integrasi "Aktif" (US-1405, Fase 3), tetapi US-205 ada di Fase 2. Di Fase 2 syarat ketiga hanya dapat diuji dengan stub status | Catat di backlog kolom Fase: AC2 syarat ketiga diverifikasi F3 | Terbuka; tenggat di `07-klarifikasi-ac-fase-1.md` §4 |
| T-24 | US-801 AC1, AC5, US-804 AC2, US-812 AC4, AB-10a, AB-27 | TU | Jenis dan tarif pajak, pembulatan, perlakuan atas ongkos kirim dan biaya transaksi, isi bukti bayar, dan perilaku saat tarif belum diatur `[TIDAK DIKETAHUI]` (sisa PT-28). Nilai pajak nyata dan kasus "tarif kosong" tidak dapat diuji; TC memakai tarif uji | Tutup PT-28 sebelum EN-24; sementara, tetapkan perilaku "tarif belum diatur" (misalnya menolak pembayaran atau pajak 0 dengan peringatan) | Terbuka; tenggat di `07-klarifikasi-ac-fase-1.md` §4 |
| T-25 | US-1605 AC1–AC3, US-1606 AC1–AC2 | TU | Terblokir PT-24 dan L-06 (retensi, benturan hak hapus dengan retensi rekam medis). TC berstatus "Terblokir" | Tutup PT-24/L-06 sebelum Fase 3 | Terbuka; tenggat di `07-klarifikasi-ac-fase-1.md` §4 |
| T-26 | US-1406 AC2 | TU | Evaluasi calon penyedia (BR-113) bukan perilaku sistem; hanya dapat diverifikasi dengan tinjauan dokumen | Ubah menjadi syarat proses pemilihan (checklist dokumen) di luar AC fungsional | Terbuka; tenggat di `07-klarifikasi-ac-fase-1.md` §4 |
| T-27 | US-1001 AC5 | TU | Lokasi data penyedia email hanya dapat diverifikasi dengan dokumen penyedia, bukan test fungsional | Sama dengan T-26 | Terbuka; tenggat di `07-klarifikasi-ac-fase-1.md` §4 |
| T-28 | US-203 AC4, US-205 AC4–AC5, US-308 AC4, US-309 AC5, US-702 AC4, US-803 AC4, US-810 AC3, US-1307 AC3, US-1602 AC3, US-1604 AC3 | TU | AC berisi `[TIDAK DIKETAHUI]` atau "ditetapkan di Tahap 4 atau legal"; tidak ada hasil yang dapat diverifikasi | Pindahkan ke bagian "pertanyaan terbuka", bukan AC, sampai terjawab | Terbuka; tenggat di `07-klarifikasi-ac-fase-1.md` §4 |
| T-29 | US-808 AC3, US-806 AC1, US-1001 matriks, US-801 AC3 | BT | AC story di Fase 2 mereferensi story fase lain (US-1201 Fase 3; RF-08/09/15 produk Fase 4; RF-18 integrasi Fase 3; kejadian notifikasi dari Fase 3-4; AB-16 untuk klinik/lab). Penempatan fase di phase plan tidak mencantumkan AC mana yang tertunda | Tandai AC yang tertunda di backlog (KEPUTUSAN-179); TC sudah diberi fase tujuan | Terbuka; tenggat di `07-klarifikasi-ac-fase-1.md` §4 |
| T-30 | NFR-CAP-04 | TU | Alat uji beban belum ditetapkan di stack; skenario beban (100 sesi video dan chat, 1.000 pengguna per menit) tidak punya alat | Putuskan alat di Fase 3 (EN-25) dengan cek registry dan catatan sumber sebelum dipakai | Terbuka; tenggat di `07-klarifikasi-ac-fase-1.md` §4 |
| T-31 | NFR-DR-02 | TU | Jam kerja dan kalender hari kerja belum ditetapkan, sehingga RTO 4 jam vs 8 jam tidak dapat dibedakan | Tetapkan jam kerja dan kalender sebelum uji pemulihan | Terbuka; tenggat di `07-klarifikasi-ac-fase-1.md` §4 |
| T-32 | US-1405 AC2, NFR-INT-03 | AM | "Gagal terus selama 30 menit" tidak menentukan apakah satu keberhasilan di tengah menghentikan hitungan atau berapa ambang kegagalan | Rumuskan: "tidak ada satu pun sinkron yang berhasil selama 30 menit berturut-turut" | Terbuka; tenggat di `07-klarifikasi-ac-fase-1.md` §4 |
| T-33 | US-309 AC8 | TU | Batas laju verifikasi resep tidak berangka | Tetapkan, misalnya N permintaan per menit per alamat IP | Terbuka; tenggat di `07-klarifikasi-ac-fase-1.md` §4 |
| T-34 | US-1202 AC3 | CS | Tidak ada ketentuan pemisahan tugas: admin Ruang Sehat yang mengubah rekening dan Verifikator yang menyetujui dapat orang yang sama | Tambah AC: pengusul dan penyetuju harus berbeda (dua orang) | Terbuka; tenggat di `07-klarifikasi-ac-fase-1.md` §4 |
| T-35 | US-1101 AC1, US-1103 AC1 | KG | Validasi isi konten dan lampiran foto (jenis, ukuran, sanitasi HTML) tidak disebut selain KU-01 dan NFR-SEC-08 | Tambah AC sanitasi konten | Terbuka; tenggat di `07-klarifikasi-ac-fase-1.md` §4 |
| T-36 | US-101 AC3, US-105 AC4e | AM | Pembandingan "email sudah terdaftar" atau "dipakai akun lain" tidak menyebut normalisasi huruf besar/kecil; normalisasi hanya disebut untuk batas laju (US-101 AC2b, US-103 AC4c). Tanpa itu, "Pasien@contoh" dan "pasien@contoh" dapat menjadi dua akun dan AC3 tidak dapat diuji untuk variasi huruf | Tambah: "email dibandingkan dan disimpan setelah dinormalisasi huruf kecil" (atau tetapkan sebaliknya) | Terbuka, non-blokir; dijawab sebelum task registrasi F1 |
| T-37 | US-106 AC4 | AM, KG | (a) Konsekuensi setelah 10 penolakan per alamat IP per 24 jam tidak dinyatakan (berbeda dengan per akun "ditahan" dan per NIK "ditolak"); (b) pesan yang tampil saat akun ditahan tidak dinyatakan; (c) tidak jelas apakah penolakan selama ditahan menambah hitungan, dan apakah jendela 24 jam bergulir | Rumuskan: "per alamat IP 10 penolakan per 24 jam bergulir, lalu pengajuan NIK dari IP itu ditolak dengan pesan umum yang sama selama 24 jam; pesan saat ditahan sama dengan pesan umum; penolakan selama ditahan tidak menambah hitungan" | Terbuka, non-blokir (teknis, ditetapkan Tech Lead sebelum task US-106 F1); TC-US-106-27 dan -29 menunggu |
| T-38 | US-201 AC8 | TU, CS | Akun admin (realm `admin`) dan akun pasien (realm `pasien`) terpisah. PRD tidak menyatakan bagaimana sistem mengetahui bahwa Verifikator "juga pemilik akun pasien" (kecocokan email, NIK, atau pernyataan Verifikator). Tanpa penentu, AC8 tidak dapat diuji dan mudah dielakkan dengan email lain. Perilaku bila tidak ada Verifikator aktif lain juga tidak diatur | Tetapkan penentunya, misalnya "kecocokan email ternormalisasi atau NIK pemilik akun yang disetujui, ditambah pernyataan akun pasien saat akun Verifikator dibuat", dan perilaku bila hanya ada satu Verifikator | Terbuka, **blokir** untuk TC-US-201-24–26 (`aturan-kerja.md` §2 butir 4: penentu ini keputusan bisnis dan privasi yang tidak boleh diambil QA atau developer); tidak memblokir TC lain |
| T-39 | US-101 AC3 | AM | "Waktu respons ditahan sampai batas tetap 900 ms" tidak menyatakan perilaku bila pemrosesan melebihi 900 ms (selisih waktu kembali terlihat) dan toleransi pengukuran yang dianggap tidak dapat dibedakan | Tambah perilaku saat pemrosesan > 900 ms dan kriteria uji (misalnya selisih median kedua kasus < N ms pada M sampel) | Terbuka, non-blokir |
| T-40 | US-103 AC6, AC7 | KG | Batas waktu pengakhiran sesi lain setelah ubah password atau daftar ulang TOTP tidak ditetapkan (AC4d memakai 2 menit); perilaku bila opsi "keluar dari perangkat lain" tidak dicentang tidak dinyatakan | Samakan: "sesi lain diakhiri paling lambat 2 menit; bila opsi tidak dicentang, sesi lain tetap berjalan" | Terbuka, non-blokir |
| T-41 | US-103 AC8, US-101 AC5, US-207 AC5 | CS, KG | (a) Apakah pengarahan ke daftar ulang TOTP setelah memakai kode pemulihan dapat dilewati; (b) perilaku saat kode pemulihan tinggal sedikit atau habis; (c) password salah di halaman tautan reset MFA: apakah dihitung untuk penguncian US-103 AC3 dan apakah tautan tetap berlaku | Tambah AC untuk (a)–(c) | Terbuka, non-blokir; dijawab sebelum task MFA F1 dan US-207 F2 |
| T-42 | US-105 AC1a, AC1b | AM | (a) "Tepat satu alamat utama" tanpa ketentuan apakah profil wajib punya minimal satu alamat dan apa yang terjadi saat alamat utama dihapus; (b) AC1b menyebut tanggal lahir "saat pendaftaran", padahal US-101 AC1 tidak menyebut tanggal lahir di formulir pendaftaran | Tetapkan alamat utama pengganti saat alamat utama dihapus; nyatakan field formulir pendaftaran | Terbuka, non-blokir |
| T-43 | US-201 AC8 | BT (keterlacakan) | US-201 AC8 ada di PRD v1.2 final, tetapi tidak tercantum di riwayat PRD §14 maupun di tabel "Perubahan AC setelah Checkpoint 2" di `docs/traceability.md` | Pemilik dokumen menambahkannya ke kedua tempat itu | Terbuka, non-blokir |

## 10. Riwayat dokumen

| Versi | Tanggal | Perubahan |
| --- | --- | --- |
| 0.1 | 2026-11-18 | Draf awal Tahap 7 dari PRD v1.2 draf; belum ada eksekusi test |
| 0.2 | 2026-11-18 | Tindak lanjut Checkpoint 2, disesuaikan dengan PRD v1.2 final: TC diubah untuk AC yang dirumuskan ulang (KU-02, KU-10, US-101, US-103, US-104 AC3, US-105, US-106, US-201, US-208, US-1501 AC3, US-1504 AC3, NFR-SEC-02/03/08); TC baru untuk US-101 AC1b, AC1c, AC2b, AC5; US-103 AC1b, AC4c–AC4e, AC6–AC8 (CR-017, CR-018); US-105 AC1a, AC1b, AC4a–AC4e; US-106 AC1 (dari "Disetujui"), AC4 (batas berlapis), AC5; US-201 AC3a, AC8; US-207 AC5; US-208 AC4–AC8; US-110 (story baru); NFR-SEC-11. §9: kolom status, T-01–T-22 ditandai tertutup atau tertutup sebagian, temuan baru T-36–T-43. Total 674 → 805 TC; belum ada eksekusi test |
