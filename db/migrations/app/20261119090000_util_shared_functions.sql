-- db/migrations/app/20261119090000_util_shared_functions.sql
-- F1-T14: fungsi bersama skema util database ruang_sehat ("pustaka migrasi" DD §D.3).
--
-- Modul kritis T1 (ADR-008, ADR-004, ADR-013, ADR-021 butir 4): review DBA + keamanan.
-- Sumber: docs/05-physical-schema.md §2.1 butir 5-6, §4.2 (1), (2), (3), (3b), §4.10, §11.2;
-- CR-020 (search_path aman). Maju saja (ADR-019); berkas ini tidak pernah diubah setelah
-- diterapkan di lingkungan mana pun, koreksi lewat berkas baru (§11.1).
--
-- Ketentuan:
-- - Semua fungsi SECURITY INVOKER (bawaan), sehingga tidak menambah hak pemanggil (§4.2,
--   KEPUTUSAN-95). search_path fungsi dipasang tetap "pg_catalog, pg_temp" (pg_temp terakhir
--   sesuai dokumentasi CREATE FUNCTION, SRC-397; selaras CR-020), dan tabel dirujuk berkualifikasi.
-- - EXECUTE tidak diberikan ke PUBLIC (REVOKE eksplisit di samping default privileges bootstrap,
--   §2.1 butir 6) dan tidak diberikan ke role mana pun: trigger memanggil fungsinya tanpa
--   pemeriksaan EXECUTE/USAGE terhadap pelaku (dibuktikan test db/test/kelas-app.sql, SRC-398).
--   Karena itu skema util tidak diberi USAGE untuk role mana pun di database ini.
-- - Nama role tidak ditulis di badan fungsi; diteruskan lewat argumen trigger (§4.2, T-24).
-- - Migrasi ini TIDAK mengubah search_path sesi: sqlx mencatat _sqlx_migrations dengan nama
--   tanpa skema di transaksi yang sama (SRC-399).

-- Penjaga (§11.2). Berkas pertama juga memeriksa urutan: bootstrap db/roles/app.sql (F1-T13)
-- wajib sudah diterapkan ke database ruang_sehat.
DO $$
BEGIN
  IF current_user <> 'rs_migration' THEN
    RAISE EXCEPTION 'migration must run as rs_migration, got % (session %)', current_user, session_user;
  END IF;
  IF pg_catalog.current_database() <> 'ruang_sehat'
     OR NOT EXISTS (SELECT 1 FROM pg_catalog.pg_namespace
                     WHERE nspname = 'util' AND nspowner = 'rs_migration'::pg_catalog.regrole)
     OR EXISTS (SELECT 1 FROM pg_catalog.pg_namespace WHERE nspname = 'public') THEN
    RAISE EXCEPTION 'migration order: bootstrap db/roles/app.sql has not been applied to database % (run it before db/migrations/app)',
      pg_catalog.current_database();
  END IF;
END $$;
SET LOCAL lock_timeout = '5s';
SET LOCAL statement_timeout = '15min';

-- (1) Penolak umum: trigger tingkat statement di kelas T, R, L, data_key (§4.2 (1), §4.11).
CREATE FUNCTION util.fn_reject_modification()
RETURNS trigger
LANGUAGE plpgsql
SET search_path = pg_catalog, pg_temp
AS $$
BEGIN
  RAISE EXCEPTION 'immutable_violation: % on %.% rejected', TG_OP, TG_TABLE_SCHEMA, TG_TABLE_NAME
    USING ERRCODE = 'insufficient_privilege';
END;
$$;
REVOKE ALL ON FUNCTION util.fn_reject_modification() FROM PUBLIC;
COMMENT ON FUNCTION util.fn_reject_modification() IS
  'Penolak UPDATE/DELETE/TRUNCATE kelas T, R, L, MK (skema fisik §4.2 (1)); galat immutable_violation';

