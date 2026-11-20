import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, test } from "vitest";
import { PASANGAN, bacaWarna, periksaKontras, rasioKontras } from "../scripts/kontras.mjs";

const tokensCss = readFileSync(join(dirname(fileURLToPath(import.meta.url)), "tokens.css"), "utf8");

/** Nilai warna menurut docs/06-design-guideline.md §3.1. Perubahan token harus mengubah dokumen juga. */
const WARNA_GUIDELINE: Record<string, string> = {
  primary: "#087f78",
  mint: "#dff4ed",
  "bg-app": "#fafbf7",
  "bg-surface": "#ffffff",
  "primary-strong": "#065f59",
  "text-primary": "#1a2b2e",
  "text-secondary": "#5c6b6f",
  "border-control": "#7a898c",
  "border-divider": "#dce3e1",
  "bg-neutral": "#eef1f0",
  danger: "#b3261e",
  "danger-bg": "#fdecea",
  success: "#1e7a3e",
  warning: "#8a5300",
  "warning-bg": "#fff4db",
  "on-primary": "#ffffff",
};

describe("tokens.css", () => {
  test("warna sama dengan design guideline §3.1 → tidak ada drift nilai", () => {
    expect(Object.fromEntries(bacaWarna(tokensCss))).toEqual(WARNA_GUIDELINE);
  });

  test("teks di atas mint memakai primary-strong dan token ruang bernama kelipatan 4", () => {
    expect(tokensCss).toContain("--s4: 16px;");
    expect(tokensCss).toContain("--s16: 64px;");
    expect(tokensCss).toContain("--r-xl: 16px;");
  });

  test("tidak memuat sumber daya jarak jauh (tanpa CDN atau font eksternal)", () => {
    expect(tokensCss).not.toMatch(/url\(|@import|https?:\/\//);
  });
});

describe("rasioKontras", () => {
  test("contoh guideline §3.2: #087F78 di atas putih → 4,86:1", () => {
    expect(rasioKontras("#087f78", "#ffffff")).toBeCloseTo(4.86, 2);
  });

  test("hitam di atas putih → 21:1 dan simetris", () => {
    expect(rasioKontras("#000000", "#ffffff")).toBeCloseTo(21, 5);
    expect(rasioKontras("#ffffff", "#000000")).toBeCloseTo(21, 5);
  });

  test("warna tidak valid → melempar galat", () => {
    expect(() => rasioKontras("biru", "#ffffff")).toThrow("warna tidak valid");
  });
});

describe("periksaKontras", () => {
  const hasil = periksaKontras(tokensCss);
  const ambil = (id: string, bg: string) => {
    const baris = hasil.find((h) => h.id === id && h.bg === bg);
    if (baris === undefined) throw new Error(`baris ${id}/${bg} tidak ada`);
    return baris;
  };

  test("semua pasangan K1–K13 dan N1–N4 lulus dengan token saat ini", () => {
    const ids = new Set(hasil.map((h) => h.id));
    for (const id of [...Array.from({ length: 13 }, (_, i) => `K${String(i + 1)}`), "N1", "N2", "N3", "N4"]) {
      expect(ids.has(id), id).toBe(true);
    }
    expect(hasil.filter((h) => !h.lulus)).toEqual([]);
  });

  test("angka mengikuti tabel guideline §3.2 (toleransi pembulatan 0,01)", () => {
    expect(ambil("K3", "bg-surface").rasio).toBeCloseTo(4.86, 1);
    expect(ambil("K6", "mint").rasio).toBeCloseTo(6.56, 1);
    expect(ambil("K7", "primary-strong").rasio).toBeCloseTo(7.53, 1);
    expect(ambil("K10", "mint").rasio).toBeCloseTo(4.68, 1);
    expect(ambil("N1", "mint").rasio).toBeCloseTo(3.16, 1);
  });

  test("K4: primary sebagai teks di atas mint tetap di bawah 4,5 (dasar aturan primary-strong)", () => {
    const k4 = ambil("K4", "mint");
    expect(k4.rasio).toBeLessThan(4.5);
    expect(k4.rasio).toBeCloseTo(4.24, 1);
    expect(k4.jenis).toBe("larangan-teks");
  });

  test("negatif: teks abu-abu muda di atas putih → gagal", () => {
    const css = tokensCss.replace("--c-text-secondary: #5c6b6f;", "--c-text-secondary: #aaaaaa;");
    const gagal = periksaKontras(css).filter((h) => !h.lulus);
    expect(gagal.length).toBeGreaterThan(0);
    expect(gagal.every((h) => h.fg === "text-secondary")).toBe(true);
  });

  test("negatif: batas kontrol terlalu pucat (< 3:1) → N1 gagal", () => {
    const css = tokensCss.replace("--c-border-control: #7a898c;", "--c-border-control: #cccccc;");
    expect(periksaKontras(css).some((h) => h.id === "N1" && !h.lulus)).toBe(true);
  });

  test("negatif: bila primary di atas mint mencapai 4,5 (primary digelapkan), K4 ditandai gagal agar aturan ditinjau", () => {
    const css = tokensCss.replace("--c-primary: #087f78;", "--c-primary: #05524d;");
    expect(periksaKontras(css).some((h) => h.id === "K4" && !h.lulus)).toBe(true);
  });

  test("negatif: token hilang → baris gagal, bukan lulus diam-diam", () => {
    const css = tokensCss.replace(/--c-danger-bg:[^;]*;/, "");
    const gagal = periksaKontras(css).filter((h) => !h.lulus);
    expect(gagal.some((h) => h.catatan === "token tidak ada")).toBe(true);
  });

  test("daftar pasangan tidak kosong dan semua id unik per kelompok", () => {
    expect(PASANGAN.length).toBe(17);
    expect(new Set(PASANGAN.map((p) => p.id)).size).toBe(PASANGAN.length);
  });
});
