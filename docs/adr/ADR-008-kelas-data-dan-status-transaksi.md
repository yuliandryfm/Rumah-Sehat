# ADR-008: Lima kelas tabel dan pola status untuk entitas transaksi

Status: Diusulkan
Tanggal: 2026-11-18

## Konteks

- ADR-003 menetapkan soft delete untuk data master. ADR-004 menetapkan data transaksi tidak pernah di-update atau di-delete, dan hak `UPDATE`/`DELETE` dicabut dari user database aplikasi `[DOK: adr/]`.
- Banyak entitas yang diklasifikasikan transaksi punya status yang berubah: pesanan, refund, klaim, kewajiban pengembalian rekanan, tiket keluhan, dan permintaan hak subjek data `[DOK: 03-prd.md]` §3.6, §8. Kalau status disimpan sebagai kolom yang di-update, ADR-004 dilanggar.
- Bila dua kejadian datang hampir bersamaan, transisi yang tersimpan lebih dulu berlaku, dan kejadian berikutnya yang tidak sah dari status baru ditolak serta tercatat (AB-36) `[DOK: 03-prd.md]`.
- Ada data teknis yang bukan master dan bukan transaksi, misalnya sesi login, antrean job, outbox, kunci idempotensi, penahanan slot, dan draf catatan klinis yang belum difinalkan. ADR-003 dan ADR-004 tidak mengaturnya.
- Klasifikasi awal entitas di PRD §8 difinalkan di Tahap 4 `[KEPUTUSAN-10]`. Pola yang belum punya standar wajib diusulkan sebagai ADR `[DOK: CLAUDE.md]`.

## Keputusan

Setiap tabel masuk tepat satu dari lima kelas berikut. Kelasnya dicatat di data dictionary (`docs/04-data-dictionary.md`) `[KEPUTUSAN-21]`.

| Kelas | Isi | Hak role aplikasi | Kolom standar |
| --- | --- | --- | --- |
| **Master** (ADR-003) | Data acuan yang boleh berubah: akun, pasien, rekanan, tenaga kesehatan, pengaturan, salinan katalog rekanan | `SELECT`, `INSERT`, `UPDATE`. Tanpa `DELETE` | `id`, `created_at`, `created_by`, `updated_at`, `updated_by`, `deleted_at`, `deleted_by`, `delete_reason` |
| **Transaksi** (ADR-004) | Bukti kejadian bisnis: pesanan, harga terkunci, pembayaran, jurnal, refund, pesan chat, versi catatan, resep, hasil lab | `SELECT`, `INSERT` saja | `id`, `created_at`, `created_by` |
| **Riwayat status** (bagian dari transaksi) | Setiap transisi status entitas transaksi | `SELECT`, `INSERT` saja | `id`, `<entitas>_id`, `version`, `from_status`, `to_status`, `reason_code`, `actor_*`, `created_at` |
| **Proyeksi** | Status terkini entitas transaksi, satu baris per entitas, diturunkan dari riwayat status | `SELECT`, `INSERT`, `UPDATE`. Tanpa `DELETE` | `<entitas>_id` (PK), `status`, `version`, `updated_at` |
| **Operasional** | Data teknis berumur pendek yang bukan bukti bisnis: sesi, antrean job, outbox, kunci idempotensi, penahanan slot, draf, kursor sinkronisasi | `SELECT`, `INSERT`, `UPDATE`, `DELETE` | `id`, `created_at`, `expires_at` bila ada |
| **Log** | Catatan kejadian teknis atau kepatuhan yang hanya ditambah: bukti persetujuan, notifikasi, galat integrasi, percobaan pengiriman SATUSEHAT, notifikasi payment gateway | `SELECT`, `INSERT` saja. Penghapusan hanya oleh role pekerja privasi setelah masa retensi habis (ADR-013) | `id`, `created_at` |

Event audit punya penyimpanan sendiri di database terpisah (ADR-009) dan tidak termasuk tabel aplikasi.

**Sub-kelas Master snapshot**: snapshot data pribadi yang menjadi bukti data yang dikirim ke pihak luar (misalnya data penerima layanan yang dikirim ke rekanan). Snapshot tetap kelas Master karena harus dapat dianonimkan (ADR-013). Role aplikasi hanya punya `SELECT` dan `INSERT`, dan `UPDATE` hanya untuk role pekerja privasi pada kolom yang dianonimkan `[KEPUTUSAN-69]`.

