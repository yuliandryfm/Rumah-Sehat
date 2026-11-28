# QA cakupan test CR-021 Fase 1 (persiapan F1-T71)

Tanggal: 2026-11-28 · Penyusun: Claude (persona QA Engineer, independen dari developer) · Basis: cabang `tahap-08-fase-1` commit `18b29a1`
Cakupan: US-1101, US-1801, US-1802, US-1901–US-1904, NFR yang diubah CR-021, UP-79–UP-87, AB-80–AB-87, event audit korporat (F1-T109). Tingkat ketelitian T1 untuk autentikasi, isolasi tenant, persetujuan, undangan, dan audit `[DOK: CLAUDE.md]` "Modul kritis".

Dokumen ini laporan cakupan, bukan pengganti `docs/test-plan.md`. Kolom "Test case" `docs/traceability.md` belum diubah di task ini, karena task ini hanya boleh menulis berkas ini. Usulan isinya ada di §6.

## 1. Temuan utama

1. **`docs/test-plan.md` belum memuat TC untuk CR-021.** Yang ada hanya TC-US-1101-1 s.d. -6 (masih berlabel fase F3) `[DOK: test-plan.md]` §US-1101. US-1801, US-1802, US-1901–US-1904, dan NFR CR-021 tidak punya TC; traceability menulis "— (dibuat QA dari PRD v1.3)" `[DOK: traceability.md]` baris BR-130–BR-136. Karena itu TC di §3 diturunkan QA langsung dari AC PRD v1.3, satu TC per AC (ID `QA-<story>-AC<n>`). TC ini usulan dan perlu dipindah ke test-plan sebelum F1-T71.
2. **Spesifikasi yang berlaku = PRD v1.3 + CR yang disetujui, tetapi teks PRD belum diperbarui.** CR-026 (form tanpa password, undangan seragam), CR-031 (dokumen diunggah setelah aktivasi), CR-035 (NIB terdaftar diproses seperti NIB baru), dan CR-036/CR-040 (koreksi data saat Ditolak) mengubah US-1901 AC1, AC2, AC7, US-1902 AC1, AC5, dan US-1903 AC4 `[DOK: backlog.md]` CR-026, CR-031, CR-035, CR-036. PRD masih Versi 1.3 dengan teks lama `[DOK: 03-prd.md]` baris 4. Penilaian di §3 memakai teks yang sudah diubah CR, dan selisihnya dicatat sebagai celah spesifikasi (§6).
3. **Test realm `korporat` di Keycloak tidak masuk gerbang otomatis.** `infra/keycloak/test/keycloak_check.py` (MFA/TOTP, kode pemulihan, penguncian, verifikasi email, undangan HR 24 jam sekali pakai) hanya berjalan lewat `bash scripts/keycloak-check.sh ephemeral`. Skrip ini tidak dipanggil `scripts/verify.sh` maupun `.github/workflows/ci.yml` (`grep keycloak-check` pada keduanya: kosong). Ini TD-16 yang masih terbuka `[DOK: backlog.md]` TD-16. QA tidak menjalankannya di task ini, dan di `docs/sources.md` tidak ditemukan catatan run yang menyebut realm `korporat` (hasil `grep`). Status lulusnya `[TIDAK DIKETAHUI]`.
4. **`pnpm --filter <app> test` tidak menjalankan apa pun.** `@rs/korporat`, `@rs/backoffice`, dan `@rs/situs` tidak punya skrip `test`. Perintah itu keluar 0 tanpa keluaran, jadi terlihat hijau padahal tidak ada test yang jalan. Test web hanya berjalan dari root workspace (`pnpm exec vitest run <app>/`, atau `pnpm run test` di `verify.sh`). Perintah yang benar ada di §2.
5. **Dua perilaku AC T1 yang belum ada:**
   - **US-1903 AC1:** perusahaan buatan Verifikator langsung Aktif tanpa dokumen dan bukti kewenangan. Badan permintaan di test hanya data perusahaan dan email HR (`apps/rs-backend/tests/corporate_admin_create.rs` fn `body`). Layar backoffice menyatakan "unggah dokumen menyusul" (`web/backoffice/src/komponen/perusahaan/perusahaan.test.tsx`, "catatan jujur").
   - **US-1802 AC7:** pratinjau backoffice tidak memakai aturan render situs. Markdown ditampilkan sebagai teks polos di `<pre>` (`web/backoffice/src/lib/cms/pratinjau.ts` baris 4–7, `TODO(F1-T96)`).

## 2. Bukti yang dijalankan (2026-11-28, worktree di `18b29a1`)

