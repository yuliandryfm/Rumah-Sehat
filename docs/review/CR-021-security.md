# Review Keamanan CR-021: situs publik, member area korporat, data peserta, invoice B2B

Tanggal: 2026-11-19
Reviewer: security-architect (independen, tidak ikut menyusun dokumen yang direview)
Tingkat ketelitian: T1 Kritis (autentikasi, isolasi tenant, data pribadi spesifik, uang; `aturan-kerja.md` §1), 2 putaran (maksimal 2)
**Status akhir (putaran 2): LULUS DENGAN CATATAN.** B-01–B-05 tertutup. Lampiran A sudah diterapkan dengan benar ke threat model v1.3. Status ancaman yang harus diubah ada di §5.3. Lingkup CR-021 di threat model baru menjadi LULUS setelah user memutuskan UP-83/PT-78/UP-87 (KOR-12) dan UP-84 (B2B-01); tidak ada lagi celah desain yang menjadi milik penyusun. Rinciannya di §5.
Dokumen yang direview (branch `tahap-08-fase-1`, commit `9530155`):
- `docs/cr/CR-021-analisis.md` v0.1
- `docs/adr/ADR-027-situs-publik-dan-member-area-korporat.md` (Diusulkan)
- `docs/adr/ADR-028-tenant-perusahaan-dan-data-peserta.md` (Diusulkan)
- `docs/adr/ADR-029-invoice-b2b-tanpa-payment-gateway.md` (Diusulkan)
- `docs/decision-log.md` KEPUTUSAN-250–263 (baris 252–265)

Acuan: keputusan user `[DOK: cr/CR-021-wawancara.md]` (P0-1–P3-4); ADR-011 (revisi Checkpoint 2), ADR-012, ADR-013, ADR-015, ADR-018, ADR-021, ADR-024, ADR-025; `docs/04-threat-model.md` v1.2; `docs/05-asvs-checklist.md`.

Metode:
- Kriteria blokir `aturan-kerja.md` §2. Untuk ancaman keamanan dipakai juga kriteria kelulusan threat model §0.4 `[KEPUTUSAN-70]`: ancaman **Tinggi** yang mitigasinya belum tercermin di desain yang diusulkan membuat lingkup itu TIDAK LULUS. Ini sama dengan preseden `docs/review/tahap-05-security.md` B-01/B-02.
- Halaman yang saya buka sendiri hari ini dan dicatat di Lampiran B (usulan SRC-405–SRC-409): OWASP XML Security Cheat Sheet, OWASP XXE Prevention Cheat Sheet, OWASP CSV Injection, MDN `Set-Cookie` (prefix `__Host-`), dan OWASP File Upload Cheat Sheet (dibuka ulang, melengkapi SRC-50).
- Reproduksi temuan: setiap temuan menyebut berkas dan baris. Cara memeriksanya adalah membaca baris itu dan mencari kontrol yang disebut. Ketiadaan kontrol itulah temuannya.

Nomor baris di §1–§4 merujuk ke working tree saat putaran 1. Nomor baris di §5 merujuk ke working tree saat putaran 2.

---

## 1. Verdict

**TIDAK LULUS (putaran 1): 5 blokir (B-01–B-05).** (Hasil putaran 2 di §5.) Semuanya dapat ditutup di tingkat teks ADR/AC tanpa mengubah arah desain. Cek ulang cukup memeriksa baris yang berubah.

Arah desain utama **aman dan saya dukung**:
- Realm `korporat` terpisah (KEPUTUSAN-250) lebih baik daripada realm `admin` (pendaftaran anonim di realm Super Admin) atau `pasien` (batas sesi 30/90 hari). MFA TOTP wajib, pesan seragam, dan atur ulang password ber-OTP sudah mengikuti ADR-011.
- Pemisahan `web/situs` tanpa sesi dari `web/korporat` ber-BFF (KEPUTUSAN-251) mempersempit dampak RCE Next.js `[SRC-246]`.
- Isolasi tenant memakai `CorporateScope` dari sesi, respons 404, dan test isolasi otomatis (KEPUTUSAN-254), konsisten dengan ADR-012 butir 5.
- Data peserta dipisah dari `care_recipient`, tanpa penautan otomatis lewat email, dengan token undangan yang di-hash (KEPUTUSAN-255).
- Ringkasan hasil hanya dibuka lewat fungsi akses sensitif yang memeriksa persetujuan saat baca dan menulis E-05, dengan wadah enkripsi sendiri (KEPUTUSAN-257, KEPUTUSAN-258). Admin Ruang Sehat tidak punya jalur baca.
- Invoice immutable, harga terkunci saat konfirmasi, jurnal berpasangan, dan AB-83 ditegakkan server (KEPUTUSAN-259).

Celah yang tersisa: jalur pemulihan MFA HR, kanal turunan yang dapat membocorkan status kesehatan ke HR, pengikatan undangan ke orang yang benar, XSS dari CMS yang kini tampil di situs publik dan di pratinjau backoffice, dan satu transfer yang dapat melunasi beberapa invoice.

Threat model: 27 ancaman baru (11 Tinggi, 14 Sedang, 2 Rendah) di Lampiran A, untuk dimasukkan ke `docs/04-threat-model.md` sebagai v1.3. Status: **LULUS untuk lingkup yang sudah disetujui**, karena tidak ada ancaman lama yang berubah. Lingkup CR-021 **TIDAK LULUS** karena 6 ancaman Tinggi belum Tercermin: SIT-01, KOR-02, KOR-09, KOR-12, B2B-02, dan B2B-01. B2B-01 bukan blokir reviewer, karena menunggu keputusan user atas UP-84.

---

## 2. Blokir

### B-01 · XSS tersimpan dari konten CMS di situs publik dan pratinjau backoffice (SIT-01, Tinggi)

**Lokasi:**
- `docs/adr/ADR-027-situs-publik-dan-member-area-korporat.md` baris 33 (butir 5): hanya mengatur status Tayang dan jalur gambar.
- `docs/cr/CR-021-analisis.md` baris 168–172 (US-1802 AC1–AC5): tidak ada AC sanitasi.
- `docs/05-asvs-checklist.md` baris 112: "Pustaka sanitasi belum dipilih `[TIDAK DIKETAHUI]`".
- `docs/04-data-dictionary.md` baris 1925: `cms_content.body` berisi Markdown.

**Alasan:**
- CR-021 menjadikan konten CMS sebagai isi utama situs publik.
- Jalur baru: pratinjau CMS di backoffice (F1-T92, F1-T95) merender konten di origin backoffice, tempat sesi Super Admin, Keuangan, dan Verifikator berada. Akun Konten yang jahat atau dikompromikan dapat menyisipkan HTML mentah atau tautan `javascript:` di Markdown. Skrip itu kemudian berjalan di sesi admin lain lewat BFF. Cookie `HttpOnly` tidak mencegah permintaan same-origin yang membawa token CSRF.
- WEB-05 dinilai Tercermin di v1.0 hanya dengan target ASVS L2. H-03 dan U-03 masih Terbuka.

**Usulan (M-40):** tambahkan di ADR-027 butir 5 dan sebagai AC US-1802:
- (a) Markdown dirender tanpa HTML mentah.
- (b) Daftar izin skema URL: `https`, `mailto`, dan relatif.
- (c) Gambar hanya dari endpoint gambar publik backend.
- (d) Validasi di backend saat konten disimpan, ditambah render aman yang sama di situs dan pratinjau.
- (e) Pratinjau backoffice ditampilkan di iframe `sandbox` tanpa `allow-same-origin` atau di origin terpisah.
- (f) CSP ketat untuk `web/situs` dan backoffice, tanpa skrip inline.
- (g) Test payload XSS di F1-T92/F1-T95.

Pustaka tetap dipilih saat task (anti-halu §5).

### B-02 · Jalur pemulihan dan reset MFA akun HR tidak dirancang (KOR-02, Tinggi)

**Lokasi:**
- `docs/adr/ADR-027-...md` baris 27: hanya "atur ulang password tidak melewati MFA".
- `docs/cr/CR-021-analisis.md` baris 236–241 (US-1904): tidak ada AC kehilangan TOTP.
- `docs/adr/ADR-011-target-asvs-autentikasi-sesi-mfa.md` baris 84: jalur H-05 hanya untuk "tenaga kesehatan, admin, dan pengguna RME". Realm `korporat` tidak tercakup.

