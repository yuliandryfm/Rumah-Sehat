# ADR-027: Situs publik dan member area korporat, yaitu dua aplikasi Next.js, realm Keycloak `korporat`, dan penyajian konten publik

Status: Diterima (2026-11-19, `[USER]` Telegram "1S3 2Y 3Y 4aV 4bY4cT"; review keamanan dan kepatuhan CR-021 LULUS DENGAN CATATAN; fact-check menyusul)
Merevisi bila diterima: ADR-006 butir 5, ADR-011 butir 3, butir 5, dan butir 7, ADR-018 butir 1 (izin bucket CMS publik dicabut), ADR-024 butir 2
Tanggal: 2026-11-19

## Konteks

- User meminta situs web Ruang Sehat bergaya company profile dengan sembilan halaman, kontennya dikelola admin lewat CMS US-1101, dan member area korporat yang dibangun di Fase 1 `[USER] P0-1, P1-1, P3-4` (`docs/cr/CR-021-wawancara.md`).
- Yang masuk member area: HR/admin perusahaan dan karyawan peserta `[USER] P0-2`. Perusahaan dapat mendaftar sendiri lalu diverifikasi, atau dibuatkan admin/sales `[USER] P2-1`. Peserta memakai akun pasien yang sama `[USER] P2-3`.
- ADR-006 butir 5 menetapkan dua web Ruang Sehat (backoffice dan portal rekanan) dengan namespace API dan audience sesi terpisah, dan menolak satu web untuk backoffice dan portal karena kesalahan otorisasi berdampak lebih besar `[DOK: adr/ADR-006-gaya-arsitektur-dan-komponen.md]`.
- ADR-011 butir 3 menetapkan realm per populasi (`pasien`, `nakes`, `admin`, `rme`, `integrasi`). Realm `admin` "hanya undangan". Satu realm untuk semua pengguna ditolak karena kebijakan pendaftaran, MFA, dan sesi berbeda per populasi `[DOK: adr/ADR-011-target-asvs-autentikasi-sesi-mfa.md]`.
- ADR-024 menetapkan Next.js dengan server Next.js sebagai BFF (modul T1) dan pengerasan karena riwayat advisory, termasuk dua RCE berderajat Critical `[SRC-246]` `[DOK: adr/ADR-024-stack-rust-nextjs-flutter.md]`.
- ADR-018 menetapkan object storage privat dengan URL bertanda tangan, dan butir 1-nya membolehkan bucket konten CMS dan katalog dibaca publik `[DOK: adr/ADR-018-berkas-unggahan.md]`.
- ADR-011 butir 5 mengatur jalur pemulihan perangkat hilang untuk pasien (lewat Layanan Pelanggan) serta untuk tenaga kesehatan, admin, dan pengguna RME (prosedur H-05); realm baru belum tercakup `[DOK: adr/ADR-011-target-asvs-autentikasi-sesi-mfa.md]`.
- Review keamanan putaran 1 menemukan XSS tersimpan dari konten CMS di situs publik dan di pratinjau backoffice (SIT-01), dan jalur reset MFA HR yang belum dirancang (KOR-02), keduanya berderajat Tinggi `[DOK: review/CR-021-security.md]`. ADR-012 butir 3 membolehkan endpoint publik tanpa login hanya untuk konten yang memang publik `[DOK: adr/ADR-012-otorisasi-dan-isolasi-data.md]`.
- NFR-SEC-02 mewajibkan MFA untuk semua akun manusia; NFR-SEC-04 sesi web berakhir setelah 30 menit tidak aktif `[DOK: 03-prd.md]`.
- Non-scope BRD §4.2 "Versi web untuk pasien" `[DOK] P3.6a` membuat kanal peserta di web belum dapat diputuskan (PT-60).

## Keputusan

1. **Dua aplikasi Next.js baru** `[KEPUTUSAN-251]`:
   - `web/situs`: situs publik sembilan halaman, dirender server dari endpoint konten publik backend. Tidak membuat cookie sesi, tidak memegang identitas OpenBao, tidak mengakses Valkey, dan tidak memegang kunci JWE.
   - `web/korporat`: member area HR dengan BFF sesuai ADR-011 butir 7 dan ADR-024 butir 3 (klien rahasia realm `korporat`, cookie `HttpOnly`/`Secure`/`SameSite`, CSRF, sesi Valkey, token JWE berkunci OpenBao). Masuk modul kritis T1.
   - Pengerasan ADR-024 butir 2 berlaku untuk keduanya. Keduanya memakai `web/packages/ui` dan token design guideline.
