-- db/test/tabel-audit-b.sql: test tabel database audit B1-B6 (F1-T21), dijalankan di
-- ruang_sehat_audit dan rme_audit. Didahului db/test/lib.sql. Variabel psql: prefiks (rs_|rme_).
-- Aturan kelas A generik (T-06 hak per tabel, T-08 trigger per tabel dan partisi, T-18 index
-- partisi) di db/test/kelas-audit.sql bagian A. Berkas ini memeriksa struktur khusus (kolom persis
-- DD B, CHECK, index, partisi 2026-11 s.d. 2029-12 tanpa DEFAULT, fn_chain_head, RLS tenant
-- rme_audit) dan perilaku dengan role sungguhan (T-09 bagian audit, T-19, RLS §5.6), positif dan
-- negatif. Satu transaksi yang di-ROLLBACK. Data sintetis dengan recorded_at tetap (2027), sehingga
-- test tidak bergantung pada tanggal hari ini (§6.2 butir 2).
-- Dasar: docs/04-data-dictionary.md §0.6 B, B1-B6; docs/05-physical-schema.md §4.11, §5.6, §6.1-§6.4,
-- §12 (T-06, T-08, T-09, T-18, T-19); ADR-005, ADR-009 butir 3-5, 8; ADR-012 butir 9.

SELECT pg_catalog.set_config('uji.prefiks', :'prefiks', false) AS _p \gset
SELECT :'prefiks' || 'audit_migration' AS pemilik, :'prefiks' || 'audit_writer_login' AS penulis,
       :'prefiks' || 'audit_verifier_login' AS pemverifikasi, :'prefiks' || 'audit_reader' AS pembaca \gset
SET TimeZone = 'UTC';

-- 1. Struktur.
SELECT pg_temp.uji_nol('B tabel skema audit persis enam tabel DD B (+ partisi) dan audit_detached kosong',
  ARRAY(SELECT format('tabel tak terduga %s.%s', n.nspname, c.relname)
          FROM pg_class c JOIN pg_namespace n ON n.oid = c.relnamespace
         WHERE n.nspname IN ('audit', 'audit_detached') AND c.relkind IN ('r', 'p') AND NOT c.relispartition
           AND c.relname <> '_sqlx_migrations'
           AND (n.nspname = 'audit_detached' OR c.relname NOT IN ('audit_event', 'audit_event_subject', 'audit_event_key',
                                                                  'audit_chain_anchor', 'audit_chain_verification', 'audit_archive_manifest'))
        UNION ALL
        SELECT format('tabel tidak ada: %s', t) FROM unnest(ARRAY['audit_event', 'audit_event_subject', 'audit_event_key',
                 'audit_chain_anchor', 'audit_chain_verification', 'audit_archive_manifest']) t
         WHERE to_regclass('audit.' || t) IS NULL));
SELECT pg_temp.uji_nol('B kolom persis DD B1-B6 (nama, tipe, NOT NULL, urutan; created_at STD-L DEFAULT now(); tanpa nilai bawaan lain)',
  ARRAY(WITH harap(t, k) AS (VALUES
      ('audit_event', 'event_id uuid NN|recorded_at timestamp with time zone NN|seq bigint NN|occurred_at timestamp with time zone NN|'
        || 'event_code text NN|action_name text NN|actor_type text NN|actor_account_id uuid|actor_account_type text|actor_roles text[]|'
        || 'actor_partner_id uuid|object_type text NN|object_id text|object_version integer|subject_id uuid|subject_count integer NN|'
        || 'tenant_id uuid|source_component text NN|source_ip inet|user_agent text|app_version text|changed_fields text[]|'
        || 'before_values jsonb|after_values jsonb|disclosure_recipient text|disclosure_data_categories text[]|outcome text NN|'
        || 'outcome_reason_code text|correlation_id uuid NN|canonical_version smallint NN|prev_hash bytea NN|event_hash bytea NN'),
      ('audit_event_key', 'event_id uuid NN|seq bigint NN|recorded_at timestamp with time zone NN'),
      ('audit_event_subject', 'id uuid NN|recorded_at timestamp with time zone NN|event_id uuid NN|subject_id uuid NN|subject_kind text NN'),
      ('audit_chain_anchor', 'id uuid NN|created_at timestamp with time zone NN DEFAULT now()|seq bigint NN|event_hash bytea NN|'
        || 'anchored_at timestamp with time zone NN|worm_object_key text NN'),
      ('audit_chain_verification', 'id uuid NN|created_at timestamp with time zone NN DEFAULT now()|started_at timestamp with time zone NN|'
        || 'finished_at timestamp with time zone NN|from_seq bigint NN|to_seq bigint NN|audit_chain_anchor_id uuid|outcome text NN|'
        || 'first_mismatch_seq bigint'),
      ('audit_archive_manifest', 'id uuid NN|created_at timestamp with time zone NN DEFAULT now()|partition_name text NN|'
        || 'range_start timestamp with time zone NN|range_end timestamp with time zone NN|from_seq bigint NN|to_seq bigint NN|'
        || 'boundary_event_hash bytea NN|archive_object_key text NN|archive_sha256 bytea NN|exported_at timestamp with time zone NN|'
        || 'detached_at timestamp with time zone NN'))
    SELECT format('%s: %s', h.t, a.k) FROM harap h,
           LATERAL (SELECT string_agg(att.attname || ' ' || format_type(att.atttypid, att.atttypmod)
                                      || CASE WHEN att.attnotnull THEN ' NN' ELSE '' END
                                      || coalesce(' DEFAULT ' || pg_get_expr(d.adbin, d.adrelid), ''), '|' ORDER BY att.attnum) AS k
                      FROM pg_attribute att LEFT JOIN pg_attrdef d ON d.adrelid = att.attrelid AND d.adnum = att.attnum
                     WHERE att.attrelid = to_regclass('audit.' || h.t) AND att.attnum > 0 AND NOT att.attisdropped) a
     WHERE a.k IS DISTINCT FROM h.k));
SELECT pg_temp.uji_nol('B CHECK persis (event_code e_01-e_17, actor_type, subject_count, outcome, hash 32 byte, seq >= 1, subject_kind, outcome verifikasi)',
  ARRAY(WITH harap(t, n, def) AS (VALUES
      ('audit_event', 'ck_audit_event_event_code', 'CHECK ((event_code = ANY (ARRAY[''e_01''::text, ''e_02''::text, ''e_03''::text, ''e_04''::text, ''e_05''::text, ''e_06''::text, ''e_07''::text, ''e_08''::text, ''e_09''::text, ''e_10''::text, ''e_11''::text, ''e_12''::text, ''e_13''::text, ''e_14''::text, ''e_15''::text, ''e_16''::text, ''e_17''::text])))'),
      ('audit_event', 'ck_audit_event_actor_type', 'CHECK ((actor_type = ANY (ARRAY[''account''::text, ''system''::text, ''partner_system''::text, ''ruang_sehat_system''::text])))'),
      ('audit_event', 'ck_audit_event_subject_count', 'CHECK ((subject_count >= 0))'),
      ('audit_event', 'ck_audit_event_outcome', 'CHECK ((outcome = ANY (ARRAY[''success''::text, ''failure''::text])))'),
      ('audit_event', 'ck_audit_event_prev_hash', 'CHECK ((octet_length(prev_hash) = 32))'),
      ('audit_event', 'ck_audit_event_event_hash', 'CHECK ((octet_length(event_hash) = 32))'),
      ('audit_event_key', 'ck_audit_event_key_seq', 'CHECK ((seq >= 1))'),
      ('audit_event_subject', 'ck_audit_event_subject_subject_kind', 'CHECK ((subject_kind = ANY (ARRAY[''care_recipient''::text, ''rme_patient''::text])))'),
      ('audit_chain_anchor', 'ck_audit_chain_anchor_event_hash', 'CHECK ((octet_length(event_hash) = 32))'),
      ('audit_chain_verification', 'ck_audit_chain_verification_outcome', 'CHECK ((outcome = ANY (ARRAY[''match''::text, ''mismatch''::text, ''error''::text])))'),
      ('audit_archive_manifest', 'ck_audit_archive_manifest_boundary_event_hash', 'CHECK ((octet_length(boundary_event_hash) = 32))'),
      ('audit_archive_manifest', 'ck_audit_archive_manifest_archive_sha256', 'CHECK ((octet_length(archive_sha256) = 32))')),
    aktual AS (SELECT c.relname::text AS t, k.conname::text AS n, pg_get_constraintdef(k.oid) AS def
                 FROM pg_constraint k JOIN pg_class c ON c.oid = k.conrelid JOIN pg_namespace s ON s.oid = c.relnamespace
                WHERE s.nspname = 'audit' AND k.contype = 'c' AND NOT c.relispartition)
    SELECT format('kurang/beda %s.%s', h.t, h.n) FROM harap h WHERE (h.t, h.n, h.def) NOT IN (SELECT t, n, def FROM aktual)
    UNION ALL
    SELECT format('lebih %s.%s: %s', a.t, a.n, a.def) FROM aktual a WHERE (a.t, a.n) NOT IN (SELECT t, n FROM harap)));
