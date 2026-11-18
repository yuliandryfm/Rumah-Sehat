# Registry `response_code`: Rumah Sehat dan RME minimalis

Tanggal: 2026-11-18
Versi: 1.0, disetujui di gate Tahap 5, 2026-11-18 `[USER] T5-G4`
Penyusun: Claude (persona Principal Engineer)
Dasar: ADR-023 `[DOK: adr/ADR-023-format-respons-api.md]` (format envelope dan `response_code`), ADR-012 (404 untuk objek terlarang), ADR-014 (idempotensi, batas waktu 5 detik), ADR-010 (isi galat tanpa data terlarang), PRD v1.1 KU-01, KU-03, KU-08, AB-36 `[DOK: 03-prd.md]`, KEPUTUSAN-84 (`docs/decision-log.md`)
Tingkat ketelitian: T2 Standar (`aturan-kerja.md` §1)

**Label sumber.** `[DOK: ...]` adalah dokumen yang lulus gate. `[KEPUTUSAN-xx]` tercatat di `docs/decision-log.md`. `[SRC-xx]` tercatat di `docs/sources.md`. `[USER] T4-G5` adalah keputusan user di gate Tahap 4.

Dokumen ini adalah sumber kebenaran daftar kode, sesuai ADR-023 butir 6: daftar kode disusun di Tahap 5, dimuat di spesifikasi OpenAPI (ADR-022), dan diperiksa contract test `[DOK: adr/ADR-023-format-respons-api.md]`.

---

## 1. Aturan format

| Aturan | Isi | Sumber |
| --- | --- | --- |
| Bentuk | String 7 digit angka `HHHMMCC`, bukan bilangan | `[DOK: adr/ADR-023-format-respons-api.md]` butir 1 |
| `HHH` | Kode status HTTP yang dikirim di header respons. Tiga digit pertama harus sama dengan kode HTTP; contract test memeriksanya | ADR-023 butir 1, butir 2, konsekuensi |
| `MM` | Kode modul 2 digit (§2) | `[KEPUTUSAN-84]` |
| `CC` | Kode kasus 2 digit di dalam modul. `00` dipakai untuk sukses; kasus galat mulai dari `01` | `[KEPUTUSAN-84]` |
| Identitas kode | Ketujuh digit bersama-sama. Satu kode tidak pernah dipakai untuk arti berbeda | ADR-023 butir 6 |
| Status HTTP di isi | Field `status` tidak dimuat di isi respons | ADR-023 butir 2 `[USER] T4-G5` |
| Deskripsi | `response_description` singkat dalam Bahasa Indonesia, sama untuk setiap kemunculan kode itu | ADR-023 butir 1, butir 4 |
| Prioritas pemakaian | Pakai kode umum modul `00` (§3) bila artinya sama persis. Kode modul hanya untuk kasus bisnis spesifik | `[KEPUTUSAN-84]` |
| Objek terlarang | Data yang tidak boleh diakses dijawab 404 dengan `4040001`, tidak membedakan "tidak ada" dari "tidak boleh" | ADR-023 butir 5, ADR-012, KU-03 |

Contoh format dari ADR-023: `2000100` (sukses, modul 01, kasus 00) dan `4220101` (validasi gagal, modul 01, kasus 01) `[DOK: adr/ADR-023-format-respons-api.md]`. Kedua contoh itu ilustrasi format; `4220101` belum terdaftar sebagai kode dan baru berlaku bila ditambahkan lewat aturan §5.

---

## 2. Kode modul

Kode modul mengikuti pembagian modul di arsitektur §3 `[DOK: 04-arsitektur.md]` `[KEPUTUSAN-84]`.

### 2.1 Umum dan Rumah Sehat

