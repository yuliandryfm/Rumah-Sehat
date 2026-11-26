-- db/migrations/app/20261126310000_privacy_consent_corporate_hr_text_draft_seed.sql
-- F1-T101 (US-1901 AC1): seed versi teks persetujuan HR perusahaan saat mendaftarkan perusahaan, bertanda DRAF,
-- beserta satu butir wajib, lewat migrasi data (pola migrasi 20261126200000, KEPUTUSAN-289).
--
-- Modul kritis T1 (persetujuan dan hak subjek data; ADR-008, ADR-021 butir 4): review DBA + keamanan.
-- Sumber: PRD US-1901 AC1 ("teks persetujuan HR menunggu L-20"), US-104 AC1 (tujuh bagian Pasal 21 ayat (1) UU
-- PDP) dan AC3; docs/04-data-dictionary.md A3.1, A3.2; KEPUTUSAN-289, KEPUTUSAN-321.
-- Maju saja (ADR-019); teks final yang disetujui legal dan Petugas PDP masuk sebagai versi baru sebelum go-live.
--
-- Ketentuan (sama dengan seed pasien 20261126200000):
-- - Isi `content` berformat JSON `rs-consent-text/1` dengan penanda `draft_notice`: production menolak
--   menyajikan dan menerima persetujuan atas versi draf (DraftPolicy::Reject).
-- - Seluruh teks isi bagian dan label butir adalah penanda DRAF, bukan teks hukum: legalitas menunggu L-20 (tim
--   legal), retensi dan jangka waktu menunggu PT-24 [TIDAK DIKETAHUI]. Label butir dan teks isi tidak
--   mengarang klaim hukum.
-- - effective_from = now() + 1 detik supaya penerbitan di transaksi yang sama lolos effective_lock (alasan di
--   seed 20261126200000). ID tetap (UUIDv7 literal); tanpa data pribadi.

-- Penjaga (§11.2) dari templat db/test/templat-penjaga-lanjutan-app.sql.
DO $$
BEGIN
  IF current_user <> 'rs_migration' THEN
    RAISE EXCEPTION 'migration must run as rs_migration, got % (session %)', current_user, session_user;
  END IF;
  IF EXISTS (SELECT 1 FROM app._sqlx_migrations WHERE version > 20261126310000) THEN
    RAISE EXCEPTION 'migration order: version 20261126310000 is older than an applied migration (forward-only, ADR-019)';
  END IF;
END $$;
SET LOCAL lock_timeout = '5s';
SET LOCAL statement_timeout = '15min';

INSERT INTO app.consent_text_version (id, consent_type, version_label, content, effective_from)
VALUES (
  '01a2008d-7400-7000-8000-000000000b01',
  'corporate_hr_registration',
  '0.1-draf',
  $json${
  "format": "rs-consent-text/1",
  "draft_notice": "DRAF: teks legalitas dan seluruh bagian menunggu L-20 (tim legal) dan Petugas PDP; retensi dan jangka waktu pemrosesan menunggu PT-24. Versi final wajib terbit sebagai versi baru sebelum go-live.",
  "sections": [
    {"code": "legal_basis", "title": "(a) Dasar hukum pemrosesan", "body": "[DRAF] Teks bagian ini menunggu L-20 (tim legal)."},
    {"code": "purpose", "title": "(b) Tujuan pemrosesan", "body": "[DRAF] Teks bagian ini menunggu L-20 dan Petugas PDP."},
    {"code": "data_types", "title": "(c) Jenis dan relevansi data yang diproses", "body": "[DRAF] Teks bagian ini menunggu L-20 dan Petugas PDP."},
    {"code": "document_retention", "title": "(d) Jangka waktu retensi dokumen yang memuat data pribadi", "body": "[DRAF] Teks bagian ini menunggu PT-24."},
    {"code": "collected_information", "title": "(e) Rincian informasi yang dikumpulkan", "body": "[DRAF] Teks bagian ini menunggu L-20 dan Petugas PDP."},
    {"code": "processing_period", "title": "(f) Jangka waktu pemrosesan", "body": "[DRAF] Teks bagian ini menunggu PT-24."},
    {"code": "subject_rights", "title": "(g) Hak Anda sebagai subjek data", "body": "[DRAF] Teks bagian ini menunggu L-20 dan Petugas PDP."}
  ],
  "contact": {"title": "Kontak Ruang Sehat", "body": "[DRAF] Kontak Ruang Sehat menunggu penetapan sebelum go-live."}
}$json$,
  now() + interval '1 second'
);

INSERT INTO app.consent_item (id, consent_text_version_id, item_code, label, is_primary, is_required, sort_order)
VALUES (
  '01a2008d-7400-7000-8000-000000000b02',
  '01a2008d-7400-7000-8000-000000000b01',
  'corporate_hr_data_processing',
  '[DRAF] Saya telah membaca dan menyetujui pemrosesan data pribadi saya dan data perusahaan saya untuk pendaftaran perusahaan, sesuai penjelasan di atas.',
  true,
  true,
  1
);

UPDATE app.consent_text_version
   SET published_at = now(), updated_at = now()
 WHERE id = '01a2008d-7400-7000-8000-000000000b01';
