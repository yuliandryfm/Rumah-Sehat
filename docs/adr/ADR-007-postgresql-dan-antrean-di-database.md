# ADR-007: PostgreSQL sebagai database, dengan antrean kerja dan outbox di PostgreSQL

Status: Diusulkan
Tanggal: 2026-11-18

## Konteks

- ADR yang sudah diterima menuntut: tipe UUID native (ADR-002), partial unique index untuk data master (ADR-003), dan pencabutan hak `UPDATE`/`DELETE` per tabel dari user database aplikasi (ADR-003, ADR-004) `[DOK: adr/]`.
- Event audit harus tercatat atomik bersama aksi bisnisnya. Kalau event gagal dicatat, aksi pengguna dibatalkan `[KEPUTUSAN-15]`.
- Rebutan slot dan stok hanya boleh dimenangkan satu pihak (KU-09), dan kiriman ganda hanya menghasilkan satu entitas (KU-05) `[DOK: 03-prd.md]`.
- Banyak aturan bisnis berupa batas waktu (PB-01–PB-20) yang harus dieksekusi otomatis `[DOK: 03-prd.md]` §3.7.
- RPO ≤ 15 menit (NFR-DR-01). Tim kecil, sehingga jumlah komponen infrastruktur perlu ditekan `[DOK: 03-prd.md]` `[DOK: 00-project-brief.md]`.

## Keputusan

1. **PostgreSQL** adalah database untuk keempat penyimpanan data: database aplikasi Rumah Sehat, database audit Rumah Sehat, database aplikasi RME minimalis, dan database audit RME minimalis. Versi major stabil tertinggi saat ini adalah 18 `[SRC-30]`; versi yang dipakai ditetapkan di Tahap 5 `[KEPUTUSAN-20]`.
2. Kemampuan PostgreSQL yang menjadi dasar desain, semuanya terverifikasi di dokumentasi resmi:
   - tipe `uuid` native 128-bit `[SRC-31]`;
   - partial unique index `[SRC-33]`;
   - hak akses per tabel dan per kolom lewat `GRANT` `[SRC-35]`;
   - Row Level Security untuk isolasi per fasyankes di RME minimalis, dengan `FORCE ROW LEVEL SECURITY` karena pemilik tabel normalnya melewati RLS `[SRC-34]` (ADR-012);
   - `timestamptz` yang disimpan sebagai UTC `[SRC-41]` (ADR-017);
   - trigger yang menolak `UPDATE`/`DELETE`/`TRUNCATE` sebagai pertahanan kedua setelah pencabutan hak. Galat di trigger membatalkan statement pemicunya `[SRC-40]`;
   - partisi deklaratif untuk tabel audit `[SRC-39]`;
   - pencarian teks dengan `pg_trgm` untuk katalog, layanan, dan tenaga kesehatan `[SRC-42]`;
   - pemulihan point-in-time dari arsip WAL `[SRC-38]` (ADR-019).
3. **Antrean kerja dan outbox memakai tabel PostgreSQL**. Ini mencakup job terjadwal (batas waktu PB, pengingat, pemeriksaan SIP harian, rekonsiliasi), outbox audit (ADR-009), outbox integrasi keluar (ADR-014), dan outbox notifikasi. Job diambil pekerja dengan `SELECT ... FOR UPDATE SKIP LOCKED`, pola yang disebut dokumentasi untuk tabel mirip antrean `[SRC-36]`. Baris outbox ditulis dalam transaksi yang sama dengan perubahan bisnisnya, sehingga perubahan dan pesan keluarnya selalu sama-sama tersimpan atau sama-sama batal `[KEPUTUSAN-20]`.
4. **Sinyal real-time antar-instans** (misalnya pesan chat baru untuk proses `realtime` di instans lain) memakai `LISTEN/NOTIFY`. Notifikasi baru terkirim setelah transaksi commit dan payload-nya di bawah 8000 byte pada konfigurasi bawaan `[SRC-37]`, sehingga payload hanya berisi ID. Isi pesan dibaca dari database `[KEPUTUSAN-20]`.
5. UUIDv7 tetap dibuat di sisi aplikasi sesuai ADR-002. PostgreSQL 18 juga menyediakan fungsi `uuidv7()` `[SRC-32]`; pemakaiannya sebagai nilai bawaan kolom dibahas di Tahap 5 tanpa mengubah ADR-002.
6. **Tanpa Redis atau message broker di rilis 1.** Kecukupan pendekatan ini dibuktikan lewat uji beban NFR-CAP-04. Kalau uji beban gagal karena antrean atau sinyal real-time, komponen tambahan diusulkan lewat ADR baru `[KEPUTUSAN-20]`.

## Alternatif yang ditolak

- **Message broker terpisah (misalnya RabbitMQ atau Kafka)**: menambah komponen yang harus dioperasikan dan dipulihkan. Pesan bisa terkirim walaupun transaksi bisnis batal, atau sebaliknya, kecuali memakai pola outbox, sehingga outbox tetap dibutuhkan.
- **Redis untuk antrean dan pub/sub**: komponen tambahan dengan kebutuhan backup dan pemulihan sendiri. Kebutuhan saat ini dapat dipenuhi PostgreSQL.
- **Database dokumen (NoSQL)**: alur uang, status, dan audit membutuhkan transaksi ACID, constraint, dan hak akses per tabel.
- **Database relasional lain**: tidak dievaluasi lebih jauh, karena PostgreSQL memenuhi semua kebutuhan di atas dengan kemampuan yang terverifikasi.

## Konsekuensi

- Satu teknologi data untuk dipelajari, dioperasikan, di-backup, dan dipulihkan.
- Antrean menambah beban tulis ke database aplikasi. Panjang antrean, umur job tertua, dan jumlah job gagal dipantau (ADR-010).
- Job harus idempoten, karena job dapat dijalankan ulang setelah pekerja gagal di tengah jalan (NFR-INT-01).
- Job tunggal seperti relay audit berantai (ADR-009) harus dijalankan oleh satu pekerja pada satu waktu. Mekanisme penguncinya ditetapkan di Tahap 5.
