-- db/test/tabel-app-a2.sql: test tabel A2 profil dan verifikasi identitas + A6.7 reason_code (F1-T18)
-- di database ruang_sehat: patient, care_recipient, address, identity_document,
-- identity_verification_request (+ _status_event, _state), account_owner_nik_registry, reason_code.
-- Didahului db/test/lib.sql; dijalankan superuser setelah migrasi. Aturan kelas generik
-- (T-06/T-07/T-08) di db/test/kelas-app.sql bagian A, T-20 generik di db/test/katalog.sql, DDL umum
-- §4.1 di db/test/tabel-app-a1.sql. Berkas ini memeriksa struktur khusus (komentar kelas, kolom
-- anonymize, index, FK komposit/sirkular, daftar kolom T-20) dan perilaku dengan role sungguhan
-- (positif dan negatif). Satu transaksi yang di-ROLLBACK. Data sintetis (NIK, nama, dan alamat
-- rekaan; ciphertext dan blind index berupa byte tiruan).
-- Dasar: docs/04-data-dictionary.md §0.2, §0.6 A, §0.7, A2.1-A2.8, A6.7; docs/05-physical-schema.md
-- §1.2, §4.1 butir 5, §4.3, §4.5-§4.7, §7.1, §7.2, §12 (T-06, T-07, T-08, T-09, T-20); ADR-003,
-- ADR-004, ADR-008, ADR-013 §4-§5; PRD v1.3 US-106 AC4-AC5, US-201 AC3, AC5, AC7, UP-40.

-- 1. Struktur.
SELECT pg_temp.uji_nol('A2/A6.7 komentar kelas dan alias (§1.2): M profil, T/R/P dengan alias ivr, registri NIK, reason_code platform',
  ARRAY(SELECT format('%s: %s', t, coalesce(obj_description(to_regclass('app.' || t), 'pg_class'), '<tidak ada>'))
          FROM (VALUES ('patient', 'class:M module:profil'), ('care_recipient', 'class:M module:profil'),
                       ('address', 'class:M module:profil'), ('identity_document', 'class:M module:profil'),
                       ('identity_verification_request', 'class:T module:profil alias:ivr'),
                       ('identity_verification_request_status_event', 'class:R module:profil alias:ivr_status_event'),
                       ('identity_verification_request_state', 'class:P module:profil alias:ivr_state'),
                       ('account_owner_nik_registry', 'class:M module:profil alias:owner_nik_registry'),
                       ('reason_code', 'class:M module:platform')) v(t, k)
         WHERE obj_description(to_regclass('app.' || t), 'pg_class') IS DISTINCT FROM k));
SELECT pg_temp.uji_nol('A2 kolom anonymize persis kolom "Anonimkan" DD A2.1-A2.4, A2.8; T/R/P dan reason_code tanpa kolom anonymize',
  ARRAY(SELECT format('%s: %s, diharapkan %s', t, coalesce(a.k, '<kosong>'), coalesce(v.k, '<kosong>'))
          FROM (VALUES ('patient', 'phone'),
                       ('care_recipient', 'birth_date,full_name,nik_bidx,nik_last4,sex'),
                       ('address', 'address_line,city_code,district_code,latitude,longitude,notes,postal_code,province_code,recipient_name,recipient_phone'),
                       ('identity_document', 'nik_bidx,nik_last4'),
                       ('identity_verification_request', NULL), ('identity_verification_request_status_event', NULL),
                       ('identity_verification_request_state', NULL),
                       ('account_owner_nik_registry', 'nik_bidx'), ('reason_code', NULL)) v(t, k),
               LATERAL (SELECT string_agg(attname::text, ',' ORDER BY attname) AS k FROM pg_attribute
                         WHERE attrelid = to_regclass('app.' || v.t) AND attnum > 0 AND NOT attisdropped
                           AND col_description(attrelid, attnum) = 'anonymize') a
         WHERE a.k IS DISTINCT FROM v.k));
SELECT pg_temp.uji_nol('A2 kolom anonymized_at ada di setiap tabel Master ber-"Anonimkan" (DD §0.2/§0.6 A, T-07)',
  ARRAY(SELECT t FROM unnest(ARRAY['patient', 'care_recipient', 'address', 'identity_document', 'account_owner_nik_registry']) t
         WHERE NOT EXISTS (SELECT 1 FROM pg_attribute WHERE attrelid = to_regclass('app.' || t) AND attname = 'anonymized_at'
                              AND atttypid = 'timestamptz'::regtype AND NOT attnotnull AND NOT attisdropped)));
SELECT pg_temp.uji_nol('A2 hak per kolom pekerja privasi persis (T-07 DD §0.6 A; T: id, created_at, FK subjek; R: id, note_data_key_id)',
  ARRAY(SELECT format('%s %s: %s, diharapkan %s', v.t, v.hak, coalesce(a.k, '<kosong>'), v.k)
          FROM (VALUES ('patient', 'UPDATE', 'anonymized_at,phone,updated_at,updated_by'),
                       ('care_recipient', 'UPDATE', 'anonymized_at,birth_date,full_name,nik_bidx,nik_last4,sex,updated_at,updated_by'),
                       ('address', 'UPDATE', 'address_line,anonymized_at,city_code,district_code,latitude,longitude,notes,postal_code,province_code,recipient_name,recipient_phone,updated_at,updated_by'),
                       ('identity_document', 'UPDATE', 'anonymized_at,nik_bidx,nik_last4,updated_at,updated_by'),
                       ('account_owner_nik_registry', 'UPDATE', 'anonymized_at,delete_reason,deleted_at,deleted_by,nik_bidx,updated_at,updated_by'),
                       ('identity_verification_request', 'SELECT', 'care_recipient_id,created_at,id'),
                       ('identity_verification_request_status_event', 'SELECT', 'id,note_data_key_id')) v(t, hak, k),
               LATERAL (SELECT string_agg(att.attname::text, ',' ORDER BY att.attname) AS k
                          FROM pg_attribute att, aclexplode(att.attacl) x
                         WHERE att.attrelid = to_regclass('app.' || v.t) AND att.attnum > 0 AND NOT att.attisdropped
                           AND pg_get_userbyid(x.grantee) = 'rs_privacy_worker' AND x.privilege_type = v.hak) a
         WHERE a.k IS DISTINCT FROM v.k));
SELECT pg_temp.uji_nol('A2/A6.7 tanpa hak untuk rs_audit_relay, rs_key_rotation, rs_ops_monitor, PUBLIC (tabel maupun kolom); hak kolom hanya pekerja privasi',
  ARRAY(SELECT format('%s: %s', c.relname, CASE WHEN a.grantee = 0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END)
          FROM pg_class c, aclexplode(c.relacl) a
         WHERE c.oid = ANY (ARRAY['app.patient', 'app.care_recipient', 'app.address', 'app.identity_document',
                                  'app.identity_verification_request', 'app.identity_verification_request_status_event',
                                  'app.identity_verification_request_state', 'app.account_owner_nik_registry',
                                  'app.reason_code']::regclass[]::oid[])
           AND (a.grantee = 0 OR pg_get_userbyid(a.grantee) IN ('rs_audit_relay', 'rs_key_rotation', 'rs_ops_monitor'))
        UNION ALL
        SELECT format('%s.%s: %s', att.attrelid::regclass, att.attname, CASE WHEN a.grantee = 0 THEN 'PUBLIC' ELSE pg_get_userbyid(a.grantee) END)
          FROM pg_attribute att, aclexplode(att.attacl) a
         WHERE att.attrelid = ANY (ARRAY['app.patient', 'app.care_recipient', 'app.address', 'app.identity_document',
                                         'app.identity_verification_request', 'app.identity_verification_request_status_event',
                                         'app.identity_verification_request_state', 'app.account_owner_nik_registry',
                                         'app.reason_code']::regclass[]::oid[])
           AND (a.grantee = 0 OR pg_get_userbyid(a.grantee) <> 'rs_privacy_worker')));
SELECT pg_temp.uji_nol('T-20 tidak hampa: 10 kolom terenkripsi/blind index skema app diperiksa persis (A2 + NIK staf A1.1, KEPUTUSAN-294)',
  ARRAY(SELECT x FROM (
    SELECT coalesce(string_agg(relname || '.' || k, ',' ORDER BY relname, k), '<kosong>') AS x
      FROM pg_temp.uji_t20_kolom('{app}')) s
   WHERE x <> 'account.staff_nik_bidx,account.staff_nik_bidx_key_version,'
            || 'account_owner_nik_registry.nik_bidx,account_owner_nik_registry.nik_bidx_key_version,'
            || 'care_recipient.nik_bidx,care_recipient.nik_bidx_key_version,'
            || 'identity_document.nik_bidx,identity_document.nik_bidx_key_version,identity_document.nik_enc,'
            || 'identity_verification_request_status_event.note_enc'));

SELECT pg_temp.uji_nol('A6.7 reason_code: daftar alasan penolakan identitas awal persis tujuh kode US-201 AC3 (migrasi data F1-T62)',
  ARRAY(SELECT x FROM (
    SELECT coalesce(string_agg(code, ',' ORDER BY id), '<kosong>') AS x
      FROM app.reason_code WHERE context = 'identity_rejection' AND deleted_at IS NULL) s
   WHERE x <> 'photo_unreadable,document_invalid,nik_mismatch,profile_mismatch,guardian_not_listed,nik_unavailable,other'));

