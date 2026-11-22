export interface PaketLisensi {
  name: string;
  versions?: string[];
  license: string | undefined;
}
export interface Pelanggaran {
  nama: string;
  versi: string;
  lisensi: string;
  alasan: string;
}
export const LISENSI_RUNTIME: string[];
export const LISENSI_TAMBAHAN_DEV: string[];
export type PohonLisensi = { id: string; op?: undefined } | { op: "AND" | "OR"; anak: PohonLisensi[]; id?: undefined };
export function uraiLisensi(ekspresi: unknown): PohonLisensi | null;
export function komponenLisensi(ekspresi: unknown): string[] | null;
export function periksaPaket(paket: PaketLisensi[], mode: "runtime" | "dev"): Pelanggaran[];
export function ratakan(keluaran: Record<string, { name: string; versions?: string[] }[]>): PaketLisensi[];
export function jalankan(webDir: string): number;
