import { createHash } from "node:crypto";
import { afterAll, beforeAll, describe, expect, test } from "vitest";
import { OidcClient, OidcError } from "./oidc.ts";
import { Sensitive } from "./sensitive.ts";
import { FakeOp } from "./testing/fake-op.ts";

const REDIRECT = "http://127.0.0.1:3000/auth/callback";
const op = new FakeOp({ clientId: "rs-backoffice-bff", clientSecret: "uji-secret:+/=", redirectUri: REDIRECT });
let client: OidcClient;

beforeAll(async () => {
  await op.start();
  client = await OidcClient.discover({
    issuer: new URL(op.issuer),
    clientId: "rs-backoffice-bff",
    clientSecret: new Sensitive("uji-secret:+/="),
    redirectUri: new URL(REDIRECT),
  });
});

afterAll(async () => {
  await op.stop();
});

describe("OidcClient (trace: ADR-011 butir 7, ADR-024 butir 3)", () => {
  test("URL otorisasi: code + PKCE S256 dari verifier, state, nonce, redirect URI terdaftar, scope openid", async () => {
    const { url, tx } = await client.beginLogin("/antrean");
    const p = url.searchParams;
    expect(url.href.startsWith(`${op.issuer}/protocol/openid-connect/auth?`)).toBe(true);
    expect(p.get("response_type")).toBe("code");
    expect(p.get("client_id")).toBe("rs-backoffice-bff");
    expect(p.get("redirect_uri")).toBe(REDIRECT);
    expect(p.get("scope")).toBe("openid");
    expect(p.get("state")).toBe(tx.state);
    expect(p.get("nonce")).toBe(tx.nonce);
    expect(p.get("code_challenge_method")).toBe("S256");
    expect(p.get("code_challenge")).toBe(createHash("sha256").update(tx.code_verifier).digest("base64url"));
    expect(url.href).not.toContain(tx.code_verifier);
    expect(tx.return_to).toBe("/antrean");
  });

  test("penukaran kode: klien rahasia lewat Basic, PKCE verifier, redirect URI dari konfigurasi", async () => {
    const { url, tx } = await client.beginLogin("/");
    const search = op.authorize(url);
    const t = await client.completeLogin(search, tx);
    expect(t.sub).toBe("f47ac10b-58cc-4372-a567-0e02b2c3d479");
    expect(t.access_token).toMatch(/^at-/);
    expect(t.refresh_token).toMatch(/^rt-/);
    expect(t.access_expires_at).toBeGreaterThan(Date.now() + 290_000);
  });

  test("state berbeda (CSRF login) → ditolak tanpa menukar kode", async () => {
    const { url, tx } = await client.beginLogin("/");
    const search = op.authorize(url);
    const sebelum = op.calls.token_code;
    await expect(client.completeLogin(search, { ...tx, state: "state-lain" })).rejects.toBeInstanceOf(OidcError);
    expect(op.calls.token_code).toBe(sebelum);
  });

  test("nonce berbeda → ditolak", async () => {
    const { url, tx } = await client.beginLogin("/");
    const search = op.authorize(url);
    await expect(client.completeLogin(search, { ...tx, nonce: "nonce-lain" })).rejects.toMatchObject({
      kind: "rejected",
    });
  });

  test("verifier PKCE lain → invalid_grant; kode tidak dapat dipakai dua kali", async () => {
    const { url, tx } = await client.beginLogin("/");
    const search = op.authorize(url);
    await expect(client.completeLogin(search, { ...tx, code_verifier: "x".repeat(43) })).rejects.toMatchObject({
      kind: "invalid_grant",
    });
    await expect(client.completeLogin(search, tx)).rejects.toMatchObject({ kind: "invalid_grant" });
  });

  test("galat dari Keycloak di callback (error=access_denied) → ditolak", async () => {
    const { tx } = await client.beginLogin("/");
    const search = `?${new URLSearchParams({ error: "access_denied", state: tx.state }).toString()}`;
    await expect(client.completeLogin(search, tx)).rejects.toMatchObject({ kind: "rejected" });
  });

  test("penyegaran: token penyegar dirotasi, token lama tidak berlaku lagi (invalid_grant)", async () => {
    const { url, tx } = await client.beginLogin("/");
    const t1 = await client.completeLogin(op.authorize(url), tx);
    const t2 = await client.refresh(t1);
    expect(t2.refresh_token).not.toBe(t1.refresh_token);
    expect(t2.access_token).not.toBe(t1.access_token);
    expect(t2.sub).toBe(t1.sub);
    await expect(client.refresh(t1)).rejects.toMatchObject({ kind: "invalid_grant" });
  });

  test("sesi SSO dicabut → penyegaran invalid_grant", async () => {
    const { url, tx } = await client.beginLogin("/");
    const t1 = await client.completeLogin(op.authorize(url), tx);
    op.revokeAllSessions();
    await expect(client.refresh(t1)).rejects.toMatchObject({ kind: "invalid_grant" });
  });

  test("logout server: token penyegar + Basic; sesi SSO berakhir; logout ulang → invalid_grant", async () => {
    const { url, tx } = await client.beginLogin("/");
    const t1 = await client.completeLogin(op.authorize(url), tx);
    const n = op.sessions.size;
    await client.logout(t1);
    expect(op.sessions.size).toBe(n - 1);
    await expect(client.refresh(t1)).rejects.toMatchObject({ kind: "invalid_grant" });
    await expect(client.logout(t1)).rejects.toMatchObject({ kind: "invalid_grant" });
  });

  test("Keycloak tidak terjangkau → unavailable; issuer http non-loopback → ditolak", async () => {
    await expect(
      OidcClient.discover({
        issuer: new URL("http://127.0.0.1:9/realms/uji"),
        clientId: "x",
        clientSecret: new Sensitive("y"),
        redirectUri: new URL(REDIRECT),
        timeoutSeconds: 2,
      }),
    ).rejects.toMatchObject({ kind: "unavailable" });
    await expect(
      OidcClient.discover({
        issuer: new URL("http://id.contoh.invalid/realms/uji"),
        clientId: "x",
        clientSecret: new Sensitive("y"),
        redirectUri: new URL(REDIRECT),
      }),
    ).rejects.toMatchObject({ kind: "rejected" });
  });
});
