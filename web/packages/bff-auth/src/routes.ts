// Route Handler BFF untuk App Router Next.js 16 (ADR-011 butir 7, ADR-024 butir 3, KEPUTUSAN-115).
// Setiap handler bertanda tangan `(request: Request) => Promise<Response>` (Web Request/Response) dan
// dipasang di berkas `route.ts` aplikasi, misalnya:
//
//   // web/backoffice/app/bff/[...path]/route.ts
//   import { routes } from "@/lib/server/bff";
//   export const { GET, POST, PUT, PATCH, DELETE } = routes.proxy;
//
// Rute: GET /auth/login, GET /auth/callback, POST /auth/logout, GET /auth/session,
// POST /auth/session/extend, dan /bff/<path> (GET/POST/PUT/PATCH/DELETE) [SRC-596].
//
// Prinsip: token akses/penyegar/ID tidak pernah keluar dari server (tidak di badan, header, cookie, atau
// URL respons); browser hanya memegang cookie sesi `__Host-` HttpOnly. Server Next.js tidak mengambil
// keputusan otorisasi data: API Rust memvalidasi token dan otorisasi pada setiap permintaan (ADR-012).

import type { BffCore } from "./core.ts";
import { CSRF_FORM_FIELD, checkCsrf } from "./csrf.ts";
import {
  BFF_CODES,
  DEFAULT_PROBLEM_TYPE_BASE,
  baseHeaders,
  clearCookie,
  cookieNames,
  correlationIdOf,
  problem,
  readCookie,
  redirect,
  safeReturnPath,
  setCookie,
  success,
  type BffCode,
} from "./http.ts";
import { OidcError } from "./oidc.ts";
import { freshTokens } from "./refresh.ts";
import { LOGIN_TX_TTL_MS, type Session } from "./session.ts";

export type BffHandler = (request: Request) => Promise<Response>;

export interface BffLogger {
  /** Kejadian keamanan (log_stream "security", ADR-010). `fields` tidak boleh memuat token/ID sesi. */
  security(event: string, fields: Readonly<Record<string, string>>): void;
  /** Galat operasional (log aplikasi). */
  error(event: string, fields: Readonly<Record<string, string>>): void;
}

/** Logger bawaan: satu baris JSON per kejadian ke stdout, tanpa nilai rahasia. */
export function jsonLogger(appName: string): BffLogger {
  const write = (level: string, stream: string, event: string, fields: Readonly<Record<string, string>>): void => {
    process.stdout.write(
      `${JSON.stringify({ ts: new Date().toISOString(), level, log_stream: stream, app: `bff-${appName}`, event, ...fields })}\n`,
    );
  };
  return {
    security: (event, fields) => {
      write("warn", "security", event, fields);
    },
    error: (event, fields) => {
      write("error", "application", event, fields);
    },
  };
}

export interface BffRoutesOptions {
  /**
   * Awalan path API Rust yang boleh diteruskan (tanpa garis miring awal), misalnya `["v1/admin/"]`.
   * Wajib diisi; path di luar awalan dijawab 404 tanpa diteruskan (ADR-006 butir 5: namespace API per kanal).
   */
  readonly allowedApiPrefixes: readonly string[];
  /** Path mount Route Handler proxy. Bawaan `/bff`. */
  readonly proxyMountPath?: string;
  /** Tujuan setelah login bila `return_to` tidak ada/tidak sah. Bawaan `/`. */
  readonly defaultReturnPath?: string;
  /** Tujuan saat login gagal/kedaluwarsa; query `auth_error=failed|expired` ditambahkan. Bawaan `/`. */
  readonly loginErrorPath?: string;
  /** Tujuan setelah keluar. Bawaan `/`. */
  readonly afterLogoutPath?: string;
  /** Batas waktu panggilan ke API Rust (ms). Bawaan 15 detik (KU-02 rumusan T-20). */
  readonly upstreamTimeoutMs?: number;
  /** Batas badan permintaan yang diteruskan (byte). Bawaan 1 MiB; unggah berkas lewat URL presigned (ADR-018). */
  readonly maxBodyBytes?: number;
  readonly problemTypeBase?: string;
  readonly logger?: BffLogger;
  /** Pengganti `fetch` untuk panggilan ke API Rust (test). */
  readonly fetch?: typeof fetch;
}

export interface BffRoutes {
  readonly login: { readonly GET: BffHandler };
  readonly callback: { readonly GET: BffHandler };
  readonly logout: { readonly POST: BffHandler };
  readonly session: { readonly GET: BffHandler };
  readonly extend: { readonly POST: BffHandler };
  readonly proxy: {
    readonly GET: BffHandler;
    readonly POST: BffHandler;
    readonly PUT: BffHandler;
    readonly PATCH: BffHandler;
    readonly DELETE: BffHandler;
  };
}

