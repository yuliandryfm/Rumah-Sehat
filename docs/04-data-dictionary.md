# Data Dictionary: Ruang Sehat

Tanggal: 2026-11-18
Versi: 1.5 (2026-11-22, F1-T104: A6.7 `reason_code` menambah konteks `corporate_rejection` dengan daftar awal draf; catatan A20 tentang jalur status keputusan Verifikator dan bucket dokumen perusahaan; migrasi `20261124200000_platform_reason_code_corporate_rejection.sql` dan `20261124201000_corporate_rejection_reason_seed.sql`). Versi 1.4 (2026-11-21, F1-T62, `[KEPUTUSAN-294]` = jawaban user PT-57/T-38 opsi A: A1.1 `account` menambah `staff_nik_bidx` dan `staff_nik_bidx_key_version`, blind index NIK staf untuk pemisahan tugas Verifikator US-201 AC8; migrasi `20261123310000_identity_account_staff_nik_bidx.sql`). Versi 1.3 (2026-11-21, F1-T98/F1-T99: rincian final tabel A20 K1 `corporate_client`, `corporate_client_document`, `corporate_verification_decision`, `corporate_member` sesuai migrasi `20261123100000_corporate_client_verification.sql` dan `20261123110000_corporate_member_account_korporat.sql`, termasuk kolom `submission_round`, `submitted_at`, `job_title` (Anonimkan), `anonymized_at`, constraint, index, batas panjang, dan hak role; perluasan A1.1 `account` dengan `corporate_admin`/`korporat` dan CHECK realm `idp_event_cursor`/`idp_event_receipt`; klasifikasi NPWP Terbatas; review DBA F1-T98/T99 LULUS). Versi 1.2 (2026-11-20, CR-024: istilah "markup" diganti "margin", termasuk nama tabel dan kolom `margin_rule`, `margin_type`, `margin_percent_bp`, `margin_fixed_idr`, `margin_rule_id`, `service_margin_total_idr`, `unit_margin_idr`, `purpose_code = margin_revenue`; belum ada migrasi yang membuat objek ini, jadi hanya dokumen yang berubah; struktur tidak berubah). Versi 1.1 (2026-11-19, CR-021: bagian A20 "Layanan korporat" ditambahkan sebagai **rancangan**, beserta usulan perubahan A1.1, A13.1, A15.4, A19.7, B3; rincian kolom, index, dan hak role final ditetapkan di task migrasi dengan review DBA dan keamanan; keputusan user `[USER] C21-P4`, PRD v1.3 EP-19, ADR-028, ADR-029). Versi 1.0, disetujui dan dikunci di gate Tahap 4, 2026-11-18 `[USER] T4-G5` (direview `dba-reviewer` tiga putaran dan `security-architect`)
Penyusun: Claude (persona Solution Architect + DBA)
Tingkat ketelitian: **T1 Kritis** (model data, data pribadi spesifik, uang, audit)
Dasar: PRD v1.1 `[DOK: 03-prd.md]`, ADR-001–ADR-005 (diterima) dan ADR-006–ADR-021 (diusulkan) `[DOK: adr/]`, catatan Tahap 4 `[DOK: 04-catatan-arsitektur.md]`, termasuk keputusan identity provider Keycloak dan Redis `[USER] T4-Q4` (ADR-007 dan ADR-011 sedang direvisi orkestrator; `[KEPUTUSAN-24]` direvisi).
Diagram: `docs/04-erd.md`.

Dokumen ini memfinalkan klasifikasi awal PRD §8 (`[KEPUTUSAN-10]`) dengan lima kelas tabel ADR-008 (`[KEPUTUSAN-21]`). Hal yang belum diketahui ditulis `[TIDAK DIKETAHUI]` beserta rujukan pertanyaan terbukanya. Keputusan teknis baru dari dokumen ini diberi nomor `[KEPUTUSAN-38]` dan seterusnya, dan dicatat oleh orkestrator di `docs/decision-log.md` (§E).

---

## 0. Konvensi

### 0.1 Label sumber

| Penanda | Arti |
| --- | --- |
| `US-`, `BR-`, `AB-`, `PB-`, `KU-`, `RF-`, `UP-`, `E-`, `NFR-` | Butir PRD v1.1. Berfungsi sebagai label `[DOK: 03-prd.md]` (BR juga `[DOK: 02-brd.md]`) |
| `ADR-xxx` | Keputusan arsitektur `[DOK: adr/]` |
| `[USER] T4-Qx` | Jawaban user di Tahap 4 (`docs/04-catatan-arsitektur.md`) |
| `[KEPUTUSAN-xx]` | Keputusan teknis. Nomor 10–37 sudah ada di `docs/decision-log.md`; nomor 38 ke atas baru dari dokumen ini (§E) |
| `[SRC-xx]` | Fakta eksternal terverifikasi di `docs/sources.md`. Kemampuan PostgreSQL dirujuk ke SRC-31–SRC-42; SRC lain (SATUSEHAT, terminologi) hanya dikutip "via ADR" yang sudah memuatnya. Tidak ada SRC baru |
| `[TIDAK DIKETAHUI]` | Belum ada informasi; rujukan pertanyaan terbuka disebut di sampingnya |

### 0.2 Konvensi penamaan dan tipe

| Aturan | Isi | Sumber |
| --- | --- | --- |
| Nama | Tabel, kolom, index, dan constraint snake_case, nama tabel tunggal | ADR-001, ADR-008 |
| Bahasa identifier | Nama tabel, kolom, dan **kode** nilai enum memakai bahasa Inggris snake_case; label tampilan berbahasa Indonesia sesuai PRD (pemetaan status di §0.9) | `[KEPUTUSAN-39]` |
| Kata kunci SQL | Tidak dipakai sebagai nama tabel. Pesanan disimpan di `patient_order` (bukan `order`), akun di `account` (bukan `user`) | `[KEPUTUSAN-38]` |
| Primary key | `id uuid`, UUIDv7 dibuat aplikasi; tipe `uuid` native 128-bit `[SRC-31]`. Tabel proyeksi memakai ID entitasnya sebagai PK | ADR-002, ADR-008 |
| Uang | `bigint`, Rupiah utuh, nama berakhiran `_idr`. Tanpa tipe pecahan | ADR-015, NFR-LOC-02 |
| Persen | `integer` basis poin (1% = 100), nama berakhiran `_bp` | `[KEPUTUSAN-59]` |
| Instan waktu | `timestamptz`, nama berakhiran `_at`, tersimpan sebagai UTC `[SRC-41]`. Tanggal kalender tanpa jam memakai `date` berakhiran `_on` atau `_date` | ADR-017 |
| Zona waktu | `text` berisi nama zona IANA (misalnya `Asia/Jakarta`), divalidasi aplikasi | ADR-017 |
| Kolom terenkripsi aplikasi | `<nama>_enc bytea` dan `data_key_id uuid` FK→`data_key` (wadah data). Isi terenkripsi tidak dapat dicari atau di-CHECK isinya; validasi panjang dilakukan aplikasi sebelum enkripsi | ADR-013 §3 |
| NIK | Tiga kolom: `nik_enc` (terenkripsi), `nik_bidx bytea` (blind index HMAC-SHA256 dengan kunci terpisah) beserta `nik_bidx_key_version smallint`, dan `nik_last4 char(4)` | ADR-013 §4 |
| Enum | `text` + `CHECK (kolom IN (...))`. Nilai yang berasal dari PRD §3.6 dipetakan di §0.9 | `[KEPUTUSAN-39]` |
| Identitas pengguna | Kredensial dan autentikasi di Keycloak. Tabel akun aplikasi menyimpan `idp_realm` + `idp_subject` (unik per realm); email hanya salinan untuk notifikasi, tanpa constraint unik di aplikasi | `[USER] T4-Q4`, `[KEPUTUSAN-45]`, `[KEPUTUSAN-46]` |
| Unique di Master | Partial unique index `WHERE deleted_at IS NULL` `[SRC-33]` | ADR-003, ADR-008 |
| Pencarian teks | Index trigram `pg_trgm` pada nama layanan, produk, rekanan, tenaga kesehatan, dan tampilan terminologi `[SRC-42]`, dibuat sebagai partial index `WHERE deleted_at IS NULL` `[SRC-33]` | ADR-007, review DBA S06 |
| FK komposit | (1) Relasi antar-tabel tenant RME memakai FK `(fasyankes_id, <induk>_id)` ke `UQ (fasyankes_id, id)` di induk, termasuk ke `rme_data_key` (review DBA S01). (2) Kesesuaian wadah DEK: tabel yang menyimpan `data_key_id` untuk wadah entitas lain memakai FK `(data_key_id, <wadah>_id)` ke `UQ (id, container_id)` di `data_key`/`rme_data_key`, sehingga DEK tidak dapat dipasangkan ke wadah lain; `container_type` diperiksa pustaka enkripsi (review DBA S02). Tabel anak dalam wadah yang sama memakai FK komposit ke induknya, misalnya `prescription_item (prescription_id, data_key_id)` → `prescription (id, data_key_id)` dan versi rekam medis RME → `rme_visit (fasyankes_id, id, data_key_id)` (review DBA S-N4). Pembeda `consultation_chat` dan `consultation_clinical`, yang punya `container_id` sama (ID sesi), tidak ditegakkan FK; pustaka enkripsi memeriksa `container_type` dan pemeriksaan ini wajib dicakup test. (3) Rantai pembayaran dan transisi status pesanan (A8, A12). Perilaku FK terhadap RLS dan FK pada kolom terhitung diverifikasi ke dokumentasi PostgreSQL di Tahap 5 | `[KEPUTUSAN-68]`, review DBA S01, S02, S04, S05 |
| Nomor yang dibaca manusia | `order_number`, `prescription_number`, `ticket_number`, `request_number`, `check_in_code`: `text` unik; format dan pembangkitnya ditetapkan Tahap 5 | `[KEPUTUSAN-57]` |

Penulisan kolom di tabel per entitas:

- **Null**: `tidak` = `NOT NULL`; `ya` = boleh `NULL`.
- **Kunci**: `PK`, `FK→tabel`, `UQ` (unik penuh), `UQp` (partial unique, kondisinya di bagian index).
- Nama constraint (`ck_`, `uq_`, `ix_`, `fk_`) ditetapkan di Tahap 5; di sini cukup isinya.
- Kolom bertanda "Anonimkan" ditimpa pekerja privasi saat anonimisasi data master (ADR-013 §5). Hak `UPDATE` per kolom role `privacy_worker` `[SRC-35]` di setiap tabel Master dan Master snapshot = kolom bertanda "Anonimkan" + `anonymized_at`, `updated_at`, `updated_by` (untuk `account_owner_nik_registry` juga kolom soft delete), di database A maupun C.

### 0.3 Legenda kelas (ADR-008)

| Kelas | Isi | Hak role aplikasi | Catatan |
| --- | --- | --- | --- |
| **M** Master | Data acuan yang boleh berubah | `SELECT`, `INSERT`, `UPDATE` | Soft delete (ADR-003). Query default memfilter `deleted_at IS NULL` (KU-04) |
| **M-snapshot** Master snapshot (sub-kelas Master) | Salinan data pribadi saat transaksi dibuat, yang hanya boleh berubah karena anonimisasi: `order_personal_snapshot`, `rme_visit_personal_snapshot` | `SELECT`, `INSERT` | `UPDATE` hanya role `privacy_worker` pada kolom "Anonimkan" `[KEPUTUSAN-69]` |
| **T** Transaksi | Bukti kejadian bisnis | `SELECT`, `INSERT` | Koreksi lewat baris pembalik atau versi baru (ADR-004) |
| **R** Riwayat status | Setiap transisi status entitas transaksi | `SELECT`, `INSERT` | `UNIQUE (<entitas>_id, version)`: transisi pertama menang (AB-36) |
| **P** Proyeksi | Status terkini, satu baris per entitas | `SELECT`, `INSERT`, `UPDATE` | Dapat dibangun ulang dari riwayat; dibandingkan harian (ADR-008 butir 4) |
| **O** Operasional | Data teknis berumur pendek | `SELECT`, `INSERT`, `UPDATE`, `DELETE` | Tidak boleh menjadi satu-satunya tempat data bisnis (ADR-008 butir 5) |
| **L** Log | Catatan kejadian teknis atau kepatuhan, hanya ditambah | `SELECT`, `INSERT` | |

Untuk kelas T, R, dan L, trigger penolak `UPDATE`, `DELETE`, dan `TRUNCATE` dipasang sebagai pertahanan kedua setelah pencabutan hak; galat trigger membatalkan statement pemicunya `[SRC-40]` (ADR-008). Untuk kelas M, hak `DELETE` tidak diberikan.

### 0.4 Kolom standar per kelas

Kolom standar ditulis sekali di sini. Di tabel per entitas, barisnya diringkas sebagai *STD-M*, *STD-T*, *STD-R*, *STD-P*, *STD-O*, atau *STD-L*. Di database RME, FK aktor menunjuk `rme_user`, bukan `account`.

**STD-M (Master)**

| Kolom | Tipe | Null | Kunci | Keterangan |
| --- | --- | --- | --- | --- |
| id | uuid | tidak | PK | UUIDv7 dari aplikasi (ADR-002) |
| created_at | timestamptz | tidak | | |
| created_by | uuid | ya | FK→account | `NULL` bila pelaku sistem atau integrasi; jenis pelaku tercatat di audit (ADR-008) |
| updated_at | timestamptz | tidak | | |
| updated_by | uuid | ya | FK→account | |
| deleted_at | timestamptz | ya | | Soft delete (ADR-003) |
| deleted_by | uuid | ya | FK→account | |
| delete_reason | text | ya | | |

CHECK: `(deleted_at IS NULL AND deleted_by IS NULL AND delete_reason IS NULL) OR (deleted_at IS NOT NULL AND delete_reason IS NOT NULL)`. Setiap soft delete dan pemulihan tercatat E-16 (ADR-003).

**STD-T (Transaksi)**: `id uuid PK`, `created_at timestamptz NOT NULL`, `created_by uuid NULL FK→account`.

**STD-R (Riwayat status)**

| Kolom | Tipe | Null | Kunci | Keterangan |
| --- | --- | --- | --- | --- |
| id | uuid | tidak | PK | |
| `<entitas>_id` | uuid | tidak | FK→entitas | |
| version | integer | tidak | UQ bersama `<entitas>_id` | Mulai 1; CHECK `version >= 1` |
| from_status | text | ya | | CHECK `(version = 1) = (from_status IS NULL)` |
| to_status | text | tidak | | CHECK nilai per entitas |
| reason_code | text | ya | | Kode alasan atau kasus refund (misalnya `rf_05`, `pb_07_elapsed`) |
| note_enc | bytea | ya | | Keterangan teks bebas (misalnya alasan pembatalan atau penolakan), terenkripsi dengan DEK wadah entitas induk. Entitas tanpa wadah sendiri memakai wadah `status_note` (§0.7). Keputusan orkestrator atas temuan F-10 |
| note_data_key_id | uuid | ya | FK→data_key | CHECK `(note_enc IS NULL) = (note_data_key_id IS NULL)` |
| actor_type | text | tidak | | CHECK (`patient`, `health_worker`, `rs_admin`, `partner_admin`, `partner_system`, `payment_gateway`, `courier`, `system`); RME memakai (`facility_user`, `ruang_sehat_system`, `system`) |
| actor_account_id | uuid | ya | FK→account | |
| actor_partner_id | uuid | ya | FK→partner | Rekanan pengirim pesan integrasi |
| correlation_id | uuid | tidak | | Sama dengan ID korelasi event audit E-09 (§7.2) |
| created_at | timestamptz | tidak | | Jam server |

**STD-R varian RME**: sama dengan STD-R, ditambah `fasyankes_id uuid NOT NULL` dengan FK komposit `(fasyankes_id, <entitas>_id)` ke induk; `actor_account_id` diganti `actor_rme_user_id uuid NULL FK→rme_user`; `actor_partner_id` dihapus; `note_data_key_id` FK→`rme_data_key` (wadah induk atau `status_note`); `actor_type` CHECK (`facility_user`, `ruang_sehat_system`, `system`) (review DBA S10).

**STD-P (Proyeksi)**: `<entitas>_id uuid PK FK→entitas`, `status text NOT NULL` (CHECK sama dengan `to_status`), `version integer NOT NULL CHECK (version >= 1)`, `updated_at timestamptz NOT NULL`. Ditulis dalam transaksi yang sama dengan riwayatnya, dengan syarat `version` lama (ADR-008 butir 2).

**STD-O (Operasional)**: `id uuid PK`, `created_at timestamptz NOT NULL`, `expires_at timestamptz` bila berumur.

**STD-L (Log)**: `id uuid PK`, `created_at timestamptz NOT NULL`.

### 0.5 Legenda sensitivitas

| Penanda | Arti | Sumber |
| --- | --- | --- |
| `PII` | Data pribadi: mengidentifikasi atau dapat dikaitkan ke orang (nama, kontak, alamat, lokasi, tanggal lahir, alamat IP, ID pengguna IdP) | ADR-013 |
| `Spesifik` | Data pribadi spesifik AB-70 (data kesehatan termasuk keluhan awal, data anak, data keuangan pribadi) **atau** data bertanda "Ya" di kolom Sensitif PRD §3.1 (NIK lengkap, dokumen identitas, foto bukti klaim). Akses detail oleh pihak selain subjek dicatat E-05 (KEPUTUSAN-07) | §3.1, AB-70 |
| `Spesifik*` | Spesifik **bila penerima layanan anak** (data anak). Usia anak ditentukan dari tanggal lahir; batasnya `[TIDAK DIKETAHUI]` (L-08, US-108 AC6) | AB-70 |
| `Keu` | Data keuangan pribadi (Spesifik) bila dibuka per pasien; tidak sensitif dalam laporan agregat (§3.1 baris Transaksi) | §3.1 |
| `Enc` | Dienkripsi tingkat aplikasi dengan DEK per wadah (ADR-013 §3) | ADR-013 |
| `Rahasia` | Kredensial atau material kunci: rahasia integrasi, token akses pihak luar, DEK terbungkus. Tidak pernah masuk audit atau log (§7.3) | ADR-005, ADR-013 |

### 0.6 Hak role database per kelas

Hak per tabel diberikan dengan `GRANT`; hak per kolom dipakai untuk pekerja privasi `[SRC-35]` (ADR-012 butir 8). `TRUNCATE` tidak diberikan ke role mana pun selain pemilik skema.

**(A) Database aplikasi Ruang Sehat**

| Kelas / tabel | `rs_app` (proses api, realtime, worker) | `audit_relay` | `privacy_worker` (pekerja privasi) | `key_rotation` (cadangan, lihat A19.7) | `rs_migration` |
| --- | --- | --- | --- | --- | --- |
| Master | S, I, U | — | S; U per kolom: kolom bertanda "Anonimkan" + `anonymized_at`, `updated_at`, `updated_by`. Untuk `account_owner_nik_registry` juga `deleted_at`, `deleted_by`, `delete_reason` | — | Pemilik skema |
| Master snapshot (`order_personal_snapshot`) | S, I | — | S; U per kolom seperti Master | — | Pemilik skema |
| Master khusus `data_key` | S, I | — | S; U hanya `wrapped_dek` (menjadi `NULL` bersama pengisian `shredded_at`, S-N3), `shredded_at`, `shred_reason` | U hanya `wrapped_dek`, `kek_ref`, dan hanya baris `shredded_at IS NULL` (trigger) | Pemilik skema |
| Transaksi | S, I | — | S kolom ID, FK subjek, `created_at`, dan `data_key_id` (untuk menemukan wadah) | — | Pemilik skema |
| Riwayat status | S, I | — | S kolom ID dan `note_data_key_id` | — | Pemilik skema |
| Proyeksi | S, I, U | — | — | — | Pemilik skema |
| Operasional | S, I, U, D | — | S, D untuk tabel operasional berisi data pribadi saat retensi | — | Pemilik skema |
| Log | S, I | — | S kolom ID, `created_at`, FK subjek; D hanya baris yang masa retensinya habis (lihat di bawah) | — | Pemilik skema |
| `retention_run` (Log) | S, I | — | S, I | — | Pemilik skema |
| `audit_outbox` | I saja | S, D | I saja (event E-16 dari pekerja privasi) | — | Pemilik skema |

Ketentuan hak `privacy_worker` (ADR-013 §5, temuan F-03, review DBA B02):
- **DELETE di tabel Log**: trigger penolak kelas L hanya mengizinkan `DELETE` bila role pelaksana adalah `privacy_worker` **dan** baris sudah melewati masa retensi kategori tabel itu menurut `retention_policy`. Selama nilai retensi masih `NULL` (PT-24), tidak ada baris Log yang dapat dihapus. Setiap penghapusan dicatat E-16 lewat `audit_outbox` dan dirangkum di `retention_run`.
- **Trigger satu arah** di `data_key` dan `rme_data_key`: (1) `privacy_worker` hanya boleh mengubah `wrapped_dek` menjadi `NULL` dalam statement yang sama dengan pengisian `shredded_at` dan `shred_reason`; perubahan lain oleh role ini ditolak (review DBA S-N3); (2) setelah `shredded_at` terisi, baris tidak dapat diubah lagi oleh role mana pun, sehingga DEK yang sudah dimusnahkan tidak dapat dihidupkan kembali `[KEPUTUSAN-66]`.
- Daftar kolom U per tabel = kolom bertanda "Anonimkan" di tabel entitas (GRANT per kolom `[SRC-35]`).

**(B) Database audit** (berlaku sama untuk database audit Ruang Sehat dan database audit RME)

| Tabel | `audit_writer` (relay) | `audit_reader` (backoffice / RME) | `audit_verifier` (job verifikasi) | `audit_migration` |
| --- | --- | --- | --- | --- |
| `audit_event`, `audit_event_subject`, `audit_event_key` | I; baca kepala rantai lewat akses sempit (temuan F-02) | S | S | Pemilik skema; juga ekspor dan *detach* partisi arsip |
| `audit_chain_anchor` | I | S | S | Pemilik skema |
| `audit_chain_verification` | — | S | S, I | Pemilik skema |
| `audit_archive_manifest` | — | S | S | Pemilik skema, I |

Role `audit_verifier` menambah daftar ADR-009 butir 4, karena job verifikasi harian perlu membaca seluruh rantai dan menulis hasilnya `[KEPUTUSAN-56]`.

**Khusus database audit RME: role `audit_tenant_reader`** (review DBA B03a, temuan F-06). Dipakai aplikasi RME untuk halaman log akses rekam medis Admin Fasyankes (§7.5, US-1310 AC3). Hak: S pada `audit_event` dan `audit_event_subject` saja, dengan RLS (`FORCE ROW LEVEL SECURITY`) yang membatasi baris ke `tenant_id` = pengaturan sesi yang dipasang aplikasi per transaksi `[SRC-34]`. RLS pada tabel terpartisi dan bentuk kebijakannya diverifikasi di Tahap 5; bila tidak dapat dipakai, cadangannya adalah fungsi baca per tenant yang hanya mengembalikan event E-05 fasyankes yang diminta. Role `audit_reader` tidak dipakai aplikasi RME untuk Admin Fasyankes.

**(C) Database aplikasi RME minimalis** (semua tabel milik tenant memakai RLS dengan `FORCE ROW LEVEL SECURITY`; role di bawah bukan pemilik tabel dan tanpa `BYPASSRLS` `[SRC-34]`, ADR-012 butir 6)

| Kelas / tabel | `rme_app` | `rme_auth` (pemetaan subjek IdP) | `rme_worker` (job, outbox, SATUSEHAT) | `rme_operator` (onboarding) | `audit_relay` | `privacy_worker` | `key_rotation` (cadangan, A19.7) | `rme_migration` |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Master tenant | S, I, U (kebijakan tenant) | — | Per tabel (lihat rincian di bawah) | S, I, U hanya `fasyankes`, `fasyankes_location`, `rs_integration_credential`, `rme_user`, `rme_user_role` | — | S; U per kolom: kolom "Anonimkan" + `anonymized_at`, `updated_at`, `updated_by` | — | Pemilik |
| Master snapshot (`rme_visit_personal_snapshot`) | S, I | — | — | — | — | S; U per kolom: kolom "Anonimkan" + `anonymized_at`, `updated_at`, `updated_by` | — | Pemilik |
| `rme_user` | S, I, U (kebijakan tenant; Admin Fasyankes mengelola akun fasyankesnya, US-1302) | S kolom `id`, `idp_subject`, `fasyankes_id`, `status` lintas tenant (kebijakan khusus) | S kolom `id`, `fasyankes_id`, `full_name` (penulis untuk resource SATUSEHAT) | S, I, U (akun operator dan Admin Fasyankes pertama) | — | S; U per kolom: kolom "Anonimkan" + `anonymized_at`, `updated_at`, `updated_by` | — | Pemilik |
| `rme_data_key` (Master khusus) | S, I | — | S, I | S, I (DEK kredensial saat onboarding) | — | S; U hanya `wrapped_dek` (menjadi `NULL` bersama pengisian `shredded_at`, S-N3), `shredded_at`, `shred_reason`, dengan kebijakan RLS khusus role ini | U hanya `wrapped_dek`, `kek_ref`, hanya baris `shredded_at IS NULL` | Pemilik |
| `rme_retention_policy` (acuan global, tanpa RLS) | S | — | — | — | — | S | — | Pemilik, I/U (perubahan nilai lewat migrasi setelah PT-24) |
| `rme_retention_run` (Log, RLS per fasyankes) | S (kebijakan tenant; Admin Fasyankes melihat laporan fasyankesnya) | — | — | — | — | S, I (kebijakan RLS khusus lintas tenant) | — | Pemilik |
| Transaksi, Riwayat status | S, I | — | Per tabel (lihat rincian di bawah) | **Tanpa** S atas `rme_patient`, `rme_visit*`, `rme_medical_record*`, `rme_lab_*`, `satusehat_*` (ADR-012 butir 6) | — | S kolom ID, FK subjek, `created_at`, `data_key_id`, `note_data_key_id` | — | Pemilik |
| Log | S, I | — | Per tabel | — | — | S kolom ID, `created_at`, FK subjek; D hanya baris yang masa retensinya habis menurut `rme_retention_policy` (trigger, seperti A) | — | Pemilik |
| Proyeksi | S, I, U | — | Per tabel | — | — | — | — | Pemilik |
| Operasional | S, I, U, D | — | Per tabel | — | — | S, D saat retensi | — | Pemilik |
| `rme_audit_outbox` | I saja | — | I saja | I saja | S, D | I saja (E-16) | — | Pemilik |
| `terminology_code` (acuan global, tanpa RLS) | S | — | S | — | — | — | — | Pemilik, I/U saat pemuatan |

Trigger penolak kelas L di database RME membaca masa retensi dari `rme_retention_policy` (C6.9); selama nilainya `NULL` (PT-24), tidak ada baris Log RME yang dapat dihapus (review DBA B10).

**Rincian hak `rme_worker`** (review DBA B03b). Worker mengambil job dari antrean semua tenant, lalu memasang konteks tenant per job; di luar antrean, semua hak tunduk pada kebijakan tenant.

| Tabel | Hak `rme_worker` | Job pemakai |
| --- | --- | --- |
| `rme_job`, `rme_integration_outbox` | S, I, U, D (kebijakan antrean lintas tenant) | Semua |
| `rme_audit_outbox` | I | Semua |
| `rme_inbound_event` | D | Pembersihan baris kedaluwarsa |
| `rme_visit`, `rme_patient`, `rme_lab_order_item`, `rme_queue_ticket`, `rme_visit_state`, `rme_medical_record_version`, `rme_lab_result`, `rme_lab_result_version`, `rme_home_visit_assignment`, `rme_queue_call_event`, `rme_service`, `rme_mcu_package_item`, `rme_home_visit_area`, `rme_schedule_rule`, `rme_schedule_exception`, `rme_slot` | S | Penyusunan pesan outbox ke Ruang Sehat dan resource SATUSEHAT |
| `satusehat_submission`, `satusehat_submission_status_event`, `satusehat_submission_attempt`, `satusehat_resource_ref` | S, I | Pengiriman SATUSEHAT (ADR-020 butir 6) |
| `satusehat_submission_state` | S, I, U | Sama |
| `satusehat_credential`, `rs_integration_credential`, `fasyankes`, `fasyankes_location` | S | Autentikasi ke SATUSEHAT dan Ruang Sehat |
| `rs_health_worker_roster`, `rme_health_worker_profile`, `rme_health_worker_link` | S | Penugasan home visit dan nomor IHS praktisi untuk resource SATUSEHAT (review DBA S-N2) |
| `rme_user` | S kolom `id`, `fasyankes_id`, `full_name` | Penulis rekam medis untuk resource SATUSEHAT |
| `external_access_token` | S, I, U, D | Cache token |
| `rme_data_key` | S, I | Dekripsi isi saat menyusun pesan dan resource; wadah `status_note` baru untuk keterangan galat pengiriman SATUSEHAT |
| `rme_stored_file` | S | Unggah PDF hasil lab ke Ruang Sehat |
| `terminology_code` | S, I, U | Pemuatan terminologi (job tanpa tenant) |

Tabel yang tidak tercantum tidak dapat diakses `rme_worker`. Pencarian nomor IHS praktisi dan pasien ke SATUSEHAT serta penyimpanannya ke `rme_health_worker_profile` dan `rme_patient` dilakukan `rme_app` (saat Admin Fasyankes melengkapi data, ADR-020 butir 3), sehingga `rme_worker` tidak perlu `UPDATE` atas tabel itu (review DBA S-N2). Webhook dari Ruang Sehat (pesanan terbayar, pembatalan, status home visit, daftar tenaga kesehatan aktif, jawaban penugasan) diproses sinkron oleh proses API RME dengan role `rme_app`, sehingga `rme_worker` tidak perlu hak `INSERT` atas tabel kunjungan dan pasien.

Role `rme_auth` dan `rme_worker` menambah daftar role ADR-012 butir 8 `[KEPUTUSAN-55]`: `rme_auth` memetakan subjek token Keycloak realm `rme` ke pengguna dan fasyankesnya **sebelum** konteks tenant dipasang; `rme_worker` mengambil job dari antrean semua tenant lalu memasang konteks tenant per job. `privacy_worker` di RME memakai kebijakan RLS khusus role itu (lintas tenant untuk job retensi dan anonimisasi). Bentuk kebijakan RLS per role diverifikasi ke dokumentasi Row Security Policies di Tahap 5; SRC-34 baru memverifikasi perilaku `BYPASSRLS` dan `FORCE ROW LEVEL SECURITY`.

### 0.7 Penempatan data pribadi dan wadah data

**Prinsip** `[KEPUTUSAN-42]` (penegasan ADR-013 §5 untuk semua tabel):

1. Tabel kelas T, R, dan L **tidak menyimpan PII dalam teks terbuka**. Mereka merujuk orang lewat ID, menyimpan PII di tabel Master yang dapat dianonimkan (misalnya `order_personal_snapshot`), atau menyimpannya terenkripsi dengan DEK wadah sehingga dapat dimusnahkan lewat crypto-shredding.
2. Notifikasi di dalam aplikasi dan log email menyimpan kode templat dan parameter ID, bukan teks jadi yang memuat nama (AB-73, NFR-PRIV-03).
3. Outbox per jenis, selaras dengan ADR-007 yang direvisi (B09):
   - **Outbox integrasi keluar** (`integration_outbox`, `rme_integration_outbox`) hanya menyimpan **rujukan**: jenis pesan dan ID entitas. Isi pesan disusun, dan bila perlu didekripsi, saat pengiriman (review DBA B04). Bila suatu pesan harus disimpan isinya (misalnya pembatalan yang entitasnya sudah berubah), disimpan sebagai `payload_enc` + `data_key_id` dengan wadah entitas terkait.
   - **Outbox audit** (`audit_outbox`, `rme_audit_outbox`) mengikuti ADR-009: event sudah dimasking pustaka audit, role aplikasi hanya `INSERT`, dan relay menghapus baris setelah terkirim.
   - **Outbox notifikasi** (`notification_outbox`) boleh memuat alamat email tujuan, lalu dihapus setelah terkirim.

**Wadah data dan DEK** (ADR-013 §3; nilai `data_key.container_type`):

| `container_type` | Satu wadah = | Kolom yang dienkripsi | Sumber |
| --- | --- | --- | --- |
| `consultation_chat` | percakapan satu sesi konsultasi | keluhan awal, teks chat; foto lampiran ikut dimusnahkan | ADR-013 §3, `[KEPUTUSAN-68]` |
| `consultation_clinical` | dokumentasi klinis satu sesi konsultasi | draf dan versi catatan, kepala dan item resep; PDF resep ikut dimusnahkan. Dipisah dari `consultation_chat` karena retensi catatan sebagai rekam medis mungkin berbeda (L-05) | ADR-013 §3, `[KEPUTUSAN-68]` |
| `lab_result` | satu hasil lab (semua versinya) | nilai hasil, alasan koreksi; PDF ikut dimusnahkan | ADR-013 §3 |
| `identity_document` | satu dokumen identitas | NIK; foto ikut dimusnahkan | ADR-013 §3 |
| `claim` | satu klaim | deskripsi klaim; foto bukti ikut dimusnahkan | ADR-013 §3 |
| `integration_client` | satu klien integrasi rekanan | rahasia penandatangan webhook HMAC | ADR-014, `[USER] T4-Q4`, `[KEPUTUSAN-43]` |
| `partner_bank_account` | satu rekening rekanan | nomor rekening | §7.3, `[KEPUTUSAN-43]` |
| `complaint_ticket` | satu tiket keluhan | teks pesan tiket | `[KEPUTUSAN-43]` |
| `data_subject_request` | satu permintaan hak subjek data | rincian permintaan dan teks tanggapan | `[KEPUTUSAN-43]` |
| `status_note` | satu entitas induk riwayat status yang tidak punya wadah sendiri (misalnya satu pesanan atau satu permintaan verifikasi) | `note_enc` di riwayat status | Keputusan orkestrator atas F-10, ADR-013 §3 |
| RME: `rme_visit` | satu kunjungan | rekam medis, hasil lab, alasan koreksi | ADR-013 §3 |
| RME: `status_note` | satu entitas induk riwayat status RME tanpa wadah sendiri | `note_enc` di riwayat status RME | Keputusan orkestrator atas F-10 |
| RME: `rme_patient_identity`, `rme_health_worker_identity` | NIK satu pasien atau satu tenaga kesehatan fasyankes | NIK | ADR-020 §3–§4 |
| RME: `satusehat_credential`, `rs_integration_credential`, `external_access_token` | satu kredensial | rahasia dan token pihak luar | ADR-020 §1, ADR-013 §3 |

Rahasia TOTP tidak lagi disimpan aplikasi karena MFA dikelola Keycloak `[USER] T4-Q4`. Kesesuaian DEK dengan wadahnya ditegakkan FK komposit (§0.2). Wadah selain lima wadah ADR-013 §3 (sesi konsultasi, kini dipecah dua; hasil lab; kunjungan RME; dokumen identitas; klaim) adalah perluasan `[KEPUTUSAN-43]` dan memerlukan penyesuaian teks ADR-013 (temuan F-04).

### 0.8 Pola pengaturan berlaku-mulai

Berlaku untuk `margin_rule`, `partner_fee_rule`, `consultation_rate`, `business_parameter`, `payment_method_fee`, dan `retention_policy` `[KEPUTUSAN-40]`:

