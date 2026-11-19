-- db/roles/rme.sql: bootstrap klaster (c) db-rme (database rme, RME minimalis).
--
-- Modul kritis T1 (ADR-012 butir 6 dan 8, ADR-008, ADR-021 butir 4): wajib review DBA + keamanan.
-- Sumber: docs/05-physical-schema.md §2.1, §2.2, §3.1, §3.2 (c), §3.4, §3.5, §10.2, §10.3.
-- Perilaku PostgreSQL 18 yang diandalkan: SRC-367 s.d. SRC-376 (docs/sources.md).
--
-- Dijalankan superuser bootstrap dengan psql (memakai \gexec dan \connect), sebelum migrasi:
--   psql -X -v ON_ERROR_STOP=1 -U <superuser> -d postgres -f db/roles/rme.sql
--
-- Sifat (§3.5, KEPUTUSAN-97): idempoten, tanpa password, tanpa hak per tabel (hak tabel, RLS
-- FORCE, dan kebijakan per role ada di migrasi db/migrations/rme/). Semua role tanpa BYPASSRLS
-- dan bukan pemilik tabel, sehingga RLS berlaku (§3.1 butir 3–4, SRC-34).

\set ON_ERROR_STOP on
SET client_min_messages = warning;

-- 0. Penjaga: hanya superuser, hanya PostgreSQL 18.
DO $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_catalog.pg_roles WHERE rolname = current_user AND rolsuper) THEN
    RAISE EXCEPTION 'db/roles/rme.sql harus dijalankan superuser bootstrap, bukan %', current_user;
  END IF;
  IF current_setting('server_version_num')::int < 180000 THEN
    RAISE EXCEPTION 'db/roles/rme.sql menargetkan PostgreSQL 18, server %', current_setting('server_version');
  END IF;
END $$;

-- 1. Role grup dan login (§3.1 butir 3, §3.2 (c)). Batas koneksi role login = §10.3 "Total" (c).
--    Batas koneksi openbao_rme_admin tidak ditetapkan dokumen: -1, lihat laporan F1-T13.
DO $$
DECLARE
  r record;
BEGIN
  FOR r IN
    SELECT * FROM (VALUES
      -- grup (NOLOGIN)
      ('rme_app',                  false, -1, false),
      ('rme_auth',                 false, -1, false),
      ('rme_worker',               false, -1, false),
      ('rme_operator',             false, -1, false),
      ('rme_audit_relay',          false, -1, false),
      ('rme_privacy_worker',       false, -1, false),
      ('rme_key_rotation',         false, -1, false),  -- anggota dinamis OpenBao saja
      ('rme_migration',            false, -1, false),  -- pemilik skema app, util; anggota dinamis saja
      ('rme_ops_monitor',          false, -1, false),
      -- login per proses (§3.3); batas koneksi §10.3
      ('rme_api',                  true,  30, false),  -- 2 x 15
      ('rme_api_auth',             true,   6, false),  -- 2 x 3
      ('rme_api_operator',         true,   4, false),  -- 2 x 2
      ('rme_worker_login',         true,  16, false),
      ('rme_audit_relay_login',    true,   6, false),
      ('rme_privacy_worker_login', true,   6, false),
      ('rme_metrics',              true,   2, false),
      ('openbao_rme_admin',        true,  -1, true)
    ) AS t(rolname, can_login, conn_limit, can_createrole)
  LOOP
    IF NOT EXISTS (SELECT 1 FROM pg_catalog.pg_roles WHERE rolname = r.rolname) THEN
      EXECUTE format('CREATE ROLE %I', r.rolname);
    END IF;
    EXECUTE format(
      'ALTER ROLE %I WITH NOSUPERUSER NOCREATEDB %s NOREPLICATION NOBYPASSRLS INHERIT %s CONNECTION LIMIT %s',
      r.rolname,
      CASE WHEN r.can_createrole THEN 'CREATEROLE' ELSE 'NOCREATEROLE' END,
      CASE WHEN r.can_login THEN 'LOGIN' ELSE 'NOLOGIN' END,
      r.conn_limit);
  END LOOP;
END $$;

