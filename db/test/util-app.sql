-- db/test/util-app.sql: test fungsi bersama skema util database ruang_sehat (F1-T14).
-- Didahului db/test/lib.sql; dijalankan superuser di database ruang_sehat setelah migrasi.
-- Dasar: docs/05-physical-schema.md §2.1 butir 5-6, §4.2, §4.10, §11.2; CR-020.

-- Isi util persis: lima fungsi trigger, SECURITY INVOKER, plpgsql, search_path tetap, milik
-- rs_migration, ACL hanya pemilik (EXECUTE tidak diberikan ke PUBLIC atau role mana pun).
DO $$
DECLARE
  diff text;
BEGIN
  WITH expected(sig) AS (VALUES
      ('util.fn_reject_modification()'), ('util.fn_guard_log_delete()'), ('util.fn_effective_row_lock()'),
      ('util.fn_effective_row_no_backdate()'), ('util.fn_data_key_one_way()')
  ), actual AS (
    SELECT p.oid::regprocedure::text AS sig
      FROM pg_proc p JOIN pg_namespace n ON n.oid = p.pronamespace
     WHERE n.nspname = 'util'
       AND p.prorettype = 'trigger'::regtype AND NOT p.prosecdef
       AND p.prolang = (SELECT oid FROM pg_language WHERE lanname = 'plpgsql')
       AND p.proconfig = ARRAY['search_path=pg_catalog, pg_temp']
       AND p.proowner = 'rs_migration'::regrole
       AND NOT EXISTS (SELECT 1 FROM aclexplode(p.proacl) a WHERE a.grantee <> p.proowner)
       AND p.proacl IS NOT NULL
  )
  SELECT string_agg(x, '; ') INTO diff FROM (
    SELECT 'kurang/beda: ' || sig AS x FROM (SELECT sig FROM expected EXCEPT SELECT sig FROM actual) e
    UNION ALL
    SELECT 'lebih: ' || p.oid::regprocedure::text FROM pg_proc p JOIN pg_namespace n ON n.oid = p.pronamespace
     WHERE n.nspname = 'util' AND p.oid::regprocedure::text NOT IN (SELECT sig FROM expected)) d;
  IF diff IS NOT NULL THEN
    RAISE EXCEPTION 'GAGAL F1-T14 isi util ruang_sehat: %', diff;
  END IF;
  RAISE NOTICE 'LULUS F1-T14 util ruang_sehat: 5 fungsi trigger SECURITY INVOKER, search_path pg_catalog, pg_temp, milik rs_migration, EXECUTE hanya pemilik';
END $$;

-- Skema util: hanya pemilik (tanpa USAGE untuk role mana pun; trigger tidak memerlukannya).
DO $$
BEGIN
  IF EXISTS (SELECT 1 FROM pg_namespace n, aclexplode(n.nspacl) a
              WHERE n.nspname = 'util' AND a.grantee <> n.nspowner) THEN
    RAISE EXCEPTION 'GAGAL F1-T14 skema util ruang_sehat punya hak selain pemilik: %',
      (SELECT nspacl FROM pg_namespace WHERE nspname = 'util');
  END IF;
  RAISE NOTICE 'LULUS F1-T14 skema util ruang_sehat tanpa USAGE untuk role selain pemilik';
END $$;

-- Tidak ada role proyek selain pemilik yang punya USAGE util atau EXECUTE fungsi util (§2.1 butir 6).
DO $$
DECLARE
  bad text;
BEGIN
  SELECT string_agg(format('%s:%s', r.rolname, x), ', ') INTO bad
    FROM pg_roles r,
         LATERAL (SELECT 'USAGE util' AS x WHERE has_schema_privilege(r.oid, 'util', 'USAGE')
                  UNION ALL
                  SELECT 'EXECUTE ' || p.oid::regprocedure::text
                    FROM pg_proc p JOIN pg_namespace n ON n.oid = p.pronamespace
                   WHERE n.nspname = 'util' AND has_function_privilege(r.oid, p.oid, 'EXECUTE')) z
   WHERE r.rolname ~ '^(rs|openbao_rs)_' AND r.rolname <> 'rs_migration' AND NOT r.rolsuper;
  IF bad IS NOT NULL THEN
    RAISE EXCEPTION 'GAGAL F1-T14 hak util berlebih: %', bad;
  END IF;
  RAISE NOTICE 'LULUS F1-T14 tidak ada role rs_*/openbao_rs_* selain rs_migration dengan USAGE util atau EXECUTE fungsi util';
END $$;

-- Test negatif dengan role sungguhan (SET ROLE): role aplikasi tidak dapat memanggil fungsi util.
BEGIN;
DO $$
DECLARE
  r text;
  f text;
BEGIN
  FOREACH r IN ARRAY ARRAY['rs_api', 'rs_realtime', 'rs_worker', 'rs_privacy_worker_login', 'rs_audit_relay_login', 'rs_metrics'] LOOP
    FOREACH f IN ARRAY ARRAY['fn_reject_modification', 'fn_guard_log_delete', 'fn_effective_row_lock',
                             'fn_effective_row_no_backdate', 'fn_data_key_one_way'] LOOP
      PERFORM pg_temp.uji_tolak(format('NEG %s tidak dapat memanggil util.%s', r, f), r,
                                format('SELECT util.%I()', f), '42501', 'permission denied for schema util');
    END LOOP;
  END LOOP;
END $$;
-- Pemilik pun tidak dapat memakai fungsi trigger sebagai fungsi biasa.
SELECT pg_temp.uji_tolak('NEG fungsi trigger util tidak dapat dipanggil langsung (pemilik)', 'rs_migration',
                         'SELECT util.fn_reject_modification()', '0A000', 'trigger functions can only be called as triggers');
-- Role aplikasi tidak dapat membuat atau mengganti fungsi di util (pembajakan pustaka).
SELECT pg_temp.uji_tolak('NEG rs_api tidak dapat mengganti util.fn_reject_modification', 'rs_api',
                         'CREATE OR REPLACE FUNCTION util.fn_reject_modification() RETURNS trigger LANGUAGE plpgsql AS $b$ BEGIN RETURN NULL; END $b$',
                         '42501', 'permission denied for schema util');
SELECT pg_temp.uji_tolak('NEG rs_privacy_worker_login tidak dapat ALTER FUNCTION util', 'rs_privacy_worker_login',
                         'ALTER FUNCTION util.fn_guard_log_delete() RESET search_path', '42501', 'permission denied for schema util|must be owner');
ROLLBACK;
