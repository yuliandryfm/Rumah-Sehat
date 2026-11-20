# Phase Plan: Ruang Sehat

Tanggal: 2026-11-20
Versi: 0.4 (kerja 7 hari per minggu dan CR-022 `[USER]` 2026-11-19, progres Fase 1 per 2026-11-20; riwayat di §14). Versi 0.3: CR-021 skenario S3 `[USER] C21-P4`. Versi 0.2: tindak lanjut Checkpoint 2
Penyusun: Claude (persona Project Manager, dengan tinjauan Tech Lead atas urutan dan dependensi task), Mode Otomatis
Pemilik dan pemutus: user (project manager, meneruskan ke CEO dan CTO) `[DOK: 00-project-brief.md]`
Dasar: Business Case v1.0 `[DOK: 01-business-case.md]`, BRD v1.3 `[DOK: 02-brd.md]`, PRD v1.1 (+ revisi CR-013 di v1.2 draf; revisi AC Checkpoint 2 dikerjakan paralel) `[DOK: 03-prd.md]`, arsitektur v1.0 dan ADR-006–ADR-025 (ADR-011 direvisi untuk CR-018, dikerjakan paralel) `[DOK: 04-arsitektur.md]`, stack v1.0 `[DOK: 05-stack.md]`, coding standards `[DOK: coding-standards.md]`, design guideline v0.1 `[DOK: 06-design-guideline.md]`, backlog `[DOK: backlog.md]`, risk register `[DOK: 04-risk-register.md]`, klarifikasi AC Fase 1 v0.2 `[DOK: 07-klarifikasi-ac-fase-1.md]`, jawaban Checkpoint 2 `[DOK: checkpoint-2.md]`
Tingkat ketelitian: T2 Standar. Urutan, dependensi, dan definisi task modul kritis T1 (daftar di `CLAUDE.md`) ditinjau teliti.

**Label sumber.** `[DOK: ...]` dokumen proyek yang lulus gate. `[USER] Tx-Qy` jawaban user yang tercatat di catatan tahap; `[USER] C2-Qy` dan `[USER] C2-Bn` (butir n) jawaban Checkpoint 2 di `docs/checkpoint-2.md` bagian "Jawaban user". `[KEPUTUSAN-xx]`, `[ASUMSI-xx]` tercatat di `docs/decision-log.md`. `[UP-xx]` usulan penyusun yang diputuskan user di Checkpoint 2 (status di §12 dan `docs/07-catatan-phase-plan.md`). `[SRC-xx]` tercatat di `docs/sources.md`. `[TIDAK DIKETAHUI]` masuk pertanyaan terbuka (§9, §12). Angka jam-task di dokumen ini adalah **estimasi**, bukan fakta, dan dasar hitungnya ditulis di sampingnya.

**Status v0.2.** Kalender fase (UP-65), gerbang re-plan (UP-66), dan Opsi B sebagai baseline (UP-67) disetujui di Checkpoint 2 `[USER] C2-Q1, C2-Q5, C2-Q6`. Opsi scope final tetap diputuskan di gerbang re-plan 2026-12-11. Angka effort dan kebutuhan kecepatan tetap estimasi.

**Status v0.3 (CR-021).** User memilih skenario **S3** `[USER] C21-P4` (`docs/cr/CR-021-wawancara.md` putaran 4; analisis `docs/cr/CR-021-analisis.md` §9):
- Fase 1 tetap selesai 2026-12-11 dan memuat situs web, CMS (US-1101 dipindah dari Fase 3), pendaftaran dan verifikasi perusahaan, serta akun HR: 31 task baru F1-T81–F1-T111 (109 jam-task).
- Tujuh task Fase 1 pindah ke Fase 2: F1-T57, F1-T67, F1-T75–F1-T79 (25 jam-task).
- Reset MFA HR oleh Verifikator (K1-F2, ±12 jam-task) masuk Fase 2.
- Pemesanan, peserta, dan invoice korporat (K2, ±195) serta ringkasan, hasil, dan penerusan ke rekanan (K3, ±36) masuk Fase 3.
- Kebutuhan baru (§6.7): Fase 1 sisa ±19,8 jam-task per hari kerja (backend/web ±17,4), Fase 2 ±45,3, Fase 3 Opsi B ±64,9. Effort Opsi B naik dari ±1.791 ke ±2.131 jam-task. Keputusan kompensasi di gerbang re-plan 2026-12-11 (PT-69).

**Status v0.4 (kerja 7 hari per minggu, CR-022, progres per 2026-11-20).** Rincian hitungan di §6.8. Angka "hari kerja" dan "jam-task per hari kerja" di bagian lain yang berlabel v0.2 atau v0.3 adalah angka historis lima hari kerja dan digantikan §6.8 bila bertentangan.
- **Kerja 7 hari per minggu** `[USER]` 2026-11-19 ("tidak ada hari libur. jalan terus"; akhir pekan dijawab **Y**; `docs/fase-1-catatan.md` "Arahan user"). Libur nasional, cuti bersama, dan akhir pekan menjadi hari kerja. Ini **mengubah C2-Q7**. Pembagi kebutuhan kini **hari kalender**: Fase 1 19 (sisa dari 2026-11-20: 22), Fase 2 26, Fase 3 19, Februari 12.
- **CR-022** `[USER]` 2026-11-19 (`docs/backlog.md` CR-022): urutan Fase 1 = task baseline yang tetap di Fase 1 dulu, lalu F1-T81–F1-T111 (CR-021); go-live **boleh mundur** bila tidak muat (risiko diterima user); gerbang 2026-12-11 menetapkan **tanggal go-live baru** dari kecepatan terukur dan keputusan tambahan tenaga (PT-53), **bukan** memilih opsi pemotongan scope. Cakupan rilis tidak diperluas: baseline Opsi B + CR-021; CR-011 dan CR-015 tetap rilis berikutnya (tafsiran tercatat di `docs/backlog.md` CR-022, lihat kontradiksi di §6.8.6).
- **Progres Fase 1 per 2026-11-20:** 73 dari 361 jam-task (20,2%), 22 dari 104 task selesai atau ter-merge; sisa 288 jam-task (§6.8.3).
- **Kebutuhan baru (Opsi B + CR-021):** Fase 1 sisa ±13,1 jam-task per hari kalender (backend/web ±11,4; mobile ±1,7), Fase 2 ±27,9, Fase 3 ±51,2, Februari ±6,0. Effort total tidak berubah (±2.131). Kenaikan kapasitas riil bergantung pada jam kerja per minggu `[TIDAK DIKETAHUI]` (§6.8.2).

---

## 1. Ringkasan

- **Kalender disetujui** `[USER] C2-Q5`: fase pengembangan (Tahap 8) dalam jendela 2026-11-23 sampai 2027-01-29, lalu QA/SIT, audit keamanan, UAT, persiapan rilis, dan deploy sampai go-live 1 Maret 2027 `[UP-65]`:
  - **Fase 1** (2026-11-23 – 2026-12-11, 15 hari kerja; v0.4: 19 hari kalender; Fase 1 sebenarnya mulai 2026-11-18): fondasi (repo, `scripts/verify.sh`, CI, skema inti, autentikasi + RBAC, Audit Trail Foundation, enkripsi, berkas, BFF, kerangka mobile), reference implementation, dan MVP tertipis "pasien terverifikasi" (daftar, persetujuan, **MFA TOTP wajib**, masuk, profil, unggah KTP, diverifikasi Verifikator di backoffice). Ditambah MFA wajib pasien (CR-018) dan ubah password saat login (CR-017) `[USER] C2-Q9, C2-B14`.
  - **Fase 2** (2026-12-14 – 2027-01-08, v0.2: 16 hari kerja setelah libur dikurangi; **v0.4: 26 hari kalender**): konsultasi online dokter dan psikolog end-to-end, termasuk pembayaran, refund otomatis, notifikasi, aplikasi tenaga kesehatan, staging, observability, dan **US-110 "Perangkat dan sesi aktif"** di semua aplikasi `[USER] C2-Q10`.
  - **Fase 3** (2027-01-11 – 2027-01-29, 15 hari kerja; v0.4: 19 hari kalender): Partner Integration API, RME minimalis, pendaftaran klinik/lab, informasi biaya, hasil lab, klaim, rekonsiliasi, portal rekanan, CMS dan keluhan, halaman audit log, hak subjek data, KPI, dan anggota keluarga (US-108, US-109, dipindah dari Fase 4 di Opsi B).
  - **Fase 4 tidak dikerjakan di baseline Opsi B** `[USER] C2-Q1`. Isinya (antrean real-time, pembelian produk, home visit) hanya dikerjakan bila gerbang re-plan 2026-12-11 memilih Opsi A `[UP-66]`.
- **(v0.2, diganti v0.4) Tim libur pada libur nasional dan cuti bersama** `[USER] C2-Q7`. **Sejak v0.4 tim bekerja 7 hari per minggu, termasuk libur nasional dan cuti bersama** `[USER]` 2026-11-19; tanggal SKB `[SRC-280]` `[SRC-282]` `[SRC-283]` hanya dicatat sebagai informasi (24–25 Desember 2026, 1 dan 5 Januari 2027, 5 Februari 2027; §6.2a).
- **Effort total estimasi Opsi A ±1.967 jam-task (±643 task)**, ditambah ±42 jam-task yang masih terblokir PT-24. **Baseline Opsi B ±1.791 jam-task** (§6) `[KEPUTUSAN-178]` `[KEPUTUSAN-216]`.
- **Fase 1: 80 task, 277 jam-task** (naik dari 71 task, 245 jam-task karena F1-T72–F1-T80). Jalur kritis tetap **65 jam-task** (§7.4).
- **Kebutuhan kecepatan (v0.2, lima hari kerja; diganti §6.8 di v0.4)**: ±18,5 jam-task per hari kerja (Fase 1), ±43,0 (Fase 2), ±50,3 di Fase 3 untuk Opsi B (±62,0 untuk Opsi A).
- **Kesimpulan kelayakan tidak berubah**: belum dapat dinyatakan layak, dan indikasinya kuat tidak layak untuk Opsi A. Kapasitas yang tercatat sekarang: mobile developer penuh waktu ±40 jam/minggu yang mengerjakan seluruh layar mobile `[USER] C2-Q3`, sesi Claude ≥ 6 jam per hari kerja yang dijalankan PM `[USER] C2-Q4`, dan developer backend/web tambahan mulai awal Januari 2027 (Opsi E) `[USER] C2-Q2, C2-Q8`. Jumlah orang dan biaya Opsi E `[TIDAK DIKETAHUI]` (PT-53). Throughput dalam jam-task tetap `[TIDAK DIKETAHUI]` dan diukur di Fase 1, terpisah untuk jalur mobile dan jalur backend/web (§6.5) `[KEPUTUSAN-221]`.
- **CR-011** (onboarding mandiri dan langganan RME) dan **CR-015** (vaksinasi/infus) **diputuskan ke rilis berikutnya** `[USER] C2-B5, C2-B6` (§11).
- **CR-021 (v0.3)**: situs web company profile dan layanan korporat, skenario S3 `[USER] C21-P4`. Fase 1 menjadi 104 task dan 361 jam-task (80 − 7 + 31 task; 277 − 25 + 109 jam-task). Rincian di §6.7, §7.3, §8.1, §8.2, §9.
- **Temuan teknis yang perlu ADR baru di awal Fase 1** (tetap): Keycloak mengirim email (verifikasi email, atur ulang password, undangan) lewat SMTP `[SRC-264]`, sedangkan endpoint SMTP Amazon SES tidak tersedia di region Jakarta `[SRC-84]`. Jalur email dirancang ulang lewat spike F1-T02 dan diusulkan sebagai ADR ke user `[KEPUTUSAN-183]`.

---

## 2. Dasar perencanaan

| Butir | Isi | Sumber |
| --- | --- | --- |
| Go-live | Target semula 1 Maret 2027 (toleransi waktu 0 hari). **v0.4: toleransi waktu 0 diubah CR-022; go-live boleh mundur**, tanggal baru ditetapkan di gerbang 2026-12-11 (§6.8.5) | `[DOK: 01-business-case.md]` §6.2, §7.1; `[USER]` CR-022 (`docs/backlog.md`) |
| Bila syarat go-live belum terpenuhi | (v0.2) Tanggal tetap, scope dikurangi. **v0.4: scope penuh (Opsi B + CR-021) diutamakan, tanggal mundur**; syarat izin dan kepatuhan, audit keamanan, dan UAT tetap tidak bisa dikurangi | `[DOK: 00-project-brief.md]` P8.3 (diubah CR-022); `[USER]` CR-022 |
| Urutan pengurangan scope (**tidak dipakai sebagai alat penjadwalan sejak CR-022**; dicatat sebagai riwayat) | (1) antrean real-time; (2) obat bebas, lalu seluruh produk; (3) home visit; (4) konsultasi fisioterapis dan tenaga medis; (5) pendaftaran lab, informasi biaya MCU/lab, hasil lab; (6) pendaftaran klinik. Konsultasi dokter dan psikolog dipertahankan paling akhir. Fitur RME minimalis ikut layanan pemakainya | `[DOK: 03-prd.md]` §2.3 (`[USER] T3-Q2`, UP-33) |
| Kelayakan scope | Dinilai di Tahap 7 dengan estimasi effort (G4) | `[DOK: 03-prd.md]` §9 (`[USER] T3-Gate`) |
| Tim | Project manager (user), Claude lewat runbook, 1 mobile developer; ditambah developer backend/web mulai awal Januari 2027 (Opsi E) | `[DOK: 00-project-brief.md]` P7.3; `[USER] C2-Q2, C2-Q8` |
| Status developer | Penuh waktu sampai go-live; setelah go-live PT-35 | `[USER] T5-Q7` |
| Biaya tim selain developer | Di luar anggaran proyek | `[USER] T5-Q9` |
| Mobile developer | Penuh waktu ±40 jam/minggu; mengerjakan seluruh layar mobile | `[USER] C2-Q3` |
| Sesi Claude | ≥ 6 jam per hari kerja, dijalankan PM; throughput tetap diukur di Fase 1 | `[USER] C2-Q4` |
| Developer backend/web tambahan | Mulai awal Januari 2027 (membantu mulai Fase 3). Jumlah orang, biaya terhadap toleransi +20%, dan tanggal mulai pasti `[TIDAK DIKETAHUI]` | `[USER] C2-Q2, C2-Q8`; PT-53 |
| Hari kerja | (v0.2) Tim libur pada libur nasional dan cuti bersama. **v0.4: 7 hari per minggu, libur nasional, cuti bersama, dan akhir pekan menjadi hari kerja** (mengubah C2-Q7). Jam kerja per hari dan per minggu `[TIDAK DIKETAHUI]` | `[USER]` 2026-11-19 (`docs/fase-1-catatan.md`); `[USER] C2-Q7` (digantikan); `[SRC-280]` `[SRC-282]` `[SRC-283]` (tanggal, informasi) |
| Opsi scope | Opsi B sebagai baseline; final di gerbang re-plan 2026-12-11 | `[USER] C2-Q1` |
| Lingkungan | Staging Biznet Gio mulai Desember 2026; uji beban Januari 2027 (satu bulan); production mulai 1 Februari 2027 | `[DOK: 05-stack.md]` §2.9, §11.2 |
| Siklus per fase | Setiap fase ditutup dengan functional test QA, consistency review, dan laporan fase | `[DOK: .claude/skills/runbook/references/tahap-08-development.md]` |
| Scope rilis 1 | 97 story aktif (92 Must, termasuk US-110 dari Checkpoint 2, 5 Should: US-108, US-109, US-405, US-602, US-1305), US-102 Won't | `[DOK: 03-prd.md]` §5, `[DOK: backlog.md]` |
| Mesin pengembangan saat ini | v0.1 mencatat Darwin 21.6.0; sesi revisi v0.2 (2026-11-18) berjalan di Linux 7.0.0-38-generic (informasi lingkungan sesi kerja). Mesin yang dipakai di Tahap 8 dan kesesuaian toolchain stack dengannya `[TIDAK DIKETAHUI]`, diperiksa di F1-T01 | `[DOK: .claude/skills/runbook/references/tahap-08-development.md]` langkah 0 |

---

## 3. Prinsip pembagian fase

1. **Fase 1 hanya Must** dan memuat semua enabler fondasi: skema inti, autentikasi dan RBAC, Audit Trail Foundation, CI pipeline, dan reference implementation `[DOK: .claude/skills/runbook/references/tahap-07-phase-plan.md]`.
2. **MVP Fase 1 adalah slice "pasien terverifikasi"**, bukan konsultasi berbayar. Alasannya teknis: slice ini menembus semua lapis (aplikasi pasien → API → database → audit → backoffice) dan semua fondasi T1 (autentikasi dua realm, otorisasi, audit, enkripsi NIK, berkas identitas) tanpa bergantung pada penyedia yang konfirmasinya belum ada (DOKU PT-36, Lalamove PT-37, Biznet Gio PT-39) `[KEPUTUSAN-180]`.
3. **Urutan layanan mengikuti kebalikan urutan pengurangan scope.** Konsultasi dokter dan psikolog (dipertahankan paling akhir `[USER] T3-Q2`) dibangun pertama setelah fondasi; antrean, produk, dan home visit (dilepas pertama) dibangun terakhir. Bila kapasitas kurang, pengurangan scope berarti melepas fase terakhir, bukan memotong fitur yang sudah setengah jadi `[UP-66]`.
4. **Skema dan autentikasi sebelum fitur; pustaka T1 dibuat sekali** di Fase 1 dan dipakai ulang (RT-10 mitigasi 3) `[DOK: 04-risk-register.md]`.
5. **Gerbang re-plan di akhir Fase 1**: kecepatan terukur (task selesai per hari kalender, dengan `scripts/verify.sh` hijau dan review lolos) dibandingkan dengan kebutuhan Fase 2–3 (§6.5), lalu `[UP-66]` (disetujui `[USER] C2-Q6`) memilih opsi scope. **v0.4 (CR-022):** gerbang tidak lagi memilih opsi pemotongan scope; gerbang **menetapkan tanggal go-live baru** dari kecepatan terukur dan keputusan tambahan tenaga (PT-53) (§6.8.5) `[USER]` CR-022. Ini juga menjawab RT-24 (kecepatan Rust diukur di reference implementation) `[DOK: 04-risk-register.md]`.
6a. **Urutan Fase 1 (v0.4, CR-022)**: task baseline yang tetap di Fase 1 dikerjakan lebih dulu, lalu F1-T81–F1-T111 (CR-021), dan F1-T71 (penutupan, bergantung pada F1-T110) paling akhir (§7.3a) `[USER]` CR-022.
6. **Story boleh selesai bertahap per acceptance criteria** bila satu AC bergantung pada story di fase berikutnya. AC yang ditunda ditulis eksplisit di backlog dan diverifikasi di fase tujuan `[KEPUTUSAN-179]`.
7. **Dependensi eksternal menjadi gerbang bertanggal** (§9). Task yang bergantung pada gerbang yang belum terbuka tidak dimulai; urutan task lain di fase yang sama disusun ulang.

---

## 4. Fase dan deliverable

Tanggal fase disetujui `[USER] C2-Q5` `[UP-65]`. Sejak v0.4 hitungan memakai hari kalender karena tim bekerja 7 hari per minggu (§6.2a, §6.8). Tanggal fase tidak diubah v0.4; tanggal akhir Fase 2–3 dan go-live dapat bergeser lewat gerbang 2026-12-11 (CR-022). Akhir pekan di antara fase (2026-12-12/13, 2027-01-09/10) tidak dialokasikan ke fase mana pun dan menjadi penyangga.

