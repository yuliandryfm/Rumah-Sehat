# Aturan Anti-Halusinasi dan Sumber Valid

Aturan ini **wajib dibaca dan dipatuhi di setiap tahap, di kedua mode**. Tujuannya bukan menjanjikan model tidak pernah salah, melainkan memastikan **tidak ada klaim yang lolos tanpa sumber yang bisa dilacak dan diperiksa**.

## 1. Prinsip dasar

1. **Tidak tahu berarti ditulis `[TIDAK DIKETAHUI]`.** Jangan pernah mengisinya dengan tebakan yang terdengar masuk akal.
2. **Setiap klaim punya label sumber.** Klaim tanpa label dianggap halusinasi sampai terbukti sebaliknya.
3. **Fakta bisnis tidak boleh diasumsikan.** Pengguna, aturan bisnis, angka target, regulasi yang berlaku, dan integrasi hanya boleh berlabel `[USER]` atau `[DOK]`.
4. **Fakta eksternal diverifikasi ke sumber primer.** Halaman sumber harus benar-benar dibuka. Cuplikan hasil pencarian tidak cukup.
5. **Klaim selesai wajib disertai bukti** berupa perintah yang dijalankan dan output-nya.
6. **Pemeriksaan oleh mesin lebih dipercaya daripada pernyataan.** Test, type check, CI, dan registry package adalah hakim akhirnya.

## 2. Label sumber

| Label | Arti | Syarat |
| --- | --- | --- |
| `[USER]` | Dinyatakan langsung oleh user | Tercatat di Project Brief, jawaban wawancara, atau catatan checkpoint |
| `[DOK: nama-file]` | Berasal dari dokumen yang dilampirkan user atau dokumen proyek yang sudah lulus gate | File ada di `docs/input/` atau `docs/` |
| `[SRC-xx]` | Fakta eksternal yang sudah diverifikasi | Tercatat di `docs/sources.md` lengkap dengan URL, tanggal akses, dan kutipan pendek yang mendukung |
| `[ASUMSI-xx]` | Dugaan Claude untuk detail **teknis** | Tercatat di `docs/decision-log.md`, wajib dikonfirmasi user di checkpoint berikutnya |
| `[KEPUTUSAN-xx]` | Pilihan teknis Claude di antara opsi yang valid | Tercatat di `docs/decision-log.md` beserta alasan dan sumbernya |
| `[TIDAK DIKETAHUI]` | Informasi belum ada | Masuk daftar pertanyaan terbuka beserta pemiliknya |

Penulisan: label diletakkan di akhir kalimat atau di kolom "Sumber" pada tabel. Contoh: "Target 500 pengguna aktif harian di bulan pertama `[USER]`."

## 3. Hierarki sumber valid

Pakai sumber dari tingkat tertinggi yang tersedia.

**Tingkat 1: primer internal**
- Pernyataan user dan dokumen yang dilampirkan user
- Dokumen proyek yang sudah lulus gate (Project Brief, BRD, PRD, ADR, dan seterusnya)

**Tingkat 2: primer eksternal resmi**
- Regulasi dari situs resmi pemerintah atau regulator (misalnya peraturan.go.id, JDIH kementerian/lembaga, situs resmi OJK atau BI)
- Dokumentasi resmi bahasa pemrograman, framework, database, dan layanan cloud
- Registry package resmi (npm, PyPI, Maven Central, crates.io, Packagist, dan sejenisnya)
- Standar terbuka: OWASP (ASVS, Top 10, Cheat Sheet Series), NIST, W3C/WCAG, RFC
- Basis data kerentanan: NVD, GitHub Advisory Database, OSV

**Tingkat 3: sekunder bereputasi**
- Literatur yang diakui luas (misalnya PMBOK Guide, manual PRINCE2, buku rekayasa perangkat lunak dari penerbit terkemuka)
- Publikasi teknis dari organisasi atau vendor kredibel

Tingkat 3 hanya untuk praktik terbaik dan kerangka kerja, **tidak untuk fakta yang bisa dicek ke sumber tingkat 2** (versi, sintaks API, isi pasal).

**Tidak valid sebagai sumber:**
- Ingatan model tanpa verifikasi
- Cuplikan hasil pencarian tanpa membuka halamannya
- Forum, blog anonim, atau konten yang dibuat AI
- Sumber berbayar yang tidak bisa dibuka (boleh disebut judulnya sebagai rujukan, tapi isinya tidak boleh dikutip seolah sudah dibaca)

Kalau sumber valid tidak ditemukan: tulis `[TIDAK DIKETAHUI]`, catat apa yang sudah dicari, dan masukkan ke pertanyaan terbuka.