**Alasan:** HR yang kehilangan TOTP dan kode pemulihan, terutama HR aktif terakhir sebuah perusahaan, hanya dapat dipulihkan lewat orang Ruang Sehat. Ini adalah sasaran rekayasa sosial KC-02 ("ponsel hilang") dengan dampak ke data pribadi seluruh karyawan perusahaan itu dan ringkasan yang dibagikan. Siapa yang mereset, dengan bukti apa, dan kepada siapa pemberitahuan dikirim belum tertulis.

**Usulan (M-41), di ADR-027 butir 3 dan sebagai AC US-1904:**
- Jalur mandiri lewat kode pemulihan didahulukan.
- Bila tidak bisa, reset dimulai sub-peran berwenang (digabung ke PT-67) setelah konfirmasi lewat kanal berbeda dari kanal permintaan: HR aktif lain di perusahaan yang sama, atau kontak resmi perusahaan yang terdaftar saat verifikasi.
- Pola KEPUTUSAN-204: tautan sekali pakai ke email akun ditambah password, dengan TOTP lama tetap berlaku sampai TOTP baru tersimpan (H-16 a).
- Semua sesi dicabut, semua HR aktif perusahaan itu diberi tahu, dan E-04 tercatat.

Sub-peran adalah keputusan user, sehingga dibawa lewat PT-67.

### B-03 · Kanal turunan yang membocorkan status kesehatan ke HR tanpa persetujuan (KOR-09, Tinggi)

**Lokasi:**
- `docs/adr/ADR-028-...md` baris 27–31 (butir 3).
- `docs/cr/CR-021-analisis.md` baris 333–335 (US-1912 AC1–AC3), baris 354 dan 357 (US-1914 AC2, AC5).

**Alasan:** jalur langsung (endpoint ringkasan, detail, dan ekspor) sudah tertutup. Yang belum dikunci sebagai invarian adalah data turunan dari hasil, yaitu:
- (a) agregat atau statistik ("n peserta unfit", persentase) yang dihitung atas peserta tanpa persetujuan. Pada kelompok kecil, angka ini mengidentifikasi orang.
- (b) status "hasil sudah tersedia", "perlu pemeriksaan lanjutan", atau "jadwal ulang" per peserta.
- (c) baris invoice atau perubahan jumlah tambahan per peserta yang muncul karena hasil, misalnya tes lanjutan (PT-63).
- (d) penyimpanan sementara ringkasan di BFF atau berkas ekspor lama setelah persetujuan ditarik.

BR-144 dan permintaan review menyebut agregat kecil secara eksplisit.

**Usulan (M-42), di ADR-028 butir 3 dan sebagai AC US-1912/US-1914:**
- "HR hanya menerima data yang berasal dari hasil (nilai, ringkasan, ketersediaan hasil, status tindak lanjut, agregat, atau baris tagihan yang bergantung pada hasil) untuk peserta yang persetujuannya aktif saat permintaan."
- Agregat hanya boleh dihitung atas peserta yang ringkasannya memang terlihat oleh HR.
- Rekap kehadiran tidak memuat status hasil.
- BFF tidak menyimpan ringkasan di cache.
- Test negatif per kanal.

### B-04 · Undangan dapat tertaut ke orang yang salah, sehingga hasil lengkap sampai ke akun lain (KOR-12, Tinggi)

**Lokasi:**
- `docs/adr/ADR-028-...md` baris 24 (butir 2).
- `docs/cr/CR-021-analisis.md` baris 289 (US-1908 AC4 → `[UP-83]`) dan baris 396 (UP-83).

**Alasan:** penautan menentukan ke akun mana hasil lengkap US-1915 dikirim. Ada tiga jalur salah tautan:
- (a) Tautan diteruskan atau disadap, lalu diterima akun pasien lain. UP-83 (email terverifikasi harus sama) menutup jalur ini, tetapi masih usulan yang dapat ditolak user.
- (b) HR salah ketik email ke alamat milik orang lain yang nyata. Pemilik alamat itu dapat mendaftar dan lolos UP-83. Jalur ini belum ada mitigasinya.
- (c) Tautan deep link ditangkap aplikasi lain bila memakai skema kustom.

Unggahan massal membuat salah ketik bukan kejadian langka.

**Usulan (M-43), di ADR-028 butir 2:**
- (1) UP-83 dijadikan syarat keamanan. Bila user menolaknya, catat sebagai risiko sisa yang diterima user (§0.4).
- (2) Usulan baru ke user: saat menerima undangan, peserta mengetik tanggal lahirnya dan dicocokkan dengan data unggahan, dengan batas percobaan. Bila gagal, tidak ditautkan dan HR melihat status netral.
- (3) Hasil (K3) baru dirilis ke akun tertaut setelah identitas dicocokkan di lokasi layanan (kaitkan dengan PT-66).
- (4) Tautan memakai App Links/Universal Links terverifikasi, bukan skema kustom. Token ditukar hanya setelah login di realm `pasien`.

### B-05 · Satu transfer bank dapat melunasi beberapa invoice (B2B-02, Tinggi)

**Lokasi:**
- `docs/adr/ADR-029-...md` baris 20 (butir 3): "referensi bank (unik per invoice)".
- `docs/cr/CR-021-analisis.md` baris 424 (`b2b_payment_receipt`: "UQ per invoice") dan baris 314 (US-1910 AC4: "untuk invoice itu").

**Alasan:** keunikan per invoice tidak mencegah satu mutasi bank yang sama dicatat sebagai pelunasan invoice A dan invoice B, baik karena salah maupun sengaja. Akibatnya dua pesanan berstatus Lunas dan dijadwalkan (AB-83) atas satu pembayaran. Ini kehilangan uang di modul T1.

**Usulan (M-44):**
- Penerimaan dicatat sebagai mutasi bank tersendiri, satu baris per transfer masuk, dengan kunci unik global atas rekening tujuan, tanggal valuta, referensi, dan nominal. Komposisi kunci ini diverifikasi terhadap format mutasi bank saat task.
- Mutasi itu dialokasikan ke invoice. Jumlah alokasi tidak boleh melebihi nominal mutasi, ditegakkan di aplikasi dan constraint database.
- Test negatif: mutasi yang sama dialokasikan ke dua invoice.

---

## 3. Catatan non-blokir (5 terpenting)

- **N-01 (rujukan ADR-018 keliru).** `docs/adr/ADR-027-...md` baris 42 dan `docs/decision-log.md` baris 254 (KEPUTUSAN-252) menyebut bucket publik "bertentangan dengan ADR-018". Padahal `docs/adr/ADR-018-berkas-unggahan.md` baris 15 justru membolehkan bucket konten CMS dibaca publik. Keputusan baru lebih ketat dan saya dukung, tetapi alasannya perlu dikoreksi. ADR-018 butir 1, H-11, dan `05-asvs-checklist.md` baris 179 juga perlu direvisi eksplisit supaya tidak tersisa izin bucket publik.
- **N-02 (UP-84 menjadi gerbang ADR-029, B2B-01 Tinggi).** Saya sarankan bawaan empat mata, rekonsiliasi harian terhadap mutasi bank oleh orang selain pencatat, dan status Lunas hanya dari mutasi bank, bukan dari bukti transfer kiriman perusahaan (M-45). Bila user menolak, catat sebagai risiko sisa yang diterima user.
- **N-03 (kardinalitas keanggotaan dan kanal peserta web).** ADR-028 butir 1 perlu menyatakan bahwa satu akun HR hanya punya satu keanggotaan aktif (UQ parsial) dan bahwa perpindahan atau pencabutan keanggotaan mencabut sesi (KOR-06, M-51). ADR-027 baris 32: bila PT-60 memilih web, klien realm `pasien` sebaiknya di aplikasi terpisah, bukan di `web/korporat`, supaya kompromi BFF HR tidak memberi token pasien.
- **N-04 (impor).** Pembacaan XLSX di worker, bukan di jalur permintaan `api`. Batas ukuran setelah dekompresi, jumlah entri, dan rasio kompresi diperiksa sebelum mengurai `[SRC-409]`. Pemrosesan DTD dimatikan `[SRC-405]` `[SRC-406]`. Berkas `.xlsm`/OLE dan entri `vbaProject.bin` ditolak. Berkas mentah disimpan di bucket impor tanpa versi objek, dengan siklus kedaluwarsa. Pesan galat tidak menggemakan isi sel (IMP-01, IMP-03; M-53, M-55). Penetralan formula `[SRC-407]` diberlakukan di semua ekspor, bukan hanya rekap (M-54 memperluas M-18).
- **N-05 (permukaan publik dan undangan).**
  - `web/situs` di jaringan Docker sendiri yang hanya menjangkau backend lewat Caddy di `/v1/public/*`, tanpa egress lain.
  - Cookie `web/korporat` berprefiks `__Host-` `[SRC-408]`. CSRF memakai synchronizer token, bukan double-submit cookie.
  - Respons konten publik boleh di-cache di Caddy dengan TTL pendek, dengan anggaran batas laju terpisah dari API pasien.
  - Tanpa skrip pihak ketiga di situs (SIT-03–SIT-05; M-46–M-48).
  - Undangan peserta baru dikirim setelah Lunas, jumlahnya dibatasi jumlah peserta pesanan, dan kirim ulang dibatasi laju (KOR-04, M-50; usulan untuk PT-63).
  - Verifikasi perusahaan sebaiknya menuntut bukti kewenangan pendaftar, bukan hanya NIB/NPWP. Ini keputusan user (KOR-03, M-49).

