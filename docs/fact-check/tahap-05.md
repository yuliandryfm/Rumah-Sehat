# Fact-check Tahap 05: Stack dan Coding Standards

Tanggal: 2026-11-18
Dokumen diperiksa: `docs/05-stack.md` (v0.1), `docs/05-response-codes.md`, `docs/05-physical-schema.md` (v0.2), `docs/coding-standards.md`, `docs/05-asvs-checklist.md` (v0.2), `docs/adr/ADR-024-stack-rust-nextjs-flutter.md`, `docs/adr/ADR-025-hosting-vps-openbao.md`, baris "Perubahan:" ADR-007, -009, -010, -011, -013, -017, -019, -021; CR-012, CR-013, CR-014 dan tiga butir DoD baru di `docs/backlog.md`; RT-21–RT-31 di `docs/04-risk-register.md`; bagian Tahap 5 di `docs/traceability.md`; bagian modul kritis, Stack, dan Perintah di `CLAUDE.md`; `docs/sources.md` SRC-84–SRC-254; `docs/decision-log.md` KEPUTUSAN-74–159
Sumber kebenaran: `docs/05-catatan-stack.md` (T5-Q1–T5-Q12), BRD v1.3, PRD v1.1, ADR-001–ADR-023, `docs/04-data-dictionary.md`, `docs/01-business-case.md`
Tingkat ketelitian: T1 untuk ADR-025, `05-physical-schema.md`, bagian keamanan/CI/secrets di `05-stack.md` (§2, §3, §5), dan `05-asvs-checklist.md`; T2 untuk lainnya
Hasil pemeriksaan awal: **TIDAK LULUS** (3 blokir: FC5-B01–B03; ketiganya perbaikan register/label yang faktanya sudah dicek benar oleh fact-checker; 5 non-blokir)
Hasil cek ulang: **LULUS**. FC5-B01–B03 tertutup; lihat bagian "Cek ulang" di akhir.

Catatan metode:
- Fact-checker tidak punya akses shell; `git diff` tidak dijalankan. Lokasi diperiksa dari daftar orkestrator.
- `05-physical-schema.md` (sekitar 1.450 baris) dibaca penuh untuk §0–§3, §8, §9, §10–§15; bagian lain dipindai lewat label. Penanda **(V8)** diperlakukan sebagai `[TIDAK DIKETAHUI]` yang dijadwalkan diverifikasi Tahap 8 Fase 1, sehingga tidak dihitung sebagai klaim tanpa sumber.
- `coding-standards.md` dibaca §1.1–§1.2 dan §5–§7.1; label `[SRC]` seluruh berkas dipindai dan dicocokkan dengan register. `05-asvs-checklist.md` dibaca §0–§1 dan §3.4–§7.

## Ringkasan
- Total klaim: sekitar 1.300 label (perkiraan): `05-stack.md` ±400, ADR-025 ±150, ADR-024 ±70, physical schema ±250, coding standards ±150, ASVS ±200, response codes ±50; ditambah 171 baris SRC dan 86 baris KEPUTUSAN.
- Terverifikasi: semua yang dicek, kecuali temuan di bawah.
- Tanpa sumber: 1 blokir (FC5-B03, nama opsi `sendDefaultPii`).
- Sumber tidak sah atau tidak lengkap: 2 blokir (FC5-B01 SRC-156 tanpa URL; FC5-B02 SRC-207 dari ringkasan endpoint tanya-jawab), ditambah 1 URL yang tidak memuat kutipan register (FC5-N02, faktanya benar).
- Kontradiksi dengan dokumen lulus gate yang tidak dicatat: 0 blokir. Penyimpangan dari ADR-013 butir 1, ADR-018 butir 5, ADR-019, ADR-021 butir 4, dan NFR-PERF-04 tercatat sebagai CR-012 atau tafsiran gate (`05-stack.md` §14); pencatatannya belum lengkap di CR-012 dan ADR-025 (FC5-N04).
- Fakta bisnis berlabel asumsi/keputusan: 0. Tidak ada `[ASUMSI]` di seluruh `docs/`. Nilai yang menyentuh bisnis (jadwal sewa, cakupan 80%, SLA Moderate 30 hari, daftar izin lisensi) ditandai "dikonfirmasi user di gate".
- Nomor tidak terdaftar atau ganda: 0. SRC-84–254 lengkap 171 baris tanpa celah (SRC-208 dan SRC-215 dibatalkan secara tercatat). KEPUTUSAN-1–159 lengkap 159 baris tanpa ganda. Tidak ada rujukan SRC ≥ 255, KEPUTUSAN ≥ 160, atau ASUMSI di `docs/`.
- TANPA INDUK: 0. CR-012 dan CR-014 tidak menambah story. CR-013 terpetakan ke AB-11, AB-12, US-801, US-812 di traceability.

