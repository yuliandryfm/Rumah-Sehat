# Review DBA Tahap 5: `docs/05-physical-schema.md` (T1, putaran 1)

Tanggal: 2026-11-18
Reviewer: subagent `dba-reviewer` (independen, hanya baca). Laporan disimpan apa adanya oleh orkestrator.
Berkas yang direview: `docs/05-physical-schema.md` v0.1.
Acuan: `docs/04-data-dictionary.md` (§0, A2, A3.1, A6.6, A12.5, A19, B, C6, D, E), ADR-003, -004, -008, -009, -012, -013, -021, -025, `docs/review/tahap-04-dba.md`, `docs/coding-standards.md` (§1.4, §2.1, §5.2, §5.3, §5.5), SRC-31–SRC-42, SRC-224, SRC-234, KEPUTUSAN-74, -79, -89.

## Status: TIDAK LULUS (3 blokir)

Sebagian besar rancangan sudah benar dan matang. Yang diperiksa dan dinilai benar: pola nama dan batas 60 byte; PK `uuid` tanpa nilai bawaan; partial unique untuk Master; FK tanpa `CASCADE`; database dimiliki superuser bootstrap; `public` dihapus; `CONNECT`, `TEMP`, dan `EXECUTE` dicabut dari `PUBLIC`; atribut role `NOBYPASSRLS`; pemilik tabel bukan role aplikasi; pola RLS `set_config(..., true)` + `NULLIF` + `FORCE` + kebijakan eksplisit untuk pemilik (gagal-tertutup benar); partisi audit dibuat migrasi dengan tanggal literal tanpa partisi `DEFAULT`; trigger statement di setiap partisi (tetap ada setelah detach); CHECK amplop ciphertext dan blind index; pembangkit nomor (alfabet 32 karakter, regex, estimasi bentrokan dihitung ulang dan cocok); `fn_chain_head` `SECURITY DEFINER` dengan `search_path` tetap.

Tidak ada fakta PostgreSQL yang salah secara nyata. Butir bertanda (V8) boleh tetap (V8).

## Blokir

| ID | File § | Masalah | Usulan perbaikan |
| --- | --- | --- | --- |
| DBA5-B01 | §3.2 tabel (b) baris `audit_verifier`; §3.3 baris `rs-backend audit-relay` dan relay RME; §14 F-P09; §15 KEPUTUSAN-141 | Kredensial `*_audit_verifier_login` ditaruh di deployment relay. ADR-009 butir 3 menyatakan relay "hanya memegang kredensial `audit_relay` dan `audit_writer`". Role verifier punya `SELECT` atas seluruh audit (termasuk PII) ditambah `INSERT` hasil verifikasi, sehingga relay yang dibobol dapat menulis rantai palsu sekaligus mencatat `match`. Deteksi manipulasi ADR-009 butir 5 menjadi pemeriksaan oleh pihak yang diperiksa sendiri. Hak berlebih dan pelanggaran ADR | Pisahkan verifikasi rantai dan ekspor arsip ke deployment `audit-verify` per sistem dengan identitas OpenBao sendiri yang hanya memegang `*_audit_verifier_login`. Relay hanya `*_audit_relay_login` dan `*_audit_writer_login`. Revisi §3.2, §3.3, §10.3, §6.5 butir 3, KEPUTUSAN-141 |
| DBA5-B02 | §11.4 butir 2; §15 KEPUTUSAN-140 | Jalur `ALTER TABLE ... DISABLE TRIGGER` lalu `UPDATE`/`DELETE` baris T/R/L untuk "perbaikan data akibat bug yang disetujui" bertentangan dengan ADR-004 (koreksi lewat reversal entry; "Update langsung dengan audit log" ditolak) dan ADR-008 (Log hanya dihapus pekerja privasi setelah retensi). Pola baru wajib lewat ADR (CLAUDE.md) | Hapus pengecualian. Koreksi T/R hanya lewat baris pembalik atau versi baru; tabel L tanpa pengecualian selain penghapusan retensi. Koreksi fisik kelak harus lewat CR atas ADR-004/ADR-008. Tambahkan pemeriksaan teks di skrip verifikasi yang menolak berkas migrasi berisi `DISABLE TRIGGER` atau `DROP TRIGGER tg_*_reject_*` |
| DBA5-B03 | §4.2 (3), `fn_effective_row_lock` (baris `consent_text_version`) | DD A3.1: setelah `published_at` terisi, kolom isi tidak dapat diubah. `fn_effective_row_lock` hanya mengunci berdasarkan `effective_from <= now()`, sehingga teks persetujuan yang sudah terbit dan diberitahukan (US-104 AC4) masih dapat diubah sampai tanggal berlaku, dan `published_at` dapat dikosongkan. Integritas bukti persetujuan hilang | Tambahkan trigger `tg_consent_text_version_published_lock` (BEFORE UPDATE FOR EACH ROW): bila `OLD.published_at IS NOT NULL`, tolak perubahan `consent_type`, `version_label`, `content`, `effective_from`, `published_at`; `change_notice_sent_at` hanya boleh diisi sekali. Aturan `effective_from` (§0.8) tetap berlaku. Tambahkan ke T-09. Usulan: `consent_item` milik versi terbit dikunci serupa |

