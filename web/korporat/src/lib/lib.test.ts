import { describe, expect, test } from "vitest";
import { HEADER_DINAMIS, HEADER_STATIS, buatCsp, buatNonce } from "../keamanan.ts";
import { idKorelasi, kodeReferensi } from "./korelasi.ts";
import { existsSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { MENU } from "./navigasi.ts";

describe("CSP dan header keamanan", () => {
  test("CSP bernonce: tanpa unsafe-*, tanpa host pihak ketiga, tanpa sematan", () => {
    const csp = buatCsp("abc123");
    expect(csp).toContain("script-src 'self' 'nonce-abc123' 'strict-dynamic'");
    expect(csp).toContain("style-src 'self' 'nonce-abc123'");
    expect(csp).toContain("frame-ancestors 'none'");
    expect(csp).toContain("object-src 'none'");
    expect(csp).toContain("connect-src 'self'");
    expect(csp).not.toMatch(/unsafe-/);
    expect(csp).not.toMatch(/https?:\/\//);
  });

  test("unsafe-eval hanya saat mode development", () => {
    expect(buatCsp("n", true)).toContain("'unsafe-eval'");
    expect(buatCsp("n", false)).not.toContain("unsafe-eval");
  });

  test("CSP berkas statis tanpa nonce", () => {
    expect(buatCsp(null)).not.toContain("nonce-");
  });

  test("nonce acak per panggilan, header statis lengkap", () => {
    expect(buatNonce()).not.toBe(buatNonce());
    const nama = HEADER_STATIS.map((h) => h.key);
    for (const wajib of ["Strict-Transport-Security", "X-Content-Type-Options", "Referrer-Policy", "Permissions-Policy", "X-Frame-Options"]) {
      expect(nama).toContain(wajib);
    }
  });
});

describe("ID korelasi dan kode referensi (KEPUTUSAN-172)", () => {
  const uuid = "0193a5e6-7f00-7abc-8def-0123456789ab";

  test("UUID sah dipakai ulang (huruf kecil); selain itu UUID baru", () => {
    expect(idKorelasi(uuid.toUpperCase())).toBe(uuid);
    expect(idKorelasi("bukan-uuid")).toMatch(/^[0-9a-f-]{36}$/);
    expect(idKorelasi("bukan-uuid")).not.toBe("bukan-uuid");
    expect(idKorelasi(null)).toMatch(/^[0-9a-f-]{36}$/);
  });

  test("correlation_id didahulukan dari digest", () => {
    expect(kodeReferensi({ correlationId: uuid, digest: "1234567890" })).toBe(uuid);
    expect(kodeReferensi({ digest: "1234567890" })).toBe("1234567890");
  });

  test("nilai tidak sah ditolak: teks galat, markup, terlalu panjang atau pendek → null", () => {
    expect(kodeReferensi({})).toBeNull();
    expect(kodeReferensi({ digest: "<script>alert(1)</script>" })).toBeNull();
    expect(kodeReferensi({ digest: "Error: kueri gagal pada tabel patients" })).toBeNull();
    expect(kodeReferensi({ digest: "a".repeat(65) })).toBeNull();
    expect(kodeReferensi({ digest: "abc" })).toBeNull();
    expect(kodeReferensi({ correlationId: 42 })).toBeNull();
  });
});

describe("navigasi member area (K01 layar 9)", () => {
  test("lima menu (CR-039); hanya halaman yang ada yang menjadi tautan; fitur belum ada bukan tautan", () => {
    expect(MENU.map((i) => i.label)).toEqual(["Beranda", "Pesanan", "Peserta", "Tagihan", "Tim HR"]);
    expect(MENU.filter((i) => i.href !== undefined).map((i) => i.href)).toEqual(["/", "/kelola-hr"]);
    expect(MENU.filter((i) => i.href === undefined).map((i) => i.label)).toEqual(["Pesanan", "Peserta", "Tagihan"]);
  });

  test("negatif: setiap tautan menu menunjuk rute yang punya berkas halaman", () => {
    const akar = fileURLToPath(new URL("../app/(terautentikasi)", import.meta.url));
    for (const i of MENU) {
      if (i.href === undefined) continue;
      const berkas = join(akar, i.href === "/" ? "" : i.href, "page.tsx");
      expect(existsSync(berkas), `${i.label} -> ${i.href}`).toBe(true);
    }
    // Item "segera hadir" tidak punya halaman: tautan ke sana akan menjadi 404.
    for (const rute of ["pesanan", "peserta", "tagihan", "invoice"]) {
      expect(existsSync(join(akar, rute, "page.tsx"))).toBe(false);
    }
  });
});

describe("header respons dinamis", () => {
  test("no-store untuk respons dinamis (data peserta); berkas statis tidak ikut", () => {
    expect(HEADER_DINAMIS).toEqual([{ key: "Cache-Control", value: "no-store" }]);
    expect(HEADER_STATIS.map((h) => h.key)).not.toContain("Cache-Control");
  });
});

