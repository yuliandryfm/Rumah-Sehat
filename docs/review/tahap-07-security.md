# Review Keamanan Tahap 7: Klarifikasi Acceptance Criteria Fase 1

Tanggal: 2026-11-18
Reviewer: security-architect (independen, tidak ikut menyusun dokumen yang direview)
Tingkat ketelitian: T1 Kritis, 2 putaran (maksimal 2) (`aturan-kerja.md` §1)
**Status akhir (putaran 2): LULUS DENGAN CATATAN.** B-01 tertutup. Rinciannya di §10.
Dokumen yang direview: `docs/07-klarifikasi-ac-fase-1.md` v0.1 (§1.4, §2, T-01–T-09, T-13, T-16, T-18, T-19, §5 S-01–S-05), KEPUTUSAN-187–197 di `docs/decision-log.md`. Putaran 2: v0.2 dan berkas pendukung yang berubah (§10)
Acuan: ADR-011, ADR-012, ADR-021, ADR-025, `docs/05-asvs-checklist.md` (V6, V7), `docs/03-prd.md` (US-101, US-103, US-105, US-106, US-201, US-208, KU-10, NFR-SEC-02/03), `docs/04-threat-model.md` (APP-04, API-10, KC-01, KC-07, WEB-07, M-12, M-16, H-01, H-04, H-05), `docs/sources.md` (SRC-261, SRC-262, SRC-265)

Metode: kriteria blokir mengikuti `aturan-kerja.md` §2. Setiap klaim keamanan di bawah merujuk ke halaman resmi yang saya buka sendiri hari ini, yaitu R-1–R-10 di §8. Nomor baris di §1–§9 merujuk ke `docs/07-klarifikasi-ac-fase-1.md` v0.1; nomor baris di §10 merujuk ke v0.2.

---

## 1. Verdict

**TIDAK LULUS (putaran 1).** Ada satu blokir (B-01). Perbaikannya cukup di tingkat teks, dan cek ulang cukup memeriksa baris yang berubah.

- **B-01**: teks KU-10 usulan dan UP-73 butir 4 menambahkan satu kalimat khusus untuk akun nonaktif. Akibatnya status nonaktif terbuka di layar login, padahal §2.3 P1 dan ADR-011 butir 4 mewajibkan pesan galat login yang umum.
- **S-01 terkonfirmasi.** Teks resmi ASVS 5.0.0 mewajibkan MFA di Level 2. Ini bukan blokir dokumen ini: pertentangannya sudah ada di ADR-011 yang lulus gate, dan penyusun sudah benar membawanya ke user tanpa memutuskannya. Opsi untuk user ada di §3.
- **S-02** saya putuskan dari sisi teknis di §5.5. 7.4.3 adalah butir Level 2, sehingga status quo (sesi lain tetap hidup setelah atur ulang password) tidak memenuhi target ADR-011.
- Ada 14 catatan substantif non-blokir (C-01–C-14, §7). Butir yang ditandai "sebelum task" perlu masuk teks AC sebelum task terkait dimulai.

Di luar B-01, isi dokumen berkualitas baik. Level ASVS yang dikutip (6.2.x, 6.3.5, 6.3.8, 6.4.3) cocok dengan teks resmi. Perilaku Keycloak yang belum terverifikasi juga ditandai jujur sebagai `[TIDAK DIKETAHUI]`, lengkap dengan task verifikasinya.

---

## 2. Blokir

### B-01 · §2.4 baris 4 (baris 89) dan §2.5 KU-10 (baris 101–102) · kriteria §2 butir 3

**Temuan.** Untuk akun nonaktif saat masuk, usulannya adalah "pesan sama dengan US-103 AC2, **ditambah** kalimat umum 'Bila tidak dapat masuk, gunakan Lupa password atau hubungi Layanan Pelanggan'". Rumusan KU-10 juga menulis "pesan yang sama dengan pesan login gagal (US-103 AC2), **ditambah** petunjuk umum ke Layanan Pelanggan". Kalimat tambahan itu hanya muncul untuk akun nonaktif. Jadi layar akun nonaktif berbeda dari layar password salah, akun tidak ada, atau akun terkunci, dan perbedaan itu sendiri menjadi sinyal status akun.

**Mengapa blokir.** Ini bertentangan dengan ADR-011 butir 4 ("Pesan galat login dan atur ulang password bersifat umum"), yang lulus gate, dan dengan prinsip P1 dokumen ini sendiri (baris 77). Perilaku yang berubah adalah enumerasi akun nonaktif. OWASP Authentication Cheat Sheet mewajibkan pesan umum, termasuk ketika "The account is locked or disabled." `[SRC-261]`.

**Cara mereproduksi (di tingkat teks).** Bandingkan teks layar yang dihasilkan AC2 (password salah) dengan teks KU-10 usulan (akun nonaktif): keduanya berbeda satu kalimat. Apakah Keycloak memeriksa password sebelum menampilkan pesan akun nonaktif masih `[TIDAK DIKETAHUI]` (baris 89). Dalam kasus terburuk, sinyal ini bisa diperoleh tanpa mengetahui password.

**Perbaikan (pilih salah satu).**
- (a) Jadikan kalimat petunjuk Lupa password dan Layanan Pelanggan bagian dari pesan standar AC2, sehingga tampil di **semua** kegagalan login: password salah, akun tidak ada, terkunci, dan nonaktif. Pesan untuk akun terkunci (baris 87) dan nonaktif lalu identik dengan AC2.
- (b) Hapus kalimat tambahan dari pesan akun nonaktif.

Test di F1-T55 perlu membandingkan teks layar keempat kondisi itu secara persis, di semua realm yang memakai tema (pasien, nakes, admin, rme).

---

## 3. Pertanyaan 1: S-01, MFA di ASVS 5.0.0 Level 2

### 3.1 Verifikasi ke teks resmi (R-1)

Benar, MFA diwajibkan di Level 2. Ada tiga tempat di bab V6 yang menyatakannya:
- **Pengantar V6.2**: "From L2 onwards, multi-factor authentication mechanisms are required, where passwords may be one of those factors."
- **Pengantar V6.3**: "L2 applications must force the use of multi-factor authentication (MFA)."
- **Butir 6.3.3 (Level 2)**: "Verify that either a multi-factor authentication mechanism or a combination of single-factor authentication mechanisms, must be used in order to access the application." Kalimat penutup butir yang sama berbunyi: "Relaxing any of the considerations in this requirement requires a fully documented rationale and a comprehensive set of mitigating controls."

