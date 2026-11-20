# Mode Otomatis

Di mode ini kamu (sesi utama) berperan sebagai **orkestrator**. Pekerjaan setiap tahap didelegasikan ke subagent dengan konteks bersih dan model yang sesuai. Gate diperiksa oleh mesin dan reviewer, bukan ditanyakan ke user. User hanya dilibatkan di **4 checkpoint**.

Aturan `anti-halu.md` tetap berlaku penuh. Justru di mode ini aturan itu menjadi penjaga utama, karena user tidak melihat setiap langkah.

## 4 checkpoint manusia

| Checkpoint | Setelah | User melakukan |
| --- | --- | --- |
| 1. Brief | Tahap 0 | Konfirmasi Project Brief. Tahap 0 selalu interaktif |
| 2. Paket perencanaan | Tahap 1–7 | Review sekali jalan: BRD, PRD, arsitektur dan ADR baru, stack, desain, rencana fase, plus semua asumsi dan keputusan |
| 3. UAT | Tahap 8–10 dan persiapan Tahap 11 | Menjalankan skenario UAT dan memberi sign-off |
| 4. Go-live | Tahap 12 | Menyetujui eksekusi deploy (Tahap 13) |

Tahap 14 (Hypercare) dijalankan setelah rilis dan dilaporkan berkala, tanpa checkpoint tambahan.

## Otomatis dan manual per tahap

Setiap tahap dijalankan dengan persona spesialisnya. Tahap yang menyangkut keputusan bisnis, penerimaan hasil, atau langkah yang tidak bisa dibatalkan selalu menunggu respons user.

| # | Tahap | Eksekusi | Persona pelaksana | Respons user |
| --- | --- | --- | --- | --- |
| 0 | Intake | **Manual** | Business Analyst + PM (sesi utama, interaktif) | Menjawab wawancara, mengonfirmasi brief (Checkpoint 1) |
| 1 | Business Case | Otomatis | PM + Business Analyst (`perencana`) | Memutuskan kelayakan sebagai Executive (Checkpoint 2) |
| 2 | BRD | Otomatis | Business Analyst (`perencana`) | Checkpoint 2; konfirmasi legal kalau ditandai |
| 3 | PRD + NFR | Otomatis | Product Manager + Tech Lead (`perencana`) | Checkpoint 2, termasuk NFR yang ditandai berisiko |
| 4 | Arsitektur | Otomatis | Solution Architect (`arsitek`) | Menerima ADR baru (Checkpoint 2) |
| 5 | Stack + Standar | Otomatis | Principal Engineer + DevOps (`perencana`) | Checkpoint 2 |
| 6 | Design Guideline | Otomatis | UI/UX Designer (`perencana`) | Menyetujui arah desain (Checkpoint 2) |
| 7 | Phase Plan | Otomatis | PM + Tech Lead (`perencana`) | Menyetujui pembagian fase (Checkpoint 2) |
| 8 | Dev + Test | Otomatis | Senior Software Engineer (`developer`); Tech Lead untuk reference implementation dan consistency review | Menyetujui pembuatan atau perubahan `scripts/verify.sh` saat diminta |
| 9 | QA + SIT | Otomatis | QA Engineer (`qa-engineer`) | Tidak ada, kecuali Claude berhenti |
| 10 | Audit + Security | Otomatis | Security Auditor (`security-auditor`) + Compliance Officer (`compliance-reviewer`) | Konfirmasi legal kalau ditandai |
| 11 | UAT | **Manual** | QA sebagai fasilitator (orkestrator) | Menjalankan UAT dan sign-off (Checkpoint 3) |
| 12 | Pre-deploy | **Semi** | DevOps/SRE + Change Manager (orkestrator) | Pentest manusia, change management, persetujuan go-live (Checkpoint 4) |
| 13 | Deploy | **Manual** | Release Manager (orkestrator), hanya setelah disetujui | Menyetujui setiap langkah yang tidak bisa dibatalkan |
| 14 | Hypercare | Otomatis + laporan | SRE + Support Lead + PM (orkestrator) | Membaca laporan, menyetujui penutupan |

