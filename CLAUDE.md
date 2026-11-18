# Ruang Sehat

Aplikasi mobile kesehatan (Flutter, Android dan iOS) untuk pasien yang kesulitan datang ke klinik atau apotek. Rilis pertama berisi tujuh layanan: konsultasi online dokter dan psikolog (chat dan video, berlangsung di Ruang Sehat), pendaftaran klinik dan lab rekanan, home visit tenaga medis, informasi biaya MCU dan lab, serta pembelian obat bebas, produk kecantikan, dan produk kesehatan dari outlet rekanan. Ruang Sehat berperan sebagai gateway: data klinik, lab, home visit, dan produk berasal dari integrasi dengan sistem rekanan. Tenaga kesehatan memakai aplikasi mobile, sedangkan admin klinik, lab, outlet, dan Ruang Sehat memakai portal web. Data wajib disimpan di Indonesia. Target go-live 1 Maret 2027. Rincian dan pertanyaan terbuka: `docs/00-project-brief.md`.

## Cara kerja proyek ini

- Proyek ini dibangun mengikuti runbook 15 tahap. Jalankan `/runbook status` untuk melihat posisi, `/runbook` untuk melanjutkan.
- Status setiap tahap dicatat di `docs/STATUS.md`. Semua dokumen proyek ada di `docs/`.
- Dokumen di `docs/` adalah sumber kebenaran. Kalau kode dan dokumen bertentangan, laporkan, jangan menebak.
- Ide atau perubahan baru di tengah jalan dicatat sebagai change request di `docs/backlog.md`, tidak langsung dikerjakan.
- Aturan kerja `.claude/skills/runbook/references/aturan-kerja.md` berlaku bersama anti-halu: ketelitian berjenjang (T1/T2/T3), kriteria blokir fact-checker (hanya hal substantif), riset secukupnya dan tepat waktu, pemilihan model per kelas task, serta commit dan push per tahap (Tahap 8: per task).

## Aturan nomor satu: tidak ada halusinasi

IMPORTANT: ikuti `.claude/skills/runbook/references/anti-halu.md` di setiap pekerjaan.
- Setiap klaim di dokumen punya label sumber. Tidak tahu ditulis `[TIDAK DIKETAHUI]`, bukan ditebak.
- Fakta bisnis tidak boleh diasumsikan. Fakta eksternal (versi, API, regulasi) diverifikasi ke sumber resmi dan dicatat di `docs/sources.md`.
- Jangan memanggil API atau menambah package yang belum dipastikan ada.
- Jangan menyatakan selesai atau test lulus tanpa menjalankan `scripts/verify.sh` dan menunjukkan output-nya.

## Aturan nomor dua: hanya commit hijau

IMPORTANT: tidak ada kode merah yang boleh di-commit, di branch mana pun. Git hook di `.githooks/` menolak commit dan merge merah. Jangan pernah memakai `--no-verify` atau `-n`, mengubah `core.hooksPath`, mengubah `.githooks/`, atau melonggarkan `scripts/verify.sh`. Kalau merah dan belum bisa diperbaiki, jangan commit; laporkan.

## Aturan inti (wajib)

- Database memakai snake_case (ADR-001).
- Primary key UUIDv7 dengan tipe UUID native (ADR-002).
- Data master memakai soft delete: `deleted_at`, `deleted_by`, `delete_reason`. Query default memfilter data terhapus (ADR-003).
- Data transaksi immutable: tanpa update/delete, koreksi lewat reversal entry (ADR-004).
- Setiap aksi sensitif wajib mencatat audit trail append-only. Jangan pernah mencatat password, PIN, OTP, token, atau nomor kartu/rekening lengkap (ADR-005).
- Perubahan skema hanya lewat file migrasi.
- Kalau menemukan pola baru yang belum ada standarnya, berhenti dan usulkan ADR.

## Modul kritis (T1)