1. Setiap perubahan adalah **baris baru** dengan `effective_from timestamptz`. Nilai yang berlaku pada instan *t* = baris dengan `effective_from <= t` terbesar dan `deleted_at IS NULL`. Riwayat perubahan tersedia tanpa membaca audit (US-809 AC1, US-810 AC1, US-812 AC1).
2. Baris yang `effective_from`-nya sudah lewat **tidak boleh diubah atau di-soft delete** (trigger penolak), karena dirujuk harga terkunci (AB-17). Baris yang dijadwalkan untuk masa depan boleh dibatalkan dengan soft delete.
3. Partial unique index `(<kunci lingkup>, effective_from) WHERE deleted_at IS NULL` `[SRC-33]`.
4. Harga terkunci menyimpan ID baris **dan** nilainya (`order_price_lock`, `order_parameter_snapshot`), sehingga pesanan tidak bergantung pada pembacaan ulang pengaturan.

### 0.9 Pemetaan kode status ke label PRD

Kode di database; label tampilan mengikuti PRD §3.6 `[UP-10]` dan §3.2 `[KEPUTUSAN-39]`.

**Pesanan** (`order_status_event.to_status`, `order_state.status`)

| Kode | Label PRD | Berlaku untuk |
| --- | --- | --- |
| `awaiting_payment` | Menunggu pembayaran | Semua |
| `confirmed` | Terkonfirmasi | Layanan terjadwal |
| `paid` | Dibayar | Produk |
| `expired` | Kedaluwarsa | Semua |
| `cancelled_by_patient` | Dibatalkan pasien | Semua |
| `cancelled_by_partner` | Dibatalkan rekanan | Layanan terjadwal |
| `cancelled_by_system` | Dibatalkan sistem | Klinik, lab, home visit (RF-18) |
| `in_progress` | Berlangsung | Konsultasi |
| `health_worker_no_show` | Tenaga kesehatan tidak hadir | Konsultasi, home visit |
| `patient_no_show` | Pasien tidak hadir | Layanan terjadwal |
| `completed` | Selesai | Semua |
| `checked_in` | Check-in | Klinik, lab |
| `assigned` | Ditugaskan | Home visit |
| `departed` | Berangkat | Home visit |
| `arrived` | Tiba | Home visit |
| `closed_unconfirmed` | Ditutup tanpa konfirmasi | Home visit |
| `reported_no_show` | Dilaporkan tidak datang | Home visit |
| `claim_rejected` | Klaim ditolak | Home visit |
| `objection_filed` | Keberatan diajukan | Home visit |
| `processing_by_outlet` | Diproses outlet | Produk |
| `rejected_by_outlet` | Ditolak outlet | Produk |
| `not_handed_over` | Tidak diserahkan | Produk |
| `shipped` | Dikirim | Produk |
| `delivered` | Diterima | Produk |
| `delivery_failed` | Gagal kirim | Produk |
| `claim_filed` | Klaim | Produk |
| `claim_approved` | Klaim disetujui | Produk |

Tabel transisi per jenis pesanan (PRD §3.6) disimpan di tabel acuan `order_status_transition` (A8.13) dan ditegakkan dengan FK komposit dari `order_status_event`, selain oleh pustaka transisi aplikasi (review DBA S05).

**Entitas lain**

| Entitas | Kode → label |
| --- | --- |
| Refund | `processing` Diproses, `succeeded` Berhasil, `failed` Gagal, `needs_review` Perlu dicek (§3.6, AB-32, AB-35) |
| Klaim | `submitted` Diajukan, `approved` Disetujui, `rejected` Ditolak (§3.6) |
| Verifikasi identitas | `pending` Menunggu verifikasi, `approved` Disetujui, `rejected` Ditolak, `cancelled` (pasien membatalkan pengiriman, US-106 AC5). "Belum diunggah" = tidak ada permintaan (AB-05) |
| Kewajiban pengembalian rekanan | `open` Terbuka, `settled` Selesai (US-808 AC3, AC5). "Menunggu penetapan penyebab" ditampilkan untuk refund RF-18 tanpa `refund_cause_determination` `[KEPUTUSAN-51]` |
| Resep | `issued` Terbit, `cancelled` Dibatalkan (US-309 AC3) |
| Tiket keluhan | `new` Baru, `in_progress` Diproses, `awaiting_patient` Menunggu pasien, `closed` Selesai (US-1104 AC1). Dibuka kembali (US-1104 AC3) = `closed → in_progress` `[KEPUTUSAN-54]` |
| Permintaan hak subjek data | `received` Diterima (US-1602 AC1), `in_progress` (tahap penanganan, US-1602 AC2) `[KEPUTUSAN-54]`, `completed` diselesaikan, `rejected` ditolak (US-1603 AC3) |
| Status integrasi rekanan | `not_started` Belum, `testing` Uji, `active` Aktif, `disrupted` Terganggu, `inactive` Nonaktif (US-1405) |
| Pengiriman SATUSEHAT (RME) | `awaiting_credentials` Menunggu kredensial, `queued` Antre, `sent` Terkirim, `failed` Gagal (KEPUTUSAN-16) |
| Percobaan pembayaran | `pending`, `succeeded`, `failed`, `expired`, `cancelled` (kode teknis, tidak ditampilkan sebagai status pesanan) |

### 0.10 Data di luar database aplikasi

**Keycloak** `[USER] T4-Q4`: hash password, faktor MFA TOTP dan kode pemulihan, token verifikasi email dan atur ulang password, penghitung penguncian login (US-103 AC3), dan sesi IdP **dikelola Keycloak** di database PostgreSQL miliknya sendiri (`keycloak`), sehingga tidak menjadi tabel aplikasi dan tidak dirinci di dokumen ini. Realm yang direncanakan: `pasien`, `nakes`, `admin` (admin Ruang Sehat dan admin rekanan), `rme` (pengguna RME minimalis), dan `integrasi` (klien service-account per rekanan dan per fasyankes untuk client credentials ADR-014). Otorisasi (ADR-012) tetap di aplikasi: peran, sub-peran, dan keterkaitan tenant disimpan di tabel aplikasi, bukan sebagai role Keycloak. Database `keycloak` mengikuti aturan lokasi Indonesia `[USER] T4-Q1`, dan cakupan backup serta retensinya ditetapkan bersama revisi ADR-011 dan ADR-019 (temuan F-13).

**Redis** `[USER] T4-Q4`: kelas Operasional, **bukan sumber kebenaran**, dan tidak berisi data pribadi selain ID. Isinya: cache status akun (untuk pemeriksaan KU-10 di setiap permintaan), daftar pencabutan sesi, penghitung batas laju, penyebaran pesan WebSocket antar-instans, dan sesi BFF aplikasi web. Kehilangan isi Redis tidak menghilangkan data bisnis: status akun dibaca ulang dari `account` dan `rme_user`, dan sesi yang hilang membuat pengguna masuk ulang. Antrean job dan outbox tetap di PostgreSQL (ADR-007). Satu-satunya tabel terkait sesi di database aplikasi adalah daftar pencabutan sesi (A1.4, RME C1.11), supaya pencabutan tetap berlaku walaupun isi Redis hilang (fact-check FC4-N2-2).

---

## A. Database aplikasi Ruang Sehat

Pengelompokan per domain mengikuti modul backend (ADR-006). Nama modul final diselaraskan dengan `docs/04-arsitektur.md` §3 (temuan F-12).

### A1. Identitas dan akun

#### A1.1 `account` (M)

Akun aplikasi untuk semua jenis pengguna Ruang Sehat, dipetakan ke pengguna Keycloak. Kredensial tidak disimpan di sini `[USER] T4-Q4`. Login Google/Apple dihapus (CR-007 `[USER] T4-Q2`). Sumber: BR-001, US-101, US-207, US-208, KU-10, ADR-011, `[KEPUTUSAN-24]` (direvisi), `[KEPUTUSAN-45]`.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-M* | | | | | |
| account_type | text | tidak | | — | CHECK (`patient`, `health_worker`, `rs_admin`, `partner_admin`, `corporate_admin`); `corporate_admin` = HR perusahaan (ADR-027 butir 3, F1-T99) |
| idp_realm | text | tidak | UQp | — | CHECK (`pasien`, `nakes`, `admin`, `korporat`). CHECK pasangan: `patient`→`pasien`, `health_worker`→`nakes`, `rs_admin`/`partner_admin`→`admin`, `corporate_admin`→`korporat` (F1-T99) |
| idp_subject | text | ya | UQp | PII | ID pengguna Keycloak di realm itu. Kosong **hanya** pada status `pending_provisioning` (CHECK `ck_account_idp_subject`): akun dibuat api lebih dulu, pengguna Keycloak dibuat worker IdP lalu ID-nya diadopsi (ADR-030 butir 5) `[KEPUTUSAN-305]` |
| email | text | tidak | | PII | Salinan untuk notifikasi; sumber kebenaran email dan verifikasinya di Keycloak. Perubahan email mandiri di konsol akun Keycloak dinonaktifkan; perubahan hanya lewat API aplikasi (US-105 AC4), yang memperbarui Keycloak dan kolom ini setelah email baru terverifikasi, sehingga salinan tidak menyimpang (review DBA S04, dicatat orkestrator di ADR-011) `[KEPUTUSAN-46]`. Anonimkan |
| status | text | tidak | | — | CHECK (`pending_provisioning`, `pending_activation`, `active`, `inactive`). Status ini yang menentukan otorisasi di setiap permintaan (KU-10); cache di Redis (§0.10). `pending_provisioning` = pengguna Keycloak belum dibuat worker IdP; berpindah ke `pending_activation` saat `idp_subject` diadopsi, atau dihapus lunak dengan `delete_reason = idp_provisioning_failed` bila penyediaan gagal (ADR-030 butir 5, realm `korporat` lebih dulu, CR-026) |
| status_reason | text | ya | | — | Alasan penonaktifan atau pengaktifan (US-207 AC1, AC4); tanpa data pribadi |
| status_changed_at | timestamptz | tidak | | — | |
| full_name | text | ya | | PII | Wajib untuk `rs_admin` dan `partner_admin` (CHECK); pasien dan tenaga kesehatan memakai tabel profil. Tidak diwajibkan untuk `corporate_admin`, karena nama HR undangan belum tentu diketahui saat akun dibuat (US-1903 AC1, F1-T99). Anonimkan |
| last_seen_at | timestamptz | ya | | — | Dipakai angka pendukung KPI (US-1701 AC2) lewat `patient_app_activity_month` |
| staff_nik_bidx | bytea | ya | | PII | Blind index NIK staf (HMAC-SHA256 32 byte, §0.2 baris NIK; tanpa `nik_enc`, sehingga NIK staf tidak dapat dibaca kembali). Hanya akun `rs_admin` (CHECK). Diisi Super Admin saat memberikan sub-peran Verifikator (US-208); dicocokkan dengan NIK pemilik akun pasien saat Verifikator memutuskan pengajuan identitas (US-201 AC8). Verifikator tanpa nilai ini tidak dapat memutuskan pengajuan apa pun (gagal-tertutup) `[KEPUTUSAN-294]`. Anonimkan (hak UPDATE kolom untuk `rs_privacy_worker`) |
| staff_nik_bidx_key_version | smallint | ya | | — | Versi kunci blind index `staff_nik_bidx` (CHECK ≥ 1; wajib ada bila `staff_nik_bidx` ada). Boleh tersisa setelah anonimisasi mengosongkan `staff_nik_bidx`. Karena NIK staf tidak tersimpan terenkripsi, rotasi kunci NIK (A2.8) tidak dapat menulis ulang kolom ini: versi kunci yang masih dirujuk tetap disimpan di peta kunci, dan pencocokan memakai versi kunci staf (NIK pemilik akun didekripsi lalu dihitung dengan versi itu) `[KEPUTUSAN-294]` |
| anonymized_at | timestamptz | ya | | — | Diisi pekerja privasi (ADR-013 §5) |

Index dan constraint:
- `UQp (idp_realm, idp_subject) WHERE deleted_at IS NULL` `[SRC-33]`.
- `ck_account_staff_nik_bidx` (32 byte), `ck_account_staff_nik_bidx_key_version` (≥ 1), `ck_account_staff_nik_bidx_rs_admin` (hanya `rs_admin`), `ck_account_staff_nik_bidx_version_present` (versi wajib bila blind index ada). Tanpa index dan tanpa keunikan: pencarian selalu per akun (PK), dan dua akun admin milik satu orang tidak dicegah di sini (di luar ruang lingkup US-201 AC8) `[KEPUTUSAN-294]`.
- `ix (status) WHERE deleted_at IS NULL`.
- `ix (idp_realm, lower(email)) WHERE deleted_at IS NULL`: pemeriksaan "belum ada akun dengan email yang sama di realm itu" oleh api di bawah kunci advisory transaksi per (realm, email ternormalisasi) sebelum akun `pending_provisioning` dibuat (ADR-030 butir 5). Bukan constraint unik: keunikan email tetap di Keycloak per realm `[KEPUTUSAN-46]` `[KEPUTUSAN-305]`.
- `UQ (id, idp_realm)` (`uq_account_id_idp_realm`): sasaran FK komposit `idp_provisioning_intent (account_id, idp_realm)` (A1.5), mengikat realm niat ke realm akun (ADR-030, review T1).
- Penonaktifan (US-207) mengubah `status`, menonaktifkan pengguna di Keycloak, dan mencatat pencabutan sesi di Redis dalam satu alur; urutan dan penanganan kegagalannya ditetapkan di revisi ADR-011.

#### A1.2 `rs_admin_role` (M)

Sub-peran admin Ruang Sehat; satu akun dapat memegang beberapa sub-peran. Otorisasi memakai tabel ini, bukan role Keycloak `[USER] T4-Q4`. Sumber: US-208, UP-18, ADR-012 butir 1 dan 6.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-M* | | | | | Pencabutan sub-peran = soft delete, dicatat E-11 |
| account_id | uuid | tidak | FK→account | — | Akun berjenis `rs_admin` (diperiksa aplikasi) |
| sub_role | text | tidak | | — | CHECK (`super_admin`, `verifier`, `customer_service`, `finance`, `content`, `auditor`, `privacy_officer`) |

Index: `UQp (account_id, sub_role) WHERE deleted_at IS NULL`; `ix (sub_role) WHERE deleted_at IS NULL` (cek Super Admin terakhir, US-208 AC2). Larangan mengubah sub-peran sendiri (US-208 AC3) ditegakkan aplikasi.

#### A1.3 `partner_admin_assignment` (M)

Keterkaitan tenant akun admin mitra ke rekanannya. Sumber: US-206, ADR-012 butir 5, `[USER] T4-Q4`.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-M* | | | | | |
| account_id | uuid | tidak | FK→account | — | Akun berjenis `partner_admin` |
| partner_id | uuid | tidak | FK→partner | — | Sumber `partner_id` sesi portal (ADR-012 butir 5) |

Index: `UQp (account_id) WHERE deleted_at IS NULL` (satu akun admin mitra untuk satu rekanan, sesuai US-206 AC2 "data rekanannya sendiri"); `ix (partner_id)`.

Tidak ada tabel sesi, MFA, kode pemulihan, atau token email di database ini; semuanya dikelola Keycloak dan Redis (§0.10).

#### A1.4 `session_revocation` (O)

Sesi Keycloak yang dicabut karena keluar atau pencabutan oleh admin, agar token akses lama ditolak walaupun cache Redis hilang (ADR-011 butir 8, US-103 AC5, KU-10). Disalin ke cache Redis; baris dibersihkan setelah `expires_at`. Sumber: ADR-011, fact-check FC4-N2-2.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| id | uuid | tidak | PK | — | |
| account_id | uuid | tidak | FK→account | — | |
| idp_session_id | text | tidak | UQ | — | ID sesi Keycloak (`sid`) yang dicabut; kehadiran klaim `sid` diverifikasi Tahap 5 |
| reason_code | text | tidak | | — | CHECK (`logout`, `admin_revoke`, `account_deactivated`, `role_changed`) |
| created_at | timestamptz | tidak | | — | |
| expires_at | timestamptz | tidak | | — | Saat token terakhir sesi itu pasti kedaluwarsa |

Index: `ix (expires_at)` untuk pembersihan.

#### A1.5 `idp_provisioning_intent` (T)

Niat penyediaan identitas di Keycloak yang dijalankan worker IdP (ADR-030 butir 2, 3). Satu baris per aksi Keycloak berhak `manage-users`; dicatat api di transaksi use case bersama job `app.job` yang muatannya hanya ID niat. Tanpa email, nama, password, atau data pribadi lain: email dibaca worker dari `account` saat dibutuhkan. Sumber: ADR-030, CR-026, `[KEPUTUSAN-302]`, `[KEPUTUSAN-305]`.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-T* | | | | | `created_by` = akun pelaku (NULL bila sistem atau pendaftar mandiri) |
| account_id | uuid | tidak | FK→account | — | Akun sasaran |
| idp_realm | text | tidak | FK komposit | — | CHECK (`pasien`, `nakes`, `admin`, `korporat`). FK komposit `(account_id, idp_realm)` → `account (id, idp_realm)`: realm niat selalu realm akun (review T1 ADR-030) |
| operation | text | tidak | | — | CHECK (`create_user`, `send_activation`, `disable_user`, `logout_user`, `delete_orphan`, `reconcile`), sama dengan daftar tertutup jenis job ADR-030 butir 3 (`idp_<operation>`) |
| purpose | text | tidak | | — | CHECK pasangan dengan `operation`: `create_user`/`send_activation` → `hr_self_registration` atau `hr_invitation` (hanya realm `korporat`); `disable_user` → `account_deactivation`; `logout_user` → `session_revocation`; `delete_orphan` → `provisioning_compensation`; `reconcile` → `reconciliation`. Susunan required action dan `lifespan` email aksi ditetapkan kode per realm dan tujuan, tidak disimpan |
| origin_intent_id | uuid | ya | FK komposit | — | Niat `create_user` yang ID-nya menjadi atribut Keycloak `rs_intent_id`; wajib tepat untuk `send_activation`, `delete_orphan`, `reconcile` (CHECK). FK komposit `(origin_intent_id, account_id)` → `(id, account_id)`: niat asal milik akun yang sama |
| correlation_id | uuid | tidak | | — | ID korelasi permintaan pemicu; dipakai event audit hasil worker |

Index: `UQ (id, account_id)` (sasaran FK komposit); `ix (account_id, created_at)`. Hak: kelas T (rs_app S+I, trigger tolak UPDATE/DELETE/TRUNCATE); pekerja privasi SELECT hanya `id`, `created_at`, `account_id`.

#### A1.6 `idp_provisioning_intent_status_event` (R)

*STD-R* dengan `idp_provisioning_intent_id`, **tanpa** `note_enc`/`note_data_key_id`/`actor_partner_id`: hasil worker hanya kode alasan, tanpa teks bebas dan tanpa pelaku rekanan `[KEPUTUSAN-305]`. `to_status` CHECK (`pending`, `succeeded`, `rejected`, `failed`); `reason_code` wajib tepat untuk `rejected`/`failed` dan berbentuk kode `^[a-z][a-z0-9_]{0,63}$` (daftar tertutup `identity::idp_domain::Refusal`). `actor_type` CHECK (`patient`, `health_worker`, `rs_admin`, `partner_admin`, `corporate_admin`, `system`). `UQ (niat, version)` membuat hasil pertama yang tersimpan berlaku, sehingga sewa job yang hilang tidak mencatat hasil dua kali (ADR-030 butir 6, AB-36). Sumber: ADR-030, ADR-008.

#### A1.7 `idp_provisioning_intent_state` (P)

*STD-P*, FK komposit ke riwayat `(niat, version, status)`. Index `ix (status, updated_at) WHERE status = 'pending'` untuk rekonsiliasi niat yang tertahan (ADR-030 butir 9 (c), langkah 8).

### A2. Pasien, penerima layanan, dan identitas

Model penerima layanan tunggal `[KEPUTUSAN-41]`: pemilik akun dan anggota keluarga sama-sama disimpan di `care_recipient`; pemilik akun ditandai `is_account_owner`. Pesanan, dokumen identitas, persetujuan anggota keluarga, dan subjek data audit merujuk `care_recipient`. Ini menyimpang secara struktur dari PRD §8.1 (entitas Pasien dan Anggota keluarga terpisah) tanpa mengubah aturannya (lihat `docs/04-erd.md` §0).

#### A2.1 `patient` (M)

Profil pemilik akun aplikasi pasien. Nama, tanggal lahir, dan jenis kelamin ada di baris `care_recipient` miliknya sendiri, supaya tidak tersimpan dua kali. Sumber: BR-002, US-105, US-1601 AC2, US-1603 AC6.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-M* | | | | | |
| account_id | uuid | tidak | FK→account, UQp | — | |
| phone | text | ya | | PII | Anonimkan |
| processing_status | text | tidak | | — | CHECK (`normal`, `consent_withdrawn`, `restricted`): "Persetujuan ditarik" dan "Pemrosesan dibatasi" (US-1601 AC2, US-1603 AC6) |
| processing_status_changed_at | timestamptz | ya | | — | Batas berlaku efek 3 x 24 jam diukur dari kolom ini (UP-54) |
| anonymized_at | timestamptz | ya | | — | |

Index: `UQp (account_id) WHERE deleted_at IS NULL`.

#### A2.2 `care_recipient` (M)

Penerima layanan: pemilik akun atau anggota keluarga. Sumber: BR-002, BR-008, US-105, US-108, US-109, UP-40, AB-03, `[KEPUTUSAN-41]`.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-M* | | | | | Hapus anggota keluarga = soft delete (US-109 AC3) |
| patient_id | uuid | tidak | FK→patient | — | Pemilik akun yang mengelola |
| is_account_owner | boolean | tidak | | — | |
| relationship | text | ya | | PII | Hubungan dengan pemilik akun; `NULL` untuk pemilik akun (CHECK `is_account_owner = (relationship IS NULL)`). Daftar nilai `[TIDAK DIKETAHUI]` (US-108 AC1 tidak merinci; ditetapkan Tahap 6) |
| full_name | text | tidak | | PII, Spesifik* | Anonimkan |
| birth_date | date | tidak | | PII, Spesifik* | Menentukan alur dewasa (KTP) atau anak (KK); batas usia `[TIDAK DIKETAHUI]` (L-08, US-108 AC6). Anonimkan |
| sex | text | tidak | | PII | CHECK (`male`, `female`). Anonimkan |
| nik_bidx | bytea | ya | | PII | Blind index NIK yang terakhir diajukan, untuk keunikan dalam satu akun. Anonimkan |
| nik_bidx_key_version | smallint | ya | | — | Versi kunci blind index (rotasi, ADR-013 §8) |
| nik_last4 | char(4) | ya | | PII | Tampilan termasking (§3.1). Anonimkan |
| latest_verification_request_id | uuid | ya | FK→identity_verification_request | — | Status verifikasi terkini = proyeksi permintaan ini; `NULL` = "Belum diunggah" |
| anonymized_at | timestamptz | ya | | — | |

Index dan constraint:
- `UQp (patient_id) WHERE is_account_owner AND deleted_at IS NULL` (satu baris pemilik akun).
- `UQp (patient_id, nik_bidx) WHERE deleted_at IS NULL AND nik_bidx IS NOT NULL`: menolak NIK anggota keluarga yang sama dengan NIK pemilik akun (US-108 AC4) dan duplikat di akun yang sama (US-108 AC5). Anggota keluarga yang dihapus lalu ditambah kembali menjadi baris baru (US-109 AC4).
- `ix (patient_id) WHERE deleted_at IS NULL`.

#### A2.3 `address` (M)

Alamat pasien untuk pengiriman produk dan home visit. Sumber: BR-002, US-105 AC2, UP-38, US-501 AC1, ADR-017 butir 2.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-M* | | | | | |
| patient_id | uuid | tidak | FK→patient | — | |
| label | text | ya | | — | Misalnya "Rumah" |
| recipient_name | text | tidak | | PII | Anonimkan |
| recipient_phone | text | tidak | | PII | Anonimkan |
| address_line | text | tidak | | PII | Anonimkan |
| province_code, city_code, district_code | text | ya | | PII | Kode wilayah untuk pencocokan area home visit `[KEPUTUSAN-60]`; sistem kodenya `[TIDAK DIKETAHUI]`, ditetapkan di spesifikasi kontrak (US-1401). Anonimkan |
| province_name, city_name | text | tidak | | — | Pencarian per kota (US-401 AC1) |
| postal_code | text | ya | | PII | Anonimkan |
| latitude, longitude | numeric(9,6) | ya | | PII | Tautan peta untuk tenaga kesehatan home visit (US-503 AC1). Anonimkan |
| time_zone | text | tidak | | — | Nama IANA (ADR-017 butir 2) |
| notes | text | ya | | PII | Patokan alamat. Anonimkan |
| is_default | boolean | tidak | | — | |

Index: `UQp (patient_id) WHERE is_default AND deleted_at IS NULL`; `ix (patient_id) WHERE deleted_at IS NULL`.

#### A2.4 `identity_document` (M)

Satu baris per pengajuan dokumen (NIK + foto KTP atau KK). Sumber: BR-003, BR-004, US-106, US-108 AC1–AC2, AB-02, AB-03, ADR-013 §4, ADR-018.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-M* | | | | | |
| care_recipient_id | uuid | tidak | FK→care_recipient | — | |
| document_type | text | tidak | | — | CHECK (`ktp`, `kk`) (AB-02, AB-03) |
| nik_enc | bytea | tidak | | Spesifik, Enc | Format 16 digit divalidasi aplikasi sebelum enkripsi (ADR-013 §4). Musnah bersama DEK |
| nik_bidx | bytea | tidak | | PII | Anonimkan |
| nik_bidx_key_version | smallint | tidak | | — | |
| nik_last4 | char(4) | tidak | | PII | Anonimkan |
| data_key_id | uuid | tidak | FK→data_key, UQ | — | Wadah `identity_document` |
| file_id | uuid | tidak | FK→stored_file | Spesifik | Foto KTP/KK, bucket `identity_document` (ADR-018) |

Index: `ix (care_recipient_id, created_at DESC)`.

#### A2.5 `identity_verification_request` (T)

Satu siklus pemeriksaan oleh Verifikator. Dibuat saat dokumen diajukan, atau saat nama, tanggal lahir, atau NIK berubah setelah disetujui (US-105 AC3, US-109 AC1, UP-39). Dipisah dari dokumen supaya keputusan pertama yang menang ditegakkan constraint riwayat status (US-201 AC5) `[KEPUTUSAN-41]`. Sumber: US-201, AB-05.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-T* | | | | | |
| care_recipient_id | uuid | tidak | FK→care_recipient | — | |
| identity_document_id | uuid | tidak | FK→identity_document | — | |
| trigger_type | text | tidak | | — | CHECK (`document_submitted`, `profile_changed`) |

Index: `ix (care_recipient_id, created_at DESC)`.

#### A2.6 `identity_verification_request_status_event` (R)

*STD-R* dengan `identity_verification_request_id`. `to_status` CHECK (`pending`, `approved`, `rejected`, `cancelled`). CHECK `to_status <> 'rejected' OR reason_code IS NOT NULL` (US-201 AC3, alasan dari `reason_code` konteks `identity_rejection`). Sumber: US-201 AC2–AC5, US-106 AC5, E-07.

#### A2.7 `identity_verification_request_state` (P)

*STD-P*. Index `ix (status, updated_at) WHERE status = 'pending'` untuk antrean urut terlama (US-201 AC1).

#### A2.8 `account_owner_nik_registry` (M)

Registri NIK yang sudah **disetujui sebagai pemilik akun**. Menegakkan UP-40 lintas akun di database: baris disisipkan dalam transaksi persetujuan, dan constraint unik menolak persetujuan kedua (US-201 AC7). Saat pengajuan, aplikasi memeriksa registri ini untuk menolak lebih awal (US-106 AC4). Sumber: UP-40, `[KEPUTUSAN-41]`.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-M* | | | | | Soft delete saat NIK pemilik berubah dan disetujui ulang, atau saat dianonimkan |
| nik_bidx | bytea | tidak | UQp | PII | Anonimkan (soft delete + timpa) |
| nik_bidx_key_version | smallint | tidak | | — | |
| care_recipient_id | uuid | tidak | FK→care_recipient, UQp | — | |
| identity_verification_request_id | uuid | tidak | FK→identity_verification_request | — | Persetujuan yang menjadi dasar |

Index: `UQp (nik_bidx) WHERE deleted_at IS NULL` `[SRC-33]`; `UQp (care_recipient_id) WHERE deleted_at IS NULL`.

Rotasi kunci blind index (ADR-013 §8): prosedur menulis ulang `nik_bidx` dan `nik_bidx_key_version` di `account_owner_nik_registry`, `care_recipient`, dan `identity_document` dalam satu transaksi per akun, sehingga keunikan tidak pernah dinilai dengan dua versi kunci berbeda (review DBA S05).

### A3. Persetujuan

#### A3.1 `consent_text_version` (M)

Versi teks persetujuan. Baris yang sudah terbit tidak boleh diubah isinya (trigger), mengikuti pola §0.8 `[KEPUTUSAN-40]`. Sumber: BR-103, BR-104, US-104 AC1, AC3–AC4, NFR-PRIV-05, `[KEPUTUSAN-12]`.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-M* | | | | | |
| consent_type | text | tidak | | — | CHECK (`patient_data_processing`, `family_adult_statement`, `guardian_consent`, `corporate_hr_registration`) (US-104, AB-04; `corporate_hr_registration` = persetujuan HR saat mendaftarkan perusahaan, US-1901 AC1, F1-T101, migrasi `20261126300000`) |
| version_label | text | tidak | | — | |
| content | text | tidak | | — | Memuat tujuh informasi Pasal 21 ayat (1) UU PDP (US-104 AC1). Teks legalitas menunggu L-08; retensi menunggu PT-24 |
| effective_from | timestamptz | tidak | | — | |
| published_at | timestamptz | ya | | — | Setelah terisi, kolom isi tidak dapat diubah |
| change_notice_sent_at | timestamptz | ya | | — | Pemberitahuan sebelum tanggal berlaku (US-104 AC4) |

Index: `UQp (consent_type, version_label) WHERE deleted_at IS NULL`.

#### A3.2 `consent_item` (M)

Butir persetujuan per versi; butir baru dapat ditambah tanpa mengubah aplikasi. Sumber: US-104 AC3, US-1601 AC2, AC4, `[KEPUTUSAN-12]`.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-M* | | | | | |
| consent_text_version_id | uuid | tidak | FK→consent_text_version | — | |
| item_code | text | tidak | | — | |
| label | text | tidak | | — | |
| is_primary | boolean | tidak | | — | Butir utama: penarikannya membuat akun "Persetujuan ditarik" (US-1601 AC2) |
| is_required | boolean | tidak | | — | Wajib untuk registrasi (US-101 AC4) |
| sort_order | smallint | tidak | | — | |

Index: `UQp (consent_text_version_id, item_code) WHERE deleted_at IS NULL`.

#### A3.3 `consent_record` (L)

Bukti persetujuan diberikan atau ditarik, termasuk pernyataan persetujuan anggota keluarga dewasa dan persetujuan orang tua/wali (AB-04). Sumber: US-104 AC2, US-108 AC1–AC2, US-1601, E-13, E-15, NFR-PRIV-05.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-L* | | | | | `created_at` = waktu persetujuan |
| patient_id | uuid | ya | FK→patient | — | Pemberi persetujuan (pasien). Tepat satu dari `patient_id` dan `account_id` terisi (CHECK `ck_consent_record_subject`, F1-T101) |
| account_id | uuid | ya | FK→account | — | Pemberi persetujuan akun HR realm korporat (US-1901 AC1, F1-T101); wajib bersama `channel = corporate_portal`, tanpa `care_recipient_id` |
| care_recipient_id | uuid | ya | FK→care_recipient | — | Subjek pernyataan anggota keluarga |
| consent_text_version_id | uuid | tidak | FK→consent_text_version | — | |
| consent_item_id | uuid | tidak | FK→consent_item | — | |
| action | text | tidak | | — | CHECK (`granted`, `withdrawn`) |
| channel | text | tidak | | — | CHECK (`patient_app`, `backoffice`, `corporate_portal`) (US-104 AC2 "kanal"; `corporate_portal` hanya untuk subjek `account_id`) |
| correlation_id | uuid | tidak | | — | Penghubung ke event E-13/E-15 |

Index: `ix (patient_id, consent_item_id, created_at DESC)` untuk status persetujuan terkini (US-1601 AC1); `ix (account_id, consent_item_id, created_at DESC) WHERE account_id IS NOT NULL` untuk subjek HR (F1-T101). Hak: `rs_privacy_worker` SELECT per kolom `id`, `created_at`, `patient_id`, `care_recipient_id`, `account_id`.

### A4. Tenaga kesehatan, tarif, dan jadwal

#### A4.1 `health_worker` (M)

Profil tenaga kesehatan; satu tenaga kesehatan dinaungi satu klinik (AB-50). Kolom `partner_id` adalah keterkaitan tenant nakes ↔ klinik `[USER] T4-Q4`. Sumber: BR-005, US-202, US-203, AB-50, AB-51, AB-54.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-M* | | | | | |
| account_id | uuid | tidak | FK→account, UQp | — | Akun realm `nakes` |
| partner_id | uuid | tidak | FK→partner | — | Klinik penaung; rekanan berkemampuan `clinic` (diperiksa aplikasi) |
| full_name | text | tidak | | PII | Anonimkan |
| profession | text | tidak | | — | CHECK (`doctor`, `psychologist`, `physiotherapist`, `medical_staff`) (US-202 AC1). Hanya `doctor` yang dapat menerbitkan resep (AB-53) |
| phone | text | ya | | PII | Anonimkan |
| verification_status | text | tidak | | — | CHECK (`pending`, `approved`, `rejected`) (US-202 AC2, US-203) |
| approved_at | timestamptz | ya | | — | |

Index: `UQp (account_id) WHERE deleted_at IS NULL`; `ix (partner_id) WHERE deleted_at IS NULL`; index trigram `full_name WHERE deleted_at IS NULL` `[SRC-42]` (US-303 AC1). Tanpa index `profession` sendiri (selektivitas rendah, review DBA S06). "Siap melayani" (AB-54) dihitung dari akun aktif, `verification_status`, izin berlaku, klinik aktif, dan tarif ada.

#### A4.2 `health_worker_license` (M)

Dokumen izin dan penugasan tenaga kesehatan; setiap unggahan baru menjadi baris baru. Sumber: BR-102, US-202 AC1, AC3, US-203, US-204, PB-14, AB-51.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-M* | | | | | |
| health_worker_id | uuid | tidak | FK→health_worker | — | |
| document_type_code | text | tidak | | — | Kode dari `document_requirement` (nilai awal `str`, `sip`, ditambah `assignment_letter` dari US-202 AC1); daftar final `[TIDAK DIKETAHUI]` (L-07) |
| document_number | text | ya | | PII | Nomor STR/SIP. Anonimkan |
| expires_on | date | ya | | — | CHECK `document_type_code <> 'sip' OR expires_on IS NOT NULL` |
| file_id | uuid | tidak | FK→stored_file | PII | Bucket `license_document` |
| review_status | text | tidak | | — | CHECK (`pending`, `approved`, `rejected`, `superseded`) |
| reviewed_at | timestamptz | ya | | — | |
| reviewed_by | uuid | ya | FK→account | — | Verifikator |
| rejection_reason | text | ya | | — | |

