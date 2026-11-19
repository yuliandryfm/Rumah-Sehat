-- db/roles/audit.sql: bootstrap klaster (b) db-audit (database ruang_sehat_audit dan rme_audit).
--
-- Modul kritis T1 (ADR-009, ADR-008, ADR-021 butir 4): perubahan wajib review DBA + keamanan.
-- Sumber: docs/05-physical-schema.md §2.1, §3.1, §3.2 (b), §3.4, §3.5, §5.6, §6.5, §10.2, §10.3.
-- Perilaku PostgreSQL 18 yang diandalkan: SRC-367 s.d. SRC-376 (docs/sources.md).
--
-- Dijalankan superuser bootstrap dengan psql (memakai \gexec dan \connect), sebelum migrasi:
--   psql -X -v ON_ERROR_STOP=1 -U <superuser> -d postgres -f db/roles/audit.sql
--
-- Sifat (§3.5, KEPUTUSAN-97): idempoten, tanpa password, tanpa hak per tabel (hak tabel audit,
-- RLS tenant rme_audit, dan fungsi util ada di migrasi db/migrations/audit/).
-- Dua database audit berbagi instans, sedangkan role berlaku per klaster, sehingga nama role
-- berawalan rs_ / rme_ (§3.1 butir 2, F-P01). Tidak ada ekstensi di database audit (§2.2).

\set ON_ERROR_STOP on
SET client_min_messages = warning;
SET search_path = pg_catalog, pg_temp;   -- cegah pembajakan search_path (CVE-2018-1058), review keamanan F1-T13

-- 0. Penjaga: hanya superuser, hanya PostgreSQL 18.
DO $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_catalog.pg_roles WHERE rolname = current_user AND rolsuper) THEN
    RAISE EXCEPTION 'db/roles/audit.sql harus dijalankan superuser bootstrap, bukan %', current_user;
  END IF;
  IF pg_catalog.current_setting('server_version_num')::int < 180000 THEN
    RAISE EXCEPTION 'db/roles/audit.sql menargetkan PostgreSQL 18, server %', pg_catalog.current_setting('server_version');
  END IF;
END $$;

-- 1. Role grup dan login (§3.1 butir 3, §3.2 (b)). Batas koneksi = rincian instans (b) §10.3.
--    rme_audit_reader sengaja TANPA role login: belum ada proses pemegangnya (§3.2 (b), T-05).
--    Batas koneksi openbao_*_audit_admin tidak ditetapkan dokumen: -1, lihat laporan F1-T13.
DO $$
DECLARE
  r record;
BEGIN
  FOR r IN
    SELECT * FROM (VALUES
      -- Ruang Sehat
      ('rs_audit_writer',                false, -1, false),
      ('rs_audit_reader',                false, -1, false),
      ('rs_audit_verifier',              false, -1, false),
      ('rs_audit_migration',             false, -1, false),  -- pemilik skema; anggota dinamis saja
      ('rs_audit_writer_login',          true,   6, false),  -- relay, 2 instans x 3
      ('rs_audit_reader_login',          true,  10, false),  -- api, 2 x 5
      ('rs_audit_verifier_login',        true,   4, false),  -- audit-verify, 2 x 2
      ('openbao_rs_audit_admin',         true,  -1, true),
      -- RME
      ('rme_audit_writer',               false, -1, false),
      ('rme_audit_reader',               false, -1, false),
      ('rme_audit_verifier',             false, -1, false),
      ('rme_audit_tenant_reader',        false, -1, false),
      ('rme_audit_migration',            false, -1, false),
      ('rme_audit_writer_login',         true,   6, false),
      ('rme_audit_verifier_login',       true,   4, false),
      ('rme_audit_tenant_reader_login',  true,   6, false),  -- rme api, 2 x 3
      ('openbao_rme_audit_admin',        true,  -1, true)
    ) AS t(rolname, can_login, conn_limit, can_createrole)
  LOOP
    IF NOT EXISTS (SELECT 1 FROM pg_catalog.pg_roles WHERE rolname = r.rolname) THEN
      EXECUTE pg_catalog.format('CREATE ROLE %I', r.rolname);
    END IF;
    EXECUTE pg_catalog.format(
      'ALTER ROLE %I WITH NOSUPERUSER NOCREATEDB %s NOREPLICATION NOBYPASSRLS INHERIT %s CONNECTION LIMIT %s',
      r.rolname,
      CASE WHEN r.can_createrole THEN 'CREATEROLE' ELSE 'NOCREATEROLE' END,
      CASE WHEN r.can_login THEN 'LOGIN' ELSE 'NOLOGIN' END,
      r.conn_limit);
  END LOOP;
END $$;

