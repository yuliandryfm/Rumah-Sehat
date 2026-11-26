-- db/test/tabel-app-a3.sql: test tabel A3 persetujuan (F1-T19) di database ruang_sehat:
-- consent_text_version, consent_item, consent_record.
-- Didahului db/test/lib.sql; dijalankan superuser setelah migrasi. Aturan kelas generik
-- (T-06/T-07/T-08, termasuk pola berlaku-mulai consent_text_version) di db/test/kelas-app.sql
-- bagian A, T-20 generik di db/test/katalog.sql, DDL umum §4.1 di db/test/tabel-app-a1.sql. Berkas
-- ini memeriksa struktur khusus (komentar kelas, hak per kolom, index, FK, CHECK, fungsi dan trigger
-- app.* kunci versi terbit, argumen penjaga DELETE Log) dan perilaku dengan role sungguhan (positif
-- dan negatif). Satu transaksi yang di-ROLLBACK. Data sintetis.
-- Dasar: docs/04-data-dictionary.md §0.3, §0.6 A, §0.8, A3.1-A3.3; docs/05-physical-schema.md
-- §4.2 (2), (3), (3b), (3c), (3d), §4.3, §4.9, §4.11, §12 (T-06, T-08, T-09); ADR-003, ADR-004,
-- ADR-008, ADR-013 §5; PRD v1.3 US-104 AC2-AC4, NFR-PRIV-05; KEPUTUSAN-154, KEPUTUSAN-159.

-- 1. Struktur.
SELECT pg_temp.uji_nol('A3 komentar kelas: M privasi (versi teks, butir), L privasi dengan kategori penanda pending_pt_24',
  ARRAY(SELECT format('%s: %s', t, coalesce(obj_description(to_regclass('app.' || t), 'pg_class'), '<tidak ada>'))
          FROM (VALUES ('consent_text_version', 'class:M module:privasi'), ('consent_item', 'class:M module:privasi'),
                       ('consent_record', 'class:L module:privasi log_category:pending_pt_24')) v(t, k)
         WHERE obj_description(to_regclass('app.' || t), 'pg_class') IS DISTINCT FROM k));
SELECT pg_temp.uji_nol('A3 tanpa kolom anonymize (DD A3 tanpa kolom "Anonimkan") dan tanpa kolom T-20 (*_enc/*_bidx)',
  ARRAY(SELECT format('%s.%s', attrelid::regclass, attname) FROM pg_attribute
         WHERE attrelid = ANY (ARRAY['app.consent_text_version', 'app.consent_item', 'app.consent_record']::regclass[]::oid[])
           AND attnum > 0 AND NOT attisdropped
           AND (col_description(attrelid, attnum) IS NOT NULL OR attname ~ '(_enc|_bidx|_bidx_key_version)$')));
SELECT pg_temp.uji_nol('A3 hak per kolom pekerja privasi persis: consent_record SELECT id, created_at, FK subjek (patient_id, care_recipient_id, account_id F1-T101) (DD §0.6 A Log); tanpa UPDATE per kolom',
  ARRAY(SELECT format('%s %s: %s', att.attrelid::regclass, x.privilege_type, string_agg(att.attname::text, ',' ORDER BY att.attname))
          FROM pg_attribute att, aclexplode(att.attacl) x
         WHERE att.attrelid = ANY (ARRAY['app.consent_text_version', 'app.consent_item', 'app.consent_record']::regclass[]::oid[])
           AND att.attnum > 0 AND NOT att.attisdropped
         GROUP BY att.attrelid, x.privilege_type, x.grantee
        HAVING NOT (att.attrelid = 'app.consent_record'::regclass AND x.privilege_type = 'SELECT'
                    AND pg_get_userbyid(x.grantee) = 'rs_privacy_worker'
                    AND string_agg(att.attname::text, ',' ORDER BY att.attname) = 'account_id,care_recipient_id,created_at,id,patient_id')
        UNION ALL
        SELECT 'consent_record: SELECT per kolom pekerja privasi tidak ada'
         WHERE NOT EXISTS (SELECT 1 FROM pg_attribute att, aclexplode(att.attacl) x
                            WHERE att.attrelid = 'app.consent_record'::regclass AND x.privilege_type = 'SELECT'
                              AND pg_get_userbyid(x.grantee) = 'rs_privacy_worker')));
SELECT pg_temp.uji_nol('A3 hak tabel persis (M: rs_app S,I,U + pekerja privasi S; L: rs_app S,I + pekerja privasi D); tanpa relay, rotasi kunci, monitor, PUBLIC',
  ARRAY(SELECT format('%s: %s, diharapkan %s', v.t, coalesce(a.k, '<kosong>'), v.k)
          FROM (VALUES ('consent_text_version', 'rs_app:INSERT,rs_app:SELECT,rs_app:UPDATE,rs_privacy_worker:SELECT'),
                       ('consent_item', 'rs_app:INSERT,rs_app:SELECT,rs_app:UPDATE,rs_privacy_worker:SELECT'),
                       ('consent_record', 'rs_app:INSERT,rs_app:SELECT,rs_privacy_worker:DELETE')) v(t, k),
               LATERAL (SELECT string_agg(CASE WHEN x.grantee = 0 THEN 'PUBLIC' ELSE pg_get_userbyid(x.grantee) END || ':' || x.privilege_type,
                                          ',' ORDER BY 1) AS k
                          FROM pg_class c, aclexplode(c.relacl) x
                         WHERE c.oid = to_regclass('app.' || v.t) AND x.grantee <> c.relowner) a
         WHERE a.k IS DISTINCT FROM v.k));

SELECT pg_temp.uji_nol('A3.1 consent_text_version: index persis pk, uqp (consent_type, version_label) aktif',
  ARRAY(SELECT pg_temp.uji_indeks('app.consent_text_version', ARRAY[
    'CREATE UNIQUE INDEX pk_consent_text_version ON app.consent_text_version USING btree (id)',
    'CREATE UNIQUE INDEX uqp_consent_text_version_consent_type_version_label ON app.consent_text_version USING btree (consent_type, version_label) WHERE (deleted_at IS NULL)'])));
SELECT pg_temp.uji_nol('A3.2 consent_item: index persis pk, uqp (consent_text_version_id, item_code) aktif',
  ARRAY(SELECT pg_temp.uji_indeks('app.consent_item', ARRAY[
    'CREATE UNIQUE INDEX pk_consent_item ON app.consent_item USING btree (id)',
    'CREATE UNIQUE INDEX uqp_consent_item_consent_text_version_id_item_code ON app.consent_item USING btree (consent_text_version_id, item_code) WHERE (deleted_at IS NULL)'])));
SELECT pg_temp.uji_nol('A3.3 consent_record: index persis pk, ix (patient_id, consent_item_id, created_at DESC) (US-1601 AC1), ix parsial per akun HR (F1-T101)',
  ARRAY(SELECT pg_temp.uji_indeks('app.consent_record', ARRAY[
    'CREATE UNIQUE INDEX pk_consent_record ON app.consent_record USING btree (id)',
    'CREATE INDEX ix_consent_record_patient_id_consent_item_id_created_at ON app.consent_record USING btree (patient_id, consent_item_id, created_at DESC)',
    'CREATE INDEX ix_consent_record_account_id_consent_item_id_created_at ON app.consent_record USING btree (account_id, consent_item_id, created_at DESC) WHERE (account_id IS NOT NULL)'])));
