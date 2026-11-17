# ADR-001: snake_case untuk penamaan database

Status: Diterima
Tanggal: 2026-11-10

## Konteks
Penamaan yang tidak seragam membuat query, migrasi, dan mapping ORM rawan salah, terutama ketika kode ditulis oleh banyak sesi atau orang.

## Keputusan
Semua nama di database memakai snake_case: tabel, kolom, index, constraint, dan nama file migrasi.

## Alternatif yang ditolak
- camelCase: perlu tanda kutip di banyak DB (misalnya PostgreSQL) dan rawan salah ketik huruf besar-kecil.
- Campuran sesuai selera: tidak konsisten.

## Konsekuensi
- Konsistensi ditegakkan lewat linter migrasi atau checklist code review.
- Mapping ke konvensi bahasa pemrograman (kalau berbeda) diatur di layer ORM/repository.
