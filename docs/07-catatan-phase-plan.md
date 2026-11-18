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
| SRC-265 | OWASP ASVS 5.0.0 bab V6 (putaran perbaikan gate) | Nilai kebijakan password (T-02), posisi Level 3 untuk anti-enumerasi (UP-73), temuan sampingan S-01 MFA Level 2 |
| SRC-262 (dibuka ulang) | Keycloak brute force detection | Parameter penguncian T-06 (KEPUTUSAN-190) |
| SRC-266–SRC-274 | Halaman yang dibuka security-architect (ASVS V7, What is the ASVS, Assessment; Forgot Password Cheat Sheet; Keycloak password policies, WebAuthn, flows, brute force 26.8.2, User Registration) | Perbaikan review keamanan putaran 1 |

## Hasil

- `docs/07-phase-plan.md` v0.1: empat fase dengan tanggal usulan, kalender sampai go-live, estimasi effort, analisis kapasitas dan opsi scope, 71 task Fase 1 dengan dependensi dan kelas T1/T2, jalur kritis, gerbang eksternal, penanganan butir jatuh tempo Tahap 7.
- `docs/backlog.md`: kolom Fase terisi untuk 96 story; enabler EN-01–EN-27 dan story UI UI-01–UI-05 dengan induk ADR/KEPUTUSAN dan BR; DoR dan DoD dilengkapi; status Tahap 7 di CR-011, CR-013, CR-014, CR-015.
- `docs/03-prd.md` v1.2 (draf): revisi CR-013 (pajak) di AB-10a, AB-11, AB-13, AB-17, AB-21, AB-27, definisi §3.5, PB-21, US-801 AC1, US-804 AC2, US-811 AC1, US-812 AC4, §12.4. Tarif dan rincian pajak tetap `[TIDAK DIKETAHUI]` (sisa PT-28).
- `docs/traceability.md`: bagian "BR → story → fase (Tahap 7)".
- Register: KEPUTUSAN-178–186, ASUMSI-01–02 (`docs/decision-log.md`); SRC-261–264 (`docs/sources.md`); UP-65–UP-72 dan PT-47–PT-51 (di bawah).
- `docs/test-plan.md` tidak disusun di sini; dikerjakan subagent `qa-engineer` setelah tahap ini.
- Putaran perbaikan gate (2026-11-18): `docs/07-klarifikasi-ac-fase-1.md` menjawab temuan QA test-plan §9 T-01–T-22 untuk story Fase 1 (T-20 KU-02, T-21 US-1501, dan T-22 US-1504 ikut karena berlaku di Fase 1), dan memetakan T-23–T-35 ke fase dan tenggatnya. Teks AC di PRD tidak diubah sampai Checkpoint 2. Register: UP-73–UP-77 (di bawah), KEPUTUSAN-187–197, SRC-265, SRC-262 dibuka ulang.
- Perbaikan atas review keamanan putaran 1 (`docs/review/tahap-07-security.md`, verdict tidak lulus dengan 1 blokir B-01): klarifikasi v0.2.
  - Pesan seragam untuk semua kegagalan login.
  - Tindak lanjut C-01–C-14.
  - Register baru: UP-78, KEPUTUSAN-198–201, SRC-266–274 (R-2–R-10 review), dan kutipan R-1 ditambahkan ke SRC-265. KEPUTUSAN-188, -190, dan -192 diperbarui. Kolom Fase story terdampak di `docs/backlog.md` diberi catatan DoR.

## Temuan, celah, dan pertentangan (tidak diselesaikan sendiri)

