export type JenisPasangan = "teks" | "non-teks" | "larangan-teks";

export interface Pasangan {
  id: string;
  fg: string;
  bg: string[];
  jenis: JenisPasangan;
}

export interface BarisKontras {
  id: string;
  fg: string;
  bg: string;
  rasio: number;
  syarat: number;
  jenis: JenisPasangan;
  lulus: boolean;
  catatan: string;
}

export const PASANGAN: Pasangan[];
export function bacaWarna(css: string): Map<string, string>;
export function luminance(hex: string): number;
export function rasioKontras(hexA: string, hexB: string): number;
export function periksaKontras(css: string, pasangan?: Pasangan[]): BarisKontras[];
