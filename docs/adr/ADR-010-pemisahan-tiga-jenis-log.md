# ADR-010: Pemisahan audit trail, application log, dan security log

Status: Diusulkan
Tanggal: 2026-11-18

## Konteks

- ADR-005 dan NFR-OBS-02 memisahkan audit trail dari application log dan security log `[DOK: adr/ADR-005-audit-trail-append-only.md]` `[DOK: 03-prd.md]`.
- Application log dan security log juga tidak boleh memuat data terlarang PRD §7.3 (NFR-OBS-03) `[DOK: 03-prd.md]` `[KEPUTUSAN-07]`.
- Ketersediaan harus dipantau otomatis dan gangguan memicu peringatan (NFR-OBS-01) `[KEPUTUSAN-17]`.
- Semua penyedia pihak ketiga, termasuk layanan log dan pemantauan, wajib memproses dan menyimpan data di Indonesia `[USER] T4-Q1`.

## Keputusan

Tiga jenis log dikelola terpisah, dengan tujuan, isi, penyimpanan, akses, dan retensi yang berbeda `[KEPUTUSAN-23]`:

| Aspek | Audit trail | Application log | Security log |
| --- | --- | --- | --- |
| Tujuan | Akuntabilitas dan kepatuhan: siapa melakukan apa terhadap data siapa (fitur produk) | Debugging dan operasi | Deteksi dan investigasi ancaman |
| Isi | Event E-01–E-17 dengan field PRD §7.2 (ADR-009) | Log terstruktur JSON: waktu, level, komponen, ID korelasi, rute (pola, bukan URL berisi ID sensitif), status, durasi, galat dan stack trace. Hanya ID internal, tanpa data pribadi | Login berhasil/gagal dan penguncian akun, kegagalan MFA, akses ditolak (termasuk percobaan ganti ID), pelampauan batas laju, tanda tangan webhook atau kredensial rekanan tidak valid, hasil verifikasi hash berantai audit, kegagalan atau keterlambatan relay audit, percobaan ubah/hapus audit, ketidakcocokan proyeksi status (ADR-008), perubahan hak akses |
| Penghasil | Pustaka audit (ADR-009) | Semua proses (`api`, `realtime`, `worker`, RME, Keycloak) | Event Keycloak (ADR-011), middleware otorisasi, gateway integrasi, job verifikasi, log database |
| Penyimpanan | Database audit terpisah dan arsip WORM | Layanan log terkelola di region Indonesia atau yang dijalankan sendiri di cloud Indonesia (Tahap 5) | Aliran atau indeks terpisah dari application log, dengan aturan peringatan |
| Akses | Auditor, Super Admin (aplikasi); Admin Fasyankes untuk log akses rekam medis fasyankesnya | Tim teknis | Hanya pemegang peran keamanan di tim teknis; tidak lewat aplikasi |
| Retensi | `[TIDAK DIKETAHUI]` (PT-24); tidak dimusnahkan sebelum ditetapkan | `[TIDAK DIKETAHUI]` (PT-24). Usulan nilai sementara 30 hari, diputuskan user di gate Tahap 4 | `[TIDAK DIKETAHUI]` (PT-24). Usulan nilai sementara 1 tahun, diputuskan user di gate Tahap 4 |

Ketentuan tambahan:

1. **Redaksi otomatis.** Logger memakai daftar tolak (password, token, header `Authorization`, cookie, NIK, nomor rekening, isi body permintaan) dan menolak menulis objek domain secara utuh. Test otomatis memeriksa contoh log terhadap pola data terlarang `[KEPUTUSAN-23]`.
2. **Satu kejadian, beberapa log.** Login gagal tercatat di audit trail (E-02) untuk akuntabilitas dan di security log untuk deteksi. Isi masing-masing mengikuti aturan log-nya, dan keduanya dihubungkan dengan ID korelasi yang sama.
3. **Pemantauan dan peringatan** (NFR-OBS-01): pemeriksaan ketersediaan berkala dari luar terhadap endpoint kesehatan layanan inti, metrik (laju galat, latensi p95 dibanding NFR-PERF, panjang antrean job, umur outbox, status integrasi per rekanan), dan peringatan ke tim teknis. Pemeriksa ketersediaan hanya memanggil endpoint kesehatan yang tidak memuat data pribadi.
4. **Laporan crash aplikasi mobile** dikirim ke layanan di Indonesia atau yang dijalankan sendiri di cloud Indonesia, tanpa data pribadi. SDK analitik atau crash pihak ketiga yang mengirim data ke luar Indonesia tidak boleh dipasang `[USER] T4-Q1`. Larangan ini diverifikasi lewat inventaris dependensi dan uji lalu lintas jaringan build rilis (ADR-021 butir 2).
5. **Security log juga menerima** log audit penyedia cloud untuk KMS, secret manager, IAM, dan akses database administratif (ADR-021 butir 3). Daftar tolak logger mencakup nilai sesi, token akses, dan parameter tanda tangan URL bertanda tangan `[SRC-83]`.
6. Retensi application log dan security log adalah keputusan bisnis, karena keduanya memuat data pribadi (alamat IP dan ID akun) (BR-112, PT-24). Nilai sementaranya dibawa ke user di gate Tahap 4. Sampai ada keputusan, kedua log tidak dihapus otomatis.

## Alternatif yang ditolak

- **Satu aliran log untuk semua**: audit tenggelam di log debug, hak akses tidak bisa dibedakan, dan retensi audit yang panjang memaksa log debug ikut disimpan lama.
- **Security log hanya dari audit trail**: audit trail tidak memuat sinyal teknis seperti batas laju dan tanda tangan webhook gagal, dan aksesnya untuk auditor, bukan untuk tim yang menangani insiden.
- **Layanan observabilitas SaaS di luar Indonesia**: dilarang `[USER] T4-Q1`.

## Konsekuensi

- Tiga konfigurasi penyimpanan dan retensi yang harus dijaga, ditambah satu sistem pemantauan.
- Pilihan alat log, metrik, dan crash di Tahap 5 dibatasi pada penyedia yang memproses data di Indonesia atau yang dijalankan sendiri.
- Ketersediaan bulanan (NFR-AVL-01) diukur dari pemeriksa ketersediaan. Gangguan pihak ketiga dan rekanan dicatat terpisah (NFR-AVL-04).
