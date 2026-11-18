# ADR-025: Hosting VPS Biznet Gio dan Hostinger, OpenBao sebagai pengelola kunci dan rahasia, Valkey, serta observability dan pipeline yang dijalankan sendiri

Status: Diusulkan (menunggu gate Tahap 5; butir Valkey menunggu konfirmasi user karena T4-Q4 menyebut "Redis")
Tanggal: 2026-11-18

## Konteks

- ADR-019 menetapkan platform kontainer terkelola, PostgreSQL terkelola dengan PITR dan standby, KMS, secret manager, dan layanan log di cloud region Indonesia. "Server virtual yang dikelola sendiri tanpa layanan terkelola" termasuk alternatif yang ditolak karena patching, backup, dan failover database menjadi beban tim kecil `[DOK: adr/ADR-019-lingkungan-backup-pemulihan.md]`.
- Di tengah T5-Q1 user menyampaikan "vpsnya hostinger dan biznet gio saja". Pada T5-Q2 user memilih "Biznet utama + Hostinger": Biznet Gio menjalankan aplikasi, database utama, Keycloak, Redis/Valkey, OpenBao, dan object storage (WORM, backup), sedangkan Hostinger Jakarta menjalankan database standby dan salinan backup terenkripsi `[USER] T5-Q2` `[DOK: 05-catatan-stack.md]`.
- Kemampuan penyedia yang terverifikasi:
  - Hostinger punya VPS di Indonesia (Jakarta) `[SRC-85]`, dengan paket KVM 1–8 yang dibayar di muka `[SRC-86]` `[SRC-87]`, firewall dan backup mingguan `[SRC-88]`. Object storage, database terkelola, load balancer, KMS, secret manager, WAF, log terkelola, dan jumlah zona tidak ditemukan di halaman produk `[DOK: sources.md]` (Pencarian tanpa hasil).
  - Biznet Gio punya data center di Indonesia `[SRC-91]`, VPS NEO Lite dan NEO Lite Pro `[SRC-93]` `[SRC-94]`, jaringan privat, firewall, security group, dan anti-DDoS "Cloudflare Magic Transit" `[SRC-95]`, NEO Object Storage kompatibel S3 di endpoint Indonesia `[SRC-97]` dengan URL bertanda tangan untuk unduh `[SRC-98]` dan Object Lock mode Governance, Compliance, dan Legal Hold `[SRC-99]`, serta load balancer dengan listener yang protokolnya dipilih `[SRC-102]`. Uptime NEO Lite disebut hingga 99,9% `[SRC-106]`. KMS, secret manager, log terkelola, WAF, Redis/Valkey terkelola, log audit penyedia, PITR/standby PostgreSQL terkelola, versi objek, dan jumlah zona tidak ditemukan `[DOK: sources.md]` (Pencarian tanpa hasil).
- Semua penyedia wajib memproses dan menyimpan data di Indonesia `[USER] T4-Q1`. Target ketersediaan 99,0%, RPO ≤ 15 menit, RTO ≤ 4 jam di jam kerja dan ≤ 8 jam di luar jam kerja, backup di Indonesia (NFR-AVL-01, NFR-DR-01–04) `[DOK: 03-prd.md]`.
- Kode disimpan di GitHub, CI berjalan di GitHub, dan registry image di GitHub, hanya untuk kode dan data sintetis. Deploy dilakukan agen di VPS, tanpa kunci SSH production di GitHub `[USER] T5-Q6`. Persetujuan deploy production dijalankan di sisi VPS `[USER] T5-Q8`. Dokumentasi GitHub menyatakan: "If you are on a GitHub Free, GitHub Pro, or GitHub Team plan, other deployment protection rules, such as a wait timer or required reviewers, are only available for public repositories." `[SRC-240]`.
- ADR-007 menunda pilihan distribusi dan lisensi Redis ke Tahap 5: Redis 8 ke atas memakai pilihan RSALv2/SSPLv1/AGPLv3, sedangkan 7.2.x ke bawah BSD3 `[SRC-76]`.

## Keputusan

**1. Penyedia dan lingkungan** `[USER] T5-Q2` `[KEPUTUSAN-78]`:
- Production berjalan di Biznet Gio sebagai lokasi utama dan di Hostinger Jakarta sebagai lokasi kedua (database standby, salinan backup terenkripsi, node ketiga OpenBao, dan pemeriksa ketersediaan dari luar jaringan Biznet).
- Staging berjalan di Biznet Gio pada host terpisah dari production, hanya dengan data sintetis (ADR-019 butir 1 tetap berlaku). Lokal tidak berubah.
- Staging berada di jaringan privat yang terpisah dari jaringan privat production, tanpa rute ke host production mana pun. Mekanisme pemisahan jaringan Biznet Gio diverifikasi di Tahap 8 Fase 1 `[KEPUTUSAN-143]`.
- Ukuran host di bawah adalah estimasi awal penyusun. Ukuran divalidasi dengan uji beban NFR-CAP-04, yang memakai staging yang dinaikkan sementara ke ukuran production (biaya satu bulan tambahan).

**2. Topologi dan biaya** `[KEPUTUSAN-78]`. Harga per bulan dari halaman resmi penyedia:

Production, Biznet Gio:

| Host | Paket | Harga/bulan | Sumber | Isi |
| --- | --- | --- | --- | --- |
| app-1, app-2 | NEO Lite Pro MM.8.4 (4 vCPU, 8 GB, 80 GB NVMe) | Rp999.000 × 2 | `[SRC-94]` | Caddy, `api`, `realtime`, `worker`, 3 server Next.js, RME `api`/`worker`, Keycloak (1 node per host, klaster 2 node), agen metrik/log, ditambah deployment khusus audit dan privasi (lihat catatan di bawah tabel) |
| db-app | NEO Lite Pro ML.16.8 (8 vCPU, 16 GB, 90 GB) | Rp1.799.000 | `[SRC-94]` | Instans PostgreSQL (a): database aplikasi Rumah Sehat dan database `keycloak`; pgBackRest |
| db-rme | NEO Lite Pro MM.4.4 (4 vCPU, 4 GB, 60 GB) | Rp699.000 | `[SRC-94]` | Instans PostgreSQL (c): RME minimalis |
| db-audit | NEO Lite Pro MS.4.2 (2 vCPU, 4 GB, 60 GB) | Rp559.000 | `[SRC-94]` | Instans PostgreSQL (b): audit Rumah Sehat dan audit RME; tanpa standby (ADR-019 butir 3) |
| cache | NEO Lite MS 4.4 (4 vCPU, 4 GB, 60 GB SSD) | Rp179.000 | `[SRC-93]` | Valkey Rumah Sehat dan Valkey RME (instans terpisah) |
| video | NEO Lite Pro MM.8.4 | Rp999.000 | `[SRC-94]` | LiveKit dan TURN (ADR-016, KEPUTUSAN-87); kebutuhan CPU dan bandwidth untuk 100 sesi `[TIDAK DIKETAHUI]`, diukur lewat uji beban |
| bao-1, bao-2 | NEO Lite MS 4.2 (2 vCPU, 4 GB, 60 GB) | Rp139.000 × 2 | `[SRC-93]` | OpenBao, penyimpanan Raft |
| ops | NEO Lite MM 8.4 (4 vCPU, 8 GB, 60 GB) | Rp269.000 | `[SRC-93]` | Prometheus, Alertmanager, Grafana, Loki, GlitchTip |
| bastion | NEO Lite SS 2.1 (1 vCPU, 2 GB, 60 GB) | Rp80.000 | `[SRC-93]` | Bastion SSH (sertifikat dari OpenBao), skrip deploy production |
| Load balancer NEO | — | `[TIDAK DIKETAHUI]` | `[SRC-102]` | Listener TCP yang meneruskan TLS ke Caddy, bila tersedia; protokol listener belum terverifikasi (PT-39) |
| NEO Object Storage | Rp1.000/GB (1 region); Rp2.000/GB (2 region) | ±Rp300.000 (estimasi awal 300 GB) | `[SRC-97]` | Bucket per tingkat sensitivitas (ADR-018), bucket backup, bucket WORM dengan Object Lock mode Compliance `[SRC-99]` |