Dua catatan penafsiran:
- **"Combination of single-factor authentication mechanisms" tidak didefinisikan** di bab V6 (R-1). Klaim bahwa password ditambah kontrol lain, misalnya pengikatan perangkat, sudah memenuhi 6.3.3 tidak punya dasar dari teks: `[TIDAK DIKETAHUI]`.
- **Cakupan kalimat "Relaxing ..." tidak tegas.** Kalimat itu muncul setelah ketentuan L3 tentang faktor perangkat keras, sehingga tidak jelas apakah berlaku juga untuk kewajiban MFA L2. Dokumen pengantar ASVS menyatakan bahwa Level 2 berarti menerapkan "all of the L1 and L2 requirements" (R-3), dan mengizinkan organisasi membuat "an organization- or domain-specific fork that adjusts requirements" dengan menjaga ketertelusuran (R-3). Dokumen asesmen meminta verifier "provide an opinion on the rationale of excluding the requirements" (R-4).

Kesimpulannya: tanpa MFA, realm `pasien` tidak dapat diklaim memenuhi Level 2 penuh. Yang dapat dipertanggungjawabkan adalah Level 2 dengan penyimpangan yang terdokumentasi dan disertai kontrol mitigasi.

### 3.2 Dampak ke ADR-011

- **Butir 1** (L2 untuk semua komponen) bertentangan dengan **butir 5** ("Pasien tidak diwajibkan MFA di rilis 1") dan tabel realm butir 3 (`pasien`: "tanpa MFA").
- Threat model APP-04 berstatus "Tercermin; risiko sisa pasien tanpa MFA", dan baris 413 threat model mencatatnya sebagai risiko sisa. Penerimaan risikonya sudah ada sejak Tahap 4. **Yang keliru adalah klaim kepatuhan L2-nya, bukan penerimaan risikonya.**
- Opsi mana pun yang dipilih memerlukan revisi ADR-011, yaitu butir 5 atau butir 1, ditambah NFR-SEC-02, baris V6 "MFA" di `docs/05-asvs-checklist.md`, dan penilaian ulang APP-04.

### 3.3 Faktor kedua yang tersedia tanpa penyedia di luar Indonesia (CR-007, T4-Q1)

| Faktor | Ketersediaan di Keycloak 26.8.2 | Posisi ASVS | Catatan lokasi data |
| --- | --- | --- | --- |
| TOTP (aplikasi authenticator) | Ada: OTP policy `[SRC-67]`; sudah dipakai realm `nakes`, `admin`, `rme` | TOTP termasuk "something you have" (pengantar V6.5); 6.5.x L2 | Kode dihitung di perangkat, tanpa penyedia pihak ketiga di sisi Ruang Sehat |
| Kode pemulihan | Ada (`recovery-codes:v1`, bawaan aktif) `[SRC-68]` | Lookup secret, 6.5.1–6.5.4 L2 | Tanpa pihak ketiga |
| WebAuthn (kunci keamanan, platform authenticator) | Ada: "{project_name} uses WebAuthn for two-factor authentication" (R-7) | Faktor perangkat; L3 menuntut faktor perangkat keras (6.3.3) | Kunci terikat perangkat tidak melibatkan pihak ketiga. **Passkey yang disinkronkan** lewat layanan awan platform: lokasi pemrosesannya `[TIDAK DIKETAHUI]`, dan apakah termasuk "penyedia" menurut T4-Q1 adalah keputusan user. Halaman R-7 tidak menyatakan status dukungan passkey |
| OTP email | Tidak boleh | Pengantar V6.6: "Unsafe out-of-band authentication mechanisms such as e-mail and VOIP are not permitted." (R-1) | Sudah dilarang ADR-011 butir 5 |
| SMS | Dikecualikan ADR-011 butir 5 | 6.6.1 (L2) mengizinkan PSTN dengan syarat (nomor tervalidasi, alternatif yang lebih kuat, informasi risiko) (R-1) | Butuh penyedia SMS di Indonesia `[TIDAK DIKETAHUI]` |

Keycloak juga mendukung **step-up** berbasis tingkat autentikasi (`acr_values`, sub-flow "Conditional - Level Of Authentication") (R-8). Dengan step-up, faktor kedua dapat diwajibkan hanya untuk klien atau aksi tertentu.

### 3.4 Opsi untuk keputusan user (tidak diputuskan reviewer)

| Opsi | Isi | Status L2 untuk realm `pasien` | Dampak dan biaya |
| --- | --- | --- | --- |
| **M1. MFA wajib untuk pasien** | TOTP wajib (alur seperti realm `nakes`), kode pemulihan; WebAuthn sebagai alternatif bila disetujui | Memenuhi 6.3.3 | Friksi pendaftaran dan login bagi populasi pasien (dampak adopsi `[TIDAK DIKETAHUI]`). Pasien yang kehilangan perangkat memerlukan pembuktian identitas setara saat pendaftaran (6.4.4 L2), yang menjadi beban Layanan Pelanggan dan memperluas prosedur H-05 ke pasien. Atur ulang password tidak boleh melewati MFA (6.4.3 L2). Perlu revisi ADR-011 butir 3 dan 5, NFR-SEC-02, US-101, US-103 |
| **M2. MFA sukarela + step-up wajib untuk aksi berisiko** | Pasien boleh mengaktifkan TOTP (H-04 sudah mengusulkannya). Aksi berisiko (ubah email, unduh salinan data, hak subjek data) mewajibkan faktor kedua lewat LoA/step-up (R-8); pasien tanpa faktor kedua diminta mendaftarkannya saat itu | Penyimpangan 6.3.3 untuk akses dasar, wajib terdokumentasi | Melindungi API-15 (salinan data) dan pengambilalihan lewat ubah email. Butuh konfigurasi alur LoA dan test. Revisi ADR-011 butir 1 (pengecualian tercatat) dan butir 5 |
| **M3. Tanpa MFA, penyimpangan terdokumentasi dengan kompensasi** | Klaim "L2 dengan penyimpangan 6.3.3 untuk realm `pasien`", dengan alasan dan kontrol mitigasi lengkap (kalimat "Relaxing ..." 6.3.3, R-1; fork ASVS, R-3). Kompensasi minimal: pemeriksaan kata sandi bocor (6.2.12), batas laju login per IP yang terdokumentasi (6.1.1), penghentian sesi setelah atur ulang (§5.5), autentikasi ulang aksi berisiko (sudah ada), pemberitahuan email ganti password dan login perangkat baru (H-04) | Bukan L2 penuh, tetapi jujur dan tertelusur | Biaya terendah. Risiko phishing dan pemakaian ulang kredensial tetap ada. Dicatat di risk register dan DPIA (APP-04 sudah menjadi masukan DPIA) |
| **M4. Target realm `pasien` diturunkan ke L1 untuk V6** | Klaim L1 untuk autentikasi pasien, L2 untuk komponen lain | L1 | Kontrol teknis sama dengan M3, tetapi labelnya lebih eksplisit. ASVS tidak membahas level berbeda per komponen (R-3, R-4), jadi pembagian ini adalah keputusan proyek. Revisi ADR-011 butir 1 |

