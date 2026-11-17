# Catatan Wawancara Tahap 2 (BRD)

Catatan apa adanya. File ini menjadi bukti untuk setiap label `[USER] T2-Qx` di `docs/02-brd.md`.

Tanggal: 2026-11-17
Pewawancara: Claude (persona Business Analyst + Compliance Officer)
Narasumber: user (project manager; persetujuan diteruskan ke CEO dan CTO, lihat brief "Stakeholder dan sign-off")
Cara bertanya: satu pertanyaan per giliran, sesuai arahan user di Tahap 0.

**Q1. Apakah CEO sudah menetapkan besaran komisi konsultasi, komisi produk, dan biaya rekanan (PT-17)?** (pilihan: belum, buat bisa diatur admin, angka diisi CEO sebelum go-live; sudah ada angka; belum, tunggu CEO)
Jawaban: "Belum, bisa diatur admin".
Catatan pewawancara: PT-17 tetap terbuka (angka oleh CEO sebelum go-live), tidak memblokir PRD. BRD mewajibkan besaran komisi dan biaya rekanan dapat diatur admin Rumah Sehat.

**Ringkasan riset kepatuhan disampaikan ke user** sebelum Q2: wajib daftar PSE Lingkup Privat; UU PDP (data spesifik, persetujuan eksplisit, 3x24 jam, DPIA, petugas PDP); tiga isu yang berpotensi mengubah model bisnis (telemedisin oleh fasyankes, syarat PSEF untuk obat, izin BI untuk penerusan dana). Disampaikan bahwa sebagian temuan berstatus indikasi, wajib dikonfirmasi legal, dan bukan nasihat hukum. Sumber: `docs/compliance/tahap-02-riset.md`.

**Q2. Bagaimana struktur penyelenggara konsultasi online dokter dan psikolog?** (konteks: keputusan lama PS6e = dokter/psikolog berasal dari mitra langsung Rumah Sehat dan tenaga klinik rekanan; riset K-1. Pilihan: A. PT. Arunika hanya platform/PSE, semua dokter/psikolog berpraktik atas nama klinik rekanan termasuk yang tadinya mitra langsung, PS6e berubah, dikonfirmasi legal sebelum Tahap 4; B. PT. Arunika/afiliasi memiliki klinik untuk menaungi mitra langsung, PS6e tetap, tambah izin/biaya/waktu; C. tunggu pendapat legal, layanan 1 dan 2 tertahan di PRD. Pertimbangan pewawancara yang disampaikan, bukan keputusan: A paling sejalan dengan indikasi regulasi dan tidak menambah izin baru.)
Jawaban: "A. PSE + fasyankes rekanan".
Catatan pewawancara: keputusan lama PS6e berubah. Semua dokter dan psikolog berpraktik atas nama fasyankes rekanan; tidak ada lagi dokter/psikolog "mitra langsung" tanpa fasyankes. Keputusan ini dikonfirmasi legal sebelum Tahap 4.

**Q3. Bagaimana nasib penjualan obat bebas di rilis pertama?** (konteks: keputusan lama PS1/PS2a = obat bebas dijual klinik rekanan; PS3 = tanpa apoteker; apotek bukan outlet rekanan; riset K-2: syarat PSEF mencakup apoteker penanggung jawab dan jaringan apotek; peraturan BPOM peredaran daring juga mencakup kosmetik dan suplemen. Pilihan: A tetap jual dan penuhi skema legal; B keluarkan obat bebas, kecantikan dan kesehatan tetap; C tunda layanan 7; D tunggu legal.)
Jawaban (teks bebas): "arunika hanya platform, semua barang milik rekanan yang mana sudah mempunyai izin".
Catatan pewawancara: obat bebas tetap di rilis 1; PT. Arunika berposisi sebagai platform; barang milik rekanan berizin. Pewawancara menyampaikan bahwa syarat PSEF ditujukan ke platform, sehingga status "hanya platform" belum tentu membebaskan Rumah Sehat dari PSEF; diklarifikasi di Q3a.

