# Phase Plan: Ruang Sehat

Tanggal: 2026-11-18
Versi: 0.1 (draf Tahap 7, menunggu Checkpoint 2)
Penyusun: Claude (persona Project Manager, dengan tinjauan Tech Lead atas urutan dan dependensi task), Mode Otomatis
Pemilik dan pemutus: user (project manager, meneruskan ke CEO dan CTO) `[DOK: 00-project-brief.md]`
Dasar: Business Case v1.0 `[DOK: 01-business-case.md]`, BRD v1.3 `[DOK: 02-brd.md]`, PRD v1.1 (+ revisi CR-013 di v1.2 draf) `[DOK: 03-prd.md]`, arsitektur v1.0 dan ADR-006–ADR-025 `[DOK: 04-arsitektur.md]`, stack v1.0 `[DOK: 05-stack.md]`, coding standards `[DOK: coding-standards.md]`, design guideline v0.1 `[DOK: 06-design-guideline.md]`, backlog `[DOK: backlog.md]`, risk register `[DOK: 04-risk-register.md]`
Tingkat ketelitian: T2 Standar. Urutan, dependensi, dan definisi task modul kritis T1 (daftar di `CLAUDE.md`) ditinjau teliti.

**Label sumber.** `[DOK: ...]` dokumen proyek yang lulus gate. `[USER] Tx-Qy` jawaban user yang tercatat di catatan tahap. `[KEPUTUSAN-xx]`, `[ASUMSI-xx]` tercatat di `docs/decision-log.md`. `[UP-xx]` usulan penyusun yang **wajib** diputuskan user di Checkpoint 2 (§12). `[SRC-xx]` tercatat di `docs/sources.md`. `[TIDAK DIKETAHUI]` masuk pertanyaan terbuka (§9, §12). Angka jam dan tanggal di dokumen ini adalah **estimasi atau usulan**, bukan fakta, dan dasar hitungnya ditulis di sampingnya.

**Semua tanggal fase, isi fase, dan opsi scope di dokumen ini adalah usulan.** Keputusan scope, penempatan CR, tanggal, biaya, dan kapasitas tim diambil user di Checkpoint 2.

---

## 1. Ringkasan untuk Checkpoint 2

- **Empat fase pengembangan** (Tahap 8) dalam jendela 2026-11-23 sampai 2027-01-29, lalu QA/SIT, audit keamanan, UAT, persiapan rilis, dan deploy sampai go-live 1 Maret 2027 `[UP-65]`:
  - **Fase 1** (2026-11-23 – 2026-12-11): fondasi (repo, `scripts/verify.sh`, CI, skema inti, autentikasi + RBAC, Audit Trail Foundation, enkripsi, berkas, BFF, kerangka mobile), reference implementation, dan MVP tertipis "pasien terverifikasi" (daftar, persetujuan, masuk, profil, unggah KTP, diverifikasi Verifikator di backoffice).
  - **Fase 2** (2026-12-14 – 2027-01-08): konsultasi online dokter dan psikolog end-to-end, termasuk pembayaran, refund otomatis, notifikasi, aplikasi tenaga kesehatan, staging, dan observability.
  - **Fase 3** (2027-01-11 – 2027-01-29): Partner Integration API, RME minimalis, pendaftaran klinik/lab, informasi biaya, hasil lab, klaim, rekonsiliasi, portal rekanan, CMS dan keluhan, halaman audit log, hak subjek data, KPI.
  - **Fase 4** (2027-01-18 – 2027-01-29, paralel dengan akhir Fase 3, **hanya bila kapasitas terbukti cukup**): antrean real-time, pembelian produk, home visit, anggota keluarga. Isinya adalah nomor 1–3 urutan pengurangan scope PRD §2.3, sehingga bila kapasitas kurang, fase ini dilepas utuh tanpa merusak fase lain `[UP-66]`.
- **Effort total estimasi ±1.857 jam-task (±608 task: 71 task nyata Fase 1 + ±537 hasil jam ÷ 3 untuk Fase 2–4 dan Februari)** ditambah ±42 jam-task untuk story yang masih terblokir PT-24 (§6) `[KEPUTUSAN-178]`.
- **Kesimpulan kelayakan: scope penuh (Opsi A) tidak dapat dinyatakan layak dengan data saat ini, dan indikasinya kuat tidak layak.** Kecepatan yang dibutuhkan naik dari ±16,3 jam-task per hari kerja (Fase 1) ke ±31,1 (Fase 2) dan ±61,2 (Fase 3 + Fase 4). Kapasitas tim yang tercatat (PM, Claude lewat runbook, 1 mobile developer penuh waktu) tidak punya angka throughput `[TIDAK DIKETAHUI]` (PT-48). Keputusan scope definitif diusulkan di **gerbang re-plan 2026-12-11**, berdasarkan kecepatan terukur di Fase 1 (§6.5) `[UP-66]`.
- **Opsi** (§6.6): A semua; B lepas nomor 1–3 (antrean, produk, home visit); C lepas nomor 1–5 (ditambah fisioterapis/tenaga medis serta lab, info biaya, hasil lab); D hanya konsultasi dokter dan psikolog + kewajiban kepatuhan; E tambah kapasitas. Usulan penyusun: **B sebagai baseline rencana, keputusan final di gerbang re-plan, dan pertanyaan kapasitas tambahan (E) dijawab sekarang** karena waktu rekrut `[UP-67]`.
- **CR-011** (onboarding mandiri dan langganan RME, ±108 jam-task) dan **CR-015** (vaksinasi/infus, ±50–160 jam-task) diusulkan ke **rilis berikutnya**; opsi lain dan dampaknya di §11 `[UP-68]` `[UP-69]`.
- **Temuan teknis yang perlu ADR baru di awal Fase 1**: Keycloak mengirim email (verifikasi email, atur ulang password, undangan) lewat SMTP `[SRC-264]`, sedangkan endpoint SMTP Amazon SES tidak tersedia di region Jakarta `[SRC-84]`. Jalur email dirancang ulang lewat spike F1-T02 dan diusulkan sebagai ADR ke user `[KEPUTUSAN-183]`.

---

## 2. Dasar perencanaan

