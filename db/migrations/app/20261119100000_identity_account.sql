-- db/migrations/app/20261119100000_identity_account.sql
-- F1-T16: tabel A1 identitas dan akun (account, rs_admin_role, session_revocation) + hak role.
--
-- Modul kritis T1 (ADR-008, ADR-011, ADR-012, ADR-021 butir 4): review DBA + keamanan.
-- Sumber: docs/04-data-dictionary.md §0.2, §0.3, §0.4 (STD-M, STD-O), §0.6 A, A1.1, A1.2, A1.4;
-- docs/05-physical-schema.md §1, §4.1, §4.3 (templat rs_admin_role), §4.8 (templat
-- session_revocation), §4.11, §11.2. Maju saja (ADR-019); koreksi lewat berkas baru (§11.1).
--
-- Ketentuan:
-- - id tanpa nilai bawaan (UUIDv7 dari aplikasi, ADR-002); created_at/updated_at DEFAULT now()
--   (KEPUTUSAN-93); FK tanpa CASCADE; FK aktor tidak diindeks (§4.1 butir 3-4).
-- - Hak hanya ke role grup (§4.1 butir 7); TRUNCATE tidak diberikan; Master tanpa DELETE (ADR-003).
-- - A1.3 partner_admin_assignment bergantung pada partner (A5) dan tidak termasuk F1-T16.

-- Penjaga (§11.2) dari templat db/test/templat-penjaga-lanjutan-app.sql.
DO $$
BEGIN
  IF current_user <> 'rs_migration' THEN
    RAISE EXCEPTION 'migration must run as rs_migration, got % (session %)', current_user, session_user;
  END IF;
  IF EXISTS (SELECT 1 FROM app._sqlx_migrations WHERE version > 20261119100000) THEN
    RAISE EXCEPTION 'migration order: version 20261119100000 is older than an applied migration (forward-only, ADR-019)';
  END IF;
END $$;
SET LOCAL lock_timeout = '5s';
SET LOCAL statement_timeout = '15min';

-- A1.1 account (M). Kredensial di Keycloak; email salinan untuk notifikasi tanpa unik (KEPUTUSAN-45, -46).
CREATE TABLE app.account (
  id                uuid        NOT NULL,
  created_at        timestamptz NOT NULL DEFAULT now(),
  created_by        uuid        NULL,
  updated_at        timestamptz NOT NULL DEFAULT now(),
  updated_by        uuid        NULL,
  deleted_at        timestamptz NULL,
  deleted_by        uuid        NULL,
  delete_reason     text        NULL,
  account_type      text        NOT NULL,
  idp_realm         text        NOT NULL,
  idp_subject       text        NOT NULL,
  email             text        NOT NULL,
  status            text        NOT NULL,
  status_reason     text        NULL,
  status_changed_at timestamptz NOT NULL,
  full_name         text        NULL,
  last_seen_at      timestamptz NULL,
  anonymized_at     timestamptz NULL,
  CONSTRAINT pk_account PRIMARY KEY (id),
  CONSTRAINT fk_account_created_by FOREIGN KEY (created_by) REFERENCES app.account (id),
  CONSTRAINT fk_account_updated_by FOREIGN KEY (updated_by) REFERENCES app.account (id),
  CONSTRAINT fk_account_deleted_by FOREIGN KEY (deleted_by) REFERENCES app.account (id),
  CONSTRAINT ck_account_soft_delete CHECK (
    (deleted_at IS NULL AND deleted_by IS NULL AND delete_reason IS NULL)
    OR (deleted_at IS NOT NULL AND delete_reason IS NOT NULL)),
  CONSTRAINT ck_account_account_type CHECK (account_type IN
    ('patient','health_worker','rs_admin','partner_admin')),
  CONSTRAINT ck_account_idp_realm CHECK (idp_realm IN ('pasien','nakes','admin')),
  CONSTRAINT ck_account_account_type_idp_realm CHECK (
    (account_type = 'patient' AND idp_realm = 'pasien')
    OR (account_type = 'health_worker' AND idp_realm = 'nakes')
    OR (account_type IN ('rs_admin','partner_admin') AND idp_realm = 'admin')),
  CONSTRAINT ck_account_status CHECK (status IN ('pending_activation','active','inactive')),
  CONSTRAINT ck_account_full_name CHECK (
    account_type NOT IN ('rs_admin','partner_admin') OR full_name IS NOT NULL)
);
CREATE UNIQUE INDEX uqp_account_idp_realm_idp_subject
  ON app.account (idp_realm, idp_subject) WHERE deleted_at IS NULL;   -- [SRC-33]
