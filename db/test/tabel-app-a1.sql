-- db/test/tabel-app-a1.sql: test tabel A1 identitas dan akun (F1-T16) di database ruang_sehat:
-- account, rs_admin_role, session_revocation. Didahului db/test/lib.sql; dijalankan superuser setelah
-- migrasi. Aturan kelas generik (T-06/T-07/T-08) diperiksa db/test/kelas-app.sql bagian A; berkas ini
-- menambah (1) ketentuan DDL umum §4.1 atas seluruh skema app beserta test negatif pemeriksanya,
-- (2) struktur per tabel yang tidak tertangkap aturan kelas (index, kolom anonymize, komentar kelas),
-- dan (3) perilaku dengan role sungguhan (positif dan negatif). Semua perubahan dalam satu transaksi
-- yang di-ROLLBACK. Data sintetis.
-- Dasar: docs/04-data-dictionary.md §0.4, §0.6 A, A1.1, A1.2, A1.4; docs/05-physical-schema.md §4.1,
-- §4.3, §4.8, §12 (T-06, T-07, T-09); ADR-003, ADR-008.

-- 1. Ketentuan DDL umum §4.1 butir 2-3 atas seluruh skema app (berlaku juga untuk tabel berikutnya).
SELECT pg_temp.uji_nol('KEPUTUSAN-93 (ruang_sehat) skema app: id tanpa default, created_at NOT NULL DEFAULT now(), FK NO ACTION',
  ARRAY(SELECT pg_temp.uji_ddl_umum('app')));

-- 2. Struktur per tabel.
SELECT pg_temp.uji_nol('A1 komentar kelas: account M, rs_admin_role M, session_revocation O (modul identitas)',
  ARRAY(SELECT format('%s: %s', t, coalesce(obj_description(to_regclass('app.' || t), 'pg_class'), '<tidak ada>'))
          FROM (VALUES ('account', 'class:M module:identitas'), ('rs_admin_role', 'class:M module:identitas'),
                       ('session_revocation', 'class:O module:identitas')) v(t, k)
         WHERE obj_description(to_regclass('app.' || t), 'pg_class') IS DISTINCT FROM k));
SELECT pg_temp.uji_nol('A1.1 account: kolom anonymize persis email, full_name (DD A1.1 "Anonimkan")',
  ARRAY(SELECT x FROM (
    SELECT 'kolom anonymize ' || coalesce(string_agg(attname::text, ',' ORDER BY attname), '<kosong>') AS x
      FROM pg_attribute WHERE attrelid = 'app.account'::regclass AND attnum > 0 AND NOT attisdropped
       AND col_description(attrelid, attnum) = 'anonymize') s
   WHERE x <> 'kolom anonymize email,full_name'));
SELECT pg_temp.uji_nol('A1 tabel lain tanpa kolom anonymize (rs_admin_role, session_revocation)',
  ARRAY(SELECT format('%s.%s', attrelid::regclass, attname) FROM pg_attribute
         WHERE attrelid IN ('app.rs_admin_role'::regclass, 'app.session_revocation'::regclass)
           AND attnum > 0 AND col_description(attrelid, attnum) IS NOT NULL));
SELECT pg_temp.uji_nol('A1.1 account: index persis pk, uqp (idp_realm, idp_subject) aktif, ix status aktif',
  ARRAY(SELECT pg_temp.uji_indeks('app.account', ARRAY[
    'CREATE UNIQUE INDEX pk_account ON app.account USING btree (id)',
    'CREATE UNIQUE INDEX uqp_account_idp_realm_idp_subject ON app.account USING btree (idp_realm, idp_subject) WHERE (deleted_at IS NULL)',
    'CREATE INDEX ix_account_status ON app.account USING btree (status) WHERE (deleted_at IS NULL)'])));
SELECT pg_temp.uji_nol('A1.2 rs_admin_role: index persis pk, uqp (account_id, sub_role) aktif, ix sub_role aktif',
  ARRAY(SELECT pg_temp.uji_indeks('app.rs_admin_role', ARRAY[
    'CREATE UNIQUE INDEX pk_rs_admin_role ON app.rs_admin_role USING btree (id)',
    'CREATE UNIQUE INDEX uqp_rs_admin_role_account_id_sub_role ON app.rs_admin_role USING btree (account_id, sub_role) WHERE (deleted_at IS NULL)',
    'CREATE INDEX ix_rs_admin_role_sub_role ON app.rs_admin_role USING btree (sub_role) WHERE (deleted_at IS NULL)'])));
