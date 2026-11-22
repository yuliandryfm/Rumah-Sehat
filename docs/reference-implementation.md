# Reference implementation: verifikasi identitas pasien (US-106 → US-201)

Dokumen pola Tahap 8 Fase 1 (F1-T66, KEPUTUSAN-181). Task berikutnya meniru pola di sini. Setiap rujukan di bawah menunjuk berkas atau fungsi yang ada di repositori saat dokumen ini ditulis (dicek dengan `grep`). Kalau kode berubah, dokumen ini diperbarui di task yang sama. Bila dokumen dan kode berbeda, kode yang menang untuk detail API, dokumen `docs/` (ADR, `coding-standards.md`) yang menang untuk aturan.

Label: `[DOK]` dokumen proyek, `[KODE]` kode di repo.

## 1. Lokasi

| Bagian | Lokasi |
| --- | --- |
| Modul bisnis | `apps/rs-backend/src/modules/profile/` |
| Jalur tulis status (pustaka bersama) | crate `crates/status-flow` (`src/lib.rs`: `transition`, `StatusStore`, `TransitionRules`, `MAX_ATTEMPTS`) |
| Pustaka bersama lain yang dipakai | `crates/audit` (`in_audited_tx`), `crates/authz` (`Policy`, `PolicyRouter`, `EnforcedRouter`, `ClientIp`, `OwnedCareRecipient`, `care_recipient_access`), `crates/crypto` (`Nik`, blind index), `crates/envelope` (`ResponseCode`), `crates/foundation` (`Sensitive`, `Clock`), `crates/valkey`, `crates/observability` (`security::rate_limited`) |
| Kontrak OpenAPI | `openapi/pasien.json` (`/patient/v1`), `openapi/backoffice.json` (`/backoffice/v1`) |
| Test integrasi | `apps/rs-backend/tests/profile_identity_submission.rs`, `profile_identity_http.rs`, `profile_identity_verifier.rs`, `authz_care_recipient.rs`, `openapi_response_codes.rs` |
| Mobile (A03) dan web (C02) | `[TIDAK DIKETAHUI]` di dokumen ini: F1-T63 dan F1-T64 belum termasuk. Isi bagian ini saat keduanya masuk |

Berkas modul `profile` dan perannya:

| Berkas | Peran |
| --- | --- |
| `mod.rs` | Hanya `pub mod api`; `patient_router()` dan `backoffice_router()` merakit rute (coding-standards §1.3) |
| `http.rs`, `verifier_http.rs` | Handler: urai DTO, panggil satu fungsi `api`, petakan hasil ke envelope |
| `dto.rs`, `verifier_dto.rs` | DTO request/response (skema OpenAPI lewat `utoipa`) |
| `api.rs` | Satu-satunya antarmuka publik modul: `submit_identity_document`, `cancel_identity_submission`, `identity_status`, `create_ktp_upload_slot`, `verification_queue`, `approve_identity`, `reject_identity`, `care_recipient_owner`, wadah `IdentityServices` |
| `service.rs`, `verification.rs` | Use case sisi pasien (`submit`, `cancel`, `status`) dan sisi Verifikator (`queue`, `detail`, `document_photo_url`, `approve`, `reject`) |
| `self_decision.rs` | Pemisahan tugas Verifikator (`SelfDecisionGuard::may_decide`, KEPUTUSAN-294) |
| `domain.rs` | Aturan murni tanpa I/O: `VerificationRules` (implementasi `TransitionRules`), `owner_nik_conflict`, `verifier_may_decide` |
| `repo.rs` | Satu-satunya tempat SQL modul; `IvrStatusStore` (implementasi `StatusStore`) |
| `policy.rs` | `patient()` dan `verifier()` mengembalikan `authz::Policy` |
| `error.rs` | `ProfileError` dan konstanta `ResponseCode` modul |

## 2. Lapisan: handler → service → repo