Index: `ix (health_worker_id, document_type_code) WHERE deleted_at IS NULL`; `ix (expires_on) WHERE document_type_code = 'sip' AND review_status = 'approved' AND deleted_at IS NULL` untuk pemeriksaan harian PB-09 dan penonaktifan otomatis (US-204).

#### A4.3 `health_worker_affiliation_confirmation` (M)

Konfirmasi afiliasi dari klinik yang dicatat Verifikator. Dipisah dari keputusan karena memuat nama petugas klinik (PII), sesuai prinsip §0.7. Sumber: US-203 AC1–AC2, UP-19.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-M* | | | | | |
| health_worker_id | uuid | tidak | FK→health_worker | — | |
| confirmed_on | date | tidak | | — | |
| confirmation_method | text | tidak | | — | Cara konfirmasi (teks; daftar nilai tidak ditetapkan PRD) |
| clinic_officer_name | text | tidak | | PII | Anonimkan |

#### A4.4 `health_worker_verification_decision` (T)

Keputusan Verifikator atas pengajuan tenaga kesehatan. Sumber: US-203, E-10.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-T* | | | | | `created_by` = Verifikator |
| health_worker_id | uuid | tidak | FK→health_worker | — | |
| decision | text | tidak | | — | CHECK (`approved`, `rejected`) |
| affiliation_confirmation_id | uuid | ya | FK→health_worker_affiliation_confirmation | — | CHECK `decision <> 'approved' OR affiliation_confirmation_id IS NOT NULL` (US-203 AC2) |
| reason_code | text | ya | | — | CHECK `decision <> 'rejected' OR reason_code IS NOT NULL` (US-203 AC3) |

#### A4.5 `consultation_rate` (M, berlaku-mulai)

Tarif dan durasi konsultasi per tenaga kesehatan dan per jenis (PB-13), diisi admin klinik di portal. Pola §0.8 `[KEPUTUSAN-40]`. Sumber: BR-018, US-301, UP-29, AB-17, E-14.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-M* | | | | | `created_by` = admin klinik |
| health_worker_id | uuid | tidak | FK→health_worker | — | |
| consultation_mode | text | tidak | | — | CHECK (`chat`, `video`) |
| fee_idr | bigint | tidak | | — | CHECK `fee_idr > 0` (US-301 AC1) |
| duration_minutes | integer | tidak | | — | CHECK `duration_minutes BETWEEN 10 AND 120 AND duration_minutes % 5 = 0` (UP-29) |
| effective_from | timestamptz | tidak | | — | Perubahan berlaku segera (US-301 AC3) |

Index: `UQp (health_worker_id, consultation_mode, effective_from) WHERE deleted_at IS NULL`. Jenis tanpa baris tarif tidak dapat dipesan (US-301 AC2).

#### A4.6 `practice_schedule` (M)

Aturan jadwal praktik online mingguan. Satu aturan untuk satu jenis konsultasi, supaya slot terbentuk dengan durasi jenis itu (US-302 AC1) `[KEPUTUSAN-47]`. Sumber: BR-017, US-302, AB-41, AB-52, AB-55, ADR-017 butir 3, E-14.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-M* | | | | | Diubah tenaga kesehatan atau admin klinik (US-302 AC4) |
| health_worker_id | uuid | tidak | FK→health_worker | — | |
| consultation_mode | text | tidak | | — | CHECK (`chat`, `video`) |
| day_of_week | smallint | tidak | | — | CHECK `BETWEEN 1 AND 7` (1 = Senin) |
| local_start_time | time | tidak | | — | Jam lokal di `time_zone` |
| local_end_time | time | tidak | | — | CHECK `local_end_time > local_start_time` |
| time_zone | text | tidak | | — | Zona perangkat saat jadwal dibuat (ADR-017 butir 3) |
| valid_from | date | tidak | | — | |
| valid_until | date | ya | | — | |

Tumpang tindih jadwal (US-302 AC2, AB-52) diperiksa aplikasi terhadap aturan dan slot yang ada.

#### A4.7 `practice_schedule_exception` (M)

Pengecualian tanggal pada jadwal praktik. Sumber: US-302 AC1.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-M* | | | | | |
| health_worker_id | uuid | tidak | FK→health_worker | — | |
| exception_date | date | tidak | | — | |
| time_zone | text | tidak | | — | |
| reason | text | ya | | — | Tanpa data pribadi |

Index: `UQp (health_worker_id, exception_date) WHERE deleted_at IS NULL`.

#### A4.8 `consultation_slot` (M)

Slot konsultasi sebagai instan UTC (ADR-017 butir 3). Satu slot paling banyak satu sesi terbayar (AB-52, PRD §8.3) ditegakkan dengan `booked_order_id` yang diisi secara kondisional (`... WHERE booked_order_id IS NULL`) saat pesanan dikonfirmasi, dan dikosongkan saat pesanan dibatalkan (US-805 AC2) `[KEPUTUSAN-47]`. Sumber: US-302, US-303, US-304, KU-09.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-M* | | | | | Soft delete hanya bila `booked_order_id IS NULL` (US-302 AC3, AC5) |
| health_worker_id | uuid | tidak | FK→health_worker | — | |
| practice_schedule_id | uuid | ya | FK→practice_schedule | — | |
| consultation_mode | text | tidak | | — | CHECK (`chat`, `video`) |
| start_at | timestamptz | tidak | | — | |
| end_at | timestamptz | tidak | | — | CHECK `end_at > start_at` |
| booked_order_id | uuid | ya | FK→patient_order, UQp | — | |

Index: `UQp (health_worker_id, start_at) WHERE deleted_at IS NULL`; `UQp (booked_order_id) WHERE booked_order_id IS NOT NULL`; `ix (health_worker_id, start_at) WHERE deleted_at IS NULL AND booked_order_id IS NULL` (slot terdekat, US-303 AC2). Pencegahan tumpang tindih rentang waktu di database (constraint exclusion) belum terverifikasi sumbernya dan dievaluasi di Tahap 5; sampai saat itu ditegakkan aplikasi.

### A5. Rekanan

#### A5.1 `partner` (M)

Klinik, lab, toko kecantikan, toko alat kesehatan, atau apotek rekanan. Sumber: BR-006, BR-031, US-205, US-207, US-1202, US-1405, AB-40, AB-62–AB-65.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-M* | | | | | |
| legal_name | text | tidak | | — | |
| display_name | text | tidak | | — | |
| partner_type | text | tidak | | — | CHECK (`clinic`, `lab`, `beauty_store`, `medical_device_store`, `pharmacy`) (US-205 AC1). Menentukan kelayakan obat bebas (BR-031) |
| address_line | text | tidak | | — | Alamat usaha |
| province_code, city_code, district_code | text | ya | | — | Sistem kode `[TIDAK DIKETAHUI]` (`[KEPUTUSAN-60]`) |
| province_name, city_name | text | tidak | | — | |
| postal_code | text | ya | | — | |
| latitude, longitude | numeric(9,6) | ya | | — | Lokasi usaha |
| time_zone | text | tidak | | — | IANA; tampilan jadwal dan laporan rekanan (AB-40, AB-42) |
| contact_phone, contact_email | text | ya | | PII | Kontak dapat milik perorangan. Anonimkan |
| system_source | text | tidak | | — | CHECK (`own_system`, `rme_minimal`) (US-205 AC1) |
| lifecycle_status | text | tidak | | — | CHECK (`pending_verification`, `active`, `inactive`, `rejected`) |
| status_reason | text | ya | | — | Alasan penonaktifan (US-207 AC1) |
| integration_status | text | tidak | | — | CHECK (`not_started`, `testing`, `active`, `disrupted`, `inactive`) (US-1405 AC1). Perubahan oleh admin dicatat E-10 |
| integration_status_changed_at | timestamptz | tidak | | — | |

Index: index trigram `display_name WHERE deleted_at IS NULL` `[SRC-42]`; `ix (city_name, lifecycle_status, integration_status) WHERE deleted_at IS NULL`. Rekanan tampil ke pasien bila izin disetujui, margin ada, dan `integration_status = 'active'` (US-205 AC2), dihitung aplikasi.

#### A5.2 `partner_capability` (M)

Kemampuan rekanan; klinik yang juga menjual produk punya dua baris (AB-65). Sumber: AB-65, UP-20, US-1201 AC3.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-M* | | | | | |
| partner_id | uuid | tidak | FK→partner | — | |
| capability | text | tidak | | — | CHECK (`clinic`, `lab`, `outlet`) |

Index: `UQp (partner_id, capability) WHERE deleted_at IS NULL`.

#### A5.3 `partner_license_document` (M)

Dokumen izin usaha rekanan. Jenis dokumen wajib per jenis rekanan `[TIDAK DIKETAHUI]` (L-02, L-03; US-205 AC4). Sumber: BR-006, US-205.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-M* | | | | | |
| partner_id | uuid | tidak | FK→partner | — | |
| document_type_code | text | tidak | | — | Kode dari `document_requirement` |
| document_number | text | ya | | — | |
| issued_on | date | ya | | — | |
| expires_on | date | ya | | — | |
| file_id | uuid | tidak | FK→stored_file | — | Bucket `license_document` |
| review_status | text | tidak | | — | CHECK (`pending`, `approved`, `rejected`, `superseded`) |
| reviewed_at | timestamptz | ya | | — | |
| reviewed_by | uuid | ya | FK→account | — | |
| rejection_reason | text | ya | | — | |

#### A5.4 `partner_bank_account` (M)

Rekening penerima bagian rekanan. Perubahan rekening baru berlaku setelah disetujui Verifikator (US-1202 AC3, UP-31). Nomor rekening dienkripsi `[KEPUTUSAN-43]`. Sumber: BR-041, US-205 AC1, AC5, US-1202, §7.3.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-M* | | | | | `created_by` = admin Ruang Sehat yang memasukkan |
| partner_id | uuid | tidak | FK→partner | — | |
| bank_code | text | tidak | | — | |
| account_holder_name | text | tidak | | PII | Dapat milik perorangan. Anonimkan |
| account_number_enc | bytea | tidak | | Enc | Tidak pernah tampil lengkap di portal (US-1202 AC1) atau audit (§7.3) |
| account_number_last4 | char(4) | tidak | | — | |
| data_key_id | uuid | tidak | FK→data_key | — | Wadah `partner_bank_account` |
| gateway_recipient_ref | text | ya | | — | ID penerima di payment gateway; cara pendaftaran `[TIDAK DIKETAHUI]` (US-205 AC5, Tahap 5) |
| approval_status | text | tidak | | — | CHECK (`pending_approval`, `active`, `rejected`, `superseded`). Diubah hanya bersama `partner_bank_account_decision` |

Index: `UQp (partner_id) WHERE approval_status = 'active' AND deleted_at IS NULL` (satu rekening aktif).

#### A5.5 `partner_bank_account_decision` (T)

Keputusan Verifikator atas rekening. Pemisahan tugas (Verifikator ≠ pemasuk data) ditegakkan aplikasi (ADR-012 butir 7). Sumber: US-1202 AC3.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-T* | | | | | `created_by` = Verifikator |
| partner_bank_account_id | uuid | tidak | FK→partner_bank_account, UQ | — | Satu keputusan per rekening |
| decision | text | tidak | | — | CHECK (`approved`, `rejected`) |
| reason_code | text | ya | | — | |

#### A5.6 `document_requirement` (M)

Daftar dokumen wajib per jenis tenaga kesehatan (PB-14) dan per jenis rekanan. PB-14 dimodelkan sebagai tabel, bukan nilai di `business_parameter`, karena isinya daftar. Sumber: PB-14, US-205 AC4, US-812 AC1, UP-19.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-M* | | | | | Diatur Super Admin |
| subject_kind | text | tidak | | — | CHECK (`health_worker`, `partner`) |
| subject_subtype | text | tidak | | — | `profession` atau `partner_type` |
| document_type_code | text | tidak | | — | |
| label | text | tidak | | — | |
| requires_expiry | boolean | tidak | | — | |

Index: `UQp (subject_kind, subject_subtype, document_type_code) WHERE deleted_at IS NULL`. Nilai awal tenaga kesehatan: STR dan SIP untuk semua jenis, menunggu L-07 (PB-14). Daftar per jenis tidak boleh kosong (US-812 AC1), diperiksa aplikasi.

### A6. Pengaturan

#### A6.1 `margin_rule` (M, berlaku-mulai)

Margin per rekanan (PB-11). Margin = persentase dari harga rekanan (bukan dari harga jual); harga ke pasien = harga rekanan × (1 + margin%) [CR-024]. Nilainya `[TIDAK DIKETAHUI]` sampai ditetapkan CEO (PT-17). Pola §0.8. Sumber: BR-042, US-809, AB-10, AB-11a, AB-17, `[KEPUTUSAN-59]`.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-M* | | | | | `created_by` = Keuangan (US-809 AC4) |
| partner_id | uuid | tidak | FK→partner | — | |
| margin_type | text | tidak | | — | CHECK (`percent`, `fixed`) |
| percent_bp | integer | ya | | — | CHECK `percent_bp BETWEEN 0 AND 10000` (0–100%, US-809 AC1) |
| fixed_idr | bigint | ya | | — | CHECK `fixed_idr >= 0` |
| effective_from | timestamptz | tidak | | — | Segera atau tanggal mendatang |

CHECK: `(margin_type = 'percent' AND percent_bp IS NOT NULL AND fixed_idr IS NULL) OR (margin_type = 'fixed' AND fixed_idr IS NOT NULL AND percent_bp IS NULL)`. Index: `UQp (partner_id, effective_from) WHERE deleted_at IS NULL`. Apakah persen boleh pecahan desimal tidak dinyatakan PRD; basis poin menampung sampai dua desimal, dan UI dapat membatasi ke bilangan bulat.

#### A6.2 `partner_fee_rule` (M, berlaku-mulai)

Biaya rekanan untuk klinik dan lab (PB-12). Nilai dan cara penagihan `[TIDAK DIKETAHUI]` (PT-17, AB-24). Sumber: BR-043, US-810.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-M* | | | | | |
| partner_id | uuid | tidak | FK→partner | — | |
| fee_type | text | tidak | | — | CHECK (`none`, `monthly_subscription`, `per_registration`) |
| amount_idr | bigint | ya | | — | CHECK `(fee_type = 'none') = (amount_idr IS NULL)` dan `amount_idr >= 0` |
| effective_from | timestamptz | tidak | | — | |

Index: `UQp (partner_id, effective_from) WHERE deleted_at IS NULL`.

#### A6.3 `business_parameter` (M, berlaku-mulai)

Parameter PB-01–PB-10 dan PB-15–PB-20. PB-11 di `margin_rule`, PB-12 di `partner_fee_rule`, PB-13 di `consultation_rate`, PB-14 di `document_requirement`. Sumber: §3.7, US-812, AB-17, AB-43.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-M* | | | | | Diatur Super Admin |
| param_code | text | tidak | | — | CHECK (`pb_01` … `pb_10`, `pb_15` … `pb_20`) |
| scope_key | text | tidak | | — | `''` bila tanpa lingkup. PB-01 per jenis layanan terjadwal (`consultation`, `clinic_registration`, `lab_registration`, `home_visit`); PB-08 `reminder_1`, `reminder_2` |
| value_number | bigint | tidak | | — | CHECK `value_number > 0` (US-812 AC3) |
| unit | text | tidak | | — | CHECK (`minute`, `hour`, `day`, `workday`, `count`). PB-05, PB-16, PB-20 = `workday` (UP-37); PB-10 = `count` |
| effective_from | timestamptz | tidak | | — | |

Index: `UQp (param_code, scope_key, effective_from) WHERE deleted_at IS NULL`. Nilai awal diisi dari PRD §3.7 lewat migrasi data. Batas usia anak (US-108 AC6) akan menjadi parameter setelah L-08; kodenya belum dibuat karena nilainya `[TIDAK DIKETAHUI]`.

#### A6.4 `payment_method_fee` (M, berlaku-mulai)

Tarif biaya transaksi per metode pembayaran yang ditampilkan sebelum bayar (AB-12), diatur Keuangan. Daftar metode `[TIDAK DIKETAHUI]` sampai payment gateway dipilih (AB-25); pembebanan menunggu L-12. Sumber: ADR-015 butir 8, AB-12, AB-22, US-801 AC1.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-M* | | | | | |
| method_code | text | tidak | | — | Kode metode dari penyedia |
| display_name | text | tidak | | — | |
| fee_type | text | tidak | | — | CHECK (`none`, `fixed`, `percent`, `fixed_plus_percent`) |
| fixed_idr | bigint | ya | | — | CHECK `>= 0` |
| percent_bp | integer | ya | | — | CHECK `BETWEEN 0 AND 10000` |
| charged_to | text | tidak | | — | CHECK (`patient`, `partner_share`): non-QRIS ditambahkan ke total pasien; QRIS sebagai MDR dipotong dari bagian rekanan (AB-12, AB-22) |
| effective_from | timestamptz | tidak | | — | |

Index: `UQp (method_code, effective_from) WHERE deleted_at IS NULL`. Biaya aktual dari laporan penyedia dicatat di `fund_split_report_line` (ADR-015 butir 8).

#### A6.5 `workday_calendar` (M)

Kalender hari kerja yang diatur Super Admin. Setiap tanggal ditulis eksplisit, sehingga tidak ada asumsi hari kerja mingguan `[KEPUTUSAN-58]`. Wajib terisi sebelum go-live (ADR-017). Sumber: UP-37, ADR-017 butir 4, PB-05, PB-16, PB-20.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-M* | | | | | |
| calendar_date | date | tidak | UQp | — | |
| is_workday | boolean | tidak | | — | |
| description | text | ya | | — | Misalnya nama hari libur |

Index: `UQp (calendar_date) WHERE deleted_at IS NULL`. Jam kerja untuk NFR-DR-02 `[TIDAK DIKETAHUI]` dan tidak disimpan di sini.

#### A6.6 `retention_policy` (M, berlaku-mulai)

Masa retensi dan tindakan akhir per kategori data. Semua nilai `[TIDAK DIKETAHUI]` (PT-24, L-05, L-06); nilai kosong berarti data tidak diproses retensi (ADR-013 §6). Sumber: BR-112, NFR-RET-01, NFR-RET-02, US-1606, `[KEPUTUSAN-29]`.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-M* | | | | | |
| data_category | text | tidak | | — | Kode kategori, misalnya `identity_document`, `consultation_clinical`, `lab_result`, `claim`, `complaint_ticket`, `data_subject_request`, `financial_transaction`, `order_personal_snapshot`, `notification`, `integration_log`. Daftar final ditetapkan bersama PT-24 |
| retention_months | integer | ya | | — | CHECK `retention_months > 0` |
| end_action | text | ya | | — | CHECK (`end_processing`, `anonymize`, `crypto_shred`); CHECK `(retention_months IS NULL) = (end_action IS NULL)` |
| legal_basis_note | text | ya | | — | |
| effective_from | timestamptz | tidak | | — | |

Index: `UQp (data_category, effective_from) WHERE deleted_at IS NULL`.

#### A6.7 `reason_code` (M)

Daftar alasan yang dipilih admin atau sistem (misalnya alasan penolakan verifikasi). Sumber: US-201 AC3, US-203 AC3, US-807 AC4, US-1603 AC3.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-M* | | | | | |
| context | text | tidak | | — | CHECK (`identity_rejection`, `health_worker_rejection`, `bank_account_rejection`, `order_cancellation`, `claim_rejection`, `dsr_rejection`, `status_transition`, `corporate_rejection`). `corporate_rejection` ditambahkan F1-T104 (US-1902 AC3; migrasi `20261124200000_platform_reason_code_corporate_rejection.sql`), isi awal empat kode **draf** di `20261124201000_corporate_rejection_reason_seed.sql` |
| code | text | tidak | | — | |
| label | text | tidak | | — | |

Index: `UQp (context, code) WHERE deleted_at IS NULL`.

### A7. Katalog salinan rekanan

Semua tabel di bagian ini adalah salinan data dari sistem rekanan (AB-60), kelas Master (PRD §8.1). Kolom bersama: `partner_id`, `external_id` (ID di sistem rekanan), dan `synced_at` (waktu sinkron terakhir untuk tampilan KU-08, ADR-014 §6). Data yang dihapus rekanan di-soft delete dengan `delete_reason = 'removed_by_partner'`. Pesan sinkronisasi diterapkan dalam satu transaksi atau ditolak seluruhnya (US-1402 AC2).

#### A7.1 `partner_service` (M)

Layanan klinik, pemeriksaan lab, paket MCU, dan home visit beserta harga rekanan. Sumber: BR-020–BR-023, US-401, US-601, AB-14.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-M* | | | | | |
| partner_id | uuid | tidak | FK→partner | — | |
| external_id | text | tidak | | — | |
| service_kind | text | tidak | | — | CHECK (`clinic_service`, `lab_test`, `mcu_package`, `home_visit`) |
| name | text | tidak | | — | |
| description | text | ya | | — | |
| preparation_info | text | ya | | — | Informasi persiapan (US-601 AC2) |
| partner_price_idr | bigint | tidak | | — | CHECK `>= 0`. Harga pasien = harga rekanan + margin (AB-10), dihitung server |
| patient_price_idr | bigint | tidak | | — | Harga tampil terhitung (harga rekanan + margin berlaku per unit, AB-11a), dihitung ulang saat sinkron dan saat margin berganti. Hanya untuk tampilan dan urutan (US-601 AC1); harga bayar selalu dihitung ulang saat dikunci (AB-17) (review DBA S06) |
| is_bookable | boolean | tidak | | — | |
| synced_at | timestamptz | tidak | | — | |

Index: `UQp (partner_id, external_id) WHERE deleted_at IS NULL`; index trigram `name WHERE deleted_at IS NULL` `[SRC-42]`; `ix (service_kind, patient_price_idr) WHERE deleted_at IS NULL` (urut harga bayar terendah, US-601 AC1, AB-14).

#### A7.2 `mcu_package_item` (M)

Daftar pemeriksaan dalam paket MCU. Sumber: US-601 AC2, US-1303 AC1.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-M* | | | | | |
| partner_service_id | uuid | tidak | FK→partner_service | — | Layanan berjenis `mcu_package` |
| external_id | text | tidak | | — | |
| item_name | text | tidak | | — | |
| sort_order | smallint | tidak | | — | |
| synced_at | timestamptz | tidak | | — | |

Index: `UQp (partner_service_id, external_id) WHERE deleted_at IS NULL`.

#### A7.3 `home_visit_area` (M)

Area layanan home visit per klinik, dalam bentuk kode wilayah `[KEPUTUSAN-60]`; sistem kodenya `[TIDAK DIKETAHUI]` dan ditetapkan di spesifikasi kontrak (US-1401). Sumber: BR-022, US-501 AC1, AC3, UP-22.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-M* | | | | | |
| partner_id | uuid | tidak | FK→partner | — | |
| region_level | text | tidak | | — | CHECK (`province`, `city`, `district`) |
| region_code | text | tidak | | — | |
| synced_at | timestamptz | tidak | | — | |

Index: `UQp (partner_id, region_level, region_code) WHERE deleted_at IS NULL`; `ix (region_level, region_code) WHERE deleted_at IS NULL`.

#### A7.4 `partner_slot` (M)

Slot dan kuota klinik, lab, dan home visit dari sistem rekanan, sebagai instan beserta zona waktu rekanan (ADR-017 butir 3). Ketersediaan final diperiksa sinkron ke rekanan sebelum bayar (AB-15, ADR-014 §2). Sumber: BR-020, BR-021, EP-14.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-M* | | | | | |
| partner_id | uuid | tidak | FK→partner | — | |
| partner_service_id | uuid | tidak | FK→partner_service | — | |
| external_id | text | tidak | | — | |
| start_at | timestamptz | tidak | | — | |
| end_at | timestamptz | tidak | | — | CHECK `end_at > start_at` |
| time_zone | text | tidak | | — | Zona rekanan (AB-40) |
| capacity | integer | ya | | — | Kuota; CHECK `>= 0` |
| available_count | integer | ya | | — | Sisa menurut rekanan; CHECK `>= 0` |
| synced_at | timestamptz | tidak | | — | |

Index: `UQp (partner_id, external_id) WHERE deleted_at IS NULL`; `ix (partner_service_id, start_at) WHERE deleted_at IS NULL`.

#### A7.5 `product` (M)

Salinan katalog outlet. Produk yang ditandai obat resep ditolak saat penerimaan dan tidak disimpan (US-702 AC3, BR-032). Sumber: BR-030–BR-033, US-701, US-702.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-M* | | | | | |
| partner_id | uuid | tidak | FK→partner | — | Rekanan berkemampuan `outlet` |
| external_id | text | tidak | | — | |
| name | text | tidak | | — | |
| description | text | ya | | — | Informasi wajib BPOM `[TIDAK DIKETAHUI]` (L-03, US-702 AC4) |
| category | text | tidak | | — | CHECK (`otc_drug`, `beauty`, `other_health`) (US-701 AC1) |
| partner_price_idr | bigint | tidak | | — | CHECK `>= 0` |
| patient_price_idr | bigint | tidak | | — | Harga tampil terhitung, sama seperti di `partner_service` (review DBA S06) |
| display_block_reason | text | ya | | — | CHECK (`otc_from_ineligible_outlet`): obat bebas dari outlet yang bukan apotek atau klinik berizin tidak ditampilkan dan dilaporkan ke admin (US-702 AC2, BR-031) |
| weight_grams | integer | ya | | — | Kebutuhan data kurir untuk ongkos kirim `[TIDAK DIKETAHUI]` (Tahap 5) |
| synced_at | timestamptz | tidak | | — | |

Index: `UQp (partner_id, external_id) WHERE deleted_at IS NULL`; index trigram `name WHERE deleted_at IS NULL` `[SRC-42]`; `ix (category, patient_price_idr) WHERE deleted_at IS NULL AND display_block_reason IS NULL`.

#### A7.6 `product_image` (M)

Foto produk dari rekanan. Disimpan di bucket katalog publik tanpa data pribadi; bucket ini belum tercantum di ADR-018 (temuan F-07). Sumber: US-702 AC1.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-M* | | | | | |
| product_id | uuid | tidak | FK→product | — | |
| file_id | uuid | tidak | FK→stored_file | — | |
| sort_order | smallint | tidak | | — | |

#### A7.7 `product_stock` (M)

Stok produk menurut outlet. Dipisah dari `product` karena diperbarui jauh lebih sering. Sumber: BR-033, US-701 AC2, US-703 AC2.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-M* | | | | | |
| product_id | uuid | tidak | FK→product, UQp | — | |
| quantity | integer | tidak | | — | CHECK `>= 0`; 0 = "habis" |
| synced_at | timestamptz | tidak | | — | |

Index: `UQp (product_id) WHERE deleted_at IS NULL`.

### A8. Pesanan, status, dan harga terkunci

Satu tabel pesanan untuk lima jenis, dengan tabel detail 1:1 per jenis: `consultation_session` (konsultasi), `appointment` (klinik dan lab), `home_visit_detail`, dan `product_order_detail` (PRD §8.1).

#### A8.1 `patient_order` (T)

Pesanan. Status disimpan di riwayat dan proyeksi (ADR-008). Sumber: BR-040, US-304, US-402, US-501, US-703, KU-05, `[KEPUTUSAN-38]`.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-T* | | | | | `created_by` = akun pasien |
| order_number | text | tidak | UQ | — | Nomor yang dibaca manusia (`[KEPUTUSAN-57]`) |
| order_kind | text | tidak | | — | CHECK (`consultation`, `clinic_registration`, `lab_registration`, `home_visit`, `product`) |
| patient_id | uuid | tidak | FK→patient | — | Pemesan (pemilik akun) |
| care_recipient_id | uuid | ya | FK→care_recipient | — | Penerima layanan; CHECK `order_kind = 'product' OR care_recipient_id IS NOT NULL` (PRD §8.3, AB-01) |
| partner_id | uuid | tidak | FK→partner | — | Rekanan pemberi layanan atau penjual; untuk konsultasi = klinik penaung tenaga kesehatan (AB-50) |
| client_request_id | uuid | tidak | | — | Kunci idempotensi dari aplikasi (KU-05) |

Index: `UQ (patient_id, client_request_id)`; `UQ (id, order_kind)` (sasaran FK komposit dari `order_status_event`); `ix (patient_id, created_at DESC)` (riwayat, US-406, US-706); `ix (partner_id, created_at)` (laporan rekanan, US-1201); `ix (care_recipient_id, created_at DESC)`.

#### A8.2 `order_status_event` (R)

*STD-R* dengan `patient_order_id`, ditambah kolom `order_kind text NOT NULL` (review DBA S05). `to_status` CHECK semua kode di §0.9 (pesanan). `reason_code` memuat kasus refund (`rf_xx`) atau pemicu (misalnya `pb_07_elapsed`, `amount_mismatch`). Keterangan "pasien tidak ada di lokasi" (US-503 AC3) dan alasan pembatalan tenaga kesehatan (US-307 AC4) memakai `reason_code` + `note_enc` (wadah `status_note` pesanan, atau `consultation_chat` untuk konsultasi). Sumber: §3.6, AB-36, E-09, US-1404 AC2.

Constraint:
- FK komposit `(patient_order_id, order_kind)` → `patient_order (id, order_kind)`, sehingga jenis pesanan di event selalu sama dengan pesanannya.
- FK komposit `(order_kind, from_status, to_status)` → `order_status_transition (order_kind, from_status, to_status)`: hanya transisi PRD §3.6 yang dapat disimpan. Untuk `version = 1`, `from_status` `NULL` dan baris acuan awal memakai `from_status IS NULL`; perilaku FK komposit dengan kolom `NULL` (baris tidak diperiksa) diverifikasi di Tahap 5, dan transisi awal juga diperiksa pustaka transisi.
- Keterkaitan ke versi sebelumnya (`from_status` = `to_status` event versi `version - 1`) lewat FK diri `(patient_order_id, version - 1, from_status)` → `(patient_order_id, version, to_status)` memerlukan kolom terhitung; kelayakannya diverifikasi di Tahap 5, dan sampai itu dijaga pustaka transisi.

#### A8.3 `order_state` (P)

*STD-P* dengan `patient_order_id`. Index `ix (status, updated_at)`; partial index `ix (status, updated_at) WHERE status NOT IN ('expired','cancelled_by_patient','cancelled_by_partner','cancelled_by_system','completed','health_worker_no_show','patient_no_show','closed_unconfirmed','claim_rejected','rejected_by_outlet','not_handed_over','delivery_failed','claim_approved')` untuk pesanan yang belum berstatus akhir (review DBA S-N1). Batas waktu dijalankan sebagai job per entitas (§D.7); proyeksi dipindai hanya sebagai jaring pengaman harian.

#### A8.4 `order_personal_snapshot` (M-snapshot)

Snapshot data pribadi penerima layanan saat pesanan dibuat: nama, tanggal lahir, jenis kelamin, nomor HP, dan alamat kunjungan atau pengiriman. Sub-kelas Master snapshot `[KEPUTUSAN-69]`: role aplikasi hanya `SELECT` dan `INSERT`; `UPDATE` hanya `privacy_worker` pada kolom "Anonimkan", sehingga snapshot dapat dianonimkan tanpa mengubah baris transaksi tetapi tidak dapat diubah aplikasi (ADR-013 §5, `[KEPUTUSAN-42]`). Isi yang dikirim ke rekanan dibatasi ke data minimal kontrak `[TIDAK DIKETAHUI]` (L-08, AB-75). Sumber: US-403 AC1, US-503 AC1, US-703 AC3, US-1403 AC1.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-M* | | | | | |
| patient_order_id | uuid | tidak | FK→patient_order, UQp | — | |
| recipient_full_name | text | tidak | | PII, Spesifik* | Anonimkan |
| recipient_birth_date | date | ya | | PII, Spesifik* | Layanan terjadwal. Anonimkan |
| recipient_sex | text | ya | | PII | CHECK (`male`, `female`). Anonimkan |
| recipient_phone | text | ya | | PII | Home visit dan produk. Tampil ke tenaga kesehatan hanya di hari kunjungan (US-503 AC1). Anonimkan |
| address_line | text | ya | | PII | Home visit dan produk. Anonimkan |
| province_code, city_code, district_code, postal_code | text | ya | | PII | Anonimkan |
| city_name, province_name | text | ya | | — | |
| latitude, longitude | numeric(9,6) | ya | | PII | Anonimkan |
| address_notes | text | ya | | PII | Anonimkan |
| time_zone | text | ya | | — | Zona alamat kunjungan |
| source_address_id | uuid | ya | FK→address | — | |
| anonymized_at | timestamptz | ya | | — | |

Index: `UQp (patient_order_id) WHERE deleted_at IS NULL`.

#### A8.5 `partner_recipient_alias` (M)

ID pseudonim penerima layanan per rekanan. Dikirim ke rekanan sebagai pengenal pasien yang stabil antar-pesanan, sehingga RME dapat menampilkan riwayat pasien di fasyankes (US-1310 AC1) tanpa membuka ID internal Ruang Sehat, dan rekanan berbeda tidak dapat menghubungkan pasien yang sama `[KEPUTUSAN-61]`. Sumber: BR-126, NFR-PRIV-03.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-M* | | | | | |
| partner_id | uuid | tidak | FK→partner | — | |
| care_recipient_id | uuid | tidak | FK→care_recipient | — | |
| alias_id | uuid | tidak | UQ | PII | Acak (bukan UUIDv7), tidak diturunkan dari ID lain |

Index: `UQp (partner_id, care_recipient_id) WHERE deleted_at IS NULL`.

#### A8.6 `order_price_lock` (T)

Harga terkunci saat pembayaran dibuat (AB-17). Dibuat satu per percobaan pembayaran, karena biaya transaksi bergantung pada metode yang dipilih (US-802 AC4); harga yang berlaku untuk pesanan adalah harga terkunci dari percobaan yang tertangkap `[KEPUTUSAN-48]`. Sumber: ADR-015 butir 3, AB-10–AB-13, AB-17, AB-22, US-801.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-T* | | | | | |
| patient_order_id | uuid | tidak | FK→patient_order | — | |
| margin_rule_id | uuid | tidak | FK→margin_rule | — | |
| margin_type | text | tidak | | — | Nilai salinan dari aturan |
| margin_percent_bp | integer | ya | | — | |
| margin_fixed_idr | bigint | ya | | — | |
| consultation_rate_id | uuid | ya | FK→consultation_rate | — | Konsultasi saja |
| service_partner_total_idr | bigint | tidak | | Keu | Jumlah harga rekanan semua baris |
| service_margin_total_idr | bigint | tidak | | Keu | |
| service_price_total_idr | bigint | tidak | | Keu | CHECK `= service_partner_total_idr + service_margin_total_idr` |
| shipping_fee_idr | bigint | tidak | | Keu | 0 untuk non-produk. Penerima dana ongkos kirim `[TIDAK DIKETAHUI]` (PT-31) |
| payment_method_fee_id | uuid | tidak | FK→payment_method_fee | — | |
| payment_method_code | text | tidak | | — | |
| transaction_fee_idr | bigint | tidak | | Keu | 0 untuk QRIS (AB-12) |
| expected_mdr_idr | bigint | ya | | Keu | QRIS; dipotong dari bagian rekanan (AB-22) |
| expected_partner_share_idr | bigint | tidak | | Keu | Harga rekanan dikurangi MDR bila QRIS (AB-22) |
| total_idr | bigint | tidak | | Keu | CHECK `= service_price_total_idr + shipping_fee_idr + transaction_fee_idr` dan `> 0` |

Semua kolom `_idr` CHECK `>= 0`. Index `ix (patient_order_id)`; `UQ (id, patient_order_id)` (sasaran FK komposit dari `payment_attempt`, review DBA S04).

