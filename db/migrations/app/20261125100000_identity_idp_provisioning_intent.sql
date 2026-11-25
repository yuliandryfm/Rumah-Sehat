-- db/migrations/app/20261125100000_identity_idp_provisioning_intent.sql
-- ADR-030 langkah 1 (CR-026, KEPUTUSAN-302, KEPUTUSAN-305): keadaan akun menunggu penyediaan identitas dan tabel
-- niat penyediaan identitas beserta riwayat status dan proyeksinya, untuk aksi Keycloak ber-`manage-users` yang
-- hanya dijalankan proses `worker`.
--
-- Modul kritis T1 (autentikasi; file migrasi skema dan hak role database; ADR-003, ADR-004, ADR-008, ADR-019,
-- ADR-021 butir 4, ADR-030): review DBA + keamanan.
-- Sumber: ADR-030 butir 2, 3, 5, 6 dan "Perubahan implementasi" langkah 1; docs/04-data-dictionary.md §0.2-§0.6,
-- A1.1, A1.5-A1.7 (baru, task ini); docs/05-physical-schema.md §4.1, §4.5-§4.7, §11.2, §11.3. Maju saja (ADR-019);
-- koreksi lewat berkas baru (§11.1).
--
-- Ketentuan:
-- - A1.1 account (expand §11.3): status `pending_provisioning` ditambahkan (CHECK hanya menambah nilai) dan
--   `idp_subject` boleh kosong HANYA pada status itu (ck_account_idp_subject). Kode versi sebelumnya tidak pernah
--   menulis status baru, sehingga tetap kompatibel. Keunikan (idp_realm, idp_subject) tidak berubah; NULL tidak
--   saling bentrok di index unik.
-- - ix_account_idp_realm_lower_email: pemeriksaan "akun belum terhapus dengan email yang sama di realm itu" yang
--   dilakukan api setelah kunci advisory transaksi per (realm, email ternormalisasi) (ADR-030 butir 5, butir 7).
--   Bukan constraint unik: keunikan email tetap di Keycloak per realm (KEPUTUSAN-46).
-- - idp_provisioning_intent (T): satu baris per niat, tidak pernah diubah. Muatan job hanya ID niat (ADR-030
--   butir 2); niat sendiri tanpa email, nama, password, atau data pribadi lain: email dibaca worker dari
--   app.account saat dibutuhkan. Pasangan operasi/tujuan dan tujuan per realm dibatasi CHECK sebagai lapis
--   kedua di belakang enum tertutup di kode (ADR-030 butir 3). origin_intent_id (niat pembuat pengguna yang
--   ID-nya disimpan di atribut Keycloak `rs_intent_id`) wajib untuk operasi yang memeriksa atribut itu, dan FK
--   komposit (origin_intent_id, account_id) memastikan niat asal milik akun yang sama (pola KEPUTUSAN-142).
-- - idp_provisioning_intent_status_event (R) + idp_provisioning_intent_state (P): jalur tulis status ADR-008
--   butir 2-3 (crate status-flow), status pending → succeeded | rejected | failed. reason_code wajib untuk
--   rejected/failed, berupa kode (bukan teks bebas).
-- - Hak: role grup rs_app sesuai kelas (T, R: S+I; P: S+I+U), tanpa role terpisah worker IdP (KEPUTUSAN-305:
--   status hasil di database aplikasi bukan batas keamanan karena prasyarat diperiksa terhadap keadaan
--   Keycloak, ADR-030 butir 3). Pekerja privasi hanya ID/FK subjek/created_at di tabel T (DD §0.6 A); R tanpa
--   kolom catatan sehingga tanpa hak per kolom.
-- - Kunci advisory transaksi per (realm, email ternormalisasi) tidak memerlukan objek database: diambil kode
--   aplikasi dengan pg_advisory_xact_lock(101, <int4 dari SHA-256>) (skema fisik §6.4 ruang kunci dua-int4,
--   KEPUTUSAN-305).

-- Penjaga (§11.2) dari templat db/test/templat-penjaga-lanjutan-app.sql.
DO $$
BEGIN
  IF current_user <> 'rs_migration' THEN
    RAISE EXCEPTION 'migration must run as rs_migration, got % (session %)', current_user, session_user;
  END IF;
  IF EXISTS (SELECT 1 FROM app._sqlx_migrations WHERE version > 20261125100000) THEN
    RAISE EXCEPTION 'migration order: version 20261125100000 is older than an applied migration (forward-only, ADR-019)';
  END IF;
END $$;
SET LOCAL lock_timeout = '5s';
SET LOCAL statement_timeout = '15min';

