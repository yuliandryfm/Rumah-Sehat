# Riset Kepatuhan Tahap 2 (Fase 1): Rumah Sehat

Tanggal: 2026-11-17
Penyusun: compliance reviewer (Claude). **Ini bukan nasihat hukum.** Laporan ini memetakan kebutuhan kepatuhan dari sumber resmi yang benar-benar terbuka dan terbaca. Kesimpulan apakah sebuah kewajiban berlaku bagi Rumah Sehat ditulis sebagai **indikasi dari sumber**, dan wajib dikonfirmasi tim legal manusia.
Tingkat ketelitian: T1 Kritis (regulasi, data pribadi, data kesehatan, alur uang).
Konteks produk: `[DOK: 00-project-brief.md]` (Scope, Data dan integrasi, Kepatuhan), `[DOK: 01-business-case.md]` (R-03).

## Keterbatasan riset (wajib dibaca)

- Sebagian besar teks pasal regulasi kesehatan hanya tersedia sebagai PDF yang **tidak terbaca** oleh alat riset (PP 28/2024, UU 17/2023, Permenkes 5/2026, Permenkes 24/2022, Peraturan BPOM 14/2024, lampiran PSEF). Untuk regulasi itu, laporan ini hanya memakai metadata/abstrak resmi JDIH BPK dan pernyataan resmi di situs Kemenkes. Isi pasalnya ditulis `[TIDAK DIKETAHUI]`. Rinciannya ada di `docs/sources.md` bagian "Pencarian tanpa hasil".
- Dua sumber Kemenkes tentang telemedisin (SRC-02, SRC-03) membahas **rancangan** peraturan (RPP 2023 dan RPMK 2024). Permenkes final tentang telemedisin pelaksana PP 28/2024 **tidak ditemukan** pada 2026-11-17. Kemenkes menyatakan pada November 2026 akan mengatur seluruh platform kesehatan digital `[SRC-06]`, sehingga aturan baru bisa terbit sebelum go-live 1 Maret 2027.
- Teks pasal UU 27/2022 (PDP), PP 71/2019, dan Permenkominfo 5/2020 terbaca langsung di JDIH Komdigi. Temuan di area itu memiliki keyakinan lebih tinggi.

## (a) Tabel temuan

Tingkat keyakinan: **Tinggi** = teks pasal resmi terbaca. **Sedang** = pernyataan resmi regulator, tetapi bukan teks pasal final. **Rendah** = hanya metadata atau sumber yang tidak langsung menjawab.

### Pertanyaan 1 (PT-02): badan usaha, izin, dan pendaftaran

