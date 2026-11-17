# Tahap 7: Phase Plan dan Backlog

Tujuan: memecah seluruh scope menjadi fase-fase yang masing-masing menghasilkan produk berjalan end-to-end. Backlog adalah artefak hidup yang terus diperbarui sampai hypercare.

Input: semua dokumen di `docs/`.

## Hierarki backlog

- **Epic**: modul besar, misalnya Autentikasi atau Pembayaran.
- **User Story**: kebutuhan dari sudut pengguna, lengkap dengan acceptance criteria dan rujukan ke PRD.
- **Task**: pekerjaan teknis konkret berdurasi 2–4 jam.

## Langkah

1. Lengkapi `docs/backlog.md` dengan **enabler story** dari Tahap 4–5 (setup repo, CI/CD, database, auth, logging, audit trail) dan **story UI** dari Tahap 6.
2. Prioritaskan dengan MoSCoW.
3. Bagi ke fase:
    - **Fase 1 hanya berisi *Must*** yang membentuk MVP end-to-end paling tipis.
    - **Fase 1 wajib memuat enabler fondasi**: core schema, autentikasi dan RBAC, Audit Trail Foundation, CI pipeline, dan reference implementation.
    - Setiap fase punya deliverable dan target tanggal.
4. Pecah story Fase 1 menjadi task. **Tinjau dari sudut pandang Tech Lead:** urutan dan dependensi teknis antar-task, task yang harus selesai lebih dulu (misalnya skema dan auth sebelum fitur), dan estimasi ukuran. Rencana fase yang disusun tanpa tinjauan ini hampir selalu salah urutan.
5. Panggil subagent `qa-engineer` untuk menyusun **test plan** dan **test case** dari PRD (bukan dari kode), simpan di `docs/test-plan.md`.

Output: `docs/07-phase-plan.md`, `docs/backlog.md` terprioritas, `docs/test-plan.md`.

## Definition of Ready (story boleh dikerjakan)

- Acceptance criteria jelas dan bisa diuji
- Merujuk ke bagian PRD dan ADR yang relevan
- Dependensi sudah selesai atau jelas
- Ukurannya muat dalam satu sesi kerja (satu PR yang bisa direview dalam 15 menit)

## Definition of Done (story dianggap selesai)

- CI hijau: lint, type check, test, security scan
- Mengikuti pola reference implementation
- Lolos code review oleh subagent `code-reviewer`
- Audit trail tercatat untuk aksi sensitif
- File migrasi disertakan kalau ada perubahan skema
- Acceptance criteria terverifikasi

Tulis DoR dan DoD ini juga di bagian atas `docs/backlog.md`.

## Change control

Ide baru di tengah development dicatat sebagai change request (CR-xxx) di `docs/backlog.md`, lalu dievaluasi dampaknya terhadap scope, waktu, dan arsitektur sebelum diprioritaskan. Perubahan pada tabel inti atau ADR selalu diperlakukan sebagai CR.

## Gate Tahap 7

- [ ] Semua story Fase 1 memenuhi Definition of Ready
- [ ] Enabler fondasi ada di Fase 1
- [ ] Test plan tersusun dari PRD
- [ ] User menyetujui pembagian fase
