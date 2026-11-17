# Tahap 8: Development + Test (diulang per fase)

Tujuan: membangun produk fase demi fase. Setiap fase ditutup dengan test hijau, review, dan functional test QA sebelum fase berikutnya dimulai.

Rekomendasi model: Sonnet untuk kerja harian, naik ke Opus untuk modul kritis (transaksi, keamanan, audit trail) atau bug yang membandel.

Peran: Senior Software Engineer untuk task harian. **Tech Lead** memegang reference implementation di Fase 1 dan consistency review di akhir setiap fase (jalankan dengan Opus).

## Langkah pertama di Fase 1 (sekali saja)

0. **Periksa mesin pengembangan** sebelum task pertama: toolchain yang dipatok di stack (versi bahasa, alat build dan uji, Docker untuk basis data uji) benar-benar terpasang dan bisa **dijalankan** di OS mesin itu (versi OS lama bisa menolak biner terbaru). Pemasangan toolchain mengubah sistem user, jadi minta persetujuan dulu. Bila mesin tidak cocok atau terlalu lambat, usulkan VPS pengembangan Linux yang sama dengan runner CI; titik pindah terbaik adalah sesudah commit hijau pertama, dengan panduan penyiapan ditulis di `docs/`.

1. **Setup repo**: `git init`, struktur folder sesuai `docs/coding-standards.md`, `.gitignore`, `.env.example` (tanpa secrets asli).
2. **Lapis otomatis**:
    - Formatter dan linter + type checker dengan aturan ketat
    - Architecture test: aturan dependensi antar-layer (misalnya controller dilarang akses DB langsung)
    - Pre-commit hook
    - CI pipeline (misalnya GitHub Actions): lint → type check → unit test → coverage minimum → SAST → dependency scan
3. **Hooks Claude Code**: tawarkan ke user untuk menambahkan hook di `.claude/settings.json` yang menjalankan formatter dan linter setiap kali file diedit. Tunjukkan isinya dan minta persetujuan sebelum menulis.
4. **`scripts/verify.sh`**: buat skrip yang menjalankan lint, type check, test, dan security scan dalam satu perintah, keluar dengan kode bukan nol kalau gagal. Skrip ini adalah definisi resmi "hijau". Pembuatan dan setiap perubahannya meminta persetujuan user. Skrip ini dipakai oleh tiga penjaga:
    - **Git hook pre-commit dan pre-merge-commit**: commit atau merge berisi kode ditolak kalau merah. Commit juga ditolak kalau ada perubahan kode yang tidak ikut di-stage, supaya yang diverifikasi persis sama dengan yang di-commit.
    - **Stop hook**: Claude tidak bisa mengakhiri giliran saat ada perubahan kode yang merah.
    - **CI**: memverifikasi ulang di server. Kalau repo ada di GitHub, aktifkan branch protection yang mewajibkan CI hijau sebelum merge ke branch utama.
5. **Reference implementation**: bangun satu fitur end-to-end dengan sempurna (API → service → DB → audit log → test) sebagai contoh resmi. Catat lokasinya di `CLAUDE.md`.

Ikuti aturan kode di `anti-halu.md` bagian 5: tidak memanggil API yang belum dipastikan ada, cek package di registry resmi sebelum ditambahkan, kunci versi dengan lockfile.

### Pola `scripts/verify.sh` yang terbukti

- **Urutan bootstrap.** Hook pre-commit menolak commit berisi kode selama `scripts/verify.sh` belum ada, dan menolak commit bila ada kode lain yang belum di-commit. Jadi commit kode pertama harus menyertakan `verify.sh`. Untuk tetap commit per task saat beberapa task kerangka selesai bersamaan, simpan task lain sementara (`git stash push -u -- <path>`), commit satu task, lalu kembalikan (`git stash pop`). Jangan stash folder yang membawa `.gitignore`-nya sendiri (misalnya folder web), karena isi yang diabaikan seperti `node_modules` langsung terhitung untracked.
- **Langkah berjenjang menurut komponen.** Langkah sebuah komponen berjalan bila komponennya ada di repo, dan begitu ada wajib lulus, tanpa flag atau variabel untuk melewatinya. Langkah yang artefaknya belum dibuat ditulis "BELUM DITERAPKAN" beserta task yang akan menerapkannya, tidak dihitung lulus, dan jumlahnya disebut di ringkasan akhir. Setiap penambahan langkah tetap meminta persetujuan user.
- **Cegah hijau palsu karena dependensi basi.** Jalankan cargo dengan `--locked`. Untuk npm, periksa `node_modules/.package-lock.json` ada dan tidak lebih tua dari `package-lock.json` (rumusan `npm ls --all` terlalu peka: gagal karena peer dependency dan paket opsional per platform). CI selalu menjalankan `npm ci` sebelum skrip.
- **Portabel.** Uji di macOS dan Linux; jangan bergantung pada perintah yang bisa dibayangi alias lokal (misalnya `head`); pakai `sed -n '1p'`.
- **Jalankan tanpa redirect.** `guard-git.sh` memblokir perintah yang menyebut `scripts/verify.sh` bersama operator tulis seperti `>` (termasuk `2>&1`); jalankan `bash scripts/verify.sh` apa adanya, atau saring dengan pipa (`| grep`).

## Siklus per task

1. Ambil task berikutnya dari `docs/backlog.md` yang memenuhi Definition of Ready.
2. Tulis kode + unit/integration test, mengikuti pola reference implementation.
3. Sertakan file migrasi kalau ada perubahan skema. Perubahan manual di database dilarang.
4. Jalankan `scripts/verify.sh` sampai hijau. Tunjukkan perintah dan ringkasan output-nya sebagai bukti, jangan hanya menyatakan berhasil.
5. Panggil subagent `code-reviewer` untuk mereview diff terhadap coding standards, acceptance criteria, dan Definition of Done. Verifikasi setiap temuan dulu, lalu perbaiki yang terkonfirmasi dan menyangkut kebenaran atau requirement.
6. Commit dengan pesan yang jelas. Git hook akan menjalankan `scripts/verify.sh` sekali lagi; kalau ditolak, perbaiki lalu commit ulang. Jangan pernah melewati hook. Tandai task selesai di backlog **beserta commit hash**, dan perbarui kolom "File kode / test" di `docs/traceability.md`.
7. Sarankan user menjalankan `/clear` sebelum task berikutnya.

Kalau menemukan pola baru yang belum ada standarnya: **berhenti**, usulkan ADR ke user, perbarui `CLAUDE.md` atau `docs/coding-standards.md`, baru lanjut.

## Penutupan setiap fase

1. **Functional test QA**: panggil subagent `qa-engineer` untuk menjalankan test case fase ini dari `docs/test-plan.md`. Bug masuk backlog dengan severity.
2. **Consistency review**: dengan Opus effort high, periksa drift pola, duplikasi logika, dan abstraksi yang tidak perlu di seluruh kode fase ini. Temuan masuk backlog sebagai tech debt.
3. Tulis laporan singkat di `docs/fase-<n>-laporan.md`.

## Gate fase

- [ ] Semua story fase memenuhi Definition of Done
- [ ] CI hijau
- [ ] Functional test QA lolos, tidak ada bug Critical/High terbuka
- [ ] Consistency review selesai
- [ ] User menyetujui fase ditutup

Tahap 8 baru berstatus `selesai` setelah semua fase lulus gate.
