# Review QA F1-T83/F1-T84: desain situs, member area HR, backoffice perusahaan dan CMS situs

Versi: 1.0 (2026-11-19) · Reviewer: qa-engineer (independen) · Ketelitian: T2
Objek: `docs/wireframes/s01-situs-publik.html`, `k01-korporat-hr.html`, `c13-bo-verifikasi-perusahaan.html`, `c14-bo-cms-situs.html`, `wireframe-situs.css`, `index.html`; `docs/06-design-guideline.md` v1.2 §10.2 (D-S01, D-K01, D-C13, D-C14); KEPUTUSAN-270–274 (`docs/decision-log.md`).
Acuan: PRD v1.3 EP-18/EP-19 (US-1801, US-1802, US-1901–US-1904, US-1916), BRD v1.4 BR-130–BR-136, ADR-027/028, `docs/cr/CR-021-wawancara.md` putaran 5, guideline §7.2, UP-73 / US-103 AC2.

## Verdict

**LULUS DENGAN CATATAN.** Dua blokir harus ditutup sebelum F1-T84 dinyatakan selesai (keduanya perbaikan dokumen kecil atau keputusan lingkup, bukan desain ulang). Tidak ada temuan Critical. Anti-enumerasi, pratinjau CMS sandbox, dan kontras token lulus.

## Bukti pemeriksaan

| Pemeriksaan | Perintah / cara | Hasil |
| --- | --- | --- |
| Pratinjau CMS tanpa `allow-same-origin` | `grep -n "<iframe" docs/wireframes/*.html`; `grep -c allow-same-origin` | Dua `<iframe class="preview" ... sandbox srcdoc=...>` di C14 baris 67 dan 88, atribut `sandbox` tanpa token (tanpa `allow-scripts`, tanpa `allow-same-origin`). Dua kemunculan string `allow-same-origin` di C14 hanya di teks penjelas. Lulus US-1802 AC8 |
| Pesan login seragam | `grep -o "Tidak dapat masuk</b>[^<]*" k01-korporat-hr.html` | "Email, password, atau kode tidak sesuai. Bila tidak dapat masuk, gunakan Lupa password atau hubungi Layanan Pelanggan." Memuat petunjuk tetap US-103 AC2 / guideline §9. Lulus UP-73 |
| Kontras token (rumus WCAG, skrip Python lokal) | luminans relatif token `wireframe.css` | crumb `ink-2`/mint 4,83; `.td` dan spanduk draf warning/warning-bg 5,79; teks putih/kaki `ink` 14,7; putih/`primary-strong` (langkah aktif) 7,53; `ink-2`/surface 5,54; batas kontrol `border-strong`/bg 3,49. Semua ≥ A2 (4,5) dan A3 (3,0). Catatan: cincin luar `primary-strong` di kaki gelap hanya 1,95, tetapi cincin dalam putih 2 px memberi 14,7 sehingga indikator tetap memenuhi A3/A8 |
| Status berkas | `git status --short docs/` | Bersih: berkas objek review sudah ada di HEAD `3f14a85` (brief menyebut belum di-commit; tidak memengaruhi isi review) |

## Cakupan AC Fase 1 terhadap layar/keadaan

