# Catatan Intake (Tahap 0)

Catatan wawancara apa adanya. File ini menjadi bukti untuk setiap label `[USER]` di `docs/00-project-brief.md`.

Tanggal mulai: 2026-11-17
Pewawancara: Claude (persona Business Analyst + PM)
Narasumber: user (Project Board: Executive, Senior User, Senior Supplier)

## Putaran 0: Jalur intake

**Q1. Jalur intake mana yang dipakai untuk menyusun Project Brief Rumah Sehat?**
Jawaban: B: Wawancara.

## Arahan cara kerja dari user

- 2026-11-17: "berikan pertanyaan satu persatu saja, sehingga mudah menjawabnya". Wawancara dilakukan satu pertanyaan per giliran, menggantikan aturan runbook "3–5 pertanyaan per putaran". Rangkuman dan konfirmasi tetap dilakukan di akhir setiap kelompok tema.

## Putaran 1: Masalah dan tujuan

**P1.1. Apa itu "Rumah Sehat"? Jelaskan dalam satu atau dua kalimat aplikasinya untuk apa.**
Jawaban: "lifestyle Mobile Apps yang untuk kesehatan, seperti halnya halodoc. ada konsultasi online, pendaftaran ke klinik rekanan, pendaftaran lab, pembelian obat dan produk kesehatan"

**P1.2. Masalah apa yang ingin diselesaikan Rumah Sehat, dan siapa yang mengalaminya?**
Jawaban: "Pasien yang mempunyai kendala untuk datang ke klinik atau apotik, sehhingg butuhh konsultasi online atau bahkan home visit dari tenaga medis. Memperluas pula pasar klinik dan lab rekanan kepada semua pengguna aplikasi rumah sehat. mencari pengetahuan perihal jumla biaya yg dikeluarkan apabila akan melakukan MCU atau pemeriksaaan lab"
Catatan pewawancara: "home visit dari tenaga medis" dan "informasi biaya MCU/lab" belum disebut di P1.1; dikonfirmasi di rangkuman Putaran 1.

**P1.3. Bagaimana pasien, klinik, dan lab mengatasi masalah ini sekarang, tanpa Rumah Sehat?**
Jawaban: "tidak pasti, tetapi memang ada aplikasi sejenis yaitu halodoc, terkadang menghubungi langsung via telp atau WhatsApp"
Catatan pewawancara: kondisi saat ini sebagian [TIDAK DIKETAHUI]; sistem internal klinik/lab rekanan belum diketahui (ditanyakan lagi di Putaran 4 Data dan integrasi).

**P1.4. Apa yang terjadi kalau Rumah Sehat tidak dibuat?**
Jawaban: "pasien kesulitan mendapat layanan atau informasi perihal kesehatan, klinik rekanan sulit berekspansi. belum ada one stop solution untuk para pasien yg mobilitasnya terbatas"

**P1.5. Apa tujuan terukur Rumah Sehat? Kalau bisa, sebutkan dalam angka.**
Jawaban: "belum ada angka pasti"
Catatan pewawancara: tujuan terukur [TIDAK DIKETAHUI], masuk pertanyaan terbuka.

**Konfirmasi rangkuman Putaran 1.** Rangkuman ditampilkan (6 layanan: konsultasi online, pendaftaran klinik rekanan, pendaftaran lab, pembelian obat dan produk kesehatan, home visit tenaga medis, informasi biaya MCU/lab; masalah; kondisi sekarang; dampak; tujuan terukur belum ada angka).
Jawaban: "Sudah benar semua".

## Putaran 2: Pengguna dan stakeholder

**P2.1. Siapa saja yang akan memakai aplikasi Rumah Sehat, dan peran apa saja yang ada?**
(Pewawancara memberi contoh peran umum sebagai bahan pertimbangan, bukan asumsi: pasien, dokter/tenaga medis, admin klinik rekanan, admin lab rekanan, apoteker/admin apotek, admin internal Rumah Sehat.)
Jawaban: "pasien,dokter, tenaga medis, admin klinik, admin lab, apoteker, admin rumah sehat sendiri, fisioterapis, bahkan psikolog juga karena ada layanan konsultan perihal kesehatan mental."
Catatan pewawancara: "layanan konsultasi kesehatan mental" adalah layanan yang baru disebut; dikonfirmasi di rangkuman Putaran 2.

**P2.2. Siapa pemilik dan pembiaya proyek Rumah Sehat?**
Jawaban: "biaya sendiri dan berdiri sendiri"
Catatan pewawancara: nama badan usaha pemilik tidak disebut, dicatat [TIDAK DIKETAHUI].

