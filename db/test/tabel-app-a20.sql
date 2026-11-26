-- db/test/tabel-app-a20.sql: test tabel A20 layanan korporat K1 (F1-T98, F1-T99) di database ruang_sehat:
-- corporate_client (M), corporate_client_document (M), corporate_verification_decision (T), corporate_member (M),
-- dan perluasan A1.1 account (corporate_admin, realm korporat).
-- Didahului db/test/lib.sql; dijalankan superuser setelah migrasi. Aturan kelas generik (T-06/T-07/T-08) di
-- db/test/kelas-app.sql bagian A, T-01/T-20 generik di db/test/katalog.sql. Berkas ini memeriksa struktur khusus
-- (komentar kelas, kolom, hak tabel persis, index, FK, CHECK, trigger) dan perilaku dengan role sungguhan
-- (positif dan negatif). Satu transaksi yang di-ROLLBACK. Data sintetis.
-- Dasar: docs/04-data-dictionary.md v1.2 §0.4, §0.6 A, A20; docs/05-physical-schema.md §1, §4.1, §4.3, §4.5,
-- §4.11, §11.3, §12 (T-06, T-07, T-08, T-09); ADR-003, ADR-004, ADR-008, ADR-027 butir 3, ADR-028 butir 1; PRD v1.3
-- US-1901 AC1/AC7, US-1902 AC3-AC4, US-1904 AC2.

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
SELECT pg_temp.uji_nol('A20 corporate_client: index persis pk, uqp nib hanya perusahaan Aktif (CR-035), ix nib, ix antrean pending',
  ARRAY(SELECT pg_temp.uji_indeks('app.corporate_client', ARRAY[
    'CREATE UNIQUE INDEX pk_corporate_client ON app.corporate_client USING btree (id)',
    'CREATE UNIQUE INDEX uqp_corporate_client_nib_active ON app.corporate_client USING btree (nib) WHERE ((deleted_at IS NULL) AND (status = ''active''::text))',
    'CREATE INDEX ix_corporate_client_nib ON app.corporate_client USING btree (nib) WHERE (deleted_at IS NULL)',
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
      ('corporate_client', 'ck_corporate_client_nib_trimmed', 'CHECK ((nib = btrim(nib)))'),
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

-- 1b. Struktur F1-T99: corporate_member (M) dan perluasan account.
SELECT pg_temp.uji_nol('A20 corporate_member: komentar kelas M modul korporat; kolom anonymize persis job_title',
  ARRAY(SELECT coalesce(obj_description('app.corporate_member'::regclass, 'pg_class'), '<tidak ada>')
         WHERE obj_description('app.corporate_member'::regclass, 'pg_class') IS DISTINCT FROM 'class:M module:korporat'
        UNION ALL
        SELECT x FROM (
          SELECT 'kolom anonymize ' || coalesce(string_agg(attname::text, ',' ORDER BY attname), '<kosong>') AS x
            FROM pg_attribute WHERE attrelid = 'app.corporate_member'::regclass AND attnum > 0 AND NOT attisdropped
             AND col_description(attrelid, attnum) = 'anonymize') s
         WHERE x <> 'kolom anonymize job_title'));
SELECT pg_temp.uji_nol('A20 corporate_member: kolom persis (STD-M + account_id, corporate_client_id, job_title, anonymized_at)',
  ARRAY(SELECT format('kolom %s', x) FROM (
          SELECT string_agg(attname || ':' || format_type(atttypid, atttypmod) || ':' || CASE WHEN attnotnull THEN 'nn' ELSE 'n' END,
                            ',' ORDER BY attnum) AS x
            FROM pg_attribute WHERE attrelid = 'app.corporate_member'::regclass AND attnum > 0 AND NOT attisdropped) s
         WHERE x IS DISTINCT FROM
           'id:uuid:nn,created_at:timestamp with time zone:nn,created_by:uuid:n,updated_at:timestamp with time zone:nn,'
           || 'updated_by:uuid:n,deleted_at:timestamp with time zone:n,deleted_by:uuid:n,delete_reason:text:n,'
           || 'account_id:uuid:nn,corporate_client_id:uuid:nn,job_title:text:n,anonymized_at:timestamp with time zone:n'));
SELECT pg_temp.uji_nol('A20 corporate_member: hak tabel persis (rs_app S,I,U + pekerja privasi S) dan U per kolom pekerja privasi persis',
  ARRAY(SELECT format('corporate_member: %s', coalesce(a.k, '<kosong>'))
          FROM (SELECT string_agg(CASE WHEN x.grantee = 0 THEN 'PUBLIC' ELSE pg_get_userbyid(x.grantee) END || ':' || x.privilege_type,
                                  ',' ORDER BY 1) AS k
                  FROM pg_class c, aclexplode(c.relacl) x
                 WHERE c.oid = 'app.corporate_member'::regclass AND x.grantee <> c.relowner) a
         WHERE a.k IS DISTINCT FROM 'rs_app:INSERT,rs_app:SELECT,rs_app:UPDATE,rs_privacy_worker:SELECT'
        UNION ALL
        SELECT format('hak per kolom %s', k) FROM (
          SELECT string_agg(att.attname || ':' || pg_get_userbyid(x.grantee) || ':' || x.privilege_type, ',' ORDER BY att.attname) AS k
            FROM pg_attribute att, aclexplode(att.attacl) x
           WHERE att.attrelid = 'app.corporate_member'::regclass AND att.attnum > 0 AND att.attacl IS NOT NULL) s
         WHERE k IS DISTINCT FROM
           'anonymized_at:rs_privacy_worker:UPDATE,job_title:rs_privacy_worker:UPDATE,updated_at:rs_privacy_worker:UPDATE,updated_by:rs_privacy_worker:UPDATE'));
SELECT pg_temp.uji_nol('A20 corporate_member: index persis pk, uqp satu keanggotaan aktif per akun (M-51), ix anggota aktif per perusahaan',
  ARRAY(SELECT pg_temp.uji_indeks('app.corporate_member', ARRAY[
    'CREATE UNIQUE INDEX pk_corporate_member ON app.corporate_member USING btree (id)',
    'CREATE UNIQUE INDEX uqp_corporate_member_account_id ON app.corporate_member USING btree (account_id) WHERE (deleted_at IS NULL)',
    'CREATE INDEX ix_corporate_member_corporate_client_id ON app.corporate_member USING btree (corporate_client_id) WHERE (deleted_at IS NULL)'])));
SELECT pg_temp.uji_nol('A20 corporate_member: FK persis, tanpa CASCADE; tanpa trigger',
  ARRAY(SELECT x FROM (VALUES
      ('fk_corporate_member_created_by', 'FOREIGN KEY (created_by) REFERENCES app.account(id)'),
      ('fk_corporate_member_updated_by', 'FOREIGN KEY (updated_by) REFERENCES app.account(id)'),
      ('fk_corporate_member_deleted_by', 'FOREIGN KEY (deleted_by) REFERENCES app.account(id)'),
      ('fk_corporate_member_account_id', 'FOREIGN KEY (account_id) REFERENCES app.account(id)'),
      ('fk_corporate_member_corporate_client_id', 'FOREIGN KEY (corporate_client_id) REFERENCES app.corporate_client(id)')) v(n, def),
        LATERAL (SELECT format('%s: %s', v.n, coalesce(pg_get_constraintdef(k.oid), '<tidak ada>')) AS x
                   FROM (SELECT 1) d LEFT JOIN pg_constraint k ON k.conname = v.n AND k.conrelid = 'app.corporate_member'::regclass) z
         WHERE x IS DISTINCT FROM format('%s: %s', v.n, v.def)
        UNION ALL
        SELECT format('jumlah FK %s', count(*)) FROM pg_constraint
         WHERE conrelid = 'app.corporate_member'::regclass AND contype = 'f' HAVING count(*) <> 5
        UNION ALL
        SELECT format('trigger %s', tgname) FROM pg_trigger WHERE tgrelid = 'app.corporate_member'::regclass AND NOT tgisinternal));
SELECT pg_temp.uji_nol('A1.1 account (expand F1-T99): CHECK jenis, realm, dan pasangan persis termasuk corporate_admin -> korporat; full_name tidak berubah',
  ARRAY(SELECT x FROM (VALUES
      ('ck_account_account_type', 'CHECK ((account_type = ANY (ARRAY[''patient''::text, ''health_worker''::text, ''rs_admin''::text, ''partner_admin''::text, ''corporate_admin''::text])))'),
      ('ck_account_idp_realm', 'CHECK ((idp_realm = ANY (ARRAY[''pasien''::text, ''nakes''::text, ''admin''::text, ''korporat''::text])))'),
      ('ck_account_account_type_idp_realm', 'CHECK ((((account_type = ''patient''::text) AND (idp_realm = ''pasien''::text)) OR ((account_type = ''health_worker''::text) AND (idp_realm = ''nakes''::text)) OR ((account_type = ANY (ARRAY[''rs_admin''::text, ''partner_admin''::text])) AND (idp_realm = ''admin''::text)) OR ((account_type = ''corporate_admin''::text) AND (idp_realm = ''korporat''::text))))'),
      ('ck_account_full_name', 'CHECK (((account_type <> ALL (ARRAY[''rs_admin''::text, ''partner_admin''::text])) OR (full_name IS NOT NULL)))')) v(n, def),
        LATERAL (SELECT format('%s: %s', v.n, coalesce(pg_get_constraintdef(k.oid), '<tidak ada>')) AS x
                   FROM (SELECT 1) d LEFT JOIN pg_constraint k ON k.conname = v.n AND k.conrelid = 'app.account'::regclass) z
         WHERE x IS DISTINCT FROM format('%s: %s', v.n, v.def)
        UNION ALL
        SELECT format('CHECK account belum tervalidasi: %s', conname) FROM pg_constraint
         WHERE conrelid = 'app.account'::regclass AND contype = 'c' AND NOT convalidated));
SELECT pg_temp.uji_nol('F1-T34 idp_event_cursor/idp_event_receipt (expand F1-T99): CHECK realm persis termasuk korporat',
  ARRAY(SELECT x FROM (VALUES
      ('idp_event_cursor', 'ck_idp_event_cursor_idp_realm', 'CHECK ((idp_realm = ANY (ARRAY[''pasien''::text, ''nakes''::text, ''admin''::text, ''korporat''::text])))'),
      ('idp_event_receipt', 'ck_idp_event_receipt_idp_realm', 'CHECK ((idp_realm = ANY (ARRAY[''pasien''::text, ''nakes''::text, ''admin''::text, ''korporat''::text])))')) v(t, n, def),
        LATERAL (SELECT format('%s: %s', v.n, coalesce(pg_get_constraintdef(k.oid), '<tidak ada>')) AS x
                   FROM (SELECT 1) d LEFT JOIN pg_constraint k ON k.conname = v.n AND k.conrelid = to_regclass('app.' || v.t)) z
         WHERE x IS DISTINCT FROM format('%s: %s', v.n, v.def)));

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
      ('T-07 UPDATE kolom non-anonymize keanggotaan untuk pekerja privasi', 'GRANT UPDATE (corporate_client_id) ON app.corporate_member TO rs_privacy_worker', kelas, 'T-07 corporate_member'),
      ('T-06 DELETE keanggotaan untuk rs_app (Master, ADR-003)', 'GRANT DELETE ON app.corporate_member TO rs_app', kelas, 'T-06 corporate_member .*lebih rs_app:DELETE'),
      ('index uqp nib aktif tanpa syarat status terdeteksi (CR-035)', 'DROP INDEX app.uqp_corporate_client_nib_active; CREATE UNIQUE INDEX uqp_corporate_client_nib_active ON app.corporate_client (nib) WHERE deleted_at IS NULL',
       $q$SELECT pg_temp.uji_indeks('app.corporate_client', ARRAY['CREATE UNIQUE INDEX pk_corporate_client ON app.corporate_client USING btree (id)', 'CREATE UNIQUE INDEX uqp_corporate_client_nib_active ON app.corporate_client USING btree (nib) WHERE ((deleted_at IS NULL) AND (status = ''active''::text))', 'CREATE INDEX ix_corporate_client_nib ON app.corporate_client USING btree (nib) WHERE (deleted_at IS NULL)', 'CREATE INDEX ix_corporate_client_submitted_at_pending ON app.corporate_client USING btree (submitted_at) WHERE ((deleted_at IS NULL) AND (status = ''pending_verification''::text))'])$q$,
       'index app.corporate_client kurang: .*status = ''active'''),
      ('index uqp nib lama (status apa pun) kembali terdeteksi (CR-035)', 'CREATE UNIQUE INDEX uqp_corporate_client_nib ON app.corporate_client (nib) WHERE deleted_at IS NULL',
       $q$SELECT pg_temp.uji_indeks('app.corporate_client', ARRAY['CREATE UNIQUE INDEX pk_corporate_client ON app.corporate_client USING btree (id)', 'CREATE UNIQUE INDEX uqp_corporate_client_nib_active ON app.corporate_client USING btree (nib) WHERE ((deleted_at IS NULL) AND (status = ''active''::text))', 'CREATE INDEX ix_corporate_client_nib ON app.corporate_client USING btree (nib) WHERE (deleted_at IS NULL)', 'CREATE INDEX ix_corporate_client_submitted_at_pending ON app.corporate_client USING btree (submitted_at) WHERE ((deleted_at IS NULL) AND (status = ''pending_verification''::text))'])$q$,
       'index app.corporate_client lebih: .*uqp_corporate_client_nib ')
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
-- CR-035 (Q10 = B2): NIB unik hanya di antara perusahaan Aktif yang belum dihapus lunak.
SELECT pg_temp.uji_boleh('A20 pengajuan menunggu verifikasi boleh memakai NIB perusahaan Aktif (CR-035)', 'rs_api',
  'INSERT INTO app.corporate_client (id, legal_name, nib, npwp, office_address, office_phone, status, origin) VALUES (''00000000-0000-7000-8000-0000000a2027'', ''PT Lain Menunggu'', ''NIB-UJI-2'', ''NPWP-UJI-7'', ''Jl. Uji 7'', ''0210000007'', ''pending_verification'', ''self_registered'')', 1);
SELECT pg_temp.uji_boleh('A20 pengajuan ditolak boleh memakai NIB yang sama (CR-035)', 'rs_api',
  'INSERT INTO app.corporate_client (id, legal_name, nib, npwp, office_address, office_phone, status, origin) VALUES (''00000000-0000-7000-8000-0000000a2028'', ''PT Lain Ditolak'', ''NIB-UJI-2'', ''NPWP-UJI-8'', ''Jl. Uji 8'', ''0210000008'', ''rejected'', ''self_registered'')', 1);
SELECT pg_temp.uji_tolak('A20 NEG perusahaan Aktif kedua dengan NIB sama ditolak (CR-035, US-1903 AC4)', 'rs_api',
  'INSERT INTO app.corporate_client (id, legal_name, nib, npwp, office_address, office_phone, status, origin) VALUES (''00000000-0000-7000-8000-0000000a2023'', ''PT Lain'', ''NIB-UJI-2'', ''NPWP-UJI-3'', ''Jl. Uji 3'', ''0210000003'', ''active'', ''created_by_admin'')',
  '23505', 'uqp_corporate_client_nib_active');
SELECT pg_temp.uji_tolak('A20 NEG persetujuan pengajuan ber-NIB perusahaan Aktif ditolak database (CR-035, 4090116)', 'rs_api',
  'UPDATE app.corporate_client SET status = ''active'' WHERE id = ''00000000-0000-7000-8000-0000000a2027''',
  '23505', 'uqp_corporate_client_nib_active');
-- Review T1 CR-035 (DBA 5): pemilik pertama yang Nonaktif atau sudah dihapus lunak tidak menghalangi perusahaan
-- Aktif kedua dengan NIB yang sama (predikat index: deleted_at IS NULL AND status = 'active').
SELECT pg_temp.uji_boleh('A20 perusahaan Aktif pertama dinonaktifkan (CR-035)', 'rs_api',
  'INSERT INTO app.corporate_client (id, legal_name, nib, npwp, office_address, office_phone, status, origin) VALUES (''00000000-0000-7000-8000-0000000a2024'', ''PT Nonaktif'', ''NIB-UJI-9'', ''NPWP-UJI-4'', ''Jl. Uji 4'', ''0210000004'', ''inactive'', ''created_by_admin'')', 1);
SELECT pg_temp.uji_boleh('A20 perusahaan Aktif kedua boleh memakai NIB pemilik pertama yang Nonaktif (CR-035)', 'rs_api',
  'INSERT INTO app.corporate_client (id, legal_name, nib, npwp, office_address, office_phone, status, origin) VALUES (''00000000-0000-7000-8000-0000000a2025'', ''PT Aktif Pengganti'', ''NIB-UJI-9'', ''NPWP-UJI-5'', ''Jl. Uji 5'', ''0210000005'', ''active'', ''created_by_admin'')', 1);
SELECT pg_temp.uji_boleh('A20 perusahaan Aktif pertama ber-NIB-UJI-10 (CR-035)', 'rs_api',
  'INSERT INTO app.corporate_client (id, legal_name, nib, npwp, office_address, office_phone, status, origin) VALUES (''00000000-0000-7000-8000-0000000a2026'', ''PT Aktif Dihapus'', ''NIB-UJI-10'', ''NPWP-UJI-6'', ''Jl. Uji 6'', ''0210000006'', ''active'', ''created_by_admin'')', 1);
SELECT pg_temp.uji_boleh('A20 perusahaan Aktif pertama dihapus lunak (ADR-003)', 'rs_api',
  'UPDATE app.corporate_client SET deleted_at = now(), deleted_by = ''00000000-0000-7000-8000-0000000a2001'', delete_reason = ''uji CR-035'' WHERE id = ''00000000-0000-7000-8000-0000000a2026''', 1);
SELECT pg_temp.uji_boleh('A20 perusahaan Aktif kedua boleh memakai NIB pemilik Aktif yang sudah dihapus lunak (CR-035)', 'rs_api',
  'INSERT INTO app.corporate_client (id, legal_name, nib, npwp, office_address, office_phone, status, origin) VALUES (''00000000-0000-7000-8000-0000000a2029'', ''PT Aktif Baru'', ''NIB-UJI-10'', ''NPWP-UJI-9'', ''Jl. Uji 9'', ''0210000009'', ''active'', ''created_by_admin'')', 1);
SELECT pg_temp.uji_tolak('A20 NEG status di luar daftar', 'rs_api',
  'UPDATE app.corporate_client SET status = ''verified'' WHERE id = ''00000000-0000-7000-8000-0000000a2021''', '23514', 'ck_corporate_client_status');
SELECT pg_temp.uji_tolak('A20 NEG asal di luar daftar', 'rs_api',
  'UPDATE app.corporate_client SET origin = ''imported'' WHERE id = ''00000000-0000-7000-8000-0000000a2021''', '23514', 'ck_corporate_client_origin');
SELECT pg_temp.uji_tolak('A20 NEG NIB kosong', 'rs_api',
  'UPDATE app.corporate_client SET nib = ''  '' WHERE id = ''00000000-0000-7000-8000-0000000a2021''', '23514', 'ck_corporate_client_nib_(length|trimmed)');
SELECT pg_temp.uji_tolak('A20 NEG NIB dengan spasi tepi tidak lolos sebagai NIB lain (review DBA)', 'rs_api',
  'INSERT INTO app.corporate_client (id, legal_name, nib, npwp, office_address, office_phone, status, origin) VALUES (''00000000-0000-7000-8000-0000000a2023'', ''PT Lain'', '' 123'', ''NPWP-UJI-3'', ''Jl. Uji 3'', ''0210000003'', ''pending_verification'', ''self_registered'')',
  '23514', 'ck_corporate_client_nib_trimmed');
SELECT pg_temp.uji_tolak('A20 NEG NIB dengan spasi akhir', 'rs_api',
  'UPDATE app.corporate_client SET nib = ''NIB-UJI-1 '' WHERE id = ''00000000-0000-7000-8000-0000000a2021''', '23514', 'ck_corporate_client_nib_trimmed');
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
-- account (expand F1-T99) dan corporate_member (M).
SELECT pg_temp.uji_boleh('A1.1 rs_api membuat akun HR corporate_admin di realm korporat (ADR-027 butir 3)', 'rs_api',
  'INSERT INTO app.account (id, account_type, idp_realm, idp_subject, email, status, status_changed_at, full_name) VALUES '
  || '(''00000000-0000-7000-8000-0000000a2051'', ''corporate_admin'', ''korporat'', ''sub-hr-1'', ''hr1@contoh.invalid'', ''pending_activation'', now(), ''HR Sintetis Satu''), '
  || '(''00000000-0000-7000-8000-0000000a2052'', ''corporate_admin'', ''korporat'', ''sub-hr-2'', ''hr2@contoh.invalid'', ''pending_activation'', now(), NULL)', 2);
SELECT pg_temp.uji_tolak('A1.1 NEG corporate_admin di realm admin', 'rs_api',
  'INSERT INTO app.account (id, account_type, idp_realm, idp_subject, email, status, status_changed_at) VALUES (''00000000-0000-7000-8000-0000000a2053'', ''corporate_admin'', ''admin'', ''sub-hr-x'', ''x@contoh.invalid'', ''active'', now())',
  '23514', 'ck_account_account_type_idp_realm');
SELECT pg_temp.uji_tolak('A1.1 NEG corporate_admin di realm pasien', 'rs_api',
  'INSERT INTO app.account (id, account_type, idp_realm, idp_subject, email, status, status_changed_at) VALUES (''00000000-0000-7000-8000-0000000a2053'', ''corporate_admin'', ''pasien'', ''sub-hr-x'', ''x@contoh.invalid'', ''active'', now())',
  '23514', 'ck_account_account_type_idp_realm');
SELECT pg_temp.uji_tolak('A1.1 NEG pasien di realm korporat', 'rs_api',
  'INSERT INTO app.account (id, account_type, idp_realm, idp_subject, email, status, status_changed_at) VALUES (''00000000-0000-7000-8000-0000000a2053'', ''patient'', ''korporat'', ''sub-hr-x'', ''x@contoh.invalid'', ''active'', now())',
  '23514', 'ck_account_account_type_idp_realm');
SELECT pg_temp.uji_tolak('A1.1 NEG admin Ruang Sehat di realm korporat', 'rs_api',
  'INSERT INTO app.account (id, account_type, idp_realm, idp_subject, email, status, status_changed_at, full_name) VALUES (''00000000-0000-7000-8000-0000000a2053'', ''rs_admin'', ''korporat'', ''sub-hr-x'', ''x@contoh.invalid'', ''active'', now(), ''X'')',
  '23514', 'ck_account_account_type_idp_realm');
SELECT pg_temp.uji_tolak('A1.1 NEG jenis akun di luar daftar', 'rs_api',
  'INSERT INTO app.account (id, account_type, idp_realm, idp_subject, email, status, status_changed_at) VALUES (''00000000-0000-7000-8000-0000000a2053'', ''corporate_owner'', ''korporat'', ''sub-hr-x'', ''x@contoh.invalid'', ''active'', now())',
  '23514', 'ck_account_(account_type_idp_realm|account_type)');
SELECT pg_temp.uji_boleh('F1-T34 rs_worker mencatat posisi baca dan tanda terima realm korporat', 'rs_worker',
  'WITH c AS (INSERT INTO app.idp_event_cursor (id, idp_realm, position_ms) VALUES (''00000000-0000-7000-8000-0000000a2071'', ''korporat'', 0) RETURNING id) '
  || 'INSERT INTO app.idp_event_receipt (id, idp_realm, idp_event_id, event_time) SELECT ''00000000-0000-7000-8000-0000000a2072'', ''korporat'', ''0193a1b2-0000-7000-8000-000000000001'', now() FROM c', 1);
SELECT pg_temp.uji_tolak('F1-T34 NEG realm rme tetap ditolak pengambil event Ruang Sehat', 'rs_worker',
  'INSERT INTO app.idp_event_receipt (id, idp_realm, idp_event_id, event_time) VALUES (''00000000-0000-7000-8000-0000000a2073'', ''rme'', ''0193a1b2-0000-7000-8000-000000000002'', now())',
  '23514', 'ck_idp_event_receipt_idp_realm');
SELECT pg_temp.uji_tolak('A1.1 NEG subjek IdP ganda di realm korporat (akun aktif)', 'rs_api',
  'INSERT INTO app.account (id, account_type, idp_realm, idp_subject, email, status, status_changed_at) VALUES (''00000000-0000-7000-8000-0000000a2053'', ''corporate_admin'', ''korporat'', ''sub-hr-1'', ''lain@contoh.invalid'', ''active'', now())',
  '23505', 'uqp_account_idp_realm_idp_subject');

SELECT pg_temp.uji_boleh('A20 rs_api mencatat keanggotaan HR pertama dengan jabatan (US-1901 AC1)', 'rs_api',
  'INSERT INTO app.corporate_member (id, account_id, corporate_client_id, job_title) VALUES (''00000000-0000-7000-8000-0000000a2061'', ''00000000-0000-7000-8000-0000000a2051'', ''00000000-0000-7000-8000-0000000a2021'', ''Manajer SDM'')', 1);
SELECT pg_temp.uji_boleh('A20 rs_api mencatat keanggotaan HR undangan tanpa jabatan (US-1904 AC2)', 'rs_api',
  'INSERT INTO app.corporate_member (id, created_by, account_id, corporate_client_id) VALUES (''00000000-0000-7000-8000-0000000a2062'', ''00000000-0000-7000-8000-0000000a2051'', ''00000000-0000-7000-8000-0000000a2052'', ''00000000-0000-7000-8000-0000000a2021'')', 1);
SELECT pg_temp.uji_tolak('A20 NEG keanggotaan aktif kedua untuk akun yang sama di perusahaan lain (ADR-028 butir 1, M-51)', 'rs_api',
  'INSERT INTO app.corporate_member (id, account_id, corporate_client_id) VALUES (''00000000-0000-7000-8000-0000000a2063'', ''00000000-0000-7000-8000-0000000a2051'', ''00000000-0000-7000-8000-0000000a2022'')',
  '23505', 'uqp_corporate_member_account_id');
SELECT pg_temp.uji_tolak('A20 NEG keanggotaan aktif kedua untuk akun yang sama di perusahaan yang sama', 'rs_api',
  'INSERT INTO app.corporate_member (id, account_id, corporate_client_id) VALUES (''00000000-0000-7000-8000-0000000a2063'', ''00000000-0000-7000-8000-0000000a2051'', ''00000000-0000-7000-8000-0000000a2021'')',
  '23505', 'uqp_corporate_member_account_id');
SELECT pg_temp.uji_tolak('A20 NEG keanggotaan ke perusahaan yang tidak ada', 'rs_api',
  'INSERT INTO app.corporate_member (id, account_id, corporate_client_id) VALUES (''00000000-0000-7000-8000-0000000a2063'', ''00000000-0000-7000-8000-0000000a2001'', ''00000000-0000-7000-8000-0000000affff'')',
  '23503', 'fk_corporate_member_corporate_client_id');
SELECT pg_temp.uji_tolak('A20 NEG keanggotaan untuk akun yang tidak ada', 'rs_api',
  'INSERT INTO app.corporate_member (id, account_id, corporate_client_id) VALUES (''00000000-0000-7000-8000-0000000a2063'', ''00000000-0000-7000-8000-0000000affff'', ''00000000-0000-7000-8000-0000000a2021'')',
  '23503', 'fk_corporate_member_account_id');
SELECT pg_temp.uji_tolak('A20 NEG jabatan kosong', 'rs_api',
  'UPDATE app.corporate_member SET job_title = '' '' WHERE id = ''00000000-0000-7000-8000-0000000a2061''', '23514', 'ck_corporate_member_job_title_length');
SELECT pg_temp.uji_tolak('A20 NEG pencabutan keanggotaan tanpa alasan', 'rs_api',
  'UPDATE app.corporate_member SET deleted_at = now() WHERE id = ''00000000-0000-7000-8000-0000000a2062''', '23514', 'ck_corporate_member_soft_delete');
SELECT pg_temp.uji_boleh('A20 perpindahan keanggotaan: baris lama dihapus lunak lalu keanggotaan baru di perusahaan lain (M-51)', 'rs_api',
  'WITH lama AS (UPDATE app.corporate_member SET deleted_at = now(), deleted_by = ''00000000-0000-7000-8000-0000000a2051'', delete_reason = ''pindah'', updated_at = now() WHERE id = ''00000000-0000-7000-8000-0000000a2062'' RETURNING account_id) '
  || 'INSERT INTO app.corporate_member (id, account_id, corporate_client_id) SELECT ''00000000-0000-7000-8000-0000000a2063'', account_id, ''00000000-0000-7000-8000-0000000a2022'' FROM lama', 1);
SELECT pg_temp.uji_tolak('A20 NEG rs_api DELETE keanggotaan (Master, ADR-003)', 'rs_api',
  'DELETE FROM app.corporate_member', '42501', 'permission denied for table corporate_member');
SELECT pg_temp.uji_tolak('A20 NEG rs_api TRUNCATE keanggotaan', 'rs_api',
  'TRUNCATE app.corporate_member', '42501', 'permission denied for table corporate_member');
SELECT pg_temp.uji_boleh('A20 pekerja privasi menganonimkan jabatan', 'rs_privacy_worker_login',
  'UPDATE app.corporate_member SET job_title = NULL, anonymized_at = now(), updated_at = now(), updated_by = NULL WHERE id = ''00000000-0000-7000-8000-0000000a2061''', 1);
SELECT pg_temp.uji_tolak('A20 NEG pekerja privasi memindahkan keanggotaan (corporate_client_id)', 'rs_privacy_worker_login',
  'UPDATE app.corporate_member SET corporate_client_id = ''00000000-0000-7000-8000-0000000a2022'' WHERE id = ''00000000-0000-7000-8000-0000000a2061''', '42501', 'permission denied for table corporate_member');
SELECT pg_temp.uji_tolak('A20 NEG pekerja privasi mencabut keanggotaan (deleted_at)', 'rs_privacy_worker_login',
  'UPDATE app.corporate_member SET deleted_at = now(), delete_reason = ''x'' WHERE id = ''00000000-0000-7000-8000-0000000a2061''', '42501', 'permission denied for table corporate_member');
SELECT pg_temp.uji_tolak('A20 NEG relay audit membaca keanggotaan', 'rs_audit_relay_login',
  'SELECT id FROM app.corporate_member', '42501', 'permission denied for table corporate_member');
SELECT pg_temp.uji_tolak('A20 NEG rotasi kunci membaca keanggotaan', 'rs_key_rotation',
  'SELECT id FROM app.corporate_member', '42501', 'permission denied for table corporate_member');
SELECT pg_temp.uji_tolak('A20 NEG perusahaan yang punya keputusan tidak dapat dihapus keras walau oleh pemilik (FK NO ACTION)', 'rs_migration',
  'DELETE FROM app.corporate_client WHERE id = ''00000000-0000-7000-8000-0000000a2021''', '23503', 'fk_');
ROLLBACK;

DO $$
BEGIN
  IF EXISTS (SELECT 1 FROM app.corporate_client) OR EXISTS (SELECT 1 FROM app.corporate_client_document)
     OR EXISTS (SELECT 1 FROM app.corporate_verification_decision) OR EXISTS (SELECT 1 FROM app.corporate_member)
     OR EXISTS (SELECT 1 FROM app.stored_file) OR EXISTS (SELECT 1 FROM app.idp_event_cursor) OR EXISTS (SELECT 1 FROM app.idp_event_receipt)
     OR EXISTS (SELECT 1 FROM app.account) THEN
    RAISE EXCEPTION 'GAGAL pembersihan tabel-app-a20: data uji tersisa';
  END IF;
  RAISE NOTICE 'LULUS pembersihan tabel-app-a20: tidak ada data uji tersisa (ROLLBACK)';
END $$;
