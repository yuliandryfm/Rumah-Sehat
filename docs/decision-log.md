# Decision Log

Mencatat setiap asumsi dan keputusan yang diambil Claude tanpa bertanya langsung. Semua entri berstatus "menunggu" dikonfirmasi user di checkpoint berikutnya.

Aturan: asumsi **hanya boleh** untuk detail teknis. Fakta bisnis (pengguna, aturan bisnis, angka target, regulasi, integrasi) tidak boleh diasumsikan.

## Asumsi

| ID | Tahap | Asumsi | Alasan | Dampak kalau salah | Status (menunggu / dikonfirmasi / dikoreksi) |
| --- | --- | --- | --- | --- | --- |
| | | | | | |

## Keputusan

| ID | Tahap | Keputusan | Opsi yang dipertimbangkan | Alasan dan sumber | Bisa dibatalkan? | Status |
| --- | --- | --- | --- | --- | --- | --- |
| KEPUTUSAN-01 | 1 | Tabel simulasi pendapatan memakai parameter bulat ilustratif (komisi 5/10/15/20%; biaya rekanan Rp250 ribu/Rp500 ribu/Rp1 juta per bulan; satuan Rp100 juta transaksi) dan dinyatakan per satuan nilai transaksi, tanpa volume transaksi | (a) tanpa simulasi; (b) simulasi dengan volume karangan; (c) simulasi berbasis rumus per satuan | User meminta simulasi (T1-Q2). Volume transaksi [TIDAK DIKETAHUI] (PT-10), jadi opsi (b) melanggar anti-halu. Parameter bukan acuan pasar dan bukan fakta bisnis; angka final oleh CEO (PT-17) | Ya, diganti angka CEO | dikonfirmasi (gate Tahap 1, 2026-11-17) |
| KEPUTUSAN-02 | 1 | Penilaian kemungkinan dan dampak risiko memakai skala kualitatif Tinggi/Sedang/Rendah oleh penyusun, berdasarkan fakta yang dirujuk di kolom Dasar; risiko yang faktanya belum diriset diberi kemungkinan [TIDAK DIKETAHUI] | (a) skala kualitatif 3 tingkat; (b) skor numerik 1–5 | Data historis untuk skor numerik tidak ada; skala 3 tingkat cukup untuk Business Case awal. Dikonfirmasi user di gate Tahap 1 | Ya | dikonfirmasi (gate Tahap 1, 2026-11-17) |
