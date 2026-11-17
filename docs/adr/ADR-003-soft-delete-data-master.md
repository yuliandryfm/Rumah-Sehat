# ADR-003: Soft delete untuk data master

Status: Diterima
Tanggal: 2026-11-10

## Konteks
Data tidak boleh hilang, dan jejak penghapusan dibutuhkan untuk audit serta investigasi fraud.

## Keputusan
Data master (user, produk, konfigurasi, dan sejenisnya) memakai soft delete dengan kolom `deleted_at`, `deleted_by`, dan `delete_reason`.

## Alternatif yang ditolak
- Hard delete: data dan jejaknya hilang permanen.
- Tabel arsip terpisah: lebih rumit dan rawan tidak sinkron.

## Konsekuensi
- Soft delete sendiri **bukan bukti fraud**, karena flag bisa dikembalikan. Setiap delete dan restore wajib tercatat di audit trail (ADR-005), dan hak `DELETE` dicabut dari user DB aplikasi.
- Unique constraint memakai **partial unique index** yang hanya berlaku untuk baris dengan `deleted_at IS NULL`.
- Semua query default **wajib memfilter data terhapus** (global scope di ORM atau view).
- Disediakan **prosedur anonimisasi PII** untuk permintaan hapus data pribadi: kolom PII ditimpa atau di-hash, catatan transaksi tetap utuh. Kewajiban hukumnya dikonfirmasi ke legal/compliance.
- Disiapkan kebijakan arsip karena tabel terus membesar.
