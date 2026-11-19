-- db/migrations/app/20261119120000_profile_identity_reason_code.sql
-- F1-T18: tabel A2 profil dan verifikasi identitas (patient, care_recipient, address,
-- identity_document, identity_verification_request beserta _status_event dan _state,
-- account_owner_nik_registry) dan A6.7 reason_code + hak role.
--
-- Modul kritis T1 (ADR-008, ADR-012, ADR-013, ADR-018, ADR-021 butir 4): review DBA + keamanan.
-- Sumber: docs/04-data-dictionary.md §0.2, §0.3, §0.4 (STD-M, STD-T, STD-R, STD-P), §0.6 A, §0.7,
-- A2.1-A2.8, A6.7; docs/05-physical-schema.md §1.2 (alias), §4.1 butir 5 (CHECK amplop *_enc),
-- §4.3 (hak per kolom pekerja privasi), §4.5-§4.7 (templat T, R, P), §7.1, §7.2 (blind index),
-- §11.2; PRD v1.3 US-106, US-201 (UP-40, AC5 keputusan pertama, AC7 registri NIK).
-- Maju saja (ADR-019); koreksi lewat berkas baru (§11.1).
--
-- Ketentuan:
-- - id tanpa nilai bawaan (ADR-002); created_at/updated_at DEFAULT now() (KEPUTUSAN-93); FK tanpa
--   CASCADE; FK aktor tidak diindeks; index FK hanya yang ada di DD (§4.1 butir 3-4).
-- - Hak hanya ke role grup (§4.1 butir 7); TRUNCATE tidak diberikan; Master tanpa DELETE (ADR-003).
-- - Pekerja privasi: Master = SELECT + UPDATE per kolom "Anonimkan" + anonymized_at, updated_at,
--   updated_by (registri NIK ditambah kolom soft delete); Transaksi = SELECT kolom ID, FK subjek,
--   created_at; Riwayat = SELECT id, note_data_key_id (DD §0.6 A).
-- - anonymized_at di address, identity_document, account_owner_nik_registry: DD §0.2/§0.6 A dan
--   skema fisik §12 T-07 mewajibkan hak UPDATE pekerja privasi atas anonymized_at di setiap tabel
--   Master yang punya kolom "Anonimkan", sedangkan tabel kolom DD A2.3, A2.4, A2.8 tidak
--   mencantumkannya. Kolom NULL ditambahkan supaya aturan umum dapat diterapkan (dilaporkan F1-T18).
-- - identity_verification_request_status_event.actor_partner_id (STD-R, FK->partner) BELUM dibuat:
--   tabel partner (A5) tidak ada di Fase 1 (preseden upload_slot.owner_partner_id, F1-T17).
--   Ditambahkan (NULL, expand §11.3) oleh migrasi yang membuat partner.
-- - Nama > 60 byte memakai alias tabel di komentar (§1.2): ivr, ivr_status_event, ivr_state,
--   owner_nik_registry.
-- - Kolom *_enc: CHECK amplop v1 (§4.1 butir 5, KEPUTUSAN-104); kolom *_bidx: CHECK 32 byte dan
--   versi kunci >= 1 (§7.2, KEPUTUSAN-105). Diperiksa generik oleh T-20 (db/test/lib.sql).

-- Penjaga (§11.2) dari templat db/test/templat-penjaga-lanjutan-app.sql.
DO $$
BEGIN
  IF current_user <> 'rs_migration' THEN
    RAISE EXCEPTION 'migration must run as rs_migration, got % (session %)', current_user, session_user;
  END IF;
  IF EXISTS (SELECT 1 FROM app._sqlx_migrations WHERE version > 20261119120000) THEN
    RAISE EXCEPTION 'migration order: version 20261119120000 is older than an applied migration (forward-only, ADR-019)';
  END IF;
END $$;
SET LOCAL lock_timeout = '5s';
SET LOCAL statement_timeout = '15min';

