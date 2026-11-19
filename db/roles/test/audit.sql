-- db/roles/test/audit.sql: test katalog dan test negatif bootstrap klaster (b) (db/roles/audit.sql).
-- Dijalankan scripts/db-roles-check.sh dengan psql di dalam kontainer PostgreSQL lokal/CI, sebagai
-- superuser di database postgres. Pindah role memakai \connect lewat soket lokal (trust di image
-- resmi postgres; HANYA lingkungan lokal/CI, bukan production).
-- Dasar: docs/05-physical-schema.md §2.1, §3.1–§3.5, §6.5 butir 5, §10.2, §11.1, §12 (T-02–T-05, T-21).
-- Setiap pemeriksaan yang lulus mencetak "LULUS <label>"; yang gagal menghentikan skrip.

\set ON_ERROR_STOP on
SET search_path = pg_catalog, pg_temp;
SELECT current_user AS su \gset

-- ---------------------------------------------------------------------------------------------
-- A. Katalog tingkat klaster (sebagai superuser)
-- ---------------------------------------------------------------------------------------------

-- T-05: role dan atribut persis §3.2 (b); rme_audit_reader_login tidak ada (tanpa proses pemegang).
DO $$
DECLARE
  diff text;
BEGIN
  WITH expected(rolname, can_login, conn_limit, can_createrole) AS (VALUES
      ('rs_audit_writer', false, -1, false), ('rs_audit_reader', false, -1, false),
      ('rs_audit_verifier', false, -1, false), ('rs_audit_migration', false, -1, false),
      ('rs_audit_writer_login', true, 6, false), ('rs_audit_reader_login', true, 10, false),
      ('rs_audit_verifier_login', true, 4, false), ('openbao_rs_audit_admin', true, -1, true),
      ('rme_audit_writer', false, -1, false), ('rme_audit_reader', false, -1, false),
      ('rme_audit_verifier', false, -1, false), ('rme_audit_tenant_reader', false, -1, false),
      ('rme_audit_migration', false, -1, false), ('rme_audit_writer_login', true, 6, false),
      ('rme_audit_verifier_login', true, 4, false), ('rme_audit_tenant_reader_login', true, 6, false),
      ('openbao_rme_audit_admin', true, -1, true)
  ), actual AS (
    SELECT r.rolname::text, r.rolcanlogin, r.rolconnlimit, r.rolcreaterole
      FROM pg_catalog.pg_roles r JOIN expected e ON e.rolname = r.rolname
     WHERE NOT r.rolsuper AND NOT r.rolcreatedb AND NOT r.rolreplication AND NOT r.rolbypassrls
       AND r.rolinherit
  )
  SELECT string_agg(format('kurang/beda: %s', e), '; ') INTO diff
    FROM (SELECT * FROM expected EXCEPT SELECT * FROM actual) e;
  IF diff IS NOT NULL THEN
    RAISE EXCEPTION 'GAGAL T-05 (b) role/atribut: %', diff;
  END IF;
  IF EXISTS (SELECT 1 FROM pg_catalog.pg_roles WHERE rolname = 'rme_audit_reader_login') THEN
    RAISE EXCEPTION 'GAGAL T-05 (b) rme_audit_reader_login ada padahal tanpa proses pemegang';
  END IF;
  RAISE NOTICE 'LULUS T-05 (b) 17 role dengan atribut dan batas koneksi sesuai skema fisik; rme_audit_reader_login tidak ada';
END $$;

-- T-05: keanggotaan persis.
DO $$
DECLARE
  diff text;