SELECT pg_temp.uji_nol('B1 audit_event: index persis PK komposit + sembilan index DD B1 di induk',
  ARRAY(SELECT pg_temp.uji_indeks('audit.audit_event', ARRAY[
    'CREATE UNIQUE INDEX pk_audit_event ON ONLY audit.audit_event USING btree (event_id, recorded_at)',
    'CREATE INDEX ix_audit_event_recorded_at ON ONLY audit.audit_event USING btree (recorded_at DESC)',
    'CREATE INDEX ix_audit_event_actor_account_id_recorded_at ON ONLY audit.audit_event USING btree (actor_account_id, recorded_at)',
    'CREATE INDEX ix_audit_event_subject_id_recorded_at ON ONLY audit.audit_event USING btree (subject_id, recorded_at)',
    'CREATE INDEX ix_audit_event_event_code_recorded_at ON ONLY audit.audit_event USING btree (event_code, recorded_at)',
    'CREATE INDEX ix_audit_event_object_type_object_id ON ONLY audit.audit_event USING btree (object_type, object_id)',
    'CREATE INDEX ix_audit_event_tenant_id_event_code_recorded_at ON ONLY audit.audit_event USING btree (tenant_id, event_code, recorded_at)',
    'CREATE INDEX ix_audit_event_source_ip_recorded_at ON ONLY audit.audit_event USING btree (source_ip, recorded_at)',
    'CREATE INDEX ix_audit_event_correlation_id ON ONLY audit.audit_event USING btree (correlation_id)',
    'CREATE INDEX ix_audit_event_seq ON ONLY audit.audit_event USING btree (seq)'])));
SELECT pg_temp.uji_nol('B3 audit_event_subject: index persis PK komposit, ix (subject_id, recorded_at), ix event_id',
  ARRAY(SELECT pg_temp.uji_indeks('audit.audit_event_subject', ARRAY[
    'CREATE UNIQUE INDEX pk_audit_event_subject ON ONLY audit.audit_event_subject USING btree (id, recorded_at)',
    'CREATE INDEX ix_audit_event_subject_subject_id_recorded_at ON ONLY audit.audit_event_subject USING btree (subject_id, recorded_at)',
    'CREATE INDEX ix_audit_event_subject_event_id ON ONLY audit.audit_event_subject USING btree (event_id)'])));
SELECT pg_temp.uji_nol('B2/B4/B5/B6 index persis (PK, UQ seq kunci dan jangkar, UQ partition_name manifest)',
  ARRAY(SELECT pg_temp.uji_indeks('audit.audit_event_key', ARRAY[
          'CREATE UNIQUE INDEX pk_audit_event_key ON audit.audit_event_key USING btree (event_id)',
          'CREATE UNIQUE INDEX uq_audit_event_key_seq ON audit.audit_event_key USING btree (seq)'])
        UNION ALL
        SELECT pg_temp.uji_indeks('audit.audit_chain_anchor', ARRAY[
          'CREATE UNIQUE INDEX pk_audit_chain_anchor ON audit.audit_chain_anchor USING btree (id)',
          'CREATE UNIQUE INDEX uq_audit_chain_anchor_seq ON audit.audit_chain_anchor USING btree (seq)'])
        UNION ALL
        SELECT pg_temp.uji_indeks('audit.audit_chain_verification', ARRAY[
          'CREATE UNIQUE INDEX pk_audit_chain_verification ON audit.audit_chain_verification USING btree (id)'])
        UNION ALL
        SELECT pg_temp.uji_indeks('audit.audit_archive_manifest', ARRAY[
          'CREATE UNIQUE INDEX pk_audit_archive_manifest ON audit.audit_archive_manifest USING btree (id)',
          'CREATE UNIQUE INDEX uq_audit_archive_manifest_partition_name ON audit.audit_archive_manifest USING btree (partition_name)'])));
SELECT pg_temp.uji_nol('B FK persis: hanya audit_chain_verification -> audit_chain_anchor (B3 tanpa FK ke audit_event, DD B3)',
  ARRAY(SELECT format('%s.%s: %s', c.relname, k.conname, pg_get_constraintdef(k.oid))
          FROM pg_constraint k JOIN pg_class c ON c.oid = k.conrelid JOIN pg_namespace s ON s.oid = c.relnamespace
         WHERE s.nspname = 'audit' AND k.contype = 'f'
           AND (k.conname, pg_get_constraintdef(k.oid)) IS DISTINCT FROM
               ('fk_audit_chain_verification_audit_chain_anchor_id', 'FOREIGN KEY (audit_chain_anchor_id) REFERENCES audit.audit_chain_anchor(id)')
        UNION ALL
        SELECT 'fk_audit_chain_verification_audit_chain_anchor_id tidak ada'
         WHERE NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conname = 'fk_audit_chain_verification_audit_chain_anchor_id'
                              AND conrelid = 'audit.audit_chain_verification'::regclass)));
-- Partisi (§6.1, §6.2): RANGE (recorded_at), 38 bulan 2026-11 s.d. 2029-12 per induk, batas UTC,
-- tanpa DEFAULT, nama <induk>_yYYYYmMM.
SELECT pg_temp.uji_nol('B partisi: RANGE (recorded_at), persis 38 partisi bulanan 2026-11 s.d. 2029-12 per induk (batas 00:00:00+00), tanpa DEFAULT',
  ARRAY(WITH induk(t) AS (VALUES ('audit_event'), ('audit_event_subject')),
    harap AS (SELECT i.t, i.t || '_' || to_char(m, '"y"YYYY"m"MM') AS nama,
                     format('FOR VALUES FROM (%L) TO (%L)', to_char(m, 'YYYY-MM-DD') || ' 00:00:00+00',
                            to_char(m + interval '1 month', 'YYYY-MM-DD') || ' 00:00:00+00') AS batas
                FROM induk i, generate_series(date '2026-11-01', date '2029-12-01', interval '1 month') m),
    aktual AS (SELECT p.relname::text AS t, c.relname::text AS nama, pg_get_expr(c.relpartbound, c.oid) AS batas
                 FROM pg_inherits h JOIN pg_class c ON c.oid = h.inhrelid JOIN pg_class p ON p.oid = h.inhparent
                 JOIN pg_namespace s ON s.oid = p.relnamespace
                WHERE s.nspname = 'audit' AND p.relname IN ('audit_event', 'audit_event_subject'))
    SELECT format('partisi kurang/beda %s %s', h.nama, h.batas) FROM harap h WHERE (h.t, h.nama, h.batas) NOT IN (SELECT t, nama, batas FROM aktual)
    UNION ALL
    SELECT format('partisi lebih %s %s', a.nama, a.batas) FROM aktual a WHERE (a.t, a.nama, a.batas) NOT IN (SELECT t, nama, batas FROM harap)
    UNION ALL
    SELECT format('kunci partisi %s: %s', t, coalesce(pg_get_partkeydef(to_regclass('audit.' || t)), '<bukan terpartisi>'))
      FROM induk WHERE pg_get_partkeydef(to_regclass('audit.' || t)) IS DISTINCT FROM 'RANGE (recorded_at)'
    UNION ALL
    SELECT format('partisi DEFAULT %s', c.relname) FROM pg_partitioned_table pt JOIN pg_class c ON c.oid = pt.partdefid
     WHERE pt.partdefid <> 0));
