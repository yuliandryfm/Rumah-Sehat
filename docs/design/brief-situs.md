# Brief Desain: Situs Web Publik Ruang Sehat (Company Profile)

Tanggal: 2026-11-26
Versi: 0.1 (draf untuk diteruskan ke UI/UX Designer)
Penyusun: Claude (persona Creative Director + Senior Brand/UI Designer, bersama Product Manager)
Pemilik dan pemutus: user (project manager, meneruskan ke CEO dan CTO)
Sifat dokumen: mandiri. Desainer tidak perlu akses repositori. Semua fakta bersumber dari dokumen proyek dan kode yang disebut di Lampiran A. Ada tiga kategori yang harus dibedakan:
1. **Fakta bisnis/konten yang belum ada** (logo final, slogan, alamat, kontak, tautan toko aplikasi, nama rekanan, domain, teks hukum, angka pencapaian) ditulis `[TIDAK DIKETAHUI]`. Desainer **tidak boleh mengarang**. Pakai penanda (placeholder) yang jelas bertanda draf.
2. **Rekomendasi desain penyusun** diberi label `[USULAN-DESAIN]`. Label ini berarti usulan yang **belum disetujui user** dan konsisten dengan aturan PRD/ADR/DG yang ada. Usulan yang menuntut perubahan kode, model konten CMS, atau design guideline **wajib lewat change request (CR) di `docs/backlog.md`** sebelum dibangun. Kandidat CR dirangkum di §10.
3. **Konten demo** yang sekarang tayang (`[DEMO]`) disusun dari product knowledge proyek sebagai contoh isi. Statusnya **draf**, belum disetujui sebagai teks final. Beberapa kalimatnya masih perlu konfirmasi (§5, Lampiran B).

Arahan user untuk brief ini: tampilan situs harus "lebih fancy, profesional, eye catching", dan tampilan sekarang dinilai "sangat kaku dan jelek" `[USER 2026-11-26]`. Arah visual baru ada di §7.

## 0. Cara membaca

> **Pembaruan CR-042 (2026-11-28, `[USER]`: "buat UIUX situs sama seperti tampilan pada file yang saya upload"):** tampilan situs kini mengikuti paket desain Stitch (`docs/design/stitch-ruang-sehat-design-system.md`, logo `docs/design/aset/logo-ruang-sehat.svg`) dan **menggantikan konsep visual CR-033** (konsep A "Ruang Teduh" + seksi gelap konsep B, Q15/Q16/Q18) di §7. Perubahan utama: font Instrument Sans/Alata/Spline Sans Mono (menggantikan Plus Jakarta Sans), header berlatar bg.app dengan pil aktif, lencana pil di hero dan pita judul, kartu layanan berpanel mint, tata letak artikel majalah, pita ajakan unduh, pita judul Untuk Perusahaan terang, seksi korporat gelap sebagai kartu bersudut 28. Aturan §6 (keamanan, konten, fakta) dan §9 (CSP ketat, tanpa sumber daya eksternal) tetap berlaku; isi tetap dari dokumen proyek, bukan salinan mockup. Keputusan: KEPUTUSAN-475–480; sisa: TD-68–70. Bagian §7 di bawah adalah riwayat konsep CR-033.


| Penanda | Arti |
| --- | --- |
| `[BRD BR-xxx]` | BRD (`docs/02-brd.md`) |
| `[PRD US-xxxx ACn]`, `[PRD NFR-xx]`, `[PRD PT-xx]`, `[PRD L-xx]` | PRD v1.4 (`docs/03-prd.md`): story, acceptance criteria, kebutuhan non-fungsional, pertanyaan terbuka, konfirmasi legal |
| `[BRIEF]` | Project brief (`docs/00-project-brief.md`) |
| `[ADR-027 b.n]` | ADR-027 butir n (situs publik dan member area korporat) |
| `[DG §x]`, `[DG D-S01]`, `[DG UP-xx]` | Design guideline v1.2 (`docs/06-design-guideline.md`) |
| `[WF S01-n]` | Wireframe situs layar n (`docs/wireframes/s01-situs-publik.html`) |
| `[PROTO n]` | Prototipe HTML situs (`docs/prototype/situs/01…10-*.html`) |
| `[IMPL nama-berkas]` | Implementasi `web/situs/src/` per commit `9475fda` (2026-11-26) |
| `[DEMO]` | Konten demo yang tayang (skrip seed konten demo, di luar repositori) |
| `[KEPUTUSAN-xx]` | Keputusan desain/teknis tercatat di `docs/decision-log.md` |
| `[CR-0xx]` | Change request di `docs/backlog.md` |
| `[BRIEF-K]` | Brief desain web korporat (`docs/design/brief-web-korporat.md` v0.1) |
| `[USER 2026-11-26]` | Keputusan atau arahan user tanggal itu |
| `[USULAN-DESAIN]` | Rekomendasi penyusun, belum disetujui. Untuk build wajib CR bila menyentuh kode/CMS/DG |
| `[TIDAK DIKETAHUI]` | Informasi belum ada, tidak boleh ditebak |

Teks di wireframe, prototipe, dan konten demo adalah **draf**, kecuali dikutip sebagai "ditetapkan" di brief ini.

## 1. Ringkasan produk dan peran situs

**Ruang Sehat** adalah aplikasi kesehatan (Flutter, Android dan iOS) untuk pasien yang sulit datang ke klinik atau apotek. Rilis pertama berisi tujuh layanan: konsultasi online dokter dan psikolog (chat dan video), pendaftaran klinik rekanan, pendaftaran lab rekanan, home visit, informasi biaya MCU dan lab, serta pembelian obat bebas, produk kecantikan, dan produk kesehatan dari outlet rekanan `[BRIEF]`. Ruang Sehat berperan sebagai gateway: data klinik, lab, home visit, dan produk berasal dari sistem rekanan `[BRIEF]`. Operator: PT. Arunika `[BRD BR-130]`. Antarmuka hanya Bahasa Indonesia `[PRD NFR-LOC-01]`. Target go-live 1 Maret 2027 `[BRIEF]`.

**Situs publik** (`web/situs`) adalah situs bergaya company profile dengan sembilan halaman: Beranda, Tentang Kami, Layanan (tujuh layanan dan layanan korporat), Untuk Perusahaan, Rekanan, Artikel/berita, Kontak, Unduh aplikasi, serta Kebijakan privasi dan syarat `[BRD BR-130]`. Kontennya dikelola admin Ruang Sehat lewat CMS backoffice `[BRD BR-131]`. Situs dibuka tanpa login `[PRD US-1801 AC1]`, juga tanpa basic auth di lingkungan demo `[USER 2026-11-26]`.

Hubungan dengan permukaan lain:

| Permukaan | Hubungan dengan situs |
| --- | --- |
| Aplikasi pasien (mobile) | Tujuan utama ajakan bertindak "Unduh aplikasi". Semua layanan dipesan dan dibayar di aplikasi, bukan di situs. Versi web untuk pasien di luar scope rilis 1 `[BRIEF]` |
| Aplikasi tenaga kesehatan (mobile) | Tenaga kesehatan mendaftar di aplikasi itu atas nama klinik rekanan yang menaunginya `[PRD US-202 AC1]` |
| Web korporat `web/korporat` (member area HR) | Aplikasi dan host terpisah dari situs `[ADR-027 b.1]`. Situs menautkan ke sana lewat tombol "Masuk member area" di Untuk Perusahaan dan tautan "Masuk HR" di footer `[CR-032]`. Tombol "Daftarkan perusahaan" menyusul bersama task F1-T103 `[CR-032; USER 2026-11-26]`. Rincian member area ada di `[BRIEF-K]` |
| CMS backoffice | Sumber semua isi halaman. Admin memilih kanal (aplikasi, situs, keduanya) dan halaman tujuan `[PRD US-1802 AC1]`. Desain editor CMS (wireframe C14) di luar brief ini |
| Portal rekanan, RME minimalis | Tidak ditautkan dari situs di sumber mana pun. Kebutuhan tautan `[TIDAK DIKETAHUI]` |

## 2. Tujuan desain dan ukuran keberhasilan

Tujuan (diturunkan dari sumber):
1. Pengunjung memahami apa itu Ruang Sehat dan tujuh layanannya dalam satu kunjungan ke Beranda `[PRD US-1801 user story]`.
2. Setiap audiens (§3) menemukan jalurnya sendiri: pasien ke Unduh aplikasi, HR ke member area korporat, calon rekanan ke kanal kemitraan `[BRD BR-130, BR-132]`.
3. Situs terlihat **profesional, menarik, dan terpercaya** untuk layanan kesehatan, tidak kaku `[USER 2026-11-26]`.
4. Semua itu tercapai **di dalam batasan**: tanpa cookie, tanpa skrip/font/sematan pihak ketiga, aksesibel WCAG 2.2 AA, cepat (§6, §9).

Ukuran yang **bersumber**:

| Ukuran | Nilai | Sumber |
| --- | --- | --- |
| Aksesibilitas | WCAG 2.2 Level AA; kontras semua teks ≥ 4,5:1, komponen ≥ 3:1; lulus pemeriksaan otomatis aturan DG §7 | `[DG §7.1, §3.2; PRD US-1801 AC6]` |
| Waktu tampil halaman | p95 ≤ 3 detik (berlaku untuk situs publik) | `[PRD NFR-PERF-04]` |
| Peramban | Dua versi stabil terakhir Chrome, Edge, Firefox, Safari | `[PRD NFR-COMP-03]` |
| Cookie dan data pribadi | Tidak ada cookie sesi dan tidak ada data pribadi dimuat | `[PRD US-1801 AC5]` |
| Pihak ketiga | Tidak ada skrip, font, atau sematan pihak ketiga | `[PRD US-1802 AC9]` |

