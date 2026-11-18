# Review Keamanan Tindak Lanjut Checkpoint 2: Revisi ADR-011 (MFA pasien, pemulihan faktor, sesi)

Tanggal: 2026-11-18
Reviewer: security-architect (independen, tidak ikut menyusun dokumen yang direview)
Tingkat ketelitian: T1 Kritis (autentikasi, sesi, MFA; `aturan-kerja.md` §1), 2 putaran (maksimal 2)
**Status akhir (putaran 2): LULUS DENGAN CATATAN.** B-01 tertutup di dokumen yang direview. Satu sisa wajib ada di test plan dan dikerjakan pada langkah pembaruan test case oleh QA. Rinciannya di §7.
Keputusan user yang menjadi acuan: `docs/checkpoint-2.md` bagian "Jawaban user" (C2-Q9, C2-Q10, C2-Q11, C2-B4, C2-B7, C2-B14) `[USER]`
Dokumen yang direview (branch `checkpoint-2`, belum di-commit):
- `docs/adr/ADR-011-target-asvs-autentikasi-sesi-mfa.md` (revisi Checkpoint 2)
- `docs/03-prd.md` v1.2: KU-10, US-101, US-103, US-105, US-106, US-110, US-201 AC8, US-207 AC5, US-208, NFR-SEC-02/03/08/11
- `docs/05-asvs-checklist.md` v1.1 (V6, V7)
- `docs/04-risk-register.md` v1.3 (RT-35–RT-37, catatan status ancaman)
- `docs/decision-log.md` KEPUTUSAN-202–205
- Putaran 2: baris yang berubah untuk B-01 (§7)

Metode: kriteria blokir mengikuti `aturan-kerja.md` §2. Teks requirement ASVS 5.0.0 V6 dan V7 saya buka sendiri hari ini di tag resmi v5.0.0 (URL yang sama dengan SRC-265/SRC-276 dan SRC-266/SRC-277). Kutipan yang saya pakai sudah tercatat di SRC tersebut, sehingga tidak ada SRC baru. Nomor baris di §1–§6 merujuk ke berkas di working tree saat putaran 1; nomor baris di §7 merujuk ke working tree saat putaran 2.

---

## 1. Verdict

**TIDAK LULUS (putaran 1): 1 blokir (B-01).** Perbaikannya cukup di tingkat teks satu butir AC, dan cek ulang cukup memeriksa baris yang berubah. (Hasil putaran 2 di §7.)

- **B-01.** US-207 AC5 mengizinkan Super Admin memulai reset MFA pasien. ADR-011 butir 5, KEPUTUSAN-204, checklist ASVS, dan RT-36 hanya menyebut Layanan Pelanggan. Pencatatan E-04 di AC yang sama juga hanya menyebut aktor Layanan Pelanggan.
- Di luar B-01, revisi memenuhi klaim ASVS 5.0.0 L2 yang diperiksa (§2) di tingkat desain. Perilaku Keycloak yang belum terbukti ditandai jujur sebagai `[TIDAK DIKETAHUI]`, lengkap dengan task verifikasi dan jalur eskalasi ke user (AC4e, KEPUTUSAN-203, KEPUTUSAN-204).
- Ada 5 catatan non-blokir (§4). C-01 dan C-02 perlu masuk teks AC atau ADR **sebelum task US-207 AC5 dimulai (Fase 2)**.
- `docs/04-threat-model.md` diperbarui ke v1.1 sesuai tabel perubahan status di risk register v1.3. Status threat model tetap LULUS (§5).

---

## 2. Pemeriksaan ASVS 5.0.0 L2 yang diklaim

