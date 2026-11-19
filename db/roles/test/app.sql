-- db/roles/test/app.sql: test katalog dan test negatif bootstrap klaster (a) (db/roles/app.sql).
-- Dijalankan scripts/db-roles-check.sh dengan psql di dalam kontainer PostgreSQL lokal/CI, sebagai
-- superuser di database postgres. Pindah role memakai \connect lewat soket lokal (trust di image
-- resmi postgres; HANYA lingkungan lokal/CI, bukan production).
-- Dasar: docs/05-physical-schema.md §2.1, §2.2, §3.1–§3.5, §10.2, §11.1, §12 (T-02, T-03, T-04, T-05, T-21).
-- Setiap pemeriksaan yang lulus mencetak "LULUS <label>"; yang gagal menghentikan skrip.

\set ON_ERROR_STOP on
SELECT current_user AS su \gset

-- ---------------------------------------------------------------------------------------------
-- A. Katalog tingkat klaster (sebagai superuser)
-- ---------------------------------------------------------------------------------------------

-- T-05: role dan atribut persis seperti §3.1 butir 3 dan §3.2 (a).
DO $$
DECLARE
  diff text;
BEGIN
  WITH expected(rolname, can_login, conn_limit, can_createrole) AS (VALUES
      ('rs_app', false, -1, false), ('rs_audit_relay', false, -1, false),
      ('rs_privacy_worker', false, -1, false), ('rs_key_rotation', false, -1, false),
      ('rs_migration', false, -1, false), ('rs_ops_monitor', false, -1, false),
      ('rs_api', true, 40, false), ('rs_realtime', true, 20, false), ('rs_worker', true, 20, false),
      ('rs_audit_relay_login', true, 6, false), ('rs_privacy_worker_login', true, 8, false),
      ('rs_metrics', true, 2, false), ('keycloak', true, 40, false),
      ('openbao_rs_admin', true, -1, true)
  ), actual AS (
    SELECT r.rolname::text, r.rolcanlogin, r.rolconnlimit, r.rolcreaterole
      FROM pg_catalog.pg_roles r JOIN expected e ON e.rolname = r.rolname
     WHERE NOT r.rolsuper AND NOT r.rolcreatedb AND NOT r.rolreplication AND NOT r.rolbypassrls
       AND r.rolinherit
  )
  SELECT string_agg(x, '; ') INTO diff FROM (
    SELECT format('kurang/beda: %s', e) AS x FROM (SELECT * FROM expected EXCEPT SELECT * FROM actual) e
  ) d;
  IF diff IS NOT NULL THEN
    RAISE EXCEPTION 'GAGAL T-05 (a) role/atribut: %', diff;
  END IF;
  RAISE NOTICE 'LULUS T-05 (a) 14 role dengan atribut dan batas koneksi sesuai skema fisik';
END $$;

-- T-05: keanggotaan persis (tidak kurang, tidak lebih), termasuk opsi INHERIT/SET/ADMIN.
DO $$
DECLARE
  diff text;
BEGIN
  WITH proj(rolname) AS (VALUES
      ('rs_app'), ('rs_audit_relay'), ('rs_privacy_worker'), ('rs_key_rotation'), ('rs_migration'),
      ('rs_ops_monitor'), ('rs_api'), ('rs_realtime'), ('rs_worker'), ('rs_audit_relay_login'),
      ('rs_privacy_worker_login'), ('rs_metrics'), ('keycloak'), ('openbao_rs_admin')
  ), expected(grp, member, adm, inh, setopt) AS (VALUES
      ('rs_app', 'rs_api', false, true, false),
      ('rs_app', 'rs_realtime', false, true, false),
      ('rs_app', 'rs_worker', false, true, false),
      ('rs_audit_relay', 'rs_audit_relay_login', false, true, false),
      ('rs_privacy_worker', 'rs_privacy_worker_login', false, true, false),
      ('rs_ops_monitor', 'rs_metrics', false, true, false),
      ('pg_monitor', 'rs_ops_monitor', false, true, false),
      ('rs_migration', 'openbao_rs_admin', true, false, false),
      ('rs_key_rotation', 'openbao_rs_admin', true, false, false),
      ('rs_ops_monitor', 'openbao_rs_admin', true, false, false)
  ), actual AS (
    SELECT g.rolname::text, m.rolname::text, a.admin_option, a.inherit_option, a.set_option
      FROM pg_catalog.pg_auth_members a
      JOIN pg_catalog.pg_roles g ON g.oid = a.roleid
      JOIN pg_catalog.pg_roles m ON m.oid = a.member
     WHERE g.rolname IN (SELECT rolname FROM proj) OR m.rolname IN (SELECT rolname FROM proj)
  )
  SELECT string_agg(x, '; ') INTO diff FROM (
    SELECT format('kurang: %s', e) AS x FROM (SELECT * FROM expected EXCEPT SELECT * FROM actual) e
    UNION ALL
    SELECT format('lebih: %s', l) FROM (SELECT * FROM actual EXCEPT SELECT * FROM expected) l
  ) d;
  IF diff IS NOT NULL THEN
    RAISE EXCEPTION 'GAGAL T-05 (a) keanggotaan: %', diff;
  END IF;
  RAISE NOTICE 'LULUS T-05 (a) keanggotaan dan opsi INHERIT/SET/ADMIN persis';
