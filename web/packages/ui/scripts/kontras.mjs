// Skrip kontras token tanpa dependensi (docs/06-design-guideline.md §3.2 dan §7.4, KEPUTUSAN-176).
// Membaca src/tokens.css, menghitung ulang pasangan K1–K13 dan N1–N4 dengan rumus WCAG 2.2
// (relative luminance, §3.2), dan gagal (kode keluar 1) bila ada yang di bawah syarat.
// K4 (primary sebagai teks di atas mint, 4,24:1) sengaja gagal untuk teks: ia dicatat sebagai
// larangan dan skrip memastikan nilainya memang di bawah 4,5, supaya aturan "teks di atas mint
// memakai primary-strong" tidak diam-diam kehilangan dasarnya bila token berubah.
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const SYARAT_TEKS = 4.5;
const SYARAT_NON_TEKS = 3;

/**
 * Pasangan yang diperiksa. `fg` dan `bg` memakai nama token tanpa awalan `--c-`.
 * `jenis`: "teks" (≥ 4,5), "non-teks" (≥ 3), atau "larangan-teks" (harus < 4,5; hanya dicatat).
 */
export const PASANGAN = [
  { id: "K1", fg: "text-primary", bg: ["bg-surface", "bg-app", "mint"], jenis: "teks" },
  { id: "K2", fg: "text-secondary", bg: ["bg-surface", "bg-app", "mint", "bg-neutral"], jenis: "teks" },
  { id: "K3", fg: "primary", bg: ["bg-surface", "bg-app"], jenis: "teks" },
  { id: "K4", fg: "primary", bg: ["mint"], jenis: "larangan-teks" },
  { id: "K5", fg: "on-primary", bg: ["primary"], jenis: "teks" },
  { id: "K6", fg: "primary-strong", bg: ["bg-surface", "bg-app", "mint"], jenis: "teks" },
  { id: "K7", fg: "on-primary", bg: ["primary-strong"], jenis: "teks" },
  { id: "K8", fg: "danger", bg: ["bg-surface", "bg-app", "danger-bg"], jenis: "teks" },
  { id: "K9", fg: "on-primary", bg: ["danger"], jenis: "teks" },
  { id: "K10", fg: "success", bg: ["bg-surface", "bg-app", "mint"], jenis: "teks" },
  { id: "K11", fg: "warning", bg: ["bg-surface", "warning-bg"], jenis: "teks" },
  { id: "K12", fg: "text-primary", bg: ["danger-bg", "warning-bg"], jenis: "teks" },
  { id: "K13", fg: "on-primary", bg: ["text-primary"], jenis: "teks" },
  { id: "N1", fg: "border-control", bg: ["bg-surface", "bg-app", "mint"], jenis: "non-teks" },
  { id: "N2", fg: "primary", bg: ["bg-surface", "bg-app"], jenis: "non-teks" },
  { id: "N3", fg: "primary-strong", bg: ["bg-surface", "bg-app", "mint"], jenis: "non-teks" },
  { id: "N4", fg: "primary", bg: ["mint"], jenis: "non-teks" },
];

/** Ambil semua `--c-nama: #RRGGBB;` dari teks CSS. Mengembalikan peta nama → "#rrggbb" huruf kecil. */
export function bacaWarna(css) {
  const tanpaKomentar = css.replace(/\/\*[\s\S]*?\*\//g, "");
  const peta = new Map();
  for (const m of tanpaKomentar.matchAll(/--c-([a-z0-9-]+)\s*:\s*(#[0-9a-fA-F]{6})\s*;/g)) {
    const nama = m[1];
    const nilai = m[2];
    if (nama === undefined || nilai === undefined) continue;
    if (peta.has(nama)) throw new Error(`token warna ganda: --c-${nama}`);
    peta.set(nama, nilai.toLowerCase());
  }
  return peta;
}

function linear(kanal8bit) {
  const c = kanal8bit / 255;
  return c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4;
}

/** Relative luminance WCAG 2.2 untuk "#rrggbb". */
export function luminance(hex) {
  if (!/^#[0-9a-f]{6}$/.test(hex)) throw new Error(`warna tidak valid: ${hex}`);
  const r = linear(Number.parseInt(hex.slice(1, 3), 16));
  const g = linear(Number.parseInt(hex.slice(3, 5), 16));
  const b = linear(Number.parseInt(hex.slice(5, 7), 16));
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

/** Rasio kontras (L1 + 0,05) / (L2 + 0,05) dengan L1 yang lebih terang. */
export function rasioKontras(hexA, hexB) {
  const a = luminance(hexA);
  const b = luminance(hexB);
  const [terang, gelap] = a >= b ? [a, b] : [b, a];
  return (terang + 0.05) / (gelap + 0.05);
}

/**
 * Periksa semua pasangan terhadap warna dalam `css`. Mengembalikan baris hasil;
 * token yang hilang menjadi baris gagal (bukan pengecualian) supaya laporan lengkap.
 */
export function periksaKontras(css, pasangan = PASANGAN) {
  const warna = bacaWarna(css);
  const hasil = [];
  for (const p of pasangan) {
    for (const bg of p.bg) {
      const fgHex = warna.get(p.fg);
      const bgHex = warna.get(bg);
      const syarat = p.jenis === "non-teks" ? SYARAT_NON_TEKS : SYARAT_TEKS;
      if (fgHex === undefined || bgHex === undefined) {
        hasil.push({ id: p.id, fg: p.fg, bg, rasio: Number.NaN, syarat, jenis: p.jenis, lulus: false, catatan: "token tidak ada" });
        continue;
      }
      const rasio = rasioKontras(fgHex, bgHex);
      const lulus = p.jenis === "larangan-teks" ? rasio < SYARAT_TEKS : rasio >= syarat;
      hasil.push({ id: p.id, fg: p.fg, bg, rasio, syarat, jenis: p.jenis, lulus, catatan: "" });
    }
  }
  return hasil;
}

function cetak(hasil) {
  for (const h of hasil) {
    const rasio = Number.isNaN(h.rasio) ? "?" : h.rasio.toFixed(2);
    const syarat = h.jenis === "larangan-teks" ? `<${SYARAT_TEKS} (dilarang untuk teks)` : `>=${h.syarat}`;
    console.log(`${h.lulus ? "LULUS" : "GAGAL"}  ${h.id.padEnd(3)} ${h.fg} / ${h.bg}: ${rasio}:1 (syarat ${syarat}) ${h.catatan}`.trimEnd());
  }
}

function utama() {
  const css = readFileSync(join(dirname(fileURLToPath(import.meta.url)), "..", "src", "tokens.css"), "utf8");
  const hasil = periksaKontras(css);
  cetak(hasil);
  const gagal = hasil.filter((h) => !h.lulus).length;
  console.log(`kontras: ${hasil.length - gagal} lulus, ${gagal} gagal dari ${hasil.length} pasangan`);
  process.exit(gagal === 0 && hasil.length > 0 ? 0 : 1);
}

if (process.argv[1] !== undefined && fileURLToPath(import.meta.url) === process.argv[1]) {
  utama();
}
