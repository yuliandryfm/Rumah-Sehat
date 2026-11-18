# Review Keamanan Tahap 5 (Stack, Hosting VPS, OpenBao, CI/CD)

Tanggal: 2026-11-18
Reviewer: security-architect (independen, tidak ikut menyusun dokumen yang direview)
Tingkat ketelitian: T1 Kritis, 2 putaran (maksimal 2) (`aturan-kerja.md` §1)
**Status akhir (putaran 2): LULUS DENGAN CATATAN.** B-01 dan B-02 tertutup. Rinciannya di §8. Butir gate G-01–G-09 tetap milik user.
Dokumen yang direview: `docs/adr/ADR-024-stack-rust-nextjs-flutter.md`, `docs/adr/ADR-025-hosting-vps-openbao.md`, `docs/05-stack.md` (§2, §3, §5, §7), `docs/05-asvs-checklist.md` (§3, §4, U-01–U-14), CR-012 di `docs/backlog.md`
Acuan: `docs/04-threat-model.md` v1.0 (lulus gate Tahap 4), ADR-009, -010, -011, -012, -013, -014, -016, -018, -019, -021, `docs/sources.md`, `docs/05-catatan-stack.md`, `docs/coding-standards.md`, `docs/04-risk-register.md` v1.1 draf

Metode: tingkat ancaman mengikuti matriks threat model §0.3, dan status Tercermin/Sebagian/Belum mengikuti §0.4. Kriteria blokir mengikuti `aturan-kerja.md` §2. Penilaian dibatasi pada dokumen di atas. Fakta penyedia (Biznet Gio, Hostinger) diambil dari `docs/sources.md` dan tidak dibuka ulang. Empat halaman resmi yang saya buka untuk menilai apakah mitigasi bisa dijalankan tercatat di §7 sebagai R-1–R-4. Pada putaran 2, keempatnya sudah terdaftar sebagai SRC-251–SRC-254.

---

## 1. Ringkasan status (putaran 1)

**TIDAK LULUS (putaran 1).** Ada 2 temuan blokir. Keduanya bisa ditutup dengan perbaikan di tingkat dokumen.

- **B-01**: ancaman baru bertingkat Tinggi (runner self-hosted staging dan kepercayaan pada "jaringan privat") belum punya mitigasi di desain. Selain itu, klaim "aman dipakai karena repositori privat" tidak didukung sumber yang dirujuk.
- **B-02**: KMS-01 dibuka kembali oleh CR-012. Prinsip "tanpa hak tetap manusia" belum dirancang untuk jalur yang kini paling berkuasa, yaitu:
  - administrasi OpenBao itu sendiri (root token, penulisan policy);
  - root OS di host `bao-*` dan `db-*`;
  - kredensial pemilik skema yang dipegang orang yang mendeploy;
  - konsol penyedia VPS.

Setelah dinilai ulang, ada 17 ancaman bertingkat Tinggi: **10 Tercermin, 6 Sebagian, 1 Belum**.

- Dua di antaranya menjadi blokir: CI-06 (Belum) dan KMS-01 (Sebagian).
- DB-02 berstatus Sebagian karena bergantung pada B-02.
- Empat Sebagian lain (CI-01, OS-01, APP-04, DB-07) tidak memblokir. Mitigasinya sudah dirancang, dan bagian yang belum terpenuhi sudah tertulis sebagai verifikasi Tahap 8 atau dibawa ke gate. Statusnya baru menjadi Tercermin setelah user memutuskan G-02, G-03, dan G-06, atau setelah verifikasi Tahap 8 selesai.

Yang membaik dibanding desain Tahap 4:
- Production tidak memakai runner dan tidak menyimpan kredensial di GitHub.
- Kredensial pemilik skema bersifat dinamis dan berumur pendek `[SRC-224]`.
- LiveKit dijalankan sendiri tanpa Egress, sehingga tingkat VID-03 dan VID-04 turun `[SRC-220]`.
- Pemisahan endpoint admin Keycloak kini bersumber `[SRC-229]` `[SRC-230]`.
- Next.js dikeraskan, dan Server Actions dilarang untuk mutasi (KEPUTUSAN-75, KEPUTUSAN-115).

---

## 2. Tabel penilaian ancaman (putaran 1)

Kolom "TM" adalah tingkat di threat model v1.0. Kolom "Ulang" adalah tingkat setelah CR-012 dan stack baru. Beberapa ancaman di daftar minimal orkestrator bertingkat **Sedang** di threat model (KMS-02, -03, -04, API-07, KC-07, PG-08, RD-01, VID-04, VID-05). Ancaman itu tetap saya nilai, tetapi tidak bisa menjadi blokir.

