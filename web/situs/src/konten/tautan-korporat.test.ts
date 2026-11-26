import { describe, expect, test } from "vitest";
import { ambilUrlKorporat } from "./tautan-korporat.ts";

const PROD = { NODE_ENV: "production" };
const DEV = { NODE_ENV: "development" };

describe("ambilUrlKorporat (CR-032)", () => {
  test("https valid → href tepat", () => {
    expect(ambilUrlKorporat({ ...PROD, SITUS_KORPORAT_URL: "https://member.contoh.test/masuk" })).toBe("https://member.contoh.test/masuk");
  });

  test("kosong atau tidak diatur → null", () => {
    expect(ambilUrlKorporat({ ...PROD })).toBeNull();
    expect(ambilUrlKorporat({ ...PROD, SITUS_KORPORAT_URL: "" })).toBeNull();
  });

  test.each([
    ["skema javascript", "javascript:alert(1)"],
    ["skema data", "data:text/html,x"],
    ["http di production", "http://member.contoh.test"],
    ["http localhost di production", "http://localhost:3000"],
    ["kredensial", "https://user:pass@member.contoh.test"],
    ["user tanpa sandi", "https://user@member.contoh.test"],
    ["protocol-relative", "//evil.test"],
    ["relatif", "/masuk"],
    ["tanpa skema", "member.contoh.test"],
    ["spasi di tengah", "https://member.contoh.test/a b"],
    ["spasi di ujung", " https://member.contoh.test"],
    ["tab", "https://member.contoh.test/\tx"],
    ["baris baru", "https://member.contoh.test/\nx"],
    ["karakter kendali NUL", "https://member.contoh.test/\u0000"],
    ["backslash", "https://member.contoh.test\\@evil.test"],
    ["host kosong", "https://"],
  ])("tidak valid (%s) → null", (_nama, nilai) => {
    expect(ambilUrlKorporat({ ...PROD, SITUS_KORPORAT_URL: nilai })).toBeNull();
  });

  test("http hanya untuk localhost/127.0.0.1 saat development", () => {
    expect(ambilUrlKorporat({ ...DEV, SITUS_KORPORAT_URL: "http://localhost:3002/" })).toBe("http://localhost:3002/");
    expect(ambilUrlKorporat({ ...DEV, SITUS_KORPORAT_URL: "http://127.0.0.1:3002/" })).toBe("http://127.0.0.1:3002/");
    expect(ambilUrlKorporat({ ...DEV, SITUS_KORPORAT_URL: "http://member.contoh.test" })).toBeNull();
    expect(ambilUrlKorporat({ ...DEV, SITUS_KORPORAT_URL: "http://localhost.evil.test" })).toBeNull();
  });

  test("tanpa NODE_ENV: http ditolak", () => {
    expect(ambilUrlKorporat({ SITUS_KORPORAT_URL: "http://localhost:3002" })).toBeNull();
  });
});
