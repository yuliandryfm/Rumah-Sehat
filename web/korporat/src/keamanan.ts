/**
 * Header keamanan korporat (ADR-024 butir 2, ADR-027 butir 5, M-48). Kebijakan sama dengan `web/situs`:
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

/**
 * Tambahan untuk respons dinamis (halaman dan BFF; bukan berkas statis `/_next/static`): data perusahaan dan
 * peserta tidak boleh tersimpan di cache peramban atau perantara (US-1904, EN-11).
 */
export const HEADER_DINAMIS: readonly HeaderKeamanan[] = [{ key: "Cache-Control", value: "no-store" }];

/** Nonce 128 bit acak per permintaan, base64. */
export function buatNonce(): string {
  const byte = new Uint8Array(16);
  crypto.getRandomValues(byte);
  return btoa(String.fromCharCode(...byte));
}

/**
 * CSP. `nonce` null untuk berkas statis (tanpa dokumen HTML). `dev` menambah `'unsafe-eval'` hanya untuk
 * `next dev` (kebutuhan React di development; tidak dipakai di produksi). `connect-src` hanya `'self'`
 * (semua panggilan API lewat BFF satu asal) ditambah, bila dikonfigurasi, SATU origin object storage untuk
 * unggah dokumen perusahaan lewat URL bertanda tangan (F1-T103, ADR-018; pola backoffice F1-T95), hanya dari
 * `originUnggahDariEnv`. Unduh dokumen adalah navigasi tautan biasa, jadi tidak menambah arahan lain.
 */
export function buatCsp(nonce: string | null, dev = false, originUnggah: string | null = null): string {
  const nonceSumber = nonce === null ? "" : ` 'nonce-${nonce}'`;
  const arahan = [
    "default-src 'self'",
    `script-src 'self'${nonceSumber}${nonce === null ? "" : " 'strict-dynamic'"}${dev ? " 'unsafe-eval'" : ""}`,
    `style-src 'self'${nonceSumber}`,
    "img-src 'self'",
    "font-src 'self'",
    `connect-src 'self'${originUnggah === null ? "" : ` ${originUnggah}`}`,
    "object-src 'none'",
    "base-uri 'self'",
    "form-action 'self'",
    "frame-ancestors 'none'",
    "upgrade-insecure-requests",
  ];
  return arahan.join("; ");
}

/** Host DNS (label huruf kecil/angka/tanda hubung) atau IPv4 bertitik, setelah normalisasi parser URL. */
const HOST_KETAT = /^(?=.{1,253}$)[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?(?:\.[a-z0-9](?:[a-z0-9-]{0,61}[a-z0-9])?)*$/;

/**
 * Origin object storage untuk unggah dokumen perusahaan dari env `KORPORAT_UPLOAD_ORIGIN` (hanya origin, tanpa
 * path/kredensial). Nilai tidak valid atau bukan https (kecuali `http` di development) diabaikan: gagal tertutup,
 * hanya unggah dokumen yang tidak jalan, CSP tidak melonggar. Sama dengan `originUnggahDariEnv` backoffice.
 */
export function originUnggahDariEnv(nilai: string | undefined, dev = false): string | null {
  if (nilai === undefined) return null;
  const masukan = nilai.trim();
  if (masukan === "") return null;
  let u: URL;
  try {
    u = new URL(masukan);
  } catch {
    return null;
  }
  const skemaOk = u.protocol === "https:" || (dev && u.protocol === "http:");
  // Parser URL WHATWG menerima `*`, `'`, `,`, `;` di host dan diam-diam membuang TAB/LF serta mengubah `\` dan
  // `/%2e`: host diperiksa dengan pola ketat, dan origin hasil parse wajib sama persis dengan masukan (tanpa
  // garis miring penutup), supaya nilai CSP tidak pernah berbeda dari yang ditulis operator (review T1 F1-T103).
  const hostOk = HOST_KETAT.test(u.hostname);
  const samaPersis = u.origin === masukan.replace(/\/$/, "").toLowerCase();
  return skemaOk && hostOk && samaPersis ? u.origin : null;
}
