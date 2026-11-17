# Mulai di Laptop

Starter kit ini berisi runbook 15 tahap sebagai skill Claude Code, dua mode kerja (Terpandu dan Otomatis), lapis anti-halusinasi, sepuluh subagent, dan folder `docs/` yang sudah memuat lima keputusan arsitektur.

## Isi

```text
CLAUDE.md                       aturan proyek, dibaca Claude setiap sesi
.claude/
  settings.json                 izin akses + hook Claude
  hooks/verify-before-stop.sh   menolak "selesai" kalau test merah
  hooks/guard-git.sh            mencegah Claude melewati git hook
.githooks/
  pre-commit, pre-merge-commit  hanya commit dan merge hijau
  skills/runbook/
    SKILL.md                    skill /runbook
    references/anti-halu.md     aturan sumber valid (wajib di setiap tahap)
    references/mode-otomatis.md alur otomatis + 4 checkpoint
    references/aturan-kerja.md  ketelitian berjenjang, kriteria blokir, riset, model, commit+push
    references/tahap-*.md       panduan per tahap
  agents/
    perencana, arsitek, developer                 pelaksana (Mode Otomatis)
    fact-checker                                  pemeriksa sumber
    qa-engineer, dba-reviewer, security-architect,
    security-auditor, compliance-reviewer,
    code-reviewer                                 reviewer independen
docs/
  STATUS.md        posisi setiap tahap
  sources.md       register sumber [SRC-xx]
  decision-log.md  asumsi dan keputusan Claude
  traceability.md  BR → story → test → kode
  backlog.md       backlog, DoR, DoD
  adr/             ADR-001 s.d. ADR-005 (sudah diputuskan)
  input/           taruh dokumen untuk Jalur A di sini
```

## Prasyarat

- Claude Code terpasang. Di macOS/Linux/WSL: `curl -fsSL https://claude.ai/install.sh | bash`. Di Windows PowerShell: `irm https://claude.ai/install.ps1 | iex`. Cek dengan `claude --version`.
- Git terpasang. Di Windows native, pasang Git for Windows supaya skrip hook (bash) bisa berjalan.

## Langkah

1. **Buat repo proyek dari template** (cara utama). Starter ini adalah template repository privat di GitHub (`yuliandryfm/app-runbook-starter`). Buat repo baru yang privat, lalu clone:

    ```bash
    gh repo create <akun>/<nama-proyek> --private --template yuliandryfm/app-runbook-starter --clone
    cd <nama-proyek>
    git config core.hooksPath .githooks
    ```

    Atau lewat browser: buka repo starter, klik **Use this template**, pilih **Private**, lalu clone dan jalankan baris `git config` di atas.

    Cara lama tanpa GitHub (dari arsip zip), bila diperlukan:

    ```bash
    mkdir ~/proyek-saya && cd ~/proyek-saya
    unzip ~/Downloads/app-runbook-starter.zip -d .
    chmod +x .claude/hooks/*.sh .githooks/*
    git init
    git config core.hooksPath .githooks
    git add . && git commit -m "Starter runbook"
    ```

    Perbaikan runbook untuk proyek-proyek berikutnya dilakukan di repo starter ini (commit dan push), bukan di repo proyek.

    Baris `git config core.hooksPath .githooks` **wajib**. Baris inilah yang mengaktifkan aturan "hanya commit hijau". `/runbook` akan menolak berjalan kalau aturan ini belum aktif. Setiap kali clone ulang (misalnya di VPS), jalankan baris ini lagi.

2. **Jalankan Claude Code** di folder itu dan login saat diminta:

    ```bash
    claude
    ```

3. **Pilih model untuk Tahap 0**: `/model` lalu pilih Opus, kemudian `/effort high`.

4. **Mulai**, pilih salah satu:

    ```text
    /runbook auto     Mode Otomatis: berhenti hanya di 4 checkpoint
    /runbook          Mode Terpandu: berhenti di setiap tahap
    ```

    Kedua mode diawali Tahap 0 (Intake) yang interaktif. Untuk Jalur A, taruh dokumenmu di `docs/input/` dulu.