| Butir | Isi | Sumber |
| --- | --- | --- |
| Go-live | 1 Maret 2027, toleransi waktu 0 hari | `[DOK: 01-business-case.md]` §6.2, §7.1 |
| Bila syarat go-live belum terpenuhi | Tanggal tetap, scope dikurangi; syarat izin dan kepatuhan, audit keamanan, dan UAT tidak bisa dikurangi | `[DOK: 00-project-brief.md]` P8.3 |
| Urutan pengurangan scope | (1) antrean real-time; (2) obat bebas, lalu seluruh produk; (3) home visit; (4) konsultasi fisioterapis dan tenaga medis; (5) pendaftaran lab, informasi biaya MCU/lab, hasil lab; (6) pendaftaran klinik. Konsultasi dokter dan psikolog dipertahankan paling akhir. Fitur RME minimalis ikut layanan pemakainya | `[DOK: 03-prd.md]` §2.3 (`[USER] T3-Q2`, UP-33) |
| Kelayakan scope | Dinilai di Tahap 7 dengan estimasi effort (G4) | `[DOK: 03-prd.md]` §9 (`[USER] T3-Gate`) |
| Tim | Project manager (user), Claude lewat runbook, 1 mobile developer | `[DOK: 00-project-brief.md]` P7.3 |
| Status developer | Penuh waktu sampai go-live; setelah go-live PT-35 | `[USER] T5-Q7` |
| Biaya tim selain developer | Di luar anggaran proyek | `[USER] T5-Q9` |
| Jam kerja per minggu developer, pembagian kerja mobile, jumlah sesi Claude per hari | `[TIDAK DIKETAHUI]` | PT-48 |
| Lingkungan | Staging Biznet Gio mulai Desember 2026; uji beban Januari 2027 (satu bulan); production mulai 1 Februari 2027 | `[DOK: 05-stack.md]` §2.9, §11.2 |
| Siklus per fase | Setiap fase ditutup dengan functional test QA, consistency review, dan laporan fase | `[DOK: .claude/skills/runbook/references/tahap-08-development.md]` |
| Scope rilis 1 | 96 story aktif (91 Must, 5 Should: US-108, US-109, US-405, US-602, US-1305), US-102 Won't | `[DOK: 03-prd.md]` §5, `[DOK: backlog.md]` |
| Mesin pengembangan saat ini | Darwin 21.6.0 (informasi lingkungan sesi kerja). Apakah toolchain yang dipatok di stack dapat dijalankan di versi OS ini `[TIDAK DIKETAHUI]`, diperiksa di F1-T01 | `[DOK: .claude/skills/runbook/references/tahap-08-development.md]` langkah 0 |

---

## 3. Prinsip pembagian fase

1. **Fase 1 hanya Must** dan memuat semua enabler fondasi: skema inti, autentikasi dan RBAC, Audit Trail Foundation, CI pipeline, dan reference implementation `[DOK: .claude/skills/runbook/references/tahap-07-phase-plan.md]`.
2. **MVP Fase 1 adalah slice "pasien terverifikasi"**, bukan konsultasi berbayar. Alasannya teknis: slice ini menembus semua lapis (aplikasi pasien → API → database → audit → backoffice) dan semua fondasi T1 (autentikasi dua realm, otorisasi, audit, enkripsi NIK, berkas identitas) tanpa bergantung pada penyedia yang konfirmasinya belum ada (DOKU PT-36, Lalamove PT-37, Biznet Gio PT-39) `[KEPUTUSAN-180]`.
3. **Urutan layanan mengikuti kebalikan urutan pengurangan scope.** Konsultasi dokter dan psikolog (dipertahankan paling akhir `[USER] T3-Q2`) dibangun pertama setelah fondasi; antrean, produk, dan home visit (dilepas pertama) dibangun terakhir. Bila kapasitas kurang, pengurangan scope berarti melepas fase terakhir, bukan memotong fitur yang sudah setengah jadi `[UP-66]`.
4. **Skema dan autentikasi sebelum fitur; pustaka T1 dibuat sekali** di Fase 1 dan dipakai ulang (RT-10 mitigasi 3) `[DOK: 04-risk-register.md]`.
5. **Gerbang re-plan di akhir Fase 1**: kecepatan terukur (task selesai per hari kerja, dengan `scripts/verify.sh` hijau dan review lolos) dibandingkan dengan kebutuhan per opsi (§6.5), lalu user memilih opsi scope `[UP-66]`. Ini juga menjawab RT-24 (kecepatan Rust diukur di reference implementation) `[DOK: 04-risk-register.md]`.
6. **Story boleh selesai bertahap per acceptance criteria** bila satu AC bergantung pada story di fase berikutnya. AC yang ditunda ditulis eksplisit di backlog dan diverifikasi di fase tujuan `[KEPUTUSAN-179]`.
7. **Dependensi eksternal menjadi gerbang bertanggal** (§9). Task yang bergantung pada gerbang yang belum terbuka tidak dimulai; urutan task lain di fase yang sama disusun ulang.

---

## 4. Fase dan deliverable

| Fase | Tanggal (usulan) | Isi utama | Deliverable (bukti di akhir fase) | Gerbang masuk |
| --- | --- | --- | --- | --- |
| 1. Fondasi + MVP "pasien terverifikasi" | 2026-11-23 – 2026-12-11 (15 hari kerja) | EN-01–EN-14, UI-01–UI-03; US-101, US-103, US-104 (kecuali AC4), US-105, US-106, US-201 (kecuali notifikasi di AC2), US-208, US-1501, US-1504 | `scripts/verify.sh` dan CI hijau; stack lokal; skema DB aplikasi dan audit dengan test hak role; realm `pasien` dan `admin`; RBAC default deny; audit outbox → relay → rantai hash → verifikasi; reference implementation tercatat di `CLAUDE.md`; demo end-to-end: pasien daftar → masuk → unggah KTP → Verifikator menyetujui → status Disetujui, rantai audit terverifikasi; laporan kecepatan dan cakupan baseline; `docs/fase-1-laporan.md` | Checkpoint 2 disetujui; persetujuan user atas `scripts/verify.sh` (F1-T05) |
| Gerbang re-plan | 2026-12-11 | Kecepatan terukur dibandingkan kebutuhan per opsi (§6.5) | Keputusan user: opsi scope A/B/C/D dan/atau E | Fase 1 lulus gate fase |
| 2. Konsultasi online + pembayaran | 2026-12-14 – 2027-01-08 (20 hari kerja; libur belum dikurangi) | EN-15–EN-20, EN-24, UI-04; US-107, US-202–US-207, US-301–US-311, US-801–US-806, US-808, US-809, US-812, US-1001–US-1003, US-1406 (pembayaran, video, email), US-104 AC4, US-201 AC2 (notifikasi) | Pasien memesan dan membayar sesi (sandbox DOKU), chat dan video, catatan, resep + halaman verifikasi publik, riwayat; refund otomatis dan eksekusinya; aplikasi tenaga kesehatan dengan MFA dan kunci 15 menit; onboarding tenaga kesehatan dan klinik; markup dan parameter; notifikasi email dan di dalam aplikasi; staging dengan deploy otomatis; observability; perubahan data dictionary pajak (CR-013) | Gerbang re-plan; PT-39 (staging); PT-47; PT-36 sebelum task pembayaran (target 2026-12-21); sisa PT-28 sebelum perhitungan pajak |
| 3. Integrasi rekanan, RME, klinik/lab, kepatuhan | 2027-01-11 – 2027-01-29 (15 hari kerja) | EN-21–EN-23, EN-25, UI-05; US-401–US-404, US-406, US-601, US-602, US-807, US-810, US-811, US-901, US-902, US-1101–US-1104, US-1201–US-1203, US-1301–US-1304, US-1306, US-1307, US-1309, US-1310, US-1401–US-1405, US-1502, US-1503, US-1601–US-1606, US-1701 | Partner Integration API v1 + sandbox; RME minimalis lulus skenario kontrak yang sama; pendaftaran klinik/lab dibayar di aplikasi dan check-in lewat sistem rekanan/RME; hasil lab; pengiriman SATUSEHAT (sandbox, bila PT-32 tersedia); klaim dan rekonsiliasi; portal rekanan; CMS dan keluhan; halaman audit log dan ekspor; hak subjek data; dasbor KPI; uji beban NFR-CAP-04 | PT-05 (rekanan dan kemampuan sistemnya); PT-32 untuk uji US-1309; PT-24 untuk US-1605/US-1606 |
| 4. Layanan yang dilepas pertama (bersyarat) | 2027-01-18 – 2027-01-29, paralel dengan Fase 3 | US-108, US-109, US-405, US-1305, US-501–US-504, US-1308, US-701–US-706, US-1406 (kurir) | Anggota keluarga; antrean real-time; home visit (penugasan dari sistem klinik/RME); pembelian produk dengan kurir Lalamove, pelacakan, klaim barang | Gerbang re-plan memilih Opsi A; PT-29, PT-30, PT-33, PT-37 terjawab |
| Pembekuan fitur | 2027-01-29 | Tidak ada fitur baru setelah tanggal ini; hanya perbaikan bug | — | — |