Deployment khusus audit dan privasi di app-1 dan app-2 (review DBA Tahap 5, DBA5-B01, F-P07, F-P09, F-P10). Setiap deployment berjalan sebagai kontainer di kedua host, tetapi hanya satu yang aktif pada satu waktu lewat kunci advisory `[SRC-234]`. Setiap deployment punya identitas OpenBao sendiri dan hanya memegang kredensial berikut:

| Deployment | Sistem | Kredensial yang dipegang | Catatan |
| --- | --- | --- | --- |
| `audit-relay` | Rumah Sehat dan RME (deployment terpisah) | Hanya `audit_relay` dan `audit_writer` | ADR-009 butir 3 |
| `audit-verify` | Rumah Sehat dan RME | Hanya role verifier audit (`audit_verifier`) dan baca bucket WORM | Membaca kepala rantai dari bucket WORM dan mengirim peringatan secara independen dari relay (ADR-009 butir 5) |
| `privacy-worker` | Rumah Sehat dan RME | Hanya role pekerja privasi dan hak hapus object storage | ADR-013 butir 5 |

Production, Hostinger Jakarta `[SRC-85]`:

| Host | Paket | Harga/bulan | Sumber | Isi |
| --- | --- | --- | --- | --- |
| db-standby | KVM 8 (8 vCPU, 32 GB, 400 GB NVMe) | promo Rp426.900, perpanjangan Rp853.900 (24 bulan dibayar di muka) | `[SRC-86]` `[SRC-87]` | Standby streaming instans (a) dan (c); repositori kedua pgBackRest (salinan backup terenkripsi) |
| bao-3 dan pemeriksa ketersediaan | KVM 1 (1 vCPU, 4 GB, 50 GB) | promo Rp116.900, perpanjangan Rp193.900 | `[SRC-86]` `[SRC-87]` | Node ketiga Raft OpenBao; pemeriksa ketersediaan dari luar jaringan Biznet (NFR-OBS-01) |

Staging, Biznet Gio:

| Host | Paket | Harga/bulan | Sumber | Isi |
| --- | --- | --- | --- | --- |
| stg-app | NEO Lite MM 8.8 (8 vCPU, 8 GB, 60 GB) | Rp289.000 | `[SRC-93]` | Semua kontainer aplikasi, Keycloak, Valkey, OpenBao satu node, LiveKit, self-hosted runner |
| stg-db | NEO Lite MM 8.4 (4 vCPU, 8 GB, 60 GB) | Rp269.000 | `[SRC-93]` | Tiga klaster PostgreSQL |

Total bulanan: Biznet production Rp6.860.000, object storage ±Rp300.000, Hostinger Rp543.800 (promo; Rp1.047.800 setelah perpanjangan), dan staging Rp558.000, sehingga **±Rp8.261.800/bulan** (estimasi). Angka ini belum termasuk load balancer `[TIDAK DIKETAHUI]`, PPN (apakah harga sudah termasuk pajak `[TIDAK DIKETAHUI]`), domain, biaya per email SES (USD 0,10 per 1.000 email a la carte; akun baru default paket Essentials USD 0,16 per 1.000 `[SRC-243]`), biaya Lalamove per order, dan MDR payment gateway. Biaya toko aplikasi: Apple Developer Program USD 99/tahun `[SRC-244]` dan Google Play USD 25 sekali bayar `[SRC-01]`. Jadwal mulai sewa (staging Desember 2026, production Februari 2027) adalah estimasi `[KEPUTUSAN-78]`.

**3. Jaringan, ingress, dan perlindungan tepi** `[KEPUTUSAN-78]`:
- Antar-host Biznet memakai jaringan privat Biznet `[SRC-95]`. Jalur Biznet–Hostinger memakai terowongan terenkripsi WireGuard. Perangkat WireGuard belum terdaftar sebagai sumber dan diverifikasi di Tahap 8 Fase 1.
- Firewall: yang publik hanya load balancer (443), bastion (22, dibatasi sumbernya), dan port WebRTC/TURN host `video` (port persis dari konfigurasi LiveKit, Tahap 8). Lalu lintas privat tidak dipercaya otomatis: production default deny dengan daftar izin per host asal, host tujuan, dan port `[KEPUTUSAN-144]`; staging di jaringan privat terpisah tanpa rute ke production `[KEPUTUSAN-143]`.
- Firewall production menolak semua lalu lintas secara bawaan (default deny) dan hanya membuka daftar izin per host asal, host tujuan, dan port, bukan seluruh jaringan privat `[KEPUTUSAN-144]`. Contoh daftar izin: app → db-app, db-rme, db-audit (PostgreSQL, misalnya 5432); app → cache (Valkey, misalnya 6379); app dan bastion → bao (OpenBao, misalnya 8200); ops → exporter di setiap host; db-standby Hostinger ↔ db-app dan db-rme hanya lewat terowongan WireGuard. Nomor port final mengikuti konfigurasi di `infra/` dan direview bersama aturan firewall.
- Observability tidak diekspos ke publik `[KEPUTUSAN-145]`: Grafana hanya dapat dibuka di belakang login SSO admin ber-MFA (mekanisme diverifikasi Tahap 8); Loki dan Prometheus hanya menerima koneksi dari host `ops` dan dari agen yang terautentikasi; akses manusia ke host `ops` lewat bastion.
- TLS diterminasi di Caddy milik Rumah Sehat di app-1 dan app-2. Caddy mendukung WebSocket di `reverse_proxy` dan HTTPS otomatis `[SRC-237]` `[SRC-189]`. Load balancer NEO dipakai sebagai listener TCP bila tersedia. Protokol listener dan dukungan WebSocket belum terverifikasi `[SRC-102]` (PT-39).
- WAF terkelola tidak tersedia. Perlindungan aplikasi web terdiri atas Caddy sebagai reverse proxy, validasi input, batas laju aplikasi di Valkey, dan batas laju per IP di Caddy (modulnya diverifikasi Tahap 8). Perangkat lunak WAF (misalnya OWASP CRS) dievaluasi di Tahap 8 Fase 1; bila tidak dipakai, dicatat sebagai risiko.
- Anti-DDoS Biznet disebut "Cloudflare Magic Transit" `[SRC-95]`, sehingga lalu lintas jaringan dapat dirutekan lewat jaringan Cloudflare. Isi lalu lintas tetap terenkripsi karena TLS diterminasi di Caddy milik Rumah Sehat, tetapi lokasi peruteannya `[TIDAK DIKETAHUI]`. Hal ini menjadi **butir gate untuk user** terhadap keputusan T4-Q1 ("tanpa kecuali") dan dikonfirmasi ke Biznet Gio (PT-39).

