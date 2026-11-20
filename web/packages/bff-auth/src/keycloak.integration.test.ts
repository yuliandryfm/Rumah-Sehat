// Test integrasi F1-T41/F1-T42: OidcClient dan Route Handler BFF terhadap Keycloak 26.8.2 sungguhan dengan
// realm `admin` dari infra/keycloak/realms/admin.json (klien rahasia `rs-backoffice-bff`, PKCE S256, MFA
// wajib, token penyegar sekali pakai). Kontainer sekali pakai; docker wajib.

import { decodeJwt } from "jose";
import { afterAll, beforeAll, describe, expect, test } from "vitest";
import { loadBffConfig } from "./config.ts";
import { createBffCore, type BffCore } from "./core.ts";
import { OidcClient } from "./oidc.ts";
import { createBffRoutes, type BffRoutes } from "./routes.ts";
import { Sensitive } from "./sensitive.ts";
import type { Session, TokenSet } from "./session.ts";
import { BffBrowser, FakeApi, expectNoSecrets, json } from "./testing/bff-harness.ts";
import { testEnv, testSecrets } from "./testing/fixtures.ts";
import { BACKOFFICE_REDIRECT, EphemeralKeycloak, loginThroughKeycloak, type TestUser } from "./testing/keycloak.ts";
import { MemoryBackend } from "./testing/memory-backend.ts";

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

describe("Route Handler BFF × Keycloak 26.8.2 (trace: ADR-011 butir 7, US-103 AC5, F1-T42)", () => {
  const api = new FakeApi();
  let routes: BffRoutes;
  let core: BffCore;
  let browser: BffBrowser;

  beforeAll(async () => {
    await api.start();
    const k = keycloak();
    core = await createBffCore(
      loadBffConfig(
        testEnv({
          BFF_OIDC_ISSUER: `${k.baseUrl}/realms/admin`,
          BFF_OIDC_REDIRECT_URI: BACKOFFICE_REDIRECT,
          BFF_API_BASE_URL: api.url,
        }),
      ),
      { secrets: testSecrets({ oidcClientSecret: new Sensitive(k.backofficeSecret) }), backend: new MemoryBackend(Date.now) },
    );
    routes = createBffRoutes(() => Promise.resolve(core), {
      allowedApiPrefixes: ["v1/admin/"],
      logger: { security: () => undefined, error: () => undefined },
    });
  });

  afterAll(async () => {
    await api.stop();
  });

  async function bffLogin(): Promise<string> {
    browser = new BffBrowser();
    const r1 = await browser.send(routes.login.GET, "/auth/login?return_to=%2Fantrean");
    const search = await loginThroughKeycloak(keycloak(), new URL(r1.res.headers.get("location") ?? ""), user);
    const r2 = await browser.send(routes.callback.GET, `/auth/callback${search}`);
    expect(r2.res.headers.get("location")).toBe("/antrean");
    const st = await browser.send(routes.session.GET, "/auth/session");
    return (json(st.body)["data"] as { csrf_token: string }).csrf_token;
  }

  test("login lewat BFF → API menerima JWT realm admin dari server; browser tidak menerima token apa pun", async () => {
    const csrf = await bffLogin();
    const r = await browser.send(routes.proxy.GET, "/bff/v1/admin/akun", { headers: { "x-csrf-token": csrf } });
    expect(r.res.status).toBe(200);
    const bearer = api.last()?.headers.authorization?.slice(7) ?? "";
    const claims = decodeJwt(bearer);
    expect(claims.iss).toBe(`${keycloak().baseUrl}/realms/admin`);
    expect(claims["azp"]).toBe("rs-backoffice-bff");
    const tokens = await core.sessions.tokens(
      ((await core.sessions.load(browser.cookies.get("__Host-rs_backoffice_sid"))) as { session: Session }).session,
    );
    expectNoSecrets(browser.responses, [tokens.access_token, tokens.refresh_token, tokens.id_token ?? "", bearer]);
  });

  test("tanpa CSRF → 403 dan API tidak dipanggil", async () => {
    await bffLogin();
    const n = api.requests.length;
    expect((await browser.send(routes.proxy.GET, "/bff/v1/admin/akun")).res.status).toBe(403);
    expect(api.requests.length).toBe(n);
  });

  test("keluar: sesi BFF dan sesi Keycloak berakhir", async () => {
    await keycloak().logoutUser(user);
    const csrf = await bffLogin();
    expect(await keycloak().userSessionCount(user)).toBe(1);
    const r = await browser.send(routes.logout.POST, "/auth/logout", { method: "POST", headers: { "x-csrf-token": csrf } });
    expect(r.res.status).toBe(303);
    expect(await keycloak().userSessionCount(user)).toBe(0);
    expect((await browser.send(routes.session.GET, "/auth/session")).res.status).toBe(401);
  });

  test("sesi dicabut admin Keycloak → 'Tetap masuk' 401 dan sesi BFF terhapus", async () => {
    const csrf = await bffLogin();
    const sid = browser.cookies.get("__Host-rs_backoffice_sid");
    await keycloak().logoutUser(user);
    const r = await browser.send(routes.extend.POST, "/auth/session/extend", {
      method: "POST",
      headers: { "x-csrf-token": csrf },
    });
    expect(r.res.status).toBe(401);
    expect((await core.sessions.load(sid)).status).toBe("absent");
  });
});