SELECT pg_temp.uji_nol('B trigger induk persis: reject_ud_row (baris) + reject_ud + reject_truncate di induk terpartisi; reject_ud + reject_truncate di tabel biasa',
  ARRAY(WITH harap(t, g) AS (
      SELECT t, 'tg_' || t || s FROM unnest(ARRAY['audit_event', 'audit_event_subject']) t, unnest(ARRAY['_reject_ud_row', '_reject_ud', '_reject_truncate']) s
      UNION ALL
      SELECT t, 'tg_' || t || s FROM unnest(ARRAY['audit_event_key', 'audit_chain_anchor', 'audit_chain_verification', 'audit_archive_manifest']) t,
             unnest(ARRAY['_reject_ud', '_reject_truncate']) s),
    aktual AS (SELECT c.relname::text AS t, g.tgname::text AS g
                 FROM pg_trigger g JOIN pg_class c ON c.oid = g.tgrelid JOIN pg_namespace s ON s.oid = c.relnamespace
                WHERE s.nspname = 'audit' AND NOT c.relispartition AND NOT g.tgisinternal)
    SELECT 'kurang ' || t || '.' || g FROM (SELECT * FROM harap EXCEPT SELECT * FROM aktual) x
    UNION ALL
    SELECT 'lebih ' || t || '.' || g FROM (SELECT * FROM aktual EXCEPT SELECT * FROM harap) y));
SELECT pg_temp.uji_nol('§6.3 audit.fn_chain_head: sql STABLE SECURITY DEFINER, search_path persis pg_catalog, audit, milik pemilik skema, EXECUTE hanya *_audit_writer; satu-satunya rutin skema audit',
  ARRAY(SELECT 'fn_chain_head tidak sesuai' WHERE NOT EXISTS (
          SELECT 1 FROM pg_proc p
           WHERE p.oid = to_regprocedure('audit.fn_chain_head()')
             AND p.prosecdef AND p.provolatile = 's' AND p.proretset
             AND p.prolang = (SELECT oid FROM pg_language WHERE lanname = 'sql')
             AND p.proconfig = ARRAY['search_path=pg_catalog, audit']
             AND pg_get_function_result(p.oid) = 'TABLE(seq bigint, event_hash bytea, recorded_at timestamp with time zone)'
             AND p.proowner = (current_setting('uji.prefiks') || 'audit_migration')::regrole
             AND (SELECT string_agg(pg_get_userbyid(a.grantee) || ':' || a.privilege_type, ',')
                    FROM aclexplode(p.proacl) a WHERE a.grantee <> p.proowner) = current_setting('uji.prefiks') || 'audit_writer:EXECUTE')
        UNION ALL
        SELECT format('rutin lain di skema audit: %s', p.oid::regprocedure)
          FROM pg_proc p JOIN pg_namespace n ON n.oid = p.pronamespace
         WHERE n.nspname = 'audit' AND p.oid IS DISTINCT FROM to_regprocedure('audit.fn_chain_head()')));
-- RLS (§5.6, KEPUTUSAN-99): hanya rme_audit, hanya audit_event dan audit_event_subject, ENABLE + FORCE,
-- kebijakan per role (F-P02). ruang_sehat_audit tanpa RLS dan tanpa kebijakan.
SELECT pg_temp.uji_nol(format('§5.6 RLS %s: %s', current_database(),
         CASE WHEN current_setting('uji.prefiks') = 'rme_' THEN 'ENABLE + FORCE di audit_event dan audit_event_subject saja, kebijakan persis tenant_reader/writer/reader+verifier/migration'
              ELSE 'tidak aktif di tabel mana pun dan tanpa kebijakan' END),
  ARRAY(WITH harap_rls(t) AS (SELECT t FROM unnest(ARRAY['audit_event', 'audit_event_subject']) t WHERE current_setting('uji.prefiks') = 'rme_'),
    harap_pol(t, n, cmd, roles, qual_true, check_true) AS (
      SELECT t, 'pol_' || t || s, cmd, roles, qual_true, check_true
        FROM unnest(ARRAY['audit_event', 'audit_event_subject']) t,
             (VALUES ('_tenant_reader', 'SELECT', '{rme_audit_tenant_reader}', false, NULL::boolean),
                     ('_writer', 'INSERT', '{rme_audit_writer}', NULL, true),
                     ('_reader', 'SELECT', '{rme_audit_reader,rme_audit_verifier}', true, NULL),
                     ('_migration', 'ALL', '{rme_audit_migration}', true, true)) v(s, cmd, roles, qual_true, check_true)
       WHERE current_setting('uji.prefiks') = 'rme_'),
    aktual_pol AS (SELECT tablename::text AS t, policyname::text AS n, cmd::text AS cmd,
                          (SELECT array_agg(r ORDER BY r) FROM unnest(roles) r)::text AS roles,
                          CASE WHEN qual IS NULL THEN NULL ELSE qual = 'true' END AS qual_true,
                          CASE WHEN with_check IS NULL THEN NULL ELSE with_check = 'true' END AS check_true
                     FROM pg_policies WHERE schemaname = 'audit' AND permissive = 'PERMISSIVE')
    SELECT format('RLS %s.%s: enable=%s force=%s', n.nspname, c.relname, c.relrowsecurity, c.relforcerowsecurity)
      FROM pg_class c JOIN pg_namespace n ON n.oid = c.relnamespace
     WHERE n.nspname IN ('audit', 'audit_detached') AND c.relkind IN ('r', 'p')
       AND (c.relrowsecurity OR c.relforcerowsecurity) IS DISTINCT FROM (NOT c.relispartition AND n.nspname = 'audit' AND c.relname IN (SELECT t FROM harap_rls))
    UNION ALL
    SELECT format('RLS %s tidak FORCE', t) FROM harap_rls WHERE NOT (SELECT relforcerowsecurity AND relrowsecurity FROM pg_class WHERE oid = to_regclass('audit.' || t))
    UNION ALL
    SELECT 'kebijakan kurang/beda ' || x::text FROM (SELECT t, n, cmd, roles, qual_true, check_true FROM harap_pol
                                                      EXCEPT SELECT * FROM aktual_pol) x
    UNION ALL
    SELECT 'kebijakan lebih ' || y::text FROM (SELECT * FROM aktual_pol EXCEPT SELECT t, n, cmd, roles, qual_true, check_true FROM harap_pol) y
    UNION ALL
    SELECT 'kebijakan RESTRICTIVE ' || policyname FROM pg_policies WHERE schemaname = 'audit' AND permissive <> 'PERMISSIVE'));

-- 2. Negatif atas pemeriksa (kerusakan di skema nyata, dibatalkan per kasus), lalu perilaku. Semua di-ROLLBACK.
BEGIN;
SET LOCAL TimeZone = 'UTC';
DO $$
DECLARE
  c record;
  p text := current_setting('uji.prefiks');
  kelas text := format('SELECT pg_temp.uji_kelas_audit(''audit'', %L)', current_setting('uji.prefiks'));
  partisi text := $q$SELECT format('%s', c.relname) FROM pg_partitioned_table pt JOIN pg_class c ON c.oid = pt.partdefid WHERE pt.partdefid <> 0
                     UNION ALL SELECT 'jumlah ' || count(*) FROM pg_inherits h WHERE h.inhparent = 'audit.audit_event'::regclass HAVING count(*) <> 38$q$;
  rutin text := $q$SELECT 'fn_chain_head berubah' WHERE NOT EXISTS (SELECT 1 FROM pg_proc p WHERE p.oid = to_regprocedure('audit.fn_chain_head()') AND p.prosecdef AND p.proconfig = ARRAY['search_path=pg_catalog, audit'] AND (SELECT count(*) FROM aclexplode(p.proacl) a WHERE a.grantee <> p.proowner) = 1)$q$;
  rls text := $q$SELECT c.relname::text FROM pg_class c WHERE c.oid IN ('audit.audit_event'::regclass, 'audit.audit_event_subject'::regclass) AND NOT (c.relrowsecurity AND c.relforcerowsecurity)
                 UNION ALL SELECT 'kebijakan ' || count(*) FROM pg_policies WHERE schemaname = 'audit' HAVING count(*) <> 8$q$;
