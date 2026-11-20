import { NextResponse, type NextRequest } from "next/server";
import { HEADER_STATIS, buatCsp, buatNonce } from "./keamanan.ts";

/**
 * Proxy Next.js 16 (dulu `middleware`): hanya untuk header keamanan dan nonce CSP. BUKAN untuk keputusan
 * otorisasi (ADR-024 butir 2); situs publik tidak punya sesi. Berjalan di runtime Node.js (bawaan Next 16),
 * bukan Edge. Tidak menyentuh cookie.
 */
export function proxy(request: NextRequest): NextResponse {
  const nonce = buatNonce();
  const csp = buatCsp(nonce, process.env.NODE_ENV === "development");

  // Next membaca nonce dari CSP pada header permintaan lalu memasangnya ke skrip dan gaya buatannya.
  const headerPermintaan = new Headers(request.headers);
  headerPermintaan.set("Content-Security-Policy", csp);

  const respons = NextResponse.next({ request: { headers: headerPermintaan } });
  respons.headers.set("Content-Security-Policy", csp);
  for (const { key, value } of HEADER_STATIS) respons.headers.set(key, value);
  return respons;
}

export const config = {
  // Semua jalur kecuali `/_next/static` (header-nya dari next.config.ts); tidak ada pengecualian prefetch
  // supaya setiap respons membawa header.
  matcher: ["/((?!_next/static).*)"],
};
