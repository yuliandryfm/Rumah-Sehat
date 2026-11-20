import { describe, expect, test } from "vitest";
import { buatKlienKontenApi } from "./api.ts";
import { BERANDA_CONTOH, KlienKontenContoh } from "./contoh.ts";
import { ambilKlienKonten, KontenTidakTersedia, type KlienKonten } from "./index.ts";

describe("ambilKlienKonten", () => {
  test("sumber contoh dipilih hanya bila diminta eksplisit", async () => {
    const klien = ambilKlienKonten({ SITUS_KONTEN_SUMBER: "contoh" });
    expect(klien).toBeInstanceOf(KlienKontenContoh);
    expect(await klien.ambilBeranda()).toEqual(BERANDA_CONTOH);
  });

  test("tanpa pengaturan → galat, bukan data contoh (tidak ada konten karangan di produksi)", () => {
    expect(() => ambilKlienKonten({})).toThrow(KontenTidakTersedia);
    expect(() => ambilKlienKonten({ SITUS_API_BASE_URL: "" })).toThrow(KontenTidakTersedia);
  });

  test("dengan alamat API → klien API; endpoint belum ada di openapi → KontenTidakTersedia", async () => {
    const klien = ambilKlienKonten({ SITUS_API_BASE_URL: "http://api.test" });
    await expect(klien.ambilBeranda()).rejects.toBeInstanceOf(KontenTidakTersedia);
    await expect(buatKlienKontenApi("http://api.test").ambilBeranda()).rejects.toThrow(/belum tersedia/);
  });

  test("antarmuka dapat di-mock oleh halaman", async () => {
    const tiruan: KlienKonten = { ambilBeranda: () => Promise.resolve({ ...BERANDA_CONTOH, judulUtama: "Judul uji" }) };
    expect((await tiruan.ambilBeranda()).judulUtama).toBe("Judul uji");
  });
});

describe("data contoh", () => {
  test("tidak mengarang konten bisnis: teks yang belum diketahui bernilai null", () => {
    expect(BERANDA_CONTOH.judulUtama).toBeNull();
    expect(BERANDA_CONTOH.kalimatPembuka).toBeNull();
    expect(BERANDA_CONTOH.ringkasanKorporat).toBeNull();
    expect(BERANDA_CONTOH.layanan).toHaveLength(6);
  });
});
