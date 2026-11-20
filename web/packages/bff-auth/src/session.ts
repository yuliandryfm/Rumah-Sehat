// Siklus hidup sesi BFF: buat, muat, perbarui aktivitas, ganti token, akhiri (ADR-011 butir 7).
//
// Batas sesi web (ADR-011 butir 7, NFR-SEC-04, KEPUTUSAN-170): tidak aktif 30 menit, mutlak 12 jam,
// peringatan 2 menit sebelum batas. Nilai sama dengan `ssoSessionIdleTimeout`/`ssoSessionMaxLifespan`
// realm `admin` (infra/keycloak/realms/admin.json). Batas diperiksa di server pada setiap muat sesi;
// TTL Valkey hanya pembersih.

import { createHash, randomBytes, timingSafeEqual } from "node:crypto";
import type { SessionBackend } from "./store.ts";
import type { TokenSealer } from "./token-crypto.ts";

export interface SessionPolicy {
  readonly idleMs: number;
  readonly absoluteMs: number;
  readonly warningMs: number;
}

export const SESSION_POLICY: SessionPolicy = Object.freeze({
  idleMs: 30 * 60 * 1000,
  absoluteMs: 12 * 60 * 60 * 1000,
  warningMs: 2 * 60 * 1000,
});

/** Masa berlaku transaksi login (state/nonce/PKCE) antara redirect ke Keycloak dan callback. */
export const LOGIN_TX_TTL_MS = 10 * 60 * 1000;

const TTL_GRACE_MS = 60 * 1000;

/** Token hasil OIDC yang disimpan terenkripsi. Tidak pernah dikirim ke browser. */
export interface TokenSet {
  readonly access_token: string;
  readonly refresh_token: string;
  readonly id_token?: string;
  /** Epoch ms saat token akses kedaluwarsa. */
  readonly access_expires_at: number;
  /** Subjek Keycloak (`sub`). */
  readonly sub: string;
}

export interface LoginTransaction {
  readonly state: string;
  readonly nonce: string;
  readonly code_verifier: string;
  /** Path relatif tujuan setelah login. */
  readonly return_to: string;
}

export interface Session {
  /** ID sesi mentah dari cookie. Hanya hash-nya yang disimpan di Valkey. */
  readonly id: string;
  readonly createdAt: number;
  readonly lastActiveAt: number;
  readonly absoluteExpiresAt: number;
  readonly csrfToken: string;
  readonly sealedTokens: string;
}

export type LoadResult =
  | { readonly status: "active"; readonly session: Session }
  | { readonly status: "absent" }
  | { readonly status: "expired"; readonly reason: "idle" | "absolute" };

export class SessionDataError extends Error {
  override readonly name = "SessionDataError";
}

/** ID acak 256 bit, base64url tanpa padding (43 karakter). */
const ID_PATTERN = /^[A-Za-z0-9_-]{43}$/;

export function newRandomId(): string {
  return randomBytes(32).toString("base64url");
}

export function isWellFormedId(id: string): boolean {
  return ID_PATTERN.test(id);
}

function sha256hex(v: string): string {
  return createHash("sha256").update(v).digest("hex");
}

/** Perbandingan waktu-konstan dua string (lewat hash agar panjang sama). */
export function constantTimeEqual(a: string, b: string): boolean {
  return timingSafeEqual(createHash("sha256").update(a).digest(), createHash("sha256").update(b).digest());
}

function angka(v: string | undefined): number | null {
  if (v === undefined || !/^\d{1,16}$/.test(v)) {
    return null;
  }
  return Number(v);
}

function isTokenSet(v: unknown): v is TokenSet {
  if (typeof v !== "object" || v === null) {
    return false;
  }
  const t = v as Record<string, unknown>;
  return (
    typeof t["access_token"] === "string" &&
    t["access_token"] !== "" &&
    typeof t["refresh_token"] === "string" &&
    t["refresh_token"] !== "" &&
    (t["id_token"] === undefined || typeof t["id_token"] === "string") &&
    typeof t["access_expires_at"] === "number" &&
    typeof t["sub"] === "string" &&
    t["sub"] !== ""
  );
}

