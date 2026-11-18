# Skema Fisik PostgreSQL: Rumah Sehat

Tanggal: 2026-11-18
Versi: 0.1 (draf Tahap 5, menunggu review `dba-reviewer` + `security-architect` dan fact-check)
Penyusun: Claude (persona DBA senior + Principal Engineer)
Tingkat ketelitian: **T1 Kritis** (model data, migrasi, hak role, RLS, audit, enkripsi)
Target: PostgreSQL 18.7, dijalankan sendiri `[SRC-118]` `[KEPUTUSAN-77]`

Dokumen ini adalah **lapisan fisik yang melengkapi** `docs/04-data-dictionary.md` (selanjutnya "DD"), yang sudah dikunci di gate Tahap 4 `[USER] T4-G5`. Tipe kolom, CHECK isi, kelas tabel, dan hak role per kelas **tidak disalin ulang**; sumber kebenarannya tetap DD. Di sini hanya ada keputusan fisik yang di DD ditandai "ditetapkan Tahap 5" atau belum diatur. Kontradiksi dan celah atas DD tidak diperbaiki diam-diam, tetapi dicatat di §14.

---

## 0. Cara membaca

| Penanda | Arti |
| --- | --- |
| `[DOK: DD §x]` | `docs/04-data-dictionary.md` bagian x (dokumen lulus gate) |
| `[DOK: ADR-xxx]` | Keputusan arsitektur di `docs/adr/` |
| `[USER] T5-Qx` | Jawaban user Tahap 5 (`docs/05-catatan-stack.md`) |
| `[SRC-xx]` | Fakta eksternal terverifikasi di `docs/sources.md`. Dokumen ini tidak menambah SRC baru |
| `[KEPUTUSAN-xx]` | Nomor 74–89 sudah terdaftar; nomor 90–113 baru dari dokumen ini (§15), dicatat orkestrator di `docs/decision-log.md` |
| **(V8)** | Perilaku atau sintaks PostgreSQL/pustaka yang **belum terdaftar sebagai SRC**. Wajib diverifikasi ke dokumentasi resmi versi terpasang dan dibuktikan test migrasi di Tahap 8 Fase 1 sebelum diandalkan. Bila verifikasi menunjukkan perilaku berbeda, rancangan terkait kembali ke DBA, tidak diakali di kode |
| `[TIDAK DIKETAHUI]` | Belum ada informasi; pemilik disebut di sampingnya |

Semua SQL di dokumen ini adalah **rancangan**, bukan migrasi siap pakai. Migrasi sungguhan ditulis di Tahap 8 dan wajib lolos `scripts/verify.sh` langkah 2 `[KEPUTUSAN-82]`.

Pemetaan butir instruksi ke bagian: (1) §1, (2) §2, (3) §4, (4) §3, (5) §5, (6) §6, (7) §7, (8) §8, (9)–(10) §9, (11) §10, (12) §11, (13) §15. Test katalog wajib dirangkum di §12, butir DD yang masih terbuka di §13.

---

## 1. Penamaan objek database

Dasar: snake_case untuk semua nama, termasuk index dan constraint `[DOK: ADR-001]`; nama constraint ditetapkan Tahap 5 `[DOK: DD §0.2]`.

### 1.1 Pola

| Objek | Pola | Contoh | Sumber |
| --- | --- | --- | --- |
| Primary key | `pk_<tabel>` | `pk_patient_order` | `[KEPUTUSAN-90]` |
| Foreign key | `fk_<tabel>_<kolom1>[_<kolom2>...]` (semua kolom lokal, urut sesuai definisi) | `fk_payment_capture_payment_receipt_id_patient_order_id` | `[KEPUTUSAN-90]` |
| Unique constraint penuh | `uq_<tabel>_<kolom...>` | `uq_order_status_event_patient_order_id_version` | `[KEPUTUSAN-90]` |
| Partial unique index | `uqp_<tabel>_<kolom...>` | `uqp_account_idp_realm_idp_subject` | `[KEPUTUSAN-90]` |
| Index biasa / partial | `ix_<tabel>_<kolom...>[_<kualifikasi>]` | `ix_job_run_at_queued`, `ix_partner_service_name_trgm` | `[KEPUTUSAN-90]` |
| CHECK | `ck_<tabel>_<kolom atau makna>` | `ck_account_status`, `ck_order_price_lock_total`, `ck_rs_admin_role_soft_delete` | `[KEPUTUSAN-90]` |
| Exclusion constraint (bila dipakai, §13) | `ex_<tabel>_<kolom...>` | `ex_consultation_slot_health_worker_id_range` | `[KEPUTUSAN-90]` |
| Trigger | `tg_<tabel>_<tujuan>` | `tg_journal_line_reject_ud`, `tg_journal_line_reject_truncate`, `tg_data_key_one_way` | `[KEPUTUSAN-90]` |
| Fungsi | `fn_<tujuan>` di skema `util` (pustaka bersama) atau skema bisnis (fungsi khusus) | `util.fn_reject_modification`, `audit.fn_chain_head` | `[KEPUTUSAN-90]` |
| Kebijakan RLS | `pol_<tabel>_<role atau tujuan>` | `pol_rme_visit_tenant`, `pol_rme_job_worker_queue` | `[KEPUTUSAN-90]` |
| Partisi bulanan | `<induk>_yYYYYmMM` | `audit_event_y2027m03` | Mengikuti contoh `measurement_y2006m02` di halaman partisi `[SRC-39]`; `[KEPUTUSAN-90]` |
| Database, skema, role | §2 dan §3 | | |

Aturan tambahan:
- Nama constraint dan index unik per skema, bukan hanya per tabel. Karena nama memuat nama tabel, bentrokan hanya terjadi bila dua constraint di satu tabel punya kolom sama; yang kedua diberi akhiran makna (misalnya `_wadah`).
- Sasaran FK selalu dideklarasikan sebagai `CONSTRAINT uq_... UNIQUE (...)`, bukan sebagai index unik, sesuai catatan DD A8.13 (persyaratan PostgreSQL atas sasaran FK **(V8)**).
- Tanpa `serial`/`identity`/sequence untuk kolom bisnis. PK UUIDv7 dibuat aplikasi `[DOK: ADR-002]`; `audit_event.seq` dihitung relay (§6.4) karena sequence dapat melompat saat rollback **(V8)**, padahal `seq` wajib tanpa celah `[DOK: DD B1]`.

### 1.2 Batas panjang

Batas panjang identifier PostgreSQL adalah 63 byte, dan nama yang lebih panjang dipotong tanpa galat **(V8)**. Supaya pemotongan tidak pernah terjadi diam-diam `[KEPUTUSAN-90]`:
- **Batas proyek 60 byte** untuk semua nama yang kita tulis.
- Bila nama lengkap melebihi 60 byte, bagian `<tabel>` diganti **alias tabel** (maksimal 20 karakter) yang dicatat di komentar tabel (§4.1), dan bila masih panjang bagian kolom diganti makna singkat. Contoh: `fk_ivr_status_event_identity_verification_request_id` (54 byte) untuk `identity_verification_request_status_event`.
- Test katalog (§12 T-01) gagal bila ada nama ≥ 61 byte di skema milik proyek. Nama yang tepat 63 byte adalah tanda pemotongan, sehingga ikut tertangkap.
- Objek turunan otomatis (index dan constraint di partisi audit) dikecualikan dari pola, karena namanya dibuat PostgreSQL.

---

## 2. Organisasi instans, database, skema, dan ekstensi

Tiga instans PostgreSQL production sesuai ADR-019 butir 3 dan topologi VPS `[DOK: ADR-019]` `[KEPUTUSAN-78]`: (a) `db-app`, (b) `db-audit`, (c) `db-rme`. Staging menjalankan tiga klaster terpisah di `stg-db` `[KEPUTUSAN-78]`; lokal dan test memakai testcontainers PostgreSQL 18 `[KEPUTUSAN-74]`. Nama database dan skema sama di semua lingkungan.

### 2.1 Database dan skema

| Instans | Database | Skema | Pemilik skema | Isi | Sumber |
| --- | --- | --- | --- | --- | --- |
| (a) `db-app` | `rumah_sehat` | `app` | `rs_migration` | Semua tabel DD bagian A | `[DOK: DD A]`, `[KEPUTUSAN-91]` |
| | | `util` | `rs_migration` | Fungsi bersama (trigger penolak, penjaga Log, satu arah `data_key`, kunci berlaku-mulai, penjaga migrasi) | `[KEPUTUSAN-91]` |
| | | `ext` | superuser bootstrap | Ekstensi `pg_trgm` | `[SRC-42]`, `[KEPUTUSAN-92]` |
| | `keycloak` | dikelola Keycloak | role `keycloak` | Data Keycloak 26.8.2; skemanya dibuat dan dimigrasi Keycloak sendiri (opsi skema Keycloak **(V8)**) | `[DOK: DD §0.10]`, `[SRC-121]` |
| (b) `db-audit` | `rumah_sehat_audit` | `audit`, `util` | `rs_audit_migration` | DD bagian B | `[DOK: DD B]`, `[DOK: ADR-009]` |
| | | `audit_detached` | `rs_audit_migration` | Partisi yang sudah di-*detach* dan menunggu penghapusan salinan daring (§6.5) | `[KEPUTUSAN-101]` |
| | `rme_audit` | `audit`, `util`, `audit_detached` | `rme_audit_migration` | DD bagian B + RLS tenant (§5.6) | `[DOK: DD §0.6 B]`, `[KEPUTUSAN-101]` |
| (c) `db-rme` | `rme` | `app`, `util`, `ext` | `rme_migration` | DD bagian C | `[DOK: DD C]` |

Aturan organisasi `[KEPUTUSAN-91]`:
1. **Satu skema bisnis per database** (`app` atau `audit`). Batas modul Rumah Sehat ditegakkan di kode dan test arsitektur, bukan skema per modul `[KEPUTUSAN-74]`. Skema per modul ditolak karena FK lintas modul banyak (misalnya `patient_order` dirujuk pembayaran, konsultasi, refund) dan `search_path` menjadi rumit.
2. **Skema `public` dihapus** di setiap database proyek saat bootstrap, sehingga tidak ada tempat objek liar. Test T-02 memastikan tidak ada objek di luar skema yang terdaftar.
3. **Database dimiliki superuser bootstrap**, bukan role migrasi. Role migrasi memiliki skema dan objek, tetapi tidak dapat `DROP DATABASE`.
4. **Hak `CONNECT` dan `TEMP` ke database dicabut dari `PUBLIC`** **(V8: hak bawaan PUBLIC atas database)**, lalu `CONNECT` diberikan per role grup. Akibatnya role `keycloak` tidak dapat masuk ke `rumah_sehat`, dan sebaliknya. Database `postgres` juga dicabut `CONNECT`-nya dari `PUBLIC`. Tanpa `TEMP`, role aplikasi tidak dapat membuat objek di `pg_temp` yang dapat membayangi nama di fungsi.
5. **`search_path`** dipasang per database dan per role login: `app, ext` untuk database aplikasi, `audit` untuk database audit (`ALTER ROLE ... IN DATABASE ... SET search_path`, **(V8)**). Fungsi `SECURITY DEFINER` selalu memasang `SET search_path = pg_catalog, <skema>` sendiri dan merujuk tabel dengan nama berkualifikasi skema (praktik dokumentasi `CREATE FUNCTION` **(V8)**). Kode SQL di repository Rust memakai nama tabel tanpa skema dan bergantung pada `search_path` ini; pustaka bersama dan migrasi memakai nama berkualifikasi.
6. **Hak `EXECUTE` fungsi dicabut dari `PUBLIC`** lewat `ALTER DEFAULT PRIVILEGES FOR ROLE <pemilik> REVOKE EXECUTE ON FUNCTIONS FROM PUBLIC` dan `REVOKE` eksplisit per fungsi **(V8: bawaan EXECUTE untuk PUBLIC)**. Hak tabel **tidak** diberikan lewat default privileges, karena hak berbeda per kelas (§4).

### 2.2 Ekstensi

| Database | Ekstensi | Alasan | Sumber |
| --- | --- | --- | --- |
| `rumah_sehat` | `pg_trgm` di skema `ext` | Index trigram nama layanan, produk, rekanan, tenaga kesehatan | `[SRC-42]`, `[DOK: DD §0.2]` |
| `rme` | `pg_trgm` di skema `ext` | Pencarian tampilan terminologi dan nama | `[SRC-42]`, `[DOK: DD §0.2]` |
| `rumah_sehat_audit`, `rme_audit`, `keycloak` | — | | |

Ketentuan `[KEPUTUSAN-92]`:
- Ekstensi dibuat di **bootstrap** oleh superuser (§3.5), bukan oleh migrasi, karena role migrasi bukan superuser. Apakah `pg_trgm` dapat dibuat non-superuser **(V8)**; bila dapat, keputusan ini tetap berlaku demi pemisahan hak.
- Operator class trigram dirujuk berkualifikasi: `USING gin (name ext.gin_trgm_ops)` **(V8: sintaks opclass berkualifikasi skema)**.
- **Tidak memakai** `pgcrypto` (enkripsi di aplikasi, ADR-013 alternatif ditolak) dan tidak memakai fungsi `uuidv7()` sebagai nilai bawaan kolom (§4.2). PostgreSQL 18 menyediakan `uuidv7()` `[SRC-32]`, tetapi ID tetap dibuat aplikasi `[DOK: ADR-002]` `[KEPUTUSAN-74]`.
- `pg_stat_statements` untuk analisis performa: dievaluasi Tahap 8 (memerlukan pengaturan server **(V8)**; teks query hanya berisi placeholder parameter, sehingga tidak memuat nilai data, perilaku ini juga **(V8)**).
- Ekstensi `btree_gist` hanya bila pencegahan tumpang tindih slot memakai exclusion constraint (§13 butir b); belum diputuskan.

---

## 3. Role database dan kredensial

### 3.1 Prinsip

1. **Role grup `NOLOGIN` memegang hak; role login per proses menjadi anggotanya.** Hak per kelas diberikan ke role grup sesuai DD §0.6. Role login per proses memberi atribusi di log dan `pg_stat_activity`, batas koneksi, dan batas waktu per proses tanpa memecah hak `[KEPUTUSAN-97]`.
2. **Nama fisik berawalan sistem.** Role berlaku untuk seluruh klaster, bukan per database **(V8)**, sementara database audit Rumah Sehat dan audit RME berbagi instans (b) dengan role terpisah `[DOK: ADR-009 butir 4]`. Karena itu nama logis DD dipetakan ke nama fisik berawalan (§3.2). Awalan juga dipakai di (a) dan (c) supaya lingkungan lokal yang menjalankan semua database di satu klaster tetap tanpa bentrokan `[KEPUTUSAN-97]`.
3. **Atribut setiap role proyek**: `NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS`; role login ditambah `LOGIN INHERIT CONNECTION LIMIT <n>`; role grup `NOLOGIN`. Hanya superuser bootstrap yang melewati RLS, karena superuser dan role `BYPASSRLS` selalu melewati RLS `[SRC-34]`. Superuser production hanya lewat akses darurat `[DOK: ADR-021]` `[KEPUTUSAN-79]`.
4. **Role aplikasi bukan pemilik tabel**, karena pemilik tabel normalnya melewati RLS `[SRC-34]` `[DOK: ADR-012 butir 6]`.
5. Keanggotaan diberikan dengan pewarisan hak (`INHERIT`). Opsi per keanggotaan (`INHERIT`, `SET`, `ADMIN`) di PostgreSQL 16 ke atas **(V8)**.

### 3.2 Daftar role per instans

**(a) `db-app`, database `rumah_sehat` dan `keycloak`**