| # | Temuan | Perlakuan |
| --- | --- | --- |
| T7-F1 | Keycloak mengirim email (verifikasi email, lupa password) lewat server SMTP `[SRC-264]`, dan ADR-011 butir 6 memakai required action verifikasi email. Endpoint SMTP Amazon SES tidak tersedia di region Jakarta `[SRC-84]`, dan adaptor email dirancang lewat API (T5-Q4). Jalur email Keycloak belum punya rancangan | Spike F1-T02 di awal Fase 1, hasilnya usulan ADR baru ke user `[KEPUTUSAN-183]`. Memblokir US-101, US-103, US-105 AC4 sampai ADR disetujui (gerbang G-03). Diusulkan masuk risk register sebagai risiko baru |
| T7-F2 | Compose lokal (stack §2.1) tidak memuat object storage, padahal US-106 dan test berkas butuh penyimpanan kompatibel S3 | Spike F1-T03 `[KEPUTUSAN-184]`. Emulator pihak ketiga hanya setelah cek registry dan lisensi |
| T7-F3 | Design guideline §12 PT-42 menulis "Data dictionary belum memuat status draf", padahal data dictionary yang lulus gate sudah punya `consultation_note_draft` (A9.6) dan `rme_medical_record_draft` (C4.1) | Teks guideline §12 PT-42 dikoreksi orkestrator 2026-11-18 karena menyangkut fakta dokumen, bukan keputusan; perilaku draf tetap diusulkan di Checkpoint 2 (UP-72) karena masih menunggu Checkpoint 2 |
| T7-F4 | US-205 AC2 mensyaratkan status integrasi "Aktif" agar rekanan tampil, sedangkan klinik yang hanya menaungi tenaga kesehatan untuk konsultasi online tidak butuh integrasi data (konsultasi berlangsung di Ruang Sehat, BR-013; PRD §1 dan AB-60 tidak memasukkan konsultasi ke data dari integrasi rekanan). Bila Opsi D dipilih, tidak ada klinik yang dapat "Aktif" | PT-47 |
| T7-F5 | KU-10 (pesan "akun nonaktif" saat masuk) dan US-101 AC3 (email terdaftar diarahkan masuk/atur ulang) membuka status akun, yang menurut OWASP sebaiknya memakai pesan umum `[SRC-261]` | Dicatat untuk reviewer keamanan bersama PT-44; keduanya AC yang sudah disetujui, perubahan hanya lewat CR bila user menghendaki (UP-70 butir c). Digabung ke keputusan terpadu enumerasi UP-73 (`docs/07-klarifikasi-ac-fase-1.md` §2) |
| T7-F8 | Temuan QA test-plan §9 T-01–T-22: AC story Fase 1 belum seluruhnya jelas dan dapat diuji, sehingga gate DoR belum terpenuhi | `docs/07-klarifikasi-ac-fase-1.md`: 11 butir Teknis (KEPUTUSAN-187–197), 6 Bisnis, dan 5 Campuran (UP-73–UP-77). DoR terpenuhi setelah klarifikasi disetujui di Checkpoint 2 |
| T7-F9 | ASVS 5.0.0 V6 menyatakan MFA wajib mulai Level 2 (`[SRC-265]`), sedangkan ADR-011 menargetkan Level 2 tetapi pasien tanpa MFA | Menyangkut ADR yang diterima; tidak diputuskan penyusun. Review keamanan (`docs/review/tahap-07-security.md` §3) mengonfirmasi temuan ini. Penerimaan risikonya sudah ada sejak Tahap 4 (APP-04); yang keliru adalah klaim kepatuhan L2. Dibawa sebagai pertanyaan Checkpoint 2 nomor 14 dengan opsi M1–M4 (klarifikasi §5.1) |
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
| UP-73 | Keputusan terpadu enumerasi akun dan NIK, mencakup dan memperluas UP-70 (PT-44) dan T7-F5. Jalur tanpa login tidak memberi sinyal keberadaan akun:<br>- pendaftaran dengan email terdaftar dijawab sama dengan pendaftaran baru, dan petunjuk dikirim ke email itu;<br>- semua kegagalan login (password salah, akun tidak ada, terkunci, nonaktif) memakai satu pesan seragam yang identik, termasuk kalimat petunjuk Lupa password dan Layanan Pelanggan (perbaikan B-01 review keamanan);<br>- batas laju per email diam.<br>Jalur yang sudah login:<br>- layar "Akun nonaktif" tetap untuk sesi terbuka;<br>- ubah email ke alamat terpakai dijawab sama, dan pemberitahuan dikirim ke alamat itu;<br>- NIK pemilik akun ganda ditolak dengan pesan umum, dengan batas per akun, per NIK lintas akun, per IP, dan global.<br>Default ini menegaskan ADR-011 butir 4; opsi A untuk pesan terkunci atau nonaktif merevisinya. Kontrol pendukungnya ada di klarifikasi §2.7 (klarifikasi §2) | Anti-enumerasi adalah Level 3 di ASVS 5.0.0 (6.3.8), bukan kewajiban target Level 2, jadi ini penerimaan risiko `[SRC-265]`. OWASP menganjurkan pesan umum termasuk untuk akun terkunci atau nonaktif `[SRC-261]`. Perilaku bawaan Keycloak sudah umum `[SRC-262]` | Opsi A per alur: pertahankan AC sekarang dan catat risiko M-12. NIK: antrean dengan penolakan umum oleh Verifikator (opsi C) |
| UP-74 | Siklus status identitas: (a) pengajuan NIK baru dari "Disetujui" diizinkan dan status menjadi "Menunggu verifikasi" (UP-39); (b) setelah batal, status kembali seperti sebelum pengajuan; pengajuan karena perubahan nama atau tanggal lahir tidak dapat dibatalkan; (c) bila persetujuan gagal karena NIK sudah dipakai, Verifikator menolak dengan alasan yang sudah terpilih (klarifikasi T-14, T-15, T-18) | Konsisten dengan UP-39 dan registri NIK di data dictionary A2.8; keputusan tetap di tangan manusia | (a) hanya lewat Layanan Pelanggan; (b) selalu "Belum diunggah"; (c) penolakan otomatis oleh sistem, atau tetap "Menunggu" |
| UP-75 | Data rujukan dikelola lewat migrasi data yang direview, tanpa layar admin di rilis 1: (a) versi teks dan butir persetujuan; (b) daftar alasan penolakan identitas (daftar awal di klarifikasi T-17) (klarifikasi T-11, T-17) | Tanpa tambahan scope; sejalan dengan seed F1-T49 dan larangan mengubah versi yang sudah terbit | Layar backoffice pengelolaan (menambah story) |
| UP-76 | Admin Ruang Sehat: (a) bootstrap Super Admin pertama lewat perintah satu kali di jalur administrasi production, hanya bila belum ada Super Admin; minimal dua Super Admin sebelum go-live; (b) menonaktifkan akun sendiri ditolak; (c) Auditor tidak boleh digabung dengan sub-peran lain (klarifikasi T-19) | File migrasi tidak boleh memuat data pribadi; konsisten dengan US-208 AC3 dan ADR-012 butir 7; menjaga independensi Auditor | (a) seed di migrasi; (b) diizinkan bila masih ada Super Admin lain; (c) kombinasi bebas, atau pembatasan lebih luas |
| UP-78 | Celah ASVS Level 2 manajemen sesi:<br>(a) 7.5.2: layar "Perangkat dan sesi aktif" (daftar dan tombol akhiri) di semua aplikasi, dibangun di Fase 2;<br>(b) 7.1.2: jumlah sesi paralel tidak dibatasi untuk pasien dan tenaga kesehatan dan didokumentasikan; untuk admin, dibatasi atau tidak diputuskan user<br>(klarifikasi §5.2) | Target L2 di ADR-011 butir 1; kedua butir belum dirancang (H-01) `[SRC-266]` | (a) hanya tombol "keluar dari semua perangkat", atau ditunda, sebagai penyimpangan terdokumentasi; (b) satu sesi web per akun admin |
| UP-77 | Parameter usia minimum pemilik akun pasien. Nilainya dari L-08, atau diisi user sebelum go-live; di Fase 1 diisi nilai uji (klarifikasi T-12) | Data anak wajib persetujuan orang tua atau wali (UU PDP Pasal 25 `[SRC-15]`); PRD belum membatasi usia pendaftar | Tanpa batas usia sampai L-08 terjawab |

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
5. Enumerasi akun dan NIK (UP-73), yang menggantikan pertanyaan UP-70/PT-44 dan mencakup T7-F5: setuju prinsip "jalur tanpa login tidak membuka keberadaan akun" beserta perilaku per alur di klarifikasi §2.4, termasuk batas NIK berlapis?
   - Default (pesan seragam untuk terkunci dan nonaktif) **menegaskan ADR-011 butir 4**. Memilih pesan khusus terkunci atau nonaktif (opsi A) berarti **merevisi ADR-011 butir 4**, yang sudah diterima.
   - Untuk setiap alur yang tidak disetujui, risikonya diterima dan dicatat.
   - Perlu email pemberitahuan penguncian?
