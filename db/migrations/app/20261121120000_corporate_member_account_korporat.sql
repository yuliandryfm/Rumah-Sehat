-- db/migrations/app/20261121120000_corporate_member_account_korporat.sql
-- F1-T99: tabel A20 K1 bagian 2, yaitu corporate_member (M, satu keanggotaan aktif per akun), dan perluasan
-- (expand) A1.1 account untuk akun HR perusahaan: account_type corporate_admin, idp_realm korporat, beserta hak role.
--
-- Modul kritis T1 (file migrasi skema dan hak role database; isolasi data per perusahaan; ADR-003, ADR-008,
-- ADR-012, ADR-019, ADR-021 butir 4): review DBA + keamanan.
-- Sumber: docs/04-data-dictionary.md v1.2 §0.2-§0.6, A1.1, A20 (tabel corporate_member dan usulan perubahan A1.1
-- "account_type + corporate_admin; idp_realm + korporat; CHECK pasangan corporate_admin -> korporat (F1-T99)");
-- docs/05-physical-schema.md §1, §4.1, §4.3, §11.2, §11.3; ADR-027 butir 3 (realm keenam korporat, pemetaan akun);
-- ADR-028 butir 1 (satu keanggotaan aktif, UQ parsial di corporate_member, M-51); PRD v1.3 US-1901 AC1,
-- US-1904 AC2-AC5. Maju saja (ADR-019); koreksi lewat berkas baru (§11.1).
--
-- Ketentuan:
-- - id tanpa nilai bawaan (ADR-002); created_at/updated_at DEFAULT now() (KEPUTUSAN-93); FK tanpa CASCADE; FK aktor
--   tidak diindeks (§4.1 butir 3-4).
-- - Hak hanya ke role grup (§4.1 butir 7); TRUNCATE tidak diberikan; Master tanpa DELETE (ADR-003). Pencabutan atau
--   perpindahan keanggotaan = hapus lunak baris lama (lalu sisip baris baru), dan aplikasi mencabut sesi akun (M-51).
-- - Satu keanggotaan aktif per akun: uqp_corporate_member_account_id (WHERE deleted_at IS NULL) [SRC-33].
-- - Jenis akun anggota (corporate_admin) diperiksa aplikasi, seperti rs_admin_role.account_id (DD A1.2); database
--   tidak menegakkan jenis akun lintas tabel.
-- - job_title (jabatan HR, US-1901 AC1) boleh kosong untuk HR yang diundang (US-1903 AC1, US-1904 AC2). Kolom ini
--   ditandai "anonymize" beserta anonymized_at: DD A20 menulis "PII lewat akun" tanpa kolom "Anonimkan", tetapi
--   jabatan bersama perusahaan dapat mengidentifikasi orang setelah akunnya dianonimkan; aturan umum DD §0.6 A
--   (U per kolom pekerja privasi) diterapkan (preseden F1-T18; dilaporkan untuk pembaruan DD).
-- - Perluasan account (expand §11.3, ADR-019 butir 2): tiga CHECK diganti dengan versi yang hanya menambah nilai,
--   dalam satu ALTER TABLE (satu kunci), sehingga kode versi sebelumnya tetap kompatibel. ck_account_full_name tidak
--   diubah: nama HR tidak selalu diketahui saat undangan dibuat (US-1903 AC1 hanya email HR pertama).
-- - idp_event_cursor/idp_event_receipt (F1-T34) tidak diperluas di sini; realm korporat untuk pengambil event
--   Keycloak (ADR-027 konsekuensi, ADR-011 butir 10) ditambahkan task realm/pengambil event korporat.

-- Penjaga (§11.2) dari templat db/test/templat-penjaga-lanjutan-app.sql.
DO $$
BEGIN
  IF current_user <> 'rs_migration' THEN
    RAISE EXCEPTION 'migration must run as rs_migration, got % (session %)', current_user, session_user;
  END IF;
  IF EXISTS (SELECT 1 FROM app._sqlx_migrations WHERE version > 20261121120000) THEN
    RAISE EXCEPTION 'migration order: version 20261121120000 is older than an applied migration (forward-only, ADR-019)';
  END IF;
END $$;
SET LOCAL lock_timeout = '5s';
SET LOCAL statement_timeout = '15min';

