# Design Guideline: Ruang Sehat

Tanggal: 2026-11-18
Versi: 1.3 (2026-11-26, CR-033 C-01–C-03, keputusan user Q15/Q16; font situs Plus Jakarta Sans Q18): token khusus situs publik `web/situs` di §3.3a (skala tipografi situs) dan §3.4a (ritme seksi, kontainer, radius, gerak), KEPUTUSAN-353–359; tidak mengubah token bersama §3.1–§3.4 maupun web lain. Versi 1.2 (2026-11-19, CR-021, F1-T83/F1-T84): wireframe HTML baru S01 (situs publik sembilan halaman), K01 (member area HR), C13 (verifikasi dan daftar perusahaan), C14 (CMS kanal situs); deskripsi D-S01, D-K01, D-C13, D-C14 di §10.2; permukaan web situs dan korporat di §2; KEPUTUSAN-270–274. Konten bisnis situs (teks Tentang Kami, daftar rekanan, kontak, teks hukum) tetap `[TIDAK DIKETAHUI]`; PT-40 dan sisa PT-45 terbuka. Versi 1.1 (2026-11-18, tindak lanjut Checkpoint 2): alur baru A15 (pendaftaran TOTP pasien, kode pemulihan, perangkat hilang) dan G01 (perangkat dan sesi aktif) sebagai deskripsi terstruktur; §9 MFA mencakup pasien; D-C05 reset MFA pasien; PT-43 dan PT-44 terjawab. Versi 0.1 (draf Tahap 6) disetujui di Checkpoint 2: arah visual, font, logo, navigasi, ubin, mode gelap, pengingat jadwal (UP-58–UP-64) dan KEPUTUSAN-160–177 `[USER] C2-B8, C2-B9, C2-B10, C2-B11, C2-B15`. PT-40, PT-41, dan sisa PT-45 tetap terbuka
Penyusun: Claude (persona UI/UX Designer / Design System Lead), Mode Otomatis
Pemilik dan penyetuju: user (project manager, meneruskan ke CEO dan CTO) `[DOK: 00-project-brief.md]`
Dasar: PRD v1.1 `[DOK: 03-prd.md]`, stack v1.0 `[DOK: 05-stack.md]`, coding standards `[DOK: coding-standards.md]`, backlog CR-007, CR-009, CR-012, CR-013, CR-015 `[DOK: backlog.md]`, pamflet `[DOK: input/Rumah Sehat Apps.jpeg]`, catatan T5-Q12 `[DOK: 05-catatan-stack.md]`
Tingkat ketelitian: T2 Standar. Bagian aksesibilitas (§7) serta layar autentikasi, persetujuan data, dan pembayaran (§9, A01, A07, A12, A13, B01, C01, C03) diperlakukan teliti karena menyentuh modul T1 di `CLAUDE.md`.

Wireframe: `docs/wireframes/index.html` (HTML statis, satu CSS bersama `docs/wireframes/wireframe.css`, tanpa CDN atau font jarak jauh).

## 0. Cara membaca

| Penanda | Arti |
| --- | --- |
| `[DOK: ...]`, `US-xxx`, `AB-xx`, `PB-xx`, `KU-xx`, `NFR-...` | Dokumen proyek yang sudah lulus gate. Penanda PRD berfungsi sebagai `[DOK: 03-prd.md]` |
| `[SRC-xx]` | Fakta eksternal terverifikasi, tercatat di `docs/sources.md` |
| `[KEPUTUSAN-xx]` | Keputusan desain/teknis penyusun, tercatat di `docs/decision-log.md` (KEPUTUSAN-160–177) |
| `[UP-xx]` | Usulan penyusun yang menyangkut preferensi bisnis atau merek dan **wajib** disetujui user di Checkpoint 2 (§11, UP-58–UP-64) |
| `[TIDAK DIKETAHUI]` | Belum ada informasi, masuk pertanyaan terbuka (§12, PT-40–PT-46) |

Semua nilai visual di dokumen ini (warna, font, logo) berstatus **usulan** sampai Checkpoint 2. Arah visual adalah preferensi bisnis/merek, sehingga tidak diputuskan penyusun `[UP-58]`.

## 1. Prinsip desain

| # | Prinsip | Dasar |
| --- | --- | --- |
| P1 | Alur mengikuti PRD. Tidak ada layar untuk fitur di luar scope rilis 1 (rating, voucher, multi-bahasa, login Google/Apple, push) | PRD §2.2, CR-007 |
| P2 | Kejelasan biaya sebelum membayar: harga satu angka, baris terpisah untuk ongkir, pajak, dan biaya transaksi, aturan pembatalan dan AB-16 selalu terlihat sebelum tombol bayar | AB-13, AB-16, US-801, CR-013 |
| P3 | Status selalu ditulis dengan teks (tidak hanya warna) memakai nama status PRD §3.6 | PRD §3.6 `[SRC-256]` (SC 1.4.1) |
| P4 | Data sensitif tampil seminimal mungkin: NIK termasking, isi klinis tidak masuk notifikasi, layar disembunyikan di pengalih aplikasi | §3.1, §7.3, AB-73, ADR-021 butir 1 |
| P5 | Tidak ada aksi ganda: tombol kirim dikunci dan menampilkan status memuat selama permintaan berjalan | KU-02, KU-05 |
| P6 | Aksesibel sejak awal: target WCAG 2.2 Level AA di semua permukaan | `[KEPUTUSAN-160]` `[SRC-256]` |
| P7 | Satu sistem token untuk dua teknologi: paket Flutter `rs_design` dan paket web `web/packages/ui` memakai nama dan nilai token yang sama | `[DOK: coding-standards.md]` §1.5, §1.6, KEPUTUSAN-116 |

## 2. Permukaan dan tempat implementasi

| Permukaan | Pengguna | Teknologi | Lokasi token/komponen | Sumber |
| --- | --- | --- | --- | --- |
| Aplikasi pasien | Pasien | Flutter 3.50.0 | `mobile/packages/rs_design` | `[DOK: 05-stack.md]` §1.3, `[DOK: coding-standards.md]` §1.6 |
| Aplikasi tenaga kesehatan | Dokter, psikolog, fisioterapis, tenaga medis | Flutter 3.50.0 | `mobile/packages/rs_design` | sama |
| Backoffice Ruang Sehat | Admin RS (7 sub-peran) | Next.js 16.4.6 | `web/packages/ui` | `[DOK: 05-stack.md]` §1.2, `[DOK: coding-standards.md]` §1.5 |
| Portal rekanan | Admin klinik, lab, outlet | Next.js 16.4.6 | `web/packages/ui` | sama |
| Web RME minimalis | Pengguna RME | Next.js 16.4.6 | `web/packages/ui` | sama; aplikasi terpisah dari Ruang Sehat (BR-120) |
| Situs publik (`web/situs`, CR-021) | Pengunjung anonim | Next.js, dirender server dari endpoint konten publik; tanpa cookie sesi, skrip, font, atau sematan pihak ketiga | `web/packages/ui` | ADR-027 butir 1 dan 5; US-1801 AC5, US-1802 AC9 `[KEPUTUSAN-270]` |
| Member area HR (`web/korporat`, CR-021) | HR perusahaan | Next.js dengan BFF (modul T1); realm Keycloak `korporat`, tema login yang sama | `web/packages/ui` | ADR-027 butir 1 dan 3 `[KEPUTUSAN-270]` |
| Halaman login, OTP, atur ulang password, aktivasi undangan, login ulang | Semua populasi | Tema login Keycloak (extend tema bawaan, stylesheet dan template sendiri) | Tema di konfigurasi realm | ADR-011 butir 7 dan konsekuensi `[SRC-233]` |
| Halaman verifikasi resep publik | Apotek (anonim) | HTML dari backend Rust | Stylesheet statis dari token yang sama | US-309 AC6–AC8 (CR-008) |
| Email notifikasi | Pasien, tenaga kesehatan, admin rekanan | Template email lewat Amazon SES API | Template HTML dengan CSS inline | `[DOK: 05-stack.md]` (SES), AB-73 |

Pustaka komponen UI web: **tidak menambah pustaka pihak ketiga**. Komponen dasar dibangun sendiri di `web/packages/ui` dengan CSS custom properties dari token §3 `[KEPUTUSAN-163]`. Alasan: `docs/05-stack.md` §1.2 menyerahkan pilihan ke Tahap 6, dan aturan proyek melarang menambah dependensi tanpa verifikasi registry; jumlah komponen di §4 cukup kecil untuk dibangun dan diuji sendiri. Bila Tahap 8 menilai perlu pustaka komponen, pemilihannya lewat CR dengan verifikasi registry dan catatan di `docs/sources.md`.

## 3. Design token

Penamaan token: `kategori.peran.varian` (misalnya `color.text.primary`, `space.4`). Di Flutter menjadi konstanta Dart di `rs_design`; di web menjadi CSS custom properties (`--c-primary`, `--s4`) di `web/packages/ui`. Nilai yang sama juga dipakai tema Keycloak dan halaman verifikasi resep `[KEPUTUSAN-163]`. Contoh penerapan: `docs/wireframes/wireframe.css`.

### 3.1 Warna

**Warna dasar dari pamflet** (usulan, `[UP-58]`; nilai heksadesimal sesuai tulisan di pamflet `[DOK: input/Rumah Sehat Apps.jpeg]`):

| Token | Nilai | Peran | Sumber |
| --- | --- | --- | --- |
| `color.brand.primary` | `#087F78` | Tombol utama, ikon aktif, batas kontrol terpilih | `[DOK: input/Rumah Sehat Apps.jpeg]` `[UP-58]` |
| `color.brand.mint` | `#DFF4ED` | Latar ubin layanan, badge info/sukses, item navigasi web aktif | sama |
| `color.bg.app` | `#FAFBF7` | Latar layar | sama |
| `color.bg.surface` | `#FFFFFF` | Kartu, lembar, dialog, isian | sama |

Catatan: di pamflet, lingkaran contoh warna kedua tampak lebih gelap dari nilai `#DFF4ED` yang tertulis di bawahnya. Dokumen ini memakai nilai tertulis; kepastiannya ditanyakan (PT-41).

**Warna turunan** (penyusun, dipilih agar lulus kontras §3.2) `[KEPUTUSAN-163]`:

| Token | Nilai | Peran |
| --- | --- | --- |
| `color.brand.primaryStrong` | `#065F59` | Teks/tautan berwarna merek, tombol utama saat ditekan/hover, cincin fokus, chip terpilih, teks badge info |
| `color.text.primary` | `#1A2B2E` | Teks utama |
| `color.text.secondary` | `#5C6B6F` | Teks pendukung, placeholder, keterangan |
| `color.border.control` | `#7A898C` | Batas isian, chip, checkbox tidak aktif (komponen yang wajib terlihat) |
| `color.border.divider` | `#DCE3E1` | Garis pemisah dekoratif saja; tidak boleh menjadi satu-satunya penanda batas komponen |
| `color.bg.neutral` | `#EEF1F0` | Latar badge netral, kepala tabel, tombol nonaktif |
| `color.status.danger` / `.dangerBg` | `#B3261E` / `#FDECEA` | Galat, aksi destruktif, status gagal/ditolak |
| `color.status.success` | `#1E7A3E` (latar memakai `brand.mint`) | Status berhasil/selesai |
| `color.status.warning` / `.warningBg` | `#8A5300` / `#FFF4DB` | Peringatan, status menunggu |
| `color.text.onPrimary` | `#FFFFFF` | Teks di atas `primary`, `primaryStrong`, `danger`, `success` |

Mode gelap tidak dirancang untuk rilis 1 `[UP-61]`.

### 3.2 Kontras warna

**Aturan** `[KEPUTUSAN-161]`:
- Semua teks, termasuk teks besar, wajib ≥ 4,5:1 terhadap latarnya. Penyusun tidak memakai pengecualian teks besar 3:1, supaya aturan sederhana dan tidak bergantung konversi ukuran. Ini lebih ketat dari SC 1.4.3 Contrast (Minimum) Level AA, yang mensyaratkan "a contrast ratio of at least 4.5:1" dan 3:1 untuk teks besar `[SRC-256]`.
- Komponen antarmuka dan grafis yang diperlukan untuk memahami isi wajib ≥ 3:1 terhadap warna di sebelahnya (SC 1.4.11 Non-text Contrast, Level AA) `[SRC-256]`. Komponen nonaktif dikecualikan oleh SC 1.4.3 dan 1.4.11 `[SRC-256]`, tetapi tetap diberi label teks.
- Teks logo tidak punya syarat kontras menurut SC 1.4.3 ("Text that is part of a logo or brand name has no contrast requirement") `[SRC-256]`; logo tetap ditempatkan pada pasangan warna yang lulus.

