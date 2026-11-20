import { describe, expect, test } from "vitest";
import * as bffProxy from "./app/bff/[...path]/route.ts";
import * as callback from "./app/auth/callback/route.ts";
import * as login from "./app/auth/login/route.ts";
import * as logout from "./app/auth/logout/route.ts";
import * as extend from "./app/auth/session/extend/route.ts";
import * as session from "./app/auth/session/route.ts";
import { routes } from "./lib/server/bff.ts";

describe("rute BFF terpasang dari createBffRoutes", () => {
  test("setiap berkas rute mengekspor handler yang sama dengan createBffRoutes", () => {
    expect(login.GET).toBe(routes.login.GET);
    expect(callback.GET).toBe(routes.callback.GET);
    expect(logout.POST).toBe(routes.logout.POST);
    expect(session.GET).toBe(routes.session.GET);
    expect(extend.POST).toBe(routes.extend.POST);
    expect(bffProxy.GET).toBe(routes.proxy.GET);
    expect(bffProxy.POST).toBe(routes.proxy.POST);
    expect(bffProxy.PUT).toBe(routes.proxy.PUT);
    expect(bffProxy.PATCH).toBe(routes.proxy.PATCH);
    expect(bffProxy.DELETE).toBe(routes.proxy.DELETE);
  });

  test("hanya metode yang memang disediakan yang diekspor (tanpa GET pada logout/extend)", () => {
    expect(Object.keys(logout)).toEqual(["POST"]);
    expect(Object.keys(extend)).toEqual(["POST"]);
    expect(Object.keys(login)).toEqual(["GET"]);
    expect(Object.keys(callback)).toEqual(["GET"]);
    expect(Object.keys(session)).toEqual(["GET"]);
    expect(Object.keys(bffProxy).sort()).toEqual(["DELETE", "GET", "PATCH", "POST", "PUT"]);
  });

  test("skenario negatif: tanpa konfigurasi BFF, rute gagal tertutup (503 envelope, no-store), bukan 500 atau data", async () => {
    // Env BFF_* tidak ada di test: createBffCore gagal cepat; handler harus menjawab terkendali.
    const res = await session.GET(new Request("http://127.0.0.1/auth/session"));
    expect(res.status).toBe(503);
    expect(res.headers.get("cache-control")).toBe("no-store");
    expect(res.headers.get("x-correlation-id")).toMatch(/^[0-9a-f-]{36}$/);
    expect(res.headers.getSetCookie()).toEqual([]);
    const badan = (await res.text()).toLowerCase();
    expect(badan).not.toContain("bff_");
    expect(badan).not.toContain("valkey");
  });
});