SELECT pg_temp.uji_nol('A6.7 reason_code: index persis pk, uqp (context, code) aktif',
  ARRAY(SELECT pg_temp.uji_indeks('app.reason_code', ARRAY[
    'CREATE UNIQUE INDEX pk_reason_code ON app.reason_code USING btree (id)',
    'CREATE UNIQUE INDEX uqp_reason_code_context_code ON app.reason_code USING btree (context, code) WHERE (deleted_at IS NULL)'])));
SELECT pg_temp.uji_nol('A2.1 patient: index persis pk, uqp account_id aktif',
  ARRAY(SELECT pg_temp.uji_indeks('app.patient', ARRAY[
    'CREATE UNIQUE INDEX pk_patient ON app.patient USING btree (id)',
    'CREATE UNIQUE INDEX uqp_patient_account_id ON app.patient USING btree (account_id) WHERE (deleted_at IS NULL)'])));
SELECT pg_temp.uji_nol('A2.2 care_recipient: index persis pk, uqp pemilik akun, uqp (patient_id, nik_bidx), ix patient_id aktif',
  ARRAY(SELECT pg_temp.uji_indeks('app.care_recipient', ARRAY[
    'CREATE UNIQUE INDEX pk_care_recipient ON app.care_recipient USING btree (id)',
    'CREATE UNIQUE INDEX uqp_care_recipient_patient_id_account_owner ON app.care_recipient USING btree (patient_id) WHERE (is_account_owner AND (deleted_at IS NULL))',
    'CREATE UNIQUE INDEX uqp_care_recipient_patient_id_nik_bidx ON app.care_recipient USING btree (patient_id, nik_bidx) WHERE ((deleted_at IS NULL) AND (nik_bidx IS NOT NULL))',
    'CREATE INDEX ix_care_recipient_patient_id ON app.care_recipient USING btree (patient_id) WHERE (deleted_at IS NULL)'])));
SELECT pg_temp.uji_nol('A2.3 address: index persis pk, uqp alamat utama, ix patient_id aktif',
  ARRAY(SELECT pg_temp.uji_indeks('app.address', ARRAY[
    'CREATE UNIQUE INDEX pk_address ON app.address USING btree (id)',
    'CREATE UNIQUE INDEX uqp_address_patient_id_default ON app.address USING btree (patient_id) WHERE (is_default AND (deleted_at IS NULL))',
    'CREATE INDEX ix_address_patient_id ON app.address USING btree (patient_id) WHERE (deleted_at IS NULL)'])));
SELECT pg_temp.uji_nol('A2.4 identity_document: index persis pk, uq data_key_id, uq (id, care_recipient_id) sasaran FK komposit (TD-08), ix (care_recipient_id, created_at DESC)',
  ARRAY(SELECT pg_temp.uji_indeks('app.identity_document', ARRAY[
    'CREATE UNIQUE INDEX pk_identity_document ON app.identity_document USING btree (id)',
    'CREATE UNIQUE INDEX uq_identity_document_data_key_id ON app.identity_document USING btree (data_key_id)',
    'CREATE UNIQUE INDEX uq_identity_document_id_care_recipient_id ON app.identity_document USING btree (id, care_recipient_id)',
    'CREATE INDEX ix_identity_document_care_recipient_id_created_at ON app.identity_document USING btree (care_recipient_id, created_at DESC)'])));
SELECT pg_temp.uji_nol('A2.5 identity_verification_request: index persis pk, uq (id, care_recipient_id) sasaran FK komposit (TD-08), ix (care_recipient_id, created_at DESC)',
  ARRAY(SELECT pg_temp.uji_indeks('app.identity_verification_request', ARRAY[
    'CREATE UNIQUE INDEX pk_identity_verification_request ON app.identity_verification_request USING btree (id)',
    'CREATE UNIQUE INDEX uq_identity_verification_request_id_care_recipient_id ON app.identity_verification_request USING btree (id, care_recipient_id)',
    'CREATE INDEX ix_ivr_care_recipient_id_created_at ON app.identity_verification_request USING btree (care_recipient_id, created_at DESC)'])));
SELECT pg_temp.uji_nol('A2.6 status_event: index persis pk, uq (request, version) AB-36, uq (request, version, to_status) sasaran FK proyeksi',
  ARRAY(SELECT pg_temp.uji_indeks('app.identity_verification_request_status_event', ARRAY[
    'CREATE UNIQUE INDEX pk_identity_verification_request_status_event ON app.identity_verification_request_status_event USING btree (id)',
    'CREATE UNIQUE INDEX uq_ivr_status_event_identity_verification_request_id_version ON app.identity_verification_request_status_event USING btree (identity_verification_request_id, version)',
    'CREATE UNIQUE INDEX uq_ivr_status_event_request_id_version_to_status ON app.identity_verification_request_status_event USING btree (identity_verification_request_id, version, to_status)'])));
SELECT pg_temp.uji_nol('A2.7 state: index persis pk, ix antrean pending (US-201 AC1)',
  ARRAY(SELECT pg_temp.uji_indeks('app.identity_verification_request_state', ARRAY[
    'CREATE UNIQUE INDEX pk_identity_verification_request_state ON app.identity_verification_request_state USING btree (identity_verification_request_id)',
    'CREATE INDEX ix_ivr_state_status_updated_at_pending ON app.identity_verification_request_state USING btree (status, updated_at) WHERE (status = ''pending''::text)'])));
SELECT pg_temp.uji_nol('A2.8 account_owner_nik_registry: index persis pk, uqp nik_bidx aktif, uqp care_recipient_id aktif',
  ARRAY(SELECT pg_temp.uji_indeks('app.account_owner_nik_registry', ARRAY[
    'CREATE UNIQUE INDEX pk_account_owner_nik_registry ON app.account_owner_nik_registry USING btree (id)',
    'CREATE UNIQUE INDEX uqp_account_owner_nik_registry_nik_bidx ON app.account_owner_nik_registry USING btree (nik_bidx) WHERE (deleted_at IS NULL)',
    'CREATE UNIQUE INDEX uqp_account_owner_nik_registry_care_recipient_id ON app.account_owner_nik_registry USING btree (care_recipient_id) WHERE (deleted_at IS NULL)'])));
SELECT pg_temp.uji_nol('A2 FK khusus: sirkular care_recipient -> request terkini, komposit proyeksi -> riwayat (KEPUTUSAN-142), wadah dan berkas dokumen',
  ARRAY(SELECT x FROM (VALUES
      ('app.care_recipient', 'fk_care_recipient_latest_verification_request_id', 'FOREIGN KEY (latest_verification_request_id) REFERENCES app.identity_verification_request(id)'),
      ('app.identity_verification_request_state', 'fk_ivr_state_request_id_version_status', 'FOREIGN KEY (identity_verification_request_id, version, status) REFERENCES app.identity_verification_request_status_event(identity_verification_request_id, version, to_status)'),
      ('app.identity_document', 'fk_identity_document_data_key_id', 'FOREIGN KEY (data_key_id) REFERENCES app.data_key(id)'),
      ('app.identity_document', 'fk_identity_document_file_id', 'FOREIGN KEY (file_id) REFERENCES app.stored_file(id)'),
      ('app.identity_verification_request_status_event', 'fk_ivr_status_event_note_data_key_id', 'FOREIGN KEY (note_data_key_id) REFERENCES app.data_key(id)'),
      ('app.account_owner_nik_registry', 'fk_owner_nik_registry_identity_verification_request_id', 'FOREIGN KEY (identity_verification_request_id) REFERENCES app.identity_verification_request(id)'),
      -- TD-08 (F1-T59): FK komposit (id, care_recipient_id) mencegah kaitan lintas penerima layanan.
      ('app.identity_verification_request', 'fk_ivr_identity_document_id_care_recipient_id', 'FOREIGN KEY (identity_document_id, care_recipient_id) REFERENCES app.identity_document(id, care_recipient_id)'),
      ('app.care_recipient', 'fk_care_recipient_latest_verification_request_id_id', 'FOREIGN KEY (latest_verification_request_id, id) REFERENCES app.identity_verification_request(id, care_recipient_id)'),
      ('app.account_owner_nik_registry', 'fk_owner_nik_registry_request_id_care_recipient_id', 'FOREIGN KEY (identity_verification_request_id, care_recipient_id) REFERENCES app.identity_verification_request(id, care_recipient_id)')) v(t, n, def),
        LATERAL (SELECT format('%s: %s', v.n, coalesce(pg_get_constraintdef(k.oid), '<tidak ada>')) AS x
                   FROM (SELECT 1) d LEFT JOIN pg_constraint k ON k.conname = v.n AND k.conrelid = v.t::regclass) z
         WHERE x IS DISTINCT FROM format('%s: %s', v.n, v.def)));
SELECT pg_temp.uji_nol('A2 tanpa trigger non-util dan tanpa fungsi app.* baru (penolak T/R lewat util, §4.11)',
  ARRAY(SELECT format('%s.%s -> %s', g.tgrelid::regclass, g.tgname, g.tgfoid::regproc)
          FROM pg_trigger g
         WHERE g.tgrelid = ANY (ARRAY['app.patient', 'app.care_recipient', 'app.address', 'app.identity_document',
                                      'app.identity_verification_request', 'app.identity_verification_request_status_event',
                                      'app.identity_verification_request_state', 'app.account_owner_nik_registry',
                                      'app.reason_code']::regclass[]::oid[])
           AND NOT g.tgisinternal AND g.tgfoid::regproc::text NOT LIKE 'util.%'));

