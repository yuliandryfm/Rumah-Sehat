import { afterEach, describe, expect, test, vi } from "vitest";

const tidakDitemukan = vi.hoisted(() => new Error("NEXT_NOT_FOUND"));
vi.mock("next/navigation", () => ({
  notFound: () => {
    throw tidakDitemukan;
  },
}));

import HalamanDetailArtikel from "./artikel/[id]/page.tsx";
import HalamanArtikel from "./artikel/page.tsx";
import HalamanKebijakan from "./kebijakan/page.tsx";

const ID = "0193a1b2-c3d4-7e5f-8a6b-7c8d9e0f1a2b";

afterEach(() => vi.unstubAllEnvs());

describe("rute situs (klien contoh)", () => {
  test("detail artikel: ID bukan UUID → tidak ditemukan tanpa memanggil backend", async () => {
    vi.stubEnv("SITUS_KONTEN_SUMBER", "");
    vi.stubEnv("SITUS_API_BASE_URL", "");
    await expect(HalamanDetailArtikel({ params: Promise.resolve({ id: "../../etc/passwd" }) })).rejects.toBe(tidakDitemukan);
  });

  test("detail artikel: tidak tayang → tidak ditemukan", async () => {
    vi.stubEnv("SITUS_KONTEN_SUMBER", "contoh");
    await expect(HalamanDetailArtikel({ params: Promise.resolve({ id: ID }) })).rejects.toBe(tidakDitemukan);
  });

  test("daftar artikel dan kebijakan: terender dengan klien contoh; kursor non-UUID diabaikan", async () => {
    vi.stubEnv("SITUS_KONTEN_SUMBER", "contoh");
    expect(await HalamanArtikel({ searchParams: Promise.resolve({ kursor: "'; drop table" }) })).toBeDefined();
    const kebijakan = await HalamanKebijakan();
    expect(kebijakan.props).toMatchObject({ draf: true });
  });

  test("sumber konten belum diatur → galat (galat umum), bukan halaman kosong", async () => {
    vi.stubEnv("SITUS_KONTEN_SUMBER", "");
    vi.stubEnv("SITUS_API_BASE_URL", "");
    await expect(HalamanArtikel({ searchParams: Promise.resolve({}) })).rejects.toMatchObject({ name: "KontenTidakTersedia" });
  });
});