2. **Namespace API dan audience** `[KEPUTUSAN-251]`: API korporat di `/v1/corporate/*` dengan audience sesi sendiri; sesi korporat tidak dapat memanggil API backoffice, portal, atau pasien, dan sebaliknya (pola ADR-006 butir 5). Konten publik di `/v1/public/content/*` tanpa login.
3. **Realm Keycloak keenam `korporat`** untuk HR perusahaan `[KEPUTUSAN-250]`:
   - pendaftaran mandiri lewat backend dengan kompensasi (pola ADR-011 butir 6 untuk pasien) dan undangan aktivasi (pola realm `admin`);
   - verifikasi email wajib; MFA TOTP wajib dengan kode pemulihan sejak login pertama `[KEPUTUSAN-253]`; atur ulang password tidak melewati MFA (ADR-011 butir 5);
   - deteksi brute force, pesan galat seragam, dan kebijakan password sama dengan realm lain (ADR-011 butir 4, NFR-SEC-03);
   - batas sesi web: tidak aktif 30 menit, mutlak 12 jam (ADR-011 butir 7);
   - backend Ruang Sehat memakai klien Admin REST terpisah untuk realm ini dengan peran minimum (ADR-011 butir 2);
   - pemetaan akun: `account.account_type = corporate_admin`, `account.idp_realm = korporat`;
   - **pemulihan MFA HR** `[KEPUTUSAN-265]` (usulan revisi ADR-011 butir 5): jalur mandiri dengan kode pemulihan didahulukan. Bila kode pemulihan juga hilang, reset dimulai sub-peran admin berwenang (PT-67) setelah konfirmasi lewat kanal berbeda dari kanal permintaan (HR aktif lain di perusahaan yang sama, atau kontak resmi perusahaan yang terdaftar saat verifikasi). Reset baru berlaku setelah HR membuka tautan sekali pakai di email akun dan memasukkan password (pola KEPUTUSAN-204); TOTP lama tetap berlaku sampai TOTP baru dan kode pemulihan baru tersimpan (H-16 a); semua sesi HR itu dicabut; semua HR aktif perusahaan diberi tahu; E-04 tercatat. Tidak ada admin yang melihat atau menetapkan kredensial HR. Super Admin tidak punya jalur reset kecuali user memutuskannya. Dibangun di Fase 2 bersama US-207 AC5.
4. **Kanal peserta**: peserta tetap memakai realm `pasien`. Bila user memutuskan peserta juga memakai web (PT-60), ditambah klien web rahasia di realm `pasien` dengan batas sesi web, **di aplikasi web terpisah** dari `web/korporat`, supaya kompromi BFF HR tidak memberi token pasien. Pilihan itu ditetapkan lewat revisi ADR ini.
5. **Konten dan gambar publik** `[KEPUTUSAN-252]` `[KEPUTUSAN-264]`:
   - endpoint publik hanya mengembalikan konten berstatus Tayang untuk kanal situs. Gambar konten tayang disajikan lewat endpoint backend yang memeriksa status konten, dengan batas laju. Draf, jadwal, dan konten terhapus tidak pernah dapat dibaca lewat jalur publik;
   - **render aman** (M-40): Markdown dirender tanpa HTML mentah; tautan hanya berskema `https`, `mailto`, atau relatif; gambar hanya dari endpoint gambar publik backend. Aturan yang sama divalidasi backend saat konten disimpan dan dipakai saat render di situs maupun pratinjau backoffice. Pustaka Markdown/sanitasi dipilih saat task dengan cek registry (anti-halu §5);
   - pratinjau CMS di backoffice ditampilkan di iframe `sandbox` tanpa `allow-same-origin` (origin opak), sehingga skrip apa pun di pratinjau tidak dapat memakai sesi backoffice;
   - Content Security Policy tanpa skrip inline untuk `web/situs` dan backoffice; situs tanpa skrip, font, atau sematan pihak ketiga (M-48);
   - penyajian tanpa `use cache`/Cache Components (ADR-024). Respons konten publik tanpa personalisasi boleh di-cache Caddy dengan TTL pendek dan anggaran batas laju terpisah dari API pasien (M-47); keputusannya di Tahap 8 dengan review keamanan.
