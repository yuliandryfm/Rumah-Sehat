# ADR-015: Pembayaran lewat adaptor payment gateway, harga terkunci, jurnal berpasangan, dan refund berbasis job

Status: Diusulkan
Tanggal: 2026-11-18

## Konteks

- Semua layanan dibayar di aplikasi lewat payment gateway berizin BI (AB-20). Pembayaran dibagi langsung oleh payment gateway: bagian rekanan ke rekening rekanan, dan markup ke rekening PT. Arunika (AB-21, AB-22) `[DOK: 03-prd.md]` §3.4.
- Harga, markup, dan parameter dikunci saat pembayaran dibuat (AB-17). Markup dihitung per unit dengan pembulatan ke Rupiah penuh (AB-11a). Rupiah tanpa desimal (NFR-LOC-02) `[DOK: 03-prd.md]`.
- Matriks refund RF-01–RF-18 dan rumus kewajiban AB-34; refund dicatat sebagai entri pembalik (AB-31). Satu pesanan paling banyak satu refund kecuali RF-12 (AB-36). Status refund mencakup Diproses, Berhasil, Gagal, dan Perlu dicek (AB-32, AB-35) `[DOK: 03-prd.md]` §3.5, §3.6.
- Notifikasi payment gateway diverifikasi keasliannya (NFR-INT-04). Notifikasi berulang hanya mengonfirmasi pesanan sekali (US-802 AC2). Rekonsiliasi berjalan harian (US-811 AC2) `[DOK: 03-prd.md]`.
- Payment gateway belum dipilih. Metode bayar, kemampuan refund ke metode asal, pengembalian MDR, mekanisme penarikan dana rekanan (PT-27), dan penyaluran ongkos kirim (PT-31) `[TIDAK DIKETAHUI]` sampai penyedia dipilih di Tahap 5 dan L-04 dijawab `[DOK: 03-prd.md]` `[DOK: 02-brd.md]`.
- Penyedia wajib memproses data di Indonesia `[USER] T4-Q1`.

## Keputusan

1. **Uang disimpan sebagai bilangan bulat Rupiah** (`bigint`, kolom berakhiran `_idr`). Tidak ada tipe pecahan untuk nominal. Semua perhitungan harga dilakukan di server `[KEPUTUSAN-31]`.
2. **Adaptor payment gateway.** Modul pembayaran berbicara ke penyedia lewat satu antarmuka: buat pembayaran (nominal, instruksi pembagian, kedaluwarsa PB-07, metode), batalkan instruksi, refund, tanya status, verifikasi notifikasi, dan ambil laporan pembagian atau penyelesaian. Implementasi konkret dibuat setelah penyedia dipilih di Tahap 5. Syarat pemilihan: berizin BI (BR-115), mendukung pembagian dana langsung ke rekening rekanan (AB-21), mendukung refund, dan memproses data di Indonesia `[KEPUTUSAN-31]`.
3. **Harga terkunci** saat pembayaran dibuat, sebagai baris transaksi immutable per pesanan: harga rekanan per unit, aturan markup yang berlaku (jenis, nilai, ID versi pengaturan), markup per unit, jumlah, ongkos kirim, biaya transaksi metode terpilih, dan total. Snapshot parameter bisnis yang memengaruhi pesanan (misalnya PB-01) juga disimpan. Aturan pembatalan dan refund selalu membaca snapshot ini (AB-17, US-812 AC2) `[KEPUTUSAN-31]`.
4. **Jurnal berpasangan (double-entry)** sebagai catatan keuangan immutable `[KEPUTUSAN-31]`:
   - Setiap kejadian uang (pembayaran diterima, pembagian dilaporkan, refund dibuat, refund berhasil, kewajiban rekanan dicatat atau dilunasi, entri pembalik) membuat satu jurnal dengan baris debit dan kredit yang jumlahnya seimbang. Keseimbangan diperiksa di aplikasi dan dengan constraint database.
   - Akun jurnal mencakup antara lain dana pasien, bagian rekanan per rekanan, pendapatan markup, biaya payment gateway, kewajiban pengembalian rekanan, dan kewajiban platform.
   - Koreksi selalu berupa jurnal pembalik yang merujuk jurnal asal (ADR-004, AB-26, AB-31). Laporan keuangan dan laporan rekanan (US-811, US-1201) dibaca dari jurnal.