SELECT pg_temp.uji_nol('A1.4 session_revocation: index persis pk, uq idp_session_id, ix expires_at',
  ARRAY(SELECT pg_temp.uji_indeks('app.session_revocation', ARRAY[
    'CREATE UNIQUE INDEX pk_session_revocation ON app.session_revocation USING btree (id)',
    'CREATE UNIQUE INDEX uq_session_revocation_idp_session_id ON app.session_revocation USING btree (idp_session_id)',
    'CREATE INDEX ix_session_revocation_expires_at ON app.session_revocation USING btree (expires_at)'])));
SELECT pg_temp.uji_nol('A1 tanpa hak untuk rs_audit_relay dan rs_key_rotation (tabel maupun kolom)',
  ARRAY(SELECT format('%s: %s', c.relname, pg_get_userbyid(a.grantee))
          FROM pg_class c, aclexplode(c.relacl) a
         WHERE c.oid IN ('app.account'::regclass, 'app.rs_admin_role'::regclass, 'app.session_revocation'::regclass)
           AND pg_get_userbyid(a.grantee) IN ('rs_audit_relay', 'rs_key_rotation')
        UNION ALL
        SELECT format('%s.%s: %s', att.attrelid::regclass, att.attname, pg_get_userbyid(a.grantee))
          FROM pg_attribute att, aclexplode(att.attacl) a
         WHERE att.attrelid IN ('app.account'::regclass, 'app.rs_admin_role'::regclass, 'app.session_revocation'::regclass)
           AND pg_get_userbyid(a.grantee) <> 'rs_privacy_worker'));

-- 3. Test negatif atas pemeriksa DDL umum dan pemeriksa index, lalu perilaku. Semua di-ROLLBACK.
BEGIN;
DO $$
DECLARE
  c record;
  ddl text := $q$SELECT pg_temp.uji_ddl_umum('app')$q$;
BEGIN
  FOR c IN SELECT * FROM (VALUES
      ('KEPUTUSAN-93 id dengan default', 'ALTER TABLE app.session_revocation ALTER COLUMN id SET DEFAULT uuidv7()', ddl, 'session_revocation.id: kolom id punya nilai bawaan'),
      ('KEPUTUSAN-93 created_at tanpa default', 'ALTER TABLE app.account ALTER COLUMN created_at DROP DEFAULT', ddl, 'account.created_at: bukan NOT NULL DEFAULT now'),
      ('KEPUTUSAN-93 created_at boleh NULL', 'ALTER TABLE app.rs_admin_role ALTER COLUMN created_at DROP NOT NULL', ddl, 'rs_admin_role.created_at: bukan NOT NULL'),
      ('KEPUTUSAN-93 FK ON DELETE CASCADE', 'ALTER TABLE app.session_revocation DROP CONSTRAINT fk_session_revocation_account_id; ALTER TABLE app.session_revocation ADD CONSTRAINT fk_session_revocation_account_id FOREIGN KEY (account_id) REFERENCES app.account (id) ON DELETE CASCADE', ddl, 'FK fk_session_revocation_account_id ON DELETE/UPDATE bukan NO ACTION'),
      ('KEPUTUSAN-93 FK ON UPDATE SET NULL', 'ALTER TABLE app.rs_admin_role DROP CONSTRAINT fk_rs_admin_role_updated_by; ALTER TABLE app.rs_admin_role ADD CONSTRAINT fk_rs_admin_role_updated_by FOREIGN KEY (updated_by) REFERENCES app.account (id) ON UPDATE SET NULL', ddl, 'fk_rs_admin_role_updated_by ON DELETE/UPDATE bukan NO ACTION'),
      ('index tambahan terdeteksi', 'CREATE INDEX ix_account_email ON app.account (email)', $q$SELECT pg_temp.uji_indeks('app.account', ARRAY['CREATE UNIQUE INDEX pk_account ON app.account USING btree (id)', 'CREATE UNIQUE INDEX uqp_account_idp_realm_idp_subject ON app.account USING btree (idp_realm, idp_subject) WHERE (deleted_at IS NULL)', 'CREATE INDEX ix_account_status ON app.account USING btree (status) WHERE (deleted_at IS NULL)'])$q$, 'index app.account lebih: CREATE INDEX ix_account_email'),
      ('uqp tanpa WHERE deleted_at terdeteksi', 'DROP INDEX app.uqp_account_idp_realm_idp_subject; CREATE UNIQUE INDEX uqp_account_idp_realm_idp_subject ON app.account (idp_realm, idp_subject)', $q$SELECT pg_temp.uji_indeks('app.account', ARRAY['CREATE UNIQUE INDEX pk_account ON app.account USING btree (id)', 'CREATE UNIQUE INDEX uqp_account_idp_realm_idp_subject ON app.account USING btree (idp_realm, idp_subject) WHERE (deleted_at IS NULL)', 'CREATE INDEX ix_account_status ON app.account USING btree (status) WHERE (deleted_at IS NULL)'])$q$, 'index app.account kurang: .*WHERE \(deleted_at IS NULL\)'),
      ('T-07 UPDATE kolom non-anonymize account untuk pekerja privasi', 'GRANT UPDATE (status) ON app.account TO rs_privacy_worker', $q$SELECT pg_temp.uji_kelas_app('app', '{}')$q$, 'T-07 account'),
      ('T-06 DELETE account untuk rs_app', 'GRANT DELETE ON app.account TO rs_app', $q$SELECT pg_temp.uji_kelas_app('app', '{}')$q$, 'T-06 account .*lebih rs_app:DELETE'),
      ('T-06 SELECT session_revocation untuk pekerja privasi (O tanpa data pribadi)', 'GRANT SELECT ON app.session_revocation TO rs_privacy_worker', $q$SELECT pg_temp.uji_kelas_app('app', '{}')$q$, 'T-06 session_revocation .*lebih rs_privacy_worker:SELECT')
    ) AS t(label, stmt, pemeriksa, pola)
  LOOP
    PERFORM pg_temp.uji_rusak(c.label, c.stmt, c.pemeriksa, c.pola);
  END LOOP;