| Fase | Tanggal | Isi utama | Deliverable (bukti di akhir fase) | Gerbang masuk |
| --- | --- | --- | --- | --- |
| 1. Fondasi + MVP "pasien terverifikasi" | 2026-11-23 – 2026-12-11 (v0.4: 19 hari kalender; mulai nyata 2026-11-18; v0.3: 15 hari kerja) | EN-01–EN-14, UI-01–UI-03; US-101, US-103, US-104 (kecuali AC4), US-105, US-106, US-201 (kecuali notifikasi di AC2), US-208, US-1501, US-1504; CR-018 MFA wajib pasien (termasuk daftar ulang TOTP realm `pasien` dan `admin`); CR-017 ubah password saat login (realm `pasien` dan `admin`). **v0.3 (CR-021):** EP-18 (US-1801, US-1802, US-1101 dipindah dari Fase 3), US-1901–US-1904, US-1916 bagian K1, F1-T81–F1-T111; F1-T57, F1-T67, F1-T75–F1-T79 pindah ke Fase 2 (US-103 AC6–AC7, US-105 AC4, layar US-208 menyusul) | `scripts/verify.sh` dan CI hijau; stack lokal; skema DB aplikasi dan audit dengan test hak role; realm `pasien` dan `admin`, keduanya dengan MFA TOTP wajib dan kode pemulihan; RBAC default deny; audit outbox → relay → rantai hash → verifikasi; reference implementation tercatat di `CLAUDE.md`; v0.3: situs tampil dari CMS, perusahaan daftar → Verifikator menyetujui → HR masuk dengan TOTP → undang HR kedua, isolasi dua perusahaan teruji (F1-T110); demo end-to-end: pasien daftar → konfigurasi TOTP → masuk dengan OTP → unggah KTP → Verifikator menyetujui → status Disetujui, rantai audit terverifikasi; laporan kecepatan per jalur dan cakupan baseline; `docs/fase-1-laporan.md` | Checkpoint 2 disetujui; persetujuan user atas `scripts/verify.sh` (F1-T05) |
| Gerbang re-plan | 2026-12-11 | Kecepatan terukur dibandingkan kebutuhan Fase 2–3 (§6.5, §6.8) | **v0.4 (CR-022):** keputusan user: **tanggal go-live baru** dan rincian tambahan tenaga (PT-53); scope tetap baseline Opsi B + CR-021 (v0.3: memilih opsi scope final) | Fase 1 selesai atau data yang ada pada 2026-12-11 |
| 2. Konsultasi online + pembayaran | 2026-12-14 – 2027-01-08 (v0.4: **26 hari kalender**; v0.3: 16 hari kerja = 20 dikurangi 24 dan 25 Desember 2026, 1 dan 5 Januari 2027) | EN-15–EN-20, EN-24, UI-04; US-107, US-202–US-207, US-301–US-311, US-801–US-806, US-808, US-809, US-812, US-1001–US-1003, US-1406 (pembayaran, video, email), US-104 AC4, US-201 AC2 (notifikasi); **US-110** perangkat dan sesi aktif; reset MFA pasien oleh Layanan Pelanggan (US-207 AC5, CR-018); CR-017 dan daftar ulang TOTP realm `nakes`; notifikasi pemberian sub-peran sensitif (UP-76). **v0.3 (CR-021):** F1-T57, F1-T67, F1-T75–F1-T79 dari Fase 1 (CR-017 dan daftar ulang TOTP realm `pasien`/`admin`, ubah email, layar kelola admin); reset MFA HR oleh Verifikator (US-1904 AC7–AC9, K1-F2) | Pasien memesan dan membayar sesi (sandbox DOKU), chat dan video, catatan, resep + halaman verifikasi publik, riwayat; refund otomatis dan eksekusinya; aplikasi tenaga kesehatan dengan MFA dan kunci 15 menit; onboarding tenaga kesehatan dan klinik; markup dan parameter; notifikasi email dan di dalam aplikasi; layar perangkat dan sesi aktif di aplikasi pasien, aplikasi tenaga kesehatan, backoffice, dan portal; staging dengan deploy otomatis; observability; perubahan data dictionary pajak (CR-013) | Gerbang re-plan; PT-39 (staging); PT-47; PT-36 sebelum task pembayaran (target 2026-12-21); sisa PT-28 sebelum perhitungan pajak |
| 3. Integrasi rekanan, RME, klinik/lab, kepatuhan | 2027-01-11 – 2027-01-29 (v0.4: 19 hari kalender; v0.3: 15 hari kerja) | EN-21–EN-23, EN-25, UI-05; US-401–US-404, US-406, US-601, US-602, US-807, US-810, US-811, US-901, US-902, US-1101–US-1104, US-1201–US-1203, US-1301–US-1304, US-1306, US-1307, US-1309, US-1310, US-1401–US-1405, US-1502, US-1503, US-1601–US-1606, US-1701; US-108, US-109 (Opsi B); CR-017, daftar ulang TOTP, dan US-110 untuk realm `rme` (web RME). **v0.3 (CR-021):** US-1101 keluar (pindah ke Fase 1); K2 US-1905–US-1913 + jurnal korporat + email korporat; K3 US-1914, US-1915, US-1917 | Partner Integration API v1 + sandbox; v0.3: pesanan korporat → invoice → mutasi bank dan alokasi → lunas → jadwal → penerusan ke rekanan → hasil ke peserta; RME minimalis lulus skenario kontrak yang sama; pendaftaran klinik/lab dibayar di aplikasi dan check-in lewat sistem rekanan/RME; hasil lab; pengiriman SATUSEHAT (sandbox, bila PT-32 tersedia); klaim dan rekonsiliasi; portal rekanan; CMS dan keluhan; halaman audit log dan ekspor; hak subjek data; dasbor KPI; anggota keluarga; uji beban NFR-CAP-04 | PT-05 (rekanan dan kemampuan sistemnya); PT-32 untuk uji US-1309; PT-24 untuk US-1605/US-1606; v0.3: G-25 (PT-59, PT-62–PT-66, L-21) dan G-26 (L-17–L-19, DPIA) untuk K2/K3 |
| 4. Layanan yang dilepas pertama | **Tidak dikerjakan di baseline Opsi B** `[USER] C2-Q1`. Bila gerbang re-plan memilih A: 2027-01-18 – 2027-01-29 (10 hari kerja), paralel dengan Fase 3 | US-405, US-1305, US-501–US-504, US-1308, US-701–US-706, US-1406 (kurir); US-108 dan US-109 hanya bila Opsi A (di Opsi B sudah di Fase 3) | Antrean real-time; home visit (penugasan dari sistem klinik/RME); pembelian produk dengan kurir Lalamove, pelacakan, klaim barang | Gerbang re-plan memilih Opsi A; PT-29, PT-30, PT-33, PT-37 terjawab |
| Pembekuan fitur | 2027-01-29 | Tidak ada fitur baru setelah tanggal ini; hanya perbaikan bug | — | — |

Story Should (US-108, US-109, US-405, US-602, US-1305) tidak ada di Fase 1. Penempatan per story ada di `docs/backlog.md` dan `docs/traceability.md` §"Story → fase".

---

## 5. Kalender sampai go-live (disetujui `[USER] C2-Q5`)

| Jendela | Tanggal | Hari kalender kerja tim (v0.4, 7 hari per minggu; v0.2 dalam kurung) | Isi | Butir `[TIDAK DIKETAHUI]` | Sumber |
| --- | --- | --- | --- | --- | --- |
| Checkpoint 2 | sebelum 2026-11-23 | — | Persetujuan design guideline, phase plan, dan opsi | — | `[UP-65]` |
| Tahap 8 Fase 1–3 (Fase 4 hanya bila Opsi A) | 2026-11-23 – 2027-01-29 | 64 (19 + 26 + 19; v0.2: 46 = 15 + 16 + 15) | §4 | Throughput tim per jalur (PT-48 sisa); Opsi E (PT-53) | `[UP-65]` |
| Staging | mulai Fase 2 (Desember 2026) | — | Deploy otomatis dari `main` hijau | Konfirmasi Biznet Gio (PT-39) | `[DOK: 05-stack.md]` §2.9 |
| Uji beban NFR-CAP-04 | Januari 2027 | — | Staging dinaikkan ke ukuran production | Kapasitas host video untuk 100 sesi | `[DOK: 05-stack.md]` §2.9, §9 |
| Tahap 9 QA + SIT | 2027-02-01 – 2027-02-05 | 5 (v0.2: 4) | SIT dengan rekanan pertama | Kesiapan rekanan (PT-05) | `[UP-65]` `[SRC-282]` |
| Tahap 10 audit keamanan | 2027-02-01 – 2027-02-12 | 12 (v0.2: 9) | Audit eksternal (syarat go-live 6) | Auditor, ketersediaan, lama audit, dan apakah auditor juga libur 5 Februari (PT-49) | `[UP-65]` `[DOK: 01-business-case.md]` §4.2 |
| Production disiapkan | mulai 2027-02-01 | 12 di jendela 2027-02-01 – 02-12 (v0.2: 9) | Host, OpenBao (unseal PT-38), replikasi, backup, uji pemulihan NFR-DR-03, bootstrap Super Admin (PT-52), deploy pertama lewat bastion (EN-26, EN-27) | Pemegang share unseal dan orang kedua (PT-38); Super Admin pertama dan penyetujunya (PT-52) | `[DOK: 05-stack.md]` §2.9 |
| Tahap 11 UAT | 2027-02-08 – 2027-02-12 | 5 | Sign-off CEO, CTO, konsultan medis | Ketersediaan peserta (PT-50) | `[UP-65]` `[DOK: 01-business-case.md]` §4.2 |
| Tahap 12 pre-deploy | 2027-02-08 – 2027-02-19 | 12 (v0.2: 10) | Pengajuan ke Play Store dan App Store paling lambat 2027-02-12 | Lama review toko dan status akun developer (R-08, PT-51) | `[UP-65]` |
| Tahap 13 deploy | 2027-02-22 – 2027-02-26 | 5 | Deploy production final, smoke test | — | `[UP-65]` |
| Go-live | 2027-03-01 | — | — | — | `[DOK: 01-business-case.md]` |

Lama setiap jendela setelah pembekuan fitur adalah usulan penyusun yang disetujui `[USER] C2-Q5`. **v0.4 (CR-022):** bila auditor atau toko aplikasi butuh waktu lebih lama, atau pengembangan tidak selesai pada 2027-01-29, go-live boleh mundur (`[USER]` CR-022); mengurangi scope bukan lagi alat yang dipakai (v0.2 memakai P8.3). Syarat izin dan kepatuhan, audit keamanan, dan UAT tetap tidak bisa dikurangi. Apakah jendela pasca-pembekuan dapat dipadatkan `[TIDAK DIKETAHUI]`.

**Libur setelah go-live (untuk Tahap 14 hypercare).** Senin 8 Maret sampai Senin 15 Maret 2027 berisi libur nasional (Nyepi 8 Maret, Idulfitri 10–11 Maret) dan cuti bersama (9, 12, dan 15 Maret) `[SRC-282]` `[SRC-283]`. **v0.4:** tim bekerja 7 hari per minggu, sehingga hari itu bukan hari libur tim. Apakah penyedia luar (auditor, rekanan, DOKU, Biznet Gio, toko aplikasi) bekerja di hari itu `[TIDAK DIKETAHUI]`; pengaturan siaga diputuskan di Tahap 12 bersama PT-35. Bila go-live mundur (CR-022), jendela libur ini ikut ditinjau di gerbang 2026-12-11.

---

## 6. Estimasi effort dan analisis kapasitas

### 6.1 Metode `[KEPUTUSAN-178]`

- **Fase 1 dihitung dari bawah**: 80 task dengan ukuran 2, 3, atau 4 jam-task (§7.3), total 277 jam-task (v0.1: 71 task, 245 jam-task; tambahan F1-T72–F1-T80 untuk CR-017 dan CR-018) `[KEPUTUSAN-216]`.
- **Fase 2–4 dihitung dari atas** per story dengan kelas ukuran yang dikalibrasi dari pecahan Fase 1, setelah fondasi tersedia: S = 2 task (6 jam-task), M = 4 task (12), L = 7 task (21), XL = 12 task (36). Satu task rata-rata 3 jam-task. Kelas per story ada di §8.
- **Jam-task** adalah ukuran usaha untuk satu task 2–4 jam termasuk `scripts/verify.sh` dan satu putaran perbaikan review `[ASUMSI-01]`, bukan jam kalender dan bukan jam kerja orang tertentu.
- **Hari kerja (v0.4: hari kalender)**. Sejak v0.4 tim bekerja 7 hari per minggu, sehingga pembagi kebutuhan adalah **hari kalender** dalam jendela fase (akhir pekan dan libur ikut dihitung) `[USER]` 2026-11-19 `[KEPUTUSAN-281]`. Dasar v0.2 (dipertahankan sebagai riwayat): dihitung Senin–Jumat, dikurangi libur nasional dan cuti bersama yang jatuh di hari Senin–Jumat, karena tim libur pada hari itu `[USER] C2-Q7` (digantikan). Tanggalnya dari SKB tiga menteri 2026 `[SRC-280]` dan 2027 `[SRC-282]` `[SRC-283]`; libur yang jatuh di akhir pekan tidak mengubah hitungan. Bila pemerintah mengubah SKB atau menetapkan libur tambahan, hitungan ditinjau di laporan fase `[KEPUTUSAN-215]`. Cuti pribadi anggota tim tidak dihitung `[TIDAK DIKETAHUI]`.
- **Ketidakpastian**: kecepatan pengembangan Rust belum diukur (RT-24) `[DOK: 04-risk-register.md]`, dan kemampuan sistem rekanan belum diketahui (PT-05). Angka di bawah dipakai untuk membandingkan opsi, bukan sebagai janji.

### 6.2 Effort per fase

**Tabel v0.2 (riwayat, Opsi A, lima hari kerja; kolom per hari kerja digantikan oleh tabel v0.4 di bawahnya).** Opsi B, yang menjadi baseline, di §6.5 dan §6.6.

| Fase | Story | Enabler, UI, penutupan fase | Total jam-task | Task | Hari kerja (v0.1 → v0.2) | Jam-task per hari kerja (v0.1 → v0.2) |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | di dalam 80 task | — | 277 | 80 (nyata, dari §7.3) | 15 → 15 | 16,3 → **18,5** |
| 2 | 576 | 112 | 688 | ±229 (jam ÷ 3) | 20 → **16** | 31,1 → **43,0** |
| 3 | 600 (+42 terblokir PT-24) | 130 | 730 (+42) | ±243 (+14) (jam ÷ 3) | 15 → 15 | 47,9 → **48,7** |
| 4 (hanya Opsi A) | 192 | 8 | 200 | ±67 (jam ÷ 3) | 10 → 10 (paralel) | 20,0 → 20,0 |
| Fase 3 + Fase 4 paralel (Opsi A) | | | 930 | ±310 | 15 → 15 | 61,2 → **62,0** |
| Production dan uji pemulihan (Februari) | — | 72 | 72 | 24 (jam ÷ 3) | 10 → **9** | 7,2 → **8,0** |
| **Total Opsi A** | | | **1.967 (+42)** | **±643 (+14)** | | |

**Tabel v0.4 (baseline Opsi B + CR-021, 7 hari per minggu; jam-task dari §6.7)** `[KEPUTUSAN-281]`. Per minggu = per hari × 7.

| Fase | Jam-task | Hari kalender (v0.3: hari kerja) | Jam-task per hari kalender (v0.3) | Per minggu |
| --- | --- | --- | --- | --- |
| 1, seluruhnya (2026-11-23 – 12-11) | 361 | 19 (15) | **19,0** (24,1) | 133,0 |
| 1, sisa dari 2026-11-20 sampai 12-11 (basis di §6.8.1) | 288 | 22 (16) | **13,1** (19,8; sisa 316) | 91,6 |
| 2 (2026-12-14 – 2027-01-08) | 725 | 26 (16) | **27,9** (45,3) | 195,2 |
| 3 (2027-01-11 – 01-29) | 973 | 19 (15) | **51,2** (64,9) | 358,5 |
| Februari (2027-02-01 – 02-12) | 72 | 12 (9) | **6,0** (8,0) | 42,0 |
| **Total Opsi B + CR-021** | **±2.131** | | | |
| Pembanding Opsi A, Fase 3 + Fase 4 | 1.149 | 19 (15) | 60,5 (76,6) | 423,3 |

Perubahan v0.2 terhadap v0.1:
- **Fase 1** +32 jam-task: F1-T72–F1-T80 (§7.3) `[KEPUTUSAN-216]`.
- **Fase 2** +66 jam-task: US-110 kelas XL (36) `[KEPUTUSAN-218]`; reset MFA pasien oleh Layanan Pelanggan, US-207 AC5 (M, 12) `[KEPUTUSAN-217]`; CR-017 dan daftar ulang TOTP di aplikasi tenaga kesehatan (M, 12) `[KEPUTUSAN-219]`; notifikasi pemberian sub-peran sensitif UP-76 (S, 6) `[KEPUTUSAN-220]`. Hari kerja turun dari 20 ke 16 karena libur (§6.2a).
- **Fase 3** +12 jam-task: CR-017, daftar ulang TOTP, dan US-110 untuk realm `rme` di web RME (M, 12) `[KEPUTUSAN-219]`.
- **Februari**: hari kerja turun dari 10 ke 9 (5 Februari 2027). Jendela dihitung 2027-02-01 – 02-12, sama dengan jendela audit `[KEPUTUSAN-215]`.

**Satuan perbandingan.** Kolom "Task" mencampur dua satuan: Fase 1 berisi 80 task nyata (rata-rata ±3,46 jam-task per task), sedangkan Fase 2–4 dan Februari adalah jam-task ÷ 3. Karena itu kebutuhan antarfase dibandingkan dalam **jam-task per hari kerja**, bukan task per hari kerja.

### 6.2a Hari libur dan hari kalender di jendela pengembangan `[KEPUTUSAN-215]` (diperbarui v0.4 `[KEPUTUSAN-281]`)

**v0.4: libur tidak mengurangi hari kerja.** Tim bekerja 7 hari per minggu, termasuk libur nasional, cuti bersama, dan akhir pekan `[USER]` 2026-11-19 (mengubah C2-Q7). Tabel libur di bawah dipertahankan hanya sebagai informasi (tanggal SKB) dan karena penyedia luar mungkin libur (`[TIDAK DIKETAHUI]`). Hari per jendela:

| Jendela | Tanggal | Hari kalender (v0.4) | Hari kerja v0.2 | Selisih: libur nasional/cuti bersama | Selisih: akhir pekan |
| --- | --- | --- | --- | --- | --- |
| Fase 1 | 2026-11-23 – 12-11 | 19 | 15 | 0 | 4 (28–29 Nov, 5–6 Des) |
| Fase 1, sisa dari hari ini | 2026-11-20 – 12-11 | 22 | 16 | 0 | 6 (21–22, 28–29 Nov, 5–6 Des) |
| Fase 2 | 2026-12-14 – 2027-01-08 | 26 | 16 | 4 (24, 25 Des, 1, 5 Jan) | 6 (19–20, 26–27 Des, 2–3 Jan) |
| Fase 3 | 2027-01-11 – 01-29 | 19 | 15 | 0 | 4 (16–17, 23–24 Jan) |
| Februari | 2027-02-01 – 02-12 | 12 | 9 | 1 (5 Feb; 6 Feb Sabtu sudah akhir pekan) | 2 (6–7 Feb) |
| Penyangga antar-fase, tidak dialokasikan | 12–13 Des, 9–10 Jan | 4 | 0 | 0 | 4 |
| Mobile developer | 2026-11-23 – 2027-02-05 | 75 | 50 | 5 | 20 |

Hitungan: tanggal dan hari dalam seminggu dari kalender (perhitungan tanggal); jumlah selisih = hari kalender − hari kerja v0.2, ditelusuri per tanggal di tabel libur. Jam kerja per hari dan per minggu tidak diketahui (`[TIDAK DIKETAHUI]`), sehingga kenaikan jumlah hari belum berarti kenaikan kapasitas sebesar itu (§6.8.2).

Riwayat v0.2: tabel libur di bawah.

| Tanggal | Hari | Jenis | Keterangan | Jendela terdampak | Sumber |
| --- | --- | --- | --- | --- | --- |
| 2026-12-24 | Kamis | Cuti bersama | Kelahiran Yesus Kristus | Fase 2 | `[SRC-280]` |
| 2026-12-25 | Jumat | Libur nasional | Kelahiran Yesus Kristus | Fase 2 | `[SRC-280]` |
| 2027-01-01 | Jumat | Libur nasional | Tahun Baru 2027 Masehi | Fase 2 | `[SRC-282]` `[SRC-283]` |
| 2027-01-05 | Selasa | Libur nasional | Isra Mikraj Nabi Muhammad saw. (1448 H) | Fase 2 | `[SRC-282]` `[SRC-283]` |
| 2027-02-05 | Jumat | Cuti bersama | Tahun Baru Imlek 2578 Kongzili | Februari (QA/SIT, audit, production) | `[SRC-282]` `[SRC-283]` |
| 2027-02-06 | Sabtu | Libur nasional | Tahun Baru Imlek 2578 Kongzili | Tidak mengurangi (akhir pekan) | `[SRC-282]` `[SRC-283]` |

- Jendela Fase 1 (2026-11-23 – 12-11) dan Fase 3 (2027-01-11 – 01-29) tidak memuat libur nasional atau cuti bersama `[SRC-280]` `[SRC-282]`.
- Untuk instansi swasta, pelaksanaan cuti bersama diatur pimpinan masing-masing `[SRC-283]`; untuk tim proyek ini, pimpinan memutuskan libur `[USER] C2-Q7`. Apakah penyedia luar (auditor, rekanan, DOKU, Biznet Gio, peninjau toko aplikasi) bekerja di hari itu `[TIDAK DIKETAHUI]`.
- Mobile developer (v0.2): 50 hari kerja di 2026-11-23 – 2027-02-05 (55 dikurangi 24 dan 25 Desember, 1 dan 5 Januari, 5 Februari). **v0.4: 75 hari kalender** (tabel di atas); jam kerja ±40 per minggu `[USER] C2-Q3` tidak diubah oleh jawaban 7 hari per minggu kecuali user menyatakan lain `[TIDAK DIKETAHUI]`.

Rincian enabler Fase 2–4: EN-15 staging dan CD 24, EN-16 observability 20, EN-17 realm nakes dan kunci aplikasi 16, EN-18 LiveKit 8, EN-19 realtime 12, EN-20 kerangka portal 10, EN-24 pajak CR-013 8, UI-04 template email 6, EN-21 gateway integrasi 30, EN-22 fondasi RME 55, EN-23 pekerja privasi 15, EN-25 uji beban 16, UI-05 uji aksesibilitas manual 6, EN-26 production 60, EN-27 uji pemulihan 12, penutupan fase 8 per fase (estimasi penyusun per enabler) `[KEPUTUSAN-178]`.

**Porsi mobile** (aplikasi pasien dan tenaga kesehatan; seluruhnya dikerjakan mobile developer `[USER] C2-Q3`) diperkirakan ±449 jam-task untuk Opsi A: Fase 1 44 (dari daftar task: 37 di v0.1 ditambah F1-T77 dan F1-T78), Fase 2 ±195 (±180 di v0.1, ditambah layar US-110 di dua aplikasi ±9 dan layar ubah password serta daftar ulang TOTP di aplikasi tenaga kesehatan ±6), Fase 3 ±120, Fase 4 ±90 (estimasi kasar per fase berdasarkan story yang punya layar mobile) `[KEPUTUSAN-178]` `[KEPUTUSAN-221]`. Untuk Opsi B: ±359 ditambah porsi mobile US-108/US-109 yang pindah ke Fase 3 (bagian dari ±90 Fase 4, belum dipisahkan). Sisanya adalah backend Rust, web Next.js, infrastruktur, dan penutupan fase: ±1.518 jam-task (Opsi A), paling banyak ±1.432 (Opsi B).

### 6.3 Kapasitas tim yang tercatat