| Perintah | Hasil |
| --- | --- |
| `~/.local/bin/rs-antre bash scripts/db-ephemeral.sh cargo test -p rs-backend --test corporate_admin_create --test corporate_audit_chain --test corporate_correction --test corporate_correction_marker --test corporate_documents_access --test corporate_documents_http --test corporate_documents_pdf_qpdf --test corporate_hr_invite --test corporate_hr_members --test corporate_isolation --test corporate_nib_shared --test corporate_registration_http --test corporate_resubmission --test corporate_verification --locked` (`SQLX_OFFLINE=true`) | EXIT=0; 14 target; **94 lulus, 0 gagal, 7 diabaikan** (semuanya `corporate_documents_pdf_qpdf.rs`, butuh biner qpdf 12.4.2) |
| `~/.local/bin/rs-antre cargo test -p authz --test authz_corporate --locked` | 5 lulus, 0 gagal |
| `pnpm --filter @rs/korporat test` | EXIT=0, **tanpa keluaran** (tidak ada skrip `test`; lihat §1 butir 4) |
| `pnpm exec vitest run korporat/` (dari `web/`) | 17 berkas, 206 lulus, 0 gagal |
| `pnpm exec vitest run backoffice/` | 9 berkas, 146 lulus, 0 gagal |
| `pnpm exec vitest run situs/` | 21 berkas, 372 lulus, 0 gagal |
| `pnpm exec vitest run scripts/caddy-snippet scripts/realm-audience scripts/batas-client packages/bff-auth/src/routes packages/bff-auth/src/session` | 95 lulus, 0 gagal |
| `pnpm --filter @rs/situs build` (termasuk `web/situs/scripts/periksa-build.mjs`) | EXIT=0; "periksa-build situs: header keamanan, CSP bernonce, tanpa cookie, 404 dan berkas statis lulus" |

Tidak dijalankan oleh QA di task ini (ada, dan berjalan di langkah `cargo-test` `verify.sh`): `apps/rs-backend/tests/content_cms.rs`, `content_images.rs`, `content_public.rs`, `identity_keycloak_events.rs`, `http_trusted_proxy.rs`, `identity_idp_worker.rs`. Test qpdf asli berjalan di job CI `test-qpdf` (`.github/workflows/ci.yml` baris 265 dst.), tidak lokal. Hasil `verify.sh` penuh dilaporkan di pesan commit task ini.

## 3. Pemetaan TC ke test otomatis

Status: **T** = setiap klausa Then diuji otomatis di lapisan yang benar; **S** = sebagian (ada klausa Then atau kondisi negatif yang belum diuji, atau test hanya ada di jalur yang tidak masuk gerbang dan belum terkonfirmasi lulus); **N** = tidak ada test. Kolom "Jalan" berarti dijalankan QA di §2 (L = lulus; V = hanya di `verify.sh`; K = hanya `keycloak-check.sh`, tidak dijalankan). Singkatan berkas: `rb/` = `apps/rs-backend/tests/`, `ko/` = `web/korporat/src/`, `bo/` = `web/backoffice/src/`, `si/` = `web/situs/src/`, `kc` = `infra/keycloak/test/keycloak_check.py`.

### 3.1 US-1101 Pengelolaan konten CMS (TC dari test-plan)

| TC | Sumber | Test otomatis | Jalan | Status |
| --- | --- | --- | --- | --- |
| TC-US-1101-1 | US-1101 AC1, BR-061 | `rb/content_cms.rs::new_content_is_draft_and_can_be_previewed`; `bo/komponen/cms/cms.test.tsx` "EditorKonten konten baru", "pratinjau mengikuti isi" | V, L | T (level E2E peramban belum ada, masuk §4) |
| TC-US-1101-2 | US-1101 AC1 | `rb/content_cms.rs::every_endpoint_rejects_missing_token_other_realm_inactive_and_other_sub_roles`; `rb/content_images.rs::image_endpoints_reject_missing_token_and_other_sub_roles`; `bo/lib/cms/cms.test.ts` "menu Konten hanya untuk sub-peran Konten" | V, L | T |
| TC-US-1101-3 | US-1101 AC2 | `rb/content_cms.rs::scheduled_content_goes_live_ends_and_archives` (jam diset tepat ke waktu tayang dan waktu selesai), `schedule_rejects_past_start_offsetless_time_and_inverted_window` | V | T |
| TC-US-1101-4 | US-1101 AC3, ADR-003, E-16 | `rb/content_cms.rs::soft_delete_requires_reason_hides_content_and_restores` (outbox `e_16` lengkap lewat `audit_support::assert_events_complete`); `bo/komponen/cms/cms.test.tsx` "hapus: alasan wajib", "filter Terhapus ... Pulihkan" | V, L | T |
| TC-US-1101-5 | US-1101 AC4 | TIDAK ADA. Fase 1 belum punya harga atau katalog, jadi "promo tidak mengubah harga" belum bisa diuji | — | N |
| TC-US-1101-6 | US-1101 AC1, NFR-SEC (T-35) | `rb/content_cms.rs::xss_payloads_are_rejected_on_save_and_never_stored`; `si/komponen/markdown.test.tsx` (48 kasus); `bo/lib/cms/cms.test.ts` "dokumen pratinjau" | V, L | T |

### 3.2 US-1801 Halaman company profile publik (BR-130, BR-131)