BEGIN
  FOR c IN SELECT * FROM (VALUES
      ('T-06 UPDATE audit_event_key untuk writer', format('GRANT UPDATE ON audit.audit_event_key TO %I', p || 'audit_writer'), kelas, 'T-06 audit_event_key: hak tabel lebih ' || p || 'audit_writer:UPDATE'),
      ('T-06 SELECT audit_event_key untuk writer (kepala rantai hanya lewat fn_chain_head)', format('GRANT SELECT ON audit.audit_event_key TO %I', p || 'audit_writer'), kelas, 'lebih ' || p || 'audit_writer:SELECT'),
      ('T-06 INSERT manifest arsip untuk verifier', format('GRANT INSERT ON audit.audit_archive_manifest TO %I', p || 'audit_verifier'), kelas, 'audit_archive_manifest: hak tabel lebih ' || p || 'audit_verifier:INSERT'),
      ('T-06 INSERT jangkar dicabut dari writer', format('REVOKE INSERT ON audit.audit_chain_anchor FROM %I', p || 'audit_writer'), kelas, 'audit_chain_anchor: hak tabel kurang ' || p || 'audit_writer:INSERT'),
      ('T-06 hak langsung di partisi subjek', format('GRANT SELECT ON audit.audit_event_subject_y2028m01 TO %I', p || 'audit_reader'), kelas, 'T-06 audit_event_subject_y2028m01: hak tabel lebih'),
      ('T-08 penolak statement partisi 2029-12 hilang', 'DROP TRIGGER tg_audit_event_y2029m12_reject_truncate ON audit.audit_event_y2029m12', kelas, 'T-08 audit_event_y2029m12'),
      ('T-08 penolak baris induk subjek hilang', 'DROP TRIGGER tg_audit_event_subject_reject_ud_row ON audit.audit_event_subject', kelas, 'T-08 audit_event_subject'),
      ('T-08 penolak kunci event nonaktif', 'ALTER TABLE audit.audit_event_key DISABLE TRIGGER tg_audit_event_key_reject_ud', kelas, 'NONAKTIF'),
      ('T-18 index induk tidak ada di partisi', 'CREATE INDEX ix_audit_event_app_version ON ONLY audit.audit_event (app_version)', kelas, 'T-18 audit_event_y20[0-9]{2}m[0-9]{2}: index induk'),
      ('§6.2 partisi DEFAULT ditambahkan', 'CREATE TABLE audit.audit_event_default PARTITION OF audit.audit_event DEFAULT', partisi, 'audit_event_default'),
      ('§6.2 partisi dilepas', 'ALTER TABLE audit.audit_event DETACH PARTITION audit.audit_event_y2027m01', partisi, 'jumlah 37'),
      ('§6.3 EXECUTE fn_chain_head untuk reader', format('GRANT EXECUTE ON FUNCTION audit.fn_chain_head() TO %I', p || 'audit_reader'), rutin, 'fn_chain_head berubah'),
      ('§6.3 fn_chain_head kehilangan SECURITY DEFINER', 'ALTER FUNCTION audit.fn_chain_head() SECURITY INVOKER', rutin, 'fn_chain_head berubah'),
      ('CR-020 fn_chain_head dengan search_path selain pg_catalog, audit', 'ALTER FUNCTION audit.fn_chain_head() SET search_path = pg_catalog, pg_temp',
       $q$SELECT pg_temp.uji_rutin('{audit,util}')$q$, 'fn_chain_head\(\): SECURITY DEFINER tanpa SET search_path persis'),
      ('CR-020 fungsi SECURITY DEFINER lain di skema audit', 'CREATE FUNCTION audit.fn_uji_definer() RETURNS int LANGUAGE sql SECURITY DEFINER SET search_path = pg_catalog, audit AS ''SELECT 1''',
       $q$SELECT pg_temp.uji_rutin('{audit,util}')$q$, 'fn_uji_definer\(\): SECURITY DEFINER tidak diizinkan')
    ) AS t(label, stmt, pemeriksa, pola)
  LOOP
    PERFORM pg_temp.uji_rusak(c.label, c.stmt, c.pemeriksa, c.pola);
  END LOOP;
  IF p = 'rme_' THEN
    PERFORM pg_temp.uji_rusak('§5.6 FORCE RLS dicabut', 'ALTER TABLE audit.audit_event NO FORCE ROW LEVEL SECURITY', rls, 'audit_event');
    PERFORM pg_temp.uji_rusak('§5.6 RLS subjek dimatikan', 'ALTER TABLE audit.audit_event_subject DISABLE ROW LEVEL SECURITY', rls, 'audit_event_subject');
    PERFORM pg_temp.uji_rusak('§5.6 kebijakan tenant dihapus', 'DROP POLICY pol_audit_event_tenant_reader ON audit.audit_event', rls, 'kebijakan 7');
  END IF;
END $$;

-- Data sintetis. Event: E1 (tenant A, e_05, 2027-03), E2 (tenant A, e_06, 2027-03), E3 (tenant B,
-- e_05, 2027-03), E4 (tenant A, e_05, tepat batas 2027-04-01 00:00 UTC -> partisi 2027-04).
SELECT pg_temp.uji_boleh('T-19 writer memanggil fn_chain_head pada rantai kosong (nol baris)', :'penulis', 'SELECT * FROM audit.fn_chain_head()', 0);
SELECT pg_temp.uji_boleh('T-19 writer INSERT audit_event_key empat event (seq 1-4)', :'penulis',
  $s$INSERT INTO audit.audit_event_key (event_id, seq, recorded_at) VALUES
     ('00000000-0000-7000-8000-00000000b101', 1, '2027-03-15 01:00:00+00'),
     ('00000000-0000-7000-8000-00000000b102', 2, '2027-03-15 02:00:00+00'),
     ('00000000-0000-7000-8000-00000000b103', 3, '2027-03-15 03:00:00+00'),
     ('00000000-0000-7000-8000-00000000b104', 4, '2027-04-01 00:00:00+00')$s$, 4);
SELECT pg_temp.uji_boleh('T-09 writer INSERT audit_event lewat induk (empat event, dua partisi)', :'penulis',
  $s$INSERT INTO audit.audit_event (event_id, recorded_at, seq, occurred_at, event_code, action_name, actor_type, object_type,
       subject_id, subject_count, tenant_id, source_component, outcome, correlation_id, canonical_version, prev_hash, event_hash) VALUES
     ('00000000-0000-7000-8000-00000000b101', '2027-03-15 01:00:00+00', 1, '2027-03-15 00:59:59+00', 'e_05', 'read_record', 'account', 'rme_medical_record',
      NULL, 2, '00000000-0000-7000-8000-00000000a00a', 'rme', 'success', '00000000-0000-7000-8000-00000000c101', 1, decode(repeat('00', 32), 'hex'), decode(repeat('11', 32), 'hex')),
     ('00000000-0000-7000-8000-00000000b102', '2027-03-15 02:00:00+00', 2, '2027-03-15 01:59:59+00', 'e_06', 'update_record', 'account', 'rme_medical_record',
      '00000000-0000-7000-8000-00000000d001', 1, '00000000-0000-7000-8000-00000000a00a', 'rme', 'success', '00000000-0000-7000-8000-00000000c102', 1, decode(repeat('11', 32), 'hex'), decode(repeat('22', 32), 'hex')),
     ('00000000-0000-7000-8000-00000000b103', '2027-03-15 03:00:00+00', 3, '2027-03-15 02:59:59+00', 'e_05', 'read_record', 'account', 'rme_medical_record',
      '00000000-0000-7000-8000-00000000d002', 1, '00000000-0000-7000-8000-00000000a00b', 'rme', 'success', '00000000-0000-7000-8000-00000000c103', 1, decode(repeat('22', 32), 'hex'), decode(repeat('33', 32), 'hex')),
     ('00000000-0000-7000-8000-00000000b104', '2027-04-01 00:00:00+00', 4, '2027-03-31 23:59:59+00', 'e_05', 'read_record', 'account', 'rme_medical_record',
      '00000000-0000-7000-8000-00000000d001', 1, '00000000-0000-7000-8000-00000000a00a', 'rme', 'success', '00000000-0000-7000-8000-00000000c104', 1, decode(repeat('33', 32), 'hex'), decode(repeat('44', 32), 'hex'))$s$, 4);
