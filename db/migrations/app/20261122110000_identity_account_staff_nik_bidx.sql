-- db/migrations/app/20261122110000_identity_account_staff_nik_bidx.sql
-- F1-T62 (US-201 AC8, KEPUTUSAN-294 = jawaban user PT-57/T-38 opsi A): expand kolom blind index NIK
-- staf di akun admin Ruang Sehat. Diisi saat sub-peran Verifikator diberikan (modul identity, US-208),
-- dicocokkan dengan NIK pemilik akun pasien saat Verifikator memutuskan pengajuan identitas (pemisahan
-- tugas). Blind index saja, tanpa ciphertext: NIK staf tidak pernah dapat dibaca kembali.
--
-- Modul kritis T1 (ADR-008, ADR-013, ADR-021 butir 4): review DBA + keamanan.
-- Sumber: docs/05-physical-schema.md §4.1 butir 5, §7.2 (blind index: bytea 32 byte + versi kunci
-- smallint >= 1, KEPUTUSAN-105; diperiksa generik T-20), §4.3 (pekerja privasi: UPDATE kolom "anonymize"),
-- §11.3 (expand: kolom baru NULL tanpa backfill, kompatibel dengan kode versi sebelumnya); DD A1.1.
--
-- Ketentuan:
-- - Kolom NULL tanpa nilai bawaan; akun lama tanpa NIK staf (Verifikator tanpa NIK staf tidak dapat
--   memutuskan pengajuan apa pun: gagal-tertutup di aplikasi).
-- - Hanya akun rs_admin yang boleh memegang NIK staf; versi kunci wajib ada bila blind index ada
--   (versi boleh tersisa setelah anonimisasi mengosongkan blind index).
-- - Tanpa index: pencarian selalu per akun (PK). Tidak ada index unik: dua akun admin satu orang tidak
--   dicegah di sini (di luar ruang lingkup AC8).
-- - Pekerja privasi mendapat UPDATE kolom blind index (anonimisasi, templat §4.3). Maju saja (ADR-019).

-- Penjaga (§11.2) dari templat db/test/templat-penjaga-lanjutan-app.sql.
DO $$
BEGIN
  IF current_user <> 'rs_migration' THEN
    RAISE EXCEPTION 'migration must run as rs_migration, got % (session %)', current_user, session_user;
  END IF;
  IF EXISTS (SELECT 1 FROM app._sqlx_migrations WHERE version > 20261122110000) THEN
    RAISE EXCEPTION 'migration order: version 20261122110000 is older than an applied migration (forward-only, ADR-019)';
  END IF;
END $$;
SET LOCAL lock_timeout = '5s';
SET LOCAL statement_timeout = '15min';

ALTER TABLE app.account
  ADD COLUMN staff_nik_bidx             bytea    NULL,
  ADD COLUMN staff_nik_bidx_key_version smallint NULL,
  ADD CONSTRAINT ck_account_staff_nik_bidx CHECK (octet_length(staff_nik_bidx) = 32),
  ADD CONSTRAINT ck_account_staff_nik_bidx_key_version CHECK (staff_nik_bidx_key_version >= 1),
  ADD CONSTRAINT ck_account_staff_nik_bidx_rs_admin CHECK (
    staff_nik_bidx IS NULL OR account_type = 'rs_admin'),
  ADD CONSTRAINT ck_account_staff_nik_bidx_version_present CHECK (
    staff_nik_bidx IS NULL OR staff_nik_bidx_key_version IS NOT NULL);
COMMENT ON COLUMN app.account.staff_nik_bidx IS 'anonymize';

GRANT UPDATE (staff_nik_bidx) ON app.account TO rs_privacy_worker;   -- templat §4.3