**P2.3. Siapa yang berwenang menyetujui hasil proyek ini (sign-off) di setiap tahap dan saat go-live?**
Jawaban: "CEO, CTO da konsultan"

**P2.3a (klarifikasi). Konsultan yang ikut menyetujui itu konsultan di bidang apa?**
Jawaban: "konsultan medis"

**P2.4. Di antara penyetuju (CEO, CTO, konsultan medis), Anda sendiri memegang peran apa?**
Jawaban: "Bukan ketiganya" (tanpa penjelasan tambahan).

**P2.4a (klarifikasi). Apa peran Anda di proyek ini, dan bagaimana persetujuan CEO, CTO, dan konsultan medis akan sampai ke sini?**
Jawaban: "saya project manager, dokumen saya teruskan ke CEO dan CTO"

**P2.4b (klarifikasi). Kapan konsultan medis dilibatkan dalam persetujuan dokumen/hasil proyek Rumah Sehat?**
Jawaban: "Hanya saat go-live/UAT".

**Konfirmasi rangkuman Putaran 2.** Rangkuman ditampilkan (9 peran; layanan konsultasi kesehatan mental sebagai layanan ke-7; dibiayai sendiri dan berdiri sendiri, nama badan usaha belum diketahui; penyetuju CEO, CTO, konsultan medis; user adalah project manager yang meneruskan dokumen gate ke CEO dan CTO; konsultan medis hanya dilibatkan saat UAT/go-live; persetujuan user di gate dianggap persetujuan CEO dan CTO yang diteruskan, sign-off UAT/go-live mencakup konsultan medis).
Jawaban: "Sudah benar semua".

## Putaran 3: Fitur dan alur utama

**P3.1. Dari 7 layanan berikut, mana yang wajib ada di rilis pertama (go-live)?** (1 Konsultasi online dokter; 2 Pendaftaran klinik rekanan; 3 Pendaftaran lab; 4 Pembelian obat dan produk kesehatan; 5 Home visit tenaga medis; 6 Informasi biaya MCU dan lab; 7 Konsultasi kesehatan mental/psikolog. Layanan yang tidak disebut dicatat sebagai rilis berikutnya.)
Jawaban: "1,2,3,5,7,6"
Catatan pewawancara: layanan 4 (pembelian obat dan produk kesehatan) tidak termasuk rilis pertama, dicatat sebagai rilis berikutnya.

**P3.2. Konsultasi online (dokter dan psikolog) di rilis pertama dilakukan lewat media apa saja?** (pilihan ganda: chat teks, panggilan suara, panggilan video)
Jawaban: "Panggilan video, Chat teks". Panggilan suara tidak dipilih.

**P3.3. Bagaimana pasien membayar layanan Rumah Sehat di rilis pertama?** (pilihan: bayar di aplikasi; bayar langsung ke mitra; campuran = konsultasi online dibayar di aplikasi, klinik/lab/home visit dibayar langsung ke mitra; gratis untuk pasien)
Jawaban: "Campuran" (konsultasi online dibayar di aplikasi; klinik, lab, dan home visit dibayar langsung ke mitra).

**P3.4. Apakah dokter bisa menerbitkan resep setelah konsultasi online di rilis pertama?** (pilihan: ya, resep digital ditebus di luar aplikasi; ya, ditebus di apotek rekanan; tidak ada resep; belum tahu)
Jawaban: "Ya, ditebus di luar" (dokter menerbitkan resep digital di aplikasi, pasien menebus sendiri di apotek mana pun).

**P3.5. Apakah peran apoteker dipakai di rilis pertama Rumah Sehat (pembelian obat ditunda, resep ditebus di luar)?**
Jawaban: "Tidak, tunggu rilis obat".

**P3.6. Fitur apa yang jelas tidak termasuk Rumah Sehat, setidaknya di rilis pertama?**
(Pewawancara memberi contoh: integrasi BPJS atau asuransi, rekam medis elektronik lengkap, konten edukasi, membership, loyalitas/poin, chatbot AI, versi web pasien.)
Jawaban: "integrasi BPJS dan asuransi belum termasuk, atau buatkan list mana yg perlu di exclude"
Catatan pewawancara: user meminta daftar usulan exclusion; daftar disusun sebagai usulan (bukan keputusan), user memilih di P3.6a.