SELECT pg_temp.uji_boleh('T-09 writer INSERT audit_event_subject (subjek ganda E1, subjek E2, E3)', :'penulis',
  $s$INSERT INTO audit.audit_event_subject (id, recorded_at, event_id, subject_id, subject_kind) VALUES
     ('00000000-0000-7000-8000-00000000e101', '2027-03-15 01:00:00+00', '00000000-0000-7000-8000-00000000b101', '00000000-0000-7000-8000-00000000d001', 'rme_patient'),
     ('00000000-0000-7000-8000-00000000e102', '2027-03-15 01:00:00+00', '00000000-0000-7000-8000-00000000b101', '00000000-0000-7000-8000-00000000d003', 'rme_patient'),
     ('00000000-0000-7000-8000-00000000e103', '2027-03-15 02:00:00+00', '00000000-0000-7000-8000-00000000b102', '00000000-0000-7000-8000-00000000d001', 'rme_patient'),
     ('00000000-0000-7000-8000-00000000e104', '2027-03-15 03:00:00+00', '00000000-0000-7000-8000-00000000b103', '00000000-0000-7000-8000-00000000d002', 'care_recipient')$s$, 4);
DO $$
BEGIN
  IF (SELECT count(*) FROM audit.audit_event_y2027m03) <> 3 OR (SELECT count(*) FROM audit.audit_event_y2027m04) <> 1 THEN
    RAISE EXCEPTION 'GAGAL §6.2 routing partisi: batas bulan UTC tidak sesuai';
  END IF;
  RAISE NOTICE 'LULUS §6.2 routing partisi: tiga event ke y2027m03, event tepat 2027-04-01 00:00 UTC ke y2027m04';
END $$;
-- Hasil verifikasi (V8) §6.4 butir 2 / T-19: ON CONFLICT dengan conflict_target membutuhkan SELECT atas
-- kolom target (dokumentasi INSERT PostgreSQL 18, SRC-470), sehingga writer yang hanya punya INSERT
-- (DD §0.6 B) DITOLAK. Kiriman ulang dapat dibedakan lewat galat 23505 per nama constraint (pk vs
-- uq seq); keputusan bentuk jalur relay diserahkan ke F1-T24 (dilaporkan F1-T21). Bentuk tanpa
-- conflict_target diterima dengan INSERT saja, tetapi juga menelan konflik seq (cabang rantai) tanpa
-- galat, sehingga tidak aman untuk relay.
SELECT pg_temp.uji_tolak('T-19 (V8) writer INSERT ... ON CONFLICT (event_id) DO NOTHING ditolak: conflict_target butuh SELECT event_id (SRC-470)', :'penulis',
  $s$INSERT INTO audit.audit_event_key (event_id, seq, recorded_at) VALUES ('00000000-0000-7000-8000-00000000b101', 5, '2027-04-01 01:00:00+00')
     ON CONFLICT (event_id) DO NOTHING$s$, '42501', 'permission denied for table audit_event_key');
SELECT pg_temp.uji_tolak('T-19 kiriman ulang event_id yang sama ditolak PK (23505 pk_audit_event_key, dapat dibedakan dari konflik seq)', :'penulis',
  $s$INSERT INTO audit.audit_event_key (event_id, seq, recorded_at) VALUES ('00000000-0000-7000-8000-00000000b101', 5, '2027-04-01 01:00:00+00')$s$,
  '23505', 'pk_audit_event_key');
SELECT pg_temp.uji_tolak('T-19 NEG dua relay: seq ganda untuk event berbeda ditolak UQ (cabang rantai, §6.4 butir 5)', :'penulis',
  $s$INSERT INTO audit.audit_event_key (event_id, seq, recorded_at) VALUES ('00000000-0000-7000-8000-00000000b105', 4, '2027-04-01 01:00:00+00')$s$,
  '23505', 'uq_audit_event_key_seq');
SELECT pg_temp.uji_boleh('T-19 (V8) ON CONFLICT DO NOTHING tanpa conflict_target diterima dengan INSERT saja (kiriman ulang: nol baris)', :'penulis',
  $s$INSERT INTO audit.audit_event_key (event_id, seq, recorded_at) VALUES ('00000000-0000-7000-8000-00000000b101', 5, '2027-04-01 01:00:00+00')
     ON CONFLICT DO NOTHING$s$, 0);
SELECT pg_temp.uji_boleh('T-19 (V8) BAHAYA: ON CONFLICT DO NOTHING tanpa conflict_target juga menelan konflik seq (nol baris, tanpa galat) -> tidak dipakai relay', :'penulis',
  $s$INSERT INTO audit.audit_event_key (event_id, seq, recorded_at) VALUES ('00000000-0000-7000-8000-00000000b105', 4, '2027-04-01 01:00:00+00')
     ON CONFLICT DO NOTHING$s$, 0);
SELECT pg_temp.uji_tolak('T-19 NEG ON CONFLICT DO UPDATE oleh writer (tanpa hak UPDATE)', :'penulis',
  $s$INSERT INTO audit.audit_event_key (event_id, seq, recorded_at) VALUES ('00000000-0000-7000-8000-00000000b101', 5, '2027-04-01 01:00:00+00')
     ON CONFLICT (event_id) DO UPDATE SET seq = 5$s$, '42501', 'permission denied for table audit_event_key');
DO $$
DECLARE
  v_seq bigint;
  v_hash bytea;
  v_at timestamptz;
BEGIN
  EXECUTE format('SET LOCAL ROLE %I', current_setting('uji.prefiks') || 'audit_writer_login');
  SELECT seq, event_hash, recorded_at INTO v_seq, v_hash, v_at FROM audit.fn_chain_head();
  RESET ROLE;
  IF v_seq IS DISTINCT FROM 4 OR v_hash IS DISTINCT FROM decode(repeat('44', 32), 'hex') OR v_at IS DISTINCT FROM '2027-04-01 00:00:00+00'::timestamptz THEN
    RAISE EXCEPTION 'GAGAL T-19 fn_chain_head: % % %', v_seq, encode(v_hash, 'hex'), v_at;
  END IF;
  RAISE NOTICE 'LULUS T-19 writer membaca kepala rantai lewat fn_chain_head: seq 4, event_hash dan recorded_at event terakhir';
END $$;
SELECT pg_temp.uji_tolak('T-19 NEG writer SELECT audit_event langsung', :'penulis', 'SELECT * FROM audit.audit_event', '42501', 'permission denied for table audit_event');
SELECT pg_temp.uji_tolak('T-19 NEG writer SELECT audit_event_key langsung', :'penulis', 'SELECT max(seq) FROM audit.audit_event_key', '42501', 'permission denied for table audit_event_key');
SELECT pg_temp.uji_tolak('T-19 NEG writer SELECT audit_event_subject langsung', :'penulis', 'SELECT * FROM audit.audit_event_subject', '42501', 'permission denied for table audit_event_subject');
SELECT pg_temp.uji_tolak('T-19 NEG writer INSERT ... RETURNING (butuh SELECT)', :'penulis',
  $s$INSERT INTO audit.audit_event_key (event_id, seq, recorded_at) VALUES ('00000000-0000-7000-8000-00000000b106', 6, '2027-04-01 01:00:00+00') RETURNING seq$s$,
  '42501', 'permission denied for table audit_event_key');