## Hal yang diperiksa dan bersih
- **Label `[USER] T5-Qx`** cocok dengan catatan: T5-Q1 "Rust + Next.js"; masukan "vpsnya hostinger dan biznet gio saja"; T5-Q2 "Biznet utama+Hostinger"; T5-Q3 "DOKU utama, Xendit cadangan"; T5-Q4 "Amazon SES Jakarta"; T5-Q5 "Lalamove saja"; T5-Q6 "GitHub + agen deploy VPS"; T5-Q7 "Penuh waktu sampai go-live"; T5-Q8 "Persetujuan di sisi VPS"; T5-Q9 dan T5-Q10 "Di luar anggaran proyek"; T5-Q11/T5-Q11a "Di atas harga rekanan+markup"; T5-Q12 "Rumah Sehat". Rincian penyusun di bawah label USER dicatat sebagai butir gate.
- **Perhitungan biaya §11 dihitung ulang dan benar.** Biznet production 2×999.000 + 1.799.000 + 699.000 + 559.000 + 179.000 + 999.000 + 2×139.000 + 269.000 + 80.000 = Rp6.860.000 (11 host). Hostinger 426.900 + 116.900 = 543.800 (perpanjangan 1.047.800). Staging 289.000 + 269.000 = 558.000. Total bulanan Rp8.261.800. Tambahan uji beban 6.302.000. Dibayar di muka 24 bulan 10.245.600 + 2.805.600 = 13.051.200. Sampai go-live 15.679.800 (basis bulanan) dan 28.187.200 (basis kas). 12 bulan 99.141.600 dan 92.616.000. Total dengan developer Rp49.679.800–83.679.800 + USD 124. Satu kalimat penjelasan selisih bermakna ganda (FC5-N03).
- **Harga di halaman resmi dibuka ulang dan cocok:** NEO Lite Pro MS.4.2 Rp559.000, MM.4.4 Rp699.000, MM.8.4 Rp999.000 (4/8/80), ML.16.8 Rp1.799.000 (8/16/90) (SRC-94). NEO Lite SS 2.1 Rp80.000, MS 4.2 Rp139.000, MS 4.4 Rp179.000, MM 8.4 Rp269.000 (coret Rp399.000), MM 8.8 Rp289.000 (SRC-93). NEO Object Storage "Rp 1.000 /GB/month" single region dan "100% S3 Compatibel" (SRC-97). Hostinger KVM 1 Rp116.900 (perpanjangan Rp193.900, "Hanya bayar Rp2.805.600 untuk 24 bulan") dan KVM 8 Rp426.900 (perpanjangan Rp853.900, Rp10.245.600 untuk 24 bulan) (SRC-87; URL `.co.id` kini dialihkan ke `hostinger.com/id/`).
- **Business Case**: mobile developer Rp34–68 juta sampai go-live sesuai `01-business-case.md` baris 130. Toleransi waktu 0 sesuai R-02. `[USER] P7.4, P5.5` di ADR-024 sesuai brief baris 134 dan 148.
- **Fakta eksternal penentu dibuka ulang dan didukung halaman:**
    - **Next.js 16.4.6**: registry npm `version` "16.4.6", `license` "MIT", `engines` `{"node":">=20.9.0"}`. Isi benar, tetapi baris register cacat (FC5-B01).
    - **SRC-248**: GHSA-pc58-hwjq-p385 "Moderate" (CVSS v4 6.3), affected ">= 16.0.0, <= 16.4.4" dan "<= 15.5.28", patched 16.4.5 dan 15.5.29, published Nov 3, 2026. Klaim "16.4.6 tidak terdampak" didukung.
    - **SRC-246**: dua Critical (GHSA-2464-fhr4-w9xw Oct 14, 2026; GHSA-vcvr-r3jv-pc5j Sep 22, 2026), tujuh Moderate tertanggal Nov 3, 2026, satu Moderate Sep 30, 2026.
    - **SRC-121**: PostgreSQL "Tested Version" 18, "Supported Versions" "18.x, 17.x, 16.x, 15.x, 14.x"; pemilih rilis memuat 26.8.2.
    - **SRC-222**: "Old data can still be decrypted due to the use of a key ring"; `min_decryption_version` mengarsipkan versi lama dan dapat diturunkan saat darurat; `rewrap` "does not reveal the plaintext data"; datakey "encrypted with the named key". Klaim ADR-025 butir 7 bahwa ADR-013 butir 5 terpenuhi didukung halaman.
    - **SRC-253**: "second set of eyes"; "implement second-party authorization by means of policy" factors pada path; "OpenBao does not send any notification to the approvers". Halaman tidak menyebut pembatasan edisi.
    - **SRC-251**: "built on top of the Identity system"; brute force TOTP tidak dilindungi secara bawaan; anjuran batas laju pada `/sys/mfa/validate`; bawaan 5 percobaan gagal.
    - **SRC-99**: "Mendukung mode Governance, Compliance maupun Legal Holds"; objek di mode Compliance "sudah tidak dapat lagi di rubah maupun di hapus".
    - **SRC-85**: "For audiences in Asia, we offer VPS servers in India, Indonesia, and Malaysia"; "from Boston to Jakarta"; "All plans are paid upfront"; masa 24 bulan.
    - **SRC-84/209/210**: endpoint API `email.ap-southeast-3.amazonaws.com`; catatan "SMTP endpoints are not currently available in ... Asia Pacific (Jakarta)"; inbound, DKIM, tracking, dan feedback Jakarta tercantum.
    - **SRC-240**: lihat FC5-N02. Faktanya benar menurut halaman reference GitHub: "If you are on a GitHub Free, GitHub Pro, or GitHub Team plan, required reviewers are only available for public repositories."