**P3.6a. Nomor mana saja yang Anda keluarkan dari rilis pertama?**
Daftar usulan pewawancara (usulan, bukan keputusan):
- Sudah dikeluarkan dari jawaban sebelumnya: A integrasi BPJS; B integrasi asuransi swasta; C pembelian obat dan produk kesehatan termasuk peran apoteker (rilis berikutnya); D konsultasi panggilan suara.
- Kandidat: 1 RME lengkap untuk klinik/lab; 2 integrasi otomatis ke sistem klinik/lab rekanan (konfirmasi manual oleh admin mitra); 3 artikel/konten edukasi kesehatan; 4 langganan/membership berbayar; 5 program loyalitas, poin, voucher promo; 6 chatbot AI / pemeriksa gejala otomatis; 7 versi web untuk pasien; 8 integrasi perangkat kesehatan/wearable; 9 antrean real-time klinik; 10 multi-bahasa (rilis pertama hanya Bahasa Indonesia); 11 pengantaran sampel lab / pengambilan sampel di rumah.
- Aturan: nomor yang tidak disebut tidak otomatis masuk scope, dicatat "belum diputuskan" dan dibahas di Tahap 3.
Jawaban: "4,6,7,8,11"
Catatan pewawancara: dikeluarkan dari rilis pertama = 4, 6, 7, 8, 11 (ditambah A–D). Belum diputuskan = 1, 2, 3, 5, 9, 10.

**Konfirmasi rangkuman Putaran 3.** Rangkuman ditampilkan (6 layanan rilis pertama; chat teks + panggilan video; pembayaran campuran; resep digital ditebus di luar; 8 peran rilis pertama, apoteker menunggu rilis obat; daftar tidak termasuk; daftar belum diputuskan).
Jawaban: "Sudah benar semua".

## Putaran 4: Data dan integrasi

**P4.1. Data pasien apa saja yang akan disimpan di Rumah Sehat?** (daftar contoh 1–10 ditampilkan)
Jawaban: belum dijawab. User menyampaikan perubahan scope (lihat "Perubahan scope 1" di bawah). P4.1 diajukan ulang setelah perubahan selesai diklarifikasi.

## Perubahan scope 1 (disampaikan saat Putaran 4)

Pernyataan user: "saya ingin memasukan pembelian obat, product kecantikan dari outlet rekanan di rilis pertama"
Keputusan lama yang terdampak (Tahap 0 belum lulus gate, jadi dicatat sebagai koreksi jawaban intake, bukan change request):
- P3.1: layanan 4 (pembelian obat dan produk kesehatan) sebelumnya rilis berikutnya. Sekarang masuk rilis pertama.
- Baru: produk kecantikan, dijual oleh "outlet rekanan".
- Perlu ditinjau ulang: P3.4 (resep ditebus di luar aplikasi), P3.5 (apoteker tidak dipakai di rilis pertama), P3.3 (pembayaran untuk pembelian produk belum diatur).

**PS1. Produk apa saja yang dijual lewat Rumah Sehat (dari outlet rekanan) di rilis pertama?** (pilihan ganda: obat bebas; obat resep; produk kecantikan; produk kesehatan lain seperti vitamin, suplemen, alat kesehatan)
Jawaban: "Obat bebas, Produk kecantikan, Produk kesehatan lain". Obat resep tidak dipilih.

**PS2. Outlet rekanan yang menjual produk di Rumah Sehat itu jenis usaha apa saja?** (pilihan ganda: apotek; toko kecantikan; toko alat kesehatan; klinik rekanan)
Jawaban: "Toko kecantikan, Toko alat kesehatan, Klinik rekanan". Apotek tidak dipilih.
Catatan pewawancara: obat bebas dijual (PS1) tetapi apotek tidak dipilih sebagai outlet; diklarifikasi di PS2a.

**PS2a (klarifikasi). Apotek tidak termasuk outlet rekanan, padahal obat bebas dijual. Siapa yang menjual obat bebas di Rumah Sehat?** (pewawancara menyampaikan bahwa aturan penjualan obat online akan diperiksa ke sumber resmi di Tahap 2)
Jawaban: "Klinik rekanan".
Catatan pewawancara: kebolehan regulasi penjualan obat bebas oleh klinik secara online [TIDAK DIKETAHUI], diverifikasi compliance-reviewer di Tahap 2.

**PS3. Jawaban lama (3.5): apoteker menunggu rilis pembelian obat. Sekarang obat bebas dijual lewat klinik rekanan di rilis pertama. Apakah peran apoteker dipakai di rilis pertama?**
Jawaban: "Tidak, tetap ditunda" (pesanan obat bebas dikelola admin klinik).