## 4. Sumber valid per tahap

| Tahap | Sumber yang diizinkan untuk klaim utama |
| --- | --- |
| 0. Intake | `[USER]`, `[DOK]` saja. Tidak ada asumsi untuk fakta bisnis |
| 1. Business Case | Project Brief. Angka biaya dan waktu wajib ditandai estimasi beserta dasar perhitungannya |
| 2. BRD | Project Brief, Business Case, dokumen user. Regulasi wajib `[SRC]` dari situs resmi |
| 3. PRD + NFR | BRD (setiap fitur menunjuk nomor BR). Angka NFR dari `[USER]` atau dari BRD |
| 4. Arsitektur | PRD, ADR. Klaim kemampuan teknologi atau layanan wajib `[SRC]` dokumentasi resmi |
| 5. Stack | Versi, lisensi, dan status maintenance wajib `[SRC]` registry atau dokumentasi resmi, dicek saat itu juga |
| 6. Design Guideline | PRD. Aturan aksesibilitas `[SRC]` WCAG |
| 7. Phase Plan | PRD, arsitektur, backlog. Tidak ada story tanpa induk BR |
| 8. Development | Dokumentasi resmi untuk setiap API atau library yang dipakai. Kode program harus lolos compile, type check, dan test |
| 9–10. QA, SIT, Audit | PRD (acceptance criteria), BRD. Temuan keamanan merujuk ke OWASP atau CVE `[SRC]` dan menyebut file serta baris |
| 11. UAT | BRD (proses *to-be*), Business Case (KPI). Sign-off hanya `[USER]` |
| 12–14. Rilis | Dokumentasi resmi platform deploy. Hasil smoke test dan monitoring sebagai bukti |

## 5. Aturan khusus kode (Tahap 8)

- **Jangan memanggil fungsi, method, atau endpoint yang belum dipastikan ada.** Cek di dokumentasi resmi versi yang dipakai, atau baca source code library-nya di folder dependensi.
- **Sebelum menambah dependensi baru**, pastikan package benar-benar ada di registry resmi dengan nama yang persis (misalnya `npm view <nama>` atau `pip index versions <nama>`), periksa maintainer dan tanggal rilis terakhir, lalu catat di `docs/sources.md`. Nama package yang dikarang bisa didaftarkan penyerang.
- **Kunci versi** dengan lockfile, dan lockfile ikut di-commit.
- **Tidak boleh menyatakan test lulus tanpa menjalankannya.** Sertakan perintah dan ringkasan output.
- **Hanya commit hijau.** Commit dan merge yang berisi kode ditolak oleh git hook kalau `scripts/verify.sh` merah. Jangan pernah mencoba melewatinya (`--no-verify`, `-n`, mengubah `core.hooksPath`, atau melonggarkan `scripts/verify.sh`). Tidak ada commit "WIP" yang merah.
- **Status task di backlog hanya berubah dengan bukti**: commit hash dan hasil `scripts/verify.sh`.

## 6. Aturan khusus temuan review

- Setiap temuan reviewer wajib menyebut file dan baris, serta cara mereproduksinya.
- Developer **memverifikasi temuan dulu** sebelum memperbaikinya. Temuan yang tidak bisa direproduksi dicatat sebagai "tidak terkonfirmasi", tidak langsung diperbaiki.

## 7. Register pendukung

- `docs/sources.md`: daftar semua `[SRC-xx]`
- `docs/decision-log.md`: daftar semua `[ASUMSI-xx]` dan `[KEPUTUSAN-xx]`
- `docs/traceability.md`: BR → story → test case → file kode/test
- `docs/fact-check/tahap-XX.md`: laporan fact-checker per tahap

## 8. Gate anti-halu (berlaku untuk SETIAP tahap)

- [ ] Subagent `fact-checker` sudah dijalankan untuk output tahap ini
- [ ] Nol klaim tanpa label sumber
- [ ] Nol fakta bisnis berlabel `[ASUMSI]`
- [ ] Setiap `[SRC-xx]` tercatat di `docs/sources.md` dengan URL dan tanggal akses
- [ ] Setiap `[ASUMSI-xx]` dan `[KEPUTUSAN-xx]` tercatat di `docs/decision-log.md`
- [ ] Mulai Tahap 2: `docs/traceability.md` diperbarui
- [ ] Mulai Tahap 8: setiap klaim selesai disertai bukti eksekusi

Kedalaman pemeriksaan gate mengikuti tingkat ketelitian, dan status blokir hanya untuk kategori substantif di `aturan-kerja.md` §1–§2. Klaim substantif tanpa sumber tetap selalu blokir.
