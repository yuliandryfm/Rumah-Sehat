// Penyimpanan sesi BFF di Valkey lewat ioredis 6.0.0 (ADR-024 butir 3, KEPUTUSAN-77) [SRC-591].
//
// Kunci (awalan per aplikasi, nilai `<h>` = SHA-256 heksadesimal dari ID acak di cookie):
//   rs:bff:<app>:s:<h>     HASH sesi; TTL = sisa batas tidak aktif/mutlak yang lebih dulu tercapai
//   rs:bff:<app>:l:<h>     transaksi login (state, nonce, PKCE verifier) terenkripsi, sekali pakai
//   rs:bff:<app>:r:<h>     kunci penyegaran token (satu penyegar per sesi)
// Field HASH diperbarui per field (HSET) lewat skrip yang hanya menulis bila sesi masih ada, sehingga
// pembaruan aktivitas tidak menimpa token hasil penyegaran dan tidak menghidupkan sesi yang sudah dihapus.

import { Redis } from "ioredis";
import type { Sensitive } from "./sensitive.ts";

export interface SessionBackend {
  /** Membuat HASH baru (gagal bila kunci sudah ada) dengan TTL milidetik. */
  createHash(key: string, fields: Readonly<Record<string, string>>, ttlMs: number): Promise<boolean>;
  /** Semua field HASH, atau `null` bila tidak ada. */
  getHash(key: string): Promise<Record<string, string> | null>;
  /** Menulis field hanya bila HASH masih ada; TTL diatur ulang bila `ttlMs` bukan `null`. */
  updateHashIfExists(key: string, fields: Readonly<Record<string, string>>, ttlMs: number | null): Promise<boolean>;
  /** Menghapus kunci; benar bila ada yang terhapus. */
  delete(key: string): Promise<boolean>;
  /** Menyimpan nilai sekali pakai (gagal bila kunci sudah ada). */
  putOnce(key: string, value: string, ttlMs: number): Promise<boolean>;
  /** Mengambil sekaligus menghapus nilai (GETDEL). */
  take(key: string): Promise<string | null>;
  /** Mengambil kunci eksklusif bila belum dipegang (SET NX PX). */
  acquireLock(key: string, owner: string, ttlMs: number): Promise<boolean>;
  /** Melepas kunci hanya bila masih dipegang `owner`. */
  releaseLock(key: string, owner: string): Promise<void>;
  /** Benar bila kunci (apa pun jenisnya) ada. */
  exists(key: string): Promise<boolean>;
  close(): Promise<void>;
}

const CREATE_IF_ABSENT = `
if redis.call('EXISTS', KEYS[1]) == 1 then
  return 0
end
for i = 2, #ARGV, 2 do
  redis.call('HSET', KEYS[1], ARGV[i], ARGV[i + 1])
end
redis.call('PEXPIRE', KEYS[1], ARGV[1])
return 1
`;

const UPDATE_IF_EXISTS = `
if redis.call('EXISTS', KEYS[1]) == 1 then
  for i = 2, #ARGV, 2 do
    redis.call('HSET', KEYS[1], ARGV[i], ARGV[i + 1])
  end
  if ARGV[1] ~= '0' then
    redis.call('PEXPIRE', KEYS[1], ARGV[1])
  end
  return 1
end
return 0
`;

const RELEASE_IF_OWNER = `
if redis.call('GET', KEYS[1]) == ARGV[1] then
  return redis.call('DEL', KEYS[1])
end
return 0
`;

function ttl(ms: number): number {
  const v = Math.floor(ms);
  if (!Number.isFinite(v) || v < 1) {
    throw new RangeError("TTL harus minimal 1 ms");
  }
  return v;
}

export interface ValkeyBackendOptions {
  readonly url: URL;
  readonly password: Sensitive<string>;
  /** Batas waktu koneksi dan perintah (ms). */
  readonly timeoutMs?: number;
}

/** Valkey tidak siap dalam batas waktu; penyimpan sesi tidak tersedia (dijawab 503 oleh rute BFF). */
export class BackendUnavailableError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "BackendUnavailableError";
  }
}

/** Implementasi `SessionBackend` di atas Valkey/ioredis. */
export class ValkeyBackend implements SessionBackend {
  readonly #redis: Redis;
  readonly #timeoutMs: number;
  #menunggu: Promise<void> | null = null;

