# Fact-check Checkpoint 2: tindak lanjut (revisi ADR-011, PRD v1.2 final, phase plan v0.2, dokumen terdampak)

Tanggal: 2026-11-18
Tingkat: T1 untuk ADR-011 dan AC autentikasi/sesi (US-101, US-103, US-110, US-207 AC5, NFR-SEC-02/11, checklist ASVS V6/V7); T2 untuk sisanya
Dokumen diperiksa: `docs/checkpoint-2.md`, `docs/adr/ADR-011-target-asvs-autentikasi-sesi-mfa.md`, `docs/03-prd.md` (v1.2 final: US-101, US-103, US-110, US-201 AC8, US-207 AC5, US-208, NFR-SEC-11, §2.2), `docs/07-phase-plan.md` (v0.2), `docs/07-catatan-phase-plan.md` (status UP), `docs/07-klarifikasi-ac-fase-1.md` (v0.3, T-19, §5.1 M1), `docs/backlog.md` (CR-017, CR-018, US-110, EN-06, UI-03), `docs/traceability.md`, `docs/test-plan.md` (v0.2, §2.9), `docs/05-asvs-checklist.md` (v1.1 V6/V7), `docs/04-risk-register.md` (v1.3), `docs/04-threat-model.md` (v1.1), `docs/06-design-guideline.md` (v1.1), `docs/decision-log.md` (KEPUTUSAN-195–205, 215–223, catatan penomoran), `docs/sources.md` (SRC-265–277, SRC-280–283), `docs/review/checkpoint-2-security.md`
Hasil: **LULUS** (0 blokir, 5 non-blokir)

Catatan cara kerja: tanpa akses shell, jadi pembandingan dengan `main` dilakukan lewat bagian berlabel v0.2/v1.1/v1.2/v1.3/2026-11-18 dan riwayat revisi. US-201 AC8 (T-38, cara mendeteksi Verifikator yang juga pasien) dikecualikan sesuai instruksi.

## Ringkasan
- Total klaim substantif diperiksa: ±120 (perkiraan, dihitung per butir keputusan, angka, rujukan SRC/KEPUTUSAN, dan ID; tidak per kalimat)
- Terverifikasi: semua kecuali yang tercantum sebagai non-blokir
- Tanpa sumber: 0
- Sumber tidak mendukung: 0
- Kontradiksi (blokir): 0; kontradiksi angka/label kosmetik: 2 (non-blokir 2 dan 5)
- Fakta bisnis berlabel asumsi: 0

## Yang diverifikasi

**(1) Label `[USER]` terhadap `docs/checkpoint-2.md`.**
- C2-Q1–Q11 cocok di ADR-011, PRD, phase plan, backlog, catatan, dan decision log: Opsi B, Opsi E backend/web mulai awal Januari, mobile developer ±40 jam/minggu, sesi Claude ≥ 6 jam, libur nasional dan cuti bersama, M1, US-110 Fase 2, sesi paralel tidak dibatasi.
- C2-B4–B15 dipakai sesuai tabel delegasi.
- Butir tanpa usulan tetap `[TIDAK DIKETAHUI]`: PT-52 Super Admin pertama, PT-53, PT-47, PT-40/41/45, PT-54, PT-55.
- Peran Layanan Pelanggan dalam pemulihan MFA pasien ada di opsi M1 yang dipilih user (klarifikasi §5.1 baris 643: "beban Layanan Pelanggan bertambah dan prosedur H-05 meluas ke pasien").
- US-201 AC8 ada di usulan default UP-76 (d) (klarifikasi baris 523).