**Cara hitung** (definisi WCAG 2.2) `[SRC-256]`:
1. Setiap kanal 8 bit dibagi 255: `c = C8bit / 255`.
2. Linearisasi: bila `c ≤ 0,04045` maka `c_lin = c / 12,92`; selain itu `c_lin = ((c + 0,055) / 1,055) ^ 2,4`.
3. Relative luminance: `L = 0,2126 R_lin + 0,7152 G_lin + 0,0722 B_lin`.
4. Rasio kontras: `(L1 + 0,05) / (L2 + 0,05)`, dengan `L1` luminance warna lebih terang.

Contoh `#087F78` di atas `#FFFFFF`: R = 8 → 0,0314 → 0,00243; G = 127 → 0,4980 → 0,21223; B = 120 → 0,4706 → 0,18782. L = 0,2126×0,00243 + 0,7152×0,21223 + 0,0722×0,18782 = 0,16586. Putih L = 1. Rasio = 1,05 / 0,21586 = **4,86:1**.

Luminance per warna (dihitung dengan rumus di atas, dibulatkan 4 desimal): `#FFFFFF` 1,0000; `#FAFBF7` 0,9603; `#DFF4ED` 0,8650; `#EEF1F0` 0,8738; `#FDECEA` 0,8681; `#FFF4DB` 0,9108; `#087F78` 0,1659; `#065F59` 0,0894; `#1A2B2E` 0,0214; `#5C6B6F` 0,1394; `#7A898C` 0,2392; `#B3261E` 0,1106; `#1E7A3E` 0,1454; `#8A5300` 0,1159.

**Hasil per pasangan** (dibulatkan 2 desimal). Perhitungan manual oleh penyusun; wajib diulang otomatis di Tahap 8 (§7.4).

| # | Teks/elemen | Latar | Rasio | Syarat | Hasil |
| --- | --- | --- | --- | --- | --- |
| K1 | `text.primary` #1A2B2E | #FFFFFF / #FAFBF7 / #DFF4ED | 14,70 / 14,14 / 12,81 | 4,5 | Lulus |
| K2 | `text.secondary` #5C6B6F | #FFFFFF / #FAFBF7 / #DFF4ED / #EEF1F0 | 5,54 / 5,33 / 4,83 / 4,88 | 4,5 | Lulus |
| K3 | `brand.primary` #087F78 sebagai teks | #FFFFFF / #FAFBF7 | 4,86 / 4,68 | 4,5 | Lulus |
| K4 | `brand.primary` #087F78 sebagai teks | #DFF4ED (mint) | **4,24** | 4,5 | **Gagal untuk teks.** Teks di atas mint memakai `primaryStrong` #065F59 (K6); aturan ini disetujui di Checkpoint 2 `[USER] C2-B15` (KEPUTUSAN-163) dan sudah diterapkan di komponen §4 (ubin, navigasi samping, badge info, tab, chip) |
| K5 | #FFFFFF di atas `brand.primary` (tombol utama) | #087F78 | 4,86 | 4,5 | Lulus |
| K6 | `primaryStrong` #065F59 (tautan, badge info, teks merek) | #FFFFFF / #FAFBF7 / #DFF4ED | 7,53 / 7,25 / 6,56 | 4,5 | Lulus |
| K7 | #FFFFFF di atas `primaryStrong` (tombol ditekan, chip terpilih) | #065F59 | 7,53 | 4,5 | Lulus |
| K8 | `danger` #B3261E | #FFFFFF / #FAFBF7 / #FDECEA | 6,54 / 6,29 / 5,72 | 4,5 | Lulus |
| K9 | #FFFFFF di atas `danger` (tombol destruktif) | #B3261E | 6,54 | 4,5 | Lulus |
| K10 | `success` #1E7A3E | #FFFFFF / #FAFBF7 / #DFF4ED | 5,37 / 5,17 / 4,68 | 4,5 | Lulus |
| K11 | `warning` #8A5300 | #FFFFFF / #FFF4DB | 6,33 / 5,79 | 4,5 | Lulus |
| K12 | `text.primary` di atas latar peringatan | #FDECEA / #FFF4DB | 12,85 / 13,45 | 4,5 | Lulus |
| K13 | #FFFFFF di atas `text.primary` (toast, layar video) | #1A2B2E | 14,70 | 4,5 | Lulus |
| N1 | `border.control` #7A898C (batas isian, chip, checkbox) | #FFFFFF / #FAFBF7 / #DFF4ED | 3,63 / 3,49 / 3,16 | 3 | Lulus |
| N2 | `brand.primary` sebagai batas/ikon kontrol aktif | #FFFFFF / #FAFBF7 | 4,86 / 4,68 | 3 | Lulus |
| N3 | Cincin fokus `primaryStrong` | #FFFFFF / #FAFBF7 / #DFF4ED | 7,53 / 7,25 / 6,56 | 3 | Lulus |
| N4 | `brand.primary` sebagai ikon/batas | #DFF4ED | 4,24 | 3 | Lulus (non-teks saja) |
| N5 | Latar ubin mint #DFF4ED | #FAFBF7 / #FFFFFF | 1,10 / 1,15 | — | Dekoratif. Batas ubin tidak memenuhi 3:1, sehingga ubin wajib dikenali dari label teks dan ikonnya, bukan dari bidang warnanya |

### 3.3 Tipografi

**Font** `[KEPUTUSAN-164]`: rilis 1 memakai font bawaan sistem/platform tanpa berkas font tambahan (Flutter: font bawaan platform; web: susunan `system-ui, -apple-system, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif`). Tidak ada font jarak jauh (tanpa CDN). Alternatif merek: Plus Jakarta Sans, berlisensi SIL Open Font License 1.1 dan tersedia sebagai font variabel (`wght`) `[SRC-259]`, dibundel lokal sebagai aset bila disetujui `[UP-59]`. Bila disetujui, cara bundel di Flutter dan Next.js serta berkas fontnya diverifikasi di Tahap 8 (bukan paket pub/npm).

**Skala** `[KEPUTUSAN-165]` (mobile dalam dp, web dalam px; ukuran minimum teks isi 14):

| Token | Mobile (ukuran/tinggi baris, tebal) | Web | Pemakaian |
| --- | --- | --- | --- |
| `type.display` | 26/32, 700 | 28/36, 700 | Judul layar penting, angka total |
| `type.titleLg` | 22/28, 600 | 24/32, 600 | Judul halaman web |
| `type.title` | 18/24, 600 | 18/24, 600 | Judul bagian, judul dialog, app bar |
| `type.subtitle` | 16/24, 600 | 16/24, 600 | Judul kartu |
| `type.body` | 16/24, 400 | 14/20, 400 | Teks isi |
| `type.bodySm` | 14/20, 400 | 13/18, 400 | Teks pendukung, tabel padat web |
| `type.label` | 16/24, 600 (tombol) | 14/20, 600 | Label tombol dan isian |
| `type.caption` | 12/16, 400 | 12/16, 400 | Metadata (waktu, nomor). Tidak untuk informasi yang wajib dibaca sebelum bertindak |

Aturan: tinggi baris ≥ 1,3 kali ukuran; kontainer teks tidak bertinggi tetap, sehingga pengaturan spasi teks pengguna tidak memotong isi (SC 1.4.12) `[SRC-256]`; angka uang dan tabel memakai angka tabular di web.

### 3.3a Skala tipografi situs publik (v1.3, CR-033)

Berlaku hanya untuk `web/situs` (situs pemasaran), sebagai CSS custom properties `--situs-*` di `web/situs/src/app/situs.css`; skala §3.3 tetap untuk web lain `[KEPUTUSAN-358]`. Nilai dari brief situs §7.3 (disetujui lewat Q15/Q16 `[USER]` 2026-11-26).

| Token situs | Desktop (≥ 768) | Seluler (< 768) | Pemakaian |
| --- | --- | --- | --- |
| `--situs-hero` | 48/56, 800 | 32/40, 800 | Judul hero Beranda |
| `--situs-h1` | 36/44, 800 | 28/36, 800 | Judul halaman (pita judul) |
| `--situs-h2` | 28/36, 800 | 24/32, 800 | Judul seksi, `##` Markdown teratas |
| `--situs-h3` | 20/28, 700 | 18/26, 700 | Judul kartu |
| `--situs-lead` | 18/28, 400 | 17/26, 400 | Kalimat pembuka (`text.secondary`, K2) |
| `--situs-body` | 16/26, 400 | 16/26, 400 | Teks isi dan artikel |
| `--situs-eyebrow` | 13/18, 700, kapital, spasi huruf 0,08em | sama | Label kecil di atas judul (`primaryStrong`, K6) |

Font situs: **Plus Jakarta Sans** (keputusan user Q18 2026-11-26 `[USER]`, menutup UP-59 untuk situs; `[KEPUTUSAN-356]`), satu berkas woff2 variabel `wght` 200–800 di-host sendiri lewat `next/font/local`, `font-display: swap`, cadangan tumpukan sistem §3.3 `[SRC-881]` `[SRC-882]`. Web lain (backoffice, korporat, portal, RME) tetap font sistem `[KEPUTUSAN-164]`. Judul memakai spasi huruf −0,015 sampai −0,025em. Catatan: tinggi baris judul situs (hero 1,17; h1 1,22; h2 1,29) di bawah aturan "≥ 1,3 kali ukuran" §3.3, sama seperti `type.display` web 28/36 (1,29) yang sudah ada; teks isi dan pembuka memenuhi (1,56–1,63). Apakah aturan 1,3 berlaku untuk judul perlu ditegaskan user/desainer (dicatat di laporan CR-033); kontainer tetap tanpa tinggi tetap (SC 1.4.12).

### 3.4 Spacing, radius, elevation

`[KEPUTUSAN-165]`

| Token | Nilai | Pemakaian |
| --- | --- | --- |
| `space.1`–`space.16` | 4, 8, 12, 16, 20, 24, 32, 40, 48, 64 | Kelipatan 4. Jarak isi layar mobile 16; jarak antarkartu 12; jarak halaman web 24 |
| `radius.sm` | 4 | Checkbox |
| `radius.md` | 8 | Isian, tombol web, gambar |
| `radius.lg` | 12 | Peringatan (alert) |
| `radius.xl` | 16 | Kartu, ubin, lembar bawah, dialog (mengikuti sudut membulat di pamflet `[UP-58]`) |
| `radius.full` | 999 | Tombol mobile, chip, badge |
| `elevation.0` | Tanpa bayangan | Latar, daftar |
| `elevation.1` | Bayangan 0 1 2 (12% `text.primary`) + garis 1 px `border.divider` | Kartu. Batas kartu tidak bergantung bayangan saja |
| `elevation.2` | 0 4 12 (16%) | Menu, bingkai |
| `elevation.3` | 0 8 24 (20%) + scrim 48% | Dialog, lembar bawah, toast |

### 3.4a Ritme dan bentuk situs publik (v1.3, CR-033)

Hanya `web/situs` `[KEPUTUSAN-358]`:
- Jarak vertikal seksi 96 (rapat 64) di desktop, 56 (rapat 48) di seluler; nilai > 64 hanya sebagai token situs, `space.*` §3.4 tidak berubah.
- Kontainer isi maksimum 1200 px, kolom baca 720 px; tepi 24 (desktop) dan 16 (seluler).
- Radius kartu situs 20, bingkai gambar dan panel besar 24 (pamflet "sudut membulat" `[UP-58]`); chip dan tombol situs `radius.full`.
- Bayangan hanya `elevation.1–3`; kartu naik 4 px + `elevation.2` saat hover, cincin fokus di kartu lewat `:focus-within` (§7.2 A8).
- Ritme latar: gradien `bg.app → mint` (hero, pita judul) → putih → `bg.app` → `primaryStrong` (seksi gelap Untuk Perusahaan, teks putih K7 7,53 dan mint 6,56) → footer `text.primary` (K13, judul kolom mint 12,81). Lapisan putih 8% di atas `primaryStrong` (kartu "kaca") memberi warna #1A6C66: teks putih 6,21 dan mint 5,41 (dihitung di `web/situs/src/kontras.test.ts`).
- Gerak: CSS saja, sekali saat dimuat (fade + geser 12 px), transisi 180 ms; semua gerak dan pergeseran dimatikan untuk `prefers-reduced-motion: reduce` `[SRC-880]`.
- Titik henti situs: < 768 satu kolom, 768–1279 dua kolom kartu, ≥ 1280 empat kolom kartu layanan/tiga kolom artikel; menu `details` di bawah 1024.
- Ikon situs: SVG sebaris buatan sendiri di `web/situs` (`[KEPUTUSAN-355]`, pengecualian lokasi dari §3.5 sampai dipindah ke paket UI).

