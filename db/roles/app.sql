-- db/roles/app.sql: bootstrap klaster (a) db-app (database ruang_sehat dan keycloak).
--
-- Modul kritis T1 (ADR-008, ADR-012, ADR-021 butir 4): perubahan wajib review DBA + keamanan.
-- Sumber: docs/05-physical-schema.md §2.1, §2.2, §3.1, §3.2 (a), §3.4, §3.5, §10.2, §10.3.
-- Perilaku PostgreSQL 18 yang diandalkan: SRC-367 s.d. SRC-376 (docs/sources.md).
--
-- Dijalankan superuser bootstrap dengan psql (memakai \gexec dan \connect), sebelum migrasi:
--   psql -X -v ON_ERROR_STOP=1 -U <superuser> -d postgres -f db/roles/app.sql
-- Production/staging: lewat akses darurat (PT-38, KEPUTUSAN-79). Lokal/test: scripts/db-roles-check.sh.
--
-- Sifat (§3.5, KEPUTUSAN-97):
-- - Idempoten: aman dijalankan berulang; atribut, keanggotaan, hak, dan pengaturan dipaksakan
--   ulang ke nilai di berkas ini setiap kali dijalankan. Keanggotaan atau hak TAMBAHAN yang dibuat
--   di luar berkas ini tidak dicabut diam-diam, tetapi ditangkap test db/roles/test/ (T-03, T-05).
-- - Tanpa password: ALTER ROLE di sini tidak menyentuh password. Password role login aplikasi
--   diisi terpisah dari OpenBao KV; pemilik skema memakai kredensial dinamis OpenBao (§3.4).
-- - Hak per tabel TIDAK ada di sini, tetapi di berkas migrasi yang membuat tabelnya (§3.5, §4).

\set ON_ERROR_STOP on
SET client_min_messages = warning;

-- 0. Penjaga: hanya superuser, hanya PostgreSQL 18 (opsi keanggotaan INHERIT/SET butuh 16+).
DO $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM pg_catalog.pg_roles WHERE rolname = current_user AND rolsuper) THEN
    RAISE EXCEPTION 'db/roles/app.sql harus dijalankan superuser bootstrap, bukan %', current_user;
  END IF;
  IF current_setting('server_version_num')::int < 180000 THEN
    RAISE EXCEPTION 'db/roles/app.sql menargetkan PostgreSQL 18, server %', current_setting('server_version');
  END IF;
END $$;

-- 1. Role grup dan login (§3.1 butir 3, §3.2 (a)).
--    Atribut semua role: NOSUPERUSER NOCREATEDB NOREPLICATION NOBYPASSRLS INHERIT; grup NOLOGIN.
--    CREATEROLE hanya openbao_rs_admin (§3.4). Batas koneksi role login = kolom "Total" §10.3.
--    Batas koneksi openbao_rs_admin tidak ditetapkan dokumen: -1 (tanpa batas), lihat laporan F1-T13.
DO $$
DECLARE
  r record;
