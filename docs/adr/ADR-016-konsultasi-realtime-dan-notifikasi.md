# ADR-016: Chat lewat WebSocket, video lewat server media di Indonesia, notifikasi lewat email dan di dalam aplikasi

Status: Diterima (gate Tahap 4, 2026-11-18 `[USER] T4-G5`)
Tanggal: 2026-11-18

## Konteks

- Konsultasi chat (teks dan foto) dan video berlangsung di Ruang Sehat (BR-010, BR-013). Isi chat disimpan, video tidak direkam dan tidak disimpan (BR-016, AB-74) `[DOK: 02-brd.md]` `[DOK: 03-prd.md]`.
- Pesan chat sampai ke penerima yang online dalam p95 2 detik (NFR-PERF-03). Pesan yang dikirim saat offline terkirim otomatis setelah koneksi pulih (US-305 AC3). Ada 100 sesi bersamaan (NFR-CAP-03). Waktu bergabung dicatat server (§3.5) `[DOK: 03-prd.md]`.
- Fitur rekam penyedia video dinonaktifkan, dan cara memverifikasinya ditetapkan di Tahap 4 (US-306 AC4) `[DOK: 03-prd.md]`.
- Semua penyedia pihak ketiga memproses data di Indonesia `[USER] T4-Q1`. Push dan WhatsApp dihapus. Notifikasi pasien dan tenaga kesehatan lewat email dan notifikasi di dalam aplikasi. Panggilan video hanya dapat diterima saat kedua pihak berada di ruang sesi (CR-007) `[USER] T4-Q2`.
- Antrean real-time diperbarui dengan keterlambatan maksimal 30 detik dari data rekanan (US-405 AC1) `[DOK: 03-prd.md]`.
- Redis ditambahkan sebagai komponen pendukung, antara lain untuk sinyal real-time antar-instans (ADR-007) `[USER] T4-Q4`.

## Keputusan

**1. Kanal real-time aplikasi.** Aplikasi yang sedang terbuka menjaga satu koneksi WebSocket (RFC 6455 `[SRC-44]`) ke proses `realtime` (ADR-006). Koneksi diautentikasi dengan token sesi (ADR-011) dan dipakai untuk: pesan chat, kehadiran di ruang sesi, pembaruan status pesanan, pembaruan antrean, dan notifikasi di dalam aplikasi `[KEPUTUSAN-32]`. Pengaman koneksi (threat model API-06, M-03):
- saat koneksi dibuka, token, status akun, dan daftar pencabutan diperiksa seperti ADR-011 butir 8;
- setiap langganan kanal (sesi, pesanan, antrean, kotak notifikasi) diperiksa modul otorisasi ADR-012;
- koneksi diputus saat token akses kedaluwarsa (klien menyambung ulang dengan token baru), dan seketika saat sinyal penonaktifan, perubahan peran, atau pencabutan sesi diterima;
- karena Redis Pub/Sub bersifat at-most-once `[SRC-74]`, sinyal diputus dilengkapi cadangan `LISTEN/NOTIFY` dan pemeriksaan ulang status dari database setiap 60 detik;
- test otomatis: akun yang dinonaktifkan saat terhubung tidak menerima pesan berikutnya dan koneksinya terputus (KU-10).

**2. Alur pesan chat** `[KEPUTUSAN-32]`:
- Klien membuat ID pesan (UUIDv7) sebelum mengirim, sehingga kiriman ulang setelah koneksi pulih tidak menggandakan pesan (KU-05).
- Server memeriksa bahwa pengirim adalah peserta sesi dan sesi masih dalam waktunya, menyimpan pesan terenkripsi (ADR-013) sebagai baris transaksi, lalu menerbitkan sinyal berisi ID pesan lewat Redis Pub/Sub (ADR-007). Proses `realtime` yang memegang koneksi penerima membaca pesan dari database dan meneruskannya.
- Karena Redis Pub/Sub bersifat at-most-once `[SRC-74]`, klien yang tersambung ulang selalu meminta pesan sejak ID terakhir yang diterimanya, sehingga tidak ada pesan yang hilang dari tampilan.
- Lampiran foto diunggah lewat alur berkas (ADR-018) dan dirujuk dari pesan.
- Setelah durasi sesi habis, server menolak pesan baru, dan ruang chat menjadi baca-saja (US-305 AC4).
- Akses tenaga kesehatan ke riwayat chat setelah sesi dicatat E-05.

