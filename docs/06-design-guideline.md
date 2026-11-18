# Design Guideline: Ruang Sehat

Tanggal: 2026-11-18
Versi: 0.1 (draf Tahap 6, menunggu Checkpoint 2)
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
| K4 | `brand.primary` #087F78 sebagai teks | #DFF4ED (mint) | **4,24** | 4,5 | **Gagal untuk teks.** Teks di atas mint memakai `primaryStrong` (K6) |
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
| Rincian harga | Layanan, produk | Baris: harga layanan, ongkir (produk), pajak, biaya transaksi, total | — | — | — | — | Kerangka; tombol bayar terkunci sampai angka dari server tiba | Semua angka dari server (US-801 AC1). Markup tidak tampil sebagai baris (AB-13). Baris pajak mengikuti CR-013; label dan tarif menunggu PT-28 |
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
| MFA (tenaga kesehatan, admin, RME) | Layar kode TOTP + tautan kode pemulihan; aktivasi pertama menampilkan QR dan kunci teks | ADR-011 butir 5 |
| Autentikasi ulang | Mengajukan hak data, mengunduh salinan data, mengubah email/password: diberi tahu lebih dulu bahwa password akan diminta lagi | ADR-011 butir 9; US-1602 AC1; US-1604 AC2 |
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
| B01 | Pendaftaran tenaga kesehatan, status verifikasi, masuk MFA, kunci aplikasi, layar privasi | EP-02 US-202, US-204; NFR-SEC-10 | Tenaga kesehatan | HTML | `b01-nakes-akun-kunci.html` |
| B02 | Jadwal praktik, daftar sesi, ruang sesi, catatan, resep, pembatalan | EP-03 US-302, US-305–US-309, US-311; EP-10 US-1003 | Tenaga kesehatan | HTML | `b02-nakes-sesi-jadwal.html` |
| B03 | Tugas dan status home visit | EP-05 US-503 | Tenaga kesehatan | HTML | `b03-nakes-home-visit.html` |
| C01 | Masuk web dengan MFA, batas sesi, akses ditolak | EP-02 US-206, US-208; US-1302; NFR-SEC-02, NFR-SEC-04; KU-03, KU-10 | Ketiga web | HTML | `c01-web-masuk-sesi.html` |
| C02 | Verifikasi identitas dan tenaga kesehatan | EP-02 US-201, US-203 | Backoffice | HTML | `c02-bo-verifikasi.html` |
| C03 | Keputusan klaim, antrean Keuangan, laporan transaksi | EP-08 US-807, US-808, US-811, US-802 AC3 | Backoffice | HTML | `c03-bo-klaim-keuangan.html` |
| C04 | Onboarding rekanan dan akun admin mitra | EP-02 US-205, US-206 | Backoffice | Deskripsi | D-C04 |
| C05 | Kelola akun: nonaktif/aktif, admin RS dan sub-peran | EP-02 US-207, US-208 | Backoffice | Deskripsi | D-C05 |
| C06 | Markup, biaya rekanan, parameter bisnis | EP-08 US-809, US-810, US-812 | Backoffice | Deskripsi | D-C06 |
| C07 | Konten (CMS) | EP-11 US-1101 | Backoffice | Deskripsi | D-C07 |
| C08 | Kelola tiket keluhan | EP-11 US-1104 | Backoffice | Deskripsi | D-C08 |
| C09 | Pemantauan integrasi rekanan | EP-14 US-1405 | Backoffice | Deskripsi | D-C09 |
| C10 | Audit log dan ekspor | EP-15 US-1502, US-1503 | Backoffice | Deskripsi | D-C10 |
| C11 | Proses hak subjek data (Petugas PDP) | EP-16 US-1603, US-1605 | Backoffice | Deskripsi | D-C11 |
| C12 | Dasbor KPI | EP-17 US-1701 | Backoffice | Deskripsi | D-C12 |
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

Ringkasan: 21 alur dengan wireframe HTML (semua alur aplikasi pasien kecuali keluhan, ketiga alur tenaga kesehatan, dan alur kunci web), 18 alur dengan deskripsi terstruktur.

### 10.2 Deskripsi terstruktur

Format: tujuan · aktor · layar berurutan · kondisi khusus · rujukan. Semua memakai komponen §4 dan pola §5; web memakai tata letak "daftar + panel detail" seperti C02/C03.