6. PT-43 batas waktu (UP-71): PB-07 dan sesi 12 jam diterima sebagai pengecualian essential dengan mitigasinya?
7. PT-42 draf di server (UP-72) dan koreksi teks guideline: setuju?
8. PT-47 status integrasi klinik untuk konsultasi online.
9. Tenggat gerbang eksternal (phase plan §9): PT-39 sebelum 2026-12-14, PT-36 sebelum 2026-12-21, sisa PT-28 sebelum perhitungan pajak, PT-05 sebelum Fase 3, PT-49/PT-50 sebelum Fase 3, PT-51 sebelum 2027-01-08, PT-38 sebelum 2027-02-01, PT-29/PT-30/PT-33/PT-37 sebelum Fase 4, PT-35 sebelum go-live: dapat dipenuhi?
10. Revisi PRD v1.2 untuk CR-013 (pajak): teks disetujui? Sisa PT-28, termasuk perilaku sistem saat tarif pajak belum diatur.
11. Paket keputusan teknis KEPUTUSAN-178–186 dan ASUMSI-01–02, termasuk pemberitahuan bahwa ADR jalur email akan diusulkan di awal Fase 1 (T7-F1): setuju sekaligus?
12. Klarifikasi AC Fase 1 (`docs/07-klarifikasi-ac-fase-1.md`), butir bisnis digabung per tema:
    - (a) siklus status identitas (UP-74);
    - (b) data rujukan lewat migrasi data, termasuk daftar alasan penolakan awal (UP-75);
    - (c) admin Ruang Sehat (UP-76):
      - bootstrap dengan persetujuan orang kedua lewat Control Group OpenBao;
      - siapa Super Admin pertama dan siapa orang kedua penyetujunya;
      - minimal dua Super Admin;
      - Auditor eksklusif;
      - perlindungan Auditor terakhir;
      - pemberitahuan pemberian sub-peran sensitif, atau mencatat risiko WEB-07/M-16 sebagai terbuka;
    - (d) usia minimum pemilik akun (UP-77);
    - (e) change request kecil untuk ubah password saat login (US-103 AC6 usulan, ASVS 6.2.2/6.2.3 L1).
    Setelah disetujui, teks AC PRD diperbarui sesuai klarifikasi.
