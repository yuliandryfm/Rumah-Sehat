-- db/test/util-audit.sql: test fungsi bersama skema util database audit (F1-T14), dijalankan di
-- ruang_sehat_audit dan rme_audit. Didahului db/test/lib.sql. Variabel psql: prefiks (rs_|rme_).
-- Dasar: docs/05-physical-schema.md §3.2 (b), §4.2 (1), §5.2, §5.6, §11.1, §11.2; CR-020.

SELECT pg_catalog.set_config('uji.prefiks', :'prefiks', false) AS _p \gset

-- Isi util persis per database.
DO $$
DECLARE
  p text := current_setting('uji.prefiks');
  diff text;
BEGIN
  WITH expected(sig, cfg, lang, vol, grantee) AS (
    SELECT * FROM (VALUES
      ('util.fn_reject_modification()', '{"search_path=pg_catalog, pg_temp"}', 'plpgsql', 'v', NULL),
      ('util.fn_audit_role(text)', '{"search_path=pg_catalog, pg_temp"}', 'plpgsql', 's', NULL),
      ('util.fn_current_fasyankes_id()', NULL, 'sql', 's', 'rme_audit_tenant_reader')) v(sig, cfg, lang, vol, grantee)
     WHERE sig <> 'util.fn_current_fasyankes_id()' OR p = 'rme_'
  ), actual AS (
    SELECT p2.oid::regprocedure::text AS sig, p2.proconfig::text AS cfg, l.lanname::text AS lang, p2.provolatile::text AS vol,
           (SELECT string_agg(pg_get_userbyid(a.grantee), ',') FROM aclexplode(p2.proacl) a WHERE a.grantee <> p2.proowner) AS grantee
      FROM pg_proc p2 JOIN pg_namespace n ON n.oid = p2.pronamespace JOIN pg_language l ON l.oid = p2.prolang
     WHERE n.nspname = 'util' AND NOT p2.prosecdef AND p2.proacl IS NOT NULL
       AND p2.proowner = (p || 'audit_migration')::regrole
  )
  SELECT string_agg(x, '; ') INTO diff FROM (
    SELECT 'kurang/beda: ' || e::text AS x FROM (SELECT * FROM expected EXCEPT SELECT * FROM actual) e
    UNION ALL
    SELECT 'lebih: ' || a::text FROM (SELECT * FROM actual EXCEPT SELECT * FROM expected) a
    UNION ALL
    SELECT 'tidak sesuai syarat: ' || p2.oid::regprocedure::text
      FROM pg_proc p2 JOIN pg_namespace n ON n.oid = p2.pronamespace
     WHERE n.nspname = 'util' AND p2.oid::regprocedure::text NOT IN (SELECT sig FROM actual)) d;
  IF diff IS NOT NULL THEN
    RAISE EXCEPTION 'GAGAL F1-T14 isi util %: %', current_database(), diff;
  END IF;
  RAISE NOTICE 'LULUS F1-T14 util %: fungsi persis (SECURITY INVOKER, search_path tetap kecuali fn_current_fasyankes_id §5.2, milik %audit_migration, EXECUTE sesuai §5.6)',
    current_database(), p;
END $$;

-- USAGE skema util: ruang_sehat_audit tanpa role; rme_audit hanya rme_audit_tenant_reader (§5.6).
DO $$
DECLARE
  p text := current_setting('uji.prefiks');
  aktual text;
  harap text := CASE WHEN p = 'rme_' THEN 'rme_audit_tenant_reader:USAGE' END;
BEGIN
  SELECT string_agg(pg_get_userbyid(a.grantee) || ':' || a.privilege_type, ',' ORDER BY 1) INTO aktual
    FROM pg_namespace n, aclexplode(n.nspacl) a WHERE n.nspname = 'util' AND a.grantee <> n.nspowner;
  IF aktual IS DISTINCT FROM harap THEN
    RAISE EXCEPTION 'GAGAL F1-T14 hak skema util %: % (diharapkan %)', current_database(), aktual, harap;
  END IF;
  RAISE NOTICE 'LULUS F1-T14 hak skema util %: %', current_database(), coalesce(aktual, 'hanya pemilik');
