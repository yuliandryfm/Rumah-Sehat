# Fact-check Tahap 04: Arsitektur (bagian 1)

Tanggal: 2026-11-18
Dokumen diperiksa: `docs/04-arsitektur.md` (v0.1); `docs/adr/ADR-006` sampai `ADR-020` (Diusulkan); `docs/04-risk-register.md`; `docs/04-catatan-arsitektur.md`; penerapan CR-007 di `docs/02-brd.md` (v1.3), `docs/03-prd.md` (v1.1), `docs/backlog.md`, `docs/traceability.md`, dan catatan KEPUTUSAN-06; `docs/decision-log.md` KEPUTUSAN-18–37; `docs/sources.md` SRC-30–63; `CLAUDE.md` (modul kritis)
Sumber kebenaran: `docs/04-catatan-arsitektur.md` (T4-Q1–T4-Q4), PRD v1.1, BRD v1.3, ADR-001–005, `docs/01-business-case.md`
Tingkat ketelitian: T1
Hasil: **TIDAK LULUS** (2 blokir: FC4-B01, FC4-B02; 5 non-blokir)

Catatan metode:
- Fact-checker tidak punya akses shell, jadi `git diff main` tidak dijalankan. Bagian BRD/PRD yang berubah ditentukan dari penanda CR-007, v1.3/v1.1, dan daftar dampak di `backlog.md` CR-007, lalu dicocokkan dengan catatan wawancara.
- Dokumen berubah **selama pemeriksaan**. Yang bertambah: T4-Q3 dan T4-Q4 di catatan (baris 21–39), baris T4-Q3 di `04-arsitektur.md` (baris 37), label `[USER] T4-Q3` di ADR-006 butir 3, dan status KEPUTUSAN-19. Nomor baris di laporan ini mengikuti salinan terakhir yang dibaca sebelum laporan ditulis.
- `docs/04-erd.md`, `docs/04-data-dictionary.md`, dan `docs/04-threat-model.md` di luar cakupan bagian 1. Ketiganya dirujuk `04-arsitektur.md` baris 10.

## Ringkasan
- Total klaim: sekitar 600 klaim berlabel (perkiraan per label). Rinciannya: arsitektur sekitar 110, ADR-006–020 sekitar 330, risk register sekitar 70, KEPUTUSAN-18–37 (20 entri), SRC-30–63 (34 entri), dan sekitar 35 baris perubahan CR-007.
- Terverifikasi: sekitar 590
- Tanpa sumber: 0 di dokumen keputusan
- Sumber tidak mendukung: 1, non-blokir (FC4-N02)
- Tidak dapat diverifikasi ulang: 1, karena URL gagal dibuka (FC4-N03)
- Kontradiksi: 2 blokir (FC4-B01 terhadap BR-112, FC4-B02 terhadap `[USER] T4-Q4`), ditambah 2 rumusan yang perlu diperjelas (FC4-N01, FC4-N04)
- Fakta bisnis berlabel asumsi atau keputusan: 1 (FC4-B01, angka retensi di KEPUTUSAN-23 dan ADR-019)
- TANPA INDUK: 0. CR-007 hanya menghapus US-102 dan tidak menambah story.

## Hal yang diperiksa dan bersih
- **Label `[USER] T4-Qx`**:
    - T4-Q1 ("Semua di Indonesia"), T4-Q2 ("Setujui CR-007 apa adanya"), dan T4-Q3 ("Tetap terpisah") cocok dengan catatan baris 13–18 dan 30–32.
    - `04-arsitektur.md` §1, BR-113, NFR-PRIV-02, US-1406 AC2, ADR-006 butir 3, dan KEPUTUSAN-19 memakai label sesuai jawaban itu.
- **Penerapan CR-007 sesuai isi yang disetujui di T4-Q2.** Bagian berikut sudah memuat penghapusan login Google/Apple, push, dan WhatsApp, serta notifikasi lewat email dan di dalam aplikasi:
    - BRD v1.3: §2.1, P-01, S-14, BR-001, BR-050, BR-071, BR-113, §7.3, §8, PT-22.
    - PRD v1.1: §2.2, US-102 (dihapus, nomor tidak dipakai ulang), US-306 AC1, US-311 AC2, US-405 AC2, matriks EP-10 (pengingat lewat email; kejadian yang tadinya hanya push tampil di dalam aplikasi), US-1001 AC1–AC5, US-1003 AC1–AC2, US-1406 AC1–AC2, AB-73, NFR-PRIV-02, E-04, US-1602 AC1.
    - Register: baris US-102 dan CR-007 di backlog; BR-001, BR-050, BR-071, BR-113 di traceability; catatan KEPUTUSAN-06.

  Tidak ada AC yang masih bergantung pada push, WhatsApp, atau login Google/Apple. ADR-011 dan ADR-016 konsisten dengan CR-007.
- **KEPUTUSAN-18–37 terdaftar semua, dan isinya sama dengan ADR yang dirujuk.** Dicek satu per satu, termasuk angka teknis: Argon2id 19 MiB/t=2/p=1; sesi 15 menit, 30/90 hari, 7/30 hari, 12 jam; idempotensi 7 hari; HMAC 5 menit; retry 1–30 menit; cek sinkron 5 detik; URL 5 menit; arsip 24 bulan; PITR ≤ 1 menit; backup 35 hari. Daftar modul kritis di `CLAUDE.md` baris 35–47 sama dengan KEPUTUSAN-37 dan arsitektur §12.
- **Semua rujukan terdaftar.** Rujukan SRC, KEPUTUSAN, dan UP di arsitektur, ADR, dan risk register ada di registernya (SRC-15, 18, 26–28, 30–50, 56–63; KEPUTUSAN-02, 05–08, 10, 11, 14–37; UP-16, 17, 27, 28, 37). Rujukan PRD yang dicek cocok dengan bunyinya: US-402 AC3, US-405 AC1, US-802 AC6, US-808, US-1202 AC3, US-1302 AC2/AC4/AC5, US-1306 AC2, US-1309, US-1405 AC1, PB-07, PB-19, PB-20, RF-12, RF-14, dan RF-18.
- **Konsistensi dengan ADR-001–005**:
    - ADR-008 tidak melonggarkan ADR-003/004. Master tetap tanpa `DELETE`, sesuai ADR-003. Proyeksi sejalan dengan "snapshot yang bisa direkonsiliasi" di konsekuensi ADR-004.
    - UUIDv7 tetap dibuat aplikasi (ADR-007 butir 5, ADR-002).
    - ADR-009 dan ADR-010 memenuhi field minimum, pemisahan log, dan larangan data ADR-005.
    - ADR-013 §5 memakai cara anonimisasi ADR-003 ("kolom PII ditimpa").