GRANT rs_audit_writer          TO rs_audit_writer_login         WITH INHERIT TRUE, SET FALSE, ADMIN FALSE;
GRANT rs_audit_reader          TO rs_audit_reader_login         WITH INHERIT TRUE, SET FALSE, ADMIN FALSE;
GRANT rs_audit_verifier        TO rs_audit_verifier_login       WITH INHERIT TRUE, SET FALSE, ADMIN FALSE;
GRANT rme_audit_writer         TO rme_audit_writer_login        WITH INHERIT TRUE, SET FALSE, ADMIN FALSE;
GRANT rme_audit_verifier       TO rme_audit_verifier_login      WITH INHERIT TRUE, SET FALSE, ADMIN FALSE;
GRANT rme_audit_tenant_reader  TO rme_audit_tenant_reader_login WITH INHERIT TRUE, SET FALSE, ADMIN FALSE;
-- Satu koneksi OpenBao per database audit (§3.4): ADMIN hanya atas pemilik skema database itu.
-- RISIKO RESIDU (review keamanan F1-T13): karena memegang ADMIN, role ini dapat memberi dirinya
-- sendiri keanggotaan baru dengan SET TRUE lalu SET ROLE ke pemilik skema; ia setara kuatnya dengan
-- pemilik skema (§3.4). SET FALSE di sini hanya keadaan awal, bukan batas keamanan. Kontrol:
-- password hanya di OpenBao, pg_hba hanya dari host OpenBao (§3.4), log audit OpenBao (detektif),
-- dan test katalog T-05 (keanggotaan persis) yang dijalankan berkala ke staging/production (§3.5).
-- createrole_self_grant dikosongkan supaya role yang dibuatnya tidak otomatis mewariskan hak.
GRANT rs_audit_migration  TO openbao_rs_audit_admin  WITH ADMIN TRUE, INHERIT FALSE, SET FALSE;
GRANT rme_audit_migration TO openbao_rme_audit_admin WITH ADMIN TRUE, INHERIT FALSE, SET FALSE;

ALTER ROLE openbao_rs_audit_admin  SET log_min_error_statement = 'panic';
ALTER ROLE openbao_rs_audit_admin  SET log_statement = 'none';
ALTER ROLE openbao_rs_audit_admin  SET log_min_duration_statement = -1;
ALTER ROLE openbao_rs_audit_admin  SET log_min_duration_sample = -1;
ALTER ROLE openbao_rs_audit_admin  SET createrole_self_grant = '';
ALTER ROLE openbao_rme_audit_admin SET log_min_error_statement = 'panic';
ALTER ROLE openbao_rme_audit_admin SET log_statement = 'none';
ALTER ROLE openbao_rme_audit_admin SET log_min_duration_statement = -1;
ALTER ROLE openbao_rme_audit_admin SET log_min_duration_sample = -1;
ALTER ROLE openbao_rme_audit_admin SET createrole_self_grant = '';

-- 2. Database (§2.1, §3.5 butir 2), dimiliki superuser bootstrap.
SELECT 'CREATE DATABASE ruang_sehat_audit'
 WHERE NOT EXISTS (SELECT 1 FROM pg_catalog.pg_database WHERE datname = 'ruang_sehat_audit') \gexec
SELECT 'CREATE DATABASE rme_audit'
 WHERE NOT EXISTS (SELECT 1 FROM pg_catalog.pg_database WHERE datname = 'rme_audit') \gexec

REVOKE CONNECT, TEMPORARY ON DATABASE ruang_sehat_audit FROM PUBLIC;
REVOKE CONNECT, TEMPORARY ON DATABASE rme_audit FROM PUBLIC;
REVOKE CONNECT ON DATABASE postgres FROM PUBLIC;
-- template1 juga tertutup bagi PUBLIC (review DBA F1-T13); template0 tidak menerima koneksi.
REVOKE CONNECT, TEMPORARY ON DATABASE template1 FROM PUBLIC;

-- Penjaga encoding: database proyek wajib UTF8 (review DBA F1-T13).
DO $$
DECLARE
  bad text;
BEGIN
  SELECT pg_catalog.string_agg(datname || '=' || pg_catalog.pg_encoding_to_char(encoding), ', ') INTO bad
    FROM pg_catalog.pg_database
   WHERE datname IN ('ruang_sehat_audit', 'rme_audit')
     AND pg_catalog.pg_encoding_to_char(encoding) <> 'UTF8';
  IF bad IS NOT NULL THEN
    RAISE EXCEPTION 'encoding database bukan UTF8: %', bad;
  END IF;
END $$;

-- Role rs_audit_* tidak dapat memasuki rme_audit dan sebaliknya (T-21).
GRANT CONNECT ON DATABASE ruang_sehat_audit
  TO rs_audit_writer, rs_audit_reader, rs_audit_verifier, rs_audit_migration, openbao_rs_audit_admin;