| Story / AC | Layar atau keadaan | Status |
| --- | --- | --- |
| US-1801 AC1, AC2 | S01 layar 1–9; Layanan dari CMS | Ada |
| US-1801 AC3 | S01 layar 10 "Halaman tidak ditemukan" identik | Ada |
| US-1801 AC4 | S01 layar 9: versi, tanggal berlaku, spanduk draf | Ada |
| US-1801 AC5, AC9 (US-1802) | Catatan tanpa cookie, skrip, font, sematan pihak ketiga; Kontak tanpa formulir (KEPUTUSAN-271) | Ada (diuji di header respons, bukan layar) |
| US-1801 AC6 | Daftar A1–A9, A17 di D-S01; uji di F1-T97 | Ada |
| US-1801 AC7 | S01 layar 10 galat umum tanpa jejak teknis (KEPUTUSAN-272) | Ada |
| US-1802 AC1–AC3, AC5–AC8 | C14 layar 1–3 | Ada |
| US-1802 AC4 | C14 layar 1 spanduk "Tidak ada akses" | Ada (lihat N-12) |
| US-1802 audit E-16 hapus/pulihkan | C14 hanya tombol Hapus dan filter "Terhapus" | Sebagian (N-6) |
| US-1901 AC1, AC2, AC3 | K01 layar 2–4 | Ada (bukti kewenangan basi, B-1) |
| US-1901 AC4 | K01 layar 5 identik | Ada |
| US-1901 AC5 | Hanya catatan backend; layar galat pengiriman tidak ada | Sebagian (N-4) |
| US-1901 AC6 | K01 layar 7, 8 | Ada |
| US-1901 AC7 | K01 layar 4 spanduk generik + Layanan Pelanggan | Ada (teks beda dari C13, N-5) |
| US-1902 AC1–AC5 | C13 layar 1; 403 merujuk C01 | Ada; kondisi antrean kosong dan dokumen gagal dibuka tidak ada (N-3) |
| US-1903 AC1, AC3, AC4 | C13 layar 2 | Ada (unggahan satu isian, N-7) |
| US-1903 AC2 | Hanya kalimat "pola layar masuk dengan pesan umum"; layar aktivasi undangan tidak digambar | Sebagian (N-2) |
| US-1904 AC1 | K01 layar 1, 6 | Ada |
| US-1904 AC2 | K01 layar 10 | Ada |
| US-1904 AC3, AC5 | Hanya teks D-K01 ("Data tidak ditemukan", layar "Akun nonaktif") | Sebagian (N-2) |
| US-1904 AC6 | K01 layar 11 | Ada |
| US-1904 AC7 (bagian F1: masuk dengan kode pemulihan) | Tautan "Pakai kode pemulihan" saja | Sebagian (N-2) |
| US-1916 AC1, AC3 | C13 layar 3 | Ada |
| US-1916 AC2 | Tombol nonaktif "dibangun di task berikutnya" | **Tidak ada** (B-2) |

## Blokir

### B-1. Bentuk bukti kewenangan masih ditandai `[TIDAK DIKETAHUI]` padahal sudah dikonfirmasi user

- Lokasi: `k01-korporat-hr.html` layar 4, hint isian `u-kw`: "Bentuk bukti kewenangan (misalnya surat kuasa atau surat penunjukan) menunggu konfirmasi user [TIDAK DIKETAHUI] (US-1901 AC1, PT-79)"; `06-design-guideline.md` D-K01 "Kondisi khusus": "bentuk bukti kewenangan (PT-79 sisa)".
- Diharapkan: sesuai `CR-021-wawancara.md` putaran 5 (4b, `[USER]` "4BY"), PRD US-1901 AC1 ("dikonfirmasi user") dan PT-79 "Terjawab": bukti kewenangan berupa surat kuasa atau surat penunjukan dari perusahaan yang ditandatangani pejabat berwenang, diunggah saat daftar, dicek Verifikator.
- Aktual: layar memberi tahu calon HR bahwa bentuknya belum diketahui; dokumen desain bertentangan dengan PRD (aturan CLAUDE.md: dokumen yang bertentangan dilaporkan).
- Usulan: hint K01 menjadi "Surat kuasa atau surat penunjukan dari perusahaan, ditandatangani pejabat berwenang `[USER] C21-P4` (putaran 5, 4b)"; hapus "bentuk bukti kewenangan (PT-79 sisa)" dari daftar `[TIDAK DIKETAHUI]` D-K01; di C13 layar 1 tambahkan petunjuk pemeriksaan untuk Verifikator ("periksa tanda tangan pejabat berwenang") agar US-1902 AC3 "bukti kewenangan tidak memadai" punya kriteria yang dapat diuji. Siapa yang dianggap "pejabat berwenang" tetap `[TIDAK DIKETAHUI]` bila belum ditetapkan.
- Severity: Medium. Rujukan: US-1901 AC1, US-1902 AC1/AC3, BR-136, PT-79.
- Catatan: `docs/backlog.md` baris US-1901 masih menulis "bentuk bukti kewenangan G-27"; di luar lingkup review ini, perlu dirapikan pemiliknya.

### B-2. US-1916 AC2 (nonaktifkan perusahaan) tidak punya layar, padahal masuk Fase 1