Story Should (US-108, US-109, US-405, US-602, US-1305) tidak ada di Fase 1. Penempatan per story ada di `docs/backlog.md` dan `docs/traceability.md` §"Story → fase".

---

## 5. Kalender sampai go-live (usulan)

| Jendela | Tanggal (usulan) | Isi | Butir `[TIDAK DIKETAHUI]` | Sumber |
| --- | --- | --- | --- | --- |
| Checkpoint 2 | sebelum 2026-11-23 | Persetujuan design guideline, phase plan, dan opsi | — | `[UP-65]` |
| Tahap 8 Fase 1–4 | 2026-11-23 – 2027-01-29 | §4 | Kecepatan tim (PT-48) | `[UP-65]` |
| Staging | mulai Fase 2 (Desember 2026) | Deploy otomatis dari `main` hijau | Konfirmasi Biznet Gio (PT-39) | `[DOK: 05-stack.md]` §2.9 |
| Uji beban NFR-CAP-04 | Januari 2027 | Staging dinaikkan ke ukuran production | Kapasitas host video untuk 100 sesi | `[DOK: 05-stack.md]` §2.9, §9 |
| Tahap 9 QA + SIT | 2027-02-01 – 2027-02-05 | SIT dengan rekanan pertama | Kesiapan rekanan (PT-05) | `[UP-65]` |
| Tahap 10 audit keamanan | 2027-02-01 – 2027-02-12 | Audit eksternal (syarat go-live 6) | Auditor, ketersediaan, dan lama audit (PT-49) | `[UP-65]` `[DOK: 01-business-case.md]` §4.2 |
| Production disiapkan | mulai 2027-02-01 | Host, OpenBao (unseal PT-38), replikasi, backup, uji pemulihan NFR-DR-03, deploy pertama lewat bastion (EN-26, EN-27) | Pemegang share unseal dan orang kedua (PT-38) | `[DOK: 05-stack.md]` §2.9 |
| Tahap 11 UAT | 2027-02-08 – 2027-02-12 | Sign-off CEO, CTO, konsultan medis | Ketersediaan peserta (PT-50) | `[UP-65]` `[DOK: 01-business-case.md]` §4.2 |
| Tahap 12 pre-deploy | 2027-02-08 – 2027-02-19 | Pengajuan ke Play Store dan App Store paling lambat 2027-02-12 | Lama review toko dan status akun developer (R-08, PT-51) | `[UP-65]` |
| Tahap 13 deploy | 2027-02-22 – 2027-02-26 | Deploy production final, smoke test | — | `[UP-65]` |
| Go-live | 2027-03-01 | — | — | `[DOK: 01-business-case.md]` |

Lama setiap jendela setelah pembekuan fitur adalah usulan penyusun. Bila auditor atau toko aplikasi butuh waktu lebih lama, satu-satunya ruang adalah memajukan pembekuan fitur (artinya mengurangi scope), karena tanggal go-live tetap `[DOK: 00-project-brief.md]` P8.3.

---

## 6. Estimasi effort dan analisis kapasitas

### 6.1 Metode `[KEPUTUSAN-178]`

- **Fase 1 dihitung dari bawah**: 71 task dengan ukuran 2, 3, atau 4 jam-task (§7.3), total 245 jam-task.
- **Fase 2–4 dihitung dari atas** per story dengan kelas ukuran yang dikalibrasi dari pecahan Fase 1, setelah fondasi tersedia: S = 2 task (6 jam-task), M = 4 task (12), L = 7 task (21), XL = 12 task (36). Satu task rata-rata 3 jam-task. Kelas per story ada di §8.
- **Jam-task** adalah ukuran usaha untuk satu task 2–4 jam termasuk `scripts/verify.sh` dan satu putaran perbaikan review `[ASUMSI-01]`, bukan jam kalender dan bukan jam kerja orang tertentu.
- **Hari kerja** dihitung Senin–Jumat. Libur nasional, cuti bersama, dan libur akhir tahun belum dikurangi `[TIDAK DIKETAHUI]`, sehingga kebutuhan per hari kerja yang sebenarnya lebih tinggi dari angka di bawah.
- **Ketidakpastian**: kecepatan pengembangan Rust belum diukur (RT-24) `[DOK: 04-risk-register.md]`, dan kemampuan sistem rekanan belum diketahui (PT-05). Angka di bawah dipakai untuk membandingkan opsi, bukan sebagai janji.

### 6.2 Effort per fase

| Fase | Story | Enabler, UI, penutupan fase | Total jam-task | Task | Hari kerja | Jam-task per hari kerja |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | ±71 (di dalam 71 task) | ±174 | 245 | 71 (nyata, dari §7.3) | 15 | 16,3 |
| 2 | 510 | 112 | 622 | ±207 (jam ÷ 3) | 20 | 31,1 |
| 3 | 588 (+42 terblokir PT-24) | 130 | 718 (+42) | ±239 (+14) (jam ÷ 3) | 15 | 47,9 |
| 4 (bersyarat) | 192 | 8 | 200 | ±67 (jam ÷ 3) | 10 (paralel) | 20,0 |
| Fase 3 + Fase 4 paralel | | | 918 | ±306 | 15 | 61,2 |
| Production dan uji pemulihan (Februari) | — | 72 | 72 | 24 (jam ÷ 3) | 10 | 7,2 |
| **Total** | | | **1.857 (+42)** | **±608 (+14)** | | |

