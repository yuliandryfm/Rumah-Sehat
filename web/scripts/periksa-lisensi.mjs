// Pemeriksa lisensi dependensi npm workspace web/ (TD-32; KEPUTUSAN-136, KEPUTUSAN-297, KEPUTUSAN-298;
// docs/coding-standards.md §9.1 butir 3). Tanpa dependensi: memakai `pnpm licenses list --json` bawaan pnpm
// (SRC-760), dijalankan per workspace karena `--prod`/`--dev` hanya berlaku untuk importer yang ditunjuk `--dir`.
// Runtime (dependencies/optionalDependencies): daftar izin. Dev (devDependencies): daftar izin + MPL-2.0.
// Lisensi Unknown atau kosong GAGAL. Ekspresi SPDX: OR lolos bila salah satu komponen diizinkan, AND wajib semuanya.
// Pengecualian sempit: LGPL-3.0-or-later hanya untuk paket bernama @img/sharp-libvips-*.
// Pemakaian: node scripts/periksa-lisensi.mjs (dari web/); keluar 1 bila ada pelanggaran.

import { execFileSync } from "node:child_process";
import { existsSync, readdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

export const LISENSI_RUNTIME = [
  "MIT",
  "Apache-2.0",
  "BSD-3-Clause",
  "BSD-2-Clause",
  "ISC",
  "0BSD",
  "CC-BY-4.0",
  "BlueOak-1.0.0",
];
export const LISENSI_TAMBAHAN_DEV = ["MPL-2.0", "Python-2.0"]; // KEPUTUSAN-298 (MPL-2.0), KEPUTUSAN-299 (Python-2.0): hanya devDependencies
export const PENGECUALIAN = [{ lisensi: "LGPL-3.0-or-later", nama: /^@img\/sharp-libvips-[a-z0-9-]+$/ }];

/**
 * Urai ekspresi SPDX (ID, AND, OR, tanda kurung; AND mengikat lebih kuat dari OR) menjadi pohon
 * { id } | { op: "AND"|"OR", anak: [...] }; null bila bentuknya tidak dikenali (mis. WITH).
 */
export function uraiLisensi(ekspresi) {
  if (typeof ekspresi !== "string") return null;
  const token = ekspresi.match(/\(|\)|[^\s()]+/g) ?? [];
  let i = 0;
  const ekspr = () => {
    const anak = [term()];
    while (token[i] === "OR") {
      i += 1;
      anak.push(term());
    }
    return anak.length === 1 ? anak[0] : { op: "OR", anak };
  };
  const term = () => {
    const anak = [faktor()];
    while (token[i] === "AND") {
      i += 1;
      anak.push(faktor());
    }
    return anak.length === 1 ? anak[0] : { op: "AND", anak };
  };
  const faktor = () => {
    const t = token[i];
    i += 1;
    if (t === "(") {
      const dalam = ekspr();
      if (token[i] !== ")") throw new Error("kurung tidak tertutup");
      i += 1;
      return dalam;
    }
    if (t === undefined || t === ")" || t === "AND" || t === "OR" || !/^[A-Za-z0-9.+-]+$/.test(t)) {
      throw new Error("token tidak dikenali");
    }
    return { id: t };
  };
  try {
    const pohon = ekspr();
    return i === token.length ? pohon : null;
  } catch {
    return null;
  }
}

/** Semua ID lisensi dalam ekspresi (untuk pesan galat); null bila tidak terurai. */
export function komponenLisensi(ekspresi) {
  const pohon = uraiLisensi(ekspresi);
  if (pohon === null) return null;
  const hasil = [];
  const kumpul = (n) => (n.id !== undefined ? hasil.push(n.id) : n.anak.forEach(kumpul));
  kumpul(pohon);
  return hasil;
}

// OR lolos bila SALAH SATU anak diizinkan (pilihan lisensi ada pada pengguna, sama dengan cargo-deny);
// AND wajib semua anak diizinkan.
function diizinkan(node, izin) {
  if (node.id !== undefined) return izin.has(node.id);
  return node.op === "OR" ? node.anak.some((a) => diizinkan(a, izin)) : node.anak.every((a) => diizinkan(a, izin));
}

/** Kembalikan daftar pelanggaran { nama, versi, lisensi, alasan } untuk satu daftar paket. */
export function periksaPaket(paket, mode) {
  const izin = new Set(mode === "dev" ? [...LISENSI_RUNTIME, ...LISENSI_TAMBAHAN_DEV] : LISENSI_RUNTIME);
  const pelanggaran = [];
  for (const p of paket) {
    const komponen = komponenLisensi(p.license);
    const tolak = (alasan) =>
      pelanggaran.push({ nama: p.name, versi: (p.versions ?? []).join(","), lisensi: String(p.license), alasan });
    if (komponen === null || p.license === "Unknown") {
      tolak("lisensi tidak diketahui, kosong, atau bentuknya tidak dikenali");
      continue;
    }
    if (PENGECUALIAN.some((e) => e.lisensi === p.license && e.nama.test(p.name))) continue;
    if (!diizinkan(uraiLisensi(p.license), izin)) {
      tolak(`lisensi di luar daftar izin ${mode}: ${komponen.filter((k) => !izin.has(k)).join(", ")}`);
    }
  }
  return pelanggaran;
}

/** Ratakan keluaran `pnpm licenses list --json` ({ "<lisensi>": [{ name, versions, ... }] }). */
export function ratakan(keluaran) {
  const hasil = [];
  for (const [lisensi, daftar] of Object.entries(keluaran)) {
    for (const p of daftar) hasil.push({ name: p.name, versions: p.versions, license: lisensi });
  }
  return hasil;
}

function daftarPnpm(webDir, dir, bendera) {
  const teks = execFileSync("pnpm", ["licenses", "list", "--json", bendera, "--dir", dir], {
    cwd: webDir,
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
  });
  return ratakan(JSON.parse(teks));
}

function daftarWorkspace(webDir) {
  const dirs = [".", "backoffice", "portal", "rme", "situs", "korporat"];
  const paketDir = join(webDir, "packages");
  if (existsSync(paketDir)) {
    for (const d of readdirSync(paketDir).sort()) dirs.push(`packages/${d}`);
  }
  return dirs.filter((d) => existsSync(join(webDir, d, "package.json")));
}

export function jalankan(webDir) {
  if (!existsSync(join(webDir, "node_modules"))) {
    console.error("web/node_modules belum terpasang; jalankan pnpm install --frozen-lockfile");
    return 1;
  }
  let diperiksa = 0;
  let gagal = 0;
  for (const dir of daftarWorkspace(webDir)) {
    for (const [mode, bendera] of [
      ["runtime", "--prod"],
      ["dev", "--dev"],
    ]) {
      const paket = daftarPnpm(webDir, dir, bendera);
      diperiksa += paket.length;
      for (const v of periksaPaket(paket, mode)) {
        gagal += 1;
        console.error(`LISENSI DITOLAK [${dir}, ${mode}] ${v.nama}@${v.versi}: ${v.lisensi} (${v.alasan})`);
      }
    }
  }
  if (diperiksa === 0) {
    console.error("tidak ada paket yang diperiksa (pnpm licenses mengembalikan kosong)");
    return 1;
  }
  console.log(`periksa-lisensi: ${diperiksa} entri paket diperiksa, ${gagal} pelanggaran`);
  return gagal === 0 ? 0 : 1;
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  process.exit(jalankan(join(dirname(fileURLToPath(import.meta.url)), "..")));
}