- **Fakta eksternal dibuka ulang** (daftar URL di bawah) dan didukung halaman:
    - **SRC-34**: superuser dan `BYPASSRLS` selalu melewati RLS; pemilik tabel normalnya juga melewati, kecuali `FORCE ROW LEVEL SECURITY` (PostgreSQL 18).
    - **SRC-37**: NOTIFY dalam transaksi baru dikirim saat commit; payload "must be shorter than 8000 bytes" pada konfigurasi bawaan.
    - **SRC-38**: `archive_timeout` membatasi umur data yang belum diarsipkan; "settings of a minute or so are usually reasonable".
    - **SRC-48**: L2 "Most applications should be striving to achieve this level"; level dipilih organisasi menurut sensitivitas; contoh bank "anything less than Level 3". Tidak ada contoh data kesehatan.
    - **SRC-49**: "Argon2id with a minimum configuration of 19 MiB of memory, an iteration count of 2, and 1 degree of parallelism."
    - **SRC-51**: "The majority of Firebase services run on global Google infrastructure"; lokasi dapat dipilih "for some services". Cloud Messaging tercantum di daftar layanan global.
    - **SRC-54**: "Cloud API processes messages on servers in Meta data centers"; isi pesan "may be stored on Meta data centers internationally while being processed"; "up to 60 minutes"; "Indonesia: ID" ada di halaman registrasi (`data_localization_region`).
    - **SRC-56**: `grant_type` "WAJIB diisi dengan `client_credentials`"; kredensial "setelah melakukan pengajuan via email"; endpoint staging; `expires_in` "3599"; URL produksi tidak tercantum.
    - **SRC-57**: Organization ID, Client ID, dan Client Secret Production; sistem mitra dapat melihat kode akses fasyankes yang memilih sistem RME-nya.
    - **SRC-61**: "SATUSEHAT Platform (SSP) menggunakan ICD-10 versi 2010"; LOINC "digunakan sebagai standar penamaan uji laboratorium"; SNOMED-CT "digunakan sebagai standar penamaan istilah klinis". Kalimat pemakaian ini ada di halaman indeks terminologi, bukan di subhalaman LOINC.
    - Tambahan (dasar CR-007 dan ADR-020): SRC-30 (PostgreSQL 19 masih RC 2), SRC-53, SRC-55, SRC-58, dan SRC-60. Halaman SRC-60 menyebut "4 langkah", tetapi daftarnya berisi 5 butir.
- **Tidak ada klaim "teruji" atau "terbukti"** yang melebihi bukti. "Terverifikasi" di ADR-007 merujuk ke dokumentasi (SRC-31–42), dan kolom "Bukti" di arsitektur §10 menyebut cara pembuktiannya kelak.

## Temuan blokir

| ID | File dan baris | Klaim | Masalah | Kategori §2 | Usulan perbaikan |
| --- | --- | --- | --- | --- | --- |
| FC4-B01 | `adr/ADR-010` baris 24 dan 32; `decision-log.md` baris 39 (KEPUTUSAN-23); `adr/ADR-019` baris 41; `04-arsitektur.md` baris 335 | Retensi application log "Nilai awal 30 hari", security log "Nilai awal 1 tahun", dan backup "nilai awal 35 hari". Di sisi lain, arsitektur §10 NFR-RET-02 menyatakan "desain tidak memusnahkan apa pun sebelum ditetapkan" | (1) BR-112 (BRD baris 312) menyatakan setiap jenis data punya masa retensi dan angkanya `[TIDAK DIKETAHUI]` (PT-24). PT-24 (BRD baris 432) dimiliki CEO berdasarkan pendapat legal. ADR-010 butir 5 sendiri mengakui kedua log memuat data pribadi (alamat IP, ID akun). Jadi angka retensi data pribadi ditetapkan penyusun lewat KEPUTUSAN, padahal BRD menyerahkannya ke CEO. (2) Arsitektur §10 tidak sesuai dengan desainnya sendiri, karena application log terhapus otomatis setelah 30 hari. (3) Masukan BR-112 dari PP 80/2019 (minimal 5–10 tahun untuk data PMSE, status belum diverifikasi) belum ditimbang. Retensi dan penghapusan data termasuk T1 | 1 dan 3 | Pilih salah satu: (a) bawa ketiga angka ke user di gate Tahap 4 sebagai nilai sementara sampai PT-24, lalu ganti labelnya dengan keputusan user; atau (b) ubah menjadi "tidak dihapus otomatis sampai PT-24 ditetapkan", seperti audit trail dan ADR-013 §6, dan catat dampak biaya penyimpanannya. Lalu ubah `04-arsitektur.md` baris 335 supaya sesuai dengan pilihan itu. Cek ulang cukup pada lima lokasi ini |
| FC4-B02 | `adr/ADR-007` baris 30 dan 35; `adr/ADR-011` (butir 4 dan 7, baris 26–33); `decision-log.md` baris 36 (KEPUTUSAN-20) dan 40–41 (KEPUTUSAN-24, -25); `04-risk-register.md` baris 24 (RT-11); `04-arsitektur.md` baris 23, 34 ("keduanya"), §3, §7, §10 (NFR-SEC-02/03/04) | ADR-007: "Tanpa Redis atau message broker di rilis 1", dan Redis masuk alternatif yang ditolak. ADR-011: autentikasi dan sesi dibangun sendiri (token acak dan tabel sesi di database Rumah Sehat), dengan identitas per sistem | Jawaban user T4-Q4 (catatan baris 37–39) adalah "Keycloak + Redis (Recommended)", dengan catatan penyusun "ADR-011 direvisi memakai Keycloak ... ADR-007 direvisi menambah Redis". Saat laporan ditulis, ADR-007, ADR-011, KEPUTUSAN-20, -24, -25, RT-11, dan ringkasan arsitektur masih memuat desain lama yang bertentangan dengan keputusan user. Kemampuan Keycloak dan Redis yang akan dipakai juga belum punya `[SRC]` | 3 (bertentangan dengan jawaban user yang mengubah keputusan); 2 untuk klaim baru | Revisi ADR-007, ADR-011, KEPUTUSAN-20/-24/-25, RT-11, dan arsitektur §1, §3, §6, §7, §10, §11 sesuai T4-Q4. Periksa juga ADR-016 bila Redis menggantikan `LISTEN/NOTIFY` untuk penyebaran WebSocket. Setiap kemampuan Keycloak dan Redis yang menjadi dasar desain wajib `[SRC]` dokumentasi resmi. Contohnya TOTP, pencabutan sesi seketika (KU-10, US-1302 AC4), algoritma hash password (KEPUTUSAN-24 menyebut Argon2id), dan pub/sub atau persistensi Redis. Perbarui juga ringkasan keputusan user di arsitektur §1 (kini T4-Q1–T4-Q4). Cek ulang mencakup bagian yang direvisi beserta SRC barunya |

