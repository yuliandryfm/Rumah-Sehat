-- db/test/kelas-audit.sql: test katalog kelas A dan trigger penolak database audit (T-06, T-08,
-- T-09 bagian audit, T-18) atas skema audit nyata dan atas tabel fixture, untuk
-- ruang_sehat_audit dan rme_audit. Didahului db/test/lib.sql. Variabel psql: prefiks (rs_|rme_).
-- Fixture di skema uji_kelas (milik pemilik skema audit) dalam satu transaksi yang di-ROLLBACK.
-- Dasar: docs/05-physical-schema.md §4.11, §5.6, §6.1, §6.2, §12; DD §0.6 B; ADR-005, ADR-009 butir 4.

SELECT pg_catalog.set_config('uji.prefiks', :'prefiks', false) AS _p \gset
SELECT :'prefiks' || 'audit_migration' AS pemilik, :'prefiks' || 'audit_writer_login' AS penulis,
       :'prefiks' || 'audit_verifier_login' AS pemverifikasi \gset

-- A. Skema audit nyata.
DO $$
DECLARE
  n integer;
BEGIN
  SELECT count(*) INTO n FROM pg_class c JOIN pg_namespace s ON s.oid = c.relnamespace
   WHERE s.nspname = 'audit' AND c.relkind IN ('r', 'p') AND c.relname <> '_sqlx_migrations';
  PERFORM pg_temp.uji_nol(format('T-06/T-08/T-18 (%s) %s tabel/partisi di skema audit sesuai aturan kelas A', current_database(), n),
    ARRAY(SELECT pg_temp.uji_kelas_audit('audit', current_setting('uji.prefiks'))));
  IF EXISTS (SELECT 1 FROM pg_class c JOIN pg_namespace s ON s.oid = c.relnamespace
              WHERE s.nspname = 'audit_detached' AND c.relacl IS NOT NULL
                AND EXISTS (SELECT 1 FROM aclexplode(c.relacl) a WHERE a.grantee <> c.relowner)) THEN
    RAISE EXCEPTION 'GAGAL §6.5 audit_detached: tabel lepas punya hak selain pemilik';
  END IF;
  RAISE NOTICE 'LULUS §6.5 (%) audit_detached tanpa hak selain pemilik', current_database();
END $$;

-- B. Fixture: tabel induk terpartisi + dua partisi (§6.1, §6.2) dan tabel biasa kelas A.
BEGIN;
CREATE SCHEMA uji_kelas AUTHORIZATION :"pemilik";
DO $$
DECLARE
  p text := current_setting('uji.prefiks');
BEGIN
  EXECUTE format('GRANT USAGE ON SCHEMA uji_kelas TO %I, %I, %I', p || 'audit_writer', p || 'audit_reader', p || 'audit_verifier');
  IF p = 'rme_' THEN
    GRANT USAGE ON SCHEMA uji_kelas TO rme_audit_tenant_reader;
  END IF;
END $$;
SET LOCAL ROLE :"pemilik";
CREATE TABLE uji_kelas.audit_event (
  event_id uuid NOT NULL, recorded_at timestamptz NOT NULL, seq bigint NOT NULL, isi text NULL,
  CONSTRAINT pk_audit_event PRIMARY KEY (event_id, recorded_at)
) PARTITION BY RANGE (recorded_at);
CREATE INDEX ix_audit_event_recorded_at ON uji_kelas.audit_event (recorded_at DESC);
COMMENT ON TABLE uji_kelas.audit_event IS 'class:A module:audit';
CREATE TRIGGER tg_audit_event_reject_ud_row BEFORE UPDATE OR DELETE ON uji_kelas.audit_event
  FOR EACH ROW EXECUTE FUNCTION util.fn_reject_modification();
CREATE TRIGGER tg_audit_event_reject_ud BEFORE UPDATE OR DELETE ON uji_kelas.audit_event
  FOR EACH STATEMENT EXECUTE FUNCTION util.fn_reject_modification();
CREATE TRIGGER tg_audit_event_reject_truncate BEFORE TRUNCATE ON uji_kelas.audit_event
  FOR EACH STATEMENT EXECUTE FUNCTION util.fn_reject_modification();
DO $$
DECLARE
  m date;
  nm text;
BEGIN
  FOR m IN SELECT generate_series(date '2026-11-01', date '2026-12-01', interval '1 month')::date LOOP
    nm := 'audit_event_' || to_char(m, '"y"YYYY"m"MM');
    EXECUTE format('CREATE TABLE uji_kelas.%I PARTITION OF uji_kelas.audit_event FOR VALUES FROM (%L) TO (%L)',
      nm, to_char(m, 'YYYY-MM-DD') || ' 00:00:00+00', to_char(m + interval '1 month', 'YYYY-MM-DD') || ' 00:00:00+00');
    EXECUTE format('CREATE TRIGGER %I BEFORE UPDATE OR DELETE ON uji_kelas.%I FOR EACH STATEMENT EXECUTE FUNCTION util.fn_reject_modification()',
      'tg_' || nm || '_reject_ud', nm);
    EXECUTE format('CREATE TRIGGER %I BEFORE TRUNCATE ON uji_kelas.%I FOR EACH STATEMENT EXECUTE FUNCTION util.fn_reject_modification()',
      'tg_' || nm || '_reject_truncate', nm);
  END LOOP;
