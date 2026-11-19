-- db/test/tabel-app-a19.sql: test tabel A19 platform (F1-T17) di database ruang_sehat: audit_outbox,
-- job, upload_slot, stored_file, data_key, data_key_retention_anchor (CR-014). Didahului
-- db/test/lib.sql; dijalankan superuser setelah migrasi. Aturan kelas generik (T-06/T-07/T-08) di
-- db/test/kelas-app.sql bagian A; DDL umum §4.1 di db/test/tabel-app-a1.sql. Berkas ini memeriksa
-- struktur khusus (komentar kelas, index, parameter penyimpanan, fungsi dan trigger app.*) dan
-- perilaku dengan role sungguhan (positif dan negatif). Satu transaksi yang di-ROLLBACK. Data sintetis.
-- Dasar: docs/04-data-dictionary.md §0.6 A, §0.7, A19.1, A19.4-A19.7, A19.10; docs/05-physical-schema.md
-- §4.8, §4.10, §4.12, §9.1, §9.3, §12 (T-06, T-08, T-09); ADR-009, ADR-013 §5, KEPUTUSAN-66, -155.

-- 1. Struktur.
SELECT pg_temp.uji_nol('A19 komentar kelas: audit_outbox OH, job O, upload_slot O, stored_file M, data_key MK, data_key_retention_anchor T',
  ARRAY(SELECT format('%s: %s', t, coalesce(obj_description(to_regclass('app.' || t), 'pg_class'), '<tidak ada>'))
          FROM (VALUES ('audit_outbox', 'class:OH module:platform'), ('job', 'class:O module:platform'),
                       ('upload_slot', 'class:O module:platform'), ('stored_file', 'class:M module:platform'),
                       ('data_key', 'class:MK module:platform'),
                       ('data_key_retention_anchor', 'class:T module:platform alias:dk_ret_anchor')) v(t, k)
         WHERE obj_description(to_regclass('app.' || t), 'pg_class') IS DISTINCT FROM k));
SELECT pg_temp.uji_nol('A19 tanpa kolom anonymize (DD A19 tidak menandai "Anonimkan")',
  ARRAY(SELECT format('%s.%s', attrelid::regclass, attname) FROM pg_attribute
         WHERE attrelid IN ('app.audit_outbox'::regclass, 'app.job'::regclass, 'app.upload_slot'::regclass,
                            'app.stored_file'::regclass, 'app.data_key'::regclass, 'app.data_key_retention_anchor'::regclass)
           AND attnum > 0 AND col_description(attrelid, attnum) IS NOT NULL));
SELECT pg_temp.uji_nol('A19.1 audit_outbox: index persis pk, ix (created_at, id) §9.3',
  ARRAY(SELECT pg_temp.uji_indeks('app.audit_outbox', ARRAY[
    'CREATE UNIQUE INDEX pk_audit_outbox ON app.audit_outbox USING btree (id)',
    'CREATE INDEX ix_audit_outbox_created_at_id ON app.audit_outbox USING btree (created_at, id)'])));
SELECT pg_temp.uji_nol('A19.4 job: index persis pk, ix run_at queued, ix status/finished_at selesai, uqp dedupe_key aktif, ix locked_until running',
  ARRAY(SELECT pg_temp.uji_indeks('app.job', ARRAY[
    'CREATE UNIQUE INDEX pk_job ON app.job USING btree (id)',
    'CREATE INDEX ix_job_run_at_queued ON app.job USING btree (run_at) WHERE (status = ''queued''::text)',
    'CREATE INDEX ix_job_status_finished_at ON app.job USING btree (status, finished_at) WHERE (status = ANY (ARRAY[''succeeded''::text, ''failed''::text, ''cancelled''::text]))',
    'CREATE UNIQUE INDEX uqp_job_dedupe_key ON app.job USING btree (dedupe_key) WHERE (status = ANY (ARRAY[''queued''::text, ''running''::text]))',
    'CREATE INDEX ix_job_locked_until_running ON app.job USING btree (locked_until) WHERE (status = ''running''::text)'])));
SELECT pg_temp.uji_nol('A19.5 upload_slot: index persis pk, uq object_key, ix expires_at §9.3',
  ARRAY(SELECT pg_temp.uji_indeks('app.upload_slot', ARRAY[
    'CREATE UNIQUE INDEX pk_upload_slot ON app.upload_slot USING btree (id)',
    'CREATE UNIQUE INDEX uq_upload_slot_object_key ON app.upload_slot USING btree (object_key)',
    'CREATE INDEX ix_upload_slot_expires_at ON app.upload_slot USING btree (expires_at)'])));