Ditetapkan di Tahap 4 (`[KEPUTUSAN-37]`, `docs/04-arsitektur.md`). Perubahan di modul ini wajib review penuh (code-reviewer, ditambah security dan/atau DBA sesuai bidangnya) dan test termasuk skenario negatif (`aturan-kerja.md` §1):
- Autentikasi, sesi, dan MFA, termasuk konfigurasi realm Keycloak dan validasi token di backend (ADR-011)
- Otorisasi dan isolasi data, termasuk RLS dan role database RME minimalis (ADR-012)
- Harga dan markup, pembayaran, jurnal, refund, klaim, dan kewajiban rekanan (ADR-015)
- Pustaka audit, relay, dan database audit (ADR-009)
- Enkripsi, kunci data, pekerja privasi, retensi, dan anonimisasi (ADR-013)
- Persetujuan dan hak subjek data (EP-16)
- Gateway Partner Integration API: autentikasi klien, tanda tangan webhook, idempotensi (ADR-014)
- Berkas dokumen identitas dan klinis (ADR-018)
- Rekam medis RME minimalis dan pengiriman SATUSEHAT (ADR-020)
- File migrasi skema dan hak role database (ADR-008)
- Konfigurasi akses administratif production dan pipeline CI/CD (ADR-021)
- Ditambah di Tahap 5 (CR-012, ADR-024/ADR-025): server Next.js sebagai BFF (sesi dan token web), konfigurasi dan policy OpenBao (pengganti KMS dan secret manager), bastion dan skrip deploy production, konfigurasi backup/replikasi PostgreSQL (pgBackRest)

## Stack (Tahap 5, `docs/05-stack.md`, ADR-024, ADR-025)

- Backend Ruang Sehat dan RME minimalis: Rust (axum, tokio, sqlx, utoipa), satu workspace Cargo. Web (backoffice, portal rekanan, web RME): Next.js dengan server Next.js sebagai BFF. Mobile (pasien, tenaga kesehatan): Flutter. Satu monorepo (struktur di `docs/coding-standards.md`).
- Data: PostgreSQL 18 (tiga instans: aplikasi RS + keycloak, audit, RME), Valkey (pengganti "Redis", menunggu konfirmasi gate), Keycloak, OpenBao, object storage NEO (Biznet Gio).
- Hosting: VPS Biznet Gio (utama) + Hostinger Jakarta (standby dan salinan backup), Docker Compose, Caddy. Penyedia: DOKU (kandidat utama PG), Amazon SES Jakarta (API), Lalamove, LiveKit self-hosted.
- Versi persis dan sumbernya hanya di `docs/05-stack.md` dan `docs/sources.md`. Jangan menambah dependensi tanpa cek registry resmi dan catatan di `docs/sources.md`.

## Perintah

Berlaku setelah repositori kode dibuat di Tahap 8 Fase 1 (rancangan: `docs/05-stack.md` §6, KEPUTUSAN-82). Sampai saat itu belum ada kode yang bisa dijalankan.

- Verifikasi penuh (wajib sebelum commit dan sebelum menyatakan selesai): `bash scripts/verify.sh`
- Backend Rust: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `cargo sqlx prepare --check --workspace`
- Migrasi (per database, SQL murni lewat sqlx-cli): `sqlx migrate run --source db/migrations/<app|audit|rme>`; migrasi production hanya lewat skrip deploy dengan kredensial dinamis OpenBao, tidak dari laptop
- Web (per aplikasi di `web/`): `pnpm install --frozen-lockfile`, `pnpm tsc --noEmit`, `pnpm test`, `pnpm build`
- Mobile (di `mobile/`): `dart format --set-exit-if-changed .`, `flutter analyze`, `flutter test`
- Menjalankan lokal: `docker compose -f infra/local/compose.yaml up` (PostgreSQL, Valkey, Keycloak, OpenBao dev, LiveKit, Swagger UI)

Nama perintah final dikunci di reference implementation Tahap 8 Fase 1. Kalau berbeda, bagian ini diperbarui di task yang sama.

## Rujukan

- Coding standards: `docs/coding-standards.md` (dibuat di Tahap 5)
- Stack, hosting, CI/CD, observability, secrets: `docs/05-stack.md`; model data fisik: `docs/05-physical-schema.md`; kode respons API: `docs/05-response-codes.md`; checklist ASVS L2: `docs/05-asvs-checklist.md`
- Keputusan arsitektur: `docs/adr/`
- Reference implementation: <lokasi modul, diisi di Tahap 8 Fase 1>