#### A8.7 `order_price_lock_line` (T)

Rincian per unit untuk perhitungan margin per unit (AB-11a). Baris harga terkunci adalah **satu-satunya sumber jumlah barang** pesanan produk; jumlah yang berlaku = baris dari harga terkunci percobaan yang tertangkap (review DBA S04). Tabel `product_order_item` dihapus karena menduplikasi jumlah. Sumber: AB-11, AB-11a, UP-51, US-801 AC2, US-703, US-706.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-T* | | | | | |
| order_price_lock_id | uuid | tidak | FK→order_price_lock | — | |
| line_no | smallint | tidak | | — | |
| item_kind | text | tidak | | — | CHECK (`service`, `product`) |
| partner_service_id | uuid | ya | FK→partner_service | — | |
| product_id | uuid | ya | FK→product | — | |
| description | text | tidak | | — | Nama layanan atau produk saat dikunci |
| quantity | integer | tidak | | — | CHECK `> 0` |
| unit_partner_price_idr | bigint | tidak | | Keu | CHECK `>= 0` |
| unit_margin_idr | bigint | tidak | | Keu | Margin per unit setelah pembulatan (setengah ke atas); CHECK `>= 0` |
| line_total_idr | bigint | tidak | | Keu | CHECK `= quantity * (unit_partner_price_idr + unit_margin_idr)` |

Index: `UQ (order_price_lock_id, line_no)`; `UQp (order_price_lock_id, product_id) WHERE product_id IS NOT NULL`; `ix (product_id)` (riwayat pesanan per produk). CHECK `(item_kind = 'product') = (product_id IS NOT NULL)`. Konsistensi jumlah baris dengan kolom total di kepala diperiksa dengan mekanisme yang sama seperti keseimbangan jurnal (§D.4).

#### A8.8 `order_parameter_snapshot` (T)

Nilai parameter bisnis yang berlaku untuk pesanan saat pembayaran dibuat (AB-17, US-812 AC2). Sumber: ADR-015 butir 3.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-T* | | | | | |
| order_price_lock_id | uuid | tidak | FK→order_price_lock | — | |
| business_parameter_id | uuid | tidak | FK→business_parameter | — | |
| param_code | text | tidak | | — | |
| scope_key | text | tidak | | — | |
| value_number | bigint | tidak | | — | |
| unit | text | tidak | | — | |

Index: `UQ (order_price_lock_id, param_code, scope_key)`.

#### A8.9 `inventory_hold` (O)

Penahanan slot atau stok selama PB-07 (US-304 AC1, US-402 AC1). Slot konsultasi dikendalikan Ruang Sehat sehingga dijaga constraint unik; slot dan stok rekanan dikendalikan sistem rekanan, dan Ruang Sehat menyimpan ID penahanan dari rekanan `[KEPUTUSAN-47]` `[KEPUTUSAN-62]`. Sumber: KU-09, AB-15, ADR-014 §2.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-O* | | | | | `expires_at` = PB-07 sejak penahanan |
| hold_kind | text | tidak | | — | CHECK (`consultation_slot`, `partner_slot`, `product_stock`) |
| patient_order_id | uuid | tidak | FK→patient_order | — | |
| consultation_slot_id | uuid | ya | FK→consultation_slot, UQp | — | |
| partner_slot_id | uuid | ya | FK→partner_slot | — | |
| product_id | uuid | ya | FK→product | — | |
| quantity | integer | tidak | | — | CHECK `> 0` |
| partner_hold_ref | text | ya | | — | ID penahanan dari rekanan |

CHECK: tepat satu target terisi sesuai `hold_kind`. Index: `UQp (consultation_slot_id) WHERE consultation_slot_id IS NOT NULL` (hanya satu pasien yang dapat menahan slot terakhir, KU-09); `ix (expires_at)`. Pelepasan slot karena kedaluwarsa tercatat sebagai transisi pesanan `expired` (ADR-008 butir 5).

#### A8.10 `order_partner_acknowledgement` (T)

Bukti pesanan sudah diterima sistem rekanan. Menjadi dasar RF-18 (belum diterima saat jadwal mulai) dan "Pasien tidak hadir" klinik/lab (§3.5, §3.6). Sumber: US-402 AC3, US-404 AC3, ADR-014 §4.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-T* | | | | | `created_at` = waktu penerimaan dikonfirmasi rekanan |
| patient_order_id | uuid | tidak | FK→patient_order, UQ | — | |
| partner_order_ref | text | ya | | — | ID pesanan di sistem rekanan |

#### A8.11 `order_follow_up_flag` (L)

Penanda pesanan yang perlu ditindaklanjuti Layanan Pelanggan. Sumber: US-402 AC3, ADR-014 §4, §3.6 (Ditutup tanpa konfirmasi).

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-L* | | | | | |
| patient_order_id | uuid | tidak | FK→patient_order | — | |
| flag_type | text | tidak | | — | CHECK (`partner_delivery_failing`, `home_visit_closed_unconfirmed`) |

Index: `ix (flag_type, created_at)`.

#### A8.12 `order_follow_up_resolution` (L)

Penyelesaian penanda tindak lanjut. Sumber: US-402 AC3.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-L* | | | | | |
| order_follow_up_flag_id | uuid | tidak | FK→order_follow_up_flag, UQ | — | |
| resolution_code | text | tidak | | — | |
| resolved_by | uuid | tidak | FK→account | — | |

#### A8.13 `order_status_transition` (M, acuan)

Tabel transisi status pesanan per jenis dari PRD §3.6, diisi lewat migrasi data dan menjadi sasaran FK komposit dari `order_status_event` (review DBA S05). Sumber: §3.6, UP-10, AB-36, US-1404 AC2.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-M* | | | | | Perubahan hanya lewat migrasi (role aplikasi S saja) |
| order_kind | text | tidak | | — | CHECK sama dengan `patient_order.order_kind` |
| from_status | text | ya | | — | `NULL` = status awal (`awaiting_payment`) |
| to_status | text | tidak | | — | |
| allowed_actor_types | text[] | tidak | | — | Aktor pemicu menurut kolom "Aktor" §3.6; diperiksa pustaka transisi |

Constraint: `UQ (order_kind, from_status, to_status)` penuh (bukan partial index), karena sasaran FK dirancang berupa constraint unik (persyaratan ini diverifikasi di Tahap 5 bersama butir §D.4 e); baris yang dirujuk event tidak di-soft delete, dan transisi yang dicabut ditandai soft delete sehingga pustaka transisi menolak transisi baru. Hak role aplikasi: S saja (pengecualian dari kelas Master, karena isinya aturan PRD).

### A9. Konsultasi

#### A9.1 `consultation_session` (T)

Detail pesanan konsultasi, dibuat saat checkout. Satu sesi punya dua wadah data: `consultation_chat` dan `consultation_clinical` `[KEPUTUSAN-68]` (ADR-013 §3). Sumber: BR-010–BR-013, US-304, US-305, US-306, §3.5.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-T* | | | | | |
| patient_order_id | uuid | tidak | FK→patient_order, UQ | — | |
| health_worker_id | uuid | tidak | FK→health_worker | — | Profesi tenaga kesehatan dapat mengungkap jenis layanan kesehatan (temuan F-09) |
| consultation_slot_id | uuid | tidak | FK→consultation_slot | — | |
| consultation_mode | text | tidak | | — | CHECK (`chat`, `video`) |
| scheduled_start_at | timestamptz | tidak | | — | |
| scheduled_end_at | timestamptz | tidak | | — | CHECK `> scheduled_start_at` |
| duration_minutes | integer | tidak | | — | Dari tarif terkunci |
| chief_complaint_enc | bytea | ya | | Spesifik, Enc | Keluhan awal, maksimal 500 karakter divalidasi sebelum enkripsi (US-304 AC1). Akses tenaga kesehatan dicatat E-05 (US-311 AC4) |
| chat_data_key_id | uuid | tidak | FK→data_key, UQ | — | Wadah `consultation_chat` (keluhan awal, chat, lampiran). FK komposit `(chat_data_key_id, id)` → `data_key (id, container_id)` |
| clinical_data_key_id | uuid | tidak | FK→data_key, UQ | — | Wadah `consultation_clinical` (catatan dan resep). FK komposit `(clinical_data_key_id, id)` → `data_key (id, container_id)` |

Index: `ix (health_worker_id, scheduled_start_at)` (daftar sesi, US-311 AC1); `ix (scheduled_start_at)` (jaring pengaman harian batas ketidakhadiran PB-02/PB-03, review DBA S-N1).

#### A9.2 `consultation_attendance_event` (T)

Waktu bergabung dan keluar dicatat server saat klien membuka ruang sesi (§3.5, UP-29). "Bergabung pertama" = baris `joined` paling awal per peran. Sumber: US-305 AC1, US-306 AC1, US-307, ADR-016 butir 3.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-T* | | | | | `created_at` = jam server (bukan jam perangkat) |
| consultation_session_id | uuid | tidak | FK→consultation_session | — | |
| participant_role | text | tidak | | — | CHECK (`patient`, `health_worker`) |
| account_id | uuid | tidak | FK→account | — | |
| event_type | text | tidak | | — | CHECK (`joined`, `left`) |

Index: `ix (consultation_session_id, participant_role, created_at)`.

#### A9.3 `video_room` (O)

Ruang video per sesi di server media (ADR-016 butir 4). Token bergabung tidak disimpan. Sumber: US-306, AB-74.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-O* | | | | | `expires_at` = akhir durasi sesi |
| consultation_session_id | uuid | tidak | FK→consultation_session, UQ | — | |
| provider_room_ref | text | tidak | | — | |
| recording_disabled_verified | boolean | tidak | | — | Hasil pemeriksaan konfigurasi ruang tanpa rekaman (ADR-016 butir 4b) |
| closed_at | timestamptz | ya | | — | |

#### A9.4 `chat_message` (T)

Pesan chat. ID dibuat aplikasi klien (UUIDv7) sehingga kiriman ulang tidak menggandakan pesan (KU-05, ADR-016 butir 2); server memvalidasi bentuk ID dan menolak ID yang sudah ada milik pengirim lain. Sumber: BR-016, US-305, NFR-PERF-03.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| id | uuid | tidak | PK | — | Dari klien |
| created_at | timestamptz | tidak | | — | Jam server saat tersimpan |
| created_by | uuid | tidak | FK→account | — | Pengirim |
| consultation_session_id | uuid | tidak | FK→consultation_session | — | |
| sender_role | text | tidak | | — | CHECK (`patient`, `health_worker`) |
| body_enc | bytea | ya | | Spesifik, Enc | `NULL` bila pesan hanya lampiran |
| data_key_id | uuid | tidak | FK→data_key | — | Wadah `consultation_chat` sesi ini; FK komposit `(data_key_id, consultation_session_id)` → `data_key (id, container_id)` |

Index: `ix (consultation_session_id, created_at)`. Pesan setelah durasi habis ditolak aplikasi (US-305 AC4).

#### A9.5 `chat_attachment` (T)

Foto lampiran chat (ADR-018). Sumber: US-305 AC2, KEPUTUSAN-08.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-T* | | | | | |
| chat_message_id | uuid | tidak | FK→chat_message | — | |
| file_id | uuid | tidak | FK→stored_file, UQ | Spesifik | Bucket `clinical`; `stored_file.container_type = 'consultation_chat'` |

Index: `ix (chat_message_id)`.

#### A9.6 `consultation_note_draft` (O)

Draf catatan yang belum difinalkan; bukan bukti sampai difinalkan sebagai versi (ADR-008 butir 5). Sumber: US-308.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-O* | | | | | Dihapus setelah difinalkan |
| consultation_session_id | uuid | tidak | FK→consultation_session, UQ | — | |
| author_health_worker_id | uuid | tidak | FK→health_worker | — | |
| payload_enc | bytea | tidak | | Spesifik, Enc | Struktur sama dengan versi |
| payload_schema_version | smallint | tidak | | — | |
| data_key_id | uuid | tidak | FK→data_key | — | Wadah `consultation_clinical`; FK komposit `(data_key_id, consultation_session_id)` → `data_key (id, container_id)` |
| updated_at | timestamptz | tidak | | — | |

#### A9.7 `consultation_note_version` (T)

Versi catatan konsultasi terstruktur dan berkode, selaras dengan rekam medis RME (AB-72, UP-16, ADR-020 butir 8). Isi klinis disimpan sebagai satu dokumen JSON terenkripsi per versi `[KEPUTUSAN-52]`. Sumber: BR-013, BR-114, US-308, E-06.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-T* | | | | | |
| consultation_session_id | uuid | tidak | FK→consultation_session | — | |
| version | integer | tidak | | — | CHECK `>= 1` |
| author_health_worker_id | uuid | tidak | FK→health_worker | — | Hanya penulis yang dapat membuat versi baru (US-308 AC3) |
| payload_enc | bytea | tidak | | Spesifik, Enc | Lihat struktur di bawah |
| payload_schema_version | smallint | tidak | | — | |
| change_reason_enc | bytea | ya | | Spesifik, Enc | CHECK `version = 1 OR change_reason_enc IS NOT NULL` (US-308 AC2) |
| data_key_id | uuid | tidak | FK→data_key | — | Wadah `consultation_clinical`; FK komposit `(data_key_id, consultation_session_id)` → `data_key (id, container_id)` |

Index: `UQ (consultation_session_id, version)`.

Struktur `payload` (sebelum enkripsi): `anamnesis_text`, `physical_findings_text`, `diagnoses[]` berisi `{system, code, display}` dengan `system = icd10_2010` (`[SRC-61]` via ADR-020 butir 5), `impression_text`, `plan_text`. Status catatan ini sebagai rekam medis dan masa simpannya `[TIDAK DIKETAHUI]` (L-05, US-308 AC4).

#### A9.8 `prescription` (T)

Resep digital bernomor unik; tidak dapat diubah, dikoreksi dengan membatalkan lalu menerbitkan resep baru (UP-42). Identitas pasien, nama dokter, nomor SIP, klinik, dan tanggal disimpan terenkripsi di `header_enc` `[KEPUTUSAN-43]`. Sumber: BR-015, US-309, AB-53, E-06, E-12.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-T* | | | | | `created_at` = tanggal terbit |
| consultation_session_id | uuid | tidak | FK→consultation_session | — | |
| prescription_number | text | tidak | UQ | — | |
| prescriber_health_worker_id | uuid | tidak | FK→health_worker | — | Berprofesi `doctor` (AB-53, diperiksa aplikasi) |
| header_enc | bytea | tidak | | PII, Spesifik, Enc | Isi kepala resep (US-309 AC1) |
| replaces_prescription_id | uuid | ya | FK→prescription | — | Resep yang dibatalkan dan digantikan (US-309 AC3) |
| verification_code_enc | bytea | tidak | | Enc | Kode verifikasi acak yang dicetak di PDF dan QR (US-309 AC6, CR-008 `[USER] T4-Q5`); terenkripsi dengan DEK wadah `consultation_clinical` |
| verification_code_bidx | bytea | tidak | UQ | — | Blind index HMAC kode verifikasi untuk pencarian dari halaman verifikasi publik tanpa mendekripsi (ADR-013 butir 4 pola sama) |
| verification_code_bidx_key_version | smallint | tidak | | — | Versi kunci HMAC yang dipakai, agar kode yang sudah tercetak tetap dapat diverifikasi setelah rotasi kunci (pola NIK; catatan DBA putaran 3) |
| data_key_id | uuid | tidak | FK→data_key | — | Wadah `consultation_clinical`; FK komposit `(data_key_id, consultation_session_id)` → `data_key (id, container_id)` |

Index: `ix (consultation_session_id)`; `UQ (id, data_key_id)` (sasaran FK wadah dari `prescription_item`). PDF resep dibuat setelah baris resep tersimpan, sehingga dicatat di tabel anak `prescription_document` (A9.12), bukan kolom yang diisi belakangan (review DBA B07). Format, tanda tangan elektronik, dan kewajiban integrasi resep elektronik `[TIDAK DIKETAHUI]` (L-05, US-309 AC5). Halaman verifikasi publik (US-309 AC6–AC8, CR-008) menampilkan status Terbit atau Dibatalkan sesuai AC7. Bila wadah `consultation_clinical` resep sudah dimusnahkan (crypto-shredding), halaman menampilkan "resep tidak ditemukan" walaupun `verification_code_bidx` masih ada (catatan DBA putaran 3).

#### A9.9 `prescription_item` (T)

Item resep. Sumber: US-309 AC1.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-T* | | | | | |
| prescription_id | uuid | tidak | FK→prescription | — | |
| line_no | smallint | tidak | | — | |
| item_enc | bytea | tidak | | Spesifik, Enc | Nama obat, kekuatan/dosis, jumlah, aturan pakai; kode obat KFA `[TIDAK DIKETAHUI]` (ADR-020 butir 7) |
| data_key_id | uuid | tidak | FK→data_key | — | FK komposit `(prescription_id, data_key_id)` → `prescription (id, data_key_id)` (review DBA S-N4) |

Index: `UQ (prescription_id, line_no)`.

#### A9.10 `prescription_status_event` (R)

*STD-R* dengan `prescription_id`. `to_status` CHECK (`issued`, `cancelled`). CHECK `to_status <> 'cancelled' OR reason_code IS NOT NULL` (alasan pembatalan, US-309 AC3).

#### A9.11 `prescription_state` (P)

*STD-P* dengan `prescription_id`.

#### A9.12 `prescription_document` (T)

PDF resep untuk ditunjukkan di apotek (US-309 AC4), dibuat setelah resep terbit (review DBA B07). Sumber: US-309, E-12.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-T* | | | | | |
| prescription_id | uuid | tidak | FK→prescription, UQ | — | |
| file_id | uuid | tidak | FK→stored_file, UQ | PII, Spesifik | Bucket `clinical`; `stored_file.container_type = 'consultation_clinical'` |

### A10. Klinik, lab, home visit, dan hasil lab

#### A10.1 `appointment` (T)

Detail pesanan pendaftaran klinik dan lab, termasuk kode check-in. Sumber: BR-020, BR-021, BR-024, US-402, US-403, US-404, UP-14.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-T* | | | | | |
| patient_order_id | uuid | tidak | FK→patient_order, UQ | — | Jenis `clinic_registration` atau `lab_registration` |
| partner_service_id | uuid | tidak | FK→partner_service | — | |
| partner_slot_id | uuid | ya | FK→partner_slot | — | |
| service_name_snapshot | text | tidak | | — | |
| scheduled_start_at | timestamptz | tidak | | — | |
| scheduled_end_at | timestamptz | tidak | | — | |
| time_zone | text | tidak | | — | Zona rekanan (AB-40) |
| check_in_code | text | tidak | UQ | — | Kode dan QR check-in (US-403 AC1). Keberlakuannya dari status pesanan (US-403 AC3, US-404 AC4) |

Index: `ix (scheduled_start_at)` untuk job RF-18 dan ketidakhadiran; `ix (partner_service_id, scheduled_start_at)` (laporan dan antrean per layanan).

#### A10.2 `appointment_queue_number` (T)

Nomor antrean pasien dari sistem rekanan; baris terbaru per pesanan berlaku. Sumber: BR-027, US-405, ADR-016 butir 6.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-T* | | | | | |
| patient_order_id | uuid | tidak | FK→patient_order | — | |
| partner_service_id | uuid | tidak | FK→partner_service | — | |
| queue_date | date | tidak | | — | |
| queue_number | integer | tidak | | — | CHECK `> 0` |

Index: `ix (patient_order_id, created_at DESC)`.

#### A10.3 `queue_board` (O)

Nomor yang sedang dilayani per layanan per hari, diperbarui dari pesan rekanan (US-405 AC1). Bukan bukti bisnis; notifikasi PB-10 dibuat dari perbandingan dengan `appointment_queue_number`. Sumber: ADR-016 butir 6, UP-12.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-O* | | | | | |
| partner_id | uuid | tidak | FK→partner | — | |
| partner_service_id | uuid | tidak | FK→partner_service | — | |
| queue_date | date | tidak | | — | |
| now_serving_number | integer | tidak | | — | |
| source_updated_at | timestamptz | tidak | | — | Waktu dari rekanan, untuk mengukur keterlambatan 30 detik |
| updated_at | timestamptz | tidak | | — | |

Index: `UQ (partner_service_id, queue_date)`.

#### A10.4 `home_visit_detail` (T)

Detail pesanan home visit. Alamat dan nomor HP ada di `order_personal_snapshot`. Sumber: BR-022, US-501, UP-13, UP-22.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-T* | | | | | |
| patient_order_id | uuid | tidak | FK→patient_order, UQ | — | |
| partner_service_id | uuid | tidak | FK→partner_service | — | |
| partner_slot_id | uuid | ya | FK→partner_slot | — | |
| service_name_snapshot | text | tidak | | — | |
| scheduled_start_at | timestamptz | tidak | | — | |
| scheduled_end_at | timestamptz | tidak | | — | |
| time_zone | text | tidak | | — | Zona klinik (AB-40) |

Index: `ix (scheduled_start_at)` (jaring pengaman RF-18 dan PB-03); `ix (scheduled_end_at)` (jaring pengaman PB-15 dan penutupan akhir hari) (review DBA S-N1).

#### A10.5 `home_visit_assignment` (T)

Penugasan tenaga kesehatan oleh sistem klinik; penugasan ulang = baris baru. Penugasan yang ditolak (bukan tenaga kesehatan aktif klinik itu, US-502 AC2) tidak disimpan dan dicatat di `integration_error_log`. Sumber: US-502, US-503, UP-13.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-T* | | | | | |
| patient_order_id | uuid | tidak | FK→patient_order | — | |
| health_worker_id | uuid | tidak | FK→health_worker | — | |
| partner_assignment_ref | text | ya | | — | |

Index: `ix (patient_order_id, created_at DESC)`; `ix (health_worker_id, created_at)`.

#### A10.6 `home_visit_assignment_release` (T)

Berakhirnya suatu penugasan. Penugasan aktif = penugasan tanpa baris release; paling banyak satu per pesanan, dijaga pustaka transisi. Sumber: §3.6 home visit, US-204 AC3, US-207 AC2, US-502 AC3.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-T* | | | | | |
| home_visit_assignment_id | uuid | tidak | FK→home_visit_assignment, UQ | — | |
| release_reason | text | tidak | | — | CHECK (`reassigned`, `health_worker_deactivated`, `sip_expired`, `order_ended`) |

#### A10.7 `lab_result` (T)

Satu hasil lab untuk pendaftaran lab; satu pendaftaran dapat punya beberapa hasil (PRD §8.3). Satu hasil = satu wadah `lab_result`. Sumber: BR-025, US-901, US-902.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-T* | | | | | |
| patient_order_id | uuid | tidak | FK→patient_order | — | Jenis `lab_registration` milik rekanan pengirim (US-901 AC2) |
| external_result_id | text | tidak | | — | Untuk RME = `rme_lab_result.id` (C4.4) |
| data_key_id | uuid | tidak | FK→data_key, UQ | — | FK komposit `(data_key_id, id)` → `data_key (id, container_id)` |

Index: `UQ (patient_order_id, external_result_id)`.

#### A10.8 `lab_result_version` (T)

Versi hasil lab; koreksi = versi baru, dan versi lama terlihat "sudah dikoreksi" karena ada versi lebih tinggi (US-901 AC3). Sumber: US-901, US-1307, E-06, `[KEPUTUSAN-52]`.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-T* | | | | | |
| lab_result_id | uuid | tidak | FK→lab_result | — | |
| version | integer | tidak | | — | CHECK `>= 1` |
| payload_enc | bytea | ya | | Spesifik, Enc | `tests[]` berisi `{code_system, code, display, value, unit, reference_range}`; LOINC bila dikirim rekanan (ADR-020 butir 5) |
| pdf_file_id | uuid | ya | FK→stored_file | Spesifik | Bucket `clinical` |
| correction_reason_enc | bytea | ya | | Spesifik, Enc | CHECK `version = 1 OR correction_reason_enc IS NOT NULL` |
| payload_schema_version | smallint | tidak | | — | |

CHECK: `payload_enc IS NOT NULL OR pdf_file_id IS NOT NULL` (US-901 AC1). Index: `UQ (lab_result_id, version)`.

### A11. Pesanan produk dan pengiriman

#### A11.1 `product_order_detail` (T)

Detail pesanan produk; satu checkout untuk satu outlet (UP-09). Sumber: BR-034, US-703.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-T* | | | | | |
| patient_order_id | uuid | tidak | FK→patient_order, UQ | — | |
| courier_code | text | tidak | | — | Kode kurir dan layanannya dari penyedia terpilih `[TIDAK DIKETAHUI]` (Tahap 5) |
| courier_service_code | text | tidak | | — | |

#### A11.2 (dihapus: `product_order_item`)

Tabel ini dihapus di putaran perbaikan 1 (review DBA S04). Barang, jumlah, dan nama produk saat dikunci dibaca dari `order_price_lock_line` milik harga terkunci percobaan yang tertangkap (A8.7). Nomor bagian dipertahankan supaya rujukan lain tidak bergeser.

#### A11.3 `shipment` (T)

Pengiriman setelah barang diserahkan ke kurir. Sumber: US-704 AC3.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-T* | | | | | |
| patient_order_id | uuid | tidak | FK→patient_order, UQ | — | |
| courier_code | text | tidak | | — | |
| tracking_number | text | tidak | | — | Nomor resi |

#### A11.4 `shipment_tracking_event` (L)

Riwayat pelacakan dari kurir. Sumber: US-704 AC3, AC4.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-L* | | | | | |
| shipment_id | uuid | tidak | FK→shipment | — | |
| courier_status_code | text | tidak | | — | Kode status dari kurir |
| location_code | text | ya | | — | Kode lokasi atau hub dari kurir |
| description_enc | bytea | ya | | PII, Enc | Deskripsi bebas dari kurir (dapat memuat nama penerima), terenkripsi dengan wadah `status_note` pesanan (review DBA S09, `[KEPUTUSAN-42]`) |
| data_key_id | uuid | ya | FK→data_key | — | CHECK `(description_enc IS NULL) = (data_key_id IS NULL)` |
| occurred_at | timestamptz | tidak | | — | Waktu dari kurir |

Index: `ix (shipment_id, occurred_at)`.

### A12. Pembayaran

#### A12.1 `payment_attempt` (T)

Satu percobaan pembayaran di payment gateway, dengan harga terkunci sendiri (`[KEPUTUSAN-48]`). Baris ini (beserta `idempotency_key`) **dibuat sebelum** memanggil penyedia, sehingga panggilan yang diulang setelah gangguan memakai kunci yang sama; referensi dari penyedia dicatat di tabel anak `payment_attempt_gateway_ref` (review DBA B07). Pasien dapat mencoba metode lain selama PB-07 belum habis (US-802 AC4). Sumber: BR-040, BR-115, US-802, ADR-015 butir 2.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-T* | | | | | |
| patient_order_id | uuid | tidak | FK→patient_order | — | |
| order_price_lock_id | uuid | tidak | FK→order_price_lock, UQ | — | FK komposit `(order_price_lock_id, patient_order_id)` → `order_price_lock (id, patient_order_id)` (review DBA S04) |
| payment_method_code | text | tidak | | — | |
| amount_idr | bigint | tidak | | Keu | CHECK `> 0`; sama dengan `total_idr` harga terkunci (US-802 AC1) |
| idempotency_key | uuid | tidak | UQ | — | Dikirim ke penyedia |
| expires_at | timestamptz | tidak | | — | PB-07 terkunci |

Index: `ix (patient_order_id)`; `ix (expires_at)` (jaring pengaman kedaluwarsa PB-07, review DBA S-N1); `UQ (id, patient_order_id)` (sasaran FK komposit dari `payment_receipt`).

#### A12.1a `payment_attempt_gateway_ref` (T)

Referensi pembayaran dari penyedia setelah pembuatan berhasil (review DBA B07). Percobaan tanpa baris ini = pembuatan gagal atau belum dijawab penyedia. Sumber: ADR-015 butir 2.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-T* | | | | | |
| payment_attempt_id | uuid | tidak | FK→payment_attempt, UQ | — | |
| gateway_payment_ref | text | tidak | UQ | — | Dipakai mencocokkan notifikasi penyedia |

#### A12.2 `payment_attempt_status_event` (R)

*STD-R* dengan `payment_attempt_id`. `to_status` CHECK (`pending`, `succeeded`, `failed`, `expired`, `cancelled`). Pembatalan instruksi saat berganti metode (US-802 AC4) = `cancelled`.

#### A12.3 `payment_attempt_state` (P)

*STD-P* dengan `payment_attempt_id`.

#### A12.4 `payment_gateway_notification` (L)

Notifikasi payment gateway yang **sudah terverifikasi** keasliannya (NFR-INT-04). Notifikasi dengan tanda tangan tidak valid tidak disimpan di sini, tetapi dicatat di security log (ADR-010). Isi disimpan tanpa rahasia dan tanpa data pembayar `[KEPUTUSAN-42]`; daftar field yang diizinkan ditetapkan bersama penyedia di Tahap 5. Sumber: ADR-015 butir 5, US-802 AC2.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-L* | | | | | `created_at` = waktu diterima |
| gateway_event_id | text | tidak | UQ | — | Notifikasi berulang hanya diproses sekali (US-802 AC2) |
| event_type | text | tidak | | — | |
| gateway_payment_ref | text | ya | | — | |
| amount_idr | bigint | ya | | Keu | |
| payload_sanitized | jsonb | tidak | | — | Field dari daftar izin saja |

Index: `ix (gateway_payment_ref)`. Pemrosesan dilakukan job; status pemrosesan ada di `job`. Satu notifikasi dapat menghasilkan paling banyak satu penerimaan; identitas transaksi penyedia disalin ke `payment_receipt.gateway_transaction_ref`.

#### A12.5 `payment_receipt` (T)

Setiap dana yang diterima dari penyedia, beserta hasil penilaiannya `[KEPUTUSAN-49]`. Satu percobaan dapat menerima lebih dari satu penerimaan (debit ganda pada instruksi yang sama); kunci idempotensinya adalah ID transaksi penyedia, dan penerimaan kedua pada percobaan yang sama dinilai `duplicate` (RF-12) (review DBA B06). Refund selalu merujuk penerimaan yang dikembalikan. Sumber: US-802 AC2, AC3, AC5, AC6, RF-12, RF-13, RF-14.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-T* | | | | | |
| payment_attempt_id | uuid | tidak | FK→payment_attempt | — | FK komposit `(payment_attempt_id, patient_order_id)` → `payment_attempt (id, patient_order_id)` |
| gateway_transaction_ref | text | tidak | UQ | — | ID transaksi penyedia; notifikasi atau rekonsiliasi yang sama tidak membuat penerimaan kedua |
| patient_order_id | uuid | tidak | FK→patient_order | — | |
| payment_gateway_notification_id | uuid | ya | FK→payment_gateway_notification, UQp | — | `NULL` bila terdeteksi di rekonsiliasi (RF-12) |
| amount_received_idr | bigint | tidak | | Keu | CHECK `> 0` |
| received_at | timestamptz | tidak | | — | Waktu menurut penyedia |
| outcome | text | tidak | | — | CHECK (`captured`, `duplicate`, `amount_mismatch`, `late_unavailable`): dikonfirmasi; pembayaran ganda RF-12; nominal berbeda RF-14; berhasil setelah kedaluwarsa tanpa slot/stok RF-13 |

Index: `ix (payment_attempt_id)`; `ix (patient_order_id)`; `UQp (payment_gateway_notification_id) WHERE payment_gateway_notification_id IS NOT NULL` (satu notifikasi paling banyak satu penerimaan, review DBA S-N5); `ix (received_at)` (rekonsiliasi harian, review DBA S-N1); `ix (outcome) WHERE outcome <> 'captured'` (antrean Keuangan, US-802 AC3); `UQ (id, payment_attempt_id)` dan `UQ (id, patient_order_id)` (sasaran FK komposit dari `payment_capture`).

#### A12.6 `payment_capture` (T)

Penerimaan dana yang mengonfirmasi pesanan. Constraint unik per pesanan membuat tangkapan kedua gagal dan dialihkan ke RF-12 (ADR-015 butir 5, US-802 AC6). Sumber: PRD §8.3 (satu pembayaran berhasil per pesanan).

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-T* | | | | | |
| patient_order_id | uuid | tidak | FK→patient_order, UQ | — | FK komposit `(payment_receipt_id, patient_order_id)` → `payment_receipt (id, patient_order_id)` |
| payment_receipt_id | uuid | tidak | FK→payment_receipt, UQ | — | Penerimaan berhasil `outcome = 'captured'` |
| payment_attempt_id | uuid | tidak | FK→payment_attempt | — | FK komposit `(payment_receipt_id, payment_attempt_id)` → `payment_receipt (id, payment_attempt_id)` |
| amount_idr | bigint | tidak | | Keu | CHECK `> 0` |

Rantai FK komposit `payment_capture` → `payment_receipt` → `payment_attempt` → `order_price_lock` menjamin tangkapan, penerimaan, percobaan, dan harga terkunci semuanya milik pesanan yang sama (review DBA S04). Index: `ix (created_at)` (laporan periode dan rekonsiliasi, review DBA S-N1).

#### A12.7 `fund_split_instruction_line` (T)

Instruksi pembagian dana yang dikirim bersama pembuatan pembayaran (ADR-015 butir 2). Satu set per harga terkunci; set yang berlaku = milik percobaan yang tertangkap. Sumber: BR-041, AB-21, AB-22, AB-23, US-803 AC1.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-T* | | | | | |
| order_price_lock_id | uuid | tidak | FK→order_price_lock | — | |
| line_no | smallint | tidak | | — | |
| recipient_type | text | tidak | | — | CHECK (`partner`, `platform`, `shipping_recipient`). Penerima dan cara penyaluran ongkos kirim `[TIDAK DIKETAHUI]` (PT-31, AB-23) |
| partner_bank_account_id | uuid | ya | FK→partner_bank_account | — | CHECK `recipient_type <> 'partner' OR partner_bank_account_id IS NOT NULL` |
| amount_idr | bigint | tidak | | Keu | CHECK `>= 0` |

Index: `UQ (order_price_lock_id, line_no)`. Jumlah `amount_idr` per harga terkunci harus sama dengan `total_idr` dikurangi bagian yang tidak dibagi; kontrol ini diperiksa dengan mekanisme yang sama seperti keseimbangan jurnal (§D.4, review DBA S03).

#### A12.8 `fund_split_report_line` (L)

Hasil pembagian dana dan biaya aktual yang dilaporkan penyedia. Sumber: US-803 AC2, US-811, ADR-015 butir 8–9.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-L* | | | | | |
| payment_capture_id | uuid | tidak | FK→payment_capture | — | |
| recipient_type | text | tidak | | — | CHECK sama dengan instruksi |
| partner_id | uuid | ya | FK→partner | — | |
| amount_idr | bigint | tidak | | Keu | |
| fee_idr | bigint | ya | | Keu | MDR atau biaya aktual |
| gateway_report_ref | text | tidak | | — | |
| reported_at | timestamptz | tidak | | — | |

Index: `ix (payment_capture_id)`; `ix (reported_at)` (rekonsiliasi, review DBA S-N1); `UQ (gateway_report_ref, payment_capture_id, recipient_type)` supaya laporan yang diimpor ulang tidak tercatat dua kali.

### A13. Jurnal berpasangan

#### A13.1 `journal_account` (M)