GRANT CONNECT ON DATABASE rme_audit
  TO rme_audit_writer, rme_audit_reader, rme_audit_verifier, rme_audit_tenant_reader,
     rme_audit_migration, openbao_rme_audit_admin;

-- 3. Skema per database (§2.1): audit, util, audit_detached milik pemilik skema audit.
--    audit_detached tanpa hak untuk role mana pun selain pemilik (§6.5 butir 5).
\connect ruang_sehat_audit
SET client_min_messages = warning;
SET search_path = pg_catalog, pg_temp;
DROP SCHEMA IF EXISTS public;
CREATE SCHEMA IF NOT EXISTS audit AUTHORIZATION rs_audit_migration;
ALTER SCHEMA audit OWNER TO rs_audit_migration;
CREATE SCHEMA IF NOT EXISTS util AUTHORIZATION rs_audit_migration;
ALTER SCHEMA util OWNER TO rs_audit_migration;
CREATE SCHEMA IF NOT EXISTS audit_detached AUTHORIZATION rs_audit_migration;
ALTER SCHEMA audit_detached OWNER TO rs_audit_migration;
REVOKE ALL ON SCHEMA audit, util, audit_detached FROM PUBLIC;
GRANT USAGE ON SCHEMA audit TO rs_audit_writer, rs_audit_reader, rs_audit_verifier;
ALTER DEFAULT PRIVILEGES FOR ROLE rs_audit_migration REVOKE EXECUTE ON FUNCTIONS FROM PUBLIC;

\connect rme_audit
SET client_min_messages = warning;
SET search_path = pg_catalog, pg_temp;
DROP SCHEMA IF EXISTS public;
CREATE SCHEMA IF NOT EXISTS audit AUTHORIZATION rme_audit_migration;
ALTER SCHEMA audit OWNER TO rme_audit_migration;
CREATE SCHEMA IF NOT EXISTS util AUTHORIZATION rme_audit_migration;
ALTER SCHEMA util OWNER TO rme_audit_migration;
CREATE SCHEMA IF NOT EXISTS audit_detached AUTHORIZATION rme_audit_migration;
ALTER SCHEMA audit_detached OWNER TO rme_audit_migration;
REVOKE ALL ON SCHEMA audit, util, audit_detached FROM PUBLIC;
GRANT USAGE ON SCHEMA audit
  TO rme_audit_writer, rme_audit_reader, rme_audit_verifier, rme_audit_tenant_reader;
-- util (fn_current_fasyankes_id untuk rme_audit_tenant_reader) diberikan migrasi (§5.6).
ALTER DEFAULT PRIVILEGES FOR ROLE rme_audit_migration REVOKE EXECUTE ON FUNCTIONS FROM PUBLIC;

-- 4. Pengaturan database dan per role login (§2.1 butir 5, §10.1, §10.2).
ALTER DATABASE ruang_sehat_audit SET search_path = audit;
ALTER DATABASE ruang_sehat_audit SET "TimeZone" = 'UTC';
ALTER DATABASE ruang_sehat_audit SET idle_in_transaction_session_timeout = '60s';
ALTER DATABASE rme_audit SET search_path = audit;
ALTER DATABASE rme_audit SET "TimeZone" = 'UTC';
ALTER DATABASE rme_audit SET idle_in_transaction_session_timeout = '60s';

DO $$
DECLARE
  r record;
BEGIN
  FOR r IN
    SELECT * FROM (VALUES
      ('rs_audit_writer_login',         'ruang_sehat_audit', '30s', '5s', '30s'),
      ('rs_audit_reader_login',         'ruang_sehat_audit', '30s', '2s', '30s'),
      ('rs_audit_verifier_login',       'ruang_sehat_audit', '0',   '5s', '60s'),
      ('rme_audit_writer_login',        'rme_audit',         '30s', '5s', '30s'),
      ('rme_audit_tenant_reader_login', 'rme_audit',         '30s', '2s', '30s'),
      ('rme_audit_verifier_login',      'rme_audit',         '0',   '5s', '60s')
    ) AS t(rolname, dbname, stmt_timeout, lck_timeout, idle_timeout)
  LOOP
    EXECUTE pg_catalog.format('ALTER ROLE %I IN DATABASE %I SET search_path = audit', r.rolname, r.dbname);
    EXECUTE pg_catalog.format('ALTER ROLE %I IN DATABASE %I SET statement_timeout = %L',
                   r.rolname, r.dbname, r.stmt_timeout);
    EXECUTE pg_catalog.format('ALTER ROLE %I IN DATABASE %I SET lock_timeout = %L',
                   r.rolname, r.dbname, r.lck_timeout);
    EXECUTE pg_catalog.format('ALTER ROLE %I IN DATABASE %I SET idle_in_transaction_session_timeout = %L',
                   r.rolname, r.dbname, r.idle_timeout);
  END LOOP;
END $$;
