// Test integrasi F1-T41: Valkey 9.1.3 (ioredis 6.0.0) dan OpenBao 2.7.2 mode dev di kontainer sekali pakai.
// Membuktikan kompatibilitas ioredis–Valkey (ADR-024 konsekuensi) dan pembacaan rahasia BFF dari OpenBao.

import { randomBytes } from "node:crypto";
import { Redis } from "ioredis";
import { afterAll, beforeAll, describe, expect, test } from "vitest";
import { loadBffConfig } from "./config.ts";
import { createBffCore } from "./core.ts";
import { BffSecretError, readBffSecrets } from "./openbao.ts";
import { Sensitive } from "./sensitive.ts";
import { ValkeyBackend } from "./store.ts";
import { Container, composeImage, randomSecret, waitFor } from "./testing/containers.ts";
import { testEnv } from "./testing/fixtures.ts";
import { sessionSuite } from "./testing/session-suite.ts";

const valkeyPassword = randomSecret();
const baoRoot = randomSecret();
let valkey: Container | undefined;
let bao: Container | undefined;
let valkeyUrl: URL;
let baoAddr: string;
let admin: Redis | undefined;
const adm = (): Redis => {
  if (admin === undefined) {
    throw new Error("Valkey belum siap");
  }
  return admin;
};
const backends: ValkeyBackend[] = [];

beforeAll(async () => {
  valkey = Container.run({
    image: composeImage("valkey/valkey"),
    ports: [6379],
    env: { VALKEY_PASSWORD: valkeyPassword },
    cmd: ["sh", "-c", 'exec valkey-server --requirepass "$VALKEY_PASSWORD" --save "" --appendonly no'],
  });
  bao = Container.run({
    image: composeImage("openbao/openbao"),
    ports: [8200],
    env: { BAO_DEV_ROOT_TOKEN_ID: baoRoot },
    plainEnv: { BAO_DEV_LISTEN_ADDRESS: "0.0.0.0:8200" },
    extra: ["--cap-add", "IPC_LOCK"],
    cmd: ["server", "-dev"],
  });
  valkeyUrl = new URL(`redis://127.0.0.1:${String(valkey.hostPort(6379))}/0`);
  baoAddr = `http://127.0.0.1:${String(bao.hostPort(8200))}`;
  const a = new Redis({ host: "127.0.0.1", port: Number(valkeyUrl.port), password: valkeyPassword, lazyConnect: true });
  a.on("error", () => undefined);
  admin = a;
  await waitFor(async () => {
    await a.ping();
    return true;
  }, 60_000, "Valkey");
  await waitFor(async () => (await fetch(`${baoAddr}/v1/sys/health`)).ok, 60_000, "OpenBao");
});

afterAll(async () => {
  try {
    for (const b of backends) {
      await b.close();
    }
    admin?.disconnect();
  } finally {
    valkey?.remove();
    bao?.remove();
  }
});

function backend(): ValkeyBackend {
  const b = new ValkeyBackend({ url: valkeyUrl, password: new Sensitive(valkeyPassword) });
  backends.push(b);
  return b;
}

sessionSuite("Valkey 9.1.3", {
  backend: () => backend(),
  async dump(app) {
    const keys = (await adm().keys(`rs:bff:${app}:*`)).sort();
    const parts: unknown[] = [];
    for (const k of keys) {
      const type = await adm().type(k);
      parts.push([k, type === "hash" ? await adm().hgetall(k) : await adm().get(k)]);
    }
    return { keys, text: JSON.stringify(parts) };
  },
});

describe("ValkeyBackend di Valkey sungguhan", () => {
  test("TTL nyata: nilai sekali pakai hilang setelah TTL; GETDEL hanya sekali", async () => {
    const b = backend();
    const k = `rs:bff:ttl:${randomBytes(6).toString("hex")}`;
    expect(await b.putOnce(k, "v", 300)).toBe(true);
    expect(await b.putOnce(k, "v2", 300)).toBe(false);
    await new Promise((r) => setTimeout(r, 600));
    expect(await b.take(k)).toBeNull();
    expect(await b.putOnce(k, "v3", 10_000)).toBe(true);
    expect(await b.take(k)).toBe("v3");
    expect(await b.take(k)).toBeNull();
  });

  test("HASH: dibuat sekali; update tanpa TTL mempertahankan PTTL; update dengan TTL mengatur ulang", async () => {
    const b = backend();
    const k = `rs:bff:hash:${randomBytes(6).toString("hex")}`;
    expect(await b.createHash(k, { a: "1" }, 60_000)).toBe(true);
    expect(await b.createHash(k, { a: "2" }, 60_000)).toBe(false);
    expect(await b.getHash(k)).toEqual({ a: "1" });
    expect(await b.updateHashIfExists(k, { b: "2" }, null)).toBe(true);
    const pttl = await adm().pttl(k);
    expect(pttl).toBeGreaterThan(50_000);
    expect(pttl).toBeLessThanOrEqual(60_000);
    expect(await b.updateHashIfExists(k, { b: "3" }, 600_000)).toBe(true);
    expect(await adm().pttl(k)).toBeGreaterThan(590_000);
    expect(await b.delete(k)).toBe(true);
    expect(await b.updateHashIfExists(k, { c: "1" }, 60_000)).toBe(false);
    expect(await adm().exists(k)).toBe(0);
  });

  test("password Valkey salah → perintah gagal (tidak tersimpan diam-diam)", async () => {
    const salah = new ValkeyBackend({ url: valkeyUrl, password: new Sensitive("salah"), timeoutMs: 2_000 });
    backends.push(salah);
    await expect(salah.putOnce("rs:bff:x", "v", 1000)).rejects.toThrow();
  });
});

