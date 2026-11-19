-- db/migrations/app/20261119110000_platform_outbox_job_file_key.sql
-- F1-T17: tabel A19 platform (audit_outbox, job, upload_slot, stored_file, data_key) dan tabel acuan
-- retensi DEK sekali-isi data_key_retention_anchor (CR-014 bagian DB aplikasi) + hak role.
--
-- Modul kritis T1 (ADR-008, ADR-009, ADR-013, ADR-018, ADR-021 butir 4): review DBA + keamanan.
-- Sumber: docs/04-data-dictionary.md §0.4 (STD-M, STD-O), §0.6 A, §0.7, A19.1, A19.4-A19.7, A19.10;
-- docs/05-physical-schema.md §1, §4.1, §4.3, §4.8 (OH, autovacuum tabel O), §4.10 (MK), §4.11,
-- §4.12 (CR-014, KEPUTUSAN-155), §9.1, §9.3 (index usulan KEPUTUSAN-109), §11.2; CR-020 (search_path
-- fungsi persis pg_catalog, pg_temp). Maju saja (ADR-019); koreksi lewat berkas baru (§11.1).
--
-- Ketentuan:
-- - id tanpa nilai bawaan (ADR-002); created_at DEFAULT now() (KEPUTUSAN-93); FK tanpa CASCADE.
-- - Hak hanya ke role grup (§4.1 butir 7); TRUNCATE tidak diberikan.
-- - upload_slot.owner_partner_id (DD A19.5, FK→partner) BELUM dibuat: tabel partner (A5) tidak ada
--   di Fase 1. Kolom ditambahkan (NULL, expand §11.3) oleh migrasi yang membuat partner.
-- - Fungsi trigger app.* SECURITY INVOKER, search_path tetap, EXECUTE hanya pemilik (trigger tidak
--   memeriksa EXECUTE pelaku, SRC-398).

-- Penjaga (§11.2) dari templat db/test/templat-penjaga-lanjutan-app.sql.
DO $$
BEGIN
  IF current_user <> 'rs_migration' THEN
    RAISE EXCEPTION 'migration must run as rs_migration, got % (session %)', current_user, session_user;
  END IF;
  IF EXISTS (SELECT 1 FROM app._sqlx_migrations WHERE version > 20261119110000) THEN
    RAISE EXCEPTION 'migration order: version 20261119110000 is older than an applied migration (forward-only, ADR-019)';
  END IF;
END $$;
SET LOCAL lock_timeout = '5s';
SET LOCAL statement_timeout = '15min';

-- A19.1 audit_outbox (OH): rs_app dan rs_privacy_worker INSERT saja; relay SELECT, DELETE (ADR-009
-- butir 2-3, §4.8). Kode tidak boleh memakai INSERT ... RETURNING (butuh SELECT).
CREATE TABLE app.audit_outbox (
  id          uuid        NOT NULL,
  created_at  timestamptz NOT NULL DEFAULT now(),
  occurred_at timestamptz NOT NULL,
  payload     jsonb       NOT NULL,
  CONSTRAINT pk_audit_outbox PRIMARY KEY (id)
);
-- §9.3 (KEPUTUSAN-109): urutan baca relay deterministik, menggantikan ix (created_at) DD A19.1.
CREATE INDEX ix_audit_outbox_created_at_id ON app.audit_outbox (created_at, id);
ALTER TABLE app.audit_outbox SET (autovacuum_vacuum_scale_factor = 0.02, autovacuum_analyze_scale_factor = 0.02);
COMMENT ON TABLE app.audit_outbox IS 'class:OH module:platform';

REVOKE ALL ON app.audit_outbox FROM PUBLIC;
GRANT INSERT ON app.audit_outbox TO rs_app, rs_privacy_worker;
GRANT SELECT, DELETE ON app.audit_outbox TO rs_audit_relay;

