# Catatan berjalan Tahap 8 Fase 1

Catatan orkestrator selama Fase 1 (bahan `docs/fase-1-laporan.md` di F1-T71). Status task resmi ada di `docs/backlog.md` dan `docs/07-phase-plan.md` §7.3.

## Log task

| Task | Commit | Review | Catatan |
| --- | --- | --- | --- |
| F1-T01 | — (tanpa kode) | — | Ubuntu 26.04.1, 4 CPU, 15 GB RAM. Dipasang atas persetujuan user (Telegram, 2026-11-18/19): Rust 1.100.0, pnpm 12.21.0 (prefix `~/.local`), sqlx-cli 0.9.1, gitleaks 8.30.1 dan osv-scanner 2.7.1 (checksum rilis resmi cocok), cargo-audit 0.23.0, cargo-deny 0.21.0 |
| F1-T04, F1-T05 | d333524 | Security auditor: putaran 1 TIDAK LULUS (B-1 bypass lewat RUSTFLAGS/RUNNER), putaran 2 LULUS DENGAN CATATAN | Skrip verifikasi disetujui user; izin tetap mengubah sesuai kebutuhan (KEPUTUSAN-241); tanpa `--quick` (KEPUTUSAN-240) |
| F1-T03 | — (spike, dokumen) | — (T2) | `docs/spike/F1-T03-penyimpanan-berkas.md`: SeaweedFS 4.54 emulator lokal/CI (KEPUTUSAN-233), presigned POST lalu PUT (KEPUTUSAN-234), endpoint dari config (KEPUTUSAN-235). Butuh akun NEO uji dari user untuk 11 celah (KEPUTUSAN-236, PT-58) |
| F1-T08 | 4480b1d (merge f6d0a0e) | Code reviewer: LULUS DENGAN CATATAN | 20 test lulus |
| F1-T09 | 9dffc33 (merge ad6133d) | Code reviewer: LULUS | 17 unit + 5 integrasi; registry dua arah terbukti bisa gagal; `ApiError::response_code()` menggantikan ilustrasi `http_and_code()` coding-standards §3.2 |
| F1-T10 | 8e3620f (worktree, menunggu review keamanan) | — | crate `observability`, 22 test; JSON Layer ditulis sendiri (menyimpang dari "fitur json" coding-standards §4.1) |
| F1-T07 | 7852b89 (worktree, menunggu code review) | — | 9 layanan healthy; port 127.0.0.1 tinggi; SeaweedFS bind mount `RS_OBJECT_STORAGE_DIR` |
| F1-T02 | — (spike, dokumen) | Security architect putaran 1: TIDAK LULUS (B-01 klaim galat lupa password, B-02 topologi relay); sedang diperbaiki | Rekomendasi Opsi 1 relay SMTP internal → SES API v2 Jakarta (ADR-026 Diusulkan) |

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
| Review F1-T09 | Pemindai literal `ResponseCode::new` hanya menangkap literal; batasi pembuatan `ResponseCode` ke modul tertentu | F1-T12 |
| Review F1-T09 | Test 404 seragam tautologis; anti-enumerasi harus ditegakkan di test modul pemakai | Task modul pemakai (US-201 dst.) |
| Review F1-T09 | `Success::with_code` diam-diam mengganti kode non-2xx; pertimbangkan `debug_assert` atau tipe kode sukses | F1-T11 |
| Review F1-T09 | Langkah skrip verifikasi "registry-response-code" memeriksa spesifikasi OpenAPI memakai `envelope::registry::parse` | F1-T11 |
| Merge F1-T12 (2026-11-19) | Percobaan pertama commit merge MERAH di langkah `cargo-test` (pre-merge-commit), percobaan kedua tanpa perubahan kode HIJAU; `cargo test --workspace --locked` manual juga hijau. Ada test flaky yang belum teridentifikasi (kemungkinan test HTTP/port atau beban paralel subagent). Test flaky dapat menutupi kegagalan nyata | Diselidiki di task berikutnya (jalankan `cargo test` berulang, catat test yang gagal); bila ketemu, perbaiki atau masukkan Tech Debt |
| Code reviewer | Hook guard-git memblokir subagent menjalankan skrip verifikasi (deteksi keliru "mengubah"); subagent memakai perintah cargo setara | Dilaporkan ke user; tidak diubah (hook dijaga) |