SELECT pg_temp.uji_nol('A19.6 stored_file: index persis pk, uq object_key, ix (container_type, container_id)',
  ARRAY(SELECT pg_temp.uji_indeks('app.stored_file', ARRAY[
    'CREATE UNIQUE INDEX pk_stored_file ON app.stored_file USING btree (id)',
    'CREATE UNIQUE INDEX uq_stored_file_object_key ON app.stored_file USING btree (object_key)',
    'CREATE INDEX ix_stored_file_container_type_container_id ON app.stored_file USING btree (container_type, container_id)'])));
SELECT pg_temp.uji_nol('A19.7 data_key: index persis pk, uq (container_type, container_id), uq (id, container_id), ix retensi belum dimusnahkan',
  ARRAY(SELECT pg_temp.uji_indeks('app.data_key', ARRAY[
    'CREATE UNIQUE INDEX pk_data_key ON app.data_key USING btree (id)',
    'CREATE UNIQUE INDEX uq_data_key_container_type_container_id ON app.data_key USING btree (container_type, container_id)',
    'CREATE UNIQUE INDEX uq_data_key_id_container_id ON app.data_key USING btree (id, container_id)',
    'CREATE INDEX ix_data_key_retention_category_retention_anchor_at ON app.data_key USING btree (retention_category, retention_anchor_at) WHERE (shredded_at IS NULL)'])));
SELECT pg_temp.uji_nol('A19.10 data_key_retention_anchor: index persis pk data_key_id, ix (kategori, acuan)',
  ARRAY(SELECT pg_temp.uji_indeks('app.data_key_retention_anchor', ARRAY[
    'CREATE UNIQUE INDEX pk_data_key_retention_anchor ON app.data_key_retention_anchor USING btree (data_key_id)',
    'CREATE INDEX ix_data_key_retention_anchor_category_anchor ON app.data_key_retention_anchor USING btree (retention_category, retention_anchor_at)'])));
SELECT pg_temp.uji_nol('A19 parameter autovacuum tabel O sering berubah (job, audit_outbox; KEPUTUSAN-109, SRC-430)',
  ARRAY(SELECT format('%s reloptions %s', c.relname, c.reloptions) FROM pg_class c
         WHERE c.oid IN ('app.job'::regclass, 'app.audit_outbox'::regclass)
           AND c.reloptions IS DISTINCT FROM ARRAY['autovacuum_vacuum_scale_factor=0.02', 'autovacuum_analyze_scale_factor=0.02']));
SELECT pg_temp.uji_nol('A19 FK data_key_retention_anchor -> data_key dan -> account (§4.12)',
  ARRAY(SELECT x FROM (VALUES ('fk_data_key_retention_anchor_data_key_id', 'FOREIGN KEY (data_key_id) REFERENCES app.data_key(id)'),
                              ('fk_data_key_retention_anchor_created_by', 'FOREIGN KEY (created_by) REFERENCES app.account(id)')) v(n, def),
          LATERAL (SELECT format('%s: %s', v.n, coalesce(pg_get_constraintdef(k.oid), '<tidak ada>')) AS x
                     FROM (SELECT 1) d LEFT JOIN pg_constraint k ON k.conname = v.n AND k.conrelid = 'app.data_key_retention_anchor'::regclass) z
         WHERE x IS DISTINCT FROM format('%s: %s', v.n, v.def)));
-- Fungsi trigger app.* dan trigger tambahan (di luar util.*, sehingga tidak tercakup T-08 generik).
SELECT pg_temp.uji_nol('A19.10 fungsi app.fn_data_key_retention_anchor_single_source: plpgsql, SECURITY INVOKER, search_path persis, milik rs_migration, EXECUTE hanya pemilik',
  ARRAY(SELECT 'fungsi tidak sesuai' WHERE NOT EXISTS (
    SELECT 1 FROM pg_proc p
     WHERE p.oid = to_regprocedure('app.fn_data_key_retention_anchor_single_source()')
       AND p.prorettype = 'trigger'::regtype AND NOT p.prosecdef
       AND p.prolang = (SELECT oid FROM pg_language WHERE lanname = 'plpgsql')
       AND p.proconfig = ARRAY['search_path=pg_catalog, pg_temp']
       AND p.proowner = 'rs_migration'::regrole AND p.proacl IS NOT NULL
       AND NOT EXISTS (SELECT 1 FROM aclexplode(p.proacl) a WHERE a.grantee <> p.proowner))));
SELECT pg_temp.uji_nol('A19.10 trigger tg_data_key_retention_anchor_single_source: BEFORE INSERT FOR EACH ROW, aktif, tanpa argumen',
  ARRAY(SELECT 'trigger tidak sesuai' WHERE NOT EXISTS (
    SELECT 1 FROM pg_trigger g
     WHERE g.tgrelid = 'app.data_key_retention_anchor'::regclass AND g.tgname = 'tg_data_key_retention_anchor_single_source'
       AND g.tgfoid = to_regprocedure('app.fn_data_key_retention_anchor_single_source()')
       AND g.tgtype = (1 | 2 | 4) AND g.tgenabled = 'O' AND g.tgnargs = 0)));