/** Header permintaan browser yang boleh diteruskan ke API Rust. Cookie dan header CSRF tidak pernah diteruskan. */
const FORWARD_REQUEST_HEADERS = ["accept", "content-type", "idempotency-key", "if-match", "if-none-match"];
/** Header respons API Rust yang boleh diteruskan ke browser. `Set-Cookie` dan `Location` tidak pernah diteruskan. */
const FORWARD_RESPONSE_HEADERS = ["content-type", "etag", "last-modified", "retry-after", "content-disposition"];
const SEGMENT = /^[A-Za-z0-9_~-][A-Za-z0-9._~-]*$/;

class BodyTooLarge extends Error {}

async function readLimited(request: Request, limit: number): Promise<Uint8Array<ArrayBuffer> | null> {
  if (request.body === null) {
    return null;
  }
  const declared = request.headers.get("content-length");
  if (declared !== null && Number(declared) > limit) {
    await request.body.cancel();
    throw new BodyTooLarge();
  }
  const chunks: Uint8Array[] = [];
  let size = 0;
  const reader = request.body.getReader();
  for (;;) {
    const { done, value } = await reader.read();
    if (done) {
      break;
    }
    size += value.byteLength;
    if (size > limit) {
      await reader.cancel();
      throw new BodyTooLarge();
    }
    chunks.push(value);
  }
  const out = new Uint8Array(size);
  let off = 0;
  for (const c of chunks) {
    out.set(c, off);
    off += c.byteLength;
  }
  return out;
}