KPI bisnis situs (kunjungan, klik unduh, pendaftaran perusahaan dari situs) `[TIDAK DIKETAHUI]`, tidak ada di sumber. Tanpa analitik pihak ketiga (§9), cara mengukurnya juga `[TIDAK DIKETAHUI]`.

## 3. Audiens, pesan utama, dan ajakan bertindak

| Audiens | Pesan utama (bersumber) | CTA utama | CTA kedua | Catatan |
| --- | --- | --- | --- | --- |
| **Calon pasien** (pengguna aplikasi) | Layanan kesehatan tanpa harus selalu datang ke klinik atau apotek `[BRIEF]`; tujuh layanan dalam satu aplikasi `[BRIEF]`; biaya MCU dan lab dapat dilihat sebelum mendaftar `[BRD BR-023]`; konsultasi berlangsung di aplikasi, video tidak direkam `[BRD BR-013, BR-016]`; data disimpan di Indonesia `[BRIEF]` | Unduh aplikasi | Lihat layanan | Tidak ada login, pemesanan, atau harga di situs (§6). Sebelum aplikasi terbit di toko, tombol tidak boleh memakai tautan palsu `[DG D-S01]` |
| **HR / admin perusahaan** | Vaksinasi massal, MCU massal, dan lab massal `[BRD BR-133]`, dilaksanakan klinik/lab rekanan `[BRD BR-134]`, on-site atau di rekanan `[BRD BR-135]`; katalog harga tetap tanpa penawaran `[BRD BR-137]`; invoice B2B transfer bank `[BRD BR-141]`; hasil lengkap hanya untuk peserta, ringkasan hanya dengan persetujuan peserta `[BRD BR-143, BR-144]` | Daftarkan perusahaan (menyusul F1-T103) | Masuk member area `[CR-032]` | Fitur pemesanan, katalog, dan invoice baru dibangun di Fase 3 (2027-01-11 sampai 01-29) `[BRIEF-K §4; PHASE]`. Teks situs tidak boleh menjanjikan fitur yang belum tersedia saat teks itu tayang |
| **Calon rekanan** (klinik, lab, apotek, toko kecantikan, toko alat kesehatan) | Perluasan pasar ke pengguna aplikasi `[BRIEF]`; data diambil dari sistem rekanan lewat integrasi `[BRD BR-033]`; portal rekanan untuk laporan dan rekap `[BRD BR-064]` | Hubungi tim kemitraan (kanal `[TIDAK DIKETAHUI]`) | Lihat rekanan | Onboarding rekanan dilakukan Verifikator, bukan pendaftaran mandiri di situs `[PRD US-205]`. Situs tanpa formulir `[KEPUTUSAN-271]` |
| **Tenaga kesehatan** (dokter, psikolog, fisioterapis, tenaga medis) | Melayani pasien lewat aplikasi tenaga kesehatan atas nama klinik rekanan `[BRD BR-013, BR-014; PRD US-202]` | Unduh aplikasi tenaga kesehatan (tautan `[TIDAK DIKETAHUI]`) | Kontak | Pendaftaran butuh klinik penaung yang aktif `[PRD US-202 AC1, AC4]` |
| **Pembaca artikel** (pengunjung umum) | Konten edukasi kesehatan `[BRIEF]` | Baca artikel lain | Unduh aplikasi | Peninjau medis artikel `[TIDAK DIKETAHUI]` |

Peserta korporat (karyawan) memakai aplikasi pasien, **bukan** web `[BRD BR-132]`. Jangan merancang "Masuk peserta" di situs.

## 4. Ruang lingkup dan status saat ini

**Masuk brief ini:** sembilan halaman BR-130, detail artikel, halaman tidak ditemukan, halaman galat umum, kerangka bersama (header, menu seluler, footer), dan arah visual (§7).

**Di luar brief ini:** member area korporat (lihat `[BRIEF-K]`), editor CMS backoffice (wireframe C14), aplikasi mobile, email.

**Status implementasi saat ini** (commit `9475fda`):

| Bagian | Wireframe/prototipe | Implementasi sekarang | Sumber |
| --- | --- | --- | --- |
| Kerangka | Logo placeholder, navigasi 8 item, menu `details` di bawah 768 px, footer | Sama. Logo hanya teks "Ruang Sehat". Footer: pernyataan platform, Kebijakan, Kontak, "Masuk HR" (bila alamat member area diatur), "Dioperasikan oleh PT. Arunika", "© 2026 PT. Arunika. Hak cipta dilindungi." | `[WF S01-1; IMPL KerangkaSitus.tsx; USER 2026-11-26]` |
| Beranda | Hero, **enam ubin layanan**, blok Untuk Perusahaan, tiga artikel | Hero dari CMS. Bagian Layanan dan Untuk Perusahaan **hanya menampilkan judul konten CMS** halaman Layanan/Untuk Perusahaan + tautan. Artikel: tanggal + judul | `[WF S01-1; IMPL Beranda.tsx]` |
| Layanan | Tujuh kartu (6 + korporat) | **Satu halaman Markdown panjang** (satu `##` per layanan) | `[WF S01-3; IMPL layanan/page.tsx]` |
| Untuk Perusahaan | Uraian + dua kartu "Perusahaan baru" / "Sudah terdaftar" | Markdown + blok "Sudah terdaftar?" dengan tombol "Masuk member area". Tanpa tombol daftar | `[WF S01-4; IMPL MasukMemberArea.tsx]` |
| Rekanan, Kontak, Unduh | Kartu logo, kartu kontak, tombol toko + alert "segera hadir" | Masing-masing satu halaman Markdown | `[WF S01-5, S01-7, S01-8; IMPL]` |
| Artikel | Kartu bergambar + ringkasan, paginasi | Kartu tanggal + judul, tombol "Artikel berikutnya" (kursor) | `[WF S01-6; IMPL Artikel.tsx]` |
| Kebijakan | Spanduk draf, versi, tanggal berlaku | Spanduk draf. "Berlaku sejak" = tanggal mulai tayang; versi selalu `[TIDAK DIKETAHUI]` | `[WF S01-9; IMPL HalamanKonten.tsx]` |
| Tidak ditemukan, galat | Ada | Ada (teks galat sedikit berbeda, Lampiran B) | `[WF S01-10; IMPL not-found.tsx, error.tsx]` |

## 5. Daftar halaman dan alur

### Tabel ringkas

| ID | Halaman | Path sekarang | Kunci konten CMS (`site_page`) | Sumber |
| --- | --- | --- | --- | --- |
| SP-00 | Kerangka bersama (header, menu, footer) | semua | — (di kode) | `[IMPL KerangkaSitus.tsx, navigasi.ts]` |
| SP-01 | Beranda | `/` | `home` (+ `services`, `for_business`, artikel) | `[IMPL page.tsx]` |
| SP-02 | Tentang Kami | `/tentang-kami` | `about` | `[IMPL navigasi.ts]` |
| SP-03 | Layanan | `/layanan` | `services` | idem |
| SP-04 | Untuk Perusahaan | `/untuk-perusahaan` | `for_business` | idem |
| SP-05 | Rekanan | `/rekanan` | `partners` | idem |
| SP-06 | Artikel (daftar) | `/artikel` | `articles` (pengantar) + konten jenis artikel | idem |
| SP-07 | Detail artikel | `/artikel/<id>` | satu artikel | `[IMPL artikel/[id]]` |
| SP-08 | Kontak | `/kontak` | `contact` | idem |
| SP-09 | Unduh aplikasi | `/unduh` | `download_app` | idem |
| SP-10 | Kebijakan privasi dan syarat | `/kebijakan` | `privacy_terms` | idem |
| SP-11 | Halaman tidak ditemukan | URL tak dikenal, konten tidak tayang | — | `[PRD US-1801 AC3]` |
| SP-12 | Galat umum | layanan konten gagal | — | `[PRD US-1801 AC7]` |

Path adalah pilihan implementasi awal `[IMPL navigasi.ts]`. Navigasi utama: tujuh tautan + tombol "Unduh aplikasi". Kebijakan ditautkan dari footer `[DG D-S01; IMPL navigasi.ts]`.

**Aturan lintas halaman:** halaman tetap yang **tidak punya versi tayang** (draf, terjadwal, berakhir, diarsipkan, terhapus) menampilkan SP-11, bukan keadaan kosong, dan tidak membocorkan keberadaan draf `[PRD US-1801 AC3; IMPL halaman.ts]`. Hanya satu versi halaman tetap yang tayang `[PRD US-1802 AC2]`.

Notasi per halaman: **Tujuan · Blok yang direkomendasikan · CMS vs kode · CTA · Keadaan · Seluler · Contoh isi demo**.

---

### SP-00 Kerangka bersama

