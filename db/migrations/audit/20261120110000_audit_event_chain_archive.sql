-- db/migrations/audit/20261120110000_audit_event_chain_archive.sql
-- F1-T21: tabel database audit B1-B6 (audit_event dan audit_event_subject dipartisi bulanan,
-- audit_event_key, audit_chain_anchor, audit_chain_verification, audit_archive_manifest), partisi
-- ke depan 2026-11 s.d. 2029-12, akses sempit kepala rantai, hak role audit_writer/audit_reader/
-- audit_verifier, dan RLS tenant khusus rme_audit. Satu berkas untuk KEDUA database audit
-- (ruang_sehat_audit dan rme_audit; DD B "skema yang sama", §11.1, KEPUTUSAN-140).
--
-- Modul kritis T1 (pustaka, relay, dan database audit; ADR-005, ADR-008, ADR-009, ADR-012 butir 9,
-- ADR-021 butir 4): review DBA + keamanan.
-- Sumber: docs/04-data-dictionary.md §0.4 (STD-L), §0.6 B, B1-B6; docs/05-physical-schema.md §1.2,
-- §3.2 (b), §4.1, §4.2 (1), §4.11, §5.6, §6.1, §6.2 (templat partisi, disalin persis), §6.3, §6.4,
-- §11.1, §11.2, §11.4, §12 (T-06, T-08, T-18, T-19); KEPUTUSAN-56, KEPUTUSAN-99, KEPUTUSAN-100,
-- KEPUTUSAN-102. Maju saja (ADR-019); koreksi lewat berkas baru (§11.1).
--
-- Ketentuan:
-- - Semua tabel kelas A ('class:A module:audit', §4.1 butir 1): penolak UPDATE/DELETE (statement)
--   dan TRUNCATE di setiap tabel dan setiap partisi; induk terpartisi ditambah penolak tingkat baris
--   yang ikut terpasang di partisi (§4.11, §6.1, review N09).
-- - Tanpa partisi DEFAULT (§6.2 butir 3): INSERT di luar rentang gagal, event tertahan di outbox.
-- - Hak per tabel hanya di induk (§5.6); partisi tanpa hak selain pemilik. Hak hanya ke role grup.
--   GRANT dan RLS ditulis statis per database dalam satu blok bercabang current_database(): GRANT
--   dinamis (EXECUTE format ... TO %I dengan util.fn_audit_role) berada di luar allowlist SQL
--   dinamis TD-01 (a), sehingga pemetaan util.fn_audit_role hanya dipakai sebagai pemeriksa silang.
-- - audit.fn_chain_head (§6.3, KEPUTUSAN-102): satu-satunya SECURITY DEFINER yang diizinkan
--   (search_path persis "pg_catalog, audit"); EXECUTE hanya untuk *_audit_writer.
-- - RLS hanya di rme_audit (§5.6, KEPUTUSAN-99): audit_event dan audit_event_subject ENABLE + FORCE,
--   kebijakan untuk setiap role yang menyentuh tabel (F-P02). ruang_sehat_audit tanpa RLS.
-- - CHECK 32 byte untuk event_hash anchor, boundary_event_hash, dan archive_sha256 mengikuti
--   ck_audit_event_*_hash §6.1 (keluaran SHA-256, ADR-009 butir 5); keputusan teknis F1-T21.
-- - audit.pr_archive_detach / pr_archive_drop_detached (§6.5) TIDAK di berkas ini: badannya memakai
--   SQL dinamis di luar allowlist TD-01 (a) sehingga perlu keputusan perluasan allowlist (dilaporkan).
-- - Hak *_audit_writer persis DD §0.6 B (INSERT saja + EXECUTE fn_chain_head). Jalur relay §6.4
--   butir 2 "ON CONFLICT (event_id) DO NOTHING" membutuhkan SELECT atas event_id (SRC-470) dan karena
--   itu DITOLAK untuk writer (dibuktikan db/test/tabel-audit-b.sql); SELECT per kolom tidak diberikan
--   tanpa keputusan (dilaporkan untuk F1-T24).

