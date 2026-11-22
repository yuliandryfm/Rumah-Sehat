-- db/migrations/app/20261124200000_platform_reason_code_corporate_rejection.sql
-- F1-T104: konteks reason_code untuk penolakan verifikasi perusahaan (A6.7 reason_code, konteks
-- corporate_rejection). DDL saja (docs/05-physical-schema.md §11.3: DDL dan DML terpisah); isi daftar
-- alasan di migrasi data 20261124201000_corporate_rejection_reason_seed.sql. Tanpa perubahan hak role.
--
-- Modul kritis T1 (file migrasi skema, ADR-008, ADR-021 butir 4): review DBA.
-- Sumber: PRD v1.3 US-1902 AC3 (penolakan dengan kode alasan, termasuk bukti kewenangan tidak memadai);
-- docs/04-data-dictionary.md A6.7 dan A20 Rincian K1 ("konteks reason_code penolakan perusahaan belum ada di
-- A6.7 dan ditambahkan di F1-T104"); migrasi 20261123100000_corporate_client_verification.sql (komentar
-- reason_code).
--
-- Ketentuan:
-- - Expand (ADR-019 butir 2, §11.3): CHECK diganti dengan versi yang hanya menambah satu nilai, dalam satu
--   ALTER TABLE (satu kunci), sehingga kode versi sebelumnya tetap kompatibel. Pola sama dengan perluasan
--   CHECK account di 20261123110000_corporate_member_account_korporat.sql.
-- - corporate_verification_decision.reason_code tetap teks kode tanpa FK (pola
--   identity_verification_request_status_event.reason_code); keanggotaan kode di konteks ini diperiksa
--   aplikasi di transaksi keputusan.

-- Penjaga (§11.2) dari templat db/test/templat-penjaga-lanjutan-app.sql.
DO $$
BEGIN
  IF current_user <> 'rs_migration' THEN
    RAISE EXCEPTION 'migration must run as rs_migration, got % (session %)', current_user, session_user;
  END IF;
  IF EXISTS (SELECT 1 FROM app._sqlx_migrations WHERE version > 20261124200000) THEN
    RAISE EXCEPTION 'migration order: version 20261124200000 is older than an applied migration (forward-only, ADR-019)';
  END IF;
END $$;
SET LOCAL lock_timeout = '5s';
SET LOCAL statement_timeout = '15min';

-- A6.7 reason_code (expand): konteks corporate_rejection (US-1902 AC3). Hak tidak berubah (F1-T18).
ALTER TABLE app.reason_code
  DROP CONSTRAINT ck_reason_code_context,
  ADD CONSTRAINT ck_reason_code_context CHECK (context IN
    ('identity_rejection','health_worker_rejection','bank_account_rejection','order_cancellation',
     'claim_rejection','dsr_rejection','status_transition','corporate_rejection'));