**PS4. Bagaimana pasien membayar pembelian produk (obat bebas, kecantikan, kesehatan lain) dari outlet rekanan?** (pilihan: bayar di aplikasi, dana diteruskan ke outlet; bayar langsung ke outlet/COD; pasien memilih)
Jawaban: "Bayar di aplikasi" (dibayar di muka lewat payment gateway; Rumah Sehat meneruskan dana ke outlet).
Catatan pewawancara: ini menambah alur penerusan dana (settlement) ke outlet; mekanisme dan jadwal penerusan dana [TIDAK DIKETAHUI], masuk pertanyaan terbuka.

**PS5. Bagaimana produk yang dibeli sampai ke pasien?** (pilihan ganda: kurir pihak ketiga terintegrasi; kurir outlet sendiri; diambil di outlet)
Jawaban: "Kurir pihak ketiga". Kurir outlet sendiri dan pick-up tidak dipilih.
Catatan pewawancara: penyedia kurir yang dipakai [TIDAK DIKETAHUI], ditanyakan di Putaran 4.

**PS6. Siapa yang mengelola katalog produk dan pesanan di toko kecantikan dan toko alat kesehatan rekanan?** (pilihan: peran baru admin outlet; admin Rumah Sehat; campuran)
Jawaban (teks bebas): "Ruma sehat nhanya berperan sebagai gateway, semuanya hasil integrasi dengan rekanan."
Catatan pewawancara: katalog, stok, dan pesanan produk berasal dari integrasi dengan sistem rekanan. Cakupan "semuanya" (apakah juga klinik, lab, home visit) diklarifikasi di PS6a. Ini berkaitan dengan kandidat 2 di P3.6a ("integrasi otomatis ke sistem klinik/lab rekanan") yang sebelumnya "belum diputuskan".

**PS6a (klarifikasi). Apakah prinsip "Rumah Sehat hanya gateway, semuanya hasil integrasi dengan rekanan" berlaku juga untuk klinik, lab, dan home visit (bukan hanya produk)?** (pewawancara menyebut keterkaitan dengan P3.6a kandidat 2 "belum diputuskan" dan peran admin klinik/admin lab)
Jawaban: "Berlaku untuk semua" (produk, jadwal klinik, jadwal/harga lab, home visit diambil dari sistem rekanan via integrasi; Rumah Sehat tidak menyimpan katalog sendiri).
Catatan pewawancara: P3.6a kandidat 2 berubah dari "belum diputuskan" menjadi in-scope rilis pertama. Dampak pada peran admin klinik/lab dan konsultasi online diklarifikasi berikutnya.

**PS6b (klarifikasi). Untuk konsultasi online (chat teks dan video), di mana dokter dan psikolog bekerja?** (pilihan: di Rumah Sehat; di sistem rekanan; belum tahu)
Jawaban: "Di Rumah Sehat" (dokter/psikolog memakai aplikasi/portal Rumah Sehat untuk chat, video, catatan, resep).

**PS6c (klarifikasi). Karena data klinik dan lab diambil lewat integrasi, apakah admin klinik dan admin lab masih perlu login ke Rumah Sehat?**
Jawaban: "Tetap perlu" (portal di Rumah Sehat, contoh yang disebut pewawancara: laporan transaksi dan penerusan dana, profil, keluhan).

**PS6d (klarifikasi). Apakah toko kecantikan dan toko alat kesehatan rekanan juga punya admin di portal Rumah Sehat (seperti admin klinik/lab)?**
Jawaban: "Ya, peran admin outlet" (portal untuk laporan, penerusan dana, profil, keluhan; katalog/pesanan tetap via integrasi).

**PS6e (klarifikasi). Dokter dan psikolog yang melayani konsultasi online di Rumah Sehat berasal dari mana?** (pilihan: mitra langsung Rumah Sehat; dari klinik rekanan; keduanya)
Jawaban: "Keduanya" (ada mitra langsung Rumah Sehat dan ada tenaga dari klinik rekanan).
Catatan pewawancara: tugas fisioterapis dan "tenaga medis" (konsultasi online, home visit, atau keduanya) serta asal mereka belum ditanyakan, dicatat sebagai pertanyaan terbuka.

**Konfirmasi rangkuman perubahan scope 1.** Tabel sebelum → sesudah ditampilkan (pembelian produk masuk rilis pertama; produk: obat bebas, kecantikan, kesehatan lain, tanpa obat resep; outlet: toko kecantikan, toko alkes, klinik rekanan, obat bebas oleh klinik; resep tetap ditebus di luar; apoteker tetap ditunda; pembayaran produk di aplikasi, dana diteruskan ke outlet; kurir pihak ketiga terintegrasi; Rumah Sehat sebagai gateway untuk produk, klinik, lab, home visit; integrasi klinik/lab masuk rilis pertama; konsultasi di Rumah Sehat; dokter/psikolog mitra langsung dan dari klinik rekanan; 9 peran termasuk admin outlet; 7 layanan).
Jawaban: "Sudah benar semua".

