import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { cleanup, render } from "@testing-library/react";
import { afterEach, describe, expect, test } from "vitest";
import { Galeri } from "../gallery/Galeri.tsx";
import { STATE_KOMPONEN, type KomponenName, type StateName } from "./states.ts";

afterEach(cleanup);

const folder = dirname(fileURLToPath(import.meta.url));
const uiCss = readFileSync(join(folder, "ui.css"), "utf8");
const galeriHtml = readFileSync(join(folder, "..", "gallery", "index.html"), "utf8");

const PREFIKS: Record<KomponenName, string> = {
  button: ".rs-btn",
  textfield: ".rs-field__input",
  checkbox: ".rs-check__input",
  card: ".rs-card--pressable",
  badge: ".rs-badge",
  alert: ".rs-alert",
  skeleton: ".rs-skeleton",
};

describe("halaman galeri", () => {
  test("setiap state di STATE_KOMPONEN tampil di galeri (hover dan memuat termasuk)", () => {
    const { container } = render(<Galeri />);
    const hilang: string[] = [];
    for (const [komponen, daftar] of Object.entries(STATE_KOMPONEN)) {
      for (const state of daftar as readonly StateName[]) {
        if (container.querySelector(`[data-galeri="${komponen}:${state}"]`) === null) {
          hilang.push(`${komponen}:${state}`);
        }
      }
    }
    expect(hilang).toEqual([]);
  });

  test("galeri tidak menampilkan state di luar matriks (tidak ada sel yatim)", () => {
    const { container } = render(<Galeri />);
    const sel = Array.from(container.querySelectorAll("[data-galeri]")).map((e) => e.getAttribute("data-galeri") ?? "");
    for (const kunci of sel) {
      const [komponen = "", state = ""] = kunci.split(":");
      const daftar: readonly string[] | undefined = (STATE_KOMPONEN as Record<string, readonly string[]>)[komponen];
      expect(daftar?.includes(state), kunci).toBe(true);
    }
  });

  test("state memuat tombol dan kartu benar-benar dalam keadaan memuat", () => {
    const { container } = render(<Galeri />);
    const tombol = container.querySelector('[data-galeri="button:memuat"] button');
    expect(tombol?.getAttribute("aria-busy")).toBe("true");
    const kartu = container.querySelector('[data-galeri="card:memuat"] [aria-busy="true"]');
    expect(kartu).not.toBeNull();
  });

  test("state nonaktif dan galat benar-benar aktif pada komponennya", () => {
    const { container } = render(<Galeri />);
    expect(container.querySelector('[data-galeri="button:nonaktif"] button:disabled')).not.toBeNull();
    expect(container.querySelector('[data-galeri="textfield:galat"] input[aria-invalid="true"]')).not.toBeNull();
    expect(container.querySelector('[data-galeri="checkbox:galat"] input[aria-invalid="true"]')).not.toBeNull();
    expect(container.querySelector('[data-galeri="checkbox:default"] input:checked')).toBeNull();
  });

  test("ui.css punya aturan untuk setiap state hover, fokus, nonaktif, dan galat di matriks", () => {
    const kurang: string[] = [];
    for (const [komponen, daftar] of Object.entries(STATE_KOMPONEN)) {
      const awalan = PREFIKS[komponen as KomponenName];
      for (const state of daftar as readonly StateName[]) {
        const pola: Record<StateName, RegExp | null> = {
          default: null,
          hover: new RegExp(`${awalan.replace(".", "\\.")}[^{]*(:hover|\\[data-state="hover"\\])`),
          fokus: new RegExp(`${awalan.replace(".", "\\.")}[^{]*(:focus-visible|\\[data-state="fokus"\\])`),
          nonaktif: new RegExp(`${awalan.replace(".", "\\.")}[^{]*:disabled`),
          galat: /aria-invalid="true"/,
          memuat: /data-loading="true"|\.rs-skeleton/,
        };
        const re = pola[state];
        if (re !== null && !re.test(uiCss)) kurang.push(`${komponen}:${state}`);
      }
    }
    expect(kurang).toEqual([]);
  });

  test("ui.css hanya memakai warna dari token (tanpa heksadesimal literal)", () => {
    const tanpaKomentar = uiCss.replace(/\/\*[\s\S]*?\*\//g, "");
    expect(tanpaKomentar).not.toMatch(/#[0-9a-fA-F]{3,8}\b/);
  });

  test("teks di atas mint tidak memakai warna primary (K4): hanya primary-strong atau teks lain", () => {
    const tanpaKomentar = uiCss.replace(/\/\*[\s\S]*?\*\//g, "");
    // Setiap aturan yang bergantung pada latar mint (badge info/sukses, alert info/sukses, kartu mint,
    // hover secondary/teks, hover checkbox) tidak boleh menetapkan `color: var(--c-primary)`.
    expect(tanpaKomentar).not.toMatch(/color:\s*var\(--c-primary\)/);
  });

  test("index.html galeri: bahasa Indonesia, tanpa sumber daya jarak jauh", () => {
    expect(galeriHtml).toContain('lang="id"');
    expect(galeriHtml).not.toMatch(/https?:\/\//);
  });
});