| TC | Test otomatis | Jalan | Status |
| --- | --- | --- | --- |
| QA-1801-AC1 sembilan halaman dari konten tayang | `rb/content_public.rs::live_fixed_page_is_public_without_cookie_or_editorial_fields`; `si/konten/konten.test.ts` (klien `/v1/public/content/pages/{halaman}`); `periksa-build.mjs` (Beranda + 8 jalur: 200) | V, L | T (periksa-build memakai mode konten `contoh`, TD-23; situs dengan backend nyata ada di §4) |
| QA-1801-AC2 urutan layanan dari CMS | `si/konten/struktur.test.ts` "uraiLayanan ... urutan sama dengan CMS"; `si/komponen/tampilan-cr033.test.tsx` "kartu layanan ... urutan CMS" | L | T |
| QA-1801-AC3 draf, terjadwal, berakhir, arsip, terhapus = tidak ditemukan | `rb/content_public.rs::draft_scheduled_ended_archived_deleted_and_app_only_content_do_not_leak`; `si/konten/konten.test.ts` "404 → null (draf dan tidak ada tidak dibedakan)"; `si/konten/halaman.test.ts` | V, L | T |
| QA-1801-AC4 versi dan tanggal berlaku, tanda draf | `si/komponen/halaman-t97.test.tsx` (tanda draf, tanggal dari tanggal tayang); `periksa-build.mjs` "/kebijakan: penanda draf" | L | **S**: versi selalu tampil `[TIDAK DIKETAHUI]` karena model konten tidak punya field versi (judul test "versi [TIDAK DIKETAHUI]") |
| QA-1801-AC5 tanpa cookie sesi dan data pribadi | `si/keamanan.test.ts` "tidak ada Set-Cookie"; `si/kepatuhan.test.ts`; `periksa-build.mjs` (tanpa Set-Cookie di 9 halaman, 404, berkas statis); `rb/content_public.rs` (tanpa field editorial) | L, V | T |
| QA-1801-AC6 aksesibilitas otomatis | `si/komponen/a11y.test.tsx` (13 keadaan, axe-core); `si/kontras.test.ts` | L | T (axe di jsdom tidak memeriksa kontras hasil render; kontras diuji per token) |
| QA-1801-AC7 backend tidak tersedia = galat umum | `si/konten/konten.test.ts` "500, 429, galat jaringan ... KontenTidakTersedia"; `si/komponen/komponen.test.tsx` "galat: pesan umum tanpa rincian"; `si/app/halaman.test.ts` | L | T |

### 3.3 US-1802 Konten situs di CMS (BR-131, BR-061)

| TC | Test otomatis | Jalan | Status |
| --- | --- | --- | --- |
| QA-1802-AC1 kanal dan halaman tujuan | `rb/content_cms.rs::channel_and_site_page_are_validated_per_field`; `rb/content_public.rs::site_list_follows_channel_page_and_new_fixed_page_version_replaces_old`, `..._app_only_content_do_not_leak`; `bo/komponen/cms/cms.test.tsx` "kanal Aplikasi menyembunyikan halaman situs" | V, L | T (arah "konten situs tidak tampil di aplikasi" menunggu endpoint US-1102, Fase 3) |
| QA-1802-AC2 satu versi tayang per halaman tetap | `rb/content_cms.rs::fixed_site_page_has_one_live_version`; `rb/content_public.rs::site_list_follows_..._replaces_old` | V | T |
| QA-1802-AC3 promo hanya informasi | TIDAK ADA (belum ada harga di Fase 1) | — | N |
| QA-1802-AC4 tanpa sub-peran Konten = 403 dan tercatat | 403: `rb/content_cms.rs::every_endpoint_rejects_..._other_sub_roles`. Pencatatan percobaan: tidak ada assertion | V | **S** |
| QA-1802-AC5 gambar tanpa alt ditolak per field | `rb/content_cms.rs::image_without_alt_text_cannot_be_scheduled`; `rb/content_images.rs::image_upload_rejects_fake_type_size_wrong_owner_missing_alt_and_non_draft`; `bo/.../cms.test.tsx` "teks alternatif wajib" | V, L | T |
| QA-1802-AC6 XSS ditolak atau dinetralkan saat simpan | `rb/content_cms.rs::xss_payloads_are_rejected_on_save_and_never_stored` (`<script>`, `<iframe>`, `onerror`, `onload`, `javascript:`, `data:`, gambar sumber luar) | V | T |
| QA-1802-AC7 aturan render sama di situs dan pratinjau | Situs: `si/komponen/markdown.test.tsx`. Pratinjau: `bo/lib/cms/cms.test.ts` "payload XSS ... menjadi teks" (di-escape sebagai teks polos `<pre>`, bukan renderer yang sama) | L | **S**: payload gagal dieksekusi di keduanya, tetapi syarat "aturan render yang sama" belum terpenuhi (`pratinjau.ts` `TODO(F1-T96)`) |
| QA-1802-AC8 iframe `sandbox` tanpa `allow-same-origin` | `bo/komponen/cms/cms.test.tsx` "PratinjauAman (e2e jsdom: payload XSS)"; `bo/lib/cms/cms.test.ts` "sandbox kosong" | L | T |
| QA-1802-AC9 CSP tanpa skrip inline dan pihak ketiga | `si/keamanan.test.ts`; `periksa-build.mjs`; `bo/lib/lib.test.ts` "CSP bernonce: tanpa unsafe-*" | L | T |

### 3.4 US-1901 Pendaftaran perusahaan mandiri (T1, BR-136, UP-79; diubah CR-026, CR-031, CR-035, CR-036)

