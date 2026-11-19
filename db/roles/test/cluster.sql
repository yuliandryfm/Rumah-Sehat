-- db/roles/test/cluster.sql: pemeriksaan tingkat klaster setelah bootstrap db/roles/*.sql.
-- Dijalankan superuser (psql) sekali per klaster oleh scripts/db-roles-check.sh.
-- Dasar: docs/05-physical-schema.md §3.1 butir 3, §3.2 (paragraf T-05), §12 T-03 dan T-05.
-- Setiap pemeriksaan yang lulus mencetak "LULUS <label>"; yang gagal menghentikan skrip.

\set ON_ERROR_STOP on

-- T-05: tidak ada role non-superuser dengan BYPASSRLS, REPLICATION, atau CREATEDB.
DO $$
DECLARE
  bad text;
BEGIN
  SELECT string_agg(rolname, ', ' ORDER BY rolname) INTO bad
    FROM pg_catalog.pg_roles
   WHERE NOT rolsuper AND rolname !~ '^pg_'
     AND (rolbypassrls OR rolreplication OR rolcreatedb);
  IF bad IS NOT NULL THEN
    RAISE EXCEPTION 'GAGAL T-05 atribut terlarang pada: %', bad;
  END IF;
  RAISE NOTICE 'LULUS T-05 tidak ada role non-superuser dengan BYPASSRLS/REPLICATION/CREATEDB';
END $$;

-- T-05: role proyek tidak ada yang SUPERUSER.
DO $$
DECLARE
  bad text;
BEGIN
  SELECT string_agg(rolname, ', ' ORDER BY rolname) INTO bad
    FROM pg_catalog.pg_roles
   WHERE rolsuper AND (rolname ~ '^(rs|rme|openbao)_' OR rolname = 'keycloak');
  IF bad IS NOT NULL THEN
    RAISE EXCEPTION 'GAGAL T-05 role proyek SUPERUSER: %', bad;
  END IF;
  RAISE NOTICE 'LULUS T-05 tidak ada role proyek yang SUPERUSER';
END $$;

-- T-05: CREATEROLE hanya pada openbao_*_admin.
DO $$
DECLARE
  bad text;
BEGIN
  SELECT string_agg(rolname, ', ' ORDER BY rolname) INTO bad
    FROM pg_catalog.pg_roles
   WHERE NOT rolsuper AND rolcreaterole AND rolname !~ '^openbao_[a-z_]+_admin$';
  IF bad IS NOT NULL THEN
    RAISE EXCEPTION 'GAGAL T-05 CREATEROLE di luar openbao_*_admin: %', bad;
  END IF;
  RAISE NOTICE 'LULUS T-05 CREATEROLE hanya pada openbao_*_admin';
END $$;

-- T-05: keanggotaan yang saling meniadakan tidak ada (mewarisi atau dapat SET ROLE):
-- *_privacy_worker dengan *_key_rotation; *_audit_writer dengan *_audit_verifier;
-- grup migrasi (*_migration) dengan grup proyek lain mana pun.
DO $$
DECLARE
  bad text;
BEGIN
  WITH grp AS (
    SELECT oid, rolname FROM pg_catalog.pg_roles
     WHERE NOT rolcanlogin AND rolname ~ '^(rs|rme)_'
  ), holder AS (
    SELECT r.rolname AS member, g.rolname AS grp
      FROM pg_catalog.pg_roles r
      JOIN grp g ON r.oid <> g.oid
     WHERE NOT r.rolsuper
       AND (pg_catalog.pg_has_role(r.oid, g.oid, 'USAGE') OR pg_catalog.pg_has_role(r.oid, g.oid, 'SET'))
  ), pair AS (
    SELECT a.member, a.grp AS grp_a, b.grp AS grp_b
      FROM holder a JOIN holder b ON a.member = b.member AND a.grp < b.grp
     WHERE (a.grp ~ '_privacy_worker$' AND b.grp ~ '_key_rotation$')
        OR (a.grp ~ '_key_rotation$' AND b.grp ~ '_privacy_worker$')
        OR (a.grp ~ '_audit_writer$' AND b.grp ~ '_audit_verifier$')
        OR (a.grp ~ '_audit_verifier$' AND b.grp ~ '_audit_writer$')
        OR (a.grp ~ '_migration$') <> (b.grp ~ '_migration$')
  )
  SELECT string_agg(format('%s: %s + %s', member, grp_a, grp_b), '; ' ORDER BY member, grp_a, grp_b)
    INTO bad FROM pair;
  IF bad IS NOT NULL THEN
    RAISE EXCEPTION 'GAGAL T-05 keanggotaan saling meniadakan: %', bad;
  END IF;
  RAISE NOTICE 'LULUS T-05 tidak ada keanggotaan yang saling meniadakan';
END $$;

-- T-05: role grup proyek tidak dapat login.
DO $$
DECLARE
  bad text;
BEGIN
  SELECT string_agg(rolname, ', ' ORDER BY rolname) INTO bad
    FROM pg_catalog.pg_roles
   WHERE rolcanlogin
     AND rolname IN ('rs_app', 'rs_audit_relay', 'rs_privacy_worker', 'rs_key_rotation', 'rs_migration',
                     'rs_ops_monitor', 'rs_audit_writer', 'rs_audit_reader', 'rs_audit_verifier',
                     'rs_audit_migration', 'rme_app', 'rme_auth', 'rme_worker', 'rme_operator',
                     'rme_audit_relay', 'rme_privacy_worker', 'rme_key_rotation', 'rme_migration',
                     'rme_ops_monitor', 'rme_audit_writer', 'rme_audit_reader', 'rme_audit_verifier',
                     'rme_audit_tenant_reader', 'rme_audit_migration');
  IF bad IS NOT NULL THEN
    RAISE EXCEPTION 'GAGAL T-05 role grup dapat login: %', bad;
  END IF;
  RAISE NOTICE 'LULUS T-05 role grup proyek NOLOGIN';
END $$;

-- T-03 (§3.5 butir 2): PUBLIC tidak punya CONNECT ke database postgres.
DO $$
BEGIN
  IF EXISTS (SELECT 1 FROM pg_catalog.pg_database d, pg_catalog.aclexplode(d.datacl) a
              WHERE d.datname = 'postgres' AND a.grantee = 0 AND a.privilege_type = 'CONNECT')
     OR (SELECT datacl IS NULL FROM pg_catalog.pg_database WHERE datname = 'postgres') THEN
    RAISE EXCEPTION 'GAGAL T-03 PUBLIC masih punya CONNECT ke database postgres';
  END IF;
  RAISE NOTICE 'LULUS T-03 CONNECT database postgres dicabut dari PUBLIC';
END $$;
