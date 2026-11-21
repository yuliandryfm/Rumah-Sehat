import { notFound } from "next/navigation";
import { ambilKlienKonten } from "./index.ts";
import type { HalamanSitus, Konten } from "./tipe.ts";

/**
 * Halaman tetap yang sedang tayang. Tidak ada versi tayang (draf, terjadwal, berakhir, diarsipkan, terhapus)
 * menjadi halaman tidak ditemukan tanpa membedakan sebabnya (US-1801 AC3). Galat backend dilempar ke
 * `error.tsx` (galat umum, US-1801 AC7).
 */
export async function ambilHalamanTayang(halaman: HalamanSitus): Promise<Konten> {
  const konten = await ambilKlienKonten().ambilHalaman(halaman);
  if (konten === null) notFound();
  return konten;
}
