# Catatan berjalan Tahap 8 Fase 1

Catatan orkestrator selama Fase 1 (bahan `docs/fase-1-laporan.md` di F1-T71). Status task resmi ada di `docs/backlog.md` dan `docs/07-phase-plan.md` §7.3.

## Log task

| Task | Commit | Review | Catatan |
| --- | --- | --- | --- |
| F1-T01 | — (tanpa kode) | — | Ubuntu 26.04.1, 4 CPU, 15 GB RAM. Dipasang atas persetujuan user (Telegram, 2026-11-18/19): Rust 1.100.0, pnpm 12.21.0 (prefix `~/.local`), sqlx-cli 0.9.1, gitleaks 8.30.1 dan osv-scanner 2.7.1 (checksum rilis resmi cocok), cargo-audit 0.23.0, cargo-deny 0.21.0 |
| F1-T04, F1-T05 | d333524 | Security auditor: putaran 1 TIDAK LULUS (B-1 bypass lewat RUSTFLAGS/RUNNER), putaran 2 LULUS DENGAN CATATAN | Skrip verifikasi disetujui user; izin tetap mengubah sesuai kebutuhan (KEPUTUSAN-241); tanpa `--quick` (KEPUTUSAN-240) |
| F1-T03 | — (spike, dokumen) | — (T2) | `docs/spike/F1-T03-penyimpanan-berkas.md`: SeaweedFS 4.54 emulator lokal/CI (KEPUTUSAN-233), presigned POST lalu PUT (KEPUTUSAN-234), endpoint dari config (KEPUTUSAN-235). Butuh akun NEO uji dari user untuk 11 celah (KEPUTUSAN-236, PT-58) |
| F1-T08 | 4480b1d (branch worktree, belum di-merge) | Code reviewer: LULUS DENGAN CATATAN | 20 test lulus |

## Catatan non-blokir yang dibawa ke task berikutnya

| Asal | Catatan | Dibawa ke |
| --- | --- | --- |
| Review F1-T08 | `clippy.toml` dan `[lints.clippy]` baru berlaku untuk crate `foundation`; pindahkan ke root workspace (`clippy.toml` root + `[workspace.lints]`) | F1-T12 |
| Review F1-T08 | Tambah `OffsetDateTime::now_local` ke `disallowed-methods` (ADR-017 UTC saja); `Instant::now` tetap boleh untuk durasi | F1-T12 |
| Review F1-T08 | `new_id()` memakai jam sistem lewat `Uuid::now_v7`, sehingga ID tidak deterministik di test | Dicatat; ditinjau bila ada test yang butuh ID deterministik |
| Review F1-T08 | `Sensitive<T>` menurunkan `Clone`/`Default`, tanpa zeroize | Ditinjau di task enkripsi (EN-09) |
| Review F1-T08 | SRC-326 menyebut menyelesaikan sebagian coding-standards §3.4/§5.7; coding-standards belum diperbarui | F1-T12 |
| Review F1-T05 | Belum ada test untuk skrip verifikasi itu sendiri (fixture merah harus MERAH) | F1-T06 |
| Review F1-T05 | Konfigurasi cargo di luar repo hanya tertutup CI di lingkungan bersih | F1-T06 |
| Code reviewer | Hook guard-git memblokir subagent menjalankan skrip verifikasi (deteksi keliru "mengubah"); subagent memakai perintah cargo setara | Dilaporkan ke user; tidak diubah (hook dijaga) |