-- A6.7 reason_code (M): daftar alasan per konteks. Isi awal (US-201 AC3, UP-75) lewat migrasi data
-- terpisah (DDL dan DML tidak dicampur, §11.3). Pemilik modul: platform (data rujukan lintas modul).
CREATE TABLE app.reason_code (
  id            uuid        NOT NULL,
  created_at    timestamptz NOT NULL DEFAULT now(),
  created_by    uuid        NULL,
  updated_at    timestamptz NOT NULL DEFAULT now(),
  updated_by    uuid        NULL,
  deleted_at    timestamptz NULL,
  deleted_by    uuid        NULL,
  delete_reason text        NULL,
  context       text        NOT NULL,
  code          text        NOT NULL,
  label         text        NOT NULL,
  CONSTRAINT pk_reason_code PRIMARY KEY (id),
  CONSTRAINT fk_reason_code_created_by FOREIGN KEY (created_by) REFERENCES app.account (id),
  CONSTRAINT fk_reason_code_updated_by FOREIGN KEY (updated_by) REFERENCES app.account (id),
  CONSTRAINT fk_reason_code_deleted_by FOREIGN KEY (deleted_by) REFERENCES app.account (id),
  CONSTRAINT ck_reason_code_soft_delete CHECK (
    (deleted_at IS NULL AND deleted_by IS NULL AND delete_reason IS NULL)
    OR (deleted_at IS NOT NULL AND delete_reason IS NOT NULL)),
  CONSTRAINT ck_reason_code_context CHECK (context IN
    ('identity_rejection','health_worker_rejection','bank_account_rejection','order_cancellation',
     'claim_rejection','dsr_rejection','status_transition'))
);
CREATE UNIQUE INDEX uqp_reason_code_context_code
  ON app.reason_code (context, code) WHERE deleted_at IS NULL;   -- [SRC-33]
COMMENT ON TABLE app.reason_code IS 'class:M module:platform';

REVOKE ALL ON app.reason_code FROM PUBLIC;
GRANT SELECT, INSERT, UPDATE ON app.reason_code TO rs_app;          -- tanpa DELETE [DOK: ADR-003]
GRANT SELECT ON app.reason_code TO rs_privacy_worker;
-- tabel ini tanpa kolom "Anonimkan", sehingga privacy_worker tidak mendapat UPDATE per kolom

-- A2.1 patient (M): profil pemilik akun aplikasi pasien.
CREATE TABLE app.patient (
  id                           uuid        NOT NULL,
  created_at                   timestamptz NOT NULL DEFAULT now(),
  created_by                   uuid        NULL,
  updated_at                   timestamptz NOT NULL DEFAULT now(),
  updated_by                   uuid        NULL,
  deleted_at                   timestamptz NULL,
  deleted_by                   uuid        NULL,
  delete_reason                text        NULL,
  account_id                   uuid        NOT NULL,
  phone                        text        NULL,
  processing_status            text        NOT NULL,
  processing_status_changed_at timestamptz NULL,
  anonymized_at                timestamptz NULL,
  CONSTRAINT pk_patient PRIMARY KEY (id),
  CONSTRAINT fk_patient_created_by FOREIGN KEY (created_by) REFERENCES app.account (id),
  CONSTRAINT fk_patient_updated_by FOREIGN KEY (updated_by) REFERENCES app.account (id),
  CONSTRAINT fk_patient_deleted_by FOREIGN KEY (deleted_by) REFERENCES app.account (id),
  CONSTRAINT fk_patient_account_id FOREIGN KEY (account_id) REFERENCES app.account (id),
  CONSTRAINT ck_patient_soft_delete CHECK (
    (deleted_at IS NULL AND deleted_by IS NULL AND delete_reason IS NULL)
    OR (deleted_at IS NOT NULL AND delete_reason IS NOT NULL)),
  CONSTRAINT ck_patient_processing_status CHECK (processing_status IN
    ('normal','consent_withdrawn','restricted'))
);
CREATE UNIQUE INDEX uqp_patient_account_id ON app.patient (account_id) WHERE deleted_at IS NULL;
COMMENT ON TABLE app.patient IS 'class:M module:profil';
COMMENT ON COLUMN app.patient.phone IS 'anonymize';

REVOKE ALL ON app.patient FROM PUBLIC;
GRANT SELECT, INSERT, UPDATE ON app.patient TO rs_app;              -- tanpa DELETE [DOK: ADR-003]
GRANT SELECT ON app.patient TO rs_privacy_worker;
GRANT UPDATE (phone, anonymized_at, updated_at, updated_by) ON app.patient TO rs_privacy_worker;

