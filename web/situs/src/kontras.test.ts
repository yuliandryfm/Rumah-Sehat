import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, test } from "vitest";
import { bacaWarna, rasioKontras } from "../../packages/ui/scripts/kontras.mjs";

const sini = dirname(fileURLToPath(import.meta.url));
const tokens = bacaWarna(readFileSync(join(sini, "../../packages/ui/src/tokens.css"), "utf8"));
const situsCss = readFileSync(join(sini, "app/situs.css"), "utf8");

/** Setiap pasangan teks/latar yang dipakai situs.css; ambang WCAG AA teks biasa 4,5:1. */
const PASANGAN_TEKS: [string, string, string][] = [
  ["badan teks", "text-primary", "bg-app"],
  ["tautan dan navigasi aktif", "primary-strong", "bg-surface"],
  ["tautan di hero/halaman mint", "primary-strong", "mint"],
  ["teks di hero (mint)", "text-primary", "mint"],
  ["tombol ajakan navigasi", "on-primary", "primary-strong"],
  ["footer", "on-primary", "text-primary"],
  ["keterangan dan placeholder gambar", "text-secondary", "bg-neutral"],
  ["tanggal/keterangan di kartu", "text-secondary", "bg-surface"],
  ["teks status 404/galat", "text-secondary", "bg-app"],
  ["penanda TIDAK DIKETAHUI", "warning", "warning-bg"],
];

describe("kontras situs (token)", () => {
  test.each(PASANGAN_TEKS)("%s: %s di atas %s ≥ 4,5:1", (_nama, fg, bg) => {
    const a = tokens.get(fg);
    const b = tokens.get(bg);
    expect(a, fg).toBeDefined();
    expect(b, bg).toBeDefined();
    expect(rasioKontras(a ?? "", b ?? "")).toBeGreaterThanOrEqual(4.5);
  });

  test("situs.css hanya memakai token: tanpa warna harfiah, tanpa URL jarak jauh, tanpa @import", () => {
    expect(situsCss).not.toMatch(/#[0-9a-fA-F]{3,8}\b|rgb\(|hsl\(/);
    expect(situsCss).not.toMatch(/url\(|@import|https?:\/\//);
  });

  test("setiap var(--c-*) yang dipakai ada di tokens.css", () => {
    const dipakai = new Set([...situsCss.matchAll(/var\(--(c-[a-z-]+)/g)].map((m) => (m[1] ?? "").replace(/^c-/, "")));
    expect(dipakai.size).toBeGreaterThan(5);
    for (const nama of dipakai) expect(tokens.has(nama), nama).toBe(true);
  });
});