Bagan akun jurnal. Sumber: ADR-015 butir 4, AB-26, AB-34.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-M* | | | | | Soft delete mencegah pemakaian baru; baris lama tetap dirujuk |
| code | text | tidak | UQp | — | |
| name | text | tidak | | — | |
| account_class | text | tidak | | — | CHECK (`asset`, `liability`, `equity`, `revenue`, `expense`) |
| normal_side | text | tidak | | — | CHECK (`debit`, `credit`) |
| purpose_code | text | tidak | | — | CHECK (`patient_funds`, `partner_share`, `margin_revenue`, `gateway_fee`, `partner_refund_obligation`, `platform_refund_obligation`, `shipping_funds`). Akun kurir (PT-29) dan pajak (PT-28) `[TIDAK DIKETAHUI]`, dapat ditambah tanpa ubah struktur (ADR-015) |
| partner_id | uuid | ya | FK→partner | — | Sub-akun per rekanan |

Index: `UQp (code) WHERE deleted_at IS NULL`; dua partial unique index supaya `NULL` di `partner_id` tidak meloloskan duplikat akun platform (review DBA S03): `UQp (purpose_code) WHERE partner_id IS NULL AND deleted_at IS NULL` (akun platform) dan `UQp (purpose_code, partner_id) WHERE partner_id IS NOT NULL AND deleted_at IS NULL` (sub-akun rekanan).

#### A13.2 `journal_entry` (T)

Kepala jurnal untuk setiap kejadian uang. Koreksi selalu jurnal pembalik yang merujuk jurnal asal (ADR-004, AB-31). Sumber: ADR-015 butir 4, US-803 AC3, US-808 AC1, E-08.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-T* | | | | | |
| entry_type | text | tidak | | — | CHECK (`payment_received`, `split_reported`, `refund_created`, `refund_succeeded`, `partner_obligation_recorded`, `partner_obligation_settled`, `platform_obligation_recorded`, `reversal`) |
| patient_order_id | uuid | ya | FK→patient_order | — | |
| payment_capture_id | uuid | ya | FK→payment_capture | — | |
| refund_id | uuid | ya | FK→refund | — | |
| partner_obligation_id | uuid | ya | FK→partner_obligation | — | |
| reverses_journal_entry_id | uuid | ya | FK→journal_entry, UQp | — | CHECK `(entry_type = 'reversal') = (reverses_journal_entry_id IS NOT NULL)` |
| occurred_at | timestamptz | tidak | | — | Waktu kejadian uang |
| description | text | tidak | | — | Tanpa data pribadi |
| total_debit_idr | bigint | tidak | | Keu | |
| total_credit_idr | bigint | tidak | | Keu | CHECK `total_debit_idr = total_credit_idr AND total_debit_idr > 0` |
| idempotency_key | text | tidak | UQ | — | Job yang diulang tidak membuat jurnal ganda |

Index: `UQp (reverses_journal_entry_id) WHERE reverses_journal_entry_id IS NOT NULL` (satu pembalik per jurnal); `ix (patient_order_id)`; `ix (refund_id) WHERE refund_id IS NOT NULL`; `ix (payment_capture_id) WHERE payment_capture_id IS NOT NULL`; `ix (partner_obligation_id) WHERE partner_obligation_id IS NOT NULL`; `ix (occurred_at)`.

#### A13.3 `journal_line` (T)

Baris debit dan kredit. Sumber: ADR-015 butir 4.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-T* | | | | | |
| journal_entry_id | uuid | tidak | FK→journal_entry | — | |
| line_no | smallint | tidak | | — | |
| journal_account_id | uuid | tidak | FK→journal_account | — | |
| side | text | tidak | | — | CHECK (`debit`, `credit`) |
| amount_idr | bigint | tidak | | Keu | CHECK `> 0` |
| occurred_at | timestamptz | tidak | | — | Salinan `journal_entry.occurred_at`, supaya laporan periode per akun tidak perlu join; FK komposit `(journal_entry_id, occurred_at)` → `journal_entry (id, occurred_at)` menjamin nilainya sama (review DBA S-N1) |

Index: `UQ (journal_entry_id, line_no)`; `ix (journal_account_id, occurred_at)` (laporan rekanan dan keuangan per periode kejadian, US-811, US-1201). Di `journal_entry` ditambahkan `UQ (id, occurred_at)` sebagai sasaran FK komposit.

**Constraint keseimbangan jurnal.** Tiga lapis: (1) CHECK di kepala `total_debit_idr = total_credit_idr`; (2) jumlah baris per sisi harus sama dengan total kepala, diperiksa di database saat commit transaksi; (3) job harian membandingkan semua jurnal. Mekanisme lapis 2 (trigger yang dijalankan saat commit) diverifikasi ke dokumentasi PostgreSQL di Tahap 5 sebelum dipakai; SRC-40 baru memverifikasi perilaku trigger umum. Sampai terverifikasi, lapis 2 ditegakkan pustaka jurnal yang menulis kepala dan baris dalam satu transaksi.

### A14. Refund, klaim, kewajiban, tagihan, dan rekonsiliasi

#### A14.1 `refund` (T)

Refund ke pasien. Sumber: BR-045, §3.5 (RF-01–RF-18), AB-30–AB-36, US-806, US-808, ADR-015 butir 6.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-T* | | | | | `created_by` = Keuangan untuk RF-14 manual; `NULL` untuk otomatis |
| patient_order_id | uuid | tidak | FK→patient_order | — | |
| payment_receipt_id | uuid | tidak | FK→payment_receipt, UQ | — | Satu refund per penerimaan dana |
| refund_case | text | tidak | | — | CHECK (`rf_01`, `rf_04`, `rf_05`, `rf_06`, `rf_07`, `rf_08`, `rf_09`, `rf_10`, `rf_12`, `rf_13`, `rf_14`, `rf_15`, `rf_16`, `rf_17`, `rf_18`) |
| liability_group | text | tidak | | — | CHECK (`patient_cancellation`, `partner_cause`, `platform_cause`, `courier_cause`, `pending_determination`) (AB-34). CHECK `(refund_case = 'rf_18') = (liability_group = 'pending_determination')` |
| claim_id | uuid | ya | FK→claim, UQp | — | CHECK `refund_case NOT IN ('rf_06','rf_07','rf_10','rf_16','rf_17') OR claim_id IS NOT NULL` |
| amount_idr | bigint | tidak | | Keu | CHECK `> 0`. RF-01: harga layanan tanpa biaya transaksi non-QRIS (US-806 AC3) |

Index dan constraint:
- `UQp (patient_order_id) WHERE refund_case NOT IN ('rf_12','rf_13','rf_14')` `[SRC-33]`: satu refund per pesanan untuk refund atas pembayaran pesanan (AB-36, ADR-015 butir 6). RF-13 dan RF-14 dikecualikan karena keduanya refund atas dana yang tidak menjadi pembayaran pesanan, sehingga refund wajib sesudahnya (misalnya RF-01 setelah pesanan dikonfirmasi oleh pembayaran berikutnya) tidak terblokir `[KEPUTUSAN-67]`. Tafsiran AB-36 ini menunggu konfirmasi user di gate Tahap 4.
- `UQ (payment_receipt_id)` tetap: satu refund per penerimaan dana.
- `ix (patient_order_id)` penuh (bukan partial), untuk semua refund termasuk RF-12–RF-14 di laporan dan halaman pesanan (review DBA S-N1).
- `UQp (claim_id) WHERE claim_id IS NOT NULL`.
- Klaim sebagian barang `[TIDAK DIKETAHUI]` (PT-33): bila refund sebanding diputuskan, `amount_idr` sudah menampungnya tanpa ubah struktur.

#### A14.2 `refund_status_event` (R)

*STD-R* dengan `refund_id`. `to_status` CHECK (`processing`, `succeeded`, `failed`, `needs_review`). Transisi §3.6 (termasuk `failed → processing` oleh Keuangan, AB-32) ditegakkan pustaka transisi.

#### A14.3 `refund_state` (P)

*STD-P* dengan `refund_id`. Index `ix (status, updated_at) WHERE status IN ('processing','failed','needs_review')` untuk job PB-19 dan antrean Keuangan (AB-35).

#### A14.4 `refund_execution_attempt` (L)

Setiap pemanggilan refund ke penyedia; kunci idempotensi = `refund.id` (ADR-015 butir 6). Sumber: US-808 AC1–AC2, AC4.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-L* | | | | | |
| refund_id | uuid | tidak | FK→refund | — | |
| attempt_no | smallint | tidak | | — | |
| gateway_refund_ref | text | ya | | — | |
| outcome | text | tidak | | — | CHECK (`accepted`, `succeeded`, `failed`, `unknown`) |
| error_code | text | ya | | — | |

Index: `UQ (refund_id, attempt_no)`.

#### A14.5 `refund_cause_determination` (T)

Penetapan penyebab RF-18 oleh Keuangan, atau penyebab platform otomatis saat PB-20 habis (US-808 AC6, UP-56). Sumber: RF-18, AB-34.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-T* | | | | | `created_by` = Keuangan; `NULL` untuk PB-20 |
| refund_id | uuid | tidak | FK→refund, UQ | — | Refund `rf_18` (diperiksa aplikasi) |
| determined_cause | text | tidak | | — | CHECK (`partner_cause`, `platform_cause`) |
| determination_type | text | tidak | | — | CHECK (`manual`, `pb20_default`) |
| integration_evidence_ref | text | ya | | — | Rujukan log integrasi; CHECK `determination_type <> 'manual' OR integration_evidence_ref IS NOT NULL` |

#### A14.6 `claim` (T)

Klaim refund. **Satu klaim per pesanan** `[KEPUTUSAN-50]`: klaim yang ditolak tidak dapat diajukan ulang (UP-36), dan klaim yang disetujui menghasilkan satu-satunya refund (AB-36). Sumber: BR-045, US-504, US-705, US-807, AB-33.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-T* | | | | | `created_by` = pasien |
| patient_order_id | uuid | tidak | FK→patient_order, UQ | — | |
| claim_case | text | tidak | | — | CHECK (`rf_06`, `rf_07`, `rf_10`, `rf_16`, `rf_17`) |
| claim_subtype | text | ya | | — | CHECK (`damaged`, `not_as_described`, `missing_items`, `not_received`) untuk produk (US-705 AC2) |
| description_enc | bytea | tidak | | PII, Spesifik, Enc | Deskripsi klaim (ADR-013 §3) |
| data_key_id | uuid | tidak | FK→data_key, UQ | — | Wadah `claim`; FK komposit `(data_key_id, id)` → `data_key (id, container_id)` |
| decision_due_at | timestamptz | tidak | | — | PB-05 hari kerja dari pengajuan (US-807 AC1) |

Index: `ix (decision_due_at)` untuk eskalasi AB-33.

#### A14.7 `claim_status_event` (R)

*STD-R* dengan `claim_id`. `to_status` CHECK (`submitted`, `approved`, `rejected`). CHECK `to_status <> 'rejected' OR (reason_code IS NOT NULL AND note_enc IS NOT NULL)` (alasan wajib, US-807 AC4; `note_enc` memakai wadah `claim`). Aktor Layanan Pelanggan atau Super Admin (klaim tereskalasi).

#### A14.8 `claim_state` (P)

*STD-P* dengan `claim_id`.

#### A14.9 `claim_evidence` (T)

Foto bukti klaim (wajib minimal satu untuk RF-10, diperiksa aplikasi). Membuka foto dicatat E-05. Sumber: US-705 AC2, US-807 AC1, §3.1.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-T* | | | | | |
| claim_id | uuid | tidak | FK→claim | — | |
| file_id | uuid | tidak | FK→stored_file, UQ | Spesifik | Bucket `claim_evidence` |

Index: `ix (claim_id)`.

#### A14.10 `claim_escalation` (T)

Klaim yang melewati PB-05 tanpa keputusan, ditandai terlambat dan dieskalasi ke Super Admin (AB-33, US-807 AC5).

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-T* | | | | | |
| claim_id | uuid | tidak | FK→claim, UQ | — | |

#### A14.11 `partner_obligation` (T)

Kewajiban pengembalian rekanan sesuai AB-34, dihitung dari harga terkunci. Dicatat saat refund dibuat, atau untuk RF-18 saat penyebab ditetapkan `[KEPUTUSAN-51]`. Kewajiban kelompok penyebab kurir `[TIDAK DIKETAHUI]` (PT-29). Sumber: US-806 AC1, US-808 AC3, AC5, UP-35, ADR-015 butir 6.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-T* | | | | | |
| refund_id | uuid | tidak | FK→refund | — | |
| partner_id | uuid | tidak | FK→partner | — | |
| liability_group | text | tidak | | — | CHECK (`patient_cancellation`, `partner_cause`, `platform_cause`) |
| amount_idr | bigint | tidak | | — | CHECK `> 0` |
| refund_cause_determination_id | uuid | ya | FK→refund_cause_determination | — | Terisi untuk RF-18 |
| journal_entry_id | uuid | tidak | FK→journal_entry | — | |

Index: `UQ (refund_id, partner_id)`; `ix (partner_id, created_at)`.

#### A14.12 `partner_obligation_status_event` (R)

*STD-R* dengan `partner_obligation_id`, ditambah kolom `evidence_reference text NULL`. `to_status` CHECK (`open`, `settled`). CHECK `to_status <> 'settled' OR evidence_reference IS NOT NULL` (US-808 AC5).

#### A14.13 `partner_obligation_state` (P)

*STD-P* dengan `partner_obligation_id`.

#### A14.14 `partner_fee_invoice` (T)

Perhitungan tagihan biaya rekanan per bulan; koreksi = revisi baru. Cara penagihan dan penyetoran `[TIDAK DIKETAHUI]` (PT-17, AB-24, US-810 AC3). Sumber: US-810 AC2, US-1201 AC1.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-T* | | | | | |
| partner_id | uuid | tidak | FK→partner | — | |
| period_month | date | tidak | | — | Tanggal 1 bulan itu; CHECK hari = 1 |
| partner_fee_rule_id | uuid | tidak | FK→partner_fee_rule | — | |
| fee_type | text | tidak | | — | |
| registration_count | integer | ya | | — | Per pendaftaran |
| amount_idr | bigint | tidak | | — | CHECK `>= 0` |
| revision_no | smallint | tidak | | — | |
| supersedes_invoice_id | uuid | ya | FK→partner_fee_invoice | — | |

Index: `UQ (partner_id, period_month, revision_no)`.

#### A14.15 `reconciliation_run` (L)

Satu kali rekonsiliasi harian (US-811 AC2, ADR-015 butir 9). Ditulis setelah selesai.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-L* | | | | | |
| run_date | date | tidak | | — | Tanggal transaksi yang dicocokkan |
| provider_report_file_id | uuid | ya | FK→stored_file | — | Bucket `export` |
| started_at | timestamptz | tidak | | — | |
| finished_at | timestamptz | tidak | | — | |
| outcome | text | tidak | | — | CHECK (`completed`, `failed`) |
| item_count | integer | tidak | | — | |

#### A14.16 `reconciliation_item` (L)

Selisih yang ditandai untuk ditindaklanjuti. Sumber: US-811 AC2.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-L* | | | | | |
| reconciliation_run_id | uuid | tidak | FK→reconciliation_run | — | |
| item_type | text | tidak | | — | CHECK (`amount_mismatch`, `unreported_split`, `duplicate_payment`, `missing_at_provider`, `missing_in_ledger`, `fee_difference`) |
| patient_order_id | uuid | ya | FK→patient_order | — | |
| payment_capture_id | uuid | ya | FK→payment_capture | — | |
| expected_idr | bigint | ya | | — | |
| actual_idr | bigint | ya | | — | |
| detail | jsonb | ya | | — | Tanpa data pribadi |

#### A14.17 `reconciliation_item_resolution` (L)

Tindak lanjut selisih oleh Keuangan.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-L* | | | | | |
| reconciliation_item_id | uuid | tidak | FK→reconciliation_item, UQ | — | |
| resolution_code | text | tidak | | — | |
| resolved_by | uuid | tidak | FK→account | — | |

### A15. Notifikasi dan konten

Push notification dan WhatsApp dihapus (CR-007 `[USER] T4-Q2`); tidak ada tabel token perangkat atau WhatsApp.

#### A15.1 `in_app_notification` (L)

Isi kotak notifikasi di dalam aplikasi. Menyimpan kode templat dan parameter, bukan teks jadi `[KEPUTUSAN-42]`. Sumber: BR-050, EP-10 (matriks kanal), US-1001, US-1003, AB-73, ADR-016 butir 5.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-L* | | | | | |
| recipient_account_id | uuid | tidak | FK→account | — | |
| event_type | text | tidak | | — | Kejadian dari matriks EP-10 |
| template_code | text | tidak | | — | |
| params | jsonb | tidak | | — | Hanya ID dan nilai non-pribadi; tanpa data kesehatan rinci (AB-73) |
| target_type | text | ya | | — | Tujuan saat diketuk (misalnya `patient_order`) |
| target_id | uuid | ya | | — | |

Index: `ix (recipient_account_id, created_at DESC)` (urut terbaru, US-1001 AC4).

#### A15.2 `in_app_notification_read` (L)

Penanda sudah dibaca, sebagai baris baru yang tidak diubah (ADR-016 butir 5). Sumber: US-1001 AC4.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-L* | | | | | `created_at` = waktu dibaca |
| in_app_notification_id | uuid | tidak | FK→in_app_notification, UQ | — | |

#### A15.3 `email_delivery` (L)

Setiap percobaan menyerahkan email ke penyedia. Alamat email tidak disimpan; diambil dari akun atau payload outbox saat kirim `[KEPUTUSAN-42]`. Sumber: US-1001 AC1, AC3, US-105 AC4, NFR-PERF-05.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-L* | | | | | |
| recipient_account_id | uuid | ya | FK→account | — | `NULL` untuk kontak rekanan |
| recipient_partner_id | uuid | ya | FK→partner | — | Email ke admin atau kontak rekanan (US-1003 AC3) |
| recipient_kind | text | tidak | | — | CHECK (`account_email`, `previous_email`, `partner_contact`) |
| template_code | text | tidak | | — | |
| notification_outbox_id | uuid | tidak | | — | Tanpa FK, karena baris outbox dihapus setelah selesai |
| attempt_no | smallint | tidak | | — | |
| outcome | text | tidak | | — | CHECK (`handed_over`, `temporary_failure`, `permanent_failure`) |
| provider_message_id | text | ya | | — | |
| error_code | text | ya | | — | |

Index: `ix (recipient_account_id, created_at)`; `ix (notification_outbox_id)`.

#### A15.4 `cms_content` (M)

Banner, halaman info, artikel, dan promo informatif. "Tayang" dihitung dari `editorial_status = 'scheduled'` dan jendela waktu. Sumber: BR-061, US-1101, US-1102.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-M* | | | | | Hapus dan pulihkan dicatat E-16 (US-1101 AC3) |
| content_type | text | tidak | | — | CHECK (`banner`, `info_page`, `article`, `promo`) |
| title | text | tidak | | — | |
| body | text | ya | | — | Markdown; tanpa data pribadi |
| category | text | ya | | — | |
| image_file_id | uuid | ya | FK→stored_file | — | Bucket `public_content` |
| link_target_type | text | ya | | — | CHECK (`article`, `service`, `partner`, `product`) |
| link_target_ref | text | ya | | — | |
| publish_start_at | timestamptz | ya | | — | |
| publish_end_at | timestamptz | ya | | — | CHECK `publish_end_at IS NULL OR publish_end_at > publish_start_at` |
| editorial_status | text | tidak | | — | CHECK (`draft`, `scheduled`, `archived`) |

Index: `ix (content_type, editorial_status, publish_start_at) WHERE deleted_at IS NULL`. Promo tidak mengubah harga (US-1101 AC4).

### A16. Layanan pelanggan

#### A16.1 `complaint_ticket` (T)

Tiket keluhan. Teks tiket dienkripsi per tiket supaya dapat dimusnahkan lewat crypto-shredding `[KEPUTUSAN-43]`. Sumber: BR-062, US-1103, US-1104, US-1203, UP-30.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-T* | | | | | |
| ticket_number | text | tidak | UQ | — | |
| patient_id | uuid | tidak | FK→patient | — | |
| category | text | tidak | | — | CHECK (`service`, `payment`, `account`, `other`) (US-1103 AC1) |
| patient_order_id | uuid | ya | FK→patient_order | — | |
| data_key_id | uuid | tidak | FK→data_key, UQ | — | Wadah `complaint_ticket`; FK komposit `(data_key_id, id)` → `data_key (id, container_id)` |

Index: `ix (patient_id, created_at DESC)`.

#### A16.2 `complaint_ticket_status_event` (R)

*STD-R* dengan `complaint_ticket_id`. `to_status` CHECK (`new`, `in_progress`, `awaiting_patient`, `closed`). Pembukaan kembali dalam 7 hari (US-1104 AC3) = `closed → in_progress` `[KEPUTUSAN-54]`.

#### A16.3 `complaint_ticket_state` (P)

*STD-P* dengan `complaint_ticket_id`. Index `ix (status, updated_at)` (antrean urut terlama, US-1104 AC1).

#### A16.4 `complaint_ticket_message` (T)

Pesan tiket dari pasien, balasan Layanan Pelanggan, dan tanggapan rekanan yang dicatat Layanan Pelanggan (US-1104 AC2). Rekanan hanya melihat pesan yang ditandai untuk rekanan, supaya data klinis tidak terbuka `[KEPUTUSAN-53]`. Sumber: US-1103 AC2, US-1104 AC2, US-1203 AC1.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-T* | | | | | |
| complaint_ticket_id | uuid | tidak | FK→complaint_ticket | — | |
| author_type | text | tidak | | — | CHECK (`patient`, `rs_admin`, `partner_response_record`) |
| author_account_id | uuid | tidak | FK→account | — | |
| body_enc | bytea | tidak | | PII, Enc | Dapat memuat data kesehatan yang ditulis pasien |
| visible_to_partner | boolean | tidak | | — | Ditetapkan Layanan Pelanggan saat meneruskan |
| data_key_id | uuid | tidak | FK→data_key | — | FK komposit `(data_key_id, complaint_ticket_id)` → `data_key (id, container_id)` |

Index: `ix (complaint_ticket_id, created_at)`.

#### A16.5 `complaint_ticket_attachment` (T)

Lampiran foto tiket (US-1103 AC1).

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-T* | | | | | |
| complaint_ticket_message_id | uuid | tidak | FK→complaint_ticket_message | — | |
| file_id | uuid | tidak | FK→stored_file, UQ | PII | Bucket `claim_evidence` (sensitivitas setara) |

Index: `ix (complaint_ticket_message_id)`.

#### A16.6 `complaint_ticket_forward` (T)

Penerusan tiket ke rekanan (US-1104 AC2). Rekanan melihat tiket di portal hanya bila ada baris ini (US-1203 AC2).

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-T* | | | | | `created_by` = Layanan Pelanggan |
| complaint_ticket_id | uuid | tidak | FK→complaint_ticket | — | |
| partner_id | uuid | tidak | FK→partner | — | |

Index: `UQ (complaint_ticket_id, partner_id)`; `ix (partner_id, created_at)`.

### A17. Hak subjek data dan retensi

#### A17.1 `data_subject_request` (T)

Permintaan hak subjek data. Sumber: BR-106, BR-111, US-1602–US-1605, E-13, ADR-011 butir 5.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-T* | | | | | |
| request_number | text | tidak | UQ | — | |
| patient_id | uuid | tidak | FK→patient | — | Pemohon |
| care_recipient_id | uuid | tidak | FK→care_recipient | — | Subjek: diri sendiri atau anggota keluarga (US-1602 AC1). Jalur anggota keluarga dewasa sendiri `[TIDAK DIKETAHUI]` (L-08, US-1602 AC3) |
| request_type | text | tidak | | — | CHECK (`information`, `access_copy`, `rectification`, `erasure`, `restriction`, `objection`, `portability`) |
| detail_enc | bytea | ya | | PII, Enc | `[KEPUTUSAN-43]` |
| data_key_id | uuid | tidak | FK→data_key, UQ | — | Wadah `data_subject_request`; FK komposit `(data_key_id, id)` → `data_key (id, container_id)` |
| reauthenticated_at | timestamptz | tidak | | — | Waktu autentikasi ulang di Keycloak (US-1602 AC1) |
| due_at | timestamptz | tidak | | — | 3 x 24 jam (US-1603 AC1, AC4) |

Index: `ix (due_at)` untuk tanda mendesak 12 jam (US-1603 AC2). Penghapusan (`erasure`) terblokir sampai L-06 dan PT-24 terjawab (US-1605 AC3).

#### A17.2 `data_subject_request_status_event` (R)

*STD-R* dengan `data_subject_request_id`. `to_status` CHECK (`received`, `in_progress`, `completed`, `rejected`). CHECK `to_status <> 'rejected' OR reason_code IS NOT NULL`.

#### A17.3 `data_subject_request_state` (P)

*STD-P* dengan `data_subject_request_id`.

#### A17.4 `data_subject_request_response` (T)

Tanggapan tertulis, termasuk untuk pengerjaan manual di luar sistem (US-1603 AC7).

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-T* | | | | | `created_by` = Petugas PDP |
| data_subject_request_id | uuid | tidak | FK→data_subject_request | — | |
| response_enc | bytea | tidak | | PII, Enc | |
| evidence_file_id | uuid | ya | FK→stored_file | PII | Bucket `export` |
| data_key_id | uuid | tidak | FK→data_key | — | FK komposit `(data_key_id, data_subject_request_id)` → `data_key (id, container_id)` |

#### A17.5 `data_export_package` (T)

Berkas salinan data pribadi, dibuat sistem tanpa dibuka Petugas PDP (§3.1). Isinya mencakup rekam jejak pemrosesan dari audit (subjek data dan E-17) (US-1604 AC1). Sumber: US-1604, UP-47.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-T* | | | | | |
| data_subject_request_id | uuid | tidak | FK→data_subject_request | — | |
| format | text | tidak | | — | CHECK (`machine_readable`, `human_readable`) |
| file_id | uuid | tidak | FK→stored_file | PII, Spesifik | Bucket `export`; berkas dihapus setelah tautan berakhir |
| download_expires_at | timestamptz | tidak | | — | 7 hari (US-1604 AC2) |

Index: `UQ (data_subject_request_id, format)`. Rekam medis RME termasuk atau tidak `[TIDAK DIKETAHUI]` (L-05, US-1604 AC3).

#### A17.6 `retention_run` (L)

Laporan hasil job retensi harian (US-1606 AC1, ADR-013 §6). Story terblokir sampai PT-24 terjawab (US-1606 AC2).

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-L* | | | | | |
| retention_policy_id | uuid | tidak | FK→retention_policy | — | |
| data_category | text | tidak | | — | |
| end_action | text | tidak | | — | |
| started_at | timestamptz | tidak | | — | |
| finished_at | timestamptz | tidak | | — | |
| items_evaluated | integer | tidak | | — | |
| items_actioned | integer | tidak | | — | |
| outcome | text | tidak | | — | CHECK (`completed`, `partial`, `failed`) |

### A18. Integrasi rekanan

Token akses client credentials diterbitkan Keycloak realm `integrasi` `[USER] T4-Q4`; tidak ada tabel token akses atau hash rahasia klien di database aplikasi.

#### A18.1 `integration_client` (M)

Pemetaan `client_id` Keycloak ke rekanan, beserta rahasia penandatangan webhook HMAC (ADR-014 §3). Satu klien per rekanan, termasuk setiap fasyankes RME. ID rekanan selalu diambil dari klien, bukan dari isi pesan (ADR-012). Sumber: ADR-014, US-1402 AC3, `[USER] T4-Q4`.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-M* | | | | | |
| partner_id | uuid | tidak | FK→partner, UQp | — | |
| idp_client_id | text | tidak | UQp | — | `client_id` di realm `integrasi` |
| webhook_signing_secret_enc | bytea | tidak | | Rahasia, Enc | Juga untuk menandatangani panggilan sinkron keluar |
| data_key_id | uuid | tidak | FK→data_key | — | Wadah `integration_client` |
| secret_rotated_at | timestamptz | tidak | | — | |
| status | text | tidak | | — | CHECK (`active`, `revoked`) |

Index: `UQp (partner_id) WHERE deleted_at IS NULL`; `UQp (idp_client_id) WHERE deleted_at IS NULL`.

#### A18.2 `partner_endpoint` (M)

URL webhook dan URL pemeriksaan ketersediaan sinkron milik rekanan (ADR-014 §2).

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-M* | | | | | |
| partner_id | uuid | tidak | FK→partner | — | |
| endpoint_kind | text | tidak | | — | CHECK (`webhook`, `availability_check`) |
| url | text | tidak | | — | HTTPS (ADR-013 §1) |
| is_active | boolean | tidak | | — | |

Index: `UQp (partner_id, endpoint_kind) WHERE deleted_at IS NULL`.

#### A18.3 `inbound_idempotency_key` (O)

Kunci idempotensi pesan masuk, disimpan 7 hari; pesan ulang mendapat respons yang sama (ADR-014 §4, NFR-INT-01).

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-O* | | | | | `expires_at` = 7 hari |
| integration_client_id | uuid | tidak | FK→integration_client | — | |
| idempotency_key | text | tidak | | — | |
| request_hash | bytea | tidak | | — | Kunci sama dengan isi berbeda ditolak |
| response_status | smallint | tidak | | — | |
| response_body | jsonb | ya | | — | Tanpa data pribadi |

Index: `UQ (integration_client_id, idempotency_key)`; `ix (expires_at)`.

#### A18.4 `integration_error_log` (L)

Galat integrasi masuk dan keluar, tanpa data pribadi. Dipakai untuk jumlah galat 24 jam (US-1405 AC1) dan bukti penetapan penyebab RF-18 (US-808 AC6). Kredensial salah dan tanda tangan tidak valid juga masuk security log (ADR-010). Sumber: US-901 AC2, US-1402 AC2, US-1404 AC2.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-L* | | | | | |
| partner_id | uuid | ya | FK→partner | — | `NULL` bila klien tidak dikenal |
| direction | text | tidak | | — | CHECK (`inbound`, `outbound_webhook`, `outbound_sync_check`) |
| message_type | text | tidak | | — | |
| error_code | text | tidak | | — | Kode galat kontrak (RFC 9457 via ADR-014) |
| http_status | smallint | ya | | — | |
| patient_order_id | uuid | ya | FK→patient_order | — | |
| detail | jsonb | ya | | — | Mengikuti skema daftar izin per `error_code` (field yang boleh disimpan, misalnya nama field yang gagal validasi dan kode aturan); nilai field dari pesan rekanan tidak disalin, sehingga tanpa data pribadi (review DBA S09) |
| correlation_id | uuid | tidak | | — | |

Index: `ix (partner_id, created_at)`; `ix (patient_order_id)`.

#### A18.5 `outbound_delivery_attempt` (L)

Setiap percobaan mengirim webhook ke rekanan (ADR-014 §4, US-1403).

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-L* | | | | | |
| event_id | uuid | tidak | | — | Sama dengan `integration_outbox.event_id`; tanpa FK karena outbox dihapus |
| partner_id | uuid | tidak | FK→partner | — | |
| message_type | text | tidak | | — | |
| patient_order_id | uuid | ya | FK→patient_order | — | |
| attempt_no | smallint | tidak | | — | |
| outcome | text | tidak | | — | CHECK (`delivered`, `rejected`, `timeout`, `error`) |
| http_status | smallint | ya | | — | |

Index: `ix (event_id)`; `ix (partner_id, created_at)`; `ix (patient_order_id)`.

#### A18.6 `partner_integration_health` (O)

Ringkasan kesehatan integrasi yang sering berubah, dipisah dari `partner` supaya pembaruan metrik tidak menjadi perubahan data master. Sumber: US-1405 AC1–AC2, NFR-INT-03, ADR-014 §7.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-O* | | | | | |
| partner_id | uuid | tidak | FK→partner, UQ | — | |
| last_success_at | timestamptz | ya | | — | Waktu sinkron terakhir yang berhasil |
| last_failure_at | timestamptz | ya | | — | |
| failing_since | timestamptz | ya | | — | Dasar status Terganggu setelah 30 menit |
| updated_at | timestamptz | tidak | | — | |

### A19. Platform: outbox, job, berkas, kunci, dan KPI

#### A19.1 `audit_outbox` (O, hak khusus)

Event audit yang menunggu dikirim relay ke database audit, ditulis dalam transaksi yang sama dengan aksi bisnisnya. Role aplikasi hanya `INSERT`; relay `SELECT` dan `DELETE` (ADR-009 butir 2–3). Sumber: ADR-009, `[KEPUTUSAN-22]`, KEPUTUSAN-15.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| id | uuid | tidak | PK | — | = `event_id` |
| created_at | timestamptz | tidak | | — | |
| occurred_at | timestamptz | tidak | | — | |
| payload | jsonb | tidak | | PII | Field PRD §7.2 yang sudah dimasking pustaka (alamat IP, ID akun) |

Index: `ix (created_at)`. Umur baris tertua dipantau (ADR-009 butir 3).

#### A19.2 `integration_outbox` (O)

Pesan keluar ke rekanan, dikirim minimal sekali dengan `event_id` unik (ADR-014 §4). Sumber: US-402 AC3, US-1403.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-O* | | | | | Dihapus setelah `delivered` atau `abandoned` |
| event_id | uuid | tidak | UQ | — | |
| partner_id | uuid | tidak | FK→partner | — | |
| message_type | text | tidak | | — | Misalnya pesanan terbayar, pembatalan, daftar tenaga kesehatan aktif, status home visit |
| patient_order_id | uuid | ya | FK→patient_order | — | |
| entity_type | text | tidak | | — | Jenis entitas yang dirujuk (misalnya `patient_order`, `health_worker`) |
| entity_id | uuid | tidak | | — | Isi pesan, termasuk data penerima minimal kontrak `[TIDAK DIKETAHUI]` (L-08), disusun dari entitas ini saat pengiriman (review DBA B04); pengirimannya dicatat E-17 |
| payload_enc | bytea | ya | | PII, Enc | Hanya bila isi pesan harus dibekukan saat kejadian (misalnya pembatalan). Wadah entitas terkait atau `status_note` pesanan |
| data_key_id | uuid | ya | FK→data_key | — | CHECK `(payload_enc IS NULL) = (data_key_id IS NULL)` |
| available_at | timestamptz | tidak | | — | Jeda membesar 1–30 menit |
| attempt_count | smallint | tidak | | — | |
| deadline_at | timestamptz | ya | | — | Jadwal mulai (RF-18) atau batas PB-06 (RF-08) |
| first_failed_at | timestamptz | ya | | — | Dasar penanda 30 menit (US-402 AC3) |
| status | text | tidak | | — | CHECK (`pending`, `delivered`, `abandoned`) |
| updated_at | timestamptz | tidak | | — | |

Index: `ix (status, available_at)`.

#### A19.3 `notification_outbox` (O)

Kejadian yang menunggu dikirim ke kanal email atau di dalam aplikasi (ADR-016 butir 5).

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-O* | | | | | Dihapus setelah selesai |
| event_type | text | tidak | | — | |
| recipient_account_id | uuid | ya | FK→account | — | |
| recipient_partner_id | uuid | ya | FK→partner | — | |
| channel | text | tidak | | — | CHECK (`email`, `in_app`); satu baris per kanal sehingga kegagalan satu kanal tidak menghentikan yang lain (US-1001 AC3) |
| template_code | text | tidak | | — | |
| params | jsonb | tidak | | PII | Boleh memuat alamat email tujuan (misalnya email lama, US-105 AC4) |
| available_at | timestamptz | tidak | | — | |
| attempt_count | smallint | tidak | | — | |
| status | text | tidak | | — | CHECK (`pending`, `done`, `failed`) |