function isLoginTransaction(v: unknown): v is LoginTransaction {
  if (typeof v !== "object" || v === null) {
    return false;
  }
  const t = v as Record<string, unknown>;
  return ["state", "nonce", "code_verifier", "return_to"].every((k) => typeof t[k] === "string" && t[k] !== "");
}

export interface SessionManagerOptions {
  readonly appName: string;
  readonly backend: SessionBackend;
  readonly sealer: TokenSealer;
  readonly now?: () => number;
  readonly policy?: SessionPolicy;
}

export class SessionManager {
  readonly #app: string;
  readonly #backend: SessionBackend;
  readonly #sealer: TokenSealer;
  readonly now: () => number;
  readonly policy: SessionPolicy;

  constructor(options: SessionManagerOptions) {
    this.#app = options.appName;
    this.#backend = options.backend;
    this.#sealer = options.sealer;
    this.now = options.now ?? Date.now;
    this.policy = options.policy ?? SESSION_POLICY;
  }

  /** Kunci Valkey untuk ID mentah. Diekspor untuk test. */
  key(kind: "s" | "l" | "r", id: string): string {
    return `rs:bff:${this.#app}:${kind}:${sha256hex(id)}`;
  }

  /** Batas tidak aktif yang berlaku (tidak melewati batas mutlak). */
  idleExpiresAt(session: Session): number {
    return Math.min(session.lastActiveAt + this.policy.idleMs, session.absoluteExpiresAt);
  }