| # | Pertanyaan | Temuan singkat | Sumber | Keyakinan | Perlu konfirmasi legal |
| --- | --- | --- | --- | --- | --- |
| 1.1 | Siapa yang boleh menyelenggarakan telemedisin? | Menurut Kemenkes, telemedisin diselenggarakan oleh **fasyankes**, secara mandiri atau bekerja sama dengan **PSE yang terdaftar**. RPMK 2024 (pelaksana PP 28/2024 Pasal 557 ayat 5, 561 ayat 5, 562 ayat 3, 564) menyebut penyelenggara "harus teregistrasi di Kemenkes" dan didaftarkan sebagai PSE. Kepemilikan aplikasi bisa "pemerintah, swasta atau mandiri". **Indikasi:** PT. Arunika (perusahaan IT, bukan fasyankes `[DOK] P2.2, P5.2a`) kemungkinan berperan sebagai PSE yang bekerja sama dengan fasyankes. Model **dokter/psikolog mitra langsung Rumah Sehat** `[DOK] PS6e` (tanpa fasyankes) belum jelas tempatnya | `[SRC-02]` `[SRC-03]` `[SRC-04]` | Sedang (pernyataan atas rancangan; teks pasal PP 28/2024 tidak terbaca) | **Ya, kritis** |
| 1.2 | Apakah harus badan usaha khusus? | Tidak ada sumber terbuka yang menyatakan platform telemedisin wajib berbentuk badan usaha khusus. Bentuk badan usaha dan KBLI untuk Rumah Sehat `[TIDAK DIKETAHUI]`. Standar kegiatan usaha subsektor kesehatan kini diatur Permenkes 11/2025 (pelaksana PP 28/2025 tentang perizinan berusaha berbasis risiko), yang mencabut Permenkes 14/2021 dan perubahannya | `[SRC-03]` `[SRC-09]` | Rendah | **Ya** |
| 1.3 | Pendaftaran PSE | Setiap PSE Lingkup Privat **wajib mendaftar sebelum sistem mulai digunakan**. Sanksi bagi yang tidak mendaftar: pemutusan akses (*access blocking*). PSE Lingkup Privat mencakup penyelenggara aplikasi daring untuk kegiatan tertentu (termasuk perdagangan dan pemrosesan data pribadi). **Indikasi kuat:** Rumah Sehat termasuk dan wajib mendaftar sebelum go-live | `[SRC-12]` `[SRC-13]` `[SRC-14]` | Tinggi (kewajiban); Sedang (kategori Rumah Sehat) | Ya (konfirmasi kategori) |
| 1.4 | Regulatory sandbox Kemenkes | Kemenkes meminta penyedia telekesehatan mendaftar *regulatory sandbox* (Kepmenkes HK.01.07/MENKES/1280/2023). Artikel resmi **tidak menyatakan wajib**. Pada November 2026 Kemenkes menyatakan akan mengatur seluruh platform kesehatan digital, termasuk telemedisin, rekam medis, SIP, dan **penetapan tarif** | `[SRC-05]` `[SRC-06]` | Tinggi (isi pernyataan); Rendah (sifat wajib) | Ya |
| 1.5 | PSEF (bila menjual obat) | Syarat pendaftaran PSEF di portal Kemenkes: NIB, Tanda Daftar PSE, izin usaha IUI/PMSE, **apoteker penanggung jawab**, **komitmen kerja sama dengan apotek**, dan **daftar apotek jaringan**. Lihat 2.2 | `[SRC-08]` `[SRC-09]` | Tinggi (daftar syarat); Rendah (apakah wajib bagi Rumah Sehat) | **Ya, kritis** |

### Pertanyaan 2 (PT-04): penjualan obat bebas oleh klinik lewat platform

| # | Pertanyaan | Temuan singkat | Sumber | Keyakinan | Perlu konfirmasi legal |
| --- | --- | --- | --- | --- | --- |
| 2.1 | Regulasi peredaran daring | Peredaran obat, obat bahan alam, obat kuasi, suplemen kesehatan, **kosmetik**, dan pangan olahan secara daring diatur Peraturan BPOM 14/2024 (berlaku 5 Agustus 2024), diubah Peraturan BPOM 30/2025 (Pasal 3, 5, 28A; berlaku 3 November 2025). Jadi **ketiga jenis produk** di layanan 7 (obat bebas, produk kecantikan, produk kesehatan) masuk ruang lingkupnya. Siapa yang boleh mengedarkan obat bebas secara daring, apakah klinik termasuk, dan kewajiban platform: `[TIDAK DIKETAHUI]` (PDF tidak terbaca) | `[SRC-10]` `[SRC-11]` | Tinggi (ruang lingkup); isi pasal `[TIDAK DIKETAHUI]` | **Ya, kritis** |
| 2.2 | Syarat bagi platform | **Indikasi:** kalau penjualan obat bebas di Rumah Sehat termasuk lingkup PSEF, model saat ini berpotensi tidak memenuhi syarat. Syarat PSEF meliputi apoteker penanggung jawab serta kerja sama dan jaringan apotek `[SRC-08]`, padahal apotek bukan outlet rekanan dan peran apoteker tidak ada di rilis pertama `[DOK] PS2, PS2a, P3.5, PS3` | `[SRC-08]` `[DOK]` | Sedang–Rendah | **Ya, kritis** |
| 2.3 | Perdagangan elektronik | Permendag 31/2023 sudah tidak berlaku, dicabut Permendag 19/2026 (berlaku 8 Juni 2026). Permendag 19/2026 mengatur pelaku usaha dan model bisnis PPMSE, persyaratan kegiatan usaha PMSE, dan perlindungan konsumen. Kewajiban rinci bagi Rumah Sehat `[TIDAK DIKETAHUI]` | `[SRC-22]` `[SRC-23]` | Rendah | Ya |