| Requirement (level) | Teks resmi (ringkas) | Tempat di desain | Penilaian |
| --- | --- | --- | --- |
| 6.3.3 (L2) | MFA atau kombinasi faktor tunggal "must be used in order to access the application" `[SRC-265]`; faktor perangkat keras tahan phishing baru wajib di L3 `[SRC-276]` | ADR-011 butir 3 (baris 51), butir 5 (baris 69–75); NFR-SEC-02 (PRD baris 1796); US-101 AC5 (baris 479–485); US-103 AC1, AC1b (baris 496–498) | **Terpenuhi di desain.** OTP diwajibkan untuk semua pengguna, bukan memakai alur bersyarat bawaan Keycloak `[SRC-70]` `[SRC-272]`. Penutupan langkah TOTP tidak memberi akses ke beranda (AC5 butir 4). Pembuktiannya ada di test CI-04 (MFA wajib) |
| 6.4.3 (L2) | Atur ulang password "does not bypass any enabled multi-factor authentication mechanisms" `[SRC-276]` | ADR-011 baris 74; US-103 AC4e (baris 517); KEPUTUSAN-199 | **Terpenuhi di desain.** Dokumentasi Keycloak hanya menyebut atur ulang "after email address confirmation" `[SRC-272]`. Kemampuan konfigurasinya belum terbukti, dan bila tidak dapat dikonfigurasi, temuannya dibawa ke user sebelum task ditutup. Ini benar: tanpa itu, 6.4.3 gagal untuk semua realm |
| 6.4.4 (L2) | "evidence of identity proofing is performed at the same level as during enrollment" `[SRC-276]` | ADR-011 baris 76–85; US-103 AC8 (baris 533–536); US-207 AC5 (baris 748–756); KEPUTUSAN-204 | **Terpenuhi secara harfiah.** Pembuktian saat pendaftaran pasien adalah kendali atas email (verifikasi email). Jalur Layanan Pelanggan meminta kendali email ditambah password, jadi setara atau lebih kuat. Dua kelemahan rancangan dicatat di C-01 dan C-02. Kasus semua faktor hilang (RT-37) jujur ditandai `[TIDAK DIKETAHUI]` |
| 6.2.2, 6.2.3 (L1) | Pengguna dapat mengubah password, dan ubah password "requires the user's current and new password" `[SRC-276]` | US-103 AC6 (baris 519–525), CR-017 `[USER] C2-B14` | **Terpenuhi di desain.** Mekanisme Keycloak `[TIDAK DIKETAHUI]`, diverifikasi Tahap 8 |
| 7.1.2 (L2) | Dokumentasi menetapkan jumlah sesi paralel dan perilaku saat batas tercapai `[SRC-277]` | NFR-SEC-11 (PRD baris 1805); ADR-011 butir 12 (baris 131–134); checklist baris 212 | **Terpenuhi.** "Tidak dibatasi" adalah nilai yang terdokumentasi `[USER] C2-Q11`. Karena tidak ada batas, tidak ada perilaku saat batas tercapai, dan kompensasinya tercatat |
| 7.4.3 (L2) | Opsi mengakhiri semua sesi lain "after a successful change or removal of any authentication factor", termasuk atur ulang dan pembaruan MFA `[SRC-266]` | US-103 AC4d, AC6, AC7; US-105 AC4d; US-207 AC5; KEPUTUSAN-198 | **Terpenuhi di desain.** Atur ulang password, ubah email, dan reset MFA mengakhiri semua sesi. Ubah password dan daftar ulang TOTP menampilkan opsi yang tercentang bawaan |
| 7.5.2 (L2) | Pengguna dapat melihat sesi aktif dan, "having authenticated again with at least one factor", mengakhiri satu atau semua sesi `[SRC-277]` | US-110 (baris 632–654); ADR-011 butir 9 dan 12; KEPUTUSAN-205; endpoint `[SRC-275]` | **Terpenuhi di desain, Fase 2.** Build Fase 1 belum memenuhinya, dan checklist baris 222 menyatakannya dengan benar. US-110 AC5 menolak ID sesi milik akun lain (KU-03) |