END $$;

-- T-03: ACL database persis. PUBLIC tanpa CONNECT/TEMP; selain pemilik hanya CONNECT ke grup.
DO $$
DECLARE
  diff text;
BEGIN
  WITH expected(datname, grantee, priv) AS (VALUES
      ('ruang_sehat', 'rs_app', 'CONNECT'), ('ruang_sehat', 'rs_audit_relay', 'CONNECT'),
      ('ruang_sehat', 'rs_privacy_worker', 'CONNECT'), ('ruang_sehat', 'rs_key_rotation', 'CONNECT'),
      ('ruang_sehat', 'rs_migration', 'CONNECT'), ('ruang_sehat', 'rs_ops_monitor', 'CONNECT'),
      ('ruang_sehat', 'openbao_rs_admin', 'CONNECT')
  ), actual AS (
    SELECT d.datname::text, CASE WHEN a.grantee = 0 THEN 'PUBLIC' ELSE pg_catalog.pg_get_userbyid(a.grantee)::text END,
           a.privilege_type::text
      FROM pg_catalog.pg_database d, pg_catalog.aclexplode(d.datacl) a
     WHERE d.datname IN ('ruang_sehat', 'keycloak') AND a.grantee <> d.datdba
  )
  SELECT string_agg(x, '; ') INTO diff FROM (
    SELECT format('kurang: %s', e) AS x FROM (SELECT * FROM expected EXCEPT SELECT * FROM actual) e
    UNION ALL
    SELECT format('lebih: %s', l) FROM (SELECT * FROM actual EXCEPT SELECT * FROM expected) l
  ) d;
  IF diff IS NOT NULL THEN
    RAISE EXCEPTION 'GAGAL T-03 (a) ACL database: %', diff;
  END IF;
  IF EXISTS (SELECT 1 FROM pg_catalog.pg_database WHERE datname IN ('ruang_sehat', 'keycloak') AND datacl IS NULL) THEN
    RAISE EXCEPTION 'GAGAL T-03 (a) datacl NULL (hak bawaan PUBLIC masih berlaku)';
  END IF;
  RAISE NOTICE 'LULUS T-03 (a) ruang_sehat/keycloak: PUBLIC tanpa CONNECT/TEMP, CONNECT hanya grup yang berhak';
END $$;

-- §2.1 butir 3: ruang_sehat dimiliki superuser bootstrap; keycloak dimiliki role keycloak.
DO $$
BEGIN
  IF NOT (SELECT r.rolsuper FROM pg_catalog.pg_database d JOIN pg_catalog.pg_roles r ON r.oid = d.datdba
           WHERE d.datname = 'ruang_sehat') THEN
    RAISE EXCEPTION 'GAGAL §2.1 pemilik ruang_sehat bukan superuser';
  END IF;
  IF (SELECT pg_catalog.pg_get_userbyid(datdba) FROM pg_catalog.pg_database WHERE datname = 'keycloak') <> 'keycloak' THEN
    RAISE EXCEPTION 'GAGAL §2.1 pemilik database keycloak bukan role keycloak';
  END IF;
  RAISE NOTICE 'LULUS §2.1 pemilik database: ruang_sehat superuser, keycloak role keycloak';
END $$;

-- T-21 (bagian hak): keycloak tidak dapat memasuki ruang_sehat; role proyek tidak dapat memasuki
-- keycloak/postgres; tidak ada role proyek dengan TEMP di ruang_sehat.
DO $$
DECLARE
  bad text;
