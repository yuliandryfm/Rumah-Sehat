import { afterAll, beforeAll, beforeEach, describe, expect, test } from "vitest";
import { loadBffConfig } from "./config.ts";
import { createBffCore, type BffCore } from "./core.ts";
import { createBffRoutes, type BffLogger, type BffRoutes } from "./routes.ts";
import { BffBrowser, FakeApi, ORIGIN, expectNoSecrets, json, type Sent } from "./testing/bff-harness.ts";
import { FakeOp } from "./testing/fake-op.ts";
import { TestClock, testEnv, testSecrets } from "./testing/fixtures.ts";
import { MemoryBackend } from "./testing/memory-backend.ts";

const MENIT = 60_000;
const secrets = testSecrets();
const op = new FakeOp({
  clientId: "rs-backoffice-bff",
  clientSecret: secrets.oidcClientSecret.expose(),
  redirectUri: `${ORIGIN}/auth/callback`,
});
const api = new FakeApi();
const SID = "__Host-rs_backoffice_sid";
const LOGIN = "__Host-rs_backoffice_login";

let clock: TestClock;
let backend: MemoryBackend;
let core: BffCore;
let routes: BffRoutes;
let browser: BffBrowser;
const events: { kind: string; event: string; fields: Readonly<Record<string, string>> }[] = [];
const logger: BffLogger = {
  security: (event, fields) => events.push({ kind: "security", event, fields }),
  error: (event, fields) => events.push({ kind: "error", event, fields }),
};

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
  core = await createBffCore(loadBffConfig(testEnv({ BFF_OIDC_ISSUER: op.issuer, BFF_API_BASE_URL: `${api.url}/` })), {
    secrets,
    backend,
    now: clock.now,
  });
  routes = createBffRoutes(() => Promise.resolve(core), {
    allowedApiPrefixes: ["v1/admin/"],
    upstreamTimeoutMs: 1_000,
    maxBodyBytes: 1024,
    logger,
  });
  browser = new BffBrowser();
  events.length = 0;
  api.next = null;
  op.failRefreshWith = null;
});

/** Login lengkap lewat BFF; mengembalikan token CSRF dari status sesi. */
async function login(returnTo = "/antrean"): Promise<string> {
  const r1 = await browser.send(routes.login.GET, `/auth/login?return_to=${encodeURIComponent(returnTo)}`);
  const location = r1.res.headers.get("location") ?? "";
  const search = op.authorize(new URL(location));
  await browser.send(routes.callback.GET, `/auth/callback${search}`);
  const st = await browser.send(routes.session.GET, "/auth/session");
  return (json(st.body)["data"] as { csrf_token: string }).csrf_token;
}

const bffGet = (path: string, headers: Record<string, string> = {}): Promise<Sent> =>
  browser.send(routes.proxy.GET, path, { headers });

