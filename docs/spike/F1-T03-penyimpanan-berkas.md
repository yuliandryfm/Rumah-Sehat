# Spike F1-T03: penyimpanan berkas untuk lokal/uji dan verifikasi NEO Object Storage

Tanggal: 2026-11-19. Kelas: T2. Rujukan: ADR-018, `docs/05-stack.md` §13.2 butir 8, `docs/07-phase-plan.md` §7.3.
Tidak ada kode yang ditambahkan ke repositori. Eksperimen dijalankan di scratchpad (Docker, tanpa memasang alat di sistem). Probe berupa satu skrip boto3 (21 pemeriksaan) yang dijalankan di kontainer `python:3.13-slim`; hasilnya diringkas di §3.

## 1. Temuan kemampuan NEO Object Storage

Dokumentasi resmi Biznet Gio sangat tipis untuk butir yang diminta. Hanya presigned GET yang terdokumentasi. Yang tidak ditemukan ditulis `[TIDAK DIKETAHUI]` dan masuk §5.

| Butir | Temuan | Sumber |
| --- | --- | --- |
| Kompatibilitas | Mengklaim "100% S3 Compatibel"; S3 Native API, semua fungsi aktif default. Daftar operasi yang didukung tidak ditulis | `[SRC-97]` `[SRC-305]` |
| Endpoint | Dokumentasi menyebut `nos.wjv-1.neo.id` dan `nos.jkt-1.neo.id`. Mana yang berlokasi di Indonesia dan mana yang dipakai untuk bucket kita `[TIDAK DIKETAHUI]` (ditanyakan ke Biznet Gio atau dilihat di portal NEO) | `[SRC-98]` `[SRC-305]` `[SRC-307]` `[SRC-308]` |
| HTTPS | Tersedia. Kedua host melayani HTTPS dengan sertifikat wildcard valid; TLS 1.2 diterima di keduanya, TLS 1.3 hanya di jkt-1 (handshake gagal di wjv-1), TLS 1.1 ditolak. Contoh resmi tidak konsisten: contoh Python memakai `http://`, contoh Ruby, rclone, dan nginx memakai `https://`. Port 80 juga menjawab, jadi HTTP polos tersedia dan harus ditolak oleh konfigurasi kita. Sertifikat jkt-1 berlaku hingga 2026-12-30, perlu dipantau perpanjangannya | `[SRC-309]` `[SRC-98]` `[SRC-307]` `[SRC-308]` |
| Gaya pengalamatan | Contoh S3cmd memakai virtual-host (`<bucket>.nos.wjv-1.neo.id`), dan sertifikat wildcard `*.nos.<zona>.neo.id` konsisten dengan itu. Dukungan path-style `[TIDAK DIKETAHUI]` | `[SRC-305]` `[SRC-309]` |
| Mesin penyimpanan | Header `server: CloudianS3` (observasi, bukan dokumentasi Biznet). Dokumentasi Cloudian (docs.cloudian.com) tidak dapat dibuka dari mesin ini (koneksi TLS terputus), jadi kemampuan mesin tidak diverifikasi lewat sumber vendor mesin | `[SRC-309]` |
| Presigned GET | Terdokumentasi (boto3 `generate_presigned_url('get_object', ...)`, `ExpiresIn` dalam detik, contoh 60) | `[SRC-98]` |
| Presigned PUT | Tidak terdokumentasi di halaman mana pun yang dibuka (KB Python, PHP, Node.js/Ruby, S3 API). `[TIDAK DIKETAHUI]` | `[SRC-98]` `[SRC-307]` |
| Presigned POST (policy) | Tidak terdokumentasi. CORS KB menyebut PUT dan POST sebagai metode unggah, tetapi itu soal CORS, bukan bukti presigned. `[TIDAK DIKETAHUI]` | `[SRC-306]` |
| Signature version dan region | Tidak dinyatakan. Region dalam contoh tidak konsisten: kosong dengan "v4 signatures" (rclone), `idn`, `id-jkt-1`, `ap-southeast-1`. Nilai region untuk SigV4 `[TIDAK DIKETAHUI]` | `[SRC-307]` `[SRC-308]` |
| Batas ukuran objek dan multipart | Tidak dinyatakan ("skalabilitas tak terbatas" hanya soal kapasitas). Batas kita (foto 5 MB, PDF 10 MB) kecil, jadi risikonya rendah, tetapi batas PUT tunggal `[TIDAK DIKETAHUI]` | `[SRC-305]` `[SRC-97]` |
| Versioning | Didukung ("untuk semua region"), diaktifkan lewat S3 Browser; ada restore versi lama. Tidak ada keterangan lewat API, cara menghapus semua versi (syarat E-16), atau lifecycle versi lama | `[SRC-304]` |
| Object Lock | Mode Governance, Compliance, Legal Hold; hanya dapat diaktifkan saat bucket dibuat, tidak bisa pada bucket yang sudah ada. Perintah CLI `create-bucket --object-lock-enabled-for-bucket` terdokumentasi (artikel memuat salah ketik) | `[SRC-99]` |
| Lifecycle (hapus otomatis berkas tak terkonfirmasi) | Tidak terdokumentasi. `[TIDAK DIKETAHUI]` | `[SRC-304]` `[SRC-305]` |
| CORS | Didukung; contoh mengizinkan PUT dan POST; origin `http` dan `https` ditulis terpisah | `[SRC-306]` |
| Enkripsi di sisi penyedia | Tidak ditemukan; tetap `[TIDAK DIKETAHUI]` (lihat stack §14) | `[SRC-97]` |
| Harga | Rp1.000/GB/bulan single region; biaya transfer keluar dan permintaan tidak disebut `[TIDAK DIKETAHUI]` | `[SRC-97]` |

