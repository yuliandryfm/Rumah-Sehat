# ADR-013: Perlindungan data pribadi spesifik, yaitu enkripsi berlapis, minimisasi, retensi, dan crypto-shredding

Status: Diusulkan
Tanggal: 2026-11-18

## Konteks

- NFR-SEC-05: data dienkripsi saat dikirim dan saat disimpan. Data pribadi spesifik (AB-70: data kesehatan, data anak, data keuangan pribadi) mendapat perlindungan tambahan yang ditetapkan di Tahap 4 `[DOK: 03-prd.md]`.
- UU PDP menggolongkan data kesehatan, data anak, dan data keuangan pribadi sebagai data spesifik, serta mewajibkan keamanan, kerahasiaan, dan pengakhiran/penghapusan/pemusnahan data `[SRC-15]`.
- Setiap jenis data punya masa retensi yang dapat diatur dan dijalankan otomatis (NFR-RET-01, US-1606). Angkanya `[TIDAK DIKETAHUI]` (PT-24). Penghapusan atas permintaan (US-1605) terblokir sampai L-06 dan PT-24 terjawab `[DOK: 03-prd.md]`.
- ADR-003 mensyaratkan prosedur anonimisasi PII dengan catatan transaksi tetap utuh. ADR-004 melarang `UPDATE`/`DELETE` data transaksi `[DOK: adr/]`.
- NIK pemilik akun tidak boleh dipakai sebagai pemilik akun di akun lain, dan NIK anggota keluarga tidak boleh ganda di akun yang sama (UP-40, US-106 AC4, US-108 AC4–AC5). Audit dan tampilan peran non-Verifikator hanya menampilkan 4 digit terakhir NIK (§3.1, §7.3) `[DOK: 03-prd.md]`.
- Data yang dikirim ke payment gateway, rekanan, dan kanal notifikasi dibatasi pada yang diperlukan (NFR-PRIV-03, AB-73, AB-75) `[DOK: 03-prd.md]`.
- Semua data dan penyedia di Indonesia `[USER] T4-Q1`.

## Keputusan

**1. Enkripsi saat dikirim.** Semua lalu lintas eksternal dan internal memakai TLS: aplikasi ke backend, backend ke database, rekanan, penyedia, dan SATUSEHAT. Versi minimum dan cipher mengikuti checklist ASVS Level 2 (ADR-011), ditetapkan di Tahap 5 `[KEPUTUSAN-28]`.

**2. Enkripsi saat disimpan, lapis penyedia.** Disk database, backup, dan object storage dienkripsi oleh penyedia cloud dengan kunci yang dikelola di layanan manajemen kunci (KMS) region Indonesia. Kemampuannya diverifikasi saat pemilihan penyedia di Tahap 5 `[KEPUTUSAN-28]`.

**3. Enkripsi tingkat aplikasi (envelope encryption) untuk data pribadi spesifik** `[KEPUTUSAN-28]`:
- Kolom yang dienkripsi: NIK; keluhan awal; teks pesan chat; isi catatan konsultasi; item resep; nilai hasil lab; isi rekam medis RME minimalis; deskripsi klaim; rahasia TOTP; kredensial rahasia rekanan dan SATUSEHAT yang disimpan sistem.
- Setiap **wadah data** punya kunci data (DEK) sendiri. Wadah adalah satu sesi konsultasi (chat, catatan, resep), satu hasil lab, satu kunjungan RME, satu dokumen identitas, atau satu klaim. DEK disimpan terbungkus oleh kunci utama (KEK) di KMS, di tabel kunci data.
- Data anak dan data keuangan pribadi yang bukan teks klinis (misalnya nama anak dan nominal transaksi) tidak dienkripsi di tingkat aplikasi. Perlindungannya lewat otorisasi (ADR-012), audit E-05, dan enkripsi lapis penyedia, karena kolom ini dibutuhkan untuk pencarian dan laporan.
- Alasannya: administrator database, salinan backup, dan log query tidak dapat membaca isi klinis tanpa akses ke KMS.