## Putaran 4: Data dan integrasi (dilanjutkan)

**P4.1 (diajukan ulang). Data pasien apa saja yang disimpan di Rumah Sehat?** (1 identitas dasar: nama, tanggal lahir, jenis kelamin, nomor HP, email, alamat untuk home visit dan pengiriman; 2 NIK/foto KTP; 3 riwayat dan isi chat konsultasi termasuk foto; 4 rekaman panggilan video; 5 catatan dokter/psikolog; 6 resep digital; 7 hasil lab dari lab rekanan; 8 riwayat pendaftaran klinik, lab, home visit; 9 data anggota keluarga; 10 riwayat pembayaran dan pesanan produk)
Jawaban: "1,2,3,5,6,7,8,9,10". Nomor 4 (rekaman panggilan video) tidak disimpan.

**P4.2. Apakah sudah ada penyedia pihak ketiga yang dipilih/dikontrak untuk payment gateway, panggilan video, kurir, dan OTP/notifikasi?**
Jawaban: "Belum ada" (pemilihan diserahkan ke tahap teknis Tahap 4–5, lalu dibawa ke user untuk disetujui).

**P4.3. Apakah sudah ada klinik, lab, atau toko rekanan yang bergabung, dan apakah sistem mereka (untuk integrasi) sudah diketahui?**
Jawaban: "Belum ada rekanan" (rekanan dicari paralel dengan pembangunan aplikasi).
Catatan pewawancara: risiko tinggi. Prinsip gateway (PS6a: semua data klinik, lab, home visit, produk dari integrasi) bergantung pada sistem rekanan yang belum ada dan belum diketahui kemampuan integrasinya. Dicatat sebagai pertanyaan terbuka dan risiko untuk Tahap 1 dan Tahap 4.

**P4.4. Bagaimana pasien masuk (login) ke aplikasi Rumah Sehat?** (pilihan ganda: nomor HP + OTP; email + password; akun Google/Apple)
Jawaban: "Email + password, Akun Google/Apple". Nomor HP + OTP tidak dipilih.

**P4.5. Lewat saluran apa Rumah Sehat mengirim pemberitahuan ke pasien (konfirmasi, pengingat, status pesanan, hasil lab)?** (pilihan ganda: notifikasi push; email; WhatsApp; SMS)
Jawaban: "Notifikasi push, Email, WhatsApp". SMS tidak dipilih.

**P4.6. Bagaimana NIK/foto KTP pasien diperlakukan di Rumah Sehat?** (pilihan: hanya disimpan; verifikasi manual; verifikasi otomatis e-KYC; wajib untuk layanan tertentu)
Jawaban: "Wajib untuk layanan tertentu" (layanan tidak disebut; diklarifikasi di P4.6a).

**P4.6a (klarifikasi). Layanan mana saja yang mewajibkan pasien memberikan NIK/KTP?** (pilihan ganda: konsultasi online termasuk resep; home visit; pendaftaran klinik & lab; pembelian produk)
Jawaban: "Home visit, Pendaftaran klinik & lab, Konsultasi online". Pembelian produk tidak mewajibkan NIK/KTP.

**P4.6b (klarifikasi). Untuk konsultasi online, home visit, dan pendaftaran klinik/lab: bagaimana NIK/KTP pasien diverifikasi?** (pilihan: hanya disimpan; verifikasi manual; verifikasi otomatis e-KYC; belum tahu)
Jawaban: "Verifikasi manual" (admin Rumah Sehat mencocokkan dengan foto KTP sebelum layanan dipakai).

**Konfirmasi rangkuman Putaran 4.** Rangkuman ditampilkan (data disimpan 1,2,3,5,6,7,8,9,10, rekaman video tidak; NIK/KTP wajib untuk konsultasi, home visit, pendaftaran klinik/lab, verifikasi manual admin Rumah Sehat; login email+password dan Google/Apple; notifikasi push, email, WhatsApp; daftar integrasi; penyedia belum ada; rekanan belum ada, risiko utama).
Jawaban: "Sudah benar semua".

## Putaran 5: Kepatuhan, keamanan, dan audit

