# ADR-022: Semua API didokumentasikan dengan OpenAPI, dengan UI interaktif di lokal/staging dan contract test

Status: Diterima (gate Tahap 4, 2026-11-18 `[USER] T4-G5`)
Tanggal: 2026-11-18

## Konteks

- User meminta dokumentasi API seperti Swagger atau yang serupa agar mudah untuk testing `[USER] T4-masukan` (`docs/04-catatan-arsitektur.md`).
- ADR-014 sudah menetapkan spesifikasi OpenAPI untuk Partner Integration API (US-1401) `[KEPUTUSAN-30]`.
- Backend Ruang Sehat melayani aplikasi pasien, aplikasi tenaga kesehatan, backoffice, portal rekanan, Partner Integration API, dan notifikasi payment gateway. Backend RME melayani web RME dan menerima webhook dari Ruang Sehat (ADR-006).
- Pengujian dilakukan oleh tim, QA (Tahap 9), dan rekanan (US-1401 AC2).

## Keputusan

1. **Spesifikasi OpenAPI untuk setiap API HTTP**: API aplikasi pasien, API aplikasi tenaga kesehatan, API backoffice, API portal rekanan, Partner Integration API, endpoint webhook (payment gateway dan webhook masuk RME), dan API RME. Spesifikasi disimpan di repositori, berversi bersama kode, dan menjadi kontrak resmi setiap endpoint, termasuk skema permintaan dan respons, format respons dan kode galat sesuai ADR-023 (envelope `response_code` yang kompatibel dengan RFC 9457 `[SRC-47]`), dan cara autentikasi `[KEPUTUSAN-72]`.
2. **UI dokumentasi interaktif** (misalnya Swagger UI atau setara) untuk membaca dan mencoba API tersedia di lingkungan **lokal dan staging**. Di production, dokumentasi API internal tidak dibuka publik, supaya tidak mempermudah pemetaan serangan. Rekanan menerima dokumentasi Partner Integration API sebagai dokumen statis dan mencobanya di sandbox staging (ADR-014 butir 8) `[KEPUTUSAN-72]`.
3. **Contract test otomatis** memastikan implementasi sesuai spesifikasi: setiap endpoint di kode ada di spesifikasi dan sebaliknya, dan respons sesuai skema. Test ini menjadi bagian `scripts/verify.sh` (Tahap 5). Spesifikasi yang tidak sesuai implementasi membuat verifikasi merah `[KEPUTUSAN-72]`.
4. **Data contoh** di spesifikasi dan di UI staging hanya data sintetis, tanpa data pribadi nyata (ADR-019) `[KEPUTUSAN-72]`.
5. **Kanal WebSocket** (ADR-016) didokumentasikan dengan daftar pesan, skema, dan aturan otorisasi per kanal. Format spesifikasinya ditetapkan di Tahap 5 `[KEPUTUSAN-72]`.
6. Versi OpenAPI, perkakas UI, dan pustaka pembangkit/validator spesifikasi dipilih di Tahap 5 dari sumber resmi `[KEPUTUSAN-72]`.

## Alternatif yang ditolak

- **Dokumentasi manual (wiki atau dokumen terpisah)**: cepat basi dan tidak dapat diuji mesin.
- **UI interaktif di production**: mempermudah pemetaan endpoint oleh penyerang. Pengujian cukup di staging.
- **Hanya Partner Integration API yang didokumentasikan**: aplikasi mobile dan web dibangun oleh orang yang berbeda (mobile developer dan Claude `[DOK: 00-project-brief.md]`), sehingga kontrak tertulis diperlukan untuk semua API.

## Konsekuensi

- Setiap story yang menambah atau mengubah endpoint wajib memperbarui spesifikasi OpenAPI (Definition of Done).
- Spesifikasi dapat dipakai untuk membangkitkan klien API di aplikasi Flutter dan web, yang menjadi pertimbangan di Tahap 5.
- Contract test menambah waktu verifikasi sedikit, tetapi mencegah dokumentasi dan implementasi menyimpang.
