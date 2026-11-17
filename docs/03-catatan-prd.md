# Catatan Wawancara Tahap 3 (PRD + NFR)

Catatan apa adanya. File ini menjadi bukti untuk setiap label `[USER] T3-Qx` di `docs/03-prd.md`.

Tanggal: 2026-11-17
Pewawancara: Claude (persona Product Manager + QA + Tech Lead)
Narasumber: user (project manager; persetujuan diteruskan ke CEO dan CTO, lihat brief "Stakeholder dan sign-off")
Cara bertanya: satu pertanyaan per giliran, sesuai arahan user di Tahap 0. Setiap pertanyaan ditulis lengkap di chat beserta konteks dan arti pilihan sebelum dialog pilihan ditampilkan.

## Tema A: scope dan prioritas

**T3-Q1. Apakah keempat butir yang belum diputuskan (RME lengkap klinik/lab, loyalitas/poin/voucher, antrean real-time klinik, multi-bahasa) ditunda dari rilis 1? (PT-13)** (konteks: BRD §4.3; 104 hari menuju go-live, R-02, R-04; RME lengkap dibedakan dari kewajiban rekam medis BR-114. Pilihan: semua ditunda (rekomendasi pewawancara); ada yang masuk rilis 1)
Jawaban: "Ada yang masuk rilis 1".

**T3-Q1a. Butir mana yang masuk rilis 1?** (pilihan ganda, disertai perkiraan beban per butir dari sudut pandang Tech Lead: RME lengkap paling berat dan bertentangan dengan prinsip gateway; loyalitas/voucher menyentuh harga dan pembagian dana; antrean real-time bergantung sistem rekanan; multi-bahasa ringan–sedang)
Jawaban: "RME lengkap klinik/lab, Antrean real-time klinik, RME degan fitur minimalis sebagai stimulus agar rekanan tanpa sistem bisa bergabung".
Catatan pewawancara: dibaca sebagai RME **minimalis** (bukan lengkap) untuk klinik dan lab rekanan yang belum punya sistem, agar mereka dapat bergabung; antrean real-time klinik masuk rilis 1; loyalitas/poin/voucher dan multi-bahasa tidak dipilih sehingga Won't di rilis 1. Disampaikan ke user: ini change request (CR-006) atas prinsip gateway BRD (BR-020, BR-021, BR-022, BR-025, BR-070); mengurangi ketergantungan R-01/PT-05; menambah beban jadwal (R-02); PMK 24/2022 mewajibkan RME fasyankes terhubung SATUSEHAT `[SRC-18]`, sehingga kewajiban Rumah Sehat sebagai penyedia RME, integrasi SATUSEHAT, dan pengendali data masuk konfirmasi legal (L-05 meluas).

**T3-Q1b. Fitur apa saja yang termasuk "RME minimalis" untuk klinik/lab rekanan tanpa sistem di rilis 1?** (pilihan ganda: data layanan di portal (jadwal, daftar layanan, harga diisi admin klinik/lab sebagai pengganti integrasi); catatan rekam medis kunjungan; hasil lab di portal; home visit + antrean di portal. Pewawancara menyebut butir 1–3 sebagai minimum agar rekanan tanpa sistem bisa bergabung, dan menyebut outlet tanpa sistem tidak dimasukkan karena user menyebut RME klinik/lab; outlet bisa disebut lewat "Other")
Jawaban: "Catatan rekam medis, Data layanan di portal, Home visit + antrean portal, Hasil lab di portal, pastikan data pada rekam medis harus sesuai dengan ketentuan satu sehhat kemenkes".
Catatan pewawancara: keempat kelompok fitur masuk. Data rekam medis wajib sesuai ketentuan SATUSEHAT Kemenkes. Outlet tanpa sistem tidak disebut user, sehingga BR-033 (katalog dari sistem outlet) tidak berubah. **Koreksi lokasi (T3-Q14, T3-Q15):** pertanyaan ini ditulis pewawancara dengan anggapan fitur berada "di portal Rumah Sehat"; user meluruskan bahwa keempat kelompok fitur berada di aplikasi RME minimalis yang terpisah dari Rumah Sehat dan terhubung lewat integrasi.