## Temuan non-blokir (5 butir terpenting)

| ID | Lokasi | Masalah | Usulan |
| --- | --- | --- | --- |
| FC4-N01 | `04-arsitektur.md` baris 167 | "pembayaran setelah kedaluwarsa menjadi RF-13". Menurut PRD §3.6 (baris 248) dan US-802 AC5 (baris 993), pesanan tetap dikonfirmasi bila slot/stok masih tersedia; RF-13 hanya berlaku bila tidak tersedia. ADR-015 baris 28 sudah benar ("ke pemeriksaan slot/stok untuk RF-13") | Samakan dengan ADR-015 |
| FC4-N02 | `04-catatan-arsitektur.md` baris 17; `adr/ADR-016` baris 28; `04-risk-register.md` baris 16 (RT-03) | Konteks T4-Q2 menyebut "email dan video dapat dipenuhi di Indonesia" sebagai bagian dari "sumber resmi yang dibuka (SRC-51–SRC-55 ...)". Kelima sumber itu tidak memuat klaim tersebut, dan opsi server media yang dijalankan sendiri (ADR-016, RT-03) juga tanpa SRC. Bukan blokir, karena pilihan 1–3 di T4-Q1 dan T4-Q2 sama-sama mensyaratkan video di Indonesia, sehingga klaim ini tidak menentukan pilihan user. Arsitektur sudah jujur menulis ketersediaan penyedia terkelola sebagai `[TIDAK DIKETAHUI]` | Tambahkan catatan koreksi di catatan (tanpa mengubah jawaban), sebut di gate bahwa kelayakan video dan email di Indonesia belum diverifikasi, dan jadikan butir awal Tahap 5 bersama RT-02 |
| FC4-N03 | `sources.md` baris 71 (SRC-63); `adr/ADR-013` baris 32 | URL Dukcapil gagal dibuka lima kali (ECONNRESET, termasuk varian `www`). Kutipan "terdiri dari 16 digit angka" **tidak diverifikasi ulang** oleh fact-checker dan tidak boleh dianggap terverifikasi oleh laporan ini. Risikonya rendah, karena hanya menyangkut aturan validasi format | Orkestrator membuka ulang SRC-63 sebelum gate, atau mencari sumber resmi lain. Bila tetap gagal, catat di "Pencarian tanpa hasil" dan biarkan panjang NIK dapat diatur |
| FC4-N04 | `adr/ADR-018` baris 20 dan PRD US-1604 AC2 (baris 1548); `adr/ADR-011` baris 32 dan PRD US-101 AC3 (baris 443) | (a) ADR-018 menetapkan URL unduh 5 menit dan "URL objek tidak pernah disimpan atau ditampilkan permanen", sedangkan US-1604 AC2 menetapkan "Tautan unduhan berlaku 7 hari" `[UP-47]`. Keduanya cocok bila yang berlaku 7 hari adalah ketersediaan salinan di aplikasi, tetapi ADR tidak menyatakannya. (b) ADR-011 butir 6 menulis "pesan galat umum yang tidak membocorkan apakah email terdaftar" setelah menyebut registrasi, padahal US-101 AC3 sengaja mengarahkan email terdaftar untuk masuk atau atur ulang | Tegaskan di ADR-018 bahwa jendela 7 hari adalah ketersediaan salinan dan setiap unduhan menerbitkan URL 5 menit setelah autentikasi ulang. Batasi butir 6 ADR-011 pada login dan atur ulang password (US-103 AC2, AC4) |
| FC4-N05 | PRD baris 1446 dan 20; BRD baris 133–151, 365, 372; `01-business-case.md` baris 138 dan 179; `sources.md` baris 59–63; `04-arsitektur.md` baris 230 dan 34 | Sisa rujukan kosmetik: (1) US-1406 masih menulis "agar login, pembayaran, ..."; (2) PRD §0.1 masih "BRD v1.2"; (3) BRD §4.2 belum mendaftar tiga butir yang dihapus CR-007 (PRD §2.2 sudah); (4) kalimat pembuka BRD §8 tidak menyebut CR-007; (5) BRD §7.3 menyebut T4-Q1 tanpa label `[USER]`; (6) Business Case baris 138 dan R-06 masih menyebut WhatsApp dan push; (7) kolom "Dipakai di" SRC-51–55 menulis arsitektur/ADR, padahal dipakai di catatan dan CR-007; (8) "client credentials per fasyankes" berlabel SRC-56, padahal aspek "per fasyankes" didukung SRC-57; (9) "keduanya oleh user" kini mencakup tiga butir | Rapikan saat revisi FC4-B02. Business Case diperbarui bersama baseline biaya di gate Tahap 4 |

