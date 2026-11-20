// Batas klien–server: modul `@rs/bff-auth/client` (yang masuk bundel JavaScript browser) tidak boleh
// mengimpor modul server paket ini, pustaka OIDC/JWE/Valkey, atau modul Node. Dengan begitu kode yang
// memegang token tidak dapat ikut terbundel ke browser (ADR-011 butir 7: token tidak pernah ke browser).

import { readFileSync, readdirSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { describe, expect, test } from "vitest";

const DIR = fileURLToPath(new URL(".", import.meta.url));

function sumberKlien(): { file: string; text: string }[] {
  return readdirSync(DIR)
    .filter((f) => /\.(ts|tsx)$/.test(f) && !/\.test\.tsx?$/.test(f))
    .map((f) => ({ file: f, text: readFileSync(`${DIR}${f}`, "utf8") }));
}

describe("@rs/bff-auth/client", () => {
  test("hanya mengimpor react, @rs/ui, dan berkas di folder client", () => {
    const files = sumberKlien();
    expect(files.map((f) => f.file).sort()).toEqual(["SessionTimeoutWarning.tsx", "index.ts"]);
    for (const { file, text } of files) {
      const specs = [...text.matchAll(/(?:import|export)\s[^;]*?from\s+"([^"]+)"/g)].map((m) => m[1]);
      for (const s of specs) {
        expect(
          s === "react" || s === "@rs/ui" || (s?.startsWith("./") === true && !s.includes("..")),
          `${file} mengimpor ${String(s)}`,
        ).toBe(true);
      }
      expect(text).not.toMatch(/import\(/);
      expect(text).not.toMatch(/\brequire\(/);
    }
  });

  test("komponen klien ditandai 'use client' dan tidak menyentuh document.cookie atau penyimpanan browser", () => {
    const comp = readFileSync(`${DIR}SessionTimeoutWarning.tsx`, "utf8");
    expect(comp.startsWith('"use client";')).toBe(true);
    expect(comp).not.toMatch(/document\.cookie|localStorage|sessionStorage|indexedDB/);
    expect(comp).not.toMatch(/access_token|refresh_token|id_token/);
  });
});
