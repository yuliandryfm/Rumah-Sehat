import { createOpsClient } from "@rs/api-client";
import { KontenTidakTersedia, type KlienKonten, type KontenBeranda } from "./tipe.ts";

export type KlienKontenApi = KlienKonten & { readonly klienApi: ReturnType<typeof createOpsClient> };

/**
 * Klien konten ke backend lewat `@rs/api-client`. Endpoint `/v1/public/content/*` belum ada di
 * `openapi/ops.json` (dibuat task lain, F1-T94), jadi pemanggilan bertipe belum mungkin: sampai spesifikasi
 * itu masuk dan klien digenerasi ulang, setiap metode melempar `KontenTidakTersedia` (halaman menampilkan
 * galat umum, bukan data karangan). Klien dibuat sekarang supaya basis URL dan opsi (tanpa kredensial, tanpa
 * cookie) sudah terkunci di satu tempat.
 */
export function buatKlienKontenApi(baseUrl: string): KlienKontenApi {
  // `credentials: "omit"`: situs publik tidak mengirim atau menerima cookie ke backend.
  const klienApi = createOpsClient({ baseUrl, credentials: "omit" });
  return {
    klienApi,
    ambilBeranda(): Promise<KontenBeranda> {
      return Promise.reject(new KontenTidakTersedia("endpoint konten publik belum tersedia di openapi/ops.json"));
    },
  };
}
