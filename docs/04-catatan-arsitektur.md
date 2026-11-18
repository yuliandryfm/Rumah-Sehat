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