## Non-blokir (10 terpenting)

- **N01. `search_path` role migrasi dinamis** (§3.4, §3.5 butir 4, §10.2): `ALTER ROLE <login> IN DATABASE ... SET` tidak dapat dipasang untuk role dinamis, dan pengaturan per role `rs_migration` tidak terpakai saat `SET ROLE`. Dengan `public` dihapus, sqlx gagal membuat `_sqlx_migrations`. Usulan: `ALTER ROLE "{{name}}" SET search_path = app` (audit: `audit`) di `creation_statements` atau `ALTER DATABASE ... SET search_path`; pastikan `_sqlx_migrations` di skema `app`/`audit` (T-02, T-04); skrip verifikasi meniru pola role dinamis; nyatakan `db/roles/` dijalankan superuser testcontainer sebelum migrasi.
- **N02. `revocation_statements` dan admin OpenBao** (§3.4): `REASSIGN OWNED`/`DROP OWNED` kemungkinan gagal dengan hak ADMIN saja (V8). Usulan: cukup `DROP ROLE IF EXISTS "{{name}}"`; satu koneksi OpenBao per database audit di instans (b); batasi `pg_hba` `openbao_*_admin` hanya dari host OpenBao; T-05 memastikan `CREATEROLE` hanya pada `openbao_*_admin`.
- **N03. Celah kebijakan RLS** (§5.3, §5.6): (a) `rme_key_rotation` belum punya kebijakan atas `rme_data_key`; (b) kebijakan `rme_operator` di `rme_data_key` `USING (true)` terlalu luas, persempit ke `container_type IN ('rs_integration_credential')`; (c) `util.fn_current_fasyankes_id()` harus dibuat di `rme_audit` dengan `USAGE`/`EXECUTE` untuk `rme_audit_tenant_reader`; (d) pembersihan `rme_inbound_event` oleh `rme_worker` butuh `GRANT SELECT (id, expires_at)` dan kebijakan `FOR DELETE` lintas tenant.
- **N04. Pembersihan `rme_slot_hold` dan `rme_upload_slot` tanpa role** (§9.3; DD §D.2 vs §0.6 C): DD mewajibkan pembersihan, tetapi hak `rme_worker` tidak memuatnya. Usulan: temuan F-P baru dan CR DD (hak S kolom + D untuk `rme_worker` dengan kebijakan pembersihan, atau pembersihan oleh `rme_app` per tenant).
- **N05. `DELETE` Log terhalang FK masuk dan `end_action`** (§4.2 (2), §4.9): beberapa tabel L dirujuk tabel non-L (mis. `payment_receipt` → `payment_gateway_notification`; `satusehat_submission` dirujuk R/P), pasangan L→L butuh urutan, dan penjaga tidak membaca `end_action`. Usulan: test katalog tabel L dengan FK masuk dari non-L; kategori ditetapkan bersama PT-24 atau FK dibawa ke CR; penjaga hanya mengizinkan `end_action` yang disepakati; job retensi menangani pelanggaran FK sebagai "dilewati".
- **N06. Mekanika sewa job, outbox, relay** (§9.1, §6.4): (a) penyelesaian job dipagari `status = 'running' AND locked_by = $2`; (b) penyapu menerapkan `attempt_count >= max_attempts` → `failed`; (c) sewa outbox = memajukan `available_at`; (d) relay memakai `INSERT ... ON CONFLICT (event_id) DO NOTHING` dan memeriksa jumlah baris.
- **N07. Pengaturan berlaku-mulai dapat ditanggalmundurkan** (§4.2 (3)): tambahkan BEFORE INSERT `NEW.effective_from >= now()` (toleransi kecil bila perlu) untuk semua tabel §0.8.
- **N08. Password berisiko masuk log server dan `pg_stat_activity`** (§3.4, §10.1): `ALTER ROLE openbao_*_admin SET log_min_error_statement = 'panic'` dan `log_statement = 'none'`; rotasi role statis memakai verifier SCRAM ter-hash di klien; agen metrik tidak mengekspor teks query.
- **N09. Katalog dan uji tabel audit** (§6.1, §6.2, §12, §3.2 (b)): penanda kelas `class:A` untuk `audit_event`; trigger penolak tingkat baris di tabel induk; T-18 akan merah sekitar pertengahan 2029 (dokumentasikan atau pisahkan dari pemantauan production); perluas T-21 untuk `CONNECT` lintas database audit; jangan buat `rme_audit_reader_login` tanpa proses pemegang.
- **N10. Kapasitas dan konsistensi dokumen**: estimasi volume audit dan ambang peringatan disk `db-audit` 60 GB; `coding-standards.md` §5.3 ("hak role di `db/roles/`") dan §1.4 (`rme-backend api | worker`) perlu diselaraskan; teks KEPUTUSAN-89 dikoreksi bersama F-P08; §0 menyebut "nomor 90–113" padahal yang dipakai 90–109 dan 139–142.

