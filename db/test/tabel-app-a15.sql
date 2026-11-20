-- db/test/tabel-app-a15.sql: test tabel A15.4 cms_content (F1-T91) di database ruang_sehat.
-- Didahului db/test/lib.sql; dijalankan superuser setelah migrasi. Aturan kelas generik (T-06/T-07/T-08)
-- di db/test/kelas-app.sql bagian A, T-01/T-20 generik di db/test/katalog.sql. Berkas ini memeriksa struktur
-- khusus (komentar kelas, hak tabel persis, index, FK, CHECK kanal/halaman/status/teks alternatif) dan
-- perilaku dengan role sungguhan (positif dan negatif). Satu transaksi yang di-ROLLBACK. Data sintetis.
-- Dasar: docs/04-data-dictionary.md §0.3, §0.6 A, A15.4 (+ usulan A20); docs/05-physical-schema.md §1, §4.1,
-- §4.3, §12 (T-06, T-09); ADR-003; PRD v1.4 US-1101 AC1-AC3, US-1802 AC1, AC2, AC5; BR-130.

-- 1. Struktur.
SELECT pg_temp.uji_nol('A15.4 komentar kelas: M modul konten',
  ARRAY(SELECT coalesce(obj_description('app.cms_content'::regclass, 'pg_class'), '<tidak ada>')
         WHERE obj_description('app.cms_content'::regclass, 'pg_class') IS DISTINCT FROM 'class:M module:konten'));
SELECT pg_temp.uji_nol('A15.4 tanpa kolom anonymize (DD A15.4 tanpa kolom "Anonimkan") dan tanpa kolom *_enc/*_bidx',
  ARRAY(SELECT attname::text FROM pg_attribute
         WHERE attrelid = 'app.cms_content'::regclass AND attnum > 0 AND NOT attisdropped
           AND (col_description(attrelid, attnum) IS NOT NULL OR attname ~ '(_enc|_bidx|_bidx_key_version)$')));
SELECT pg_temp.uji_nol('A15.4 hak tabel persis (M: rs_app S,I,U + pekerja privasi S); tanpa DELETE, relay, rotasi kunci, PUBLIC; tanpa hak per kolom',
  ARRAY(SELECT format('cms_content: %s', coalesce(a.k, '<kosong>'))
          FROM (SELECT string_agg(CASE WHEN x.grantee = 0 THEN 'PUBLIC' ELSE pg_get_userbyid(x.grantee) END || ':' || x.privilege_type,
                                  ',' ORDER BY 1) AS k
                  FROM pg_class c, aclexplode(c.relacl) x
                 WHERE c.oid = 'app.cms_content'::regclass AND x.grantee <> c.relowner) a
         WHERE a.k IS DISTINCT FROM 'rs_app:INSERT,rs_app:SELECT,rs_app:UPDATE,rs_privacy_worker:SELECT'
        UNION ALL
        SELECT format('hak per kolom %s', att.attname) FROM pg_attribute att
         WHERE att.attrelid = 'app.cms_content'::regclass AND att.attnum > 0 AND att.attacl IS NOT NULL));
SELECT pg_temp.uji_nol('A15.4 kolom persis (STD-M + DD A15.4 + image_alt_text, channel, site_page)',
  ARRAY(SELECT format('kolom %s', x) FROM (
          SELECT string_agg(attname || ':' || format_type(atttypid, atttypmod) || ':' || CASE WHEN attnotnull THEN 'nn' ELSE 'n' END,
                            ',' ORDER BY attnum) AS x
            FROM pg_attribute WHERE attrelid = 'app.cms_content'::regclass AND attnum > 0 AND NOT attisdropped) s
         WHERE x IS DISTINCT FROM
           'id:uuid:nn,created_at:timestamp with time zone:nn,created_by:uuid:n,updated_at:timestamp with time zone:nn,'
           || 'updated_by:uuid:n,deleted_at:timestamp with time zone:n,deleted_by:uuid:n,delete_reason:text:n,'
           || 'content_type:text:nn,title:text:nn,body:text:n,category:text:n,image_file_id:uuid:n,image_alt_text:text:n,'
           || 'link_target_type:text:n,link_target_ref:text:n,publish_start_at:timestamp with time zone:n,'
           || 'publish_end_at:timestamp with time zone:n,editorial_status:text:nn,channel:text:nn,site_page:text:n'));
