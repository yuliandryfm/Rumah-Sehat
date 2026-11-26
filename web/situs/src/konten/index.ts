import { buatKlienKontenApi } from "./api.ts";
import { KlienKontenContoh } from "./contoh.ts";
import { KontenTidakTersedia, type KlienKonten } from "./tipe.ts";

export * from "./tipe.ts";

/**
 * Memilih klien konten dari lingkungan (dibaca per permintaan, di server saja):
 * - `SITUS_KONTEN_SUMBER=contoh`: data contoh (hanya test dan pratinjau lokal).
 * - selain itu: API backend di `SITUS_API_BASE_URL` (wajib; alamat Caddy untuk `/v1/public/*`, nilainya
 *   `[TIDAK DIKETAHUI]` sampai F1-T111). Tanpa nilai, `KontenTidakTersedia`.
 * Bawaan aman: tanpa pengaturan eksplisit, situs tidak pernah menampilkan data contoh.
 * Variabel situs lain: `SITUS_KORPORAT_URL` (opsional, CR-032) = alamat member area korporat; fail-closed, tautan
 * tidak dirender bila kosong atau tidak valid (`tautan-korporat.ts`).
 */
export function ambilKlienKonten(env: Record<string, string | undefined> = process.env): KlienKonten {
  if (env["SITUS_KONTEN_SUMBER"] === "contoh") return new KlienKontenContoh();
  const baseUrl = env["SITUS_API_BASE_URL"];
  if (baseUrl === undefined || baseUrl === "") throw new KontenTidakTersedia("SITUS_API_BASE_URL belum diatur");
  return buatKlienKontenApi(baseUrl);
}
