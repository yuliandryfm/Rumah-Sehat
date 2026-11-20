-- db/migrations/app/20261121100000_content_cms_content.sql
-- F1-T91: tabel A15.4 cms_content (kelas M) dengan kanal (aplikasi/situs) dan halaman tujuan situs + hak role.
--
-- Modul kritis T1 (file migrasi skema dan hak role database; ADR-003, ADR-008, ADR-021 butir 4): review DBA.
-- Sumber: docs/04-data-dictionary.md v1.2 §0.3, §0.4 (STD-M), §0.6 A, A15.4 dan usulan perubahan A15.4 di
-- bagian A20 ("kanal (`app`, `site`) dan halaman tujuan situs (F1-T91)"); docs/05-physical-schema.md §1, §4.1,
-- §4.3, §4.11, §11.2; PRD v1.4 US-1101 AC1-AC4, US-1801 AC1-AC3, US-1802 AC1-AC2, AC5; BR-130 (sembilan halaman
-- situs); ADR-027 butir 5 (hanya konten Tayang yang publik; gambar lewat endpoint backend).
-- Maju saja (ADR-019); koreksi lewat berkas baru (§11.1).
--
-- Ketentuan:
-- - id tanpa nilai bawaan (ADR-002); created_at/updated_at DEFAULT now() (KEPUTUSAN-93); FK tanpa CASCADE;
--   FK aktor dan image_file_id tidak diindeks (§4.1 butir 3-4).
-- - Hak hanya ke role grup (§4.1 butir 7): rs_app S, I, U (Master tanpa DELETE, ADR-003); pekerja privasi S.
--   Tabel ini tanpa kolom "Anonimkan" dan tanpa kolom *_enc (body Markdown tanpa data pribadi, DD A15.4).
-- - Kanal dan halaman situs (rancangan DD A20, rincian final di task ini):
--   channel CHECK (app, site, app_and_site) = "aplikasi, situs, atau keduanya" (US-1802 AC1);
--   site_page CHECK sembilan halaman BR-130, wajib terisi bila kanal memuat situs dan kosong untuk aplikasi saja.
-- - "Tayang" dihitung, bukan disimpan (DD A15.4): editorial_status = 'scheduled' dan now() di dalam jendela
--   [publish_start_at, publish_end_at). Konten terjadwal wajib punya publish_start_at.
-- - image_alt_text: teks alternatif gambar (US-1802 AC5, guideline §7); konten terjadwal yang bergambar wajib
--   punya teks alternatif tidak kosong. Kolom tambahan atas DD A15.4 (dicatat untuk pembaruan DD).
-- - Halaman tetap situs (info_page dengan site_page) paling banyak satu versi terjadwal tanpa batas akhir per
--   halaman (US-1802 AC2, uqp_cms_content_site_page_open); jendela yang tumpang tindih diatur service.
-- - Batas panjang sebagai pertahanan terakhir (coding-standards §5.1); batas yang sama divalidasi aplikasi.

-- Penjaga (§11.2) dari templat db/test/templat-penjaga-lanjutan-app.sql.
DO $$
BEGIN
  IF current_user <> 'rs_migration' THEN
    RAISE EXCEPTION 'migration must run as rs_migration, got % (session %)', current_user, session_user;
  END IF;
  IF EXISTS (SELECT 1 FROM app._sqlx_migrations WHERE version > 20261121100000) THEN
    RAISE EXCEPTION 'migration order: version 20261121100000 is older than an applied migration (forward-only, ADR-019)';
  END IF;
END $$;
SET LOCAL lock_timeout = '5s';
SET LOCAL statement_timeout = '15min';