SELECT pg_temp.uji_nol('A15.4 index persis: pk, ix DD (jenis, status, mulai), ix halaman situs, uqp satu halaman tetap terbuka',
  ARRAY(SELECT pg_temp.uji_indeks('app.cms_content', ARRAY[
    'CREATE UNIQUE INDEX pk_cms_content ON app.cms_content USING btree (id)',
    'CREATE INDEX ix_cms_content_type_status_start ON app.cms_content USING btree (content_type, editorial_status, publish_start_at) WHERE (deleted_at IS NULL)',
    'CREATE INDEX ix_cms_content_site_page_status_start ON app.cms_content USING btree (site_page, editorial_status, publish_start_at) WHERE ((deleted_at IS NULL) AND (site_page IS NOT NULL))',
    'CREATE UNIQUE INDEX uqp_cms_content_site_page_open ON app.cms_content USING btree (site_page) WHERE ((deleted_at IS NULL) AND (content_type = ''info_page''::text) AND (editorial_status = ''scheduled''::text) AND (publish_end_at IS NULL) AND (site_page IS NOT NULL))'])));
SELECT pg_temp.uji_nol('A15.4 FK persis (aktor -> account, gambar -> stored_file), tanpa CASCADE',
  ARRAY(SELECT x FROM (VALUES
      ('fk_cms_content_created_by', 'FOREIGN KEY (created_by) REFERENCES app.account(id)'),
      ('fk_cms_content_updated_by', 'FOREIGN KEY (updated_by) REFERENCES app.account(id)'),
      ('fk_cms_content_deleted_by', 'FOREIGN KEY (deleted_by) REFERENCES app.account(id)'),
      ('fk_cms_content_image_file_id', 'FOREIGN KEY (image_file_id) REFERENCES app.stored_file(id)')) v(n, def),
        LATERAL (SELECT format('%s: %s', v.n, coalesce(pg_get_constraintdef(k.oid), '<tidak ada>')) AS x
                   FROM (SELECT 1) d LEFT JOIN pg_constraint k ON k.conname = v.n AND k.conrelid = 'app.cms_content'::regclass) z
         WHERE x IS DISTINCT FROM format('%s: %s', v.n, v.def)
        UNION ALL
        SELECT format('jumlah FK %s', count(*)) FROM pg_constraint
         WHERE conrelid = 'app.cms_content'::regclass AND contype = 'f' HAVING count(*) <> 4));
SELECT pg_temp.uji_nol('A15.4 CHECK enum persis (jenis, tujuan tautan, status, kanal, sembilan halaman BR-130)',
  ARRAY(SELECT x FROM (VALUES
      ('ck_cms_content_content_type', 'CHECK ((content_type = ANY (ARRAY[''banner''::text, ''info_page''::text, ''article''::text, ''promo''::text])))'),
      ('ck_cms_content_link_target_type', 'CHECK ((link_target_type = ANY (ARRAY[''article''::text, ''service''::text, ''partner''::text, ''product''::text])))'),
      ('ck_cms_content_editorial_status', 'CHECK ((editorial_status = ANY (ARRAY[''draft''::text, ''scheduled''::text, ''archived''::text])))'),
      ('ck_cms_content_channel', 'CHECK ((channel = ANY (ARRAY[''app''::text, ''site''::text, ''app_and_site''::text])))'),
      ('ck_cms_content_site_page', 'CHECK ((site_page = ANY (ARRAY[''home''::text, ''about''::text, ''services''::text, ''for_business''::text, ''partners''::text, ''articles''::text, ''contact''::text, ''download_app''::text, ''privacy_terms''::text])))'),
      ('ck_cms_content_channel_site_page', 'CHECK (((channel = ''app''::text) = (site_page IS NULL)))'),
      ('ck_cms_content_scheduled_start', 'CHECK (((editorial_status <> ''scheduled''::text) OR (publish_start_at IS NOT NULL)))'),
      ('ck_cms_content_publish_window', 'CHECK (((publish_end_at IS NULL) OR (publish_end_at > publish_start_at)))')) v(n, def),
        LATERAL (SELECT format('%s: %s', v.n, coalesce(pg_get_constraintdef(k.oid), '<tidak ada>')) AS x
                   FROM (SELECT 1) d LEFT JOIN pg_constraint k ON k.conname = v.n AND k.conrelid = 'app.cms_content'::regclass) z
         WHERE x IS DISTINCT FROM format('%s: %s', v.n, v.def)));