describe("login dan callback (trace: ADR-011 butir 7)", () => {
  test("login → 303 ke Keycloak; cookie transaksi login __Host- HttpOnly Secure SameSite=Lax Path=/ tanpa Domain", async () => {
    const r = await browser.send(routes.login.GET, "/auth/login");
    expect(r.res.status).toBe(303);
    expect(r.res.headers.get("location")?.startsWith(`${op.issuer}/protocol/openid-connect/auth?`)).toBe(true);
    expect(r.res.headers.get("cache-control")).toBe("no-store");
    expect(r.setCookies).toHaveLength(1);
    const c = r.setCookies[0] ?? "";
    expect(c).toMatch(/^__Host-rs_backoffice_login=[A-Za-z0-9_-]{43}; Path=\/; Secure; HttpOnly; SameSite=Lax; Max-Age=600$/);
    expect(c).not.toMatch(/Domain=/i);
  });

  test("callback sukses → cookie sesi __Host- (tanpa Max-Age), cookie login dihapus, 303 ke return_to", async () => {
    const r1 = await browser.send(routes.login.GET, "/auth/login?return_to=%2Fverifikasi%3Fhal%3D2");
    const search = op.authorize(new URL(r1.res.headers.get("location") ?? ""));
    const r2 = await browser.send(routes.callback.GET, `/auth/callback${search}`);
    expect(r2.res.status).toBe(303);
    expect(r2.res.headers.get("location")).toBe("/verifikasi?hal=2");
    expect(r2.setCookies).toContain(`${LOGIN}=; Path=/; Secure; HttpOnly; SameSite=Lax; Max-Age=0`);
    const sesi = r2.setCookies.find((c) => c.startsWith(`${SID}=`)) ?? "";
    expect(sesi).toMatch(/^__Host-rs_backoffice_sid=[A-Za-z0-9_-]{43}; Path=\/; Secure; HttpOnly; SameSite=Lax$/);
    expect(browser.cookies.has(SID)).toBe(true);
    expectNoSecrets(browser.responses, op.issuedSecrets);
  });

  test.each([
    ["//evil.contoh.invalid/x"],
    ["https://evil.contoh.invalid/"],
    ["/\\evil.contoh.invalid"],
    ["javascript:alert(1)"],
    ["/%0d%0aSet-Cookie:x"],
  ])("return_to %s → diganti /", async (rt) => {
    const r1 = await browser.send(routes.login.GET, `/auth/login?return_to=${encodeURIComponent(rt)}`);
    const search = op.authorize(new URL(r1.res.headers.get("location") ?? ""));
    const r2 = await browser.send(routes.callback.GET, `/auth/callback${search}`);
    const loc = r2.res.headers.get("location") ?? "";
    expect(loc.startsWith("/")).toBe(true);
    expect(loc.startsWith("//")).toBe(false);
    expect(new URL(loc, ORIGIN).origin).toBe(ORIGIN);
    expect(loc).not.toMatch(/[\r\n]/);
  });

  test("callback tanpa cookie transaksi login (CSRF login) → tidak ada sesi, kode tidak ditukar", async () => {
    const r1 = await browser.send(routes.login.GET, "/auth/login");
    const search = op.authorize(new URL(r1.res.headers.get("location") ?? ""));
    browser.cookies.delete(LOGIN);
    const n = op.calls.token_code;
    const r2 = await browser.send(routes.callback.GET, `/auth/callback${search}`);
    expect(r2.res.status).toBe(303);
    expect(r2.res.headers.get("location")).toBe("/?auth_error=expired");
    expect(browser.cookies.has(SID)).toBe(false);
    expect(op.calls.token_code).toBe(n);
  });

  test("transaksi login dipakai dua kali → kedua kalinya ditolak", async () => {
    const r1 = await browser.send(routes.login.GET, "/auth/login");
    const loginCookie = browser.cookies.get(LOGIN) ?? "";
    const search = op.authorize(new URL(r1.res.headers.get("location") ?? ""));
    await browser.send(routes.callback.GET, `/auth/callback${search}`);
    const b2 = new BffBrowser();
    b2.cookies.set(LOGIN, loginCookie);
    const r3 = await b2.send(routes.callback.GET, `/auth/callback${search}`);
    expect(r3.res.headers.get("location")).toBe("/?auth_error=expired");
    expect(b2.cookies.has(SID)).toBe(false);
  });

  test("state dari login lain → ditolak (auth_error=failed)", async () => {
    await browser.send(routes.login.GET, "/auth/login");
    const other = await new BffBrowser().send(routes.login.GET, "/auth/login");
    const search = op.authorize(new URL(other.res.headers.get("location") ?? ""));
    const r = await browser.send(routes.callback.GET, `/auth/callback${search}`);
    expect(r.res.headers.get("location")).toBe("/?auth_error=failed");
    expect(browser.cookies.has(SID)).toBe(false);
    expect(events.some((e) => e.event === "bff_login_rejected")).toBe(true);
  });

  test("login ulang mengganti ID sesi dan mengakhiri sesi lama (cegah fiksasi sesi)", async () => {
    await login();
    const lama = browser.cookies.get(SID) ?? "";
    await login();
    const baru = browser.cookies.get(SID) ?? "";
    expect(baru).not.toBe(lama);
    expect((await core.sessions.load(lama)).status).toBe("absent");
  });
});