| TC | Test otomatis | Jalan | Status |
| --- | --- | --- | --- |
| QA-1901-AC1 Menunggu verifikasi, akun HR realm korporat, verifikasi email, bukti persetujuan per versi | `rb/corporate_registration_http.rs::corporate_registration_creates_pending_company_hr_account_consent_and_complete_audit`, `..._requires_consent_and_current_text_version`, `..._draft_text_is_refused_in_production`; `ko/komponen/daftar/daftar.test.tsx`; verifikasi email wajib: `kc::cek_verifikasi_email_dulu` dan `cek_pekerja_idp` (realm korporat) | L, K | **S**: bagian "verifikasi email wajib" hanya diuji di `keycloak-check` (§1 butir 3) |
| QA-1901-AC2 dokumen wajib, jenis, ukuran (setelah aktivasi, CR-031) | `rb/corporate_documents_access.rs::slot_requests_are_limited_in_type_size_and_count` (batas 5.242.880/5.242.881 dan 10.485.760/10.485.761 byte), `queue_marks_incomplete_submissions_and_approval_requires_all_three_documents`; `rb/corporate_documents_http.rs::rejected_content_is_removed_never_recorded_and_audited_as_failed_upload`; `ko/verifikasi-e2e.test.ts` (415 PDF); `ko/komponen/verifikasi/verifikasi.test.tsx` | L | T (inspeksi PDF dengan qpdf asli hanya di CI `test-qpdf`) |
| QA-1901-AC3 format NIB/NPWP per field | `rb/corporate_registration_http.rs::corporate_registration_validates_each_field_and_never_accepts_a_password`; `ko/lib/daftar/daftar.test.ts` | L | T |
| QA-1901-AC4 email terdaftar = layar sama, petunjuk dikirim | `rb/corporate_registration_http.rs::corporate_registration_existing_email_gets_identical_answer_and_changes_nothing`, `..._new_and_duplicate_take_at_least_the_minimum_time`; `ko/komponen/hr/hr.test.tsx` "anti-enumerasi" (undangan) | L | **S**: test menegaskan nol efek ("tidak ada ... niat, job"), jadi email petunjuk ke pemilik alamat tidak pernah dikirim. Klausa "petunjuk dikirim ke email itu" belum terpenuhi (§6 C-03) |
| QA-1901-AC5 kompensasi: tidak ada HR tanpa perusahaan dan persetujuan | Jalur sinkron: transaksi tunggal di `corporate_registration_creates_...`, `..._concurrent_same_nib_...`. Jalur asinkron (worker IdP gagal permanen): `rb/identity_idp_worker.rs::activation_and_orphan_deletion_refuse_credentialed_or_foreign_users` hanya menutup pengguna Keycloak | L, V | **S**: perusahaan dan keanggotaan yatim setelah penyediaan gagal belum ditangani (TD-42, TD-41, tenggat "sebelum penutupan Fase 1") |
| QA-1901-AC6 Menunggu/Ditolak: hanya status, alasan, profil | `rb/corporate_resubmission.rs::status_page_shows_own_company_profile_only`; `rb/corporate_hr_members.rs::every_endpoint_rejects_..._unverified_companies`; `rb/corporate_hr_invite.rs::invitations_are_rejected_for_..._unverified_companies`; `rb/corporate_documents_access.rs::documents_are_manageable_only_while_pending_or_rejected`; `ko/komponen/beranda/beranda.test.tsx` | L | T |
| QA-1901-AC7 NIB terdaftar (CR-035: diproses seperti NIB baru, ditandai di antrean) | `rb/corporate_registration_http.rs::corporate_registration_registered_nib_is_processed_exactly_like_a_new_one`; `rb/corporate_nib_shared.rs` (4 test); `bo/komponen/perusahaan/perusahaan.test.tsx` "CR-035" | L | T terhadap CR-035 (teks PRD AC7 usang, §6 C-01) |

### 3.5 US-1902 Verifikasi perusahaan oleh Verifikator (T1)

| TC | Test otomatis | Jalan | Status |
| --- | --- | --- | --- |
| QA-1902-AC1 antrean terlama, dokumen lewat URL pendek setelah otorisasi | `rb/corporate_verification.rs::queue_lists_pending_submissions_oldest_first_without_npwp`, `queue_cursor_keeps_its_position_...`, `document_url_is_issued_only_after_e05_is_recorded`, `opening_company_detail_records_e05_without_restricted_data`; `rb/corporate_documents_http.rs::expired_download_url_is_rejected_by_the_provider`; `bo/.../perusahaan.test.tsx` | L | T |
| QA-1902-AC2 setujui = Aktif | `rb/corporate_verification.rs::approval_activates_the_company_and_records_e10`; `rb/corporate_documents_access.rs::decision_is_bound_to_the_document_version_the_verifier_saw` | L | T ("menu pesanan" belum ada di Fase 1, K2) |
| QA-1902-AC3 tolak dengan kode alasan, HR melihat alasan, ajukan ulang | `rb/corporate_verification.rs::rejection_requires_an_active_listed_reason`, `resubmission_after_rejection_opens_a_new_round`; `rb/corporate_resubmission.rs` (6); `rb/corporate_correction.rs` (9); `ko/komponen/verifikasi/verifikasi.test.tsx` "Ditolak dan ajukan ulang" | L | T |
| QA-1902-AC4 keputusan bersamaan: yang kedua ditolak dan tercatat | `rb/corporate_verification.rs::first_decision_wins_when_two_verifiers_decide_concurrently` (409 `4090001` + E-10 `outcome = failure` lengkap) | L | T |
| QA-1902-AC5 bukan Verifikator 403, HR 404, tercatat | `rb/corporate_verification.rs::access_is_denied_to_everyone_but_active_verifiers`: sub-peran lain 403 `4030001`; **HR dijawab 401 `4010001`, bukan 404**; tidak ada assertion pencatatan (test hanya memastikan outbox tidak berubah untuk objek itu) | L | **S** (selisih 401/404 tercatat di TD-37, belum diselaraskan) |

