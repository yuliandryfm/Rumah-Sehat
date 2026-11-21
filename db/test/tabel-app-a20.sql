-- db/test/tabel-app-a20.sql: test tabel A20 layanan korporat K1 (F1-T98) di database ruang_sehat:
-- corporate_client (M), corporate_client_document (M), corporate_verification_decision (T).
-- Didahului db/test/lib.sql; dijalankan superuser setelah migrasi. Aturan kelas generik (T-06/T-07/T-08) di
-- db/test/kelas-app.sql bagian A, T-01/T-20 generik di db/test/katalog.sql. Berkas ini memeriksa struktur khusus
-- (komentar kelas, kolom, hak tabel persis, index, FK, CHECK, trigger) dan perilaku dengan role sungguhan
-- (positif dan negatif). Satu transaksi yang di-ROLLBACK. Data sintetis.
-- Dasar: docs/04-data-dictionary.md v1.2 §0.4, §0.6 A, A20; docs/05-physical-schema.md §1, §4.1, §4.3, §4.5,
-- §4.11, §12 (T-06, T-08, T-09); ADR-003, ADR-004, ADR-008, ADR-028; PRD v1.3 US-1901 AC7, US-1902 AC3-AC4.

-- 1. Struktur.
SELECT pg_temp.uji_nol('A20 komentar kelas: corporate_client M, corporate_client_document M (alias), corporate_verification_decision T (alias), modul korporat',
  ARRAY(SELECT format('%s: %s', t, coalesce(obj_description(to_regclass('app.' || t), 'pg_class'), '<tidak ada>'))
          FROM (VALUES ('corporate_client', 'class:M module:korporat'),
                       ('corporate_client_document', 'class:M module:korporat alias:corp_client_doc'),
                       ('corporate_verification_decision', 'class:T module:korporat alias:corp_verif_decision')) v(t, k)
         WHERE obj_description(to_regclass('app.' || t), 'pg_class') IS DISTINCT FROM k));
SELECT pg_temp.uji_nol('A20 tanpa kolom anonymize (DD A20 tanpa kolom "Anonimkan") dan tanpa kolom *_enc/*_bidx',
  ARRAY(SELECT format('%s.%s', attrelid::regclass, attname) FROM pg_attribute
         WHERE attrelid IN ('app.corporate_client'::regclass, 'app.corporate_client_document'::regclass,
                            'app.corporate_verification_decision'::regclass)
           AND attnum > 0 AND NOT attisdropped
           AND (col_description(attrelid, attnum) IS NOT NULL OR attname ~ '(_enc|_bidx|_bidx_key_version)$')));
SELECT pg_temp.uji_nol('A20 kolom persis per tabel (STD-M/STD-T + kolom A20)',
  ARRAY(SELECT format('%s: %s', v.t, coalesce(s.x, '<kosong>'))
          FROM (VALUES
            ('corporate_client',
             'id:uuid:nn,created_at:timestamp with time zone:nn,created_by:uuid:n,updated_at:timestamp with time zone:nn,'
             || 'updated_by:uuid:n,deleted_at:timestamp with time zone:n,deleted_by:uuid:n,delete_reason:text:n,'
             || 'legal_name:text:nn,nib:text:nn,npwp:text:nn,office_address:text:nn,office_phone:text:nn,status:text:nn,'
             || 'origin:text:nn,submission_round:integer:nn,submitted_at:timestamp with time zone:nn'),
            ('corporate_client_document',
             'id:uuid:nn,created_at:timestamp with time zone:nn,created_by:uuid:n,updated_at:timestamp with time zone:nn,'
             || 'updated_by:uuid:n,deleted_at:timestamp with time zone:n,deleted_by:uuid:n,delete_reason:text:n,'
             || 'corporate_client_id:uuid:nn,document_kind:text:nn,stored_file_id:uuid:nn'),
            ('corporate_verification_decision',
             'id:uuid:nn,created_at:timestamp with time zone:nn,created_by:uuid:n,corporate_client_id:uuid:nn,'
             || 'submission_round:integer:nn,decision:text:nn,reason_code:text:n,decided_by:uuid:nn')) v(t, harap)
          LEFT JOIN LATERAL (
            SELECT string_agg(attname || ':' || format_type(atttypid, atttypmod) || ':' || CASE WHEN attnotnull THEN 'nn' ELSE 'n' END,
                              ',' ORDER BY attnum) AS x
              FROM pg_attribute WHERE attrelid = to_regclass('app.' || v.t) AND attnum > 0 AND NOT attisdropped) s ON true
         WHERE s.x IS DISTINCT FROM v.harap));
