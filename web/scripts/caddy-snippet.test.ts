// Test teks snippet Caddy pendaftaran publik korporat (F1-T103, ADR-031, review T1 B1/B4). Dijalankan tanpa
// Docker dan tanpa biner caddy (verify/CI). Validasi sintaks dengan `caddy adapt` (image caddy:2.11.9) dilakukan
// manual dan dicatat di ADR-031; test ini menjaga invarian keamanannya agar tidak hilang saat berkas diubah.
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, test } from "vitest";

const berkas = join(dirname(fileURLToPath(import.meta.url)), "..", "..", "infra", "caddy", "snippets", "korporat-pendaftaran-publik.caddy");
/** Isi tanpa komentar dan baris kosong. */
const isi = readFileSync(berkas, "utf8")
  .split("\n")
  .map((b) => b.replace(/#.*$/, "").trim())
  .filter((b) => b !== "");

/** Isi blok `handle @nama { ... }` (satu tingkat kurung kurawal bersarang). */
function blokHandle(nama: string): string[] {
  const awal = isi.indexOf(`handle @${nama} {`);
  expect(awal, `handle @${nama} ada`).toBeGreaterThanOrEqual(0);
  const hasil: string[] = [];
  let kedalaman = 1;
  for (const baris of isi.slice(awal + 1)) {
    if (baris.endsWith("{")) kedalaman += 1;
    if (baris === "}") kedalaman -= 1;
    if (kedalaman === 0) break;
    hasil.push(baris);
  }
  return hasil;
}

describe("snippet Caddy pendaftaran publik korporat", () => {
  test("hanya dua matcher, persis (tanpa wildcard), dengan metode", () => {
    expect(isi.filter((b) => b.startsWith("path "))).toEqual([
      "path /v1/public/corporate-registrations",
      "path /v1/public/corporate-registrations/consent-text",
    ]);
    expect(isi.filter((b) => b.startsWith("method "))).toEqual(["method POST", "method GET HEAD"]);
    expect(isi.join("\n")).not.toMatch(/\*\s*$|path [^\n]*\*/m);
    expect(isi.filter((b) => b.startsWith("handle "))).toEqual(["handle @rs_pendaftaran_kirim {", "handle @rs_pendaftaran_teks {"]);
  });

  test.each([
    ["rs_pendaftaran_kirim", "/v1/public/corporate-registrations?"],
    ["rs_pendaftaran_teks", "/v1/public/corporate-registrations/consent-text?"],
  ])("handle @%s: URI ditulis ulang ke path tetap tanpa query dan badan dibatasi SEBELUM reverse_proxy", (nama, uri) => {
    const b = blokHandle(nama);
    const iRewrite = b.indexOf(`rewrite * ${uri}`);
    const iBatas = b.indexOf("max_size 16KB");
    const iProxy = b.findIndex((x) => x.startsWith("reverse_proxy "));
    expect(iRewrite).toBeGreaterThanOrEqual(0);
    expect(iBatas).toBeGreaterThanOrEqual(0);
    expect(iProxy).toBeGreaterThan(iRewrite);
    expect(iProxy).toBeGreaterThan(iBatas);
    expect(b.filter((x) => x.startsWith("reverse_proxy "))).toEqual(["reverse_proxy {args[0]}"]);
  });
});
