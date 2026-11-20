import { randomBytes } from "node:crypto";
import { describe, expect, test } from "vitest";
import { loadBffConfig } from "./config.ts";
import { BffSecretError, parseBffSecrets, readBffSecrets } from "./openbao.ts";
import { testEnv } from "./testing/fixtures.ts";

const kunci = (): string => randomBytes(32).toString("base64url");

function sah(extra: Record<string, unknown> = {}): Record<string, unknown> {
  return {
    oidc_client_secret: "uji-secret-klien",
    valkey_password: "uji-valkey",
    session_key: kunci(),
    session_key_id: "k2",
    ...extra,
  };
}

describe("parseBffSecrets", () => {
  test("rahasia lengkap → kunci 32 byte dan rahasia terbungkus", () => {
    const s = parseBffSecrets(sah({ session_key_previous: kunci(), session_key_previous_id: "k1" }));
    expect(s.sessionKeys.current.key.expose()).toHaveLength(32);
    expect(s.sessionKeys.previous?.id).toBe("k1");
    expect(JSON.stringify(s)).not.toContain("uji-secret-klien");
  });

  test.each([
    ["kunci 16 byte", { session_key: randomBytes(16).toString("base64url") }, /tepat 32 byte/],
    ["kunci base64 standar berpadding", { session_key: `${randomBytes(32).toString("base64")}=` }, /base64url/],
    ["ID kunci tidak sah", { session_key_id: "K 1" }, /session_key_id/],
    ["secret klien hilang", { oidc_client_secret: undefined }, /oidc_client_secret/],
    ["password Valkey kosong", { valkey_password: "" }, /valkey_password/],
    ["kunci sebelumnya tanpa ID", { session_key_previous: kunci() }, /session_key_previous_id/],
    ["ID kunci sebelumnya sama", { session_key_previous: kunci(), session_key_previous_id: "k2" }, /tidak boleh sama/],
  ])("%s → ditolak", (_n, extra, pola) => {
    expect(() => parseBffSecrets(sah(extra))).toThrow(pola);
  });
});

describe("readBffSecrets", () => {
  const cfg = loadBffConfig(testEnv()).openbao;

  test("memanggil KV v2 dengan header X-Vault-Token dan mengurai data.data", async () => {
    let dipanggil: { url: string; token: string | null } | null = null;
    const s = await readBffSecrets(cfg, {
      fetch: (input, init) => {
        const headers = new Headers(init?.headers);
        dipanggil = { url: input instanceof URL ? input.href : (input as string), token: headers.get("X-Vault-Token") };
        return Promise.resolve(Response.json({ data: { data: sah(), metadata: { version: 3 } } }));
      },
    });
    expect(dipanggil).toEqual({
      url: "http://127.0.0.1:18200/v1/secret/data/rs/bff/backoffice",
      token: "uji-token-openbao-sintetis",
    });
    expect(s.sessionKeys.current.id).toBe("k2");
  });

  test("OpenBao menolak (403) → BffSecretError tanpa token di pesan", async () => {
    const p = readBffSecrets(cfg, {
      fetch: () => Promise.resolve(new Response('{"errors":["permission denied"]}', { status: 403 })),
    });
    await expect(p).rejects.toBeInstanceOf(BffSecretError);
    await expect(p).rejects.not.toThrow(/uji-token-openbao-sintetis/);
  });

  test("OpenBao tidak terjangkau → BffSecretError", async () => {
    await expect(readBffSecrets(cfg, { fetch: () => Promise.reject(new Error("ECONNREFUSED")) })).rejects.toThrow(
      /tidak dapat dihubungi/,
    );
  });

  test("respons bukan KV v2 → ditolak", async () => {
    await expect(
      readBffSecrets(cfg, { fetch: () => Promise.resolve(Response.json({ data: sah() })) }),
    ).rejects.toThrow(/KV v2/);
  });
});
