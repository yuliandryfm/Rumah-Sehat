# Catatan Tahap 4 (Arsitektur)

Catatan apa adanya. File ini menjadi bukti untuk setiap label `[USER] T4-Qx` di dokumen Tahap 4.

Tanggal: 2026-11-18
Penyusun: Claude (persona Solution Architect + DBA + Security Architect)
Narasumber: user (project manager; persetujuan diteruskan ke CEO dan CTO, lihat brief "Stakeholder dan sign-off")
Cara bertanya: satu pertanyaan per giliran, sesuai arahan user di Tahap 0. Setiap pertanyaan ditulis lengkap di chat beserta konteks dan arti pilihan sebelum dialog pilihan ditampilkan.
Tingkat ketelitian: T1 Kritis (keputusan arsitektur, model data, audit trail, keamanan; `aturan-kerja.md` §1).

## Pertanyaan dan jawaban

**T4-Q1. Apakah keputusan "data wajib disimpan di Indonesia" juga berlaku untuk penyedia pihak ketiga (login Google/Apple, push notification, email, WhatsApp, panggilan video)? (PT-22)** (konteks: keputusan lama brief P5.5 dan BR-113; bagian inti (database, berkas, isi chat, rekam medis, audit trail, backup) tetap di cloud region Indonesia apa pun jawabannya; lokasi pemrosesan login dan push ditentukan Google/Apple dan sedang diverifikasi. Pilihan: (1) inti dan video di Indonesia, login/push/email/WhatsApp boleh di luar dengan data minimal, tunduk Pasal 56 UU PDP dan L-09 (rekomendasi penyusun); (2) semua di Indonesia tanpa kecuali; bila lokasi Google/Apple tidak bisa dipilih, login Google/Apple (BR-001) dan push notification (BR-050) tidak bisa dipakai dan harus diubah lewat change request; (3) hanya data tersimpan di Indonesia, semua penyedia termasuk video boleh di luar)
Jawaban: "Semua di Indonesia".
Catatan penyusun: PT-22 terjawab. Seluruh penyedia pihak ketiga yang memproses data pribadi wajib memproses dan menyimpan data di Indonesia. Konsekuensi terhadap BR-001 (login Google/Apple) dan BR-050 (push, WhatsApp) dinilai setelah verifikasi sumber resmi dan diajukan sebagai change request.

**T4-Q2. Apakah change request CR-007 (akibat jawaban T4-Q1 "semua di Indonesia") disetujui?** (konteks yang disampaikan, dari sumber resmi yang dibuka 2026-11-18 (`docs/sources.md` SRC-51–SRC-55 dan bagian "Pencarian tanpa hasil"): notifikasi ke perangkat iOS dikirim lewat APNs; Firebase berjalan di infrastruktur global Google dan pilihan lokasi hanya untuk sebagian layanan, FCM tidak disebut; tanpa FCM, mode Doze Android membatasi akses jaringan aplikasi; tidak ditemukan pilihan lokasi untuk login Google/Apple; WhatsApp Cloud API memproses pesan di data center Meta dan opsi Local Storage Indonesia hanya untuk data tersimpan, sedangkan isi pesan dapat diproses di luar negeri sampai 60 menit; jalur On-Premises berakhir 23 Oktober 2025; email dan video dapat dipenuhi di Indonesia. Isi CR-007: login Google/Apple (BR-001, US-102) dihapus; push dan WhatsApp (BR-050, EP-10) dihapus; notifikasi lewat email dan notifikasi di dalam aplikasi; pengingat jadwal lewat email; kejadian yang tadinya hanya push hanya terlihat saat aplikasi dibuka; panggilan video hanya dapat diterima saat kedua pihak di ruang sesi; dampak: risiko lupa jadwal atau terlambat bergabung naik. Pilihan: (1) setujui CR-007 apa adanya; (2) setujui dengan pengecualian push tanpa isi (hanya token perangkat keluar Indonesia, mengubah T4-Q1, perlu L-09); (3) ubah jawaban T4-Q1 ke rekomendasi awal sehingga CR-007 tidak diperlukan)
Jawaban: "Setujui CR-007 apa adanya".
Catatan penyusun: CR-007 diterapkan ke BRD (v1.3), PRD (v1.1), `docs/backlog.md`, `docs/traceability.md`, dan catatan KEPUTUSAN-06 di `docs/decision-log.md`. Teks revisi diperiksa fact-checker sebelum gate Tahap 4.

**Pertanyaan user (di luar daftar):** "tech stack apa yg kamu gunakan? akan ada berapa aplikasi disini?"
Jawaban penyusun: stack yang sudah pasti hanya Flutter (brief P7.4); PostgreSQL diusulkan (ADR-007, menunggu gate); bahasa/framework backend, framework web, cloud, dan penyedia pihak ketiga dipilih di Tahap 5. Lima aplikasi pengguna (aplikasi pasien, aplikasi tenaga kesehatan, backoffice, portal rekanan, RME minimalis) ditambah dua backend (Rumah Sehat, RME) dan tiga instans database.

