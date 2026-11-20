// Authorization server OIDC tiruan KHUSUS TEST unit (tidak diekspor dari paket). Menegakkan perilaku yang
// diandalkan BFF dari Keycloak: klien rahasia lewat Basic, PKCE S256 wajib, kode sekali pakai, redirect URI
// tepat, nonce di ID token, token penyegar sekali pakai (rotasi), dan logout non-browser dengan token
// penyegar. Perilaku Keycloak sungguhan dibuktikan test integrasi `keycloak.integration.test.ts`.

import { createHash, randomBytes } from "node:crypto";
import { createServer, type IncomingMessage, type Server, type ServerResponse } from "node:http";
import type { AddressInfo } from "node:net";
import { SignJWT, exportJWK, generateKeyPair } from "jose";

export interface FakeOpOptions {
  readonly clientId: string;
  readonly clientSecret: string;
  readonly redirectUri: string;
  readonly accessTokenSeconds?: number;
}

interface PendingCode {
  challenge: string;
  method: string;
  redirectUri: string;
  nonce: string;
  sub: string;
  sid: string;
}

function b64url(buf: Buffer): string {
  return buf.toString("base64url");
}

export class FakeOp {
  readonly options: FakeOpOptions;
  issuer = "";
  /** Jumlah permintaan per jenis, untuk pemeriksaan test. */
  readonly calls = { token_code: 0, token_refresh: 0, logout: 0 };
  /** Semua access token yang pernah diterbitkan (untuk memastikan tidak bocor ke browser). */
  readonly issuedSecrets: string[] = [];
  /** Sesi SSO aktif (sid → sub). */
  readonly sessions = new Map<string, string>();
  /** Bila diisi, token endpoint membalas galat ini untuk grant refresh. */
  failRefreshWith: { status: number; error: string } | null = null;
  #server: Server | null = null;
  #codes = new Map<string, PendingCode>();
  #refresh = new Map<string, { sub: string; sid: string }>();
  #key: CryptoKey | null = null;
  #jwk: Record<string, unknown> = {};

  constructor(options: FakeOpOptions) {
    this.options = options;
  }