**4. Kontainer dan infrastruktur sebagai kode** `[KEPUTUSAN-78]`:
- Docker Engine 29.10.0 dan Docker Compose 5.8.0 per host `[SRC-187]` `[SRC-188]`. Tanpa Kubernetes: alternatif "Kubernetes yang dikelola sendiri" yang ditolak ADR-019 tetap berlaku, dan NEO Kubernetes `[SRC-100]` tidak dipakai.
- Perangkat provisioning VM `[TIDAK DIKETAHUI]`, dipilih dan diverifikasi di Tahap 8 Fase 1. Kandidatnya OpenTofu 1.13.9 (MPL-2.0) `[SRC-127]` bila ada provider untuk Biznet Gio dan Hostinger, atau skrip idempoten di `infra/`.
- Konfigurasi host, berkas Compose, Caddyfile, konfigurasi realm Keycloak, konfigurasi LiveKit, policy OpenBao, pgBackRest, dan aturan peringatan disimpan sebagai kode di `infra/` (ADR-024 butir 5).

**5. Database, replikasi, dan backup** `[KEPUTUSAN-77]` `[KEPUTUSAN-78]`:
- PostgreSQL 18.7 dijalankan sendiri, dengan akhir dukungan 2030-11-14 `[SRC-118]`. Keycloak 26.8.2 mendukung PostgreSQL 14.x–18.x dan diuji dengan 18 `[SRC-121]`. Pembagian tiga instans mengikuti ADR-019 butir 3.
- Instans (a) dan (c) direplikasi streaming ke db-standby di Hostinger. Promosi standby dilakukan manual menurut runbook. RTO 4/8 jam memberi ruang untuk langkah manual. Patroni tidak dipakai di rilis 1 karena menambah komponen yang harus dioperasikan dan dipulihkan. Instans audit (b) tanpa standby (ADR-019 butir 3).
- Backup dengan pgBackRest 2.60.1 `[SRC-186]`: PITR dengan `--type=time` `[SRC-235]`; repo1 di NEO Object Storage (S3) dan repo2 di Hostinger; repositori dienkripsi dengan `repo-cipher-type` aes-256-cbc `[SRC-235]`; `archive_timeout` ≤ 1 menit `[SRC-38]`; backup penuh harian; masa simpan 35 hari `[USER] T4-G2`.
- Database `keycloak` ikut PITR dan backup instans (a) (ADR-019 butir 4 tetap berlaku).

**6. Valkey sebagai implementasi komponen "Redis" ADR-007** `[KEPUTUSAN-77]` (**perlu konfirmasi user di gate**, karena T4-Q4 menyebut "Redis"):
- Valkey 9.1.3, lisensi BSD-3-Clause `[SRC-124]`. Valkey memuat kode Redis berlisensi BSD 3-Clause `[SRC-247]`. Redis 8 ke atas memakai pilihan RSALv2/SSPLv1/AGPLv3 `[SRC-125]` `[SRC-76]`.
- Dua instans terpisah, satu untuk Rumah Sehat dan satu untuk RME (ADR-007 butir 6), di host `cache`, hanya dapat dicapai dari jaringan privat, dengan autentikasi wajib. Kewajiban TLS dari ADR-007 dan ADR-013 tetap berlaku. Kemampuan TLS Valkey `[TIDAK DIKETAHUI]` (belum terdaftar sebagai sumber) dan diverifikasi di Tahap 8 Fase 1.
- Valkey tidak di-backup sebagai sumber data (ADR-019 butir 4).
- Kompatibilitas klien (crate `redis`, ioredis) dibuktikan lewat test integrasi Tahap 8 (ADR-024).

