# ADR-017: Waktu disimpan sebagai instan UTC, zona waktu sebagai nama IANA, dan kalender hari kerja di database

Status: Diusulkan
Tanggal: 2026-11-18

## Konteks

- Jadwal klinik, lab, dan home visit tampil dalam zona waktu lokasi rekanan berlabel (AB-40). Konsultasi tampil dalam zona waktu perangkat (AB-41). Laporan rekanan memakai zona waktu rekanan, sedangkan laporan dan backoffice Rumah Sehat memakai WIB (AB-42) `[USER] T3-Q10` `[DOK: 03-prd.md]`.
- Batas waktu berbasis jam atau hari kalender dihitung dari waktu absolut (AB-43) `[KEPUTUSAN-11]`. Batas hari kerja berakhir pukul 23.59 WIB pada hari kerja ke-N menurut kalender yang diatur Super Admin `[UP-37]`.
- Waktu audit disimpan dalam satu acuan baku yang ditetapkan di Tahap 4 (PRD §7.2) `[DOK: 03-prd.md]`.

## Keputusan

1. **Semua instan waktu** disimpan sebagai `timestamptz`. PostgreSQL menyimpannya sebagai UTC `[SRC-41]`. Server aplikasi berjalan dalam UTC. API mengirim waktu dalam format ISO 8601 dengan offset `[KEPUTUSAN-33]`.
2. **Zona waktu lokasi** disimpan sebagai nama zona IANA (misalnya `Asia/Jakarta`) pada data rekanan dan alamat layanan. Konversi tampilan dilakukan di klien atau di lapisan presentasi laporan sesuai AB-40–AB-42 `[KEPUTUSAN-33]`.
3. **Jadwal praktik online berulang** disimpan sebagai aturan (hari, jam lokal mulai dan selesai) beserta nama zona waktu tenaga kesehatan saat jadwal dibuat (AB-41). Slot yang terbentuk disimpan sebagai instan UTC. Jadwal rekanan berasal dari integrasi sebagai instan beserta zona waktu rekanan (ADR-014) `[KEPUTUSAN-33]`.
4. **Batas waktu PB** berbasis jam atau hari kalender dihitung sebagai instan absolut saat kejadian, lalu dijadwalkan sebagai job (ADR-007). Batas berbasis hari kerja (PB-05, PB-16, PB-20) dihitung dengan tabel kalender hari kerja (kelas Master), yang diatur Super Admin dan berakhir pukul 23.59 WIB pada hari kerja ke-N `[KEPUTUSAN-33]`.
5. **Sumber waktu**: hanya jam server, yang disinkronkan dengan layanan waktu penyedia cloud (diverifikasi Tahap 5). Jam perangkat tidak dipakai untuk aturan bisnis (misalnya waktu bergabung, §3.5) atau untuk audit (ADR-009) `[KEPUTUSAN-33]`.

## Alternatif yang ditolak

- **Menyimpan waktu lokal tanpa zona waktu**: jadwal lintas WIB, WITA, dan WIT menjadi ambigu, dan batas waktu absolut (AB-43) tidak dapat dihitung dengan benar.
- **Menyimpan offset tetap (misalnya +07:00) sebagai pengganti nama zona**: kehilangan identitas lokasi untuk label tampilan dan untuk perubahan aturan zona di masa depan.

## Konsekuensi

- Semua test yang menyentuh waktu memakai jam yang dapat diatur dan mencakup minimal dua zona waktu Indonesia.
- Kalender hari kerja wajib diisi sebelum go-live (UP-37, NFR-DR-02 jam kerja).