-- (2) Penjaga DELETE kelas Log (§4.2 (2)). Argumen trigger:
--     TG_ARGV[0] = kategori retensi, TG_ARGV[1] = kolom acuan waktu (umumnya created_at),
--     TG_ARGV[2] = daftar end_action yang mengizinkan penghapusan baris (ditetapkan bersama PT-24),
--     TG_ARGV[3] = role grup pekerja privasi database ini ('rs_privacy_worker').
-- app.retention_policy dibuat migrasi A6.6; sampai ada, DELETE pekerja privasi gagal (tertutup).
CREATE FUNCTION util.fn_guard_log_delete()
RETURNS trigger
LANGUAGE plpgsql
SET search_path = pg_catalog, pg_temp
AS $$
DECLARE
  v_months     integer;
  v_end_action text;
  v_anchor     timestamptz;
BEGIN
  IF TG_NARGS < 4 OR NOT pg_has_role(TG_ARGV[3], 'MEMBER') THEN
    RAISE EXCEPTION 'immutable_violation: DELETE on %.% rejected', TG_TABLE_SCHEMA, TG_TABLE_NAME
      USING ERRCODE = 'insufficient_privilege';
  END IF;
  SELECT rp.retention_months, rp.end_action INTO v_months, v_end_action
    FROM app.retention_policy rp
   WHERE rp.data_category = TG_ARGV[0]
     AND rp.deleted_at IS NULL
     AND rp.effective_from <= now()
   ORDER BY rp.effective_from DESC
   LIMIT 1;
  IF v_months IS NULL THEN
    RAISE EXCEPTION 'retention_not_set: % (%.%)', TG_ARGV[0], TG_TABLE_SCHEMA, TG_TABLE_NAME
      USING ERRCODE = 'insufficient_privilege';
  END IF;
  IF TG_NARGS < 3 OR NOT (v_end_action = ANY (TG_ARGV[2]::text[])) THEN
    RAISE EXCEPTION 'retention_action_not_allowed: % % (%.%)', TG_ARGV[0], v_end_action, TG_TABLE_SCHEMA, TG_TABLE_NAME
      USING ERRCODE = 'insufficient_privilege';
  END IF;
  v_anchor := (to_jsonb(OLD) ->> TG_ARGV[1])::timestamptz;
  IF v_anchor IS NULL OR v_anchor > now() - make_interval(months => v_months) THEN
    RAISE EXCEPTION 'retention_not_expired: %.% id=%', TG_TABLE_SCHEMA, TG_TABLE_NAME, OLD.id
      USING ERRCODE = 'insufficient_privilege';
  END IF;
  RETURN OLD;
END;
$$;
REVOKE ALL ON FUNCTION util.fn_guard_log_delete() FROM PUBLIC;
COMMENT ON FUNCTION util.fn_guard_log_delete() IS
  'Penjaga DELETE kelas L: hanya pekerja privasi, setelah masa retensi habis (skema fisik §4.2 (2))';

-- (3) Kunci baris berlaku-mulai (DD §0.8, §4.2 (3)): BEFORE UPDATE FOR EACH ROW.
CREATE FUNCTION util.fn_effective_row_lock()
RETURNS trigger
LANGUAGE plpgsql
SET search_path = pg_catalog, pg_temp
AS $$
BEGIN
  IF OLD.effective_from <= now() OR NEW.effective_from <= now() THEN
    RAISE EXCEPTION 'effective_row_locked: %.% id=%', TG_TABLE_SCHEMA, TG_TABLE_NAME, OLD.id
      USING ERRCODE = 'insufficient_privilege';
  END IF;
  RETURN NEW;
END;
$$;
REVOKE ALL ON FUNCTION util.fn_effective_row_lock() FROM PUBLIC;
COMMENT ON FUNCTION util.fn_effective_row_lock() IS
  'Baris berlaku-mulai yang sudah berlaku tidak dapat diubah (skema fisik §4.2 (3))';