  constructor(options: ValkeyBackendOptions) {
    const url = options.url;
    const db = url.pathname.length > 1 ? Number(url.pathname.slice(1)) : 0;
    if (!Number.isInteger(db) || db < 0) {
      throw new RangeError("nomor database Valkey tidak sah");
    }
    const timeout = options.timeoutMs ?? 5_000;
    this.#timeoutMs = timeout;
    this.#redis = new Redis({
      host: url.hostname.replace(/^\[|\]$/g, ""),
      port: url.port === "" ? 6379 : Number(url.port),
      db,
      password: options.password.expose(),
      ...(url.protocol === "rediss:" ? { tls: { servername: url.hostname } } : {}),
      connectTimeout: timeout,
      commandTimeout: timeout,
      maxRetriesPerRequest: 1,
      // Tanpa antrean diam-diam saat Valkey putus: permintaan gagal cepat (503), bukan menggantung.
      enableOfflineQueue: false,
      lazyConnect: true,
    });
    // Galat koneksi ditangani per perintah; pendengar ini mencegah event `error` tak tertangani
    // menghentikan proses. Pesan galat ioredis tidak memuat password.
    this.#redis.on("error", () => undefined);
  }

  /**
   * Menunggu koneksi siap (batas `timeoutMs`). Permintaan pertama setelah start dan permintaan serentak
   * berbagi satu penantian: tanpa ini perintah yang tiba saat status `connecting` langsung gagal
   * ("Stream isn't writeable and enableOfflineQueue options is false", CR-028). Bila tetap tidak siap,
   * dilempar `BackendUnavailableError` (rute BFF menjawabnya 503, bukan 500).
   */
  async #ready(): Promise<Redis> {
    if (this.#redis.status === "ready") {
      return this.#redis;
    }
    this.#menunggu ??= this.#tungguSiap().finally(() => {
      this.#menunggu = null;
    });
    await this.#menunggu;
    return this.#redis;
  }

  async #tungguSiap(): Promise<void> {
    const r = this.#redis;
    let timer: NodeJS.Timeout | undefined;
    const habis = new Promise<never>((_, tolak) => {
      timer = setTimeout(() => {
        tolak(new BackendUnavailableError("koneksi Valkey belum siap sebelum batas waktu"));
      }, this.#timeoutMs);
    });
    const siap = new Promise<void>((selesai, tolak) => {
      const gagal = (): void => {
        tolak(new BackendUnavailableError("koneksi Valkey gagal"));
      };
      if (r.status === "wait") {
        r.connect().then(() => { selesai(); }, gagal);
        return;
      }
      if (r.status === "end") {
        gagal();
        return;
      }
      if (r.status === "ready") {
        selesai();
        return;
      }
      const bersih = (): void => {
        r.off("ready", berhasil);
        r.off("end", gagal);
      };
      const berhasil = (): void => {
        bersih();
        selesai();
      };
      r.once("ready", berhasil);
      r.once("end", () => { bersih(); gagal(); });
    });
    try {
      await Promise.race([siap, habis]);
    } finally {
      clearTimeout(timer);
    }
  }

  async createHash(key: string, fields: Readonly<Record<string, string>>, ttlMs: number): Promise<boolean> {
    const r = await this.#ready();
    // Kunci sudah ada (tabrakan ID acak 256 bit, praktis mustahil): jangan ditimpa.
    const res = await r.eval(CREATE_IF_ABSENT, 1, key, String(ttl(ttlMs)), ...Object.entries(fields).flat());
    return res === 1;
  }

  async getHash(key: string): Promise<Record<string, string> | null> {
    const r = await this.#ready();
    const res = await r.hgetall(key);
    return Object.keys(res).length === 0 ? null : res;
  }

  async updateHashIfExists(
    key: string,
    fields: Readonly<Record<string, string>>,
    ttlMs: number | null,
  ): Promise<boolean> {
    const r = await this.#ready();
    const args = Object.entries(fields).flat();
    const res = await r.eval(UPDATE_IF_EXISTS, 1, key, ttlMs === null ? "0" : String(ttl(ttlMs)), ...args);
    return res === 1;
  }

  async delete(key: string): Promise<boolean> {
    const r = await this.#ready();
    return (await r.del(key)) > 0;
  }

  async putOnce(key: string, value: string, ttlMs: number): Promise<boolean> {
    const r = await this.#ready();
    return (await r.set(key, value, "PX", ttl(ttlMs), "NX")) === "OK";
  }

  async take(key: string): Promise<string | null> {
    const r = await this.#ready();
    return r.getdel(key);
  }

  async acquireLock(key: string, owner: string, ttlMs: number): Promise<boolean> {
    return this.putOnce(key, owner, ttlMs);
  }

  async releaseLock(key: string, owner: string): Promise<void> {
    const r = await this.#ready();
    await r.eval(RELEASE_IF_OWNER, 1, key, owner);
  }

  async exists(key: string): Promise<boolean> {
    const r = await this.#ready();
    return (await r.exists(key)) === 1;
  }

  async close(): Promise<void> {
    if (this.#redis.status !== "ready") {
      this.#redis.disconnect();
      return;
    }
    try {
      await this.#redis.quit();
    } catch {
      this.#redis.disconnect();
    }
  }
}
