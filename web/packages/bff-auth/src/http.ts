// Respons HTTP BFF: envelope ADR-023 dengan kode dari registry docs/05-response-codes.md (coding-standards
// §3.5: galat BFF sendiri memakai envelope dan kode dari registry yang sama), header keamanan, ID korelasi
// (coding-standards §4.2), dan cookie `__Host-` (SRC-408).

import { randomUUID } from "node:crypto";

/** Kode yang dipakai BFF. Test `codes.test.ts` memastikan setiap kode terdaftar dengan deskripsi sama. */
export const BFF_CODES = {
  ok: { code: "2000000", http: 200, description: "Berhasil" },
  badRequest: { code: "4000001", http: 400, description: "Permintaan tidak dapat dibaca" },
  unauthenticated: { code: "4010001", http: 401, description: "Belum login atau sesi berakhir" },
  csrfRejected: { code: "4030101", http: 403, description: "Permintaan tidak dapat diverifikasi" },
  notFound: { code: "4040001", http: 404, description: "Data tidak ditemukan" },
  tooLarge: { code: "4130001", http: 413, description: "Berkas terlalu besar" },
  internal: { code: "5000001", http: 500, description: "Terjadi kesalahan pada sistem" },
  unavailable: { code: "5030001", http: 503, description: "Layanan sedang tidak tersedia" },
} as const;

export type BffCode = (typeof BFF_CODES)[keyof typeof BFF_CODES];

/** Awalan URI `type` RFC 9457; sama dengan bawaan backend (`DEFAULT_PROBLEM_TYPE_BASE`). */
export const DEFAULT_PROBLEM_TYPE_BASE = "https://problems.ruang-sehat.invalid";

const UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;

/** ID korelasi: nilai masuk hanya bila UUID sah; bila tidak, UUID baru. */
export function correlationIdOf(request: Request): string {
  const v = request.headers.get("x-correlation-id");
  return v !== null && UUID.test(v) ? v.toLowerCase() : randomUUID();
}

/** Header untuk semua respons BFF: tanpa cache (data sesi dan data bisnis tidak disimpan perantara). */
export function baseHeaders(correlationId: string): Headers {
  return new Headers({
    "Cache-Control": "no-store",
    Pragma: "no-cache",
    "X-Content-Type-Options": "nosniff",
    "Referrer-Policy": "no-referrer",
    "X-Correlation-Id": correlationId,
  });
}

export function problem(code: BffCode, correlationId: string, typeBase: string, setCookies: readonly string[] = []): Response {
  const headers = baseHeaders(correlationId);
  headers.set("Content-Type", "application/problem+json");
  for (const c of setCookies) {
    headers.append("Set-Cookie", c);
  }
  return new Response(
    JSON.stringify({
      type: `${typeBase.replace(/\/+$/, "")}/${code.code}`,
      response_code: code.code,
      response_description: code.description,
      correlation_id: correlationId,
    }),
    { status: code.http, headers },
  );
}

export function success(data: unknown, correlationId: string, setCookies: readonly string[] = []): Response {
  const headers = baseHeaders(correlationId);
  headers.set("Content-Type", "application/json");
  for (const c of setCookies) {
    headers.append("Set-Cookie", c);
  }
  return new Response(
    JSON.stringify({
      response_code: BFF_CODES.ok.code,
      response_description: BFF_CODES.ok.description,
      data,
      correlation_id: correlationId,
    }),
    { status: 200, headers },
  );
}

export function redirect(location: string, correlationId: string, setCookies: readonly string[] = []): Response {
  const headers = baseHeaders(correlationId);
  headers.set("Location", location);
  for (const c of setCookies) {
    headers.append("Set-Cookie", c);
  }
  return new Response(null, { status: 303, headers });
}

// --- Cookie ---

export interface CookieNames {
  readonly session: string;
  readonly login: string;
}

/**
 * Nama cookie per aplikasi. Prefiks `__Host-` mewajibkan `Secure`, tanpa `Domain`, dan `Path=/`, sehingga
 * cookie hanya dikirim ke host yang menyetelnya [SRC-408]. Nama aplikasi ikut di nama cookie karena cookie
 * tidak dipisah per port (backoffice dan portal lokal di 127.0.0.1 dengan port berbeda).
 */
export function cookieNames(appName: string): CookieNames {
  return { session: `__Host-rs_${appName}_sid`, login: `__Host-rs_${appName}_login` };
}

/**
 * `HttpOnly` (tidak terbaca JavaScript), `Secure`, `SameSite=Lax`. Lax (bukan Strict) karena callback OIDC
 * tiba sebagai navigasi lintas situs dari Keycloak dan cookie transaksi login harus ikut; perlindungan CSRF
 * utama tetap token synchronizer + pemeriksaan Origin (WEB-04) [SRC-595].
 */
export function setCookie(name: string, value: string, maxAgeSeconds?: number): string {
  const base = `${name}=${value}; Path=/; Secure; HttpOnly; SameSite=Lax`;
  return maxAgeSeconds === undefined ? base : `${base}; Max-Age=${String(Math.floor(maxAgeSeconds))}`;
}

export function clearCookie(name: string): string {
  return `${name}=; Path=/; Secure; HttpOnly; SameSite=Lax; Max-Age=0`;
}

/** Nilai cookie dari header `Cookie`, atau `undefined`. Hanya nilai berbentuk ID acak yang dikembalikan. */
export function readCookie(request: Request, name: string): string | undefined {
  const header = request.headers.get("cookie");
  if (header === null) {
    return undefined;
  }
  for (const part of header.split(";")) {
    const i = part.indexOf("=");
    if (i > 0 && part.slice(0, i).trim() === name) {
      const v = part.slice(i + 1).trim();
      return /^[A-Za-z0-9_-]{43}$/.test(v) ? v : undefined;
    }
  }
  return undefined;
}

/** Path relatif aman untuk redirect setelah login (mencegah open redirect). */
export function safeReturnPath(value: string | null, fallback: string): string {
  // Review keamanan F1-T42 B-1: input ambigu ditolak SEBELUM parsing, lalu hasil normalisasi `URL` wajib
  // identik dengan input dan tidak diawali `//` atau `/\`.
  if (value === null || value.length === 0 || value.length > 512) {
    return fallback;
  }
  // Daftar izin karakter: tanpa backslash, spasi, tab/newline/karakter kontrol, `#`, atau non-ASCII.
  if (!/^\/[A-Za-z0-9\-._~!$&'()*+,;=:@/%?]*$/.test(value)) {
    return fallback;
  }
  // Encoding yang dapat dinormalkan menjadi titik, garis miring, backslash, atau karakter kontrol; `%` liar.
  if (/%(2e|2f|5c|[01][0-9a-f]|7f)/i.test(value) || /%(?![0-9a-f]{2})/i.test(value)) {
    return fallback;
  }
  const path = value.split("?", 1)[0] ?? "";
  if (path !== "/") {
    const segments = path.slice(1).split("/");
    // Segmen `.`/`..` dan segmen kosong (`//`, `/a//b`) ditolak; hanya garis miring akhir tunggal yang boleh.
    if (segments.some((seg, i) => seg === "." || seg === ".." || (seg === "" && i !== segments.length - 1))) {
      return fallback;
    }
  }
  let u: URL;
  try {
    u = new URL(value, "http://bff.invalid");
  } catch {
    return fallback;
  }
  const hasil = `${u.pathname}${u.search}`;
  if (u.origin !== "http://bff.invalid" || hasil !== value || hasil.startsWith("//") || hasil.startsWith("/\\")) {
    return fallback;
  }
  return hasil;
}