SELECT pg_temp.uji_nol('A20 hak tabel persis (M: rs_app S,I,U + pekerja privasi S; T: rs_app S,I); tanpa DELETE, relay, rotasi kunci, PUBLIC; tanpa hak per kolom',
  ARRAY(SELECT format('%s: %s', v.t, coalesce(a.k, '<kosong>'))
          FROM (VALUES
            ('corporate_client', 'rs_app:INSERT,rs_app:SELECT,rs_app:UPDATE,rs_privacy_worker:SELECT'),
            ('corporate_client_document', 'rs_app:INSERT,rs_app:SELECT,rs_app:UPDATE,rs_privacy_worker:SELECT'),
            ('corporate_verification_decision', 'rs_app:INSERT,rs_app:SELECT')) v(t, harap)
          LEFT JOIN LATERAL (
            SELECT string_agg(CASE WHEN x.grantee = 0 THEN 'PUBLIC' ELSE pg_get_userbyid(x.grantee) END || ':' || x.privilege_type,
                              ',' ORDER BY 1) AS k
              FROM pg_class c, aclexplode(c.relacl) x
             WHERE c.oid = to_regclass('app.' || v.t) AND x.grantee <> c.relowner) a ON true
         WHERE a.k IS DISTINCT FROM v.harap
        UNION ALL
        SELECT format('hak per kolom %s.%s', att.attrelid::regclass, att.attname) FROM pg_attribute att
         WHERE att.attrelid IN ('app.corporate_client'::regclass, 'app.corporate_client_document'::regclass,
                                'app.corporate_verification_decision'::regclass)
           AND att.attnum > 0 AND att.attacl IS NOT NULL));
SELECT pg_temp.uji_nol('A20 corporate_client: index persis pk, uqp nib aktif (US-1901 AC7), ix antrean pending',
  ARRAY(SELECT pg_temp.uji_indeks('app.corporate_client', ARRAY[
    'CREATE UNIQUE INDEX pk_corporate_client ON app.corporate_client USING btree (id)',
    'CREATE UNIQUE INDEX uqp_corporate_client_nib ON app.corporate_client USING btree (nib) WHERE (deleted_at IS NULL)',
    'CREATE INDEX ix_corporate_client_submitted_at_pending ON app.corporate_client USING btree (submitted_at) WHERE ((deleted_at IS NULL) AND (status = ''pending_verification''::text))'])));
SELECT pg_temp.uji_nol('A20 corporate_client_document: index persis pk, uq stored_file_id, uqp satu dokumen aktif per jenis',
  ARRAY(SELECT pg_temp.uji_indeks('app.corporate_client_document', ARRAY[
    'CREATE UNIQUE INDEX pk_corporate_client_document ON app.corporate_client_document USING btree (id)',
    'CREATE UNIQUE INDEX uq_corporate_client_document_stored_file_id ON app.corporate_client_document USING btree (stored_file_id)',
    'CREATE UNIQUE INDEX uqp_corp_client_doc_corporate_client_id_document_kind ON app.corporate_client_document USING btree (corporate_client_id, document_kind) WHERE (deleted_at IS NULL)'])));
SELECT pg_temp.uji_nol('A20 corporate_verification_decision: index persis pk, uq (perusahaan, pengajuan)',
  ARRAY(SELECT pg_temp.uji_indeks('app.corporate_verification_decision', ARRAY[
    'CREATE UNIQUE INDEX pk_corporate_verification_decision ON app.corporate_verification_decision USING btree (id)',
    'CREATE UNIQUE INDEX uq_corp_verif_decision_corporate_client_id_submission_round ON app.corporate_verification_decision USING btree (corporate_client_id, submission_round)'])));