---

## 4. Penerapan

- Lampiran A dimasukkan ke `docs/04-threat-model.md` sebagai v1.3 oleh pelaksana, dengan pola review F1-T02. Saya tidak mengedit berkas itu langsung karena alat yang tersedia hanya dapat menulis ulang berkas penuh, dan itu berisiko merusak 625 baris yang tidak berubah.
- Lampiran B dimasukkan ke `docs/sources.md`. Nomor SRC-405–SRC-409 diperiksa ulang terhadap branch lain sebelum digabung.
- Status ancaman Tinggi baru yang masih Sebagian/Belum berubah menjadi Tercermin setelah B-01–B-05 diterapkan dan user memutuskan UP-83, UP-84, serta usulan tanggal lahir di M-43.

---

## 5. Putaran 2 (cek ulang, 2026-11-19)

Yang diperiksa (working tree belum di-commit):
- Baris yang berubah di `docs/cr/CR-021-analisis.md` v0.2, ADR-027, ADR-028, dan ADR-029.
- KEPUTUSAN-264–269 di `docs/decision-log.md` (6 baris tercatat).
- SRC-405–SRC-409 di `docs/sources.md` baris 400–404 (tercatat). SRC-440, yang dirujuk ADR-028 baris 58, juga tercatat di baris 398. Isinya bukan bagian review ini.
- `docs/04-threat-model.md` v1.3.

### 5.1 Status blokir

| Blokir | Hasil | Bukti (berkas:baris) |
| --- | --- | --- |
| B-01 (SIT-01) | **Tertutup** | `ADR-027:38–40`: render aman, aturan sama divalidasi saat simpan dan dipakai saat render, iframe `sandbox` tanpa `allow-same-origin`, CSP tanpa skrip inline. `CR-021-analisis.md:179–182`: US-1802 AC6–AC9, termasuk payload XSS di situs dan pratinjau. `:692–693`: F1-T95/T96 bergantung pada F1-T111 |
| B-02 (KOR-02) | **Tertutup** | `ADR-027:34`: jalur pemulihan HR, KEPUTUSAN-265, usulan revisi ADR-011 butir 5. `CR-021-analisis.md:252–254`: US-1904 AC7–AC9, termasuk penolakan tanpa konfirmasi kanal kedua dan reset atas akun milik staf sendiri. AC dibangun di Fase 2 (KEPUTUSAN-179). Sub-peran tetap PT-67 |
| B-03 (KOR-09) | **Tertutup** | `ADR-028:44–49`: invarian AB-87 beserta konsekuensinya. `CR-021-analisis.md:121` (AB-87), `:324` (US-1909 AC6), `:360–361` (US-1912 AC5–AC6, `no-store`), `:384–385` (US-1914 AC7 agregat, AC8 penarikan) |
| B-04 (KOR-12) | **Tertutup** (sebagai blokir reviewer) | `ADR-028:32–37`: UP-83 sebagai syarat keamanan, App Links/Universal Links, PT-78 dan UP-87 diajukan ke user, batas undangan. `CR-021-analisis.md:303` (AC1a), `:308` (AC4a), `:395` (US-1915 AC4), `:424`, `:428`, `:776`. Desain sudah lengkap. Status Tercermin menunggu keputusan user (§0.4) |
| B-05 (B2B-02) | **Tertutup** | `ADR-029:20–26`: `bank_statement_entry` dengan UQ global, alokasi ≤ nominal mutasi di aplikasi dan database, test negatif. `CR-021-analisis.md:332–336` (US-1910 AC1, AC1a, AC4: lintas invoice ditolak), `:453–454` (tabel) |

N-01–N-05 juga ditangani:
- N-01: `ADR-027:50`, koreksi v0.2 ditambah rencana revisi ADR-018 butir 1, H-11, dan checklist (`CR-021-analisis.md:566`, `:728`).
- N-02: US-1910 AC6 di `:338`.
- N-03: `ADR-028:24` dan `ADR-027:35`.
- N-04: `ADR-028:57–59` dan US-1907 AC10–AC12.
- N-05: `ADR-027:41–42`, US-1908 AC1b, dan PT-79.

### 5.2 Perbandingan threat model v1.3 dengan v1.2

**Keterbatasan:** saya tidak dapat menjalankan `git diff`, karena alat yang tersedia tidak termasuk shell. Perbandingan dilakukan terhadap isi v1.2 yang saya baca di putaran 1 (HEAD `9530155`).

Hasil:
- Kepala dokumen (baris 4, 7, 25), AS-19–AS-21, AK-08, TB-15/TB-16, §4.19–§4.22 (baris 460–501), §5.1, M-40–M-56, H-18, §8, dan §9 baris 1.3 cocok dengan Lampiran A.
- Urutan §4.1–§4.18 utuh.
- Paragraf status v1.0–v1.2 dan baris riwayat 0.1–1.2 tidak berubah.
- Angka v1.2 dicatat sebagai pembanding (baris 518).

Tidak ditemukan perubahan pada ancaman lama.

Yang perlu dikoreksi pelaksana (non-blokir):
- (a) Baris 7 "Dasar tambahan v1.3" masih "KEPUTUSAN-250–263". Ganti menjadi 250–269, tambahkan `CR-021-analisis.md` v0.2 dan SRC-405–SRC-409.
- (b) Baris 719 "Catatan pelaksana" dihapus dan diganti paragraf status di §5.4.
- (c) Kolom Status di §4.19–§4.22, §5.1–§5.3, §6, dan §9 diperbarui sesuai §5.3–§5.4.

### 5.3 Status ancaman yang harus diubah di threat model v1.3