### Pertanyaan 3: penerusan dana ke outlet/mitra

| # | Pertanyaan | Temuan singkat | Sumber | Keyakinan | Perlu konfirmasi legal |
| --- | --- | --- | --- | --- | --- |
| 3.1 | Perlu izin sistem pembayaran? | "Setiap pihak dilarang menyelenggarakan aktivitas Sistem Pembayaran sebelum memperoleh izin atau penetapan dari BI." Aktivitas PJP meliputi penatausahaan sumber dana, penyediaan informasi sumber dana, *payment initiation* dan/atau *acquiring*, serta remitansi | `[SRC-19]` `[SRC-20]` | Tinggi (isi ringkasan resmi BI) | Ya |
| 3.2 | Penerusan dana oleh platform | BI "dapat menetapkan aspek prudensial kepada Penyelenggara Penunjang yang melakukan **penerusan pembayaran dari PJP kepada Penyedia Barang dan/atau Jasa**", termasuk jangka waktu penampungan dana. **Indikasi:** model Rumah Sehat (pasien membayar di aplikasi, lalu dana diteruskan ke outlet `[DOK] P3.3, PS4`) mirip uraian ini. Masih `[TIDAK DIKETAHUI]`: apakah Rumah Sehat tergolong Penyelenggara Penunjang yang memerlukan izin atau penetapan BI, atau cukup memakai PJP berizin yang menyalurkan dana langsung ke rekening mitra | `[SRC-20]` | Sedang | **Ya, kritis** |

### Pertanyaan 4 (PT-03): data pribadi, rekam medis, resep, tenaga kesehatan, pengawas

