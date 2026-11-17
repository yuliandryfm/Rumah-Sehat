# Rumah Sehat

Aplikasi mobile kesehatan (Flutter, Android dan iOS) untuk pasien yang kesulitan datang ke klinik atau apotek. Rilis pertama berisi tujuh layanan: konsultasi online dokter dan psikolog (chat dan video, berlangsung di Rumah Sehat), pendaftaran klinik dan lab rekanan, home visit tenaga medis, informasi biaya MCU dan lab, serta pembelian obat bebas, produk kecantikan, dan produk kesehatan dari outlet rekanan. Rumah Sehat berperan sebagai gateway: data klinik, lab, home visit, dan produk berasal dari integrasi dengan sistem rekanan. Tenaga kesehatan memakai aplikasi mobile, sedangkan admin klinik, lab, outlet, dan Rumah Sehat memakai portal web. Data wajib disimpan di Indonesia. Target go-live 1 Maret 2027. Rincian dan pertanyaan terbuka: `docs/00-project-brief.md`.

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

## Perintah

<Diisi di Tahap 5: build, lint, test, migrasi, menjalankan aplikasi lokal.>

## Rujukan

- Coding standards: `docs/coding-standards.md` (dibuat di Tahap 5)
- Keputusan arsitektur: `docs/adr/`
- Reference implementation: <lokasi modul, diisi di Tahap 8 Fase 1>
