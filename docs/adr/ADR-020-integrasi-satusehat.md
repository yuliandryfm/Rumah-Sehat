# ADR-020: Integrasi SATUSEHAT dari RME minimalis atas nama fasyankes, lewat antrean asinkron

Status: Diterima (gate Tahap 4, 2026-11-18 `[USER] T4-G5`)
Tanggal: 2026-11-18

## Konteks

- Rekam medis RME minimalis wajib sesuai ketentuan SATUSEHAT `[USER] T3-Q1b`, dan dikirim ke SATUSEHAT atas nama fasyankes pengguna (US-1309, UP-28). Status pengiriman: Menunggu kredensial → Antre → Terkirim atau Gagal. Percobaan ulang sampai 24 jam. Koreksi dikirim sebagai pembaruan atas data yang sama `[KEPUTUSAN-16]` `[DOK: 03-prd.md]`.
- Prasyarat registrasi sistem RME: nomor PSE, nomor afiliasi SNOMED CT, kode registrasi fasyankes mitra atau surat pernyataan resmi, survei keamanan, dan verifikasi `[SRC-27]` (PT-32).
- SATUSEHAT memakai HL7 FHIR `[SRC-26]`. Path endpoint memuat `/fhir-r4/v1/` `[SRC-58]`; pernyataan eksplisit versi FHIR tidak ditemukan (`docs/sources.md`, pencarian tanpa hasil).
- Autentikasi API memakai OAuth2 `client_credentials`. Client ID dan client secret diperoleh setelah pengajuan via email `[SRC-56]`. Kode akses (Organization ID, Client ID, Client Secret) ditampilkan per fasyankes, dan sistem mitra dapat melihat kode akses fasyankes yang terhubung. Ada lingkungan Sandbox (Staging) dan Production `[SRC-57]`.
- Sebelum mengirim data pendaftaran pasien dan diagnosis, perlu registrasi Organization dan Location serta pengambilan nomor IHS `[SRC-60]`. Pasien dan praktisi dicari berdasarkan NIK `[SRC-58]` `[SRC-59]`.
- Terminologi: ICD-10 versi 2010 untuk diagnosis, LOINC untuk penamaan uji laboratorium, dan SNOMED CT untuk istilah klinis `[SRC-61]`. KFA adalah kamus farmasi dan alat kesehatan Kemenkes; apakah wajib dipakai untuk kode obat belum diverifikasi `[SRC-62]`.
- Playbook rawat jalan memuat resource antara lain Encounter, Condition, Observation, Procedure, MedicationRequest, ServiceRequest, dan Composition `[SRC-28]`. Bagian pemetaan variabel belum dibaca.
- Daftar final kelompok data dan terminologi ditetapkan di Tahap 4 setelah L-05 (US-1306 AC2). L-05 belum terjawab (PT-20) `[DOK: 03-prd.md]`.

## Keputusan