1. **Handler** (`http.rs`, `verifier_http.rs`): tanpa logika bisnis. Mengurai path/body, mengubah ID path yang bukan UUID menjadi `NotFound`, mengubah galat JSON tanpa meneruskan teks serde (dapat memuat NIK; lihat `body_error` di `http.rs`), memanggil fungsi `api`, memetakan `ProfileError` lewat `ApiFailure` (`apps/rs-backend/src/http_support.rs`).
2. **Service**: urutan tetap yang tertulis di komentar modul `service.rs`: validasi input dan IP klien dulu; pemeriksaan batas laju; panggilan ke pihak luar (object storage, OpenBao Transit, Valkey) **sebelum** transaksi dibuka; lalu satu transaksi `audit::in_audited_tx` untuk semua tulisan basis data.
3. **Repo** (`repo.rs`): hanya SQL; tabel Transaksi dan Riwayat hanya `INSERT` (ADR-004, ADR-008). Tidak memanggil authz atau audit.
4. **Domain** (`domain.rs`): fungsi murni yang dapat diuji tanpa DB.
5. Modul lain hanya bisa memanggil `profile::api` (visibilitas `mod.rs`, dipaksa compiler dan `crates/arch-check`).

## 3. Jalur tulis status (ADR-008 butir 2-3)

- Seluruh perubahan status memanggil `status_flow::transition` di dalam transaksi `in_audited_tx` (contoh: `verification.rs`, fungsi `approve`, memakai `IvrStatusStore` dari `repo.rs`).
- Modul menyediakan dua hal saja: `impl TransitionRules<VerificationStatus> for VerificationRules` (`domain.rs`) dan `impl StatusStore<VerificationStatus> for IvrStatusStore` (`repo.rs`). Pustaka tidak memuat SQL.
- Langkah (a)-(e) dan perilaku kalah bersaing (`Appended::Lost` → evaluasi ulang, `MAX_ATTEMPTS = 3`, `TransitionError::{NotAllowed, Contended, ProjectionMismatch}`) didokumentasikan di komentar modul `crates/status-flow/src/lib.rs`. `INSERT` event wajib `ON CONFLICT ON CONSTRAINT ... DO NOTHING`.
- Pemetaan galat di `verification.rs` (`T::ProjectionMismatch => ProfileError::ProjectionMismatch`): proyeksi menyimpang membatalkan transaksi dan mencatat kejadian keamanan; `NotAllowed` menjadi `4090204` (keputusan pertama yang berlaku, `ALREADY_DECIDED` di `error.rs`).
- Dilarang: `UPDATE` langsung kolom status di luar `IvrStatusStore`.

## 4. Audit (ADR-009)

- Semua aksi sensitif memakai `audit::in_audited_tx(pool, clock, |conn| ...)`: transaksi tingkat atas milik pustaka; event masuk `audit_outbox` di transaksi yang sama sehingga gagal tulis outbox membatalkan aksi (`AuditedTxError::{Work, Audit, Database}`).
- Pembangun event dan konteks di `service.rs`: `request_event`, `status_change`, `audit_context`, `actor`, `source_component`. Field "Sumber" memerlukan IP klien; tanpa IP aksi ditolak gagal-tertutup (`client_ip` di `service.rs`; test `missing_client_ip_fails_closed_without_side_effects`).
- Aksi gagal dicatat di transaksi tersendiri lewat `record_failed` (`service.rs`).
- Field wajib per kode event ditegakkan pustaka (`audit::check_payload` di `AuditEvent::finish`, F1-T69). Pemancar event baru wajib didaftarkan di `audit-event-coverage.toml` dengan test yang memanggil `assert_event_complete` (`apps/rs-backend/tests/audit_support`) atau `audit_events` (`identity_support`); aturan arch-check `audit-coverage` menolak yang tidak terdaftar.
- Akses data sensitif (E-05) dicatat sebelum data dibuka: foto dokumen (`document_photo_url`) dan pembukaan NIK (`open_document_nik` di `verification.rs`). Field sensitif memakai aturan `audit::FieldRule` (NIK hanya 4 digit terakhir, `FieldRule::last4("nik")` di `service.rs`; tipe di `crates/audit/src/policy.rs`).

