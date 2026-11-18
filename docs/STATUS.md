# Status Runbook

Mode terakhir: otomatis

Diperbarui otomatis oleh `/runbook`. Status: `belum` | `berjalan` | `selesai (menunggu checkpoint N)` | `selesai`.

| # | Tahap | Status | Tanggal gate | Catatan |
| --- | --- | --- | --- | --- |
| 0 | Intake | selesai | 2026-11-17 | Jalur B (wawancara). Brief disetujui user (Checkpoint 1). Fact-check LULUS setelah 1 perbaikan. 15 pertanyaan terbuka, 4 risiko untuk Tahap 1 |
| 1 | Business Case | selesai | 2026-11-17 | Layak dilanjutkan (keputusan Executive). Fact-check LULUS, 0 blokir. KEPUTUSAN-01/02 dikonfirmasi. PT baru: PT-16–PT-19 (PT-17 sebelum Tahap 2, PT-19 masih terbuka) |
| 2 | BRD | selesai | 2026-11-17 | BRD v1.0, 67 BR. Compliance LULUS (1 blokir MDR QRIS ditutup via Q12), fact-check LULUS 0 blokir. CR-001–CR-004. 14 konfirmasi legal (L-01–L-14), 7 sebelum Tahap 4 (PT-20). PT-26, PT-27 dijawab awal Tahap 3. Business Case perlu update markup (CR-004). BRD v1.1 (CR-005: semua layanan dibayar di aplikasi) disetujui 2026-11-17; fact-check dan compliance CR-005 LULUS. PT-28 (pajak) baru |
| 3 | PRD + NFR | selesai | 2026-11-17 | PRD v1.0: 17 epic, 97 story, NFR, audit trail, model data. BRD v1.2 (CR-006: RME minimalis aplikasi terpisah + SATUSEHAT, antrean real-time; anggota keluarga). QA LULUS (19 blokir, 2 putaran), kepatuhan LULUS (1 blokir), fact-check LULUS (2 blokir). UP-01–UP-57 dan KEPUTUSAN-03–17 disetujui (T3-Gate). G4: kelayakan scope dinilai di Tahap 7; PSE RME + registrasi SATUSEHAT (PT-32) dimulai segera. G5: RTO 4 jam kerja/8 jam luar. PT-29–PT-33 baru; L-15 baru; eskalasi PT-20 (legal belum ditunjuk) |
| 4 | Arsitektur | selesai | 2026-11-18 | Tingkat T1. ADR-006–ADR-023 diterima; ERD/DD dikunci (RS 139, audit 6, RME 48 tabel). DBA LULUS dgn catatan (3 putaran), threat model LULUS (54 ancaman Tinggi tercermin), fact-check LULUS (3 bagian). T4-Q1 semua penyedia di Indonesia → CR-007 (tanpa login Google/Apple, push, WhatsApp); T4-Q4 Keycloak + Redis; CR-008 verifikasi resep, CR-009 kunci app nakes 15 menit, CR-010 AB-36, CR-011 onboarding mandiri/langganan RME (diputuskan Tahap 7, PT-34). Retensi sementara log 30 hr/1 th, backup 35 hr (G2). PT-18/19/25/28 ke gate Tahap 5; PT-32 sedang diurus. Eskalasi PT-20 (legal belum ditunjuk) |
| 5 | Stack + Coding Standards | selesai | 2026-11-18 | Rust (backend RS + RME), Next.js (web, BFF), Flutter (T5-Q1); VPS Biznet Gio utama + Hostinger cadangan, OpenBao pengganti KMS (T5-Q2, CR-012, ADR-024/025); DOKU utama/Xendit cadangan, SES Jakarta, Lalamove; persetujuan deploy production di sisi VPS. DBA dan keamanan LULUS DENGAN CATATAN (2 putaran), fact-check LULUS. PT-18/25 di luar anggaran proyek, PT-19 penuh waktu sampai go-live, PT-28 sebagian (CR-013 pajak di atas harga + markup). CR-014, RT-21–RT-31. Risiko G-01/02/03/04/06/08, RT-31 diterima sementara (PT-39). PT-35–PT-39 baru. Nama tetap "Rumah Sehat" (T5-Q12; diganti menjadi "Ruang Sehat" oleh CR-016, 2026-11-18). PDF ringkasan BRD dan presentasi BOD di docs/ringkasan/ |
| 6 | Design Guideline | selesai (menunggu checkpoint 2) | 2026-11-18 | Mode otomatis, T2. Guideline v0.1 (token, komponen, aksesibilitas WCAG 2.2 AA), 39 alur (21 wireframe HTML, 18 deskripsi). QA LULUS DENGAN CATATAN, fact-check LULUS, 0 blokir. KEPUTUSAN-160–177, UP-58–64, PT-40–44 menunggu Checkpoint 2. Teks #087F78 di atas mint gagal kontras (4,24:1), diusulkan #065F59. Pesan akun terkunci dinetralkan (PT-44, Tahap 7) |
| 7 | Phase Plan + Backlog | belum | | |
| 8 | Development + Test | belum | | |
| 9 | QA + SIT | belum | | |
| 10 | Audit + Security | belum | | |
| 11 | UAT | belum | | |
| 12 | Pre-deploy | belum | | |
| 13 | Deploy | belum | | |
| 14 | Hypercare | belum | | |
