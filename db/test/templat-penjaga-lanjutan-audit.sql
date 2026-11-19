-- db/test/templat-penjaga-lanjutan-audit.sql: templat penjaga berkas migrasi LANJUTAN
-- db/migrations/audit (kedua database audit). Ganti @VERSI@ dengan versi nama berkas (14 digit).
-- Bila berkas util audit belum diterapkan, util.fn_audit_role tidak ada dan penjaga gagal.
-- Dipakai juga scripts/db-migrations-check.sh sebagai fixture test.
-- Dasar: docs/05-physical-schema.md §11.1, §11.2; review DBA F1-T14/T15 butir 3b.
DO $$
BEGIN
  IF current_user <> util.fn_audit_role('audit_migration') THEN
    RAISE EXCEPTION 'migration must run as %, got % (session %)', util.fn_audit_role('audit_migration'), current_user, session_user;
  END IF;
  IF EXISTS (SELECT 1 FROM audit._sqlx_migrations WHERE version > @VERSI@) THEN
    RAISE EXCEPTION 'migration order: version @VERSI@ is older than an applied migration (forward-only, ADR-019)';
  END IF;
END $$;
SET LOCAL lock_timeout = '5s';
SET LOCAL statement_timeout = '15min';
