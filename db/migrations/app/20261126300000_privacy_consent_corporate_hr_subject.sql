-- db/migrations/app/20261126300000_privacy_consent_corporate_hr_subject.sql
-- F1-T101 (US-1901 AC1, AC5): perluasan sempit A3 supaya bukti persetujuan dapat dicatat untuk subjek HR
-- perusahaan (akun realm korporat), bukan hanya pasien. Tiga perubahan, semuanya hanya menambah nilai atau
-- kolom nullable (expand, ADR-019 butir 2): kode lama yang menulis bukti pasien tetap sah.
--
-- Modul kritis T1 (persetujuan dan hak subjek data; file migrasi skema dan hak role; ADR-004, ADR-008,
-- ADR-013 butir 5, ADR-021 butir 4): review DBA + keamanan.
-- Sumber: docs/04-data-dictionary.md A3.1, A3.3; docs/05-physical-schema.md §4.1, §4.9, §11.2, §11.3;
-- PRD v1.3 US-1901 AC1, AC5, NFR-PRIV-05; ADR-028; KEPUTUSAN-335 (privacy::api menunggu perluasan skema),
-- KEPUTUSAN-321 (rancangan perluasan ini). Maju saja (ADR-019); koreksi lewat berkas baru (§11.1).
--
-- Perubahan:
-- 1. consent_text_version.consent_type: + 'corporate_hr_registration' (teks persetujuan HR saat mendaftarkan
--    perusahaan, PRD L-20). Isi versi awal (draf) ada di migrasi data 20261126310000.
-- 2. consent_record: patient_id NOT NULL -> NULL, kolom baru account_id (FK app.account), CHECK tepat satu
--    subjek (pasien XOR akun HR), CHECK care_recipient_id kosong untuk subjek HR, channel + 'corporate_portal'
--    dengan CHECK saling-mengunci (account_id terisi <=> channel 'corporate_portal').
-- 3. Index parsial untuk status persetujuan per akun HR, dan hak SELECT per kolom account_id bagi pekerja
--    privasi (supaya retensi/anonimisasi dapat mencapai bukti subjek HR; DELETE tetap ditolak penjaga selama
--    kategori retensi menunggu PT-24).
--
-- Ketentuan:
-- - Bukti tetap kelas L (ADR-004): tidak ada UPDATE/DELETE baru untuk rs_app; trigger penolak, jam server, dan
--   penjaga DELETE tidak berubah. ALTER TABLE tidak memicu trigger baris.
-- - Kesesuaian jenis teks dengan subjek (teks HR hanya untuk akun HR, teks pasien hanya untuk pasien) diperiksa
--   aplikasi (privacy::api); database tidak menegakkan lintas tabel (pola DD A1.2, seperti jenis akun anggota di
--   corporate_member). [TIDAK DIKETAHUI] apakah DBA meminta trigger penjaga lintas tabel.
-- - Tidak ada backfill: baris lama berisi patient_id terisi, account_id NULL, channel patient_app/backoffice,
--   sehingga semua CHECK baru terpenuhi tanpa menulis ulang baris. ADD CONSTRAINT memindai tabel sekali di bawah
--   lock_timeout 5 detik. [TIDAK DIKETAHUI] ukuran consent_record di production; bila besar, DBA memilih
--   NOT VALID + VALIDATE CONSTRAINT.

-- Penjaga (§11.2) dari templat db/test/templat-penjaga-lanjutan-app.sql.
DO $$
BEGIN
  IF current_user <> 'rs_migration' THEN
    RAISE EXCEPTION 'migration must run as rs_migration, got % (session %)', current_user, session_user;
  END IF;
  IF EXISTS (SELECT 1 FROM app._sqlx_migrations WHERE version > 20261126300000) THEN
    RAISE EXCEPTION 'migration order: version 20261126300000 is older than an applied migration (forward-only, ADR-019)';
  END IF;
END $$;
SET LOCAL lock_timeout = '5s';
SET LOCAL statement_timeout = '15min';

-- 1. A3.1: jenis teks persetujuan HR perusahaan.
ALTER TABLE app.consent_text_version
  DROP CONSTRAINT ck_consent_text_version_consent_type,
  ADD CONSTRAINT ck_consent_text_version_consent_type CHECK (consent_type IN
    ('patient_data_processing','family_adult_statement','guardian_consent','corporate_hr_registration'));

-- 2. A3.3: subjek HR (akun realm korporat) selain pasien, dalam satu ALTER TABLE (satu kunci).
ALTER TABLE app.consent_record
  ALTER COLUMN patient_id DROP NOT NULL,
  ADD COLUMN account_id uuid NULL,
  ADD CONSTRAINT fk_consent_record_account_id FOREIGN KEY (account_id) REFERENCES app.account (id),
  DROP CONSTRAINT ck_consent_record_channel,
  ADD CONSTRAINT ck_consent_record_channel CHECK (channel IN ('patient_app','backoffice','corporate_portal')),
  -- Tepat satu subjek: pasien atau akun HR.
  ADD CONSTRAINT ck_consent_record_subject CHECK ((patient_id IS NOT NULL) <> (account_id IS NOT NULL)),
  -- Subjek HR tidak punya penerima layanan.
  ADD CONSTRAINT ck_consent_record_account_no_care_recipient CHECK (account_id IS NULL OR care_recipient_id IS NULL),
  -- Kanal member area korporat hanya untuk subjek HR, dan sebaliknya.
  ADD CONSTRAINT ck_consent_record_channel_subject CHECK ((account_id IS NOT NULL) = (channel = 'corporate_portal'));

-- 3. Status persetujuan terkini per akun HR (analog index per pasien, US-1601 AC1); parsial karena hanya
--    sebagian kecil baris bersubjek akun.
CREATE INDEX ix_consent_record_account_id_consent_item_id_created_at
  ON app.consent_record (account_id, consent_item_id, created_at DESC) WHERE account_id IS NOT NULL;

GRANT SELECT (account_id) ON app.consent_record TO rs_privacy_worker;   -- FK subjek, seperti patient_id (DD §0.6 A Log)
