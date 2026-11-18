# Fact-check Tahap 06: Design Guideline

Tanggal: 2026-11-18
Dokumen diperiksa: `docs/06-design-guideline.md`, `docs/06-catatan-design.md`, `docs/wireframes/` (sampling teks klaim bisnis: a05, a06, a07, a08, a12, index; cek CDN/skrip pada semua berkas), register `docs/sources.md` SRC-256–260, `docs/decision-log.md` KEPUTUSAN-160–176, UP-58–64, PT-40–43, `docs/traceability.md` "Story → wireframe (Tahap 6)"
Tingkat: T2 (aksesibilitas dan layar T1 diperiksa lebih teliti)
Hasil: LULUS

## Ringkasan
- Total klaim substantif diperiksa: sekitar 95 (klaim bernomor SRC/KEPUTUSAN/UP/PT, klaim PRD/ADR/CR, sampel wireframe)
- Terverifikasi: semua yang diperiksa
- Tanpa sumber: 0 blokir
- Sumber tidak mendukung: 0
- Kontradiksi: 0 yang mengubah keputusan
- Fakta bisnis berlabel asumsi: 0 (tidak ada ASUMSI baru; nilai visual/merek berstatus UP atau KEPUTUSAN "menunggu Checkpoint 2")

## Verifikasi sumber eksternal (URL dibuka 2026-11-18)
| SRC | Hasil |
| --- | --- |
| SRC-256 WCAG 2.2 | Didukung. Dibuka: SC 2.2.1 (Extend: peringatan dan minimal 20 detik; pengecualian real-time, essential, 20 jam) dan pengecualian logo SC 1.4.3 cocok. Bagian SC 3.3.8 dan 2.5.8 terpotong oleh batas halaman, tidak dibuka ulang; kutipan di sources.md konsisten dengan teks standar. Rumus luminance dan kontras tidak diperiksa ulang (sudah dicek orkestrator) |
| SRC-257 Flutter Accessibility | Didukung: kutipan 48x48, kontras 4,5:1 (dengan pengecualian komponen nonaktif), skala besar, TalkBack/VoiceOver. Halaman ini tidak menyebut 44x44; guideline (A10) menyandarkan 44 dp iOS ke SRC-258, benar |
| SRC-258 Accessibility testing | Didukung: androidTapTargetGuideline 48x48, iOSTapTargetGuideline 44x44, labeledTapTargetGuideline, textContrastGuideline, impor `flutter_test` |
| SRC-259 Plus Jakarta Sans | Didukung: "licensed under the SIL Open Font License, Version 1.1", Gumpita Rahayu/Tokotype, "Variable fonts support (wght)" |
| SRC-260 Flutter `Icons` | Didukung: "Identifiers for the supported Material Icons"; `uses-material-design: true`. Halaman 487 rb karakter, hanya 100 rb pertama dibaca (cukup untuk dua klaim itu). Catatan "lisensi ikon tidak disebut" jujur dan benar |