### 3.5 Ikon, ilustrasi, dan logo

- **Ikon mobile**: kumpulan Material Icons bawaan Flutter (kelas `Icons`, dengan `uses-material-design: true` di `pubspec.yaml`) `[SRC-260]`, tanpa paket ikon tambahan `[KEPUTUSAN-166]`.
- **Ikon web**: SVG sebaris yang disimpan di `web/packages/ui` `[KEPUTUSAN-166]`. Sumber gambar SVG dan lisensinya `[TIDAK DIKETAHUI]`; dipilih dan dicatat di `docs/sources.md` saat Tahap 8. Emoji di wireframe hanya penanda posisi ikon.
- Ikon tanpa teks wajib punya label untuk pembaca layar. Ikon dekoratif disembunyikan dari pembaca layar.
- **Ilustrasi**: lingkaran mint dengan satu ikon (pola "state", §5). Ilustrasi bergambar orang seperti di pamflet tidak dipakai sampai ada aset berlisensi.
- **Logo**: simbol rumah dengan daun dari pamflet menjadi **arah** logo. Nama resmi aplikasi "Ruang Sehat" `[USER]` (CR-016, menggantikan T5-Q12 yang menetapkan "Rumah Sehat"). Nama ditulis "Ruang Sehat" dengan spasi di semua tempat; wordmark pamflet "RuangSehat" (tanpa spasi) tidak dipakai `[USER] PT-45 "dengan spasi"` (2026-11-18). Slogan pamflet "Satu ruang untuk kesehatan Anda." tidak disebut dalam jawaban itu, sehingga statusnya `[TIDAK DIKETAHUI]` dan tidak dipakai di wireframe sampai user menyatakan lain (Checkpoint 2, UP-60) `[DOK: input/Rumah Sehat Apps.jpeg]`. Logo bertulisan "Ruang Sehat" di wireframe hanya **placeholder** bertanda "draf logo", bukan desain final. Logo final dan ikon aplikasi dibuat desainer dan disetujui user `[UP-60]` (PT-40).

## 4. Komponen dasar dan state

Semua komponen tersedia di `rs_design` (Flutter) dan `web/packages/ui` (web) dengan nama sama. State hover hanya di web; di mobile digantikan state ditekan (pressed). Ukuran target: §7.2.

| Komponen | Varian | Default | Hover / ditekan | Fokus | Nonaktif | Galat | Memuat | Catatan aksesibilitas dan aturan |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Tombol | Utama, sekunder (garis), teks, destruktif | Utama: latar `primary`, teks putih (K5). Sekunder: putih, garis dan teks `primaryStrong` | Latar `primaryStrong` (K7); sekunder: latar mint | Cincin 2 px `primaryStrong` dengan jarak 2 px putih (N3) | Latar `bg.neutral`, teks `border.control`; dipakai hanya bila aksi memang tidak tersedia, disertai alasan tertulis | — | Pemutar + label tetap ("Memproses..."), tombol terkunci sampai respons (KU-05) | Label kata kerja ("Bayar Rp…", "Kirim untuk verifikasi"). Tombol utama **tidak** dinonaktifkan untuk validasi isian; menekan menampilkan galat per isian `[KEPUTUSAN-168]`. Satu tombol utama per layar |
| Tombol ikon | Standar, di atas media (video) | Ikon `text.primary` | Latar `bg.neutral` | Cincin fokus | Ikon `border.control` | — | Pemutar | Wajib label pembaca layar. Target ≥ 48 dp (mobile) |
| Isian teks | Satu baris, banyak baris, angka, password | Batas 1 px `border.control` (N1), label di atas | Batas `text.primary` | Cincin fokus | Latar `bg.neutral` | Batas 2 px `danger` + pesan teks di bawah dengan ikon "!" | — | Label selalu terlihat (bukan placeholder saja). Pesan galat berasal dari `errors[].message` respons API (ADR-023) dan menyebut cara memperbaiki (SC 3.3.1) `[SRC-256]`. Password: tombol tampilkan, paste diizinkan |
| Pilihan (select) | Dropdown, lembar pilihan (mobile) | Seperti isian | Seperti isian | Cincin fokus | Seperti isian | Seperti isian | Pemutar di dalam isian | Mobile: lembar bawah untuk daftar > 5 opsi |
| Checkbox | Tunggal, persetujuan | Kotak 24 di area sentuh 48, batas `primary` (N2) | Latar mint | Cincin fokus | Batas `border.control` | Batas `danger` + pesan | — | **Kotak persetujuan tidak pernah tercentang otomatis**; satu butir persetujuan satu kotak `[KEPUTUSAN-177]` (melengkapi US-104 AC3) |
| Radio | Daftar pilihan | Lingkaran 24, batas `primary` | Latar mint | Cincin fokus | `border.control` | Pesan di grup | — | Grup diberi judul |
| Kartu | Standar, mint (informasi) | `elevation.1` | Bila dapat ditekan: latar `bg.neutral` | Cincin fokus di seluruh kartu | — | — | Kerangka (skeleton) | Kartu yang dapat ditekan berlabel jelas |
| Ubin layanan | Beranda | Latar mint + ikon `primaryStrong` + label `text.primary` (K1) | Latar lebih gelap 1 tingkat | Cincin fokus | — | — | Kerangka | Dikenali dari label, bukan warna (N5) |
| Item daftar | Satu/dua baris, dengan aksi | Tinggi ≥ 56 | Latar `bg.neutral` | Cincin fokus | Teks `text.secondary` + alasan | — | Kerangka | — |
| Chip | Filter, pilihan slot | Garis `border.control` | Latar mint | Cincin fokus | Teks "penuh"/"tidak tersedia" + opasitas | — | — | Terpilih: latar `primaryStrong`, teks putih, tanda centang (bukan warna saja) |
| Badge status | Info, sukses, peringatan, bahaya, netral | §5.6 | — | — | — | — | — | Selalu ikon/simbol + teks nama status |
| Tab | Tab atas | Teks `text.secondary` | Latar `bg.neutral` | Cincin fokus | — | — | — | Terpilih: teks `primaryStrong` + garis bawah 3 px |
| Navigasi bawah (mobile) | 4 item | Ikon + label `text.secondary` | — | Cincin fokus | — | — | — | Terpilih: `primaryStrong` tebal + penanda "halaman saat ini" untuk pembaca layar. Struktur: `[UP-62]` |
| Navigasi samping (web) | Menu per peran | Teks `text.primary` | Latar `bg.neutral` | Cincin fokus | — | — | — | Terpilih: latar mint, teks `primaryStrong` (K6). Menu hanya memuat fitur yang diizinkan peran; otorisasi tetap di backend (ADR-012) |
| App bar / kepala web | — | Putih + garis pemisah | — | — | — | — | Bilah progres tipis | Judul layar = judul halaman (web: elemen judul dokumen) |
| Dialog / lembar bawah | Informasi, konfirmasi, destruktif | `elevation.3` | — | Fokus berpindah ke dialog, kembali ke pemicu saat ditutup | — | — | Tombol memuat | Aksi destruktif memakai tombol destruktif berlabel aksinya ("Hapus", "Tarik persetujuan") |
| Peringatan (alert) | Info, sukses, peringatan, bahaya | Latar status + garis kiri 4 px + ikon + judul tebal | — | — | — | — | — | Teks `text.primary` (K12). Diumumkan sebagai pesan status bila muncul setelah aksi (SC 4.1.3) `[SRC-256]` |
| Toast | Satu baris + aksi opsional | Latar `text.primary`, teks putih (K13) | — | Dapat dijangkau keyboard | — | — | — | Tidak menutupi elemen yang sedang difokus (SC 2.4.11) `[SRC-256]`; tidak dipakai untuk galat yang perlu tindakan |
| Rincian harga | Layanan, produk | Baris: harga layanan, ongkir (produk), pajak, biaya transaksi, total | — | — | — | — | Kerangka; tombol bayar terkunci sampai angka dari server tiba | Semua angka dari server (US-801 AC1). Margin tidak tampil sebagai baris (AB-13). Baris pajak mengikuti CR-013; label dan tarif menunggu PT-28 |
| Hitung mundur | Penahanan slot, sisa sesi | Teks waktu + jam berakhir | — | — | — | Berubah ke peringatan saat ≤ 5 menit | — | Selalu menulis jam berakhir, bukan hanya detik berjalan. Tidak mengumumkan setiap detik ke pembaca layar |
| Linimasa | Status pesanan | Titik + nama status + waktu | — | — | — | — | — | Status selesai/sekarang/akan datang dibedakan juga oleh teks |
| Unggah berkas | Foto (kamera/galeri), PDF | Tombol + pratinjau | — | Cincin fokus | — | Pesan jenis dan ukuran yang diizinkan (JPG/PNG ≤ 5 MB; PDF ≤ 10 MB) | Bilah progres | NFR-SEC-08. Pratinjau tidak disimpan ke galeri |
| Gelembung chat | Saya, lawan bicara | Saya: `primaryStrong` + teks putih (K7); lawan: putih + garis | — | — | — | "Belum terkirim, dicoba lagi otomatis" (teks) | — | Pesan masuk diumumkan sebagai pesan status |
| Kartu QR/kode | Appointment | QR + kode 8 karakter dikelompokkan 4-4 | — | — | "Tidak berlaku" + QR disamarkan | — | — | Kode juga tampil sebagai teks (KEPUTUSAN-107) |
| Tabel (web) | Data, dengan pilihan baris | Kepala `bg.neutral` | Baris: latar `bg.neutral` | Cincin fokus pada baris/aksi | — | — | Baris kerangka | Kepala kolom berlabel; angka rata kanan; gulir horizontal diizinkan untuk tabel lebar (§8) |
| Paginasi (web) | — | Teks jumlah + tombol | — | Cincin fokus | Tombol nonaktif di ujung | — | — | — |
| Kerangka (skeleton) | Baris, kartu | `bg.neutral` | — | — | — | — | — | Tanpa animasi berkedip; ada teks tersembunyi "Memuat" untuk pembaca layar |

## 5. Pola kondisi

Pola di bagian ini adalah keputusan desain penyusun `[KEPUTUSAN-167]`, kecuali yang menunjuk KU/AB/US PRD.

### 5.1 Memuat

- Muatan > 300 ms: kerangka sesuai bentuk isi. Aksi kirim: tombol memuat dan terkunci `[KEPUTUSAN-168]`.
- Pembayaran dan refund tidak memakai tampilan optimistis: status hanya berubah setelah server mengonfirmasi (US-802 AC2).
- Data rekanan yang lambat tidak memblokir bagian lain layar (KU-08, US-1406 AC3).

### 5.2 Kosong

Pola: ilustrasi lingkaran mint + judul + satu kalimat penjelasan + satu tombol aksi berikutnya. Contoh teks: "Belum ada riwayat konsultasi" + "Cari tenaga kesehatan"; "Tidak ada tenaga kesehatan tersedia" (US-303 AC3); "Home visit belum tersedia di alamat ini" (US-501 AC3). Teks final adalah pekerjaan T3 Tahap 8.

### 5.3 Galat

| Kondisi | Tampilan | Sumber |
| --- | --- | --- |
| Validasi | Galat per isian, isian benar tidak hilang, fokus pindah ke isian galat pertama | KU-01 `[SRC-256]` (SC 3.3.1) |
| Gagal jaringan / batas waktu | "Gagal terhubung" + tombol "Coba lagi"; percobaan ulang memakai kunci idempotensi yang sama | KU-02, KU-05 |
| Akses ditolak / data tidak ada | Satu pesan yang sama: "Data tidak ditemukan" (tidak membedakan "tidak ada" dan "tidak berhak") | KU-03 |
| Rekanan/pihak ketiga tidak tersedia | "Data sementara tidak tersedia" + waktu pembaruan terakhir bila ada; tombol pesan/bayar tidak tersedia | KU-08 |
| Rebutan slot/stok | "Slot baru saja dipesan orang lain. Anda belum dikenai biaya." | KU-09 |
| Akun nonaktif | Layar penuh "Akun nonaktif" + kontak layanan pelanggan | KU-10 |
| Galat server | Pesan umum + **kode referensi** (nilai `correlation_id` respons galat) untuk dilaporkan ke layanan pelanggan | ADR-023 `[KEPUTUSAN-172]` |

