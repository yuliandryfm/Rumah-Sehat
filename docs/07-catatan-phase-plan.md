# Catatan Tahap 7 (Phase Plan + Backlog)

Catatan apa adanya. File ini menjadi bukti asal setiap usulan dan pertanyaan Tahap 7, dan menjadi bahan Checkpoint 2.

Tanggal: 2026-11-18
Penyusun: Claude (persona Project Manager, dengan tinjauan Tech Lead atas urutan dan dependensi task)
Mode: otomatis (tanpa wawancara user di tahap ini). Keputusan bisnis (scope, penempatan CR, tanggal, biaya, kapasitas tim) dibawa ke Checkpoint 2 sebagai UP atau PT, tidak diputuskan penyusun.
Tingkat ketelitian: T2 Standar; urutan, dependensi, dan definisi task modul kritis T1 ditinjau teliti.

## Input yang dibaca

- `docs/01-business-case.md` (tim, jadwal, toleransi, R-02, R-04, R-08, §9 tinjauan di gate Tahap 7)
- `docs/03-prd.md` v1.1 (semua epic dan story, §2.3 urutan pengurangan scope, §3, §6, §7, §9 G4, §10, §12)
- `docs/04-arsitektur.md` (modul, alur, §12 modul T1, §13 titik perluasan CR-011), `docs/04-data-dictionary.md` (daftar tabel, A9.6, C4.1, A2.4), `docs/04-risk-register.md` (RT-01–RT-31)
- ADR-008, ADR-011 (butir 4, 6, 7), dan ringkasan ADR-006–ADR-025 di arsitektur §1 dan stack
- `docs/05-stack.md` (§1, §2 lingkungan dan jadwal sewa, §3 CI/CD, §6 rancangan `verify.sh`, §7 Keycloak, §13 pertanyaan dan butir verifikasi Tahap 8 Fase 1), `docs/05-catatan-stack.md` (T5-Q7, T5-Q9), `docs/05-physical-schema.md` (struktur)
- `docs/coding-standards.md` (§1 struktur repo dan modul, §7 test, §8 commit dan DoD, §10 checklist, §11)
- `docs/06-design-guideline.md` (§5.5, §7, §9, §10, §11, §12), `docs/06-catatan-design.md`, `docs/qa/tahap-06-review-wireframe.md` (N1–N5)
- `docs/backlog.md` (DoR, DoD, CR-001–CR-016), `docs/traceability.md`, `docs/decision-log.md`, `docs/sources.md`, `docs/STATUS.md`
- Referensi runbook: `anti-halu.md`, `aturan-kerja.md`, `tahap-07-phase-plan.md`, `tahap-08-development.md`, `templates.md`

## Riset (4 halaman resmi; `docs/sources.md`)

| SRC | Halaman | Untuk |
| --- | --- | --- |
| SRC-261 | OWASP Authentication Cheat Sheet | PT-44: pesan galat umum termasuk untuk akun terkunci; lupa password tetap tersedia saat terkunci |
| SRC-262 | Keycloak Server Admin Guide: brute force detection (adoc, dibuka ulang) | PT-44: pesan bawaan Keycloak untuk akun terkunci |
| SRC-263 | W3C Understanding SC 2.2.1 | PT-43: pengecualian essential untuk batas waktu keamanan dan penahanan sumber daya terbatas |
| SRC-264 | Keycloak Server Administration Guide 26.8.2: email realm | Temuan T7-F1: Keycloak mengirim email lewat SMTP |

## Hasil

- `docs/07-phase-plan.md` v0.1: empat fase dengan tanggal usulan, kalender sampai go-live, estimasi effort, analisis kapasitas dan opsi scope, 71 task Fase 1 dengan dependensi dan kelas T1/T2, jalur kritis, gerbang eksternal, penanganan butir jatuh tempo Tahap 7.
- `docs/backlog.md`: kolom Fase terisi untuk 96 story; enabler EN-01–EN-27 dan story UI UI-01–UI-05 dengan induk ADR/KEPUTUSAN dan BR; DoR dan DoD dilengkapi; status Tahap 7 di CR-011, CR-013, CR-014, CR-015.
- `docs/03-prd.md` v1.2 (draf): revisi CR-013 (pajak) di AB-10a, AB-11, AB-13, AB-17, AB-21, AB-27, definisi §3.5, PB-21, US-801 AC1, US-804 AC2, US-811 AC1, US-812 AC4, §12.4. Tarif dan rincian pajak tetap `[TIDAK DIKETAHUI]` (sisa PT-28).
- `docs/traceability.md`: bagian "BR → story → fase (Tahap 7)".
- Register: KEPUTUSAN-178–186, ASUMSI-01–02 (`docs/decision-log.md`); SRC-261–264 (`docs/sources.md`); UP-65–UP-72 dan PT-47–PT-51 (di bawah).
- `docs/test-plan.md` tidak disusun di sini; dikerjakan subagent `qa-engineer` setelah tahap ini.