END $$;

-- Data awal oleh pemilik skema (seperti seed migrasi). Data sintetis.
SET LOCAL ROLE rs_migration;
INSERT INTO app.account (id, account_type, idp_realm, idp_subject, email, status, status_changed_at, full_name)
  VALUES ('00000000-0000-7000-8000-00000000a001', 'rs_admin', 'admin', 'sub-admin-1', 'admin1@contoh.invalid', 'active', now(), 'Admin Sintetis'),
         ('00000000-0000-7000-8000-00000000a002', 'patient', 'pasien', 'sub-pasien-1', 'pasien1@contoh.invalid', 'active', now(), NULL);
RESET ROLE;

-- account (M): S, I, U untuk rs_app; tanpa DELETE/TRUNCATE.
SELECT pg_temp.uji_boleh('A1.1 rs_api INSERT account pasien', 'rs_api',
  'INSERT INTO app.account (id, account_type, idp_realm, idp_subject, email, status, status_changed_at) VALUES (''00000000-0000-7000-8000-00000000a003'', ''patient'', ''pasien'', ''sub-pasien-2'', ''p2@contoh.invalid'', ''pending_activation'', now())', 1);
SELECT pg_temp.uji_boleh('A1.1 rs_api UPDATE status account (KU-10)', 'rs_api',
  'UPDATE app.account SET status = ''inactive'', status_reason = ''uji'', status_changed_at = now(), updated_at = now() WHERE id = ''00000000-0000-7000-8000-00000000a003''', 1);
SELECT pg_temp.uji_tolak('A1.1 NEG rs_api DELETE account (Master, ADR-003)', 'rs_api', 'DELETE FROM app.account', '42501', 'permission denied for table account');
SELECT pg_temp.uji_tolak('A1.1 NEG rs_worker TRUNCATE account', 'rs_worker', 'TRUNCATE app.account CASCADE', '42501', 'permission denied for table');
SELECT pg_temp.uji_tolak('A1.1 NEG pasangan account_type/idp_realm salah (patient di realm admin)', 'rs_api',
  'INSERT INTO app.account (id, account_type, idp_realm, idp_subject, email, status, status_changed_at) VALUES (''00000000-0000-7000-8000-00000000a004'', ''patient'', ''admin'', ''sub-x'', ''x@contoh.invalid'', ''active'', now())',
  '23514', 'ck_account_account_type_idp_realm');