**Penilaian kompensasi "panjang minimum 15 karakter" (usulan S-01, baris 530).** 6.2.1 menyebut 15 sebagai "strongly recommended" untuk semua level (R-1), bukan sebagai pengganti MFA. Panjang minimum memperkuat password terhadap tebakan dan brute force, tetapi tidak menahan phishing atau pemakaian ulang kredensial yang bocor. Pemakaian ulang lebih efektif ditahan oleh pemeriksaan kata sandi bocor (6.2.12) dan batas laju. Angka 15 boleh menjadi bagian M3/M4, tetapi tidak boleh disajikan sebagai pemenuhan 6.3.3.

---

## 4. Pertanyaan 2: apakah UP-73 menutup enumerasi di semua jalur tanpa login

| Jalur | Status opsi B | Kesesuaian dengan Keycloak | Celah |
| --- | --- | --- | --- |
| Pendaftaran (baris 86, 99) | Tertutup di layar dan isi respons | Pendaftaran lewat backend (ADR-011 butir 6) | **C-02** kompensasi tidak boleh menyentuh akun dan persetujuan yang sudah ada; **C-03** halaman pendaftaran bawaan Keycloak harus mati; **C-04** email "Anda sudah punya akun" belum dibatasi lajunya; **C-05** sisa sinyal waktu dari Argon2 dan pembuatan user |
| Kirim ulang verifikasi (T-01 AC2b) | Tertutup, batas per email diam | Backend | — |
| Masuk, password salah atau akun tidak ada | Tertutup (ADR-011 butir 4) | Pesan bawaan umum | Sinyal waktu di Keycloak adalah ranah 6.3.8 (L3), risiko sisa |
| Akun terkunci (baris 87) | Tertutup | Sesuai: Keycloak menampilkan "Invalid username or password" (R-9, `[SRC-262]`) | **C-06** waktu kunci Keycloak dapat bertambah |
| Akun nonaktif saat masuk (baris 89) | **Belum tertutup** karena kalimat tambahan | Pesan bawaan Keycloak untuk akun nonaktif tidak disebut di halaman brute force (R-9) | **B-01** |
| Lupa password (US-103 AC4, T-08) | Tertutup di teks layar | Halaman milik Keycloak. Per-akun sesuai Forgot Password Cheat Sheet (R-5) | **C-08** `4290001` tidak berlaku di halaman Keycloak; mekanisme batas per email `[TIDAK DIKETAHUI]` (S-03) |
| Ubah email (baris 92, T-13 AC4e) | Tertutup terhadap pemohon | Hanya lewat API aplikasi (ADR-011 butir 6) | **C-03** jalur Update Email Keycloak harus tertutup; **C-04** email "alamat sudah terdaftar" belum dibatasi lajunya |
| Pengajuan NIK (baris 93, T-16) | Sinyal sisa diakui, dengan batas 3/24 jam per akun | Tidak terkait Keycloak | **C-11** batas per akun dapat dilipatgandakan dengan banyak akun |

**Jalur yang terlewat.**
1. **Halaman pendaftaran bawaan Keycloak.** Bila "User Registration" aktif, "a Register link displays on the login page" (R-10). Halaman itu adalah jalur pendaftaran kedua di luar kontrol UP-73. ASVS 6.3.4 (L2) mensyaratkan "there are no undocumented pathways" (R-1). Ini menjadi C-03.
2. **Pendaftaran tenaga kesehatan (US-202, Fase 2)** memakai pola yang sama di realm `nakes` (ADR-011 butir 6). UP-73 perlu dinyatakan berlaku juga untuk US-202 supaya klarifikasi Fase 2 tidak mengulang keputusan ini (non-blokir, Fase 2).

UP-73 secara keseluruhan konsisten dengan M-12 di threat model, yang mengusulkan pola yang sama persis (pesan layar sama, arahan dikirim ke email). Pembingkaian §2.2 juga tepat: 6.3.8 berlevel L3 (R-1), sehingga butir ini adalah keputusan penerimaan risiko.

---

## 5. Pertanyaan 3: kecukupan untuk ASVS L2 dan threat model

### 5.1 Kebijakan password (T-02, KEPUTUSAN-188)

Butir 1–7 cocok dengan 6.2.1, 6.2.4–6.2.12 (R-1). Celah yang tersisa:
- **6.2.2 dan 6.2.3 (L1)** belum punya AC. Pengguna yang sedang login harus dapat mengubah password, dan pengubahan itu meminta password lama dan baru. PRD tidak punya AC ubah password saat login (pencarian di `docs/03-prd.md`: nihil). Lihat C-07.
- **Mekanisme Keycloak (R-6):**
  - "Not email" hanya menolak password yang **sama dengan** email, bukan password yang memuat bagian lokal email.
  - "Password blacklist" memakai berkas teks yang dicocokkan tanpa membedakan huruf besar-kecil, lewat Bloom filter dengan kemungkinan positif palsu.
  - Tidak ada kebijakan bawaan yang memeriksa layanan kata sandi bocor eksternal.
  - Konsekuensinya ada di C-07.
