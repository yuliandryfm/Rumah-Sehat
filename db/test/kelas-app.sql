-- db/test/kelas-app.sql: test katalog kelas data dan trigger penolak database ruang_sehat
-- (T-06, T-07, T-08, T-09, T-22) atas skema app nyata dan atas tabel fixture, ditambah test negatif
-- pemeriksa T-20 (CHECK *_enc/*_bidx, F1-T18) atas fixture; T-20 skema nyata di db/test/katalog.sql.
-- Didahului db/test/lib.sql; dijalankan superuser di database ruang_sehat setelah migrasi.
--
-- Fixture dibuat di skema uji_kelas (milik rs_migration) dan, bila belum ada, fixture minimal
-- app.retention_policy, semuanya di dalam SATU transaksi yang di-ROLLBACK di akhir berkas,
-- sehingga tidak ada sisa di database. Perilaku diuji dengan SET LOCAL ROLE ke role sungguhan
-- (pemeriksaan hak memakai current_user).
-- Dasar: docs/05-physical-schema.md §4.1-§4.11, §11.4, §12; DD §0.3, §0.6 A; ADR-004, ADR-008, ADR-013 §5.

-- Daftar tabel pola berlaku-mulai (§4.2 (3)).
\set berlaku '{markup_rule,partner_fee_rule,consultation_rate,business_parameter,payment_method_fee,retention_policy,consent_text_version}'

-- A. Skema app nyata: semua tabel memenuhi aturan kelas (tabel bisnis sejak F1-T16; test per tabel
--    di db/test/tabel-app-*.sql).
SELECT pg_catalog.set_config('uji.berlaku', :'berlaku', false) AS _b \gset
DO $$
DECLARE
  n integer;
BEGIN
  SELECT count(*) INTO n FROM pg_class c JOIN pg_namespace s ON s.oid = c.relnamespace
   WHERE s.nspname = 'app' AND c.relkind IN ('r', 'p') AND c.relname <> '_sqlx_migrations';
  PERFORM pg_temp.uji_nol(format('T-06/T-07/T-08 (ruang_sehat) %s tabel di skema app sesuai aturan kelas', n),
    ARRAY(SELECT pg_temp.uji_kelas_app('app', current_setting('uji.berlaku')::text[])));
  -- T-22: snapshot tabel L yang dirujuk FK dari tabel non-L (perubahan wajib review bersama PT-24).
  IF pg_temp.uji_t22('app') IS DISTINCT FROM '{}'::text[] THEN
    RAISE EXCEPTION 'GAGAL T-22 tabel L dengan FK masuk dari non-L berubah: % (perbarui snapshot setelah review)', pg_temp.uji_t22('app');
  END IF;
  RAISE NOTICE 'LULUS T-22 (ruang_sehat) snapshot tabel L dengan FK masuk dari non-L: {}';
END $$;

-- B. Fixture per kelas, dibuat pemilik skema seperti migrasi sungguhan.
BEGIN;
CREATE SCHEMA uji_kelas AUTHORIZATION rs_migration;
GRANT USAGE ON SCHEMA uji_kelas TO rs_app, rs_audit_relay, rs_privacy_worker, rs_key_rotation;
SET LOCAL ROLE rs_migration;

CREATE TABLE uji_kelas.uji_m (
  id uuid NOT NULL, created_at timestamptz NOT NULL DEFAULT now(), created_by uuid NULL,
  updated_at timestamptz NOT NULL DEFAULT now(), updated_by uuid NULL,
  deleted_at timestamptz NULL, deleted_by uuid NULL, delete_reason text NULL,
  full_name text NULL, anonymized_at timestamptz NULL,
  nik_enc bytea NULL, nik_bidx bytea NULL, nik_bidx_key_version smallint NULL,
  CONSTRAINT pk_uji_m PRIMARY KEY (id),
  CONSTRAINT ck_uji_m_nik_enc_format CHECK (nik_enc IS NULL OR (octet_length(nik_enc) >= 29 AND get_byte(nik_enc, 0) = 1)),
  CONSTRAINT ck_uji_m_nik_bidx CHECK (octet_length(nik_bidx) = 32),
  CONSTRAINT ck_uji_m_nik_bidx_key_version CHECK (nik_bidx_key_version >= 1));
COMMENT ON TABLE uji_kelas.uji_m IS 'class:M module:uji';
COMMENT ON COLUMN uji_kelas.uji_m.full_name IS 'anonymize';
REVOKE ALL ON uji_kelas.uji_m FROM PUBLIC;
GRANT SELECT, INSERT, UPDATE ON uji_kelas.uji_m TO rs_app;
GRANT SELECT ON uji_kelas.uji_m TO rs_privacy_worker;
GRANT UPDATE (full_name, anonymized_at, updated_at, updated_by) ON uji_kelas.uji_m TO rs_privacy_worker;

CREATE TABLE uji_kelas.uji_m_berlaku (
  id uuid NOT NULL, created_at timestamptz NOT NULL DEFAULT now(), created_by uuid NULL,
  updated_at timestamptz NOT NULL DEFAULT now(), updated_by uuid NULL,
  deleted_at timestamptz NULL, deleted_by uuid NULL, delete_reason text NULL,
  effective_from timestamptz NOT NULL, nilai integer NOT NULL,
  CONSTRAINT pk_uji_m_berlaku PRIMARY KEY (id));
COMMENT ON TABLE uji_kelas.uji_m_berlaku IS 'class:M module:uji';
GRANT SELECT, INSERT, UPDATE ON uji_kelas.uji_m_berlaku TO rs_app;
GRANT SELECT ON uji_kelas.uji_m_berlaku TO rs_privacy_worker;
CREATE TRIGGER tg_uji_m_berlaku_effective_lock BEFORE UPDATE ON uji_kelas.uji_m_berlaku
  FOR EACH ROW EXECUTE FUNCTION util.fn_effective_row_lock();
CREATE TRIGGER tg_uji_m_berlaku_no_backdate BEFORE INSERT ON uji_kelas.uji_m_berlaku
  FOR EACH ROW EXECUTE FUNCTION util.fn_effective_row_no_backdate('rs_migration');

CREATE TABLE uji_kelas.uji_t (
  id uuid NOT NULL, created_at timestamptz NOT NULL DEFAULT now(), created_by uuid NULL,
  subject_id uuid NOT NULL, data_key_id uuid NULL, payload text NULL,
  CONSTRAINT pk_uji_t PRIMARY KEY (id),
  CONSTRAINT uq_uji_t_subject_id UNIQUE (subject_id));
COMMENT ON TABLE uji_kelas.uji_t IS 'class:T module:uji';
REVOKE ALL ON uji_kelas.uji_t FROM PUBLIC;
GRANT SELECT, INSERT ON uji_kelas.uji_t TO rs_app;
GRANT SELECT (id, created_at, subject_id, data_key_id) ON uji_kelas.uji_t TO rs_privacy_worker;
CREATE TRIGGER tg_uji_t_reject_ud BEFORE UPDATE OR DELETE ON uji_kelas.uji_t
  FOR EACH STATEMENT EXECUTE FUNCTION util.fn_reject_modification();
CREATE TRIGGER tg_uji_t_reject_truncate BEFORE TRUNCATE ON uji_kelas.uji_t
  FOR EACH STATEMENT EXECUTE FUNCTION util.fn_reject_modification();

CREATE TABLE uji_kelas.uji_r (
  id uuid NOT NULL, uji_t_id uuid NOT NULL, version integer NOT NULL, to_status text NOT NULL,
  note_data_key_id uuid NULL, created_at timestamptz NOT NULL DEFAULT now(),
  CONSTRAINT pk_uji_r PRIMARY KEY (id),
  CONSTRAINT fk_uji_r_uji_t_id FOREIGN KEY (uji_t_id) REFERENCES uji_kelas.uji_t (id),
  CONSTRAINT uq_uji_r_uji_t_id_version UNIQUE (uji_t_id, version));
COMMENT ON TABLE uji_kelas.uji_r IS 'class:R module:uji';
GRANT SELECT, INSERT ON uji_kelas.uji_r TO rs_app;
GRANT SELECT (id, note_data_key_id) ON uji_kelas.uji_r TO rs_privacy_worker;
CREATE TRIGGER tg_uji_r_reject_ud BEFORE UPDATE OR DELETE ON uji_kelas.uji_r
  FOR EACH STATEMENT EXECUTE FUNCTION util.fn_reject_modification();
CREATE TRIGGER tg_uji_r_reject_truncate BEFORE TRUNCATE ON uji_kelas.uji_r
  FOR EACH STATEMENT EXECUTE FUNCTION util.fn_reject_modification();

CREATE TABLE uji_kelas.uji_p (
  uji_t_id uuid NOT NULL, status text NOT NULL, version integer NOT NULL,
  updated_at timestamptz NOT NULL DEFAULT now(),
  CONSTRAINT pk_uji_p PRIMARY KEY (uji_t_id));
COMMENT ON TABLE uji_kelas.uji_p IS 'class:P module:uji';
GRANT SELECT, INSERT, UPDATE ON uji_kelas.uji_p TO rs_app;

CREATE TABLE uji_kelas.uji_o (
  id uuid NOT NULL, created_at timestamptz NOT NULL DEFAULT now(), expires_at timestamptz NOT NULL,
  CONSTRAINT pk_uji_o PRIMARY KEY (id));
CREATE INDEX ix_uji_o_expires_at ON uji_kelas.uji_o (expires_at);
COMMENT ON TABLE uji_kelas.uji_o IS 'class:O module:uji';
GRANT SELECT, INSERT, UPDATE, DELETE ON uji_kelas.uji_o TO rs_app;
GRANT SELECT, DELETE ON uji_kelas.uji_o TO rs_privacy_worker;

CREATE TABLE uji_kelas.uji_l (
  id uuid NOT NULL, created_at timestamptz NOT NULL DEFAULT now(), subject_id uuid NOT NULL, isi text NULL,
  CONSTRAINT pk_uji_l PRIMARY KEY (id));
COMMENT ON TABLE uji_kelas.uji_l IS 'class:L module:uji log_category:uji_kategori';
GRANT SELECT, INSERT ON uji_kelas.uji_l TO rs_app;
GRANT SELECT (id, created_at, subject_id) ON uji_kelas.uji_l TO rs_privacy_worker;
GRANT DELETE ON uji_kelas.uji_l TO rs_privacy_worker;
CREATE TRIGGER tg_uji_l_reject_update BEFORE UPDATE ON uji_kelas.uji_l
  FOR EACH STATEMENT EXECUTE FUNCTION util.fn_reject_modification();
CREATE TRIGGER tg_uji_l_reject_truncate BEFORE TRUNCATE ON uji_kelas.uji_l
  FOR EACH STATEMENT EXECUTE FUNCTION util.fn_reject_modification();
CREATE TRIGGER tg_uji_l_guard_delete BEFORE DELETE ON uji_kelas.uji_l
  FOR EACH ROW EXECUTE FUNCTION util.fn_guard_log_delete('uji_kategori', 'created_at', '{delete_row}', 'rs_privacy_worker');

CREATE TABLE uji_kelas.uji_mk (
  id uuid NOT NULL, created_at timestamptz NOT NULL DEFAULT now(),
  container_type text NOT NULL, container_id uuid NOT NULL,
  retention_category text NULL, retention_anchor_at timestamptz NULL,
  wrapped_dek bytea NULL, kek_ref text NOT NULL, shredded_at timestamptz NULL, shred_reason text NULL,
  CONSTRAINT pk_uji_mk PRIMARY KEY (id),
  CONSTRAINT ck_uji_mk_shred CHECK ((wrapped_dek IS NULL) = (shredded_at IS NOT NULL)));
COMMENT ON TABLE uji_kelas.uji_mk IS 'class:MK module:uji';
GRANT SELECT, INSERT ON uji_kelas.uji_mk TO rs_app;
GRANT SELECT ON uji_kelas.uji_mk TO rs_privacy_worker;
GRANT UPDATE (wrapped_dek, shredded_at, shred_reason) ON uji_kelas.uji_mk TO rs_privacy_worker;
GRANT SELECT (id, wrapped_dek, kek_ref, shredded_at) ON uji_kelas.uji_mk TO rs_key_rotation;
GRANT UPDATE (wrapped_dek, kek_ref) ON uji_kelas.uji_mk TO rs_key_rotation;
CREATE TRIGGER tg_uji_mk_one_way BEFORE UPDATE ON uji_kelas.uji_mk
  FOR EACH ROW EXECUTE FUNCTION util.fn_data_key_one_way('rs_privacy_worker', 'rs_key_rotation');
CREATE TRIGGER tg_uji_mk_reject_delete BEFORE DELETE ON uji_kelas.uji_mk
  FOR EACH STATEMENT EXECUTE FUNCTION util.fn_reject_modification();
CREATE TRIGGER tg_uji_mk_reject_truncate BEFORE TRUNCATE ON uji_kelas.uji_mk
  FOR EACH STATEMENT EXECUTE FUNCTION util.fn_reject_modification();

CREATE TABLE uji_kelas.uji_oh (
  id uuid NOT NULL, created_at timestamptz NOT NULL DEFAULT now(), payload text NOT NULL,
  CONSTRAINT pk_uji_oh PRIMARY KEY (id));
COMMENT ON TABLE uji_kelas.uji_oh IS 'class:OH module:uji';
GRANT INSERT ON uji_kelas.uji_oh TO rs_app, rs_privacy_worker;
GRANT SELECT, DELETE ON uji_kelas.uji_oh TO rs_audit_relay;
RESET ROLE;

-- Fixture minimal app.retention_policy (DD A6.6) hanya bila tabel nyata belum ada. Bila sudah ada,
-- berkas ini GAGAL supaya test penjaga Log diperbarui ke tabel nyata (tidak dilewati diam-diam).
DO $$
BEGIN
  IF to_regclass('app.retention_policy') IS NOT NULL THEN
    RAISE EXCEPTION 'GAGAL fixture: app.retention_policy sudah ada; perbarui db/test/kelas-app.sql bagian T-09 penjaga Log ke tabel nyata';
  END IF;
END $$;
SET LOCAL ROLE rs_migration;
CREATE TABLE app.retention_policy (
  id uuid NOT NULL, data_category text NOT NULL, retention_months integer NOT NULL, end_action text NOT NULL,
  effective_from timestamptz NOT NULL, deleted_at timestamptz NULL,
  CONSTRAINT pk_retention_policy PRIMARY KEY (id));
GRANT SELECT ON app.retention_policy TO rs_privacy_worker;
RESET ROLE;

SELECT pg_temp.uji_nol('T-06/T-07/T-08 fixture sembilan kelas (M, M berlaku-mulai, T, R, P, O, L, MK, OH) lulus pemeriksa',
  ARRAY(SELECT pg_temp.uji_kelas_app('uji_kelas', '{uji_m_berlaku}')));
SELECT pg_temp.uji_nol('T-01 fixture: nama objek sesuai pola', ARRAY(SELECT pg_temp.uji_t01('{uji_kelas}')));
SELECT pg_temp.uji_nol('T-04 fixture: objek milik rs_migration', ARRAY(SELECT pg_temp.uji_t04('{uji_kelas}')));
SELECT pg_temp.uji_nol('T-24 fixture: argumen trigger hanya role rs_*', ARRAY(SELECT pg_temp.uji_t24('{uji_kelas}', 'rs_')));
SELECT pg_temp.uji_nol('T-20 fixture: kolom *_enc/*_bidx/*_bidx_key_version uji_m sesuai (3 kolom)',
  ARRAY(SELECT pg_temp.uji_t20('{uji_kelas}')
        UNION ALL
        SELECT format('jumlah kolom T-20 fixture %s, diharapkan 3', count(*)) FROM pg_temp.uji_t20_kolom('{uji_kelas}') HAVING count(*) <> 3));

-- C. Test negatif atas pemeriksa: kerusakan yang disengaja wajib terdeteksi.
DO $$
DECLARE
  c record;
  kelas text := $q$SELECT pg_temp.uji_kelas_app('uji_kelas', '{uji_m_berlaku}')$q$;
BEGIN
  FOR c IN SELECT * FROM (VALUES
      ('T-06 UPDATE di tabel T untuk rs_app', 'GRANT UPDATE ON uji_kelas.uji_t TO rs_app', kelas, 'T-06 uji_t .*lebih rs_app:UPDATE'),
      ('T-06 DELETE di tabel M untuk rs_app', 'GRANT DELETE ON uji_kelas.uji_m TO rs_app', kelas, 'T-06 uji_m .*lebih rs_app:DELETE'),
      ('T-06 INSERT tabel R dicabut', 'REVOKE INSERT ON uji_kelas.uji_r FROM rs_app', kelas, 'T-06 uji_r .*kurang rs_app:INSERT'),
      ('T-06 hak ke role login (bukan grup)', 'GRANT SELECT ON uji_kelas.uji_p TO rs_api', kelas, 'T-06 uji_p .*lebih rs_api:SELECT'),
      ('T-06 TRUNCATE ke rs_app', 'GRANT TRUNCATE ON uji_kelas.uji_o TO rs_app', kelas, 'T-06 uji_o .*lebih rs_app:TRUNCATE'),
      ('T-06 hak ke PUBLIC', 'GRANT SELECT ON uji_kelas.uji_l TO PUBLIC', kelas, 'T-06 uji_l .*lebih PUBLIC:SELECT'),
      ('T-06 UPDATE outbox untuk rs_app', 'GRANT SELECT ON uji_kelas.uji_oh TO rs_app', kelas, 'T-06 uji_oh .*lebih rs_app:SELECT'),
      ('T-06 komentar kelas hilang', 'COMMENT ON TABLE uji_kelas.uji_p IS NULL', kelas, 'T-06 uji_p: komentar kelas tidak sah'),
      ('T-06 SELECT kolom isi T untuk pekerja privasi', 'GRANT SELECT (payload) ON uji_kelas.uji_t TO rs_privacy_worker', kelas, 'T-06 uji_t: SELECT per kolom'),
      ('T-07 UPDATE kolom bukan anonymize', 'GRANT UPDATE (delete_reason) ON uji_kelas.uji_m TO rs_privacy_worker', kelas, 'T-07 uji_m'),
      ('T-07 UPDATE kek_ref oleh pekerja privasi', 'GRANT UPDATE (kek_ref) ON uji_kelas.uji_mk TO rs_privacy_worker', kelas, 'T-07 uji_mk'),
      ('T-06 hak per kolom rs_app', 'GRANT UPDATE (payload) ON uji_kelas.uji_t TO rs_app', kelas, 'hak per kolom tidak diizinkan rs_app:UPDATE'),
      ('T-08 trigger reject_ud T hilang', 'DROP TRIGGER tg_uji_t_reject_ud ON uji_kelas.uji_t', kelas, 'T-08 uji_t \(kelas T\)'),
      ('T-08 trigger reject_truncate R nonaktif', 'ALTER TABLE uji_kelas.uji_r DISABLE TRIGGER tg_uji_r_reject_truncate', kelas, 'T-08 uji_r: trigger tg_uji_r_reject_truncate tidak aktif'),
      ('T-08 semua trigger L nonaktif', 'ALTER TABLE uji_kelas.uji_l DISABLE TRIGGER ALL', kelas, 'T-08 uji_l .*NONAKTIF'),
      ('T-08 guard_delete tanpa daftar end_action', 'DROP TRIGGER tg_uji_l_guard_delete ON uji_kelas.uji_l; CREATE TRIGGER tg_uji_l_guard_delete BEFORE DELETE ON uji_kelas.uji_l FOR EACH ROW EXECUTE FUNCTION util.fn_guard_log_delete(''uji_kategori'', ''created_at'')', kelas, 'T-08 uji_l: argumen tg_uji_l_guard_delete'),
      ('T-08 one_way dengan role salah', 'DROP TRIGGER tg_uji_mk_one_way ON uji_kelas.uji_mk; CREATE TRIGGER tg_uji_mk_one_way BEFORE UPDATE ON uji_kelas.uji_mk FOR EACH ROW EXECUTE FUNCTION util.fn_data_key_one_way(''rs_app'', ''rs_key_rotation'')', kelas, 'T-08 uji_mk: argumen tg_uji_mk_one_way'),
      ('T-08 trigger penolak T tingkat baris (bukan statement)', 'DROP TRIGGER tg_uji_t_reject_ud ON uji_kelas.uji_t; CREATE TRIGGER tg_uji_t_reject_ud BEFORE UPDATE OR DELETE ON uji_kelas.uji_t FOR EACH ROW EXECUTE FUNCTION util.fn_reject_modification()', kelas, 'T-08 uji_t \(kelas T\)'),
      ('T-08 M dengan effective_from tidak terdaftar', 'ALTER TABLE uji_kelas.uji_m ADD COLUMN effective_from timestamptz', kelas, 'T-08 uji_m: kolom effective_from'),
      ('T-08 effective_lock hilang', 'DROP TRIGGER tg_uji_m_berlaku_effective_lock ON uji_kelas.uji_m_berlaku', kelas, 'T-08 uji_m_berlaku \(kelas M\)'),
      ('T-24 argumen trigger role sistem lain', 'DROP TRIGGER tg_uji_l_guard_delete ON uji_kelas.uji_l; CREATE TRIGGER tg_uji_l_guard_delete BEFORE DELETE ON uji_kelas.uji_l FOR EACH ROW EXECUTE FUNCTION util.fn_guard_log_delete(''uji_kategori'', ''created_at'', ''{}'', ''rme_privacy_worker'')',
         $q$SELECT pg_temp.uji_t24('{uji_kelas}', 'rs_')$q$, 'T-24 argumen trigger uji_l.tg_uji_l_guard_delete: rme_privacy_worker'),
      ('T-01 nama tabel 61 byte', 'CREATE TABLE uji_kelas.uji_nama_yang_sangat_panjang_sekali_melebihi_batas_proyek_6x1 (x int)',
         $q$SELECT pg_temp.uji_t01('{uji_kelas}')$q$, 'lebih dari 60 byte'),
      ('T-01 index tanpa prefiks', 'CREATE INDEX uji_o_created_at_idx ON uji_kelas.uji_o (created_at)', $q$SELECT pg_temp.uji_t01('{uji_kelas}')$q$, 'index tanpa prefiks ix_'),
      ('T-01 check tanpa prefiks', 'ALTER TABLE uji_kelas.uji_o ADD CONSTRAINT uji_o_cek CHECK (expires_at > created_at)', $q$SELECT pg_temp.uji_t01('{uji_kelas}')$q$, 'check tanpa prefiks ck_'),
      ('T-01 sequence dilarang', 'CREATE SEQUENCE uji_kelas.uji_seq', $q$SELECT pg_temp.uji_t01('{uji_kelas}')$q$, 'sequence dilarang'),
      ('T-03 PUBLIC EXECUTE fungsi util', 'GRANT EXECUTE ON FUNCTION util.fn_reject_modification() TO PUBLIC', $q$SELECT pg_temp.uji_t03('{app,util}')$q$, 'T-03 util.fn_reject_modification\(\): PUBLIC EXECUTE'),
      ('T-03 fungsi baru superuser dengan ACL bawaan', 'CREATE FUNCTION util.fn_uji_bawaan() RETURNS int LANGUAGE sql AS ''SELECT 1''', $q$SELECT pg_temp.uji_t03('{app,util}')$q$, 'ACL bawaan'),
      ('T-04 objek milik role lain', 'CREATE TABLE app.uji_salah_pemilik (x int)', $q$SELECT pg_temp.uji_t04('{app,util}')$q$, 'T-04 relasi app.uji_salah_pemilik dimiliki'),
      ('CR-020 fungsi SECURITY DEFINER', 'SET LOCAL ROLE rs_migration; CREATE FUNCTION util.fn_uji_definer() RETURNS int LANGUAGE sql SECURITY DEFINER SET search_path = pg_catalog, pg_temp AS ''SELECT 1''; RESET ROLE',
         $q$SELECT pg_temp.uji_rutin('{app,util}')$q$, 'SECURITY DEFINER tidak diizinkan'),
      ('CR-020 fungsi dengan search_path skema bisnis (bukan persis pg_catalog, pg_temp)', 'SET LOCAL ROLE rs_migration; CREATE FUNCTION util.fn_uji_path_app() RETURNS int LANGUAGE sql SET search_path = pg_catalog, app AS ''SELECT 1''; RESET ROLE',
         $q$SELECT pg_temp.uji_rutin('{app,util}')$q$, 'fn_uji_path_app\(\): SET search_path bukan persis'),
      ('CR-020 fungsi dengan search_path pg_catalog saja (pg_temp tidak terakhir)', 'SET LOCAL ROLE rs_migration; CREATE FUNCTION util.fn_uji_path_cat() RETURNS int LANGUAGE sql SET search_path = pg_catalog AS ''SELECT 1''; RESET ROLE',
         $q$SELECT pg_temp.uji_rutin('{app,util}')$q$, 'fn_uji_path_cat\(\): SET search_path bukan persis'),
      ('CR-020 fungsi tanpa search_path', 'SET LOCAL ROLE rs_migration; CREATE FUNCTION util.fn_uji_tanpa_path() RETURNS int LANGUAGE sql AS ''SELECT 1''; RESET ROLE',
         $q$SELECT pg_temp.uji_rutin('{app,util}')$q$, 'fn_uji_tanpa_path\(\): SET search_path bukan persis'),
      ('T-24 badan fungsi menyebut role sistem lain', 'SET LOCAL ROLE rs_migration; CREATE FUNCTION util.fn_uji_role() RETURNS boolean LANGUAGE sql SET search_path = pg_catalog AS ''SELECT pg_has_role(''''rme_app'''', ''''MEMBER'''')''; RESET ROLE',
         $q$SELECT pg_temp.uji_t24('{app,util}', 'rs_')$q$, 'T-24 badan util.fn_uji_role\(\) menyebut role rme_app'),
      ('T-20 CHECK amplop *_enc dihapus', 'ALTER TABLE uji_kelas.uji_m DROP CONSTRAINT ck_uji_m_nik_enc_format', $q$SELECT pg_temp.uji_t20('{uji_kelas}')$q$, 'T-20 uji_kelas.uji_m.nik_enc: tanpa CHECK'),
      ('T-20 CHECK amplop *_enc dilonggarkan (>= 1 byte)', 'ALTER TABLE uji_kelas.uji_m DROP CONSTRAINT ck_uji_m_nik_enc_format; ALTER TABLE uji_kelas.uji_m ADD CONSTRAINT ck_uji_m_nik_enc_format CHECK (nik_enc IS NULL OR octet_length(nik_enc) >= 1)', $q$SELECT pg_temp.uji_t20('{uji_kelas}')$q$, 'T-20 uji_kelas.uji_m.nik_enc: tanpa CHECK'),
      ('T-20 CHECK amplop *_enc tanpa byte versi', 'ALTER TABLE uji_kelas.uji_m DROP CONSTRAINT ck_uji_m_nik_enc_format; ALTER TABLE uji_kelas.uji_m ADD CONSTRAINT ck_uji_m_nik_enc_format CHECK (nik_enc IS NULL OR octet_length(nik_enc) >= 29)', $q$SELECT pg_temp.uji_t20('{uji_kelas}')$q$, 'T-20 uji_kelas.uji_m.nik_enc: tanpa CHECK'),
      ('T-20 CHECK amplop *_enc NOT VALID', 'ALTER TABLE uji_kelas.uji_m DROP CONSTRAINT ck_uji_m_nik_enc_format; ALTER TABLE uji_kelas.uji_m ADD CONSTRAINT ck_uji_m_nik_enc_format CHECK (nik_enc IS NULL OR (octet_length(nik_enc) >= 29 AND get_byte(nik_enc, 0) = 1)) NOT VALID', $q$SELECT pg_temp.uji_t20('{uji_kelas}')$q$, 'T-20 uji_kelas.uji_m.nik_enc: tanpa CHECK'),
      ('T-20 CHECK amplop *_enc digabung kolom lain', 'ALTER TABLE uji_kelas.uji_m DROP CONSTRAINT ck_uji_m_nik_enc_format; ALTER TABLE uji_kelas.uji_m ADD CONSTRAINT ck_uji_m_nik_enc_format CHECK (full_name IS NOT NULL OR nik_enc IS NULL OR (octet_length(nik_enc) >= 29 AND get_byte(nik_enc, 0) = 1))', $q$SELECT pg_temp.uji_t20('{uji_kelas}')$q$, 'T-20 uji_kelas.uji_m.nik_enc: tanpa CHECK'),
      ('T-20 kolom *_enc baru tanpa CHECK', 'ALTER TABLE uji_kelas.uji_t ADD COLUMN payload_enc bytea NULL', $q$SELECT pg_temp.uji_t20('{uji_kelas}')$q$, 'T-20 uji_kelas.uji_t.payload_enc: tanpa CHECK'),
      ('T-20 kolom *_enc bertipe text', 'ALTER TABLE uji_kelas.uji_r ADD COLUMN note_enc text NULL', $q$SELECT pg_temp.uji_t20('{uji_kelas}')$q$, 'T-20 uji_kelas.uji_r.note_enc: tipe text, diharapkan bytea'),
      ('T-20 CHECK *_bidx 16 byte (dipotong)', 'ALTER TABLE uji_kelas.uji_m DROP CONSTRAINT ck_uji_m_nik_bidx; ALTER TABLE uji_kelas.uji_m ADD CONSTRAINT ck_uji_m_nik_bidx CHECK (octet_length(nik_bidx) = 16)', $q$SELECT pg_temp.uji_t20('{uji_kelas}')$q$, 'T-20 uji_kelas.uji_m.nik_bidx: tanpa CHECK'),
      ('T-20 kolom *_bidx baru tanpa CHECK', 'ALTER TABLE uji_kelas.uji_m ADD COLUMN code_bidx bytea NULL', $q$SELECT pg_temp.uji_t20('{uji_kelas}')$q$, 'T-20 uji_kelas.uji_m.code_bidx: tanpa CHECK'),
      ('T-20 versi kunci blind index boleh 0', 'ALTER TABLE uji_kelas.uji_m DROP CONSTRAINT ck_uji_m_nik_bidx_key_version; ALTER TABLE uji_kelas.uji_m ADD CONSTRAINT ck_uji_m_nik_bidx_key_version CHECK (nik_bidx_key_version >= 0)', $q$SELECT pg_temp.uji_t20('{uji_kelas}')$q$, 'T-20 uji_kelas.uji_m.nik_bidx_key_version: tanpa CHECK'),
      ('T-20 versi kunci blind index bertipe integer', 'ALTER TABLE uji_kelas.uji_m ALTER COLUMN nik_bidx_key_version TYPE integer', $q$SELECT pg_temp.uji_t20('{uji_kelas}')$q$, 'T-20 uji_kelas.uji_m.nik_bidx_key_version: tipe integer, diharapkan smallint'),
      ('T-02 skema liar', 'CREATE SCHEMA uji_liar', $q$SELECT pg_temp.uji_t02('{app,util,ext,uji_kelas}', 'app')$q$, 'T-02 skema tidak terdaftar: uji_liar'),
      ('T-02 objek liar di ext', 'CREATE TABLE ext.uji_liar (x int)', $q$SELECT pg_temp.uji_t02('{app,util,ext,uji_kelas}', 'app')$q$, 'objek bukan milik ekstensi di skema ext: uji_liar')
    ) AS t(label, stmt, pemeriksa, pola)
  LOOP
    PERFORM pg_temp.uji_rusak(c.label, c.stmt, c.pemeriksa, c.pola);
  END LOOP;
END $$;

-- D. T-09 perilaku dengan role sungguhan. Data sintetis.
SET LOCAL ROLE rs_migration;
INSERT INTO uji_kelas.uji_t (id, subject_id, payload)
  VALUES ('00000000-0000-7000-8000-000000000001', '00000000-0000-7000-8000-0000000000a1', 'isi sintetis');
INSERT INTO uji_kelas.uji_r (id, uji_t_id, version, to_status)
  VALUES ('00000000-0000-7000-8000-000000000011', '00000000-0000-7000-8000-000000000001', 1, 'dibuat');
INSERT INTO uji_kelas.uji_l (id, created_at, subject_id)
  VALUES ('00000000-0000-7000-8000-000000000021', now() - interval '3 months', '00000000-0000-7000-8000-0000000000a1'),
         ('00000000-0000-7000-8000-000000000022', now(), '00000000-0000-7000-8000-0000000000a1');
INSERT INTO uji_kelas.uji_mk (id, container_type, container_id, wrapped_dek, kek_ref)
  VALUES ('00000000-0000-7000-8000-000000000031', 'uji_wadah', '00000000-0000-7000-8000-0000000000b1', '\x0102', 'openbao-transit:uji:k:v1'),
         ('00000000-0000-7000-8000-000000000032', 'uji_wadah', '00000000-0000-7000-8000-0000000000b2', '\x0304', 'openbao-transit:uji:k:v1');
RESET ROLE;

-- Kelas T dan R: role aplikasi tidak punya hak UPDATE/DELETE/TRUNCATE (ditolak sebelum trigger).
SELECT pg_temp.uji_tolak('T-09 NEG rs_api UPDATE tabel T', 'rs_api', 'UPDATE uji_kelas.uji_t SET payload = ''x''', '42501', 'permission denied for table uji_t');
SELECT pg_temp.uji_tolak('T-09 NEG rs_api DELETE tabel T', 'rs_api', 'DELETE FROM uji_kelas.uji_t', '42501', 'permission denied for table uji_t');
SELECT pg_temp.uji_tolak('T-09 NEG rs_api TRUNCATE tabel T', 'rs_api', 'TRUNCATE uji_kelas.uji_t CASCADE', '42501', 'permission denied for table');
SELECT pg_temp.uji_tolak('T-09 NEG rs_worker UPDATE tabel R', 'rs_worker', 'UPDATE uji_kelas.uji_r SET to_status = ''x''', '42501', 'permission denied for table uji_r');
SELECT pg_temp.uji_tolak('T-09 NEG rs_api DELETE tabel R', 'rs_api', 'DELETE FROM uji_kelas.uji_r', '42501', 'permission denied for table uji_r');
SELECT pg_temp.uji_boleh('T-09 rs_api INSERT tabel T (append-only tetap dapat ditambah)', 'rs_api',
  'INSERT INTO uji_kelas.uji_t (id, subject_id) VALUES (''00000000-0000-7000-8000-000000000002'', ''00000000-0000-7000-8000-0000000000a2'')', 1);
-- Pemilik skema pun ditolak trigger (ADR-004, §4.5, §11.4).
SELECT pg_temp.uji_tolak('T-09 NEG pemilik UPDATE tabel T ditolak trigger', 'rs_migration', 'UPDATE uji_kelas.uji_t SET payload = ''x''', '42501', '^immutable_violation: UPDATE on uji_kelas.uji_t rejected$');
SELECT pg_temp.uji_tolak('T-09 NEG pemilik DELETE tabel T ditolak trigger', 'rs_migration', 'DELETE FROM uji_kelas.uji_t', '42501', '^immutable_violation: DELETE on uji_kelas.uji_t rejected$');
SELECT pg_temp.uji_tolak('T-09 NEG pemilik TRUNCATE tabel R ditolak trigger', 'rs_migration', 'TRUNCATE uji_kelas.uji_r', '42501', '^immutable_violation: TRUNCATE on uji_kelas.uji_r rejected$');
SELECT pg_temp.uji_tolak('T-09 NEG pemilik UPDATE nol baris tetap ditolak (trigger statement)', 'rs_migration', 'UPDATE uji_kelas.uji_r SET to_status = ''x'' WHERE false', '42501', '^immutable_violation: UPDATE on uji_kelas.uji_r rejected$');
SELECT pg_temp.uji_tolak('T-09 NEG INSERT ON CONFLICT DO UPDATE di tabel T ditolak (trigger BEFORE UPDATE statement)', 'rs_migration',
  'INSERT INTO uji_kelas.uji_t (id, subject_id) VALUES (''00000000-0000-7000-8000-000000000003'', ''00000000-0000-7000-8000-0000000000a3'') ON CONFLICT (subject_id) DO UPDATE SET payload = ''x''',
  '42501', '^immutable_violation: UPDATE on uji_kelas.uji_t rejected$');
SELECT pg_temp.uji_boleh('T-09 INSERT ON CONFLICT DO NOTHING di tabel T tetap dapat (idempotensi)', 'rs_api',
  'INSERT INTO uji_kelas.uji_t (id, subject_id) VALUES (''00000000-0000-7000-8000-000000000004'', ''00000000-0000-7000-8000-0000000000a1'') ON CONFLICT (subject_id) DO NOTHING', 0);

-- Kelas L: hanya pekerja privasi setelah masa retensi habis (§4.2 (2), §4.9, ADR-013 §5).
SELECT pg_temp.uji_tolak('T-09 NEG rs_api UPDATE tabel L', 'rs_api', 'UPDATE uji_kelas.uji_l SET isi = ''x''', '42501', 'permission denied for table uji_l');
SELECT pg_temp.uji_tolak('T-09 NEG rs_api DELETE tabel L', 'rs_api', 'DELETE FROM uji_kelas.uji_l', '42501', 'permission denied for table uji_l');
SELECT pg_temp.uji_tolak('T-09 NEG pemilik UPDATE tabel L ditolak trigger', 'rs_migration', 'UPDATE uji_kelas.uji_l SET isi = ''x''', '42501', '^immutable_violation: UPDATE on uji_kelas.uji_l rejected$');
SELECT pg_temp.uji_tolak('T-09 NEG pemilik DELETE tabel L ditolak penjaga (bukan pekerja privasi)', 'rs_migration', 'DELETE FROM uji_kelas.uji_l', '42501', '^immutable_violation: DELETE on uji_kelas.uji_l rejected$');
SELECT pg_temp.uji_tolak('T-09 NEG pekerja privasi DELETE Log selama retention_policy kosong (PT-24)', 'rs_privacy_worker_login',
  'DELETE FROM uji_kelas.uji_l WHERE id = ''00000000-0000-7000-8000-000000000021''', '42501', '^retention_not_set: uji_kategori');
SELECT pg_temp.uji_tolak('T-09 NEG pekerja privasi UPDATE tabel L (tanpa hak)', 'rs_privacy_worker_login', 'UPDATE uji_kelas.uji_l SET isi = ''x''', '42501', 'permission denied for table uji_l');
INSERT INTO app.retention_policy VALUES ('00000000-0000-7000-8000-000000000041', 'uji_kategori', 1, 'anonymize', now() - interval '1 day', NULL);
SELECT pg_temp.uji_tolak('T-09 NEG pekerja privasi DELETE Log bila end_action tidak di daftar trigger', 'rs_privacy_worker_login',
  'DELETE FROM uji_kelas.uji_l WHERE id = ''00000000-0000-7000-8000-000000000021''', '42501', '^retention_action_not_allowed: uji_kategori anonymize');
INSERT INTO app.retention_policy VALUES ('00000000-0000-7000-8000-000000000042', 'uji_kategori', 1, 'delete_row', now() - interval '1 hour', NULL);
SELECT pg_temp.uji_tolak('T-09 NEG pekerja privasi DELETE Log yang masa retensinya belum habis', 'rs_privacy_worker_login',
  'DELETE FROM uji_kelas.uji_l WHERE id = ''00000000-0000-7000-8000-000000000022''', '42501', '^retention_not_expired: uji_kelas.uji_l');
SELECT pg_temp.uji_boleh('T-09 pekerja privasi DELETE Log setelah masa retensi habis dan end_action diizinkan (tanpa USAGE/EXECUTE util)', 'rs_privacy_worker_login',
  'DELETE FROM uji_kelas.uji_l WHERE id = ''00000000-0000-7000-8000-000000000021''', 1);
SELECT pg_temp.uji_tolak('T-09 NEG pekerja privasi TRUNCATE tabel L', 'rs_privacy_worker_login', 'TRUNCATE uji_kelas.uji_l', '42501', 'permission denied for table uji_l');

-- Pola berlaku-mulai (§4.2 (3), (3b), KEPUTUSAN-159).
SELECT pg_temp.uji_tolak('T-09 NEG rs_api INSERT baris berlaku-mulai lampau', 'rs_api',
  'INSERT INTO uji_kelas.uji_m_berlaku (id, effective_from, nilai) VALUES (''00000000-0000-7000-8000-000000000051'', now() - interval ''1 minute'', 1)',
  '42501', '^effective_row_backdated: uji_kelas.uji_m_berlaku');
SELECT pg_temp.uji_boleh('T-09 pemilik skema mengisi baris acuan lampau (seed migrasi)', 'rs_migration',
  'INSERT INTO uji_kelas.uji_m_berlaku (id, effective_from, nilai) VALUES (''00000000-0000-7000-8000-000000000052'', now() - interval ''1 day'', 1)', 1);
SELECT pg_temp.uji_boleh('T-09 rs_api INSERT baris berlaku-mulai masa depan', 'rs_api',
  'INSERT INTO uji_kelas.uji_m_berlaku (id, effective_from, nilai) VALUES (''00000000-0000-7000-8000-000000000053'', now() + interval ''1 day'', 2)', 1);
SELECT pg_temp.uji_tolak('T-09 NEG rs_api UPDATE baris yang sudah berlaku', 'rs_api',
  'UPDATE uji_kelas.uji_m_berlaku SET nilai = 9 WHERE id = ''00000000-0000-7000-8000-000000000052''', '42501', '^effective_row_locked: uji_kelas.uji_m_berlaku');
SELECT pg_temp.uji_tolak('T-09 NEG pemilik UPDATE baris yang sudah berlaku', 'rs_migration',
  'UPDATE uji_kelas.uji_m_berlaku SET deleted_at = now(), delete_reason = ''x'' WHERE id = ''00000000-0000-7000-8000-000000000052''', '42501', '^effective_row_locked');
SELECT pg_temp.uji_tolak('T-09 NEG rs_api memajukan baris masa depan menjadi berlaku', 'rs_api',
  'UPDATE uji_kelas.uji_m_berlaku SET effective_from = now() WHERE id = ''00000000-0000-7000-8000-000000000053''', '42501', '^effective_row_locked');
SELECT pg_temp.uji_boleh('T-09 rs_api mengubah baris yang belum berlaku', 'rs_api',
  'UPDATE uji_kelas.uji_m_berlaku SET nilai = 3 WHERE id = ''00000000-0000-7000-8000-000000000053''', 1);
SELECT pg_temp.uji_tolak('T-09 NEG rs_api DELETE tabel Master', 'rs_api', 'DELETE FROM uji_kelas.uji_m', '42501', 'permission denied for table uji_m');

-- Kelas MK satu arah (§4.10, KEPUTUSAN-66, KEPUTUSAN-96).
SELECT pg_temp.uji_tolak('T-09 NEG rs_api UPDATE data_key', 'rs_api', 'UPDATE uji_kelas.uji_mk SET wrapped_dek = NULL', '42501', 'permission denied for table uji_mk');
SELECT pg_temp.uji_tolak('T-09 NEG rs_api DELETE data_key', 'rs_api', 'DELETE FROM uji_kelas.uji_mk', '42501', 'permission denied for table uji_mk');
SELECT pg_temp.uji_tolak('T-09 NEG pemilik UPDATE data_key ditolak trigger', 'rs_migration',
  'UPDATE uji_kelas.uji_mk SET kek_ref = ''x'' WHERE id = ''00000000-0000-7000-8000-000000000031''', '42501', '^immutable_violation: UPDATE on data_key rejected$');
SELECT pg_temp.uji_tolak('T-09 NEG pemilik DELETE data_key ditolak trigger', 'rs_migration', 'DELETE FROM uji_kelas.uji_mk', '42501', '^immutable_violation: DELETE on uji_kelas.uji_mk rejected$');
SELECT pg_temp.uji_tolak('T-09 NEG pekerja privasi memusnahkan tanpa shred_reason', 'rs_privacy_worker_login',
  'UPDATE uji_kelas.uji_mk SET wrapped_dek = NULL WHERE id = ''00000000-0000-7000-8000-000000000031''', '42501', '^immutable_violation: invalid shred');
SELECT pg_temp.uji_tolak('T-09 NEG pekerja privasi mengganti wrapped_dek (bukan pemusnahan)', 'rs_privacy_worker_login',
  'UPDATE uji_kelas.uji_mk SET wrapped_dek = ''\x09'', shred_reason = ''x'' WHERE id = ''00000000-0000-7000-8000-000000000031''', '42501', '^immutable_violation: invalid shred');
SELECT pg_temp.uji_tolak('T-09 NEG rotasi kunci memusnahkan DEK', 'rs_key_rotation',
  'UPDATE uji_kelas.uji_mk SET wrapped_dek = NULL WHERE id = ''00000000-0000-7000-8000-000000000032''', '42501', '^immutable_violation: invalid rewrap');
SELECT pg_temp.uji_boleh('T-09 rotasi kunci membungkus ulang DEK', 'rs_key_rotation',
  'UPDATE uji_kelas.uji_mk SET wrapped_dek = ''\x0506'', kek_ref = ''openbao-transit:uji:k:v2'' WHERE id = ''00000000-0000-7000-8000-000000000032''', 1);
SELECT pg_temp.uji_boleh('T-09 pekerja privasi memusnahkan DEK (shredded_at diisi server)', 'rs_privacy_worker_login',
  'UPDATE uji_kelas.uji_mk SET wrapped_dek = NULL, shred_reason = ''retensi_habis'' WHERE id = ''00000000-0000-7000-8000-000000000031''', 1);
DO $$
BEGIN
  IF (SELECT shredded_at IS NULL OR wrapped_dek IS NOT NULL FROM uji_kelas.uji_mk WHERE id = '00000000-0000-7000-8000-000000000031') THEN
    RAISE EXCEPTION 'GAGAL T-09 pemusnahan DEK: shredded_at tidak diisi trigger';
  END IF;
  RAISE NOTICE 'LULUS T-09 shredded_at diisi trigger dengan jam server, wrapped_dek NULL';
END $$;
SELECT pg_temp.uji_tolak('T-09 NEG DEK yang dimusnahkan tidak dapat dihidupkan kembali (rotasi kunci)', 'rs_key_rotation',
  'UPDATE uji_kelas.uji_mk SET wrapped_dek = ''\x07'' WHERE id = ''00000000-0000-7000-8000-000000000031''', '42501', '^data_key_shredded: id=00000000-0000-7000-8000-000000000031 immutable$');
SELECT pg_temp.uji_tolak('T-09 NEG DEK yang dimusnahkan tidak dapat diubah pekerja privasi', 'rs_privacy_worker_login',
  'UPDATE uji_kelas.uji_mk SET shred_reason = ''lain'' WHERE id = ''00000000-0000-7000-8000-000000000031''', '42501', '^data_key_shredded');
SELECT pg_temp.uji_tolak('T-09 NEG DEK yang dimusnahkan tidak dapat diubah pemilik', 'rs_migration',
  'UPDATE uji_kelas.uji_mk SET container_type = ''lain'' WHERE id = ''00000000-0000-7000-8000-000000000031''', '42501', '^data_key_shredded');

-- Kelas OH dan P.
SELECT pg_temp.uji_tolak('T-09 NEG rs_api SELECT outbox audit (INSERT saja)', 'rs_api', 'SELECT * FROM uji_kelas.uji_oh', '42501', 'permission denied for table uji_oh');
SELECT pg_temp.uji_tolak('T-09 NEG INSERT ... RETURNING outbox oleh rs_api ditolak (butuh SELECT)', 'rs_api',
  'INSERT INTO uji_kelas.uji_oh (id, payload) VALUES (''00000000-0000-7000-8000-000000000061'', ''{}'') RETURNING id', '42501', 'permission denied for table uji_oh');
SELECT pg_temp.uji_boleh('T-09 rs_api INSERT outbox audit', 'rs_api', 'INSERT INTO uji_kelas.uji_oh (id, payload) VALUES (''00000000-0000-7000-8000-000000000062'', ''{}'')', 1);
SELECT pg_temp.uji_boleh('T-09 relay membaca dan menghapus outbox audit', 'rs_audit_relay_login', 'DELETE FROM uji_kelas.uji_oh WHERE id = ''00000000-0000-7000-8000-000000000062''', 1);
SELECT pg_temp.uji_tolak('T-09 NEG rs_api DELETE proyeksi', 'rs_api', 'DELETE FROM uji_kelas.uji_p', '42501', 'permission denied for table uji_p');

ROLLBACK;

-- Sisa fixture tidak boleh ada.
DO $$
BEGIN
  IF to_regnamespace('uji_kelas') IS NOT NULL OR to_regclass('app.retention_policy') IS NOT NULL THEN
    RAISE EXCEPTION 'GAGAL pembersihan fixture kelas-app';
  END IF;
  RAISE NOTICE 'LULUS pembersihan: fixture uji_kelas dan app.retention_policy tidak tersisa (ROLLBACK)';
END $$;