SELECT pg_temp.uji_nol('A3 FK persis DD A3 (butir -> versi; bukti -> pasien, penerima layanan, versi, butir; aktor -> account)',
  ARRAY(SELECT x FROM (VALUES
      ('app.consent_item', 'fk_consent_item_consent_text_version_id', 'FOREIGN KEY (consent_text_version_id) REFERENCES app.consent_text_version(id)'),
      ('app.consent_record', 'fk_consent_record_patient_id', 'FOREIGN KEY (patient_id) REFERENCES app.patient(id)'),
      ('app.consent_record', 'fk_consent_record_care_recipient_id', 'FOREIGN KEY (care_recipient_id) REFERENCES app.care_recipient(id)'),
      ('app.consent_record', 'fk_consent_record_account_id', 'FOREIGN KEY (account_id) REFERENCES app.account(id)'),
      ('app.consent_record', 'fk_consent_record_consent_text_version_id', 'FOREIGN KEY (consent_text_version_id) REFERENCES app.consent_text_version(id)'),
      ('app.consent_record', 'fk_consent_record_consent_item_id', 'FOREIGN KEY (consent_item_id) REFERENCES app.consent_item(id)'),
      ('app.consent_text_version', 'fk_consent_text_version_created_by', 'FOREIGN KEY (created_by) REFERENCES app.account(id)'),
      ('app.consent_item', 'fk_consent_item_deleted_by', 'FOREIGN KEY (deleted_by) REFERENCES app.account(id)')) v(t, n, def),
        LATERAL (SELECT format('%s: %s', v.n, coalesce(pg_get_constraintdef(k.oid), '<tidak ada>')) AS x
                   FROM (SELECT 1) d LEFT JOIN pg_constraint k ON k.conname = v.n AND k.conrelid = v.t::regclass) z
         WHERE x IS DISTINCT FROM format('%s: %s', v.n, v.def)
        UNION ALL
        SELECT format('jumlah FK %s = %s', t, n) FROM (VALUES ('app.consent_text_version', 3), ('app.consent_item', 4), ('app.consent_record', 5)) e(t, harap),
               LATERAL (SELECT count(*) AS n FROM pg_constraint WHERE conrelid = e.t::regclass AND contype = 'f') c
         WHERE c.n <> e.harap));
SELECT pg_temp.uji_nol('A3 CHECK persis DD (consent_type, action, channel) dan subjek bukti pasien XOR akun HR (F1-T101)',
  ARRAY(SELECT x FROM (VALUES
      ('app.consent_text_version', 'ck_consent_text_version_consent_type', 'CHECK ((consent_type = ANY (ARRAY[''patient_data_processing''::text, ''family_adult_statement''::text, ''guardian_consent''::text, ''corporate_hr_registration''::text])))'),
      ('app.consent_record', 'ck_consent_record_action', 'CHECK ((action = ANY (ARRAY[''granted''::text, ''withdrawn''::text])))'),
      ('app.consent_record', 'ck_consent_record_channel', 'CHECK ((channel = ANY (ARRAY[''patient_app''::text, ''backoffice''::text, ''corporate_portal''::text])))'),
      ('app.consent_record', 'ck_consent_record_subject', 'CHECK (((patient_id IS NOT NULL) <> (account_id IS NOT NULL)))'),
      ('app.consent_record', 'ck_consent_record_account_no_care_recipient', 'CHECK (((account_id IS NULL) OR (care_recipient_id IS NULL)))'),
      ('app.consent_record', 'ck_consent_record_channel_subject', 'CHECK (((account_id IS NOT NULL) = (channel = ''corporate_portal''::text)))')) v(t, n, def),
        LATERAL (SELECT format('%s: %s', v.n, coalesce(pg_get_constraintdef(k.oid), '<tidak ada>')) AS x
                   FROM (SELECT 1) d LEFT JOIN pg_constraint k ON k.conname = v.n AND k.conrelid = v.t::regclass) z
         WHERE x IS DISTINCT FROM format('%s: %s', v.n, v.def)));
-- Fungsi trigger app.* (di luar util.*, sehingga tidak tercakup T-08 generik).
SELECT pg_temp.uji_nol('A3 fungsi app.fn_consent_* (published_lock x2, server_time): plpgsql, SECURITY INVOKER, search_path persis, milik rs_migration, EXECUTE hanya pemilik',
  ARRAY(SELECT f FROM unnest(ARRAY['app.fn_consent_text_version_published_lock()', 'app.fn_consent_item_published_lock()',
                                   'app.fn_consent_record_server_time()']) f
         WHERE NOT EXISTS (
           SELECT 1 FROM pg_proc p
            WHERE p.oid = to_regprocedure(f)
              AND p.prorettype = 'trigger'::regtype AND NOT p.prosecdef
              AND p.prolang = (SELECT oid FROM pg_language WHERE lanname = 'plpgsql')
              AND p.proconfig = ARRAY['search_path=pg_catalog, pg_temp']
              AND p.proowner = 'rs_migration'::regrole AND p.proacl IS NOT NULL
              AND NOT EXISTS (SELECT 1 FROM aclexplode(p.proacl) a WHERE a.grantee <> p.proowner))));
SELECT pg_temp.uji_nol('A3.2 fn_consent_item_published_lock mengunci baris versi dengan FOR SHARE (race READ COMMITTED dengan penerbitan)',
  ARRAY(SELECT 'tanpa FOR SHARE' WHERE coalesce((SELECT prosrc FROM pg_proc WHERE oid = to_regprocedure('app.fn_consent_item_published_lock()')), '') !~ 'FOR SHARE'));
-- Semua trigger A3 persis (nama, tgtype, fungsi, argumen, aktif). tgtype: 1 ROW, 2 BEFORE, 4 INSERT,
-- 8 DELETE, 16 UPDATE, 32 TRUNCATE.
SELECT pg_temp.uji_nol('A3 trigger persis §4.11: versi (effective_lock, no_backdate rs_migration, published_lock), butir (published_lock INSERT/UPDATE), bukti L (reject_update, reject_truncate, guard_delete pending_pt_24 + end_action kosong)',
  ARRAY(WITH harap(rel, tgname, tgtype, fn, args) AS (VALUES
      ('app.consent_text_version', 'tg_consent_text_version_effective_lock', 1 | 2 | 16, 'util.fn_effective_row_lock', '{}'::text[]),
      ('app.consent_text_version', 'tg_consent_text_version_no_backdate', 1 | 2 | 4, 'util.fn_effective_row_no_backdate', '{rs_migration}'),
      ('app.consent_text_version', 'tg_consent_text_version_published_lock', 1 | 2 | 16, 'app.fn_consent_text_version_published_lock', '{}'),
      ('app.consent_item', 'tg_consent_item_published_lock', 1 | 2 | 4 | 16, 'app.fn_consent_item_published_lock', '{}'),
      ('app.consent_record', 'tg_consent_record_reject_update', 2 | 16, 'util.fn_reject_modification', '{}'),
      ('app.consent_record', 'tg_consent_record_reject_truncate', 2 | 32, 'util.fn_reject_modification', '{}'),
      ('app.consent_record', 'tg_consent_record_server_time', 1 | 2 | 4, 'app.fn_consent_record_server_time', '{}'),
      ('app.consent_record', 'tg_consent_record_guard_delete', 1 | 2 | 8, 'util.fn_guard_log_delete', '{pending_pt_24,created_at,"{}",rs_privacy_worker}')),
    aktual AS (
      SELECT g.tgrelid::regclass::text AS rel, g.tgname::text AS tgname, g.tgtype::int AS tgtype, g.tgfoid::regproc::text AS fn,
             CASE WHEN g.tgnargs = 0 THEN '{}'::text[]
                  ELSE string_to_array(regexp_replace(encode(g.tgargs, 'escape'), '\\000$', ''), '\000') END AS args,
             g.tgenabled
        FROM pg_trigger g
       WHERE g.tgrelid = ANY (ARRAY['app.consent_text_version', 'app.consent_item', 'app.consent_record']::regclass[]::oid[])
         AND NOT g.tgisinternal)
    SELECT format('kurang/beda %s.%s', h.rel, h.tgname) FROM harap h
     WHERE NOT EXISTS (SELECT 1 FROM aktual a WHERE a.rel = h.rel AND a.tgname = h.tgname AND a.tgtype = h.tgtype
                          AND a.fn = h.fn AND a.args = h.args AND a.tgenabled = 'O')
    UNION ALL
    SELECT format('lebih %s.%s', a.rel, a.tgname) FROM aktual a
     WHERE NOT EXISTS (SELECT 1 FROM harap h WHERE h.rel = a.rel AND h.tgname = a.tgname)));