- Lokasi: `c13-bo-verifikasi-perusahaan.html` layar 3: tombol "Nonaktifkan perusahaan" `aria-disabled`, alasan "aturan nasib pesanan berjalan menunggu PT-63; penonaktifan (US-1916 AC2) dibangun di task berikutnya"; D-C13 butir 3.
- Diharapkan: `docs/backlog.md` menjadwalkan US-1916 di Fase 1 "bagian K1, F1-T106; pesanan dan invoice di Fase 3", dan phase plan §2 Fase 1 memuat "US-1916 bagian K1". Di Fase 1 belum ada pesanan korporat, sehingga PT-63 (nasib pesanan) tidak menghalangi penonaktifan. KU-10 dan US-1904 AC5 (sesi HR ditolak setelah perusahaan nonaktif) juga Fase 1 dan butuh pemicu ini untuk diuji.
- Aktual: tidak ada dialog nonaktifkan (alasan, dampak "semua akun HR ditolak seketika"), tidak ada keadaan berhasil/gagal, dan keputusan menunda tidak tercatat sebagai KEPUTUSAN.
- Usulan: pilih salah satu: (a) gambar dialog nonaktifkan dengan dampak dan catatan "nasib pesanan berjalan `[TIDAK DIKETAHUI]` (PT-63), belum relevan di Fase 1"; atau (b) catat KEPUTUSAN yang mengeluarkan US-1916 AC2 dari Fase 1 dan perbarui backlog/phase plan (perubahan lingkup, perlu persetujuan user).
- Severity: Medium. Rujukan: US-1916 AC2, US-1904 AC5, KU-10, BR-136.

## Non-blokir