SELECT pg_temp.uji_nol('A15.4 tanpa trigger (kelas M, §4.11)',
  ARRAY(SELECT tgname::text FROM pg_trigger WHERE tgrelid = 'app.cms_content'::regclass AND NOT tgisinternal));

-- 2. Negatif atas pemeriksa, lalu perilaku. Semua di-ROLLBACK.
BEGIN;
DO $$
DECLARE
  c record;
  kelas text := $q$SELECT pg_temp.uji_kelas_app('app', '{consent_text_version}')$q$;
BEGIN
  FOR c IN SELECT * FROM (VALUES
      ('T-06 DELETE konten untuk rs_app (Master, ADR-003)', 'GRANT DELETE ON app.cms_content TO rs_app', kelas, 'T-06 cms_content .*lebih rs_app:DELETE'),
      ('T-06 UPDATE konten untuk pekerja privasi', 'GRANT UPDATE ON app.cms_content TO rs_privacy_worker', kelas, 'T-06 cms_content .*lebih rs_privacy_worker:UPDATE'),
      ('T-06 INSERT konten dicabut dari rs_app', 'REVOKE INSERT ON app.cms_content FROM rs_app', kelas, 'T-06 cms_content .*kurang rs_app:INSERT'),
      ('T-06 komentar kelas hilang', 'COMMENT ON TABLE app.cms_content IS NULL', kelas, 'T-06 cms_content: komentar kelas tidak sah')
    ) AS t(label, stmt, pemeriksa, pola)
  LOOP
    PERFORM pg_temp.uji_rusak(c.label, c.stmt, c.pemeriksa, c.pola);
  END LOOP;
END $$;

-- Data awal oleh pemilik skema (data sintetis): akun admin Konten dan satu berkas gambar konten tervalidasi.
SET LOCAL ROLE rs_migration;
INSERT INTO app.account (id, account_type, idp_realm, idp_subject, email, status, status_changed_at, full_name)
  VALUES ('00000000-0000-7000-8000-0000000a1501', 'rs_admin', 'admin', 'sub-a15-1', 'a15-1@contoh.invalid', 'active', now(), 'Admin Konten Sintetis');
INSERT INTO app.stored_file (id, bucket, object_key, category, mime_type, size_bytes, sha256, validated_at)
  VALUES ('00000000-0000-7000-8000-0000000a1511', 'public_content', 'a15-objek-1', 'cms_image', 'image/png', 10,
          decode(repeat('ab', 32), 'hex'), now());
RESET ROLE;

-- Positif: draf aplikasi, draf situs, terjadwal bergambar dengan teks alternatif, halaman tetap terbuka.
SELECT pg_temp.uji_boleh('A15.4 rs_api membuat draf kanal aplikasi (tanpa halaman situs) dan draf situs (dengan halaman)', 'rs_api',
  'INSERT INTO app.cms_content (id, created_by, updated_by, content_type, title, body, editorial_status, channel, site_page) VALUES '
  || '(''00000000-0000-7000-8000-0000000a1521'', ''00000000-0000-7000-8000-0000000a1501'', ''00000000-0000-7000-8000-0000000a1501'', ''article'', ''Artikel sintetis'', ''## Isi'', ''draft'', ''app'', NULL), '
  || '(''00000000-0000-7000-8000-0000000a1522'', ''00000000-0000-7000-8000-0000000a1501'', ''00000000-0000-7000-8000-0000000a1501'', ''info_page'', ''Tentang Kami'', ''Isi'', ''draft'', ''site'', ''about'')', 2);
SELECT pg_temp.uji_boleh('A15.4 rs_api menjadwalkan halaman tetap bergambar dengan teks alternatif (US-1802 AC5)', 'rs_api',
  'UPDATE app.cms_content SET editorial_status = ''scheduled'', publish_start_at = now(), image_file_id = ''00000000-0000-7000-8000-0000000a1511'', '
  || 'image_alt_text = ''Foto gedung sintetis'', updated_at = now() WHERE id = ''00000000-0000-7000-8000-0000000a1522''', 1);
