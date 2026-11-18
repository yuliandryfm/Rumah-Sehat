# Fact-check Tahap 07: Phase Plan + Backlog

Tanggal: 2026-11-18
Dokumen diperiksa: `docs/07-phase-plan.md`, `docs/07-catatan-phase-plan.md`, perubahan Tahap 7 di `docs/backlog.md` (EN-01–EN-27, UI-01–UI-05, kolom Fase, catatan CR-011/013/014/015), `docs/03-prd.md` v1.2 draf (revisi CR-013), `docs/traceability.md` (bagian "BR → story → fase"), register `docs/decision-log.md` (KEPUTUSAN-178–186, ASUMSI-01–02), `docs/sources.md` (SRC-261–264)
Tingkat: T2, fact-check substantif; bagian T1 (pajak/harga CR-013, autentikasi, email) diperiksa teliti
Hasil: LULUS

Catatan metode: tool shell tidak tersedia, jadi `git diff main` tidak dipakai. Perubahan PRD dan backlog ditemukan lewat pencarian teks (AB-10a, CR-013, EN-/UI-). Keterbatasan: teks lama AB-11 tidak dapat dibandingkan baris demi baris.

## Ringkasan
- Total klaim: ±130 (semua klaim berlabel SRC dan semua klaim di PRD yang berlabel CR-013; sisanya klaim fakta bisnis, angka, kalender, rujukan nomor)
- Terverifikasi: ±125
- Tanpa sumber: 0 yang substantif
- Sumber tidak mendukung: 0
- Kontradiksi: 0 yang mengubah keputusan (2 inkonsistensi angka internal, non-blokir)
- Fakta bisnis berlabel asumsi: 0 (ASUMSI-01 dan ASUMSI-02 murni teknis/perencanaan)
- TANPA INDUK: 0. Ke-96 story aktif ada di traceability (cek per epic EP-01–EP-17); EN-01–EN-27 dan UI-01–UI-05 menunjuk ADR/KEPUTUSAN/BR yang terdaftar.

## Verifikasi per fokus
1. **Revisi CR-013 di PRD**: sesuai T5-Q11a ("Di atas harga rekanan+markup") dan teks CR-013 di backlog yang disetujui di gate T5-G4. Jenis, tarif, pembulatan, ongkir/biaya transaksi, pemungut, penyaluran dana, bukti bayar, pajak dalam refund, dan perilaku saat tarif belum diatur tetap `[TIDAK DIKETAHUI]` (AB-10a, AB-21, AB-27, PB-21, §3.5, US-804 AC2). Tidak ada tarif atau jenis pajak yang dikarang. Rumus: AB-11 = harga + pajak + ongkir. Biaya transaksi tetap di AB-12 dan US-801 AC1, tidak hilang.
2. **Fakta bisnis**: go-live, toleransi 0, syarat 1–7, urutan pengurangan scope, tim, developer penuh waktu, biaya di luar anggaran, toleransi +20% sesuai brief P7.3/P8.2/P8.3, BC §6–7, PRD §2.3, T5-Q7/Q9, T1-Q8. Semua tanggal fase, pembekuan fitur, jendela audit/UAT/toko, dan opsi A–E ditandai usulan (UP-65–67) untuk Checkpoint 2. Kapasitas tim `[TIDAK DIKETAHUI]` (PT-48) dan tidak diisi tebakan. Angka jam-task ditandai estimasi dengan dasar (KEPUTUSAN-178, ASUMSI-01).
3. **SRC dibuka langsung**: SRC-264 (Keycloak Admin Guide 26.8.2, kutipan SMTP) mendukung. SRC-84 (AWS SES endpoints): catatan resmi "SMTP endpoints are not currently available in ... Asia Pacific (Jakarta)", sedangkan API endpoint Jakarta ada. Mendukung. SRC-261 (OWASP: pesan generik termasuk akun terkunci; lupa password saat terkunci) mendukung. SRC-262 (Keycloak brute-force adoc: "Invalid username or password"; tidak menyebut email) mendukung. SRC-263 (WCAG 2.2.1: essential exception; batas waktu keamanan; penahanan tiket) mendukung. Temuan T7-F1 sah: `05-stack.md` dan ADR-011 tidak memuat rancangan email Keycloak (stack baris 166 hanya adaptor API SES). Pernyataan "belum punya rancangan" benar.
4. **Konsistensi gate**: versi toolchain F1-T01 (Rust 1.100, Node 26, Flutter 3.50) cocok dengan `05-stack.md`. Pustaka yang dipakai task (testcontainers, openid-client, ioredis, flutter_appauth, keycloak-config-cli, gitleaks, osv-scanner, cargo-deny/audit, jsonwebtoken) semuanya sudah ada di stack/ADR-024/coding standards. Sesi mutlak 12 jam sesuai ADR-011. Tidak ada ADR diterima yang diubah diam-diam; pola baru (jalur email) diusulkan sebagai ADR (KEPUTUSAN-183). Semua KEPUTUSAN-1…186, ASUMSI-01–02, SRC-261–264, UP-65–72, PT-47–51 terdaftar. KEPUTUSAN-178–186 berstatus "menunggu Checkpoint 2".
5. **ASUMSI-01–02**: detail teknis/perencanaan, bukan fakta bisnis, dengan dampak-jika-salah dan status "menunggu".

