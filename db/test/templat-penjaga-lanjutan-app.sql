-- db/test/templat-penjaga-lanjutan-app.sql: templat penjaga berkas migrasi LANJUTAN db/migrations/app
-- (semua berkas selain berkas pertama). Salin ke awal berkas baru dan ganti @VERSI@ dengan versi
-- nama berkas (14 digit). Dipakai juga scripts/db-migrations-check.sh sebagai fixture test.
-- Dasar: docs/05-physical-schema.md §11.2; review DBA F1-T14/T15 butir 3b (penjaga urutan).
DO $$
BEGIN
  IF current_user <> 'rs_migration' THEN
    RAISE EXCEPTION 'migration must run as rs_migration, got % (session %)', current_user, session_user;
  END IF;
  IF EXISTS (SELECT 1 FROM app._sqlx_migrations WHERE version > @VERSI@) THEN
    RAISE EXCEPTION 'migration order: version @VERSI@ is older than an applied migration (forward-only, ADR-019)';
  END IF;
END $$;
SET LOCAL lock_timeout = '5s';
SET LOCAL statement_timeout = '15min';