SELECT pg_temp.uji_nol('A20 FK persis, tanpa CASCADE',
  ARRAY(SELECT x FROM (VALUES
      ('corporate_client', 'fk_corporate_client_created_by', 'FOREIGN KEY (created_by) REFERENCES app.account(id)'),
      ('corporate_client', 'fk_corporate_client_updated_by', 'FOREIGN KEY (updated_by) REFERENCES app.account(id)'),
      ('corporate_client', 'fk_corporate_client_deleted_by', 'FOREIGN KEY (deleted_by) REFERENCES app.account(id)'),
      ('corporate_client_document', 'fk_corporate_client_document_created_by', 'FOREIGN KEY (created_by) REFERENCES app.account(id)'),
      ('corporate_client_document', 'fk_corporate_client_document_updated_by', 'FOREIGN KEY (updated_by) REFERENCES app.account(id)'),
      ('corporate_client_document', 'fk_corporate_client_document_deleted_by', 'FOREIGN KEY (deleted_by) REFERENCES app.account(id)'),
      ('corporate_client_document', 'fk_corporate_client_document_corporate_client_id', 'FOREIGN KEY (corporate_client_id) REFERENCES app.corporate_client(id)'),
      ('corporate_client_document', 'fk_corporate_client_document_stored_file_id', 'FOREIGN KEY (stored_file_id) REFERENCES app.stored_file(id)'),
      ('corporate_verification_decision', 'fk_corporate_verification_decision_created_by', 'FOREIGN KEY (created_by) REFERENCES app.account(id)'),
      ('corporate_verification_decision', 'fk_corporate_verification_decision_corporate_client_id', 'FOREIGN KEY (corporate_client_id) REFERENCES app.corporate_client(id)'),
      ('corporate_verification_decision', 'fk_corporate_verification_decision_decided_by', 'FOREIGN KEY (decided_by) REFERENCES app.account(id)')) v(t, n, def),
        LATERAL (SELECT format('%s: %s', v.n, coalesce(pg_get_constraintdef(k.oid), '<tidak ada>')) AS x
                   FROM (SELECT 1) d LEFT JOIN pg_constraint k ON k.conname = v.n AND k.conrelid = to_regclass('app.' || v.t)) z
         WHERE x IS DISTINCT FROM format('%s: %s', v.n, v.def)
        UNION ALL
        SELECT format('jumlah FK %s: %s', c.relname, count(k.oid)) FROM pg_class c LEFT JOIN pg_constraint k ON k.conrelid = c.oid AND k.contype = 'f'
         WHERE c.oid IN ('app.corporate_client'::regclass, 'app.corporate_client_document'::regclass, 'app.corporate_verification_decision'::regclass)
         GROUP BY c.relname
        HAVING count(k.oid) <> CASE c.relname WHEN 'corporate_client' THEN 3 WHEN 'corporate_client_document' THEN 5 ELSE 3 END));
SELECT pg_temp.uji_nol('A20 CHECK enum persis (status, asal, jenis dokumen, keputusan, kode alasan)',
  ARRAY(SELECT x FROM (VALUES
      ('corporate_client', 'ck_corporate_client_status', 'CHECK ((status = ANY (ARRAY[''pending_verification''::text, ''active''::text, ''rejected''::text, ''inactive''::text])))'),
      ('corporate_client', 'ck_corporate_client_origin', 'CHECK ((origin = ANY (ARRAY[''self_registered''::text, ''created_by_admin''::text])))'),
      ('corporate_client', 'ck_corporate_client_submission_round', 'CHECK ((submission_round >= 1))'),
      ('corporate_client_document', 'ck_corporate_client_document_document_kind', 'CHECK ((document_kind = ANY (ARRAY[''nib''::text, ''npwp''::text, ''authority_proof''::text])))'),
      ('corporate_verification_decision', 'ck_corporate_verification_decision_decision', 'CHECK ((decision = ANY (ARRAY[''approved''::text, ''rejected''::text])))'),
      ('corporate_verification_decision', 'ck_corporate_verification_decision_reason_code', 'CHECK (((decision = ''rejected''::text) = (reason_code IS NOT NULL)))'),
      ('corporate_verification_decision', 'ck_corporate_verification_decision_submission_round', 'CHECK ((submission_round >= 1))')) v(t, n, def),
        LATERAL (SELECT format('%s: %s', v.n, coalesce(pg_get_constraintdef(k.oid), '<tidak ada>')) AS x
                   FROM (SELECT 1) d LEFT JOIN pg_constraint k ON k.conname = v.n AND k.conrelid = to_regclass('app.' || v.t)) z
         WHERE x IS DISTINCT FROM format('%s: %s', v.n, v.def)));