END $$;

-- Tidak ada role lain (termasuk role sistem audit lain dan role aplikasi) yang dapat memakai util.
DO $$
DECLARE
  p text := current_setting('uji.prefiks');
  bad text;
BEGIN
  SELECT string_agg(format('%s:%s', r.rolname, x), ', ') INTO bad
    FROM pg_roles r,
         LATERAL (SELECT 'USAGE util' AS x WHERE has_schema_privilege(r.oid, 'util', 'USAGE')
                  UNION ALL
                  SELECT 'EXECUTE ' || f.oid::regprocedure::text
                    FROM pg_proc f JOIN pg_namespace n ON n.oid = f.pronamespace
                   WHERE n.nspname = 'util' AND has_function_privilege(r.oid, f.oid, 'EXECUTE')) z
   WHERE r.rolname ~ '^(rs|rme|openbao)_' AND NOT r.rolsuper
     AND r.rolname <> p || 'audit_migration'
     AND NOT (p = 'rme_' AND r.rolname IN ('rme_audit_tenant_reader', 'rme_audit_tenant_reader_login')
              AND x IN ('USAGE util', 'EXECUTE util.fn_current_fasyankes_id()'));
  IF bad IS NOT NULL THEN
    RAISE EXCEPTION 'GAGAL F1-T14 hak util berlebih di %: %', current_database(), bad;
  END IF;
  RAISE NOTICE 'LULUS F1-T14 tidak ada role lain dengan USAGE util atau EXECUTE fungsi util di %', current_database();
END $$;

-- util.fn_audit_role: pemetaan benar dan gagal-tertutup.
DO $$
DECLARE
  p text := current_setting('uji.prefiks');
  n text;
  st text;
BEGIN
  FOREACH n IN ARRAY ARRAY['audit_writer', 'audit_reader', 'audit_verifier', 'audit_migration'] LOOP
    IF util.fn_audit_role(n) IS DISTINCT FROM p || n THEN
      RAISE EXCEPTION 'GAGAL fn_audit_role(%) = % di %', n, util.fn_audit_role(n), current_database();
    END IF;
  END LOOP;
  IF p = 'rme_' AND util.fn_audit_role('audit_tenant_reader') <> 'rme_audit_tenant_reader' THEN
    RAISE EXCEPTION 'GAGAL fn_audit_role(audit_tenant_reader) di rme_audit';
  END IF;
  RAISE NOTICE 'LULUS fn_audit_role di % memetakan nama logis DD §0.6 B ke %*', current_database(), p;
  FOR n IN SELECT unnest(ARRAY['audit_tenant_reader_x', 'app', 'audit_migration; DROP ROLE x', 'AUDIT_WRITER', ''])
            UNION ALL SELECT CASE WHEN p = 'rs_' THEN 'audit_tenant_reader' END
            UNION ALL SELECT NULL LOOP
    st := NULL;
    BEGIN
      PERFORM util.fn_audit_role(n);
    EXCEPTION WHEN invalid_parameter_value THEN
      st := SQLSTATE;
    END;
    IF st IS NULL THEN
      RAISE EXCEPTION 'GAGAL NEG fn_audit_role(%) di % seharusnya ditolak', coalesce(n, '<null>'), current_database();
    END IF;
  END LOOP;
  RAISE NOTICE 'LULUS NEG fn_audit_role di % menolak nama di luar daftar tertutup, NULL, dan role yang tidak ada (22023)', current_database();
END $$;

-- Test negatif dengan role sungguhan: role runtime audit tidak dapat memanggil fungsi util.
BEGIN;
DO $$
DECLARE
  p text := current_setting('uji.prefiks');
  r text;
