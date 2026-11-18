# Catatan Tahap 6 (Design Guideline)

Catatan apa adanya. File ini menjadi bukti asal setiap usulan dan pertanyaan Tahap 6, dan menjadi bahan Checkpoint 2.

Tanggal: 2026-11-18
Penyusun: Claude (persona UI/UX Designer / Design System Lead)
Mode: otomatis (tanpa wawancara user di tahap ini). Semua preferensi bisnis/merek dibawa ke Checkpoint 2 sebagai UP, tidak diputuskan penyusun.
Tingkat ketelitian: T2 Standar; bagian aksesibilitas dan layar autentikasi, persetujuan data, pembayaran diperlakukan teliti (modul T1 di `CLAUDE.md`).

## Input yang dibaca

- `docs/03-prd.md` v1.1 (semua epic, KU, AB, NFR §6.4, §6.7, §6.8, §6.11, §7.3)
- `docs/05-stack.md` §1.2–§1.3, `docs/coding-standards.md` §1.5–§1.6 (KEPUTUSAN-116 `rs_design`, `web/packages/ui`, aturan data klinis di perangkat)
- `docs/adr/ADR-011-target-asvs-autentikasi-sesi-mfa.md` (realm, MFA, pendaftaran native pasien, login browser sistem, tema login, batas sesi, autentikasi ulang)
- `docs/05-response-codes.md` (`errors[]`, `correlation_id`)
- `docs/backlog.md` CR-007, CR-009, CR-012 (tafsiran US-703 AC3), CR-013, CR-015
- `docs/05-catatan-stack.md` T5-Q12 (nama resmi "Rumah Sehat")
- `docs/input/Rumah Sehat Apps.jpeg` (dibuka sebagai gambar)

## Riset (5 halaman resmi; WCAG 2.2 dibaca dalam dua bagian; `docs/sources.md`)

| SRC | Halaman | Untuk |
| --- | --- | --- |
| SRC-256 | WCAG 2.2 (w3.org/TR/WCAG22) | Teks SC 1.4.1, 1.4.3, 1.4.4, 1.4.10, 1.4.11, 1.4.12, 2.1.1, 2.2.1, 2.4.7, 2.4.11, 2.5.8, 3.3.1, 3.3.7, 3.3.8, 4.1.3; rumus relative luminance dan rasio kontras |
| SRC-257 | Flutter docs: Accessibility | Target 48×48, kontras, skala teks, TalkBack/VoiceOver |
| SRC-258 | Flutter docs: Accessibility testing | Guideline API `flutter_test` (48×48 Android, 44×44 iOS, label, kontras) |
| SRC-259 | Repositori Plus Jakarta Sans | Lisensi OFL 1.1 untuk opsi font merek |
| SRC-260 | Flutter API kelas `Icons` | Ikon bawaan tanpa paket tambahan |

Halaman resmi Material Design dan Apple HIG tidak dibuka; ukuran target platform diambil dari dokumentasi Flutter (diizinkan instruksi tahap).

## Pengamatan atas pamflet

1. Palet tertulis: #087F78, #DFF4ED, #FAFBF7, #FFFFFF. Lingkaran contoh warna kedua tampak lebih gelap dari #DFF4ED, dan lingkaran ketiga tampak kehijauan muda, bukan putih gading. Dokumen memakai nilai tertulis (PT-41).
2. `#087F78` sebagai teks di atas mint #DFF4ED hanya 4,24:1 (di bawah 4,5:1). Karena itu diusulkan warna turunan `#065F59` untuk teks merek di atas mint (6,56:1) (KEPUTUSAN-161, KEPUTUSAN-163).
3. Wordmark dan slogan "RuangSehat" tidak dipakai (T5-Q12). Simbol rumah-daun dipakai sebagai arah; wireframe hanya memuat placeholder bertanda "draf logo" (UP-60).
4. Layanan di pamflet dipetakan ke scope PRD: "Konsultasi Dokter" dan "Kesehatan Mental" → konsultasi online (EP-03); "Cek Lab"/"Pemeriksaan Kesehatan" → pendaftaran lab dan info biaya (EP-04, EP-06); "Layanan Rumah" → home visit (EP-05); "Edukasi Kesehatan" → konten (US-1102); "Buat Janji" → pemesanan layanan terjadwal. "Vaksinasi" dan "Suntik & Infus Vitamin" hanya tampil sebagai butir katalog lab bertanda "Menunggu CR-015" (`a08-klinik-lab.html` layar 2), tanpa alur. "Gaya Hidup Sehat" tidak punya padanan fitur di PRD dan tidak dipakai.
5. Navigasi pamflet (Beranda, Jadwal, Riwayat, Profil) diganti usulan Beranda, Pesanan, Kesehatan, Profil agar produk dan hasil lab punya tempat (UP-62).