1. **Pengirim adalah RME minimalis, bukan backend Rumah Sehat.** Modul SATUSEHAT berada di RME minimalis dan memakai kredensial masing-masing fasyankes (Organization ID, Client ID, Client Secret), yang disimpan terenkripsi (ADR-013). Token akses disimpan sementara sampai kedaluwarsa `[KEPUTUSAN-36]`.
2. **Lingkungan**: RME staging ke SATUSEHAT Sandbox (Staging), dan RME production ke SATUSEHAT Production `[SRC-57]`. URL dasar production diambil dari portal SATUSEHAT saat registrasi (PT-32) `[KEPUTUSAN-36]`.
3. **Data prasyarat per fasyankes** disimpan di RME: Organization ID; ID Location untuk setiap lokasi pelayanan; nomor IHS praktisi yang dicari dengan NIK tenaga kesehatan fasyankes (US-1302 AC2); dan nomor IHS pasien yang dicari dengan NIK pasien `[SRC-58]` `[SRC-59]` `[SRC-60]` `[KEPUTUSAN-36]`.
4. **NIK pasien di RME.** Pencarian pasien SATUSEHAT memerlukan NIK `[SRC-58]`. Kontrak integrasi (ADR-014) menyediakan field NIK penerima layanan yang ditandai sebagai data pribadi. Field ini hanya diisi untuk fasyankes yang membutuhkannya untuk rekam medis, dan baru diaktifkan setelah L-05 dan L-08 menjawab dasar pengirimannya (§3.1 baris NIK) `[KEPUTUSAN-36]`.
5. **Model rekam medis terstruktur dan berkode.** Setiap kelompok data minimal (UP-27) disimpan sebagai field terstruktur dengan kode (sistem kode, kode, tampilan) agar dapat dipetakan ke resource FHIR: kunjungan ke Encounter; keluhan dan diagnosis ke Condition dengan ICD-10 2010; tanda vital dan hasil lab ke Observation, dengan LOINC untuk uji lab; tindakan ke Procedure; peresepan ke MedicationRequest; permintaan lab ke ServiceRequest; resume ke Composition `[SRC-28]` `[SRC-61]`. Pemetaan variabel final ditetapkan saat pengembangan US-1306 dari playbook SATUSEHAT, setelah L-05 `[KEPUTUSAN-36]`.
6. **Pengiriman asinkron lewat antrean** (ADR-007) `[KEPUTUSAN-36]`:
   - Saat rekam medis difinalkan, dibuat catatan pengiriman (kelas Log) berstatus Antre, atau Menunggu kredensial bila kredensial fasyankes belum lengkap.
   - Job mengirim resource berurutan (kunjungan dulu, lalu resource yang merujuknya), menyimpan ID resource yang dikembalikan SATUSEHAT, lalu memakai ID itu untuk mengirim koreksi sebagai pembaruan (KEPUTUSAN-16).
   - Percobaan ulang dengan jeda membesar sampai 24 jam sejak percobaan pertama.
   - Galat digolongkan: gangguan sementara dicoba ulang; galat data klinis dikembalikan ke penulis; galat identitas atau kredensial fasyankes ditangani Admin Fasyankes (US-1309 AC2).
   - Setiap pengiriman dicatat E-17 (jenis data dan tujuan, tanpa isi).
7. **Terminologi** disimpan sebagai tabel acuan (kelas Master) yang dimuat dari sumber resmi. Pemakaian SNOMED CT bergantung pada nomor afiliasi yang menjadi syarat registrasi (PT-32). Pemakaian KFA untuk obat `[TIDAK DIKETAHUI]` dan diverifikasi saat pengembangan US-1306 `[KEPUTUSAN-36]`.
8. **Catatan konsultasi online di Rumah Sehat** memakai struktur berkode yang sama (AB-72, UP-16), tetapi tidak dikirim ke SATUSEHAT di rilis 1 sampai L-05 menetapkan statusnya `[KEPUTUSAN-36]`.

## Alternatif yang ditolak

- **Backend Rumah Sehat mengirim ke SATUSEHAT untuk semua fasyankes**: Rumah Sehat bukan sistem rekam medis fasyankes, dan prinsip pemisahan T3-Q14 dilanggar.
- **Pengiriman sinkron saat tenaga kesehatan menekan "final"**: gangguan SATUSEHAT akan menghambat pelayanan, padahal KEPUTUSAN-16 sudah menyediakan status dan percobaan ulang.
- **Rekam medis teks bebas tanpa kode**: tidak dapat dipetakan ke ketentuan SATUSEHAT yang diminta user (T3-Q1b).

## Konsekuensi

- US-1309 bergantung pada registrasi PT-32, termasuk nomor PSE dan afiliasi SNOMED CT, yang lama prosesnya `[TIDAK DIKETAHUI]`.
- Cara setiap fasyankes memperoleh kredensial dan menghubungkannya ke RME sebagai sistem mitra baru terverifikasi sebagian `[SRC-57]`, dan diperjelas saat registrasi.
- Batas laju API SATUSEHAT belum diketahui. Job pengiriman memakai batas laju yang dapat diatur.
- Modul SATUSEHAT dan model rekam medis RME adalah modul kritis (T1).