- **Physical schema**: total pool §10.3 dihitung ulang (a) 136, (c) 70, (b) 38, semuanya di bawah `max_connections` 200/120/80. KEPUTUSAN-90–109, 139–142, dan 153–159 di §15 sama dengan decision-log. Perilaku PostgreSQL yang belum ber-SRC diberi penanda (V8) dan dikumpulkan di §13. KEPUTUSAN-159 (tanpa backdate `effective_from`) konsisten dengan AB-17. Ukuran berkas 5/10 MB di coding standards sesuai NFR-SEC-08.
- **Risk register RT-21–RT-31** konsisten dengan ADR-024 dan ADR-025, dan nomornya dirujuk dengan benar (RT-21–RT-29, RT-31). **Traceability** Tahap 5 memetakan NFR ke stack tanpa butir yatim.
- **Klaim "teruji/terbukti"**: tidak ada yang melebihi bukti. Status "Terpenuhi" di `05-stack.md` §12 adalah status rancangan dengan sumber, dan kompatibilitas Valkey/pgBackRest/Loki dinyatakan "dibuktikan Tahap 8".

## Temuan blokir

| ID | File dan baris | Klaim | Masalah | Kategori §2 | Usulan perbaikan |
| --- | --- | --- | --- | --- | --- |
| FC5-B01 | `sources.md` baris 164 (SRC-156); dirujuk `05-stack.md` baris 60, ADR-024 baris 34, `coding-standards.md` baris 180, ASVS baris 110, KEPUTUSAN-75 | Next.js 16.4.6, MIT, `engines` node ≥ 20.9.0 | Kolom URL **kosong**. Kolom kutipan terpotong menjadi "peerDependencies `react ^18.2.0. Kutipan/output: 19.0.0-rc-de68d2f4-20241204" (karakter `\|\|` merusak tabel). Gate anti-halu §8 mensyaratkan setiap SRC tercatat dengan URL. Versi ini menjadi dasar klaim keamanan "16.4.6 sudah memuat tambalan GHSA-pc58". Fact-checker membuka `https://registry.npmjs.org/next/latest` dan isinya benar | 2 | Isi URL `https://registry.npmjs.org/next` (atau `/latest`). Perbaiki kutipan: `"version":"16.4.6"`, `"license":"MIT"`, `"engines":{"node":">=20.9.0"}`, dengan peerDependencies react ditulis tanpa karakter pipa. Cek ulang cukup baris ini |
| FC5-B02 | `sources.md` baris 215 (SRC-207); `05-stack.md` baris 623 (§10, baris DOKU) | "retry notifikasi hingga 6 kali `[SRC-207]`" | Register sendiri menulis kutipan itu berasal "dari ringkasan ask-endpoint docs.doku.com". SRC-208 dibatalkan dengan alasan yang sama ("bukan halaman dokumentasi"). Jadi klaim ini tidak punya sumber resmi yang sah. Bagian lain SRC-207 (DOKU Public Key) tidak terdampak | 2 | Hapus "retry notifikasi hingga 6 kali" dari SRC-207 dan `05-stack.md` §10, lalu pindahkan ke "Pencarian tanpa hasil" dan ke daftar konfirmasi PT-36. Cara lain: buka halaman dokumentasi DOKU yang memuat kalimat itu dan kutip dari sana |
| FC5-B03 | ADR-024 baris 62; ADR-025 baris 180 (tabel ADR-010 butir 4); `coding-standards.md` baris 424; ASVS baris 325 | sentry_flutter "dengan `sendDefaultPii` dimatikan `[SRC-143]`" | Nama opsi API ini tidak ada di register. SRC-143 hanya memuat versi, lisensi, dan syarat Flutter. `05-stack.md` baris 101 sendiri menulis "nama opsinya diverifikasi Tahap 8 Fase 1", sehingga dokumen saling bertentangan. Ini kontrol PII (ADR-010 butir 4) di dokumen T1. Fact-checker membuka `https://docs.sentry.io/platforms/dart/guides/flutter/configuration/options/`: opsi `sendDefaultPii` ada ("If this flag is enabled, certain personally identifiable information (PII) is added by active integrations"), bawaan `false` | 2 | Daftarkan halaman Sentry itu sebagai SRC baru dan rujuk dari keempat lokasi, lalu selaraskan `05-stack.md` baris 101 dan §13.2 butir 13. Cara lain: ganti keempat lokasi dengan rumusan `05-stack.md` ("opsi pengiriman PII bawaan dimatikan; nama opsi diverifikasi Tahap 8 Fase 1") |