BEGIN
  SELECT string_agg(format('%s->%s:%s', r, db, p), ', ') INTO bad
    FROM (VALUES ('keycloak', 'ruang_sehat', 'CONNECT')) v(r, db, p)
   WHERE pg_catalog.has_database_privilege(r, db, p);
  IF bad IS NULL THEN
    SELECT string_agg(format('%s->%s:%s', r.rolname, db, p), ', ') INTO bad
      FROM pg_catalog.pg_roles r,
           (VALUES ('keycloak', 'CONNECT'), ('postgres', 'CONNECT'), ('ruang_sehat', 'TEMPORARY')) v(db, p)
     WHERE r.rolname ~ '^(rs|openbao_rs)_' AND pg_catalog.has_database_privilege(r.oid, db, p);
  END IF;
  IF bad IS NOT NULL THEN
    RAISE EXCEPTION 'GAGAL T-21 (a) hak database berlebih: %', bad;
  END IF;
  RAISE NOTICE 'LULUS T-21 (a) keycloak tanpa CONNECT ruang_sehat; role rs_* tanpa CONNECT keycloak/postgres dan tanpa TEMP';
END $$;

-- §3.5 butir 4: pengaturan database dan per role login persis.
DO $$
DECLARE
  diff text;
BEGIN
  WITH expected(datname, rolname, cfg) AS (
    SELECT 'ruang_sehat', '-', c FROM unnest(ARRAY['search_path=app, ext', 'TimeZone=UTC',
                                                   'idle_in_transaction_session_timeout=60s']) c
    UNION ALL
    SELECT 'ruang_sehat', r, c FROM (VALUES
        ('rs_api', ARRAY['search_path=app, ext', 'statement_timeout=3s', 'lock_timeout=2s', 'idle_in_transaction_session_timeout=30s']),
        ('rs_realtime', ARRAY['search_path=app, ext', 'statement_timeout=3s', 'lock_timeout=2s', 'idle_in_transaction_session_timeout=30s']),
        ('rs_worker', ARRAY['search_path=app, ext', 'statement_timeout=60s', 'lock_timeout=5s', 'idle_in_transaction_session_timeout=60s']),
        ('rs_privacy_worker_login', ARRAY['search_path=app, ext', 'statement_timeout=300s', 'lock_timeout=5s', 'idle_in_transaction_session_timeout=60s']),
        ('rs_audit_relay_login', ARRAY['search_path=app, ext', 'statement_timeout=30s', 'lock_timeout=5s', 'idle_in_transaction_session_timeout=30s']),
        ('rs_metrics', ARRAY['search_path=app, ext'])
      ) v(r, cs), unnest(cs) c
    UNION ALL
    SELECT 'keycloak', 'keycloak', 'idle_in_transaction_session_timeout=60s'
    UNION ALL
    SELECT '-', 'openbao_rs_admin', c FROM unnest(ARRAY['log_min_error_statement=panic', 'log_statement=none']) c
  ), actual AS (
    SELECT coalesce(d.datname::text, '-'), coalesce(r.rolname::text, '-'), c
      FROM pg_catalog.pg_db_role_setting s
      LEFT JOIN pg_catalog.pg_database d ON d.oid = s.setdatabase
      LEFT JOIN pg_catalog.pg_roles r ON r.oid = s.setrole,
           unnest(s.setconfig) c
     WHERE d.datname IN ('ruang_sehat', 'keycloak')
        OR r.rolname IN ('rs_api', 'rs_realtime', 'rs_worker', 'rs_privacy_worker_login',
                         'rs_audit_relay_login', 'rs_metrics', 'keycloak', 'openbao_rs_admin')
  )
  SELECT string_agg(x, '; ') INTO diff FROM (
    SELECT format('kurang: %s', e) AS x FROM (SELECT * FROM expected EXCEPT SELECT * FROM actual) e
    UNION ALL
    SELECT format('lebih: %s', l) FROM (SELECT * FROM actual EXCEPT SELECT * FROM expected) l
  ) d;
  IF diff IS NOT NULL THEN
    RAISE EXCEPTION 'GAGAL §3.5 (a) pengaturan: %', diff;
  END IF;
  RAISE NOTICE 'LULUS §3.5 (a) search_path, TimeZone, batas waktu §10.2, dan log role admin OpenBao persis';
END $$;

-- ---------------------------------------------------------------------------------------------
-- B. Katalog database ruang_sehat (sebagai superuser)
-- ---------------------------------------------------------------------------------------------
\connect ruang_sehat :su

-- T-02: skema persis app, util, ext; public tidak ada; pemilik sesuai §2.1.
DO $$
DECLARE
  diff text;
