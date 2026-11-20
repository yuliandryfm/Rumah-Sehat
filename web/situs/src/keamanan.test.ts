import { NextRequest } from "next/server";
import { describe, expect, test } from "vitest";
import konfigurasi from "../next.config.ts";
import { HEADER_STATIS, buatCsp, buatNonce } from "./keamanan.ts";
import { config, proxy } from "./proxy.ts";

function arahan(csp: string, nama: string): string {
  const baris = csp.split("; ").find((a) => a === nama || a.startsWith(`${nama} `));
  if (baris === undefined) throw new Error(`arahan ${nama} tidak ada`);
  return baris;
}

describe("buatCsp", () => {
  const csp = buatCsp("abc123");

  test("script-src hanya self + nonce + strict-dynamic, tanpa unsafe-inline/eval → skrip inline ditolak", () => {
    expect(arahan(csp, "script-src")).toBe("script-src 'self' 'nonce-abc123' 'strict-dynamic'");
    expect(csp).not.toContain("unsafe-inline");
    expect(csp).not.toContain("unsafe-eval");
  });

  test("tanpa host pihak ketiga: semua sumber 'self' atau 'none' → tidak ada CDN, font, atau sematan", () => {
    expect(csp).not.toMatch(/https?:|\*|\/\//);
    expect(arahan(csp, "default-src")).toBe("default-src 'self'");
    expect(arahan(csp, "font-src")).toBe("font-src 'self'");
    expect(arahan(csp, "img-src")).toBe("img-src 'self'");
    expect(arahan(csp, "object-src")).toBe("object-src 'none'");
  });

  test("frame-ancestors 'none', base-uri dan form-action self", () => {
    expect(arahan(csp, "frame-ancestors")).toBe("frame-ancestors 'none'");
    expect(arahan(csp, "base-uri")).toBe("base-uri 'self'");
    expect(arahan(csp, "form-action")).toBe("form-action 'self'");
  });

  test("berkas statis (tanpa nonce) tetap tanpa unsafe-*", () => {
    const statis = buatCsp(null);
    expect(arahan(statis, "script-src")).toBe("script-src 'self'");
    expect(statis).not.toContain("unsafe-");
  });

  test("unsafe-eval hanya muncul di mode dev, tidak pernah unsafe-inline", () => {
    expect(buatCsp("n", true)).toContain("'unsafe-eval'");
    expect(buatCsp("n", true)).not.toContain("unsafe-inline");
    expect(buatCsp("n", false)).not.toContain("unsafe-eval");
  });
});

describe("buatNonce", () => {
  test("nonce 128 bit base64 dan berbeda tiap panggilan", () => {
    const a = buatNonce();
    const b = buatNonce();
    expect(a).toMatch(/^[A-Za-z0-9+/]{22}==$/);
    expect(a).not.toBe(b);
  });
});

describe("proxy", () => {
  test("setiap respons membawa CSP bernonce dan semua header keamanan → pengerasan lengkap", () => {
    const res = proxy(new NextRequest("http://situs.test/"));
    const csp = res.headers.get("content-security-policy") ?? "";
    expect(csp).toMatch(/script-src 'self' 'nonce-[^']+' 'strict-dynamic'/);
    expect(res.headers.get("strict-transport-security")).toMatch(/^max-age=\d{8,}; includeSubDomains$/);
    expect(res.headers.get("x-content-type-options")).toBe("nosniff");
    expect(res.headers.get("referrer-policy")).toBe("strict-origin-when-cross-origin");
    expect(res.headers.get("permissions-policy")).toContain("camera=()");
    expect(res.headers.get("permissions-policy")).toContain("microphone=()");
    expect(res.headers.get("permissions-policy")).toContain("geolocation=()");
    expect(csp).toContain("frame-ancestors 'none'");
    expect(res.headers.get("x-frame-options")).toBe("DENY");
    for (const { key, value } of HEADER_STATIS) expect(res.headers.get(key)).toBe(value);
  });

  test("tidak ada Set-Cookie, juga untuk permintaan yang membawa cookie → situs tanpa sesi", () => {
    const res = proxy(new NextRequest("http://situs.test/tidak-ada", { headers: { cookie: "a=b" } }));
    expect(res.headers.getSetCookie()).toEqual([]);
    expect(res.headers.get("set-cookie")).toBeNull();
  });

  test("nonce berbeda tiap permintaan dan diteruskan ke render lewat header permintaan", () => {
    const ambil = (r: Response) => /'nonce-([^']+)'/.exec(r.headers.get("content-security-policy") ?? "")?.[1];
    const a = proxy(new NextRequest("http://situs.test/"));
    const b = proxy(new NextRequest("http://situs.test/"));
    expect(ambil(a)).toBeDefined();
    expect(ambil(a)).not.toBe(ambil(b));
    // Next menandai header permintaan yang diteruskan dengan awalan ini.
    expect(a.headers.get("x-middleware-request-content-security-policy")).toBe(a.headers.get("content-security-policy"));
  });

  test("matcher mencakup halaman, 404, dan prefetch; hanya /_next/static dikecualikan", () => {
    expect(config.matcher).toHaveLength(1);
    const pola = new RegExp(`^${(config.matcher[0] ?? "").replace(/\(\?!/, "(?!")}$`);
    expect(pola.test("/")).toBe(true);
    expect(pola.test("/tidak-ada")).toBe(true);
    expect(pola.test("/favicon.ico")).toBe(true);
    expect(pola.test("/_next/static/css/a.css")).toBe(false);
  });
});

describe("next.config", () => {
  test("pengerasan ADR-024: standalone, tanpa optimasi gambar, tanpa X-Powered-By", () => {
    expect(konfigurasi.output).toBe("standalone");
    expect(konfigurasi.images?.unoptimized).toBe(true);
    expect(konfigurasi.poweredByHeader).toBe(false);
    expect(konfigurasi.experimental).toBeUndefined();
  });

  test("berkas statis mendapat header keamanan dan CSP tanpa unsafe-*", async () => {
    const aturan = (await konfigurasi.headers?.()) ?? [];
    expect(aturan).toHaveLength(1);
    expect(aturan[0]?.source).toBe("/_next/static/:path*");
    const h = new Map((aturan[0]?.headers ?? []).map((x) => [x.key.toLowerCase(), x.value]));
    expect(h.get("x-content-type-options")).toBe("nosniff");
    expect(h.get("strict-transport-security")).toBeDefined();
    expect(h.get("content-security-policy")).toContain("frame-ancestors 'none'");
    expect(h.get("content-security-policy")).not.toContain("unsafe-");
  });
});
