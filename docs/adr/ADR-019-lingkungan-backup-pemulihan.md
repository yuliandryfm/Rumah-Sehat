# ADR-019: Tiga lingkungan di cloud Indonesia, deployment berbasis kontainer, PITR, dan uji pemulihan

Status: Diusulkan
Tanggal: 2026-11-18

## Konteks

- Ketersediaan bulanan layanan inti dan RME minimalis 99,0%. Pemeliharaan terjadwal diumumkan 48 jam sebelumnya (NFR-AVL-01–03) `[USER] T3-Q12` `[DOK: 03-prd.md]`.
- RPO ≤ 15 menit. RTO ≤ 4 jam di jam kerja dan ≤ 8 jam di luar jam kerja. Uji pemulihan dilakukan sebelum go-live. Backup disimpan di Indonesia (NFR-DR-01–04) `[USER] T3-Q13, T3-Gate G5` `[KEPUTUSAN-14]`.
- Semua data dan penyedia di Indonesia `[USER] T4-Q1`. Perubahan skema hanya lewat file migrasi `[DOK: CLAUDE.md]`.
- Rekanan memerlukan lingkungan uji (US-1401). SATUSEHAT menyediakan kode akses Sandbox (Staging) dan Production `[SRC-57]`.
- Penyedia cloud dipilih di Tahap 5 dengan persetujuan user `[DOK: 00-project-brief.md]`.

## Keputusan

**1. Lingkungan** `[KEPUTUSAN-35]`:

| Lingkungan | Lokasi | Data | Kegunaan |
| --- | --- | --- | --- |
| Lokal | Laptop developer (kontainer) | Sintetis | Pengembangan dan test otomatis |
| Staging | Cloud region Indonesia, akun/proyek terpisah dari production | Sintetis saja | QA, SIT, UAT, lingkungan uji rekanan (US-1401), RME ke SATUSEHAT Sandbox |
| Production | Cloud region Indonesia, akun/proyek tersendiri | Data nyata | Layanan go-live |

Data production tidak pernah disalin ke staging atau lokal.

**2. Deployment** `[KEPUTUSAN-35]`:
- Setiap komponen backend dan web dibangun sebagai image kontainer di pipeline CI, lalu dijalankan di platform kontainer terkelola region Indonesia (produk dipilih Tahap 5).
- Infrastruktur didefinisikan sebagai kode, sehingga lingkungan dapat dibangun ulang untuk pemulihan.
- Deploy bergulir tanpa henti layanan. Migrasi skema dijalankan sebagai langkah terpisah sebelum deploy, dengan pola *expand/contract* supaya versi lama dan baru dapat berjalan bersamaan.
- Pipeline CI hanya memproses kode dan data sintetis, tidak pernah data pribadi.

**3. Topologi production** `[KEPUTUSAN-35]`:
- Proses `api`, `realtime`, dan `worker` (ADR-006) masing-masing minimal 2 instans, tersebar di lebih dari satu zona bila region menyediakannya. Job tunggal dijalankan satu instans lewat penguncian (ADR-007).
- Penyeimbang beban dengan TLS dan dukungan WebSocket, ditambah perlindungan aplikasi web dan batas laju di region yang sama.
- Database PostgreSQL terkelola, tiga instans: (a) aplikasi Rumah Sehat, ditambah database `keycloak` dengan role terpisah; (b) audit, berisi database audit Rumah Sehat dan database audit RME sebagai database terpisah dengan role terpisah; (c) aplikasi RME minimalis.
- Keycloak minimal 2 instans terklaster `[SRC-66]`, dan Redis terkelola atau dijalankan sendiri di region Indonesia (ADR-007, ADR-011) `[USER] T4-Q4`.
- Instans (a) dan (c) memakai standby dengan failover otomatis di zona lain bila layanan terkelola menyediakannya. Instans audit (b) tanpa standby, karena gangguan audit store tidak menghentikan aksi: event tertahan di outbox (ADR-009).
- Biaya topologi ini dihitung di Tahap 5. Kalau biayanya tidak dapat diterima, pengurangan (misalnya tanpa standby) dibawa ke user beserta dampaknya terhadap NFR-AVL-01.

**4. Backup dan RPO** `[KEPUTUSAN-35]`:
- Pemulihan point-in-time dari arsip WAL berkelanjutan untuk ketiga instans. Dokumentasi PostgreSQL menyatakan `archive_timeout` membatasi umur data yang belum diarsipkan, dan nilai sekitar satu menit biasanya wajar `[SRC-38]`. Target interval arsip ≤ 1 menit, jauh di bawah RPO 15 menit. Padanan pengaturan ini di layanan terkelola diverifikasi di Tahap 5.
- Backup penuh harian. Masa simpan backup `[TIDAK DIKETAHUI]` (PT-24), dengan usulan nilai sementara 35 hari yang diputuskan user di gate Tahap 4. Sampai ada keputusan, backup tidak dihapus otomatis.
- Redis tidak di-backup sebagai sumber data, karena isinya dibangun ulang dari PostgreSQL (ADR-007).
- Semua backup disimpan di Indonesia (NFR-DR-04), di zona berbeda dari database utama bila tersedia.
- Object storage memakai versi objek. Kepala rantai audit memakai WORM (ADR-009).

**5. RTO dan uji pemulihan** `[KEPUTUSAN-35]`:
- Runbook pemulihan tertulis mencakup: pemulihan database ke titik waktu tertentu, pembangunan ulang lingkungan dari kode infrastruktur, rotasi kredensial, dan verifikasi rantai audit setelah pemulihan.
- Uji pemulihan penuh di lingkungan terpisah sebelum go-live, dengan waktu dan titik pemulihan diukur terhadap NFR-DR-01 dan NFR-DR-02 (NFR-DR-03, KEPUTUSAN-14).

**6. Ketersediaan** diukur dari pemeriksa ketersediaan (ADR-010). Gangguan rekanan dan pihak ketiga dicatat terpisah (NFR-AVL-04) `[KEPUTUSAN-35]`.

## Alternatif yang ditolak

- **Server virtual yang dikelola sendiri tanpa layanan terkelola**: patching, backup, dan failover database menjadi beban tim, padahal tim kecil.
- **Kubernetes yang dikelola sendiri**: kompleksitas operasional tidak sebanding untuk tiga jenis proses.
- **Satu instans database untuk aplikasi, audit, dan RME**: pemisahan audit dari aplikasi (ADR-005, ADR-009) dan pemisahan RME (ADR-006) melemah, dan satu gangguan berdampak ke semuanya.
- **Backup ke region di luar Indonesia**: dilarang (NFR-DR-04, `[USER] T4-Q1`).

## Konsekuensi

- Pemilihan penyedia cloud di Tahap 5 harus memverifikasi ke dokumentasi resmi bahwa region Indonesia menyediakan: platform kontainer terkelola, PostgreSQL terkelola dengan PITR dan standby, object storage dengan URL bertanda tangan, versi objek, dan WORM, KMS, secret manager, layanan log, dan jumlah zona.
- Jam kerja untuk NFR-DR-02 dan kalender hari kerja wajib ditetapkan sebelum go-live.
- Petugas siaga di luar jam kerja tetap menjadi pertanyaan biaya dan kapasitas tim (PT-18).
