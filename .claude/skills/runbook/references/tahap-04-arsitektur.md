# Tahap 4: Arsitektur

Tujuan: membuat keputusan teknis yang paling mahal kalau salah. Memperbaiki arsitektur yang keliru biasanya berarti rewrite, jadi tahap ini layak memakai model terkuat dengan effort tertinggi.

Input: `docs/03-prd.md` (terutama NFR) dan dokumen sebelumnya. Baca juga ADR yang sudah ada di `docs/adr/`.

## Aktivitas

1. **Arsitektur sistem**: komponen, batas layanan, alur data, titik integrasi, dan lingkungan (dev, staging, production). Gambarkan dengan diagram Mermaid di `docs/04-arsitektur.md`.
2. **ADR** untuk setiap keputusan penting, memakai template di `templates.md`. Simpan di `docs/adr/ADR-xxx-<judul>.md`.
3. **Model data logikal**: atribut setiap entitas, primary/foreign key, normalisasi, kardinalitas, dan klasifikasi data (mana yang PII atau sensitif). Output: ERD (Mermaid) di `docs/04-erd.md` dan Data Dictionary di `docs/04-data-dictionary.md`.
4. **ADR audit trail**: mekanisme pencatatan, storage append-only yang terpisah dari DB aplikasi, perlindungan dari manipulasi, sinkronisasi waktu server, dan kebijakan arsip.
5. **Pemisahan tiga jenis log**: audit trail (akuntabilitas, fitur produk), application log (debugging), dan security log (deteksi ancaman). Ketiganya dikelola terpisah.
6. **Risk register** di `docs/04-risk-register.md`: risiko teknis, dampak, kemungkinan, mitigasi, dan pemiliknya.

## Keputusan data yang sudah ditetapkan

ADR-001 sampai ADR-005 di `docs/adr/` sudah diputuskan user (snake_case, UUIDv7, soft delete untuk data master, transaksi immutable, audit trail append-only). Ikuti semuanya. Kalau arsitektur menuntut perubahan pada salah satunya, ajukan ke user sebagai change request, jangan ubah diam-diam.

Keputusan data yang masih perlu ditetapkan di tahap ini (kalau relevan): multi-tenancy, strategi partisi tabel besar, dan enkripsi kolom.

## Threat modeling

Setelah arsitektur, ERD, dan data dictionary tersusun, panggil subagent `security-architect` untuk menyusun `docs/04-threat-model.md` (aset, trust boundary, ancaman STRIDE, mitigasi). Setiap mitigasi untuk ancaman Tinggi harus tercermin di arsitektur, ADR, NFR, atau backlog. Kalau threat model TIDAK LULUS, perbaiki desain lalu minta `security-architect` memeriksa ulang.

## Review DBA

Sebelum ERD dikunci, panggil subagent `dba-reviewer` untuk mencari redundansi, relasi janggal, index yang kurang, dan data sensitif yang belum terlindungi. Perbaiki berdasarkan temuannya.

## Gate Tahap 4

- [ ] Setiap NFR di PRD terjawab oleh arsitektur
- [ ] Setiap keputusan penting tercatat sebagai ADR
- [ ] ERD dan Data Dictionary lolos review DBA
- [ ] Audit trail punya ADR sendiri
- [ ] Threat model LULUS: setiap ancaman Tinggi punya mitigasi yang tercermin di desain
- [ ] Setiap risiko tinggi di risk register punya mitigasi