  /**
   * TTL Valkey = sisa batas logis + tenggang 60 detik, supaya pemeriksaan batas di `load` (yang
   * menentukan alasan kedaluwarsa dan menghapus sesi) selalu berjalan lebih dulu daripada TTL.
   */
  #ttl(lastActiveAt: number, absoluteExpiresAt: number): number {
    return Math.max(1, Math.min(lastActiveAt + this.policy.idleMs, absoluteExpiresAt) - this.now()) + TTL_GRACE_MS;
  }

  /** Membuat sesi baru (ID baru setiap login: mencegah fiksasi sesi). */
  async create(tokens: TokenSet): Promise<Session> {
    const now = this.now();
    const id = newRandomId();
    const csrfToken = newRandomId();
    const absoluteExpiresAt = now + this.policy.absoluteMs;
    const sealedTokens = await this.#sealer.seal(id, "tokens", tokens);
    const ok = await this.#backend.createHash(
      this.key("s", id),
      {
        v: "1",
        created_at: String(now),
        last_active_at: String(now),
        absolute_expires_at: String(absoluteExpiresAt),
        csrf: csrfToken,
        tokens: sealedTokens,
      },
      this.#ttl(now, absoluteExpiresAt),
    );
    if (!ok) {
      throw new SessionDataError("sesi tidak dapat dibuat");
    }
    return { id, createdAt: now, lastActiveAt: now, absoluteExpiresAt, csrfToken, sealedTokens };
  }

  /** Memuat sesi dan menegakkan batas tidak aktif dan mutlak. Sesi kedaluwarsa langsung dihapus. */
  async load(id: string | undefined): Promise<LoadResult> {
    if (id === undefined || !isWellFormedId(id)) {
      return { status: "absent" };
    }
    const h = await this.#backend.getHash(this.key("s", id));
    if (h === null) {
      return { status: "absent" };
    }
    const createdAt = angka(h["created_at"]);
    const lastActiveAt = angka(h["last_active_at"]);
    const absoluteExpiresAt = angka(h["absolute_expires_at"]);
    const csrfToken = h["csrf"];
    const sealedTokens = h["tokens"];
    if (
      h["v"] !== "1" ||
      createdAt === null ||
      lastActiveAt === null ||
      absoluteExpiresAt === null ||
      csrfToken === undefined ||
      sealedTokens === undefined
    ) {
      // Data rusak atau versi lain: perlakukan sebagai tidak ada dan bersihkan.
      await this.#backend.delete(this.key("s", id));
      return { status: "absent" };
    }
    const now = this.now();
    if (now >= absoluteExpiresAt) {
      await this.destroy(id);
      return { status: "expired", reason: "absolute" };
    }
    if (now >= lastActiveAt + this.policy.idleMs) {
      await this.destroy(id);
      return { status: "expired", reason: "idle" };
    }
    return {
      status: "active",
      session: { id, createdAt, lastActiveAt, absoluteExpiresAt, csrfToken, sealedTokens },
    };
  }

  /** Mencatat aktivitas pengguna. `null` bila sesi sudah tidak ada (misalnya keluar di tab lain). */
  async touch(session: Session): Promise<Session | null> {
    const now = this.now();
    const ok = await this.#backend.updateHashIfExists(
      this.key("s", session.id),
      { last_active_at: String(now) },
      this.#ttl(now, session.absoluteExpiresAt),
    );
    return ok ? { ...session, lastActiveAt: now } : null;
  }

  async tokens(session: Session): Promise<TokenSet> {
    const v = await this.#sealer.open(session.id, "tokens", session.sealedTokens);
    if (!isTokenSet(v)) {
      throw new SessionDataError("isi token sesi tidak sah");
    }
    return v;
  }

  /** Mengganti token setelah penyegaran. `null` bila sesi sudah tidak ada. */
  async replaceTokens(session: Session, tokens: TokenSet): Promise<Session | null> {
    const sealedTokens = await this.#sealer.seal(session.id, "tokens", tokens);
    const ok = await this.#backend.updateHashIfExists(
      this.key("s", session.id),
      { tokens: sealedTokens },
      // TTL tidak diubah: salinan `session` dapat lebih lama dari aktivitas yang dicatat permintaan lain.
      null,
    );
    return ok ? { ...session, sealedTokens } : null;
  }

  async destroy(id: string): Promise<void> {
    if (!isWellFormedId(id)) {
      return;
    }
    await this.#backend.delete(this.key("s", id));
    await this.#backend.delete(this.key("r", id));
  }

  /** Menyimpan transaksi login terenkripsi; mengembalikan ID untuk cookie login. */
  async saveLogin(tx: LoginTransaction): Promise<string> {
    const id = newRandomId();
    const sealed = await this.#sealer.seal(id, "login", tx);
    if (!(await this.#backend.putOnce(this.key("l", id), sealed, LOGIN_TX_TTL_MS))) {
      throw new SessionDataError("transaksi login tidak dapat disimpan");
    }
    return id;
  }

  /** Mengambil transaksi login sekali pakai. `null` bila tidak ada, kedaluwarsa, atau rusak. */
  async takeLogin(id: string | undefined): Promise<LoginTransaction | null> {
    if (id === undefined || !isWellFormedId(id)) {
      return null;
    }
    const sealed = await this.#backend.take(this.key("l", id));
    if (sealed === null) {
      return null;
    }
    try {
      const v = await this.#sealer.open(id, "login", sealed);
      return isLoginTransaction(v) ? v : null;
    } catch {
      return null;
    }
  }

  /** Kunci penyegaran per sesi (satu penyegar; token penyegar Keycloak sekali pakai). */
  acquireRefreshLock(session: Session, owner: string, ttlMs: number): Promise<boolean> {
    return this.#backend.acquireLock(this.key("r", session.id), owner, ttlMs);
  }

  releaseRefreshLock(session: Session, owner: string): Promise<void> {
    return this.#backend.releaseLock(this.key("r", session.id), owner);
  }

  refreshLockHeld(session: Session): Promise<boolean> {
    return this.#backend.exists(this.key("r", session.id));
  }
}
