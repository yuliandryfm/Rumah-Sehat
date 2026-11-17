# Tahap 5: Stack dan Coding Standards

Tujuan: memilih teknologi berdasarkan arsitektur dan NFR, lalu mengunci aturan menulis kode sebelum baris kode pertama ditulis.

Input: `docs/04-*.md`, `docs/adr/`.

## Aktivitas stack

- Pilih bahasa, framework, database, cache, queue, dan hosting, masing-masing dengan alasan dan versinya.
- **Gunakan web search** untuk mengecek versi stabil terbaru, status maintenance, lisensi, dan riwayat celah keamanan setiap dependensi utama. Jangan mengandalkan ingatan untuk versi.
- Terjemahkan model data logikal ke **model data fisikal**: tipe data sesuai DB yang dipilih, index, constraint, partisi, dan enkripsi kolom. Tambahkan ke `docs/04-erd.md` atau buat `docs/05-physical-schema.md`.
- Catat keputusan stack penting sebagai ADR.
- **Setiap versi, lisensi, dan status maintenance diberi label `[SRC-xx]`** dan dicatat di `docs/sources.md` (halaman registry atau dokumentasi resmi yang benar-benar dibuka hari itu).

Output: `docs/05-stack.md`.

## Tanggung jawab DevOps/Platform Engineer

Di tahap yang sama, dari sudut pandang DevOps/Platform Engineer, tetapkan:

- Hosting dan lingkungan (dev, staging, production), termasuk rencana pindah dari laptop ke VPS kalau relevan
- Rancangan CI/CD: tahapan pipeline yang menjalankan `scripts/verify.sh` dan cara deploy
- Observability: logging aplikasi, metrik, dan alerting, terpisah dari audit trail sesuai ADR-005
- Pengelolaan secrets di setiap lingkungan

## Skrip verifikasi

Rancang `scripts/verify.sh` yang menjalankan lint, type check, dan test dalam satu perintah, keluar dengan kode bukan nol kalau ada yang gagal. Skrip ini dibuat sungguhan di Tahap 8 Fase 1, dan dipakai oleh Stop hook untuk mencegah Claude menyatakan selesai saat test masih merah.

## Coding standards

Tulis `docs/coding-standards.md` berisi:

- Struktur folder dan layering, misalnya controller → service → repository
- Naming convention untuk kode dan database (snake_case untuk database sesuai ADR-001)
- Pola error handling dan format response API
- Cara logging dan cara memanggil audit trail
- Cara validasi input, transaksi DB, serta pengelolaan konfigurasi dan secrets
- Konvensi test, commit, dan PR

Setelah itu, **perbarui `CLAUDE.md`**: isi bagian "Perintah" (build, lint, test, migrasi) dan ringkasan stack.

## Gate Tahap 5

- [ ] Setiap teknologi punya alasan dan versi berlabel `[SRC-xx]`
- [ ] Model data fisikal tersusun
- [ ] Hosting, CI/CD, observability, dan pengelolaan secrets sudah ditetapkan
- [ ] `docs/coding-standards.md` disetujui user
- [ ] `CLAUDE.md` diperbarui

---

# Tahap 6: Design Guideline

Tujuan: memastikan tampilan konsisten di semua layar. Datang setelah PRD karena alur layar mengikuti fitur.

Input: `docs/03-prd.md`, `docs/05-stack.md`.

## Isi minimum

- Design token: warna, tipografi, spacing, radius, elevation
- Komponen dasar beserta state-nya (default, hover, disabled, error, loading)
- Pola untuk kondisi kosong, error, dan loading
- Wireframe untuk alur utama dari PRD (boleh berupa HTML statis sederhana di `docs/wireframes/` atau deskripsi terstruktur)
- Aturan aksesibilitas dan responsivitas

Output: `docs/06-design-guideline.md` + wireframe.

## Gate Tahap 6

- [ ] Semua alur utama di PRD punya wireframe
- [ ] Komponen dasar dan design token terdefinisi
- [ ] User menyetujui arah desain