## Temuan, celah, dan pertentangan (tidak diselesaikan sendiri)

| # | Temuan | Perlakuan |
| --- | --- | --- |
| T7-F1 | Keycloak mengirim email (verifikasi email, lupa password) lewat server SMTP `[SRC-264]`, dan ADR-011 butir 6 memakai required action verifikasi email. Endpoint SMTP Amazon SES tidak tersedia di region Jakarta `[SRC-84]`, dan adaptor email dirancang lewat API (T5-Q4). Jalur email Keycloak belum punya rancangan | Spike F1-T02 di awal Fase 1, hasilnya usulan ADR baru ke user `[KEPUTUSAN-183]`. Memblokir US-101, US-103, US-105 AC4 sampai ADR disetujui (gerbang G-03). Diusulkan masuk risk register sebagai risiko baru |
| T7-F2 | Compose lokal (stack §2.1) tidak memuat object storage, padahal US-106 dan test berkas butuh penyimpanan kompatibel S3 | Spike F1-T03 `[KEPUTUSAN-184]`. Emulator pihak ketiga hanya setelah cek registry dan lisensi |
| T7-F3 | Design guideline §12 PT-42 menulis "Data dictionary belum memuat status draf", padahal data dictionary yang lulus gate sudah punya `consultation_note_draft` (A9.6) dan `rme_medical_record_draft` (C4.1) | Teks guideline §12 PT-42 dikoreksi orkestrator 2026-11-18 karena menyangkut fakta dokumen, bukan keputusan; perilaku draf tetap diusulkan di Checkpoint 2 (UP-72) karena masih menunggu Checkpoint 2 |
| T7-F4 | US-205 AC2 mensyaratkan status integrasi "Aktif" agar rekanan tampil, sedangkan klinik yang hanya menaungi tenaga kesehatan untuk konsultasi online tidak butuh integrasi data (konsultasi berlangsung di Ruang Sehat, BR-013; PRD §1 dan AB-60 tidak memasukkan konsultasi ke data dari integrasi rekanan). Bila Opsi D dipilih, tidak ada klinik yang dapat "Aktif" | PT-47 |
| T7-F5 | KU-10 (pesan "akun nonaktif" saat masuk) dan US-101 AC3 (email terdaftar diarahkan masuk/atur ulang) membuka status akun, yang menurut OWASP sebaiknya memakai pesan umum `[SRC-261]` | Dicatat untuk reviewer keamanan bersama PT-44; keduanya AC yang sudah disetujui, perubahan hanya lewat CR bila user menghendaki (UP-70 butir c) |
| T7-F6 | Mesin pengembangan sesi ini Darwin 21.6.0; kesesuaian toolchain stack dengan versi OS ini belum diketahui | F1-T01 (runbook Tahap 8 langkah 0); opsi VPS pengembangan Linux bila tidak cocok |
| T7-F7 | Kebutuhan kecepatan Fase 3 + Fase 4 (±61,2 jam-task per hari kerja) hampir empat kali Fase 1 (±16,3), dengan tim tanpa angka throughput. Jalur kritis Fase 1 sendiri 65 jam-task berurutan dan memuat layar mobile (F1-T58, F1-T63) | §6 phase plan; UP-66, UP-67, PT-48 |

Tidak ada fakta bisnis `[TIDAK DIKETAHUI]` yang memblokir penyusunan rencana: kapasitas tim yang tidak tercatat ditulis `[TIDAK DIKETAHUI]` dan diganti dengan kriteria gerbang re-plan yang dapat diukur. Tidak ada dokumen gate yang saling bertentangan sampai mengubah keputusan; T7-F3 adalah pertentangan antara draf Tahap 6 (belum lulus checkpoint) dan data dictionary (lulus gate), dan yang berlaku adalah data dictionary.

Risiko baru dicatat orkestrator di `docs/04-risk-register.md` sebagai RT-32 (email Keycloak), RT-33 (kecepatan per fase), dan RT-34 (jendela audit dan toko aplikasi), versi 1.2 draf. Usulan awal: jalur email Keycloak tanpa SMTP (T7-F1); kebutuhan kecepatan per fase melampaui kapasitas (memperkuat RT-10 dan RT-24); jendela audit eksternal dan review toko aplikasi tidak diketahui lamanya (PT-49, PT-51).

## Usulan penyusun untuk Checkpoint 2