## Hasil

- `docs/06-design-guideline.md` v0.1: token (§3), kontras 13 pasangan teks + 5 non-teks (§3.2), 27 komponen dengan state (§4), pola kondisi (§5), konten dan notifikasi (§6), aksesibilitas WCAG 2.2 AA (§7), responsivitas (§8), aturan UI modul T1 (§9), 39 alur utama dengan peta wireframe (§10).
- `docs/wireframes/`: `index.html`, `wireframe.css`, 21 berkas alur (A01–A13, B01–B03, C01–C03, D01, E01). Tanpa CDN, font jarak jauh, atau skrip.
- 18 alur lain sebagai deskripsi terstruktur (§10.2); story tanpa layar sendiri di §10.3.
- Register: KEPUTUSAN-160–177 (decision-log), SRC-256–260 (sources), tabel "Story → wireframe (Tahap 6)" (traceability). ASUMSI baru: tidak ada.

## Pertentangan atau celah yang ditemukan (tidak diselesaikan sendiri)

| # | Temuan | Perlakuan |
| --- | --- | --- |
| N1 | PRD AB-11/AB-13 dan US-801 belum memuat baris pajak, sedangkan CR-013 (disetujui gate Tahap 5) menambah baris pajak terpisah; penerapan ke PRD dijadwalkan Tahap 7 | Wireframe menampilkan baris "Pajak" bertanda "CR-013, tarif menunggu PT-28". Bukan blokir |
| N2 | Coding standards §1.6 melarang menyimpan isi klinis persisten di perangkat, sementara catatan konsultasi panjang berisiko hilang bila aplikasi ditutup atau dikunci (CR-009). Data dictionary belum punya status draf | PT-42 |
| N3 | Batas mutlak sesi web 12 jam (ADR-011) dan batas pembayaran PB-07 tidak dapat diperpanjang pengguna; perlu penegasan terhadap SC 2.2.1 | PT-43 |
| N4 | Pamflet menampilkan layanan di luar scope rilis 1 | Tidak dibuat alur; lihat pengamatan 4 |
| N5 | US-103 AC3 mewajibkan penguncian sementara setelah 5 gagal, tetapi tidak mengatur pesan yang tampil; AC2 mewajibkan pesan umum agar tidak membuka informasi akun. Draf awal wireframe A01 layar 5 menampilkan "Akun terkunci sementara" (temuan QA N2) | Wireframe dinetralkan menjadi "menunggu keputusan"; PT-44, diputuskan di Tahap 7 bersama review keamanan (modul T1 autentikasi) |

Tidak ada fakta bisnis `[TIDAK DIKETAHUI]` yang memblokir tahap ini, dan tidak ada dokumen gate yang saling bertentangan sampai mengubah keputusan.

## Usulan penyusun untuk Checkpoint 2

Rincian dan alternatif: `docs/06-design-guideline.md` §11.