### 3.6 US-1903 Perusahaan dibuatkan Verifikator (T1)

| TC | Test otomatis | Jalan | Status |
| --- | --- | --- | --- |
| QA-1903-AC1 data, dokumen, bukti kewenangan, email HR; Aktif + undangan | `rb/corporate_admin_create.rs::verifier_creates_an_active_company_with_one_provisioning_intent_and_audit`, `repeating_the_same_creation_never_duplicates_...`, `concurrent_creations_with_the_same_nib_or_the_same_email_create_exactly_one`, `invalid_input_is_rejected_before_any_effect`; `bo/.../perusahaan.test.tsx` "Buat perusahaan" | L | **S**: dokumen dan bukti kewenangan tidak diterima maupun diuji (§1 butir 5) |
| QA-1903-AC2 tautan undangan kedaluwarsa atau dipakai ditolak | `kc::cek_undangan_hr` (24 jam, sekali pakai; KEPUTUSAN-334) | K | **S** (tidak di gerbang, belum terkonfirmasi lulus) |
| QA-1903-AC3 E-10 memuat pelaku dan sumber "dibuat admin" | `rb/corporate_admin_create.rs::verifier_creates_...` (komentar uji baris 261, 362: E-10 pelaku + sumber) | L | T |
| QA-1903-AC4 NIB perusahaan Aktif ditolak | `rb/corporate_admin_create.rs::duplicate_nib_is_rejected_generically_without_any_side_effect`, `nib_held_only_by_non_active_companies_does_not_block_creation`; `bo/.../perusahaan.test.tsx` "409 4090110" | L | T |

### 3.7 US-1904 Akun HR (T1)

| TC | Test otomatis | Jalan | Status |
| --- | --- | --- | --- |
| QA-1904-AC1 password + TOTP, pendaftaran TOTP dan kode pemulihan wajib | `kc::cek_konfigurasi`, `cek_mfa_dan_token`, `cek_batal_pendaftaran_totp` (realm korporat) | K | **S** (tidak di gerbang). Risiko: CR-041/ADR-032 (Diusulkan) mengganti TOTP dengan OTP WhatsApp untuk realm korporat, sehingga AC dan test ini akan berubah |
| QA-1904-AC2 undang HR, nonaktifkan, HR aktif terakhir | `rb/corporate_hr_invite.rs` (9), `rb/corporate_hr_members.rs::deactivating_another_hr_revokes_access_immediately_and_is_audited`, `only_active_hr_stays_and_invited_hr_can_be_deactivated`, `concurrent_mutual_deactivation_leaves_exactly_one_active_hr`, `self_deactivation_is_rejected_without_side_effects`; `ko/komponen/hr/hr.test.tsx`; `ko/hr-e2e.test.ts` | L | T (tanpa batas laju undangan: TD-40, §6 C-05) |
| QA-1904-AC3 isolasi: perusahaan lain = 404 | `rb/corporate_isolation.rs` (harness 9 dari 9 endpoint `openapi/korporat.json`, termasuk ID disuntik di query/body dan sidik penulisan ke akun HR korban); `crates/authz/tests/authz_corporate.rs::hr_sees_only_own_company_and_foreign_objects_are_indistinguishable_404`; `rb/corporate_documents_access.rs::hr_reaches_only_its_own_companys_documents_slots_and_urls`; `rb/corporate_hr_members.rs::foreign_unknown_or_malformed_member_is_not_found` | L | T untuk endpoint Fase 1 (pesanan, peserta, batch impor, invoice, rekap belum ada; "percobaan ke data peserta tercatat" baru dapat diuji di K2) |
| QA-1904-AC4 token realm korporat ditolak di API lain dan sebaliknya | `crates/authz/tests/authz_corporate.rs::korporat_tokens_and_other_realm_tokens_are_rejected_both_ways`; `rb/corporate_hr_members.rs::every_endpoint_rejects_missing_tokens_other_realms_...`; `rb/corporate_verification.rs::access_is_denied_...` (token HR di `/backoffice`); `ko/bff-sesi.test.ts` (13 jalur ditolak); `web/scripts/realm-audience.test.ts` | L | T |
| QA-1904-AC5 HR atau perusahaan dinonaktifkan: permintaan berikutnya ditolak | `rb/corporate_hr_members.rs::deactivating_another_hr_revokes_access_immediately_and_is_audited`; `rb/corporate_isolation.rs::database_membership_and_company_status_gate_every_request`; `crates/authz/tests/authz_corporate.rs::membership_and_company_status_are_enforced_on_every_request`; `ko/hr-e2e.test.ts` "akun nonaktif (401 4010002)" | L | T (penonaktifan perusahaan oleh admin = US-1916, belum ada jalurnya) |
| QA-1904-AC6 idle 30 menit | `ko/bff-sesi.test.ts` "tidak aktif 30 menit", "batas mutlak 12 jam"; `ko/hr-e2e.test.ts` (29 vs 30 menit); `kc` `SESI["korporat"] = (1800, 43200)` | L, K | T |
| QA-1904-AC7 (porsi Fase 1) masuk dengan kode pemulihan | `kc::cek_kode_pemulihan`, `cek_otp_dan_kode_pemulihan_salah_dikunci` (realm korporat) | K | **S** (tidak di gerbang) |
| QA-1904-AC8, AC9 | Fase 2 `[DOK: 07-phase-plan.md]` §8 "Reset MFA HR, K1-F2". Catatan: ADR-032 P32-7 B memajukan pemulihan lewat tiket support ke Fase 1, tetapi ADR masih Diusulkan | — | di luar hitungan |

