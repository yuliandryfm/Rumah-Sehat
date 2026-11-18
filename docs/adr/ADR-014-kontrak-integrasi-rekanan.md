# ADR-014: Kontrak integrasi rekanan, yaitu Partner Integration API v1 dan webhook bertanda tangan

Status: Diusulkan
Tanggal: 2026-11-18

## Konteks

- Data klinik, lab, home visit, dan produk berasal dari sistem rekanan lewat integrasi (AB-60). Kontrak yang sama berlaku untuk sistem milik rekanan dan RME minimalis, tanpa jalur khusus (AB-61, US-1401 AC3). Mekanisme teknis (API, format, keamanan) ditetapkan di Tahap 4 `[DOK: 03-prd.md]` EP-14.
- Arah data per jenis rekanan tercantum di tabel EP-14. Sebagian data mengalir dari rekanan ke Rumah Sehat (katalog, jadwal, status, hasil lab), sebagian dari Rumah Sehat ke rekanan (pesanan terbayar, pembatalan, daftar tenaga kesehatan aktif, status home visit), dan ada pemeriksaan harga serta ketersediaan sebelum bayar (AB-15) `[DOK: 03-prd.md]`.
- Pesan diproses idempoten dan dicoba ulang (NFR-INT-01). Perubahan data rekanan tampil paling lambat 15 menit (NFR-INT-02). Integrasi yang gagal terus selama 30 menit memicu peringatan (NFR-INT-03). Pesan tidak valid ditolak seluruhnya (US-1402 AC2). Rekanan dengan kredensial salah dicatat sebagai kejadian keamanan (US-1402 AC3) `[DOK: 03-prd.md]`.
- Batas waktu tunggu ke rekanan saat pemeriksaan ulang sebelum bayar ditetapkan di Tahap 4 (PRD §9). Batas waktu percobaan ulang pengiriman pesanan ditetapkan di Tahap 4 (US-402 AC3) `[DOK: 03-prd.md]`.
- Kemampuan sistem rekanan `[TIDAK DIKETAHUI]` (PT-05) `[DOK: 02-brd.md]`.

## Keputusan

**1. Bentuk kontrak.** Partner Integration API versi 1: REST dengan JSON di atas HTTPS, dengan spesifikasi OpenAPI yang diterbitkan untuk rekanan (US-1401). Galat dikembalikan dalam format Problem Details (RFC 9457) `[SRC-47]`, dengan kode galat yang terdokumentasi. Versi ada di path (`/v1`). Perubahan yang merusak kompatibilitas diterbitkan sebagai versi baru dengan masa transisi `[KEPUTUSAN-30]`.

**2. Arah dan pola** `[KEPUTUSAN-30]`:

| Pola | Isi (sesuai tabel EP-14) |
| --- | --- |
| Rekanan memanggil Rumah Sehat (masuk) | Unggah dan ubah layanan, harga, paket MCU, jadwal, slot, kuota, area home visit, katalog, stok; status check-in, selesai, antrean, penugasan home visit, status pesanan produk; hasil lab (nilai terstruktur dan/atau PDF lewat URL unggah bertanda tangan, ADR-018); pembatalan oleh rekanan |
| Rumah Sehat mengirim webhook ke rekanan (keluar) | Pesanan terbayar beserta status lunas dan data penerima minimal; pembatalan oleh pasien atau sistem; perubahan daftar tenaga kesehatan aktif klinik; status kunjungan home visit |
| Rumah Sehat memanggil rekanan secara sinkron | Pemeriksaan harga dan ketersediaan sekaligus penahanan slot atau stok sebelum pembayaran dibuat (AB-15, KU-09). Rekanan mengembalikan ID penahanan dan masa berlakunya, minimal PB-07 |

**3. Keamanan** `[KEPUTUSAN-30]`:
- Panggilan masuk: setiap rekanan, dan setiap fasyankes di RME minimalis, punya klien sendiri dengan OAuth 2.0 client credentials `[SRC-45]` dan token akses berumur pendek. Lapisan mTLS dipertimbangkan di Tahap 5.
- Webhook dan panggilan sinkron keluar: ditandatangani HMAC-SHA256 `[SRC-46]` atas stempel waktu dan isi pesan, dengan rahasia per rekanan. Rekanan menolak pesan yang lebih tua dari 5 menit.
- Setiap rekanan hanya dapat menulis dan membaca data miliknya. ID rekanan diambil dari kredensial, bukan dari isi pesan (ADR-012).
- Batas laju dan batas ukuran pesan per rekanan. Kredensial salah dan tanda tangan tidak valid dicatat di security log (ADR-010).

