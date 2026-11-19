-- db/test/lib.sql: fungsi bantu test katalog migrasi (dibuat di pg_temp, hilang saat sesi selesai).
-- Disisipkan scripts/db-migrations-check.sh di depan setiap berkas db/test/*.sql; dijalankan
-- superuser di kontainer PostgreSQL lokal/CI (BUKAN production).
-- Dasar: docs/05-physical-schema.md §1, §2.1, §3, §4, §11, §12 (T-01–T-09, T-18, T-22, T-24).
--
-- Konvensi: fungsi pemeriksa mengembalikan SETOF text berisi temuan (kosong = lulus), sehingga
-- dapat dipakai untuk skema nyata dan untuk tabel fixture yang sengaja dirusak (test negatif atas
-- pemeriksanya sendiri). Pemeriksaan yang lulus mencetak "LULUS <label>"; yang gagal menghentikan
-- skrip. Fungsi pg_temp tidak pernah dicari tanpa kualifikasi, sehingga selalu dipanggil pg_temp.*.

\set ON_ERROR_STOP on
SET client_min_messages = notice;
SET search_path = pg_catalog, pg_temp;

-- uji_nol(label, temuan[]): lulus bila tidak ada temuan.
CREATE FUNCTION pg_temp.uji_nol(p_label text, p_temuan text[]) RETURNS void
LANGUAGE plpgsql AS $f$
BEGIN
  IF coalesce(cardinality(p_temuan), 0) > 0 THEN
    RAISE EXCEPTION 'GAGAL %: %', p_label, array_to_string(p_temuan, ' | ');
  END IF;
  RAISE NOTICE 'LULUS %', p_label;
END $f$;

-- uji_tolak(label, role, sql, sqlstate, pola): sql dijalankan sebagai role (SET LOCAL ROLE di
-- sub-transaksi) dan WAJIB gagal dengan SQLSTATE dan pesan yang cocok pola. Perubahan selalu
-- dibatalkan. Dipanggil di dalam blok transaksi.
CREATE FUNCTION pg_temp.uji_tolak(p_label text, p_role text, p_sql text, p_state text, p_pola text) RETURNS void
LANGUAGE plpgsql AS $f$
DECLARE
  v_state text;
  v_msg   text;
BEGIN
  BEGIN
    EXECUTE format('SET LOCAL ROLE %I', p_role);
    EXECUTE p_sql;
    v_state := 'sukses';
    RAISE EXCEPTION USING ERRCODE = 'P0099', MESSAGE = 'batalkan';
  EXCEPTION WHEN OTHERS THEN
    IF v_state IS NULL THEN
      v_state := SQLSTATE;
      v_msg := SQLERRM;
    END IF;
  END;
  IF v_state IS DISTINCT FROM p_state OR coalesce(v_msg, '') !~ p_pola THEN
    RAISE EXCEPTION 'GAGAL %: % sebagai % -> SQLSTATE % "%" (diharapkan % /%/)',
      p_label, p_sql, p_role, v_state, coalesce(v_msg, ''), p_state, p_pola;
  END IF;
  RAISE NOTICE 'LULUS %', p_label;
END $f$;

-- uji_boleh(label, role, sql, baris): sql dijalankan sebagai role dan WAJIB berhasil (perubahan
-- tetap ada sampai transaksi luar selesai); bila baris tidak NULL, jumlah baris terdampak dicek.
CREATE FUNCTION pg_temp.uji_boleh(p_label text, p_role text, p_sql text, p_baris integer DEFAULT NULL) RETURNS void
LANGUAGE plpgsql AS $f$
DECLARE
  n bigint;
BEGIN
  EXECUTE format('SET LOCAL ROLE %I', p_role);
  EXECUTE p_sql;
  GET DIAGNOSTICS n = ROW_COUNT;
  RESET ROLE;
  IF p_baris IS NOT NULL AND n <> p_baris THEN
    RAISE EXCEPTION 'GAGAL %: % sebagai % mengenai % baris (diharapkan %)', p_label, p_sql, p_role, n, p_baris;
  END IF;
  RAISE NOTICE 'LULUS %', p_label;
END $f$;

-- uji_rusak(label, sql_rusak, pemeriksa, pola): pasang kerusakan, jalankan pemeriksa (query yang
-- mengembalikan temuan), pastikan ada temuan cocok pola, lalu batalkan kerusakan (sub-transaksi).
CREATE FUNCTION pg_temp.uji_rusak(p_label text, p_sql text, p_pemeriksa text, p_pola text) RETURNS void
LANGUAGE plpgsql AS $f$
DECLARE
  v_temuan text;
BEGIN
  BEGIN
    EXECUTE p_sql;
    EXECUTE format('SELECT string_agg(t, '' | '') FROM (%s) AS q(t)', p_pemeriksa) INTO v_temuan;
    RAISE EXCEPTION USING ERRCODE = 'P0099', MESSAGE = coalesce(v_temuan, '');
  EXCEPTION WHEN SQLSTATE 'P0099' THEN
    v_temuan := SQLERRM;
  END;
  IF v_temuan !~ p_pola THEN
    RAISE EXCEPTION 'GAGAL NEG pemeriksa %: kerusakan "%" tidak terdeteksi (temuan: "%", diharapkan /%/)',
      p_label, p_sql, v_temuan, p_pola;
  END IF;
  RAISE NOTICE 'LULUS NEG pemeriksa %', p_label;
END $f$;

-- ---------------------------------------------------------------------------------------------
-- T-01 nama objek (§1.1, §1.2): prefiks, snake_case, panjang <= 60 byte.
-- Dikecualikan: tabel catatan sqlx (_sqlx_migrations, nama dari sqlx), constraint NOT NULL
-- (contype n, nama otomatis PostgreSQL 18), dan objek turunan partisi (§1.2).
-- ---------------------------------------------------------------------------------------------
CREATE FUNCTION pg_temp.uji_t01(p_skema text[]) RETURNS SETOF text
LANGUAGE sql AS $f$
  WITH obj AS (
    SELECT 'relasi' AS jenis, n.nspname, c.relname::text AS nama, c.relkind::text AS sub,
           c.relispartition AS turunan,
           (SELECT p.relname::text FROM pg_inherits i JOIN pg_class p ON p.oid = i.inhparent
             WHERE i.inhrelid = c.oid AND c.relispartition) AS induk
      FROM pg_class c JOIN pg_namespace n ON n.oid = c.relnamespace
     WHERE n.nspname = ANY (p_skema) AND c.relkind IN ('r', 'p', 'v', 'm', 'S', 'f', 'c')
       AND c.relname <> '_sqlx_migrations'
    UNION ALL
    SELECT 'index', n.nspname, c.relname::text,
           CASE WHEN x.indisprimary THEN 'p'
                WHEN EXISTS (SELECT 1 FROM pg_constraint k WHERE k.conindid = c.oid AND k.contype = 'u') THEN 'u'
                WHEN x.indisunique THEN 'up' ELSE 'i' END,
           t.relispartition, NULL
      FROM pg_index x JOIN pg_class c ON c.oid = x.indexrelid JOIN pg_class t ON t.oid = x.indrelid
      JOIN pg_namespace n ON n.oid = c.relnamespace
     WHERE n.nspname = ANY (p_skema) AND t.relname <> '_sqlx_migrations'
    UNION ALL
    SELECT 'constraint', n.nspname, k.conname::text, k.contype::text,
           (t.relispartition OR k.conparentid <> 0 OR k.coninhcount > 0), NULL
      FROM pg_constraint k JOIN pg_class t ON t.oid = k.conrelid JOIN pg_namespace n ON n.oid = t.relnamespace
     WHERE n.nspname = ANY (p_skema) AND k.contype NOT IN ('n') AND t.relname <> '_sqlx_migrations'
    UNION ALL
    SELECT 'trigger', n.nspname, g.tgname::text, '', g.tgparentid <> 0, NULL
      FROM pg_trigger g JOIN pg_class t ON t.oid = g.tgrelid JOIN pg_namespace n ON n.oid = t.relnamespace
     WHERE n.nspname = ANY (p_skema) AND NOT g.tgisinternal
    UNION ALL
    SELECT 'rutin', n.nspname, p.proname::text, p.prokind::text, false, NULL
      FROM pg_proc p JOIN pg_namespace n ON n.oid = p.pronamespace
     WHERE n.nspname = ANY (p_skema)
    UNION ALL
    SELECT 'kebijakan', n.nspname, o.polname::text, '', false, NULL
      FROM pg_policy o JOIN pg_class t ON t.oid = o.polrelid JOIN pg_namespace n ON n.oid = t.relnamespace
     WHERE n.nspname = ANY (p_skema)
  )
  SELECT format('T-01 %s %s.%s: %s', jenis, nspname, nama, alasan)
    FROM (
      SELECT o.*,
             CASE
               WHEN octet_length(nama) > 60 THEN 'lebih dari 60 byte'
               WHEN nama !~ '^[a-z][a-z0-9_]*$' THEN 'bukan snake_case'
               WHEN jenis = 'relasi' AND sub = 'S' THEN 'sequence dilarang (§1.1, ADR-002)'
               WHEN jenis = 'relasi' AND turunan AND nama !~ ('^' || induk || '_y[0-9]{4}m[0-9]{2}$') THEN 'nama partisi bukan <induk>_yYYYYmMM'
               WHEN turunan THEN NULL
               WHEN jenis = 'index' AND sub = 'p' AND nama !~ '^pk_' THEN 'index primary key tanpa prefiks pk_'
               WHEN jenis = 'index' AND sub = 'u' AND nama !~ '^uq_' THEN 'index unique constraint tanpa prefiks uq_'
               WHEN jenis = 'index' AND sub = 'up' AND nama !~ '^uqp_' THEN 'index unik tanpa prefiks uqp_'
               WHEN jenis = 'index' AND sub = 'i' AND nama !~ '^ix_' THEN 'index tanpa prefiks ix_'
               WHEN jenis = 'constraint' AND sub = 'p' AND nama !~ '^pk_' THEN 'primary key tanpa prefiks pk_'
               WHEN jenis = 'constraint' AND sub = 'f' AND nama !~ '^fk_' THEN 'foreign key tanpa prefiks fk_'
               WHEN jenis = 'constraint' AND sub = 'u' AND nama !~ '^uq_' THEN 'unique tanpa prefiks uq_'
               WHEN jenis = 'constraint' AND sub = 'c' AND nama !~ '^ck_' THEN 'check tanpa prefiks ck_'
               WHEN jenis = 'constraint' AND sub = 'x' AND nama !~ '^ex_' THEN 'exclusion tanpa prefiks ex_'
               WHEN jenis = 'constraint' AND sub = 't' AND nama !~ '^tg_' THEN 'constraint trigger tanpa prefiks tg_'
               WHEN jenis = 'trigger' AND nama !~ '^tg_' THEN 'trigger tanpa prefiks tg_'
               WHEN jenis = 'rutin' AND sub = 'f' AND nama !~ '^fn_' THEN 'fungsi tanpa prefiks fn_'
               WHEN jenis = 'rutin' AND sub = 'p' AND nama !~ '^pr_' THEN 'prosedur tanpa prefiks pr_'
               WHEN jenis = 'rutin' AND sub NOT IN ('f', 'p') THEN 'jenis rutin tidak dikenal (' || sub || ')'
               WHEN jenis = 'kebijakan' AND nama !~ '^pol_' THEN 'kebijakan tanpa prefiks pol_'
             END AS alasan
        FROM obj o
    ) z
   WHERE alasan IS NOT NULL
   ORDER BY 1
$f$;

-- T-02 (§2.1, KEPUTUSAN-91): skema non-sistem persis daftar; public tidak ada; tabel catatan
-- migrasi sqlx hanya di skema bisnis; skema ext hanya berisi objek ekstensi.
CREATE FUNCTION pg_temp.uji_t02(p_skema_harap text[], p_skema_mig text) RETURNS SETOF text
LANGUAGE sql AS $f$
  SELECT format('T-02 skema tidak terdaftar: %s', nspname)
    FROM pg_namespace
   WHERE nspname !~ '^pg_' AND nspname <> 'information_schema' AND NOT (nspname = ANY (p_skema_harap))
  UNION ALL
  SELECT format('T-02 skema terdaftar tidak ada: %s', s)
    FROM unnest(p_skema_harap) s WHERE NOT EXISTS (SELECT 1 FROM pg_namespace WHERE nspname = s)
  UNION ALL
  SELECT format('T-02 tabel catatan migrasi di luar skema %s: %s.%s', p_skema_mig, n.nspname, c.relname)
    FROM pg_class c JOIN pg_namespace n ON n.oid = c.relnamespace
   WHERE c.relname = '_sqlx_migrations' AND n.nspname <> p_skema_mig
  UNION ALL
  SELECT format('T-02 tabel catatan migrasi tidak ada di %s', p_skema_mig)
   WHERE NOT EXISTS (SELECT 1 FROM pg_class c JOIN pg_namespace n ON n.oid = c.relnamespace
                      WHERE c.relname = '_sqlx_migrations' AND n.nspname = p_skema_mig AND c.relkind = 'r')
  UNION ALL
  SELECT format('T-02 objek bukan milik ekstensi di skema ext: %s', o.nama)
    FROM (SELECT c.oid, 'pg_class'::regclass AS kls, c.relname::text AS nama
            FROM pg_class c JOIN pg_namespace n ON n.oid = c.relnamespace WHERE n.nspname = 'ext'
          UNION ALL
          SELECT p.oid, 'pg_proc'::regclass, p.oid::regprocedure::text
            FROM pg_proc p JOIN pg_namespace n ON n.oid = p.pronamespace WHERE n.nspname = 'ext') o
   WHERE NOT EXISTS (SELECT 1 FROM pg_depend d
                      WHERE d.classid = o.kls AND d.objid = o.oid AND d.deptype = 'e')
     AND NOT EXISTS (SELECT 1 FROM pg_depend d JOIN pg_class t ON t.oid = d.refobjid
                      WHERE d.classid = 'pg_class'::regclass AND d.objid = o.oid AND d.deptype IN ('i', 'a')
                        AND EXISTS (SELECT 1 FROM pg_depend e WHERE e.classid = 'pg_class'::regclass
                                       AND e.objid = t.oid AND e.deptype = 'e'))
$f$;

-- T-03 (KEPUTUSAN-91, KEPUTUSAN-157): PUBLIC tanpa EXECUTE atas fungsi/prosedur proyek. proacl NULL
-- berarti hak bawaan (termasuk EXECUTE untuk PUBLIC, SRC-370) masih berlaku.
CREATE FUNCTION pg_temp.uji_t03(p_skema text[]) RETURNS SETOF text
LANGUAGE sql AS $f$
  SELECT format('T-03 %s: %s', p.oid::regprocedure,
                CASE WHEN p.proacl IS NULL THEN 'ACL bawaan (PUBLIC EXECUTE)' ELSE 'PUBLIC EXECUTE' END)
    FROM pg_proc p JOIN pg_namespace n ON n.oid = p.pronamespace
   WHERE n.nspname = ANY (p_skema)
     AND (p.proacl IS NULL OR EXISTS (SELECT 1 FROM aclexplode(p.proacl) a WHERE a.grantee = 0))
  UNION ALL
  SELECT format('T-03 database %s: PUBLIC %s', d.datname, a.privilege_type)
    FROM pg_database d, aclexplode(d.datacl) a
   WHERE d.datname = current_database() AND a.grantee = 0
  UNION ALL
  SELECT format('T-03 database %s: datacl NULL (hak bawaan PUBLIC)', datname)
    FROM pg_database WHERE datname = current_database() AND datacl IS NULL
$f$;

-- T-04 (KEPUTUSAN-97): semua relasi, rutin, dan tipe buatan di skema proyek dimiliki pemilik
-- skemanya (termasuk tabel catatan migrasi sqlx), bukan role dinamis atau superuser.
CREATE FUNCTION pg_temp.uji_t04(p_skema text[]) RETURNS SETOF text
LANGUAGE sql AS $f$
  SELECT format('T-04 relasi %s.%s dimiliki %s, bukan %s', n.nspname, c.relname,
                pg_get_userbyid(c.relowner), pg_get_userbyid(n.nspowner))
    FROM pg_class c JOIN pg_namespace n ON n.oid = c.relnamespace
   WHERE n.nspname = ANY (p_skema) AND c.relowner <> n.nspowner
  UNION ALL
  SELECT format('T-04 rutin %s dimiliki %s, bukan %s', p.oid::regprocedure,
                pg_get_userbyid(p.proowner), pg_get_userbyid(n.nspowner))
    FROM pg_proc p JOIN pg_namespace n ON n.oid = p.pronamespace
   WHERE n.nspname = ANY (p_skema) AND p.proowner <> n.nspowner
  UNION ALL
  SELECT format('T-04 tipe %s.%s dimiliki %s', n.nspname, t.typname, pg_get_userbyid(t.typowner))
    FROM pg_type t JOIN pg_namespace n ON n.oid = t.typnamespace
   WHERE n.nspname = ANY (p_skema) AND t.typowner <> n.nspowner
  UNION ALL
  SELECT format('T-04 skema %s dimiliki superuser/role tak terduga %s', n.nspname, pg_get_userbyid(n.nspowner))
    FROM pg_namespace n JOIN pg_roles r ON r.oid = n.nspowner
   WHERE n.nspname = ANY (p_skema) AND (r.rolsuper OR r.rolcanlogin)
$f$;

-- T-24 (review putaran 2 catatan 4): badan rutin dan argumen trigger hanya menyebut role
-- berawalan sistem database ini; setiap role di argumen trigger ada di pg_roles.
CREATE FUNCTION pg_temp.uji_t24(p_skema text[], p_prefiks text) RETURNS SETOF text
LANGUAGE sql AS $f$
  SELECT format('T-24 badan %s menyebut role %s (awalan database ini %s)', p.oid::regprocedure, m[1], p_prefiks)
    FROM pg_proc p JOIN pg_namespace n ON n.oid = p.pronamespace,
         regexp_matches(p.prosrc, '\m((rs|rme|openbao)_[a-z0-9_]+)', 'g') AS m
   WHERE n.nspname = ANY (p_skema)
     AND EXISTS (SELECT 1 FROM pg_roles r WHERE r.rolname = m[1])
     AND (m[1] !~ ('^' || p_prefiks) OR m[1] ~ '^openbao_')
  UNION ALL
  SELECT format('T-24 argumen trigger %s.%s: %s', c.relname, g.tgname, a.arg)
    FROM pg_trigger g JOIN pg_class c ON c.oid = g.tgrelid JOIN pg_namespace n ON n.oid = c.relnamespace,
         unnest(string_to_array(regexp_replace(encode(g.tgargs, 'escape'), '\\000$', ''), '\000')) AS a(arg)
   WHERE n.nspname = ANY (p_skema) AND NOT g.tgisinternal AND g.tgnargs > 0
     AND a.arg ~ '^(rs|rme|openbao)_'
     AND (a.arg !~ ('^' || p_prefiks) OR a.arg ~ '^openbao_'
          OR NOT EXISTS (SELECT 1 FROM pg_roles r WHERE r.rolname = a.arg))
$f$;

-- Rutin proyek (CR-020, §2.1 butir 5, §4.2, §5.2): SECURITY DEFINER hanya yang diizinkan dokumen
-- (audit.fn_chain_head §6.3, dengan proconfig persis search_path=pg_catalog, audit); setiap rutin
-- lain memasang proconfig persis search_path=pg_catalog, pg_temp (review DBA F1-T14/T15), kecuali util.fn_current_fasyankes_id (§5.2, sengaja tanpa SET agar dapat
-- di-inline; rujukannya berkualifikasi pg_catalog).
CREATE FUNCTION pg_temp.uji_rutin(p_skema text[]) RETURNS SETOF text
LANGUAGE sql AS $f$
  SELECT format('rutin %s: %s', p.oid::regprocedure, alasan)
    FROM pg_proc p JOIN pg_namespace n ON n.oid = p.pronamespace,
         LATERAL (SELECT CASE
           WHEN p.prosecdef AND NOT (n.nspname = 'audit' AND p.proname = 'fn_chain_head')
             THEN 'SECURITY DEFINER tidak diizinkan dokumen'
           WHEN p.prosecdef AND p.proconfig IS DISTINCT FROM ARRAY['search_path=pg_catalog, audit']
             THEN 'SECURITY DEFINER tanpa SET search_path persis pg_catalog, audit (§6.3)'
           WHEN (n.nspname, p.proname) = ('util', 'fn_current_fasyankes_id')
             THEN CASE WHEN p.proconfig IS NOT NULL THEN 'fn_current_fasyankes_id tidak boleh memasang SET (§5.2)' END
           WHEN p.proconfig IS DISTINCT FROM ARRAY['search_path=pg_catalog, pg_temp']
             THEN 'SET search_path bukan persis pg_catalog, pg_temp (CR-020, SRC-397)'
         END AS alasan) z
   WHERE n.nspname = ANY (p_skema) AND alasan IS NOT NULL
$f$;

-- ---------------------------------------------------------------------------------------------
-- T-06/T-07/T-08 kelas data database aplikasi Ruang Sehat (DD §0.3, §0.6 A; skema fisik §4).
-- Hak tingkat tabel per grup dibandingkan persis (tidak kurang, tidak lebih); hak per kolom hanya
-- untuk pekerja privasi dan rotasi kunci; trigger per kelas sesuai §4.11.
-- p_berlaku: daftar tabel pola berlaku-mulai (§4.2 (3)); tabel M lain dengan kolom effective_from
-- dianggap temuan supaya penambahan pola baru direview.
-- ---------------------------------------------------------------------------------------------
CREATE FUNCTION pg_temp.uji_kelas_app(p_skema text, p_berlaku text[]) RETURNS SETOF text
LANGUAGE plpgsql AS $f$
DECLARE
  t record;
  r record;
  v_kelas text;
  v_cmt text;
  v_harap text[];
  v_aktual text[];
  v_kolom text[];
  v_anon text[];
  v_selisih text;
BEGIN
  FOR t IN
    SELECT c.oid, c.relname::text AS relname, c.relkind
      FROM pg_class c JOIN pg_namespace n ON n.oid = c.relnamespace
     WHERE n.nspname = p_skema AND c.relkind IN ('r', 'p') AND NOT c.relispartition
       AND c.relname <> '_sqlx_migrations'
     ORDER BY c.relname
  LOOP
    v_cmt := obj_description(t.oid, 'pg_class');
    v_kelas := substring(v_cmt FROM '^class:([A-Z]+) ');
    IF v_kelas IS NULL OR v_kelas NOT IN ('M', 'MS', 'MK', 'T', 'R', 'P', 'O', 'L', 'OH')
       OR v_cmt !~ '^class:[A-Z]+ module:[a-z_]+( |$)' THEN
      RETURN NEXT format('T-06 %s: komentar kelas tidak sah (%s)', t.relname, coalesce(v_cmt, '<kosong>'));
      CONTINUE;
    END IF;

    -- Hak tingkat tabel yang diharapkan: daftar "grup:HAK".
    v_harap := CASE v_kelas
      WHEN 'M'  THEN ARRAY['rs_app:INSERT', 'rs_app:SELECT', 'rs_app:UPDATE', 'rs_privacy_worker:SELECT']
      WHEN 'MS' THEN ARRAY['rs_app:INSERT', 'rs_app:SELECT', 'rs_privacy_worker:SELECT']
      WHEN 'MK' THEN ARRAY['rs_app:INSERT', 'rs_app:SELECT', 'rs_privacy_worker:SELECT']
      WHEN 'T'  THEN ARRAY['rs_app:INSERT', 'rs_app:SELECT']
      WHEN 'R'  THEN ARRAY['rs_app:INSERT', 'rs_app:SELECT']
      WHEN 'P'  THEN ARRAY['rs_app:INSERT', 'rs_app:SELECT', 'rs_app:UPDATE']
      WHEN 'O'  THEN ARRAY['rs_app:DELETE', 'rs_app:INSERT', 'rs_app:SELECT', 'rs_app:UPDATE']
      WHEN 'L'  THEN ARRAY['rs_app:INSERT', 'rs_app:SELECT', 'rs_privacy_worker:DELETE']
      WHEN 'OH' THEN ARRAY['rs_app:INSERT', 'rs_audit_relay:DELETE', 'rs_audit_relay:SELECT', 'rs_privacy_worker:INSERT']
    END;
    -- Pengecualian yang ditetapkan dokumen.
    IF t.relname = 'order_status_transition' THEN                     -- §4.3, DD A8.13
      v_harap := ARRAY['rs_app:SELECT', 'rs_privacy_worker:SELECT'];
    ELSIF t.relname = 'retention_run' THEN                            -- DD §0.6 A
      v_harap := ARRAY['rs_app:INSERT', 'rs_app:SELECT', 'rs_privacy_worker:INSERT', 'rs_privacy_worker:SELECT'];
    ELSIF t.relname = 'data_key_retention_anchor' THEN                -- §4.12 (CR-014)
      v_harap := v_harap || 'rs_privacy_worker:SELECT'::text;
    END IF;

    SELECT coalesce(array_agg(x ORDER BY x), '{}') INTO v_aktual FROM (
      SELECT CASE WHEN a.grantee = 0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END || ':' || a.privilege_type AS x
        FROM pg_class c, aclexplode(c.relacl) a
       WHERE c.oid = t.oid AND a.grantee <> c.relowner) s;
    -- Operasional berisi data pribadi boleh menambah S, D untuk pekerja privasi (DD §0.6 A).
    IF v_kelas = 'O' AND v_aktual @> ARRAY['rs_privacy_worker:DELETE', 'rs_privacy_worker:SELECT'] THEN
      v_harap := v_harap || ARRAY['rs_privacy_worker:DELETE', 'rs_privacy_worker:SELECT'];
    END IF;
    SELECT string_agg(d, ', ') INTO v_selisih FROM (
      SELECT 'lebih ' || x AS d FROM unnest(v_aktual) x WHERE NOT (x = ANY (v_harap))
      UNION ALL
      SELECT 'kurang ' || x FROM unnest(v_harap) x WHERE NOT (x = ANY (v_aktual))) s;
    IF v_selisih IS NOT NULL THEN
      RETURN NEXT format('T-06 %s (kelas %s): hak tabel %s', t.relname, v_kelas, v_selisih);
    END IF;

    -- Hak per kolom (T-06/T-07): hanya pekerja privasi dan rotasi kunci.
    FOR r IN
      SELECT CASE WHEN a.grantee = 0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END AS grantee,
             a.privilege_type AS hak, array_agg(att.attname::text ORDER BY att.attname) AS kolom
        FROM pg_attribute att, aclexplode(att.attacl) a
       WHERE att.attrelid = t.oid AND att.attnum > 0 AND NOT att.attisdropped AND att.attacl IS NOT NULL
       GROUP BY 1, 2
    LOOP
      IF r.grantee = 'rs_privacy_worker' AND r.hak = 'UPDATE' AND v_kelas IN ('M', 'MS') THEN
        SELECT coalesce(array_agg(att.attname::text ORDER BY att.attname), '{}') INTO v_anon
          FROM pg_attribute att
         WHERE att.attrelid = t.oid AND att.attnum > 0 AND NOT att.attisdropped
           AND col_description(t.oid, att.attnum) = 'anonymize';
        v_kolom := v_anon || ARRAY['anonymized_at', 'updated_at', 'updated_by'];
        IF t.relname = 'account_owner_nik_registry' THEN
          v_kolom := v_kolom || ARRAY['deleted_at', 'deleted_by', 'delete_reason'];
        END IF;
        IF cardinality(v_anon) = 0 OR NOT (r.kolom @> v_kolom AND v_kolom @> r.kolom) THEN
          RETURN NEXT format('T-07 %s: UPDATE per kolom rs_privacy_worker %s, diharapkan kolom anonymize + anonymized_at, updated_at, updated_by (%s)',
                             t.relname, r.kolom, v_kolom);
        END IF;
      ELSIF r.grantee = 'rs_privacy_worker' AND r.hak = 'UPDATE' AND v_kelas = 'MK' THEN
        IF NOT (r.kolom @> ARRAY['shred_reason', 'shredded_at', 'wrapped_dek'] AND ARRAY['shred_reason', 'shredded_at', 'wrapped_dek'] @> r.kolom) THEN
          RETURN NEXT format('T-07 %s: UPDATE per kolom rs_privacy_worker %s (MK: wrapped_dek, shredded_at, shred_reason)', t.relname, r.kolom);
        END IF;
      ELSIF r.grantee = 'rs_privacy_worker' AND r.hak = 'SELECT' AND v_kelas IN ('T', 'L') THEN
        IF EXISTS (SELECT 1 FROM unnest(r.kolom) k WHERE k NOT IN ('id', 'created_at') AND k !~ '_id$') THEN
          RETURN NEXT format('T-06 %s: SELECT per kolom rs_privacy_worker %s melebihi ID, FK subjek, created_at, data_key_id', t.relname, r.kolom);
        END IF;
      ELSIF r.grantee = 'rs_privacy_worker' AND r.hak = 'SELECT' AND v_kelas = 'R' THEN
        IF EXISTS (SELECT 1 FROM unnest(r.kolom) k WHERE k NOT IN ('id', 'note_data_key_id')) THEN
          RETURN NEXT format('T-06 %s: SELECT per kolom rs_privacy_worker %s melebihi id, note_data_key_id', t.relname, r.kolom);
        END IF;
      ELSIF r.grantee = 'rs_key_rotation' AND v_kelas = 'MK' AND r.hak = 'SELECT' THEN
        IF EXISTS (SELECT 1 FROM unnest(r.kolom) k WHERE k NOT IN ('id', 'wrapped_dek', 'kek_ref', 'shredded_at')) THEN
          RETURN NEXT format('T-06 %s: SELECT per kolom rs_key_rotation %s', t.relname, r.kolom);
        END IF;
      ELSIF r.grantee = 'rs_key_rotation' AND v_kelas = 'MK' AND r.hak = 'UPDATE' THEN
        IF NOT (r.kolom @> ARRAY['kek_ref', 'wrapped_dek'] AND ARRAY['kek_ref', 'wrapped_dek'] @> r.kolom) THEN
          RETURN NEXT format('T-06 %s: UPDATE per kolom rs_key_rotation %s (MK: wrapped_dek, kek_ref)', t.relname, r.kolom);
        END IF;
      ELSE
        RETURN NEXT format('T-06 %s (kelas %s): hak per kolom tidak diizinkan %s:%s %s', t.relname, v_kelas, r.grantee, r.hak, r.kolom);
      END IF;
    END LOOP;

    -- T-08 trigger per kelas (§4.11): ada, aktif, bentuk dan fungsi benar. tgtype: 1 ROW,
    -- 2 BEFORE, 4 INSERT, 8 DELETE, 16 UPDATE, 32 TRUNCATE.
    SELECT coalesce(array_agg(x ORDER BY x), '{}') INTO v_aktual FROM (
      SELECT CASE WHEN g.tgenabled <> 'O' THEN 'NONAKTIF:' ELSE '' END
             || CASE WHEN g.tgtype & 2 = 2 THEN 'before' ELSE 'after' END
             || CASE WHEN g.tgtype & 1 = 1 THEN ':row' ELSE ':stmt' END
             || CASE WHEN g.tgtype & 4 = 4 THEN ':ins' ELSE '' END
             || CASE WHEN g.tgtype & 16 = 16 THEN ':upd' ELSE '' END
             || CASE WHEN g.tgtype & 8 = 8 THEN ':del' ELSE '' END
             || CASE WHEN g.tgtype & 32 = 32 THEN ':trunc' ELSE '' END
             || ':' || g.tgfoid::regproc::text AS x
        FROM pg_trigger g
       WHERE g.tgrelid = t.oid AND NOT g.tgisinternal
         AND g.tgfoid::regproc::text LIKE 'util.%') s;
    v_harap := CASE
      WHEN v_kelas IN ('T', 'R') THEN ARRAY['before:stmt:trunc:util.fn_reject_modification',
                                            'before:stmt:upd:del:util.fn_reject_modification']
      WHEN v_kelas = 'L' THEN ARRAY['before:row:del:util.fn_guard_log_delete',
                                    'before:stmt:trunc:util.fn_reject_modification',
                                    'before:stmt:upd:util.fn_reject_modification']
      WHEN v_kelas = 'MK' THEN ARRAY['before:row:upd:util.fn_data_key_one_way',
                                     'before:stmt:del:util.fn_reject_modification',
                                     'before:stmt:trunc:util.fn_reject_modification']
      WHEN v_kelas = 'M' AND t.relname = ANY (p_berlaku) THEN ARRAY['before:row:ins:util.fn_effective_row_no_backdate',
                                                                    'before:row:upd:util.fn_effective_row_lock']
      ELSE '{}'::text[]
    END;
    IF v_kelas = 'M' AND NOT (t.relname = ANY (p_berlaku))
       AND EXISTS (SELECT 1 FROM pg_attribute WHERE attrelid = t.oid AND attname = 'effective_from' AND NOT attisdropped) THEN
      RETURN NEXT format('T-08 %s: kolom effective_from tetapi tidak terdaftar sebagai pola berlaku-mulai (§4.2 (3))', t.relname);
    END IF;
    IF NOT (v_aktual @> v_harap AND v_harap @> v_aktual) THEN
      RETURN NEXT format('T-08 %s (kelas %s): trigger util %s, diharapkan %s', t.relname, v_kelas, v_aktual, v_harap);
    END IF;
    -- Trigger proyek apa pun yang nonaktif (DISABLE TRIGGER) adalah temuan.
    FOR r IN SELECT g.tgname FROM pg_trigger g WHERE g.tgrelid = t.oid AND NOT g.tgisinternal AND g.tgenabled <> 'O' LOOP
      RETURN NEXT format('T-08 %s: trigger %s tidak aktif', t.relname, r.tgname);
    END LOOP;
    -- Argumen trigger per fungsi (T-08, T-24).
    FOR r IN
      SELECT g.tgname, g.tgfoid::regproc::text AS fn, g.tgnargs,
             string_to_array(regexp_replace(encode(g.tgargs, 'escape'), '\\000$', ''), '\000') AS args
        FROM pg_trigger g WHERE g.tgrelid = t.oid AND NOT g.tgisinternal
    LOOP
      IF r.fn = 'util.fn_guard_log_delete' AND NOT (
           r.tgnargs = 4 AND r.args[1] <> '' AND r.args[2] <> '' AND r.args[3] ~ '^\{.*\}$'
           AND r.args[4] = 'rs_privacy_worker') THEN
        RETURN NEXT format('T-08 %s: argumen %s %s (kategori, kolom acuan, daftar end_action, rs_privacy_worker)', t.relname, r.tgname, r.args);
      ELSIF r.fn = 'util.fn_data_key_one_way' AND r.args IS DISTINCT FROM ARRAY['rs_privacy_worker', 'rs_key_rotation'] THEN
        RETURN NEXT format('T-08 %s: argumen %s %s (rs_privacy_worker, rs_key_rotation)', t.relname, r.tgname, r.args);
      ELSIF r.fn = 'util.fn_effective_row_no_backdate' AND r.args IS DISTINCT FROM ARRAY['rs_migration'] THEN
        RETURN NEXT format('T-08 %s: argumen %s %s (rs_migration)', t.relname, r.tgname, r.args);
      ELSIF r.fn IN ('util.fn_reject_modification', 'util.fn_effective_row_lock') AND r.tgnargs <> 0 THEN
        RETURN NEXT format('T-08 %s: argumen %s tidak diharapkan', t.relname, r.tgname);
      END IF;
    END LOOP;
  END LOOP;
END $f$;

-- Ketentuan DDL umum §4.1 butir 2-3 (KEPUTUSAN-93), F1-T16: kolom id tanpa nilai bawaan/identity/
-- generated (UUIDv7 dari aplikasi, ADR-002); created_at NOT NULL DEFAULT now(); semua FK ON DELETE
-- dan ON UPDATE NO ACTION (tanpa CASCADE/SET NULL/SET DEFAULT/RESTRICT; kode 'a', SRC-431).
CREATE FUNCTION pg_temp.uji_ddl_umum(p_skema text) RETURNS SETOF text
LANGUAGE sql AS $f$
  SELECT format('KEPUTUSAN-93 %s.%s: kolom id punya nilai bawaan/identity/generated', c.relname, a.attname)
    FROM pg_attribute a JOIN pg_class c ON c.oid = a.attrelid JOIN pg_namespace n ON n.oid = c.relnamespace
   WHERE n.nspname = p_skema AND c.relkind IN ('r', 'p') AND c.relname <> '_sqlx_migrations'
     AND a.attname = 'id' AND NOT a.attisdropped
     AND (a.atthasdef OR a.attidentity <> '' OR a.attgenerated <> '')
  UNION ALL
  SELECT format('KEPUTUSAN-93 %s.created_at: bukan NOT NULL DEFAULT now() (%s)', c.relname,
                coalesce(pg_get_expr(d.adbin, d.adrelid), '<tanpa default>'))
    FROM pg_attribute a JOIN pg_class c ON c.oid = a.attrelid JOIN pg_namespace n ON n.oid = c.relnamespace
    LEFT JOIN pg_attrdef d ON d.adrelid = a.attrelid AND d.adnum = a.attnum
   WHERE n.nspname = p_skema AND c.relkind IN ('r', 'p') AND c.relname <> '_sqlx_migrations'
     AND a.attname = 'created_at' AND NOT a.attisdropped
     AND (NOT a.attnotnull OR pg_get_expr(d.adbin, d.adrelid) IS DISTINCT FROM 'now()')
  UNION ALL
  SELECT format('KEPUTUSAN-93 %s: FK %s ON DELETE/UPDATE bukan NO ACTION (%s/%s)', c.relname, k.conname,
                k.confdeltype, k.confupdtype)
    FROM pg_constraint k JOIN pg_class c ON c.oid = k.conrelid JOIN pg_namespace n ON n.oid = c.relnamespace
   WHERE n.nspname = p_skema AND k.contype = 'f' AND (k.confdeltype <> 'a' OR k.confupdtype <> 'a')
$f$;

-- uji_indeks(tabel berkualifikasi, definisi[]): index tabel persis daftar pg_get_indexdef (tidak kurang,
-- tidak lebih), termasuk index PK/UQ. Dipakai test per tabel (F1-T16 dst.).
CREATE FUNCTION pg_temp.uji_indeks(p_tabel regclass, p_harap text[]) RETURNS SETOF text
LANGUAGE sql AS $f$
  WITH aktual AS (SELECT pg_get_indexdef(indexrelid) AS def FROM pg_index WHERE indrelid = p_tabel)
  SELECT format('index %s lebih: %s', p_tabel, def) FROM aktual WHERE NOT (def = ANY (p_harap))
  UNION ALL
  SELECT format('index %s kurang: %s', p_tabel, h) FROM unnest(p_harap) h WHERE h NOT IN (SELECT def FROM aktual)
$f$;

-- T-22 (review N05): daftar tabel L yang punya FK masuk dari tabel non-L (snapshot di pemanggil).
CREATE FUNCTION pg_temp.uji_t22(p_skema text) RETURNS text[]
LANGUAGE sql AS $f$
  SELECT coalesce(array_agg(DISTINCT tl.relname::text ORDER BY tl.relname::text), '{}')
    FROM pg_constraint k
    JOIN pg_class tl ON tl.oid = k.confrelid
    JOIN pg_class ts ON ts.oid = k.conrelid
    JOIN pg_namespace n ON n.oid = tl.relnamespace
   WHERE k.contype = 'f' AND n.nspname = p_skema
     AND obj_description(tl.oid, 'pg_class') ~ '^class:L '
     AND coalesce(obj_description(ts.oid, 'pg_class'), '') !~ '^class:L '
$f$;

-- ---------------------------------------------------------------------------------------------
-- Kelas A database audit (DD §0.6 B; skema fisik §4.11, §5.6, §6.1, §6.2; T-06, T-08, T-18).
-- Hak per tabel menurut nama tabel (nama logis DD §0.6 B dipetakan ke awalan rs_/rme_).
-- ---------------------------------------------------------------------------------------------
CREATE FUNCTION pg_temp.uji_kelas_audit(p_skema text, p_prefiks text) RETURNS SETOF text
LANGUAGE plpgsql AS $f$
DECLARE
  t record;
  r record;
  v_cmt text;
  v_harap text[];
  v_aktual text[];
  v_selisih text;
  v_tenant text[] := CASE WHEN p_prefiks = 'rme_' THEN ARRAY['rme_audit_tenant_reader:SELECT'] ELSE '{}' END;
BEGIN
  FOR t IN
    SELECT c.oid, c.relname::text AS relname, c.relkind, c.relispartition,
           (SELECT i.inhparent FROM pg_inherits i WHERE i.inhrelid = c.oid) AS induk
      FROM pg_class c JOIN pg_namespace n ON n.oid = c.relnamespace
     WHERE n.nspname = p_skema AND c.relkind IN ('r', 'p') AND c.relname <> '_sqlx_migrations'
     ORDER BY c.relname
  LOOP
    SELECT coalesce(array_agg(x ORDER BY x), '{}') INTO v_aktual FROM (
      SELECT CASE WHEN a.grantee = 0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END || ':' || a.privilege_type AS x
        FROM pg_class c, aclexplode(c.relacl) a
       WHERE c.oid = t.oid AND a.grantee <> c.relowner) s;
    IF t.relispartition THEN
      v_harap := '{}';                                                  -- hak hanya di induk (§5.6)
    ELSE
      v_cmt := obj_description(t.oid, 'pg_class');
      IF v_cmt IS DISTINCT FROM 'class:A module:audit' THEN
        RETURN NEXT format('T-06 %s: komentar kelas bukan "class:A module:audit" (%s)', t.relname, coalesce(v_cmt, '<kosong>'));
      END IF;
      v_harap := CASE t.relname
        WHEN 'audit_event' THEN ARRAY['audit_writer:INSERT', 'audit_reader:SELECT', 'audit_verifier:SELECT']
        WHEN 'audit_event_subject' THEN ARRAY['audit_writer:INSERT', 'audit_reader:SELECT', 'audit_verifier:SELECT']
        WHEN 'audit_event_key' THEN ARRAY['audit_writer:INSERT', 'audit_reader:SELECT', 'audit_verifier:SELECT']
        WHEN 'audit_chain_anchor' THEN ARRAY['audit_writer:INSERT', 'audit_reader:SELECT', 'audit_verifier:SELECT']
        WHEN 'audit_chain_verification' THEN ARRAY['audit_reader:SELECT', 'audit_verifier:INSERT', 'audit_verifier:SELECT']
        WHEN 'audit_archive_manifest' THEN ARRAY['audit_reader:SELECT', 'audit_verifier:SELECT']
      END;
      IF v_harap IS NULL THEN
        RETURN NEXT format('T-06 %s: tabel audit tanpa aturan hak DD §0.6 B (tambahkan aturan bersama migrasinya)', t.relname);
        CONTINUE;
      END IF;
      SELECT array_agg(p_prefiks || x ORDER BY x) INTO v_harap FROM unnest(v_harap) x;
      IF t.relname IN ('audit_event', 'audit_event_subject') THEN
        v_harap := v_harap || v_tenant;
      END IF;
    END IF;
    SELECT string_agg(d, ', ') INTO v_selisih FROM (
      SELECT 'lebih ' || x AS d FROM unnest(v_aktual) x WHERE NOT (x = ANY (v_harap))
      UNION ALL
      SELECT 'kurang ' || x FROM unnest(v_harap) x WHERE NOT (x = ANY (v_aktual))) s;
    IF v_selisih IS NOT NULL THEN
      RETURN NEXT format('T-06 %s: hak tabel %s', t.relname, v_selisih);
    END IF;
    IF EXISTS (SELECT 1 FROM pg_attribute WHERE attrelid = t.oid AND attacl IS NOT NULL) THEN
      RETURN NEXT format('T-06 %s: hak per kolom tidak diharapkan di tabel audit', t.relname);
    END IF;

    -- T-08/T-18: penolak statement di setiap tabel dan partisi; induk terpartisi ditambah penolak baris.
    SELECT coalesce(array_agg(x ORDER BY x), '{}') INTO v_aktual FROM (
      SELECT CASE WHEN g.tgenabled <> 'O' THEN 'NONAKTIF:' ELSE '' END
             || CASE WHEN g.tgtype & 2 = 2 THEN 'before' ELSE 'after' END
             || CASE WHEN g.tgtype & 1 = 1 THEN ':row' ELSE ':stmt' END
             || CASE WHEN g.tgtype & 4 = 4 THEN ':ins' ELSE '' END
             || CASE WHEN g.tgtype & 16 = 16 THEN ':upd' ELSE '' END
             || CASE WHEN g.tgtype & 8 = 8 THEN ':del' ELSE '' END
             || CASE WHEN g.tgtype & 32 = 32 THEN ':trunc' ELSE '' END
             || CASE WHEN g.tgparentid <> 0 THEN ':klon' ELSE '' END
             || ':' || g.tgfoid::regproc::text AS x
        FROM pg_trigger g WHERE g.tgrelid = t.oid AND NOT g.tgisinternal) s;
    v_harap := ARRAY['before:stmt:trunc:util.fn_reject_modification', 'before:stmt:upd:del:util.fn_reject_modification'];
    IF t.relkind = 'p' THEN
      v_harap := v_harap || 'before:row:upd:del:util.fn_reject_modification'::text;
    END IF;
    IF t.relispartition AND EXISTS (SELECT 1 FROM pg_class p WHERE p.oid = t.induk AND p.relkind = 'p') THEN
      v_harap := v_harap || 'before:row:upd:del:klon:util.fn_reject_modification'::text;
    END IF;
    IF NOT (v_aktual @> v_harap AND v_harap @> v_aktual) THEN
      RETURN NEXT format('T-08 %s: trigger %s, diharapkan %s', t.relname, v_aktual, v_harap);
    END IF;
    -- T-18: partisi punya semua index induk (index partisi terpasang ke index induk).
    IF t.relispartition THEN
      FOR r IN
        SELECT pi.indexrelid::regclass::text AS idx
          FROM pg_index pi
         WHERE pi.indrelid = t.induk
           AND NOT EXISTS (SELECT 1 FROM pg_inherits ih JOIN pg_index ci ON ci.indexrelid = ih.inhrelid
                            WHERE ih.inhparent = pi.indexrelid AND ci.indrelid = t.oid)
      LOOP
        RETURN NEXT format('T-18 %s: index induk %s tidak ada di partisi', t.relname, r.idx);
      END LOOP;
    END IF;
  END LOOP;
END $f$;