| Nama logis DD §0.6 | Role grup (NOLOGIN) | Role login (proses) | Kredensial | Sumber |
| --- | --- | --- | --- | --- |
| `rs_app` | `rs_app` | `rs_api` (api), `rs_realtime` (realtime), `rs_worker` (worker) | Password statis di OpenBao KV `[KEPUTUSAN-79]` | `[DOK: DD §0.6 A]`, `[KEPUTUSAN-74]` |
| `audit_relay` | `rs_audit_relay` | `rs_audit_relay_login` (audit-relay) | KV | `[DOK: ADR-009 butir 3]` |
| `privacy_worker` | `rs_privacy_worker` | `rs_privacy_worker_login` (pekerja privasi; proses pemegangnya §14 F-P10) | KV | `[DOK: ADR-013 §5]` |
| `key_rotation` | `rs_key_rotation` | Tanpa role login tetap; anggota dinamis dari OpenBao saat prosedur bungkus ulang (§3.4) | Dinamis | `[DOK: DD A19.7]`, `[KEPUTUSAN-96]` |
| `rs_migration` | `rs_migration` (pemilik skema `app`, `util`) | Tanpa role login tetap; anggota dinamis dari OpenBao saat migrasi | Dinamis `[SRC-224]` | `[DOK: ADR-021 butir 4]` |
| — | `rs_ops_monitor` (anggota role pemantauan bawaan `pg_monitor` **(V8)**) | `rs_metrics` (agen metrik); manusia lewat kredensial dinamis | KV / dinamis | `[KEPUTUSAN-80]` |
| — | — | `keycloak` (pemilik database `keycloak`) | KV | `[DOK: DD §0.10]` |

**(b) `db-audit`, database `rumah_sehat_audit` dan `rme_audit`**

| Nama logis DD §0.6 B | Rumah Sehat (grup / login) | RME (grup / login) | Proses pemegang |
| --- | --- | --- | --- |
| `audit_writer` | `rs_audit_writer` / `rs_audit_writer_login` | `rme_audit_writer` / `rme_audit_writer_login` | audit-relay sistem masing-masing |
| `audit_reader` | `rs_audit_reader` / `rs_audit_reader_login` | `rme_audit_reader` / `rme_audit_reader_login` | api backoffice (RS); RME tidak memakainya untuk Admin Fasyankes `[DOK: DD §0.6 B]` |
| `audit_verifier` | `rs_audit_verifier` / `rs_audit_verifier_login` | `rme_audit_verifier` / `rme_audit_verifier_login` | Job verifikasi harian, dijalankan di deployment audit-relay (§14 F-P09) `[KEPUTUSAN-141]` |
| `audit_tenant_reader` | — | `rme_audit_tenant_reader` / `rme_audit_tenant_reader_login` | rme api |
| `audit_migration` | `rs_audit_migration` (pemilik) | `rme_audit_migration` (pemilik) | Dinamis, pipeline migrasi dan prosedur arsip |

**(c) `db-rme`, database `rme`**

| Nama logis DD §0.6 C | Role grup | Role login | Sumber |
| --- | --- | --- | --- |
| `rme_app` | `rme_app` | `rme_api` | `[DOK: DD §0.6 C]` |
| `rme_auth` | `rme_auth` | `rme_api_auth` (pool kecil di proses api, dipakai sebelum konteks tenant dipasang) | `[KEPUTUSAN-55]` |
| `rme_worker` | `rme_worker` | `rme_worker_login` | `[KEPUTUSAN-55]` |
| `rme_operator` | `rme_operator` | `rme_api_operator` (jalur operator di proses api) | `[DOK: ADR-012 butir 6]` |
| `audit_relay` | `rme_audit_relay` | `rme_audit_relay_login` | `[DOK: ADR-009 butir 3]` |
| `privacy_worker` | `rme_privacy_worker` | `rme_privacy_worker_login` | `[DOK: ADR-013 §5]` |
| `key_rotation` | `rme_key_rotation` | dinamis | `[KEPUTUSAN-96]` |
| `rme_migration` | `rme_migration` (pemilik) | dinamis | `[DOK: ADR-021]` |
| — | `rme_ops_monitor` | `rme_metrics` | `[KEPUTUSAN-80]` |

Test T-05 (§12) memastikan: tidak ada role proyek dengan `BYPASSRLS` atau `SUPERUSER`; role grup tidak dapat login; tidak ada role login yang menjadi anggota dua grup yang saling meniadakan (`*_privacy_worker` dan `*_key_rotation`, atau grup aplikasi dan grup migrasi).

### 3.3 Proses dan kredensial

| Proses `[KEPUTUSAN-74]` | Kredensial database yang dipegang |
| --- | --- |
| `rs-backend api` | `rs_api` (rumah_sehat), `rs_audit_reader_login` (rumah_sehat_audit) |
| `rs-backend realtime` | `rs_realtime` |
| `rs-backend worker` | `rs_worker` |
| `rs-backend audit-relay` | `rs_audit_relay_login`, `rs_audit_writer_login`, `rs_audit_verifier_login`. Worker bisnis tidak memegangnya (threat model M-23) `[DOK: ADR-009 butir 3]` |
| `rme-backend api` | `rme_api`, `rme_api_auth`, `rme_api_operator`, `rme_audit_tenant_reader_login` |
| `rme-backend worker` | `rme_worker_login` |
| relay audit RME (§14 F-P09) | `rme_audit_relay_login`, `rme_audit_writer_login`, `rme_audit_verifier_login` |

### 3.4 Kredensial dinamis OpenBao untuk pemilik skema

OpenBao database secrets engine membangkitkan kredensial PostgreSQL berumur pendek (`default_ttl`/`max_ttl`, `VALID UNTIL`) `[SRC-224]`, dipakai untuk pemilik skema saat migrasi dan akses administratif `[KEPUTUSAN-79]`.

Masalah yang harus dicegah: objek yang dibuat oleh role login dinamis akan **dimiliki role dinamis itu**, sehingga role tidak dapat dihapus saat kedaluwarsa dan kepemilikan tersebar. Karena itu `[KEPUTUSAN-97]`:

```sql
-- creation_statements role OpenBao "rs-migration" (sintaks templat dari SRC-224;
-- klausa IN ROLE dan ALTER ROLE ... SET role adalah (V8))
CREATE ROLE "{{name}}" WITH LOGIN PASSWORD '{{password}}' VALID UNTIL '{{expiration}}'
  NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS
  IN ROLE rs_migration;
ALTER ROLE "{{name}}" SET role = 'rs_migration';   -- sesi langsung berjalan sebagai rs_migration

-- revocation_statements
REASSIGN OWNED BY "{{name}}" TO rs_migration;
DROP OWNED BY "{{name}}";
DROP ROLE IF EXISTS "{{name}}";
```

| Role OpenBao | Grup sasaran | `default_ttl` / `max_ttl` (nilai awal) | Pemakai |
| --- | --- | --- | --- |
| `rs-migration`, `rme-migration`, `rs-audit-migration`, `rme-audit-migration` | Pemilik skema | 30 menit / 2 jam | Runner staging (token OIDC GitHub dibatasi `sub` repo + ref `main`) dan skrip deploy production lewat bastion `[KEPUTUSAN-81]` `[USER] T5-Q8` |
| `rs-key-rotation`, `rme-key-rotation` | `*_key_rotation` | 1 jam / 4 jam | Prosedur bungkus ulang DEK (§7.3), dua orang |
| `rs-ops-monitor`, `rme-ops-monitor` | `*_ops_monitor` | 1 jam / 8 jam | Diagnosis manusia (MFA OpenBao) |
| Akses darurat (superuser) | — | ditetapkan sebelum go-live | `[KEPUTUSAN-79]`, PT-38 |

Pengaman berlapis:
- **Penjaga di setiap berkas migrasi** memeriksa `current_user` = role pemilik (§11.2). Bila `ALTER ROLE ... SET role` ternyata tidak berlaku, migrasi gagal di baris pertama, tidak membuat objek dengan pemilik salah.
- Test T-04: semua objek di skema proyek, termasuk tabel catatan migrasi sqlx, dimiliki role pemilik **(V8: nama dan lokasi tabel catatan migrasi sqlx-cli 0.9.1)**.
- Koneksi OpenBao ke PostgreSQL memakai role `openbao_rs_admin` / `openbao_rme_admin` / `openbao_audit_admin` (LOGIN, `CREATEROLE`, opsi `ADMIN` hanya atas grup sasaran di tabel di atas, tanpa superuser). Kemampuan role `CREATEROLE` non-superuser untuk memberi keanggotaan di PostgreSQL 16 ke atas **(V8)**. Role ini setara kuatnya dengan pemilik skema, sehingga passwordnya hanya di OpenBao dan perubahannya tercatat di log audit OpenBao `[SRC-226]`.
- Password role login aplikasi disimpan di OpenBao KV `[KEPUTUSAN-79]`. Rotasi: `ALTER ROLE ... PASSWORD` oleh admin bootstrap, perbarui KV, restart bergulir; prosedur tertulis sebelum go-live `[KEPUTUSAN-88]`. Fitur rotasi otomatis role statis di OpenBao dievaluasi Tahap 8 **(V8)**.

### 3.5 Bootstrap klaster (`db/roles/`)

Pembuatan role berlaku untuk seluruh klaster dan memerlukan `CREATEROLE`, yang tidak dimiliki pemilik skema. Karena itu `db/roles/` dijalankan **sekali per lingkungan oleh admin bootstrap** (kredensial administratif dinamis atau akses darurat), bukan oleh pemilik skema seperti tertulis di KEPUTUSAN-89 (§14 F-P08) `[KEPUTUSAN-97]`. Isinya, berurutan dan idempoten:
1. `CREATE ROLE` grup dan login (§3.2) dengan atribut §3.1, `GRANT <grup> TO <login>`.
2. `CREATE DATABASE` (pemilik superuser bootstrap), `REVOKE CONNECT, TEMP ON DATABASE ... FROM PUBLIC`, `GRANT CONNECT` per grup.
3. Per database: `DROP SCHEMA public`, `CREATE SCHEMA app|audit|util AUTHORIZATION <pemilik>`, `CREATE SCHEMA ext` + `CREATE EXTENSION pg_trgm SCHEMA ext`, `GRANT USAGE ON SCHEMA` ke grup yang berhak.
4. `ALTER ROLE <login> IN DATABASE <db> SET` untuk `search_path` dan batas waktu (§10.2).
5. `ALTER DATABASE <db> SET TimeZone = 'UTC'` (§10.1).

**Hak per tabel tidak ada di `db/roles/`**, tetapi di berkas migrasi yang sama dengan `CREATE TABLE`-nya (§4), supaya tabel baru tidak pernah ada tanpa hak yang benar atau dengan hak berlebih.

---

## 4. Templat DDL per kelas tabel

### 4.1 Ketentuan umum

1. **Komentar tabel sebagai penanda kelas** `[KEPUTUSAN-94]`. Setiap tabel diberi `COMMENT ON TABLE ... IS 'class:<M|MS|MK|T|R|P|O|L|OH> module:<modul> [alias:<alias>] [log_category:<kategori>]'` (`MS` = Master snapshot, `MK` = Master khusus kunci data, `OH` = Operasional hak khusus seperti outbox audit). Kolom bertanda "Anonimkan" di DD diberi `COMMENT ON COLUMN ... IS 'anonymize'`. Test katalog (§12 T-06–T-09) membaca penanda ini dan membandingkan hak, trigger, dan hak per kolom `privacy_worker` dengan aturan DD §0.6. Dengan begitu kesesuaian kelas dan hak diperiksa mesin, bukan hanya checklist DBA `[DOK: ADR-008 konsekuensi]`.
2. **Nilai bawaan kolom** `[KEPUTUSAN-93]`:
   - `id uuid NOT NULL` **tanpa** nilai bawaan. UUIDv7 dibuat aplikasi dengan `Uuid::now_v7` `[SRC-238]` `[DOK: ADR-002]`. Baris tanpa ID gagal di `NOT NULL`, sehingga bug terlihat.
   - `created_at timestamptz NOT NULL DEFAULT now()`; aplikasi tidak mengirim nilainya. `now()` adalah waktu awal transaksi **(V8)**, yang memenuhi "jam server" STD-R `[DOK: DD §0.4]`.
   - `updated_at` diisi pernyataan `UPDATE` (`updated_at = now()`) bersama `updated_by`; tanpa trigger penyentuh.
   - Baris data acuan yang diisi migrasi (misalnya `order_status_transition`, `journal_account`) memakai literal UUIDv7 tetap di berkas migrasi, sehingga ID sama di semua lingkungan `[DOK: ADR-002]`.
3. **FK tanpa `CASCADE`**. Semua FK memakai perilaku bawaan `NO ACTION`, karena data tidak dihapus keras kecuali baris Operasional dan Log yang dihapus secara eksplisit `[DOK: ADR-003]` `[DOK: ADR-004]` `[KEPUTUSAN-93]`.
4. **Index FK** hanya sesuai daftar index di DD dan usulan §9.3. FK aktor (`created_by`, `updated_by`, `deleted_by`) tidak diindeks, karena akun tidak pernah dihapus keras. PostgreSQL tidak membuat index FK otomatis **(V8)**.
5. **Kolom terenkripsi** `<nama>_enc bytea` mendapat CHECK format (§7.1): `ck_<tabel>_<kolom>_format CHECK (<kolom> IS NULL OR (octet_length(<kolom>) >= 29 AND get_byte(<kolom>, 0) = 1))` **(V8: fungsi `get_byte`)** `[KEPUTUSAN-104]`. CHECK ini hanya memeriksa bentuk amplop, bukan isi, sehingga tidak bertentangan dengan DD §0.2 ("isi terenkripsi tidak dapat di-CHECK"). Tujuannya menangkap bug yang menulis teks terbuka.
6. `TRUNCATE` tidak diberikan ke role mana pun selain pemilik skema `[DOK: DD §0.6]`.
7. Hak diberikan ke **role grup** (§3.2), tidak pernah ke role login.

### 4.2 Fungsi bersama (skema `util`, dibuat sekali per database)

Fungsi ini adalah "pustaka migrasi" DD §D.3 `[DOK: DD §D]`. Semuanya `SECURITY INVOKER` (berjalan dengan hak pelaku, sehingga tidak menambah hak), memasang `search_path` tetap, dan merujuk tabel dengan nama berkualifikasi `[KEPUTUSAN-95]`. Galat trigger membatalkan statement pemicunya `[SRC-40]`. Pesan galat diawali kode tetap (`immutable_violation`, `retention_not_set`, `retention_not_expired`, `data_key_shredded`, `effective_row_locked`) supaya aturan peringatan di security log dapat mencocokkannya (ADR-009 butir 4) `[DOK: ADR-009]`.

```sql
-- (1) Penolak umum: dipakai trigger tingkat statement di kelas T, R, L, data_key, dan audit
CREATE FUNCTION util.fn_reject_modification()
RETURNS trigger
LANGUAGE plpgsql
SET search_path = pg_catalog
AS $$
BEGIN
  RAISE EXCEPTION 'immutable_violation: % on %.% rejected', TG_OP, TG_TABLE_SCHEMA, TG_TABLE_NAME
    USING ERRCODE = 'insufficient_privilege';   -- nama kondisi galat (V8)
END;
$$;
REVOKE ALL ON FUNCTION util.fn_reject_modification() FROM PUBLIC;
```

```sql
-- (2) Penjaga DELETE kelas Log (database rumah_sehat). Argumen trigger:
--     TG_ARGV[0] = kategori retensi, TG_ARGV[1] = kolom acuan waktu (umumnya created_at)
CREATE FUNCTION util.fn_guard_log_delete()
RETURNS trigger
LANGUAGE plpgsql
SET search_path = pg_catalog
AS $$
DECLARE
  v_months integer;
  v_anchor timestamptz;
BEGIN
  IF NOT pg_has_role('rs_privacy_worker', 'MEMBER') THEN            -- (V8) semantik pg_has_role
    RAISE EXCEPTION 'immutable_violation: DELETE on %.% rejected', TG_TABLE_SCHEMA, TG_TABLE_NAME
      USING ERRCODE = 'insufficient_privilege';
  END IF;
  SELECT rp.retention_months INTO v_months
    FROM app.retention_policy rp
   WHERE rp.data_category = TG_ARGV[0]
     AND rp.deleted_at IS NULL
     AND rp.effective_from <= now()
   ORDER BY rp.effective_from DESC
   LIMIT 1;
  IF v_months IS NULL THEN                                            -- PT-24 belum terjawab: tidak ada yang terhapus
    RAISE EXCEPTION 'retention_not_set: % (%.%)', TG_ARGV[0], TG_TABLE_SCHEMA, TG_TABLE_NAME
      USING ERRCODE = 'insufficient_privilege';
  END IF;
  v_anchor := (to_jsonb(OLD) ->> TG_ARGV[1])::timestamptz;
  IF v_anchor IS NULL OR v_anchor > now() - make_interval(months => v_months) THEN
    RAISE EXCEPTION 'retention_not_expired: %.% id=%', TG_TABLE_SCHEMA, TG_TABLE_NAME, OLD.id
      USING ERRCODE = 'insufficient_privilege';
  END IF;
  RETURN OLD;
END;
$$;
```
Versi database `rme` identik, dengan `rme_privacy_worker` dan `app.rme_retention_policy` `[DOK: DD §0.6 C]` (review DBA B10). Nilai berlaku diambil dengan pola berlaku-mulai DD §0.8. Kategori per tabel Log baru final bersama PT-24; selama kategori tidak punya baris kebijakan, semua `DELETE` ditolak `[DOK: DD §0.6]` `[DOK: ADR-013 §5]`.