**Satuan perbandingan.** Kolom "Task" mencampur dua satuan: Fase 1 berisi 71 task nyata (rata-rata ±3,45 jam-task per task), sedangkan Fase 2–4 dan Februari adalah jam-task ÷ 3. Karena itu kebutuhan antarfase dibandingkan dalam **jam-task per hari kerja**, bukan task per hari kerja.

Rincian enabler Fase 2–4: EN-15 staging dan CD 24, EN-16 observability 20, EN-17 realm nakes dan kunci aplikasi 16, EN-18 LiveKit 8, EN-19 realtime 12, EN-20 kerangka portal 10, EN-24 pajak CR-013 8, UI-04 template email 6, EN-21 gateway integrasi 30, EN-22 fondasi RME 55, EN-23 pekerja privasi 15, EN-25 uji beban 16, UI-05 uji aksesibilitas manual 6, EN-26 production 60, EN-27 uji pemulihan 12, penutupan fase 8 per fase (estimasi penyusun per enabler) `[KEPUTUSAN-178]`.

**Porsi mobile** (aplikasi pasien dan tenaga kesehatan) diperkirakan ±430 jam-task: Fase 1 ±37 (dari daftar task), Fase 2 ±180, Fase 3 ±120, Fase 4 ±90 (estimasi kasar per fase berdasarkan story yang punya layar mobile) `[KEPUTUSAN-178]`. Sisanya, ±1.430 jam-task, adalah backend Rust, web Next.js, infrastruktur, dan penutupan fase.

### 6.3 Kapasitas tim yang tercatat

| Anggota | Yang tercatat | Yang belum tercatat | Sumber |
| --- | --- | --- | --- |
| Project manager (user) | Menjalankan runbook, memutuskan di checkpoint | Waktu yang tersedia untuk menjalankan sesi Claude per hari | `[DOK: 00-project-brief.md]` P7.3; PT-48 |
| Claude lewat runbook | Mengerjakan task Tahap 8 per sesi dengan review subagent | Throughput (jam-task per hari kerja) `[TIDAK DIKETAHUI]`; diukur di Fase 1 | `[DOK: 00-project-brief.md]` P7.3; RT-24 |
| Mobile developer (1 orang) | Penuh waktu sampai go-live | Jam kerja per minggu dan apakah seluruh task mobile dikerjakannya `[TIDAK DIKETAHUI]` | `[USER] T5-Q7`; PT-48 |
| QA manusia, DevOps, UI/UX | Tidak disebut | — | `[DOK: 01-business-case.md]` R-04 |

Penyusun tidak mengisi angka kapasitas yang tidak tercatat. Angka ini dibawa ke Checkpoint 2 (PT-48).

### 6.4 Kebutuhan dibandingkan kapasitas

- **Seluruh tim**: kebutuhan naik dari ±16,3 jam-task per hari kerja (Fase 1) ke ±31,1 (Fase 2) dan ±47,9 (Fase 3), atau ±61,2 bila Fase 3 dan Fase 4 berjalan paralel (918 jam-task dalam 15 hari kerja). Throughput tim `[TIDAK DIKETAHUI]`.
- **Durasi minimum Fase 1**: jalur kritis 65 jam-task berurutan (§7.4) dari 245 jam-task. Sisa 180 jam-task hanya muat di 15 hari kerja bila dikerjakan paralel di jalur backend, web, dan mobile; jalur kritis sendiri memuat 15 jam-task mobile (F1-T58, F1-T63) dan pola reference implementation (F1-T66) sebelum profil.
- **Mobile developer**: bila seluruh porsi mobile (±430 jam-task) dikerjakan satu orang, dibutuhkan rata-rata ±7,8 jam-task per hari kerja selama ±55 hari kerja (2026-11-23 – 2027-02-05), dengan puncak ±9 jam-task per hari kerja di Fase 2 (180 ÷ 20). Perhitungan ini belum mengurangi libur dan belum memasukkan perbaikan bug dari QA, SIT, audit, dan UAT.
- **Backend, web, infrastruktur** (±1.430 jam-task) bergantung pada throughput Claude yang dijalankan PM, ditambah review subagent per task. Angka throughput `[TIDAK DIKETAHUI]`.
- **Risiko yang menaikkan kebutuhan**: RT-24 (Rust lebih lambat), RT-01 (rekanan tidak mampu mengikuti kontrak), RT-05 (registrasi SATUSEHAT), RT-27 (penyedia diganti), dan putaran review T1 (maksimal 2 putaran per `aturan-kerja.md` §1) `[DOK: 04-risk-register.md]`.

### 6.5 Kesimpulan kelayakan dan kriteria gerbang re-plan

**Kesimpulan:** jadwal 1 Maret 2027 untuk scope penuh **tidak dapat dinyatakan layak**. Indikasinya kuat tidak layak: kebutuhan kecepatan Fase 3 + Fase 4 (±61,2 jam-task per hari kerja) hampir empat kali kebutuhan Fase 1 (±16,3), dalam tim yang tercatat tanpa peran QA, DevOps, atau developer backend manusia, dengan toleransi waktu 0. Opsi yang lebih kecil (§6.6) menurunkan kebutuhan, tetapi kelayakannya juga baru dapat dinilai setelah kecepatan terukur.

**Kriteria gerbang re-plan 2026-12-11** `[UP-66]`. Kecepatan terukur `v` = jumlah ukuran (kolom "Jam" §7.3) task yang selesai per hari kerja, dirata-rata selama 10 hari kerja terakhir Fase 1 (selesai = commit hijau + `code-reviewer` lolos, anti-halu §5). Satuan ini sama dengan estimasi Fase 2–4.

| Opsi | Jam-task per hari kerja yang dibutuhkan di Fase 2 | Di Fase 3 (+ Fase 4 bila ada) |
| --- | --- | --- |
| A | 31,1 | 61,2 (918 ÷ 15) |
| B | 31,1 | 49,5 (742 ÷ 15) |
| C | 31,1 | 44,6 (669 ÷ 15) |
| D | 31,1 | 20,0 (300 ÷ 15) |

- `v` di bawah kebutuhan Fase 2 berarti Fase 2 sendiri tidak muat. Pilihan yang tersisa: E (tambah kapasitas) atau memotong isi Fase 2 (misalnya video atau resep), yang bukan bagian urutan P8.3 sehingga butuh keputusan user sebagai CR.
- Bila Fase 1 tidak selesai 2026-12-11, gerbang tetap dilaksanakan pada tanggal itu dengan data yang ada, karena setiap minggu keterlambatan mengurangi jendela Fase 2–3.

### 6.6 Opsi scope dan dampaknya