-- Penjaga (§11.2) dari templat db/test/templat-penjaga-lanjutan-audit.sql.
DO $$
BEGIN
  IF current_user <> util.fn_audit_role('audit_migration') THEN
    RAISE EXCEPTION 'migration must run as %, got % (session %)', util.fn_audit_role('audit_migration'), current_user, session_user;
  END IF;
  IF EXISTS (SELECT 1 FROM audit._sqlx_migrations WHERE version > 20261120110000) THEN
    RAISE EXCEPTION 'migration order: version 20261120110000 is older than an applied migration (forward-only, ADR-019)';
  END IF;
END $$;
SET LOCAL lock_timeout = '5s';
SET LOCAL statement_timeout = '15min';

-- B1 audit_event (A, dipartisi bulanan per recorded_at UTC; §6.1, KEPUTUSAN-56).
CREATE TABLE audit.audit_event (
  event_id                   uuid        NOT NULL,
  recorded_at                timestamptz NOT NULL,
  seq                        bigint      NOT NULL,
  occurred_at                timestamptz NOT NULL,
  event_code                 text        NOT NULL,
  action_name                text        NOT NULL,
  actor_type                 text        NOT NULL,
  actor_account_id           uuid        NULL,
  actor_account_type         text        NULL,
  actor_roles                text[]      NULL,
  actor_partner_id           uuid        NULL,
  object_type                text        NOT NULL,
  object_id                  text        NULL,
  object_version             integer     NULL,
  subject_id                 uuid        NULL,
  subject_count              integer     NOT NULL,
  tenant_id                  uuid        NULL,
  source_component           text        NOT NULL,
  source_ip                  inet        NULL,
  user_agent                 text        NULL,
  app_version                text        NULL,
  changed_fields             text[]      NULL,
  before_values              jsonb       NULL,
  after_values               jsonb       NULL,
  disclosure_recipient       text        NULL,
  disclosure_data_categories text[]      NULL,
  outcome                    text        NOT NULL,
  outcome_reason_code        text        NULL,
  correlation_id             uuid        NOT NULL,
  canonical_version          smallint    NOT NULL,
  prev_hash                  bytea       NOT NULL,
  event_hash                 bytea       NOT NULL,
  CONSTRAINT pk_audit_event PRIMARY KEY (event_id, recorded_at),
  CONSTRAINT ck_audit_event_event_code CHECK (event_code IN
    ('e_01','e_02','e_03','e_04','e_05','e_06','e_07','e_08','e_09','e_10','e_11','e_12','e_13','e_14',
     'e_15','e_16','e_17')),
  CONSTRAINT ck_audit_event_actor_type CHECK (actor_type IN
    ('account','system','partner_system','ruang_sehat_system')),
  CONSTRAINT ck_audit_event_subject_count CHECK (subject_count >= 0),
  CONSTRAINT ck_audit_event_outcome CHECK (outcome IN ('success','failure')),
  CONSTRAINT ck_audit_event_prev_hash CHECK (octet_length(prev_hash) = 32),
  CONSTRAINT ck_audit_event_event_hash CHECK (octet_length(event_hash) = 32)
) PARTITION BY RANGE (recorded_at);
-- Index DD B1 di induk; partisi mewarisinya (T-18).
CREATE INDEX ix_audit_event_recorded_at ON audit.audit_event (recorded_at DESC);
CREATE INDEX ix_audit_event_actor_account_id_recorded_at ON audit.audit_event (actor_account_id, recorded_at);
CREATE INDEX ix_audit_event_subject_id_recorded_at ON audit.audit_event (subject_id, recorded_at);
CREATE INDEX ix_audit_event_event_code_recorded_at ON audit.audit_event (event_code, recorded_at);
CREATE INDEX ix_audit_event_object_type_object_id ON audit.audit_event (object_type, object_id);
CREATE INDEX ix_audit_event_tenant_id_event_code_recorded_at ON audit.audit_event (tenant_id, event_code, recorded_at);
CREATE INDEX ix_audit_event_source_ip_recorded_at ON audit.audit_event (source_ip, recorded_at);
CREATE INDEX ix_audit_event_correlation_id ON audit.audit_event (correlation_id);
CREATE INDEX ix_audit_event_seq ON audit.audit_event (seq);
COMMENT ON TABLE audit.audit_event IS 'class:A module:audit';
REVOKE ALL ON audit.audit_event FROM PUBLIC;

