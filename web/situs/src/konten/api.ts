import { createPublicClient } from "@rs/api-client";
import { KontenTidakTersedia, type HalamanSitus, type HasilDaftar, type KlienKonten, type Konten, type OpsiDaftar } from "./tipe.ts";

const BATAS_WAKTU_MS = 5000;

/** Memeriksa bentuk minimum satu konten; selain itu dianggap galat backend (tidak dirender). */
function pastikanKonten(data: unknown): Konten {
  if (typeof data !== "object" || data === null) throw new KontenTidakTersedia("bentuk respons konten tidak valid");
  const k = data as Partial<Konten>;
  if (typeof k.content_id !== "string" || typeof k.title !== "string") throw new KontenTidakTersedia("bentuk respons konten tidak valid");
  return k as Konten;
}

/**
 * Klien konten ke backend lewat `@rs/api-client` (`openapi/public.json`). Tanpa kredensial dan tanpa cookie
 * (`credentials: "omit"`), tanpa cache (konten yang dicabut harus segera hilang, US-1801 AC3), dengan batas
 * waktu. 404 menjadi `null`; status lain atau galat jaringan menjadi `KontenTidakTersedia` (galat umum,
 * US-1801 AC7).
 */
export function buatKlienKontenApi(baseUrl: string, fetchFn?: typeof fetch): KlienKonten {
  const klien = createPublicClient({
    baseUrl,
    credentials: "omit",
    cache: "no-store",
    ...(fetchFn === undefined ? {} : { fetch: fetchFn }),
  });
  const sinyal = () => AbortSignal.timeout(BATAS_WAKTU_MS);

  async function bungkus<T>(panggil: () => Promise<{ data?: T; response: Response }>): Promise<T | null> {
    let hasil;
    try {
      hasil = await panggil();
    } catch {
      throw new KontenTidakTersedia("backend konten tidak terjangkau");
    }
    if (hasil.response.status === 404) return null;
    if (!hasil.response.ok || hasil.data === undefined) throw new KontenTidakTersedia(`backend konten membalas ${String(hasil.response.status)}`);
    return hasil.data;
  }

  return {
    async ambilHalaman(halaman: HalamanSitus): Promise<Konten | null> {
      const data = await bungkus(() =>
        klien.GET("/v1/public/content/pages/{site_page}", { params: { path: { site_page: halaman } }, signal: sinyal() }),
      );
      return data === null ? null : pastikanKonten(data.data);
    },
    async ambilItem(id: string): Promise<Konten | null> {
      const data = await bungkus(() =>
        klien.GET("/v1/public/content/items/{content_id}", { params: { path: { content_id: id } }, signal: sinyal() }),
      );
      return data === null ? null : pastikanKonten(data.data);
    },
    async daftar(opsi: OpsiDaftar = {}): Promise<HasilDaftar> {
      const data = await bungkus(() =>
        klien.GET("/v1/public/content/items", {
          params: {
            query: {
              ...(opsi.halaman === undefined ? {} : { site_page: opsi.halaman }),
              ...(opsi.jenis === undefined ? {} : { content_type: opsi.jenis }),
              ...(opsi.kursor === undefined ? {} : { cursor: opsi.kursor }),
              ...(opsi.batas === undefined ? {} : { limit: opsi.batas }),
            },
          },
          signal: sinyal(),
        }),
      );
      if (data === null || !Array.isArray(data.data)) throw new KontenTidakTersedia("bentuk respons daftar konten tidak valid");
      return { item: data.data.map(pastikanKonten), kursorBerikut: data.meta.next_cursor ?? null };
    },
  };
}