SELECT pg_temp.uji_nol('A19 trigger non-util di skema app hanya yang terdaftar dokumen (§4.11)',
  ARRAY(SELECT format('%s.%s -> %s', g.tgrelid::regclass, g.tgname, g.tgfoid::regproc)
          FROM pg_trigger g JOIN pg_class c ON c.oid = g.tgrelid JOIN pg_namespace n ON n.oid = c.relnamespace
         WHERE n.nspname = 'app' AND NOT g.tgisinternal AND g.tgfoid::regproc::text NOT LIKE 'util.%'
           AND (c.relname, g.tgname) NOT IN (('data_key_retention_anchor', 'tg_data_key_retention_anchor_single_source'))));

-- 2. Negatif atas pemeriksa struktur, lalu perilaku. Semua di-ROLLBACK.
BEGIN;
DO $$
DECLARE
  c record;
BEGIN
  FOR c IN SELECT * FROM (VALUES
      ('trigger single_source dinonaktifkan', 'ALTER TABLE app.data_key_retention_anchor DISABLE TRIGGER tg_data_key_retention_anchor_single_source',
       $q$SELECT 'trigger tidak sesuai' WHERE NOT EXISTS (SELECT 1 FROM pg_trigger g WHERE g.tgname = 'tg_data_key_retention_anchor_single_source' AND g.tgenabled = 'O')$q$, 'trigger tidak sesuai'),
      ('trigger non-util tak terdaftar', 'CREATE TRIGGER tg_job_uji BEFORE INSERT ON app.job FOR EACH ROW EXECUTE FUNCTION app.fn_data_key_retention_anchor_single_source()',
       $q$SELECT format('%s.%s', g.tgrelid::regclass, g.tgname) FROM pg_trigger g JOIN pg_class c ON c.oid = g.tgrelid JOIN pg_namespace n ON n.oid = c.relnamespace WHERE n.nspname = 'app' AND NOT g.tgisinternal AND g.tgfoid::regproc::text NOT LIKE 'util.%' AND (c.relname, g.tgname) NOT IN (('data_key_retention_anchor', 'tg_data_key_retention_anchor_single_source'))$q$, 'app.job.tg_job_uji'),
      ('T-06 SELECT audit_outbox untuk rs_app', 'GRANT SELECT ON app.audit_outbox TO rs_app', $q$SELECT pg_temp.uji_kelas_app('app', '{}')$q$, 'T-06 audit_outbox .*lebih rs_app:SELECT'),
      ('T-06 UPDATE data_key_retention_anchor untuk rs_app', 'GRANT UPDATE ON app.data_key_retention_anchor TO rs_app', $q$SELECT pg_temp.uji_kelas_app('app', '{}')$q$, 'T-06 data_key_retention_anchor .*lebih rs_app:UPDATE'),
      ('T-06 SELECT kolom container_id data_key untuk rotasi kunci', 'GRANT SELECT (container_id) ON app.data_key TO rs_key_rotation', $q$SELECT pg_temp.uji_kelas_app('app', '{}')$q$, 'T-06 data_key: SELECT per kolom rs_key_rotation'),
      ('T-08 trigger one_way data_key hilang', 'DROP TRIGGER tg_data_key_one_way ON app.data_key', $q$SELECT pg_temp.uji_kelas_app('app', '{}')$q$, 'T-08 data_key \(kelas MK\)'),
      ('T-08 trigger reject_ud acuan retensi hilang', 'DROP TRIGGER tg_data_key_retention_anchor_reject_ud ON app.data_key_retention_anchor', $q$SELECT pg_temp.uji_kelas_app('app', '{}')$q$, 'T-08 data_key_retention_anchor \(kelas T\)'),
      ('autovacuum job dilepas', 'ALTER TABLE app.job RESET (autovacuum_vacuum_scale_factor)',
       $q$SELECT c.relname::text FROM pg_class c WHERE c.oid = 'app.job'::regclass AND c.reloptions IS DISTINCT FROM ARRAY['autovacuum_vacuum_scale_factor=0.02', 'autovacuum_analyze_scale_factor=0.02']$q$, 'job')
    ) AS t(label, stmt, pemeriksa, pola)
  LOOP
    PERFORM pg_temp.uji_rusak(c.label, c.stmt, c.pemeriksa, c.pola);
  END LOOP;
END $$;

SET LOCAL ROLE rs_migration;
INSERT INTO app.account (id, account_type, idp_realm, idp_subject, email, status, status_changed_at)
  VALUES ('00000000-0000-7000-8000-00000000a101', 'patient', 'pasien', 'sub-a19', 'a19@contoh.invalid', 'active', now());
RESET ROLE;