- **Konsistensi lintas jalur (6.1.3, 6.3.4 L2).** Kebijakan yang sama harus berlaku di pendaftaran backend, halaman atur ulang Keycloak, aktivasi admin, dan ubah password. Bila pemeriksaan kata konteks hanya dibuat di backend, jalur atur ulang di Keycloak akan lolos.

### 5.2 Batas laju (KEPUTUSAN-192)

Sudah memenuhi semangat 6.1.1 untuk kirim ulang, atur ulang, dan NIK. Celah yang tersisa:
- Email pendaftaran dan ubah email belum dibatasi (C-04).
- Batas laju login per IP terhadap credential stuffing belum berangka (C-04). Penguncian Keycloak bersifat per akun, sehingga tidak menahan satu percobaan untuk banyak akun.
- Kode `4290001` tidak berlaku di halaman Keycloak (C-08).
- NIK (C-11).

### 5.3 Bootstrap Super Admin (UP-76 a)

Arahnya benar: tidak ada data pribadi di migrasi, ada jalur administrasi production, dan audit mencatat aktor "sistem (bootstrap)" beserta identitas operator. Celahnya ada di C-09:
- definisi "Super Admin aktif" belum mencakup undangan yang belum diaktifkan;
- belum ada penguncian terhadap eksekusi bersamaan;
- belum ada persetujuan orang kedua, padahal perintah ini membuat hak tertinggi;
- masa berlaku undangan belum diatur (6.4.1 L1).

### 5.4 Pemisahan tugas (UP-76 c)

Auditor eksklusif dan larangan menonaktifkan akun sendiri sudah benar. Namun keduanya tidak menutup WEB-07/M-16: Super Admin masih dapat membuat akun boneka, termasuk akun Auditor yang ia kendalikan. Lihat C-10.

### 5.5 Perilaku sesi (S-02), keputusan teknis yang direkomendasikan

**Dasar.**
- ASVS 7.4.3 (L2): "Verify that the application gives the option to terminate all other active sessions after a successful change ..." Butir ini mencakup perubahan password lewat atur ulang atau pemulihan, dan perubahan MFA (R-2).
- ASVS 7.5.1 (L2): email termasuk atribut akun sensitif yang perubahannya memerlukan autentikasi ulang penuh (R-2).
- Forgot Password Cheat Sheet: "Ask the user if they want to invalidate all of their existing sessions, or invalidate the sessions automatically."; "Don't automatically log the user in"; "Send the user an email informing them that their password has been reset" (R-5).
- Sesi mobile pasien bertahan sampai 30/90 hari (ADR-011 butir 7). Tanpa penghentian sesi, pengambil alih akun (APP-04) tetap memegang sesinya setelah korban mengatur ulang password.

**Rekomendasi (dicatat orkestrator sebagai KEPUTUSAN, lalu masuk AC T-08 AC4d dan T-13 AC4d):**
1. **Atur ulang password lewat Lupa password**: semua sesi pengguna diakhiri otomatis, yaitu sesi Keycloak lewat `POST /admin/realms/{realm}/users/{user-id}/logout` `[SRC-72]` ditambah catatan `session_revocation` di aplikasi (ADR-011 butir 8). Setelah itu pengguna masuk lewat alur biasa.
2. **Ubah password saat login**: opsi "keluar dari perangkat lain" ditampilkan dan tercentang secara bawaan. Sesi yang sedang dipakai tetap berjalan.
3. **Ubah email selesai diverifikasi**: semua sesi lain diakhiri otomatis, karena email adalah kanal pemulihan akun.
4. **Reset MFA oleh admin**: semua sesi diakhiri (sudah ada di H-05).
5. Email pemberitahuan setelah password diatur ulang. Biayanya kecil, sesuai R-5. Butir 6.3.7 sendiri berlevel L3.

Ada dua hal yang belum diketahui dan perlu diverifikasi di F1-T54:
- Apakah alur Lupa password bawaan Keycloak 26.8.2 mengakhiri sesi lain dan apakah alur itu langsung memasukkan pengguna. Saya tidak menemukan halaman resmi yang menyebutnya: `[TIDAK DIKETAHUI]`. Karena itu rekomendasi butir 1 dijalankan oleh backend tanpa bergantung pada perilaku bawaan.
- Bila backend mengetahui atur ulang dari event Keycloak, padahal event hanya diambil setiap menit (ADR-011 butir 10), ada jeda sampai sesi berakhir. Jeda itu perlu diukur dan ditulis di AC.

**Celah V7 lain yang terkait (C-14):**
- 7.5.2 (L2): pengguna dapat melihat dan mengakhiri sesi aktifnya;
- 7.1.2 (L2): jumlah sesi paralel terdokumentasi.

Keduanya belum dirancang (H-01) dan berlaku untuk semua realm.

---

## 6. Pertanyaan 4: pertentangan lain dengan ADR yang diterima

1. **B-01**: bertentangan dengan ADR-011 butir 4 (lihat §2).
2. **Opsi A di §2.4 baris 2 dan 4** ("pesan khusus akun terkunci", dan pesan "akun nonaktif" saat masuk) bertentangan dengan ADR-011 butir 4. Opsi ini sah untuk disajikan, tetapi harus diberi tanda bahwa memilihnya berarti merevisi ADR-011 (C-13).
3. **`4290001` untuk halaman Keycloak** (T-08 AC4c baris 237; KEPUTUSAN-192). Kode ini berasal dari registry envelope API (ADR-023, `docs/05-response-codes.md`), sedangkan halaman Lupa password adalah halaman HTML Keycloak (ADR-011 butir 4, butir 7). Ini ketidaksesuaian kecil (C-08).
4. **Tidak ada pertentangan** di butir-butir berikut:
   - T-09/KEPUTUSAN-193 dengan ADR-011 butir 8;
   - T-13 dengan ADR-011 butir 6 dan butir 9 (juga sesuai 7.5.1 L2);
   - T-19 (b) dan (c) dengan ADR-012 butir 7 (keduanya lebih ketat dan tetap kompatibel);
   - T-21 dengan ADR-010;
   - T-22 dengan ADR-009 butir 5;
   - bootstrap dengan ADR-021 dan ADR-025, dengan catatan C-09 tentang Control Group.