SELECT pg_temp.uji_boleh('A15.4 rs_api soft delete dengan alasan lalu memulihkan (US-1101 AC3, ADR-003)', 'rs_api',
  'UPDATE app.cms_content SET deleted_at = now(), deleted_by = ''00000000-0000-7000-8000-0000000a1501'', delete_reason = ''uji'' WHERE id = ''00000000-0000-7000-8000-0000000a1521''', 1);
SELECT pg_temp.uji_boleh('A15.4 rs_api memulihkan konten terhapus', 'rs_api',
  'UPDATE app.cms_content SET deleted_at = NULL, deleted_by = NULL, delete_reason = NULL WHERE id = ''00000000-0000-7000-8000-0000000a1521''', 1);

-- Negatif CHECK dan index.
SELECT pg_temp.uji_tolak('A15.4 NEG kanal situs tanpa halaman tujuan (US-1802 AC1)', 'rs_api',
  'INSERT INTO app.cms_content (id, content_type, title, editorial_status, channel) VALUES (''00000000-0000-7000-8000-0000000a1523'', ''article'', ''x'', ''draft'', ''site'')',
  '23514', 'ck_cms_content_channel_site_page');
SELECT pg_temp.uji_tolak('A15.4 NEG kanal aplikasi dengan halaman situs', 'rs_api',
  'INSERT INTO app.cms_content (id, content_type, title, editorial_status, channel, site_page) VALUES (''00000000-0000-7000-8000-0000000a1523'', ''article'', ''x'', ''draft'', ''app'', ''home'')',
  '23514', 'ck_cms_content_channel_site_page');
SELECT pg_temp.uji_tolak('A15.4 NEG halaman situs di luar sembilan halaman BR-130', 'rs_api',
  'INSERT INTO app.cms_content (id, content_type, title, editorial_status, channel, site_page) VALUES (''00000000-0000-7000-8000-0000000a1523'', ''article'', ''x'', ''draft'', ''site'', ''admin'')',
  '23514', 'ck_cms_content_site_page');
SELECT pg_temp.uji_tolak('A15.4 NEG status "published" (Tayang dihitung, bukan disimpan)', 'rs_api',
  'INSERT INTO app.cms_content (id, content_type, title, editorial_status, channel) VALUES (''00000000-0000-7000-8000-0000000a1523'', ''article'', ''x'', ''published'', ''app'')',
  '23514', 'ck_cms_content_editorial_status');
SELECT pg_temp.uji_tolak('A15.4 NEG terjadwal tanpa waktu mulai', 'rs_api',
  'INSERT INTO app.cms_content (id, content_type, title, editorial_status, channel) VALUES (''00000000-0000-7000-8000-0000000a1523'', ''article'', ''x'', ''scheduled'', ''app'')',
  '23514', 'ck_cms_content_scheduled_start');
SELECT pg_temp.uji_tolak('A15.4 NEG waktu selesai tidak setelah waktu mulai', 'rs_api',
  'INSERT INTO app.cms_content (id, content_type, title, editorial_status, channel, publish_start_at, publish_end_at) VALUES (''00000000-0000-7000-8000-0000000a1523'', ''article'', ''x'', ''draft'', ''app'', now(), now())',
  '23514', 'ck_cms_content_publish_window');
SELECT pg_temp.uji_tolak('A15.4 NEG terjadwal bergambar tanpa teks alternatif (US-1802 AC5)', 'rs_api',
  'INSERT INTO app.cms_content (id, content_type, title, editorial_status, channel, publish_start_at, image_file_id, image_alt_text) VALUES (''00000000-0000-7000-8000-0000000a1523'', ''banner'', ''x'', ''scheduled'', ''app'', now(), ''00000000-0000-7000-8000-0000000a1511'', ''  '')',
  '23514', 'ck_cms_content_image_alt_text');
