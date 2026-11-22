import { describe, expect, test, vi } from "vitest";
import { buatKlienKontenApi } from "./api.ts";
import { KlienKontenContoh } from "./contoh.ts";
import { ambilKlienKonten, KontenTidakTersedia } from "./index.ts";

const ID = "0193a1b2-c3d4-7e5f-8a6b-7c8d9e0f1a2b";
const DTO = { content_id: ID, content_type: "info_page", title: "Judul uji", body: "isi", published_at: "2026-11-20T00:00:00Z" };

function respons(status: number, badan: unknown): Response {
  return new Response(JSON.stringify(badan), { status, headers: { "content-type": "application/json" } });
}

describe("ambilKlienKonten", () => {
  test("sumber contoh dipilih hanya bila diminta eksplisit", () => {
    expect(ambilKlienKonten({ SITUS_KONTEN_SUMBER: "contoh" })).toBeInstanceOf(KlienKontenContoh);
  });

  test("tanpa pengaturan → galat, bukan data contoh (tidak ada konten karangan di produksi)", () => {
    expect(() => ambilKlienKonten({})).toThrow(KontenTidakTersedia);
    expect(() => ambilKlienKonten({ SITUS_API_BASE_URL: "" })).toThrow(KontenTidakTersedia);
  });

  test("dengan alamat API → klien API", () => {
    expect(ambilKlienKonten({ SITUS_API_BASE_URL: "http://api.test" })).not.toBeInstanceOf(KlienKontenContoh);
  });
});

describe("klien API publik", () => {
  test("halaman: memanggil /v1/public/content/pages/{halaman} tanpa kredensial dan tanpa cookie", async () => {
    const fetchTiruan = vi.fn<typeof fetch>(() => Promise.resolve(respons(200, { response_code: "2000000", response_description: "Berhasil", data: DTO })));
    const konten = await buatKlienKontenApi("http://api.test", fetchTiruan).ambilHalaman("about");
    expect(konten?.title).toBe("Judul uji");
    const req = fetchTiruan.mock.calls[0]?.[0] as Request | undefined;
    expect(req?.url).toBe("http://api.test/v1/public/content/pages/about");
    expect(req?.method).toBe("GET");
    expect(req?.credentials).toBe("omit");
    expect(req?.headers.get("cookie")).toBeNull();
    expect(req?.headers.get("authorization")).toBeNull();
  });

  test("404 → null (draf dan tidak ada tidak dibedakan)", async () => {
    const f = () => Promise.resolve(respons(404, { response_code: "4040001" }));
    const klien = buatKlienKontenApi("http://api.test", f);
    expect(await klien.ambilHalaman("about")).toBeNull();
    expect(await klien.ambilItem(ID)).toBeNull();
  });

  test("500, 429, galat jaringan, atau bentuk respons salah → KontenTidakTersedia tanpa membawa isi respons", async () => {
    for (const f of [
      () => Promise.resolve(respons(500, { rahasia: "jejak internal" })),
      () => Promise.resolve(respons(429, {})),
      () => Promise.reject(new Error("ECONNREFUSED db://internal")),
      () => Promise.resolve(respons(200, { data: { bukan: "konten" } })),
    ]) {
      const klien = buatKlienKontenApi("http://api.test", f);
      const galat = await klien.ambilHalaman("home").catch((e: unknown) => e);
      expect(galat).toBeInstanceOf(KontenTidakTersedia);
      expect((galat as Error).message).not.toContain("jejak internal");
      expect((galat as Error).message).not.toContain("db://");
    }
  });

  test("daftar: saringan dan kursor masuk query, kursor berikut dari meta", async () => {
    const fetchTiruan = vi.fn<typeof fetch>(() =>
      Promise.resolve(respons(200, { response_code: "2000000", response_description: "Berhasil", data: [DTO], meta: { has_more: true, next_cursor: ID } })),
    );
    const hasil = await buatKlienKontenApi("http://api.test", fetchTiruan).daftar({ jenis: "article", batas: 3, kursor: ID });
    expect(hasil.item).toHaveLength(1);
    expect(hasil.kursorBerikut).toBe(ID);
    const url = new URL((fetchTiruan.mock.calls[0]?.[0] as Request).url);
    expect(url.pathname).toBe("/v1/public/content/items");
    expect(url.searchParams.get("content_type")).toBe("article");
    expect(url.searchParams.get("limit")).toBe("3");
    expect(url.searchParams.get("cursor")).toBe(ID);
  });
});

describe("klien contoh", () => {
  test("tidak mengarang konten bisnis: Beranda dan Artikel kosong, halaman lain hanya isi penanda contoh", async () => {
    const klien = new KlienKontenContoh();
    expect(await klien.ambilHalaman("home")).toBeNull();
    expect(await klien.ambilHalaman("articles")).toBeNull();
    expect((await klien.ambilHalaman("about"))?.body).toMatch(/Bukan konten bisnis/);
    expect((await klien.daftar()).item).toEqual([]);
  });
});
