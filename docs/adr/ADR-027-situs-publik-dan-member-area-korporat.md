# ADR-027: Situs publik dan member area korporat, yaitu dua aplikasi Next.js, realm Keycloak `korporat`, dan penyajian konten publik

Status: **Diusulkan** (CR-021, 2026-11-19; menunggu persetujuan user bersama `docs/cr/CR-021-analisis.md`)
Merevisi bila diterima: ADR-006 butir 5, ADR-011 butir 3 dan butir 7, ADR-018 (gambar konten publik), ADR-024 butir 2
Tanggal: 2026-11-19

## Konteks

- User meminta situs web Ruang Sehat bergaya company profile dengan sembilan halaman, kontennya dikelola admin lewat CMS US-1101, dan member area korporat yang dibangun di Fase 1 `[USER] P0-1, P1-1, P3-4` (`docs/cr/CR-021-wawancara.md`).
- Yang masuk member area: HR/admin perusahaan dan karyawan peserta `[USER] P0-2`. Perusahaan dapat mendaftar sendiri lalu diverifikasi, atau dibuatkan admin/sales `[USER] P2-1`. Peserta memakai akun pasien yang sama `[USER] P2-3`.
- ADR-006 butir 5 menetapkan dua web Ruang Sehat (backoffice dan portal rekanan) dengan namespace API dan audience sesi terpisah, dan menolak satu web untuk backoffice dan portal karena kesalahan otorisasi berdampak lebih besar `[DOK: adr/ADR-006-gaya-arsitektur-dan-komponen.md]`.
- ADR-011 butir 3 menetapkan realm per populasi (`pasien`, `nakes`, `admin`, `rme`, `integrasi`). Realm `admin` "hanya undangan". Satu realm untuk semua pengguna ditolak karena kebijakan pendaftaran, MFA, dan sesi berbeda per populasi `[DOK: adr/ADR-011-target-asvs-autentikasi-sesi-mfa.md]`.
- ADR-024 menetapkan Next.js dengan server Next.js sebagai BFF (modul T1) dan pengerasan karena riwayat advisory, termasuk dua RCE berderajat Critical `[SRC-246]` `[DOK: adr/ADR-024-stack-rust-nextjs-flutter.md]`.
- ADR-018 menetapkan object storage privat dengan URL bertanda tangan `[DOK: adr/ADR-018-berkas-unggahan.md]`. ADR-012 butir 3 membolehkan endpoint publik tanpa login hanya untuk konten yang memang publik `[DOK: adr/ADR-012-otorisasi-dan-isolasi-data.md]`.
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
   - pemetaan akun: `account.account_type = corporate_admin`, `account.idp_realm = korporat`.
4. **Kanal peserta**: peserta tetap memakai realm `pasien`. Bila user memutuskan peserta juga memakai web (PT-60), ditambah klien web rahasia di realm `pasien` dengan batas sesi web, di aplikasi terpisah atau di `web/korporat` dengan audience terpisah. Pilihan itu ditetapkan lewat revisi ADR ini.
5. **Konten dan gambar publik** `[KEPUTUSAN-252]`: endpoint publik hanya mengembalikan konten berstatus Tayang untuk kanal situs. Gambar konten tayang disajikan lewat endpoint backend yang memeriksa status konten, dengan batas laju; object storage tetap privat (ADR-018). Draf, jadwal, dan konten terhapus tidak pernah dapat dibaca lewat jalur publik. Penyajian tanpa `use cache`/Cache Components (ADR-024); cache HTTP di depan aplikasi, bila diperlukan untuk kinerja, ditetapkan di Tahap 8 dengan review keamanan.
6. **Deployment**: dua kontainer tambahan dan rute Caddy per nama host; hanya `web/korporat` yang memperoleh identitas OpenBao (ADR-025). Domain `[TIDAK DIKETAHUI]` (PT-46, PT-70).

## Alternatif yang ditolak

- **Satu aplikasi untuk situs publik dan member area**: situs publik menerima lalu lintas anonim terbanyak. RCE di server Next.js `[SRC-246]` pada aplikasi yang sama akan memberi akses ke sesi HR dan kunci JWE. Biaya pemisahan kecil (satu kerangka aplikasi, ±3 jam-task).
- **Member area di dalam backoffice atau portal rekanan**: pengguna eksternal yang mendaftar sendiri berada di aplikasi yang sama dengan Super Admin atau admin rekanan, bertentangan dengan alasan ADR-006.
- **HR di realm `admin`**: realm `admin` hanya undangan, sedangkan perusahaan dapat mendaftar sendiri `[USER] P2-1`; pendaftaran anonim di realm yang sama dengan Super Admin memperbesar dampak kesalahan konfigurasi.
- **HR di realm `pasien`**: batas sesi pasien mobile 30/90 hari tidak cocok untuk web; mencampur populasi; HR dipaksa membuat akun pasien.
- **Gambar publik lewat bucket atau prefix publik**: menambah kebijakan akses publik di object storage dan berisiko membuka objek draf; bertentangan dengan ADR-018.
- **Halaman statis tanpa CMS**: user memilih konten lewat CMS `[USER] P3-4`.

## Konsekuensi

- Jumlah web Ruang Sehat menjadi empat (backoffice, portal, situs, korporat) ditambah web RME; ADR-006 butir 5 dan ADR-024 butir 2 direvisi.
- Realm keenam menambah konfigurasi sebagai kode, tema login, test negatif lintas realm, dan pengambil event Keycloak untuk realm ini (ADR-011 butir 10).
- HR yang juga karyawan peserta memegang dua identitas yang tidak ditautkan; hasil peserta tidak pernah tampil di member area HR (BR-143).
- US-1101 (CMS) pindah ke Fase 1.
- Permukaan serangan publik bertambah (RT-45): batas laju pendaftaran, pesan seragam, dan kewajiban patch Next.js ≤ 7 hari (KEPUTUSAN-75) berlaku untuk dua aplikasi baru.
- Modul kritis baru: konfigurasi realm `korporat` dan BFF `web/korporat`.
