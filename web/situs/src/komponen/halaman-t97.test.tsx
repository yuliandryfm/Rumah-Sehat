import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import type { Konten } from "../konten/tipe.ts";
import { DaftarArtikel, DetailArtikel } from "./Artikel.tsx";
import { HalamanKonten } from "./HalamanKonten.tsx";

afterEach(cleanup);

const ID = "0193a1b2-c3d4-7e5f-8a6b-7c8d9e0f1a2b";
const ID2 = "0193a1b2-c3d4-7e5f-8a6b-7c8d9e0f1a2c";
const ARTIKEL: Konten = { content_id: ID, content_type: "article", title: "Judul artikel", body: "Isi **artikel**", published_at: "2026-11-20T10:00:00Z", category: "Kesehatan" };

describe("Kebijakan privasi dan syarat bertanda draf (US-1801 AC4)", () => {
  test("penanda draf, tanggal berlaku dari tanggal tayang, versi [TIDAK DIKETAHUI]", () => {
    render(<HalamanKonten konten={{ ...ARTIKEL, content_type: "info_page", title: "Kebijakan" }} draf />);
    const catatan = screen.getByRole("note");
    expect(catatan.textContent).toContain("Draf.");
    expect(catatan.textContent).toContain("Berlaku sejak: 20 November 2026");
    expect(catatan.textContent).toMatch(/Versi: \[TIDAK DIKETAHUI\]/);
  });

  test("tanggal tidak terbaca → [TIDAK DIKETAHUI], bukan tanggal karangan", () => {
    render(<HalamanKonten konten={{ ...ARTIKEL, published_at: "bukan-tanggal" }} draf />);
    expect(screen.getByRole("note").textContent).toMatch(/Berlaku sejak: \[TIDAK DIKETAHUI\]\./);
  });

  test("halaman biasa tanpa penanda draf", () => {
    render(<HalamanKonten konten={ARTIKEL} />);
    expect(screen.queryByRole("note")).toBeNull();
  });

  test("isi kosong dari CMS → penanda [TIDAK DIKETAHUI]", () => {
    render(<HalamanKonten konten={{ ...ARTIKEL, body: null }} />);
    expect(screen.getByText("[TIDAK DIKETAHUI]")).toBeDefined();
  });
});

describe("Artikel", () => {
  test("daftar: tautan ke detail, tanggal WIB, kursor berikut ke ?kursor=", () => {
    render(<DaftarArtikel pengantar={null} artikel={[ARTIKEL, { ...ARTIKEL, content_id: ID2, title: "Dua" }]} kursorBerikut={ID2} />);
    expect(screen.getByRole("heading", { level: 1 }).textContent).toBe("Artikel");
    expect(screen.getByRole("link", { name: "Judul artikel" }).getAttribute("href")).toBe(`/artikel/${ID}`);
    expect(screen.getByRole("link", { name: "Artikel berikutnya" }).getAttribute("href")).toBe(`/artikel?kursor=${ID2}`);
    expect(screen.getAllByText("20 November 2026").length).toBe(2);
  });

  test("daftar kosong: pesan dan penanda, tanpa tombol berikutnya", () => {
    render(<DaftarArtikel pengantar={null} artikel={[]} kursorBerikut={null} />);
    expect(screen.getByText("[TIDAK DIKETAHUI]")).toBeDefined();
    expect(screen.queryByRole("link", { name: "Artikel berikutnya" })).toBeNull();
  });

  test("detail: judul h1, kategori, isi Markdown dirender aman", () => {
    const { container } = render(<DetailArtikel artikel={{ ...ARTIKEL, body: "Isi **artikel** <script>alert(1)</script>" }} />);
    expect(screen.getByRole("heading", { level: 1 }).textContent).toBe("Judul artikel");
    expect(container.textContent).toContain("Kesehatan");
    expect(container.querySelector("strong")?.textContent).toBe("artikel");
    expect(container.querySelector("script")).toBeNull();
  });
});
