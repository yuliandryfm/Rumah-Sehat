-- db/migrations/app/20261126350000_platform_upload_slot_owner_index.sql
-- F1-T102: index kuota unggah per akun pada upload_slot (A19.5, kelas O). Kuota "paling banyak N slot per akun per
-- bucket per jam" (dokumen perusahaan HR, US-1901 AC2) menghitung baris upload_slot per owner_account_id,
-- bucket, dan created_at. Tanpa index ini query itu memindai seluruh tabel; index ini juga melayani FK
-- fk_upload_slot_owner_account_id (docs/05-physical-schema.md §4.1 butir 3: FK tanpa index pelayan dibuatkan index
-- bila dipakai query).
--
-- Modul kritis T1 (file migrasi skema; ADR-008, ADR-018): review DBA + keamanan.
-- Sumber: docs/04-data-dictionary.md A19.5; docs/05-physical-schema.md §4.1, §11.2, §11.3.
-- Maju saja (ADR-019); koreksi lewat berkas baru (§11.1).
--
-- Ketentuan (expand §11.3, aman untuk kode versi sebelumnya):
-- - Hanya menambah index; tanpa perubahan kolom, constraint, atau hak. Tabel kelas O berumur pendek (baris hidup
--   paling lama satu jam ditambah jeda pembersihan, TD-17), sehingga CREATE INDEX di dalam transaksi dengan
--   lock_timeout dari penjaga tidak menahan penulisan slot lebih dari sekejap.
-- - Tidak memuat data pribadi; kolom yang diindeks hanya ID akun, nama bucket logis, dan waktu.

-- Penjaga (§11.2) dari templat db/test/templat-penjaga-lanjutan-app.sql.
DO $$
BEGIN
  IF current_user <> 'rs_migration' THEN
    RAISE EXCEPTION 'migration must run as rs_migration, got % (session %)', current_user, session_user;
  END IF;
  IF EXISTS (SELECT 1 FROM app._sqlx_migrations WHERE version > 20261126350000) THEN
    RAISE EXCEPTION 'migration order: version 20261126350000 is older than an applied migration (forward-only, ADR-019)';
  END IF;
END $$;
SET LOCAL lock_timeout = '5s';
SET LOCAL statement_timeout = '15min';

CREATE INDEX ix_upload_slot_owner_account_id_bucket_created_at
  ON app.upload_slot (owner_account_id, bucket, created_at);