BEGIN
  WITH expected(nspname, owner_kind) AS (VALUES ('app', 'rs_migration'), ('util', 'rs_migration'), ('ext', 'superuser')),
  actual AS (
    SELECT n.nspname::text, CASE WHEN r.rolsuper THEN 'superuser' ELSE r.rolname::text END
      FROM pg_catalog.pg_namespace n JOIN pg_catalog.pg_roles r ON r.oid = n.nspowner
     WHERE n.nspname !~ '^pg_' AND n.nspname <> 'information_schema'
  )
  SELECT string_agg(x, '; ') INTO diff FROM (
    SELECT format('kurang: %s', e) AS x FROM (SELECT * FROM expected EXCEPT SELECT * FROM actual) e
    UNION ALL
    SELECT format('lebih: %s', l) FROM (SELECT * FROM actual EXCEPT SELECT * FROM expected) l
  ) d;
  IF diff IS NOT NULL THEN
    RAISE EXCEPTION 'GAGAL T-02 (ruang_sehat) skema: %', diff;
  END IF;
  RAISE NOTICE 'LULUS T-02 (ruang_sehat) skema persis app, util (rs_migration), ext (superuser); public tidak ada';
END $$;

-- §3.5 butir 3: ACL skema persis; PUBLIC tanpa hak; CREATE hanya pemilik.
DO $$
DECLARE
  diff text;
BEGIN
  WITH expected(nspname, grantee, priv) AS (VALUES
      ('app', 'rs_app', 'USAGE'), ('app', 'rs_audit_relay', 'USAGE'),
      ('app', 'rs_privacy_worker', 'USAGE'), ('app', 'rs_key_rotation', 'USAGE'),
      ('ext', 'rs_app', 'USAGE'), ('ext', 'rs_migration', 'USAGE')
  ), actual AS (
    SELECT n.nspname::text, CASE WHEN a.grantee = 0 THEN 'PUBLIC' ELSE pg_catalog.pg_get_userbyid(a.grantee)::text END,
           a.privilege_type::text
      FROM pg_catalog.pg_namespace n, pg_catalog.aclexplode(n.nspacl) a
     WHERE n.nspname IN ('app', 'util', 'ext') AND a.grantee <> n.nspowner
  )
  SELECT string_agg(x, '; ') INTO diff FROM (
    SELECT format('kurang: %s', e) AS x FROM (SELECT * FROM expected EXCEPT SELECT * FROM actual) e
    UNION ALL
    SELECT format('lebih: %s', l) FROM (SELECT * FROM actual EXCEPT SELECT * FROM expected) l
  ) d;
  IF diff IS NOT NULL THEN
    RAISE EXCEPTION 'GAGAL §3.5 (ruang_sehat) ACL skema: %', diff;
  END IF;
  RAISE NOTICE 'LULUS §3.5 (ruang_sehat) USAGE app/ext hanya grup yang berhak, util tanpa hak, CREATE hanya pemilik';
END $$;

-- §2.2: ekstensi persis plpgsql (pg_catalog) dan pg_trgm (ext).
DO $$
DECLARE
  diff text;
BEGIN
  WITH expected(extname, nspname) AS (VALUES ('plpgsql', 'pg_catalog'), ('pg_trgm', 'ext')),
  actual AS (
    SELECT e.extname::text, n.nspname::text
      FROM pg_catalog.pg_extension e JOIN pg_catalog.pg_namespace n ON n.oid = e.extnamespace
  )
  SELECT string_agg(x, '; ') INTO diff FROM (
    SELECT format('kurang: %s', e) AS x FROM (SELECT * FROM expected EXCEPT SELECT * FROM actual) e
    UNION ALL
    SELECT format('lebih: %s', l) FROM (SELECT * FROM actual EXCEPT SELECT * FROM expected) l
  ) d;
  IF diff IS NOT NULL THEN
    RAISE EXCEPTION 'GAGAL §2.2 (ruang_sehat) ekstensi: %', diff;
  END IF;
  RAISE NOTICE 'LULUS §2.2 (ruang_sehat) ekstensi hanya pg_trgm di skema ext';
END $$;

-- §2.1 butir 6: default privileges rs_migration mencabut EXECUTE fungsi dari PUBLIC.
DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM pg_catalog.pg_default_acl d
     WHERE d.defaclrole = 'rs_migration'::regrole AND d.defaclobjtype = 'f' AND d.defaclnamespace = 0
       AND NOT EXISTS (SELECT 1 FROM pg_catalog.aclexplode(d.defaclacl) a WHERE a.grantee = 0)) THEN
    RAISE EXCEPTION 'GAGAL §2.1 butir 6 (ruang_sehat) default privileges EXECUTE PUBLIC belum dicabut';
  END IF;
  RAISE NOTICE 'LULUS §2.1 butir 6 (ruang_sehat) ALTER DEFAULT PRIVILEGES FOR ROLE rs_migration tanpa EXECUTE PUBLIC';