SELECT pg_temp.uji_nol('A20 trigger: Master tanpa trigger; Transaksi persis dua trigger penolak tingkat statement (§4.5, §4.11)',
  ARRAY(SELECT format('%s: %s', tgrelid::regclass, tgname) FROM pg_trigger
         WHERE tgrelid IN ('app.corporate_client'::regclass, 'app.corporate_client_document'::regclass) AND NOT tgisinternal
        UNION ALL
        SELECT format('corporate_verification_decision: %s', coalesce(string_agg(tgname::text, ',' ORDER BY tgname), '<kosong>'))
          FROM pg_trigger WHERE tgrelid = 'app.corporate_verification_decision'::regclass AND NOT tgisinternal
        HAVING coalesce(string_agg(tgname::text, ',' ORDER BY tgname), '') <>
               'tg_corporate_verification_decision_reject_truncate,tg_corporate_verification_decision_reject_ud'));

-- 2. Negatif atas pemeriksa, lalu perilaku. Semua di-ROLLBACK.
BEGIN;
DO $$
DECLARE
  c record;
  kelas text := $q$SELECT pg_temp.uji_kelas_app('app', '{consent_text_version}')$q$;
BEGIN
  FOR c IN SELECT * FROM (VALUES
      ('T-06 DELETE perusahaan untuk rs_app (Master, ADR-003)', 'GRANT DELETE ON app.corporate_client TO rs_app', kelas, 'T-06 corporate_client .*lebih rs_app:DELETE'),
      ('T-06 UPDATE dokumen untuk pekerja privasi', 'GRANT UPDATE ON app.corporate_client_document TO rs_privacy_worker', kelas, 'T-06 corporate_client_document .*lebih rs_privacy_worker:UPDATE'),
      ('T-06 UPDATE keputusan untuk rs_app (Transaksi, ADR-004)', 'GRANT UPDATE ON app.corporate_verification_decision TO rs_app', kelas, 'T-06 corporate_verification_decision .*lebih rs_app:UPDATE'),
      ('T-06 SELECT kolom pemutus untuk pekerja privasi (melebihi ID/FK subjek)', 'GRANT SELECT (decided_by) ON app.corporate_verification_decision TO rs_privacy_worker', kelas, 'T-06 corporate_verification_decision: SELECT per kolom rs_privacy_worker'),
      ('T-08 trigger penolak keputusan dilepas', 'DROP TRIGGER tg_corporate_verification_decision_reject_ud ON app.corporate_verification_decision', kelas, 'T-08 corporate_verification_decision'),
      ('T-08 trigger penolak keputusan dinonaktifkan', 'ALTER TABLE app.corporate_verification_decision DISABLE TRIGGER tg_corporate_verification_decision_reject_truncate', kelas, 'T-08 corporate_verification_decision'),
      ('index uqp nib tanpa WHERE deleted_at terdeteksi', 'DROP INDEX app.uqp_corporate_client_nib; CREATE UNIQUE INDEX uqp_corporate_client_nib ON app.corporate_client (nib)',
       $q$SELECT pg_temp.uji_indeks('app.corporate_client', ARRAY['CREATE UNIQUE INDEX pk_corporate_client ON app.corporate_client USING btree (id)', 'CREATE UNIQUE INDEX uqp_corporate_client_nib ON app.corporate_client USING btree (nib) WHERE (deleted_at IS NULL)', 'CREATE INDEX ix_corporate_client_submitted_at_pending ON app.corporate_client USING btree (submitted_at) WHERE ((deleted_at IS NULL) AND (status = ''pending_verification''::text))'])$q$,
       'index app.corporate_client kurang: .*WHERE \(deleted_at IS NULL\)')
    ) AS t(label, stmt, pemeriksa, pola)
  LOOP
    PERFORM pg_temp.uji_rusak(c.label, c.stmt, c.pemeriksa, c.pola);
  END LOOP;
END $$;

-- Data awal oleh pemilik skema (data sintetis): akun Verifikator dan empat berkas dokumen tervalidasi.
SET LOCAL ROLE rs_migration;
INSERT INTO app.account (id, account_type, idp_realm, idp_subject, email, status, status_changed_at, full_name)
  VALUES ('00000000-0000-7000-8000-0000000a2001', 'rs_admin', 'admin', 'sub-a20-1', 'a20-1@contoh.invalid', 'active', now(), 'Verifikator Sintetis');
INSERT INTO app.stored_file (id, bucket, object_key, category, mime_type, size_bytes, sha256, validated_at)
  VALUES ('00000000-0000-7000-8000-0000000a2011', 'identity_document', 'a20-objek-1', 'corporate_document', 'application/pdf', 10, decode(repeat('a1', 32), 'hex'), now()),
         ('00000000-0000-7000-8000-0000000a2012', 'identity_document', 'a20-objek-2', 'corporate_document', 'application/pdf', 10, decode(repeat('a2', 32), 'hex'), now()),
         ('00000000-0000-7000-8000-0000000a2013', 'identity_document', 'a20-objek-3', 'corporate_document', 'application/pdf', 10, decode(repeat('a3', 32), 'hex'), now()),
         ('00000000-0000-7000-8000-0000000a2014', 'identity_document', 'a20-objek-4', 'corporate_document', 'application/pdf', 10, decode(repeat('a4', 32), 'hex'), now());