| ID | Usulan | Alasan | Alternatif |
| --- | --- | --- | --- |
| UP-65 | Kalender: Fase 1 2026-11-23 – 2026-12-11; Fase 2 2026-12-14 – 2027-01-08; Fase 3 2027-01-11 – 2027-01-29; Fase 4 bersyarat 2027-01-18 – 2027-01-29; pembekuan fitur 2027-01-29; QA/SIT 2027-02-01 – 02-05; audit keamanan 2027-02-01 – 02-12; UAT 2027-02-08 – 02-12; pengajuan toko aplikasi paling lambat 2027-02-12; deploy 2027-02-22 – 02-26 (phase plan §4–§5) | Go-live tetap dengan toleransi 0; staging Desember, uji beban Januari, production 1 Februari sudah disetujui di stack §2.9; tahap 9–13 butuh kode beku | Pembekuan fitur lebih awal (scope lebih kecil) atau lebih lambat (jendela audit/UAT/toko lebih pendek) |
| UP-66 | Urutan layanan dibalik dari urutan pengurangan scope PRD §2.3 (konsultasi dulu, antrean/produk/home visit terakhir); gerbang re-plan 2026-12-11 dengan kriteria kecepatan terukur per opsi (phase plan §6.5) | Pengurangan scope menjadi pelepasan fase utuh, bukan fitur setengah jadi; kapasitas tidak tercatat sehingga diputuskan dengan data Fase 1 (RT-24) | Urutan per epic PRD tanpa gerbang; menetapkan scope final sekarang tanpa data kecepatan |
| UP-67 | Opsi B (lepas antrean, produk, home visit) sebagai baseline rencana; opsi final di gerbang re-plan; pertanyaan kapasitas tambahan (Opsi E) dijawab sekarang | Kebutuhan Opsi A ±61,2 jam-task per hari kerja di Januari (Opsi B ±49,5), dibanding ±16,3 di Fase 1; rekrut butuh waktu | A, C, D, atau E (phase plan §6.6) |
| UP-68 | CR-011 (onboarding mandiri + langganan RME, ±108 jam-task) ke rilis berikutnya | Kapasitas (§6) dan PT-34 belum terjawab; titik perluasan sudah disiapkan arsitektur §13 | (a) rilis 1 penuh; (b) rilis 1 hanya pendaftaran mandiri tanpa penagihan (±45 jam-task) |
| UP-69 | CR-015 (vaksinasi/infus) ke rilis berikutnya; bila rilis 1, hanya di lokasi lab dan dengan jawaban legal sebelum Fase 3 | Syarat regulasi `[TIDAK DIKETAHUI]`; varian home visit memperluas P-05 dan AB-50 serta bersinggungan dengan butir di luar scope BRD §4.2; home visit ada di nomor 3 urutan pengurangan | (a) rilis 1 di lokasi lab (±50–60 jam-task); (b) rilis 1 termasuk home visit (tambahan ±60–100) |
| UP-70 | PT-44: (a) akun terkunci menampilkan pesan yang sama dengan US-103 AC2, dengan revisi teks AC3 "... dan pesan yang tampil sama dengan AC2" `[SRC-261]` `[SRC-262]`; (b) atur ulang password tetap dapat dipakai saat terkunci `[SRC-261]` (perilaku Keycloak diverifikasi F1-T55); (c) catatan untuk reviewer keamanan: KU-10 dan US-101 AC3 (T7-F5), perubahan hanya lewat CR bila dikehendaki | Mencegah enumerasi akun; sejalan dengan perilaku bawaan Keycloak sehingga tanpa kode tambahan | Pesan khusus "akun terkunci" (membuka status akun); opsi tambahan email pemberitahuan ke pemilik saat terkunci (tidak disebut sumber; menambah kode event listener) |
| UP-71 | PT-43: (a) PB-07 diperlakukan sebagai pengecualian essential SC 2.2.1, dengan semua isian diminta sebelum slot ditahan dan mitigasi guideline §5.5; (b) sesi web mutlak 12 jam diperlakukan sebagai batas keamanan essential, dengan peringatan sebelum berakhir dan isian panjang disimpan di server `[SRC-263]` | Understanding SC 2.2.1 menyebut batas waktu keamanan dapat dianggap essential dan memperlakukan penahanan tiket sebagai essential | Jalur perpanjangan PB-07 (menahan slot lebih lama, bertentangan dengan keadilan slot); perpanjangan sesi 12 jam (melemahkan ADR-011) |
| UP-72 | PT-42: draf catatan konsultasi dan rekam medis disimpan di server sesuai data dictionary A9.6/C4.1, disimpan otomatis saat berhenti mengetik dan saat aplikasi ke latar belakang/terkunci `[KEPUTUSAN-186]`; teks guideline §12 PT-42 dikoreksi (T7-F3) | Tabel draf terenkripsi sudah lulus gate; coding standards §1.6 melarang salinan klinis persisten di perangkat | Draf hanya di memori (hilang saat aplikasi ditutup) |

