import { render, screen } from "@testing-library/react";
import { afterEach, describe, expect, test, vi } from "vitest";

const ambil = vi.hoisted(() => vi.fn());
vi.mock("../../../konten/halaman.ts", () => ({ ambilHalamanTayang: ambil }));
vi.mock("next/navigation", () => ({
  usePathname: () => "/layanan/layanan-1",
  notFound: () => {
    throw new Error("NEXT_NOT_FOUND");
  },
}));

import HalamanDetailLayanan from "./page.tsx";

const MD = "## Konsultasi dokter\n\nChat dan video.\n\n### Cara memakai\n\n#### Pilih\n\nA.\n\n#### Bicara\n\nB.\n\n## Home visit\n\nKunjungan.";
afterEach(() => {
  ambil.mockReset();
});

async function muat(id: string) {
  return await HalamanDetailLayanan({ params: Promise.resolve({ id }) });
}

describe("rute detail layanan: pemeriksaan parameter", () => {
  test("ID layanan-N yang ada dirender dengan judul dan ringkasan kartu", async () => {
    ambil.mockResolvedValue({ body: MD });
    render(await muat("layanan-2"));
    expect(screen.getByRole("heading", { level: 1 }).textContent).toBe("Home visit");
  });

  test.each(["layanan-999", "xyz", "layanan-", "layanan-1x", "LAYANAN-1", "layanan-1/../x", "layanan-1234", "..%2f", "", "layanan-0001"])("ID %j menjadi tidak ditemukan", async (id) => {
    ambil.mockResolvedValue({ body: MD });
    await expect(muat(id)).rejects.toThrow("NEXT_NOT_FOUND");
  });

  test("ID di luar pola tidak pernah memanggil CMS; halaman layanan tak berstruktur kartu → tidak ditemukan", async () => {
    await expect(muat("xyz")).rejects.toThrow("NEXT_NOT_FOUND");
    expect(ambil).not.toHaveBeenCalled();
    ambil.mockResolvedValue({ body: "# Bukan kartu\n\nIsi." });
    await expect(muat("layanan-1")).rejects.toThrow("NEXT_NOT_FOUND");
  });
});