-- A19.4 job (O): antrean dengan sewa (§9.1, KEPUTUSAN-108). Payload hanya ID, tanpa data pribadi.
CREATE TABLE app.job (
  id            uuid        NOT NULL,
  created_at    timestamptz NOT NULL DEFAULT now(),
  job_type      text        NOT NULL,
  payload       jsonb       NOT NULL,
  run_at        timestamptz NOT NULL,
  status        text        NOT NULL,
  attempt_count smallint    NOT NULL,
  max_attempts  smallint    NOT NULL,
  locked_by     text        NULL,
  locked_until  timestamptz NULL,
  last_error    text        NULL,
  dedupe_key    text        NULL,
  finished_at   timestamptz NULL,
  CONSTRAINT pk_job PRIMARY KEY (id),
  CONSTRAINT ck_job_status CHECK (status IN ('queued','running','succeeded','failed','cancelled')),
  CONSTRAINT ck_job_attempt_count CHECK (attempt_count >= 0),       -- review DBA F1-T17 butir 7
  CONSTRAINT ck_job_max_attempts CHECK (max_attempts >= 1)
);
CREATE INDEX ix_job_run_at_queued ON app.job (run_at) WHERE status = 'queued';
CREATE INDEX ix_job_status_finished_at ON app.job (status, finished_at)
  WHERE status IN ('succeeded','failed','cancelled');
CREATE UNIQUE INDEX uqp_job_dedupe_key ON app.job (dedupe_key) WHERE status IN ('queued','running');
CREATE INDEX ix_job_locked_until_running ON app.job (locked_until) WHERE status = 'running';   -- §9.3
ALTER TABLE app.job SET (autovacuum_vacuum_scale_factor = 0.02, autovacuum_analyze_scale_factor = 0.02);
COMMENT ON TABLE app.job IS 'class:O module:platform';

REVOKE ALL ON app.job FROM PUBLIC;
GRANT SELECT, INSERT, UPDATE, DELETE ON app.job TO rs_app;

-- A19.5 upload_slot (O): slot unggah yang belum dikonfirmasi; dihapus bersama berkasnya bila gagal
-- validasi atau kedaluwarsa (ADR-018 butir 2). Tanpa data pribadi selain ID akun.
CREATE TABLE app.upload_slot (
  id                uuid        NOT NULL,
  created_at        timestamptz NOT NULL DEFAULT now(),
  expires_at        timestamptz NOT NULL,
  owner_account_id  uuid        NULL,
  bucket            text        NOT NULL,
  object_key        text        NOT NULL,
  allowed_mime_type text        NOT NULL,
  max_size_bytes    integer     NOT NULL,
  CONSTRAINT pk_upload_slot PRIMARY KEY (id),
  CONSTRAINT fk_upload_slot_owner_account_id FOREIGN KEY (owner_account_id) REFERENCES app.account (id),
  CONSTRAINT uq_upload_slot_object_key UNIQUE (object_key),
  -- review DBA F1-T17 butir 2: nilai sama dengan stored_file.bucket (DD A19.5 "Sama dengan").
  CONSTRAINT ck_upload_slot_bucket CHECK (bucket IN
    ('identity_document','clinical','claim_evidence','license_document','export','public_content','public_catalog')),
  CONSTRAINT ck_upload_slot_max_size_bytes CHECK (max_size_bytes > 0)
);
CREATE INDEX ix_upload_slot_expires_at ON app.upload_slot (expires_at);   -- §9.3
COMMENT ON TABLE app.upload_slot IS 'class:O module:platform';

REVOKE ALL ON app.upload_slot FROM PUBLIC;
GRANT SELECT, INSERT, UPDATE, DELETE ON app.upload_slot TO rs_app;