**D-A14 Keluhan pasien** · Pasien · (1) Profil → Bantuan dan keluhan: daftar tiket (nomor, kategori, status Baru/Diproses/Menunggu pasien/Selesai); (2) Formulir: kategori (layanan, pembayaran, akun, lainnya), pesanan terkait opsional, deskripsi, foto opsional → nomor tiket tampil; (3) Detail tiket: utas balasan + kotak balas. · Memilih kategori klaim refund atau hak data mengarahkan ke A12/A13 (US-1103 AC3). Tiket yang ditutup dibuka lagi bila dibalas dalam 7 hari (US-1104 AC3). Kosong: "Belum ada keluhan". · US-1103.

**D-C04 Onboarding rekanan dan admin mitra** · Verifikator · (1) Daftar rekanan dengan status (Menunggu verifikasi, Aktif, Nonaktif) dan tiga syarat tampil (dokumen disetujui, markup diatur, integrasi Aktif) sebagai daftar centang bertulisan; (2) Formulir rekanan: nama, jenis (klinik, lab, toko kecantikan, toko alat kesehatan, apotek; klinik dapat sekaligus outlet), alamat dan zona waktu, dokumen izin, rekening penerima (termasking setelah disimpan), sumber sistem; (3) Panel akun admin mitra: undang lewat email. · Perubahan rekening hanya berlaku setelah disetujui Verifikator (US-1202 AC3). Daftar dokumen wajib menunggu L-02/L-03. · US-205, US-206.

**D-C05 Kelola akun** · Super Admin, Verifikator, Layanan Pelanggan sesuai hak · (1) Cari akun (pasien, tenaga kesehatan, rekanan, admin); (2) Detail akun + tombol Nonaktifkan/Aktifkan dengan alasan wajib; (3) Admin RS: centang sub-peran. · Ditolak dengan pesan bila pasien punya pesanan aktif (US-207 AC3), bila mencabut Super Admin terakhir (US-208 AC2), atau mengubah peran sendiri (AC3). Dialog menyebut dampak penonaktifan rekanan/tenaga kesehatan ke pesanan (US-207 AC2). · US-207, US-208.

**D-C06 Markup, biaya rekanan, parameter** · Keuangan (markup, biaya), Super Admin (parameter) · (1) Pilih rekanan → riwayat markup (jenis, nilai, berlaku mulai) + tombol "Atur markup"; (2) Formulir: jenis persen/nominal, nilai, berlaku segera atau tanggal mendatang, ringkasan konfirmasi; (3) Biaya rekanan dengan pola sama; (4) Parameter PB-01–PB-10, PB-14–PB-20: tabel nilai, satuan, riwayat. · Nilai tidak valid ditolak per isian (US-812 AC3). Teks penjelas bahwa perubahan tidak memengaruhi pesanan yang pembayarannya sudah dibuat (AB-17). Tarif pajak menjadi parameter Keuangan setelah CR-013 diterapkan. · US-809, US-810, US-812.

**D-C07 Konten** · Konten · (1) Daftar konten (banner, halaman info, artikel, promo) dengan status Draf/Tayang/Selesai/Diarsipkan; (2) Editor: judul, isi, gambar, kategori, jadwal tayang, pratinjau seluler; (3) Hapus dan pulihkan. · Teks bantu "Promo hanya informasi, tidak mengubah harga" (US-1101 AC4). Gambar wajib teks alternatif. · US-1101.

**D-C08 Tiket keluhan** · Layanan Pelanggan · (1) Antrean tiket urut terlama dengan filter status; (2) Detail: utas, ubah status, teruskan ke rekanan, catat tanggapan rekanan. · Tiket yang diteruskan tidak memuat data klinis (US-1104 AC2). · US-1104.

**D-C09 Pemantauan integrasi** · Admin RS · (1) Tabel rekanan: status integrasi (Belum, Uji, Aktif, Terganggu, Nonaktif) dengan badge bertulisan, sinkron terakhir berhasil, jumlah galat 24 jam; (2) Detail + ubah status dengan alasan. · Rekanan Terganggu ditandai peringatan di atas tabel (US-1405 AC2). · US-1405.