-- A15.4 cms_content (M): banner, halaman info, artikel, dan promo informatif untuk aplikasi dan/atau situs.
CREATE TABLE app.cms_content (
  id               uuid        NOT NULL,
  created_at       timestamptz NOT NULL DEFAULT now(),
  created_by       uuid        NULL,
  updated_at       timestamptz NOT NULL DEFAULT now(),
  updated_by       uuid        NULL,
  deleted_at       timestamptz NULL,
  deleted_by       uuid        NULL,
  delete_reason    text        NULL,
  content_type     text        NOT NULL,
  title            text        NOT NULL,
  body             text        NULL,
  category         text        NULL,
  image_file_id    uuid        NULL,
  image_alt_text   text        NULL,
  link_target_type text        NULL,
  link_target_ref  text        NULL,
  publish_start_at timestamptz NULL,
  publish_end_at   timestamptz NULL,
  editorial_status text        NOT NULL,
  channel          text        NOT NULL,
  site_page        text        NULL,
  CONSTRAINT pk_cms_content PRIMARY KEY (id),
  CONSTRAINT fk_cms_content_created_by FOREIGN KEY (created_by) REFERENCES app.account (id),
  CONSTRAINT fk_cms_content_updated_by FOREIGN KEY (updated_by) REFERENCES app.account (id),
  CONSTRAINT fk_cms_content_deleted_by FOREIGN KEY (deleted_by) REFERENCES app.account (id),
  CONSTRAINT fk_cms_content_image_file_id FOREIGN KEY (image_file_id) REFERENCES app.stored_file (id),
  CONSTRAINT ck_cms_content_soft_delete CHECK (
    (deleted_at IS NULL AND deleted_by IS NULL AND delete_reason IS NULL)
    OR (deleted_at IS NOT NULL AND delete_reason IS NOT NULL)),
  CONSTRAINT ck_cms_content_content_type CHECK (content_type IN ('banner','info_page','article','promo')),
  CONSTRAINT ck_cms_content_link_target_type CHECK (link_target_type IN ('article','service','partner','product')),
  CONSTRAINT ck_cms_content_link_target CHECK ((link_target_type IS NULL) = (link_target_ref IS NULL)),
  CONSTRAINT ck_cms_content_publish_window CHECK (publish_end_at IS NULL OR publish_end_at > publish_start_at),
  CONSTRAINT ck_cms_content_editorial_status CHECK (editorial_status IN ('draft','scheduled','archived')),
  CONSTRAINT ck_cms_content_scheduled_start CHECK (editorial_status <> 'scheduled' OR publish_start_at IS NOT NULL),
  CONSTRAINT ck_cms_content_channel CHECK (channel IN ('app','site','app_and_site')),
  CONSTRAINT ck_cms_content_site_page CHECK (site_page IN
    ('home','about','services','for_business','partners','articles','contact','download_app','privacy_terms')),
  CONSTRAINT ck_cms_content_channel_site_page CHECK ((channel = 'app') = (site_page IS NULL)),
  CONSTRAINT ck_cms_content_image_alt_text CHECK (
    editorial_status <> 'scheduled' OR image_file_id IS NULL
    OR (image_alt_text IS NOT NULL AND btrim(image_alt_text) <> '')),
  CONSTRAINT ck_cms_content_title_length CHECK (char_length(title) BETWEEN 1 AND 200),
  CONSTRAINT ck_cms_content_body_length CHECK (body IS NULL OR char_length(body) <= 100000),
  CONSTRAINT ck_cms_content_alt_text_length CHECK (image_alt_text IS NULL OR char_length(image_alt_text) <= 300),
  CONSTRAINT ck_cms_content_category_length CHECK (category IS NULL OR char_length(category) <= 100),
  CONSTRAINT ck_cms_content_link_ref_length CHECK (link_target_ref IS NULL OR char_length(link_target_ref) <= 500),
  CONSTRAINT ck_cms_content_reason_length CHECK (delete_reason IS NULL OR char_length(delete_reason) <= 500)
);
-- DD A15.4: ix (content_type, editorial_status, publish_start_at) WHERE deleted_at IS NULL; nama lengkap 61 byte,
-- sehingga kolom diganti makna singkat (§1.2).
CREATE INDEX ix_cms_content_type_status_start
  ON app.cms_content (content_type, editorial_status, publish_start_at) WHERE deleted_at IS NULL;
-- Jalur publik situs per halaman (F1-T94).
CREATE INDEX ix_cms_content_site_page_status_start
  ON app.cms_content (site_page, editorial_status, publish_start_at)
  WHERE deleted_at IS NULL AND site_page IS NOT NULL;
-- US-1802 AC2: satu versi halaman tetap terjadwal tanpa batas akhir per halaman situs.
CREATE UNIQUE INDEX uqp_cms_content_site_page_open
  ON app.cms_content (site_page)
  WHERE deleted_at IS NULL AND content_type = 'info_page' AND editorial_status = 'scheduled'
    AND publish_end_at IS NULL AND site_page IS NOT NULL;   -- [SRC-33]
COMMENT ON TABLE app.cms_content IS 'class:M module:konten';

REVOKE ALL ON app.cms_content FROM PUBLIC;
GRANT SELECT, INSERT, UPDATE ON app.cms_content TO rs_app;          -- tanpa DELETE [DOK: ADR-003]
GRANT SELECT ON app.cms_content TO rs_privacy_worker;
-- tabel ini tanpa kolom "Anonimkan", sehingga privacy_worker tidak mendapat UPDATE per kolom