| Anggota | Yang tercatat | Yang belum tercatat | Sumber |
| --- | --- | --- | --- |
| Project manager (user) | Menjalankan runbook, memutuskan di checkpoint; menjalankan sesi Claude ≥ 6 jam per hari kerja | — | `[DOK: 00-project-brief.md]` P7.3; `[USER] C2-Q4` |
| Claude lewat runbook | Mengerjakan task Tahap 8 (backend, web, infrastruktur) per sesi dengan review subagent, ≥ 6 jam sesi per hari kerja | Throughput (jam-task per hari kerja) `[TIDAK DIKETAHUI]`; diukur di Fase 1 | `[DOK: 00-project-brief.md]` P7.3; `[USER] C2-Q4`; RT-24 |
| Mobile developer (1 orang) | Penuh waktu sampai go-live, ±40 jam/minggu; mengerjakan seluruh layar mobile | Throughput dalam jam-task `[TIDAK DIKETAHUI]`; diukur di Fase 1 | `[USER] T5-Q7, C2-Q3` |
| Developer backend/web tambahan (Opsi E) | Disetujui; mulai awal Januari 2027, membantu mulai Fase 3. Kapasitas Fase 2 tetap tim lama | Jumlah orang, biaya terhadap toleransi +20%, tanggal mulai pasti `[TIDAK DIKETAHUI]` (PT-53, pemilik CEO, tenggat gerbang re-plan 2026-12-11); waktu onboarding `[TIDAK DIKETAHUI]` | `[USER] C2-Q2, C2-Q8` |
| QA manusia, DevOps, UI/UX | Tidak disebut | — | `[DOK: 01-business-case.md]` R-04 |

**Satuan.** Jam sesi Claude dan jam kerja mobile developer adalah jam orang atau jam sesi, sedangkan kebutuhan di dokumen ini dalam jam-task (ukuran task termasuk `verify.sh` dan satu putaran review, `[ASUMSI-01]`). Rasio keduanya belum diketahui, sehingga penyusun **tidak mengonversi** jam sesi atau jam kerja ke jam-task. Rasionya diukur di Fase 1 lewat kecepatan per jalur (§6.5) `[KEPUTUSAN-221]`.

### 6.4 Kebutuhan dibandingkan kapasitas

- **(Angka v0.2, lima hari kerja; angka v0.4 di §6.8.2)** **Seluruh tim (Opsi B baseline)**: kebutuhan naik dari ±18,5 jam-task per hari kerja (Fase 1) ke ±43,0 (Fase 2) dan ±50,3 (Fase 3, 754 jam-task dalam 15 hari kerja). Opsi A: ±62,0 bila Fase 3 dan Fase 4 berjalan paralel (930 jam-task dalam 15 hari kerja). Throughput tim `[TIDAK DIKETAHUI]`.
- **Fase 2 menjadi titik terberat tim lama.** Kebutuhannya naik ±38% dari v0.1 (31,1 → 43,0): hari kerja turun 20 → 16 karena libur, dan isi bertambah 66 jam-task (US-110, reset MFA pasien US-207 AC5, CR-017 dan daftar ulang TOTP realm `nakes`, notifikasi sub-peran). Opsi E baru membantu mulai Fase 3 `[USER] C2-Q8`, sehingga Fase 2 ditanggung PM + Claude dan mobile developer saja.
- **Durasi minimum Fase 1**: jalur kritis 65 jam-task berurutan (§7.4) dari 277 jam-task. Sisa 212 jam-task hanya muat di 15 hari kerja bila dikerjakan paralel di jalur backend, web, dan mobile; jalur kritis sendiri memuat 15 jam-task mobile (F1-T58, F1-T63) dan pola reference implementation (F1-T66) sebelum profil.

Kebutuhan per jalur, **angka v0.2 (lima hari kerja, tanpa CR-021; v0.4 di §6.8.2)** (Opsi B; jam-task per hari kerja; jalur mobile = mobile developer, jalur backend/web = Claude yang dijalankan PM, ditambah Opsi E mulai Fase 3) `[KEPUTUSAN-221]`:

| Fase | Hari kerja | Mobile (jam-task) | Mobile per hari kerja | Backend, web, infrastruktur (jam-task) | Backend/web per hari kerja |
| --- | --- | --- | --- | --- | --- |
| 1 | 15 | 44 | 2,9 | 233 | 15,5 |
| 2 | 16 | ±195 | ±12,2 | ±493 | ±30,8 |
| 3 (Opsi B) | 15 | ±120 + porsi US-108/US-109 | ≥ ±8,0 | ≤ ±634 | ≤ ±42,3 |
| Februari | 9 | perbaikan bug QA/SIT/audit/UAT (belum diestimasi) | — | 72 | 8,0 |

- **Mobile developer (v0.2; v0.4: porsi Opsi B + CR-021 ≥ 383 jam-task, 37 + 202 + ≥144, selama 75 hari kalender = rata-rata ≥ ±5,1 per hari kalender, puncak ±7,8 di Fase 2 = 202 ÷ 26; §6.8.2)**: porsi Opsi B (±359 jam-task ditambah porsi US-108/US-109) berarti rata-rata ≥ ±7,2 jam-task per hari kerja selama 50 hari kerja (2026-11-23 – 2027-02-05), dengan puncak ±12,2 di Fase 2 (195 ÷ 16). Opsi A: ±449 jam-task, rata-rata ±9,0. Kapasitas yang tercatat ±40 jam kerja per minggu `[USER] C2-Q3` beda satuan dengan jam-task dan tidak dikonversi (§6.3). Perhitungan ini belum memasukkan perbaikan bug dari QA, SIT, audit, dan UAT. Fase 1 relatif ringan untuk jalur mobile (2,9 per hari kerja), tetapi dua task mobile ada di jalur kritis (F1-T58, F1-T63).
- **Backend, web, infrastruktur** bergantung pada throughput Claude dalam sesi ≥ 6 jam per hari kerja `[USER] C2-Q4`, ditambah review subagent per task, dan mulai Fase 3 pada developer tambahan (PT-53). Angka throughput `[TIDAK DIKETAHUI]`.
- **Risiko yang menaikkan kebutuhan**: RT-24 (Rust lebih lambat), RT-01 (rekanan tidak mampu mengikuti kontrak), RT-05 (registrasi SATUSEHAT), RT-27 (penyedia diganti), dan putaran review T1 (maksimal 2 putaran per `aturan-kerja.md` §1) `[DOK: 04-risk-register.md]`.

### 6.5 Kesimpulan kelayakan dan kriteria gerbang re-plan

**Pembaruan v0.4 (CR-022 dan 7 hari per minggu).** Fungsi gerbang berubah: gerbang 2026-12-11 menetapkan **tanggal go-live baru**, bukan memilih opsi pemotongan scope (§6.8.5). Definisi kecepatan terukur `v` memakai **hari kalender**: rata-rata 10 hari kalender terakhir Fase 1, yaitu 2026-12-02 – 12-11 (v0.2: 10 hari kerja 2026-11-30 – 12-11). Tabel opsi di bawah dihitung ulang dengan hari kalender dan jam-task v0.3 (§6.7); Opsi C dan D hanya pembanding angka, tidak dipakai sebagai pemotongan (CR-022). Teks kesimpulan dan kriteria v0.2 di bawah tabel dipertahankan sebagai riwayat; bila bertentangan, §6.8 berlaku.

**Kesimpulan (v0.2):** jadwal 1 Maret 2027 untuk scope penuh (Opsi A) **tidak dapat dinyatakan layak**. Indikasinya kuat tidak layak: kebutuhan kecepatan Fase 3 + Fase 4 (±62,0 jam-task per hari kerja) lebih dari tiga kali kebutuhan Fase 1 (±18,5), dan Fase 2 sendiri (±43,0) sudah lebih dari dua kali Fase 1, dalam tim tanpa peran QA atau DevOps, dengan toleransi waktu 0. Baseline Opsi B `[USER] C2-Q1` dan tambahan developer backend/web mulai Januari `[USER] C2-Q2, C2-Q8` menurunkan beban Fase 3, tetapi kelayakannya juga baru dapat dinilai setelah kecepatan terukur.

**Kriteria gerbang re-plan 2026-12-11** `[UP-66]`, disetujui `[USER] C2-Q6`. Kecepatan terukur `v` = jumlah ukuran (kolom "Jam" §7.3) task yang selesai per hari kerja, dirata-rata selama 10 hari kerja terakhir Fase 1 (selesai = commit hijau + `code-reviewer` lolos, anti-halu §5). Satuan ini sama dengan estimasi Fase 2–4. Fase 1 tidak memuat hari libur, sehingga 10 hari kerja terakhirnya adalah 2026-11-30 – 2026-12-11. `v` dicatat juga per jalur: `v_mobile` (task mobile developer) dan `v_bw` (task backend, web, infrastruktur) `[KEPUTUSAN-221]`.

| Opsi (v0.4, dengan CR-021) | Jam-task per hari kalender yang dibutuhkan di Fase 2 (26 hari kalender) | Di Fase 3 (+ Fase 4 bila ada; 19 hari kalender) |
| --- | --- | --- |
| A (pembanding) | 27,9 (725 ÷ 26) | 60,5 (1.149 ÷ 19) |
| B (baseline, CR-022) | 27,9 | 51,2 (973 ÷ 19) |
| C (pembanding, tidak dipakai) | 27,9 | 47,4 (900 ÷ 19) |
| D (pembanding, tidak dipakai) | 27,9 | 27,3 (519 ÷ 19) |

Angka v0.2 (lima hari kerja, tanpa CR-021): A 43,0 dan 62,0; B 43,0 dan 50,3; C 43,0 dan 45,4; D 43,0 dan 20,0. Peringatan dependensi Opsi C dan D atas K2/K3 (§6.7) tetap berlaku bila salah satunya pernah dipertimbangkan.

Kebutuhan per jalur untuk Opsi B ada di tabel §6.4: Fase 2 mobile ±12,2 dan backend/web ±30,8; Fase 3 mobile ≥ ±8,0 dan backend/web ≤ ±42,3.

- `v` di bawah kebutuhan Fase 2 berarti Fase 2 sendiri tidak muat, karena Opsi E baru membantu mulai Fase 3. Pilihan yang tersisa: memajukan mulai Opsi E (keputusan CEO, PT-53) atau memotong isi Fase 2 (misalnya video atau resep). Memotong isi Fase 2 bukan bagian urutan P8.3, sehingga butuh keputusan user sebagai CR.
- **Opsi E di gerbang** `[KEPUTUSAN-222]`. Kontribusi developer tambahan belum dapat dihitung sampai jumlah orangnya diketahui (PT-53) dan kecepatannya terukur. Karena itu gerbang menyajikan **kekurangan Fase 3 jalur backend/web** = kebutuhan opsi terpilih − `v_bw`, dalam jam-task per hari kerja. Angka ini adalah kecepatan yang harus disumbang Opsi E. Bila PT-53 belum terjawab pada 2026-12-11, Fase 3 direncanakan tanpa kontribusi Opsi E, dan pilihan opsi scope mengikuti `v` tim lama.
- `v_mobile` di bawah kebutuhan jalur mobile berarti layar mobile menjadi pembatas, walaupun jalur backend/web cukup. Opsi E disetujui untuk backend/web, bukan mobile `[USER] C2-Q2`, jadi kekurangan jalur mobile ditutup lewat pilihan opsi scope atau keputusan user baru.
- Bila Fase 1 tidak selesai 2026-12-11, gerbang tetap dilaksanakan pada tanggal itu dengan data yang ada, karena setiap minggu keterlambatan mengurangi jendela Fase 2–3.

### 6.6 Opsi scope dan dampaknya

| Opsi | Isi | Effort (jam-task) | Dampak bisnis | Sumber dampak |
| --- | --- | --- | --- | --- |
| A. Semua | Fase 1–4 | ±1.967 (+42) | Ketujuh layanan berjalan (syarat go-live 3). Butuh kecepatan tertinggi | `[DOK: 01-business-case.md]` §4.2 |
| B. Lepas nomor 1–3 (**baseline**, `[USER] C2-Q1`) | Tanpa antrean real-time, pembelian produk, dan home visit; anggota keluarga (US-108, US-109) dipindah ke Fase 3 | ±1.791 | Syarat go-live 3 ("ketujuh layanan berjalan") tidak terpenuhi, yang diizinkan P8.3; dua layanan (produk, home visit) dan antrean menyusul setelah go-live; syarat go-live 1 bagian "3 outlet terintegrasi" tidak terpenuhi; pemasaran menyesuaikan (R-07) | `[DOK: 03-prd.md]` §2.3; `[DOK: 01-business-case.md]` R-07 |
| C. Lepas nomor 1–5 | B + konsultasi fisioterapis/tenaga medis (dinonaktifkan per jenis, effort kecil) + pendaftaran lab, info biaya, hasil lab (EP-06, EP-09, US-1307, bagian lab EP-04) | ±1.718 | Syarat go-live 3 tidak terpenuhi (diizinkan P8.3), dan syarat go-live 1 bagian lab dan outlet juga tidak; tersisa konsultasi dokter dan psikolog serta pendaftaran klinik; RME minimalis tetap dibangun untuk klinik | `[DOK: 03-prd.md]` §2.3 |
| D. Lepas nomor 1–6 | Hanya konsultasi dokter dan psikolog, pembayaran, dan fitur kepatuhan; tanpa Partner Integration API, RME minimalis, klinik/lab | ±1.337 | Syarat go-live 1 (rekanan terintegrasi) dan 3 (tujuh layanan) tidak terpenuhi, yang diizinkan P8.3. **Perlu jawaban PT-47**: klinik penaung tenaga kesehatan tanpa integrasi tidak memenuhi syarat tampil US-205 AC2. CR-006 (RME minimalis) tidak berjalan di rilis 1 | `[DOK: 00-project-brief.md]` P8.3; `[DOK: 03-prd.md]` US-205 AC2, UP-33 |
| E. Tambah kapasitas (**dijalankan bersama B**, `[USER] C2-Q2`) | Developer backend/web tambahan mulai awal Januari 2027 `[USER] C2-Q8` | Effort sama; kebutuhan per orang turun mulai Fase 3 | Biaya di luar baseline (toleransi +20% `[USER] T1-Q8`); jumlah orang, biaya, tanggal mulai pasti, dan waktu onboarding `[TIDAK DIKETAHUI]` (PT-53); keputusan CEO | `[DOK: 01-business-case.md]` §6.1, §7.1 |
| — Tanggal mundur | **v0.4: tersedia** (v0.2: tidak tersedia). Go-live boleh mundur bila scope penuh tidak muat; tanggal ditetapkan di gerbang 2026-12-11 (§6.8.5) | — | Toleransi waktu 0 diubah CR-022. Dampak biaya (toleransi +20%, `[USER] T1-Q8`), jendela audit dan toko aplikasi (RT-34), dan komitmen pemasaran (R-07) perlu ditinjau di gerbang | `[USER]` CR-022 (`docs/backlog.md`); `[DOK: 01-business-case.md]` §7.1 (diubah) |

Effort B, C, D dihitung dari tabel §6.2 dikurangi story yang dilepas: B = A − 176 (Fase 4 tanpa US-108/US-109); C = B − 73 (EP-06 18, EP-09 24, US-1307 12, bagian lab EP-04 ±19); D = B − 454 (EP-04 63, EP-06 18, EP-09 24, EP-13 tanpa US-1305/US-1308 156, EP-14 96, EN-21 30, EN-22 55, dan bagian realm `rme` dari CR-017, daftar ulang TOTP, dan US-110 sebesar 12) `[KEPUTUSAN-178]`. Tambahan v0.2 lainnya (F1-T72–F1-T80, US-110, US-207 AC5, CR-017 dan daftar ulang TOTP realm `nakes`, notifikasi sub-peran) berlaku di semua opsi.

**Keputusan Checkpoint 2** `[UP-67]`: Opsi B sebagai baseline rencana (Fase 4 tidak dikerjakan kecuali gerbang re-plan memilih A), opsi final di gerbang re-plan 2026-12-11 `[USER] C2-Q1`, dan Opsi E dijalankan dengan developer backend/web mulai awal Januari 2027 `[USER] C2-Q2, C2-Q8`.

### 6.7 Perubahan v0.3: CR-021 skenario S3 `[USER] C21-P4`

Dasar hitung `[KEPUTUSAN-260]` `[KEPUTUSAN-261]`:
- Bagian Fase 1 dihitung dari bawah, yaitu F1-T81–F1-T111 di §7.3 (31 task, 109 jam-task).
- Bagian lain per story kelas KEPUTUSAN-178: K1-F2 reset MFA HR M 12; K2 195 (backend/web 177, mobile 18); K3 36 (backend/web 30, mobile 6).
- US-1101 (M 12) keluar dari Fase 3 karena sudah termasuk F1-T91–F1-T95.
- Rincian di `docs/cr/CR-021-analisis.md` §8–§10, dikoreksi fact-check N-2.

| Fase | Opsi B v0.2 (jam-task; per hari kerja) | Opsi B v0.3 (jam-task; per hari kerja) | Backend/web v0.3 | Mobile v0.3 |
| --- | --- | --- | --- | --- |
| 1 (15 hari kerja kalender) | 277; 18,5 | **361; 24,1** (277 − 25 + 109) | 324; 21,6 | 37; 2,5 |
| 1, sisa per 2026-11-19 (16 hari kerja, 2026-11-20 – 12-11) | — | **316; 19,8** (361 − 45 selesai) | 279; **17,4** | 37; 2,3 |
| 2 (16 hari kerja) | 688; 43,0 | **725; 45,3** (688 + 25 + 12) | 523; 32,7 | 202; 12,6 |
| 3 (15 hari kerja) | 754; 50,3 | **973; 64,9** (754 − 12 + 195 + 36) | ≤ 829; ≤ 55,3 | ≥ 144; ≥ 9,6 |
| Februari (9 hari kerja) | 72; 8,0 | 72; 8,0 | 72; 8,0 | — |
| **Total Opsi B** | **±1.791** | **±2.131** (+340, ±19%) | | |

Catatan:
- Fase 1 berjalan sejak 2026-11-18 `[DOK: STATUS.md]`. Per 2026-11-19 tercatat 14 task (±45 jam-task) selesai atau ter-merge (F1-T01–T05, T07–T15) `[DOK: fase-1-catatan.md]`. Angka ini indikatif, bukan `v`.
- Pembanding dari atas (semua per kelas): tambahan ±435 jam-task, total ±2.226.

**Opsi lain dengan CR-021 v0.3** (Fase 3, 15 hari kerja). Dihitung dengan cara §6.6 ditambah CR-021:
- A: 930 − 12 + 195 + 36 = 1.149 (US-108/US-109 tidak dihitung dua kali), yaitu 76,6 per hari kerja; total ±2.307 (koreksi fact-check `docs/fact-check/CR-021-penerapan.md` B-2).
- C: 681 − 12 + 195 + 36 = 900, yaitu 60,0.
- D: 300 − 12 + 195 + 36 = 519, yaitu 34,6.
- **Peringatan dependensi:**
  - Opsi C melepas EP-09 (hasil lab), padahal US-1915 (hasil lengkap peserta) memakai alur US-901.
  - Opsi D melepas Partner Integration API (EN-21), padahal katalog korporat masuk lewat integrasi (K-04 opsi a) dan US-1917 memakai kontrak yang sama.
  - Karena itu, memilih C atau D di gerbang re-plan juga memerlukan keputusan atas K2/K3 CR-021. Angka C dan D di atas hanya dapat dicapai bila keputusan itu diambil (PT-69).

**Kriteria gerbang re-plan 2026-12-11 (pembaruan v0.3).** Kebutuhan Opsi B kini 45,3 di Fase 2 dan 64,9 di Fase 3. Kekurangan Fase 3 jalur backend/web yang harus disumbang Opsi E = 55,3 − `v_bw` (§6.5). Aturan §6.5 lainnya tetap berlaku.

**Dampak ke jalur kritis Fase 1.** Longest path tetap 65 jam-task. Rantai CR-021 terpanjang (F1-T01 → … → T17 → F1-T98 → F1-T99 → F1-T100 → F1-T104 → F1-T105 → F1-T106 → F1-T110) selesai paling awal di jam-task ke-51, dengan slack ±11 terhadap F1-T70. Pembatas nyata adalah kapasitas jalur backend/web.

*Tabel §6.7 di atas adalah angka v0.3 (lima hari kerja); versi hari kalender ada di tabel v0.4 di §6.2 dan di §6.8.*

### 6.8 Perubahan v0.4: 7 hari per minggu, CR-022, progres Fase 1 per 2026-11-20

Sumber arahan: `[USER]` 2026-11-19 ("tidak ada hari libur. jalan terus"; akhir pekan **Y**) di `docs/fase-1-catatan.md`; `[USER]` CR-022 di `docs/backlog.md`. Semua angka di bawah adalah **perhitungan penyusun** dari jam-task §7.3 dan §6.7 (estimasi, bukan fakta) `[KEPUTUSAN-281]`–`[KEPUTUSAN-284]`.

#### 6.8.1 Basis hitung

- Fase 1 **mulai nyata 2026-11-18** `[DOK: STATUS.md]`. Jendela rencana tetap 2026-11-23 – 12-11 (19 hari kalender). Dari mulai nyata sampai 12-11: 24 hari kalender (361 ÷ 24 = 15,0 jam-task per hari).
- **Sisa** dihitung dari **hari ini, 2026-11-20, inklusif** sampai 2026-12-11: 22 hari kalender (11 hari di November + 11 di Desember). Alasannya: sisa pekerjaan dibagi sisa waktu yang masih bisa dipakai. Bila hari ini tidak dihitung (mulai 11-21): 21 hari, 288 ÷ 21 = 13,7.
- Fase 2–3 dan Februari memakai hari kalender di jendela yang disetujui `[USER] C2-Q5` (§6.2a). Akhir pekan antar-fase (12–13 Des, 9–10 Jan) tidak dialokasikan; bila dipakai, jendela Fase 2 + Fase 3 menjadi 49 hari (2026-12-12 – 2027-01-29).

#### 6.8.2 Kebutuhan per hari kalender

Jam-task Opsi B + CR-021 dari §6.7; per minggu = per hari × 7.

| Fase | Hari kalender | Jam-task (total / backend-web / mobile) | Per hari kalender (total / backend-web / mobile) | Per minggu (total) |
| --- | --- | --- | --- | --- |
| 1, seluruhnya | 19 | 361 / 324 / 37 | 19,0 / 17,1 / 1,9 | 133,0 |
| 1, sisa dari 2026-11-20 | 22 | 288 / 251 / 37 | **13,1 / 11,4 / 1,7** | 91,6 |
| 2 | 26 | 725 / 523 / 202 | **27,9 / 20,1 / 7,8** | 195,2 |
| 3 | 19 | 973 / ≤ 829 / ≥ 144 | **51,2 / ≤ 43,6 / ≥ 7,6** | 358,5 |
| Februari | 12 | 72 / 72 / — | **6,0** / 6,0 / — | 42,0 |