13. Paket keputusan teknis klarifikasi AC KEPUTUSAN-187–201: setuju sekaligus? Isinya:
    - tautan email, kebijakan password, dan penguncian (termasuk Max Wait);
    - satuan berkas dan batas laju;
    - penolakan sesi, validasi profil, autentikasi ulang, dan batas waktu klien;
    - peringatan audit;
    - penghentian sesi setelah perubahan kredensial, atur ulang yang tidak melewati MFA, dan pemberitahuan email;
    - pengerasan jalur pendaftaran.
    Catatan:
    - Bagian batas NIK di KEPUTUSAN-192 mengikuti keputusan UP-73 (nomor 5), tidak ikut tersetujui lewat paket ini.
    - Penundaan peringatan audit ke Fase 2 (KEPUTUSAN-197) bergantung pada KEPUTUSAN-179. Bila disetujui, definisi F1-T69 disesuaikan.
14. **Perubahan ADR yang sudah diterima (ADR-011)**, setelah review keamanan.
    - (a) MFA pasien terhadap target ASVS Level 2 (T7-F9, klarifikasi §5.1). Pilih satu opsi:
      - M1: MFA wajib (TOTP dan kode pemulihan; WebAuthn bila disetujui);
      - M2: MFA sukarela dengan step-up wajib untuk aksi berisiko;
      - M3: tanpa MFA, penyimpangan 6.3.3 terdokumentasi dengan kompensasi;
      - M4: target V6 realm pasien diturunkan ke L1.
      Faktor kedua yang tersedia tanpa penyedia luar negeri: TOTP, kode pemulihan, dan WebAuthn terikat perangkat. Lokasi pemrosesan passkey yang disinkronkan lewat awan `[TIDAK DIKETAHUI]`. Penyusun tidak memberi default.
    - (b) Celah L2 manajemen sesi (UP-78): layar sesi aktif dan kebijakan sesi paralel.