### 2.1 Ancaman lama

| ID | TM | Ulang | Perubahan karena CR-012/stack | Mitigasi di desain baru (rujukan) | Status | Temuan |
| --- | --- | --- | --- | --- | --- | --- |
| KMS-01 | Tinggi | Tinggi (K naik) | KMS dan IAM penyedia diganti OpenBao yang dioperasikan tim sendiri | Policy per proses; akses manusia lewat kredensial dinamis dan sertifikat SSH berumur pendek; MFA login (mekanisme `[TIDAK DIKETAHUI]`); break-glass dengan orang kedua (mekanisme "sebelum go-live"); audit device ke security log (ADR-025 butir 7; tabel ADR-021; 05-stack §5.3) | **Sebagian** | **B-02**. Belum dirancang: pemegang root token dan hak menulis policy, cakupan peran SSH CA, kredensial pemilik skema di tangan deployer, konsol penyedia |
| KMS-02 | Sedang | Sedang (K naik) | Unseal Shamir manual; snapshot Raft di NOS (Biznet) | Raft 3 node lintas penyedia `[SRC-227]`; cache DEK; snapshot terenkripsi; PT-38; RT-22 | Sebagian | NB-07: kunci cipher pgBackRest disimpan di OpenBao dan snapshot hanya ada di Biznet. Bila Biznet padam total, backup database tidak bisa dibuka |
| KMS-03 | Sedang | Sedang | KEK kini berada di memori VM sendiri | M-25 diperluas ke OpenBao lewat U-06 (belum diadopsi) | Sebagian | U-06 perlu menjadi KEPUTUSAN |
| KMS-04 | Sedang | Sedang | Kunci blind index di KV OpenBao | KV dengan kunci berversi (ADR-025 butir 8) | Tercermin | — |
| KMS-05 | Tinggi | Tinggi | Satu rahasia sengaja disimpan di GitHub (kunci privat cosign) | Semua rahasia staging/production di OpenBao; gitleaks di CI dan `verify.sh`; `.env.local` tidak di-commit (ADR-025 butir 8; 05-stack §5.1) | Tercermin | Pengecualian cosign dinilai terpisah (CI-07, G-04) |
| CI-01 | Tinggi | Tinggi | Required reviewers tidak tersedia untuk repo privat Free/Pro/Team `[SRC-240]` | Production tanpa runner dan kredensial di GitHub; orang kedua mendeploy digest yang lulus staging lewat bastion; verifikasi cosign; migrasi dengan kredensial dinamis; review DBA/keamanan untuk migrasi berisiko (ADR-025 butir 10; 05-stack §3.3) | Sebagian | Tidak blokir; dibawa ke gate lewat CR-012. Isi pemeriksaan orang kedua belum ditetapkan (U-11). NB-01, G-03 |
| DB-02 | Tinggi | Tinggi | Enkripsi lapis penyedia `[TIDAK DIKETAHUI]`; replika penuh di Hostinger | Envelope encryption tetap (Transit `[SRC-222]`); backup pgBackRest terenkripsi `[SRC-235]` | Sebagian | Bergantung pada B-02. Lapis penyedia untuk PII non-envelope masuk G-02 |
| OS-01 | Tinggi | Tinggi | Contoh resmi URL bertanda tangan NOS memakai `http://` `[SRC-98]` | Umur 5 menit, unduh sebagai lampiran, redaksi log (ADR-018 butir 3); verifikasi HTTPS di Tahap 8 | Sebagian | NB-04 |
| OS-02 | Tinggi | Tinggi | Perangkat IaC belum dipilih | Bucket privat per sensitivitas, kebijakan sebagai kode, H-11 | Tercermin | H-11 Tertunda Tahap 8 |
| OS-05 | Sedang | Sedang | Versi objek NOS `[TIDAK DIKETAHUI]`; Object Lock tersedia `[SRC-99]` | Pemusnahan mencakup semua versi (ADR-018 butir 5) | Sebagian | NB-04 |
| APP-02 | Tinggi | Tinggi | Mekanisme platform ditunda ke Tahap 8 | local_auth `[SRC-137]`, flutter_secure_storage `[SRC-136]`, ADR-021 butir 1 | Tercermin | NB-06 |
| APP-04 | Tinggi | Tinggi (K naik) | Tanpa WAF terkelola; batas laju Caddy dan IP klien belum terverifikasi | Brute force Keycloak per akun `[SRC-69]`, Argon2, pesan galat umum, autentikasi ulang | Sebagian | Penguncian per akun tidak menahan credential stuffing. NB-03, G-06 |
| APP-05 | Tinggi | Tinggi | sentry_flutter mengirim ke GlitchTip sendiri | ADR-021 butir 2; `sendDefaultPii` mati | Tercermin | NB-06: metode uji ditunda dari Tahap 5 |
| API-07 | Sedang | Sedang | Cadangan batas laju di LB hilang | Valkey + Caddy | Sebagian | NB-03 |
| KC-06 | Tinggi | Tinggi | Pembatasan sepenuhnya di Caddy | Path yang dibuka `[SRC-229]`; `--hostname-admin` `[SRC-230]`; firewall | Tercermin | NB-10 |
| KC-07 | Sedang | Sedang | Tanpa WAF | Penguncian 15 menit; M-14 sebagian | Sebagian | — |
| KC-09 | Tinggi | Tinggi | Patch semua komponen kini beban tim | KEPUTUSAN-138; Trivy | Tercermin | PT-35 (G-09); NB-10 |
| PG-08 | Sedang | Sedang | Tanpa WAF dan batas laju LB | Batas laju aplikasi; retry penyedia | Sebagian | NB-03 |
| RD-01 | Sedang | Sedang | Cadangan M-15 hilang; Valkey di satu host | Batas per IP di Caddy (belum terverifikasi) | Sebagian | NB-03 |
| RD-02 | Tinggi | Tinggi | Host app dan host `cache` dipakai bersama RS dan RME; TLS Valkey `[TIDAK DIKETAHUI]` | Instans terpisah dengan autentikasi; token JWE terikat ID sesi; Redis bukan sumber kebenaran | Tercermin (bersyarat) | Syarat: B-01 dan verifikasi TLS Valkey di Tahap 8 |
| VID-03 | Tinggi | Tinggi (K Rendah) | LiveKit dijalankan sendiri di Biznet | KEPUTUSAN-87 `[SRC-126]` `[SRC-217]` | Tercermin | Magic Transit (G-01) |
| VID-04 | Sedang | Rendah | Egress tidak dipasang `[SRC-220]` | KEPUTUSAN-87 | Tercermin | — |
| VID-05 | Sedang | Sedang | Satu host `video` | Uji beban; chat tetap jalan | Sebagian | NB-09 |