- Sisa Fase 1 per jalur: backend/web 324 − 73 selesai = 251; mobile 37 (tidak ada task mobile tercatat selesai; Flutter dikerjakan terpisah di laptop `[USER]` 2026-11-19, `docs/fase-1-catatan.md`).
- **Peringatan penafsiran.** Kebutuhan per hari turun terutama karena pembagi hari bertambah (Fase 1 15 → 19, Fase 2 16 → 26, Fase 3 15 → 19, Februari 9 → 12). Kapasitas riil naik sebesar itu hanya bila jam kerja per hari tetap sama di akhir pekan dan libur. Jam kerja per hari dan per minggu (sesi Claude ≥ 6 jam per hari kerja `[USER] C2-Q4`; mobile developer ±40 jam per minggu `[USER] C2-Q3`) apakah berubah `[TIDAK DIKETAHUI]`; penyusun tidak mengonversinya ke jam-task (§6.3). Karena itu kolom per minggu ditampilkan sebagai pembanding yang tidak bergantung pada jumlah hari kerja.
- Mobile developer: ≥ 383 jam-task (37 + 202 + ≥ 144) dalam 75 hari kalender (2026-11-23 – 2027-02-05) = rata-rata ≥ 5,1 per hari kalender.

#### 6.8.3 Progres Fase 1 per 2026-11-20 (jam-task dari kolom "Jam" §7.3)

| Kelompok | Task selesai atau ter-merge | Jam-task selesai | Dari total | Persen jam-task |
| --- | --- | --- | --- | --- |
| Baseline Fase 1 (73 task, 252 jam-task) | F1-T01–F1-T18 (18 task; termasuk spike F1-T02, F1-T03) | 59 (2+3+3+3+3+3+3+3+3+4+4+3+3+4+4+3+4+4) | 252 | 23,4% |
| CR-021 Fase 1 (31 task, 109 jam-task) | F1-T81, F1-T82 (dokumen gate, diterapkan 2026-11-19 `[DOK: backlog.md]` CR-021), F1-T83, F1-T84 (desain) | 14 (4+4+3+3) | 109 | 12,8% |
| **Fase 1 seluruhnya (104 task, 361 jam-task)** | **22 task (21,2%)** | **73** | **361** | **20,2%** |

- Sisa: **82 task, 288 jam-task**: baseline 55 task, 193 jam-task (F1-T19–F1-T80 tanpa tujuh task yang pindah ke Fase 2); CR-021 27 task, 95 jam-task (F1-T85–F1-T111).
- TD-01 (pemeriksa teks migrasi `EXECUTE`, `[DOK: backlog.md]` Tech Debt) selesai tetapi tidak punya ukuran jam-task, sehingga tidak dihitung `[KEPUTUSAN-284]`. TD-07–TD-10 (review F1-T18) belum dikerjakan dan tidak ada di total.
- Bila F1-T81 dan F1-T82 tidak dihitung (daftar status resmi di `docs/STATUS.md` hanya menyebut F1-T83/F1-T84 untuk CR-021): 65 jam-task selesai (18,0%), sisa 296 `[KEPUTUSAN-284]`.
- Jalur kritis 65 jam-task (§7.4): 26 jam-task di belakang (F1-T01, T04, T05, T07, T13, T14, T15, T17), sisa 39 (F1-T27, T29, T60, T61, T62, T66, T56, T58, T63, T70, T71). F1-T19 belum dikerjakan.
- **Kecepatan indikatif** (bukan `v` gerbang): 73 jam-task dalam 3 hari kalender (2026-11-18 – 11-20) = ±24,3 per hari; kebutuhan sisa 13,1 per hari (backend/web 11,4). Angka ini dipengaruhi isi yang ringan (spike, dokumen, desain), tidak memuat task mobile, dan sampel tiga hari. Bila bertahan, sisa 288 selesai dalam ±12 hari (±2026-12-01 – 12-02); penyusun tidak memakainya sebagai komitmen. `v` yang sah diukur 2026-12-02 – 12-11 (§6.5).

#### 6.8.4 Urutan Fase 1 menurut CR-022

1. **Baseline lebih dulu**: F1-T19–F1-T80 yang tetap di Fase 1, kecuali F1-T71 (54 task, 190 jam-task).
2. **Lalu CR-021**: F1-T85–F1-T111 (27 task, 95 jam-task), mengikuti dependensi §7.3.
3. **F1-T71 (penutupan Fase 1) paling akhir** (3 jam-task) karena bergantung pada F1-T110.
- Waktu bila kecepatan backend/web tepat sebesar kebutuhan rata-rata (11,4 per hari): baseline backend/web 156 jam-task (193 − 37 mobile) selesai ±hari ke-14 (≈ 2026-12-03); CR-021 95 jam-task butuh ±8,3 hari lagi, selesai tepat 2026-12-11. Jalur mobile (37 jam-task, kebutuhan 1,7 per hari) berjalan paralel di laptop. Jadi 2026-12-11 tercapai **hanya** bila kecepatan backend/web ≥ 11,4 jam-task per hari kalender sejak hari ini; tidak ada slack di luar kecepatan itu.
- Tidak ada task baseline yang bergantung pada task CR-021 selain F1-T71; urutan ini tidak melanggar dependensi §7.3. Koreksi dependensi: **F1-T19 bergantung pada F1-T16 dan F1-T18** (tabel `consent_record` memakai FK ke `patient`/`care_recipient`) `[DOK: fase-1-catatan.md]` "Posisi saat sesi berhenti"; baris F1-T19 di §7.3 diperbarui.
- Bila Fase 1 belum selesai pada 2026-12-11, gerbang tetap berjalan pada tanggal itu dengan data yang ada; sisa Fase 1 yang belum selesai menjadi tambahan beban Fase 2 dan masuk perhitungan tanggal go-live (§6.8.5). Risiko mundur diterima user `[USER]` CR-022.

#### 6.8.5 Gerbang 2026-12-11: tanggal go-live baru `[KEPUTUSAN-283]`

Yang diputuskan user di gerbang: (1) tanggal go-live baru, dan (2) tambahan tenaga (PT-53: jumlah orang, biaya terhadap toleransi +20% `[USER] T1-Q8`, tanggal mulai). Pilihan opsi pemotongan scope (C, D, atau mengurangi Fase 2–3) **tidak** diajukan di gerbang; scope tetap Opsi B + CR-021 `[USER]` CR-022. Keputusan atas Opsi A (Fase 4) tetap di gerbang (lihat §6.8.6).

Cara menetapkan tanggal:
- `S` = sisa jam-task Fase 1 yang belum selesai pada 2026-12-11 + 725 (Fase 2) + 973 (Fase 3) = 1.698 bila Fase 1 selesai penuh. `v` = kecepatan tim terukur (§6.5), hari kalender.
- Tanggal pembekuan fitur baru = 2026-12-11 + ⌈S ÷ v⌉ hari kalender. Go-live baru = pembekuan + **31 hari kalender**, yaitu panjang jendela 2027-01-29 → 2027-03-01 di rencana yang disetujui (QA/SIT, audit, UAT, pre-deploy, deploy; §5). Apakah jendela ini dapat dipadatkan `[TIDAK DIKETAHUI]`. Pekerjaan Februari (72 jam-task) berjalan paralel di jendela itu.
- Asumsi ilustrasi: Fase 1 selesai penuh pada 12-11, `v` konstan, tanpa tambahan tenaga, ukuran task tidak berubah (penilaian ulang ukuran, KEPUTUSAN-223, belum dilakukan sehingga cenderung terlalu rendah), akhir pekan antar-fase ikut dipakai (kontinu dari 12-12), dan tidak ada pemisahan jalur mobile.

| `v` (jam-task per hari kalender, seluruh tim) | Hari yang dibutuhkan (1.698 ÷ v) | Pembekuan fitur | Go-live | Selisih dari 2027-03-01 |
| --- | --- | --- | --- | --- |
| 25 | 68 | 2027-02-17 | 2027-03-20 | +19 hari |
| 30 | 57 | 2027-02-06 | 2027-03-09 | +8 hari |
| ≥ 34,7 (1.698 ÷ 49) | ≤ 49 | ≤ 2027-01-29 | ≤ 2027-03-01 | 0 |

Tabel ini sensitivitas aritmetika, bukan prakiraan; kebutuhan agar 2027-03-01 tetap tercapai: ±34,7 per hari kalender bila 12-12 – 01-29 dipakai seluruhnya (49 hari), atau ±37,7 bila hanya 45 hari yang dialokasikan (1.698 ÷ 45). `v` terukur 2026-12-02 – 12-11 dibandingkan dengan angka ini. Tambahan tenaga (Opsi E, PT-53) menaikkan `v` mulai Fase 3; kontribusinya dihitung di gerbang sesuai §6.5 (kekurangan Fase 3 backend/web = 43,6 − `v_bw` per hari kalender). Dampak biaya, jendela audit dan toko aplikasi (RT-34), serta komitmen pemasaran (R-07) bila go-live mundur ditinjau di gerbang; go-live di sekitar Idulfitri (10–11 Maret 2027 `[SRC-282]`) berisiko pada ketersediaan penyedia luar `[TIDAK DIKETAHUI]`.

#### 6.8.6 Kontradiksi dan butir yang dilaporkan, tidak diselesaikan sendiri

| # | Temuan | Perlakuan |
| --- | --- | --- |
| P-1 | CR-022 "Dampak" di `docs/backlog.md` menulis baseline scope "menjadi Opsi A (Fase 4 dikerjakan) ditambah CR-021", sedangkan tafsiran orkestrator di entri yang sama dan instruksi pembaruan ini menyatakan cakupan tidak diperluas (baseline Opsi B + CR-021) | v0.4 mengikuti Opsi B + CR-021; Opsi A hanya pembanding (§6.2 tabel v0.4, §6.5). Perlu konfirmasi user; bila Opsi A dimaksud, Fase 3 menjadi 1.149 jam-task (60,5 per hari kalender) dan total ±2.307 |
| P-2 | `docs/01-business-case.md` §7.1 dan brief P8.3 (toleransi waktu 0) bertentangan dengan CR-022 | Business case diberi catatan perubahan; P8.3 di brief tidak diubah di tugas ini |
| P-3 | C2-Q7 (libur nasional dan cuti bersama = libur tim) bertentangan dengan arahan 7 hari per minggu | Digantikan di v0.4; riwayat v0.2 dipertahankan |
| P-4 | F1-T19 bergantung pada F1-T16 saja di v0.3, padahal membutuhkan F1-T18 | Diperbaiki di §7.3 |
| P-5 | `docs/STATUS.md` tidak menyebut F1-T81/F1-T82 selesai, sedangkan `docs/backlog.md` CR-021 menyebut "diterapkan 2026-11-19" | Dihitung selesai; angka tanpa keduanya di §6.8.3 |
| P-6 | Jam kerja per hari dan per minggu di akhir pekan `[TIDAK DIKETAHUI]` | Kenaikan kapasitas tidak dikonversi (§6.8.2) |

---

## 7. Fase 1: story, enabler, dan task

### 7.1 Isi Fase 1

| ID | Judul | Jenis | Induk | Catatan |
| --- | --- | --- | --- | --- |
| EN-01 | Repo dan kerangka monorepo | Enabler | ADR-024, KEPUTUSAN-89 | |
| EN-02 | `scripts/verify.sh`, git hook, CI | Enabler | ADR-021, ADR-025, KEPUTUSAN-81, KEPUTUSAN-82; BR-108 | Pembuatan `verify.sh` meminta persetujuan user |
| EN-03 | Lingkungan lokal | Enabler | ADR-019, KEPUTUSAN-88 | LiveKit menyusul di Fase 2 (EN-18) |
| EN-04 | Skema inti dan migrasi | Enabler | ADR-001–ADR-004, ADR-008, KEPUTUSAN-74; CR-014 (bagian DB aplikasi) | Hanya tabel yang dipakai Fase 1 |
| EN-05 | Audit Trail Foundation | Enabler | ADR-005, ADR-009, ADR-010; BR-080, BR-081, BR-107 | |
| EN-06 | Autentikasi dan sesi (realm `pasien`, `admin`) | Enabler | ADR-011; BR-001, BR-060, BR-108 | |
| EN-07 | Otorisasi dan RBAC | Enabler | ADR-012; BR-060, BR-105, BR-108 | |
| EN-08 | Crate dasar, envelope, logging, OpenAPI, test arsitektur | Enabler | ADR-010, ADR-022, ADR-023, KEPUTUSAN-110, KEPUTUSAN-113, KEPUTUSAN-121–KEPUTUSAN-124, KEPUTUSAN-127, KEPUTUSAN-132 | |
| EN-09 | Kriptografi dan kunci data | Enabler | ADR-013; BR-105, BR-108 | |
| EN-10 | Berkas unggahan | Enabler | ADR-018; BR-003 | |
| EN-11 | BFF Next.js dan kerangka backoffice | Enabler | ADR-011 butir 7, ADR-024, KEPUTUSAN-75, KEPUTUSAN-115; BR-060 | |
| EN-12 | Kerangka mobile, `rs_auth`, data di perangkat | Enabler | ADR-021 butir 1, ADR-024, KEPUTUSAN-76, KEPUTUSAN-116; BR-007 | |
| EN-13 | Jalur email (spike, usulan ADR, adaptor) | Enabler | ADR-016, CR-012 (SES API), KEPUTUSAN-183; BR-001, BR-050 | Gap SMTP: §1 |
| EN-14 | Reference implementation dan pengukuran kecepatan | Enabler | `tahap-08-development.md` langkah 5, KEPUTUSAN-181, RT-24, KEPUTUSAN-134 | Wujudnya US-106 → US-201 |
| UI-01 | Token desain dan komponen dasar web + galeri state | UI | KEPUTUSAN-163, KEPUTUSAN-169, KEPUTUSAN-176; BR-060 | Menutup catatan QA N4 |
| UI-02 | Token desain dan komponen dasar Flutter + test aksesibilitas | UI | KEPUTUSAN-162, KEPUTUSAN-163, KEPUTUSAN-176; BR-007 | |
| UI-03 | Tema login Keycloak Ruang Sehat | UI | ADR-011 butir 7, KEPUTUSAN-175; BR-001 | Pesan akun terkunci menunggu PT-44 |
| US-101 | Registrasi dengan email dan password | Story Must | BR-001, BR-103 | T1 |
| US-103 | Masuk, keluar, atur ulang password | Story Must | BR-001 | T1; AC3 menunggu PT-44 |
| US-104 | Persetujuan pemrosesan data | Story Must | BR-103, BR-104, BR-107 | T1 (EP-16). AC4 (pemberitahuan versi baru) di Fase 2 bersama US-1001. Teks AC1 menunggu L-08/PT-24, struktur dibangun dengan teks bertanda draf |
| US-105 | Profil data diri | Story Must | BR-002 | AC4 (ubah email) T1; catatan QA N3 (formulir alamat, ubah email) |
| US-106 | Unggah NIK dan foto KTP | Story Must | BR-003, BR-004 | T1; reference implementation |
| US-201 | Verifikasi identitas (KTP/KK) | Story Must | BR-004, BR-008 | T1; reference implementation. Pemberitahuan ke pasien di AC2 diverifikasi di Fase 2 (US-1001) |
| US-208 | Akun dan sub-peran admin Ruang Sehat | Story Must | BR-060, BR-080 | T1 (otorisasi) |
| US-1501 | Pencatatan event audit | Story Must | BR-080, BR-081, BR-107 | T1; event Fase 1 (E-01–E-05, E-07, E-10, E-11, E-13) |
| US-1504 | Perlindungan audit log | Story Must | BR-080 | T1 |
| CR-018 | MFA wajib untuk pasien: TOTP wajib dan kode pemulihan (US-101 AC5), atur ulang password tidak melewati OTP (US-103 AC4e), daftar ulang TOTP dan perangkat hilang (US-103 AC7, AC8) | Change request (revisi ADR-011 butir 3 dan 5) | ADR-011 (revisi), NFR-SEC-02, US-101, US-103; BR-001, BR-108 | T1; `[USER] C2-Q9`. Teks AC dari revisi PRD yang dikerjakan paralel. Reset MFA pasien oleh Layanan Pelanggan (US-207 AC5) di Fase 2 `[KEPUTUSAN-217]` |
| CR-017 | Ubah password saat login (US-103 AC6) | Change request | US-103; BR-001; ASVS 6.2.2/6.2.3 L1 `[SRC-265]`; KEPUTUSAN-195, KEPUTUSAN-198 | T1; `[USER] C2-B14`. Realm `pasien` dan `admin` di Fase 1; `nakes` di Fase 2; `rme` di Fase 3 `[KEPUTUSAN-219]`. **v0.3:** realm `pasien`/`admin` pindah ke Fase 2 (F1-T75, F1-T76, F1-T77, F1-T78, F1-T79) `[USER] C21-P4` |
| CR-021 (v0.3) | Situs web, CMS, pendaftaran dan verifikasi perusahaan, akun HR (K1) | Change request | BR-130–BR-136, BR-145; ADR-027, ADR-028; EP-18, US-1901–US-1904, US-1916 | T1/T2 per task; `[USER] C21-P4` (S3). F1-T81–F1-T111 |
| US-1101 (v0.3) | Pengelolaan konten (CMS), dipindah dari Fase 3 | Story Must | BR-061, BR-131 | F1-T91–F1-T95 |
| US-1801, US-1802 (v0.3) | Halaman company profile; konten situs di CMS | Story Must | BR-130, BR-131 | F1-T89, F1-T93–F1-T97, F1-T111 |
| US-1901–US-1904, US-1916 (v0.3) | Pendaftaran, verifikasi, pembuatan perusahaan; akun HR; pengelolaan perusahaan (bagian K1) | Story Must | BR-132, BR-136, BR-145 | T1. US-1904 AC7–AC9 (reset MFA HR) di Fase 2 `[KEPUTUSAN-179]` |

### 7.2 Tinjauan Tech Lead: urutan dan alasannya

1. **Mesin dan keputusan teknis pembuka dulu** (F1-T01–T03). Toolchain harus dapat dijalankan di mesin pengembangan sebelum task kode pertama (runbook Tahap 8 langkah 0). Dua gap teknis (jalur email Keycloak, penyimpanan berkas lokal) diselesaikan sebagai spike karena memblokir US-101, US-103, dan US-106.
2. **Commit kode pertama wajib menyertakan `scripts/verify.sh`** (pola bootstrap di runbook Tahap 8), jadi F1-T05 mendahului semua task kode lain.
3. **Database sebelum layanan**: bootstrap role → fungsi `util` → test katalog hak role → tabel. Test katalog (F1-T15) dibuat sebelum tabel bisnis, sehingga setiap migrasi berikutnya langsung diuji hak role dan trigger penolaknya (ADR-008, ADR-021 butir 4).
4. **Audit, autentikasi, otorisasi, dan kriptografi sebelum story**. Semua story Fase 1 menulis event audit dan membaca konteks autentikasi; US-106 butuh enkripsi NIK dan berkas.
5. **Reference implementation sebelum story lain yang serupa**. US-106 → US-201 dibangun lebih dulu dan didokumentasikan (F1-T66) sebelum US-105, US-208, dan registrasi diselesaikan, supaya story lain mengikuti polanya (Definition of Done "mengikuti pola reference implementation").
6. **Tiga jalur paralel setelah fondasi**: backend Rust, web Next.js, mobile Flutter. Jalur mobile dikerjakan mobile developer `[USER] C2-Q3` begitu kontrak API (OpenAPI) tersedia.
7. **MFA pasien dan ubah password menempel ke fondasi autentikasi, bukan ke reference implementation** (v0.2). Konfigurasi MFA realm `pasien` (F1-T72) langsung setelah realm sebagai kode (F1-T31), supaya semua test login pasien sejak awal sudah melewati OTP. Atur ulang password dengan OTP (F1-T74) menunggu jalur email (F1-T36) dan harness test Keycloak (F1-T55). Daftar ulang TOTP (F1-T75) dan ubah password (F1-T76) menunggu penolakan sesi (F1-T33) dan pengambil event (F1-T34), karena opsi "keluar dari perangkat lain" dan E-04 bergantung pada keduanya. Uji end-to-end MFA (F1-T80) menjadi prasyarat demo MVP (F1-T70) `[KEPUTUSAN-216]`.

### 7.3 Daftar task Fase 1

Kolom: **Jam** = ukuran 2/3/4 jam-task. **Kelas** = tingkat ketelitian (`aturan-kerja.md` §1); T1 memakai Opus effort high dan review penuh, T2 memakai Sonnet medium dengan satu reviewer. **Review** = reviewer selain `code-reviewer` (Sec = keamanan, DBA). Semua task T1 wajib test negatif (`docs/coding-standards.md` §7.3).