| ID | Tingkat | Status putaran 1 | Status putaran 2 | Dasar |
| --- | --- | --- | --- | --- |
| SIT-01 | Tinggi | Sebagian | **Tercermin** | ADR-027 butir 5; US-1802 AC6–AC9; F1-T111; KEPUTUSAN-264 |
| KOR-02 | Tinggi | Belum | **Tercermin** | ADR-027 butir 3; US-1904 AC7–AC9; KEPUTUSAN-265 (dibangun Fase 2; sub-peran PT-67) |
| KOR-09 | Tinggi | Sebagian | **Tercermin** | ADR-028 butir 3 (AB-87); US-1909 AC6; US-1912 AC5–AC6; US-1914 AC7–AC8; KEPUTUSAN-266 |
| B2B-02 | Tinggi | Sebagian | **Tercermin** | ADR-029 butir 3; US-1910 AC1, AC1a, AC4; KEPUTUSAN-268 |
| KOR-12 | Tinggi | Sebagian | **Sebagian (desain lengkap; menunggu user: UP-83, PT-78/UP-87)** | ADR-028 butir 2; US-1908 AC1a, AC4a; US-1915 AC4; KEPUTUSAN-267. Menjadi Tercermin bila user menyetujui, atau menerima risiko sisa secara tercatat |
| B2B-01 | Tinggi | Sebagian | Sebagian (tetap; menunggu UP-84) | US-1910 AC6 |
| SIT-03 | Sedang | Sebagian | **Tercermin** | ADR-027 butir 6; KEPUTUSAN-269 |
| SIT-05 | Sedang | Belum | **Tercermin** | ADR-027 butir 5; US-1802 AC9 |
| KOR-04 | Sedang | Belum | **Sebagian** | US-1908 AC1b; ADR-028 butir 2. Sisa: waktu kirim setelah Lunas masih usulan untuk PT-63 |
| KOR-06 | Sedang | Sebagian | **Tercermin** | ADR-028 butir 1 (UQ parsial, pencabutan sesi) |
| IMP-01 | Sedang | Sebagian | **Tercermin** | ADR-028 butir 6; US-1907 AC11 |
| IMP-02 | Sedang | Sebagian | **Tercermin** | ADR-028 butir 6 (semua ekspor); US-1908 AC1b |
| IMP-03 | Sedang | Sebagian | **Tercermin** | ADR-028 butir 6 (bucket impor tanpa versi); US-1907 AC12 |
| B2B-03 | Sedang | Belum | **Sebagian** | US-1909 AC7 (berlaku setelah PT-59). Sisa: DMARC belum tertulis |
| SIT-04, KOR-03, KOR-08 | Sedang | Sebagian | Sebagian (tetap) | Cache ditetapkan Tahap 8; PT-79/PT-72 menunggu user; M-52 belum diterapkan |

Status mitigasi di §6:
- **Diterapkan (di ADR Diusulkan, CR-021 v0.2):** M-40, M-41, M-42, M-43, M-44, M-46, M-48, M-51, M-53, M-54, M-55, dan H-18 (`CR-021-analisis.md:506`).
- **Sebagian:** M-47, M-50, M-56.
- **Menunggu user:** M-45 (UP-84) dan M-49 (PT-79).
- **Terbuka:** M-52.

### 5.4 Angka §5.1 v1.3 dan paragraf status

| Tingkat | Jumlah | Tercermin | Sebagian | Belum |
| --- | --- | --- | --- | --- |
| Tinggi | 66 | 64 | 2 | 0 |
| Sedang | 75 | 50 | 20 | 5 |
| Rendah | 8 | 7 | 1 | 0 |
| **Total** | **149** | **121** | **23** | **5** |

Kolom §5.2 "Tinggi belum tercermin":
- 4.19: —
- 4.20: KOR-12 (menunggu user)
- 4.21: —
- 4.22: B2B-01 (menunggu user)

§5.3 butir v1.3:
- HR vs hasil: Tercermin.
- Akun HR dan pemulihan MFA: Tercermin.
- Penautan undangan: Sebagian, menunggu user.
- Impor: Tercermin.
- XSS CMS: Tercermin.
- Pembayaran manual: B2B-02 Tercermin; B2B-01 menunggu UP-84.

Paragraf §8 v1.3 (pengganti baris 712–719): "Versi 1.3 (setelah cek ulang review keamanan CR-021 putaran 2): ancaman lama tidak berubah status; lingkup yang sudah disetujui tetap **LULUS**. Di lingkup CR-021, blokir B-01–B-05 tertutup. Dua ancaman Tinggi masih Sebagian dan hanya menunggu keputusan user: KOR-12 (UP-83, PT-78/UP-87) dan B2B-01 (UP-84). Bila user menyetujui usulan itu atau menerima risiko sisanya secara tercatat, keduanya menjadi Tercermin (§0.4) dan lingkup CR-021 **LULUS**. Sampai saat itu lingkup CR-021 berstatus **LULUS BERSYARAT keputusan user**, tanpa celah desain yang tersisa bagi penyusun. ADR-029 tetap tidak dapat Diterima sebelum PT-59, L-21, dan UP-84 diputuskan."

Riwayat §9: tambahkan ke baris 1.3, "Cek ulang putaran 2: SIT-01, KOR-02, KOR-09, B2B-02, SIT-03, SIT-05, KOR-06, IMP-01–03 ke Tercermin; KOR-04, B2B-03 ke Sebagian; KOR-12 dan B2B-01 menunggu user".

### 5.5 Catatan baru (non-blokir)

- **R2-01 (`bank_statement_entry`).** Nominal ikut menjadi bagian kunci unik, sehingga salah ketik nominal untuk referensi yang sama lolos sebagai mutasi kedua. Selain itu, membalik mutasi lalu mencatat ulang dengan kunci identik akan ditolak UQ. Saat task (DBA), tetapkan dua hal: (i) kunci tanpa nominal, atau peringatan bila referensi sama dengan nominal berbeda; (ii) UQ yang mengecualikan mutasi yang sudah dibalik, atau koreksi lewat alokasi pembalik tanpa mencatat ulang mutasi (`ADR-029:21`, `CR-021-analisis.md:453`).
- **R2-02.** KOR-02 dan KOR-12 bergantung pada keputusan user (PT-67, PT-78, UP-83, UP-87). Pastikan pertanyaan itu tampil di §16 analisis. Di baris 819–820 sudah tampil, kecuali PT-67, yang sudah ada sejak v0.1 (butir 5).

### 5.6 Verdict putaran 2

**LULUS DENGAN CATATAN.**
- B-01–B-05 tertutup.
- Catatan non-blokir R2-01, R2-02, dan §5.2 (a)–(c) tidak memicu putaran baru.
- Status lingkup CR-021 di threat model mengikuti §5.4 dan menjadi LULUS penuh setelah keputusan user atas UP-83/PT-78/UP-87 dan UP-84.

---

## Lampiran A · Tambahan `docs/04-threat-model.md` v1.3 (siap-tempel)

### A.1 Kepala dokumen

- `Versi:` "1.3 (2026-11-19, review keamanan CR-021: situs publik, member area korporat, data peserta, invoice B2B; berlaku bila CR-021 disetujui). Sebelumnya 1.2 (...)".
- `Dasar tambahan v1.3:` `docs/cr/CR-021-analisis.md` v0.1, ADR-027–ADR-029 (Diusulkan), KEPUTUSAN-250–263, `docs/review/CR-021-security.md`.
- Paragraf baru **Perubahan v1.3 (CR-021).** "Komponen baru `web/situs`, `web/korporat`, realm `korporat`, tenant perusahaan, impor berkas peserta, dan invoice B2B dinilai di §4.19–§4.22 (SIT, KOR, IMP, B2B). Ancaman berlaku bila CR-021 dan ADR-027–ADR-029 disetujui. Ancaman lama tidak berubah status. TB-15 dan TB-16 baru; AS-19–AS-21 baru; AK-08 baru; M-40–M-56 dan H-18 baru."

### A.2 Aset (tambahkan ke §1)

| ID | Aset | Lokasi utama | Klasifikasi | Properti | Sumber |
| --- | --- | --- | --- | --- | --- |
| AS-19 | Data peserta korporat dan turunannya: nama, kontak, tanggal lahir, NIK opsional, status undangan dan kehadiran, persetujuan berbagi, ringkasan hasil | `corporate_participant`, `participant_import_row`, `corporate_invitation*`, `corporate_attendance_record`, `corporate_share_consent_record`, `corporate_result_summary`; berkas impor mentah | PII; NIK dan ringkasan Spesifik, Enc | K, I | CR-021 §5; ADR-028 butir 2–4 |
| AS-20 | Data dan dokumen perusahaan klien serta keanggotaan HR | `corporate_client`, `corporate_client_document`, `corporate_member`; bucket dokumen | Badan usaha; dokumen dapat memuat PII pengurus | K, I | CR-021 §5; US-1901–US-1904 |
| AS-21 | Konten dan gambar situs publik (integritas dan keaslian) | `cms_content` kanal situs; endpoint `/v1/public/content/*` | — | I, A | ADR-027 butir 5; US-1801, US-1802 |