-- audit_outbox (OH): rs_app/pekerja privasi INSERT saja; relay SELECT, DELETE (ADR-009 butir 2-3).
SELECT pg_temp.uji_boleh('A19.1 rs_api INSERT audit_outbox (tanpa RETURNING)', 'rs_api',
  'INSERT INTO app.audit_outbox (id, occurred_at, payload) VALUES (''00000000-0000-7000-8000-00000000d001'', now(), ''{"event":"E-01"}'')', 1);
SELECT pg_temp.uji_boleh('A19.1 pekerja privasi INSERT audit_outbox (E-16)', 'rs_privacy_worker_login',
  'INSERT INTO app.audit_outbox (id, occurred_at, payload) VALUES (''00000000-0000-7000-8000-00000000d002'', now(), ''{"event":"E-16"}'')', 1);
SELECT pg_temp.uji_tolak('A19.1 NEG rs_api SELECT audit_outbox', 'rs_api', 'SELECT * FROM app.audit_outbox', '42501', 'permission denied for table audit_outbox');
SELECT pg_temp.uji_tolak('A19.1 NEG rs_api INSERT ... RETURNING audit_outbox (butuh SELECT)', 'rs_api',
  'INSERT INTO app.audit_outbox (id, occurred_at, payload) VALUES (''00000000-0000-7000-8000-00000000d003'', now(), ''{}'') RETURNING id', '42501', 'permission denied for table audit_outbox');
SELECT pg_temp.uji_tolak('A19.1 NEG rs_api UPDATE audit_outbox', 'rs_api', 'UPDATE app.audit_outbox SET payload = ''{}''', '42501', 'permission denied for table audit_outbox');
SELECT pg_temp.uji_tolak('A19.1 NEG rs_worker DELETE audit_outbox (hanya relay)', 'rs_worker', 'DELETE FROM app.audit_outbox', '42501', 'permission denied for table audit_outbox');
SELECT pg_temp.uji_tolak('A19.1 NEG pekerja privasi SELECT audit_outbox', 'rs_privacy_worker_login', 'SELECT * FROM app.audit_outbox', '42501', 'permission denied for table audit_outbox');
SELECT pg_temp.uji_tolak('A19.1 NEG relay INSERT audit_outbox', 'rs_audit_relay_login',
  'INSERT INTO app.audit_outbox (id, occurred_at, payload) VALUES (''00000000-0000-7000-8000-00000000d004'', now(), ''{}'')', '42501', 'permission denied for table audit_outbox');
SELECT pg_temp.uji_tolak('A19.1 NEG relay UPDATE audit_outbox (tidak dapat mengubah event)', 'rs_audit_relay_login',
  'UPDATE app.audit_outbox SET payload = ''{}''', '42501', 'permission denied for table audit_outbox');
SELECT pg_temp.uji_boleh('A19.1 relay membaca urut (created_at, id) lalu menghapus yang terkirim', 'rs_audit_relay_login',
  'DELETE FROM app.audit_outbox WHERE id IN (SELECT id FROM app.audit_outbox ORDER BY created_at, id LIMIT 10)', 2);
SELECT pg_temp.uji_tolak('A19.1 NEG relay membaca job (di luar haknya)', 'rs_audit_relay_login', 'SELECT * FROM app.job', '42501', 'permission denied for table job');

-- job (O): pengambilan dengan sewa (§9.1).
SELECT pg_temp.uji_boleh('A19.4 rs_api menjadwalkan job', 'rs_api',
  'INSERT INTO app.job (id, job_type, payload, run_at, status, attempt_count, max_attempts, dedupe_key) VALUES (''00000000-0000-7000-8000-00000000e001'', ''uji'', ''{"id":"x"}'', now() - interval ''1 minute'', ''queued'', 0, 3, ''uji:x'')', 1);
SELECT pg_temp.uji_tolak('A19.4 NEG dedupe_key ganda untuk job queued/running', 'rs_api',
  'INSERT INTO app.job (id, job_type, payload, run_at, status, attempt_count, max_attempts, dedupe_key) VALUES (''00000000-0000-7000-8000-00000000e002'', ''uji'', ''{}'', now(), ''queued'', 0, 3, ''uji:x'')',
  '23505', 'uqp_job_dedupe_key');
SELECT pg_temp.uji_boleh('A19.4 rs_worker mengambil job dengan sewa (FOR UPDATE SKIP LOCKED, §9.1)', 'rs_worker',
  'WITH next AS (SELECT id FROM app.job WHERE status = ''queued'' AND run_at <= now() ORDER BY run_at LIMIT 5 FOR UPDATE SKIP LOCKED) UPDATE app.job j SET status = ''running'', locked_by = ''w1'', locked_until = now() + interval ''5 minutes'', attempt_count = j.attempt_count + 1 FROM next WHERE j.id = next.id', 1);