Index: `ix (status, available_at)`.

#### A19.4 `job` (O)

Antrean job terjadwal dan latar belakang, diambil dengan `SELECT ... FOR UPDATE SKIP LOCKED` `[SRC-36]` (ADR-007 butir 3). Contoh: batas waktu PB, pengingat PB-08, pemeriksaan SIP harian, rekonsiliasi, retensi, pembandingan proyeksi.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-O* | | | | | |
| job_type | text | tidak | | — | |
| payload | jsonb | tidak | | — | Hanya ID |
| run_at | timestamptz | tidak | | — | Instan absolut (ADR-017 butir 4) |
| status | text | tidak | | — | CHECK (`queued`, `running`, `succeeded`, `failed`, `cancelled`) |
| attempt_count | smallint | tidak | | — | |
| max_attempts | smallint | tidak | | — | |
| locked_by | text | ya | | — | |
| locked_until | timestamptz | ya | | — | |
| last_error | text | ya | | — | Tanpa data pribadi |
| dedupe_key | text | ya | UQp | — | Mencegah job ganda untuk entitas dan tujuan yang sama |
| finished_at | timestamptz | ya | | — | |

Index: `ix (run_at) WHERE status = 'queued'` (pengambilan job, review DBA S08); `ix (status, finished_at) WHERE status IN ('succeeded','failed','cancelled')` (pembersihan); `UQp (dedupe_key) WHERE status IN ('queued','running')`. Job idempoten (ADR-007). Baris selesai dihapus bertahap oleh job pembersihan (§D.2).

#### A19.5 `upload_slot` (O)

Slot unggah yang belum dikonfirmasi. Berkas yang gagal validasi atau tidak pernah dikonfirmasi dihapus otomatis bersama baris ini (ADR-018 butir 2); baris `stored_file` baru dibuat setelah lolos validasi.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-O* | | | | | |
| owner_account_id | uuid | ya | FK→account | — | `NULL` untuk unggahan rekanan (PDF hasil lab) |
| owner_partner_id | uuid | ya | FK→partner | — | Belum dibuat di migrasi F1-T17 karena tabel `partner` (A5) belum ada di Fase 1. Kolom dibuat bersama migrasi A5 (expand §11.3 skema fisik: kolom `NULL`, FK ditambah `NOT VALID` lalu `VALIDATE`) (review DBA F1-T17 butir 3) |
| bucket | text | tidak | | — | Sama dengan `stored_file.bucket` |
| object_key | text | tidak | UQ | — | UUID buatan aplikasi; berprefiks `incoming/` sejak TD-17 |
| allowed_mime_type | text | tidak | | — | |
| max_size_bytes | integer | tidak | | — | 5 MB foto, 10 MB PDF (KEPUTUSAN-08) |
| consumed_at | timestamptz | ya | | — | TD-17: slot sudah dipakai (`stored_file` dicatat) atau berkasnya ditolak. Baris tetap ada sebagai tombstone sampai pembersih menghapusnya setelah `expires_at` + jeda, karena URL unggah masih dapat dipakai sampai kedaluwarsa |
| sanitized_object_key | text | ya | UQ | — | TD-17: kunci objek hasil kode ulang (prefiks `files/`) yang diklaim slot sebelum objeknya ditulis; CHECK `<> object_key`. F1-T102: untuk PDF klaim ini dibuat **sebelum** unduh dan pemeriksaan (klaim atomik "sedang divalidasi"), dan dilepas bila kegagalan bukan salah berkas |

Index tambahan F1-T102 (migrasi `20261126350000_platform_upload_slot_owner_index.sql`): `ix_upload_slot_owner_account_id_bucket_created_at (owner_account_id, bucket, created_at)` untuk kuota slot per akun per jam. Pembersih tidak menghapus baris (dan objek) sebelum `created_at` + 1 jam (`UPLOAD_SLOT_QUOTA_RETENTION`), supaya hitungan kuota benar; `created_at` diisi jam aplikasi.


Pembersihan (TD-17, migrasi `20261120131700_platform_upload_slot_tombstone.sql`): job pembersih menghapus semua versi objek `object_key` dan objek `sanitized_object_key` yang tidak tercatat di `stored_file`, lalu barisnya, setelah `expires_at` + jeda 15 menit (usulan teknis); rekonsiliasi berkala menghapus objek di prefiks `incoming/` dan `files/` yang tidak tercatat.

#### A19.6 `stored_file` (M)

Metadata berkas yang sudah lolos validasi. URL objek tidak disimpan (ADR-018 butir 3). Sumber: ADR-018, NFR-SEC-08.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-M* | | | | | |
| bucket | text | tidak | | — | CHECK (`identity_document`, `clinical`, `claim_evidence`, `license_document`, `export`, `public_content`, `public_catalog`). `public_catalog` belum tercantum di ADR-018 (temuan F-07) |
| object_key | text | tidak | UQ | — | |
| category | text | tidak | | — | Misalnya `ktp_photo`, `chat_photo`, `lab_pdf` |
| owner_account_id | uuid | ya | FK→account | — | |
| container_type | text | ya | | — | Wadah retensi (ADR-018 butir 4) |
| container_id | uuid | ya | | — | |
| mime_type | text | tidak | | — | CHECK (`image/jpeg`, `image/png`, `application/pdf`, `text/csv`, `application/json`) |
| size_bytes | integer | tidak | | — | CHECK `> 0` |
| sha256 | bytea | tidak | | — | |
| validated_at | timestamptz | tidak | | — | |
| purged_at | timestamptz | ya | | — | Objek dan semua versinya dihapus (ADR-018 butir 5, E-16) |

Index: `ix (container_type, container_id)`. Sensitivitas berkas mengikuti tabel yang merujuknya.

#### A19.7 `data_key` (M khusus)

Kunci data (DEK) terbungkus KEK di KMS, satu per wadah (ADR-013 §3). Aturan khusus `[KEPUTUSAN-44]`: tanpa soft delete; crypto-shredding dilakukan role `privacy_worker` dengan mengosongkan `wrapped_dek` dan mengisi `shredded_at`, sehingga baris tetap ada untuk FK dari tabel transaksi (ADR-004). Selama backup lama masih tersimpan (masa simpan backup 35 hari sebagai nilai sementara sampai PT-24 `[USER] T4-G2`, ADR-019), DEK terbungkus masih ada di backup itu (temuan F-08).

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| id | uuid | tidak | PK | — | |
| created_at | timestamptz | tidak | | — | |
| container_type | text | tidak | | — | CHECK nilai §0.7 |
| container_id | uuid | tidak | | — | |
| wrapped_dek | bytea | ya | | Rahasia | CHECK `(wrapped_dek IS NULL) = (shredded_at IS NOT NULL)` |
| kek_ref | text | tidak | | — | ID dan versi KEK di KMS |
| retention_category | text | ya | | — | Kategori di `retention_policy`. Bila belum diketahui saat `INSERT`, diisi sekali lewat `data_key_retention_anchor` (A19.10, CR-014) |
| retention_anchor_at | timestamptz | ya | | — | Tanggal acuan retensi (ADR-013 §6). Bila baru diketahui kemudian, dicatat sekali di `data_key_retention_anchor` (A19.10, CR-014); kolom ini tidak dapat diubah setelah `INSERT` (trigger satu arah) |
| shredded_at | timestamptz | ya | | — | |
| shred_reason | text | ya | | — | CHECK (`retention_expired`, `erasure_request`) |

Index dan constraint: `UQ (container_type, container_id)`; `UQ (id, container_id)` (sasaran FK komposit kesesuaian wadah, §0.2); `ix (retention_category, retention_anchor_at) WHERE shredded_at IS NULL`.

Perlindungan dan rotasi `[KEPUTUSAN-66]` (review DBA B01):
- **Trigger satu arah**: `privacy_worker` hanya boleh mengisi `wrapped_dek = NULL` bersamaan dengan pengisian `shredded_at` dan `shred_reason` (review DBA S-N3); setelah `shredded_at` terisi, `UPDATE` atas baris itu ditolak untuk semua role, sehingga DEK yang sudah dimusnahkan tidak dapat dihidupkan kembali. Role aplikasi hanya S dan I.
- **Rotasi KEK, cara utama**: memakai versi kunci di KMS yang tetap dapat membuka DEK yang dibungkus versi lama, sehingga baris tidak perlu dibungkus ulang dan `kek_ref` cukup mencatat versi saat dibungkus. Kemampuan ini diverifikasi ke dokumentasi KMS penyedia terpilih di Tahap 5.
- **Rotasi KEK, cadangan** (bila KMS tidak menyediakan cara utama): role terpisah `key_rotation` dengan U hanya pada `wrapped_dek` dan `kek_ref`, dan hanya untuk baris `shredded_at IS NULL` (ditegakkan trigger yang sama). Setiap pembungkusan ulang dicatat di security log.

#### A19.8 `kpi_snapshot` (L)

Angka KPI akhir bulan yang tidak dapat direkonstruksi kemudian (misalnya jumlah tenaga kesehatan siap melayani pada akhir bulan, US-1701 AC3). Sumber: BR-090, US-1701.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-L* | | | | | |
| metric_code | text | tidak | | — | CHECK (`kpi_01`, `kpi_01_active_owners`, `kpi_02_doctor`, `kpi_02_psychologist`, `kpi_03`, `kpi_04`) |
| period_month | date | tidak | | — | Bulan kalender WIB |
| value | bigint | tidak | | — | Agregat, tanpa data pribadi (US-1701 AC5) |

Index: `UQ (metric_code, period_month)`.

#### A19.9 `patient_app_activity_month` (L)

Penanda pemilik akun membuka aplikasi dalam keadaan masuk minimal sekali di suatu bulan, untuk angka pendukung KPI-01 (US-1701 AC2).

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-L* | | | | | |
| patient_id | uuid | tidak | FK→patient | — | |
| activity_month | date | tidak | | — | |

Index: `UQ (patient_id, activity_month)`.

#### A19.10 `data_key_retention_anchor` (T)

Ditambahkan oleh **CR-014** (disetujui di gate Tahap 5, T5-G2 e; dijadwalkan F1-T17 `[KEPUTUSAN-185]`), desain fisik `docs/05-physical-schema.md` §4.12 `[KEPUTUSAN-155]`. Acuan retensi DEK yang baru diketahui setelah wadah dibuat (misalnya akhir sesi konsultasi), dicatat **sekali** per DEK, karena `data_key.retention_category`/`retention_anchor_at` tidak dapat diubah setelah `INSERT` dan role aplikasi tidak punya `UPDATE` atas `data_key` (A19.7, KEPUTUSAN-66). Sumber: CR-014, ADR-013 §5–§6, review DBA Tahap 5 F-P04.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| data_key_id | uuid | tidak | PK, FK→data_key | — | Satu baris per DEK (PK menolak pengisian kedua) |
| retention_category | text | tidak | | — | Kategori di `retention_policy`; wajib sama dengan `data_key.retention_category` bila kolom itu sudah terisi (trigger) |
| retention_anchor_at | timestamptz | tidak | | — | Tanggal acuan retensi. Ditolak bila `data_key.retention_anchor_at` sudah terisi (trigger) |
| created_at | timestamptz | tidak | | — | |
| created_by | uuid | ya | FK→account | — | |

Index: `ix (retention_category, retention_anchor_at)`.

Aturan:
- Acuan yang berlaku = `coalesce(data_key.retention_anchor_at, data_key_retention_anchor.retention_anchor_at)` (begitu pula kategori). Acuan `NULL` berarti DEK **tidak memenuhi syarat retensi** (gagal-tertutup): job retensi tidak pernah memusnahkan DEK tanpa acuan.
- Kelas T: tanpa `UPDATE`/`DELETE`/`TRUNCATE` (trigger penolak). Koreksi acuan yang salah tidak dapat dilakukan dengan `UPDATE`; bila kelak diperlukan, polanya diusulkan lewat CR.
- Hak (melengkapi §0.6 A): `rs_app` S, I; `privacy_worker` S (seluruh kolom, untuk menghitung acuan berlaku); role lain tanpa hak.
- Padanan RME `rme_data_key_retention_anchor` (C, dengan `fasyankes_id` dan RLS) dijadwalkan di EN-22 Fase 3.

---

### A20. Layanan korporat (CR-021, rancangan)

Status: **rancangan** (data dictionary v1.1); tabel K1 (`corporate_client`, `corporate_client_document`, `corporate_verification_decision`, `corporate_member`) **final** sejak v1.3 (F1-T98/F1-T99), rinciannya di "Rincian K1" di bawah tabel. Sumber: PRD v1.3 EP-19, AB-80–AB-87; ADR-028 (Diterima); ADR-029 (Diusulkan, arah dana menunggu PT-59/L-21); `docs/cr/CR-021-analisis.md` §5 `[KEPUTUSAN-262]`. Konvensi §0.2–§0.7 berlaku: PK UUIDv7, kolom standar STD-x, uang `bigint _idr`, PII di Master yang dapat dianonimkan ("Anonimkan"), isi spesifik terenkripsi per wadah. Rincian kolom, constraint, index, hak role (§0.6), dan kebijakan retensi (PT-24) ditetapkan di task migrasi: F1-T98/F1-T99 untuk tabel K1 (Fase 1), task K2 untuk sisanya (Fase 3), keduanya dengan review DBA dan keamanan. Tabel K1 dibangun di Fase 1; tabel K2/K3 di Fase 3 (phase plan v0.3, skenario S3 `[USER] C21-P4`).

| Tabel | Kelas | Isi utama (rancangan) | Sensitivitas | Fase |
| --- | --- | --- | --- | --- |
| `corporate_client` | M | `legal_name`, `nib`, `npwp`, `office_address`, `office_phone`, `status` CHECK (`pending_verification`, `active`, `rejected`, `inactive`), `origin` CHECK (`self_registered`, `created_by_admin`), `submission_round`, `submitted_at`. UQp `nib WHERE deleted_at IS NULL AND status = 'active'` (NIB unik hanya di antara perusahaan Aktif; **diubah CR-035** dari "NIB ganda ditolak" `[USER] C21-P4` 4c oleh keputusan user Q10 = B2, KEPUTUSAN-362, migrasi `20261126400000`/`20261126410000`); ix `nib WHERE deleted_at IS NULL` (penanda antrean, KEPUTUSAN-363) | — (badan usaha); `npwp` Terbatas (lihat Rincian K1) | 1 |
| `corporate_client_document` | M | `corporate_client_id`, `document_kind` CHECK (`nib`, `npwp`, `authority_proof`), `stored_file_id` (bukti kewenangan pendaftar wajib, `[USER] C21-P4` 4b); satu dokumen aktif per jenis | Dokumen dapat memuat PII pengurus | 1 |
| `corporate_verification_decision` | T | `corporate_client_id`, `submission_round`, `decision`, `reason_code`, `decided_by` (Verifikator); UQ `(corporate_client_id, submission_round)` (keputusan pertama menang) | — | 1 |
| `corporate_member` | M | `account_id` (jenis `corporate_admin`, diperiksa aplikasi), `corporate_client_id`, `job_title` (Anonimkan), `anonymized_at`; UQp satu keanggotaan aktif per akun (M-51) | PII lewat akun; `job_title` PII | 1 |
| `corporate_package` | M (salinan rekanan) | `partner_id`, `external_id`, `package_kind` CHECK (`mass_vaccination`, `mass_mcu`, `mass_lab`), nama, isi, `location_options`, `partner_price_idr`, `requires_nik`, `synced_at`. Masuk lewat Partner Integration API (K-04 opsi a) | — | 3 |
| `corporate_order` | T | `corporate_client_id`, `corporate_package_id`, `partner_id`, `location_kind` CHECK (`onsite`, `partner_site`), alamat on-site, jumlah peserta | — | 3 |
| `corporate_order_status_event` / `corporate_order_state` | R / P | Status UP-80 | — | 3 |
| `corporate_order_price_lock` | T | Harga rekanan per satuan, `margin_rule_id` dan nilainya, margin per satuan, jumlah, tarif dan nominal pajak, total (AB-81) | — | 3 |
| `corporate_participant` | M | `corporate_client_id`, `full_name`, `email`, `phone`, `birth_date` (Anonimkan); `nik_enc`, `nik_bidx`, `nik_bidx_key_version`, `nik_last4` opsional | PII; NIK Spesifik, Enc (wadah `corporate_participant_identity`) | 3 |
| `corporate_order_participant` | T | Keikutsertaan peserta dalam pesanan; pencabutan lewat baris pembalik | Keikutsertaan diperlakukan sensitif sampai L-08/L-18 | 3 |
| `participant_import_batch` | L | Pesanan, pengunggah, jumlah valid/galat/ganda, status; tanpa isi baris | — | 3 |
| `participant_import_row` | O (berumur) | Baris pratinjau terenkripsi, hasil validasi; dihapus saat konfirmasi, pembatalan, atau kedaluwarsa | PII, Enc (wadah `corporate_import_preview`) | 3 |
| `corporate_invitation` / `corporate_invitation_response` | T / T | Hash token (bukan token), `expires_at` (PB-24); respons diterima/ditolak dan `care_recipient_id` yang ditautkan | — | 3 |
| `corporate_attendance_record` | T | Status hadir, sumber (PT-64), pencatat, konfirmasi identitas di lokasi (UP-87) | Diperlakukan sensitif sampai L-08/L-18 | 3 |
| `b2b_invoice`, `b2b_invoice_line`, `b2b_invoice_status_event`, `b2b_invoice_state` | T / T / R / P | Nomor invoice (format Tahap 8, KEPUTUSAN-57), jatuh tempo (PB-25), total, pembatalan dan invoice pengganti; baris tidak menyebut peserta (AB-87) | — | 3 |
| `bank_statement_entry` | T | Satu baris per transfer masuk: rekening tujuan, tanggal valuta, referensi bank, nominal, `stored_file_id` bukti, pencatat, penyetuju (UP-84); kunci unik global `[KEPUTUSAN-268]` | Nomor rekening pembayar hanya 4 digit terakhir di kolom; berkas bukti hanya Keuangan, tidak pernah masuk log/audit | 3 |
| `b2b_payment_allocation` | T | `bank_statement_entry_id`, `b2b_invoice_id`, nominal alokasi, penyetuju, pembalik; jumlah alokasi per mutasi ≤ nominal mutasi (aplikasi dan database) | — | 3 |
| `corporate_share_consent_record` | L | Peserta tertaut, perusahaan, pesanan, `consent_text_version_id`, aksi (beri/tarik) | Merujuk orang lewat ID (KEPUTUSAN-42) | 3 |
| `corporate_result_summary` | T | Peserta, pesanan, `summary_enc`, `data_key_id`, versi | Spesifik, Enc (wadah `corporate_result_summary`) | 3 |

**Rincian K1 (final, F1-T98/F1-T99, review DBA LULUS).** Migrasi: `db/migrations/app/20261123100000_corporate_client_verification.sql` dan `20261123110000_corporate_member_account_korporat.sql`; test `db/test/tabel-app-a20.sql`. Semua tabel memakai STD-M/STD-T, FK tanpa `CASCADE`, komentar `class:<kelas> module:korporat`.

| Tabel | Kolom tambahan dan batas | Constraint dan index | Hak role (§0.6 A) |
| --- | --- | --- | --- |
| `corporate_client` (M) | `legal_name` 1–200, `nib` 1–64 tanpa spasi tepi (`CHECK nib = btrim(nib)`), `npwp` 1–64, `office_address` 1–500, `office_phone` 1–32 (panjang dihitung setelah `btrim`); `submission_round integer NOT NULL DEFAULT 1` (≥ 1, naik satu saat pengajuan ulang setelah ditolak, US-1902 AC3); `submitted_at timestamptz NOT NULL DEFAULT now()` (waktu pengajuan terakhir); `delete_reason` ≤ 500. **Koreksi data oleh HR** (CR-036, `[USER]` Q20 = A, KEPUTUSAN-410): `legal_name`, `nib`, `npwp`, `office_address`, `office_phone` hanya diubah aplikasi bersama pengajuan ulang, dalam satu `UPDATE` bersyarat `status = 'rejected' AND submission_round = $n` setelah kunci baris (status hasil `pending_verification`, sehingga `uqp_corporate_client_nib_active` tidak terlibat); validasi sama dengan pendaftaran (KEPUTUSAN-325); audit E-10 dengan before/after field yang berubah, NIB/NPWP hanya 4 digit terakhir (KEPUTUSAN-411 diubah KEPUTUSAN-416); `submitted_at`/`updated_at` pengajuan ulang = `statement_timestamp()` setelah kunci baris (KEPUTUSAN-417); token keputusan Verifikator `documents_version` = hash putaran + ID dokumen aktif, sehingga koreksi (putaran naik) membatalkan token detail yang dibuka sebelumnya (`4090114`, KEPUTUSAN-415); tanpa perubahan skema | `UQp (nib) WHERE deleted_at IS NULL AND status = 'active'` (`uqp_corporate_client_nib_active`; **diubah CR-035**, keputusan user Q10 = B2, KEPUTUSAN-362: migrasi `20261126400000` expand menambah index ini, `20261126410000` contract menghapus `uqp_corporate_client_nib` lama yang berlaku untuk status apa pun; pengajuan `pending_verification`/`rejected` dan perusahaan `inactive` boleh ber-NIB sama, perusahaan Aktif kedua ditolak 23505); `ix (nib) WHERE deleted_at IS NULL` (`ix_corporate_client_nib`: penanda `nib_used_by_other_company` KEPUTUSAN-363 dan cek persetujuan `4090116` KEPUTUSAN-364); **jalur masa depan yang menulis `status = 'active'` atau mengosongkan `deleted_at`** (reaktivasi US-1916, pemulihan hapus lunak) wajib mengunci baris (`FOR UPDATE`), memeriksa perusahaan Aktif lain ber-NIB sama (`repo::other_active_client_with_nib`), dan memetakan 23505 `uqp_corporate_client_nib_active` ke 409, bukan 500 (TD-51; aturan bisnisnya menunggu pertanyaan user Q22); `ix (submitted_at) WHERE deleted_at IS NULL AND status = 'pending_verification'` (antrean terlama, US-1902 AC1) | `rs_app` S, I, U; `rs_privacy_worker` S (tanpa kolom Anonimkan) |
| `corporate_client_document` (M) | `delete_reason` ≤ 500. Penggantian dokumen = hapus lunak baris lama lalu sisip baris baru | `UQp (corporate_client_id, document_kind) WHERE deleted_at IS NULL` (alias `corp_client_doc`); `UQ (stored_file_id)` | `rs_app` S, I, U; `rs_privacy_worker` S. PII pengurus hanya di isi berkas (dimusnahkan lewat `stored_file.purged_at`, ADR-018) |
| `corporate_verification_decision` (T) | `submission_round integer NOT NULL` (≥ 1); `decision` CHECK (`approved`, `rejected`); `reason_code` 1–64, wajib bila dan hanya bila `rejected`; `decided_by uuid NOT NULL FK→account` | `UQ (corporate_client_id, submission_round)` (alias `corp_verif_decision`; keputusan pertama menang, US-1902 AC4); trigger penolak UPDATE/DELETE/TRUNCATE tingkat statement | `rs_app` S, I; `rs_privacy_worker` tanpa hak (tidak ada FK subjek data pribadi) |
| `corporate_member` (M) | `job_title` 1–100 atau `NULL` (HR undangan, US-1903 AC1, US-1904 AC2), Anonimkan; `anonymized_at`; `delete_reason` ≤ 500. Pencabutan atau perpindahan = hapus lunak lalu sisip baris baru; aplikasi mencabut sesi (M-51) | `UQp (account_id) WHERE deleted_at IS NULL`; `ix (corporate_client_id) WHERE deleted_at IS NULL` (daftar HR aktif, pemeriksaan HR aktif terakhir); `ix_corporate_member_account_id_deleted_at (account_id, deleted_at DESC NULLS FIRST, id DESC)` (F1-T109, migrasi `20261127100000`, KEPUTUSAN-386: tenant audit akun HR termasuk keanggotaan yang sudah dicabut, dan pelayan FK `account_id` untuk semua baris). Koreksi: komentar "juga melayani FK account_id" di migrasi `20261123110000` tidak akurat untuk baris terhapus (UQp parsial); pelayan FK penuh adalah index F1-T109. Invarian (KEPUTUSAN-387): satu akun HR hanya pernah punya satu baris keanggotaan; perpindahan perusahaan belum ada (TD-52) | `rs_app` S, I, U; `rs_privacy_worker` S + U per kolom `job_title`, `anonymized_at`, `updated_at`, `updated_by` |

Catatan:
- Jenis akun anggota (`account.account_type = corporate_admin`) **tidak ditegakkan database**; diperiksa aplikasi, seperti `rs_admin_role.account_id` (A1.2).
- Format NIB tepat 13 angka dan NPWP badan tepat 16 angka (KEPUTUSAN-325, keputusan user Q6, `[SRC-854]`, `[SRC-855]`; dipakai pendaftaran mandiri dan koreksi HR CR-036). Database masih hanya membatasi panjang dan spasi tepi NIB, dan jalur pembuatan oleh Verifikator (`admin_create`) masih menerima 1–64 karakter tanpa spasi; penyatuan validasi lalu CHECK format lewat migrasi expand dicatat TD-57 butir a.
- `npwp` berklasifikasi **Terbatas** (keputusan review F1-T98/T99): nilai penuh tidak boleh muncul di log aplikasi, payload audit (`before_values`/`after_values`), maupun pesan galat; satu-satunya bentuk di audit adalah 4 digit terakhir (`last4`) pada event koreksi CR-036 (KEPUTUSAN-416). Perlu-tidaknya enkripsi tingkat aplikasi diputuskan di F1-T101. Label "Terbatas" belum ada di legenda §0.5.
- `reason_code` penolakan perusahaan berupa teks kode; konteks A6.7 `corporate_rejection` ditambahkan F1-T104 (migrasi `20261124200000`), dengan daftar awal **draf** (`nib_document_invalid`, `npwp_document_invalid`, `company_data_mismatch`, `authority_proof_insufficient`; hanya yang terakhir disebut PRD US-1902 AC3, sisanya usulan F1-T104 `[TIDAK DIKETAHUI]` persetujuan user). Keanggotaan kode diperiksa aplikasi di transaksi keputusan.
- Status `corporate_client.status` diubah langsung di baris Master oleh keputusan Verifikator (F1-T104), dengan penjaga: kunci baris `FOR UPDATE`, UQ `(corporate_client_id, submission_round)`, dan `UPDATE ... WHERE status = 'pending_verification' AND submission_round = n`; skema K1 tidak memakai pasangan `_status_event`/`_state`.
- Dokumen perusahaan disimpan sebagai `stored_file` bucket `license_document` kategori `corporate_document` (usulan teknis F1-T104, preseden A5.3; jalur unggah F1-T102 memakai bucket dan kategori yang sama; wadah retensi dan pemusnahan dokumen ini belum ditetapkan, lihat TD-49).
- Dengan F1-T99, CHECK realm `idp_event_cursor` dan `idp_event_receipt` (pengambil event Keycloak, F1-T34) juga menerima `korporat`. `actor_type` riwayat status verifikasi identitas tidak diperluas, karena HR tidak menulis tabel itu.

**Catatan untuk DBA (review keamanan CR-021 putaran 2, R2-01)** atas `bank_statement_entry`, diputuskan di task migrasinya:
1. Nominal ikut menjadi bagian kunci unik, sehingga salah ketik nominal untuk referensi yang sama lolos sebagai mutasi kedua. Pilih kunci tanpa nominal, atau peringatan bila referensi sama muncul dengan nominal berbeda.
2. Mutasi yang dibalik lalu dicatat ulang dengan kunci identik akan ditolak UQ. Pilih UQ parsial yang mengecualikan mutasi yang sudah dibalik, atau koreksi lewat alokasi pembalik tanpa mencatat ulang mutasi.
Komposisi kunci juga diverifikasi terhadap format mutasi bank yang dipakai.

**Usulan perubahan tabel lain** (migrasi *expand*, ADR-019 butir 2):
- A1.1 `account`: `account_type` + `corporate_admin`; `idp_realm` + `korporat`; CHECK pasangan `corporate_admin`→`korporat` (F1-T99, Fase 1). **Diterapkan** v1.3 (lihat A1.1).
- A15.4 `cms_content`: kanal (`app`, `site`) dan halaman tujuan situs (F1-T91, Fase 1).
- A19.7 `data_key.container_type`: + `corporate_participant_identity`, `corporate_import_preview`, `corporate_result_summary` `[KEPUTUSAN-258]` (Fase 3).
- A13.1 `journal_account`: piutang korporat per perusahaan, pendapatan margin korporat, dan (bila L-21 memilih penerimaan penuh oleh PT. Arunika) kewajiban kepada rekanan atas pesanan korporat (ADR-029, Fase 3).
- B3 `audit_event_subject`: subjek dapat berupa `corporate_participant` sebelum peserta tertaut ke `care_recipient` (ADR-028 butir 7).

## B. Database audit

Skema yang sama dipakai dua database terpisah: database audit Ruang Sehat dan database audit RME, masing-masing dengan role sendiri (ADR-009 butir 4, ADR-019). Semua tabel kelas Log. Isi klinis, foto dokumen, dan nilai hasil lab tidak pernah disalin ke sini; yang disimpan hanya referensi objek dan versi (PRD §7.3). Password, token, OTP, dan rahasia MFA tidak pernah diterima pustaka audit (ADR-009 butir 7); dengan Keycloak, event autentikasi E-01–E-04 dibentuk aplikasi dari hasil autentikasi tanpa nilai kredensial `[USER] T4-Q4`.

Desain tabel audit `[KEPUTUSAN-56]`: `audit_event` dipartisi per bulan berdasarkan `recorded_at` `[SRC-39]`, supaya rentang `seq` setiap partisi berurutan dan batas arsip sejalan dengan rantai hash. PK berupa komposit `(event_id, recorded_at)` yang memuat kunci partisi; keunikan global `event_id` dan `seq` dijamin tabel tak terpartisi `audit_event_key`. Keharusan memuat kunci partisi pada constraint unik tabel terpartisi diverifikasi ke halaman dokumentasi partisi (SRC-39) di Tahap 5. Nuansa PK komposit terhadap ADR-002 dicatat di temuan F-01.

#### B1. `audit_event` (L, dipartisi bulanan)

Event audit E-01–E-17 dengan field PRD §7.2. Sumber: ADR-005, ADR-009, PRD §7, US-1501–US-1504, `[KEPUTUSAN-22]`.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| event_id | uuid | tidak | PK (bersama `recorded_at`) | — | UUIDv7 dari pustaka audit (ID event, §7.2) |
| recorded_at | timestamptz | tidak | PK, kunci partisi | — | Jam server audit store saat tersimpan (ADR-009 butir 6) |
| seq | bigint | tidak | | — | Nomor urut relay, monoton tanpa celah |
| occurred_at | timestamptz | tidak | | — | Jam server aplikasi saat aksi; ditampilkan WIB (AB-42) |
| event_code | text | tidak | | — | CHECK (`e_01` … `e_17`); sejak PRD v1.3 (CR-021) ditambah `e_18`, lihat catatan di bawah tabel |
| action_name | text | tidak | | — | |
| actor_type | text | tidak | | — | CHECK (`account`, `system`, `partner_system`, `ruang_sehat_system`) |
| actor_account_id | uuid | ya | | PII | ID akun (Ruang Sehat) atau `rme_user` (RME) |
| actor_account_type | text | ya | | — | |
| actor_roles | text[] | ya | | — | Peran dan sub-peran saat aksi |
| actor_partner_id | uuid | ya | | — | Rekanan atau fasyankes pengirim integrasi |
| object_type | text | tidak | | — | |
| object_id | text | ya | | — | Teks, karena objek dapat berupa export atau filter |
| object_version | integer | ya | | — | Versi konten klinis atau dokumen |
| subject_id | uuid | ya | | PII | Subjek data utama: `care_recipient.id` (Ruang Sehat) atau `rme_patient.id` (RME) (§7.2) |
| subject_count | integer | tidak | | — | CHECK `>= 0`; lebih dari 1 → semua subjek di `audit_event_subject`. `integer` karena export laporan dapat memuat lebih dari 32.767 subjek (review DBA S07) |
| tenant_id | uuid | ya | | — | `fasyankes_id` di audit RME (filter log akses Admin Fasyankes, §7.5); `partner_id` bila relevan di audit Ruang Sehat |
| source_component | text | tidak | | — | Aplikasi pasien, aplikasi nakes, backoffice, portal, RME, worker, integrasi |
| source_ip | inet | ya | | PII | Wajib untuk aksi pengguna dan integrasi (§7.2) |
| user_agent | text | ya | | PII | Perangkat atau user agent |
| app_version | text | ya | | — | |
| changed_fields | text[] | ya | | — | Untuk konten klinis dan dokumen hanya ini yang dicatat (§7.2) |
| before_values | jsonb | ya | | PII | Field dari daftar izin per jenis objek, termasking (NIK dan rekening 4 digit terakhir, §7.3) |
| after_values | jsonb | ya | | PII | Sama |
| disclosure_recipient | text | ya | | — | E-17: tujuan pengiriman data pribadi (rekanan, SATUSEHAT, payment gateway) |
| disclosure_data_categories | text[] | ya | | — | E-17: jenis data, tanpa isi. Sumber rekam jejak pemrosesan untuk salinan data (US-1604 AC1) |
| outcome | text | tidak | | — | CHECK (`success`, `failure`) |
| outcome_reason_code | text | ya | | — | |
| correlation_id | uuid | tidak | | — | |
| canonical_version | smallint | tidak | | — | Versi bentuk kanonis untuk perhitungan hash |
| prev_hash | bytea | tidak | | — | `event_hash` event sebelumnya (ADR-009 butir 5) |
| event_hash | bytea | tidak | | — | SHA-256(`prev_hash` ‖ bentuk kanonis) |

Index (per partisi): `ix (recorded_at DESC)`; `ix (actor_account_id, recorded_at)`; `ix (subject_id, recorded_at)`; `ix (event_code, recorded_at)`; `ix (object_type, object_id)`; `ix (tenant_id, event_code, recorded_at)`; `ix (source_ip, recorded_at)`; `ix (correlation_id)`; `ix (seq)`. Filter US-1502 AC1 tercakup. Trigger menolak `UPDATE`, `DELETE`, dan `TRUNCATE` `[SRC-40]`. Retensi `[TIDAK DIKETAHUI]` (PT-24); partisi lebih tua dari 24 bulan diarsipkan dan di-*detach* (ADR-009 butir 9) `[SRC-39]`. Di database audit RME, `audit_event` dan `audit_event_subject` memakai RLS per `tenant_id` untuk role `audit_tenant_reader` (§0.6 B). Index BRIN untuk `recorded_at` dievaluasi di Tahap 5 (kemampuannya belum ada SRC) (review DBA S07).

Catatan F1-T21 (review gabungan F1-T19/F1-T21): CHECK `event_code` fisik memuat `e_01` … `e_18`. E-18 (impor data peserta korporat dan pengiriman undangan) ditambahkan PRD v1.3 E-18 sebagai bagian CR-021 dan sudah ditulis crate audit; tanpa nilai ini relay gagal menyisipkan event E-18 dan berhenti. Migrasi: `db/migrations/audit/20261120110000_audit_event_chain_archive.sql`.