**Pertanyaan user:** "aplikasi tenaga medis apa tidak menggunakan rme saja?"
Jawaban penyusun: aplikasi tenaga kesehatan melayani konsultasi online dan home visit untuk semua tenaga kesehatan Rumah Sehat (BR-013, mobile T3-Q9), sedangkan RME minimalis hanya untuk klinik/lab tanpa sistem (BR-120) dan rekam medisnya terpisah dari Rumah Sehat (T3-Q14). Memindahkan konsultasi ke RME membalik BR-013, T3-Q9, dan T3-Q14.

**Pertanyaan user (sebagai jawaban T4-Q3 pertama):** "bagaiaman pengiriman data dari rumah sehat ke RME nya?"
Jawaban penyusun: lewat Partner Integration API seperti rekanan lain (ADR-014, tabel EP-14): RME mengirim layanan/jadwal/harga, check-in, selesai, antrean, penugasan home visit, dan hasil lab; Rumah Sehat mengirim cek ketersediaan, pesanan terbayar, pembatalan, daftar tenaga kesehatan aktif, dan status home visit. Rekam medis RME tidak dikirim ke Rumah Sehat; data konsultasi Rumah Sehat tidak dikirim ke RME; NIK pasien menunggu L-05/L-08; setiap pengiriman data pribadi dicatat E-17.

**T4-Q3. Bagaimana aplikasi tenaga kesehatan dibangun?** (konteks: KEPUTUSAN-19 (aplikasi terpisah, belum lulus gate); diserahkan ke Tahap 4 di T3-Q9. Pilihan: (1) tetap terpisah, dua aplikasi mobile, RME tetap web untuk klinik/lab tanpa sistem (rekomendasi penyusun); (2) gabung ke aplikasi pasien dengan mode peran; (3) pindahkan konsultasi dan home visit ke RME web, perlu change request atas BR-013, T3-Q9, T3-Q14)
Jawaban: "Tetap terpisah (Recommended)".
Catatan penyusun: KEPUTUSAN-19 dikonfirmasi user; ADR-006 butir 3 diberi label `[USER] T4-Q3`.

**Masukan user:** "jika butuh message broker atau redis bahkan keycloack sepertinya akan lebih baik"
Tanggapan penyusun: Keycloak layak dan sebaiknya diputuskan sekarang karena mengubah ADR-011 dan model data akun; Redis layak sebagai komponen pendukung (batas laju, cache sesi, penyebaran WebSocket), bukan sumber data; message broker belum perlu karena outbox PostgreSQL tetap dibutuhkan untuk atomisitas (KEPUTUSAN-15) dan broker menambah komponen bertahan data yang harus dipulihkan sesuai RPO.

**T4-Q4. Komponen mana yang ditambahkan ke arsitektur?** (konteks: ADR-007 tanpa Redis/broker dan ADR-011 autentikasi dibangun sendiri, keduanya belum lulus gate. Pilihan: (1) Keycloak + Redis, tanpa broker; ADR-011 direvisi setelah kemampuan Keycloak diverifikasi ke dokumentasi resmi (rekomendasi penyusun); (2) Keycloak + Redis + message broker; (3) evaluasi semuanya di Tahap 5; (4) tetap desain sekarang)
Jawaban: "Keycloak + Redis (Recommended)".
Catatan penyusun: ADR-011 direvisi memakai Keycloak sebagai identity provider yang dijalankan sendiri di cloud Indonesia; ADR-007 direvisi menambah Redis sebagai komponen pendukung; antrean dan outbox tetap PostgreSQL, tanpa message broker. Kemampuan Keycloak dan Redis diverifikasi ke dokumentasi resmi sebelum ADR ditulis.

## Review sebelum gate

- Fact-check bagian 1 (`docs/fact-check/tahap-04.md`): TIDAK LULUS, 2 blokir.
  - FC4-B01: angka retensi application log, security log, dan backup ditetapkan penyusun, padahal keputusan PT-24. Diperbaiki: ditandai `[TIDAK DIKETAHUI]` dengan usulan nilai sementara untuk diputuskan user di gate; tidak ada penghapusan otomatis sebelum diputuskan.
  - FC4-B02: dokumen belum mengikuti T4-Q4. Diperbaiki: ADR-011 ditulis ulang untuk Keycloak; ADR-006, ADR-007, ADR-009, ADR-010, ADR-013, ADR-014, ADR-016, ADR-019, KEPUTUSAN-20/23/24/25/28/35, risk register (RT-11, RT-17, RT-18), dan arsitektur diperbarui; SRC-64–SRC-77 ditambahkan.
  - Non-blokir yang dirapikan: N01 (RF-13 di arsitektur §4.1), N04 (ADR-018 vs US-1604 AC2; pesan galat di ADR-011 butir 4), dan N05 (PRD §0.1, US-1406, BRD §4.2). N02 dan N03 disampaikan di gate. Business Case R-06 dan baris biaya WhatsApp/push dibiarkan untuk ditinjau saat baseline biaya Tahap 5.