END $$;
CREATE TABLE uji_kelas.audit_chain_verification (
  id uuid NOT NULL, created_at timestamptz NOT NULL DEFAULT now(), hasil text NOT NULL,
  CONSTRAINT pk_audit_chain_verification PRIMARY KEY (id));
COMMENT ON TABLE uji_kelas.audit_chain_verification IS 'class:A module:audit';
CREATE TRIGGER tg_audit_chain_verification_reject_ud BEFORE UPDATE OR DELETE ON uji_kelas.audit_chain_verification
  FOR EACH STATEMENT EXECUTE FUNCTION util.fn_reject_modification();
CREATE TRIGGER tg_audit_chain_verification_reject_truncate BEFORE TRUNCATE ON uji_kelas.audit_chain_verification
  FOR EACH STATEMENT EXECUTE FUNCTION util.fn_reject_modification();
DO $$
DECLARE
  p text := current_setting('uji.prefiks');
BEGIN
  EXECUTE format('GRANT INSERT ON uji_kelas.audit_event TO %I', p || 'audit_writer');
  EXECUTE format('GRANT SELECT ON uji_kelas.audit_event, uji_kelas.audit_chain_verification TO %I, %I',
                 p || 'audit_reader', p || 'audit_verifier');
  EXECUTE format('GRANT INSERT ON uji_kelas.audit_chain_verification TO %I', p || 'audit_verifier');
  IF p = 'rme_' THEN
    GRANT SELECT ON uji_kelas.audit_event TO rme_audit_tenant_reader;
  END IF;
END $$;
RESET ROLE;

SELECT pg_temp.uji_nol('T-06/T-08/T-18 fixture kelas A (induk terpartisi, 2 partisi, tabel biasa) lulus pemeriksa',
  ARRAY(SELECT pg_temp.uji_kelas_audit('uji_kelas', current_setting('uji.prefiks'))));
SELECT pg_temp.uji_nol('T-01 fixture audit: nama objek (termasuk partisi _yYYYYmMM)', ARRAY(SELECT pg_temp.uji_t01('{uji_kelas}')));
SELECT pg_temp.uji_nol('T-04 fixture audit: objek milik pemilik skema audit', ARRAY(SELECT pg_temp.uji_t04('{uji_kelas}')));

-- C. Test negatif atas pemeriksa kelas A.
DO $$
DECLARE
  c record;
  p text := current_setting('uji.prefiks');
  kelas text := format('SELECT pg_temp.uji_kelas_audit(''uji_kelas'', %L)', current_setting('uji.prefiks'));
BEGIN
  FOR c IN SELECT * FROM (VALUES
      ('T-06 UPDATE audit_event untuk writer', format('GRANT UPDATE ON uji_kelas.audit_event TO %I', p || 'audit_writer'), 'T-06 audit_event: hak tabel lebih ' || p || 'audit_writer:UPDATE'),
      ('T-06 SELECT audit_event untuk writer (baca kepala rantai hanya lewat fn_chain_head)', format('GRANT SELECT ON uji_kelas.audit_event TO %I', p || 'audit_writer'), 'lebih ' || p || 'audit_writer:SELECT'),
      ('T-06 INSERT audit_event untuk verifier', format('GRANT INSERT ON uji_kelas.audit_event TO %I', p || 'audit_verifier'), 'lebih ' || p || 'audit_verifier:INSERT'),
      ('T-06 hak langsung di partisi', format('GRANT SELECT ON uji_kelas.audit_event_y2026m11 TO %I', p || 'audit_reader'), 'T-06 audit_event_y2026m11: hak tabel lebih'),
      ('T-06 tabel audit tanpa aturan hak', 'CREATE TABLE uji_kelas.audit_liar (x int); COMMENT ON TABLE uji_kelas.audit_liar IS ''class:A module:audit''', 'audit_liar: tabel audit tanpa aturan hak'),
      ('T-08 penolak statement partisi hilang', 'DROP TRIGGER tg_audit_event_y2026m12_reject_ud ON uji_kelas.audit_event_y2026m12', 'T-08 audit_event_y2026m12'),
      ('T-08 penolak baris induk hilang', 'DROP TRIGGER tg_audit_event_reject_ud_row ON uji_kelas.audit_event', 'T-08 audit_event: trigger'),
      ('T-08 penolak truncate nonaktif', 'ALTER TABLE uji_kelas.audit_chain_verification DISABLE TRIGGER tg_audit_chain_verification_reject_truncate', 'NONAKTIF'),
      ('T-18 index induk tidak ada di partisi', 'CREATE INDEX ix_audit_event_isi ON ONLY uji_kelas.audit_event (isi)', 'T-18 audit_event_y2026m1[12]: index induk')
    ) AS t(label, stmt, pola)
  LOOP
    PERFORM pg_temp.uji_rusak(c.label, c.stmt, kelas, c.pola);
  END LOOP;
