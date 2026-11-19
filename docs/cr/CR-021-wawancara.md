# CR-021: wawancara requirement web company profile + member area korporat

Sumber semua jawaban: `[USER]` lewat Telegram, 2026-11-19. Konteks: `docs/backlog.md` CR-021.

## Putaran 0 (penempatan)

| Q | Pertanyaan | Jawaban |
| --- | --- | --- |
| P0-1 | Penempatan rilis | Awalnya "1A" (company profile rilis 1, member area rilis berikutnya), lalu "Web dan fitur ini pun masuk fase 1 dan harus"; setelah klarifikasi: **Y** (company profile + member area korporat dibangun di Fase 1; Fase 1 mundur atau sebagian task pindah ke Fase 2, rincian diajukan dulu) |
| P0-2 | Siapa login di member area | **K**: HR/admin perusahaan + karyawan peserta |

## Putaran 1

| Q | Pertanyaan | Jawaban |
| --- | --- | --- |
| P1-1 | Halaman company profile | **A**: Beranda, Tentang Kami, Layanan (7 layanan + korporat), Untuk Perusahaan, Rekanan, Artikel/berita, Kontak, Unduh aplikasi, Kebijakan privasi & syarat |
| P1-2 | Layanan korporat rilis ini | **V, M, L**: vaksinasi massal, MCU massal, pemeriksaan lab massal (non-MCU) |
| P1-3 | Pelaksana di lokasi perusahaan | **R**: klinik/lab rekanan (Ruang Sehat sebagai perantara) |
| P1-4 | Pembayaran korporat | **I**: invoice B2B (transfer, termin/jatuh tempo), tanpa payment gateway |

## Putaran 2

| Q | Pertanyaan | Jawaban |
| --- | --- | --- |
| P2-1 | Cara perusahaan menjadi member | **DA**: daftar sendiri di web lalu diverifikasi admin Ruang Sehat (dokumen NIB/NPWP), **dan** dibuatkan admin/sales Ruang Sehat setelah kesepakatan |
| P2-2 | Alur pemesanan | **K**: HR memilih paket dari katalog harga tetap, lalu langsung memesan (bukan alur penawaran) |
| P2-3 | Data dan akun peserta | **U**: HR unggah daftar peserta (Excel/CSV: nama, email/HP, tanggal lahir; NIK hanya bila wajib untuk layanan); peserta menerima undangan email dan memakai akun pasien Ruang Sehat yang sama |
| P2-4 | Siapa melihat hasil MCU/lab | **PK**: hasil lengkap hanya untuk peserta; HR melihat rekap kehadiran, dan ringkasan (mis. fit/unfit) hanya bila peserta memberi persetujuan. Tafsiran penyusun (dikonfirmasi di rekap): tanpa persetujuan peserta, HR hanya melihat rekap kehadiran |

## Putaran 3

| Q | Pertanyaan | Jawaban |
| --- | --- | --- |
| P3-1 | Penetapan harga katalog korporat | **M**: harga dari rekanan (integrasi/portal rekanan) + markup Ruang Sehat, mengikuti aturan markup ADR-015 |
| P3-2 | Lokasi pelaksanaan | **OK**: on-site di kantor perusahaan atau di klinik/lab rekanan, dipilih per pesanan |
| P3-3 | Syarat invoice bawaan | **D**: invoice terbit saat pesanan dikonfirmasi, jatuh tempo 14 hari, layanan dijadwalkan setelah lunas |
| P3-4 | Pengelola konten company profile | **C**: admin Ruang Sehat lewat CMS (US-1101) |
