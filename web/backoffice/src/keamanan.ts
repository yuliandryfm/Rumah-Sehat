/**
 * Header keamanan backoffice (ADR-024 butir 2, ADR-027 butir 5, M-48). Kebijakan sama dengan `web/situs`:
 * CSP ketat, tanpa skrip inline (hanya `'self'` + nonce per permintaan), tanpa pihak ketiga, tanpa font
 * jarak jauh, tanpa sematan. Cookie sesi hanya dibuat Route Handler BFF (`@rs/bff-auth`), bukan di sini.
 */

export interface HeaderKeamanan {
  key: string;
  value: string;
}

/** Header yang sama untuk setiap respons (selain CSP, yang bernonce di proxy). */
export const HEADER_STATIS: readonly HeaderKeamanan[] = [
  // Dua tahun, termasuk subdomain. Tanpa `preload` (pendaftaran daftar preload adalah keputusan terpisah).
  { key: "Strict-Transport-Security", value: "max-age=63072000; includeSubDomains" },
  { key: "X-Content-Type-Options", value: "nosniff" },
  { key: "Referrer-Policy", value: "strict-origin-when-cross-origin" },
  {
    key: "Permissions-Policy",
    value:
      "accelerometer=(), autoplay=(), camera=(), display-capture=(), geolocation=(), gyroscope=(), " +
      "magnetometer=(), microphone=(), midi=(), payment=(), publickey-credentials-get=(), usb=()",
  },
  // Cadangan untuk peramban lama; `frame-ancestors 'none'` di CSP adalah kontrol utama.
  { key: "X-Frame-Options", value: "DENY" },
  { key: "Cross-Origin-Opener-Policy", value: "same-origin" },
];

/** Nonce 128 bit acak per permintaan, base64. */
export function buatNonce(): string {
  const byte = new Uint8Array(16);
  crypto.getRandomValues(byte);
  return btoa(String.fromCharCode(...byte));
}

/**
 * CSP. `nonce` null untuk berkas statis (tanpa dokumen HTML). `dev` menambah `'unsafe-eval'` hanya untuk
 * `next dev` (kebutuhan React di development; tidak dipakai di produksi).
 */
export function buatCsp(nonce: string | null, dev = false): string {
  const nonceSumber = nonce === null ? "" : ` 'nonce-${nonce}'`;
  const arahan = [
    "default-src 'self'",
    `script-src 'self'${nonceSumber}${nonce === null ? "" : " 'strict-dynamic'"}${dev ? " 'unsafe-eval'" : ""}`,
    `style-src 'self'${nonceSumber}`,
    "img-src 'self'",
    "font-src 'self'",
    "connect-src 'self'",
    "object-src 'none'",
    "base-uri 'self'",
    "form-action 'self'",
    "frame-ancestors 'none'",
    "upgrade-insecure-requests",
  ];
  return arahan.join("; ");
}
