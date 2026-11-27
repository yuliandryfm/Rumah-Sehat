import { renderToStaticMarkup } from "react-dom/server";
import { afterEach, describe, expect, test, vi } from "vitest";
import { KerangkaSitus } from "./KerangkaSitus.tsx";
import { MasukMemberArea, urlDaftar } from "./MasukMemberArea.tsx";

vi.mock("next/link", () => ({
  default: ({ href, children }: { href: string; children: unknown }) => <a href={href}>{children as never}</a>,
}));
vi.mock("./NavUtama.tsx", () => ({ NavUtama: () => null }));

afterEach(() => vi.unstubAllEnvs());

const URL_UJI = "https://member.contoh.test/";

describe("tautan member area korporat (CR-032)", () => {
  test("URL valid: blok Sudah terdaftar? dengan tombol primer, rel aman, dan kalimat penjelas", () => {
    vi.stubEnv("SITUS_KORPORAT_URL", URL_UJI);
    const html = renderToStaticMarkup(<MasukMemberArea />);
    expect(html).toContain("Sudah terdaftar?");
    expect(html).toContain(`href="${URL_UJI}"`);
    expect(html).toMatch(/<a class="rs-btn rs-btn--secondary" href="https:\/\/member\.contoh\.test\/" rel="noopener noreferrer">Masuk member area<\/a>/);
    expect(html).toContain("Member area adalah aplikasi terpisah dari situs ini.");
  });

  test("F1-T103: tombol utama Daftarkan perusahaan ke /daftar di origin web korporat, rel aman", () => {
    vi.stubEnv("SITUS_KORPORAT_URL", URL_UJI);
    const html = renderToStaticMarkup(<MasukMemberArea />);
    expect(html).toContain("Belum terdaftar?");
    expect(html).toMatch(/<a class="rs-btn rs-btn--primary" href="https:\/\/member\.contoh\.test\/daftar" rel="noopener noreferrer">Daftarkan perusahaan<\/a>/);
    // Satu tombol utama per layar (DG §4).
    expect(html.match(/rs-btn--primary/g)).toHaveLength(1);
    // Tidak ada formulir ke domain korporat (KEPUTUSAN-352).
    expect(html).not.toContain("<form");
  });

  test("F1-T103: URL korporat berjalur tetap menghasilkan /daftar di origin yang sama", () => {
    expect(urlDaftar("https://member.contoh.test/masuk")).toBe("https://member.contoh.test/daftar");
    expect(urlDaftar("http://localhost:3002/")).toBe("http://localhost:3002/daftar");
  });

  test("footer: tautan Masuk HR ke URL yang sama", () => {
    vi.stubEnv("SITUS_KORPORAT_URL", URL_UJI);
    const html = renderToStaticMarkup(<KerangkaSitus>isi</KerangkaSitus>);
    expect(html).toMatch(/<a href="https:\/\/member\.contoh\.test\/" rel="noopener noreferrer">Masuk HR<\/a>/);
  });

  test.each(["", "javascript:alert(1)", "http://member.contoh.test", "https://user:pass@x.test", "//evil.test", "https://a.test/ x", "https://a.test/\u0007"])(
    "env %j: tidak ada tautan sama sekali",
    (nilai) => {
      vi.stubEnv("NODE_ENV", "production");
      vi.stubEnv("SITUS_KORPORAT_URL", nilai);
      expect(renderToStaticMarkup(<MasukMemberArea />)).toBe("");
      const kerangka = renderToStaticMarkup(<KerangkaSitus>isi</KerangkaSitus>);
      expect(kerangka).not.toContain("Masuk HR");
      expect(kerangka).not.toContain("member area");
      expect(kerangka).not.toContain("Daftarkan perusahaan");
    },
  );
});
