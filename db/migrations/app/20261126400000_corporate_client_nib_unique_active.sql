-- db/migrations/app/20261126400000_corporate_client_nib_unique_active.sql
-- CR-035 (keputusan user Q10 = B2, 2026-11-26), langkah EXPAND: NIB unik hanya di antara perusahaan Aktif yang
-- belum dihapus lunak. Pengajuan yang Menunggu verifikasi atau Ditolak boleh memakai NIB yang sudah dipakai
-- perusahaan lain; pengajuan itu ditandai di antrean Verifikator, dan persetujuan ditolak bila NIB sudah dipakai
-- perusahaan Aktif lain (4090116). Menutup oracle NIB TD-45 dan bagian penguncian NIB TD-46.
--
-- Modul kritis T1 (file migrasi skema; ADR-008, ADR-019): review DBA + keamanan.
-- Sumber: CR-035 docs/backlog.md (mengubah BR-136 "NIB yang sudah terdaftar ditolak", PRD US-1901 AC7, US-1903
-- AC4); docs/05-physical-schema.md §4.1, §11.2, §11.3; migrasi 20261123100000_corporate_client_verification.sql
-- (uqp_corporate_client_nib lama, status apa pun).
-- Maju saja (ADR-019); koreksi lewat berkas baru (§11.1).
--
-- Ketentuan (expand §11.3, aman untuk kode versi sebelumnya):
-- - Hanya MENAMBAH dua index; uqp_corporate_client_nib lama (status apa pun) tetap ada sampai migrasi contract
--   20261126410000, sehingga kode versi sebelumnya (ON CONFLICT (nib) WHERE deleted_at IS NULL) tetap berjalan.
--   Data yang ada sudah memenuhi index baru karena index lama lebih ketat.
-- - uqp_corporate_client_nib_active: penjaga terakhir "satu perusahaan Aktif per NIB", termasuk dua persetujuan
--   bersamaan atas dua pengajuan dengan NIB sama (UPDATE status kedua gagal 23505 dan dijawab 4090116).
-- - ix_corporate_client_nib: melayani pencarian NIB lintas status (penanda "NIB sudah dipakai perusahaan lain"
--   di antrean dan detail Verifikator) setelah index lama dilepas.
-- - Tabel corporate_client masih kecil (belum go-live), sehingga CREATE INDEX di dalam transaksi dengan
--   lock_timeout dari penjaga cukup (§11.3: CONCURRENTLY baru wajib setelah go-live).
-- - Tanpa perubahan kolom, constraint, data, atau hak role.

-- Penjaga (§11.2) dari templat db/test/templat-penjaga-lanjutan-app.sql.
DO $$
BEGIN
  IF current_user <> 'rs_migration' THEN
    RAISE EXCEPTION 'migration must run as rs_migration, got % (session %)', current_user, session_user;
  END IF;
  IF EXISTS (SELECT 1 FROM app._sqlx_migrations WHERE version > 20261126400000) THEN
    RAISE EXCEPTION 'migration order: version 20261126400000 is older than an applied migration (forward-only, ADR-019)';
  END IF;
END $$;
SET LOCAL lock_timeout = '5s';
SET LOCAL statement_timeout = '15min';

CREATE UNIQUE INDEX uqp_corporate_client_nib_active
  ON app.corporate_client (nib) WHERE deleted_at IS NULL AND status = 'active';   -- CR-035 (Q10 = B2)
CREATE INDEX ix_corporate_client_nib
  ON app.corporate_client (nib) WHERE deleted_at IS NULL;                          -- penanda NIB dipakai, CR-035
