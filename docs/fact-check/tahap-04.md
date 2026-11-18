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
