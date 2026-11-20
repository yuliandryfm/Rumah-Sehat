// @rs/bff-auth: BFF autentikasi server Next.js (ADR-011 butir 7, ADR-024 butir 3). Modul kritis T1.
// Hanya untuk kode server (Route Handler, komponen server). Tidak boleh diimpor dari komponen klien.

export { BFF_ENV, BffConfigError, isLoopbackHost, loadBffConfig, type BffConfig } from "./config.ts";
export { createBffCore, type BffCore, type CreateBffCoreOptions } from "./core.ts";
export { CSRF_FORM_FIELD, CSRF_HEADER, checkCsrf, type CsrfResult } from "./csrf.ts";
export { BFF_CODES, DEFAULT_PROBLEM_TYPE_BASE, cookieNames, safeReturnPath, type CookieNames } from "./http.ts";
export { REFRESH_MARGIN_MS, freshTokens, type FreshTokens } from "./refresh.ts";
export {
  createBffRoutes,
  jsonLogger,
  type BffHandler,
  type BffLogger,
  type BffRoutes,
  type BffRoutesOptions,
} from "./routes.ts";
export { OidcClient, OidcError, type OidcClientOptions } from "./oidc.ts";
export { BffSecretError, parseBffSecrets, readBffSecrets, type BffSecrets, type SessionKey } from "./openbao.ts";
export { Sensitive } from "./sensitive.ts";
export {
  LOGIN_TX_TTL_MS,
  SESSION_POLICY,
  SessionDataError,
  SessionManager,
  type LoadResult,
  type LoginTransaction,
  type Session,
  type SessionPolicy,
  type TokenSet,
} from "./session.ts";
export { ValkeyBackend, type SessionBackend, type ValkeyBackendOptions } from "./store.ts";
export { TokenCryptoError, TokenSealer, type SealPurpose } from "./token-crypto.ts";
