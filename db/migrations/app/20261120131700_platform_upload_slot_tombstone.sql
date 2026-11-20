-- db/migrations/app/20261120131700_platform_upload_slot_tombstone.sql
-- TD-17 (review F1-T29, data KTP): upload_slot sebagai tombstone sampai expires_at + jeda, dan klaim
-- objek hasil kode ulang, supaya pembersih dan rekonsiliasi dapat menghapus objek mentah yang diunggah
-- ulang lewat URL yang masih berlaku dan hasil kode ulang yang tidak pernah dicatat (ADR-018 butir 2).
--
-- Modul kritis T1 (ADR-008, ADR-018): migrasi skema; review DBA + keamanan.
-- Sumber: docs/04-data-dictionary.md A19.5; docs/05-physical-schema.md §11.2, §11.3; ADR-019.
-- Maju saja (ADR-019); koreksi lewat berkas baru (§11.1).
--
-- Ketentuan (expand §11.3, aman untuk kode versi sebelumnya):
-- - Kolom baru NULL tanpa nilai bawaan: hanya perubahan katalog, tanpa menulis ulang tabel. Kode lama tidak
--   membaca kolom ini dan tetap menghapus slot saat dipakai; kode baru menandai consumed_at.
-- - Constraint baru hanya atas kolom baru (semua NULL saat migrasi), sehingga tidak ada baris lama yang
--   melanggar dan validasinya instan. Tabel kelas O berumur pendek (slot <= 1 jam + jeda), jadi index UNIQUE
--   dibangun di dalam transaksi dengan lock_timeout dari penjaga.
-- - Hak tabel tidak berubah: rs_app S/I/U/D atas upload_slot (kelas O, DD §0.6 A) sudah mencakup kolom baru.

-- Penjaga (§11.2) dari templat db/test/templat-penjaga-lanjutan-app.sql.
DO $$
BEGIN
  IF current_user <> 'rs_migration' THEN
    RAISE EXCEPTION 'migration must run as rs_migration, got % (session %)', current_user, session_user;
  END IF;
  IF EXISTS (SELECT 1 FROM app._sqlx_migrations WHERE version > 20261120131700) THEN
    RAISE EXCEPTION 'migration order: version 20261120131700 is older than an applied migration (forward-only, ADR-019)';
  END IF;
END $$;
SET LOCAL lock_timeout = '5s';
SET LOCAL statement_timeout = '15min';

-- consumed_at: slot sudah dipakai (stored_file dicatat) atau berkasnya ditolak; baris tetap ada sebagai
-- tombstone sampai pembersih menghapusnya setelah expires_at + jeda.
-- sanitized_object_key: kunci objek hasil kode ulang yang diklaim slot sebelum objeknya ditulis.
ALTER TABLE app.upload_slot
  ADD COLUMN consumed_at          timestamptz NULL,
  ADD COLUMN sanitized_object_key text        NULL;

ALTER TABLE app.upload_slot
  ADD CONSTRAINT uq_upload_slot_sanitized_object_key UNIQUE (sanitized_object_key),
  ADD CONSTRAINT ck_upload_slot_sanitized_object_key CHECK (sanitized_object_key <> object_key);