BEGIN
  FOR r IN
    SELECT * FROM (VALUES
      -- grup (NOLOGIN)
      ('rs_app',                  false, -1, false),
      ('rs_audit_relay',          false, -1, false),
      ('rs_privacy_worker',       false, -1, false),
      ('rs_key_rotation',         false, -1, false),   -- anggota dinamis OpenBao saja
      ('rs_migration',            false, -1, false),   -- pemilik skema app, util; anggota dinamis saja
      ('rs_ops_monitor',          false, -1, false),
      -- login per proses (§3.3); batas koneksi §10.3
      ('rs_api',                  true,  40, false),
      ('rs_realtime',             true,  20, false),
      ('rs_worker',               true,  20, false),
      ('rs_audit_relay_login',    true,   6, false),
      ('rs_privacy_worker_login', true,   8, false),
      ('rs_metrics',              true,   2, false),
      ('keycloak',                true,  40, false),   -- pemilik database keycloak (§2.1)
      ('openbao_rs_admin',        true,  -1, true)     -- koneksi OpenBao (§3.4)
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

-- Keanggotaan dengan opsi eksplisit (§3.1 butir 5, §3.5 butir 1).
-- Login proses mewarisi hak grupnya (INHERIT TRUE) tetapi tidak dapat SET ROLE ke grup (SET FALSE),
-- sehingga objek tidak pernah dibuat atas nama grup aplikasi.
GRANT rs_app            TO rs_api, rs_realtime, rs_worker WITH INHERIT TRUE, SET FALSE, ADMIN FALSE;
GRANT rs_audit_relay    TO rs_audit_relay_login           WITH INHERIT TRUE, SET FALSE, ADMIN FALSE;
GRANT rs_privacy_worker TO rs_privacy_worker_login        WITH INHERIT TRUE, SET FALSE, ADMIN FALSE;
GRANT rs_ops_monitor    TO rs_metrics                     WITH INHERIT TRUE, SET FALSE, ADMIN FALSE;
GRANT pg_monitor        TO rs_ops_monitor                 WITH INHERIT TRUE, SET FALSE, ADMIN FALSE;
-- openbao_rs_admin: hanya ADMIN atas grup sasaran role OpenBao (§3.4), tanpa mewarisi dan tanpa
-- SET ROLE, sehingga tidak memegang hak pemilik skema sendiri.
GRANT rs_migration, rs_key_rotation, rs_ops_monitor TO openbao_rs_admin
  WITH ADMIN TRUE, INHERIT FALSE, SET FALSE;

-- Pengaturan log role admin OpenBao: teks CREATE ROLE ... PASSWORD tidak tercatat (§3.4, review N08).
ALTER ROLE openbao_rs_admin SET log_min_error_statement = 'panic';
ALTER ROLE openbao_rs_admin SET log_statement = 'none';

-- 2. Database (§2.1 butir 3–4, §3.5 butir 2). ruang_sehat dimiliki superuser bootstrap;
--    keycloak dimiliki role keycloak (§2.1, §3.2 (a)).
SELECT 'CREATE DATABASE ruang_sehat'
 WHERE NOT EXISTS (SELECT 1 FROM pg_catalog.pg_database WHERE datname = 'ruang_sehat') \gexec
SELECT 'CREATE DATABASE keycloak OWNER keycloak'
 WHERE NOT EXISTS (SELECT 1 FROM pg_catalog.pg_database WHERE datname = 'keycloak') \gexec
ALTER DATABASE keycloak OWNER TO keycloak;

REVOKE CONNECT, TEMPORARY ON DATABASE ruang_sehat FROM PUBLIC;
REVOKE CONNECT, TEMPORARY ON DATABASE keycloak FROM PUBLIC;
REVOKE CONNECT ON DATABASE postgres FROM PUBLIC;

GRANT CONNECT ON DATABASE ruang_sehat
  TO rs_app, rs_audit_relay, rs_privacy_worker, rs_key_rotation, rs_migration, rs_ops_monitor,
     openbao_rs_admin;
GRANT CONNECT ON DATABASE keycloak TO keycloak;

-- 3. Skema dan ekstensi database ruang_sehat (§2.1, §2.2, §3.5 butir 3).
\connect ruang_sehat
SET client_min_messages = warning;

-- Skema public dihapus (§2.1 butir 2). Tanpa CASCADE: gagal bila ternyata berisi objek.
DROP SCHEMA IF EXISTS public;

CREATE SCHEMA IF NOT EXISTS app AUTHORIZATION rs_migration;
ALTER SCHEMA app OWNER TO rs_migration;
CREATE SCHEMA IF NOT EXISTS util AUTHORIZATION rs_migration;
ALTER SCHEMA util OWNER TO rs_migration;
CREATE SCHEMA IF NOT EXISTS ext;   -- dimiliki superuser bootstrap (§2.1)
REVOKE ALL ON SCHEMA app, util, ext FROM PUBLIC;

-- Ekstensi dibuat superuser saat bootstrap, bukan migrasi (§2.2, KEPUTUSAN-92).
CREATE EXTENSION IF NOT EXISTS pg_trgm SCHEMA ext;

-- USAGE skema app untuk grup yang punya hak tabel di DD §0.6 (A).
GRANT USAGE ON SCHEMA app TO rs_app, rs_audit_relay, rs_privacy_worker, rs_key_rotation;
-- USAGE skema ext: rs_app (operator/fungsi trigram) dan rs_migration (opclass ext.gin_trgm_ops, §2.2).
GRANT USAGE ON SCHEMA ext TO rs_app, rs_migration;
-- Skema util: USAGE/EXECUTE diberikan migrasi fungsi bersama (F1-T14, §4.2), bukan di sini.

-- EXECUTE fungsi dan prosedur baru milik rs_migration tidak diberikan ke PUBLIC (§2.1 butir 6).
ALTER DEFAULT PRIVILEGES FOR ROLE rs_migration REVOKE EXECUTE ON FUNCTIONS FROM PUBLIC;

-- 4a. Pengaturan tingkat database (§2.1 butir 5, §10.1, §10.2 baris pemilik skema).
ALTER DATABASE ruang_sehat SET search_path = app, ext;
ALTER DATABASE ruang_sehat SET "TimeZone" = 'UTC';
ALTER DATABASE ruang_sehat SET idle_in_transaction_session_timeout = '60s';

-- 4b. Pengaturan per role login per database (§2.1 butir 5, §10.2). NULL = tidak ditetapkan dokumen.
DO $$
DECLARE
  r record;
BEGIN
  FOR r IN
    SELECT * FROM (VALUES
      ('rs_api',                  '3s',   '2s', '30s'),
      ('rs_realtime',             '3s',   '2s', '30s'),
      ('rs_worker',               '60s',  '5s', '60s'),
      ('rs_privacy_worker_login', '300s', '5s', '60s'),
      ('rs_audit_relay_login',    '30s',  '5s', '30s'),
      ('rs_metrics',              NULL,   NULL, NULL)
    ) AS t(rolname, stmt_timeout, lck_timeout, idle_timeout)
  LOOP
    EXECUTE format('ALTER ROLE %I IN DATABASE ruang_sehat SET search_path = app, ext', r.rolname);
    IF r.stmt_timeout IS NOT NULL THEN
      EXECUTE format('ALTER ROLE %I IN DATABASE ruang_sehat SET statement_timeout = %L',
                     r.rolname, r.stmt_timeout);
      EXECUTE format('ALTER ROLE %I IN DATABASE ruang_sehat SET lock_timeout = %L',
                     r.rolname, r.lck_timeout);
      EXECUTE format('ALTER ROLE %I IN DATABASE ruang_sehat SET idle_in_transaction_session_timeout = %L',
                     r.rolname, r.idle_timeout);
    END IF;
  END LOOP;
END $$;
ALTER ROLE keycloak IN DATABASE keycloak SET idle_in_transaction_session_timeout = '60s';