| # | Pertanyaan | Temuan singkat | Sumber | Keyakinan | Perlu konfirmasi legal |
| --- | --- | --- | --- | --- | --- |
| 4.1 | Kategori data spesifik | Data spesifik meliputi "data dan informasi kesehatan", "data anak", dan data keuangan pribadi (Pasal 4 ayat 2). Catatan dokter/psikolog, isi chat konsultasi, resep, dan hasil lab `[DOK] P4.1` termasuk data kesehatan. Data anggota keluarga yang masih anak termasuk data anak | `[SRC-15]` | Tinggi (kategori); Sedang (pemetaan ke data Rumah Sehat) | Ya (pemetaan) |
| 4.2 | Dasar pemrosesan dan persetujuan | Pengendali wajib punya dasar pemrosesan, antara lain "persetujuan yang sah secara eksplisit" (Pasal 20). Informasi yang wajib disampaikan saat meminta persetujuan diatur di Pasal 21, termasuk tujuan dan masa retensi. Persetujuan "tertulis atau terekam" (Pasal 22). Data anak wajib "persetujuan dari orang tua anak dan/ atau wali anak" (Pasal 25) | `[SRC-15]` | Tinggi | Ya (dasar pemrosesan per data) |
| 4.3 | Hak subjek data | Hak atas informasi, perbaikan, akses dan salinan, penghapusan/pemusnahan, penarikan persetujuan, keberatan atas keputusan otomatis, penundaan/pembatasan, gugatan ganti rugi, dan portabilitas (Pasal 5–13). Batas 3 x 24 jam berlaku untuk perbaikan (Pasal 30 ayat 1), akses (Pasal 32 ayat 2), penghentian pemrosesan setelah persetujuan ditarik (Pasal 40 ayat 2), serta penundaan/pembatasan (Pasal 41 ayat 1) | `[SRC-15]` | Tinggi | Ya (benturan hak hapus dengan retensi rekam medis) |
| 4.4 | Kewajiban pengendali | Wajib merekam seluruh kegiatan pemrosesan (Pasal 31), melakukan penilaian dampak bila berisiko tinggi (Pasal 34), menjaga keamanan dan kerahasiaan (Pasal 35–36), serta mengakhiri, menghapus, dan memusnahkan data dalam kondisi tertentu, termasuk saat masa retensi berakhir (Pasal 42–44). Pejabat/petugas PDP wajib ditunjuk dalam kondisi Pasal 53 ayat (1), termasuk pemrosesan skala besar atas data spesifik. Menurut Putusan MK 151/PUU-XXII/2024, kata "dan" pada huruf b dibaca "dan/atau". Masa penyesuaian 2 tahun sejak 17 Oktober 2022 (Pasal 74) **sudah berakhir** | `[SRC-15]` `[SRC-16]` | Tinggi | Ya (wajib DPO/DPIA bagi Rumah Sehat) |
| 4.5 | Notifikasi kegagalan pelindungan | Pemberitahuan tertulis "paling lambat 3 x 24 (tiga kali dua puluh empat) jam" kepada subjek data dan lembaga, dan dalam hal tertentu kepada masyarakat (Pasal 46). PP 71/2019 Pasal 14 ayat (5) juga mewajibkan pemberitahuan tertulis kepada pemilik data | `[SRC-15]` `[SRC-13]` | Tinggi | Tidak (kewajiban jelas); ya untuk prosedur |
| 4.6 | Lokasi data | Ketentuan umum membolehkan PSE Lingkup Privat mengelola data di luar Indonesia dengan syarat (PP 71/2019 Pasal 21). Transfer data pribadi ke luar wilayah hukum Indonesia bersyarat (UU PDP Pasal 56). Kewajiban lokasi khusus sektor kesehatan `[TIDAK DIKETAHUI]`. Keputusan bisnis "data wajib di Indonesia" `[DOK] P5.5` lebih ketat daripada ketentuan umum yang ditemukan. **Catatan:** penyedia pihak ketiga (video, email, WhatsApp, push, login Google/Apple `[DOK]`) bisa memproses data di luar negeri, dan itu harus dicek terhadap keputusan ini dan Pasal 56 | `[SRC-13]` `[SRC-15]` | Tinggi (ketentuan umum); `[TIDAK DIKETAHUI]` (sektor kesehatan) | Ya |
| 4.7 | Rekam medis dan kerahasiaan | PMK 24/2022 mewajibkan fasyankes menjalankan rekam medis elektronik dengan prinsip keamanan dan kerahasiaan, terhubung dengan SATUSEHAT. Kemenkes (Nov 2026): telemedisin "harus terintegrasi dengan pencatatan rekam medis". PP 28/2024 mencabut PP 10/1966 tentang Wajib Simpan Rahasia Kedokteran; ketentuan penggantinya tidak terbaca. **`[TIDAK DIKETAHUI]`:** masa retensi rekam medis; apakah catatan konsultasi di Rumah Sehat berstatus rekam medis dan siapa pengendalinya (fasyankes atau PT. Arunika); kewajiban integrasi SATUSEHAT bagi platform | `[SRC-18]` `[SRC-06]` `[SRC-04]` | Sedang | **Ya, kritis** |
| 4.8 | Resep elektronik | Permenkes 5/2026 tentang Perbekalan Kesehatan berlaku sejak 4 Mei 2026, tetapi isinya tentang resep elektronik `[TIDAK DIKETAHUI]` (PDF tidak terbaca). Kewajiban format, tanda tangan, integrasi, dan penebusan resep di apotek mana pun `[DOK] P3.4` juga `[TIDAK DIKETAHUI]` | `[SRC-07]` | Rendah | **Ya** |
| 4.9 | Verifikasi STR/SIP | Menurut Kemenkes, SDM telemedisin wajib memiliki SIP dan STR. SIP "diterbitkan pemerintah kabupaten/kota" tempat praktik, dan permohonannya melampirkan STR serta surat keterangan tempat praktik. **`[TIDAK DIKETAHUI]`:** kanal resmi verifikasi STR/SIP, status psikolog (apakah wajib STR/SIP), dan SIP mana yang berlaku untuk praktik telemedisin dokter mitra langsung | `[SRC-02]` `[SRC-03]` `[SRC-21]` | Sedang | Ya |
| 4.10 | Instansi pengawas | Kemenkes: telemedisin, rekam medis, perbekalan kesehatan, PSEF, perizinan subsektor kesehatan `[SRC-03]` `[SRC-06]` `[SRC-08]` `[SRC-09]`. Komdigi: pendaftaran PSE `[SRC-12]`. BPOM: obat dan makanan daring `[SRC-10]`. BI: sistem pembayaran `[SRC-20]`. Kemendag: PMSE `[SRC-23]`. Lembaga PDP: penyelenggaraan PDP (Pasal 58), status pembentukannya `[TIDAK DIKETAHUI]` `[SRC-15]`. Pemda kabupaten/kota: SIP `[SRC-21]` | lihat kolom temuan | Sedang | Ya |