Untuk tahap yang dijalankan orkestrator sendiri (11–14), orkestrator mengambil persona di kolom itu.

## Siapa mengerjakan apa

| Tahap | Pelaksana | Reviewer sebelum gate |
| --- | --- | --- |
| 1 | subagent `perencana` | `fact-checker` |
| 2 | subagent `perencana` | `compliance-reviewer`, `fact-checker` |
| 3 | subagent `perencana` (termasuk tinjauan Tech Lead atas NFR) | `qa-engineer` (acceptance criteria), `fact-checker` |
| 4 | subagent `arsitek` | `security-architect` (threat model), `dba-reviewer`, `fact-checker` |
| 5, 6 | subagent `perencana` (termasuk tanggung jawab DevOps di Tahap 5) | `dba-reviewer` (skema fisik, Tahap 5), `fact-checker` |
| 7 | subagent `perencana` (termasuk tinjauan Tech Lead) | `qa-engineer` (test plan), `fact-checker` |
| 8 | subagent `developer`, satu pemanggilan per task | `code-reviewer` per task; `qa-engineer` dan consistency review (Tech Lead, Opus) per fase |
| 9 | subagent `qa-engineer` | `fact-checker` (laporan SIT) |
| 10 | subagent `security-auditor` | `compliance-reviewer`; verifikasi ulang remediasi oleh `security-auditor` |
| 11 (persiapan) | orkestrator | `fact-checker` (skenario UAT sesuai BRD) |
| 12 | orkestrator | checklist go-live (termasuk pentest manusia dan change management bila teregulasi) |

## Alur per tahap otomatis

1. Buat branch `tahap-XX-<nama>` dari branch utama.
2. Ubah status tahap di `docs/STATUS.md` menjadi `berjalan`.
3. Panggil subagent pelaksana. Instruksinya menyebut: nomor tahap, **persona tahap itu** (dari tabel "Otomatis dan manual per tahap"), path file referensi tahap, **tingkat ketelitian (T1/T2/T3)**, dan pengingat untuk mematuhi `anti-halu.md` serta `aturan-kerja.md`. Parameter `model` dan `effort` diisi menurut tabel "Daftar tahap" dan kelas task (`aturan-kerja.md` §4). Jangan menyalin isi dokumen ke instruksi; subagent membacanya sendiri.
4. Baca laporan akhir subagent. Kalau statusnya BERHENTI, lanjut ke "Kondisi berhenti".
5. Panggil reviewer sesuai tabel. Teruskan temuan yang terkonfirmasi ke subagent pelaksana untuk diperbaiki.
6. Panggil `fact-checker` untuk output tahap ini. Hasilnya harus LULUS.
7. Periksa checklist gate tahap (dari file referensinya) ditambah gate anti-halu. Setiap butir dicek terhadap file yang ada, bukan berdasarkan laporan subagent saja.
8. Kalau gate gagal: perbaiki lewat subagent pelaksana, maksimal 2 putaran perbaikan. Masih gagal, lanjut ke "Kondisi berhenti".
9. Kalau gate lulus: commit, merge ke branch utama (keduanya hanya bisa berhasil kalau hijau, karena dijaga git hook), **push `main`**, ubah status menjadi `selesai (menunggu checkpoint N)` beserta tanggal, lalu lanjut ke tahap berikutnya tanpa bertanya. Reviewer dan fact-checker dipanggil dengan kriteria blokir `aturan-kerja.md` §2; catatan non-blokir tidak memicu putaran perbaikan.

## Alur Tahap 8 (per fase)