## Temuan non-blokir (5 butir terpenting)

| ID | Lokasi | Masalah | Usulan |
| --- | --- | --- | --- |
| FC5-N01 | ADR-025 baris 69; ADR-025 baris 96; ADR-024 baris 70 dan 75; `05-stack.md` baris 7 dan 47–48; ASVS baris 57–58 | Sisa teks dari sebelum perbaikan review DBA dan security: (a) ADR-025 butir 3 masih "hanya load balancer (443) dan bastion (22 ...) yang publik", padahal `05-stack.md` §2.5 dan ASVS §6 butir 1 menyatakan host `video` wajib publik untuk WebRTC/TURN. ADR-025 adalah dokumen yang akan "Diterima"; (b) daftar identitas proses ADR-025 butir 7 tanpa `audit-verify`; (c) pohon monorepo ADR-024 §5 masih `apps/rme-backend/ (api, worker)` dan `db/roles/ ... (dijalankan pemilik skema)`, bertentangan dengan KEPUTUSAN-89, KEPUTUSAN-97, dan physical schema §3.5; (d) `05-stack.md` baris 48 mengulang `apps/rme-backend` dengan "api, worker" saja; (e) ASVS §0.4 masih `rs-backend` api/realtime/worker/audit-relay dan `rme-backend` api/worker; (f) header `05-stack.md` masih "T5-Q1–T5-Q8, KEPUTUSAN-74–89, SRC-84–247" | Selaraskan dengan KEPUTUSAN-74, -89, -97, -141, -153, dan §2.5. Untuk ADR-025 butir 3, tambahkan host `video` (port TURN 3478/443, port media ditetapkan Tahap 8) |
| FC5-N02 | ADR-024 baris 93; `sources.md` baris 248 (SRC-240), 230 (SRC-222), 168 (SRC-160), 92/217/224 (SRC-84, 209, 216); `05-stack.md` baris 110 | Presisi rujukan: (a) ADR-024 menulis chrono tidak dipelihara "(catatan riset di `[SRC-177]`)", padahal advisory ada di SRC-250 (`05-stack.md` §14.5 sudah mengoreksi decision-log, tetapi ADR-024 belum); (b) halaman URL SRC-240 (manage-environments) saat dibuka ulang **tidak memuat** kutipan register. Kalimat setara ada di `https://docs.github.com/en/actions/reference/workflows-and-actions/deployments-and-environments`; (c) kutipan SRC-222 belum memuat "Old data can still be decrypted due to the use of a key ring", padahal kalimat itu dasar klaim ADR-013 butir 5; (d) SRC-160 menggandakan SRC-129 (React) dengan kutipan "lihat E1"; (e) kutipan SRC-84, 209, dan 216 terpotong oleh karakter pipa; (f) lisensi PostgreSQL ditulis sebagai nilai padahal "belum diverifikasi" | Ganti rujukan ADR-024 ke SRC-250. Perbaiki URL atau tambah URL kedua di SRC-240. Lengkapi kutipan SRC-222. Rujuk SRC-129 dan tandai SRC-160 duplikat. Escape pipa. Tulis lisensi PostgreSQL `[TIDAK DIKETAHUI]` seperti Node.js |
| FC5-N03 | `05-stack.md` baris 689 | "Selisih kedua basis (Rp5.981.800) ... (11 × 543.800)" diletakkan tepat di bawah tabel §11.4, padahal selisih kedua kolom §11.4 adalah 99.141.600 − 92.616.000 = Rp6.525.600 (12 × 543.800). Angka 5.981.800 benar hanya sebagai selisih kumulatif §11.3 + §11.4 (114.821.400 vs 120.803.200) | Tulis "selisih kumulatif sampai go-live dan 12 bulan operasi", atau ganti dengan Rp6.525.600 untuk §11.4 saja |
| FC5-N04 | `backlog.md` baris 320 (CR-012); ADR-025 tabel "Butir ADR lama yang diubah"; ADR-021 baris 4 | Daftar dampak belum lengkap: ADR-018 butir 5 (enkripsi lapis penyedia dengan KMS, versi objek), tafsiran ADR-013 butir 1 (tanpa TLS di dalam satu host), tafsiran NFR-PERF-04, perubahan arsitektur §6/§7/§10, dan ADR-007 butir 6 (cadangan batas laju ke Caddy, ada di tabel ADR-025 tetapi tidak di CR-012). Semuanya sudah dibawa ke gate di `05-stack.md` §14, jadi tidak blokir. Baris "Perubahan:" ADR-021 hanya menyebut T5-Q2, padahal butir 4 diubah oleh T5-Q6 dan T5-Q8 | Tambahkan butir-butir itu ke CR-012 dan tabel ADR-025, dan tambahkan T5-Q6/T5-Q8 di baris Perubahan ADR-021 |
| FC5-N05 | `CLAUDE.md` baris 49 dan 64; ADR-024 baris 54; `backlog.md` baris 321; `05-stack.md` §13.3 | (a) Modul kritis Tahap 5 sudah ditulis di `CLAUDE.md`, padahal ADR-024 dan CR-012 menyatakan "diperbarui setelah gate Tahap 5"; (b) perintah `sqlx migrate run --source ...` di `CLAUDE.md` belum ber-SRC (physical schema §11.1 menandainya V8), walaupun bagian itu sudah diberi catatan "dikunci Tahap 8 Fase 1"; (c) §13.3 menulis "nomor RT ditetapkan saat dicatat", padahal RT-21–RT-31 sudah dicatat | Samakan pernyataan waktu pembaruan `CLAUDE.md`, tandai perintah migrasi "diverifikasi Tahap 8 Fase 1", dan rujuk RT-21–RT-31 di §13.3 |