```sql
-- (3) Kunci baris berlaku-mulai (DD §0.8): baris yang sudah berlaku tidak boleh diubah atau di-soft delete
CREATE FUNCTION util.fn_effective_row_lock()
RETURNS trigger
LANGUAGE plpgsql
SET search_path = pg_catalog
AS $$
BEGIN
  IF OLD.effective_from <= now() OR NEW.effective_from <= now() THEN
    RAISE EXCEPTION 'effective_row_locked: %.% id=%', TG_TABLE_SCHEMA, TG_TABLE_NAME, OLD.id
      USING ERRCODE = 'insufficient_privilege';
  END IF;
  RETURN NEW;
END;
$$;
```
Dipasang `BEFORE UPDATE ... FOR EACH ROW` di `markup_rule`, `partner_fee_rule`, `consultation_rate`, `business_parameter`, `payment_method_fee`, `retention_policy`, `consent_text_version`, dan `rme_retention_policy` `[DOK: DD §0.8, A3.1, C6.9]`.

Fungsi satu arah `data_key` di §4.10; penjaga migrasi di §11.2; konteks tenant RME di §5.2.

### 4.3 Kelas M (Master)

Contoh: `rs_admin_role` `[DOK: DD A1.2]`.

```sql
CREATE TABLE app.rs_admin_role (
  id            uuid        NOT NULL,
  created_at    timestamptz NOT NULL DEFAULT now(),
  created_by    uuid        NULL,
  updated_at    timestamptz NOT NULL DEFAULT now(),
  updated_by    uuid        NULL,
  deleted_at    timestamptz NULL,
  deleted_by    uuid        NULL,
  delete_reason text        NULL,
  account_id    uuid        NOT NULL,
  sub_role      text        NOT NULL,
  CONSTRAINT pk_rs_admin_role PRIMARY KEY (id),
  CONSTRAINT fk_rs_admin_role_created_by FOREIGN KEY (created_by) REFERENCES app.account (id),
  CONSTRAINT fk_rs_admin_role_updated_by FOREIGN KEY (updated_by) REFERENCES app.account (id),
  CONSTRAINT fk_rs_admin_role_deleted_by FOREIGN KEY (deleted_by) REFERENCES app.account (id),
  CONSTRAINT fk_rs_admin_role_account_id FOREIGN KEY (account_id) REFERENCES app.account (id),
  CONSTRAINT ck_rs_admin_role_soft_delete CHECK (
    (deleted_at IS NULL AND deleted_by IS NULL AND delete_reason IS NULL)
    OR (deleted_at IS NOT NULL AND delete_reason IS NOT NULL)),
  CONSTRAINT ck_rs_admin_role_sub_role CHECK (sub_role IN
    ('super_admin','verifier','customer_service','finance','content','auditor','privacy_officer'))
);
CREATE UNIQUE INDEX uqp_rs_admin_role_account_id_sub_role
  ON app.rs_admin_role (account_id, sub_role) WHERE deleted_at IS NULL;   -- [SRC-33]
CREATE INDEX ix_rs_admin_role_sub_role ON app.rs_admin_role (sub_role) WHERE deleted_at IS NULL;
COMMENT ON TABLE app.rs_admin_role IS 'class:M module:identitas';

REVOKE ALL ON app.rs_admin_role FROM PUBLIC;
GRANT SELECT, INSERT, UPDATE ON app.rs_admin_role TO rs_app;        -- tanpa DELETE [DOK: ADR-003]
GRANT SELECT ON app.rs_admin_role TO rs_privacy_worker;
-- tabel ini tanpa kolom "Anonimkan", sehingga privacy_worker tidak mendapat UPDATE per kolom
```

Tabel Master yang punya kolom "Anonimkan" (misalnya `care_recipient`) menambah hak per kolom `[SRC-35]` `[DOK: DD §0.2]`:
```sql
GRANT UPDATE (full_name, birth_date, sex, nik_bidx, nik_last4, anonymized_at, updated_at, updated_by)
  ON app.care_recipient TO rs_privacy_worker;
```
`account_owner_nik_registry` menambah `deleted_at`, `deleted_by`, `delete_reason` `[DOK: DD §0.6 A]`. Pengecualian kelas Master: `order_status_transition` hanya `SELECT` untuk `rs_app` `[DOK: DD A8.13]`.

### 4.4 Kelas MS (Master snapshot)

Contoh: `order_personal_snapshot` `[DOK: DD A8.4]` `[KEPUTUSAN-69]`. Struktur kolom STD-M seperti §4.3; perbedaannya hanya hak:
```sql
COMMENT ON TABLE app.order_personal_snapshot IS 'class:MS module:pemesanan';
GRANT SELECT, INSERT ON app.order_personal_snapshot TO rs_app;               -- tanpa UPDATE
GRANT SELECT ON app.order_personal_snapshot TO rs_privacy_worker;
GRANT UPDATE (recipient_full_name, recipient_birth_date, recipient_sex, recipient_phone, address_line,
              province_code, city_code, district_code, postal_code, latitude, longitude, address_notes,
              anonymized_at, updated_at, updated_by)
  ON app.order_personal_snapshot TO rs_privacy_worker;
```

### 4.5 Kelas T (Transaksi)

Contoh: `order_partner_acknowledgement` `[DOK: DD A8.10]`.

```sql
CREATE TABLE app.order_partner_acknowledgement (
  id                uuid        NOT NULL,
  created_at        timestamptz NOT NULL DEFAULT now(),
  created_by        uuid        NULL,
  patient_order_id  uuid        NOT NULL,
  partner_order_ref text        NULL,
  CONSTRAINT pk_order_partner_acknowledgement PRIMARY KEY (id),
  CONSTRAINT fk_order_partner_acknowledgement_created_by FOREIGN KEY (created_by) REFERENCES app.account (id),
  CONSTRAINT fk_order_partner_acknowledgement_patient_order_id
    FOREIGN KEY (patient_order_id) REFERENCES app.patient_order (id),
  CONSTRAINT uq_order_partner_acknowledgement_patient_order_id UNIQUE (patient_order_id)
);
COMMENT ON TABLE app.order_partner_acknowledgement IS 'class:T module:pemesanan';

REVOKE ALL ON app.order_partner_acknowledgement FROM PUBLIC;
GRANT SELECT, INSERT ON app.order_partner_acknowledgement TO rs_app;
GRANT SELECT (id, created_at, patient_order_id) ON app.order_partner_acknowledgement TO rs_privacy_worker;

CREATE TRIGGER tg_order_partner_acknowledgement_reject_ud
  BEFORE UPDATE OR DELETE ON app.order_partner_acknowledgement
  FOR EACH STATEMENT EXECUTE FUNCTION util.fn_reject_modification();
CREATE TRIGGER tg_order_partner_acknowledgement_reject_truncate
  BEFORE TRUNCATE ON app.order_partner_acknowledgement
  FOR EACH STATEMENT EXECUTE FUNCTION util.fn_reject_modification();   -- TRUNCATE hanya tingkat statement [SRC-40]
```

Keputusan bentuk trigger `[KEPUTUSAN-95]`: trigger penolak `UPDATE`/`DELETE` di kelas T dan R memakai **tingkat statement**, sehingga statement ditolak walaupun tidak ada baris yang cocok (percobaan tetap terlihat di log). Dua trigger terpisah dipakai untuk `UPDATE OR DELETE` dan `TRUNCATE`. Akibat yang disengaja: `INSERT ... ON CONFLICT DO UPDATE` tidak dapat dipakai di tabel T, R, dan L (lagi pula role aplikasi tidak punya `UPDATE`); idempotensi memakai `ON CONFLICT DO NOTHING` atau penanganan pelanggaran unik **(V8: trigger yang terpicu oleh `ON CONFLICT`)**. Trigger penolak juga berlaku bagi pemilik skema, sehingga migrasi tidak dapat mengubah baris T/R/L tanpa langkah luar biasa (§11.4).

Hak `privacy_worker` atas T = kolom ID, FK subjek, `created_at`, dan `data_key_id` `[DOK: DD §0.6 A]`. Daftar kolom per tabel diturunkan dari DD saat migrasi ditulis.

### 4.6 Kelas R (Riwayat status)

Contoh: `refund_status_event` `[DOK: DD A14.2, §0.4 STD-R]`.

```sql
CREATE TABLE app.refund_status_event (
  id               uuid        NOT NULL,
  refund_id        uuid        NOT NULL,
  version          integer     NOT NULL,
  from_status      text        NULL,
  to_status        text        NOT NULL,
  reason_code      text        NULL,
  note_enc         bytea       NULL,
  note_data_key_id uuid        NULL,
  actor_type       text        NOT NULL,
  actor_account_id uuid        NULL,
  actor_partner_id uuid        NULL,
  correlation_id   uuid        NOT NULL,
  created_at       timestamptz NOT NULL DEFAULT now(),
  CONSTRAINT pk_refund_status_event PRIMARY KEY (id),
  CONSTRAINT fk_refund_status_event_refund_id FOREIGN KEY (refund_id) REFERENCES app.refund (id),
  CONSTRAINT fk_refund_status_event_note_data_key_id FOREIGN KEY (note_data_key_id) REFERENCES app.data_key (id),
  CONSTRAINT fk_refund_status_event_actor_account_id FOREIGN KEY (actor_account_id) REFERENCES app.account (id),
  CONSTRAINT fk_refund_status_event_actor_partner_id FOREIGN KEY (actor_partner_id) REFERENCES app.partner (id),
  CONSTRAINT uq_refund_status_event_refund_id_version UNIQUE (refund_id, version),             -- AB-36
  CONSTRAINT uq_refund_status_event_refund_id_version_to_status UNIQUE (refund_id, version, to_status), -- sasaran FK proyeksi (§4.7)
  CONSTRAINT ck_refund_status_event_version CHECK (version >= 1),
  CONSTRAINT ck_refund_status_event_from_status CHECK ((version = 1) = (from_status IS NULL)),
  CONSTRAINT ck_refund_status_event_to_status CHECK (to_status IN ('processing','succeeded','failed','needs_review')),
  CONSTRAINT ck_refund_status_event_note CHECK ((note_enc IS NULL) = (note_data_key_id IS NULL)),
  CONSTRAINT ck_refund_status_event_note_enc_format CHECK (note_enc IS NULL
    OR (octet_length(note_enc) >= 29 AND get_byte(note_enc, 0) = 1)),
  CONSTRAINT ck_refund_status_event_actor_type CHECK (actor_type IN
    ('patient','health_worker','rs_admin','partner_admin','partner_system','payment_gateway','courier','system'))
);
COMMENT ON TABLE app.refund_status_event IS 'class:R module:pembayaran';
GRANT SELECT, INSERT ON app.refund_status_event TO rs_app;
GRANT SELECT (id, note_data_key_id) ON app.refund_status_event TO rs_privacy_worker;
-- trigger tg_refund_status_event_reject_ud dan tg_refund_status_event_reject_truncate seperti §4.5
```
Varian RME mengikuti STD-R varian RME `[DOK: DD §0.4]` ditambah RLS (§5).

### 4.7 Kelas P (Proyeksi)

Contoh: `refund_state` `[DOK: DD A14.3, §0.4 STD-P]`.

```sql
CREATE TABLE app.refund_state (
  refund_id  uuid        NOT NULL,
  status     text        NOT NULL,
  version    integer     NOT NULL,
  updated_at timestamptz NOT NULL DEFAULT now(),
  CONSTRAINT pk_refund_state PRIMARY KEY (refund_id),
  CONSTRAINT fk_refund_state_refund_id FOREIGN KEY (refund_id) REFERENCES app.refund (id),
  CONSTRAINT fk_refund_state_refund_id_version_status FOREIGN KEY (refund_id, version, status)
    REFERENCES app.refund_status_event (refund_id, version, to_status),       -- usulan [KEPUTUSAN-142]
  CONSTRAINT ck_refund_state_status CHECK (status IN ('processing','succeeded','failed','needs_review')),
  CONSTRAINT ck_refund_state_version CHECK (version >= 1)
);
COMMENT ON TABLE app.refund_state IS 'class:P module:pembayaran';
GRANT SELECT, INSERT, UPDATE ON app.refund_state TO rs_app;     -- tanpa DELETE [DOK: ADR-008]
```

Jalur tulis (ADR-008 aturan status 2) `[DOK: ADR-008]`: `INSERT` event `version = n+1`, lalu `UPDATE app.refund_state SET status = $s, version = $n + 1, updated_at = now() WHERE refund_id = $id AND version = $n`. Nol baris ter-update berarti kalah balapan, sehingga transaksi dibatalkan dan dievaluasi ulang.

**Usulan FK proyeksi ke riwayat** `[KEPUTUSAN-142]`: FK komposit `(<entitas>_id, version, status)` ke `UQ (<entitas>_id, version, to_status)` di tabel riwayat membuat proyeksi **tidak dapat menunjuk status yang tidak pernah tercatat di riwayat**. Pembandingan harian ADR-008 butir 4 tetap dibutuhkan untuk proyeksi yang tertinggal versi. Biayanya satu index unik tambahan per tabel riwayat. Ini penambahan constraint di luar DD, sehingga dibawa ke review DBA (§14 F-P12).

### 4.8 Kelas O (Operasional)

Contoh: `session_revocation` `[DOK: DD A1.4]`.

```sql
CREATE TABLE app.session_revocation (
  id             uuid        NOT NULL,
  account_id     uuid        NOT NULL,
  idp_session_id text        NOT NULL,
  reason_code    text        NOT NULL,
  created_at     timestamptz NOT NULL DEFAULT now(),
  expires_at     timestamptz NOT NULL,
  CONSTRAINT pk_session_revocation PRIMARY KEY (id),
  CONSTRAINT fk_session_revocation_account_id FOREIGN KEY (account_id) REFERENCES app.account (id),
  CONSTRAINT uq_session_revocation_idp_session_id UNIQUE (idp_session_id),
  CONSTRAINT ck_session_revocation_reason_code CHECK (reason_code IN
    ('logout','admin_revoke','account_deactivated','role_changed'))
);
CREATE INDEX ix_session_revocation_expires_at ON app.session_revocation (expires_at);
COMMENT ON TABLE app.session_revocation IS 'class:O module:identitas';
GRANT SELECT, INSERT, UPDATE, DELETE ON app.session_revocation TO rs_app;
```
Tabel operasional berisi data pribadi menambah `GRANT SELECT, DELETE ... TO rs_privacy_worker` `[DOK: DD §0.6 A]`.

Pengaturan penyimpanan tabel O yang sering di-update/dihapus (`job`, `rme_job`, ketiga outbox, `queue_board`, `inventory_hold`), nilai awal `[KEPUTUSAN-109]` **(V8: nama parameter penyimpanan per tabel)**, divalidasi uji beban NFR-CAP-04:
```sql
ALTER TABLE app.job SET (autovacuum_vacuum_scale_factor = 0.02, autovacuum_analyze_scale_factor = 0.02);
```
`fillfactor` tidak diturunkan dulu: perubahan `status` mengubah keanggotaan partial index `WHERE status = 'queued'`, sehingga pembaruan *heap-only* tidak dapat diharapkan **(V8)**.

**Kelas OH (outbox audit)** `[DOK: DD A19.1, C6.5]`:
```sql
GRANT INSERT ON app.audit_outbox TO rs_app, rs_privacy_worker;         -- INSERT saja [DOK: ADR-009 butir 2]
GRANT SELECT, DELETE ON app.audit_outbox TO rs_audit_relay;
```
Karena role aplikasi tidak punya `SELECT`, kode tidak boleh memakai `INSERT ... RETURNING` di tabel ini **(V8: `RETURNING` memerlukan hak `SELECT`)**; test integrasi pustaka audit menjalankan penulisan dengan role `rs_app` sungguhan.

### 4.9 Kelas L (Log)

Contoh: `order_follow_up_flag` `[DOK: DD A8.11]`.