| ID | Task | Untuk | Jam | Bergantung pada | Kelas | Review |
| --- | --- | --- | --- | --- | --- | --- |
| F1-T01 | Periksa mesin pengembangan: Rust 1.100.0, Node 26, pnpm, Flutter 3.50, Docker dapat dijalankan; minta persetujuan sebelum memasang; bila tidak cocok, usulkan VPS pengembangan Linux | EN-01 | 2 | Checkpoint 2 | T2 | — |
| F1-T02 | Spike jalur email Keycloak (SMTP) vs Amazon SES API, termasuk verifikasi pernyataan lokasi data SES (stack §13.2 butir 18); hasil berupa usulan ADR ke user | EN-13 | 3 | F1-T01 | T1 | Sec |
| F1-T03 | Spike penyimpanan berkas untuk lokal/uji dan verifikasi NEO Object Storage (endpoint HTTPS, presigned PUT, batas ukuran, versioning; stack §13.2 butir 8); pilihan emulator S3 hanya dengan cek registry dan lisensi | EN-10 | 3 | F1-T01 | T2 | — |
| F1-T04 | Struktur monorepo (KEPUTUSAN-89), `rust-toolchain.toml`, workspace Cargo, `.gitignore`, `.env.example`; verifikasi lisensi Node.js dan PostgreSQL (stack §13.2 butir 19) | EN-01 | 3 | F1-T01 | T2 | — |
| F1-T05 | `scripts/verify.sh` awal (langkah berjenjang per komponen, "BELUM DITERAPKAN" dihitung di ringkasan) dan sambungan git hook; tawarkan hook Claude Code; persetujuan user | EN-02 | 3 | F1-T04 | T1 | Sec |
| F1-T06 | CI GitHub Actions: `verify.sh`, gitleaks, osv-scanner, cargo-deny, cargo-audit; versi action dipin | EN-02 | 3 | F1-T05 | T1 | Sec |
| F1-T07 | Compose lokal: PostgreSQL 18 tiga klaster, Valkey, Keycloak, OpenBao dev, Swagger UI | EN-03 | 3 | F1-T04, F1-T05 (commit pertama wajib memuat `verify.sh`) | T2 | — |
| F1-T08 | Crate `foundation`: `Clock`/`FixedClock`, `new_id()` UUIDv7, `Idr` terperiksa, `CorrelationId`, `Sensitive<T>` | EN-08 | 3 | F1-T05 | T2 | — |
| F1-T09 | Crate `envelope`: envelope ADR-023, `ResponseCode`, pemetaan galat → HTTP; pemeriksaan registry `response_code` | EN-08 | 3 | F1-T08 | T2 | — |
| F1-T10 | Konfigurasi bertipe (KEPUTUSAN-132), log JSON dengan redaksi, aliran security log (KEPUTUSAN-124), middleware korelasi (KEPUTUSAN-123) | EN-08 | 4 | F1-T08 | T2 | Sec |
| F1-T11 | Kerangka `rs-backend` (subcommand `api`, `worker`, `audit-relay`, `audit-verify`), endpoint kesehatan, pembangkitan OpenAPI (utoipa) dan pembanding berkas `openapi/` | EN-08 | 4 | F1-T09, F1-T10 | T2 | — |
| F1-T12 | Test arsitektur: `module-tables.toml` + pemeriksa, lint larangan `unwrap`/`expect` (nama lint diverifikasi), larangan jam sistem dan aws-lc-rs langsung, pola log terlarang | EN-08 | 3 | F1-T11 | T2 | — |
| F1-T13 | `db/roles/` bootstrap idempoten tiga klaster: role, skema, ekstensi, pencabutan PUBLIC, tanpa password | EN-04 | 3 | F1-T07 | T1 | DBA, Sec |
| F1-T14 | Migrasi fungsi bersama skema `util` (DB aplikasi dan audit) dan penjaga di awal berkas migrasi (skema fisik §4.2, §11.2) | EN-04 | 4 | F1-T13 | T1 | DBA, Sec |
| F1-T15 | `verify.sh` langkah 2: migrasi ke PostgreSQL 18 kosong + test katalog wajib (hak role per kelas, trigger penolak, skema fisik §12) | EN-04 | 4 | F1-T14 | T1 | DBA, Sec |
| F1-T16 | Migrasi A1: `account`, `rs_admin_role`, `session_revocation` + hak role | EN-04 | 3 | F1-T15 | T1 | DBA, Sec |
| F1-T17 | Migrasi A19: `audit_outbox`, `job`, `upload_slot`, `stored_file`, `data_key`, dan `data_key_retention_anchor` (CR-014 bagian DB aplikasi; data dictionary A19 diperbarui di task yang sama) + hak role | EN-04 | 4 | F1-T15 | T1 | DBA, Sec |
| F1-T18 | Migrasi A2 + A6.7: `patient`, `care_recipient`, `address`, `identity_document`, `identity_verification_request` (+ `_status_event`, `_state`), `account_owner_nik_registry`, `reason_code` + hak role | EN-04 | 4 | F1-T16, F1-T17 | T1 | DBA, Sec |
| F1-T19 | Migrasi A3: `consent_text_version`, `consent_item`, `consent_record` + hak role | EN-04 | 2 | F1-T16, F1-T18 (v0.4: F1-T18 ditambahkan, `consent_record` memakai FK ke `patient`/`care_recipient`) | T1 | DBA, Sec |
| F1-T20 | Crate `platform-jobs`: tabel job, `FOR UPDATE SKIP LOCKED`, kunci job tunggal `pg_try_advisory_lock`, idempotensi | EN-05 | 4 | F1-T17 | T1 | DBA |
| F1-T21 | Migrasi DB audit B1–B6 (partisi bulanan, kunci, subjek, jangkar, verifikasi, manifest arsip), role `audit_relay`/`audit_writer`/`audit_verifier`, partisi ke depan | EN-05 | 4 | F1-T15 | T1 | DBA, Sec |
| F1-T22 | Crate `audit` (1): tipe E-01–E-17, field PRD §7.2, masking §7.3; test field terlarang tidak pernah masuk | EN-05 | 4 | F1-T08 | T1 | Sec |
| F1-T23 | Crate `audit` (2): tulis `audit_outbox` di transaksi pemanggil; gagal tulis membatalkan aksi | EN-05 | 3 | F1-T22, F1-T17 | T1 | DBA, Sec |
| F1-T24 | `audit-relay`: ambil outbox berurutan, `audit_event` dengan `seq`/`prev_hash`/`event_hash`, hapus outbox terkirim; satu relay aktif; test dua relay | EN-05 | 4 | F1-T21, F1-T23, F1-T20, F1-T26 | T1 | DBA, Sec |
| F1-T25 | `audit-verify`: verifikasi rantai harian dan jangkar kepala rantai (lokal: berkas; staging: Object Lock NEO); test perubahan satu event terdeteksi | EN-05 | 4 | F1-T24 | T1 | DBA, Sec |
| F1-T26 | Crate `crypto` (1): AES-256-GCM dengan AAD (KEPUTUSAN-130), HMAC-SHA256, SHA-256, RNG lewat aws-lc-rs (API diverifikasi); format ciphertext v1; test ciphertext diubah/dipindah gagal | EN-09 | 4 | F1-T08 | T1 | Sec |
| F1-T27 | Crate `crypto` (2): klien Transit OpenBao (bungkus/buka DEK), wadah `data_key`, cache DEK berbatas waktu, pemeriksaan `container_type`; test DEK dimusnahkan | EN-09 | 4 | F1-T26, F1-T17, F1-T07 | T1 | Sec |
| F1-T28 | Blind index NIK (kunci dari OpenBao, `nik_bidx_key_version`) | EN-09 | 2 | F1-T27 | T1 | Sec |
| F1-T29 | Modul `platform` berkas (1): upload slot, URL bertanda tangan sesuai hasil F1-T03, validasi isi JPG/PNG dan ukuran, buang metadata, `stored_file` | EN-10 | 4 | F1-T03, F1-T17, F1-T27 | T1 | Sec |
| F1-T30 | Berkas (2): URL unduh berumur pendek hanya setelah otorisasi dan event E-05 tercatat; test negatif jenis palsu, ukuran, URL tanpa izin | EN-10 | 3 | F1-T29, F1-T37 | T1 | Sec |
| F1-T31 | Realm `pasien` dan `admin` sebagai kode (keycloak-config-cli): brute force 5 gagal/15 menit, parameter Argon2, Revoke Refresh Token, MFA TOTP wajib admin, sesi admin idle 30 menit/mutlak 12 jam; opsi Keycloak yang belum terverifikasi dicek (stack §13.2 butir 11) | EN-06 | 4 | F1-T07 | T1 | Sec |
| F1-T32 | Validasi token backend: JWKS per realm di-cache, daftar izin algoritma, issuer, audience (fungsi jsonwebtoken diverifikasi); ekstraktor konteks autentikasi; test token realm lain, audience/issuer salah, kedaluwarsa, algoritma di luar daftar | EN-06 | 4 | F1-T31, F1-T11 | T1 | Sec |
| F1-T33 | Penolakan sesi seketika: status akun dan `session_revocation` (Valkey + cadangan DB, RT-18); kompatibilitas crate `redis` dengan Valkey dibuktikan test; KU-10 | EN-06 | 3 | F1-T32, F1-T16 | T1 | Sec |
| F1-T34 | Pengambil event Keycloak (E-01–E-04) → `audit_outbox`, job tunggal; peringatan keterlambatan (ADR-011 butir 10) | EN-06 | 4 | F1-T31, F1-T23, F1-T20 | T1 | Sec |
| F1-T35 | Tema login Keycloak (login, lupa password, OTP, aktivasi undangan), pesan galat umum sesuai keputusan PT-44, token desain dari UI-01 | UI-03 | 4 | F1-T31, F1-T40 | T1 | Sec |
| F1-T36 | Jalur email sesuai ADR hasil F1-T02: konfigurasi email Keycloak, adaptor SES API (akun AWS khusus email), penangkap email untuk test | EN-13 | 4 | F1-T02 + persetujuan ADR | T1 | Sec |
| F1-T37 | Crate `authz`: registri kebijakan per endpoint, pembungkus router wajib kebijakan + test saat start (KEPUTUSAN-127), sub-peran admin (UP-18), default deny, 404 untuk objek terlarang | EN-07 | 4 | F1-T11, F1-T32 | T1 | Sec |
| F1-T38 | `authz` relasi: kepemilikan pasien/penerima layanan; akses ditolak ke data sensitif tercatat hasil gagal (KU-03) | EN-07 | 3 | F1-T37, F1-T23 | T1 | Sec |
| F1-T39 | Workspace web: pnpm, TypeScript ketat, pustaka ESLint dan test unit dipilih dengan cek registry (SRC), `web/packages/api-client` dari `openapi/`; `verify.sh` langkah 5 | EN-11 | 4 | F1-T05, F1-T11 | T2 | — |
| F1-T40 | `web/packages/ui`: token CSS, skrip kontras K1–K13/N1–N4 tanpa dependensi, komponen dasar Fase 1 dengan semua state §4 termasuk memuat dan hover, halaman galeri state (catatan QA N4) | UI-01 | 4 | F1-T39 | T2 | — |
| F1-T41 | `bff-auth` (1): OIDC Authorization Code klien rahasia (openid-client), sesi Valkey (ioredis), token JWE (jose, kunci OpenBao) terikat ID sesi; fungsi pustaka diverifikasi | EN-11 | 4 | F1-T39, F1-T31, F1-T07 | T1 | Sec |
| F1-T42 | `bff-auth` (2): cookie `HttpOnly`/`Secure`/`SameSite`, CSRF, Route Handler `bff/[...path]`, idle 30 menit + dialog peringatan (KEPUTUSAN-170), logout; test tanpa CSRF ditolak dan token tidak sampai ke browser | EN-11 | 4 | F1-T41 | T1 | Sec |
| F1-T43 | Kerangka backoffice: tata letak, layar akses ditolak dan akun nonaktif (C01), galat dengan kode referensi (KEPUTUSAN-172) | EN-11 | 3 | F1-T42, F1-T40 | T2 | — |
| F1-T44 | Pub workspace Flutter + kerangka `apps/pasien` (go_router, riverpod, flavor lingkungan); `verify.sh` langkah 6 | EN-12 | 3 | F1-T05 | T2 | — |
| F1-T45 | `rs_design`: token Dart, komponen dasar Fase 1, test guideline aksesibilitas `flutter_test` (KEPUTUSAN-176) | UI-02 | 4 | F1-T44 | T2 | — |
| F1-T46 | `rs_core` (envelope, `response_code`, `Clock`, kode referensi) dan `rs_api_client` (generator Dart dipilih dengan cek registry, SRC) | EN-12 | 4 | F1-T44, F1-T11 | T2 | — |
| F1-T47 | `rs_auth`: flutter_appauth (PKCE, browser sistem), flutter_secure_storage, penyegaran token, keluar | EN-12 | 4 | F1-T44, F1-T31 | T1 | Sec |
| F1-T48 | Data di perangkat (ADR-021 butir 1): pengecualian cadangan, layar privasi saat di latar belakang (mekanisme platform diverifikasi), daftar izin host, inventaris dependensi mobile | EN-12 | 3 | F1-T44 | T2 | Sec |
| F1-T49 | US-104: versi teks dan butir persetujuan (seed versi draf bertanda "menunggu L-08/PT-24"), endpoint teks aktif, E-13 | US-104 | 3 | F1-T19, F1-T37, F1-T23 | T1 | Sec |
| F1-T50 | US-101 (1): registrasi native, persetujuan wajib (AC4), user Keycloak lewat Admin REST + kompensasi bila langkah lain gagal (ADR-011 butir 6), `consent_record`, status "email belum diverifikasi", E-04/E-13, email terdaftar (AC3) | US-101 | 4 | F1-T49, F1-T36, F1-T33 | T1 | Sec |
| F1-T51 | US-101 (2): verifikasi email (berlaku, kedaluwarsa, sudah dipakai, kirim ulang; AC2) | US-101 | 3 | F1-T50 | T1 | Sec |
| F1-T52 | Mobile: registrasi dua langkah + layar persetujuan tujuh butir (A01 layar 1–3), kotak tidak tercentang otomatis, KU-01 | US-101, US-104 | 4 | F1-T45, F1-T46, F1-T49 | T2 | — |
| F1-T53 | Mobile: layar verifikasi email dan kirim ulang (A01 layar 4) | US-101 | 3 | F1-T52, F1-T51 | T2 | — |
| F1-T54 | Mobile: masuk lewat browser sistem, keluar (sesi server diakhiri, token lama ditolak; AC5), atur ulang password (AC4) | US-103 | 4 | F1-T47, F1-T35, F1-T33 | T1 | Sec |
| F1-T55 | Test otomatis Keycloak: penguncian 5 gagal/15 menit walau password benar (AC3) dengan pesan sesuai PT-44, E-02 tercatat, pesan atur ulang sama untuk email terdaftar/tidak (AC4) | US-103 | 3 | F1-T31, F1-T34 | T1 | Sec |
| F1-T56 | US-105 backend: profil (Master, soft delete), alamat jamak (AC2), ubah nama/tanggal lahir mengembalikan status identitas (AC3), E-10 | US-105 | 4 | F1-T18, F1-T38, F1-T66 | T2 | — |
| F1-T57 (**→ Fase 2, v0.3**) | US-105 AC4: ubah email lewat Keycloak + verifikasi + pemberitahuan ke email lama, autentikasi ulang (ADR-011 butir 9) | US-105 | 3 | F1-T56, F1-T36 | T1 | Sec |
| F1-T58 | Mobile: profil, daftar dan formulir alamat, ubah email; tambahkan deskripsi D-A03a di design guideline untuk formulir alamat dan ubah email (catatan QA N3) | US-105 | 4 | F1-T45, F1-T46, F1-T56 | T2 | — |
| F1-T59 | RI (1) domain: status AB-05 dan transisi lewat jalur tulis status ADR-008 (R/P), aturan UP-40; unit test | US-106, EN-14 | 3 | F1-T18 | T1 | DBA |
| F1-T60 | RI (2) service + repository pengajuan identitas: NIK 16 digit terenkripsi + blind index, foto KTP lewat upload slot, NIK pemilik akun ganda ditolak (AC4), foto terkunci saat menunggu (AC5), batal pengiriman, E-10 | US-106, EN-14 | 4 | F1-T59, F1-T28, F1-T29, F1-T38 | T1 | Sec, DBA |
| F1-T61 | RI (3) handler, OpenAPI, `response_code` baru terdaftar, contract test, lima skenario penolakan endpoint (coding standards §7.3) | US-106, EN-14 | 3 | F1-T60 | T1 | Sec |
| F1-T62 | RI (4) Verifikator: antrean terlama, buka foto (E-05 dulu), setujui/tolak dengan `reason_code`, pemeriksaan KK, keputusan pertama yang berlaku (AC5), NIK sudah dipakai (AC7), E-07 | US-201, EN-14 | 4 | F1-T61, F1-T30 | T1 | Sec, DBA |
| F1-T63 | RI (5) mobile: unggah NIK + KTP (kamera/galeri), status AB-05, alasan penolakan, unggah ulang (A03) | US-106 | 4 | F1-T58, F1-T61 | T2 | — |
| F1-T64 | RI (6) backoffice panel Verifikator (C02): daftar + panel detail, jam berlabel WIB (catatan QA N5b), foto hanya di panel Verifikator; e2e Playwright | US-201 | 4 | F1-T43, F1-T62 | T2 | Sec |
| F1-T65 | US-208: bootstrap Super Admin pertama lewat prosedur terkontrol yang tercatat; kelola akun admin dan sub-peran (AC1–AC3), MFA saat pertama masuk, E-10/E-11; test Super Admin terakhir dan ubah peran sendiri | US-208 | 4 | F1-T37, F1-T31, F1-T23 | T1 | Sec |
| F1-T66 | RI (7) dokumentasi pola reference implementation (lokasi di `CLAUDE.md`, checklist pola), pengukuran kecepatan awal (RT-24) dan baseline cakupan (KEPUTUSAN-134) | EN-14 | 2 | F1-T61, F1-T62 | T2 | — |
| F1-T67 (**→ Fase 2, v0.3**) | Backoffice: layar kelola admin dan sub-peran (bagian admin RS dari D-C05) | US-208 | 3 | F1-T43, F1-T65 | T2 | — |
| F1-T68 | US-1504: tidak ada endpoint ubah/hapus audit; permintaan ubah/hapus ditolak dan dicatat sebagai kejadian keamanan; test `UPDATE`/`DELETE`/`TRUNCATE` di `audit_event` ditolak | US-1504 | 2 | F1-T21, F1-T37 | T1 | DBA, Sec |
| F1-T69 | US-1501 AC1–AC3: field wajib §7.2 untuk semua event Fase 1; aksi pengguna batal bila event gagal dicatat, aksi sistem diulang dari antrean, peringatan keamanan | US-1501 | 3 | F1-T24, F1-T62 | T1 | Sec |
| F1-T70 | Uji end-to-end MVP: daftar → verifikasi email → konfigurasi TOTP dan simpan kode pemulihan → masuk dengan OTP (CR-018, v0.2) → unggah KTP → Verifikator menyetujui → status Disetujui; rantai audit terverifikasi | Fase 1 | 3 | F1-T53, F1-T54, F1-T63, F1-T64, F1-T69, F1-T80 | T2 | — |
| F1-T71 | Penutupan Fase 1: functional test QA dari `docs/test-plan.md`, consistency review (Opus high), `docs/fase-1-laporan.md` berisi kecepatan terukur untuk gerbang re-plan (total dan per jalur `v_mobile`/`v_bw`, §6.5) | Fase 1 | 3 | F1-T70 | T2 | — |
| F1-T72 | CR-018: realm `pasien` sebagai kode dengan MFA TOTP wajib (US-101 AC5, US-103 AC1, AC1b; `[KEPUTUSAN-202]`): required action Configure OTP pada login pertama setelah email terverifikasi, alur browser yang selalu meminta OTP, kebijakan OTP sama dengan realm lain; kode pemulihan wajib dibuat saat pendaftaran TOTP pertama (fitur `recovery-codes` `[SRC-68]`; mekanisme mewajibkannya diverifikasi, bila tidak bisa dibawa ke user, `[KEPUTUSAN-203]`); penguncian berlaku untuk OTP dan kode pemulihan `[SRC-273]`; periksa apakah token memuat klaim tingkat autentikasi yang dapat diperiksa backend, hasilnya untuk reviewer keamanan. Test negatif: login pasien tanpa OTP tidak menerbitkan token, OTP salah dihitung penguncian, kode pemulihan yang sudah dipakai ditolak, akun yang membatalkan pendaftaran TOTP tidak masuk | EN-06, US-101, US-103 | 4 | F1-T31 | T1 | Sec |
| F1-T73 | CR-018: tema Keycloak untuk pendaftaran TOTP pasien (kode QR dan kunci teks, 12 kode pemulihan ditampilkan sekali dengan petunjuk menyimpannya, masuk dengan OTP atau kode pemulihan), petunjuk menghubungi Layanan Pelanggan di layar kode TOTP (US-103 AC8); pesan galat seragam UP-73; token desain, kontras, dan target sentuh guideline | UI-03, US-101, US-103 | 3 | F1-T72, F1-T35 | T1 | Sec |
| F1-T74 | CR-018: alur Reset Credentials realm `pasien` meminta kode TOTP atau kode pemulihan sebelum password baru berlaku dan sebelum akses diberikan (US-103 AC4e, `[KEPUTUSAN-199]`); penghentian semua sesi setelah atur ulang tetap berjalan (`[KEPUTUSAN-198]`). Test negatif otomatis: tautan atur ulang tanpa OTP tidak mengubah password dan tidak memberi akses, OTP salah dihitung penguncian, tautan dipakai ulang ditolak | US-103 | 3 | F1-T72, F1-T36, F1-T55 | T1 | Sec |
| F1-T75 (**→ Fase 2, v0.3**) | CR-018: daftar ulang TOTP untuk realm `pasien` dan `admin` (US-103 AC7; AC8 bagian kode pemulihan; `[KEPUTUSAN-204]` butir 1). Mekanisme Keycloak diverifikasi lebih dulu dan dicatat di `docs/sources.md`. Isi: autentikasi ulang (`[KEPUTUSAN-195]`), TOTP lama tidak berlaku setelah TOTP baru tersimpan, kode pemulihan baru menggantikan yang lama, opsi "keluar dari perangkat lain" tercentang bawaan (`[KEPUTUSAN-198]`), E-04 tanpa rahasia. Test negatif: TOTP lama dan kode pemulihan lama ditolak setelah daftar ulang, daftar ulang tanpa autentikasi ulang ditolak | US-103 | 4 | F1-T72, F1-T33, F1-T34 | T1 | Sec |
| F1-T76 (**→ Fase 2, v0.3**) | CR-017: ubah password saat login untuk realm `pasien` dan `admin` (US-103 AC6). Mekanisme Keycloak (required action atas permintaan aplikasi atau konsol akun) diverifikasi lebih dulu dan dicatat di `docs/sources.md`. Isi: password lama wajib, autentikasi ulang (`[KEPUTUSAN-195]`), NFR-SEC-03, opsi "keluar dari perangkat lain" tercentang bawaan yang mengakhiri sesi Keycloak lain dan menulis `session_revocation` (`[KEPUTUSAN-198]`), E-04 tanpa nilai kredensial. Test negatif: password lama salah, password baru melanggar kebijakan, autentikasi ulang kedaluwarsa, token sesi lain ditolak setelah opsi dicentang | US-103 | 4 | F1-T31, F1-T33, F1-T34 | T1 | Sec |
| F1-T77 (**→ Fase 2, v0.3**) | Mobile CR-018: layar pengantar sebelum browser sistem dibuka untuk pendaftaran TOTP pertama, penanganan batal atau kembali dari browser (US-101 AC5), layar "Keamanan akun" (status MFA, daftar ulang TOTP, petunjuk kode pemulihan), dan pengarahan ke daftar ulang TOTP setelah masuk dengan kode pemulihan (US-103 AC7, AC8) | US-101, US-103 | 4 | F1-T54, F1-T73, F1-T75 | T1 | Sec |
| F1-T78 (**→ Fase 2, v0.3**) | Mobile CR-017: ubah password dari layar "Keamanan akun" sesuai mekanisme F1-T76, opsi keluar dari perangkat lain tercentang bawaan, pesan berhasil dan galat | US-103 | 3 | F1-T77, F1-T76 | T1 | Sec |
| F1-T79 (**→ Fase 2, v0.3**) | Web CR-017 dan CR-018: menu ubah password dan daftar ulang TOTP di kerangka backoffice lewat BFF (realm `admin`); sesi BFF lain diakhiri bila opsi dicentang; e2e Playwright termasuk sesi lama ditolak | US-103 | 4 | F1-T42, F1-T43, F1-T75, F1-T76 | T1 | Sec |
| F1-T80 | Uji end-to-end CR-018 dan CR-017 di aplikasi pasien: daftar → verifikasi email → daftar TOTP → simpan kode pemulihan → masuk dengan OTP → masuk dengan kode pemulihan (kode yang sama ditolak saat dipakai ulang) dan diarahkan ke daftar ulang TOTP → ubah password dengan keluar dari perangkat lain → atur ulang password meminta OTP; pembangkit TOTP di test; event E-01–E-04 tercatat. **v0.3:** tanpa langkah ubah password dan daftar ulang TOTP (pindah ke Fase 2 bersama F1-T75–F1-T79); bergantung pada F1-T53, F1-T73, F1-T74 saja | US-101, US-103 | 3 | F1-T53, F1-T73, F1-T74 (v0.2: juga F1-T78, F1-T79) | T1 | Sec |
| F1-T81 | CR-021 dokumen gate (1): BRD v1.4 dan PRD v1.3 (BR-130–BR-145, EP-18, EP-19, AB-80–AB-87, PB-23–PB-25, E-18, UP-79–UP-87, L-16–L-21) | CR-021 | 4 | Persetujuan user `[USER] C21-P4` | T1 | fact-checker, compliance-reviewer |
| F1-T82 | CR-021 dokumen gate (2): data dictionary A20 (rancangan, catatan R2-01), revisi ADR-006/011/018/024, threat model v1.3, phase plan v0.3, backlog, traceability, daftar modul kritis `CLAUDE.md` | CR-021 | 4 | F1-T81 | T1 | DBA, Sec |
| F1-T83 | Desain: deskripsi terstruktur dan wireframe sembilan halaman situs (responsif, aksesibilitas guideline §7) | US-1801 | 3 | F1-T81 | T2 | qa-engineer |
| F1-T84 | Desain: member area HR (daftar, menunggu verifikasi, dasbor, kelola HR) dan layar backoffice verifikasi perusahaan serta CMS kanal situs | US-1901–US-1904, US-1802 | 3 | F1-T81 | T2 | qa-engineer |
| F1-T85 | Realm `korporat` sebagai kode: MFA TOTP wajib + kode pemulihan (pola F1-T72), brute force 5/15 menit, sesi web 30 menit/12 jam, Revoke Refresh Token, klien rahasia BFF; test negatif login tanpa OTP, kode pemulihan bekas ditolak. Daftar ulang TOTP setelah masuk dengan kode pemulihan (US-1904 AC7) memakai mekanisme F1-T75, sehingga ikut pindah ke Fase 2 | US-1904, EN-06 | 4 | F1-T31, F1-T72, F1-T82 | T1 | Sec |
| F1-T86 | Validasi token realm `korporat` di backend (issuer, audience `/v1/corporate`), ekstraktor konteks HR; test token dan audience lain ditolak dua arah | US-1904 | 3 | F1-T32, F1-T85 | T1 | Sec |
| F1-T87 | Tema login realm `korporat` (memakai ulang F1-T35/F1-T73), pesan galat seragam UP-73 | US-1904, UI-03 | 2 | F1-T35, F1-T73, F1-T85 | T1 | Sec |
| F1-T88 | Atur ulang password ber-OTP dan uji penguncian di realm `korporat` (pola F1-T55/F1-T74), test negatif | US-1904 | 3 | F1-T55, F1-T74, F1-T85 | T1 | Sec |
| F1-T89 | `web/situs`: kerangka Next.js publik tanpa sesi (standalone, pengerasan ADR-024, CSP tanpa skrip inline dan pihak ketiga), tanpa identitas OpenBao dan Valkey; jaringan Docker sendiri hanya ke `/v1/public/*`; `verify.sh` mencakup aplikasi baru | US-1801 | 3 | F1-T39, F1-T40 | T2 | Sec |
| F1-T90 | `web/korporat`: kerangka + BFF realm `korporat` (cookie `__Host-`, CSRF synchronizer token, idle 30 menit, `no-store` untuk data peserta); test sesi korporat tidak dapat memanggil API backoffice/portal/pasien, token tidak sampai browser | US-1904, EN-11 | 4 | F1-T42, F1-T86, F1-T40 | T1 | Sec |
| F1-T91 | Migrasi `cms_content` dengan kanal dan halaman situs + hak role | US-1101, US-1802 | 3 | F1-T15, F1-T82 | T1 | DBA |
| F1-T92 | Backend CMS (US-1101 AC1–AC4, US-1802 AC1–AC4): draf, pratinjau, jadwal, arsip, soft delete dan pulihkan (E-16), kebijakan sub-peran Konten | US-1101, US-1802 | 4 | F1-T91, F1-T37, F1-T23 | T2 | Sec |
| F1-T93 | Gambar konten lewat upload slot, teks alternatif wajib; endpoint publik gambar tayang lewat backend (bukan bucket publik), batas laju | US-1802 | 3 | F1-T29, F1-T92 | T2 | Sec |
| F1-T94 | Endpoint konten publik tanpa login: hanya konten Tayang, batas laju, ADR-012 butir 3; OpenAPI dan contract test; test draf tidak bocor | US-1801, US-1802 | 3 | F1-T92 | T2 | Sec |
| F1-T95 | Backoffice layar CMS (D-C07 + kanal situs), pratinjau di iframe `sandbox` tanpa `allow-same-origin`; e2e termasuk payload XSS di pratinjau | US-1101, US-1802 | 4 | F1-T43, F1-T92, F1-T93, F1-T84, F1-T111 | T2 | Sec |
| F1-T96 | Situs: Beranda, Tentang Kami, Layanan, Untuk Perusahaan dari endpoint publik dengan render aman; halaman tidak ditemukan dan galat umum | US-1801 | 4 | F1-T89, F1-T94, F1-T83, F1-T111 | T2 | — |
| F1-T97 | Situs: Rekanan, Artikel, Kontak, Unduh aplikasi, Kebijakan privasi dan syarat (bertanda draf); uji aksesibilitas otomatis dan tanpa cookie sesi | US-1801 | 4 | F1-T96 | T2 | — |
| F1-T98 | Migrasi A20 (1): `corporate_client` (UQp NIB), `corporate_client_document` (termasuk bukti kewenangan), `corporate_verification_decision` + hak role + test katalog | US-1901, US-1902 | 4 | F1-T15, F1-T17, F1-T82 | T1 | DBA, Sec |
| F1-T99 | Migrasi A20 (2): `corporate_member` (satu keanggotaan aktif); *expand* `account` (`corporate_admin`, `korporat`) + hak role | US-1904 | 3 | F1-T16, F1-T98 | T1 | DBA, Sec |
| F1-T100 | `authz` tenant perusahaan: `CorporateScope` dari sesi, kebijakan HR, 404 objek perusahaan lain, harness isolasi otomatis semua endpoint `/v1/corporate` (H-18) | US-1904 AC3, EN-07 | 4 | F1-T37, F1-T38, F1-T99, F1-T86 | T1 | Sec |
| F1-T101 | Backend pendaftaran perusahaan mandiri (US-1901 AC1, AC3–AC7): akun HR lewat Admin REST + kompensasi, bukti persetujuan, pesan seragam email terdaftar, NIB ganda ditolak, batas laju; E-04/E-10/E-13. Format NIB/NPWP diverifikasi ke sumber resmi lebih dulu | US-1901 | 4 | F1-T99, F1-T36, F1-T49, F1-T85, F1-T33 | T1 | Sec |
| F1-T102 | Unggah dokumen NIB, NPWP, dan bukti kewenangan (US-1901 AC2) lewat upload slot, validasi isi, penetralan PDF ADR-018 | US-1901 | 3 | F1-T29, F1-T101 | T1 | Sec |
| F1-T103 | Web korporat: formulir daftar bertahap, unggah dokumen, layar menunggu verifikasi dan ditolak; e2e | US-1901 | 4 | F1-T90, F1-T101, F1-T102, F1-T84 | T2 | Sec |
| F1-T104 | Backend verifikasi perusahaan oleh Verifikator (US-1902): antrean terlama, buka dokumen (E-05), setujui/tolak dengan `reason_code`, keputusan pertama berlaku, E-10; test negatif HR dan sub-peran lain | US-1902 | 4 | F1-T98, F1-T100, F1-T65 | T1 | Sec, DBA |
| F1-T105 | Backend perusahaan dibuatkan Verifikator (US-1903): perusahaan Aktif + undangan aktivasi HR pertama; test undangan kedaluwarsa dan dipakai ulang, NIB ganda | US-1903 | 3 | F1-T104, F1-T36 | T1 | Sec |
| F1-T106 | Backoffice: antrean verifikasi perusahaan, panel dokumen, buat perusahaan, daftar perusahaan dan anggota HR (US-1902, US-1903, US-1916 bagian K1); e2e | US-1902, US-1903, US-1916 | 4 | F1-T43, F1-T104, F1-T105, F1-T84 | T2 | Sec |
| F1-T107 | Backend akun HR (US-1904 AC2, AC5): undang HR lain, nonaktifkan, HR terakhir, penolakan sesi seketika, E-10/E-11 | US-1904 | 4 | F1-T100, F1-T33, F1-T36 | T1 | Sec |
| F1-T108 | Web korporat: masuk/keluar, dasbor, kelola HR, layar akses ditolak dan akun nonaktif; e2e termasuk idle 30 menit | US-1904 | 4 | F1-T90, F1-T107, F1-T87 | T2 | Sec |
| F1-T109 | Event audit korporat (E-04, E-05, E-10, E-11, E-13, E-16) dengan field PRD §7.2; test field terlarang | US-1501 | 3 | F1-T22, F1-T101, F1-T104, F1-T107 | T1 | Sec |
| F1-T110 | Uji end-to-end CR-021 Fase 1: situs dari CMS → daftar perusahaan → verifikasi email → TOTP → Verifikator menyetujui → HR masuk → undang HR kedua; isolasi dua perusahaan; rantai audit terverifikasi | CR-021 | 3 | F1-T97, F1-T95, F1-T103, F1-T106, F1-T108, F1-T109, F1-T88 | T1 | Sec |
| F1-T111 | Render aman konten CMS (M-40): validasi backend saat simpan (HTML mentah ditolak/dinetralkan, daftar izin skema URL, gambar hanya dari endpoint publik), satu aturan render Markdown untuk situs dan pratinjau, CSP backoffice tanpa skrip inline; pustaka dipilih dengan cek registry dan dicatat di `docs/sources.md`; test payload XSS | US-1802, US-1801 | 4 | F1-T92 | T1 | Sec |

