# Fact-check Tahap 01: Business Case

Tanggal: 2026-11-17
Dokumen diperiksa: `docs/01-business-case.md` (bukti sumber: `docs/01-catatan-business-case.md`, `docs/00-project-brief.md`, `docs/00-catatan-intake.md`; register: `docs/sources.md`, `docs/decision-log.md`; referensi tahap: `.claude/skills/runbook/SKILL.md`)
Tingkat ketelitian: T2 Standar
Hasil: LULUS (tidak ada temuan blokir; 5 catatan non-blokir)

## Ringkasan
- Total klaim: sekitar 110 (perkiraan, satu kalimat bisa memuat beberapa klaim)
- Terverifikasi: sekitar 106
- Tanpa sumber: 2 (non-blokir: butir 1 dan butir 5 di tabel)
- Sumber tidak mendukung: 0
- Kontradiksi: 0 yang mengubah keputusan (1 sebagian, non-blokir, butir 1)
- Fakta bisnis berlabel asumsi/keputusan: 0

## Hal yang diperiksa dan bersih
- Kode rujukan `T1-Q1` sampai `T1-Q8` (termasuk Q5a, Q7a) ada di catatan Tahap 1 dan isinya cocok. Kode `P..`/`PS..` yang dirujuk (P1.1 sampai P8.3, PS1 sampai PS6e) ada di `00-catatan-intake.md` dan isinya cocok dengan klaim.
- Tabel simulasi §5.2 jelas ditandai "simulasi, bukan keputusan dan bukan proyeksi", atas permintaan user (T1-Q2), parameter ilustratif berlabel `[KEPUTUSAN-01]`, volume transaksi `[TIDAK DIKETAHUI]` (PT-10). Parameter itu bukan fakta bisnis. KEPUTUSAN-01 dan KEPUTUSAN-02 terdaftar di `decision-log.md`. Tidak ada `[ASUMSI]` yang dipakai.
- Penilaian risiko §7.2 ditandai sebagai penilaian penyusun (`[KEPUTUSAN-02]`), dasarnya fakta berkode, dikonfirmasi user di gate. Brief (bagian "Risiko yang dicatat untuk dinilai di Tahap 1") memang menyerahkan penilaian ini ke Tahap 1. R-03 memakai `[TIDAK DIKETAHUI]` untuk kemungkinan, bukan tebakan.
- Estimasi biaya §6.1: hanya angka developer (T1-Q5a) dan US$25 (SRC-01) yang bernilai; komponen lain `[TIDAK DIKETAHUI]` dengan penjelasan. Total diberi catatan "bukan total biaya proyek". Apple Developer ditulis `[TIDAK DIKETAHUI]` dan kegagalan buka halaman tercatat di `sources.md`, sesuai aturan.
- Aritmetika (dihitung ulang):
    - 2026-11-17 sampai 2027-03-01 = 104 hari (13 sisa November + 31 Desember + 31 Januari + 28 Februari + 1). 104 hari = sekitar 3,4 bulan. Benar.
    - Rp10–20 juta × 3,4 = Rp34–68 juta; × 12 = Rp120–240 juta. Benar.
    - Tabel A (5/10/15/20% × Rp100 juta = Rp5/10/15/20 juta): benar.
    - Tabel B (biaya ÷ komisi): 10 juta → 200 / 100 / 66,7 / 50 juta; 20 juta → 400 / 200 / 133,3 / 100 juta. Benar.
    - Tabel C: 3 × (0,25 / 0,5 / 1 juta) = 0,75 / 1,5 / 3 juta; 30 × = 7,5 / 15 / 30 juta. Benar. Angka 3 dan 30 rekanan sesuai P8.2 dan P8.1.
- SRC-01: halaman https://support.google.com/googleplay/android-developer/answer/6112435?hl=en dibuka. Isinya: "There is a US$25 one-time registration fee."; "Verification requirements depend on the account type chosen in Step 4."; "you may be asked for a valid government ID and a credit card, both under your legal name." Mendukung klaim di §6.1 dan R-08. Entri di `sources.md` lengkap (URL, tanggal akses, kutipan).
- Konsistensi dengan Project Brief: angka KPI, syarat go-live, tenggat, tim, Flutter, urutan toleransi (tanggal tetap, scope dikurangi, kecuali izin/audit/UAT), pembayaran campuran, dan prinsip gateway konsisten. Pembaruan atas PT-01, PT-09, PT-11, PT-14 sesuai catatan T1. Peran "Executive = user" sesuai `SKILL.md` (baris Tahap 1 dan 11). Pemetaan tahap di §6.2 (Tahap 1–7, 8, 9–11, 12–13) sesuai `SKILL.md`.
- Keputusan bisnis ke user: toleransi biaya (Q8), alasan go-live (Q4), pemilik (Q6), opsi (Q7) semuanya ditanyakan. Respons risiko, pemilik/tenggat PT baru, dan keputusan kelayakan ditulis sebagai usulan untuk Executive.

