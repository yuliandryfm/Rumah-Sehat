-- db/migrations/app/20261127440000_corporate_client_correction.sql
-- CR-040 (keputusan user Q34 = A, 2026-11-27): tabel A20 corporate_client_correction (T), nilai sebelumnya dari
-- field yang dikoreksi HR saat pengajuan ulang setelah Ditolak (CR-036), supaya Verifikator melihat penanda
-- "Data dikoreksi" beserta nilai sebelumnya di panel detail (KEPUTUSAN-442).
--
-- Modul kritis T1 (file migrasi skema dan hak role database; ADR-004, ADR-008, ADR-012): review DBA + keamanan.
-- Sumber: docs/04-data-dictionary.md §0.2-§0.6 (STD-T, hak §0.6 A), A20; docs/05-physical-schema.md §1, §4.1,
-- §4.5, §11.2, §11.3; ADR-001, ADR-002, ADR-004, ADR-005; CR-036 (KEPUTUSAN-410, KEPUTUSAN-416).
-- Maju saja (ADR-019); koreksi lewat berkas baru (§11.1).
--
-- Ketentuan:
-- - Satu baris per koreksi = per putaran hasil pengajuan ulang yang membawa koreksi; ditulis hanya oleh jalur
--   koreksi (`corporate::documents::resubmit_with`) di transaksi yang sama dengan UPDATE corporate_client dan
--   E-10 `corporate_client_data_corrected`, setelah kunci baris. submission_round = putaran BARU (>= 2), sehingga
--   "berubah dibandingkan putaran sebelumnya" = baris dengan submission_round = putaran berjalan. Pengajuan ulang
--   tanpa koreksi (dokumen saja) atau koreksi tanpa field yang benar-benar berubah tidak menulis baris.
-- - Kolom previous_* NULL = field itu tidak berubah; minimal satu terisi (ck_corp_client_corr_any_field).
-- - NIB dan NPWP hanya 4 karakter terakhir (pola audit KEPUTUSAN-416 dan NIB termasking antrean wireframe C13):
--   nilai lengkap sebelumnya tidak disalin ke tabel ini (minimasi data; NPWP Terbatas, DD A20). Nama, alamat,
--   dan telepon kantor = data badan usaha, kelas sama dengan corporate_client (tanpa kolom "Anonimkan").
-- - Transaksi (ADR-004): rs_app hanya SELECT, INSERT; trigger penolak UPDATE/DELETE/TRUNCATE tingkat statement.
--   Pekerja privasi tanpa hak (tidak ada FK subjek data pribadi, pola corporate_verification_decision).
-- - id tanpa nilai bawaan (ADR-002); created_at DEFAULT now(); FK tanpa CASCADE; FK corporate_client_id dilayani
--   UQ (corporate_client_id, submission_round); FK aktor tanpa index (§4.1 butir 3-4).
-- - Batas panjang sama dengan corporate_client (coding-standards §5.1).
-- - Expand murni (§11.3): tabel baru, aman untuk kode versi sebelumnya. Koreksi sebelum migrasi ini tidak punya
--   baris (tanpa backfill: nilai sebelumnya tidak tersimpan di data bisnis, dan audit bukan sumber data bisnis).
-- - Nama > 60 byte memakai alias tabel di komentar (§1.2): corp_client_corr.

-- Penjaga (§11.2) dari templat db/test/templat-penjaga-lanjutan-app.sql.
DO $$
BEGIN
  IF current_user <> 'rs_migration' THEN
    RAISE EXCEPTION 'migration must run as rs_migration, got % (session %)', current_user, session_user;
  END IF;
  IF EXISTS (SELECT 1 FROM app._sqlx_migrations WHERE version > 20261127440000) THEN
    RAISE EXCEPTION 'migration order: version 20261127440000 is older than an applied migration (forward-only, ADR-019)';
  END IF;
END $$;
SET LOCAL lock_timeout = '5s';
SET LOCAL statement_timeout = '15min';

-- A20 corporate_client_correction (T): nilai sebelumnya field yang dikoreksi HR pada satu pengajuan ulang.
CREATE TABLE app.corporate_client_correction (
  id                      uuid        NOT NULL,
  created_at              timestamptz NOT NULL DEFAULT now(),
  created_by              uuid        NULL,
  corporate_client_id     uuid        NOT NULL,
  submission_round        integer     NOT NULL,
  previous_legal_name     text        NULL,
  previous_nib_last4      text        NULL,
  previous_npwp_last4     text        NULL,
  previous_office_address text        NULL,
  previous_office_phone   text        NULL,
  CONSTRAINT pk_corporate_client_correction PRIMARY KEY (id),
  CONSTRAINT fk_corporate_client_correction_created_by FOREIGN KEY (created_by) REFERENCES app.account (id),
  CONSTRAINT fk_corporate_client_correction_corporate_client_id
    FOREIGN KEY (corporate_client_id) REFERENCES app.corporate_client (id),
  -- Satu koreksi per putaran; juga melayani FK corporate_client_id dan pembacaan detail Verifikator.
  CONSTRAINT uq_corp_client_corr_corporate_client_id_submission_round UNIQUE (corporate_client_id, submission_round),
  -- Koreksi hanya ada pada pengajuan ulang (putaran 1 = pengajuan pertama).
  CONSTRAINT ck_corp_client_corr_submission_round CHECK (submission_round >= 2),
  CONSTRAINT ck_corp_client_corr_any_field CHECK (
    num_nonnulls(previous_legal_name, previous_nib_last4, previous_npwp_last4,
                 previous_office_address, previous_office_phone) >= 1),
  CONSTRAINT ck_corp_client_corr_prev_legal_name_length
    CHECK (previous_legal_name IS NULL OR char_length(btrim(previous_legal_name)) BETWEEN 1 AND 200),
  CONSTRAINT ck_corp_client_corr_prev_nib_last4_length
    CHECK (previous_nib_last4 IS NULL OR char_length(previous_nib_last4) BETWEEN 1 AND 4),
  CONSTRAINT ck_corp_client_corr_prev_npwp_last4_length
    CHECK (previous_npwp_last4 IS NULL OR char_length(previous_npwp_last4) BETWEEN 1 AND 4),
  CONSTRAINT ck_corp_client_corr_prev_office_address_length
    CHECK (previous_office_address IS NULL OR char_length(btrim(previous_office_address)) BETWEEN 1 AND 500),
  CONSTRAINT ck_corp_client_corr_prev_office_phone_length
    CHECK (previous_office_phone IS NULL OR char_length(btrim(previous_office_phone)) BETWEEN 1 AND 32)
);
COMMENT ON TABLE app.corporate_client_correction IS 'class:T module:korporat alias:corp_client_corr';

REVOKE ALL ON app.corporate_client_correction FROM PUBLIC;
GRANT SELECT, INSERT ON app.corporate_client_correction TO rs_app;

CREATE TRIGGER tg_corporate_client_correction_reject_ud
  BEFORE UPDATE OR DELETE ON app.corporate_client_correction
  FOR EACH STATEMENT EXECUTE FUNCTION util.fn_reject_modification();
CREATE TRIGGER tg_corporate_client_correction_reject_truncate
  BEFORE TRUNCATE ON app.corporate_client_correction
  FOR EACH STATEMENT EXECUTE FUNCTION util.fn_reject_modification();   -- TRUNCATE hanya tingkat statement [SRC-40]
