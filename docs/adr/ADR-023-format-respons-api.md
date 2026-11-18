# ADR-023: Envelope respons API dengan `response_code` 7 digit, dengan kode HTTP tetap dipakai

Status: Diterima (gate Tahap 4, 2026-11-18 `[USER] T4-G5`)
Tanggal: 2026-11-18

## Konteks

- Semua API memakai REST/JSON dan didokumentasikan dengan OpenAPI (ADR-014, ADR-022). Galat sebelumnya ditetapkan memakai Problem Details RFC 9457 `[SRC-47]`.
- RFC 9457 memakai media type `application/problem+json`. Member standarnya `type`, `status`, `title`, `detail`, dan `instance`. Definisi jenis masalah boleh menambah member lain ("Problem type definitions MAY extend the problem details object with additional members") `[SRC-47]`.
- User meminta envelope yang menyimpan kode respons berupa string numerik dan deskripsi respons, dengan field status HTTP dilepas dari isi respons. Kode HTTP tetap dipakai di header `[USER] T4-G5` (`docs/04-catatan-arsitektur.md`).
- Validasi gagal harus menampilkan pesan per isian (KU-01). Akses ditolak tidak boleh membocorkan keberadaan data (KU-03). Antarmuka berbahasa Indonesia (NFR-LOC-01) `[DOK: 03-prd.md]`.

## Keputusan

**1. Envelope untuk semua respons API HTTP** (aplikasi, web, Partner Integration API, RME) `[USER] T4-G5`:

| Field | Ada di | Isi |
| --- | --- | --- |
| `response_code` | Semua respons | String 7 digit angka: 3 digit kategori HTTP, 2 digit kode modul, 2 digit kode kasus. Contoh `2000100` (sukses, modul 01, kasus 00) dan `4220101` (validasi gagal, modul 01, kasus 01) |
| `response_description` | Semua respons | Deskripsi singkat dalam Bahasa Indonesia, misalnya "Berhasil" atau "Data tidak valid" |
| `data` | Sukses | Objek atau daftar objek |
| `meta` | Sukses berupa daftar | Pagination berbasis cursor: `next_cursor`, `has_more` |
| `errors` | Galat validasi | Daftar `{ field, code, message }` per isian (KU-01) |
| `correlation_id` | Semua respons galat; boleh di sukses | ID pelacakan yang sama dengan application log dan audit trail (ADR-009, ADR-010) |

**2. Kode HTTP tetap dipakai di header** sesuai makna (200, 201, 4xx, 5xx), sehingga penyeimbang beban, perlindungan aplikasi web, pemantauan galat (NFR-OBS-01), batas laju, dan percobaan ulang integrasi (ADR-014) tetap bekerja. Field `status` tidak dimuat di isi respons `[USER] T4-G5`.

**3. Respons galat tetap kompatibel dengan RFC 9457**: memakai media type `application/problem+json`; member standar yang dipakai adalah `type` (URI dokumentasi jenis masalah) dan, bila relevan, `instance`; `response_code`, `response_description`, `errors`, dan `correlation_id` adalah member tambahan yang diizinkan `[SRC-47]`. Respons sukses memakai `application/json` `[KEPUTUSAN-73]`.

**4. Konvensi isi**: nama field dan kode berbahasa Inggris `snake_case` (KEPUTUSAN-39); waktu ISO 8601 dengan offset (ADR-017); uang bilangan bulat Rupiah berakhiran `_idr` (ADR-015); ID UUID (ADR-002). Pesan untuk pengguna dalam Bahasa Indonesia `[KEPUTUSAN-73]`.

**5. Keamanan isi galat**: data yang tidak boleh diakses dijawab 404 dengan `response_code` umum "tidak ditemukan" (KU-03, ADR-012). Galat tidak memuat stack trace, query, data pribadi, atau rahasia (ADR-010) `[KEPUTUSAN-73]`.

**6. Daftar kode** (kode modul, kode kasus, deskripsi, dan pemetaan ke kode HTTP) disusun di Tahap 5, dimuat di spesifikasi OpenAPI (ADR-022), dan diperiksa contract test. Satu kode kasus tidak boleh dipakai ulang untuk arti berbeda `[KEPUTUSAN-73]`.

## Alternatif yang ditolak

- **Selalu HTTP 200 dengan status hanya di `response_code`**: pemantauan, peringatan galat, batas laju, percobaan ulang rekanan, dan cache tidak dapat membedakan sukses dari gagal tanpa membaca isi. Opsi ini ditawarkan dan tidak dipilih user `[USER] T4-G5`.
- **Problem Details murni tanpa envelope**: user meminta kode respons numerik dan deskripsi di semua respons.
- **Kode respons bebas tanpa struktur**: sulit dipetakan ke kategori HTTP dan modul saat investigasi.

## Konsekuensi

- Aplikasi Flutter, web, dan rekanan membaca `response_code` untuk logika, dan menampilkan `response_description` atau `errors[].message` ke pengguna.
- Kode HTTP dan 3 digit pertama `response_code` harus konsisten. Hal ini diperiksa contract test.
- ADR-014 dan ADR-022 merujuk ADR ini untuk format galat.