Pemeriksaan tambahan, tidak diklaim tetapi relevan:
- **7.5.1 (L2).** Autentikasi ulang penuh sebelum mengubah atribut yang memengaruhi autentikasi `[SRC-266]`. Terpenuhi untuk ubah email (US-105 AC4a), ubah password (US-103 AC6), dan daftar ulang TOTP (AC7).
- **6.3.4 (L2).** Tidak boleh ada jalur autentikasi yang tidak terdokumentasi `[SRC-265]`. Pemeriksaan password di langkah konfirmasi US-207 AC5 adalah jalur baru (C-02).

---

## 3. Blokir

### B-01 · `docs/03-prd.md` baris 754–755 (US-207 AC5) · kriteria §2 butir 3 dan 4

**Temuan.** Butir terakhir US-207 AC5 berbunyi "sub-peran selain Layanan Pelanggan **dan Super Admin** tidak dapat memulai reset MFA pasien". Artinya Super Admin juga berhak memulai reset. Dokumen lain dalam paket revisi yang sama membatasi hak itu ke Layanan Pelanggan:
- ADR-011 baris 78–79 ("Jalur pasien lewat Layanan Pelanggan ... Layanan Pelanggan memulai reset MFA di backoffice");
- KEPUTUSAN-204 butir 2 (`docs/decision-log.md` baris 221);
- `docs/05-asvs-checklist.md` baris 196–197 ("untuk pasien oleh Layanan Pelanggan");
- RT-36 (`docs/04-risk-register.md` baris 52).

AC5 sendiri juga tidak konsisten: E-04 dicatat "dengan aktor Layanan Pelanggan" (baris 754), padahal Super Admin dapat menjadi pemula.

**Mengapa blokir.** Ini perbedaan peran pada modul T1 (autentikasi dan otorisasi), bukan soal rumusan. User memilih M1 dengan konsekuensi yang disajikan: "beban Layanan Pelanggan bertambah dan prosedur H-05 meluas ke pasien" (`docs/07-klarifikasi-ac-fase-1.md` baris 643; `[USER] C2-Q9`). Penambahan Super Admin tidak dibawa ke user dan tidak dicatat sebagai KEPUTUSAN.

**Penilaian risiko perluasan hak.** Risiko tambahannya **Rendah–Sedang**, bukan Tinggi:
- Reset tetap baru berlaku setelah konfirmasi lewat email akun ditambah password. Jadi Super Admin yang berniat jahat atau akunnya terkompromi tidak dapat mengambil alih akun pasien hanya dengan memulai reset.
- Risiko yang tetap ada:
  - (a) Pemisahan tugas melemah. Super Admin sudah memegang administrasi peran (US-208), dan kini ditambah pemulihan kredensial pengguna.
  - (b) Akun Super Admin yang terkompromi dapat memicu email konfirmasi reset ke banyak pasien sekaligus. Email itu bisa menjadi umpan rekayasa sosial, dan kombinasinya dengan C-01 memperbesar peluang.
  - (c) Atribusi E-04 salah.
- Super Admin juga tidak memerlukan hak ini untuk keadaan darurat. Ia dapat meminta Super Admin lain memberi sub-peran Layanan Pelanggan kepada staf. Memberi peran ke diri sendiri sudah ditolak oleh US-208 AC3.

**Cara mereproduksi (tingkat teks).** Bandingkan PRD baris 755 dengan ADR-011 baris 79 dan decision-log baris 221. Peran yang boleh memulai reset berbeda.

**Perbaikan (pilih salah satu).**
- **(a) Disarankan:** ubah butir itu menjadi "sub-peran selain Layanan Pelanggan tidak dapat memulai reset MFA pasien (KU-03)". Tidak perlu keputusan baru karena sudah sesuai pilihan user.
- **(b)** Bila Super Admin memang diinginkan sebagai cadangan, catat KEPUTUSAN baru dengan alasannya dan bawa ke user sebagai perubahan hak peran di gate berikutnya. Setelah itu samakan ADR-011 butir 5 langkah 1, KEPUTUSAN-204 butir 2, checklist baris 196–197, dan RT-36, lalu ubah E-04 menjadi "aktor = admin yang memulai, beserta sub-perannya".