## Rekomendasi untuk F-P04, F-P07, F-P08, F-P09, F-P10

- **F-P04** (`data_key.retention_anchor_at`/`retention_category` boleh NULL tanpa role pengisi): opsi memberi `rs_app` `UPDATE` jangan dipilih (bertentangan DD §0.6 A/A19.7, ADR-013 §5, KEPUTUSAN-66). Utama: tabel sekali-isi lewat CR DD, `data_key_retention_anchor` kelas T (`data_key_id uuid PK FK→data_key`, `retention_category text NOT NULL`, `retention_anchor_at timestamptz NOT NULL`, `created_at`, `created_by`; `rs_app` S, I; `privacy_worker` S; trigger penolak T; index `(retention_category, retention_anchor_at)`; BEFORE INSERT menolak bila kolom acuan di `data_key` sudah terisi; padanan RME dengan `fasyankes_id`, FK komposit, RLS). Alternatif tanpa CR: acuan diturunkan saat job berjalan. Untuk keduanya: acuan `NULL` = tidak memenuhi syarat retensi (gagal-tertutup), metrik "DEK tanpa acuan > N hari", trigger §4.10 tetap menolak perubahan kedua kolom.
- **F-P07** (detach arsip di luar migrasi): detach bukan berkas sqlx, tetapi langkah manusia dipersempit menjadi satu pemanggilan prosedur yang kodenya ikut migrasi, `audit.pr_archive_detach(...)` (milik pemilik skema; `EXECUTE` dicabut dari semua role; memeriksa umur partisi, verifikasi `match` terakhir, jumlah baris dan rentang `seq`, `boundary_event_hash`; `DETACH` non-concurrent dengan `lock_timeout`; manifest; `SET SCHEMA audit_detached`). Prosedur `DROP` tabel lepas serupa. Ekspor memakai role dinamis OpenBao terpisah (hanya baca). Pemanggilan lewat bastion dengan kredensial pemilik dinamis dan orang kedua. Tafsiran "langkah migrasi" ADR-021 butir 4 dicatat di ADR-025 dan dikonfirmasi di gate. Opsional: event trigger yang menolak `DROP TRIGGER`/`DISABLE TRIGGER` di skema `audit`.
- **F-P08** (`db/roles/`): bootstrap klaster dijalankan superuser (production lewat akses darurat PT-38; skrip verifikasi lewat superuser testcontainer), idempoten, berisi role dan atribut, keanggotaan dengan opsi eksplisit (V8), database dan skema, ekstensi, pencabutan hak `PUBLIC`, default privileges, pengaturan per role, role `openbao_*_admin`; tanpa password; hak per tabel tetap di migrasi. Koreksi KEPUTUSAN-89 dan `coding-standards.md` §5.3. T-03 dan T-05 juga dijalankan berkala terhadap staging/production.
- **F-P09** (relay dan verifikasi audit RME): tambahkan proses relay RME sebagai deployment terpisah (subcommand `audit-relay` di `rme-backend` atau binary relay bersama) yang hanya memegang `rme_audit_relay_login` + `rme_audit_writer_login`. Untuk kedua sistem, verifikasi rantai dan ekspor arsip di deployment `audit-verify` tersendiri, mencocokkan rantai dengan kepala rantai di bucket WORM yang dibaca langsung, dan mengirim peringatan ke security log secara independen. Perbarui KEPUTUSAN-141, KEPUTUSAN-74, `coding-standards.md` §1.4, tabel host ADR-025.
- **F-P10** (pemegang `*_privacy_worker`): subcommand `privacy-worker` di `rs-backend` dan `rme-backend` sebagai deployment tersendiri dengan identitas OpenBao sendiri, hanya memegang `*_privacy_worker_login` dan izin hapus object storage; tanpa Transit `decrypt`/`datakey`; satu instans aktif lewat kunci advisory baru (7001004, 7002004); penjadwalan internal; untuk US-1605 kelak perlu kanal masukan lewat CR (mis. tabel O `privacy_task`).