### 2.2 Ancaman baru

| ID usulan | STRIDE | Ancaman | K | D | Tingkat | Mitigasi di desain (rujukan) | Status | Temuan |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| TB1-01 | I | Perutean Magic Transit lewat jaringan Cloudflare di lokasi `[TIDAK DIKETAHUI]` `[SRC-95]`; isi tetap terenkripsi TLS sampai Caddy | `[TIDAK DIKETAHUI]` | Sedang (teknis) | Sedang | TLS diterminasi di Caddy; PT-39; RT-26 | Butir gate | G-01 |
| BFF-01 | S, I, E | RCE di server Next.js `[SRC-246]` membuka kunci JWE dan Valkey, sehingga sesi aktif bisa dibajak | S | T | **Tinggi** | ADR-024 butir 2–3; KEPUTUSAN-115; RT-25 | Tercermin | NB-05; GHSA-pc58-hwjq-p385 sudah ditambal sejak 16.4.5 `[SRC-248]` |
| CI-06 | T, E, I | Runner self-hosted staging yang terkompromi bergerak ke jaringan privat production | S | T | **Tinggi** | Hanya pemisahan host | **Belum** | **B-01** |
| CI-07 | T | Kunci cosign di secret GitHub dipakai untuk menandatangani image jahat | S | S | Sedang | Akses minimum; tetap ada gerbang orang kedua | Tercermin (pengecualian dibawa ke gate) | NB-02, G-04 |
| KMS-06 | D, E | Unseal Shamir manual: kuorum hilang bila bao-1 dan bao-2 restart bersamaan; kolusi pemegang share | S | T | **Tinggi** | Raft 3 node `[SRC-227]`; cache DEK; peringatan; PT-38; RT-22 | Tercermin (risiko sisa milik user) | G-05 |
| DB-07 | I | Replika penuh dan repo backup di Hostinger, termasuk database `keycloak` dan PII non-envelope | S | T | **Tinggi** | WireGuard + TLS; envelope encryption; pgBackRest terenkripsi | Sebagian | NB-08, G-02 |