## (b) Usulan kebutuhan kepatuhan untuk dijadikan BR

Nomor BR final ditetapkan penyusun BRD. Butir bertanda **(bergantung K-x)** menunggu keputusan di bagian (c).

| # | Usulan kebutuhan bisnis | Sumber |
| --- | --- | --- |
| C-01 | Rumah Sehat harus terdaftar sebagai PSE Lingkup Privat sebelum aplikasi digunakan pengguna (sebelum go-live) | `[SRC-12]` `[SRC-13]` |
| C-02 | Layanan konsultasi online harus berjalan dalam skema penyelenggara telemedisin yang sesuai ketentuan Kemenkes, yaitu fasyankes penyelenggara dengan Rumah Sehat sebagai PSE terdaftar dan registrasi di Kemenkes. Bentuk final **(bergantung K-1)** | `[SRC-02]` `[SRC-03]` |
| C-03 | Hanya dokter, psikolog, dan tenaga kesehatan dengan STR dan SIP yang masih berlaku yang dapat diaktifkan untuk melayani. Masa berlaku SIP dipantau, dan akun dinonaktifkan saat SIP berakhir | `[SRC-02]` `[SRC-03]` `[SRC-21]` |
| C-04 | Pasien memberikan persetujuan eksplisit (tertulis atau terekam) per tujuan pemrosesan sebelum datanya diproses. Informasi yang diwajibkan Pasal 21 UU PDP (termasuk tujuan dan masa retensi) ditampilkan, dan bukti persetujuan disimpan | `[SRC-15]` |
| C-05 | Data anggota keluarga yang masih anak hanya boleh diproses dengan persetujuan orang tua atau wali | `[SRC-15]` |
| C-06 | Data kesehatan (catatan dokter/psikolog, isi chat konsultasi, resep, hasil lab), data anak, dan data keuangan pribadi diperlakukan sebagai data pribadi spesifik dengan kontrol akses dan perlindungan lebih ketat | `[SRC-15]` |
| C-07 | Pasien dapat menjalankan hak subjek data: informasi, akses dan salinan, perbaikan, penarikan persetujuan, penghapusan, penundaan/pembatasan, keberatan, dan portabilitas. Permintaan yang diatur batas waktunya ditanggapi paling lambat 3 x 24 jam. Benturan hak hapus dengan kewajiban retensi rekam medis diputuskan legal | `[SRC-15]` |
| C-08 | Seluruh kegiatan pemrosesan data pribadi direkam. Pencatatan ini diselaraskan dengan audit trail append-only di brief `[DOK]` dan ADR-005 | `[SRC-15]` |
| C-09 | Rumah Sehat menjaga keamanan dan kerahasiaan data pribadi yang diprosesnya | `[SRC-15]` `[SRC-13]` `[SRC-02]` |
| C-10 | Tersedia prosedur pemberitahuan kegagalan pelindungan data pribadi, tertulis dan paling lambat 3 x 24 jam, kepada subjek data dan lembaga (serta masyarakat dalam hal tertentu) | `[SRC-15]` `[SRC-13]` |
| C-11 | Penilaian dampak pelindungan data pribadi dilakukan sebelum go-live, karena Rumah Sehat memproses data spesifik | `[SRC-15]` |
| C-12 | Rumah Sehat menunjuk pejabat/petugas pelindungan data pribadi. Kewajibannya bagi Rumah Sehat dikonfirmasi legal | `[SRC-15]` `[SRC-16]` |
| C-13 | Setiap jenis data punya masa retensi, dan data diakhiri pemrosesannya, dihapus, atau dimusnahkan sesuai ketentuan. Angka retensi (terutama rekam medis) `[TIDAK DIKETAHUI]`, ditetapkan setelah konfirmasi legal | `[SRC-15]` |
| C-14 | Data disimpan di Indonesia `[DOK] P5.5`. Transfer data pribadi ke luar negeri, termasuk ke penyedia pihak ketiga, hanya boleh bila memenuhi syarat Pasal 56 UU PDP dan disetujui user | `[SRC-15]` `[SRC-13]` |
| C-15 | Catatan konsultasi dan resep digital memenuhi ketentuan rekam medis elektronik dan resep elektronik yang berlaku, termasuk integrasi dengan sistem Kemenkes bila diwajibkan. Rincian `[TIDAK DIKETAHUI]` **(bergantung K-1)** | `[SRC-18]` `[SRC-06]` `[SRC-07]` |
| C-16 | Pembayaran di aplikasi hanya diproses melalui penyedia jasa pembayaran berizin BI. Model penerusan dana ke outlet dan mitra harus dikonfirmasi tidak memerlukan izin atau penetapan BI bagi PT. Arunika, atau izinnya diperoleh sebelum go-live **(bergantung K-3)** | `[SRC-19]` `[SRC-20]` |
| C-17 | Obat bebas, produk kecantikan, dan produk kesehatan yang dijual lewat Rumah Sehat memenuhi ketentuan peredaran daring BPOM, dan skema PSEF dipenuhi bila berlaku. Rincian `[TIDAK DIKETAHUI]` **(bergantung K-2)** | `[SRC-08]` `[SRC-10]` `[SRC-11]` |
| C-18 | PT. Arunika memiliki perizinan berusaha yang sesuai untuk kegiatan Rumah Sehat (subsektor kesehatan dan PMSE) sebelum go-live. Jenis izin dan KBLI `[TIDAK DIKETAHUI]` | `[SRC-09]` `[SRC-23]` |
| C-19 | Regulasi kesehatan digital dipantau sampai go-live, karena Kemenkes menyatakan akan mengatur seluruh platform kesehatan digital (termasuk tarif). Perubahan regulasi dicatat sebagai change request | `[SRC-06]` |

