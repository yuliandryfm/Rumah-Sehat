-- db/migrations/app/20261127100000_corporate_member_account_history_index.sql
-- F1-T109 (review T1 DBA butir 1 dan 3, KEPUTUSAN-386): index riwayat keanggotaan per akun HR pada corporate_member
-- (A20 K1, kelas M). Pencarian tenant audit akun HR (`corporate::repo::hr_account_tenant`, KEPUTUSAN-381) memilih
-- keanggotaan aktif lebih dulu, lalu keanggotaan terakhir yang dihapus lunak, dalam satu query
-- `WHERE account_id = $1 ORDER BY deleted_at DESC NULLS FIRST, id DESC LIMIT 1`. Index parsial
-- uqp_corporate_member_account_id (WHERE deleted_at IS NULL) tidak dapat melayani baris terhapus, sehingga langkah itu
-- memindai seluruh tabel. Index ini melayani query tersebut secara penuh (urutan index sama dengan ORDER BY) dan
-- sekaligus FK fk_corporate_member_account_id untuk semua baris (docs/05-physical-schema.md §4.1 butir 3); catatan
-- "juga melayani FK account_id" di migrasi 20261123110000 tidak akurat untuk baris terhapus, koreksinya dicatat di
-- docs/04-data-dictionary.md A20 (berkas migrasi lama tidak diubah, §11.1).
--
-- Modul kritis T1 (file migrasi skema; ADR-008): review DBA + keamanan.
-- Sumber: docs/04-data-dictionary.md A20 Rincian K1; docs/05-physical-schema.md §4.1, §11.2, §11.3.
-- Maju saja (ADR-019); koreksi lewat berkas baru (§11.1).
--
-- Ketentuan (expand §11.3, aman untuk kode versi sebelumnya):
-- - Hanya menambah index; tanpa perubahan kolom, constraint, atau hak. Tabel kecil (satu baris per keanggotaan HR),
--   sehingga CREATE INDEX di dalam transaksi dengan lock_timeout dari penjaga tidak menahan penulisan lebih dari
--   sekejap.
-- - Tidak memuat data pribadi; kolom yang diindeks hanya ID akun, waktu hapus lunak, dan ID baris.

-- Penjaga (§11.2) dari templat db/test/templat-penjaga-lanjutan-app.sql.
DO $$
BEGIN
  IF current_user <> 'rs_migration' THEN
    RAISE EXCEPTION 'migration must run as rs_migration, got % (session %)', current_user, session_user;
  END IF;
  IF EXISTS (SELECT 1 FROM app._sqlx_migrations WHERE version > 20261127100000) THEN
    RAISE EXCEPTION 'migration order: version 20261127100000 is older than an applied migration (forward-only, ADR-019)';
  END IF;
END $$;
SET LOCAL lock_timeout = '5s';
SET LOCAL statement_timeout = '15min';

CREATE INDEX ix_corporate_member_account_id_deleted_at
  ON app.corporate_member (account_id, deleted_at DESC NULLS FIRST, id DESC);