RESET ROLE;

-- corporate_client (M).
SELECT pg_temp.uji_boleh('A20 rs_api mendaftarkan perusahaan menunggu verifikasi (US-1901 AC1)', 'rs_api',
  'INSERT INTO app.corporate_client (id, legal_name, nib, npwp, office_address, office_phone, status, origin) VALUES (''00000000-0000-7000-8000-0000000a2021'', ''PT Sintetis Satu'', ''NIB-UJI-1'', ''NPWP-UJI-1'', ''Jl. Uji 1'', ''0210000001'', ''pending_verification'', ''self_registered'')', 1);
SELECT pg_temp.uji_boleh('A20 rs_api membuat perusahaan aktif oleh admin (US-1903 AC1)', 'rs_api',
  'INSERT INTO app.corporate_client (id, created_by, legal_name, nib, npwp, office_address, office_phone, status, origin) VALUES (''00000000-0000-7000-8000-0000000a2022'', ''00000000-0000-7000-8000-0000000a2001'', ''PT Sintetis Dua'', ''NIB-UJI-2'', ''NPWP-UJI-2'', ''Jl. Uji 2'', ''0210000002'', ''active'', ''created_by_admin'')', 1);
SELECT pg_temp.uji_tolak('A20 NEG NIB ganda pada perusahaan aktif ditolak (US-1901 AC7, US-1903 AC4)', 'rs_api',
  'INSERT INTO app.corporate_client (id, legal_name, nib, npwp, office_address, office_phone, status, origin) VALUES (''00000000-0000-7000-8000-0000000a2023'', ''PT Lain'', ''NIB-UJI-1'', ''NPWP-UJI-3'', ''Jl. Uji 3'', ''0210000003'', ''pending_verification'', ''self_registered'')',
  '23505', 'uqp_corporate_client_nib');
SELECT pg_temp.uji_tolak('A20 NEG status di luar daftar', 'rs_api',
  'UPDATE app.corporate_client SET status = ''verified'' WHERE id = ''00000000-0000-7000-8000-0000000a2021''', '23514', 'ck_corporate_client_status');
SELECT pg_temp.uji_tolak('A20 NEG asal di luar daftar', 'rs_api',
  'UPDATE app.corporate_client SET origin = ''imported'' WHERE id = ''00000000-0000-7000-8000-0000000a2021''', '23514', 'ck_corporate_client_origin');
SELECT pg_temp.uji_tolak('A20 NEG NIB kosong', 'rs_api',
  'UPDATE app.corporate_client SET nib = ''  '' WHERE id = ''00000000-0000-7000-8000-0000000a2021''', '23514', 'ck_corporate_client_nib_length');
SELECT pg_temp.uji_tolak('A20 NEG nama badan usaha melebihi batas', 'rs_api',
  'UPDATE app.corporate_client SET legal_name = repeat(''x'', 201) WHERE id = ''00000000-0000-7000-8000-0000000a2021''', '23514', 'ck_corporate_client_legal_name_length');
SELECT pg_temp.uji_tolak('A20 NEG pengajuan ke-0', 'rs_api',
  'UPDATE app.corporate_client SET submission_round = 0 WHERE id = ''00000000-0000-7000-8000-0000000a2021''', '23514', 'ck_corporate_client_submission_round');
SELECT pg_temp.uji_tolak('A20 NEG soft delete tanpa alasan', 'rs_api',
  'UPDATE app.corporate_client SET deleted_at = now() WHERE id = ''00000000-0000-7000-8000-0000000a2022''', '23514', 'ck_corporate_client_soft_delete');
SELECT pg_temp.uji_tolak('A20 NEG rs_api DELETE perusahaan (Master, ADR-003)', 'rs_api',
  'DELETE FROM app.corporate_client', '42501', 'permission denied for table corporate_client');
SELECT pg_temp.uji_tolak('A20 NEG rs_worker TRUNCATE perusahaan', 'rs_worker',
  'TRUNCATE app.corporate_client CASCADE', '42501', 'permission denied for table');