Aritmetika yang diperiksa dan benar: total 245 jam-task Fase 1 (71 task, 45 T1); 1.857 = 245+622+718+200+72; kebutuhan task/hari (10,4; 16,0; 20,4; 6,7); effort Opsi B 1.681, C 1.608, D 1.239 beserta rincian pengurangan; mobile ±430; hari kerja per fase dan hari dalam seminggu (23 Nov 2026 Senin, 29 Jan 2027 Jumat).

## Temuan

### Blokir
Tidak ada.

### Non-blokir (5 butir terpenting)
| # | Lokasi | Klaim | Masalah | Usulan |
| --- | --- | --- | --- | --- |
| 1 | `07-phase-plan.md` §7.4; KEPUTUSAN-182 | Jalur kritis Fase 1 = 54 jam-task (T01→T04→T05→T07→T13→T14→T15→T17→T18→T59→T60→T61→T62→T64→T70→T71) | Ada rantai lebih panjang: …T62→T66→T56→T58→T63→T70→T71 (T56 bergantung T66, T58 pada T56, T63 pada T58) = 40+2+4+4+4+3+3 = 60 jam-task. Klaim "rantai terpanjang" keliru. Tidak mengubah keputusan, tetapi mengubah angka dan kandidat jalur | Hitung ulang jalur kritis; pertimbangkan apakah T56 perlu menunggu T66 |
| 2 | `07-phase-plan.md` §6.2 | Total ±619 task; kebutuhan 4,7 task/hari Fase 1 vs 10,4 Fase 2 | Fase 1 memakai jumlah task nyata (71, rata-rata 3,45 jam-task), Fase 2–4 memakai jam÷3; 71+207+239+67+24 = 608, bukan 619. Perbandingan "task/hari" antar fase tidak satu satuan (dalam jam-task rasio Fase 2 vs Fase 1 ±1,9, bukan 2,2). Arah kesimpulan kelayakan tidak berubah | Pakai jam-task per hari sebagai ukuran tunggal, atau tulis catatan perbedaan satuan |
| 3 | `07-phase-plan.md` §1 | CR-011 ±110 jam-task | Di §11.1, backlog CR-011, dan catatan: ±108 (rentang 90–150) | Samakan ke ±108 |
| 4 | `03-prd.md` AB-10a, US-812 AC4; `backlog.md` CR-013 | Label `[USER] T5-Q11a` | Pengatur tarif = Keuangan, E-14, dan "berlaku segera atau tanggal mendatang" adalah rincian penyusun (pola data dictionary §0.8) yang disetujui lewat teks CR-013 di gate T5-G4, bukan ucapan user; catatan serupa sudah ada di fact-check Tahap 5 | Ganti label rincian itu menjadi `[DOK: backlog.md CR-013]` atau `[KEPUTUSAN-xx]` |
| 5 | `07-phase-plan.md` §11.6, §6.6; catatan T7-F4 | SRC-263 untuk sesi 12 jam; Opsi B "3 outlet tidak relevan"; AB-54 untuk "konsultasi berjalan di Ruang Sehat" | (a) Contoh keamanan di SRC-263 adalah token 2FA berbatas waktu; menyamakan sesi 12 jam sebagai essential adalah tafsiran, sudah disajikan sebagai usulan UP-71 untuk keputusan user dan reviewer keamanan (cukup). (b) Dampak Opsi B tidak menyebut syarat go-live 3 (tujuh layanan) juga tidak terpenuhi. (c) AB-54 hanya mensyaratkan klinik aktif; fakta "konsultasi berjalan di Ruang Sehat" berasal dari brief/CLAUDE.md | Lengkapi dampak Opsi B; perbaiki rujukan label |

Rekomendasi proses: tidak ada putaran perbaikan wajib. Butir 1–3 sebaiknya dirapikan sebelum Checkpoint 2 karena angka-angkanya dipresentasikan ke user.