-- 2. Negatif atas pemeriksa, lalu perilaku. Semua di-ROLLBACK.
BEGIN;
DO $$
DECLARE
  c record;
  kelas text := $q$SELECT pg_temp.uji_kelas_app('app', '{}')$q$;
  t20 text := $q$SELECT pg_temp.uji_t20('{app}')$q$;
BEGIN
  FOR c IN SELECT * FROM (VALUES
      ('T-06 NIK terenkripsi dibuka untuk rotasi kunci', 'GRANT SELECT (nik_enc) ON app.identity_document TO rs_key_rotation', kelas, 'T-06 identity_document .*hak per kolom tidak diizinkan rs_key_rotation:SELECT'),
      ('T-06 NIK terenkripsi dibuka untuk relay audit', 'GRANT SELECT ON app.identity_document TO rs_audit_relay', kelas, 'T-06 identity_document .*lebih rs_audit_relay:SELECT'),
      ('T-07 pekerja privasi mengubah nik_enc', 'GRANT UPDATE (nik_enc) ON app.identity_document TO rs_privacy_worker', kelas, 'T-07 identity_document'),
      ('T-07 pekerja privasi mengubah care_recipient_id registri', 'GRANT UPDATE (care_recipient_id) ON app.account_owner_nik_registry TO rs_privacy_worker', kelas, 'T-07 account_owner_nik_registry'),
      ('T-06 pekerja privasi membaca trigger_type (bukan ID/FK subjek)', 'GRANT SELECT (trigger_type) ON app.identity_verification_request TO rs_privacy_worker', kelas, 'T-06 identity_verification_request: SELECT per kolom rs_privacy_worker'),
      ('T-06 pekerja privasi membaca note_enc riwayat', 'GRANT SELECT (note_enc) ON app.identity_verification_request_status_event TO rs_privacy_worker', kelas, 'T-06 identity_verification_request_status_event: SELECT per kolom rs_privacy_worker'),
      ('T-06 UPDATE riwayat untuk rs_app', 'GRANT UPDATE ON app.identity_verification_request_status_event TO rs_app', kelas, 'T-06 identity_verification_request_status_event .*lebih rs_app:UPDATE'),
      ('T-06 DELETE proyeksi untuk rs_app', 'GRANT DELETE ON app.identity_verification_request_state TO rs_app', kelas, 'T-06 identity_verification_request_state .*lebih rs_app:DELETE'),
      ('T-06 DELETE registri NIK untuk rs_app', 'GRANT DELETE ON app.account_owner_nik_registry TO rs_app', kelas, 'T-06 account_owner_nik_registry .*lebih rs_app:DELETE'),
      ('T-08 penolak riwayat status hilang', 'DROP TRIGGER tg_identity_verification_request_status_event_reject_ud ON app.identity_verification_request_status_event', kelas, 'T-08 identity_verification_request_status_event \(kelas R\)'),
      ('T-08 penolak TRUNCATE permintaan verifikasi nonaktif', 'ALTER TABLE app.identity_verification_request DISABLE TRIGGER tg_identity_verification_request_reject_truncate', kelas, 'T-08 identity_verification_request: trigger tg_identity_verification_request_reject_truncate tidak aktif'),
      ('T-20 CHECK amplop nik_enc dihapus', 'ALTER TABLE app.identity_document DROP CONSTRAINT ck_identity_document_nik_enc_format', t20, 'T-20 app.identity_document.nik_enc: tanpa CHECK'),
      ('T-20 CHECK blind index registri dihapus', 'ALTER TABLE app.account_owner_nik_registry DROP CONSTRAINT ck_account_owner_nik_registry_nik_bidx', t20, 'T-20 app.account_owner_nik_registry.nik_bidx: tanpa CHECK'),
      ('T-20 CHECK amplop note_enc riwayat dihapus', 'ALTER TABLE app.identity_verification_request_status_event DROP CONSTRAINT ck_ivr_status_event_note_enc_format', t20, 'T-20 app.identity_verification_request_status_event.note_enc: tanpa CHECK')
    ) AS t(label, stmt, pemeriksa, pola)
  LOOP
    PERFORM pg_temp.uji_rusak(c.label, c.stmt, c.pemeriksa, c.pola);
  END LOOP;
END $$;

-- Data awal oleh pemilik skema (seperti seed migrasi). Data sintetis.
SET LOCAL ROLE rs_migration;
INSERT INTO app.account (id, account_type, idp_realm, idp_subject, email, status, status_changed_at, full_name)
  VALUES ('00000000-0000-7000-8000-0000000a2001', 'patient', 'pasien', 'sub-a2-1', 'a2-1@contoh.invalid', 'active', now(), NULL),
         ('00000000-0000-7000-8000-0000000a2002', 'patient', 'pasien', 'sub-a2-2', 'a2-2@contoh.invalid', 'active', now(), NULL),
         ('00000000-0000-7000-8000-0000000a2003', 'rs_admin', 'admin', 'sub-a2-3', 'a2-3@contoh.invalid', 'active', now(), 'Verifikator Sintetis');
INSERT INTO app.data_key (id, container_type, container_id, wrapped_dek, kek_ref)
  VALUES ('00000000-0000-7000-8000-0000000a2401', 'identity_document', '00000000-0000-7000-8000-0000000a2501', '\x01', 'openbao-transit:transit:rs-dek-kek:v1'),
         ('00000000-0000-7000-8000-0000000a2402', 'identity_document', '00000000-0000-7000-8000-0000000a2502', '\x01', 'openbao-transit:transit:rs-dek-kek:v1'),
         ('00000000-0000-7000-8000-0000000a2403', 'identity_document', '00000000-0000-7000-8000-0000000a2503', '\x01', 'openbao-transit:transit:rs-dek-kek:v1'),
         ('00000000-0000-7000-8000-0000000a2404', 'identity_document', '00000000-0000-7000-8000-0000000a2504', '\x01', 'openbao-transit:transit:rs-dek-kek:v1'),
         ('00000000-0000-7000-8000-0000000a2411', 'status_note', '00000000-0000-7000-8000-0000000a2601', '\x01', 'openbao-transit:transit:rs-dek-kek:v1');
INSERT INTO app.stored_file (id, bucket, object_key, category, owner_account_id, container_type, container_id, mime_type, size_bytes, sha256, validated_at)
  SELECT ('00000000-0000-7000-8000-0000000a230' || n)::uuid, 'identity_document', 'obj-a2-' || n, 'ktp_photo',
         '00000000-0000-7000-8000-0000000a2001', 'identity_document', ('00000000-0000-7000-8000-0000000a250' || n)::uuid,
         'image/jpeg', 1024, '\x00', now()
    FROM generate_series(1, 4) n;
RESET ROLE;

-- reason_code (M, A6.7).
SELECT pg_temp.uji_boleh('A6.7 rs_api INSERT alasan penolakan identitas', 'rs_api',
  'INSERT INTO app.reason_code (id, context, code, label) VALUES (''00000000-0000-7000-8000-0000000a2901'', ''identity_rejection'', ''uji_alasan_sintetis'', ''Label sintetis'')', 1);
SELECT pg_temp.uji_tolak('A6.7 NEG kode ganda dalam konteks yang sama (aktif)', 'rs_api',
  'INSERT INTO app.reason_code (id, context, code, label) VALUES (''00000000-0000-7000-8000-0000000a2902'', ''identity_rejection'', ''uji_alasan_sintetis'', ''x'')',
  '23505', 'uqp_reason_code_context_code');
SELECT pg_temp.uji_boleh('A6.7 kode sama di konteks lain diterima', 'rs_api',
  'INSERT INTO app.reason_code (id, context, code, label) VALUES (''00000000-0000-7000-8000-0000000a2903'', ''claim_rejection'', ''nik_unavailable'', ''x'')', 1);
SELECT pg_temp.uji_tolak('A6.7 NEG konteks di luar daftar', 'rs_api',
  'INSERT INTO app.reason_code (id, context, code, label) VALUES (''00000000-0000-7000-8000-0000000a2904'', ''lainnya'', ''x'', ''x'')',
  '23514', 'ck_reason_code_context');
SELECT pg_temp.uji_tolak('A6.7 NEG rs_api DELETE reason_code (Master)', 'rs_api', 'DELETE FROM app.reason_code', '42501', 'permission denied for table reason_code');
SELECT pg_temp.uji_tolak('A6.7 NEG pekerja privasi UPDATE reason_code (tanpa kolom anonymize)', 'rs_privacy_worker_login',
  'UPDATE app.reason_code SET label = ''x''', '42501', 'permission denied for table reason_code');

-- patient (M, A2.1).
SELECT pg_temp.uji_boleh('A2.1 rs_api membuat profil pasien untuk dua akun', 'rs_api',
  'INSERT INTO app.patient (id, account_id, phone, processing_status) VALUES (''00000000-0000-7000-8000-0000000a2101'', ''00000000-0000-7000-8000-0000000a2001'', ''+620000000001'', ''normal''), (''00000000-0000-7000-8000-0000000a2102'', ''00000000-0000-7000-8000-0000000a2002'', NULL, ''normal'')', 2);