| Kode | Modul | Cakupan |
| --- | --- | --- |
| `00` | Umum / lintas modul | Kode kasus umum §3; dipakai Rumah Sehat dan RME |
| `01` | Identitas dan akses | Akun, registrasi, login, sesi, MFA, sub-peran, penonaktifan |
| `02` | Profil dan verifikasi | Profil pasien, alamat, anggota keluarga, dokumen identitas, verifikasi KTP/KK |
| `03` | Tenaga kesehatan | Pendaftaran, dokumen izin, verifikasi, SIP, tarif dan jadwal online |
| `04` | Rekanan | Onboarding, rekening, dokumen izin, status integrasi |
| `05` | Katalog salinan | Layanan, harga, jadwal, slot, katalog, stok, pencarian |
| `06` | Pemesanan | Pesanan, penahanan slot/stok, harga terkunci, status, pembatalan |
| `07` | Konsultasi | Sesi, chat, video, catatan, resep, verifikasi resep publik |
| `08` | Pembayaran | Adaptor PG, notifikasi PG, jurnal, refund, klaim, kewajiban, markup |
| `09` | Hasil lab | Penerimaan dan penyajian hasil lab |
| `10` | Notifikasi | Templat, email, kotak notifikasi di aplikasi |
| `11` | Konten dan layanan pelanggan | CMS, tiket keluhan |
| `12` | Portal rekanan | Laporan, profil, keluhan, pengaturan konsultasi |
| `13` | Gateway integrasi | Partner Integration API, webhook keluar, idempotensi, pemantauan |
| `14` | Audit | Halaman Audit Log, export |
| `15` | Privasi | Persetujuan, hak subjek data, salinan data, anonimisasi, retensi, kunci data |
| `16` | KPI | Dasbor KPI |
| `17` | Platform | Berkas, job, outbox, kalender hari kerja, parameter bisnis |
| `18`–`49` | Cadangan Rumah Sehat | Belum dipakai |

### 2.2 RME minimalis

Rentang `50`–`69` dicadangkan untuk RME minimalis `[KEPUTUSAN-84]`. Modul RME mengikuti arsitektur §3 `[DOK: 04-arsitektur.md]`.

| Kode | Modul RME |
| --- | --- |
| `50` | Identitas dan akses RME |
| `51` | Onboarding fasyankes (Operator RME) |
| `52` | Layanan, jadwal, dan harga |
| `53` | Appointment dan check-in |
| `54` | Antrean |
| `55` | Rekam medis kunjungan |
| `56` | Hasil lab RME |
| `57` | Home visit RME |
| `58` | SATUSEHAT |
| `59` | Klien integrasi ke Rumah Sehat |
| `60` | Audit RME |
| `61`–`69` | Cadangan RME |

`70`–`99` belum dialokasikan.

---

## 3. Kode kasus umum (modul `00`)

`[KEPUTUSAN-84]`

| `response_code` | HTTP | `response_description` | Kapan dipakai | Isi tambahan |
| --- | --- | --- | --- | --- |
| `2000000` | 200 | Berhasil | Permintaan baca atau ubah berhasil | `data`; `meta` bila berupa daftar |
| `2010000` | 201 | Berhasil dibuat | Sumber daya baru dibuat | `data` |
| `2020000` | 202 | Diterima untuk diproses | Permintaan diterima dan diproses asinkron lewat job (misalnya refund lewat job, ADR-015) | `data` berisi ID untuk pelacakan bila ada |
| `4000001` | 400 | Permintaan tidak dapat dibaca | Badan permintaan bukan JSON yang sah, header wajib tidak ada, atau parameter tidak dapat diurai. Bukan untuk aturan isian (pakai `4220001`) | — |
| `4010001` | 401 | Belum login atau sesi berakhir | Token tidak ada, tidak sah, kedaluwarsa, issuer atau audience salah (ADR-011 butir 8), sesi dicabut, atau akun dinonaktifkan | — |
| `4030001` | 403 | Tidak diizinkan | Peran pengguna tidak boleh menjalankan fungsi atau endpoint itu, dan keberadaan fungsinya tidak rahasia. Untuk objek data tertentu yang tidak boleh diakses, pakai `4040001` | — |
| `4040001` | 404 | Data tidak ditemukan | Data tidak ada, **atau** ada tetapi tidak boleh diakses pengguna (KU-03, ADR-012). Percobaan akses ke data sensitif dicatat di audit trail dengan hasil "gagal" | — |
| `4090001` | 409 | Konflik status | Transisi status kalah dari transisi yang tersimpan lebih dulu, atau tidak sah dari status saat ini (AB-36) | — |
| `4090002` | 409 | Kunci idempotensi sudah dipakai dengan isi berbeda | Kunci idempotensi yang sama dikirim ulang dengan badan permintaan berbeda (ADR-014) | — |
| `4130001` | 413 | Berkas terlalu besar | Ukuran berkas atau badan permintaan melebihi batas (ADR-018) | — |
| `4150001` | 415 | Jenis berkas tidak didukung | Jenis berkas dari isi berkas tidak ada di daftar izin, termasuk PDF berkonten aktif yang ditolak (ADR-018 butir 6) | — |
| `4220001` | 422 | Data tidak valid | Satu atau lebih isian gagal validasi (KU-01) | `errors` per isian |
| `4290001` | 429 | Terlalu banyak permintaan | Batas laju per pengguna, per klien rekanan, atau per IP terlampaui | — |
| `5000001` | 500 | Terjadi kesalahan pada sistem | Galat tak terduga di server. Isi tidak memuat stack trace, query, data pribadi, atau rahasia (ADR-023 butir 5) | — |
| `5020001` | 502 | Layanan pihak ketiga tidak merespons dengan benar | Rekanan atau penyedia mengembalikan respons tidak sah atau galat | — |
| `5030001` | 503 | Layanan sedang tidak tersedia | Layanan Rumah Sehat/RME atau dependensinya sedang tidak tersedia atau dalam pemeliharaan | — |
| `5040001` | 504 | Layanan pihak ketiga melewati batas waktu | Panggilan sinkron ke rekanan atau penyedia habis waktu, misalnya cek ketersediaan sebelum bayar dengan batas 5 detik (ADR-014 butir 5, KU-08). Pembayaran tidak dibuat | — |