SELECT pg_temp.uji_tolak('A15.4 NEG judul kosong', 'rs_api',
  'INSERT INTO app.cms_content (id, content_type, title, editorial_status, channel) VALUES (''00000000-0000-7000-8000-0000000a1523'', ''article'', '''', ''draft'', ''app'')',
  '23514', 'ck_cms_content_title_length');
SELECT pg_temp.uji_tolak('A15.4 NEG tujuan tautan tanpa rujukan', 'rs_api',
  'INSERT INTO app.cms_content (id, content_type, title, editorial_status, channel, link_target_type) VALUES (''00000000-0000-7000-8000-0000000a1523'', ''banner'', ''x'', ''draft'', ''app'', ''service'')',
  '23514', 'ck_cms_content_link_target');
SELECT pg_temp.uji_tolak('A15.4 NEG gambar ke berkas yang tidak ada', 'rs_api',
  'INSERT INTO app.cms_content (id, content_type, title, editorial_status, channel, image_file_id) VALUES (''00000000-0000-7000-8000-0000000a1523'', ''banner'', ''x'', ''draft'', ''app'', ''00000000-0000-7000-8000-0000000a15ff'')',
  '23503', 'fk_cms_content_image_file_id');
SELECT pg_temp.uji_tolak('A15.4 NEG soft delete tanpa alasan (ADR-003)', 'rs_api',
  'UPDATE app.cms_content SET deleted_at = now() WHERE id = ''00000000-0000-7000-8000-0000000a1521''', '23514', 'ck_cms_content_soft_delete');
SELECT pg_temp.uji_tolak('A15.4 NEG halaman tetap kedua terjadwal tanpa batas akhir untuk halaman yang sama (US-1802 AC2)', 'rs_api',
  'INSERT INTO app.cms_content (id, content_type, title, editorial_status, channel, site_page, publish_start_at) VALUES (''00000000-0000-7000-8000-0000000a1523'', ''info_page'', ''Tentang Kami v2'', ''scheduled'', ''app_and_site'', ''about'', now() + interval ''1 day'')',
  '23505', 'uqp_cms_content_site_page_open');
SELECT pg_temp.uji_boleh('A15.4 halaman tetap lama diberi batas akhir lalu versi baru terbuka diterima (US-1802 AC2)', 'rs_api',
  'WITH akhir AS (UPDATE app.cms_content SET publish_end_at = now() + interval ''1 day'' WHERE id = ''00000000-0000-7000-8000-0000000a1522'' RETURNING id) '
  || 'INSERT INTO app.cms_content (id, content_type, title, editorial_status, channel, site_page, publish_start_at) '
  || 'SELECT ''00000000-0000-7000-8000-0000000a1524'', ''info_page'', ''Tentang Kami v2'', ''draft'', ''site'', ''about'', NULL FROM akhir', 1);
SELECT pg_temp.uji_boleh('A15.4 versi baru dijadwalkan setelah batas akhir versi lama', 'rs_api',
  'UPDATE app.cms_content SET editorial_status = ''scheduled'', publish_start_at = now() + interval ''1 day'' WHERE id = ''00000000-0000-7000-8000-0000000a1524''', 1);

-- Negatif hak role.
SELECT pg_temp.uji_tolak('A15.4 NEG rs_api DELETE konten (Master tanpa DELETE, ADR-003)', 'rs_api',
  'DELETE FROM app.cms_content WHERE id = ''00000000-0000-7000-8000-0000000a1521''', '42501', 'permission denied for table cms_content');
SELECT pg_temp.uji_tolak('A15.4 NEG rs_api TRUNCATE konten', 'rs_api',
  'TRUNCATE app.cms_content', '42501', 'permission denied for table cms_content');
SELECT pg_temp.uji_tolak('A15.4 NEG pekerja privasi UPDATE konten', 'rs_privacy_worker_login',
  'UPDATE app.cms_content SET title = ''x'' WHERE id = ''00000000-0000-7000-8000-0000000a1521''', '42501', 'permission denied for table cms_content');
SELECT pg_temp.uji_tolak('A15.4 NEG relay audit membaca konten', 'rs_audit_relay_login',
  'SELECT id FROM app.cms_content', '42501', 'permission denied for table cms_content');
SELECT pg_temp.uji_tolak('A15.4 NEG rotasi kunci membaca konten', 'rs_key_rotation',
  'SELECT id FROM app.cms_content', '42501', 'permission denied for table cms_content');
ROLLBACK;

DO $$
BEGIN
  IF EXISTS (SELECT 1 FROM app.cms_content) OR EXISTS (SELECT 1 FROM app.stored_file)
     OR EXISTS (SELECT 1 FROM app.account) THEN
    RAISE EXCEPTION 'GAGAL pembersihan tabel-app-a15: data uji tersisa';
  END IF;
  RAISE NOTICE 'LULUS pembersihan tabel-app-a15: tidak ada data uji tersisa (ROLLBACK)';
END $$;
