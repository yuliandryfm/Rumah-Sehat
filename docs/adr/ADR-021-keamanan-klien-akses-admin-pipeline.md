# ADR-021: Keamanan aplikasi mobile, akses administratif production, dan pipeline CI/CD

Status: Diterima (gate Tahap 4, 2026-11-18 `[USER] T4-G5`)
Perubahan: sebagian diubah oleh ADR-025 (CR-012, [USER] T5-Q2), menunggu gate Tahap 5.
Tanggal: 2026-11-18

## Konteks

Threat model Tahap 4 (`docs/04-threat-model.md`) menemukan empat ancaman Tinggi yang belum tercermin di desain dan bersumber di luar backend `[KEPUTUSAN-70]`:
- **APP-02**: perangkat tenaga kesehatan hilang dengan sesi aktif dan data klinis tersimpan di perangkat.
- **APP-05**: SDK aplikasi mobile mengirim data ke luar Indonesia tanpa cara verifikasi, padahal semua penyedia wajib di Indonesia `[USER] T4-Q1`.
- **KMS-01**: akses manusia ke dekripsi KMS, secret production, dan superuser database tidak dibatasi dan tidak dipantau.
- **CI-01**: pipeline memegang kredensial pemilik skema tanpa kontrol keamanan pipeline.

Rujukan tambahan:
- OWASP CI/CD Security Cheat Sheet: "Secrets should never be hardcoded in code repositories or CI/CD configuration files"; "Any identities used within the pipeline must be assigned only the minimum permissions necessary to do its job"; "Require pull requests to be reviewed before merging and ensure this review step cannot be bypassed" `[SRC-82]`.
- OWASP Logging Cheat Sheet mengecualikan antara lain nilai identitas sesi, token akses, dan data pribadi sensitif dari log `[SRC-83]`.

## Keputusan

**1. Aplikasi mobile: data di perangkat** (APP-02, M-01 a, b, d) `[KEPUTUSAN-71]`:
- Isi klinis (chat, keluhan awal, catatan, resep, hasil lab, lampiran) tidak disimpan di penyimpanan persisten aplikasi. Data diambil dari server saat dibutuhkan.
- Pengecualiannya berkas yang sengaja diunduh pasien (US-309 AC4, US-902 AC1) dan kode/QR appointment yang wajib tampil saat offline (US-403 AC2).
- Antrean pesan keluar yang belum terkirim (US-305 AC3) disimpan terenkripsi dengan kunci dari penyimpanan aman platform, lalu dihapus setelah terkirim atau saat keluar.
- Data aplikasi dikecualikan dari cadangan perangkat, dan layar disembunyikan saat aplikasi berada di latar belakang, sejauh platform mendukungnya (diverifikasi Tahap 5).
- **Prosedur perangkat hilang**: pengguna atau admin mencabut semua sesi akun lewat mekanisme ADR-011 butir 8. Prosedurnya ditulis sebelum go-live.
- **Kunci aplikasi tenaga kesehatan** (M-01 c): aplikasi meminta verifikasi pemilik perangkat (biometrik atau PIN perangkat) saat dibuka kembali setelah 15 menit tidak aktif `[USER] T4-Q6` (CR-009, NFR-SEC-10). Pustaka verifikasi perangkat ditetapkan di Tahap 5.

**2. Aplikasi mobile: kendali SDK dan lalu lintas jaringan** (APP-05, M-02) `[KEPUTUSAN-71]`:
- Inventaris dependensi dan plugin kedua aplikasi disusun per rilis.
- Setiap dependensi mobile baru direview perilaku jaringannya dan data yang dikumpulkannya sebelum dipakai.
- Daftar izin host tujuan jaringan: backend Rumah Sehat, Keycloak, server media video, dan layanan crash di Indonesia.
- Uji lalu lintas jaringan build rilis membuktikan tidak ada koneksi ke host di luar daftar izin. Uji ini menjadi syarat rilis (Tahap 12) dan butir Definition of Done. Metode ujinya ditetapkan di Tahap 5.

**3. Akses administratif manusia ke production** (KMS-01, M-08) `[KEPUTUSAN-71]`:
- MFA wajib untuk semua identitas manusia di akun cloud production.
- Hak dekripsi KEK dan baca secret production hanya dimiliki identitas proses (`api`, `realtime`, `worker`, relay audit, RME, pekerja privasi) sesuai kebutuhan masing-masing. Tidak ada identitas manusia dengan hak tetap.
- **Akses darurat (break-glass)**: memerlukan persetujuan orang kedua, berjangka waktu, dan memicu peringatan otomatis. Superuser database production hanya tersedia lewat jalur ini. Operasi rutin memakai role berhak minimum.
- Log audit penyedia cloud untuk KMS, secret manager, IAM, dan akses database administratif dikirim ke security log, dengan aturan peringatan (ADR-010).
- Tinjauan hak akses dilakukan berkala dan dicatat.
- Kemampuan IAM, KMS, dan log audit penyedia diverifikasi saat pemilihan penyedia di Tahap 5.

**4. Pipeline CI/CD** (CI-01, M-09) `[KEPUTUSAN-71]`:
- Setiap lingkungan punya identitas pipeline sendiri dengan hak minimum dan kredensial cloud jangka pendek. Rahasia diambil dari secret manager, tidak ditulis di repositori atau berkas konfigurasi CI `[SRC-82]`.
- Deploy production hanya dari branch `main` yang dilindungi, dengan review yang tidak dapat dilewati `[SRC-82]` dan persetujuan manual. Ini melengkapi git hook commit hijau proyek.
- Kredensial pemilik skema (migrasi aplikasi, audit, dan RME) hanya tersedia pada langkah migrasi.
- Migrasi yang menyentuh database audit, role, grant, trigger penolak, atau kebijakan RLS wajib direview DBA dan keamanan.
- Versi dependensi dipin, dan integritasnya diperiksa dengan hash `[SRC-82]`.
- Pemindaian rahasia, dependensi, dan image di CI melengkapi butir "security scan" di Definition of Done.
- Image dirujuk lewat digest dan ditandatangani bila platform mendukung (verifikasi Tahap 5).
- Kunci penandatangan aplikasi mobile disimpan di secret manager atau layanan penandatanganan dengan akses minimal yang tercatat.

## Alternatif yang ditolak

- **Mengandalkan kunci layar perangkat saja**: tidak melindungi data aplikasi yang tersimpan tanpa enkripsi, dan tidak berlaku bila perangkat sudah terbuka.
- **Larangan SDK tanpa verifikasi**: tidak dapat dibuktikan di audit keamanan (syarat go-live 6).
- **Akses admin tetap untuk tim teknis**: enkripsi tingkat aplikasi (ADR-013) tidak lagi melindungi dari orang dalam.
- **Pipeline dengan kredensial statis berhak penuh**: satu kebocoran kredensial CI membuka seluruh production, termasuk database audit.

## Konsekuensi

- Aplikasi mobile memerlukan koneksi untuk membuka isi klinis, kecuali berkas yang sengaja diunduh dan kode appointment.
- Uji lalu lintas jaringan dan review dependensi menambah langkah rilis. Butir Definition of Done ditambahkan di `docs/backlog.md`.
- Operasi darurat lebih lambat karena perlu persetujuan orang kedua. Untuk tim kecil, orang kedua dapat dipegang CTO atau user (PM), dan prosedurnya ditetapkan sebelum go-live.
- Proteksi branch dan persetujuan deploy diatur di pengaturan repositori dan platform CI. Pengaturan itu dibuat user atau tim di Tahap 5 atau 12, karena Claude tidak mengubah pengaturan repositori tanpa persetujuan.
