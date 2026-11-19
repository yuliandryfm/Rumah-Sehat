-- db/roles/test/umpan-periksa.sql: setelah bootstrap dijalankan ulang di atas umpan
-- (umpan-tanam.sql), memastikan tidak ada umpan yang terpanggil dan role sasaran tetap bukan
-- superuser, lalu menghapus umpan. Variabel psql: -v skema=... -v sasaran=...
-- Review keamanan F1-T13 (BLOKIR, High).

\set ON_ERROR_STOP on
SET search_path = pg_catalog, pg_temp;
SELECT pg_catalog.set_config('uji.skema', :'skema', false) AS _uji_a,
       pg_catalog.set_config('uji.sasaran', :'sasaran', false) AS _uji_b \gset

DO $$
DECLARE
  sk text := pg_catalog.current_setting('uji.skema');
  sasaran text := pg_catalog.current_setting('uji.sasaran');
  dipanggil text;
  f record;
BEGIN
  EXECUTE pg_catalog.format('SELECT pg_catalog.string_agg(fn, '', '') FROM %I.uji_umpan_dipanggil', sk) INTO dipanggil;
  -- Bersihkan dulu supaya klaster tidak tertinggal dalam keadaan tercemar, lalu nilai hasilnya.
  FOR f IN SELECT p.oid::regprocedure AS sig
             FROM pg_catalog.pg_proc p JOIN pg_catalog.pg_namespace n ON n.oid = p.pronamespace
            WHERE n.nspname = sk
              AND p.proname IN ('format', 'current_setting', 'set_config', 'lower', 'quote_ident', 'pg_has_role')
  LOOP
    EXECUTE pg_catalog.format('DROP FUNCTION %s', f.sig);
  END LOOP;
  EXECUTE pg_catalog.format('DROP TABLE %I.uji_umpan_dipanggil', sk);
  IF dipanggil IS NOT NULL OR (SELECT rolsuper FROM pg_catalog.pg_roles WHERE rolname = sasaran) THEN
    EXECUTE pg_catalog.format('ALTER ROLE %I NOSUPERUSER', sasaran);
    RAISE EXCEPTION 'GAGAL NEG pembajakan search_path di %.%: umpan terpanggil [%], % dinaikkan menjadi SUPERUSER',
      pg_catalog.current_database(), sk, coalesce(dipanggil, '-'), sasaran;
  END IF;
  RAISE NOTICE 'LULUS NEG pembajakan search_path: umpan pemilik skema di %.% tidak terpanggil saat bootstrap ulang; % tetap bukan superuser',
    pg_catalog.current_database(), sk, sasaran;
END $$;
