// Penyegaran token akses di server dengan satu penyegar per sesi. Token penyegar Keycloak sekali pakai
// (`revokeRefreshToken`, `refreshTokenMaxReuse` 0, infra/keycloak/realms/admin.json), sehingga dua
// penyegaran serentak dengan token yang sama membuat salah satunya `invalid_grant` dan mengakhiri sesi.
// Kunci Valkey (SET NX PX) memastikan hanya satu yang menyegarkan; permintaan lain menunggu lalu membaca
// token baru.

import { randomUUID } from "node:crypto";
import type { BffCore } from "./core.ts";
import { OidcError } from "./oidc.ts";
import { SessionDataError, type Session, type TokenSet } from "./session.ts";
import { TokenCryptoError } from "./token-crypto.ts";

/** Token akses disegarkan bila sisa masa berlakunya kurang dari ini. */
export const REFRESH_MARGIN_MS = 30_000;
const LOCK_TTL_MS = 15_000;
const WAIT_STEP_MS = 100;
const WAIT_MAX_MS = 10_000;

export type FreshTokens =
  | { readonly kind: "ok"; readonly session: Session; readonly tokens: TokenSet }
  /** Sesi Keycloak berakhir/dicabut atau sesi BFF hilang: sesi BFF sudah dihapus. */
  | { readonly kind: "revoked" }
  /** Keycloak atau Valkey tidak tersedia: sesi dipertahankan, permintaan dijawab 503. */
  | { readonly kind: "unavailable" };

function fresh(core: BffCore, t: TokenSet): boolean {
  return t.access_expires_at - core.sessions.now() > REFRESH_MARGIN_MS;
}

/**
 * Token sesi, atau `null` bila tidak dapat dibuka (kunci sesi dirotasi tanpa kunci lama, atau data rusak):
 * sesi seperti itu tidak dapat dipulihkan, jadi dihapus dan pengguna diminta masuk lagi.
 */
async function openTokens(core: BffCore, session: Session): Promise<TokenSet | null> {
  try {
    return await core.sessions.tokens(session);
  } catch (err) {
    if (err instanceof TokenCryptoError || err instanceof SessionDataError) {
      await core.sessions.destroy(session.id);
      return null;
    }
    throw err;
  }
}

async function reload(core: BffCore, id: string): Promise<{ session: Session; tokens: TokenSet } | null> {
  const r = await core.sessions.load(id);
  if (r.status !== "active") {
    return null;
  }
  const tokens = await openTokens(core, r.session);
  return tokens === null ? null : { session: r.session, tokens };
}

/**
 * Token akses yang masih berlaku untuk sesi. `force` menyegarkan walau token masih berlaku (dipakai
 * "Tetap masuk", supaya sesi Keycloak ikut diperpanjang dan pencabutan di Keycloak segera terdeteksi).
 */
export async function freshTokens(core: BffCore, session: Session, force = false): Promise<FreshTokens> {
  const current = await openTokens(core, session);
  if (current === null) {
    return { kind: "revoked" };
  }
  if (!force && fresh(core, current)) {
    return { kind: "ok", session, tokens: current };
  }
  const owner = randomUUID();
  if (await core.sessions.acquireRefreshLock(session, owner, LOCK_TTL_MS)) {
    try {
      const now = await reload(core, session.id);
      if (now === null) {
        return { kind: "revoked" };
      }
      // Permintaan lain sudah menyegarkan di antara pemuatan sesi dan pengambilan kunci.
      if (now.tokens.refresh_token !== current.refresh_token || (!force && fresh(core, now.tokens))) {
        return { kind: "ok", ...now };
      }
      const oidc = await core.oidc();
      const next = await oidc.refresh(now.tokens);
      const saved = await core.sessions.replaceTokens(now.session, next);
      if (saved === null) {
        return { kind: "revoked" };
      }
      return { kind: "ok", session: saved, tokens: next };
    } catch (err) {
      if (err instanceof OidcError && err.kind === "invalid_grant") {
        await core.sessions.destroy(session.id);
        return { kind: "revoked" };
      }
      return { kind: "unavailable" };
    } finally {
      await core.sessions.releaseRefreshLock(session, owner).catch(() => undefined);
    }
  }
  // Penyegar lain sedang berjalan: tunggu kuncinya dilepas, lalu baca token terbaru.
  const until = Date.now() + WAIT_MAX_MS;
  while (Date.now() < until && (await core.sessions.refreshLockHeld(session))) {
    await new Promise((r) => setTimeout(r, WAIT_STEP_MS));
  }
  const now = await reload(core, session.id);
  if (now === null) {
    return { kind: "revoked" };
  }
  if (now.tokens.refresh_token !== current.refresh_token || (!force && fresh(core, now.tokens))) {
    return { kind: "ok", ...now };
  }
  return { kind: "unavailable" };
}
