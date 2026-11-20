-- db/migrations/app/20261120100000_privacy_consent.sql
-- F1-T19: tabel A3 persetujuan (consent_text_version, consent_item, consent_record) + hak role.
--
-- Modul kritis T1 (persetujuan dan hak subjek data; ADR-003, ADR-004, ADR-008, ADR-013 §5,
-- ADR-021 butir 4): review DBA + keamanan.
-- Sumber: docs/04-data-dictionary.md §0.3, §0.4 (STD-M, STD-L), §0.6 A, §0.8, A3.1-A3.3;
-- docs/05-physical-schema.md §4.1, §4.2 (3), (3b), (3c), (3d), §4.3, §4.9, §4.11, §11.2;
-- PRD v1.3 US-104 AC1-AC5, NFR-PRIV-05; KEPUTUSAN-40, KEPUTUSAN-154, KEPUTUSAN-159.
-- Maju saja (ADR-019); koreksi lewat berkas baru (§11.1).
--
-- Ketentuan:
-- - id tanpa nilai bawaan (ADR-002); created_at/updated_at DEFAULT now() (KEPUTUSAN-93); FK tanpa
--   CASCADE; FK aktor tidak diindeks; index hanya yang ada di DD A3 (§4.1 butir 3-4).
-- - Hak hanya ke role grup (§4.1 butir 7); TRUNCATE tidak diberikan; Master tanpa DELETE (ADR-003).
-- - A3 tidak punya kolom "Anonimkan" maupun kolom *_enc, sehingga pekerja privasi hanya SELECT di
--   Master dan tidak ada CHECK amplop (TD-07 tidak berlaku di berkas ini).
-- - consent_text_version: pola berlaku-mulai (effective_lock + no_backdate, §4.2 (3), (3b)) dan
--   kunci versi terbit (§4.2 (3c)); consent_item: kunci butir milik versi terbit (§4.2 (3d)).
--   Fungsi trigger khusus tabel berada di skema app (§4.2 (3c), (3d)); search_path persis
--   "pg_catalog, pg_temp" (CR-020, menggantikan "pg_catalog" di teks §4.2), rujukan berkualifikasi.
-- - consent_record (L): penolak UPDATE dan TRUNCATE + penjaga DELETE tingkat baris (§4.9).
--   Kategori retensi dan daftar end_action ditetapkan bersama PT-24 [TIDAK DIKETAHUI]. Sampai itu
--   argumen kategori berupa penanda "pending_pt_24" (bukan kategori DD A6.6) dan daftar end_action
--   kosong '{}', sehingga semua DELETE ditolak (§4.2 (2) catatan PT-24; dilaporkan F1-T19).

-- Penjaga (§11.2) dari templat db/test/templat-penjaga-lanjutan-app.sql.
DO $$
BEGIN
  IF current_user <> 'rs_migration' THEN
    RAISE EXCEPTION 'migration must run as rs_migration, got % (session %)', current_user, session_user;
  END IF;
  IF EXISTS (SELECT 1 FROM app._sqlx_migrations WHERE version > 20261120100000) THEN
    RAISE EXCEPTION 'migration order: version 20261120100000 is older than an applied migration (forward-only, ADR-019)';
  END IF;
END $$;
SET LOCAL lock_timeout = '5s';
SET LOCAL statement_timeout = '15min';

-- A3.1 consent_text_version (M): versi teks persetujuan (US-104 AC1, AC3, AC4; KEPUTUSAN-12).
CREATE TABLE app.consent_text_version (
  id                    uuid        NOT NULL,
  created_at            timestamptz NOT NULL DEFAULT now(),
  created_by            uuid        NULL,
  updated_at            timestamptz NOT NULL DEFAULT now(),
  updated_by            uuid        NULL,
  deleted_at            timestamptz NULL,
  deleted_by            uuid        NULL,
  delete_reason         text        NULL,
  consent_type          text        NOT NULL,
  version_label         text        NOT NULL,
  content               text        NOT NULL,
  effective_from        timestamptz NOT NULL,
  published_at          timestamptz NULL,
  change_notice_sent_at timestamptz NULL,
  CONSTRAINT pk_consent_text_version PRIMARY KEY (id),
  CONSTRAINT fk_consent_text_version_created_by FOREIGN KEY (created_by) REFERENCES app.account (id),
  CONSTRAINT fk_consent_text_version_updated_by FOREIGN KEY (updated_by) REFERENCES app.account (id),
  CONSTRAINT fk_consent_text_version_deleted_by FOREIGN KEY (deleted_by) REFERENCES app.account (id),
  CONSTRAINT ck_consent_text_version_soft_delete CHECK (
    (deleted_at IS NULL AND deleted_by IS NULL AND delete_reason IS NULL)
    OR (deleted_at IS NOT NULL AND delete_reason IS NOT NULL)),
  CONSTRAINT ck_consent_text_version_consent_type CHECK (consent_type IN
    ('patient_data_processing','family_adult_statement','guardian_consent'))
);
CREATE UNIQUE INDEX uqp_consent_text_version_consent_type_version_label
  ON app.consent_text_version (consent_type, version_label) WHERE deleted_at IS NULL;   -- [SRC-33]
