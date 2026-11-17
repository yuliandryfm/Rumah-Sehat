# Traceability Matrix

Menghubungkan setiap kebutuhan bisnis sampai ke kode dan test-nya. Fitur atau story tanpa induk BR adalah tanda ada yang dikarang dan harus dihapus atau dikonfirmasi user.

Diisi bertahap: kolom BR di Tahap 2, story di Tahap 3, test case di Tahap 7, file kode/test di Tahap 8.

| BR | Kebutuhan bisnis | Sumber BR | Story | Test case | File kode / test | Status |
| --- | --- | --- | --- | --- | --- | --- |
| BR-001 | Pasien dapat mendaftar dan masuk dengan email + password atau akun Google/Apple | `[DOK] P4.4` | | | | BRD v1.0 |
| BR-002 | Rumah Sehat menyimpan data identitas dasar pasien (nama, tanggal lahir, jenis kelamin, nomor HP, email,… | `[DOK] P4.1` | | | | BRD v1.0 |
| BR-003 | NIK dan foto KTP pasien wajib sebelum memakai konsultasi online, home visit, serta pendaftaran klinik dan… | `[DOK] P4.6a` | | | | BRD v1.0 |
| BR-004 | Admin Rumah Sehat memverifikasi NIK dan foto KTP secara manual (disetujui atau ditolak) sebelum layanan… | `[DOK] P4.6b` | | | | BRD v1.0 |
| BR-005 | Akun dokter, psikolog, fisioterapis, dan tenaga medis diverifikasi dan dikelola admin Rumah Sehat. Setiap… | `[DOK] P8.2a` `[USER] T2-Q2` | | | | BRD v1.0 |
| BR-006 | Akun rekanan (klinik, lab, toko kecantikan, toko alat kesehatan, apotek) dan admin mitranya diverifikasi… | `[DOK] P8.2a` `[USER] T2-Q3, T2-Q3b` | | | | BRD v1.0 |
| BR-007 | Aplikasi pasien tersedia di Android dan iOS; tenaga kesehatan memakai aplikasi mobile; para admin memakai… | `[DOK] P6.1, P6.2` | | | | BRD v1.0 |
| BR-010 | Pasien dapat berkonsultasi dengan dokter lewat chat teks dan panggilan video | `[DOK] P3.1, P3.2` | | | | BRD v1.0 |
| BR-011 | Pasien dapat berkonsultasi kesehatan mental dengan psikolog lewat chat teks dan panggilan video | `[DOK] P2.1, P3.1, P3.2` | | | | BRD v1.0 |
| BR-012 | Pasien dapat berkonsultasi dengan fisioterapis dan tenaga medis lewat chat teks dan panggilan video | `[USER] T2-Q5` | | | | BRD v1.0 |
| BR-013 | Konsultasi berlangsung di Rumah Sehat. Tenaga kesehatan memakai aplikasi tenaga kesehatan untuk chat,… | `[DOK] PS6b` | | | | BRD v1.0 |
| BR-014 | Setiap konsultasi diberikan oleh tenaga kesehatan atas nama klinik rekanan yang menaunginya; PT. Arunika… | `[USER] T2-Q2` | | | | BRD v1.0 |
| BR-015 | Dokter dapat menerbitkan resep digital setelah konsultasi. Pasien menebusnya sendiri di apotek mana pun;… | `[DOK] P3.4, PS1` | | | | BRD v1.0 |
| BR-016 | Isi chat konsultasi (termasuk foto), catatan tenaga kesehatan, dan resep disimpan. Panggilan video tidak… | `[DOK] P4.1` | | | | BRD v1.0 |
| BR-017 | Aplikasi tersedia 24 jam, 7 hari. Ketersediaan tenaga kesehatan mengikuti jadwal masing-masing. Target… | `[DOK] P6.4` | | | | BRD v1.0 |
| BR-018 | Tarif konsultasi berasal dari rekanan | `[USER] T2-Q6` | | | | BRD v1.0 |
| BR-020 | Pasien dapat mendaftar ke klinik rekanan berdasarkan jadwal dan data klinik dari sistem rekanan | `[DOK] P3.1, PS6a` | | | | BRD v1.0 |
| BR-021 | Pasien dapat mendaftar pemeriksaan di lab rekanan berdasarkan jadwal dan harga dari sistem rekanan | `[DOK] P3.1, PS6a` | | | | BRD v1.0 |
| BR-022 | Pasien dapat memesan home visit yang disediakan klinik rekanan, dilayani tenaga kesehatan klinik itu… | `[DOK] P1.2, PS6a` `[USER] T2-Q5` | | | | BRD v1.0 |
| BR-023 | Pasien dapat melihat informasi biaya MCU dan pemeriksaan lab dari sistem rekanan sebelum mendaftar | `[DOK] P1.2, PS6a` | | | | BRD v1.0 |
| BR-024 | Pembayaran pendaftaran klinik, lab, dan home visit dilakukan langsung ke mitra, tidak lewat aplikasi | `[DOK] P3.3` | | | | BRD v1.0 |
| BR-025 | Hasil lab dari lab rekanan diterima lewat integrasi, disimpan, dan diberitahukan ke pasien | `[DOK] P4.1, P4.5, PS6a` | | | | BRD v1.0 |
| BR-026 | Riwayat pendaftaran klinik, lab, dan home visit pasien disimpan | `[DOK] P4.1` | | | | BRD v1.0 |
| BR-030 | Pasien dapat membeli obat bebas, produk kecantikan, dan produk kesehatan lain (vitamin, suplemen, alat… | `[DOK] PS1` | | | | BRD v1.0 |
| BR-031 | Outlet rekanan terdiri atas toko kecantikan, toko alat kesehatan, klinik rekanan, dan apotek rekanan. Obat… | `[DOK] PS2` `[USER] T2-Q3, T2-Q3b` | | | | BRD v1.0 |
| BR-032 | Obat resep tidak dijual di Rumah Sehat | `[DOK] PS1` | | | | BRD v1.0 |
| BR-033 | Katalog, stok, dan pesanan produk berasal dari sistem outlet rekanan lewat integrasi. Rumah Sehat tidak… | `[DOK] PS6, PS6a` | | | | BRD v1.0 |
| BR-034 | Produk dikirim kurir pihak ketiga yang terintegrasi. Ongkos kirim dibayar pasien dan ditambahkan ke total… | `[DOK] PS5` `[USER] T2-Q7` | | | | BRD v1.0 |
| BR-035 | Riwayat pesanan produk pasien disimpan | `[DOK] P4.1` | | | | BRD v1.0 |
| BR-040 | Konsultasi online dan pembelian produk dibayar di aplikasi melalui payment gateway berizin Bank Indonesia | `[DOK] P3.3, PS4` `[USER] T2-Q4` `[SRC-19]` | | | | BRD v1.0 |
| BR-041 | Payment gateway membagi dana setiap pembayaran langsung: bagian rekanan ke rekening rekanan, bagian Rumah… | `[USER] T2-Q4` | | | | BRD v1.0 |
| BR-042 | Harga yang dibayar pasien untuk konsultasi dan produk = harga dari rekanan + markup Rumah Sehat. Markup… | `[USER] T2-Q1, T2-Q6, T2-Q6a` | | | | BRD v1.0 |
| BR-043 | Biaya dari rekanan (langganan atau biaya per pendaftaran yang ditagihkan ke klinik dan lab) dapat diatur… | `[DOK: 01-business-case.md]` `[USER] T1-Q1, T2-Q1` | | | | BRD v1.0 |
| BR-044 | Biaya MDR untuk pembayaran QRIS ditanggung rekanan (dipotong dari bagian rekanan) dan tidak dibebankan ke… | `[USER] T2-Q8, T2-Q12` `[SRC-24]` | | | | BRD v1.0 |
| BR-045 | Refund hanya diberikan bila layanan gagal terjadi (contoh: tenaga kesehatan tidak hadir, barang tidak… | `[USER] T2-Q9` | | | | BRD v1.0 |
| BR-046 | Admin Rumah Sehat, admin klinik, admin lab, dan admin outlet dapat melihat laporan transaksi dan pembagian… | `[DOK] P8.2a, PS6c, PS6d` `[USER] T2-Q4` | | | | BRD v1.0 |
| BR-050 | Rumah Sehat mengirim pemberitahuan ke pasien (konfirmasi, pengingat, status pesanan, hasil lab) lewat… | `[DOK] P4.5` | | | | BRD v1.0 |
| BR-060 | Backoffice admin Rumah Sehat menyediakan verifikasi dan pengelolaan akun (pasien, tenaga kesehatan, rekanan) | `[DOK] P8.2a` | | | | BRD v1.0 |
| BR-061 | Backoffice admin Rumah Sehat menyediakan CMS untuk banner, halaman info, artikel/konten edukasi kesehatan,… | `[DOK] P8.2a` | | | | BRD v1.0 |
| BR-062 | Backoffice admin Rumah Sehat menyediakan layanan pelanggan, termasuk penanganan keluhan | `[DOK] P8.2a` | | | | BRD v1.0 |
| BR-063 | Backoffice admin Rumah Sehat menyediakan pengaturan markup per rekanan dan biaya rekanan | `[USER] T2-Q1, T2-Q6` | | | | BRD v1.0 |
| BR-064 | Portal admin klinik dan admin lab menyediakan laporan transaksi dan pembagian dana, profil, dan keluhan | `[DOK] PS6c` | | | | BRD v1.0 |
| BR-065 | Portal admin outlet (toko kecantikan, toko alat kesehatan, apotek) menyediakan laporan, pembagian dana,… | `[DOK] PS6d` `[USER] T2-Q3b` | | | | BRD v1.0 |
| BR-070 | Rumah Sehat terintegrasi otomatis dengan sistem klinik, lab, dan outlet rekanan untuk jadwal, harga dan… | `[DOK] PS6a, P4.3` `[USER] T2-Q6` | | | | BRD v1.0 |
| BR-071 | Rumah Sehat terintegrasi dengan payment gateway (dengan pembagian dana), layanan panggilan video, kurir… | `[DOK] P4.2` `[USER] T2-Q4` | | | | BRD v1.0 |
| BR-080 | Audit trail append-only mencatat: (1) login, logout, dan percobaan login gagal; (2) akses ke data pasien… | `[DOK] P5.4` `[DOK: adr/ADR-005-audit-trail-append-only.md]` | | | | BRD v1.0 |
| BR-081 | Audit trail tidak boleh memuat password, PIN, OTP, token, atau nomor kartu/rekening lengkap | `[DOK: adr/ADR-005-audit-trail-append-only.md]` | | | | BRD v1.0 |
| BR-090 | Rumah Sehat menyediakan data untuk mengukur KPI-01 sampai KPI-04 sesuai definisi di bagian 1.2: MAU… | `[USER] T2-Q10, T2-Q11` `[DOK: 01-business-case.md]` | | | | BRD v1.0 |
| BR-100 | Rumah Sehat terdaftar sebagai Penyelenggara Sistem Elektronik Lingkup Privat sebelum aplikasi digunakan… | `[SRC-12]` `[SRC-13]` | | | | BRD v1.0 |
| BR-101 | Konsultasi online berjalan dalam skema telemedisin yang sesuai ketentuan Kemenkes: klinik rekanan… | `[SRC-02]` `[SRC-03]` `[USER] T2-Q2` | | | | BRD v1.0 |
| BR-102 | Hanya tenaga kesehatan dengan STR dan SIP yang masih berlaku yang dapat diaktifkan melayani. Masa berlaku… | `[SRC-02]` `[SRC-03]` `[SRC-21]` | | | | BRD v1.0 |
| BR-103 | Pasien memberikan persetujuan eksplisit, tertulis atau terekam, sebelum datanya diproses, dengan informasi… | `[SRC-15]` | | | | BRD v1.0 |
| BR-104 | Data anggota keluarga yang masih anak hanya diproses dengan persetujuan orang tua atau wali | `[SRC-15]` | | | | BRD v1.0 |
| BR-105 | Data kesehatan (catatan tenaga kesehatan, isi chat konsultasi, resep, hasil lab), data anak, dan data… | `[SRC-15]` | | | | BRD v1.0 |
| BR-106 | Pasien dapat menjalankan hak subjek data: informasi, akses dan salinan, perbaikan, penarikan persetujuan,… | `[SRC-15]` `[DOK] P5.4` | | | | BRD v1.0 |
| BR-107 | Seluruh kegiatan pemrosesan data pribadi direkam, selaras dengan audit trail BR-080 | `[SRC-15]` | | | | BRD v1.0 |
| BR-108 | Rumah Sehat menjaga keamanan dan kerahasiaan data pribadi yang diprosesnya | `[SRC-15]` `[SRC-13]` | | | | BRD v1.0 |
| BR-109 | Tersedia prosedur pemberitahuan kegagalan pelindungan data pribadi secara tertulis paling lambat 3 x 24… | `[SRC-15]` `[SRC-13]` | | | | BRD v1.0 |
| BR-110 | Penilaian dampak pelindungan data pribadi (DPIA) dilakukan sebelum go-live | `[SRC-15]` | | | | BRD v1.0 |
| BR-111 | Rumah Sehat menunjuk pejabat/petugas pelindungan data pribadi | `[SRC-15]` `[SRC-16]` | | | | BRD v1.0 |
| BR-112 | Setiap jenis data memiliki masa retensi; setelah masa retensi berakhir, data diakhiri pemrosesannya,… | `[SRC-15]` `[SRC-25]` | | | | BRD v1.0 |
| BR-113 | Data disimpan di server atau cloud yang berlokasi di Indonesia. Pengiriman data pribadi ke luar wilayah… | `[DOK] P5.5` `[SRC-15]` `[SRC-13]` | | | | BRD v1.0 |
| BR-114 | Catatan konsultasi dan resep digital memenuhi ketentuan rekam medis elektronik dan resep elektronik yang… | `[SRC-18]` `[SRC-06]` `[SRC-07]` `[USER] T2-Q2` | | | | BRD v1.0 |
| BR-115 | Pembayaran hanya diproses lewat penyedia jasa pembayaran berizin BI. Model pembagian dana langsung… | `[SRC-19]` `[SRC-20]` `[USER] T2-Q4` | | | | BRD v1.0 |
| BR-116 | Obat bebas, produk kecantikan, dan produk kesehatan yang dijual lewat Rumah Sehat memenuhi ketentuan… | `[SRC-08]` `[SRC-10]` `[SRC-11]` `[USER] T2-Q3` | | | | BRD v1.0 |
| BR-117 | PT. Arunika memiliki perizinan berusaha yang sesuai untuk kegiatan Rumah Sehat sebelum go-live. Jenis… | `[SRC-09]` `[SRC-23]` | | | | BRD v1.0 |
| BR-118 | Perkembangan regulasi kesehatan digital dipantau sampai go-live, karena Kemenkes menyatakan akan mengatur… | `[SRC-06]` | | | | BRD v1.0 |
