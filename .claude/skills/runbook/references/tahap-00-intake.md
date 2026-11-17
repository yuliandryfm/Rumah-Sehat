# Tahap 0: Intake

Tujuan: mengubah ide mentah menjadi **Project Brief** 1–2 halaman yang disepakati user. Brief ini menjadi bahan baku semua tahap berikutnya.

## Langkah

1. Tanyakan ke user (pakai AskUserQuestion kalau tersedia) jalur mana yang dipakai:
    - **Jalur A: dokumen terlampir.** User sudah punya proposal, TOR, deck, catatan rapat, mockup, atau dokumentasi sistem lama.
    - **Jalur B: wawancara.** Idenya belum tertulis.
2. Jalankan jalur yang dipilih (lihat di bawah).
3. Susun Project Brief memakai template di `templates.md`, simpan di `docs/00-project-brief.md`. **Setiap butir diberi label `[USER]` atau `[DOK: nama-file]`** sesuai `anti-halu.md`. Butir yang belum terjawab ditulis `[TIDAK DIKETAHUI]`, jangan diisi dugaan.
4. Simpan catatan wawancara apa adanya (pertanyaan dan jawaban user) di `docs/00-catatan-intake.md`. File ini menjadi bukti untuk setiap label `[USER]`.
5. Tampilkan daftar pertanyaan terbuka, minta user mengonfirmasi atau mengoreksi isi brief.

Tahap 0 **selalu interaktif**, juga di Mode Otomatis. Ini satu-satunya sesi tanya-jawab panjang, jadi gali sampai tuntas di sini supaya tahap berikutnya tidak perlu bertanya lagi.

## Jalur A: dokumen terlampir

1. Minta user menaruh dokumen di folder `docs/input/` atau menyebut path-nya dengan `@`.
2. Baca semua dokumen, lalu petakan isinya ke checklist kelengkapan di bawah. Tandai setiap butir: **lengkap**, **sebagian**, atau **kosong**.
3. Tampilkan hasil pemetaan ke user dalam bentuk tabel.
4. Untuk butir yang sebagian atau kosong, beralih ke wawancara Jalur B, **hanya untuk kelompok pertanyaan yang belum terjawab**.

## Checklist kelengkapan

1. Masalah yang diselesaikan dan untuk siapa
2. Target pengguna dan peran-perannya
3. Fitur utama dan alur kerja inti
4. Data yang dikelola dan sistem lain yang harus terintegrasi
5. Kebutuhan kepatuhan, keamanan, dan audit trail
6. Kebutuhan non-fungsional: jumlah pengguna, ketersediaan, platform
7. Batasan: anggaran, tenggat, tim, teknologi yang diwajibkan
8. Ukuran sukses (KPI) dan definisi go-live
9. Pihak yang menyetujui hasil (sign-off)

## Jalur B: wawancara terstruktur

Aturan:

- Satu kelompok tema per putaran, 3–5 pertanyaan sekaligus.
- Setelah setiap putaran, rangkum jawaban dan minta konfirmasi sebelum lanjut.
- Jawaban "belum tahu" dicatat sebagai pertanyaan terbuka, bukan ditebak.
- Boleh mengusulkan opsi, tapi keputusan tetap di tangan user.
- Jangan tanyakan hal yang sudah dijawab di dokumen (kalau datang dari Jalur A).

Kelompok pertanyaan:

1. **Masalah dan tujuan.** Masalah apa yang ingin diselesaikan? Bagaimana orang mengatasinya sekarang? Apa yang terjadi kalau aplikasi ini tidak dibuat?
2. **Pengguna dan stakeholder.** Siapa pengguna utamanya? Peran apa saja yang ada (admin, operator, nasabah, approver)? Siapa yang membiayai dan siapa yang menyetujui?
3. **Fitur dan alur utama.** Tiga hal terpenting yang harus bisa dilakukan pengguna di hari pertama? Bagaimana alur kerja dari awal sampai akhir? Fitur apa yang jelas tidak termasuk?
4. **Data dan integrasi.** Data apa yang disimpan? Mana yang sensitif (data pribadi, finansial)? Sistem apa yang harus terhubung (core banking, payment gateway, SSO, notifikasi)?
5. **Kepatuhan, keamanan, dan audit.** Regulasi atau standar apa yang berlaku? Siapa yang akan mengaudit? Aktivitas apa yang wajib tercatat di audit trail?
6. **Non-fungsional.** Perkiraan jumlah pengguna dan transaksi? Platform (web, mobile, keduanya)? Target ketersediaan dan jam operasional?
7. **Batasan.** Anggaran, tenggat, dan siapa saja yang mengerjakan? Ada teknologi atau infrastruktur yang wajib dipakai atau dilarang?
8. **Sukses dan go-live.** Bagaimana kita tahu aplikasi ini berhasil? Apa syarat minimum untuk rilis pertama?

## Gate Tahap 0

- [ ] Project Brief tersimpan di `docs/00-project-brief.md`, setiap butir berlabel `[USER]`, `[DOK]`, atau `[TIDAK DIKETAHUI]`
- [ ] Catatan intake tersimpan di `docs/00-catatan-intake.md`
- [ ] Semua butir checklist terisi atau tercatat sebagai pertanyaan terbuka beserta pemiliknya
- [ ] Gate anti-halu terpenuhi (lihat `anti-halu.md` bagian 8)
- [ ] User mengonfirmasi isi brief dan pertanyaan terbuka (Checkpoint 1)