Tidak tercakup di spike ini: kompatibilitas pgBackRest dan Loki dengan NEO (bagian lain dari stack §13.2 butir 8); keduanya tetap terbuka.

## 2. Kandidat emulator S3 untuk lokal dan CI

Semua data akses di tabel diambil 2026-11-19. Digest adalah digest manifest yang ditarik ke mesin ini.

| Kandidat | Lisensi | Status pemeliharaan | Image dan digest | Hasil uji | Sumber |
| --- | --- | --- | --- | --- | --- |
| **SeaweedFS** | Apache-2.0 | Aktif; repositori sejak 2014; rilis 4.54 (2026-11-16) | `chrislusf/seaweedfs:4.54` `sha256:4b42c212543a0acd4c0e675215ecdcaf62a186ef98def76c7e12701871222d15` (1,08 GB) | Lulus 21/21 | `[SRC-312]` |
| **RustFS** | Apache-2.0 | Aktif; repositori sejak 2023; rilis 1.0.0 baru 2026-09-16, 1.0.6 (2026-11-15); rilis sangat sering | `rustfs/rustfs:1.0.6` `sha256:ee717072290bec9b11abfc0365f8310b00df1cd9b3639943edae47bdef05044a` (487 MB) | Lulus 21/21 | `[SRC-311]` |
| Adobe S3Mock | Apache-2.0 | Aktif; rilis 5.2.3 (2026-09-19) | `adobe/s3mock:5.2.3` `sha256:ab01a6946750f451ca215a47e91030695b260e4003b8a5a6201d25029b8fca92` | Tidak memvalidasi tanda tangan, kedaluwarsa, Content-Type/Length, policy POST; akses anonim ke bucket diterima. Tidak layak untuk test keamanan | `[SRC-313]` |
| Versity Gateway | Apache-2.0 | Aktif; rilis v1.11.0 (2026-11-06) | `versity/versitygw:v1.11.0` `sha256:8319a530d08d537426e2f855bae5280709795dc73a3f0c347973ef915d6343c5` | Penegakan URL benar, tetapi versioning dan lifecycle ditolak 501 pada konfigurasi default (opsi versioning tidak dicoba) | `[SRC-313]` |
| MinIO | AGPL-3.0 | Repositori diarsipkan, "NO LONGER MAINTAINED" | tidak dipakai | tidak diuji | `[SRC-310]` |
| LocalStack | NOASSERTION (API GitHub) | Repositori community diarsipkan | tidak dipakai | tidak diuji | `[SRC-310]` |
| Garage | tidak diverifikasi di spike ini | Aktif (halaman dokumentasi terbuka) | tidak dipakai | Tanpa PutBucketVersioning ("Missing"), jadi tidak memenuhi ADR-018 butir 5 | `[SRC-313]` |

Rujukan konfigurasi uji yang berhasil (hanya untuk lokal, kredensial uji statis):
- SeaweedFS: `server -dir=/data -s3 -s3.port=<port> -s3.config=/etc/s3.json -master.volumeSizeLimitMB=64`, dengan `s3.json` berisi satu identitas (aksi Admin, Read, Write, List, Tagging).
- RustFS: env `RUSTFS_ACCESS_KEY`, `RUSTFS_SECRET_KEY`, `RUSTFS_ADDRESS=:<port>`, `RUSTFS_CONSOLE_ENABLE=false`, argumen `/data`. Bind-mount direktori host ke `/data` gagal ("File access denied") karena izin tulis; pakai named volume atau atur kepemilikan.

## 3. Hasil probe pada emulator (path-style, SigV4, region `us-east-1`)