**4. NIK** disimpan dalam tiga bentuk `[KEPUTUSAN-28]`:
- terenkripsi, untuk Verifikator dan pencatatan rekam medis bila diperlukan;
- *blind index* HMAC-SHA256 dengan kunci rahasia terpisah `[SRC-46]`, untuk pemeriksaan keunikan UP-40 tanpa mendekripsi;
- 4 digit terakhir terbuka, untuk tampilan termasking dan audit.
Validasi format: 16 digit angka, sesuai keterangan resmi Ditjen Dukcapil `[SRC-63]`. Teks peraturannya belum dibuka (US-106 AC2).

**5. Crypto-shredding untuk pemusnahan dan anonimisasi tanpa mengubah baris transaksi** `[KEPUTUSAN-29]`:
- Memusnahkan isi wadah data dilakukan dengan menghapus DEK-nya. Baris transaksi tetap utuh (ADR-004), tetapi isinya tidak dapat dibaca lagi.
- Berkas terkait di object storage dihapus, termasuk versi lamanya (ADR-018).
- Anonimisasi PII di data master (ADR-003) dilakukan dengan menimpa kolom PII di tabel master (nama, kontak, alamat, NIK, dan blind index-nya). Kunci dokumen identitas dihapus. Baris transaksi tidak diubah, karena transaksi hanya merujuk data master lewat ID, atau menyimpan snapshot data pribadi di tabel snapshot kelas Master yang ikut dianonimkan.
- Hanya role **pekerja privasi** yang boleh menghapus DEK dan menjalankan anonimisasi, bukan role aplikasi biasa. Setiap tindakannya dicatat E-16.

**6. Retensi yang dapat diatur** `[KEPUTUSAN-29]`:
- Tabel kebijakan retensi (kelas Master) menyimpan masa retensi dan tindakan akhir (akhiri pemrosesan, anonimkan, atau musnahkan lewat crypto-shredding) per kategori data.
- Nilai kosong berarti data tidak diproses retensi sampai ditetapkan (PT-24). Job harian menjalankan kebijakan dan menyimpan laporan hasilnya (US-1606).
- Wadah data menyimpan kategori dan tanggal acuan retensinya.

**7. Minimisasi data keluar** `[KEPUTUSAN-28]`:
- Isi notifikasi tanpa data kesehatan rinci (AB-73).
- Payment gateway menerima ID pesanan, nominal, dan deskripsi umum. Data pembayar hanya yang wajib menurut penyedia, ditetapkan di Tahap 5 bersama L-08.
- Rekanan menerima data penerima layanan minimal yang ditandai di kontrak integrasi (ADR-014), menunggu L-08.
- Setiap pengiriman data pribadi ke pihak luar dicatat E-17 (jenis data dan tujuan, tanpa isi).

**8. Pengelolaan rahasia**: kredensial dan kunci disimpan di layanan rahasia terkelola (secret manager) region Indonesia. Rahasia tidak ada di repositori, image, atau log. Rotasi KEK dan kunci blind index punya prosedur tertulis sebelum go-live `[KEPUTUSAN-28]`.

## Alternatif yang ditolak

- **Hanya enkripsi lapis penyedia**: tidak memenuhi "perlindungan tambahan" NFR-SEC-05, karena isi klinis tetap terbaca oleh siapa pun yang memegang akses database atau backup.
- **Enkripsi di database (misalnya fungsi kriptografi di PostgreSQL)**: kunci ikut melewati server database, sehingga tidak melindungi dari pemegang akses database.
- **Hard delete baris transaksi saat retensi habis**: bertentangan dengan ADR-004. Kalau legal mewajibkan penghapusan fisik baris (L-06), hal itu diajukan sebagai change request atas ADR-004.
- **Satu DEK per pasien**: pemusnahan per wadah tidak mungkin, padahal masa retensi dihitung per catatan.

## Konsekuensi

- Isi klinis tidak dapat dicari dengan query database. Rilis 1 memang tidak membutuhkan pencarian isi klinis.
- KMS menjadi ketergantungan kritis. DEK disimpan sementara di memori proses dengan batas waktu untuk menekan panggilan ke KMS. Kehilangan KEK berarti kehilangan data, sehingga prosedur backup kunci mengikuti kemampuan KMS penyedia (Tahap 5).
- Pustaka enkripsi, pekerja privasi, dan job retensi adalah modul kritis (T1).
- Apakah data yang dianonimkan atau yang di-*crypto-shred* memenuhi kewajiban "menghapus atau memusnahkan" ditanyakan ke legal bersama L-06.
