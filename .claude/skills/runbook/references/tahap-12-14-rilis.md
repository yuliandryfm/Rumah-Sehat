# Tahap 12: Pre-deploy

Tujuan: memastikan rilis bisa dibatalkan dengan aman kalau terjadi masalah.

## Checklist go-live (tulis di `docs/12-go-live-checklist.md`)

- [ ] Environment target siap: konfigurasi, secrets, sertifikat
- [ ] Dry-run migrasi di staging dengan salinan data yang sudah dianonimkan
- [ ] Backup database diambil dan **restore-nya sudah diuji**
- [ ] Skrip rollback aplikasi dan migrasi siap pakai
- [ ] Monitoring, alerting, dan dashboard aktif
- [ ] Runbook operasional dan kontak eskalasi tersedia
- [ ] Jadwal rilis dan rencana komunikasi disepakati
- [ ] Kriteria rollback ditetapkan (misalnya error rate melewati ambang tertentu)

Tambahan untuk aplikasi di lingkungan teregulasi:

- [ ] Pentest independen oleh manusia sudah dilakukan dan temuan Critical/High sudah ditutup (audit AI di Tahap 10 tidak menggantikannya)
- [ ] Persetujuan change management diperoleh sesuai prosedur organisasi (peran Change Manager)
- [ ] Rencana serah terima ke tim support disepakati (peran Support Lead)

Catatan: kalau proyek masih uji coba di laptop, tahap ini boleh disederhanakan menjadi persiapan deploy ke staging/VPS. Tulis keputusan itu di checklist.

## Gate Tahap 12

- [ ] Checklist go-live lengkap dan disetujui user

---

# Tahap 13: Deploy

Tujuan: menjalankan rencana, bukan mengambil keputusan baru.

## Aturan

- **Setiap langkah yang tidak bisa dibatalkan wajib disetujui user secara eksplisit sebelum dijalankan.** Tunjukkan perintahnya dulu, tunggu jawaban.
- Rilis bertahap kalau memungkinkan.
- Jalankan smoke test segera setelah rilis dan tunjukkan hasilnya.
- Kalau kriteria rollback terpenuhi, hentikan dan tawarkan rollback.

Output: catatan rilis di `docs/13-release-notes.md`.

## Gate Tahap 13

- [ ] Smoke test lolos
- [ ] Metrik stabil selama periode observasi awal
- [ ] User mengonfirmasi rilis berhasil

---

# Tahap 14: Hypercare

Tujuan: pengawasan intensif selama 1–2 minggu setelah go-live.

## Aktivitas

- Pantau error, performa, query lambat, volume log, dan storage audit trail
- Triage bug produksi harian: yang kritis langsung diperbaiki, sisanya masuk backlog
- Setel index berdasarkan pola penggunaan nyata
- Sebagai Support Lead: siapkan serah terima ke tim support L1/L2 berisi runbook operasional, daftar masalah yang diketahui, dan jalur eskalasi
- Pantau alert aktivitas mencurigakan di audit trail
- Tutup proyek dengan **lessons learned**, lalu cek ulang KPI di Business Case

Output: `docs/14-hypercare-report.md` dan `docs/14-lessons-learned.md`.

## Gate penutupan

- [ ] Tidak ada insiden kritis selama periode hypercare
- [ ] Lessons learned tertulis
- [ ] Serah terima ke tim support selesai
- [ ] Backlog rilis berikutnya tersusun
- [ ] User menyetujui serah terima ke operasional rutin