#### B2. `audit_event_key` (L, tidak dipartisi)

Penjamin keunikan global event dan nomor urut. Relay menulis baris ini dalam transaksi yang sama dengan `audit_event`; pengiriman ulang dari outbox setelah relay gagal di tengah jalan ditolak constraint lalu dilewati `[KEPUTUSAN-56]`.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| event_id | uuid | tidak | PK | — | |
| seq | bigint | tidak | UQ | — | |
| recorded_at | timestamptz | tidak | | — | Penunjuk partisi |

Baris untuk partisi yang diarsipkan ikut dicatat di manifest arsip dan dipertahankan selama retensi audit. Pemangkasan baris untuk partisi yang sudah diarsipkan (keunikan cukup dijaga jendela daring karena `event_id` UUIDv7 berurutan waktu) dievaluasi di Tahap 5 dan setelah PT-24 (review DBA S07).

#### B3. `audit_event_subject` (L, dipartisi bulanan)

Semua subjek data untuk event yang menyentuh lebih dari satu subjek (ADR-009 butir 7), misalnya export laporan berisi banyak pasien. Tanpa FK deklaratif ke `audit_event`; konsistensi dijaga relay dalam transaksi yang sama.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| id | uuid | tidak | PK (bersama `recorded_at`) | — | |
| recorded_at | timestamptz | tidak | PK, kunci partisi | — | Sama dengan event induk |
| event_id | uuid | tidak | | — | |
| subject_id | uuid | tidak | | PII | |
| subject_kind | text | tidak | | — | CHECK (`care_recipient`, `rme_patient`) |

Index: `ix (subject_id, recorded_at)`; `ix (event_id)`.

#### B4. `audit_chain_anchor` (L)

Kepala rantai yang ditulis setiap jam ke object storage WORM dan security log (ADR-009 butir 5).

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-L* | | | | | |
| seq | bigint | tidak | UQ | — | |
| event_hash | bytea | tidak | | — | |
| anchored_at | timestamptz | tidak | | — | |
| worm_object_key | text | tidak | | — | |

#### B5. `audit_chain_verification` (L)

Hasil verifikasi rantai harian (ADR-009 butir 5). Ketidakcocokan memicu peringatan keamanan (ADR-010).

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-L* | | | | | |
| started_at | timestamptz | tidak | | — | |
| finished_at | timestamptz | tidak | | — | |
| from_seq | bigint | tidak | | — | |
| to_seq | bigint | tidak | | — | |
| audit_chain_anchor_id | uuid | ya | FK→audit_chain_anchor | — | |
| outcome | text | tidak | | — | CHECK (`match`, `mismatch`, `error`) |
| first_mismatch_seq | bigint | ya | | — | |

#### B6. `audit_archive_manifest` (L)

Catatan partisi yang diekspor ke WORM lalu di-*detach* (ADR-009 butir 9).

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-L* | | | | | |
| partition_name | text | tidak | UQ | — | |
| range_start | timestamptz | tidak | | — | |
| range_end | timestamptz | tidak | | — | |
| from_seq | bigint | tidak | | — | |
| to_seq | bigint | tidak | | — | |
| boundary_event_hash | bytea | tidak | | — | Kepala rantai pada batas partisi |
| archive_object_key | text | tidak | | — | |
| archive_sha256 | bytea | tidak | | — | |
| exported_at | timestamptz | tidak | | — | |
| detached_at | timestamptz | tidak | | — | |

---

## C. Database aplikasi RME minimalis

Database terpisah dari Ruang Sehat (ADR-006 butir 2). Hanya pasien yang datang lewat Ruang Sehat yang tercatat (BR-126, US-1306 AC4). Semua tabel milik tenant punya `fasyankes_id uuid NOT NULL FK→fasyankes` (kecuali disebut lain) dan **RLS: ya** dengan `FORCE ROW LEVEL SECURITY` `[SRC-34]` (ADR-012 butir 6, `[KEPUTUSAN-27]`). Baris *STD-M* dan *STD-T* di bagian ini memakai FK aktor ke `rme_user`. Autentikasi pengguna RME memakai Keycloak realm `rme`; password, MFA, dan sesi tidak disimpan di database ini `[USER] T4-Q4`.

Ruang Sehat dan RME tidak berbagi database, sehingga kolom yang menunjuk entitas Ruang Sehat (`ruang_sehat_partner_id`, `ruang_sehat_order_id`, `ruang_sehat_health_worker_id`, `rs_recipient_alias`) **tanpa FK deklaratif**; konsistensinya dari kontrak integrasi (ADR-014).

**FK komposit tenant** (review DBA S01). Setiap tabel tenant yang menjadi induk punya `UQ (fasyankes_id, id)`. Setiap FK antar-tabel tenant ditulis sebagai `(fasyankes_id, <induk>_id)` → `<induk> (fasyankes_id, id)`, termasuk ke `rme_data_key`, ke `rme_stored_file`, dan ke `rme_user` untuk kolom penulis. Dengan begitu baris tidak pernah dapat merujuk baris milik fasyankes lain, walaupun RLS salah konfigurasi. Di tabel-tabel bagian C, kolom bertanda "FK→x" berarti FK komposit ini; FK ke `fasyankes` sendiri tetap tunggal. FK aktor STD-M/STD-T (`created_by` dan kawan-kawan) tetap tunggal karena operator RME tidak punya `fasyankes_id`. Wadah DEK RME juga diperiksa dengan FK komposit `(fasyankes_id, data_key_id, <wadah>_id)` → `rme_data_key (fasyankes_id, id, container_id)`. Perilaku FK terhadap RLS diverifikasi di Tahap 5.

### C1. Fasyankes, kredensial, dan pengguna

#### C1.1 `fasyankes` (M, RLS: ya, kebijakan `id` = konteks tenant)

Klinik atau lab rekanan pengguna RME; tepat satu per rekanan bersumber sistem RME minimalis (PRD §8.3). Sumber: BR-120, BR-121, US-1301, UP-49, UP-50.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-M* | | | | | Dibuat Operator RME |
| ruang_sehat_partner_id | uuid | tidak | UQp | — | ID rekanan di Ruang Sehat |
| name | text | tidak | | — | |
| facility_kind | text | tidak | | — | CHECK (`clinic`, `lab`) |
| facility_registration_code | text | ya | | — | Kode registrasi fasyankes (US-1301 AC1). Rincian dan maknanya `[TIDAK DIKETAHUI]` (PT-32) |
| address_line | text | tidak | | — | |
| city_name | text | tidak | | — | |
| time_zone | text | tidak | | — | IANA |
| agreement_reference | text | tidak | | — | Perjanjian penggunaan RME dan pemrosesan data wajib tercatat sebelum aktif (UP-49); isinya menunggu L-05/L-15 |
| agreement_recorded_at | timestamptz | tidak | | — | |
| lifecycle_status | text | tidak | | — | CHECK (`onboarding`, `active`, `offboarding`, `closed`) (US-1301 AC4) |
| activated_at | timestamptz | ya | | — | |

Index: `UQp (ruang_sehat_partner_id) WHERE deleted_at IS NULL`.

#### C1.2 `fasyankes_location` (M, RLS: ya)

Lokasi pelayanan dan ID Location SATUSEHAT (ADR-020 butir 3).

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-M* | | | | | |
| fasyankes_id | uuid | tidak | FK→fasyankes | — | |
| name | text | tidak | | — | |
| satusehat_location_id | text | ya | | — | |

#### C1.3 `rs_integration_credential` (M, RLS: ya)

Kredensial RME sebagai klien Partner Integration API untuk satu fasyankes: rahasia klien service-account di Keycloak realm `integrasi` dan rahasia verifikasi webhook dari Ruang Sehat (ADR-006 butir 2, ADR-014 §3, `[USER] T4-Q4`).

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-M* | | | | | |
| fasyankes_id | uuid | tidak | FK→fasyankes, UQp | — | |
| idp_client_id | text | tidak | | — | |
| client_secret_enc | bytea | tidak | | Rahasia, Enc | |
| webhook_secret_enc | bytea | tidak | | Rahasia, Enc | Memverifikasi tanda tangan HMAC webhook Ruang Sehat |
| data_key_id | uuid | tidak | FK→rme_data_key | — | Wadah `rs_integration_credential` |
| rotated_at | timestamptz | tidak | | — | |

#### C1.4 `satusehat_credential` (M, RLS: ya)

Kode akses SATUSEHAT per fasyankes (Organization ID, Client ID, Client Secret) `[SRC-57]` via ADR-020 butir 1. Cara fasyankes memperolehnya `[TIDAK DIKETAHUI]` (PT-32, US-1309 AC3).

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-M* | | | | | Diisi Admin Fasyankes |
| fasyankes_id | uuid | tidak | FK→fasyankes, UQp | — | |
| organization_id | text | ya | | — | |
| client_id | text | ya | | — | |
| client_secret_enc | bytea | ya | | Rahasia, Enc | |
| data_key_id | uuid | tidak | FK→rme_data_key | — | |
| completed_at | timestamptz | ya | | — | CHECK `completed_at IS NULL OR (organization_id IS NOT NULL AND client_id IS NOT NULL AND client_secret_enc IS NOT NULL)`. `NULL` = status "Menunggu kredensial" (KEPUTUSAN-16) |

#### C1.5 `external_access_token` (O, RLS: ya)

Token akses berumur pendek ke Ruang Sehat (dari Keycloak realm `integrasi`) dan ke SATUSEHAT, disimpan sampai kedaluwarsa (ADR-020 butir 1).

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-O* | | | | | `expires_at` dari penerbit |
| fasyankes_id | uuid | tidak | FK→fasyankes | — | |
| target | text | tidak | | — | CHECK (`ruang_sehat`, `satusehat`) |
| access_token_enc | bytea | tidak | | Rahasia, Enc | |
| data_key_id | uuid | tidak | FK→rme_data_key | — | |

Index: `UQ (fasyankes_id, target)`.

#### C1.6 `rme_user` (M, RLS: ya; kebijakan pemetaan untuk `rme_auth`)

Pengguna RME minimalis, dipetakan ke Keycloak realm `rme`. Operator RME (staf PT. Arunika) memakai tabel yang sama dengan `user_kind = 'operator'` dan tanpa `fasyankes_id`, sehingga tidak pernah terlihat oleh kebijakan tenant `rme_app` `[KEPUTUSAN-55]`. Sumber: BR-120, US-1302, UP-26, `[USER] T4-Q4`.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-M* | | | | | |
| user_kind | text | tidak | | — | CHECK (`facility_user`, `operator`) |
| fasyankes_id | uuid | ya | FK→fasyankes | — | CHECK `(user_kind = 'operator') = (fasyankes_id IS NULL)` |
| idp_subject | text | tidak | UQp | PII | ID pengguna di realm `rme` |
| email | text | tidak | | PII | Salinan untuk notifikasi. Anonimkan |
| full_name | text | tidak | | PII | Anonimkan |
| status | text | tidak | | — | CHECK (`pending_activation`, `active`, `inactive`). Penonaktifan mengakhiri akses seketika (US-1302 AC4) lewat pencabutan sesi di Redis (§0.10) |
| status_reason | text | ya | | — | |

Index: `UQp (idp_subject) WHERE deleted_at IS NULL`; `ix (fasyankes_id) WHERE deleted_at IS NULL`.

#### C1.7 `rme_user_role` (M, RLS: ya)

Peran pengguna fasyankes. Admin Fasyankes terakhir tidak dapat dinonaktifkan (US-1302 AC5), ditegakkan aplikasi. Sumber: US-1302, UP-26, §3.1.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-M* | | | | | Pencabutan = soft delete (E-11) |
| fasyankes_id | uuid | tidak | FK→fasyankes | — | |
| rme_user_id | uuid | tidak | FK→rme_user | — | |
| role | text | tidak | | — | CHECK (`facility_admin`, `facility_health_worker`, `lab_officer`) |

Index: `UQp (rme_user_id, role) WHERE deleted_at IS NULL`; `ix (fasyankes_id, role) WHERE deleted_at IS NULL`.

#### C1.8 `rme_health_worker_profile` (M, RLS: ya)

Identitas profesi Tenaga Kesehatan Fasyankes untuk pencarian praktisi SATUSEHAT dengan NIK (ADR-020 butir 3). Field identitas lain yang dibutuhkan SATUSEHAT `[TIDAK DIKETAHUI]` (US-1302 AC2).

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-M* | | | | | |
| fasyankes_id | uuid | tidak | FK→fasyankes | — | |
| rme_user_id | uuid | tidak | FK→rme_user, UQp | — | |
| profession | text | tidak | | — | |
| nik_enc | bytea | tidak | | Spesifik, Enc | |
| nik_bidx | bytea | tidak | | PII | Anonimkan |
| nik_bidx_key_version | smallint | tidak | | — | |
| nik_last4 | char(4) | tidak | | PII | Anonimkan |
| data_key_id | uuid | tidak | FK→rme_data_key | — | Wadah `rme_health_worker_identity` |
| satusehat_practitioner_ihs | text | ya | | PII | Nomor IHS praktisi. Anonimkan |

#### C1.9 `rs_health_worker_roster` (M, RLS: ya)

Salinan daftar tenaga kesehatan aktif klinik dari webhook Ruang Sehat (EP-14, ADR-014 §2). Dipakai untuk penugasan home visit (US-1308 AC2) dan tautan akun (US-1302 AC6).

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-M* | | | | | |
| fasyankes_id | uuid | tidak | FK→fasyankes | — | |
| ruang_sehat_health_worker_id | uuid | tidak | | — | |
| full_name | text | tidak | | PII | Anonimkan |
| profession | text | tidak | | — | |
| is_active | boolean | tidak | | — | |
| synced_at | timestamptz | tidak | | — | |

Index: `UQp (fasyankes_id, ruang_sehat_health_worker_id) WHERE deleted_at IS NULL`.

#### C1.10 `rme_health_worker_link` (M, RLS: ya)

Tautan akun Tenaga Kesehatan Fasyankes ↔ tenaga kesehatan Ruang Sehat, sehingga ia dapat menulis rekam medis home visit yang ditugaskan kepadanya (US-1302 AC6, US-1306 AC6).

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-M* | | | | | `created_by` = Admin Fasyankes |
| fasyankes_id | uuid | tidak | FK→fasyankes | — | |
| rme_user_id | uuid | tidak | FK→rme_user, UQp | — | |
| rs_health_worker_roster_id | uuid | tidak | FK→rs_health_worker_roster, UQp | — | |

Index: `UQp (rme_user_id) WHERE deleted_at IS NULL`; `UQp (rs_health_worker_roster_id) WHERE deleted_at IS NULL`.

#### C1.11 `rme_session_revocation` (O, RLS: ya)

Sama dengan A1.4 untuk pengguna RME (`rme_user_id` FK→`rme_user`, `fasyankes_id`, `idp_session_id` UQ, `reason_code`, `created_at`, `expires_at`). Sumber: ADR-011 butir 8, US-1302 AC4.

### C2. Layanan, jadwal, dan ketersediaan

RME berperan sebagai sistem rekanan: datanya dikirim ke Ruang Sehat lewat kontrak yang sama (AB-61) dan tampil paling lambat 15 menit (US-1303 AC1). ID baris RME menjadi `external_id` di Ruang Sehat.

#### C2.1 `rme_service` (M, RLS: ya)

Layanan klinik, pemeriksaan lab, paket MCU, dan home visit beserta harga rekanan. Sumber: BR-122, US-1303, AB-17, E-14.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-M* | | | | | |
| fasyankes_id | uuid | tidak | FK→fasyankes | — | |
| service_kind | text | tidak | | — | CHECK (`clinic_service`, `lab_test`, `mcu_package`, `home_visit`) |
| name | text | tidak | | — | |
| description | text | ya | | — | |
| preparation_info | text | ya | | — | |
| partner_price_idr | bigint | tidak | | — | CHECK `>= 0` |
| code_system, code, code_display | text | ya | | — | LOINC untuk pemeriksaan lab (ADR-020 butir 5) |
| is_active | boolean | tidak | | — | |

Index: `UQ (fasyankes_id, id)`; `ix (fasyankes_id, service_kind) WHERE deleted_at IS NULL` (review DBA S-N1).

#### C2.2 `rme_mcu_package_item` (M, RLS: ya)

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-M* | | | | | |
| fasyankes_id | uuid | tidak | FK→fasyankes | — | |
| package_service_id | uuid | tidak | FK→rme_service | — | |
| lab_test_service_id | uuid | ya | FK→rme_service | — | |
| item_name | text | tidak | | — | |
| sort_order | smallint | tidak | | — | |

Index: `ix (fasyankes_id, package_service_id) WHERE deleted_at IS NULL` (review DBA S-N1).

#### C2.3 `rme_home_visit_area` (M, RLS: ya)

Area layanan home visit dengan kode wilayah yang sama seperti kontrak (`[KEPUTUSAN-60]`).

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-M* | | | | | |
| fasyankes_id | uuid | tidak | FK→fasyankes | — | |
| region_level | text | tidak | | — | CHECK (`province`, `city`, `district`) |
| region_code | text | tidak | | — | |

Index: `UQp (fasyankes_id, region_level, region_code) WHERE deleted_at IS NULL`.

#### C2.4 `rme_schedule_rule` (M, RLS: ya)

Jadwal mingguan per layanan: hari, jam, panjang slot, dan kuota per slot (US-1303 AC1).

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-M* | | | | | |
| fasyankes_id | uuid | tidak | FK→fasyankes | — | |
| service_id | uuid | tidak | FK→rme_service | — | |
| day_of_week | smallint | tidak | | — | CHECK `BETWEEN 1 AND 7` |
| local_start_time | time | tidak | | — | |
| local_end_time | time | tidak | | — | CHECK `> local_start_time` |
| slot_minutes | integer | tidak | | — | CHECK `> 0` |
| quota_per_slot | integer | tidak | | — | CHECK `> 0` |
| time_zone | text | tidak | | — | |
| valid_from | date | tidak | | — | |
| valid_until | date | ya | | — | |

Index: `UQ (fasyankes_id, id)`; `ix (fasyankes_id, service_id) WHERE deleted_at IS NULL` (review DBA S-N1).

#### C2.5 `rme_schedule_exception` (M, RLS: ya)

Hari libur fasyankes (US-1303 AC1).

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-M* | | | | | |
| fasyankes_id | uuid | tidak | FK→fasyankes | — | |
| exception_date | date | tidak | | — | |
| reason | text | ya | | — | |

Index: `UQp (fasyankes_id, exception_date) WHERE deleted_at IS NULL`.

#### C2.6 `rme_slot` (M, RLS: ya)

Slot terbentuk sebagai instan. Slot dengan pesanan terbayar tidak dapat dihapus atau diubah sampai pesanan dibatalkan rekanan (US-1303 AC3–AC4).

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-M* | | | | | |
| fasyankes_id | uuid | tidak | FK→fasyankes | — | |
| service_id | uuid | tidak | FK→rme_service | — | |
| schedule_rule_id | uuid | ya | FK→rme_schedule_rule | — | |
| start_at | timestamptz | tidak | | — | |
| end_at | timestamptz | tidak | | — | CHECK `> start_at` |
| quota | integer | tidak | | — | CHECK `>= 0` |

Index: `UQp (service_id, start_at) WHERE deleted_at IS NULL`.

#### C2.7 `rme_slot_hold` (O, RLS: ya)

Penahanan slot yang diminta Ruang Sehat lewat pemeriksaan sinkron sebelum bayar (ADR-014 §2). ID baris dikembalikan sebagai ID penahanan. Kuota tersisa = `quota` − kunjungan aktif − penahanan berlaku, dihitung dalam transaksi dengan penguncian baris slot.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-O* | | | | | `expires_at` minimal PB-07 (ADR-014 §2) |
| fasyankes_id | uuid | tidak | FK→fasyankes | — | |
| slot_id | uuid | tidak | FK→rme_slot | — | |
| quantity | integer | tidak | | — | CHECK `> 0` |
| ruang_sehat_order_id | uuid | ya | | — | |

Index: `ix (slot_id, expires_at)` (perhitungan kuota tersisa dan pembersihan, review DBA S-N1).

### C3. Pasien, kunjungan, dan antrean

#### C3.1 `rme_patient` (M, RLS: ya)

Pasien fasyankes; hanya pasien Ruang Sehat (BR-126). Dikenali lewat alias penerima per rekanan dari Ruang Sehat (`[KEPUTUSAN-61]`). NIK hanya diisi bila field NIK di kontrak sudah diaktifkan setelah L-05 dan L-08 (ADR-020 butir 4). Sumber: US-1304, US-1310, ADR-020 butir 3–4.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-M* | | | | | |
| fasyankes_id | uuid | tidak | FK→fasyankes | — | |
| rs_recipient_alias | uuid | tidak | | PII | `partner_recipient_alias.alias_id` |
| full_name | text | tidak | | PII, Spesifik* | Anonimkan |
| birth_date | date | tidak | | PII, Spesifik* | Anonimkan |
| sex | text | tidak | | PII | CHECK (`male`, `female`). Anonimkan |
| nik_enc | bytea | ya | | Spesifik, Enc | `[TIDAK DIKETAHUI]` apakah diaktifkan (L-05, L-08) |
| nik_bidx | bytea | ya | | PII | Anonimkan |
| nik_bidx_key_version | smallint | ya | | — | |
| nik_last4 | char(4) | ya | | PII | Anonimkan |
| data_key_id | uuid | ya | FK→rme_data_key | — | Wadah `rme_patient_identity`; wajib bila `nik_enc` terisi (CHECK) |
| satusehat_patient_ihs | text | ya | | PII | Nomor IHS pasien. Anonimkan |
| anonymized_at | timestamptz | ya | | — | |

Index: `UQp (fasyankes_id, rs_recipient_alias) WHERE deleted_at IS NULL`.

#### C3.2 `rme_visit` (T, RLS: ya)

Kunjungan; tepat satu per pesanan Ruang Sehat (PRD §8.3, BR-126). Satu kunjungan = satu wadah `rme_visit`. Sumber: BR-123, BR-125, US-1304, US-1308.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-T* | | | | | |
| fasyankes_id | uuid | tidak | FK→fasyankes | — | |
| rme_patient_id | uuid | tidak | FK→rme_patient | — | |
| ruang_sehat_order_id | uuid | tidak | UQ | — | |
| visit_kind | text | tidak | | — | CHECK (`clinic`, `lab`, `home_visit`) |
| service_id | uuid | tidak | FK→rme_service | — | |
| slot_id | uuid | ya | FK→rme_slot | — | |
| location_id | uuid | ya | FK→fasyankes_location | — | |
| scheduled_start_at | timestamptz | tidak | | — | |
| scheduled_end_at | timestamptz | tidak | | — | |
| check_in_code | text | ya | | — | Kode dari Ruang Sehat; `NULL` untuk home visit |
| data_key_id | uuid | tidak | FK→rme_data_key, UQ | — | Wadah `rme_visit`; FK komposit `(fasyankes_id, data_key_id, id)` → `rme_data_key (fasyankes_id, id, container_id)` |

Index: `UQ (fasyankes_id, id)` (induk FK komposit); `UQ (ruang_sehat_order_id)`; `UQp (fasyankes_id, check_in_code) WHERE check_in_code IS NOT NULL` (US-1304 AC3); `ix (fasyankes_id, scheduled_start_at)` (daftar harian, US-1304 AC1); `ix (rme_patient_id, scheduled_start_at)` (riwayat, US-1310); `ix (slot_id) WHERE slot_id IS NOT NULL` (perhitungan kuota, review DBA S-N1); `UQ (fasyankes_id, id, data_key_id)` (sasaran FK wadah dari rekam medis dan hasil lab, review DBA S-N4).

#### C3.3 `rme_visit_personal_snapshot` (M-snapshot, RLS: ya)

Alamat dan nomor HP untuk kunjungan home visit. Sub-kelas Master snapshot `[KEPUTUSAN-69]`: role aplikasi hanya S dan I; `UPDATE` hanya `privacy_worker` pada kolom "Anonimkan" (`[KEPUTUSAN-42]`). Sumber: US-1308 AC1.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-M* | | | | | |
| fasyankes_id | uuid | tidak | FK→fasyankes | — | |
| rme_visit_id | uuid | tidak | FK→rme_visit, UQp | — | |
| phone | text | ya | | PII | Anonimkan |
| address_line | text | ya | | PII | Anonimkan |
| region_code | text | ya | | PII | Anonimkan |
| latitude, longitude | numeric(9,6) | ya | | PII | Anonimkan |
| address_notes | text | ya | | PII | Anonimkan |
| time_zone | text | ya | | — | |
| anonymized_at | timestamptz | ya | | — | |

#### C3.4 `rme_visit_status_event` (R, RLS: ya)

*STD-R varian RME* (§0.4) dengan `rme_visit_id`. `to_status` CHECK (`scheduled`, `checked_in`, `completed`, `cancelled`, `patient_no_show`, `assigned`, `departed`, `arrived`, `health_worker_no_show`, `closed_unconfirmed`). Status home visit dari aplikasi Ruang Sehat diterima sebagai event beraktor `ruang_sehat_system` (US-1308 AC3). Check-in yang ditolak (US-1304 AC3) tidak menghasilkan event.

#### C3.5 `rme_visit_state` (P, RLS: ya)

*STD-P* dengan `rme_visit_id`, ditambah `fasyankes_id`.

#### C3.6 `rme_queue_ticket` (T, RLS: ya)

Nomor antrean berurutan per layanan per hari (US-1305 AC1, UP-12).

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-T* | | | | | |
| fasyankes_id | uuid | tidak | FK→fasyankes | — | |
| rme_visit_id | uuid | tidak | FK→rme_visit, UQ | — | |
| service_id | uuid | tidak | FK→rme_service | — | |
| queue_date | date | tidak | | — | |
| queue_number | integer | tidak | | — | CHECK `> 0` |

Index: `UQ (fasyankes_id, service_id, queue_date, queue_number)`.

#### C3.7 `rme_queue_call_event` (T, RLS: ya)

Panggilan, lewati, dan panggil ulang (US-1305 AC2–AC3). Nomor yang sedang dilayani = panggilan terakhir.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-T* | | | | | |
| fasyankes_id | uuid | tidak | FK→fasyankes | — | |
| rme_queue_ticket_id | uuid | tidak | FK→rme_queue_ticket | — | |
| event_type | text | tidak | | — | CHECK (`called`, `skipped`, `recalled`) |

Index: `ix (fasyankes_id, created_at DESC)`.

#### C3.8 `rme_home_visit_assignment` (T, RLS: ya)

Penugasan tenaga kesehatan dari daftar aktif klinik (US-1308 AC2). Penugasan terbaru berlaku.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-T* | | | | | `created_by` = Admin Fasyankes |
| fasyankes_id | uuid | tidak | FK→fasyankes | — | |
| rme_visit_id | uuid | tidak | FK→rme_visit | — | |
| rs_health_worker_roster_id | uuid | tidak | FK→rs_health_worker_roster | — | |

#### C3.9 `rme_home_visit_assignment_result` (T, RLS: ya)

Jawaban Ruang Sehat atas penugasan: diterima atau ditolak beserta alasannya (US-502 AC2).

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-T* | | | | | |
| fasyankes_id | uuid | tidak | FK→fasyankes | — | |
| rme_home_visit_assignment_id | uuid | tidak | FK→rme_home_visit_assignment, UQ | — | |
| result | text | tidak | | — | CHECK (`accepted`, `rejected`) |
| reason_code | text | ya | | — | |

### C4. Rekam medis dan hasil lab

Isi klinis disimpan sebagai dokumen JSON terenkripsi per versi dengan field berkode `{system, code, display}` `[KEPUTUSAN-52]`, dengan struktur yang sama seperti catatan konsultasi Ruang Sehat (AB-72). Daftar final kelompok data, variabel, dan terminologi `[TIDAK DIKETAHUI]` sampai L-05 (US-1306 AC2); pemetaan variabel final ditetapkan saat pengembangan US-1306 dari playbook SATUSEHAT (ADR-020 butir 5).

#### C4.1 `rme_medical_record_draft` (O, RLS: ya)

Draf rekam medis sebelum difinalkan (ADR-008 butir 5).

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-O* | | | | | Dihapus setelah difinalkan |
| fasyankes_id | uuid | tidak | FK→fasyankes | — | |
| rme_visit_id | uuid | tidak | FK→rme_visit, UQ | — | |
| author_rme_user_id | uuid | tidak | FK→rme_user | — | |
| payload_enc | bytea | tidak | | Spesifik, Enc | |
| payload_schema_version | smallint | tidak | | — | |
| data_key_id | uuid | tidak | FK→rme_data_key | — | Wadah kunjungan; FK komposit `(fasyankes_id, rme_visit_id, data_key_id)` → `rme_visit (fasyankes_id, id, data_key_id)` (review DBA S-N4) |
| updated_at | timestamptz | tidak | | — | |

#### C4.2 `rme_medical_record_version` (T, RLS: ya)

Versi rekam medis kunjungan atas nama penulis dan fasyankes (US-1306 AC1, AC3). Hanya Tenaga Kesehatan Fasyankes yang dapat menulis (US-1306 AC5); untuk home visit, hanya yang terhubung ke tenaga kesehatan yang ditugaskan (US-1306 AC6). Sumber: BR-123, US-1306, E-06, UP-27.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-T* | | | | | |
| fasyankes_id | uuid | tidak | FK→fasyankes | — | |
| rme_visit_id | uuid | tidak | FK→rme_visit | — | |
| version | integer | tidak | | — | CHECK `>= 1` |
| author_rme_user_id | uuid | tidak | FK→rme_user | — | |
| payload_enc | bytea | tidak | | Spesifik, Enc | Struktur di bawah |
| payload_schema_version | smallint | tidak | | — | |
| correction_reason_enc | bytea | ya | | Spesifik, Enc | CHECK `version = 1 OR correction_reason_enc IS NOT NULL` |
| data_key_id | uuid | tidak | FK→rme_data_key | — | Wadah kunjungan; FK komposit `(fasyankes_id, rme_visit_id, data_key_id)` → `rme_visit (fasyankes_id, id, data_key_id)` (review DBA S-N4) |

Index: `UQ (rme_visit_id, version)`.

Struktur `payload` per kelompok data UP-27 dan resource FHIR tujuannya (ADR-020 butir 5):

| Kelompok data (UP-27) | Field | Kode | Resource FHIR |
| --- | --- | --- | --- |
| Kunjungan | dari `rme_visit` | — | Encounter |
| Keluhan / anamnesis | `complaints[]` `{text, coding?}` | SNOMED CT bila afiliasi tersedia (PT-32) | Condition |
| Pemeriksaan fisik, tanda vital | `vital_signs[]` `{coding, value, unit}`, `physical_exam_text` | LOINC atau SNOMED CT `[TIDAK DIKETAHUI]` (L-05) | Observation |
| Diagnosis | `diagnoses[]` `{coding, rank}` | ICD-10 versi 2010 | Condition |
| Tindakan | `procedures[]` `{coding, text}` | `[TIDAK DIKETAHUI]` (L-05) | Procedure |
| Peresepan obat | `medications[]` `{coding?, name, dose, quantity, instructions}` | KFA `[TIDAK DIKETAHUI]` (ADR-020 butir 7) | MedicationRequest |
| Permintaan pemeriksaan lab | `lab_requests[]` `{coding}` | LOINC | ServiceRequest |
| Resume | `summary_text` | — | Composition |

#### C4.3 `rme_lab_order_item` (T, RLS: ya)

Pemeriksaan yang dipesan lewat pendaftaran lab Ruang Sehat (permintaan lab). Sumber: BR-124, PRD §8.2.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-T* | | | | | |
| fasyankes_id | uuid | tidak | FK→fasyankes | — | |
| rme_visit_id | uuid | tidak | FK→rme_visit | — | |
| service_id | uuid | tidak | FK→rme_service | — | |
| code_system, code, code_display | text | ya | | — | LOINC |

Index: `ix (fasyankes_id, rme_visit_id)` (review DBA S-N1).

#### C4.4 `rme_lab_result` (T, RLS: ya)

Satu hasil lab dalam satu kunjungan; satu kunjungan dapat punya beberapa hasil (misalnya hasil sebagian), masing-masing dengan rantai versinya sendiri, sejalan dengan `lab_result` di Ruang Sehat (PRD §8.3) (review DBA B08). `id` dikirim ke Ruang Sehat sebagai `external_result_id`. Sumber: BR-124, US-1307, US-901.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-T* | | | | | |
| fasyankes_id | uuid | tidak | FK→fasyankes | — | |
| rme_visit_id | uuid | tidak | FK→rme_visit | — | Kunjungan berjenis `lab` |
| data_key_id | uuid | tidak | FK→rme_data_key | — | Wadah kunjungan; FK komposit `(fasyankes_id, rme_visit_id, data_key_id)` → `rme_visit (fasyankes_id, id, data_key_id)` (review DBA S-N4) |

Index: `UQ (fasyankes_id, id)`; `UQ (id, data_key_id)` (sasaran FK wadah dari versi); `ix (rme_visit_id)`.

#### C4.5 `rme_lab_result_version` (T, RLS: ya)

Versi hasil lab yang diisi atau diunggah Petugas Lab, lalu dikirim ke Ruang Sehat (US-1307, US-901). Apakah hasil wajib divalidasi penanggung jawab lab `[TIDAK DIKETAHUI]` (L-05, US-1307 AC3).

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-T* | | | | | |
| fasyankes_id | uuid | tidak | FK→fasyankes | — | |
| rme_lab_result_id | uuid | tidak | FK→rme_lab_result | — | |
| version | integer | tidak | | — | CHECK `>= 1` |
| author_rme_user_id | uuid | tidak | FK→rme_user | — | |
| payload_enc | bytea | ya | | Spesifik, Enc | `tests[]` `{coding LOINC, value, unit, reference_range}` (US-1307 AC1) |
| pdf_file_id | uuid | ya | FK→rme_stored_file | Spesifik | |
| correction_reason_enc | bytea | ya | | Spesifik, Enc | CHECK `version = 1 OR correction_reason_enc IS NOT NULL` (US-1307 AC2) |
| payload_schema_version | smallint | tidak | | — | |
| data_key_id | uuid | tidak | FK→rme_data_key | — | Wadah kunjungan induk; FK komposit `(rme_lab_result_id, data_key_id)` → `rme_lab_result (id, data_key_id)` (review DBA S-N4) |

CHECK `payload_enc IS NOT NULL OR pdf_file_id IS NOT NULL`. Index: `UQ (rme_lab_result_id, version)`.

### C5. SATUSEHAT dan terminologi

#### C5.1 `satusehat_submission` (L, RLS: ya)

Catatan pengiriman per versi rekam medis; koreksi mendapat catatan pengiriman sendiri (US-1309 AC4). Status disimpan dengan pola riwayat + proyeksi supaya catatan pengiriman tetap tidak diubah `[KEPUTUSAN-65]`. Pengiriman hasil lab ke SATUSEHAT `[TIDAK DIKETAHUI]` (L-05). Sumber: BR-123, US-1309, KEPUTUSAN-16, ADR-020 butir 6.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-L* | | | | | |
| fasyankes_id | uuid | tidak | FK→fasyankes | — | |
| rme_medical_record_version_id | uuid | ya | FK→rme_medical_record_version, UQp | — | |
| rme_lab_result_version_id | uuid | ya | FK→rme_lab_result_version, UQp | — | |

CHECK: tepat satu dari kedua FK terisi.

#### C5.2 `satusehat_submission_status_event` (R, RLS: ya)

