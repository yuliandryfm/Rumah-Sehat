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

## Putaran 4: keputusan atas analisis (§17 `docs/cr/CR-021-analisis.md`)

Jawaban `[USER]` Telegram 2026-11-19 15:35 UTC: "1S3 2Y 3Y 4aV 4bY4cT".

| Q | Keputusan |
| --- | --- |
| 1 | **Skenario S3**: Fase 1 tetap selesai 2026-12-11; 7 task Fase 1 pindah ke Fase 2; pemesanan, peserta, dan invoice korporat di Fase 3. Fase 1 memuat website, CMS, pendaftaran dan verifikasi perusahaan, akun HR |
| 2 | Semua usulan kontradiksi disetujui: K-01 (a) peserta memakai aplikasi mobile; K-02 menunggu L-21, bila (a) butuh izin BI maka (b) dua tagihan + pengecualian AB-13; K-03 BR-141 pengecualian korporat dari pembayaran lewat PG; K-04 (a) harga lewat integrasi; K-05 hanya vaksinasi korporat yang maju, vaksinasi pasien tetap CR-015; K-12 dasar unggah HR ditetapkan legal (L-17); K-06–K-11 disetujui (CMS ke Fase 1, revisi ADR-006/011/018/024, unggah CSV/XLSX, tafsiran on-site, BR-126 lewat PT-74) |
| 3 | UP-79–UP-87 disetujui semua (termasuk UP-83, UP-84, UP-87 sebagai syarat keamanan) |
| 4a | PT-67: **Verifikator** yang memverifikasi/membuat akun perusahaan dan memulai reset MFA HR. Sub-pertanyaan PT-67 lain (perlu sub-peran baru? persetujuan orang kedua untuk perusahaan buatan admin, M-49) **belum terjawab** dan tetap terbuka |
| 4b | PT-79: **wajib** bukti kewenangan pendaftar (jawaban "Y"). Bentuk bukti (misalnya surat kuasa atau surat penunjukan dari perusahaan) adalah **tafsiran penyusun**; opsi §17 hanya menyebut "bukti kewenangan pendaftar" dan contoh di PT-79 "surat penunjukan atau konfirmasi lewat telepon kantor". Bentuk final dikonfirmasi user |
| 4c | PT-72: NIB sudah terdaftar → **tolak** (jawaban "T"). "Arahkan menghubungi Layanan Pelanggan" adalah **tafsiran penyusun** (pola M-52: tanggapan generik, konflik diselesaikan admin di luar layar), dikonfirmasi user |

Catatan fact-check `docs/fact-check/CR-021.md` N-3: jawaban putaran 4 berupa kode singkat; rincian di luar teks opsi §17 ditandai tafsiran.

## Putaran 5: konfirmasi tafsiran (koreksi fact-check `docs/fact-check/CR-021-penerapan.md` B-1)

Jawaban `[USER]` Telegram 2026-11-19 23:13 UTC: "4BY 4CY".

| Q | Tafsiran yang dikonfirmasi |
| --- | --- |
| 4b | Bukti kewenangan pendaftar berupa surat kuasa atau surat penunjukan dari perusahaan (ditandatangani pejabat berwenang), diunggah saat daftar, dicek Verifikator |
| 4c | NIB sudah terdaftar → pendaftaran ditolak dengan pesan umum dan arahan menghubungi Layanan Pelanggan, tanpa memberi tahu akun mana yang memakai NIB itu |

Dengan konfirmasi ini, label "tafsiran penyusun yang dikonfirmasi user" di BR-136 dan US-1901 AC1/AC7 menjadi benar, dan PT-72 tetap terjawab.