-- A2.2 care_recipient (M): pemilik akun atau anggota keluarga (KEPUTUSAN-41). FK
-- latest_verification_request_id ditambahkan setelah identity_verification_request dibuat.
CREATE TABLE app.care_recipient (
  id                             uuid        NOT NULL,
  created_at                     timestamptz NOT NULL DEFAULT now(),
  created_by                     uuid        NULL,
  updated_at                     timestamptz NOT NULL DEFAULT now(),
  updated_by                     uuid        NULL,
  deleted_at                     timestamptz NULL,
  deleted_by                     uuid        NULL,
  delete_reason                  text        NULL,
  patient_id                     uuid        NOT NULL,
  is_account_owner               boolean     NOT NULL,
  relationship                   text        NULL,
  full_name                      text        NOT NULL,
  birth_date                     date        NOT NULL,
  sex                            text        NOT NULL,
  nik_bidx                       bytea       NULL,
  nik_bidx_key_version           smallint    NULL,
  nik_last4                      char(4)     NULL,
  latest_verification_request_id uuid        NULL,
  anonymized_at                  timestamptz NULL,
  CONSTRAINT pk_care_recipient PRIMARY KEY (id),
  CONSTRAINT fk_care_recipient_created_by FOREIGN KEY (created_by) REFERENCES app.account (id),
  CONSTRAINT fk_care_recipient_updated_by FOREIGN KEY (updated_by) REFERENCES app.account (id),
  CONSTRAINT fk_care_recipient_deleted_by FOREIGN KEY (deleted_by) REFERENCES app.account (id),
  CONSTRAINT fk_care_recipient_patient_id FOREIGN KEY (patient_id) REFERENCES app.patient (id),
  CONSTRAINT ck_care_recipient_soft_delete CHECK (
    (deleted_at IS NULL AND deleted_by IS NULL AND delete_reason IS NULL)
    OR (deleted_at IS NOT NULL AND delete_reason IS NOT NULL)),
  CONSTRAINT ck_care_recipient_relationship CHECK (is_account_owner = (relationship IS NULL)),
  CONSTRAINT ck_care_recipient_sex CHECK (sex IN ('male','female')),
  CONSTRAINT ck_care_recipient_nik_bidx CHECK (octet_length(nik_bidx) = 32),
  CONSTRAINT ck_care_recipient_nik_bidx_key_version CHECK (nik_bidx_key_version >= 1)
);
CREATE UNIQUE INDEX uqp_care_recipient_patient_id_account_owner
  ON app.care_recipient (patient_id) WHERE is_account_owner AND deleted_at IS NULL;
CREATE UNIQUE INDEX uqp_care_recipient_patient_id_nik_bidx
  ON app.care_recipient (patient_id, nik_bidx) WHERE deleted_at IS NULL AND nik_bidx IS NOT NULL;
CREATE INDEX ix_care_recipient_patient_id ON app.care_recipient (patient_id) WHERE deleted_at IS NULL;
COMMENT ON TABLE app.care_recipient IS 'class:M module:profil';
COMMENT ON COLUMN app.care_recipient.full_name IS 'anonymize';
COMMENT ON COLUMN app.care_recipient.birth_date IS 'anonymize';
COMMENT ON COLUMN app.care_recipient.sex IS 'anonymize';
COMMENT ON COLUMN app.care_recipient.nik_bidx IS 'anonymize';
COMMENT ON COLUMN app.care_recipient.nik_last4 IS 'anonymize';

REVOKE ALL ON app.care_recipient FROM PUBLIC;
GRANT SELECT, INSERT, UPDATE ON app.care_recipient TO rs_app;       -- tanpa DELETE [DOK: ADR-003]
GRANT SELECT ON app.care_recipient TO rs_privacy_worker;
GRANT UPDATE (full_name, birth_date, sex, nik_bidx, nik_last4, anonymized_at, updated_at, updated_by)
  ON app.care_recipient TO rs_privacy_worker;                       -- templat §4.3