5. **Notifikasi payment gateway** `[KEPUTUSAN-31]`:
   - Endpoint notifikasi memverifikasi keaslian sesuai mekanisme penyedia (NFR-INT-04).
   - Isi notifikasi disimpan sebagai log (tanpa rahasia), unik per ID kejadian penyedia, lalu diproses oleh job.
   - Setiap percobaan pembayaran dicatat sebagai baris transaksi beserta kunci idempotensinya **sebelum** penyedia dipanggil. Referensi dari penyedia disimpan di tabel transaksi anak, sehingga tidak ada `UPDATE` pada baris percobaan dan tidak ada instruksi bayar di penyedia yang tidak tercatat.
   - Setiap dana masuk dicatat sebagai penerimaan, dengan ID transaksi penyedia sebagai kunci unik. Satu percobaan boleh punya lebih dari satu penerimaan, supaya debit ganda atas instruksi yang sama tetap tercatat.
   - Penerimaan yang sah menjadi tangkapan pembayaran dengan constraint unik per pesanan. Penerimaan berikutnya untuk pesanan yang sama ditandai ganda dan dialihkan ke RF-12 (US-802 AC6).
   - Nominal yang berbeda diarahkan ke RF-14, dan pembayaran yang berhasil setelah kedaluwarsa ke pemeriksaan slot/stok untuk RF-13.
6. **Refund** `[KEPUTUSAN-31]`:
   - Refund adalah entitas transaksi dengan riwayat status dan proyeksi (ADR-008).
   - Constraint unik "satu refund per pesanan" ditegakkan dengan partial unique index `[SRC-33]`, dengan pengecualian RF-12 (AB-36). RF-13 dan RF-14 juga dikecualikan, karena keduanya refund atas dana yang tidak menjadi pembayaran pesanan. Tanpa pengecualian ini, refund yang wajib menurut matriks (misalnya RF-01 setelah ada RF-14 pada percobaan lain) ditolak constraint. Tafsiran AB-36 ini menunggu konfirmasi user di gate Tahap 4 `[KEPUTUSAN-67]`. Satu penerimaan dana paling banyak satu refund.
   - Eksekusi lewat job yang memanggil adaptor dengan kunci idempotensi = ID refund, sehingga percobaan ulang tidak menggandakan refund.
   - Job PB-19 memindahkan refund yang tidak dikonfirmasi ke "Perlu dicek" tanpa eksekusi ulang (AB-35).
   - Kewajiban pengembalian rekanan dan platform (AB-34) dicatat sebagai jurnal saat refund dibuat, atau untuk RF-18 saat penyebab ditetapkan, berstatus Terbuka sampai Keuangan menandainya Selesai dengan rujukan bukti (US-808).
7. **Mekanisme penarikan dana rekanan** (PT-27) dan **penyaluran ongkos kirim** (PT-31) tidak dikunci di arsitektur. Jurnal mencatat kewajiban apa pun mekanismenya. Pilihan mekanisme ditetapkan di Tahap 5 berdasarkan kemampuan penyedia terpilih dan jawaban L-04, lalu dibawa ke user `[KEPUTUSAN-31]`.
8. **Biaya transaksi per metode** yang ditampilkan sebelum bayar (AB-12) diambil dari tabel tarif metode pembayaran (kelas Master, diatur Keuangan). Biaya aktual dari laporan penyedia dicatat terpisah. Selisihnya muncul di rekonsiliasi `[KEPUTUSAN-31]`.
9. **Rekonsiliasi harian**: job mengambil laporan transaksi, pembagian, dan refund dari penyedia, membandingkannya dengan jurnal, lalu menandai selisih nominal, pembagian yang tidak terlaporkan, dan pembayaran ganda untuk ditindaklanjuti (US-811 AC2) `[KEPUTUSAN-31]`.

## Alternatif yang ditolak

- **Mencatat nominal hanya di tabel pesanan dan refund tanpa jurnal**: kewajiban rekanan, entri pembalik, dan rekonsiliasi lintas kejadian sulit dibuktikan, dan saldo kewajiban per rekanan harus dihitung dengan logika yang tersebar.
- **Tipe desimal atau float untuk uang**: float tidak eksak. Desimal tidak diperlukan karena Rupiah tanpa desimal.
- **Integrasi langsung ke SDK satu penyedia di seluruh kode**: mengunci desain sebelum penyedia dipilih dan menyulitkan penggantian penyedia.

## Konsekuensi

- Modul pembayaran, harga, jurnal, dan refund adalah modul kritis (T1) dengan test wajib termasuk skenario negatif (notifikasi ganda, nominal berbeda, refund ganda, kegagalan penyedia).
- Pemilihan payment gateway di Tahap 5 harus memverifikasi syarat di butir 2 ke dokumentasi resmi penyedia. Kalau tidak ada penyedia yang memenuhi pembagian dana langsung dengan data di Indonesia, hal itu menjadi risiko tinggi yang dibawa ke user (risk register).
- Perlakuan pajak (PT-28) belum diketahui. Kolom pajak tidak ditambahkan sampai ketentuannya jelas; jurnal dapat menampung akun pajak tanpa perubahan struktur.