- **Tujuan:** orientasi, akses cepat ke Unduh aplikasi dan member area.
- **Ditetapkan:** tautan "Lewati ke isi", landmark `header/nav/main/footer` berlabel, `lang="id"` `[DG D-S01]`; menu seluler berupa tombol "Menu" 48 px **dengan teks**, memakai elemen `details` tanpa skrip `[DG D-S01; IMPL KerangkaSitus.tsx]`; footer memuat pernyataan bahwa Ruang Sehat berperan sebagai platform `[DG D-S01]`; teks hak cipta "© 2026 PT. Arunika. Hak cipta dilindungi." (ditetapkan, opsi A, tanpa alamat badan usaha) `[USER 2026-11-26]`.
- **Bersyarat:** tautan footer "Masuk HR" dan blok "Sudah terdaftar?" **hilang** bila alamat member area tidak diatur (fail-closed) `[CR-032; IMPL tautan-korporat.ts]`. Desain harus tetap rapi tanpa elemen itu.
- **Blok footer** `[USULAN-DESAIN]`: kolom (1) logo + pernyataan platform + "Dioperasikan oleh PT. Arunika"; (2) Layanan (tautan ke Layanan, Unduh aplikasi); (3) Perusahaan (Untuk Perusahaan, Masuk HR, Rekanan); (4) Bantuan (Kontak, Kebijakan privasi dan syarat); baris bawah hak cipta. Akun media sosial `[TIDAK DIKETAHUI]`; bila ada, hanya tautan teks/ikon lokal, tanpa widget.
- **Header** `[USULAN-DESAIN]`: logo simbol + wordmark "Ruang Sehat"; tombol "Unduh aplikasi" sebagai satu-satunya tombol utama; "Masuk HR" sebagai tautan teks kecil di header (selain footer) perlu CR.
- **Seluler:** satu kolom, reflow 320 px tanpa gulir dua arah `[DG §7.2 A5]`.

### SP-01 Beranda

- **Tujuan:** dalam satu layar pertama, pengunjung tahu apa itu Ruang Sehat dan apa langkah berikutnya.
- **Blok yang direkomendasikan** `[USULAN-DESAIN]` (urutan):
  1. **Hero**: judul, kalimat pembuka pendek (≤ 2 kalimat), CTA ganda "Unduh aplikasi" (utama) dan "Untuk Perusahaan" (sekunder) `[DG D-S01]`, visual kuat (§7).
  2. **Pita nilai** (3 butir faktual, §7.4 "bukti kepercayaan").
  3. **Kartu tujuh layanan** dengan ikon (atau enam ubin `[DG UP-63]` + kartu korporat), tautan ke Layanan.
  4. **Cara memakai aplikasi** (3 langkah bernomor; isi dari demo Unduh aplikasi).
  5. **Blok Untuk Perusahaan** berlatar gelap/berwarna: tiga layanan korporat + alur ringkas + dua CTA.
  6. **Untuk rekanan dan tenaga kesehatan**: dua kartu kecil.
  7. **Artikel terbaru** (3 kartu).
  8. **FAQ** (opsional, perlu CR).
  9. **Banner CTA penutup**: "Unduh aplikasi Ruang Sehat".
- **CMS vs kode:** hero (judul = judul konten `home`, isi = Markdown, satu gambar opsional dengan teks alternatif) dari CMS `[IMPL Beranda.tsx]`. Judul bagian "Layanan", "Untuk Perusahaan", "Artikel terbaru" dan label tombol ada di kode. **Keterbatasan sekarang:** bagian Layanan dan Untuk Perusahaan hanya menampilkan **judul** konten CMS halaman masing-masing, bukan ringkasan atau kartu `[IMPL Beranda.tsx]`. Blok 2, 3, 4, 6, 8, 9 belum ada dan butuh CR (§10).
- **Keadaan:** konten `home` belum tayang → sekarang tampil penanda `[TIDAK DIKETAHUI]` yang terlihat publik `[IMPL Beranda.tsx]`; artikel kosong → "Belum ada artikel. [TIDAK DIKETAHUI]" `[IMPL]`. `[USULAN-DESAIN]` sembunyikan blok artikel bila kosong; jangan menampilkan penanda anti-halu di production (CR).
- **Seluler:** hero satu kolom, visual di bawah teks atau dipangkas; CTA lebar penuh `[WF S01-1b]`.
- **Contoh isi demo** `[DEMO]`: judul "Kesehatan Anda, lebih dekat dari rumah"; paragraf pembuka; tiga butir (konsultasi online di aplikasi; "Biaya jelas sejak awal. Harga yang tampil sama dengan yang Anda bayar"; "Data tersimpan di Indonesia dan dilindungi sesuai peraturan pelindungan data pribadi"). **Perlu konfirmasi:** butir harga hanya bersumber untuk MCU dan lab `[BRD BR-023]`, sedangkan produk ditambah ongkos kirim `[BRD BR-034]`; frasa "sesuai peraturan" adalah klaim kepatuhan yang menunggu legal `[PRD L-20]`. Slogan pamflet "Satu ruang untuk kesehatan Anda." belum boleh dipakai `[DG §3.5]`.

### SP-02 Tentang Kami

- **Tujuan:** kredibilitas: siapa operatornya, apa perannya, prinsipnya.
- **Blok** `[USULAN-DESAIN]`: hero halaman pendek (judul + satu kalimat + ilustrasi); "Siapa kami"; "Peran kami" sebagai diagram sederhana pasien ↔ Ruang Sehat ↔ rekanan (gateway) `[BRIEF]`; empat prinsip sebagai kartu berikon; CTA penutup.
- **CMS vs kode:** satu konten Markdown + satu gambar `[IMPL HalamanKonten.tsx]`. Diagram dan kartu prinsip butuh aset lokal atau CR.
- **Contoh isi demo** `[DEMO]`: "Tentang Ruang Sehat" berisi Siapa kami, Peran kami, Prinsip kami (Mudah dijangkau, Transparan, Aman, Bertanggung jawab). Visi, misi, sejarah, foto tim `[TIDAK DIKETAHUI]` `[WF S01-2]`.

### SP-03 Layanan

- **Tujuan:** pasien mengenali layanan yang ia butuhkan dan tahu semuanya dipakai di aplikasi.
- **Ditetapkan:** isi dan **urutan** layanan berasal dari CMS, tidak ditulis tetap di kode `[PRD US-1801 AC2]`; tanpa harga, tombol beli, atau tombol daftar `[WF S01-3]`; promo hanya informasi `[PRD US-1802 AC3]`.
- **Daftar layanan** (fakta untuk struktur): konsultasi dokter `[BRD BR-010]`, konsultasi psikolog `[BRD BR-011]` (fisioterapis dan tenaga medis juga melayani konsultasi `[BRD BR-012]`), pendaftaran klinik `[BRD BR-020]`, pendaftaran lab `[BRD BR-021]`, home visit `[BRD BR-022]`, informasi biaya MCU dan lab `[BRD BR-023]`, belanja produk `[BRD BR-030]`, ditambah layanan korporat `[BRD BR-130, BR-133]`. Ubin "Vaksinasi" dan "Suntik & Infus Vitamin" dari pamflet **tidak** tampil untuk pasien `[DG UP-63]`.
- **Fakta pendukung per layanan** yang boleh dipakai: resep digital ditebus di apotek mana pun, obat resep tidak dijual `[BRD BR-015, BR-032]`; video tidak direkam `[BRD BR-016]`; cukup menunjukkan appointment saat datang `[BRD BR-024]`; hasil lab dikirim ke aplikasi `[BRD BR-025]`; antrean real-time setelah check-in `[BRD BR-027]`; produk dikirim kurir pihak ketiga, ongkir dibayar pasien `[BRD BR-034]`.
- **Blok** `[USULAN-DESAIN]`: grid kartu berikon (judul, 1–2 kalimat, 2–3 poin), kartu korporat berwarna beda dengan tautan ke SP-04, banner penutup "Semua layanan ada di aplikasi" + Unduh.
- **CMS vs kode sekarang:** satu Markdown panjang, setiap layanan satu `##` `[IMPL; DEMO]`. Kartu butuh CR (§10 C-02).
- **Contoh isi demo** `[DEMO]`: judul "Tujuh layanan kesehatan dalam satu aplikasi", delapan bagian (dokter, psikolog, klinik, lab, home visit, biaya MCU dan lab, belanja produk, layanan korporat). Catatan: demo memecah konsultasi menjadi dua bagian sehingga ada tujuh layanan pasien + korporat, sesuai brief.

### SP-04 Untuk Perusahaan

- **Tujuan:** HR paham penawaran, proses, dan privasi peserta, lalu daftar atau masuk.
- **Blok** `[USULAN-DESAIN]`: hero perusahaan (judul, kalimat, CTA "Daftarkan perusahaan" + "Masuk member area"); tiga kartu layanan korporat; pilihan lokasi (on-site / di rekanan) `[BRD BR-135]`; **alur 5 langkah bernomor** (daftar dan verifikasi → pilih paket → bayar invoice → undang peserta → pantau kehadiran, urutan lihat Lampiran B); blok "Privasi peserta terjaga"; daftar dokumen pendaftaran; blok dua kartu "Perusahaan baru" / "Sudah terdaftar" `[WF S01-4]`; FAQ (opsional).
- **Ditetapkan:** dokumen pendaftaran: NIB 13 digit, NPWP 16 digit, surat kuasa atau surat penugasan dari direksi sebagai bukti kewenangan `[USER 2026-11-26 Q6, Q9]`; PDF bertanda tangan digital ditolak, yang diunggah adalah hasil pindaian `[USER 2026-11-26 Q12]`; perusahaan diverifikasi oleh Verifikator Ruang Sehat `[BRD BR-136]`; harga dan katalog tidak tampil di situs `[WF S01-4]`.
- **CMS vs kode:** uraian dari CMS; blok "Sudah terdaftar?" dan tombol "Masuk member area" di kode `[IMPL MasukMemberArea.tsx]`. Tombol "Daftarkan perusahaan" menyusul F1-T103 `[CR-032]`. Pendaftaran berlangsung di web korporat, bukan di situs `[WF S01-4]`.
- **Keadaan:** alamat member area tidak diatur → blok "Sudah terdaftar?" tidak tampil `[CR-032]`.
- **Contoh isi demo** `[DEMO]`: "Layanan kesehatan untuk karyawan perusahaan": layanan, Cara kerjanya (5 langkah), Privasi peserta terjaga, Dokumen pendaftaran, surel `korporat@ruangsehat.example` (**placeholder**, domain contoh). Kalimat "Harga dan katalog paket tersedia di member area setelah perusahaan terverifikasi" berlaku setelah Fase 3 `[BRIEF-K §4]`.

