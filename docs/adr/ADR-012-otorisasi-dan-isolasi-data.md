# ADR-012: Otorisasi berbasis peran dan relasi, dengan isolasi per rekanan dan per fasyankes

Status: Diusulkan
Tanggal: 2026-11-18

## Konteks

- Matriks akses PRD §3.1 membatasi data per peran: tenaga kesehatan hanya sesi yang ditanganinya; admin rekanan hanya data rekanannya; tidak ada admin Rumah Sehat yang melihat isi klinis; foto KTP/KK hanya untuk Verifikator; Operator RME tidak dapat melihat rekam medis `[DOK: 03-prd.md]` `[UP-17]`.
- Percobaan akses dengan mengganti ID harus ditolak tanpa membocorkan keberadaan data, dan percobaan ke data sensitif dicatat (KU-03) `[DOK: 03-prd.md]`.
- Isolasi antar-rekanan dan antar-fasyankes wajib diuji (NFR-SEC-06), dan hak akses mengikuti prinsip hak minimum (NFR-SEC-07) `[DOK: 03-prd.md]`.
- Multi-tenancy adalah keputusan data yang ditetapkan di Tahap 4 `[DOK: .claude/skills/runbook/references/tahap-04-arsitektur.md]`.

## Keputusan

1. **Model otorisasi**: peran (RBAC) ditambah pemeriksaan relasi terhadap objek. Setiap endpoint menyatakan kebijakannya, dan permintaan tanpa kebijakan ditolak (*default deny*). Kebijakan dievaluasi di satu modul otorisasi di backend, tidak tersebar di handler `[KEPUTUSAN-27]`. Relasi yang diperiksa:
   - **kepemilikan**: pasien atas dirinya dan anggota keluarganya;
   - **penugasan**: tenaga kesehatan atas sesi dan kunjungan yang ditugaskan kepadanya, dalam batas waktu yang berlaku (misalnya nomor HP pasien home visit hanya di hari kunjungan, US-503);
   - **tenant**: admin rekanan atas data rekanannya; pengguna RME atas data fasyankesnya;
   - **sub-peran admin**: tujuh sub-peran admin Rumah Sehat sesuai §3.1.
2. **Data sensitif** (§3.1 kolom "Sensitif") hanya dibuka lewat fungsi akses yang memeriksa kebijakan dan menulis event E-05 sebelum data dikirim (ADR-009). Akses yang ditolak ke data sensitif dicatat dengan hasil gagal `[KEPUTUSAN-27]`.
3. **Endpoint publik tanpa login** hanya untuk konten yang memang publik (UP-21) dan halaman verifikasi resep (US-309 AC6–AC8, CR-008 `[USER] T4-Q5`). Halaman verifikasi resep hanya dapat dibuka dengan kode verifikasi acak dari PDF, diberi batas laju per sumber, menampilkan data minimal, dan setiap akses dicatat E-05 `[KEPUTUSAN-27]`.
4. **Respons penolakan**: objek yang tidak boleh diakses dijawab seperti objek yang tidak ada (HTTP 404), sehingga keberadaannya tidak bocor (KU-03). Aksi yang dikenal tetapi tidak diizinkan untuk peran itu dijawab 403 `[KEPUTUSAN-27]`.
5. **Isolasi di database Rumah Sehat**: skema bersama. Setiap query portal rekanan wajib melewati lapisan repository yang meminta `partner_id` dari sesi, bukan dari parameter permintaan. Kepatuhan diperiksa dengan test isolasi otomatis yang mencoba mengakses data rekanan lain lewat ID untuk setiap endpoint portal (NFR-SEC-06). Row Level Security tidak dipakai di database ini, karena hanya sedikit endpoint yang bersifat per tenant (laporan, profil, keluhan, pengaturan konsultasi) `[KEPUTUSAN-27]`.
6. **Isolasi di database RME minimalis**: skema bersama, dengan kolom `fasyankes_id` di setiap tabel milik tenant, dan **Row Level Security** sebagai lapisan kedua di bawah pemeriksaan aplikasi `[KEPUTUSAN-27]`:
   - kebijakan RLS membatasi baris ke `fasyankes_id` milik pengguna, dari pengaturan sesi database yang dipasang per transaksi oleh aplikasi;
   - role aplikasi bukan pemilik tabel, tidak punya atribut `BYPASSRLS`, dan tabel memakai `FORCE ROW LEVEL SECURITY`, karena superuser, role `BYPASSRLS`, dan pemilik tabel normalnya melewati RLS `[SRC-34]`;
   - fungsi Operator RME (onboarding fasyankes) memakai jalur dan role database terpisah yang tidak punya hak `SELECT` atas tabel rekam medis, kunjungan, dan hasil lab.
7. **Pemisahan tugas**: admin tidak dapat mengubah peran akunnya sendiri; Super Admin terakhir dan Admin Fasyankes terakhir tidak dapat dinonaktifkan (US-208, US-1302); perubahan rekening rekanan berlaku setelah disetujui Verifikator (US-1202 AC3); export audit log hanya oleh Auditor (US-1503).
8. **Hak database minimum**: role aplikasi, role relay audit, role pembaca audit, role verifikasi audit, role pekerja privasi dan role rotasi kunci (ADR-013), serta role migrasi dipisah, dengan hak per tabel sesuai kelasnya (ADR-008). Di RME ada dua role tambahan dengan cakupan sempit, yaitu `rme_auth` (memetakan subjek Keycloak ke pengguna lintas tenant saat login) dan `rme_worker` (memproses antrean lintas tenant). Keduanya tidak punya akses ke tabel rekam medis kecuali yang diperlukan job-nya, dan rinciannya ada di data dictionary §0.6 `[KEPUTUSAN-55]`.
9. **Isolasi log akses di database audit RME**: Admin Fasyankes membaca event fasyankesnya lewat role `audit_tenant_reader` yang dibatasi RLS berdasarkan `tenant_id`. Bila RLS pada tabel terpartisi ternyata tidak memenuhi kebutuhan saat diverifikasi di Tahap 5, dipakai fungsi baca per tenant. Auditor internal tetap memakai role pembaca audit biasa `[KEPUTUSAN-27]`.

## Alternatif yang ditolak

- **Database atau skema terpisah per fasyankes di RME**: migrasi dan operasi berlipat sesuai jumlah fasyankes, terlalu berat untuk tim. RLS memberi lapisan kedua dengan satu skema.
- **RLS juga di database Rumah Sehat**: hampir semua tabel Rumah Sehat tidak bersifat per tenant tunggal, karena satu pesanan melibatkan pasien, rekanan, dan tenaga kesehatan. Biaya pemasangan konteks per transaksi tidak sebanding dengan manfaatnya.
- **Mengandalkan kerahasiaan ID**: UUIDv7 sulit ditebak, tetapi otorisasi tidak boleh bergantung pada ID yang tidak diketahui.

## Konsekuensi

- Satu titik kebijakan memudahkan review dan test. Setiap endpoint baru wajib menambah kebijakan dan test penolakan (Definition of Done Tahap 8).
- Test isolasi otomatis menjadi bagian `scripts/verify.sh` untuk endpoint portal dan RME (rincian Tahap 5).
- Konfigurasi RLS dan role database RME adalah modul kritis (T1) yang direview DBA dan keamanan.