**D-C10 Audit log** · Auditor, Super Admin · (1) Filter: rentang waktu, aktor, peran, jenis event, jenis dan ID objek, subjek data, hasil, alamat IP; (2) Tabel berhalaman urut terbaru; (3) Panel detail semua field §7.2 termasking; (4) Ekspor CSV hanya untuk Auditor. · Tidak ada tombol ubah/hapus di mana pun (US-1504 AC1). · US-1502, US-1503.

**D-C11 Hak subjek data** · Petugas PDP · (1) Antrean dengan kolom sisa waktu menuju 3×24 jam dan badge "Mendesak" saat tinggal 12 jam; (2) Detail: jenis, subjek, tahap; isian tanggapan dan alasan; aksi sesuai jenis (perbaikan, pembatasan, salinan otomatis). · NIK termasking; isi klinis tidak dibuka petugas (salinan dibuat sistem). Penghapusan (US-1605) terblokir sampai L-06/PT-24: tombol tampil nonaktif dengan alasan tertulis. · US-1603, US-1605.

**D-C12 Dasbor KPI** · Super Admin · (1) Pilih bulan (WIB); (2) Kartu KPI-01 (dan angka pendukung), KPI-02, KPI-03, KPI-04 dengan angka dan label teks; grafik opsional selalu disertai tabel angka; (3) Ekspor CSV agregat. · US-1701.

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
| PT-42 | Apakah catatan konsultasi dan rekam medis yang belum difinalkan perlu disimpan sebagai draf di server agar tidak hilang saat aplikasi ditutup? Data dictionary belum memuat status draf | Tech Lead (Claude) bersama user | Tahap 7, sebelum EP-03 |
| PT-43 | Dua batas waktu tidak dapat diperpanjang pengguna: sesi web mutlak 12 jam (ADR-011) dan batas pembayaran PB-07 (30 menit, menahan slot/stok). Apakah keduanya diterima sebagai pengecualian SC 2.2.1 (alasan keamanan dan keadilan slot), atau perlu jalur perpanjangan (misalnya login ulang tanpa kehilangan isian; memilih slot lagi dengan isian tersimpan)? | User + security reviewer | Tahap 8 Fase 1 |
| PT-44 | Pesan apa yang tampil saat akun terkunci sementara (US-103 AC3): pesan umum yang sama dengan AC2 (tidak membuka status akun) atau pesan khusus terkunci? Draf wireframe A01 layar 5 ditandai menunggu keputusan (temuan QA N2) | Tech Lead + security reviewer, disetujui user | Tahap 7 |
| PT-45 | ~~Wordmark pamflet "RuangSehat" atau "Ruang Sehat" dengan spasi?~~ **Terjawab 2026-11-18:** "Ruang Sehat" dengan spasi di semua tempat; wordmark pamflet tidak dipakai `[USER] "dengan spasi"`. Sisa: slogan pamflet dipakai atau tidak (Checkpoint 2, UP-60) | User | Terjawab (slogan: Checkpoint 2) |
| PT-46 | Apakah nama "Ruang Sehat" sudah dicek ketersediaannya sebagai merek (DJKI) dan nama domain, serta apakah nama aplikasi di toko aplikasi mengikuti nama ini? | User, legal (PT-20) | Sebelum pengajuan ke toko aplikasi (Tahap 12). Status 2026-11-18: belum dicek `[USER] "belum dicek"` |

Pertanyaan yang sudah ada dan memengaruhi layar: L-08 dan PT-24 (teks persetujuan), PT-28 (baris pajak), AB-25 (metode pembayaran), L-03 (informasi produk BPOM), L-05 (resep, rekam medis), batas usia anak (US-108 AC6), CR-015.

## 13. Gate Tahap 6

| Syarat | Status |
| --- | --- |
| Semua alur utama di PRD punya wireframe | Terpenuhi: 39 alur di §10.1 (21 HTML, 18 deskripsi); story tanpa layar dijelaskan di §10.3. Dicek QA: semua story PRD terpetakan kecuali US-102 (dihapus CR-007) |
| Komponen dasar dan design token terdefinisi | Terpenuhi: §3, §4 |
| User menyetujui arah desain | Menunggu Checkpoint 2 (UP-58–UP-64) |
| Gate anti-halu | Fact-checker LULUS (docs/fact-check/tahap-06.md); QA LULUS DENGAN CATATAN (docs/qa/tahap-06-review-wireframe.md). SRC-256–SRC-260 tercatat; KEPUTUSAN-160–177 tercatat; traceability story → wireframe diperbarui |
