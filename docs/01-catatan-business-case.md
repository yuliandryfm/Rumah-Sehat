# Catatan Wawancara Tahap 1 (Business Case)

Catatan apa adanya. File ini menjadi bukti untuk setiap label `[USER] T1-Qx` di `docs/01-business-case.md`.

Tanggal: 2026-11-17
Pewawancara: Claude (persona Project Manager + Business Analyst)
Narasumber: user (project manager; persetujuan diteruskan ke CEO dan CTO, lihat brief "Stakeholder dan sign-off")
Cara bertanya: satu pertanyaan per giliran, sesuai arahan user di Tahap 0.

**Q1. Dari mana Rumah Sehat mendapat pemasukan?** (pilihan ganda: komisi konsultasi; komisi produk; biaya dari rekanan, contoh: langganan atau biaya per pendaftaran yang ditagihkan ke klinik/lab; belum ditentukan)
Jawaban: "Komisi konsultasi, Komisi produk, Biaya dari rekanan".

**Q2. Apakah besaran komisi konsultasi, komisi produk, dan biaya rekanan sudah ditentukan?** (pilihan: belum ada angka; buat tabel simulasi yang ditandai bukan keputusan, angka final oleh CEO; sudah ada angka)
Jawaban: "Buat tabel simulasi".
Catatan pewawancara: pewawancara menyampaikan bahwa simulasi dibuat berbasis rumus (pendapatan per nilai transaksi pada beberapa persentase), tanpa volume transaksi karangan; volume riil tetap [TIDAK DIKETAHUI] (PT-10).

**Q3. "10.000 user aktif" dihitung harian atau bulanan, dan apakah "70 dokter" termasuk psikolog?** (pilihan: bulanan, 70 dokter saja; bulanan, termasuk psikolog; harian; belum tahu. Teks pertanyaan di chat menjelaskan "bulanan" sebagai "10.000 pasien yang memakai aplikasi minimal sekali dalam sebulan")
Jawaban: "Bulanan, incl. psikolog".
Catatan pewawancara: kriteria "memakai" (cukup membuka aplikasi/login, atau harus bertransaksi) tidak dipilih user; dicatat sebagai pertanyaan terbuka PT-16.

**Q4. Apa alasan go-live harus 1 Maret 2027?** (pilihan: komitmen investor/pemilik; komitmen ke rekanan; rencana pemasaran; target internal)
Jawaban: "Rencana pemasaran".

**Q5. Untuk perkiraan biaya Business Case, angka apa yang bisa Anda berikan?** (pilihan: struktur biaya saja; biaya developer per bulan; batas anggaran total; keduanya)
Jawaban: "Biaya dev per bulan" (angka belum ditulis, ditanyakan di Q5a).
Catatan pewawancara: batas anggaran total tidak diberikan; anggaran tetap [TIDAK DIKETAHUI] (PT-11).

**Q5a. Berapa biaya mobile developer per bulan?** (pilihan rentang: < Rp10 juta; Rp10–20 juta; Rp20–30 juta; > Rp30 juta; Other untuk angka pasti dan biaya tim lain)
Jawaban: "Rp10–20 juta".
Catatan pewawancara: biaya tim lain (PM, langganan Claude) tidak disebut, dicatat [TIDAK DIKETAHUI].

**Q6. Apa nama badan hukum perusahaan IT pemilik Rumah Sehat?** (pilihan: geser ke Tahap 2; tulis nama)
Jawaban di dialog: "Tulis nama (Other)" tanpa nama; pewawancara meminta nama diketik langsung.
Jawaban (teks): "PT. Arunika".
Catatan pewawancara: PT-01 terjawab. Ditulis persis seperti jawaban user.

**Q7. Apakah perusahaan pernah mempertimbangkan opsi selain membangun sendiri (mis. beli/sewa platform white-label)?** (pilihan: sudah, memilih bangun sendiri; belum pernah; terbuka untuk opsi beli)
Jawaban: "Sudah, pilih bangun".

**Q7a. Apa alasan memilih membangun sendiri?** (pilihan ganda: kendali produk; kepemilikan kode dan data; biaya jangka panjang; kapabilitas perusahaan IT)
Jawaban: "Kepemilikan kode & data, Kendali produk, Biaya jangka panjang, Kapabilitas perusahaan IT".

**Q8. Berapa toleransi biaya: seberapa jauh biaya aktual boleh melebihi perkiraan sebelum wajib dieskalasi ke Anda/CEO/CTO?** (pilihan: +10%; +20%; +30%; belum ditentukan. Konteks yang disampaikan: toleransi waktu dan scope sudah ditetapkan di P8.3)
Jawaban: "+20%".

## Gate Tahap 1

2026-11-17. Rekap jawaban Q1–Q8 ditampilkan, beserta hal yang disusun penyusun (tenggat KPI 1 Maret 2028, parameter simulasi KEPUTUSAN-01, penilaian risiko KEPUTUSAN-02, biaya developer berlaku bila dibayar penuh/PT-19). Pertanyaan: apakah Executive menyatakan Rumah Sehat layak dilanjutkan dan menyetujui Business Case?
Jawaban (T1-Gate): "Layak, setujui".
