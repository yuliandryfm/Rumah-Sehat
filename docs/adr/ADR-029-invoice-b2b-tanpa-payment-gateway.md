# ADR-029: Penagihan korporat lewat invoice B2B tanpa payment gateway, dengan harga terkunci saat konfirmasi dan jurnal berpasangan

Status: **Diusulkan; keputusan inti menunggu PT-59** (penerima dana invoice) dan L-21 (CR-021, 2026-11-19)
Merevisi bila diterima: ADR-015 (konteks "semua layanan lewat payment gateway", daftar akun jurnal)
Tanggal: 2026-11-19

## Konteks

- Pembayaran korporat lewat invoice B2B (transfer, jatuh tempo), tanpa payment gateway `[USER] P1-4`. Invoice terbit saat pesanan dikonfirmasi, jatuh tempo 14 hari, layanan dijadwalkan setelah lunas `[USER] P3-3`. Harga = harga rekanan + markup sesuai aturan markup `[USER] P3-1` (`docs/cr/CR-021-wawancara.md`).
- ADR-015 menetapkan uang sebagai bilangan bulat Rupiah, harga terkunci, jurnal berpasangan immutable dengan jurnal pembalik, dan adaptor payment gateway. Konteksnya: semua layanan dibayar lewat payment gateway dan dibagi langsung, PT. Arunika tidak menampung bagian rekanan `[DOK: adr/ADR-015-pembayaran-ledger-refund.md]`.
- BRD BR-040, BR-041, BR-115 dan non-scope CR-003 ("Penampungan dana pasien di rekening PT. Arunika lalu diteruskan sendiri ke mitra") `[DOK: 02-brd.md]` berbenturan dengan transfer langsung tanpa payment gateway, kecuali alurnya dirancang agar PT. Arunika tidak menampung bagian rekanan (`docs/cr/CR-021-analisis.md` K-02, K-03).
- Pajak: arah rumus CR-013 diketahui; jenis, tarif, dan kewajiban faktur pajak B2B `[TIDAK DIKETAHUI]` (sisa PT-28, PT-73).

## Keputusan

Butir 1–5 berlaku apa pun jawaban PT-59. Butir 6 menunggu PT-59.

1. **Invoice sebagai entitas transaksi** `[KEPUTUSAN-259]`: `b2b_invoice` dan `b2b_invoice_line` (kelas T), riwayat status dan proyeksi (ADR-008). Nomor invoice unik, formatnya ditetapkan Tahap 8 (pola KEPUTUSAN-57). Koreksi berupa pembatalan invoice dan invoice pengganti yang merujuk invoice lama; baris tidak pernah diubah atau dihapus (ADR-004).
2. **Harga terkunci saat konfirmasi pesanan** (AB-81 usulan): `corporate_order_price_lock` menyimpan harga rekanan per satuan, ID dan nilai aturan markup berlaku-mulai, markup per satuan (pembulatan AB-11a), jumlah, tarif dan nominal pajak, dan total. Invoice dibangun dari baris ini.
3. **Penerimaan transfer dicatat manual** oleh Keuangan di `b2b_payment_receipt` (kelas T): tanggal terima, nominal, referensi bank (unik per invoice), dan bukti lewat upload slot. Nomor rekening hanya 4 digit terakhir (PRD §7.3). Koreksi lewat baris pembalik. Pemisahan tugas pencatat dan penyetuju mengikuti keputusan user atas UP-84.
4. **Jurnal berpasangan** (ADR-015 butir 4) untuk penerbitan invoice, penerimaan, pembatalan, dan pembalik; keseimbangan diperiksa di aplikasi dan constraint database. Laporan piutang korporat dibaca dari jurnal.
5. **Tanpa adaptor payment gateway** untuk pesanan korporat: tidak ada notifikasi, refund otomatis, atau rekonsiliasi otomatis penyedia. Status Lunas hanya dari penerimaan tercatat yang jumlahnya sama dengan total invoice; penjadwalan ditolak server sebelum Lunas (AB-83 usulan).
6. **Arah dana dan akun jurnal (menunggu PT-59)**:
   - Opsi a — seluruh invoice ke rekening PT. Arunika: akun piutang korporat per perusahaan, pendapatan markup korporat, kewajiban kepada rekanan per rekanan (dilunasi lewat jurnal pelunasan kewajiban), dan pajak. Memerlukan CR atas BR-041/CR-003 dan L-21.
   - Opsi b — dua tagihan (bagian rekanan langsung ke rekening rekanan, markup ke PT. Arunika): piutang korporat hanya atas markup dan pajak bagian PT. Arunika; bagian rekanan dicatat sebagai informasi tagihan rekanan, dan konfirmasi lunasnya `[TIDAK DIKETAHUI]` (siapa yang mencatat).
   - Opsi c — virtual account dengan pembagian dana penyedia pembayaran: bertentangan dengan "tanpa payment gateway" `[USER] P1-4`; hanya bila user mengubah keputusan.

## Alternatif yang ditolak

- **Mencatat nominal hanya di tabel invoice tanpa jurnal**: alasan yang sama dengan ADR-015 (kewajiban dan koreksi sulit dibuktikan).
- **Memakai adaptor payment gateway yang ada dengan metode "transfer manual"**: mencampur penerimaan manual dengan notifikasi penyedia yang wajib diverifikasi (NFR-INT-04), dan tidak sesuai keputusan user.
- **Harga dikunci saat pelunasan**: invoice yang sudah terbit dapat berubah nilainya, padahal invoice adalah dokumen tagihan.

## Konsekuensi

- Penagihan korporat masuk modul kritis T1 (harga, uang, jurnal) dengan test negatif: pencatatan ganda, nominal berbeda, penjadwalan sebelum lunas, akses invoice lintas perusahaan, pembalik.
- ADR-015 diberi catatan pengecualian korporat dan daftar akun baru setelah PT-59.
- Pembatalan, refund, bayar sebagian, kelebihan bayar, dan akibat lewat jatuh tempo belum diatur (PT-63); matriks RF-01–RF-18 tidak berlaku untuk pesanan korporat sampai diputuskan.
- Rekonsiliasi dengan mutasi bank bersifat manual di rilis ini; ketersediaan data mutasi bank untuk otomatisasi `[TIDAK DIKETAHUI]`.
- ADR ini tidak dapat berstatus Diterima sebelum PT-59 dan L-21 dijawab.