### SP-05 Rekanan

- **Tujuan:** calon rekanan tertarik bergabung; pasien melihat jaringan rekanan.
- **Ditetapkan:** daftar rekanan **ditulis di CMS**, tidak diambil otomatis dari data rekanan `[DG D-S01]`; logo wajib teks alternatif (nama rekanan) `[WF S01-5]`.
- **Blok** `[USULAN-DESAIN]`: hero "Tumbuh bersama Ruang Sehat"; manfaat (kartu berikon); jenis rekanan (klinik, lab, apotek, toko kecantikan, toko alat kesehatan) `[BRD BR-031]`; dinding logo rekanan (hanya yang sudah memberi izin, `[TIDAK DIKETAHUI]`); "Cara bergabung" 3 langkah; CTA kanal kemitraan.
- **Keadaan:** belum ada rekanan yang boleh tampil → wireframe "Daftar rekanan segera hadir" + tautan ke Kontak `[WF S01-5]`; implementasi sekarang tidak punya keadaan ini (seluruh halaman satu Markdown).
- **Contoh isi demo** `[DEMO]`: manfaat, jenis rekanan, cara bergabung, surel `rekanan@ruangsehat.example` (placeholder). **Perlu konfirmasi:** kalimat "bagian rekanan diteruskan langsung ke rekening rekanan" bergantung model pembagian dana yang menunggu legal `[BRD L-04]`. Tidak ada nama rekanan; rekanan belum ada yang bergabung saat brief proyek ditulis `[BRIEF]`.

### SP-06 Artikel (daftar) dan SP-07 Detail artikel

- **Tujuan:** edukasi kesehatan dan alasan kembali ke situs.
- **Model konten:** judul, isi Markdown, satu gambar dengan teks alternatif, kategori, tanggal mulai tayang `[IMPL tipe.ts; openapi public]`. **Tidak ada field ringkasan.** Daftar memakai kursor ("Artikel berikutnya"), jumlah total tidak tersedia `[IMPL Artikel.tsx]`.
- **Daftar** `[USULAN-DESAIN]`: satu artikel unggulan besar + grid kartu (gambar, kategori sebagai chip, judul, tanggal); kategori sebagai filter perlu CR. Sekarang kartu hanya tanggal + judul `[IMPL]`.
- **Detail** `[USULAN-DESAIN]`: lebar baca ±68 karakter `[WF S01-2]`, gambar utama, kategori, tanggal, isi dengan tipografi bacaan yang nyaman, blok CTA "Konsultasi lewat aplikasi" di akhir, tautan "Semua artikel". Tanpa komentar, tombol bagikan pihak ketiga, atau pelacak `[WF S01-6]`.
- **Keadaan:** kosong → "Belum ada artikel" + kalimat + tombol ke Beranda `[WF S01-6]`; artikel tidak tayang → SP-11 `[PRD US-1801 AC3]`.
- **Contoh isi demo** `[DEMO]`: tiga artikel kategori "Tips layanan" (kapan memilih konsultasi online; persiapan pemeriksaan lab; memakai resep digital). Isi kesehatan membutuhkan peninjau medis `[TIDAK DIKETAHUI]`.

### SP-08 Kontak

- **Ditetapkan:** tanpa formulir dan tanpa peta sematan `[KEPUTUSAN-271]`; keluhan pasien lewat aplikasi `[WF S01-7]`.
- **Blok** `[USULAN-DESAIN]`: tiga kartu per audiens (Pengguna aplikasi, Perusahaan, Calon rekanan) dengan ikon dan kanal masing-masing; kartu alamat kantor (teks); kotak peringatan keamanan "jangan pernah mengirim password, kode OTP, atau nomor kartu" (dari demo).
- **Batasan:** tautan di konten CMS hanya `https`, `mailto`, atau relatif. Tautan `tel:` (ketuk untuk menelepon) **tidak** dirender sebagai tautan `[PRD US-1802 AC6; IMPL aturan-aman.ts]`, jadi nomor telepon tampil sebagai teks. Tautan `https` ke kanal pesan pihak ketiga hanya berupa tautan keluar biasa, bukan sematan; kanal resminya `[TIDAK DIKETAHUI]`.
- **Contoh isi demo** `[DEMO]`: tiga surel `.example` (placeholder), tanpa telepon, jam layanan, dan alamat, semuanya `[TIDAK DIKETAHUI]`.

### SP-09 Unduh aplikasi

- **Ditetapkan:** sebelum terbit di toko tampil "Aplikasi segera hadir" tanpa tautan palsu `[DG D-S01]`; lencana toko adalah aset lokal setelah aturan merek toko diperiksa (Tahap 12) `[WF S01-8]`.
- **Blok** `[USULAN-DESAIN]`: hero dengan mockup ponsel; dua tombol toko; "Mulai dalam tiga langkah" bernomor; kartu "Untuk tenaga kesehatan".
- **CMS vs kode sekarang:** satu Markdown `[IMPL unduh/page.tsx]`; tombol toko belum ada. Tautan toko `[TIDAK DIKETAHUI]` (Tahap 12).
- **Contoh isi demo** `[DEMO]`: tiga langkah (daftar + verifikasi dua langkah; lengkapi profil, NIK dan foto KTP untuk konsultasi, home visit, klinik, lab, tidak untuk produk `[BRIEF]`; pilih layanan dan bayar), anggota keluarga, aplikasi tenaga kesehatan, "Tautan toko aplikasi akan tersedia saat aplikasi dirilis."

### SP-10 Kebijakan privasi dan syarat

- **Ditetapkan:** versi dan tanggal berlaku tercantum; teks final menunggu L-08, L-20, PT-24, dan sampai itu ditandai draf `[PRD US-1801 AC4]`. Spanduk draf bernada peringatan dengan teks `[WF S01-9]`. Tanpa spanduk persetujuan cookie karena tidak ada cookie `[DG D-S01]`.
- **Blok** `[USULAN-DESAIN]`: kepala (judul, versi, tanggal berlaku), daftar isi bertaut, dua bagian (privasi, syarat), lebar baca nyaman.
- **Keterbatasan:** model konten tidak punya field versi, dan "Berlaku sejak" memakai tanggal mulai tayang `[IMPL HalamanKonten.tsx]`. Daftar isi bertaut butuh ID judul (CR).
- **Contoh isi demo** `[DEMO]`: ringkasan kebijakan privasi dan syarat, **draf tanpa persetujuan legal**.

### SP-11 Halaman tidak ditemukan dan SP-12 Galat umum

- **SP-11:** "Halaman tidak ditemukan" + "Halaman yang Anda cari tidak tersedia." + "Ke Beranda", **identik** untuk URL tak dikenal dan konten tidak tayang `[PRD US-1801 AC3; IMPL not-found.tsx]`.
- **SP-12:** tanpa jejak teknis dan tanpa kode referensi `[PRD US-1801 AC7; KEPUTUSAN-272]`; tombol "Coba lagi" dan "Ke Beranda" `[IMPL error.tsx]`. Teks sekarang "Terjadi kesalahan / Halaman belum dapat ditampilkan…" `[IMPL]`.
- `[USULAN-DESAIN]` ilustrasi kecil (lingkaran mint + ikon, pola DG §5.2) dan tautan ke halaman populer (Layanan, Unduh, Kontak) di SP-11.

## 6. Aturan keamanan, privasi, dan konten yang memengaruhi UI

| # | Aturan | Dampak desain | Sumber |
| --- | --- | --- | --- |
| S1 | Tanpa cookie sesi, tanpa data pribadi dimuat | Tidak ada spanduk cookie, "ingat saya", personalisasi, atau keranjang | `[PRD US-1801 AC5; DG D-S01]` |
| S2 | Tanpa formulir di situs | Tidak ada formulir kontak, newsletter, pendaftaran, pencarian berbasis server baru tanpa CR | `[KEPUTUSAN-271; WF S01-4]` |
| S3 | CSP: tanpa skrip inline dan tanpa sumber pihak ketiga; gambar, font, dan koneksi hanya dari situs sendiri; tidak boleh dibingkai | Tidak ada Google Fonts, peta, video YouTube, widget chat, analitik, tombol bagikan, iframe. Semua ikon, font, dan ilustrasi di-host sendiri | `[PRD US-1802 AC9; ADR-027 b.5; IMPL keamanan.ts]` |
| S4 | Tanpa atribut `style` sebaris | Gaya harus bisa diwujudkan lewat kelas stylesheet; tidak ada warna/tata letak khusus per konten CMS | `[IMPL situs.css]` |
| S5 | Markdown CommonMark tanpa ekstensi dan tanpa HTML mentah | Lihat §9: tidak ada tabel, kolom, tombol, video, atau warna dari CMS | `[ADR-027 b.5; IMPL Markdown.tsx]` |
| S6 | Gambar hanya dari endpoint gambar publik, teks alternatif wajib; JPG/PNG dikode ulang tanpa metadata | Ilustrasi/foto dari CMS berupa JPG/PNG. SVG hanya sebagai aset kode | `[PRD US-1802 AC5, AC6; IMPL aturan-aman.ts; backend content_support]` |
| S7 | Hanya konten Tayang yang tampil | Tidak ada teaser untuk konten draf | `[PRD US-1801 AC3]` |
| S8 | Tanpa harga dan tombol beli; promo hanya informasi | Tidak ada kartu harga, badge diskon, atau "Pesan sekarang" ke layanan | `[WF S01-3; PRD US-1802 AC3, US-1101 AC4]` |
| S9 | Tautan ke member area: host terpisah, hilang bila alamat tidak diatur | Tata letak tetap seimbang tanpa blok itu | `[CR-032]` |
| S10 | Klaim harus bersumber | Tidak ada angka pengguna, rating, testimoni, sertifikasi, atau logo media yang tidak nyata (§7.4) | anti-halu proyek |
| S11 | Tidak ada fitur di luar scope | Tidak ada BPJS, asuransi, konsultasi suara, chatbot, langganan, login Google/Apple, rating | `[BRIEF; DG P1]` |