- Review DBA (`docs/review/tahap-04-dba.md`): putaran 1 TIDAK LULUS (8 blokir), putaran 2 menutup B01–B08 dengan 2 blokir baru kecil (B09, B10), diperbaiki di putaran perbaikan kedua (batas T1).
- Threat model (`docs/04-threat-model.md`): TIDAK LULUS, 9 ancaman Tinggi belum tercermin. Mitigasi teknis M-01 (a, b, d), M-02, M-03, M-05–M-09 diterapkan (ADR-021 baru; revisi ADR-006, ADR-007, ADR-009, ADR-010, ADR-011, ADR-014, ADR-015, ADR-016, ADR-018; DoD backlog), ditambah M-13, M-17, M-19, M-23, H-06, H-10. M-04 dan M-01 (c) dibawa ke user.

**T4-Q5. Bagaimana apotek memastikan resep digital Rumah Sehat asli dan tidak diubah? (threat model API-13)** (konteks: US-309 AC5 kanal apotek `[TIDAK DIKETAHUI]`; PDF dapat dipalsukan. Pilihan awal: kode verifikasi + QR (rekomendasi); terima risiko rilis 1; tunggu L-05)
Jawaban pertama (teks bebas): "apakah yg bisa membuktikan resep digital itu genuine by rumah sehat? karena apotek ini bukan rekanan rumah sehat, agar tidak ada pemalsuan resep".
Catatan penyusun: dijelaskan dua cara pembuktian untuk apotek non-rekanan: (1) verifikasi online ke server Rumah Sehat lewat QR/kode di domain resmi (bukti adalah data server, bukan tampilan PDF; risiko QR ke situs tiruan, sehingga alamat resmi juga dicetak sebagai teks); (2) tanda tangan elektronik pada PDF (hanya untuk berkas asli, menunggu L-05). Ditambahkan opsi penandaan "sudah ditebus" untuk mencegah penebusan berulang. Pertanyaan diulang dengan pilihan: (1) QR/kode ke domain resmi, tanda tangan elektronik setelah L-05 (rekomendasi); (2) QR/kode + tandai sudah ditebus; (3) terima risiko rilis 1; (4) tunggu L-05.
Jawaban: "QR/kode ke domain resmi (Recommended)".
Catatan penyusun: CR-008 diterapkan ke PRD v1.1 (US-309 AC5–AC8), `docs/backlog.md`, data dictionary A9.8, ADR-012 butir 3, dan arsitektur §3. Penandaan "sudah ditebus" tidak termasuk.

**T4-Q6. Apakah aplikasi tenaga kesehatan dikunci (biometrik/PIN perangkat) saat dibuka kembali setelah tidak aktif? (threat model APP-02)** (konteks: sesi tenaga kesehatan bertahan sampai 7 hari tanpa aktivitas; isi klinis tidak disimpan di perangkat (ADR-021), tetapi aplikasi yang terbuka di ponsel hilang memperlihatkan keluhan dan riwayat chat sampai sesi dicabut. Pilihan: (1) kunci setelah 5 menit (rekomendasi penyusun); (2) kunci setelah 15 menit; (3) tanpa kunci aplikasi, risiko Tinggi diterima user)
Jawaban: "Kunci setelah 15 menit".
Catatan penyusun: diterapkan sebagai CR-009 (PRD NFR-SEC-10), ADR-021 butir 1, dan `docs/backlog.md`.
- Review DBA putaran 3: LULUS DENGAN CATATAN (B09, B10 tertutup); catatan CR-008 diterapkan (versi kunci kode verifikasi; resep yang wadahnya dimusnahkan tampil "tidak ditemukan").
- Threat model v0.2: LULUS (54 ancaman Tinggi tercermin). Ancaman baru halaman verifikasi resep: RX-01 (Sedang, mitigasi M-31 diterapkan di ADR-012 butir 3), RX-02 dan RX-03 (Tinggi, dijelaskan dan dipilih user di T4-Q5; dicatat RT-19, RT-20). Aturan metode tambahan di KEPUTUSAN-70.
- Fact-check bagian 2: LULUS, FC4-B01 dan FC4-B02 tertutup. Non-blokir yang dirapikan: KEPUTUSAN-32 dan diagram arsitektur §4.2/§6 (Redis Pub/Sub, dua instans Redis, Keycloak admin internal); pencabutan sesi disimpan di tabel Operasional (A1.4, C1.11, ADR-011 butir 8); rujukan basi (35 hari, header, ADR-021, nomor butir ADR-012); ringkasan arsitektur §1/§14 dan KEPUTUSAN-27/-31; kutipan batasan partisi di SRC-39. Butir gate dari fact-checker (data di halaman verifikasi resep, tafsiran ADR-002/003, pengecualian KEPUTUSAN-15 untuk event login) ditampilkan di gate.