describe("OpenBao KV v2 → rahasia BFF", () => {
  const path = "rs/bff/backoffice";
  let bffToken = "";

  beforeAll(async () => {
    const put = await fetch(`${baoAddr}/v1/secret/data/${path}`, {
      method: "POST",
      headers: { "X-Vault-Token": baoRoot, "Content-Type": "application/json" },
      body: JSON.stringify({
        data: {
          oidc_client_secret: randomSecret(),
          valkey_password: valkeyPassword,
          session_key: randomBytes(32).toString("base64url"),
          session_key_id: "k1",
        },
      }),
    });
    expect(put.ok).toBe(true);
    // Policy minimum identitas BFF: hanya baca path rahasianya sendiri (KEPUTUSAN-79).
    const pol = await fetch(`${baoAddr}/v1/sys/policies/acl/bff-backoffice`, {
      method: "PUT",
      headers: { "X-Vault-Token": baoRoot, "Content-Type": "application/json" },
      body: JSON.stringify({ policy: `path "secret/data/${path}" { capabilities = ["read"] }` }),
    });
    expect(pol.ok).toBe(true);
    const tok = await fetch(`${baoAddr}/v1/auth/token/create`, {
      method: "POST",
      headers: { "X-Vault-Token": baoRoot, "Content-Type": "application/json" },
      body: JSON.stringify({ policies: ["bff-backoffice"], no_default_policy: true, ttl: "10m" }),
    });
    bffToken = ((await tok.json()) as { auth: { client_token: string } }).auth.client_token;
  });

  const cfg = (token: string, kvPath = path) =>
    loadBffConfig(
      testEnv({
        OPENBAO_ADDR: baoAddr,
        OPENBAO_TOKEN: token,
        BFF_OPENBAO_KV_PATH: kvPath,
        BFF_VALKEY_URL: valkeyUrl.href,
      }),
    );

  test("token identitas BFF membaca rahasianya; kunci sesi 32 byte", async () => {
    const s = await readBffSecrets(cfg(bffToken).openbao);
    expect(s.sessionKeys.current.key.expose()).toHaveLength(32);
    expect(s.valkeyPassword.expose()).toBe(valkeyPassword);
  });

  test("path lain dengan token BFF → ditolak (403), galat tanpa token", async () => {
    const p = readBffSecrets(cfg(bffToken, "rs/bff/portal").openbao);
    await expect(p).rejects.toBeInstanceOf(BffSecretError);
    await expect(p).rejects.toThrow(/HTTP 403/);
    await expect(p).rejects.not.toThrow(new RegExp(bffToken));
  });

  test("token tidak sah → ditolak", async () => {
    await expect(readBffSecrets(cfg("token-salah").openbao)).rejects.toThrow(BffSecretError);
  });

  test("createBffCore: rahasia dari OpenBao, password Valkey dari OpenBao, sesi tersimpan dan terbuka", async () => {
    const core = await createBffCore(cfg(bffToken));
    try {
      const s = await core.sessions.create({
        access_token: "at-uji",
        refresh_token: "rt-uji",
        access_expires_at: Date.now() + 300_000,
        sub: "f47ac10b-58cc-4372-a567-0e02b2c3d479",
      });
      const r = await core.sessions.load(s.id);
      expect(r.status).toBe("active");
      if (r.status === "active") {
        expect((await core.sessions.tokens(r.session)).access_token).toBe("at-uji");
      }
      const pttl = await adm().pttl(core.sessions.key("s", s.id));
      expect(pttl).toBeGreaterThan(30 * 60_000);
      expect(pttl).toBeLessThanOrEqual(31 * 60_000);
      await core.sessions.destroy(s.id);
    } finally {
      await core.close();
    }
  });
});