## 7. Arah visual dan kreatif

Bagian ini menjawab arahan user `[USER 2026-11-26]`. Semua isinya `[USULAN-DESAIN]`, kecuali kalimat berlabel lain.

### 7.1 Diagnosis: kenapa tampilan sekarang terasa kaku

Hasil pemeriksaan `web/situs/src/app/situs.css`, `komponen/*.tsx`, dan `web/packages/ui/src/ui.css` (commit `9475fda`):

| # | Temuan | Bukti di kode | Akibat visual |
| --- | --- | --- | --- |
| D1 | Hampir semua halaman adalah **satu kolom artikel Markdown** | `HalamanKonten` merender judul + satu gambar opsional + Markdown dalam `.situs-baca` (lebar maks 780 px) untuk tujuh halaman | Situs terasa seperti dokumen teks, bukan company profile. Tidak ada seksi, ritme, atau visual |
| D2 | Hero polos | `.situs-hero`: latar mint rata, grid `1.2fr 1fr`; kolom gambar hanya dirender bila konten punya gambar | Tanpa gambar, kolom kanan kosong dan hero berat ke kiri. Seluruh isi Markdown `home` (paragraf + daftar butir) masuk ke hero, sehingga hero padat teks |
| D3 | Bingkai gambar tampak seperti placeholder | `.situs-gambar`: garis putus-putus `border.control`, latar `bg.neutral`, padding | Gambar asli pun tampil di dalam kotak putus-putus wireframe |
| D4 | Beranda tanpa kartu layanan | `Beranda.tsx`: bagian Layanan dan Untuk Perusahaan hanya berisi **judul** konten CMS + tautan | Inti penawaran (tujuh layanan) tidak terlihat di Beranda |
| D5 | Tanpa ikon, ilustrasi, logo simbol | Logo = teks 18 px tebal (`.situs-logo`); tidak ada SVG di komponen situs | Tidak ada identitas visual dan titik fokus mata |
| D6 | Hierarki tipografi datar | Teks isi 14/20 (`--t-body-size`), judul halaman 28/36, h2 Markdown 24/32; hero 36/44 dan 18/28 di luar skala DG | Jarak ukuran judul-isi kecil. Teks isi 14 px kecil untuk situs pemasaran |
| D7 | Ritme spasi seragam | Semua seksi `.situs-sec` padding 32/24 px, tanpa kontainer lebar maksimum di Beranda | Seksi tidak bernapas, dan di layar lebar baris teks Beranda membentang penuh |
| D8 | Kartu seragam dan diam | `.rs-card`: putih, radius 16, `elevation.1`; kartu artikel hanya tanggal + judul; tanpa hover kecuali varian `pressable` | Semua kartu sama berat, tidak ada yang menonjol |
| D9 | Latar seksi monoton | Hanya dua variasi: `bg.app` dan `.situs-sec--alt` (putih + garis) | Tidak ada kontras antarseksi, halaman panjang terasa rata |
| D10 | Tanpa gerak | Tidak ada `transition` atau animasi di `situs.css` | Interaksi terasa mati |
| D11 | Penanda anti-halu terlihat publik | `TidakDiketahui` menampilkan "[TIDAK DIKETAHUI]" kuning saat konten kosong | Tampak belum selesai bila CMS kosong |
| D12 | Footer tipis | Latar `text.primary`, tiga kolom berisi sedikit tautan | Penutup halaman tidak memberi arah |

Kesimpulan: masalahnya bukan palet DG, melainkan **struktur halaman** (semua halaman dirender sebagai satu artikel Markdown), **tidak adanya aset visual**, dan **skala tipografi/spasi aplikasi** yang dipakai untuk situs pemasaran. Karena itu sebagian besar perbaikan butuh perubahan kode (§10), bukan hanya CSS.

### 7.2 Tiga konsep visual

Semua konsep memakai token DG §3.1 dan pasangan kontras yang sudah lulus di DG §3.2. Pasangan baru wajib dihitung dengan rumus DG §3.2.

**Konsep A: "Ruang Teduh" (direkomendasikan)**
- Kata kunci: tenang, lapang, hangat-klinis, organik, ramah.
- Palet: dominan `bg.app #FAFBF7` dan putih; bidang besar mint `#DFF4ED`; aksen `primary #087F78`; seksi gelap `primaryStrong #065F59`. Gradien lembut `bg.app → mint` untuk hero (teks `text.primary`: 14,14 dan 12,81, lulus di kedua ujung; ketiga kanal turun searah, jadi semua titik antara ikut lulus) `[DG §3.2 K1]`. Teks merek di atas mint memakai `primaryStrong` (6,56), **bukan** `primary` (4,24, gagal) `[DG §3.2 K4, K6]`.
- Bentuk/ornamen: bentuk daun dan lengkung rumah dari arah logo pamflet `[DG §3.5]` sebagai blob dekoratif besar di belakang visual hero; sudut membulat 16–24; garis putus halus sebagai "jalur" yang menghubungkan langkah bernomor.
- Ilustrasi: flat duotone teal–mint dengan sedikit garis, tokoh abstrak tanpa wajah realistis (aman dari kesan pasien nyata).
- Foto: opsional, hanya untuk Tentang Kami dan Untuk Perusahaan, berlisensi dengan model release.
- Hero Beranda: kiri judul besar 2 baris + kalimat pendek + CTA ganda + tiga "chip nilai" faktual; kanan mockup ponsel berisi layar aplikasi bergaya (ilustrasi UI, bukan data nyata) dengan dua kartu kecil mengambang ("Konsultasi chat dan video", "Biaya MCU dan lab terlihat sebelum daftar") di atas blob daun mint.
- Alasan rekomendasi: sesuai palet pamflet yang sudah disetujui `[DG UP-58]`; tidak bergantung pada foto berlisensi yang belum ada; ilustrasi SVG/PNG ringan sehingga mendukung p95 ≤ 3 detik; nada tenang cocok untuk pasien yang sulit datang ke fasilitas.

**Konsep B: "Presisi Klinis"**
- Kata kunci: modern, tegas, terpercaya, data, institusional.
- Palet: putih dominan, blok besar `primaryStrong` dengan teks putih (7,53) `[DG §3.2 K7]`, garis grid tipis `border.divider` (dekoratif saja), aksen mint kecil.
- Bentuk: grid tegas, sudut 8–12, garis lurus, angka besar bernomor.
- Visual: foto profesional (tenaga kesehatan, ruang klinik) berlisensi dengan model release; ikon garis tipis.
- Hero Beranda: panel kiri `primaryStrong` dengan judul putih dan CTA (tombol putih bergaris), panel kanan foto penuh.
- Cocok untuk halaman Untuk Perusahaan dan Rekanan (audiens B2B). Risiko: sangat bergantung foto berlisensi `[TIDAK DIKETAHUI]` dan terasa dingin bagi pasien.

**Konsep C: "Hangat Sehari-hari"**
- Kata kunci: keluarga, dekat, ceria, sehari-hari.
- Palet: DG + satu **aksen hangat baru** (mis. koral atau kuning pasir). Nilai aksen `[TIDAK DIKETAHUI]`, butuh revisi DG §3.1, hitung kontras, dan persetujuan user. Jangan memakai warna status (`warningBg`, `dangerBg`) sebagai dekorasi, karena warna itu bermakna status `[DG §5.6]`.
- Visual: ilustrasi karakter keluarga lintas usia, bentuk bulat besar.
- Risiko: menambah token merek, kurang cocok untuk audiens B2B.

**Rekomendasi:** Konsep A untuk seluruh situs, dengan **seksi gelap ala Konsep B** (latar `primaryStrong`, teks putih) untuk blok Untuk Perusahaan di Beranda dan hero halaman Untuk Perusahaan, agar audiens HR mendapat nada yang lebih institusional.

### 7.3 Tipografi

- **Font:** rilis 1 memakai font sistem `[DG §3.3 KEPUTUSAN-164]`. Opsi merek **Plus Jakarta Sans** berlisensi SIL Open Font License 1.1 dan tersedia sebagai font variabel (`wght`) `[SRC-259]` `[DG UP-59]`. Usulan: satu keluarga Plus Jakarta Sans untuk judul dan isi (karakter geometris-humanis karya desainer Indonesia `[SRC-259]`), di-host sendiri sebagai berkas di situs (CSP mengizinkan `font-src 'self'` `[IMPL keamanan.ts]`), dengan font sistem sebagai cadangan. Membutuhkan keputusan user (UP-59 masih terbuka) dan CR. Pasangan font kedua tidak diusulkan karena lisensinya belum diverifikasi `[TIDAK DIKETAHUI]`.
- **Beban:** berkas font menambah ukuran halaman. Batasi satu berkas variabel dengan subset huruf Latin; cara bundel dan ukurannya diverifikasi di Tahap 8 `[DG §3.3]`.
- **Skala situs** (token baru khusus situs, butuh revisi DG §3.3):

