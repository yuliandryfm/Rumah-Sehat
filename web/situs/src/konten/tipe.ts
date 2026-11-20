/**
 * Tipe konten publik yang dipakai situs. Bentuk final mengikuti endpoint `/v1/public/content/*`
 * (ADR-027 butir 2, F1-T94) dan akan diganti tipe hasil generate `@rs/api-client` setelah endpoint itu
 * masuk `openapi/`. Sampai itu, tipe ini adalah antarmuka sementara yang dapat di-mock.
 * Teks yang belum ditetapkan user bernilai `null` dan ditampilkan sebagai `[TIDAK DIKETAHUI]`.
 */

/** Nilai teks dari CMS; `null` = belum ada (tampil sebagai penanda `[TIDAK DIKETAHUI]`). */
export type TeksCms = string | null;

export interface KartuLayanan {
  judul: string;
  uraian: TeksCms;
}

export interface RingkasanArtikel {
  slug: string;
  judul: string;
  /** ISO 8601 tanggal tayang; `null` bila belum diketahui. */
  tanggalTayang: string | null;
}

export interface KontenBeranda {
  judulUtama: TeksCms;
  kalimatPembuka: TeksCms;
  layanan: KartuLayanan[];
  ringkasanKorporat: TeksCms;
  artikelTerbaru: RingkasanArtikel[];
}

/** Kontrak klien konten. Halaman lain (F1-T96, F1-T97) menambah metode di sini. */
export interface KlienKonten {
  ambilBeranda(): Promise<KontenBeranda>;
}

/** Konten tidak dapat diambil (sumber belum tersambung atau backend gagal): halaman menampilkan galat umum. */
export class KontenTidakTersedia extends Error {
  constructor(alasan: string) {
    super(alasan);
    this.name = "KontenTidakTersedia";
  }
}