AS-06 diperluas: "invoice B2B, harga terkunci korporat, penerimaan transfer manual, jurnal korporat (ADR-029)". AS-10 diperluas: "sesi BFF `web/korporat`, faktor TOTP HR, token undangan peserta (hash)".

### A.3 Aktor (tambahkan ke §2)

| ID | Aktor | Kemampuan awal yang diasumsikan | Motif yang dinilai |
| --- | --- | --- | --- |
| AK-08 | HR perusahaan jahat atau akun HR terkompromi | Akun realm `korporat` aktif dengan keanggotaan di satu perusahaan; dapat memanggil API korporat langsung dan mengunggah berkas | Membaca hasil atau ringkasan karyawan tanpa persetujuan, data perusahaan lain, menyalahgunakan undangan, injeksi lewat berkas |

### A.4 Trust boundary (tambahkan ke §3)

Baris diagram tambahan:
```
  HRB[Peramban HR] -->|TB-1| LB
  HRB -->|TB-9 realm korporat| KC
  PUB[Peramban publik] -->|TB-1| LB
  LB --> SITUS[web/situs tanpa sesi] & KORP[web/korporat BFF]
  SITUS -->|hanya /v1/public| API
  KORP -->|TB-15 /v1/corporate| API
  WK -->|TB-16 berkas impor| OS
```

| TB | Batas | Yang melintas | Kontrol utama di desain | Ancaman terkait |
| --- | --- | --- | --- | --- |
| TB-15 | Domain pemberi kerja (HR, tenant perusahaan) ke data kesehatan peserta (batas logis di backend) | Rekap kehadiran, status undangan, ringkasan berpersetujuan | `CorporateScope` dari sesi, 404, fungsi akses sensitif yang memeriksa persetujuan saat baca dan menulis E-05, tanpa jalur hasil lengkap (ADR-028 butir 1–3) | KOR-07, KOR-09, KOR-10, KOR-13 |
| TB-16 | Berkas CSV/XLSX tak tepercaya dari HR ke pengurai di backend | Isi berkas, baris pratinjau | Validasi isi, batas PB-23, berkas terenkripsi ditolak, pembatasan sumber daya, pratinjau terenkripsi berumur (ADR-028 butir 6) | IMP-01–IMP-04 |

TB-1 diperluas: "situs publik, member area korporat, endpoint konten dan gambar publik (v1.3)". TB-9 diperluas: "realm `korporat` (v1.3)".

### A.5 §4.19 Situs publik `web/situs` dan konten publik (TB-1), baru di v1.3

| ID | STRIDE | Aktor | Ancaman | K | D | Tingkat | Mitigasi dalam desain | Tercermin di | Status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| SIT-01 | T, I, E | AK-03 (Konten), AK-05 (akun Konten terkompromi) | XSS tersimpan dari Markdown CMS (HTML mentah, tautan `javascript:`, sumber gambar eksternal) di situs publik dan di pratinjau CMS backoffice. Di backoffice, skrip berjalan di origin sesi admin lain dan memanggil BFF dengan token CSRF yang sah | S | T | **Tinggi** | Target ASVS L2; cookie `HttpOnly`; CSP situs disebut di F1-T89; sanitasi CMS "Tertunda Tahap 8" | WEB-05; H-03; F1-T89; `05-asvs-checklist.md` V1 | **Sebagian** (M-40; blokir B-01 review CR-021) |
| SIT-02 | I | AK-05 | Konten draf, terjadwal, atau terhapus bocor lewat endpoint publik, ID gambar yang ditebak, atau cache basi setelah konten diturunkan | S | S | Sedang | Hanya konten Tayang; gambar lewat endpoint yang memeriksa status; object storage privat; tanpa `use cache`; test draf tidak bocor | ADR-027 butir 5; US-1801 AC3; F1-T93, F1-T94; KEPUTUSAN-252 | Tercermin |
| SIT-03 | E, T | AK-05 | RCE di `web/situs` `[SRC-246]` dipakai untuk menjangkau layanan internal di jaringan Docker yang sama, atau untuk menanam cookie di domain induk (cookie tossing) terhadap `web/korporat` | S | S | Sedang | Tanpa identitas OpenBao, Valkey, atau kunci JWE; patch ≤ 7 hari; cookie host terpisah (F1-T90). Belum ada: segmentasi jaringan dan prefiks cookie | ADR-027 butir 1, 6; KEPUTUSAN-75 | **Sebagian** (M-46) |
| SIT-04 | D | AK-05 | Scraping atau banjir permintaan ke halaman yang dirender server tanpa cache dan ke endpoint gambar, membebani proses `api` yang sama dengan layanan pasien (NFR-AVL-01) | S | S | Sedang | Batas laju endpoint publik; cache HTTP ditetapkan Tahap 8 dengan review keamanan | ADR-027 butir 5; ADR-019 butir 3 | **Sebagian** (M-47) |
| SIT-05 | I | AK-06 | Skrip analitik, peta, font, atau sematan pihak ketiga di situs mengirim data pengunjung (IP, perilaku) ke luar Indonesia, atau disusupi sebagai rantai pasok skrip | S | S | Sedang | Tidak ada aturan pihak ketiga untuk situs | — | **Belum** (M-48) |

### A.6 §4.20 Realm `korporat`, BFF `web/korporat`, dan tenant perusahaan (TB-1, TB-9, TB-15), baru di v1.3

