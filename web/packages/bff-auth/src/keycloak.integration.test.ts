// Test integrasi F1-T41: OidcClient terhadap Keycloak 26.8.2 sungguhan dengan realm `admin` dari
// infra/keycloak/realms/admin.json (klien rahasia `rs-backoffice-bff`, PKCE S256, MFA wajib, token
// penyegar sekali pakai). Kontainer sekali pakai; docker wajib.

import { decodeJwt } from "jose";
import { afterAll, beforeAll, describe, expect, test } from "vitest";
import { OidcClient } from "./oidc.ts";
import { Sensitive } from "./sensitive.ts";
import type { TokenSet } from "./session.ts";
import { BACKOFFICE_REDIRECT, EphemeralKeycloak, loginThroughKeycloak, type TestUser } from "./testing/keycloak.ts";

let kc: EphemeralKeycloak | undefined;
let client: OidcClient;
let user: TestUser;

const keycloak = (): EphemeralKeycloak => {
  if (kc === undefined) {
    throw new Error("Keycloak belum siap");
  }
  return kc;
};

beforeAll(async () => {
  kc = await EphemeralKeycloak.start();
  client = await OidcClient.discover({
    issuer: new URL(`${kc.baseUrl}/realms/admin`),
    clientId: "rs-backoffice-bff",
    clientSecret: new Sensitive(kc.backofficeSecret),
    redirectUri: new URL(BACKOFFICE_REDIRECT),
  });
  user = await kc.createUser();
});

afterAll(() => {
  kc?.stop();
});

async function login(): Promise<TokenSet> {
  const { url, tx } = await client.beginLogin("/");
  return client.completeLogin(await loginThroughKeycloak(keycloak(), url, user), tx);
}

describe("OidcClient × Keycloak 26.8.2 realm admin (trace: ADR-011 butir 7, KEPUTUSAN-75)", () => {
  let t1: TokenSet;

  test("Authorization Code + PKCE klien rahasia: login + MFA → token; sub dan issuer realm admin", async () => {
    t1 = await login();
    const at = decodeJwt(t1.access_token);
    expect(at.iss).toBe(`${keycloak().baseUrl}/realms/admin`);
    expect(at["azp"]).toBe("rs-backoffice-bff");
    expect(t1.sub).toBe(user.id);
    expect(t1.access_expires_at - Date.now()).toBeLessThanOrEqual(300_000);
    expect(await keycloak().userSessionCount(user)).toBe(1);
  });

  test("secret klien salah → penukaran kode ditolak", async () => {
    const salah = await OidcClient.discover({
      issuer: new URL(`${keycloak().baseUrl}/realms/admin`),
      clientId: "rs-backoffice-bff",
      clientSecret: new Sensitive("secret-salah"),
      redirectUri: new URL(BACKOFFICE_REDIRECT),
    });
    const { url, tx } = await salah.beginLogin("/");
    const search = await loginThroughKeycloak(keycloak(), url, user);
    await expect(salah.completeLogin(search, tx)).rejects.toMatchObject({ kind: "rejected" });
  });

  test("penyegaran merotasi token penyegar; token lama ditolak Keycloak (invalid_grant)", async () => {
    const t2 = await client.refresh(t1);
    expect(t2.refresh_token).not.toBe(t1.refresh_token);
    expect(t2.sub).toBe(t1.sub);
    await expect(client.refresh(t1)).rejects.toMatchObject({ kind: "invalid_grant" });
    t1 = t2;
  });

  test("logout dari server mengakhiri sesi Keycloak; token penyegar tidak berlaku lagi", async () => {
    const sebelum = await keycloak().userSessionCount(user);
    await client.logout(t1);
    expect(await keycloak().userSessionCount(user)).toBe(sebelum - 1);
    await expect(client.refresh(t1)).rejects.toMatchObject({ kind: "invalid_grant" });
  });

  test("sesi dicabut admin (Admin REST logout user) → penyegaran invalid_grant", async () => {
    const t = await login();
    await keycloak().logoutUser(user);
    expect(await keycloak().userSessionCount(user)).toBe(0);
    await expect(client.refresh(t)).rejects.toMatchObject({ kind: "invalid_grant" });
  });
});
