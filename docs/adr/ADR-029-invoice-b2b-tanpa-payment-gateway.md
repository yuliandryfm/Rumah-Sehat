# ADR-029: Penagihan korporat lewat invoice B2B tanpa payment gateway, dengan harga terkunci saat konfirmasi dan jurnal berpasangan

Status: **Diusulkan; keputusan inti menunggu PT-59** (penerima dana invoice), L-21, dan keputusan user atas UP-84 (CR-021, 2026-11-19; revisi setelah review keamanan putaran 1 B-05, `docs/review/CR-021-security.md`)
Merevisi bila diterima: ADR-015 (konteks "semua layanan lewat payment gateway", daftar akun jurnal)
Tanggal: 2026-11-19

## Konteks

- Pembayaran korporat lewat invoice B2B (transfer, jatuh tempo), tanpa payment gateway `[USER] P1-4`. Invoice terbit saat pesanan dikonfirmasi, jatuh tempo 14 hari, layanan dijadwalkan setelah lunas `[USER] P3-3`. Harga = harga rekanan + margin sesuai aturan margin `[USER] P3-1` (`docs/cr/CR-021-wawancara.md`).
- ADR-015 menetapkan uang sebagai bilangan bulat Rupiah, harga terkunci, jurnal berpasangan immutable dengan jurnal pembalik, dan adaptor payment gateway. Konteksnya: semua layanan dibayar lewat payment gateway dan dibagi langsung, PT. Arunika tidak menampung bagian rekanan `[DOK: adr/ADR-015-pembayaran-ledger-refund.md]`.
- BRD BR-040, BR-041, BR-115 dan non-scope CR-003 ("Penampungan dana pasien di rekening PT. Arunika lalu diteruskan sendiri ke mitra") `[DOK: 02-brd.md]` berbenturan dengan transfer langsung tanpa payment gateway, kecuali alurnya dirancang agar PT. Arunika tidak menampung bagian rekanan (`docs/cr/CR-021-analisis.md` K-02, K-03).
- Pajak: arah rumus CR-013 diketahui; jenis, tarif, dan kewajiban faktur pajak B2B `[TIDAK DIKETAHUI]` (sisa PT-28, PT-73).

## Keputusan

Butir 1–5 berlaku apa pun jawaban PT-59. Butir 6 menunggu PT-59.

1. **Invoice sebagai entitas transaksi** `[KEPUTUSAN-259]`: `b2b_invoice` dan `b2b_invoice_line` (kelas T), riwayat status dan proyeksi (ADR-008). Nomor invoice unik, formatnya ditetapkan Tahap 8 (pola KEPUTUSAN-57). Koreksi berupa pembatalan invoice dan invoice pengganti yang merujuk invoice lama; baris tidak pernah diubah atau dihapus (ADR-004).
2. **Harga terkunci saat konfirmasi pesanan** (AB-81 usulan): `corporate_order_price_lock` menyimpan harga rekanan per satuan, ID dan nilai aturan margin berlaku-mulai, margin per satuan (pembulatan AB-11a), jumlah, tarif dan nominal pajak, dan total. Invoice dibangun dari baris ini.
3. **Mutasi bank dan alokasi** `[KEPUTUSAN-268]` (review keamanan B-05, M-44):
   - setiap transfer masuk dicatat manual oleh Keuangan **sekali** sebagai `bank_statement_entry` (kelas T): rekening tujuan, tanggal valuta, referensi bank, nominal, bukti lewat upload slot. Kunci unik **global** atas (rekening tujuan, tanggal valuta, referensi, nominal); komposisinya diverifikasi terhadap format mutasi bank yang dipakai saat task;
   - mutasi dialokasikan ke satu atau beberapa invoice lewat `b2b_payment_allocation` (kelas T). Jumlah alokasi dari satu mutasi tidak pernah melebihi nominal mutasi, ditegakkan di aplikasi dan di database (mekanisme constraint diverifikasi saat task). Dengan begitu satu transfer tidak dapat melunasi beberapa invoice melebihi nilainya;
   - koreksi mutasi atau alokasi lewat baris pembalik;
   - nomor rekening pembayar di kolom hanya 4 digit terakhir (PRD §7.3). Berkas bukti transfer hampir pasti memuat nomor rekening lengkap, sehingga disimpan di bucket privat, hanya dapat dibuka Keuangan lewat URL berumur pendek, dan isinya tidak pernah masuk log maupun audit trail (audit hanya merujuk ID berkas);
   - pemisahan tugas pencatat dan penyetuju, rekonsiliasi harian terhadap rekening koran oleh orang selain pencatat, status Lunas hanya dari mutasi yang diperiksa terhadap rekening koran (bukan dari bukti kiriman perusahaan), dan persetujuan orang kedua atas perubahan rekening tujuan invoice mengikuti keputusan user atas UP-84 (M-45, M-56). Bila user menolak, B2B-01 dicatat sebagai risiko sisa yang diterima user;
   - test negatif: mutasi yang sama dicatat dua kali; mutasi dialokasikan ke dua invoice melebihi nominalnya; alokasi oleh akun tanpa sub-peran Keuangan.