SELECT pg_temp.uji_tolak('T-19 NEG writer membaca partisi langsung', :'penulis', 'SELECT * FROM audit.audit_event_y2027m03', '42501', 'permission denied for table audit_event_y2027m03');
SELECT pg_temp.uji_tolak('T-09 NEG writer UPDATE audit_event', :'penulis', 'UPDATE audit.audit_event SET outcome = ''failure''', '42501', 'permission denied for table audit_event');
SELECT pg_temp.uji_tolak('T-09 NEG writer DELETE audit_event_key', :'penulis', 'DELETE FROM audit.audit_event_key', '42501', 'permission denied for table audit_event_key');
SELECT pg_temp.uji_tolak('T-09 NEG writer TRUNCATE audit_event', :'penulis', 'TRUNCATE audit.audit_event', '42501', 'permission denied for table audit_event');
SELECT pg_temp.uji_tolak('T-09 NEG writer INSERT hasil verifikasi (pihak yang diperiksa, DBA5-B01)', :'penulis',
  $s$INSERT INTO audit.audit_chain_verification (id, started_at, finished_at, from_seq, to_seq, outcome) VALUES ('00000000-0000-7000-8000-00000000f101', now(), now(), 1, 4, 'match')$s$,
  '42501', 'permission denied for table audit_chain_verification');
SELECT pg_temp.uji_tolak('T-09 NEG writer INSERT manifest arsip', :'penulis',
  $s$INSERT INTO audit.audit_archive_manifest (id, partition_name, range_start, range_end, from_seq, to_seq, boundary_event_hash, archive_object_key, archive_sha256, exported_at, detached_at)
     VALUES ('00000000-0000-7000-8000-00000000f201', 'audit_event_y2027m03', '2027-03-01 00:00:00+00', '2027-04-01 00:00:00+00', 1, 3, decode(repeat('33', 32), 'hex'), 'k', decode(repeat('aa', 32), 'hex'), now(), now())$s$,
  '42501', 'permission denied for table audit_archive_manifest');
-- §6.2 butir 3: tanpa partisi DEFAULT, INSERT di luar rentang gagal (event tertahan di outbox).
SELECT pg_temp.uji_tolak('§6.2 NEG INSERT audit_event setelah partisi terakhir (2030-01) gagal, tanpa DEFAULT', :'penulis',
  $s$INSERT INTO audit.audit_event (event_id, recorded_at, seq, occurred_at, event_code, action_name, actor_type, object_type, subject_count, source_component, outcome, correlation_id, canonical_version, prev_hash, event_hash)
     VALUES ('00000000-0000-7000-8000-00000000b107', '2030-01-01 00:00:00+00', 7, now(), 'e_05', 'x', 'system', 'x', 0, 'worker', 'success', '00000000-0000-7000-8000-00000000c107', 1, decode(repeat('00', 32), 'hex'), decode(repeat('00', 32), 'hex'))$s$,
  '23514', 'no partition of relation "audit_event" found for row');
SELECT pg_temp.uji_tolak('§6.2 NEG INSERT audit_event_subject sebelum partisi pertama (2026-10) gagal', :'penulis',
  $s$INSERT INTO audit.audit_event_subject (id, recorded_at, event_id, subject_id, subject_kind)
     VALUES ('00000000-0000-7000-8000-00000000e105', '2026-10-31 23:59:59+00', '00000000-0000-7000-8000-00000000b107', '00000000-0000-7000-8000-00000000d001', 'rme_patient')$s$,
  '23514', 'no partition of relation "audit_event_subject" found for row');
-- CHECK isi DD B1-B6 (role sungguhan; kolom lain sah).
DO $$
DECLARE
  c record;
  w text := current_setting('uji.prefiks') || 'audit_writer_login';
  ev text := $s$INSERT INTO audit.audit_event (event_id, recorded_at, seq, occurred_at, event_code, action_name, actor_type, object_type, subject_count, source_component, outcome, correlation_id, canonical_version, prev_hash, event_hash)
               VALUES ('00000000-0000-7000-8000-00000000b108', '2027-05-01 00:00:00+00', 8, now(), %s, %s, %s, 'x', %s, 'worker', %s, '00000000-0000-7000-8000-00000000c108', 1, %s, %s)$s$;
BEGIN
  FOR c IN SELECT * FROM (VALUES
      ('event_code e_18', format(ev, '''e_18''', '''x''', '''system''', '0', '''success''', 'decode(repeat(''00'', 32), ''hex'')', 'decode(repeat(''00'', 32), ''hex'')'), 'ck_audit_event_event_code'),
      ('actor_type user', format(ev, '''e_01''', '''x''', '''user''', '0', '''success''', 'decode(repeat(''00'', 32), ''hex'')', 'decode(repeat(''00'', 32), ''hex'')'), 'ck_audit_event_actor_type'),
      ('subject_count negatif', format(ev, '''e_01''', '''x''', '''system''', '-1', '''success''', 'decode(repeat(''00'', 32), ''hex'')', 'decode(repeat(''00'', 32), ''hex'')'), 'ck_audit_event_subject_count'),
      ('outcome ok', format(ev, '''e_01''', '''x''', '''system''', '0', '''ok''', 'decode(repeat(''00'', 32), ''hex'')', 'decode(repeat(''00'', 32), ''hex'')'), 'ck_audit_event_outcome'),
      ('prev_hash 31 byte', format(ev, '''e_01''', '''x''', '''system''', '0', '''success''', 'decode(repeat(''00'', 31), ''hex'')', 'decode(repeat(''00'', 32), ''hex'')'), 'ck_audit_event_prev_hash'),
      ('event_hash 33 byte', format(ev, '''e_01''', '''x''', '''system''', '0', '''success''', 'decode(repeat(''00'', 32), ''hex'')', 'decode(repeat(''00'', 33), ''hex'')'), 'ck_audit_event_event_hash'),
      ('subject_kind patient', $s$INSERT INTO audit.audit_event_subject (id, recorded_at, event_id, subject_id, subject_kind) VALUES ('00000000-0000-7000-8000-00000000e106', '2027-05-01 00:00:00+00', '00000000-0000-7000-8000-00000000b108', '00000000-0000-7000-8000-00000000d001', 'patient')$s$, 'ck_audit_event_subject_subject_kind'),
      ('seq 0 di kunci event', $s$INSERT INTO audit.audit_event_key (event_id, seq, recorded_at) VALUES ('00000000-0000-7000-8000-00000000b108', 0, '2027-05-01 00:00:00+00')$s$, 'ck_audit_event_key_seq'),
      ('event_hash jangkar 16 byte', $s$INSERT INTO audit.audit_chain_anchor (id, seq, event_hash, anchored_at, worm_object_key) VALUES ('00000000-0000-7000-8000-00000000a101', 4, decode(repeat('44', 16), 'hex'), now(), 'k')$s$, 'ck_audit_chain_anchor_event_hash')
    ) AS t(label, stmt, pola)
  LOOP
    PERFORM pg_temp.uji_tolak('B NEG CHECK ' || c.label, w, c.stmt, '23514', c.pola);
  END LOOP;
END $$;
SELECT pg_temp.uji_boleh('B4 writer INSERT jangkar kepala rantai (seq 4)', :'penulis',
  $s$INSERT INTO audit.audit_chain_anchor (id, seq, event_hash, anchored_at, worm_object_key) VALUES ('00000000-0000-7000-8000-00000000a101', 4, decode(repeat('44', 32), 'hex'), '2027-04-01 01:00:00+00', 'worm/anchor/2027-04-01T01')$s$, 1);
SELECT pg_temp.uji_tolak('B4 NEG jangkar ganda untuk seq yang sama', :'penulis',
  $s$INSERT INTO audit.audit_chain_anchor (id, seq, event_hash, anchored_at, worm_object_key) VALUES ('00000000-0000-7000-8000-00000000a102', 4, decode(repeat('44', 32), 'hex'), now(), 'k')$s$,
  '23505', 'uq_audit_chain_anchor_seq');