**T3-Q1c. Pasien siapa saja yang dicatat di RME minimalis rekanan tanpa sistem?** (pilihan: hanya pasien Rumah Sehat (rekomendasi pewawancara); semua pasien rekanan termasuk walk-in)
Jawaban: "Hanya pasien Rumah Sehat (Recommended)".
Catatan pewawancara: RME minimalis hanya mencatat pasien dengan appointment, home visit, atau pemeriksaan lab yang dipesan lewat aplikasi Rumah Sehat.

**T3-Q2. Kapan urutan pengurangan scope (layanan mana dilepas lebih dulu bila tidak siap 1 Maret 2027) ditetapkan?** (konteks: brief P8.3; Business Case R-01 "Tahap 3 atau 7"; CR-006 menambah scope. Pilihan: di Tahap 7 (rekomendasi pewawancara); sekarang, pakai usulan: dilepas lebih dulu antrean real-time → obat bebas lalu seluruh pembelian produk → home visit → konsultasi fisioterapis dan tenaga medis → lab dan info biaya MCU/lab → klinik; konsultasi dokter dan psikolog dipertahankan paling akhir)
Jawaban: "Sekarang, pakai usulan".
Catatan pewawancara: urutan usulan berlaku. RME minimalis tidak ditempatkan di urutan karena merupakan syarat tampilnya rekanan tanpa sistem; nasibnya mengikuti layanan yang memakainya (disampaikan ke user setelah jawaban).

## Tema B: alur layanan

**T3-Q3. Bagaimana data anggota keluarga dipakai di rilis 1? (PT-26)** (konteks: BR-002, BR-104; pembelian produk tidak memerlukan identitas per orang. Pilihan: bisa dipesankan; data saja; tidak dipakai rilis 1)
Jawaban: "Bisa dipesankan".
Catatan pewawancara: pasien dapat memesan konsultasi, pendaftaran klinik/lab, dan home visit untuk anggota keluarga di profilnya; catatan konsultasi, rekam medis, dan hasil lab tercatat atas nama anggota keluarga itu. PT-26 terjawab.

**T3-Q3a. Bagaimana identitas anggota keluarga diverifikasi sebelum dipesankan layanan?** (konteks: BR-003, BR-004; anak umumnya belum memiliki KTP; persetujuan anggota keluarga dewasa dan orang tua/wali untuk anak menunggu L-08. Pilihan: dewasa KTP, anak KK (rekomendasi pewawancara); hanya anak; cukup NIK)
Jawaban: "Dewasa KTP, anak KK (Recommended)".
Catatan pewawancara: anggota keluarga dewasa: NIK + foto KTP; anak: NIK + foto Kartu Keluarga yang memuat pemilik akun sebagai orang tua/wali; semuanya diverifikasi manual admin Rumah Sehat. Pemilik akun menyatakan telah mendapat persetujuan anggota keluarga dewasa, atau memberi persetujuan sebagai orang tua/wali untuk anak.

**T3-Q4. Dari mana jadwal, tarif, dan durasi sesi konsultasi online berasal?** (konteks: BR-013, BR-018, P-03; catatan T2-Q6. Pilihan: diatur di Rumah Sehat (rekomendasi pewawancara); integrasi bila ada, portal bila tidak)
Jawaban: "Diatur di Rumah Sehat (Recommended)".
Catatan pewawancara: admin klinik mengisi tarif dan durasi sesi per tenaga kesehatan dan per jenis konsultasi (chat/video) di portal; jadwal praktik online diatur tenaga kesehatan di aplikasinya dan dapat dilihat serta diubah admin klinik di portal. Tidak lewat integrasi.

## Tema C: refund (T1)

