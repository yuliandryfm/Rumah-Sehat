# ADR-009: Mekanisme audit trail, yaitu pencatatan di aplikasi, outbox atomik, store terpisah, dan hash berantai

Status: Diusulkan
Tanggal: 2026-11-18

## Konteks

- ADR-005: audit trail append-only, terpisah dari tabel aplikasi, tidak dapat diubah atau dihapus siapa pun lewat aplikasi, dengan field minimum dan larangan data tertentu. Mekanisme, storage, perlindungan dari manipulasi, dan retensi ditetapkan di Tahap 4 `[DOK: adr/ADR-005-audit-trail-append-only.md]`.
- PRD §7 merinci 17 jenis event (E-01–E-17), field per event (termasuk subjek data dan ID korelasi), masking, dan hak akses. E-05 mencatat **akses baca** ke data sensitif `[DOK: 03-prd.md]`.
- Bila event tidak dapat dicatat: aksi pengguna dibatalkan; aksi sistem atau integrasi dicoba ulang dari antrean; efek eksternal yang sudah terjadi tidak boleh hilang `[KEPUTUSAN-15]`.
- Salinan data pribadi untuk subjek data memuat rekam jejak pemrosesan (US-1604 AC1, UU PDP Pasal 32 ayat (1)) `[DOK: 03-prd.md]` `[SRC-15]`.
- Masa retensi audit trail `[TIDAK DIKETAHUI]` (PT-24) `[DOK: 03-prd.md]` §7.4.

## Keputusan

**1. Event dibuat oleh aplikasi**, lewat satu pustaka audit yang dipakai semua modul Rumah Sehat dan RME minimalis. Pencatatan tidak memakai trigger database atau CDC, karena keduanya tidak melihat aktor, alamat IP, perangkat, subjek data, dan ID korelasi, dan tidak dapat mencatat akses baca (E-05) `[KEPUTUSAN-22]`.

**2. Jalur tulis atomik lewat outbox** `[KEPUTUSAN-22]`:
- Aksi yang mengubah data: event ditulis ke tabel `audit_outbox` di database aplikasi dalam **transaksi yang sama** dengan perubahan bisnisnya. Kalau penulisan event gagal, seluruh transaksi batal dan pengguna menerima pesan gagal (KEPUTUSAN-15).
- Akses baca data sensitif (E-05): event ditulis dan di-commit sebelum data dikirim ke peminta. Kalau gagal, akses ditolak.
- Aksi yang gagal (misalnya E-02 login gagal, atau akses ditolak menurut KU-03): event ditulis di transaksi tersendiri dan dicoba ulang, tanpa mengubah hasil aksi.
- Event login, logout, login gagal, dan perubahan kredensial terjadi di Keycloak. Keycloak menyimpannya sebagai penyangga, lalu worker mengambilnya lewat Admin REST API dan menulisnya ke outbox secara idempoten (ADR-011 butir 10) `[USER] T4-Q4`.
- Aksi sistem dan integrasi: job tidak ditandai selesai sebelum event-nya tertulis. Pesan integrasi masuk dibalas galat sementara supaya dikirim ulang. Efek eksternal yang sudah terjadi (misalnya refund yang sudah dieksekusi payment gateway) dicatat dari job yang terus dicoba ulang.
- Role database aplikasi hanya punya hak `INSERT` di `audit_outbox`, tanpa `SELECT`, `UPDATE`, atau `DELETE`.

**3. Relay ke audit store.** Relay berjalan sebagai deployment terpisah yang hanya memegang kredensial `audit_relay` dan `audit_writer`; worker bisnis tidak memegangnya (threat model M-23). Satu pekerja relay (satu instans aktif pada satu waktu) membaca outbox berurutan, menulis event ke audit store, lalu menghapus baris outbox yang sudah terkirim. Target jeda outbox ke store di bawah 1 menit pada kondisi normal. Umur baris outbox tertua dipantau, dan jeda di atas 5 menit memicu peringatan keamanan `[KEPUTUSAN-22]`.

**4. Audit store terpisah** `[KEPUTUSAN-22]`:
- Database PostgreSQL tersendiri, di instans database yang terpisah dari database aplikasi di production (ADR-019). Rumah Sehat dan RME minimalis masing-masing punya database audit sendiri.
- Tabel `audit_event` dipartisi per bulan berdasarkan `recorded_at` `[SRC-39]`. Primary key-nya komposit `(event_id, recorded_at)` karena kolom partisi harus ikut di constraint unik. `event_id` tetap UUIDv7 dari aplikasi, dan keunikannya secara global dijaga lewat tabel kunci event. Bentuk ini dianggap memenuhi ADR-002 (detail di data dictionary §B) `[KEPUTUSAN-56]`.
- Role: `audit_writer` (relay, `INSERT` ditambah baca sempit atas kepala rantai, yaitu `seq` dan `event_hash` terakhir, lewat fungsi atau view khusus supaya relay dapat melanjutkan rantai setelah restart), `audit_reader` (backoffice dan RME, hanya `SELECT`), dan `audit_verifier` (job verifikasi: baca event dan tulis hasil verifikasi). Tidak ada role aplikasi yang punya `UPDATE`, `DELETE`, atau `TRUNCATE`. Role pemilik skema hanya dipakai pipeline migrasi.
- Trigger menolak `UPDATE`, `DELETE`, dan `TRUNCATE` di `audit_event` `[SRC-40]`. Percobaan yang ditolak menghasilkan peringatan keamanan dari log database (ADR-010).
- Aplikasi dan API tidak menyediakan fitur ubah atau hapus event (US-1504). Permintaan ke endpoint semacam itu ditolak dan dicatat sebagai kejadian keamanan.