| Token usulan | Desktop | Seluler (< 768) | Pemakaian |
| --- | --- | --- | --- |
| `site.hero` | 48/56, 700 | 32/40, 700 | Judul hero Beranda |
| `site.h1` | 36/44, 700 | 28/36, 700 | Judul halaman |
| `site.h2` | 28/36, 700 | 24/32, 700 | Judul seksi |
| `site.h3` | 20/28, 600 | 18/26, 600 | Judul kartu |
| `site.lead` | 18/28, 400 | 17/26, 400 | Kalimat pembuka di bawah judul |
| `site.body` | 16/26, 400 | 16/26, 400 | Teks isi dan artikel |
| `site.eyebrow` | 13/18, 700, huruf kapital, spasi huruf 0,08em | sama | Label kecil di atas judul seksi ("LAYANAN") |

  Aturan DG tetap berlaku: tinggi baris ≥ 1,3× ukuran, kontainer teks tanpa tinggi tetap (spasi teks pengguna tidak memotong isi), teks dapat diperbesar 200% `[DG §3.3, §7.2 A4, A6]`. Lebar baca isi 60–72 karakter.

### 7.4 Pola komponen yang menarik dan tetap terpercaya

| Pola | Rancangan | Batasan wajib |
| --- | --- | --- |
| **Hero dengan visual kuat** | Gradien lembut + blob daun + mockup ponsel atau ilustrasi; CTA ganda (satu utama, satu sekunder) `[DG §4 "satu tombol utama per layar"]` | Gambar dari CMS wajib teks alternatif; ilustrasi dekoratif disembunyikan dari pembaca layar `[DG §3.5]` |
| **Kartu layanan berikon** | Ikon dalam lingkaran mint 48–56 px (ikon `primaryStrong`), judul, 1–2 kalimat, tautan "Selengkapnya"; seluruh kartu dapat diklik dengan cincin fokus | Dikenali dari label, bukan warna (latar mint vs latar halaman hanya 1,10) `[DG §3.2 N5]` |
| **Langkah bernomor** | Angka besar dalam lingkaran `primary` + teks putih (4,86), garis penghubung putus-putus; horizontal di desktop, vertikal di seluler | Urutan tetap terbaca sebagai daftar bernomor oleh pembaca layar |
| **Seksi berlatar** | Ritme: `bg.app` → putih → gradien mint → `primaryStrong` (gelap) → `bg.app`. Teks di seksi gelap putih (7,53) atau mint (6,56) | Hanya pasangan yang sudah lulus DG §3.2 |
| **Pita nilai / bukti kepercayaan** | 3–4 butir faktual dengan ikon: "Data disimpan di Indonesia" `[BRIEF]`; "Tenaga kesehatan diverifikasi dokumen izinnya sebelum melayani" `[PRD US-203]`; "Biaya MCU dan lab terlihat sebelum mendaftar" `[BRD BR-023]`; "Panggilan video tidak direkam" `[BRD BR-016]` | Tidak ada angka pengguna, rating, peringkat, sertifikasi, atau logo media. Target KPI (mis. 10.000 user aktif `[BRIEF]`) adalah **target**, bukan pencapaian, dan tidak boleh tampil sebagai klaim. Klaim publik tetap butuh persetujuan user |
| **Mockup aplikasi** | Bingkai ponsel berisi tangkapan layar aplikasi nyata saat sudah ada, atau ilustrasi UI bergaya | Tanpa nama, foto, atau data yang terlihat seperti pasien/dokter nyata; tanpa harga |
| **Testimoni** | Komponen boleh dirancang tetapi **tidak tampil** sampai ada testimoni nyata dengan persetujuan tertulis | Testimoni `[TIDAK DIKETAHUI]`; dilarang mengarang |
| **Dinding logo rekanan** | Logo grayscale yang berwarna saat hover/fokus, dengan nama sebagai teks alternatif | Hanya rekanan dengan izin logo `[TIDAK DIKETAHUI]` |
| **FAQ** | Akordeon pertanyaan (elemen `details`, tanpa skrip, seperti menu seluler) | Butuh konvensi konten + kode (CR). Jawaban tidak boleh memuat janji di luar sumber |
| **Banner CTA penutup** | Bidang `primaryStrong` lebar penuh, judul putih, tombol putih bergaris + gambar ponsel kecil | Satu tombol utama |
| **Footer kaya** | Empat kolom (SP-00), latar `text.primary` + teks putih (14,70) `[DG §3.2 K13]` | Tanpa widget sosial pihak ketiga |
| **Kartu artikel** | Gambar 16:9, chip kategori, judul 2 baris, tanggal | Model konten tanpa ringkasan (§9) |

### 7.5 Ikon, ilustrasi, dan foto

- **Ikon:** SVG sebaris disimpan lokal di paket UI `[DG §3.5]`. Sumber set ikon dan lisensinya `[TIDAK DIKETAHUI]` `[DG §3.5]`. Kandidat set ikon sumber terbuka (mis. Lucide, Tabler Icons, Phosphor) **belum diverifikasi**: lisensi, versi, dan registry wajib dicek ke sumber resmi dan dicatat di `docs/sources.md` sebelum dipakai. Brief ini tidak mengklaim versi atau lisensi apa pun. Pakai satu set dengan ketebalan garis konsisten. Ikon yang dibutuhkan: tujuh layanan + korporat, tiga layanan korporat, manfaat rekanan, kontak, langkah.
- **Ilustrasi:** usulan dibuat khusus oleh desainer (hak milik PT. Arunika) dengan gaya Konsep A. DG saat ini membatasi ilustrasi bergambar orang sampai ada aset berlisensi `[DG §3.5]`; setelah aset itu ada, batasan ini perlu direvisi di DG. Ekspor untuk CMS sebagai PNG (CMS hanya menerima JPG/PNG `[IMPL backend content_support]`); ornamen dekoratif sebagai SVG di kode.
- **Foto:** hanya berlisensi dengan model release. Hindari foto stok yang mengesankan pasien atau dokter Ruang Sehat yang nyata, logo rumah sakit nyata, tindakan medis yang mengganggu, atau stereotip. Sumber foto `[TIDAK DIKETAHUI]`.
- **Logo:** logo final dan ikon situs belum ada `[DG PT-40]`. Selama itu pakai placeholder "draf logo". Nama selalu "Ruang Sehat" dengan spasi `[DG §3.5]`.

### 7.6 Mikro-interaksi dan animasi

- CSS saja, tanpa pustaka animasi atau skrip pihak ketiga `[PRD US-1802 AC9]`.
- Kartu: naik 2–4 px + bayangan `elevation.2` saat hover/fokus, transisi 150–200 ms. Tombol: transisi warna latar. Tautan: garis bawah yang tumbuh.
- Hero: muncul halus (fade + geser 8–16 px) satu kali saat halaman dimuat. Tidak ada animasi berulang, karusel otomatis, atau video latar.
- Hormati preferensi sistem "kurangi gerakan" (media query `prefers-reduced-motion`): semua gerak dimatikan, hanya perubahan warna yang tersisa. Dukungan peramban diverifikasi di Tahap 8.
- Fokus keyboard selalu terlihat (cincin 2 px `primaryStrong` + jarak 2 px) dan tidak tertutup header lengket `[DG §7.2 A8]`.
- Animasi yang dipicu gulir membutuhkan JavaScript sendiri. Tidak diusulkan untuk rilis ini.

### 7.7 Tata letak dan spasi

- Kontainer: lebar isi maksimum ±1200 px untuk seksi, ±720 px untuk teks bacaan; margin samping 24 (desktop) dan 16 (seluler) `[DG §3.4]`.
- Jarak vertikal antarseksi 64–96 px di desktop dan 48–64 px di seluler. Nilai di atas 64 belum ada di token DG (`space` maksimum 64 `[DG §3.4]`), jadi perlu revisi DG.
- Grid kartu 3–4 kolom (desktop), 2 kolom (768–1279), 1 kolom (< 768) `[DG §8]`.
- Header boleh lengket dengan latar putih dan bayangan tipis setelah digulir, selama tidak menutupi elemen yang difokus `[DG §7.2 A8]`.

## 8. Sistem desain (dasar dari DG)

Nilai berikut sudah ada di design guideline v1.2 dan disetujui user sebagai arah (UP-58–UP-64) `[DG §13]`. §7 memperluasnya untuk situs.

- **Warna** `[DG §3.1]`: `primary #087F78`, `mint #DFF4ED`, `bg.app #FAFBF7`, `surface #FFFFFF`, `primaryStrong #065F59`, `text.primary #1A2B2E`, `text.secondary #5C6B6F`, `border.control #7A898C`, `border.divider #DCE3E1` (dekoratif saja), `bg.neutral #EEF1F0`, status danger/success/warning. Mode gelap tidak dirancang di rilis 1 `[DG UP-61]`. Nilai mint yang benar masih ditanyakan `[DG PT-41]`.
- **Kontras** `[DG §3.2]`: semua teks ≥ 4,5:1 (tanpa pengecualian teks besar); komponen ≥ 3:1; `primary` tidak dipakai untuk teks di atas mint.
- **Spasi/radius/elevasi** `[DG §3.4]`: kelipatan 4 (4–64); radius md 8, lg 12, xl 16, full 999; kartu `elevation.1`, menu `elevation.2`.
- **Komponen** `[DG §4]`: tombol (utama tidak dinonaktifkan untuk validasi, satu tombol utama per layar), kartu, ubin layanan, badge, peringatan, kerangka. Pustaka UI pihak ketiga tidak dipakai; komponen dibangun sendiri `[DG §2 KEPUTUSAN-163]`.
- **Bahasa** `[DG §6]`: Bahasa Indonesia, sapaan "Anda", tombol kata kerja, istilah PRD apa adanya ("rekanan", "tenaga kesehatan").
- **Aksesibilitas** `[DG §7.2, D-S01]`: judul bertingkat per halaman, landmark berlabel, target ≥ 40 px (desktop) dan 48 px (sempit), fokus terlihat, reflow 320 px, teks diperbesar 200%, gambar bermakna wajib teks alternatif.
- **Responsif** `[DG §8]`: ≥ 1280, 768–1279, < 768 hingga 320 px. Implementasi situs saat ini memakai satu titik henti 768 px `[IMPL situs.css]`.