SELECT pg_temp.uji_tolak('T-09 NEG verifier INSERT jangkar (hanya writer)', :'pemverifikasi',
  $s$INSERT INTO audit.audit_chain_anchor (id, seq, event_hash, anchored_at, worm_object_key) VALUES ('00000000-0000-7000-8000-00000000a103', 3, decode(repeat('33', 32), 'hex'), now(), 'k')$s$,
  '42501', 'permission denied for table audit_chain_anchor');

-- Verifier (§3.3, KEPUTUSAN-141): baca seluruh rantai, tulis hasil verifikasi saja.
SELECT pg_temp.uji_boleh('T-09 verifier membaca seluruh audit_event (4 baris)', :'pemverifikasi', 'SELECT * FROM audit.audit_event', 4);
SELECT pg_temp.uji_boleh('T-09 verifier membaca audit_event_key, audit_event_subject, jangkar', :'pemverifikasi',
  'SELECT k.seq FROM audit.audit_event_key k, audit.audit_chain_anchor a WHERE a.seq = k.seq UNION ALL SELECT 1 FROM audit.audit_event_subject', 5);
SELECT pg_temp.uji_boleh('B5 verifier INSERT hasil verifikasi match (FK ke jangkar)', :'pemverifikasi',
  $s$INSERT INTO audit.audit_chain_verification (id, started_at, finished_at, from_seq, to_seq, audit_chain_anchor_id, outcome)
     VALUES ('00000000-0000-7000-8000-00000000f101', '2027-04-02 00:00:00+00', '2027-04-02 00:01:00+00', 1, 4, '00000000-0000-7000-8000-00000000a101', 'match')$s$, 1);
SELECT pg_temp.uji_tolak('B5 NEG hasil verifikasi dengan jangkar yang tidak ada', :'pemverifikasi',
  $s$INSERT INTO audit.audit_chain_verification (id, started_at, finished_at, from_seq, to_seq, audit_chain_anchor_id, outcome)
     VALUES ('00000000-0000-7000-8000-00000000f102', now(), now(), 1, 4, '00000000-0000-7000-8000-00000000ffff', 'match')$s$,
  '23503', 'fk_audit_chain_verification_audit_chain_anchor_id');
SELECT pg_temp.uji_tolak('B5 NEG outcome verifikasi di luar daftar', :'pemverifikasi',
  $s$INSERT INTO audit.audit_chain_verification (id, started_at, finished_at, from_seq, to_seq, outcome) VALUES ('00000000-0000-7000-8000-00000000f102', now(), now(), 1, 4, 'ok')$s$,
  '23514', 'ck_audit_chain_verification_outcome');
SELECT pg_temp.uji_tolak('T-09 NEG verifier mengubah hasil verifikasi menjadi match', :'pemverifikasi',
  'UPDATE audit.audit_chain_verification SET outcome = ''match''', '42501', 'permission denied for table audit_chain_verification');
SELECT pg_temp.uji_tolak('T-19 NEG verifier INSERT audit_event', :'pemverifikasi',
  $s$INSERT INTO audit.audit_event_key (event_id, seq, recorded_at) VALUES ('00000000-0000-7000-8000-00000000b109', 9, '2027-05-01 00:00:00+00')$s$,
  '42501', 'permission denied for table audit_event_key');
SELECT pg_temp.uji_tolak('T-09 NEG verifier INSERT manifest arsip (pemilik saja, DD §0.6 B)', :'pemverifikasi',
  $s$INSERT INTO audit.audit_archive_manifest (id, partition_name, range_start, range_end, from_seq, to_seq, boundary_event_hash, archive_object_key, archive_sha256, exported_at, detached_at)
     VALUES ('00000000-0000-7000-8000-00000000f201', 'audit_event_y2027m03', '2027-03-01 00:00:00+00', '2027-04-01 00:00:00+00', 1, 3, decode(repeat('33', 32), 'hex'), 'k', decode(repeat('aa', 32), 'hex'), now(), now())$s$,
  '42501', 'permission denied for table audit_archive_manifest');
SELECT pg_temp.uji_tolak('§6.3 NEG verifier memanggil fn_chain_head', :'pemverifikasi', 'SELECT * FROM audit.fn_chain_head()', '42501', 'permission denied for function fn_chain_head');

-- Reader (backoffice): SELECT saja.
SELECT pg_temp.uji_boleh('T-09 reader membaca audit_event dan manifest', :'pembaca', 'SELECT event_id FROM audit.audit_event UNION ALL SELECT id FROM audit.audit_archive_manifest', 4);
SELECT pg_temp.uji_tolak('T-09 NEG reader INSERT audit_event_subject', :'pembaca',
  $s$INSERT INTO audit.audit_event_subject (id, recorded_at, event_id, subject_id, subject_kind) VALUES ('00000000-0000-7000-8000-00000000e107', '2027-03-15 01:00:00+00', '00000000-0000-7000-8000-00000000b101', '00000000-0000-7000-8000-00000000d004', 'rme_patient')$s$,
  '42501', 'permission denied for table audit_event_subject');
SELECT pg_temp.uji_tolak('T-09 NEG reader DELETE jangkar', :'pembaca', 'DELETE FROM audit.audit_chain_anchor', '42501', 'permission denied for table audit_chain_anchor');
SELECT pg_temp.uji_tolak('§6.3 NEG reader memanggil fn_chain_head', :'pembaca', 'SELECT * FROM audit.fn_chain_head()', '42501', 'permission denied for function fn_chain_head');

-- Pemilik skema: INSERT manifest diizinkan (DD §0.6 B); UPDATE/DELETE/TRUNCATE ditolak trigger di
-- setiap tabel, termasuk partisi langsung (ADR-009 butir 4, §11.4 tanpa pengecualian).
SELECT pg_temp.uji_boleh('B6 pemilik INSERT manifest arsip', :'pemilik',
  $s$INSERT INTO audit.audit_archive_manifest (id, partition_name, range_start, range_end, from_seq, to_seq, boundary_event_hash, archive_object_key, archive_sha256, exported_at, detached_at)
     VALUES ('00000000-0000-7000-8000-00000000f201', 'audit_event_y2027m03', '2027-03-01 00:00:00+00', '2027-04-01 00:00:00+00', 1, 3, decode(repeat('33', 32), 'hex'), 'archive/audit_event_y2027m03.jsonl', decode(repeat('aa', 32), 'hex'), now(), now())$s$, 1);
SELECT pg_temp.uji_tolak('B6 NEG manifest ganda untuk partisi yang sama', :'pemilik',
  $s$INSERT INTO audit.audit_archive_manifest (id, partition_name, range_start, range_end, from_seq, to_seq, boundary_event_hash, archive_object_key, archive_sha256, exported_at, detached_at)
     VALUES ('00000000-0000-7000-8000-00000000f202', 'audit_event_y2027m03', '2027-03-01 00:00:00+00', '2027-04-01 00:00:00+00', 1, 3, decode(repeat('33', 32), 'hex'), 'k2', decode(repeat('aa', 32), 'hex'), now(), now())$s$,
  '23505', 'uq_audit_archive_manifest_partition_name');
SELECT pg_temp.uji_tolak('B6 NEG manifest archive_sha256 bukan 32 byte', :'pemilik',
  $s$INSERT INTO audit.audit_archive_manifest (id, partition_name, range_start, range_end, from_seq, to_seq, boundary_event_hash, archive_object_key, archive_sha256, exported_at, detached_at)
     VALUES ('00000000-0000-7000-8000-00000000f202', 'audit_event_y2027m04', '2027-04-01 00:00:00+00', '2027-05-01 00:00:00+00', 4, 4, decode(repeat('44', 32), 'hex'), 'k2', decode(repeat('aa', 20), 'hex'), now(), now())$s$,
  '23514', 'ck_audit_archive_manifest_archive_sha256');
DO $$
DECLARE
  c record;
  o text := current_setting('uji.prefiks') || 'audit_migration';