**7. OpenBao 2.7.2 (MPL-2.0) `[SRC-185]` menggantikan KMS dan secret manager** `[KEPUTUSAN-79]`:
- **Transit** memegang KEK. DEK dibangkitkan lewat `datakey` dan dikembalikan dalam bentuk terbungkus KEK. Rotasi KEK menyimpan versi lama, dan `min_decryption_version` mengatur versi yang masih dapat mendekripsi. Nilai lama dapat dibungkus ulang ke versi kunci baru dengan `rewrap` bila diperlukan `[SRC-222]`. Ini memenuhi "versi KEK yang tetap dapat membuka DEK lama tanpa membungkus ulang" pada ADR-013 butir 5.
- **KV** menyimpan rahasia aplikasi: password database aplikasi, kunci blind index, kunci enkripsi sesi BFF, rahasia HMAC webhook bila tidak di database, serta kredensial SES, DOKU, dan Lalamove.
- **Identitas proses**: setiap proses punya identitas sendiri dengan policy minimum, yaitu `api`, `realtime`, `worker`, `audit-relay`, RME, pekerja privasi, rotasi kunci, dan server Next.js (BFF). Metode autentikasi proses (AppRole atau JWT) dipilih di Tahap 8. Auth JWT/OIDC OpenBao mendukung `bound_claims` dan `bound_subject` `[SRC-223]`. Token OIDC GitHub Actions memakai klaim `sub` berformat `repo:ORG/REPO:...` `[SRC-239]` dan dipakai untuk runner staging.
- **Database secrets engine PostgreSQL** menerbitkan kredensial dinamis berumur pendek (`default_ttl`/`max_ttl`) `[SRC-224]` untuk role pemilik skema saat migrasi dan untuk akses administratif.
- **SSH CA** menandatangani sertifikat SSH berumur pendek untuk bastion dan host `[SRC-225]`. Peran SSH CA dipisah per kelas host (app, db, bao, ops, bastion), masing-masing dengan TTL pendek. Peran untuk deploy rutin hanya berlaku ke bastion dan host app. Sertifikat ke host db dan bao hanya terbit lewat Control Group, sebagai akses darurat `[KEPUTUSAN-150]`.
- **Login manusia** ke OpenBao memakai Login MFA dengan TOTP, yang dibangun di atas sistem Identity OpenBao `[SRC-251]`. Perlindungan brute force passcode TOTP tidak aktif secara bawaan, sehingga batas laju dipasang pada jalur login dan `/sys/mfa/validate` `[SRC-251]` `[KEPUTUSAN-148]`.
- **Root token** hanya dibuat lewat `bao operator generate-root` dengan persetujuan kuorum pemegang share unseal, dipakai seperlunya untuk setup awal, lalu langsung dicabut. Tidak ada root token permanen `[SRC-252]` `[KEPUTUSAN-147]`.
- **Administrasi OpenBao sebagai kode**: policy, auth method, dan peran disimpan di repositori (`infra/openbao/`) dan diterapkan oleh skrip dengan token admin yang hanya terbit lewat Control Group, yaitu persetujuan orang kedua lewat *factors* pada policy path `[SRC-253]`. OpenBao tidak mengirim notifikasi ke approver `[SRC-253]`, sehingga approver diberi tahu lewat prosedur di luar OpenBao (ditetapkan sebelum go-live) `[KEPUTUSAN-149]`.
- **Akses darurat (break-glass)** berupa penerbitan sertifikat atau kredensial superuser berjangka waktu lewat Control Group (persetujuan orang kedua) `[SRC-253]`. Prosedur dan pemegangnya ditetapkan sebelum go-live (PT-38).
- **Konsol panel penyedia** (Biznet Gio dan Hostinger) dapat mengakses VM di luar jalur OpenBao, sehingga diatur terpisah: akun bernama per orang, MFA bila tersedia (dukungannya `[TIDAK DIKETAHUI]`, ditanyakan lewat PT-39), jumlah pemegang akun seminimal mungkin, dan setiap perubahan dicatat `[KEPUTUSAN-152]`.
- **Audit device** (file, socket, atau syslog) dikirim ke security log. Sebagian besar nilai string di-hash HMAC-SHA256 bersalt `[SRC-226]`. Peringatan cakupan hash di halaman sumber dibaca penuh sebelum diandalkan, dan jalur seperti `sys/unseal` dan `sys/seal` tidak diaudit `[SRC-226]`.
- **Ketersediaan**: penyimpanan Raft 3 node, yaitu bao-1 dan bao-2 di Biznet dan bao-3 di Hostinger `[SRC-227]`. Snapshot Raft terenkripsi dikirim ke NEO Object Storage. Perintah snapshot belum ditemukan di halaman yang dibuka dan diverifikasi di Tahap 12.
- **Seal Shamir**, yaitu seal bawaan OpenBao `[SRC-228]`: tanpa KMS tidak ada auto-unseal `[KEPUTUSAN-79]`, sehingga node yang restart harus di-unseal manual oleh pemegang share. Jumlah share, ambang, dan pemegangnya (misalnya CTO, user, dan satu pihak ketiga) adalah keputusan user sebelum go-live (PT-38). Kehilangan share melebihi ambang berarti data terenkripsi tidak dapat dibuka. Dampaknya ke RTO dicatat sebagai RT-22.
- DEK di-cache di memori proses dengan batas waktu (ADR-013), sehingga gangguan singkat OpenBao tidak langsung menghentikan pembacaan data klinis.

**8. Konfigurasi dan rahasia per lingkungan** `[KEPUTUSAN-88]`:
- Lokal: berkas `.env.local` berisi nilai sintetis, tidak di-commit (tercantum di `.gitignore`), dengan OpenBao mode dev di Compose lokal.
- Staging dan production: semua rahasia ada di OpenBao. Proses mengambil rahasia saat start dan saat rotasi. Tidak ada rahasia di image, repositori, berkas Compose, atau variabel CI. Satu-satunya pengecualian adalah kunci privat cosign di secret GitHub dengan akses minimum.
- Konfigurasi non-rahasia lewat variabel lingkungan di berkas Compose per lingkungan di repositori.
- Rotasi: kredensial database dinamis (TTL), KEK lewat Transit rotate, kunci blind index dengan versi (`nik_bidx_key_version`, data dictionary). Prosedur tertulis dibuat sebelum go-live (ADR-013 butir 8).

**9. Observability dijalankan sendiri di host `ops`** `[KEPUTUSAN-80]`:
- Prometheus 3.16.0 `[SRC-190]`, Grafana 13.3.1 `[SRC-191]`, dan Loki 3.7.10 `[SRC-192]`. Grafana dan Loki berlisensi AGPL-3.0 `[SRC-191]` `[SRC-192]`; keduanya dipakai tanpa modifikasi untuk keperluan internal, dan bila perlu dimodifikasi, legal ditanya lebih dulu.
- Loki multi-tenant dengan header `X-Scope-OrgID`: tenant `app` untuk application log (30 hari) dan tenant `security` untuk security log (1 tahun) `[USER] T4-G2`. Retensi per tenant diatur lewat runtime overrides, dan retensi compactor diaktifkan secara eksplisit karena bawaannya mati `[SRC-236]`. Chunk disimpan di NEO Object Storage. Tenant `security` hanya dapat diakses pemegang peran keamanan (ADR-010).
- Metrik: laju galat, latensi p95, panjang antrean job, umur outbox audit, jeda relay, status integrasi per rekanan, keterlambatan pengambilan event Keycloak, sertifikat TLS, disk, replikasi standby, dan umur backup terakhir. Alertmanager mengirim peringatan ke email tim teknis lewat SES. Versi Alertmanager diverifikasi di Tahap 8.
- Pemeriksa ketersediaan berjalan di host Hostinger dan memeriksa endpoint kesehatan di Biznet dari luar jaringan Biznet (NFR-OBS-01, ADR-010 butir 3). Perangkatnya diverifikasi di Tahap 8.
- GlitchTip 7.0.0 (MIT), yang kompatibel dengan SDK Sentry `[SRC-150]`, menerima crash mobile dan galat backend dan web tanpa data pribadi (ADR-010 butir 4).
- Agen pengirim log dan metrik dari host dipilih dan diverifikasi di Tahap 8 Fase 1.
- Audit trail tetap di database audit (ADR-009), tidak di Loki.

