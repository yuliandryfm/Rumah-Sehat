# Tahap 3: PRD + NFR

Tujuan: menerjemahkan BRD menjadi spesifikasi produk yang bisa dibangun dan diuji. Dari tahap ini lahir backlog awal, model data konseptual, dan spesifikasi audit trail.

Input: `docs/02-brd.md` dan dokumen sebelumnya.

## Isi PRD

1. **Fitur A–Z** dikelompokkan per modul, masing-masing merujuk ke nomor BR.
2. **User story**: "Sebagai [peran], saya ingin [aksi], agar [manfaat]".
3. **Acceptance criteria** per story yang bisa diuji, format Given–When–Then.
4. **NFR** dengan angka: performa (misalnya waktu respons p95), ketersediaan, keamanan, skalabilitas, kompatibilitas platform, retensi data.
5. **Model data konseptual**: daftar entitas bisnis dan relasinya, tanpa detail teknis.
6. **Spesifikasi audit trail**, ditulis dua kali:
    - Sebagai NFR: event yang dicatat (login/logout termasuk gagal, perubahan hak akses, create/update/delete data penting, transaksi, perubahan konfigurasi, export data), field per event (siapa, apa, kapan, dari mana, nilai before/after, hasil), retensi, hak akses, dan data yang wajib di-masking (password, PIN, OTP, token, nomor kartu/rekening lengkap).
    - Sebagai fitur: halaman Audit Log untuk admin dengan cari, filter, dan export.
7. **Prioritas MoSCoW** per fitur (Must, Should, Could, Won't).

Output: `docs/03-prd.md`.

## Backlog awal

Dari PRD, susun backlog awal di `docs/backlog.md`: Epic per modul, lalu User Story di bawahnya (pakai template di `templates.md`). Kalau user memakai GitHub Issues, tawarkan untuk membuatnya di sana dengan `gh`.

## Review QA

Panggil subagent `qa-engineer` untuk mereview setiap acceptance criteria: apakah bisa diuji, dan adakah kondisi yang belum didefinisikan (input kosong, gagal jaringan, hak akses ditolak, data duplikat). Perbaiki PRD berdasarkan temuannya.

## Tinjauan Tech Lead atas NFR

Sebelum PRD dikunci, tinjau setiap NFR dari sudut pandang Tech Lead: apakah angkanya realistis dicapai dengan skala tim dan anggaran di Business Case? NFR yang berisiko tidak tercapai ditandai, beserta alasan dan opsi penyesuaiannya, untuk diputuskan user di checkpoint. Jangan mengubah angka NFR yang berlabel `[USER]` tanpa persetujuan user.

## Gate Tahap 3

- [ ] Setiap fitur bisa ditelusuri ke nomor BR
- [ ] Setiap story punya acceptance criteria yang lolos review QA
- [ ] NFR terukur dengan angka dan sudah ditinjau kelayakannya dari sudut pandang Tech Lead
- [ ] Audit trail terspesifikasi sebagai NFR dan sebagai fitur
- [ ] Backlog awal tersusun di `docs/backlog.md`