CREATE INDEX ix_account_status ON app.account (status) WHERE deleted_at IS NULL;
COMMENT ON TABLE app.account IS 'class:M module:identitas';
COMMENT ON COLUMN app.account.email IS 'anonymize';
COMMENT ON COLUMN app.account.full_name IS 'anonymize';

REVOKE ALL ON app.account FROM PUBLIC;
GRANT SELECT, INSERT, UPDATE ON app.account TO rs_app;              -- tanpa DELETE [DOK: ADR-003]
GRANT SELECT ON app.account TO rs_privacy_worker;
GRANT UPDATE (email, full_name, anonymized_at, updated_at, updated_by) ON app.account TO rs_privacy_worker;

-- A1.2 rs_admin_role (M): templat §4.3 persis.
CREATE TABLE app.rs_admin_role (
  id            uuid        NOT NULL,
  created_at    timestamptz NOT NULL DEFAULT now(),
  created_by    uuid        NULL,
  updated_at    timestamptz NOT NULL DEFAULT now(),
  updated_by    uuid        NULL,
  deleted_at    timestamptz NULL,
  deleted_by    uuid        NULL,
  delete_reason text        NULL,
  account_id    uuid        NOT NULL,
  sub_role      text        NOT NULL,
  CONSTRAINT pk_rs_admin_role PRIMARY KEY (id),
  CONSTRAINT fk_rs_admin_role_created_by FOREIGN KEY (created_by) REFERENCES app.account (id),
  CONSTRAINT fk_rs_admin_role_updated_by FOREIGN KEY (updated_by) REFERENCES app.account (id),
  CONSTRAINT fk_rs_admin_role_deleted_by FOREIGN KEY (deleted_by) REFERENCES app.account (id),
  CONSTRAINT fk_rs_admin_role_account_id FOREIGN KEY (account_id) REFERENCES app.account (id),
  CONSTRAINT ck_rs_admin_role_soft_delete CHECK (
    (deleted_at IS NULL AND deleted_by IS NULL AND delete_reason IS NULL)
    OR (deleted_at IS NOT NULL AND delete_reason IS NOT NULL)),
  CONSTRAINT ck_rs_admin_role_sub_role CHECK (sub_role IN
    ('super_admin','verifier','customer_service','finance','content','auditor','privacy_officer'))
);
CREATE UNIQUE INDEX uqp_rs_admin_role_account_id_sub_role
  ON app.rs_admin_role (account_id, sub_role) WHERE deleted_at IS NULL;   -- [SRC-33]
CREATE INDEX ix_rs_admin_role_sub_role ON app.rs_admin_role (sub_role) WHERE deleted_at IS NULL;
COMMENT ON TABLE app.rs_admin_role IS 'class:M module:identitas';

REVOKE ALL ON app.rs_admin_role FROM PUBLIC;
GRANT SELECT, INSERT, UPDATE ON app.rs_admin_role TO rs_app;        -- tanpa DELETE [DOK: ADR-003]
GRANT SELECT ON app.rs_admin_role TO rs_privacy_worker;
-- tabel ini tanpa kolom "Anonimkan", sehingga privacy_worker tidak mendapat UPDATE per kolom

-- A1.4 session_revocation (O): templat §4.8 persis. Tanpa data pribadi selain ID akun, sehingga
-- pekerja privasi tidak diberi hak (DD §0.6 A: S, D hanya untuk tabel O berisi data pribadi).
CREATE TABLE app.session_revocation (
  id             uuid        NOT NULL,
  account_id     uuid        NOT NULL,
  idp_session_id text        NOT NULL,
  reason_code    text        NOT NULL,
  created_at     timestamptz NOT NULL DEFAULT now(),
  expires_at     timestamptz NOT NULL,
  CONSTRAINT pk_session_revocation PRIMARY KEY (id),
  CONSTRAINT fk_session_revocation_account_id FOREIGN KEY (account_id) REFERENCES app.account (id),
  CONSTRAINT uq_session_revocation_idp_session_id UNIQUE (idp_session_id),
  CONSTRAINT ck_session_revocation_reason_code CHECK (reason_code IN
    ('logout','admin_revoke','account_deactivated','role_changed'))
);
CREATE INDEX ix_session_revocation_expires_at ON app.session_revocation (expires_at);
COMMENT ON TABLE app.session_revocation IS 'class:O module:identitas';

REVOKE ALL ON app.session_revocation FROM PUBLIC;
GRANT SELECT, INSERT, UPDATE, DELETE ON app.session_revocation TO rs_app;
