-- db/roles/test/rme.sql: test katalog dan test negatif bootstrap klaster (c) (db/roles/rme.sql).
-- Dijalankan scripts/db-roles-check.sh dengan psql di dalam kontainer PostgreSQL lokal/CI, sebagai
-- superuser di database postgres. Pindah role memakai \connect lewat soket lokal (trust di image
-- resmi postgres; HANYA lingkungan lokal/CI, bukan production).
-- Dasar: docs/05-physical-schema.md §2.1, §2.2, §3.1–§3.5, §10.2, §11.1, §12 (T-02–T-05, T-21);
-- ADR-012 butir 6 dan 8 (role RME terpisah, tanpa BYPASSRLS).
-- Setiap pemeriksaan yang lulus mencetak "LULUS <label>"; yang gagal menghentikan skrip.

\set ON_ERROR_STOP on
SELECT current_user AS su \gset

-- ---------------------------------------------------------------------------------------------
-- A. Katalog tingkat klaster (sebagai superuser)
-- ---------------------------------------------------------------------------------------------

-- T-05: role dan atribut persis §3.2 (c).
DO $$
DECLARE
  diff text;
BEGIN
  WITH expected(rolname, can_login, conn_limit, can_createrole) AS (VALUES
      ('rme_app', false, -1, false), ('rme_auth', false, -1, false), ('rme_worker', false, -1, false),
      ('rme_operator', false, -1, false), ('rme_audit_relay', false, -1, false),
      ('rme_privacy_worker', false, -1, false), ('rme_key_rotation', false, -1, false),
      ('rme_migration', false, -1, false), ('rme_ops_monitor', false, -1, false),
      ('rme_api', true, 30, false), ('rme_api_auth', true, 6, false), ('rme_api_operator', true, 4, false),
      ('rme_worker_login', true, 16, false), ('rme_audit_relay_login', true, 6, false),
      ('rme_privacy_worker_login', true, 6, false), ('rme_metrics', true, 2, false),
      ('openbao_rme_admin', true, -1, true)
  ), actual AS (
    SELECT r.rolname::text, r.rolcanlogin, r.rolconnlimit, r.rolcreaterole
      FROM pg_catalog.pg_roles r JOIN expected e ON e.rolname = r.rolname
     WHERE NOT r.rolsuper AND NOT r.rolcreatedb AND NOT r.rolreplication AND NOT r.rolbypassrls
       AND r.rolinherit
  )
  SELECT string_agg(format('kurang/beda: %s', e), '; ') INTO diff
    FROM (SELECT * FROM expected EXCEPT SELECT * FROM actual) e;
  IF diff IS NOT NULL THEN
    RAISE EXCEPTION 'GAGAL T-05 (c) role/atribut: %', diff;
  END IF;
  RAISE NOTICE 'LULUS T-05 (c) 17 role dengan atribut dan batas koneksi sesuai skema fisik (tanpa BYPASSRLS)';
END $$;

-- T-05: keanggotaan persis.
DO $$
DECLARE
  diff text;