| ID | Ringkas |
| --- | --- |
| UP-58 | Arah visual dari palet pamflet + warna turunan; RME memakai identitas yang sama dengan nama permukaan "RME" |
| UP-59 | Font sistem untuk rilis 1; Plus Jakarta Sans (OFL 1.1) sebagai opsi font merek |
| UP-60 | Logo dan ikon aplikasi final dibuat desainer dengan arah simbol rumah-daun |
| UP-61 | Tanpa mode gelap di rilis 1 |
| UP-62 | Navigasi pasien: Beranda, Pesanan, Kesehatan, Profil; tenaga kesehatan: Sesi, Home visit, Jadwal, Profil |
| UP-63 | Enam ubin beranda untuk tujuh layanan; ubin vaksinasi/infus tidak tampil |
| UP-64 | Pengingat "buka aplikasi pada jadwal" di detail pesanan dan ruang tunggu (mitigasi CR-007) |

## Pertanyaan terbuka baru

| ID | Ringkas | Pemilik |
| --- | --- | --- |
| PT-40 | Pembuat logo final, ikon aplikasi, aset toko aplikasi | User |
| PT-41 | Nilai warna mint yang dimaksud (tulisan vs tampilan pamflet) | User / desainer |
| PT-42 | Draf catatan konsultasi/rekam medis di server | Tech Lead + user |
| PT-43 | Batas sesi web 12 jam dan PB-07 terhadap SC 2.2.1 | User + security reviewer |
| PT-44 | Pesan saat akun terkunci (US-103 AC2 vs AC3) | Tech Lead + security reviewer, disetujui user |

## Daftar pertanyaan untuk Checkpoint 2 (diajukan satu per giliran)

1. Apakah arah visual UP-58 disetujui (palet pamflet + warna turunan, sudut membulat, nuansa terang)? Sertakan PT-41: nilai mint yang dimaksud.
2. Font: font sistem saja, atau Plus Jakarta Sans sebagai font merek (UP-59)?
3. Logo dan ikon aplikasi: siapa yang membuat, dan apakah simbol rumah-daun dipertahankan sebagai arah (UP-60, PT-40)?
4. Struktur navigasi dan ubin beranda (UP-62, UP-63): setuju, atau memakai label pamflet?
5. Mode gelap ditunda (UP-61) dan pengingat "buka aplikasi pada jadwal" (UP-64): setuju?
6. Target WCAG 2.2 AA dan paket keputusan desain KEPUTUSAN-160–177: setuju sekaligus?
7. PT-43: batas sesi web 12 jam dan PB-07 diterima sebagai pengecualian, atau perlu jalur perpanjangan?

## Review sebelum gate

- QA (`docs/qa/tahap-06-review-wireframe.md`): LULUS DENGAN CATATAN, 0 blokir. Gate cakupan alur terpenuhi (semua story PRD terpetakan kecuali US-102 yang dihapus CR-007). Tindak lanjut orkestrator atas non-blokir:
  - N1 (tombol "Batalkan pesanan" sebelum bayar di A07): **tidak terkonfirmasi**; PRD §3.6 memuat transisi "Menunggu pembayaran → Dibatalkan pasien" `[DOK: 03-prd.md]`. Wireframe tetap.
  - N2 (pesan akun terkunci): terkonfirmasi sebagai celah spesifikasi; ditangani sebagai N5 di atas dan PT-44.
  - N3 (layar yang hanya berupa keterangan: US-105 AC2/AC4, US-203, US-811 AC2, US-402), N4 (state memuat/hover belum diperagakan di CSS wireframe), N5 (baris pajak CR-013; label WIB di C02): dicatat untuk Tahap 7/8, tidak memicu putaran ulang.
- Fact-checker (`docs/fact-check/tahap-06.md`): LULUS, 0 blokir. Non-blokir N1 (rumusan "dikonfirmasi" diubah menjadi "akan dikonfirmasi" di §7.1 dan KEPUTUSAN-160) dan N3 (aturan kotak persetujuan diberi label KEPUTUSAN-177) diperbaiki orkestrator; N2 (istilah "pengembalian dana") dan N4 (KEPUTUSAN-164/UP-59 ganda) dibawa ke Checkpoint 2.
