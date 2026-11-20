// Perakitan inti BFF: konfigurasi → rahasia OpenBao → Valkey → penyegel JWE → pengelola sesi → klien OIDC.
// Dibuat sekali per proses server Next.js dan dipakai bersama oleh Route Handler.

import type { BffConfig } from "./config.ts";
import { OidcClient } from "./oidc.ts";
import { readBffSecrets, type BffSecrets } from "./openbao.ts";
import { SessionManager, type SessionPolicy } from "./session.ts";
import { ValkeyBackend, type SessionBackend } from "./store.ts";
import { TokenSealer } from "./token-crypto.ts";

export interface BffCore {
  readonly config: BffConfig;
  readonly sessions: SessionManager;
  /** Klien OIDC (discovery malas; bila gagal, dicoba lagi pada pemanggilan berikutnya). */
  oidc(): Promise<OidcClient>;
  close(): Promise<void>;
}

export interface CreateBffCoreOptions {
  /** Pengganti untuk test: rahasia yang sudah dibaca, backend sesi, jam, kebijakan sesi. */
  readonly secrets?: BffSecrets;
  readonly backend?: SessionBackend;
  readonly now?: () => number;
  readonly policy?: SessionPolicy;
}

export async function createBffCore(config: BffConfig, options: CreateBffCoreOptions = {}): Promise<BffCore> {
  const secrets = options.secrets ?? (await readBffSecrets(config.openbao));
  const backend =
    options.backend ?? new ValkeyBackend({ url: config.valkeyUrl, password: secrets.valkeyPassword });
  const now = options.now ?? Date.now;
  const sessions = new SessionManager({
    appName: config.appName,
    backend,
    sealer: new TokenSealer(secrets.sessionKeys),
    now,
    ...(options.policy === undefined ? {} : { policy: options.policy }),
  });

  let discovering: Promise<OidcClient> | null = null;
  const oidc = (): Promise<OidcClient> => {
    discovering ??= OidcClient.discover({
      issuer: config.oidc.issuer,
      clientId: config.oidc.clientId,
      clientSecret: secrets.oidcClientSecret,
      redirectUri: config.oidc.redirectUri,
      now,
    }).catch((err: unknown) => {
      discovering = null;
      throw err;
    });
    return discovering;
  };

  return {
    config,
    sessions,
    oidc,
    close: () => backend.close(),
  };
}
