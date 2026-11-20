import { readdirSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, test } from "vitest";

const sini = dirname(fileURLToPath(import.meta.url));
const akar = join(sini, "..");

function berkas(dir: string): string[] {
  return readdirSync(dir, { withFileTypes: true }).flatMap((e) => {
    const p = join(dir, e.name);
    if (e.isDirectory()) return e.name === "node_modules" || e.name === ".next" ? [] : berkas(p);
    return [p];
  });
}

const kode = berkas(join(sini)).filter((p) => /\.(ts|tsx|css)$/.test(p) && !/\.test\.tsx?$/.test(p));

describe("batas keamanan situs (ADR-027 butir 1 dan 5)", () => {
  test("sumber ada dan dipindai", () => {
    expect(kode.length).toBeGreaterThan(10);
  });

  test("tanpa skrip inline, HTML mentah, atribut style inline, atau URL pihak ketiga di kode", () => {
    for (const p of kode) {
      const isi = readFileSync(p, "utf8");
      expect(isi, `${p}: dangerouslySetInnerHTML`).not.toContain("dangerouslySetInnerHTML");
      expect(isi, `${p}: <script`).not.toMatch(/<script/i);
      expect(isi, `${p}: style inline`).not.toMatch(/\sstyle=\{/);
      expect(isi, `${p}: URL`).not.toMatch(/https?:\/\/(?!localhost)/);
    }
  });

  test("tanpa cookie, OpenBao, Valkey, next/og, next/image, dan edge runtime", () => {
    const larangan = [/cookies\s*\(/, /openbao/i, /valkey/i, /next\/og/, /next\/image/, /runtime\s*=\s*["']edge/, /use cache/];
    for (const p of kode.filter((k) => !k.endsWith("keamanan.ts") && !k.endsWith("next.config.ts"))) {
      const isi = readFileSync(p, "utf8").replace(/\/\*[\s\S]*?\*\/|\/\/.*$/gm, "");
      for (const pola of larangan) expect(isi, `${p}: ${String(pola)}`).not.toMatch(pola);
    }
  });

  test("package.json: tanpa dependensi di luar daftar yang disetujui", () => {
    const pkg = JSON.parse(readFileSync(join(akar, "package.json"), "utf8")) as { dependencies: Record<string, string> };
    expect(Object.keys(pkg.dependencies).sort()).toEqual(["@rs/api-client", "@rs/ui", "next", "react", "react-dom"]);
    expect(pkg.dependencies["next"]).toBe("16.4.6");
  });
});