-- 2. Negatif atas pemeriksa, lalu perilaku. Semua di-ROLLBACK.
BEGIN;
DO $$
DECLARE
  c record;
  kelas text := $q$SELECT pg_temp.uji_kelas_app('app', '{consent_text_version}')$q$;
  trg text := $q$SELECT 'published_lock hilang/nonaktif' FROM pg_trigger g WHERE g.tgrelid = ANY (ARRAY['app.consent_text_version', 'app.consent_item']::regclass[]::oid[]) AND g.tgname LIKE '%published_lock' AND g.tgenabled = 'O' HAVING count(*) < 2$q$;
BEGIN
  FOR c IN SELECT * FROM (VALUES
      ('T-06 UPDATE bukti persetujuan untuk rs_app', 'GRANT UPDATE ON app.consent_record TO rs_app', kelas, 'T-06 consent_record .*lebih rs_app:UPDATE'),
      ('T-06 DELETE bukti persetujuan untuk rs_app', 'GRANT DELETE ON app.consent_record TO rs_app', kelas, 'T-06 consent_record .*lebih rs_app:DELETE'),
      ('T-06 DELETE versi teks untuk rs_app (Master)', 'GRANT DELETE ON app.consent_text_version TO rs_app', kelas, 'T-06 consent_text_version .*lebih rs_app:DELETE'),
      ('T-06 pekerja privasi membaca action (bukan ID/FK subjek)', 'GRANT SELECT (action) ON app.consent_record TO rs_privacy_worker', kelas, 'T-06 consent_record: SELECT per kolom rs_privacy_worker'),
      ('T-06 DELETE pekerja privasi dicabut', 'REVOKE DELETE ON app.consent_record FROM rs_privacy_worker', kelas, 'T-06 consent_record .*kurang rs_privacy_worker:DELETE'),
      ('T-07 pekerja privasi UPDATE butir (tanpa kolom anonymize)', 'GRANT UPDATE (label) ON app.consent_item TO rs_privacy_worker', kelas, 'T-07 consent_item'),
      ('T-08 penjaga DELETE bukti hilang', 'DROP TRIGGER tg_consent_record_guard_delete ON app.consent_record', kelas, 'T-08 consent_record \(kelas L\)'),
      ('T-08 penolak UPDATE bukti nonaktif', 'ALTER TABLE app.consent_record DISABLE TRIGGER tg_consent_record_reject_update', kelas, 'T-08 consent_record: trigger tg_consent_record_reject_update tidak aktif'),
      ('T-08 effective_lock versi teks hilang', 'DROP TRIGGER tg_consent_text_version_effective_lock ON app.consent_text_version', kelas, 'T-08 consent_text_version \(kelas M\)'),
      ('T-08 no_backdate versi teks dengan role lain', 'DROP TRIGGER tg_consent_text_version_no_backdate ON app.consent_text_version; CREATE TRIGGER tg_consent_text_version_no_backdate BEFORE INSERT ON app.consent_text_version FOR EACH ROW EXECUTE FUNCTION util.fn_effective_row_no_backdate(''rs_app'')', kelas, 'T-08 consent_text_version: argumen tg_consent_text_version_no_backdate'),
      ('published_lock versi teks hilang', 'DROP TRIGGER tg_consent_text_version_published_lock ON app.consent_text_version', trg, '.'),
      ('published_lock butir nonaktif', 'ALTER TABLE app.consent_item DISABLE TRIGGER tg_consent_item_published_lock', trg, '.'),
      ('trigger jam server bukti hilang', 'DROP TRIGGER tg_consent_record_server_time ON app.consent_record',
       $q$SELECT 'server_time hilang' WHERE NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgrelid = 'app.consent_record'::regclass AND tgname = 'tg_consent_record_server_time' AND tgenabled = 'O')$q$, 'server_time hilang'),
      ('FOR SHARE dihapus dari kunci butir', 'CREATE OR REPLACE FUNCTION app.fn_consent_item_published_lock() RETURNS trigger LANGUAGE plpgsql SET search_path = pg_catalog, pg_temp AS ''BEGIN RETURN NEW; END''',
       $q$SELECT 'tanpa FOR SHARE' WHERE (SELECT prosrc FROM pg_proc WHERE oid = to_regprocedure('app.fn_consent_item_published_lock()')) !~ 'FOR SHARE'$q$, 'tanpa FOR SHARE')
    ) AS t(label, stmt, pemeriksa, pola)
  LOOP
    PERFORM pg_temp.uji_rusak(c.label, c.stmt, c.pemeriksa, c.pola);
  END LOOP;
END $$;

-- Data awal oleh pemilik skema. Data sintetis. Versi ...a3101 sudah berlaku (seed lampau, terbit).
SET LOCAL ROLE rs_migration;
INSERT INTO app.account (id, account_type, idp_realm, idp_subject, email, status, status_changed_at, full_name)
  VALUES ('00000000-0000-7000-8000-0000000a3001', 'patient', 'pasien', 'sub-a3-1', 'a3-1@contoh.invalid', 'active', now(), NULL);
INSERT INTO app.account (id, account_type, idp_realm, idp_subject, email, status, status_changed_at, full_name)
  VALUES ('00000000-0000-7000-8000-0000000a3002', 'corporate_admin', 'korporat', 'sub-a3-hr', 'a3-hr@contoh.invalid', 'active', now(), 'HR Sintetis');
INSERT INTO app.patient (id, account_id, processing_status)
  VALUES ('00000000-0000-7000-8000-0000000a3011', '00000000-0000-7000-8000-0000000a3001', 'normal');
