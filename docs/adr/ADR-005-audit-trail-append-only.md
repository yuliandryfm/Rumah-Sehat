# ADR-005: Audit trail append-only

Status: Diterima (mekanisme teknis didetailkan di Tahap 4)
Tanggal: 2026-11-10

## Konteks
Aplikasi membutuhkan jejak siapa melakukan apa untuk akuntabilitas, kepatuhan, dan investigasi fraud. Audit trail yang ditambahkan belakangan biasanya bolong.

## Keputusan
- Audit trail adalah fitur produk yang dibangun di Fase 1 sebagai fondasi.
- Disimpan append-only, terpisah dari tabel aplikasi. Admin pun tidak bisa mengubah atau menghapusnya lewat aplikasi.
- Field minimum per event: siapa (user ID, role), apa (aksi, objek), kapan (timestamp server), dari mana (IP, device), nilai before/after, hasil (sukses/gagal).
- Data yang dilarang masuk log: password, PIN, OTP, token, nomor kartu atau rekening lengkap.
- Dipisahkan dari application log (debugging) dan security log (deteksi ancaman).

## Alternatif yang ditolak
- Mencampur audit trail dengan application log: sulit dibaca auditor dan mudah tenggelam oleh log debug.
- Audit trail ditambahkan setelah fitur selesai: event lama tidak tercatat dan data before/after tidak bisa direkonstruksi.

## Konsekuensi
- Setiap story yang menyentuh aksi sensitif wajib mencatat event (bagian dari Definition of Done).
- Mekanisme (trigger, event di aplikasi, atau CDC), storage, perlindungan dari manipulasi, dan retensi ditetapkan di Tahap 4.