-- A19.6 stored_file (M): metadata berkas yang lolos validasi; URL objek tidak disimpan (ADR-018).
CREATE TABLE app.stored_file (
  id               uuid        NOT NULL,
  created_at       timestamptz NOT NULL DEFAULT now(),
  created_by       uuid        NULL,
  updated_at       timestamptz NOT NULL DEFAULT now(),
  updated_by       uuid        NULL,
  deleted_at       timestamptz NULL,
  deleted_by       uuid        NULL,
  delete_reason    text        NULL,
  bucket           text        NOT NULL,
  object_key       text        NOT NULL,
  category         text        NOT NULL,
  owner_account_id uuid        NULL,
  container_type   text        NULL,
  container_id     uuid        NULL,
  mime_type        text        NOT NULL,
  size_bytes       integer     NOT NULL,
  sha256           bytea       NOT NULL,
  validated_at     timestamptz NOT NULL,
  purged_at        timestamptz NULL,
  CONSTRAINT pk_stored_file PRIMARY KEY (id),
  CONSTRAINT fk_stored_file_created_by FOREIGN KEY (created_by) REFERENCES app.account (id),
  CONSTRAINT fk_stored_file_updated_by FOREIGN KEY (updated_by) REFERENCES app.account (id),
  CONSTRAINT fk_stored_file_deleted_by FOREIGN KEY (deleted_by) REFERENCES app.account (id),
  CONSTRAINT fk_stored_file_owner_account_id FOREIGN KEY (owner_account_id) REFERENCES app.account (id),
  CONSTRAINT uq_stored_file_object_key UNIQUE (object_key),
  CONSTRAINT ck_stored_file_soft_delete CHECK (
    (deleted_at IS NULL AND deleted_by IS NULL AND delete_reason IS NULL)
    OR (deleted_at IS NOT NULL AND delete_reason IS NOT NULL)),
  CONSTRAINT ck_stored_file_bucket CHECK (bucket IN
    ('identity_document','clinical','claim_evidence','license_document','export','public_content','public_catalog')),
  CONSTRAINT ck_stored_file_mime_type CHECK (mime_type IN
    ('image/jpeg','image/png','application/pdf','text/csv','application/json')),
  CONSTRAINT ck_stored_file_size_bytes CHECK (size_bytes > 0),
  -- review DBA F1-T17 butir 5: wadah retensi = nilai DD §0.7 (DB aplikasi); tipe dan ID berpasangan.
  CONSTRAINT ck_stored_file_container_type CHECK (container_type IN
    ('consultation_chat','consultation_clinical','lab_result','identity_document','claim',
     'integration_client','partner_bank_account','complaint_ticket','data_subject_request','status_note')),
  CONSTRAINT ck_stored_file_container CHECK ((container_type IS NULL) = (container_id IS NULL))
);
CREATE INDEX ix_stored_file_container_type_container_id ON app.stored_file (container_type, container_id);
COMMENT ON TABLE app.stored_file IS 'class:M module:platform';

REVOKE ALL ON app.stored_file FROM PUBLIC;
GRANT SELECT, INSERT, UPDATE ON app.stored_file TO rs_app;          -- tanpa DELETE [DOK: ADR-003]
GRANT SELECT ON app.stored_file TO rs_privacy_worker;
-- tabel ini tanpa kolom "Anonimkan" di DD A19.6, sehingga privacy_worker tidak mendapat UPDATE per kolom

-- A19.7 data_key (MK, §4.10): DEK terbungkus per wadah; tanpa soft delete (KEPUTUSAN-44); trigger
-- satu arah (KEPUTUSAN-66, KEPUTUSAN-96). container_type = wadah DB aplikasi DD §0.7.
CREATE TABLE app.data_key (
  id                  uuid        NOT NULL,
  created_at          timestamptz NOT NULL DEFAULT now(),
  container_type      text        NOT NULL,
  container_id        uuid        NOT NULL,
  wrapped_dek         bytea       NULL,
  kek_ref             text        NOT NULL,
  retention_category  text        NULL,
  retention_anchor_at timestamptz NULL,
  shredded_at         timestamptz NULL,
  shred_reason        text        NULL,
  CONSTRAINT pk_data_key PRIMARY KEY (id),
  CONSTRAINT uq_data_key_container_type_container_id UNIQUE (container_type, container_id),
  CONSTRAINT uq_data_key_id_container_id UNIQUE (id, container_id),   -- sasaran FK komposit wadah (DD §0.2)
  CONSTRAINT ck_data_key_container_type CHECK (container_type IN
    ('consultation_chat','consultation_clinical','lab_result','identity_document','claim',
     'integration_client','partner_bank_account','complaint_ticket','data_subject_request','status_note')),
  CONSTRAINT ck_data_key_shredded CHECK ((wrapped_dek IS NULL) = (shredded_at IS NOT NULL)),
  CONSTRAINT ck_data_key_shred_reason CHECK (shred_reason IN ('retention_expired','erasure_request')),
  CONSTRAINT ck_data_key_shred_reason_shredded_at CHECK ((shred_reason IS NULL) = (shredded_at IS NULL))   -- review DBA butir 6
);
CREATE INDEX ix_data_key_retention_category_retention_anchor_at
  ON app.data_key (retention_category, retention_anchor_at) WHERE shredded_at IS NULL;
COMMENT ON TABLE app.data_key IS 'class:MK module:platform';

REVOKE ALL ON app.data_key FROM PUBLIC;
GRANT SELECT, INSERT ON app.data_key TO rs_app;
GRANT SELECT ON app.data_key TO rs_privacy_worker;
GRANT UPDATE (wrapped_dek, shredded_at, shred_reason) ON app.data_key TO rs_privacy_worker;   -- [SRC-35]
GRANT SELECT (id, wrapped_dek, kek_ref, shredded_at) ON app.data_key TO rs_key_rotation;
GRANT UPDATE (wrapped_dek, kek_ref) ON app.data_key TO rs_key_rotation;

