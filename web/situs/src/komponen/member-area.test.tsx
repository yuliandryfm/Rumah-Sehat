import { renderToStaticMarkup } from "react-dom/server";
import { afterEach, describe, expect, test, vi } from "vitest";
import { KerangkaSitus } from "./KerangkaSitus.tsx";
import { MasukMemberArea } from "./MasukMemberArea.tsx";

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
    expect(html).toContain('class="rs-btn rs-btn--primary"');
    expect(html).toContain('rel="noopener noreferrer"');
    expect(html).toContain("Masuk member area");
    expect(html).toContain("Member area adalah aplikasi terpisah dari situs ini.");
    expect(html).not.toContain("Daftarkan perusahaan");
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
    },
  );
});