6. **Deployment** `[KEPUTUSAN-269]`: dua kontainer tambahan dan rute Caddy per nama host; hanya `web/korporat` yang memperoleh identitas OpenBao (ADR-025). `web/situs` berada di jaringan Docker sendiri dan hanya menjangkau backend lewat Caddy pada `/v1/public/*`, tanpa egress lain (M-46). Cookie sesi `web/korporat` berprefiks `__Host-` (wajib `Secure`, tanpa `Domain`, `Path=/`, hanya dikirim ke host yang menetapkannya `[SRC-408]`), dan CSRF memakai synchronizer token, bukan double-submit cookie. Domain `[TIDAK DIKETAHUI]` (PT-46, PT-70).

## Alternatif yang ditolak

- **Satu aplikasi untuk situs publik dan member area**: situs publik menerima lalu lintas anonim terbanyak. RCE di server Next.js `[SRC-246]` pada aplikasi yang sama akan memberi akses ke sesi HR dan kunci JWE. Biaya pemisahan kecil (satu kerangka aplikasi, ±3 jam-task).
- **Member area di dalam backoffice atau portal rekanan**: pengguna eksternal yang mendaftar sendiri berada di aplikasi yang sama dengan Super Admin atau admin rekanan, bertentangan dengan alasan ADR-006.
- **HR di realm `admin`**: realm `admin` hanya undangan, sedangkan perusahaan dapat mendaftar sendiri `[USER] P2-1`; pendaftaran anonim di realm yang sama dengan Super Admin memperbesar dampak kesalahan konfigurasi.
- **HR di realm `pasien`**: batas sesi pasien mobile 30/90 hari tidak cocok untuk web; mencampur populasi; HR dipaksa membuat akun pasien.
- **Gambar publik lewat bucket atau prefix publik** (dibolehkan ADR-018 butir 1): objek gambar milik draf, konten terjadwal, atau konten yang sudah diturunkan tetap dapat dibaca siapa pun yang mengetahui atau menebak kuncinya, dan status tayang tidak dapat diperiksa per permintaan. Keputusan ini lebih ketat dari ADR-018, sehingga ADR-018 butir 1, H-11 threat model, dan checklist ASVS direvisi agar izin bucket CMS publik dicabut. (Koreksi v0.2: v0.1 keliru menyebut bucket publik "bertentangan dengan ADR-018".)
- **HTML mentah di konten CMS dengan sanitasi di sisi tampilan saja**: pratinjau di origin backoffice tetap menjadi jalur XSS ke sesi admin bila sanitasi tampilan terlewat; validasi saat simpan ditambah render aman yang sama di dua tempat menutup jalur itu (SIT-01).
- **Reset MFA HR oleh Super Admin**: sejalan dengan penutupan blokir review Checkpoint 2 untuk pasien (hanya Layanan Pelanggan yang dapat memulai), jalur Super Admin tidak dibuka kecuali user memutuskannya.
- **Halaman statis tanpa CMS**: user memilih konten lewat CMS `[USER] P3-4`.

## Konsekuensi

- Jumlah web Ruang Sehat menjadi empat (backoffice, portal, situs, korporat) ditambah web RME; ADR-006 butir 5 dan ADR-024 butir 2 direvisi.
- Realm keenam menambah konfigurasi sebagai kode, tema login, test negatif lintas realm, dan pengambil event Keycloak untuk realm ini (ADR-011 butir 10).
- HR yang juga karyawan peserta memegang dua identitas yang tidak ditautkan; hasil peserta tidak pernah tampil di member area HR (BR-143).
- US-1101 (CMS) pindah ke Fase 1.
- Permukaan serangan publik bertambah (RT-45): batas laju pendaftaran, pesan seragam, dan kewajiban patch Next.js ≤ 7 hari (KEPUTUSAN-75) berlaku untuk dua aplikasi baru.
- Modul kritis baru: konfigurasi realm `korporat` (termasuk jalur reset MFA HR), BFF `web/korporat`, dan render aman konten CMS.
- Reset MFA HR oleh admin baru tersedia di Fase 2; sampai itu, HR yang kehilangan TOTP dan kode pemulihan tidak dapat dipulihkan, karena tidak ada jalur darurat lain.
- Satu task Fase 1 tambahan untuk render aman CMS (F1-T111, 4 jam-task) dan satu kelompok kerja Fase 2 untuk reset MFA HR (±12 jam-task).