---

## 3. Temuan blokir (putaran 1)

### B-01. Runner staging dan kepercayaan pada jaringan privat (CI-06)

**Alasan:**
- 05-stack §3.2 menyatakan runner "aman dipakai karena repositori privat". Sumber yang dirujuk (SRC-241) tidak mendukung kesimpulan itu. Halaman yang sama justru meminta "be cautious when using self-hosted runners on private or internal repositories" (R-4).
- Staging hanya dipisah pada tingkat host, sementara firewall production mempercayai seluruh jaringan privat. Autentikasi Loki `[TIDAK DIKETAHUI]`.

**Usulan:**
- Staging dipindah ke jaringan terpisah.
- Firewall production memakai daftar izin per host dan port.
- Observability diberi autentikasi di depannya.
- Runner dibuat JIT, sekali pakai, dan hanya menerima workflow dari `main`.

### B-02. KMS-01: jalur administratif OpenBao, host kunci, dan konsol penyedia

**Alasan:** empat hal belum dirancang:
- siklus hidup root token;
- hak menulis policy;
- cakupan peran SSH CA;
- kredensial pemilik skema yang dipegang deployer.

Konsol panel penyedia juga belum masuk ADR-021 butir 3. Padahal Login MFA (R-1), `generate-root` berkuorum (R-2), dan Control Groups (R-3) tersedia di OpenBao.

**Usulan:**
- Root token hanya dibuat lewat `generate-root`, lalu dicabut.
- Administrasi OpenBao dijalankan sebagai kode lewat Control Group.
- Peran SSH CA dibuat per kelas host.
- Kredensial pemilik skema dilindungi Control Group.
- Login MFA dipasang dengan batas laju.
- Konsol panel diatur.

---

## 4. Temuan non-blokir putaran 1 (10 terpenting)

| # | Temuan | Usulan |
| --- | --- | --- |
| NB-01 | Isi pemeriksaan orang kedua di gerbang production belum ditetapkan | U-11 dijadikan KEPUTUSAN. Status "lulus staging" dibuktikan dengan attestation yang ditandatangani di sisi staging |
| NB-02 | Kunci cosign di secret repo bisa dibaca workflow dari branch mana pun | Tanda tangan dilakukan di job terpisah; prosedur rotasi ditulis; pertimbangkan penandatanganan di sisi VPS |
| NB-03 | Batas laju per IP dan security log bergantung pada IP klien yang belum pasti | U-10 dijadikan syarat keras; rencana pengganti untuk modul Caddy/CRS; peringatan lonjakan login gagal |
| NB-04 | HTTPS, `Content-Disposition`, dan versi objek NOS belum terverifikasi | Verifikasi sebelum EP berkas; rencana pengganti berupa unduhan lewat backend; bucket berdata pribadi tanpa Object Lock dan tanpa backup penyedia |
| NB-05 | Proxy `bff/[...path]` belum dibatasi; kunci JWE masih tunggal | Daftar izin namespace dan header; kunci JWE dan prefiks ACL Valkey per aplikasi; rotasi ID sesi; batas egress |
| NB-06 | Status GHSA-pc58 tertulis `[TIDAK DIKETAHUI]` padahal SRC-248 menyatakan sudah ditambal; penundaan APP-02/APP-05 tanpa KEPUTUSAN | Samakan teks dengan SRC-248; catat penundaan sebagai KEPUTUSAN |
| NB-07 | Kunci pgBackRest dan snapshot Raft hanya ada di Biznet/OpenBao | Simpan salinan offline kunci; salin snapshot ke Hostinger; siapkan jalur SSH darurat yang tidak bergantung pada OpenBao |
| NB-08 | Backup dan snapshot di tingkat penyedia berada di luar retensi | U-14 dijadikan KEPUTUSAN |
| NB-09 | GlitchTip diekspos tanpa batas path; TLS untuk sinyal LiveKit belum dirancang | Ekspos hanya endpoint ingest; tetapkan terminasi TLS LiveKit |
| NB-10 | Daftar realm publik belum eksplisit; image pihak ketiga belum masuk Trivy dan pin digest; CR-012 belum mencantumkan ADR-018/ADR-007 butir 6/NFR-SEC-05; firewall ADR-025 butir 3 belum memuat host `video` dan WireGuard | Lengkapi keempatnya |

---

## 5. Butir gate untuk user (penerimaan risiko, tidak diputuskan reviewer)