### 5.4 Luring (offline)

- Spanduk "Anda sedang offline" di atas layar. Kode/QR appointment yang pernah dibuka tetap tampil (US-403 AC2).
- Pesan chat yang belum terkirim berstatus "Belum terkirim" dan dikirim ulang otomatis selama sesi berjalan (US-305 AC3).

### 5.5 Waktu, sesi, dan kunci

| Kondisi | Pola | Sumber |
| --- | --- | --- |
| Penahanan slot/pembayaran (PB-07) | Spanduk info dengan jam berakhir, peringatan saat sisa ≤ 5 menit; saat habis: status Kedaluwarsa dan tombol "Pilih jadwal lagi" dengan penerima dan keluhan yang sudah diisi tetap terisi | US-304 AC4, US-802. Kesesuaian dengan SC 2.2.1: PT-43 |
| Sesi konsultasi | Sisa waktu di app bar; ruang chat menjadi baca-saja saat habis | US-305 AC4. Event waktu nyata termasuk pengecualian SC 2.2.1 `[SRC-256]` |
| Sesi web tidak aktif 30 menit | Dialog peringatan 2 menit sebelum berakhir dengan tombol "Tetap masuk" | NFR-SEC-04; SC 2.2.1 opsi "Extend" (peringatan dan minimal 20 detik untuk memperpanjang) `[SRC-256]` `[KEPUTUSAN-170]` |
| Sesi web mutlak 12 jam | Peringatan sebelum berakhir; perpanjangan tidak ditawarkan | ADR-011 butir 7. Kesesuaian dengan SC 2.2.1: PT-43 |
| Kunci aplikasi tenaga kesehatan (15 menit) | Layar "Aplikasi terkunci" + "Buka dengan biometrik atau PIN perangkat"; isian di memori tetap ada selama aplikasi tidak ditutup | NFR-SEC-10, CR-009; aturan data di perangkat `[DOK: coding-standards.md]` §1.6 |
| Aplikasi di latar belakang | Layar privasi polos (logo di atas `primaryStrong`) | ADR-021 butir 1 |

### 5.6 Status pesanan, klaim, refund, dan verifikasi

Nama status selalu nama PRD §3.6/AB-05; badge memberi nada warna + simbol `[KEPUTUSAN-169]`.

| Nada | Simbol | Status |
| --- | --- | --- |
| Info (`primaryStrong` di atas mint, K6) | ● | Terkonfirmasi, Dibayar, Ditugaskan, Berangkat, Tiba, Berlangsung, Check-in, Diproses outlet, Dikirim, Diproses (refund dan permintaan hak data), Antre (SATUSEHAT) |
| Sukses (`success` di atas mint, K10) | ✓ | Selesai, Diterima (produk), Disetujui, Berhasil (refund), Terkirim (SATUSEHAT), Klaim disetujui |
| Peringatan (`warning` di atas `warningBg`, K11) | ! | Menunggu pembayaran, Menunggu verifikasi, Diajukan (klaim), Klaim, Dilaporkan tidak datang, Keberatan diajukan, Menunggu kredensial, Menunggu penetapan penyebab, Perlu dicek (admin), "Sedang ditangani" (refund Gagal di sisi pasien, AB-32) |
| Bahaya (`danger` di atas `dangerBg`, K8) | ✕ | Ditolak, Dibatalkan rekanan, Dibatalkan sistem, Tenaga kesehatan tidak hadir, Pasien tidak hadir, Ditolak outlet, Gagal kirim, Tidak diserahkan, Klaim ditolak, Gagal (refund di backoffice, SATUSEHAT) |
| Netral (`text.secondary` di atas `bg.neutral`, K2) | ○ | Belum diunggah, Dibatalkan pasien, Kedaluwarsa, Ditutup tanpa konfirmasi, Draf |

Refund berstatus "Perlu dicek" tampil ke pasien sebagai "Diproses", karena hasil di penyedia belum diketahui (AB-35); berstatus "Gagal" tampil sebagai "Sedang ditangani" (AB-32) `[KEPUTUSAN-169]`.

## 6. Konten, format, dan notifikasi

- Bahasa Indonesia saja (NFR-LOC-01). Sapaan "Anda". Tombol memakai kata kerja. Istilah PRD dipakai apa adanya ("tenaga kesehatan", "pesanan", "rekanan"); di aplikasi pasien "refund" ditulis "pengembalian dana" `[KEPUTUSAN-171]`.
- Uang: `Rp150.000` (tanpa spasi, titik ribuan, tanpa desimal) (NFR-LOC-02) `[KEPUTUSAN-171]`.
- Tanggal: `Sen, 1 Mar 2027`; jam: `10.00` dengan label zona (`10.00 WITA`) untuk jadwal klinik/lab/home visit (AB-40); jadwal konsultasi dalam zona perangkat (AB-41); backoffice dalam WIB, portal dalam zona rekanan (AB-42) `[KEPUTUSAN-171]`.
- NIK dan nomor rekening: hanya 4 digit terakhir terlihat (`************0001`), kecuali NIK lengkap di panel Verifikator (§3.1, §7.3 PRD).
- Nomor pesanan/resep/permintaan sesuai format KEPUTUSAN-107; kode check-in ditampilkan dalam kelompok 4-4.
- **Notifikasi di aplikasi dan email** tidak memuat diagnosis, nama obat, nilai hasil lab, atau isi chat (AB-73). Kotak notifikasi menandai "Baru" dengan teks (US-1001 AC4).
- **Email**: satu kolom, lebar baca nyaman, CSS inline, logo sebagai teks (tanpa gambar jarak jauh dan tanpa piksel pelacak), tombol tautan "Buka aplikasi Ruang Sehat", kaki email berisi pernyataan bahwa Ruang Sehat berperan sebagai platform dan kontak layanan pelanggan `[KEPUTUSAN-175]`. Isi per kejadian mengikuti matriks EP-10.

## 7. Aksesibilitas

### 7.1 Target

Semua permukaan menargetkan **WCAG 2.2 Level AA** `[KEPUTUSAN-160]` `[SRC-256]`. PRD §6.11 menyerahkan standar ini ke Tahap 6. Level ini dipilih karena kriteria AA mencakup kebutuhan yang langsung relevan untuk pasien yang kesulitan datang ke fasilitas (kontras, ukuran target, teks yang dapat diperbesar, autentikasi yang tidak membebani ingatan), sedangkan Level AAA tidak diwajibkan. Target ini akan dikonfirmasi user di Checkpoint 2.

### 7.2 Aturan

| # | Aturan | Nilai yang diterapkan | Sumber |
| --- | --- | --- | --- |
| A1 | Warna bukan satu-satunya penanda | Status, galat, item terpilih, nilai di luar rujukan diberi teks atau simbol | SC 1.4.1 (A) `[SRC-256]` |
| A2 | Kontras teks | ≥ 4,5:1 untuk semua teks (§3.2) | SC 1.4.3 (AA) `[SRC-256]` `[KEPUTUSAN-161]` |
| A3 | Kontras non-teks | ≥ 3:1 untuk batas kontrol, ikon bermakna, cincin fokus | SC 1.4.11 (AA) `[SRC-256]` |
| A4 | Teks dapat diperbesar | Web: dapat diperbesar 200% tanpa kehilangan isi. Mobile: tata letak diuji pada skala teks 200% dan tetap dapat dipakai | SC 1.4.4 (AA) `[SRC-256]`; Flutter: "The UI should remain legible and usable at very large scale factors" `[SRC-257]` `[KEPUTUSAN-162]` |
| A5 | Reflow | Web dapat dipakai pada lebar setara 320 CSS px tanpa gulir dua arah, kecuali tabel data | SC 1.4.10 (AA) `[SRC-256]` |
| A6 | Spasi teks | Tidak ada isi terpotong saat tinggi baris 1,5x, spasi paragraf 2x, spasi huruf 0,12x, spasi kata 0,16x | SC 1.4.12 (AA) `[SRC-256]` |
| A7 | Keyboard (web) | Semua fungsi dapat dijalankan dengan keyboard; urutan fokus mengikuti urutan baca | SC 2.1.1 (A) `[SRC-256]` |
| A8 | Fokus terlihat dan tidak tertutup | Cincin 2 px `primaryStrong` + jarak 2 px; kepala lengket, toast, dan dialog tidak menutupi elemen yang difokus | SC 2.4.7 (AA), SC 2.4.11 (AA) `[SRC-256]` |
| A9 | Ukuran target web | Minimum 24×24 CSS px (atau jarak memadai); standar komponen web 40 px tinggi | SC 2.5.8 (AA) `[SRC-256]` `[KEPUTUSAN-162]` |
| A10 | Ukuran target mobile | 48×48 dp di Android dan iOS (memenuhi pedoman iOS 44×44) | Flutter: "All tappable targets should be at least 48x48 pixels"; `androidTapTargetGuideline` 48×48, `iOSTapTargetGuideline` 44×44 `[SRC-257]` `[SRC-258]` `[KEPUTUSAN-162]` |
| A11 | Label | Setiap kontrol punya label yang dibaca pembaca layar (TalkBack, VoiceOver, pembaca layar web) | Flutter `labeledTapTargetGuideline`, uji TalkBack dan VoiceOver `[SRC-257]` `[SRC-258]` |
| A12 | Identifikasi galat | Isian galat ditandai dan galat dijelaskan dengan teks | SC 3.3.1 (A) `[SRC-256]` |
| A13 | Isian berulang | Alamat, penerima layanan, dan data anggota keluarga dipilih dari data tersimpan, tidak diketik ulang dalam satu proses | SC 3.3.7 (A) `[SRC-256]` |
| A14 | Autentikasi yang aksesibel | Password dan kode TOTP boleh ditempel dan diisi pengelola password; tanpa CAPTCHA teka-teki; tidak ada larangan paste | SC 3.3.8 (AA): langkah autentikasi tidak boleh menuntut tes fungsi kognitif kecuali tersedia mekanisme bantuan `[SRC-256]` |
| A15 | Pesan status | Hasil aksi, pesan chat masuk, perubahan antrean diumumkan tanpa memindah fokus | SC 4.1.3 (AA) `[SRC-256]` |
| A16 | Batas waktu | Lihat §5.5 | SC 2.2.1 (A) `[SRC-256]` |
| A17 | Gambar bermakna | Foto dokumen dan QR punya teks alternatif; nilai kode check-in juga tampil sebagai teks | `[KEPUTUSAN-162]` |

Mekanisme teknis per platform (properti semantik Flutter, atribut ARIA dan `autocomplete` web, pengumuman pembaca layar) dipilih dan diverifikasi ke dokumentasi resmi di Tahap 8 `[KEPUTUSAN-176]`.

### 7.3 Data sensitif dan pembaca layar

Pembaca layar membacakan isi yang tampil. Karena itu layar yang memuat data klinis tidak menampilkan isi klinis di pratinjau notifikasi atau ringkasan daftar (AB-73), dan NIK di daftar sudah termasking sebelum dibacakan.

### 7.4 Verifikasi otomatis (Tahap 8)

`[KEPUTUSAN-176]`
- Mobile: test widget per layar utama memanggil `androidTapTargetGuideline`, `iOSTapTargetGuideline`, `labeledTapTargetGuideline`, dan `textContrastGuideline` dari `flutter_test` (bagian SDK, bukan dependensi baru) `[SRC-258]`. Masuk ke `flutter test` di `scripts/verify.sh`.
- Token: skrip kecil tanpa dependensi menghitung ulang semua pasangan K1–K13 dan N1–N4 dengan rumus §3.2 dan gagal bila di bawah syarat. Tempat skrip ditetapkan Tahap 8.
- Web: test Playwright (sudah di stack) untuk navigasi keyboard dan fokus pada alur utama web. Pemeriksa aksesibilitas otomatis tambahan hanya lewat CR dengan verifikasi registry.
- Uji manual dengan TalkBack dan VoiceOver pada alur A01, A05, A07 sebelum UAT `[SRC-257]`.

## 8. Responsivitas

| Permukaan | Aturan | Sumber |
| --- | --- | --- |
| Mobile | Rancangan dasar lebar 360 dp; tata letak harus utuh pada lebar 320–480 dp. Satu kolom. Tombol utama di bilah bawah tetap terlihat saat papan ketik terbuka. Orientasi tidak dikunci; perilaku lanskap diuji di Tahap 9 | `[KEPUTUSAN-174]` |
| Web ≥ 1280 px | Kepala 56 px + navigasi samping 240 px + isi; pola daftar + panel detail berdampingan | `[KEPUTUSAN-174]` |
| Web 768–1279 px | Navigasi samping dapat diciutkan; panel detail di bawah daftar | `[KEPUTUSAN-174]` |
| Web < 768 px sampai 320 px | Navigasi menjadi tombol menu; satu kolom; tabel data boleh gulir horizontal (konten dua dimensi dikecualikan SC 1.4.10) | `[SRC-256]` `[KEPUTUSAN-174]` |
| Tema login Keycloak dan halaman verifikasi resep | Satu kolom, lebar maksimum 400 px, utuh pada 320 px | `[KEPUTUSAN-175]` |