export function createBffRoutes(getCore: () => Promise<BffCore>, options: BffRoutesOptions): BffRoutes {
  if (options.allowedApiPrefixes.length === 0 || options.allowedApiPrefixes.some((p) => !/^[a-z0-9]+\/([a-z0-9-]+\/)*$/.test(p))) {
    throw new RangeError("allowedApiPrefixes wajib berisi awalan berbentuk `v1/admin/`");
  }
  const mount = (options.proxyMountPath ?? "/bff").replace(/\/+$/, "");
  const typeBase = options.problemTypeBase ?? DEFAULT_PROBLEM_TYPE_BASE;
  const timeoutMs = options.upstreamTimeoutMs ?? 15_000;
  const maxBody = options.maxBodyBytes ?? 1024 * 1024;
  const doFetch = options.fetch ?? fetch;
  const defaultReturn = safeReturnPath(options.defaultReturnPath ?? "/", "/");
  const errorPath = safeReturnPath(options.loginErrorPath ?? "/", "/");
  const afterLogout = safeReturnPath(options.afterLogoutPath ?? "/", "/");
  let logger: BffLogger | undefined = options.logger;

  const fail = (code: BffCode, cid: string, cookies: readonly string[] = []): Response =>
    problem(code, cid, typeBase, cookies);

  const withError = (path: string, reason: "failed" | "expired"): string =>
    `${path}${path.includes("?") ? "&" : "?"}auth_error=${reason}`;

  /** Membungkus handler: galat infrastruktur tak terduga → 503 tanpa rincian (ADR-023 butir 5). */
  const guard =
    (name: string, fn: (request: Request, core: BffCore, cid: string) => Promise<Response>): BffHandler =>
    async (request) => {
      const cid = correlationIdOf(request);
      let core: BffCore;
      try {
        core = await getCore();
      } catch {
        (logger ?? jsonLogger("unknown")).error("bff_core_unavailable", { route: name, correlation_id: cid });
        return fail(BFF_CODES.unavailable, cid);
      }
      logger ??= jsonLogger(core.config.appName);
      try {
        return await fn(request, core, cid);
      } catch (err) {
        logger.error("bff_route_failed", {
          route: name,
          correlation_id: cid,
          error: err instanceof Error ? err.name : "unknown",
        });
        return fail(BFF_CODES.unavailable, cid);
      }
    };

  /** Sesi aktif dari cookie; bila tidak ada/kedaluwarsa, respons 401 dengan cookie dihapus. */
  const activeSession = async (
    request: Request,
    core: BffCore,
    cid: string,
  ): Promise<{ session: Session } | { response: Response }> => {
    const names = cookieNames(core.config.appName);
    const id = readCookie(request, names.session);
    const r = await core.sessions.load(id);
    if (r.status === "active") {
      return { session: r.session };
    }
    if (r.status === "expired") {
      logger?.security("bff_session_expired", { reason: r.reason, correlation_id: cid });
    }
    const hadCookie = request.headers.get("cookie")?.includes(`${names.session}=`) === true;
    return { response: fail(BFF_CODES.unauthenticated, cid, hadCookie ? [clearCookie(names.session)] : []) };
  };

  const csrfOk = (request: Request, core: BffCore, session: Session, cid: string, formToken?: string): Response | null => {
    const r = checkCsrf(request, session, core.config.appOrigin, formToken);
    if (r === "ok") {
      return null;
    }
    logger?.security("bff_csrf_rejected", { reason: r, correlation_id: cid });
    return fail(BFF_CODES.csrfRejected, cid);
  };

  const revoked = async (core: BffCore, session: Session, cid: string, why: string): Promise<Response> => {
    await core.sessions.destroy(session.id);
    logger?.security("bff_session_revoked", { reason: why, correlation_id: cid });
    return fail(BFF_CODES.unauthenticated, cid, [clearCookie(cookieNames(core.config.appName).session)]);
  };

  const statusData = (core: BffCore, s: Session): Record<string, unknown> => ({
    idle_expires_at: new Date(core.sessions.idleExpiresAt(s)).toISOString(),
    absolute_expires_at: new Date(s.absoluteExpiresAt).toISOString(),
    server_time: new Date(core.sessions.now()).toISOString(),
    warning_seconds: Math.floor(core.sessions.policy.warningMs / 1000),
    csrf_token: s.csrfToken,
  });

  // --- /auth/login ---
  const login = guard("login", async (request, core, cid) => {
    const returnTo = safeReturnPath(new URL(request.url).searchParams.get("return_to"), defaultReturn);
    const oidc = await core.oidc();
    const { url, tx } = await oidc.beginLogin(returnTo);
    const loginId = await core.sessions.saveLogin(tx);
    return redirect(url.href, cid, [setCookie(cookieNames(core.config.appName).login, loginId, LOGIN_TX_TTL_MS / 1000)]);
  });

  // --- /auth/callback ---
  const callback = guard("callback", async (request, core, cid) => {
    const names = cookieNames(core.config.appName);
    const clearLogin = clearCookie(names.login);
    const tx = await core.sessions.takeLogin(readCookie(request, names.login));
    if (tx === null) {
      logger?.security("bff_login_rejected", { reason: "transaction_missing", correlation_id: cid });
      return redirect(withError(errorPath, "expired"), cid, [clearLogin]);
    }
    const oidc = await core.oidc();
    let tokens;
    try {
      tokens = await oidc.completeLogin(new URL(request.url).search, tx);
    } catch (err) {
      if (err instanceof OidcError && err.kind !== "unavailable") {
        logger?.security("bff_login_rejected", { reason: err.kind, correlation_id: cid });
        return redirect(withError(errorPath, "failed"), cid, [clearLogin]);
      }
      throw err;
    }
    // Sesi lama (bila ada) diakhiri; sesi baru selalu ber-ID baru (cegah fiksasi sesi).
    const old = readCookie(request, names.session);
    if (old !== undefined) {
      await core.sessions.destroy(old);
    }
    const session = await core.sessions.create(tokens);
    return redirect(tx.return_to, cid, [clearLogin, setCookie(names.session, session.id)]);
  });

  // --- /auth/session (status; tidak dihitung sebagai aktivitas) ---
  const sessionStatus = guard("session", async (request, core, cid) => {
    const a = await activeSession(request, core, cid);
    if ("response" in a) {
      return a.response;
    }
    return success(statusData(core, a.session), cid);
  });

  // --- /auth/session/extend ("Tetap masuk", KEPUTUSAN-170) ---
  const extend = guard("extend", async (request, core, cid) => {
    const a = await activeSession(request, core, cid);
    if ("response" in a) {
      return a.response;
    }
    const bad = csrfOk(request, core, a.session, cid);
    if (bad !== null) {
      return bad;
    }
    const touched = await core.sessions.touch(a.session);
    if (touched === null) {
      return revoked(core, a.session, cid, "session_missing");
    }
    const t = await freshTokens(core, touched, true);
    if (t.kind === "revoked") {
      return revoked(core, touched, cid, "refresh_rejected");
    }
    if (t.kind === "unavailable") {
      return fail(BFF_CODES.unavailable, cid);
    }
    return success(statusData(core, t.session), cid);
  });

  // --- /auth/logout ---
  const logout = guard("logout", async (request, core, cid) => {
    const names = cookieNames(core.config.appName);
    let formToken: string | undefined;
    if ((request.headers.get("content-type") ?? "").startsWith("application/x-www-form-urlencoded")) {
      try {
        const raw = await readLimited(request, 4096);
        formToken = new URLSearchParams(new TextDecoder().decode(raw ?? new Uint8Array())).get(CSRF_FORM_FIELD) ?? undefined;
      } catch {
        return fail(BFF_CODES.badRequest, cid);
      }
    }
    const r = await core.sessions.load(readCookie(request, names.session));
    if (r.status !== "active") {
      return redirect(afterLogout, cid, [clearCookie(names.session)]);
    }
    const bad = csrfOk(request, core, r.session, cid, formToken);
    if (bad !== null) {
      return bad;
    }
    const tokens = await core.sessions.tokens(r.session).catch(() => null);
    // Sesi server diakhiri lebih dulu, apa pun hasil panggilan ke Keycloak.
    await core.sessions.destroy(r.session.id);
    if (tokens !== null) {
      try {
        await (await core.oidc()).logout(tokens);
      } catch (err) {
        logger?.security("bff_keycloak_logout_failed", {
          reason: err instanceof OidcError ? err.kind : "unknown",
          correlation_id: cid,
        });
      }
    }
    return redirect(afterLogout, cid, [clearCookie(names.session)]);
  });

  // --- /bff/<path> ---
  const proxy = guard("proxy", async (request, core, cid) => {
    const url = new URL(request.url);
    if (!url.pathname.startsWith(`${mount}/`)) {
      return fail(BFF_CODES.notFound, cid);
    }
    const segments = url.pathname.slice(mount.length + 1).split("/");
    const path = segments.join("/");
    if (
      segments.some((s) => !SEGMENT.test(s)) ||
      !options.allowedApiPrefixes.some((p) => path.startsWith(p) && path.length > p.length)
    ) {
      return fail(BFF_CODES.notFound, cid);
    }
    const a = await activeSession(request, core, cid);
    if ("response" in a) {
      return a.response;
    }
    const bad = csrfOk(request, core, a.session, cid);
    if (bad !== null) {
      return bad;
    }
    let body: Uint8Array<ArrayBuffer> | null = null;
    if (request.method !== "GET") {
      try {
        body = await readLimited(request, maxBody);
      } catch (err) {
        if (err instanceof BodyTooLarge) {
          return fail(BFF_CODES.tooLarge, cid);
        }
        return fail(BFF_CODES.badRequest, cid);
      }
    }
    const touched = await core.sessions.touch(a.session);
    if (touched === null) {
      return revoked(core, a.session, cid, "session_missing");
    }
    const t = await freshTokens(core, touched);
    if (t.kind === "revoked") {
      return revoked(core, touched, cid, "refresh_rejected");
    }
    if (t.kind === "unavailable") {
      return fail(BFF_CODES.unavailable, cid);
    }

    const base = core.config.apiBaseUrl;
    const target = new URL(`${base.pathname.replace(/\/+$/, "")}/${path}${url.search}`, base);
    const headers = new Headers();
    for (const h of FORWARD_REQUEST_HEADERS) {
      const v = request.headers.get(h);
      if (v !== null) {
        headers.set(h, v);
      }
    }
    headers.set("Authorization", `Bearer ${t.tokens.access_token}`);
    headers.set("X-Correlation-Id", cid);
    let upstream: Response;
    try {
      upstream = await doFetch(target, {
        method: request.method,
        headers,
        ...(body === null ? {} : { body }),
        redirect: "manual",
        signal: AbortSignal.timeout(timeoutMs),
      });
    } catch {
      logger?.error("bff_upstream_unavailable", { correlation_id: cid });
      return fail(BFF_CODES.unavailable, cid);
    }

    const out = baseHeaders(cid);
    for (const h of FORWARD_RESPONSE_HEADERS) {
      const v = upstream.headers.get(h);
      if (v !== null) {
        out.set(h, v);
      }
    }
    if (upstream.status === 401) {
      // API Rust menolak token yang masih berlaku: sesi dicabut atau akun nonaktif (KU-10). Sesi BFF diakhiri.
      await core.sessions.destroy(touched.id);
      logger?.security("bff_session_revoked", { reason: "upstream_401", correlation_id: cid });
      out.append("Set-Cookie", clearCookie(cookieNames(core.config.appName).session));
    }
    const noBody = [204, 205, 304].includes(upstream.status) || request.method === "HEAD";
    if (noBody) {
      await upstream.body?.cancel();
    }
    return new Response(noBody ? null : upstream.body, { status: upstream.status, headers: out });
  });

  return {
    login: { GET: login },
    callback: { GET: callback },
    logout: { POST: logout },
    session: { GET: sessionStatus },
    extend: { POST: extend },
    proxy: { GET: proxy, POST: proxy, PUT: proxy, PATCH: proxy, DELETE: proxy },
  };
}
