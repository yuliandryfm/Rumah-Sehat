// Data uji sintetis KHUSUS TEST (coding-standards §7.2): kunci acak per proses, tidak ada nilai nyata.

import { randomBytes } from "node:crypto";
import type { BffSecrets } from "../openbao.ts";
import { Sensitive } from "../sensitive.ts";

export function randomKey(): Sensitive<Uint8Array> {
  return new Sensitive(new Uint8Array(randomBytes(32)));
}

export function testSecrets(overrides: Partial<BffSecrets> = {}): BffSecrets {
  return {
    oidcClientSecret: new Sensitive(`uji-secret-${randomBytes(8).toString("hex")}`),
    valkeyPassword: new Sensitive(`uji-valkey-${randomBytes(8).toString("hex")}`),
    sessionKeys: { current: { id: "k1", key: randomKey() } },
    ...overrides,
  };
}

/** Jam yang dapat diatur untuk test batas sesi. */
export class TestClock {
  #t: number;

  constructor(start = Date.UTC(2026, 10, 20, 2, 0, 0)) {
    this.#t = start;
  }

  readonly now = (): number => this.#t;

  advance(ms: number): void {
    this.#t += ms;
  }
}

/** Variabel lingkungan BFF yang sah (lokal, sintetis). */
export function testEnv(overrides: Record<string, string | undefined> = {}): Record<string, string | undefined> {
  return {
    BFF_APP_NAME: "backoffice",
    BFF_APP_ORIGIN: "http://127.0.0.1:3000",
    BFF_OIDC_ISSUER: "http://127.0.0.1:18081/realms/admin",
    BFF_OIDC_CLIENT_ID: "rs-backoffice-bff",
    BFF_OIDC_REDIRECT_URI: "http://127.0.0.1:3000/auth/callback",
    BFF_API_BASE_URL: "http://127.0.0.1:8080",
    BFF_VALKEY_URL: "redis://127.0.0.1:16379/0",
    OPENBAO_ADDR: "http://127.0.0.1:18200",
    OPENBAO_TOKEN: "uji-token-openbao-sintetis",
    BFF_OPENBAO_KV_MOUNT: "secret",
    BFF_OPENBAO_KV_PATH: "rs/bff/backoffice",
    ...overrides,
  };
}