## (c) Keputusan yang harus diambil CEO/user

| # | Keputusan | Opsi (contoh, bukan rekomendasi hukum) | Memblokir PRD? |
| --- | --- | --- | --- |
| K-1 | Struktur penyelenggara konsultasi online | (a) PT. Arunika hanya sebagai PSE yang bekerja sama dengan fasyankes rekanan, dan semua dokter/psikolog berpraktik atas nama fasyankes. (b) PT. Arunika atau afiliasinya memiliki fasyankes (misalnya klinik) untuk menaungi dokter/psikolog mitra langsung. (c) Opsi lain dari legal | **Ya.** Menentukan peran, alur konsultasi, siapa pengendali rekam medis, siapa yang menerbitkan resep, dan status "mitra langsung" (PS6e). Berlaku untuk layanan 1 dan 2 |
| K-2 | Penjualan obat bebas di rilis pertama | (a) Lanjut dengan memenuhi skema yang dikonfirmasi legal (mungkin PSEF, apoteker, jaringan apotek). (b) Keluarkan obat bebas dari rilis pertama; produk kecantikan dan produk kesehatan tetap jalan setelah ketentuan BPOM/PMSE dikonfirmasi. (c) Tunda layanan 7 seluruhnya | **Ya, sebagian.** Memblokir bagian obat bebas di layanan 7. Layanan lain tidak terblokir |
| K-3 | Model penerusan dana (terkait PT-06) | (a) PJP berizin menyalurkan dana langsung ke rekening outlet/mitra, dan komisi dipotong oleh PJP (dana tidak ditampung PT. Arunika). (b) PT. Arunika menampung lalu meneruskan dana, dengan potensi kewajiban izin atau penetapan BI | **Ya** untuk alur uang (T1): pembayaran konsultasi, pembelian produk, refund, dan penerusan dana |
| K-4 | Anggaran dan waktu perizinan serta legal | Konsultan hukum, pendaftaran PSE, izin usaha, kemungkinan PSEF, DPO, DPIA. Masukan untuk tinjauan ulang Business Case di gate Tahap 2 | Tidak. Memengaruhi syarat go-live 5 yang tidak bisa dikurangi `[DOK] P8.3` |
| K-5 | Penyedia pihak ketiga dan lokasi pemrosesan data | Apakah keputusan "data di Indonesia" berlaku juga untuk penyedia video, email, WhatsApp, push, dan login | Tidak untuk PRD. Memblokir pemilihan penyedia di Tahap 4–5 |
| K-6 | Penunjukan pejabat/petugas PDP dan pemilik proses notifikasi insiden | Internal atau eksternal | Tidak |
| K-7 | Kebijakan retensi per jenis data | Ditetapkan setelah legal mengonfirmasi angka retensi rekam medis | Tidak untuk PRD (BR bisa memakai `[TIDAK DIKETAHUI]`), tetapi dibutuhkan sebelum model data di Tahap 4 |