---

## 4. Catatan non-blokir (5 terpenting)

**C-01 · Jendela tanpa MFA setelah reset lewat Layanan Pelanggan · ADR-011 baris 83; US-207 AC5 baris 753 · sebelum task US-207 AC5.**
Setelah pasien mengonfirmasi, MFA dihapus, dan "TOTP baru wajib didaftarkan saat masuk berikutnya". Selama jendela itu, siapa pun yang tahu password (misalnya dari credential stuffing) dapat masuk lebih dulu dan mendaftarkan TOTP miliknya. Pemilik sah lalu terkunci, dan untuk sementara akun hanya dilindungi satu faktor, berlawanan dengan maksud 6.3.3 `[SRC-265]`.
Usulan:
- (a) Langkah konfirmasi langsung berlanjut ke pendaftaran TOTP baru di peramban yang sama. TOTP lama baru diganti, dan reset baru dianggap berlaku, setelah TOTP baru dan kode pemulihan baru tersimpan. Dengan begitu tidak ada keadaan akun berpassword tanpa MFA.
- (b) Pemberitahuan permintaan reset dikirim saat reset dimulai, bukan hanya setelah selesai, ke email akun dan ke sesi aktif lain bila ada, dengan tautan pembatalan.
- (c) Hanya satu permintaan reset yang tertunda per akun. Permintaan baru membatalkan tautan sebelumnya.
Dicatat sebagai H-16 di threat model.

**C-02 · Pemeriksaan password di langkah konfirmasi adalah jalur autentikasi baru · ADR-011 baris 81; US-207 AC5 baris 750 · sebelum task US-207 AC5.**
Mekanismenya `[TIDAK DIKETAHUI]`, dan itu jujur. Dari sisi keamanan, rancangan Tahap 8 wajib memenuhi syarat berikut:
- Jalur ini didokumentasikan sebagai jalur autentikasi (6.3.4 L2 `[SRC-265]`).
- Kegagalannya dihitung ke penguncian yang sama dengan login. Deteksi brute force Keycloak hanya berlaku untuk "password, OTP and recovery codes" di alurnya sendiri `[SRC-273]`, sehingga pemeriksaan di luar alur Keycloak tidak otomatis terlindungi.
- Jalur ini tidak memakai grant password langsung pada klien publik, karena ADR-011 menolaknya (Alternatif ditolak, butir 2).
- Hasil benar dan salah tidak dapat dibedakan pihak ketiga.
Masukkan syarat ini ke ADR jalur email (KEPUTUSAN-183) atau ADR baru yang dijanjikan KEPUTUSAN-204.

**C-03 · Penyalahgunaan jalur Layanan Pelanggan · US-207 AC5; RT-36 · prosedur H-05.**
Prosedur H-05 untuk pasien sebaiknya memuat kontrol berikut:
- batas jumlah reset per staf per hari, dengan peringatan security log bila terlampaui;
- reset atas akun milik staf sendiri ditolak, sejalan dengan pola US-201 AC8;
- staf tidak mengisi atau mengubah alamat tujuan email. Tautan selalu ke email akun, dan saat ini memang tidak ada jalur admin untuk mengubah email pasien (hanya US-105 AC4). Kondisi ini perlu dipertahankan sebagai larangan tertulis.
Untuk realm `nakes`, `admin`, dan `rme`, "sub-peran berwenang" pada H-05 (ADR-011 baris 84) masih belum disebut namanya.