BEGIN
  WITH proj(rolname) AS (VALUES
      ('rs_audit_writer'), ('rs_audit_reader'), ('rs_audit_verifier'), ('rs_audit_migration'),
      ('rs_audit_writer_login'), ('rs_audit_reader_login'), ('rs_audit_verifier_login'),
      ('openbao_rs_audit_admin'), ('rme_audit_writer'), ('rme_audit_reader'), ('rme_audit_verifier'),
      ('rme_audit_tenant_reader'), ('rme_audit_migration'), ('rme_audit_writer_login'),
      ('rme_audit_verifier_login'), ('rme_audit_tenant_reader_login'), ('openbao_rme_audit_admin')
  ), expected(grp, member, adm, inh, setopt) AS (VALUES
      ('rs_audit_writer', 'rs_audit_writer_login', false, true, false),
      ('rs_audit_reader', 'rs_audit_reader_login', false, true, false),
      ('rs_audit_verifier', 'rs_audit_verifier_login', false, true, false),
      ('rme_audit_writer', 'rme_audit_writer_login', false, true, false),
      ('rme_audit_verifier', 'rme_audit_verifier_login', false, true, false),
      ('rme_audit_tenant_reader', 'rme_audit_tenant_reader_login', false, true, false),
      ('rs_audit_migration', 'openbao_rs_audit_admin', true, false, false),
      ('rme_audit_migration', 'openbao_rme_audit_admin', true, false, false)
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
    RAISE EXCEPTION 'GAGAL T-05 (b) keanggotaan: %', diff;
  END IF;
  RAISE NOTICE 'LULUS T-05 (b) keanggotaan dan opsi INHERIT/SET/ADMIN persis';
END $$;

-- T-03 dan T-21: ACL database persis; rs_audit_* hanya ke ruang_sehat_audit, rme_audit_* hanya ke rme_audit.
DO $$
DECLARE
  diff text;
BEGIN
  WITH expected(datname, grantee, priv) AS (VALUES
      ('ruang_sehat_audit', 'rs_audit_writer', 'CONNECT'), ('ruang_sehat_audit', 'rs_audit_reader', 'CONNECT'),
      ('ruang_sehat_audit', 'rs_audit_verifier', 'CONNECT'), ('ruang_sehat_audit', 'rs_audit_migration', 'CONNECT'),
      ('ruang_sehat_audit', 'openbao_rs_audit_admin', 'CONNECT'),
      ('rme_audit', 'rme_audit_writer', 'CONNECT'), ('rme_audit', 'rme_audit_reader', 'CONNECT'),
      ('rme_audit', 'rme_audit_verifier', 'CONNECT'), ('rme_audit', 'rme_audit_tenant_reader', 'CONNECT'),
      ('rme_audit', 'rme_audit_migration', 'CONNECT'), ('rme_audit', 'openbao_rme_audit_admin', 'CONNECT')
  ), actual AS (
    SELECT d.datname::text, CASE WHEN a.grantee = 0 THEN 'PUBLIC' ELSE pg_catalog.pg_get_userbyid(a.grantee)::text END,
           a.privilege_type::text
      FROM pg_catalog.pg_database d, pg_catalog.aclexplode(d.datacl) a
     WHERE d.datname IN ('ruang_sehat_audit', 'rme_audit') AND a.grantee <> d.datdba
  )
  SELECT string_agg(x, '; ') INTO diff FROM (
    SELECT format('kurang: %s', e) AS x FROM (SELECT * FROM expected EXCEPT SELECT * FROM actual) e
    UNION ALL
    SELECT format('lebih: %s', l) FROM (SELECT * FROM actual EXCEPT SELECT * FROM expected) l
  ) d;
  IF diff IS NOT NULL THEN
    RAISE EXCEPTION 'GAGAL T-03/T-21 (b) ACL database: %', diff;
  END IF;
  IF EXISTS (SELECT 1 FROM pg_catalog.pg_database d JOIN pg_catalog.pg_roles r ON r.oid = d.datdba
              WHERE d.datname IN ('ruang_sehat_audit', 'rme_audit') AND (d.datacl IS NULL OR NOT r.rolsuper)) THEN
    RAISE EXCEPTION 'GAGAL §2.1 (b) datacl NULL atau pemilik database audit bukan superuser';
  END IF;
  RAISE NOTICE 'LULUS T-03/T-21 (b) PUBLIC tanpa CONNECT/TEMP; rs_audit_* hanya ruang_sehat_audit, rme_audit_* hanya rme_audit; pemilik superuser';
END $$;

-- T-21 (hak efektif lewat keanggotaan): login audit tidak dapat memasuki database audit sistem lain.
DO $$
DECLARE
  bad text;
BEGIN
  SELECT string_agg(format('%s->%s', r.rolname, v.db), ', ') INTO bad
    FROM pg_catalog.pg_roles r,
         (VALUES ('^(rs_audit|openbao_rs_audit)_', 'rme_audit'), ('^(rme_audit|openbao_rme_audit)_', 'ruang_sehat_audit'),
                 ('^(rs|rme|openbao)_', 'postgres')) v(pat, db)
   WHERE r.rolname ~ v.pat AND pg_catalog.has_database_privilege(r.oid, v.db, 'CONNECT');
  IF bad IS NULL THEN
    SELECT string_agg(format('%s->%s:TEMP', r.rolname, d), ', ') INTO bad
      FROM pg_catalog.pg_roles r, unnest(ARRAY['ruang_sehat_audit', 'rme_audit']) d
     WHERE r.rolname ~ '^(rs|rme|openbao)_' AND NOT r.rolsuper
       AND pg_catalog.has_database_privilege(r.oid, d, 'TEMPORARY');
  END IF;
  IF bad IS NOT NULL THEN
    RAISE EXCEPTION 'GAGAL T-21 (b) hak database lintas sistem: %', bad;
  END IF;
  RAISE NOTICE 'LULUS T-21 (b) hak efektif: tidak ada CONNECT lintas database audit, tidak ada TEMP';
END $$;

-- §3.5 butir 4: pengaturan database dan per role login persis.
DO $$
DECLARE
  diff text;
BEGIN
  WITH expected(datname, rolname, cfg) AS (
    SELECT d, '-', c FROM unnest(ARRAY['ruang_sehat_audit', 'rme_audit']) d,
                          unnest(ARRAY['search_path=audit', 'TimeZone=UTC', 'idle_in_transaction_session_timeout=60s']) c
    UNION ALL
    SELECT d, r, c FROM (VALUES
        ('ruang_sehat_audit', 'rs_audit_writer_login', ARRAY['search_path=audit', 'statement_timeout=30s', 'lock_timeout=5s', 'idle_in_transaction_session_timeout=30s']),
        ('ruang_sehat_audit', 'rs_audit_reader_login', ARRAY['search_path=audit', 'statement_timeout=30s', 'lock_timeout=2s', 'idle_in_transaction_session_timeout=30s']),
        ('ruang_sehat_audit', 'rs_audit_verifier_login', ARRAY['search_path=audit', 'statement_timeout=0', 'lock_timeout=5s', 'idle_in_transaction_session_timeout=60s']),
        ('rme_audit', 'rme_audit_writer_login', ARRAY['search_path=audit', 'statement_timeout=30s', 'lock_timeout=5s', 'idle_in_transaction_session_timeout=30s']),
        ('rme_audit', 'rme_audit_tenant_reader_login', ARRAY['search_path=audit', 'statement_timeout=30s', 'lock_timeout=2s', 'idle_in_transaction_session_timeout=30s']),
        ('rme_audit', 'rme_audit_verifier_login', ARRAY['search_path=audit', 'statement_timeout=0', 'lock_timeout=5s', 'idle_in_transaction_session_timeout=60s'])
      ) v(d, r, cs), unnest(cs) c
    UNION ALL
    SELECT '-', r, c FROM unnest(ARRAY['openbao_rs_audit_admin', 'openbao_rme_audit_admin']) r,
                          unnest(ARRAY['log_min_error_statement=panic', 'log_statement=none', 'log_min_duration_statement=-1',
                         'log_min_duration_sample=-1', 'createrole_self_grant=']) c
  ), actual AS (
    SELECT coalesce(d.datname::text, '-'), coalesce(r.rolname::text, '-'), c
      FROM pg_catalog.pg_db_role_setting s
      LEFT JOIN pg_catalog.pg_database d ON d.oid = s.setdatabase
      LEFT JOIN pg_catalog.pg_roles r ON r.oid = s.setrole,
           unnest(s.setconfig) c
     WHERE d.datname IN ('ruang_sehat_audit', 'rme_audit')
        OR r.rolname ~ '^(rs_audit_(writer|reader|verifier|migration)|rme_audit_(writer|reader|verifier|tenant_reader|migration)|openbao_(rs|rme)_audit_admin)'
  )
  SELECT string_agg(x, '; ') INTO diff FROM (
    SELECT format('kurang: %s', e) AS x FROM (SELECT * FROM expected EXCEPT SELECT * FROM actual) e
    UNION ALL
    SELECT format('lebih: %s', l) FROM (SELECT * FROM actual EXCEPT SELECT * FROM expected) l
  ) d;
  IF diff IS NOT NULL THEN
    RAISE EXCEPTION 'GAGAL §3.5 (b) pengaturan: %', diff;
  END IF;
  RAISE NOTICE 'LULUS §3.5 (b) search_path audit, TimeZone, batas waktu §10.2, dan log role admin OpenBao persis';
END $$;

-- ---------------------------------------------------------------------------------------------
-- B. Katalog per database audit (sebagai superuser)
-- ---------------------------------------------------------------------------------------------
\connect ruang_sehat_audit :su
SET search_path = pg_catalog, pg_temp;
DO $$
DECLARE
  diff text;
BEGIN
  WITH expected(kind, a, b, c) AS (VALUES
      ('schema', 'audit', 'rs_audit_migration', '-'), ('schema', 'util', 'rs_audit_migration', '-'),
      ('schema', 'audit_detached', 'rs_audit_migration', '-'),
      ('acl', 'audit', 'rs_audit_writer', 'USAGE'), ('acl', 'audit', 'rs_audit_reader', 'USAGE'),
      ('acl', 'audit', 'rs_audit_verifier', 'USAGE'),
      ('ext', 'plpgsql', 'pg_catalog', '-')
  ), actual AS (
    SELECT 'schema', n.nspname::text, pg_catalog.pg_get_userbyid(n.nspowner)::text, '-'
      FROM pg_catalog.pg_namespace n WHERE n.nspname !~ '^pg_' AND n.nspname <> 'information_schema'
    UNION ALL
    SELECT 'acl', n.nspname::text, CASE WHEN x.grantee = 0 THEN 'PUBLIC' ELSE pg_catalog.pg_get_userbyid(x.grantee)::text END,
           x.privilege_type::text
      FROM pg_catalog.pg_namespace n, pg_catalog.aclexplode(n.nspacl) x
     WHERE n.nspname IN ('audit', 'util', 'audit_detached') AND x.grantee <> n.nspowner
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
    RAISE EXCEPTION 'GAGAL T-02 (ruang_sehat_audit) skema/ACL/ekstensi: %', diff;
  END IF;
  IF NOT EXISTS (SELECT 1 FROM pg_catalog.pg_default_acl d
                  WHERE d.defaclrole = 'rs_audit_migration'::regrole AND d.defaclobjtype = 'f' AND d.defaclnamespace = 0
                    AND NOT EXISTS (SELECT 1 FROM pg_catalog.aclexplode(d.defaclacl) a WHERE a.grantee = 0)) THEN
    RAISE EXCEPTION 'GAGAL §2.1 butir 6 (ruang_sehat_audit) default privileges EXECUTE PUBLIC belum dicabut';
  END IF;
  RAISE NOTICE 'LULUS T-02 (ruang_sehat_audit) skema audit/util/audit_detached milik rs_audit_migration; USAGE audit hanya writer/reader/verifier; audit_detached tanpa hak; tanpa ekstensi; default privileges';
END $$;

\connect rme_audit :su
SET search_path = pg_catalog, pg_temp;
DO $$
DECLARE
  diff text;
BEGIN
  WITH expected(kind, a, b, c) AS (VALUES
      ('schema', 'audit', 'rme_audit_migration', '-'), ('schema', 'util', 'rme_audit_migration', '-'),
      ('schema', 'audit_detached', 'rme_audit_migration', '-'),
      ('acl', 'audit', 'rme_audit_writer', 'USAGE'), ('acl', 'audit', 'rme_audit_reader', 'USAGE'),
      ('acl', 'audit', 'rme_audit_verifier', 'USAGE'), ('acl', 'audit', 'rme_audit_tenant_reader', 'USAGE'),
      ('ext', 'plpgsql', 'pg_catalog', '-')
  ), actual AS (
    SELECT 'schema', n.nspname::text, pg_catalog.pg_get_userbyid(n.nspowner)::text, '-'
      FROM pg_catalog.pg_namespace n WHERE n.nspname !~ '^pg_' AND n.nspname <> 'information_schema'
    UNION ALL
    SELECT 'acl', n.nspname::text, CASE WHEN x.grantee = 0 THEN 'PUBLIC' ELSE pg_catalog.pg_get_userbyid(x.grantee)::text END,
           x.privilege_type::text
      FROM pg_catalog.pg_namespace n, pg_catalog.aclexplode(n.nspacl) x
     WHERE n.nspname IN ('audit', 'util', 'audit_detached') AND x.grantee <> n.nspowner
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
    RAISE EXCEPTION 'GAGAL T-02 (rme_audit) skema/ACL/ekstensi: %', diff;
  END IF;
  IF NOT EXISTS (SELECT 1 FROM pg_catalog.pg_default_acl d
                  WHERE d.defaclrole = 'rme_audit_migration'::regrole AND d.defaclobjtype = 'f' AND d.defaclnamespace = 0
                    AND NOT EXISTS (SELECT 1 FROM pg_catalog.aclexplode(d.defaclacl) a WHERE a.grantee = 0)) THEN
    RAISE EXCEPTION 'GAGAL §2.1 butir 6 (rme_audit) default privileges EXECUTE PUBLIC belum dicabut';
  END IF;
  RAISE NOTICE 'LULUS T-02 (rme_audit) skema audit/util/audit_detached milik rme_audit_migration; USAGE audit hanya writer/reader/verifier/tenant_reader; tanpa ekstensi; default privileges';
END $$;

-- ---------------------------------------------------------------------------------------------
-- C. ruang_sehat_audit: kredensial dinamis tiruan + test negatif
-- ---------------------------------------------------------------------------------------------
\connect ruang_sehat_audit openbao_rs_audit_admin
SET client_min_messages = warning;
DROP ROLE IF EXISTS "v-uji-rs-audit-migration";
SET client_min_messages = notice;
DO $$
DECLARE
  c record;
  st text;
BEGIN
  FOR c IN SELECT * FROM (VALUES
      ('NEG openbao_rs_audit_admin tidak dapat membuat anggota rme_audit_migration (lintas sistem)', 'CREATE ROLE uji_neg_a IN ROLE rme_audit_migration', '42501'),
      ('NEG openbao_rs_audit_admin tidak dapat membuat anggota rs_audit_writer', 'CREATE ROLE uji_neg_b IN ROLE rs_audit_writer', '42501'),
      ('NEG openbao_rs_audit_admin tanpa self-grant tidak dapat SET ROLE ke rs_audit_migration (keadaan awal SET FALSE)', 'SET ROLE rs_audit_migration', '42501'),
      ('NEG openbao_rs_audit_admin tidak mewarisi hak pemilik: CREATE TABLE audit ditolak', 'CREATE TABLE audit.uji_neg (x int)', '42501')
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
                 'NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS IN ROLE rs_audit_migration',
                 'v-uji-rs-audit-migration', md5(random()::text || clock_timestamp()::text),
                 (now() + interval '30 minutes')::text);
  SET LOCAL client_min_messages = warning;
  EXECUTE format('ALTER ROLE %I SET role = %L', 'v-uji-rs-audit-migration', 'rs_audit_migration');
  EXECUTE format('ALTER ROLE %I SET search_path = audit', 'v-uji-rs-audit-migration');
  SET LOCAL client_min_messages = notice;
  RAISE NOTICE 'LULUS §3.4 openbao_rs_audit_admin menjalankan creation_statements rs-audit-migration';
END $$;

\connect ruang_sehat_audit v-uji-rs-audit-migration
DO $$
BEGIN
  IF current_user <> 'rs_audit_migration' OR current_setting('search_path') <> 'audit' THEN
    RAISE EXCEPTION 'GAGAL §3.4 sesi dinamis audit: current_user=% search_path=%', current_user, current_setting('search_path');
  END IF;
  CREATE TABLE uji_probe (x integer);
  CREATE TABLE util.uji_probe (x integer);
  CREATE TABLE audit_detached.uji_probe (x integer);
  RAISE NOTICE 'LULUS §3.4 sesi dinamis berjalan sebagai rs_audit_migration; objek tanpa skema masuk ke audit';
END $$;

\connect ruang_sehat_audit rs_audit_writer_login
DO $$
DECLARE
  c record;
  st text;
BEGIN
  IF current_setting('statement_timeout') <> '30s' OR current_setting('search_path') <> 'audit' THEN
    RAISE EXCEPTION 'GAGAL §10.2 sesi rs_audit_writer_login';
  END IF;
  RAISE NOTICE 'LULUS §10.2 sesi rs_audit_writer_login (30s, search_path audit)';
  FOR c IN SELECT * FROM (VALUES
      ('NEG rs_audit_writer_login tidak dapat membuat objek di audit', 'CREATE TABLE audit.uji_neg (x int)', '42501'),
      ('NEG rs_audit_writer_login tidak dapat membaca tabel audit tanpa GRANT', 'SELECT count(*) FROM audit.uji_probe', '42501'),
      ('NEG rs_audit_writer_login tidak dapat membaca audit_detached', 'SELECT count(*) FROM audit_detached.uji_probe', '42501'),
      ('NEG rs_audit_writer_login tidak dapat membaca util', 'SELECT count(*) FROM util.uji_probe', '42501'),
      ('NEG rs_audit_writer_login tidak dapat membuat tabel temporer', 'CREATE TEMP TABLE uji_neg (x int)', '42501'),
      ('NEG rs_audit_writer_login tidak dapat SET ROLE ke rs_audit_verifier', 'SET ROLE rs_audit_verifier', '42501'),
      ('NEG rs_audit_writer_login tidak dapat SET ROLE ke rs_audit_writer (SET FALSE)', 'SET ROLE rs_audit_writer', '42501'),
      ('NEG rs_audit_writer_login: skema public tidak ada', 'CREATE TABLE public.uji_neg (x int)', '3F000')
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

\connect ruang_sehat_audit rs_audit_reader_login
DO $$ BEGIN
  IF current_setting('lock_timeout') <> '2s' OR pg_catalog.pg_has_role('rs_audit_writer', 'USAGE') THEN
    RAISE EXCEPTION 'GAGAL §10.2 sesi rs_audit_reader_login'; END IF;
  RAISE NOTICE 'LULUS §10.2 sesi rs_audit_reader_login (lock 2s), bukan anggota writer';
END $$;
\connect ruang_sehat_audit rs_audit_verifier_login
DO $$ BEGIN
  IF current_setting('statement_timeout') <> '0' OR pg_catalog.pg_has_role('rs_audit_writer', 'USAGE') THEN
    RAISE EXCEPTION 'GAGAL §10.2 sesi rs_audit_verifier_login: %', current_setting('statement_timeout'); END IF;
  RAISE NOTICE 'LULUS §10.2 sesi rs_audit_verifier_login (tanpa batas statement), bukan anggota writer';
END $$;

\connect ruang_sehat_audit v-uji-rs-audit-migration
DROP TABLE audit.uji_probe;
DROP TABLE util.uji_probe;
DROP TABLE audit_detached.uji_probe;
\connect ruang_sehat_audit openbao_rs_audit_admin
DO $$ BEGIN
  DROP ROLE "v-uji-rs-audit-migration";
  RAISE NOTICE 'LULUS §3.4 revocation_statements rs-audit-migration: DROP ROLE berhasil';
END $$;

-- ---------------------------------------------------------------------------------------------
-- D. rme_audit: kredensial dinamis tiruan + test negatif
-- ---------------------------------------------------------------------------------------------
\connect rme_audit openbao_rme_audit_admin
SET client_min_messages = warning;
DROP ROLE IF EXISTS "v-uji-rme-audit-migration";
SET client_min_messages = notice;
DO $$
DECLARE
  c record;
  st text;
BEGIN
  FOR c IN SELECT * FROM (VALUES
      ('NEG openbao_rme_audit_admin tidak dapat membuat anggota rs_audit_migration (lintas sistem)', 'CREATE ROLE uji_neg_a IN ROLE rs_audit_migration', '42501'),
      ('NEG openbao_rme_audit_admin tidak dapat membuat anggota rme_audit_tenant_reader', 'CREATE ROLE uji_neg_b IN ROLE rme_audit_tenant_reader', '42501')
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
                 'NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS IN ROLE rme_audit_migration',
                 'v-uji-rme-audit-migration', md5(random()::text || clock_timestamp()::text),
                 (now() + interval '30 minutes')::text);
  SET LOCAL client_min_messages = warning;
  EXECUTE format('ALTER ROLE %I SET role = %L', 'v-uji-rme-audit-migration', 'rme_audit_migration');
  EXECUTE format('ALTER ROLE %I SET search_path = audit', 'v-uji-rme-audit-migration');
  SET LOCAL client_min_messages = notice;
  RAISE NOTICE 'LULUS §3.4 openbao_rme_audit_admin menjalankan creation_statements rme-audit-migration';
END $$;

\connect rme_audit v-uji-rme-audit-migration
DO $$
BEGIN
  IF current_user <> 'rme_audit_migration' OR current_setting('search_path') <> 'audit' THEN
    RAISE EXCEPTION 'GAGAL §3.4 sesi dinamis rme_audit: current_user=%', current_user;
  END IF;
  CREATE TABLE uji_probe (x integer);
  RAISE NOTICE 'LULUS §3.4 sesi dinamis berjalan sebagai rme_audit_migration; objek tanpa skema masuk ke audit';
END $$;

\connect rme_audit rme_audit_tenant_reader_login
DO $$
DECLARE
  c record;
  st text;
BEGIN
  IF current_setting('statement_timeout') <> '30s' OR current_setting('lock_timeout') <> '2s' THEN
    RAISE EXCEPTION 'GAGAL §10.2 sesi rme_audit_tenant_reader_login';
  END IF;
  RAISE NOTICE 'LULUS §10.2 sesi rme_audit_tenant_reader_login (30s/2s)';
  FOR c IN SELECT * FROM (VALUES
      ('NEG rme_audit_tenant_reader_login tidak dapat membuat objek di audit', 'CREATE TABLE audit.uji_neg (x int)', '42501'),
      ('NEG rme_audit_tenant_reader_login tidak dapat membaca tabel audit tanpa GRANT', 'SELECT count(*) FROM audit.uji_probe', '42501'),
      ('NEG rme_audit_tenant_reader_login tidak dapat SET ROLE ke rme_audit_reader', 'SET ROLE rme_audit_reader', '42501'),
      ('NEG rme_audit_tenant_reader_login tidak dapat membuat tabel temporer', 'CREATE TEMP TABLE uji_neg (x int)', '42501')
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
\connect rme_audit rme_audit_writer_login
DO $$ BEGIN
  IF current_setting('statement_timeout') <> '30s' OR pg_catalog.pg_has_role('rme_audit_verifier', 'USAGE') THEN
    RAISE EXCEPTION 'GAGAL §10.2 sesi rme_audit_writer_login'; END IF;
  RAISE NOTICE 'LULUS §10.2 sesi rme_audit_writer_login (30s), bukan anggota verifier';
END $$;
\connect rme_audit rme_audit_verifier_login
DO $$ BEGIN
  IF current_setting('statement_timeout') <> '0' OR pg_catalog.pg_has_role('rme_audit_writer', 'USAGE') THEN
    RAISE EXCEPTION 'GAGAL §10.2 sesi rme_audit_verifier_login'; END IF;
  RAISE NOTICE 'LULUS §10.2 sesi rme_audit_verifier_login (tanpa batas statement), bukan anggota writer';
END $$;

\connect rme_audit v-uji-rme-audit-migration
DROP TABLE audit.uji_probe;
\connect rme_audit openbao_rme_audit_admin
DO $$ BEGIN
  DROP ROLE "v-uji-rme-audit-migration";
  RAISE NOTICE 'LULUS §3.4 revocation_statements rme-audit-migration: DROP ROLE berhasil';
END $$;

\connect postgres :su
SET search_path = pg_catalog, pg_temp;
DO $$
BEGIN
  IF EXISTS (SELECT 1 FROM pg_catalog.pg_roles WHERE rolname ~ '^v-uji-') THEN
    RAISE EXCEPTION 'GAGAL sisa role dinamis uji';
  END IF;
  RAISE NOTICE 'LULUS pembersihan (b): tidak ada role dinamis tersisa';
END $$;
