-- db/migrations/app/20261126410000_corporate_client_nib_unique_any_status_drop.sql
-- CR-035 (keputusan user Q10 = B2, 2026-11-26), langkah CONTRACT: melepas uqp_corporate_client_nib lama (NIB unik
-- di antara perusahaan belum terhapus berstatus apa pun). Penggantinya uqp_corporate_client_nib_active (hanya
-- perusahaan Aktif) dan ix_corporate_client_nib dari migrasi expand 20261126400000.
--
-- Modul kritis T1 (file migrasi skema; ADR-008, ADR-019): review DBA + keamanan.
-- Sumber: CR-035 docs/backlog.md; docs/05-physical-schema.md §11.2, §11.3; KEPUTUSAN-362.
-- Maju saja (ADR-019); koreksi lewat berkas baru (§11.1).
--
-- Ketentuan:
-- - Contract (§11.3): kode versi sebelumnya memakai ON CONFLICT (nib) WHERE deleted_at IS NULL yang membutuhkan
--   index ini, jadi berkas ini hanya boleh diterapkan setelah tidak ada instans versi sebelumnya yang berjalan.
--   Sebelum go-live (tanpa deploy bergulir di production) expand dan contract diterapkan bersama dalam rilis yang
--   sama (KEPUTUSAN-362); setelah go-live pola ini dipisah ke rilis berikutnya.
-- - Tanpa perubahan kolom, data, atau hak role.

-- Penjaga (§11.2) dari templat db/test/templat-penjaga-lanjutan-app.sql.
DO $$
BEGIN
  IF current_user <> 'rs_migration' THEN
    RAISE EXCEPTION 'migration must run as rs_migration, got % (session %)', current_user, session_user;
  END IF;
  IF EXISTS (SELECT 1 FROM app._sqlx_migrations WHERE version > 20261126410000) THEN
    RAISE EXCEPTION 'migration order: version 20261126410000 is older than an applied migration (forward-only, ADR-019)';
  END IF;
END $$;
SET LOCAL lock_timeout = '5s';
SET LOCAL statement_timeout = '15min';

DROP INDEX app.uqp_corporate_client_nib;