| Opsi | Isi | Effort (jam-task) | Dampak bisnis | Sumber dampak |
| --- | --- | --- | --- | --- |
| A. Semua | Fase 1–4 | ±1.857 (+42) | Ketujuh layanan berjalan (syarat go-live 3). Butuh kecepatan tertinggi | `[DOK: 01-business-case.md]` §4.2 |
| B. Lepas nomor 1–3 | Tanpa antrean real-time, pembelian produk, dan home visit; anggota keluarga (US-108, US-109) dipindah ke Fase 3 | ±1.681 | Syarat go-live 3 ("ketujuh layanan berjalan") tidak terpenuhi, yang diizinkan P8.3; dua layanan (produk, home visit) dan antrean menyusul setelah go-live; syarat go-live 1 bagian "3 outlet terintegrasi" tidak terpenuhi; pemasaran menyesuaikan (R-07) | `[DOK: 03-prd.md]` §2.3; `[DOK: 01-business-case.md]` R-07 |
| C. Lepas nomor 1–5 | B + konsultasi fisioterapis/tenaga medis (dinonaktifkan per jenis, effort kecil) + pendaftaran lab, info biaya, hasil lab (EP-06, EP-09, US-1307, bagian lab EP-04) | ±1.608 | Syarat go-live 3 tidak terpenuhi (diizinkan P8.3), dan syarat go-live 1 bagian lab dan outlet juga tidak; tersisa konsultasi dokter dan psikolog serta pendaftaran klinik; RME minimalis tetap dibangun untuk klinik | `[DOK: 03-prd.md]` §2.3 |
| D. Lepas nomor 1–6 | Hanya konsultasi dokter dan psikolog, pembayaran, dan fitur kepatuhan; tanpa Partner Integration API, RME minimalis, klinik/lab | ±1.239 | Syarat go-live 1 (rekanan terintegrasi) dan 3 (tujuh layanan) tidak terpenuhi, yang diizinkan P8.3. **Perlu jawaban PT-47**: klinik penaung tenaga kesehatan tanpa integrasi tidak memenuhi syarat tampil US-205 AC2. CR-006 (RME minimalis) tidak berjalan di rilis 1 | `[DOK: 00-project-brief.md]` P8.3; `[DOK: 03-prd.md]` US-205 AC2, UP-33 |
| E. Tambah kapasitas | Developer backend/web dan/atau QA tambahan | Effort sama; kebutuhan per orang turun | Biaya di luar baseline (toleransi +20% `[USER] T1-Q8`); waktu rekrut dan onboarding `[TIDAK DIKETAHUI]`; keputusan CEO | `[DOK: 01-business-case.md]` §6.1, §7.1 |
| — Tanggal mundur | Tidak tersedia | — | Toleransi waktu 0 | `[DOK: 01-business-case.md]` §7.1 |

Effort B, C, D dihitung dari tabel §6.2 dikurangi story yang dilepas: B = A − 176 (Fase 4 tanpa US-108/US-109); C = B − 73 (EP-06 18, EP-09 24, US-1307 12, bagian lab EP-04 ±19); D = B − 442 (EP-04 63, EP-06 18, EP-09 24, EP-13 tanpa US-1305/US-1308 156, EP-14 96, EN-21 30, EN-22 55) `[KEPUTUSAN-178]`.

**Usulan penyusun** `[UP-67]`: pakai Opsi B sebagai baseline rencana (Fase 4 tidak dikerjakan kecuali gerbang re-plan memilih A), putuskan opsi final di gerbang re-plan 2026-12-11, dan jawab pertanyaan kapasitas tambahan (E) sekarang karena rekrut butuh waktu. Keputusan ada di user.

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

### 7.2 Tinjauan Tech Lead: urutan dan alasannya

1. **Mesin dan keputusan teknis pembuka dulu** (F1-T01–T03). Toolchain harus dapat dijalankan di mesin pengembangan sebelum task kode pertama (runbook Tahap 8 langkah 0). Dua gap teknis (jalur email Keycloak, penyimpanan berkas lokal) diselesaikan sebagai spike karena memblokir US-101, US-103, dan US-106.
2. **Commit kode pertama wajib menyertakan `scripts/verify.sh`** (pola bootstrap di runbook Tahap 8), jadi F1-T05 mendahului semua task kode lain.
3. **Database sebelum layanan**: bootstrap role → fungsi `util` → test katalog hak role → tabel. Test katalog (F1-T15) dibuat sebelum tabel bisnis, sehingga setiap migrasi berikutnya langsung diuji hak role dan trigger penolaknya (ADR-008, ADR-021 butir 4).
4. **Audit, autentikasi, otorisasi, dan kriptografi sebelum story**. Semua story Fase 1 menulis event audit dan membaca konteks autentikasi; US-106 butuh enkripsi NIK dan berkas.
5. **Reference implementation sebelum story lain yang serupa**. US-106 → US-201 dibangun lebih dulu dan didokumentasikan (F1-T66) sebelum US-105, US-208, dan registrasi diselesaikan, supaya story lain mengikuti polanya (Definition of Done "mengikuti pola reference implementation").
6. **Tiga jalur paralel setelah fondasi**: backend Rust, web Next.js, mobile Flutter. Jalur mobile dapat dikerjakan mobile developer begitu kontrak API (OpenAPI) tersedia.

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
| F1-T19 | Migrasi A3: `consent_text_version`, `consent_item`, `consent_record` + hak role | EN-04 | 2 | F1-T16 | T1 | DBA, Sec |
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
| F1-T57 | US-105 AC4: ubah email lewat Keycloak + verifikasi + pemberitahuan ke email lama, autentikasi ulang (ADR-011 butir 9) | US-105 | 3 | F1-T56, F1-T36 | T1 | Sec |
| F1-T58 | Mobile: profil, daftar dan formulir alamat, ubah email; tambahkan deskripsi D-A03a di design guideline untuk formulir alamat dan ubah email (catatan QA N3) | US-105 | 4 | F1-T45, F1-T46, F1-T56 | T2 | — |
| F1-T59 | RI (1) domain: status AB-05 dan transisi lewat jalur tulis status ADR-008 (R/P), aturan UP-40; unit test | US-106, EN-14 | 3 | F1-T18 | T1 | DBA |
| F1-T60 | RI (2) service + repository pengajuan identitas: NIK 16 digit terenkripsi + blind index, foto KTP lewat upload slot, NIK pemilik akun ganda ditolak (AC4), foto terkunci saat menunggu (AC5), batal pengiriman, E-10 | US-106, EN-14 | 4 | F1-T59, F1-T28, F1-T29, F1-T38 | T1 | Sec, DBA |
| F1-T61 | RI (3) handler, OpenAPI, `response_code` baru terdaftar, contract test, lima skenario penolakan endpoint (coding standards §7.3) | US-106, EN-14 | 3 | F1-T60 | T1 | Sec |
| F1-T62 | RI (4) Verifikator: antrean terlama, buka foto (E-05 dulu), setujui/tolak dengan `reason_code`, pemeriksaan KK, keputusan pertama yang berlaku (AC5), NIK sudah dipakai (AC7), E-07 | US-201, EN-14 | 4 | F1-T61, F1-T30 | T1 | Sec, DBA |
| F1-T63 | RI (5) mobile: unggah NIK + KTP (kamera/galeri), status AB-05, alasan penolakan, unggah ulang (A03) | US-106 | 4 | F1-T58, F1-T61 | T2 | — |
| F1-T64 | RI (6) backoffice panel Verifikator (C02): daftar + panel detail, jam berlabel WIB (catatan QA N5b), foto hanya di panel Verifikator; e2e Playwright | US-201 | 4 | F1-T43, F1-T62 | T2 | Sec |
| F1-T65 | US-208: bootstrap Super Admin pertama lewat prosedur terkontrol yang tercatat; kelola akun admin dan sub-peran (AC1–AC3), MFA saat pertama masuk, E-10/E-11; test Super Admin terakhir dan ubah peran sendiri | US-208 | 4 | F1-T37, F1-T31, F1-T23 | T1 | Sec |
| F1-T66 | RI (7) dokumentasi pola reference implementation (lokasi di `CLAUDE.md`, checklist pola), pengukuran kecepatan awal (RT-24) dan baseline cakupan (KEPUTUSAN-134) | EN-14 | 2 | F1-T61, F1-T62 | T2 | — |
| F1-T67 | Backoffice: layar kelola admin dan sub-peran (bagian admin RS dari D-C05) | US-208 | 3 | F1-T43, F1-T65 | T2 | — |
| F1-T68 | US-1504: tidak ada endpoint ubah/hapus audit; permintaan ubah/hapus ditolak dan dicatat sebagai kejadian keamanan; test `UPDATE`/`DELETE`/`TRUNCATE` di `audit_event` ditolak | US-1504 | 2 | F1-T21, F1-T37 | T1 | DBA, Sec |
| F1-T69 | US-1501 AC1–AC3: field wajib §7.2 untuk semua event Fase 1; aksi pengguna batal bila event gagal dicatat, aksi sistem diulang dari antrean, peringatan keamanan | US-1501 | 3 | F1-T24, F1-T62 | T1 | Sec |
| F1-T70 | Uji end-to-end MVP: daftar → verifikasi email → masuk → unggah KTP → Verifikator menyetujui → status Disetujui; rantai audit terverifikasi | Fase 1 | 3 | F1-T53, F1-T54, F1-T63, F1-T64, F1-T69 | T2 | — |
| F1-T71 | Penutupan Fase 1: functional test QA dari `docs/test-plan.md`, consistency review (Opus high), `docs/fase-1-laporan.md` berisi kecepatan terukur untuk gerbang re-plan | Fase 1 | 3 | F1-T70 | T2 | — |