SELECT pg_temp.uji_tolak('A20 NEG pekerja privasi UPDATE perusahaan (tanpa kolom anonymize)', 'rs_privacy_worker_login',
  'UPDATE app.corporate_client SET updated_at = now()', '42501', 'permission denied for table corporate_client');
SELECT pg_temp.uji_tolak('A20 NEG relay audit membaca perusahaan', 'rs_audit_relay_login',
  'SELECT id FROM app.corporate_client', '42501', 'permission denied for table corporate_client');
SELECT pg_temp.uji_tolak('A20 NEG rotasi kunci membaca perusahaan', 'rs_key_rotation',
  'SELECT id FROM app.corporate_client', '42501', 'permission denied for table corporate_client');

-- corporate_client_document (M).
SELECT pg_temp.uji_boleh('A20 rs_api menyimpan dokumen NIB, NPWP, bukti kewenangan (US-1901 AC1, C21-P4 4b)', 'rs_api',
  'INSERT INTO app.corporate_client_document (id, corporate_client_id, document_kind, stored_file_id) VALUES '
  || '(''00000000-0000-7000-8000-0000000a2031'', ''00000000-0000-7000-8000-0000000a2021'', ''nib'', ''00000000-0000-7000-8000-0000000a2011''), '
  || '(''00000000-0000-7000-8000-0000000a2032'', ''00000000-0000-7000-8000-0000000a2021'', ''npwp'', ''00000000-0000-7000-8000-0000000a2012''), '
  || '(''00000000-0000-7000-8000-0000000a2033'', ''00000000-0000-7000-8000-0000000a2021'', ''authority_proof'', ''00000000-0000-7000-8000-0000000a2013'')', 3);
SELECT pg_temp.uji_tolak('A20 NEG dokumen aktif kedua untuk jenis yang sama', 'rs_api',
  'INSERT INTO app.corporate_client_document (id, corporate_client_id, document_kind, stored_file_id) VALUES (''00000000-0000-7000-8000-0000000a2034'', ''00000000-0000-7000-8000-0000000a2021'', ''authority_proof'', ''00000000-0000-7000-8000-0000000a2014'')',
  '23505', 'uqp_corp_client_doc_corporate_client_id_document_kind');
SELECT pg_temp.uji_tolak('A20 NEG satu berkas dipakai dua dokumen (perusahaan lain)', 'rs_api',
  'INSERT INTO app.corporate_client_document (id, corporate_client_id, document_kind, stored_file_id) VALUES (''00000000-0000-7000-8000-0000000a2034'', ''00000000-0000-7000-8000-0000000a2022'', ''nib'', ''00000000-0000-7000-8000-0000000a2011'')',
  '23505', 'uq_corporate_client_document_stored_file_id');
SELECT pg_temp.uji_tolak('A20 NEG jenis dokumen di luar daftar', 'rs_api',
  'INSERT INTO app.corporate_client_document (id, corporate_client_id, document_kind, stored_file_id) VALUES (''00000000-0000-7000-8000-0000000a2034'', ''00000000-0000-7000-8000-0000000a2022'', ''ktp'', ''00000000-0000-7000-8000-0000000a2014'')',
  '23514', 'ck_corporate_client_document_document_kind');
SELECT pg_temp.uji_tolak('A20 NEG dokumen untuk perusahaan yang tidak ada', 'rs_api',
  'INSERT INTO app.corporate_client_document (id, corporate_client_id, document_kind, stored_file_id) VALUES (''00000000-0000-7000-8000-0000000a2034'', ''00000000-0000-7000-8000-0000000affff'', ''nib'', ''00000000-0000-7000-8000-0000000a2014'')',
  '23503', 'fk_corporate_client_document_corporate_client_id');
SELECT pg_temp.uji_tolak('A20 NEG dokumen dengan berkas yang tidak ada', 'rs_api',
  'INSERT INTO app.corporate_client_document (id, corporate_client_id, document_kind, stored_file_id) VALUES (''00000000-0000-7000-8000-0000000a2034'', ''00000000-0000-7000-8000-0000000a2022'', ''nib'', ''00000000-0000-7000-8000-0000000affff'')',
  '23503', 'fk_corporate_client_document_stored_file_id');