## Mode Otomatis: 4 checkpoint

| Checkpoint | Kapan | Yang kamu lakukan |
| --- | --- | --- |
| 1. Brief | Setelah Tahap 0 | Konfirmasi Project Brief |
| 2. Paket perencanaan | Setelah Tahap 1–7 | Review dokumen, asumsi, dan keputusan sekali jalan di `docs/checkpoint-2.md` |
| 3. UAT | Setelah development, SIT, dan audit | Coba aplikasinya, beri sign-off |
| 4. Go-live | Sebelum deploy | Setujui eksekusi |

Claude juga akan berhenti di luar checkpoint kalau menemukan fakta bisnis yang tidak diketahui, perubahan scope atau ADR, langkah yang tidak bisa dibatalkan, masalah yang gagal diperbaiki berulang, atau batas sesi (20 task / 6 tahap per pemanggilan). Lanjutkan dengan `/runbook auto` lagi.

## Cara memeriksa bahwa tidak ada halusinasi

- Setiap klaim di dokumen berlabel sumber. Klaim berlabel `[SRC-xx]` bisa dicek ke `docs/sources.md`.
- Laporan pemeriksa fakta ada di `docs/fact-check/tahap-XX.md`. Semuanya harus berstatus LULUS.
- Semua dugaan Claude ada di `docs/decision-log.md` dan menunggu konfirmasimu di checkpoint.
- Mulai Tahap 8, Claude tidak bisa mengakhiri giliran saat `scripts/verify.sh` gagal, karena Stop hook akan menolaknya.

## Hanya commit hijau

Tidak ada kode merah yang bisa sampai ke commit, di branch mana pun. Penjaganya berlapis:

| Lapis | Yang dijaga |
| --- | --- |
| `.githooks/pre-commit` | Commit berisi kode ditolak kalau `scripts/verify.sh` merah, kalau verify.sh belum ada, atau kalau ada perubahan kode yang tidak ikut di-stage |
| `.githooks/pre-merge-commit` | Merge ditolak kalau hasil gabungannya merah, meskipun masing-masing branch hijau |
| `.claude/hooks/guard-git.sh` | Claude tidak bisa memakai `--no-verify`/`-n`, mengubah `core.hooksPath`, atau mengubah `.githooks/` dan `scripts/verify.sh` lewat shell |
| `.claude/settings.json` | Perubahan `scripts/verify.sh`, settings, dan hook Claude selalu meminta persetujuanmu |
| Stop hook | Claude tidak bisa mengakhiri giliran saat perubahan kodenya merah |
| CI (dibuat di Tahap 8) | Verifikasi ulang di server. Di GitHub, aktifkan branch protection yang mewajibkan CI hijau |

Commit yang hanya berisi dokumen (`docs/`, `.claude/`, README, CLAUDE.md) tidak perlu verifikasi, karena tidak mengubah kode.

Catatan: kamu sendiri sebagai manusia tetap bisa melewati git hook dengan `--no-verify`. Itu hakmu sebagai pemilik repo. Penjaga ini ditujukan agar Claude tidak bisa melakukannya.

## Tips

- Izin akses di `.claude/settings.json` memblokir pembacaan `.env`, `sudo`, `rm -rf`, dan force push, serta meminta konfirmasi untuk `git push` dan `rm`. Sesuaikan kalau perlu.
- Untuk Tahap 4, kalau akunmu punya akses model Fable, ganti `model: opus` menjadi `model: fable` di `.claude/agents/arsitek.md`.
- Mode Otomatis memakai cukup banyak kuota. Pantau dengan `/usage`.

## Saat pindah ke VPS

Cukup clone repo ini di VPS, lalu jalankan `git config core.hooksPath .githooks`. Skill, subagent, hook, `CLAUDE.md`, dan semua dokumen ikut terbawa, dan `docs/STATUS.md` menunjukkan posisi terakhir.
