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