END $$;

-- D. Perilaku (T-09 bagian audit, ADR-009 butir 4). Data sintetis.
SELECT pg_temp.uji_boleh('T-09 audit writer INSERT audit_event lewat induk', :'penulis',
  'INSERT INTO uji_kelas.audit_event (event_id, recorded_at, seq) VALUES (''00000000-0000-7000-8000-000000000101'', ''2026-11-20 00:00:00+00'', 1)', 1);
SELECT pg_temp.uji_tolak('T-09 NEG audit writer SELECT audit_event', :'penulis', 'SELECT * FROM uji_kelas.audit_event', '42501', 'permission denied for table audit_event');
SELECT pg_temp.uji_tolak('T-09 NEG audit writer UPDATE audit_event', :'penulis', 'UPDATE uji_kelas.audit_event SET isi = ''x''', '42501', 'permission denied for table audit_event');
SELECT pg_temp.uji_tolak('T-09 NEG audit writer DELETE audit_event', :'penulis', 'DELETE FROM uji_kelas.audit_event', '42501', 'permission denied for table audit_event');
SELECT pg_temp.uji_tolak('T-09 NEG audit verifier INSERT audit_event', :'pemverifikasi',
  'INSERT INTO uji_kelas.audit_event (event_id, recorded_at, seq) VALUES (''00000000-0000-7000-8000-000000000102'', ''2026-11-20 00:00:00+00'', 2)', '42501', 'permission denied for table audit_event');
SELECT pg_temp.uji_tolak('T-09 NEG pemilik UPDATE audit_event ditolak trigger', :'pemilik', 'UPDATE uji_kelas.audit_event SET isi = ''x''', '42501', '^immutable_violation: UPDATE on uji_kelas.audit_event rejected$');
SELECT pg_temp.uji_tolak('T-09 NEG pemilik DELETE audit_event nol baris tetap ditolak', :'pemilik', 'DELETE FROM uji_kelas.audit_event WHERE false', '42501', '^immutable_violation: DELETE on uji_kelas.audit_event rejected$');
SELECT pg_temp.uji_tolak('T-09 NEG pemilik TRUNCATE audit_event ditolak trigger', :'pemilik', 'TRUNCATE uji_kelas.audit_event', '42501', '^immutable_violation: TRUNCATE on uji_kelas.audit_event rejected$');
SELECT pg_temp.uji_tolak('T-09 NEG pemilik UPDATE langsung ke partisi ditolak', :'pemilik', 'UPDATE uji_kelas.audit_event_y2026m11 SET isi = ''x''', '42501', '^immutable_violation: UPDATE on uji_kelas.audit_event_y2026m11 rejected$');
SELECT pg_temp.uji_tolak('T-09 NEG pemilik TRUNCATE langsung ke partisi ditolak', :'pemilik', 'TRUNCATE uji_kelas.audit_event_y2026m11', '42501', '^immutable_violation: TRUNCATE on uji_kelas.audit_event_y2026m11 rejected$');
SELECT pg_temp.uji_tolak('T-09 NEG penolak baris induk (klon) tetap menolak UPDATE partisi walau trigger statement partisi hilang (review N09)', :'pemilik',
  'DROP TRIGGER tg_audit_event_y2026m11_reject_ud ON uji_kelas.audit_event_y2026m11; UPDATE uji_kelas.audit_event_y2026m11 SET isi = ''x''',
  '42501', '^immutable_violation: UPDATE on uji_kelas.audit_event_y2026m11 rejected$');
SELECT pg_temp.uji_tolak('T-09 NEG audit reader UPDATE audit_chain_verification', :'prefiks' || 'audit_reader', 'UPDATE uji_kelas.audit_chain_verification SET hasil = ''x''', '42501', 'permission denied for table audit_chain_verification');
SELECT pg_temp.uji_boleh('T-09 verifier INSERT audit_chain_verification', :'pemverifikasi',
  'INSERT INTO uji_kelas.audit_chain_verification (id, hasil) VALUES (''00000000-0000-7000-8000-000000000111'', ''match'')', 1);
SELECT pg_temp.uji_tolak('T-09 NEG verifier UPDATE hasil verifikasi', :'pemverifikasi', 'UPDATE uji_kelas.audit_chain_verification SET hasil = ''mismatch''', '42501', 'permission denied for table audit_chain_verification');
SELECT pg_temp.uji_tolak('T-09 NEG pemilik DELETE hasil verifikasi ditolak trigger', :'pemilik', 'DELETE FROM uji_kelas.audit_chain_verification', '42501', '^immutable_violation: DELETE on uji_kelas.audit_chain_verification rejected$');
ROLLBACK;

DO $$
BEGIN
  IF to_regnamespace('uji_kelas') IS NOT NULL THEN
    RAISE EXCEPTION 'GAGAL pembersihan fixture kelas-audit';
  END IF;
  RAISE NOTICE 'LULUS pembersihan: fixture uji_kelas % tidak tersisa (ROLLBACK)', current_database();
END $$;