INSERT INTO app.care_recipient (id, patient_id, is_account_owner, relationship, full_name, birth_date, sex)
  VALUES ('00000000-0000-7000-8000-0000000a3021', '00000000-0000-7000-8000-0000000a3011', true, NULL, 'Pemilik Sintetis', '1990-01-01', 'female'),
         ('00000000-0000-7000-8000-0000000a3022', '00000000-0000-7000-8000-0000000a3011', false, 'parent', 'Orang Tua Sintetis', '1960-01-01', 'male');
INSERT INTO app.consent_text_version (id, consent_type, version_label, content, effective_from, published_at)
  VALUES ('00000000-0000-7000-8000-0000000a3101', 'patient_data_processing', 'seed-lampau', 'Teks sintetis (menunggu L-08/PT-24)',
          now() - interval '1 day', now() - interval '2 days');
RESET ROLE;

-- consent_text_version (M, A3.1): pola berlaku-mulai dan kunci versi terbit.
SELECT pg_temp.uji_boleh('A3.1 rs_api membuat versi teks terjadwal (belum terbit)', 'rs_api',
  'INSERT INTO app.consent_text_version (id, consent_type, version_label, content, effective_from) VALUES '
  || '(''00000000-0000-7000-8000-0000000a3102'', ''patient_data_processing'', ''v2'', ''Teks v2 sintetis'', now() + interval ''7 days''), '
  || '(''00000000-0000-7000-8000-0000000a3103'', ''guardian_consent'', ''v1'', ''Teks wali sintetis'', now() + interval ''7 days''), '
  || '(''00000000-0000-7000-8000-0000000a3104'', ''family_adult_statement'', ''v1'', ''Teks keluarga sintetis'', now() + interval ''7 days'')', 3);
SELECT pg_temp.uji_tolak('A3.1 NEG rs_api versi teks dengan effective_from lampau (no_backdate)', 'rs_api',
  'INSERT INTO app.consent_text_version (id, consent_type, version_label, content, effective_from) VALUES (''00000000-0000-7000-8000-0000000a3105'', ''patient_data_processing'', ''v3'', ''x'', now() - interval ''1 minute'')',
  '42501', '^effective_row_backdated: app.consent_text_version');
SELECT pg_temp.uji_tolak('A3.1 NEG label versi ganda dalam jenis yang sama (aktif)', 'rs_api',
  'INSERT INTO app.consent_text_version (id, consent_type, version_label, content, effective_from) VALUES (''00000000-0000-7000-8000-0000000a3105'', ''patient_data_processing'', ''v2'', ''x'', now() + interval ''8 days'')',
  '23505', 'uqp_consent_text_version_consent_type_version_label');
SELECT pg_temp.uji_tolak('A3.1 NEG consent_type di luar daftar', 'rs_api',
  'INSERT INTO app.consent_text_version (id, consent_type, version_label, content, effective_from) VALUES (''00000000-0000-7000-8000-0000000a3105'', ''marketing'', ''v1'', ''x'', now() + interval ''8 days'')',
  '23514', 'ck_consent_text_version_consent_type');
SELECT pg_temp.uji_tolak('A3.1 NEG soft delete tanpa alasan', 'rs_api',
  'UPDATE app.consent_text_version SET deleted_at = now() WHERE id = ''00000000-0000-7000-8000-0000000a3104''', '23514', 'ck_consent_text_version_soft_delete');
SELECT pg_temp.uji_boleh('A3.1 rs_api mengubah isi versi yang belum terbit', 'rs_api',
  'UPDATE app.consent_text_version SET content = ''Teks v2 sintetis revisi'', updated_at = now() WHERE id = ''00000000-0000-7000-8000-0000000a3102''', 1);

-- consent_item (M, A3.2) pada versi belum terbit.
SELECT pg_temp.uji_boleh('A3.2 rs_api menambah butir ke versi belum terbit', 'rs_api',
  'INSERT INTO app.consent_item (id, consent_text_version_id, item_code, label, is_primary, is_required, sort_order) VALUES '
  || '(''00000000-0000-7000-8000-0000000a3201'', ''00000000-0000-7000-8000-0000000a3102'', ''data_processing'', ''Butir utama sintetis'', true, true, 1), '
  || '(''00000000-0000-7000-8000-0000000a3202'', ''00000000-0000-7000-8000-0000000a3102'', ''research'', ''Butir opsional sintetis'', false, false, 2), '
  || '(''00000000-0000-7000-8000-0000000a3203'', ''00000000-0000-7000-8000-0000000a3103'', ''guardian'', ''Butir wali sintetis'', true, true, 1)', 3);
SELECT pg_temp.uji_tolak('A3.2 NEG kode butir ganda dalam versi yang sama (aktif)', 'rs_api',
  'INSERT INTO app.consent_item (id, consent_text_version_id, item_code, label, is_primary, is_required, sort_order) VALUES (''00000000-0000-7000-8000-0000000a3204'', ''00000000-0000-7000-8000-0000000a3102'', ''research'', ''x'', false, false, 3)',
  '23505', 'uqp_consent_item_consent_text_version_id_item_code');
SELECT pg_temp.uji_tolak('A3.2 NEG butir untuk versi yang tidak ada', 'rs_api',
  'INSERT INTO app.consent_item (id, consent_text_version_id, item_code, label, is_primary, is_required, sort_order) VALUES (''00000000-0000-7000-8000-0000000a3204'', ''00000000-0000-7000-8000-00000000ffff'', ''x'', ''x'', false, false, 3)',
  '23503', 'fk_consent_item_consent_text_version_id');
SELECT pg_temp.uji_boleh('A3.2 rs_api mengubah butir versi belum terbit', 'rs_api',
  'UPDATE app.consent_item SET label = ''Butir opsional sintetis revisi'', updated_at = now() WHERE id = ''00000000-0000-7000-8000-0000000a3202''', 1);

-- Penerbitan (US-104 AC3): butir lengkap dulu, lalu published_at diisi.
SELECT pg_temp.uji_boleh('A3.1 rs_api menerbitkan versi terjadwal (effective_from masa depan)', 'rs_api',
  'UPDATE app.consent_text_version SET published_at = now(), updated_at = now() WHERE id IN (''00000000-0000-7000-8000-0000000a3102'', ''00000000-0000-7000-8000-0000000a3103'')', 2);
SELECT pg_temp.uji_tolak('A3.1 NEG versi terbit: content diubah (published_lock)', 'rs_api',
  'UPDATE app.consent_text_version SET content = ''diubah'' WHERE id = ''00000000-0000-7000-8000-0000000a3102''', '42501', '^published_row_locked: consent_text_version id=00000000-0000-7000-8000-0000000a3102$');
SELECT pg_temp.uji_tolak('A3.1 NEG versi terbit: version_label diubah', 'rs_api',
  'UPDATE app.consent_text_version SET version_label = ''v2b'' WHERE id = ''00000000-0000-7000-8000-0000000a3102''', '42501', '^published_row_locked: consent_text_version');
SELECT pg_temp.uji_tolak('A3.1 NEG versi terbit: consent_type diubah', 'rs_api',
  'UPDATE app.consent_text_version SET consent_type = ''guardian_consent'' WHERE id = ''00000000-0000-7000-8000-0000000a3102''', '42501', '^published_row_locked: consent_text_version');
SELECT pg_temp.uji_tolak('A3.1 NEG versi terbit: effective_from dimundurkan tetap di masa depan', 'rs_api',
  'UPDATE app.consent_text_version SET effective_from = effective_from + interval ''1 day'' WHERE id = ''00000000-0000-7000-8000-0000000a3102''', '42501', '^published_row_locked: consent_text_version');
