-- db/migrations/app/20261126200000_privacy_consent_text_draft_seed.sql
-- F1-T49 (US-104): seed versi teks persetujuan pemrosesan data pasien bertanda DRAF dan butir
-- persetujuannya, lewat migrasi data (UP-75 a, KEPUTUSAN-12, klarifikasi T-10/T-11).
--
-- Modul kritis T1 (persetujuan dan hak subjek data; migrasi skema/data, ADR-008, ADR-021 butir 4):
-- review DBA + keamanan.
-- Sumber: PRD US-104 AC1-AC3; docs/07-klarifikasi-ac-fase-1.md T-10, T-11; docs/04-data-dictionary.md A3.1,
-- A3.2; KEPUTUSAN-289 (seed: effective_from = waktu migrasi, isi bertanda draf "menunggu L-08/PT-24",
-- terkunci effective_lock; teks final masuk sebagai versi baru sebelum go-live).
-- Maju saja (ADR-019); koreksi lewat berkas baru.
--
-- Ketentuan:
-- - Isi `content` berformat JSON `rs-consent-text/1` (dibaca modul privacy, apps/rs-backend/src/modules/privacy/
--   domain.rs): tujuh bagian Pasal 21 ayat (1) UU PDP dengan kode tetap (legal_basis, purpose, data_types,
--   document_retention, collected_information, processing_period, subject_rights) ditambah `contact`, serta
--   `draft_notice`. Teks isi bagian adalah penanda DRAF, bukan teks hukum: legalitas menunggu L-08, retensi dan
--   jangka waktu pemrosesan menunggu PT-24, bagian lain menunggu persetujuan legal dan Petugas PDP
--   [TIDAK DIKETAHUI] (US-104 AC1: wajib terisi sebelum go-live; pintu go-live TC-US-104-2).
-- - Waktu berlaku = waktu migrasi: effective_from = now() + 1 detik (now() = awal transaksi migrasi).
--   Urutan di satu transaksi: (1) versi disisipkan tanpa published_at, (2) butir disisipkan (published_lock
--   butir hanya menolak butir milik versi yang sudah terbit, §4.2 (3d)), (3) versi diterbitkan dengan UPDATE
--   published_at. Langkah (3) hanya lolos effective_lock (§4.2 (3): tolak bila effective_from lama/baru
--   <= now()) karena effective_from berada 1 detik setelah now(); dengan effective_from = now() persis,
--   UPDATE itu ditolak. Setelah migrasi, versi berlaku dan isinya terkunci effective_lock dan published_lock
--   (A3.1 §4.2 (3), (3c)); butirnya terkunci published_lock butir (§4.2 (3d)).
--   Akibatnya ada jeda 1 detik setelah migrasi selesai sebelum versi berlaku: permintaan teks dalam jeda itu
--   dijawab 503 (belum ada versi berlaku); tidak berdampak di deploy karena api baru dilayani sesudah migrasi.
-- - Label butir diambil dari wireframe A01 layar 3 (docs/wireframes/a01-registrasi-masuk.html, desain Tahap 6),
--   juga menunggu persetujuan copy/legal. Satu butir utama wajib (is_primary, is_required, US-101 AC4);
--   butir tambahan menyusul lewat versi baru (US-104 AC3).
-- - ID tetap (UUIDv7 literal) supaya data rujukan dapat dicek ulang; tidak ada data pribadi.

-- Penjaga (§11.2) dari templat db/test/templat-penjaga-lanjutan-app.sql.
DO $$
BEGIN
  IF current_user <> 'rs_migration' THEN
    RAISE EXCEPTION 'migration must run as rs_migration, got % (session %)', current_user, session_user;
  END IF;
  IF EXISTS (SELECT 1 FROM app._sqlx_migrations WHERE version > 20261126200000) THEN
    RAISE EXCEPTION 'migration order: version 20261126200000 is older than an applied migration (forward-only, ADR-019)';
  END IF;
END $$;
SET LOCAL lock_timeout = '5s';
SET LOCAL statement_timeout = '15min';

INSERT INTO app.consent_text_version (id, consent_type, version_label, content, effective_from)
VALUES (
  '01a2008d-7400-7000-8000-000000000a49',
  'patient_data_processing',
  '0.1-draf',
  $json${
  "format": "rs-consent-text/1",
  "draft_notice": "DRAF: teks legalitas menunggu L-08 (tim legal); teks retensi dan jangka waktu pemrosesan menunggu PT-24. Versi final yang disetujui legal dan Petugas PDP wajib terbit sebagai versi baru sebelum go-live.",
  "sections": [
    {"code": "legal_basis", "title": "(a) Dasar hukum pemrosesan", "body": "[DRAF] Teks bagian ini menunggu L-08 (tim legal)."},
    {"code": "purpose", "title": "(b) Tujuan pemrosesan", "body": "[DRAF] Teks bagian ini menunggu persetujuan legal dan Petugas PDP."},
    {"code": "data_types", "title": "(c) Jenis dan relevansi data yang diproses, termasuk data kesehatan", "body": "[DRAF] Teks bagian ini menunggu persetujuan legal dan Petugas PDP."},
    {"code": "document_retention", "title": "(d) Jangka waktu retensi dokumen yang memuat data pribadi", "body": "[DRAF] Teks bagian ini menunggu PT-24."},
    {"code": "collected_information", "title": "(e) Rincian informasi yang dikumpulkan", "body": "[DRAF] Teks bagian ini menunggu persetujuan legal dan Petugas PDP."},
    {"code": "processing_period", "title": "(f) Jangka waktu pemrosesan", "body": "[DRAF] Teks bagian ini menunggu PT-24."},
    {"code": "subject_rights", "title": "(g) Hak Anda sebagai subjek data", "body": "[DRAF] Teks bagian ini menunggu persetujuan legal dan Petugas PDP."}
  ],
  "contact": {"title": "Kontak Ruang Sehat", "body": "[DRAF] Kontak Ruang Sehat menunggu penetapan sebelum go-live."}
}$json$,
  now() + interval '1 second'
);

INSERT INTO app.consent_item (id, consent_text_version_id, item_code, label, is_primary, is_required, sort_order)
VALUES (
  '01a2008d-7400-7000-8000-000000000a4a',
  '01a2008d-7400-7000-8000-000000000a49',
  'data_processing',
  'Saya telah membaca dan menyetujui pemrosesan data pribadi saya sesuai penjelasan di atas.',
  true,
  true,
  1
);

-- Terbit di transaksi yang sama: baris belum berlaku (effective_from = now() + 1 detik), sehingga
-- effective_lock dan published_lock belum menolak; setelah UPDATE ini isi versi dan butirnya terkunci.
UPDATE app.consent_text_version
   SET published_at = now(), updated_at = now()
 WHERE id = '01a2008d-7400-7000-8000-000000000a49';