4. **Jurnal berpasangan** (ADR-015 butir 4) untuk penerbitan invoice, penerimaan, pembatalan, dan pembalik; keseimbangan diperiksa di aplikasi dan constraint database. Laporan piutang korporat dibaca dari jurnal.
5. **Tanpa adaptor payment gateway** untuk pesanan korporat: tidak ada notifikasi, refund otomatis, atau rekonsiliasi otomatis penyedia. Status Lunas hanya dari total alokasi mutasi yang sama dengan total invoice; penjadwalan ditolak server sebelum Lunas (AB-83 usulan). Baris invoice tidak menyebut peserta dan tidak dapat dikaitkan ke peserta tanpa persetujuan aktif (invarian kanal turunan ADR-028 butir 3).
6. **Arah dana dan akun jurnal (menunggu PT-59)**:
   - Opsi a — seluruh invoice ke rekening PT. Arunika: akun piutang korporat per perusahaan, pendapatan margin korporat, kewajiban kepada rekanan per rekanan (dilunasi lewat jurnal pelunasan kewajiban), dan pajak. Memerlukan CR atas BR-041/CR-003 dan L-21.
   - Opsi a juga berisiko terhadap BR-115: BI dapat menetapkan aspek prudensial bagi penyelenggara penunjang yang meneruskan pembayaran kepada penyedia barang dan/atau jasa `[SRC-20]`; apakah penerusan bagian rekanan termasuk aktivitas yang butuh izin BI `[TIDAK DIKETAHUI]` (L-21).
   - Opsi b — dua tagihan (bagian rekanan langsung ke rekening rekanan, margin ke PT. Arunika): piutang korporat hanya atas margin dan pajak bagian PT. Arunika; bagian rekanan dicatat sebagai informasi tagihan rekanan, dan konfirmasi lunasnya `[TIDAK DIKETAHUI]` (siapa yang mencatat). Opsi ini membuka margin kepada perusahaan, bertentangan dengan AB-13 dan US-1905 AC2, sehingga butuh pengecualian korporat.
   - Opsi c — virtual account dengan pembagian dana penyedia pembayaran: bertentangan dengan "tanpa payment gateway" `[USER] P1-4`; hanya bila user mengubah keputusan.

## Alternatif yang ditolak

- **Mencatat nominal hanya di tabel invoice tanpa jurnal**: alasan yang sama dengan ADR-015 (kewajiban dan koreksi sulit dibuktikan).
- **Memakai adaptor payment gateway yang ada dengan metode "transfer manual"**: mencampur penerimaan manual dengan notifikasi penyedia yang wajib diverifikasi (NFR-INT-04), dan tidak sesuai keputusan user.
- **Harga dikunci saat pelunasan**: invoice yang sudah terbit dapat berubah nilainya, padahal invoice adalah dokumen tagihan.
- **Penerimaan per invoice dengan referensi bank unik per invoice** (rancangan v0.1): satu mutasi bank yang sama dapat dicatat sebagai pelunasan invoice A dan B, sehingga dua pesanan dijadwalkan atas satu pembayaran (B2B-02).

## Konsekuensi

- Penagihan korporat masuk modul kritis T1 (harga, uang, jurnal) dengan test negatif: pencatatan mutasi ganda, alokasi melebihi nominal mutasi, nominal berbeda, penjadwalan sebelum lunas, akses invoice lintas perusahaan, pembalik.
- ADR-015 diberi catatan pengecualian korporat dan daftar akun baru setelah PT-59.
- Pembatalan, refund, bayar sebagian, kelebihan bayar, dan akibat lewat jatuh tempo belum diatur (PT-63); matriks RF-01–RF-18 tidak berlaku untuk pesanan korporat sampai diputuskan.
- Rekonsiliasi dengan mutasi bank bersifat manual di rilis ini; ketersediaan data mutasi bank untuk otomatisasi `[TIDAK DIKETAHUI]`.
- ADR ini tidak dapat berstatus Diterima sebelum PT-59 dan L-21 dijawab, B-05 ditutup di cek ulang review keamanan, dan UP-84 diputuskan user.