SELECT pg_temp.uji_tolak('A2.1 NEG dua profil aktif untuk satu akun', 'rs_api',
  'INSERT INTO app.patient (id, account_id, processing_status) VALUES (''00000000-0000-7000-8000-0000000a2103'', ''00000000-0000-7000-8000-0000000a2001'', ''normal'')',
  '23505', 'uqp_patient_account_id');
SELECT pg_temp.uji_tolak('A2.1 NEG processing_status di luar daftar', 'rs_api',
  'UPDATE app.patient SET processing_status = ''frozen'' WHERE id = ''00000000-0000-7000-8000-0000000a2101''', '23514', 'ck_patient_processing_status');
SELECT pg_temp.uji_boleh('A2.1 pembatasan pemrosesan (US-1603 AC6) dengan waktu perubahan', 'rs_api',
  'UPDATE app.patient SET processing_status = ''restricted'', processing_status_changed_at = now(), updated_at = now() WHERE id = ''00000000-0000-7000-8000-0000000a2102''', 1);
SELECT pg_temp.uji_tolak('A2.1 NEG pekerja privasi mengubah processing_status', 'rs_privacy_worker_login',
  'UPDATE app.patient SET processing_status = ''normal''', '42501', 'permission denied for table patient');

-- care_recipient (M, A2.2): satu pemilik akun per pasien, NIK unik dalam satu akun (US-108 AC4-AC5),
-- NIK anggota keluarga boleh di akun lain (UP-40).
SELECT pg_temp.uji_boleh('A2.2 rs_api membuat pemilik akun dan anggota keluarga', 'rs_api',
  'INSERT INTO app.care_recipient (id, patient_id, is_account_owner, relationship, full_name, birth_date, sex, nik_bidx, nik_bidx_key_version, nik_last4) VALUES '
  || '(''00000000-0000-7000-8000-0000000a2201'', ''00000000-0000-7000-8000-0000000a2101'', true, NULL, ''Pemilik Sintetis'', ''1990-01-01'', ''female'', decode(repeat(''11'', 32), ''hex''), 1, ''0001''), '
  || '(''00000000-0000-7000-8000-0000000a2202'', ''00000000-0000-7000-8000-0000000a2101'', false, ''child'', ''Anak Sintetis'', ''2020-01-01'', ''male'', decode(repeat(''22'', 32), ''hex''), 1, ''0002''), '
  || '(''00000000-0000-7000-8000-0000000a2203'', ''00000000-0000-7000-8000-0000000a2102'', true, NULL, ''Pemilik Kedua'', ''1985-05-05'', ''male'', NULL, NULL, NULL)', 3);
SELECT pg_temp.uji_tolak('A2.2 NEG dua pemilik akun aktif untuk satu pasien', 'rs_api',
  'INSERT INTO app.care_recipient (id, patient_id, is_account_owner, full_name, birth_date, sex) VALUES (''00000000-0000-7000-8000-0000000a2204'', ''00000000-0000-7000-8000-0000000a2101'', true, ''x'', ''1990-01-01'', ''male'')',
  '23505', 'uqp_care_recipient_patient_id_account_owner');
SELECT pg_temp.uji_tolak('A2.2 NEG NIK anggota keluarga sama dengan NIK pemilik akun di akun yang sama (US-108 AC4)', 'rs_api',
  'INSERT INTO app.care_recipient (id, patient_id, is_account_owner, relationship, full_name, birth_date, sex, nik_bidx, nik_bidx_key_version) VALUES (''00000000-0000-7000-8000-0000000a2204'', ''00000000-0000-7000-8000-0000000a2101'', false, ''spouse'', ''x'', ''1990-01-01'', ''male'', decode(repeat(''11'', 32), ''hex''), 1)',
  '23505', 'uqp_care_recipient_patient_id_nik_bidx');
SELECT pg_temp.uji_boleh('A2.2 NIK anggota keluarga yang sama boleh di akun lain (UP-40)', 'rs_api',
  'INSERT INTO app.care_recipient (id, patient_id, is_account_owner, relationship, full_name, birth_date, sex, nik_bidx, nik_bidx_key_version) VALUES (''00000000-0000-7000-8000-0000000a2205'', ''00000000-0000-7000-8000-0000000a2102'', false, ''child'', ''Anak Sintetis'', ''2020-01-01'', ''male'', decode(repeat(''22'', 32), ''hex''), 1)', 1);
SELECT pg_temp.uji_tolak('A2.2 NEG pemilik akun dengan relationship', 'rs_api',
  'INSERT INTO app.care_recipient (id, patient_id, is_account_owner, relationship, full_name, birth_date, sex) VALUES (''00000000-0000-7000-8000-0000000a2206'', ''00000000-0000-7000-8000-0000000a2102'', true, ''self'', ''x'', ''1990-01-01'', ''male'')',
  '23514', 'ck_care_recipient_relationship');
SELECT pg_temp.uji_tolak('A2.2 NEG anggota keluarga tanpa relationship', 'rs_api',
  'INSERT INTO app.care_recipient (id, patient_id, is_account_owner, full_name, birth_date, sex) VALUES (''00000000-0000-7000-8000-0000000a2206'', ''00000000-0000-7000-8000-0000000a2102'', false, ''x'', ''1990-01-01'', ''male'')',
  '23514', 'ck_care_recipient_relationship');
SELECT pg_temp.uji_tolak('A2.2 NEG sex di luar daftar', 'rs_api',
  'UPDATE app.care_recipient SET sex = ''x'' WHERE id = ''00000000-0000-7000-8000-0000000a2201''', '23514', 'ck_care_recipient_sex');
SELECT pg_temp.uji_tolak('A2.2 NEG blind index NIK bukan 32 byte (T-20)', 'rs_api',
  'UPDATE app.care_recipient SET nik_bidx = decode(repeat(''33'', 31), ''hex'') WHERE id = ''00000000-0000-7000-8000-0000000a2203''', '23514', 'ck_care_recipient_nik_bidx');
SELECT pg_temp.uji_tolak('A2.2 NEG versi kunci blind index 0 (T-20)', 'rs_api',
  'UPDATE app.care_recipient SET nik_bidx_key_version = 0 WHERE id = ''00000000-0000-7000-8000-0000000a2201''', '23514', 'ck_care_recipient_nik_bidx_key_version');
SELECT pg_temp.uji_boleh('A2.2 hapus anggota keluarga = soft delete (US-109 AC3)', 'rs_api',
  'UPDATE app.care_recipient SET deleted_at = now(), deleted_by = ''00000000-0000-7000-8000-0000000a2001'', delete_reason = ''dihapus pasien'', updated_at = now() WHERE id = ''00000000-0000-7000-8000-0000000a2202''', 1);
SELECT pg_temp.uji_boleh('A2.2 anggota keluarga yang dihapus ditambah kembali sebagai baris baru (US-109 AC4)', 'rs_api',
  'INSERT INTO app.care_recipient (id, patient_id, is_account_owner, relationship, full_name, birth_date, sex, nik_bidx, nik_bidx_key_version) VALUES (''00000000-0000-7000-8000-0000000a2207'', ''00000000-0000-7000-8000-0000000a2101'', false, ''child'', ''Anak Sintetis'', ''2020-01-01'', ''male'', decode(repeat(''22'', 32), ''hex''), 1)', 1);
SELECT pg_temp.uji_tolak('A2.2 NEG rs_api DELETE care_recipient (Master)', 'rs_api', 'DELETE FROM app.care_recipient', '42501', 'permission denied for table care_recipient');
SELECT pg_temp.uji_tolak('A2.2 NEG request terkini yang tidak ada (FK sirkular)', 'rs_api',
  'UPDATE app.care_recipient SET latest_verification_request_id = ''00000000-0000-7000-8000-00000000ffff'' WHERE id = ''00000000-0000-7000-8000-0000000a2201''',
  '23503', 'fk_care_recipient_latest_verification_request_id');

-- address (M, A2.3).
SELECT pg_temp.uji_boleh('A2.3 rs_api menambah alamat utama', 'rs_api',
  'INSERT INTO app.address (id, patient_id, label, recipient_name, recipient_phone, address_line, province_name, city_name, latitude, longitude, time_zone, is_default) VALUES (''00000000-0000-7000-8000-0000000a2a01'', ''00000000-0000-7000-8000-0000000a2101'', ''Rumah'', ''Penerima Sintetis'', ''+620000000001'', ''Jalan Sintetis 1'', ''Provinsi Uji'', ''Kota Uji'', -6.200000, 106.816666, ''Asia/Jakarta'', true)', 1);
SELECT pg_temp.uji_tolak('A2.3 NEG dua alamat utama aktif untuk satu pasien', 'rs_api',
  'INSERT INTO app.address (id, patient_id, recipient_name, recipient_phone, address_line, province_name, city_name, time_zone, is_default) VALUES (''00000000-0000-7000-8000-0000000a2a02'', ''00000000-0000-7000-8000-0000000a2101'', ''x'', ''x'', ''x'', ''x'', ''x'', ''Asia/Jakarta'', true)',
  '23505', 'uqp_address_patient_id_default');
SELECT pg_temp.uji_boleh('A2.3 alamat jamak (US-105 AC2): alamat kedua bukan utama', 'rs_api',
  'INSERT INTO app.address (id, patient_id, recipient_name, recipient_phone, address_line, province_name, city_name, time_zone, is_default) VALUES (''00000000-0000-7000-8000-0000000a2a02'', ''00000000-0000-7000-8000-0000000a2101'', ''x'', ''x'', ''x'', ''x'', ''x'', ''Asia/Jakarta'', false)', 1);