SELECT pg_temp.uji_boleh('A20 penggantian bukti kewenangan: baris lama dihapus lunak lalu baris baru disisipkan (US-1902 AC3)', 'rs_api',
  'WITH lama AS (UPDATE app.corporate_client_document SET deleted_at = now(), deleted_by = ''00000000-0000-7000-8000-0000000a2001'', delete_reason = ''diganti'', updated_at = now() WHERE id = ''00000000-0000-7000-8000-0000000a2033'' RETURNING corporate_client_id) '
  || 'INSERT INTO app.corporate_client_document (id, corporate_client_id, document_kind, stored_file_id) '
  || 'SELECT ''00000000-0000-7000-8000-0000000a2034'', corporate_client_id, ''authority_proof'', ''00000000-0000-7000-8000-0000000a2014'' FROM lama', 1);
SELECT pg_temp.uji_tolak('A20 NEG rs_api DELETE dokumen (Master, ADR-003)', 'rs_api',
  'DELETE FROM app.corporate_client_document', '42501', 'permission denied for table corporate_client_document');
SELECT pg_temp.uji_tolak('A20 NEG pekerja privasi UPDATE dokumen', 'rs_privacy_worker_login',
  'UPDATE app.corporate_client_document SET updated_at = now()', '42501', 'permission denied for table corporate_client_document');
SELECT pg_temp.uji_boleh('A20 pekerja privasi membaca dokumen (Master: S)', 'rs_privacy_worker_login',
  'SELECT id FROM app.corporate_client_document', 4);

-- corporate_verification_decision (T).
SELECT pg_temp.uji_tolak('A20 NEG penolakan tanpa kode alasan (US-1902 AC3)', 'rs_api',
  'INSERT INTO app.corporate_verification_decision (id, corporate_client_id, submission_round, decision, decided_by) VALUES (''00000000-0000-7000-8000-0000000a2041'', ''00000000-0000-7000-8000-0000000a2021'', 1, ''rejected'', ''00000000-0000-7000-8000-0000000a2001'')',
  '23514', 'ck_corporate_verification_decision_reason_code');
SELECT pg_temp.uji_tolak('A20 NEG persetujuan dengan kode alasan', 'rs_api',
  'INSERT INTO app.corporate_verification_decision (id, corporate_client_id, submission_round, decision, reason_code, decided_by) VALUES (''00000000-0000-7000-8000-0000000a2041'', ''00000000-0000-7000-8000-0000000a2021'', 1, ''approved'', ''x'', ''00000000-0000-7000-8000-0000000a2001'')',
  '23514', 'ck_corporate_verification_decision_reason_code');
SELECT pg_temp.uji_tolak('A20 NEG keputusan tanpa pemutus', 'rs_api',
  'INSERT INTO app.corporate_verification_decision (id, corporate_client_id, submission_round, decision) VALUES (''00000000-0000-7000-8000-0000000a2041'', ''00000000-0000-7000-8000-0000000a2021'', 1, ''approved'')',
  '23502', 'decided_by');
SELECT pg_temp.uji_tolak('A20 NEG keputusan di luar daftar', 'rs_api',
  'INSERT INTO app.corporate_verification_decision (id, corporate_client_id, submission_round, decision, decided_by) VALUES (''00000000-0000-7000-8000-0000000a2041'', ''00000000-0000-7000-8000-0000000a2021'', 1, ''pending'', ''00000000-0000-7000-8000-0000000a2001'')',
  '23514', 'ck_corporate_verification_decision_decision');
SELECT pg_temp.uji_boleh('A20 rs_api mencatat penolakan pengajuan ke-1 dengan kode alasan', 'rs_api',
  'INSERT INTO app.corporate_verification_decision (id, created_by, corporate_client_id, submission_round, decision, reason_code, decided_by) VALUES (''00000000-0000-7000-8000-0000000a2041'', ''00000000-0000-7000-8000-0000000a2001'', ''00000000-0000-7000-8000-0000000a2021'', 1, ''rejected'', ''authority_proof_insufficient'', ''00000000-0000-7000-8000-0000000a2001'')', 1);
SELECT pg_temp.uji_tolak('A20 NEG keputusan kedua atas pengajuan yang sama ditolak (keputusan pertama menang, US-1902 AC4)', 'rs_api',
  'INSERT INTO app.corporate_verification_decision (id, corporate_client_id, submission_round, decision, decided_by) VALUES (''00000000-0000-7000-8000-0000000a2042'', ''00000000-0000-7000-8000-0000000a2021'', 1, ''approved'', ''00000000-0000-7000-8000-0000000a2001'')',
  '23505', 'uq_corp_verif_decision_corporate_client_id_submission_round');