END $$;

-- ---------------------------------------------------------------------------------------------
-- C. Jalur kredensial dinamis OpenBao tiruan (§3.4, §11.1) dan test negatif role admin OpenBao
-- ---------------------------------------------------------------------------------------------
\connect ruang_sehat openbao_rs_admin
SET client_min_messages = warning;
DROP ROLE IF EXISTS "v-uji-rs-migration";   -- sisa uji sebelumnya (lokal) bila ada
SET client_min_messages = notice;

DO $$
DECLARE
  c record;
  st text;
BEGIN
  FOR c IN SELECT * FROM (VALUES
      ('NEG openbao_rs_admin tidak dapat membuat role anggota rs_app (tanpa ADMIN)', 'CREATE ROLE uji_neg_a IN ROLE rs_app', '42501'),
      ('NEG openbao_rs_admin tidak dapat membuat role SUPERUSER', 'CREATE ROLE uji_neg_b SUPERUSER', '42501'),
      ('NEG openbao_rs_admin tidak dapat membuat role BYPASSRLS', 'CREATE ROLE uji_neg_c BYPASSRLS', '42501'),
      ('NEG openbao_rs_admin tidak dapat membuat role REPLICATION', 'CREATE ROLE uji_neg_d REPLICATION', '42501'),
      ('NEG openbao_rs_admin tidak dapat mengubah role login aplikasi', 'ALTER ROLE rs_api CONNECTION LIMIT 1', '42501'),
      ('NEG openbao_rs_admin tidak dapat memberi keanggotaan rs_app', 'GRANT rs_app TO openbao_rs_admin', '42501'),
      ('NEG openbao_rs_admin tidak dapat SET ROLE ke rs_migration (SET FALSE)', 'SET ROLE rs_migration', '42501'),
      ('NEG openbao_rs_admin tidak mewarisi hak pemilik: CREATE TABLE app ditolak', 'CREATE TABLE app.uji_neg (x int)', '42501'),
      ('NEG openbao_rs_admin tidak dapat CREATE SCHEMA', 'CREATE SCHEMA uji_neg', '42501')
    ) AS t(label, stmt, expected)
  LOOP
    st := NULL;
    BEGIN
      EXECUTE c.stmt;
    EXCEPTION WHEN OTHERS THEN
      st := SQLSTATE;
    END;
    IF st IS DISTINCT FROM c.expected THEN
      RAISE EXCEPTION 'GAGAL %: SQLSTATE % (diharapkan %)', c.label, coalesce(st, 'tanpa galat'), coalesce(c.expected, 'sukses');
    END IF;
    RAISE NOTICE 'LULUS %', c.label;
  END LOOP;
END $$;

-- creation_statements OpenBao "rs-migration" (§3.4) dengan password acak sekali pakai.
DO $$
BEGIN
  EXECUTE format('CREATE ROLE %I WITH LOGIN PASSWORD %L VALID UNTIL %L '
                 'NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS IN ROLE rs_migration',
                 'v-uji-rs-migration', md5(random()::text || clock_timestamp()::text),
                 (now() + interval '30 minutes')::text);
  EXECUTE format('ALTER ROLE %I SET role = %L', 'v-uji-rs-migration', 'rs_migration');
  EXECUTE format('ALTER ROLE %I SET search_path = app', 'v-uji-rs-migration');
  RAISE NOTICE 'LULUS §3.4 openbao_rs_admin menjalankan creation_statements rs-migration (IN ROLE, SET role, SET search_path)';
END $$;

\connect ruang_sehat v-uji-rs-migration
DO $$
BEGIN
  IF current_user <> 'rs_migration' OR session_user <> 'v-uji-rs-migration' THEN
    RAISE EXCEPTION 'GAGAL §3.4 sesi dinamis: current_user=% session_user=%', current_user, session_user;
  END IF;
  IF current_setting('search_path') <> 'app' OR current_setting('TimeZone') <> 'UTC' THEN
    RAISE EXCEPTION 'GAGAL §3.4 sesi dinamis: search_path=% TimeZone=%', current_setting('search_path'), current_setting('TimeZone');
  END IF;
  RAISE NOTICE 'LULUS §3.4 sesi role dinamis berjalan sebagai rs_migration, search_path app, TimeZone UTC';

  -- Objek probe: nama tanpa skema masuk ke app (search_path), opclass trigram berkualifikasi ext (§2.2).
  CREATE TABLE uji_probe (x text);
  CREATE INDEX ix_uji_probe_x_trgm ON app.uji_probe USING gin (x ext.gin_trgm_ops);
  CREATE TABLE util.uji_probe (x integer);
  CREATE FUNCTION util.fn_uji_probe() RETURNS integer LANGUAGE sql AS 'SELECT 1';
  CREATE PROCEDURE util.pr_uji_probe() LANGUAGE sql AS 'SELECT 1';
  RAISE NOTICE 'LULUS §3.4 role dinamis membuat tabel/index/fungsi/prosedur di app dan util (opclass ext.gin_trgm_ops)';