SELECT pg_temp.uji_boleh('A19.4 penyelesaian dipagari sewa', 'rs_worker',
  'UPDATE app.job SET status = ''succeeded'', finished_at = now() WHERE id = ''00000000-0000-7000-8000-00000000e001'' AND status = ''running'' AND locked_by = ''w1''', 1);
SELECT pg_temp.uji_boleh('A19.4 dedupe_key yang sama boleh lagi setelah job selesai', 'rs_api',
  'INSERT INTO app.job (id, job_type, payload, run_at, status, attempt_count, max_attempts, dedupe_key) VALUES (''00000000-0000-7000-8000-00000000e003'', ''uji'', ''{}'', now(), ''queued'', 0, 3, ''uji:x'')', 1);
SELECT pg_temp.uji_tolak('A19.4 NEG status di luar daftar', 'rs_worker',
  'UPDATE app.job SET status = ''done'' WHERE id = ''00000000-0000-7000-8000-00000000e003''', '23514', 'ck_job_status');
SELECT pg_temp.uji_boleh('A19.4 rs_worker menghapus job selesai (pembersihan)', 'rs_worker',
  'DELETE FROM app.job WHERE status IN (''succeeded'', ''failed'', ''cancelled'')', 1);
SELECT pg_temp.uji_tolak('A19.4 NEG pekerja privasi membaca job', 'rs_privacy_worker_login', 'SELECT * FROM app.job', '42501', 'permission denied for table job');
SELECT pg_temp.uji_tolak('A19.4 NEG rs_api TRUNCATE job', 'rs_api', 'TRUNCATE app.job', '42501', 'permission denied for table job');

-- upload_slot (O).
SELECT pg_temp.uji_boleh('A19.5 rs_api membuat slot unggah', 'rs_api',
  'INSERT INTO app.upload_slot (id, expires_at, owner_account_id, bucket, object_key, allowed_mime_type, max_size_bytes) VALUES (''00000000-0000-7000-8000-00000000f001'', now() + interval ''15 minutes'', ''00000000-0000-7000-8000-00000000a101'', ''identity_document'', ''00000000-0000-7000-8000-0000000000f1'', ''image/jpeg'', 5242880)', 1);
SELECT pg_temp.uji_tolak('A19.5 NEG object_key ganda', 'rs_api',
  'INSERT INTO app.upload_slot (id, expires_at, bucket, object_key, allowed_mime_type, max_size_bytes) VALUES (''00000000-0000-7000-8000-00000000f002'', now(), ''identity_document'', ''00000000-0000-7000-8000-0000000000f1'', ''image/png'', 1)',
  '23505', 'uq_upload_slot_object_key');
SELECT pg_temp.uji_tolak('A19.5 NEG slot tanpa expires_at', 'rs_api',
  'INSERT INTO app.upload_slot (id, bucket, object_key, allowed_mime_type, max_size_bytes) VALUES (''00000000-0000-7000-8000-00000000f002'', ''identity_document'', ''k2'', ''image/png'', 1)',
  '23502', 'expires_at');
SELECT pg_temp.uji_tolak('A19.5 NEG pemilik akun yang tidak ada', 'rs_api',
  'INSERT INTO app.upload_slot (id, expires_at, owner_account_id, bucket, object_key, allowed_mime_type, max_size_bytes) VALUES (''00000000-0000-7000-8000-00000000f002'', now(), ''00000000-0000-7000-8000-00000000ffff'', ''identity_document'', ''k2'', ''image/png'', 1)',
  '23503', 'fk_upload_slot_owner_account_id');
SELECT pg_temp.uji_boleh('A19.5 rs_worker menghapus slot kedaluwarsa', 'rs_worker',
  'DELETE FROM app.upload_slot WHERE expires_at < now() + interval ''1 hour''', 1);
SELECT pg_temp.uji_tolak('A19.5 NEG pekerja privasi membaca upload_slot', 'rs_privacy_worker_login', 'SELECT * FROM app.upload_slot', '42501', 'permission denied for table upload_slot');

-- stored_file (M).
SELECT pg_temp.uji_boleh('A19.6 rs_api mencatat berkas tervalidasi', 'rs_api',
  'INSERT INTO app.stored_file (id, bucket, object_key, category, owner_account_id, container_type, container_id, mime_type, size_bytes, sha256, validated_at) VALUES (''00000000-0000-7000-8000-000000001001'', ''identity_document'', ''obj-1'', ''ktp_photo'', ''00000000-0000-7000-8000-00000000a101'', ''identity_document'', ''00000000-0000-7000-8000-000000002001'', ''image/jpeg'', 1024, ''\x00'', now())', 1);
SELECT pg_temp.uji_tolak('A19.6 NEG bucket di luar daftar', 'rs_api',
  'INSERT INTO app.stored_file (id, bucket, object_key, category, mime_type, size_bytes, sha256, validated_at) VALUES (''00000000-0000-7000-8000-000000001002'', ''publik'', ''obj-2'', ''x'', ''image/jpeg'', 1, ''\x00'', now())',
  '23514', 'ck_stored_file_bucket');
