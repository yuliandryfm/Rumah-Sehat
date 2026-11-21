import { cleanup, render, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, test, vi } from "vitest";
import TidakDitemukan from "../app/not-found.tsx";
import Galat from "../app/error.tsx";
import type { Konten } from "../konten/tipe.ts";
import { SEMBILAN_HALAMAN } from "../navigasi.ts";
import { Beranda, type DataBeranda } from "./Beranda.tsx";
import { KerangkaSitus } from "./KerangkaSitus.tsx";

const jalur = vi.hoisted(() => ({ nilai: "/" }));
vi.mock("next/navigation", () => ({ usePathname: () => jalur.nilai }));

afterEach(() => {
  cleanup();
  jalur.nilai = "/";
});

describe("KerangkaSitus: landmark dan navigasi", () => {
  test("header, main, footer, tautan lewati, dan dua navigasi utama berlabel berbeda", () => {
    render(
      <KerangkaSitus>
        <p>isi</p>
      </KerangkaSitus>,
    );
    expect(screen.getByRole("banner")).toBeDefined();
    const main = screen.getByRole("main");
    expect(main.id).toBe("isi");
    expect(screen.getByRole("contentinfo")).toBeDefined();
    expect(screen.getByRole("link", { name: "Lewati ke isi" }).getAttribute("href")).toBe("#isi");
    const nama = screen.getAllByRole("navigation").map((n) => n.getAttribute("aria-label"));
    expect(nama).toEqual(["Utama", "Utama (seluler)", "Tautan kaki"]);
    expect(new Set(nama).size).toBe(nama.length);
  });

  test("sembilan halaman dapat dicapai dari header dan footer", () => {
    render(
      <KerangkaSitus>
        <p>isi</p>
      </KerangkaSitus>,
    );
    expect(SEMBILAN_HALAMAN).toHaveLength(9);
    const href = new Set(screen.getAllByRole("link").map((a) => a.getAttribute("href")));
    for (const h of SEMBILAN_HALAMAN) expect(href.has(h.href), h.href).toBe(true);
  });

  test("halaman aktif ditandai aria-current=page pada navigasi", () => {
    jalur.nilai = "/layanan";
    render(
      <KerangkaSitus>
        <p>isi</p>
      </KerangkaSitus>,
    );
    const nav = screen.getAllByRole("navigation", { name: "Utama" })[0];
    if (nav === undefined) throw new Error("navigasi tidak ada");
    const aktif = within(nav).getAllByRole("link", { current: "page" });
    expect(aktif.map((a) => a.textContent)).toEqual(["Layanan"]);
  });

  test("menu seluler memakai details/summary berlabel teks, tanpa skrip", () => {
    const { container } = render(
      <KerangkaSitus>
        <p>isi</p>
      </KerangkaSitus>,
    );
    const ringkasan = container.querySelector("details > summary");
    expect(ringkasan?.textContent).toBe("Menu");
    expect(container.querySelector("script")).toBeNull();
  });

  test("setiap tautan dan tombol punya nama yang dapat dibaca", () => {
    render(
      <KerangkaSitus>
        <p>isi</p>
      </KerangkaSitus>,
    );
    for (const a of screen.getAllByRole("link")) expect(a.textContent.trim().length, a.outerHTML).toBeGreaterThan(0);
  });
});

const KONTEN: Konten = { content_id: "0193a1b2-c3d4-7e5f-8a6b-7c8d9e0f1a2b", content_type: "info_page", title: "Judul CMS", body: "Isi **CMS**", published_at: "2026-11-20T00:00:00Z" };
const KOSONG: DataBeranda = { beranda: null, layanan: null, untukPerusahaan: null, artikelTerbaru: [] };

describe("Beranda: heading dan penanda", () => {
  test("CMS kosong: tepat satu h1, h2 per bagian, tampil [TIDAK DIKETAHUI] bukan teks karangan", () => {
    render(<Beranda data={KOSONG} />);
    expect(screen.getAllByRole("heading", { level: 1 })).toHaveLength(1);
    expect(screen.getAllByRole("heading", { level: 2 }).map((h) => h.textContent)).toEqual(["Layanan", "Untuk Perusahaan", "Artikel terbaru"]);
    expect(screen.getAllByText("[TIDAK DIKETAHUI]").length).toBeGreaterThanOrEqual(4);
    expect(screen.getByRole("heading", { level: 1 }).textContent).toContain("[TIDAK DIKETAHUI]");
  });

  test("dengan konten CMS: judul dan isi tampil, artikel terbaru bertautan ke detail", () => {
    render(<Beranda data={{ ...KOSONG, beranda: KONTEN, artikelTerbaru: [{ ...KONTEN, content_type: "article", title: "Artikel A" }] }} />);
    expect(screen.getByRole("heading", { level: 1 }).textContent).toBe("Judul CMS");
    expect(screen.getByText("CMS").tagName).toBe("STRONG");
    expect(screen.getByRole("link", { name: "Artikel A" }).getAttribute("href")).toBe(`/artikel/${KONTEN.content_id}`);
  });

  test("teks CMS dirender sebagai teks (React meng-escape), bukan HTML", () => {
    const { container } = render(<Beranda data={{ ...KOSONG, beranda: { ...KONTEN, title: "<img src=x onerror=alert(1)>" } }} />);
    expect(container.querySelector("img")).toBeNull();
    expect(screen.getByRole("heading", { level: 1 }).textContent).toBe("<img src=x onerror=alert(1)>");
  });
});

describe("halaman tidak ditemukan dan galat umum", () => {
  test("404: h1 dan tautan kembali ke Beranda", () => {
    render(<TidakDitemukan />);
    expect(screen.getByRole("heading", { level: 1 }).textContent).toBe("Halaman tidak ditemukan");
    expect(screen.getByRole("link", { name: "Ke Beranda" }).getAttribute("href")).toBe("/");
  });

  test("galat: pesan umum tanpa rincian galat, dapat dicoba ulang → tidak membocorkan internal", () => {
    const reset = vi.fn();
    const galat = Object.assign(new Error("rahasia: koneksi db://internal gagal"), { digest: "d1g3st" });
    const { container } = render(<Galat error={galat} reset={reset} />);
    expect(screen.getByRole("alert")).toBeDefined();
    expect(screen.getByRole("heading", { level: 1 }).textContent).toBe("Terjadi kesalahan");
    expect(container.textContent).not.toContain("rahasia");
    expect(container.textContent).not.toContain("d1g3st");
    screen.getByRole("button", { name: "Coba lagi" }).click();
    expect(reset).toHaveBeenCalledOnce();
  });
});