### 3.8 NFR CR-021, UP, AB, dan audit

| TC | Test otomatis | Jalan | Status |
| --- | --- | --- | --- |
| QA-NFR-SEC-02 MFA wajib HR | lihat QA-1904-AC1 | K | **S** |
| QA-NFR-SEC-04 sesi member area 30 menit | lihat QA-1904-AC6 | L | T |
| QA-NFR-SEC-06 isolasi antar-perusahaan termasuk unduhan berkas | lihat QA-1904-AC3; `rb/corporate_documents_access.rs::hr_reaches_only_its_own_companys_documents_slots_and_urls` | L | T (ekspor belum ada di Fase 1) |
| QA-NFR-SEC-08 jenis dan ukuran berkas | lihat QA-1901-AC2; `rb/content_images.rs::image_upload_rejects_fake_type_size_...` (5.242.881 byte) | L, V | T (CSV/XLSX peserta = K2) |
| QA-NFR-PERF-04 p95 ≤ 3 detik situs dan member area | TIDAK ADA | — | N |
| QA-NFR-COMP-03 dua versi terakhir Chrome, Edge, Firefox, Safari | TIDAK ADA (CR-033 mencatat pemeriksaan di peramban sungguhan belum dilakukan) | — | N |
| QA-UP-79 isian formulir pendaftaran (tanpa password, CR-026) | `rb/corporate_registration_http.rs::corporate_registration_validates_each_field_and_never_accepts_a_password`; `ko/lib/daftar/daftar.test.ts` "tanpa field lain (tanpa password)" | L | T |
| QA-AB-86 ID perusahaan dari sesi, bukan parameter | `rb/corporate_hr_invite.rs::invalid_bodies_and_injected_company_ids_are_rejected_without_effect`; `rb/corporate_isolation.rs` (`injected_query`); `rb/corporate_resubmission.rs::resubmission_only_ever_touches_the_callers_company`; `rb/corporate_correction.rs::correction_only_ever_touches_the_callers_company` | L | T |
| QA-AUD-1 event korporat E-04, E-05, E-10, E-11, E-13, E-16 lengkap, berantai, tanpa field terlarang (F1-T109, ADR-005) | `rb/corporate_audit_chain.rs::corporate_events_are_clean_and_chained_in_the_audit_database`; `rb/audit_support/mod.rs` (`assert_corporate_events_clean`, `corporate_checker_rejects_leaks`); `rb/corporate_verification.rs::npwp_never_reaches_audit_payloads_or_error_bodies`; `rb/corporate_documents_http.rs::hr_uploads_three_documents_..._audited_without_secrets`; `rb/identity_keycloak_events.rs::hr_events_from_korporat_realm_are_recorded_with_the_company_as_tenant` (E-01–E-04) | L, V | T |

**Di luar Fase 1 (tidak dihitung):** UP-80, UP-81, UP-82, UP-83, UP-84, UP-85, UP-86 (E-18 impor dan undangan peserta), UP-87, AB-80, AB-81, AB-82, AB-83, AB-84, AB-85, AB-87. Semuanya turunan US-1905–US-1915 (K2, Fase 2) `[DOK: 07-phase-plan.md]` §8 baris "Layanan korporat K2". Di Fase 1 belum ada endpoint impor peserta, undangan peserta, persetujuan berbagi ringkasan, invoice, atau mutasi bank, jadi negatif T1 untuk bidang itu belum dapat diuji. Pemeriksaan tanpa endpoint itu sudah tercakup harness isolasi (QA-1904-AC3), yang gagal bila endpoint `/v1/corporate` baru tidak didaftarkan (`every_corporate_endpoint_is_registered_in_harness`).

### 3.9 Hitungan

| Kelompok | TC | T | S | N |
| --- | --- | --- | --- | --- |
| US-1101 | 6 | 5 | 0 | 1 |
| US-1801 | 7 | 6 | 1 | 0 |
| US-1802 | 9 | 6 | 2 | 1 |
| US-1901 | 7 | 4 | 3 | 0 |
| US-1902 | 5 | 4 | 1 | 0 |
| US-1903 | 4 | 2 | 2 | 0 |
| US-1904 (AC1–AC7) | 7 | 5 | 2 | 0 |
| NFR, UP-79, AB-86, audit | 9 | 6 | 1 | 2 |
| **Total** | **54** | **38** | **12** | **4** |

## 4. Gap menurut severity

