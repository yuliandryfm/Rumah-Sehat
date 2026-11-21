-- db/migrations/app/20261123100000_corporate_client_verification.sql
-- F1-T98: tabel A20 K1 bagian 1, yaitu corporate_client (M), corporate_client_document (M), dan
-- corporate_verification_decision (T), beserta hak role.
--
-- Modul kritis T1 (file migrasi skema dan hak role database; ADR-003, ADR-004, ADR-008, ADR-012, ADR-021 butir 4):
-- review DBA + keamanan.
-- Sumber: docs/04-data-dictionary.md v1.2 §0.2-§0.6 (STD-M, STD-T, hak §0.6 A), A20 (rancangan; rincian kolom,
-- constraint, index, dan hak ditetapkan di task ini); docs/05-physical-schema.md §1, §4.1, §4.3, §4.5, §4.11, §11.2;
-- PRD v1.3 US-1901 AC1/AC3/AC7, US-1902 AC1-AC4, US-1903 AC1/AC4, UP-79; ADR-027 butir 3; ADR-028 butir 1.
-- Maju saja (ADR-019); koreksi lewat berkas baru (§11.1).
--
-- Ketentuan:
-- - id tanpa nilai bawaan (ADR-002); created_at/updated_at DEFAULT now() (KEPUTUSAN-93); FK tanpa CASCADE;
--   FK aktor dan FK yang sudah tercakup index unik tidak diberi index tambahan (§4.1 butir 3-4).
-- - Hak hanya ke role grup (§4.1 butir 7); TRUNCATE tidak diberikan; Master tanpa DELETE (ADR-003);
--   Transaksi hanya S, I dengan trigger penolak tingkat statement (ADR-004, KEPUTUSAN-95).
-- - Tiga tabel ini tanpa kolom "Anonimkan" di DD A20: corporate_client berisi data badan usaha; PII pengurus
--   hanya ada di isi berkas (stored_file, dimusnahkan lewat purged_at, ADR-018). Pekerja privasi mendapat S pada
--   tabel Master (§0.6 A) dan tidak mendapat hak di corporate_verification_decision, karena tabel itu tidak punya
--   FK subjek data pribadi (subjeknya badan usaha; pemutus dirujuk lewat ID akun).
-- - Format resmi NIB dan NPWP [TIDAK DIKETAHUI] (US-1901 AC3; diverifikasi ke sumber resmi di F1-T101). Database
--   hanya menolak nilai kosong dan terlalu panjang; validasi format ada di aplikasi.
-- - NIB ganda ditolak lewat uqp_corporate_client_nib (US-1901 AC7, US-1903 AC4, [USER] C21-P4 4c), termasuk NIB
--   perusahaan berstatus rejected/inactive selama belum dihapus lunak.
-- - Pengajuan ulang setelah ditolak (US-1902 AC3): submission_round naik satu dan submitted_at diperbarui oleh
--   aplikasi. "Satu keputusan per pengajuan, keputusan pertama menang" (DD A20, US-1902 AC4) dijamin
--   uq_corp_verif_decision_corporate_client_id_submission_round.
-- - Dokumen: satu dokumen aktif per jenis per perusahaan (nib, npwp, authority_proof; bukti kewenangan pendaftar
--   wajib, [USER] C21-P4 4b). Penggantian dokumen = hapus lunak baris lama lalu sisip baris baru.
-- - reason_code penolakan berupa teks kode (pola identity_verification_request_status_event); konteks reason_code
--   untuk penolakan perusahaan belum ada di A6.7 dan ditambahkan task backend verifikasi (F1-T104).
-- - Batas panjang sebagai pertahanan terakhir (coding-standards §5.1); batas yang sama divalidasi aplikasi.
-- - Nama > 60 byte memakai alias tabel di komentar (§1.2): corp_client_doc, corp_verif_decision.

-- Penjaga (§11.2) dari templat db/test/templat-penjaga-lanjutan-app.sql.
DO $$
BEGIN
  IF current_user <> 'rs_migration' THEN
    RAISE EXCEPTION 'migration must run as rs_migration, got % (session %)', current_user, session_user;
  END IF;
  IF EXISTS (SELECT 1 FROM app._sqlx_migrations WHERE version > 20261123100000) THEN
    RAISE EXCEPTION 'migration order: version 20261123100000 is older than an applied migration (forward-only, ADR-019)';
  END IF;
END $$;
SET LOCAL lock_timeout = '5s';
SET LOCAL statement_timeout = '15min';