## Verifikasi fakta bisnis dan konsistensi
- Palet pamflet (#087F78, #DFF4ED, #FAFBF7, #FFFFFF), tab Beranda/Jadwal/Riwayat/Profil, ubin Vaksinasi dan Suntik & Infus Vitamin, dan lingkaran mint tampak lebih gelap dari tulisan: cocok dengan gambar `docs/input/Rumah Sehat Apps.jpeg` (dibuka).
- T5-Q12 (`05-catatan-stack.md`): nama tetap "Rumah Sehat", wordmark/slogan RuangSehat tidak dipakai. Guideline dan wireframe konsisten; "RuangSehat" tidak muncul di wireframe. Catatan penyusun T5-Q12 menyebut logo bertulisan "Rumah Sehat" dibuat di Tahap 6; guideline hanya menyediakan placeholder dan menyerahkan logo final ke UP-60/PT-40 (usulan, tidak diputuskan sepihak).
- CR-015: katalog lab bertanda "Menunggu CR-015" tanpa alur (a08 layar 2), ubin beranda tidak ditampilkan (UP-63), D-E03 tidak menambah butir. Sesuai Keputusan CR-015 di `backlog.md`.
- Dicocokkan ke PRD/ADR: PB-07 30 menit, PB-01 24 jam, PB-03 15 menit, PB-08 pengingat 24 jam dan 1 jam, NFR-SEC-04 30 menit, ADR-011 sesi mutlak 12 jam, NFR-SEC-10 15 menit, AB-12/US-801 AC1 (QRIS Rp0), AB-32/AB-35 (status refund ke pasien), AB-73, KU-02/03/05/09, US-104 AC1 (tujuh informasi Pasal 21), US-1601 AC3, UP-47 (3x24 jam, mendesak 12 jam, tautan 7 hari), UP-30 (7 hari), KEPUTUSAN-07 (NIK 4 digit), US-403 AC2, US-305 AC3/AC4, versi Flutter 3.50.0 dan Next.js 16.4.6 di `05-stack.md`. Semua cocok.
- Register: SRC-256–260 ada beserta URL, tanggal akses, kutipan. KEPUTUSAN-160–176 ada (status "menunggu (Checkpoint 2)"). UP-58–64 baru (PRD memakai UP-01–UP-57, tidak bentrok); PT-40–43 baru (terakhir PT-39). Semua rujukan nomor yang dipakai di dokumen Tahap 6 terdaftar.
- Traceability: 21 berkas HTML ada (cocok dengan 21 alur HTML di §10.1), 18 alur deskripsi; semua story US-101–US-1701 (kecuali US-102 yang dihapus CR-007) punya alur atau alasan "tanpa layar" di §10.3; semua story induknya BR di PRD (kolom Rujukan). Tidak ada TANPA INDUK.
- Wireframe tidak memuat CDN, skrip, atau font jarak jauh (hanya `<link>` ke `wireframe.css` lokal).

## Temuan
Blokir: tidak ada.

Non-blokir (4 butir):
| # | Lokasi | Klaim | Masalah | Usulan |
| --- | --- | --- | --- | --- |
| N1 | `06-design-guideline.md` §7.1 dan baris KEPUTUSAN-160 di decision-log | "Target ini dikonfirmasi user di Checkpoint 2" / "Dikonfirmasi user di Checkpoint 2" | Ditulis seolah sudah terjadi, padahal checkpoint belum berlangsung (status log "menunggu", §13 menyatakan menunggu) | Ganti menjadi "akan dikonfirmasi" atau "menunggu konfirmasi" |
| N2 | `06-design-guideline.md` §6 dan KEPUTUSAN-171 | Di aplikasi pasien "refund" ditulis "pengembalian dana" | PRD memakai "refund" dan tidak memuat frasa "pengembalian dana" (0 kemunculan); ini perubahan istilah UI, sudah berstatus keputusan menunggu Checkpoint 2 | Masukkan ke pertanyaan Checkpoint 2 nomor 6 bersama paket KEPUTUSAN, atau pertahankan "refund" |
| N3 | `06-design-guideline.md` §4 Checkbox | "satu butir persetujuan satu kotak (US-104 AC3)"; "tidak pernah tercentang otomatis" | US-104 AC3 hanya menyatakan butir persetujuan dapat ditambah; aturan "tidak tercentang otomatis" tidak ada di PRD (itu keputusan desain penyusun, tanpa label KEPUTUSAN di baris itu) | Beri label `[KEPUTUSAN-xx]` atau tulis "selaras US-104 AC2/AC3" |
| N4 | KEPUTUSAN-164 vs UP-59 | Font sistem tercatat sebagai KEPUTUSAN sekaligus UP | Duplikasi pencatatan untuk satu pilihan merek; keduanya "menunggu Checkpoint 2" sehingga tidak mengubah keputusan | Cukup UP-59 yang menjadi penentu; KEPUTUSAN-164 menyebut itu |

Catatan lain tanpa tindakan: wireframe A07 menampilkan QRIS dengan "Biaya Rp0" (didukung AB-12 dan US-801 AC1); header A07 menegaskan metode final mengikuti AB-25 `[TIDAK DIKETAHUI]`.