## Butir yang perlu disebut eksplisit di gate (di luar temuan)
- **KEPUTUSAN-26, ASVS Level 2.** SRC-48 menyerahkan pilihan level ke organisasi menurut sensitivitas, dan contoh bank adalah Level 3. Pilihan L2 untuk data kesehatan adalah penerimaan risiko yang sebaiknya disetujui user, CEO, dan CTO secara sadar.
- **KEPUTUSAN-25, MFA wajib untuk tenaga kesehatan.** Keputusan ini menambah langkah onboarding (US-202) dan dapat memengaruhi KPI-02.
- **ADR-015 butir 8.** Biaya transaksi per metode yang dilihat pasien berasal dari tabel tarif yang "diatur Keuangan". Siapa yang mengatur tarif ini tidak disebut di PRD, dan aturan selisih dengan biaya aktual menunggu L-12.
- **ADR-009 butir 9.** Halaman Audit Log hanya memuat 24 bulan terakhir; event yang lebih tua dipulihkan dari arsip.
- **ADR-020 butir 8.** Catatan konsultasi Rumah Sehat tidak dikirim ke SATUSEHAT di rilis 1 sampai L-05 terjawab.

## URL yang benar-benar dibuka (2026-11-18)
1. https://www.postgresql.org/docs/current/ddl-rowsecurity.html (SRC-34)
2. https://www.postgresql.org/docs/current/sql-notify.html (SRC-37)
3. https://www.postgresql.org/docs/current/runtime-config-wal.html (SRC-38)
4. https://raw.githubusercontent.com/OWASP/ASVS/master/5.0/en/0x03-What-is-the-ASVS.md (SRC-48)
5. https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html (SRC-49)
6. https://firebase.google.com/support/privacy (SRC-51)
7. https://developers.facebook.com/documentation/business-messaging/whatsapp/data-privacy-and-security/ (SRC-54)
8. https://developers.facebook.com/documentation/business-messaging/whatsapp/local-storage.md (SRC-54; halaman ini tidak memuat daftar negara)
9. https://developers.facebook.com/documentation/business-messaging/whatsapp/business-phone-numbers/registration (SRC-54; "Indonesia: ID")
10. https://satusehat.kemkes.go.id/platform/docs/id/api-catalogue/authentication/apis/token/ (SRC-56)
11. https://satusehat.kemkes.go.id/platform/docs/id/api-code/access-production/ (SRC-57; halaman Sandbox tidak dibuka)
12. https://satusehat.kemkes.go.id/platform/docs/id/terminology/ (SRC-61)
13. https://satusehat.kemkes.go.id/platform/docs/id/terminology/icd/icd-10/ (SRC-61)
14. https://satusehat.kemkes.go.id/platform/docs/id/terminology/loinc/ (SRC-61; tidak memuat kalimat pemakaian, yang ada di halaman indeks)
15. https://satusehat.kemkes.go.id/platform/docs/id/api-catalogue/onboardings/apis/patient/ (SRC-58)
16. https://satusehat.kemkes.go.id/platform/docs/id/fhir/prerequisites/ (SRC-60)
17. https://developer.android.com/training/monitoring-device-state/doze-standby (SRC-53)
18. https://developers.facebook.com/docs/whatsapp/on-premises/ (SRC-55)
19. https://www.postgresql.org/ (SRC-30)
20. **Gagal**: https://dukcapil.kemendagri.go.id/blog/read/yuk-simak-ini-perbedaan-nik-dan-nit dan varian `www.` (SRC-63), ECONNRESET lima kali

Tidak dibuka ulang (kutipan register hanya dicocokkan dengan klaim): SRC-31–33, 35, 36, 39–47, 50, 52, 59, dan 62.

---

# Fact-check Tahap 04: Arsitektur (bagian 2)

Tanggal: 2026-11-18
Dokumen diperiksa:
- Cek ulang FC4-B01 dan FC4-B02, hanya pada baris yang berubah: `adr/ADR-006`, `ADR-007`, `ADR-009`, `ADR-010`, `ADR-011`, `ADR-013`, `ADR-014`, `ADR-016`, `ADR-019`; `decision-log.md` KEPUTUSAN-20, -23, -24, -25, -28, -35; `04-risk-register.md` RT-11, RT-17, RT-18; `04-arsitektur.md` §1, §3, §5–§8, §10, §11.
- Berkas baru: `docs/04-data-dictionary.md`, `docs/04-erd.md`, `docs/04-threat-model.md`, `docs/adr/ADR-021-keamanan-klien-akses-admin-pipeline.md`, `docs/review/tahap-04-dba.md`.
- Revisi ADR-007, 008, 009, 010, 011, 012, 013, 014, 015, 016, 018, dan 019, diperiksa terhadap PRD v1.1 dan ADR-001–005.
- CR-008 dan CR-009 di `docs/03-prd.md` (US-309 AC5–AC8, NFR-SEC-10) dan `docs/backlog.md`, dicocokkan dengan T4-Q5 dan T4-Q6 di `docs/04-catatan-arsitektur.md`.
- `docs/sources.md` SRC-64–SRC-83; `docs/decision-log.md` KEPUTUSAN-38–71.

Sumber kebenaran: `docs/04-catatan-arsitektur.md` (T4-Q1–T4-Q6), PRD v1.1, BRD v1.3, ADR-001–005
Tingkat ketelitian: T1
Hasil: **LULUS**. Tidak ada blokir baru, FC4-B01 dan FC4-B02 tertutup, dan ada 5 temuan non-blokir.