Sumber kolom "Kapan dipakai": `[DOK: 03-prd.md]` (KU-01, KU-03, KU-08, AB-36), `[DOK: adr/ADR-012-otorisasi-dan-isolasi-data.md]`, `[DOK: adr/ADR-014-kontrak-integrasi-rekanan.md]`, `[DOK: adr/ADR-018-berkas-unggahan.md]`, `[DOK: adr/ADR-023-format-respons-api.md]`.

**Kasus yang belum punya kode** (ditambahkan lewat aturan §5 saat story pertama membutuhkannya): metode HTTP tidak didukung (405), rute tidak dikenal, tanda tangan webhook atau kredensial rekanan tidak valid (kandidat modul `13`), dan kasus bisnis spesifik per modul.

---

## 4. Contoh respons

Nama field dan kode berbahasa Inggris `snake_case`; waktu ISO 8601 dengan offset; uang bilangan bulat Rupiah berakhiran `_idr`; ID berupa UUID (ADR-023 butir 4). Semua nilai di contoh adalah data sintetis (ADR-022 butir 4).

### 4.1 Media type

| Jenis respons | Media type | Sumber |
| --- | --- | --- |
| Sukses | `application/json` | ADR-023 butir 3 `[KEPUTUSAN-73]` |
| Galat | `application/problem+json` | ADR-023 butir 3 `[SRC-47]` |

### 4.2 Sukses: satu objek

`HTTP 200`, `Content-Type: application/json`

```json
{
  "response_code": "2000000",
  "response_description": "Berhasil",
  "data": {
    "order_id": "01936f8e-7a2b-7c3d-9e4f-5a6b7c8d9e0f",
    "status": "paid",
    "total_idr": 150000,
    "created_at": "2027-03-05T09:15:00+07:00"
  },
  "correlation_id": "01936f8e-7a2c-7d11-8a22-3b44c55d66e7"
}
```

### 4.3 Sukses: daftar dengan pagination cursor

`HTTP 200`, `Content-Type: application/json`

```json
{
  "response_code": "2000000",
  "response_description": "Berhasil",
  "data": [
    { "clinic_id": "01936f8e-8b10-7a20-9b30-4c405d506e70", "name": "Klinik Contoh Satu" },
    { "clinic_id": "01936f8e-8b11-7a21-9b31-4c415d516e71", "name": "Klinik Contoh Dua" }
  ],
  "meta": {
    "next_cursor": "eyJpZCI6IjAxOTM2ZjhlIn0",
    "has_more": true
  }
}
```

`correlation_id` boleh tidak dimuat di respons sukses (ADR-023 butir 1).

### 4.4 Galat validasi

`HTTP 422`, `Content-Type: application/problem+json`

```json
{
  "type": "https://<domain-dokumentasi>/problems/4220001",
  "response_code": "4220001",
  "response_description": "Data tidak valid",
  "errors": [
    { "field": "email", "code": "invalid_format", "message": "Format email tidak valid" },
    { "field": "address.postal_code", "code": "required", "message": "Kode pos wajib diisi" }
  ],
  "correlation_id": "01936f8e-9c01-7e02-8f03-1a2b3c4d5e6f"
}
```

### 4.5 Galat akses ke objek terlarang (KU-03)

`HTTP 404`, `Content-Type: application/problem+json`

```json
{
  "type": "https://<domain-dokumentasi>/problems/4040001",
  "response_code": "4040001",
  "response_description": "Data tidak ditemukan",
  "correlation_id": "01936f8e-9d01-7f02-8a03-2b3c4d5e6f70"
}
```