| # | Butir | Pilihan yang perlu diputuskan user |
| --- | --- | --- |
| G-01 | Perutean Magic Transit terhadap T4-Q1, termasuk media video (RT-26, PT-39) | Menerima, meminta Magic Transit dimatikan, atau mengganti ingress/penyedia |
| G-02 | Enkripsi lapis penyedia `[TIDAK DIKETAHUI]` untuk foto KTP/KK, PDF klinis, PII non-envelope termasuk data anak, dan database `keycloak` | Konfirmasi tertulis penyedia; enkripsi berkas oleh aplikasi ditambah enkripsi disk OS; atau menerima risiko |
| G-03 | CI-01: tidak ada review yang tidak dapat dilewati sebelum `main` | Menerima dengan U-11 sebagai kompensasi, atau memakai paket GitHub yang mendukung required reviewers |
| G-04 | Kunci cosign di secret GitHub | Menerima, atau memindahkan penandatanganan ke sisi VPS |
| G-05 | PT-38: jumlah share, ambang, dan pemegang; approver Control Group | Memilih pemegang yang terjangkau dalam RTO; kehilangan share melebihi ambang berarti data hilang permanen |
| G-06 | Credential stuffing pada pasien tanpa MFA bila batas laju Caddy, CRS, atau IP klien tidak tersedia | Menerima risiko sisa, dan/atau MFA sukarela untuk pasien (H-04) |
| G-07 | Tanpa TLS di dalam satu host (tafsiran ADR-013 butir 1) | Pendapat reviewer: dapat diterima dengan jaringan Docker terpisah per sistem, tanpa `network_mode: host` |
| G-08 | MFA konsol panel Biznet Gio dan Hostinger `[TIDAK DIKETAHUI]` | Menerima risiko dengan pemegang minimal, atau mencari penyedia lain |
| G-09 | PT-35: siapa menjalankan patch, unseal, dan failover setelah go-live | Menetapkan pemilik sebelum Tahap 7 |

---

## 6. Catatan untuk putaran 2 (dari putaran 1)

Cek ulang cukup memeriksa baris yang berubah untuk B-01 dan B-02. Threat model diperbarui ke v1.1 (U-09) setelah gate Tahap 5.

---

## 7. Halaman resmi yang dibuka reviewer

| Ref | Kini terdaftar | Sumber | URL | Tanggal akses |
| --- | --- | --- | --- | --- |
| R-1 | SRC-251 | OpenBao Login MFA | https://openbao.org/docs/auth/login-mfa/ | 2026-11-18 |
| R-2 | SRC-252 | OpenBao Tokens (root token, `generate-root`) | https://openbao.org/docs/concepts/tokens/ | 2026-11-18 |
| R-3 | SRC-253 | OpenBao Control Groups | https://openbao.org/docs/concepts/control-groups/ | 2026-11-18 |
| R-4 | SRC-254 | GitHub Secure use reference (self-hosted runner) | https://docs.github.com/en/actions/reference/security/secure-use | 2026-11-18 |

---

## 8. Putaran 2 (cek ulang baris yang berubah)

Yang diperiksa:
- ADR-025 butir 1 (baris 22), butir 3 (baris 70–71), dan butir 7 (baris 98–103);
- tabel deployment audit/privasi (baris 43–49);
- butir 10 (baris 126–127);
- tabel KEPUTUSAN-143–152 (baris 139–148);
- tabel pengganti ADR-021 (baris 168–175);
- 05-stack §2.1 (baris 209), §2.5 (baris 308), §2.6 (baris 331), §3.2 (baris 421–428), §5.1 (baris 510), §5.3 (baris 536);
- pendaftaran SRC-251–SRC-254.

### 8.1 Status blokir