CREATE TRIGGER tg_audit_event_reject_ud_row
  BEFORE UPDATE OR DELETE ON audit.audit_event
  FOR EACH ROW EXECUTE FUNCTION util.fn_reject_modification();
CREATE TRIGGER tg_audit_event_reject_ud
  BEFORE UPDATE OR DELETE ON audit.audit_event
  FOR EACH STATEMENT EXECUTE FUNCTION util.fn_reject_modification();
CREATE TRIGGER tg_audit_event_reject_truncate
  BEFORE TRUNCATE ON audit.audit_event
  FOR EACH STATEMENT EXECUTE FUNCTION util.fn_reject_modification();

-- B3 audit_event_subject (A, dipartisi bulanan; tanpa FK deklaratif ke audit_event, DD B3).
CREATE TABLE audit.audit_event_subject (
  id           uuid        NOT NULL,
  recorded_at  timestamptz NOT NULL,
  event_id     uuid        NOT NULL,
  subject_id   uuid        NOT NULL,
  subject_kind text        NOT NULL,
  CONSTRAINT pk_audit_event_subject PRIMARY KEY (id, recorded_at),
  CONSTRAINT ck_audit_event_subject_subject_kind CHECK (subject_kind IN ('care_recipient','rme_patient'))
) PARTITION BY RANGE (recorded_at);
CREATE INDEX ix_audit_event_subject_subject_id_recorded_at ON audit.audit_event_subject (subject_id, recorded_at);
CREATE INDEX ix_audit_event_subject_event_id ON audit.audit_event_subject (event_id);
COMMENT ON TABLE audit.audit_event_subject IS 'class:A module:audit';
REVOKE ALL ON audit.audit_event_subject FROM PUBLIC;

CREATE TRIGGER tg_audit_event_subject_reject_ud_row
  BEFORE UPDATE OR DELETE ON audit.audit_event_subject
  FOR EACH ROW EXECUTE FUNCTION util.fn_reject_modification();
CREATE TRIGGER tg_audit_event_subject_reject_ud
  BEFORE UPDATE OR DELETE ON audit.audit_event_subject
  FOR EACH STATEMENT EXECUTE FUNCTION util.fn_reject_modification();
CREATE TRIGGER tg_audit_event_subject_reject_truncate
  BEFORE TRUNCATE ON audit.audit_event_subject
  FOR EACH STATEMENT EXECUTE FUNCTION util.fn_reject_modification();

-- Partisi bulanan 2026-11 s.d. 2029-12 beserta penolak per partisi. Templat docs/05-physical-schema.md
-- §6.2 (KEPUTUSAN-100), disalin persis (allowlist TD-01 (a) R3).
DO $$
DECLARE
  m date;
  p text;
BEGIN
  FOR m IN SELECT generate_series(date '2026-11-01', date '2029-12-01', interval '1 month')::date LOOP
    FOREACH p IN ARRAY ARRAY['audit_event', 'audit_event_subject'] LOOP
      EXECUTE format(
        'CREATE TABLE audit.%I PARTITION OF audit.%I FOR VALUES FROM (%L) TO (%L)',
        p || '_' || to_char(m, '"y"YYYY"m"MM'), p,
        to_char(m, 'YYYY-MM-DD') || ' 00:00:00+00',
        to_char(m + interval '1 month', 'YYYY-MM-DD') || ' 00:00:00+00');
      EXECUTE format('CREATE TRIGGER %I BEFORE UPDATE OR DELETE ON audit.%I FOR EACH STATEMENT EXECUTE FUNCTION util.fn_reject_modification()',
        'tg_' || p || '_' || to_char(m, '"y"YYYY"m"MM') || '_reject_ud', p || '_' || to_char(m, '"y"YYYY"m"MM'));
      EXECUTE format('CREATE TRIGGER %I BEFORE TRUNCATE ON audit.%I FOR EACH STATEMENT EXECUTE FUNCTION util.fn_reject_modification()',
        'tg_' || p || '_' || to_char(m, '"y"YYYY"m"MM') || '_reject_truncate', p || '_' || to_char(m, '"y"YYYY"m"MM'));
    END LOOP;
  END LOOP;