END $$;

DO $$
DECLARE
  c record;
  st text;
BEGIN
  FOR c IN SELECT * FROM (VALUES
      ('NEG rs_migration (dinamis) tidak dapat CREATE SCHEMA', 'CREATE SCHEMA uji_neg', '42501'),
      ('NEG rs_migration (dinamis) tidak dapat CREATE EXTENSION', 'CREATE EXTENSION btree_gist', '42501'),
      ('NEG rs_migration (dinamis) tidak dapat membuat tabel temporer (TEMP dicabut)', 'CREATE TEMP TABLE uji_neg (x int)', '42501'),
      ('NEG rs_migration (dinamis) tidak dapat membuat objek di ext', 'CREATE TABLE ext.uji_neg (x int)', '42501'),
      ('NEG rs_migration (dinamis) tidak dapat SET ROLE ke rs_app', 'SET ROLE rs_app', '42501')
    ) AS t(label, stmt, expected)
  LOOP
    st := NULL;
    BEGIN
      EXECUTE c.stmt;
    EXCEPTION WHEN OTHERS THEN
      st := SQLSTATE;
    END;
    IF st IS DISTINCT FROM c.expected THEN
      RAISE EXCEPTION 'GAGAL %: SQLSTATE % (diharapkan %)', c.label, coalesce(st, 'tanpa galat'), coalesce(c.expected, 'sukses');
    END IF;
    RAISE NOTICE 'LULUS %', c.label;
  END LOOP;
END $$;

\connect ruang_sehat :su
-- T-04 (pratinjau): objek buatan role dinamis dimiliki rs_migration; T-03: tanpa EXECUTE PUBLIC.
DO $$
DECLARE
  bad text;
BEGIN
  SELECT string_agg(format('%s.%s owner=%s', n.nspname, c.relname, pg_catalog.pg_get_userbyid(c.relowner)), ', ') INTO bad
    FROM pg_catalog.pg_class c JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
   WHERE n.nspname IN ('app', 'util') AND c.relowner <> 'rs_migration'::regrole;
  IF bad IS NULL THEN
    SELECT string_agg(format('%s.%s owner=%s', n.nspname, p.proname, pg_catalog.pg_get_userbyid(p.proowner)), ', ') INTO bad
      FROM pg_catalog.pg_proc p JOIN pg_catalog.pg_namespace n ON n.oid = p.pronamespace
     WHERE n.nspname IN ('app', 'util') AND p.proowner <> 'rs_migration'::regrole;
  END IF;
  IF bad IS NOT NULL THEN
    RAISE EXCEPTION 'GAGAL T-04 objek tidak dimiliki rs_migration: %', bad;
  END IF;
  IF NOT EXISTS (SELECT 1 FROM pg_catalog.pg_class c JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
                  WHERE n.nspname = 'app' AND c.relname = 'uji_probe') THEN
    RAISE EXCEPTION 'GAGAL §3.4 tabel tanpa skema tidak masuk ke app';
  END IF;
  RAISE NOTICE 'LULUS T-04 objek buatan role dinamis dimiliki rs_migration; tabel tanpa skema masuk ke app';

  IF EXISTS (SELECT 1 FROM pg_catalog.pg_proc p JOIN pg_catalog.pg_namespace n ON n.oid = p.pronamespace
              WHERE n.nspname = 'util' AND p.proname IN ('fn_uji_probe', 'pr_uji_probe')
                AND (p.proacl IS NULL OR EXISTS (SELECT 1 FROM pg_catalog.aclexplode(p.proacl) a WHERE a.grantee = 0))) THEN
    RAISE EXCEPTION 'GAGAL T-03 fungsi/prosedur baru masih dapat dieksekusi PUBLIC';
  END IF;
  RAISE NOTICE 'LULUS T-03 fungsi dan prosedur baru milik rs_migration tanpa EXECUTE PUBLIC (default privileges)';
END $$;

