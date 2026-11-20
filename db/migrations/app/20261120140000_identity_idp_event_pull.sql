-- db/migrations/app/20261120140000_identity_idp_event_pull.sql
-- F1-T34: posisi baca dan tanda terima pengambil event Keycloak (E-01-E-04 -> audit_outbox, ADR-011
-- butir 10, ADR-009 butir 2 "menulisnya ke outbox secara idempoten").
--
-- Modul kritis T1 (ADR-008, ADR-009, ADR-011): review DBA + keamanan.
-- Kelas Operasional (ADR-008: "kursor sinkronisasi", "kunci idempotensi"; DD §0.4 STD-O): rs_app S/I/U/D,
-- tanpa trigger kelas. Kedua tabel BELUM ada di docs/04-data-dictionary.md: rancangan task ini, wajib
-- ditambahkan ke DD (A1) oleh orkestrator/DBA bila disetujui.
-- Tidak memuat data pribadi: hanya nama realm, ID event Keycloak (UUID acak), dan waktu event.
-- Sumber: DD §0.4 (STD-O), §0.6 A, A1.1 (CHECK idp_realm); docs/05-physical-schema.md §4.1, §6.4 (kunci
-- 7001002), §11.2 (penjaga). Maju saja (ADR-019).

-- Penjaga (§11.2) dari templat db/test/templat-penjaga-lanjutan-app.sql.
DO $$
BEGIN
  IF current_user <> 'rs_migration' THEN
    RAISE EXCEPTION 'migration must run as rs_migration, got % (session %)', current_user, session_user;
  END IF;
  IF EXISTS (SELECT 1 FROM app._sqlx_migrations WHERE version > 20261120140000) THEN
    RAISE EXCEPTION 'migration order: version 20261120140000 is older than an applied migration (forward-only, ADR-019)';
  END IF;
END $$;
SET LOCAL lock_timeout = '5s';
SET LOCAL statement_timeout = '15min';

-- Posisi baca per realm: waktu event Keycloak (milidetik epoch) yang sudah diproses, dimajukan hanya oleh
-- pemegang kunci 7001002 dan tidak pernah mundur (GREATEST di kode).
CREATE TABLE app.idp_event_cursor (
  id             uuid        NOT NULL,
  created_at     timestamptz NOT NULL DEFAULT now(),
  idp_realm      text        NOT NULL,
  position_ms    bigint      NOT NULL,
  last_pulled_at timestamptz NULL,
  updated_at     timestamptz NOT NULL DEFAULT now(),
  CONSTRAINT pk_idp_event_cursor PRIMARY KEY (id),
  CONSTRAINT uq_idp_event_cursor_idp_realm UNIQUE (idp_realm),
  CONSTRAINT ck_idp_event_cursor_idp_realm CHECK (idp_realm IN ('pasien','nakes','admin')),
  CONSTRAINT ck_idp_event_cursor_position_ms CHECK (position_ms >= 0)
);
COMMENT ON TABLE app.idp_event_cursor IS 'class:O module:identitas';

REVOKE ALL ON app.idp_event_cursor FROM PUBLIC;
GRANT SELECT, INSERT, UPDATE, DELETE ON app.idp_event_cursor TO rs_app;

-- Tanda terima event yang sudah ditulis ke audit_outbox, ditulis di transaksi yang sama (pustaka audit).
-- UNIQUE (idp_realm, idp_event_id) = pengaman idempotensi: event yang dibaca ulang (jendela tumpang-tindih,
-- restart, atau dua pengambil bersamaan) tidak tercatat dua kali. Baris di luar jendela dihapus pengambil.
CREATE TABLE app.idp_event_receipt (
  id           uuid        NOT NULL,
  created_at   timestamptz NOT NULL DEFAULT now(),
  idp_realm    text        NOT NULL,
  idp_event_id uuid        NOT NULL,
  event_time   timestamptz NOT NULL,
  CONSTRAINT pk_idp_event_receipt PRIMARY KEY (id),
  CONSTRAINT uq_idp_event_receipt_idp_realm_idp_event_id UNIQUE (idp_realm, idp_event_id),
  CONSTRAINT ck_idp_event_receipt_idp_realm CHECK (idp_realm IN ('pasien','nakes','admin'))
);
CREATE INDEX ix_idp_event_receipt_idp_realm_event_time ON app.idp_event_receipt (idp_realm, event_time);
COMMENT ON TABLE app.idp_event_receipt IS 'class:O module:identitas';

REVOKE ALL ON app.idp_event_receipt FROM PUBLIC;
GRANT SELECT, INSERT, UPDATE, DELETE ON app.idp_event_receipt TO rs_app;
