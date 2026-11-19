# Fact-check CR-021 (penerapan): dokumen gate setelah keputusan putaran 4

Tanggal: 2026-11-19
Dokumen diperiksa: `docs/02-brd.md` v1.4 (§2.2, §4.1–§4.2, BR-040–BR-042, BR-061, BR-103, BR-115, §5.12, L-16–L-21), `docs/03-prd.md` v1.3 (§1.1–§2.2, §3.1, AB-13/17/20, §3.7 PB-23–PB-25, §3.12, EP-18, EP-19, NFR terkait, E-18, §11 UP-79–UP-87, §12.6, §14), `docs/04-data-dictionary.md` v1.1 (A20), `docs/07-phase-plan.md` v0.3 (§0, §6.7, §7.3–§7.4, §8.1–§8.2, §9), ADR-006/011/018/024 (revisi), `docs/04-threat-model.md` v1.3 (§4.19–§4.22, §5), `docs/traceability.md`, `docs/backlog.md` (EP-18/EP-19, CR-021), `docs/decision-log.md` KEPUTUSAN-250–269.
Acuan: `docs/cr/CR-021-wawancara.md` putaran 0–4, `docs/cr/CR-021-analisis.md` v0.3, `docs/fact-check/CR-021.md`.
Tingkat: T1 (autentikasi, data peserta, penagihan), T2 untuk bagian lain.
Hasil: **TIDAK LULUS** (2 temuan blokir, keduanya murah diperbaiki; 5 catatan non-blokir)

## Ringkasan
- Total klaim diperiksa: ±170 (BR-130–BR-145 dan catatan BR lama; 19 story beserta AC yang berubah dari analysis; AB/PB/E/UP; PT-59–PT-79; seluruh aritmetika phase plan §6.7 dan §7.3; ringkasan dan 149 baris status threat model; konsistensi ID di lima dokumen)
- Terverifikasi: ±163
- Tanpa sumber: 0
- Sumber tidak mendukung: 1 (B-1)
- Kontradiksi: 1 blokir (B-2), 4 non-blokir
- Fakta bisnis berlabel asumsi: 0

## Yang terverifikasi (ringkas)
- **Isi sama dengan analisis v0.3 + putaran 4**: BR-130–BR-145 sama dengan §3 analisis, ditambah keputusan putaran 4: K-01 (a), K-03 (BR-141), K-04 (a), K-05, K-12 (L-17), dan K-02 yang menunggu L-21, dengan dua tagihan dan pengecualian AB-13 bila butuh izin BI. Di PRD, UP-79–UP-87 kini berlaku tanpa syarat: US-1909 AC7, US-1910 AC6, US-1915 AC4, AB-85. PT-78 tetap bersyarat (US-1908 AC4a). Verifikator menggantikan "admin berwenang (PT-67)" di US-1902–US-1904 dan US-1916. PT-60, PT-61, PT-67 (sebagian), PT-72, PT-75, dan PT-79 berstatus terjawab sesuai jawaban "1S3 2Y 3Y 4aV 4bY 4cT". Tidak ada BR, story, atau AC dari analysis yang hilang.
- **Label `[USER] C21-*`**: semua cocok dengan wawancara, kecuali B-1.
- **Aritmetika phase plan**: hasil cocok.
  - Kolom Jam F1-T81–F1-T111 berjumlah 109 dari 31 task, 19 di antaranya T1.
  - Task yang dipindah: 25 jam-task (backend/web 18, mobile 7).
  - Fase 1: 80 − 7 + 31 = 104 task; 277 − 25 + 109 = 361 jam-task. Per hari kerja 24,1 (backend/web 324, 21,6; mobile 37, 2,5).
  - Sisa Fase 1: 316 ÷ 16 = 19,8 (backend/web 279 ÷ 16 = 17,4; mobile 2,3).
  - Fase 2: 688 + 25 + 12 = 725 ÷ 16 = 45,3 (backend/web 523, 32,7; mobile 202, 12,6).
  - Fase 3: 754 − 12 + 195 + 36 = 973 ÷ 15 = 64,9 (backend/web 829, 55,3; mobile 144, 9,6). K2 = 195 per kelas (mobile 18).
  - Total Opsi B: 361 + 725 + 973 + 72 = 2.131 (+340, 19%). Total Opsi A 1.967 + 340 = 2.307. C = 900 (60,0); D = 519 (34,6).
- **Threat model §5.1**: 66 + 75 + 8 = 149; 66 + 49 + 7 = 122; 0 + 21 + 1 = 22; 0 + 5 + 0 = 5.
  - Dicek per baris tabel §4: 149 baris, 22 berstatus Sebagian, 5 berstatus Belum, sehingga Tercermin 122. KOR-12 dan B2B-01 kini Tercermin dengan rujukan `[USER] C21-P4`.
  - Jumlah §5.2 per komponen = 149, dan jumlah Tinggi = 66.
- **Konsistensi ID**: semua ID muncul konsisten di BRD/PRD/backlog/traceability/phase plan. Yang diperiksa: BR-130–BR-145, EP-18/EP-19, US-1801/1802/1901–1917, AB-80–AB-87, PB-23–PB-25, E-18, UP-79–UP-87, PT-59–PT-79, L-17–L-21, F1-T81–F1-T111.
  - Setiap story CR-021 punya induk BR di traceability, dan BR-nya sama dengan tabel PRD. Tidak ada story TANPA INDUK.
  - A20 berisi 24 tabel. KEPUTUSAN-250–269 terdaftar, statusnya diperbarui, dan tidak ada fakta bisnis yang berlabel KEPUTUSAN.
  - Isi revisi ADR-006/011/018/024 cocok dengan §7 analisis dan PT-67.

