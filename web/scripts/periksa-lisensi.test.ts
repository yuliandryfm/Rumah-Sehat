import { describe, expect, it } from "vitest";
import { jalankan, komponenLisensi, periksaPaket, ratakan, workspaceDariYaml } from "./periksa-lisensi.mjs";

const p = (name: string, license: string | undefined) => ({ name, versions: ["1.0.0"], license });

describe("periksaPaket", () => {
  it("meluluskan lisensi izin dan kombinasi OR/AND yang semua komponennya diizinkan", () => {
    const paket = [
      p("a", "MIT"),
      p("b", "(MIT OR Apache-2.0)"),
      p("c", "BSD-2-Clause AND ISC"),
      p("d", "0BSD"),
      p("e", "CC-BY-4.0"),
      p("f", "BlueOak-1.0.0"),
    ];
    expect(periksaPaket(paket, "runtime")).toEqual([]);
  });
  it("menolak lisensi terlarang di runtime dan dev", () => {
    for (const lisensi of ["GPL-3.0-only", "AGPL-3.0-or-later", "CC0-1.0", "SSPL-1.0", "(GPL-3.0-only AND MIT)"]) {
      expect(periksaPaket([p("x", lisensi)], "runtime")).toHaveLength(1);
      expect(periksaPaket([p("x", lisensi)], "dev")).toHaveLength(1);
    }
  });
  it("OR lolos bila salah satu komponen diizinkan, gagal bila tidak ada", () => {
    expect(periksaPaket([p("x", "(MIT OR GPL-3.0-only)")], "runtime")).toEqual([]);
    expect(periksaPaket([p("x", "(MIT OR CC0-1.0)")], "runtime")).toEqual([]);
    expect(periksaPaket([p("x", "(GPL-3.0-only OR CC0-1.0)")], "runtime")).toHaveLength(1);
  });
  it("AND wajib semua komponen diizinkan; AND mengikat lebih kuat dari OR", () => {
    expect(periksaPaket([p("x", "MIT AND GPL-3.0-only")], "runtime")).toHaveLength(1);
    expect(periksaPaket([p("x", "(MIT AND ISC)")], "runtime")).toEqual([]);
    expect(periksaPaket([p("x", "GPL-3.0-only AND MIT OR ISC")], "runtime")).toEqual([]);
    expect(periksaPaket([p("x", "MIT AND (GPL-3.0-only OR CC0-1.0)")], "runtime")).toHaveLength(1);
  });
  it("menolak ekspresi cacat", () => {
    for (const lisensi of ["MIT OR", "(MIT", "MIT)", "AND MIT"]) {
      expect(periksaPaket([p("x", lisensi)], "runtime")).toHaveLength(1);
    }
  });
  it("menolak MPL-2.0 di runtime tetapi meluluskannya di dev", () => {
    expect(periksaPaket([p("lightningcss", "MPL-2.0")], "runtime")).toHaveLength(1);
    expect(periksaPaket([p("lightningcss", "MPL-2.0")], "dev")).toEqual([]);
  });
  it("menolak Python-2.0 di runtime tetapi meluluskannya di dev", () => {
    expect(periksaPaket([p("argparse", "Python-2.0")], "runtime")).toHaveLength(1);
    expect(periksaPaket([p("argparse", "Python-2.0")], "dev")).toEqual([]);
  });
  it("menolak lisensi Unknown, kosong, atau tak dikenali", () => {
    for (const lisensi of ["Unknown", "", undefined, "SEE LICENSE IN LICENSE.md"]) {
      expect(periksaPaket([p("x", lisensi)], "dev")).toHaveLength(1);
    }
  });
  it("mengizinkan LGPL-3.0-or-later hanya untuk @img/sharp-libvips-*", () => {
    expect(periksaPaket([p("@img/sharp-libvips-linux-x64", "LGPL-3.0-or-later")], "runtime")).toEqual([]);
    expect(periksaPaket([p("@img/sharp-libvips-darwin-arm64", "LGPL-3.0-or-later")], "runtime")).toEqual([]);
    expect(periksaPaket([p("paket-lain", "LGPL-3.0-or-later")], "runtime")).toHaveLength(1);
    expect(periksaPaket([p("@img/sharp-linux-x64", "LGPL-3.0-or-later")], "runtime")).toHaveLength(1);
    expect(periksaPaket([p("@img/sharp-libvips-linux-x64", "GPL-3.0-only")], "runtime")).toHaveLength(1);
  });
});

describe("komponenLisensi dan ratakan", () => {
  it("memecah ekspresi dan menolak bentuk aneh", () => {
    expect(komponenLisensi("(MIT OR Apache-2.0)")).toEqual(["MIT", "Apache-2.0"]);
    expect(komponenLisensi("")).toBeNull();
    expect(komponenLisensi("MIT WITH Foo")).toBeNull();
  });
  it("meratakan keluaran pnpm", () => {
    expect(ratakan({ MIT: [{ name: "a", versions: ["1"] }] })).toEqual([{ name: "a", versions: ["1"], license: "MIT" }]);
  });
});

describe("workspaceDariYaml", () => {
  const anak = (induk: string) => (induk === "packages" ? ["ui", "api"] : []);
  it("menurunkan workspace dari pnpm-workspace.yaml, termasuk glob satu tingkat", () => {
    const yaml = "# komentar\npackages:\n  - backoffice\n  - situs\n  - packages/*\n";
    expect(workspaceDariYaml(yaml, anak)).toEqual([".", "backoffice", "situs", "packages/ui", "packages/api"]);
  });
  it("gagal pada pola tak didukung atau daftar kosong", () => {
    expect(() => workspaceDariYaml("packages:\n  - '**'\n", anak)).toThrow();
    expect(() => workspaceDariYaml("packages:\n  - '!x'\n", anak)).toThrow();
    expect(() => workspaceDariYaml("catalog:\n  a: 1\n", anak)).toThrow();
  });
});

describe("jalankan", () => {
  it("gagal bila node_modules belum terpasang", () => {
    expect(jalankan("/tidak/ada/web")).toBe(1);
  });
});