-- PostgreSQL memeriksa CHECK menurut urutan nama, sehingga CHECK pasangan dapat menolak lebih dulu.
SELECT pg_temp.uji_tolak('A1.1 NEG realm di luar daftar', 'rs_api',
  'INSERT INTO app.account (id, account_type, idp_realm, idp_subject, email, status, status_changed_at) VALUES (''00000000-0000-7000-8000-00000000a004'', ''patient'', ''rme'', ''sub-x'', ''x@contoh.invalid'', ''active'', now())',
  '23514', 'ck_account_(account_type_)?idp_realm');
SELECT pg_temp.uji_tolak('A1.1 NEG admin tanpa full_name', 'rs_api',
  'INSERT INTO app.account (id, account_type, idp_realm, idp_subject, email, status, status_changed_at) VALUES (''00000000-0000-7000-8000-00000000a004'', ''partner_admin'', ''admin'', ''sub-x'', ''x@contoh.invalid'', ''active'', now())',
  '23514', 'ck_account_full_name');
SELECT pg_temp.uji_tolak('A1.1 NEG status di luar daftar', 'rs_api',
  'UPDATE app.account SET status = ''banned'' WHERE id = ''00000000-0000-7000-8000-00000000a002''', '23514', 'ck_account_status');
SELECT pg_temp.uji_tolak('A1.1 NEG subjek IdP ganda di realm yang sama (akun aktif)', 'rs_api',
  'INSERT INTO app.account (id, account_type, idp_realm, idp_subject, email, status, status_changed_at) VALUES (''00000000-0000-7000-8000-00000000a005'', ''patient'', ''pasien'', ''sub-pasien-1'', ''lain@contoh.invalid'', ''active'', now())',
  '23505', 'uqp_account_idp_realm_idp_subject');
SELECT pg_temp.uji_boleh('A1.1 subjek sama di realm lain diterima', 'rs_api',
  'INSERT INTO app.account (id, account_type, idp_realm, idp_subject, email, status, status_changed_at) VALUES (''00000000-0000-7000-8000-00000000a006'', ''health_worker'', ''nakes'', ''sub-pasien-1'', ''nakes@contoh.invalid'', ''active'', now())', 1);
SELECT pg_temp.uji_boleh('A1.1 email sama diterima (tanpa unik, KEPUTUSAN-45)', 'rs_api',
  'INSERT INTO app.account (id, account_type, idp_realm, idp_subject, email, status, status_changed_at) VALUES (''00000000-0000-7000-8000-00000000a007'', ''patient'', ''pasien'', ''sub-pasien-3'', ''pasien1@contoh.invalid'', ''active'', now())', 1);
SELECT pg_temp.uji_tolak('A1.1 NEG soft delete tanpa delete_reason', 'rs_api',
  'UPDATE app.account SET deleted_at = now() WHERE id = ''00000000-0000-7000-8000-00000000a007''', '23514', 'ck_account_soft_delete');
SELECT pg_temp.uji_tolak('A1.1 NEG delete_reason tanpa deleted_at', 'rs_api',
  'UPDATE app.account SET delete_reason = ''x'' WHERE id = ''00000000-0000-7000-8000-00000000a007''', '23514', 'ck_account_soft_delete');
SELECT pg_temp.uji_boleh('A1.1 soft delete dengan alasan dan pelaku', 'rs_api',
  'UPDATE app.account SET deleted_at = now(), deleted_by = ''00000000-0000-7000-8000-00000000a001'', delete_reason = ''uji'' WHERE id = ''00000000-0000-7000-8000-00000000a007''', 1);
SELECT pg_temp.uji_boleh('A1.1 subjek IdP akun terhapus dapat dipakai ulang (partial unique)', 'rs_api',
  'INSERT INTO app.account (id, account_type, idp_realm, idp_subject, email, status, status_changed_at) VALUES (''00000000-0000-7000-8000-00000000a008'', ''patient'', ''pasien'', ''sub-pasien-3'', ''p3@contoh.invalid'', ''active'', now())', 1);
SELECT pg_temp.uji_tolak('A1.1 NEG FK created_by ke akun yang tidak ada', 'rs_api',
  'INSERT INTO app.account (id, created_by, account_type, idp_realm, idp_subject, email, status, status_changed_at) VALUES (''00000000-0000-7000-8000-00000000a009'', ''00000000-0000-7000-8000-00000000ffff'', ''patient'', ''pasien'', ''sub-x'', ''x@contoh.invalid'', ''active'', now())',
  '23503', 'fk_account_created_by');