SELECT pg_temp.uji_boleh('A2.3 pekerja privasi menganonimkan kolom alamat', 'rs_privacy_worker_login',
  'UPDATE app.address SET recipient_name = ''Anonim'', recipient_phone = ''-'', address_line = ''-'', province_code = NULL, city_code = NULL, district_code = NULL, postal_code = NULL, latitude = NULL, longitude = NULL, notes = NULL, anonymized_at = now(), updated_at = now(), updated_by = NULL WHERE patient_id = ''00000000-0000-7000-8000-0000000a2101''', 2);
SELECT pg_temp.uji_tolak('A2.3 NEG pekerja privasi mengubah label/is_default (bukan Anonimkan)', 'rs_privacy_worker_login',
  'UPDATE app.address SET is_default = false', '42501', 'permission denied for table address');

-- identity_document (M, A2.4): NIK terenkripsi (amplop v1, T-20), blind index 32 byte, satu DEK per dokumen.
SELECT pg_temp.uji_boleh('A2.4 rs_api menyimpan dokumen dengan ciphertext amplop v1 (29 byte minimum)', 'rs_api',
  'INSERT INTO app.identity_document (id, care_recipient_id, document_type, nik_enc, nik_bidx, nik_bidx_key_version, nik_last4, data_key_id, file_id) VALUES '
  || '(''00000000-0000-7000-8000-0000000a2501'', ''00000000-0000-7000-8000-0000000a2201'', ''ktp'', decode(''01'' || repeat(''ab'', 28), ''hex''), decode(repeat(''11'', 32), ''hex''), 1, ''0001'', ''00000000-0000-7000-8000-0000000a2401'', ''00000000-0000-7000-8000-0000000a2301''), '
  || '(''00000000-0000-7000-8000-0000000a2502'', ''00000000-0000-7000-8000-0000000a2203'', ''ktp'', decode(''01'' || repeat(''cd'', 44), ''hex''), decode(repeat(''11'', 32), ''hex''), 1, ''0001'', ''00000000-0000-7000-8000-0000000a2402'', ''00000000-0000-7000-8000-0000000a2302'')', 2);
SELECT pg_temp.uji_tolak('A2.4 NEG NIK teks terbuka di nik_enc (16 digit ASCII, byte pertama bukan 0x01)', 'rs_api',
  'INSERT INTO app.identity_document (id, care_recipient_id, document_type, nik_enc, nik_bidx, nik_bidx_key_version, nik_last4, data_key_id, file_id) VALUES (''00000000-0000-7000-8000-0000000a2503'', ''00000000-0000-7000-8000-0000000a2207'', ''kk'', convert_to(''3200000000000001'', ''UTF8''), decode(repeat(''22'', 32), ''hex''), 1, ''0001'', ''00000000-0000-7000-8000-0000000a2403'', ''00000000-0000-7000-8000-0000000a2303'')',
  '23514', 'ck_identity_document_nik_enc_format');
SELECT pg_temp.uji_tolak('A2.4 NEG NIK teks terbuka 29 byte (panjang cukup, byte versi bukan 0x01)', 'rs_api',
  'INSERT INTO app.identity_document (id, care_recipient_id, document_type, nik_enc, nik_bidx, nik_bidx_key_version, nik_last4, data_key_id, file_id) VALUES (''00000000-0000-7000-8000-0000000a2503'', ''00000000-0000-7000-8000-0000000a2207'', ''kk'', convert_to(''NIK 3200000000000001 sintetis'', ''UTF8''), decode(repeat(''22'', 32), ''hex''), 1, ''0001'', ''00000000-0000-7000-8000-0000000a2403'', ''00000000-0000-7000-8000-0000000a2303'')',
  '23514', 'ck_identity_document_nik_enc_format');
SELECT pg_temp.uji_tolak('A2.4 NEG amplop terpotong 28 byte berawalan 0x01', 'rs_api',
  'INSERT INTO app.identity_document (id, care_recipient_id, document_type, nik_enc, nik_bidx, nik_bidx_key_version, nik_last4, data_key_id, file_id) VALUES (''00000000-0000-7000-8000-0000000a2503'', ''00000000-0000-7000-8000-0000000a2207'', ''kk'', decode(''01'' || repeat(''ab'', 27), ''hex''), decode(repeat(''22'', 32), ''hex''), 1, ''0001'', ''00000000-0000-7000-8000-0000000a2403'', ''00000000-0000-7000-8000-0000000a2303'')',
  '23514', 'ck_identity_document_nik_enc_format');
SELECT pg_temp.uji_tolak('A2.4 NEG nik_enc kosong (0 byte) ditolak', 'rs_api',
  'INSERT INTO app.identity_document (id, care_recipient_id, document_type, nik_enc, nik_bidx, nik_bidx_key_version, nik_last4, data_key_id, file_id) VALUES (''00000000-0000-7000-8000-0000000a2503'', ''00000000-0000-7000-8000-0000000a2207'', ''kk'', ''\x'', decode(repeat(''22'', 32), ''hex''), 1, ''0001'', ''00000000-0000-7000-8000-0000000a2403'', ''00000000-0000-7000-8000-0000000a2303'')',
  '23514', 'ck_identity_document_nik_enc_format');
SELECT pg_temp.uji_tolak('A2.4 NEG nik_enc NULL', 'rs_api',
  'INSERT INTO app.identity_document (id, care_recipient_id, document_type, nik_bidx, nik_bidx_key_version, nik_last4, data_key_id, file_id) VALUES (''00000000-0000-7000-8000-0000000a2503'', ''00000000-0000-7000-8000-0000000a2207'', ''kk'', decode(repeat(''22'', 32), ''hex''), 1, ''0001'', ''00000000-0000-7000-8000-0000000a2403'', ''00000000-0000-7000-8000-0000000a2303'')',
  '23502', 'nik_enc');
SELECT pg_temp.uji_tolak('A2.4 NEG blind index 33 byte', 'rs_api',
  'INSERT INTO app.identity_document (id, care_recipient_id, document_type, nik_enc, nik_bidx, nik_bidx_key_version, nik_last4, data_key_id, file_id) VALUES (''00000000-0000-7000-8000-0000000a2503'', ''00000000-0000-7000-8000-0000000a2207'', ''kk'', decode(''01'' || repeat(''ab'', 28), ''hex''), decode(repeat(''22'', 33), ''hex''), 1, ''0001'', ''00000000-0000-7000-8000-0000000a2403'', ''00000000-0000-7000-8000-0000000a2303'')',
  '23514', 'ck_identity_document_nik_bidx');
SELECT pg_temp.uji_tolak('A2.4 NEG satu DEK untuk dua dokumen', 'rs_api',
  'INSERT INTO app.identity_document (id, care_recipient_id, document_type, nik_enc, nik_bidx, nik_bidx_key_version, nik_last4, data_key_id, file_id) VALUES (''00000000-0000-7000-8000-0000000a2503'', ''00000000-0000-7000-8000-0000000a2207'', ''kk'', decode(''01'' || repeat(''ab'', 28), ''hex''), decode(repeat(''22'', 32), ''hex''), 1, ''0001'', ''00000000-0000-7000-8000-0000000a2401'', ''00000000-0000-7000-8000-0000000a2303'')',
  '23505', 'uq_identity_document_data_key_id');
SELECT pg_temp.uji_tolak('A2.4 NEG document_type di luar daftar (AB-02, AB-03)', 'rs_api',
  'INSERT INTO app.identity_document (id, care_recipient_id, document_type, nik_enc, nik_bidx, nik_bidx_key_version, nik_last4, data_key_id, file_id) VALUES (''00000000-0000-7000-8000-0000000a2503'', ''00000000-0000-7000-8000-0000000a2207'', ''paspor'', decode(''01'' || repeat(''ab'', 28), ''hex''), decode(repeat(''22'', 32), ''hex''), 1, ''0001'', ''00000000-0000-7000-8000-0000000a2403'', ''00000000-0000-7000-8000-0000000a2303'')',
  '23514', 'ck_identity_document_document_type');
SELECT pg_temp.uji_tolak('A2.4 NEG berkas foto yang tidak ada', 'rs_api',
  'INSERT INTO app.identity_document (id, care_recipient_id, document_type, nik_enc, nik_bidx, nik_bidx_key_version, nik_last4, data_key_id, file_id) VALUES (''00000000-0000-7000-8000-0000000a2503'', ''00000000-0000-7000-8000-0000000a2207'', ''kk'', decode(''01'' || repeat(''ab'', 28), ''hex''), decode(repeat(''22'', 32), ''hex''), 1, ''0001'', ''00000000-0000-7000-8000-0000000a2403'', ''00000000-0000-7000-8000-00000000ffff'')',
  '23503', 'fk_identity_document_file_id');
SELECT pg_temp.uji_tolak('A2.4 NEG rs_api menimpa nik_enc dengan teks terbuka', 'rs_api',
  'UPDATE app.identity_document SET nik_enc = convert_to(''3200000000000001'', ''UTF8'') WHERE id = ''00000000-0000-7000-8000-0000000a2501''', '23514', 'ck_identity_document_nik_enc_format');
