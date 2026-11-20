// Konfigurasi bertipe BFF, dimuat sekali saat start dan gagal cepat (KEPUTUSAN-132, coding-standards §6).
// Galat hanya menyebut NAMA variabel, tidak pernah nilainya. Rahasia (secret klien OIDC, kunci sesi,
// password Valkey) TIDAK dibaca dari variabel lingkungan: semuanya dari OpenBao (ADR-024 butir 3, ADR-025).
// Satu-satunya kredensial di lingkungan adalah token OpenBao identitas proses (KEPUTUSAN-79), sama dengan
// crate `crypto` di Rust.

import { Sensitive } from "./sensitive.ts";

export class BffConfigError extends Error {
  override readonly name = "BffConfigError";
}

export interface BffConfig {
  /** Nama aplikasi (`backoffice`, `portal`, ...): dipakai di nama cookie dan awalan kunci Valkey. */
  readonly appName: string;
  /** Asal publik aplikasi (skema + host + port), dipakai untuk pemeriksaan header Origin. */
  readonly appOrigin: URL;
  readonly oidc: {
    /** Issuer realm Keycloak, misalnya `https://id.contoh/realms/admin`. */
    readonly issuer: URL;
    readonly clientId: string;
    /** Redirect URI terdaftar di klien Keycloak; harus satu asal dengan `appOrigin`. */
    readonly redirectUri: URL;
  };
  /** URL dasar API Rust tujuan penerusan Route Handler `bff/`. */
  readonly apiBaseUrl: URL;
  /** URL Valkey (`redis://` atau `rediss://`), tanpa kredensial; password dari OpenBao. */
  readonly valkeyUrl: URL;
  readonly openbao: {
    readonly address: URL;
    readonly token: Sensitive<string>;
    /** Mount KV v2, misalnya `secret`. */
    readonly kvMount: string;
    /** Path rahasia BFF di mount itu, misalnya `rs/bff/backoffice`. */
    readonly kvPath: string;
  };
}

/** Variabel lingkungan yang dibaca `loadBffConfig`. */
export const BFF_ENV = {
  appName: "BFF_APP_NAME",
  appOrigin: "BFF_APP_ORIGIN",
  issuer: "BFF_OIDC_ISSUER",
  clientId: "BFF_OIDC_CLIENT_ID",
  redirectUri: "BFF_OIDC_REDIRECT_URI",
  apiBaseUrl: "BFF_API_BASE_URL",
  valkeyUrl: "BFF_VALKEY_URL",
  openbaoAddr: "OPENBAO_ADDR",
  openbaoToken: "OPENBAO_TOKEN",
  kvMount: "BFF_OPENBAO_KV_MOUNT",
  kvPath: "BFF_OPENBAO_KV_PATH",
} as const;

const APP_NAME = /^[a-z][a-z0-9]{1,30}$/;
const CLIENT_ID = /^[A-Za-z0-9._-]{1,128}$/;
/** Segmen path OpenBao: `[a-z0-9_-]` (sama dengan `valid_path` di crate crypto). */
const BAO_SEGMENT = /^[a-z0-9_-]+$/;

/** Benar bila host URL berupa alamat IP loopback (127.0.0.0/8 atau ::1). Nama host tidak diterima. */
export function isLoopbackHost(url: URL): boolean {
  const host = url.hostname;
  if (host === "[::1]") {
    return true;
  }
  const parts = host.split(".");
  return (
    parts.length === 4 &&
    parts[0] === "127" &&
    parts.every((p) => /^\d{1,3}$/.test(p) && Number(p) <= 255)
  );
}

function wajib(env: Readonly<Record<string, string | undefined>>, nama: string): string {
  const nilai = env[nama];
  if (nilai === undefined || nilai.trim() === "") {
    throw new BffConfigError(`${nama} wajib diisi`);
  }
  return nilai.trim();
}

function urlDari(nilai: string, nama: string): URL {
  let url: URL;
  try {
    url = new URL(nilai);
  } catch {
    throw new BffConfigError(`${nama} bukan URL yang sah`);
  }
  if (url.username !== "" || url.password !== "") {
    throw new BffConfigError(`${nama} tidak boleh memuat kredensial`);
  }
  if (url.search !== "" || url.hash !== "") {
    throw new BffConfigError(`${nama} tidak boleh memuat query atau fragmen`);
  }
  return url;
}

