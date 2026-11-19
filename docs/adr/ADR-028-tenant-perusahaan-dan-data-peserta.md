# ADR-028: Tenant perusahaan, data peserta korporat, impor berkas, dan persetujuan berbagi ringkasan hasil

Status: **Diusulkan** (CR-021, 2026-11-19; revisi setelah review keamanan putaran 1 B-03 dan B-04 serta review kepatuhan B-1, `docs/review/CR-021-security.md`, `docs/review/CR-021-kepatuhan.md`; menunggu cek ulang review dan persetujuan user bersama `docs/cr/CR-021-analisis.md` v0.2; bagian pemrosesan data nyata menunggu konfirmasi legal L-17 dan L-18)
Merevisi bila diterima: ADR-012 butir 1 dan butir 2, ADR-013 butir 3 (wadah data), ADR-018 (jenis berkas)
Tanggal: 2026-11-19

## Konteks

- HR mengunggah daftar peserta (Excel/CSV: nama, email/HP, tanggal lahir; NIK hanya bila wajib untuk layanan); peserta menerima undangan email dan memakai akun pasien yang sama `[USER] P2-3`.
- Hasil lengkap hanya untuk peserta; HR melihat rekap kehadiran; ringkasan (misalnya fit/unfit) hanya bila peserta memberi persetujuan `[USER] P2-4`.
- ADR-012 menetapkan RBAC + relasi dengan default deny, isolasi tenant di lapisan aplikasi untuk database Ruang Sehat (tanpa RLS), data sensitif hanya lewat fungsi akses yang menulis E-05, dan 404 untuk objek terlarang `[DOK: adr/ADR-012-otorisasi-dan-isolasi-data.md]`.
- UU PDP menggolongkan data kesehatan sebagai data pribadi spesifik `[SRC-15]` (via BRD BR-105). ADR-013 menetapkan enkripsi tingkat aplikasi per wadah, NIK dalam tiga bentuk, dan crypto-shredding `[DOK: adr/ADR-013-perlindungan-data-pribadi-spesifik.md]`.
- ADR-018 dan NFR-SEC-08 hanya mengizinkan JPG/PNG dan PDF `[DOK: adr/ADR-018-berkas-unggahan.md]` `[DOK: 03-prd.md]`.
- Dasar pemrosesan data karyawan yang diunggah HR dan keabsahan persetujuan berbagi ringkasan dalam relasi kerja `[TIDAK DIKETAHUI]` (L-17, L-18). UU PDP mengenal enam dasar pemrosesan (Pasal 20 ayat (2) `[SRC-440]`), dan pemberi kerja wajib menyelenggarakan pemeriksaan kesehatan tenaga kerja dengan pelaporan hasil kepada otoritas terkait (abstrak PER-02/MEN/1980 `[SRC-441]`), sehingga BR-143 dapat berbenturan dengan kewajiban itu (L-18).
- UU PDP mendefinisikan data kesehatan sebagai keterangan individu yang berkaitan dengan kesehatan atau "pelayanan kesehatan" `[SRC-440]`; status keikutsertaan dan kehadiran peserta karena itu diperlakukan sensitif sampai legal menjawab (L-08, L-18).
- Review keamanan putaran 1 menemukan dua celah Tinggi: kanal turunan yang membocorkan status kesehatan ke HR tanpa persetujuan (KOR-09) dan undangan yang dapat tertaut ke orang yang salah (KOR-12) `[DOK: review/CR-021-security.md]`.

## Keputusan

1. **Relasi otorisasi baru "tenant perusahaan"** (perluasan ADR-012 butir 1) `[KEPUTUSAN-254]`:
   - HR atas data perusahaannya. `corporate_client_id` diambil dari keanggotaan aktif di sesi, bukan dari parameter permintaan.
   - Repository modul Korporat mewajibkan `CorporateScope`. Setiap endpoint `/v1/corporate/*` punya test isolasi otomatis di `scripts/verify.sh` yang mencoba mengakses objek perusahaan lain lewat ID (NFR-SEC-06 diperluas).
   - RLS tidak dipakai di database aplikasi, konsisten dengan ADR-012 butir 5 dan alasan penolakannya.
   - Satu akun HR hanya punya satu keanggotaan aktif (UQ parsial di `corporate_member`); pencabutan atau perpindahan keanggotaan mencabut sesi akun itu (M-51).
   - Harness isolasi mencakup ekspor, unduhan invoice dan berkas, pratinjau impor, endpoint ID anak (keikutsertaan, undangan), job asinkron pembangkit berkas, dan route handler BFF `web/korporat` (H-18).
   - Rekap kehadiran dan data peserta diperlakukan sebagai data sensitif (akses detail oleh selain subjek mencatat E-05, penolakan dicatat gagal) sampai L-08/L-18 menjawab klasifikasinya.