## 9. Aturan UI untuk modul T1

| Area | Aturan | Sumber |
| --- | --- | --- |
| Autentikasi | Masuk selalu di browser sistem dengan tema Ruang Sehat, bukan webview. Pesan galat masuk dan atur ulang password bersifat umum. Tidak ada tombol Google/Apple | ADR-011 butir 4, 7; US-103; CR-007 |
| Pendaftaran pasien | Formulir native dua langkah; akun dibuat hanya setelah persetujuan (A01) | ADR-011 butir 6; US-101 AC4 |
| MFA (semua populasi: pasien, tenaga kesehatan, admin, RME) | Layar kode TOTP + tautan kode pemulihan; aktivasi pertama menampilkan QR dan kunci teks, lalu 12 kode pemulihan sekali tampil. Kode boleh ditempel (aturan §7.2 A14). Pasien: alur A15 (D-A15) | ADR-011 butir 5 revisi Checkpoint 2 `[USER] C2-Q9`; US-101 AC5, US-103 AC1, AC7, AC8 |
| Pesan login gagal | Satu pesan seragam untuk password salah, akun tidak ada, terkunci, dan nonaktif, dengan petunjuk tetap "Bila tidak dapat masuk, gunakan Lupa password atau hubungi Layanan Pelanggan"; tanpa pesan khusus terkunci | US-103 AC2 `[USER] C2-B4` (UP-73) |
| Autentikasi ulang | Mengajukan hak data, mengunduh salinan data, mengubah email/password, mendaftarkan ulang TOTP, mengakhiri sesi dari daftar sesi: diberi tahu lebih dulu bahwa login akan diminta lagi | ADR-011 butir 9; US-1602 AC1; US-1604 AC2; US-103 AC6, AC7; US-110 AC2 |
| Persetujuan data | Tujuh informasi Pasal 21 ayat (1) UU PDP tampil sebagai bagian yang dapat dibuka; versi teks tampil; kotak tidak tercentang otomatis; penarikan menampilkan akibat dan pesanan terdampak sebelum konfirmasi | US-104, US-1601 `[SRC-15]` |
| Anggota keluarga | Alur KTP/KK ditentukan sistem dari tanggal lahir; pernyataan/persetujuan orang tua wajib dicentang | US-108 AC1–AC3, AC6 |
| Identitas | NIK hanya angka; foto JPG/PNG ≤ 5 MB; status AB-05 selalu terlihat; pemesanan layanan berbatas dihentikan sebelum pembayaran dengan tautan ke langkah berikutnya | US-106, US-107 |
| Harga dan pembayaran | Angka hanya dari server; perubahan harga/ketersediaan ditampilkan sebagai dialog yang wajib disetujui; tombol bayar terkunci saat diproses; konfirmasi pesanan hanya dari notifikasi server | US-801, US-802, AB-15, AB-17, KU-05 |
| Refund dan klaim | Nominal pengembalian tampil sebelum konfirmasi batal; penolakan klaim wajib alasan; status refund dengan teks | US-805, US-807, US-808 |
| Data klinis di perangkat | Tidak disimpan persisten selain berkas yang sengaja diunduh dan kode appointment | `[DOK: coding-standards.md]` §1.6 |
| Backoffice | Foto dokumen hanya di panel Verifikator dan pembukaannya dicatat (E-05); tombol ekspor hanya untuk peran berhak (US-1503 AC2) | §3.1 PRD, US-201 AC6 |

## 10. Alur utama dan peta wireframe

Alur utama diturunkan dari epic PRD §5. Setiap alur punya wireframe HTML (`docs/wireframes/`) atau deskripsi terstruktur (§10.2). Kolom "Jenis": HTML atau Deskripsi. HTML dipakai untuk semua alur aplikasi pasien yang memuat pemesanan, pembayaran, identitas, atau persetujuan, ketiga alur tenaga kesehatan, dan alur web yang menyentuh modul T1; alur lain cukup deskripsi `[KEPUTUSAN-173]`.

### 10.1 Daftar alur

| ID | Alur | Epic / story | Permukaan | Jenis | Wireframe |
| --- | --- | --- | --- | --- | --- |
| A01 | Registrasi, persetujuan data, verifikasi email, masuk, atur ulang password, keluar | EP-01 US-101, US-103, US-104 | Pasien + tema login | HTML | `a01-registrasi-masuk.html` |
| A02 | Beranda (masuk dan tamu), navigasi, kotak notifikasi, konten | EP-10 US-1001; EP-11 US-1102; US-107 | Pasien | HTML | `a02-beranda-notifikasi.html` |
| A03 | Profil, data diri, verifikasi KTP, status identitas | EP-01 US-105, US-106, US-107 | Pasien | HTML | `a03-profil-identitas.html` |
| A04 | Tambah, ubah, hapus anggota keluarga | EP-01 US-108, US-109 | Pasien | HTML | `a04-anggota-keluarga.html` |
| A05 | Cari tenaga kesehatan dan pesan konsultasi | EP-03 US-303, US-304 | Pasien | HTML | `a05-pesan-konsultasi.html` |
| A06 | Ruang sesi, chat, video, ketidakhadiran, catatan, resep, riwayat | EP-03 US-305–US-310 | Pasien | HTML | `a06-sesi-konsultasi.html` |
| A07 | Ringkasan harga, pembayaran, hasil, bukti bayar | EP-08 US-801, US-802, US-804 | Pasien | HTML | `a07-pembayaran.html` |
| A08 | Pendaftaran klinik/lab, appointment, antrean, hasil lab, daftar pesanan | EP-04 US-401–US-406; EP-09 US-902; EP-07 US-706 | Pasien | HTML | `a08-klinik-lab.html` |
| A09 | Informasi biaya MCU/lab dan lanjut daftar | EP-06 US-601, US-602 | Pasien | HTML | `a09-info-biaya.html` |
| A10 | Pesan, pantau, lapor tidak datang, keberatan home visit | EP-05 US-501, US-504; US-807 | Pasien | HTML | `a10-home-visit.html` |
| A11 | Katalog, detail, keranjang, checkout, pelacakan, terima, klaim barang | EP-07 US-701–US-706 | Pasien | HTML | `a11-belanja-produk.html` |
| A12 | Pembatalan layanan terjadwal, klaim, status refund | EP-08 US-805–US-808 | Pasien | HTML | `a12-batal-klaim-refund.html` |
| A13 | Persetujuan, penarikan, hak subjek data, salinan, versi baru persetujuan | EP-16 US-1601, US-1602, US-1604; US-104 AC4 | Pasien | HTML | `a13-privasi-hak-data.html` |
| A14 | Ajukan dan pantau keluhan | EP-11 US-1103 | Pasien | Deskripsi | §10.2 D-A14 |
| A15 | Pendaftaran TOTP pasien, kode pemulihan, masuk dengan kode, ubah password, daftar ulang TOTP, perangkat hilang (Checkpoint 2) | EP-01 US-101 AC5, US-103 AC1, AC1b, AC6–AC8 | Pasien + tema login | Deskripsi | §10.2 D-A15 |
| G01 | Perangkat dan sesi aktif (Checkpoint 2) | EP-01 US-110; NFR-SEC-11 | Semua aplikasi (pasien, tenaga kesehatan, ketiga web) | Deskripsi | §10.2 D-G01 |
| B01 | Pendaftaran tenaga kesehatan, status verifikasi, masuk MFA, kunci aplikasi, layar privasi | EP-02 US-202, US-204; NFR-SEC-10 | Tenaga kesehatan | HTML | `b01-nakes-akun-kunci.html` |
| B02 | Jadwal praktik, daftar sesi, ruang sesi, catatan, resep, pembatalan | EP-03 US-302, US-305–US-309, US-311; EP-10 US-1003 | Tenaga kesehatan | HTML | `b02-nakes-sesi-jadwal.html` |
| B03 | Tugas dan status home visit | EP-05 US-503 | Tenaga kesehatan | HTML | `b03-nakes-home-visit.html` |
| C01 | Masuk web dengan MFA, batas sesi, akses ditolak | EP-02 US-206, US-208; US-1302; NFR-SEC-02, NFR-SEC-04; KU-03, KU-10 | Ketiga web | HTML | `c01-web-masuk-sesi.html` |
| C02 | Verifikasi identitas dan tenaga kesehatan | EP-02 US-201, US-203 | Backoffice | HTML | `c02-bo-verifikasi.html` |
| C03 | Keputusan klaim, antrean Keuangan, laporan transaksi | EP-08 US-807, US-808, US-811, US-802 AC3 | Backoffice | HTML | `c03-bo-klaim-keuangan.html` |
| C04 | Onboarding rekanan dan akun admin mitra | EP-02 US-205, US-206 | Backoffice | Deskripsi | D-C04 |
| C05 | Kelola akun: nonaktif/aktif, admin RS dan sub-peran | EP-02 US-207, US-208 | Backoffice | Deskripsi | D-C05 |
| C06 | Margin, biaya rekanan, parameter bisnis | EP-08 US-809, US-810, US-812 | Backoffice | Deskripsi | D-C06 |
| C07 | Konten (CMS) | EP-11 US-1101 | Backoffice | Deskripsi | D-C07 |
| C08 | Kelola tiket keluhan | EP-11 US-1104 | Backoffice | Deskripsi | D-C08 |
| C09 | Pemantauan integrasi rekanan | EP-14 US-1405 | Backoffice | Deskripsi | D-C09 |
| C10 | Audit log dan ekspor | EP-15 US-1502, US-1503 | Backoffice | Deskripsi | D-C10 |
| C11 | Proses hak subjek data (Petugas PDP) | EP-16 US-1603, US-1605 | Backoffice | Deskripsi | D-C11 |
| C12 | Dasbor KPI | EP-17 US-1701 | Backoffice | Deskripsi | D-C12 |
| C13 | Antrean verifikasi perusahaan, buat perusahaan, daftar perusahaan dan anggota HR (CR-021) | EP-19 US-1902, US-1903, US-1916 | Backoffice | HTML | `c13-bo-verifikasi-perusahaan.html`, D-C13 |
| C14 | CMS kanal situs: daftar, editor Markdown, pratinjau aman (CR-021) | EP-18 US-1802; EP-11 US-1101 | Backoffice | HTML | `c14-bo-cms-situs.html`, D-C14 |
| K01 | Member area HR: masuk, daftar perusahaan, TOTP, menunggu verifikasi, ditolak, dasbor, kelola HR, sesi (CR-021) | EP-19 US-1901, US-1904; US-1902 AC3 | Web korporat | HTML | `k01-korporat-hr.html`, D-K01 |
| S01 | Situs publik sembilan halaman, tidak ditemukan dan galat umum (CR-021) | EP-18 US-1801 | Situs publik | HTML | `s01-situs-publik.html`, D-S01 |
| D01 | Laporan transaksi dan pengaturan konsultasi online | EP-12 US-1201; EP-03 US-301, US-302 AC4 | Portal rekanan | HTML | `d01-portal-rekanan.html` |
| D02 | Lihat profil rekanan dan keluhan terkait | EP-12 US-1202, US-1203 | Portal rekanan | Deskripsi | D-D02 |
| E01 | Appointment, check-in, antrean, rekam medis kunjungan, status SATUSEHAT | EP-13 US-1304, US-1305, US-1306, US-1309 | Web RME | HTML | `e01-rme-kunjungan.html` |
| E02 | Onboarding fasyankes dan pengguna RME | EP-13 US-1301, US-1302 | Web RME | Deskripsi | D-E02 |
| E03 | Jadwal, layanan, dan harga | EP-13 US-1303 | Web RME | Deskripsi | D-E03 |
| E04 | Input hasil lab | EP-13 US-1307 | Web RME | Deskripsi | D-E04 |
| E05 | Penugasan home visit | EP-13 US-1308; EP-05 US-502 | Web RME | Deskripsi | D-E05 |
| E06 | Riwayat rekam medis dan log akses | EP-13 US-1310 | Web RME | Deskripsi | D-E06 |
| F01 | Halaman verifikasi resep publik | EP-03 US-309 AC6–AC8 (CR-008) | Web publik | Deskripsi | D-F01 |
| F02 | Template email notifikasi | EP-10 US-1001, US-1002, US-1003 | Email | Deskripsi | §6, D-F02 |