## (d) Hal yang wajib dikonfirmasi ke tim legal/compliance manusia

1. **Telemedisin:** isi final PP 28/2024 Pasal 557–564 dan status Permenkes pelaksananya. Apakah PT. Arunika (bukan fasyankes) boleh menjalankan konsultasi online dokter dan psikolog, dan dalam skema apa. Bagaimana status dokter/psikolog mitra langsung tanpa fasyankes. Apakah registrasi di Kemenkes dan *regulatory sandbox* wajib.
2. **Perizinan berusaha:** KBLI dan jenis perizinan yang dibutuhkan PT. Arunika menurut Permenkes 11/2025, PP 28/2025, dan Permendag 19/2026. Apakah perlu badan usaha terpisah.
3. **Obat bebas (PT-04):** apakah klinik boleh menjual obat bebas secara daring lewat platform pihak ketiga menurut Peraturan BPOM 14/2024 jo. 30/2025 dan Permenkes 5/2026. Apakah Rumah Sehat wajib menjadi PSEF, mengingat syarat apoteker penanggung jawab dan jaringan apotek. Juga ketentuan bagi kosmetik, suplemen, dan alat kesehatan.
4. **Penerusan dana:** apakah model Rumah Sehat tergolong Penyelenggara Penunjang atau aktivitas sistem pembayaran yang memerlukan izin atau penetapan BI (PBI 23/6/PBI/2021, PBI 22/23/PBI/2020), atau cukup memakai PJP berizin. Konfirmasi juga ketentuan penampungan dana dan refund.
5. **Rekam medis dan resep:** apakah catatan konsultasi di Rumah Sehat berstatus rekam medis, siapa pengendalinya, berapa masa retensinya, apakah wajib terintegrasi dengan SATUSEHAT, ketentuan resep elektronik di Permenkes 5/2026, dan ketentuan pengganti PP 10/1966 tentang rahasia kedokteran.
6. **Hak hapus versus retensi:** bagaimana permintaan hapus data dari pasien (UU PDP Pasal 8 dan 43) diperlakukan terhadap kewajiban retensi rekam medis.
7. **Tenaga kesehatan:** apakah psikolog wajib STR/SIP, kanal resmi verifikasi STR/SIP, dan SIP mana yang berlaku untuk praktik telemedisin.
8. **PDP:** dasar pemrosesan per kategori data, kewajiban DPO dan DPIA bagi Rumah Sehat, status pembentukan lembaga PDP sebagai penerima notifikasi insiden, dan perlakuan data anggota keluarga (termasuk anak).
9. **Lokasi dan transfer data:** apakah ada kewajiban lokasi data khusus sektor kesehatan, dan syarat Pasal 56 UU PDP untuk penyedia pihak ketiga di luar negeri.
10. **Tarif:** apakah ada pengaturan tarif konsultasi atau komisi platform oleh Kemenkes `[SRC-06]` yang memengaruhi model pendapatan `[DOK: 01-business-case.md]` §5.
11. **Auditor dan regulator:** instansi mana yang akan mengaudit atau mengawasi `[DOK] P5.3`, sesuai daftar di temuan 4.10.
