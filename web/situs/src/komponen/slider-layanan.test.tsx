import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { act, cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import { renderToStaticMarkup } from "react-dom/server";
import { afterEach, beforeEach, describe, expect, test, vi } from "vitest";
import { ikonUntukJudul, kategoriLayanan, uraiLayanan } from "../konten/struktur.ts";
import type { Konten } from "../konten/tipe.ts";
import { Beranda, type DataBeranda } from "./Beranda.tsx";
import { type KartuSlider, SliderLayanan } from "./SliderLayanan.tsx";

const sini = dirname(fileURLToPath(import.meta.url));
const css = readFileSync(join(sini, "../app/situs.css"), "utf8");

const TUJUH = [
  "Konsultasi dokter",
  "Konsultasi psikolog",
  "Pendaftaran klinik",
  "Pendaftaran lab",
  "Home visit",
  "Informasi biaya MCU dan lab",
  "Belanja produk kesehatan",
];
const MD = ["Pengantar.", ...TUJUH.flatMap((j) => ["", `## ${j}`, "", `Ringkasan ${j}.`])].join("\n");
const konten = (isi: Partial<Konten>): Konten => ({
  content_id: "0193a1b2-c3d4-7e5f-8a6b-7c8d9e0f1a2b",
  content_type: "info_page",
  title: "Tujuh layanan kesehatan dalam satu aplikasi",
  body: null,
  published_at: "2026-11-20T00:00:00Z",
  ...isi,
});
const DATA: DataBeranda = { beranda: null, layanan: konten({ body: MD }), untukPerusahaan: null, artikelTerbaru: [] };

const kartu = (): KartuSlider[] => (uraiLayanan(MD)?.kartu ?? []).map(({ id, judul, ringkasan, ikon }) => ({ id, judul, ringkasan, ikon }));
const tampilan = () => render(<SliderLayanan kartu={kartu()} kepala={<h2>Layanan</h2>} />);
const kartuTampak = (c: HTMLElement) => [...c.querySelectorAll<HTMLElement>(".situs-slider__kartu")].filter((li) => !li.hidden).map((li) => li.querySelector("h3")?.textContent);

/** jsdom tidak menata letak: ukuran jalur diatur manual supaya status tombol dapat diuji. */
function ukuran(jalur: HTMLElement, v: { scrollWidth: number; clientWidth: number; scrollLeft: number }) {
  for (const [k, n] of Object.entries(v)) Object.defineProperty(jalur, k, { configurable: true, writable: true, value: n });
}

const aktif = (): Element => document.activeElement ?? document.body;
const scrollBy = vi.fn();
const scrollTo = vi.fn();
beforeEach(() => {
  Object.defineProperty(HTMLElement.prototype, "scrollBy", { configurable: true, value: scrollBy });
  Object.defineProperty(HTMLElement.prototype, "scrollTo", { configurable: true, value: scrollTo });
  vi.stubGlobal("requestAnimationFrame", (f: () => void) => {
    f();
    return 0;
  });
});
afterEach(() => {
  cleanup();
  scrollBy.mockClear();
  scrollTo.mockClear();
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
});

describe("pemetaan kategori layanan (KEPUTUSAN-490)", () => {
  test("tujuh layanan rilis 1 terkelompok sesuai desain; korporat dan judul tak dikenal hanya di Semua", () => {
    const peta = Object.fromEntries(TUJUH.map((j) => [j, kategoriLayanan(ikonUntukJudul(j))]));
    expect(peta).toEqual({
      "Konsultasi dokter": "konsultasi",
      "Konsultasi psikolog": "konsultasi",
      "Pendaftaran klinik": "klinik-lab",
      "Pendaftaran lab": "klinik-lab",
      "Informasi biaya MCU dan lab": "klinik-lab",
      "Home visit": "home-belanja",
      "Belanja produk kesehatan": "home-belanja",
    });
    expect(kategoriLayanan(ikonUntukJudul("Layanan korporat"))).toBeNull();
    expect(kategoriLayanan(ikonUntukJudul("Kolaborasi"))).toBeNull();
  });
});

describe("penggeser layanan tanpa JS (peningkatan bertahap)", () => {
  const html = renderToStaticMarkup(<Beranda data={DATA} />);
  const dok = new DOMParser().parseFromString(html, "text/html");

  test("HTML server memuat ketujuh kartu terbaca dengan satu tautan per kartu; tidak ada kartu tersembunyi", () => {
    const li = [...dok.querySelectorAll(".situs-slider__daftar > li")];
    expect(li.map((k) => k.querySelector("h3")?.textContent)).toEqual(TUJUH);
    for (const k of li) {
      expect(k.hasAttribute("hidden")).toBe(false);
      expect(k.querySelectorAll("a")).toHaveLength(1);
      expect(k.querySelector("p")?.textContent).toMatch(/^Ringkasan /);
    }
  });

  test("tab dan tombol panah ada di HTML tetapi tersembunyi sampai JS berjalan; jalur tetap dapat difokus dan dibaca", () => {
    const kontrol = dok.querySelector(".situs-slider__kontrol");
    expect(kontrol?.hasAttribute("hidden")).toBe(true);
    expect(css).toMatch(/\.situs-slider__kontrol\[hidden\][^{]*\{ display: none; \}/);
    const jalur = dok.querySelector(".situs-slider__jalur");
    expect(jalur?.getAttribute("tabindex")).toBe("0");
    expect(jalur?.getAttribute("role")).toBe("region");
    expect(jalur?.getAttribute("aria-label")).toBe("Daftar layanan");
    expect(css).toMatch(/\.situs-slider__jalur \{[^}]*overflow-x: auto;[^}]*scroll-snap-type: x mandatory;/);
    expect(css).toMatch(/\.situs-slider__kartu \{[^}]*scroll-snap-align: start;/);
  });

  test("tanpa atribut style dan tanpa skrip sebaris (CSP)", () => {
    expect(html).not.toMatch(/\sstyle=|<script/i);
  });
});

describe("penggeser layanan dengan JS: tab filter", () => {
  test("empat tab (Semua, Konsultasi, Klinik & Lab, Home & Belanja); Semua terpilih dan menampilkan tujuh kartu", () => {
    const { container } = tampilan();
    const tab = within(screen.getByRole("tablist", { name: "Filter kategori layanan" })).getAllByRole("tab");
    expect(tab.map((t) => t.textContent)).toEqual(["Semua", "Konsultasi", "Klinik & Lab", "Home & Belanja"]);
    expect(tab.map((t) => t.getAttribute("aria-selected"))).toEqual(["true", "false", "false", "false"]);
    expect(tab.map((t) => t.tabIndex)).toEqual([0, -1, -1, -1]);
    expect(kartuTampak(container)).toEqual(TUJUH);
    expect(container.querySelector(".situs-slider__kontrol")?.hasAttribute("hidden")).toBe(false);
  });

  test.each([
    ["Konsultasi", ["Konsultasi dokter", "Konsultasi psikolog"]],
    ["Klinik & Lab", ["Pendaftaran klinik", "Pendaftaran lab", "Informasi biaya MCU dan lab"]],
    ["Home & Belanja", ["Home visit", "Belanja produk kesehatan"]],
  ])("klik tab %s hanya menampilkan kelompoknya, mengatur aria-selected dan mengembalikan gulir ke awal", (nama, harapan) => {
    const { container } = tampilan();
    fireEvent.click(screen.getByRole("tab", { name: nama }));
    expect(kartuTampak(container)).toEqual(harapan);
    expect(screen.getByRole("tab", { name: nama }).getAttribute("aria-selected")).toBe("true");
    expect(screen.getByRole("tab", { name: "Semua" }).getAttribute("aria-selected")).toBe("false");
    expect(scrollTo).toHaveBeenCalledWith({ left: 0, behavior: "auto" });
    expect(screen.getByRole("status").textContent).toBe(`Menampilkan ${String(harapan.length)} dari 7 layanan`);
    fireEvent.click(screen.getByRole("tab", { name: "Semua" }));
    expect(kartuTampak(container)).toEqual(TUJUH);
  });

  test("panel berperan tabpanel, dirujuk tab (aria-controls) dan bernama dari tab terpilih", () => {
    tampilan();
    const panel = screen.getByRole("tabpanel", { name: "Semua" });
    const tab = screen.getByRole("tab", { name: "Konsultasi" });
    expect(tab.getAttribute("aria-controls")).toBe(panel.id);
    fireEvent.click(tab);
    expect(screen.getByRole("tabpanel", { name: "Konsultasi" }).id).toBe(panel.id);
    expect(panel.getAttribute("tabindex")).toBe("0");
  });

  test("hanya kategori yang punya kartu mendapat tab; di bawah dua kategori filter tidak ditampilkan", () => {
    const dua = kartu().filter((k) => ["Konsultasi dokter", "Home visit"].includes(k.judul));
    render(<SliderLayanan kartu={dua} kepala={<h2>Layanan</h2>} />);
    expect(screen.getAllByRole("tab").map((t) => t.textContent)).toEqual(["Semua", "Konsultasi", "Home & Belanja"]);
    cleanup();
    render(<SliderLayanan kartu={dua.slice(0, 1)} kepala={<h2>Layanan</h2>} />);
    expect(screen.queryByRole("tablist")).toBeNull();
    expect(screen.getByRole("region", { name: "Daftar layanan" })).toBeDefined();
  });

  test("kartu di luar kelompok (korporat) hanya tampil di Semua", () => {
    const md = `${MD}\n\n## Layanan korporat\n\nUntuk perusahaan.`;
    const k = (uraiLayanan(md)?.kartu ?? []).map(({ id, judul, ringkasan, ikon }) => ({ id, judul, ringkasan, ikon }));
    const { container } = render(<SliderLayanan kartu={k} kepala={<h2>Layanan</h2>} />);
    expect(kartuTampak(container)).toHaveLength(8);
    for (const nama of ["Konsultasi", "Klinik & Lab", "Home & Belanja"]) {
      fireEvent.click(screen.getByRole("tab", { name: nama }));
      expect(kartuTampak(container)).not.toContain("Layanan korporat");
    }
  });
});

describe("penggeser layanan dengan JS: papan ketik", () => {
  test("ArrowRight/ArrowLeft pindah dan memilih tab (berputar), fokus ikut; tabindex berjalan", () => {
    const { container } = tampilan();
    const semua = screen.getByRole("tab", { name: "Semua" });
    semua.focus();
    fireEvent.keyDown(semua, { key: "ArrowRight" });
    const konsultasi = screen.getByRole("tab", { name: "Konsultasi" });
    expect(document.activeElement).toBe(konsultasi);
    expect(konsultasi.getAttribute("aria-selected")).toBe("true");
    expect(konsultasi.tabIndex).toBe(0);
    expect(semua.tabIndex).toBe(-1);
    expect(kartuTampak(container)).toHaveLength(2);
    fireEvent.keyDown(konsultasi, { key: "ArrowLeft" });
    expect(document.activeElement).toBe(semua);
    fireEvent.keyDown(semua, { key: "ArrowLeft" });
    expect(document.activeElement).toBe(screen.getByRole("tab", { name: "Home & Belanja" }));
    fireEvent.keyDown(aktif(), { key: "ArrowRight" });
    expect(document.activeElement).toBe(semua);
  });

  test("Home dan End ke tab pertama dan terakhir; tombol lain diabaikan", () => {
    tampilan();
    const semua = screen.getByRole("tab", { name: "Semua" });
    fireEvent.keyDown(semua, { key: "End" });
    expect(document.activeElement).toBe(screen.getByRole("tab", { name: "Home & Belanja" }));
    fireEvent.keyDown(aktif(), { key: "Home" });
    expect(document.activeElement).toBe(semua);
    fireEvent.keyDown(semua, { key: "a" });
    expect(semua.getAttribute("aria-selected")).toBe("true");
  });
});

describe("penggeser layanan dengan JS: tombol panah", () => {
  test("nama aksesibel Indonesia; di awal 'sebelumnya' nonaktif; di ujung 'berikutnya' nonaktif; di tengah keduanya aktif", () => {
    const { container } = tampilan();
    const jalur = container.querySelector<HTMLElement>(".situs-slider__jalur");
    if (jalur === null) throw new Error("jalur tidak ada");
    const sebelumnya = screen.getByRole<HTMLButtonElement>("button", { name: "Layanan sebelumnya" });
    const berikutnya = screen.getByRole<HTMLButtonElement>("button", { name: "Layanan berikutnya" });
    ukuran(jalur, { scrollWidth: 2000, clientWidth: 1000, scrollLeft: 0 });
    fireEvent.scroll(jalur);
    expect([sebelumnya.disabled, berikutnya.disabled]).toEqual([true, false]);
    ukuran(jalur, { scrollWidth: 2000, clientWidth: 1000, scrollLeft: 400 });
    fireEvent.scroll(jalur);
    expect([sebelumnya.disabled, berikutnya.disabled]).toEqual([false, false]);
    ukuran(jalur, { scrollWidth: 2000, clientWidth: 1000, scrollLeft: 1000 });
    fireEvent.scroll(jalur);
    expect([sebelumnya.disabled, berikutnya.disabled]).toEqual([false, true]);
  });

  test("filter yang muat tanpa gulir menonaktifkan kedua tombol", () => {
    const { container } = tampilan();
    const jalur = container.querySelector<HTMLElement>(".situs-slider__jalur");
    if (jalur === null) throw new Error("jalur tidak ada");
    ukuran(jalur, { scrollWidth: 700, clientWidth: 1000, scrollLeft: 0 });
    fireEvent.click(screen.getByRole("tab", { name: "Konsultasi" }));
    expect(screen.getByRole<HTMLButtonElement>("button", { name: "Layanan sebelumnya" }).disabled).toBe(true);
    expect(screen.getByRole<HTMLButtonElement>("button", { name: "Layanan berikutnya" }).disabled).toBe(true);
  });

  test("klik menggulir satu kartu ke kanan/kiri dengan gerak halus; prefers-reduced-motion: tanpa gerak halus", () => {
    const { container } = tampilan();
    const jalur = container.querySelector<HTMLElement>(".situs-slider__jalur");
    if (jalur === null) throw new Error("jalur tidak ada");
    ukuran(jalur, { scrollWidth: 2000, clientWidth: 1000, scrollLeft: 400 });
    fireEvent.scroll(jalur);
    vi.spyOn(HTMLElement.prototype, "getBoundingClientRect").mockReturnValue({ width: 336 } as DOMRect);
    vi.stubGlobal("matchMedia", () => ({ matches: false }));
    fireEvent.click(screen.getByRole("button", { name: "Layanan berikutnya" }));
    expect(scrollBy).toHaveBeenLastCalledWith({ left: 360, behavior: "smooth" });
    fireEvent.click(screen.getByRole("button", { name: "Layanan sebelumnya" }));
    expect(scrollBy).toHaveBeenLastCalledWith({ left: -360, behavior: "smooth" });
    vi.stubGlobal("matchMedia", () => ({ matches: true }));
    fireEvent.click(screen.getByRole("button", { name: "Layanan berikutnya" }));
    expect(scrollBy).toHaveBeenLastCalledWith({ left: 360, behavior: "auto" });
  });

  test("CSS: gulir halus hanya bila gerak tidak dikurangi; fokus terlihat memakai cincin fokus; tombol nonaktif diredam", () => {
    expect(css).toMatch(/@media \(prefers-reduced-motion: no-preference\) \{ \.situs-slider__jalur \{ scroll-behavior: smooth; \} \}/);
    expect(css.replace(/@media \(prefers-reduced-motion: no-preference\)[^\n]*/, "")).not.toMatch(/\.situs-slider__jalur \{[^}]*(?<!over)scroll-behavior/);
    expect(css).toMatch(/\.situs-slider__tab-satu:focus-visible, \.situs-slider__tombol:focus-visible, \.situs-slider__jalur:focus-visible \{ outline: none; box-shadow: var\(--focus-ring\); \}/);
    expect(css).toMatch(/\.situs-slider__tombol:disabled \{ opacity: 0\.4;/);
  });
});

describe("slot foto kartu (TD-69)", () => {
  test("kartu tanpa foto memakai panel ikon dekoratif; dengan foto lokal memakai <img> beralt dan tanpa panel ikon", () => {
    const [a, b] = kartu();
    if (a === undefined || b === undefined) throw new Error("kartu kosong");
    const { container } = render(<SliderLayanan kartu={[a, { ...b, gambar: { src: "/foto/konsultasi-psikolog.webp", alt: "Psikolog mendampingi sesi konseling online" }}]} kepala={<h2>Layanan</h2>} />);
    const [li1, li2] = [...container.querySelectorAll(".situs-slider__kartu")];
    expect(li1?.querySelector(".situs-ikon-lingkar")?.getAttribute("aria-hidden")).toBe("true");
    expect(li1?.querySelector("img")).toBeNull();
    expect(li2?.querySelector(".situs-ikon-lingkar")).toBeNull();
    const img = li2?.querySelector("img");
    expect(img?.getAttribute("src")).toBe("/foto/konsultasi-psikolog.webp");
    expect(img?.getAttribute("alt")).toBe("Psikolog mendampingi sesi konseling online");
  });
});

test("act dipakai agar efek hidrasi selesai sebelum asersi", () => {
  act(() => {
    tampilan();
  });
  expect(screen.getByRole("tablist")).toBeDefined();
});