-- Pekerja privasi: hanya kolom anonymize + anonymized_at, updated_at, updated_by (T-07).
SELECT pg_temp.uji_boleh('A1.1 pekerja privasi menganonimkan email dan full_name', 'rs_privacy_worker_login',
  'UPDATE app.account SET email = ''anon@contoh.invalid'', full_name = ''Anonim'', anonymized_at = now(), updated_at = now(), updated_by = NULL WHERE id = ''00000000-0000-7000-8000-00000000a001''', 1);
SELECT pg_temp.uji_tolak('A1.1 NEG pekerja privasi mengubah status', 'rs_privacy_worker_login',
  'UPDATE app.account SET status = ''inactive'' WHERE id = ''00000000-0000-7000-8000-00000000a002''', '42501', 'permission denied for table account');
SELECT pg_temp.uji_tolak('A1.1 NEG pekerja privasi mengubah idp_subject', 'rs_privacy_worker_login',
  'UPDATE app.account SET idp_subject = ''lain'' WHERE id = ''00000000-0000-7000-8000-00000000a002''', '42501', 'permission denied for table account');
SELECT pg_temp.uji_tolak('A1.1 NEG pekerja privasi INSERT account', 'rs_privacy_worker_login',
  'INSERT INTO app.account (id, account_type, idp_realm, idp_subject, email, status, status_changed_at) VALUES (''00000000-0000-7000-8000-00000000a00a'', ''patient'', ''pasien'', ''sub-y'', ''y@contoh.invalid'', ''active'', now())',
  '42501', 'permission denied for table account');
SELECT pg_temp.uji_tolak('A1.1 NEG relay audit membaca account (di luar haknya)', 'rs_audit_relay_login', 'SELECT * FROM app.account', '42501', 'permission denied for table account');
SELECT pg_temp.uji_tolak('A1.1 NEG rotasi kunci membaca account (di luar haknya)', 'rs_key_rotation', 'SELECT id FROM app.account', '42501', 'permission denied for table account');

-- rs_admin_role (M).
SELECT pg_temp.uji_boleh('A1.2 rs_api INSERT sub-peran', 'rs_api',
  'INSERT INTO app.rs_admin_role (id, created_by, account_id, sub_role) VALUES (''00000000-0000-7000-8000-00000000b001'', ''00000000-0000-7000-8000-00000000a001'', ''00000000-0000-7000-8000-00000000a001'', ''super_admin'')', 1);
SELECT pg_temp.uji_tolak('A1.2 NEG sub-peran ganda yang masih aktif', 'rs_api',
  'INSERT INTO app.rs_admin_role (id, account_id, sub_role) VALUES (''00000000-0000-7000-8000-00000000b002'', ''00000000-0000-7000-8000-00000000a001'', ''super_admin'')',
  '23505', 'uqp_rs_admin_role_account_id_sub_role');
SELECT pg_temp.uji_tolak('A1.2 NEG sub-peran di luar daftar', 'rs_api',
  'INSERT INTO app.rs_admin_role (id, account_id, sub_role) VALUES (''00000000-0000-7000-8000-00000000b002'', ''00000000-0000-7000-8000-00000000a001'', ''root'')',
  '23514', 'ck_rs_admin_role_sub_role');
SELECT pg_temp.uji_tolak('A1.2 NEG akun yang tidak ada', 'rs_api',
  'INSERT INTO app.rs_admin_role (id, account_id, sub_role) VALUES (''00000000-0000-7000-8000-00000000b002'', ''00000000-0000-7000-8000-00000000ffff'', ''auditor'')',
  '23503', 'fk_rs_admin_role_account_id');
SELECT pg_temp.uji_boleh('A1.2 pencabutan sub-peran = soft delete', 'rs_api',
  'UPDATE app.rs_admin_role SET deleted_at = now(), deleted_by = ''00000000-0000-7000-8000-00000000a001'', delete_reason = ''dicabut'', updated_at = now() WHERE id = ''00000000-0000-7000-8000-00000000b001''', 1);
