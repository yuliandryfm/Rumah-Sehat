-- db/migrations/app/20261124201000_corporate_rejection_reason_seed.sql
-- F1-T104: migrasi DATA daftar alasan penolakan verifikasi perusahaan (A6.7 reason_code, konteks
-- corporate_rejection). DML saja, tanpa DDL (docs/05-physical-schema.md §11.3). Tanpa perubahan hak role.
--
-- Modul kritis T1 (ADR-008, ADR-021 butir 4): review DBA.
-- Sumber: PRD v1.3 US-1902 AC3 (menolak "dengan kode alasan (termasuk bukti kewenangan tidak memadai)");
-- US-1901 AC1 (data dan dokumen yang diperiksa: NIB, NPWP, bukti kewenangan pendaftar, `[USER] C21-P4` 4b);
-- KEPUTUSAN-93 (data acuan memakai UUID literal, sama di semua lingkungan; ID tetap UUIDv7, ADR-002).
--
-- STATUS DAFTAR: DRAF. PRD tidak menyebut daftar final alasan penolakan perusahaan; satu-satunya alasan yang
-- disebut PRD adalah "bukti kewenangan tidak memadai" (authority_proof_insufficient). Tiga kode lain adalah
-- usulan penyusun F1-T104 yang diturunkan dari isi pemeriksaan US-1901 AC1 (dokumen NIB, dokumen NPWP, data
-- perusahaan), `[TIDAK DIKETAHUI]` apakah disetujui user. Penggantian kode atau label kelak lewat migrasi
-- data baru (soft delete baris lama + sisip baris baru, atau UPDATE label), bukan layar admin; daftar tertutup
-- di kode (`corporate::domain::RejectionReason`) diperbarui di task yang sama.
--
-- Ketentuan:
-- - Idempoten: baris yang sudah ada (ID sama, atau kode aktif yang sama di konteks yang sama) tidak
--   disisipkan ulang (`ON CONFLICT DO NOTHING`).
-- - Tanpa kode `other`: endpoint penolakan perusahaan tidak menerima keterangan bebas (US-1902 AC3 hanya kode
--   alasan), sehingga "Lainnya" tanpa keterangan tidak dapat ditindaklanjuti HR.
-- - Label tidak menyebut perusahaan lain atau isi dokumen (KOR-08, M-52).
-- - created_by NULL: aktor = migrasi (DD STD-M, kolom aktor boleh NULL untuk sistem).

-- Penjaga (§11.2) dari templat db/test/templat-penjaga-lanjutan-app.sql.
DO $$
BEGIN
  IF current_user <> 'rs_migration' THEN
    RAISE EXCEPTION 'migration must run as rs_migration, got % (session %)', current_user, session_user;
  END IF;
  IF EXISTS (SELECT 1 FROM app._sqlx_migrations WHERE version > 20261124201000) THEN
    RAISE EXCEPTION 'migration order: version 20261124201000 is older than an applied migration (forward-only, ADR-019)';
  END IF;
END $$;
SET LOCAL lock_timeout = '5s';
SET LOCAL statement_timeout = '15min';

INSERT INTO app.reason_code (id, context, code, label) VALUES
  ('01a20f24-ce00-73b1-857a-14a3c9462341', 'corporate_rejection', 'nib_document_invalid',
   'Dokumen NIB tidak terbaca, tidak sah, atau tidak sesuai NIB yang diisi'),
  ('01a20f24-ce01-717a-a44b-c2d779b486b7', 'corporate_rejection', 'npwp_document_invalid',
   'Dokumen NPWP tidak terbaca, tidak sah, atau tidak sesuai NPWP yang diisi'),
  ('01a20f24-ce02-77c5-8412-ddab61ba42b4', 'corporate_rejection', 'company_data_mismatch',
   'Data perusahaan tidak sesuai dengan dokumen'),
  ('01a20f24-ce03-76b9-b9f6-5772c63ca9c3', 'corporate_rejection', 'authority_proof_insufficient',
   'Bukti kewenangan pendaftar tidak memadai')
ON CONFLICT DO NOTHING;