-- A1.1 account (expand): jenis akun corporate_admin di realm korporat (ADR-027 butir 3).
ALTER TABLE app.account
  DROP CONSTRAINT ck_account_account_type_idp_realm,
  DROP CONSTRAINT ck_account_idp_realm,
  DROP CONSTRAINT ck_account_account_type,
  ADD CONSTRAINT ck_account_account_type CHECK (account_type IN
    ('patient','health_worker','rs_admin','partner_admin','corporate_admin')),
  ADD CONSTRAINT ck_account_idp_realm CHECK (idp_realm IN ('pasien','nakes','admin','korporat')),
  ADD CONSTRAINT ck_account_account_type_idp_realm CHECK (
    (account_type = 'patient' AND idp_realm = 'pasien')
    OR (account_type = 'health_worker' AND idp_realm = 'nakes')
    OR (account_type IN ('rs_admin','partner_admin') AND idp_realm = 'admin')
    OR (account_type = 'corporate_admin' AND idp_realm = 'korporat'));
-- Hak account tidak berubah (F1-T16): rs_app S, I, U; pekerja privasi S + U per kolom anonymize.

-- A20 corporate_member (M): keanggotaan akun HR di perusahaan (tenant, ADR-028 butir 1).
CREATE TABLE app.corporate_member (
  id                  uuid        NOT NULL,
  created_at          timestamptz NOT NULL DEFAULT now(),
  created_by          uuid        NULL,
  updated_at          timestamptz NOT NULL DEFAULT now(),
  updated_by          uuid        NULL,
  deleted_at          timestamptz NULL,
  deleted_by          uuid        NULL,
  delete_reason       text        NULL,
  account_id          uuid        NOT NULL,
  corporate_client_id uuid        NOT NULL,
  job_title           text        NULL,
  anonymized_at       timestamptz NULL,
  CONSTRAINT pk_corporate_member PRIMARY KEY (id),
  CONSTRAINT fk_corporate_member_created_by FOREIGN KEY (created_by) REFERENCES app.account (id),
  CONSTRAINT fk_corporate_member_updated_by FOREIGN KEY (updated_by) REFERENCES app.account (id),
  CONSTRAINT fk_corporate_member_deleted_by FOREIGN KEY (deleted_by) REFERENCES app.account (id),
  CONSTRAINT fk_corporate_member_account_id FOREIGN KEY (account_id) REFERENCES app.account (id),
  CONSTRAINT fk_corporate_member_corporate_client_id FOREIGN KEY (corporate_client_id) REFERENCES app.corporate_client (id),
  CONSTRAINT ck_corporate_member_soft_delete CHECK (
    (deleted_at IS NULL AND deleted_by IS NULL AND delete_reason IS NULL)
    OR (deleted_at IS NOT NULL AND delete_reason IS NOT NULL)),
  CONSTRAINT ck_corporate_member_job_title_length CHECK (job_title IS NULL OR char_length(btrim(job_title)) BETWEEN 1 AND 100),
  CONSTRAINT ck_corporate_member_reason_length CHECK (delete_reason IS NULL OR char_length(delete_reason) <= 500)
);
-- ADR-028 butir 1, M-51: satu keanggotaan aktif per akun; juga melayani FK account_id.
CREATE UNIQUE INDEX uqp_corporate_member_account_id
  ON app.corporate_member (account_id) WHERE deleted_at IS NULL;   -- [SRC-33]
-- Daftar anggota HR aktif per perusahaan (US-1904 AC2, termasuk pemeriksaan HR aktif terakhir).
CREATE INDEX ix_corporate_member_corporate_client_id
  ON app.corporate_member (corporate_client_id) WHERE deleted_at IS NULL;
COMMENT ON TABLE app.corporate_member IS 'class:M module:korporat';
COMMENT ON COLUMN app.corporate_member.job_title IS 'anonymize';

REVOKE ALL ON app.corporate_member FROM PUBLIC;
GRANT SELECT, INSERT, UPDATE ON app.corporate_member TO rs_app;    -- tanpa DELETE [DOK: ADR-003]
GRANT SELECT ON app.corporate_member TO rs_privacy_worker;
GRANT UPDATE (job_title, anonymized_at, updated_at, updated_by) ON app.corporate_member TO rs_privacy_worker;