describe("Route Handler bff/[...path] dan CSRF (trace: ADR-011 butir 7, WEB-04)", () => {
  test("tanpa header CSRF → 403 4030101, API tidak dipanggil", async () => {
    await login();
    const n = api.requests.length;
    const r = await bffGet("/bff/v1/admin/akun");
    expect(r.res.status).toBe(403);
    expect(json(r.body)["response_code"]).toBe("4030101");
    expect(r.res.headers.get("content-type")).toBe("application/problem+json");
    expect(api.requests.length).toBe(n);
    expect(events.some((e) => e.event === "bff_csrf_rejected" && e.fields["reason"] === "token_mismatch")).toBe(true);
  });

  test.each(["POST", "PUT", "PATCH", "DELETE"])("%s tanpa CSRF → 403", async (method) => {
    await login();
    const r = await browser.send(routes.proxy[method as "POST"], "/bff/v1/admin/akun/1", {
      method,
      headers: { "content-type": "application/json" },
      body: "{}",
    });
    expect(r.res.status).toBe(403);
  });

  test("token CSRF salah atau milik sesi lain → 403", async () => {
    await login();
    const other = new BffBrowser();
    const saved = browser;
    browser = other;
    const csrfLain = await login();
    browser = saved;
    expect((await bffGet("/bff/v1/admin/akun", { "x-csrf-token": csrfLain })).res.status).toBe(403);
    expect((await bffGet("/bff/v1/admin/akun", { "x-csrf-token": "A".repeat(43) })).res.status).toBe(403);
  });

  test("token benar tetapi Origin asing atau Sec-Fetch-Site cross-site → 403", async () => {
    const csrf = await login();
    const a = await browser.send(routes.proxy.POST, "/bff/v1/admin/akun", {
      method: "POST",
      headers: { "x-csrf-token": csrf, origin: "https://evil.contoh.invalid" },
      body: "{}",
    });
    expect(a.res.status).toBe(403);
    const b = await bffGet("/bff/v1/admin/akun", { "x-csrf-token": csrf, "sec-fetch-site": "cross-site" });
    expect(b.res.status).toBe(403);
    const c = await bffGet("/bff/v1/admin/akun", { "x-csrf-token": csrf, origin: "null" });
    expect(c.res.status).toBe(403);
  });

  test("dengan CSRF → diteruskan dengan Bearer di server; cookie dan CSRF tidak diteruskan; token tidak sampai browser", async () => {
    const csrf = await login();
    const r = await browser.send(routes.proxy.POST, "/bff/v1/admin/akun?hal=2", {
      method: "POST",
      headers: {
        "x-csrf-token": csrf,
        origin: ORIGIN,
        "sec-fetch-site": "same-origin",
        "content-type": "application/json",
        "idempotency-key": "0193a1b2-0000-7000-8000-000000000001",
        authorization: "Bearer palsu-dari-browser",
        "x-correlation-id": "0193a1b2-0000-7000-8000-0000000000aa",
      },
      body: '{"nama":"uji"}',
    });
    expect(r.res.status).toBe(200);
    const up = api.last();
    expect(up?.method).toBe("POST");
    expect(up?.path).toBe("/v1/admin/akun?hal=2");
    expect(up?.body).toBe('{"nama":"uji"}');
    expect(up?.headers.authorization).toMatch(/^Bearer at-/);
    expect(op.issuedSecrets).toContain(up?.headers.authorization?.slice(7));
    expect(up?.headers.cookie).toBeUndefined();
    expect(up?.headers["x-csrf-token"]).toBeUndefined();
    expect(up?.headers["idempotency-key"]).toBe("0193a1b2-0000-7000-8000-000000000001");
    expect(up?.headers["x-correlation-id"]).toBe("0193a1b2-0000-7000-8000-0000000000aa");
    expect(r.res.headers.get("cache-control")).toBe("no-store");
    expectNoSecrets(browser.responses, op.issuedSecrets);
  });

  test("Set-Cookie dan Location dari API tidak diteruskan ke browser", async () => {
    const csrf = await login();
    api.next = {
      status: 200,
      headers: { "content-type": "application/json", "set-cookie": "jahat=1; Path=/", location: "https://evil.contoh.invalid" },
      body: "{}",
    };
    const r = await bffGet("/bff/v1/admin/akun", { "x-csrf-token": csrf });
    expect(r.setCookies).toEqual([]);
    expect(r.res.headers.get("location")).toBeNull();
  });

  test.each([
    "/bff/v1/admin/../internal/x",
    "/bff/v1/admin/%2e%2e/x",
    "/bff/v1/admin/a%2Fb",
    "/bff/v1/admin/",
    "/bff/v1/partner/akun",
    "/bff/v1/public/x",
    "/bff/v1//admin/x",
    "/bff/http:/evil",
    "/lain/v1/admin/akun",
  ])("path %s → 404 tanpa diteruskan", async (path) => {
    const csrf = await login();
    const n = api.requests.length;
    const r = await bffGet(path, { "x-csrf-token": csrf });
    expect(r.res.status).toBe(404);
    expect(json(r.body)["response_code"]).toBe("4040001");
    expect(api.requests.length).toBe(n);
  });

  test("badan lebih dari batas → 413 tanpa diteruskan", async () => {
    const csrf = await login();
    const n = api.requests.length;
    const r = await browser.send(routes.proxy.POST, "/bff/v1/admin/akun", {
      method: "POST",
      headers: { "x-csrf-token": csrf, "content-type": "application/json" },
      body: "x".repeat(2048),
    });
    expect(r.res.status).toBe(413);
    expect(api.requests.length).toBe(n);
  });

  test("API Rust tidak merespons dalam batas waktu → 503 5030001", async () => {
    const csrf = await login();
    api.next = { delayMs: 3_000 };
    const r = await bffGet("/bff/v1/admin/akun", { "x-csrf-token": csrf });
    expect(r.res.status).toBe(503);
    expect(json(r.body)["response_code"]).toBe("5030001");
  });

  test("envelope galat API Rust diteruskan apa adanya", async () => {
    const csrf = await login();
    const envelope = '{"type":"x","response_code":"4040001","response_description":"Data tidak ditemukan","correlation_id":"c"}';
    api.next = { status: 404, headers: { "content-type": "application/problem+json" }, body: envelope };
    const r = await bffGet("/bff/v1/admin/akun/9", { "x-csrf-token": csrf });
    expect(r.res.status).toBe(404);
    expect(r.body).toBe(envelope);
  });
});