-- A20 corporate_client (M): perusahaan pelanggan layanan korporat (tenant, ADR-028 butir 1).
CREATE TABLE app.corporate_client (
  id               uuid        NOT NULL,
  created_at       timestamptz NOT NULL DEFAULT now(),
  created_by       uuid        NULL,
  updated_at       timestamptz NOT NULL DEFAULT now(),
  updated_by       uuid        NULL,
  deleted_at       timestamptz NULL,
  deleted_by       uuid        NULL,
  delete_reason    text        NULL,
  legal_name       text        NOT NULL,
  nib              text        NOT NULL,
  npwp             text        NOT NULL,
  office_address   text        NOT NULL,
  office_phone     text        NOT NULL,
  status           text        NOT NULL,
  origin           text        NOT NULL,
  submission_round integer     NOT NULL DEFAULT 1,
  submitted_at     timestamptz NOT NULL DEFAULT now(),
  CONSTRAINT pk_corporate_client PRIMARY KEY (id),
  CONSTRAINT fk_corporate_client_created_by FOREIGN KEY (created_by) REFERENCES app.account (id),
  CONSTRAINT fk_corporate_client_updated_by FOREIGN KEY (updated_by) REFERENCES app.account (id),
  CONSTRAINT fk_corporate_client_deleted_by FOREIGN KEY (deleted_by) REFERENCES app.account (id),
  CONSTRAINT ck_corporate_client_soft_delete CHECK (
    (deleted_at IS NULL AND deleted_by IS NULL AND delete_reason IS NULL)
    OR (deleted_at IS NOT NULL AND delete_reason IS NOT NULL)),
  CONSTRAINT ck_corporate_client_status CHECK (status IN ('pending_verification','active','rejected','inactive')),
  CONSTRAINT ck_corporate_client_origin CHECK (origin IN ('self_registered','created_by_admin')),
  CONSTRAINT ck_corporate_client_submission_round CHECK (submission_round >= 1),
  CONSTRAINT ck_corporate_client_legal_name_length CHECK (char_length(btrim(legal_name)) BETWEEN 1 AND 200),
  CONSTRAINT ck_corporate_client_nib_length CHECK (char_length(btrim(nib)) BETWEEN 1 AND 64),
  -- Review DBA F1-T98: NIB disimpan tanpa spasi tepi, supaya ' 123' tidak lolos UQp sebagai NIB lain.
  CONSTRAINT ck_corporate_client_nib_trimmed CHECK (nib = btrim(nib)),
  CONSTRAINT ck_corporate_client_npwp_length CHECK (char_length(btrim(npwp)) BETWEEN 1 AND 64),
  CONSTRAINT ck_corporate_client_office_address_length CHECK (char_length(btrim(office_address)) BETWEEN 1 AND 500),
  CONSTRAINT ck_corporate_client_office_phone_length CHECK (char_length(btrim(office_phone)) BETWEEN 1 AND 32),
  CONSTRAINT ck_corporate_client_reason_length CHECK (delete_reason IS NULL OR char_length(delete_reason) <= 500)
);
CREATE UNIQUE INDEX uqp_corporate_client_nib
  ON app.corporate_client (nib) WHERE deleted_at IS NULL;   -- [SRC-33], US-1901 AC7
-- US-1902 AC1: antrean verifikasi urut dari pengajuan terlama.
CREATE INDEX ix_corporate_client_submitted_at_pending
  ON app.corporate_client (submitted_at) WHERE deleted_at IS NULL AND status = 'pending_verification';
COMMENT ON TABLE app.corporate_client IS 'class:M module:korporat';

REVOKE ALL ON app.corporate_client FROM PUBLIC;
GRANT SELECT, INSERT, UPDATE ON app.corporate_client TO rs_app;    -- tanpa DELETE [DOK: ADR-003]
GRANT SELECT ON app.corporate_client TO rs_privacy_worker;
-- tabel ini tanpa kolom "Anonimkan", sehingga privacy_worker tidak mendapat UPDATE per kolom