Catatan metode:
- **Data dictionary** (sekitar 3.180 baris) tidak dibaca utuh.
    - Dibaca penuh: §0 (konvensi, hak role, wadah data, §0.10 Keycloak/Redis), A1.1–A1.2, A6.1–A6.4, A9.8, §B (pengantar, B1, B2), §D, dan §E.
    - Seluruh berkas dipindai dengan pencarian terarah: semua label `[SRC-xx]`, `[USER]`, dan KEPUTUSAN; semua penanda "Tahap 5"; semua angka durasi; kata kunci fitur PostgreSQL (DEFERRABLE, EXCLUDE, BRIN, kolom terhitung, fillfactor, RLS, FK komposit); serta angka retensi.
- **ERD** dibaca bagian pengantar dan §0.1 (kardinalitas terhadap PRD §8.3). Label di bagian lain dipindai. ERD merujuk data dictionary untuk label per kolom.
- **Threat model** dibaca utuh. Dokumen ini adalah penilaian sebelum mitigasi (status §8 TIDAK LULUS). Penerapan mitigasinya diperiksa di ADR yang direvisi dan ADR-021.

## Ringkasan
- Total klaim: sekitar 290 baris berlabel formal di berkas baru, yaitu label `[SRC]`, `[KEPUTUSAN]`, `[USER]`, `[DOK]`, atau `[TIDAK DIKETAHUI]`. Rinciannya: data dictionary 161, threat model 41, ADR-021 12, ERD 9, ditambah sekitar 40 baris revisi ADR, 15 baris CR-008/CR-009, 20 entri SRC, dan 34 entri KEPUTUSAN. Rujukan implisit (US-, AB-, NFR-, ADR-xxx butir n) berfungsi sebagai label `[DOK]` sesuai konvensi §0.1 di tiap dokumen, dan dicek per sampel.
- Terverifikasi: semua baris yang dicek, kecuali butir di tabel non-blokir
- Tanpa sumber: 0
- Sumber tidak mendukung: 0. Satu kutipan register kurang lengkap, tetapi halamannya mendukung klaim (FC4-N2-5).
- Kontradiksi: 0 blokir. Ada 2 ketidakkonsistenan internal di dokumen Tahap 4 yang belum lulus gate (FC4-N2-1, FC4-N2-2)
- Fakta bisnis berlabel asumsi: 0. Keputusan yang menyentuh aturan bisnis (KEPUTUSAN-46, -54, -67) sudah ditandai "perlu konfirmasi user di gate".
- TANPA INDUK: 0. CR-008 menambah AC ke US-309 (induk BR-015), dan CR-009 menambah NFR. Tidak ada story baru.

## Cek ulang blokir bagian 1

| ID | Status | Bukti |
| --- | --- | --- |
| FC4-B01 | **Tertutup** | Kelima lokasi kini menulis `[TIDAK DIKETAHUI]` (PT-24) dengan usulan nilai sementara untuk diputuskan user di gate, dan tidak ada penghapusan otomatis sebelum diputuskan: `adr/ADR-010` baris 24 dan butir 6 (baris 33); `adr/ADR-019` butir 4 (baris 42); `decision-log.md` KEPUTUSAN-23 (baris 39) dan KEPUTUSAN-35 (baris 51); `04-arsitektur.md` §10 NFR-RET-02 (baris 356). Pencarian "30 hari", "1 tahun", dan "35 hari" di ADR dan arsitektur tidak menemukan sisa angka retensi yang ditetapkan. Satu sisa rujukan ada di data dictionary (FC4-N2-3) |
| FC4-B02 | **Tertutup** | ADR-011 ditulis ulang untuk Keycloak. ADR-006 butir 4, ADR-007 butir 4, 6, dan 7, ADR-009 butir 2, ADR-010 (penghasil log), ADR-013 butir 3 dan 5, ADR-014 butir 3, ADR-016 butir 1–2, dan ADR-019 butir 3–4 mengikuti T4-Q4. KEPUTUSAN-20, -24, -25, -28, dan -35 diperbarui. RT-11, RT-17, dan RT-18 diperbarui. Arsitektur §1 (T4-Q4 tercantum), §3, §5, §6, §7, §8, §10 (NFR-SEC-02/03/04, NFR-PERF-03), dan §11 diperbarui. Kemampuan Keycloak dan Redis yang menjadi dasar desain bersumber SRC-64–77; yang menentukan dibuka ulang dan didukung halaman (lihat di bawah). Sisa desain lama di KEPUTUSAN-32 dan arsitektur §4.2 tidak termasuk lokasi blokir dan dicatat sebagai FC4-N2-1 |

## Hal yang diperiksa dan bersih
- **T4-Q5/CR-008.**
    - Jawaban "QR/kode ke domain resmi (Recommended)" diterapkan di US-309 AC5–AC8 (PRD baris 728–731), `backlog.md` CR-008 (baris 273–279), ADR-012 butir 3, data dictionary A9.8, dan arsitektur §3.
    - Penandaan "sudah ditebus" tidak dimasukkan, sesuai catatan baris 54.
    - Alamat halaman dicetak sebagai teks, sesuai penjelasan penyusun di baris 52.
- **T4-Q6/CR-009.** Jawaban "Kunci setelah 15 menit" diterapkan apa adanya di NFR-SEC-10 (PRD baris 1639), ADR-021 butir 1, dan `backlog.md` CR-009 (baris 281–287).
- **KEPUTUSAN-38–71 terdaftar semua** di `decision-log.md` baris 54–87.
    - Isi KEPUTUSAN-38–69 sama dengan data dictionary §E.1 baris 3104–3135. Dicek satu per satu.
    - KEPUTUSAN-70 sama dengan metode threat model §0.2–§0.4.
    - KEPUTUSAN-71 sama dengan ADR-021.
- **Tidak ada nomor yang dirujuk tetapi tidak terdaftar.** Pencarian di seluruh `docs/` tidak menemukan KEPUTUSAN ≥ 72, SRC ≥ 84, atau ASUMSI apa pun.
- **Klaim kemampuan PostgreSQL di data dictionary hanya merujuk SRC-31–SRC-42.**
    - Butir yang belum bersumber ditandai untuk diverifikasi di Tahap 5 (§D.4 a–f; §0.2 FK komposit; §0.6 RLS per role dan RLS tabel terpartisi; A8.13 sasaran FK; BRIN; fillfactor/autovacuum).
    - Tidak ditemukan klaim fitur PostgreSQL tanpa SRC dan tanpa penanda itu.
