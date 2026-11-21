# Checkpoint 2: Paket perencanaan

Tanggal: 2026-11-18
Mode: otomatis (`docs/STATUS.md`)
Tahap yang selesai: Tahap 6 (Design Guideline) dan Tahap 7 (Phase Plan + Backlog). Tahap 1–5 sudah disetujui lewat gate terpandu, jadi checkpoint ini hanya meminta keputusan atas hasil Tahap 6–7 dan perubahan yang terjadi di antaranya (CR-015, CR-016).

## Dokumen untuk direview

- `docs/06-design-guideline.md`: design token, komponen dan state, pola kosong/error/loading, aksesibilitas WCAG 2.2 AA, 39 alur utama
- `docs/wireframes/index.html`: 21 wireframe HTML statis (buka di browser)
- `docs/07-phase-plan.md`: pembagian fase, kalender usulan, kelayakan jadwal, 71 task Fase 1, jalur kritis
- `docs/backlog.md`: story berfase, enabler EN-01–EN-27, story UI UI-01–UI-05, CR-015 dan CR-016
- `docs/03-prd.md` v1.2 draf: revisi CR-013 (pajak di atas harga rekanan + markup)
- `docs/test-plan.md`: test plan dan 674 test case dari PRD (temuan AC di §9)
- `docs/07-klarifikasi-ac-fase-1.md`: usulan klarifikasi acceptance criteria story Fase 1 (temuan QA T-01–T-22); teks AC PRD diubah hanya setelah disetujui di sini
- `docs/04-risk-register.md` v1.2 draf: RT-32–RT-34
- Catatan tahap: `docs/06-catatan-design.md`, `docs/07-catatan-phase-plan.md`

## Perubahan di antara tahap (sudah diputuskan user, untuk diketahui)

| ID | Isi | Sumber |
| --- | --- | --- |
| CR-015 | Vaksinasi dan suntik/infus vitamin masuk scope lab, di lokasi lab atau home visit sesuai regulasi. Penempatan rilis ditanyakan di bawah (UP-69) | `[USER]` 2026-11-18 |
| CR-016 | Nama aplikasi menjadi "Ruang Sehat" (menggantikan T5-Q12); ditulis dengan spasi, wordmark pamflet "RuangSehat" tidak dipakai (PT-45); merek dan domain belum dicek (PT-46) | `[USER]` 2026-11-18 |

## Asumsi yang perlu dikonfirmasi

| ID | Asumsi | Dampak kalau salah |
| --- | --- | --- |
| ASUMSI-01 | Satu task 2–4 jam-task sudah mencakup `scripts/verify.sh`, code review (plus review keamanan/DBA untuk T1), dan satu putaran perbaikan | Estimasi effort terlalu rendah; jadwal makin tidak layak |
| ASUMSI-02 | Penguncian akun Keycloak (5 gagal, 15 menit) dapat diuji otomatis dengan Keycloak di testcontainers di dalam `scripts/verify.sh` | Test negatif autentikasi harus manual; bukti T1 lebih lemah |

## Keputusan teknis yang diambil (untuk diketahui, disetujui sekaligus)