**P5.1. Apakah ada regulasi atau standar yang menurut Anda/CEO/CTO/konsultan medis wajib dipatuhi Rumah Sehat?**
Jawaban: "Belum tahu, teliti Tahap 2" (compliance reviewer mencari dari sumber resmi di Tahap 2, hasilnya dibawa ke user).

**P5.2. Bagaimana status badan usaha dan perizinan Rumah Sehat saat ini?** (pilihan: badan usaha ada, izin belum; ada sebagian izin; badan usaha belum berdiri; belum tahu)
Jawaban (teks bebas, berupa pertanyaan balik): "apakah harus badan usaha terpisah? tidak bisa hanya palikasi sja, yang dimiliki oaehh perussaaan IT ?"
Tanggapan pewawancara: pertanyaan regulasi tidak dijawab dari ingatan; dicatat sebagai pertanyaan terbuka prioritas tinggi untuk compliance-reviewer di Tahap 2 (diverifikasi ke sumber resmi). Pernyataan "dimiliki oleh perusahaan IT" diklarifikasi di P5.2a karena berkaitan dengan P2.2 ("biaya sendiri dan berdiri sendiri").

**P5.2a (klarifikasi). Apakah rencananya Rumah Sehat dimiliki dan dijalankan oleh perusahaan IT yang sudah ada (berbadan hukum)?**
Jawaban: "Ya, perusahaan IT ada" (Rumah Sehat adalah produk milik perusahaan IT berbadan hukum; "berdiri sendiri" di P2.2 berarti bukan bagian grup klinik/apotek).
Catatan pewawancara: nama perusahaan IT tetap [TIDAK DIKETAHUI]. Status izin terkait layanan kesehatan digital dan kebutuhan badan usaha terpisah [TIDAK DIKETAHUI], diteliti di Tahap 2.

**P5.3. Siapa yang akan mengaudit Rumah Sehat atau kepada siapa kepatuhan harus dibuktikan?** (pilihan ganda: auditor internal; auditor eksternal/sertifikasi; regulator/pemerintah; mitra rekanan)
Jawaban: "Auditor eksternal, Regulator/pemerintah".
Catatan pewawancara: nama auditor/standar sertifikasi dan instansi regulator [TIDAK DIKETAHUI]; instansi diteliti di Tahap 2.

**P5.4. Aktivitas apa saja yang wajib tercatat di audit trail?** (usulan 1–10: login/logout/gagal login; akses data pasien; pembuatan/perubahan catatan dokter/psikolog dan resep; verifikasi KTP; pembayaran, refund, penerusan dana; perubahan status pendaftaran/home visit/pesanan; pengelolaan akun nakes dan admin mitra; perubahan hak akses/peran; ekspor/unduh data; permintaan pasien menghapus/melihat datanya)
Jawaban: "semua transaksi yang terjadi pun masuk audit trail"
Catatan pewawancara: ditafsirkan sebagai usulan 1–10 seluruhnya ditambah setiap transaksi yang terjadi; tafsiran ini dikonfirmasi di rangkuman Putaran 5.

**P5.5. Apakah ada ketentuan lokasi penyimpanan data (server/cloud) Rumah Sehat dari sisi bisnis Anda?** (pilihan: wajib di Indonesia; boleh di luar negeri; belum tahu, ikuti Tahap 2)
Jawaban: "Wajib di Indonesia".

**Konfirmasi rangkuman Putaran 5.** Rangkuman ditampilkan (regulasi diteliti Tahap 2; pemilik perusahaan IT berbadan hukum, nama belum diketahui; kebutuhan badan usaha terpisah = pertanyaan terbuka prioritas tinggi; status izin belum diketahui; auditor eksternal dan regulator; audit trail = usulan 1–10 seluruhnya + setiap transaksi; data wajib di Indonesia).
Jawaban: "Sudah benar semua".

## Putaran 6: Non-fungsional

**P6.1. Aplikasi mobile pasien Rumah Sehat diluncurkan di platform apa pada rilis pertama?** (pilihan: Android dan iOS; Android dulu; iOS dulu)
Jawaban: "Android dan iOS".

**P6.2. Di platform apa dokter, psikolog, tenaga medis, fisioterapis, dan para admin (klinik, lab, outlet, Rumah Sehat) bekerja?** (pilihan: web untuk semua; mobile untuk nakes, web untuk admin; nakes keduanya, web untuk admin)
Jawaban: "Mobile nakes, web admin" (dokter, psikolog, tenaga medis, fisioterapis memakai aplikasi mobile; para admin memakai portal web).
Catatan pewawancara: apakah aplikasi mobile nakes juga Android dan iOS [TIDAK DIKETAHUI], masuk pertanyaan terbuka.

