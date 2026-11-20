// Klien OIDC BFF: Authorization Code + PKCE S256 sebagai klien rahasia (ADR-011 butir 7, ADR-024 butir 3)
// dengan openid-client 6.9.0 [SRC-592]. Fungsi yang dipakai diperiksa ke `build/index.d.ts`/`index.js`
// versi terpasang: `discovery`, `ClientSecretBasic`, `allowInsecureRequests`, `randomPKCECodeVerifier`,
// `calculatePKCECodeChallenge`, `randomState`, `randomNonce`, `buildAuthorizationUrl`,
// `authorizationCodeGrant` (redirect_uri diambil dari URL yang diberikan tanpa query), `refreshTokenGrant`,
// `ResponseBodyError`.
//
// Keluar: endpoint logout Keycloak dipanggil dari server dengan token penyegar dan autentikasi klien
// (Basic), jalur "non-browser" `LogoutEndpoint.logoutToken()` Keycloak 26.8.2 yang mengakhiri sesi
// pengguna dan membalas 204 [SRC-593]. Dengan begitu ID token tidak perlu dikirim ke browser sebagai
// `id_token_hint`.

import * as oidc from "openid-client";
import { isLoopbackHost } from "./config.ts";
import type { Sensitive } from "./sensitive.ts";
import type { LoginTransaction, TokenSet } from "./session.ts";

export class OidcError extends Error {
  override readonly name = "OidcError";
  /** `invalid_grant`: token penyegar/kode tidak berlaku lagi (sesi Keycloak berakhir atau dicabut). */
  readonly kind: "invalid_grant" | "rejected" | "unavailable";

  constructor(kind: OidcError["kind"], message: string) {
    super(message);
    this.kind = kind;
  }
}

export interface OidcClientOptions {
  readonly issuer: URL;
  readonly clientId: string;
  readonly clientSecret: Sensitive<string>;
  readonly redirectUri: URL;
  /** Batas waktu setiap panggilan ke Keycloak (detik). */
  readonly timeoutSeconds?: number;
  readonly now?: () => number;
}

function klasifikasi(err: unknown, konteks: string): OidcError {
  if (err instanceof oidc.ResponseBodyError) {
    return new OidcError(err.error === "invalid_grant" ? "invalid_grant" : "rejected", `${konteks}: ${err.error}`);
  }
  if (err instanceof oidc.AuthorizationResponseError) {
    return new OidcError("rejected", `${konteks}: ${err.error}`);
  }
  if (err instanceof oidc.ClientError) {
    return new OidcError("rejected", `${konteks}: respons tidak sah`);
  }
  return new OidcError("unavailable", `${konteks}: Keycloak tidak dapat dihubungi`);
}

export class OidcClient {
  readonly #config: oidc.Configuration;
  readonly #options: OidcClientOptions;
  readonly #now: () => number;

  private constructor(config: oidc.Configuration, options: OidcClientOptions) {
    this.#config = config;
    this.#options = options;
    this.#now = options.now ?? Date.now;
  }

  /** Discovery metadata issuer (`/.well-known/openid-configuration`), dengan validasi issuer. */
  static async discover(options: OidcClientOptions): Promise<OidcClient> {
    const insecure = options.issuer.protocol === "http:";
    if (insecure && !isLoopbackHost(options.issuer)) {
      throw new OidcError("rejected", "issuer http hanya untuk alamat IP loopback");
    }
    try {
      const config = await oidc.discovery(
        options.issuer,
        options.clientId,
        { redirect_uris: [options.redirectUri.href], response_types: ["code"] },
        oidc.ClientSecretBasic(options.clientSecret.expose()),
        {
          timeout: options.timeoutSeconds ?? 10,
          // Hanya untuk issuer http di alamat IP loopback (Keycloak lokal), diperiksa di atas. Penanda
          // `@deprecated` pustaka sengaja menandai fungsi ini sebagai khusus pengembangan lokal.
          // eslint-disable-next-line @typescript-eslint/no-deprecated
          ...(insecure ? { execute: [oidc.allowInsecureRequests] } : {}),
        },
      );
      return new OidcClient(config, options);
    } catch (err) {
      throw klasifikasi(err, "discovery OIDC");
    }
  }

