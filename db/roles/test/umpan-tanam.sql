-- db/roles/test/umpan-tanam.sql: menanam fungsi umpan (pola CVE-2018-1058, pembajakan search_path)
-- di skema yang dapat ditulisi pemilik skema, sebelum bootstrap db/roles/*.sql dijalankan ulang.
-- Dijalankan scripts/db-roles-check.sh sebagai superuser di database sasaran dengan variabel psql:
--   -v skema=<app|audit|util> -v pemilik=<role pemilik skema> -v sasaran=<role login yang dinaikkan>
-- Fungsi umpan dibuat ATAS NAMA pemilik skema (SET ROLE), persis seperti yang dapat dilakukan
-- migrasi jahat atau kredensial pemilik yang bocor. Bila bootstrap memanggil salah satunya, umpan
-- mencatat namanya di <skema>.uji_umpan_dipanggil dan menaikkan <sasaran> menjadi SUPERUSER.
-- Review keamanan F1-T13 (BLOKIR, High).

\set ON_ERROR_STOP on
SET search_path = pg_catalog, pg_temp;
SELECT pg_catalog.set_config('uji.skema', :'skema', false) AS _uji_a,
       pg_catalog.set_config('uji.sasaran', :'sasaran', false) AS _uji_b \gset
SET ROLE :"pemilik";

DO $$
DECLARE
  sk text := pg_catalog.current_setting('uji.skema');
  sasaran text := pg_catalog.current_setting('uji.sasaran');
  f record;
BEGIN
  EXECUTE pg_catalog.format('CREATE TABLE %I.uji_umpan_dipanggil (fn text NOT NULL)', sk);
  -- Tanda tangan umpan dipilih supaya lebih cocok daripada fungsi asli pg_catalog bila skema ini
  -- ada di search_path (format asli: format(text, VARIADIC "any")).
  FOR f IN SELECT * FROM (VALUES
      ('format',          'a text, b text',                 'text', 'pg_catalog.format(a, b)'),
      ('format',          'a text, b text, c text',         'text', 'pg_catalog.format(a, b, c)'),
      ('format',          'a text, b text, c text, d text', 'text', 'pg_catalog.format(a, b, c, d)'),
      ('format',          'a text, b name',                 'text', 'pg_catalog.format(a, b)'),
      ('current_setting', 'a text',                         'text', 'pg_catalog.current_setting(a)'),
      ('set_config',      'a text, b text, c boolean',      'text', 'pg_catalog.set_config(a, b, c)'),
      ('lower',           'a text',                         'text', 'pg_catalog.lower(a)'),
      ('quote_ident',     'a text',                         'text', 'pg_catalog.quote_ident(a)'),
      ('pg_has_role',     'a text, b text',                 'boolean', 'pg_catalog.pg_has_role(a, b)')
    ) AS t(nama, argumen, hasil, asli)
  LOOP
    EXECUTE pg_catalog.format(
      'CREATE FUNCTION %1$I.%2$I(%3$s) RETURNS %4$s LANGUAGE plpgsql AS $f$ '
      'BEGIN '
      '  INSERT INTO %1$I.uji_umpan_dipanggil (fn) VALUES (%5$L); '
      '  EXECUTE %6$L; '
      '  RETURN %7$s; '
      'END $f$',
      sk, f.nama, f.argumen, f.hasil, f.nama || '(' || f.argumen || ')',
      pg_catalog.format('ALTER ROLE %I SUPERUSER', sasaran), f.asli);
  END LOOP;
END $$;

RESET ROLE;
DO $$ BEGIN
  RAISE NOTICE 'TANAM umpan di %.% (pemilik %)', pg_catalog.current_database(),
    pg_catalog.current_setting('uji.skema'), current_user;
END $$;