**P6.3. Berapa perkiraan jumlah pengguna dan aktivitas di tahun pertama?** (contoh: pasien terdaftar, konsultasi per hari, jumlah dokter/psikolog, jumlah rekanan)
Jawaban: "belum ada angka"
Catatan pewawancara: volume pengguna dan transaksi [TIDAK DIKETAHUI], masuk pertanyaan terbuka.

**P6.4. Kapan layanan Rumah Sehat (aplikasi dan konsultasi online) harus tersedia?** (pilihan: 24 jam, 7 hari; jam operasional tertentu; belum tahu)
Jawaban: "24 jam, 7 hari" (ketersediaan dokter mengikuti jadwal masing-masing).
Catatan pewawancara: target ketersediaan dalam angka (mis. persen uptime) [TIDAK DIKETAHUI], dibahas sebagai NFR di Tahap 3.

**P6.5. Di wilayah mana Rumah Sehat beroperasi di rilis pertama (terutama home visit, kurir, klinik/lab rekanan)?** (pilihan: seluruh Indonesia; kota/provinsi tertentu; belum tahu)
Jawaban: "Seluruh Indonesia" (di mana pun ada rekanan).
Catatan pewawancara: Indonesia memiliki lebih dari satu zona waktu; zona waktu acuan jadwal [TIDAK DIKETAHUI], ditangani sebagai detail teknis/NFR di Tahap 3.

**Konfirmasi rangkuman Putaran 6.** Rangkuman ditampilkan (pasien Android+iOS; nakes aplikasi mobile, platform belum diketahui; admin portal web; volume belum ada angka; 24/7; seluruh Indonesia).
Jawaban: "Sudah benar semua".

## Putaran 7: Batasan

**P7.1. Berapa anggaran yang disiapkan untuk membangun Rumah Sehat sampai rilis pertama?**
Jawaban: "belum ada"
Catatan pewawancara: anggaran [TIDAK DIKETAHUI], masuk pertanyaan terbuka; estimasi disusun di Tahap 1.

**P7.2. Apakah ada target tanggal rilis pertama (go-live)?**
Jawaban: "1 maret 2027"
Catatan pewawancara: alasan tanggal tidak disebut. Jarak dari tanggal intake (2026-11-17) sekitar 3,5 bulan; kelayakan jadwal terhadap scope (7 layanan, 9 peran, integrasi rekanan yang belum ada) dicatat sebagai risiko untuk dinilai di Tahap 1.

**P7.3. Siapa saja yang akan mengerjakan Rumah Sehat?**
Jawaban: "saya andalkan runbook ini plus 1 mobile dev"
Catatan pewawancara: tim = user (project manager) + Claude via runbook + 1 mobile developer. Peran lain (QA manusia, DevOps, UI/UX) tidak disebut.

**P7.4. Apakah ada teknologi, infrastruktur, atau penyedia yang wajib dipakai atau dilarang?**
Jawaban: "flutter"
Catatan pewawancara: ditafsirkan sebagai Flutter wajib untuk aplikasi mobile (pasien dan nakes); teknologi lain (backend, web admin, cloud) tidak ada batasan selain data di Indonesia (P5.5), diserahkan ke Tahap 5. Tafsiran dikonfirmasi di rangkuman Putaran 7.

**Konfirmasi rangkuman Putaran 7.** Rangkuman ditampilkan (anggaran belum ada; go-live 1 Maret 2027, risiko jadwal; tim user + Claude/runbook + 1 mobile dev; Flutter wajib untuk aplikasi mobile pasien dan nakes; teknologi lain diserahkan ke Tahap 5, data di Indonesia).
Jawaban: "Sudah benar semua".

## Putaran 8: Sukses dan go-live

**P8.1. Bagaimana Anda (dan CEO, CTO) akan menilai Rumah Sehat berhasil, misalnya 6 atau 12 bulan setelah go-live?**
Jawaban: "ada 10ribu user aktif, 70 dokter, 20 klinik, 10 lab"
Catatan pewawancara: kurun waktu target diklarifikasi di P8.1a. Definisi "user aktif" (harian/bulanan) dan apakah "70 dokter" termasuk psikolog/nakes lain [TIDAK DIKETAHUI], masuk pertanyaan terbuka untuk Tahap 1. Jawaban ini melengkapi P1.5 ("belum ada angka pasti").

**P8.1a (klarifikasi). Target 10.000 user aktif, 70 dokter, 20 klinik, 10 lab harus tercapai dalam berapa lama setelah go-live (1 Maret 2027)?** (pilihan: 6 bulan; 12 bulan; akhir 2027; belum tahu)
Jawaban: "12 bulan" (sekitar Maret 2028).

