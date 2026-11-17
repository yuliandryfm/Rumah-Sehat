# Tahap 1: Business Case

Tujuan: menjawab apakah proyek ini layak dikerjakan. Sesuai prinsip PRINCE2 *continued business justification*, dokumen ini dicek ulang di setiap gate besar.

Peran: Claude menyusun sebagai Project Manager + Business Analyst. **Pemilik dan pemutus business case adalah Executive, yaitu user.** Claude tidak memutuskan kelayakan; Claude menyajikan opsi dan dasar perbandingannya.

Input: `docs/00-project-brief.md`.

## Isi minimum

- Latar belakang dan masalah, diambil dari Project Brief
- Opsi yang dipertimbangkan: tidak melakukan apa-apa, membeli solusi jadi, atau membangun sendiri, beserta alasan memilih
- Manfaat yang diharapkan dan cara mengukurnya (KPI dengan angka dan tenggat)
- Perkiraan biaya dan waktu (kasar dulu, dipertajam setelah Tahap 4)
- Risiko utama dan toleransinya

Output: `docs/01-business-case.md`.

## Gate Tahap 1

- [ ] Setiap KPI terukur (angka + tenggat)
- [ ] Opsi alternatif sudah dibandingkan
- [ ] User menyatakan proyek layak dilanjutkan

---

# Tahap 2: BRD (Business Requirements Document)

Tujuan: mendefinisikan *apa* yang dibutuhkan bisnis, belum *bagaimana* membangunnya.

Input: `docs/00-project-brief.md`, `docs/01-business-case.md`.

## Isi minimum

- Tujuan bisnis dan KPI dari Business Case
- Stakeholder dan peran pengguna
- Proses bisnis saat ini (*as-is*) dan yang diinginkan (*to-be*)
- Kebutuhan bisnis bernomor (BR-001, BR-002, dan seterusnya) agar bisa ditelusuri sampai ke test case
- **Scope dan non-scope yang eksplisit**
- Kebutuhan kepatuhan, termasuk audit trail sebagai kebutuhan wajib kalau menyasar perbankan atau keuangan
- Asumsi, batasan, dan dependensi

Output: `docs/02-brd.md`.

## Review kepatuhan

Panggil subagent `compliance-reviewer` untuk mereview BRD. Kebutuhan kepatuhan yang hilang ditambahkan sebagai BR, klaim regulasi tanpa sumber resmi diperbaiki atau diubah menjadi `[TIDAK DIKETAHUI]`, dan hal yang perlu konfirmasi legal masuk pertanyaan terbuka.

## Gate Tahap 2

- [ ] Review `compliance-reviewer` LULUS (laporan di `docs/compliance/tahap-02.md`)
- [ ] Setiap kebutuhan bernomor (BR-xxx)
- [ ] Scope dan non-scope ditulis eksplisit dan disetujui user
- [ ] Tidak ada pertanyaan terbuka yang memblokir PRD