COMMENT ON TABLE app.consent_text_version IS 'class:M module:privasi';

REVOKE ALL ON app.consent_text_version FROM PUBLIC;
GRANT SELECT, INSERT, UPDATE ON app.consent_text_version TO rs_app;   -- tanpa DELETE [DOK: ADR-003]
GRANT SELECT ON app.consent_text_version TO rs_privacy_worker;
-- tabel ini tanpa kolom "Anonimkan", sehingga privacy_worker tidak mendapat UPDATE per kolom

-- Pola berlaku-mulai (§4.2 (3), (3b), §4.11; KEPUTUSAN-159).
CREATE TRIGGER tg_consent_text_version_effective_lock
  BEFORE UPDATE ON app.consent_text_version
  FOR EACH ROW EXECUTE FUNCTION util.fn_effective_row_lock();
CREATE TRIGGER tg_consent_text_version_no_backdate
  BEFORE INSERT ON app.consent_text_version
  FOR EACH ROW EXECUTE FUNCTION util.fn_effective_row_no_backdate('rs_migration');

-- (3c) Kunci versi teks yang sudah terbit (§4.2 (3c), DBA5-B03, KEPUTUSAN-154).
CREATE FUNCTION app.fn_consent_text_version_published_lock()
RETURNS trigger
LANGUAGE plpgsql
SET search_path = pg_catalog, pg_temp
AS $$
BEGIN
  IF OLD.published_at IS NOT NULL THEN
    IF NEW.consent_type    IS DISTINCT FROM OLD.consent_type
       OR NEW.version_label  IS DISTINCT FROM OLD.version_label
       OR NEW.content        IS DISTINCT FROM OLD.content
       OR NEW.effective_from IS DISTINCT FROM OLD.effective_from
       OR NEW.published_at   IS DISTINCT FROM OLD.published_at THEN
      RAISE EXCEPTION 'published_row_locked: consent_text_version id=%', OLD.id
        USING ERRCODE = 'insufficient_privilege';
    END IF;
    IF OLD.change_notice_sent_at IS NOT NULL
       AND NEW.change_notice_sent_at IS DISTINCT FROM OLD.change_notice_sent_at THEN
      RAISE EXCEPTION 'published_row_locked: change_notice_sent_at id=%', OLD.id
        USING ERRCODE = 'insufficient_privilege';
    END IF;
  END IF;
  RETURN NEW;
END;
$$;
REVOKE ALL ON FUNCTION app.fn_consent_text_version_published_lock() FROM PUBLIC;
COMMENT ON FUNCTION app.fn_consent_text_version_published_lock() IS
  'Kunci isi versi teks persetujuan yang sudah terbit (skema fisik §4.2 (3c), KEPUTUSAN-154)';

CREATE TRIGGER tg_consent_text_version_published_lock
  BEFORE UPDATE ON app.consent_text_version
  FOR EACH ROW EXECUTE FUNCTION app.fn_consent_text_version_published_lock();

-- A3.2 consent_item (M): butir persetujuan per versi (US-104 AC3, US-1601 AC2, AC4).
CREATE TABLE app.consent_item (
  id                      uuid        NOT NULL,
  created_at              timestamptz NOT NULL DEFAULT now(),
  created_by              uuid        NULL,
  updated_at              timestamptz NOT NULL DEFAULT now(),
  updated_by              uuid        NULL,
  deleted_at              timestamptz NULL,
  deleted_by              uuid        NULL,
  delete_reason           text        NULL,
  consent_text_version_id uuid        NOT NULL,
  item_code               text        NOT NULL,
  label                   text        NOT NULL,
  is_primary              boolean     NOT NULL,
  is_required             boolean     NOT NULL,
  sort_order              smallint    NOT NULL,
  CONSTRAINT pk_consent_item PRIMARY KEY (id),
  CONSTRAINT fk_consent_item_created_by FOREIGN KEY (created_by) REFERENCES app.account (id),
  CONSTRAINT fk_consent_item_updated_by FOREIGN KEY (updated_by) REFERENCES app.account (id),
  CONSTRAINT fk_consent_item_deleted_by FOREIGN KEY (deleted_by) REFERENCES app.account (id),
  CONSTRAINT fk_consent_item_consent_text_version_id
    FOREIGN KEY (consent_text_version_id) REFERENCES app.consent_text_version (id),
  CONSTRAINT ck_consent_item_soft_delete CHECK (
    (deleted_at IS NULL AND deleted_by IS NULL AND delete_reason IS NULL)
    OR (deleted_at IS NOT NULL AND delete_reason IS NOT NULL))
);
CREATE UNIQUE INDEX uqp_consent_item_consent_text_version_id_item_code
  ON app.consent_item (consent_text_version_id, item_code) WHERE deleted_at IS NULL;   -- [SRC-33]