CREATE TRIGGER tg_data_key_one_way BEFORE UPDATE ON app.data_key
  FOR EACH ROW EXECUTE FUNCTION util.fn_data_key_one_way('rs_privacy_worker', 'rs_key_rotation');
CREATE TRIGGER tg_data_key_reject_delete BEFORE DELETE ON app.data_key
  FOR EACH STATEMENT EXECUTE FUNCTION util.fn_reject_modification();
CREATE TRIGGER tg_data_key_reject_truncate BEFORE TRUNCATE ON app.data_key
  FOR EACH STATEMENT EXECUTE FUNCTION util.fn_reject_modification();

-- A19.10 data_key_retention_anchor (T, §4.12, CR-014, KEPUTUSAN-155): acuan retensi DEK sekali-isi.
CREATE TABLE app.data_key_retention_anchor (
  data_key_id         uuid        NOT NULL,
  retention_category  text        NOT NULL,
  retention_anchor_at timestamptz NOT NULL,
  created_at          timestamptz NOT NULL DEFAULT now(),
  created_by          uuid        NULL,
  CONSTRAINT pk_data_key_retention_anchor PRIMARY KEY (data_key_id),          -- sekali per DEK
  CONSTRAINT fk_data_key_retention_anchor_data_key_id FOREIGN KEY (data_key_id) REFERENCES app.data_key (id),
  CONSTRAINT fk_data_key_retention_anchor_created_by FOREIGN KEY (created_by) REFERENCES app.account (id)
);
CREATE INDEX ix_data_key_retention_anchor_category_anchor
  ON app.data_key_retention_anchor (retention_category, retention_anchor_at);
COMMENT ON TABLE app.data_key_retention_anchor IS 'class:T module:platform alias:dk_ret_anchor';

REVOKE ALL ON app.data_key_retention_anchor FROM PUBLIC;
GRANT SELECT, INSERT ON app.data_key_retention_anchor TO rs_app;
GRANT SELECT ON app.data_key_retention_anchor TO rs_privacy_worker;

CREATE TRIGGER tg_data_key_retention_anchor_reject_ud
  BEFORE UPDATE OR DELETE ON app.data_key_retention_anchor
  FOR EACH STATEMENT EXECUTE FUNCTION util.fn_reject_modification();
CREATE TRIGGER tg_data_key_retention_anchor_reject_truncate
  BEFORE TRUNCATE ON app.data_key_retention_anchor
  FOR EACH STATEMENT EXECUTE FUNCTION util.fn_reject_modification();

-- Satu sumber acuan waktu per DEK (§4.12): ditolak bila data_key.retention_anchor_at sudah terisi;
-- kategori wajib sama bila data_key.retention_category sudah terisi.
CREATE FUNCTION app.fn_data_key_retention_anchor_single_source()
RETURNS trigger
LANGUAGE plpgsql
SET search_path = pg_catalog, pg_temp
AS $$
DECLARE
  v_anchor   timestamptz;
  v_category text;
BEGIN
  SELECT dk.retention_anchor_at, dk.retention_category INTO v_anchor, v_category
    FROM app.data_key dk WHERE dk.id = NEW.data_key_id;
  IF v_anchor IS NOT NULL THEN
    RAISE EXCEPTION 'retention_anchor_already_set: data_key id=%', NEW.data_key_id
      USING ERRCODE = 'insufficient_privilege';
  END IF;
  IF v_category IS NOT NULL AND v_category IS DISTINCT FROM NEW.retention_category THEN
    RAISE EXCEPTION 'retention_category_mismatch: data_key id=%', NEW.data_key_id
      USING ERRCODE = 'insufficient_privilege';
  END IF;
  RETURN NEW;
END;
$$;
REVOKE ALL ON FUNCTION app.fn_data_key_retention_anchor_single_source() FROM PUBLIC;
COMMENT ON FUNCTION app.fn_data_key_retention_anchor_single_source() IS
  'Acuan retensi DEK sekali-isi: tolak bila data_key sudah punya acuan waktu atau kategori berbeda (skema fisik §4.12, CR-014)';

CREATE TRIGGER tg_data_key_retention_anchor_single_source
  BEFORE INSERT ON app.data_key_retention_anchor
  FOR EACH ROW EXECUTE FUNCTION app.fn_data_key_retention_anchor_single_source();