```sql
CREATE TABLE app.order_follow_up_flag (
  id               uuid        NOT NULL,
  created_at       timestamptz NOT NULL DEFAULT now(),
  patient_order_id uuid        NOT NULL,
  flag_type        text        NOT NULL,
  CONSTRAINT pk_order_follow_up_flag PRIMARY KEY (id),
  CONSTRAINT fk_order_follow_up_flag_patient_order_id FOREIGN KEY (patient_order_id) REFERENCES app.patient_order (id),
  CONSTRAINT ck_order_follow_up_flag_flag_type CHECK (flag_type IN
    ('partner_delivery_failing','home_visit_closed_unconfirmed'))
);
CREATE INDEX ix_order_follow_up_flag_flag_type_created_at ON app.order_follow_up_flag (flag_type, created_at);
COMMENT ON TABLE app.order_follow_up_flag IS 'class:L module:pemesanan log_category:<ditetapkan bersama PT-24>';

GRANT SELECT, INSERT ON app.order_follow_up_flag TO rs_app;
GRANT SELECT (id, created_at, patient_order_id) ON app.order_follow_up_flag TO rs_privacy_worker;
GRANT DELETE ON app.order_follow_up_flag TO rs_privacy_worker;          -- dibatasi trigger penjaga

CREATE TRIGGER tg_order_follow_up_flag_reject_update
  BEFORE UPDATE ON app.order_follow_up_flag
  FOR EACH STATEMENT EXECUTE FUNCTION util.fn_reject_modification();
CREATE TRIGGER tg_order_follow_up_flag_reject_truncate
  BEFORE TRUNCATE ON app.order_follow_up_flag
  FOR EACH STATEMENT EXECUTE FUNCTION util.fn_reject_modification();
CREATE TRIGGER tg_order_follow_up_flag_guard_delete
  BEFORE DELETE ON app.order_follow_up_flag
  FOR EACH ROW EXECUTE FUNCTION util.fn_guard_log_delete('<kategori PT-24>', 'created_at');
```
Penjaga `DELETE` memakai trigger **tingkat baris**, karena masa retensi dinilai per baris `[DOK: DD §0.6 A]` `[DOK: ADR-013 §5]` `[KEPUTUSAN-95]`. Penghapusan dilakukan bertahap dalam batch kecil, dicatat E-16 lewat `audit_outbox`, dan dirangkum di `retention_run` `[DOK: DD §0.6, §D.2]`. `retention_run` sendiri: `rs_privacy_worker` S, I `[DOK: DD §0.6 A]`.

### 4.10 Kelas MK (`data_key`, `rme_data_key`)

Struktur kolom di DD A19.7 dan C6.4. Fisik `[DOK: DD A19.7]` `[KEPUTUSAN-66]` `[KEPUTUSAN-96]`:

```sql
CREATE FUNCTION util.fn_data_key_one_way()
RETURNS trigger
LANGUAGE plpgsql
SET search_path = pg_catalog
AS $$
BEGIN
  IF OLD.shredded_at IS NOT NULL THEN                          -- DEK yang sudah dimusnahkan tidak dapat diubah siapa pun
    RAISE EXCEPTION 'data_key_shredded: id=% immutable', OLD.id USING ERRCODE = 'insufficient_privilege';
  END IF;
  IF NEW.id IS DISTINCT FROM OLD.id
     OR NEW.created_at IS DISTINCT FROM OLD.created_at
     OR NEW.container_type IS DISTINCT FROM OLD.container_type
     OR NEW.container_id IS DISTINCT FROM OLD.container_id
     OR NEW.retention_category IS DISTINCT FROM OLD.retention_category
     OR NEW.retention_anchor_at IS DISTINCT FROM OLD.retention_anchor_at THEN
    RAISE EXCEPTION 'immutable_violation: data_key id=% column not updatable', OLD.id
      USING ERRCODE = 'insufficient_privilege';
  END IF;
  IF pg_has_role('rs_privacy_worker', 'MEMBER') THEN            -- pemusnahan (S-N3)
    IF NEW.wrapped_dek IS NOT NULL OR NEW.shred_reason IS NULL
       OR NEW.kek_ref IS DISTINCT FROM OLD.kek_ref THEN
      RAISE EXCEPTION 'immutable_violation: invalid shred on data_key id=%', OLD.id
        USING ERRCODE = 'insufficient_privilege';
    END IF;
    NEW.shredded_at := now();                                   -- waktu pemusnahan dari server
    RETURN NEW;
  ELSIF pg_has_role('rs_key_rotation', 'MEMBER') THEN           -- bungkus ulang (cadangan)
    IF NEW.wrapped_dek IS NULL OR NEW.shredded_at IS NOT NULL OR NEW.shred_reason IS NOT NULL THEN
      RAISE EXCEPTION 'immutable_violation: invalid rewrap on data_key id=%', OLD.id
        USING ERRCODE = 'insufficient_privilege';
    END IF;
    RETURN NEW;
  END IF;
  RAISE EXCEPTION 'immutable_violation: UPDATE on data_key rejected' USING ERRCODE = 'insufficient_privilege';
END;
$$;

CREATE TRIGGER tg_data_key_one_way BEFORE UPDATE ON app.data_key
  FOR EACH ROW EXECUTE FUNCTION util.fn_data_key_one_way();
CREATE TRIGGER tg_data_key_reject_delete BEFORE DELETE ON app.data_key
  FOR EACH STATEMENT EXECUTE FUNCTION util.fn_reject_modification();
CREATE TRIGGER tg_data_key_reject_truncate BEFORE TRUNCATE ON app.data_key
  FOR EACH STATEMENT EXECUTE FUNCTION util.fn_reject_modification();

COMMENT ON TABLE app.data_key IS 'class:MK module:platform';
GRANT SELECT, INSERT ON app.data_key TO rs_app;
GRANT SELECT ON app.data_key TO rs_privacy_worker;
GRANT UPDATE (wrapped_dek, shredded_at, shred_reason) ON app.data_key TO rs_privacy_worker;   -- [SRC-35]
GRANT SELECT (id, wrapped_dek, kek_ref, shredded_at) ON app.data_key TO rs_key_rotation;
GRANT UPDATE (wrapped_dek, kek_ref) ON app.data_key TO rs_key_rotation;
```
- CHECK DD `(wrapped_dek IS NULL) = (shredded_at IS NOT NULL)` tetap berlaku; trigger mengisi `shredded_at` sehingga CHECK terpenuhi dalam statement yang sama dengan `wrapped_dek = NULL` `[DOK: DD A19.7]`.
- `rs_key_rotation` tidak punya role login tetap; anggota dinamis hanya diterbitkan saat prosedur bungkus ulang (§7.3). Cara utama rotasi KEK adalah versi kunci OpenBao Transit yang tetap dapat membuka DEK lama selama versinya di atas `min_decryption_version` `[SRC-222]` `[KEPUTUSAN-79]`.
- Celah DD soal `retention_anchor_at` yang boleh `NULL` tetapi tidak dapat diisi belakangan: §14 F-P04.

### 4.11 Ringkasan trigger per kelas

| Kelas | Trigger | Sumber |
| --- | --- | --- |
| M, MS, P | Tanpa trigger penolak; hak `DELETE` tidak diberikan. Pola berlaku-mulai memakai `fn_effective_row_lock` | `[DOK: DD §0.3, §0.8]` |
| T, R | `reject_ud` (statement) + `reject_truncate` (statement) | `[DOK: DD §0.3]`, `[SRC-40]` |
| L | `reject_update` (statement) + `reject_truncate` (statement) + `guard_delete` (baris) | `[DOK: DD §0.6]`, `[DOK: ADR-013 §5]` |
| MK | `one_way` (baris) + `reject_delete` + `reject_truncate` | `[KEPUTUSAN-66]` |
| O, OH | Tanpa trigger | `[DOK: DD §0.3]` |
| Audit (`audit_event`, `audit_event_subject`, `audit_event_key`, `audit_chain_anchor`, `audit_chain_verification`, `audit_archive_manifest`) | `reject_ud` + `reject_truncate`, juga di setiap partisi (§6.2) | `[DOK: ADR-009 butir 4]` |

---

## 5. Row Level Security di RME

Dasar: semua tabel milik tenant di database `rme` memakai RLS dengan `FORCE ROW LEVEL SECURITY`, role aplikasi bukan pemilik dan tanpa `BYPASSRLS` `[SRC-34]` `[DOK: ADR-012 butir 6]` `[DOK: DD §0.6 C]`. Bentuk kebijakan per role diverifikasi ke halaman Row Security Policies `[DOK: DD §0.6 C]`; halaman itu sudah terdaftar sebagai SRC-34, tetapi kutipan yang tercatat baru mencakup `BYPASSRLS` dan `FORCE`. Karena itu sintaks `CREATE POLICY`, perilaku tanpa kebijakan (tolak semua), `USING` vs `WITH CHECK`, dan perlakuan pemeriksaan FK terhadap RLS ditandai **(V8)**.

### 5.1 Konteks tenant per transaksi

`[KEPUTUSAN-98]`:
- Nama pengaturan: `rme.fasyankes_id` (pengaturan buatan sendiri berawalan titik **(V8)**). Database `rme_audit` memakai nama yang sama (§5.6).
- Dipasang **hanya** dengan `SELECT set_config('rme.fasyankes_id', $1, true)` di awal transaksi. Argumen ketiga `true` berarti nilainya hanya berlaku sampai akhir transaksi **(V8)**, sehingga tidak terbawa ke pemakai koneksi berikutnya di pool (ancaman RME-02). `set_config` dipilih, bukan `SET LOCAL`, karena nilainya dapat diikat sebagai parameter query, bukan disusun sebagai teks SQL.
- Nilai `fasyankes_id` berasal dari sesi pengguna yang terautentikasi atau dari tanda tangan HMAC webhook yang terverifikasi, tidak pernah dari parameter permintaan `[DOK: ADR-012 butir 6]` `[DOK: DD C6.8]`.
- Di kode Rust, akses tabel tenant RME hanya lewat pembungkus `with_tenant_tx(pool, fasyankes_id, |tx| ...)` di crate `authz` `[KEPUTUSAN-74]`, yang membuka transaksi lalu memasang konteks. Test arsitektur di `verify.sh` menolak pemanggilan `set_config` dengan argumen ketiga selain `true`, `SET rme.` tanpa `LOCAL`, dan pembukaan transaksi langsung di repository RME `[KEPUTUSAN-82]`.

### 5.2 Fungsi konteks dan kegagalan tertutup

```sql
CREATE FUNCTION util.fn_current_fasyankes_id()
RETURNS uuid
LANGUAGE sql
STABLE
AS $$ SELECT NULLIF(pg_catalog.current_setting('rme.fasyankes_id', true), '')::uuid $$;
REVOKE ALL ON FUNCTION util.fn_current_fasyankes_id() FROM PUBLIC;
GRANT EXECUTE ON FUNCTION util.fn_current_fasyankes_id() TO rme_app, rme_worker;
GRANT USAGE ON SCHEMA util TO rme_app, rme_worker;
```
- Bila konteks belum dipasang, `current_setting(..., true)` mengembalikan `NULL` atau string kosong **(V8)**. `NULLIF` mengubah keduanya menjadi `NULL`, dan `fasyankes_id = NULL` tidak pernah benar, sehingga **tidak ada baris yang terlihat atau dapat ditulis** (H-12 "gagal-tertutup") `[DOK: 04-threat-model.md H-12]`. Nilai yang bukan UUID menghasilkan galat, juga tertutup.
- Fungsi tidak memasang `SET search_path` supaya dapat di-*inline* oleh planner **(V8)**; semua rujukannya berkualifikasi `pg_catalog`.
- Kebijakan memakai bentuk `(SELECT util.fn_current_fasyankes_id())` supaya nilai dihitung sekali per query **(V8)**.

### 5.3 Templat kebijakan

Contoh: `rme_user_role` `[DOK: DD C1.7]`.

```sql
ALTER TABLE app.rme_user_role ENABLE ROW LEVEL SECURITY;
ALTER TABLE app.rme_user_role FORCE ROW LEVEL SECURITY;                      -- [SRC-34]

-- (1) Tenant: rme_app (dan rme_worker untuk tabel yang ia akses per job)
CREATE POLICY pol_rme_user_role_tenant ON app.rme_user_role
  AS PERMISSIVE FOR ALL TO rme_app
  USING      (fasyankes_id = (SELECT util.fn_current_fasyankes_id()))
  WITH CHECK (fasyankes_id = (SELECT util.fn_current_fasyankes_id()));

-- (2) Operator RME: hanya lima tabel onboarding + rme_data_key [DOK: DD §0.6 C]
CREATE POLICY pol_rme_user_role_operator ON app.rme_user_role
  AS PERMISSIVE FOR ALL TO rme_operator
  USING (true) WITH CHECK (true);                                            -- dipersempit M-24, lihat di bawah

-- (3) Pekerja privasi: lintas tenant, dibatasi hak per kolom [DOK: DD §0.6 C]
CREATE POLICY pol_rme_user_role_privacy ON app.rme_user_role
  AS PERMISSIVE FOR ALL TO rme_privacy_worker USING (true) WITH CHECK (true);

-- (4) Pemilik skema: FORCE juga berlaku bagi pemilik, sehingga migrasi data perlu kebijakan eksplisit
CREATE POLICY pol_rme_user_role_migration ON app.rme_user_role
  AS PERMISSIVE FOR ALL TO rme_migration USING (true) WITH CHECK (true);

GRANT SELECT, INSERT, UPDATE ON app.rme_user_role TO rme_app, rme_operator;
GRANT SELECT ON app.rme_user_role TO rme_privacy_worker;
```

Kebijakan per role `[KEPUTUSAN-98]`:

| Role | Bentuk kebijakan | Tabel |
| --- | --- | --- |
| `rme_app` | Tenant (`fasyankes_id` = konteks). Di `fasyankes`: `id` = konteks | Semua tabel ber-RLS yang ia akses |
| `rme_worker` | Tenant untuk tabel data; `USING (true) WITH CHECK (true)` hanya di `rme_job` dan `rme_integration_outbox` (antrean lintas tenant) | DD §0.6 C rincian `rme_worker` |
| `rme_auth` | `FOR SELECT USING (true)` di `rme_user` saja, dengan `GRANT SELECT (id, idp_subject, fasyankes_id, status)` | `[DOK: DD C1.6]` `[KEPUTUSAN-55]` |
| `rme_operator` | `USING (true)` di `fasyankes`, `fasyankes_location`, `rs_integration_credential`, `rme_user`, `rme_user_role`, `rme_data_key`; tanpa kebijakan dan tanpa hak di tabel klinis | `[DOK: ADR-012 butir 6]` |
| `rme_privacy_worker` | `USING (true)` di tabel tempat ia punya hak; `rme_retention_run` S, I lintas tenant | `[DOK: DD §0.6 C, C6.10]` |
| `rme_migration` | `USING (true) WITH CHECK (true)` di semua tabel ber-RLS | `[KEPUTUSAN-98]` |
| `rme_audit_relay` | — (`rme_audit_outbox` tanpa RLS) | `[DOK: DD C6.5]` |

Alasan kebijakan untuk `rme_migration`: dengan `FORCE`, pemilik tunduk pada RLS `[SRC-34]`, sehingga migrasi data tanpa kebijakan akan diam-diam tidak melihat baris. Pemilik tetap dapat mengubah kebijakan, jadi kebijakan eksplisit ini tidak menambah kemampuan, hanya membuatnya terlihat dan dapat direview. Migrasi apa pun yang menyentuh kebijakan RLS wajib direview DBA dan keamanan `[DOK: ADR-021 butir 4]`.

Tabel tanpa RLS di database `rme`: `terminology_code`, `rme_retention_policy` (acuan global) dan `rme_audit_outbox` (hak INSERT saja) `[DOK: DD C5.6, C6.9, C6.5]`. Test T-10 memastikan daftar ini tertutup.

**Batas hak Operator RME (M-24, terbuka).** Bentuk penegakannya di database diusulkan di Tahap 8 bersama M-24: `WITH CHECK` kebijakan `rme_operator` di `rme_user_role` hanya mengizinkan `role = 'facility_admin'` untuk fasyankes berstatus `onboarding` tanpa Admin Fasyankes aktif. Kebijakan yang membaca tabelnya sendiri dapat memicu galat rekursi kebijakan **(V8)**, sehingga pemeriksaan "belum ada admin aktif" kemungkinan memerlukan fungsi `SECURITY DEFINER` sempit. Belum diputuskan di dokumen ini `[DOK: 04-threat-model.md M-24]`.