## Penilaian temuan lain di §14

- F-P01: benar; pemetaan nama fisik berawalan diterima, ditambah uji `CONNECT` lintas database (N09).
- F-P02: benar; dengan RLS `FORCE`, kebijakan untuk semua role diperlukan.
- F-P03: subquery diterima; CR penambahan `tenant_id` ke B3 hanya bila T-17 bermasalah.
- F-P05, F-P11, F-P12: setuju.
- F-P06: CHECK format amplop/blind index dan FK proyeksi ke riwayat (KEPUTUSAN-142) diterima.

---

## Putaran 2 (cek ulang terbatas, 2026-11-18): LULUS DENGAN CATATAN

Ketiga blokir tertutup; baris yang berubah tidak memuat pelanggaran ADR/DD, hak berlebih, atau kehilangan integritas.

| ID | Status | Dasar |
| --- | --- | --- |
| DBA5-B01 | Tertutup | §3.3: `audit-relay` hanya `*_audit_relay_login` + `*_audit_writer_login`; `audit-verify` deployment tersendiri dengan identitas OpenBao sendiri, hanya `*_audit_verifier_login`, membaca kepala rantai dari bucket WORM, peringatan sendiri; T-05 melarang satu role login anggota grup writer dan verifier sekaligus; ekspor arsip oleh `audit-verify` (§6.5). Sinkron dengan `coding-standards.md` §1.4, ADR-025, KEPUTUSAN-141 |
| DBA5-B02 | Tertutup | §11.4 "Tanpa pengecualian"; koreksi lewat baris pembalik/versi baru; T-23 dan T-08 menegakkan larangan |
| DBA5-B03 | Tertutup | §4.2 (3c) `tg_consent_text_version_published_lock` sesuai DD A3.1 |

Butir putaran 1 lain diterapkan dengan benar (F-P04/CR-014, N04/KEPUTUSAN-156, F-P07/KEPUTUSAN-157, F-P08, F-P09/F-P10, N01–N10). F-P13 dan F-P14 dirumuskan tepat sebagai butir gate.

Catatan non-blokir:
1. §4.12 `fn_data_key_retention_anchor_single_source` terlalu luas: tolak hanya bila `dk.retention_anchor_at IS NOT NULL`; bila `dk.retention_category IS NOT NULL`, wajibkan kategori sama. Selaraskan kalimat CR-014. Putuskan hak `rme_worker` atas tabel acuan RME sebelum CR-014 disetujui.
2. §6.5 `pr_archive_detach`: periksa `p_partition_month` adalah awal bulan; tambahkan parameter jumlah baris partisi `audit_event_subject`.
3. KEPUTUSAN-154: pasang kunci `consent_item` versi terbit (INSERT/UPDATE/soft delete ditolak bila induk sudah terbit), bukan hanya usulan.
4. Fungsi `util` menyebut nama role langsung; teruskan lewat argumen trigger atau tambahkan pemeriksaan katalog bahwa fungsi di database `rme` hanya merujuk role `rme_*`.
5. F-P13 (volume audit vs disk `db-audit` 60 GB) perlu pemilik dan tenggat paling lambat setelah uji beban NFR-CAP-04.