1. Untuk setiap task di fase itu, berurutan: panggil `developer` untuk satu task → panggil `code-reviewer` → kalau ada temuan terkonfirmasi, panggil `developer` lagi untuk memperbaikinya.
2. Setiap pemanggilan subagent memulai konteks baru, jadi tidak perlu `/clear`.
3. Di akhir fase: `qa-engineer` menjalankan functional test, lalu consistency review (subagent `code-reviewer` dengan instruksi memeriksa drift pola di seluruh fase). Bug dan tech debt masuk backlog.
4. Gate fase lulus → lanjut ke fase berikutnya tanpa bertanya.

## Kerja paralel antar-subagent

- Subagent pelaksana yang berjalan paralel dalam satu tahap diberi **kepemilikan berkas yang terpisah**, ditulis eksplisit di instruksinya. Berkas bersama (register `docs/decision-log.md`, `docs/sources.md`) dibagi **rentang nomor** per subagent; baris catatan penomoran di register diperbarui oleh orkestrator sesudah semua selesai, termasuk nomor yang sengaja tidak terpakai.
- Subagent reviewer yang tidak punya alat tulis (misalnya `dba-reviewer`) mengembalikan laporan; orkestrator menyimpan **ringkasannya** di `docs/`, bukan menyalin laporan panjang apa adanya.
- Temuan reviewer diverifikasi orkestrator terhadap berkas sebelum diteruskan untuk diperbaiki (`anti-halu.md` §6).

## Kondisi berhenti (keluar dari mode otomatis dan melapor ke user)

Berhenti segera, tulis laporan, dan tunggu user, kalau:

- Ada **fakta bisnis** yang `[TIDAK DIKETAHUI]` dan memblokir.
- Pekerjaan menuntut perubahan scope, perubahan ADR yang sudah diterima, atau tambahan biaya.
- Langkah berikutnya **tidak bisa dibatalkan** (deploy, migrasi di production, menghapus data).
- `fact-checker` tetap TIDAK LULUS setelah 2 putaran perbaikan.
- Masalah yang sama gagal diperbaiki 3 kali.
- **Batas sesi tercapai**: 45 task development atau fase yang sedang berjalan selesai (mana yang lebih dulu), atau 6 tahap perencanaan, dalam satu pemanggilan `/runbook auto`. Batas ini menjaga pemakaian kuota tetap terkendali. User melanjutkan dengan `/runbook auto` lagi. (Diubah dari 20 task atas arahan user 2026-11-20, Telegram: "berhenti per 45 task atau fasenya sudah selesai".)

## Laporan checkpoint

Di setiap checkpoint (dan setiap kali berhenti), tulis `docs/checkpoint-N.md` lalu tampilkan ringkasannya:

```markdown
# Checkpoint N: <nama>

Tanggal: YYYY-MM-DD
Tahap yang selesai: ...

## Dokumen untuk direview
- <file>: <satu baris isi>

## Asumsi yang perlu dikonfirmasi
| ID | Asumsi | Dampak kalau salah |

## Keputusan yang diambil (untuk diketahui)
| ID | Keputusan | Alasan | Bisa dibatalkan? |

## Pertanyaan terbuka
| # | Pertanyaan | Memblokir tahap |

## Hasil fact-check
<ringkasan per tahap, semua harus LULUS>

## Yang dibutuhkan dari user
1. ...
```

Cara bertanya di checkpoint: butir yang sudah punya usulan default ditawarkan **disetujui sekaligus** dalam satu pertanyaan (user boleh menyebut butir yang ingin diubah); butir tanpa default dan butir yang mengubah keputusan lama ditanyakan terpisah, mengikuti aturan "Cara bertanya ke user" di `SKILL.md`.

Setelah user menjawab: catat jawaban di file checkpoint (sebagai sumber `[USER]`), perbarui status asumsi di `docs/decision-log.md`, ubah ADR "Diusulkan" yang disetujui menjadi "Diterima", ubah status tahap terkait menjadi `selesai`, lalu lanjutkan mode otomatis. Kalau jawaban user mengoreksi sesuatu, perbaiki dokumen yang terdampak dan jalankan ulang `fact-checker` sebelum lanjut.