### 5.4 Interaksi dengan FK komposit tenant

FK komposit `(fasyankes_id, <induk>_id)` `[DOK: DD C]` tetap menjadi pertahanan kedua bila RLS salah konfigurasi. Pemeriksaan integritas referensial di PostgreSQL disebut melewati RLS (halaman SRC-34, kutipan belum dicatat) **(V8)**; akibatnya FK tetap valid walaupun baris induk tidak terlihat oleh role, dan FK komposit mencegah rujukan ke baris tenant lain. Butir DD §D.4 (e) tetap terbuka (§13).

### 5.5 Test wajib RLS (bagian dari `verify.sh` langkah 2)

| ID | Test | Dasar |
| --- | --- | --- |
| T-10 | Setiap tabel di skema `app` database `rme`, kecuali daftar tertutup §5.3, punya RLS aktif dan `FORCE` (katalog `pg_class`) **(V8: nama kolom katalog)** | `[DOK: ADR-012 butir 6]` |
| T-11 | Untuk setiap tabel ber-RLS dan setiap role yang punya hak atasnya, ada kebijakan yang menyebut role itu (mencegah tolak-semua diam-diam) | `[KEPUTUSAN-98]` |
| T-12 | Isolasi per tabel: dua fasyankes sintetis; dengan konteks A, `SELECT` hanya baris A; `INSERT` dengan `fasyankes_id` B ditolak; `UPDATE`/`DELETE` baris B mengenai 0 baris | NFR-SEC-06 `[DOK: ADR-012 konsekuensi]` |
| T-13 | Gagal-tertutup: tanpa konteks dan dengan konteks string kosong, 0 baris terlihat dan `INSERT` ditolak; konteks bukan UUID menghasilkan galat | H-12 |
| T-14 | Kebocoran pool: pool berukuran 1; transaksi 1 memasang konteks A lalu commit; transaksi 2 di koneksi yang sama tanpa konteks melihat 0 baris | RME-02, H-12 |
| T-15 | Role lintas tenant: `rme_auth` gagal membaca kolom selain empat kolom pemetaan; `rme_worker` melihat semua baris antrean tetapi hanya baris tenant di tabel data; `rme_operator` mendapat galat hak atas `rme_patient`, `rme_visit*`, `rme_medical_record*`, `rme_lab_*`, `satusehat_*` | RME-03, RME-04, `[DOK: DD §0.6 C]` |
| T-16 | FK komposit menolak rujukan lintas tenant walaupun dijalankan oleh `rme_migration` | `[DOK: DD C]` |
| T-17 | Database `rme_audit`: `rme_audit_tenant_reader` lewat tabel induk hanya melihat E-05 fasyankesnya; akses langsung ke partisi ditolak | AUD-04, §5.6 |

### 5.6 RLS di database audit RME

Dasar: `rme_audit_tenant_reader` membaca `audit_event` dan `audit_event_subject` dengan RLS per `tenant_id` dan `FORCE` `[DOK: DD §0.6 B]` `[DOK: ADR-012 butir 9]`. Perilaku RLS pada tabel terpartisi belum terverifikasi **(V8)**. Rancangan utama `[KEPUTUSAN-99]`:

```sql
-- database rme_audit
ALTER TABLE audit.audit_event ENABLE ROW LEVEL SECURITY;
ALTER TABLE audit.audit_event FORCE ROW LEVEL SECURITY;

CREATE POLICY pol_audit_event_tenant_reader ON audit.audit_event
  FOR SELECT TO rme_audit_tenant_reader
  USING (tenant_id = (SELECT util.fn_current_fasyankes_id()) AND event_code = 'e_05');
CREATE POLICY pol_audit_event_writer ON audit.audit_event
  FOR INSERT TO rme_audit_writer WITH CHECK (true);
CREATE POLICY pol_audit_event_reader ON audit.audit_event
  FOR SELECT TO rme_audit_reader, rme_audit_verifier USING (true);
CREATE POLICY pol_audit_event_migration ON audit.audit_event
  FOR ALL TO rme_audit_migration USING (true) WITH CHECK (true);

-- audit_event_subject tidak punya tenant_id (DD B3), sehingga dibatasi lewat event induknya;
-- subquery ikut tunduk pada RLS audit_event untuk role yang sama (V8)
CREATE POLICY pol_audit_event_subject_tenant_reader ON audit.audit_event_subject
  FOR SELECT TO rme_audit_tenant_reader
  USING (EXISTS (SELECT 1 FROM audit.audit_event e
                  WHERE e.event_id = audit_event_subject.event_id
                    AND e.recorded_at = audit_event_subject.recorded_at));
-- ditambah kebijakan writer/reader/verifier/migration seperti audit_event

GRANT SELECT ON audit.audit_event, audit.audit_event_subject TO rme_audit_tenant_reader;  -- induk saja, tanpa hak atas partisi
```
- **Semua role yang menyentuh tabel ini perlu kebijakan**, bukan hanya `audit_tenant_reader`, karena RLS yang aktif berlaku untuk setiap role yang tidak melewatinya **(V8)**. DD §0.6 B hanya menyebut kebijakan untuk `audit_tenant_reader` (§14 F-P02).
- Filter `event_code = 'e_05'` mengikuti ADR-009 butir 8 dan PRD §7.5 (Admin Fasyankes membaca E-05 fasyankesnya) `[DOK: ADR-009 butir 8]`.
- Hak hanya diberikan pada tabel induk, sehingga partisi tidak dapat dibaca langsung. Apakah hak atas induk cukup untuk membaca baris partisi lewat induk **(V8)**.
- **Cadangan** bila T-17 menunjukkan RLS pada tabel terpartisi tidak memadai: fungsi `audit.fn_tenant_access_log(p_fasyankes_id uuid, p_from timestamptz, p_to timestamptz)` `SECURITY DEFINER`, hanya mengembalikan event E-05 tenant itu, dan `rme_audit_tenant_reader` hanya punya `EXECUTE` atas fungsi tersebut `[DOK: ADR-012 butir 9]`.
- Di `rumah_sehat_audit` RLS tidak diaktifkan; Auditor membaca lewat `rs_audit_reader` `[DOK: ADR-009 butir 8]`.

---

## 6. Partisi `audit_event`, rantai hash, dan arsip

### 6.1 DDL induk

`audit_event` dan `audit_event_subject` dipartisi per bulan berdasarkan `recorded_at` dengan PK komposit yang memuat kunci partisi, karena constraint unik di tabel terpartisi wajib memuat semua kolom kunci partisi `[SRC-39]` `[KEPUTUSAN-56]` `[DOK: DD B1, B3]`. Sintaks deklaratif mengikuti halaman SRC-39; kutipan sintaksnya belum dicatat di register **(V8)**.

```sql
CREATE TABLE audit.audit_event (
  event_id            uuid        NOT NULL,
  recorded_at         timestamptz NOT NULL,
  seq                 bigint      NOT NULL,
  -- ... kolom lain persis DD B1 ...
  prev_hash           bytea       NOT NULL,
  event_hash          bytea       NOT NULL,
  CONSTRAINT pk_audit_event PRIMARY KEY (event_id, recorded_at),
  CONSTRAINT ck_audit_event_prev_hash CHECK (octet_length(prev_hash) = 32),
  CONSTRAINT ck_audit_event_event_hash CHECK (octet_length(event_hash) = 32)
  -- ... CHECK isi DD B1 (event_code, actor_type, outcome, subject_count) ...
) PARTITION BY RANGE (recorded_at);

CREATE INDEX ix_audit_event_recorded_at ON audit.audit_event (recorded_at DESC);
-- ... index DD B1 lainnya didefinisikan di induk; partisi mewarisinya otomatis (V8)
COMMENT ON TABLE audit.audit_event IS 'class:L module:audit';

CREATE TABLE audit.audit_event_key (
  event_id    uuid        NOT NULL,
  seq         bigint      NOT NULL,
  recorded_at timestamptz NOT NULL,
  CONSTRAINT pk_audit_event_key PRIMARY KEY (event_id),
  CONSTRAINT uq_audit_event_key_seq UNIQUE (seq),
  CONSTRAINT ck_audit_event_key_seq CHECK (seq >= 1)
);
```
Panjang 32 byte untuk `prev_hash` dan `event_hash` mengikuti keluaran SHA-256 `[DOK: ADR-009 butir 5]`; jumlah byte SHA-256 sendiri **(V8)** (tidak ada SRC FIPS 180-4).

Hak `[DOK: DD §0.6 B]` (contoh Rumah Sehat; RME memakai awalan `rme_`):
```sql
GRANT INSERT ON audit.audit_event, audit.audit_event_subject, audit.audit_event_key, audit.audit_chain_anchor
  TO rs_audit_writer;
GRANT SELECT ON audit.audit_event, audit.audit_event_subject, audit.audit_event_key,
  audit.audit_chain_anchor, audit.audit_chain_verification, audit.audit_archive_manifest
  TO rs_audit_reader, rs_audit_verifier;
GRANT INSERT ON audit.audit_chain_verification TO rs_audit_verifier;
-- audit_archive_manifest: INSERT hanya oleh pemilik (rs_audit_migration) [DOK: DD §0.6 B]
```

### 6.2 Pembuatan partisi ke depan

Instruksi tahap ini menyebut pembuatan partisi "oleh job". Membuat partisi memerlukan hak pemilik atas tabel induk **(V8)**, padahal kredensial pemilik skema hanya tersedia pada langkah migrasi `[DOK: ADR-021 butir 4]` dan DD §0.6 B tidak memberi hak DDL ke role runtime mana pun. Karena itu `[KEPUTUSAN-100]`:

1. **Migrasi membuat partisi jauh ke depan**: migrasi awal database audit membuat partisi bulanan dari `2026-11` sampai `2029-12` untuk `audit_event` dan `audit_event_subject`. Tanggal mulai berupa literal tetap (bukan `now()`), sehingga migrasi deterministik di semua lingkungan.
2. **Job hanya memantau**: job verifikasi harian mengirim metrik jumlah partisi masa depan; peringatan bila kurang dari 6 bulan `[KEPUTUSAN-80]`. Tim lalu menulis migrasi lanjutan untuk 36 bulan berikutnya.
3. **Tanpa partisi `DEFAULT`**. Bila partisi bulan berjalan tidak ada, `INSERT` relay gagal, event tertahan di outbox tanpa hilang, dan peringatan jeda relay di atas 5 menit menyala `[DOK: ADR-009 butir 3]`. Partisi default ditolak karena menampung baris secara diam-diam dan menyulitkan penambahan partisi kemudian **(V8)**.
4. Batas bulan dalam **UTC**, sesuai penyimpanan `timestamptz` `[SRC-41]`.
5. Trigger penolak dibuat **di induk dan di setiap partisi**, karena statement yang langsung menyasar partisi tidak memicu trigger tingkat statement milik induk **(V8)**. Test T-18 memastikan setiap partisi punya kedua trigger dan semua index induk.

```sql
DO $$
DECLARE
  m date;
  p text;
BEGIN
  FOR m IN SELECT generate_series(date '2026-11-01', date '2029-12-01', interval '1 month')::date LOOP
    FOREACH p IN ARRAY ARRAY['audit_event', 'audit_event_subject'] LOOP
      EXECUTE format(
        'CREATE TABLE audit.%I PARTITION OF audit.%I FOR VALUES FROM (%L) TO (%L)',
        p || '_' || to_char(m, '"y"YYYY"m"MM'), p,
        to_char(m, 'YYYY-MM-DD') || ' 00:00:00+00',
        to_char(m + interval '1 month', 'YYYY-MM-DD') || ' 00:00:00+00');
      EXECUTE format('CREATE TRIGGER %I BEFORE UPDATE OR DELETE ON audit.%I FOR EACH STATEMENT EXECUTE FUNCTION util.fn_reject_modification()',
        'tg_' || p || '_' || to_char(m, '"y"YYYY"m"MM') || '_reject_ud', p || '_' || to_char(m, '"y"YYYY"m"MM'));
      EXECUTE format('CREATE TRIGGER %I BEFORE TRUNCATE ON audit.%I FOR EACH STATEMENT EXECUTE FUNCTION util.fn_reject_modification()',
        'tg_' || p || '_' || to_char(m, '"y"YYYY"m"MM') || '_reject_truncate', p || '_' || to_char(m, '"y"YYYY"m"MM'));
    END LOOP;
  END LOOP;
END $$;
```
Nama trigger terpanjang `tg_audit_event_subject_y2029m12_reject_truncate` = 47 byte, di bawah batas §1.2.

Alternatif yang ditolak: fungsi `SECURITY DEFINER` milik pemilik yang dipanggil job untuk membuat partisi. Ditolak karena menambah kemampuan DDL di runtime di luar DD §0.6 B dan ADR-021.

Index BRIN untuk `recorded_at` tidak dipakai di rilis 1; tetap index B-tree DD B1. Kemampuan BRIN belum ber-SRC dan dievaluasi di Tahap 8 bila volume menuntut `[DOK: DD B1]`.

### 6.3 Akses sempit kepala rantai (temuan F-02)

`[KEPUTUSAN-102]` `[DOK: ADR-009 butir 4]`:
```sql
CREATE FUNCTION audit.fn_chain_head()
RETURNS TABLE (seq bigint, event_hash bytea, recorded_at timestamptz)
LANGUAGE sql
STABLE
SECURITY DEFINER
SET search_path = pg_catalog, audit
AS $$
  SELECT e.seq, e.event_hash, e.recorded_at
    FROM audit.audit_event_key k
    JOIN audit.audit_event e ON e.event_id = k.event_id AND e.recorded_at = k.recorded_at
   ORDER BY k.seq DESC
   LIMIT 1
$$;
REVOKE ALL ON FUNCTION audit.fn_chain_head() FROM PUBLIC;
GRANT EXECUTE ON FUNCTION audit.fn_chain_head() TO rs_audit_writer;
```
Pemilik fungsi adalah role pemilik skema audit. Di `rme_audit`, kebijakan `pol_audit_event_migration` (§5.6) memastikan fungsi tetap melihat semua baris.

### 6.4 Jalur tulis relay

`[KEPUTUSAN-102]` `[KEPUTUSAN-103]`:
1. Relay memegang **kunci advisory tingkat sesi** di database aplikasi (`pg_try_advisory_lock`), yang dilepas saat sesi berakhir `[SRC-234]` `[KEPUTUSAN-74]`. Kunci dipegang di **koneksi khusus di luar pool**, dan batch outbox dibaca serta dihapus **lewat koneksi yang sama**. Bila koneksi putus, kunci lepas dan batch yang sedang berjalan ikut gagal, sehingga dua relay tidak memproses bersamaan.
2. Per batch, di database audit dalam satu transaksi: panggil `audit.fn_chain_head()` (rantai kosong: `seq = 0`, `prev_hash` = 32 byte nol); untuk setiap event urut `created_at, id` dari outbox, sisipkan `audit_event_key` dengan `seq = head + 1`. Bila `event_id` sudah ada (kiriman ulang setelah relay gagal di tengah), event dilewati tanpa menaikkan `seq` `[DOK: DD B2]`. Lalu sisipkan `audit_event` dan `audit_event_subject`, dan commit.
3. `recorded_at = greatest(now(), head.recorded_at)`. Dengan begitu `recorded_at` tidak pernah mundur terhadap `seq` walaupun jam server melangkah mundur, sehingga rentang `seq` per partisi tetap berurutan `[DOK: DD B]`. Selisih terhadap jam server tetap terpantau lewat `recorded_at − occurred_at` `[DOK: ADR-009 butir 6]`.
4. Setelah commit di database audit, hapus baris outbox yang terkirim di database aplikasi.
5. `UQ (seq)` di `audit_event_key` adalah pengaman terakhir: dua relay yang sempat berjalan bersamaan tidak dapat membuat cabang rantai, karena salah satunya gagal di constraint.

Registri kunci advisory (konstanta di crate `platform-jobs`, tidak pernah dipakai ulang) `[KEPUTUSAN-103]`:

| Kunci | Database | Pemegang |
| --- | --- | --- |
| 7001001 | `rumah_sehat` | relay audit Rumah Sehat |
| 7001002 | `rumah_sehat` | pengambil event Keycloak `[DOK: ADR-009 butir 2]` |
| 7001003 | `rumah_sehat_audit` | job verifikasi rantai Rumah Sehat |
| 7002001 | `rme` | relay audit RME |
| 7002003 | `rme_audit` | job verifikasi rantai RME |