  /** Membuat URL otorisasi beserta transaksi yang disimpan server sampai callback. */
  async beginLogin(returnTo: string): Promise<{ url: URL; tx: LoginTransaction }> {
    const codeVerifier = oidc.randomPKCECodeVerifier();
    const tx: LoginTransaction = {
      state: oidc.randomState(),
      nonce: oidc.randomNonce(),
      code_verifier: codeVerifier,
      return_to: returnTo,
    };
    const url = oidc.buildAuthorizationUrl(this.#config, {
      redirect_uri: this.#options.redirectUri.href,
      response_type: "code",
      scope: "openid",
      state: tx.state,
      nonce: tx.nonce,
      code_challenge: await oidc.calculatePKCECodeChallenge(codeVerifier),
      code_challenge_method: "S256",
    });
    return { url, tx };
  }

  /**
   * Menukar kode otorisasi. `callbackSearch` adalah query string permintaan callback; redirect URI yang
   * dikirim ke token endpoint selalu nilai terdaftar dari konfigurasi, bukan URL permintaan (yang di
   * belakang reverse proxy dapat berupa alamat internal).
   */
  async completeLogin(callbackSearch: string, tx: LoginTransaction): Promise<TokenSet> {
    const current = new URL(this.#options.redirectUri.href);
    current.search = callbackSearch;
    let res: Awaited<ReturnType<typeof oidc.authorizationCodeGrant>>;
    try {
      res = await oidc.authorizationCodeGrant(this.#config, current, {
        pkceCodeVerifier: tx.code_verifier,
        expectedState: tx.state,
        expectedNonce: tx.nonce,
        idTokenExpected: true,
      });
    } catch (err) {
      throw klasifikasi(err, "penukaran kode otorisasi");
    }
    const sub = res.claims()?.sub;
    if (sub === undefined || sub === "") {
      throw new OidcError("rejected", "penukaran kode otorisasi: ID token tanpa sub");
    }
    return this.#tokenSet(res, sub, undefined);
  }

  /** Menyegarkan token akses. Token penyegar Keycloak sekali pakai (`revokeRefreshToken`). */
  async refresh(current: TokenSet): Promise<TokenSet> {
    let res: Awaited<ReturnType<typeof oidc.refreshTokenGrant>>;
    try {
      res = await oidc.refreshTokenGrant(this.#config, current.refresh_token);
    } catch (err) {
      throw klasifikasi(err, "penyegaran token");
    }
    const sub = res.claims()?.sub ?? current.sub;
    if (sub !== current.sub) {
      throw new OidcError("rejected", "penyegaran token: subjek berubah");
    }
    return this.#tokenSet(res, sub, current);
  }

  #tokenSet(
    res: oidc.TokenEndpointResponse & oidc.TokenEndpointResponseHelpers,
    sub: string,
    previous: TokenSet | undefined,
  ): TokenSet {
    if (res.token_type.toLowerCase() !== "bearer") {
      throw new OidcError("rejected", "jenis token bukan Bearer");
    }
    const refresh = res.refresh_token ?? previous?.refresh_token;
    if (refresh === undefined || refresh === "") {
      throw new OidcError("rejected", "respons token tanpa refresh_token");
    }
    const expiresIn = res.expiresIn();
    if (expiresIn === undefined) {
      throw new OidcError("rejected", "respons token tanpa expires_in");
    }
    const idToken = res.id_token ?? previous?.id_token;
    return {
      access_token: res.access_token,
      refresh_token: refresh,
      ...(idToken === undefined ? {} : { id_token: idToken }),
      access_expires_at: this.#now() + expiresIn * 1000,
      sub,
    };
  }

  /**
   * Mengakhiri sesi Keycloak dari server (lihat komentar berkas). Galat dilempar sebagai `OidcError`;
   * `invalid_grant`/400 berarti sesi Keycloak sudah tidak berlaku.
   */
  async logout(tokens: TokenSet): Promise<void> {
    const endpoint = this.#config.serverMetadata().end_session_endpoint;
    if (endpoint === undefined) {
      throw new OidcError("rejected", "issuer tidak mengiklankan end_session_endpoint");
    }
    const url = new URL(endpoint);
    if (url.protocol !== "https:" && !(url.protocol === "http:" && isLoopbackHost(url))) {
      throw new OidcError("rejected", "end_session_endpoint harus https");
    }
    // RFC 6749 §2.3.1: id dan secret di-encode form-urlencoded sebelum Basic.
    const basic = Buffer.from(
      `${encodeURIComponent(this.#options.clientId)}:${encodeURIComponent(this.#options.clientSecret.expose())}`,
    ).toString("base64");
    let res: Response;
    try {
      res = await fetch(url, {
        method: "POST",
        headers: {
          Authorization: `Basic ${basic}`,
          "Content-Type": "application/x-www-form-urlencoded",
          Accept: "application/json",
        },
        body: new URLSearchParams({ refresh_token: tokens.refresh_token }).toString(),
        redirect: "manual",
        signal: AbortSignal.timeout((this.#options.timeoutSeconds ?? 10) * 1000),
      });
    } catch {
      throw new OidcError("unavailable", "logout Keycloak: tidak dapat dihubungi");
    }
    await res.body?.cancel();
    if (res.status === 204 || res.status === 200) {
      return;
    }
    throw new OidcError(res.status === 400 ? "invalid_grant" : "rejected", `logout Keycloak: HTTP ${String(res.status)}`);
  }
}