SELECT pg_temp.uji_tolak('A2.4 NEG rs_api DELETE identity_document (Master)', 'rs_api', 'DELETE FROM app.identity_document', '42501', 'permission denied for table identity_document');
-- NIK terenkripsi di luar hak: hanya rs_app dan pekerja privasi (tanpa Transit decrypt, §3.3) yang punya SELECT.
SELECT pg_temp.uji_tolak('A2.4 NEG relay audit membaca nik_enc', 'rs_audit_relay_login', 'SELECT nik_enc FROM app.identity_document', '42501', 'permission denied for table identity_document');
SELECT pg_temp.uji_tolak('A2.4 NEG rotasi kunci membaca nik_enc', 'rs_key_rotation', 'SELECT nik_enc FROM app.identity_document', '42501', 'permission denied for table identity_document');
SELECT pg_temp.uji_tolak('A2.4 NEG agen metrik (pg_monitor, tanpa USAGE skema app) membaca nik_enc', 'rs_metrics', 'SELECT nik_enc FROM app.identity_document', '42501', 'permission denied for schema app');
SELECT pg_temp.uji_tolak('A2.4 NEG agen metrik membaca registri NIK', 'rs_metrics', 'SELECT nik_bidx FROM app.account_owner_nik_registry', '42501', 'permission denied for schema app');
SELECT pg_temp.uji_tolak('A2.4 NEG pekerja privasi menimpa nik_enc (musnah lewat DEK, bukan UPDATE)', 'rs_privacy_worker_login',
  'UPDATE app.identity_document SET nik_enc = decode(''01'' || repeat(''00'', 28), ''hex'')', '42501', 'permission denied for table identity_document');
SELECT pg_temp.uji_tolak('A2.4 NEG pekerja privasi memindahkan data_key_id', 'rs_privacy_worker_login',
  'UPDATE app.identity_document SET data_key_id = ''00000000-0000-7000-8000-0000000a2404''', '42501', 'permission denied for table identity_document');
SELECT pg_temp.uji_tolak('A2.4 NEG pekerja privasi menimpa blind index dengan nilai bukan 32 byte', 'rs_privacy_worker_login',
  'UPDATE app.identity_document SET nik_bidx = ''\x00'' WHERE id = ''00000000-0000-7000-8000-0000000a2502''', '23514', 'ck_identity_document_nik_bidx');

-- identity_verification_request (T) + status_event (R) + state (P): jalur tulis ADR-008 butir 2.
SELECT pg_temp.uji_boleh('A2.5 rs_api membuat permintaan verifikasi (dokumen diajukan)', 'rs_api',
  'INSERT INTO app.identity_verification_request (id, created_by, care_recipient_id, identity_document_id, trigger_type) VALUES '
  || '(''00000000-0000-7000-8000-0000000a2601'', ''00000000-0000-7000-8000-0000000a2001'', ''00000000-0000-7000-8000-0000000a2201'', ''00000000-0000-7000-8000-0000000a2501'', ''document_submitted''), '
  || '(''00000000-0000-7000-8000-0000000a2602'', ''00000000-0000-7000-8000-0000000a2002'', ''00000000-0000-7000-8000-0000000a2203'', ''00000000-0000-7000-8000-0000000a2502'', ''document_submitted'')', 2);
SELECT pg_temp.uji_tolak('A2.5 NEG trigger_type di luar daftar', 'rs_api',
  'INSERT INTO app.identity_verification_request (id, care_recipient_id, identity_document_id, trigger_type) VALUES (''00000000-0000-7000-8000-0000000a2603'', ''00000000-0000-7000-8000-0000000a2201'', ''00000000-0000-7000-8000-0000000a2501'', ''manual'')',
  '23514', 'ck_identity_verification_request_trigger_type');
SELECT pg_temp.uji_tolak('A2.5 NEG rs_api UPDATE permintaan (T)', 'rs_api', 'UPDATE app.identity_verification_request SET trigger_type = ''profile_changed''', '42501', 'permission denied for table identity_verification_request');
SELECT pg_temp.uji_tolak('A2.5 NEG rs_worker DELETE permintaan (T)', 'rs_worker', 'DELETE FROM app.identity_verification_request', '42501', 'permission denied for table identity_verification_request');
SELECT pg_temp.uji_tolak('A2.5 NEG rs_api TRUNCATE permintaan (T)', 'rs_api', 'TRUNCATE app.identity_verification_request CASCADE', '42501', 'permission denied for table');
SELECT pg_temp.uji_tolak('A2.5 NEG pemilik UPDATE permintaan ditolak trigger', 'rs_migration',
  'UPDATE app.identity_verification_request SET trigger_type = ''profile_changed''', '42501', '^immutable_violation: UPDATE on app.identity_verification_request rejected$');
SELECT pg_temp.uji_tolak('A2.5 NEG pemilik DELETE permintaan ditolak trigger', 'rs_migration',
  'DELETE FROM app.identity_verification_request', '42501', '^immutable_violation: DELETE on app.identity_verification_request rejected$');
SELECT pg_temp.uji_tolak('A2.5 NEG pemilik TRUNCATE permintaan ditolak trigger', 'rs_migration',
  'TRUNCATE app.identity_verification_request CASCADE', '42501', '^immutable_violation: TRUNCATE on app\.identity_verification_request(_status_event|_state)? rejected$');
SELECT pg_temp.uji_boleh('A2.5 pekerja privasi membaca kolom ID, FK subjek, created_at', 'rs_privacy_worker_login',
  'SELECT id, created_at, care_recipient_id FROM app.identity_verification_request', 2);
SELECT pg_temp.uji_tolak('A2.5 NEG pekerja privasi membaca trigger_type', 'rs_privacy_worker_login',
  'SELECT trigger_type FROM app.identity_verification_request', '42501', 'permission denied for table identity_verification_request');

SELECT pg_temp.uji_boleh('A2.6 rs_api mencatat transisi pertama pending (version 1) untuk dua permintaan', 'rs_api',
  'INSERT INTO app.identity_verification_request_status_event (id, identity_verification_request_id, version, from_status, to_status, actor_type, actor_account_id, correlation_id) VALUES '
  || '(''00000000-0000-7000-8000-0000000a2701'', ''00000000-0000-7000-8000-0000000a2601'', 1, NULL, ''pending'', ''patient'', ''00000000-0000-7000-8000-0000000a2001'', ''00000000-0000-7000-8000-0000000c0001''), '
  || '(''00000000-0000-7000-8000-0000000a2702'', ''00000000-0000-7000-8000-0000000a2602'', 1, NULL, ''pending'', ''patient'', ''00000000-0000-7000-8000-0000000a2002'', ''00000000-0000-7000-8000-0000000c0002'')', 2);
SELECT pg_temp.uji_boleh('A2.7 rs_api membuat proyeksi pending', 'rs_api',
  'INSERT INTO app.identity_verification_request_state (identity_verification_request_id, status, version) VALUES (''00000000-0000-7000-8000-0000000a2601'', ''pending'', 1), (''00000000-0000-7000-8000-0000000a2602'', ''pending'', 1)', 2);
SELECT pg_temp.uji_tolak('A2.6 NEG version 1 dengan from_status', 'rs_api',
  'INSERT INTO app.identity_verification_request_status_event (id, identity_verification_request_id, version, from_status, to_status, actor_type, correlation_id) VALUES (''00000000-0000-7000-8000-0000000a2703'', ''00000000-0000-7000-8000-0000000a2601'', 1, ''pending'', ''pending'', ''system'', ''00000000-0000-7000-8000-0000000c0003'')',
  '23514', 'ck_identity_verification_request_status_event_from_status');
SELECT pg_temp.uji_tolak('A2.6 NEG penolakan tanpa reason_code (US-201 AC3)', 'rs_api',
  'INSERT INTO app.identity_verification_request_status_event (id, identity_verification_request_id, version, from_status, to_status, actor_type, actor_account_id, correlation_id) VALUES (''00000000-0000-7000-8000-0000000a2703'', ''00000000-0000-7000-8000-0000000a2601'', 2, ''pending'', ''rejected'', ''rs_admin'', ''00000000-0000-7000-8000-0000000a2003'', ''00000000-0000-7000-8000-0000000c0003'')',
  '23514', 'ck_identity_verification_request_status_event_reason_code');
SELECT pg_temp.uji_tolak('A2.6 NEG to_status di luar daftar', 'rs_api',
  'INSERT INTO app.identity_verification_request_status_event (id, identity_verification_request_id, version, from_status, to_status, actor_type, correlation_id) VALUES (''00000000-0000-7000-8000-0000000a2703'', ''00000000-0000-7000-8000-0000000a2601'', 2, ''pending'', ''verified'', ''rs_admin'', ''00000000-0000-7000-8000-0000000c0003'')',
  '23514', 'ck_identity_verification_request_status_event_to_status');
SELECT pg_temp.uji_tolak('A2.6 NEG actor_type di luar daftar', 'rs_api',
  'INSERT INTO app.identity_verification_request_status_event (id, identity_verification_request_id, version, from_status, to_status, actor_type, correlation_id) VALUES (''00000000-0000-7000-8000-0000000a2703'', ''00000000-0000-7000-8000-0000000a2601'', 2, ''pending'', ''approved'', ''verifier'', ''00000000-0000-7000-8000-0000000c0003'')',
  '23514', 'ck_identity_verification_request_status_event_actor_type');