- **Fakta bisnis di data dictionary dan ERD merujuk PRD/BRD.** Contoh yang dicek: kode status ke label PRD §3.6; biaya rekanan (PB-12, US-810 AC1); Operator RME (PRD baris 69); klaim satu per pesanan (UP-36, PRD baris 328); RF-14 seluruh nominal (PRD baris 212); pembukaan kembali tiket 7 hari (US-1104 AC3); salinan data 7 hari (US-1604 AC2).
    - Fakta yang tidak dinyatakan PRD ditulis `[TIDAK DIKETAHUI]` (§E.3), misalnya batas usia anak, hari kerja mingguan, dan sistem kode wilayah.
    - Tidak ada keputusan bisnis penyusun yang tidak ditandai untuk user. Tiga keputusan berikut sudah ditandai "perlu konfirmasi user di gate":
        - KEPUTUSAN-46: email yang sama boleh dipakai di realm `pasien` dan `nakes`.
        - KEPUTUSAN-54: status tujuan pembukaan kembali tiket.
        - KEPUTUSAN-67: tafsiran AB-36, yaitu pengecualian RF-13 dan RF-14.
- **Revisi ADR terhadap PRD dan ADR-001–005**: tidak ada kontradiksi substantif.
    - ADR-008 kelas Log: penghapusan hanya oleh `privacy_worker` setelah retensi, dan tidak aktif sebelum PT-24. Kelas Log berada di luar cakupan ADR-004, yang hanya mengatur data transaksi.
    - Sub-kelas Master snapshot tidak melonggarkan ADR-003.
    - ADR-015 butir 5–6 sesuai RF-12, RF-14 (PRD baris 212, US-802 AC3), dan AB-36, dengan tafsiran RF-13/RF-14 dibawa ke user.
    - ADR-018 butir 3 kini sesuai US-1604 AC2. ADR-011 butir 4 kini sesuai US-101 AC3. ADR-021 butir 1 mengecualikan QR appointment offline (US-403 AC2), satu-satunya kebutuhan offline di PRD.
    - Penyimpangan tafsir atas ADR-002 dan ADR-003 dinyatakan terbuka di ADR (lihat butir gate).
- **Threat model**:
    - Label `[SRC]` (SRC-34, 40, 46, 50, 71, 77–83) terdaftar dan isinya sesuai kutipan register.
    - Mitigasi M-01 (a, b, d), M-02, M-03, M-05–M-09, M-13, M-17, M-19, M-23, H-06, dan H-10 tercermin di ADR-007, -009, -010, -011, -014, -015, -016, -018, -021, dan di DoD `backlog.md` baris 20–21.
    - M-04 dan M-01 (c) dibawa ke user (T4-Q5, T4-Q6).
- **Review DBA.** Ringkasan konsisten dengan data dictionary (KEPUTUSAN-63, -66, -67, -68, dan -69; `rme_retention_policy`; `verification_code_bidx_key_version`). Putaran perbaikan tidak melebihi dua.
- **Tidak ada klaim "teruji" atau "terbukti"** di berkas baru yang melebihi bukti.
- **Fakta eksternal dibuka ulang** pada 2026-11-18 dan didukung halaman:
    - **SRC-69** (brute force):
        - Pengaturan yang dikutip ada: "Max Login Failures": "The maximum number of login failures."; "Wait Increment": "The time added to the time a user is temporarily disabled when the user's login attempts exceed Max Login Failures."; "Max Wait": "The maximum time a user is temporarily disabled."
        - Ada mode permanen, sementara, dan campuran, dan halaman menyatakan "Brute force detection is disabled by default."
        - Catatan: halaman tidak memakai istilah "berturut-turut". Hitungan direset oleh login berhasil dan oleh Failure Reset Time. ADR-011 butir 4 sudah menjadwalkan uji kesesuaiannya di Tahap 8.
    - **SRC-70** (Server Admin Guide "Version 26.8.2"):
        - Hash dan OTP: "Argon2 (default for non-FIPS deployments)"; "The **Browser - Conditional 2FA** sub-flow executes only when the user has a configured OTP credential."; Configure OTP: "The user must configure a one-time password generator on their mobile device"; "a number of sequential one-time passwords (currently 12)".
        - Verifikasi dan atur ulang: "The user must verify their email account."; "Specifies independent timeout for forgot password."
        - Sesi dan token: SSO Session Idle, SSO Session Max, dan Access Token Lifespan beserta deskripsinya.
        - Logout dan realm: "Keycloak sends a HTTP POST request containing a logout token to all clients logged into Keycloak."; "Realms are isolated from one another and can only manage and authenticate the users that they control."
    - **SRC-72** (OpenAPI resmi, `info.version` "1.0", dibaca dalam empat potongan):
        - Endpoint user: `POST /admin/realms/{realm}/users` "Create a new user Username must be unique."; `PUT .../users/{user-id}` "Update the user"; `DELETE .../users/{user-id}` "Delete the user"; `POST .../users/{user-id}/logout` "Remove all user sessions associated with the user Also send notification to all clients that have an admin URL".
        - Endpoint event: `GET .../events` "Get events Returns all events, or filters them based on URL query parameters listed here"; `GET .../admin-events` ada.
        - Skema `UserRepresentation` memuat `enabled` bertipe `boolean`.
    - **SRC-74**: "Redis' Pub/Sub exhibits _at-most-once_ message delivery semantics."; "If the subscriber is unable to handle the message (for example, due to an error or a network disconnect) the message is forever lost."
    - **SRC-77**: §5 "native apps MUST use an external user-agent to perform OAuth authorization requests."; §6 "Public native app clients MUST implement the Proof Key for Code Exchange".
    - **SRC-79** (BCP 225): §3.1 "Libraries MUST enable the caller to specify a supported set of algorithms and MUST NOT use any other algorithms"; §3.8 "the application MUST reject the JWT"; §3.9 "MUST validate the audience value" ... "if the audience value is not present or not associated with the recipient, it MUST reject the JWT."
    - **SRC-82**: keempat kutipan register ditemukan, yaitu rahasia tidak di-hardcode, identitas pipeline dengan hak minimum, review PR yang tidak dapat dilewati, dan "Version pinning should be performed" dengan pemeriksaan hash.
    - Tambahan:
        - **SRC-71**: kelima kutipan ditemukan, termasuk `pkce-enforcer` "makes sure that client use PKCE with secured S256 algorithm".
        - **SRC-73**: "Toggle *Save events* to *ON*"; "Specify the length of time to store events in the *Expiration* field"; Update Password, Update TOTP, dan Remove TOTP ditandai "(deprecated)".
        - **Halaman SRC-39**: kalimat batasan constraint unik di tabel terpartisi ditemukan (lihat FC4-N2-5).