**(2) SRC baru, URL dibuka:**
- **SRC-280 (Setneg 2026)**: dibuka. Ada 24 Des (Kamis) cuti bersama dan 25 Des (Jumat) libur nasional. Tidak ada libur di November 2026. Nomor SKB 1497/2/5 Tahun 2025 dan tanggal 19 September 2025 cocok.
- **SRC-282 (Kemenko PMK 2027)**: dibuka. Cocok untuk 1 Jan (Jumat), 5 Jan (Selasa) Isra Mikraj 1448 H, 6 Feb (Sabtu), cuti bersama 5 Feb (Jumat), 8 Mar Nyepi, 10–11 Mar Idulfitri, dan cuti bersama 9/12/15 Mar. Nomor SKB 1205/3/2 Tahun 2026 dan terbit 16 September 2026 juga cocok.
- **SRC-283 (Setneg 2027)**: dibuka. Ditandatangani 15 September 2026. Memuat kalimat "pelaksanaan cuti bersama bagi lembaga/instansi swasta diatur oleh pimpinan masing-masing", dan menyebut 18 hari dengan 17 butir.
- Tidak ada libur di jendela Fase 1 (23 Nov–11 Des 2026) maupun Fase 3 (11–29 Jan 2027). Terverifikasi dari daftar lengkap kedua SKB.
- **SRC-275**: dibuka di spesifikasi OpenAPI. Kutipan "Get sessions associated with the user" dan "Remove a specific user session." beserta deskripsinya cocok. Properti `UserSessionRepresentation` juga cocok (tanpa nama perangkat).
- **SRC-276/277 (dan SRC-265/266)**: teks ASVS v5.0.0 dibuka. Kutipan 6.2.2, 6.2.3, 6.3.3 (termasuk L3 perangkat keras dan "Relaxing..."), 6.4.3, 6.4.4, 6.4.6, 6.5.1, 6.5.5, pengantar V6.3 "L2 applications must force the use of multi-factor authentication (MFA).", 7.1.2, 7.4.5, dan 7.5.2 cocok kata per kata.
- **SRC-281**: tidak dibuka. Hanya dipakai sebagai pembanding jumlah; tanggal diambil dari SRC-280.

**(3) Aritmetika phase plan: semua cocok.**
- **Hari kerja per fase.** Tanggal 2026-11-23 jatuh hari Senin.
  - Fase 1 = 15.
  - Fase 2 = 20 − 4 = 16.
  - Fase 3 = 15.
  - Februari 1–12 = 10 − 1 = 9; QA/SIT 1–5 Feb = 4; UAT 8–12 = 5; pre-deploy 8–19 = 10.
  - Mobile developer 23 Nov–5 Feb = 55 − 5 = 50.
  - Hypercare: 6 hari libur (8, 9, 10, 11, 12, 15 Maret).
- **Jam-task per hari kerja.**
  - 277/15 = 18,47; 688/16 = 43,0; 730/15 = 48,67; 930/15 = 62,0; 72/9 = 8,0.
  - Opsi B 754/15 = 50,27; Opsi C 681/15 = 45,4; Opsi D 300/15 = 20,0.
  - Nilai v0.1: 245/15 = 16,3; 622/20 = 31,1; 718/15 = 47,9; 918/15 = 61,2; 72/10 = 7,2.
  - Kenaikan Fase 2: 43,0/31,1 = +38%.
- **Total Fase 1.** F1-T72–T80 = 4+3+3+4+4+4+3+4+3 = 32, jadi 245 + 32 = 277, dari 71 + 9 = 80 task. Task T1 = 45 + 9 = 54. Porsi mobile = 37 + T77 (4) + T78 (3) = 44 dalam 12 task.
- **Effort per opsi.**
  - A: 277 + 688 + 730 + 200 + 72 = 1.967; 80 + 229 + 243 + 67 + 24 = 643 task. Selisih dari v0.1 1.857 = +32 +66 +12.
  - B = 1.967 − 176 = 1.791.
  - C = 1.791 − 73 (18+24+12+19) = 1.718.
  - D = 1.791 − 454 (63+18+24+156+96+30+55+12) = 1.337.
  - Tambahan Fase 2: 36 + 12 + 12 + 6 = 66. Enabler Fase 2 = 112, Fase 3 = 130, Februari = 72.
- **Porsi per jalur.**
  - Mobile: 44 + 195 + 120 + 90 = 449 (B: 359); sisa non-mobile 1.518 (B: 1.432).
  - Per hari kerja: 44/15 = 2,9; 233/15 = 15,5; 195/16 = 12,2; 493/16 = 30,8; 120/15 = 8,0; 634/15 = 42,3.
  - Rata-rata mobile developer: 359/50 = 7,2 dan 449/50 = 9,0.
- **Jalur kritis.** Dihitung ulang earliest finish untuk seluruh 80 task dari kolom "Jam" dan "Bergantung pada". Hasilnya tetap 65 jam-task (T70 mulai di jam-task ke-59).
  - Rantai MFA selesai di ke-48; T55 → T74 di ke-40; T76 → T79 di ke-42; registrasi T53 di ke-42; web T64 di ke-49; audit T69 di ke-48.
  - Slack 10, 11, dan 14 cocok.
  - Urutan jalur mobile: T78 selesai ke-47, T53 ke-50, T56 ke-51, T80 ke-53, T63 ke-59. Cocok.