## Temuan
Tidak ada temuan blokir.

| # | Lokasi | Klaim | Masalah | Usulan | Status |
| --- | --- | --- | --- | --- | --- |
| 1 (non-blokir) | §5.1, paragraf sesudah tabel | "Pendapatan dari klinik dan lab hanya mungkin lewat biaya dari rekanan yang ditagihkan terpisah" | Kesimpulan penyusun tanpa label. "Hanya mungkin" terlalu kuat: klinik rekanan juga outlet penjual obat bebas dibayar di aplikasi (PS2, PS4), dan sebagian dokter berasal dari klinik rekanan (PS6e, konsultasi dibayar di aplikasi per P3.3). Tidak mengubah angka atau keputusan karena sumber komisi sudah terdaftar di tabel | Ubah menjadi "untuk layanan pendaftaran klinik dan lab, pembayaran langsung ke mitra (P3.3), jadi pendapatan dari layanan itu hanya lewat biaya rekanan" dan beri `[DOK] P3.3` | Terbuka, non-blokir |
| 2 (non-blokir) | §1 butir "Biaya yang sudah diketahui" | "hanya biaya mobile developer" | Tidak konsisten dengan §6.1 yang juga memuat US$25 `[SRC-01]` | Tulis "biaya developer dan biaya akun Google Play US$25" | Terbuka, non-blokir |
| 3 (non-blokir) | §6.1, baris Cloud dan baris Audit keamanan eksternal | Cloud: "Penyedia belum dipilih `[DOK] P4.2, P5.5`". Audit: "auditor eksternal `[DOK] P5.3`" | Label kurang presisi. P4.2 tidak menyebut cloud; dasar yang tepat adalah brief bagian Batasan (P5.5, Konfirmasi P7: cloud dipilih di Tahap 5). P5.3 menyebut auditor eksternal untuk audit atau bukti kepatuhan, bukan secara eksplisit audit keamanan. Nilainya tetap `[TIDAK DIKETAHUI]`, jadi tidak ada angka yang salah | Rujuk "brief Batasan, Konfirmasi P7" untuk cloud; tulis "pihak pelaksana audit keamanan belum ditentukan (P5.3 menyebut auditor eksternal)" | Terbuka, non-blokir |
| 4 (non-blokir) | §3 "Alasan memilih opsi C" dan §4.1 | Alasan 1 ("prinsip gateway ditentukan sendiri"), alasan 3 ("sewa atau lisensi dinilai lebih mahal"), tenggat "1 Maret 2028" | Elaborasi penyusun atas nama opsi yang dipilih user (T1-Q7a hanya mencatat nama opsi). Alasan 3 sudah diberi catatan penyusun; alasan 1 dan 2 belum. Tenggat KPI tepat 1 Maret 2028 adalah hitungan dari "12 bulan" (P8.1a: "sekitar Maret 2028"), dasarnya sudah ditulis | Konfirmasi redaksi alasan dan tenggat tanggal tepat di gate | Terbuka, non-blokir |
| 5 (non-blokir) | Kalimat pembuka (rujukan PRINCE2) dan §5.2 | "prinsip PRINCE2 *continued business justification*"; parameter simulasi 5/10/15/20% dan Rp250 ribu/500 ribu/1 juta; Tabel C mengandaikan langganan bulanan | Rujukan PRINCE2 tanpa label sumber (praktik terbaik tingkat 3, tidak menentukan angka). Nilai parameter spesifik dipilih penyusun dan belum ditunjukkan ke user di wawancara (Q2 hanya menyebut "beberapa persentase"); KEPUTUSAN-01 berstatus "menunggu". Bukan fakta bisnis dan sudah dinyatakan ilustratif | Pastikan user mengonfirmasi KEPUTUSAN-01 dan KEPUTUSAN-02 di gate; opsional tambah `[DOK: SKILL.md]` untuk rujukan PRINCE2 | Terbuka, non-blokir |

## Tindak lanjut orkestrator (2026-11-17)

- Non-blokir 1 (§5.1) dan 2 (§1) diperbaiki: kalimat pendapatan klinik/lab dirumuskan ulang dengan label `[USER] T1-Q1` dan `[DOK] PS2a, PS4`; ringkasan biaya kini menyebut US$25 `[SRC-01]`. Tidak memicu putaran ulang (aturan-kerja §2).
- Non-blokir 4 dan 5 (tenggat KPI 1 Maret 2028, parameter simulasi, KEPUTUSAN-01/02) dibawa ke rekap gate untuk dikonfirmasi user.