SELECT pg_temp.uji_tolak('A3.1 NEG versi terbit: published_at dikosongkan', 'rs_api',
  'UPDATE app.consent_text_version SET published_at = NULL WHERE id = ''00000000-0000-7000-8000-0000000a3102''', '42501', '^published_row_locked: consent_text_version');
SELECT pg_temp.uji_tolak('A3.1 NEG versi terbit: pemilik skema pun ditolak', 'rs_migration',
  'UPDATE app.consent_text_version SET content = ''diubah'' WHERE id = ''00000000-0000-7000-8000-0000000a3102''', '42501', '^published_row_locked: consent_text_version');
SELECT pg_temp.uji_boleh('A3.1 pemberitahuan perubahan dicatat sekali (US-104 AC4)', 'rs_api',
  'UPDATE app.consent_text_version SET change_notice_sent_at = now(), updated_at = now() WHERE id = ''00000000-0000-7000-8000-0000000a3102''', 1);
SELECT pg_temp.uji_tolak('A3.1 NEG change_notice_sent_at diisi ulang', 'rs_api',
  'UPDATE app.consent_text_version SET change_notice_sent_at = now() + interval ''1 hour'' WHERE id = ''00000000-0000-7000-8000-0000000a3102''', '42501', '^published_row_locked: change_notice_sent_at id=');
SELECT pg_temp.uji_tolak('A3.1 NEG change_notice_sent_at dikosongkan', 'rs_api',
  'UPDATE app.consent_text_version SET change_notice_sent_at = NULL WHERE id = ''00000000-0000-7000-8000-0000000a3102''', '42501', '^published_row_locked: change_notice_sent_at');
SELECT pg_temp.uji_boleh('A3.1 versi terbit yang belum berlaku dibatalkan dengan soft delete (§4.2 (3c), DD §0.8 butir 2)', 'rs_api',
  'UPDATE app.consent_text_version SET deleted_at = now(), deleted_by = ''00000000-0000-7000-8000-0000000a3001'', delete_reason = ''dibatalkan'', updated_at = now() WHERE id = ''00000000-0000-7000-8000-0000000a3103''', 1);
SELECT pg_temp.uji_tolak('A3.1 NEG versi yang sudah berlaku diubah rs_api (effective_lock)', 'rs_api',
  'UPDATE app.consent_text_version SET change_notice_sent_at = now() WHERE id = ''00000000-0000-7000-8000-0000000a3101''', '42501', '^effective_row_locked: app.consent_text_version');
SELECT pg_temp.uji_tolak('A3.1 NEG versi yang sudah berlaku di-soft delete pemilik skema (effective_lock)', 'rs_migration',
  'UPDATE app.consent_text_version SET deleted_at = now(), delete_reason = ''x'' WHERE id = ''00000000-0000-7000-8000-0000000a3101''', '42501', '^effective_row_locked');
SELECT pg_temp.uji_tolak('A3.1 NEG pemilik skema menerbitkan versi lampau yang belum terbit (effective_lock; seed F1-T49 wajib terbit saat INSERT atau effective_from masa depan)', 'rs_migration',
  'INSERT INTO app.consent_text_version (id, consent_type, version_label, content, effective_from) VALUES (''00000000-0000-7000-8000-0000000a3106'', ''guardian_consent'', ''seed-belum-terbit'', ''x'', now() - interval ''1 day''); '
  || 'UPDATE app.consent_text_version SET published_at = now() WHERE id = ''00000000-0000-7000-8000-0000000a3106''',
  '42501', '^effective_row_locked: app.consent_text_version');
SELECT pg_temp.uji_tolak('A3.1 NEG rs_api DELETE versi teks (Master)', 'rs_api', 'DELETE FROM app.consent_text_version', '42501', 'permission denied for table consent_text_version');
SELECT pg_temp.uji_tolak('A3.1 NEG pekerja privasi UPDATE versi teks', 'rs_privacy_worker_login',
  'UPDATE app.consent_text_version SET content = ''x''', '42501', 'permission denied for table consent_text_version');

-- consent_item milik versi terbit (§4.2 (3d)).
SELECT pg_temp.uji_tolak('A3.2 NEG butir baru ke versi terbit', 'rs_api',
  'INSERT INTO app.consent_item (id, consent_text_version_id, item_code, label, is_primary, is_required, sort_order) VALUES (''00000000-0000-7000-8000-0000000a3205'', ''00000000-0000-7000-8000-0000000a3102'', ''marketing'', ''x'', false, false, 3)',
  '42501', '^published_row_locked: consent_item \(version published\) id=00000000-0000-7000-8000-0000000a3205$');
SELECT pg_temp.uji_tolak('A3.2 NEG butir baru ke versi terbit oleh pemilik skema', 'rs_migration',
  'INSERT INTO app.consent_item (id, consent_text_version_id, item_code, label, is_primary, is_required, sort_order) VALUES (''00000000-0000-7000-8000-0000000a3205'', ''00000000-0000-7000-8000-0000000a3101'', ''marketing'', ''x'', false, false, 3)',
  '42501', '^published_row_locked: consent_item');
SELECT pg_temp.uji_tolak('A3.2 NEG label butir versi terbit diubah', 'rs_api',
  'UPDATE app.consent_item SET label = ''diubah'' WHERE id = ''00000000-0000-7000-8000-0000000a3201''', '42501', '^published_row_locked: consent_item');
SELECT pg_temp.uji_tolak('A3.2 NEG butir versi terbit di-soft delete', 'rs_api',
  'UPDATE app.consent_item SET deleted_at = now(), delete_reason = ''x'' WHERE id = ''00000000-0000-7000-8000-0000000a3202''', '42501', '^published_row_locked: consent_item');
SELECT pg_temp.uji_tolak('A3.2 NEG butir dipindah keluar dari versi terbit', 'rs_api',
  'UPDATE app.consent_item SET consent_text_version_id = ''00000000-0000-7000-8000-0000000a3104'' WHERE id = ''00000000-0000-7000-8000-0000000a3201''', '42501', '^published_row_locked: consent_item');
SELECT pg_temp.uji_boleh('A3.2 butir ke versi belum terbit (keluarga dewasa)', 'rs_api',
  'INSERT INTO app.consent_item (id, consent_text_version_id, item_code, label, is_primary, is_required, sort_order) VALUES (''00000000-0000-7000-8000-0000000a3206'', ''00000000-0000-7000-8000-0000000a3104'', ''family_statement'', ''Butir keluarga sintetis'', true, true, 1)', 1);
SELECT pg_temp.uji_tolak('A3.2 NEG butir dipindah dari versi belum terbit ke versi terbit', 'rs_api',
  'UPDATE app.consent_item SET consent_text_version_id = ''00000000-0000-7000-8000-0000000a3102'' WHERE id = ''00000000-0000-7000-8000-0000000a3206''', '42501', '^published_row_locked: consent_item');
SELECT pg_temp.uji_tolak('A3.2 NEG rs_api DELETE butir (Master)', 'rs_api', 'DELETE FROM app.consent_item', '42501', 'permission denied for table consent_item');

