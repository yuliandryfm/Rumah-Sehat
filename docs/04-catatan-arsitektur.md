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