**10. CI/CD dan persetujuan deploy** `[USER] T5-Q6, T5-Q8` `[KEPUTUSAN-81]`:
- **CI** di GitHub Actions dengan runner GitHub-hosted: `scripts/verify.sh` mode penuh, build image per komponen, pemindaian rahasia dengan gitleaks 8.30.1 `[SRC-151]`, dependensi dengan osv-scanner 2.7.1 `[SRC-152]` serta cargo-deny dan cargo-audit (ADR-024), image dengan Trivy 0.76.2 `[SRC-153]`. Image di-push ke GHCR dan dirujuk lewat digest `[SRC-242]`, lalu ditandatangani cosign 3.1.4 dengan pasangan kunci `[SRC-154]`. Kunci privat disimpan sebagai secret GitHub berakses minimum, dan kunci publik dipasang di host untuk verifikasi. Penandatanganan keyless Sigstore tidak dipakai karena mencatat metadata ke log transparansi publik.
- **Staging** di-deploy otomatis oleh self-hosted runner untuk setiap `main` yang hijau. GitHub menganjurkan kehati-hatian memakai self-hosted runner juga di repositori privat, karena runner dapat dikompromikan secara persisten oleh kode tak tepercaya di workflow, dan tidak ada jaminan runner biasa hanya menjalankan satu job `[SRC-254]` `[SRC-241]`. Karena itu runner staging adalah runner JIT sekali pakai, yang menjalankan paling banyak satu job lalu dihapus otomatis `[SRC-254]`, di kontainer atau VM bersih per job; hanya menerima workflow dari `main`; hanya memegang identitas OpenBao staging; dan berada di jaringan staging yang tanpa rute ke production (bagian 1). Staging hanya berisi data sintetis `[KEPUTUSAN-146]`. Langkahnya: tarik image berdasarkan digest, verifikasi tanda tangan, jalankan migrasi dengan kredensial dinamis OpenBao `[SRC-224]`, lalu `docker compose up` bergulir. Runner masuk ke OpenBao dengan token OIDC GitHub yang dibatasi klaim `sub` repositori dan ref `main` `[SRC-239]` `[SRC-223]`.
- **Production**: tidak ada runner dan tidak ada kredensial production di GitHub. Orang kedua yang berwenang login ke OpenBao dengan MFA, menerima sertifikat SSH berumur pendek, masuk lewat bastion, lalu menjalankan skrip deploy dengan digest yang sudah lulus staging. Skrip memverifikasi tanda tangan, menjalankan migrasi dengan kredensial pemilik skema dinamis, lalu deploy bergulir app-1 kemudian app-2. Kredensial pemilik skema diminta skrip deploy sendiri dari database secrets engine `[SRC-224]`, dengan TTL setara durasi migrasi dan dicabut setelah migrasi selesai. Path-nya dilindungi Control Group `[SRC-253]`, dan nilainya tidak pernah ditampilkan ke operator `[KEPUTUSAN-151]`. Akibatnya, langkah migrasi production membutuhkan persetujuan dari identitas lain selain operator deploy. Siapa saja orangnya ditetapkan bersama PT-38. Semua langkah tercatat di log audit OpenBao dan security log `[USER] T5-Q8`.
- Migrasi yang menyentuh database audit, role, grant, trigger penolak, atau RLS wajib direview DBA dan keamanan (ADR-021 butir 4 tetap berlaku).
- Proteksi branch `main` di GitHub diatur user bila paketnya mendukung. Proteksi ini bukan pengaman utama. Pengaman utama adalah git hook commit hijau dan persetujuan production di sisi VPS.

**11. Waktu server** `[KEPUTUSAN-78]`: jam server disinkronkan dengan NTP (chrony) ke sumber waktu yang ditetapkan. Sumber waktu dan perangkatnya diverifikasi di Tahap 8. Ini menggantikan "layanan waktu penyedia cloud" pada ADR-009 butir 6 dan ADR-017 butir 5. Jam perangkat klien tetap tidak dipakai.

**12. Object storage** `[KEPUTUSAN-78]`: NEO Object Storage menyimpan bucket per tingkat sensitivitas (ADR-018), bucket backup, dan bucket WORM dengan Object Lock mode Compliance, yang membuat objek "tidak dapat lagi di rubah maupun di hapus" `[SRC-99]`. URL bertanda tangan untuk unduh didukung `[SRC-98]`. Versi objek, URL bertanda tangan untuk unggah (PUT/POST), dan batas ukuran unggah `[TIDAK DIKETAHUI]`, diverifikasi di Tahap 8 Fase 1.

**13. Keputusan tambahan dari review keamanan Tahap 5 putaran 1** (`docs/review/tahap-05-security.md`, B-01 dan B-02):

| No | Keputusan | Bagian | Sumber |
| --- | --- | --- | --- |
| `[KEPUTUSAN-143]` | Staging di jaringan privat terpisah dari production, tanpa rute ke host production; mekanisme Biznet diverifikasi Tahap 8 Fase 1 | 1 | B-01 |
| `[KEPUTUSAN-144]` | Firewall production default deny dengan daftar izin per host asal, host tujuan, dan port; standby Hostinger hanya lewat WireGuard | 3 | B-01 |
| `[KEPUTUSAN-145]` | Observability tidak diekspos: Grafana di belakang SSO admin ber-MFA, Loki/Prometheus hanya dari host `ops` dan agen terautentikasi, akses manusia lewat bastion | 3 | B-01 |
| `[KEPUTUSAN-146]` | Runner staging JIT sekali pakai di kontainer/VM bersih per job, hanya workflow `main`, hanya identitas OpenBao staging | 10 | B-01; `[SRC-254]` |
| `[KEPUTUSAN-147]` | Root token hanya lewat `generate-root` dengan kuorum pemegang share, untuk setup awal, lalu dicabut; tanpa root token permanen | 7 | B-02; `[SRC-252]` |
| `[KEPUTUSAN-148]` | Login manusia ke OpenBao lewat Login MFA TOTP, dengan batas laju pada jalur login dan `/sys/mfa/validate` | 7 | B-02; `[SRC-251]` |
| `[KEPUTUSAN-149]` | Policy, auth method, dan peran OpenBao sebagai kode di `infra/openbao/`, diterapkan dengan token admin yang hanya terbit lewat Control Group; approver diberi tahu lewat prosedur di luar OpenBao | 7 | B-02; `[SRC-253]` |
| `[KEPUTUSAN-150]` | Peran SSH CA per kelas host (app, db, bao, ops, bastion) dengan TTL pendek; deploy rutin hanya ke bastion dan app; host db dan bao hanya lewat Control Group | 7 | B-02; `[SRC-225]` `[SRC-253]` |
| `[KEPUTUSAN-151]` | Kredensial pemilik skema production diminta skrip deploy, TTL setara durasi migrasi, dicabut setelahnya, path dilindungi Control Group, nilai tidak ditampilkan ke operator | 10 | B-02; `[SRC-224]` `[SRC-253]` |
| `[KEPUTUSAN-152]` | Konsol panel Biznet Gio dan Hostinger: akun bernama, MFA bila tersedia (PT-39), pemegang minimal, perubahan dicatat | 7 | B-02 |

## Butir ADR lama yang diubah

Teks ADR lama tidak ditulis ulang. Baris Status-nya diberi catatan rujukan ke ADR ini. Butir yang tidak tercantum di tabel tetap berlaku apa adanya.