-- A20 corporate_client_document (M): dokumen NIB, NPWP, dan bukti kewenangan pendaftar (US-1901 AC1).
CREATE TABLE app.corporate_client_document (
  id                  uuid        NOT NULL,
  created_at          timestamptz NOT NULL DEFAULT now(),
  created_by          uuid        NULL,
  updated_at          timestamptz NOT NULL DEFAULT now(),
  updated_by          uuid        NULL,
  deleted_at          timestamptz NULL,
  deleted_by          uuid        NULL,
  delete_reason       text        NULL,
  corporate_client_id uuid        NOT NULL,
  document_kind       text        NOT NULL,
  stored_file_id      uuid        NOT NULL,
  CONSTRAINT pk_corporate_client_document PRIMARY KEY (id),
  CONSTRAINT fk_corporate_client_document_created_by FOREIGN KEY (created_by) REFERENCES app.account (id),
  CONSTRAINT fk_corporate_client_document_updated_by FOREIGN KEY (updated_by) REFERENCES app.account (id),
  CONSTRAINT fk_corporate_client_document_deleted_by FOREIGN KEY (deleted_by) REFERENCES app.account (id),
  CONSTRAINT fk_corporate_client_document_corporate_client_id
    FOREIGN KEY (corporate_client_id) REFERENCES app.corporate_client (id),
  CONSTRAINT fk_corporate_client_document_stored_file_id FOREIGN KEY (stored_file_id) REFERENCES app.stored_file (id),
  CONSTRAINT uq_corporate_client_document_stored_file_id UNIQUE (stored_file_id),   -- satu berkas satu dokumen
  CONSTRAINT ck_corporate_client_document_soft_delete CHECK (
    (deleted_at IS NULL AND deleted_by IS NULL AND delete_reason IS NULL)
    OR (deleted_at IS NOT NULL AND delete_reason IS NOT NULL)),
  CONSTRAINT ck_corporate_client_document_document_kind CHECK (document_kind IN ('nib','npwp','authority_proof')),
  CONSTRAINT ck_corporate_client_document_reason_length CHECK (delete_reason IS NULL OR char_length(delete_reason) <= 500)
);
-- Satu dokumen aktif per jenis per perusahaan; index ini juga melayani FK corporate_client_id.
CREATE UNIQUE INDEX uqp_corp_client_doc_corporate_client_id_document_kind
  ON app.corporate_client_document (corporate_client_id, document_kind) WHERE deleted_at IS NULL;   -- [SRC-33]
COMMENT ON TABLE app.corporate_client_document IS 'class:M module:korporat alias:corp_client_doc';

REVOKE ALL ON app.corporate_client_document FROM PUBLIC;
GRANT SELECT, INSERT, UPDATE ON app.corporate_client_document TO rs_app;   -- tanpa DELETE [DOK: ADR-003]
GRANT SELECT ON app.corporate_client_document TO rs_privacy_worker;
-- tabel ini tanpa kolom "Anonimkan" (PII pengurus hanya di isi berkas), sehingga tanpa UPDATE per kolom

-- A20 corporate_verification_decision (T): keputusan Verifikator atas satu pengajuan (US-1902 AC2-AC4).
CREATE TABLE app.corporate_verification_decision (
  id                  uuid        NOT NULL,
  created_at          timestamptz NOT NULL DEFAULT now(),
  created_by          uuid        NULL,
  corporate_client_id uuid        NOT NULL,
  submission_round    integer     NOT NULL,
  decision            text        NOT NULL,
  reason_code         text        NULL,
  decided_by          uuid        NOT NULL,
  CONSTRAINT pk_corporate_verification_decision PRIMARY KEY (id),
  CONSTRAINT fk_corporate_verification_decision_created_by FOREIGN KEY (created_by) REFERENCES app.account (id),
  CONSTRAINT fk_corporate_verification_decision_corporate_client_id
    FOREIGN KEY (corporate_client_id) REFERENCES app.corporate_client (id),
  CONSTRAINT fk_corporate_verification_decision_decided_by FOREIGN KEY (decided_by) REFERENCES app.account (id),
  -- Keputusan pertama menang (US-1902 AC4, pola AB-36); juga melayani FK corporate_client_id.
  CONSTRAINT uq_corp_verif_decision_corporate_client_id_submission_round UNIQUE (corporate_client_id, submission_round),
  CONSTRAINT ck_corporate_verification_decision_submission_round CHECK (submission_round >= 1),
  CONSTRAINT ck_corporate_verification_decision_decision CHECK (decision IN ('approved','rejected')),
  -- Penolakan wajib ber-kode alasan (US-1902 AC3); persetujuan tanpa kode alasan.
  CONSTRAINT ck_corporate_verification_decision_reason_code CHECK ((decision = 'rejected') = (reason_code IS NOT NULL)),
  CONSTRAINT ck_corporate_verification_decision_reason_length CHECK (reason_code IS NULL OR char_length(reason_code) BETWEEN 1 AND 64)
);
COMMENT ON TABLE app.corporate_verification_decision IS 'class:T module:korporat alias:corp_verif_decision';

REVOKE ALL ON app.corporate_verification_decision FROM PUBLIC;
GRANT SELECT, INSERT ON app.corporate_verification_decision TO rs_app;

CREATE TRIGGER tg_corporate_verification_decision_reject_ud
  BEFORE UPDATE OR DELETE ON app.corporate_verification_decision
  FOR EACH STATEMENT EXECUTE FUNCTION util.fn_reject_modification();
CREATE TRIGGER tg_corporate_verification_decision_reject_truncate
  BEFORE TRUNCATE ON app.corporate_verification_decision
  FOR EACH STATEMENT EXECUTE FUNCTION util.fn_reject_modification();   -- TRUNCATE hanya tingkat statement [SRC-40]