| # | Temuan | Usulan | Sev. | Rujukan |
| --- | --- | --- | --- | --- |
| N-1 | K01 layar 4 menambah "Pernyataan: saya berwenang mewakili perusahaan ini" yang tidak ada di US-1901 AC1/UP-79 (unsur UI baru, bukan hanya bunyinya yang belum diketahui) | Hapus, atau usulkan sebagai UP/KEPUTUSAN bertanda menunggu | Low | US-1901 AC1 |
| N-2 | Keadaan Fase 1 yang hanya dideskripsikan, belum digambar: aktivasi undangan HR (set password + TOTP) dan undangan kedaluwarsa/terpakai (US-1903 AC2, US-1904 AC2); masuk dengan kode pemulihan (US-1904 AC7 bagian F1); "Data tidak ditemukan" isolasi (AC3); layar "Akun nonaktif" (AC5, KU-10) | Tambahkan bingkai ringkas atau rujuk eksplisit ke layar pola yang sudah ada (C01/A15) beserta teksnya | Low | US-1903 AC2, US-1904 AC2/AC3/AC5/AC7 |
| N-3 | Keadaan kosong/memuat/galat belum ada di: antrean verifikasi perusahaan kosong; URL dokumen kedaluwarsa atau gagal dibuka (C13); daftar konten kosong dan gagal simpan (C14); galat umum halaman member area (K01) | Tambahkan, atau nyatakan memakai pola guideline §5.3 | Low | US-1902 AC1, ADR-018, US-1802 |
| N-4 | US-1901 AC5: tidak ada layar untuk kegagalan pengiriman di tengah proses (jaringan putus, server galat). Pengguna tidak tahu apakah harus mengirim ulang | Gambar galat umum dengan "Coba lagi", isian tetap, dan pernyataan bahwa belum ada akun yang dibuat | Medium | US-1901 AC5, KU-05 |
| N-5 | Teks NIB ganda berbeda: K01 "Data yang Anda kirim tidak dapat digunakan untuk mendaftar..." (spanduk), C13 "NIB tidak dapat digunakan untuk mendaftar..." (di isian NIB), padahal caption C13 menyebut "pesan generik yang sama". US-1903 AC4 meminta "ditolak seperti US-1901 AC7" | Satukan teks kanonik di guideline | Low | US-1901 AC7, US-1903 AC4 |
| N-6 | C14: hapus tanpa isian alasan (ADR-003 `delete_reason`), tanpa aksi Pulihkan, padahal D-C07/D-C14 menyebut "hapus dan pulihkan" (E-16) | Gambar dialog hapus dengan alasan wajib dan aksi Pulihkan di filter "Terhapus" | Medium | US-1802 (Audit E-16), ADR-003 |
| N-7 | C13 "Buat perusahaan" memakai satu `input type=file multiple` untuk tiga dokumen, sehingga galat per field (KU-01) dan label dokumen di panel Verifikator tidak dapat dibedakan | Tiga isian terpisah seperti K01 layar 4 | Medium | US-1903 AC1, US-1901 AC2 |
| N-8 | K01 layar 2 "Kemajuan tersimpan di perangkat selama sesi pendaftaran": tidak ada di PRD/KEPUTUSAN; berisiko menyimpan password dan data perusahaan di penyimpanan peramban | Tegaskan password dan berkas tidak pernah disimpan di sisi klien, atau hapus kalimat itu; bila dipertahankan, catat sebagai KEPUTUSAN dengan review security | Medium | US-1901 AC1, ADR-027 |
| N-9 | Galat ukuran di K01 layar 4 selalu "melebihi 10 MiB"; untuk JPG/PNG batasnya 5 MiB. Atribut `accept` hanya ada di isian NIB | Pesan bergantung jenis berkas; `accept` sama di ketiga isian | Low | NFR-SEC-08, US-1901 AC2 |
| N-10 | Kotak centang persetujuan K01 (layar 4, 6) berupa `span role=checkbox tabindex=0`; kanal di C14 berupa `span.chip` tanpa peran interaktif | Implementasi wajib memakai `input type=checkbox` asli (A7, A11); tulis di caption | Low | §7.2 A7, A11 |
| N-11 | Tautan `aria-disabled` dengan `href="#"` (Unduh aplikasi S01 layar 8, menu nonaktif K01 layar 9) tetap dapat difokus dan diaktifkan | Pakai teks/elemen non-tautan dengan alasan tertulis | Low | §7.2 A7; D-S01 butir 3 |
| N-12 | C14 layar 1 menggabungkan daftar konten dengan spanduk 403 di bingkai yang sama, sehingga tidak jelas apakah akun tanpa sub-peran Konten dapat melihat daftar | Pisahkan bingkai 403 (pola C01) | Low | US-1802 AC4 |
| N-13 | Tautan kaki situs `min-height: 32px` (`wireframe-situs.css`), sedangkan D-S01 menyatakan target ≥ 40 px desktop. Tetap lulus WCAG 2.5.8 (24 px) | Samakan klaim atau ukuran | Low | §7.2 A9 |
| N-14 | `summary` (tombol Menu) tidak termasuk selektor `:focus-visible` token di `wireframe.css`, sehingga memakai outline bawaan peramban | Tambahkan `summary:focus-visible` | Low | §7.2 A8 |
| N-15 | Wireframe memuat banyak `<main>` bersarang di dalam `<main class="cap-grid">` dan `header`/`footer` situs di dalam `section` (bukan landmark). Hanya di berkas wireframe; implementasi wajib satu `main` dan landmark tingkat atas | Catat sebagai pengingat F1-T96/F1-T97 | Low | §7.2, D-S01 |
| N-16 | Pesan login realm `korporat` belum disamakan kata per kata dengan realm lain; A01 masih "Email atau password salah" (menunggu UI-03) | Tetapkan satu teks kanonik di guideline §9 dan uji identik lintas realm termasuk `korporat` | Low | US-103 AC2, UP-73 |
| N-17 | KEPUTUSAN-274: status "Tayang (terjadwal)"; filter C14 memakai "Selesai" sementara US-1801 AC3/US-1802 AC2 memakai "berakhir" | Lihat celah spesifikasi S-6 | Low | US-1101 AC2, US-1801 AC3 |

Tidak ditemukan fakta bisnis karangan pada konten situs: teks Tentang Kami, visi/misi, rekanan, kontak, tautan toko, versi teks hukum, slogan, logo, dan domain ditandai `[TIDAK DIKETAHUI]`; kalimat peran platform dan "Android dan iOS" berlabel `[DOK: 00-project-brief.md]`; "Dioperasikan oleh PT. Arunika" sesuai BRD BR-130. Pengecualian hanya N-1 dan N-8.

## Celah spesifikasi (untuk pemilik PRD, bukan untuk desain)

