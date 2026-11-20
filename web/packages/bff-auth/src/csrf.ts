// Perlindungan CSRF BFF (ADR-011 butir 7, threat model WEB-04, KEPUTUSAN-269: synchronizer token, bukan
// double-submit cookie) [SRC-595]:
// 1. Token synchronizer per sesi (acak 256 bit, disimpan di sesi server), dikirim klien lewat header
//    `X-CSRF-Token` (atau field formulir `csrf_token` khusus formulir keluar), dibandingkan waktu-konstan.
//    Token tidak pernah ditaruh di cookie.
// 2. Pertahanan berlapis: bila ada, `Sec-Fetch-Site` harus `same-origin` dan `Origin` harus sama persis dengan
//    asal aplikasi. Permintaan tanpa kedua header (klien non-browser) tetap wajib token.

import { constantTimeEqual, type Session } from "./session.ts";

export const CSRF_HEADER = "x-csrf-token";
export const CSRF_FORM_FIELD = "csrf_token";

export type CsrfResult = "ok" | "origin_mismatch" | "token_mismatch";

export function checkCsrf(request: Request, session: Session, appOrigin: URL, formToken?: string): CsrfResult {
  const site = request.headers.get("sec-fetch-site");
  if (site !== null && site !== "same-origin") {
    return "origin_mismatch";
  }
  const origin = request.headers.get("origin");
  if (origin !== null && origin !== appOrigin.origin) {
    return "origin_mismatch";
  }
  const token = request.headers.get(CSRF_HEADER) ?? formToken;
  if (token === undefined || token === "" || !constantTimeEqual(token, session.csrfToken)) {
    return "token_mismatch";
  }
  return "ok";
}