*STD-R varian RME* (§0.4) dengan `satusehat_submission_id`. `to_status` CHECK (`awaiting_credentials`, `queued`, `sent`, `failed`). `reason_code` memuat golongan galat (`transient`, `clinical_data`, `facility_identity`, `credential`) (ADR-020 butir 6, US-1309 AC2). Kirim ulang oleh Admin Fasyankes = `failed → queued` (US-1309 AC5).

#### C5.3 `satusehat_submission_state` (P, RLS: ya)

*STD-P* dengan `satusehat_submission_id`, ditambah `fasyankes_id`. Index `ix (status) WHERE status IN ('awaiting_credentials','queued')`.

#### C5.4 `satusehat_submission_attempt` (L, RLS: ya)

Setiap panggilan ke SATUSEHAT. Batas 24 jam dihitung dari percobaan pertama (US-1309 AC2). Setiap pengiriman juga dicatat E-17.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-L* | | | | | |
| fasyankes_id | uuid | tidak | FK→fasyankes | — | |
| satusehat_submission_id | uuid | tidak | FK→satusehat_submission | — | |
| attempt_no | smallint | tidak | | — | |
| resource_type | text | tidak | | — | |
| outcome | text | tidak | | — | CHECK (`success`, `transient_error`, `clinical_data_error`, `facility_identity_error`, `credential_error`) |
| http_status | smallint | ya | | — | |
| error_code | text | ya | | — | Tanpa isi klinis |

Index: `ix (satusehat_submission_id, attempt_no)`.

#### C5.5 `satusehat_resource_ref` (L, RLS: ya)

ID resource yang dikembalikan SATUSEHAT, dipakai untuk mengirim koreksi sebagai pembaruan atas data yang sama (KEPUTUSAN-16, ADR-020 butir 6).

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-L* | | | | | |
| fasyankes_id | uuid | tidak | FK→fasyankes | — | |
| rme_visit_id | uuid | tidak | FK→rme_visit | — | |
| resource_type | text | tidak | | — | Misalnya `Encounter`, `Condition`, `Observation` |
| local_ref_key | text | tidak | | — | Penunjuk elemen di payload (misalnya `diagnoses[0]`) |
| satusehat_resource_id | text | tidak | | — | |

Index: `UQ (fasyankes_id, rme_visit_id, resource_type, local_ref_key)`.

#### C5.6 `terminology_code` (M, RLS: tidak, acuan global)

Tabel acuan kode yang dimuat dari sumber resmi (ADR-020 butir 7). Bukan milik tenant, sehingga tanpa RLS dan hanya dapat dibaca role aplikasi `[KEPUTUSAN-64]`. Sumber: `[SRC-61]` `[SRC-62]` via ADR-020.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-M* | | | | | |
| code_system | text | tidak | | — | CHECK (`icd10_2010`, `loinc`, `snomed_ct`, `kfa`). SNOMED CT bergantung nomor afiliasi (PT-32); KFA `[TIDAK DIKETAHUI]` |
| system_version | text | tidak | | — | |
| code | text | tidak | | — | |
| display | text | tidak | | — | |
| is_active | boolean | tidak | | — | |

Index: `UQp (code_system, system_version, code) WHERE deleted_at IS NULL`; index trigram `display WHERE deleted_at IS NULL` `[SRC-42]`.

### C6. Ekspor, berkas, kunci, dan operasional RME

#### C6.1 `rme_record_export` (T, RLS: ya)

Ekspor rekam medis fasyankes sebelum akses ditutup (US-1301 AC4, UP-50). Perlakuan data setelahnya menunggu L-05.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-T* | | | | | `created_by` = Admin Fasyankes |
| fasyankes_id | uuid | tidak | FK→fasyankes | — | |
| file_id | uuid | tidak | FK→rme_stored_file | Spesifik | |

#### C6.2 `rme_stored_file` (M, RLS: ya)

Struktur sama dengan `stored_file` (A19.6) ditambah `fasyankes_id`. Bucket CHECK (`clinical`, `export`).

#### C6.3 `rme_upload_slot` (O, RLS: ya)

Struktur sama dengan `upload_slot` (A19.5) ditambah `fasyankes_id`.

#### C6.4 `rme_data_key` (M khusus, RLS: ya)

Struktur dan aturan sama dengan `data_key` (A19.7, `[KEPUTUSAN-44]`, `[KEPUTUSAN-66]`) ditambah `fasyankes_id`; `retention_category` merujuk kategori di `rme_retention_policy` (C6.9), bukan `retention_policy` Ruang Sehat (review DBA B10). `container_type` CHECK (`rme_visit`, `rme_patient_identity`, `rme_health_worker_identity`, `satusehat_credential`, `rs_integration_credential`, `external_access_token`, `status_note`). Constraint: `UQ (fasyankes_id, id)`, `UQ (fasyankes_id, id, container_id)` (sasaran FK komposit). Trigger satu arah setelah `shredded_at` terisi, dan rotasi KEK, sama dengan A19.7. Hak per role di §0.6 (C): `rme_app`, `rme_worker`, `rme_operator` S dan I; `privacy_worker` S dan U kolom `wrapped_dek`, `shredded_at`, `shred_reason` dengan kebijakan RLS khusus role itu (review DBA B01).

#### C6.5 `rme_audit_outbox` (O, hak khusus)

Struktur sama dengan `audit_outbox` (A19.1). Payload memuat `tenant_id` = `fasyankes_id`. Tanpa RLS, karena role aplikasi hanya punya `INSERT` dan relay membaca semua baris.

#### C6.6 `rme_integration_outbox` (O, RLS: ya)

Pesan keluar ke Ruang Sehat (layanan, jadwal, slot, check-in, antrean, penugasan, hasil lab). Struktur sama dengan `integration_outbox` (A19.2) dengan `fasyankes_id` menggantikan `partner_id`, dan kunci idempotensi pesan = `event_id` (ADR-014 §4). Seperti A19.2, outbox hanya menyimpan rujukan (`entity_type`, `entity_id`); nilai hasil lab dan data lain disusun dan didekripsi saat pengiriman, bukan disimpan sebagai teks terbuka (review DBA B04). `payload_enc` memakai FK komposit ke `rme_data_key`.

#### C6.7 `rme_job` (O, RLS: ya)

Struktur sama dengan `job` (A19.4) ditambah `fasyankes_id NULL` (job lintas tenant seperti pemuatan terminologi tanpa tenant). Diambil `rme_worker` dengan kebijakan antrean `[KEPUTUSAN-55]`.

#### C6.8 `rme_inbound_event` (O, RLS: ya)

Penolak duplikat webhook dari Ruang Sehat berdasarkan `event_id` (US-1403 AC2).

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-O* | | | | | `expires_at` mengikuti masa simpan kunci idempotensi 7 hari (ADR-014 §4) |
| fasyankes_id | uuid | tidak | FK→fasyankes | — | |
| event_id | uuid | tidak | UQ | — | |
| message_type | text | tidak | | — | |

Syarat pemrosesan webhook dari Ruang Sehat oleh `rme_app` (US-1403 AC2, ADR-014 §3–§4):
1. Baris `rme_inbound_event` disisipkan **dalam transaksi yang sama** dengan perubahan bisnisnya (misalnya pembuatan `rme_visit`), sehingga event yang sudah diproses dan perubahannya selalu sama-sama tersimpan atau sama-sama batal.
2. Konteks tenant (`fasyankes_id`) diambil dari tanda tangan HMAC yang **terverifikasi** dengan rahasia `rs_integration_credential` milik fasyankes itu, bukan dari isi pesan (ADR-012).
3. Bila langkah apa pun gagal, seluruh transaksi dibatalkan dan endpoint membalas galat sementara, sehingga Ruang Sehat mengirim ulang pesan dari outbox-nya.

#### C6.9 `rme_retention_policy` (M, berlaku-mulai, RLS: tidak, acuan global)

Masa retensi dan tindakan akhir per kategori data RME. Database RME terpisah dari Ruang Sehat, sehingga punya tabel kebijakan sendiri untuk trigger DELETE Log dan `rme_data_key.retention_category` (review DBA B10). Acuan global tanpa RLS, seperti `terminology_code`, karena satu nilai berlaku untuk semua fasyankes; perbedaan per fasyankes `[TIDAK DIKETAHUI]` (L-05). Nilai kosong berarti data tidak diproses retensi sampai PT-24 terjawab (ADR-013 §6). Pola §0.8 `[KEPUTUSAN-40]`. Sumber: BR-112, NFR-RET-01, NFR-RET-02, US-1606.

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-M* | | | | | Diubah hanya lewat migrasi setelah PT-24 |
| data_category | text | tidak | | — | Misalnya `rme_medical_record`, `rme_lab_result`, `rme_patient_identity`, `rme_visit_personal_snapshot`, `satusehat_log`. Daftar final ditetapkan bersama PT-24 dan L-05 |
| retention_months | integer | ya | | — | CHECK `retention_months > 0` |
| end_action | text | ya | | — | CHECK (`end_processing`, `anonymize`, `crypto_shred`); CHECK `(retention_months IS NULL) = (end_action IS NULL)` |
| legal_basis_note | text | ya | | — | |
| effective_from | timestamptz | tidak | | — | |

Index: `UQp (data_category, effective_from) WHERE deleted_at IS NULL`.

#### C6.10 `rme_retention_run` (L, RLS: ya, per fasyankes)

Laporan hasil job retensi RME, satu baris per kategori per fasyankes per run (US-1606 AC1). Dipilih **per fasyankes dengan RLS** karena rekam medis milik fasyankes (UP-50) dan pengendali datanya menunggu L-05; Admin Fasyankes perlu dapat melihat laporan pemusnahan data fasyankesnya, sementara laporan lintas fasyankes tidak boleh terlihat. `privacy_worker` menulis dengan kebijakan RLS khusus lintas tenant (§0.6 C) (review DBA B10).

| Kolom | Tipe | Null | Kunci | Sensitivitas | Keterangan |
| --- | --- | --- | --- | --- | --- |
| *STD-L* | | | | | |
| fasyankes_id | uuid | tidak | FK→fasyankes | — | |
| rme_retention_policy_id | uuid | tidak | FK→rme_retention_policy | — | FK tunggal (tabel acuan global) |
| data_category | text | tidak | | — | |
| end_action | text | tidak | | — | |
| started_at | timestamptz | tidak | | — | |
| finished_at | timestamptz | tidak | | — | |
| items_evaluated | integer | tidak | | — | |
| items_actioned | integer | tidak | | — | |
| outcome | text | tidak | | — | CHECK (`completed`, `partial`, `failed`) |

Index: `ix (fasyankes_id, created_at DESC)`.

---

## D. Strategi partisi dan catatan untuk Tahap 5

1. **Partisi** `[KEPUTUSAN-63]`: rilis 1 hanya mempartisi `audit_event` dan `audit_event_subject` (bulanan) `[SRC-39]`. Kandidat yang dievaluasi setelah uji beban NFR-CAP-04 dan setelah PT-24 menetapkan retensi: `chat_message`, `in_app_notification`, `email_delivery`, `integration_error_log`, `outbound_delivery_attempt`, `payment_gateway_notification`, `shipment_tracking_event`. Volume target (30.000 pasien aktif bulanan, 100 sesi bersamaan, NFR-CAP) belum menuntut partisi tabel aplikasi (review DBA S08).
2. **Pembersihan Operasional**: job membersihkan baris kedaluwarsa `inbound_idempotency_key`, `inventory_hold`, `upload_slot`, `external_access_token`, `rme_slot_hold`, dan `rme_inbound_event`, baris outbox yang selesai, serta baris `job` dan `rme_job` yang selesai. Penghapusan dilakukan bertahap dalam batch kecil supaya tidak mengunci tabel lama. Pengaturan `fillfactor` dan autovacuum per tabel untuk tabel kelas O yang sering di-update atau dihapus (`job`, outbox, `queue_board`, `inventory_hold`) ditetapkan di Tahap 5 setelah diverifikasi ke dokumentasi (review DBA S08).
3. **Trigger penolak** untuk kelas T, R, L (dengan pengecualian DELETE retensi oleh `privacy_worker` di kelas L, membaca `retention_policy` di database A dan `rme_retention_policy` di database C), trigger "baris yang sudah berlaku tidak boleh diubah" untuk pola §0.8, dan trigger satu arah `data_key`/`rme_data_key` (termasuk syarat `wrapped_dek = NULL` hanya bersama `shredded_at`, S-N3) dibuat sekali sebagai pustaka migrasi `[SRC-40]`.
4. **Hal yang diverifikasi ke dokumentasi PostgreSQL di Tahap 5** sebelum dipakai (belum ada SRC): (a) constraint yang diperiksa saat commit untuk keseimbangan jurnal (A13), jumlah instruksi pembagian dana terhadap total terkunci (A12.7), dan jumlah baris harga terkunci terhadap kepalanya (A8.7) (review DBA S03); (b) pencegahan tumpang tindih rentang slot (A4.8); (c) kebijakan RLS per role, termasuk RLS pada tabel audit terpartisi (§0.6 B dan C); (d) constraint unik di tabel terpartisi (B): sudah terverifikasi, kolom constraint wajib memuat semua kolom kunci partisi `[SRC-39]`, sehingga PK `audit_event` memuat `recorded_at`; (e) perilaku FK komposit dengan kolom `NULL`, FK terhadap RLS, dan FK pada kolom terhitung (§0.2, A8.2); (f) index BRIN untuk tabel audit (B1).
5. **Nama modul pemilik tabel** diselaraskan dengan `docs/04-arsitektur.md` §3 (temuan F-12).
6. **Index tambahan**:
   - Putaran perbaikan 1 (review DBA S06): `payment_receipt (payment_attempt_id)`, `journal_entry (refund_id | payment_capture_id | partner_obligation_id)`, `order_price_lock_line (product_id)`, `appointment (partner_service_id, scheduled_start_at)`, `chat_attachment (chat_message_id)`, `claim_evidence (claim_id)`, `complaint_ticket_attachment (complaint_ticket_message_id)`, `prescription (consultation_session_id)`, `job (run_at) WHERE status = 'queued'`, `rme_lab_result (rme_visit_id)`; urutan harga katalog memakai `patient_price_idr`; index trigram dibuat partial; index `health_worker (profession)` dihapus.
   - Putaran perbaikan 2 (review DBA S-N1): `home_visit_detail (scheduled_start_at)` dan `(scheduled_end_at)`; `payment_attempt (expires_at)`; `consultation_session (scheduled_start_at)`; partial index `order_state` untuk status yang belum akhir; `payment_receipt (received_at)`; `payment_capture (created_at)`; `fund_split_report_line (reported_at)`; laporan periode jurnal lewat `journal_entry (occurred_at)` dan `journal_line (journal_account_id, occurred_at)` dengan `occurred_at` disalin ke `journal_line` (FK komposit menjamin nilainya sama); `refund (patient_order_id)` penuh; `rme_slot_hold (slot_id, expires_at)`; `rme_visit (slot_id)`; index berawalan `fasyankes_id` untuk `rme_service`, `rme_schedule_rule`, `rme_mcu_package_item`, `rme_lab_order_item`; `UQp payment_receipt (payment_gateway_notification_id)` (S-N5).
7. **Batas waktu dijalankan sebagai job per entitas** (review DBA S-N1): setiap batas PB (PB-01–PB-07, PB-15–PB-20) dan akhir durasi sesi dijadwalkan sebagai satu baris `job` dengan `run_at` = instan batas, dibuat dalam transaksi yang sama dengan kejadian pemicunya (ADR-007, ADR-017 butir 4). Job memeriksa ulang status saat berjalan dan selesai tanpa efek bila tidak lagi relevan. Pemindaian tabel (`order_state`, kolom jadwal, `payment_attempt.expires_at`) hanya jaring pengaman harian untuk menemukan entitas yang job-nya hilang; index di butir 6 mendukung pemindaian itu.

---

## E. Keputusan baru, temuan, dan butir yang belum diketahui

### E.1 Keputusan teknis baru (dicatat orkestrator di `docs/decision-log.md`)

| ID | Keputusan | Opsi lain | Alasan |
| --- | --- | --- | --- |
| KEPUTUSAN-38 | Tabel pesanan bernama `patient_order`; kata kunci SQL tidak dipakai sebagai nama tabel (`account`, bukan `user`) | `order` dengan tanda kutip | `ORDER` kata kunci SQL; ADR-001 menolak penamaan yang memerlukan tanda kutip. Contoh `order` di ADR-008 hanya ilustrasi |
| KEPUTUSAN-39 | Identifier dan kode enum berbahasa Inggris snake_case, dipetakan ke label PRD (§0.9) | Kode berbahasa Indonesia | Konsisten dengan contoh ADR-008; label tampilan tetap Bahasa Indonesia (NFR-LOC-01) |
| KEPUTUSAN-40 | Pengaturan berlaku-mulai: baris baru per perubahan dengan `effective_from`; baris yang sudah berlaku dikunci trigger; baris masa depan boleh dibatalkan | Satu baris yang di-update + audit; tabel riwayat terpisah | Riwayat tanpa membaca audit (US-809/810/812 AC1); harga terkunci merujuk baris yang tidak berubah (AB-17) |
| KEPUTUSAN-41 | Penerima layanan tunggal `care_recipient` (pemilik akun = baris `is_account_owner`); dokumen identitas per pengajuan; permintaan verifikasi sebagai transaksi berstatus; keunikan NIK lewat blind index di `care_recipient` (dalam akun) dan `account_owner_nik_registry` (lintas akun) | Tabel pasien dan anggota keluarga terpisah dengan FK ganda di pesanan; status verifikasi sebagai kolom | Satu FK untuk pesanan, dokumen, dan subjek audit; US-201 AC5 dan UP-40 ditegakkan constraint |
| KEPUTUSAN-42 | Tabel T, R, L tanpa PII teks terbuka; PII di Master yang dapat dianonimkan atau terenkripsi per wadah; notifikasi menyimpan templat + parameter ID | PII di tabel transaksi | ADR-004 melarang mengubah transaksi, sehingga PII di sana tidak dapat dianonimkan (ADR-013 §5) |
| KEPUTUSAN-43 | Perluasan enkripsi aplikasi dan wadah: nomor rekening rekanan, teks tiket keluhan, teks permintaan hak subjek data, kepala resep, alasan koreksi klinis, rahasia webhook | Hanya daftar ADR-013 §3 | Teks bebas tersebut berisi data pribadi di tabel transaksi; tanpa enkripsi tidak dapat dimusnahkan. Perlu penyesuaian ADR-013 (F-04) |
| KEPUTUSAN-44 | `data_key` tanpa soft delete; crypto-shredding = `privacy_worker` mengosongkan `wrapped_dek` dan mengisi `shredded_at` | Hapus baris DEK | Baris tetap untuk FK dari tabel transaksi; tindakan pemusnahan tetap terlihat |
| KEPUTUSAN-45 | Akun aplikasi dipetakan ke Keycloak lewat `(idp_realm, idp_subject)` unik; status akun aplikasi menentukan otorisasi di setiap permintaan, dengan cache di Redis | Status hanya di Keycloak | `[USER] T4-Q4`; otorisasi tetap di aplikasi (ADR-012); KU-10 |
| KEPUTUSAN-46 | Email di tabel akun hanya salinan untuk notifikasi, tanpa constraint unik di aplikasi; keunikan dan verifikasi email ditangani di Keycloak per realm (diverifikasi saat revisi ADR-011) | Unique email di aplikasi | Satu sumber kebenaran kredensial; realm `pasien` dan `nakes` terpisah, sehingga dokter dapat juga menjadi pasien. Perlu konfirmasi user |
| KEPUTUSAN-47 | Satu aturan jadwal per jenis konsultasi; slot sebagai baris Master dengan `booked_order_id` yang diisi kondisional; penahanan slot konsultasi unik di `inventory_hold` | Slot campuran chat/video; slot dihitung tanpa disimpan | US-302 AC1 (durasi per jenis), AB-52, KU-09 ditegakkan database |
| KEPUTUSAN-48 | Harga terkunci per percobaan pembayaran; harga pesanan = harga terkunci percobaan yang tertangkap | Satu harga terkunci per pesanan | Biaya transaksi bergantung metode (AB-12) dan pasien boleh berganti metode (US-802 AC4) |
| KEPUTUSAN-49 | `payment_receipt` mencatat semua dana masuk dengan hasil `captured`/`duplicate`/`amount_mismatch`/`late_unavailable`; `payment_capture` unik per pesanan | Hanya tangkapan berhasil | RF-12, RF-13, RF-14 punya baris sumber yang dirujuk refund |
| KEPUTUSAN-50 | Satu klaim per pesanan (`UQ patient_order_id`) | "Satu klaim aktif" dengan partial unique | Setara dengan UP-36 + AB-36 tetapi lebih sederhana dan lebih ketat |
| KEPUTUSAN-51 | Kewajiban rekanan dibuat saat refund dibuat, atau untuk RF-18 saat penyebab ditetapkan; "Menunggu penetapan penyebab" ditampilkan dari refund RF-18 tanpa penetapan | Kewajiban dengan nominal kosong | Nominal kewajiban bergantung penyebab (AB-34) |
| KEPUTUSAN-52 | Isi klinis (catatan konsultasi, rekam medis RME, hasil lab, item resep) disimpan sebagai satu dokumen JSON terenkripsi per versi dengan `payload_schema_version` dan field berkode `{system, code, display}` | Tabel anak per kelompok data | Semua isi klinis wajib terenkripsi (ADR-013), sehingga kolom terpisah tidak menambah kemampuan query; struktur tetap dapat dipetakan ke FHIR (ADR-020) |
| KEPUTUSAN-53 | Pesan tiket punya penanda `visible_to_partner` yang ditetapkan Layanan Pelanggan | Rekanan melihat semua pesan | US-1104 AC2: rekanan melihat tiket tanpa data klinis |
| KEPUTUSAN-54 | Status tambahan: tiket dibuka kembali = `closed → in_progress`; permintaan hak subjek data punya `in_progress` | Status baru "Dibuka kembali" | PRD tidak menyebut status tujuan pembukaan kembali dan hanya menyebut "tahap penanganan"; perlu konfirmasi user |
| KEPUTUSAN-55 | Role database RME tambahan `rme_auth` (pemetaan subjek Keycloak lintas tenant) dan `rme_worker` (antrean lintas tenant); operator RME = `rme_user` tanpa `fasyankes_id` | Tanpa RLS di tabel pengguna dan antrean | Menjaga RLS di semua tabel tenant (ADR-012) sambil memungkinkan pemetaan login dan job lintas tenant |
| KEPUTUSAN-56 | `audit_event` dipartisi per `recorded_at`, PK `(event_id, recorded_at)`, keunikan global lewat `audit_event_key`; role `audit_verifier` | Partisi per `occurred_at`; tanpa tabel kunci | Rentang `seq` per partisi berurutan untuk arsip; relay tahan pengiriman ulang; job verifikasi perlu baca + tulis hasil |
| KEPUTUSAN-57 | Nomor yang dibaca manusia sebagai `text` unik; format ditetapkan Tahap 5 | Format ditetapkan sekarang | Bergantung pustaka dan kebutuhan cetak |
| KEPUTUSAN-58 | Kalender hari kerja berisi baris eksplisit per tanggal | Aturan Senin–Jumat + daftar libur | Hari kerja mingguan adalah fakta bisnis yang tidak dinyatakan PRD |
| KEPUTUSAN-59 | Persen disimpan sebagai basis poin `integer` | `numeric` | Perhitungan bilangan bulat dan pembulatan eksplisit (AB-11a) |
| KEPUTUSAN-60 | Area home visit dan alamat memakai kode wilayah per level (provinsi, kota, kecamatan); sistem kode ditetapkan di spesifikasi kontrak | Poligon atau radius | Sederhana, cukup untuk UP-22; sistem kode `[TIDAK DIKETAHUI]` |
| KEPUTUSAN-61 | ID pseudonim penerima per rekanan (`partner_recipient_alias`) dikirim ke rekanan | Mengirim ID internal `care_recipient` | Riwayat pasien di RME (US-1310) tanpa membuka ID internal; minimisasi (NFR-PRIV-03) |
| KEPUTUSAN-62 | Penahanan slot dan stok rekanan hanya menyimpan ID penahanan dari rekanan; sumber kebenaran ketersediaan tetap di sistem rekanan | Ruang Sehat menghitung kuota rekanan sendiri | ADR-014 §2, AB-60 |
| KEPUTUSAN-63 | Rilis 1 hanya mempartisi tabel audit | Partisi tabel aplikasi besar sejak awal | Volume NFR-CAP belum menuntut; dievaluasi setelah uji beban |
| KEPUTUSAN-64 | `terminology_code` sebagai acuan global tanpa RLS | Salinan per tenant | Kode terminologi bukan data tenant |
| KEPUTUSAN-65 | Status pengiriman SATUSEHAT memakai riwayat + proyeksi, sementara catatan pengiriman tetap kelas Log | Kolom status di tabel Log | ADR-020 menyebut catatan pengiriman kelas Log "berstatus"; Log tidak boleh di-update (ADR-008) |
| KEPUTUSAN-66 | Tabel kunci data: role aplikasi hanya S, I; `privacy_worker` U `wrapped_dek`/`shredded_at`/`shred_reason` (RME dengan kebijakan RLS khusus); trigger satu arah setelah `shredded_at` terisi; rotasi KEK utamanya lewat versi kunci KMS yang tetap membuka DEK lama (diverifikasi Tahap 5), cadangannya role `key_rotation` dengan U `wrapped_dek`, `kek_ref` hanya untuk baris belum dimusnahkan | Role aplikasi boleh mengubah DEK; bungkus ulang massal oleh role aplikasi | Review DBA B01: DEK tidak boleh diubah aplikasi atau dihidupkan setelah dimusnahkan; rotasi tanpa menyentuh baris lebih aman |
| KEPUTUSAN-67 | Partial unique refund `WHERE refund_case NOT IN ('rf_12','rf_13','rf_14')`; `UQ (payment_receipt_id)` tetap. Tafsiran AB-36: RF-13 dan RF-14 adalah refund atas dana yang tidak menjadi pembayaran pesanan; menunggu konfirmasi user di gate Tahap 4 | Pengecualian hanya RF-12 (memblokir refund wajib sesudah RF-13/RF-14) | Review DBA B05 |
| KEPUTUSAN-68 | Wadah konsultasi dipecah menjadi `consultation_chat` (keluhan awal, chat, lampiran) dan `consultation_clinical` (catatan, resep, PDF resep); kesesuaian DEK dengan wadah dan tenant RME ditegakkan FK komposit | Satu wadah per sesi; tanpa FK komposit | Retensi catatan sebagai rekam medis mungkin berbeda dari chat (L-05); mencegah DEK atau baris tertaut ke wadah atau fasyankes lain (review DBA S01, S02) |
| KEPUTUSAN-69 | Sub-kelas Master snapshot (`order_personal_snapshot`, `rme_visit_personal_snapshot`): aplikasi hanya S, I; UPDATE hanya `privacy_worker` pada kolom "Anonimkan" | Master biasa yang dapat di-update aplikasi | Snapshot adalah salinan saat transaksi, sehingga hanya boleh berubah karena anonimisasi (review DBA S09) |

### E.2 Temuan untuk diputuskan orkestrator

| ID | Temuan | Usulan | Keputusan orkestrator (2026-11-18) |
| --- | --- | --- | --- |
| F-01 | ADR-002 mewajibkan PK UUIDv7; tabel audit terpartisi memakai PK komposit `(event_id, recorded_at)` | Nyatakan di ADR-009 bahwa PK komposit dengan UUIDv7 memenuhi ADR-002 untuk tabel terpartisi | Diterima. ADR-009 butir 4 menyatakan PK komposit dengan `event_id` UUIDv7 memenuhi ADR-002 |
| F-02 | ADR-009 butir 4: `audit_writer` hanya `INSERT`, padahal relay perlu membaca kepala rantai (`seq`, `event_hash` terakhir) setelah restart | Tambahkan akses baca sempit ke kepala rantai (view atau fungsi) di ADR-009; bentuknya Tahap 5 | Diterima. ADR-009 butir 4: `audit_writer` mendapat baca sempit atas kepala rantai lewat fungsi atau view; ditambah role `audit_verifier` |
| F-03 | Log dan operasional yang memuat data pribadi (misalnya `consent_record` yang merujuk pasien, alamat IP di audit) tidak punya jalur retensi: ADR-008 memasang trigger penolak `DELETE` di kelas Log, dan ADR-013 hanya mengatur transaksi (crypto-shredding) dan master (anonimisasi) | Tetapkan di ADR-013 cara pemusnahan baris Log saat retensi habis (misalnya hak `DELETE` khusus `privacy_worker` dengan pengecualian trigger), setelah PT-24 | Diterima. ADR-013 §5 dan ADR-008: baris Log dihapus oleh `privacy_worker` setelah retensi habis; trigger penolak mengizinkan hanya role itu; tidak aktif sebelum PT-24 |
| F-04 | `[KEPUTUSAN-43]` memperluas daftar enkripsi dan wadah ADR-013 §3 | Revisi ADR-013 (status masih Diusulkan) | Diterima. ADR-013 §3 direvisi sesuai KEPUTUSAN-43, ditambah wadah `status_note` (F-10) |
| F-05 | RME memerlukan role `rme_auth` dan `rme_worker` di luar daftar ADR-012 butir 8 | Revisi ADR-012 | Diterima. ADR-012 butir 8 memuat `rme_auth` dan `rme_worker` |
| F-06 | Admin Fasyankes membaca log akses dari database audit RME lewat `audit_reader` yang dapat membaca semua fasyankes; isolasi hanya di aplikasi RME | Pertimbangkan RLS per `tenant_id` di database audit RME, atau view per tenant; bawa ke threat model | Diterima. ADR-012 butir 9: RLS per `tenant_id` untuk role pembaca Admin Fasyankes di database audit RME, atau fungsi baca per tenant bila RLS pada tabel terpartisi tidak memadai (verifikasi Tahap 5); dibawa ke threat model |
| F-07 | ADR-018 tidak mencantumkan bucket untuk foto produk dari katalog rekanan | Tambah bucket publik `public_catalog` (tanpa data pribadi) di ADR-018 | Diterima. ADR-018 butir 1 memuat foto katalog rekanan di bucket publik tanpa data pribadi |
| F-08 | Crypto-shredding baru tuntas setelah backup yang memuat DEK terbungkus habis masa simpannya (masa simpan backup 35 hari sebagai nilai sementara sampai PT-24 `[USER] T4-G2`, ADR-019) | Cantumkan di ADR-013 dan pertanyaan legal L-06 | Diterima. Dicantumkan di konsekuensi ADR-013; ditanyakan ke legal bersama L-06 |
| F-09 | Profesi tenaga kesehatan dan jenis pesanan (misalnya konsultasi psikolog) tidak dienkripsi tetapi dapat mengungkap data kesehatan | Bawa ke threat model dan DPIA (BR-110); perlindungan saat ini lewat otorisasi dan audit | Diterima sebagai risiko sisa; dicantumkan di ADR-013 dan dibawa ke threat model serta DPIA (BR-110) |
| F-10 | Kolom `note` di riwayat status adalah teks bebas di tabel transaksi; bila petugas menulis data pribadi, kolom ini tidak dapat dianonimkan | Batasi lewat UI (kode alasan + teks pendek) dan pedoman; terima sebagai risiko sisa atau enkripsi per pesanan | Diputuskan: kolom `note` diganti `note_enc` + `note_data_key_id` dengan wadah entitas induk atau wadah `status_note` (§0, §0.7) |
| F-11 | Struktur penerima layanan (`care_recipient`) dan klaim (satu per pesanan) menyimpang dari bentuk PRD §8; kardinalitas lain dicatat di `docs/04-erd.md` §0 | Konfirmasi di gate Tahap 4 | Dinilai konsisten dengan aturan PRD (UP-36 dan AB-36 menyiratkan satu klaim per pesanan; pesanan tanpa pembayaran adalah pesanan Menunggu pembayaran/Kedaluwarsa). Ditampilkan di gate Tahap 4 |
| F-12 | `docs/04-arsitektur.md` §3 (daftar modul) belum ada saat dokumen ini disusun | Selaraskan nama domain dengan modul setelah arsitektur ditulis | Diselaraskan: `docs/04-arsitektur.md` §3 memuat pemetaan modul ke bagian data dictionary |
| F-13 | Database `keycloak` adalah penyimpanan data pribadi tambahan (email, nama, alamat IP sesi) yang belum tercakup ADR-019 (tiga instans) dan retensi PT-24 | Cakupkan di revisi ADR-011 dan ADR-019 | Diterima. ADR-019 dan ADR-013 mencakup database `keycloak` (PITR, retensi PT-24) |

### E.3 Butir `[TIDAK DIKETAHUI]` yang memengaruhi model data

| Butir | Dampak | Rujukan |
| --- | --- | --- |
| Masa retensi per kategori data, termasuk audit | Isi `retention_policy`; job retensi tidak berjalan | PT-24, L-05, L-06 |
| Perlakuan pajak | Kolom atau akun pajak belum dibuat | PT-28 |
| Penerima dana ongkos kirim | `fund_split_instruction_line.recipient_type = 'shipping_recipient'` dan akun `shipping_funds` belum final | PT-31, AB-23 |
| Penanggung kegagalan kurir | Kewajiban kelompok `courier_cause` | PT-29 |
| Nilai margin dan biaya rekanan, cara penagihan | Isi `margin_rule`, `partner_fee_rule`; status tagihan | PT-17, AB-24 |
| Metode pembayaran dan field penyedia | `payment_method_fee.method_code`, `gateway_recipient_ref`, isi `payload_sanitized` | AB-25, US-205 AC5, Tahap 5 |
| Data minimal ke rekanan dan payment gateway | Isi payload `integration_outbox`, kolom `order_personal_snapshot` yang dikirim | L-08, AB-75 |
| NIK pasien di RME | `rme_patient.nik_*` tetap kosong | L-05, L-08 |
| Kelompok data dan terminologi final rekam medis; KFA; kode tindakan dan tanda vital | Struktur `payload` RME dan catatan konsultasi | L-05, US-1306 AC2, ADR-020 |
| Status catatan konsultasi sebagai rekam medis | Retensi dan kemungkinan pengiriman ke SATUSEHAT | L-05 |
| Identitas profesi SATUSEHAT untuk tenaga kesehatan fasyankes | Kolom tambahan `rme_health_worker_profile` | US-1302 AC2 |
| Makna kode registrasi fasyankes dan cara memperoleh kredensial SATUSEHAT | `fasyankes.facility_registration_code`, `satusehat_credential` | PT-32 |
| Batas usia anak | Penentuan KTP/KK dan penanda `Spesifik*` | L-08, US-108 AC6 |
| Daftar hubungan keluarga | Nilai `care_recipient.relationship` | US-108 AC1 |
| Sistem kode wilayah | `*_code` alamat dan area home visit | `[KEPUTUSAN-60]`, US-1401 |
| Kebutuhan data kurir (misalnya berat) | `product.weight_grams` | Tahap 5 |
| Dokumen izin wajib per jenis | Isi `document_requirement` | L-02, L-03, L-07 |
| Klaim sebagian barang dan pengembalian barang | Nominal refund dan alur pengembalian | PT-33, PT-30 |
| Validasi hasil lab oleh penanggung jawab | Status tambahan hasil lab RME | L-05, US-1307 AC3 |
| *Merchant of record* di bukti bayar | Data bukti pembayaran | L-12, US-804 AC2 |