## Butir yang perlu ditampilkan ke user di gate
1. **Valkey menggantikan "Redis"** (T4-Q4) `[KEPUTUSAN-77]`. Butir ini belum diputuskan user.
2. **Cloudflare Magic Transit terhadap T4-Q1 "tanpa kecuali"** (RT-26, PT-39). Lokasi perutean belum diketahui.
3. **Baseline biaya**: total ±Rp8,26 juta/bulan, ukuran host masih estimasi penyusun, dan **jadwal mulai sewa** (staging Desember 2026, uji beban Januari 2027, production 1 Februari 2027, Hostinger dibayar 24 bulan di muka Rp13.051.200) `[KEPUTUSAN-78]`. Komponen yang belum bernilai: LB, PPN, SES, domain, Lalamove, MDR, audit keamanan eksternal. Baseline acuan +20% ditetapkan user.
4. **Rincian penyusun di bawah label `[USER]`**:
    - T5-Q9: "ditanggung PT. Arunika" (`05-stack.md` baris 698). User hanya menjawab "Di luar anggaran proyek".
    - T5-Q11a/CR-013: tarif pajak sebagai parameter berlaku-mulai yang "diatur Keuangan". Peran pengatur tarif adalah rincian penyusun, polanya sama dengan ADR-015 butir 8 di gate Tahap 4.
    - T5-Q4: akun AWS khusus email berhak minimum adalah catatan penyusun.