-- consent_record (L, A3.3): bukti diberikan dan ditarik (US-104 AC2, US-1601).
SELECT pg_temp.uji_boleh('A3.3 rs_api mencatat persetujuan diberikan (pemilik akun) dan pernyataan keluarga', 'rs_api',
  'INSERT INTO app.consent_record (id, patient_id, care_recipient_id, consent_text_version_id, consent_item_id, action, channel, correlation_id) VALUES '
  || '(''00000000-0000-7000-8000-0000000a3301'', ''00000000-0000-7000-8000-0000000a3011'', NULL, ''00000000-0000-7000-8000-0000000a3102'', ''00000000-0000-7000-8000-0000000a3201'', ''granted'', ''patient_app'', ''00000000-0000-7000-8000-0000000a3c01''), '
  || '(''00000000-0000-7000-8000-0000000a3302'', ''00000000-0000-7000-8000-0000000a3011'', ''00000000-0000-7000-8000-0000000a3022'', ''00000000-0000-7000-8000-0000000a3104'', ''00000000-0000-7000-8000-0000000a3206'', ''granted'', ''patient_app'', ''00000000-0000-7000-8000-0000000a3c02'')', 2);
SELECT pg_temp.uji_boleh('A3.3 penarikan = baris baru (append-only), lewat backoffice', 'rs_api',
  'INSERT INTO app.consent_record (id, patient_id, consent_text_version_id, consent_item_id, action, channel, correlation_id) VALUES (''00000000-0000-7000-8000-0000000a3303'', ''00000000-0000-7000-8000-0000000a3011'', ''00000000-0000-7000-8000-0000000a3102'', ''00000000-0000-7000-8000-0000000a3201'', ''withdrawn'', ''backoffice'', ''00000000-0000-7000-8000-0000000a3c03'')', 1);
SELECT pg_temp.uji_tolak('A3.3 NEG action di luar daftar', 'rs_api',
  'INSERT INTO app.consent_record (id, patient_id, consent_text_version_id, consent_item_id, action, channel, correlation_id) VALUES (''00000000-0000-7000-8000-0000000a3304'', ''00000000-0000-7000-8000-0000000a3011'', ''00000000-0000-7000-8000-0000000a3102'', ''00000000-0000-7000-8000-0000000a3201'', ''revoked'', ''patient_app'', ''00000000-0000-7000-8000-0000000a3c04'')',
  '23514', 'ck_consent_record_action');
SELECT pg_temp.uji_tolak('A3.3 NEG channel di luar daftar', 'rs_api',
  'INSERT INTO app.consent_record (id, patient_id, consent_text_version_id, consent_item_id, action, channel, correlation_id) VALUES (''00000000-0000-7000-8000-0000000a3304'', ''00000000-0000-7000-8000-0000000a3011'', ''00000000-0000-7000-8000-0000000a3102'', ''00000000-0000-7000-8000-0000000a3201'', ''granted'', ''email'', ''00000000-0000-7000-8000-0000000a3c04'')',
  '23514', 'ck_consent_record_channel');
SELECT pg_temp.uji_tolak('A3.3 NEG tanpa correlation_id (penghubung E-13/E-15)', 'rs_api',
  'INSERT INTO app.consent_record (id, patient_id, consent_text_version_id, consent_item_id, action, channel) VALUES (''00000000-0000-7000-8000-0000000a3304'', ''00000000-0000-7000-8000-0000000a3011'', ''00000000-0000-7000-8000-0000000a3102'', ''00000000-0000-7000-8000-0000000a3201'', ''granted'', ''patient_app'')',
  '23502', 'correlation_id');
SELECT pg_temp.uji_tolak('A3.3 NEG pasien yang tidak ada', 'rs_api',
  'INSERT INTO app.consent_record (id, patient_id, consent_text_version_id, consent_item_id, action, channel, correlation_id) VALUES (''00000000-0000-7000-8000-0000000a3304'', ''00000000-0000-7000-8000-00000000ffff'', ''00000000-0000-7000-8000-0000000a3102'', ''00000000-0000-7000-8000-0000000a3201'', ''granted'', ''patient_app'', ''00000000-0000-7000-8000-0000000a3c04'')',
  '23503', 'fk_consent_record_patient_id');
SELECT pg_temp.uji_tolak('A3.3 NEG butir yang tidak ada', 'rs_api',
  'INSERT INTO app.consent_record (id, patient_id, consent_text_version_id, consent_item_id, action, channel, correlation_id) VALUES (''00000000-0000-7000-8000-0000000a3304'', ''00000000-0000-7000-8000-0000000a3011'', ''00000000-0000-7000-8000-0000000a3102'', ''00000000-0000-7000-8000-00000000ffff'', ''granted'', ''patient_app'', ''00000000-0000-7000-8000-0000000a3c04'')',
  '23503', 'fk_consent_record_consent_item_id');
-- Kelas L: role aplikasi tanpa UPDATE/DELETE/TRUNCATE; pemilik ditolak trigger (ADR-004, §4.9).
SELECT pg_temp.uji_tolak('A3.3 NEG rs_api UPDATE bukti persetujuan', 'rs_api', 'UPDATE app.consent_record SET action = ''withdrawn''', '42501', 'permission denied for table consent_record');
SELECT pg_temp.uji_tolak('A3.3 NEG rs_api DELETE bukti persetujuan', 'rs_api', 'DELETE FROM app.consent_record', '42501', 'permission denied for table consent_record');
SELECT pg_temp.uji_tolak('A3.3 NEG rs_worker TRUNCATE bukti persetujuan', 'rs_worker', 'TRUNCATE app.consent_record', '42501', 'permission denied for table consent_record');
SELECT pg_temp.uji_tolak('A3.3 NEG pemilik UPDATE bukti ditolak trigger', 'rs_migration', 'UPDATE app.consent_record SET channel = ''backoffice''', '42501', '^immutable_violation: UPDATE on app.consent_record rejected$');
SELECT pg_temp.uji_tolak('A3.3 NEG pemilik UPDATE nol baris tetap ditolak (trigger statement)', 'rs_migration', 'UPDATE app.consent_record SET channel = ''backoffice'' WHERE false', '42501', '^immutable_violation: UPDATE on app.consent_record rejected$');
SELECT pg_temp.uji_tolak('A3.3 NEG pemilik DELETE bukti ditolak penjaga (bukan pekerja privasi)', 'rs_migration', 'DELETE FROM app.consent_record', '42501', '^immutable_violation: DELETE on app.consent_record rejected$');
SELECT pg_temp.uji_tolak('A3.3 NEG pemilik TRUNCATE bukti ditolak trigger', 'rs_migration', 'TRUNCATE app.consent_record', '42501', '^immutable_violation: TRUNCATE on app.consent_record rejected$');
SELECT pg_temp.uji_tolak('A3.3 NEG ON CONFLICT DO UPDATE bukti ditolak (trigger BEFORE UPDATE statement)', 'rs_migration',
  'INSERT INTO app.consent_record (id, patient_id, consent_text_version_id, consent_item_id, action, channel, correlation_id) VALUES (''00000000-0000-7000-8000-0000000a3301'', ''00000000-0000-7000-8000-0000000a3011'', ''00000000-0000-7000-8000-0000000a3102'', ''00000000-0000-7000-8000-0000000a3201'', ''granted'', ''patient_app'', ''00000000-0000-7000-8000-0000000a3c05'') ON CONFLICT (id) DO UPDATE SET action = ''withdrawn''',
  '42501', '^immutable_violation: UPDATE on app.consent_record rejected$');
