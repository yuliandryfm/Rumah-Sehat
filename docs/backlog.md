# Backlog

Diisi mulai Tahap 3 (PRD). Format story dan change request: lihat `.claude/skills/runbook/references/templates.md`.

## Definition of Ready

- Acceptance criteria jelas dan bisa diuji
- Merujuk ke bagian PRD dan ADR yang relevan
- Dependensi sudah selesai atau jelas
- Ukurannya muat dalam satu sesi kerja (satu PR yang bisa direview dalam 15 menit)

## Definition of Done

- CI hijau: lint, type check, test, security scan
- Mengikuti pola reference implementation
- Lolos review subagent `code-reviewer`
- Audit trail tercatat untuk aksi sensitif
- File migrasi disertakan kalau ada perubahan skema
- Acceptance criteria terverifikasi

## Epic dan Story

_Belum ada._

## Bug

_Belum ada._

## Change Request

### CR-001: Semua tenaga kesehatan berpraktik atas nama klinik rekanan

Diajukan: 2026-11-17 (wawancara Tahap 2, `docs/02-catatan-brd.md` Q2)
Deskripsi: Dokter, psikolog, fisioterapis, dan tenaga medis yang melayani di Rumah Sehat berpraktik atas nama klinik rekanan (fasyankes). PT. Arunika berperan sebagai platform (PSE). Tenaga kesehatan "mitra langsung" Rumah Sehat tanpa fasyankes ditiadakan. Mengubah brief PS6e.
Alasan: riset kepatuhan Tahap 2 (`docs/compliance/tahap-02-riset.md` temuan 1.1, SRC-02, SRC-03) mengindikasikan telemedisin diselenggarakan fasyankes, sendiri atau bekerja sama dengan PSE terdaftar.
Dampak ke scope / waktu / arsitektur: konsultasi online kini bergantung pada klinik rekanan (R-01 meluas). Model data menghubungkan setiap nakes ke klinik rekanan. Dikonfirmasi legal sebelum Tahap 4 (L-01).
Keputusan: Diterima user (T2-Q2), berlaku sejak BRD v1.0

### CR-002: Apotek rekanan menjadi outlet rekanan untuk obat bebas

Diajukan: 2026-11-17 (`docs/02-catatan-brd.md` Q3, Q3a, Q3b)
Deskripsi: Apotek rekanan ditambahkan sebagai outlet rekanan. Obat bebas dijual oleh apotek dan klinik rekanan yang sudah berizin; barang milik rekanan. Mengubah brief PS2 (apotek bukan outlet) dan memperluas PS2a. Peran apoteker (PS3) tetap ditunda.
Alasan: keputusan user setelah riset PSEF (SRC-08) dan BPOM (SRC-10, SRC-11).
Dampak ke scope / waktu / arsitektur: portal admin outlet juga dipakai apotek. Kewajiban PSEF bagi platform menunggu L-03; tindak lanjut bila wajib di PT-21.
Keputusan: Diterima user (T2-Q3, T2-Q3b), berlaku sejak BRD v1.0

### CR-003: Pembagian dana langsung oleh payment gateway

Diajukan: 2026-11-17 (`docs/02-catatan-brd.md` Q4)
Deskripsi: Pembayaran konsultasi dan produk dibagi langsung oleh payment gateway berizin BI ke rekening rekanan dan rekening PT. Arunika. PT. Arunika tidak menampung dan tidak meneruskan dana rekanan sendiri. Mengubah brief PS4; "penerusan dana" di P8.2a dibaca sebagai pemantauan, laporan, dan rekonsiliasi.
Alasan: riset (SRC-19, SRC-20) menemukan larangan aktivitas sistem pembayaran tanpa izin BI dan pengaturan penerusan pembayaran dari PJP ke penyedia barang/jasa.
Dampak ke scope / waktu / arsitektur: pemilihan payment gateway di Tahap 4–5 harus mendukung pembagian dana. Konfirmasi legal L-04 sebelum Tahap 4.
Keputusan: Diterima user (T2-Q4), berlaku sejak BRD v1.0

### CR-004: Pendapatan transaksi berupa markup, menggantikan komisi potongan

Diajukan: 2026-11-17 (`docs/02-catatan-brd.md` Q6, Q6a)
Deskripsi: Harga konsultasi dan produk diambil dari rekanan, lalu Rumah Sehat menambahkan markup berbentuk persen atau nominal yang dapat diatur per rekanan oleh admin Rumah Sehat. Model komisi potongan di Business Case §5.1 diganti. Biaya dari rekanan tetap.
Alasan: keputusan user.
Dampak ke scope / waktu / arsitektur: Business Case §5.1–5.2 diperbarui istilahnya (komisi → markup) setelah gate Tahap 2. Pengaturan markup per rekanan masuk backoffice (BR-063). Pencatatannya di audit trail (BR-080) adalah usulan penyusun berdasarkan ADR-005, disetujui user di gate Tahap 2.
Keputusan: Diterima user (T2-Q6, T2-Q6a), berlaku sejak BRD v1.0

### CR-005: Semua layanan yang dipesan di aplikasi dibayar di aplikasi

Diajukan: 2026-11-17 (setelah gate Tahap 2, `docs/02-catatan-brd.md` Q13–Q20)
Deskripsi: Setiap layanan yang dimulai atau didaftarkan di aplikasi dibayar di aplikasi, termasuk pendaftaran klinik, pendaftaran lab, dan home visit. Pasien yang datang ke klinik atau lab cukup menunjukkan appointment di aplikasi mobile. Membalik BR-024 dan butir non-scope "Pembayaran klinik, lab, dan home visit di aplikasi" di BRD v1.0 §4.2; memperluas BR-040, BR-041, P-04, P-05.
Alasan: keputusan user.
Dampak ke scope / waktu / arsitektur: pembagian dana langsung (CR-003) dan aturan refund (BR-045, PT-27) berlaku juga untuk klinik, lab, dan home visit. Cakupan konfirmasi legal L-04, L-08, L-10, L-12, L-13, dan L-14 meluas; PT-28 (pajak) ditambahkan. Sistem rekanan perlu menerima status lunas dan mencocokkan appointment saat pasien datang (PT-05). Business Case §5 perlu memuat markup klinik, lab, dan home visit saat diperbarui.
Keputusan: Diterima user (T2-Q13 sampai T2-Q20): tindakan tambahan yang tidak tercantum di aplikasi dibayar di luar aplikasi; markup dan aturan biaya transaksi T2-Q12 berlaku untuk semua layanan; biaya rekanan dipakai atau tidak diputuskan CEO (PT-17); pembatalan layanan terjadwal sebelum batas waktu di-refund; informasi biaya MCU/lab menampilkan harga bayar di aplikasi. Berlaku sejak BRD v1.1 (disetujui user di gate revisi 2026-11-17, T2-Gate-CR005)

## Tech Debt

_Belum ada._