Jumlah: **71 task, 245 jam-task**; 45 task berkelas T1 (jumlah baris T1 di tabel).

### 7.4 Jalur kritis Fase 1

Rantai terpanjang yang harus berurutan (tanpa memperhitungkan siapa yang mengerjakan), dihitung sebagai longest path dari kolom "Jam" dan "Bergantung pada" di §7.3:

**F1-T01 → T04 → T05 → T07 → T13 → T14 → T15 → T17 → T27 → T29 → T60 → T61 → T62 → T66 → T56 → T58 → T63 → T70 → T71**, total **65 jam-task berurutan** (2+3+3+3+3+4+4+4+4+4+4+3+4+2+4+4+4+3+3).

Arti rantai ini:
- Kriptografi dan berkas (T27 → T29) masuk jalur kritis sebelum reference implementation.
- Story profil dan layar mobile (T56 → T58 → T63) ada di jalur kritis karena sengaja menunggu dokumentasi pola reference implementation (T66). Jalur mobile (mobile developer) ikut menentukan selesainya Fase 1.
- Kalau ketergantungan T56 pada T66 dilepas (profil dibangun tanpa menunggu pola tertulis), rantai turun menjadi 62 jam-task (T62 → T64 → T70 → T71 jadi terpanjang), dengan risiko pola profil menyimpang dari reference implementation. Penyusun mempertahankan ketergantungan itu.

Slack rantai lain terhadap F1-T70 (selesai paling awal di jam-task ke-59 pada jalur kritis):
- Web: T39 → T41 → T42 → T43 → T64 selesai paling awal di jam-task ke-49 (slack ±10).
- Audit: T22 → T23 → T24 → T69 selesai paling awal di jam-task ke-48 (slack ±11).
- Registrasi: T36 → T50 → T51 → T53 selesai paling awal di jam-task ke-42 bila ADR jalur email disetujui tanpa jeda (slack ±17).

**Keputusan user di jalur kritis**: persetujuan `scripts/verify.sh` (F1-T05) ada di jalur kritis. Persetujuan ADR jalur email (F1-T02 → F1-T36) **tidak** ada di jalur kritis yang dihitung, tetapi slack-nya hanya ±17 jam-task. Bila persetujuannya tertunda lebih lama, rantai registrasi (US-101, US-103) menjadi jalur kritis dan story itu digeser ke akhir Fase 1.

### 7.5 Pemeriksaan Definition of Ready untuk story Fase 1

| Story | AC jelas dan dapat diuji (PRD) | Rujukan PRD/ADR | Dependensi | Ukuran |
| --- | --- | --- | --- | --- |
| US-101 | Ya | PRD §5 EP-01; ADR-011 butir 6 | F1-T36 (ADR email) — jelas, belum selesai | 4 task |
| US-103 | Ya, kecuali pesan akun terkunci (PT-44) | PRD §5; ADR-011 butir 4, 7 | PT-44 di Checkpoint 2 | 2 task + tema |
| US-104 | Ya; teks AC1 menunggu L-08/PT-24, struktur dapat dibangun | PRD §5; ADR-011 butir 6; ADR-013 | AC4 ke Fase 2 (US-1001) | 2 task |
| US-105 | Ya | PRD §5; ADR-003 | Pola RI (F1-T66) | 3 task |
| US-106 | Ya; format NIK 16 digit ditetapkan data dictionary A2.4 (ADR-013 §4) | PRD §5; ADR-013, ADR-018 | Fondasi Fase 1 | 4 task |
| US-201 | Ya | PRD §5; ADR-008, ADR-009 | Notifikasi AC2 ke Fase 2 | 2 task |
| US-208 | Ya | PRD §5; ADR-011, ADR-012 | Fondasi Fase 1 | 2 task |
| US-1501 | Ya | PRD §5, §7; ADR-009 | Fondasi audit | 1 task + EN-05 |
| US-1504 | Ya | PRD §5; ADR-009 | Fondasi audit | 1 task |

Semua story Fase 1 memenuhi Definition of Ready dengan dependensi yang jelas. Dua dependensi berupa keputusan user (ADR jalur email, PT-44).

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
| Notifikasi | US-1001 L (SES + kotak notifikasi), US-1002 M, US-1003 M, UI-04 template email (D-F02); menutup US-104 AC4 dan pemberitahuan US-201 AC2 | EN-13 | — |