**C-04 · Syarat go-live untuk butir yang baru terpenuhi di fase berikutnya · checklist baris 222; KEPUTUSAN-217, KEPUTUSAN-218.**
7.5.2 (US-110) dan jalur Layanan Pelanggan 6.4.4 (US-207 AC5) baru dibangun di Fase 2, sedangkan web RME menyusul di Fase 3. Klaim L2 untuk rilis production bergantung pada ketiganya. Tambahkan ke checklist go-live, dengan rujukan ke checklist ASVS V6/V7, supaya tidak tertinggal bila gerbang re-plan 2026-12-11 memangkas scope.

**C-05 · Autentikasi ulang pemula reset dan atribusi audit · ADR-011 butir 9 (baris 115); US-207 AC5 baris 754.**
Butir 9 menyebut "reset MFA" sebagai aksi yang memerlukan autentikasi ulang, tetapi tidak jelas apakah yang dimaksud admin pemula, pasien, atau keduanya. Usulan:
- admin pemula mengautentikasi ulang di backoffice;
- E-04 mencatat dua kejadian, yaitu "reset dimulai" (aktor admin, alasan) dan "reset dikonfirmasi" (aktor pasien), supaya setiap reset dapat dilacak dari awal sampai akhir dalam tinjauan RT-36 butir 5.

---

## 5. Pembaruan threat model

`docs/04-threat-model.md` diperbarui ke v1.1 sesuai tabel "Catatan status ancaman threat model (v1.3, Checkpoint 2)" di risk register (baris 63–74):

| Butir | Perubahan di threat model v1.1 |
| --- | --- |
| APP-04 | Mitigasi ditambah MFA TOTP wajib di realm `pasien`. Risiko sisa "pasien tanpa MFA" tertutup. Risiko sisa baru: phishing seketika atas password dan TOTP, serta rekayasa sosial Layanan Pelanggan (RT-36) |
| API-10 / M-12 | Sebagian → Tercermin. M-12 diterapkan (UP-73 opsi B). Sisa: selisih waktu respons (6.3.8 L3) |
| KC-02 / H-05 | Mitigasi diperluas ke jalur pasien (US-207 AC5, KEPUTUSAN-204). H-05 berstatus Sebagian diterapkan |
| KC-10 (baru, Tinggi) | Pengambilalihan akun pasien lewat jalur pemulihan MFA Layanan Pelanggan. Tercermin, dengan penguatan H-16 (C-01, C-02, C-03) |
| KC-11 (baru, Sedang) | Melihat atau mengakhiri sesi milik akun lain lewat US-110. Tercermin (US-110 AC5) |
| H-01, H-04 | Sebagian diterapkan |
| WEB-07 / M-16 | Sebagian → Tercermin. M-16 diterapkan lewat US-208 AC6–AC8, dengan frekuensi tinjauan `[TIDAK DIKETAHUI]` (PT-55) |
| §7 masukan DPIA | "Pasien tanpa MFA" diganti dengan phishing seketika TOTP, rekayasa sosial Layanan Pelanggan, dan RT-37 |

Status threat model tetap **LULUS**: ke-55 ancaman Tinggi berstatus Tercermin. Pembaruan topologi VPS (usulan U-09 di checklist ASVS) **tidak** tercakup revisi ini dan tetap terbuka.

---

## 6. Yang perlu dilakukan pelaksana

1. Perbaiki B-01 (opsi a disarankan). Cek ulang cukup pada PRD baris 755 (dan baris 754 bila opsi b dipilih).
2. Masukkan C-01 dan C-02 ke KEPUTUSAN-204 dan US-207 AC5, atau ke ADR jalur email, sebelum task US-207 AC5 di Fase 2.
3. C-03–C-05 masuk prosedur H-05 dan checklist go-live, tanpa putaran review baru.

---

## 7. Putaran 2 (cek ulang B-01)

Cakupan: baris yang berubah untuk B-01, ditambah pencarian sisa penyebutan Super Admin untuk reset MFA pasien di `docs/`. Pencarian memakai pola teks "reset MFA" dan "Super Admin" di seluruh `docs/`. `git diff main --stat` tidak dapat saya jalankan karena reviewer ini tidak punya akses shell, sehingga pencarian dilakukan atas semua berkas di `docs/`, yang mencakup berkas yang diubah di branch.