| # | Celah | Usulan rumusan AC |
| --- | --- | --- |
| S-1 | US-1901 AC7 "NIB yang sama sudah dipakai perusahaan terdaftar" ambigu: apakah termasuk status Menunggu verifikasi dan Ditolak? Bila ya, pendaftar tak berhak dapat "mengunci" NIB perusahaan lain sebelum diverifikasi | "Given NIB sudah dipakai perusahaan berstatus Menunggu verifikasi, Aktif, Ditolak, atau Nonaktif `[pilih]`, When mendaftar, Then ditolak dengan pesan generik... dan Verifikator dapat menyelesaikan konflik lewat Layanan Pelanggan" |
| S-2 | US-1901 AC4 vs AC7: urutan pemeriksaan email terdaftar dan NIB ganda, serta nasib data/dokumen perusahaan saat email sudah terdaftar, tidak diatur | "Given email sudah terdaftar, Then tidak ada perusahaan, dokumen, maupun persetujuan yang tersimpan; NIB ganda diperiksa sebelum/terlepas dari status email sehingga respons tidak membedakan email" |
| S-3 | Tidak ada batas laju untuk endpoint publik pendaftaran perusahaan (unggahan dokumen, pengecekan NIB), berbeda dengan US-103 AC4c | Tambahkan AC batas laju per IP dan per email, dengan pesan sama |
| S-4 | US-1904 tidak memuat AC lupa password realm `korporat`, padahal layar masuk K01 menampilkan "Lupa password" | Tambahkan AC yang merujuk pola US-103 AC4–AC4c untuk realm `korporat` |
| S-5 | US-1904 AC2: boleh-tidaknya HR menonaktifkan dirinya sendiri (bila bukan terakhir), dan perilaku saat email yang diundang sudah menjadi HR perusahaan lain, tidak diatur. US-1903 AC1 juga tidak mengatur email HR pertama yang sudah terdaftar | Tetapkan aturan dan pesan anti-enumerasi untuk ketiganya |
| S-6 | Kosakata status konten tidak seragam: US-1101 AC2 "Selesai", US-1801 AC3/US-1802 AC2 "berakhir", "terjadwal" tanpa definisi (KEPUTUSAN-274) | Satu daftar status di PRD §3.6 dipakai di semua AC |
| S-7 | US-1802 tidak mengatur pemulihan versi halaman tetap yang terhapus ketika versi lain sedang Tayang (bertabrakan dengan AC2 satu versi tayang) | "When versi halaman tetap dipulihkan, Then berstatus Draf, tidak langsung Tayang" `[usulan]` |

## Skenario uji yang dapat diturunkan dari desain ini (usulan untuk task implementasi)

Belum dicatat di kolom "Test case" `docs/traceability.md` karena brief membatasi perubahan ke berkas ini saja; dicatat saat task implementasi terkait.

| ID usulan | Skenario | Sumber |
| --- | --- | --- |
| TC-1801-03 | URL konten Draf, terjadwal, berakhir, diarsipkan, terhapus, dan URL acak: HTML respons identik (kecuali `<title>`/path), status 404 | US-1801 AC3 |
| TC-1801-07 | Backend konten mati: halaman galat tanpa stack trace, nama host, atau correlation_id | US-1801 AC7, KEPUTUSAN-272 |
| TC-1802-08 | Payload `<script>`, `on*`, `javascript:` di pratinjau: tidak tereksekusi; `iframe.sandbox` tidak memuat `allow-same-origin` maupun `allow-scripts`; `document.cookie` backoffice tidak terbaca | US-1802 AC6–AC8 |
| TC-1901-04 | Daftar dengan email baru vs email terdaftar: respons dan layar identik byte per byte (kecuali token CSRF) | US-1901 AC4 |
| TC-1901-07 | NIB ganda: pesan tidak memuat nama, email, atau status perusahaan lain; teks sama di pendaftaran mandiri dan buat perusahaan | US-1901 AC7, US-1903 AC4 |
| TC-1902-04 | Dua Verifikator memutuskan bersamaan: satu berhasil, kedua "Pengajuan sudah diputuskan", keduanya tercatat (E-10) | US-1902 AC4 |
| TC-1904-02 | Nonaktifkan HR aktif terakhir: ditolak | US-1904 AC2 |
| TC-1904-L | Login gagal (password salah, akun tidak ada, terkunci, nonaktif, kode salah): teks identik | US-1904 AC1, US-103 AC2, UP-73 |

## Tindak lanjut

1. Pemilik desain menutup B-1 dan B-2, lalu meminta QA memeriksa ulang (cukup diff).
2. N-4, N-6, N-7, N-8 (Medium) sebaiknya ditutup sebelum F1-T103/F1-T106/F1-T95 dimulai.
3. S-1–S-7 diteruskan ke pemilik PRD sebagai pertanyaan; S-1 dan S-2 menyentuh modul T1 (autentikasi, dokumen perusahaan) dan perlu jawaban sebelum F1-T101.
