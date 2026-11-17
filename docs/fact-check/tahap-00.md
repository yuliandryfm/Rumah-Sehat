# Fact-check Tahap 00: Intake

Tanggal: 2026-11-17
Dokumen diperiksa: `docs/00-project-brief.md` (bukti sumber: `docs/00-catatan-intake.md`; register: `docs/decision-log.md`, `docs/sources.md`)
Tingkat ketelitian: T2 Standar
Hasil: LULUS (setelah cek ulang 2026-11-17; pemeriksaan awal TIDAK LULUS karena 1 temuan blokir, kini tertutup)

## Ringkasan (pemeriksaan awal)
- Total klaim: 96 (perkiraan, satu kalimat bisa memuat beberapa klaim)
- Terverifikasi: 92
- Tanpa sumber: 2 (non-blokir, rincian yang tidak tercatat di catatan intake)
- Sumber tidak mendukung: 1 (blokir)
- Kontradiksi: 1 (non-blokir, internal brief)
- Fakta bisnis berlabel asumsi: 0

Hal yang diperiksa dan bersih:
- Semua kode rujukan (P1.1 sampai P8.3, PS1 sampai PS6e, Konfirmasi P2/P5/P7/P8/PS, Perubahan scope 1) ada di catatan intake.
- Koreksi jawaban sudah diterapkan dengan benar: layanan 4 masuk rilis pertama, kandidat P3.6a nomor 2 dan 3 menjadi in-scope, daftar "belum diputuskan" tinggal RME, loyalitas/poin/voucher, antrean real-time, multi-bahasa. P1.5 digantikan P8.1/P8.1a.
- Tidak ada fakta dari usulan pewawancara yang tidak dipilih user.
- Tidak ada klaim eksternal yang disajikan sebagai fakta. `docs/sources.md` dan `docs/decision-log.md` kosong, sesuai tahap ini.
- Risiko dan tenggat pertanyaan terbuka ditandai sebagai catatan/usulan penyusun.

## Temuan (pemeriksaan awal)
| # | Lokasi | Klaim | Masalah | Usulan | Status |
| --- | --- | --- | --- | --- | --- |
| 1 (BLOKIR) | Brief, Data dan integrasi, butir "Data sensitif" | "Data sensitif: ... `[USER] P4.1`" | User tidak pernah menyatakan data mana yang sensitif; klasifikasi adalah penilaian penyusun berlabel `[USER]`, dan "sensitif" bermakna hukum tanpa `[SRC]`. | Jadikan usulan penyusun/`[TIDAK DIKETAHUI]`, rujuk PT-03. | Tertutup |
| 2 | Brief, R-02 vs PT-08 | "2 aplikasi mobile untuk 2 platform" | Bertentangan dengan PT-08 (platform nakes belum diketahui). | Rumuskan ulang. | Tertutup |
| 3 | Brief, Wilayah dan PT | (tidak ada) | Zona waktu acuan `[TIDAK DIKETAHUI]` (P6.5) tidak ada di tabel PT. | Tambah PT. | Tertutup |
| 4 | Brief, data pasien dan backoffice | Rincian identitas dasar; "kelola akun (pasien, tenaga kesehatan, rekanan)" | Rincian tidak tercatat di catatan intake. | Catat teks opsi lengkap. | Tertutup |
| 5 | Brief, PT (Pemilik) dan Stakeholder | Pemilik "keputusan oleh CEO/CTO"; "bukan bagian grup klinik/apotek" | Pemilik tidak diberi keterangan usulan; tafsiran "berdiri sendiri" tidak ada di Konfirmasi P5. | Nyatakan usulan penyusun. | Pemilik tertutup; tafsiran P5.2a tidak diubah (non-blokir, tetap tercatat) |

## Cek ulang 2026-11-17

Hanya baris yang berubah diperiksa.

1. Blokir (butir 1): `00-project-brief.md` baris 103 kini "Klasifikasi sensitivitas data: `[TIDAK DIKETAHUI]`", menyatakan user belum menyatakan data mana yang sensitif, kategori hukum diteliti Tahap 2 (PT-03), dan daftar ditandai "Usulan penyusun ... (bukan fakta)". Baris 102 (data yang disimpan, `[USER] P4.1`) tetap sesuai catatan. Blokir tertutup.
2. R-02 (baris 171): kini memisahkan aplikasi pasien (Android dan iOS) dan aplikasi nakes (platform belum diketahui, PT-08). Konsisten dengan PT-08. Tertutup.
3. PT-15 (baris 199): zona waktu acuan ditambahkan dengan pemilik dan tenggat. Tertutup.
4. Catatan intake: P4.1 (baris 165) dan P8.2a (baris 278) kini memuat teks opsi lengkap yang ditampilkan ke user, sehingga rincian identitas dasar dan "verifikasi & kelola akun" di brief punya dasar tercatat. Tertutup. Catatan kecil: jawaban user tetap hanya nomor/nama opsi, dan teks opsi tambahan diedit sesudah wawancara; bukan blokir.
5. Keterangan tabel PT (baris 181): Pemilik dan Tenggat dinyatakan usulan penyusun yang dikonfirmasi user di gate. Tertutup.

Sisa non-blokir (tidak menggagalkan): tafsiran "berdiri sendiri = bukan bagian grup klinik/apotek" (P5.2a) tidak muncul di Konfirmasi Putaran 5; sebaiknya dikonfirmasi di gate.

Hasil cek ulang: LULUS. Tidak ada temuan blokir tersisa.
