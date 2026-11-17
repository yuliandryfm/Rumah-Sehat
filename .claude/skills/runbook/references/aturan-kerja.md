# Aturan Kerja: Ketelitian Berjenjang dan Efisiensi

Berlaku di setiap tahap dan di kedua mode, bersama `anti-halu.md`. Aturan ini **tidak melonggarkan** anti-halu: setiap klaim substantif tetap wajib bersumber. Yang diatur di sini adalah seberapa dalam pemeriksaan dilakukan, kapan riset dilakukan, model apa yang dipakai, dan kapan hasil di-commit dan di-push.

Asal aturan: pelajaran proyek sebelumnya, ketika fact-checker terlalu banyak memeriksa hal kosmetik, riset terlalu mendalam, model terlalu besar untuk pekerjaan ringan, dan hasil kerja terlalu lama hanya tersimpan di laptop. Kalau user proyek ini memberi arahan lain, catat di catatan wawancara dan sesuaikan berkas ini lewat change request.

## 1. Ketelitian berjenjang

Orkestrator menetapkan tingkat setiap tahap, bagian, atau task di awal, dan menuliskannya di instruksi subagent. Kalau ragu, naikkan satu tingkat. Daftar modul kritis per proyek ditetapkan di Tahap 4 (arsitektur) dan dicatat di `CLAUDE.md`.

| Tingkat | Cakupan | Perlakuan |
| --- | --- | --- |
| **T1 Kritis** | Alur uang dan saldo, kalkulasi harga/tarif yang diatur, persediaan atau ledger, audit trail, autentikasi/otorisasi/isolasi data antar-tenant, retensi dan penghapusan data, migrasi skema, keamanan, data pribadi, keputusan arsitektur | Review penuh oleh subagent yang relevan (DBA, security, code-reviewer). Test wajib, termasuk skenario negatif. Fact-check atas semua klaim substantif. Maksimal 2 putaran perbaikan, lalu kondisi berhenti |
| **T2 Standar** | Fitur bisnis lain, dokumen perencanaan, ADR non-kritis, design guideline, test plan | Satu reviewer yang relevan ditambah fact-check substantif. Catatan non-blokir dicatat dan tidak memicu putaran review baru |
| **T3 Ringan** | Teks UI, format, merapikan dokumen, catatan perubahan, penomoran register, polesan visual wireframe | Cek mandiri oleh pelaksana ditambah pemeriksaan mesin (lint, `scripts/verify.sh` bila ada). Tanpa putaran review khusus |

## 2. Kriteria blokir fact-checker

Fact-checker dan reviewer hanya boleh memberi status **blokir** untuk:

1. Fakta bisnis tanpa `[USER]`/`[DOK]` yang sah, atau fakta bisnis yang berlabel `[ASUMSI]`/`[KEPUTUSAN]`.
2. Fakta eksternal (versi, API, regulasi, kemampuan teknis) yang salah, tanpa `[SRC]` dari sumber resmi, atau sumbernya tidak mendukung klaim.
3. Kontradiksi dengan dokumen yang sudah lulus gate, bila kontradiksi itu mengubah keputusan, perilaku, angka, peran, atau istilah.
4. Keputusan bisnis yang diambil penyusun tanpa dibawa ke user atau checkpoint.
5. Nomor `SRC`/`KEPUTUSAN`/`ASUMSI` yang dirujuk tetapi tidak terdaftar, atau klaim "teruji/terbukti" yang melebihi bukti eksekusi.

**Non-blokir** (dicatat singkat, paling banyak 5 butir terpenting, dan tidak memicu putaran ulang): rumusan kalimat, format, catatan perubahan, rentang penomoran, kolom "dipakai di", label yang kurang presisi tetapi tidak mengubah makna, dan rujukan silang kosmetik.

Cek ulang cukup memeriksa baris yang berubah untuk menutup blokir, tidak mengulang pemeriksaan penuh.

## 3. Riset secukupnya dan tepat waktu

- Riset hanya untuk pertanyaan yang sedang menentukan keputusan atau pekerjaan saat itu (*just-in-time*), bukan survei menyeluruh di muka.
- Cukup sumber primer resmi yang menjawab pertanyaan, lalu berhenti. Umumnya 1–3 halaman per pertanyaan; rujukan desain visual sekitar 5–8 halaman.
- Versi, API, dan kemampuan pustaka diverifikasi saat dipakai (misalnya saat menambah dependensi di Tahap 8), bukan semuanya di depan.
- Hasil dicatat ringkas di `docs/sources.md`. Laporan riset panjang hanya dibuat bila user memintanya.

## 4. Pemilihan model per kelas task

Default per tahap mengikuti tabel "Daftar tahap" di `SKILL.md`. Di dalam tahap, pilih per kelas task dan jangan berlebihan. Model dan effort ditulis di parameter `model`/`effort` pemanggilan subagent pelaksana (satu subagent seperti `perencana` dipakai banyak tahap dengan effort berbeda, jadi frontmatter-nya saja tidak cukup).

| Kelas task | Model | Effort |
| --- | --- | --- |
| Keputusan arsitektur, model data, modul T1, review keamanan/DBA, consistency review Tahap 8 | Opus | high |
| Perencanaan (PRD, phase plan), desain visual yang dinilai user (FE) | Opus | medium–high |
| Fitur T2, dokumen standar, design guideline, test plan, fact-check substantif | Sonnet | medium |
| Pekerjaan mekanis T3: format, rename, merapikan, meringkas, pemetaan | Sonnet (low) atau Haiku | low |

Reviewer independen memakai frontmatter masing-masing, kecuali tingkat task menuntut lain (misalnya fact-check dokumen T2 cukup Sonnet).

## 5. Commit dan push per tahap dan per task

- **Tahap selesai** (gate lolos, termasuk status `selesai (menunggu checkpoint N)`): commit, merge ke `main`, lalu push `main` tanpa menunggu checkpoint.
- **Pekerjaan tahap yang panjang**: commit dan push branch tahap per bagian yang selesai.
- **Tahap 8**: satu commit dan push per task, setelah `scripts/verify.sh` hijau dan code review lolos.
- Tetap hanya commit hijau. Tidak ada `--no-verify`, tidak mengubah hook.
- Push ke GitHub tanpa mengubah konfigurasi git global: `GIT_TERMINAL_PROMPT=0 git -c credential.helper= -c 'credential.helper=!gh auth git-credential' push`. `GIT_TERMINAL_PROMPT=0` mencegah perintah menggantung menunggu kata sandi. Repositori sebaiknya privat sebelum push pertama.