-- Pekerja privasi: hanya kolom ID/subjek; DELETE ditolak selama kategori/end_action menunggu PT-24.
SELECT pg_temp.uji_boleh('A3.3 pekerja privasi membaca kolom ID, created_at, FK subjek', 'rs_privacy_worker_login',
  'SELECT id, created_at, patient_id, care_recipient_id FROM app.consent_record', 3);
SELECT pg_temp.uji_tolak('A3.3 NEG pekerja privasi membaca action', 'rs_privacy_worker_login', 'SELECT action FROM app.consent_record', '42501', 'permission denied for table consent_record');
SELECT pg_temp.uji_tolak('A3.3 NEG pekerja privasi UPDATE bukti', 'rs_privacy_worker_login', 'UPDATE app.consent_record SET created_at = now()', '42501', 'permission denied for table consent_record');
SELECT pg_temp.uji_tolak('A3.3 NEG pekerja privasi TRUNCATE bukti', 'rs_privacy_worker_login', 'TRUNCATE app.consent_record', '42501', 'permission denied for table consent_record');
SELECT pg_temp.uji_tolak('A3.3 NEG relay audit membaca bukti', 'rs_audit_relay_login', 'SELECT id FROM app.consent_record', '42501', 'permission denied for table consent_record');
-- Subjek HR perusahaan (F1-T101): seed draf teks HR 01a2008d-7400-7000-8000-000000000b01 dan butir 01a2008d-7400-7000-8000-000000000b02 (migrasi 20261126310000).
SELECT pg_temp.uji_boleh('A3.3 F1-T101 rs_api mencatat persetujuan akun HR (kanal corporate_portal)', 'rs_api',
  'INSERT INTO app.consent_record (id, account_id, consent_text_version_id, consent_item_id, action, channel, correlation_id) VALUES (''00000000-0000-7000-8000-0000000a3311'', ''00000000-0000-7000-8000-0000000a3002'', ''01a2008d-7400-7000-8000-000000000b01'', ''01a2008d-7400-7000-8000-000000000b02'', ''granted'', ''corporate_portal'', ''00000000-0000-7000-8000-0000000a3c11'')', 1);
SELECT pg_temp.uji_tolak('A3.3 NEG F1-T101 bukti tanpa subjek (pasien dan akun kosong)', 'rs_api',
  'INSERT INTO app.consent_record (id, consent_text_version_id, consent_item_id, action, channel, correlation_id) VALUES (''00000000-0000-7000-8000-0000000a3312'', ''01a2008d-7400-7000-8000-000000000b01'', ''01a2008d-7400-7000-8000-000000000b02'', ''granted'', ''patient_app'', ''00000000-0000-7000-8000-0000000a3c12'')',
  '23514', 'ck_consent_record_subject');
SELECT pg_temp.uji_tolak('A3.3 NEG F1-T101 bukti dengan dua subjek (pasien dan akun)', 'rs_api',
  'INSERT INTO app.consent_record (id, patient_id, account_id, consent_text_version_id, consent_item_id, action, channel, correlation_id) VALUES (''00000000-0000-7000-8000-0000000a3312'', ''00000000-0000-7000-8000-0000000a3011'', ''00000000-0000-7000-8000-0000000a3002'', ''01a2008d-7400-7000-8000-000000000b01'', ''01a2008d-7400-7000-8000-000000000b02'', ''granted'', ''corporate_portal'', ''00000000-0000-7000-8000-0000000a3c12'')',
  '23514', 'ck_consent_record_subject');
SELECT pg_temp.uji_tolak('A3.3 NEG F1-T101 subjek akun dengan penerima layanan', 'rs_api',
  'INSERT INTO app.consent_record (id, account_id, care_recipient_id, consent_text_version_id, consent_item_id, action, channel, correlation_id) VALUES (''00000000-0000-7000-8000-0000000a3312'', ''00000000-0000-7000-8000-0000000a3002'', ''00000000-0000-7000-8000-0000000a3021'', ''01a2008d-7400-7000-8000-000000000b01'', ''01a2008d-7400-7000-8000-000000000b02'', ''granted'', ''corporate_portal'', ''00000000-0000-7000-8000-0000000a3c12'')',
  '23514', 'ck_consent_record_account_no_care_recipient');
SELECT pg_temp.uji_tolak('A3.3 NEG F1-T101 subjek akun dengan kanal patient_app', 'rs_api',
  'INSERT INTO app.consent_record (id, account_id, consent_text_version_id, consent_item_id, action, channel, correlation_id) VALUES (''00000000-0000-7000-8000-0000000a3312'', ''00000000-0000-7000-8000-0000000a3002'', ''01a2008d-7400-7000-8000-000000000b01'', ''01a2008d-7400-7000-8000-000000000b02'', ''granted'', ''patient_app'', ''00000000-0000-7000-8000-0000000a3c12'')',
  '23514', 'ck_consent_record_channel_subject');
SELECT pg_temp.uji_tolak('A3.3 NEG F1-T101 subjek pasien dengan kanal corporate_portal', 'rs_api',
  'INSERT INTO app.consent_record (id, patient_id, consent_text_version_id, consent_item_id, action, channel, correlation_id) VALUES (''00000000-0000-7000-8000-0000000a3312'', ''00000000-0000-7000-8000-0000000a3011'', ''00000000-0000-7000-8000-0000000a3102'', ''00000000-0000-7000-8000-0000000a3201'', ''granted'', ''corporate_portal'', ''00000000-0000-7000-8000-0000000a3c12'')',
  '23514', 'ck_consent_record_channel_subject');
SELECT pg_temp.uji_tolak('A3.3 NEG F1-T101 akun yang tidak ada', 'rs_api',
  'INSERT INTO app.consent_record (id, account_id, consent_text_version_id, consent_item_id, action, channel, correlation_id) VALUES (''00000000-0000-7000-8000-0000000a3312'', ''00000000-0000-7000-8000-00000000ffff'', ''01a2008d-7400-7000-8000-000000000b01'', ''01a2008d-7400-7000-8000-000000000b02'', ''granted'', ''corporate_portal'', ''00000000-0000-7000-8000-0000000a3c12'')',
  '23503', 'fk_consent_record_account_id');
SELECT pg_temp.uji_tolak('A3.3 NEG F1-T101 rs_api UPDATE bukti akun HR (kelas L)', 'rs_api',
  'UPDATE app.consent_record SET action = ''withdrawn'' WHERE account_id IS NOT NULL', '42501', 'permission denied for table consent_record');
SELECT pg_temp.uji_boleh('A3.3 F1-T101 pekerja privasi membaca account_id (FK subjek HR)', 'rs_privacy_worker_login',
  'SELECT id, account_id FROM app.consent_record WHERE account_id IS NOT NULL', 1);
SELECT pg_temp.uji_tolak('A3.3 NEG F1-T101 pekerja privasi membaca kolom channel', 'rs_privacy_worker_login',
  'SELECT channel FROM app.consent_record', '42501', 'permission denied for table consent_record');
