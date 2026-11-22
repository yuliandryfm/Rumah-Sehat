import type { PublicComponents } from "@rs/api-client";

/**
 * Tipe konten publik situs: langsung dari skema `openapi/public.json` (`/v1/public/content/*`, F1-T94),
 * dengan nama field snake_case apa adanya (KEPUTUSAN-119). Teks yang belum ada di CMS tidak dikarang:
 * halaman menampilkan penanda `[TIDAK DIKETAHUI]`.
 */
export type Konten = PublicComponents["schemas"]["PublicContentDto"];
export type HalamanSitus = PublicComponents["schemas"]["SitePage"];
export type JenisKonten = PublicComponents["schemas"]["ContentType"];

export interface OpsiDaftar {
  halaman?: HalamanSitus;
  jenis?: JenisKonten;
  /** UUID dari `meta.next_cursor` halaman sebelumnya. */
  kursor?: string;
  batas?: number;
}

export interface HasilDaftar {
  item: Konten[];
  kursorBerikut: string | null;
}

/**
 * Kontrak klien konten. `null` = tidak ada versi tayang (404 backend: draf, terjadwal, berakhir, diarsipkan,
 * dan terhapus tidak dibedakan, US-1801 AC3). Kegagalan lain melempar `KontenTidakTersedia`.
 */
export interface KlienKonten {
  ambilHalaman(halaman: HalamanSitus): Promise<Konten | null>;
  ambilItem(id: string): Promise<Konten | null>;
  daftar(opsi?: OpsiDaftar): Promise<HasilDaftar>;
}

/** Konten tidak dapat diambil (sumber belum tersambung atau backend gagal): halaman menampilkan galat umum. */
export class KontenTidakTersedia extends Error {
  constructor(alasan: string) {
    super(alasan);
    this.name = "KontenTidakTersedia";
  }
}
