# Template dokumen

## Project Brief (`docs/00-project-brief.md`)

```markdown
# Project Brief: <nama kerja aplikasi>

Tanggal: YYYY-MM-DD
Jalur intake: A (dokumen) | B (wawancara) | A+B

Setiap butir wajib diberi label sumber: [USER], [DOK: nama-file], atau [TIDAK DIKETAHUI].
Contoh: "Pengguna utama adalah petugas cabang [USER]."

## Ringkasan
Satu paragraf: apa aplikasinya, untuk siapa, masalah apa yang diselesaikan.

## Masalah dan tujuan
- Masalah:
- Dampak kalau tidak diselesaikan:
- Tujuan terukur:

## Pengguna
| Peran | Kebutuhan utama |
| --- | --- |

## Scope awal
- In-scope:
- Out-of-scope:

## Data dan integrasi
- Entitas utama:
- Data sensitif:
- Sistem yang terhubung:

## Kepatuhan
- Regulasi/standar:
- Kebutuhan audit trail:
- Pihak auditor:

## Batasan
- Anggaran:
- Tenggat:
- Tim:
- Teknologi wajib/dilarang:

## KPI dan go-live
- Ukuran sukses:
- Syarat rilis pertama:

## Asumsi
- 

## Pertanyaan terbuka
| # | Pertanyaan | Pemilik | Tenggat |
| --- | --- | --- | --- |
```

## ADR (`docs/adr/ADR-xxx-<judul>.md`)

```markdown
# ADR-XXX: <judul keputusan>

Status: Diusulkan | Diterima | Digantikan oleh ADR-YYY
Tanggal: YYYY-MM-DD

## Konteks
Masalah atau kebutuhan yang memicu keputusan ini.

## Keputusan
Apa yang diputuskan, dalam satu atau dua kalimat.

## Alternatif yang ditolak
- Opsi A: alasan ditolak
- Opsi B: alasan ditolak

## Konsekuensi
Dampak positif, dampak negatif, dan hal yang harus dijaga.
```

## User Story (di `docs/backlog.md` atau GitHub Issues)

```markdown
### [EPIC-XX] STORY-XXX: <judul story>

Sebagai <peran>, saya ingin <aksi>, agar <manfaat>.

Rujukan: BR-XXX, PRD bagian X.X, ADR-XXX
Prioritas: Must | Should | Could
Fase: 1
Status: Backlog | Ready | In progress | Review | Done

Acceptance criteria:
- Given <kondisi>, When <aksi>, Then <hasil>
- Given <kondisi>, When <aksi>, Then <hasil>

Audit trail: <nama event> (atau: tidak ada aksi sensitif)

Task:
- [ ] <task 2–4 jam>
- [ ] <task 2–4 jam>
```

## Change Request

```markdown
### CR-XXX: <judul>

Diajukan: YYYY-MM-DD
Deskripsi:
Alasan:
Dampak ke scope / waktu / arsitektur:
Keputusan: Diterima (masuk Fase N) | Ditunda | Ditolak
```
