import { inspect } from "node:util";
import { describe, expect, test } from "vitest";
import { BffConfigError, isLoopbackHost, loadBffConfig } from "./config.ts";
import { testEnv } from "./testing/fixtures.ts";

describe("loadBffConfig", () => {
  test("lingkungan lokal yang sah → konfigurasi bertipe", () => {
    const c = loadBffConfig(testEnv());
    expect(c.appName).toBe("backoffice");
    expect(c.oidc.redirectUri.href).toBe("http://127.0.0.1:3000/auth/callback");
    expect(c.openbao.kvPath).toBe("rs/bff/backoffice");
  });

  test("token OpenBao tidak tercetak lewat JSON, String, atau inspect", () => {
    const c = loadBffConfig(testEnv());
    const dump = `${JSON.stringify(c)} ${String(c.openbao.token)} ${inspect(c, { depth: 5 })}`;
    expect(dump).not.toContain("uji-token-openbao-sintetis");
    expect(c.openbao.token.expose()).toBe("uji-token-openbao-sintetis");
  });

  test("variabel hilang → galat menyebut nama, tidak menyebut nilai lain", () => {
    expect(() => loadBffConfig(testEnv({ BFF_OIDC_CLIENT_ID: undefined }))).toThrow(/BFF_OIDC_CLIENT_ID wajib/);
    try {
      loadBffConfig(testEnv({ BFF_OPENBAO_KV_PATH: "  " }));
    } catch (e) {
      expect(e).toBeInstanceOf(BffConfigError);
      expect(String(e)).not.toContain("uji-token-openbao-sintetis");
    }
  });

  test.each([
    ["issuer http non-loopback", { BFF_OIDC_ISSUER: "http://id.contoh.invalid/realms/admin" }, /BFF_OIDC_ISSUER/],
    ["origin http non-loopback", { BFF_APP_ORIGIN: "http://backoffice.contoh.invalid" }, /BFF_APP_ORIGIN/],
    ["origin dengan path", { BFF_APP_ORIGIN: "https://bo.contoh.invalid/app" }, /BFF_APP_ORIGIN/],
    ["redirect URI beda asal", { BFF_OIDC_REDIRECT_URI: "http://127.0.0.1:3001/auth/callback" }, /satu asal/],
    ["Valkey dengan password di URL", { BFF_VALKEY_URL: "redis://:rahasia@127.0.0.1:6379" }, /kredensial/],
    ["Valkey bukan redis://", { BFF_VALKEY_URL: "http://127.0.0.1:6379" }, /BFF_VALKEY_URL/],
    ["OpenBao https (TLS belum diterapkan)", { OPENBAO_ADDR: "https://bao.contoh.invalid:8200" }, /OPENBAO_ADDR/],
    ["OpenBao non-loopback", { OPENBAO_ADDR: "http://10.0.0.5:8200" }, /OPENBAO_ADDR/],
    ["path KV dengan ..", { BFF_OPENBAO_KV_PATH: "rs/../root" }, /BFF_OPENBAO_KV_PATH/],
    ["nama aplikasi tidak sah", { BFF_APP_NAME: "Back_Office" }, /BFF_APP_NAME/],
    ["client id tidak sah", { BFF_OIDC_CLIENT_ID: "a b" }, /BFF_OIDC_CLIENT_ID/],
  ])("%s → ditolak", (_nama, override, pola) => {
    expect(() => loadBffConfig(testEnv(override))).toThrow(pola);
  });

  test("produksi: https diterima", () => {
    const c = loadBffConfig(
      testEnv({
        BFF_APP_ORIGIN: "https://backoffice.contoh.invalid",
        BFF_OIDC_ISSUER: "https://id.contoh.invalid/realms/admin",
        BFF_OIDC_REDIRECT_URI: "https://backoffice.contoh.invalid/auth/callback",
        BFF_VALKEY_URL: "rediss://valkey.internal:6379/0",
      }),
    );
    expect(c.appOrigin.origin).toBe("https://backoffice.contoh.invalid");
  });
});

describe("isLoopbackHost", () => {
  test.each([
    ["http://127.0.0.1:1", true],
    ["http://127.10.0.1:1", true],
    ["http://[::1]:1", true],
    ["http://localhost:1", false],
    ["http://127.0.0.1.contoh.invalid:1", false],
    ["http://10.0.0.1:1", false],
  ])("%s → %s", (url, hasil) => {
    expect(isLoopbackHost(new URL(url))).toBe(hasil);
  });
});