## Review sebelum gate

- Fact-checker rencana fase (`docs/fact-check/tahap-07.md`): LULUS, 0 blokir. Lima catatan non-blokir (jalur kritis, satuan estimasi, angka CR-011, label) diperbaiki; jalur kritis dihitung ulang orkestrator dengan skrip menjadi 65 jam-task.
- QA (`docs/test-plan.md`): 674 test case; semua story Must punya test case; 35 temuan AC (§9). Temuan T-01–T-22 menyentuh story Fase 1, sehingga gate Definition of Ready ditangani lewat `docs/07-klarifikasi-ac-fase-1.md` (putaran perbaikan 1).
- Fact-checker klarifikasi (`docs/fact-check/tahap-07-klarifikasi.md`): LULUS, 0 blokir; konfirmasi teks ASVS 5.0.0 "From L2 onwards, multi-factor authentication mechanisms are required" (S-01).
- Security architect (`docs/review/tahap-07-security.md`): putaran 1 TIDAK LULUS (B-01: pesan akun nonaktif membuka status akun); putaran 2 LULUS DENGAN CATATAN, 0 blokir. Catatan label baris M4 dan rumusan alasan penolakan jalur asinkron diperbaiki orkestrator; P2-1 (ukur p95 pendaftaran email baru di F1-T50, keputusan bila > 900 ms) dibawa ke Tahap 8.
- Gate: enabler fondasi di Fase 1 dan test plan dari PRD terpenuhi; Definition of Ready story Fase 1 terpenuhi setelah klarifikasi AC disetujui di Checkpoint 2; persetujuan pembagian fase di Checkpoint 2.

## Tindak lanjut Checkpoint 2: phase plan v0.2 (2026-11-18)

Sumber keputusan: `docs/checkpoint-2.md` bagian "Jawaban user" (label `[USER] C2-Qx` dan `[USER] C2-Bn`). Persona: Project Manager dengan tinjauan Tech Lead. Tingkat ketelitian T2. Berkas yang diubah: `docs/07-phase-plan.md` (v0.2), catatan ini, `docs/sources.md` (SRC-280–SRC-283), `docs/decision-log.md` (KEPUTUSAN-215–KEPUTUSAN-223).

### Status usulan UP-65–UP-78