-- Setiap login hanya anggota satu grup (rme_api, rme_api_auth, rme_api_operator dipakai proses
-- api yang sama tetapi tetap terpisah, ADR-012 butir 6 dan 8).
GRANT rme_app            TO rme_api                  WITH INHERIT TRUE, SET FALSE, ADMIN FALSE;
GRANT rme_auth           TO rme_api_auth             WITH INHERIT TRUE, SET FALSE, ADMIN FALSE;
GRANT rme_operator       TO rme_api_operator         WITH INHERIT TRUE, SET FALSE, ADMIN FALSE;
GRANT rme_worker         TO rme_worker_login         WITH INHERIT TRUE, SET FALSE, ADMIN FALSE;
GRANT rme_audit_relay    TO rme_audit_relay_login    WITH INHERIT TRUE, SET FALSE, ADMIN FALSE;
GRANT rme_privacy_worker TO rme_privacy_worker_login WITH INHERIT TRUE, SET FALSE, ADMIN FALSE;
GRANT rme_ops_monitor    TO rme_metrics              WITH INHERIT TRUE, SET FALSE, ADMIN FALSE;
GRANT pg_monitor         TO rme_ops_monitor          WITH INHERIT TRUE, SET FALSE, ADMIN FALSE;
GRANT rme_migration, rme_key_rotation, rme_ops_monitor TO openbao_rme_admin
  WITH ADMIN TRUE, INHERIT FALSE, SET FALSE;

ALTER ROLE openbao_rme_admin SET log_min_error_statement = 'panic';
ALTER ROLE openbao_rme_admin SET log_statement = 'none';

-- 2. Database (§2.1, §3.5 butir 2), dimiliki superuser bootstrap.
SELECT 'CREATE DATABASE rme'
 WHERE NOT EXISTS (SELECT 1 FROM pg_catalog.pg_database WHERE datname = 'rme') \gexec

REVOKE CONNECT, TEMPORARY ON DATABASE rme FROM PUBLIC;
REVOKE CONNECT ON DATABASE postgres FROM PUBLIC;

GRANT CONNECT ON DATABASE rme
  TO rme_app, rme_auth, rme_worker, rme_operator, rme_audit_relay, rme_privacy_worker,
     rme_key_rotation, rme_migration, rme_ops_monitor, openbao_rme_admin;

-- 3. Skema dan ekstensi (§2.1, §2.2, §3.5 butir 3).
\connect rme
SET client_min_messages = warning;
DROP SCHEMA IF EXISTS public;
CREATE SCHEMA IF NOT EXISTS app AUTHORIZATION rme_migration;
ALTER SCHEMA app OWNER TO rme_migration;
CREATE SCHEMA IF NOT EXISTS util AUTHORIZATION rme_migration;
ALTER SCHEMA util OWNER TO rme_migration;
CREATE SCHEMA IF NOT EXISTS ext;   -- dimiliki superuser bootstrap (§2.1)
REVOKE ALL ON SCHEMA app, util, ext FROM PUBLIC;
CREATE EXTENSION IF NOT EXISTS pg_trgm SCHEMA ext;

-- USAGE skema app untuk grup yang punya hak tabel di DD §0.6 (C).
GRANT USAGE ON SCHEMA app
  TO rme_app, rme_auth, rme_worker, rme_operator, rme_audit_relay, rme_privacy_worker,
     rme_key_rotation;
GRANT USAGE ON SCHEMA ext TO rme_app, rme_migration;
-- util (fn_current_fasyankes_id untuk rme_app, rme_worker) diberikan migrasi (§5.2), bukan di sini.
ALTER DEFAULT PRIVILEGES FOR ROLE rme_migration REVOKE EXECUTE ON FUNCTIONS FROM PUBLIC;

-- 4. Pengaturan database dan per role login (§2.1 butir 5, §10.1, §10.2).
ALTER DATABASE rme SET search_path = app, ext;
ALTER DATABASE rme SET "TimeZone" = 'UTC';
ALTER DATABASE rme SET idle_in_transaction_session_timeout = '60s';

DO $$
DECLARE
  r record;
BEGIN
  FOR r IN
    SELECT * FROM (VALUES
      ('rme_api',                  '3s',   '2s', '30s'),
      ('rme_api_auth',             '3s',   '2s', '30s'),
      ('rme_api_operator',         '3s',   '2s', '30s'),
      ('rme_worker_login',         '60s',  '5s', '60s'),
      ('rme_privacy_worker_login', '300s', '5s', '60s'),
      ('rme_audit_relay_login',    '30s',  '5s', '30s'),
      ('rme_metrics',              NULL,   NULL, NULL)
    ) AS t(rolname, stmt_timeout, lck_timeout, idle_timeout)
  LOOP
    EXECUTE format('ALTER ROLE %I IN DATABASE rme SET search_path = app, ext', r.rolname);
    IF r.stmt_timeout IS NOT NULL THEN
      EXECUTE format('ALTER ROLE %I IN DATABASE rme SET statement_timeout = %L',
                     r.rolname, r.stmt_timeout);
      EXECUTE format('ALTER ROLE %I IN DATABASE rme SET lock_timeout = %L',
                     r.rolname, r.lck_timeout);
      EXECUTE format('ALTER ROLE %I IN DATABASE rme SET idle_in_transaction_session_timeout = %L',
                     r.rolname, r.idle_timeout);
    END IF;
  END LOOP;
END $$;