**T3-Q5. Berapa batas waktu pembatalan oleh pasien agar layanan terjadwal (konsultasi, klinik, lab, home visit) masih di-refund?** (konteks: BR-045, T2-Q17, T2-Q18. Pilihan: diatur admin, nilai awal 24 jam (rekomendasi pewawancara); diatur admin, nilai awal 2 jam; ikut kebijakan tiap rekanan)
Jawaban: "Diatur admin, awal 24 jam (Recommended)".
Catatan pewawancara: parameter per jenis layanan yang dapat diubah admin Rumah Sehat; nilai awal 24 jam sebelum jadwal untuk semua jenis.

**T3-Q6. Bagaimana perlakuan pasien yang tidak hadir (no-show) di layanan terjadwal tanpa membatalkan?** (konteks: BR-045; opsi reschedule di T2-Q17 tidak dipilih user. Definisi no-show yang diusulkan: konsultasi = pasien tidak bergabung sampai batas toleransi keterlambatan (parameter admin) habis; klinik/lab = appointment tidak di-check-in sampai jadwalnya berakhir; home visit = tenaga kesehatan menandai pasien tidak ada di lokasi. Pilihan: tidak di-refund (rekomendasi pewawancara); refund sebagian; boleh jadwal ulang sekali)
Jawaban: "Tidak di-refund (Recommended)".
Catatan pewawancara: no-show diperlakukan sama dengan pembatalan setelah batas waktu; rekanan tetap menerima bagiannya. Definisi no-show yang diusulkan disampaikan bersama pertanyaan dan dipakai di PRD.

**T3-Q7. Berapa yang dikembalikan ke pasien saat refund, dan siapa menanggung bagian rekanan, markup, dan biaya payment gateway yang tidak kembali? (PT-27)** (konteks: BR-041, BR-044; pengembalian biaya PG saat refund bergantung penyedia `[TIDAK DIKETAHUI]`; mekanisme teknis di Tahap 4–5. Pilihan: penyebab menanggung (rekomendasi pewawancara); selalu refund penuh, Rumah Sehat menanggung biaya; tunggu keputusan CEO)
Jawaban: "Penyebab tanggung (Recommended)".
Catatan pewawancara: isi pilihan yang dipilih:
- *Layanan gagal* (tenaga kesehatan tidak hadir, rekanan membatalkan, barang tidak dikirim atau rusak): pasien menerima kembali seluruh pembayaran termasuk ongkir dan biaya transaksi. Bagian rekanan dikembalikan rekanan; markup dikembalikan Rumah Sehat; biaya payment gateway yang tidak kembali ditanggung pihak penyebab kegagalan (rekanan, atau Rumah Sehat bila penyebabnya gangguan platform).
- *Pembatalan pasien sebelum batas waktu*: pasien menerima kembali harga layanan (harga rekanan + markup); biaya transaksi non-QRIS tidak dikembalikan; MDR QRIS tetap ditanggung rekanan.
PT-27 terjawab untuk aturan bisnis; mekanisme penarikan dana di Tahap 4–5; konfirmasi legal tetap lewat L-04 dan L-14.

**T3-Q8. Siapa yang memutuskan refund dan berapa nilai awal batas waktunya?** (konteks: BR-045. Usulan pembagian: refund otomatis untuk kejadian tercatat sistem (pembatalan pasien sebelum batas waktu, pembatalan oleh rekanan atau tenaga kesehatan, tenaga kesehatan tidak bergabung sampai batas toleransi, pesanan tidak diproses outlet sampai batas waktu, kurir gagal kirim); refund lewat pengajuan untuk klaim yang perlu bukti (barang rusak atau tidak sesuai, dengan foto). Semua angka menjadi parameter admin. Pilihan: otomatis + admin Rumah Sehat (rekomendasi pewawancara); otomatis + rekanan ikut memutuskan; semua lewat admin Rumah Sehat)
Jawaban: "Otomatis + admin RS (Recommended)".
Catatan pewawancara: nilai awal yang disetujui: toleransi tenaga kesehatan tidak hadir 15 menit; batas pengajuan klaim barang 2 x 24 jam setelah barang diterima; keputusan admin paling lambat 3 hari kerja.

## Tema D: NFR