describe("batas sesi dan pencabutan (trace: ADR-011 butir 7-8, NFR-SEC-04, KU-10)", () => {
  test("tanpa sesi → 401 4010001", async () => {
    const r = await bffGet("/bff/v1/admin/akun", { "x-csrf-token": "A".repeat(43) });
    expect(r.res.status).toBe(401);
    expect(json(r.body)["response_code"]).toBe("4010001");
  });

  test("tidak aktif 30 menit → 401, cookie dihapus, sesi server terhapus", async () => {
    const csrf = await login();
    const sid = browser.cookies.get(SID) ?? "";
    clock.advance(30 * MENIT);
    const r = await bffGet("/bff/v1/admin/akun", { "x-csrf-token": csrf });
    expect(r.res.status).toBe(401);
    expect(r.setCookies).toContain(`${SID}=; Path=/; Secure; HttpOnly; SameSite=Lax; Max-Age=0`);
    expect(backend.keys().some((k) => k === core.sessions.key("s", sid))).toBe(false);
    expect(events.some((e) => e.event === "bff_session_expired" && e.fields["reason"] === "idle")).toBe(true);
  });

  test("permintaan BFF dihitung aktivitas; status sesi tidak", async () => {
    const csrf = await login();
    clock.advance(20 * MENIT);
    await bffGet("/bff/v1/admin/akun", { "x-csrf-token": csrf });
    clock.advance(20 * MENIT);
    const st = await browser.send(routes.session.GET, "/auth/session");
    expect(st.res.status).toBe(200);
    clock.advance(9 * MENIT);
    await browser.send(routes.session.GET, "/auth/session");
    clock.advance(MENIT);
    expect((await browser.send(routes.session.GET, "/auth/session")).res.status).toBe(401);
  });

  test("batas mutlak 12 jam walau terus aktif", async () => {
    const csrf = await login();
    for (let i = 0; i < 47; i++) {
      clock.advance(15 * MENIT);
      expect((await bffGet("/bff/v1/admin/akun", { "x-csrf-token": csrf })).res.status).toBe(200);
    }
    clock.advance(15 * MENIT);
    expect((await bffGet("/bff/v1/admin/akun", { "x-csrf-token": csrf })).res.status).toBe(401);
  });

  test("sesi dicabut di server (keluar dari tab lain) → 401", async () => {
    const csrf = await login();
    await core.sessions.destroy(browser.cookies.get(SID) ?? "");
    expect((await bffGet("/bff/v1/admin/akun", { "x-csrf-token": csrf })).res.status).toBe(401);
  });

  test("sesi Keycloak dicabut → penyegaran invalid_grant → 401 dan sesi BFF terhapus", async () => {
    const csrf = await login();
    op.revokeAllSessions();
    clock.advance(5 * MENIT);
    const r = await bffGet("/bff/v1/admin/akun", { "x-csrf-token": csrf });
    expect(r.res.status).toBe(401);
    expect((await core.sessions.load(browser.cookies.get(SID))).status).toBe("absent");
    expect(events.some((e) => e.event === "bff_session_revoked" && e.fields["reason"] === "refresh_rejected")).toBe(true);
  });

  test("API Rust menjawab 401 (akun nonaktif/sesi dicabut, KU-10) → sesi BFF diakhiri", async () => {
    const csrf = await login();
    api.next = { status: 401, headers: { "content-type": "application/problem+json" }, body: '{"response_code":"4010001"}' };
    const r = await bffGet("/bff/v1/admin/akun", { "x-csrf-token": csrf });
    expect(r.res.status).toBe(401);
    expect(r.setCookies.some((c) => c.startsWith(`${SID}=;`))).toBe(true);
    api.next = null;
    expect((await bffGet("/bff/v1/admin/akun", { "x-csrf-token": csrf })).res.status).toBe(401);
  });

  test("kunci sesi diganti tanpa kunci lama → token tidak terbuka → 401 dan sesi dihapus (bukan 503 berulang)", async () => {
    const csrf = await login();
    const sid = browser.cookies.get(SID) ?? "";
    const core2 = await createBffCore(core.config, {
      secrets: testSecrets({ oidcClientSecret: secrets.oidcClientSecret }),
      backend,
      now: clock.now,
    });
    const routes2 = createBffRoutes(() => Promise.resolve(core2), { allowedApiPrefixes: ["v1/admin/"], logger });
    const n = api.requests.length;
    const r = await browser.send(routes2.proxy.GET, "/bff/v1/admin/akun", { headers: { "x-csrf-token": csrf } });
    expect(r.res.status).toBe(401);
    expect(api.requests.length).toBe(n);
    expect((await core2.sessions.load(sid)).status).toBe("absent");
  });

  test("Keycloak tidak terjangkau saat penyegaran → 503, sesi dipertahankan", async () => {
    const csrf = await login();
    op.failRefreshWith = { status: 500, error: "server_error" };
    clock.advance(5 * MENIT);
    expect((await bffGet("/bff/v1/admin/akun", { "x-csrf-token": csrf })).res.status).toBe(503);
    op.failRefreshWith = null;
    expect((await bffGet("/bff/v1/admin/akun", { "x-csrf-token": csrf })).res.status).toBe(200);
  });

  test("token akses hampir kedaluwarsa + permintaan serentak → satu penyegaran, semua berhasil", async () => {
    const csrf = await login();
    clock.advance(5 * MENIT);
    const n = op.calls.token_refresh;
    const rs = await Promise.all(
      Array.from({ length: 5 }, () => bffGet("/bff/v1/admin/akun", { "x-csrf-token": csrf })),
    );
    expect(rs.map((r) => r.res.status)).toEqual([200, 200, 200, 200, 200]);
    expect(op.calls.token_refresh).toBe(n + 1);
    const auth = new Set(api.requests.slice(-5).map((q) => q.headers.authorization));
    expect(auth.size).toBe(1);
    expectNoSecrets(browser.responses, op.issuedSecrets);
  });
});