Ringkasan (diperbarui v1.2: S01, K01, C13, C14 menambah empat alur HTML; deskripsi D-S01, D-K01, D-C13, D-C14 menyertai wireframe-nya, jadi total 25 HTML dan 20 deskripsi mandiri): 21 alur dengan wireframe HTML (semua alur aplikasi pasien kecuali keluhan, ketiga alur tenaga kesehatan, dan alur kunci web), 20 alur dengan deskripsi terstruktur (18 dari Tahap 6, ditambah A15 dan G01 dari Checkpoint 2). A15 menyentuh modul T1 sehingga menurut KEPUTUSAN-173 semestinya berwireframe HTML. Pada tindak lanjut ini A15 sengaja hanya dideskripsikan (instruksi orkestrator: tanpa wireframe HTML baru). Wireframe A01 belum menampilkan langkah TOTP; D-A15 melengkapinya sampai wireframe diperbarui di pekerjaan UI Tahap 8 (UI-03).

### 10.2 Deskripsi terstruktur

Format: tujuan · aktor · layar berurutan · kondisi khusus · rujukan. Semua memakai komponen §4 dan pola §5; web memakai tata letak "daftar + panel detail" seperti C02/C03.

**D-A14 Keluhan pasien** · Pasien · (1) Profil → Bantuan dan keluhan: daftar tiket (nomor, kategori, status Baru/Diproses/Menunggu pasien/Selesai); (2) Formulir: kategori (layanan, pembayaran, akun, lainnya), pesanan terkait opsional, deskripsi, foto opsional → nomor tiket tampil; (3) Detail tiket: utas balasan + kotak balas. · Memilih kategori klaim refund atau hak data mengarahkan ke A12/A13 (US-1103 AC3). Tiket yang ditutup dibuka lagi bila dibalas dalam 7 hari (US-1104 AC3). Kosong: "Belum ada keluhan". · US-1103.

**D-A15 MFA pasien (Checkpoint 2)** · Pasien · Layar 1–4 berada di tema login Keycloak (browser sistem); layar 5–8 di aplikasi pasien.
1. Pendaftaran TOTP saat masuk pertama setelah email terverifikasi: judul "Lindungi akun Anda dengan kode masuk", penjelasan singkat bahwa aplikasi authenticator diperlukan, QR, kunci teks yang dapat disalin, dan isian kode 6 digit.
2. Kode pemulihan: 12 kode dalam daftar bernomor, tombol "Salin semua", peringatan bahwa kode hanya tampil sekali dan setiap kode hanya berlaku sekali, dan centang "Saya sudah menyimpan kode" sebelum lanjut.
3. Masuk berikutnya: setelah password benar, layar kode TOTP dengan tautan "Pakai kode pemulihan".
4. Petunjuk perangkat hilang di layar kode: "Kehilangan ponsel? Pakai kode pemulihan, atau hubungi Layanan Pelanggan".
5. Profil → Keamanan akun: "Ubah password" (password lama, password baru, opsi "Keluar dari perangkat lain" tercentang).
6. Profil → Keamanan akun: "Daftar ulang kode masuk" (QR dan kode baru, kode pemulihan baru, opsi keluar dari perangkat lain tercentang).
7. Setelah masuk dengan kode pemulihan: spanduk anjuran mendaftarkan ulang kode masuk.
8. Profil → Keamanan akun: "Perangkat dan sesi aktif" (D-G01).

Kondisi khusus:
- Kode salah memakai pesan umum per isian (KU-01).
- Menutup layar 1 atau 2 tidak membuat pasien masuk ke beranda.
- Aksi di layar 5 dan 6 didahului pemberitahuan bahwa login akan diminta lagi (§9).
- Pembaca layar membaca kunci teks per kelompok 4 karakter. Kode pemulihan tidak dimasukkan ke notifikasi atau tangkapan layar privasi (§5.5).
- Teks final adalah pekerjaan T3 Tahap 8.

Rujukan: US-101 AC5, US-103 AC1, AC1b, AC6–AC8, ADR-011 butir 5, KEPUTUSAN-202–KEPUTUSAN-204.

**D-G01 Perangkat dan sesi aktif (Checkpoint 2)** · Pasien, tenaga kesehatan, admin Ruang Sehat, admin rekanan, pengguna RME · Mobile di Profil → Keamanan akun; web di menu akun (kepala halaman).
1. Daftar sesi berupa item daftar dua baris: nama aplikasi, alamat IP, "Mulai Sen, 1 Mar 2027 10.00 WIB", "Terakhir aktif …". Sesi yang sedang dipakai di urutan teratas dengan badge info bertulisan "Sesi ini" (simbol + teks, bukan warna saja).
2. Tombol teks "Akhiri" per sesi.
3. Tombol sekunder "Akhiri semua sesi lain".
4. Dialog konfirmasi destruktif: menyebut sesi yang diakhiri dan bahwa login akan diminta lagi.
5. Setelah login ulang: daftar diperbarui dan pesan status "Sesi diakhiri" diumumkan (aturan §7.2 A15).

Kondisi khusus:
- Nama perangkat hanya tampil bila tersedia (KEPUTUSAN-205).
- Kosong (hanya "Sesi ini"): teks "Tidak ada sesi lain".
- Galat jaringan mengikuti KU-02.
- Mengakhiri "Sesi ini" sama dengan Keluar.
- Web: tabel atau daftar mengikuti §8 responsivitas.
- Tidak ada batas jumlah sesi, sehingga tidak ada pesan "batas sesi tercapai" (NFR-SEC-11).

Rujukan: US-110, ADR-011 butir 12, KEPUTUSAN-205.

**D-C04 Onboarding rekanan dan admin mitra** · Verifikator · (1) Daftar rekanan dengan status (Menunggu verifikasi, Aktif, Nonaktif) dan tiga syarat tampil (dokumen disetujui, margin diatur, integrasi Aktif) sebagai daftar centang bertulisan; (2) Formulir rekanan: nama, jenis (klinik, lab, toko kecantikan, toko alat kesehatan, apotek; klinik dapat sekaligus outlet), alamat dan zona waktu, dokumen izin, rekening penerima (termasking setelah disimpan), sumber sistem; (3) Panel akun admin mitra: undang lewat email. · Perubahan rekening hanya berlaku setelah disetujui Verifikator (US-1202 AC3). Daftar dokumen wajib menunggu L-02/L-03. · US-205, US-206.

**D-C05 Kelola akun** · Super Admin, Verifikator, Layanan Pelanggan sesuai hak · (1) Cari akun (pasien, tenaga kesehatan, rekanan, admin); (2) Detail akun + tombol Nonaktifkan/Aktifkan dengan alasan wajib; (3) Admin RS: centang sub-peran; (4) detail akun pasien (hanya Layanan Pelanggan): tombol "Reset kode masuk (MFA)" dengan alasan wajib → status "Menunggu konfirmasi pasien lewat email" sampai pasien membuka tautan dan masuk dengan password; tidak ada kolom untuk melihat atau mengisi kredensial (US-207 AC5, Checkpoint 2). · Ditolak dengan pesan bila pasien punya pesanan aktif (US-207 AC3), bila mencabut Super Admin terakhir (US-208 AC2), mengubah peran sendiri (AC3), menonaktifkan akun sendiri (AC5), menggabungkan Auditor dengan sub-peran lain (AC6), atau mencabut Auditor terakhir (AC7). Dialog menyebut dampak penonaktifan rekanan/tenaga kesehatan ke pesanan (US-207 AC2). · US-207, US-208.

**D-C06 Margin, biaya rekanan, parameter** · Keuangan (margin, biaya), Super Admin (parameter) · (1) Pilih rekanan → riwayat margin (jenis, nilai, berlaku mulai) + tombol "Atur margin"; (2) Formulir: jenis persen/nominal, nilai, berlaku segera atau tanggal mendatang, ringkasan konfirmasi; (3) Biaya rekanan dengan pola sama; (4) Parameter PB-01–PB-10, PB-14–PB-20: tabel nilai, satuan, riwayat. · Nilai tidak valid ditolak per isian (US-812 AC3). Teks penjelas bahwa perubahan tidak memengaruhi pesanan yang pembayarannya sudah dibuat (AB-17). Tarif pajak menjadi parameter Keuangan setelah CR-013 diterapkan. · US-809, US-810, US-812.

**D-C07 Konten** · Konten · (1) Daftar konten (banner, halaman info, artikel, promo) dengan status Draf/Tayang/Selesai/Diarsipkan; (2) Editor: judul, isi, gambar, kategori, jadwal tayang, pratinjau seluler; (3) Hapus dan pulihkan. · Teks bantu "Promo hanya informasi, tidak mengubah harga" (US-1101 AC4). Gambar wajib teks alternatif. · US-1101.

**D-C08 Tiket keluhan** · Layanan Pelanggan · (1) Antrean tiket urut terlama dengan filter status; (2) Detail: utas, ubah status, teruskan ke rekanan, catat tanggapan rekanan. · Tiket yang diteruskan tidak memuat data klinis (US-1104 AC2). · US-1104.

**D-C09 Pemantauan integrasi** · Admin RS · (1) Tabel rekanan: status integrasi (Belum, Uji, Aktif, Terganggu, Nonaktif) dengan badge bertulisan, sinkron terakhir berhasil, jumlah galat 24 jam; (2) Detail + ubah status dengan alasan. · Rekanan Terganggu ditandai peringatan di atas tabel (US-1405 AC2). · US-1405.

**D-C10 Audit log** · Auditor, Super Admin · (1) Filter: rentang waktu, aktor, peran, jenis event, jenis dan ID objek, subjek data, hasil, alamat IP; (2) Tabel berhalaman urut terbaru; (3) Panel detail semua field §7.2 termasking; (4) Ekspor CSV hanya untuk Auditor. · Tidak ada tombol ubah/hapus di mana pun (US-1504 AC1). · US-1502, US-1503.

**D-C11 Hak subjek data** · Petugas PDP · (1) Antrean dengan kolom sisa waktu menuju 3×24 jam dan badge "Mendesak" saat tinggal 12 jam; (2) Detail: jenis, subjek, tahap; isian tanggapan dan alasan; aksi sesuai jenis (perbaikan, pembatasan, salinan otomatis). · NIK termasking; isi klinis tidak dibuka petugas (salinan dibuat sistem). Penghapusan (US-1605) terblokir sampai L-06/PT-24: tombol tampil nonaktif dengan alasan tertulis. · US-1603, US-1605.

**D-C12 Dasbor KPI** · Super Admin · (1) Pilih bulan (WIB); (2) Kartu KPI-01 (dan angka pendukung), KPI-02, KPI-03, KPI-04 dengan angka dan label teks; grafik opsional selalu disertai tabel angka; (3) Ekspor CSV agregat. · US-1701.

**D-S01 Situs publik (CR-021, F1-T83)** · Pengunjung anonim · Wireframe `s01-situs-publik.html`. Sembilan halaman: Beranda, Tentang Kami, Layanan, Untuk Perusahaan, Rekanan, Artikel/berita (daftar, detail, kosong), Kontak, Unduh aplikasi, Kebijakan privasi dan syarat (BR-130 `[USER] C21-P1-1`), ditambah "tidak ditemukan" dan galat umum.
1. Kerangka bersama: tautan "Lewati ke isi", kepala dengan logo placeholder dan navigasi delapan item (Kebijakan privasi dan syarat di kaki), isi, kaki dengan pernyataan peran platform. Navigasi berupa deretan tautan di header; di bawah 1140 px menjadi baris kedua yang dapat digulir mendatar, target 44 px (CR-038, KEPUTUSAN-420/421, menggantikan menu `details`). Pojok kanan: tautan Google Play dan App Store (KEPUTUSAN-422–424).
2. Beranda: hero (judul, kalimat pembuka, dua tombol: Unduh aplikasi, Untuk Perusahaan), enam ubin layanan (UP-63), blok Untuk Perusahaan, tiga artikel terbaru. Layanan: kartu per layanan ditambah layanan korporat, isi dan urutan dari CMS (US-1801 AC2). Untuk Perusahaan: uraian dari CMS dan dua jalur "Daftarkan perusahaan" dan "Masuk member area" ke `web/korporat` (aplikasi dan host terpisah).
3. Rekanan: kartu logo dan nama yang ditulis di CMS (bukan diambil otomatis dari data rekanan). Kontak: info kontak dan alamat teks, tanpa formulir dan tanpa peta sematan `[KEPUTUSAN-271]`. Unduh aplikasi: tombol Android dan iOS; sebelum terbit di toko tampil "Aplikasi segera hadir" tanpa tautan palsu. Kebijakan privasi dan syarat: spanduk draf, versi dan tanggal berlaku di kepala (US-1801 AC4).

