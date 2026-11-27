import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, test } from "vitest";
import { bacaWarna, rasioKontras } from "../../packages/ui/scripts/kontras.mjs";

const sini = dirname(fileURLToPath(import.meta.url));
const tokens = bacaWarna(readFileSync(join(sini, "../../packages/ui/src/tokens.css"), "utf8"));
const situsCss = readFileSync(join(sini, "app/situs.css"), "utf8");

/** Setiap pasangan teks/latar yang dipakai situs.css; ambang WCAG AA teks biasa 4,5:1 (DG §3.2, tanpa pengecualian teks besar). */
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
  // CR-033 (konsep A + seksi gelap): pasangan baru atau pemakaian baru.
  ["tombol utama situs dan CTA navigasi", "on-primary", "primary"],
  ["tombol sekunder/terang dan eyebrow di latar putih", "primary-strong", "bg-surface"],
  ["eyebrow dan judul di gradien hero (ujung bg.app)", "primary-strong", "bg-app"],
  ["judul hero di gradien (ujung bg.app)", "text-primary", "bg-app"],
  ["judul hero di sorot putih gradien", "text-primary", "bg-surface"],
  ["kalimat pembuka hero (ujung bg.app)", "text-secondary", "bg-app"],
  ["kalimat pembuka hero (ujung mint)", "text-secondary", "mint"],
  ["kalimat pembuka hero (sorot putih)", "text-secondary", "bg-surface"],
  ["chip nilai, kartu mengambang, judul kartu", "text-primary", "bg-surface"],
  ["chip kategori, hover navigasi dan tombol sekunder", "primary-strong", "mint"],
  ["teks di seksi gelap dan kartu korporat", "on-primary", "primary-strong"],
  ["eyebrow/kalimat pembuka di seksi gelap, ringkasan kartu korporat", "mint", "primary-strong"],
  ["judul kolom dan teks redup footer", "mint", "text-primary"],
  ["butir pita isi Beranda", "text-primary", "bg-app"],
  ["kutipan Markdown", "text-primary", "mint"],
  ["spanduk draf", "text-primary", "warning-bg"],
];

/** Elemen non-teks yang diperlukan untuk memahami isi (ikon kartu, tanda cek, cincin fokus): ≥ 3:1. */
const PASANGAN_NON_TEKS: [string, string, string][] = [
  ["ikon dalam lingkaran mint, tanda cek pita", "primary-strong", "mint"],
  ["ikon terbalik saat hover kartu", "on-primary", "primary-strong"],
  ["batas tombol menu", "border-control", "bg-surface"],
  ["cincin fokus di seksi gelap (cincin putih)", "on-primary", "primary-strong"],
  ["cincin fokus kartu korporat (cincin luar) di latar halaman", "primary-strong", "bg-app"],
];

/** Campuran sRGB `atas` dengan opasitas `alfa` di atas `bawah` (pembulatan per kanal, seperti peramban). */
function campur(atas: string, alfa: number, bawah: string): string {
  const kanal = (hex: string, i: number) => Number.parseInt(hex.slice(1 + i * 2, 3 + i * 2), 16);
  const hasil = [0, 1, 2].map((i) => Math.round(kanal(atas, i) * alfa + kanal(bawah, i) * (1 - alfa)));
  return `#${hasil.map((c) => c.toString(16).padStart(2, "0")).join("")}`;
}

function warna(nama: string): string {
  const w = tokens.get(nama);
  expect(w, nama).toBeDefined();
  return w ?? "";
}