Isi respons sama untuk data yang tidak ada dan data yang tidak boleh diakses (ADR-023 butir 5).

### 4.6 Galat pihak ketiga habis waktu (KU-08)

`HTTP 504`, `Content-Type: application/problem+json`

```json
{
  "type": "https://<domain-dokumentasi>/problems/5040001",
  "response_code": "5040001",
  "response_description": "Layanan pihak ketiga melewati batas waktu",
  "correlation_id": "01936f8e-9e01-7a02-8b03-3c4d5e6f7081"
}
```

### 4.7 Ketentuan field galat

| Field | Ketentuan | Sumber |
| --- | --- | --- |
| `type` | Member standar RFC 9457. URI dokumentasi jenis masalah: `https://<domain-dokumentasi>/problems/<response_code>`. Domain dokumentasi `[TIDAK DIKETAHUI]`, ditetapkan sebelum rilis | `[SRC-47]` `[KEPUTUSAN-84]` |
| `instance` | Member standar RFC 9457, dimuat bila relevan. Tidak boleh memuat ID data, query string, atau data pribadi (ADR-010: rute ditulis sebagai pola). Bentuk persisnya ditetapkan di `docs/coding-standards.md` | ADR-023 butir 3 `[SRC-47]` |
| `response_code`, `response_description`, `errors`, `correlation_id` | Member tambahan yang diizinkan RFC 9457 | ADR-023 butir 3 `[SRC-47]` |
| `correlation_id` | Wajib di semua respons galat; nilainya sama dengan application log dan audit trail | ADR-023 butir 1 |
| `errors[]` | `{ field, code, message }` per isian. `code` berbahasa Inggris `snake_case`; `message` Bahasa Indonesia untuk pengguna. Nilai `code` dan format path `field` di contoh adalah ilustrasi; daftar baku ditetapkan di `docs/coding-standards.md` | ADR-023 butir 1, butir 4; KU-01 |
| `title`, `status`, `detail` | Tidak dipakai. `status` sengaja dilepas dari isi respons | ADR-023 butir 2 `[USER] T4-G5` |
| Larangan isi | Tanpa stack trace, query, data pribadi, atau rahasia | ADR-023 butir 5, ADR-010 |

---

## 5. Aturan penambahan kode (Definition of Done)

`[KEPUTUSAN-84]` `[DOK: adr/ADR-023-format-respons-api.md]`

1. Kode spesifik modul ditambahkan per story di Tahap 8, sebagai bagian dari Definition of Done story yang menambah atau mengubah endpoint.
2. Sebelum menambah kode, periksa §3. Bila arti yang dibutuhkan sudah ada di kode umum, pakai kode umum itu.
3. Kode baru ditulis di tabel modulnya (§6) dalam commit yang sama dengan endpoint yang memakainya. Isinya: kode, HTTP, deskripsi Bahasa Indonesia, kapan dipakai, dan story asal (US-xxx).
4. `HHH` harus sama dengan kode HTTP yang dikirim. `MM` harus modul pemilik endpoint. `CC` memakai nomor berikutnya yang belum pernah dipakai di kombinasi `HHHMM` itu.
5. Kode tidak pernah dihapus atau dipakai ulang untuk arti lain. Kode yang tidak lagi dipakai ditandai "usang" beserta tanggal dan alasannya.
6. Kode dimuat di spesifikasi OpenAPI endpoint yang memakainya (ADR-022). `scripts/verify.sh` memeriksa setiap `response_code` di spesifikasi terdaftar di registry ini dan tiga digit pertamanya sama dengan kode HTTP `[KEPUTUSAN-82]`. Salinan registry yang dapat dibaca mesin untuk pemeriksaan itu dibuat di Tahap 8 Fase 1; lokasinya ditetapkan di `docs/coding-standards.md`.
7. Perubahan kode atau deskripsi yang sudah dipakai rekanan (Partner Integration API) adalah perubahan kontrak: perlu review, dan perubahan yang merusak kompatibilitas diterbitkan sebagai versi baru dengan masa transisi (ADR-014 butir 1 dan konsekuensi).

---

## 6. Kode spesifik modul

Belum ada. Diisi per story di Tahap 8 mengikuti §5.

Format tabel per modul:

| `response_code` | HTTP | `response_description` | Kapan dipakai | Story | Status |
| --- | --- | --- | --- | --- | --- |
| — | — | — | — | — | — |