Kondisi khusus:
- Semua konten Markdown dirender aman dan hanya status Tayang tampil. URL konten Draf, terjadwal, berakhir, diarsipkan, atau terhapus menampilkan "Halaman tidak ditemukan" yang identik (AC3).
- Galat layanan konten: halaman galat umum tanpa jejak teknis dan tanpa kode referensi `[KEPUTUSAN-272]` (AC7). Kosong: Artikel "Belum ada artikel"; Rekanan "Daftar rekanan segera hadir".
- Tanpa cookie sesi, tanpa spanduk persetujuan cookie (tidak ada cookie), tanpa skrip, font, atau sematan pihak ketiga (US-1801 AC5, US-1802 AC9).
- Aksesibilitas §7.2: kontras token §3.2 (A2, A3), judul bertingkat per halaman, landmark `header`, `nav`, `main`, `footer` berlabel, `lang="id"`, target ≥ 40 px (desktop) dan 48 px (sempit), fokus terlihat, reflow 320 px, teks dapat diperbesar 200%, gambar bermakna wajib teks alternatif dari CMS (A1–A9, A17). Verifikasi otomatis di F1-T97 (US-1801 AC6).
- `[TIDAK DIKETAHUI]` di wireframe: teks Tentang Kami, visi dan misi, uraian layanan dan Untuk Perusahaan, nama rekanan yang boleh tampil dan izin logonya, kanal kontak dan alamat, tautan toko aplikasi, versi dan tanggal berlaku teks hukum (L-08, L-20, PT-24), logo dan gambar (PT-40), slogan (UP-60), domain (PT-46, PT-70).

Rujukan: US-1801, US-1802 AC9, BR-130, BR-131, ADR-027, KEPUTUSAN-270–272.

**D-K01 Member area HR (CR-021, F1-T84)** · HR perusahaan · Wireframe `k01-korporat-hr.html`. Layar 1 dan 6 di tema login Keycloak, lainnya di `web/korporat`.
1. Masuk: email dan password, lalu kode TOTP dengan tautan "Pakai kode pemulihan"; satu pesan galat seragam (UP-73, §9); petunjuk kehilangan TOTP mengarah ke Layanan Pelanggan (reset oleh Verifikator di Fase 2, US-1904 AC8).
2. Daftar tiga langkah: (a) data perusahaan: nama badan usaha, NIB, NPWP, alamat kantor, telepon kantor; (b) data HR: nama, jabatan, email, password; (c) unggah dokumen NIB, NPWP, bukti kewenangan (JPG/PNG ≤ 5 MiB, PDF ≤ 10 MiB, NFR-SEC-08) dan centang persetujuan teks pemrosesan data (tidak tercentang otomatis, versi tampil). Galat per field, isian benar tidak hilang, tombol kirim terkunci saat memuat.
3. Setelah kirim: layar "Periksa email Anda" yang identik untuk email baru dan email yang sudah terdaftar (US-1901 AC4). NIB yang sudah dipakai: galat generik tanpa nama perusahaan lain dan arahan ke Layanan Pelanggan (AC7). Setelah email terverifikasi dan masuk pertama: pendaftaran TOTP dan 12 kode pemulihan sekali tampil (pola D-A15, KEPUTUSAN-253).
4. Menunggu verifikasi: badge "Menunggu verifikasi", linimasa, profil baca-saja (NIB dan NPWP termasking), menu hanya Status dan Profil (AC6). Ditolak: alasan dari Verifikator, tombol "Perbaiki dan ajukan ulang" membuka formulir dengan data sebelumnya.
5. Dasbor Aktif: status, jumlah HR aktif, tautan Kelola HR; menu Pesanan, Peserta, Invoice tampil nonaktif "belum tersedia" (Fase 3). Kelola HR: tabel anggota (nama, email, status), undang lewat email, nonaktifkan dengan dialog dampak; HR aktif terakhir tidak dapat dinonaktifkan (US-1904 AC2).
6. Dialog sesi: peringatan 2 menit sebelum 30 menit tidak aktif dengan "Tetap masuk"; batas 12 jam tanpa perpanjangan (§5.5).

Kondisi khusus:
- Isolasi: objek perusahaan lain dijawab "Data tidak ditemukan" (US-1904 AC3, KU-03). Akun atau perusahaan nonaktif: sesi berakhir pada permintaan berikutnya (AC5) dengan layar "Akun nonaktif" (KU-10).
- Dasbor tidak pernah menampilkan hasil atau ringkasan peserta (BR-143); isi dasbor lanjutan menunggu fase layanan korporat `[TIDAK DIKETAHUI]`.
- Bukti kewenangan: surat kuasa atau surat penunjukan dari perusahaan, ditandatangani pejabat berwenang `[USER] C21-P4` (putaran 5, 4b). Tiga isian unggah terpisah (NIB, NPWP, bukti kewenangan) dengan galat per field; pesan ukuran menyebut batas per jenis berkas. Galat pengiriman di tengah proses: "Pendaftaran belum terkirim", dinyatakan belum ada akun dibuat, tombol "Coba lagi" (AC5). Kemajuan pendaftaran tidak disimpan di perangkat; password dan berkas tidak pernah disimpan di sisi klien. Kotak centang di implementasi memakai `input type=checkbox` asli (A7, A11).
- `[TIDAK DIKETAHUI]` di wireframe: format NIB dan NPWP (US-1901 AC3), siapa yang dianggap pejabat berwenang, teks persetujuan (L-20), daftar kode alasan penolakan selain "bukti kewenangan tidak memadai", peran dalam perusahaan (PT-68), isi dasbor korporat (PT-61, PT-62).

Rujukan: US-1901, US-1902 AC3, US-1904, BR-132, BR-136, BR-145, ADR-027, ADR-028, KEPUTUSAN-253, KEPUTUSAN-273.

**D-C13 Verifikasi dan pengelolaan perusahaan (CR-021, F1-T84)** · Verifikator · Wireframe `c13-bo-verifikasi-perusahaan.html`; pola C02.
1. Antrean "Perusahaan": urut terlama, kolom badan usaha, NIB termasking, sumber (mendaftar sendiri atau pengajuan ulang), status. Panel: data lengkap (NIB, NPWP, alamat, pendaftar), tiga dokumen dibuka satu per satu lewat URL berumur pendek (E-05), alasan penolakan wajib saat Tolak (kode alasan dan keterangan opsional), Setujui, dan pesan "Pengajuan sudah diputuskan" bagi keputusan kedua (US-1902 AC4, E-10).
2. Buat perusahaan: nama, NIB, NPWP, alamat, telepon, dokumen, email HR pertama → perusahaan Aktif dan undangan aktivasi; NIB ganda ditolak dengan pesan generik (US-1903 AC4).
3. Daftar perusahaan: filter status dan nama; panel status, sumber, anggota HR; tanpa pesanan atau invoice sampai tersedia (US-1916 AC1).

Tambahan (review QA): panel memuat petunjuk pemeriksaan bukti kewenangan (nama badan usaha cocok dengan NIB, tanda tangan pejabat berwenang, nama pendaftar tercantum; kalau tidak, tolak dengan "bukti kewenangan tidak memadai"); keadaan "Dokumen tidak dapat dibuka" dengan tombol Buka ulang (E-05); Buat perusahaan memakai tiga isian unggah terpisah; dialog "Nonaktifkan perusahaan" dengan alasan wajib dan dampak "semua HR ditolak seketika" (US-1916 AC2, di Fase 1 belum ada pesanan; aturan pesanan berjalan menyusul PT-63). Teks galat NIB ganda kanonik: "Data yang Anda kirim tidak dapat digunakan untuk mendaftar. Periksa kembali data Anda, atau hubungi Layanan Pelanggan untuk bantuan." (sama di K01 dan C13).

Kondisi khusus: akun tanpa sub-peran: 403 dan layar akses ditolak (C01); akun HR: 404. Tidak ada layar untuk melihat atau menetapkan kredensial HR; reset MFA HR bukan di layar ini (US-1904 AC8, Fase 2). Persetujuan orang kedua untuk perusahaan buatan admin `[TIDAK DIKETAHUI]` (PT-67 sisa, M-49); daftar kode alasan penolakan final `[TIDAK DIKETAHUI]`; nasib pesanan saat perusahaan dinonaktifkan `[TIDAK DIKETAHUI]` (PT-63), (hanya relevan setelah pesanan ada). · US-1902, US-1903, US-1916.

**D-C14 CMS kanal situs (CR-021, F1-T84)** · Konten · Wireframe `c14-bo-cms-situs.html`; memperluas D-C07.
1. Daftar: filter kanal (aplikasi, situs, keduanya), status, dan halaman situs; kolom kanal dan halaman situs; hapus dan pulihkan (E-16).
2. Editor: judul, kanal, halaman situs tujuan (untuk situs), jadwal tayang, isi Markdown, gambar dengan teks alternatif wajib (US-1802 AC5). Pratinjau di sebelah isian (di bawahnya pada < 768 px) dirender di iframe `sandbox` tanpa `allow-same-origin` (AC8). Galat per baris untuk HTML mentah, skema tautan di luar https/mailto/relatif, dan gambar dari luar endpoint gambar publik (AC6).
3. Tayangkan halaman tetap: dialog menyebut bahwa versi yang sedang tayang akan berakhir (AC2). Teks bantu: promo hanya informasi (AC3).

Hapus konten: dialog dengan alasan wajib (`delete_reason`, ADR-003); filter Terhapus menampilkan konten terhapus dengan aksi "Pulihkan"; keduanya E-16. Daftar kosong: "Tidak ada konten".

Kondisi khusus: akun tanpa sub-peran Konten: 403 dan percobaan tercatat (AC4). Isi berbahaya yang tampil di pesan galat dirender sebagai teks, bukan HTML. Pemilihan pustaka Markdown di F1-T111. Nama status untuk jadwal tayang belum tiba `[TIDAK DIKETAHUI]` (KEPUTUSAN-274). · US-1802, US-1101, ADR-027 butir 5.

**D-D02 Profil dan keluhan di portal** · Admin rekanan · (1) Profil baca-saja: nama, jenis, alamat, zona waktu, kontak, rekening termasking, sumber sistem, status integrasi, dan petunjuk "Hubungi Layanan Pelanggan untuk perubahan"; (2) Keluhan: daftar tiket yang diteruskan + riwayat tanggapan, tanpa kotak balas. · US-1202, US-1203.

**D-E02 Onboarding dan pengguna RME** · Operator RME, Admin Fasyankes · (1) Operator: daftarkan fasyankes (hanya rekanan terverifikasi bersumber sistem "RME minimalis" dan perjanjian tercatat) + akun Admin Fasyankes pertama; Operator tidak punya menu data pasien; (2) Admin Fasyankes: daftar pengguna, undang (peran Tenaga Kesehatan Fasyankes, Petugas Lab, Admin Fasyankes), nonaktifkan/ubah peran dengan alasan, hubungkan akun tenaga kesehatan Ruang Sehat; (3) Ekspor rekam medis saat berhenti. · Admin Fasyankes terakhir tidak dapat dinonaktifkan (US-1302 AC5). Identitas profesi SATUSEHAT wajib untuk Tenaga Kesehatan Fasyankes (rincian `[TIDAK DIKETAHUI]`). · US-1301, US-1302.

**D-E03 Jadwal, layanan, harga (RME)** · Admin Fasyankes · (1) Daftar layanan (layanan klinik, pemeriksaan lab, paket MCU, home visit + area) dengan harga rekanan; (2) Formulir layanan; (3) Jadwal: hari, jam, kuota per slot, hari libur. · Perubahan yang menyentuh slot terbayar ditolak dengan pesan (US-1303 AC3–AC4). Catatan "tampil di aplikasi pasien paling lambat 15 menit" (NFR-INT-02). Butir vaksinasi/infus tidak ditambahkan sampai CR-015 diputuskan. · US-1303.

**D-E04 Hasil lab (RME)** · Petugas Lab · (1) Daftar pemeriksaan yang sudah check-in; (2) Formulir nilai per pemeriksaan (nilai, satuan, nilai rujukan) dan/atau unggah PDF ≤ 10 MB → Finalkan; (3) Koreksi dengan alasan. · Validasi penanggung jawab lab `[TIDAK DIKETAHUI]` (L-05). · US-1307.