5. **Celah, bukan pertentangan.** Tanpa keputusan S-02 sesuai §5.5, target L2 ADR-011 butir 1 tidak terpenuhi untuk 7.4.3.

---

## 7. Catatan substantif non-blokir

Butir bertanda **(sebelum task)** perlu masuk teks AC atau KEPUTUSAN sebelum task yang disebut dimulai.

| # | Lokasi | Temuan | Usulan | Sumber |
| --- | --- | --- | --- | --- |
| C-01 | S-02 (baris 531); T-08 AC4d (baris 238); T-13 AC4d (baris 329–332) | Penghentian sesi setelah atur ulang password atau ubah email belum diputuskan | Rekomendasi §5.5 **(sebelum F1-T54, F1-T57)** | R-2, R-5, `[SRC-72]` |
| C-02 | §2.4 baris 1 (baris 86); T-04 AC1b (baris 165–170) | Pada opsi B, pendaftaran dengan email terdaftar melewati logika yang dekat dengan kompensasi. Kompensasi yang mencari user berdasarkan email dapat menghapus atau mengubah user milik korban. Persetujuan dari pendaftar baru juga dapat tertulis ke akun yang sudah ada, sehingga bukti persetujuan (E-13) rusak | Tambahkan AC negatif: pendaftaran dengan email terdaftar tidak mengubah, menghapus, atau mengganti kredensial user Keycloak dan akun aplikasi yang ada, dan tidak menulis bukti persetujuan baru. Kompensasi hanya menghapus user berdasarkan ID yang dibuat oleh permintaan itu **(sebelum F1-T50)** | ADR-011 butir 6; threat model (Tampering bukti persetujuan) |
| C-03 | §2.3 P1 (baris 77); ADR-011 butir 6 | Jalur Keycloak yang melewati kontrol UP-73 belum tertutup secara eksplisit: halaman pendaftaran bawaan ("User Registration", R-10) dan perubahan email lewat Keycloak (required action/Update Email, konsol akun) | Konfigurasi realm sebagai kode di F1-T31: User Registration OFF di `pasien` (dan nantinya `nakes`, `admin`, `rme`); pengguna tidak dapat mengubah email lewat Keycloak. Test konfigurasi di CI **(sebelum F1-T31)** | R-10; ASVS 6.3.4 L2 (R-1) |
| C-04 | §2.4 baris 1 dan 7; KEPUTUSAN-192 | (1) Email "Anda sudah punya akun" dan "alamat ini sudah terdaftar" belum dibatasi, sehingga dapat dipakai membanjiri kotak masuk korban dan membebani kuota SES. (2) Batas laju login per IP terhadap credential stuffing belum berangka, padahal 6.1.1 meminta dokumentasi konfigurasi | (1) Pakai batas diam per alamat tujuan seperti kirim ulang verifikasi (1/60 detik, 5/60 menit) ditambah batas per IP. (2) Tetapkan angka batas login per IP di proxy depan Keycloak dan uji risiko banyak pengguna sah di balik satu IP publik **(sebelum F1-T50, F1-T57, F1-T31)** | 6.1.1, 6.3.1 (R-1); R-5 |
| C-05 | §2.5 US-101 AC3 (baris 99) | Untuk email baru, backend menjalankan hash Argon2 dan pembuatan user. Untuk email terdaftar, tidak. Jadi waktu respons tetap berbeda walaupun email dikirim lewat antrean | Proses pembuatan akun secara asinkron setelah validasi sinkron (format, kebijakan password, persetujuan), atau samakan waktunya. Kalau tidak dilakukan, catat sebagai risiko sisa (6.3.8 berlevel L3) | R-5 ("consistent amount of time"); R-1 |
| C-06 | T-06 (baris 199–210); KEPUTUSAN-190 | Keycloak: "The time period that the account is disabled increases as the attack continues"; ada parameter Wait Increment, Max Wait, dan strategi (R-9). Tanpa nilai Max Wait dan strategi, "15 menit" dan "5 berturut-turut" tidak dijamin setelah penguncian pertama | Tetapkan di KEPUTUSAN-190: Wait Increment 15 menit, Max Wait 15 menit, dan strategi yang dipilih. Test F1-T55 mencakup kegagalan setelah penguncian pertama berakhir **(sebelum F1-T31)** | R-9 |
| C-07 | T-02 (baris 136–153); KEPUTUSAN-188 | (1) "Not email" Keycloak hanya memeriksa kesamaan, dan pemeriksaan kata konteks per pengguna tidak tersedia bawaan (R-6). (2) Aturan "memuat bagian lokal email" akan menolak hampir semua password bila bagian lokalnya sangat pendek (misalnya satu huruf). (3) Pemeriksaan kata sandi bocor lewat API eksternal akan melanggar T4-Q1/CR-007. (4) 6.2.2 dan 6.2.3 (L1) belum punya AC | (1)(2) Terapkan aturan kata konteks yang sama di semua jalur, termasuk halaman Keycloak, dan beri panjang minimum bagian lokal sebelum aturan "memuat" berlaku. (3) Daftar bocor berupa berkas lokal; Keycloak mendukung Bloom filter (R-6). (4) Tambah AC ubah password (password lama + baru, autentikasi ulang ADR-011 butir 9) lewat CR kecil. Opsional: tambahkan kata konteks "rumah sehat"/"rumahsehat" (nama lama, CR-016) | R-6; R-1 |
| C-08 | T-08 AC4c (baris 237); T-01 AC2b (baris 126) | `4290001` adalah kode envelope API dan tidak dapat dihasilkan halaman Lupa password Keycloak | Untuk jalur Keycloak, rumuskan sebagai "HTTP 429 dari proxy di depan Keycloak" atau halaman galat bertema. `4290001` tetap untuk endpoint backend (kirim ulang verifikasi) | ADR-023; ADR-011 butir 4 |
| C-09 | T-19 (a) (baris 448), AC4 (baris 454–458) | Bootstrap: (1) "Super Admin aktif" belum mencakup undangan yang belum diaktifkan, sehingga perintah dapat diulang selama undangan menggantung; (2) belum ada penguncian terhadap eksekusi bersamaan; (3) operator dapat memasukkan emailnya sendiri tanpa persetujuan orang kedua; (4) masa berlaku dan sifat sekali pakai undangan belum diatur | Perintah ditolak bila ada Super Admin aktif **atau** menunggu aktivasi; pemeriksaan dan pembuatan dalam satu transaksi berkunci; kredensial perintah hanya terbit lewat Control Group OpenBao (persetujuan orang kedua, ADR-025 `[SRC-253]`); undangan sekali pakai dengan masa berlaku tertulis **(sebelum F1-T65)** | ADR-021 butir 3; ADR-025; 6.4.1 L1 (R-1) |
| C-10 | T-19 (c) (baris 451, 460) | Auditor eksklusif tidak menutup WEB-07/M-16 (Super Admin membuat akun boneka bersub-peran sensitif, termasuk Auditor). Belum ada perlindungan "Auditor terakhir". Verifikator yang juga pasien dapat memutuskan pengajuan identitas akunnya sendiri | Bawa M-16 (pemberitahuan pemberian sub-peran sensitif ke semua Super Admin dan Auditor, serta tinjauan akses berkala) ke Checkpoint 2 sebagai bagian UP-76, atau catat eksplisit sebagai risiko terbuka. Pertimbangkan larangan mencabut Auditor terakhir. Catat risiko konflik kepentingan Verifikator | Threat model WEB-07, M-16; ADR-012 butir 7 |
| C-11 | §2.4 baris 8 (baris 93); T-16 (baris 388–391) | Batas 3/24 jam per akun dapat dilipatgandakan dengan banyak akun berpassword dan berverifikasi email | Tambah batas per IP dan ambang global, beserta peringatan lonjakan penolakan `nik_unavailable` di security log. Pemeriksaan registri hanya saat pengajuan lengkap (NIK + foto), bukan saat NIK diketik | KEPUTUSAN-192; ADR-010 |
| C-12 | F1-T31 (realm `admin`); T-02 | 6.4.3 (L2): atur ulang password "does not bypass any enabled multi-factor authentication mechanisms" (R-1). Halaman alur Keycloak hanya menyebut Reset Credentials berjalan "after email address confirmation" tanpa langkah OTP (R-8) | Verifikasi dan konfigurasikan alur Reset Credentials di realm `admin`, `nakes`, `rme` (dan `pasien` bila opsi M1 atau M2 dipilih) supaya OTP tetap diminta. Masukkan ke test F1-T31/F1-T55 **(sebelum F1-T31)** | R-1, R-8 |
| C-13 | §2.4 baris 2 dan 4, kolom Opsi A | Opsi A bertentangan dengan ADR-011 butir 4 | Tambahkan keterangan "memerlukan revisi ADR-011 butir 4" di kolom Opsi A | ADR-011 butir 4 |
| C-14 | `docs/05-asvs-checklist.md` V7 (H-01) | 7.5.2 (L2, melihat dan mengakhiri sesi) dan 7.1.2 (L2, sesi paralel terdokumentasi) belum dirancang | Bawa ke Checkpoint 2 bersama S-01 dan S-02 sebagai celah L2 ADR-011. Tetapkan fase penerapannya | R-2 |