SELECT pg_temp.uji_tolak('A3.1 NEG F1-T101 teks HR kedua dengan label yang sama (aktif)', 'rs_api',
  'INSERT INTO app.consent_text_version (id, consent_type, version_label, content, effective_from) VALUES (''00000000-0000-7000-8000-0000000a3107'', ''corporate_hr_registration'', ''0.1-draf'', ''x'', now() + interval ''8 days'')',
  '23505', 'uqp_consent_text_version_consent_type_version_label');
-- Waktu persetujuan dipaksa jam server (trigger tg_consent_record_server_time), untuk semua role.
SELECT pg_temp.uji_boleh('A3.3 rs_api mengirim created_at mundur 1 tahun dan maju 1 hari: baris tetap tersimpan', 'rs_api',
  'INSERT INTO app.consent_record (id, created_at, patient_id, consent_text_version_id, consent_item_id, action, channel, correlation_id) VALUES '
  || '(''00000000-0000-7000-8000-0000000a3306'', now() - interval ''1 year'', ''00000000-0000-7000-8000-0000000a3011'', ''00000000-0000-7000-8000-0000000a3102'', ''00000000-0000-7000-8000-0000000a3202'', ''granted'', ''patient_app'', ''00000000-0000-7000-8000-0000000a3c07''), '
  || '(''00000000-0000-7000-8000-0000000a3307'', now() + interval ''1 day'', ''00000000-0000-7000-8000-0000000a3011'', ''00000000-0000-7000-8000-0000000a3102'', ''00000000-0000-7000-8000-0000000a3202'', ''withdrawn'', ''patient_app'', ''00000000-0000-7000-8000-0000000a3c08'')', 2);
SELECT pg_temp.uji_boleh('A3.3 pemilik skema mengirim created_at mundur 10 tahun', 'rs_migration',
  'INSERT INTO app.consent_record (id, created_at, patient_id, consent_text_version_id, consent_item_id, action, channel, correlation_id) VALUES (''00000000-0000-7000-8000-0000000a3308'', now() - interval ''10 years'', ''00000000-0000-7000-8000-0000000a3011'', ''00000000-0000-7000-8000-0000000a3102'', ''00000000-0000-7000-8000-0000000a3202'', ''granted'', ''backoffice'', ''00000000-0000-7000-8000-0000000a3c09'')', 1);
DO $$
BEGIN
  IF EXISTS (SELECT 1 FROM app.consent_record
              WHERE id IN ('00000000-0000-7000-8000-0000000a3306', '00000000-0000-7000-8000-0000000a3307', '00000000-0000-7000-8000-0000000a3308')
                AND created_at IS DISTINCT FROM now())
     OR (SELECT count(*) FROM app.consent_record WHERE id IN ('00000000-0000-7000-8000-0000000a3306', '00000000-0000-7000-8000-0000000a3307', '00000000-0000-7000-8000-0000000a3308')) <> 3 THEN
    RAISE EXCEPTION 'GAGAL A3.3 created_at bukti persetujuan tidak dipaksa jam server';
  END IF;
  RAISE NOTICE 'LULUS A3.3 created_at bukti persetujuan = now() transaksi untuk rs_api (mundur/maju) dan pemilik skema (mundur), tidak dapat diisi mundur';
END $$;
SELECT pg_temp.uji_tolak('A3.3 NEG rotasi kunci membaca versi teks', 'rs_key_rotation', 'SELECT id FROM app.consent_text_version', '42501', 'permission denied for table consent_text_version');

-- Penjaga DELETE Log dengan fixture app.retention_policy (DD A6.6) bila tabel nyata belum ada; bila
-- sudah ada, berkas ini GAGAL supaya test diperbarui ke tabel nyata (pola db/test/kelas-app.sql).
DO $$
BEGIN
  IF to_regclass('app.retention_policy') IS NOT NULL THEN
    RAISE EXCEPTION 'GAGAL fixture: app.retention_policy sudah ada; perbarui db/test/tabel-app-a3.sql bagian penjaga DELETE consent_record';
  END IF;
END $$;
SET LOCAL ROLE rs_migration;
CREATE TABLE app.retention_policy (
  id uuid NOT NULL, data_category text NOT NULL, retention_months integer NOT NULL, end_action text NOT NULL,
  effective_from timestamptz NOT NULL, deleted_at timestamptz NULL,
  CONSTRAINT pk_retention_policy PRIMARY KEY (id));
GRANT SELECT ON app.retention_policy TO rs_privacy_worker;
-- created_at dipaksa now() oleh trigger jam server; penjaga menolak sebelum memeriksa umur baris.
INSERT INTO app.consent_record (id, created_at, patient_id, consent_text_version_id, consent_item_id, action, channel, correlation_id)
  VALUES ('00000000-0000-7000-8000-0000000a3305', now() - interval '10 years', '00000000-0000-7000-8000-0000000a3011',
          '00000000-0000-7000-8000-0000000a3102', '00000000-0000-7000-8000-0000000a3202', 'granted', 'patient_app', '00000000-0000-7000-8000-0000000a3c06');
RESET ROLE;
SELECT pg_temp.uji_tolak('A3.3 NEG pekerja privasi DELETE bukti selama kategori belum punya kebijakan (PT-24)', 'rs_privacy_worker_login',
  'DELETE FROM app.consent_record WHERE id = ''00000000-0000-7000-8000-0000000a3305''', '42501', '^retention_not_set: pending_pt_24 \(app.consent_record\)$');
INSERT INTO app.retention_policy VALUES ('00000000-0000-7000-8000-0000000a3401', 'pending_pt_24', 1, 'delete_row', now() - interval '1 day', NULL);
SELECT pg_temp.uji_tolak('A3.3 NEG pekerja privasi DELETE bukti walau ada kebijakan: daftar end_action trigger kosong sampai PT-24', 'rs_privacy_worker_login',
  'DELETE FROM app.consent_record WHERE id = ''00000000-0000-7000-8000-0000000a3305''', '42501', '^retention_action_not_allowed: pending_pt_24 delete_row');
ROLLBACK;

DO $$
BEGIN
  -- Seed versi draf migrasi 20261126200000 (F1-T49, pasien) dan 20261126310000 (F1-T101, HR; KEPUTUSAN-289)
  -- adalah data rujukan, bukan data uji.
  IF EXISTS (SELECT 1 FROM app.consent_record)
     OR EXISTS (SELECT 1 FROM app.consent_item WHERE id NOT IN ('01a2008d-7400-7000-8000-000000000a4a', '01a2008d-7400-7000-8000-000000000b02'))
     OR EXISTS (SELECT 1 FROM app.consent_text_version WHERE id NOT IN ('01a2008d-7400-7000-8000-000000000a49', '01a2008d-7400-7000-8000-000000000b01'))
     OR (SELECT count(*) FROM app.consent_text_version) <> 2 OR (SELECT count(*) FROM app.consent_item) <> 2
     OR EXISTS (SELECT 1 FROM app.patient)
     OR EXISTS (SELECT 1 FROM app.care_recipient) OR EXISTS (SELECT 1 FROM app.account)
     OR to_regclass('app.retention_policy') IS NOT NULL THEN
    RAISE EXCEPTION 'GAGAL pembersihan tabel-app-a3: data atau fixture uji tersisa';
  END IF;
  RAISE NOTICE 'LULUS pembersihan tabel-app-a3: tidak ada data uji atau fixture tersisa (ROLLBACK)';
END $$;