**3. Kehadiran sesi.** "Bergabung" dicatat server saat klien membuka ruang sesi lewat API, setelah waktu mulai tiba (§3.5). Waktu dari jam perangkat tidak dipakai. Batas ketidakhadiran PB-02 dan PB-03 serta akhir durasi dijalankan oleh job terjadwal yang dibuat saat sesi terkonfirmasi (ADR-007) `[KEPUTUSAN-32]`.

**4. Video** `[KEPUTUSAN-32]`:
- Panggilan memakai WebRTC melalui server media (SFU) dan server relay (TURN) yang **berlokasi di Indonesia**. Bentuknya dipilih di Tahap 5 dari dua opsi: penyedia terkelola yang memproses media di Indonesia, atau perangkat lunak server media yang dijalankan sendiri di cloud Indonesia.
- Backend membuat ruang per sesi dan menerbitkan token bergabung berumur pendek hanya untuk dua peserta sesi, hanya dalam jendela waktu sesi. Ruang ditutup saat durasi habis.
- **Tanpa rekaman**: fitur rekaman tidak diaktifkan di konfigurasi server media, dan aplikasi tidak menyediakan tombol rekam. Verifikasi: (a) konfigurasi server media atau ruang disimpan sebagai kode dan diperiksa di code review; (b) test otomatis memastikan ruang yang dibuat backend tidak punya pengaturan rekaman aktif, sejauh API penyedia atau server media menyediakannya; (c) penyimpanan rekaman pada server media yang dijalankan sendiri tidak dikonfigurasi. Rinciannya mengikuti produk terpilih di Tahap 5 (US-306 AC4).
- Panggilan hanya dapat dimulai dan diterima di dalam ruang sesi yang terbuka di kedua aplikasi (CR-007).

**5. Notifikasi** `[KEPUTUSAN-32]`:
- Satu modul notifikasi menerima kejadian bisnis dari outbox, menentukan kanal sesuai matriks EP-10 (email dan/atau di dalam aplikasi), membuat isi dari templat tanpa data kesehatan rinci (AB-73), lalu mengirim lewat adaptor kanal.
- **Di dalam aplikasi**: notifikasi disimpan di tabel kotak notifikasi (kelas Log untuk isinya, status dibaca di tabel terpisah). Notifikasi dikirim lewat WebSocket bila pengguna sedang terhubung, dan tampil di kotak notifikasi saat aplikasi dibuka.
- **Email**: lewat penyedia email atau relay SMTP yang memproses dan menyimpan data di Indonesia, dipilih di Tahap 5. Kegagalan dicoba ulang dan dicatat. Kegagalan satu kanal tidak menghentikan kanal lain (US-1001 AC3).
- Waktu dari kejadian sampai notifikasi diserahkan ke penyedia email atau tersedia di dalam aplikasi diukur terhadap NFR-PERF-05 (≤ 1 menit).

**6. Antrean real-time**: status antrean dari rekanan (ADR-014) disimpan, lalu diteruskan lewat WebSocket ke pasien yang sudah check-in dan sedang membuka appointment. Klien yang tidak terhubung mengambil status terbaru saat membuka layar `[KEPUTUSAN-32]`.

## Alternatif yang ditolak

- **Polling HTTP berkala untuk chat**: sulit memenuhi p95 2 detik tanpa beban permintaan besar.
- **Penyedia chat SaaS pihak ketiga**: isi chat adalah data kesehatan yang wajib disimpan di Indonesia dan terenkripsi di tingkat aplikasi. Penyedia SaaS menambah salinan data di luar kendali.
- **Penyedia video yang memproses media di luar Indonesia**: dilarang `[USER] T4-Q1`.
- **Push notification dan WhatsApp**: dihapus oleh CR-007 `[USER] T4-Q2`.

## Konsekuensi

- Gangguan Redis memperlambat penerusan pesan antar-instans (cadangan `LISTEN/NOTIFY`, ADR-007), tetapi tidak menghilangkan pesan.
- Proses `realtime` menyimpan koneksi panjang. Penyeimbang beban harus mendukung WebSocket, dan kapasitasnya diuji di NFR-CAP-03/CAP-04.
- Tanpa push, pengguna hanya menerima pemberitahuan di layar ponsel lewat email. Risiko lupa jadwal dan terlambat bergabung dicatat di risk register. Pengingat email (PB-08) dan tampilan jadwal di aplikasi menjadi mitigasinya.
- Kalau server media dijalankan sendiri, tim menanggung operasi dan pemantauannya, termasuk kapasitas untuk 100 sesi video bersamaan. Kalau memakai penyedia terkelola, lokasi pemrosesan media di Indonesia wajib dibuktikan dari dokumentasi resmi penyedia di Tahap 5.
