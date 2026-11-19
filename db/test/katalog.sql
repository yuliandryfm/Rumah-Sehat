-- db/test/katalog.sql: test katalog umum setelah migrasi, untuk setiap database proyek
-- (ruang_sehat, ruang_sehat_audit, rme_audit, rme). Didahului db/test/lib.sql.
-- Variabel psql (dari scripts/db-migrations-check.sh):
--   skema_harap  skema non-sistem yang boleh ada, mis. {app,util,ext}
--   skema_proyek skema milik pemilik skema, mis. {app,util}
--   skema_mig    skema tabel catatan migrasi sqlx (app atau audit)
--   prefiks      awalan role sistem database ini (rs_ atau rme_)
-- Dasar: docs/05-physical-schema.md §12 (T-01, T-02, T-03, T-04, T-24), §2.1 butir 5, CR-020.

SELECT pg_catalog.set_config('uji.skema_harap', :'skema_harap', false) AS _a,
       pg_catalog.set_config('uji.skema_proyek', :'skema_proyek', false) AS _b,
       pg_catalog.set_config('uji.skema_mig', :'skema_mig', false) AS _c,
       pg_catalog.set_config('uji.prefiks', :'prefiks', false) AS _d \gset

DO $$
DECLARE
  db text := current_database();
  proyek text[] := current_setting('uji.skema_proyek')::text[];
BEGIN
  PERFORM pg_temp.uji_nol(format('T-01 (%s) nama objek: prefiks, snake_case, <= 60 byte', db),
    ARRAY(SELECT pg_temp.uji_t01(proyek)));
  PERFORM pg_temp.uji_nol(format('T-02 (%s) skema persis %s, public tidak ada, _sqlx_migrations di %s, ext hanya objek ekstensi',
                                 db, current_setting('uji.skema_harap'), current_setting('uji.skema_mig')),
    ARRAY(SELECT pg_temp.uji_t02(current_setting('uji.skema_harap')::text[], current_setting('uji.skema_mig'))));
  PERFORM pg_temp.uji_nol(format('T-03 (%s) PUBLIC tanpa EXECUTE fungsi/prosedur proyek dan tanpa CONNECT/TEMP', db),
    ARRAY(SELECT pg_temp.uji_t03(proyek)));
  PERFORM pg_temp.uji_nol(format('T-04 (%s) semua relasi, rutin, tipe (termasuk _sqlx_migrations) milik pemilik skema', db),
    ARRAY(SELECT pg_temp.uji_t04(proyek)));
  PERFORM pg_temp.uji_nol(format('T-24 (%s) badan rutin dan argumen trigger hanya menyebut role %s*', db, current_setting('uji.prefiks')),
    ARRAY(SELECT pg_temp.uji_t24(proyek, current_setting('uji.prefiks'))));
  PERFORM pg_temp.uji_nol(format('CR-020 (%s) rutin proyek: SECURITY DEFINER hanya yang diizinkan, search_path tetap', db),
    ARRAY(SELECT pg_temp.uji_rutin(proyek)));
END $$;

-- Tabel catatan migrasi sqlx: semua migrasi tercatat berhasil (tanpa migrasi kotor/gagal).
DO $$
DECLARE
  n_gagal bigint;
BEGIN
  EXECUTE format('SELECT count(*) FROM %I._sqlx_migrations WHERE NOT success', current_setting('uji.skema_mig')) INTO n_gagal;
  IF n_gagal <> 0 THEN
    RAISE EXCEPTION 'GAGAL _sqlx_migrations (%): % migrasi tercatat gagal', current_database(), n_gagal;
  END IF;
  RAISE NOTICE 'LULUS _sqlx_migrations (%) tanpa migrasi gagal/kotor', current_database();
END $$;