-- A2.3 address (M): alamat pasien untuk pengiriman produk dan home visit.
CREATE TABLE app.address (
  id              uuid         NOT NULL,
  created_at      timestamptz  NOT NULL DEFAULT now(),
  created_by      uuid         NULL,
  updated_at      timestamptz  NOT NULL DEFAULT now(),
  updated_by      uuid         NULL,
  deleted_at      timestamptz  NULL,
  deleted_by      uuid         NULL,
  delete_reason   text         NULL,
  patient_id      uuid         NOT NULL,
  label           text         NULL,
  recipient_name  text         NOT NULL,
  recipient_phone text         NOT NULL,
  address_line    text         NOT NULL,
  province_code   text         NULL,
  city_code       text         NULL,
  district_code   text         NULL,
  province_name   text         NOT NULL,
  city_name       text         NOT NULL,
  postal_code     text         NULL,
  latitude        numeric(9,6) NULL,
  longitude       numeric(9,6) NULL,
  time_zone       text         NOT NULL,
  notes           text         NULL,
  is_default      boolean      NOT NULL,
  anonymized_at   timestamptz  NULL,
  CONSTRAINT pk_address PRIMARY KEY (id),
  CONSTRAINT fk_address_created_by FOREIGN KEY (created_by) REFERENCES app.account (id),
  CONSTRAINT fk_address_updated_by FOREIGN KEY (updated_by) REFERENCES app.account (id),
  CONSTRAINT fk_address_deleted_by FOREIGN KEY (deleted_by) REFERENCES app.account (id),
  CONSTRAINT fk_address_patient_id FOREIGN KEY (patient_id) REFERENCES app.patient (id),
  CONSTRAINT ck_address_soft_delete CHECK (
    (deleted_at IS NULL AND deleted_by IS NULL AND delete_reason IS NULL)
    OR (deleted_at IS NOT NULL AND delete_reason IS NOT NULL))
);
CREATE UNIQUE INDEX uqp_address_patient_id_default
  ON app.address (patient_id) WHERE is_default AND deleted_at IS NULL;
CREATE INDEX ix_address_patient_id ON app.address (patient_id) WHERE deleted_at IS NULL;
COMMENT ON TABLE app.address IS 'class:M module:profil';
COMMENT ON COLUMN app.address.recipient_name IS 'anonymize';
COMMENT ON COLUMN app.address.recipient_phone IS 'anonymize';
COMMENT ON COLUMN app.address.address_line IS 'anonymize';
COMMENT ON COLUMN app.address.province_code IS 'anonymize';
COMMENT ON COLUMN app.address.city_code IS 'anonymize';
COMMENT ON COLUMN app.address.district_code IS 'anonymize';
COMMENT ON COLUMN app.address.postal_code IS 'anonymize';
COMMENT ON COLUMN app.address.latitude IS 'anonymize';
COMMENT ON COLUMN app.address.longitude IS 'anonymize';
COMMENT ON COLUMN app.address.notes IS 'anonymize';

REVOKE ALL ON app.address FROM PUBLIC;
GRANT SELECT, INSERT, UPDATE ON app.address TO rs_app;              -- tanpa DELETE [DOK: ADR-003]
GRANT SELECT ON app.address TO rs_privacy_worker;
GRANT UPDATE (recipient_name, recipient_phone, address_line, province_code, city_code, district_code,
              postal_code, latitude, longitude, notes, anonymized_at, updated_at, updated_by)
  ON app.address TO rs_privacy_worker;

-- A2.4 identity_document (M): satu baris per pengajuan dokumen (NIK + foto KTP/KK). NIK terenkripsi
-- dengan DEK wadah identity_document (DD §0.7); foto di bucket identity_document (ADR-018).
CREATE TABLE app.identity_document (
  id                   uuid        NOT NULL,
  created_at           timestamptz NOT NULL DEFAULT now(),
  created_by           uuid        NULL,
  updated_at           timestamptz NOT NULL DEFAULT now(),
  updated_by           uuid        NULL,
  deleted_at           timestamptz NULL,
  deleted_by           uuid        NULL,
  delete_reason        text        NULL,
  care_recipient_id    uuid        NOT NULL,
  document_type        text        NOT NULL,
  nik_enc              bytea       NOT NULL,
  nik_bidx             bytea       NOT NULL,
  nik_bidx_key_version smallint    NOT NULL,
  nik_last4            char(4)     NOT NULL,
  data_key_id          uuid        NOT NULL,
  file_id              uuid        NOT NULL,
  anonymized_at        timestamptz NULL,
  CONSTRAINT pk_identity_document PRIMARY KEY (id),
  CONSTRAINT fk_identity_document_created_by FOREIGN KEY (created_by) REFERENCES app.account (id),
  CONSTRAINT fk_identity_document_updated_by FOREIGN KEY (updated_by) REFERENCES app.account (id),
  CONSTRAINT fk_identity_document_deleted_by FOREIGN KEY (deleted_by) REFERENCES app.account (id),
  CONSTRAINT fk_identity_document_care_recipient_id FOREIGN KEY (care_recipient_id) REFERENCES app.care_recipient (id),
  CONSTRAINT fk_identity_document_data_key_id FOREIGN KEY (data_key_id) REFERENCES app.data_key (id),
  CONSTRAINT fk_identity_document_file_id FOREIGN KEY (file_id) REFERENCES app.stored_file (id),
  CONSTRAINT uq_identity_document_data_key_id UNIQUE (data_key_id),
  CONSTRAINT ck_identity_document_soft_delete CHECK (
    (deleted_at IS NULL AND deleted_by IS NULL AND delete_reason IS NULL)
    OR (deleted_at IS NOT NULL AND delete_reason IS NOT NULL)),
  CONSTRAINT ck_identity_document_document_type CHECK (document_type IN ('ktp','kk')),
  CONSTRAINT ck_identity_document_nik_enc_format CHECK (nik_enc IS NULL
    OR (octet_length(nik_enc) >= 29 AND get_byte(nik_enc, 0) = 1)),
  CONSTRAINT ck_identity_document_nik_bidx CHECK (octet_length(nik_bidx) = 32),
  CONSTRAINT ck_identity_document_nik_bidx_key_version CHECK (nik_bidx_key_version >= 1)
);
CREATE INDEX ix_identity_document_care_recipient_id_created_at
  ON app.identity_document (care_recipient_id, created_at DESC);