SELECT pg_temp.uji_tolak('A19.6 NEG mime_type di luar daftar', 'rs_api',
  'INSERT INTO app.stored_file (id, bucket, object_key, category, mime_type, size_bytes, sha256, validated_at) VALUES (''00000000-0000-7000-8000-000000001002'', ''export'', ''obj-2'', ''x'', ''image/gif'', 1, ''\x00'', now())',
  '23514', 'ck_stored_file_mime_type');
SELECT pg_temp.uji_tolak('A19.6 NEG size_bytes nol', 'rs_api',
  'INSERT INTO app.stored_file (id, bucket, object_key, category, mime_type, size_bytes, sha256, validated_at) VALUES (''00000000-0000-7000-8000-000000001002'', ''export'', ''obj-2'', ''x'', ''text/csv'', 0, ''\x00'', now())',
  '23514', 'ck_stored_file_size_bytes');
SELECT pg_temp.uji_tolak('A19.6 NEG object_key ganda', 'rs_api',
  'INSERT INTO app.stored_file (id, bucket, object_key, category, mime_type, size_bytes, sha256, validated_at) VALUES (''00000000-0000-7000-8000-000000001002'', ''export'', ''obj-1'', ''x'', ''text/csv'', 1, ''\x00'', now())',
  '23505', 'uq_stored_file_object_key');
SELECT pg_temp.uji_tolak('A19.6 NEG soft delete tanpa alasan', 'rs_api',
  'UPDATE app.stored_file SET deleted_at = now() WHERE id = ''00000000-0000-7000-8000-000000001001''', '23514', 'ck_stored_file_soft_delete');
SELECT pg_temp.uji_tolak('A19.6 NEG rs_api DELETE stored_file (Master)', 'rs_api', 'DELETE FROM app.stored_file', '42501', 'permission denied for table stored_file');
SELECT pg_temp.uji_boleh('A19.6 pekerja privasi membaca stored_file', 'rs_privacy_worker_login', 'SELECT id FROM app.stored_file', 1);
SELECT pg_temp.uji_tolak('A19.6 NEG pekerja privasi UPDATE stored_file (tanpa kolom anonymize)', 'rs_privacy_worker_login',
  'UPDATE app.stored_file SET purged_at = now()', '42501', 'permission denied for table stored_file');

-- data_key (MK, §4.10).
SELECT pg_temp.uji_boleh('A19.7 rs_api membuat DEK untuk sepuluh wadah DD §0.7', 'rs_api',
  'INSERT INTO app.data_key (id, container_type, container_id, wrapped_dek, kek_ref, retention_category, retention_anchor_at) SELECT (''00000000-0000-7000-8000-0000000030'' || lpad(n::text, 2, ''0''))::uuid, t, (''00000000-0000-7000-8000-0000000040'' || lpad(n::text, 2, ''0''))::uuid, ''\x0102''::bytea, ''openbao-transit:transit:rs-dek-kek:v1'', CASE WHEN n = 2 THEN ''kategori_uji'' END, CASE WHEN n = 3 THEN now() END FROM unnest(ARRAY[''consultation_chat'', ''consultation_clinical'', ''lab_result'', ''identity_document'', ''claim'', ''integration_client'', ''partner_bank_account'', ''complaint_ticket'', ''data_subject_request'', ''status_note'']) WITH ORDINALITY AS u(t, n)', 10);
SELECT pg_temp.uji_tolak('A19.7 NEG container_type wadah RME di DB aplikasi', 'rs_api',
  'INSERT INTO app.data_key (id, container_type, container_id, wrapped_dek, kek_ref) VALUES (''00000000-0000-7000-8000-000000003099'', ''rme_visit'', ''00000000-0000-7000-8000-000000004099'', ''\x01'', ''k'')',
  '23514', 'ck_data_key_container_type');
SELECT pg_temp.uji_tolak('A19.7 NEG dua DEK untuk wadah yang sama', 'rs_api',
  'INSERT INTO app.data_key (id, container_type, container_id, wrapped_dek, kek_ref) VALUES (''00000000-0000-7000-8000-000000003099'', ''claim'', ''00000000-0000-7000-8000-000000004005'', ''\x01'', ''k'')',
  '23505', 'uq_data_key_container_type_container_id');
SELECT pg_temp.uji_tolak('A19.7 NEG DEK baru tanpa wrapped_dek (bukan pemusnahan)', 'rs_api',
  'INSERT INTO app.data_key (id, container_type, container_id, kek_ref) VALUES (''00000000-0000-7000-8000-000000003099'', ''claim'', ''00000000-0000-7000-8000-000000004099'', ''k'')',
  '23514', 'ck_data_key_shredded');