## 9. Batasan teknis untuk desainer

- **Tumpukan:** Next.js, dirender server per permintaan dari endpoint konten publik; token sebagai CSS custom properties di paket UI bersama `[ADR-027 b.1; IMPL layout.tsx]`.
- **Model konten CMS per item:** judul, isi Markdown, **satu** gambar (URL + teks alternatif), kategori, tanggal mulai tayang, halaman tujuan `[IMPL tipe.ts; openapi public]`. Tidak ada field ringkasan, urutan, subjudul, label tombol, ikon, versi, atau warna. Desain yang membutuhkan data terstruktur (kartu per layanan, logo per rekanan, tombol toko) butuh salah satu dari: (a) **konvensi Markdown** yang diurai kode (mis. setiap `##` = satu kartu, paragraf pertama = ringkasan), atau (b) field/jenis konten baru di CMS. Keduanya CR (§10).
- **Markdown yang tersedia** `[IMPL Markdown.tsx]`: paragraf, judul (`#` dirender sebagai h2), tebal, miring, kode, daftar berbutir dan bernomor, kutipan, garis pemisah, tautan, gambar sebaris. **Tidak tersedia:** HTML, tabel, kolom, tombol, video, warna teks, ID judul untuk tautan jangkar. Tautan hanya `https`, `mailto`, relatif (tanpa `tel:`) `[PRD US-1802 AC6]`.
- **Gambar:** JPG/PNG, dikode ulang tanpa metadata `[IMPL backend content_support]`; batas unggahan foto ≤ 5 MiB `[PRD NFR-SEC-08]`. Tidak ada varian ukuran atau titik potong otomatis `[TIDAK DIKETAHUI]`, sehingga desain harus tahan rasio gambar apa pun (gunakan `object-fit`/bingkai rasio tetap) dan desainer memberi panduan dimensi unggahan untuk admin.
- **Skrip:** situs memakai JavaScript milik sendiri yang minimal; menu seluler dan FAQ usulan memakai `details` tanpa skrip `[IMPL KerangkaSitus.tsx]`. Hindari pola yang wajib JavaScript (karusel, modal, tab dinamis).
- **Performa:** p95 ≤ 3 detik `[PRD NFR-PERF-04]`. Anggaran usulan: gambar hero ≤ 200 KB setelah kompresi, satu berkas font variabel, ilustrasi dekoratif sebagai SVG sebaris kecil `[USULAN-DESAIN]`.
- **Peramban:** dua versi stabil terakhir Chrome, Edge, Firefox, Safari `[PRD NFR-COMP-03]`.
- **Judul tab:** "<Judul halaman> - Ruang Sehat" `[IMPL layout.tsx]`.
- **Cache:** respons konten publik boleh di-cache dengan TTL pendek `[ADR-027 b.5]`. Perubahan CMS mungkin tidak langsung terlihat.

## 10. Peluang perbaikan dan kandidat change request

| # | Usulan `[USULAN-DESAIN]` | Jenis perubahan | Catatan |
| --- | --- | --- | --- |
| C-01 | Arah visual Konsep A: skala tipografi situs, ritme seksi, gradien, kontainer, footer kaya, hover/transisi | CSS + revisi DG §3.3/§3.4 | Tidak mengubah konten |
| C-02 | Beranda dan Layanan menampilkan **kartu tujuh layanan berikon**, diurai dari Markdown `services` (`##` = kartu, paragraf pertama = ringkasan), urutan tetap dari CMS | Kode | Memenuhi PRD US-1801 AC2. Ikon dipetakan per urutan atau per kata kunci judul (risiko salah peta bila admin mengganti judul); alternatif: field ikon di CMS |
| C-03 | Hero Beranda dengan visual tetap (mockup/ilustrasi) bila CMS tanpa gambar, dan perbaikan bingkai gambar (D3) | Kode + aset | Ilustrasi berlisensi `[TIDAK DIKETAHUI]` |
| C-04 | Pita nilai, langkah bernomor, banner CTA penutup, kartu rekanan/nakes di Beranda | Kode (teks di kode atau CMS) | Teks pita nilai butuh persetujuan klaim |
| C-05 | Untuk Perusahaan: hero gelap, tiga kartu layanan, alur 5 langkah (gaya `ol` khusus), blok dua kartu; tombol "Daftarkan perusahaan" | Kode | Tombol daftar sudah dijadwalkan F1-T103 `[CR-032]` |
| C-06 | Kartu artikel dengan gambar dan chip kategori; artikel unggulan | Kode | Data sudah ada di model. Ringkasan butuh field baru atau paragraf pertama |
| C-07 | Rekanan: dinding logo dari daftar gambar Markdown; keadaan "segera hadir" | Kode | Butuh izin logo |
| C-08 | Unduh: tombol toko dengan lencana lokal; tautan dari pengaturan server (pola CR-032) | Kode + aset | Tahap 12 |
| C-09 | Kontak: kartu per audiens dari `##` | Kode | — |
| C-10 | FAQ akordeon dari konvensi Markdown | Kode | Tempat (Beranda, Untuk Perusahaan) diputuskan user |
| C-11 | Ganti penanda "[TIDAK DIKETAHUI]" publik dengan keadaan kosong DG §5.2 | Kode | Menjaga anti-halu di dev, rapi di production |
| C-12 | Kebijakan: field versi dan tanggal berlaku, daftar isi bertaut (ID judul) | Kode + backend + CMS | Memenuhi PRD US-1801 AC4 penuh |
| C-13 | Font merek Plus Jakarta Sans di-host sendiri | Kode + keputusan UP-59 | `[SRC-259]` |
| C-14 | Ikon SVG lokal (set terverifikasi) | Kode + `docs/sources.md` | Lisensi dicek dulu |
| C-15 | Header: "Masuk HR" juga di header; breadcrumb `[WF S01-2]` | Kode | — |
| C-16 | Banner/promo kanal situs di Beranda | Kode | Promo boleh di situs sebagai informasi `[PRD US-1802 AC3]`, tetapi belum dirender `[IMPL]`. Kebutuhannya `[TIDAK DIKETAHUI]` |

Saran urutan bila disetujui: C-01 + C-02 + C-03 dulu (dampak visual terbesar), lalu C-05 dan C-06.

**Status (2026-11-26):** user memilih konsep A + seksi gelap ala B (Q15) dan menyetujui C-01, C-02, C-03 (Q16) `[USER 2026-11-26]`; ketiganya dibangun sebagai CR-033 (`docs/backlog.md`, KEPUTUSAN-353–359, DG v1.3 §3.3a/§3.4a). Lanjutan: Q17 menyetujui kelima teks klaim hero apa adanya (§12 butir 4 selesai untuk teks itu) dan Q18 memilih Plus Jakarta Sans di-host sendiri (C-13 dikerjakan) `[USER 2026-11-26]`. Catatan implementasi: ikon SVG buatan sendiri (bukan set C-14); pita nilai memakai tiga butir faktual tanpa klaim yang menunggu legal; penanda `[TIDAK DIKETAHUI]` publik tetap (C-11). C-04–C-16 belum dikerjakan.

**Pembaruan CR-038 (2026-11-27, arahan user `[USER]`):** menu lipat "Menu" diganti deretan tujuh tautan navigasi di header (baris kedua yang dapat digulir di bawah 1140 px); pojok kanan memuat tautan Google Play dan App Store, alamat dari env `SITUS_PLAYSTORE_URL`/`SITUS_APPSTORE_URL` (belum diketahui, fallback ke halaman Unduh). Lencana resmi menunggu aset dan persetujuan lisensi pemilik akun pengembang; sementara teks (KEPUTUSAN-420–424, SRC-945–947). Bagian kerangka di §3 dan §5 yang menyebut menu `details` di bawah 768 px dan "navigasi 8 item" berlaku sampai CR-038: kini tanpa menu lipat.

## 11. Deliverable yang diminta dari desainer (usulan)

1. **Moodboard dan dua kalimat arah** untuk Konsep A (dan B/C bila user ingin membandingkan), dengan contoh hero Beranda per konsep.
2. **Desain hi-fi** SP-00–SP-12 di desktop (1440), tablet (≈ 768), dan seluler (360, dicek di 320), termasuk keadaan: konten kosong, blok member area tidak tampil, artikel kosong, tidak ditemukan, galat.
3. **Dua versi Beranda:** "sekarang dapat dibangun" (hanya C-01, tanpa perubahan kode) dan "target" (dengan C-02–C-06), agar user dapat memilih CR.
4. **Pustaka komponen situs:** hero, kartu layanan, langkah bernomor, pita nilai, seksi berlatar, banner CTA, dinding logo, kartu artikel, FAQ, footer, header lengket, dengan semua state (default, hover, fokus, aktif).
5. **Token tambahan** (tipografi situs, spasi > 64, gradien) dengan **hasil hitung kontras** setiap pasangan baru memakai rumus DG §3.2.
6. **Set ikon** terpilih (dengan sumber dan lisensi) dan **ilustrasi** hero/seksi, plus panduan ekspor (SVG untuk kode, PNG untuk CMS).
7. **Panduan konten untuk admin CMS:** struktur Markdown per halaman (konvensi `##`), panjang judul dan kalimat ideal, dimensi dan rasio gambar, cara menulis teks alternatif.
8. **Catatan aksesibilitas** per halaman: urutan fokus, landmark, tingkat judul, teks alternatif, perilaku gerak dikurangi.
9. **Copy deck** teks di kode (label navigasi, tombol, judul seksi, keadaan kosong/galat) dengan kolom "ditetapkan / draf desainer / menunggu keputusan".
10. **Prototipe klik** Beranda → Layanan → Unduh dan Beranda → Untuk Perusahaan → member area.

