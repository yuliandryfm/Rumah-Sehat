import { afterAll, beforeAll, beforeEach, describe, expect, test } from "vitest";
import { cookieNames, createBffCore, createBffRoutes, loadBffConfig, type BffCore, type BffRoutes } from "@rs/bff-auth";
import { BffBrowser, FakeApi, ORIGIN, expectNoSecrets, json } from "../../packages/bff-auth/src/testing/bff-harness.ts";
import { FakeOp } from "../../packages/bff-auth/src/testing/fake-op.ts";
import { TestClock, testEnv, testSecrets } from "../../packages/bff-auth/src/testing/fixtures.ts";
import { MemoryBackend } from "../../packages/bff-auth/src/testing/memory-backend.ts";
import { BFF_APP, OPSI_ROUTES } from "./lib/server/bff.ts";
import { buatKlienHr, type KlienHr } from "./lib/hr/klien.ts";

/**
 * F1-T108 e2e sisi server: klien HR yang sama dengan yang dipakai browser (openapi-fetch dari
 * `@rs/api-client`) memanggil `/bff/v1/corporate/members...` lewat Route Handler BFF `korporat`
 * (cookie sesi, CSRF, proxy, kebijakan sesi) ke API Rust tiruan. Tanpa browser sungguhan dan tanpa
 * Keycloak realm `korporat` sungguhan (menunggu F1-T85/F1-T110); layar yang memetakan galat ke tampilan
 * diuji di `komponen/hr/hr.test.tsx`. Jam virtual, jadi idle 30 menit diuji tanpa menunggu.
 */
const MENIT = 60_000;
const CLIENT_ID = "rs-korporat-bff";
const ID_B = "0193a1b2-c3d4-7e5f-8a6b-7c8d9e0f1a2c";
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

async function masuk(): Promise<KlienHr> {
  const r1 = await browser.send(routes.login.GET, "/auth/login");
  const search = op.authorize(new URL(r1.res.headers.get("location") ?? ""));
  await browser.send(routes.callback.GET, `/auth/callback${search}`);
  const st = await browser.send(routes.session.GET, "/auth/session");
  const csrf = (json(st.body)["data"] as { csrf_token: string }).csrf_token;
  // Jembatan fetch: permintaan klien diteruskan ke Route Handler BFF seperti dari browser same-origin.
  const jembatan = async (r: Request): Promise<Response> => {
    const u = new URL(r.url);
    const handler = routes.proxy[r.method as "GET" | "POST"];
    const headers: Record<string, string> = { "x-csrf-token": r.headers.get("x-csrf-token") ?? "" };
    if (r.method !== "GET") headers["origin"] = ORIGIN;
    const sent = await browser.send(handler, `${u.pathname}${u.search}`, { method: r.method, headers });
    return sent.res.status === 204 ? new Response(null, { status: 204 }) : new Response(sent.body, { status: sent.res.status, headers: sent.res.headers });
  };
  return buatKlienHr(csrf, { basis: `${ORIGIN}/bff`, fetch: jembatan as typeof fetch });
}

const amplop = (data: unknown) => JSON.stringify({ response_code: "2000000", response_description: "Berhasil", data });
const galatApi = (status: number, kode: string) => ({
  status,
  headers: { "content-type": "application/problem+json" },
  body: JSON.stringify({ type: `https://problems.ruang-sehat.invalid/${kode}`, response_code: kode, response_description: "x", correlation_id: "c" }),
});
const sesiAda = (): boolean => browser.cookies.has(SID);