SELECT pg_temp.uji_tolak('A19.7 NEG rs_api UPDATE data_key', 'rs_api', 'UPDATE app.data_key SET kek_ref = ''x''', '42501', 'permission denied for table data_key');
SELECT pg_temp.uji_tolak('A19.7 NEG rs_api DELETE data_key', 'rs_api', 'DELETE FROM app.data_key', '42501', 'permission denied for table data_key');
SELECT pg_temp.uji_tolak('A19.7 NEG pemilik UPDATE data_key ditolak trigger satu arah', 'rs_migration',
  'UPDATE app.data_key SET retention_anchor_at = now() WHERE id = ''00000000-0000-7000-8000-000000003001''', '42501', '^immutable_violation: data_key id=.* column not updatable$');
SELECT pg_temp.uji_tolak('A19.7 NEG pemilik DELETE data_key ditolak trigger', 'rs_migration', 'DELETE FROM app.data_key', '42501', '^immutable_violation: DELETE on app.data_key rejected$');
SELECT pg_temp.uji_tolak('A19.7 NEG pemilik TRUNCATE data_key ditolak trigger', 'rs_migration', 'TRUNCATE app.data_key CASCADE', '42501', '^immutable_violation: TRUNCATE on app\.data_key(_retention_anchor)? rejected$');
SELECT pg_temp.uji_tolak('A19.7 NEG rotasi kunci membaca container_id (di luar kolomnya)', 'rs_key_rotation', 'SELECT container_id FROM app.data_key', '42501', 'permission denied for table data_key');
SELECT pg_temp.uji_boleh('A19.7 rotasi kunci membungkus ulang DEK', 'rs_key_rotation',
  'UPDATE app.data_key SET wrapped_dek = ''\x0506'', kek_ref = ''openbao-transit:transit:rs-dek-kek:v2'' WHERE id = ''00000000-0000-7000-8000-000000003001''', 1);
SELECT pg_temp.uji_tolak('A19.7 NEG pekerja privasi memusnahkan dengan shred_reason di luar daftar', 'rs_privacy_worker_login',
  'UPDATE app.data_key SET wrapped_dek = NULL, shred_reason = ''bosan'' WHERE id = ''00000000-0000-7000-8000-000000003004''', '23514', 'ck_data_key_shred_reason');
SELECT pg_temp.uji_tolak('A19.7 NEG pekerja privasi memusnahkan tanpa shred_reason', 'rs_privacy_worker_login',
  'UPDATE app.data_key SET wrapped_dek = NULL WHERE id = ''00000000-0000-7000-8000-000000003004''', '42501', '^immutable_violation: invalid shred');
SELECT pg_temp.uji_boleh('A19.7 pekerja privasi memusnahkan DEK (shredded_at dari trigger)', 'rs_privacy_worker_login',
  'UPDATE app.data_key SET wrapped_dek = NULL, shred_reason = ''erasure_request'' WHERE id = ''00000000-0000-7000-8000-000000003004''', 1);
SELECT pg_temp.uji_tolak('A19.7 NEG DEK yang dimusnahkan tidak dapat dihidupkan kembali', 'rs_key_rotation',
  'UPDATE app.data_key SET wrapped_dek = ''\x07'' WHERE id = ''00000000-0000-7000-8000-000000003004''', '42501', '^data_key_shredded: id=00000000-0000-7000-8000-000000003004 immutable$');

-- data_key_retention_anchor (T, CR-014): sekali-isi, satu sumber acuan waktu, kategori sama.
SELECT pg_temp.uji_boleh('A19.10 rs_api mengisi acuan retensi DEK yang belum punya acuan', 'rs_api',
  'INSERT INTO app.data_key_retention_anchor (data_key_id, retention_category, retention_anchor_at, created_by) VALUES (''00000000-0000-7000-8000-000000003001'', ''kategori_a'', now(), ''00000000-0000-7000-8000-00000000a101'')', 1);
SELECT pg_temp.uji_tolak('A19.10 NEG acuan kedua untuk DEK yang sama (PK)', 'rs_api',
  'INSERT INTO app.data_key_retention_anchor (data_key_id, retention_category, retention_anchor_at) VALUES (''00000000-0000-7000-8000-000000003001'', ''kategori_a'', now())',
  '23505', 'pk_data_key_retention_anchor');
SELECT pg_temp.uji_boleh('A19.10 acuan kedua ON CONFLICT DO NOTHING tidak menulis apa pun (idempotensi)', 'rs_api',
  'INSERT INTO app.data_key_retention_anchor (data_key_id, retention_category, retention_anchor_at) VALUES (''00000000-0000-7000-8000-000000003001'', ''kategori_a'', now()) ON CONFLICT (data_key_id) DO NOTHING', 0);