- **Test plan.** 805 baris `| TC-`, dihitung dengan pencarian. Pecahan per fase 276 + 245 + 205 + 77 + 2, per tipe 431 + 132 + 242, dan per level 633 + 104 + 52 + 16 masing-masing berjumlah 805.

**(4) Konsistensi ID antar dokumen.**
- US-110 (BR-001, BR-108; F2, web RME F3), CR-017/CR-018, RT-35–37, PT-52–56, dan F1-T72–T80 konsisten di PRD, ADR-011, backlog, traceability, phase plan, checklist, risk register, threat model, dan decision log.
- Celah KEPUTUSAN-206–214 dan 224–225 serta SRC-278–279 tercatat sengaja tidak terpakai (decision-log baris 233). Tidak ada rujukan ke nomor itu.
- Story baru US-110 punya induk BR di traceability, jadi tidak ada temuan TANPA INDUK.
- Sisa S-1 dari review keamanan (TC-US-207-15) sudah diperbaiki: Super Admin ditolak.

**(5) Klaim ASVS di ADR-011 dan checklist sesuai teks resmi.**
- Ini mencakup batas "faktor perangkat keras hanya di L3", 6.4.4 "same level as during enrollment", dan 7.5.2 "having authenticated again with at least one factor".
- Pernyataan bahwa 7.5.2 belum terpenuhi di build Fase 1 ditulis jujur.

## Temuan

Blokir: tidak ada.

| # | Lokasi (file, bagian) | Klaim | Masalah | Usulan |
| --- | --- | --- | --- | --- |
| N1 | `docs/07-phase-plan.md` baris 7 (Dasar), 258 (§7.1 CR-018), 395–396 dan 405 (§7.5) | Merujuk "PRD v1.1 (+ v1.2 draf; revisi AC dikerjakan paralel)", "ADR-011 ... dikerjakan paralel", "design guideline v0.1", "klarifikasi AC v0.2", dan "teks AC dari revisi PRD yang dikerjakan paralel" | Rujukan sudah usang: PRD v1.2 final, ADR-011 sudah direvisi, guideline v1.1, klarifikasi v0.3. Maknanya tidak berubah | Perbarui ke versi final |
| N2 | `docs/07-phase-plan.md` §2 baris 51 | "96 story aktif (91 Must, 5 Should)" | Bertentangan dengan `traceability.md` baris 226 (97 story aktif, termasuk US-110) dan `test-plan.md` baris 118 (92 Must). Hitungan effort tidak terdampak karena US-110 sudah dihitung di Fase 2 | Ubah menjadi 97 story aktif (92 Must, 5 Should) |
| N3 | `docs/adr/ADR-011-...md` butir 12 baris 128; PRD US-110 "Fase"; KEPUTUSAN-219 | Layar sesi aktif "dibangun di Fase 2 untuk semua aplikasi", sedangkan web RME diverifikasi di Fase 3 | Jawaban user C2-Q10 berbunyi "lengkap di Fase 2, semua aplikasi". Penundaan web RME secara teknis wajar (realm `rme` baru ada di EN-22 Fase 3) dan sudah dicatat sebagai KEPUTUSAN-219 berstatus "menunggu (checkpoint berikutnya)", jadi tidak memblokir | Sebutkan penyimpangan ini secara eksplisit ke user di checkpoint berikutnya, jangan hanya lewat persetujuan sekaligus |
| N4 | `docs/05-asvs-checklist.md` baris 193; `docs/backlog.md` baris 467 | "6.3.3 kini terpenuhi untuk realm `pasien`" | Belum ada implementasi. Kolom status checklist berbunyi "Dirancang", dan ADR-011 baris 159 memakai "dapat diklaim memenuhi". Kata "terpenuhi" bisa terbaca sebagai bukti kepatuhan | Ganti dengan "dirancang memenuhi 6.3.3 (diverifikasi Tahap 8/10)" |
| N5 | Kosmetik: `07-phase-plan.md` §5 baris 103; §12 baris 585–588; PRD US-201 AC8 baris 687 | Tanggal Maret dirujuk ke `[SRC-283]`; tabel register punya kepala ganda; label "`[USER]` checkpoint-2 butir 7" | Entri SRC-283 di `sources.md` tidak mengutip tanggal Maret, padahal halamannya memuatnya (terverifikasi) dan SRC-282 sudah mengutipnya. Kepala tabel tertulis dua kali. Format label berbeda dari `[USER] C2-B7` | Lengkapi kutipan SRC-283 atau cukup rujuk SRC-282; hapus kepala tabel ganda; seragamkan label menjadi `[USER] C2-B7` |