  async start(): Promise<void> {
    const { privateKey, publicKey } = await generateKeyPair("RS256");
    this.#key = privateKey;
    this.#jwk = { ...(await exportJWK(publicKey)), kid: "uji", alg: "RS256", use: "sig" };
    this.#server = createServer((req, res) => {
      this.#handle(req, res).catch(() => {
        res.statusCode = 500;
        res.end();
      });
    });
    await new Promise<void>((resolve) => this.#server?.listen(0, "127.0.0.1", resolve));
    const port = (this.#server.address() as AddressInfo).port;
    this.issuer = `http://127.0.0.1:${String(port)}/realms/uji`;
  }

  async stop(): Promise<void> {
    await new Promise<void>((resolve) => {
      if (this.#server === null) {
        resolve();
        return;
      }
      this.#server.close(() => {
        resolve();
      });
      this.#server.closeAllConnections();
    });
  }

  /**
   * Meniru halaman login Keycloak: memvalidasi parameter URL otorisasi lalu mengembalikan query string
   * callback (`?code=...&state=...`) seperti redirect ke klien.
   */
  authorize(authUrl: URL, sub = "f47ac10b-58cc-4372-a567-0e02b2c3d479"): string {
    const p = authUrl.searchParams;
    if (authUrl.href.split("?")[0] !== `${this.issuer}/protocol/openid-connect/auth`) {
      throw new Error("endpoint otorisasi salah");
    }
    if (p.get("client_id") !== this.options.clientId || p.get("response_type") !== "code") {
      throw new Error("client_id/response_type salah");
    }
    if (p.get("redirect_uri") !== this.options.redirectUri) {
      throw new Error("redirect_uri tidak terdaftar");
    }
    if (p.get("code_challenge_method") !== "S256" || (p.get("code_challenge") ?? "") === "") {
      throw new Error("PKCE S256 wajib");
    }
    if (!(p.get("scope") ?? "").split(" ").includes("openid")) {
      throw new Error("scope openid wajib");
    }
    const code = b64url(randomBytes(24));
    const sid = b64url(randomBytes(12));
    this.sessions.set(sid, sub);
    this.#codes.set(code, {
      challenge: p.get("code_challenge") ?? "",
      method: "S256",
      redirectUri: p.get("redirect_uri") ?? "",
      nonce: p.get("nonce") ?? "",
      sub,
      sid,
    });
    return `?${new URLSearchParams({ code, state: p.get("state") ?? "" }).toString()}`;
  }

  /** Mencabut semua sesi SSO (padanan "Remove sessions" Admin REST Keycloak). */
  revokeAllSessions(): void {
    this.sessions.clear();
  }

  #json(res: ServerResponse, status: number, body: unknown): void {
    res.statusCode = status;
    res.setHeader("Content-Type", "application/json");
    res.setHeader("Cache-Control", "no-store");
    res.end(JSON.stringify(body));
  }

  async #body(req: IncomingMessage): Promise<URLSearchParams> {
    const chunks: Buffer[] = [];
    for await (const c of req) {
      chunks.push(c as Buffer);
    }
    return new URLSearchParams(Buffer.concat(chunks).toString("utf8"));
  }

  #basicOk(req: IncomingMessage): boolean {
    const h = req.headers.authorization ?? "";
    if (!h.startsWith("Basic ")) {
      return false;
    }
    const [id, secret] = Buffer.from(h.slice(6), "base64").toString("utf8").split(":").map(decodeURIComponent);
    return id === this.options.clientId && secret === this.options.clientSecret;
  }

  async #tokens(sub: string, sid: string, nonce: string | null): Promise<Record<string, unknown>> {
    if (this.#key === null) {
      throw new Error("belum start");
    }
    const now = Math.floor(Date.now() / 1000);
    const access = `at-${b64url(randomBytes(24))}`;
    const refresh = `rt-${b64url(randomBytes(24))}`;
    this.#refresh.set(refresh, { sub, sid });
    const idToken = await new SignJWT({ sub, sid, ...(nonce === null ? {} : { nonce }) })
      .setProtectedHeader({ alg: "RS256", kid: "uji" })
      .setIssuer(this.issuer)
      .setAudience(this.options.clientId)
      .setIssuedAt(now)
      .setExpirationTime(now + 300)
      .sign(this.#key);
    this.issuedSecrets.push(access, refresh, idToken);
    return {
      access_token: access,
      token_type: "Bearer",
      expires_in: this.options.accessTokenSeconds ?? 300,
      refresh_token: refresh,
      id_token: idToken,
      scope: "openid",
    };
  }

  async #handle(req: IncomingMessage, res: ServerResponse): Promise<void> {
    const path = new URL(req.url ?? "/", "http://x").pathname;
    const base = new URL(this.issuer).pathname;
    if (req.method === "GET" && path === `${base}/.well-known/openid-configuration`) {
      this.#json(res, 200, {
        issuer: this.issuer,
        authorization_endpoint: `${this.issuer}/protocol/openid-connect/auth`,
        token_endpoint: `${this.issuer}/protocol/openid-connect/token`,
        end_session_endpoint: `${this.issuer}/protocol/openid-connect/logout`,
        jwks_uri: `${this.issuer}/protocol/openid-connect/certs`,
        response_types_supported: ["code"],
        id_token_signing_alg_values_supported: ["RS256"],
        code_challenge_methods_supported: ["S256"],
        token_endpoint_auth_methods_supported: ["client_secret_basic", "client_secret_post"],
      });
      return;
    }
    if (req.method === "GET" && path === `${base}/protocol/openid-connect/certs`) {
      this.#json(res, 200, { keys: [this.#jwk] });
      return;
    }
    if (req.method === "POST" && path === `${base}/protocol/openid-connect/token`) {
      if (!this.#basicOk(req)) {
        this.#json(res, 401, { error: "invalid_client" });
        return;
      }
      const form = await this.#body(req);
      if (form.has("client_secret")) {
        this.#json(res, 400, { error: "invalid_request", error_description: "dua metode autentikasi klien" });
        return;
      }
      if (form.get("grant_type") === "authorization_code") {
        this.calls.token_code++;
        const code = form.get("code") ?? "";
        const pending = this.#codes.get(code);
        this.#codes.delete(code);
        const verifier = form.get("code_verifier") ?? "";
        const challenge = b64url(createHash("sha256").update(verifier).digest());
        if (pending?.redirectUri !== form.get("redirect_uri") || pending.challenge !== challenge) {
          this.#json(res, 400, { error: "invalid_grant" });
          return;
        }
        this.#json(res, 200, await this.#tokens(pending.sub, pending.sid, pending.nonce));
        return;
      }
      if (form.get("grant_type") === "refresh_token") {
        this.calls.token_refresh++;
        if (this.failRefreshWith !== null) {
          this.#json(res, this.failRefreshWith.status, { error: this.failRefreshWith.error });
          return;
        }
        const rt = form.get("refresh_token") ?? "";
        const entry = this.#refresh.get(rt);
        this.#refresh.delete(rt);
        if (entry === undefined || !this.sessions.has(entry.sid)) {
          this.#json(res, 400, { error: "invalid_grant" });
          return;
        }
        this.#json(res, 200, await this.#tokens(entry.sub, entry.sid, null));
        return;
      }
      this.#json(res, 400, { error: "unsupported_grant_type" });
      return;
    }
    if (req.method === "POST" && path === `${base}/protocol/openid-connect/logout`) {
      this.calls.logout++;
      if (!this.#basicOk(req)) {
        this.#json(res, 401, { error: "unauthorized_client" });
        return;
      }
      const form = await this.#body(req);
      const entry = this.#refresh.get(form.get("refresh_token") ?? "");
      if (entry === undefined) {
        this.#json(res, 400, { error: "invalid_grant" });
        return;
      }
      this.sessions.delete(entry.sid);
      res.statusCode = 204;
      res.end();
      return;
    }
    res.statusCode = 404;
    res.end();
  }
}