Jumlah v0.3 `[USER] C21-P4`: **104 task, 361 jam-task** di Fase 1 = 80 task lama dikurangi 7 yang pindah ke Fase 2 (F1-T57, F1-T67, F1-T75–F1-T79; 25 jam-task), ditambah F1-T81–F1-T111 (31 task, 109 jam-task, 19 berkelas T1, semua jalur backend/web). F1-T71 bergantung juga pada F1-T110. Jumlah v0.2: **80 task, 277 jam-task**; 54 task berkelas T1 (jumlah baris T1 di tabel: 45 di v0.1 ditambah F1-T72–F1-T80). Task mobile developer: F1-T44–F1-T48, F1-T52–F1-T54, F1-T58, F1-T63, F1-T77, F1-T78 (44 jam-task). Reset MFA pasien oleh Layanan Pelanggan (US-207 AC5, `[KEPUTUSAN-204]` butir 2) ada di Fase 2 `[KEPUTUSAN-217]`.

### 7.3a Status dan urutan per 2026-11-20 (v0.4, CR-022)

Sumber status: `docs/fase-1-catatan.md` (posisi akhir pemanggilan `/runbook auto`) dan `docs/STATUS.md`; status task resmi tetap di `docs/backlog.md` dengan bukti commit.

| Kelompok | Task | Jam-task |
| --- | --- | --- |
| Selesai atau ter-merge ke `tahap-08-fase-1` | F1-T01–F1-T18 (termasuk spike F1-T02, F1-T03); F1-T81, F1-T82 (dokumen gate CR-021); F1-T83, F1-T84 (desain CR-021) | 73 dari 361 (20,2%); 22 dari 104 task |
| Selesai, tanpa ukuran | TD-01 | tidak dihitung |
| Berikutnya (baseline) | F1-T19 (setelah F1-T18), lalu F1-T20 dst. (audit trail, realm dan MFA, RBAC, enkripsi, berkas, BFF, kerangka web), reference implementation, F1-T70 | 193 sisa baseline (55 task) |
| Sesudah baseline (CR-021) | F1-T85–F1-T111 | 95 (27 task) |
| Paling akhir | F1-T71 (bergantung pada F1-T110) | termasuk di 193 |

Ukuran, dependensi, dan kelas task tidak berubah di v0.4 selain dependensi F1-T19. Total Fase 1 tetap 104 task, 361 jam-task. Hitungan lengkap di §6.8.3–§6.8.4.

**Penyesuaian definisi dari klarifikasi AC** (disetujui di Checkpoint 2 `[USER] C2-B4, C2-B7, C2-B14, C2-B15`). Task berikut menyesuaikan definisinya dengan klarifikasi yang disetujui, tanpa perubahan ukuran di versi ini: F1-T25, F1-T29–F1-T31, F1-T33, F1-T35, F1-T36, F1-T42, F1-T46, F1-T50, F1-T51, F1-T53–F1-T60, F1-T62, F1-T63, F1-T65, F1-T68, F1-T69 `[DOK: 07-klarifikasi-ac-fase-1.md]` §6. Rincian per task ada di baris "Task." setiap butir T-01–T-22 di dokumen itu. Penyesuaian yang menonjol:
- F1-T60: batas NIK berlapis (3/24 jam per akun, 3/24 jam per NIK lintas akun, 10/24 jam per IP, peringatan > 20/jam global) `[USER] C2-B4`.
- F1-T65: bootstrap Super Admin dengan persetujuan orang kedua (Control Group OpenBao), penolakan menonaktifkan akun sendiri, Auditor eksklusif, dan Auditor terakhir tidak dapat dicabut (UP-76) `[USER] C2-B7`. Siapa Super Admin pertama dan penyetujunya: PT-52.
- F1-T69: peringatan kegagalan audit lewat Alertmanager ditunda ke Fase 2 (KEPUTUSAN-197).
- Email pemberitahuan penguncian **tidak** dibangun di rilis 1 `[USER] C2-B4`.

Klarifikasi §6 menyatakan ukuran task dinilai ulang saat AC disetujui. Penilaian ulang itu belum dilakukan di versi ini, karena teks AC PRD yang final sedang diperbarui paralel. Penilaian ulang dilakukan (persona Tech Lead) setelah PRD v1.2 final dan sebelum task terdampak pertama dimulai. Task yang melebihi 4 jam-task dipecah dengan nomor baru, dan perubahannya dicatat di §14. Sampai itu, total 277 jam-task cenderung terlalu rendah. Kecepatan terukur di gerbang re-plan memakai ukuran yang tertulis di tabel saat task selesai `[KEPUTUSAN-223]`.

### 7.4 Jalur kritis Fase 1

Rantai terpanjang yang harus berurutan (tanpa memperhitungkan siapa yang mengerjakan), dihitung sebagai longest path dari kolom "Jam" dan "Bergantung pada" di §7.3:

**F1-T01 → T04 → T05 → T07 → T13 → T14 → T15 → T17 → T27 → T29 → T60 → T61 → T62 → T66 → T56 → T58 → T63 → T70 → T71**, total **65 jam-task berurutan** (2+3+3+3+3+4+4+4+4+4+4+3+4+2+4+4+4+3+3).

Arti rantai ini:
- Kriptografi dan berkas (T27 → T29) masuk jalur kritis sebelum reference implementation.
- Story profil dan layar mobile (T56 → T58 → T63) ada di jalur kritis karena sengaja menunggu dokumentasi pola reference implementation (T66). Jalur mobile (mobile developer) ikut menentukan selesainya Fase 1.
- Kalau ketergantungan T56 pada T66 dilepas (profil dibangun tanpa menunggu pola tertulis), rantai turun menjadi 62 jam-task (T62 → T64 → T70 → T71 jadi terpanjang), dengan risiko pola profil menyimpang dari reference implementation. Penyusun mempertahankan ketergantungan itu.

**v0.3: jalur kritis tetap 65 jam-task** setelah F1-T81–F1-T111 ditambahkan dan F1-T75–F1-T79 pindah ke Fase 2. Rantai CR-021 terpanjang selesai paling awal di jam-task ke-51 (slack ±11 terhadap F1-T70); F1-T80 kini ditentukan F1-T74 (ke-40) dan F1-T73 (§6.7) `[KEPUTUSAN-261]`.

**v0.2: jalur kritis tetap 65 jam-task** setelah F1-T72–F1-T80 ditambahkan dan F1-T70 bergantung pada F1-T80 (dihitung ulang dengan cara yang sama: earliest finish per task dari kolom "Jam" dan "Bergantung pada") `[KEPUTUSAN-216]`.

Slack rantai lain terhadap F1-T70 (mulai paling awal di jam-task ke-59 pada jalur kritis):
- Web: T39 → T41 → T42 → T43 → T64 selesai paling awal di jam-task ke-49 (slack ±10).
- Audit: T22 → T23 → T24 → T69 selesai paling awal di jam-task ke-48 (slack ±11).
- MFA dan ubah password (v0.2): T31 → T34 → T75 → T77 → T78 → T80 selesai paling awal di jam-task ke-48 (slack ±11). Cabang lain yang masuk ke F1-T80: T55 → T74 (ke-40), T76 → T79 (ke-42).
- Registrasi (v0.2): T36 → T50 → T51 → T53 selesai paling awal di jam-task ke-42 bila ADR jalur email disetujui tanpa jeda, lalu masuk ke F1-T80 (slack ±14 sampai F1-T80 menjadi penentu; v0.1: slack ±17 terhadap F1-T70).

**Keputusan user di jalur kritis**: persetujuan `scripts/verify.sh` (F1-T05) ada di jalur kritis. Persetujuan ADR jalur email (F1-T02 → F1-T36) **tidak** ada di jalur kritis yang dihitung, tetapi slack-nya turun menjadi ±14 jam-task karena uji end-to-end MFA (F1-T80) kini ada di antara rantai registrasi dan F1-T70. Bila persetujuannya tertunda lebih lama, rantai registrasi dan MFA (US-101, US-103, CR-018) menjadi jalur kritis dan story itu digeser ke akhir Fase 1.

**Pembatas sumber daya (tidak dihitung di longest path).** Mobile developer memegang 12 task (44 jam-task), dua di jalur kritis (F1-T58, F1-T63) dan dua di rantai MFA (F1-T77, F1-T78). Urutan kerja yang disarankan untuk jalur mobile: F1-T44 → T45 → T46 → T47 → T48 → T52 → T54 → T77 → T78 → T53 → T58 → T63. Bila dikerjakan berurutan oleh satu orang dengan ukuran tertulis, urutan ini menyelesaikan F1-T78 di jam-task ke-47, F1-T53 di ke-50, dan F1-T63 di ke-59, sama dengan jalur kritis. Layar MFA dan ubah password dikerjakan selagi F1-T58 menunggu F1-T56 (ke-51), sehingga tidak menahan layar di jalur kritis dan F1-T80 (ke-53) tidak menahan F1-T70 `[KEPUTUSAN-216]`.

### 7.5 Pemeriksaan Definition of Ready untuk story Fase 1

| Story | AC jelas dan dapat diuji (PRD) | Rujukan PRD/ADR | Dependensi | Ukuran |
| --- | --- | --- | --- | --- |
| US-101 | Ya, setelah teks AC diperbarui sesuai klarifikasi dan CR-018 (dikerjakan paralel di PRD) | PRD §5 EP-01; ADR-011 butir 6; ADR-011 revisi CR-018 | F1-T36 (ADR email) — jelas, belum selesai | 4 task + MFA (F1-T72, F1-T73, F1-T77) |
| US-103 | Ya, setelah teks AC diperbarui (AC1/AC1b TOTP, AC3 pesan seragam UP-73, AC4a–AC4e, AC6 CR-017, AC7–AC8 daftar ulang TOTP dan perangkat hilang); PT-44 terjawab lewat UP-73 `[USER] C2-B4`. Mekanisme Keycloak untuk AC6, AC7, dan kode pemulihan wajib `[TIDAK DIKETAHUI]`, diverifikasi di F1-T72, F1-T75, F1-T76 | PRD §5; ADR-011 butir 4, 5, 7, 8, 9; ADR-011 revisi CR-018 | — | 2 task + tema + F1-T74–F1-T76, F1-T78–F1-T80 |
| US-104 | Ya; teks AC1 menunggu L-08/PT-24, struktur dapat dibangun | PRD §5; ADR-011 butir 6; ADR-013 | AC4 ke Fase 2 (US-1001) | 2 task |
| US-105 | Ya | PRD §5; ADR-003 | Pola RI (F1-T66) | 3 task |
| US-106 | Ya; format NIK 16 digit ditetapkan data dictionary A2.4 (ADR-013 §4) | PRD §5; ADR-013, ADR-018 | Fondasi Fase 1 | 4 task |
| US-201 | Ya | PRD §5; ADR-008, ADR-009 | Notifikasi AC2 ke Fase 2 | 2 task |
| US-208 | Ya | PRD §5; ADR-011, ADR-012 | Fondasi Fase 1 | 2 task |
| US-1501 | Ya | PRD §5, §7; ADR-009 | Fondasi audit | 1 task + EN-05 |
| US-1504 | Ya | PRD §5; ADR-009 | Fondasi audit | 1 task |