5. **Penyesuaian ADR-021 butir 4**: "review yang tidak dapat dilewati" sebelum `main` hanya dapat dipaksa bila paket GitHub mendukung (status "Sebagian"). KEPUTUSAN-151 juga membuat migrasi production butuh **dua identitas berwenang** di OpenBao (operator dan penyetuju Control Group), lebih banyak dari satu "orang kedua" di T5-Q8. Tambahan lain: kunci privat cosign disimpan di secret GitHub.
6. **PT-38**: jumlah, ambang, dan pemegang share unseal OpenBao; orang kedua untuk deploy dan akses darurat; prosedur akses host saat OpenBao tersegel (N-23).
7. **Tafsiran atas dokumen yang lulus gate**:
    - BFF di server Next.js (ADR-011 butir 7). Akibatnya server Next.js menjadi modul T1.
    - NFR-PERF-04.
    - Tanpa TLS di dalam satu host (ADR-013 butir 1).
    - US-703 AC3 dengan Lalamove saja.
    - Prosedur `pr_archive_detach` sebagai pemakaian kredensial pemilik di luar langkah migrasi (F-P07).
    - CR-014, yaitu perubahan teknis data dictionary.
8. **Enkripsi saat tersimpan di penyedia** (disk VPS dan NEO Object Storage) belum diketahui, padahal ADR-013 butir 2 dan ADR-018 butir 5 mengandalkannya. Begitu pula versioning NOS. Pilihannya ada di ASVS §6 butir 5.
9. **Butir yang ditunda dari Tahap 5 ke Tahap 8**, padahal ADR menyerahkannya ke Tahap 5:
    - metode uji lalu lintas build mobile dan mekanisme pengecualian cadangan atau penyembunyian layar (ADR-021 butir 1–2);
    - mTLS klien integrasi (ADR-014 butir 3, ASVS §6 butir 4);
    - butir DD §D.4 a, b, e;
    - checklist ASVS yang baru sampai tingkat bab, belum per requirement (U-01).
10. **Angka usulan yang perlu dikonfirmasi**: cakupan 80% baris untuk crate T1 (KEPUTUSAN-134) dan SLA patch Moderate ≤ 30 hari (KEPUTUSAN-138). Daftar izin lisensi (KEPUTUSAN-136) juga perlu dikonfirmasi, termasuk pemakaian Grafana/Loki AGPL-3.0 dan OpenBao/OpenTofu MPL-2.0 tanpa modifikasi. Penilaian AGPL "untuk keperluan internal" adalah penilaian penyusun, bukan legal.
11. **RT-31**: volume audit dapat melebihi disk `db-audit` 60 GB. Pilihannya (biaya atau jendela daring lebih pendek) diputuskan setelah uji beban.
12. **Risiko komponen satu host** (`cache`, `video`, `ops`, `bastion`) dan ketiadaan WAF terkelola serta antivirus. Belum semuanya punya nomor RT.

## URL yang benar-benar dibuka (2026-11-18)
1. https://registry.npmjs.org/next/latest (SRC-156)
2. https://github.com/vercel/next.js/security/advisories/GHSA-pc58-hwjq-p385 (SRC-248)
3. https://github.com/vercel/next.js/security/advisories (SRC-246)
4. https://www.keycloak.org/server/db (SRC-121)
5. https://openbao.org/docs/secrets/transit/ (SRC-222)
6. https://openbao.org/docs/concepts/control-groups/ (SRC-253)
7. https://openbao.org/docs/auth/login-mfa/ (SRC-251)
8. https://kb.biznetgio.com/neo-object-storage/mengenal-object-lock-pada-layanan-neo-object-storage (SRC-99)
9. https://docs.aws.amazon.com/general/latest/gr/ses.html (SRC-84, SRC-209, SRC-210)
10. https://www.hostinger.com/vps-hosting (SRC-85, SRC-86)
11. https://www.hostinger.com/id/vps-hosting (SRC-87; URL register `hostinger.co.id` dialihkan 301 ke sini)
12. https://www.biznetgio.com/en/product/neo-lite-pro (SRC-94)
13. https://www.biznetgio.com/en/product/neo-lite (SRC-93, SRC-95)
14. https://www.biznetgio.com/en/product/neo-object-storage (SRC-97)
15. https://docs.github.com/en/actions/how-tos/deploy/configure-and-manage-deployments/manage-environments (SRC-240; kutipan register **tidak ditemukan**, dibuka dua kali)
16. https://docs.github.com/en/actions/reference/workflows-and-actions/deployments-and-environments (kalimat setara SRC-240 ditemukan)
17. https://docs.sentry.io/platforms/dart/guides/flutter/configuration/options/ (untuk FC5-B03; kini terdaftar sebagai SRC-255)