## Temuan blokir

Tidak ada.

## Temuan non-blokir (5 butir terpenting)

| ID | Lokasi (file, baris) | Klaim | Masalah | Usulan |
| --- | --- | --- | --- | --- |
| FC4-N2-1 | `decision-log.md` baris 48 (KEPUTUSAN-32); `04-arsitektur.md` §4.2 baris 190–193 dan diagram §6 baris 262, 271 | KEPUTUSAN-32: "pesan disimpan terenkripsi lalu `NOTIFY`". Diagram §4.2: "simpan terenkripsi + NOTIFY(id)", "DB-->>RT: NOTIFY". Diagram §6: satu node Redis yang dipakai `api`, `realtime`, dan RME | Isi ini sisa desain sebelum T4-Q4 dan M-06. ADR-016 butir 2, ADR-007 butir 4, KEPUTUSAN-20, dan arsitektur §10 (NFR-PERF-03) memakai Redis Pub/Sub sebagai jalur utama, dengan `LISTEN/NOTIFY` hanya sebagai cadangan. ADR-007 butir 6, arsitektur §7 baris 298, dan KEPUTUSAN-20 menetapkan instans Redis terpisah untuk Rumah Sehat dan RME. KEPUTUSAN-32 adalah entri yang disetujui user di gate, jadi isinya perlu sama dengan ADR | Samakan KEPUTUSAN-32 dan diagram §4.2 dengan ADR-016 butir 2. Pecah node Redis di diagram §6 menjadi dua instans |
| FC4-N2-2 | `adr/ADR-011` baris 89; `adr/ADR-007` baris 39; `decision-log.md` baris 40 (KEPUTUSAN-24); `04-data-dictionary.md` baris 303 | ADR-011: saat keluar, "backend menandai sesi itu dicabut di Redis sampai token kedaluwarsa". ADR-007: data Redis boleh hilang, "Satu-satunya akibatnya, pengguna web harus login ulang". KEPUTUSAN-24: "daftar pencabutan di Redis/DB" | Data dictionary §0.10 menyatakan tidak ada tabel sesi autentikasi di database aplikasi, jadi daftar pencabutan untuk logout hanya ada di Redis. Bila Redis kehilangan data atau tidak tersedia, token akses lama (umur sampai 5 menit) dapat diterima lagi. Akibatnya kalimat "satu-satunya akibatnya" di ADR-007 tidak tepat, dan "Redis/DB" di KEPUTUSAN-24 tidak sesuai dengan desain. US-103 AC5 ("memakai token lama ditolak") tidak terpenuhi pada skenario gangguan itu. Bukan blokir, karena dokumen Tahap 4 belum lulus gate dan dampaknya terbatas pada umur token akses | Pilih salah satu dan tuliskan di ADR-011 butir 8: (a) simpan pencabutan logout di tabel kelas Operasional sebagai sumber kebenaran, dengan Redis sebagai cache; atau (b) bila Redis tidak tersedia atau kosong, periksa sesi lewat introspeksi Keycloak `[SRC-71]` atau tolak sampai token kedaluwarsa (fail-closed). Lalu samakan ADR-007 butir 6 dan KEPUTUSAN-24 |
| FC4-N2-3 | `04-data-dictionary.md` baris 7, 2269, 3148; `04-erd.md` baris 7; `04-arsitektur.md` baris 10; `adr/ADR-007` baris 17; rujukan butir ADR-012 di data dictionary baris 129, 164, 203 | Sisa rujukan yang basi | Rinciannya: (1) "nilai awal 35 hari, ADR-019" untuk masa simpan backup, padahal ADR-019 kini `[TIDAK DIKETAHUI]` dengan usulan untuk gate (sisa FC4-B01); (2) "ADR-007 dan ADR-011 sedang direvisi orkestrator"; (3) ERD "Putaran perbaikan 1 ... sudah diterapkan", padahal putaran 2–3 juga; (4) "ADR-006–ADR-020", tanpa ADR-021; (5) ADR-007 butir 1 menyebut "keempat penyimpanan data", sedangkan arsitektur §8 dan KEPUTUSAN-20 menyebut lima database termasuk `keycloak`; (6) penomoran ADR-012 bergeser setelah butir 3 ditambahkan: RLS RME kini butir 6, role kini butir 8, dan RLS audit RME kini butir 9, sedangkan data dictionary masih merujuk butir 5 dan 7 | Ganti (1) menjadi "masa simpan backup (PT-24, usulan sementara di ADR-019)". Rapikan (2)–(6) |
| FC4-N2-4 | `04-arsitektur.md` §1 baris 35–40 dan §14 baris 405–411; `decision-log.md` baris 43 (KEPUTUSAN-27) dan 47 (KEPUTUSAN-31) | Ringkasan keputusan user dan daftar CR | (1) Arsitektur §1 hanya mendaftar T4-Q1 sampai T4-Q4. (2) §14 hanya memuat CR-007, padahal CR-008 dan CR-009 juga mengubah PRD yang sudah lulus gate. (3) KEPUTUSAN-27 belum menyebut endpoint verifikasi resep publik dan RLS audit RME, walaupun ADR-012 butir 3 dan 9 berlabel KEPUTUSAN-27. (4) KEPUTUSAN-31 masih "satu refund per pesanan kecuali RF-12", padahal KEPUTUSAN-67 memperluasnya | Tambahkan T4-Q5, T4-Q6, CR-008, dan CR-009 di arsitektur §1 dan §14. Tambahkan rujukan silang ke KEPUTUSAN-67 di KEPUTUSAN-31, dan ringkasan butir 3 dan 9 di KEPUTUSAN-27 |
| FC4-N2-5 | `sources.md` baris 47 (SRC-39); `adr/ADR-009` baris 30; `04-data-dictionary.md` baris 2322 dan 3089 (§D.4 d) | ADR-009: PK komposit "karena kolom partisi harus ikut di constraint unik", di samping label `[SRC-39]`. Data dictionary: hal ini diverifikasi di Tahap 5, "belum ada SRC" | Halaman SRC-39 (PostgreSQL 18.7) mendukung klaim: "To create a unique or primary key constraint on a partitioned table, the partition keys must not include any expressions or function calls and the constraint's columns must include all of the partition key columns." Kutipan ini belum ada di register, sehingga label ADR-009 belum dapat dilacak dari register dan penanda Tahap 5 di data dictionary tidak lagi perlu | Tambahkan kutipan itu ke SRC-39. Hapus butir (d) dari §D.4 dan ubah kalimat di baris 2322 menjadi rujukan SRC-39 |