-- (3b) Larangan menanggalmundurkan baris berlaku-mulai baru (§4.2 (3b), KEPUTUSAN-159):
--      BEFORE INSERT FOR EACH ROW; TG_ARGV[0] = role pemilik skema ('rs_migration').
CREATE FUNCTION util.fn_effective_row_no_backdate()
RETURNS trigger
LANGUAGE plpgsql
SET search_path = pg_catalog, pg_temp
AS $$
BEGIN
  IF NEW.effective_from < now() AND NOT pg_has_role(TG_ARGV[0], 'MEMBER') THEN
    RAISE EXCEPTION 'effective_row_backdated: %.% effective_from=%', TG_TABLE_SCHEMA, TG_TABLE_NAME, NEW.effective_from
      USING ERRCODE = 'insufficient_privilege';
  END IF;
  RETURN NEW;
END;
$$;
REVOKE ALL ON FUNCTION util.fn_effective_row_no_backdate() FROM PUBLIC;
COMMENT ON FUNCTION util.fn_effective_row_no_backdate() IS
  'Baris berlaku-mulai baru tidak boleh lampau kecuali oleh pemilik skema (skema fisik §4.2 (3b))';

-- (4) Satu arah data_key (§4.10, KEPUTUSAN-66, KEPUTUSAN-96): BEFORE UPDATE FOR EACH ROW;
--     TG_ARGV[0] = role pekerja privasi, TG_ARGV[1] = role rotasi kunci.
CREATE FUNCTION util.fn_data_key_one_way()
RETURNS trigger
LANGUAGE plpgsql
SET search_path = pg_catalog, pg_temp
AS $$
BEGIN
  IF OLD.shredded_at IS NOT NULL THEN
    RAISE EXCEPTION 'data_key_shredded: id=% immutable', OLD.id USING ERRCODE = 'insufficient_privilege';
  END IF;
  IF NEW.id IS DISTINCT FROM OLD.id
     OR NEW.created_at IS DISTINCT FROM OLD.created_at
     OR NEW.container_type IS DISTINCT FROM OLD.container_type
     OR NEW.container_id IS DISTINCT FROM OLD.container_id
     OR NEW.retention_category IS DISTINCT FROM OLD.retention_category
     OR NEW.retention_anchor_at IS DISTINCT FROM OLD.retention_anchor_at THEN
    RAISE EXCEPTION 'immutable_violation: data_key id=% column not updatable', OLD.id
      USING ERRCODE = 'insufficient_privilege';
  END IF;
  IF pg_has_role(TG_ARGV[0], 'MEMBER') THEN
    IF NEW.wrapped_dek IS NOT NULL OR NEW.shred_reason IS NULL
       OR NEW.kek_ref IS DISTINCT FROM OLD.kek_ref THEN
      RAISE EXCEPTION 'immutable_violation: invalid shred on data_key id=%', OLD.id
        USING ERRCODE = 'insufficient_privilege';
    END IF;
    NEW.shredded_at := now();
    RETURN NEW;
  ELSIF pg_has_role(TG_ARGV[1], 'MEMBER') THEN
    IF NEW.wrapped_dek IS NULL OR NEW.shredded_at IS NOT NULL OR NEW.shred_reason IS NOT NULL THEN
      RAISE EXCEPTION 'immutable_violation: invalid rewrap on data_key id=%', OLD.id
        USING ERRCODE = 'insufficient_privilege';
    END IF;
    RETURN NEW;
  END IF;
  RAISE EXCEPTION 'immutable_violation: UPDATE on data_key rejected' USING ERRCODE = 'insufficient_privilege';
END;
$$;
REVOKE ALL ON FUNCTION util.fn_data_key_one_way() FROM PUBLIC;
COMMENT ON FUNCTION util.fn_data_key_one_way() IS
  'Trigger satu arah data_key/rme_data_key: pemusnahan oleh pekerja privasi, bungkus ulang oleh rotasi kunci (skema fisik §4.10)';