**Non-blokir kosmetik (maksimal 5).**
1. Baris 71: tambahkan kutipan pengantar V6.3 ("L2 applications must force the use of multi-factor authentication (MFA).") ke catatan SRC-265, supaya S-01 tidak hanya bersandar pada pengantar V6.2.
2. Baris 95: 6.3.7 (L3, pemberitahuan setelah perubahan detail autentikasi) juga relevan untuk email pemberitahuan atur ulang password (§5.5 butir 5).
3. Baris 125: AC2b menyebut "per email". Perjelas apakah batas dihitung per alamat yang dinormalisasi (huruf kecil).
4. §2.5 belum memuat rumusan AC untuk T-08 AC4d terkait sesi. Tambahkan setelah keputusan S-02.
5. §7: SRC baru dari review ini (R-2–R-10) belum tercantum. Tambahkan setelah didaftarkan.

---

## 8. Halaman resmi yang dibuka (2026-11-18), untuk didaftarkan orkestrator ke `docs/sources.md`

Reviewer hanya diizinkan menulis berkas ini, sehingga pendaftaran ke `docs/sources.md` diserahkan ke orkestrator.

| ID | Sumber | URL | Kutipan pendek |
| --- | --- | --- | --- |
| R-1 | OWASP ASVS 5.0.0 V6 (= SRC-265, dibuka ulang) | https://raw.githubusercontent.com/OWASP/ASVS/v5.0.0/5.0/en/0x15-V6-Authentication.md | Pengantar V6.3 "L2 applications must force the use of multi-factor authentication (MFA)."; 6.3.3 (L2) "...must be used in order to access the application." dan "Relaxing any of the considerations in this requirement requires a fully documented rationale and a comprehensive set of mitigating controls."; 6.2.2 (L1) "Verify that users can change their password."; 6.2.3 (L1) "...requires the user's current and new password"; 6.3.4 (L2) "there are no undocumented pathways"; 6.1.3 (L2) jalur autentikasi terdokumentasi; 6.4.3 (L2) "...does not bypass any enabled multi-factor authentication mechanisms."; 6.4.4 (L2) pembuktian identitas saat faktor hilang; pengantar V6.6 "Unsafe out-of-band authentication mechanisms such as e-mail and VOIP are not permitted."; 6.6.1 (L2) PSTN bersyarat. Frasa "combination of single-factor authentication mechanisms" tidak didefinisikan |
| R-2 | OWASP ASVS 5.0.0 V7 | https://raw.githubusercontent.com/OWASP/ASVS/v5.0.0/5.0/en/0x16-V7-Session-Management.md | 7.4.3 (L2) "Verify that the application gives the option to terminate all other active sessions after a successful change", termasuk atur ulang password dan perubahan MFA; 7.5.1 (L2) "requires full re-authentication before allowing modifications to sensitive account attributes" (contohnya email); 7.5.2 (L2) melihat dan mengakhiri sesi aktif; 7.1.2 (L2) jumlah sesi paralel terdokumentasi; 7.4.2 (L1) sesi diakhiri saat akun dinonaktifkan |
| R-3 | OWASP ASVS 5.0.0 What is the ASVS | https://raw.githubusercontent.com/OWASP/ASVS/v5.0.0/5.0/en/0x03-What-is-the-ASVS.md | Level 2 "(all of the L1 and L2 requirements)"; "create an organization- or domain-specific fork that adjusts requirements"; "it is important to maintain traceability" |
| R-4 | OWASP ASVS 5.0.0 Assessment and Certification | https://raw.githubusercontent.com/OWASP/ASVS/v5.0.0/5.0/en/0x04-Assessment_and_Certification.md | "They should also provide an opinion on the rationale of excluding the requirements" |
| R-5 | OWASP Forgot Password Cheat Sheet | https://cheatsheetseries.owasp.org/cheatsheets/Forgot_Password_Cheat_Sheet.html | "Return a consistent message for both existent and non-existent accounts."; "Ensure that responses return in a consistent amount of time"; "rate-limiting on a per-account basis"; "Don't automatically log the user in"; "Ask the user if they want to invalidate all of their existing sessions, or invalidate the sessions automatically."; email pemberitahuan setelah atur ulang |
| R-6 | Keycloak 26.8.2 Server Admin: password policies (adoc, tag 26.8.2) | https://raw.githubusercontent.com/keycloak/keycloak/26.8.2/docs/documentation/server_admin/topics/authentication/password-policies.adoc | "Not email: The password cannot be the same as the email address of the user."; "Password blacklist: Password must not be in a blacklist file." (pencocokan tanpa membedakan huruf besar-kecil, Bloom filter, positif palsu bawaan 0,01%); kebijakan Length dan pemeriksaan kata sandi bocor eksternal tidak terbaca di halaman ini |
| R-7 | Keycloak 26.8.2 Server Admin: WebAuthn (adoc, tag 26.8.2) | https://raw.githubusercontent.com/keycloak/keycloak/26.8.2/docs/documentation/server_admin/topics/authentication/webauthn.adoc | "{project_name} uses WebAuthn for two-factor authentication, but you can use WebAuthn as the first-factor authentication."; wajib bagi semua pengguna atau bersyarat (OTP atau WebAuthn); status dukungan passkey tidak disebut |
| R-8 | Keycloak 26.8.2 Server Admin: authentication flows (adoc, tag 26.8.2) | https://raw.githubusercontent.com/keycloak/keycloak/26.8.2/docs/documentation/server_admin/topics/authentication/flows.adoc | Step-up dengan `acr_values` dan sub-flow "Conditional - Level Of Authentication"; "The Browser - Conditional 2FA sub-flow executes only when the user has a configured OTP credential."; Reset Credentials "where users can reset credentials after email address confirmation" (langkah OTP tidak disebut) |
| R-9 | Keycloak 26.8.2 Server Admin: brute force (adoc, tag 26.8.2; halaman SRC-262 pada tag versi) | https://raw.githubusercontent.com/keycloak/keycloak/26.8.2/docs/documentation/server_admin/topics/threat/brute-force.adoc | "The time period that the account is disabled increases as the attack continues."; Max Wait "The maximum time a user is temporarily disabled."; strategi kelipatan dan linear; "displays the default `Invalid username or password` error message"; berlaku "only to password, OTP and recovery codes" |
| R-10 | Keycloak Server Admin Guide 26.8.2 (halaman SRC-70) | https://www.keycloak.org/docs/latest/server_admin/index.html | Versi halaman 26.8.2; "Toggle **User Registration** to **ON**", lalu "a **Register** link displays on the login page"; nilai bawaan tidak disebut di bagian yang terbaca |