| ID | Ringkas | Bisa dibatalkan? |
| --- | --- | --- |
| KEPUTUSAN-160–177 | Desain: target WCAG 2.2 AA, aturan kontras (semua teks ≥ 4,5:1), target sentuh 48 dp/24 px, token turunan (#065F59 untuk teks di atas mint), font sistem, skala tipografi/spacing/radius, Material Icons bawaan, pola memuat/kosong/galat, format Rupiah/tanggal/zona waktu, kode referensi galat, responsivitas, uji aksesibilitas otomatis, satu kotak per persetujuan | Ya |
| KEPUTUSAN-178–186 | Perencanaan: metode estimasi, struktur backlog, MVP Fase 1 "pasien terverifikasi", reference implementation US-106 → US-201, urutan teknis Fase 1, spike jalur email Keycloak (F1-T02) dan penyimpanan berkas (F1-T03), penjadwalan CR-013/CR-014, draf catatan klinis di server | Ya |
| KEPUTUSAN-187–201 | Klarifikasi AC Fase 1: tautan email, kebijakan password (ASVS 5.0.0 V6), penguncian, satuan berkas, batas laju, penolakan sesi, validasi profil, autentikasi ulang, penghentian semua sesi setelah atur ulang password/ubah email, atur ulang tidak melewati MFA, pengerasan jalur pendaftaran. **Bagian batas NIK di KEPUTUSAN-192 ikut keputusan UP-73 (pertanyaan 4)** | Ya |
| RT-32–RT-34 | Risiko baru: jalur email Keycloak tanpa SMTP, kecepatan per fase melampaui kapasitas, jendela audit dan toko aplikasi | — |

## Temuan penting

1. **Scope penuh kemungkinan besar tidak muat sampai 1 Maret 2027.** Estimasi total ±1.857 jam-task (plus 42 yang terblokir PT-24). Kebutuhan ±16,3 jam-task per hari kerja di Fase 1 naik ke ±61,2 bila Fase 3 dan 4 paralel. Throughput tim belum tercatat (PT-48), sehingga kelayakan tidak dapat dinyatakan; indikasinya kuat tidak layak (`docs/07-phase-plan.md` §6).
2. **Jalur email Keycloak belum punya rancangan** (RT-32): Keycloak mengirim email lewat SMTP `[SRC-264]`, sedangkan Amazon SES Jakarta hanya menyediakan API `[SRC-84]`. Spike di awal Fase 1, hasilnya usulan ADR baru untuk Anda.
3. **Jalur kritis Fase 1: 65 jam-task berurutan** (dihitung ulang orkestrator dari tabel task), termasuk layar mobile. Satu-satunya keputusan Anda di jalur kritis: persetujuan pembuatan `scripts/verify.sh`.
4. **MFA pasien bertentangan dengan target ASVS Level 2.** ASVS 5.0.0 menyatakan "From L2 onwards, multi-factor authentication mechanisms are required" `[SRC-265]`, sedangkan ADR-011 menargetkan L2 tetapi pasien tidak wajib MFA. Ini perubahan ADR yang sudah diterima, sehingga diputuskan Anda (pertanyaan 3).
5. **Celah acceptance criteria Fase 1.** QA menemukan 22 AC Fase 1 yang belum dapat diuji atau saling bertentangan (mis. kebijakan password tanpa angka, pesan yang membuka keberadaan akun/NIK, perubahan NIK setelah disetujui, Super Admin pertama). Story Fase 1 memenuhi Definition of Ready setelah klarifikasi disetujui.
6. Kontras: teks #087F78 di atas mint #DFF4ED hanya 4,24:1 (di bawah 4,5:1); diusulkan #065F59 (6,56:1) untuk teks di atas mint.

## Pertanyaan terbuka

| # | Pertanyaan | Memblokir |
| --- | --- | --- |
| PT-48 | Kapasitas tim (jam mobile developer, porsi Claude, sesi per hari, tambahan tenaga) | Kelayakan jadwal; ditanyakan di bawah |
| PT-47 | Klinik yang hanya menaungi konsultasi online wajib integrasi "Aktif"? | Fase 2 (2026-12-14) |
| PT-40, PT-41 | Pembuat logo/ikon final; nilai warna mint pamflet | Aset toko aplikasi; token warna |
| PT-46 | Cek merek DJKI dan domain "Ruang Sehat" (belum dicek) | Sebelum Tahap 12; disarankan dimulai segera |
| PT-49, PT-50, PT-51 | Auditor keamanan eksternal; konsultan medis UAT; akun Apple/Google Play | Sebelum Fase 3 / 2027-01-08 |
| PT-28, PT-36, PT-39, PT-05, PT-38, PT-29/30/33/37, PT-35 | Gerbang eksternal (phase plan §9) | Sesuai tenggat masing-masing |

## Hasil review dan fact-check

| Tahap | Reviewer | Fact-check |
| --- | --- | --- |
| 6 | QA (cakupan wireframe): LULUS DENGAN CATATAN, 0 blokir (`docs/qa/tahap-06-review-wireframe.md`) | LULUS, 0 blokir (`docs/fact-check/tahap-06.md`) |
| 7 | QA: test plan 674 test case, 35 temuan AC (`docs/test-plan.md` §9). Security architect atas klarifikasi AC: putaran 1 TIDAK LULUS (1 blokir), putaran 2 LULUS DENGAN CATATAN (`docs/review/tahap-07-security.md`) | Rencana fase LULUS, 0 blokir (`docs/fact-check/tahap-07.md`; 5 catatan angka/label diperbaiki). Klarifikasi AC LULUS, 0 blokir (`docs/fact-check/tahap-07-klarifikasi.md`) |

## Yang dibutuhkan dari user (diajukan satu per giliran)

Butir tanpa usulan default dan butir yang mengubah keputusan lama ditanyakan terpisah; butir yang punya usulan default ditawarkan disetujui sekaligus di akhir (Anda boleh menyebut butir yang ingin diubah).

1. **Kapasitas tim dan opsi scope** (PT-48, UP-67; usulan: Opsi B sebagai baseline, final di gerbang re-plan 2026-12-11; tambahan tenaga dipertimbangkan sekarang).
2. **Kalender fase dan gerbang re-plan** (UP-65, UP-66).
3. **MFA pasien, perubahan ADR-011** (T7-F9, opsi M1–M4, tanpa default) dan celah sesi L2 (UP-78).
4. **Enumerasi akun dan NIK** (UP-73; default menegaskan ADR-011 butir 4, opsi A merevisinya), termasuk batas NIK berlapis dan email pemberitahuan penguncian.
5. **CR-011 onboarding mandiri dan langganan RME** (UP-68; usulan: rilis berikutnya).
6. **CR-015 vaksinasi/infus** (UP-69; usulan: rilis berikutnya).
7. **Admin Ruang Sehat** (UP-76): siapa Super Admin pertama dan siapa orang kedua penyetujunya (tanpa default), bootstrap dengan persetujuan orang kedua, minimal dua Super Admin, Auditor eksklusif, risiko WEB-07/M-16.
8. **Arah visual** (UP-58) dan nilai mint (PT-41).
9. **Font** (UP-59; usulan: font sistem).
10. **Logo dan ikon** (UP-60, PT-40) dan slogan pamflet (sisa PT-45).
11. **Navigasi dan ubin beranda** (UP-62, UP-63).
12. **PT-47** status integrasi klinik untuk konsultasi online.
13. **Tenggat gerbang eksternal** (phase plan §9): dapat dipenuhi?
14. **Klarifikasi AC lain dengan default**: siklus status identitas (UP-74), data rujukan lewat migrasi (UP-75), usia minimum sebagai parameter (UP-77), CR kecil ubah password saat login.
15. **Disetujui sekaligus**: UP-61 tanpa mode gelap; UP-64 pengingat jadwal; UP-71 PB-07 dan sesi 12 jam sebagai pengecualian essential WCAG 2.2.1; UP-72 draf klinis di server; teks PRD v1.2 (CR-013); istilah "pengembalian dana" (KEPUTUSAN-171); KEPUTUSAN-160–201 (kecuali bagian NIK KEPUTUSAN-192); ASUMSI-01–02; RT-32–RT-34.

## Jawaban user

_Diisi setelah user menjawab (sumber `[USER]`)._

### Butir 1: kapasitas tim dan opsi scope (2026-11-18)

| Q | Pertanyaan | Jawaban `[USER]` | Dampak ke dokumen |
| --- | --- | --- | --- |
| C2-Q1 | Opsi scope baseline (UP-67) | **Opsi B** sebagai baseline; final di gerbang re-plan 2026-12-11 | UP-67 disetujui; Fase 4 tidak dikerjakan kecuali gerbang re-plan memilih A |
| C2-Q2 | Tambah tenaga (Opsi E) | **Ya, developer backend/web** | Opsi E dijalankan bersama B; waktu rekrut, jumlah orang, dan biaya `[TIDAK DIKETAHUI]` (PT baru) |
| C2-Q3 | Mobile developer (PT-48) | **Penuh waktu (±40 jam/minggu), seluruh layar mobile dikerjakannya** | PT-48 sebagian terjawab |
| C2-Q4 | Sesi Claude per hari kerja (PT-48) | **≥ 6 jam per hari kerja** | PT-48 sebagian terjawab; throughput tetap diukur di Fase 1 |


### Butir 2: kalender fase dan gerbang re-plan (2026-11-18)

| Q | Pertanyaan | Jawaban `[USER]` | Dampak ke dokumen |
| --- | --- | --- | --- |
| C2-Q5 | Kalender fase (UP-65) | **Setuju** apa adanya (tanpa Fase 4, Opsi B) | UP-65 disetujui |
| C2-Q6 | Gerbang re-plan 2026-12-11 (UP-66) | **Setuju** dengan kriteria phase plan §6.5 | UP-66 disetujui |
| C2-Q7 | Libur akhir tahun | **Tim libur pada libur nasional dan cuti bersama** | Hari kerja Fase 2 (dan fase lain yang terkena) harus dikurangi; kebutuhan jam-task per hari kerja dihitung ulang. Tanggal libur dan cuti bersama 2026–2027 wajib dari sumber resmi (SKB) `[TIDAK DIKETAHUI]` sampai diverifikasi |
| C2-Q8 | Mulai developer backend/web tambahan | **Awal Januari 2027** (membantu mulai Fase 3) | Kapasitas Fase 2 tetap tim lama; jumlah orang dan biaya `[TIDAK DIKETAHUI]` (PT baru, pemilik CEO) |

### Butir 3: MFA pasien dan celah sesi Level 2 (2026-11-18)

| Q | Pertanyaan | Jawaban `[USER]` | Dampak ke dokumen |
| --- | --- | --- | --- |
| C2-Q9 | MFA pasien (S-01/T7-F9), **mengubah ADR-011 yang sudah diterima** | **M1: MFA wajib untuk pasien** (TOTP wajib + kode pemulihan) | Revisi ADR-011 butir 3 dan 5, NFR-SEC-02, US-101, US-103; AC4e berlaku untuk realm `pasien`; prosedur pemulihan perangkat hilang (6.4.4, H-05) meluas ke pasien; task Fase 1 bertambah (ukuran dinilai ulang); dampak adopsi `[TIDAK DIKETAHUI]` dicatat di risk register |
| C2-Q10 | 7.5.2 layar sesi aktif (UP-78 a) | **(a) Layar "Perangkat dan sesi aktif" lengkap di Fase 2**, semua aplikasi | UP-78 butir 7.5.2 disetujui; satu story lintas aplikasi ditambah di Fase 2 |
| C2-Q11 | 7.1.2 sesi paralel admin (UP-78 b) | **Tidak dibatasi**, didokumentasikan (juga untuk pasien dan tenaga kesehatan) | Dicatat di `docs/05-asvs-checklist.md` beserta alasan |

### Butir 4–15: didelegasikan ke usulan penyusun (2026-11-18)

Setelah butir 3, user menjawab lewat Telegram (2026-11-18 10:58 UTC): "jawab sesuai dengan rekomendasimu atau lihat dokumen yg sudah ada. tidak perlu berulang" `[USER]`. Karena itu setiap butir yang punya usulan default dianggap **disetujui sesuai usulan**. Butir yang berisi fakta bisnis tanpa usulan (nama orang, pihak ketiga, nilai merek) **tidak diisi penyusun** dan tetap `[TIDAK DIKETAHUI]` sebagai pertanyaan terbuka dengan tenggat (anti-halu §1.3).

| Butir | Pertanyaan | Keputusan | Label |
| --- | --- | --- | --- |
| 4 | Enumerasi akun dan NIK (UP-73) | Opsi B untuk semua alur (pesan seragam, menegaskan ADR-011 butir 4); NIK ditolak dini dengan pesan umum dan batas berlapis 3/24 jam per akun, 3/24 jam per NIK lintas akun, 10/24 jam per IP, peringatan > 20/jam global; email pemberitahuan penguncian **tidak** dibangun di rilis 1. UP-70 tercakup | `[USER]` (delegasi ke usulan UP-73) |
| 5 | CR-011 onboarding mandiri dan langganan RME (UP-68) | Rilis berikutnya | `[USER]` (delegasi) |
| 6 | CR-015 vaksinasi/infus (UP-69) | Rilis berikutnya; ubin vaksinasi/infus tidak tampil (UP-63) | `[USER]` (delegasi) |
| 7 | Admin Ruang Sehat (UP-76) | Aturan disetujui: bootstrap satu kali dengan persetujuan orang kedua (Control Group OpenBao), minimal dua Super Admin sebelum go-live, menonaktifkan akun sendiri ditolak, Auditor eksklusif dan Auditor terakhir tidak dapat dicabut, pemberitahuan pemberian sub-peran sensitif (risiko WEB-07/M-16). **Siapa Super Admin pertama dan orang kedua penyetujunya: `[TIDAK DIKETAHUI]`** → PT-52, tenggat sebelum EN-26 production (2027-02-01) | `[USER]` (delegasi) + PT-52 |
| 8 | Arah visual (UP-58) | Disetujui sesuai usulan. Nilai mint pamflet (PT-41) tetap terbuka; token #DFF4ED dipakai sampai dijawab | `[USER]` (delegasi); PT-41 tetap |
| 9 | Font (UP-59) | Font sistem rilis 1 | `[USER]` (delegasi) |
| 10 | Logo dan ikon (UP-60), slogan | Arah disetujui (desainer, simbol rumah-daun, "Ruang Sehat" dengan spasi). Pembuat logo (PT-40) dan slogan (sisa PT-45) tetap terbuka; wireframe memakai placeholder | `[USER]` (delegasi); PT-40, PT-45 tetap |
| 11 | Navigasi dan ubin beranda (UP-62, UP-63) | Disetujui. Catatan penyusun: dengan baseline Opsi B, ubin Home Visit dan Belanja Produk mengikuti keputusan scope final di gerbang re-plan 2026-12-11 | `[USER]` (delegasi) |
| 12 | PT-47 status integrasi klinik untuk konsultasi online | Fakta bisnis tanpa usulan: tetap `[TIDAK DIKETAHUI]`, tenggat sebelum Fase 2 (2026-12-14) | PT-47 tetap |
| 13 | Tenggat gerbang eksternal (phase plan §9) | Tenggat dipakai sebagaimana tertulis; pemenuhannya dipantau di laporan fase dan gerbang re-plan | `[USER]` (delegasi) |
| 14 | UP-74, UP-75, UP-77, CR kecil ubah password saat login | UP-74, UP-75 (migrasi data, tanpa layar admin), UP-77 (parameter usia, nilai dari L-08) disetujui; CR kecil ubah password saat login (US-103 AC6 usulan, ASVS 6.2.2/6.2.3 Level 1, dirujuk KEPUTUSAN-198) **diambil** di rilis 1 dan dicatat sebagai CR-017 | `[USER]` (delegasi) |
| 15 | Persetujuan sekaligus | UP-61, UP-64, UP-71, UP-72, teks PRD v1.2 (CR-013), istilah "pengembalian dana" (KEPUTUSAN-171), KEPUTUSAN-160–201, ASUMSI-01–02, RT-32–RT-34: disetujui | `[USER]` (delegasi) |

### Pertanyaan terbuka baru dari checkpoint ini

| # | Pertanyaan | Pemilik | Tenggat |
| --- | --- | --- | --- |
| PT-52 | Siapa Super Admin pertama dan orang kedua penyetuju bootstrap (UP-76) | User/CEO | Sebelum EN-26 (2027-02-01) |
| PT-53 | Developer backend/web tambahan: jumlah orang, biaya terhadap toleransi +20%, tanggal mulai pasti (target awal Januari 2027) | User/CEO | Gerbang re-plan 2026-12-11 |
| PT-54 | Bukti identitas pengganti untuk pasien yang kehilangan perangkat authenticator, kode pemulihan, dan akses email/password sekaligus (KEPUTUSAN-204, US-207 AC5, RT-37) | User (PM), Petugas PDP | Sebelum prosedur H-05 final, sebelum go-live |
| PT-55 | Frekuensi tinjauan akses berkala admin (US-208 AC8) | User (PM) | Sebelum go-live |
| PT-57 | Cara sistem mengenali Verifikator yang juga pemilik akun pasien untuk US-201 AC8 (QA T-38; opsi A NIK staf ter-hash dicocokkan ke registri NIK [usulan], B email, C pernyataan staf, D hapus AC8). Ditanyakan ke user lewat Telegram 2026-11-18; **dijawab A** `[USER]` 2026-11-21 (KEPUTUSAN-294). Memblokir hanya TC-US-201-24–26 | User | Sebelum task US-201 di Fase 1 |
| PT-56 | Larangan Verifikator memutuskan pengajuan identitas akunnya sendiri (klarifikasi T-19 d). **Terjawab**: usulan default T-19 d termasuk delegasi butir 7, diterapkan sebagai US-201 AC8 | Orkestrator | Selesai |

### Tindak lanjut wajib sebelum Tahap 8

1. Revisi ADR-011 (MFA wajib untuk pasien, UP-78) dan dokumen terdampak (NFR-SEC-02, US-101, US-103, checklist ASVS, risk register).
2. Teks AC PRD diperbarui sesuai klarifikasi AC Fase 1 yang disetujui; PRD v1.2 menjadi final.
3. Phase plan: baseline Opsi B, kapasitas PT-48, hari kerja dikurangi libur nasional dan cuti bersama (sumber resmi), task MFA pasien di Fase 1, story layar sesi aktif di Fase 2.
4. Review keamanan atas revisi ADR-011, pembaruan test case oleh QA, lalu fact-check.

### Hasil tindak lanjut (2026-11-18)

Angka di bagian atas laporan ini adalah angka sebelum checkpoint. Angka yang berlaku sekarang:
- Phase plan v0.2: libur nasional dan cuti bersama dikurangi (SRC-280, SRC-282, SRC-283). Fase 1 15 hari kerja, Fase 2 16, Fase 3 15, Februari 9. Kebutuhan: Fase 1 18,5; Fase 2 43,0; Fase 3 Opsi B 50,3 jam-task per hari kerja. Fase 1 80 task / 277 jam-task (F1-T72–F1-T80 untuk CR-017/CR-018), jalur kritis tetap 65 jam-task. Effort Opsi B ±1.791 jam-task.
- PRD v1.2 final; ADR-011 direvisi (MFA wajib pasien); US-110 di Fase 2; CR-017, CR-018; RT-35–RT-37; KEPUTUSAN-202–205, KEPUTUSAN-215–223.
- Test plan v0.2: 805 test case.
- Catatan untuk user: bagian web RME dari US-110 diverifikasi di Fase 3 (bersama fondasi RME EN-22), bukan Fase 2 seperti jawaban C2-Q10 "semua aplikasi di Fase 2", karena web RME belum ada di Fase 2 (KEPUTUSAN-219). Aplikasi pasien, tenaga kesehatan, backoffice, dan portal tetap di Fase 2.

| Review | Hasil | Laporan |
| --- | --- | --- |
| Security architect (revisi ADR-011, AC autentikasi, threat model v1.1) | Putaran 1 TIDAK LULUS (B-01: Super Admin ikut memulai reset MFA pasien), putaran 2 LULUS DENGAN CATATAN | `docs/review/checkpoint-2-security.md` |
| QA (test plan v0.2) | 805 TC; temuan baru T-36–T-43; T-38 blokir hanya untuk TC-US-201-24–26 (PT-57) | `docs/test-plan.md` §9 |
| Fact-check | LULUS, 0 blokir, 5 non-blokir (2 diperbaiki) | `docs/fact-check/checkpoint-2.md` |