-- ---------------------------------------------------------------------------------------------
-- D. Role login aplikasi: pengaturan sesi dan test negatif (login sungguhan, bukan superuser)
-- ---------------------------------------------------------------------------------------------
\connect ruang_sehat rs_api
DO $$
BEGIN
  IF current_user <> 'rs_api'
     OR current_setting('search_path') <> 'app, ext'
     OR current_setting('statement_timeout') <> '3s'
     OR current_setting('lock_timeout') <> '2s'
     OR current_setting('idle_in_transaction_session_timeout') <> '30s'
     OR current_setting('TimeZone') <> 'UTC' THEN
    RAISE EXCEPTION 'GAGAL §10.2 sesi rs_api: % % % % %', current_setting('search_path'), current_setting('statement_timeout'),
      current_setting('lock_timeout'), current_setting('idle_in_transaction_session_timeout'), current_setting('TimeZone');
  END IF;
  RAISE NOTICE 'LULUS §10.2 sesi rs_api: search_path app, ext; 3s/2s/30s; UTC';
END $$;

DO $$
DECLARE
  c record;
  st text;
BEGIN
  FOR c IN SELECT * FROM (VALUES
      ('NEG rs_api tidak dapat membuat objek di app', 'CREATE TABLE app.uji_neg (x int)', '42501'),
      ('NEG rs_api tidak dapat membuat objek di util', 'CREATE TABLE util.uji_neg (x int)', '42501'),
      ('NEG rs_api tidak dapat membuat objek di ext', 'CREATE TABLE ext.uji_neg (x int)', '42501'),
      ('NEG rs_api tidak dapat membuat objek tanpa skema (search_path)', 'CREATE TABLE uji_neg (x int)', '42501'),
      ('NEG rs_api tidak dapat membuat fungsi di app', 'CREATE FUNCTION app.fn_uji_neg() RETURNS int LANGUAGE sql AS ''SELECT 1''', '42501'),
      ('NEG rs_api: skema public tidak ada', 'CREATE TABLE public.uji_neg (x int)', '3F000'),
      ('NEG rs_api tidak dapat membuat tabel temporer (TEMP dicabut)', 'CREATE TEMP TABLE uji_neg (x int)', '42501'),
      ('NEG rs_api tidak dapat CREATE SCHEMA', 'CREATE SCHEMA uji_neg', '42501'),
      ('NEG rs_api tidak dapat CREATE EXTENSION', 'CREATE EXTENSION btree_gist', '42501'),
      ('NEG rs_api tidak dapat SET ROLE ke rs_app (SET FALSE)', 'SET ROLE rs_app', '42501'),
      ('NEG rs_api tidak dapat SET ROLE ke rs_migration', 'SET ROLE rs_migration', '42501'),
      ('NEG rs_api tidak dapat membaca tabel app tanpa GRANT', 'SELECT count(*) FROM app.uji_probe', '42501'),
      ('NEG rs_api tidak dapat membaca skema util', 'SELECT count(*) FROM util.uji_probe', '42501'),
      ('NEG rs_api tidak dapat mengeksekusi fungsi util', 'SELECT util.fn_uji_probe()', '42501'),
      ('NEG rs_api tidak dapat memanggil prosedur util', 'CALL util.pr_uji_probe()', '42501'),
      ('POS rs_api dapat memakai fungsi pg_trgm lewat search_path ext', 'SELECT similarity(''ruang'', ''ruangan'')', NULL)
    ) AS t(label, stmt, expected)
  LOOP
    st := NULL;
    BEGIN
      EXECUTE c.stmt;
    EXCEPTION WHEN OTHERS THEN
      st := SQLSTATE;
    END;
    IF st IS DISTINCT FROM c.expected THEN
      RAISE EXCEPTION 'GAGAL %: SQLSTATE % (diharapkan %)', c.label, coalesce(st, 'tanpa galat'), coalesce(c.expected, 'sukses');
    END IF;
    RAISE NOTICE 'LULUS %', c.label;
  END LOOP;
END $$;

\connect ruang_sehat rs_metrics
DO $$
DECLARE
  c record;
  st text;
