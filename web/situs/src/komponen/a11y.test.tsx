import { cleanup, render } from "@testing-library/react";
import axe from "axe-core";
import type { ReactNode } from "react";
import { afterEach, beforeAll, describe, expect, test, vi } from "vitest";
import Galat from "../app/error.tsx";
import TidakDitemukan from "../app/not-found.tsx";
import type { Konten } from "../konten/tipe.ts";
import { DaftarArtikel, DetailArtikel } from "./Artikel.tsx";
import { Beranda, type DataBeranda } from "./Beranda.tsx";
import { HalamanKonten } from "./HalamanKonten.tsx";
import { KerangkaSitus } from "./KerangkaSitus.tsx";

vi.mock("next/navigation", () => ({ usePathname: () => "/" }));

beforeAll(() => {
  document.documentElement.lang = "id";
});
afterEach(cleanup);

const IMG = "/v1/public/content/images/0193a1b2-c3d4-7e5f-8a6b-7c8d9e0f1a2b";
const BODY = `# Bagian satu\n\nTeks dengan [tautan](/layanan) dan **tebal**.\n\n## Sub\n\n- a\n- b\n\n![Foto klinik](${IMG})\n`;
const KONTEN: Konten = {
  content_id: "0193a1b2-c3d4-7e5f-8a6b-7c8d9e0f1a2b",
  content_type: "info_page",
  title: "Judul CMS",
  body: BODY,
  published_at: "2026-11-20T00:00:00Z",
  image: { url: IMG, alt_text: "Gedung klinik" },
};
const KOSONG: DataBeranda = { beranda: null, layanan: null, untukPerusahaan: null, artikelTerbaru: [] };
const PENUH: DataBeranda = { beranda: KONTEN, layanan: KONTEN, untukPerusahaan: KONTEN, artikelTerbaru: [{ ...KONTEN, content_type: "article" }] };

/**
 * Pemeriksaan aksesibilitas otomatis (US-1801 AC6): axe-core dengan tag WCAG 2.x A/AA dan praktik terbaik.
 * `color-contrast` dimatikan karena jsdom tidak menghitung gaya; kontras token diperiksa `pnpm kontras`
 * (design guideline §7.4).
 */
async function pelanggaranA11y(wadah: HTMLElement): Promise<string[]> {
  const hasil = await axe.run(wadah, {
    runOnly: { type: "tag", values: ["wcag2a", "wcag2aa", "wcag21a", "wcag21aa", "wcag22aa", "best-practice"] },
    rules: { "color-contrast": { enabled: false } },
  });
  return hasil.violations.map((v) => `${v.id}: ${v.nodes.map((n) => n.target.join(" ")).join(", ")}`);
}

function Halaman({ children }: { children: ReactNode }) {
  return <KerangkaSitus>{children}</KerangkaSitus>;
}

describe("aksesibilitas otomatis (axe-core)", () => {
  test.each([
    ["Beranda, CMS kosong", <Beranda key="a" data={KOSONG} />],
    ["Beranda, CMS terisi", <Beranda key="b" data={PENUH} />],
    ["Halaman isi CMS", <HalamanKonten key="c" konten={KONTEN} />],
    ["Halaman isi bertanda draf (kebijakan)", <HalamanKonten key="f" konten={KONTEN} draf />],
    ["Daftar artikel, ada kursor", <DaftarArtikel key="g" pengantar={KONTEN} artikel={[{ ...KONTEN, content_type: "article" }]} kursorBerikut={KONTEN.content_id} />],
    ["Daftar artikel kosong", <DaftarArtikel key="h" pengantar={null} artikel={[]} kursorBerikut={null} />],
    ["Detail artikel", <DetailArtikel key="i" artikel={{ ...KONTEN, content_type: "article", category: "Kesehatan" }} />],
    ["Halaman tidak ditemukan", <TidakDitemukan key="d" />],
    ["Galat umum", <Galat key="e" error={new Error("x")} reset={() => undefined} />],
  ])("%s dalam kerangka situs: tanpa pelanggaran", async (_nama, isi) => {
    const { container } = render(<Halaman>{isi}</Halaman>);
    expect(await pelanggaranA11y(container)).toEqual([]);
  });
});