## 5. Otorisasi (ADR-012, coding-standards §5.4)

- **Default deny**: rute hanya terdaftar lewat `PolicyRouter::routes(policy, routes!(handler))` (`http.rs`, `router()`); rute tanpa kebijakan tidak dapat dirakit. Router akhir dibungkus `EnforcedRouter` (`crates/authz/src/router.rs`) yang memasang middleware token, status akun, dan sesi sebelum handler; dilayani lewat `serve_with_peer_addr` (dipanggil di `apps/rs-backend/src/cli.rs`; perakitan router di `app.rs`).
- **Kebijakan per endpoint** di `policy.rs`: `Policy::patient()` untuk pasien, `Policy::rs_admin(&[AdminSubRole::Verifier])` untuk Verifikator (Super Admin tidak otomatis berhak).
- **Relasi objek** (KU-03): `care_recipient_owner` (`api.rs`) membaca pemilik; `authz::care_recipient_access` memutuskan dan menghasilkan bukti `authz::OwnedCareRecipient` (`crates/authz/src/object.rs`), yang menjadi syarat fungsi yang menyentuh data penerima layanan (`owned_recipient` di `service.rs`). ID tidak ada dan milik orang lain memakai query yang sama dan menghasilkan `NotFound` (404) yang tidak dapat dibedakan.
- **IP klien**: `authz::ClientIp` dipasang dari alamat peer TCP; dibaca handler lewat ekstraktor di `http_support.rs`.
- **Pemisahan tugas** Verifikator memutuskan pengajuannya sendiri: `SelfDecisionGuard::may_decide` (`self_decision.rs`), `4090205`.

## 6. Enkripsi NIK dan blind index (ADR-013)

- `crypto::Nik::parse` memvalidasi 16 digit; `Debug`/`Display` tersamar, nilai hanya lewat `expose()` (`crates/crypto/src/blind_index.rs`).
- NIK dienkripsi dengan DEK wadah `identity_document` (dibuat lewat OpenBao Transit) dengan AAD baris dokumen (`EncryptionContext::IDENTITY_DOCUMENT_NIK` di `service.rs`, `submit`).
- Blind index `HMAC-SHA256` untuk keunikan NIK; pencocokan dengan semua versi kunci aktif selama rotasi (`lookup_candidates`, UP-40/TD-10). Kunci Valkey tidak pernah berisi NIK: memakai SHA-256 dari blind index (`LimitKeys::new` di `service.rs`).
- Panggilan Transit diprobe saat start (`probe_transit` di `apps/rs-backend/src/cli.rs`).

## 7. Batas laju Valkey, gagal-tertutup

- Batas NIK berlapis (per akun, per NIK lintas akun, per IP, peringatan global) di `service.rs`: `LimitKeys`, `is_held`, pencatat penolakan; konstanta ambang di `domain.rs` (`NIK_REJECTIONS_BEFORE_HOLD`, `NIK_HOLD`, `NIK_REJECTION_WINDOW`).
- Gagal-tertutup: Valkey tidak tersedia → `ProfileError::Database` (503), bukan dilewati; penghitung yang sudah mencapai ambang menahan walau kunci penahanan gagal ditulis. Tertahan memakai pesan umum `4090202` sama dengan NIK bentrok (anti-enumerasi) dan memanggil `observability::security::rate_limited`.
- Test: `valkey_unavailable_fails_closed`, `counter_at_threshold_holds_even_without_hold_key`, `three_rejections_hold_the_account_and_hold_skips_validation` di `profile_identity_submission.rs`.

## 8. Kode respons

- Kode modul didefinisikan sebagai `const ... = ResponseCode::new(...)` hanya di `error.rs` modul (lint arch-check membatasi `ResponseCode::new` ke crate `envelope` dan `error.rs`). Contoh: `4090201`-`4090205` di `profile/error.rs`.
- Setiap kode baru wajib terdaftar di `docs/05-response-codes.md`; `apps/rs-backend/tests/openapi_response_codes.rs` memeriksa semua spesifikasi OpenAPI memakai kode terdaftar dengan status HTTP dan deskripsi yang cocok.
- OpenAPI di `openapi/*.json` dihasilkan dari kode (langkah `openapi-sinkron` di `scripts/verify.sh`, test `openapi_sync.rs`, `openapi_schema_drift.rs`); jangan menyunting tangan.