Yang dicari tetapi tidak ditemukan di sumber resmi: apakah alur Lupa password Keycloak 26.8.2 mengakhiri sesi lain atau langsung memasukkan pengguna, dan pesan bawaan Keycloak untuk akun nonaktif. Hasil pencarian hanya menunjuk forum dan diskusi komunitas, yang tidak valid sebagai sumber. Keduanya tetap `[TIDAK DIKETAHUI]` dan diverifikasi di F1-T54/F1-T55.

---

## 9. Langkah berikutnya

1. Penyusun memperbaiki B-01 (baris 89 dan 101–102). Cek ulang cukup memeriksa baris yang berubah.
2. S-01 dibawa ke Checkpoint 2 sebagai pertanyaan terpisah dengan opsi M1–M4 (§3.4), bersama C-14. Keputusan apa pun memerlukan revisi ADR-011.
3. Orkestrator mencatat keputusan S-02 (§5.5) sebagai KEPUTUSAN, mendaftarkan R-2–R-10 ke `docs/sources.md`, dan menambahkan kutipan tambahan R-1 ke SRC-265.
4. Catatan bertanda "sebelum task" dimasukkan ke AC atau KEPUTUSAN terkait setelah Checkpoint 2, sebelum F1-T31, F1-T50, F1-T54, F1-T57, dan F1-T65 dimulai.

---

## 10. Cek ulang putaran 2 (terakhir)

Cakupan: hanya baris yang berubah untuk menutup blokir dan memeriksa tindak lanjut catatan (`aturan-kerja.md` §2), bukan review penuh. Yang diperiksa:
- `docs/07-klarifikasi-ac-fase-1.md` v0.2: §2.3–§2.7, T-02 (baris 188), T-06 (baris 261), T-08 AC4d/AC4e (baris 302–308), T-13 (baris 395–401), T-19 (baris 519–535), §5, §5.1, §5.2, §6;
- `docs/07-catatan-phase-plan.md`: pertanyaan Checkpoint 2 nomor 5, 13, dan 14;
- `docs/decision-log.md`: KEPUTUSAN-188, -190, -192, dan -198 sampai -201;
- `docs/sources.md`: SRC-265 sampai SRC-274;
- pembanding: `docs/03-prd.md` NFR-PERF-02 (baris 1601) dan `docs/adr/ADR-005-audit-trail-append-only.md` baris 13.

### 10.1 B-01: **tertutup**