### 6.5 Detach dan arsip (ADR-009 butir 9)

Jendela daring awal 24 bulan; tidak ada pemusnahan sebelum PT-24 `[DOK: ADR-009 butir 9]`. Prosedur `[KEPUTUSAN-101]`:

1. **Pemilihan**: partisi yang seluruh rentangnya lebih tua dari jendela daring.
2. **Verifikasi**: hasil `audit_chain_verification` terakhir `match` sampai `seq` akhir partisi.
3. **Ekspor** oleh alat arsip dengan role `*_audit_verifier` (hak baca seluruh rantai): baris partisi `audit_event` urut `seq`, partisi `audit_event_subject` bulan yang sama, dan baris `audit_event_key` rentang itu, ke berkas JSON Lines; hitung SHA-256 berkas; unggah ke bucket NOS dengan Object Lock mode Compliance `[KEPUTUSAN-78]`; lalu unduh ulang dan cocokkan hash.
4. **Detach + manifest** oleh pemilik skema audit, lewat langkah terkendali yang sama dengan migrasi production (kredensial dinamis OpenBao, bastion, orang kedua, tercatat di log audit OpenBao) `[KEPUTUSAN-81]` `[USER] T5-Q8`. Dalam satu transaksi: periksa ulang `min(seq)`, `max(seq)`, dan jumlah baris partisi terhadap nilai hasil ekspor; `ALTER TABLE audit.audit_event DETACH PARTITION audit.audit_event_y2025m03` `[SRC-39]` (dan pasangan `audit_event_subject`); `INSERT` baris `audit_archive_manifest` dengan `detached_at = now()`; `ALTER TABLE ... SET SCHEMA audit_detached`.
   Langkah ini **bukan berkas migrasi sqlx**, karena nilainya khusus production dan migrasi harus sama di semua lingkungan (§11). Penafsiran "langkah migrasi" ADR-021 ini dibawa ke §14 F-P07.
5. **Tabel yang sudah dilepas** berada di skema `audit_detached` tanpa hak untuk role mana pun selain pemilik, dan trigger penolaknya tetap ada **(V8: trigger ikut terbawa setelah detach)**. Tabel di-`DROP` hanya setelah uji pemulihan berkas arsip berhasil **dan** disetujui DBA + keamanan; langkah ini tercatat dan bukan pemusnahan arsip (arsip WORM tetap ada).
6. Baris `audit_event_key` untuk rentang yang diarsipkan tetap disimpan; pemangkasannya dievaluasi setelah PT-24 `[DOK: DD B2]`.

---

## 7. Format ciphertext, blind index, dan DEK terbungkus

Dasar: enkripsi aplikasi dengan DEK per wadah dan KEK di luar database `[DOK: ADR-013 §3]`; aws-lc-rs 1.19.0 untuk AES-256-GCM, HMAC-SHA256, dan RNG `[SRC-173]` `[KEPUTUSAN-74]`; KEK di OpenBao Transit `[SRC-222]` `[KEPUTUSAN-79]`. Parameter AES-GCM (panjang nonce 12 byte dan tag 16 byte) dan batas pemakaian nonce acak per kunci adalah nilai standar algoritma yang **belum terdaftar SRC** (NIST SP 800-38D tidak ada di register) **(V8)**, dan API aws-lc-rs yang dipakai juga **(V8)**.

### 7.1 Amplop ciphertext versi 1

`[KEPUTUSAN-104]`

| Offset (byte) | Panjang | Isi |
| --- | --- | --- |
| 0 | 1 | Versi format = `0x01` (AES-256-GCM, nonce 12 byte, tag 16 byte, AAD v1) |
| 1 | 12 | Nonce acak dari RNG aws-lc-rs, baru untuk setiap enkripsi |
| 13 | *n* | Ciphertext (panjang = plaintext) |
| 13 + *n* | 16 | Tag autentikasi GCM |

Panjang kolom `bytea` = *n* + 29 byte. Plaintext kosong tetap 29 byte, sehingga CHECK minimal 29 byte dan byte pertama `0x01` (§4.1 butir 5) selalu terpenuhi oleh ciphertext yang sah. Versi baru (misalnya algoritma lain) memakai byte versi berbeda; CHECK format diperluas lewat migrasi *expand* sebelum versi baru ditulis.

**AAD (data terautentikasi tambahan) v1**, disusun berurutan, setiap bidang didahului panjang 2 byte *big-endian*:

| # | Bidang | Isi |
| --- | --- | --- |
| 1 | Label AAD | ASCII `rs.aad.v1` |
| 2 | Konteks enkripsi | Pengenal logis kolom dari registri di crate `crypto`, misalnya `chat_message.body_enc`. Bukan nama fisik dari katalog, sehingga penggantian nama tabel dengan *expand/contract* tidak merusak dekripsi |
| 3 | Sistem | ASCII `rs` atau `rme` |
| 4 | ID baris | 16 byte UUID baris pemilik kolom (diketahui sebelum `INSERT` karena UUIDv7 dibuat aplikasi, ADR-002) |
| 5 | `data_key_id` | 16 byte UUID DEK yang dipakai |

Akibatnya ciphertext yang dipindah ke baris, kolom, sistem, atau DEK lain gagal didekripsi (tag tidak cocok). Ini melengkapi FK komposit wadah DD §0.2 terhadap penyerang yang punya akses tulis database. Pemeriksaan `container_type` tetap di pustaka `[DOK: DD §0.2]`. Kolom yang diperbarui di tempat (misalnya `client_secret_enc` saat rotasi) dienkripsi ulang dengan nonce baru dan ID baris yang sama.

Contoh ukuran: NIK 16 digit ASCII → 45 byte; kode verifikasi resep → panjang kode + 29 byte. Batas panjang teks bebas (chat, tiket) ditetapkan validasi API sebelum enkripsi `[DOK: DD §0.2]`; batas ukuran `bytea` di PostgreSQL **(V8)**.

Satu DEK per wadah membuat jumlah enkripsi per kunci kecil (misalnya pesan chat satu sesi), jauh di bawah batas yang biasa disarankan untuk nonce acak 96 bit **(V8)**.

### 7.2 Blind index

`[KEPUTUSAN-105]` `[DOK: ADR-013 §4]` `[SRC-46]`:
- `nik_bidx = HMAC-SHA256(K_nik[v], "rs.nik.v1" ‖ 0x00 ‖ NIK_normal)`, disimpan **utuh 32 byte**; `NIK_normal` = 16 digit ASCII setelah membuang spasi dan tanda baca dan lolos validasi 16 digit `[DOK: ADR-013 §4]`. CHECK `ck_<tabel>_nik_bidx CHECK (octet_length(nik_bidx) = 32)` dan `ck_<tabel>_nik_bidx_key_version CHECK (nik_bidx_key_version >= 1)`.
- Kode verifikasi resep: `verification_code_bidx = HMAC-SHA256(K_rx[v], "rs.rxcode.v1" ‖ 0x00 ‖ kode_normal)` `[DOK: DD A9.8]`.
- **Kunci terpisah per tujuan dan per sistem**: `K_nik` Rumah Sehat, `K_rx` Rumah Sehat, `K_nik` RME (pasien dan tenaga kesehatan fasyankes). Blind index NIK tidak pernah dicocokkan lintas sistem, sehingga bocornya kunci satu sistem tidak membuka sistem lain.
- Kunci disimpan di OpenBao KV `[KEPUTUSAN-79]` sebagai peta `{versi: kunci}` dengan nomor versi eksplisit di isi rahasia, tidak bergantung pada fitur versi bawaan KV **(V8)**. Kolom `*_key_version` = nomor versi itu `[DOK: DD §0.2]`. Panjang kunci 32 byte acak.
- Rotasi kunci NIK mengikuti prosedur DD A2.8 (tulis ulang ketiga tabel dalam satu transaksi per akun) `[DOK: DD A2.8]`. Kode verifikasi resep yang sudah tercetak tetap dicari dengan versi kuncinya `[DOK: DD A9.8]`.

### 7.3 DEK terbungkus dari OpenBao Transit

`[KEPUTUSAN-106]`:
- Satu kunci Transit per sistem (`rs-dek-kek`, `rme-dek-kek`) berjenis AES-256 GCM (nama tipe kunci Transit **(V8)**). Hak `datakey` dan `decrypt` hanya untuk identitas proses yang membutuhkan; `rewrap` hanya untuk prosedur rotasi `[KEPUTUSAN-79]`.
- DEK baru: 256 bit dari endpoint `datakey` Transit, yang mengembalikan kunci acak yang terbungkus kunci bernama `[SRC-222]`; varian yang juga mengembalikan plaintext dipakai saat wadah dibuat **(V8: nama endpoint dan bentuk respons)**. Plaintext DEK hanya di memori proses, di-cache dengan batas waktu `[DOK: ADR-013 konsekuensi]` `[KEPUTUSAN-79]`.
- `data_key.wrapped_dek` menyimpan ciphertext Transit **apa adanya sebagai byte UTF-8** (opaque); aplikasi tidak mengurai isinya. Format string ciphertext Transit **(V8)**.
- `data_key.kek_ref` = `openbao-transit:<mount>:<nama kunci>:v<versi>`. Versi diambil dari respons Transit bila tersedia **(V8)**; bila tidak, cukup `<mount>:<nama kunci>` dan versi dibaca dari awalan ciphertext setelah formatnya terverifikasi.
- Rotasi KEK: `rotate` di Transit; DEK lama tetap terbuka selama versinya ≥ `min_decryption_version` `[SRC-222]`. Bila `min_decryption_version` hendak dinaikkan, baris dengan versi lama dibungkus ulang lewat `rewrap` oleh prosedur dua orang yang memakai kredensial dinamis `*_key_rotation` (§3.4, §4.10) dan dicatat di security log `[DOK: DD A19.7]`.

---

## 8. Nomor yang dibaca manusia

Dasar: `text` unik, format ditetapkan Tahap 5 `[KEPUTUSAN-57]` `[DOK: DD §0.2]`. Kebutuhan: dapat diketik dan dibacakan; tidak membocorkan volume transaksi; sulit ditebak (nomor resep dipasangkan dengan kode verifikasi di halaman publik, RX-01 `[DOK: ADR-012 butir 3]`; kode check-in dapat diketik petugas fasyankes, US-1304 AC2 `[DOK: 03-prd.md]`).

`[KEPUTUSAN-107]`:

| Kolom | Format | Contoh | Ruang acak |
| --- | --- | --- | --- |
| `patient_order.order_number` | `RS-YYMMDD-XXXXXX` | `RS-270301-7K3QXP` | 32⁶ ≈ 1,07 × 10⁹ per hari |
| `prescription.prescription_number` | `RX-YYMMDD-XXXXXX` | `RX-270301-M4D9TZ` | sama |
| `complaint_ticket.ticket_number` | `TK-YYMMDD-XXXXXX` | `TK-270301-2HQ8WN` | sama |
| `data_subject_request.request_number` | `DS-YYMMDD-XXXXXX` | `DS-270301-B7R1KC` | sama |
| `appointment.check_in_code` | `XXXXXXXX` (8 karakter, tanpa tanggal; ditampilkan `XXXX-XXXX`) | `7K3Q-XP2M` | 32⁸ ≈ 1,1 × 10¹² |

Ketentuan:
- **Alfabet 32 karakter**: `0123456789ABCDEFGHJKMNPQRSTVWXYZ` (tanpa I, L, O, U supaya tidak tertukar dengan 1 dan 0). Masukan manual dinormalisasi: huruf besar, buang `-` dan spasi, `O`→`0`, `I`/`L`→`1`.
- **Bagian acak dari RNG aws-lc-rs** `[SRC-173]`, dibangkitkan di aplikasi. Sequence database ditolak karena membocorkan volume dan dapat ditebak berurutan.
- `YYMMDD` = tanggal WIB saat entitas dibuat, mengikuti tampilan backoffice WIB `[DOK: ADR-017]` (AB-42). Tanggal pesanan bukan rahasia bagi pemegang nomornya.
- **Keunikan** dijaga `UQ` yang sudah ada di DD `[DOK: DD A8.1, A9.8, A10.1, A16.1, A17.1]`. Bila `INSERT` gagal karena pelanggaran unik pada kolom nomor, aplikasi membangkitkan ulang, paling banyak 5 kali. Perkiraan bentrokan untuk 10.000 pesanan per hari ≈ 10.000² / (2 × 32⁶) ≈ 0,05 kejadian per hari (estimasi penyusun), cukup ditangani pengulangan.
- CHECK format (sintaks ekspresi reguler PostgreSQL **(V8)**):
  ```sql
  CONSTRAINT ck_patient_order_order_number CHECK (order_number ~ '^RS-[0-9]{6}-[0-9A-HJKMNP-TV-Z]{6}$')
  CONSTRAINT ck_appointment_check_in_code CHECK (check_in_code ~ '^[0-9A-HJKMNP-TV-Z]{8}$')
  ```
- Format nomor resep dapat berubah bila regulasi resep elektronik menetapkan format lain `[TIDAK DIKETAHUI]` (L-05, US-309 AC5) `[DOK: DD A9.8]`; perubahan = CHECK baru lewat *expand/contract*.
- Di RME, `rme_visit.check_in_code` menyimpan kode dari Rumah Sehat apa adanya dengan keunikan per fasyankes `[DOK: DD C3.2]`; CHECK format yang sama ditambahkan supaya kode yang rusak dari integrasi ditolak.

---

## 9. Antrean job, outbox, dan index tambahan

### 9.1 Tabel antrean job (butir 10)

Tabel antrean sudah ada di DD: `job` `[DOK: DD A19.4]` dan `rme_job` `[DOK: DD C6.7]`, diambil dengan `SELECT ... FOR UPDATE SKIP LOCKED` `[SRC-36]` `[DOK: ADR-007 butir 3]`. Tidak ada tabel baru. Mekanisme penguncian job tunggal ditetapkan di KEPUTUSAN-74 (`pg_try_advisory_lock` tingkat sesi `[SRC-234]`) dan registrinya di §6.4.

Pola pengambilan dengan sewa `[KEPUTUSAN-108]`:
```sql
WITH next AS (
  SELECT id FROM app.job
   WHERE status = 'queued' AND run_at <= now()
   ORDER BY run_at
   LIMIT $1
   FOR UPDATE SKIP LOCKED
)
UPDATE app.job j
   SET status = 'running', locked_by = $2, locked_until = now() + $3::interval,
       attempt_count = j.attempt_count + 1
  FROM next
 WHERE j.id = next.id
RETURNING j.id, j.job_type, j.payload, j.attempt_count, j.max_attempts;
```
- Transaksi pengambilan di-commit segera; job dikerjakan di transaksi tersendiri, lalu `status` menjadi `succeeded`, atau kembali `queued` dengan `run_at` mundur bertahap, atau `failed` bila `attempt_count >= max_attempts`. Job idempoten `[DOK: ADR-007 konsekuensi]`.
- Penyapu periodik mengembalikan job `running` yang `locked_until < now()` (pekerja mati) ke `queued`.
- **RME**: `rme_worker` mengambil dari `rme_job` dengan kebijakan antrean lintas tenant, lalu memasang `set_config('rme.fasyankes_id', <fasyankes_id job>, true)` di transaksi pengerjaan `[DOK: DD §0.6 C]` `[KEPUTUSAN-55]`. Job tanpa tenant (pemuatan terminologi) tidak memasang konteks dan hanya menyentuh `terminology_code`.

Outbox (`integration_outbox`, `notification_outbox`, `rme_integration_outbox`) memakai pola sewa yang sama dengan `status` dan `available_at`. Outbox audit dibaca relay urut `created_at, id` **tanpa kursor**: baris yang terkirim dihapus, sehingga transaksi yang commit terlambat dengan `created_at` lebih awal tetap terambil di batch berikutnya, tidak terlewat `[KEPUTUSAN-108]`.

### 9.2 Index yang tidak ditambah

- Proyeksi status: index DD (`ix (status, updated_at)` dan partial index status belum akhir) sudah mencakup jaring pengaman harian `[DOK: DD A8.3, §D.7]`.
- Pencarian katalog `pg_trgm`: index trigram partial sudah ada di DD `[DOK: DD §0.2]`.
- `integration_outbox` dan `notification_outbox`: `ix (status, available_at)` DD cukup, karena baris selesai dihapus `[DOK: DD A19.2, A19.3]`.