| ADR | Butir | Bunyi lama (ringkas) | Cara baru memenuhinya | Sumber |
| --- | --- | --- | --- | --- |
| ADR-019 | 1 Lingkungan | Staging dan production di cloud region Indonesia, akun/proyek terpisah | Staging dan production di VPS Biznet Gio pada host terpisah; production juga di Hostinger Jakarta. Staging di jaringan privat terpisah tanpa rute ke production `[KEPUTUSAN-143]`; pemisahan akun penyedia antara staging dan production belum ditetapkan `[TIDAK DIKETAHUI]` | `[USER] T5-Q2`; `[KEPUTUSAN-78]` |
| ADR-019 | 2 Deployment | Platform kontainer terkelola region Indonesia; infrastruktur sebagai kode | Docker Compose per host di VPS, deploy bergulir app-1 lalu app-2; konfigurasi sebagai kode di `infra/`; perangkat provisioning dipilih Tahap 8 Fase 1 | `[SRC-187]` `[SRC-188]` `[SRC-127]`; `[KEPUTUSAN-78]` |
| ADR-019 | 3 Topologi | Proses minimal 2 instans di lebih dari satu zona bila ada; penyeimbang beban dengan TLS, WebSocket, dan perlindungan aplikasi web; PostgreSQL terkelola; standby dengan failover otomatis bila tersedia | Dua host aplikasi; jumlah zona Biznet `[TIDAK DIKETAHUI]`. Load balancer NEO sebagai listener TCP (protokol belum terverifikasi) dengan TLS dan WebSocket di Caddy. WAF terkelola tidak ada (bagian 3). PostgreSQL 18.7 dijalankan sendiri. Standby streaming di Hostinger dengan promosi manual | `[SRC-102]` `[SRC-237]` `[SRC-118]`; `[KEPUTUSAN-78]` |
| ADR-019 | 3 Topologi | Keycloak 2 instans; Redis terkelola atau dijalankan sendiri | Keycloak 1 node di app-1 dan app-2; Valkey dijalankan sendiri di host `cache` | `[SRC-66]` `[SRC-124]`; `[KEPUTUSAN-77]` |
| ADR-019 | 4 Backup | Padanan `archive_timeout` di layanan terkelola diverifikasi Tahap 5; backup di zona berbeda bila ada; object storage memakai versi objek | pgBackRest dengan `archive_timeout` ≤ 1 menit; repo1 NEO Object Storage, repo2 Hostinger (penyedia lain, Jakarta). Versi objek NEO `[TIDAK DIKETAHUI]`, diverifikasi Tahap 8 Fase 1 | `[SRC-186]` `[SRC-235]` `[SRC-38]`; `[USER] T4-G2` |
| ADR-019 | Alternatif yang ditolak | "Server virtual yang dikelola sendiri tanpa layanan terkelola" ditolak karena beban tim kecil | Kini dipilih user. Alasan penolakan lama tetap berlaku sebagai risiko (RT-21) | `[USER] T5-Q2` |
| ADR-019 | Konsekuensi | Penyedia harus diverifikasi menyediakan kontainer terkelola, PostgreSQL terkelola, object storage dengan WORM, KMS, secret manager, log, dan zona | Hasil verifikasi di `docs/sources.md`. Yang tidak tersedia dijalankan sendiri: PostgreSQL, OpenBao, Loki/Prometheus | `[DOK: sources.md]` |
| ADR-013 | 2 Enkripsi lapis penyedia | Disk database, backup, dan object storage dienkripsi penyedia dengan kunci di KMS region Indonesia | Backup database dienkripsi pgBackRest (aes-256-cbc). Enkripsi disk VPS dan enkripsi sisi server NEO Object Storage `[TIDAK DIKETAHUI]`; cara memenuhinya ditetapkan di Tahap 8 Fase 1 dan dibawa ke gate bila tidak tersedia | `[SRC-235]` |
| ADR-013 | 3 Envelope encryption | DEK dibungkus KEK di KMS | KEK di Transit OpenBao; DEK dari `datakey`, disimpan terbungkus di tabel kunci data | `[SRC-222]`; `[KEPUTUSAN-79]` |
| ADR-013 | 5 Rotasi KEK | Versi KEK di KMS yang tetap membuka DEK lama tanpa membungkus ulang (diverifikasi Tahap 5) | Transit rotate dengan versi kunci dan `min_decryption_version`; `rewrap` tersedia bila perlu | `[SRC-222]` |
| ADR-013 | 8 Pengelolaan rahasia | Secret manager terkelola region Indonesia | KV OpenBao di Biznet Gio dan Hostinger; aturan per lingkungan (bagian 8) | `[SRC-185]`; `[KEPUTUSAN-88]` |
| ADR-013 | Konsekuensi | KMS ketergantungan kritis; prosedur backup kunci mengikuti kemampuan KMS penyedia | OpenBao Raft 3 node lintas penyedia, snapshot terenkripsi, unseal Shamir manual (PT-38); DEK di-cache dengan batas waktu | `[SRC-227]` `[SRC-228]`; `[KEPUTUSAN-79]` |
| ADR-021 | 3 Akses administratif | MFA untuk semua identitas manusia di akun cloud production | Login MFA TOTP di OpenBao dengan batas laju jalur login dan validasi MFA, serta sertifikat SSH berumur pendek per kelas host lewat bastion. Konsol panel Biznet Gio dan Hostinger: akun bernama, MFA bila tersedia (`[TIDAK DIKETAHUI]`, PT-39), pemegang seminimal mungkin, perubahan dicatat | `[SRC-251]` `[SRC-225]`; `[KEPUTUSAN-148]` `[KEPUTUSAN-150]` `[KEPUTUSAN-152]` |
| ADR-021 | 3 Akses administratif | Hak dekripsi KEK dan baca secret hanya identitas proses; tanpa hak tetap manusia | Policy OpenBao per proses; akses manusia lewat kredensial dinamis dan sertifikat berumur pendek | `[SRC-224]` `[SRC-225]`; `[KEPUTUSAN-79]` |
| ADR-021 | 3 Akses administratif | Break-glass dengan persetujuan orang kedua, berjangka waktu, memicu peringatan | Penerbitan sertifikat atau kredensial superuser berjangka waktu lewat Control Group OpenBao (persetujuan orang kedua); root token hanya lewat `generate-root` berkuorum dan dicabut setelah setup; prosedur dan pemegang ditetapkan sebelum go-live (PT-38) | `[SRC-253]` `[SRC-252]`; `[KEPUTUSAN-147]` `[KEPUTUSAN-149]` |
| ADR-021 | 3 Akses administratif | Log audit penyedia cloud untuk KMS, secret manager, IAM, dan akses database dikirim ke security log | Audit device OpenBao, log SSH bastion, dan log akses administratif PostgreSQL dikirim ke tenant `security` Loki | `[SRC-226]` `[SRC-236]`; `[KEPUTUSAN-80]` |
| ADR-021 | 4 Pipeline | Identitas pipeline per lingkungan dengan kredensial cloud jangka pendek; rahasia dari secret manager | Runner staging JIT sekali pakai, hanya workflow `main`, masuk ke OpenBao staging dengan token OIDC GitHub yang dibatasi `sub` dan ref, di jaringan staging tanpa rute ke production; production tanpa identitas pipeline | `[SRC-254]` `[SRC-239]` `[SRC-223]`; `[USER] T5-Q6`; `[KEPUTUSAN-143]` `[KEPUTUSAN-146]` |
| ADR-021 | 4 Pipeline | Deploy production dari `main` yang dilindungi, dengan review yang tidak dapat dilewati dan persetujuan manual | Required reviewers GitHub tidak tersedia untuk repositori privat pada paket Free/Pro/Team. Persetujuan manual dijalankan orang kedua di sisi VPS (bastion, sertifikat OpenBao, MFA) atas digest yang lulus staging. Review sebelum merge ditegakkan git hook commit hijau dan review subagent; proteksi branch diatur user bila paket mendukung | `[SRC-240]`; `[USER] T5-Q8` |
| ADR-021 | 4 Pipeline | Kredensial pemilik skema hanya pada langkah migrasi | Kredensial pemilik skema dinamis dari OpenBao, diminta skrip deploy hanya untuk langkah migrasi, TTL setara durasi migrasi, dicabut setelahnya, path dilindungi Control Group, nilai tidak ditampilkan ke operator | `[SRC-224]` `[SRC-253]`; `[KEPUTUSAN-151]` |
| ADR-021 | 4 Pipeline (tafsiran, ditampilkan di gate) | Kredensial pemilik skema hanya pada langkah migrasi | Detach dan drop partisi arsip audit (ADR-009 butir 9) dijalankan lewat prosedur `audit.pr_archive_detach`. Kode prosedur ikut file migrasi, sedangkan pemanggilannya lewat bastion dengan kredensial pemilik dinamis dan persetujuan orang kedua (Control Group). Ini satu-satunya pemakaian kredensial pemilik di luar langkah migrasi deploy | `[SRC-224]` `[SRC-253]`; review DBA Tahap 5 (DBA5-B01) |
| ADR-021 | 4 Pipeline | Image dirujuk lewat digest dan ditandatangani bila platform mendukung | GHCR berdasarkan digest; tanda tangan cosign dengan pasangan kunci, diverifikasi di host sebelum deploy | `[SRC-242]` `[SRC-154]` |
| ADR-021 | 4 Pipeline | Kunci penandatangan aplikasi mobile di secret manager atau layanan penandatanganan | Disimpan di OpenBao sesuai aturan umum bagian 8. Tempat build rilis mobile dan cara kunci dipakai saat build `[TIDAK DIKETAHUI]`, ditetapkan sebelum Tahap 12 | `[KEPUTUSAN-88]` |
| ADR-010 | Tabel, baris Penyimpanan | Application log di layanan log terkelola atau dijalankan sendiri; security log di aliran terpisah | Loki dijalankan sendiri: tenant `app` (30 hari) dan tenant `security` (1 tahun), retensi compactor aktif | `[SRC-192]` `[SRC-236]`; `[USER] T4-G2` |
| ADR-010 | 3 Pemantauan | Pemeriksaan ketersediaan dari luar, metrik, peringatan | Pemeriksa di host Hostinger; Prometheus, Alertmanager (email lewat SES), Grafana | `[SRC-190]` `[SRC-191]`; `[KEPUTUSAN-80]` |
| ADR-010 | 4 Crash mobile | Layanan di Indonesia atau dijalankan sendiri | GlitchTip dijalankan sendiri di host `ops`; `sendDefaultPii` mati | `[SRC-150]` `[SRC-143]` `[SRC-255]` |
| ADR-010 | 5 Security log | Menerima log audit penyedia cloud untuk KMS, secret manager, IAM, akses database | Menerima audit device OpenBao, log bastion, dan log akses administratif PostgreSQL | `[SRC-226]` |
| ADR-009 | 5 Hash berantai | Kepala rantai ditulis ke object storage WORM; kemampuan WORM diverifikasi Tahap 5 | Bucket NEO Object Storage dengan Object Lock mode Compliance | `[SRC-99]` |
| ADR-009 | 6 Waktu | Jam server disinkronkan dengan layanan waktu penyedia cloud | NTP (chrony) yang dikelola sendiri; sumber dan perangkat diverifikasi Tahap 8 | `[KEPUTUSAN-78]` |
| ADR-009 | 9 Arsip | Partisi lama diekspor ke object storage WORM | Bucket WORM yang sama (Object Lock Compliance) | `[SRC-99]` |
| ADR-009 | Konsekuensi | Biaya WORM dan instans database audit dihitung di Tahap 5 | db-audit Rp559.000/bulan; object storage Rp1.000/GB/bulan (1 region) | `[SRC-94]` `[SRC-97]` |
| ADR-017 | 5 Sumber waktu | Jam server disinkronkan dengan layanan waktu penyedia cloud (diverifikasi Tahap 5) | NTP (chrony) yang dikelola sendiri; sumber dan perangkat diverifikasi Tahap 8 | `[KEPUTUSAN-78]` |
| ADR-007 | 6 Redis | Komponen "Redis"; distribusi dan lisensi ditetapkan Tahap 5 | Valkey 9.1.3 (BSD-3-Clause), dua instans; menunggu konfirmasi user | `[SRC-124]` `[SRC-247]` `[SRC-125]`; `[KEPUTUSAN-77]` |
| ADR-007 | 6 Redis | Bila Redis tidak tersedia, batas laju beralih ke batas laju di penyeimbang beban (M-15) | Batas laju per IP di Caddy (modul diverifikasi Tahap 8), karena kemampuan batas laju load balancer NEO tidak terverifikasi | `[SRC-102]`; `[KEPUTUSAN-78]` |
| ADR-011 | 7 Alur login web | Token di sesi BFF dienkripsi dengan kunci dari secret manager | Kunci diambil dari OpenBao. Penempatan BFF di server Next.js diatur ADR-024 | `[KEPUTUSAN-75]` `[KEPUTUSAN-79]` |

