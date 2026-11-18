# ADR-011: Target ASVS Level 2, autentikasi, sesi yang divalidasi server, dan MFA TOTP

Status: Diusulkan
Tanggal: 2026-11-18

## Konteks

- NFR-SEC-01: kontrol keamanan diverifikasi terhadap OWASP ASVS 5.0.0; level dan pemetaan ditetapkan di Tahap 4 `[DOK: 03-prd.md]` `[KEPUTUSAN-05]`. ASVS 5.0 menggambarkan Level 2 sebagai level yang "Most applications should be striving to achieve", Level 3 untuk aplikasi yang ingin menunjukkan tingkat keamanan tertinggi, dan pemilihan level bergantung pada sensitivitas aplikasi `[SRC-48]`.
- Login pasien hanya email dan password; login Google/Apple dihapus oleh CR-007 `[USER] T4-Q2` (BR-001).
- KEPUTUSAN-06 menetapkan: verifikasi email sebelum akun aktif, penguncian 15 menit setelah 5 kali gagal berturut-turut, tautan atur ulang password sekali pakai 60 menit, MFA wajib untuk admin Rumah Sehat, admin rekanan, dan pengguna RME minimalis, serta sesi web berakhir setelah 30 menit tanpa aktivitas. MFA tenaga kesehatan diputuskan di Tahap 4 `[KEPUTUSAN-06]`.
- Akun nonaktif harus ditolak termasuk pada sesi yang masih terbuka (KU-10). Untuk pengguna RME, akses berakhir seketika (US-1302 AC4). Keluar mengakhiri sesi di server (US-103 AC5) `[DOK: 03-prd.md]`.
- Tenaga kesehatan membuka keluhan awal dan riwayat chat banyak pasien (§3.1, E-05) `[DOK: 03-prd.md]`.
- Tidak ada SMS (BR-050), dan semua penyedia pihak ketiga wajib di Indonesia `[USER] T4-Q1`.

## Keputusan

1. **Target verifikasi: ASVS 5.0.0 Level 2** untuk semua komponen (aplikasi mobile, web, backend Rumah Sehat, RME minimalis, Partner Integration API). Pemetaan kontrol per bab ASVS disusun di Tahap 5 sebagai checklist, dan diverifikasi di Tahap 10 `[KEPUTUSAN-26]`.
2. **Penyimpanan password** memakai Argon2id dengan parameter minimal 19 MiB memori, 2 iterasi, dan paralelisme 1, sesuai OWASP Password Storage Cheat Sheet `[SRC-49]`. Kebijakan panjang dan isi password mengikuti ASVS Level 2 dan ditetapkan di Tahap 5 `[KEPUTUSAN-24]`.
3. **MFA dengan TOTP** (RFC 6238 `[SRC-43]`) lewat aplikasi authenticator di perangkat pengguna. MFA wajib untuk admin Rumah Sehat, admin rekanan, pengguna RME minimalis (KEPUTUSAN-06), **dan tenaga kesehatan** `[KEPUTUSAN-25]`:
   - diminta saat login di perangkat atau browser baru, yaitu saat membuat sesi baru;
   - disertai 10 kode pemulihan sekali pakai yang disimpan sebagai hash;
   - reset MFA hanya oleh admin yang berwenang setelah verifikasi identitas, dan tercatat (E-04);
   - rahasia TOTP disimpan terenkripsi (ADR-013);
   - SMS dan OTP email tidak dipakai sebagai faktor kedua.
   Pasien tidak diwajibkan MFA di rilis 1.
4. **Sesi divalidasi server di setiap permintaan** `[KEPUTUSAN-24]`:
   - Token sesi berupa nilai acak, bukan token yang memuat klaim. Server hanya menyimpan hash-nya di tabel sesi (kelas Operasional, ADR-008).
   - Setiap permintaan memeriksa sesi dan status akun. Penonaktifan akun, perubahan peran, dan keluar langsung mengakhiri semua sesi terkait (KU-10, US-1302 AC4, US-103 AC5).
   - **Mobile**: token akses berumur 15 menit dan token penyegar yang dirotasi setiap dipakai. Pemakaian ulang token penyegar lama mencabut seluruh rangkaian sesi itu. Batas tidak aktif dan batas mutlak (nilai awal): pasien 30 hari dan 90 hari; tenaga kesehatan 7 hari dan 30 hari. Token disimpan di penyimpanan aman platform perangkat (pustaka ditetapkan Tahap 5).
   - **Web** (backoffice, portal, RME): cookie `HttpOnly` dan `Secure` dengan atribut `SameSite`, perlindungan CSRF untuk permintaan yang mengubah data, batas tidak aktif 30 menit (NFR-SEC-04), dan batas mutlak 12 jam.
5. **Autentikasi ulang** untuk aksi berisiko: mengajukan hak subjek data dan mengunduh salinan data (US-1602, US-1604), mengubah email atau password, dan reset MFA `[KEPUTUSAN-24]`.
6. **Kontrol pendukung** (KEPUTUSAN-06): verifikasi email sebelum akun aktif; penguncian 15 menit setelah 5 kali gagal berturut-turut per akun; batas laju per akun dan per alamat IP untuk login, registrasi, dan atur ulang password; pesan galat umum yang tidak membocorkan apakah email terdaftar; tautan atur ulang sekali pakai 60 menit, disimpan sebagai hash.
7. **Identitas terpisah per sistem.** Akun Rumah Sehat (pasien, tenaga kesehatan, admin Rumah Sehat, admin rekanan) dan akun RME minimalis disimpan di database masing-masing. Klien sistem rekanan memakai kredensial integrasi, bukan akun pengguna (ADR-014).
8. **Pencatatan**: E-01–E-04 di audit trail (ADR-009), dan sinyal teknis di security log (ADR-010).

## Alternatif yang ditolak

- **ASVS Level 3 untuk seluruh sistem**: lebih sesuai dengan sensitivitas data kesehatan, tetapi menambah kontrol dan bukti verifikasi yang tidak realistis dalam jadwal rilis 1 dengan tim saat ini. Kontrol Level 3 tertentu yang murah dapat diambil di Tahap 5 tanpa mengubah target.
- **Token klaim bertanda tangan (misalnya JWT) tanpa pengecekan server**: penonaktifan dan keluar baru berlaku setelah token kedaluwarsa, sehingga KU-10 dan US-1302 AC4 tidak terpenuhi.
- **MFA lewat SMS**: SMS tidak dipakai di proyek ini (BR-050).
- **MFA lewat OTP email**: kotak email sering menjadi titik kompromi yang sama dengan password.
- **Tanpa MFA untuk tenaga kesehatan**: satu password yang bocor membuka data kesehatan banyak pasien.

## Konsekuensi

- Satu query sesi per permintaan. Biayanya kecil pada NFR-CAP-02 dan diukur di uji beban.
- Tenaga kesehatan memerlukan aplikasi authenticator, dan onboarding-nya (US-202) memuat langkah aktivasi MFA. Ini menambah sedikit hambatan pendaftaran.
- Kehilangan perangkat MFA memerlukan proses reset oleh admin, dan prosedurnya ditulis sebelum go-live.
- Nilai batas sesi adalah nilai awal teknis yang diselaraskan dengan checklist ASVS di Tahap 5.