Klarifikasi AC Fase 1 disetujui di Checkpoint 2 `[USER] C2-B4–C2-B15`. Story Fase 1 memenuhi Definition of Ready setelah teks AC PRD diperbarui sesuai klarifikasi dan CR-017/CR-018 (tindak lanjut wajib sebelum Tahap 8 `[DOK: checkpoint-2.md]`). Satu dependensi masih berupa keputusan user: ADR jalur email (G-03). PT-44 sudah terjawab.

### 7.6 Butir verifikasi Tahap 8 Fase 1 dari stack §13.2

| Butir stack §13.2 | Task |
| --- | --- |
| 8 (NEO Object Storage), 18 (lokasi data SES), 19 (lisensi Node.js dan PostgreSQL) | F1-T03, F1-T02, F1-T04 |
| 11 (opsi Keycloak), 13 (API jsonwebtoken, aws-lc-rs, jose, sqlx-cli), 14 (generator Dart, ESLint, lint clippy), 15 (mekanisme platform mobile), 21 (Valkey dengan `redis` dan ioredis) | F1-T31, F1-T32, F1-T26, F1-T41, F1-T14 (perintah sqlx-cli), F1-T39, F1-T46, F1-T12, F1-T48, F1-T33 |
| 1–7, 9, 9a, 10, 16, 17, 20 (WireGuard, NTP, replikasi, observability, OpenBao, isolasi staging, OIDC GitHub, GHCR, LB NEO, TLS antar-host, port LiveKit, batas laju Caddy, IaC, build iOS) | EN-15, EN-16, EN-18 (Fase 2), EN-26 (Februari). Butir yang terkait PT-39 baru dapat diverifikasi setelah PT-39 dijawab |

---

## 8. Fase 2–4: isi, ukuran, dan dependensi

Kelas ukuran per story (S/M/L/XL, §6.1). Task Fase 2–4 dipecah di awal fase masing-masing setelah gerbang re-plan, mengikuti pola Fase 1.

### 8.1 Fase 2: konsultasi online dokter dan psikolog + pembayaran

| Kelompok | Story / enabler (kelas) | Dependensi teknis | Gerbang |
| --- | --- | --- | --- |
| Lingkungan | EN-15 staging + CD (runner JIT, OIDC GitHub → OpenBao staging, cosign, deploy bergulir); EN-16 observability (Prometheus, Alertmanager, Loki tenant app/security, Grafana, GlitchTip, peringatan umur outbox RT-12) | EN-02 | PT-39 |
| Tenaga kesehatan dan rekanan | EN-17 realm `nakes`, MFA, kunci aplikasi 15 menit (CR-009), kerangka aplikasi nakes; EN-20 kerangka portal + `PartnerScope`; US-202 L, US-203 M (catatan QA N3: deskripsi layar), US-204 M, US-205 L, US-206 M, US-207 L (kaskade ke produk/home visit diverifikasi di Fase 4) | EN-06, EN-07 | PT-47 |
| Konsultasi | EN-18 LiveKit (token per peserta, test Egress tidak terpasang); EN-19 proses `realtime` WebSocket + Pub/Sub Valkey + AsyncAPI; US-107 S, US-301 M, US-302 L, US-303 M, US-304 L, US-305 XL, US-306 L, US-307 M, US-308 M (draf di server, UP-72), US-309 L (PDF, QR, halaman verifikasi publik), US-310 M, US-311 S | EN-17, EN-19 | PT-42 (UP-72) |
| Pembayaran (T1) | EN-24 pajak CR-013 (data dictionary A6/A8/A13 + migrasi, review DBA) **sebelum** US-801; US-801 L, US-802 XL (adaptor DOKU, verifikasi notifikasi, RF-12/RF-13/RF-14), US-803 L, US-804 M, US-805 M, US-806 L, US-808 L, US-809 M, US-812 M, US-1406 S (pembayaran, video, email) | EN-04, EN-05, EN-09 | PT-36 (target 2026-12-21), sisa PT-28 |
| Notifikasi | US-1001 L (SES + kotak notifikasi), US-1002 M, US-1003 M, UI-04 template email (D-F02); menutup US-104 AC4 dan pemberitahuan US-201 AC2; notifikasi pemberian sub-peran sensitif (Super Admin, Auditor, Verifikator, Keuangan, Petugas PDP) ke semua Super Admin dan Auditor, UP-76 d-2, S `[KEPUTUSAN-220]` | EN-13 | — |
| Sesi dan akun (T1, v0.2) | **US-110 "Perangkat dan sesi aktif" XL** (induk BR-001, BR-108 `[DOK: 03-prd.md]`) `[USER] C2-Q10` `[KEPUTUSAN-218]`: daftar sesi dan tombol akhiri satu atau semua sesi di aplikasi pasien, aplikasi tenaga kesehatan, backoffice, dan portal rekanan; rancangan mengikuti `[KEPUTUSAN-205]` (daftar dan pengakhiran sesi lewat Admin REST API Keycloak ditambah `session_revocation`, autentikasi ulang, pengakhiran sesi BFF, E-03); test negatif mengakhiri sesi milik akun lain (US-110 AC5). Sesi paralel tidak dibatasi (NFR-SEC-11) `[USER] C2-Q11`. **Reset MFA pasien oleh Layanan Pelanggan, US-207 AC5, M** (CR-018; rancangan `[KEPUTUSAN-204]` butir 2: tautan konfirmasi ke email akun, password tetap diminta, semua sesi diakhiri, E-04); bukti identitas untuk pasien yang kehilangan perangkat, kode pemulihan, dan akses email sekaligus `[TIDAK DIKETAHUI]` (KEPUTUSAN-204 butir 3) `[KEPUTUSAN-217]`. **CR-017 dan daftar ulang TOTP (US-103 AC6, AC7) di realm `nakes`, M**: layar di aplikasi tenaga kesehatan, bersama EN-17, memakai mekanisme Keycloak dari F1-T75 dan F1-T76 `[KEPUTUSAN-219]` | EN-06, EN-07, EN-17, EN-20, F1-T75, F1-T76 | — |

**Tambahan v0.3 (CR-021, S3 `[USER] C21-P4`):**

| Kelompok | Isi (kelas atau jam) | Dependensi teknis | Gerbang |
| --- | --- | --- | --- |
| Pindahan dari Fase 1 (25 jam-task) | F1-T57 ubah email (3), F1-T67 layar kelola admin (3), F1-T75 daftar ulang TOTP `pasien`/`admin` (4), F1-T76 ubah password `pasien`/`admin` (4), F1-T77 dan F1-T78 layar mobile (4 + 3), F1-T79 web backoffice (4); dikerjakan bersama CR-017/daftar ulang TOTP realm `nakes` (KEPUTUSAN-219) | F1-T33, F1-T34, F1-T72 | — |
| Reset MFA HR, K1-F2 (M 12) | US-1904 AC7–AC9: daftar ulang TOTP HR, reset oleh Verifikator dengan konfirmasi kanal kedua, pola KEPUTUSAN-204, pemberitahuan semua HR | US-207 AC5, F1-T75, F1-T85 | — |

Bila PT-36 belum terjawab 2026-12-21, adaptor DOKU tetap dapat dirancang dari dokumentasi publik `[SRC-203]`–`[SRC-207]`, tetapi tidak difinalkan; risiko penggantian penyedia mengikuti RT-27 `[DOK: 04-risk-register.md]`.

### 8.2 Fase 3: integrasi rekanan, RME minimalis, klinik/lab, kepatuhan

| Kelompok | Story / enabler (kelas) | Dependensi teknis | Gerbang |
| --- | --- | --- | --- |
| Integrasi (T1) | EN-21 gateway Partner Integration API (realm `integrasi` client credentials, HMAC webhook, idempotensi, outbox integrasi, pengaman SSRF, sandbox staging); US-1401 L, US-1402 L, US-1403 L, US-1404 L, US-1405 M | EN-15 | PT-05 |
| RME minimalis (T1) | EN-22 fondasi RME (`rme-backend`, realm `rme`, RLS + konteks tenant, migrasi C1–C6 termasuk CR-014 bagian DB RME dan hak pembersihan `rme_worker`, audit RME, web RME BFF); US-1301 L, US-1302 L, US-1303 L, US-1304 M, US-1306 L, US-1307 M, US-1309 XL, US-1310 M | EN-21 (kontrak sama), EN-04–EN-07 | PT-32 (uji US-1309), PT-20/L-05 |
| Klinik, lab, info biaya | US-401 M, US-402 L (catatan QA N3: layar penerima layanan), US-403 M, US-404 M, US-406 S, US-601 M, US-602 S, US-901 M, US-902 M | EN-21, US-801–US-806 | PT-05 |
| Uang lanjutan (T1) | US-807 L, US-810 M, US-811 L (catatan QA N3: isi tab Rekonsiliasi) | Fase 2 pembayaran | PT-36 |
| Backoffice, portal, konten | US-1101 M (**v0.3: pindah ke Fase 1**, F1-T91–F1-T95), US-1102 S, US-1103 M, US-1104 M, US-1201 L, US-1202 S, US-1203 S, US-1502 M, US-1503 M, US-1701 M | EN-20 | — |
| Privasi (T1) | EN-23 pekerja privasi (kebijakan retensi kosong tidak memproses apa pun); US-1601 M, US-1602 M, US-1603 L, US-1604 L; US-1605 L dan US-1606 L terblokir | EN-09 | PT-24, L-06 |
| Kualitas | EN-25 uji beban NFR-CAP-04 (Januari); UI-05 uji manual TalkBack/VoiceOver alur A01, A05, A07 | EN-15 | — |
| Anggota keluarga (Opsi B, v0.2) | US-108 M, US-109 M (dipindah dari Fase 4) `[USER] C2-Q1` | EN-04–EN-07 | — |
| Akun dan sesi web RME (T1, v0.2) | CR-017, daftar ulang TOTP (US-103 AC6, AC7), dan US-110 untuk realm `rme` di web RME, M, memakai komponen dan pola Fase 1–2 `[KEPUTUSAN-219]` | EN-22, US-110 | — |
| Layanan korporat K2 (T1, v0.3, CR-021) | US-1905 M, US-1906 L, US-1907 L, US-1908 L (porsi mobile ±6), US-1909 L, US-1910 L, US-1911 M (porsi mobile ±6), US-1912 M, US-1913 M (porsi mobile ±6); jurnal korporat M; email aplikasi korporat (outbox, templat undangan dan invoice) M; mutasi bank dan alokasi S; pengerasan undangan (App Links, batas kirim ulang, konfirmasi tanggal lahir bila PT-78 disetujui) S; test kanal turunan S = 195 `[USER] C21-P4` | EN-21 (katalog lewat integrasi, K-04 opsi a), jurnal Fase 2 (US-801–US-806), US-1001 (email), F1-T98–F1-T100 | G-25 (PT-59 dan L-21 sebelum penagihan; PT-62–PT-66), G-26 (L-17–L-19, DPIA sebelum data peserta nyata) |
| Layanan korporat K3 (T1, v0.3, CR-021) | US-1914 M, US-1915 M (porsi mobile ±6), US-1917 M = 36 | EN-21, US-901, K2 | G-26; PT-64, PT-65, PT-74 |

### 8.3 Fase 4: layanan yang dilepas pertama (tidak dikerjakan di baseline Opsi B)

Fase 4 **tidak dikerjakan** di baseline Opsi B `[USER] C2-Q1`. Tabel ini berlaku hanya bila gerbang re-plan 2026-12-11 memilih Opsi A.

| Kelompok | Story (kelas) | Gerbang |
| --- | --- | --- |
| Anggota keluarga (Should) | US-108 M, US-109 M | — (di Opsi B sudah dipindah ke Fase 3, §8.2) |
| Antrean (Should, nomor 1) | US-405 M, US-1305 M | — |
| Produk (nomor 2) | US-701 M, US-702 M, US-703 L, US-704 L, US-705 M, US-706 S, US-1406 (kurir) | PT-29, PT-30, PT-33, PT-37; L-03/PT-21 untuk obat bebas |
| Home visit (nomor 3) | US-501 M, US-502 M, US-503 M, US-504 M, US-1308 M | — |

### 8.4 Production dan uji pemulihan (Februari 2027, sebelum go-live)

EN-26 (host production Biznet Gio dan Hostinger, Docker Compose, Caddy, OpenBao Raft 3 node dengan unseal Shamir, pgBackRest dan standby lewat WireGuard, bastion dengan SSH CA, skrip deploy dengan Control Group; T1, review keamanan dan DBA) dan EN-27 (uji pemulihan NFR-DR-03). Kode infrastrukturnya ditulis di `infra/` sejak Fase 2 bersama staging; penyewaan host mulai 1 Februari 2027 `[DOK: 05-stack.md]` §2.9. Jendela 2027-02-01 – 02-12 berisi 9 hari kerja tim (5 Februari cuti bersama, §6.2a). Gerbang: PT-38 dan PT-52 (bootstrap Super Admin pertama di production).

---

## 9. Gerbang dan dependensi eksternal

| # | Gerbang | Dibutuhkan sebelum | Bila belum terbuka | Pemilik |
| --- | --- | --- | --- | --- |
| G-01 | Checkpoint 2: phase plan, design guideline, opsi | Fase 1 mulai | Tahap 8 belum dimulai | User. **Dijawab 2026-11-18** `[DOK: checkpoint-2.md]`; tindak lanjut wajib sebelum Tahap 8 tercatat di sana |
| G-02 | Persetujuan `scripts/verify.sh` | F1-T05 | Tidak ada commit kode | User |
| G-03 | ADR jalur email (dari F1-T02) | F1-T36, US-101, US-103, US-105 AC4, F1-T74, F1-T80 | Story itu digeser ke akhir Fase 1 | User |
| G-04 | PT-44 pesan akun terkunci | F1-T35, F1-T55 | — | **Selesai**: pesan seragam UP-73 opsi B `[USER] C2-B4` |
| G-05 | Gerbang re-plan dengan kecepatan terukur (v0.4, CR-022: menetapkan tanggal go-live baru dan tambahan tenaga, bukan memilih opsi pemotongan scope; §6.8.5) | Fase 2 | Fase 2 dimulai dengan baseline Opsi B + CR-021 pada tanggal yang disetujui; tanggal go-live mengikuti kecepatan terukur `[USER]` CR-022 | User |
| G-06 | PT-39 konfirmasi Biznet Gio | EN-15 (awal Fase 2), rancangan ingress production | Staging tertunda; deploy hanya lokal | User |
| G-07 | PT-47 status integrasi klinik untuk konsultasi | US-205, US-303 (Fase 2) | Syarat tampil klinik mengikuti US-205 AC2 apa adanya | User/CEO |
| G-08 | PT-42 draf catatan (UP-72) | US-308 (Fase 2) | — | **Selesai**: UP-72 disetujui `[USER] C2-B15` |
| G-09 | PT-36 konfirmasi tertulis DOKU | Task pembayaran Fase 2 (target 2026-12-21) | Adaptor dirancang tetapi tidak difinalkan (RT-27) | User |
| G-10 | Sisa PT-28 (jenis, tarif, pembulatan, perlakuan atas ongkir dan biaya transaksi, penerima dana pajak, isi bukti bayar, perilaku saat tarif belum diatur) | Perhitungan pajak US-801 (Fase 2); go-live | Komponen pajak dibangun sebagai parameter tanpa nilai | CEO |
| G-11 | PT-05 rekanan pertama dan kemampuan sistemnya | Fase 3 (EP-14), SIT | Integrasi hanya diuji dengan sandbox dan RME | User, CEO |
| G-12 | PT-32 registrasi SATUSEHAT | Uji US-1309 (Fase 3) | Status "Menunggu kredensial" (KEPUTUSAN-16) | User, CEO |
| G-13 | PT-24 retensi (setelah L-05, L-06) | US-1605, US-1606 (Fase 3) | Story tetap terblokir; risiko kepatuhan go-live | CEO + legal |
| G-14 | PT-20 legal (L-01–L-15) | Teks persetujuan (US-104 AC1), resep (L-05), refund (L-14), data minimal (L-08) sebelum go-live | AC terkait tidak dapat difinalkan | User, CEO |
| G-15 | PT-49 auditor keamanan eksternal | Pemesanan jadwal sebelum Fase 3 | Jendela audit Februari tidak terjamin | User, CEO |
| G-16 | PT-29, PT-30, PT-33, PT-37 | Fase 4 (EP-07), hanya bila gerbang re-plan memilih Opsi A | Produk tidak dibangun (sudah demikian di baseline Opsi B) | CEO, User |
| G-17 | PT-17 nilai markup (PB-11) | UAT dan go-live | Rekanan tidak tampil (US-809 AC3) | CEO |
| G-18 | PT-38 pemegang share unseal dan orang kedua | Production (2027-02-01) | Production tidak dapat diinisialisasi | User + CTO |
| G-19 | PT-50 peserta UAT; PT-51 akun developer toko aplikasi | Februari 2027; pengajuan toko 2027-02-12 | Syarat go-live 4 dan 7 terancam | User, CEO |
| G-20 | PT-35 pemelihara setelah go-live | Sebelum go-live (Tahap 12) | RT-29 | User, CEO |
| G-21 | CR-011, CR-015 diputuskan | Checkpoint 2 | — | **Selesai**: keduanya rilis berikutnya `[USER] C2-B5, C2-B6` |
| G-22 | PT-52 Super Admin pertama dan orang kedua penyetuju bootstrap (UP-76) | EN-26 production (2027-02-01); bootstrap di F1-T65 memakai data uji di lokal dan staging | Bootstrap production tidak dapat dijalankan; syarat minimal dua Super Admin sebelum go-live tidak terpenuhi | User/CEO |
| G-24 (v0.3) | Persetujuan CR-021 dan ADR-027/ADR-028 | F1-T81–F1-T111 | **Selesai**: `[USER] C21-P4`; ADR-027 dan ADR-028 Diterima | User |
| G-25 (v0.3) | PT-59 dan L-21 (arah dana invoice B2B), PT-62–PT-66 (markup korporat, aturan pesanan, kehadiran, ringkasan, NIK) | K2 penagihan dan pesanan (Fase 3); ADR-029 Diterima | Struktur dibangun tanpa arah dana final; penagihan tidak diaktifkan | User, CEO, legal |
| G-26 (v0.3) | L-17, L-18, L-19 dan DPIA bagian CR-021; PT-76, PT-77 | Data peserta nyata pertama diproses; katalog berisi layanan nyata | Pengembangan memakai data uji | CEO + legal |
| G-27 (v0.3) | Sisa PT-67 (sub-peran baru? persetujuan orang kedua untuk perusahaan buatan Verifikator, M-49); bentuk bukti kewenangan pendaftar (PT-79) | F1-T101, F1-T105 | F1-T105 dibangun tanpa persetujuan orang kedua, ditambah bila user memutuskan | User |
| G-23 | PT-53 developer backend/web tambahan: jumlah orang, biaya terhadap toleransi +20%, tanggal mulai pasti (target awal Januari 2027) | Gerbang re-plan 2026-12-11 (perencanaan Fase 3) | Fase 3 direncanakan tanpa kontribusi Opsi E; opsi scope mengikuti `v` tim lama (§6.5) | User/CEO |

---

## 10. Jalur kritis proyek

1. Fase 1 fondasi dan reference implementation (jalur kritis 65 jam-task, §7.4).
2. Gerbang re-plan 2026-12-11 (v0.4: penetapan tanggal go-live baru; rincian Opsi E, PT-53; §6.8.5).
3. Fase 2 pembayaran dalam 26 hari kalender (v0.2: 16 hari kerja), bergantung pada PT-36 (target 2026-12-21) dan EN-24 pajak. Libur 24–25 Desember 2026 dan 1, 5 Januari 2027 bukan hari libur tim sejak v0.4 (§6.2a); apakah penyedia luar (DOKU) bekerja di hari itu `[TIDAK DIKETAHUI]`.
4. Fase 3 integrasi, bergantung pada PT-05 (rekanan nyata untuk SIT), untuk RME pada PT-32, dan pada onboarding developer tambahan yang mulai awal Januari 2027 (lama onboarding `[TIDAK DIKETAHUI]`).
5. Pembekuan fitur 2027-01-29 (v0.4: dapat bergeser lewat gerbang 2026-12-11) → SIT → audit keamanan eksternal (PT-49) → UAT (PT-50) → pengajuan toko aplikasi paling lambat 2027-02-12 (PT-51, R-08) → deploy → go-live (target semula 2027-03-01; boleh mundur, CR-022).

Butir eksternal di jalur kritis yang lamanya `[TIDAK DIKETAHUI]`: konfirmasi DOKU, kesiapan rekanan, registrasi SATUSEHAT, audit eksternal, review toko aplikasi, dan perizinan (PT-20, PT-25).

---

## 11. Butir yang jatuh tempo di Tahap 7

Status setelah Checkpoint 2 (2026-11-18):

| Butir | Keputusan | Sumber |
| --- | --- | --- |
| CR-011 (§11.1) | Rilis berikutnya (UP-68 opsi c) | `[USER] C2-B5` |
| CR-015 (§11.2) | Rilis berikutnya (UP-69 opsi c); ubin vaksinasi/infus tidak tampil (UP-63) | `[USER] C2-B6` |
| CR-013 (§11.3) | Teks PRD v1.2 disetujui; EN-24 tetap di awal Fase 2 | `[USER] C2-B15` |
| CR-014 (§11.4) | Tetap terjadwal (F1-T17, EN-22); KEPUTUSAN-185 disetujui | `[USER] C2-B15` |
| PT-42 (§11.6) | **Selesai**: UP-72 disetujui (draf di server) | `[USER] C2-B15` |
| PT-43 (§11.6) | **Selesai**: UP-71 disetujui (PB-07 dan sesi 12 jam sebagai pengecualian essential) | `[USER] C2-B15` |
| PT-44 (§11.6) | **Selesai**: digabung ke UP-73 opsi B (pesan seragam); UP-70 tercakup; email pemberitahuan penguncian tidak dibangun di rilis 1 | `[USER] C2-B4` |
| CR-017 (baru) | Ubah password saat login masuk rilis 1 (§7.3 F1-T76, F1-T78, F1-T79; Fase 2 dan 3 untuk realm `nakes`, `rme`) | `[USER] C2-B14` |
| CR-018 (baru) | MFA wajib pasien masuk rilis 1, revisi ADR-011 (§7.3 F1-T72–F1-T75, F1-T77, F1-T79, F1-T80; reset MFA pasien oleh Layanan Pelanggan, US-207 AC5, di Fase 2) | `[USER] C2-Q9` |