BEGIN
  FOREACH r IN ARRAY CASE WHEN p = 'rs_'
      THEN ARRAY['rs_audit_writer_login', 'rs_audit_reader_login', 'rs_audit_verifier_login']
      ELSE ARRAY['rme_audit_writer_login', 'rme_audit_verifier_login'] END LOOP
    PERFORM pg_temp.uji_tolak(format('NEG %s tidak dapat memanggil util.fn_audit_role', r), r,
                              'SELECT util.fn_audit_role(''audit_writer'')', '42501', 'permission denied for schema util');
    PERFORM pg_temp.uji_tolak(format('NEG %s tidak dapat memanggil util.fn_reject_modification', r), r,
                              'SELECT util.fn_reject_modification()', '42501', 'permission denied for schema util');
    IF p = 'rme_' THEN
      PERFORM pg_temp.uji_tolak(format('NEG %s tidak dapat memanggil util.fn_current_fasyankes_id', r), r,
                                'SELECT util.fn_current_fasyankes_id()', '42501', 'permission denied for schema util');
    END IF;
  END LOOP;
  IF p = 'rme_' THEN
    PERFORM pg_temp.uji_tolak('NEG rme_audit_tenant_reader_login tidak dapat memanggil util.fn_audit_role', 'rme_audit_tenant_reader_login',
                              'SELECT util.fn_audit_role(''audit_writer'')', '42501', 'permission denied for function fn_audit_role');
    PERFORM pg_temp.uji_tolak('NEG rme_audit_tenant_reader_login tidak dapat memanggil util.fn_reject_modification', 'rme_audit_tenant_reader_login',
                              'SELECT util.fn_reject_modification()', '42501', 'permission denied for function fn_reject_modification');
    PERFORM pg_temp.uji_tolak('NEG rme_audit_tenant_reader_login tidak dapat membuat fungsi di util', 'rme_audit_tenant_reader_login',
                              'CREATE FUNCTION util.fn_uji() RETURNS int LANGUAGE sql AS ''SELECT 1''', '42501', 'permission denied for schema util');
  END IF;
END $$;
ROLLBACK;

-- util.fn_current_fasyankes_id (hanya rme_audit, §5.2): gagal-tertutup, nilai lokal transaksi.
SELECT (:'prefiks' = 'rme_') AS rme \gset
\if :rme
SET ROLE rme_audit_tenant_reader_login;
DO $$
DECLARE
  st text;
BEGIN
  IF util.fn_current_fasyankes_id() IS NOT NULL THEN
    RAISE EXCEPTION 'GAGAL fn_current_fasyankes_id tanpa konteks harus NULL';
  END IF;
  PERFORM pg_catalog.set_config('rme.fasyankes_id', '', true);
  IF util.fn_current_fasyankes_id() IS NOT NULL THEN
    RAISE EXCEPTION 'GAGAL fn_current_fasyankes_id dengan string kosong harus NULL';
  END IF;
  PERFORM pg_catalog.set_config('rme.fasyankes_id', '00000000-0000-7000-8000-0000000000f1', true);
  IF util.fn_current_fasyankes_id() IS DISTINCT FROM '00000000-0000-7000-8000-0000000000f1'::uuid THEN
    RAISE EXCEPTION 'GAGAL fn_current_fasyankes_id tidak mengembalikan konteks';
  END IF;
  PERFORM pg_catalog.set_config('rme.fasyankes_id', 'bukan-uuid', true);
  BEGIN
    PERFORM util.fn_current_fasyankes_id();
  EXCEPTION WHEN invalid_text_representation THEN
    st := SQLSTATE;
  END;
  IF st IS NULL THEN
    RAISE EXCEPTION 'GAGAL fn_current_fasyankes_id dengan nilai bukan UUID harus galat';
  END IF;
  RAISE NOTICE 'LULUS fn_current_fasyankes_id (rme_audit_tenant_reader_login): tanpa konteks/kosong NULL, UUID dikembalikan, bukan UUID galat 22P02';
END $$;
BEGIN;
SELECT pg_catalog.set_config('rme.fasyankes_id', '00000000-0000-7000-8000-0000000000f2', true) AS _k \gset
COMMIT;
DO $$
BEGIN
  IF util.fn_current_fasyankes_id() IS NOT NULL THEN
    RAISE EXCEPTION 'GAGAL konteks set_config(..., true) terbawa melewati COMMIT (kebocoran pool RME-02)';
  END IF;
  RAISE NOTICE 'LULUS konteks rme.fasyankes_id lokal transaksi: setelah COMMIT fn_current_fasyankes_id NULL';
END $$;
RESET ROLE;
\endif
