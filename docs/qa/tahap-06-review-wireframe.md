# Review QA Tahap 6 (Design Guideline dan Wireframe)

Tanggal: 2026-11-18 · Reviewer: QA independen · Tingkat: T2 · Dokumen yang direview tidak diubah.
Dasar: `docs/03-prd.md` v1.1, `docs/backlog.md` (CR-007, CR-009, CR-013, CR-015), kriteria blokir `aturan-kerja.md` §2.

## Verdict: LULUS DENGAN CATATAN

Tidak ada butir blokir. Lima catatan non-blokir di bawah, termasuk tiga celah spesifikasi PRD yang perlu keputusan di Tahap 7.

## 1. Gate "semua alur utama punya wireframe"

Pemeriksaan: semua `#### US-xxx` di PRD (grep) dibandingkan dengan nomor story di bagian "Story → wireframe (Tahap 6)" pada `traceability.md` (skrip mengurai rentang `US-a–US-b`).
Hasil: satu-satunya story PRD yang tidak muncul adalah US-102 (dihapus CR-007, sah). Semua story lain, termasuk Should (US-108, 109, 405, 602, 1305), terpetakan ke wireframe HTML, deskripsi D-xx, atau dibenarkan di §10.3.
Hitungan §10.1 cocok: 21 HTML (13 A + 3 B + 3 C + 1 D + 1 E = jumlah baris `index.html`), 18 deskripsi (A14, C04-C12, D02, E02-E06, F01, F02), total 39.
Referensi KEPUTUSAN-160-176 ada di `decision-log.md`; SRC-256-260 ada di `sources.md`. Nilai kontras K1-K12 dan N1 dihitung ulang dengan rumus WCAG di guideline dan cocok sampai 2 desimal.
Kesimpulan: gate terpenuhi. Kedalaman layar untuk beberapa story hanya lewat keterangan, lihat N3.

## 2. Pertentangan wireframe dengan PRD

Dicek dan sesuai: AB-16 (A07 layar 1, A10 layar 2), AB-13 (satu angka, tanpa baris markup), AB-15/US-801 AC5 (A07 layar 2), PB-01/PB-07 (30 menit)/PB-08/PB-03/PB-15/PB-05/PB-04/PB-18 (A05, A07, A10, A11, A12), US-104 AC1/AC5 (7 butir, kotak tidak tercentang, A01 layar 3), US-107 (A05 layar 4), US-108 AC1-AC6 (A04), AB-05 empat status (A03), RF-01/RF-02 dan nominal sebelum konfirmasi (A12), AB-32/AB-35 (A12 layar 4, guideline §5.6), US-1601/1602/1604 (A13), NFR-SEC-02/04 (C01), NFR-SEC-10 dan CR-009 biometrik/PIN perangkat 15 menit (B01 layar 4), ADR-011 butir 5 (TOTP 6 digit, 12 kode pemulihan, B01 layar 3). CR-007: tidak ada tombol Google/Apple, push, atau WhatsApp di semua HTML (grep; satu-satunya kemunculan adalah kalimat larangan). Ubin Vaksinasi/Infus tidak ada di A02.
Temuan kontradiksi yang mengubah perilaku PRD: tidak ada.

## 3. Layanan atau fitur di luar scope

- Vaksinasi/infus hanya muncul di `a08-klinik-lab.html` layar 2 sebagai butir katalog bertanda "Menunggu CR-015 / Belum dapat dipesan" (`aria-disabled`). Sesuai CR-015 ("hanya sebagai butir katalog lab bertanda menunggu"). Guideline D-E03 dan UP-63 konsisten.
- Rating, voucher, multi-bahasa, mode gelap: tidak ada.
- Perilaku yang tidak ada di PRD: lihat N1 (batal pesanan belum dibayar).

## 4. Komponen dasar, state, dan pola

`06-design-guideline.md` §4 mendefinisikan default, hover/ditekan, fokus, nonaktif, galat, memuat untuk tiap komponen (sel tidak berlaku diberi "—"); §5.1-§5.6 mendefinisikan memuat, kosong, galat, offline, waktu/sesi, dan status. Cakupan memadai untuk T2.
Selisih: lihat N4 (state yang didefinisikan tetapi tidak diperagakan di wireframe).

## Blokir

Tidak ada.

## Non-blokir (maks. 5)

N1. Celah spesifikasi: `a07-pembayaran.html` layar 3 menampilkan tombol "Batalkan pesanan" saat menunggu pembayaran. PRD hanya mengatur pembatalan pesanan Terkonfirmasi (US-805 AC1; AC4 menutup status lain) dan kedaluwarsa otomatis (US-304 AC4, US-802). Pembatalan oleh pasien sebelum bayar (pelepasan slot/stok, status akhir, audit E-08/E-09) belum diatur. Usul: tambahkan AC di US-802 atau US-805 (status "Dibatalkan pasien" sebelum bayar, slot dilepas, tanpa refund), atau hapus tombol dari wireframe. Cek: grep "Batalkan pesanan" di `a07-pembayaran.html`, bandingkan US-805 AC1-AC4.

N2. Celah spesifikasi (T1, autentikasi): `a01-registrasi-masuk.html` layar 5 menampilkan "Akun terkunci sementara" untuk akun yang terkunci. US-103 AC3 mewajibkan akun terkunci walaupun password benar, tetapi AC2/ADR-011 mewajibkan pesan umum. Pesan khusus terkunci membuka enumerasi akun. Usul rumusan AC: "pesan yang sama dengan AC2 ditampilkan untuk akun terkunci; pemberitahuan penguncian dikirim ke email pemilik akun" (keputusan keamanan, bawa ke reviewer keamanan Tahap 7). Cek: layar 5 A01 dan US-103 AC2/AC3.

N3. Story dengan layar hanya lewat keterangan, tanpa tata letak atau deskripsi D-xx sendiri: US-105 AC2 dan AC4 (formulir alamat, ubah email; A03 hanya menu "Daftar alamat" dan catatan), US-203 (hanya "pola" di keterangan C02), US-811 AC2 (tab "Rekonsiliasi" di C03 tanpa isi), US-402 (layar penerima layanan klinik/lab hanya catatan di A08). Cukup tambah entri D-xx singkat; tidak menghalangi gate.

N4. State: `is-loading` (tombol memuat) dan hover/ditekan untuk isian, chip, dan tab didefinisikan di §4 tetapi tidak ada di `wireframe.css` maupun diperagakan di HTML manapun (grep `is-loading` di `*.html` nol hasil). Pola kerangka, galat jaringan, dan kosong hanya diperagakan di A02. Usul: satu layar "galeri state" di Tahap 8 atau catatan eksplisit bahwa §4 yang normatif.

N5. Konsistensi kecil: (a) baris pajak CR-013 tampil di A07/A11 dan US-801 AC1 PRD belum memuat pajak (penerapan ke PRD dijadwalkan Tahap 7; pastikan terjadi sebelum EP-08); (b) `c02-bo-verifikasi.html` menampilkan jam antrean tanpa label zona, padahal AB-42 menetapkan WIB untuk backoffice (C03 sudah menyebut WIB).