BEGIN
  FOR c IN SELECT * FROM (VALUES
      ('UPDATE audit_event', 'UPDATE audit.audit_event SET outcome = ''failure''', 'UPDATE on audit.audit_event'),
      ('DELETE audit_event nol baris', 'DELETE FROM audit.audit_event WHERE false', 'DELETE on audit.audit_event'),
      ('UPDATE partisi audit_event_y2027m04 langsung', 'UPDATE audit.audit_event_y2027m04 SET outcome = ''failure''', 'UPDATE on audit.audit_event_y2027m04'),
      ('DELETE partisi subjek langsung', 'DELETE FROM audit.audit_event_subject_y2027m03', 'DELETE on audit.audit_event_subject_y2027m03'),
      ('TRUNCATE audit_event_subject', 'TRUNCATE audit.audit_event_subject', 'TRUNCATE on audit.audit_event_subject'),
      ('UPDATE audit_event_key', 'UPDATE audit.audit_event_key SET seq = seq + 100', 'UPDATE on audit.audit_event_key'),
      ('TRUNCATE audit_event_key', 'TRUNCATE audit.audit_event_key', 'TRUNCATE on audit.audit_event_key'),
      ('DELETE jangkar', 'DELETE FROM audit.audit_chain_anchor', 'DELETE on audit.audit_chain_anchor'),
      ('UPDATE hasil verifikasi', 'UPDATE audit.audit_chain_verification SET outcome = ''mismatch''', 'UPDATE on audit.audit_chain_verification'),
      ('TRUNCATE jangkar + verifikasi', 'TRUNCATE audit.audit_chain_anchor, audit.audit_chain_verification', 'TRUNCATE on audit.audit_chain_(anchor|verification)'),
      ('UPDATE manifest', 'UPDATE audit.audit_archive_manifest SET archive_object_key = ''x''', 'UPDATE on audit.audit_archive_manifest'),
      ('TRUNCATE manifest', 'TRUNCATE audit.audit_archive_manifest', 'TRUNCATE on audit.audit_archive_manifest')
    ) AS t(label, stmt, pola)
  LOOP
    PERFORM pg_temp.uji_tolak('T-09 NEG pemilik ' || c.label || ' ditolak trigger', o, c.stmt, '42501', '^immutable_violation: ' || c.pola || ' rejected$');
  END LOOP;
END $$;

-- RLS tenant rme_audit (§5.6, ADR-009 butir 8, ADR-012 butir 9): Admin Fasyankes hanya E-05 fasyankesnya.
DO $$
DECLARE
  ta text := '00000000-0000-7000-8000-00000000a00a';
  tb text := '00000000-0000-7000-8000-00000000a00b';
  t text := 'rme_audit_tenant_reader_login';
BEGIN
  IF current_setting('uji.prefiks') <> 'rme_' THEN
    IF EXISTS (SELECT 1 FROM pg_class c JOIN pg_namespace n ON n.oid = c.relnamespace, aclexplode(c.relacl) a
                WHERE n.nspname = 'audit' AND pg_get_userbyid(a.grantee) LIKE 'rme\_%') THEN
      RAISE EXCEPTION 'GAGAL T-21/T-24 ruang_sehat_audit: ada hak untuk role rme_*';
    END IF;
    RAISE NOTICE 'LULUS ruang_sehat_audit tanpa hak role rme_* (tenant reader hanya di rme_audit)';
    RETURN;
  END IF;
  PERFORM pg_temp.uji_boleh('§5.6 tenant reader tanpa konteks tenant melihat nol event (gagal-tertutup)', t, 'SELECT * FROM audit.audit_event', 0);
  PERFORM pg_temp.uji_boleh('§5.6 tenant reader tanpa konteks tenant melihat nol subjek', t, 'SELECT * FROM audit.audit_event_subject', 0);
  PERFORM set_config('rme.fasyankes_id', ta, true);
  PERFORM pg_temp.uji_boleh('§5.6 tenant reader fasyankes A melihat hanya E-05 fasyankes A (E1, E4; bukan E-06 A, bukan E-05 B)', t,
    format('SELECT * FROM audit.audit_event WHERE event_id IN (%L, %L)', '00000000-0000-7000-8000-00000000b101', '00000000-0000-7000-8000-00000000b104'), 2);
  PERFORM pg_temp.uji_boleh('§5.6 tenant reader fasyankes A: total baris terlihat = 2', t, 'SELECT * FROM audit.audit_event', 2);
  PERFORM pg_temp.uji_boleh('§5.6 tenant reader fasyankes A: subjek hanya milik event yang terlihat (2 subjek E1)', t, 'SELECT * FROM audit.audit_event_subject', 2);
  PERFORM pg_temp.uji_boleh('§5.6 subjek event E-06 (E2) tidak terlihat walau fasyankes sama', t,
    format('SELECT * FROM audit.audit_event_subject WHERE event_id = %L', '00000000-0000-7000-8000-00000000b102'), 0);
  PERFORM set_config('rme.fasyankes_id', tb, true);
  PERFORM pg_temp.uji_boleh('§5.6 tenant reader fasyankes B melihat hanya E3', t, 'SELECT * FROM audit.audit_event', 1);
  PERFORM pg_temp.uji_boleh('§5.6 tenant reader fasyankes B melihat hanya subjek E3', t, 'SELECT * FROM audit.audit_event_subject', 1);
  PERFORM pg_temp.uji_tolak('§5.6 NEG tenant reader membaca partisi langsung (hak hanya di induk)', t, 'SELECT * FROM audit.audit_event_y2027m03', '42501', 'permission denied for table audit_event_y2027m03');
  PERFORM pg_temp.uji_tolak('§5.6 NEG tenant reader membaca audit_event_key', t, 'SELECT * FROM audit.audit_event_key', '42501', 'permission denied for table audit_event_key');
  PERFORM pg_temp.uji_tolak('§5.6 NEG tenant reader INSERT audit_event_subject', t,
    $s$INSERT INTO audit.audit_event_subject (id, recorded_at, event_id, subject_id, subject_kind) VALUES ('00000000-0000-7000-8000-00000000e108', '2027-03-15 03:00:00+00', '00000000-0000-7000-8000-00000000b103', '00000000-0000-7000-8000-00000000d005', 'rme_patient')$s$,
    '42501', 'permission denied for table audit_event_subject');
  PERFORM pg_temp.uji_tolak('§5.6 NEG tenant reader memanggil fn_chain_head', t, 'SELECT * FROM audit.fn_chain_head()', '42501', 'permission denied for function fn_chain_head');
  PERFORM set_config('rme.fasyankes_id', '', true);
  PERFORM pg_temp.uji_boleh('§5.6 konteks dikosongkan: tenant reader kembali melihat nol event', t, 'SELECT * FROM audit.audit_event', 0);
  PERFORM pg_temp.uji_boleh('§5.6 verifier tetap melihat semua event di bawah FORCE RLS (kebijakan reader)', 'rme_audit_verifier_login', 'SELECT * FROM audit.audit_event', 4);
  PERFORM pg_temp.uji_boleh('§5.6 reader RME melihat semua subjek di bawah FORCE RLS', 'rme_audit_reader', 'SELECT * FROM audit.audit_event_subject', 4);
  PERFORM pg_temp.uji_boleh('§5.6 pemilik (FORCE RLS) membaca lewat kebijakan migration', 'rme_audit_migration', 'SELECT * FROM audit.audit_event', 4);
END $$;
ROLLBACK;

DO $$
BEGIN
  IF EXISTS (SELECT 1 FROM audit.audit_event) OR EXISTS (SELECT 1 FROM audit.audit_event_key)
     OR EXISTS (SELECT 1 FROM audit.audit_event_subject) OR EXISTS (SELECT 1 FROM audit.audit_chain_anchor)
     OR EXISTS (SELECT 1 FROM audit.audit_chain_verification) OR EXISTS (SELECT 1 FROM audit.audit_archive_manifest) THEN
    RAISE EXCEPTION 'GAGAL pembersihan tabel-audit-b %: data uji tersisa', current_database();
  END IF;
  RAISE NOTICE 'LULUS pembersihan tabel-audit-b %: tidak ada data uji tersisa (ROLLBACK)', current_database();
END $$;
