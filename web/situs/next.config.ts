import { dirname } from "node:path";
import { fileURLToPath } from "node:url";
import type { NextConfig } from "next";
import { HEADER_STATIS, buatCsp } from "./src/keamanan.ts";

/**
 * Konfigurasi situs publik (ADR-024 butir 2, ADR-027 butir 1 dan 5).
 * - `output: "standalone"`, self-hosted; tanpa Vercel.
 * - Optimasi gambar dimatikan (`images.unoptimized`); `next/og`, Edge runtime, `use cache`, dan
 *   `next/image` tidak dipakai (riwayat advisory, docs/05-stack.md §1.2).
 * - `poweredByHeader: false`: tanpa `X-Powered-By`.
 * - Header keamanan untuk berkas statis `/_next/static` dipasang di sini; semua rute lain mendapat
 *   header yang sama, plus CSP bernonce per permintaan, dari `src/proxy.ts`.
 */
const akarWorkspace = dirname(dirname(fileURLToPath(import.meta.url)));

const konfigurasi: NextConfig = {
  output: "standalone",
  outputFileTracingRoot: akarWorkspace,
  poweredByHeader: false,
  reactStrictMode: true,
  images: { unoptimized: true },
  transpilePackages: ["@rs/ui", "@rs/api-client"],
  headers() {
    return Promise.resolve([
      {
        source: "/_next/static/:path*",
        headers: [...HEADER_STATIS, { key: "Content-Security-Policy", value: buatCsp(null) }],
      },
    ]);
  },
};

export default konfigurasi;