## Butir yang perlu disebut eksplisit di gate (di luar temuan)
- **US-309 AC7–AC8 (CR-008).**
    - Rincian berikut adalah rincian penyusun di bawah label `[USER] T4-Q5`: data yang tampil ke pemegang kode tanpa login (status, tanggal, dokter dan nomor SIP, klinik, **daftar obat**, dan nama pasien yang disamarkan), batas laju, serta pencatatan IP "pemeriksa anonim". Catatan T4-Q5 (baris 50–54) hanya memuat pilihan "QR/kode ke domain resmi" dan penjelasan bahwa buktinya adalah data server.
    - Daftar obat adalah data kesehatan yang tampil ke pihak ketiga. AC7 sudah menyatakan isi final mengikuti L-05/L-08, dan header PRD menyatakan teks revisi diperiksa di gate. Daftar field ini perlu disetujui user secara eksplisit.
- **Tafsir atas ADR yang sudah diterima.** Arsitektur §14 menulis "ADR-001–ADR-005 tidak diubah", padahal ada dua tafsir terbuka. Keduanya sudah ditulis di ADR dan perlu diterima user secara sadar:
    - PK komposit `(event_id, recorded_at)` untuk `audit_event` dinyatakan memenuhi ADR-002 (ADR-009 butir 4, KEPUTUSAN-56, F-01).
    - `data_key` adalah Master tanpa soft delete, dengan pemusnahan lewat `shredded_at`/`shred_reason` (KEPUTUSAN-44), sebagai pengecualian ADR-003.
- **KEPUTUSAN-15 untuk event autentikasi.**
    - Dengan Keycloak, E-01–E-04 dicatat lebih dulu di penyimpanan event Keycloak, lalu diambil worker (ADR-009 butir 2, ADR-011 butir 10). Login tidak dapat dibatalkan aplikasi bila event gagal disimpan.
    - SRC-73 hanya menyatakan event dapat disimpan dengan masa simpan. Apakah login gagal bila penyimpanan event gagal belum diverifikasi.
    - Sebutkan di gate sebagai pengecualian KEPUTUSAN-15, dan verifikasi perilakunya di Tahap 5.
- **Status threat model.** `04-threat-model.md` §8 masih TIDAK LULUS, dan pemeriksaan ulang baris yang berubah (diminta di §8 baris 532) belum tercatat. Catatan Tahap 4 baris 48 menyatakan mitigasinya sudah diterapkan, dan fact-checker mengonfirmasi penerapan itu di ADR (lihat di atas). Status akhir threat model sebaiknya diperbarui oleh `security-architect` sebelum gate.
- **Nilai yang menunggu keputusan user:**
    - usulan retensi application log 30 hari, security log 1 tahun, dan backup 35 hari (FC4-B01);
    - KEPUTUSAN-46, -54, dan -67;
    - butir gate bagian 1 yang masih berlaku: KEPUTUSAN-25, KEPUTUSAN-26, dan ADR-015 butir 8.

## URL yang benar-benar dibuka (2026-11-18, bagian 2)
1. https://raw.githubusercontent.com/keycloak/keycloak/main/docs/documentation/server_admin/topics/threat/brute-force.adoc (SRC-69)
2. https://www.keycloak.org/docs/latest/server_admin/index.html (SRC-70; dibaca per potongan offset 0–600000)
3. https://www.keycloak.org/docs-api/latest/rest-api/openapi.yaml (SRC-72; potongan offset 0, 100000, 200000, 300000)
4. https://redis.io/docs/latest/develop/pubsub/ (SRC-74)
5. https://www.rfc-editor.org/rfc/rfc8252.html (SRC-77)
6. https://www.rfc-editor.org/rfc/rfc8725.html (SRC-79)
7. https://cheatsheetseries.owasp.org/cheatsheets/CI_CD_Security_Cheat_Sheet.html (SRC-82)
8. https://www.keycloak.org/securing-apps/oidc-layers (SRC-71)
9. https://raw.githubusercontent.com/keycloak/keycloak/main/docs/documentation/server_admin/topics/events/login.adoc (SRC-73)
10. https://www.postgresql.org/docs/current/ddl-partitioning.html (SRC-39, kalimat batasan constraint unik)

Tidak dibuka ulang (kutipan register hanya dicocokkan dengan klaim): SRC-64, 65, 66, 67, 68, 75, 76, 78, 80, 81, dan 83.