describe("status sesi, Tetap masuk, dan keluar (trace: KEPUTUSAN-170, US-103 AC5)", () => {
  test("status sesi: batas tidak aktif/mutlak, waktu server, peringatan 120 detik, token CSRF; tanpa token OIDC", async () => {
    await login();
    clock.advance(10 * MENIT);
    const st = await browser.send(routes.session.GET, "/auth/session");
    const d = json(st.body)["data"] as Record<string, unknown>;
    expect(Date.parse(String(d["idle_expires_at"])) - clock.now()).toBe(20 * MENIT);
    expect(d["warning_seconds"]).toBe(120);
    expect(String(d["csrf_token"])).toMatch(/^[A-Za-z0-9_-]{43}$/);
    expect(Object.keys(d).sort()).toEqual(
      ["absolute_expires_at", "csrf_token", "idle_expires_at", "server_time", "warning_seconds"].sort(),
    );
    expectNoSecrets(browser.responses, op.issuedSecrets);
  });

  test("Tetap masuk tanpa CSRF → 403; dengan CSRF → token disegarkan dan batas tidak aktif diperpanjang", async () => {
    const csrf = await login();
    clock.advance(28 * MENIT);
    expect((await browser.send(routes.extend.POST, "/auth/session/extend", { method: "POST" })).res.status).toBe(403);
    const n = op.calls.token_refresh;
    const r = await browser.send(routes.extend.POST, "/auth/session/extend", {
      method: "POST",
      headers: { "x-csrf-token": csrf },
    });
    expect(r.res.status).toBe(200);
    expect(op.calls.token_refresh).toBe(n + 1);
    const d = json(r.body)["data"] as Record<string, unknown>;
    expect(Date.parse(String(d["idle_expires_at"])) - clock.now()).toBe(30 * MENIT);
    clock.advance(29 * MENIT);
    expect((await browser.send(routes.session.GET, "/auth/session")).res.status).toBe(200);
    expectNoSecrets(browser.responses, op.issuedSecrets);
  });

  test("Tetap masuk setelah sesi Keycloak dicabut → 401 dan sesi BFF terhapus", async () => {
    const csrf = await login();
    op.revokeAllSessions();
    const r = await browser.send(routes.extend.POST, "/auth/session/extend", {
      method: "POST",
      headers: { "x-csrf-token": csrf },
    });
    expect(r.res.status).toBe(401);
    expect((await core.sessions.load(browser.cookies.get(SID))).status).toBe("absent");
  });

  test("keluar tanpa CSRF → 403 dan sesi tetap", async () => {
    await login();
    const r = await browser.send(routes.logout.POST, "/auth/logout", { method: "POST" });
    expect(r.res.status).toBe(403);
    expect((await core.sessions.load(browser.cookies.get(SID))).status).toBe("active");
  });

  test("keluar lewat formulir (csrf_token): sesi server dan Keycloak diakhiri, cookie dihapus, 303 ke /", async () => {
    const csrf = await login();
    const sid = browser.cookies.get(SID) ?? "";
    const ssoSebelum = op.sessions.size;
    const r = await browser.send(routes.logout.POST, "/auth/logout", {
      method: "POST",
      headers: { "content-type": "application/x-www-form-urlencoded", origin: ORIGIN },
      body: `csrf_token=${csrf}`,
    });
    expect(r.res.status).toBe(303);
    expect(r.res.headers.get("location")).toBe("/");
    expect(r.setCookies).toContain(`${SID}=; Path=/; Secure; HttpOnly; SameSite=Lax; Max-Age=0`);
    expect(op.sessions.size).toBe(ssoSebelum - 1);
    expect((await core.sessions.load(sid)).status).toBe("absent");
    // Cookie lama yang disimpan penyerang tidak berlaku lagi.
    const b2 = new BffBrowser();
    b2.cookies.set(SID, sid);
    expect((await b2.send(routes.proxy.GET, "/bff/v1/admin/akun", { headers: { "x-csrf-token": csrf } })).res.status).toBe(401);
    expectNoSecrets(browser.responses, op.issuedSecrets);
  });

  test("keluar saat Keycloak gagal → sesi server tetap diakhiri dan kejadian keamanan dicatat", async () => {
    const csrf = await login();
    const sid = browser.cookies.get(SID) ?? "";
    op.revokeAllSessions();
    await op.stop();
    try {
      const r = await browser.send(routes.logout.POST, "/auth/logout", {
        method: "POST",
        headers: { "x-csrf-token": csrf },
      });
      expect(r.res.status).toBe(303);
      expect((await core.sessions.load(sid)).status).toBe("absent");
      expect(events.some((e) => e.event === "bff_keycloak_logout_failed")).toBe(true);
    } finally {
      await op.restart();
    }
  });

  test("kejadian log tidak memuat token, ID sesi, atau token CSRF", async () => {
    const csrf = await login();
    const sid = browser.cookies.get(SID) ?? "";
    await bffGet("/bff/v1/admin/akun");
    clock.advance(31 * MENIT);
    await bffGet("/bff/v1/admin/akun", { "x-csrf-token": csrf });
    const text = JSON.stringify(events);
    expect(events.length).toBeGreaterThan(0);
    for (const s of [...op.issuedSecrets, sid, csrf]) {
      expect(text.includes(s)).toBe(false);
    }
  });
});

describe("infrastruktur tidak tersedia", () => {
  test("inti BFF gagal dibuat (OpenBao/Valkey) → 503 tanpa rincian", async () => {
    const r2 = createBffRoutes(() => Promise.reject(new Error("rahasia-bocor")), { allowedApiPrefixes: ["v1/admin/"], logger });
    const r = await browser.send(r2.session.GET, "/auth/session");
    expect(r.res.status).toBe(503);
    expect(r.body).not.toContain("rahasia-bocor");
  });

  test("allowedApiPrefixes kosong atau tidak sah → ditolak saat konfigurasi", () => {
    expect(() => createBffRoutes(() => Promise.resolve(core), { allowedApiPrefixes: [] })).toThrow(RangeError);
    expect(() => createBffRoutes(() => Promise.resolve(core), { allowedApiPrefixes: ["/v1"] })).toThrow(RangeError);
  });
});
