// Padanan `SessionBackend` di memori, KHUSUS TEST (tidak diekspor dari paket). Perilaku TTL mengikuti jam
// yang disuntikkan. Kesesuaian perilakunya dengan Valkey dibuktikan suite yang sama di test integrasi.

import type { SessionBackend } from "../store.ts";

interface Entry {
  value: string | Map<string, string>;
  expiresAt: number;
}

export class MemoryBackend implements SessionBackend {
  readonly #data = new Map<string, Entry>();
  readonly #now: () => number;

  constructor(now: () => number) {
    this.#now = now;
  }

  #get(key: string): Entry | undefined {
    const e = this.#data.get(key);
    if (e !== undefined && e.expiresAt <= this.#now()) {
      this.#data.delete(key);
      return undefined;
    }
    return e;
  }

  /** Daftar kunci yang masih hidup (untuk pemeriksaan test). */
  keys(): string[] {
    return [...this.#data.keys()].filter((k) => this.#get(k) !== undefined);
  }

  /** Nilai mentah (untuk pemeriksaan test). */
  raw(key: string): string | Record<string, string> | undefined {
    const e = this.#get(key);
    if (e === undefined) {
      return undefined;
    }
    return typeof e.value === "string" ? e.value : Object.fromEntries(e.value);
  }

  createHash(key: string, fields: Readonly<Record<string, string>>, ttlMs: number): Promise<boolean> {
    if (this.#get(key) !== undefined) {
      return Promise.resolve(false);
    }
    this.#data.set(key, { value: new Map(Object.entries(fields)), expiresAt: this.#now() + ttlMs });
    return Promise.resolve(true);
  }

  getHash(key: string): Promise<Record<string, string> | null> {
    const e = this.#get(key);
    if (e === undefined || typeof e.value === "string") {
      return Promise.resolve(null);
    }
    return Promise.resolve(Object.fromEntries(e.value));
  }

  updateHashIfExists(key: string, fields: Readonly<Record<string, string>>, ttlMs: number | null): Promise<boolean> {
    const e = this.#get(key);
    if (e === undefined || typeof e.value === "string") {
      return Promise.resolve(false);
    }
    for (const [k, v] of Object.entries(fields)) {
      e.value.set(k, v);
    }
    if (ttlMs !== null) {
      e.expiresAt = this.#now() + ttlMs;
    }
    return Promise.resolve(true);
  }

  delete(key: string): Promise<boolean> {
    const ada = this.#get(key) !== undefined;
    this.#data.delete(key);
    return Promise.resolve(ada);
  }

  putOnce(key: string, value: string, ttlMs: number): Promise<boolean> {
    if (this.#get(key) !== undefined) {
      return Promise.resolve(false);
    }
    this.#data.set(key, { value, expiresAt: this.#now() + ttlMs });
    return Promise.resolve(true);
  }

  take(key: string): Promise<string | null> {
    const e = this.#get(key);
    this.#data.delete(key);
    return Promise.resolve(e !== undefined && typeof e.value === "string" ? e.value : null);
  }

  acquireLock(key: string, owner: string, ttlMs: number): Promise<boolean> {
    return this.putOnce(key, owner, ttlMs);
  }

  releaseLock(key: string, owner: string): Promise<void> {
    const e = this.#get(key);
    if (e?.value === owner) {
      this.#data.delete(key);
    }
    return Promise.resolve();
  }

  exists(key: string): Promise<boolean> {
    return Promise.resolve(this.#get(key) !== undefined);
  }

  close(): Promise<void> {
    return Promise.resolve();
  }
}