**Q3a. Obat bebas tetap di rilis 1 (rekanan wajib berizin). Kalau legal menyatakan Rumah Sehat sebagai platform tetap wajib PSEF (apoteker + jaringan apotek), apa yang dilakukan?** (pilihan: keluarkan obat bebas lewat P8.3; penuhi PSEF, PS2 dan PS3 berubah; putuskan saat itu oleh CEO)
Jawaban (teks bebas): "obat bebas pun milik apotik rekanan".
Catatan pewawancara: jawaban mengubah keputusan lama PS2 (apotek kini termasuk outlet rekanan) dan PS2a (penjual obat bebas). Pilihan kontinjensi bila legal menyatakan wajib PSEF tidak dipilih user; dicatat sebagai pertanyaan terbuka untuk CEO. Pewawancara menyampaikan bahwa dengan apotek rekanan, syarat kerja sama dan jaringan apotek lebih mungkin terpenuhi, sedangkan apoteker penanggung jawab di sisi platform tetap perlu konfirmasi legal. Penjual obat bebas diklarifikasi di Q3b.

**Q3b. Di rilis 1, siapa saja yang menjual obat bebas lewat Rumah Sehat?** (pilihan: hanya apotek rekanan, PS2a diganti; apotek dan klinik rekanan, PS2 diperluas dan PS2a tetap)
Jawaban (teks bebas): "Apotel dan Klinik rekanan yang sudah mempunyai ijin".
Catatan pewawancara: obat bebas dijual oleh apotek rekanan dan klinik rekanan yang sudah berizin. Outlet rekanan menjadi: toko kecantikan, toko alat kesehatan, klinik rekanan, dan apotek rekanan. Keputusan lama PS2 diperluas; PS2a tetap dengan tambahan apotek. Peran apoteker (PS3) tidak diubah user: tetap ditunda.

**Q4. Bagaimana model aliran uang untuk konsultasi online dan pembelian produk yang dibayar di aplikasi?** (konteks: keputusan lama PS4 = pasien bayar di aplikasi, Rumah Sehat meneruskan dana ke outlet; P8.2a = backoffice "laporan & penerusan dana"; riset K-3: larangan aktivitas sistem pembayaran tanpa izin BI dan pengaturan penerusan pembayaran dari PJP ke penyedia barang/jasa. Pilihan: A. payment gateway berizin BI menyalurkan bagian mitra langsung ke rekening klinik/outlet, komisi dipotong di sana, dana tidak ditampung PT. Arunika, backoffice memantau dan merekonsiliasi; B. dana ditampung di rekening PT. Arunika lalu diteruskan, potensi izin/penetapan BI; C. tunggu legal. Pertimbangan pewawancara yang disampaikan, bukan keputusan: A menghindari risiko izin BI; ketersediaan fitur pembagian dana di penyedia diverifikasi di Tahap 4–5.)
Jawaban: "A. Dibagi langsung gateway".
Catatan pewawancara: keputusan lama PS4 berubah: Rumah Sehat tidak menampung dan tidak meneruskan dana sendiri. "Penerusan dana" di P8.2a dibaca sebagai pemantauan, laporan, dan rekonsiliasi pembagian dana oleh payment gateway. PT-06 (mekanisme dan jadwal penerusan dana) terjawab sebagian: mekanismenya pembagian langsung oleh payment gateway; jadwal pencairan mengikuti penyedia yang dipilih di Tahap 4–5.

**Q5. Apa tugas fisioterapis dan tenaga medis di Rumah Sehat pada rilis 1 (PT-07)?** (konteks: keduanya peran aplikasi mobile nakes di Tahap 0 tanpa rincian tugas; home visit berasal dari sistem rekanan lewat integrasi (PS6a); dari Q2 semua nakes berpraktik atas nama klinik rekanan. Pilihan: home visit saja (tugas dari klinik rekanan via integrasi, di aplikasi melihat jadwal dan memperbarui status berangkat/tiba/selesai); konsultasi online (chat/video) + home visit; tanpa akun Rumah Sehat, kedua peran keluar dari rilis 1.)
Jawaban: "Konsultasi + home visit".
Catatan pewawancara: PT-07 terjawab. Fisioterapis dan tenaga medis melayani konsultasi online (chat dan video) dan home visit. Asal mereka mengikuti Q2: berpraktik atas nama klinik rekanan. Rincian alur home visit di aplikasi nakes dibahas di PRD.