- §2.3 P1 (baris 84) mewajibkan satu pesan seragam yang identik kata per kata untuk password salah, akun tidak ada, terkunci, dan nonaktif, tanpa kalimat tambahan.
- Pesan untuk akun terkunci dan nonaktif sama dengan pesan seragam itu: §2.4 butir 2 (baris 94) dan butir 4 (baris 96). Kalimat Layanan Pelanggan kini menjadi bagian tetap pesan seragam.
- US-103 AC2 dan KU-10 di §2.5 (baris 107–111) sudah menyatakan hal yang sama, termasuk "tanpa kalimat atau tanda tambahan apa pun".
- Uji teks persis untuk keempat kondisi di empat realm sudah ditulis untuk F1-T55 (baris 112).
- Opsi A sekarang diberi tanda "Memerlukan revisi ADR-011 butir 4", sehingga C-13 juga tertutup.

### 10.2 Dua penyimpangan dari saran putaran 1

**(a) C-05: penyamaan waktu 900 ms, sebagai ganti pembuatan akun asinkron (§2.7 butir 4, KEPUTUSAN-201). Dapat diterima, dengan catatan.**
- **Kesimpulannya benar, rujukannya kurang tepat.** ADR-005 baris 13 melarang password "masuk log", sedangkan muatan job bukan log. Meski begitu, menyimpan password teks asli di tabel antrean tetap tidak boleh: kredensial semestinya hanya tersimpan sebagai hash Argon2 di Keycloak (ADR-011 butir 4). Saran saya, rumuskan alasannya sebagai larangan menyimpan kredensial teks asli at rest, dengan ADR-005 sebagai pendukung. Ini non-blokir.
- **Konsistensi dengan NFR-PERF-02.** NFR-PERF-02 menetapkan p95 ≤ 1 detik untuk API tulis (`docs/03-prd.md` baris 1601). Batas 900 ms memang di bawah target itu, tetapi hanya menyisakan 100 ms. Bila jalur pembuatan akun di atas 900 ms, dua hal terjadi bersamaan: selisih waktu kembali terlihat, dan p95 pendaftaran dapat melewati 1 detik.
- **Catatan P2-1 (non-blokir, F1-T50).** Uji yang sudah ditulis (≥ 100 permintaan per kasus, median dan p95) juga harus melaporkan p95 jalur email baru tanpa penahanan. Bila hasilnya di atas 900 ms, pilihannya (1) menaikkan batas dengan pengecualian NFR-PERF-02 yang dicatat untuk endpoint pendaftaran, atau (2) menerima selisih sebagai risiko sisa. Keduanya dibawa sebagai keputusan, tidak diputuskan diam-diam di kode.

**(b) Aturan "memuat bagian lokal email" dihapus (T-02 baris 188, KEPUTUSAN-188). Dapat diterima.**
- ASVS 6.2.11 dan 6.1.2 (L2) mensyaratkan daftar kata konteks yang terdokumentasi. Contoh di 6.1.2 adalah nama organisasi, produk, dan sistem; bagian lokal email bukan syarat butir tersebut `[SRC-265]`.
- Daftar kata konteks tetap ada dan sudah ditambah "rumah sehat"/"rumahsehat". "Not email" tetap aktif `[SRC-270]`.
- Kebijakan yang sama berlaku di semua jalur, sesuai 6.1.3 dan 6.3.4.
- Ketiga daftar (kata sandi umum, bocor, kata konteks) berupa berkas lokal, sehingga tidak bertentangan dengan T4-Q1.

### 10.3 Pencerminan catatan lain

Tidak ada yang salah secara substantif.

- **S-01** (§5.1; pertanyaan nomor 14 berlabel "Perubahan ADR yang sudah diterima (ADR-011)"): kutipan ketiga teks ASVS, opsi M1–M4, tabel faktor kedua, dan posisi angka 15 karakter cocok dengan §3. Penyusun tidak memberi default. Satu catatan kecil: kalimat "ASVS tidak membahas level berbeda per komponen" di baris 646 sebaiknya diberi label `[SRC-267]` `[SRC-268]` dan dibatasi pada halaman yang dibuka (non-blokir).
- **UP-78** (§5.2): 7.5.2 dan 7.1.2 sudah disajikan sebagai opsi untuk user, dan opsi (b)/(c) diberi tanda tidak memenuhi L2 penuh. Sudah benar.
- **KEPUTUSAN-198**: sesuai §5.5. Jeda paling lama 2 menit sudah tertulis dan dapat diuji. Pengguna juga tidak otomatis masuk setelah atur ulang.
- **KEPUTUSAN-199**: sesuai C-12. Bila alur Reset Credentials ternyata tidak dapat dikonfigurasi supaya meminta OTP, temuannya wajib dibawa ke user sebelum F1-T31 ditutup.
- **Bootstrap Super Admin** (T-19 baris 519–533): keempat kontrol C-09 masuk, yaitu Control Group (persetujuan orang kedua), undangan yang menunggu aktivasi ikut dihitung, transaksi berkunci, dan undangan sekali pakai 24 jam. Audit juga mencatat identitas penyetuju. Butir (d) dengan opsi M-16 dan larangan mencabut Auditor terakhir menjawab C-10.
- **Catatan lain yang juga tertutup**: C-03, C-04, C-06, C-08, C-11 (dengan batas per NIK lintas akun, lebih kuat dari usulan saya), dan C-14.

### 10.4 SRC-266 sampai SRC-274: **sesuai**

URL dan kutipan SRC-266 sampai SRC-274 cocok dengan R-2 sampai R-10 di §8. Kutipan tambahan di SRC-265 cocok dengan R-1. Tidak ada kutipan yang melebihi isi halaman yang saya buka.

### 10.5 Verdict akhir

**LULUS DENGAN CATATAN.** Blokir: nol. Catatan non-blokir yang tersisa:
- **P2-1**: margin 900 ms terhadap NFR-PERF-02 diukur di F1-T50 (§10.2 a).
- Alasan penolakan jalur asinkron dirumuskan sebagai larangan menyimpan kredensial teks asli at rest.
- Label sumber untuk baris 646.

Ketiganya tidak memicu putaran review baru. S-01 dan UP-78 tetap menjadi keputusan user di Checkpoint 2.
