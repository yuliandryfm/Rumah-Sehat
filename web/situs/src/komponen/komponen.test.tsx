import { cleanup, render, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, test, vi } from "vitest";
import TidakDitemukan from "../app/not-found.tsx";
import Galat from "../app/error.tsx";
import { BERANDA_CONTOH } from "../konten/contoh.ts";
import { SEMBILAN_HALAMAN } from "../navigasi.ts";
import { Beranda } from "./Beranda.tsx";
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

describe("Beranda: heading dan penanda", () => {
  test("tepat satu h1, h2 untuk bagian, h3 untuk kartu → hierarki tidak melompat", () => {
    render(<Beranda konten={BERANDA_CONTOH} />);
    expect(screen.getAllByRole("heading", { level: 1 })).toHaveLength(1);
    expect(screen.getAllByRole("heading", { level: 2 }).map((h) => h.textContent)).toEqual(["Layanan", "Untuk Perusahaan", "Artikel terbaru"]);
    expect(screen.getAllByRole("heading", { level: 3 })).toHaveLength(6);
  });

  test("teks yang belum diketahui tampil sebagai [TIDAK DIKETAHUI], bukan teks karangan", () => {
    render(<Beranda konten={BERANDA_CONTOH} />);
    expect(screen.getAllByText("[TIDAK DIKETAHUI]").length).toBeGreaterThanOrEqual(3);
    expect(screen.getByRole("heading", { level: 1 }).textContent).toContain("[TIDAK DIKETAHUI]");
  });

  test("teks CMS dirender sebagai teks (React meng-escape), bukan HTML", () => {
    const { container } = render(<Beranda konten={{ ...BERANDA_CONTOH, judulUtama: "<img src=x onerror=alert(1)>" }} />);
    expect(container.querySelector("img")).toBeNull();
    expect(screen.getByRole("heading", { level: 1 }).textContent).toBe("<img src=x onerror=alert(1)>");
  });

  test("gambar placeholder punya teks alternatif", () => {
    render(<Beranda konten={BERANDA_CONTOH} />);
    expect(screen.getByRole("img").getAttribute("aria-label")).toMatch(/Placeholder gambar utama/);
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