SELECT pg_temp.uji_tolak('A2.6 NEG keterangan teks terbuka di note_enc (T-20)', 'rs_api',
  'INSERT INTO app.identity_verification_request_status_event (id, identity_verification_request_id, version, from_status, to_status, reason_code, note_enc, note_data_key_id, actor_type, actor_account_id, correlation_id) VALUES (''00000000-0000-7000-8000-0000000a2703'', ''00000000-0000-7000-8000-0000000a2601'', 2, ''pending'', ''rejected'', ''other'', convert_to(''Keterangan teks terbuka yang cukup panjang'', ''UTF8''), ''00000000-0000-7000-8000-0000000a2411'', ''rs_admin'', ''00000000-0000-7000-8000-0000000a2003'', ''00000000-0000-7000-8000-0000000c0003'')',
  '23514', 'ck_ivr_status_event_note_enc_format');
SELECT pg_temp.uji_tolak('A2.6 NEG note_enc tanpa note_data_key_id', 'rs_api',
  'INSERT INTO app.identity_verification_request_status_event (id, identity_verification_request_id, version, from_status, to_status, reason_code, note_enc, actor_type, actor_account_id, correlation_id) VALUES (''00000000-0000-7000-8000-0000000a2703'', ''00000000-0000-7000-8000-0000000a2601'', 2, ''pending'', ''rejected'', ''other'', decode(''01'' || repeat(''ab'', 40), ''hex''), ''rs_admin'', ''00000000-0000-7000-8000-0000000a2003'', ''00000000-0000-7000-8000-0000000c0003'')',
  '23514', 'ck_identity_verification_request_status_event_note');
-- Keputusan pertama menang (US-201 AC5, US-106 AC5): dua transisi version 2 untuk permintaan yang sama.
SELECT pg_temp.uji_boleh('A2.6 Verifikator menolak dengan alasan dan keterangan terenkripsi (version 2)', 'rs_api',
  'INSERT INTO app.identity_verification_request_status_event (id, identity_verification_request_id, version, from_status, to_status, reason_code, note_enc, note_data_key_id, actor_type, actor_account_id, correlation_id) VALUES (''00000000-0000-7000-8000-0000000a2703'', ''00000000-0000-7000-8000-0000000a2601'', 2, ''pending'', ''rejected'', ''other'', decode(''01'' || repeat(''ab'', 40), ''hex''), ''00000000-0000-7000-8000-0000000a2411'', ''rs_admin'', ''00000000-0000-7000-8000-0000000a2003'', ''00000000-0000-7000-8000-0000000c0003'')', 1);
SELECT pg_temp.uji_tolak('A2.6 NEG keputusan kedua (pembatalan pasien bersamaan) version 2 ditolak (AB-36)', 'rs_api',
  'INSERT INTO app.identity_verification_request_status_event (id, identity_verification_request_id, version, from_status, to_status, actor_type, actor_account_id, correlation_id) VALUES (''00000000-0000-7000-8000-0000000a2704'', ''00000000-0000-7000-8000-0000000a2601'', 2, ''pending'', ''cancelled'', ''patient'', ''00000000-0000-7000-8000-0000000a2001'', ''00000000-0000-7000-8000-0000000c0004'')',
  '23505', 'uq_ivr_status_event_identity_verification_request_id_version');
SELECT pg_temp.uji_boleh('A2.7 proyeksi maju dengan pagar versi lama (ADR-008 butir 2)', 'rs_api',
  'UPDATE app.identity_verification_request_state SET status = ''rejected'', version = 2, updated_at = now() WHERE identity_verification_request_id = ''00000000-0000-7000-8000-0000000a2601'' AND version = 1', 1);
SELECT pg_temp.uji_boleh('A2.7 pemenang kedua dengan versi usang mengenai nol baris', 'rs_api',
  'UPDATE app.identity_verification_request_state SET status = ''cancelled'', version = 2, updated_at = now() WHERE identity_verification_request_id = ''00000000-0000-7000-8000-0000000a2601'' AND version = 1', 0);
SELECT pg_temp.uji_tolak('A2.7 NEG proyeksi menunjuk status yang tidak tercatat di riwayat (KEPUTUSAN-142)', 'rs_api',
  'UPDATE app.identity_verification_request_state SET status = ''approved'', version = 2 WHERE identity_verification_request_id = ''00000000-0000-7000-8000-0000000a2602''',
  '23503', 'fk_ivr_state_request_id_version_status');
SELECT pg_temp.uji_tolak('A2.7 NEG status proyeksi di luar daftar', 'rs_api',
  'UPDATE app.identity_verification_request_state SET status = ''x'' WHERE identity_verification_request_id = ''00000000-0000-7000-8000-0000000a2602''',
  '23514', 'ck_identity_verification_request_state_status');
SELECT pg_temp.uji_tolak('A2.7 NEG rs_api DELETE proyeksi', 'rs_api', 'DELETE FROM app.identity_verification_request_state', '42501', 'permission denied for table identity_verification_request_state');
SELECT pg_temp.uji_tolak('A2.7 NEG pekerja privasi membaca proyeksi', 'rs_privacy_worker_login', 'SELECT * FROM app.identity_verification_request_state', '42501', 'permission denied for table identity_verification_request_state');
SELECT pg_temp.uji_tolak('A2.6 NEG rs_api UPDATE riwayat status (R)', 'rs_api',
  'UPDATE app.identity_verification_request_status_event SET to_status = ''approved''', '42501', 'permission denied for table identity_verification_request_status_event');
SELECT pg_temp.uji_tolak('A2.6 NEG rs_worker DELETE riwayat status (R)', 'rs_worker',
  'DELETE FROM app.identity_verification_request_status_event', '42501', 'permission denied for table identity_verification_request_status_event');
SELECT pg_temp.uji_tolak('A2.6 NEG rs_api INSERT ON CONFLICT DO UPDATE riwayat (tanpa hak UPDATE)', 'rs_api',
  'INSERT INTO app.identity_verification_request_status_event (id, identity_verification_request_id, version, from_status, to_status, actor_type, correlation_id) VALUES (''00000000-0000-7000-8000-0000000a2705'', ''00000000-0000-7000-8000-0000000a2601'', 2, ''pending'', ''approved'', ''rs_admin'', ''00000000-0000-7000-8000-0000000c0005'') ON CONFLICT (identity_verification_request_id, version) DO UPDATE SET to_status = ''approved''',
  '42501', 'permission denied for table identity_verification_request_status_event');
SELECT pg_temp.uji_tolak('A2.6 NEG pemilik UPDATE riwayat ditolak trigger', 'rs_migration',
  'UPDATE app.identity_verification_request_status_event SET to_status = ''approved'' WHERE false', '42501', '^immutable_violation: UPDATE on app.identity_verification_request_status_event rejected$');
SELECT pg_temp.uji_tolak('A2.6 NEG pemilik DELETE riwayat ditolak trigger', 'rs_migration',
  'DELETE FROM app.identity_verification_request_status_event', '42501', '^immutable_violation: DELETE on app.identity_verification_request_status_event rejected$');
SELECT pg_temp.uji_tolak('A2.6 NEG pemilik TRUNCATE riwayat ditolak trigger', 'rs_migration',
  'TRUNCATE app.identity_verification_request_status_event CASCADE', '42501', '^immutable_violation: TRUNCATE on app\.identity_verification_request_(status_event|state) rejected$');
SELECT pg_temp.uji_boleh('A2.6 pekerja privasi membaca id dan note_data_key_id riwayat', 'rs_privacy_worker_login',
  'SELECT id, note_data_key_id FROM app.identity_verification_request_status_event', 3);
SELECT pg_temp.uji_tolak('A2.6 NEG pekerja privasi membaca note_enc riwayat', 'rs_privacy_worker_login',
  'SELECT note_enc FROM app.identity_verification_request_status_event', '42501', 'permission denied for table identity_verification_request_status_event');

-- TD-08 (F1-T59): kaitan lintas penerima layanan ditolak FK komposit, walaupun baris rujukannya ada.
SELECT pg_temp.uji_tolak('A2.5 NEG permintaan untuk dokumen milik penerima layanan lain (TD-08)', 'rs_api',
  'INSERT INTO app.identity_verification_request (id, care_recipient_id, identity_document_id, trigger_type) VALUES (''00000000-0000-7000-8000-0000000a2603'', ''00000000-0000-7000-8000-0000000a2201'', ''00000000-0000-7000-8000-0000000a2502'', ''document_submitted'')',
  '23503', 'fk_ivr_identity_document_id_care_recipient_id');
SELECT pg_temp.uji_tolak('A2.2 NEG penunjuk status terkini ke permintaan penerima layanan lain (TD-08)', 'rs_api',
  'UPDATE app.care_recipient SET latest_verification_request_id = ''00000000-0000-7000-8000-0000000a2602'' WHERE id = ''00000000-0000-7000-8000-0000000a2201''',
  '23503', 'fk_care_recipient_latest_verification_request_id_id');
SELECT pg_temp.uji_boleh('A2.2 penunjuk status terkini ke permintaannya sendiri (TD-08 positif)', 'rs_api',
  'UPDATE app.care_recipient SET latest_verification_request_id = ''00000000-0000-7000-8000-0000000a2601'', updated_at = now() WHERE id = ''00000000-0000-7000-8000-0000000a2201''', 1);
SELECT pg_temp.uji_tolak('A2.8 NEG registri berdasar permintaan penerima layanan lain (TD-08)', 'rs_api',
  'INSERT INTO app.account_owner_nik_registry (id, nik_bidx, nik_bidx_key_version, care_recipient_id, identity_verification_request_id) VALUES (''00000000-0000-7000-8000-0000000a2804'', decode(repeat(''55'', 32), ''hex''), 1, ''00000000-0000-7000-8000-0000000a2201'', ''00000000-0000-7000-8000-0000000a2602'')',
  '23503', 'fk_owner_nik_registry_request_id_care_recipient_id');