## 9. Debug tersamar (KEPUTUSAN-122, ADR-005)

- Nilai rahasia dan data pribadi dibungkus `foundation::Sensitive<T>` (`crates/foundation/src/sensitive.rs`): `Debug`/`Display` mencetak `[REDACTED]`, tanpa `Serialize`.
- DTO berisi NIK, URL slot unggah, atau token memakai `impl fmt::Debug` manual tanpa nilai itu (`IdentitySubmissionRequest`, `UploadSlotDto` di `dto.rs`; `IdentityServices` di `api.rs`). Jangan `#[derive(Debug)]` pada tipe yang memuat rahasia.
- Galat tidak meneruskan teks pustaka yang dapat memuat isi (lihat `body_error`).

## 10. Lima skenario penolakan endpoint (coding-standards §7.3) dan pola test negatif

Setiap endpoint baru wajib memiliki lima skenario, dijalankan pada seluruh daftar endpoint modul lewat tabel (pola `endpoints(...)` di test):

| # | Skenario | Contoh test |
| --- | --- | --- |
| 1-2 | Tanpa token; token realm lain, audience/issuer salah, algoritma tak diizinkan, kedaluwarsa → 401 | `every_endpoint_rejects_missing_and_invalid_tokens` (`profile_identity_http.rs`) |
| 3 | Akun nonaktif atau sesi dicabut dengan token masih berlaku → ditolak | `inactive_account_revoked_session_and_admin_are_rejected` (`profile_identity_http.rs`) |
| 4 | Peran tidak diizinkan → 403 | `access_is_denied_to_everyone_but_active_verifiers` (`profile_identity_verifier.rs`) |
| 5 | Objek milik pengguna lain lewat ganti ID → 404 dan tercatat | `foreign_care_recipient_is_not_found_and_audited` (`profile_identity_http.rs`), `foreign_or_unknown_care_recipient_is_not_found` (`profile_identity_submission.rs`) |

Pola test negatif tambahan yang dipakai di modul ini:

- Efek samping nol: pada penolakan, tabel dan outbox tidak berubah (`nik_not_sixteen_digits_is_rejected_before_anything_is_stored`, `missing_client_ip_fails_closed_without_side_effects`).
- Gagal-tertutup dependensi: Valkey mati (`valkey_unavailable_fails_closed`).
- Konkurensi: keputusan pertama menang (`first_decision_wins`); pemisahan tugas (`verifier_cannot_decide_own_account`).
- Pesan umum seragam untuk dua sebab berbeda (`owner_nik_in_use_returns_general_message`).
- Test DB memakai DB efemeral (`bash scripts/db-ephemeral.sh cargo test ...`) dan fixture bersama di `apps/rs-backend/tests/common`.

## 11. Checklist pola untuk task berikutnya (salin ke deskripsi PR/commit)