| ID | STRIDE | Aktor | Ancaman | K | D | Tingkat | Mitigasi dalam desain | Tercermin di | Status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| KOR-01 | S | AK-05 | Pengambilalihan akun HR lewat credential stuffing atau phishing | S | T | **Tinggi** | MFA TOTP wajib dengan kode pemulihan sejak login pertama; deteksi brute force 5/15 menit; pesan seragam; atur ulang password ber-OTP; sesi web 30 menit/12 jam; penonaktifan seketika | ADR-027 butir 3; US-1904 AC1, AC5, AC6; KEPUTUSAN-253; F1-T85, F1-T88 | Tercermin (risiko sisa seperti APP-04: TOTP tidak tahan phishing `[SRC-276]`) |
| KOR-02 | S, E | AK-05 (rekayasa sosial ke staf Ruang Sehat) | HR yang mengaku kehilangan TOTP dan kode pemulihan meminta reset MFA ke Ruang Sehat, lalu penyerang mendaftarkan TOTP miliknya. Paling rawan untuk HR aktif terakhir | S | T | **Tinggi** | Tidak ada jalur reset MFA untuk realm `korporat`; ADR-011 baris 84 hanya mencakup nakes, admin, dan RME | — | **Belum** (M-41; blokir B-02) |
| KOR-03 | S | AK-05, AK-03 | Penyerang mendaftarkan perusahaan nyata dengan salinan NIB/NPWP (dokumen tidak membuktikan kewenangan pendaftar), atau admin/sales membuat perusahaan Aktif tanpa verifikasi kedua, lalu menerima persetujuan berbagi ringkasan atas nama perusahaan itu | R | T | Sedang | Verifikasi admin atas NIB/NPWP; invoice harus lunas sebelum layanan; E-10 mencatat sumber pembuatan. Sub-peran (PT-67) dan NIB ganda (PT-72) `[TIDAK DIKETAHUI]` | US-1901–US-1903; AB-83 | **Sebagian** (M-49; keputusan user) |
| KOR-04 | D, S | AK-08, AK-05 | Undangan peserta dan kirim ulang dipakai sebagai meriam email ber-DKIM sah (spam, email bombing, phishing berlabel nama perusahaan), menghabiskan kuota SES yang dipakai bersama email autentikasi (EML-04); injeksi HTML dari nama peserta ke templat email; pendaftaran perusahaan palsu massal mengisi antrean verifikasi dan penyimpanan dokumen | S | S | Sedang | Templat tanpa data kesehatan (AB-73); batas laju pendaftaran (RT-45). Waktu pengiriman undangan `[TIDAK DIKETAHUI]` (PT-63) | US-1908 AC1, AC7; US-1901 | **Belum** (M-50) |
| KOR-05 | E | AK-01, AK-04, AK-08 | Token realm `korporat` diterima di API pasien, backoffice, atau portal, atau sebaliknya | S | T | **Tinggi** | Validasi issuer dan audience; namespace `/v1/corporate/*` dengan audience sendiri; test dua arah | ADR-027 butir 2; US-1904 AC4; F1-T86, F1-T90; ADR-011 butir 8 | Tercermin |
| KOR-06 | E, I | AK-08 | `CorporateScope` ambigu bila satu akun punya lebih dari satu keanggotaan, bila keanggotaan dipindah atau dicabut saat sesi masih terbuka, atau bila HR yang keluar dari perusahaan adalah HR terakhir | R | T | Sedang | Scope dari keanggotaan aktif di sesi; penolakan sesi seketika (KU-10); HR terakhir tidak dapat dinonaktifkan. Kardinalitas keanggotaan tidak dinyatakan | ADR-028 butir 1; US-1904 AC2, AC5 | **Sebagian** (M-51) |
| KOR-07 | I, E | AK-08 | IDOR lintas perusahaan pada perusahaan, anggota HR, pesanan, peserta, batch impor, pratinjau, undangan, invoice, unduhan berkas, rekap, dan ekspor, termasuk lewat ID anak dan job asinkron | S | T | **Tinggi** | `CorporateScope` dari sesi; 404; test isolasi otomatis semua endpoint `/v1/corporate/*` di `scripts/verify.sh` | ADR-028 butir 1; US-1904 AC3; US-1907 AC8; US-1909 AC4; US-1912 AC4; F1-T100; KEPUTUSAN-254 | Tercermin; penguatan H-18 |
| KOR-08 | I | AK-05, AK-08 | Enumerasi: tanggapan atas NIB/NPWP yang sudah terdaftar membocorkan daftar klien; undangan HR ke email yang sudah menjadi HR perusahaan lain membocorkan keanggotaan; status token undangan membocorkan nama perusahaan | S | S | Sedang | Pesan seragam untuk email terdaftar; tautan undangan yang tidak valid tidak membocorkan perusahaan. NIB ganda dan undangan HR lintas perusahaan belum diatur | US-1901 AC4; US-1908 AC5; ADR-011 butir 4 | **Sebagian** (M-52) |
| KOR-09 | I | AK-08 | HR memperoleh status kesehatan peserta tanpa persetujuan lewat kanal turunan: agregat atau statistik pada kelompok kecil, status "hasil tersedia" atau "tindak lanjut", baris invoice tambahan yang bergantung pada hasil, ringkasan yang tersimpan di cache BFF atau ekspor lama | S | T | **Tinggi** | Jalur langsung, detail, dan ekspor memeriksa persetujuan saat baca; teks seragam UP-85; hasil lengkap tanpa jalur HR | ADR-028 butir 3; AB-84; US-1912 AC2; US-1914 AC2, AC5 | **Sebagian** (M-42; blokir B-03) |
| KOR-10 | I | AK-03, AK-04, AK-07 | Admin Ruang Sehat atau rekanan membaca ringkasan atau hasil peserta korporat | S | T | **Tinggi** | Tanpa kebijakan baca admin; wadah enkripsi `corporate_result_summary`; hanya fungsi akses sensitif; E-05 | ADR-028 butir 3–4; US-1914 AC4; US-1915 AC2; KEPUTUSAN-258 | Tercermin |
| KOR-11 | S, T, R | AK-08, AK-03, AK-07 | Persetujuan berbagi dipalsukan (diberikan HR atau admin atas nama peserta), diubah, atau disangkal | R | T | Sedang | Hanya akun pasien yang tertaut dapat memberi atau menarik; tabel Log append-only dengan versi teks; E-13 | ADR-028 butir 3; US-1913 AC2–AC3; ADR-008 | Tercermin |
| KOR-12 | S, I | AK-05, kesalahan HR | Undangan tertaut ke orang yang salah: tautan diteruskan atau disadap; HR salah ketik email ke alamat milik orang lain yang nyata (lolos syarat email sama); deep link skema kustom ditangkap aplikasi lain. Jadwal dan kemudian hasil lengkap mengalir ke akun itu | S | T | **Tinggi** | Penautan hanya oleh peserta yang sudah masuk; token sekali pakai di-hash; masa berlaku PB-24 `[TIDAK DIKETAHUI]`; UP-83 masih usulan | ADR-028 butir 2; AB-85; US-1908 AC3–AC5; UP-83 | **Sebagian** (M-43; blokir B-04) |
| KOR-13 | I | AK-08 | HR membaca data akun pasien peserta (profil, identitas, riwayat layanan lain) | R | T | Sedang | HR hanya melihat data yang diunggahnya dan status undangan | ADR-028 butir 2; US-1908 AC8 | Tercermin |

### A.7 §4.21 Impor berkas peserta (TB-16), baru di v1.3

| ID | STRIDE | Aktor | Ancaman | K | D | Tingkat | Mitigasi dalam desain | Tercermin di | Status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| IMP-01 | D, I | AK-08 | XLSX (arsip ZIP berisi XML) berupa zip bomb atau XML bomb/"billion laughs" `[SRC-409]` `[SRC-405]`, entitas eksternal (XXE) `[SRC-406]`, atau shared strings dan baris raksasa yang menghabiskan memori atau CPU proses `api` | S | S | Sedang | Batas ukuran dan baris PB-23; berkas terenkripsi ditolak; "pembacaan XLSX dibatasi sumber dayanya". Belum ada: batas setelah dekompresi, DTD mati, isolasi worker | ADR-028 butir 6; US-1907 AC3; KEPUTUSAN-256 | **Sebagian** (M-53) |
| IMP-02 | T | AK-08 | Sel diawali `=`, `+`, `-`, `@`, tab, CR, atau LF `[SRC-407]` dijalankan saat ekspor dibuka di spreadsheet oleh HR lain, Keuangan, Layanan Pelanggan, atau rekanan; nama peserta berisi HTML/skrip tampil di web korporat, backoffice, portal, atau email (XSS tersimpan) | S | S | Sedang | Isi sel diperlakukan sebagai teks dan dinetralkan saat ekspor rekap | ADR-028 butir 6; US-1907 AC6; US-1912 AC3; M-18 (Terbuka) | **Sebagian** (M-54) |
| IMP-03 | I | AK-07 | Berkas mentah (dengan NIK tanpa enkripsi tingkat aplikasi) tertinggal sebagai versi objek lama, di backup object storage, di log, atau dalam pesan galat yang menggemakan isi sel | S | S | Sedang | Berkas asli dan pratinjau dihapus setelah impor, pembatalan, atau kedaluwarsa; log tanpa isi baris; pratinjau terenkripsi. ADR-018 butir 5 mengaktifkan versi objek | ADR-028 butir 6; US-1907 AC7; ADR-018 butir 5 | **Sebagian** (M-55) |
| IMP-04 | T | AK-08 | Berkas berjenis lain disamarkan sebagai CSV/XLSX (makro `.xlsm`, wadah OLE, poliglot) | R | S | Rendah | Validasi isi, bukan ekstensi; berkas terenkripsi ditolak; berkas tidak pernah disajikan kembali ke siapa pun | ADR-028 butir 6; US-1907 AC3; ADR-018 butir 2 | Tercermin |

### A.8 §4.22 Invoice B2B dan penerimaan manual (TB-1), baru di v1.3