**5. Deteksi manipulasi dengan hash berantai** `[KEPUTUSAN-22]`:
- Relay memberi setiap event nomor urut `seq` dan menghitung `event_hash = SHA-256(prev_hash ‖ bentuk kanonis event)`. `prev_hash` adalah `event_hash` event sebelumnya.
- Setiap jam, kepala rantai (`seq` dan `event_hash` terakhir) ditulis ke object storage dengan kunci retensi tulis-sekali (WORM) dan ke security log. Kemampuan WORM penyedia diverifikasi di Tahap 5.
- Job verifikasi harian menghitung ulang rantai dan mencocokkannya dengan kepala rantai yang tersimpan. Ketidakcocokan memicu peringatan keamanan.
- Mengubah satu event mengharuskan penyerang menulis ulang seluruh rantai sesudahnya **dan** kepala rantai di WORM, yang tidak dapat diubah sebelum masa kuncinya habis.

**6. Waktu** `[KEPUTUSAN-22]`:
- Setiap event menyimpan `occurred_at` (jam server aplikasi saat aksi terjadi) dan `recorded_at` (jam server audit store saat event tersimpan), keduanya `timestamptz` yang tersimpan sebagai UTC `[SRC-41]`. Tampilan memakai WIB (AB-42).
- Jam server disinkronkan dengan layanan waktu penyedia cloud, yang diverifikasi di Tahap 5. Jam perangkat klien tidak pernah dipakai sebagai waktu event.
- Selisih `recorded_at − occurred_at` yang tidak wajar dipantau sebagai tanda masalah jam atau antrean.

**7. Isi event** mengikuti PRD §7.2. Masking dan larangan PRD §7.3 diterapkan di pustaka, bukan oleh pemanggil `[KEPUTUSAN-22]`:
- Nilai before/after hanya untuk field di daftar izin per jenis objek. Field klinis dan dokumen hanya dicatat nama field dan versinya.
- NIK dan nomor rekening hanya 4 digit terakhir. Password, token, OTP, dan rahasia MFA tidak pernah diterima pustaka.
- Test otomatis memastikan event tidak memuat field terlarang (US-1501 AC2).
- Subjek data disimpan di kolom terindeks, dan event yang menyentuh lebih dari satu subjek memakai tabel anak. Dengan begitu rekam jejak pemrosesan per subjek dapat diambil untuk salinan data pribadi (US-1604).

**8. Akses baca** `[KEPUTUSAN-22]`:
- Auditor (baca dan export) dan Super Admin (baca) membaca lewat backoffice, yang memakai role `audit_reader`. Export dicatat sebagai event E-12 (US-1503).
- Admin Fasyankes membaca event E-05 fasyankesnya lewat RME minimalis (PRD §7.5).

**9. Arsip dan retensi** `[KEPUTUSAN-22]`:
- Partisi yang lebih tua dari jendela daring (nilai awal 24 bulan) diekspor ke object storage WORM, beserta hash berkas dan kepala rantai pada batas partisi, lalu di-*detach* dari database daring `[SRC-39]`. Arsip dapat dipulihkan untuk investigasi.
- Tidak ada pemusnahan sebelum masa retensi ditetapkan (PT-24). Setelah ditetapkan, pemusnahan hanya lewat prosedur terjadwal yang tercatat, setelah masa kunci WORM habis.

## Alternatif yang ditolak

- **Trigger database atau CDC sebagai sumber event**: tidak melihat konteks permintaan dan tidak dapat mencatat akses baca. Event yang dihasilkan tidak memenuhi PRD §7.2.
- **Menulis langsung ke audit store dalam permintaan pengguna**: tanpa transaksi terdistribusi, aksi bisnis dan event bisa tidak konsisten. Gangguan audit store juga langsung menghentikan semua aksi sensitif.
- **Audit trail di tabel database aplikasi**: bertentangan dengan ADR-005 (terpisah dari tabel aplikasi), dan pemegang kredensial aplikasi menjadi lebih dekat ke data audit.
- **Layanan log pihak ketiga di luar Indonesia**: dilarang `[USER] T4-Q1`.
- **Hash berantai tanpa jangkar eksternal**: penyerang dengan hak tulis dapat menghitung ulang seluruh rantai.

## Konsekuensi

- Event dari aksi yang sudah di-commit tidak hilang walaupun audit store sedang gangguan; event tertahan di outbox sampai relay pulih.
- Ada jeda singkat sebelum event muncul di halaman Audit Log. Halaman menampilkan waktu event terakhir yang tersinkron.
- Relay tunggal menjadi titik serial. Throughput-nya diuji di uji beban NFR-CAP-04.
- Satu pustaka audit dipakai semua modul dan RME minimalis. Pustaka ini modul kritis (T1).
- Biaya penyimpanan WORM dan instans database audit dihitung di Tahap 5.