END $$;

-- B2 audit_event_key (A, tidak dipartisi): keunikan global event_id dan seq (§6.1, §6.4 butir 5).
CREATE TABLE audit.audit_event_key (
  event_id    uuid        NOT NULL,
  seq         bigint      NOT NULL,
  recorded_at timestamptz NOT NULL,
  CONSTRAINT pk_audit_event_key PRIMARY KEY (event_id),
  CONSTRAINT uq_audit_event_key_seq UNIQUE (seq),
  CONSTRAINT ck_audit_event_key_seq CHECK (seq >= 1)
);
COMMENT ON TABLE audit.audit_event_key IS 'class:A module:audit';
REVOKE ALL ON audit.audit_event_key FROM PUBLIC;

CREATE TRIGGER tg_audit_event_key_reject_ud
  BEFORE UPDATE OR DELETE ON audit.audit_event_key
  FOR EACH STATEMENT EXECUTE FUNCTION util.fn_reject_modification();
CREATE TRIGGER tg_audit_event_key_reject_truncate
  BEFORE TRUNCATE ON audit.audit_event_key
  FOR EACH STATEMENT EXECUTE FUNCTION util.fn_reject_modification();

-- B4 audit_chain_anchor (A): kepala rantai per jam yang juga ditulis ke WORM (ADR-009 butir 5).
CREATE TABLE audit.audit_chain_anchor (
  id              uuid        NOT NULL,
  created_at      timestamptz NOT NULL DEFAULT now(),
  seq             bigint      NOT NULL,
  event_hash      bytea       NOT NULL,
  anchored_at     timestamptz NOT NULL,
  worm_object_key text        NOT NULL,
  CONSTRAINT pk_audit_chain_anchor PRIMARY KEY (id),
  CONSTRAINT uq_audit_chain_anchor_seq UNIQUE (seq),
  CONSTRAINT ck_audit_chain_anchor_event_hash CHECK (octet_length(event_hash) = 32)
);
COMMENT ON TABLE audit.audit_chain_anchor IS 'class:A module:audit';
REVOKE ALL ON audit.audit_chain_anchor FROM PUBLIC;

CREATE TRIGGER tg_audit_chain_anchor_reject_ud
  BEFORE UPDATE OR DELETE ON audit.audit_chain_anchor
  FOR EACH STATEMENT EXECUTE FUNCTION util.fn_reject_modification();
CREATE TRIGGER tg_audit_chain_anchor_reject_truncate
  BEFORE TRUNCATE ON audit.audit_chain_anchor
  FOR EACH STATEMENT EXECUTE FUNCTION util.fn_reject_modification();

-- B5 audit_chain_verification (A): hasil verifikasi rantai harian oleh audit-verify (§3.3, KEPUTUSAN-141).
CREATE TABLE audit.audit_chain_verification (
  id                    uuid        NOT NULL,
  created_at            timestamptz NOT NULL DEFAULT now(),
  started_at            timestamptz NOT NULL,
  finished_at           timestamptz NOT NULL,
  from_seq              bigint      NOT NULL,
  to_seq                bigint      NOT NULL,
  audit_chain_anchor_id uuid        NULL,
  outcome               text        NOT NULL,
  first_mismatch_seq    bigint      NULL,
  CONSTRAINT pk_audit_chain_verification PRIMARY KEY (id),
  CONSTRAINT fk_audit_chain_verification_audit_chain_anchor_id
    FOREIGN KEY (audit_chain_anchor_id) REFERENCES audit.audit_chain_anchor (id),
  CONSTRAINT ck_audit_chain_verification_outcome CHECK (outcome IN ('match','mismatch','error'))
);
COMMENT ON TABLE audit.audit_chain_verification IS 'class:A module:audit';
REVOKE ALL ON audit.audit_chain_verification FROM PUBLIC;