## Temuan
| # | Lokasi (file, bagian) | Klaim | Masalah | Usulan |
| --- | --- | --- | --- | --- |
| B-1 (blokir, §2 butir 1/4) | `02-brd.md` BR-136; `03-prd.md` US-1901 AC7, US-1901 AC1, §12.6 PT-72 dan PT-79 | US-1901 AC7: "diarahkan menghubungi Layanan Pelanggan `[USER] C21-P4` (4c; ... tafsiran penyusun yang dikonfirmasi user)". BR-136: "Bentuk bukti kewenangan dan cara menangani NIB ganda ... tafsiran penyusun yang dikonfirmasi user". PT-72 "**Terjawab**", tenggat "—" | Jawaban user hanya kode "4bY" dan "4cT". Wawancara 4b menulis "Bentuk final dikonfirmasi user", yaitu konfirmasi yang masih akan datang. Phase plan G-27 juga masih membuka bentuk bukti (PT-79). Dokumen gate membacanya sebagai sudah dikonfirmasi. Untuk 4c (arahan ke Layanan Pelanggan) tidak ada pertanyaan atau gerbang yang tersisa, karena PT-72 ditutup. Akibatnya perilaku AC yang berasal dari tafsiran penyusun terkunci dengan label `[USER]` tanpa keputusan user | Ubah menjadi "tafsiran penyusun, menunggu konfirmasi user" di BR-136, US-1901 AC1, dan AC7. Biarkan PT-72 terbuka untuk butir arahan Layanan Pelanggan (dan cabang perusahaan). Tambahkan 4c ke G-27, atau tanyakan langsung ke user |
| B-2 (blokir, §2 butir 3) | `07-phase-plan.md` §6.7 "Opsi lain dengan CR-021 v0.3" | "A: 973 + 200 = 1.173, yaitu 78,2 per hari kerja; total ±2.307" | US-108/US-109 (24) terhitung dua kali. Menurut §6.5–§6.6, Fase 3+4 Opsi A = 930 = 754 + 176, sedangkan 973 (Fase 3 Opsi B v0.3) sudah memuat US-108/US-109 dan Fase 4 = 200 juga memuatnya. Yang benar: 930 − 12 + 195 + 36 = **1.149**, yaitu **76,6** per hari kerja. Total 2.307 (= 361 + 725 + 1.149 + 72) benar, tetapi tidak cocok dengan angka 1.173 di kalimat yang sama. Angka ini masukan gerbang re-plan 2026-12-11 | Koreksi menjadi "A: 930 − 12 + 195 + 36 = 1.149, yaitu 76,6 per hari kerja; total ±2.307" |
| N-1 | `02-brd.md` judul §2.2 "(9 peran)"; `03-prd.md` §1.2 "Sembilan peran"; PRD baris "Dasar: BRD v1.3" | Jumlah peran dan versi dasar | BRD §2.2 kini menyebut 11 peran, dan PRD v1.3 berdasar BRD v1.4 | Samakan judul dan rujukan |
| N-2 | `03-prd.md` US-1903 AC1 | Verifikator mengisi "bukti kewenangan" saat membuatkan perusahaan | PT-79 hanya menyangkut pendaftar mandiri ("bukti kewenangan pendaftar"). Perluasan ke jalur buatan Verifikator adalah tafsiran tanpa label | Tandai sebagai tafsiran, dan bawa ke G-27 bersama sisa PT-67 |
| N-3 | `04-threat-model.md` status M-40–M-56 dan riwayat v1.3 | "Diterapkan (di ADR Diusulkan, CR-021 v0.2)"; "ADR-027–ADR-029 (Diusulkan)" | Basi: ADR-027/028 sudah Diterima | Perbarui saat threat model berikutnya disentuh |
| N-4 | `backlog.md` US-1801; `traceability.md` BR-132 | US-1801 → F1-T83, T89, T94, T96, T97, T111; BR-132 → F1-T85–F1-T90 | Phase plan §4 memetakan US-1801 juga ke F1-T93, dan rentang F1-T85–F1-T90 ikut memuat F1-T89 (situs, bukan member area) | Rapikan rujukan task |
| N-5 | `07-phase-plan.md` §1 dan tabel §6.4 | "Fase 1: 80 task, 277 jam-task"; laju 18,5/43,0/50,3 | Masih angka v0.2 tanpa penanda. Yang berlaku dijelaskan di "Status v0.3" dan §6.7 | Beri penanda "v0.2, lihat §6.7" |

## Batas pemeriksaan
- Riwayat git tidak dibuka (tidak ada akses shell). Pemeriksaan dilakukan atas isi berkas saat ini di branch `tahap-08-fase-1`.
- Jalur kritis (65; rantai CR-021 ke-51) tidak dihitung ulang dari seluruh graf, sama dengan batas fact-check CR-021 sebelumnya.
- Tidak ada SRC baru di penerapan ini. SRC-405–409, SRC-440, dan SRC-441 sudah dibuka di `docs/fact-check/CR-021.md` dan tidak dibuka ulang.