## Arahan user selama Fase 1

| Tanggal | Arahan `[USER]` | Dampak |
| --- | --- | --- |
| 2026-11-19 | "blm ada server storage gunakan server saat ini dengan membuat folder khusus" (Telegram) | Pengembangan dan test memakai SeaweedFS di server pengembangan, data di folder host khusus (`RS_OBJECT_STORAGE_DIR`, default `~/ruang-sehat-data/object-storage`). Target production tetap NEO sesuai ADR-018/`docs/05-stack.md`; verifikasi NEO (11 celah spike F1-T03) menjadi PT-58, wajib sebelum production (EN-26) |
| 2026-11-19 | "kita fokus pada BE kemudian web based. flutter dijalankan terpisah di laptop" (Telegram) | Urutan pengerjaan Claude: backend Rust dulu, lalu web Next.js. Task Flutter (F1-T44, T45, T47, T48, T52, T53, T54, T58, T63, T77, T78) dikerjakan terpisah di laptop (mobile developer, sesuai `[USER] T5-Q7` dan C2-Q3), tidak dijalankan Claude di server ini. Dampak: jalur kritis Fase 1 memuat task mobile (F1-T58, F1-T63; phase plan §7.4) dan demo end-to-end F1-T70 butuh aplikasi pasien, sehingga keduanya bergantung pada kecepatan pengembangan mobile di laptop. Kontrak API (OpenAPI) menjadi antarmuka utama dengan tim mobile |

| 2026-11-19 | Atas usulan percepatan A–D: "semua CR tetap. masuk, jalankan paralel opsinua. tidak mundur" (Telegram) | Tafsiran orkestrator (user dipersilakan mengoreksi): A (bekukan CR) **ditolak**, CR baru tetap diterima lewat backlog; kerja paralel **tetap**; C (test skrip verifikasi penuh hanya di CI) dan D (reviewer gabungan keamanan+DBA untuk putaran pertama task T1) **diterapkan**; B dipakai sempit: hanya task yang mengubah skrip verifikasi/runner yang tidak dijalankan bersamaan; tanggal go-live dan gerbang tidak mundur |

| 2026-11-19 | "tidak ada hari libur. jalan terus" (Telegram) | **Mengubah C2-Q7** (tim libur pada libur nasional dan cuti bersama): libur nasional dan cuti bersama kini hari kerja. Akhir pekan ditanyakan ke user. Phase plan §6.2a dihitung ulang setelah jawaban |

## Posisi saat sesi berhenti (2026-11-19, batas kuota)

- Di-merge ke `tahap-08-fase-1`: F1-T04/05, T07–T15 (+ spike T02, T03). verify.sh 10 langkah hijau.
- Menunggu sebelum merge:
  - F1-T06 (branch `worktree-agent-ae01385c7094662bc`, HEAD 6be5add): perbaikan B-1 dan merge selesai, verify 14 langkah hijau, test-verify 30/30. **Belum dicek ulang security auditor (T1)**. Ada folder `fx-target` dan `dl` di scratchpad yang perlu dihapus user.
  - TD-01(a) + F1-T16 + F1-T17 (branch `worktree-agent-ae5db0b532d091f0e`: 8c7ec79, 4c46e11, 65389a7): verify hijau, 506 pemeriksaan migrasi. **Belum direview DBA + keamanan.**
  - F1-T19 BERHENTI: `consent_record` butuh FK ke `patient`/`care_recipient` (F1-T18), padahal phase plan menulis F1-T19 hanya bergantung F1-T16. Usulan: kerjakan F1-T19 setelah F1-T18.
- CR-021: analisis + review kepatuhan/keamanan LULUS DENGAN CATATAN; keputusan user diajukan (Telegram, 1–4); fact-check `docs/fact-check/CR-021.md` sedang/baru berjalan.
