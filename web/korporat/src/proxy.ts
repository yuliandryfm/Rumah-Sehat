import { NextResponse, type NextRequest } from "next/server";
import { HEADER_DINAMIS, HEADER_STATIS, buatCsp, buatNonce, originUnggahDariEnv } from "./keamanan.ts";
import { idKorelasi } from "./lib/korelasi.ts";

/**
 * Proxy Next.js 16 (dulu `middleware`): hanya header keamanan, nonce CSP, dan ID korelasi. BUKAN untuk
 * keputusan otorisasi (ADR-024 butir 2, KEPUTUSAN-75): sesi diperiksa di server component dan Route Handler.
 * Berjalan di runtime Node.js. Tidak menyentuh cookie.
 */
export function proxy(request: NextRequest): NextResponse {
  const nonce = buatNonce();
  const dev = process.env.NODE_ENV === "development";
  const csp = buatCsp(nonce, dev, originUnggahDariEnv(process.env["KORPORAT_UPLOAD_ORIGIN"], dev));
  const korelasi = idKorelasi(request.headers.get("x-correlation-id"));

  // Next membaca nonce dari CSP pada header permintaan lalu memasangnya ke skrip dan gaya buatannya.
  // `X-Correlation-Id` dipakai ulang oleh BFF dan layar galat (kode referensi, KEPUTUSAN-172).
  const headerPermintaan = new Headers(request.headers);
  headerPermintaan.set("Content-Security-Policy", csp);
  headerPermintaan.set("X-Correlation-Id", korelasi);

  const respons = NextResponse.next({ request: { headers: headerPermintaan } });
  respons.headers.set("Content-Security-Policy", csp);
  respons.headers.set("X-Correlation-Id", korelasi);
  for (const { key, value } of [...HEADER_STATIS, ...HEADER_DINAMIS]) respons.headers.set(key, value);
  return respons;
}

export const config = {
  matcher: ["/((?!_next/static).*)"],
};
