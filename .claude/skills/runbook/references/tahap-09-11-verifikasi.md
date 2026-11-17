# Tahap 9–11: Verifikasi independen

Developer tidak boleh menjadi satu-satunya pihak yang menyatakan pekerjaannya lulus (*segregation of duties*). Ketiga tahap ini dijalankan lewat subagent atau sesi baru.

---

# Tahap 9: QA dan SIT

Jalankan lewat subagent `qa-engineer`.

## Cakupan SIT

- End-to-end untuk semua alur utama di PRD
- Regression terhadap fitur dari semua fase
- Integrasi dengan sistem luar (pakai mock/sandbox kalau sistem asli belum tersedia, dan catat sebagai keterbatasan)
- Performance/load test sesuai angka NFR
- Setiap test case untuk aksi sensitif ikut memverifikasi audit trail tercatat benar
- Soft delete: delete, restore, unique constraint setelah delete, data terhapus tidak bocor ke list atau export
- Migrasi naik dan turun (rollback) dengan test data realistis

Output: `docs/09-sit-report.md` berisi ringkasan, daftar bug dengan severity, dan status.

## Gate Tahap 9

- [ ] Semua test case di test plan sudah dijalankan
- [ ] Tidak ada bug Critical/High terbuka
- [ ] Bug Medium/Low tercatat di backlog dan disetujui user untuk dibawa

---

# Tahap 10: Audit dan Security Test

Jalankan lewat subagent `security-auditor`. Boleh paralel dengan SIT.

## Cakupan

- Code audit: autentikasi, otorisasi, validasi input, penanganan data sensitif, pengelolaan secrets
- Dependency: celah keamanan pada library yang dipakai
- Audit trail: bisakah log diubah atau dihapus? Adakah password, PIN, OTP, token, atau nomor rekening lengkap yang bocor ke log?
- Database: least privilege, hak `DELETE` dicabut dari user aplikasi, enkripsi kolom sensitif, potensi SQL injection
- Kalau aplikasi bisa dijalankan lokal: uji dasar terhadap aplikasi yang berjalan (header keamanan, rate limiting, error yang membocorkan detail internal)

Output: `docs/10-audit-report.md` dengan temuan, severity, rekomendasi, dan status remediasi.

## Audit kepatuhan

Panggil subagent `compliance-reviewer` (mode Tahap 10) untuk memastikan setiap BR kepatuhan sudah diimplementasikan dan diuji. Laporan di `docs/compliance/tahap-10.md`.

Catatan: audit oleh AI membantu menangkap masalah lebih awal, tapi **tidak menggantikan pentest independen oleh manusia** untuk aplikasi di lingkungan teregulasi. Pentest manusia dicek di checklist Tahap 12.

## Gate Tahap 10

- [ ] Semua temuan Critical/High sudah diremediasi
- [ ] Remediasi diverifikasi ulang oleh `security-auditor`
- [ ] Audit kepatuhan `compliance-reviewer` LULUS, dan hal yang perlu konfirmasi legal sudah dicatat

---

# Tahap 11: UAT

Tujuan: membuktikan aplikasi memenuhi kebutuhan bisnis. **Pemutus adalah Business Owner, yaitu user.** Claude berperan sebagai QA yang memfasilitasi: menyiapkan skenario, data uji, dan mencatat hasil. Claude tidak pernah menyatakan UAT lulus atas nama user.

## Langkah

1. Turunkan skenario UAT dari proses *to-be* di BRD dan KPI di Business Case. Simpan di `docs/11-uat-scenarios.md`.
2. Siapkan data uji dan cara menjalankan aplikasi secara lokal.
3. User menjalankan skenario. Bantu mencatat hasilnya.
4. Kategorikan temuan: bug (diperbaiki sebelum rilis) atau CR (masuk backlog).

## Gate Tahap 11

- [ ] Semua skenario UAT dijalankan user
- [ ] Bug dari UAT sudah diperbaiki
- [ ] User memberikan sign-off tertulis (dicatat di `docs/11-uat-scenarios.md`)