**Q6. Siapa yang menetapkan tarif konsultasi online (dokter, psikolog, fisioterapis, tenaga medis)?** (konteks: konsultasi berlangsung di Rumah Sehat (PS6b); dari Q2 nakes di bawah klinik rekanan; dari Q4 bagian klinik dibagi langsung oleh payment gateway; Kemenkes menyatakan akan mengatur platform termasuk tarif [SRC-06]. Pilihan: klinik rekanan; Rumah Sehat (tarif standar per jenis nakes); klinik dalam batas minimum-maksimum Rumah Sehat.)
Jawaban (teks bebas): "Tarif ini semuakita ambil dari rekanan, Rumah Sehat melakukan semacam markup dalam bentuk persdn atau nominal dan itu bisa disetting per rekanan di admin rumah sehat".
Catatan pewawancara: tarif berasal dari rekanan; Rumah Sehat menambahkan markup berbentuk persen atau nominal yang dapat diatur per rekanan oleh admin Rumah Sehat. Ini berbeda dari Business Case §5.1 ("komisi konsultasi" = potongan dari biaya yang dibayar pasien); disampaikan ke user bahwa perubahan ini masuk tinjauan ulang Business Case di gate Tahap 2. Cakupan markup diklarifikasi di Q6a. Cara tarif diambil dari rekanan (integrasi atau diisi di portal) dibahas di PRD/arsitektur.

**Q6a. Model markup ini berlaku untuk apa saja, dan bagaimana kaitannya dengan "komisi produk" di Business Case?** (pilihan: konsultasi dan produk, model potongan/komisi tidak dipakai; konsultasi saja, produk tetap komisi potongan; markup dan komisi, admin bisa memakai salah satu atau keduanya per rekanan)
Jawaban: "Konsultasi & produk, ganti".
Catatan pewawancara: pendapatan dari transaksi di aplikasi berupa markup (persen atau nominal, per rekanan) di atas harga rekanan untuk konsultasi dan produk. Model komisi potongan (Business Case §5.1 baris komisi konsultasi dan komisi produk) diganti. Biaya dari rekanan (langganan/biaya per pendaftaran) tidak disinggung dan tetap berlaku sesuai Q1. Konsekuensi: harga yang dilihat pasien = harga rekanan + markup; rekanan menerima harga rekanan.

**Q7. Siapa yang menanggung ongkos kirim produk lewat kurir pihak ketiga?** (konteks: Business Case §6.1 mencatat penanggung ongkir belum diketahui; kurir pihak ketiga terintegrasi (PS5). Pilihan: pasien, ditambahkan ke total pembayaran; outlet rekanan; diatur per outlet.)
Jawaban: "Pasien".

**Q8. Siapa yang menanggung biaya transaksi payment gateway?** (konteks: dari Q4 gateway membagi dana langsung ke mitra; dari Q6a pendapatan = markup; Business Case §6.1 mencatat biaya per transaksi mengurangi pendapatan; besarnya diketahui setelah penyedia dipilih di Tahap 4–5. Pilihan: Rumah Sehat, dipotong dari markup; pasien, ditampilkan sebagai biaya layanan/transaksi; rekanan; diatur admin.)
Jawaban: "Pasien".
Catatan pewawancara: biaya transaksi payment gateway ditampilkan dan dibebankan ke pasien di total pembayaran.

**Q9. Bagaimana aturan pembatalan dan refund untuk transaksi yang dibayar di aplikasi (konsultasi dan produk)?** (konteks: audit trail P5.4 butir 5 mencakup refund; dari Q4 dana mitra langsung ke rekening mitra; rincian angka di PRD. Pilihan: kebijakan seragam Rumah Sehat; ikut kebijakan rekanan; hanya kalau layanan gagal = refund bila nakes tidak hadir atau barang tidak dikirim/rusak, pembatalan oleh pasien setelah membayar tidak di-refund.)
Jawaban: "Hanya kalau layanan gagal".
Catatan pewawancara: refund hanya untuk layanan yang gagal terjadi (contoh yang disampaikan: nakes tidak hadir, barang tidak dikirim atau rusak). Pembatalan oleh pasien setelah membayar tidak di-refund. Daftar lengkap kondisi "layanan gagal", siapa yang memutuskan, dan batas waktunya disusun di PRD dan disetujui user.