SELECT pg_temp.uji_boleh('A20 pengajuan ulang setelah ditolak (US-1902 AC3): putaran naik', 'rs_api',
  'UPDATE app.corporate_client SET submission_round = submission_round + 1, submitted_at = now(), status = ''pending_verification'', updated_at = now() WHERE id = ''00000000-0000-7000-8000-0000000a2021''', 1);
SELECT pg_temp.uji_boleh('A20 keputusan atas pengajuan ke-2 diterima', 'rs_api',
  'INSERT INTO app.corporate_verification_decision (id, corporate_client_id, submission_round, decision, decided_by) VALUES (''00000000-0000-7000-8000-0000000a2042'', ''00000000-0000-7000-8000-0000000a2021'', 2, ''approved'', ''00000000-0000-7000-8000-0000000a2001'')', 1);
SELECT pg_temp.uji_tolak('A20 NEG keputusan untuk perusahaan yang tidak ada', 'rs_api',
  'INSERT INTO app.corporate_verification_decision (id, corporate_client_id, submission_round, decision, decided_by) VALUES (''00000000-0000-7000-8000-0000000a2043'', ''00000000-0000-7000-8000-0000000affff'', 1, ''approved'', ''00000000-0000-7000-8000-0000000a2001'')',
  '23503', 'fk_corporate_verification_decision_corporate_client_id');
SELECT pg_temp.uji_tolak('A20 NEG pemutus yang tidak ada', 'rs_api',
  'INSERT INTO app.corporate_verification_decision (id, corporate_client_id, submission_round, decision, decided_by) VALUES (''00000000-0000-7000-8000-0000000a2043'', ''00000000-0000-7000-8000-0000000a2022'', 1, ''approved'', ''00000000-0000-7000-8000-0000000affff'')',
  '23503', 'fk_corporate_verification_decision_decided_by');
SELECT pg_temp.uji_tolak('A20 NEG rs_api UPDATE keputusan (Transaksi, ADR-004)', 'rs_api',
  'UPDATE app.corporate_verification_decision SET decision = ''approved''', '42501', 'permission denied for table corporate_verification_decision');
SELECT pg_temp.uji_tolak('A20 NEG rs_worker DELETE keputusan', 'rs_worker',
  'DELETE FROM app.corporate_verification_decision', '42501', 'permission denied for table corporate_verification_decision');
SELECT pg_temp.uji_tolak('A20 NEG pemilik UPDATE keputusan ditolak trigger', 'rs_migration',
  'UPDATE app.corporate_verification_decision SET decision = ''approved''', '42501', '^immutable_violation: UPDATE on app\.corporate_verification_decision rejected$');
SELECT pg_temp.uji_tolak('A20 NEG pemilik DELETE keputusan ditolak trigger', 'rs_migration',
  'DELETE FROM app.corporate_verification_decision', '42501', '^immutable_violation: DELETE on app\.corporate_verification_decision rejected$');
SELECT pg_temp.uji_tolak('A20 NEG pemilik TRUNCATE keputusan ditolak trigger', 'rs_migration',
  'TRUNCATE app.corporate_verification_decision', '42501', '^immutable_violation: TRUNCATE on app\.corporate_verification_decision rejected$');
SELECT pg_temp.uji_tolak('A20 NEG pekerja privasi membaca keputusan (tanpa FK subjek data pribadi)', 'rs_privacy_worker_login',
  'SELECT id FROM app.corporate_verification_decision', '42501', 'permission denied for table corporate_verification_decision');
SELECT pg_temp.uji_tolak('A20 NEG perusahaan yang punya keputusan tidak dapat dihapus keras walau oleh pemilik (FK NO ACTION)', 'rs_migration',
  'DELETE FROM app.corporate_client WHERE id = ''00000000-0000-7000-8000-0000000a2021''', '23503', 'fk_');
ROLLBACK;

DO $$
BEGIN
  IF EXISTS (SELECT 1 FROM app.corporate_client) OR EXISTS (SELECT 1 FROM app.corporate_client_document)
     OR EXISTS (SELECT 1 FROM app.corporate_verification_decision) OR EXISTS (SELECT 1 FROM app.stored_file)
     OR EXISTS (SELECT 1 FROM app.account) THEN
    RAISE EXCEPTION 'GAGAL pembersihan tabel-app-a20: data uji tersisa';
  END IF;
  RAISE NOTICE 'LULUS pembersihan tabel-app-a20: tidak ada data uji tersisa (ROLLBACK)';
END $$;
