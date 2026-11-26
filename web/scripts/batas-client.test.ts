// CR-028: komponen yang memakai handler event atau hook state/efek wajib berdirektif "use client".
// Tanpa direktif, komponen itu berjalan sebagai server component dan Next.js melempar
// "Event handlers cannot be passed to Client Component props" (HTTP 500) di runtime. Test jsdom
// tidak menangkap kelas bug ini, maka pemeriksaan dilakukan statis atas sumber.
import { readdirSync, readFileSync, statSync } from "node:fs";
import { dirname, join, relative } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";

const akar = join(dirname(fileURLToPath(import.meta.url)), "..");
const AKAR_SUMBER = ["packages/ui/src", "korporat/src", "backoffice/src", "portal/src", "rme/src"];

function kumpulkan(dir: string): string[] {
  let daftar: string[];
  try {
    daftar = readdirSync(dir);
  } catch {
    return [];
  }
  return daftar.flatMap((nama) => {
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

  const berkas = AKAR_SUMBER.flatMap((d) => kumpulkan(join(akar, d)));

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