/** HTTPS wajib, kecuali ke alamat IP loopback (pengembangan lokal). */
function httpsAtauLoopback(url: URL, nama: string): void {
  if (url.protocol === "https:") {
    return;
  }
  if (url.protocol === "http:" && isLoopbackHost(url)) {
    return;
  }
  throw new BffConfigError(`${nama} harus https (http hanya untuk alamat IP loopback)`);
}

export function loadBffConfig(env: Readonly<Record<string, string | undefined>>): BffConfig {
  const appName = wajib(env, BFF_ENV.appName);
  if (!APP_NAME.test(appName)) {
    throw new BffConfigError(`${BFF_ENV.appName} harus huruf kecil/angka, diawali huruf, 2-31 karakter`);
  }

  const appOrigin = urlDari(wajib(env, BFF_ENV.appOrigin), BFF_ENV.appOrigin);
  httpsAtauLoopback(appOrigin, BFF_ENV.appOrigin);
  if (appOrigin.pathname !== "/") {
    throw new BffConfigError(`${BFF_ENV.appOrigin} hanya skema, host, dan port (tanpa path)`);
  }

  const issuer = urlDari(wajib(env, BFF_ENV.issuer), BFF_ENV.issuer);
  httpsAtauLoopback(issuer, BFF_ENV.issuer);

  const clientId = wajib(env, BFF_ENV.clientId);
  if (!CLIENT_ID.test(clientId)) {
    throw new BffConfigError(`${BFF_ENV.clientId} memuat karakter yang tidak diizinkan`);
  }

  const redirectUri = urlDari(wajib(env, BFF_ENV.redirectUri), BFF_ENV.redirectUri);
  if (redirectUri.origin !== appOrigin.origin) {
    throw new BffConfigError(`${BFF_ENV.redirectUri} harus satu asal dengan ${BFF_ENV.appOrigin}`);
  }

  const apiBaseUrl = urlDari(wajib(env, BFF_ENV.apiBaseUrl), BFF_ENV.apiBaseUrl);
  // Server Next.js ke API di host yang sama lewat jaringan Docker privat tanpa TLS (docs/05-stack.md §2.1,
  // KEPUTUSAN-78), sehingga http diterima di sini; antar-host wajib https.
  if (apiBaseUrl.protocol !== "http:" && apiBaseUrl.protocol !== "https:") {
    throw new BffConfigError(`${BFF_ENV.apiBaseUrl} harus http atau https`);
  }

  const valkeyUrl = urlDari(wajib(env, BFF_ENV.valkeyUrl), BFF_ENV.valkeyUrl);
  if (valkeyUrl.protocol !== "redis:" && valkeyUrl.protocol !== "rediss:") {
    throw new BffConfigError(`${BFF_ENV.valkeyUrl} harus redis:// atau rediss://`);
  }

  const openbaoAddress = urlDari(wajib(env, BFF_ENV.openbaoAddr), BFF_ENV.openbaoAddr);
  // Sama dengan crate crypto (gagal-tertutup): TLS ke OpenBao staging/production belum diterapkan,
  // jadi hanya http ke alamat IP loopback (OpenBao mode dev) yang diterima.
  if (openbaoAddress.protocol !== "http:" || !isLoopbackHost(openbaoAddress) || openbaoAddress.pathname !== "/") {
    throw new BffConfigError(
      `${BFF_ENV.openbaoAddr} harus http://<ip-loopback>:<port> (TLS ke OpenBao belum diterapkan)`,
    );
  }
  const token = wajib(env, BFF_ENV.openbaoToken);

  const kvMount = wajib(env, BFF_ENV.kvMount);
  if (!BAO_SEGMENT.test(kvMount)) {
    throw new BffConfigError(`${BFF_ENV.kvMount} hanya [a-z0-9_-]`);
  }
  const kvPath = wajib(env, BFF_ENV.kvPath);
  if (!kvPath.split("/").every((s) => BAO_SEGMENT.test(s))) {
    throw new BffConfigError(`${BFF_ENV.kvPath} hanya segmen [a-z0-9_-] dipisah /`);
  }

  return {
    appName,
    appOrigin,
    oidc: { issuer, clientId, redirectUri },
    apiBaseUrl,
    valkeyUrl,
    openbao: { address: openbaoAddress, token: new Sensitive(token), kvMount, kvPath },
  };
}
