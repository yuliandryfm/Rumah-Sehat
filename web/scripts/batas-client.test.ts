// CR-028: komponen yang memakai handler event atau hook state/efek wajib berdirektif "use client".
// Tanpa direktif, komponen itu berjalan sebagai server component dan Next.js melempar
// "Event handlers cannot be passed to Client Component props" (HTTP 500) di runtime. Test jsdom
// tidak menangkap kelas bug ini, maka pemeriksaan dilakukan statis atas sumber.
import { existsSync, readdirSync, readFileSync, statSync } from "node:fs";
import { dirname, join, relative } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

const akar = join(dirname(fileURLToPath(import.meta.url)), "..");

/** Semua `src` aplikasi (`web/<app>/src`) dan paket (`web/packages/<paket>/src`), ditemukan dinamis. */
function akarSumber(): string[] {
  const hasil: string[] = [];
  for (const induk of [akar, join(akar, "packages")]) {
    for (const nama of readdirSync(induk)) {
      const src = join(induk, nama, "src");
      if (nama !== "node_modules" && nama !== "packages" && existsSync(src) && statSync(src).isDirectory()) {
        hasil.push(src);
      }
    }
  }
  return hasil;
}

function kumpulkan(dir: string): string[] {
  return readdirSync(dir).flatMap((nama) => {
    const p = join(dir, nama);
    if (statSync(p).isDirectory()) {
      return nama === "node_modules" || nama === ".next" ? [] : kumpulkan(p);
    }
    return nama.endsWith(".tsx") && !/\.(test|spec)\.tsx$/.test(nama) ? [p] : [];
  });
}

/** Buang komentar agar contoh di dokumentasi tidak dihitung. */
function bersihkan(sumber: string): string {
  return sumber.replace(/\/\*[\s\S]*?\*\//g, "").replace(/(^|[^:])\/\/.*$/gm, "$1");
}

function butuhClient(sumber: string): boolean {
  const s = bersihkan(sumber);
  const handlerJsx = /\son[A-Z][A-Za-z]*\s*=\s*\{/.test(s);
  const hook =
    /\buse(State|Effect|LayoutEffect|Reducer|Ref|Transition|Optimistic|ActionState|FormStatus|Callback|Memo|SyncExternalStore|ImperativeHandle|Context)\s*\(/.test(
      s,
    );
  return handlerJsx || hook;
}

function punyaUseClient(sumber: string): boolean {
  return /^\s*(?:(?:\/\/[^\n]*|\/\*[\s\S]*?\*\/)\s*)*["']use client["']\s*;?/.test(sumber);
}

describe("batas client/server komponen web (CR-028)", () => {
  it("pendeteksi mengenali berkas yang butuh client dan direktifnya", () => {
    expect(butuhClient("export const A = () => <button onClick={f} />;")).toBe(true);
    expect(butuhClient("const [a, b] = useState(0);")).toBe(true);
    expect(butuhClient('export const A = () => <form method="post" />;')).toBe(false);
    expect(butuhClient("// contoh: <b onClick={f} />\nexport const A = 1;")).toBe(false);
    expect(punyaUseClient("\"use client\";\nimport x from 'y';")).toBe(true);
    expect(punyaUseClient("import type { X } from 'react';\n\"use client\";")).toBe(false);
  });

  const akarDitemukan = akarSumber();
  const berkas = akarDitemukan.flatMap((d) => kumpulkan(d));

  it("menemukan semua src yang diharapkan (gagal bila ada yang hilang)", () => {
    const rel = akarDitemukan.map((d) => relative(akar, d));
    for (const wajib of ["packages/ui/src", "korporat/src", "backoffice/src", "situs/src"]) {
      expect(rel).toContain(wajib);
    }
  });

  it("menemukan berkas komponen untuk diperiksa", () => {
    expect(berkas.length).toBeGreaterThan(10);
  });

  it('setiap komponen dengan handler event atau hook berdirektif "use client"', () => {
    const pelanggar = berkas
      .filter((f) => {
        const sumber = readFileSync(f, "utf8");
        return butuhClient(sumber) && !punyaUseClient(sumber);
      })
      .map((f) => relative(akar, f));
    expect(pelanggar).toEqual([]);
  });
});