COMMENT ON TABLE app.consent_item IS 'class:M module:privasi';

REVOKE ALL ON app.consent_item FROM PUBLIC;
GRANT SELECT, INSERT, UPDATE ON app.consent_item TO rs_app;           -- tanpa DELETE [DOK: ADR-003]
GRANT SELECT ON app.consent_item TO rs_privacy_worker;
-- tabel ini tanpa kolom "Anonimkan", sehingga privacy_worker tidak mendapat UPDATE per kolom

-- (3d) Kunci butir milik versi terbit (§4.2 (3d), KEPUTUSAN-154): INSERT, UPDATE, soft delete, dan
-- pemindahan butir keluar dari versi terbit ditolak.
CREATE FUNCTION app.fn_consent_item_published_lock()
RETURNS trigger
LANGUAGE plpgsql
SET search_path = pg_catalog, pg_temp
AS $$
BEGIN
  IF EXISTS (SELECT 1 FROM app.consent_text_version v
              WHERE v.id IN (NEW.consent_text_version_id,
                             CASE WHEN TG_OP = 'UPDATE' THEN OLD.consent_text_version_id END)
                AND v.published_at IS NOT NULL) THEN
    RAISE EXCEPTION 'published_row_locked: consent_item (version published) id=%', NEW.id
      USING ERRCODE = 'insufficient_privilege';
  END IF;
  RETURN NEW;
END;
$$;
REVOKE ALL ON FUNCTION app.fn_consent_item_published_lock() FROM PUBLIC;
COMMENT ON FUNCTION app.fn_consent_item_published_lock() IS
  'Butir persetujuan milik versi terbit tidak dapat ditambah, diubah, atau dipindah (skema fisik §4.2 (3d))';

CREATE TRIGGER tg_consent_item_published_lock
  BEFORE INSERT OR UPDATE ON app.consent_item
  FOR EACH ROW EXECUTE FUNCTION app.fn_consent_item_published_lock();

-- A3.3 consent_record (L): bukti persetujuan diberikan atau ditarik (US-104 AC2, US-108 AC1-AC2,
-- US-1601, E-13, E-15, NFR-PRIV-05). created_at = waktu persetujuan (jam server).
CREATE TABLE app.consent_record (
  id                      uuid        NOT NULL,
  created_at              timestamptz NOT NULL DEFAULT now(),
  patient_id              uuid        NOT NULL,
  care_recipient_id       uuid        NULL,
  consent_text_version_id uuid        NOT NULL,
  consent_item_id         uuid        NOT NULL,
  action                  text        NOT NULL,
  channel                 text        NOT NULL,
  correlation_id          uuid        NOT NULL,
  CONSTRAINT pk_consent_record PRIMARY KEY (id),
  CONSTRAINT fk_consent_record_patient_id FOREIGN KEY (patient_id) REFERENCES app.patient (id),
  CONSTRAINT fk_consent_record_care_recipient_id FOREIGN KEY (care_recipient_id) REFERENCES app.care_recipient (id),
  CONSTRAINT fk_consent_record_consent_text_version_id
    FOREIGN KEY (consent_text_version_id) REFERENCES app.consent_text_version (id),
  CONSTRAINT fk_consent_record_consent_item_id FOREIGN KEY (consent_item_id) REFERENCES app.consent_item (id),
  CONSTRAINT ck_consent_record_action CHECK (action IN ('granted','withdrawn')),
  CONSTRAINT ck_consent_record_channel CHECK (channel IN ('patient_app','backoffice'))
);
CREATE INDEX ix_consent_record_patient_id_consent_item_id_created_at
  ON app.consent_record (patient_id, consent_item_id, created_at DESC);   -- status terkini (US-1601 AC1)
COMMENT ON TABLE app.consent_record IS 'class:L module:privasi log_category:pending_pt_24';

REVOKE ALL ON app.consent_record FROM PUBLIC;
GRANT SELECT, INSERT ON app.consent_record TO rs_app;
GRANT SELECT (id, created_at, patient_id, care_recipient_id) ON app.consent_record TO rs_privacy_worker;
GRANT DELETE ON app.consent_record TO rs_privacy_worker;              -- dibatasi trigger penjaga

CREATE TRIGGER tg_consent_record_reject_update
  BEFORE UPDATE ON app.consent_record
  FOR EACH STATEMENT EXECUTE FUNCTION util.fn_reject_modification();
CREATE TRIGGER tg_consent_record_reject_truncate
  BEFORE TRUNCATE ON app.consent_record
  FOR EACH STATEMENT EXECUTE FUNCTION util.fn_reject_modification();
CREATE TRIGGER tg_consent_record_guard_delete
  BEFORE DELETE ON app.consent_record
  FOR EACH ROW EXECUTE FUNCTION util.fn_guard_log_delete('pending_pt_24', 'created_at', '{}', 'rs_privacy_worker');
