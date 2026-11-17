# ADR-004: Data transaksi immutable

Status: Diterima
Tanggal: 2026-11-10

## Konteks
Data transaksi, mutasi, dan jurnal adalah bukti finansial. Kemampuan mengubah atau menghapusnya membuka celah fraud.

## Keputusan
Data transaksi tidak pernah di-update atau di-delete, termasuk soft delete. Koreksi dilakukan dengan reversal entry (transaksi pembalik) yang merujuk transaksi asal.

## Alternatif yang ditolak
- Soft delete pada transaksi: flag bisa dibalik, dan saldo bisa tidak konsisten.
- Update langsung dengan audit log: nilai asli hanya tersimpan di log, bukan di data utama.

## Konsekuensi
- Hak `UPDATE` dan `DELETE` pada tabel transaksi dicabut dari user DB aplikasi.
- Saldo dihitung dari rangkaian transaksi atau disimpan sebagai snapshot yang bisa direkonsiliasi.
- Laporan harus memperhitungkan reversal entry.