describe("e2e web korporat: kelola HR lewat BFF (US-1904)", () => {
  test("daftar dan nonaktifkan diteruskan ke /v1/corporate/... dengan token sesi; token tidak sampai browser", async () => {
    const klien = await masuk();
    api.next = { body: amplop([{ corporate_member_id: ID_B, email: "budi@perusahaan.example", status: "active", is_self: false }]) };
    const d = await klien.daftar();
    expect(d.ok && d.data[0]?.email).toBe("budi@perusahaan.example");
    expect(api.last()?.path).toBe("/v1/corporate/members");
    expect(api.last()?.headers.authorization).toMatch(/^Bearer /);

    api.next = { body: amplop({ corporate_member_id: ID_B, status: "inactive", identity_provider_synced: true }) };
    const n = await klien.nonaktifkan(ID_B);
    expect(n.ok).toBe(true);
    expect(api.last()?.method).toBe("POST");
    expect(api.last()?.path).toBe(`/v1/corporate/members/${ID_B}/deactivate`);
    expectNoSecrets(browser.responses, op.issuedSecrets);
  });

  test("negatif: HR aktif terakhir (409 4090105) sampai ke klien utuh dan sesi tetap hidup", async () => {
    const klien = await masuk();
    api.next = galatApi(409, "4090105");
    const n = await klien.nonaktifkan(ID_B);
    expect(n).toMatchObject({ ok: false, status: 409, kode: "4090105", pesan: "HR aktif terakhir tidak dapat dinonaktifkan." });
    expect(sesiAda()).toBe(true);
  });

  test("negatif: perusahaan belum Aktif (403) sampai ke klien dan sesi tetap hidup (layar akses ditolak)", async () => {
    const klien = await masuk();
    api.next = galatApi(403, "4030001");
    expect(await klien.daftar()).toMatchObject({ ok: false, status: 403 });
    expect(sesiAda()).toBe(true);
  });

  test("negatif: akun nonaktif (401 4010002) → kode sampai ke klien, sesi server dan cookie dihapus, permintaan berikutnya tidak diteruskan", async () => {
    const klien = await masuk();
    const sid = browser.cookies.get(SID) ?? "";
    api.next = galatApi(401, "4010002");
    expect(await klien.daftar()).toMatchObject({ ok: false, status: 401, kode: "4010002" });
    expect(sesiAda()).toBe(false);
    expect(backend.keys().some((k) => k === core.sessions.key("s", sid))).toBe(false);
    const n = api.requests.length;
    api.next = null;
    expect(await klien.daftar()).toMatchObject({ ok: false, status: 401 });
    expect(api.requests.length).toBe(n);
  });

  test("tidak aktif 30 menit → 401, sesi server terhapus, API tidak dipanggil", async () => {
    const klien = await masuk();
    const sid = browser.cookies.get(SID) ?? "";
    expect((await klien.daftar()).ok).toBe(true);
    const n = api.requests.length;
    clock.advance(30 * MENIT);
    const h = await klien.daftar();
    expect(h).toMatchObject({ ok: false, status: 401 });
    expect(api.requests.length).toBe(n);
    expect(backend.keys().some((k) => k === core.sessions.key("s", sid))).toBe(false);
    expect(sesiAda()).toBe(false);
  });

  test("aktivitas 29 menit sekali memperpanjang idle, tetapi 30 menit setelah aktivitas terakhir sesi berakhir", async () => {
    const klien = await masuk();
    clock.advance(29 * MENIT);
    expect((await klien.daftar()).ok).toBe(true);
    clock.advance(29 * MENIT);
    expect((await klien.daftar()).ok).toBe(true);
    clock.advance(30 * MENIT);
    expect(await klien.daftar()).toMatchObject({ ok: false, status: 401 });
  });

  test("negatif: tanpa sesi (belum masuk) → 401 dan tidak diteruskan", async () => {
    const klien = await masuk();
    await browser.send(routes.logout.POST, "/auth/logout", {
      method: "POST",
      headers: { origin: ORIGIN, "content-type": "application/x-www-form-urlencoded" },
      body: `csrf_token=${encodeURIComponent((json((await browser.send(routes.session.GET, "/auth/session")).body)["data"] as { csrf_token: string }).csrf_token)}`,
    });
    const n = api.requests.length;
    expect(await klien.daftar()).toMatchObject({ ok: false, status: 401 });
    expect(api.requests.length).toBe(n);
  });
});
