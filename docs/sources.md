# Register Sumber

Setiap fakta eksternal di dokumen proyek merujuk ke salah satu entri di sini dengan label `[SRC-xx]`. Aturan lengkap: `.claude/skills/runbook/references/anti-halu.md`.

Sebuah entri hanya boleh ditambahkan setelah halamannya **benar-benar dibuka**, bukan berdasarkan cuplikan hasil pencarian.

| ID | Judul / nama sumber | URL | Tingkat (1/2/3) | Tanggal akses | Mendukung klaim | Dipakai di |
| --- | --- | --- | --- | --- | --- | --- |
| SRC-01 | Google Play Console Help: "Get started with Play Console" | https://support.google.com/googleplay/android-developer/answer/6112435?hl=en | 2 | 2026-11-17 | "There is a US$25 one-time registration fee." Verifikasi identitas "depend[s] on the account type chosen"; "you may be asked for a valid government ID and a credit card, both under your legal name." | 01-business-case.md §6.1, §7.2 (R-08) |

## Pencarian tanpa hasil

| Tanggal | Yang dicari | Hasil | Tindak lanjut |
| --- | --- | --- | --- |
| 2026-11-17 | Biaya tahunan dan syarat pendaftaran organisasi Apple Developer Program (https://developer.apple.com/programs/enroll/, https://developer.apple.com/support/enrollment/) | Halaman gagal dibuka (WebFetch gagal, curl timeout). Tidak ada klaim yang diambil | Ditulis `[TIDAK DIKETAHUI]` di Business Case; diverifikasi ulang di Tahap 5 |