-- A1.1 account (expand): status pending_provisioning; idp_subject kosong hanya pada status itu (ADR-030 butir 5).
ALTER TABLE app.account
  DROP CONSTRAINT ck_account_status,
  ADD CONSTRAINT ck_account_status CHECK (status IN
    ('pending_provisioning','pending_activation','active','inactive')),
  ALTER COLUMN idp_subject DROP NOT NULL,
  ADD CONSTRAINT ck_account_idp_subject CHECK (idp_subject IS NOT NULL OR status = 'pending_provisioning');
CREATE INDEX ix_account_idp_realm_lower_email
  ON app.account (idp_realm, lower(email)) WHERE deleted_at IS NULL;
-- Hak account tidak berubah (F1-T16): rs_app S, I, U; pekerja privasi S + U per kolom anonymize.

-- A1.5 idp_provisioning_intent (T): niat penyediaan identitas (ADR-030 butir 2, 3).
CREATE TABLE app.idp_provisioning_intent (
  id               uuid        NOT NULL,
  created_at       timestamptz NOT NULL DEFAULT now(),
  created_by       uuid        NULL,
  account_id       uuid        NOT NULL,
  idp_realm        text        NOT NULL,
  operation        text        NOT NULL,
  purpose          text        NOT NULL,
  origin_intent_id uuid        NULL,
  correlation_id   uuid        NOT NULL,
  CONSTRAINT pk_idp_provisioning_intent PRIMARY KEY (id),
  CONSTRAINT fk_idp_provisioning_intent_created_by FOREIGN KEY (created_by) REFERENCES app.account (id),
  CONSTRAINT fk_idp_provisioning_intent_account_id FOREIGN KEY (account_id) REFERENCES app.account (id),
  CONSTRAINT uq_idp_provisioning_intent_id_account_id UNIQUE (id, account_id),   -- sasaran FK niat asal
  CONSTRAINT fk_idp_intent_origin_intent_id_account_id FOREIGN KEY (origin_intent_id, account_id)
    REFERENCES app.idp_provisioning_intent (id, account_id),
  CONSTRAINT ck_idp_provisioning_intent_idp_realm CHECK (idp_realm IN ('pasien','nakes','admin','korporat')),
  CONSTRAINT ck_idp_provisioning_intent_operation CHECK (operation IN
    ('create_user','send_activation','disable_user','logout_user','delete_orphan','reconcile')),
  CONSTRAINT ck_idp_provisioning_intent_purpose CHECK (
    (operation IN ('create_user','send_activation') AND purpose IN ('hr_self_registration','hr_invitation'))
    OR (operation = 'disable_user' AND purpose = 'account_deactivation')
    OR (operation = 'logout_user' AND purpose = 'session_revocation')
    OR (operation = 'delete_orphan' AND purpose = 'provisioning_compensation')
    OR (operation = 'reconcile' AND purpose = 'reconciliation')),
  CONSTRAINT ck_idp_provisioning_intent_purpose_realm CHECK (
    purpose NOT IN ('hr_self_registration','hr_invitation') OR idp_realm = 'korporat'),
  CONSTRAINT ck_idp_provisioning_intent_origin CHECK (
    (operation IN ('send_activation','delete_orphan','reconcile')) = (origin_intent_id IS NOT NULL))
);
CREATE INDEX ix_idp_provisioning_intent_account_id_created_at
  ON app.idp_provisioning_intent (account_id, created_at);
COMMENT ON TABLE app.idp_provisioning_intent IS 'class:T module:identitas alias:idp_intent';

REVOKE ALL ON app.idp_provisioning_intent FROM PUBLIC;
GRANT SELECT, INSERT ON app.idp_provisioning_intent TO rs_app;
GRANT SELECT (id, created_at, account_id) ON app.idp_provisioning_intent TO rs_privacy_worker;

CREATE TRIGGER tg_idp_provisioning_intent_reject_ud
  BEFORE UPDATE OR DELETE ON app.idp_provisioning_intent
  FOR EACH STATEMENT EXECUTE FUNCTION util.fn_reject_modification();
CREATE TRIGGER tg_idp_provisioning_intent_reject_truncate
  BEFORE TRUNCATE ON app.idp_provisioning_intent
  FOR EACH STATEMENT EXECUTE FUNCTION util.fn_reject_modification();