## Pertanyaan terbuka baru

| ID | Pertanyaan | Pemilik | Tenggat |
| --- | --- | --- | --- |
| PT-47 | Klinik yang hanya menaungi tenaga kesehatan untuk konsultasi online (tanpa pendaftaran klinik, lab, atau home visit): apakah tetap wajib berstatus integrasi "Aktif" (US-205 AC2) agar tenaga kesehatannya siap melayani (AB-54), atau cukup terverifikasi dengan markup diatur? | User / CEO | Sebelum Fase 2 (2026-12-14) |
| PT-48 | Kapasitas tim yang dapat dipakai untuk menghitung kelayakan: jam kerja mobile developer per minggu; apakah seluruh task mobile dikerjakannya atau sebagian oleh Claude; berapa sesi Claude per hari yang dapat dijalankan PM; dan apakah ada tambahan tenaga (developer backend/web, QA) bila dibutuhkan (Opsi E) | User (PM), CEO | Checkpoint 2 |
| PT-49 | Siapa auditor keamanan eksternal (syarat go-live 6), kapan tersedia, dan berapa lama audit berlangsung? Jendela usulan 2027-02-01 – 02-12 | User, CEO | Sebelum Fase 3 (pemesanan jadwal) |
| PT-50 | Siapa konsultan medis untuk UAT, dan apakah CEO, CTO, dan konsultan medis tersedia di jendela usulan 2027-02-08 – 02-12? | User | Sebelum Fase 3 |
| PT-51 | Status akun Apple Developer Program dan Google Play Console atas nama PT. Arunika (pendaftaran dan verifikasi), mengingat pengajuan toko usulan paling lambat 2027-02-12 (Business Case R-08) | User | Sebelum Fase 2 berakhir (2027-01-08) |

## Daftar pertanyaan untuk Checkpoint 2 (diajukan satu per giliran)

Pertanyaan Checkpoint 2 dari Tahap 6 (`docs/06-catatan-design.md`) tetap berlaku dan diajukan lebih dulu atau bergantian sesuai orkestrator. Pertanyaan Tahap 7, urut prioritas:

1. Pembagian fase dan kalender (UP-65), termasuk prinsip urutan terbalik dan gerbang re-plan 2026-12-11 (UP-66): setuju?
2. Kapasitas tim (PT-48) dan opsi scope baseline (UP-67): Opsi B sebagai baseline dengan keputusan final di gerbang re-plan, atau opsi lain? Apakah CEO bersedia menimbang tambahan tenaga (E) sekarang?
3. CR-011 onboarding mandiri dan langganan RME (UP-68): rilis 1 penuh, rilis 1 sebagian, atau rilis berikutnya? Bila rilis 1, sertakan jawaban PT-34.
4. CR-015 vaksinasi/infus (UP-69): rilis 1 di lokasi lab, rilis 1 termasuk home visit, atau rilis berikutnya?
5. PT-44 pesan akun terkunci (UP-70): setuju pesan umum dan revisi teks US-103 AC3? Perlu email pemberitahuan penguncian?
6. PT-43 batas waktu (UP-71): PB-07 dan sesi 12 jam diterima sebagai pengecualian essential dengan mitigasinya?
7. PT-42 draf di server (UP-72) dan koreksi teks guideline: setuju?
8. PT-47 status integrasi klinik untuk konsultasi online.
9. Tenggat gerbang eksternal (phase plan §9): PT-39 sebelum 2026-12-14, PT-36 sebelum 2026-12-21, sisa PT-28 sebelum perhitungan pajak, PT-05 sebelum Fase 3, PT-49/PT-50 sebelum Fase 3, PT-51 sebelum 2027-01-08, PT-38 sebelum 2027-02-01, PT-29/PT-30/PT-33/PT-37 sebelum Fase 4, PT-35 sebelum go-live: dapat dipenuhi?
10. Revisi PRD v1.2 untuk CR-013 (pajak): teks disetujui? Sisa PT-28, termasuk perilaku sistem saat tarif pajak belum diatur.
11. Paket keputusan teknis KEPUTUSAN-178–186 dan ASUMSI-01–02, termasuk pemberitahuan bahwa ADR jalur email akan diusulkan di awal Fase 1 (T7-F1): setuju sekaligus?

## Review sebelum gate

Belum dijalankan saat catatan ini ditulis. Fact-checker dan reviewer dijalankan orkestrator; `qa-engineer` menyusun `docs/test-plan.md` dari PRD.