COMMENT ON TABLE app.identity_document IS 'class:M module:profil';
COMMENT ON COLUMN app.identity_document.nik_bidx IS 'anonymize';
COMMENT ON COLUMN app.identity_document.nik_last4 IS 'anonymize';

REVOKE ALL ON app.identity_document FROM PUBLIC;
GRANT SELECT, INSERT, UPDATE ON app.identity_document TO rs_app;    -- tanpa DELETE [DOK: ADR-003]
GRANT SELECT ON app.identity_document TO rs_privacy_worker;
GRANT UPDATE (nik_bidx, nik_last4, anonymized_at, updated_at, updated_by)
  ON app.identity_document TO rs_privacy_worker;                    -- nik_enc musnah bersama DEK

-- A2.5 identity_verification_request (T): satu siklus pemeriksaan Verifikator (KEPUTUSAN-41).
CREATE TABLE app.identity_verification_request (
  id                   uuid        NOT NULL,
  created_at           timestamptz NOT NULL DEFAULT now(),
  created_by           uuid        NULL,
  care_recipient_id    uuid        NOT NULL,
  identity_document_id uuid        NOT NULL,
  trigger_type         text        NOT NULL,
  CONSTRAINT pk_identity_verification_request PRIMARY KEY (id),
  CONSTRAINT fk_identity_verification_request_created_by FOREIGN KEY (created_by) REFERENCES app.account (id),
  CONSTRAINT fk_identity_verification_request_care_recipient_id
    FOREIGN KEY (care_recipient_id) REFERENCES app.care_recipient (id),
  CONSTRAINT fk_identity_verification_request_identity_document_id
    FOREIGN KEY (identity_document_id) REFERENCES app.identity_document (id),
  CONSTRAINT ck_identity_verification_request_trigger_type CHECK (trigger_type IN
    ('document_submitted','profile_changed'))
);
CREATE INDEX ix_ivr_care_recipient_id_created_at
  ON app.identity_verification_request (care_recipient_id, created_at DESC);
COMMENT ON TABLE app.identity_verification_request IS 'class:T module:profil alias:ivr';

REVOKE ALL ON app.identity_verification_request FROM PUBLIC;
GRANT SELECT, INSERT ON app.identity_verification_request TO rs_app;
GRANT SELECT (id, created_at, care_recipient_id) ON app.identity_verification_request TO rs_privacy_worker;

CREATE TRIGGER tg_identity_verification_request_reject_ud
  BEFORE UPDATE OR DELETE ON app.identity_verification_request
  FOR EACH STATEMENT EXECUTE FUNCTION util.fn_reject_modification();
CREATE TRIGGER tg_identity_verification_request_reject_truncate
  BEFORE TRUNCATE ON app.identity_verification_request
  FOR EACH STATEMENT EXECUTE FUNCTION util.fn_reject_modification();

ALTER TABLE app.care_recipient
  ADD CONSTRAINT fk_care_recipient_latest_verification_request_id
  FOREIGN KEY (latest_verification_request_id) REFERENCES app.identity_verification_request (id);