-- account_owner_nik_registry (M, A2.8): UP-40 lintas akun (US-201 AC7).
SELECT pg_temp.uji_boleh('A2.8 persetujuan pertama mencatat NIK pemilik akun', 'rs_api',
  'INSERT INTO app.account_owner_nik_registry (id, created_by, nik_bidx, nik_bidx_key_version, care_recipient_id, identity_verification_request_id) VALUES (''00000000-0000-7000-8000-0000000a2801'', ''00000000-0000-7000-8000-0000000a2003'', decode(repeat(''11'', 32), ''hex''), 1, ''00000000-0000-7000-8000-0000000a2201'', ''00000000-0000-7000-8000-0000000a2601'')', 1);
SELECT pg_temp.uji_tolak('A2.8 NEG NIK yang sama disetujui sebagai pemilik akun lain (US-201 AC7)', 'rs_api',
  'INSERT INTO app.account_owner_nik_registry (id, nik_bidx, nik_bidx_key_version, care_recipient_id, identity_verification_request_id) VALUES (''00000000-0000-7000-8000-0000000a2802'', decode(repeat(''11'', 32), ''hex''), 1, ''00000000-0000-7000-8000-0000000a2203'', ''00000000-0000-7000-8000-0000000a2602'')',
  '23505', 'uqp_account_owner_nik_registry_nik_bidx');
SELECT pg_temp.uji_tolak('A2.8 NEG dua NIK aktif untuk satu pemilik akun', 'rs_api',
  'INSERT INTO app.account_owner_nik_registry (id, nik_bidx, nik_bidx_key_version, care_recipient_id, identity_verification_request_id) VALUES (''00000000-0000-7000-8000-0000000a2802'', decode(repeat(''44'', 32), ''hex''), 1, ''00000000-0000-7000-8000-0000000a2201'', ''00000000-0000-7000-8000-0000000a2601'')',
  '23505', 'uqp_account_owner_nik_registry_care_recipient_id');
SELECT pg_temp.uji_tolak('A2.8 NEG blind index registri bukan 32 byte', 'rs_api',
  'INSERT INTO app.account_owner_nik_registry (id, nik_bidx, nik_bidx_key_version, care_recipient_id, identity_verification_request_id) VALUES (''00000000-0000-7000-8000-0000000a2802'', decode(repeat(''44'', 16), ''hex''), 1, ''00000000-0000-7000-8000-0000000a2203'', ''00000000-0000-7000-8000-0000000a2602'')',
  '23514', 'ck_account_owner_nik_registry_nik_bidx');
SELECT pg_temp.uji_tolak('A2.8 NEG registri tanpa permintaan dasar yang ada', 'rs_api',
  'INSERT INTO app.account_owner_nik_registry (id, nik_bidx, nik_bidx_key_version, care_recipient_id, identity_verification_request_id) VALUES (''00000000-0000-7000-8000-0000000a2802'', decode(repeat(''44'', 32), ''hex''), 1, ''00000000-0000-7000-8000-0000000a2203'', ''00000000-0000-7000-8000-00000000ffff'')',
  '23503', 'fk_owner_nik_registry_identity_verification_request_id');
SELECT pg_temp.uji_tolak('A2.8 NEG rs_api DELETE registri (Master)', 'rs_api', 'DELETE FROM app.account_owner_nik_registry', '42501', 'permission denied for table account_owner_nik_registry');
SELECT pg_temp.uji_tolak('A2.8 NEG pekerja privasi memindahkan registri ke penerima lain', 'rs_privacy_worker_login',
  'UPDATE app.account_owner_nik_registry SET care_recipient_id = ''00000000-0000-7000-8000-0000000a2203''', '42501', 'permission denied for table account_owner_nik_registry');
SELECT pg_temp.uji_boleh('A2.8 pekerja privasi menganonimkan registri: soft delete + timpa blind index', 'rs_privacy_worker_login',
  'UPDATE app.account_owner_nik_registry SET nik_bidx = decode(repeat(''ee'', 32), ''hex''), deleted_at = now(), delete_reason = ''anonymized'', anonymized_at = now(), updated_at = now() WHERE id = ''00000000-0000-7000-8000-0000000a2801''', 1);
SELECT pg_temp.uji_boleh('A2.8 NIK yang registrinya sudah dihapus dapat disetujui untuk pemilik akun lain', 'rs_api',
  'INSERT INTO app.account_owner_nik_registry (id, nik_bidx, nik_bidx_key_version, care_recipient_id, identity_verification_request_id) VALUES (''00000000-0000-7000-8000-0000000a2803'', decode(repeat(''11'', 32), ''hex''), 1, ''00000000-0000-7000-8000-0000000a2203'', ''00000000-0000-7000-8000-0000000a2602'')', 1);

-- Pekerja privasi atas care_recipient dan patient (T-07): hanya kolom "Anonimkan" + penanda.
SELECT pg_temp.uji_boleh('A2.2 pekerja privasi menganonimkan penerima layanan', 'rs_privacy_worker_login',
  'UPDATE app.care_recipient SET full_name = ''Anonim'', birth_date = ''1900-01-01'', sex = ''male'', nik_bidx = NULL, nik_last4 = NULL, anonymized_at = now(), updated_at = now(), updated_by = NULL WHERE id = ''00000000-0000-7000-8000-0000000a2203''', 1);
SELECT pg_temp.uji_tolak('A2.2 NEG pekerja privasi mengubah patient_id', 'rs_privacy_worker_login',
  'UPDATE app.care_recipient SET patient_id = ''00000000-0000-7000-8000-0000000a2101''', '42501', 'permission denied for table care_recipient');
SELECT pg_temp.uji_tolak('A2.2 NEG pekerja privasi mengubah latest_verification_request_id', 'rs_privacy_worker_login',
  'UPDATE app.care_recipient SET latest_verification_request_id = NULL', '42501', 'permission denied for table care_recipient');
SELECT pg_temp.uji_boleh('A2.1 pekerja privasi menganonimkan telepon pasien', 'rs_privacy_worker_login',
  'UPDATE app.patient SET phone = NULL, anonymized_at = now(), updated_at = now() WHERE id = ''00000000-0000-7000-8000-0000000a2101''', 1);
SELECT pg_temp.uji_tolak('A2 NEG pekerja privasi INSERT dokumen identitas', 'rs_privacy_worker_login',
  'INSERT INTO app.identity_document (id, care_recipient_id, document_type, nik_enc, nik_bidx, nik_bidx_key_version, nik_last4, data_key_id, file_id) VALUES (''00000000-0000-7000-8000-0000000a2504'', ''00000000-0000-7000-8000-0000000a2201'', ''ktp'', decode(''01'' || repeat(''ab'', 28), ''hex''), decode(repeat(''55'', 32), ''hex''), 1, ''0001'', ''00000000-0000-7000-8000-0000000a2404'', ''00000000-0000-7000-8000-0000000a2304'')',
  '42501', 'permission denied for table identity_document');
SELECT pg_temp.uji_tolak('A2 NEG relay audit membaca care_recipient', 'rs_audit_relay_login', 'SELECT full_name FROM app.care_recipient', '42501', 'permission denied for table care_recipient');
SELECT pg_temp.uji_tolak('A2 NEG rotasi kunci membaca address', 'rs_key_rotation', 'SELECT address_line FROM app.address', '42501', 'permission denied for table address');
SELECT pg_temp.uji_tolak('A2 NEG relay audit membaca riwayat status', 'rs_audit_relay_login', 'SELECT * FROM app.identity_verification_request_status_event', '42501', 'permission denied for table identity_verification_request_status_event');
SELECT pg_temp.uji_tolak('A2 NEG rs_api TRUNCATE registri NIK', 'rs_api', 'TRUNCATE app.account_owner_nik_registry', '42501', 'permission denied for table account_owner_nik_registry');
ROLLBACK;

DO $$
BEGIN
  IF EXISTS (SELECT 1 FROM app.patient) OR EXISTS (SELECT 1 FROM app.care_recipient) OR EXISTS (SELECT 1 FROM app.address)
     OR EXISTS (SELECT 1 FROM app.identity_document) OR EXISTS (SELECT 1 FROM app.identity_verification_request)
     OR EXISTS (SELECT 1 FROM app.identity_verification_request_status_event)
     OR EXISTS (SELECT 1 FROM app.identity_verification_request_state)
     OR EXISTS (SELECT 1 FROM app.account_owner_nik_registry)
     -- reason_code berisi data acuan migrasi data F1-T62 (tujuh alasan identity_rejection); data uji = ID lain.
     OR EXISTS (SELECT 1 FROM app.reason_code WHERE id::text LIKE '00000000-0000-7000-8000-%')
     OR EXISTS (SELECT 1 FROM app.data_key) OR EXISTS (SELECT 1 FROM app.stored_file) OR EXISTS (SELECT 1 FROM app.account) THEN
    RAISE EXCEPTION 'GAGAL pembersihan tabel-app-a2: data uji tersisa';
  END IF;
  RAISE NOTICE 'LULUS pembersihan tabel-app-a2: tidak ada data uji tersisa (ROLLBACK)';
END $$;