| ID | Status | Sumber |
| --- | --- | --- |
| UP-65 | **Disetujui** apa adanya (tanpa Fase 4, Opsi B). Tanggal fase tetap; hari kerja dihitung ulang setelah libur | `[USER] C2-Q5` |
| UP-66 | **Disetujui** dengan kriteria phase plan §6.5 | `[USER] C2-Q6` |
| UP-67 | **Disetujui**: Opsi B baseline, final di gerbang re-plan 2026-12-11; Opsi E dijalankan (developer backend/web mulai awal Januari 2027) | `[USER] C2-Q1, C2-Q2, C2-Q8` |
| UP-68 | **Disetujui** opsi (c): CR-011 ke rilis berikutnya | `[USER] C2-B5` (delegasi) |
| UP-69 | **Disetujui** opsi (c): CR-015 ke rilis berikutnya; ubin vaksinasi/infus tidak tampil | `[USER] C2-B6` (delegasi) |
| UP-70 | **Tercakup** UP-73 | `[USER] C2-B4` (delegasi) |
| UP-71 | **Disetujui** (PB-07 dan sesi 12 jam sebagai pengecualian essential) | `[USER] C2-B15` (delegasi) |
| UP-72 | **Disetujui** (draf di server) | `[USER] C2-B15` (delegasi) |
| UP-73 | **Disetujui** opsi B untuk semua alur; batas NIK berlapis 3/24 jam per akun, 3/24 jam per NIK lintas akun, 10/24 jam per IP, peringatan > 20/jam global; email pemberitahuan penguncian tidak dibangun di rilis 1 | `[USER] C2-B4` (delegasi) |
| UP-74 | **Disetujui** | `[USER] C2-B14` (delegasi) |
| UP-75 | **Disetujui**: migrasi data, tanpa layar admin | `[USER] C2-B14` (delegasi) |
| UP-76 | **Disetujui** (bootstrap dengan persetujuan orang kedua, minimal dua Super Admin, menonaktifkan akun sendiri ditolak, Auditor eksklusif, Auditor terakhir tidak dapat dicabut, pemberitahuan pemberian sub-peran sensitif). Siapa Super Admin pertama dan penyetujunya: PT-52 | `[USER] C2-B7` (delegasi) |
| UP-77 | **Disetujui** (parameter usia; nilai dari L-08) | `[USER] C2-B14` (delegasi) |
| UP-78 | **Disetujui**: (a) layar "Perangkat dan sesi aktif" lengkap di Fase 2 untuk semua aplikasi, menjadi US-110; (b) sesi paralel tidak dibatasi untuk semua jenis akun, didokumentasikan | `[USER] C2-Q10, C2-Q11` |

Temuan T7-F9 (MFA pasien terhadap ASVS L2) ditutup dengan opsi M1, MFA wajib untuk pasien `[USER] C2-Q9` (CR-018). Revisi ADR-011 dikerjakan pemegang berkas lain.

### Status pertanyaan terbuka

| ID | Status |
| --- | --- |
| PT-42, PT-43, PT-44 | Selesai (UP-72, UP-71, UP-73) |
| PT-47 | Tetap terbuka, tenggat sebelum Fase 2 (2026-12-14) |
| PT-48 | Sebagian terjawab: mobile developer penuh waktu ±40 jam/minggu dan mengerjakan seluruh layar mobile (C2-Q3); sesi Claude ≥ 6 jam per hari kerja (C2-Q4); tambahan developer backend/web mulai awal Januari 2027 (C2-Q2, C2-Q8). Throughput dalam jam-task tetap diukur di Fase 1 |
| PT-49, PT-50, PT-51 | Tetap terbuka |
| PT-52 (baru) | Super Admin pertama dan orang kedua penyetuju bootstrap; tenggat sebelum EN-26 (2027-02-01); gerbang G-22 |
| PT-53 (baru) | Developer backend/web tambahan: jumlah orang, biaya terhadap toleransi +20%, tanggal mulai pasti; tenggat gerbang re-plan 2026-12-11; gerbang G-23 |

### Riset hari libur (SRC-280–SRC-283)

| SRC | Halaman | Hasil |
| --- | --- | --- |
| SRC-280 | Setneg, SKB 2026 | 24 Desember 2026 (Kamis) cuti bersama, 25 Desember 2026 (Jumat) libur nasional; tidak ada libur di November dan 1–23 Desember 2026 |
| SRC-281 | Kemenko PMK, SKB 2026 | Hanya judul dan jumlah (17 libur nasional, 8 cuti bersama); lampiran PDF tidak dibuka; dipakai sebagai pembanding |
| SRC-282 | Kemenko PMK, SKB 2027 | 1 Januari (Jumat), 5 Januari (Selasa), 6 Februari (Sabtu) libur nasional; 5 Februari (Jumat) cuti bersama; rangkaian 8–15 Maret 2027 |
| SRC-283 | Setneg, SKB 2027 | Tanggal sama dengan SRC-282; cuti bersama swasta diatur pimpinan masing-masing |