**4. Keandalan** `[KEPUTUSAN-30]`:
- Pesan masuk yang mengubah data wajib membawa kunci idempotensi. Kunci disimpan 7 hari (kelas Operasional), dan pesan ulang dengan kunci yang sama mendapat respons yang sama tanpa diproses ulang.
- Pesan masuk divalidasi skema dan diterapkan dalam satu transaksi. Satu bagian tidak valid membuat seluruh pesan ditolak (US-1402 AC2).
- Pesan keluar dikirim dari outbox (ADR-007) minimal sekali, dengan `event_id` unik yang dipakai rekanan untuk menolak duplikat (US-1403 AC2).
- Percobaan ulang memakai jeda yang membesar (1, 2, 4, ... menit, maksimal 30 menit di antara percobaan). Untuk layanan terjadwal, percobaan berjalan sampai diterima atau sampai jadwal mulai; setelah itu berlaku RF-18 (US-402 AC3). Untuk produk, percobaan berjalan sampai diterima atau PB-06 habis; setelah itu berlaku RF-08.
- Bila pengiriman satu pesanan gagal terus selama 30 menit, pesanan ditandai untuk tindak lanjut Layanan Pelanggan (US-402 AC3).
- Status yang tidak sesuai urutan transisi PRD §3.6 ditolak dengan galat yang dikembalikan ke rekanan (US-1404 AC2).

**5. Pemeriksaan sebelum bayar.** Panggilan sinkron ke rekanan memakai batas waktu 5 detik tanpa percobaan ulang di dalam permintaan pasien. Bila gagal atau habis waktu, pasien diberi tahu bahwa data sementara tidak tersedia, dan pembayaran tidak dibuat (KU-08). Kalau harga atau ketersediaan berbeda, berlaku AB-15 `[KEPUTUSAN-30]`.

**6. Kesegaran data.** Rekanan wajib mengirim perubahan katalog, harga, dan jadwal begitu terjadi. Rumah Sehat menyediakan endpoint untuk pengiriman snapshot penuh sebagai rekonsiliasi berkala. Setiap data salinan rekanan menyimpan waktu sinkron terakhir untuk tampilan KU-08 `[KEPUTUSAN-30]`.

**7. Pemantauan per rekanan** (US-1405): status Belum, Uji, Aktif, Terganggu, Nonaktif; waktu sinkron terakhir yang berhasil; dan jumlah galat 24 jam. Bila tidak ada pesan berhasil selama 30 menit sementara ada percobaan yang gagal, status menjadi Terganggu dan admin diberi peringatan (NFR-INT-03) `[KEPUTUSAN-30]`.

**8. Lingkungan uji.** Rekanan menguji di lingkungan staging dengan kredensial uji dan skenario wajib yang sama untuk semua rekanan, termasuk RME minimalis (US-1401 AC2–AC3) `[KEPUTUSAN-30]`.

**9. Data pribadi.** Field data pribadi di kontrak ditandai, dan isinya dibatasi pada data minimal setelah L-08 dijawab. Setiap pengiriman dicatat E-17 (ADR-009, ADR-013).

## Alternatif yang ditolak

- **Rumah Sehat menarik data dari sistem setiap rekanan (polling) dengan adaptor per rekanan**: setiap rekanan baru menambah kode khusus, dan RME tidak lagi lulus "tanpa jalur khusus". Rumah Sehat yang menetapkan kontrak.
- **Pertukaran berkas terjadwal (CSV atau SFTP)**: tidak memenuhi kebutuhan status real-time (check-in, antrean 30 detik) dan pemeriksaan sinkron sebelum bayar.
- **Kunci API statis tanpa token**: rahasia jangka panjang ikut terkirim di setiap permintaan, dan rotasinya sulit.

## Konsekuensi

- Rekanan dengan sistem sendiri harus mampu memanggil API, menerima webhook, dan menyediakan endpoint pemeriksaan sinkron. Kalau sistem rekanan tidak mampu (PT-05), pilihannya rekanan memakai RME minimalis, atau Rumah Sehat membangun adaptor khusus. Pilihan itu keputusan bisnis yang dibawa ke user saat PT-05 terjawab (risk register).
- Kontrak ini dirancang sekali dan dipakai RME minimalis sebagai klien pertama, sehingga RME menjadi uji kontrak sejak awal.
- Spesifikasi OpenAPI menjadi artefak yang dijaga versinya, dan perubahan kontrak memerlukan review.