**P8.2. Apa syarat minimum agar rilis pertama boleh go-live pada 1 Maret 2027?** (usulan 1–7 ditampilkan)
Jawaban: belum dijawab. User menyampaikan kebutuhan tambahan: "oiya harus ada semacam CMS atau backoffice untuk admin rumah sehat ya"
Catatan pewawancara: backoffice untuk admin Rumah Sehat (portal web, konsisten dengan P6.2) wajib ada. Fungsi backoffice/CMS diklarifikasi di P8.2a. P8.2 diajukan ulang sesudahnya.

**P8.2a (klarifikasi). Fungsi apa saja yang wajib ada di backoffice admin Rumah Sehat pada rilis pertama?** (pilihan ganda: verifikasi & kelola akun = verifikasi KTP pasien, verifikasi dan pengelolaan akun dokter, psikolog, tenaga kesehatan, serta pendaftaran klinik, lab, dan outlet rekanan; laporan & penerusan dana; kelola konten CMS (banner, halaman info, artikel, promo; konten edukasi jadi masuk rilis pertama); layanan pelanggan)
Jawaban: "Verifikasi & kelola akun, Laporan & penerusan dana, Kelola konten (CMS), Layanan pelanggan" (keempatnya).
Catatan pewawancara: P3.6a kandidat 3 (artikel/konten edukasi) berubah dari "belum diputuskan" menjadi in-scope rilis pertama. Kandidat 5 (loyalitas, poin, voucher) tetap "belum diputuskan"; banner promo di CMS tidak sama dengan sistem voucher.

**P8.2 (diajukan ulang). Apa syarat minimum agar rilis pertama boleh go-live pada 1 Maret 2027?** (1 minimal rekanan terintegrasi; 2 minimal dokter/psikolog terverifikasi; 3 ketujuh layanan berjalan; 4 tayang di Play Store dan App Store; 5 izin dan kepatuhan regulasi terpenuhi; 6 lulus audit keamanan; 7 sign-off UAT CEO, CTO, konsultan medis)
Jawaban: "2 klinik,1 lab, 3 outlet, 5 dokter, 1 psikolog; 3;4;5;6;7"
Catatan pewawancara: ditafsirkan sebagai syarat 1 = 2 klinik, 1 lab, 3 outlet terintegrasi; syarat 2 = 5 dokter, 1 psikolog terverifikasi; ditambah syarat 3, 4, 5, 6, 7. Semua 7 syarat berlaku.

**P8.3. Kalau menjelang 1 Maret 2027 ada syarat go-live yang belum terpenuhi, mana yang diutamakan?** (pilihan: tanggal tetap, scope dikurangi; scope tetap, tanggal mundur; diputuskan nanti oleh CEO dan CTO. Pewawancara menyampaikan pertimbangan profesional bahwa syarat 5, 6, 7 tidak termasuk yang bisa dikurangi, keputusan tetap di user.)
Jawaban: "Tanggal tetap, scope kurang" (go-live 1 Maret 2027 dengan layanan yang siap; layanan lain menyusul).
Catatan pewawancara: apakah syarat 5, 6, 7 dikecualikan dari pengurangan dikonfirmasi di rangkuman Putaran 8.

**Konfirmasi rangkuman Putaran 8.** Rangkuman ditampilkan (KPI 12 bulan: 10.000 user aktif, 70 dokter, 20 klinik, 10 lab; 7 syarat go-live dengan angka minimum; tanggal tetap, scope dikurangi, syarat 5, 6, 7 tidak bisa dikurangi; backoffice 4 fungsi; konten edukasi masuk rilis pertama).
Jawaban: "Sudah benar semua".

## Akhir wawancara

Wawancara 8 putaran selesai 2026-11-17. Project Brief disusun di `docs/00-project-brief.md`.

## Catatan revisi

- 2026-11-17 (setelah fact-check): teks opsi di P4.1 dan P8.2a dilengkapi sesuai teks yang ditampilkan ke user saat wawancara (sebelumnya diringkas). Jawaban user tidak diubah.

## Checkpoint 1 (gate Tahap 0)

2026-11-17. Pertanyaan: apakah user menyetujui Project Brief Tahap 0, termasuk tafsiran "berdiri sendiri = bukan bagian grup klinik/apotek", usulan pemilik/tenggat PT-01–PT-15, dan risiko R-01–R-04 sebagai catatan?
Jawaban: "Setujui".
