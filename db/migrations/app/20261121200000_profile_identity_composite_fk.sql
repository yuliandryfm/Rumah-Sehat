-- db/migrations/app/20261121200000_profile_identity_composite_fk.sql
-- F1-T59 (TD-08, review F1-T18): FK komposit (id, care_recipient_id) supaya permintaan verifikasi,
-- penunjuk status terkini penerima layanan, dan registri NIK pemilik akun tidak dapat dikaitkan ke
-- dokumen atau permintaan milik penerima layanan LAIN (pola KEPUTUSAN-142: FK komposit sebagai penjaga
-- keterkaitan di database, selain pemeriksaan aplikasi).
--
-- Modul kritis T1 (ADR-008, ADR-021 butir 4): review DBA + keamanan.
-- Sumber: docs/04-data-dictionary.md A2.2, A2.4, A2.5, A2.8; docs/05-physical-schema.md §4.1, §4.7
-- (FK komposit proyeksi), §11.3 (expand: constraint baru tanpa backfill, tabel belum berisi data
-- production); docs/backlog.md TD-08.
--
-- Ketentuan:
-- - Expand saja: FK tunggal lama tetap ada (tidak di-drop; migrasi maju saja, ADR-019). FK komposit baru
--   memakai MATCH SIMPLE bawaan, sehingga `care_recipient.latest_verification_request_id` NULL ("Belum
--   diunggah") tetap sah.
-- - Sasaran FK komposit wajib constraint unik penuh (bukan partial index): UNIQUE (id, care_recipient_id)
--   di identity_document dan identity_verification_request. Keduanya trivially unik karena `id` adalah PK.
-- - Bagian TD-08 "registri hanya untuk pemilik akun" TIDAK ditegakkan di database oleh berkas ini
--   (butuh kolom baru di registri atau trigger; perubahan DD). Ditegakkan aplikasi (modul profile hanya
--   menyisipkan registri untuk penerima layanan `is_account_owner`) dan dilaporkan untuk keputusan DBA.
-- - Tanpa perubahan hak role. Maju saja (ADR-019).

-- Penjaga (§11.2) dari templat db/test/templat-penjaga-lanjutan-app.sql.
DO $$
BEGIN
  IF current_user <> 'rs_migration' THEN
    RAISE EXCEPTION 'migration must run as rs_migration, got % (session %)', current_user, session_user;
  END IF;
  IF EXISTS (SELECT 1 FROM app._sqlx_migrations WHERE version > 20261121200000) THEN
    RAISE EXCEPTION 'migration order: version 20261121200000 is older than an applied migration (forward-only, ADR-019)';
  END IF;
END $$;
SET LOCAL lock_timeout = '5s';
SET LOCAL statement_timeout = '15min';

-- Sasaran FK komposit.
ALTER TABLE app.identity_document
  ADD CONSTRAINT uq_identity_document_id_care_recipient_id UNIQUE (id, care_recipient_id);
ALTER TABLE app.identity_verification_request
  ADD CONSTRAINT uq_identity_verification_request_id_care_recipient_id UNIQUE (id, care_recipient_id);

-- Permintaan verifikasi hanya untuk dokumen penerima layanan yang sama.
ALTER TABLE app.identity_verification_request
  ADD CONSTRAINT fk_ivr_identity_document_id_care_recipient_id
  FOREIGN KEY (identity_document_id, care_recipient_id)
  REFERENCES app.identity_document (id, care_recipient_id);

-- Penunjuk status terkini hanya ke permintaan milik penerima layanan itu sendiri.
ALTER TABLE app.care_recipient
  ADD CONSTRAINT fk_care_recipient_latest_verification_request_id_id
  FOREIGN KEY (latest_verification_request_id, id)
  REFERENCES app.identity_verification_request (id, care_recipient_id);

-- Registri NIK pemilik akun hanya berdasar persetujuan permintaan penerima layanan yang sama.
ALTER TABLE app.account_owner_nik_registry
  ADD CONSTRAINT fk_owner_nik_registry_request_id_care_recipient_id
  FOREIGN KEY (identity_verification_request_id, care_recipient_id)
  REFERENCES app.identity_verification_request (id, care_recipient_id);