SELECT pg_temp.uji_tolak('A19.10 NEG acuan untuk DEK yang acuan waktunya sudah ada di data_key', 'rs_api',
  'INSERT INTO app.data_key_retention_anchor (data_key_id, retention_category, retention_anchor_at) VALUES (''00000000-0000-7000-8000-000000003003'', ''kategori_b'', now())',
  '42501', '^retention_anchor_already_set: data_key id=00000000-0000-7000-8000-000000003003$');
SELECT pg_temp.uji_tolak('A19.10 NEG kategori berbeda dari data_key.retention_category', 'rs_api',
  'INSERT INTO app.data_key_retention_anchor (data_key_id, retention_category, retention_anchor_at) VALUES (''00000000-0000-7000-8000-000000003002'', ''kategori_lain'', now())',
  '42501', '^retention_category_mismatch: data_key id=00000000-0000-7000-8000-000000003002$');
SELECT pg_temp.uji_boleh('A19.10 kategori sama dengan data_key.retention_category diterima', 'rs_api',
  'INSERT INTO app.data_key_retention_anchor (data_key_id, retention_category, retention_anchor_at) VALUES (''00000000-0000-7000-8000-000000003002'', ''kategori_uji'', now())', 1);
SELECT pg_temp.uji_tolak('A19.10 NEG DEK yang tidak ada (FK)', 'rs_api',
  'INSERT INTO app.data_key_retention_anchor (data_key_id, retention_category, retention_anchor_at) VALUES (''00000000-0000-7000-8000-00000000ffff'', ''kategori_a'', now())',
  '23503', 'fk_data_key_retention_anchor_data_key_id');
SELECT pg_temp.uji_tolak('A19.10 NEG rs_api UPDATE acuan', 'rs_api', 'UPDATE app.data_key_retention_anchor SET retention_anchor_at = now()', '42501', 'permission denied for table data_key_retention_anchor');
SELECT pg_temp.uji_tolak('A19.10 NEG rs_api DELETE acuan', 'rs_api', 'DELETE FROM app.data_key_retention_anchor', '42501', 'permission denied for table data_key_retention_anchor');
SELECT pg_temp.uji_tolak('A19.10 NEG pemilik UPDATE acuan ditolak trigger', 'rs_migration',
  'UPDATE app.data_key_retention_anchor SET retention_anchor_at = now()', '42501', '^immutable_violation: UPDATE on app.data_key_retention_anchor rejected$');
SELECT pg_temp.uji_tolak('A19.10 NEG pemilik DELETE acuan ditolak trigger', 'rs_migration',
  'DELETE FROM app.data_key_retention_anchor', '42501', '^immutable_violation: DELETE on app.data_key_retention_anchor rejected$');
SELECT pg_temp.uji_tolak('A19.10 NEG pemilik TRUNCATE acuan ditolak trigger', 'rs_migration',
  'TRUNCATE app.data_key_retention_anchor', '42501', '^immutable_violation: TRUNCATE on app.data_key_retention_anchor rejected$');
SELECT pg_temp.uji_boleh('A19.10 pekerja privasi membaca acuan berlaku (coalesce data_key, acuan)', 'rs_privacy_worker_login',
  'SELECT dk.id FROM app.data_key dk LEFT JOIN app.data_key_retention_anchor a ON a.data_key_id = dk.id WHERE coalesce(dk.retention_anchor_at, a.retention_anchor_at) IS NOT NULL', 3);
SELECT pg_temp.uji_tolak('A19.10 NEG pekerja privasi INSERT acuan', 'rs_privacy_worker_login',
  'INSERT INTO app.data_key_retention_anchor (data_key_id, retention_category, retention_anchor_at) VALUES (''00000000-0000-7000-8000-000000003005'', ''k'', now())',
  '42501', 'permission denied for table data_key_retention_anchor');
SELECT pg_temp.uji_tolak('A19.10 NEG rotasi kunci membaca acuan', 'rs_key_rotation', 'SELECT * FROM app.data_key_retention_anchor', '42501', 'permission denied for table data_key_retention_anchor');
ROLLBACK;

DO $$
BEGIN
  IF EXISTS (SELECT 1 FROM app.audit_outbox) OR EXISTS (SELECT 1 FROM app.job) OR EXISTS (SELECT 1 FROM app.upload_slot)
     OR EXISTS (SELECT 1 FROM app.stored_file) OR EXISTS (SELECT 1 FROM app.data_key)
     OR EXISTS (SELECT 1 FROM app.data_key_retention_anchor) OR EXISTS (SELECT 1 FROM app.account) THEN
    RAISE EXCEPTION 'GAGAL pembersihan tabel-app-a19: data uji tersisa';
  END IF;
  RAISE NOTICE 'LULUS pembersihan tabel-app-a19: tidak ada data uji tersisa (ROLLBACK)';
END $$;