| Blokir | Status | Dasar |
| --- | --- | --- |
| B-01 | **Tertutup** | Staging dipindah ke jaringan privat terpisah tanpa rute ke production (KEPUTUSAN-143). Firewall production memakai default deny dengan daftar izin per host asal, host tujuan, dan port (KEPUTUSAN-144). Observability tidak publik: Grafana di belakang SSO ber-MFA, Loki dan Prometheus hanya menerima koneksi dari `ops` dan agen terautentikasi (KEPUTUSAN-145). Runner staging dibuat JIT sekali pakai, hanya menerima workflow `main`, dan hanya memegang identitas OpenBao staging (KEPUTUSAN-146). Klaim "aman dipakai" diganti kutipan yang bersumber `[SRC-254]`. Mekanisme isolasi jaringan Biznet diverifikasi Tahap 8 (05-stack §13.2 butir 5a), dengan staging hanya berisi data sintetis. Ini sesuai kriteria karena desain sudah ada dan yang tersisa hanya verifikasi kemampuan |
| B-02 | **Tertutup** | Root token hanya dibuat lewat `generate-root` dengan kuorum, lalu dicabut `[SRC-252]` (KEPUTUSAN-147). Login MFA TOTP dipasang dengan batas laju `[SRC-251]` (KEPUTUSAN-148). Administrasi OpenBao dijalankan sebagai kode dengan token admin yang hanya terbit lewat Control Group `[SRC-253]` (KEPUTUSAN-149). Peran SSH CA dibuat per kelas host; host db dan bao hanya bisa diakses lewat Control Group (KEPUTUSAN-150). Kredensial pemilik skema diminta skrip, TTL-nya setara durasi migrasi, dan path-nya dilindungi Control Group (KEPUTUSAN-151). Konsol panel diatur dengan akun bernama, MFA bila tersedia, dan pemegang minimal (KEPUTUSAN-152) |

Akibatnya, status ancaman yang dinilai ulang di §2 berubah:
- CI-06 menjadi **Tercermin**.
- KMS-01 menjadi **Tercermin**, dengan risiko sisa MFA panel di G-08.
- DB-02 menjadi **Tercermin** untuk intinya (isi klinis dan NIK). Lapis penyedia tetap di G-02.
- RD-02 tidak lagi bersyarat B-01. Yang tersisa hanya verifikasi TLS Valkey di Tahap 8.
- Deployment `audit-verify` dan `privacy-worker` yang terpisah, masing-masing dengan identitas OpenBao sendiri, serta relay yang hanya memegang `audit_relay` dan `audit_writer` (baris 43–49), memperkuat AUD-01, AUD-02, dan DB-04. Verifikasi rantai kini independen dari relay.

Tally ancaman Tinggi: **13 Tercermin, 4 Sebagian (CI-01, OS-01, APP-04, DB-07), 0 Belum**. Keempat Sebagian bergantung pada keputusan user (G-02, G-03, G-06) atau verifikasi Tahap 8 yang sudah tertulis.

### 8.2 Status akhir

**LULUS DENGAN CATATAN.** Tidak ada blokir tersisa. Sebelum threat model v1.1 dapat dinyatakan LULUS menurut §0.4, user perlu memutuskan G-01–G-09 di gate Tahap 5. G-05 kini juga mencakup minimal dua orang berwenang di OpenBao, yaitu operator deploy dan approver Control Group (ADR-025 baris 213).

### 8.3 Catatan baru (non-blokir)

1. **Kalimat lama tertinggal di ADR-025 butir 3 baris 69.** Kalimat "hanya dapat dicapai dari jaringan privat" masih ada di samping aturan default deny di baris 70, dan masih belum memuat port publik host `video` dan port WireGuard (lanjutan NB-10 d). Usul: hapus kalimat itu atau rujuk ke baris 70 dan 05-stack §2.5.
2. **Tabel pengganti ADR-019 butir 1 (baris 156) belum diperbarui.** Isinya masih "host terpisah" dengan pemisahan akun `[TIDAK DIKETAHUI]`. Usul: tambahkan rujukan KEPUTUSAN-143 (jaringan terpisah).
3. **Batas KEPUTUSAN-151.** "Nilai tidak ditampilkan ke operator" adalah pengurangan risiko, bukan jaminan. Operator yang punya shell di bastion masih bisa membaca lingkungan proses skrip selama TTL berlaku. Kontrol utamanya adalah persetujuan identitas kedua lewat Control Group dan log audit OpenBao. Usul: catat sebagai risiko sisa, lalu cocokkan penerbitan kredensial dengan daftar migrasi di security log.
4. **Approver Control Group tidak mendapat notifikasi** `[SRC-253]`, dan prosedur pemberitahuannya baru ditetapkan "sebelum go-live". Usul: tenggatnya disamakan dengan PT-38, dan setiap permintaan yang menunggu persetujuan memicu peringatan Alertmanager, supaya break-glass tidak terhambat di luar jam kerja.
5. **NB-01 dan NB-02 masih terbuka.** U-11 (isi pemeriksaan gerbang production) dan rotasi kunci cosign belum menjadi KEPUTUSAN. Keduanya tetap kompensasi yang diharapkan untuk G-03 dan G-04.
