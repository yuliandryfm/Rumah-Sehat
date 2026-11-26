import localFont from "next/font/local";

/**
 * Font merek Plus Jakarta Sans (keputusan user Q18 2026-11-26, UP-59/C-13; KEPUTUSAN-356). Satu berkas variabel
 * `wght` (200–800) dari repositori resmi tokotype/PlusJakartaSans pada commit 18d1cd2 (SRC-881), lisensi
 * SIL OFL 1.1 (`fonts/OFL.txt`, SRC-882). Disajikan dari origin situs lewat `next/font/local` (berkas disalin ke
 * `/_next/static/media`), sehingga CSP `font-src 'self'` tetap cukup. `display: swap`, cadangan tumpukan font
 * sistem, dan hanya satu berkas yang di-preload.
 */
export const fontSitus = localFont({
  src: "./fonts/PlusJakartaSans-wght.woff2",
  weight: "200 800",
  style: "normal",
  display: "swap",
  preload: true,
  variable: "--font-situs",
  fallback: ["system-ui", "-apple-system", "Segoe UI", "Roboto", "Helvetica Neue", "Arial", "sans-serif"],
});