Bila PT-36 belum terjawab 2026-12-21, adaptor DOKU tetap dapat dirancang dari dokumentasi publik `[SRC-203]`–`[SRC-207]`, tetapi tidak difinalkan; risiko penggantian penyedia mengikuti RT-27 `[DOK: 04-risk-register.md]`.

### 8.2 Fase 3: integrasi rekanan, RME minimalis, klinik/lab, kepatuhan

| Kelompok | Story / enabler (kelas) | Dependensi teknis | Gerbang |
| --- | --- | --- | --- |
| Integrasi (T1) | EN-21 gateway Partner Integration API (realm `integrasi` client credentials, HMAC webhook, idempotensi, outbox integrasi, pengaman SSRF, sandbox staging); US-1401 L, US-1402 L, US-1403 L, US-1404 L, US-1405 M | EN-15 | PT-05 |
| RME minimalis (T1) | EN-22 fondasi RME (`rme-backend`, realm `rme`, RLS + konteks tenant, migrasi C1–C6 termasuk CR-014 bagian DB RME dan hak pembersihan `rme_worker`, audit RME, web RME BFF); US-1301 L, US-1302 L, US-1303 L, US-1304 M, US-1306 L, US-1307 M, US-1309 XL, US-1310 M | EN-21 (kontrak sama), EN-04–EN-07 | PT-32 (uji US-1309), PT-20/L-05 |
| Klinik, lab, info biaya | US-401 M, US-402 L (catatan QA N3: layar penerima layanan), US-403 M, US-404 M, US-406 S, US-601 M, US-602 S, US-901 M, US-902 M | EN-21, US-801–US-806 | PT-05 |
| Uang lanjutan (T1) | US-807 L, US-810 M, US-811 L (catatan QA N3: isi tab Rekonsiliasi) | Fase 2 pembayaran | PT-36 |
| Backoffice, portal, konten | US-1101 M, US-1102 S, US-1103 M, US-1104 M, US-1201 L, US-1202 S, US-1203 S, US-1502 M, US-1503 M, US-1701 M | EN-20 | — |
| Privasi (T1) | EN-23 pekerja privasi (kebijakan retensi kosong tidak memproses apa pun); US-1601 M, US-1602 M, US-1603 L, US-1604 L; US-1605 L dan US-1606 L terblokir | EN-09 | PT-24, L-06 |
| Kualitas | EN-25 uji beban NFR-CAP-04 (Januari); UI-05 uji manual TalkBack/VoiceOver alur A01, A05, A07 | EN-15 | — |

### 8.3 Fase 4: layanan yang dilepas pertama (bersyarat)

| Kelompok | Story (kelas) | Gerbang |
| --- | --- | --- |
| Anggota keluarga (Should) | US-108 M, US-109 M | — (di Opsi B dipindah ke Fase 3) |
| Antrean (Should, nomor 1) | US-405 M, US-1305 M | — |
| Produk (nomor 2) | US-701 M, US-702 M, US-703 L, US-704 L, US-705 M, US-706 S, US-1406 (kurir) | PT-29, PT-30, PT-33, PT-37; L-03/PT-21 untuk obat bebas |
| Home visit (nomor 3) | US-501 M, US-502 M, US-503 M, US-504 M, US-1308 M | — |

### 8.4 Production dan uji pemulihan (Februari 2027, sebelum go-live)

EN-26 (host production Biznet Gio dan Hostinger, Docker Compose, Caddy, OpenBao Raft 3 node dengan unseal Shamir, pgBackRest dan standby lewat WireGuard, bastion dengan SSH CA, skrip deploy dengan Control Group; T1, review keamanan dan DBA) dan EN-27 (uji pemulihan NFR-DR-03). Kode infrastrukturnya ditulis di `infra/` sejak Fase 2 bersama staging; penyewaan host mulai 1 Februari 2027 `[DOK: 05-stack.md]` §2.9. Gerbang: PT-38.

---

## 9. Gerbang dan dependensi eksternal

| # | Gerbang | Dibutuhkan sebelum | Bila belum terbuka | Pemilik |
| --- | --- | --- | --- | --- |
| G-01 | Checkpoint 2: phase plan, design guideline, opsi | Fase 1 mulai | Tahap 8 belum dimulai | User |
| G-02 | Persetujuan `scripts/verify.sh` | F1-T05 | Tidak ada commit kode | User |
| G-03 | ADR jalur email (dari F1-T02) | F1-T36, US-101, US-103, US-105 AC4 | Story itu digeser ke akhir Fase 1 | User |
| G-04 | PT-44 pesan akun terkunci | F1-T35, F1-T55 | Tema dan test penguncian tertunda | User + reviewer keamanan |
| G-05 | Gerbang re-plan dengan kecepatan terukur | Fase 2 | Fase 2 dimulai dengan baseline Opsi B | User |
| G-06 | PT-39 konfirmasi Biznet Gio | EN-15 (awal Fase 2), rancangan ingress production | Staging tertunda; deploy hanya lokal | User |
| G-07 | PT-47 status integrasi klinik untuk konsultasi | US-205, US-303 (Fase 2) | Syarat tampil klinik mengikuti US-205 AC2 apa adanya | User/CEO |
| G-08 | PT-42 draf catatan (UP-72) | US-308 (Fase 2) | Draf mengikuti data dictionary A9.6 | User + Tech Lead |
| G-09 | PT-36 konfirmasi tertulis DOKU | Task pembayaran Fase 2 (target 2026-12-21) | Adaptor dirancang tetapi tidak difinalkan (RT-27) | User |
| G-10 | Sisa PT-28 (jenis, tarif, pembulatan, perlakuan atas ongkir dan biaya transaksi, penerima dana pajak, isi bukti bayar, perilaku saat tarif belum diatur) | Perhitungan pajak US-801 (Fase 2); go-live | Komponen pajak dibangun sebagai parameter tanpa nilai | CEO |
| G-11 | PT-05 rekanan pertama dan kemampuan sistemnya | Fase 3 (EP-14), SIT | Integrasi hanya diuji dengan sandbox dan RME | User, CEO |
| G-12 | PT-32 registrasi SATUSEHAT | Uji US-1309 (Fase 3) | Status "Menunggu kredensial" (KEPUTUSAN-16) | User, CEO |
| G-13 | PT-24 retensi (setelah L-05, L-06) | US-1605, US-1606 (Fase 3) | Story tetap terblokir; risiko kepatuhan go-live | CEO + legal |
| G-14 | PT-20 legal (L-01–L-15) | Teks persetujuan (US-104 AC1), resep (L-05), refund (L-14), data minimal (L-08) sebelum go-live | AC terkait tidak dapat difinalkan | User, CEO |
| G-15 | PT-49 auditor keamanan eksternal | Pemesanan jadwal sebelum Fase 3 | Jendela audit Februari tidak terjamin | User, CEO |
| G-16 | PT-29, PT-30, PT-33, PT-37 | Fase 4 (EP-07) | Produk tidak dibangun | CEO, User |
| G-17 | PT-17 nilai markup (PB-11) | UAT dan go-live | Rekanan tidak tampil (US-809 AC3) | CEO |
| G-18 | PT-38 pemegang share unseal dan orang kedua | Production (2027-02-01) | Production tidak dapat diinisialisasi | User + CTO |
| G-19 | PT-50 peserta UAT; PT-51 akun developer toko aplikasi | Februari 2027; pengajuan toko 2027-02-12 | Syarat go-live 4 dan 7 terancam | User, CEO |
| G-20 | PT-35 pemelihara setelah go-live | Sebelum go-live (Tahap 12) | RT-29 | User, CEO |
| G-21 | CR-011, CR-015 diputuskan | Checkpoint 2 | Tidak masuk rencana | User |