### 7.1 Baris yang berubah

| Berkas dan baris | Isi sekarang | Penilaian |
| --- | --- | --- |
| `docs/03-prd.md` baris 755 (US-207 AC5, butir terakhir) | "sub-peran selain Layanan Pelanggan, termasuk Super Admin, tidak dapat memulai reset MFA pasien (KU-03)", beserta alasan dan rujukan B-01 | **Sesuai perbaikan (a).** Kini selaras dengan ADR-011 baris 79, KEPUTUSAN-204 butir 2, checklist baris 196–197, dan RT-36. E-04 "dengan aktor Layanan Pelanggan" (baris 754) kini konsisten |
| `docs/06-design-guideline.md` baris 434 (D-C05 langkah 4) | "detail akun pasien (hanya Layanan Pelanggan): tombol 'Reset kode masuk (MFA)' ... tidak ada kolom untuk melihat atau mengisi kredensial" | **Sesuai.** Selaras dengan US-207 AC5 dan 6.4.6 `[SRC-276]` |

**B-01 tertutup** untuk dokumen yang direview (PRD, ADR-011, decision log, checklist ASVS, risk register, design guideline).

### 7.2 Sisa yang ditemukan

**S-1 · `docs/test-plan.md` baris 639 (TC-US-207-15) · wajib diperbaiki pada langkah pembaruan test case oleh QA.** Kasus uji ini masih menyatakan "serta Super Admin; ... T: lima sub-peran pertama ditolak 403 tanpa tautan terkirim; **Super Admin berhasil**". Itu bertentangan dengan US-207 AC5 baris 755 yang sudah diperbaiki. Kalau dibiarkan, test T1 akan menuntut perilaku yang salah dan meloloskan perluasan hak yang ditolak di B-01.

Perbaikannya: ubah hasil yang diharapkan menjadi "keenam sub-peran, termasuk Super Admin, ditolak 403 tanpa tautan terkirim".

S-1 tidak saya jadikan blokir putaran ini, dengan dua alasan:
- `docs/test-plan.md` bukan bagian dokumen yang direview.
- `docs/checkpoint-2.md` tindak lanjut 4 menempatkan "pembaruan test case oleh QA" sebagai langkah tersendiri setelah review keamanan ini.

S-1 **wajib** selesai di langkah itu. Bila masih ada saat fact-check, statusnya blokir (kriteria §2 butir 3).

**S-2 · `docs/04-threat-model.md` baris 258 (KC-10, kolom Status) dan baris 562 (§7 butir 6) · T3.** Kedua catatan masih menyebut peran pemula reset "belum konsisten" dan merujuk B-01 sebagai blokir terbuka. Status KC-10 tetap Tercermin, dan threat model tetap LULUS. Teks itu cukup diperbarui menjadi "B-01 tertutup di putaran 2" pada pembaruan threat model berikutnya. Berkas itu tidak diubah di putaran ini sesuai instruksi.

Pencarian tidak menemukan penyebutan lain bahwa Super Admin berhak memulai reset MFA pasien. Penyebutan Super Admin di `docs/07-phase-plan.md` baris 341 (F1-T65) dan `docs/test-plan.md` baris 470 menyangkut bootstrap dan pengelolaan Super Admin (US-208), bukan reset MFA pasien.

### 7.3 Verdict putaran 2

**LULUS DENGAN CATATAN.**
- 0 blokir pada dokumen yang direview.
- S-1 wajib diperbaiki QA sebelum fact-check.
- S-2 pembaruan teks T3.
- Catatan C-01–C-05 (§4) tetap non-blokir. C-01 dan C-02 masuk AC atau ADR sebelum task US-207 AC5 di Fase 2.