BEGIN
  WITH proj(rolname) AS (VALUES
      ('rme_app'), ('rme_auth'), ('rme_worker'), ('rme_operator'), ('rme_audit_relay'),
      ('rme_privacy_worker'), ('rme_key_rotation'), ('rme_migration'), ('rme_ops_monitor'),
      ('rme_api'), ('rme_api_auth'), ('rme_api_operator'), ('rme_worker_login'),
      ('rme_audit_relay_login'), ('rme_privacy_worker_login'), ('rme_metrics'), ('openbao_rme_admin')
  ), expected(grp, member, adm, inh, setopt) AS (VALUES
      ('rme_app', 'rme_api', false, true, false),
      ('rme_auth', 'rme_api_auth', false, true, false),
      ('rme_operator', 'rme_api_operator', false, true, false),
      ('rme_worker', 'rme_worker_login', false, true, false),
      ('rme_audit_relay', 'rme_audit_relay_login', false, true, false),
      ('rme_privacy_worker', 'rme_privacy_worker_login', false, true, false),
      ('rme_ops_monitor', 'rme_metrics', false, true, false),
      ('pg_monitor', 'rme_ops_monitor', false, true, false),
      ('rme_migration', 'openbao_rme_admin', true, false, false),
      ('rme_key_rotation', 'openbao_rme_admin', true, false, false),
      ('rme_ops_monitor', 'openbao_rme_admin', true, false, false)
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
    RAISE EXCEPTION 'GAGAL T-05 (c) keanggotaan: %', diff;
  END IF;
  RAISE NOTICE 'LULUS T-05 (c) keanggotaan dan opsi INHERIT/SET/ADMIN persis (satu grup per login)';
END $$;

-- T-03: ACL database rme persis; pemilik superuser.
DO $$
DECLARE
  diff text;
BEGIN
  WITH expected(grantee, priv) AS (VALUES
      ('rme_app', 'CONNECT'), ('rme_auth', 'CONNECT'), ('rme_worker', 'CONNECT'), ('rme_operator', 'CONNECT'),
      ('rme_audit_relay', 'CONNECT'), ('rme_privacy_worker', 'CONNECT'), ('rme_key_rotation', 'CONNECT'),
      ('rme_migration', 'CONNECT'), ('rme_ops_monitor', 'CONNECT'), ('openbao_rme_admin', 'CONNECT')
  ), actual AS (
    SELECT CASE WHEN a.grantee = 0 THEN 'PUBLIC' ELSE pg_catalog.pg_get_userbyid(a.grantee)::text END, a.privilege_type::text
      FROM pg_catalog.pg_database d, pg_catalog.aclexplode(d.datacl) a
     WHERE d.datname = 'rme' AND a.grantee <> d.datdba
  )
  SELECT string_agg(x, '; ') INTO diff FROM (
    SELECT format('kurang: %s', e) AS x FROM (SELECT * FROM expected EXCEPT SELECT * FROM actual) e
    UNION ALL
    SELECT format('lebih: %s', l) FROM (SELECT * FROM actual EXCEPT SELECT * FROM expected) l
  ) d;
  IF diff IS NOT NULL THEN
    RAISE EXCEPTION 'GAGAL T-03 (c) ACL database rme: %', diff;
  END IF;
  IF EXISTS (SELECT 1 FROM pg_catalog.pg_database d JOIN pg_catalog.pg_roles r ON r.oid = d.datdba
              WHERE d.datname = 'rme' AND (d.datacl IS NULL OR NOT r.rolsuper)) THEN
    RAISE EXCEPTION 'GAGAL §2.1 (c) datacl NULL atau pemilik rme bukan superuser';
  END IF;
  IF EXISTS (SELECT 1 FROM pg_catalog.pg_roles r
              WHERE r.rolname ~ '^(rme|openbao_rme)_' AND NOT r.rolsuper
                AND (pg_catalog.has_database_privilege(r.oid, 'rme', 'TEMPORARY')
                     OR pg_catalog.has_database_privilege(r.oid, 'postgres', 'CONNECT'))) THEN
    RAISE EXCEPTION 'GAGAL T-03 (c) role rme_* punya TEMP di rme atau CONNECT ke postgres';
  END IF;
  RAISE NOTICE 'LULUS T-03 (c) rme: PUBLIC tanpa CONNECT/TEMP, CONNECT hanya grup yang berhak, tanpa TEMP; pemilik superuser';
END $$;

-- §3.5 butir 4: pengaturan database dan per role login persis.
DO $$
DECLARE
  diff text;
BEGIN
  WITH expected(datname, rolname, cfg) AS (
    SELECT 'rme', '-', c FROM unnest(ARRAY['search_path=app, ext', 'TimeZone=UTC',
                                           'idle_in_transaction_session_timeout=60s']) c
    UNION ALL
    SELECT 'rme', r, c FROM (VALUES
        ('rme_api', ARRAY['search_path=app, ext', 'statement_timeout=3s', 'lock_timeout=2s', 'idle_in_transaction_session_timeout=30s']),
        ('rme_api_auth', ARRAY['search_path=app, ext', 'statement_timeout=3s', 'lock_timeout=2s', 'idle_in_transaction_session_timeout=30s']),
        ('rme_api_operator', ARRAY['search_path=app, ext', 'statement_timeout=3s', 'lock_timeout=2s', 'idle_in_transaction_session_timeout=30s']),
        ('rme_worker_login', ARRAY['search_path=app, ext', 'statement_timeout=60s', 'lock_timeout=5s', 'idle_in_transaction_session_timeout=60s']),
        ('rme_privacy_worker_login', ARRAY['search_path=app, ext', 'statement_timeout=300s', 'lock_timeout=5s', 'idle_in_transaction_session_timeout=60s']),
        ('rme_audit_relay_login', ARRAY['search_path=app, ext', 'statement_timeout=30s', 'lock_timeout=5s', 'idle_in_transaction_session_timeout=30s']),
        ('rme_metrics', ARRAY['search_path=app, ext'])
      ) v(r, cs), unnest(cs) c
    UNION ALL
    SELECT '-', 'openbao_rme_admin', c FROM unnest(ARRAY['log_min_error_statement=panic', 'log_statement=none']) c
  ), actual AS (
    SELECT coalesce(d.datname::text, '-'), coalesce(r.rolname::text, '-'), c
      FROM pg_catalog.pg_db_role_setting s
      LEFT JOIN pg_catalog.pg_database d ON d.oid = s.setdatabase
      LEFT JOIN pg_catalog.pg_roles r ON r.oid = s.setrole,
           unnest(s.setconfig) c
     WHERE d.datname = 'rme'
        OR r.rolname IN ('rme_api', 'rme_api_auth', 'rme_api_operator', 'rme_worker_login',
                         'rme_privacy_worker_login', 'rme_audit_relay_login', 'rme_metrics', 'openbao_rme_admin')
  )
  SELECT string_agg(x, '; ') INTO diff FROM (
    SELECT format('kurang: %s', e) AS x FROM (SELECT * FROM expected EXCEPT SELECT * FROM actual) e
    UNION ALL
    SELECT format('lebih: %s', l) FROM (SELECT * FROM actual EXCEPT SELECT * FROM expected) l
  ) d;
  IF diff IS NOT NULL THEN
    RAISE EXCEPTION 'GAGAL §3.5 (c) pengaturan: %', diff;
  END IF;
  RAISE NOTICE 'LULUS §3.5 (c) search_path, TimeZone, batas waktu §10.2, dan log role admin OpenBao persis';
END $$;

-- ---------------------------------------------------------------------------------------------
-- B. Katalog database rme (sebagai superuser)
-- ---------------------------------------------------------------------------------------------
\connect rme :su
DO $$
DECLARE
  diff text;
BEGIN
  WITH expected(kind, a, b, c) AS (VALUES
      ('schema', 'app', 'rme_migration', '-'), ('schema', 'util', 'rme_migration', '-'),
      ('schema', 'ext', 'superuser', '-'),
      ('acl', 'app', 'rme_app', 'USAGE'), ('acl', 'app', 'rme_auth', 'USAGE'), ('acl', 'app', 'rme_worker', 'USAGE'),
      ('acl', 'app', 'rme_operator', 'USAGE'), ('acl', 'app', 'rme_audit_relay', 'USAGE'),
      ('acl', 'app', 'rme_privacy_worker', 'USAGE'), ('acl', 'app', 'rme_key_rotation', 'USAGE'),
      ('acl', 'ext', 'rme_app', 'USAGE'), ('acl', 'ext', 'rme_migration', 'USAGE'),
      ('ext', 'plpgsql', 'pg_catalog', '-'), ('ext', 'pg_trgm', 'ext', '-')
  ), actual AS (
    SELECT 'schema', n.nspname::text, CASE WHEN r.rolsuper THEN 'superuser' ELSE r.rolname::text END, '-'
      FROM pg_catalog.pg_namespace n JOIN pg_catalog.pg_roles r ON r.oid = n.nspowner
     WHERE n.nspname !~ '^pg_' AND n.nspname <> 'information_schema'
    UNION ALL
    SELECT 'acl', n.nspname::text, CASE WHEN x.grantee = 0 THEN 'PUBLIC' ELSE pg_catalog.pg_get_userbyid(x.grantee)::text END,
           x.privilege_type::text
      FROM pg_catalog.pg_namespace n, pg_catalog.aclexplode(n.nspacl) x
     WHERE n.nspname IN ('app', 'util', 'ext') AND x.grantee <> n.nspowner
    UNION ALL
    SELECT 'ext', e.extname::text, n.nspname::text, '-'
      FROM pg_catalog.pg_extension e JOIN pg_catalog.pg_namespace n ON n.oid = e.extnamespace
  )
  SELECT string_agg(x, '; ') INTO diff FROM (
    SELECT format('kurang: %s', e) AS x FROM (SELECT * FROM expected EXCEPT SELECT * FROM actual) e
    UNION ALL
    SELECT format('lebih: %s', l) FROM (SELECT * FROM actual EXCEPT SELECT * FROM expected) l
  ) d;
  IF diff IS NOT NULL THEN
    RAISE EXCEPTION 'GAGAL T-02 (rme) skema/ACL/ekstensi: %', diff;
  END IF;
  IF NOT EXISTS (SELECT 1 FROM pg_catalog.pg_default_acl d
                  WHERE d.defaclrole = 'rme_migration'::regrole AND d.defaclobjtype = 'f' AND d.defaclnamespace = 0
                    AND NOT EXISTS (SELECT 1 FROM pg_catalog.aclexplode(d.defaclacl) a WHERE a.grantee = 0)) THEN
    RAISE EXCEPTION 'GAGAL §2.1 butir 6 (rme) default privileges EXECUTE PUBLIC belum dicabut';
  END IF;
  RAISE NOTICE 'LULUS T-02 (rme) skema app/util (rme_migration), ext (superuser), public tidak ada; ACL skema persis; pg_trgm di ext; default privileges';
END $$;

-- ---------------------------------------------------------------------------------------------
-- C. Kredensial dinamis tiruan (§3.4, §11.1) dan test negatif
-- ---------------------------------------------------------------------------------------------
\connect rme openbao_rme_admin
SET client_min_messages = warning;
DROP ROLE IF EXISTS "v-uji-rme-migration";
SET client_min_messages = notice;
DO $$
DECLARE
  c record;
  st text;
BEGIN
  FOR c IN SELECT * FROM (VALUES
      ('NEG openbao_rme_admin tidak dapat membuat anggota rme_app (tanpa ADMIN)', 'CREATE ROLE uji_neg_a IN ROLE rme_app', '42501'),
      ('NEG openbao_rme_admin tidak dapat membuat anggota rme_operator', 'CREATE ROLE uji_neg_b IN ROLE rme_operator', '42501'),
      ('NEG openbao_rme_admin tidak dapat membuat role BYPASSRLS', 'CREATE ROLE uji_neg_c BYPASSRLS', '42501'),
      ('NEG openbao_rme_admin tidak dapat SET ROLE ke rme_migration', 'SET ROLE rme_migration', '42501'),
      ('NEG openbao_rme_admin tidak mewarisi hak pemilik: CREATE TABLE app ditolak', 'CREATE TABLE app.uji_neg (x int)', '42501')
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
  EXECUTE format('CREATE ROLE %I WITH LOGIN PASSWORD %L VALID UNTIL %L '
                 'NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS IN ROLE rme_migration',
                 'v-uji-rme-migration', md5(random()::text || clock_timestamp()::text),
                 (now() + interval '30 minutes')::text);
  SET LOCAL client_min_messages = warning;
  EXECUTE format('ALTER ROLE %I SET role = %L', 'v-uji-rme-migration', 'rme_migration');
  EXECUTE format('ALTER ROLE %I SET search_path = app', 'v-uji-rme-migration');
  SET LOCAL client_min_messages = notice;
  RAISE NOTICE 'LULUS §3.4 openbao_rme_admin menjalankan creation_statements rme-migration';
END $$;

\connect rme v-uji-rme-migration
DO $$
BEGIN
  IF current_user <> 'rme_migration' OR current_setting('search_path') <> 'app' THEN
    RAISE EXCEPTION 'GAGAL §3.4 sesi dinamis rme: current_user=% search_path=%', current_user, current_setting('search_path');
  END IF;
  CREATE TABLE uji_probe (x text);
  CREATE INDEX ix_uji_probe_x_trgm ON app.uji_probe USING gin (x ext.gin_trgm_ops);
  CREATE TABLE util.uji_probe (x integer);
  CREATE FUNCTION util.fn_uji_probe() RETURNS integer LANGUAGE sql AS 'SELECT 1';
  RAISE NOTICE 'LULUS §3.4 sesi dinamis berjalan sebagai rme_migration; tabel/index trigram/fungsi di app dan util';
END $$;

\connect rme :su
DO $$
BEGIN
  IF EXISTS (SELECT 1 FROM pg_catalog.pg_class c JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
              WHERE n.nspname IN ('app', 'util') AND c.relowner <> 'rme_migration'::regrole)
     OR EXISTS (SELECT 1 FROM pg_catalog.pg_proc p JOIN pg_catalog.pg_namespace n ON n.oid = p.pronamespace
                 WHERE n.nspname IN ('app', 'util')
                   AND (p.proowner <> 'rme_migration'::regrole OR p.proacl IS NULL
                        OR EXISTS (SELECT 1 FROM pg_catalog.aclexplode(p.proacl) a WHERE a.grantee = 0))) THEN
    RAISE EXCEPTION 'GAGAL T-04/T-03 (rme) objek bukan milik rme_migration atau fungsi dapat dieksekusi PUBLIC';
  END IF;
  RAISE NOTICE 'LULUS T-04/T-03 (rme) objek buatan role dinamis dimiliki rme_migration; fungsi tanpa EXECUTE PUBLIC';
END $$;

-- Login sungguhan setiap role login RME + test negatif.
\connect rme rme_api
DO $$
DECLARE
  c record;
  st text;
BEGIN
  IF current_setting('search_path') <> 'app, ext' OR current_setting('statement_timeout') <> '3s'
     OR current_setting('TimeZone') <> 'UTC' THEN
    RAISE EXCEPTION 'GAGAL §10.2 sesi rme_api';
  END IF;
  RAISE NOTICE 'LULUS §10.2 sesi rme_api: search_path app, ext; 3s; UTC';
  FOR c IN SELECT * FROM (VALUES
      ('NEG rme_api tidak dapat membuat objek di app', 'CREATE TABLE app.uji_neg (x int)', '42501'),
      ('NEG rme_api tidak dapat membuat objek di util', 'CREATE TABLE util.uji_neg (x int)', '42501'),
      ('NEG rme_api tidak dapat membuat objek di ext', 'CREATE TABLE ext.uji_neg (x int)', '42501'),
      ('NEG rme_api: skema public tidak ada', 'CREATE TABLE public.uji_neg (x int)', '3F000'),
      ('NEG rme_api tidak dapat membuat tabel temporer', 'CREATE TEMP TABLE uji_neg (x int)', '42501'),
      ('NEG rme_api tidak dapat CREATE SCHEMA', 'CREATE SCHEMA uji_neg', '42501'),
      ('NEG rme_api tidak dapat SET ROLE ke rme_app (SET FALSE)', 'SET ROLE rme_app', '42501'),
      ('NEG rme_api tidak dapat SET ROLE ke rme_operator', 'SET ROLE rme_operator', '42501'),
      ('NEG rme_api tidak dapat SET ROLE ke rme_auth', 'SET ROLE rme_auth', '42501'),
      ('NEG rme_api tidak dapat membaca tabel app tanpa GRANT', 'SELECT count(*) FROM app.uji_probe', '42501'),
      ('NEG rme_api tidak dapat membaca skema util', 'SELECT count(*) FROM util.uji_probe', '42501'),
      ('NEG rme_api tidak dapat mengeksekusi fungsi util', 'SELECT util.fn_uji_probe()', '42501'),
      ('POS rme_api dapat memakai fungsi pg_trgm lewat search_path ext', 'SELECT similarity(''rekam'', ''rekaman'')', NULL)
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

\connect rme rme_api_auth
DO $$
DECLARE
  c record;
  st text;
BEGIN
  FOR c IN SELECT * FROM (VALUES
      ('NEG rme_api_auth tidak dapat SET ROLE ke rme_app', 'SET ROLE rme_app', '42501'),
      ('NEG rme_api_auth tidak dapat memakai fungsi pg_trgm (tanpa USAGE ext)', 'SELECT ext.similarity(''a'', ''b'')', '42501'),
      ('NEG rme_api_auth tidak dapat membaca tabel app tanpa GRANT', 'SELECT count(*) FROM app.uji_probe', '42501')
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
  IF current_setting('statement_timeout') <> '3s' OR pg_catalog.pg_has_role('rme_app', 'USAGE') THEN
    RAISE EXCEPTION 'GAGAL §10.2 sesi rme_api_auth';
  END IF;
  RAISE NOTICE 'LULUS §10.2 sesi rme_api_auth (3s), bukan anggota rme_app';
END $$;

\connect rme rme_api_operator
DO $$ BEGIN
  IF current_setting('statement_timeout') <> '3s' OR pg_catalog.pg_has_role('rme_app', 'USAGE')
     OR NOT pg_catalog.pg_has_role('rme_operator', 'USAGE') THEN
    RAISE EXCEPTION 'GAGAL §10.2 sesi rme_api_operator'; END IF;
  RAISE NOTICE 'LULUS §10.2 sesi rme_api_operator (3s), hanya anggota rme_operator';
END $$;
\connect rme rme_worker_login
DO $$ BEGIN
  IF current_setting('statement_timeout') NOT IN ('1min', '60s') OR pg_catalog.pg_has_role('rme_app', 'USAGE') THEN
    RAISE EXCEPTION 'GAGAL §10.2 sesi rme_worker_login: %', current_setting('statement_timeout'); END IF;
  RAISE NOTICE 'LULUS §10.2 sesi rme_worker_login (60s), bukan anggota rme_app';
END $$;
\connect rme rme_privacy_worker_login
DO $$ BEGIN
  IF current_setting('statement_timeout') NOT IN ('5min', '300s') OR pg_catalog.pg_has_role('rme_key_rotation', 'USAGE') THEN
    RAISE EXCEPTION 'GAGAL §10.2 sesi rme_privacy_worker_login: %', current_setting('statement_timeout'); END IF;
  RAISE NOTICE 'LULUS §10.2 sesi rme_privacy_worker_login (300s), tanpa rme_key_rotation';
END $$;
\connect rme rme_audit_relay_login
DO $$ BEGIN
  IF current_setting('statement_timeout') <> '30s' THEN
    RAISE EXCEPTION 'GAGAL §10.2 sesi rme_audit_relay_login'; END IF;
  RAISE NOTICE 'LULUS §10.2 sesi rme_audit_relay_login (30s)';
END $$;
\connect rme rme_metrics
DO $$ BEGIN
  IF NOT pg_catalog.pg_has_role('pg_monitor', 'USAGE') OR has_schema_privilege('app', 'USAGE') THEN
    RAISE EXCEPTION 'GAGAL §3.2 rme_metrics'; END IF;
  RAISE NOTICE 'LULUS §3.2 rme_metrics mewarisi pg_monitor, tanpa USAGE skema app';
END $$;

-- Pembersihan.
\connect rme v-uji-rme-migration
DROP FUNCTION util.fn_uji_probe();
DROP TABLE util.uji_probe;
DROP TABLE app.uji_probe;
\connect rme openbao_rme_admin
DO $$ BEGIN
  DROP ROLE "v-uji-rme-migration";
  RAISE NOTICE 'LULUS §3.4 revocation_statements rme-migration: DROP ROLE berhasil (tidak memiliki objek)';
END $$;
\connect postgres :su
DO $$
BEGIN
  IF EXISTS (SELECT 1 FROM pg_catalog.pg_roles WHERE rolname ~ '^v-uji-') THEN
    RAISE EXCEPTION 'GAGAL sisa role dinamis uji';
  END IF;
  RAISE NOTICE 'LULUS pembersihan (c): tidak ada role dinamis tersisa';
END $$;