**D-E05 Penugasan home visit (RME)** · Admin Fasyankes · (1) Daftar pesanan home visit terbayar: jadwal, alamat, layanan, nama pasien, status; (2) Pilih tenaga kesehatan dari daftar tenaga kesehatan aktif klinik di Ruang Sehat → simpan; (3) Status kunjungan dari aplikasi tenaga kesehatan tampil di daftar. · Penugasan ulang mengganti penugasan lama (US-502 AC3). Klinik dengan sistem sendiri menugaskan di sistemnya (tanpa layar Ruang Sehat). · US-1308, US-502.

**D-E06 Riwayat dan log akses (RME)** · Tenaga Kesehatan Fasyankes, Admin Fasyankes · (1) Cari pasien yang pernah datang lewat Ruang Sehat → riwayat rekam medis di fasyankes itu (urut terbaru, per versi); (2) Admin Fasyankes: log akses rekam medis (siapa, kapan, pasien). · Data Ruang Sehat (chat, catatan konsultasi online, hasil fasyankes lain) tidak tersedia (US-1310 AC2). · US-1310.

**D-F01 Verifikasi resep publik** · Pemeriksa anonim (apotek) · (1) Halaman dengan isian kode verifikasi (atau terbuka langsung dari QR); (2) Hasil: status Terbit/Dibatalkan (badge bertulisan), tanggal terbit, dokter dan nomor SIP, klinik, daftar obat, nama pasien disamarkan; (3) "Resep tidak ditemukan" untuk kode tidak valid; (4) Pesan batas laju "Terlalu banyak percobaan, coba lagi nanti". · Satu kolom ≤ 400 px, token yang sama, tanpa skrip pihak ketiga. Isi final menunggu L-05/L-08. · US-309 AC6–AC8.

**D-F02 Email notifikasi** · Sistem · Satu template dasar (§6) dengan variasi per kejadian matriks EP-10 dan US-1003: judul kejadian, satu kalimat tanpa data klinis, nomor pesanan, tombol "Buka aplikasi Ruang Sehat". Pengingat jadwal (US-1002) memuat waktu dengan label zona. · AB-73.

### 10.3 Story tanpa layar sendiri

| Story | Alasan | Tampak di |
| --- | --- | --- |
| US-102 | Dihapus CR-007 | — |
| US-204 | Proses terjadwal | Peringatan SIP di B01; email |
| US-404 | Status dari sistem rekanan | Badge "Check-in" di A08; check-in RME di E01 |
| US-803 | Pencatatan pembagian dana oleh sistem | Laporan C03, D01 |
| US-806 | Refund otomatis oleh sistem | Status refund A12 |
| US-901 | Penerimaan hasil lewat integrasi | Notifikasi A02, hasil A08 |
| US-1002 | Email pengingat | D-F02 |
| US-1401–US-1404, US-1406 | Kontrak dan proses integrasi | — |
| US-1501, US-1504 | Pencatatan dan perlindungan audit oleh sistem | D-C10 (tanpa fitur ubah/hapus) |
| US-1606 | Proses terjadwal, terblokir PT-24 | — |

## 11. Usulan penyusun untuk Checkpoint 2

| ID | Usulan | Alasan | Alternatif |
| --- | --- | --- | --- |
| UP-58 | Arah visual memakai palet pamflet (#087F78, #DFF4ED, #FAFBF7, #FFFFFF) sebagai warna dasar, sudut membulat 16, nuansa terang dan bersih, dengan warna turunan §3.1 untuk teks dan status. Nama tampil "Ruang Sehat"; web RME memakai nama permukaan "RME" dengan identitas visual yang sama | Pamflet adalah satu-satunya masukan merek dari user `[DOK: input/Rumah Sehat Apps.jpeg]`; T5-Q12 menyebut palet dan simbol dapat menjadi masukan Tahap 6 | Palet lain dari desainer; RME dengan identitas visual terpisah |
| UP-59 | Rilis 1 memakai font sistem; Plus Jakarta Sans (OFL 1.1 `[SRC-259]`) sebagai opsi font merek yang dibundel lokal | Font sistem tanpa beban berkas dan lisensi; Plus Jakarta Sans berlisensi terbuka bila merek menginginkan font khas | Font berlisensi lain pilihan desainer |
| UP-60 | Logo bertulisan "Ruang Sehat" dan ikon aplikasi dibuat desainer dengan arah simbol rumah-daun pamflet; nama ditulis "Ruang Sehat" dengan spasi, wordmark pamflet "RuangSehat" tidak dipakai (`[USER]` PT-45); slogan pamflet menunggu konfirmasi; wireframe hanya memakai placeholder | Logo adalah aset merek; penyusun tidak membuat logo final | User menyediakan logo sendiri |
| UP-61 | Mode gelap tidak dirancang di rilis 1 | Menghemat waktu desain dan uji kontras ganda menjelang 1 Maret 2027 | Rancang mode gelap sejak rilis 1 |
| UP-62 | Navigasi aplikasi pasien empat tab: Beranda, Pesanan (aktif + riwayat semua layanan), Kesehatan (catatan konsultasi, resep, hasil lab), Profil; notifikasi di ikon lonceng. Aplikasi tenaga kesehatan: Sesi, Home visit, Jadwal, Profil | Mengelompokkan story PRD per kebutuhan pengguna; mengikuti pola empat tab pamflet dengan label disesuaikan scope | Label pamflet apa adanya (Beranda, Jadwal, Riwayat, Profil) |
| UP-63 | Beranda menampilkan enam ubin untuk tujuh layanan rilis 1: Konsultasi Online, Pendaftaran Klinik, Pendaftaran Lab, Home Visit, Biaya MCU & Lab, Belanja Produk (obat bebas, kecantikan, kesehatan lain di dalamnya). Ubin pamflet "Vaksinasi" dan "Suntik & Infus Vitamin" tidak tampil (CR-015 belum diputuskan) | Mengikuti scope PRD §2.1; CR-015 hanya boleh tampil sebagai butir katalog lab | Ubin terpisah untuk obat bebas dan produk lain; ubin terpisah psikolog |
| UP-64 | Pengingat tetap di detail pesanan dan ruang tunggu: "Buka aplikasi pada jadwal; Ruang Sehat tidak mengirim notifikasi ke layar ponsel" | Mengurangi risiko produk CR-007 (pasien melewatkan jadwal tanpa push) | Tanpa pengingat di layar |

## 12. Pertanyaan terbuka baru

| ID | Pertanyaan | Pemilik | Tenggat |
| --- | --- | --- | --- |
| PT-40 | Siapa membuat logo final "Ruang Sehat", ikon aplikasi, dan aset toko aplikasi (desainer internal, eksternal, atau user)? | User | Sebelum pengajuan ke toko aplikasi (Tahap 12); arah logo di Checkpoint 2 |
| PT-41 | Di pamflet, contoh warna kedua tampak lebih gelap dari nilai `#DFF4ED` yang tertulis. Nilai mana yang dimaksud (tulisan heksadesimal atau warna yang tampak)? | User / desainer | Checkpoint 2 |
| PT-42 | Apakah catatan konsultasi dan rekam medis yang belum difinalkan perlu disimpan sebagai draf di server agar tidak hilang saat aplikasi ditutup? Koreksi 2026-11-18 (Tahap 7): data dictionary **sudah** memuat tabel draf di server, A9.6 `consultation_note_draft` dan C4.1 `rme_medical_record_draft` `[DOK: 04-data-dictionary.md]`; premis awal "belum memuat status draf" keliru. Usulan perlakuan: UP-72 (`docs/07-catatan-phase-plan.md`) | Tech Lead (Claude) bersama user | Tahap 7, sebelum EP-03 |
| PT-43 | Dua batas waktu tidak dapat diperpanjang pengguna: sesi web mutlak 12 jam (ADR-011) dan batas pembayaran PB-07 (30 menit, menahan slot/stok). Apakah keduanya diterima sebagai pengecualian SC 2.2.1 (alasan keamanan dan keadilan slot), atau perlu jalur perpanjangan (misalnya login ulang tanpa kehilangan isian; memilih slot lagi dengan isian tersimpan)? **Terjawab di Checkpoint 2:** keduanya pengecualian essential SC 2.2.1 dengan mitigasi §5.5 (UP-71) `[USER] C2-B15` | User + security reviewer | Terjawab |
| PT-44 | Pesan apa yang tampil saat akun terkunci sementara (US-103 AC3): pesan umum yang sama dengan AC2 (tidak membuka status akun) atau pesan khusus terkunci? Draf wireframe A01 layar 5 ditandai menunggu keputusan (temuan QA N2). **Terjawab di Checkpoint 2:** pesan seragam yang identik untuk semua kegagalan login, termasuk terkunci dan nonaktif (UP-73 opsi B, US-103 AC2); tanpa email pemberitahuan penguncian `[USER] C2-B4`. Wireframe A01 layar 5 disesuaikan di Tahap 8 (UI-03) | Tech Lead + security reviewer, disetujui user | Terjawab |
| PT-45 | ~~Wordmark pamflet "RuangSehat" atau "Ruang Sehat" dengan spasi?~~ **Terjawab 2026-11-18:** "Ruang Sehat" dengan spasi di semua tempat; wordmark pamflet tidak dipakai `[USER] "dengan spasi"`. Sisa: slogan pamflet dipakai atau tidak (Checkpoint 2, UP-60) | User | Terjawab (slogan: Checkpoint 2) |
| PT-46 | Apakah nama "Ruang Sehat" sudah dicek ketersediaannya sebagai merek (DJKI) dan nama domain, serta apakah nama aplikasi di toko aplikasi mengikuti nama ini? | User, legal (PT-20) | Sebelum pengajuan ke toko aplikasi (Tahap 12). Status 2026-11-18: belum dicek `[USER] "belum dicek"` |

Celah spesifikasi CR-021 dari review QA F1-T83/T84 (`docs/qa/F1-T83-T84-review.md`), belum diputuskan desain dan diteruskan ke pemilik PRD:

| ID | Pertanyaan | Pemilik |
| --- | --- | --- |
| S-1 | Apakah NIB ganda (US-1901 AC7) mencakup perusahaan berstatus Menunggu verifikasi, Ditolak, Nonaktif, atau hanya Aktif? | User (PRD) |
| S-2 | Urutan pemeriksaan email terdaftar dan NIB ganda, serta nasib data dan dokumen saat email sudah terdaftar (AC4 vs AC7) | User (PRD) |
| S-3 | Batas laju endpoint pendaftaran perusahaan (unggahan dokumen, pengecekan NIB) | User, security |
| S-4 | Lupa password realm `korporat` (US-1904 tidak memuat AC; K01 layar 1 menampilkan tautannya) | User (PRD) |
| S-5 | HR menonaktifkan diri sendiri; email undangan sudah menjadi HR perusahaan lain; email HR pertama sudah terdaftar (US-1903 AC1) | User (PRD) |
| S-6 | Daftar status konten seragam (Selesai atau berakhir, terjadwal) di PRD §3.6 (terkait KEPUTUSAN-274) | User (PRD) |
| S-7 | Memulihkan halaman tetap terhapus saat versi lain Tayang: usulan QA status Draf | User (PRD) |

Pertanyaan yang sudah ada dan memengaruhi layar: L-08 dan PT-24 (teks persetujuan), PT-28 (baris pajak), AB-25 (metode pembayaran), L-03 (informasi produk BPOM), L-05 (resep, rekam medis), batas usia anak (US-108 AC6), CR-015.

## 13. Gate Tahap 6

| Syarat | Status |
| --- | --- |
| Semua alur utama di PRD punya wireframe | Terpenuhi: 39 alur di §10.1 (21 HTML, 18 deskripsi); story tanpa layar dijelaskan di §10.3. Dicek QA: semua story PRD terpetakan kecuali US-102 (dihapus CR-007) |
| Komponen dasar dan design token terdefinisi | Terpenuhi: §3, §4 |
| User menyetujui arah desain | Disetujui di Checkpoint 2, 2026-11-18 (UP-58–UP-64) `[USER] C2-B8, C2-B9, C2-B10, C2-B11, C2-B15`; PT-40, PT-41, sisa PT-45 tetap terbuka |
| Gate anti-halu | Fact-checker LULUS (docs/fact-check/tahap-06.md); QA LULUS DENGAN CATATAN (docs/qa/tahap-06-review-wireframe.md). SRC-256–SRC-260 tercatat; KEPUTUSAN-160–177 tercatat; traceability story → wireframe diperbarui |
