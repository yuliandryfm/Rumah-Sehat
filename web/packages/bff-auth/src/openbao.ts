// Pembaca rahasia BFF dari OpenBao KV v2 (ADR-025: KV menyimpan "kunci enkripsi sesi BFF"; ADR-024 butir 3:
// rahasia klien dari OpenBao). API: `GET /v1/<mount>/data/<path>` dengan header `X-Vault-Token`, respons
// `{"data": {"data": {...}, "metadata": {...}}}` [SRC-488]. Isi respons dan token tidak pernah masuk ke
// pesan galat atau log.
//
// Bentuk rahasia (semua string):
//   oidc_client_secret      secret klien OIDC (Keycloak)
//   session_key             kunci induk enkripsi token sesi, 32 byte base64url (tanpa padding)
//   session_key_id          ID kunci (`[a-z0-9-]{1,32}`), ditulis di header `kid` JWE
//   session_key_previous    opsional: kunci sebelumnya, hanya untuk membuka sesi lama saat rotasi
//   session_key_previous_id opsional, wajib bila session_key_previous ada
//   valkey_password         password Valkey (requirepass)

import type { BffConfig } from "./config.ts";
import { Sensitive } from "./sensitive.ts";

export class BffSecretError extends Error {
  override readonly name = "BffSecretError";
}

export interface SessionKey {
  readonly id: string;
  readonly key: Sensitive<Uint8Array>;
}

export interface BffSecrets {
  readonly oidcClientSecret: Sensitive<string>;
  readonly valkeyPassword: Sensitive<string>;
  readonly sessionKeys: { readonly current: SessionKey; readonly previous?: SessionKey };
}

const KEY_ID = /^[a-z0-9-]{1,32}$/;
const BASE64URL = /^[A-Za-z0-9_-]+$/;
const MAX_RESPONSE_BYTES = 64 * 1024;

function teks(data: Record<string, unknown>, nama: string): string {
  const v = data[nama];
  if (typeof v !== "string" || v === "") {
    throw new BffSecretError(`rahasia OpenBao tidak memuat ${nama} yang sah`);
  }
  return v;
}

function kunci(data: Record<string, unknown>, namaKunci: string, namaId: string): SessionKey {
  const raw = teks(data, namaKunci);
  const id = teks(data, namaId);
  if (!KEY_ID.test(id)) {
    throw new BffSecretError(`${namaId} harus [a-z0-9-]{1,32}`);
  }
  if (!BASE64URL.test(raw)) {
    throw new BffSecretError(`${namaKunci} harus base64url tanpa padding`);
  }
  const bytes = new Uint8Array(Buffer.from(raw, "base64url"));
  if (bytes.length !== 32) {
    throw new BffSecretError(`${namaKunci} harus tepat 32 byte`);
  }
  return { id, key: new Sensitive(bytes) };
}

/** Mengurai `data.data` rahasia BFF. Diekspor untuk test. */
export function parseBffSecrets(data: Record<string, unknown>): BffSecrets {
  const current = kunci(data, "session_key", "session_key_id");
  let previous: SessionKey | undefined;
  if (data["session_key_previous"] !== undefined || data["session_key_previous_id"] !== undefined) {
    previous = kunci(data, "session_key_previous", "session_key_previous_id");
    if (previous.id === current.id) {
      throw new BffSecretError("session_key_previous_id tidak boleh sama dengan session_key_id");
    }
  }
  return {
    oidcClientSecret: new Sensitive(teks(data, "oidc_client_secret")),
    valkeyPassword: new Sensitive(teks(data, "valkey_password")),
    sessionKeys: previous === undefined ? { current } : { current, previous },
  };
}

export interface ReadSecretOptions {
  readonly timeoutMs?: number;
  readonly fetch?: typeof fetch;
}

/** Membaca rahasia BFF dari OpenBao. Gagal dengan `BffSecretError` tanpa membocorkan isi. */
export async function readBffSecrets(
  openbao: BffConfig["openbao"],
  options: ReadSecretOptions = {},
): Promise<BffSecrets> {
  const doFetch = options.fetch ?? fetch;
  const url = new URL(`/v1/${openbao.kvMount}/data/${openbao.kvPath}`, openbao.address);
  let res: Response;
  try {
    res = await doFetch(url, {
      method: "GET",
      headers: { "X-Vault-Token": openbao.token.expose(), Accept: "application/json" },
      redirect: "error",
      signal: AbortSignal.timeout(options.timeoutMs ?? 10_000),
    });
  } catch {
    throw new BffSecretError("OpenBao tidak dapat dihubungi");
  }
  if (!res.ok) {
    await res.body?.cancel();
    throw new BffSecretError(`OpenBao menolak pembacaan rahasia BFF (HTTP ${String(res.status)})`);
  }
  const body = await res.arrayBuffer();
  if (body.byteLength > MAX_RESPONSE_BYTES) {
    throw new BffSecretError("respons OpenBao terlalu besar");
  }
  let parsed: unknown;
  try {
    parsed = JSON.parse(new TextDecoder().decode(body));
  } catch {
    throw new BffSecretError("respons OpenBao bukan JSON");
  }
  const data = (parsed as { data?: { data?: unknown } } | null)?.data?.data;
  if (typeof data !== "object" || data === null || Array.isArray(data)) {
    throw new BffSecretError("respons OpenBao tidak berbentuk KV v2");
  }
  return parseBffSecrets(data as Record<string, unknown>);
}