describe("kontras situs (token)", () => {
  test.each(PASANGAN_TEKS)("%s: %s di atas %s ≥ 4,5:1", (_nama, fg, bg) => {
    expect(rasioKontras(warna(fg), warna(bg))).toBeGreaterThanOrEqual(4.5);
  });

  test.each(PASANGAN_NON_TEKS)("non-teks %s: %s di atas %s ≥ 3:1", (_nama, fg, bg) => {
    expect(rasioKontras(warna(fg), warna(bg))).toBeGreaterThanOrEqual(3);
  });

  test("kartu kaca seksi gelap: lapisan putih 8% di atas primary-strong; teks putih dan mint ≥ 4,5:1", () => {
    // Harus sama dengan `.situs-kaca::before { opacity: 0.08 }` di situs.css.
    expect(situsCss).toMatch(/\.situs-kaca::before \{[^}]*background: var\(--c-on-primary\); opacity: 0\.08;/);
    const latar = campur(warna("on-primary"), 0.08, warna("primary-strong"));
    expect(latar).toBe("#1a6c66");
    expect(rasioKontras(warna("on-primary"), latar)).toBeGreaterThanOrEqual(4.5);
    expect(rasioKontras(warna("mint"), latar)).toBeGreaterThanOrEqual(4.5);
  });

  test("ornamen seksi gelap: cincin mint 16% di atas primary-strong tetap menjaga teks putih ≥ 4,5:1", () => {
    expect(situsCss).toMatch(/\.situs-orn__cincin \{[^}]*opacity: 0\.16;/);
    const latar = campur(warna("mint"), 0.16, warna("primary-strong"));
    expect(rasioKontras(warna("on-primary"), latar)).toBeGreaterThanOrEqual(4.5);
  });

  test("ornamen terang: daun primary 12% di atas mint tetap menjaga teks utama ≥ 4,5:1", () => {
    expect(situsCss).toMatch(/\.situs-orn__daun \{[^}]*opacity: 0\.12;/);
    const latar = campur(warna("primary"), 0.12, warna("mint"));
    expect(rasioKontras(warna("text-primary"), latar)).toBeGreaterThanOrEqual(4.5);
    expect(rasioKontras(warna("primary-strong"), latar)).toBeGreaterThanOrEqual(4.5);
  });

  test("pita judul terang: semua warna teks di atas latar terburuk (daun primary 12% di atas mint; seluler tanpa ornamen = mint)", () => {
    // Warna teks pita judul terang: judul/lead/keterangan text-primary, eyebrow/tautan/chip primary-strong.
    expect(situsCss).toMatch(/\.situs-halhero \.situs-lead, \.situs-halhero \.situs-lead p, \.situs-halhero \.situs-cap \{ color: var\(--c-text-primary\); \}/);
    expect(situsCss).toMatch(/@media \(max-width: 1023px\) \{[^}]*\}?[\s\S]*?\.situs-halhero \.situs-orn \{ display: none; \}/);
    const desktop = campur(warna("primary"), 0.12, warna("mint"));
    expect(desktop).toBe("#c5e6df");
    for (const latar of [desktop, warna("mint"), warna("bg-app")]) {
      for (const teks of ["text-primary", "primary-strong"]) expect(rasioKontras(warna(teks), latar), `${teks} / ${latar}`).toBeGreaterThanOrEqual(4.5);
    }
    // Penyebab temuan review: text-secondary di atas daun gagal (< 4,5), sehingga tidak boleh dipakai di pita judul.
    expect(rasioKontras(warna("text-secondary"), desktop)).toBeLessThan(4.5);
  });

  test("pita judul gelap: putih dan mint di atas latar terburuk (cincin mint 16% lalu daun primary 12% di atas primary-strong)", () => {
    const terburuk = campur(warna("primary"), 0.12, campur(warna("mint"), 0.16, warna("primary-strong")));
    for (const latar of [terburuk, warna("primary-strong")]) {
      for (const teks of ["on-primary", "mint"]) expect(rasioKontras(warna(teks), latar), `${teks} / ${latar}`).toBeGreaterThanOrEqual(4.5);
    }
  });

  test("larangan DG §3.2 K4 tetap: primary tidak dipakai sebagai warna teks di atas mint (rasio < 4,5)", () => {
    expect(rasioKontras(warna("primary"), warna("mint"))).toBeLessThan(4.5);
  });

  test("situs.css hanya memakai token: tanpa warna harfiah, tanpa URL jarak jauh, tanpa @import", () => {
    expect(situsCss).not.toMatch(/#[0-9a-fA-F]{3,8}\b|rgb\(|hsl\(/);
    expect(situsCss).not.toMatch(/url\(|@import|https?:\/\//);
  });

  test("header ponsel dua baris (CR-038, KEPUTUSAN-421): pill ringkas, awalan disembunyikan, padding gulir, scroll-padding selaras", () => {
    const ponsel = /@media \(max-width: 767px\) \{([\s\S]*?)\n\}/.exec(situsCss)?.[1] ?? "";
    expect(ponsel).toMatch(/\.situs-toko__awal \{ display: none; \}/);
    expect(ponsel).toMatch(/\.situs-logo \{ font-size: 18px; \}/);
    expect(ponsel).toMatch(/\.situs-toko__tautan \{ padding: 0 var\(--s3\); font-size: 13px; \}/);
    const sempit = /@media \(max-width: 1139px\) \{([\s\S]*?)\n\}/.exec(situsCss)?.[1] ?? "";
    expect(sempit).toMatch(/scroll-padding-top: calc\(var\(--situs-tinggi-kepala\) \+ 48px \+ var\(--s4\)\)/);
    expect(sempit).toMatch(/\.situs-nav-baris \{[^}]*scroll-padding-inline: 12px 28px;/);
    expect(sempit).toMatch(/flex: 0 0 100%/);
  });

  test("setiap var(--c-*) yang dipakai ada di tokens.css", () => {
    const dipakai = new Set([...situsCss.matchAll(/var\(--(c-[a-z-]+)/g)].map((m) => (m[1] ?? "").replace(/^c-/, "")));
    expect(dipakai.size).toBeGreaterThan(5);
    for (const nama of dipakai) expect(tokens.has(nama), nama).toBe(true);
  });

  test("setiap var(--situs-*) yang dipakai dideklarasikan di :root situs.css", () => {
    const dideklarasi = new Set([...situsCss.matchAll(/(--situs-[a-z0-9-]+)\s*:/g)].map((m) => m[1]));
    for (const m of situsCss.matchAll(/var\((--situs-[a-z0-9-]+)\)/g)) expect(dideklarasi.has(m[1]), m[1]).toBe(true);
  });

  test("gerak menghormati prefers-reduced-motion (animasi dan pergeseran dimatikan)", () => {
    expect(situsCss).toMatch(/@media \(prefers-reduced-motion: reduce\) \{[\s\S]*animation: none !important/);
    expect(situsCss).toMatch(/@keyframes situs-muncul/);
  });
});