Aset yang tidak boleh dibuat/diisi desainer sebagai data nyata: logo final (kecuali ditugaskan user), teks hukum, kontak, alamat, nama dan logo rekanan, angka pencapaian, testimoni, tautan toko, domain.

## 12. Aset, konten, dan fakta yang masih dibutuhkan

| # | Kebutuhan | Dipakai di | Rujukan | Pemilik |
| --- | --- | --- | --- | --- |
| 1 | Logo final, ikon situs (favicon), siapa pembuatnya | SP-00, semua | PT-40, UP-60 | User |
| 2 | Palet final (nilai mint), keputusan font merek | Semua | PT-41, UP-59 | User, desainer |
| 3 | Slogan dipakai atau tidak | Hero | PT-45 sisa, UP-60 | User |
| 4 | Persetujuan teks final semua halaman (demo sebagai draf) dan klaim publik (data di Indonesia, "harga sama", perlindungan data) | Semua | PT-70, L-20 | User, legal |
| 5 | Profil PT. Arunika: visi, misi, sejarah, foto tim | SP-02 | PT-70 | User |
| 6 | Kontak resmi: surel per audiens, telepon/WhatsApp, jam layanan, alamat kantor | SP-08, footer | PT-70 | User |
| 7 | Daftar rekanan yang boleh tampil + izin logo; kanal dan syarat kemitraan | SP-05 | PT-70 | User, rekanan |
| 8 | Tautan toko aplikasi pasien dan tenaga kesehatan, nama aplikasi di toko, lencana toko | SP-09 | PT-46, Tahap 12 | User |
| 9 | Domain production situs dan member area | Tautan, SP-04 | PT-46, PT-70 | User, legal |
| 10 | Teks kebijakan privasi dan syarat, versi, tanggal berlaku; perlu-tidaknya mencantumkan nomor pendaftaran PSE | SP-10, footer | L-08, L-20, PT-24 | Legal |
| 11 | Ilustrasi dan foto berlisensi (dengan model release); set ikon dan lisensinya | §7 | DG §3.5 | Desainer, user |
| 12 | Tangkapan layar aplikasi nyata untuk mockup | Hero, SP-09 | Repo mobile | Mobile developer |
| 13 | Peninjau medis untuk artikel kesehatan | SP-06/07 | `[TIDAK DIKETAHUI]` | User, konsultan medis |
| 14 | Akun media sosial resmi (bila ada) | Footer | `[TIDAK DIKETAHUI]` | User |
| 15 | Testimoni nyata dengan persetujuan (bila ingin ditampilkan) | Beranda | `[TIDAK DIKETAHUI]` | User |
| 16 | KPI situs dan cara mengukurnya tanpa analitik pihak ketiga | §2 | `[TIDAK DIKETAHUI]` | User |
| 17 | Keputusan formulir kontak (sekarang tanpa formulir) | SP-08 | KEPUTUSAN-271 | User |

---

## Lampiran A. Daftar sumber

| Kode | Dokumen | Bagian yang dipakai |
| --- | --- | --- |
| BRIEF | `docs/00-project-brief.md` | Ringkasan, scope, out-of-scope, data, KPI |
| BRD | `docs/02-brd.md` | BR-010–BR-035, BR-130–BR-145, L-04, L-20 |
| PRD | `docs/03-prd.md` v1.4 | EP-18 US-1801 AC1–AC7, US-1802 AC1–AC9; US-1101; US-202, US-203, US-205; NFR-PERF-04, NFR-COMP-03, NFR-SEC-08, NFR-LOC-01; L-20; PT-46, PT-69, PT-70 |
| ADR | `docs/adr/ADR-027-situs-publik-dan-member-area-korporat.md` | Butir 1, 2, 5, 6 |
| DG | `docs/06-design-guideline.md` v1.2 | §1–§8, §10.2 D-S01, §11 UP-58–UP-64, §12 PT-40–PT-46 |
| WF | `docs/wireframes/s01-situs-publik.html` | Layar 1–10 |
| PROTO | `docs/prototype/situs/01…10-*.html` | Sepuluh layar (isi sama dengan wireframe) |
| IMPL | `web/situs/src/` (commit `9475fda`): `navigasi.ts`, `keamanan.ts`, `app/*`, `komponen/Beranda.tsx`, `HalamanKonten.tsx`, `Markdown.tsx`, `Artikel.tsx`, `KerangkaSitus.tsx`, `MasukMemberArea.tsx`, `NavUtama.tsx`, `TidakDiketahui.tsx`, `konten/aturan-aman.ts`, `konten/tipe.ts`, `konten/tautan-korporat.ts`, `app/situs.css`; `web/packages/ui/src/ui.css`, `tokens.css`; `openapi/public.json`; backend `apps/rs-backend/src/modules/content_support/` | Struktur, CSS, model konten, aturan render |
| DEMO | Skrip seed konten demo (di luar repositori) | Isi contoh sembilan halaman dan tiga artikel |
| Lainnya | `docs/decision-log.md` KEPUTUSAN-271, -272; `docs/backlog.md` CR-032, baris US-1901; `docs/07-phase-plan.md` kalender fase; `docs/design/brief-web-korporat.md` | — |

Sumber eksternal: `[SRC-256]` (WCAG 2.2) dan `[SRC-259]` (Plus Jakarta Sans) sudah tercatat di `docs/sources.md`. Brief ini tidak mengutip fakta eksternal baru. Kandidat set ikon dan perilaku `prefers-reduced-motion` diverifikasi saat Tahap 8.

## Lampiran B. Catatan konsistensi sumber

Ditemukan saat menyusun, **tidak diselesaikan diam-diam**:

1. **PRD US-1802 AC6** berbunyi "tidak ada **margin** aktif yang tersimpan sebagai konten siap tayang". Kemungkinan besar ini efek penggantian istilah "markup" menjadi "margin" di PRD v1.4. Maksud aslinya markup/HTML aktif. Perlu koreksi PRD.
2. **Beranda:** DG D-S01, wireframe, dan prototipe menampilkan enam ubin layanan, sedangkan implementasi hanya menampilkan judul konten CMS (D4).
3. **Layanan, Rekanan, Kontak, Unduh:** wireframe memakai kartu, tombol toko, alert "segera hadir", dan keadaan kosong "Daftar rekanan segera hadir". Implementasi merender satu Markdown per halaman, dan halaman tanpa konten tayang menjadi "tidak ditemukan".
4. **Kontak:** wireframe S01-7 menyebut tautan `tel:`, tetapi render aman hanya mengizinkan `https`, `mailto`, relatif (PRD US-1802 AC6, `aturan-aman.ts`).
5. **Artikel:** wireframe menampilkan gambar, ringkasan, dan paginasi "1-2 dari [n]". Model konten tidak punya ringkasan, dan daftar memakai kursor tanpa total.
6. **Kebijakan:** PRD US-1801 AC4 mensyaratkan versi dan tanggal berlaku. Model konten tidak punya field versi, dan "Berlaku sejak" memakai tanggal mulai tayang.
7. **Galat umum:** wireframe "Halaman sedang tidak dapat dimuat… Bila berlanjut, hubungi Layanan Pelanggan" dan KEPUTUSAN-272 menyebut arahan ke Layanan Pelanggan. Implementasi "Terjadi kesalahan…" tanpa arahan itu.
8. **Tipografi hero** implementasi (36/44 dan 18/28) dan teks isi 14 px di luar/di bawah kebutuhan situs, sedangkan skala web DG maksimum `display 28/36`. Perlu token situs (§7.3) lewat revisi DG.
9. **Penanda `[TIDAK DIKETAHUI]`** tampil ke publik saat konten kosong (Beranda, Artikel), berbeda dari pola kosong wireframe ("Belum ada artikel" + kalimat + tombol).
10. **Brief web korporat v0.1** masih menulis format NIB/NPWP `[TIDAK DIKETAHUI]` dan bukti kewenangan "surat kuasa atau surat penunjukan … pejabat berwenang". Keputusan user 2026-11-26 (Q6, Q9): NIB 13 digit, NPWP 16 digit, surat kuasa/penugasan direksi; Q12 (pindaian, PDF bertanda tangan digital ditolak) belum ada di dokumen proyek mana pun selain arahan orkestrator.
11. **Urutan alur korporat:** konten demo "Cara kerjanya" menaruh "Undang peserta" (3) sebelum "Bayar lewat invoice" (4), sedangkan urutan end-to-end di brief korporat J10 (lunas) → J11/J12 (peserta). PRD hanya mewajibkan penjadwalan setelah lunas (AB-83). Urutan perlu diselaraskan sebelum teks final.
12. **Klaim di konten demo** yang belum didukung penuh sumber: "harga yang tampil sama dengan yang Anda bayar" (umum; sumber hanya MCU/lab, BR-023), "dilindungi sesuai peraturan pelindungan data pribadi" (menunggu L-20), "bagian rekanan diteruskan langsung ke rekening rekanan" (menunggu L-04), dan katalog korporat "tersedia di member area" (baru Fase 3).