**T3-Q9. Di platform apa aplikasi tenaga kesehatan tersedia? (PT-08)** (konteks: P6.1, P7.4; terpisah atau digabung dengan aplikasi pasien diputuskan di Tahap 4. Pilihan: Android dan iOS (rekomendasi pewawancara); Android saja; iOS saja)
Jawaban: "Android dan iOS (Recommended)".

**T3-Q10. Zona waktu apa yang ditampilkan untuk jadwal konsultasi, klinik, lab, dan home visit? (PT-15)** (konteks: P6.5; penyimpanan waktu dalam satu acuan baku adalah keputusan teknis Tahap 4. Pilihan: ikut lokasi layanan (rekomendasi pewawancara); semua WIB)
Jawaban: "Ikut lokasi layanan (Recommended)".
Catatan pewawancara: isi pilihan yang dipilih: jadwal klinik, lab, dan home visit dalam zona waktu lokasi rekanan dengan label zona (misalnya "10.00 WITA"); konsultasi online dalam zona waktu perangkat pengguna; laporan portal rekanan dalam zona waktu rekanan; laporan Rumah Sehat dalam WIB.

**T3-Q11. Kapasitas berapa yang dipakai sebagai target desain dan uji beban? (PT-10)** (konteks: P6.3; KPI-01 10.000, KPI-02 70; usulan satu tenaga kesehatan melayani satu sesi pada satu waktu. Angka pilihan adalah usulan Tech Lead yang menjadi target keputusan user, bukan prediksi pasar. Pilihan: 3 x KPI (rekomendasi pewawancara); setara KPI; tunggu angka CEO/CTO)
Jawaban: "3 x KPI (Recommended)".
Catatan pewawancara: target desain dan uji beban: 30.000 pasien aktif bulanan, 1.000 pengguna aktif bersamaan, 100 sesi konsultasi (chat/video) bersamaan. PT-10 terjawab sebagai target desain; volume riil tetap tidak diketahui.

**T3-Q12. Berapa target ketersediaan bulanan layanan inti Rumah Sehat (di luar pemeliharaan terjadwal dan gangguan pihak ketiga)? (PT-12)** (konteks: P6.4, BR-017. Cara ukur yang diusulkan: bulanan untuk layanan inti (login, pemesanan, pembayaran, konsultasi, portal); di luar pemeliharaan terjadwal yang diumumkan paling lambat 48 jam sebelumnya di jam sepi; gangguan sistem rekanan dan penyedia pihak ketiga dicatat terpisah. Pilihan: 99,5% (rekomendasi pewawancara); 99,9%; 99,0%)
Jawaban: "99,0%".
Catatan pewawancara: target 99,0% per bulan (sekitar 7,2 jam gangguan per 30 hari), dengan cara ukur yang diusulkan. PT-12 terjawab.

**T3-Q13. Paket target performa dan pemulihan mana yang dipakai di NFR?** (tabel tiga paket ditampilkan; waktu tersambung video dan waktu sampai notifikasi di perangkat dikecualikan karena bergantung penyedia. Pilihan: standar (rekomendasi pewawancara); ketat; longgar)
Jawaban: "Standar (Recommended)".
Catatan pewawancara: paket standar: respons API baca 500 ms dan tulis 1 detik (p95, tanpa panggilan pihak ketiga); pesan chat sampai ke penerima yang online 2 detik (p95); halaman portal tampil 3 detik (p95); notifikasi diserahkan ke penyedia 1 menit setelah kejadian; RPO 15 menit; RTO 4 jam.

## Klarifikasi CR-006: posisi portal dan RME minimalis