### 9.3 Index tambahan (usulan, divalidasi uji beban NFR-CAP-04)

`[KEPUTUSAN-109]`

| Index usulan | Alasan |
| --- | --- |
| `ix_job_locked_until_running ON app.job (locked_until) WHERE status = 'running'` dan padanannya di `rme_job` | Penyapu sewa kedaluwarsa (§9.1) |
| `ix_upload_slot_expires_at ON app.upload_slot (expires_at)` dan `ix_rme_upload_slot_expires_at` | Pembersihan DD §D.2 menyebut tabel ini, tetapi DD A19.5 belum punya index `expires_at` |
| `ix_rme_inbound_event_expires_at ON app.rme_inbound_event (expires_at)` | Sama, DD C6.8 |
| `ix_audit_outbox_created_at_id ON app.audit_outbox (created_at, id)` menggantikan `ix (created_at)` DD; padanan di `rme_audit_outbox` | Urutan baca relay yang deterministik |

`external_access_token` tidak diberi index `expires_at`, karena barisnya paling banyak dua per fasyankes (`UQ (fasyankes_id, target)`) `[DOK: DD C1.5]`.

---

## 10. Pengaturan server dan koneksi

Semua nilai di bagian ini adalah **nilai awal penyusun** `[KEPUTUSAN-139]`, divalidasi uji beban NFR-CAP-04 di staging yang dinaikkan ke ukuran production `[KEPUTUSAN-78]`. Nama parameter server dan opsi pool sqlx **(V8)**.

### 10.1 Server (per instans)

| Pengaturan | (a) `db-app` 8 vCPU/16 GB | (b) `db-audit` 2 vCPU/4 GB | (c) `db-rme` 4 vCPU/4 GB | Dasar |
| --- | --- | --- | --- | --- |
| `max_connections` | 200 | 80 | 120 | Jumlah pool §10.3 + cadangan admin |
| `shared_buffers` | 4 GB | 1 GB | 1 GB | Estimasi penyusun ±25% RAM **(V8: panduan dokumentasi)** |
| `TimeZone` (server dan `ALTER DATABASE`) | `UTC` | `UTC` | `UTC` | Server aplikasi UTC `[DOK: ADR-017 butir 1]`; `timestamptz` tersimpan UTC `[SRC-41]` |
| TLS | `ssl = on`, versi minimum TLS 1.3 | sama | sama | Semua lalu lintas internal TLS `[DOK: ADR-013 butir 1]`; kedua ujung milik kita sehingga TLS 1.2 tidak perlu |
| Autentikasi | `pg_hba`: hanya `hostssl` dengan `scram-sha-256`, per database dan per grup, dari subnet privat dan jaringan WireGuard; selain itu `reject` | sama | sama | `[KEPUTUSAN-78]` (firewall privat) |
| `archive_mode`, `archive_timeout` | aktif, 60 detik, `archive_command` pgBackRest | sama | sama | `archive_timeout` sekitar satu menit `[SRC-38]`; pgBackRest `[SRC-235]` `[KEPUTUSAN-78]` |
| Log | `log_connections`, `log_disconnections` aktif; `log_statement = none`; `log_min_duration_statement = 1s`; nilai parameter tidak dicatat (panjang log parameter = 0, juga saat galat) | sama | sama | Log PostgreSQL masuk security/application log tanpa data pribadi `[DOK: ADR-010]` |
| `log_line_prefix` | memuat waktu, PID, `user`, `database`, `application_name` | sama | sama | Atribusi per role login (§3.1) |

Standby streaming di Hostinger, role replikasi, dan role koneksi pgBackRest ditetapkan Tahap 8 Fase 1 **(V8)**; replikasi berjalan lewat WireGuard + TLS `[KEPUTUSAN-78]`. Tanpa PgBouncer di rilis 1: jumlah koneksi kecil, dan kunci advisory tingkat sesi tidak cocok dengan pooling per transaksi `[KEPUTUSAN-139]`.

### 10.2 Batas waktu per role login (`ALTER ROLE ... IN DATABASE ... SET`)

| Role login | `statement_timeout` | `lock_timeout` | `idle_in_transaction_session_timeout` | Alasan |
| --- | --- | --- | --- | --- |
| `rs_api`, `rs_realtime`, `rme_api`, `rme_api_auth`, `rme_api_operator` | 3 s | 2 s | 30 s | p95 baca 500 ms, tulis 1 s (NFR-PERF-01/02 `[DOK: 03-prd.md]`); di atas 3 s permintaan sudah gagal secara layanan |
| `rs_worker`, `rme_worker_login` | 60 s | 5 s | 60 s | Job batch |
| `rs_privacy_worker_login`, `rme_privacy_worker_login` | 300 s | 5 s | 60 s | Retensi dan anonimisasi per batch |
| `*_audit_relay_login`, `*_audit_writer_login` | 30 s | 5 s | 30 s | Batch relay |
| `rs_audit_reader_login`, `rme_audit_tenant_reader_login` | 30 s | 2 s | 30 s | Halaman Audit Log; export Auditor di job tersendiri |
| `*_audit_verifier_login` | 0 (tanpa batas) | 5 s | 60 s | Verifikasi seluruh rantai harian |
| Pemilik skema (migrasi) | 15 min (dipasang di penjaga migrasi §11.2) | 5 s | 60 s | Migrasi gagal cepat daripada mengantre di belakang kunci tabel |
| `keycloak` | bawaan | bawaan | 60 s | Dikelola Keycloak |

### 10.3 Pool sqlx per proses

| Proses | Role login | Instans | Pool maks per instans | Total |
| --- | --- | --- | --- | --- |
| api | `rs_api` | 2 | 20 | 40 |
| realtime | `rs_realtime` | 2 | 10 | 20 |
| worker | `rs_worker` | 2 | 10 | 20 |
| pekerja privasi | `rs_privacy_worker_login` | 1 | 4 | 4 |
| audit-relay (1 aktif) | `rs_audit_relay_login` | 2 | 3 (1 koneksi khusus kunci + 2) | 6 |
| Keycloak | `keycloak` | 2 | 20 (opsi pool Keycloak **(V8)**) | 40 |
| agen metrik | `rs_metrics` | 1 | 2 | 2 |
| **Total (a)** | | | | **132** |
| rme api | `rme_api` / `rme_api_auth` / `rme_api_operator` | 2 | 15 / 3 / 2 | 40 |
| rme worker | `rme_worker_login` | 2 | 8 | 16 |
| relay audit RME, pekerja privasi RME, metrik | | | | 12 |
| **Total (c)** | | | | **68** |
| Instans (b): writer, reader, tenant reader, verifier kedua sistem | | | | **±40** |

Ketentuan koneksi `[KEPUTUSAN-139]`:
- URL koneksi memakai `sslmode=verify-full` dengan sertifikat CA internal; perangkat CA internal ditetapkan Tahap 8 Fase 1 **(V8)**.
- `application_name` = nama proses dan versi.
- Pool: `min_connections` 2, `acquire_timeout` 3 s untuk api (5 s lainnya), `idle_timeout` 10 menit, `max_lifetime` 30 menit, sehingga password yang dirotasi berlaku tanpa restart penuh.
- Saat koneksi dibuka, aplikasi memeriksa `current_setting('TimeZone') = 'UTC'` dan `current_user` sesuai harapan, lalu menolak start bila berbeda. Tidak ada pengaturan sesi lain yang dipasang di tingkat koneksi; konteks tenant RME hanya per transaksi (§5.1).
- Batas waktu operasi sqlx di sisi klien dibuat sedikit di atas `statement_timeout` role, sehingga galat yang terlihat berasal dari server dengan kode yang jelas.

---

## 11. Aturan migrasi

### 11.1 Alat dan direktori

- **sqlx-cli 0.9.1** `[SRC-182]`, SQL murni, maju saja (*forward-only*), pola *expand/contract* `[KEPUTUSAN-74]` `[DOK: ADR-019]`. Perintah dan opsi CLI (misalnya sumber direktori dan URL database) **(V8)**.
- **Satu direktori per skema database** `[KEPUTUSAN-89]`: `db/migrations/app/` → `rumah_sehat`; `db/migrations/rme/` → `rme`; `db/migrations/audit/` → **kedua** database audit (skema sama, DD B). Perbedaan antar-database audit (awalan nama role, RLS tenant hanya di `rme_audit`) ditangani di SQL lewat fungsi pemetaan `util.fn_audit_role(<nama logis>)` yang mengembalikan `rs_<nama>` atau `rme_<nama>` berdasarkan `current_database()`, dan blok RLS yang hanya berjalan bila `current_database() = 'rme_audit'` `[KEPUTUSAN-140]`. Satu sumber mencegah kedua skema audit menyimpang; test menjalankan direktori ini ke kedua database dan memeriksa hak masing-masing.
- Nama berkas: awalan versi sqlx + deskripsi snake_case `[DOK: ADR-001]` (bentuk awalan **(V8)**).
- Berkas yang sudah diterapkan di lingkungan mana pun **tidak pernah diubah**; koreksi = berkas baru. Pemeriksaan checksum sqlx **(V8)**.
- Setiap perubahan skema diikuti `cargo sqlx prepare` dan metadata `.sqlx` di-commit; CI menjalankan `cargo sqlx prepare --check` `[SRC-166]` `[KEPUTUSAN-74]`.

### 11.2 Penjaga di awal setiap berkas

`[KEPUTUSAN-140]`:
```sql
DO $$
BEGIN
  IF current_user <> 'rs_migration' THEN
    RAISE EXCEPTION 'migration must run as rs_migration, got % (session %)', current_user, session_user;
  END IF;
END $$;
SET LOCAL lock_timeout = '5s';
SET LOCAL statement_timeout = '15min';
```
`SET LOCAL` bergantung pada sqlx menjalankan setiap berkas dalam transaksi **(V8)**. Direktori `audit/` memeriksa `current_user` terhadap `util.fn_audit_role('audit_migration')`.

### 11.3 Expand/contract dan migrasi data

- Deploy bergulir menjalankan versi N dan N+1 bersamaan; migrasi diterapkan **sebelum** deploy, dan setiap migrasi harus kompatibel dengan kode versi sebelumnya `[DOK: ADR-019]`. Urutan: *expand* (tambah kolom/tabel/index, kolom baru boleh `NULL` atau bernilai bawaan konstan) → deploy kode yang menulis keduanya → migrasi data → deploy kode yang hanya membaca bentuk baru → *contract* (hapus yang lama) di rilis berikutnya.
- **Migrasi data terpisah dari migrasi skema**: satu berkas hanya berisi DDL atau hanya DML. Migrasi data idempoten. Pengisian ulang tabel besar dikerjakan sebagai job bertahap (§9.1), bukan dalam satu migrasi.
- Index pada tabel yang sudah besar dibuat dengan `CREATE INDEX CONCURRENTLY`, yang tidak dapat berjalan di dalam transaksi **(V8)**; dukungan sqlx untuk berkas tanpa transaksi **(V8)**. Sampai keduanya terverifikasi, index baru pada tabel besar dibuat di jendela pemeliharaan NFR-AVL-03 `[DOK: 03-prd.md]`. Saat go-live semua tabel masih kecil.
- `NOT NULL` baru pada tabel besar: tambah sebagai CHECK `NOT VALID`, validasi terpisah, baru ubah ke `NOT NULL` **(V8)**.

### 11.4 Tabel immutable dan migrasi

- Tabel T, R, dan L **tidak dapat di-backfill** karena trigger penolak berlaku juga bagi pemilik (§4.5). Kolom baru di tabel ini harus `NULL` atau bernilai bawaan konstan, dan nilai lama dibiarkan `NULL` `[DOK: ADR-004]`.
- Pengecualian (misalnya perbaikan data akibat bug yang disetujui) memerlukan `ALTER TABLE ... DISABLE TRIGGER` dalam transaksi migrasi yang sama lalu `ENABLE` kembali. Langkah ini adalah perubahan T1 dengan review DBA + keamanan, dicatat di decision-log, dan **dilarang di database audit** `[DOK: ADR-009 butir 4]` `[KEPUTUSAN-140]`.

### 11.5 Review dan bukti

- Migrasi yang menyentuh database audit, role, grant, trigger penolak, atau kebijakan RLS wajib direview DBA dan keamanan `[DOK: ADR-021 butir 4]` `[KEPUTUSAN-81]`. Semua migrasi termasuk modul kritis T1 "file migrasi skema dan hak role database" `[DOK: CLAUDE.md]`.
- `verify.sh` langkah 2 menerapkan semua migrasi ke PostgreSQL 18 kosong (testcontainers) untuk `rumah_sehat`, `rumah_sehat_audit`, `rme_audit`, dan `rme`, lalu menjalankan test katalog §12 dan test RLS §5.5 `[KEPUTUSAN-82]`.
- Production: skrip deploy di bastion menjalankan migrasi dengan kredensial pemilik dinamis sebelum deploy bergulir `[KEPUTUSAN-81]` `[USER] T5-Q8`.

---

## 12. Test katalog wajib (`verify.sh` langkah 2)

Ditambah test RLS T-10–T-17 (§5.5). Semua dijalankan dengan role sungguhan, bukan superuser.

| ID | Pemeriksaan | Dasar |
| --- | --- | --- |
| T-01 | Nama objek: prefiks §1.1, panjang ≤ 60 byte, snake_case | `[KEPUTUSAN-90]`, `[DOK: ADR-001]` |
| T-02 | Tidak ada objek di luar skema terdaftar (§2.1); skema `public` tidak ada | `[KEPUTUSAN-91]` |
| T-03 | `PUBLIC` tidak punya `CONNECT`/`TEMP` di database proyek dan tidak punya `EXECUTE` atas fungsi proyek | `[KEPUTUSAN-91]` |
| T-04 | Semua tabel, fungsi, dan tabel catatan migrasi dimiliki role pemilik skema | `[KEPUTUSAN-97]` |
| T-05 | Atribut role (§3.1): tanpa `SUPERUSER`/`BYPASSRLS`; grup `NOLOGIN`; keanggotaan yang saling meniadakan tidak ada | `[DOK: ADR-012 butir 8]` |
| T-06 | Setiap tabel punya komentar `class:`; hak tabel per grup sama persis dengan aturan kelas DD §0.6 (tidak kurang, tidak lebih) | `[KEPUTUSAN-94]` |
| T-07 | Hak `UPDATE` per kolom `*_privacy_worker` = kolom bertanda `anonymize` + `anonymized_at`, `updated_at`, `updated_by` (+ kolom soft delete untuk `account_owner_nik_registry`) | `[DOK: DD §0.2, §0.6]` |
| T-08 | Trigger sesuai §4.11 untuk setiap kelas; setiap tabel L punya argumen kategori | `[DOK: DD §0.3]` |
| T-09 | Perilaku: `rs_app` gagal `UPDATE`/`DELETE`/`TRUNCATE` di T, R, L; pemilik pun gagal (trigger); `rs_privacy_worker` gagal `DELETE` Log selama `retention_policy` kosong dan berhasil setelah baris kebijakan uji dengan masa habis; `data_key` yang sudah dimusnahkan tidak dapat diubah siapa pun; `rs_app` gagal `SELECT` di `audit_outbox` | `[DOK: ADR-008]`, `[DOK: ADR-013 §5]`, `[KEPUTUSAN-66]` |
| T-18 | Setiap partisi audit punya trigger penolak dan semua index induk; ada ≥ 6 partisi masa depan | `[KEPUTUSAN-100]` |
| T-19 | `rs_audit_writer` hanya dapat `INSERT` dan memanggil `fn_chain_head`; `SELECT` langsung ditolak | `[DOK: ADR-009 butir 4]` |
| T-20 | Kolom `*_enc` menolak teks terbuka (CHECK format); kolom `*_bidx` menolak panjang selain 32 byte | `[KEPUTUSAN-104]`, `[KEPUTUSAN-105]` |
| T-21 | Database `keycloak` tidak dapat dimasuki role aplikasi, dan `keycloak` tidak dapat memasuki `rumah_sehat` | `[KEPUTUSAN-91]` |

---

## 13. Butir DD §D.4 yang masih terbuka