| ID | STRIDE | Aktor | Ancaman | K | D | Tingkat | Mitigasi dalam desain | Tercermin di | Status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| B2B-01 | T | AK-03 (Keuangan), kolusi dengan perusahaan | Penerimaan fiktif atau salah dicatat (termasuk berdasarkan bukti transfer palsu kiriman perusahaan), sehingga invoice Lunas dan layanan dilaksanakan tanpa dana masuk | S | T | **Tinggi** | Penerimaan immutable dengan bukti; jurnal berpasangan; entri pembalik; E-08; status Lunas hanya dari penerimaan tercatat. Pemisahan tugas menunggu UP-84 | ADR-029 butir 3–5; US-1910 AC1–AC7; UP-84 | **Sebagian** (M-45; menjadi Tercermin bila user menerima UP-84 atau menerima risiko sisa secara tercatat) |
| B2B-02 | T | AK-03, kesalahan Keuangan | Satu transfer bank yang sama dicatat sebagai pelunasan beberapa invoice, karena keunikan referensi hanya per invoice | S | T | **Tinggi** | UQ referensi bank per invoice | ADR-029 butir 3; US-1910 AC4; CR-021 §5 `b2b_payment_receipt` | **Sebagian** (M-44; blokir B-05) |
| B2B-03 | T, S | AK-03, AK-05 (business email compromise) | Rekening tujuan di invoice diganti, atau invoice palsu dengan rekening lain dikirim ke perusahaan atas nama Ruang Sehat | S | S | Sedang | Invoice diunduh dari member area (E-12). Rekening tujuan `[TIDAK DIKETAHUI]` (PT-59) | US-1909 AC1, AC3 | **Belum** (M-56) |
| B2B-04 | T | AK-08, AK-03 | Manipulasi harga dari klien, pesanan dan invoice ganda, atau penjadwalan sebelum lunas | S | T | **Tinggi** | Harga dihitung server dan dikunci saat konfirmasi; idempotensi; AB-83 ditegakkan server; nilai berubah harus disetujui ulang | ADR-029 butir 2, 5; AB-80, AB-81, AB-83; US-1906 AC2–AC4; US-1911 AC2 | Tercermin |
| B2B-05 | R | AK-03 | Pencatat atau pembalik penerimaan menyangkal tindakannya | R | S | Rendah | Baris immutable dengan pelaku; E-08; koreksi lewat pembalik | ADR-029 butir 3–4; ADR-004; ADR-005 | Tercermin |

### A.9 §5 dihitung ulang

| Tingkat | Jumlah | Tercermin | Sebagian | Belum |
| --- | --- | --- | --- | --- |
| Tinggi | 66 | 60 | 5 | 1 |
| Sedang | 75 | 44 | 23 | 8 |
| Rendah | 8 | 7 | 1 | 0 |
| **Total** | **149** | **111** | **29** | **9** |

Catatan: angka v1.3 dibanding v1.2 bertambah 27 ancaman (11 Tinggi, 14 Sedang, 2 Rendah), semuanya di §4.19–§4.22. Ancaman Tinggi yang belum Tercermin hanya ada di lingkup CR-021.

§5.2, baris tambahan:

| Komponen | Jumlah | Tinggi | Tinggi belum tercermin |
| --- | --- | --- | --- |
| 4.19 Situs publik | 5 | 1 | SIT-01 |
| 4.20 Realm korporat, BFF, tenant perusahaan | 13 | 7 | KOR-02, KOR-09, KOR-12 |
| 4.21 Impor berkas peserta | 4 | 0 | — |
| 4.22 Invoice B2B | 5 | 3 | B2B-01, B2B-02 |

§5.3, baris tambahan:

| Butir | Ancaman | Tingkat tertinggi | Status |
| --- | --- | --- | --- |
| Isolasi antar-perusahaan dan HR vs hasil peserta (v1.3) | KOR-07, KOR-09, KOR-10, KOR-13 | Tinggi | KOR-09 Sebagian (B-03) |
| Pengambilalihan akun HR dan pemulihan MFA (v1.3) | KOR-01, KOR-02 | Tinggi | KOR-02 Belum (B-02) |
| Penautan undangan ke akun pasien (v1.3) | KOR-12 | Tinggi | Sebagian (B-04) |
| Impor CSV/XLSX (v1.3) | IMP-01–IMP-04 | Sedang | Sebagian |
| XSS dari CMS di situs publik (v1.3) | SIT-01, WEB-05 | Tinggi | SIT-01 Sebagian (B-01) |
| Pencatatan pembayaran manual (v1.3) | B2B-01, B2B-02 | Tinggi | Sebagian (UP-84; B-05) |

### A.10 Mitigasi baru (§6.2 dan §6.3)

| ID | Ancaman | Mitigasi konkret yang diusulkan | Usulan penempatan | Status |
| --- | --- | --- | --- | --- |
| M-40 | SIT-01 | (a) Markdown tanpa HTML mentah; (b) daftar izin skema URL `https`, `mailto`, relatif; (c) gambar hanya dari endpoint gambar publik backend; (d) validasi saat simpan di backend dan render aman yang sama di situs dan pratinjau; (e) pratinjau backoffice di iframe `sandbox` tanpa `allow-same-origin` atau di origin terpisah; (f) CSP tanpa skrip inline untuk `web/situs` dan backoffice; (g) test payload XSS | ADR-027 butir 5; AC baru US-1802; F1-T89, F1-T92, F1-T95; U-03 | Baru (blokir B-01) |
| M-41 | KOR-02 | Kode pemulihan didahulukan; reset oleh sub-peran berwenang (PT-67) setelah konfirmasi lewat kanal berbeda (HR aktif lain atau kontak resmi perusahaan); tautan sekali pakai ke email akun ditambah password (KEPUTUSAN-204); TOTP lama berlaku sampai TOTP baru tersimpan (H-16 a); cabut semua sesi; beri tahu semua HR aktif; E-04 | ADR-027 butir 3; AC baru US-1904; prosedur H-05 | Baru (blokir B-02) |
| M-42 | KOR-09 | Invarian: HR hanya menerima data turunan hasil (nilai, ringkasan, ketersediaan hasil, status tindak lanjut, agregat, baris tagihan yang bergantung pada hasil) untuk peserta dengan persetujuan aktif saat permintaan; agregat hanya atas data yang memang terlihat oleh HR; rekap tanpa status hasil; BFF tidak menyimpan ringkasan di cache; test negatif per kanal | ADR-028 butir 3; AC US-1912, US-1914; masukan PT-63, PT-65, DPIA | Baru (blokir B-03) |
| M-43 | KOR-12 | (1) UP-83 sebagai syarat keamanan, atau risiko sisa diterima user; (2) usulan ke user: konfirmasi tanggal lahir saat menerima undangan dengan batas percobaan; (3) hasil dirilis setelah identitas dicocokkan di lokasi layanan (PT-66); (4) App Links/Universal Links terverifikasi, token ditukar setelah login realm `pasien`; masa berlaku PB-24 ditetapkan | ADR-028 butir 2; US-1908 AC3–AC4; US-1915; UP baru | Baru (blokir B-04) |
| M-44 | B2B-02 | Mutasi bank sebagai baris tersendiri dengan kunci unik global (rekening tujuan, tanggal valuta, referensi, nominal; diverifikasi terhadap format mutasi saat task); alokasi ke invoice dengan jumlah ≤ nominal mutasi (aplikasi dan constraint); test alokasi ganda | ADR-029 butir 3; data dictionary A20; US-1910 AC4 | Baru (blokir B-05) |
| M-45 | B2B-01 | Empat mata (UP-84); rekonsiliasi harian terhadap mutasi bank oleh orang selain pencatat; Lunas hanya dari mutasi bank yang diperiksa Keuangan, bukan dari bukti kiriman perusahaan; laporan piutang ke Auditor | Keputusan user UP-84; ADR-029 butir 3; prosedur Keuangan | Baru (menunggu user) |
| M-46 | SIT-03 | `web/situs` di jaringan Docker sendiri, hanya menjangkau backend lewat Caddy pada `/v1/public/*`, tanpa egress lain; cookie `web/korporat` berprefiks `__Host-` `[SRC-408]`; CSRF dengan synchronizer token, bukan double-submit cookie | ADR-027 butir 6; ADR-025; F1-T89, F1-T90 | Baru |
| M-47 | SIT-04 | Respons konten publik tanpa personalisasi di-cache Caddy dengan TTL pendek dan dibuang saat konten berubah status; anggaran batas laju publik terpisah dari API pasien | ADR-027 butir 5 (keputusan cache Tahap 8); ADR-019 butir 3 | Baru |
| M-48 | SIT-05 | Situs tanpa skrip, font, atau sematan pihak ketiga; CSP `default-src 'self'`; penambahan pihak ketiga lewat CR dengan penilaian lokasi data (T4-Q1) | ADR-027 butir 1; U-03 | Baru |
| M-49 | KOR-03 | Verifikasi menuntut bukti kewenangan pendaftar (misalnya surat penunjukan atau konfirmasi lewat telepon kantor dari sumber independen) — keputusan user; perusahaan buatan admin/sales disetujui orang kedua; aturan NIB ganda (PT-72) tanpa membocorkan keanggotaan | Keputusan user (PT-67, PT-72, PT baru); US-1902, US-1903 | Baru (menunggu user) |
| M-50 | KOR-04 | Undangan dikirim setelah Lunas (usulan untuk PT-63); jumlah undangan ≤ jumlah peserta pesanan; batas kirim ulang per peserta dan per perusahaan per hari; templat tanpa teks bebas dari HR dan pengodean keluaran HTML; verifikasi email sebelum unggah dokumen; pendaftaran tak terverifikasi kedaluwarsa (PT-24) | US-1908; US-1901; PT-63; ADR-016 butir 5 | Baru |
| M-51 | KOR-06 | Satu keanggotaan aktif per akun HR (UQ parsial); perpindahan atau pencabutan keanggotaan mencabut sesi; HR terakhir yang keluar ditangani lewat M-41 | ADR-028 butir 1; data dictionary A20 | Baru |
| M-52 | KOR-08 | Tanggapan generik untuk NIB/NPWP terdaftar dan undangan HR ke akun yang sudah menjadi HR perusahaan lain; konflik diselesaikan admin di luar layar | US-1901 AC7 (PT-72); US-1904 AC2 | Baru |
| M-53 | IMP-01 | Penguraian di worker dengan batas waktu dan memori; batas ukuran setelah dekompresi, jumlah entri, dan rasio kompresi sebelum mengurai `[SRC-409]`; DTD dimatikan `[SRC-405]` `[SRC-406]`; tolak `.xlsm`, OLE, dan `vbaProject.bin` | ADR-028 butir 6; pemilihan pustaka saat task | Baru |
| M-54 | IMP-02 | Penetralan formula `[SRC-407]` di semua ekspor CSV (rekap, backoffice, portal) dan pengodean keluaran field peserta di web dan email; memperluas M-18 | Coding standards; ADR-028 butir 6 | Baru |
| M-55 | IMP-03 | Bucket impor tanpa versi objek dengan aturan kedaluwarsa siklus hidup; dikecualikan dari backup object storage atau berumur sama; pesan galat per baris tanpa menggemakan isi sel | ADR-018 butir 1, 5 (revisi); ADR-028 butir 6 | Baru |
| M-56 | B2B-03 | Rekening tujuan invoice sebagai konfigurasi dengan persetujuan orang kedua dan E-14; email invoice tanpa lampiran dan mengarah ke member area; DMARC ditegakkan untuk domain pengirim | ADR-029 butir 6 (setelah PT-59); ADR-012 butir 7; ADR-026 | Baru |
| H-18 | KOR-07 | Harness isolasi mencakup ekspor, unduhan invoice dan berkas, pratinjau impor, endpoint ID anak (`corporate_order_participant`, undangan), job asinkron yang membangkitkan berkas, dan route handler BFF `web/korporat` | F1-T100; NFR-SEC-06 | Terbuka |