**T3-Q14. Apakah klinik/lab yang melayani lewat portal Rumah Sehat (tanpa integrasi sistem) dihitung "terintegrasi" untuk syarat go-live 1 (min. 2 klinik, 1 lab) dan KPI-03/KPI-04?** (konteks: keputusan lama brief P8.2 dan T2-Q11 memakai kata "terintegrasi". Pilihan: dihitung (rekomendasi pewawancara); tidak dihitung)
Jawaban pertama (teks bebas): "portal rumah sehat itu apa?"
Catatan pewawancara: pewawancara menjelaskan bahwa portal Rumah Sehat adalah situs web untuk admin rekanan (BR-007, S-13, brief P6.2), lalu membandingkan dua cara data masuk (integrasi sistem otomatis dan isi manual di portal), dan mengulang pertanyaan.
Jawaban kedua (teks bebas): "web rumah sehat untuk rekanan hanya perihal pelaporan atau rekap saja, bukan media untukrekam medis. rekam medis itu terpisah dengan rumah sehat web. ingat rumah sehat itu hanya platform untuk mengintegrasikan antara pasien dan klinik".
Catatan pewawancara: pewawancara mengakui salah tafsir di T3-Q1b (fitur RME ditulis "di portal Rumah Sehat") dan menyatakan pemahaman baru: (1) Rumah Sehat hanya platform yang mengintegrasikan pasien dengan rekanan, prinsip gateway BRD tetap; (2) portal Rumah Sehat untuk rekanan hanya untuk pelaporan dan rekap (BR-064, BR-065); (3) RME minimalis adalah sistem terpisah dari Rumah Sehat yang menjadi "sistem rekanan" bagi klinik/lab tanpa sistem, berisi keempat kelompok fitur T3-Q1b, dan terhubung ke Rumah Sehat lewat integrasi yang sama dengan sistem rekanan lain. Akibatnya T3-Q14 gugur: klinik/lab yang memakai RME minimalis terhubung lewat integrasi sehingga terhitung "terintegrasi" untuk syarat go-live 1 dan KPI-03/04; konsekuensi ini disampaikan ke user untuk dikonfirmasi di rekap gate.

**T3-Q15. RME minimalis (sistem terpisah untuk klinik/lab tanpa sistem, terhubung ke Rumah Sehat lewat integrasi) dibangun di mana?** (pilihan: di proyek ini sebagai aplikasi terpisah untuk rilis 1, dijelaskan di chat sebagai "aplikasi RME minimalis (web untuk klinik/lab) dibangun tim yang sama dan ikut target go-live 1 Maret 2027", dengan konsekuensi beban jadwal naik besar dan PT. Arunika berposisi sebagai penyedia sistem RME yang mendaftarkan sistemnya ke SATUSEHAT dan wajib memiliki nomor PSE; proyek terpisah di luar scope Rumah Sehat; ditunda)
Jawaban: "Proyek ini, rilis 1".
Catatan pewawancara: RME minimalis dibangun di proyek ini sebagai aplikasi terpisah dari aplikasi dan portal Rumah Sehat, ikut target go-live 1 Maret 2027, dan masuk PRD sebagai modul tersendiri.

**T3-Q16. Portal rekanan hanya untuk laporan/rekap. Lalu tarif dan durasi konsultasi online diatur di mana? (mengubah T3-Q4)** (konteks: jawaban T3-Q4 menempatkan pengisian tarif dan durasi serta pengubahan jadwal oleh admin klinik di portal, yang berbenturan dengan prinsip di jawaban kedua T3-Q14. Pilihan: lewat integrasi (rekomendasi pewawancara); lewat admin Rumah Sehat; tetap seperti T3-Q4 sebagai pengecualian)
Jawaban: "Tetap T3-Q4 (pengecualian)".
Catatan pewawancara: T3-Q4 tetap berlaku. Portal rekanan memiliki satu menu pengaturan konsultasi online (tarif, durasi, jadwal tenaga kesehatan); selebihnya portal hanya untuk laporan dan rekap. Check-in pasien, antrean, rekam medis, hasil lab, dan home visit berjalan lewat integrasi dengan sistem klinik/lab sendiri atau dengan RME minimalis.

## Akhir wawancara

Wawancara Tahap 3 (T3-Q1 sampai T3-Q16, termasuk Q1a, Q1b, Q1c, Q3a) selesai 2026-11-17. PRD disusun di `docs/03-prd.md`.
