-- db/migrations/app/20261122100000_profile_identity_rejection_reason_seed.sql
-- F1-T62: migrasi DATA daftar alasan penolakan identitas awal (A6.7 reason_code, konteks
-- identity_rejection). DML saja, tanpa DDL (docs/05-physical-schema.md §11.3: DDL dan DML terpisah,
-- migrasi data idempoten). Tanpa perubahan hak role.
--
-- Modul kritis T1 (ADR-008, ADR-021 butir 4): review DBA.
-- Sumber: PRD v1.4 US-201 AC3 (daftar alasan awal dikelola lewat migrasi data, tanpa layar admin;
-- `[USER] C2-B14`, UP-75 b, klarifikasi T-17); docs/04-data-dictionary.md A6.7; KEPUTUSAN-93 (data acuan
-- memakai UUID literal, sama di semua lingkungan; ID tetap UUIDv7, ADR-002).
--
-- Ketentuan:
-- - Tujuh kode persis US-201 AC3, urutan daftar PRD. Label = teks dalam kurung di PRD; "Label final
--   diselaraskan dengan pedoman teks UI" (US-201 AC3), sehingga penggantian label kelak lewat migrasi
--   data baru (UPDATE label), bukan layar admin (TC-US-201-21).
-- - Label `nik_unavailable` = teks umum US-106 AC4 (tidak menyebut akun lain, US-201 AC7).
-- - Idempoten: baris yang sudah ada (ID sama, atau kode aktif yang sama di konteks yang sama) tidak
--   disisipkan ulang (`ON CONFLICT DO NOTHING`).
-- - created_by NULL: aktor = migrasi (DD STD-M, kolom aktor boleh NULL untuk sistem).

-- Penjaga (§11.2) dari templat db/test/templat-penjaga-lanjutan-app.sql.
DO $$
BEGIN
  IF current_user <> 'rs_migration' THEN
    RAISE EXCEPTION 'migration must run as rs_migration, got % (session %)', current_user, session_user;
  END IF;
  IF EXISTS (SELECT 1 FROM app._sqlx_migrations WHERE version > 20261122100000) THEN
    RAISE EXCEPTION 'migration order: version 20261122100000 is older than an applied migration (forward-only, ADR-019)';
  END IF;
END $$;
SET LOCAL lock_timeout = '5s';
SET LOCAL statement_timeout = '15min';

INSERT INTO app.reason_code (id, context, code, label) VALUES
  ('01a1fbb5-46b5-7438-b0a4-16ef04d0d2d7', 'identity_rejection', 'photo_unreadable',
   'Foto dokumen buram atau tidak terbaca'),
  ('01a1fbb5-46b6-7bc6-83f9-ba413d5ceb2f', 'identity_rejection', 'document_invalid',
   'Dokumen bukan KTP/KK, tidak utuh, atau tidak sesuai jenis yang diminta'),
  ('01a1fbb5-46b7-7d0b-a9f6-61bfc9c84ccf', 'identity_rejection', 'nik_mismatch',
   'NIK yang diisi tidak sama dengan NIK di dokumen'),
  ('01a1fbb5-46b8-72d2-8fb4-7450e115b86a', 'identity_rejection', 'profile_mismatch',
   'Nama atau tanggal lahir di profil tidak sama dengan dokumen'),
  ('01a1fbb5-46b9-725a-949e-815a005f99dd', 'identity_rejection', 'guardian_not_listed',
   'Pemilik akun tidak tercantum sebagai orang tua/wali di Kartu Keluarga'),
  ('01a1fbb5-46ba-7286-911f-b123e709f97f', 'identity_rejection', 'nik_unavailable',
   'NIK ini tidak dapat dipakai untuk akun Anda. Hubungi Layanan Pelanggan bila ini NIK Anda.'),
  ('01a1fbb5-46bb-77b7-95ec-5a099c556716', 'identity_rejection', 'other',
   'Lainnya')
ON CONFLICT DO NOTHING;
