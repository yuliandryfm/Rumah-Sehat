-- db/migrations/audit/20261119090000_util_shared_functions.sql
-- F1-T14: fungsi bersama skema util database audit. Satu direktori untuk KEDUA database audit
-- (ruang_sehat_audit dan rme_audit); perbedaan ditangani lewat current_database() (§11.1,
-- KEPUTUSAN-140).
--
-- Modul kritis T1 (ADR-009, ADR-005, ADR-008, ADR-021 butir 4): review DBA + keamanan.
-- Sumber: docs/05-physical-schema.md §4.2 (1), §5.2, §5.6, §11.1, §11.2; CR-020.
-- Maju saja (ADR-019); berkas ini tidak pernah diubah setelah diterapkan (§11.1).
--
-- Ketentuan:
-- - SECURITY INVOKER, search_path fungsi tetap "pg_catalog, pg_temp" (SRC-397, CR-020), kecuali
--   util.fn_current_fasyankes_id yang menurut §5.2 sengaja tanpa SET search_path (supaya dapat
--   di-inline) dan karena itu semua rujukannya berkualifikasi pg_catalog.
-- - EXECUTE dicabut dari PUBLIC. USAGE skema util hanya untuk rme_audit_tenant_reader di
--   rme_audit beserta EXECUTE util.fn_current_fasyankes_id (§5.6, definisi sama dengan §5.2).
--   Di ruang_sehat_audit tidak ada role yang diberi USAGE util.
-- - Migrasi ini TIDAK mengubah search_path sesi (SRC-399).

-- Penjaga (§11.2). Berkas pertama belum dapat memakai util.fn_audit_role (dibuat di sini), sehingga
-- pemetaan role ditulis langsung. Berkas berikutnya memeriksa current_user terhadap
-- util.fn_audit_role('audit_migration'); bila berkas ini belum diterapkan, penjaga itu gagal.
DO $$
DECLARE
  v_owner text := CASE pg_catalog.current_database()
                    WHEN 'ruang_sehat_audit' THEN 'rs_audit_migration'
                    WHEN 'rme_audit' THEN 'rme_audit_migration'
                  END;
BEGIN
  IF v_owner IS NULL THEN
    RAISE EXCEPTION 'migration order: db/migrations/audit must run on ruang_sehat_audit or rme_audit, got %',
      pg_catalog.current_database();
  END IF;
  IF current_user <> v_owner THEN
    RAISE EXCEPTION 'migration must run as %, got % (session %)', v_owner, current_user, session_user;
  END IF;
  IF NOT EXISTS (SELECT 1 FROM pg_catalog.pg_namespace
                  WHERE nspname = 'util' AND nspowner = v_owner::pg_catalog.regrole)
     OR NOT EXISTS (SELECT 1 FROM pg_catalog.pg_namespace
                     WHERE nspname = 'audit' AND nspowner = v_owner::pg_catalog.regrole)
     OR EXISTS (SELECT 1 FROM pg_catalog.pg_namespace WHERE nspname = 'public') THEN
    RAISE EXCEPTION 'migration order: bootstrap db/roles/audit.sql has not been applied to database % (run it before db/migrations/audit)',
      pg_catalog.current_database();
  END IF;
END $$;
SET LOCAL lock_timeout = '5s';
SET LOCAL statement_timeout = '15min';

-- (1) Penolak umum untuk tabel audit (kelas A) dan partisinya (§4.2 (1), §4.11, §6.1, §6.2).
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
  'Penolak UPDATE/DELETE/TRUNCATE tabel audit kelas A (skema fisik §4.2 (1), §6.1)';

-- (2) Pemetaan nama role logis DD §0.6 B ke nama fisik berawalan per database (§3.2 (b), §11.1).
--     Daftar nama logis tertutup; role hasil pemetaan wajib ada (gagal-tertutup). Dipakai penjaga
--     dan GRANT di migrasi audit berikutnya, bukan oleh role runtime.
CREATE FUNCTION util.fn_audit_role(p_logical_name text)
RETURNS text
LANGUAGE plpgsql
STABLE
SET search_path = pg_catalog, pg_temp
AS $$
DECLARE
  v_prefix text;
  v_role   text;
BEGIN
  v_prefix := CASE current_database()
                WHEN 'ruang_sehat_audit' THEN 'rs_'
                WHEN 'rme_audit' THEN 'rme_'
              END;
  IF v_prefix IS NULL THEN
    RAISE EXCEPTION 'audit_role_unknown_database: %', current_database()
      USING ERRCODE = 'invalid_parameter_value';
  END IF;
  IF p_logical_name IS NULL OR p_logical_name NOT IN
       ('audit_writer', 'audit_reader', 'audit_verifier', 'audit_tenant_reader', 'audit_migration') THEN
    RAISE EXCEPTION 'audit_role_invalid_name: %', coalesce(p_logical_name, '<null>')
      USING ERRCODE = 'invalid_parameter_value';
  END IF;
  v_role := v_prefix || p_logical_name;
  IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = v_role) THEN
    RAISE EXCEPTION 'audit_role_not_found: % (database %)', v_role, current_database()
      USING ERRCODE = 'invalid_parameter_value';
  END IF;
  RETURN v_role;
END;
$$;
REVOKE ALL ON FUNCTION util.fn_audit_role(text) FROM PUBLIC;
COMMENT ON FUNCTION util.fn_audit_role(text) IS
  'Nama role audit logis DD §0.6 B -> nama fisik rs_/rme_ menurut current_database() (skema fisik §11.1)';

-- (3) Fungsi konteks tenant, hanya di rme_audit (§5.6: definisi sama dengan §5.2).
DO $$
BEGIN
  IF pg_catalog.current_database() = 'rme_audit' THEN
    CREATE FUNCTION util.fn_current_fasyankes_id()
    RETURNS uuid
    LANGUAGE sql
    STABLE
    AS $f$ SELECT NULLIF(pg_catalog.current_setting('rme.fasyankes_id', true), '')::pg_catalog.uuid $f$;
    REVOKE ALL ON FUNCTION util.fn_current_fasyankes_id() FROM PUBLIC;
    COMMENT ON FUNCTION util.fn_current_fasyankes_id() IS
      'Konteks tenant transaksi (rme.fasyankes_id); NULL bila belum dipasang, gagal-tertutup (skema fisik §5.2, §5.6)';
    GRANT USAGE ON SCHEMA util TO rme_audit_tenant_reader;
    GRANT EXECUTE ON FUNCTION util.fn_current_fasyankes_id() TO rme_audit_tenant_reader;
  END IF;
END $$;