BEGIN
  IF NOT pg_catalog.pg_has_role('pg_monitor', 'USAGE') THEN
    RAISE EXCEPTION 'GAGAL §3.2 rs_metrics tidak mewarisi pg_monitor';
  END IF;
  RAISE NOTICE 'LULUS §3.2 rs_metrics mewarisi pg_monitor lewat rs_ops_monitor';
  FOR c IN SELECT * FROM (VALUES
      ('NEG rs_metrics tidak dapat membaca skema app', 'SELECT count(*) FROM app.uji_probe', '42501'),
      ('NEG rs_metrics tidak dapat SET ROLE ke pg_monitor', 'SET ROLE pg_monitor', '42501'),
      ('NEG rs_metrics tidak dapat membuat objek di app', 'CREATE TABLE app.uji_neg (x int)', '42501')
    ) AS t(label, stmt, expected)
  LOOP
    st := NULL;
    BEGIN
      EXECUTE c.stmt;
    EXCEPTION WHEN OTHERS THEN
      st := SQLSTATE;
    END;
    IF st IS DISTINCT FROM c.expected THEN
      RAISE EXCEPTION 'GAGAL %: SQLSTATE % (diharapkan %)', c.label, coalesce(st, 'tanpa galat'), coalesce(c.expected, 'sukses');
    END IF;
    RAISE NOTICE 'LULUS %', c.label;
  END LOOP;
END $$;

-- Login sungguhan role login lain: dapat masuk ke ruang_sehat dengan batas waktu §10.2.
\connect ruang_sehat rs_realtime
DO $$ BEGIN
  IF current_setting('statement_timeout') <> '3s' OR pg_catalog.pg_has_role('rs_app', 'SET') THEN
    RAISE EXCEPTION 'GAGAL §10.2 sesi rs_realtime'; END IF;
  RAISE NOTICE 'LULUS §10.2 sesi rs_realtime (3s), tanpa SET ROLE rs_app';
END $$;
\connect ruang_sehat rs_worker
DO $$ BEGIN
  IF current_setting('statement_timeout') <> '1min' AND current_setting('statement_timeout') <> '60s' THEN
    RAISE EXCEPTION 'GAGAL §10.2 sesi rs_worker: %', current_setting('statement_timeout'); END IF;
  RAISE NOTICE 'LULUS §10.2 sesi rs_worker (60s)';
END $$;
\connect ruang_sehat rs_privacy_worker_login
DO $$ BEGIN
  IF current_setting('statement_timeout') NOT IN ('5min', '300s') OR NOT pg_catalog.pg_has_role('rs_privacy_worker', 'USAGE')
     OR pg_catalog.pg_has_role('rs_key_rotation', 'USAGE') THEN
    RAISE EXCEPTION 'GAGAL §10.2 sesi rs_privacy_worker_login: %', current_setting('statement_timeout'); END IF;
  RAISE NOTICE 'LULUS §10.2 sesi rs_privacy_worker_login (300s), tanpa rs_key_rotation';
END $$;
\connect ruang_sehat rs_audit_relay_login
DO $$ BEGIN
  IF current_setting('statement_timeout') <> '30s' OR pg_catalog.pg_has_role('rs_app', 'USAGE') THEN
    RAISE EXCEPTION 'GAGAL §10.2 sesi rs_audit_relay_login: %', current_setting('statement_timeout'); END IF;
  RAISE NOTICE 'LULUS §10.2 sesi rs_audit_relay_login (30s), bukan anggota rs_app';
END $$;

-- ---------------------------------------------------------------------------------------------
-- E. Pembersihan: probe dihapus pemiliknya, lalu revocation_statements OpenBao (DROP ROLE saja)
-- ---------------------------------------------------------------------------------------------
\connect ruang_sehat v-uji-rs-migration
DROP PROCEDURE util.pr_uji_probe();
DROP FUNCTION util.fn_uji_probe();
DROP TABLE util.uji_probe;
DROP TABLE app.uji_probe;

\connect ruang_sehat openbao_rs_admin
DO $$
BEGIN
  DROP ROLE "v-uji-rs-migration";
  RAISE NOTICE 'LULUS §3.4 revocation_statements: DROP ROLE role dinamis berhasil (tidak memiliki objek)';
END $$;

\connect ruang_sehat :su
DO $$
BEGIN
  IF EXISTS (SELECT 1 FROM pg_catalog.pg_class c JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
              WHERE n.nspname IN ('app', 'util') AND c.relname ~ 'uji_')
     OR EXISTS (SELECT 1 FROM pg_catalog.pg_proc p JOIN pg_catalog.pg_namespace n ON n.oid = p.pronamespace
                 WHERE n.nspname IN ('app', 'util') AND p.proname ~ 'uji_')
     OR EXISTS (SELECT 1 FROM pg_catalog.pg_roles WHERE rolname ~ '^v-uji-') THEN
    RAISE EXCEPTION 'GAGAL sisa objek uji di app/util';
  END IF;
  RAISE NOTICE 'LULUS pembersihan: tidak ada sisa objek uji di app/util dan tidak ada role dinamis tersisa';
END $$;