-- A2.6 identity_verification_request_status_event (R): STD-R; keputusan pertama menang lewat
-- UQ (request, version) (US-201 AC5, US-106 AC5, AB-36). note_enc memakai wadah status_note.
CREATE TABLE app.identity_verification_request_status_event (
  id                               uuid        NOT NULL,
  identity_verification_request_id uuid        NOT NULL,
  version                          integer     NOT NULL,
  from_status                      text        NULL,
  to_status                        text        NOT NULL,
  reason_code                      text        NULL,
  note_enc                         bytea       NULL,
  note_data_key_id                 uuid        NULL,
  actor_type                       text        NOT NULL,
  actor_account_id                 uuid        NULL,
  correlation_id                   uuid        NOT NULL,
  created_at                       timestamptz NOT NULL DEFAULT now(),
  CONSTRAINT pk_identity_verification_request_status_event PRIMARY KEY (id),
  CONSTRAINT fk_ivr_status_event_identity_verification_request_id
    FOREIGN KEY (identity_verification_request_id) REFERENCES app.identity_verification_request (id),
  CONSTRAINT fk_ivr_status_event_note_data_key_id FOREIGN KEY (note_data_key_id) REFERENCES app.data_key (id),
  CONSTRAINT fk_ivr_status_event_actor_account_id FOREIGN KEY (actor_account_id) REFERENCES app.account (id),
  CONSTRAINT uq_ivr_status_event_identity_verification_request_id_version
    UNIQUE (identity_verification_request_id, version),                       -- AB-36
  CONSTRAINT uq_ivr_status_event_request_id_version_to_status
    UNIQUE (identity_verification_request_id, version, to_status),            -- sasaran FK proyeksi (§4.7)
  CONSTRAINT ck_identity_verification_request_status_event_version CHECK (version >= 1),
  CONSTRAINT ck_identity_verification_request_status_event_from_status CHECK ((version = 1) = (from_status IS NULL)),
  CONSTRAINT ck_identity_verification_request_status_event_to_status CHECK (to_status IN
    ('pending','approved','rejected','cancelled')),
  CONSTRAINT ck_identity_verification_request_status_event_reason_code CHECK (
    to_status <> 'rejected' OR reason_code IS NOT NULL),                      -- US-201 AC3
  CONSTRAINT ck_identity_verification_request_status_event_note CHECK ((note_enc IS NULL) = (note_data_key_id IS NULL)),
  CONSTRAINT ck_ivr_status_event_note_enc_format CHECK (note_enc IS NULL
    OR (octet_length(note_enc) >= 29 AND get_byte(note_enc, 0) = 1)),
  CONSTRAINT ck_identity_verification_request_status_event_actor_type CHECK (actor_type IN
    ('patient','health_worker','rs_admin','partner_admin','partner_system','payment_gateway','courier','system'))
);
COMMENT ON TABLE app.identity_verification_request_status_event IS 'class:R module:profil alias:ivr_status_event';

REVOKE ALL ON app.identity_verification_request_status_event FROM PUBLIC;
GRANT SELECT, INSERT ON app.identity_verification_request_status_event TO rs_app;
GRANT SELECT (id, note_data_key_id) ON app.identity_verification_request_status_event TO rs_privacy_worker;

CREATE TRIGGER tg_identity_verification_request_status_event_reject_ud
  BEFORE UPDATE OR DELETE ON app.identity_verification_request_status_event
  FOR EACH STATEMENT EXECUTE FUNCTION util.fn_reject_modification();
CREATE TRIGGER tg_ivr_status_event_reject_truncate
  BEFORE TRUNCATE ON app.identity_verification_request_status_event
  FOR EACH STATEMENT EXECUTE FUNCTION util.fn_reject_modification();

-- A2.7 identity_verification_request_state (P): STD-P + FK komposit ke riwayat (KEPUTUSAN-142).
CREATE TABLE app.identity_verification_request_state (
  identity_verification_request_id uuid        NOT NULL,
  status                           text        NOT NULL,
  version                          integer     NOT NULL,
  updated_at                       timestamptz NOT NULL DEFAULT now(),
  CONSTRAINT pk_identity_verification_request_state PRIMARY KEY (identity_verification_request_id),
  CONSTRAINT fk_ivr_state_identity_verification_request_id
    FOREIGN KEY (identity_verification_request_id) REFERENCES app.identity_verification_request (id),
  CONSTRAINT fk_ivr_state_request_id_version_status
    FOREIGN KEY (identity_verification_request_id, version, status)
    REFERENCES app.identity_verification_request_status_event (identity_verification_request_id, version, to_status),
  CONSTRAINT ck_identity_verification_request_state_status CHECK (status IN
    ('pending','approved','rejected','cancelled')),
  CONSTRAINT ck_identity_verification_request_state_version CHECK (version >= 1)
);
CREATE INDEX ix_ivr_state_status_updated_at_pending
  ON app.identity_verification_request_state (status, updated_at) WHERE status = 'pending';   -- US-201 AC1