Severity mengikuti dampak bila perilaku itu salah di production. Rujukan TD dari `docs/backlog.md`.

| # | Severity | Gap | TC / sumber | Tindakan yang diusulkan |
| --- | --- | --- | --- | --- |
| G-01 | High | US-1903 AC1: perusahaan buatan Verifikator Aktif tanpa dokumen dan bukti kewenangan; tidak ada test | QA-1903-AC1, BR-136 | Developer: lampiran dokumen di jalur buat perusahaan (atau keputusan user menunda, dicatat sebagai CR); test negatif "tanpa dokumen ditolak" |
| G-02 | High | Test realm `korporat` (TOTP wajib, kode pemulihan, penguncian, verifikasi email, undangan 24 jam sekali pakai) tidak masuk gerbang dan tidak ada catatan run untuk korporat | QA-1901-AC1, QA-1903-AC2, QA-1904-AC1, AC7, NFR-SEC-02; TD-16 | Jalankan `bash scripts/keycloak-check.sh ephemeral` setelah agen tema korporat merge dan lampirkan keluaran di F1-T71; sambungkan ke CI (butuh izin user, TD-16) |
| G-03 | Medium (T1) | US-1901 AC5: penyediaan HR gagal permanen meninggalkan perusahaan dan keanggotaan yatim; tidak ada test negatif | QA-1901-AC5; TD-41, TD-42 (tenggat sebelum penutupan Fase 1) | Developer: perbaiki di transaksi hasil worker; test: job `create_user` gagal permanen → perusahaan dan keanggotaan terhapus lunak dengan alasan, E-10 tercatat |
| G-04 | Medium (T1) | Percobaan akses ditolak tidak diuji tercatat: US-1802 AC4 (CMS 403) dan US-1902 AC5 (403/401) | QA-1802-AC4, QA-1902-AC5; TD-37 (tenggat sebelum F1-T110) | Test: penolakan menulis log keamanan atau E-05 hasil gagal (pola `http_trusted_proxy.rs::denial_e05_records_real_client_behind_trusted_proxy`) |
| G-05 | Medium | US-1802 AC7: pratinjau backoffice tidak memakai renderer situs (teks polos `<pre>`) | QA-1802-AC7; TD-25; `pratinjau.ts` `TODO(F1-T96)` | Developer (agen situs/CMS): satu renderer bersama; test: payload yang sama menghasilkan DOM identik di situs dan pratinjau |
| G-06 | Medium | US-1901 AC4: email petunjuk ke pemilik email terdaftar tidak dikirim | QA-1901-AC4 | Keputusan: kirim "Anda sudah punya akun" dengan batas KEPUTUSAN-192, atau ubah AC lewat CR (§6 C-03) |
| G-07 | Medium | US-1902 AC5: HR dijawab 401, PRD meminta 404 | QA-1902-AC5; TD-37 | Selaraskan PRD (CR-026) atau implementasi; test mengikuti keputusan |
| G-08 | Medium | `pnpm --filter @rs/<app> test` keluar 0 tanpa menjalankan test | §1 butir 4 | Tambah skrip `test` per app, atau dokumentasikan perintah root; F1-T71 jangan memakai `--filter` sebagai bukti |
| G-09 | Medium | NFR-PERF-04 (p95 ≤ 3 detik) dan NFR-COMP-03 (empat peramban) tanpa test | QA-NFR-PERF-04, QA-NFR-COMP-03 | Uji manual di F1-T110 (§5 di bawah) atau tunda ke Tahap 9 dengan catatan |
| G-10 | Low | US-1801 AC4: versi kebijakan selalu `[TIDAK DIKETAHUI]`, tidak ada field versi | QA-1801-AC4 | Keputusan model konten (field versi) setelah L-08/L-20/PT-24 |
| G-11 | Low | Inspeksi PDF qpdf asli hanya di CI `test-qpdf`, lokal diabaikan (7 test) | QA-1901-AC2; TD-48 | Pastikan job CI `test-qpdf` hijau pada commit penutupan |
| G-12 | Low | Tidak ada batas laju undangan HR | QA-1904-AC2; TD-40 | Di luar AC (celah spesifikasi C-05) |

Negatif T1 yang **sudah** tercakup baik: isolasi tenant (harness 9/9 endpoint, ID disuntik, sidik penulisan), otorisasi per sub-peran (CMS, verifikasi, buat perusahaan), token lintas realm dua arah, persetujuan versi teks basi dan teks draf di production, keputusan bersamaan, nonaktif bersamaan, pengajuan ulang dan koreksi bersamaan, NPWP tidak masuk audit, dan event gagal E-10 untuk anti-enumerasi.

## 5. Verifikasi manual / e2e (F1-T110) dan set minimal

TC yang butuh verifikasi manual atau e2e, karena test otomatis memakai tiruan, jsdom, atau tidak di gerbang: QA-1801-AC1 (situs + backend nyata), TC-US-1101-1 (level E2E di test-plan), QA-1802-AC7/AC8 (peramban sungguhan), QA-1901-AC1 (email verifikasi lewat relay), QA-1903-AC2, QA-1904-AC1, AC6, AC7, NFR-SEC-02, NFR-PERF-04, NFR-COMP-03, QA-AUD-1 (rantai audit pada alur gabungan).