**Aturan status entitas transaksi:**

1. Setiap entitas transaksi berstatus memiliki tabel riwayat status `<entitas>_status_event` dan tabel proyeksi `<entitas>_state`.
2. Transisi ditulis dalam satu transaksi database: (a) baca `version` terkini dari proyeksi, (b) validasi transisi terhadap tabel transisi PRD §3.6, (c) `INSERT` event dengan `version + 1`, (d) `UPDATE` proyeksi dengan syarat `version` lama, (e) tulis outbox audit E-09 (ADR-009).
3. Constraint unik `(<entitas>_id, version)` di tabel riwayat menjamin hanya satu transisi yang menang. Transisi kedua yang membaca versi yang sama gagal di constraint, lalu dievaluasi ulang dari status baru. Kalau tidak sah, transisi ditolak dan tercatat dengan hasil gagal, sesuai AB-36 `[KEPUTUSAN-21]`.
4. Proyeksi tidak pernah menjadi sumber kebenaran. Job harian membandingkan setiap proyeksi dengan event terakhirnya. Selisih memicu peringatan keamanan (ADR-010), dan proyeksi dibangun ulang dari riwayat.
5. Data operasional tidak boleh menjadi satu-satunya tempat data bisnis. Contohnya, draf catatan konsultasi baru menjadi bukti setelah difinalkan sebagai versi transaksi. Kejadian bermakna bisnis di data operasional, misalnya slot yang dilepas, tetap menghasilkan event audit atau transaksi.

**Konvensi lain untuk semua tabel:**

- Nama tabel dan kolom snake_case, tunggal (misalnya `patient_order`, `patient_order_status_event`), dan tidak memakai kata kunci SQL (ADR-001, KEPUTUSAN-38).
- Primary key `id uuid` UUIDv7 dari aplikasi (ADR-002). Tabel proyeksi memakai ID entitasnya sebagai primary key.
- Kolom `created_by`, `updated_by`, dan `deleted_by` berisi ID akun, atau `NULL` bila pelakunya sistem. Jenis pelakunya tercatat di event audit.
- Unique constraint di tabel master memakai partial unique index `WHERE deleted_at IS NULL` (ADR-003) `[SRC-33]`.
- Hak `UPDATE`, `DELETE`, dan `TRUNCATE` di tabel transaksi, riwayat status, dan log tidak diberikan ke role aplikasi. Trigger penolak dipasang sebagai pertahanan kedua `[SRC-40]`.
- Anonimisasi dan pemusnahan data pribadi di tabel transaksi tidak dilakukan dengan `UPDATE` atau `DELETE` baris, tetapi lewat penghapusan kunci enkripsi (*crypto-shredding*) dan anonimisasi data master (ADR-013).

## Alternatif yang ditolak

- **Kolom status di tabel transaksi dengan hak `UPDATE` per kolom** `[SRC-35]`: melanggar bunyi ADR-004 ("tidak pernah di-update") dan menyamarkan batas antara bukti dan status turunan.
- **Tanpa tabel proyeksi** (status terkini dihitung dari event terakhir di setiap query): sah, tetapi setiap daftar pesanan memerlukan subquery event terakhir. Job batas waktu juga harus memindai riwayat. Proyeksi yang dapat dibangun ulang lebih sederhana untuk dibaca.
- **Event sourcing penuh** (semua entitas direkonstruksi dari event): terlalu besar untuk tim dan jadwal.
- **Tabel riwayat status generik untuk semua entitas**: kehilangan foreign key ke entitas induknya.

## Konsekuensi

- Setiap entitas transaksi berstatus memerlukan tiga tabel (induk, riwayat status, proyeksi) dan satu jalur tulis standar. Pola ini dibuat sekali sebagai pustaka di reference implementation Tahap 8.
- Riwayat status lengkap tersedia untuk audit dan investigasi tanpa membaca audit trail.
- Kelas Proyeksi dan Operasional adalah kelas baru di luar ADR-003 dan ADR-004. Keduanya tidak melonggarkan kedua ADR itu: data bukti tetap immutable, dan data master tetap memakai soft delete.
- DBA reviewer memeriksa bahwa setiap tabel punya kelas dan hak yang sesuai (Tahap 4 dan Tahap 5).