## Alternatif yang ditolak

- **Tetap memakai cloud terkelola (ADR-019 apa adanya)**: termasuk pilihan T5-Q2 dan tidak dipilih user `[USER] T5-Q2`. Biaya cloud terkelola belum dihitung saat pertanyaan diajukan `[DOK: 05-catatan-stack.md]`.
- **Hanya Biznet Gio**: tanpa lokasi kedua di penyedia lain. Tidak dipilih user `[USER] T5-Q2`.
- **Hostinger utama dengan object storage Biznet Gio**: tidak dipilih user `[USER] T5-Q2`.
- **NEO App (PaaS) dengan PostgreSQL terkelola**: versi PostgreSQL yang tersedia 14–16 `[SRC-101]`, sedangkan PITR dan standby tidak ditemukan `[DOK: sources.md]`.
- **NEO Kubernetes** `[SRC-100]`: alasan penolakan Kubernetes di ADR-019 tetap berlaku.
- **Failover database otomatis dengan Patroni**: menambah komponen yang harus dioperasikan dan dipulihkan; RTO 4/8 jam memberi ruang untuk promosi manual `[KEPUTUSAN-78]`.
- **HashiCorp Vault**: berlisensi Business Source License untuk versi 1.15.0 ke atas `[SRC-249]`, sedangkan OpenBao berlisensi MPL-2.0 dan aktif dirilis `[SRC-185]`.
- **Kunci dan rahasia di berkas konfigurasi host**: kunci terbaca oleh siapa pun yang memegang akses host atau backup, sehingga tujuan ADR-013 butir 3 dan butir 8 tidak tercapai.
- **Redis 8**: lisensi RSALv2/SSPLv1/AGPLv3 `[SRC-125]` `[SRC-76]`. **Redis 7.2.x** (BSD3 `[SRC-76]`): status pemeliharaannya tidak diverifikasi di Tahap 5, sedangkan Valkey aktif dirilis `[SRC-124]`.
- **Terraform sebagai perangkat provisioning**: berlisensi Business Source License sejak versi 1.6.0 `[SRC-128]`; OpenTofu (MPL-2.0) menjadi kandidat `[SRC-127]`.
- **Deploy production lewat SSH dari GitHub**: kredensial statis di GitHub melemahkan ADR-021. Tidak dipilih user `[USER] T5-Q6`.
- **Git server dan CI sendiri di VPS**: tidak dipilih user `[USER] T5-Q6`.
- **GitHub Enterprise untuk required reviewers; deploy tanpa persetujuan manual**: tidak dipilih user `[USER] T5-Q8`.
- **Penandatanganan keyless Sigstore**: mencatat metadata ke log transparansi publik `[KEPUTUSAN-81]`.
- **Layanan observabilitas SaaS di luar Indonesia**: dilarang `[USER] T4-Q1`.

