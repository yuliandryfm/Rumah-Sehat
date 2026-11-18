# ADR-018: Berkas di object storage privat, unggah dan unduh lewat URL bertanda tangan setelah otorisasi

Status: Diusulkan
Tanggal: 2026-11-18

## Konteks

- Berkas yang dikelola: foto KTP dan KK, foto lampiran chat, foto bukti klaim, PDF hasil lab, PDF resep, dokumen izin tenaga kesehatan dan rekanan, gambar konten CMS, salinan data pribadi, dan export laporan `[DOK: 03-prd.md]`.
- Berkas dibatasi jenis dan ukurannya (foto JPG/PNG ≤ 5 MB; PDF ≤ 10 MB) dan hanya dapat diakses pihak yang berhak (NFR-SEC-08) `[KEPUTUSAN-08]`. Membuka foto dokumen identitas dan bukti klaim dicatat E-05, dan unduhan dicatat E-12 `[DOK: 03-prd.md]` §7.1.
- OWASP File Upload Cheat Sheet menganjurkan daftar ekstensi yang diizinkan, validasi jenis berkas tanpa mempercayai header `Content-Type`, batas ukuran, nama berkas yang dibuat aplikasi, dan penyimpanan di luar webroot `[SRC-50]`.
- Data di Indonesia `[USER] T4-Q1`.

## Keputusan

1. **Object storage privat di region Indonesia**, dengan bucket terpisah per sensitivitas: dokumen identitas, berkas klinis (lampiran chat, hasil lab, resep), bukti klaim, dokumen izin, export dan salinan data, konten publik CMS, serta foto produk dan layanan dari katalog rekanan. Hanya bucket konten CMS dan katalog yang boleh dibaca publik, dan isinya tidak boleh memuat data pribadi `[KEPUTUSAN-34]`.
2. **Unggah** `[KEPUTUSAN-34]`:
   - Klien meminta slot unggah. Backend memeriksa hak, lalu membuat nama objek UUID (bukan nama asli) dan URL unggah bertanda tangan berumur pendek yang dibatasi jenis dan ukuran, sejauh kemampuan penyedia (diverifikasi Tahap 5).
   - Setelah unggah, backend memvalidasi ukuran, jenis dari isi berkas (bukan dari header), dan untuk foto mendekode lalu mengode ulang gambar sehingga metadata (misalnya lokasi) terbuang.
   - Berkas baru dapat dirujuk setelah lolos validasi. Berkas yang gagal validasi atau tidak pernah dikonfirmasi dihapus otomatis.
3. **Unduh** `[KEPUTUSAN-34]`: backend memeriksa kebijakan (ADR-012), menulis event audit bila berlaku (E-05 atau E-12), lalu menerbitkan URL unduh bertanda tangan berumur pendek (nilai awal 5 menit) dengan header unduhan sebagai lampiran. URL objek tidak pernah disimpan atau ditampilkan permanen. Ketersediaan unduhan di aplikasi (misalnya salinan data pribadi yang dapat diunduh selama 7 hari, US-1604 AC2) berbeda dari umur URL: selama masa ketersediaan, setiap unduhan menerbitkan URL baru berumur pendek setelah autentikasi ulang.
4. **Metadata berkas** disimpan di tabel berkas: pemilik, kategori, wadah data untuk retensi (ADR-013), checksum, ukuran, jenis, dan status validasi.
5. **Enkripsi dan pemusnahan**: enkripsi lapis penyedia dengan kunci KMS (ADR-013). Versi objek diaktifkan untuk pemulihan. Pemusnahan karena retensi atau permintaan menghapus objek beserta seluruh versinya, dan dicatat E-16 `[KEPUTUSAN-34]`.
6. **Pemindaian malware** untuk PDF dari pengguna dan rekanan dievaluasi di Tahap 5, dengan syarat layanannya memproses data di Indonesia. Bila tidak tersedia, PDF hanya disajikan sebagai unduhan dan tidak pernah ditampilkan langsung di backoffice `[KEPUTUSAN-34]`.

## Alternatif yang ditolak

- **Menyimpan berkas di database**: membesarkan database dan backup, dan memperlambat pemulihan (RTO).
- **Unggah lewat backend (proxy)**: menambah beban proses `api` untuk berkas sampai 10 MB tanpa manfaat keamanan, karena validasi tetap dilakukan setelah unggah.
- **URL objek publik atau berumur panjang**: siapa pun yang memegang tautan dapat membuka dokumen identitas atau data klinis.

## Konsekuensi

- Alur unggah memerlukan tiga langkah (minta slot, unggah, konfirmasi) di aplikasi dan di kontrak integrasi rekanan untuk PDF hasil lab (ADR-014).
- Kemampuan pembatasan URL bertanda tangan, versi objek, dan WORM penyedia (ADR-009) menjadi syarat pemilihan object storage di Tahap 5.