---

## 10. Jalur kritis proyek

1. Fase 1 fondasi dan reference implementation (jalur kritis 65 jam-task, §7.4).
2. Gerbang re-plan 2026-12-11 (keputusan opsi).
3. Fase 2 pembayaran, bergantung pada PT-36 (target 2026-12-21) dan EN-24 pajak.
4. Fase 3 integrasi, bergantung pada PT-05 (rekanan nyata untuk SIT) dan, untuk RME, PT-32.
5. Pembekuan fitur 2027-01-29 → SIT → audit keamanan eksternal (PT-49) → UAT (PT-50) → pengajuan toko aplikasi paling lambat 2027-02-12 (PT-51, R-08) → deploy → go-live.

Butir eksternal di jalur kritis yang lamanya `[TIDAK DIKETAHUI]`: konfirmasi DOKU, kesiapan rekanan, registrasi SATUSEHAT, audit eksternal, review toko aplikasi, dan perizinan (PT-20, PT-25).

---

## 11. Butir yang jatuh tempo di Tahap 7

### 11.1 CR-011: onboarding mandiri dan langganan berbayar RME (PT-34)

| Bagian | Isi | Ukuran (estimasi) |
| --- | --- | --- |
| Pendaftaran mandiri calon rekanan | Formulir, unggah dokumen izin, persetujuan perjanjian RME dan pemrosesan data; verifikasi tetap oleh Verifikator (BR-006, US-205, UP-49) | M + L (±33 jam-task) |
| Provisioning tenant RME otomatis | Menggantikan langkah Operator RME di US-1301 | M (±12) |
| Penagihan langganan | Paket, tagihan, pembayaran ke PT. Arunika lewat adaptor terpisah dari pembagian dana pasien (arsitektur §13), penangguhan saat telat bayar | XL + L (±57) |
| Penyesuaian backoffice | Antrean pendaftaran mandiri | S (±6) |
| **Total** | Belum termasuk revisi BRD/PRD dan review kepatuhan | **±108 (rentang ±90–150)** |

Fakta bisnis yang belum ada dan memblokir bagian penagihan: apakah RME berbayar, model/harga/siklus, cara bayar, perlakuan telat bayar `[TIDAK DIKETAHUI]` (PT-34) `[DOK: backlog.md]` CR-011.

Opsi `[UP-68]`: (a) rilis 1 penuh, menambah ±108 jam-task ke Fase 3 yang sudah membutuhkan kecepatan tertinggi; (b) rilis 1 sebagian, hanya pendaftaran mandiri tanpa penagihan (±45 jam-task), dengan verifikasi tetap manual; (c) rilis berikutnya; titik perluasan sudah disiapkan arsitektur §13. **Usulan penyusun: (c)**, karena hasil §6 dan karena PT-34 belum terjawab.

### 11.2 CR-015: vaksinasi dan suntik/infus vitamin sebagai layanan lab

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

### 11.6 PT-42, PT-43, PT-44 (usulan untuk Checkpoint 2)

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
| Keputusan | KEPUTUSAN-178 | Metode estimasi effort, hari kerja, porsi mobile | menunggu (Checkpoint 2) |
| Keputusan | KEPUTUSAN-179 | Penomoran EN-/UI-/F1-T; story boleh selesai bertahap per AC | menunggu |
| Keputusan | KEPUTUSAN-180 | MVP Fase 1 = slice "pasien terverifikasi" | menunggu |
| Keputusan | KEPUTUSAN-181 | Reference implementation = US-106 → US-201 | menunggu |
| Keputusan | KEPUTUSAN-182 | Urutan teknis Fase 1 dan jalur kritis | menunggu |
| Keputusan | KEPUTUSAN-183 | Spike jalur email Keycloak–SES di awal Fase 1, hasilnya usulan ADR ke user | menunggu |
| Keputusan | KEPUTUSAN-184 | Spike penyimpanan berkas lokal/uji sebelum EN-10 | menunggu |
| Keputusan | KEPUTUSAN-185 | Penjadwalan CR-013 (EN-24) dan CR-014 (F1-T17, EN-22) dengan review DBA | menunggu |
| Keputusan | KEPUTUSAN-186 | Perilaku simpan draf ke server (PT-42) | menunggu |
| Asumsi | ASUMSI-01 | Satu task 2–4 jam mencakup `verify.sh` dan satu putaran perbaikan review | menunggu |
| Asumsi | ASUMSI-02 | Test penguncian Keycloak dapat diotomasi dengan testcontainers di `verify.sh` | menunggu |
| Sumber | SRC-261–SRC-264 | OWASP Authentication Cheat Sheet; Keycloak brute force (pesan galat); W3C Understanding SC 2.2.1; Keycloak email lewat SMTP | tercatat |
| Usulan | UP-65–UP-72 | Kalender fase; prinsip urutan dan gerbang re-plan; opsi scope; CR-011; CR-015; PT-44; PT-43; PT-42 | Checkpoint 2 |
| Pertanyaan | PT-47–PT-51 | Status integrasi klinik untuk konsultasi; kapasitas tim; auditor eksternal; peserta UAT; akun developer toko | Checkpoint 2 |

Rincian UP dan PT ada di `docs/07-catatan-phase-plan.md`.

---

## 13. Gate Tahap 7

| Syarat | Status |
| --- | --- |
| Semua story Fase 1 memenuhi Definition of Ready | Terpenuhi dengan dua dependensi keputusan user (ADR jalur email, PT-44) (§7.5) |
| Enabler fondasi ada di Fase 1 | Terpenuhi: skema inti (EN-04), autentikasi + RBAC (EN-06, EN-07), Audit Trail Foundation (EN-05), CI pipeline (EN-02), reference implementation (EN-14) |
| Test plan tersusun dari PRD | Dikerjakan subagent `qa-engineer` (`docs/test-plan.md`) setelah dokumen ini |
| User menyetujui pembagian fase | Menunggu Checkpoint 2 (UP-65–UP-67) |
| Gate anti-halu | Menunggu fact-checker |
