import { afterAll, beforeAll, beforeEach, describe, expect, test } from "vitest";
import { cookieNames, createBffCore, createBffRoutes, loadBffConfig, type BffCore, type BffRoutes } from "@rs/bff-auth";
import { BffBrowser, FakeApi, ORIGIN, expectNoSecrets, json } from "../../packages/bff-auth/src/testing/bff-harness.ts";
import { FakeOp } from "../../packages/bff-auth/src/testing/fake-op.ts";
import { TestClock, testEnv, testSecrets } from "../../packages/bff-auth/src/testing/fixtures.ts";
import { MemoryBackend } from "../../packages/bff-auth/src/testing/memory-backend.ts";
import { BFF_APP, OPSI_ROUTES } from "./lib/server/bff.ts";

/**
 * F1-T90: perilaku BFF korporat dengan opsi RUNTIME (`OPSI_ROUTES`) terhadap authorization server tiruan
 * dan API Rust tiruan. Integrasi dengan Keycloak sungguhan realm `korporat` menunggu F1-T85 (belum ada di
 * helper); `keycloak.integration.test.ts` paket bff-auth memverifikasi alur yang sama terhadap realm `admin`.
 */
const MENIT = 60_000;
const CLIENT_ID = "rs-korporat-bff";
const secrets = testSecrets();
const op = new FakeOp({ clientId: CLIENT_ID, clientSecret: secrets.oidcClientSecret.expose(), redirectUri: `${ORIGIN}/auth/callback` });
const api = new FakeApi();
const { session: SID } = cookieNames(BFF_APP);

let clock: TestClock;
let backend: MemoryBackend;
let core: BffCore;
let routes: BffRoutes;
let browser: BffBrowser;

beforeAll(async () => {
  await op.start();
  await api.start();
});
afterAll(async () => {
  await op.stop();
  await api.stop();
});
beforeEach(async () => {
  clock = new TestClock(Date.now());
  backend = new MemoryBackend(clock.now);
  core = await createBffCore(
    loadBffConfig(
      testEnv({
        BFF_APP_NAME: BFF_APP,
        BFF_OIDC_ISSUER: op.issuer,
        BFF_OIDC_CLIENT_ID: CLIENT_ID,
        BFF_API_BASE_URL: `${api.url}/`,
        BFF_OPENBAO_KV_PATH: "rs/bff/korporat",
      }),
    ),
    { secrets, backend, now: clock.now },
  );
  routes = createBffRoutes(() => Promise.resolve(core), OPSI_ROUTES);
  browser = new BffBrowser();
  api.requests.length = 0;
  api.next = null;
});

async function login(): Promise<string> {
  const r1 = await browser.send(routes.login.GET, "/auth/login");
  const search = op.authorize(new URL(r1.res.headers.get("location") ?? ""));
  await browser.send(routes.callback.GET, `/auth/callback${search}`);
  const st = await browser.send(routes.session.GET, "/auth/session");
  return (json(st.body)["data"] as { csrf_token: string }).csrf_token;
}

const ambil = (csrf: string, path = "/bff/v1/corporate/profil") =>
  browser.send(routes.proxy.GET, path, { headers: { "x-csrf-token": csrf } });

describe("BFF korporat (ADR-027, ADR-011 butir 7)", () => {
  test("cookie sesi __Host-rs_korporat_sid: Secure, HttpOnly, SameSite=Lax, Path=/, tanpa Domain", async () => {
    await login();
    expect(SID).toBe("__Host-rs_korporat_sid");
    const sesi = browser.responses.flatMap((r) => r.setCookies).find((c) => c.startsWith(`${SID}=`)) ?? "";
    expect(sesi).toMatch(/^__Host-rs_korporat_sid=[A-Za-z0-9_-]{43}; Path=\/; Secure; HttpOnly; SameSite=Lax$/);
  });

  test("API korporat diteruskan dengan token sesi, token tidak sampai browser", async () => {
    const csrf = await login();
    const r = await ambil(csrf);
    expect(r.res.status).toBe(200);
    const up = api.requests.at(-1);
    expect(up?.path).toBe("/v1/corporate/profil");
    expect(up?.headers.authorization).toMatch(/^Bearer /);
    expectNoSecrets(browser.responses, op.issuedSecrets);
  });

  test.each([
    "/bff/v1/admin/akun",
    "/bff/backoffice/v1/konten",
    "/bff/v1/backoffice/konten",
    "/bff/v1/partner/akun",
    "/bff/v1/portal/akun",
    "/bff/v1/patient/profil",
    "/bff/v1/public/content/x",
    "/bff/v1/corporate/",
    "/bff/v1/corporate/../admin/akun",
    "/bff/v1/corporate/%2e%2e/admin/akun",
    "/bff/v1/corporates/x",
    "/bff/v1//corporate/x",
  ])("negatif: sesi korporat tidak dapat memanggil %s → 404 tanpa diteruskan", async (path) => {
    const csrf = await login();
    const n = api.requests.length;
    const r = await ambil(csrf, path);
    expect(r.res.status).toBe(404);
    expect(api.requests.length).toBe(n);
  });

  test("negatif: metode ubah tanpa atau dengan token CSRF salah ditolak dan tidak diteruskan", async () => {
    await login();
    const n = api.requests.length;
    const dasar = { method: "POST", body: "{}" };
    const r = await browser.send(routes.proxy.POST, "/bff/v1/corporate/hr", {
      ...dasar,
      headers: { origin: ORIGIN, "content-type": "application/json" },
    });
    expect([401, 403]).toContain(r.res.status);
    const r2 = await browser.send(routes.proxy.POST, "/bff/v1/corporate/hr", {
      ...dasar,
      headers: { origin: ORIGIN, "content-type": "application/json", "x-csrf-token": "A".repeat(43) },
    });
    expect([401, 403]).toContain(r2.res.status);
    expect(api.requests.length).toBe(n);
  });

  test("negatif: tanpa sesi → 401 dan tidak diteruskan", async () => {
    const r = await ambil("A".repeat(43));
    expect(r.res.status).toBe(401);
    expect(api.requests).toHaveLength(0);
  });

  test("tidak aktif 30 menit → 401 dan sesi server terhapus", async () => {
    const csrf = await login();
    const sid = browser.cookies.get(SID) ?? "";
    clock.advance(30 * MENIT);
    const r = await ambil(csrf);
    expect(r.res.status).toBe(401);
    expect(backend.keys().some((k) => k === core.sessions.key("s", sid))).toBe(false);
  });

  test("batas mutlak 12 jam: aktivitas tiap 29 menit tidak memperpanjang melewati 12 jam", async () => {
    const csrf = await login();
    const sid = browser.cookies.get(SID) ?? "";
    for (let i = 0; i < 24; i++) {
      clock.advance(29 * MENIT);
      expect((await ambil(csrf)).res.status).toBe(200);
    }
    clock.advance(29 * MENIT);
    const r = await ambil(csrf);
    expect(r.res.status).toBe(401);
    expect(backend.keys().some((k) => k === core.sessions.key("s", sid))).toBe(false);
  }, 30_000);

  test("galat login mendarat di /masuk-gagal, bukan halaman terautentikasi (pelajaran review F1-T43)", async () => {
    expect(OPSI_ROUTES.loginErrorPath).toBe("/masuk-gagal");
    const r = await browser.send(routes.callback.GET, "/auth/callback?code=x&state=salah");
    expect(r.res.status).toBe(303);
    expect(r.res.headers.get("location")).toBe("/masuk-gagal?auth_error=expired");
  });
});