## Konsekuensi

- Tim menjalankan sendiri patching OS, PostgreSQL, Keycloak, OpenBao, Valkey, LiveKit, Caddy, dan stack observability, serta backup, replikasi, dan failover. Beban operasional dan risiko kapasitas tim (Business Case R-04) naik. Siapa yang menjalankannya setelah go-live belum diketahui (PT-35), begitu pula petugas siaga (PT-18).
- Production bergantung pada orang: unseal OpenBao setelah restart, promosi standby, dan persetujuan deploy production memerlukan pemegang share dan orang kedua yang tersedia (PT-38).
- KEPUTUSAN-151 membuat migrasi production (dan prosedur arsip audit) membutuhkan identitas penyetuju Control Group yang berbeda dari operator deploy. Dengan begitu dibutuhkan minimal dua orang berwenang di OpenBao, lebih banyak daripada satu "orang kedua" di T5-Q8. Orangnya ditetapkan di PT-38 dan ditampilkan ke user di gate Tahap 5.
- Konfigurasi OpenBao (policy, Transit, SSH CA, database engine, audit device), bastion, dan skrip deploy production termasuk modul kritis T1 pada kategori enkripsi dan kunci data (ADR-013) serta akses administratif production dan pipeline (ADR-021).
- Runbook pemulihan (ADR-019 butir 5) wajib mencakup: promosi standby lintas penyedia, unseal dan pemulihan snapshot OpenBao, pemulihan PITR pgBackRest dari repo1 atau repo2, dan dekripsi data hasil pulihan. Uji pemulihan sebelum go-live mengukur semuanya terhadap NFR-DR-01 dan NFR-DR-02.
- Biaya infrastruktur ±Rp8,26 juta/bulan (bagian 2) dibandingkan dengan baseline Business Case di gate Tahap 5. Biaya load balancer dan PPN masih `[TIDAK DIKETAHUI]` (PT-39).

## Risiko sisa

| Risiko sisa | Penanganan | Register |
| --- | --- | --- |
| Beban operasional VPS kelola sendiri tanpa DevOps khusus | Infrastruktur sebagai kode, runbook, peringatan otomatis, uji pemulihan; keputusan PT-35 dan PT-18 | RT-21 |
| OpenBao tersegel setelah restart tanpa auto-unseal, atau share unseal hilang melebihi ambang | Raft 3 node lintas penyedia, cache DEK, pemegang share dan ambang ditetapkan user (PT-38), latihan unseal di uji pemulihan | RT-22, RT-06 |
| Failover database manual lintas penyedia melampaui RTO | Runbook dan latihan, pemantauan jeda replikasi, PITR pgBackRest sebagai jalur kedua | RT-23 |
| Riwayat advisory Next.js pada server yang memegang sesi | Pengerasan, patch ≤ 7 hari (ADR-024) | RT-25 |
| Perutean Magic Transit melewati jaringan Cloudflare dengan lokasi `[TIDAK DIKETAHUI]` | Butir gate user terhadap T4-Q1; konfirmasi Biznet Gio (PT-39) | RT-26 |
| Protokol listener load balancer NEO dan dukungan WebSocket belum terverifikasi | Verifikasi Tahap 8 Fase 1 (PT-39) | RT-28 |
| Tidak ada WAF terkelola | Caddy, validasi input, batas laju di Valkey dan Caddy; evaluasi OWASP CRS di Tahap 8 Fase 1 | Dicatat bila evaluasi Tahap 8 tidak menghasilkan WAF |
| Enkripsi disk VPS dan enkripsi sisi server object storage `[TIDAK DIKETAHUI]` (ADR-013 butir 2) | Verifikasi Tahap 8 Fase 1; bila tidak tersedia, cara pengganti dibawa ke gate | Butir gate Tahap 5 |
| Versi objek NEO Object Storage `[TIDAK DIKETAHUI]` (ADR-019 butir 4, ADR-018) | Verifikasi Tahap 8 Fase 1 | Butir gate Tahap 5 |
| Jumlah zona Biznet Gio `[TIDAK DIKETAHUI]`, sehingga app-1 dan app-2 mungkin berada di data center yang sama | Lokasi kedua di Hostinger untuk database dan OpenBao; ditanyakan ke Biznet Gio | RT-21 |
| Kunci privat cosign tersimpan di secret GitHub | Akses minimum; deploy production tetap butuh orang kedua yang memverifikasi digest hasil staging | — |
| Review sebelum merge di GitHub hanya dapat dipaksa bila paket mendukung proteksi branch | Git hook commit hijau, review subagent, dan persetujuan production di sisi VPS | — |
| Dukungan MFA panel akun Biznet Gio dan Hostinger `[TIDAK DIKETAHUI]`; panel dapat mengakses VM di luar jalur OpenBao | Akun bernama, pemegang minimal, perubahan dicatat `[KEPUTUSAN-152]`; ditanyakan lewat PT-39 | Butir gate Tahap 5 |
| Control Group tidak mengirim notifikasi ke approver `[SRC-253]` | Prosedur pemberitahuan di luar OpenBao, ditetapkan sebelum go-live | — |
| Mekanisme pemisahan jaringan staging dan production di Biznet Gio belum terverifikasi | Verifikasi Tahap 8 Fase 1; staging hanya berisi data sintetis | — |
| Perangkat yang belum terverifikasi: WireGuard, provisioning VM, agen log/metrik, Alertmanager, chrony, pemeriksa ketersediaan, SSO admin untuk Grafana, perintah snapshot Raft | Diverifikasi ke sumber resmi di Tahap 8 Fase 1 atau Tahap 12 sebelum dipakai | — |
