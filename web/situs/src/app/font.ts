import localFont from "next/font/local";

/**
 * Font merek situs (CR-042, KEPUTUSAN-475; sistem desain Stitch `docs/design/stitch-ruang-sehat-design-system.md`):
 * Instrument Sans (judul, label, tombol), Alata (isi), Spline Sans Mono (angka dan stempel waktu). Berkas woff2
 * subset `latin` dari paket resmi Fontsource 5.3.0 (SRC-1000 s.d. SRC-1002), lisensi SIL OFL 1.1 (`fonts/OFL-*.txt`).
 * Disajikan dari origin situs lewat `next/font/local` (disalin ke `/_next/static/media`), sehingga CSP
 * `font-src 'self'` tetap cukup dan tidak ada font jarak jauh. Instrument Sans dan Alata di-preload; Spline Sans
 * Mono hanya dimuat saat dipakai. `display: swap` dengan cadangan tumpukan font sistem. Alata hanya punya bobot
 * 400: tebal sintetis dimatikan di CSS (`font-synthesis: none`), teks tebal memakai Instrument Sans.
 */
// Catatan: next/font mensyaratkan nilai literal, sehingga daftar cadangan ditulis ulang per font.

export const fontJudul = localFont({
  src: "./fonts/InstrumentSans-wght.woff2",
  weight: "400 700",
  style: "normal",
  display: "swap",
  preload: true,
  variable: "--font-judul",
  fallback: ["system-ui", "-apple-system", "Segoe UI", "Roboto", "Helvetica Neue", "Arial", "sans-serif"],
});

export const fontIsi = localFont({
  src: "./fonts/Alata-400.woff2",
  weight: "400",
  style: "normal",
  display: "swap",
  preload: true,
  variable: "--font-isi",
  fallback: ["system-ui", "-apple-system", "Segoe UI", "Roboto", "Helvetica Neue", "Arial", "sans-serif"],
});

export const fontAngka = localFont({
  src: "./fonts/SplineSansMono-wght.woff2",
  weight: "300 700",
  style: "normal",
  display: "swap",
  preload: false,
  variable: "--font-angka",
  fallback: ["ui-monospace", "SFMono-Regular", "Menlo", "Consolas", "monospace"],
});
