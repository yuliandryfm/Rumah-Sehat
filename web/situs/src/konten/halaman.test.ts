import { afterEach, describe, expect, test, vi } from "vitest";

const tidakDitemukan = vi.hoisted(() => new Error("NEXT_NOT_FOUND"));
vi.mock("next/navigation", () => ({
  notFound: () => {
    throw tidakDitemukan;
  },
}));

import { ambilHalamanTayang } from "./halaman.ts";

afterEach(() => vi.unstubAllEnvs());

describe("ambilHalamanTayang", () => {
  test("ada versi tayang → konten", async () => {
    vi.stubEnv("SITUS_KONTEN_SUMBER", "contoh");
    expect((await ambilHalamanTayang("about")).title).toBe("Tentang Kami");
  });

  test("tidak ada versi tayang (draf dan lainnya) → halaman tidak ditemukan", async () => {
    vi.stubEnv("SITUS_KONTEN_SUMBER", "contoh");
    await expect(ambilHalamanTayang("home")).rejects.toBe(tidakDitemukan);
  });

  test("sumber konten belum diatur → galat (galat umum), bukan tidak ditemukan", async () => {
    vi.stubEnv("SITUS_KONTEN_SUMBER", "");
    vi.stubEnv("SITUS_API_BASE_URL", "");
    await expect(ambilHalamanTayang("about")).rejects.toMatchObject({ name: "KontenTidakTersedia" });
  });
});