-- A1.6 idp_provisioning_intent_status_event (R): STD-R tanpa catatan; hasil pertama yang tersimpan berlaku lewat
-- UQ (niat, versi) (AB-36), sehingga hasil tidak tercatat dua kali bila sewa job hilang (ADR-030 butir 6).
CREATE TABLE app.idp_provisioning_intent_status_event (
  id                         uuid        NOT NULL,
  idp_provisioning_intent_id uuid        NOT NULL,
  version                    integer     NOT NULL,
  from_status                text        NULL,
  to_status                  text        NOT NULL,
  reason_code                text        NULL,
  actor_type                 text        NOT NULL,
  actor_account_id           uuid        NULL,
  correlation_id             uuid        NOT NULL,
  created_at                 timestamptz NOT NULL DEFAULT now(),
  CONSTRAINT pk_idp_provisioning_intent_status_event PRIMARY KEY (id),
  CONSTRAINT fk_idp_intent_event_idp_provisioning_intent_id
    FOREIGN KEY (idp_provisioning_intent_id) REFERENCES app.idp_provisioning_intent (id),
  CONSTRAINT fk_idp_intent_event_actor_account_id FOREIGN KEY (actor_account_id) REFERENCES app.account (id),
  CONSTRAINT uq_idp_intent_event_intent_id_version
    UNIQUE (idp_provisioning_intent_id, version),                                   -- AB-36
  CONSTRAINT uq_idp_intent_event_intent_id_version_to_status
    UNIQUE (idp_provisioning_intent_id, version, to_status),                        -- sasaran FK proyeksi (§4.7)
  CONSTRAINT ck_idp_intent_event_version CHECK (version >= 1),
  CONSTRAINT ck_idp_intent_event_from_status CHECK ((version = 1) = (from_status IS NULL)),
  CONSTRAINT ck_idp_intent_event_to_status CHECK (to_status IN
    ('pending','succeeded','rejected','failed')),
  CONSTRAINT ck_idp_intent_event_reason_code CHECK (
    (to_status IN ('rejected','failed')) = (reason_code IS NOT NULL)
    AND (reason_code IS NULL OR reason_code ~ '^[a-z][a-z0-9_]{0,63}$')),
  CONSTRAINT ck_idp_intent_event_actor_type CHECK (actor_type IN
    ('patient','health_worker','rs_admin','partner_admin','corporate_admin','system'))
);
COMMENT ON TABLE app.idp_provisioning_intent_status_event IS 'class:R module:identitas alias:idp_intent_event';

REVOKE ALL ON app.idp_provisioning_intent_status_event FROM PUBLIC;
GRANT SELECT, INSERT ON app.idp_provisioning_intent_status_event TO rs_app;

CREATE TRIGGER tg_idp_intent_event_reject_ud
  BEFORE UPDATE OR DELETE ON app.idp_provisioning_intent_status_event
  FOR EACH STATEMENT EXECUTE FUNCTION util.fn_reject_modification();
CREATE TRIGGER tg_idp_intent_event_reject_truncate
  BEFORE TRUNCATE ON app.idp_provisioning_intent_status_event
  FOR EACH STATEMENT EXECUTE FUNCTION util.fn_reject_modification();

-- A1.7 idp_provisioning_intent_state (P): STD-P + FK komposit ke riwayat (KEPUTUSAN-142).
CREATE TABLE app.idp_provisioning_intent_state (
  idp_provisioning_intent_id uuid        NOT NULL,
  status                     text        NOT NULL,
  version                    integer     NOT NULL,
  updated_at                 timestamptz NOT NULL DEFAULT now(),
  CONSTRAINT pk_idp_provisioning_intent_state PRIMARY KEY (idp_provisioning_intent_id),
  CONSTRAINT fk_idp_intent_state_idp_provisioning_intent_id
    FOREIGN KEY (idp_provisioning_intent_id) REFERENCES app.idp_provisioning_intent (id),
  CONSTRAINT fk_idp_intent_state_intent_id_version_status
    FOREIGN KEY (idp_provisioning_intent_id, version, status)
    REFERENCES app.idp_provisioning_intent_status_event (idp_provisioning_intent_id, version, to_status),
  CONSTRAINT ck_idp_intent_state_status CHECK (status IN ('pending','succeeded','rejected','failed')),
  CONSTRAINT ck_idp_intent_state_version CHECK (version >= 1)
);
-- Niat tertahan di status pending (ADR-030 butir 9 (c), rekonsiliasi langkah 8).
CREATE INDEX ix_idp_intent_state_status_updated_at_pending
  ON app.idp_provisioning_intent_state (status, updated_at) WHERE status = 'pending';
COMMENT ON TABLE app.idp_provisioning_intent_state IS 'class:P module:identitas alias:idp_intent_state';

REVOKE ALL ON app.idp_provisioning_intent_state FROM PUBLIC;
GRANT SELECT, INSERT, UPDATE ON app.idp_provisioning_intent_state TO rs_app;   -- tanpa DELETE [DOK: ADR-008]