SKB 2027 sudah terbit (ditandatangani 15 September 2026), sehingga tanggal 2027 tidak perlu ditulis `[TIDAK DIKETAHUI]`. Perubahan atas SKB 2026 tidak ditemukan lewat pencarian (tidak ada halaman resmi perubahan untuk dibuka).

### Perubahan phase plan v0.2

- Hari kerja: Fase 1 15, Fase 2 16 (sebelumnya 20), Fase 3 15, Februari 9 (sebelumnya 10); mobile developer 50 hari kerja di 2026-11-23 – 2027-02-05 (sebelumnya ±55).
- Fase 1: F1-T72–F1-T80 (CR-018 dan CR-017), 80 task, 277 jam-task, 54 task T1; jalur kritis tetap 65 jam-task.
- Fase 2: US-110 (XL), US-207 AC5 reset MFA pasien (M), CR-017 dan daftar ulang TOTP realm `nakes` (M), notifikasi sub-peran sensitif (S): +66 jam-task.
- Fase 3: US-108/US-109 (Opsi B), CR-017, daftar ulang TOTP, dan US-110 realm `rme` (M).
- Fase 4 ditandai tidak dikerjakan di baseline Opsi B.
- Effort: A ±1.967 (+42), B ±1.791, C ±1.718, D ±1.337 jam-task.
- Kebutuhan jam-task per hari kerja: Fase 1 18,5; Fase 2 43,0; Fase 3 Opsi A (+Fase 4) 62,0, B 50,3, C 45,4, D 20,0; Februari 8,0.
- Gerbang re-plan mengukur kecepatan per jalur (`v_mobile`, `v_bw`) dan menyajikan kekurangan Fase 3 jalur backend/web yang harus ditutup Opsi E.

### Temuan dan pertentangan (tidak diselesaikan sendiri)

| # | Temuan | Perlakuan |
| --- | --- | --- |
| C2-F1 | Pemegang PRD menambah US-103 AC7 (daftar ulang TOTP) dan AC8 (perangkat hilang) serta US-207 AC5 (reset MFA pasien oleh Layanan Pelanggan), dengan rancangan KEPUTUSAN-202–KEPUTUSAN-205. Instruksi awal tugas ini hanya menyebut pendaftaran TOTP, kode pemulihan, Reset Credentials, layar mobile, dan test negatif | Task Fase 1 diselaraskan dengan AC baru (F1-T75, F1-T77, F1-T79, F1-T80); US-207 AC5 dijadwalkan di Fase 2 (KEPUTUSAN-217). KEPUTUSAN-202–205 berstatus menunggu review keamanan; bila berubah, task Fase 1 menyesuaikan |
| C2-F2 | `docs/decision-log.md` ASUMSI-01 dan KEPUTUSAN-178/182 masih menulis 245 jam-task dan "libur tidak dikurangi" | Di luar rentang yang boleh diubah tugas ini; dilaporkan ke orkestrator. KEPUTUSAN-215 dan KEPUTUSAN-216 menggantikan bagian itu |
| C2-F3 | `docs/checkpoint-2.md` (Temuan penting 1 dan 3) memuat angka v0.1 (±1.857 jam-task, 16,3 → 61,2) | Catatan historis checkpoint; tidak diubah (bukan berkas tugas ini) |
| C2-F4 | Phase plan v0.1 §2 mencatat mesin pengembangan Darwin 21.6.0, sedangkan sesi revisi ini berjalan di Linux | Baris §2 diperbarui; mesin Tahap 8 tetap diperiksa di F1-T01 |
| C2-F5 | Backlog dan traceability perlu task F1-T72–F1-T80, US-110 di Fase 2, US-108/US-109 di Fase 3, dan penanda Fase 4 tidak dikerjakan | Dikerjakan pemegang berkas lain |
| C2-F6 | Ukuran task yang definisinya disesuaikan klarifikasi AC (klarifikasi §6) belum dinilai ulang | KEPUTUSAN-223: dinilai ulang setelah PRD v1.2 final, sebelum task terdampak pertama dimulai |