Tidak dibuka ulang (kutipan register hanya dicocokkan dengan klaim): SRC-86 sampai SRC-92, SRC-96, SRC-98, SRC-100 sampai SRC-120, SRC-122 sampai SRC-220 (kecuali yang tercantum di atas), SRC-223 sampai SRC-245, SRC-247, SRC-249, SRC-250, SRC-252, dan SRC-254.

---

## Cek ulang (2026-11-18)

Cakupan terbatas: hanya baris yang berubah untuk menutup FC5-B01–B03 (`aturan-kerja.md` §2), ditambah sampel baris non-blokir yang dilaporkan sudah dirapikan. Tidak ada URL baru yang dibuka. Isi SRC-156 dan SRC-255 dicocokkan dengan halaman yang dibuka fact-checker sebelumnya (URL nomor 1 dan 17 di atas).

| ID | Status | Bukti |
| --- | --- | --- |
| FC5-B01 | **Tertutup** | `sources.md` baris 164: URL `https://registry.npmjs.org/next/latest`. Kutipan memuat 16.4.6, MIT, `engines.node` ">=20.9.0", dan peerDependencies react dengan pemisah "atau" sebagai pengganti pipa. Isinya sama dengan hasil registry yang dibuka fact-checker |
| FC5-B02 | **Tertutup** | SRC-207 (`sources.md` baris 215) kini hanya memuat DOKU Public Key dan algoritma yang belum ditemukan, dengan catatan penghapusan klaim "retry notifikasi hingga 6 kali". Pencarian "6 kali" di `docs/` tidak menemukan sisa klaim itu di dokumen proyek, termasuk `05-stack.md` §10 |
| FC5-B03 | **Tertutup** | SRC-255 terdaftar (`sources.md` baris 263): URL Sentry Flutter options, kutipan "If this flag is enabled, certain personally identifiable information (PII) is added by active integrations", default `false`, tanggal akses 2026-11-18. Kutipan sesuai halaman yang dibuka fact-checker. Label `[SRC-255]` ada di ADR-024 baris 62, ADR-025 baris 180, `coding-standards.md` baris 424, dan `05-asvs-checklist.md` baris 325 |

Non-blokir yang diperiksa sampel:
- **Selesai**:
    - SRC-240 kini memuat URL halaman reference deployments-and-environments (FC5-N02 b).
    - ADR-024 §5 baris 70 dan 75 sesuai KEPUTUSAN-89 dan KEPUTUSAN-97 (FC5-N01 c).
    - ADR-025 baris 69 memuat port WebRTC/TURN host `video` (FC5-N01 a).
    - Kalimat selisih di `05-stack.md` baris 689 membedakan Rp6.525.600 (kolom §11.4) dari Rp5.981.800 (kumulatif) (FC5-N03).
- **Sisa, non-blokir** (tidak memicu putaran ulang):
    - `05-stack.md` baris 48 masih mengulang `apps/rme-backend` dengan "`api`, `worker`" saja. Baris ini bertentangan dengan baris 47.
    - `05-stack.md` baris 101 belum merujuk SRC-255 dan masih menulis "nama opsinya diverifikasi Tahap 8 Fase 1". §13.2 butir 13 juga masih memuat "nama opsi PII sentry_flutter". Tulisan ini tidak salah, tetapi sudah usang sejak SRC-255 terdaftar.

**Status akhir: LULUS.** Tidak ada blokir tersisa. Butir gate di bagian "Butir yang perlu ditampilkan ke user di gate" tetap berlaku.