CREATE TRIGGER tg_audit_chain_verification_reject_ud
  BEFORE UPDATE OR DELETE ON audit.audit_chain_verification
  FOR EACH STATEMENT EXECUTE FUNCTION util.fn_reject_modification();
CREATE TRIGGER tg_audit_chain_verification_reject_truncate
  BEFORE TRUNCATE ON audit.audit_chain_verification
  FOR EACH STATEMENT EXECUTE FUNCTION util.fn_reject_modification();

-- B6 audit_archive_manifest (A): partisi yang diekspor ke WORM lalu di-detach (ADR-009 butir 9).
-- INSERT hanya oleh pemilik skema (DD §0.6 B), lewat prosedur arsip §6.5 (task terpisah).
CREATE TABLE audit.audit_archive_manifest (
  id                  uuid        NOT NULL,
  created_at          timestamptz NOT NULL DEFAULT now(),
  partition_name      text        NOT NULL,
  range_start         timestamptz NOT NULL,
  range_end           timestamptz NOT NULL,
  from_seq            bigint      NOT NULL,
  to_seq              bigint      NOT NULL,
  boundary_event_hash bytea       NOT NULL,
  archive_object_key  text        NOT NULL,
  archive_sha256      bytea       NOT NULL,
  exported_at         timestamptz NOT NULL,
  detached_at         timestamptz NOT NULL,
  CONSTRAINT pk_audit_archive_manifest PRIMARY KEY (id),
  CONSTRAINT uq_audit_archive_manifest_partition_name UNIQUE (partition_name),
  CONSTRAINT ck_audit_archive_manifest_boundary_event_hash CHECK (octet_length(boundary_event_hash) = 32),
  CONSTRAINT ck_audit_archive_manifest_archive_sha256 CHECK (octet_length(archive_sha256) = 32)
);
COMMENT ON TABLE audit.audit_archive_manifest IS 'class:A module:audit';
REVOKE ALL ON audit.audit_archive_manifest FROM PUBLIC;

CREATE TRIGGER tg_audit_archive_manifest_reject_ud
  BEFORE UPDATE OR DELETE ON audit.audit_archive_manifest
  FOR EACH STATEMENT EXECUTE FUNCTION util.fn_reject_modification();
CREATE TRIGGER tg_audit_archive_manifest_reject_truncate
  BEFORE TRUNCATE ON audit.audit_archive_manifest
  FOR EACH STATEMENT EXECUTE FUNCTION util.fn_reject_modification();

-- Akses sempit kepala rantai (§6.3, temuan F-02, KEPUTUSAN-102). Pemilik = pemilik skema audit.
CREATE FUNCTION audit.fn_chain_head()
RETURNS TABLE (seq bigint, event_hash bytea, recorded_at timestamptz)
LANGUAGE sql
STABLE
SECURITY DEFINER
SET search_path = pg_catalog, audit
AS $$
  SELECT e.seq, e.event_hash, e.recorded_at
    FROM audit.audit_event_key k
    JOIN audit.audit_event e ON e.event_id = k.event_id AND e.recorded_at = k.recorded_at
   ORDER BY k.seq DESC
   LIMIT 1
$$;
REVOKE ALL ON FUNCTION audit.fn_chain_head() FROM PUBLIC;
COMMENT ON FUNCTION audit.fn_chain_head() IS
  'Kepala rantai audit (seq, event_hash, recorded_at terakhir) untuk relay; satu-satunya akses baca *_audit_writer (skema fisik §6.3)';