SELECT pg_temp.uji_boleh('A1.2 sub-peran yang dicabut dapat diberikan ulang', 'rs_api',
  'INSERT INTO app.rs_admin_role (id, account_id, sub_role) VALUES (''00000000-0000-7000-8000-00000000b003'', ''00000000-0000-7000-8000-00000000a001'', ''super_admin'')', 1);
SELECT pg_temp.uji_tolak('A1.2 NEG rs_api DELETE sub-peran', 'rs_api', 'DELETE FROM app.rs_admin_role', '42501', 'permission denied for table rs_admin_role');
SELECT pg_temp.uji_tolak('A1.2 NEG pekerja privasi UPDATE sub-peran (tanpa kolom anonymize)', 'rs_privacy_worker_login',
  'UPDATE app.rs_admin_role SET updated_at = now()', '42501', 'permission denied for table rs_admin_role');
SELECT pg_temp.uji_tolak('A1.2 NEG akun aktif tidak dapat dihapus keras walau oleh pemilik (FK NO ACTION)', 'rs_migration',
  'DELETE FROM app.account WHERE id = ''00000000-0000-7000-8000-00000000a001''', '23503', 'fk_');

-- session_revocation (O): S, I, U, D untuk rs_app; tanpa hak pekerja privasi.
SELECT pg_temp.uji_boleh('A1.4 rs_api mencatat pencabutan sesi', 'rs_api',
  'INSERT INTO app.session_revocation (id, account_id, idp_session_id, reason_code, expires_at) VALUES (''00000000-0000-7000-8000-00000000c001'', ''00000000-0000-7000-8000-00000000a002'', ''sid-1'', ''logout'', now() + interval ''1 hour'')', 1);
SELECT pg_temp.uji_tolak('A1.4 NEG sid yang sama dicatat dua kali', 'rs_api',
  'INSERT INTO app.session_revocation (id, account_id, idp_session_id, reason_code, expires_at) VALUES (''00000000-0000-7000-8000-00000000c002'', ''00000000-0000-7000-8000-00000000a002'', ''sid-1'', ''admin_revoke'', now() + interval ''1 hour'')',
  '23505', 'uq_session_revocation_idp_session_id');
SELECT pg_temp.uji_tolak('A1.4 NEG reason_code di luar daftar', 'rs_api',
  'INSERT INTO app.session_revocation (id, account_id, idp_session_id, reason_code, expires_at) VALUES (''00000000-0000-7000-8000-00000000c002'', ''00000000-0000-7000-8000-00000000a002'', ''sid-2'', ''lupa'', now())',
  '23514', 'ck_session_revocation_reason_code');
SELECT pg_temp.uji_tolak('A1.4 NEG expires_at wajib', 'rs_api',
  'INSERT INTO app.session_revocation (id, account_id, idp_session_id, reason_code) VALUES (''00000000-0000-7000-8000-00000000c002'', ''00000000-0000-7000-8000-00000000a002'', ''sid-2'', ''logout'')',
  '23502', 'expires_at');
SELECT pg_temp.uji_boleh('A1.4 rs_worker membersihkan baris kedaluwarsa (DELETE, kelas O)', 'rs_worker',
  'DELETE FROM app.session_revocation WHERE expires_at < now() + interval ''2 hours''', 1);
SELECT pg_temp.uji_tolak('A1.4 NEG pekerja privasi membaca session_revocation', 'rs_privacy_worker_login', 'SELECT * FROM app.session_revocation', '42501', 'permission denied for table session_revocation');
SELECT pg_temp.uji_tolak('A1.4 NEG relay audit membaca session_revocation', 'rs_audit_relay_login', 'SELECT * FROM app.session_revocation', '42501', 'permission denied for table session_revocation');
SELECT pg_temp.uji_tolak('A1.4 NEG rs_api TRUNCATE session_revocation', 'rs_api', 'TRUNCATE app.session_revocation', '42501', 'permission denied for table session_revocation');
ROLLBACK;

DO $$
BEGIN
  IF EXISTS (SELECT 1 FROM app.account) OR EXISTS (SELECT 1 FROM app.rs_admin_role) OR EXISTS (SELECT 1 FROM app.session_revocation) THEN
    RAISE EXCEPTION 'GAGAL pembersihan tabel-app-a1: data uji tersisa';
  END IF;
  RAISE NOTICE 'LULUS pembersihan tabel-app-a1: tidak ada data uji tersisa (ROLLBACK)';
END $$;