Isi §11.1–§11.6 di bawah dipertahankan sebagai dasar keputusan.

### 11.1 CR-011: onboarding mandiri dan langganan berbayar RME (PT-34) — diputuskan rilis berikutnya

| Bagian | Isi | Ukuran (estimasi) |
| --- | --- | --- |
| Pendaftaran mandiri calon rekanan | Formulir, unggah dokumen izin, persetujuan perjanjian RME dan pemrosesan data; verifikasi tetap oleh Verifikator (BR-006, US-205, UP-49) | M + L (±33 jam-task) |
| Provisioning tenant RME otomatis | Menggantikan langkah Operator RME di US-1301 | M (±12) |
| Penagihan langganan | Paket, tagihan, pembayaran ke PT. Arunika lewat adaptor terpisah dari pembagian dana pasien (arsitektur §13), penangguhan saat telat bayar | XL + L (±57) |
| Penyesuaian backoffice | Antrean pendaftaran mandiri | S (±6) |
| **Total** | Belum termasuk revisi BRD/PRD dan review kepatuhan | **±108 (rentang ±90–150)** |

Fakta bisnis yang belum ada dan memblokir bagian penagihan: apakah RME berbayar, model/harga/siklus, cara bayar, perlakuan telat bayar `[TIDAK DIKETAHUI]` (PT-34) `[DOK: backlog.md]` CR-011.

Opsi `[UP-68]`: (a) rilis 1 penuh, menambah ±108 jam-task ke Fase 3 yang sudah membutuhkan kecepatan tertinggi; (b) rilis 1 sebagian, hanya pendaftaran mandiri tanpa penagihan (±45 jam-task), dengan verifikasi tetap manual; (c) rilis berikutnya; titik perluasan sudah disiapkan arsitektur §13. **Usulan penyusun: (c)**, karena hasil §6 dan karena PT-34 belum terjawab.

### 11.2 CR-015: vaksinasi dan suntik/infus vitamin sebagai layanan lab — diputuskan rilis berikutnya

| Varian | Perubahan | Ukuran (estimasi) |
| --- | --- | --- |
| Di lokasi lab saja | Revisi BRD/PRD (jenis layanan baru tanpa alur hasil lab, karena alur hasil lab terhubung ke LOINC, ADR-020 butir 5); enum jenis layanan di data dictionary A7/C2 lewat migrasi; kontrak Partner Integration API; layanan RME (US-1303); katalog dan pemesanan pasien (A08); test | ±50–60 jam-task |
| Ditambah home visit oleh lab | Memperluas BRD P-05 (home visit kini hanya klinik) dan AB-50 (tenaga kesehatan dinaungi satu klinik): afiliasi tenaga kesehatan ke lab, penugasan dari sistem lab/RME, aturan refund home visit; keputusan eksplisit atas butir di luar scope BRD §4.2 "pengambilan sampel di rumah" | +±60–100 jam-task |
| Pencatatan tindakan dan vaksin di RME serta pengiriman ke SATUSEHAT | `[TIDAK DIKETAHUI]` (perlu cek sumber resmi bila CR diterima) | `[TIDAK DIKETAHUI]` |

Syarat regulasi (lab boleh melakukan vaksinasi/infus, dan syarat home visit-nya) `[TIDAK DIKETAHUI]`, menunggu legal (PT-20) `[DOK: backlog.md]` CR-015. Varian home visit juga bergantung pada home visit, yang berada di nomor 3 urutan pengurangan scope.

Opsi `[UP-69]`: (a) rilis 1, di lokasi lab saja; (b) rilis 1, termasuk home visit; (c) rilis berikutnya. **Usulan penyusun: (c)**; bila user memilih rilis 1, varian (a) dengan syarat jawaban legal tersedia sebelum Fase 3.

### 11.3 CR-013: pajak di atas harga rekanan + markup

- Diterapkan ke PRD sebagai **v1.2 (draf revisi CR-013)**: AB-10a baru, AB-11, AB-13, AB-17, AB-21, AB-27, definisi §3.5, PB-21, US-801 AC1, US-804 AC2, US-811 AC1, US-812 AC4 `[DOK: 03-prd.md]`. Arah rumus disetujui `[USER] T5-Q11a`; jenis, tarif, dan rincian lain tetap `[TIDAK DIKETAHUI]` (sisa PT-28).
- Data dictionary dan skema **tidak diubah sekarang**. Perubahannya dijadwalkan sebagai **EN-24** di awal Fase 2, sebelum task US-801, dengan review DBA (pola CR-014) `[KEPUTUSAN-185]`.

### 11.4 CR-014: perbaikan teknis data dictionary

Dijadwalkan sebagai task migrasi: bagian DB aplikasi di **F1-T17** (`data_key_retention_anchor`, data dictionary A19 diperbarui di task yang sama), bagian DB RME di **EN-22** Fase 3 (`rme_data_key_retention_anchor` dan hak pembersihan `rme_worker`), keduanya dengan review DBA dan keamanan `[KEPUTUSAN-185]`.

### 11.5 Pertanyaan terbuka yang ditempatkan sebagai gerbang

PT-29, PT-30, PT-33 → G-16 (Fase 4); PT-35 → G-20; PT-36 → G-09; PT-37 → G-16; PT-39 → G-06 (§9).

### 11.6 PT-42, PT-43, PT-44 (usulan untuk Checkpoint 2) — selesai, lihat tabel status di awal §11

- **PT-44 (pesan akun terkunci, modul T1)** `[UP-70]`: tampilkan pesan yang sama dengan US-103 AC2 untuk akun terkunci. OWASP: aplikasi "must respond with a generic error message regardless of whether ... The account is locked or disabled" `[SRC-261]`. Perilaku bawaan Keycloak untuk akun yang dikunci deteksi brute force sejalan: menampilkan "Invalid username or password" agar penyerang tidak tahu akun dinonaktifkan `[SRC-262]`. Usulan revisi teks US-103 AC3: "... akun terkunci sementara sampai 15 menit berlalu, walaupun password benar, dan pesan yang tampil sama dengan AC2". OWASP juga menyarankan fitur lupa password tetap dapat dipakai saat akun terkunci, untuk mencegah penguncian dipakai sebagai serangan penolakan layanan `[SRC-261]`; apakah atur ulang password di Keycloak membuka kunci `[TIDAK DIKETAHUI]`, diverifikasi di F1-T55. Pemberitahuan email ke pemilik akun saat terkunci tidak disebut di kedua sumber; ditawarkan sebagai opsi produk terpisah. Terkait, untuk dicermati reviewer keamanan: KU-10 (pesan "akun nonaktif" saat masuk) dan US-101 AC3 (email terdaftar diarahkan masuk) adalah dua titik lain yang menurut OWASP sebaiknya memakai pesan umum `[SRC-261]`; keduanya AC yang sudah disetujui, jadi perubahan hanya lewat CR bila user menghendaki.
- **PT-43 (batas waktu dan WCAG 2.2.1)** `[UP-71]`: SC 2.2.1 mengizinkan pengecualian "The time limit is essential and extending it would invalidate the activity" `[SRC-263]`. Dokumen Understanding menyebut batas waktu tertentu "implemented for security reasons" dapat dianggap essential, dan memperlakukan penahanan tiket sebagai kasus essential sambil menganjurkan memindahkan sebanyak mungkin proses ke luar periode yang dibatasi waktu `[SRC-263]`. Usulan: (a) PB-07 diperlakukan sebagai pengecualian essential, dengan mitigasi yang sudah ada di guideline §5.5 (peringatan sisa ≤ 5 menit, isian penerima dan keluhan tetap terisi saat memilih slot lagi) ditambah semua isian diminta sebelum slot ditahan; (b) sesi web mutlak 12 jam diperlakukan sebagai batas keamanan essential, dengan peringatan sebelum berakhir (guideline §5.5) dan isian panjang yang disimpan di server (draf, UP-72). Diputuskan user bersama reviewer keamanan.
- **PT-42 (draf catatan dan rekam medis)** `[UP-72]`: data dictionary yang lulus gate **sudah memuat** draf di server: `consultation_note_draft` (A9.6) dan `rme_medical_record_draft` (C4.1), kelas Operasional, terenkripsi, dihapus setelah difinalkan `[DOK: 04-data-dictionary.md]`. Kalimat di design guideline §12 PT-42 ("Data dictionary belum memuat status draf") tidak sesuai dan diusulkan dikoreksi di Checkpoint 2. Yang tersisa adalah perilaku: draf disimpan ke server saat pengguna berhenti mengetik dan saat aplikasi ke latar belakang atau terkunci, tanpa salinan persisten di perangkat (coding standards §1.6) `[KEPUTUSAN-186]`. Masa simpan draf yang tidak pernah difinalkan `[TIDAK DIKETAHUI]` (PT-24, L-05).

### 11.7 Catatan non-blokir QA Tahap 6 (N3–N5)

| Catatan | Ditangani di |
| --- | --- |
| N3 formulir alamat dan ubah email (US-105) | F1-T58 |
| N3 US-203 | Fase 2, task US-203 |
| N3 US-811 AC2 (tab Rekonsiliasi), US-402 (layar penerima layanan) | Fase 3, task story terkait |
| N4 state memuat/hover belum diperagakan | F1-T40 (galeri state web), F1-T45 (komponen Flutter dengan test) |
| N5a baris pajak | Revisi PRD v1.2 (CR-013) dan EN-24 |
| N5b label WIB di C02 | F1-T64 |

---

## 12. Register baru dari tahap ini

| Jenis | Nomor | Isi singkat | Status |
| --- | --- | --- | --- |
Status per v0.2. Status di `docs/decision-log.md` diperbarui pemegang berkas itu; kolom di sini mencatat keputusan Checkpoint 2.

| Jenis | Nomor | Isi singkat | Status |
| --- | --- | --- | --- |
| Keputusan | KEPUTUSAN-178 | Metode estimasi effort, hari kerja, porsi mobile | disetujui `[USER] C2-B15`; bagian "libur tidak dikurangi" diganti KEPUTUSAN-215 |
| Keputusan | KEPUTUSAN-179 | Penomoran EN-/UI-/F1-T; story boleh selesai bertahap per AC | disetujui (C2-B15) |
| Keputusan | KEPUTUSAN-180 | MVP Fase 1 = slice "pasien terverifikasi" | disetujui (C2-B15) |
| Keputusan | KEPUTUSAN-181 | Reference implementation = US-106 → US-201 | disetujui (C2-B15) |
| Keputusan | KEPUTUSAN-182 | Urutan teknis Fase 1 dan jalur kritis | disetujui (C2-B15); dilengkapi KEPUTUSAN-216 |
| Keputusan | KEPUTUSAN-183 | Spike jalur email Keycloak–SES di awal Fase 1, hasilnya usulan ADR ke user | disetujui (C2-B15) |
| Keputusan | KEPUTUSAN-184 | Spike penyimpanan berkas lokal/uji sebelum EN-10 | disetujui (C2-B15) |
| Keputusan | KEPUTUSAN-185 | Penjadwalan CR-013 (EN-24) dan CR-014 (F1-T17, EN-22) dengan review DBA | disetujui (C2-B15) |
| Keputusan | KEPUTUSAN-186 | Perilaku simpan draf ke server (PT-42) | disetujui (C2-B15) |
| Asumsi | ASUMSI-01 | Satu task 2–4 jam mencakup `verify.sh` dan satu putaran perbaikan review | disetujui (C2-B15) |
| Asumsi | ASUMSI-02 | Test penguncian Keycloak dapat diotomasi dengan testcontainers di `verify.sh` | disetujui (C2-B15) |
| Sumber | SRC-261–SRC-264 | OWASP Authentication Cheat Sheet; Keycloak brute force (pesan galat); W3C Understanding SC 2.2.1; Keycloak email lewat SMTP | tercatat |
| Usulan | UP-65–UP-72 | Kalender fase; prinsip urutan dan gerbang re-plan; opsi scope; CR-011; CR-015; PT-44; PT-43; PT-42 | diputuskan di Checkpoint 2 (rincian per UP di `docs/07-catatan-phase-plan.md`) |
| Pertanyaan | PT-47–PT-51 | Status integrasi klinik untuk konsultasi; kapasitas tim; auditor eksternal; peserta UAT; akun developer toko | PT-48 sebagian terjawab (C2-Q3, C2-Q4, C2-Q2, C2-Q8; throughput diukur di Fase 1); PT-47, PT-49, PT-50, PT-51 tetap terbuka |
| Keputusan (v0.2) | KEPUTUSAN-215 | Hari kerja dikurangi libur nasional dan cuti bersama SKB 2026/2027; jendela Februari 2027-02-01 – 02-12 | menunggu (checkpoint berikutnya) |
| Keputusan (v0.2) | KEPUTUSAN-216 | Task F1-T72–F1-T80 (CR-018, CR-017), dependensi, F1-T70 bergantung pada F1-T80, urutan jalur mobile | menunggu |
| Keputusan (v0.2) | KEPUTUSAN-217 | Reset MFA pasien oleh Layanan Pelanggan (US-207 AC5) di Fase 2, kelas M | menunggu |
| Keputusan (v0.2) | KEPUTUSAN-218 | US-110 kelas XL di Fase 2 (pasien, nakes, backoffice, portal) | menunggu |
| Keputusan (v0.2) | KEPUTUSAN-219 | CR-017 + daftar ulang TOTP realm `nakes` (Fase 2, M); CR-017 + daftar ulang TOTP + US-110 realm `rme` (Fase 3, M) | menunggu |
| Keputusan (v0.2) | KEPUTUSAN-220 | Notifikasi pemberian sub-peran sensitif (UP-76 d-2) di Fase 2, kelas S | menunggu |
| Keputusan (v0.2) | KEPUTUSAN-221 | Porsi mobile per opsi; kecepatan diukur per jalur; jam orang/sesi tidak dikonversi ke jam-task | menunggu |
| Keputusan (v0.2) | KEPUTUSAN-222 | Opsi E di gerbang re-plan dinyatakan sebagai kekurangan kecepatan backend/web Fase 3 | menunggu |
| Keputusan (v0.2) | KEPUTUSAN-223 | Penilaian ulang ukuran task yang disesuaikan klarifikasi setelah PRD v1.2 final | menunggu |
| Sumber (v0.2) | SRC-280–SRC-283 | SKB libur nasional dan cuti bersama 2026 (Setneg, Kemenko PMK) dan 2027 (Kemenko PMK, Setneg) | tercatat |
| Keputusan (v0.4) | KEPUTUSAN-281 | Pembagi kebutuhan = hari kalender (7 hari per minggu); akhir pekan antar-fase tidak dialokasikan; sisa Fase 1 dihitung dari 2026-11-20 inklusif | usulan, menunggu pencatatan di `docs/decision-log.md` |
| Keputusan (v0.4) | KEPUTUSAN-282 | Urutan Fase 1 CR-022: baseline (kecuali F1-T71), lalu F1-T85–F1-T111, lalu F1-T71; F1-T19 bergantung pula pada F1-T18 | usulan, menunggu pencatatan |
| Keputusan (v0.4) | KEPUTUSAN-283 | Rumus tanggal go-live baru di gerbang (pembekuan = 2026-12-11 + ⌈S ÷ v⌉; ekor 31 hari) dan kriteria `v` per 10 hari kalender | usulan, menunggu pencatatan |
| Keputusan (v0.4) | KEPUTUSAN-284 | Jam-task selesai dari kolom "Jam" §7.3; F1-T81/F1-T82 dihitung selesai; TD-01 tanpa ukuran dan tidak dihitung | usulan, menunggu pencatatan |
| Pertanyaan (Checkpoint 2) | PT-52, PT-53 | Super Admin pertama dan penyetujunya; developer backend/web tambahan | terbuka (§9 G-22, G-23) |

Rincian UP dan PT ada di `docs/07-catatan-phase-plan.md`.

---

## 13. Gate Tahap 7

| Syarat | Status |
| --- | --- |
| Semua story Fase 1 memenuhi Definition of Ready | Klarifikasi AC disetujui di Checkpoint 2; terpenuhi setelah teks AC PRD diperbarui (klarifikasi, CR-017, CR-018), dengan satu dependensi keputusan user (ADR jalur email) (§7.5) |
| Enabler fondasi ada di Fase 1 | Terpenuhi: skema inti (EN-04), autentikasi + RBAC (EN-06, EN-07), Audit Trail Foundation (EN-05), CI pipeline (EN-02), reference implementation (EN-14) |
| Test plan tersusun dari PRD | Terpenuhi (`docs/test-plan.md`, 674 test case); test case CR-017, CR-018, dan US-110 diperbarui QA (tindak lanjut Checkpoint 2) |
| User menyetujui pembagian fase | **Disetujui di Checkpoint 2**: UP-65 `[USER] C2-Q5`, UP-66 `[USER] C2-Q6`, UP-67 `[USER] C2-Q1` (Opsi B baseline, Opsi E `[USER] C2-Q2`) |
| Gate anti-halu | v0.1: fact-check LULUS (`docs/fact-check/tahap-07.md`). v0.2: menunggu fact-check atas baris yang berubah. v0.3: analisis CR-021 lulus fact-check (`docs/fact-check/CR-021.md`); baris v0.3 menunggu fact-check. v0.4: baris yang berubah (§6.8) menunggu fact-check (T2) |

---

## 14. Riwayat revisi

| Versi | Tanggal | Perubahan |
| --- | --- | --- |
| 0.1 | 2026-11-18 | Draf Tahap 7: empat fase, kalender usulan, estimasi effort, opsi scope, 71 task Fase 1, jalur kritis 65 jam-task, gerbang eksternal |
| 0.2 | 2026-11-18 | Tindak lanjut Checkpoint 2. (1) Hari kerja dikurangi libur nasional dan cuti bersama SKB 2026/2027 (SRC-280–SRC-283): Fase 2 20 → 16, Februari 10 → 9; §6.2a baru. (2) Kapasitas PT-48 dan Opsi E diisi dari jawaban user; kecepatan diukur per jalur. (3) Fase 1: F1-T72–F1-T80 (CR-018 MFA wajib pasien termasuk daftar ulang TOTP, CR-017 ubah password saat login; diselaraskan dengan revisi PRD US-101 AC5 dan US-103 AC1b, AC6–AC8), F1-T70 bergantung pada F1-T80; 80 task, 277 jam-task; jalur kritis tetap 65. (4) Fase 2: US-110 (XL), reset MFA pasien US-207 AC5 (M), CR-017 + daftar ulang TOTP `nakes` (M), notifikasi sub-peran (S); Fase 3: US-108/US-109 (Opsi B), CR-017 + daftar ulang TOTP + US-110 `rme` (M); Fase 4 tidak dikerjakan di baseline B. (5) Effort A ±1.967, B ±1.791, C ±1.718, D ±1.337; kebutuhan Fase 1 ±18,5, Fase 2 ±43,0, Fase 3 B ±50,3. (6) §1, §9 (G-22 PT-52, G-23 PT-53), §11, §12, §13 diperbarui. KEPUTUSAN-215–223 |
| 0.3 | 2026-11-19 | CR-021 skenario S3 `[USER] C21-P4`. (1) Fase 1: F1-T81–F1-T111 (31 task, 109 jam-task; situs, CMS US-1101 dari Fase 3, pendaftaran dan verifikasi perusahaan oleh Verifikator, akun HR); F1-T57, F1-T67, F1-T75–F1-T79 pindah ke Fase 2; F1-T80 didefinisikan ulang; F1-T71 bergantung juga pada F1-T110; total 104 task, 361 jam-task; jalur kritis tetap 65. (2) Fase 2: +25 pindahan + reset MFA HR (M 12) = 725 (45,3 per hari kerja). (3) Fase 3: −12 (US-1101) + K2 195 + K3 36 = 973 (64,9). (4) Opsi B ±2.131 (+340), Opsi A ±2.307; peringatan dependensi Opsi C/D atas K2/K3. (5) §6.7 baru; §1, §4, §7.1, §7.3, §7.4, §8.1, §8.2, §9 (G-24–G-27) diperbarui. KEPUTUSAN-260–263 |
| 0.4 | 2026-11-20 | Kerja 7 hari per minggu `[USER]` 2026-11-19 (mengubah C2-Q7) dan CR-022. (1) Hitungan memakai hari kalender: Fase 1 19 (sisa dari 2026-11-20: 22), Fase 2 26, Fase 3 19, Februari 12, mobile developer 75; §6.2a, §6.2 (tabel v0.4), §5, §4. (2) Kebutuhan baru Opsi B + CR-021: Fase 1 sisa 13,1 (backend/web 11,4; mobile 1,7), Fase 2 27,9, Fase 3 51,2, Februari 6,0 jam-task per hari kalender; effort ±2.131 tidak berubah; per minggu sebagai pembanding. (3) CR-022: urutan Fase 1 baseline dulu lalu F1-T81–F1-T111, F1-T71 terakhir; toleransi waktu 0 diubah (go-live boleh mundur); gerbang 2026-12-11 menetapkan tanggal go-live baru, bukan memilih opsi pemotongan scope; scope tetap Opsi B + CR-021 (§2, §3, §6.5, §6.6, §9 G-05, §10). (4) Progres per 2026-11-20: 73 dari 361 jam-task (20,2%), 22 dari 104 task; sisa 288 (§6.8.3, §7.3a). (5) F1-T19 bergantung pula pada F1-T18. (6) §6.8 baru, kontradiksi P-1–P-6. KEPUTUSAN-281–284 |