Set e2e minimal yang diusulkan, sesuai definisi F1-T110 `[DOK: 07-phase-plan.md]` §7 F1-T110, di lingkungan `infra/local/compose.yaml` dengan Keycloak, relay email, dan backend nyata:

| # | Skenario | Menutup |
| --- | --- | --- |
| E2E-1 | Admin Konten membuat halaman situs, menjadwalkan, lalu halaman tampil di `web/situs`; draf dan arsip menjawab 404; tanpa `Set-Cookie` | US-1101 AC1–AC2, US-1801 AC1, AC3, AC5, US-1802 AC1 |
| E2E-2 | Payload XSS (`<script>`, `javascript:`, `<iframe>`) disimpan lalu dibuka di pratinjau backoffice dan di situs pada peramban sungguhan: tidak ada eksekusi, iframe tanpa `allow-same-origin` | US-1802 AC6–AC8 |
| E2E-3 | Daftar perusahaan di `/daftar` → email aktivasi lewat relay → set password → daftar TOTP + kode pemulihan → unggah tiga dokumen → status Menunggu verifikasi | US-1901 AC1–AC2, AC6, US-1904 AC1 |
| E2E-4 | Verifikator membuka dokumen (E-05), menolak dengan alasan → HR melihat alasan, koreksi, ajukan ulang → Verifikator menyetujui → HR melihat Aktif | US-1902 AC1–AC3 |
| E2E-5 | HR mengundang HR kedua → aktivasi; HR kedua menonaktifkan HR pertama → sesi HR pertama ditolak di permintaan berikutnya; HR terakhir tidak dapat dinonaktifkan | US-1904 AC2, AC5 |
| E2E-6 | Dua perusahaan A dan B: HR A mencoba ID anggota, dokumen, dan URL unduh milik B lewat BFF → 404; token korporat ke API backoffice dan pasien ditolak | US-1904 AC3, AC4, NFR-SEC-06 |
| E2E-7 | Masuk dengan kode pemulihan; kode yang sama kedua kali ditolak; idle 30 menit mengakhiri sesi | US-1904 AC6, AC7 |
| E2E-8 | Setelah E2E-1 s.d. E2E-7: `audit-verify` rantai tenant kedua perusahaan utuh; payload tanpa password, OTP, token, NPWP lengkap | QA-AUD-1, ADR-005 |
| M-1 (manual) | Situs dan member area di dua versi terakhir Chrome, Edge, Firefox, Safari; ukur p95 muat halaman | NFR-COMP-03, NFR-PERF-04 |

## 6. Celah spesifikasi dan usulan rumusan

| # | AC | Masalah | Usulan rumusan |
| --- | --- | --- | --- |
| C-01 | US-1901 AC1, AC2, AC7; US-1902 AC1; US-1903 AC4 | Teks PRD v1.3 bertentangan dengan CR-026, CR-031, CR-035 yang sudah disetujui | Revisi PRD (v1.5) memasukkan isi CR; misalnya AC7: "Given NIB sudah dipakai perusahaan lain, When mendaftar, Then jawaban dan email identik dengan NIB baru, dan pengajuan ditandai 'NIB dipakai perusahaan lain' di antrean Verifikator" |
| C-02 | US-1902 AC5 | 404 untuk HR tidak sesuai penolakan realm seragam (401) | "Given token realm selain `admin` ..., Then ditolak 401 tanpa membedakan keberadaan objek; admin tanpa sub-peran Verifikator 403; keduanya tercatat di log keamanan" |
| C-03 | US-1901 AC4 | "petunjuk dikirim" bertentangan dengan nol efek KEPUTUSAN-324 | Pilih salah satu: (a) "petunjuk 'Anda sudah punya akun' dikirim dengan batas KEPUTUSAN-192", atau (b) hapus klausa itu |
| C-04 | US-1802 AC4, US-1902 AC5 | "tercatat" tidak menyebut tempat dan field | "... tercatat di log keamanan (atau E-05 hasil gagal) dengan aktor, endpoint, kode alasan, tanpa isi permintaan" |
| C-05 | US-1904 AC2 | Tidak ada batas laju undangan (TD-40) | "Undangan dibatasi N per HR per jam dan M per perusahaan per hari; melewati batas = 429 tanpa efek" (angka `[TIDAK DIKETAHUI]`, keputusan user) |
| C-06 | US-1801 AC4 | Sumber nomor versi kebijakan tidak diatur | "Versi diambil dari field `version` konten kebijakan; kosong = penanda draf" |
| C-07 | US-1904 AC1, AC7, NFR-SEC-02 | CR-041/ADR-032 (Diusulkan) mengganti TOTP dengan OTP WhatsApp untuk realm korporat | Bila diterima: tulis ulang AC1/AC7 dan TC terkait sebelum F1-T110; test `kc` korporat ikut diganti |

Usulan isi kolom "Test case" traceability (belum diterapkan, di luar izin tulis task ini): BR-130 → QA-1801-AC1..AC7; BR-131 → TC-US-1101-1..6, QA-1802-AC1..AC9; BR-136 → QA-1901-AC1..AC7, QA-1902-AC1..AC5, QA-1903-AC1..AC4, QA-UP-79; BR-132/BR-145 → QA-1904-AC1..AC7, QA-AB-86, QA-NFR-SEC-06.