| Pemeriksaan | SeaweedFS 4.54 | RustFS 1.0.6 | S3Mock 5.2.3 | Versity v1.11.0 |
| --- | --- | --- | --- | --- |
| Presigned PUT dasar | 200 | 200 | 200 | 200 |
| PUT, `Content-Type` bertanda tangan, tipe cocok / tidak cocok | 200 / 403 | 200 / 403 | 200 / 200 | 200 / 403 |
| PUT, `Content-Length` bertanda tangan, cocok / tidak cocok | 200 / 403 | 200 / 403 | 200 / 200 | 200 / 403 |
| Presigned POST (policy ukuran 1-5000 B + tipe): sesuai / terlalu besar / tipe salah | 204 / 400 / 403 | 204 / 400 / 400 | 200 / 200 / 200 | 204 / 400 / 403 |
| URL kedaluwarsa (2 detik, uji setelah 4 detik) | 403 | 403 | 200 | 403 |
| Versioning aktif, 2 PUT menghasilkan 2 versi | ya | ya | ya | 501 |
| Hapus semua versi dan delete marker per VersionId | bersih | bersih | bersih | tidak berlaku |
| Lifecycle (Expiration, AbortIncompleteMultipartUpload, NoncurrentVersionExpiration) put/get | ya | ya | ya | 501 |
| CORS put | ya | ya | galat aneh (409) | ya |
| Presigned GET dengan `ResponseContentDisposition` attachment | header diterapkan | diterapkan | diterapkan | diterapkan |
| PUT 12 MB lewat presigned | 200 | 200 | 200 | 200 |
| GET anonim objek private | 403 | 403 | 200 | 403 |
| Buat bucket dengan Object Lock | ya | ya | ya | ya |

Catatan: penegakan Object Lock tidak diuji. Virtual-host style tidak diuji pada emulator. Hasil emulator membuktikan kode kita, bukan perilaku NEO.

## 4. Rekomendasi

1. **Compose lokal (F1-T07) dan CI**: pakai SeaweedFS 4.54 dengan digest dipin; RustFS 1.0.6 sebagai cadangan bila SeaweedFS bermasalah (lebih ringan, satu proses, tetapi rilis 1.0 baru dua bulan) `[KEPUTUSAN-233]`. Jangan memakai S3Mock untuk test yang menyangkut pembatasan URL.
2. **Alur unggah (F1-T29)**: presigned POST dengan policy lebih disukai, presigned PUT dengan `Content-Type` dan `Content-Length` bertanda tangan sebagai alternatif; validasi pasca-unggah tetap wajib `[KEPUTUSAN-234]`. Keduanya sudah dibuktikan di emulator, belum di NEO.
3. **Konfigurasi dan test**: endpoint, region, dan gaya pengalamatan dari config; endpoint `http://` ditolak di luar lokal; satu rangkaian test kontrak dijalankan terhadap emulator (CI) dan NEO nyata (manual atau terjadwal) `[KEPUTUSAN-235]`.
4. **Penghapusan berkas tak terkonfirmasi**: karena lifecycle NEO `[TIDAK DIKETAHUI]`, rancang worker yang menghapus slot kedaluwarsa (tidak bergantung pada lifecycle penyedia); lifecycle penyedia hanya pengaman tambahan bila terbukti didukung.
5. **Tidak ada perubahan ADR yang diusulkan saat ini.** ADR-018 butir 2 sudah memberi ruang ("sejauh kemampuan penyedia"). Perubahan ADR atau biaya hanya muncul bila celah (a) atau (c) di `[KEPUTUSAN-236]` terbukti; itu keputusan user.

## 5. Celah yang harus diuji dengan akun NEO nyata (butuh akses user)

Butuh: akun NEO Object Storage uji, access key dan secret key (tidak dikirim ke repositori), nama zona/endpoint yang dipilih. Uji dengan bucket sementara, lalu hapus.

1. Presigned PUT dengan SigV4: nilai region yang diterima, apakah `Content-Type` dan `Content-Length` bertanda tangan ditegakkan.
2. Presigned POST dengan policy (`content-length-range`, `Content-Type`).
3. Umur maksimum URL bertanda tangan; URL yang kedaluwarsa ditolak.
4. Gaya pengalamatan: virtual-host berjalan; path-style didukung atau tidak; bentuk URL presigned dalam HTTPS.
5. Versioning lewat API (`PutBucketVersioning`, `ListObjectVersions`), penghapusan semua versi dan delete marker per VersionId (syarat E-16), dan interaksinya dengan bucket Object Lock.
6. Lifecycle (`Expiration`, `AbortIncompleteMultipartUpload`, `NoncurrentVersionExpiration`) diterima atau tidak.
7. CORS untuk unggah dari browser (portal web), termasuk preflight PUT/POST.
8. Batas PUT tunggal dan multipart (relevan untuk export besar, bukan untuk batas 5/10 MB).
9. Akses anonim ke bucket private ditolak (hanya akar host yang diuji, tanpa bucket).
10. Biaya transfer keluar dan permintaan, enkripsi sisi penyedia, dan lokasi fisik endpoint (wjv-1 vs jkt-1) ditanyakan ke Biznet Gio.
11. Dokumentasi vendor mesin (Cloudian) dapat dibuka dari jaringan lain sebagai pembanding, tetapi bukan pengganti uji nyata karena konfigurasi Biznet bisa berbeda.