COMMENT ON TABLE app.identity_verification_request_state IS 'class:P module:profil alias:ivr_state';

REVOKE ALL ON app.identity_verification_request_state FROM PUBLIC;
GRANT SELECT, INSERT, UPDATE ON app.identity_verification_request_state TO rs_app;   -- tanpa DELETE [DOK: ADR-008]

-- A2.8 account_owner_nik_registry (M): NIK yang disetujui sebagai pemilik akun (UP-40 lintas akun,
-- US-201 AC7). Disisipkan dalam transaksi persetujuan; UQp menolak persetujuan kedua.
CREATE TABLE app.account_owner_nik_registry (
  id                               uuid        NOT NULL,
  created_at                       timestamptz NOT NULL DEFAULT now(),
  created_by                       uuid        NULL,
  updated_at                       timestamptz NOT NULL DEFAULT now(),
  updated_by                       uuid        NULL,
  deleted_at                       timestamptz NULL,
  deleted_by                       uuid        NULL,
  delete_reason                    text        NULL,
  nik_bidx                         bytea       NOT NULL,
  nik_bidx_key_version             smallint    NOT NULL,
  care_recipient_id                uuid        NOT NULL,
  identity_verification_request_id uuid        NOT NULL,
  anonymized_at                    timestamptz NULL,
  CONSTRAINT pk_account_owner_nik_registry PRIMARY KEY (id),
  CONSTRAINT fk_account_owner_nik_registry_created_by FOREIGN KEY (created_by) REFERENCES app.account (id),
  CONSTRAINT fk_account_owner_nik_registry_updated_by FOREIGN KEY (updated_by) REFERENCES app.account (id),
  CONSTRAINT fk_account_owner_nik_registry_deleted_by FOREIGN KEY (deleted_by) REFERENCES app.account (id),
  CONSTRAINT fk_account_owner_nik_registry_care_recipient_id
    FOREIGN KEY (care_recipient_id) REFERENCES app.care_recipient (id),
  CONSTRAINT fk_owner_nik_registry_identity_verification_request_id
    FOREIGN KEY (identity_verification_request_id) REFERENCES app.identity_verification_request (id),
  CONSTRAINT ck_account_owner_nik_registry_soft_delete CHECK (
    (deleted_at IS NULL AND deleted_by IS NULL AND delete_reason IS NULL)
    OR (deleted_at IS NOT NULL AND delete_reason IS NOT NULL)),
  CONSTRAINT ck_account_owner_nik_registry_nik_bidx CHECK (octet_length(nik_bidx) = 32),
  CONSTRAINT ck_account_owner_nik_registry_nik_bidx_key_version CHECK (nik_bidx_key_version >= 1)
);
CREATE UNIQUE INDEX uqp_account_owner_nik_registry_nik_bidx
  ON app.account_owner_nik_registry (nik_bidx) WHERE deleted_at IS NULL;            -- [SRC-33]
CREATE UNIQUE INDEX uqp_account_owner_nik_registry_care_recipient_id
  ON app.account_owner_nik_registry (care_recipient_id) WHERE deleted_at IS NULL;
COMMENT ON TABLE app.account_owner_nik_registry IS 'class:M module:profil alias:owner_nik_registry';
COMMENT ON COLUMN app.account_owner_nik_registry.nik_bidx IS 'anonymize';

REVOKE ALL ON app.account_owner_nik_registry FROM PUBLIC;
GRANT SELECT, INSERT, UPDATE ON app.account_owner_nik_registry TO rs_app;   -- tanpa DELETE [DOK: ADR-003]
GRANT SELECT ON app.account_owner_nik_registry TO rs_privacy_worker;
GRANT UPDATE (nik_bidx, anonymized_at, updated_at, updated_by, deleted_at, deleted_by, delete_reason)
  ON app.account_owner_nik_registry TO rs_privacy_worker;           -- soft delete + timpa [DOK: DD §0.6 A]
