# Fact-check Tahap 7 (terarah): Klarifikasi AC Fase 1

Tanggal: 2026-11-18
Dokumen diperiksa: `docs/07-klarifikasi-ac-fase-1.md` (baru); tambahan di `docs/07-catatan-phase-plan.md` (T7-F8/F9, UP-73–UP-77, Checkpoint 2 no. 5, 12–14), `docs/backlog.md` (catatan DoR), `docs/decision-log.md` (KEPUTUSAN-187–197), `docs/sources.md` (SRC-265, tambahan SRC-262), `docs/traceability.md` (paragraf klarifikasi)
Hasil: LULUS (nol blokir)

## Ringkasan
- Cakupan: pemeriksaan terarah pada baris yang berubah; sampel klaim penentu, bukan hitung penuh
- Terverifikasi: semua klaim yang diperiksa kecuali butir non-blokir di bawah
- Tanpa sumber (blokir): 0
- Sumber tidak mendukung (blokir): 0
- Kontradiksi (blokir): 0
- Fakta bisnis berlabel asumsi: 0
- Keputusan bisnis tidak dibawa ke user: 0

Hasil per fokus:
1. Butir Teknis: NIK 16 digit sesuai data dictionary A2.4; kompensasi sesuai ADR-011 butir 6; penolakan seketika sesuai ADR-011 butir 8; autentikasi ulang sesuai ADR-011 butir 9; registri NIK sesuai A2.8; pasien tanpa MFA sesuai ADR-011 butir 5. Tidak ada aturan bisnis yang disamarkan sebagai teknis (lihat non-blokir 3).
2. Butir Bisnis dan Campuran (T-03, T-11, T-14, T-17, T-18, T-19; T-06, T-12, T-13, T-15, T-16) dibawa sebagai UP-73–UP-77 dengan opsi dan usulan default. Semuanya ada di Checkpoint 2 no. 5 dan 12. Jumlah 11 Teknis + 6 Bisnis + 5 Campuran = 22 (T-01–T-22), konsisten di klarifikasi §1.2 dan T7-F8.
3. `docs/03-prd.md` tidak berisi rumusan baru (tidak ada UP-73–77, AC1b, AC2b, `nik_unavailable`, 5.242.880) dan tidak ada di daftar berkas yang berubah. Usulan hanya ada di dokumen klarifikasi.
4. SRC-265: URL (raw v5.0.0 bab V6) dibuka. Didukung teks resmi: 6.2.1 (L1, minimum 8, 15 "strongly recommended"), 6.2.4, 6.2.5, 6.2.6, 6.2.7, 6.2.8 (L1), 6.2.9, 6.2.10, 6.2.11, 6.2.12 (L2), 6.1.1, 6.3.1 (L1), 6.3.3 (L2), 6.3.5 dan 6.3.8 (L3), 6.3.7 (L3), 6.4.3 (L2), serta kalimat "From L2 onwards, multi-factor authentication mechanisms are required". Klaim S-01 sesuai, dan ADR-011 butir 1 serta butir 5 memang menyatakan L2 dengan pasien tanpa MFA. SRC-262 dibuka ulang: pesan "Invalid username or password" saat terkunci, Failure Reset Time 12 jam dihitung dari kegagalan terakhir, hitungan tidak bertambah saat terkunci, Quick Login Check 1000 ms, Minimum Quick Login Wait 1 menit, Max Login Failures bawaan 30, tidak menyebut perpanjangan penguncian maupun email. Semua cocok, dan kelalaian itu sudah dinyatakan sebagai `[TIDAK DIKETAHUI]` (diuji di F1-T55).
5. Rujukan terdaftar: KEPUTUSAN-187–197, ASUMSI-02, KEPUTUSAN-24/46/84/85/183/186, SRC-15/63/70/233/261/262/265, UP-73–77, T7-F8/F9, F1-Txx, kode respons 4010001/4130001/4150001/4220001/4290001, T-01–T-22 sesuai `docs/test-plan.md` §9, backlog dan traceability menunjuk dokumen yang benar.

## Temuan
| # | Lokasi | Klaim | Masalah | Usulan |
| --- | --- | --- | --- | --- |
| (blokir) | - | - | Tidak ada | - |
| 1 | Klarifikasi T-13 AC4b/AC4d | Email lama tetap dipakai sampai tautan dibuka; email lama menerima pemberitahuan perubahan | Perilaku baru tanpa KEPUTUSAN atau sumber; pemberitahuan setelah ubah email adalah ASVS 6.3.7 (Level 3, `[SRC-265]`), bukan kewajiban L2 | Catat di KEPUTUSAN (mis. di 187/195) atau jadikan opsi |
| 2 | Klarifikasi T-21; F1-T69 | PRD US-1501 AC3 mewajibkan "peringatan keamanan"; usulan menunda peringatan email ke Fase 2 | Penundaan bertumpu pada KEPUTUSAN-179 yang belum disetujui. F1-T69 di phase plan masih memuat "peringatan keamanan" di Fase 1 | Sebut ketergantungan ini di pertanyaan no. 13; sesuaikan F1-T69 setelah disetujui |
| 3 | KEPUTUSAN-192 vs UP-73 | Batas NIK (3 kali per 24 jam, tahan 24 jam) ada di KEPUTUSAN teknis (no. 13) dan di UP-73 (no. 5) | Bila no. 13 disetujui sekaligus, bagian bisnis NIK bisa tersetujui tanpa lewat no. 5 | Tandai di no. 13 bahwa butir NIK mengikuti keputusan UP-73 |
| 4 | Klarifikasi T-06 sumber | "diuji di F1-T55 (ASUMSI-02)" | ASUMSI-02 hanya soal testabilitas penguncian, bukan soal apakah percobaan saat terkunci memperpanjang penguncian | Tulis sebagai `[TIDAK DIKETAHUI]` dengan rujukan F1-T55 saja |
| 5 | KEPUTUSAN-194 / T-12 | Nilai 120 tahun, 8–13 digit nomor HP, 5–500 karakter alamat | Tanpa sumber eksternal (diakui di KEPUTUSAN-194, sudah menunggu Checkpoint 2) | Tidak perlu tindakan; tetap pada paket no. 13 |