### A.11 §8 Status (paragraf tambahan)

"Versi 1.3: ancaman lama tidak berubah status; lingkup yang sudah disetujui tetap **LULUS**. Lingkup CR-021 (§4.19–§4.22, berlaku bila CR-021 disetujui) **TIDAK LULUS**. Enam ancaman Tinggi belum Tercermin:
- SIT-01, KOR-09, KOR-12, B2B-02: Sebagian;
- KOR-02: Belum;
- B2B-01: Sebagian, menunggu keputusan user UP-84.

Kelima yang pertama adalah blokir B-01–B-05 `docs/review/CR-021-security.md`. ADR-027 dan ADR-028 belum dapat berstatus Diterima sebelum B-01–B-04 ditutup. ADR-029 belum dapat berstatus Diterima sebelum B-05 ditutup dan UP-84 diputuskan."

### A.12 §9 Riwayat revisi (baris baru)

| Versi | Tanggal | Perubahan | Sumber |
| --- | --- | --- | --- |
| 1.3 | 2026-11-19 | Review keamanan CR-021:<br>- §4.19–§4.22 baru (SIT-01–05, KOR-01–13, IMP-01–04, B2B-01–05);<br>- AS-19–AS-21, AK-08, TB-15, TB-16 baru; TB-1, TB-9, AS-06, AS-10 diperluas;<br>- M-40–M-56 dan H-18 baru;<br>- §5 dihitung ulang (149 ancaman).<br>Lingkup lama tetap LULUS; lingkup CR-021 TIDAK LULUS (B-01–B-05, UP-84) | `docs/cr/CR-021-analisis.md`; ADR-027–ADR-029 (Diusulkan); KEPUTUSAN-250–263; `docs/review/CR-021-security.md` |

---

## Lampiran B · Entri `docs/sources.md` (siap-tempel; halaman dibuka 2026-11-19)

| ID | Judul / nama sumber | URL | Tingkat (1/2/3) | Tanggal akses | Mendukung klaim | Dipakai di |
| --- | --- | --- | --- | --- | --- | --- |
| SRC-405 | OWASP XML Security Cheat Sheet | https://cheatsheetseries.owasp.org/cheatsheets/XML_Security_Cheat_Sheet.html | 2 | 2026-11-19 | "nested entities that each reference the next layer several times expand exponentially (the "billion laughs" attack)"; "Disable DTD processing for untrusted XML; this blocks all of the attacks above."; bila DTD diperlukan, "enforce entity expansion and entity size limits" | 04-threat-model.md IMP-01, M-53; review/CR-021-security.md |
| SRC-406 | OWASP XML External Entity Prevention Cheat Sheet | https://cheatsheetseries.owasp.org/cheatsheets/XML_External_Entity_Prevention_Cheat_Sheet.html | 2 | 2026-11-19 | "Disable document type definitions (DTDs) whenever possible."; "Preventing XXE does not prevent every XML denial-of-service attack." Halaman tidak menyebut dokumen Office/XLSX | 04-threat-model.md IMP-01, M-53; review/CR-021-security.md |
| SRC-407 | OWASP Community: CSV Injection (dialihkan dari owasp.org/www-community/attacks/CSV_Injection) | https://community.owasp.org/attacks/CSV_Injection | 2 | 2026-11-19 | Karakter awal berbahaya: "Equals to (`=`)", "Plus (`+`)", "Minus (`-`)", "At (`@`)", "Tab (`0x09`)", "Carriage return (`0x0D`)", "Line feed (`0x0A`)"; mitigasi: "Wrap each cell field in double quotes", "Prepend each cell field with a single quote", "Escape every double quote using an additional double quote"; halaman memperingatkan escaping berbasis kutip dapat gagal setelah Excel menyimpan dan membuka ulang berkas | 04-threat-model.md IMP-02, M-54 (memperluas M-18); review/CR-021-security.md |
| SRC-408 | MDN Web Docs: Set-Cookie header (cookie prefixes) | https://developer.mozilla.org/en-US/docs/Web/HTTP/Reference/Headers/Set-Cookie | 2 | 2026-11-19 | "Cookies with names starting with `__Host-` must be set with the `Secure` attribute by a secure page (HTTPS). In addition, they must not have a `Domain` attribute specified, and the `Path` attribute must be set to `/`."; "This guarantees that such cookies are only sent to the host that set them, and not to any other host on the domain." Halaman tidak menyatakan eksplisit bahwa subdomain tidak dapat menimpa cookie itu | 04-threat-model.md SIT-03, M-46; review/CR-021-security.md |
| SRC-409 | OWASP File Upload Cheat Sheet (dibuka ulang; halaman yang sama dengan SRC-50, kutipan tambahan) | https://cheatsheetseries.owasp.org/cheatsheets/File_Upload_Cheat_Sheet.html | 2 | 2026-11-19 | Daftar ancaman: "Send ZIP bombs, XML bombs"; "the file size limit should be considered after file decompression is conducted"; "by using secure methods to calculate zip files size"; untuk dokumen: "to ensure that no macros or hidden scripts are available". Halaman tidak menyebut XLSX atau CSV | 04-threat-model.md IMP-01, M-53; review/CR-021-security.md |