DD §D.4 menjadwalkan verifikasi butir berikut ke dokumentasi PostgreSQL di Tahap 5 `[DOK: DD §D]`. Register sumber Tahap 5 (SRC-84–SRC-247) tidak memuat halaman untuk butir ini, dan dokumen ini tidak boleh menambah SRC. Karena itu semuanya **dipindah ke Tahap 8 Fase 1** (§14 F-P05):

| Butir DD §D.4 | Rancangan yang diusulkan untuk diverifikasi | Status |
| --- | --- | --- |
| (a) Keseimbangan jurnal, jumlah instruksi pembagian dana, jumlah baris harga terkunci | *Constraint trigger* `DEFERRABLE INITIALLY DEFERRED` yang memeriksa jumlah saat commit **(V8)** | Terbuka, sampai itu dijaga pustaka + test |
| (b) Tumpang tindih slot (A4.8) | Exclusion constraint dengan `tstzrange` dan ekstensi `btree_gist` **(V8)**; menambah ekstensi kedua (§2.2) | Terbuka |
| (c) RLS per role, termasuk tabel audit terpartisi | §5.3, §5.6, test T-11–T-17 | Rancangan ada, (V8) |
| (d) Constraint unik di tabel terpartisi | Sudah terverifikasi `[SRC-39]` | Selesai (DD) |
| (e) FK komposit dengan kolom `NULL`, FK terhadap RLS, FK pada kolom terhitung (A8.2) | Untuk `order_status_event`: kolom terhitung tersimpan `coalesce(from_status, '_start')` sebagai bagian FK ke tabel transisi **(V8)** | Terbuka, sampai itu dijaga pustaka transisi `[DOK: DD A8.2]` |
| (f) BRIN tabel audit | Tidak dipakai di rilis 1 (§6.2) | Ditunda |

Juga belum ber-SRC dan dipakai rancangan ini: batas identifier 63 byte, `set_config`/`current_setting`, `CREATE POLICY`, `pg_has_role`, trigger pada tabel terpartisi, `ALTER ROLE ... SET role`, `ALTER DEFAULT PRIVILEGES`, `CREATE INDEX CONCURRENTLY`, parameter server §10, opsi pool sqlx, API aws-lc-rs, dan respons Transit `datakey`. Semua bertanda **(V8)** di tempatnya.

---

## 14. Temuan dan kontradiksi untuk orkestrator

Tidak ada yang diperbaiki diam-diam di DD (DD terkunci).

| ID | Temuan | Dampak | Usulan |
| --- | --- | --- | --- |
| F-P01 | DD §0.6 B memakai nama role yang sama untuk database audit Rumah Sehat dan RME, padahal keduanya berbagi instans (b) dan role berlaku untuk seluruh klaster **(V8)** | Nama logis DD tidak dapat dipakai apa adanya | Pemetaan nama fisik berawalan §3.2; DD tidak perlu diubah bila tabel pemetaan ini diterima |
| F-P02 | Mengaktifkan RLS di `audit_event`/`audit_event_subject` RME berlaku untuk semua role, sedangkan DD §0.6 B hanya menyebut kebijakan `audit_tenant_reader` | Tanpa kebijakan tambahan, relay, pembaca, verifikasi, dan pemilik ditolak | Kebijakan semua role §5.6 |
| F-P03 | `audit_event_subject` tidak punya `tenant_id` (DD B3), sehingga RLS tenant memerlukan subquery ke `audit_event` | Biaya query dan ketergantungan pada perilaku RLS bertingkat **(V8)** | Pakai subquery (§5.6); alternatif menambah `tenant_id` ke B3 lewat CR bila T-17 menunjukkan masalah |
| F-P04 | `data_key.retention_anchor_at` (dan `retention_category`) boleh `NULL` di DD A19.7, tetapi tidak ada role yang boleh mengubahnya setelah `INSERT` (`rs_app` S, I) | Wadah yang tanggal acuannya baru diketahui kemudian (misalnya akhir sesi) tidak pernah dapat diproses retensi | Putuskan: (a) tanggal acuan dihitung dari entitas wadah saat job retensi, atau (b) hak `UPDATE` sekali-isi (`NULL`→nilai) untuk `rs_app` pada dua kolom itu, ditegakkan trigger satu arah. Trigger §4.10 saat ini menolak perubahan keduanya |
| F-P05 | DD §D.4 menjadwalkan verifikasi PostgreSQL di Tahap 5, tetapi SRC-84–SRC-247 tidak memuat halaman itu | Butir (a), (b), (e) masih terbuka; (b) mungkin menambah ekstensi `btree_gist` | Pindah ke Tahap 8 Fase 1 (§13); orkestrator dapat menambah SRC bila ingin menutupnya di Tahap 5 |
| F-P06 | FK proyeksi → riwayat (§4.7) dan CHECK format amplop/blind index (§4.1, §7) menambah constraint di luar DD | Perubahan skema logis kecil | Review DBA; bila diterima, cukup dicatat di sini tanpa mengubah DD |
| F-P07 | ADR-021 butir 4: kredensial pemilik skema "hanya pada langkah migrasi". Detach arsip audit (DD §0.6 B: oleh `audit_migration`) tidak dapat menjadi berkas migrasi sqlx karena nilainya khusus production | Penafsiran ADR-021 | Detach berjalan di langkah terkendali yang sama dengan migrasi production (§6.5); konfirmasi penafsiran di gate |
| F-P08 | KEPUTUSAN-89: `db/roles/` "dijalankan pemilik skema". `CREATE ROLE` memerlukan `CREATEROLE`, yang tidak dimiliki pemilik skema | Isi `db/roles/` tidak dapat dijalankan pemilik | `db/roles/` = bootstrap klaster oleh admin; hak per tabel di berkas migrasi (§3.5) |
| F-P09 | KEPUTUSAN-74: binary `rme-backend` hanya `api|worker`, padahal ADR-009 butir 3 dan M-23 mewajibkan relay audit sebagai deployment terpisah yang hanya memegang kredensial audit; tempat job verifikasi rantai juga belum disebut | RME tanpa proses relay yang memenuhi ADR-009 | Tambah subcommand `audit-relay` di `rme-backend` (atau pakai crate `audit` bersama dalam binary relay terpisah); job verifikasi dijalankan di deployment relay masing-masing `[KEPUTUSAN-141]` |
| F-P10 | KEPUTUSAN-79 menyebut identitas "pekerja privasi" tersendiri, tetapi daftar subcommand KEPUTUSAN-74 tidak memuatnya | Belum jelas proses mana yang memegang kredensial `*_privacy_worker` | Tetapkan di Tahap 8: subcommand tersendiri lebih sesuai prinsip satu deployment satu kredensial berhak khusus |
| F-P11 | DD A19.7 menyebut KEK "di KMS" dan rotasi "KMS penyedia"; kini OpenBao Transit (CR-012, ADR-025) | Rujukan istilah | Ditangani ADR-025; format `kek_ref` §7.3 |
| F-P12 | M-24 (batas hak Operator RME di database) masih terbuka; kebijakan RLS yang membaca tabelnya sendiri berisiko rekursi **(V8)** | RME-04 tetap "Sebagian" | Rancang bersama M-24 di Tahap 8 (§5.3) |

---

## 15. Keputusan fisik baru

Dicatat orkestrator di `docs/decision-log.md`. Semua bertanda (V8) di badan dokumen memerlukan bukti test di Tahap 8 Fase 1.

| ID | Keputusan | Alternatif | Alasan |
| --- | --- | --- | --- |
| KEPUTUSAN-90 | Pola nama `pk_`, `fk_`, `uq_`, `uqp_`, `ix_`, `ck_`, `ex_`, `tg_`, `fn_`, `pol_`, partisi `_yYYYYmMM`; batas proyek 60 byte dengan alias tabel; test katalog menolak ≥ 61 byte | Nama bawaan PostgreSQL; batas 63 byte | Nama dapat dibaca di galat dan log; pemotongan diam-diam tertangkap |
| KEPUTUSAN-91 | Database `rumah_sehat`, `keycloak`, `rumah_sehat_audit`, `rme_audit`, `rme`; satu skema bisnis (`app`/`audit`) + `util` + `ext`; `public` dihapus; database milik superuser bootstrap; `CONNECT`/`TEMP`/`EXECUTE` dicabut dari `PUBLIC`; `search_path` per role | Skema per modul; skema `public` | FK lintas modul banyak; permukaan serangan kecil; role migrasi tidak dapat menghapus database |
| KEPUTUSAN-92 | Ekstensi hanya `pg_trgm` di skema `ext`, dibuat saat bootstrap; tanpa `pgcrypto`; `uuidv7()` tidak dipakai sebagai nilai bawaan | Ekstensi dibuat migrasi; `pgcrypto` | Role migrasi bukan superuser; enkripsi di aplikasi (ADR-013) |
| KEPUTUSAN-93 | `id` tanpa nilai bawaan; `created_at DEFAULT now()`; `updated_at` diisi pernyataan; data acuan memakai UUID literal; FK tanpa `CASCADE` | Default `uuidv7()`; trigger penyentuh | ADR-002; bug ID kosong terlihat; ID acuan sama di semua lingkungan |
| KEPUTUSAN-94 | Kelas, modul, alias, kategori log di komentar tabel; kolom "Anonimkan" di komentar kolom; test katalog membandingkan hak/trigger dengan kelas | Berkas manifest terpisah; checklist manual | Aturan DD §0.6 diperiksa mesin; penanda ikut migrasi |
| KEPUTUSAN-95 | Trigger penolak T/R tingkat statement (UPDATE OR DELETE dan TRUNCATE terpisah); L: tolak UPDATE/TRUNCATE + penjaga DELETE tingkat baris dengan argumen kategori dan kolom acuan; fungsi `SECURITY INVOKER`, `search_path` tetap, kode pesan tetap | Trigger tingkat baris; satu fungsi per tabel | Statement kosong pun tertolak dan terlihat; retensi dinilai per baris; satu pustaka |
| KEPUTUSAN-96 | Trigger satu arah `data_key`: hanya kolom kunci yang boleh berubah, cabang per role, `shredded_at := now()`; `*_key_rotation` tanpa login tetap | Role rotasi berlogin tetap | DEK tidak hidup kembali; waktu pemusnahan dari server; hak rotasi hanya saat prosedur |
| KEPUTUSAN-97 | Nama role fisik berawalan sistem; grup `NOLOGIN` + login per proses; atribut minimum; password app di KV; pemilik skema lewat kredensial dinamis OpenBao `IN ROLE` + `SET role`, penjaga `current_user`; bootstrap role oleh admin | Satu role login per kelas; pemilik berlogin tetap | Role berlaku per klaster; atribusi per proses; objek tidak dimiliki role dinamis; ADR-021 |
| KEPUTUSAN-98 | RLS RME: `set_config('rme.fasyankes_id', $1, true)` per transaksi lewat `with_tenant_tx`; fungsi konteks `NULLIF` gagal-tertutup; kebijakan permisif per role; kebijakan eksplisit untuk pemilik; larangan konteks tingkat sesi diuji | `SET` tingkat sesi; `BYPASSRLS` untuk role lintas tenant | RME-02, H-12; pemilik tunduk `FORCE` [SRC-34] |
| KEPUTUSAN-99 | RLS audit RME di tabel induk untuk semua role; pembaca tenant hanya E-05 fasyankesnya; hak hanya di induk; cadangan fungsi baca per tenant | RLS hanya untuk pembaca tenant | RLS aktif berlaku untuk semua role; ADR-009 butir 8, ADR-012 butir 9 |
| KEPUTUSAN-100 | Partisi audit bulanan UTC dibuat migrasi sampai 36+ bulan ke depan dengan tanggal literal; job hanya memantau (peringatan < 6 bulan); tanpa partisi default; trigger di setiap partisi | Job membuat partisi lewat fungsi `SECURITY DEFINER`; partisi default | Tanpa DDL runtime (ADR-021, DD §0.6 B); kegagalan terlihat dan event tertahan di outbox |
| KEPUTUSAN-101 | Arsip: ekspor oleh role verifikasi ke WORM dengan hash, lalu detach + manifest oleh pemilik di langkah terkendali production; tabel lepas ke `audit_detached` dan di-drop hanya setelah uji pulih + persetujuan | Detach dalam berkas migrasi; drop segera | Migrasi harus sama di semua lingkungan; jejak dan kemampuan pulih (AUD-03) |
| KEPUTUSAN-102 | `audit.fn_chain_head()` `SECURITY DEFINER` untuk baca sempit kepala rantai; `seq` dihitung relay tanpa sequence; genesis `prev_hash` 32 byte nol; `recorded_at = greatest(now(), head.recorded_at)`; `UQ (seq)` sebagai penjaga cabang | View; sequence database | F-02; `seq` tanpa celah; rentang per partisi tetap urut |
| KEPUTUSAN-103 | Registri kunci advisory (7001xxx RS, 7002xxx RME); kunci dipegang koneksi khusus yang juga memproses batch | Kunci dari pool; kunci tingkat transaksi | Koneksi putus = kunci lepas dan batch gagal bersamaan |
| KEPUTUSAN-104 | Amplop v1: versi 1 byte + nonce 12 + ciphertext + tag 16; AAD berpanjang-prefiks berisi label, konteks logis, sistem, ID baris, `data_key_id`; CHECK panjang ≥ 29 dan byte pertama `0x01` | Tanpa AAD; nama tabel fisik di AAD; tanpa CHECK | Mencegah pemindahan ciphertext; tahan ganti nama tabel; teks terbuka tertolak |
| KEPUTUSAN-105 | Blind index HMAC-SHA256 32 byte utuh, label tujuan, kunci terpisah per tujuan dan per sistem, versi kunci eksplisit di KV | Dipotong; satu kunci bersama | Kebocoran satu kunci tidak menjalar; rotasi terukur |
| KEPUTUSAN-106 | DEK 256 bit dari Transit `datakey`; `wrapped_dek` = ciphertext Transit apa adanya; `kek_ref` `openbao-transit:<mount>:<kunci>:v<versi>`; satu kunci Transit per sistem; bungkus ulang hanya lewat prosedur `*_key_rotation` | Mengurai format ciphertext; satu kunci untuk dua sistem | Tidak bergantung format internal; isolasi RS/RME |
| KEPUTUSAN-107 | Nomor `RS-/RX-/TK-/DS-YYMMDD-XXXXXX` dan kode check-in 8 karakter, alfabet 32 karakter tanpa I/L/O/U, acak dari RNG aws-lc-rs, ulang saat bentrok, CHECK regex | Sequence; UUID dipendekkan | Tidak membocorkan volume; dapat diketik; RX-01 |
| KEPUTUSAN-108 | Pengambilan job dengan sewa (`running` + `locked_until`, `SKIP LOCKED`), penyapu sewa kedaluwarsa; outbox audit dibaca tanpa kursor | Menahan kunci baris selama pengerjaan | Transaksi pendek; tahan pekerja mati; tidak ada event terlewat |
| KEPUTUSAN-109 | Index usulan §9.3 dan pengaturan autovacuum tabel O; `fillfactor` tidak diturunkan | Tanpa index pembersihan | Pembersihan DD §D.2 tanpa pemindaian penuh; divalidasi uji beban |
| KEPUTUSAN-139 | Nilai awal server, batas waktu per role, dan pool §10; TLS 1.3 `verify-full`; `scram-sha-256` `hostssl` saja; UTC; log tanpa nilai parameter; tanpa PgBouncer | TLS 1.2; PgBouncer transaksi | ADR-013 butir 1, ADR-010, NFR-PERF; kunci advisory sesi |
| KEPUTUSAN-140 | Aturan migrasi §11: satu direktori per skema (audit bersama dengan pemetaan role), penjaga `current_user` + `lock_timeout`, DDL dan DML terpisah, T/R/L tanpa backfill kecuali `DISABLE TRIGGER` T1 (dilarang di audit) | Direktori audit terpisah per database | Skema audit tidak menyimpang; migrasi gagal cepat; ADR-004 |
| KEPUTUSAN-141 | Job verifikasi rantai dijalankan di deployment relay audit masing-masing sistem; RME memerlukan proses relay tersendiri | Verifikasi di worker bisnis | M-23: kredensial audit hanya di deployment relay |
| KEPUTUSAN-142 | FK komposit proyeksi `(<entitas>_id, version, status)` → riwayat `UQ (<entitas>_id, version, to_status)` (usulan, review DBA) | Hanya pembandingan harian | Proyeksi tidak dapat menunjuk status yang tidak pernah tercatat (memperkuat ADR-008 butir 4) |