- [ ] Kode modul di `apps/rs-backend/src/modules/<modul>/`; hanya `api` yang `pub`; SQL hanya di `repo.rs`; tabel milik modul sesuai `apps/rs-backend/module-tables.toml`.
- [ ] Handler tipis: DTO → satu fungsi `api` → envelope; ID path bukan UUID → 404; teks galat pustaka tidak diteruskan.
- [ ] Setiap rute lewat `PolicyRouter::routes(policy, ...)` dengan `Policy` eksplisit; tidak ada rute tanpa kebijakan.
- [ ] Akses objek milik pengguna lewat relasi (`care_recipient_access`/bukti `Owned...`), 404 yang sama untuk tidak ada dan bukan milik.
- [ ] Perubahan status lewat `status_flow::transition` (bukan `UPDATE` status langsung); tabel T/R hanya `INSERT`.
- [ ] Tulisan bisnis di `audit::in_audited_tx`; panggilan pihak luar sebelum transaksi; aksi gagal dicatat (`record_failed`); akses data sensitif E-05 sebelum data dibuka; field terlarang tidak masuk event.
- [ ] Data pribadi/rahasia: `Sensitive<T>` atau `Debug` manual; NIK/nomor identitas terenkripsi dengan AAD baris dan blind index bila perlu dicari; kunci Valkey tanpa nilai mentah.
- [ ] Batas laju/anti-enumerasi di Valkey gagal-tertutup; test Valkey mati.
- [ ] `ResponseCode` baru hanya di `error.rs` modul, terdaftar di `docs/05-response-codes.md`; OpenAPI dibangkitkan ulang dan dikomit.
- [ ] Lima skenario penolakan (§10) pada semua endpoint baru, ditambah skenario negatif modul T1 (coding-standards §7.3 tabel).
- [ ] Waktu lewat `Clock` yang dapat diatur; tanpa `unwrap`/`expect` di kode non-test (lint workspace).
- [ ] `bash scripts/verify.sh` hijau, tanpa `--no-verify`; backlog dan `docs/traceability.md` diperbarui di task yang sama.

## 12. Pengukuran kecepatan awal (RT-24)

Metode (bisa diulang): untuk setiap commit di riwayat `--first-parent` branch `tahap-08-fase-1` yang subjeknya menyebut `F1-Txx` (tunggal, daftar, atau rentang `F1-Txx–F1-Tyy`; teks setelah titik koma diabaikan), tandai task itu selesai pada waktu commit tersebut (UTC), sekali per task (kemunculan pertama). Jam-task diambil dari kolom Jam di `docs/07-phase-plan.md` §7.3. Waktu = waktu merge, bukan waktu selesai menulis kode. Skrip: dijalankan sekali dengan Python (tidak disimpan di repo).

Hasil (riwayat sampai commit `c02e43f`, 2026-11-22):

| Tanggal (UTC) | Jam-task merge | Task |
| --- | --- | --- |
| 2026-11-19 | 54 | F1-T02, T04–T18 |
| 2026-11-20 | 125 | F1-T19–T42, T55, T65, T72–T74, T89, T91–T94, T111 |
| 2026-11-21 | 34 | F1-T43, T59–T61, T68, T95–T99 |
| 2026-11-22 (sampai 05:40) | 21 | F1-T62, T85, T87, T88, T90, T100 |

- Total terukur: 67 task, 234 jam-task, dari merge pertama 2026-11-19 11:19 UTC sampai terakhir 2026-11-22 05:40 UTC (±2,76 hari kalender), atau ±84,6 jam-task per hari kalender (rata-rata kasar).
- Batas ketelitian: (1) F1-T01 dan F1-T03 tidak punya commit yang subjeknya memuat ID yang terbaca metode ini, jadi tidak dihitung; (2) F1-T27 dihitung penuh walau subjek merge menulis "sebagian"; (3) jam-task adalah estimasi phase plan, bukan jam kerja nyata; (4) pekerjaan dilakukan paralel oleh beberapa agen, sehingga angka ini kecepatan seluruh pipeline, bukan kecepatan satu pengembang; (5) task yang selesai tetapi belum masuk branch tidak terhitung; (6) perbaikan di luar ID task (TD-xx) tidak dihitung.
- Yang **tidak** dapat disimpulkan dari angka ini: besaran perlambatan Rust (RT-24) tetap `[TIDAK DIKETAHUI]` karena tidak ada pembanding non-Rust dan tidak ada pencatatan waktu per siklus verifikasi. Lama `scripts/verify.sh` juga tidak diukur di dokumen ini. Penilaian akhir ada di gerbang re-plan 2026-12-11 (`docs/07-phase-plan.md` §3 butir 5).
- Baseline cakupan (KEPUTUSAN-134, coding-standards §7.5) belum diukur di task ini: `[TIDAK DIKETAHUI]`, diukur terpisah.
