# ADR-002: UUIDv7 sebagai primary key

Status: Diterima
Tanggal: 2026-11-10

## Konteks
ID harus aman dipindahkan antar-environment dan antar-sistem saat migrasi data, tidak bisa ditebak, dan bisa dibuat sebelum data masuk database.

## Keputusan
Primary key memakai UUIDv7, disimpan dengan tipe UUID native, dan dibuat di sisi aplikasi.

## Alternatif yang ditolak
- Auto-increment integer: bentrok saat menggabungkan data antar-environment, mudah ditebak (risiko IDOR).
- UUIDv4: sepenuhnya acak, sehingga insert tersebar ke seluruh index dan performa turun saat data sudah besar.
- UUID disimpan sebagai VARCHAR(36): boros (36 byte dibanding 16) dan perbandingannya lebih lambat.

## Konsekuensi
- UUIDv7 diawali timestamp, sehingga index tetap rapi dan urutan ID mencerminkan urutan kejadian (berguna untuk investigasi).
- Library pembuat UUIDv7 dipilih di Tahap 5.