**Q10. Apa kriteria "pasien aktif" untuk KPI-01 (10.000 pasien aktif bulanan pada 1 Maret 2028)?** (konteks: T1-Q3 = bulanan; kriteria "memakai" terbuka di PT-16, tenggat sebelum Tahap 3. Pilihan: login/membuka aplikasi dalam keadaan login minimal sekali per bulan kalender; bertransaksi = menyelesaikan minimal satu layanan (konsultasi, pendaftaran klinik/lab, home visit, pembelian produk) per bulan; keduanya dilaporkan, KPI resmi = bertransaksi, angka login sebagai pendukung.)
Jawaban: "Keduanya dilaporkan".
Catatan pewawancara: KPI-01 resmi = pasien unik yang menyelesaikan minimal satu layanan dalam satu bulan kalender. Pasien unik yang membuka aplikasi dalam keadaan login minimal sekali per bulan dilaporkan sebagai angka pendukung.

**Q11. Status apa yang dihitung untuk KPI-02 (70 dokter dan psikolog), KPI-03 (20 klinik), dan KPI-04 (10 lab)?** (konteks: bagian kedua PT-16; syarat go-live P8.2 memakai "terverifikasi" untuk dokter/psikolog dan "terintegrasi" untuk klinik/lab/outlet. Pilihan: sama dengan syarat go-live; terdaftar saja; aktif melayani minimal satu transaksi per bulan pengukuran.)
Jawaban: "Sama syarat go-live".
Catatan pewawancara: KPI-02 menghitung dokter dan psikolog yang terverifikasi dan aktif; KPI-03 dan KPI-04 menghitung klinik dan lab yang terintegrasi. PT-16 terjawab.

## Akhir wawancara

Wawancara Tahap 2 (Q1–Q11) selesai 2026-11-17. BRD disusun di `docs/02-brd.md`.

## Putaran tambahan setelah review kepatuhan

**Konteks.** Review kepatuhan BRD v0.1 (`docs/compliance/tahap-02.md`, blokir B-1) menemukan halaman resmi BI [SRC-24]: biaya MDR QRIS "ditanggung oleh merchant dan tidak boleh dibebankan kepada konsumen" (berlaku 1 Oktober 2026); untuk kartu, virtual account, dan e-wallet [TIDAK DIKETAHUI]. Ini bertentangan dengan jawaban Q8 untuk pembayaran QRIS.

**Q12. Siapa yang menanggung biaya transaksi payment gateway, setelah ada aturan BI bahwa MDR QRIS tidak boleh dibebankan ke konsumen?** (konteks: jawaban lama Q8 = pasien; dari Q6a markup dapat diatur admin. Pilihan: A. semua biaya PG ditanggung Rumah Sehat dari markup, Q8 berubah; B. MDR QRIS dari markup Rumah Sehat, metode lain ke pasien sambil menunggu konfirmasi legal per metode; C. MDR QRIS dipotong dari bagian rekanan, metode lain ke pasien sambil menunggu konfirmasi legal. Pertimbangan pewawancara yang disampaikan, bukan keputusan: A paling sederhana dan aman dari sisi aturan.)
Jawaban: "C. QRIS rekanan, lain pasien".
Catatan pewawancara: jawaban Q8 dikoreksi. MDR QRIS ditanggung rekanan (dipotong dari bagian rekanan). Biaya transaksi metode pembayaran lain dibebankan ke pasien, menunggu konfirmasi legal per metode (L-12).

## Catatan revisi

- 2026-11-17 (setelah cek ulang fact-check): pada konteks Q12, tanggal "berlaku 1 Oktober 2026" yang disampaikan ke user di halaman SRC-24 merujuk ke skema tarif MDR QRIS, bukan khusus ke kalimat "ditanggung oleh merchant dan tidak boleh dibebankan kepada konsumen". Ketentuan pokok bahwa MDR QRIS tidak boleh dibebankan ke konsumen tidak berubah, sehingga jawaban user tidak terdampak.

## Gate Tahap 2

2026-11-17. Ditampilkan: rekap jawaban Q1–Q12, butir susunan penyusun yang perlu persetujuan (BR-080 pencatatan perubahan markup/biaya rekanan di audit trail; catatan PT-06 honor nakes menjadi urusan klinik rekanan; pemilik/tenggat PT-20–PT-27 dan tenggat baru PT-19 = gate Tahap 4), checklist gate, serta catatan 14 butir konfirmasi legal dan tinjauan ulang Business Case. Pertanyaan: apakah user menyetujui BRD termasuk scope/non-scope §4 dan butir-butir tersebut?
Jawaban (T2-Gate): "Setujui".