2. **Data peserta terpisah dari data pasien** `[KEPUTUSAN-255]`:
   - Data unggahan HR disimpan di `corporate_participant` (kelas Master, PII dapat dianonimkan), bukan di `care_recipient`.
   - Penautan ke akun pasien hanya terjadi ketika peserta yang sudah masuk di realm `pasien` membuka tautan undangannya dan menerima. Tidak ada pencocokan otomatis berdasarkan email. Token undangan sekali pakai, disimpan sebagai hash, berumur PB-24.
   - Penautan dicatat sebagai baris transaksi (`corporate_invitation_response`) yang merujuk `care_recipient` peserta.
   - HR tidak pernah membaca data akun pasien peserta; HR hanya melihat data yang diunggahnya dan status undangan.
   - **Pengikatan undangan ke orang yang benar** (M-43) `[KEPUTUSAN-267]`:
     - undangan hanya dapat diterima akun pasien yang email terverifikasinya sama dengan email undangan (UP-83). Ini **syarat keamanan**; bila user menolaknya, dicatat sebagai risiko sisa yang diterima user (KOR-12);
     - tautan undangan memakai App Links/Universal Links terverifikasi untuk domain resmi, bukan skema URL kustom; token hanya ditukar setelah peserta masuk di realm `pasien`; mekanisme platform diverifikasi saat task;
     - konfirmasi tanggal lahir saat menerima undangan, dengan batas percobaan, **diajukan ke user** (PT-78), bukan diputuskan di ADR ini;
     - rilis hasil lengkap ke akun tertaut hanya setelah identitas peserta dicocokkan di lokasi layanan **diajukan ke user** (UP-87; bergantung PT-64, PT-66);
     - jumlah undangan aktif tidak melebihi jumlah peserta pesanan, kirim ulang dibatasi laju, templat tanpa teks bebas dari HR, dan nama peserta dikodekan sebagai teks (M-50, M-54).
   - Email undangan memuat pemberitahuan pemrosesan data (siapa mengirim data, tujuan, cara mengajukan hak subjek data); teks menunggu L-17 dan PT-76.
3. **Ringkasan hasil sebagai data sensitif bersyarat persetujuan** (perluasan ADR-012 butir 2) `[KEPUTUSAN-257]`:
   - Persetujuan per peserta, per perusahaan, per pesanan, dicatat append-only di `corporate_share_consent_record` (kelas Log) dengan versi teks; bawaan tidak berbagi; penarikan berlaku ke depan.
   - HR membaca ringkasan hanya lewat fungsi akses sensitif yang memeriksa persetujuan aktif pada saat baca, menulis E-05 sebelum data dikirim, dan mencatat penolakan sebagai hasil gagal.
   - Hanya akun pasien yang tertaut ke peserta yang dapat memberi atau menarik persetujuan.
   - Hasil lengkap tidak punya jalur baca untuk HR maupun admin Ruang Sehat.
   - **Invarian kanal turunan** (M-42) `[KEPUTUSAN-266]`: HR hanya menerima data yang berasal dari hasil (nilai, ringkasan, ketersediaan hasil, status tindak lanjut, agregat, atau baris tagihan yang bergantung pada hasil) untuk peserta yang persetujuannya aktif saat permintaan. Konsekuensinya:
     - rekap kehadiran tidak memuat status hasil, "perlu tindak lanjut", atau "jadwal ulang karena hasil";
     - rilis ini tidak menyediakan agregat hasil bagi HR; agregat yang ditambahkan kelak lewat CR hanya dihitung atas data yang memang terlihat HR dan tidak ditampilkan untuk kelompok di bawah ambang minimum;
     - baris invoice tidak menyebut peserta dan tidak dapat dikaitkan ke peserta tanpa persetujuan aktif;
     - BFF dan browser tidak menyimpan respons berisi ringkasan atau rekap di cache (`no-store`); ekspor dibangkitkan saat diminta dan tidak disimpan di server;
     - setiap kanal punya test negatif (US-1909 AC6, US-1912 AC5–AC6, US-1914 AC7–AC8).
   - Teks persetujuan menyatakan bahwa penarikan berlaku ke depan dan tidak menarik ekspor yang sudah diunduh HR.
