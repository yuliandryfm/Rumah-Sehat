# Review DBA Tahap 4: ringkasan

Tanggal: 2026-11-18
Reviewer: subagent `dba-reviewer` (independen). Berkas: `docs/04-data-dictionary.md`, `docs/04-erd.md`, dan ADR terkait. Tingkat T1.
Ringkasan ini disimpan orkestrator (`mode-otomatis.md`: laporan reviewer tanpa alat tulis disimpan ringkasannya). Temuan diverifikasi orkestrator terhadap berkas sebelum diteruskan (`anti-halu.md` §6).

## Putaran 1: TIDAK LULUS (8 blokir, 10 saran)

| ID | Masalah | Tindak lanjut |
| --- | --- | --- |
| DBA-B01 | Hak atas tabel kunci data: role aplikasi RME dapat mengubah DEK; pekerja privasi tidak dapat crypto-shred di RME; DEK yang dimusnahkan dapat dihidupkan; rotasi KEK tanpa role | Matriks `rme_data_key`, trigger satu arah, rotasi KEK lewat versi KMS atau role rotasi (KEPUTUSAN-66; ADR-013) |
| DBA-B02 | Hak `privacy_worker` belum mengikuti F-03/ADR-013 §5 (DELETE Log, INSERT outbox audit, kolom anonimisasi, RLS) | Matriks hak dan trigger penolak diperbarui |
| DBA-B03 | DB audit RME tanpa role pembaca per tenant; `rme_worker` terlalu luas | `audit_tenant_reader` dengan RLS `tenant_id`; hak `rme_worker` per tabel (ADR-012 butir 7–8) |
| DBA-B04 | Nilai hasil lab teks terbuka di outbox RME | Payload outbox hanya rujukan; isi disusun saat kirim (ADR-007) |
| DBA-B05 | Unique refund "kecuali RF-12" memblokir refund wajib setelah RF-13/RF-14 | Pengecualian diperluas ke RF-13/RF-14 (KEPUTUSAN-67; tafsiran AB-36 menunggu konfirmasi user di gate) |
| DBA-B06 | Satu penerimaan per percobaan bayar, sehingga debit ganda tidak tercatat | Kunci idempotensi penerimaan = ID transaksi penyedia; banyak penerimaan per percobaan (ADR-015) |
| DBA-B07 | Kolom di tabel transaksi yang baru terisi setelah baris dibuat (referensi penyedia, PDF resep) | Tabel transaksi anak; percobaan dicatat sebelum memanggil penyedia (ADR-015) |
| DBA-B08 | RME hanya satu rantai versi hasil lab per kunjungan | Tabel `rme_lab_result` + versi per hasil |
| DBA-S01–S10 | Composite FK tenant RME dan wadah DEK; dua wadah konsultasi; akun jurnal platform; salinan data yang dapat tidak sinkron; tabel acuan transisi status; index; ukuran kolom audit; pembersihan job; sub-kelas Master snapshot; inkonsistensi kolom `note` | Diterima (KEPUTUSAN-68, -69; ADR-008, ADR-011, ADR-013); S08 tanpa partisi tabel aplikasi di rilis 1 (KEPUTUSAN-63 tetap) |

Catatan verifikasi (bukan blokir): perilaku FK terhadap RLS, FK pada kolom terhitung, dan BRIN diverifikasi ke dokumentasi PostgreSQL di Tahap 5.
