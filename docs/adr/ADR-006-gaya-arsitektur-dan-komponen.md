# ADR-006: Modular monolith untuk backend Rumah Sehat, dengan RME minimalis sebagai sistem terpisah

Status: Diusulkan
Tanggal: 2026-11-18

## Konteks

- Rilis 1 berisi enam komponen: aplikasi pasien, aplikasi tenaga kesehatan, backoffice admin Rumah Sehat, portal rekanan, aplikasi RME minimalis, serta backend, integrasi, dan audit trail `[DOK: 03-prd.md]` §1.1.
- Tim: project manager, Claude lewat runbook, dan 1 mobile developer. Go-live 1 Maret 2027 `[DOK: 00-project-brief.md]`.
- RME minimalis terpisah dari aplikasi dan portal Rumah Sehat. Ia terhubung lewat kontrak integrasi yang sama dengan sistem rekanan lain dan harus lulus skenario uji yang sama tanpa jalur khusus (AB-61, US-1401 AC3) `[DOK: 03-prd.md]` `[USER] T3-Q14, T3-Q15`.
- Pengendali data rekam medis RME minimalis (fasyankes atau PT. Arunika) belum diketahui (L-05) `[DOK: 02-brd.md]` §6.3.
- Aplikasi tenaga kesehatan tersedia di Android dan iOS. Apakah dibuat terpisah atau digabung dengan aplikasi pasien diputuskan di Tahap 4 `[USER] T3-Q9`.
- Target kapasitas: 30.000 pasien aktif bulanan, 1.000 pengguna aktif bersamaan, dan 100 sesi konsultasi bersamaan (NFR-CAP-01–03) `[DOK: 03-prd.md]`.

## Keputusan

1. **Backend Rumah Sehat adalah modular monolith**: satu codebase dan satu database aplikasi, dengan modul berbatas jelas (daftar modul di `docs/04-arsitektur.md` §3). Modul berinteraksi hanya lewat antarmuka publik modul lain, tidak membaca tabel modul lain secara langsung. Satu image dijalankan sebagai tiga jenis proses yang diskalakan terpisah: `api` (HTTP untuk aplikasi, web, rekanan, dan webhook), `realtime` (WebSocket untuk chat, kehadiran sesi, dan antrean), dan `worker` (job terjadwal, outbox, notifikasi, integrasi keluar, rekonsiliasi, relay audit) `[KEPUTUSAN-18]`.
2. **RME minimalis adalah sistem terpisah**: backend, frontend web, dan database sendiri, dengan deployment terpisah. RME tidak berbagi database atau proses dengan Rumah Sehat, dan berbicara dengan Rumah Sehat hanya lewat Partner Integration API (ADR-014), dengan satu kredensial rekanan per fasyankes. Kode pustaka bersama (misalnya pustaka audit) boleh dipakai ulang lewat monorepo, tetapi tidak ada ketergantungan runtime `[KEPUTUSAN-18]`.
3. **Dua aplikasi Flutter terpisah**: Aplikasi Pasien dan Aplikasi Tenaga Kesehatan, masing-masing dengan listing toko sendiri. Komponen desain, klien API, dan utilitas dibagi sebagai paket di monorepo yang sama `[KEPUTUSAN-19]`, dikonfirmasi user `[USER] T4-Q3`.
4. **Komponen platform bersama**: Keycloak sebagai identity provider (ADR-011) dan Redis sebagai komponen pendukung (ADR-007), keduanya dijalankan di cloud Indonesia `[USER] T4-Q4`. Keduanya infrastruktur, bukan tempat logika bisnis. RME dan Rumah Sehat memakai realm Keycloak yang terpisah dan instans Redis yang terpisah (ADR-007). Relay audit berjalan sebagai deployment `worker` tersendiri yang hanya memegang kredensial audit (ADR-009).
5. **Dua web frontend terpisah**: Backoffice (admin Rumah Sehat) dan Portal Rekanan. Keduanya memakai backend Rumah Sehat dengan namespace API dan audience sesi yang terpisah, sehingga sesi portal tidak dapat memanggil API backoffice `[KEPUTUSAN-18]`.

## Alternatif yang ditolak

- **Microservices per domain**: menambah deployment, jaringan antar-layanan, dan transaksi terdistribusi. Event audit tidak lagi bisa ditulis atomik bersama aksi bisnis (KEPUTUSAN-15). Beban operasionalnya tidak sebanding dengan ukuran tim.
- **RME minimalis sebagai modul di backend Rumah Sehat**: bertentangan dengan pemisahan yang diputuskan user (T3-Q14, T3-Q15), membuka jalur khusus yang dilarang US-1401 AC3, dan mencampur data yang pengendalinya mungkin berbeda (L-05).
- **Satu aplikasi mobile untuk pasien dan tenaga kesehatan**: permukaan serangan lebih luas, karena kode dan izin tenaga kesehatan ikut terpasang di perangkat pasien. Aturan sesi dan MFA juga berbeda per peran (ADR-011), dan siklus rilis kedua kelompok pengguna menjadi terikat. Satu-satunya keuntungannya adalah satu listing toko.
- **Satu web untuk backoffice dan portal rekanan**: fungsi Super Admin berada di aplikasi yang sama dengan yang dipakai pihak luar, sehingga kesalahan otorisasi berdampak lebih besar.

## Konsekuensi

- Positif: satu pipeline deploy untuk backend, transaksi database lokal, pengujian end-to-end lebih sederhana, dan sesuai untuk tim kecil.
- Negatif: modul tidak bisa diskalakan sendiri-sendiri. Skala dilakukan per jenis proses (`api`, `realtime`, `worker`), dan kecukupannya dibuktikan lewat uji beban NFR-CAP-04.
- Batas modul harus dijaga. Aturan impor antar-modul ditegakkan lewat lint arsitektur atau checklist code review, ditetapkan di Tahap 5.
- Ada dua aplikasi yang wajib tayang di Play Store dan App Store (syarat go-live 4), dan dua web yang diuji kompatibilitas (NFR-COMP-03).
- RME minimalis menambah biaya operasional sendiri (database dan deployment terpisah). Biayanya dihitung di Tahap 5.