-- Hak (DD §0.6 B, §6.1, §6.3) dan RLS tenant rme_audit (§5.6), statis per database. Pemetaan
-- util.fn_audit_role diperiksa silang supaya cabang dan pemetaan tidak menyimpang.
DO $$
BEGIN
  IF pg_catalog.current_database() = 'ruang_sehat_audit' THEN
    IF util.fn_audit_role('audit_writer') <> 'rs_audit_writer'
       OR util.fn_audit_role('audit_reader') <> 'rs_audit_reader'
       OR util.fn_audit_role('audit_verifier') <> 'rs_audit_verifier' THEN
      RAISE EXCEPTION 'audit role mapping mismatch on %', pg_catalog.current_database();
    END IF;
    GRANT INSERT ON audit.audit_event, audit.audit_event_subject, audit.audit_event_key, audit.audit_chain_anchor
      TO rs_audit_writer;
    GRANT SELECT ON audit.audit_event, audit.audit_event_subject, audit.audit_event_key,
      audit.audit_chain_anchor, audit.audit_chain_verification, audit.audit_archive_manifest
      TO rs_audit_reader, rs_audit_verifier;
    GRANT INSERT ON audit.audit_chain_verification TO rs_audit_verifier;
    GRANT EXECUTE ON FUNCTION audit.fn_chain_head() TO rs_audit_writer;
  ELSIF pg_catalog.current_database() = 'rme_audit' THEN
    IF util.fn_audit_role('audit_writer') <> 'rme_audit_writer'
       OR util.fn_audit_role('audit_reader') <> 'rme_audit_reader'
       OR util.fn_audit_role('audit_verifier') <> 'rme_audit_verifier'
       OR util.fn_audit_role('audit_tenant_reader') <> 'rme_audit_tenant_reader' THEN
      RAISE EXCEPTION 'audit role mapping mismatch on %', pg_catalog.current_database();
    END IF;
    GRANT INSERT ON audit.audit_event, audit.audit_event_subject, audit.audit_event_key, audit.audit_chain_anchor
      TO rme_audit_writer;
    GRANT SELECT ON audit.audit_event, audit.audit_event_subject, audit.audit_event_key,
      audit.audit_chain_anchor, audit.audit_chain_verification, audit.audit_archive_manifest
      TO rme_audit_reader, rme_audit_verifier;
    GRANT INSERT ON audit.audit_chain_verification TO rme_audit_verifier;
    GRANT EXECUTE ON FUNCTION audit.fn_chain_head() TO rme_audit_writer;
    -- RLS tenant (§5.6): induk saja; partisi tidak diberi hak sehingga tidak dapat dibaca langsung.
    GRANT SELECT ON audit.audit_event, audit.audit_event_subject TO rme_audit_tenant_reader;
    ALTER TABLE audit.audit_event ENABLE ROW LEVEL SECURITY;
    ALTER TABLE audit.audit_event FORCE ROW LEVEL SECURITY;
    CREATE POLICY pol_audit_event_tenant_reader ON audit.audit_event
      FOR SELECT TO rme_audit_tenant_reader
      USING (tenant_id = (SELECT util.fn_current_fasyankes_id()) AND event_code = 'e_05');
    CREATE POLICY pol_audit_event_writer ON audit.audit_event
      FOR INSERT TO rme_audit_writer WITH CHECK (true);
    CREATE POLICY pol_audit_event_reader ON audit.audit_event
      FOR SELECT TO rme_audit_reader, rme_audit_verifier USING (true);
    CREATE POLICY pol_audit_event_migration ON audit.audit_event
      FOR ALL TO rme_audit_migration USING (true) WITH CHECK (true);
    ALTER TABLE audit.audit_event_subject ENABLE ROW LEVEL SECURITY;
    ALTER TABLE audit.audit_event_subject FORCE ROW LEVEL SECURITY;
    CREATE POLICY pol_audit_event_subject_tenant_reader ON audit.audit_event_subject
      FOR SELECT TO rme_audit_tenant_reader
      USING (EXISTS (SELECT 1 FROM audit.audit_event e
                      WHERE e.event_id = audit_event_subject.event_id
                        AND e.recorded_at = audit_event_subject.recorded_at));
    CREATE POLICY pol_audit_event_subject_writer ON audit.audit_event_subject
      FOR INSERT TO rme_audit_writer WITH CHECK (true);
    CREATE POLICY pol_audit_event_subject_reader ON audit.audit_event_subject
      FOR SELECT TO rme_audit_reader, rme_audit_verifier USING (true);
    CREATE POLICY pol_audit_event_subject_migration ON audit.audit_event_subject
      FOR ALL TO rme_audit_migration USING (true) WITH CHECK (true);
  ELSE
    RAISE EXCEPTION 'migration order: db/migrations/audit must run on ruang_sehat_audit or rme_audit, got %',
      pg_catalog.current_database();
  END IF;
END $$;