4. **Enkripsi** (perluasan ADR-013 butir 3) `[KEPUTUSAN-258]`: wadah baru `corporate_participant_identity` (NIK peserta, bentuk tiga ADR-013 §4), `corporate_import_preview` (baris pratinjau impor), dan `corporate_result_summary` (isi ringkasan). HR hanya melihat 4 digit terakhir NIK.
5. **Minimisasi**: NIK hanya diterima bila paket menandai `requires_nik` (PT-66). Data yang dikirim ke rekanan pelaksana dibatasi data minimal setelah L-17 dijawab, dan dicatat E-17 (ADR-013 butir 7, ADR-014 butir 9).
6. **Impor berkas CSV/XLSX** (perluasan ADR-018) `[KEPUTUSAN-256]`:
   - unggah lewat upload slot privat; validasi isi berkas, bukan ekstensi atau header `Content-Type`; batas ukuran dan baris PB-23; berkas terenkripsi atau terkunci password ditolak; pembacaan XLSX dibatasi sumber dayanya;
   - diproses dua langkah: pratinjau validasi per baris (terenkripsi, kelas Operasional berumur), lalu impor baris valid setelah HR mengonfirmasi;
   - berkas asli dan baris pratinjau dihapus setelah impor, pembatalan, atau kedaluwarsa; log aplikasi tidak memuat isi baris; E-18 mencatat jumlah, bukan isi;
   - isi sel yang dapat ditafsirkan sebagai rumus diperlakukan sebagai teks, dan karakter awal `=`, `+`, `-`, `@`, tab, CR, dan LF dinetralkan di **semua** ekspor CSV (rekap, backoffice, portal) `[SRC-407]` (M-54, memperluas M-18);
   - kolom di luar skema dibuang saat pratinjau dan tidak pernah disimpan (minimisasi, Pasal 16 ayat (2) huruf a `[SRC-440]`);
   - pengerasan XLSX `[KEPUTUSAN-269]` (M-53, M-55): penguraian di worker dengan batas waktu dan memori, bukan di proses `api`; batas ukuran setelah dekompresi, jumlah entri, dan rasio kompresi diperiksa sebelum mengurai `[SRC-409]`; pemrosesan DTD dimatikan `[SRC-405]` `[SRC-406]`; `.xlsm`, wadah OLE, dan entri `vbaProject.bin` ditolak; berkas mentah disimpan di bucket impor tanpa versi objek dengan aturan kedaluwarsa siklus hidup, dikecualikan dari backup object storage atau berumur sama; pesan galat per baris tidak menggemakan isi sel;
   - pustaka pembaca XLSX diverifikasi di registry resmi saat task (anti-halu §5), belum dipilih.
7. **Subjek data audit**: event yang menyangkut peserta sebelum tertaut memakai `corporate_participant` sebagai subjek; setelah tertaut, `care_recipient` (perluasan B3).

## Alternatif yang ditolak

- **Menyimpan peserta langsung di `care_recipient` akun pasien yang emailnya cocok**: HR dapat menautkan data ke akun orang lain karena salah ketik atau sengaja; peserta kehilangan kendali atas penautan (RT-40).
- **Akun pasien dibuat otomatis dari unggahan HR**: akun tercipta tanpa persetujuan US-104 dari subjeknya, bertentangan dengan US-101 AC4 dan ADR-011 butir 6.
- **RLS untuk tabel korporat**: konsisten dengan penolakan ADR-012 untuk database Ruang Sehat; satu pesanan korporat juga melibatkan rekanan dan pasien, sehingga konteks satu tenant tidak cukup.
- **Persetujuan berbagi tingkat perusahaan (sekali untuk semua pesanan)**: peserta tidak dapat memilih per layanan; ringkasan vaksinasi dan MCU berbeda sifatnya (PT-65).
- **Menyimpan berkas impor asli untuk audit**: menambah salinan data pribadi tanpa kebutuhan; jumlah dan hasil impor cukup dicatat E-18.
- **Persetujuan berbagi dicatat di `consent_record` pasien yang sudah ada**: ruang lingkup (perusahaan dan pesanan) tidak tercakup struktur itu; tabel terpisah dipakai dengan pola yang sama.

## Konsekuensi

- Modul kritis baru: otorisasi tenant perusahaan, fungsi akses ringkasan, impor data peserta.
- DPIA (BR-110) mencakup data karyawan dan ringkasan hasil, dan diusulkan menjadi syarat gerbang G-26 (data peserta nyata pertama), karena MCU massal dapat memenuhi dua pemicu Pasal 34 ayat (2) huruf b–c (data spesifik, skala besar) `[SRC-440]`.
- Kanal hak subjek data bagi peserta yang tidak tertaut (PT-76) dan aturan peserta anak (PT-77) belum ada; keduanya syarat sebelum data peserta nyata diproses.
- Bila user menolak UP-83 atau PT-78/UP-87, salah tautan dicatat sebagai risiko sisa yang diterima user (KOR-12).
- Retensi tabel baru mengikuti PT-24; data peserta yang tidak pernah menerima undangan menunggu PT-71.
- NFR-SEC-08 dan ADR-018 direvisi untuk CSV/XLSX.
- Pemrosesan data peserta nyata dan penayangan ringkasan untuk HR tidak dijalankan sebelum L-17 dan L-18 dijawab (gerbang G-26 usulan); pembangunan dengan data uji tetap dapat berjalan.
